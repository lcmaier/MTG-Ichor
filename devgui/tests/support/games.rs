//! A game for a test to play, dealt or from a review board, and the engine's
//! next message, waited for.

use std::path::{Path, PathBuf};
use std::time::Duration;

use devgui::bridge::{EngineHandle, GameSetup, Pool, ToWindow};

/// A dealt game at `seed` from the performance pool.
pub fn dealt(seed: u64, log_path: Option<PathBuf>) -> GameSetup {
    GameSetup { seed, pool: Pool::Performance, log_path, scenario: None }
}

/// A game from the review board `name` under `tests/scenarios/`.
pub fn from_board(name: &str, log_path: Option<PathBuf>) -> GameSetup {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("scenarios").join(name);
    GameSetup { scenario: Some(path), ..dealt(0, log_path) }
}

pub fn next(engine: &EngineHandle) -> ToWindow {
    engine.from_engine.recv_timeout(Duration::from_secs(120)).expect("the engine went quiet for two minutes")
}
