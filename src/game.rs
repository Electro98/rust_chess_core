use crate::engine::{Board, Color, Move as ImplMove, Piece, PieceType};
use crate::{Cell, Figure, MatchInterface, Move};

#[derive(Default)]
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
}

impl MatchInterface<ImplMove> for Game {
    fn current_board(&self) -> Vec<Vec<Cell>> {
        todo!()
    }

    fn possible_moves(&self, rank: u32, file: u32) -> Option<Vec<Move<ImplMove>>> {
        if self.finished {
            return None;
        }
        let pos = (rank << 4) as u8 | file as u8;
        let piece = Piece::from_code(self.board.inside()[pos as usize], pos);
        if piece.type_() == PieceType::EmptySquare && piece.color() != self.current_player {
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
        todo!()
    }

    fn wait_move(&self) {
        todo!()
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
