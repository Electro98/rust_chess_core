use std::{fmt::Display, ops::AddAssign};

use crate::core::engine::{CheckType, Game, GameEndState, Move, MoveType};

#[derive(Debug, Default)]
pub struct PERFResult {
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
    pub fn combine(self, other: PERFResult) -> Self {
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
            MoveType::EnPassantCapture(_, _) => {
                result.captures += 1;
                result.en_passaunt += 1;
            }
        }
        match _move.check() {
            CheckType::None => (),
            CheckType::Direct => result.checks += 1,
            CheckType::Discovered => {
                result.checks += 1;
                result.discovery_checks += 1;
            }
            CheckType::Double => {
                result.checks += 1;
                result.double_checks += 1;
            }
        }
    }
    result
}

fn perf_test_step_undo(game: &mut Game, depth: usize) -> PERFResult {
    let possible_moves = game.get_possible_moves(true);
    if depth == 0 {
        PERFResult {
            all: 1,
            ..Default::default()
        }
    } else if depth == 1 {
        count_perf_result(possible_moves)
    } else {
        let mut result = PERFResult::default();
        for _move in possible_moves.into_iter() {
            result += if let Some(end_state) = game.execute(_move, true) {
                game.undo_last_move().expect("Failed to undo valid move");
                PERFResult {
                    checkmates: if matches!(end_state, GameEndState::CheckMate) {
                        1
                    } else {
                        0
                    },
                    ..Default::default()
                }
            } else {
                let result = perf_test_step_undo(game, depth - 1);
                game.undo_last_move().expect("Failed to undo valid move");
                result
            };
        }
        result
    }
}

fn perf_test_step_copy(game: Game, depth: usize) -> PERFResult {
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
                } else {
                    perf_test_step_copy(game, depth - 1)
                }
            })
            .reduce(PERFResult::combine)
            .expect("Result should exist")
    }
}

pub fn perf_test(
    fen_string: &str,
    depth: usize,
    expected: usize,
    detailed: bool,
    undo: bool,
) -> bool {
    let game = Game::from_fen(fen_string).unwrap();
    #[cfg(test)]
    println!(" - setup: | {fen_string} | depth: {depth} detailed: {detailed}");
    if detailed {
        let mut total: PERFResult = Default::default();
        for _move in game.get_possible_moves(true) {
            let mut temp_game = game.clone();
            temp_game.execute(_move.clone(), true);
            let result = if undo {
                perf_test_step_undo(&mut temp_game, depth - 1)
            } else {
                perf_test_step_copy(temp_game, depth - 1)
            };
            println!(" {_move} : {result}");
            total += result;
        }
        println!("+ total: {total}");
        total.all == expected
    } else {
        let result = perf_test_step_copy(game, depth);
        #[cfg(test)]
        println!(" details: {result}");
        result.all == expected
    }
}
