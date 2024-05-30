use std::cmp::min;

use crate::definitions::GameState;
use crate::engine::{Board, Color, Move as ImplMove, Piece, PieceFlag, PieceType};
use crate::utils::{compact_pos, unpack_pos};
use crate::{Cell, DefaultMove, Figure, MatchInterface};

use rand::seq::IteratorRandom;

#[derive(Debug)]
pub struct InvalidMoveError {}

impl TryInto<DefaultMove> for ImplMove {
    type Error = InvalidMoveError;

    fn try_into(self) -> Result<DefaultMove, Self::Error> {
        if matches!(self, ImplMove::NullMove) {
            Err(InvalidMoveError {})
        } else {
            Ok(DefaultMove {
                from: unpack_pos(self.piece().unwrap().position() as u8),
                to: unpack_pos(self.end_position().unwrap()),
                _move: self,
            })
        }
    }
}

#[inline]
fn is_move_valid(_move: &ImplMove, board: &Board, current_player: Color, default: bool) -> bool {
    let mut board: Board = board.clone();
    board.execute(_move.clone());
    match board.is_checked(current_player) {
        Some((checked, _)) => !checked,
        None => default,
    }
}

pub fn ui_board(board: &Board) -> Vec<Vec<Cell>> {
    (0..8).map(|rank|
        (0..8).map(|file| compact_pos(rank, file))
            .map(|pos| (board.inside()[pos as usize], pos))
            .map(|(code, position)| {
                if PieceFlag::UnknownCellFlag.is_set(code) {
                    Cell::Unknown
                } else if code == 0x00 {
                    Cell::Empty
                } else {
                    let piece = Piece::from_code(code, position);
                    Cell::Figure(Figure {
                        kind: piece.type_(),
                        color: piece.color(),
                        last_move: false,
                        impose_check: false,
                        can_move: true,
                    })
                }
            })
            .collect())
        .collect()
}

#[derive(Debug, Clone)]
pub struct DarkGame {
    board: Board,
    player: Color,
    last_move: ImplMove,
    finished: bool,
}

impl DarkGame {
    pub fn new(board: Board, player: Color) -> Self {
        DarkGame { board, player, last_move: ImplMove::NullMove, finished: false }
    }
}

impl MatchInterface<ImplMove> for DarkGame {
    fn current_board(&self) -> Vec<Vec<Cell>> {
        ui_board(&self.board)
    }
    
    fn cell(&self, rank: usize, file: usize) -> Option<Cell> {
        if file < 8 && rank < 8 {
            let pos = compact_pos(rank as u8, file as u8);
            let piece = Piece::from_code(self.board.inside()[pos as usize], pos);
            Some(if piece.type_() == PieceType::EmptySquare {
                Cell::Empty
            } else {
                Cell::Figure(Figure {
                    kind: piece.type_(),
                    color: piece.color(),
                    last_move: false,
                    impose_check: false,
                    can_move: true,
                })
            })
        } else {
            None
        }
    }

    fn possible_moves(&self, rank: usize, file: usize) -> Option<Vec<DefaultMove>> {
        let piece = self.board.get(rank as u8, file as u8);
        if self.finished
            || matches!(piece.type_(), PieceType::EmptySquare)
            || piece.color() != self.player {
            None
        } else {
            let moves: Vec<_> = self.board
                .get_possible_moves(self.player, self.last_move.clone(), false)
                .into_iter()
                .filter(|_move| _move.piece().map(|move_piece| move_piece == &piece).unwrap_or(false))
                .filter(|_move| is_move_valid(_move, &self.board, self.player, true))
                .map(|_move| _move.try_into().unwrap())
                .collect();
            if moves.is_empty() {
                None
            } else {
                Some(moves)
            }
        }
    }

    fn execute_move(&mut self, _move: DefaultMove) -> GameState {
        todo!()
    }

    fn wait_move(&mut self) -> GameState {
        todo!()
    }

    fn current_player(&self) -> Color {
        self.player
    }

    fn checked(&self) -> bool {
        self.board
            .is_checked(self.player)
            .map(|(checked, _)| checked)
            .unwrap_or(false)
    }

    fn game_ended(&self) -> bool {
        self.finished
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    board: Board,
    current_player: Color,
    history: Vec<DefaultMove>,
    checked: bool,
    finished: bool,
}

impl Game {
    pub fn new(board: Board) -> Game {
        Game::with_player(board, Color::White)
    }

