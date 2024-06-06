#![allow(dead_code)]

use std::fmt::Display;
use std::{fmt::Debug, iter::zip};

use log::trace;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

use crate::definitions::ImplicitMove;
use crate::utils::{
    between, compact_pos, distance, in_direction, is_in_diagonal_line, is_in_straight_line,
    is_valid_coord, unpack_pos,
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CastlingSide {
    KingSide = 0x07,
    QueenSide = 0x00,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Move {
    pub fn piece(&self) -> Option<&Piece> {
        match self {
            Move::QuietMove(piece, _) => Some(piece),
            Move::Capture(piece, _) => Some(piece),
            Move::Castling(piece, _, _) => Some(piece),
            Move::PromotionQuiet(piece, _, _) => Some(piece),
            Move::PromotionCapture(piece, _, _) => Some(piece),
            Move::PawnDoublePush(piece, _) => Some(piece),
            Move::EnPassantCapture(piece, _) => Some(piece),
            Move::NullMove => None,
        }
    }
    pub fn end_position(&self) -> Option<u8> {
        match &self {
            Move::QuietMove(_, pos) => Some(*pos),
            Move::Capture(_, piece) => Some(piece.position() as u8),
            Move::Castling(_, _, piece) => Some(piece.position() as u8),
            Move::PromotionQuiet(_, pos, _) => Some(*pos),
            Move::PromotionCapture(_, piece, _) => Some(piece.position() as u8),
            Move::PawnDoublePush(_, pos) => Some(*pos),
            Move::EnPassantCapture(_, piece) => Some(piece.position() as u8),
            Move::NullMove => None,
        }
    }
}

impl ImplicitMove for Move {
    fn promotion(&self) -> bool {
        matches!(self, Move::PromotionQuiet(..) | Move::PromotionCapture(..))
    }

    fn set_promotion_type(&mut self, king: PieceType) {
        match self {
            Move::PromotionQuiet(_, _, _type) => *_type = king,
            Move::PromotionCapture(_, _, _type) => *_type = king,
            _ => panic!("`set_promotion_type` on non-promotion move"),
        }
    }
}

/** Variation of 0x88 board */
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    #[serde_as(as = "Bytes")]
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
    pub fn from_FEN(fen: &str) -> (Board, Color, Move) {
        todo!()
    }

    pub fn inside(&self) -> &[u8; 128] {
        &self.arr
    }

    pub fn get(&self, rank: u8, file: u8) -> Piece {
        let position = compact_pos(rank, file);
        Piece::from_code(self.arr[position as usize], position)
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
                    !PieceFlag::Moved.is_set(king.code),
                    "King have already moved!"
                );
                assert!(
                    ((castling_side == CastlingSide::KingSide
                        && PieceFlag::CanCastleKingSide.is_set(king.code))
                        || (castling_side == CastlingSide::QueenSide
                            && PieceFlag::CanCastleQueenSide.is_set(king.code))),
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
                    !PieceFlag::Moved.is_set(rook.code),
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
                    self.arr[target.position.wrapping_add(step) as usize] == 0x00,
                    "Something is in way of EnPassant!"
                );
                self.arr[pawn.position()] = 0x00;
                self.arr[target.position()] = 0x00;
                self.arr[target.position.wrapping_add(step) as usize] = pawn.code;
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
                self.arr[target.position.wrapping_add(step) as usize] = 0x00;
                self.arr[pawn.position()] = pawn.code;
                self.arr[target.position()] = target.code;
            }
        }
    }

    pub fn who_can_attack(&self, target: Piece) -> Option<Vec<Piece>> {
        let attackers: Vec<_> = self
            .iter_pieces()
            .filter(|piece| {
                piece.type_().is_valid()
                    && piece.color() != target.color()
                    && piece.can_attack(target.position, self.arr)
            })
            .collect();
        if attackers.is_empty() {
            None
        } else {
            Some(attackers)
        }
    }

    fn is_attacked(&self, position: u8, color: Color) -> bool {
        self.iter_pieces()
            .find(|piece| {
                piece.type_().is_valid()
                    && piece.color() == color
                    && piece.can_attack(position, self.arr)
            })
            .is_some()
    }

    pub fn get_possible_moves(&self, color: Color, last_move: Move, bot: bool) -> Vec<Move> {
        // Check for pawn double push
        let enpassant_pawn = match last_move {
            Move::PawnDoublePush(pawn, pos) => Some(Piece::from_code(pawn.code, pos)),
            _ => None,
        };
        let mut possible_moves = Vec::with_capacity(256);
        for piece in self
            .iter_pieces()
            .filter(|piece| piece.color() == color && piece.type_().is_valid())
        {
            match piece.type_() {
                // Special cases
                PieceType::Pawn => {
                    let step: u8 = match color {
                        Color::Black => 0x10,
                        Color::White => 0xf0,
                    };
                    // push
                    let front_pos: u8 = piece.position.wrapping_add(step);
                    let in_front = self.arr[front_pos as usize];
                    let promotion = front_pos & 0xf0 == 0 || front_pos & 0xf0 == 0x70;
                    if in_front == 0x00 {
                        possible_moves.push({
                            if !promotion {
                                Move::QuietMove(piece.clone(), front_pos)
                            } else {
                                Move::PromotionQuiet(piece.clone(), front_pos, PieceType::Invalid)
                            }
                        });
                    }
                    // capture
                    for step in [0x01, 0xff] {
                        let pos = front_pos.wrapping_add(step);
                        if !is_valid_coord(pos) {
                            continue;
                        }
                        let cell = self.arr[pos as usize];
                        if cell != 0x00 && Color::from_byte(cell) != color {
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
                    if !PieceFlag::Moved.is_set(piece.code) && in_front == 0x00 {
                        let pos = front_pos.wrapping_add(step);
                        if self.arr[pos as usize] == 0x00 {
                            possible_moves.push(Move::PawnDoublePush(piece.clone(), pos))
                        }
                    }
                    // enpassant
                    if let Some(pawn) = &enpassant_pawn {
                        if pawn.position.abs_diff(piece.position) == 0x01 && pawn.color() != color {
                            possible_moves.push(Move::EnPassantCapture(piece, pawn.clone()))
                        }
                    }
                }
                PieceType::Knight => {
                    for pos in KNIGHT_MOVES
                        .iter()
                        .map(|off| off.wrapping_add(piece.position))
                        .filter(|pos| is_valid_coord(*pos))
                    {
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
                    for pos in KING_MOVES
                        .iter()
                        .map(|off| off.wrapping_add(piece.position))
                        .filter(|pos| is_valid_coord(*pos))
                    {
                        let cell = self.arr[pos as usize];
                        if cell == 0x00 {
                            possible_moves.push(Move::QuietMove(piece.clone(), pos));
                        } else if Color::from_byte(cell) != color {
                            possible_moves
                                .push(Move::Capture(piece.clone(), Piece::from_code(cell, pos)))
                        }
                    }
                    if PieceFlag::Moved.is_set(piece.code) {
                        break;
                    }
                    for (castling_side, piece_flag) in zip(
                        [CastlingSide::KingSide, CastlingSide::QueenSide],
                        [PieceFlag::CanCastleKingSide, PieceFlag::CanCastleQueenSide],
                    ) {
                        let rook_pos = piece.position & 0xf0 | castling_side as u8;
                        let cell = self.arr[rook_pos as usize];
                        if !PieceFlag::Moved.is_set(cell)
                            && Color::from_byte(cell) == color
                            && PieceType::from_byte(cell) == PieceType::Rook
                            && piece_flag.is_set(piece.code)
                            && between(rook_pos, piece.position)
                                .map(|pos| self.arr[pos as usize])
                                .all(|code| code == 0x00)
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
                        for pos in in_direction(piece.position, *dir) {
                            let cell = self.arr[pos as usize];
                            if cell == 0x00 {
                                possible_moves.push(Move::QuietMove(piece.clone(), pos));
                            } else if Color::from_byte(cell) != color {
                                possible_moves.push(Move::Capture(
                                    piece.clone(),
                                    Piece::from_code(cell, pos),
                                ));
                                break;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
        if bot {
            for idx in 0..possible_moves.len() {
                possible_moves[idx] = match possible_moves[idx].clone() {
                    Move::PromotionQuiet(pawn, pos, _) => {
                        for _type in [PieceType::Knight, PieceType::Rook, PieceType::Bishop] {
                            possible_moves.push(Move::PromotionQuiet(pawn.clone(), pos, _type));
                        }
                        Move::PromotionQuiet(pawn, pos, PieceType::Queen)
                    }
                    Move::PromotionCapture(pawn, piece, _) => {
                        for _type in [PieceType::Knight, PieceType::Rook, PieceType::Bishop] {
                            possible_moves.push(Move::PromotionCapture(
                                pawn.clone(),
                                piece.clone(),
                                _type,
                            ));
                        }
                        Move::PromotionCapture(pawn, piece, PieceType::Queen)
                    }
                    _move => _move,
                }
            }
        }
        possible_moves
    }

    pub fn is_checked(&self, color: Color) -> Option<(bool, Piece)> {
        self.iter_pieces()
            .find(|piece| piece.color() == color && matches!(piece.type_(), PieceType::King))
            .map(|piece| (self.is_attacked(piece.position, color.opposite()), piece))
    }

    pub fn castling_right_check(&self, king: Piece) -> (bool, bool) {
        if PieceFlag::Moved.is_set(king.code) {
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

    pub fn castling_rights(&mut self, king: Piece) {
        let cast_rights = self.castling_right_check(king.clone());
        self.arr[king.position()] = PieceFlag::set_kings_rights(king.code, cast_rights);
    }

    pub fn obstruct_board(&self, player: Color) -> Vec<Vec<bool>> {
        let mut mask = Vec::with_capacity(8);
        for _ in 0..8u8 {
            mask.push(vec![false; 8]);
        }
        for file in 0..8u8 {
            for rank in 0..8u8 {
                let pos = rank << 4 | file;
                let piece = Piece::from_code(self.arr[pos as usize], pos);
                if piece.code == 0x00 || piece.color() != player {
                    continue;
                }
                let (rank, file): (usize, usize) = unpack_pos(pos);
                mask[file][rank] = true;
                match piece.type_() {
                    PieceType::EmptySquare => unreachable!(),
                    PieceType::Invalid => unreachable!(),
                    PieceType::Pawn => {
                        let step: u8 = match player {
                            Color::Black => 0x10,
                            Color::White => 0xf0,
                        };
                        // push
                        let front_pos: u8 = piece.position.wrapping_add(step);
                        if !is_valid_coord(front_pos) {
                            continue;
                        }
                        // capture cells
                        for step in [0x01, 0xff] {
                            let pos = front_pos.wrapping_add(step);
                            if !is_valid_coord(pos) {
                                continue;
                            }
                            let (rank, file): (usize, usize) = unpack_pos(pos);
                            mask[file][rank] = true;
                        }
                        let cell = self.arr[front_pos as usize];
                        let (rank, file): (usize, usize) = unpack_pos(front_pos);
                        mask[file][rank] = true;
                        if cell == 0x00 {
                            let pos = front_pos.wrapping_add(step);
                            let (rank, file): (usize, usize) = unpack_pos(pos);
                            mask[file][rank] = true;
                        }
                    }
                    PieceType::Knight => {
                        for offset in KNIGHT_MOVES {
                            let pos = pos.wrapping_add(*offset);
                            if !is_valid_coord(pos) {
                                continue;
                            }
                            let (rank, file): (usize, usize) = unpack_pos(pos);
                            mask[file][rank] = true;
                        }
                    }
                    PieceType::King => {
                        for offset in KING_MOVES {
                            let pos = pos.wrapping_add(*offset);
                            if !is_valid_coord(pos) {
                                continue;
                            }
                            let (rank, file): (usize, usize) = unpack_pos(pos);
                            mask[file][rank] = true;
                        }
                    }
                    // Sliding pieces
                    sliding_piece => {
                        let possible_directions = match sliding_piece {
                            PieceType::Bishop => BISHOP_DIR,
                            PieceType::Rook => ROOK_DIR,
                            PieceType::Queen => QUEEN_DIR,
                            _ => panic!("Unreachable!"),
                        };
                        for dir in possible_directions {
                            let mut pos = pos.wrapping_add(*dir);
                            while is_valid_coord(pos) {
                                let (rank, file): (usize, usize) = unpack_pos(pos);
                                mask[file][rank] = true;
                                let cell = self.arr[pos as usize];
                                if cell != 0x00 {
                                    break;
                                }
                                pos = pos.wrapping_add(*dir);
                            }
                        }
                    }
                }
            }
        }
        mask
    }

    pub fn hide(mut self, point_of_view: Color) -> Self {
        ITER_INDEX.iter().for_each(|pos| {
            self.arr[*pos] = PieceFlag::UnknownCellFlag.set(self.arr[*pos]);
        });
        ITER_INDEX.iter().for_each(|pos| {
            let piece = Piece::from_code(self.arr[*pos], *pos as u8);
            if !piece.type_().is_valid() || piece.color() != point_of_view {
                return;
            }
            self.arr[*pos] = PieceFlag::UnknownCellFlag.unset(piece.code);
            match piece.type_() {
                PieceType::EmptySquare => unreachable!(),
                PieceType::Invalid => unreachable!(),
                PieceType::Pawn => {
                    let step: u8 = match piece.color() {
                        Color::Black => 0x10,
                        Color::White => 0xf0,
                    };
                    // push
                    let front_pos: u8 = piece.position.wrapping_add(step);
                    if !is_valid_coord(front_pos) {
                        return;
                    }
                    // capture cells
                    for step in [0x01, 0xff] {
                        let pos = front_pos.wrapping_add(step);
                        if !is_valid_coord(pos) {
                            continue;
                        }
                        self.arr[pos as usize] =
                            PieceFlag::UnknownCellFlag.unset(self.arr[pos as usize]);
                    }
                    let cell = PieceFlag::UnknownCellFlag.unset(self.arr[front_pos as usize]);
                    self.arr[front_pos as usize] = cell;
                    if cell == 0x00 {
                        let pos = front_pos.wrapping_add(step);
                        self.arr[pos as usize] =
                            PieceFlag::UnknownCellFlag.unset(self.arr[pos as usize]);
                    }
                }
                PieceType::Knight => {
                    for offset in KNIGHT_MOVES {
                        let pos = piece.position.wrapping_add(*offset);
                        if !is_valid_coord(pos) {
                            continue;
                        }
                        self.arr[pos as usize] =
                            PieceFlag::UnknownCellFlag.unset(self.arr[pos as usize]);
                    }
                }
                PieceType::King => {
                    for offset in KING_MOVES {
                        let pos = piece.position.wrapping_add(*offset);
                        if !is_valid_coord(pos) {
                            continue;
                        }
                        self.arr[pos as usize] =
                            PieceFlag::UnknownCellFlag.unset(self.arr[pos as usize]);
                    }
                }
                // Sliding pieces
                sliding_piece => {
                    let possible_directions = match sliding_piece {
                        PieceType::Bishop => BISHOP_DIR,
                        PieceType::Rook => ROOK_DIR,
                        PieceType::Queen => QUEEN_DIR,
                        _ => panic!("Unreachable!"),
                    };
                    for dir in possible_directions {
                        for pos in in_direction(piece.position, *dir) {
                            let cell = PieceFlag::UnknownCellFlag.unset(self.arr[pos as usize]);
                            self.arr[pos as usize] = cell;
                            if cell != 0x00 {
                                break;
                            }
                        }
                    }
                }
            }
        });
        self
    }

    pub fn hide_and_obstruct(self, point_of_view: Color) -> Self {
        let mut board = self.hide(point_of_view);
        ITER_INDEX.iter().for_each(|pos| {
            let code = board.arr[*pos];
            board.arr[*pos] = if PieceFlag::UnknownCellFlag.is_set(code) {
                PieceFlag::UnknownCellFlag as u8
            } else {
                code
            };
        });
        board
    }

    pub fn obstruct(self, player: Color) -> Self {
        let mask = self.obstruct_board(player);
        let mut obstructed_board = self;
        for file in 0..8u8 {
            for rank in 0..8u8 {
                let pos = rank << 4 | file;
                if mask[file as usize][rank as usize] {
                    obstructed_board.arr[pos as usize] = 0x00;
                }
            }
        }
        obstructed_board
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u8> + 'a {
        ITER_INDEX.iter().map(|&i| self.arr[i])
    }

    pub fn iter_pieces<'a>(&'a self) -> impl Iterator<Item = Piece> + 'a {
        ITER_INDEX
            .iter()
            .map(|&i| Piece::from_code(self.arr[i], i as u8))
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

const ITER_INDEX: [usize; 64] = {
    let mut arr = [0; 64];
    let mut file = 0;
    let mut rank = 0;
    while file < 8 {
        arr[file * 8 + rank] = rank << 4 | file;
        if rank < 7 {
            rank += 1;
        } else {
            rank = 0;
            file += 1;
        }
    }
    arr
};

/** Tables directions for pieces */
const BISHOP_DIR: &[u8] = &[0x11, 0x0f, 0xef, 0xf1];
const ROOK_DIR: &[u8] = &[0x10, 0xff, 0xf0, 0x01];
const QUEEN_DIR: &[u8] = &[0x11, 0x0f, 0xef, 0xf1, 0x10, 0xff, 0xf0, 0x01];

/** Possible moves for pieces */
const KING_MOVES: &[u8] = QUEEN_DIR;
const KNIGHT_MOVES: &[u8] = &[0x12, 0x21, 0x1f, 0x0e, 0xee, 0xdf, 0xe1, 0xf2];

/** Bits structure of piece code
 * Bit 7 -- Color of the piece
 * - 1 -- Black
 * - 0 -- White
 * Bit 6 -- Unknown flag
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
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Piece {
    code: u8,
    position: u8,
}

pub enum PieceFlag {
    /** Bit 3 -- Piece has moved flag */
    Moved = 0x08,
    /** Bit 4 -- Castle flag for Kings only - KingSide */
    CanCastleKingSide = 0x10,
    /** Bit 5 -- Castle flag for Kings only - QueenSide */
    CanCastleQueenSide = 0x20,
    /** Bit 6 -- That's cell isn't given to us */
    UnknownCellFlag = 0x40,
}

impl PieceFlag {
    pub fn is_set(self, code: u8) -> bool {
        code & self as u8 != 0
    }

    fn set(self, code: u8) -> u8 {
        code | self as u8
    }

    fn unset(self, code: u8) -> u8 {
        code & (0xff ^ self as u8)
    }

    fn set_kings_rights(king: u8, rights: (bool, bool)) -> u8 {
        let (lr, rr) = rights;
        king & !(PieceFlag::CanCastleKingSide as u8 | PieceFlag::CanCastleQueenSide as u8)
            | (PieceFlag::CanCastleKingSide as u8 * lr as u8)
            | (PieceFlag::CanCastleQueenSide as u8 * rr as u8)
    }
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

    fn can_attack(&self, target: u8, board: [u8; 128]) -> bool {
        // precomputed tables should increase speed of this dramatically
        match self.type_() {
            PieceType::Pawn => {
                let step: u8 = match self.color() {
                    Color::White => 0x10,
                    Color::Black => 0xf0,
                };
                distance(self.position, target) == 2
                    && (self.position & 0xf0).wrapping_add(step) == target & 0xf0
            }
            PieceType::Bishop => {
                is_in_diagonal_line(self.position, target)
                    && between(self.position, target)
                        .map(|pos| board[pos as usize])
                        .all(|cell| cell == 0x00)
            }
            PieceType::Rook => {
                is_in_straight_line(self.position, target)
                    && between(self.position, target)
                        .map(|pos| board[pos as usize])
                        .all(|cell| cell == 0x00)
            }
            PieceType::Queen => {
                (is_in_straight_line(self.position, target)
                    || is_in_diagonal_line(self.position, target))
                    && between(self.position, target)
                        .map(|pos| board[pos as usize])
                        .all(|cell| cell == 0x00)
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

#[rifgen::rifgen_attr::generate_interface]
#[derive(PartialEq, Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum Color {
    Black = 0x00,
    #[default]
    White = 0x80,
}

impl Color {
    #[inline]
    fn from_byte(byte: u8) -> Color {
        unsafe { std::mem::transmute(byte & 0x80) }
    }

    pub fn opposite(self) -> Color {
        if self == Color::White {
            Color::Black
        } else {
            Color::White
        }
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        Color::from_byte(value)
    }
}

impl From<Color> for u8 {
    fn from(value: Color) -> Self {
        value as u8
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(if self == &Self::White {
            "White"
        } else {
            "Black"
        })
    }
}

#[rifgen::rifgen_attr::generate_interface]
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
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

    fn is_valid(&self) -> bool {
        matches!(
            self,
            Self::Pawn | Self::Knight | Self::Bishop | Self::Rook | Self::Queen | Self::King
        )
    }
}

impl From<u8> for PieceType {
    fn from(value: u8) -> Self {
        PieceType::from_byte(value)
    }
}

impl From<PieceType> for u8 {
    fn from(value: PieceType) -> Self {
        value as u8
    }
}
