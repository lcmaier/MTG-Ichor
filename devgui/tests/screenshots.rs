//! The window drawn offscreen at the boards a seeded game reaches, as pictures
//! to review. `UPDATE_SNAPSHOTS=true cargo test --test screenshots` redraws
//! them. A plain run compares against the committed PNGs, which holds on the
//! machine that drew them: another GPU draws a few pixels differently.

#[path = "support/window_by_rule.rs"]
mod window_by_rule;

use std::collections::BTreeMap;
use std::sync::Arc;

use devgui::app::draw;
use devgui::bridge::{GameSetup, Pool, spawn_game};
use devgui::prompt::Answer;
use devgui::view_model::{Input, WindowState};
use egui_kittest::{Harness, SnapshotResult, SnapshotResults};
use window_by_rule::next;

/// The game whose prompts cover the most kinds in the first forty seeds.
const SEED: u64 = 33;
const SETUP_LINE: &str = "seed 33 · Performance pool · decision log logs/seed-33.log";

/// Each picture: the first prompt of a kind, with any clicks made before it is
/// drawn. Picks and an allocation; this pool asks for no number and no order.
/// Kept to what a review needs, since each redraw commits every PNG again.
const PICTURES: [(&str, &str); 6] = [
    ("PriorityAction", "priority"),
    ("ManaAbilityWindow", "mana_window"),
    ("SelectRecipients", "targets"),
    ("DeclareAttackers", "attackers"),
    ("DeclareBlockers", "blockers"),
    ("AssignCombatDamage", "combat_damage"),
];

#[test]
fn the_window_at_each_kind_of_prompt_the_seed_reaches() {
    let mut first: BTreeMap<&str, WindowState> = BTreeMap::new();
    window_by_rule::play_by_rule(SEED, None, |state| {
        let Some(prompt) = &state.prompt else { return };
        let Some((_, name)) = PICTURES.iter().find(|(kind, _)| *kind == prompt.kind) else { return };
        // A priority prompt with a spell or two in reach and a board built up.
        let worth_it = prompt.kind != "PriorityAction" || (prompt.options.len() >= 3 && state.board.as_ref().is_some_and(|b| b.turn >= 6));
        if worth_it && !first.contains_key(name) {
            first.insert(name, clicked_once(state));
        }
    });
    let mut results = SnapshotResults::new();
    for (name, state) in &first {
        results.add(picture(state, name));
    }
    results.add(picture(&panicked(), "engine_panic"));
    let missing: Vec<&str> = PICTURES.iter().map(|(_, name)| *name).filter(|name| !first.contains_key(name)).collect();
    assert!(missing.is_empty(), "seed {SEED} no longer reaches {missing:?}");
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
    let engine = spawn_game(GameSetup { seed: SEED, pool: Pool::Performance, log_path: None }, Arc::new(|| {}));
    let mut state = WindowState::default();
    state.receive(next(&engine));
    engine.answers.send(Answer::Picks(vec![99])).unwrap();
    state.receive(next(&engine));
    state
}

fn picture(state: &WindowState, name: &str) -> SnapshotResult {
    let mut harness = Harness::builder().with_size([1280.0, 800.0]).build_ui(|ui| {
        draw(ui, state, SETUP_LINE);
    });
    harness.run();
    harness.try_snapshot(name)
}
