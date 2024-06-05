
#[allow(unused_imports)]
use log::{trace, debug, info, warn};
use rifgen::rifgen_attr::generate_interface;

use crate::{Cell, Color, DefaultMove, Figure, Game, GameState, MatchInterface, PieceType};


pub struct WrapperGame {
    game: Game,
}

#[generate_interface]
pub enum GameStatus {
    Finished,
    MoveWhite,
    MoveBlack,
    DistantMoveWhite,
    DistantMoveBlack,
}

#[derive(Clone)]
pub struct WrapperMove {
    _move: DefaultMove,
}

#[derive(Clone)]
pub struct WrappedCell {
    cell: Cell
}

pub type Moves = Vec<WrapperMove>;
pub type Cells = Vec<WrappedCell>;

impl WrapperGame {
    #[generate_interface(constructor)]
    pub fn new() -> WrapperGame {
        Self { game: Default::default() }
    }

    #[generate_interface]
    pub fn make_move(&mut self, _move: WrapperMove) -> GameStatus {
        self.game.execute_move(_move._move).into()
    }

    #[generate_interface]
    pub fn possible_moves(&self, rank: usize, file: usize) -> Moves {
        debug!("Counting moves for r: {} f: {}", rank, file);
        let moves = self.game.possible_moves(rank, file);
        debug!("Moves count: {}", moves.as_ref().map(|m| m.len()).unwrap_or(0));
        moves
            .map(|moves|
                moves.into_iter()
                    .map(|inner| WrapperMove {_move: inner})
                    .collect())
            .unwrap_or_else(Vec::new)
    }

    #[generate_interface]
    pub fn board(&self) -> Cells {
        self.game.current_board()
            .into_iter()
            .flatten()
            .map(WrappedCell::new)
            .collect()
    }

    #[generate_interface]
    pub fn current_player(&self) -> Color {
        self.game.current_player()
    }

    #[generate_interface]
    pub fn checked(&self) -> bool {
        self.game.checked()
    }

    #[generate_interface]
    pub fn game_ended(&self) -> bool {
        self.game.game_ended()
    }
}

impl WrapperMove {
    #[generate_interface]
    pub fn to_rank(&self) -> u32 {
        self._move.to.0
    }

    #[generate_interface]
    pub fn to_file(&self) -> u32 {
        self._move.to.1
    }

    #[generate_interface]
    pub fn from_rank(&self) -> u32 {
        self._move.from.0
    }

    #[generate_interface]
    pub fn from_file(&self) -> u32 {
        self._move.from.1
    }
}

impl From<GameState> for GameStatus {
    fn from(value: GameState) -> Self {
        match value {
            GameState::PlayerMove(color) => match color {
                Color::Black => Self::MoveBlack,
                Color::White => Self::MoveWhite,
            },
            GameState::DistantMove(color) => match color {
                Color::Black => Self::DistantMoveBlack,
                Color::White => Self::DistantMoveWhite,
            },
            GameState::Finished => Self::Finished,
        }
    }
}

impl WrappedCell {
    fn new(cell: Cell) -> Self {
        Self { cell }
    }

    #[generate_interface]
    pub fn hidden(&self) -> bool {
        matches!(self.cell, Cell::Unknown)
    }

    #[generate_interface]
    pub fn has_figure(&self) -> bool {
        matches!(self.cell, Cell::Figure(..))
    }

    fn figure(&self) -> Option<&Figure> {
        match &self.cell {
            Cell::Figure(fig) => Some(fig),
            _ => None,
        }
    }

    #[generate_interface]
    pub fn kind(&self) -> PieceType {
        self.figure().unwrap().kind.clone()
    }
    #[generate_interface]
    pub fn color(&self) -> Color {
        self.figure().unwrap().color
    }
    #[generate_interface]
    pub fn impose_check(&self) -> bool {
        self.figure().unwrap().impose_check
    }
    #[generate_interface]
    pub fn can_move(&self) -> bool {
        self.figure().unwrap().can_move
    }
}
