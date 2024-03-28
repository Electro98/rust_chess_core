use std::cmp::min;
use std::fs::{File, OpenOptions};
use std::io::Write;

use crate::engine::{Board, Color, Move as ImplMove, Piece, PieceType};
use crate::{Cell, Figure, MatchInterface, Move};

use rand::seq::{IteratorRandom, SliceRandom};

pub struct Game {
    board: Board,
    current_player: Color,
    history: Vec<Move<ImplMove>>,
    checked: bool,
    finished: bool,
}

impl Game {
    pub fn new(board: Board) -> Game {
        Game::with_player(board, Color::White)
    }

    fn with_player(board: Board, player: Color) -> Game {
        #[cfg(debug_assertions)]
        {
            let (checked, _) = board.is_checked(player.opposite());
            assert!(!checked, "King in danger before move!");
        }
        let (checked, _) = board.is_checked(player);
        Game {
            board,
            current_player: Color::White,
            history: Vec::new(),
            checked,
            finished: false,
        }
    }

    fn make_move(&mut self, impl_move: ImplMove) {
        self.board.execute(impl_move.clone());
        // let prev_player = self.current_player;
        self.current_player = if self.current_player == Color::White {
            Color::Black
        } else {
            Color::White
        };
        let (checked, king) = self.board.is_checked(self.current_player);
        self.checked = checked;
        let moves: Vec<_> = self
            .board
            .get_possible_moves(
                self.current_player,
                self.last_move().unwrap_or(ImplMove::NullMove),
            )
            .into_iter()
            .filter(|impl_move| {
                let mut board: Board = self.board.clone();
                board.execute(impl_move.clone());
                let (checked, _) = board.is_checked(self.current_player);
                !checked
            })
            .collect();
        self.finished = moves.is_empty();
        self.board.castling_rights(king);
    }

    pub fn last_move(&self) -> Option<ImplMove> {
        self.history
            .last()
            .and_then(|_move| Some(_move._move.clone()))
    }

    pub fn make_random_move(&mut self) {
        if self.finished {
            return;
        }
        let chosen_move = self
            .board
            .get_possible_moves(
                self.current_player,
                self.last_move().unwrap_or(ImplMove::NullMove),
            )
            .into_iter()
            .filter(|impl_move| {
                let mut board: Board = self.board.clone();
                board.execute(impl_move.clone());
                let (checked, _) = board.is_checked(self.current_player);
                !checked
            })
            .choose(&mut rand::thread_rng());

        if let Some(_move) = chosen_move {
            let wrapped_move = Move {
                from: {
                    let pos: u32 = match &_move {
                        ImplMove::QuietMove(piece, _) => piece,
                        ImplMove::Capture(piece, _) => piece,
                        ImplMove::Castling(piece, _, _) => piece,
                        ImplMove::PromotionQuiet(piece, _, _) => piece,
                        ImplMove::PromotionCapture(piece, _, _) => piece,
                        ImplMove::PawnDoublePush(piece, _) => piece,
                        ImplMove::EnPassantCapture(piece, _) => piece,
                        ImplMove::NullMove => panic!(),
                    }
                    .position() as u32;
                    ((pos & 0xf0) >> 4, pos & 0x0f)
                },
                to: {
                    let pos: u32 = match &_move {
                        ImplMove::QuietMove(_, pos) => *pos as u32,
                        ImplMove::Capture(_, piece) => piece.position() as u32,
                        ImplMove::Castling(_, _, piece) => piece.position() as u32,
                        ImplMove::PromotionQuiet(_, pos, _) => *pos as u32,
                        ImplMove::PromotionCapture(_, piece, _) => piece.position() as u32,
                        ImplMove::PawnDoublePush(_, pos) => *pos as u32,
                        ImplMove::EnPassantCapture(_, piece) => piece.position() as u32,
                        ImplMove::NullMove => panic!(),
                    };
                    ((pos & 0xf0) >> 4, pos & 0x0f)
                },
                _move: _move.clone(),
            };
            self.execute_move(wrapped_move);
        } else {
            self.finished = true;
        }
    }

