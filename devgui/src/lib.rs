//! The dev GUI (`roadmap-v2.md` row A6g): a window onto a game the engine plays
//! on a worker thread. The window plays every seat.
//!
//! Read in this order: `bridge`, the thread and the seats that ask the window;
//! `snapshot` and `prompt`, the owned data that crosses the channel;
//! `view_model`, what the window shows and what a click means; `session`, one
//! game and what Reload and Save do to it; `launch`, the command line read
//! into the game to start; then `app`, the egui drawing over them, which
//! decides nothing.

pub mod bridge;
pub mod snapshot;
pub mod prompt;
pub mod view_model;
pub mod session;
pub mod launch;
pub mod app;
