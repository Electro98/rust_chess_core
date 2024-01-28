#![allow(dead_code)]

use std::fmt::Debug;

use log::{debug, trace};

#[derive(Clone, Copy, PartialEq)]
pub enum CastlingSide {
    KingSide = 0x07,
    QueenSide = 0x00,
}

pub enum Move {
    /** skip of move, probably will be deleted */
    NullMove,
    /** who is moving, where it's moving */
    QuietMove(Piece, u8),
    /** who is capturing, whom is beeing captured */
    Capture(Piece, Piece),
    /** king, which side */
    Castling(Piece, CastlingSide, Piece),
    /** who is promoting, where it goes, to whom we are promoting */
    PromotionQuiet(Piece, u8, PieceType),
    /** who is promoting, whom is beeing captured, to whom we are promoting */
    PromotionCapture(Piece, Piece, PieceType),
    /** who is moving, where it's moving */
    PawnDoublePush(Piece, u8),
    /** who is capturing, whom is beeing captured */
    EnPassantCapture(Piece, Piece),
}

/** Variation of 0x88 board */
pub struct Board {
    arr: [u8; 128],
}

impl Board {
    #[rustfmt::skip]
    pub fn new() -> Board {
        Board {
            arr: [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            ]
        }
    }

    #[allow(non_snake_case)]
    fn from_FEN() -> Board {
        todo!()
    }

    pub fn inside(&self) -> &[u8; 128] {
        &self.arr
    }

    /** Execute ***valid*** move. */
    pub fn execute(&mut self, _move: Move) {
        use Move::*;
        match _move {
            NullMove => trace!("Detected NullMove! Is it intentional?"),
            QuietMove(piece, move_to) => {
                assert!(
                    self.arr[move_to as usize] == 0x00,
                    "Trying to move in busy place!"
                );
                self.arr[piece.position()] = 0x00;
                self.arr[move_to as usize] = piece.code | PieceFlag::Moved as u8;
            }
            Capture(piece, target) => {
                assert!(
                    piece.color() != target.color(),
                    "That's a bug! Piece captured teammate!"
                );
                self.arr[piece.position()] = 0x00;
                self.arr[target.position()] = piece.code | PieceFlag::Moved as u8;
            }
            PawnDoublePush(pawn, move_to) => {
                assert!(
                    self.arr[move_to as usize] == 0x00,
                    "Trying to move in busy place!"
                );
                self.arr[pawn.position()] = 0x00;
                self.arr[move_to as usize] = pawn.code | PieceFlag::Moved as u8;
            }
            PromotionQuiet(pawn, move_to, new_type) => {
                assert!(
                    pawn.type_() == PieceType::Pawn,
                    "Trying to promote non-pawn piece!"
                );
                assert!(
                    self.arr[move_to as usize] == 0x00,
                    "Trying to move in busy place!"
                );
                self.arr[pawn.position()] = 0x00;
                self.arr[move_to as usize] =
                    pawn.color() as u8 | new_type as u8 | PieceFlag::Moved as u8;
            }
            PromotionCapture(pawn, target, new_type) => {
                assert!(
                    pawn.type_() == PieceType::Pawn,
                    "Trying to promote non-pawn piece!"
                );
                assert!(
                    pawn.color() != target.color(),
                    "That's a bug! Pawn captured teammate!"
                );
                self.arr[pawn.position()] = 0x00;
                self.arr[target.position()] =
                    pawn.color() as u8 | new_type as u8 | PieceFlag::Moved as u8;
            }
            Castling(king, castling_side, rook) => {
                // Checks for king
                assert!(
                    king.type_() == PieceType::King,
                    "Trying to castle without king?!"
                );
                assert!(
                    king.code & PieceFlag::Moved as u8 == 0,
                    "King have already moved!"
                );
                assert!(
                    ((castling_side == CastlingSide::KingSide
                        && king.code & PieceFlag::CanCastleKingSide as u8 != 0)
                        || (castling_side == CastlingSide::QueenSide
                            && king.code & PieceFlag::CanCastleQueenSide as u8 != 0)),
                    "King can't castle!"
                );
                // TODO: check if king crosses square under attack (castle rights bits)
                // Checks for rook
                assert!(
                    rook.type_() == PieceType::Rook,
                    "Non-rook piece used for castling!"
                );
                assert!(
                    rook.color() == king.color(),
                    "Enemy rook used for castling!"
                );
                assert!(
                    rook.code & PieceFlag::Moved as u8 == 0,
                    "Moved rook used for castling!"
                );
                #[cfg(debug_assertions)]
                for cell in between(rook.position, king.position) {
                    assert!(
                        self.arr[cell as usize] == 0x00,
                        "There is something in way of castling!"
                    );
                }
                self.arr[king.position()] = 0x00;
                self.arr[rook.position()] = 0x00;
                let rank = king.position & 0xf0;
                match castling_side {
                    CastlingSide::KingSide => {
                        self.arr[(rank | 0x06) as usize] = king.code | PieceFlag::Moved as u8;
                        self.arr[(rank | 0x05) as usize] = rook.code | PieceFlag::Moved as u8;
                    }
                    CastlingSide::QueenSide => {
                        self.arr[(rank | 0x02) as usize] = king.code | PieceFlag::Moved as u8;
                        self.arr[(rank | 0x03) as usize] = rook.code | PieceFlag::Moved as u8;
                    }
                }
            }
            EnPassantCapture(pawn, target) => {
                assert!(
                    pawn.type_() == PieceType::Pawn,
                    "Trying to use EnPassant by non-pawn piece!"
                );
                assert!(
                    target.type_() == PieceType::Pawn,
                    "Trying to capture non-pawn piece with EnPassant!"
                );
                assert!(
                    pawn.color() != target.color(),
                    "That's a bug! Pawn captured teammate!"
                );
                let step: u8 = match pawn.color() {
                    Color::Black => 0x10,
                    Color::White => 0xf0,
                };
                assert!(
                    self.arr[(target.position + step) as usize] == 0x00,
                    "Something is in way of EnPassant!"
                );
                self.arr[pawn.position()] = 0x00;
                self.arr[target.position()] = 0x00;
                self.arr[(target.position + step) as usize] = pawn.code;
            }
        }
    }

