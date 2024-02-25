use crate::engine::{Board, Color, Move, Piece, PieceType};


#[derive(Default)]
pub struct Game {
    board: Board,
    current_player: Color,
    checked: bool,
    moves: Vec<Move>,
}

impl Game {
    fn new(board: Board) -> Game {
        Game::with_player(board, Color::White)
    }

    fn with_player(board: Board, player: Color) -> Game {
        #[cfg(debug_assertions)] {
            let (checked, _) = board.is_checked(player.opposite());
            assert!(!checked, "King in danger before move!");
        }
        let (checked, _) = board.is_checked(player);
        Game {
            board,
            current_player: Color::White,
            checked,
            moves: Vec::new(),
        }
    }

    pub fn make_move(&mut self, ui_move: UiMove) {
        self.board.execute(ui_move._move);
        // let prev_player = self.current_player;
        self.current_player = if self.current_player == Color::White {
            Color::Black
        } else {
            Color::White
        }; 
        let (checked, king) = self.board.is_checked(self.current_player);
        self.checked = checked;
        self.board.castling_rights(king);
    }

    pub fn current_player(&self) -> Color {
        self.current_player
    }

    pub fn history(&self) -> &Vec<Move> {
        &self.moves
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn possible_moves(&self, rank: u32, file: u32) -> Option<Vec<UiMove>> {
        let pos = (rank << 4) as u8 | file as u8;
        let piece = Piece::from_code(self.board.inside()[pos as usize], pos);
        if piece.type_() == PieceType::EmptySquare && piece.color() != self.current_player {
            return None;
        }
        let moves: Vec<_> = self.board.get_possible_moves(
            self.current_player,
            self.moves.last()
                .and_then(|_move| Some(_move.clone()))
                .unwrap_or(Move::NullMove))
            .into_iter()
            .filter(|_move| match _move {
                Move::NullMove => true,
                Move::QuietMove(_piece, _) => &piece == _piece,
                Move::Capture(_piece, _) => &piece == _piece,
                Move::Castling(_piece, _, _) => &piece == _piece,
                Move::PromotionQuiet(_piece, _, _) => &piece == _piece,
                Move::PromotionCapture(_piece, _, _) => &piece == _piece,
                Move::PawnDoublePush(_piece, _) => &piece == _piece,
                Move::EnPassantCapture(_piece, _) => &piece == _piece,
            })
            .map(|_move| UiMove::new(&self.board, self.current_player(), _move))
            .collect();
        if moves.is_empty() {
            None
        } else {
            Some(moves)
        }
    }
}

#[derive(Debug, Clone)]
pub struct UiMove {
    _move: Move,
    pub player: Color,
    pub impose_check: bool,
}

impl UiMove {
    fn new(board: &Board, player: Color, _move: Move) -> UiMove {
        let mut board: Board = board.clone();
        board.execute(_move.clone());
        let (impose_check, _) = board.is_checked(player.opposite());
        UiMove {
            _move,
            player,
            impose_check,
        }
    }

    pub fn position(&self) -> usize {
        match &self._move {
            Move::NullMove => panic!("Position of null move?"),
            Move::QuietMove(_, pos) => *pos as usize,
            Move::Capture(_, piece) => piece.position(),
            Move::Castling(_, _, rook) => rook.position(),
            Move::PromotionQuiet(_, pos, _) => *pos as usize,
            Move::PromotionCapture(_, piece, _) => piece.position(),
            Move::PawnDoublePush(_, pos) => *pos as usize,
            Move::EnPassantCapture(_, piece) => piece.position(),
        }
    }  
}
