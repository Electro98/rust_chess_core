mod definitions;
pub mod engine;
mod game;
mod utils;
pub use definitions::{Cell, DefaultMove, Figure, GameState, MatchInterface};
pub use engine::{Color, PieceType};
pub use game::Game;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
