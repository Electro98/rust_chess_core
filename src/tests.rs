use std::{fmt::Display, ops::AddAssign};

use engine::{CheckType, Game, GameEndState, Move, MoveType, Piece};
use game::ui_board;
use rand::seq::IteratorRandom;
use utils::{between, compact_pos, distance, is_in_diagonal_line, is_in_straight_line, unpack_pos};

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

#[derive(Debug, Default)]
struct PERFResult {
    all: usize,
    captures: usize,
    en_passaunt: usize,
    castles: usize,
    promotions: usize,
    checks: usize,
    discovery_checks: usize,
    double_checks: usize,
    checkmates: usize,
}

impl PERFResult {
    fn combine(self, other: PERFResult) -> Self {
        PERFResult {
            all: self.all + other.all,
            captures: self.captures + other.captures,
            en_passaunt: self.en_passaunt + other.en_passaunt,
            castles: self.castles + other.castles,
            promotions: self.promotions + other.promotions,
            checks: self.checks + other.checks,
            discovery_checks: self.discovery_checks + other.discovery_checks,
            double_checks: self.double_checks + other.double_checks,
            checkmates: self.checkmates + other.checkmates,
        }
    }
}

impl AddAssign for PERFResult {
    fn add_assign(&mut self, rhs: Self) {
        self.all += rhs.all;
        self.captures += rhs.captures;
        self.en_passaunt += rhs.en_passaunt;
        self.castles += rhs.castles;
        self.promotions += rhs.promotions;
        self.checks += rhs.checks;
        self.discovery_checks += rhs.discovery_checks;
        self.double_checks += rhs.double_checks;
        self.checkmates += rhs.checkmates;
    }
}

impl Display for PERFResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - cp: {:<4} ep: {:<4} cs: {:<4} pr: {:<4} Ch: {:<4} dCh: {:<4} Ch2: {:<4} CM: {:4}",
            self.all,
            self.captures,
            self.en_passaunt,
            self.castles,
            self.promotions,
            self.checks,
            self.discovery_checks,
            self.double_checks,
            self.checkmates
        )
    }
}

fn count_perf_result(moves: Vec<Move>) -> PERFResult {
    let mut result = PERFResult {
        all: moves.len(),
        ..Default::default()
    };
    for _move in moves {
        match _move.move_type() {
            MoveType::QuietMove(_) => (),
            MoveType::Capture(_) => result.captures += 1,
            MoveType::Castling(_, _) => result.castles += 1,
            MoveType::PromotionQuiet(_, _) => result.promotions += 1,
            MoveType::PromotionCapture(_, _) => {
                result.captures += 1;
                result.promotions += 1;
            }
            MoveType::PawnDoublePush(_) => (),
            MoveType::EnPassantCapture(_) => {
                result.captures += 1;
                result.en_passaunt += 1;
            }
        }
        match _move.check() {
            engine::CheckType::None => (),
            engine::CheckType::Direct => result.checks += 1,
            engine::CheckType::Discovered => {
                result.checks += 1;
                result.discovery_checks += 1;
            }
            engine::CheckType::Double => {
                result.checks += 1;
                result.double_checks += 1;
            }
        }
    }
    result
}

fn perf_test_step(game: Game, depth: usize) -> PERFResult {
    let possible_moves = game.get_possible_moves(true);
    if depth == 0 {
        PERFResult {
            all: 1,
            ..Default::default()
        }
    } else if depth == 1 {
        count_perf_result(possible_moves)
    } else {
        possible_moves
            .into_iter()
            .map(|_move| {
                let mut game = game.light_clone();
                if let Some(end_state) = game.execute(_move, true) {
                    PERFResult {
                        checkmates: if matches!(end_state, GameEndState::CheckMate) {
                            1
                        } else {
                            0
                        },
                        ..Default::default()
                    }
                    // perf_test_step(game, 0)
                } else {
                    perf_test_step(game, depth - 1)
                }
            })
            .reduce(PERFResult::combine)
            .expect("Result should exist")
    }
}

fn perf_test(fen_string: &str, depth: usize, expected: usize, detailed: bool) -> bool {
    let game = Game::from_fen(fen_string).unwrap();
    println!(" - setup: | {fen_string} | depth: {depth} detailed: {detailed}");
    if detailed {
        let mut total: PERFResult = Default::default();
        for _move in game.get_possible_moves(true) {
            let mut temp_game = game.light_clone();
            temp_game.execute(_move.clone(), true);
            let result = perf_test_step(temp_game, depth - 1);
            println!(" {_move} : {result}");
            total += result;
        }
        println!("+ total: {total}");
        total.all == expected
    } else {
        let result = perf_test_step(game, depth);
        println!(" details: {result}");
        result.all == expected
    }
}

#[test]
fn move_generation() {
    let perf_setup: [(&str, Vec<usize>); 2] = [
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            vec![
                1,
                20,
                400,
                8902,
                197_281,
                4_865_609,
                119_060_324,
                3_195_901_860,
            ],
        ),
        (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ",
            vec![1, 48, 2039, 97_862, 4_085_603, 193_690_690, 8_031_647_685],
        ),
    ];
    for (fen_string, results) in perf_setup {
        for (depth, expected) in results.iter().enumerate().skip(2).take(6) {
            assert!(
                perf_test(fen_string, depth, *expected, true),
                "Result don't match up"
            )
        }
    }
}
