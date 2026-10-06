//! The dev GUI (`roadmap-v2.md` row A6g): a window onto a game the engine plays
//! on a worker thread. The window plays every seat.
//!
//! Read in this order: `bridge`, the thread and the seats that ask the window;
//! `snapshot` and `prompt`, the owned data that crosses the channel;
//! `view_model`, what the window shows and what a click means; `editor`, the
//! board editor beside the game, with `search`, its list of names; `boards`,
//! where a board and its games' records live; `save`, the journal of play
//! beside each log, which undo and savestates move through; `why_replay`,
//! the replay that answers a why about the past from its trace; `session`, one
//! game and one board, and what Play, Reload and Save do to them; `launch`,
//! the command line read into how the window starts; then `app`, the egui
//! drawing over them, which decides nothing.

pub mod bridge;
pub mod snapshot;
pub mod prompt;
pub mod view_model;
pub mod search;
pub mod editor;
pub mod boards;
pub mod save;
pub mod why_replay;
pub mod session;
pub mod launch;
pub mod app;
