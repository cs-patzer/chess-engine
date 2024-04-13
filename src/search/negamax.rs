use crate::board::position::Position;
use crate::{evaluation, move_gen};
use crate::search::Search;

impl Search {
    /// Search the given position with iterative deepening.
    pub fn iterative_search(&mut self, position: Position, max_depth: u64) {
        // initialize the best move with a random one, in case the search is stopped immediately
        let mut prev_best_move = move_gen::generates_moves(position)[0];

        for depth in 1..=max_depth {
            self.negamax(position, depth, 0);

            match self.best_move {
                None => break, // the search was stopped before it could complete
                Some(best_move) => {
                    // set the best move of the previous iteration to the new best move
                    prev_best_move = best_move;
                    self.best_move = None;
                }
            }
        }

        self.send_output(format!("bestmove {}", prev_best_move));
    }

    /// A basic implementation of the [negamax](https://www.chessprogramming.org/Negamax) algorithm.
    ///
    /// Instead of implementing two routines for the maximizing and minimizing players, this method
    /// negates the scores for each recursive call, making minimax easier to implement.
    pub fn negamax(&mut self, position: Position, depth: u64, ply_num: u64) -> i32 {
        // the maximum score that can be reached in this position
        let mut max_score = evaluation::NEGATIVE_INFINITY;

        // generate all legal moves for the current position
        let moves = move_gen::generates_moves(position);

        // if there are no legal moves, check for mate or stalemate
        if moves.is_empty() {
            return if position.is_in_check(position.color_to_move) {
                // In case of checkmate, return a large negative number.
                // By adding a large number (larger than the worth of a queen) for each ply in the search tree, 
                // and thus decreasing the penalty for getting checkmated, the engine is incentivised to sacrifice material in order to delay checkmate.
                // It will also prefer shorter mates when being on the winning side.
                evaluation::NEGATIVE_INFINITY + (ply_num as i32 * 5000)
            } else {
                0
            };
        }

        // if depth 0 is reached, break out of the recursion by returning the static evaluation of the position
        if depth == 0 {
            return evaluation::evaluate(position);
        }

        // make all moves and call negamax recursively for the arising positions
        for ply in moves {
            // the score of the position arising after playing the move
            let score = -self.negamax(position.make_move(ply), depth - 1, ply_num + 1);
            // check if the score of the position is better than the current max score
            if score > max_score {
                // update the max score
                max_score = score;
                // we're at the root node - update the best move
                if ply_num == 0 {
                    self.best_move = Some(ply);
                }
            }
        }
        max_score
    }
}