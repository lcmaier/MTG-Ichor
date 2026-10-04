//! A game for a test to play, dealt or from a review board, and the engine's
//! next message, waited for.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use devgui::bridge::{EngineHandle, GameSetup, Play, Pool, ToWindow, spawn_game};

/// A dealt game at `seed` from the performance pool.
pub fn dealt(seed: u64) -> GameSetup {
    GameSetup { seed: Some(seed), pool: Pool::Performance, players: 2, scenario: None }
}

/// A game from the review board `name` under `tests/scenarios/`.
pub fn from_board(name: &str) -> GameSetup {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("scenarios").join(name);
    GameSetup { scenario: Some(path), ..dealt(0) }
}

/// `setup`'s game, its start read now, logged nowhere.
pub fn play(setup: GameSetup) -> Play {
    Play::new(setup.start().unwrap_or_else(|refusal| panic!("{refusal}")))
}

/// `setup`'s game on a thread of its own.
pub fn spawn(setup: GameSetup) -> EngineHandle {
    spawn_game(play(setup), Arc::new(|| {}))
}

pub fn next(engine: &EngineHandle) -> ToWindow {
    engine.from_engine.recv_timeout(Duration::from_secs(120)).expect("the engine went quiet for two minutes")
}
