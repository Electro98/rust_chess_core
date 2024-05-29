mod definitions;
pub mod engine;
mod game;
pub mod server;
pub mod utils;
pub use definitions::{Cell, DefaultMove, Figure, GameState, MatchInterface};
pub use engine::{Color, PieceType};
pub use game::Game;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use engine::Piece;

    use self::engine::Board;

    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_iters() {
        let board = Board::default();
        let mut iter = board.iter();
        let mut iter_pieces = board.iter_pieces();
        for file in 0..8u8 {
            for rank in 0..8u8 {
                let pos = rank << 4 | file;
                let code = board.inside()[pos as usize];
                let piece = Piece::from_code(code, pos);
                let iter_code = iter.next();
                let iter_piece = iter_pieces.next();
                assert!(iter_piece.is_some(), "Iterator is exhausted too soon!");
                assert!(iter_code.is_some(), "Raw iterator is exhausted too soon!");
                assert!(piece == iter_piece.unwrap(), "Piece is different from for loop!");
                assert!(code == iter_code.unwrap(), "Code is different from for loop!");
            }
        }
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