    pub fn with_player(board: Board, player: Color) -> Game {
        #[cfg(debug_assertions)]
        {
            let check_possibility = board.is_checked(player.opposite());
            assert!(check_possibility.is_some(), "Board must be valid!");
            if let Some((checked, _)) = check_possibility {
                assert!(!checked, "King in danger before move!");
            }
        }
        let (checked, _) = board.is_checked(player).expect("Board must be valid!");
        Game {
            board,
            current_player: player,
            history: Vec::new(),
            checked,
            finished: false,
        }
    }

    fn make_move(&mut self, impl_move: ImplMove) -> GameState {
        self.board.execute(impl_move.clone());
        // let prev_player = self.current_player;
        self.current_player = if self.current_player == Color::White {
            Color::Black
        } else {
            Color::White
        };
        let check_possibility = self.board.is_checked(self.current_player);
        if check_possibility.is_none() {
            return GameState::Finished;
        }
        let (checked, king) = check_possibility.unwrap();
        self.checked = checked;
        let moves: Vec<_> = self
            .board
            .get_possible_moves(
                self.current_player,
                self.last_move().unwrap_or(ImplMove::NullMove),
                true,
            )
            .into_iter()
            .filter(|impl_move| is_move_valid(impl_move, &self.board, self.current_player, false))
            .collect();
        self.finished = moves.is_empty();
        self.board.castling_rights(king);
        GameState::PlayerMove(self.current_player)
    }

    pub fn last_move(&self) -> Option<ImplMove> {
        self.history.last().map(|_move| _move._move.clone())
    }

    pub fn make_random_move(&mut self) -> GameState {
        if self.finished {
            return GameState::Finished;
        }
        let chosen_move = self
            .board
            .get_possible_moves(
                self.current_player,
                self.last_move().unwrap_or(ImplMove::NullMove),
                true,
            )
            .into_iter()
            .filter(|impl_move| is_move_valid(impl_move, &self.board, self.current_player, false))
            .choose(&mut rand::thread_rng());

        if let Some(_move) = chosen_move {
            self.execute_move(_move.try_into().unwrap())
        } else {
            self.finished = true;
            GameState::Finished
        }
    }

    pub fn vision_board(&self, _player: Color) -> Board {
        self.board.clone()
            .hide_and_obstruct(_player)
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new(Default::default())
    }
}

impl MatchInterface<ImplMove> for Game {
    fn current_board(&self) -> Vec<Vec<Cell>> {
        ui_board(&self.board)
    }
    
    fn cell(&self, rank: usize, file: usize) -> Option<Cell> {
        if file < 8 && rank < 8 {
            let pos = compact_pos(rank as u8, file as u8);
            let piece = Piece::from_code(self.board.inside()[pos as usize], pos);
            Some(if piece.type_() == PieceType::EmptySquare {
                Cell::Empty
            } else {
                Cell::Figure(Figure {
                    kind: piece.type_(),
                    color: piece.color(),
                    last_move: false,
                    impose_check: false,
                    can_move: true,
                })
            })
        } else {
            None
        }
    }

    fn possible_moves(&self, rank: usize, file: usize) -> Option<Vec<DefaultMove>> {
        let piece = self.board.get(rank as u8, file as u8);
        if self.finished
            || matches!(piece.type_(), PieceType::EmptySquare)
            || piece.color() != self.current_player{
            None
        } else {
            let moves: Vec<_> = self.board
                .get_possible_moves(self.current_player, self.last_move().unwrap_or(ImplMove::NullMove), false)
                .into_iter()
                .filter(|_move| _move.piece().map(|move_piece| move_piece == &piece).unwrap_or(false))
                .filter(|_move| is_move_valid(_move, &self.board, self.current_player, false))
                .map(|_move| _move.try_into().unwrap())
                .collect();
            if moves.is_empty() {
                None
            } else {
                Some(moves)
            }
        }
    }

    fn execute_move(&mut self, _move: DefaultMove) -> GameState {
        self.history.push(_move.clone());
        self.make_move(_move._move)
    }

    fn wait_move(&mut self) -> GameState {
        // nothing
        if self.finished {
            return GameState::Finished;
        }
        GameState::PlayerMove(self.current_player)
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

#[allow(dead_code)]
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
