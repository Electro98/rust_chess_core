use crate::core::definitions::{Cell, Figure};

use self::core::engine::{Board, Game, GameEndState, Piece};
use self::core::game::ui_board;
use self::core::utils::{between, compact_pos, is_in_diagonal_line, is_in_straight_line};
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
    ($fen_string:literal $($(#[$attr:meta])* $name:ident: $value:expr)*) => {
    $(
        #[test]
        $(#[$attr])*
        fn $name() {
            let fen_string = $fen_string;
            let (expected, depth) = $value;
            assert!(perf_test(fen_string, depth, expected, true, true), "Results don't match up");
        }
    )*
    }
}

macro_rules! perft_suit {
    ($($(#[$attr:meta])* $name:ident: $value:expr)*) => {
    $(
        #[test]
        $(#[$attr])*
        fn $name() {
            let (fen_string, depth, expected) = $value;
            assert!(perf_test(fen_string, depth, expected, true, true), "Results don't match up");
        }
    )*
    };
}

perf_tests! {
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    perft_base_2: (400, 2)
    perft_base_3: (8902, 3)
    perft_base_4: (197_281, 4)
    perft_base_5: (4_865_609, 5)
    #[ignore="slow"] perft_base_6: (119_060_324, 6)
    #[ignore="slow"] perft_base_7: (3_195_901_860, 7)
}

perf_tests! {
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "
    perft_kiwipete_2: (2039, 2)
    perft_kiwipete_3: (97_862, 3)
    perft_kiwipete_4: (4_085_603, 4)
    #[ignore="slow"] perft_kiwipete_5: (193_690_690, 5)
    #[ignore="slow"] perft_kiwipete_6: (8_031_647_685, 6)
}

perf_tests! {
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"
    perft_pos3_1: (14, 1)
    perft_pos3_2: (191, 2)
    perft_pos3_3: (2812, 3)
    perft_pos3_4: (43_238, 4)
    perft_pos3_5: (674_624, 5)
    perft_pos3_6: (11_030_083, 6)
    #[ignore="slow"] perft_pos3_7: (178_633_661, 7)
    #[ignore="slow"] perft_pos3_8: (3_009_794_393, 8)
}

perf_tests! {
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"
    perft_pos4_2: (264, 2)
    perft_pos4_3: (9467, 3)
    perft_pos4_4: (422_333, 4)
    #[ignore="slow"] perft_pos4_5: (15_833_292, 5)
    #[ignore="slow"] perft_pos4_6: (706_045_033, 6)
}

perf_tests! {
    "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"
    perft_pos4_mirror_2: (264, 2)
    perft_pos4_mirror_3: (9467, 3)
    perft_pos4_mirror_4: (422_333, 4)
    #[ignore="slow"] perft_pos4_mirror_5: (15_833_292, 5)
    #[ignore="slow"] perft_pos4_mirror_6: (706_045_033, 6)
}

perf_tests! {
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"
    perft_pos5_1: (44, 1)
    perft_pos5_2: (1486, 2)
    perft_pos5_3: (62_379, 3)
    perft_pos5_4: (2_103_487, 4)
    #[ignore="slow"] perft_pos5_5: (89_941_194, 5)
}

perf_tests! {
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
    perft_pos6_1: (46, 1)
    perft_pos6_2: (2079, 2)
    perft_pos6_3: (89_890, 3)
    perft_pos6_4: (3_894_594, 4)
    #[ignore="slow"] perft_pos6_5: (164_075_551, 5)
    #[ignore="slow"] perft_pos6_6: (6_923_051_137, 6)
}

// Specials
perft_suit! {
    special_1: ("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1134888)
    special_2: ("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1274206)
    special_3: ("8/8/8/8/k1p4R/8/3P4/3K4 w - - 0 1", 6, 1134888)
    special_4: ("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1440467)
    special_5: ("8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1", 6, 1440467)
    special_6: ("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1015133)
    special_7: ("8/b2p2k1/8/2P5/8/4K3/8/8 b - - 0 1", 6, 1015133)
    special_8: ("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661072)
    special_9: ("4k2r/8/8/8/8/8/8/5K2 b k - 0 1", 6, 661072)
    special_10: ("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803711)
    special_11: ("r3k3/8/8/8/8/8/8/3K4 b q - 0 1", 6, 803711)
}

// en passant capture checks opponent
perft_suit! {
    enpassant_check_1: ("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1440467)
    enpassant_check_2: ("8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1", 6, 1440467)
}

// avoid illegal ep(thanks to Steve Maughan)
perft_suit! {
    illegal_ep_1: ("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1134888)
    illegal_ep_2: ("8/8/8/8/k1p4R/8/3P4/3K4 w - - 0 1", 6, 1134888)
    illegal_ep_3: ("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1015133)
    illegal_ep_4: ("8/b2p2k1/8/2P5/8/4K3/8/8 b - - 0 1", 6, 1015133)
}

// short castling gives check
perft_suit! {
    short_castling_check_1: ("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661072)
    short_castling_check_2: ("4k2r/8/8/8/8/8/8/5K2 b k - 0 1", 6, 661072)
}

// long castling gives check
perft_suit! {
    long_castling_check_1: ("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803711)
    long_castling_check_2: ("r3k3/8/8/8/8/8/8/3K4 b q - 0 1", 6, 803711)
}

// castling(including losing cr due to rook capture)
perft_suit! {
    castling_1: ("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1274206)
    castling_2: ("r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1", 4, 1274206)
}

// castling prevented
perft_suit! {
    castling_prevented_1: ("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4, 1720476)
    castling_prevented_2: ("r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1", 4, 1720476)
}
//  promote out of check
perft_suit! {
    promote_out_of_check_1: ("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6, 3821001)
    promote_out_of_check_2: ("3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1", 6, 3821001)
}

// "# discovered check
perft_suit! {
    discovered_check_1: ("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5, 1004658)
    discovered_check_2: ("5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1", 5, 1004658)
}

// "# promote to give check
perft_suit! {
    promote_check_1: ("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6, 217342)
    promote_check_2: ("8/k7/8/8/8/8/1p6/4K3 b - - 0 1", 6, 217342)
}

// "# underpromote to check
perft_suit! {
    underpromote_check_1: ("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6, 92683)
    underpromote_check_2: ("8/8/8/8/8/k7/p1K5/8 b - - 0 1", 6, 92683)
}

// "# self stalemate
perft_suit! {
    self_stalemate_1: ("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6, 2217)
    self_stalemate_2: ("8/8/8/8/8/p7/8/k1K5 b - - 0 1", 6, 2217)
}

// stalemate/checkmate:
perft_suit! {
    stalemate_checkmate_1: ("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7, 567584)
    stalemate_checkmate_2: ("8/8/8/8/1k6/8/K1p5/8 b - - 0 1", 7, 567584)
}

// double check
perft_suit! {
    double_check_1: ("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4, 23527)
}

// short castling impossible although the rook never moved away from its corner
perft_suit! {
    impossible_short_castling_1: ("1k6/1b6/8/8/7R/8/8/4K2R b K - 0 1", 5, 1063513)
    impossible_short_castling_2: ("4k2r/8/8/7r/8/8/1B6/1K6 w k - 0 1", 5, 1063513)
}

// long castling impossible although the rook never moved away from its corner
perft_suit! {
    impossible_long_castling_1: ("1k6/8/8/8/R7/1n6/8/R3K3 b Q - 0 1", 5, 346695)
    impossible_long_castling_2: ("r3k3/8/1N6/r7/8/8/8/1K6 w q - 0 1", 5, 346695)
}
