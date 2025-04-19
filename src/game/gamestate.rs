// src/game/gamestate.rs
mod core;
mod combat;
mod special_actions;
mod players;
mod event_handlers;
mod effects;
mod casting;

// Re-export the Game struct and its implementations
pub use core::Game;