mod definitions;
pub mod engine;
mod game;
pub mod server;
mod utils;
pub use definitions::{Cell, DefaultMove, Figure, GameState, MatchInterface};
pub use engine::{Color, PieceType};
pub use game::Game;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use self::engine::Board;

    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    #[ignore = "slow"]
    fn random_moves_game() {
        for _ in 0..100 {
            let mut game: Game = Default::default();
            while !matches!(game.make_random_move(), GameState::Finished) {
                // do nothing
            }
        }
    }

    #[test]
    fn move_generation() {
        let fen_string = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let (board, player, last_move) = Board::from_FEN(fen_string);
        let possible_moves = board.get_possible_moves(player, last_move, true);
    }
}
