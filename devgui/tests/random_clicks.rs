//! Whole games with the window's part played by seeded random clicks. At each
//! prompt the test clicks anything the window would let a person click, until
//! an answer completes: a half chosen and abandoned, a reset in the middle of
//! an ordering, a bucket filled and emptied, the clicks no fixed rule makes.
//! Every game must finish, the engine thread must not panic, and the engine
//! must accept every answer the view model builds.

#[path = "support/games.rs"]
mod games;

use std::sync::Arc;

use devgui::bridge::{GameSetup, Outcome, Pool, ToWindow, spawn_game};
use devgui::prompt::Primitive;
use devgui::view_model::{Amount, BoardView, DoneButton, Input, Item, NumberField, WindowState};
use games::{dealt, from_board, next};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const SEEDS: [u64; 3] = [1, 2, 3];

/// A game this long has stopped being a game; fail rather than hang.
const PROMPT_CAP: usize = 20_000;

/// Clicks at one prompt before the window is judged to offer no way to answer
/// it. An ordering of ten, with a reset one click in twenty, finishes in a few
/// hundred.
const CLICK_CAP: usize = 5_000;

/// How often a click is "Start over", where the window offers it: as often as
/// a button among twenty would be, so an ordering still finishes.
const RESET_ONE_IN: u32 = 20;

#[test]
fn random_clicks_finish_dealt_games_from_both_pools() {
    let mut seen = Vec::new();
    for pool in [Pool::Performance, Pool::Stress] {
        for seed in SEEDS {
            seen.extend(play_at_random(GameSetup { pool, ..dealt(seed, None) }, seed));
        }
    }
    let reached = |primitive: fn(&Primitive) -> bool| seen.iter().any(primitive);
    assert!(reached(|p| matches!(p, Primitive::PickN { max: 1, .. })), "no single pick reached");
    assert!(reached(|p| matches!(p, Primitive::PickN { max, .. } if *max > 1)), "no multiple pick reached");
}

/// The review boards reach what a dealt game reaches only now and then: a
/// blocker with two attackers to choose between, and damage to divide.
#[test]
fn random_clicks_finish_games_from_the_review_boards() {
    let mut seen = Vec::new();
    for board in ["main.scenario", "blocks.scenario", "damage.scenario"] {
        for seed in SEEDS {
            seen.extend(play_at_random(from_board(board, None), seed));
        }
    }
    assert!(seen.iter().any(|p| matches!(p, Primitive::Allocate { .. })), "no allocation reached");
}

/// Plays a whole game by random clicks; the primitive of every prompt asked.
fn play_at_random(setup: GameSetup, seed: u64) -> Vec<Primitive> {
    let game = match &setup.scenario {
        Some(path) => format!("{} at seed {seed}", path.display()),
        None => format!("{:?} pool, seed {}", setup.pool, setup.seed),
    };
    let engine = spawn_game(setup, Arc::new(|| {}));
    let mut rng = StdRng::seed_from_u64(seed);
    let mut state = WindowState::default();
    let mut asked = Vec::new();
    let mut last = String::from("no answer yet");
    for _ in 0..PROMPT_CAP {
        let message = next(&engine);
        match &message {
            ToWindow::Finished { outcome, .. } => {
                assert!(matches!(outcome, Outcome::Won(_) | Outcome::Draw), "{game}: {outcome:?}");
                return asked;
            }
            ToWindow::Panicked { message } => panic!("{game}: the engine thread panicked after {last}: {message}"),
            ToWindow::Refused { message } => panic!("{game}: the scenario did not load: {message}"),
            ToWindow::Prompt { prompt, .. } => asked.push(prompt.primitive.clone()),
        }
        state.receive(message);
        let prompt = state.prompt.clone();
        let answer = click_until_answered(&mut state, &mut rng)
            .unwrap_or_else(|| panic!("{game}: no answer in {CLICK_CAP} clicks to {prompt:?}"));
        last = format!("answering {prompt:?} with {answer:?}");
        engine.answers.send(answer).expect("the engine hung up with a prompt open");
    }
    panic!("{game}: no result after {PROMPT_CAP} prompts");
}

fn click_until_answered(state: &mut WindowState, rng: &mut StdRng) -> Option<devgui::prompt::Answer> {
    for _ in 0..CLICK_CAP {
        let can_reset = state.prompt_view().is_some_and(|prompt| prompt.can_reset);
        let input = if can_reset && rng.random_ratio(1, RESET_ONE_IN) {
            Input::Reset
        } else {
            let offered = clickable(state, rng);
            assert!(!offered.is_empty(), "the window offers nothing to click at {:?}", state.prompt);
            offered[rng.random_range(0..offered.len())]
        };
        if let Some(answer) = state.input(input) {
            return Some(answer);
        }
    }
    None
}

/// What `app::draw` lets a person click now, besides "Start over": each
/// option's button or an allocation's live "−" and "+", the number's field,
/// the confirm button while it is live, and each board item marked clickable.
fn clickable(state: &WindowState, rng: &mut StdRng) -> Vec<Input> {
    let Some(prompt) = state.prompt_view() else {
        return Vec::new();
    };
    let mut inputs = Vec::new();
    for (i, option) in prompt.options.iter().enumerate() {
        match option.amount {
            Some(Amount { can_lower, can_raise, .. }) => {
                inputs.extend(can_lower.then_some(Input::OneFewer(i)));
                inputs.extend(can_raise.then_some(Input::OneMore(i)));
            }
            None => inputs.push(Input::OptionButton(i)),
        }
    }
    if let Some(NumberField { min, max, .. }) = prompt.number {
        inputs.push(Input::Number(rng.random_range(min..=max.min(min.saturating_add(20)))));
    }
    if let Some(DoneButton { live: true, .. }) = prompt.done {
        inputs.push(Input::Done);
    }
    if let Some(board) = state.board_view() {
        inputs.extend(items(&board).filter(|item| item.clickable).filter_map(|item| item.target).map(Input::Board));
    }
    inputs
}

fn items(board: &BoardView) -> impl Iterator<Item = &Item> {
    let seats = board.seats.iter().flat_map(|seat| std::iter::once(&seat.player).chain(seat.zones.iter().flat_map(|zone| &zone.items)));
    seats.chain(&board.stack).chain(&board.pending_triggers).chain(&board.exile).chain(&board.command)
}
