use std::sync::mpsc::{Receiver, Sender};
use crate::board::Board;
use crate::move_gen::ply::Ply;
use crate::search::SearchCommand;
use crate::uci;
use crate::uci::{UciCommand};

/// The main character in this project!
/// The Ladybug struct acts as the UCI client and can receive and handle UCI commands.
pub struct Ladybug {
   /// The current board position.
    board: Board,
    /// The current state of Ladybug.
    state: State,
    /// Used to send commands to the search thread.
    search_command_sender: Sender<SearchCommand>,
    /// Used to send output to the console.
    console_output_sender: Sender<String>,
    /// Used to receive input from both the console and the search thread.
    input_receiver: Receiver<String>,
}

/// The possible states of Ladybug.
enum State {
    Idle,
    GoPerft,
}

impl Ladybug {
    /// Constructs Ladybug.
    pub fn new(search_command_sender: Sender<SearchCommand>, console_output_sender: Sender<String>, input_receiver: Receiver<String>) -> Self {
        Self {
            board: Board::default(),
            state: State::Idle,
            search_command_sender,
            console_output_sender,
            input_receiver
        }
    }
    
    /// Starts running Ladybug.
    pub fn run(&mut self) {
        loop {
            // blocks until Ladybug receives input
            let input = self.input_receiver.recv();

            // if the input thread closes the connection, Ladybug must not continue running
            if input.is_err() {
                panic!("The input thread has unexpectedly closed the channel connection.")
            }

            // get the input string from the result
            let input = input.unwrap();

            // try to parse the uci command
            let uci_command = uci::parse_uci(input);

            let uci_command = match uci_command {
                // if the uci command cannot be parsed, send the error message to the output thread
                Err(message) => {
                    self.send_output(message);
                    continue;
                }
                Ok(command) => command
            };

            // delegate the handling of the uci command to the respective method
            match uci_command {
                UciCommand::Uci => self.handle_uci(),
                UciCommand::IsReady => self.handle_is_ready(),
                UciCommand::Position(args) => self.handle_position(args),
                UciCommand::GoPerft(depth) => self.handle_go_perft(depth),
                UciCommand::Quit => {
                    self.handle_quit();
                    break;
                }
                UciCommand::Help => self.handle_help(),
                UciCommand::Display => self.handle_display()
            }
        }
    }

    /// Sends the given String to the output thread.
    fn send_output(&self, output: String) {
        let send_result = self.console_output_sender.send(output);

        // if the output thread closes the connection, Ladybug must not continue running
        if send_result.is_err() {
            panic!("The output thread has unexpectedly closed the channel connection.")
        }
    }

    /// Handles the "uci" command.
    fn handle_uci(&self) {
        self.send_output(format!("id name Ladybug 0.1.0"));
        self.send_output(format!("id author Felix O."));
        self.send_output( String::from("uciok"));
    }

    /// Handles the "isready" command.
    fn handle_is_ready(&self) {
        self.send_output(String::from("readyok"));
    }

    /// Handles the "position" command.
    fn handle_position(&mut self, args: Vec<String>) {
        if args.is_empty() {
            self.send_output(String::from("info string unknown command"));
            return;
        }

        let mut fen = String::from("");

        // build the fen string from the provided args
        match args[0].as_str() {
            "startpos" => {
                fen += "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
            }
            "fen" => {
                for (index, arg) in args.iter().enumerate() {
                    if index == 0 {
                        continue;
                    }
                    if arg == "moves" {
                        break;
                    }
                    fen += " ";
                    fen += arg.as_str();
                }
            }
            _other => {
                self.send_output(String::from("info string unknown command"));
                return;
            }
        };

        // try to parse the fen
        let board = Board::from_fen(fen.as_str());
        if board.is_err() {
            self.send_output(String::from("info string invalid fen"));
            return;
        }
        let mut board = board.unwrap();

        // split the args vector to only contain the moves
        let moves_index = args.iter().position(|r| r == "moves");
        if moves_index.is_none() {
            // command contains no moves - finish
            self.board = board;
            return;
        }
        let moves_index = moves_index.unwrap() + 1;
        if moves_index > args.len() {
            return;
        }
        let (_, moves) = args.split_at(moves_index);

        // loop over moves strings and try to make the moves on the board
        for move_string in moves {
            let ply = Ply::from_string(move_string, board.position);
            match ply {
                Some(ply) => board = board.make_move(ply),
                None => {
                    self.send_output(String::from("info string invalid moves"));
                    return;
                }
            }
        }

        self.board = board;
    }

