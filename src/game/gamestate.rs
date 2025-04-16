// src/game/gamestate.rs
mod core;
mod combat;
mod special_actions;
mod effects;
mod players;

// Re-export the Game struct and its implementations
pub use core::Game;