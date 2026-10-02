//! One game the window plays, and what Reload and "Save board as scenario"
//! do to it. Plain Rust, so a test drives it with no window; `app` draws
//! over it and forwards each click here.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::bridge::{EngineHandle, GameSetup, spawn_game};
use crate::view_model::{Input, WindowState};

pub struct Session {
    /// The game as launched; each start's decision log is its own file
    /// beside `log_path`'s.
    pub setup: GameSetup,
    /// This game's decision log.
    pub log_path: Option<PathBuf>,
    /// What the window shows: the engine's last messages and the answer in
    /// progress.
    pub state: WindowState,
    /// What the last "Save board as scenario" did, for the header: where it
    /// saved, or why it could not.
    pub saved: Option<Result<String, String>>,
    engine: EngineHandle,
    wake: Arc<dyn Fn() + Send + Sync>,
}

impl Session {
    pub fn start(setup: GameSetup, wake: Arc<dyn Fn() + Send + Sync>) -> Session {
        let (engine, log_path) = start_game(&setup, &wake);
        Session { setup, log_path, state: WindowState::default(), saved: None, engine, wake }
    }

    /// The window's clock, which settles each new prompt (`WindowState::settling_for`).
    pub fn tick(&mut self, now: f64) {
        self.state.tick(now);
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
                (self.engine, self.log_path) = start_game(&self.setup, &self.wake);
                let full_control = self.state.full_control;
                self.engine.full_control.set(full_control);
                self.state = WindowState { full_control, now: self.state.now, ..WindowState::default() };
                self.saved = None;
            }
            // From the window's thread, at any moment: the seat reads the
            // switch at its next prompt, and the log never records it.
            Input::FullControl(on) => {
                self.engine.full_control.set(on);
                self.state.input(input);
            }
            Input::SaveBoard => {
                let Some(board) = &self.state.board else { return };
                let path = saved_board_path(self.log_path.as_deref(), board.turn);
                self.saved = Some(match std::fs::write(&path, &board.board_text) {
                    Ok(()) => Ok(format!("saved {}", path.display())),
                    Err(e) => Err(format!("cannot save {}: {e}", path.display())),
                });
            }
            input => {
                if let Some(reply) = self.state.input(input) {
                    // Fails only once the engine thread has ended, and its last message said why.
                    let _ = self.engine.answers.send(reply);
                }
            }
        }
    }
}

/// A game on its own thread, with a decision log no earlier game wrote.
fn start_game(setup: &GameSetup, wake: &Arc<dyn Fn() + Send + Sync>) -> (EngineHandle, Option<PathBuf>) {
    let log_path = setup.log_path.as_deref().map(own_log);
    let engine = spawn_game(GameSetup { log_path: log_path.clone(), ..setup.clone() }, Arc::clone(wake));
    (engine, log_path)
}

/// `path`, or the first of `stem-2.log`, `stem-3.log`, … beside it that does
/// not exist: Reload keeps the record of the game it replaces, and a thread
/// still finishing that game writes only its own file.
fn own_log(path: &Path) -> PathBuf {
    let stem = path.file_stem().map_or("game".to_string(), |s| s.to_string_lossy().into_owned());
    let extension = path.extension().map_or(String::new(), |e| format!(".{}", e.to_string_lossy()));
    first_unused(|n| match n {
        1 => path.to_path_buf(),
        n => path.with_file_name(format!("{stem}-{n}{extension}")),
    })
}

/// Where "Save board as scenario" writes: beside the decision log, named for
/// the turn, and never over an earlier save.
pub fn saved_board_path(log_path: Option<&Path>, turn: u32) -> PathBuf {
    let log = log_path.map_or_else(|| PathBuf::from("logs").join("board.log"), Path::to_path_buf);
    let stem = log.file_stem().map_or("board".to_string(), |s| s.to_string_lossy().into_owned());
    first_unused(|n| match n {
        1 => log.with_file_name(format!("{stem}-turn-{turn}.scenario")),
        n => log.with_file_name(format!("{stem}-turn-{turn}-{n}.scenario")),
    })
}

/// The first of `named(1)`, `named(2)`, … that does not exist yet.
fn first_unused(named: impl Fn(u32) -> PathBuf) -> PathBuf {
    (1..).map(&named).find(|path| !path.exists()).unwrap_or_else(|| named(1))
}