    pub fn make_minimax_move(&mut self, file: &std::sync::Arc<std::sync::Mutex<File>>) {
        // nothing
        if self.finished {
            return;
        }
        let moves: Vec<_> = self
            .board
            .get_possible_moves(
                self.current_player,
                self.last_move().unwrap_or(ImplMove::NullMove),
            )
            .into_iter()
            .filter(|impl_move| {
                let mut board: Board = self.board.clone();
                board.execute(impl_move.clone());
                let (checked, _) = board.is_checked(self.current_player);
                !checked
            })
            .collect();
        let moves_count = moves.len();
        let mut complexity = 0;
        use std::time::Instant;
        let now = Instant::now();
        let chosen_move = alpha_beta(&self, moves, 5, &mut complexity);
        let elapsed = now.elapsed();

        // Save info
        {
            let mut file = file.lock().unwrap();
            writeln!(
                file,
                "{},{},{},{},{}",
                self.history.len(),
                material_advantage(&self.board, self.current_player),
                moves_count,
                complexity,
                elapsed.as_micros()
            )
            .expect("failed to write data");
            let _ = file.flush();
        }

        if let Some(_move) = chosen_move {
            let wrapped_move = Move {
                from: {
                    let pos: u32 = match &_move {
                        ImplMove::QuietMove(piece, _) => piece,
                        ImplMove::Capture(piece, _) => piece,
                        ImplMove::Castling(piece, _, _) => piece,
                        ImplMove::PromotionQuiet(piece, _, _) => piece,
                        ImplMove::PromotionCapture(piece, _, _) => piece,
                        ImplMove::PawnDoublePush(piece, _) => piece,
                        ImplMove::EnPassantCapture(piece, _) => piece,
                        ImplMove::NullMove => panic!(),
                    }
                    .position() as u32;
                    ((pos & 0xf0) >> 4, pos & 0x0f)
                },
                to: {
                    let pos: u32 = match &_move {
                        ImplMove::QuietMove(_, pos) => *pos as u32,
                        ImplMove::Capture(_, piece) => piece.position() as u32,
                        ImplMove::Castling(_, _, piece) => piece.position() as u32,
                        ImplMove::PromotionQuiet(_, pos, _) => *pos as u32,
                        ImplMove::PromotionCapture(_, piece, _) => piece.position() as u32,
                        ImplMove::PawnDoublePush(_, pos) => *pos as u32,
                        ImplMove::EnPassantCapture(_, piece) => piece.position() as u32,
                        ImplMove::NullMove => panic!(),
                    };
                    ((pos & 0xf0) >> 4, pos & 0x0f)
                },
                _move: _move.clone(),
            };
            self.execute_move(wrapped_move);
        } else {
            self.finished = true;
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new(Default::default())
    }
}

impl MatchInterface<ImplMove> for Game {
    fn current_board(&self) -> Vec<Vec<Cell>> {
        let mut board = Vec::with_capacity(8);
        for rank in 0..8u8 {
            let mut row = Vec::with_capacity(8);
            for file in 0..8u8 {
                let pos = (rank << 4) + file;
                let piece = Piece::from_code(self.board.inside()[pos as usize], pos);
                row.push(if piece.type_() == PieceType::EmptySquare {
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
            board.push(row);
        }
        board
    }

    fn possible_moves(&self, rank: u32, file: u32) -> Option<Vec<Move<ImplMove>>> {
        if self.finished {
            return None;
        }
        let pos = (rank << 4) as u8 | file as u8;
        let piece = Piece::from_code(self.board.inside()[pos as usize], pos);
        println!("Piece: {:?}", piece);
        if piece.type_() == PieceType::EmptySquare || piece.color() != self.current_player {
            return None;
        }
        let moves: Vec<_> = self
            .board
            .get_possible_moves(
                self.current_player,
                self.last_move().unwrap_or(ImplMove::NullMove),
            )
            .into_iter()
            .filter(|_move| match _move {
                ImplMove::NullMove => false,
                ImplMove::QuietMove(_piece, _) => &piece == _piece,
                ImplMove::Capture(_piece, _) => &piece == _piece,
                ImplMove::Castling(_piece, _, _) => &piece == _piece,
                ImplMove::PromotionQuiet(_piece, _, _) => &piece == _piece,
                ImplMove::PromotionCapture(_piece, _, _) => &piece == _piece,
                ImplMove::PawnDoublePush(_piece, _) => &piece == _piece,
                ImplMove::EnPassantCapture(_piece, _) => &piece == _piece,
            })
            .filter(|impl_move| {
                let mut board: Board = self.board.clone();
                board.execute(impl_move.clone());
                let (checked, _) = board.is_checked(self.current_player);
                !checked
            })
            .map(|_move| Move {
                from: (rank, file),
                to: {
                    let pos: u32 = match &_move {
                        ImplMove::QuietMove(_, pos) => *pos as u32,
                        ImplMove::Capture(_, piece) => piece.position() as u32,
                        ImplMove::Castling(_, _, piece) => piece.position() as u32,
                        ImplMove::PromotionQuiet(_, pos, _) => *pos as u32,
                        ImplMove::PromotionCapture(_, piece, _) => piece.position() as u32,
                        ImplMove::PawnDoublePush(_, pos) => *pos as u32,
                        ImplMove::EnPassantCapture(_, piece) => piece.position() as u32,
                        ImplMove::NullMove => panic!(),
                    };
                    ((pos & 0xf0) >> 4, pos & 0x0f)
                },
                _move,
            })
            .collect();
        if moves.is_empty() {
            None
        } else {
            Some(moves)
        }
    }

    fn execute_move(&mut self, _move: Move<ImplMove>) {
        self.make_move(_move._move.clone());
        self.history.push(_move);
    }

    fn wait_move(&mut self) {
        // nothing
        if self.finished {
            return;
        }
        let moves: Vec<_> = self
            .board
            .get_possible_moves(
                self.current_player,
                self.last_move().unwrap_or(ImplMove::NullMove),
            )
            .into_iter()
            .filter(|impl_move| {
                let mut board: Board = self.board.clone();
                board.execute(impl_move.clone());
                let (checked, _) = board.is_checked(self.current_player);
                !checked
            })
            .collect();
        let moves_count = moves.len();
        let mut complexity = 0;
        use std::time::Instant;
        let now = Instant::now();
        let chosen_move = alpha_beta(&self, moves, 3, &mut complexity);
        let elapsed = now.elapsed();

        // Save info
        // open file.csv -> save, try again
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("results.csv")
            .expect("Probably, we are dead!");
        writeln!(
            file,
            "{},{},{},{},{}",
            self.history.len(),
            material_advantage(&self.board, self.current_player),
            moves_count,
            complexity,
            elapsed.as_micros()
        )
        .expect("failed to write data");
        let _ = file.flush();

        if let Some(_move) = chosen_move {
            let wrapped_move = Move {
                from: {
                    let pos: u32 = match &_move {
                        ImplMove::QuietMove(piece, _) => piece,
                        ImplMove::Capture(piece, _) => piece,
                        ImplMove::Castling(piece, _, _) => piece,
                        ImplMove::PromotionQuiet(piece, _, _) => piece,
                        ImplMove::PromotionCapture(piece, _, _) => piece,
                        ImplMove::PawnDoublePush(piece, _) => piece,
                        ImplMove::EnPassantCapture(piece, _) => piece,
                        ImplMove::NullMove => panic!(),
                    }
                    .position() as u32;
                    ((pos & 0xf0) >> 4, pos & 0x0f)
                },
                to: {
                    let pos: u32 = match &_move {
                        ImplMove::QuietMove(_, pos) => *pos as u32,
                        ImplMove::Capture(_, piece) => piece.position() as u32,
                        ImplMove::Castling(_, _, piece) => piece.position() as u32,
                        ImplMove::PromotionQuiet(_, pos, _) => *pos as u32,
                        ImplMove::PromotionCapture(_, piece, _) => piece.position() as u32,
                        ImplMove::PawnDoublePush(_, pos) => *pos as u32,
                        ImplMove::EnPassantCapture(_, piece) => piece.position() as u32,
                        ImplMove::NullMove => panic!(),
                    };
                    ((pos & 0xf0) >> 4, pos & 0x0f)
                },
                _move: _move.clone(),
            };
            self.execute_move(wrapped_move);
        } else {
            self.finished = true;
        }
    }

    fn current_player(&self) -> Color {
        self.current_player
    }

    fn checked(&self) -> bool {
        self.checked
    }

    fn game_ended(&self) -> bool {
        self.finished
    }
}

fn material_advantage(board: &Board, player: Color) -> i32 {
    let mut material_difference: i32 = 0;
    let mut material_total = 0;
    let mut pawn_advantage = 0;
    for rank in 0..8u8 {
        for file in 0..8u8 {
            let pos = (rank << 4) + file;
            let piece = Piece::from_code(board.inside()[pos as usize], pos);
            let material = match piece.type_() {
                PieceType::Pawn => 100,
                PieceType::Knight => 325,
                PieceType::Bishop => 350,
                PieceType::Rook => 500,
                PieceType::Queen => 900,
                PieceType::King => 0,
                PieceType::Invalid => 0,
                PieceType::EmptySquare => 0,
            };
            material_total += material;
            material_difference += if piece.color() == player {
                material
            } else {
                -material
            };
            if piece.color() == player && piece.type_() == PieceType::Pawn {
                pawn_advantage += 1;
            }
        }
    }
    let ms = min(2400, material_difference.abs())
        + (material_difference.abs() * pawn_advantage * (8100 - material_total))
            / (6400 * (pawn_advantage + 1));
    let total_material_advantage = min(3100, ms);
    if material_difference >= 0 {
        total_material_advantage
    } else {
        -total_material_advantage
    }
}

fn min_max(
    game: &Game,
    moves: Vec<ImplMove>,
    depth: i32,
    moves_count: &mut i32,
) -> Option<ImplMove> {
    let opponent = game.current_player.opposite();
    let mut moves: Vec<_> = moves
        .into_iter()
        .filter_map(|_move| {
            let mut board_copy = game.board.clone();
            board_copy.execute(_move.clone());
            *moves_count += 1;
            let (checked, _) = board_copy .is_checked(game.current_player);
            if checked {
                return None;
            }
            let score = mini(&board_copy, depth, opponent, _move.clone(), moves_count);
            score.and_then(|score| Some((score, _move)))
        })
        .collect();
    moves.shuffle(&mut rand::thread_rng());
    moves
        .into_iter()
        .max_by(|x, y| x.0.cmp(&y.0))
        .map(|(score, _move)| _move)
}

fn mini(
    board: &Board,
    depth: i32,
    player: Color,
    last_move: ImplMove,
    moves_count: &mut i32,
) -> Option<i32> {
    if depth <= 0 {
        return Some(material_advantage(board, player));
    }
    board
        .get_possible_moves(player, last_move)
        .into_iter()
        .filter_map(|_move| {
            let mut board_copy = board.clone();
            board_copy.execute(_move.clone());
            *moves_count += 1;
            let (checked, _) = board.is_checked(player);
            if checked {
                return None;
            }
            maxi(
                &board_copy,
                depth - 1,
                player.opposite(),
                _move,
                moves_count,
            )
        })
        .min()
}

fn maxi(
    board: &Board,
    depth: i32,
    player: Color,
    last_move: ImplMove,
    moves_count: &mut i32,
) -> Option<i32> {
    if depth <= 0 {
        return Some(material_advantage(board, player));
    }
    board
        .get_possible_moves(player, last_move)
        .into_iter()
        .filter_map(|_move| {
            let mut board_copy = board.clone();
            board_copy.execute(_move.clone());
            *moves_count += 1;
            let (checked, _) = board.is_checked(player);
            if checked {
                return None;
            }
            mini(
                &board_copy,
                depth - 1,
                player.opposite(),
                _move,
                moves_count,
            )
        })
        .max()
}

fn alpha_beta(
    game: &Game,
    moves: Vec<ImplMove>,
    depth: i32,
    moves_count: &mut i32,
) -> Option<ImplMove> {
    let opponent = game.current_player.opposite();
    let mut moves: Vec<_> = moves
        .into_iter()
        .filter_map(|_move| {
            let mut board_copy = game.board.clone();
            board_copy.execute(_move.clone());
            *moves_count += 1;
            let (checked, _) = board_copy .is_checked(game.current_player);
            if checked {
                return None;
            }
            let score = alpha_beta_negamax(
                &board_copy,
                material_advantage(&game.board, opponent),
                material_advantage(&game.board, game.current_player),
                depth,
                opponent,
                _move.clone(),
                moves_count,
            );
            Some((score, _move))
        })
        .collect();
    moves.shuffle(&mut rand::thread_rng());
    moves
        .into_iter()
        .max_by(|x, y| x.0.cmp(&y.0))
        .map(|(score, _move)| _move)
}

fn alpha_beta_negamax(
    board: &Board,
    mut alpha: i32,
    beta: i32,
    depth: i32,
    player: Color,
    last_move: ImplMove,
    moves_count: &mut i32,
) -> i32 {
    if depth <= 0 {
        return material_advantage(board, player);
    }
    for _move in board.get_possible_moves(player, last_move) {
        let mut board_copy = board.clone();
        board_copy.execute(_move.clone());
        *moves_count += 1;
        let (checked, _) = board_copy.is_checked(player);
        if checked {
            continue;
        }
        let score = alpha_beta_negamax(
            &board_copy,
            beta,
            alpha,
            depth - 1,
            player.opposite(),
            _move,
            moves_count,
        );
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    return alpha;
}
