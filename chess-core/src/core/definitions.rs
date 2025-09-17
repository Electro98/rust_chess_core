use crate::core::engine::{Color, PieceType};

// TODO: Delete!
#[derive(Clone, Debug, PartialEq)]
pub struct Figure {
    pub kind: PieceType,
    pub color: Color,
    pub last_move: bool,
    pub impose_check: bool,
    pub can_move: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Figure(Figure),
    Unknown,
}