    /** Undo valid move. */
    pub fn undo(&mut self, _move: Move) {
        use Move::*;
        match _move {
            NullMove => trace!("Undo NullMove! Is it intentional?"),
            QuietMove(piece, moved_to) => {
                self.arr[moved_to as usize] = 0x00;
                self.arr[piece.position()] = piece.code;
            }
            Capture(piece, target) => {
                self.arr[target.position()] = target.code;
                self.arr[piece.position()] = piece.code;
            }
            PawnDoublePush(pawn, moved_to) => {
                self.arr[moved_to as usize] = 0x00;
                self.arr[pawn.position()] = pawn.code;
            }
            PromotionQuiet(pawn, moved_to, _) => {
                self.arr[moved_to as usize] = 0x00;
                self.arr[pawn.position()] = pawn.code;
            }
            PromotionCapture(pawn, target, _) => {
                self.arr[target.position()] = target.code;
                self.arr[pawn.position()] = pawn.code;
            }
            Castling(king, castling_side, rook) => {
                let rank = king.position & 0xf0;
                match castling_side {
                    CastlingSide::KingSide => {
                        self.arr[(rank | 0x06) as usize] = 0x00;
                        self.arr[(rank | 0x05) as usize] = 0x00;
                    }
                    CastlingSide::QueenSide => {
                        self.arr[(rank | 0x02) as usize] = 0x00;
                        self.arr[(rank | 0x03) as usize] = 0x00;
                    }
                }
                self.arr[king.position()] = king.code;
                self.arr[rook.position()] = rook.code;
            }
            EnPassantCapture(pawn, target) => {
                let step: u8 = match pawn.color() {
                    Color::Black => 0x10,
                    Color::White => 0xf0,
                };
                self.arr[(target.position + step) as usize] = 0x00;
                self.arr[pawn.position()] = pawn.code;
                self.arr[target.position()] = target.code;
            }
        }
    }

    pub fn who_can_attack(&self, piece: Piece) -> Option<Vec<Piece>> {
        let color = piece.color();
        let mut attackers = Vec::with_capacity(8);
        for file in 0..8u8 {
            for rank in 0..8u8 {
                let pos = rank << 4 | file;
                let cell = Piece::from_code(self.arr[pos as usize], pos);
                if cell.code != 0x00 && cell.color() != color && cell.can_attack(piece.position) {
                    attackers.push(cell);
                }
            }
        }
        if attackers.is_empty() {
            None
        } else {
            Some(attackers)
        }
    }

