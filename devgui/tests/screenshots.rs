//! The window drawn offscreen at the boards the review scenarios under
//! `tests/scenarios/` reach, as pictures to review: each board is set up for
//! its prompts, so no seed is hunted for them. `UPDATE_SNAPSHOTS=true cargo
//! test --test screenshots` redraws them. A plain run compares against the
//! committed PNGs, which holds on the machine that drew them: another GPU
//! draws a few pixels differently.

#[path = "support/games.rs"]
mod games;
#[path = "support/window_by_rule.rs"]
mod window_by_rule;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use devgui::app::{SessionHeader, draw};
use devgui::bridge::{GameSetup, ToWindow, spawn_game};
use devgui::prompt::{Answer, Reply};
use devgui::view_model::{Input, WindowState};
use egui_kittest::{Harness, SnapshotResult, SnapshotResults};
use games::{from_board, next};

/// Each picture: the first prompt of a kind any review board reaches, with
/// any clicks made before it is drawn. Kept to what a review needs, since
/// each redraw commits every PNG again.
const PICTURES: [(&str, &str); 6] = [
    ("PriorityAction", "priority"),
    ("ManaAbilityWindow", "mana_window"),
    ("SelectRecipients", "targets"),
    ("DeclareAttackers", "attackers"),
    ("DeclareBlockers", "blockers"),
    ("AssignCombatDamage", "combat_damage"),
];

/// The review boards, in the order their prompts are taken.
const BOARDS: [&str; 3] = ["main.scenario", "blocks.scenario", "damage.scenario"];

/// The header's line and decision log for a review board.
fn header(board: &str) -> (String, PathBuf) {
    let stem = board.trim_end_matches(".scenario");
    (format!("scenario tests/scenarios/{board} · seed 0"), PathBuf::from(format!("logs/{stem}-seed-0.log")))
}

#[test]
fn the_window_at_each_kind_of_prompt_the_review_boards_reach() {
    let mut first: BTreeMap<&str, (WindowState, &str)> = BTreeMap::new();
    for board in BOARDS {
        window_by_rule::play_by_rule(from_board(board, None), |state| {
            let Some(prompt) = &state.prompt else { return };
            let Some((_, name)) = PICTURES.iter().find(|(kind, _)| *kind == prompt.kind) else { return };
            // A priority prompt with a spell or two in reach.
            let worth_it = prompt.kind != "PriorityAction" || prompt.options.len() >= 3;
            if worth_it && !first.contains_key(name) {
                first.insert(name, (clicked_once(state), board));
            }
        });
    }
    let mut results = SnapshotResults::new();
    for (name, (state, board)) in &first {
        results.add(picture(state, &header(board), None, name));
    }
    results.add(picture(&panicked(), &header("main.scenario"), None, "engine_panic"));
    results.add(picture(&refused(), &header("refused.scenario"), None, "scenario_refused"));
    let sample = "../mtgsim/scenarios/bolt-into-giant-growth.scenario";
    let line = (format!("scenario {sample} · seed 0"), PathBuf::from("logs/bolt-into-giant-growth-seed-0.log"));
    results.add(picture(&setup_stack(sample), &line, None, "setup_stack"));
    results.add(picture(&blocks_rejected(), &header("reask.scenario"), None, "blocks_rejected"));
    let (state, board) = &first["priority"];
    results.add(picture(state, &header(board), Some(Ok("saved logs/main-seed-0-turn-3.scenario")), "board_saved"));
    let missing: Vec<&str> = PICTURES.iter().map(|(_, name)| *name).filter(|name| !first.contains_key(name)).collect();
    assert!(missing.is_empty(), "the review boards no longer reach {missing:?}");
}

/// The first click the rule would make, when it does not answer the prompt,
/// so a choice in progress shows.
fn clicked_once(state: &WindowState) -> WindowState {
    let mut after = state.clone();
    if let Some(input) = window_by_rule::inputs_by_rule(state).into_iter().find(|input| *input != Input::Done)
        && after.clone().input(input).is_none()
    {
        after.input(input);
    }
    after
}

/// What the window shows when the engine panics: an answer the validators refuse.
fn panicked() -> WindowState {
    let engine = spawn_game(from_board("main.scenario", None), Arc::new(|| {}));
    let mut state = WindowState::default();
    state.receive(next(&engine));
    engine.answers.send(Reply::Answer(Answer::Picks(vec![99]))).unwrap();
    state.receive(next(&engine));
    state
}

/// The window's first prompt on §5.3's sample: the stack its setup actions
/// built, player 0 holding priority over it with a Bolt left to cast.
fn setup_stack(sample: &str) -> WindowState {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let engine = spawn_game(GameSetup { scenario: Some(path), ..from_board("main.scenario", None) }, Arc::new(|| {}));
    let mut state = WindowState::default();
    state.receive(next(&engine));
    assert_eq!(state.board.as_ref().map(|board| board.stack.len()), Some(3), "the setup actions' stack");
    state
}

/// CR 509.1a's re-ask: the window declared its one Wall of Stone blocking
/// both Bears, and is asked again, told why.
fn blocks_rejected() -> WindowState {
    let engine = spawn_game(from_board("reask.scenario", None), Arc::new(|| {}));
    let mut state = WindowState::default();
    state.receive(next(&engine));
    state.input(Input::OptionButton(0));
    state.input(Input::OptionButton(1));
    engine.answers.send(state.input(Input::Done).expect("both blocks declared")).unwrap();
    state.receive(next(&engine));
    assert!(state.prompt.as_ref().is_some_and(|prompt| prompt.rejected.is_some()), "the re-ask says why");
    state
}

/// What the window shows when the file does not load.
fn refused() -> WindowState {
    let path = std::env::temp_dir().join("devgui-picture-refused.scenario");
    std::fs::write(&path, "turn 3\nstep precombat main\nbattlefield: Grizzly Bears | controller 0, attacking player 1\n").unwrap();
    let engine = spawn_game(GameSetup { scenario: Some(path), ..from_board("main.scenario", None) }, Arc::new(|| {}));
    let message = next(&engine);
    assert!(matches!(message, ToWindow::Refused { .. }), "{message:?}");
    let mut state = WindowState::default();
    state.receive(message);
    state
}

fn picture(state: &WindowState, (line, log): &(String, PathBuf), saved: Option<Result<&str, &str>>, name: &str) -> SnapshotResult {
    let header = SessionHeader { line, log: Some(log), reloadable: true, saved };
    let mut harness = Harness::builder().with_size([1280.0, 800.0]).build_ui(|ui| {
        draw(ui, state, &header);
    });
    harness.run();
    harness.try_snapshot(name)
}
