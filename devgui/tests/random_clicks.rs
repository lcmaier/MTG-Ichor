//! Whole games with the window's part played by seeded random clicks. At each
//! prompt the test clicks anything the window would let a person click, until
//! an answer completes: a half chosen and abandoned, a reset in the middle of
//! an ordering, a bucket filled and emptied, a yield set or stopped, the
//! clicks no fixed rule makes, a shortcut key as often as its button; and
//! between prompts it turns full control on or off now and then, as the
//! header's switch does. The window's clock runs too: a click in the beat
//! after each prompt arrives must do nothing, and nothing may show as live.
//! Every game must finish, the engine thread must not panic, the engine must
//! accept every reply the view model builds, and every click the window offers
//! must move the answer along or act on the seat.

#[path = "support/games.rs"]
mod games;

use std::sync::Arc;

use devgui::bridge::{EngineHandle, GameSetup, Outcome, Pool, ToWindow, spawn_game};
use devgui::prompt::{Primitive, Reply};
use devgui::view_model::{
    Amount, BoardView, DoneButton, Input, Item, Key, NumberField, SETTLE_SECONDS, SeatButton, WindowState,
};
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

/// How often full control is switched between two prompts.
const FULL_CONTROL_ONE_IN: u32 = 25;

/// What the games reached, beside the primitives.
#[derive(Default)]
struct Reached {
    primitives: Vec<Primitive>,
    /// Priority prompts with `Pass` alone, which only full control asks.
    pass_only: usize,
    yields: usize,
    stops: usize,
}

#[test]
fn random_clicks_finish_dealt_games_from_both_pools() {
    let mut reached = Reached::default();
    for pool in [Pool::Performance, Pool::Stress] {
        for seed in SEEDS {
            play_at_random(GameSetup { pool, ..dealt(seed, None) }, seed, &mut reached);
        }
    }
    let primitive = |test: fn(&Primitive) -> bool| reached.primitives.iter().any(test);
    assert!(primitive(|p| matches!(p, Primitive::PickN { max: 1, .. })), "no single pick reached");
    assert!(primitive(|p| matches!(p, Primitive::PickN { max, .. } if *max > 1)), "no multiple pick reached");
    assert!(reached.pass_only > 0, "full control asked nothing a seat can only pass at");
    assert!(reached.yields > 0 && reached.stops > 0, "yields set {}, stopped {}", reached.yields, reached.stops);
}

/// The review boards reach what a dealt game reaches only now and then: a
/// blocker with two attackers to choose between, damage to divide, and
/// blocks declared illegally and asked for again.
#[test]
fn random_clicks_finish_games_from_the_review_boards() {
    let mut reached = Reached::default();
    for board in ["main.scenario", "blocks.scenario", "damage.scenario", "reask.scenario"] {
        for seed in SEEDS {
            play_at_random(from_board(board, None), seed, &mut reached);
        }
    }
    assert!(reached.primitives.iter().any(|p| matches!(p, Primitive::Allocate { .. })), "no allocation reached");
}

/// Plays a whole game by random clicks, adding what it reached.
fn play_at_random(setup: GameSetup, seed: u64, reached: &mut Reached) {
    let game = match &setup.scenario {
        Some(path) => format!("{} at seed {seed}", path.display()),
        None => format!("{:?} pool, seed {}", setup.pool, setup.seed),
    };
    let engine = spawn_game(setup, Arc::new(|| {}));
    let mut rng = StdRng::seed_from_u64(seed);
    let mut state = WindowState::default();
    let mut clock = 0.0;
    let mut last = String::from("no answer yet");
    for _ in 0..PROMPT_CAP {
        if rng.random_ratio(1, FULL_CONTROL_ONE_IN) {
            // What `Session::input` does with the header's switch.
            let on = !state.full_control;
            engine.full_control.set(on);
            state.input(Input::FullControl(on));
        }
        let message = next(&engine);
        match &message {
            ToWindow::Finished { outcome, .. } => {
                assert!(matches!(outcome, Outcome::Won(_) | Outcome::Draw), "{game}: {outcome:?}");
                return;
            }
            ToWindow::Panicked { message } => panic!("{game}: the engine thread panicked after {last}: {message}"),
            ToWindow::Refused { message } => panic!("{game}: the scenario did not load: {message}"),
            ToWindow::Prompt { prompt, .. } => {
                reached.primitives.push(prompt.primitive.clone());
                reached.pass_only += usize::from(prompt.pass.is_some() && prompt.options.len() == 1);
            }
        }
        clock += 1.0;
        state.tick(clock);
        state.receive(message);
        // Item 201: the beat after a prompt arrives offers nothing and drops a click.
        assert!(clickable(&state, &mut rng).is_empty(), "{game}: live in the beat at {:?}", state.prompt);
        assert_eq!(state.clone().input(Input::OptionButton(0)), None, "{game}: a click in the beat");
        state.tick(clock + 2.0 * SETTLE_SECONDS);
        let prompt = state.prompt.clone();
        let reply = click_until_answered(&mut state, &engine, &mut rng, reached)
            .unwrap_or_else(|| panic!("{game}: no answer in {CLICK_CAP} clicks to {prompt:?}"));
        last = format!("answering {prompt:?} with {reply:?}");
        reached.yields += usize::from(matches!(reply, Reply::Yield(_)));
        engine.answers.send(reply).expect("the engine hung up with a prompt open");
    }
    panic!("{game}: no result after {PROMPT_CAP} prompts");
}

