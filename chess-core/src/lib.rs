pub mod core;
// pub mod online_game;
#[cfg(feature = "network")]
pub mod online_game;
pub mod utils;

// module re-exports
pub use core::engine::{Color, PieceType};

#[cfg(test)]
mod tests;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
