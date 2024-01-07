#![allow(dead_code)]
use log::{debug, trace};


#[derive(Clone, Copy)]
enum CastlingSide { KingSide = 0x07, QueenSide = 0x00}

enum Move {
    /** skip of move, probably will be deleted */
    NullMove,
    /** who is moving, where it's moving */
    QuietMove(Piece, u8),
    /** who is capturing, whom is beeing captured */
    Capture(Piece, Piece),
    /** king, which side */
    Castling(Piece, CastlingSide),
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
struct Board {
    arr: [u8; 128],
}

impl Board {
    fn new() -> Board {
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

    /** Execute ***valid*** move. */
    fn execute(&mut self, _move: Move) {
        use Move::*;
        match _move {
            NullMove => trace!("Detected NullMove! Is it intentional?"),
            QuietMove(piece, move_to) => {
                assert!(self.arr[move_to as usize] == 0x00, "Trying to move in busy place!");
                self.arr[piece.position()] = 0x00;
                self.arr[move_to as usize] = piece.code | PieceFlag::Moved as u8;
            },
            Capture(piece, target) => {
                assert!(piece.color() != target.color(), "That's a bug! Piece captured teammate!");
                self.arr[piece.position()] = 0x00;
                self.arr[target.position()] = piece.code | PieceFlag::Moved as u8;
            },
            PawnDoublePush(pawn, move_to) => {
                assert!(self.arr[move_to as usize] == 0x00, "Trying to move in busy place!");
                self.arr[pawn.position()] = 0x00;
                self.arr[move_to as usize] = pawn.code | PieceFlag::Moved as u8;
            },
            PromotionQuiet(pawn, move_to, new_type) => {
                assert!(pawn.type_() == PieceType::Pawn, "Trying to promote non-pawn piece!");
                assert!(self.arr[move_to as usize] == 0x00, "Trying to move in busy place!");
                self.arr[pawn.position()] = 0x00;
                self.arr[move_to as usize] = pawn.color() as u8 | new_type as u8 | PieceFlag::Moved as u8;
            },
            PromotionCapture(pawn, target, new_type) => {
                assert!(pawn.type_() == PieceType::Pawn, "Trying to promote non-pawn piece!");
                assert!(pawn.color() != target.color(), "That's a bug! Pawn captured teammate!");
                self.arr[pawn.position()] = 0x00;
                self.arr[target.position()] = pawn.color() as u8 | new_type as u8 | PieceFlag::Moved as u8;
            },
            Castling(king, castling_side) => {
                // Checks for king
                assert!(king.type_() == PieceType::King, "Trying to castle without king?!");
                assert!(king.code & PieceFlag::Moved as u8 == 0, "King have already moved!");
                assert!(king.code & PieceFlag::CanCastle as u8 != 0, "King can't castle!");
                // TODO: check if king crosses square under attack (castle rights bits)
                let rook_pos = king.position & 0xf0 | castling_side.clone() as u8;
                let rook_code = self.arr[rook_pos as usize];
                // Checks for rook
                assert!(PieceType::from(rook_code) == PieceType::Rook, "Non-rook piece used for castling!");
                assert!(Color::from(rook_code) == king.color(), "Enemy rook used for castling!");
                assert!(rook_code & PieceFlag::Moved as u8 == 0, "Moved rook used for castling!");
                #[cfg(debug_assertions)]
                for cell in between(rook_pos, king.position) {
                    assert!(self.arr[cell as usize] == 0x00, "There is something in way of castling!");
                }
                self.arr[king.position()] = 0x00;
                self.arr[rook_pos as usize] = 0x00;
                let rank = king.position & 0xf0;
                match castling_side {
                    CastlingSide::KingSide => {
                        self.arr[(rank | 0x06) as usize] = king.code | PieceFlag::Moved as u8;
                        self.arr[(rank | 0x05) as usize] = rook_code | PieceFlag::Moved as u8;
                    },
                    CastlingSide::QueenSide => {
                        self.arr[(rank | 0x02) as usize] = king.code | PieceFlag::Moved as u8;
                        self.arr[(rank | 0x03) as usize] = rook_code | PieceFlag::Moved as u8;
                    },
                }
            },
            EnPassantCapture(pawn, target) => {
                assert!(pawn.type_() == PieceType::Pawn, "Trying to use EnPassant by non-pawn piece!");
                assert!(target.type_() == PieceType::Pawn, "Trying to capture non-pawn piece with EnPassant!");
                assert!(pawn.color() != target.color(), "That's a bug! Pawn captured teammate!");
                let step: u8 = match pawn.color() {
                    Color::Black => 0x10,
                    Color::White => 0xf0,
                };
                assert!(self.arr[(target.position + step) as usize] == 0x00, "Something is in way of EnPassant!");
                self.arr[pawn.position()] = 0x00;
                self.arr[target.position()] = 0x00;
                self.arr[(target.position + step) as usize] = pawn.code;
            },
        }
    }

    /** Undo valid move. */
    fn undo(&mut self, _move: Move) {
        use Move::*;
        match _move {
            NullMove => trace!("Undo NullMove! Is it intentional?"),
            QuietMove(piece, moved_to) => {
                self.arr[moved_to as usize] = 0x00;
                self.arr[piece.position()] = piece.code;
            },
            Capture(piece, target) => {
                self.arr[target.position()] = target.code;
                self.arr[piece.position()] = piece.code;
            },
            PawnDoublePush(pawn, moved_to) => {
                self.arr[moved_to as usize] = 0x00;
                self.arr[pawn.position()] = pawn.code;
            },
            PromotionQuiet(pawn, moved_to, _) => {
                self.arr[moved_to as usize] = 0x00;
                self.arr[pawn.position()] = pawn.code;
            },
            PromotionCapture(pawn, target, _) => {
                self.arr[target.position()] = target.code;
                self.arr[pawn.position()] = pawn.code;
            },
            Castling(king, castling_side) => {
                let rook_pos = king.position & 0xf0 | castling_side.clone() as u8;
                let rank = king.position & 0xf0;
                let rook_code = match castling_side {
                    CastlingSide::KingSide => {
                        self.arr[(rank | 0x06) as usize] = 0x00;
                        let rook = self.arr[(rank | 0x05) as usize];
                        self.arr[(rank | 0x05) as usize] = 0x00;
                        rook
                    },
                    CastlingSide::QueenSide => {
                        self.arr[(rank | 0x02) as usize] = 0x00;
                        let rook = self.arr[(rank | 0x03) as usize];
                        self.arr[(rank | 0x03) as usize] = 0x00;
                        rook
                    },
                };
                self.arr[king.position()] = king.code;
                self.arr[rook_pos as usize] = rook_code ^ PieceFlag::Moved as u8;
            },
            EnPassantCapture(pawn, target) => {
                let step: u8 = match pawn.color() {
                    Color::Black => 0x10,
                    Color::White => 0xf0,
                };
                self.arr[(target.position + step) as usize] = 0x00;
                self.arr[pawn.position()] = pawn.code;
                self.arr[target.position()] = target.code;
            },
        }
    }

    fn who_can_attack(&self, piece: Piece) -> Option<Vec<Piece>> {
        let color = piece.color();
        let mut attackers = Vec::with_capacity(8);
        for file in 0..8u8 {
            for rank in 0..8u8 {
                let pos = rank << 4 & file;
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
                let pos = rank << 4 & file;
                let piece = Piece::from_code(self.arr[pos as usize], pos);
                if piece.code != 0x00 && piece.color() != color && piece.can_attack(position) {
                    return true
                }
            }
        }
        false
    }
}

impl Default for Board {
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

/** Bits structure of piece code
 * Bit 7 -- Color of the piece
 * - 1 -- Black 
 * - 0 -- White 
 * Bit 6 -- Not used 
 * Bit 5 -- Not used 
 * Bit 4 -- Castle flag for Kings only
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
struct Piece {
    code: u8,
    position: u8,
}

enum PieceFlag {
    /** Bit 3 -- Piece has moved flag */
    Moved = 0x08,
    /** Bit 4 -- Castle flag for Kings only */
    CanCastle = 0x10,
}

impl Piece {
    fn new(piece_type: PieceType, color: Color, position: u8) -> Piece {
        Piece { code: piece_type as u8 | color as u8, position }
    }

