use std::cmp::min;

use crate::core::definitions::{Cell, Figure};
use crate::core::engine::{Board, Color, Piece, PieceFlag, PieceType};
use crate::core::utils::compact_pos;

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

#[allow(dead_code)]
fn material_advantage(board: &Board, player: Color) -> i32 {
    let mut material_difference: i32 = 0;
    let mut material_total = 0;
    let mut pawn_advantage = 0;
    for file in 0..8u8 {
        for rank in 0..8u8 {
            let pos = (file << 4) + rank;
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
