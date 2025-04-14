// src/game/gamestate.rs
mod core;
mod phases;
mod combat;
mod special_actions;
mod zones;

// Re-export the Game struct and its implementations
pub use core::Game;