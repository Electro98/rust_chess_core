use self::core::engine::{Board, CheckType, Game, GameEndState, Move, MoveType, Piece};
use self::core::game::ui_board;
use self::core::utils::{
    between, compact_pos, distance, is_in_diagonal_line, is_in_straight_line, unpack_pos,
};
use self::utils::perf_test;
use rand::seq::IteratorRandom;

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
    for rank in 0..8u8 {
        for file in 0..8u8 {
            let pos = rank << 4 | file;
            let code = board.inside()[pos as usize];
            let piece = Piece::from_code(code, pos);
            let iter_code = iter.next();
            let iter_piece = iter_pieces.next();
            assert!(iter_piece.is_some(), "Iterator is exhausted too soon!");
            assert!(iter_code.is_some(), "Raw iterator is exhausted too soon!");
            assert!(
                piece == iter_piece.unwrap(),
                "Piece is different from for loop!"
            );
            assert!(
                code == iter_code.unwrap(),
                "Code is different from for loop!"
            );
        }
    }
}

#[test]
fn math() {
    assert!(is_in_diagonal_line(71, 116), "This line is straight");
    let cells: Vec<_> = between(71, 116).collect();
    assert!(cells == vec![86, 101]);
}

#[test]
fn straight_line() {
    const STRAIGHT_LINE: [u8; 9] = [0x02, 0x12, 0x20, 0x21, 0x22, 0x23, 0x24, 0x32, 0x42];
    let test_piece = 0x22;
    for rank in 0..5 {
        for file in 0..5 {
            let pos = compact_pos(rank, file);
            println!("Current position: 0x{pos:x}");
            assert!(STRAIGHT_LINE.contains(&pos) == is_in_straight_line(test_piece, pos));
            assert!(STRAIGHT_LINE.contains(&pos) == is_in_straight_line(pos, test_piece));
        }
    }
}

#[test]
fn diagonal_line() {
    const DIAGONAL_LINE: [u8; 9] = [0x00, 0x04, 0x11, 0x13, 0x22, 0x31, 0x33, 0x40, 0x44];
    let test_piece = 0x22;
    for rank in 0..5 {
        for file in 0..5 {
            let pos = compact_pos(rank, file);
            println!("Current position: 0x{pos:x}");
            assert!(DIAGONAL_LINE.contains(&pos) == is_in_diagonal_line(test_piece, pos));
            assert!(DIAGONAL_LINE.contains(&pos) == is_in_diagonal_line(pos, test_piece));
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
    assert!(
        player_board(&board, Color::White)
            == ui_board(&board.clone().hide_and_obstruct(Color::White)),
        "White board are obstructed incorrectly!"
    );
    assert!(
        player_board(&board, Color::Black)
            == ui_board(&board.clone().hide_and_obstruct(Color::Black)),
        "Black board are obstructed incorrectly!"
    );
}

#[test]
#[ignore = "slow"]
fn random_moves_game() {
    fn make_random_move(game: &mut Game) -> Option<GameEndState> {
        game.execute(
            game.get_possible_moves(true)
                .into_iter()
                .choose(&mut rand::thread_rng())
                .unwrap(),
            true,
        )
    }
    for _ in 0..100 {
        let mut game: Game = Default::default();
        while make_random_move(&mut game).is_none() {
            // do nothing
        }
    }
}

#[test]
fn fen_parsing() {
    const FENs: [&str; 1] = ["rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"];
    for setup in FENs {
        let _game = Game::from_fen(setup).unwrap();
    }
}

macro_rules! perf_tests {
    ($fen_string:literal $($name:ident: $value:expr)*) => {
    $(
        #[test]
        fn $name() {
            let fen_string = $fen_string;
            let (expected, depth) = $value;
            assert!(perf_test(fen_string, depth, expected, true, true), "Results don't match up");
        }
    )*
    }
}

perf_tests! {
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    perft_base_1: (400, 2)
    perft_base_2: (8902, 3)
    perft_base_3: (197_281, 4)
    perft_base_4: (4_865_609, 5)
    perft_base_5: (119_060_324, 6)
    // perft_base_6: (3_195_901_860, 7)
}

perf_tests! {
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "
    perft_kiwipete_1: (2039, 2)
    perft_kiwipete_2: (97_862, 3)
    perft_kiwipete_3: (4_085_603, 4)
    perft_kiwipete_4: (193_690_690, 5)
    // perft_kiwipete_5: (8_031_647_685, 6)
}
