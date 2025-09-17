#![allow(dead_code)]
use crate::core::{Board, Color, Game, Move, Piece, PieceType};

// Only for server-side/offline match
pub trait Algorithm {
    fn solve(&self, game: &Game) -> Move;
}

pub type EvaluationFunc = dyn Fn(&Board) -> i32;

pub struct MinMaxBot {
    max_depth: u32,
    evaluate_fn: &'static EvaluationFunc,
}

impl MinMaxBot {
    fn new(max_depth: u32, evaluate_fn: &'static EvaluationFunc) -> Self {
        MinMaxBot {
            max_depth,
            evaluate_fn,
        }
    }
}

impl Algorithm for MinMaxBot {
    fn solve(&self, game: &Game) -> Move {
        todo!()
    }
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
    let ms = material_difference.abs().min(2400)
        + (material_difference.abs() * pawn_advantage * (8100 - material_total))
            / (6400 * (pawn_advantage + 1));
    let total_material_advantage = ms.min(3100);
    if material_difference >= 0 {
        total_material_advantage
    } else {
        -total_material_advantage
    }
}
