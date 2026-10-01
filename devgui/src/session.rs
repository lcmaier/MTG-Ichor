//! One game the window plays, and what Reload and "Save board as scenario"
//! do to it. Plain Rust, so a test drives it with no window; `app` draws
//! over it and forwards each click here.

use std::path::PathBuf;
use std::sync::Arc;

use crate::bridge::{EngineHandle, GameSetup, spawn_game};
use crate::view_model::{Input, WindowState};

pub struct Session {
    pub setup: GameSetup,
    /// What the window shows: the engine's last messages and the answer in
    /// progress.
    pub state: WindowState,
    /// What the last "Save board as scenario" did, for the header.
    pub saved: Option<String>,
    engine: EngineHandle,
    wake: Arc<dyn Fn() + Send + Sync>,
}

impl Session {
    pub fn start(setup: GameSetup, wake: Arc<dyn Fn() + Send + Sync>) -> Session {
        let engine = spawn_game(setup.clone(), Arc::clone(&wake));
        Session { setup, state: WindowState::default(), saved: None, engine, wake }
    }

    /// Take every message the engine has sent since the last call.
    pub fn receive(&mut self) {
        while let Ok(message) = self.engine.from_engine.try_recv() {
            self.state.receive(message);
        }
    }

    /// Act on one input: a prompt's goes to the engine once it completes an
    /// answer, and the window's own controls act here.
    pub fn input(&mut self, input: Input) {
        match input {
            // The game again from its file, read again. The old engine thread
            // unwinds when its channel closes, as a closed window ends it.
            Input::Reload => {
                self.engine = spawn_game(self.setup.clone(), Arc::clone(&self.wake));
                self.state = WindowState::default();
                self.saved = None;
            }
            Input::SaveBoard => {
                let Some(board) = &self.state.board else { return };
                let path = saved_board_path(&self.setup, board.turn);
                self.saved = Some(match std::fs::write(&path, &board.board_text) {
                    Ok(()) => format!("saved {}", path.display()),
                    Err(e) => format!("cannot save {}: {e}", path.display()),
                });
            }
            input => {
                if let Some(answer) = self.state.input(input) {
                    // Fails only once the engine thread has ended, and its last message said why.
                    let _ = self.engine.answers.send(answer);
                }
            }
        }
    }
}

/// Where "Save board as scenario" writes: beside the decision log, named for
/// the turn, and never over an earlier save.
pub fn saved_board_path(setup: &GameSetup, turn: u32) -> PathBuf {
    let log = setup.log_path.clone().unwrap_or_else(|| PathBuf::from("logs").join("board.log"));
    let stem = log.file_stem().map_or("board".to_string(), |s| s.to_string_lossy().into_owned());
    let named = |n: u32| match n {
        1 => log.with_file_name(format!("{stem}-turn-{turn}.scenario")),
        n => log.with_file_name(format!("{stem}-turn-{turn}-{n}.scenario")),
    };
    (1..).map(named).find(|path| !path.exists()).unwrap_or_else(|| named(1))
}
