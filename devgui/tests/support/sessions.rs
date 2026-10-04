//! A session for a test to drive with no window, writing its boards and
//! records under a fresh temporary folder, and its next question, waited for.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use devgui::boards::Folders;
use devgui::bridge::GameSetup;
use devgui::launch::Start;
use devgui::session::Session;

use crate::games::dealt;

/// Seat 0 holds a spell it can cast, so its first prompt is turn 1's.
pub const BOLT_IN_HAND: &str = "hand 0: Lightning Bolt
library 0: Mountain | x5
library 1: Mountain | x5
                            battlefield: Mountain | controller 0
player 0: life 13
";

/// A session writing under a fresh temporary folder named `name`, and a
/// file there holding `text` as `<name>.scenario`.
pub fn session_with(name: &str, text: &str, start: impl FnOnce(&Path) -> Start) -> (Session, Folders, PathBuf) {
    let root = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let file = root.join(format!("{name}.scenario"));
    std::fs::write(&file, text).unwrap();
    let folders = Folders { logs: root.join("logs"), boards: root.join("boards"), ..Folders::default() };
    (Session::start(start(&file), folders.clone(), Arc::new(|| {})), folders, file)
}

pub fn scenario_game(file: &Path) -> Start {
    Start::Game(GameSetup { scenario: Some(file.to_path_buf()), ..dealt(0) })
}

/// The session's next question, waited for: the engine thread sends it. A
/// game that ends first, or does not start, fails the test saying which.
pub fn next_prompt(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(60);
    while session.state.prompt.is_none() {
        let state = &session.state;
        assert!(state.outcome.is_none() && state.panic.is_none() && state.refused.is_none(), "{}", state.status());
        assert!(Instant::now() < deadline, "no prompt in a minute");
        std::thread::sleep(Duration::from_millis(5));
        session.receive();
    }
}