    /// Handles the "go perft <depth>" command
    fn handle_go_perft(&self, depth_str: String) {
        let depth = depth_str.parse::<u64>();
        match depth {
            Err(_) => {
                self.send_output(String::from("info string unknown command"));
            }
            Ok(depth) => {
                //perft(self.board.position, depth);
            }
        }
    }

    /// Handles the "quit" command.
    fn handle_quit(&self) {
        self.send_output(String::from("quit"));
    }

    /// Handles the "help" command.
    fn handle_help(&self) {
        self.send_output(String::from("Ladybug is a free and UCI compatible chess engine."));
        self.send_output(String::from("Currently, Ladybug only implements a subset of the UCI protocol:"));
        self.send_output(String::from("uci                              : Ask Ladybug if she supports UCI"));
        self.send_output(String::from("isready                          : Synchronize Ladybug with the GUI"));
        self.send_output(String::from("position fen <fen> moves <moves> : Setup the board position"));
        self.send_output(String::from("quit                             : Quit Ladybug"));
        self.send_output(String::from("display                          : Print the fen of the current position"));
    }

    /// Handles the "display" command.
    fn handle_display(&self) {
        self.send_output(self.board.to_fen());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    use std::{thread};
    use crate::ladybug::Ladybug;
    use crate::lookup::LOOKUP_TABLE;
    use crate::lookup::lookup_table::LookupTable;
    use crate::search::{Search, SearchCommand};

    /// Creates a new Ladybug thread and returns the input_sender and output_receiver.
    fn setup() -> (Sender<String>, Receiver<String>) {
        initialize_lookup_table();

        // create search_command_sender and search_command_receiver so that the ladybug thread can send commands to the search thread
        let (search_command_sender, search_command_receiver): (Sender<SearchCommand>, Receiver<SearchCommand>) = mpsc::channel();

        // create input_sender and input_receiver so that the input thread can send input to the ladybug thread
        let (input_sender, input_receiver): (Sender<String>, Receiver<String>) = mpsc::channel();

        // create output_sender and output_receiver so that the ladybug thread can send output to the output thread.
        let (output_sender, output_receiver): (Sender<String>, Receiver<String>) = mpsc::channel();
        
        // initialize the search
        let mut search = Search::new(search_command_receiver, input_sender.clone());
        
        // spawn the search thread
        thread::spawn(move || search.run());

        let mut ladybug = Ladybug::new(search_command_sender, output_sender.clone(), input_receiver);

        // spawn the Ladybug thread
        thread::spawn(move || ladybug.run());

        (input_sender, output_receiver)
    }

    // helper function to initialize the lookup table
    fn initialize_lookup_table() {
        let mut lookup = LookupTable::default();
        lookup.initialize_tables();
        let _ = LOOKUP_TABLE.set(lookup);
    }

    #[test]
    fn test_ladybug_with_invalid_uci_input_prints_error_message() {
        let (input_sender, output_receiver) = setup();

        let _ = input_sender.send(String::from("Not Uci"));
        assert_eq!("info string unknown command", output_receiver.recv().unwrap());

        let _ = input_sender.send(String::from("       "));
        assert_eq!("info string unknown command", output_receiver.recv().unwrap());

        let _ = input_sender.send(String::from("123456789"));
        assert_eq!("info string unknown command", output_receiver.recv().unwrap());

        let _ = input_sender.send(String::from("position test"));
        assert_eq!("info string unknown command", output_receiver.recv().unwrap());

        let _ = input_sender.send(String::from("position fen this is invalid fen"));
        assert_eq!("info string invalid fen", output_receiver.recv().unwrap());
    }

    #[test]
    fn test_ladybug_for_uci() {
        let (input_sender, output_receiver) = setup();

        let _ = input_sender.send(String::from("uci"));
        assert_eq!("id name Ladybug 0.1.0", output_receiver.recv().unwrap());
        assert_eq!("id author Felix O.", output_receiver.recv().unwrap());
        assert_eq!("uciok", output_receiver.recv().unwrap());
    }

    #[test]
    fn test_ladybug_for_isready() {
        let (input_sender, output_receiver) = setup();

        let _ = input_sender.send(String::from("isready"));
        assert_eq!("readyok", output_receiver.recv().unwrap());
    }

    #[test]
    fn test_ladybug_for_position() {
        let (input_sender, output_receiver) = setup();

        let _ = input_sender.send(String::from("position startpos"));
        let _ = input_sender.send(String::from("display"));
        assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", output_receiver.recv().unwrap());

        let _ = input_sender.send(String::from("position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
        let _ = input_sender.send(String::from("display"));
        assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", output_receiver.recv().unwrap());

        let _ = input_sender.send(String::from("position startpos moves e2e4 c7c5 c2c3 b8c6 d2d4"));
        let _ = input_sender.send(String::from("display"));
        assert_eq!("r1bqkbnr/pp1ppppp/2n5/2p5/3PP3/2P5/PP3PPP/RNBQKBNR b KQkq d3 0 3", output_receiver.recv().unwrap());

        let _ = input_sender.send(String::from("position fen r1bqkbnr/pp1ppppp/2n5/2p5/3PP3/2P5/PP3PPP/RNBQKBNR b KQkq d3 0 3 moves c5d4 h2h4 d4c3 g1f3 c3b2 f1b5 b2c1q"));
        let _ = input_sender.send(String::from("display"));
        assert_eq!("r1bqkbnr/pp1ppppp/2n5/1B6/4P2P/5N2/P4PP1/RNqQK2R w KQkq - 0 7", output_receiver.recv().unwrap());
    }

    #[test]
    fn test_ladybug_for_quit() {
        let (input_sender, output_receiver) = setup();

        let _ = input_sender.send(String::from("quit"));
        assert_eq!("quit", output_receiver.recv().unwrap());
    }

    #[test]
    fn test_ladybug_for_help() {
        let (input_sender, output_receiver) = setup();

        let _ = input_sender.send(String::from("help"));
        assert_eq!("Ladybug is a free and UCI compatible chess engine.", output_receiver.recv().unwrap());
        assert_eq!("Currently, Ladybug only implements a subset of the UCI protocol:", output_receiver.recv().unwrap());
        assert_eq!("uci                              : Ask Ladybug if she supports UCI", output_receiver.recv().unwrap());
        assert_eq!("isready                          : Synchronize Ladybug with the GUI", output_receiver.recv().unwrap());
        assert_eq!("position fen <fen> moves <moves> : Setup the board position", output_receiver.recv().unwrap());
        assert_eq!("quit                             : Quit Ladybug", output_receiver.recv().unwrap());
        assert_eq!("display                          : Print the fen of the current position", output_receiver.recv().unwrap());
    }

    #[test]
    fn test_ladybug_for_display() {
        let (input_sender, output_receiver) = setup();

        let _ = input_sender.send(String::from("position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
        let _ = input_sender.send(String::from("display"));
        assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", output_receiver.recv().unwrap());

        let _ = input_sender.send(String::from("position fen r1bqk1nr/ppp1bBpp/3p4/n7/3PP3/1Q3N2/P4PPP/RNB1K2R b KQkq - 0 9"));
        let _ = input_sender.send(String::from("display"));
        assert_eq!("r1bqk1nr/ppp1bBpp/3p4/n7/3PP3/1Q3N2/P4PPP/RNB1K2R b KQkq - 0 9", output_receiver.recv().unwrap());
    }
}
