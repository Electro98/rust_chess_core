use crate::engine::{Color, PieceType};

#[derive(Clone, Debug, PartialEq)]
pub struct Figure {
    pub kind: PieceType,
    pub color: Color,
    pub last_move: bool,
    pub impose_check: bool,
    pub can_move: bool,
}

#[derive(Debug)]
pub enum Cell {
    Empty,
    Figure(Figure),
    Unknown,
}

pub trait ImplicitMove {
    fn promotion(&self) -> bool;
    fn set_promotion_type(&mut self, kind: PieceType);
}

#[derive(Clone)]
pub struct Move<T: ImplicitMove> {
    pub from: (u32, u32),
    pub to: (u32, u32),
    pub _move: T,
}

pub trait MatchInterface<T: ImplicitMove> {
    fn current_board(&self) -> Vec<Vec<Cell>>;
    fn possible_moves(&self, file: u32, rank: u32) -> Option<Vec<Move<T>>>;
    fn execute_move(&mut self, _move: Move<T>);
    fn wait_move(&self);
    // info
    fn current_player(&self) -> Color;
    fn checked(&self) -> bool;
    fn game_ended(&self) -> bool;
}
