use crate::board::piece::Piece;
use crate::board::square::Square;

/// This struct represents a halfmove, also known as [ply](https://www.chessprogramming.org/Ply).
pub struct Ply {
    /// The source square.
    pub source: Square,
    /// The destination square.
    pub destination: Square,
    /// The type of the piece to move.
    pub piece: Piece,
    /// If the move is a capture move, this field will contain the type of the captured piece.
    pub captured_piece: Option<Piece>,
}