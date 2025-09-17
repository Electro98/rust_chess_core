use crate::core::definitions::{Cell, Figure};
use crate::core::engine::{
    Board, CheckType, Color, CompressedBoard, GameEndState, Move, MoveType, Piece, PieceFlag,
    PieceType,
};
use crate::core::utils::{compact_pos, unpack_pos};

pub fn ui_board(board: &Board) -> Vec<Vec<Cell>> {
    (0..8)
        .map(|file| {
            (0..8)
                .map(|rank| compact_pos(file, rank))
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
                .collect()
        })
        .collect()
}

#[derive(Clone, Debug)]
pub enum GameHistory {
    LastMove(Option<Move>),
    FullHistory(Vec<Move>),
}

impl Default for GameHistory {
    fn default() -> Self {
        Self::LastMove(None)
    }
}

impl GameHistory {
    pub fn last_move(&self) -> Option<Move> {
        match self {
            GameHistory::LastMove(last_move) => last_move.clone(),
            GameHistory::FullHistory(moves) => moves.last().cloned(),
        }
    }

    fn record(&mut self, new_move: Move) {
        match self {
            GameHistory::LastMove(last_move) => *last_move = Some(new_move),
            GameHistory::FullHistory(moves) => moves.push(new_move),
        }
    }

    fn unrecord(&mut self) {
        match self {
            GameHistory::LastMove(last_move) => {
                if last_move.is_some() {
                    *last_move = None;
                } else {
                    panic!("Trying to undo unrecorded move!");
                }
            }
            GameHistory::FullHistory(moves) => {
                moves.pop();
            }
        }
    }

    fn light_clone(&self) -> Self {
        GameHistory::LastMove(self.last_move())
    }
}

#[derive(Clone, Default, Debug)]
struct ExistedPositions {
    boards: Vec<CompressedBoard>,
    offsets: Vec<usize>,
}

impl ExistedPositions {
    fn new() -> Self {
        Default::default()
    }

    fn push(&mut self, board: CompressedBoard) {
        self.boards.push(board);
    }

    fn count(&mut self, board: &CompressedBoard) -> usize {
        self.boards
            .iter()
            .skip(*self.offsets.last().unwrap_or(&0))
            .filter(|exboard| *exboard == board)
            .count()
    }

    fn clear(&mut self) {
        self.offsets.push(self.boards.len());
    }