    fn from_code(code: u8, position: u8) -> Piece {
        Piece { code, position }
    }

    fn color(&self) -> Color {
        Color::from_byte(self.code)
    }

    fn type_(&self) -> PieceType {
        PieceType::from_byte(self.code)
    }

    fn position(&self) -> usize {
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
            },
            PieceType::Bishop => {
                is_in_diagonal_line(self.position, target) &&
                    between(self.position, target)
                        .all(|cell| cell == 0x00)
            },
            PieceType::Rook => {
                is_in_straight_line(self.position, target) &&
                    between(self.position, target)
                        .all(|cell| cell == 0x00)
            },
            PieceType::Queen => {
                (is_in_straight_line(self.position, target)
                || is_in_diagonal_line(self.position, target)) &&
                    between(self.position, target)
                        .all(|cell| cell == 0x00)
            },
            PieceType::Knight => {
                let diff = (self.position & 0x0f).abs_diff(target & 0x0f);
                distance(self.position, target) == 3 && diff != 0 && diff != 3
            },
            PieceType::King => distance(self.position, target) == 1,
            PieceType::Invalid => panic!("Invalid square is trying to attack?!"),
            PieceType::EmptySquare => panic!("Empty square is trying to attack?!"),
        }
    }
}

#[derive(PartialEq)]
enum Color {
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

#[derive(PartialEq)]
enum PieceType {
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
    BetweenIterator { current: from, target: to, step}
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