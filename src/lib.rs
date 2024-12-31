mod core;
pub mod online_game;
pub mod server;

// module re-exports
pub use core::*;

pub use core::definitions::{Cell, DefaultExternalMove, Figure, GameState, MatchInterface};
pub use core::engine::{Color, PieceType};
pub use core::game::{DarkGame, Game};

#[cfg(test)]
mod tests;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
