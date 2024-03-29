/// Represents a piece on a chessboard.
#[derive(Copy, Clone)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

/// The number of different pieces.
pub const NUM_PIECES: u8 = 6;

impl Piece {
    /// Returns the index of the piece.
    pub fn to_index(&self) -> u8 {
        *self as u8
    }
}

#[cfg(test)]
mod tests {
    use crate::board::piece::Piece::*;

    #[test]
    fn to_index_returns_correct_index() {
        assert_eq!(0, Pawn.to_index());
        assert_eq!(1, Knight.to_index());
        assert_eq!(2, Bishop.to_index());
        assert_eq!(3, Rook.to_index());
        assert_eq!(4, Queen.to_index());
        assert_eq!(5, King.to_index());
    }
}