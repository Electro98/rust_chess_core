#![allow(dead_code)]

use std::fmt::{write, Display};
use std::{fmt::Debug, iter::zip};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

use crate::core::definitions::ImplicitMove;
use crate::core::utils::{
    between, compact_pos, distance, in_direction, is_in_diagonal_line, is_in_straight_line,
    is_valid_coord, pos_to_str, unpack_pos,
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CastlingSide {
    KingSide = 0x07,
    QueenSide = 0x00,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MoveType {
    /** who is moving, where it's moving */
    QuietMove(u8),
    /** who is capturing, whom is beeing captured */
    Capture(Piece),
    /** king, which side */
    Castling(CastlingSide, Piece),
    /** who is promoting, where it goes, to whom we are promoting */
    PromotionQuiet(u8, PieceType),
    /** who is promoting, whom is beeing captured, to whom we are promoting */
    PromotionCapture(Piece, PieceType),
    /** who is moving, where it's moving */
    PawnDoublePush(u8),
    /** who is capturing, whom is beeing captured, where to move */
    EnPassantCapture(Piece, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CheckType {
    None,
    Direct,
    Discovered,
    Double,
}

impl CheckType {
    fn from_bools(direct: bool, discovered: bool) -> Self {
        match (direct, discovered) {
            (true, true) => Self::Double,
            (true, false) => Self::Direct,
            (false, true) => Self::Discovered,
            (false, false) => Self::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    piece: Piece,
    move_type: MoveType,
    /** Does this move induce the check? */
    check: CheckType,
}

impl Move {
    #[cfg(debug_assertions)]
    pub fn new_debug(piece: Piece, move_type: MoveType, check: CheckType) -> Move {
        Move {
            piece,
            move_type,
            check,
        }
    }

    pub fn piece(&self) -> &Piece {
        &self.piece
    }

    pub fn end_position(&self) -> u8 {
        match &self.move_type {
            MoveType::QuietMove(pos) => *pos,
            MoveType::Capture(piece) => piece.position() as u8,
            MoveType::Castling(_, piece) => piece.position() as u8,
            MoveType::PromotionQuiet(pos, _) => *pos,
            MoveType::PromotionCapture(piece, _) => piece.position() as u8,
            MoveType::PawnDoublePush(pos) => *pos,
            MoveType::EnPassantCapture(_, pos) => *pos,
        }
    }

    pub fn move_type(&self) -> &MoveType {
        &self.move_type
    }

    pub fn check(&self) -> CheckType {
        self.check
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            pos_to_str(self.piece.position),
            pos_to_str(self.end_position())
        )
    }
}

impl ImplicitMove for Move {
    fn promotion(&self) -> bool {
        matches!(
            self.move_type,
            MoveType::PromotionQuiet(..) | MoveType::PromotionCapture(..)
        )
    }

    fn set_promotion_type(&mut self, new_type: PieceType) {
        match &mut self.move_type {
            MoveType::PromotionQuiet(_, _type) => *_type = new_type,
            MoveType::PromotionCapture(_, _type) => *_type = new_type,
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

type CompressedBoard = [u32; 8];

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

    #[cfg(debug_assertions)]
    pub fn new_debug(arr: &[u8; 64]) -> Board {
        let mut board = Self::new();
        for i in 0..8 {
            for j in 0..8 {
                board.arr[compact_pos(j, i) as usize] = arr[(j * 8 + i) as usize];
            }
        }
        board
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
        use MoveType::*;
        let piece = _move.piece;
        match _move.move_type {
            QuietMove(new_pos) => {
                assert!(
                    self.arr[new_pos as usize] == 0x00,
                    "Trying to move in busy place!"
                );
                self.arr[piece.position()] = 0x00;
                self.arr[new_pos as usize] = piece.code | PieceFlag::Moved as u8;
            }
            Capture(target) => {
                assert!(
                    piece.color() != target.color(),
                    "That's a bug! Piece captured teammate!"
                );
                self.arr[piece.position()] = 0x00;
                self.arr[target.position()] = piece.code | PieceFlag::Moved as u8;
            }
            PawnDoublePush(new_pos) => {
                assert!(
                    self.arr[new_pos as usize] == 0x00,
                    "Trying to move in busy place!"
                );
                self.arr[piece.position()] = 0x00;
                self.arr[new_pos as usize] = piece.code | PieceFlag::Moved as u8;
            }
            PromotionQuiet(new_pos, new_type) => {
                assert!(
                    piece.type_() == PieceType::Pawn,
                    "Trying to promote non-pawn piece!"
                );
                assert!(
                    self.arr[new_pos as usize] == 0x00,
                    "Trying to move in busy place!"
                );
                self.arr[piece.position()] = 0x00;
                self.arr[new_pos as usize] =
                    piece.color() as u8 | new_type as u8 | PieceFlag::Moved as u8;
            }
            PromotionCapture(target, new_type) => {
                assert!(
                    piece.type_() == PieceType::Pawn,
                    "Trying to promote non-pawn piece!"
                );
                assert!(
                    piece.color() != target.color(),
                    "That's a bug! Pawn captured teammate!"
                );
                self.arr[piece.position()] = 0x00;
                self.arr[target.position()] =
                    piece.color() as u8 | new_type as u8 | PieceFlag::Moved as u8;
            }
            Castling(castling_side, rook) => {
                // Checks for king
                assert!(
                    piece.type_() == PieceType::King,
                    "Trying to castle without king?!"
                );
                assert!(
                    !PieceFlag::Moved.is_set(piece.code),
                    "King have already moved!"
                );
                assert!(
                    ((castling_side == CastlingSide::KingSide
                        && PieceFlag::CanCastleKingSide.is_set(piece.code))
                        || (castling_side == CastlingSide::QueenSide
                            && PieceFlag::CanCastleQueenSide.is_set(piece.code))),
                    "King can't castle!"
                );
                // TODO: check if king crosses square under attack (castle rights bits)
                // Checks for rook
                assert!(
                    rook.type_() == PieceType::Rook,
                    "Non-rook piece used for castling!"
                );
                assert!(
                    rook.color() == piece.color(),
                    "Enemy rook used for castling!"
                );
                assert!(
                    !PieceFlag::Moved.is_set(rook.code),
                    "Moved rook used for castling!"
                );
                #[cfg(debug_assertions)]
                for cell in between(rook.position, piece.position) {
                    assert!(
                        self.arr[cell as usize] == 0x00,
                        "There is something in way of castling!"
                    );
                }
                self.arr[piece.position()] = 0x00;
                self.arr[rook.position()] = 0x00;
                let rank = piece.position & 0xf0;
                match castling_side {
                    CastlingSide::KingSide => {
                        self.arr[(rank | 0x06) as usize] = piece.code | PieceFlag::Moved as u8;
                        self.arr[(rank | 0x05) as usize] = rook.code | PieceFlag::Moved as u8;
                    }
                    CastlingSide::QueenSide => {
                        self.arr[(rank | 0x02) as usize] = piece.code | PieceFlag::Moved as u8;
                        self.arr[(rank | 0x03) as usize] = rook.code | PieceFlag::Moved as u8;
                    }
                }
            }
            EnPassantCapture(target, new_pos) => {
                assert!(
                    piece.type_() == PieceType::Pawn,
                    "Trying to use EnPassant by non-pawn piece!"
                );
                assert!(
                    target.type_() == PieceType::Pawn,
                    "Trying to capture non-pawn piece with EnPassant!"
                );
                assert!(
                    piece.color() != target.color(),
                    "That's a bug! Pawn captured teammate!"
                );
                assert!(
                    self.arr[new_pos as usize] == 0x00,
                    "Something is in way of EnPassant!"
                );
                self.arr[piece.position()] = 0x00;
                self.arr[target.position()] = 0x00;
                self.arr[new_pos as usize] = piece.code;
            }
        }
    }

    /** Undo valid move. */
    pub fn undo(&mut self, _move: Move) {
        use MoveType::*;
        let piece = _move.piece;
        match _move.move_type {
            QuietMove(moved_to) => {
                self.arr[moved_to as usize] = 0x00;
                self.arr[piece.position()] = piece.code;
            }
            Capture(target) => {
                self.arr[target.position()] = target.code;
                self.arr[piece.position()] = piece.code;
            }
            PawnDoublePush(moved_to) => {
                self.arr[moved_to as usize] = 0x00;
                self.arr[piece.position()] = piece.code;
            }
            PromotionQuiet(moved_to, _) => {
                self.arr[moved_to as usize] = 0x00;
                self.arr[piece.position()] = piece.code;
            }
            PromotionCapture(target, _) => {
                self.arr[target.position()] = target.code;
                self.arr[piece.position()] = piece.code;
            }
            Castling(castling_side, rook) => {
                let rank = piece.position & 0xf0;
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
                self.arr[piece.position()] = piece.code;
                self.arr[rook.position()] = rook.code;
            }
            EnPassantCapture(target, new_pos) => {
                self.arr[new_pos as usize] = 0x00;
                self.arr[piece.position()] = piece.code;
                self.arr[target.position()] = target.code;
            }
        }
    }

    pub fn who_can_attack(&self, target: Piece) -> Option<Vec<Piece>> {
        let attackers: Vec<_> = self
            .iter_pieces()
            .filter(|piece| {
                piece.color() != target.color()
                    && piece.type_().is_valid()
                    && piece.can_attack(target.position, self.arr)
            })
            .collect();
        if attackers.is_empty() {
            None
        } else {
            Some(attackers)
        }
    }

    fn count_pinned_pieces(
        &self,
        target_king: Piece,
        enpassant_pawn: Option<u8>,
    ) -> Vec<(Piece, Piece)> {
        let pinned_pieces: Vec<_> = self
            .iter_pieces()
            .filter(|attacker| {
                attacker.color() != target_king.color()
                    && match attacker.type_() {
                        PieceType::Bishop => {
                            is_in_diagonal_line(attacker.position, target_king.position)
                        }
                        PieceType::Rook => {
                            is_in_straight_line(attacker.position, target_king.position)
                        }
                        PieceType::Queen => {
                            is_in_diagonal_line(attacker.position, target_king.position)
                                || is_in_straight_line(attacker.position, target_king.position)
                        }
                        // No one else cannot pin another piece
                        _ => false,
                    }
            })
            .filter_map(|attacker| {
                let mut pinned_piece = None;
                // TODO: Remove?
                let mut possible_enpassant = false;
                for pos in between(attacker.position, target_king.position) {
                    let code = self.arr[pos as usize];
                    if code != 0x00 {
                        if pinned_piece.is_none() {
                            let piece = Piece::from_code(code, pos);
                            possible_enpassant = piece.type_() == PieceType::Pawn
                                && enpassant_pawn
                                    .map_or(false, |pawn| is_in_straight_line(pos, pawn));
                            pinned_piece = Some(piece);
                        // } else if possible_enpassant && matches!(PieceType::from_byte(code), PieceType::Pawn) && is_in_straight_line(enpassant_pawn.unwrap(), pos) {
                        } else {
                            pinned_piece = None;
                            break;
                        }
                    }
                }
                pinned_piece.map(|piece| (piece, attacker))
            })
            .collect();
        pinned_pieces
    }

    fn is_attacked(&self, position: u8, by_color: Color) -> bool {
        self.iter_pieces()
            .find(|piece| {
                piece.color() == by_color
                    && piece.type_().is_valid()
                    && piece.can_attack(position, self.arr)
            })
            .is_some()
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
                            Color::Black => 0xf0,
                            Color::White => 0x10,
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
                        Color::Black => 0xf0,
                        Color::White => 0x10,
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
        for rank in 0..8u8 {
            for file in 0..8u8 {
                let pos = rank << 4 | file;
                if mask[file as usize][rank as usize] {
                    obstructed_board.arr[pos as usize] = 0x00;
                }
            }
        }
        obstructed_board
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u8> + 'a {
        ITER_INDEX.iter().map(|&i| self.arr[i])
    }

    #[inline]
    pub fn iter_pieces<'a>(&'a self) -> impl Iterator<Item = Piece> + 'a {
        ITER_INDEX
            .iter()
            .map(|&i| Piece::from_code(self.arr[i], i as u8))
    }

    pub fn compress(&self) -> CompressedBoard {
        let mut compressed_board = [0; 8];
        for rank in 0..8u8 {
            let mut col: u32 = 0;
            for file in 0..8u8 {
                let pos = compact_pos(rank, file) as usize;
                let cell = self.arr[pos] & 0x80 >> 4 | self.arr[pos] & 0x05;
                col |= (cell as u32) << (file * 4);
            }
            compressed_board[rank as usize] = col;
        }
        compressed_board
    }
}

impl Default for Board {
    #[rustfmt::skip]
    fn default() -> Self {
        Board {
            arr: [
                0x84, 0x82, 0x83, 0x85, 0x86, 0x83, 0x82, 0x84, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x04, 0x02, 0x03, 0x05, 0x06, 0x03, 0x02, 0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            ]
        }
    }
}

const ITER_INDEX: [usize; 64] = {
    let mut arr = [0; 64];
    let mut file = 0;
    let mut rank = 0;
    while file < 8 {
        arr[rank * 8 + file] = rank << 4 | file;
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

#[derive(Clone, Copy, Debug)]
pub enum GameEndState {
    CheckMate,
    DrawStalemate,
    DrawTreefoldRepetion,
    DrawFiftyMoveRule,
    DrawInsufficientMaterial,
}

fn flag_piece_moved(piece: PieceType, color: Color, pos: u8) -> u8 {
    let (rank, file): (u8, u8) = unpack_pos(pos);
    let right_file = match piece {
        PieceType::Pawn => true,
        PieceType::Rook => file == 0x0 || file == 0x7,
        PieceType::Knight => file == 0x1 || file == 0x6,
        PieceType::Bishop => file == 0x2 || file == 0x5,
        PieceType::Queen => file == 0x3,
        PieceType::King => file == 0x4,
        PieceType::EmptySquare => true,
        PieceType::Invalid => unreachable!("Created an invalid piece?"),
    };
    let is_pawn = matches!(piece, PieceType::Pawn);
    let right_rank = match color {
        Color::Black => (is_pawn && rank == 0x06) || (!is_pawn && rank == 0x07),
        Color::White => (is_pawn && rank == 0x01) || (!is_pawn && rank == 0x00),
    };
    if right_file && right_rank {
        0
    } else {
        PieceFlag::Moved as u8
    }
}

impl Game {
    #[cfg(debug_assertions)]
    pub fn new_debug(board: Board, current_player: Color, last_move: Option<Move>) -> Game {
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
        for rank in (0..8).rev() {
            let mut file = 0;
            while let Some(letter) = chars.next() {
                if let Some(num) = letter.to_digit(10) {
                    file += num;
                    continue;
                }
                if file == 8 {
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
                let pos = compact_pos(rank as u8, file as u8);
                board.arr[pos as usize] =
                    piece as u8 | color as u8 | flag_piece_moved(piece, color, pos);
                file += 1;
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
        while let Some(letter) = chars.next() {
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
                let file = match letter.to_ascii_lowercase() {
                    'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' => letter as u8 - 'a' as u8,
                    _ => return Err("Error in move file".to_string()),
                };
                let mut rank = chars.next().unwrap() as u8 - '0' as u8;
                if current_player == Color::White {
                    rank -= 2;
                }
                let pos = compact_pos(rank, file);
                let piece = Piece::from_code(board.arr[pos as usize], pos);
                if piece.type_() == PieceType::Pawn {
                    Some(Move {
                        piece,
                        move_type: MoveType::PawnDoublePush(pos),
                        check: CheckType::None,
                    })
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
                    && piece.can_attack(king.position, board.arr)
            })
            .collect();
        if let Some(check) = match attackers.len() {
            0 => None,
            1 if last_move
                .as_ref()
                .map_or(false, |_move| _move.end_position() == attackers[0].position) =>
            {
                Some(CheckType::Direct)
            }
            1 => Some(CheckType::Discovered),
            _ => Some(CheckType::Double),
        } {
            last_move = last_move.map_or(
                Some(Move {
                    piece: Piece::from_code(0xff, 0xff),
                    move_type: MoveType::QuietMove(0xff),
                    check,
                }),
                |last_move| Some(Move { check, ..last_move }),
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
        // Check for pawn double push
        let last_move = self.history.last_move();
        let enpassant_pawn = match last_move.as_ref().map(|_move| _move.move_type) {
            Some(MoveType::PawnDoublePush(_)) => Some(last_move.unwrap().end_position()),
            _ => None,
        };
        let mut possible_moves = Vec::with_capacity(256);
        // Count pinned pieces
        let (king, enemy_king) = {
            let mut king = None;
            let mut enemy_king = None;
            for piece in self.board.iter_pieces() {
                if matches!(piece.type_(), PieceType::King) {
                    if piece.color() == self.current_player {
                        king = Some(piece);
                    } else {
                        enemy_king = Some(piece);
                    }
                    if king.is_some() && enemy_king.is_some() {
                        break;
                    }
                }
            }
            (
                king.expect("King of current player should be present to make a move"),
                enemy_king,
            )
        };
        let king_in_check = self.history.last_move().map(|_move| _move.check);
        let pinned_pieces = self.board.count_pinned_pieces(king, None);

        for piece in self
            .board
            .iter_pieces()
            .filter(|piece| piece.color() == self.current_player && piece.type_().is_valid())
        {
            let (attacker, possible_positions) = if let Some((_, attacker)) = pinned_pieces
                .iter()
                .find(|(pinned_piece, _)| *pinned_piece == piece)
            {
                let possible_possitions: Vec<_> = between(attacker.position, king.position)
                    .filter(|pos| self.board.arr[*pos as usize] == 0x00)
                    .collect();
                (Some(attacker), Some(possible_possitions))
            } else {
                (None, None)
            };
            if matches!(king_in_check, Some(CheckType::Double)) && piece.type_() != PieceType::King
            {
                continue;
            }
            match piece.type_() {
                // Special cases
                PieceType::Pawn => {
                    let step: u8 = match self.current_player {
                        Color::Black => 0xf0,
                        Color::White => 0x10,
                    };
                    // push
                    let front_pos: u8 = piece.position.wrapping_add(step);
                    let in_front = self.board.arr[front_pos as usize];
                    let promotion = front_pos & 0xf0 == 0 || front_pos & 0xf0 == 0x70;
                    if in_front == 0x00
                        && possible_positions
                            .as_ref()
                            .map_or(true, |positions| positions.contains(&front_pos))
                    {
                        possible_moves.push({
                            Move {
                                piece,
                                move_type: if !promotion {
                                    MoveType::QuietMove(front_pos)
                                } else {
                                    MoveType::PromotionQuiet(front_pos, PieceType::Invalid)
                                },
                                check: CheckType::None,
                            }
                        });
                    }
                    // capture
                    for step in [0x01, 0xff] {
                        let pos = front_pos.wrapping_add(step);
                        if !is_valid_coord(pos) {
                            continue;
                        }
                        let cell = self.board.arr[pos as usize];
                        if cell != 0x00
                            && Color::from_byte(cell) != self.current_player
                            && attacker.map_or(true, |attacker| attacker.position == pos)
                        {
                            possible_moves.push({
                                let target = Piece::from_code(cell, pos);
                                Move {
                                    piece,
                                    move_type: if !promotion {
                                        MoveType::Capture(target)
                                    } else {
                                        MoveType::PromotionCapture(target, PieceType::Invalid)
                                    },
                                    check: CheckType::None,
                                }
                            });
                        }
                    }
                    // double push
                    if !PieceFlag::Moved.is_set(piece.code) && in_front == 0x00 {
                        let pos = front_pos.wrapping_add(step);
                        if self.board.arr[pos as usize] == 0x00
                            && possible_positions
                                .as_ref()
                                .map_or(true, |positions| positions.contains(&pos))
                        {
                            possible_moves.push(Move {
                                piece,
                                move_type: MoveType::PawnDoublePush(pos),
                                check: CheckType::None,
                            })
                        }
                    }
                    // enpassant
                    if let Some(pawn_pos) = &enpassant_pawn {
                        if pawn_pos.abs_diff(piece.position) == 0x01
                            && attacker.map_or(true, |attacker| attacker.position == *pawn_pos)
                        {
                            let step: u8 = match piece.color() {
                                Color::Black => 0xf0,
                                Color::White => 0x10,
                            };
                            let enpassant = Move {
                                piece,
                                move_type: MoveType::EnPassantCapture(
                                    Piece::from_code(self.board.arr[*pawn_pos as usize], *pawn_pos),
                                    pawn_pos.wrapping_add(step),
                                ),
                                check: CheckType::None,
                            };
                            let mut temp_board = self.board.clone();
                            temp_board.execute(enpassant.clone());
                            if !temp_board
                                .is_attacked(king.position, self.current_player.opposite())
                            {
                                // temp_board.arr[pawn_pos.wrapping_add(step) as usize] = 0xff;
                                possible_moves.push(Move {
                                    check: if enemy_king.map_or(false, |enemy_king| {
                                        temp_board
                                            .is_attacked(enemy_king.position, self.current_player)
                                    }) {
                                        CheckType::Discovered
                                    } else {
                                        CheckType::None
                                    },
                                    ..enpassant
                                });
                            }
                        }
                    }
                }
                PieceType::Knight => {
                    // Pinned horse cannot do anything
                    if attacker.is_some() {
                        continue;
                    }
                    for pos in KNIGHT_MOVES
                        .iter()
                        .map(|off| off.wrapping_add(piece.position))
                        .filter(|pos| is_valid_coord(*pos))
                    {
                        let cell = self.board.arr[pos as usize];
                        if cell == 0x00 {
                            possible_moves.push(Move {
                                piece,
                                move_type: MoveType::QuietMove(pos),
                                check: CheckType::None,
                            })
                        } else if Color::from_byte(cell) != self.current_player {
                            possible_moves.push(Move {
                                piece,
                                move_type: MoveType::Capture(Piece::from_code(cell, pos)),
                                check: CheckType::None,
                            })
                        }
                    }
                }
                PieceType::King => {
                    // This is to prevent king itself from defending some position
                    let mut kingless_board = self.board.clone();
                    kingless_board.arr[piece.position as usize] = 0x00;
                    for pos in KING_MOVES
                        .iter()
                        .map(|off| off.wrapping_add(piece.position))
                        .filter(|pos| is_valid_coord(*pos))
                        .filter(|pos| {
                            !kingless_board.is_attacked(*pos, self.current_player.opposite())
                        })
                    {
                        let cell = self.board.arr[pos as usize];
                        if cell == 0x00 {
                            possible_moves.push(Move {
                                piece,
                                move_type: MoveType::QuietMove(pos),
                                check: CheckType::None,
                            })
                        } else if Color::from_byte(cell) != self.current_player {
                            possible_moves.push(Move {
                                piece,
                                move_type: MoveType::Capture(Piece::from_code(cell, pos)),
                                check: CheckType::None,
                            })
                        }
                    }
                    if PieceFlag::Moved.is_set(piece.code)
                        || king_in_check.map_or(false, |check| !matches!(check, CheckType::None))
                    {
                        continue;
                    }
                    for (castling_side, piece_flag) in zip(
                        [CastlingSide::KingSide, CastlingSide::QueenSide],
                        [PieceFlag::CanCastleKingSide, PieceFlag::CanCastleQueenSide],
                    ) {
                        let rook_pos = piece.position & 0xf0 | castling_side as u8;
                        let cell = self.board.arr[rook_pos as usize];
                        if !PieceFlag::Moved.is_set(cell)
                            && Color::from_byte(cell) == self.current_player
                            && PieceType::from_byte(cell) == PieceType::Rook
                            && piece_flag.is_set(piece.code)
                            && between(rook_pos, piece.position)
                                .map(|pos| (self.board.arr[pos as usize], pos))
                                .enumerate()
                                .all(|(i, (code, pos))| {
                                    code == 0x00
                                        && ((castling_side == CastlingSide::QueenSide && i == 0)
                                            || !self
                                                .board
                                                .is_attacked(pos, self.current_player.opposite()))
                                })
                        {
                            possible_moves.push(Move {
                                piece,
                                move_type: MoveType::Castling(
                                    castling_side,
                                    Piece::from_code(cell, rook_pos),
                                ),
                                check: CheckType::None,
                            })
                        }
                    }
                }
                // Invalid block
                PieceType::Invalid => unreachable!("That's bug! Invalid square in valid space!"),
                PieceType::EmptySquare => unreachable!("Empty square can't move"),
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
                            let cell = self.board.arr[pos as usize];
                            if cell == 0x00
                                && possible_positions
                                    .as_ref()
                                    .map_or(true, |possitions| possitions.contains(&pos))
                            {
                                possible_moves.push(Move {
                                    piece,
                                    move_type: MoveType::QuietMove(pos),
                                    check: CheckType::None,
                                });
                            } else if Color::from_byte(cell) != self.current_player
                                && attacker.map_or(true, |attacker| attacker.position == pos)
                            {
                                possible_moves.push(Move {
                                    piece,
                                    move_type: MoveType::Capture(Piece::from_code(cell, pos)),
                                    check: CheckType::None,
                                });
                                break;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
        if matches!(
            king_in_check,
            Some(CheckType::Direct) | Some(CheckType::Discovered)
        ) {
            // let attack_pieces = self
            //     .board
            //     .who_can_attack(king)
            //     .expect("Incorrect check: attacker is not found");
            let attack_pieces = if let Some(pieces) = self.board.who_can_attack(king) {
                pieces
            } else {
                println!("debug");
                panic!("Incorrect check: attacker is not found");
            };
            assert!(
                attack_pieces.len() == 1,
                "There can be only one piece to attack the king"
            );
            let attacker = attack_pieces.into_iter().next().unwrap();
            let possible_positions: Vec<_> = if matches!(
                attacker.type_(),
                PieceType::Bishop | PieceType::Queen | PieceType::Rook
            ) {
                between(attacker.position, king.position).collect()
            } else {
                Vec::new()
            };
            possible_moves = possible_moves
                .into_iter()
                .filter(|_move| match _move.move_type() {
                    _ if _move.piece().type_() == PieceType::King => true,
                    MoveType::EnPassantCapture(pawn, _) if pawn == &attacker => true,
                    _ => {
                        possible_positions.contains(&_move.end_position())
                            || _move.end_position() == attacker.position
                    }
                })
                .collect();
        }
        if bot {
            for idx in 0..possible_moves.len() {
                let piece = possible_moves[idx].piece;
                possible_moves[idx].move_type = match possible_moves[idx].move_type.clone() {
                    MoveType::PromotionQuiet(pos, _) => {
                        for _type in [PieceType::Knight, PieceType::Rook, PieceType::Bishop] {
                            possible_moves.push(Move {
                                piece,
                                move_type: MoveType::PromotionQuiet(pos, _type),
                                check: CheckType::None,
                            });
                        }
                        MoveType::PromotionQuiet(pos, PieceType::Queen)
                    }
                    MoveType::PromotionCapture(target, _) => {
                        for _type in [PieceType::Knight, PieceType::Rook, PieceType::Bishop] {
                            possible_moves.push(Move {
                                piece,
                                move_type: MoveType::PromotionCapture(target, _type),
                                check: CheckType::None,
                            });
                        }
                        MoveType::PromotionCapture(target, PieceType::Queen)
                    }
                    _move => _move,
                }
            }
        }
        // Find discovery checks and direct checks
        // This can be not executed on dark chess client
        if let Some(enemy_king) = enemy_king {
            let pinned_pieces = self.board.count_pinned_pieces(enemy_king, enpassant_pawn);
            for _move in possible_moves.iter_mut() {
                let direct_check = match _move.move_type {
                    MoveType::PromotionQuiet(_, new_type)
                    | MoveType::PromotionCapture(_, new_type) => {
                        let piece = _move.piece();
                        !matches!(new_type, PieceType::Invalid)
                            && Piece::from_code(
                                piece.color() as u8 | new_type as u8,
                                _move.end_position(),
                            )
                            .can_attack(enemy_king.position, self.board.arr)
                    }
                    _ => Piece::from_code(_move.piece().code, _move.end_position())
                        .can_attack(enemy_king.position, self.board.arr),
                };
                let discovered_check = matches!(_move.check(), CheckType::Discovered)
                    || pinned_pieces
                        .iter()
                        .find(|(piece, _)| piece == _move.piece())
                        .map_or(false, |(_, attacker)| {
                            between(enemy_king.position, attacker.position)
                                .find(|pos| *pos == _move.end_position())
                                .is_none()
                        });
                _move.check = CheckType::from_bools(direct_check, discovered_check);
            }
        }
        possible_moves
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
            Move {
                check: if new_piece.can_attack(enemy_king.position, self.board.arr) {
                    if matches!(_move.check(), CheckType::Discovered | CheckType::Double) {
                        CheckType::Double
                    } else {
                        CheckType::Direct
                    }
                } else {
                    _move.check
                },
                .._move
            }
        } else {
            _move
        };
        self.history.record(_move.clone());
        let compressed_board = self.board.compress();
        match _move.move_type() {
            MoveType::QuietMove(_) => {
                if self.existed_positions.count(&compressed_board) >= 2 {
                    return Some(GameEndState::DrawTreefoldRepetion);
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
            match _move.check {
                CheckType::None => Some(GameEndState::DrawStalemate),
                _ => Some(GameEndState::CheckMate),
            }
        } else {
            None
        }
    }

    pub fn undo_last_move(&mut self) -> Result<(), &'static str> {
        let last_move = self
            .history
            .last_move()
            .ok_or_else(|| "There's no move to undo.")?;
        self.existed_positions.undo_move(*last_move.move_type());
        self.board.undo(last_move);
        self.history.unrecord();
        self.current_player = self.current_player.opposite();
        Ok(())
    }

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
                    if self.history.last_move().map_or(false, |last_move| {
                        let last_piece = match last_move.move_type() {
                            MoveType::PromotionCapture(_, new_type)
                            | MoveType::PromotionQuiet(_, new_type) => Piece::from_code(
                                last_move.piece.color() as u8
                                    | *new_type as u8
                                    | PieceFlag::Moved as u8,
                                last_move.end_position(),
                            ),
                            _ => Piece::from_code(
                                last_move.piece.code | PieceFlag::Moved as u8,
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
                _ => unreachable!("Should not be possiable in legal chess game"),
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

/** Bits structure of piece code
 * Bit 7 -- Color of the piece
 * - 0 -- Black
 * - 1 -- White
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
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
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
        // No pieces can attact itself
        if self.position == target {
            return false;
        }
        match self.type_() {
            PieceType::Pawn => {
                let step: u8 = match self.color() {
                    Color::White => 0x10,
                    Color::Black => 0xf0,
                };
                distance(self.position, target) == 2
                    && self.position.wrapping_add(step) & 0xf0 == target & 0xf0
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
            PieceType::King => {
                let rank_diff = (self.position & 0xf0).abs_diff(target & 0xf0) >> 4;
                let file_diff = (self.position & 0x0f).abs_diff(target & 0x0f);
                (0 == rank_diff || rank_diff == 1) && (0 == file_diff || 1 == file_diff)
            }
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

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
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
