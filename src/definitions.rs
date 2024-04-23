use crate::engine::{Color, Move as BaseMove, PieceType};

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

#[derive(Clone, Debug)]
pub struct Move<T: ImplicitMove> {
    pub from: (u32, u32),
    pub to: (u32, u32),
    pub _move: T,
}

pub type DefaultMove = Move<BaseMove>;

pub trait MatchInterface<T: ImplicitMove> {
    fn current_board(&self) -> Vec<Vec<Cell>>;
    fn possible_moves(&self, file: u32, rank: u32) -> Option<Vec<Move<T>>>;
    fn execute_move(&mut self, _move: Move<T>) -> GameState;
    fn wait_move(&mut self) -> GameState;
    // info
    fn current_player(&self) -> Color;
    fn checked(&self) -> bool;
    fn game_ended(&self) -> bool;
}

pub enum GameState {
    PlayerMove(Color),
    DistantMove(Color),
    Finished,
}

// ---
// Implementation block
// ---

impl<T: ImplicitMove> Move<T> {
    pub fn is_promotion(&self) -> bool {
        self._move.promotion()
    }
    pub fn set_promotion_type(&mut self, kind: PieceType) {
        self._move.set_promotion_type(kind)
    }
}
