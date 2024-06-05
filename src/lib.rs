pub mod server;
pub mod online_game;
pub mod android_bridge;
mod core;

// module re-exports
pub use core::*;

pub use core::definitions::{Cell, DefaultMove, Figure, GameState, MatchInterface};
pub use core::engine::{Color, PieceType};
pub use core::game::{DarkGame, Game};

#[cfg(test)]
mod tests;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(target_os = "android")]
mod glue;
#[cfg(target_os = "android")]
pub use crate::glue::*;
#[cfg(target_os = "android")]
pub use crate::android_bridge::*;

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_ru_electro98_dark_1chess_MainActivity_00024Companion_initChessEngine(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Trace));

    log::info!("Chess engine library was successfully loaded!");
}