/// Clicks until a reply closes the prompt. "Stop yielding" is sent as it is
/// clicked, as `Session::input` sends it, and the prompt stays open.
fn click_until_answered(state: &mut WindowState, engine: &EngineHandle, rng: &mut StdRng, reached: &mut Reached) -> Option<Reply> {
    for _ in 0..CLICK_CAP {
        let can_reset = state.prompt_view().is_some_and(|prompt| prompt.can_reset);
        let input = if can_reset && rng.random_ratio(1, RESET_ONE_IN) {
            Input::Reset
        } else {
            let offered = clickable(state, rng);
            assert!(!offered.is_empty(), "the window offers nothing to click at {:?}", state.prompt);
            offered[rng.random_range(0..offered.len())]
        };
        let before = (state.selection.clone(), state.yielding);
        match state.input(input) {
            Some(Reply::StopYielding) => {
                reached.stops += 1;
                engine.answers.send(Reply::StopYielding).expect("the engine hung up with a prompt open");
            }
            Some(reply) => return Some(reply),
            None => {
                let typed = matches!(input, Input::Number(_));
                let after = (state.selection.clone(), state.yielding);
                assert!(typed || after != before, "the window offered {input:?}, which did nothing, at {:?}", state.prompt);
            }
        }
    }
    None
}

/// What `app::draw` lets a person click now, besides "Start over": each
/// option's button or an allocation's live "−" and "+", the number's field,
/// the confirm button while it is live, each live yield and "Stop yielding",
/// each board item marked clickable, and the key for each live button that
/// has one.
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
            None => inputs.extend(option.live.then_some(Input::OptionButton(i))),
        }
    }
    if let Some(NumberField { min, max, .. }) = prompt.number {
        inputs.push(Input::Number(rng.random_range(min..=max.min(min.saturating_add(20)))));
    }
    if let Some(DoneButton { live: true, .. }) = prompt.done {
        inputs.push(Input::Done);
    }
    let seat_buttons = prompt.yields.iter().chain(prompt.yielding.as_ref().map(|(_, stop)| stop));
    inputs.extend(seat_buttons.filter(|button| button.live).map(|SeatButton { input, .. }| *input));
    let key = |key| Input::Key { key, repeat: false };
    let mut keyed: Vec<Input> = inputs
        .iter()
        .filter_map(|input| match *input {
            Input::OptionButton(i) if i < 9 => Some(key(Key::Digit(i as u8 + 1))),
            Input::Done => Some(key(Key::Enter)),
            _ => None,
        })
        .collect();
    if let Some(pass) = state.prompt.as_ref().and_then(|prompt| prompt.pass)
        && inputs.contains(&Input::OptionButton(pass))
    {
        keyed.push(key(Key::Space));
    }
    inputs.extend(keyed);
    if let Some(board) = state.board_view() {
        inputs.extend(items(&board).filter(|item| item.clickable).filter_map(|item| item.target).map(Input::Board));
    }
    inputs
}

fn items(board: &BoardView) -> impl Iterator<Item = &Item> {
    let seats = board.seats.iter().flat_map(|seat| std::iter::once(&seat.player).chain(seat.zones.iter().flat_map(|zone| &zone.items)));
    seats.chain(&board.stack).chain(&board.pending_triggers).chain(&board.exile).chain(&board.command)
}