    fn is_attacked(&self, position: u8, color: Color) -> bool {
        for file in 0..8u8 {
            for rank in 0..8u8 {
                let pos = rank << 4 | file;
                let piece = Piece::from_code(self.arr[pos as usize], pos);
                if piece.code != 0x00 && piece.color() != color && piece.can_attack(position) {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_possible_moves(&self, color: Color, last_move: Move) -> Vec<Move> {
        // Check for pawn double push
        let enpassant_pawn = match last_move {
            Move::PawnDoublePush(pawn, pos) => Some(Piece::from_code(pawn.code, pos)),
            _ => None,
        };
        let mut possible_moves = Vec::with_capacity(256);
        for file in 0..8u8 {
            for rank in 0..8u8 {
                let pos = rank << 4 | file;
                let piece = Piece::from_code(self.arr[pos as usize], pos);
                if piece.code == 0x00 || piece.color() != color {
                    continue;
                }
                match piece.type_() {
                    // Special cases
                    PieceType::Pawn => {
                        let step: u8 = match color {
                            Color::Black => 0x10,
                            Color::White => 0xf0,
                        };
                        // push
                        let front_pos: u8 = piece.position + step;
                        let promotion = front_pos & 0xf0 == 0 || front_pos & 0xf0 == 0x70;
                        if self.arr[front_pos as usize] == 0x00 {
                            possible_moves.push({
                                if !promotion {
                                    Move::QuietMove(piece.clone(), front_pos)
                                } else {
                                    Move::PromotionQuiet(
                                        piece.clone(),
                                        front_pos,
                                        PieceType::Invalid,
                                    )
                                }
                            });
                        }
                        // capture
                        for step in [0x01u8, 0x0f] {
                            let pos = front_pos + step;
                            let cell = self.arr[pos as usize];
                            if cell & 0x88 == 0 && cell != 0x00 && Color::from_byte(cell) != color {
                                possible_moves.push({
                                    let target = Piece::from_code(cell, pos);
                                    if !promotion {
                                        Move::Capture(piece.clone(), target)
                                    } else {
                                        Move::PromotionCapture(
                                            piece.clone(),
                                            target,
                                            PieceType::Invalid,
                                        )
                                    }
                                });
                            }
                        }
                        // double push
                        if piece.code & PieceFlag::Moved as u8 != 0 {
                            let pos = front_pos + step;
                            if self.arr[pos as usize] == 0x00 {
                                possible_moves.push(Move::PawnDoublePush(piece.clone(), pos))
                            }
                        }
                        // enpassant
                        if let Some(pawn) = &enpassant_pawn {
                            if pawn.position.abs_diff(piece.position) == 0x01 {
                                possible_moves.push(Move::EnPassantCapture(piece, pawn.clone()))
                            }
                        }
                    }
                    PieceType::Knight => {
                        for offset in KNIGHT_MOVES {
                            let pos = pos + offset;
                            if pos & 0x00 != 0x00 || pos & 0x88 != 0 {
                                continue;
                            }
                            let cell = self.arr[pos as usize];
                            if cell == 0x00 {
                                possible_moves.push(Move::QuietMove(piece.clone(), pos));
                            } else if Color::from_byte(cell) != color {
                                possible_moves
                                    .push(Move::Capture(piece.clone(), Piece::from_code(cell, pos)))
                            }
                        }
                    }
                    PieceType::King => {
                        for offset in KING_MOVES {
                            let pos = pos + offset;
                            if pos & 0x00 != 0x00 {
                                continue;
                            }
                            let cell = self.arr[pos as usize];
                            if cell == 0x00 {
                                possible_moves.push(Move::QuietMove(piece.clone(), pos));
                            } else if Color::from_byte(cell) != color {
                                possible_moves
                                    .push(Move::Capture(piece.clone(), Piece::from_code(cell, pos)))
                            }
                        }
                        if piece.code & PieceFlag::Moved as u8 != 0 {
                            break;
                        }
                        for castling_side in [CastlingSide::KingSide, CastlingSide::QueenSide] {
                            let rook_pos = piece.position & 0xf0 | castling_side as u8;
                            let cell = self.arr[rook_pos as usize];
                            if cell & PieceFlag::Moved as u8 == 0
                                && Color::from_byte(cell) == color
                                && PieceType::from_byte(cell) == PieceType::Rook
                                && ((castling_side == CastlingSide::KingSide
                                    && piece.code & PieceFlag::CanCastleKingSide as u8 != 0)
                                    || (castling_side == CastlingSide::QueenSide
                                        && piece.code & PieceFlag::CanCastleQueenSide as u8 != 0))
                            {
                                possible_moves.push(Move::Castling(
                                    piece.clone(),
                                    castling_side,
                                    Piece::from_code(cell, rook_pos),
                                ))
                            }
                        }
                    }
                    // Invalid block
                    PieceType::Invalid => panic!("That's bug! Invalid square in valid space!"),
                    PieceType::EmptySquare => panic!("Empty square can't move"),
                    // Sliding pieces
                    sliding_type => {
                        let possible_directions = match sliding_type {
                            PieceType::Bishop => BISHOP_DIR,
                            PieceType::Rook => ROOK_DIR,
                            PieceType::Queen => QUEEN_DIR,
                            _ => panic!("Unreachable!"),
                        };
                        for dir in possible_directions {
                            let mut pos = pos + dir;
                            while pos & 0x88 == 0x00 {
                                let cell = self.arr[pos as usize];
                                if cell == 0x00 {
                                    possible_moves.push(Move::QuietMove(piece.clone(), pos));
                                } else if Color::from_byte(cell) != color {
                                    possible_moves.push(Move::Capture(
                                        piece.clone(),
                                        Piece::from_code(cell, pos),
                                    ))
                                } else {
                                    break;
                                }
                                pos += dir;
                            }
                        }
                    }
                }
            }
        }
        possible_moves
    }

    pub fn is_checked(&self, color: Color) -> (bool, Piece) {
        for file in 0..8u8 {
            for rank in 0..8u8 {
                let pos = rank << 4 | file;
                let piece = Piece::from_code(self.arr[pos as usize], pos);
                if piece.type_() != PieceType::King || piece.color() != color {
                    continue;
                }
                return (self.is_attacked(piece.position, color), piece);
            }
        }
        panic!("Where is the king?????????????????????????????")
    }

    pub fn castling_right_check(&self, king: Piece) -> (bool, bool) {
        if king.code & PieceFlag::Moved as u8 != 0 {
            return (false, false);
        }
        let rank = king.position & 0xf0;
        (
            {
                // KingSide
                let mut flag = true;
                for file in [0x06, 0x05] {
                    let pos = rank | file;
                    let code = self.arr[pos as usize];
                    if code != 0 || self.is_attacked(pos, king.color()) {
                        flag = false;
                        break;
                    }
                }
                flag
            },
            {
                // QueenSide
                let mut flag = true;
                for file in [0x03, 0x02] {
                    let pos = rank | file;
                    let code = self.arr[pos as usize];
                    if code != 0 || self.is_attacked(pos, king.color()) {
                        flag = false;
                        break;
                    }
                }
                flag
            },
        )
    }
}

impl Default for Board {
    #[rustfmt::skip]
    fn default() -> Self {
        Board {
            arr: [
                0x04, 0x02, 0x03, 0x05, 0x06, 0x03, 0x02, 0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x84, 0x82, 0x83, 0x85, 0x86, 0x83, 0x82, 0x84, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            ]
        }
    }
}

/** Tables directions for pieces */
const BISHOP_DIR: &[u8] = &[0x11, 0x1f, 0xff, 0xf1];
const ROOK_DIR: &[u8] = &[0x10, 0x0f, 0xf0, 0x01];
const QUEEN_DIR: &[u8] = &[0x11, 0x1f, 0xff, 0xf1, 0x10, 0x0f, 0xf0, 0x01];

/** Possible moves for pieces */
const KING_MOVES: &[u8] = QUEEN_DIR;
const KNIGHT_MOVES: &[u8] = &[0x12, 0x21, 0x2f, 0x1e, 0xfe, 0xef, 0xe1, 0xf2];

/** Bits structure of piece code
 * Bit 7 -- Color of the piece
 * - 1 -- Black
 * - 0 -- White
 * Bit 6 -- Not used
 * Bit 5 -- Castle flag for Kings only - QueenSide
 * Bit 4 -- Castle flag for Kings only - KingSide
 * Bit 3 -- Piece has moved flag
 * Bits 2-0 Piece type
 * - 1 -- Pawn
 * - 2 -- Knight
 * - 3 -- Bishop
 * - 4 -- Rook
 * - 5 -- Queen
 * - 6 -- King
 * - 7 -- Not used
 * - 0 -- Empty Square */
#[derive(Clone, PartialEq)]
pub struct Piece {
    code: u8,
    position: u8,
}

enum PieceFlag {
    /** Bit 3 -- Piece has moved flag */
    Moved = 0x08,
    /** Bit 4 -- Castle flag for Kings only - KingSide */
    CanCastleKingSide = 0x10,
    /** Bit 5 -- Castle flag for Kings only - QueenSide */
    CanCastleQueenSide = 0x20,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color, position: u8) -> Piece {
        Piece {
            code: piece_type as u8 | color as u8,
            position,
        }
    }

    pub fn from_code(code: u8, position: u8) -> Piece {
        Piece { code, position }
    }

    pub fn color(&self) -> Color {
        Color::from_byte(self.code)
    }

    pub fn type_(&self) -> PieceType {
        PieceType::from_byte(self.code)
    }

    pub fn position(&self) -> usize {
        self.position as usize
    }

    fn can_attack(&self, target: u8) -> bool {
        // precomputed tables should increase speed of this dramatically
        match self.type_() {
            PieceType::Pawn => {
                let step: u8 = match self.color() {
                    Color::White => 0x10,
                    Color::Black => 0xf0,
                };
                distance(self.position, target) == 2 && (self.position & 0xf0 + step) == target
            }
            PieceType::Bishop => {
                is_in_diagonal_line(self.position, target)
                    && between(self.position, target).all(|cell| cell == 0x00)
            }
            PieceType::Rook => {
                is_in_straight_line(self.position, target)
                    && between(self.position, target).all(|cell| cell == 0x00)
            }
            PieceType::Queen => {
                (is_in_straight_line(self.position, target)
                    || is_in_diagonal_line(self.position, target))
                    && between(self.position, target).all(|cell| cell == 0x00)
            }
            PieceType::Knight => {
                let diff = (self.position & 0x0f).abs_diff(target & 0x0f);
                distance(self.position, target) == 3 && diff != 0 && diff != 3
            }
            PieceType::King => distance(self.position, target) == 1,
            PieceType::Invalid => panic!("Invalid square is trying to attack?!"),
            PieceType::EmptySquare => panic!("Empty square is trying to attack?!"),
        }
    }
}

impl Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Piece")
            .field("code", &self.code)
            .field("position", &self.position)
            .field("color", &self.color())
            .field("type", &self.type_())
            .finish()
    }
}

#[derive(PartialEq, Debug)]
pub enum Color {
    Black = 0x00,
    White = 0x80,
}

impl Color {
    #[inline]
    fn from_byte(byte: u8) -> Color {
        unsafe { std::mem::transmute(byte & 0x80) }
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        Color::from_byte(value)
    }
}

impl Into<u8> for Color {
    fn into(self) -> u8 {
        self as u8
    }
}

#[derive(PartialEq, Debug)]
pub enum PieceType {
    Pawn = 0x01,
    Knight = 0x02,
    Bishop = 0x03,
    Rook = 0x04,
    Queen = 0x05,
    King = 0x06,
    Invalid = 0x07,
    EmptySquare = 0x00,
}

impl PieceType {
    #[inline]
    fn from_byte(byte: u8) -> PieceType {
        unsafe { std::mem::transmute(byte & 0x07) }
    }
}

impl From<u8> for PieceType {
    fn from(value: u8) -> Self {
        PieceType::from_byte(value)
    }
}

impl Into<u8> for PieceType {
    fn into(self) -> u8 {
        self as u8
    }
}

struct BetweenIterator {
    current: u8,
    target: u8,
    step: u8,
}

impl Iterator for BetweenIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.current += self.step;
        if self.current == self.target {
            None
        } else {
            Some(self.current)
        }
    }
}