    fn undo_move(&mut self, move_type: MoveType) {
        self.boards.pop();
        if !matches!(move_type, MoveType::QuietMove(_)) {
            self.offsets.pop();
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Game {
    board: Board,
    current_player: Color,
    existed_positions: ExistedPositions,
    history: GameHistory,
}

fn flag_piece_moved(piece: PieceType, color: Color, pos: u8) -> u8 {
    let (file, rank): (u8, u8) = unpack_pos(pos);
    let right_rank = match piece {
        PieceType::Pawn => true,
        PieceType::Rook => rank == 0x0 || rank == 0x7,
        PieceType::Knight => rank == 0x1 || rank == 0x6,
        PieceType::Bishop => rank == 0x2 || rank == 0x5,
        PieceType::Queen => rank == 0x3,
        PieceType::King => rank == 0x4,
        PieceType::EmptySquare => true,
        PieceType::Invalid => unreachable!("Created an invalid piece?"),
    };
    let is_pawn = matches!(piece, PieceType::Pawn);
    let right_file = match color {
        Color::Black => (is_pawn && file == 0x06) || (!is_pawn && file == 0x07),
        Color::White => (is_pawn && file == 0x01) || (!is_pawn && file == 0x00),
    };
    if right_rank && right_file {
        0
    } else {
        PieceFlag::Moved as u8
    }
}

impl Game {
    pub fn new(board: Board, current_player: Color, last_move: Option<Move>) -> Game {
        Game {
            board,
            current_player,
            history: GameHistory::FullHistory(if let Some(_move) = last_move {
                vec![_move]
            } else {
                Vec::new()
            }),
            ..Default::default()
        }
    }

    pub fn from_fen(fen: &str) -> Result<Game, String> {
        let mut chars = fen.chars();
        let mut board = Board::new();
        // Board portion
        for file in (0..8).rev() {
            let mut rank = 0;
            for letter in chars.by_ref() {
                if let Some(num) = letter.to_digit(10) {
                    rank += num;
                    continue;
                }
                if rank == 8 {
                    break;
                }
                let color: Color = if letter.is_uppercase() {
                    Color::White
                } else {
                    Color::Black
                };
                let piece: PieceType = match letter {
                    'p' | 'P' => PieceType::Pawn,
                    'r' | 'R' => PieceType::Rook,
                    'n' | 'N' => PieceType::Knight,
                    'b' | 'B' => PieceType::Bishop,
                    'q' | 'Q' => PieceType::Queen,
                    'k' | 'K' => PieceType::King,
                    // This point should not be reached,
                    //  because final iteration of loop will consume '/' or whitespace
                    // '/' => break,
                    _ => {
                        return Err(format!(
                            "Unexpected symbol '{letter}' during parsing board layout"
                        ))
                    }
                };
                let pos = compact_pos(file as u8, rank as u8);
                board.arr[pos as usize] =
                    piece as u8 | color as u8 | flag_piece_moved(piece, color, pos);
                rank += 1;
            }
        }
        // Active player
        let current_player = match chars.next() {
            Some('w') => Color::White,
            Some('b') => Color::Black,
            Some(letter) => {
                return Err(format!(
                    "Unexpected symbol '{letter}' during parsing active player"
                ))
            }
            None => return Err("String exhausted too early".to_string()),
        };
        chars.next();
        // Castling availability
        let mut rights = 0u8;
        let mut rights_color = Color::White;
        let update_king = |board: &mut Board, color: Color, rights: u8| {
            let king = if let Some(king) = board
                .iter_pieces()
                .find(|piece| piece.type_() == PieceType::King && piece.color() == color)
            {
                king
            } else {
                return Err(format!("Can't find {color} king"));
            };
            board.arr[king.position()] = king.code | rights;
            Ok(())
        };
        for letter in chars.by_ref() {
            if letter.is_lowercase() && rights_color == Color::White {
                update_king(&mut board, rights_color, rights)?;
                rights = 0;
                rights_color = Color::Black;
            }
            match letter {
                'Q' | 'q' => rights |= PieceFlag::CanCastleQueenSide as u8,
                'K' | 'k' => rights |= PieceFlag::CanCastleKingSide as u8,
                '-' => continue,
                ' ' => break,
                _ => {
                    return Err(format!(
                        "Unexpected symbol '{letter}' during parsing castling rights"
                    ))
                }
            }
        }
        update_king(&mut board, rights_color, rights)?;
        // En Passant target square
        let mut last_move = match chars.next() {
            Some('-') => None,
            Some(letter) => {
                let rank = match letter.to_ascii_lowercase() {
                    'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' => letter as u8 - b'a',
                    _ => return Err("Error in move rank".to_string()),
                };
                let mut file = chars.next().unwrap() as u8 - b'0';
                if current_player == Color::White {
                    file -= 2;
                }
                let pos = compact_pos(file, rank);
                let piece = Piece::from_code(board.arr[pos as usize], pos);
                if piece.type_() == PieceType::Pawn {
                    Some(Move::new(
                        piece,
                        MoveType::PawnDoublePush(pos),
                        CheckType::None,
                    ))
                } else {
                    None
                }
            }
            None => return Err("FEN string ended too early".to_string()),
        };
        let king = board
            .iter_pieces()
            .find(|piece| piece.color() == current_player && piece.type_() == PieceType::King)
            .expect("The player should have a king to move.");
        let attackers: Vec<_> = board
            .iter_pieces()
            .filter(|piece| {
                piece.color() != current_player
                    && piece.type_() != PieceType::EmptySquare
                    && piece.can_attack(king.position, &board.arr)
            })
            .collect();
        if let Some(check) = match attackers.len() {
            0 => None,
            1 if last_move
                .as_ref()
                .is_some_and(|_move| _move.end_position() == attackers[0].position) =>
            {
                Some(CheckType::Direct)
            }
            1 => Some(CheckType::Discovered),
            _ => Some(CheckType::Double),
        } {
            last_move = last_move.map_or(
                Some(Move::new(
                    Piece::from_code(0xff, 0xff),
                    MoveType::QuietMove(0xff),
                    check,
                )),
                |last_move| Some(Move::new(*last_move.piece(), *last_move.move_type(), check)),
            )
        }
        // The rest is currently ignored
        Ok(Self {
            board,
            current_player,
            existed_positions: Default::default(),
            history: GameHistory::FullHistory(if let Some(last_move) = last_move {
                vec![last_move]
            } else {
                Vec::new()
            }),
        })
    }

    pub fn get_possible_moves(&self, bot: bool) -> Vec<Move> {
        self.board
            .get_possible_moves(self.current_player, self.history.last_move(), bot)
    }

    pub fn execute(&mut self, _move: Move) -> Option<GameEndState> {
        self.current_player = self.current_player.opposite();
        self.board.execute(_move.clone());
        // Check to see if move caused direct check!
        let _move = if _move.promotion() {
            let new_piece = Piece::from_code(
                self.board.arr[_move.end_position() as usize],
                _move.end_position(),
            );
            assert!(
                !matches!(
                    new_piece.type_(),
                    PieceType::Invalid | PieceType::EmptySquare
                ),
                "Something is gone horrible wrong!"
            );
            let enemy_king = self
                .board
                .iter_pieces()
                .find(|piece| {
                    piece.color() == self.current_player && piece.type_() == PieceType::King
                })
                .expect("King of the enemy should be present to make check of danger");
            let check = if new_piece.can_attack(enemy_king.position, self.board.inside()) {
                if matches!(_move.check(), CheckType::Discovered | CheckType::Double) {
                    CheckType::Double
                } else {
                    CheckType::Direct
                }
            } else {
                _move.check()
            };
            Move::new(*_move.piece(), *_move.move_type(), check)
        } else {
            _move
        };
        self.history.record(_move.clone());
        let compressed_board = self.board.compress();
        match _move.move_type() {
            MoveType::QuietMove(_) => {
                if self.existed_positions.count(&compressed_board) >= 2 {
                    return Some(GameEndState::DrawThreefoldRepetition);
                }
            }
            _ => self.existed_positions.clear(),
        }
        self.existed_positions.push(compressed_board);
        #[cfg(debug_assertions)]
        {
            let current_check = self.current_check_state();
            let check_from_move = _move.check();
            if current_check != check_from_move {
                let _ = self.current_check_state();
                println!("Current Player: {}", self.current_player);
                println!("Last move: {:?}", self.history.last_move());
                println!("Board: {:?}", self.board());
                panic!("Got different check type current: {current_check:?} from move: {check_from_move:?}");
            }
        }
        if self.get_possible_moves(false).is_empty() {
            match _move.check() {
                CheckType::None => Some(GameEndState::DrawStalemate),
                _ => Some(GameEndState::CheckMate(self.current_player.opposite())),
            }
        } else {
            None
        }
    }

    pub fn undo_last_move(&mut self) -> Result<(), &'static str> {
        let last_move = self.history.last_move().ok_or("There's no move to undo.")?;
        self.existed_positions.undo_move(*last_move.move_type());
        self.board.undo(last_move);
        self.history.unrecord();
        self.current_player = self.current_player.opposite();
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn current_check_state(&self) -> CheckType {
        let king =
            if let Some(king) = self.board.iter_pieces().find(|piece| {
                piece.color() == self.current_player && piece.type_() == PieceType::King
            }) {
                king
            } else {
                let mut last_board = self.board.clone();
                last_board.undo(self.history.last_move().unwrap());
                println!("Last player: {}", self.current_player.opposite());
                println!("Board: {:?}", last_board);
                panic!("King of the current player should be present to make check of danger");
            };
        match self.board.who_can_attack(king) {
            Some(attackers) => match attackers.len() {
                0 => unreachable!("This is just bug!"),
                1 => {
                    let attacker = attackers[0];
                    if self.history.last_move().is_some_and(|last_move| {
                        let last_piece = match last_move.move_type() {
                            MoveType::PromotionCapture(_, new_type)
                            | MoveType::PromotionQuiet(_, new_type) => Piece::from_code(
                                last_move.piece().color() as u8
                                    | *new_type as u8
                                    | PieceFlag::Moved as u8,
                                last_move.end_position(),
                            ),
                            MoveType::Castling(_, rook) => Piece::from_code(
                                rook.code | PieceFlag::Moved as u8,
                                last_move.castling_rook_position(),
                            ),
                            _ => Piece::from_code(
                                last_move.piece().code | PieceFlag::Moved as u8,
                                last_move.end_position(),
                            ),
                        };
                        last_piece == attacker
                    }) {
                        CheckType::Direct
                    } else {
                        CheckType::Discovered
                    }
                }
                2 => CheckType::Double,
                _ => unreachable!("Should not be possible in legal chess game"),
            },
            None => CheckType::None,
        }
    }

    pub fn light_clone(&self) -> Self {
        Self {
            board: self.board.clone(),
            current_player: self.current_player,
            existed_positions: self.existed_positions.clone(),
            history: self.history.light_clone(),
        }
    }

    pub fn current_player(&self) -> Color {
        self.current_player
    }

    pub fn history(&self) -> GameHistory {
        self.history.clone()
    }

    pub fn board(&self) -> &Board {
        &self.board
    }
}
