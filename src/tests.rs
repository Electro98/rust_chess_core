use engine::Piece;
use game::ui_board;

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

fn player_board(board: &Board, player: Color) -> Vec<Vec<Cell>> {
    let mask = board.obstruct_board(player);
    let mut vec_board = Vec::with_capacity(8);
    for rank in 0..8u8 {
        let mut row = Vec::with_capacity(8);
        for file in 0..8u8 {
            let pos = (rank << 4) + file;
            let piece = Piece::from_code(board.inside()[pos as usize], pos);
            row.push(if !mask[file as usize][rank as usize] {
                Cell::Unknown
            } else if piece.type_() == PieceType::EmptySquare {
                Cell::Empty
            } else {
                Cell::Figure(Figure {
                    kind: piece.type_(),
                    color: piece.color(),
                    last_move: false,
                    impose_check: false,
                    can_move: true,
                })
            });
        }
        vec_board.push(row);
    }
    vec_board
}

#[test]
fn obstruction() {
    let board = Board::default();
    assert!(player_board(&board, Color::White) == ui_board(&board.clone().hide_and_obstruct(Color::White)), "White board are obstructed incorrectly!");
    assert!(player_board(&board, Color::Black) == ui_board(&board.clone().hide_and_obstruct(Color::Black)), "Black board are obstructed incorrectly!");
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
