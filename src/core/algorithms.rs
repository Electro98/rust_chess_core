use crate::core::engine::{Board, Game, Move};

// Only for server-side/offline match
pub trait Algorithm {
    fn solve(&self, game: &Game) -> Move;
}

pub type EvaluationFunc = dyn Fn(&Board) -> i32;

pub struct MinMaxBot {
    max_depth: u32,
    evaluate_fn: &'static EvaluationFunc,
}

impl MinMaxBot {
    fn new(max_depth: u32, evaluate_fn: &'static EvaluationFunc) -> Self {
        MinMaxBot {
            max_depth,
            evaluate_fn,
        }
    }
}

impl Algorithm for MinMaxBot {
    fn solve(&self, game: &Game) -> Move {
        todo!()
    }
}