fn between(from: u8, to: u8) -> BetweenIterator {
    // TODO: Check if they on one side of spectre
    #[cfg(debug_assertions)]
    if !is_in_diagonal_line(from, to) && !is_in_straight_line(from, to) {
        panic!("Points can't form line to search between them!")
    }
    let diff = to - from;
    // TODO: precompute table?
    // const TABLE: [u8; 256] = {
    //     let mut table = [0; 256];
    //     ... something ...
    //     table
    // };
    let dx: u8 = {
        if diff & 0xf0 == 0 {
            0
        } else if diff & 0x80 == 0x80 {
            0xf0
        } else {
            0x10
        }
    };
    let dy: u8 = {
        if diff & 0x0f == 0 {
            0
        } else if diff & 0x08 == 0x08 {
            0x0f
        } else {
            0x01
        }
    };
    let step = dx & dy;
    BetweenIterator {
        current: from,
        target: to,
        step,
    }
}

fn distance(from: u8, to: u8) -> u8 {
    (from & 0x0f).abs_diff(to & 0x0f) + (from & 0xf0 >> 4).abs_diff(to & 0xf0 >> 4)
}

fn is_in_straight_line(a: u8, b: u8) -> bool {
    let diff = a.abs_diff(b);
    diff & 0x0f == 0 || diff & 0xf0 == 0
}

fn is_in_diagonal_line(a: u8, b: u8) -> bool {
    let diff = a.abs_diff(b);
    diff & 0x0f == diff & 0xf0
}
