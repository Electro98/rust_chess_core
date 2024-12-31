use engine::{Move, Piece};
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
    for _ in 0..100 {
        let mut game: Game = Default::default();
        while !matches!(game.make_random_move(), GameState::Finished) {
            // do nothing
        }
    }
}

#[test]
fn fen_parsing() {
    const FENs: [&str; 1] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    ];
    for setup in FENs {
        let (_board, _, _) = Board::from_fen(setup).unwrap();
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
    // discovery_checks: usize,
    // double_checks: usize,
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
            checkmates: self.checkmates + other.checkmates,
        }
    }
}

fn count_perf_result(moves: Vec<Move>) -> PERFResult {
    let mut result = PERFResult {
        all: moves.len(),
        ..Default::default()
    };
    for _move in moves {
        match _move {
            Move::NullMove => panic!("Generator created a NULL move."),
            Move::QuietMove(_, _) => (),
            Move::Capture(_, _) => result.captures += 1,
            Move::Castling(_, _, _) => result.castles += 1,
            Move::PromotionQuiet(_, _, _) => result.promotions += 1,
            Move::PromotionCapture(_, _, _) => {
                result.captures += 1;
                result.promotions += 1;
            },
            Move::PawnDoublePush(_, _) => (),
            Move::EnPassantCapture(_, _) => result.en_passaunt += 1,
        }
    }
    result
}

fn perf_test_step(board: &Board, player: Color, last_move: Move, depth: usize) -> PERFResult {
    let possible_moves = board.get_possible_moves(player, last_move, true);
    if depth == 0 {
        PERFResult { all: 1, ..Default::default() }
    } else if depth == 1 {
        count_perf_result(possible_moves)
    } else {
        possible_moves.into_iter().map(|_move| {
            let mut board = board.clone();
            board.execute(_move.clone());
            perf_test_step(&board, player.opposite(), _move, depth - 1)
        }).reduce(PERFResult::combine).expect("Result should exist")
    }
}

#[test]
fn move_generation() {
    let PERF_SETUP: [(&str, Vec<usize>); 1] = [
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", vec![1, 20, 400, 8902, 197_281, 4_865_609, 119_060_324, 3_195_901_860])
    ];
    for (fen_string, results) in PERF_SETUP {
        let (board, player, last_move) = Board::from_fen(fen_string).unwrap();
        for (depth, expected) in results.iter().enumerate() {
            let result = perf_test_step(&board, player, last_move.clone(), depth);
            let nodes_count = result.all;
            println!("Step {depth} - Result: {nodes_count} - Expected: {expected}");
            println!("Details: {result:#?}");
            assert!(result.all == *expected, "Results don't match up");
        } 
    }
}
