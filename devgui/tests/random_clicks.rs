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
//!
//! The board editor's clicks are random too, with no game: every click it
//! offers must change the board or what is selected, the board must always
//! be its own text read back, and Undo must walk back to the board opened.

#[path = "support/games.rs"]
mod games;

use std::path::Path;

use devgui::bridge::{EngineHandle, GameSetup, Outcome, Pool, ToWindow};
use devgui::editor::{EditButton, Editor, EditorInput, EditorView, Source, Stepper, Typed};
use devgui::prompt::{Primitive, Reply};
use devgui::view_model::{
    Amount, BoardView, DoneButton, Input, Item, Key, NumberField, SETTLE_SECONDS, SeatButton, WindowState,
};
use games::{dealt, from_board, next, spawn};
use mtgsim::cards::registry::CardRegistry;
use mtgsim::scenario::Scenario;
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

/// How often a click is a yield, where the prompt offers one: as often as
/// "Start over". The window plays every seat, and a yield as likely as any
/// other click had both seats passing at most priority prompts, so a game
/// ran on to the end of the libraries: 112 turns and 40 s for one stress
/// game in debug.
const YIELD_ONE_IN: u32 = 20;

/// How often a click is in the why panel or asks a why on the board: as
/// often as a yield, so a game still plays on.
const WHY_ONE_IN: u32 = 20;

/// What the games reached, beside the primitives.
#[derive(Default)]
struct Reached {
    primitives: Vec<Primitive>,
    /// Priority prompts with `Pass` alone, which only full control asks.
    pass_only: usize,
    yields: usize,
    stops: usize,
    /// Whys asked and answered, and the panel closed.
    whys: usize,
    closed: usize,
}

#[test]
fn random_clicks_finish_dealt_games_from_both_pools() {
    let mut reached = Reached::default();
    for pool in [Pool::Performance, Pool::Stress] {
        for seed in SEEDS {
            play_at_random(GameSetup { pool, ..dealt(seed) }, seed, &mut reached);
        }
    }
    let primitive = |test: fn(&Primitive) -> bool| reached.primitives.iter().any(test);
    assert!(primitive(|p| matches!(p, Primitive::PickN { max: 1, .. })), "no single pick reached");
    assert!(primitive(|p| matches!(p, Primitive::PickN { max, .. } if *max > 1)), "no multiple pick reached");
    assert!(reached.pass_only > 0, "full control asked nothing a seat can only pass at");
    assert!(reached.yields > 0 && reached.stops > 0, "yields set {}, stopped {}", reached.yields, reached.stops);
    assert!(reached.whys > 0 && reached.closed > 0, "whys asked {}, the panel closed {}", reached.whys, reached.closed);
}

/// The review boards reach what a dealt game reaches only now and then: a
/// blocker with two attackers to choose between, damage to divide, and
/// blocks declared illegally and asked for again. The four-seat sample has
/// three seats the window plays and one that has left the game.
#[test]
fn random_clicks_finish_games_from_the_review_boards() {
    let mut reached = Reached::default();
    for board in ["main.scenario", "blocks.scenario", "damage.scenario", "reask.scenario"] {
        for seed in SEEDS {
            play_at_random(from_board(board), seed, &mut reached);
        }
    }
    let four_seats = Path::new(env!("CARGO_MANIFEST_DIR")).join("../mtgsim/scenarios/four-seats-commander.scenario");
    for seed in SEEDS {
        play_at_random(GameSetup { scenario: Some(four_seats.clone()), ..dealt(0) }, seed, &mut reached);
    }
    assert!(reached.primitives.iter().any(|p| matches!(p, Primitive::Allocate { .. })), "no allocation reached");
}

/// Plays a whole game by random clicks, adding what it reached.
fn play_at_random(setup: GameSetup, seed: u64, reached: &mut Reached) {
    let game = match &setup.scenario {
        Some(path) => format!("{} at seed {seed}", path.display()),
        None => format!("{:?} pool, seed {}", setup.pool, setup.seed.unwrap_or_default()),
    };
    let engine = spawn(setup);
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
            ToWindow::Refused { message, .. } => panic!("{game}: the scenario did not load: {message}"),
            ToWindow::Diverged { message, .. } => panic!("{game}: a game played from its start replayed nothing, yet {message}"),
            ToWindow::Why(why) => panic!("{game}: a why came with no question asked: {}", why.title),
            ToWindow::WhyFromTrace { .. } => panic!("{game}: a replay's why came over the game's own channel"),
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

/// Clicks until a reply closes the prompt. "Stop yielding" and a why are
/// sent as they are clicked, as `Session::input` sends them, and the prompt
/// stays open; the seat answers a why at once, and its answer must show.
fn click_until_answered(state: &mut WindowState, engine: &EngineHandle, rng: &mut StdRng, reached: &mut Reached) -> Option<Reply> {
    for _ in 0..CLICK_CAP {
        let can_reset = state.prompt_view().is_some_and(|prompt| prompt.can_reset);
        let yields = live_yields(state);
        let whys = why_clicks(state);
        let input = if can_reset && rng.random_ratio(1, RESET_ONE_IN) {
            Input::Reset
        } else if !yields.is_empty() && rng.random_ratio(1, YIELD_ONE_IN) {
            yields[rng.random_range(0..yields.len())].clone()
        } else if !whys.is_empty() && rng.random_ratio(1, WHY_ONE_IN) {
            whys[rng.random_range(0..whys.len())].clone()
        } else {
            let offered = clickable(state, rng);
            assert!(!offered.is_empty(), "the window offers nothing to click at {:?}", state.prompt);
            offered[rng.random_range(0..offered.len())].clone()
        };
        let before = (state.selection.clone(), state.yielding);
        match state.input(input.clone()) {
            Some(Reply::StopYielding) => {
                reached.stops += 1;
                engine.answers.send(Reply::StopYielding).expect("the engine hung up with a prompt open");
            }
            Some(Reply::Why(about)) => {
                engine.answers.send(Reply::Why(about)).expect("the engine hung up with a prompt open");
                let from_the_trace = state.prompt.as_ref().is_some_and(|prompt| prompt.why_reads_the_trace);
                match about {
                    // A replay answers it, which these clicks start none of:
                    // the seat stays quiet, and the panel says it waits.
                    Some(_) if from_the_trace => {
                        assert!(state.reading_the_trace.is_some(), "{input:?} at {:?} waits on no replay", state.prompt);
                        reached.whys += 1;
                    }
                    Some(about) => {
                        let answer = next(engine);
                        assert!(matches!(answer, ToWindow::Why(_)), "{input:?} at {:?} was answered with {answer:?}", state.prompt);
                        state.receive(answer);
                        let shown = state.why_view().unwrap_or_else(|| panic!("the why of {about:?} does not show"));
                        assert!(!shown.sections.is_empty(), "the why of {about:?} says nothing");
                        reached.whys += 1;
                    }
                    None => {
                        assert_eq!(state.why_view(), None, "the panel closed and still shows");
                        reached.closed += 1;
                    }
                }
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

/// What a person can click for a why now: a right-click on any object or
/// player on the board, and, with the panel open, each live link, Back while
/// it is live, and the panel's close.
fn why_clicks(state: &WindowState) -> Vec<Input> {
    let mut inputs: Vec<Input> = Vec::new();
    if let Some(board) = state.board_view() {
        inputs.extend(items(&board).filter_map(|item| item.target.map(Input::Why)));
    }
    if let Some(panel) = state.why_view() {
        let links = panel.sections.iter().flat_map(|section| &section.lines).flat_map(|line| &line.links);
        inputs.extend(links.filter(|link| link.live).map(|link| link.input.clone()));
        inputs.extend(panel.back.then_some(Input::WhyBack));
        inputs.push(Input::WhyClose);
    }
    inputs
}

/// The yields the prompt offers now, each one live.
fn live_yields(state: &WindowState) -> Vec<Input> {
    let Some(prompt) = state.prompt_view() else {
        return Vec::new();
    };
    prompt.yields.iter().filter(|button| button.live).map(|SeatButton { input, .. }| input.clone()).collect()
}

/// What `app::draw` lets a person click now, besides "Start over" and the
/// yields: each option's button or an allocation's live "−" and "+", the
/// number's field, the confirm button while it is live, "Stop yielding", each
/// board item marked clickable, and the key for each live button that has
/// one.
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
    let stop = prompt.yielding.as_ref().map(|(_, stop)| stop).filter(|button| button.live);
    inputs.extend(stop.map(|SeatButton { input, .. }| input.clone()));
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

/// Clicks on one board in the editor before Undo walks it back.
const EDITOR_CLICKS: usize = 200;

/// What a person types into the search, now and then.
const QUERIES: [&str; 6] = ["", "bear", "of", "forest", "thalia", "x"];

/// From an empty board and from the samples (a Commander table, a board
/// with every word, setup actions), anything the editor offers, clicked at
/// random (`setup-architecture.md` §8).
#[test]
fn random_clicks_build_boards_in_the_editor() {
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../mtgsim/scenarios");
    let mut boards = vec![String::new()];
    for name in ["template", "four-seats-commander", "holy-strength", "bolt-into-giant-growth"] {
        boards.push(std::fs::read_to_string(samples.join(format!("{name}.scenario"))).unwrap());
    }
    let mut reached = EditorReached::default();
    for (b, text) in boards.iter().enumerate() {
        for seed in SEEDS {
            edit_at_random(text, 10 * b as u64 + seed, &mut reached);
        }
    }
    let EditorReached { named, refused, accepted, undone } = reached;
    assert!(named > 0 && refused > 0 && accepted > 0 && undone > 0, "named {named}, refused {refused}, accepted {accepted}, undone {undone}");
}

/// What the editor's clicks reached: a card named by a click on it, boards
/// the loader refused and accepted, and Undo taken along the way.
#[derive(Default)]
struct EditorReached {
    named: usize,
    refused: usize,
    accepted: usize,
    undone: usize,
}

fn edit_at_random(text: &str, seed: u64, reached: &mut EditorReached) {
    let mut editor = Editor::open(text, Source::Empty, CardRegistry::default_registry()).unwrap();
    let opened = editor.board().clone();
    let mut rng = StdRng::seed_from_u64(seed);
    for click in 0..EDITOR_CLICKS {
        let groups = editor_clicks(&editor.view(), &mut rng);
        let group = &groups[rng.random_range(0..groups.len())];
        let input = group[rng.random_range(0..group.len())].clone();
        let state = |e: &Editor| (e.board().clone(), e.editing, e.picking, e.chosen, e.search().query().to_string());
        let before = state(&editor);
        reached.named += usize::from(editor.picking.is_some() && matches!(input, EditorInput::Card(_)));
        reached.undone += usize::from(input == EditorInput::Undo);
        editor.input(input.clone());
        assert_ne!(state(&editor), before, "seed {seed}, click {click}: {input:?} changed nothing in\n{}", editor.text());
        let read = Scenario::parse(&editor.board().to_string()).unwrap();
        assert_eq!(&read, editor.board(), "seed {seed}, click {click}: the board is not its text read back");
        reached.refused += usize::from(editor.refusal().is_some());
        reached.accepted += usize::from(editor.refusal().is_none());
    }
    while editor.view().undo.live {
        editor.input(EditorInput::Undo);
    }
    assert_eq!(editor.board(), &opened, "seed {seed}: Undo did not walk back to the board opened");
}

/// What `app::draw` lets a person click in the editor, besides Play and
/// Save, which are the session's, in groups so the long list of names does
/// not crowd out the board: the game's facts, the seats, the cards, the card
/// being edited, the lines shown as text, the search, and Undo.
fn editor_clicks(view: &EditorView, rng: &mut StdRng) -> Vec<Vec<EditorInput>> {
    let live = |buttons: &mut dyn Iterator<Item = &EditButton>| -> Vec<EditorInput> {
        buttons.filter(|button| button.live).map(|button| button.input.clone()).collect()
    };
    let mut facts = live(&mut view.active.iter().chain(&view.steps));
    facts.extend(view.facts.iter().flat_map(|stepper| stepped(stepper, rng)));
    facts.push(EditorInput::Seed(view.seed.wrapping_add(rng.random_range(1..100))));
    let mut seats = Vec::new();
    let mut cards = Vec::new();
    for seat in &view.seats {
        seats.extend(stepped(&seat.life, rng));
        seats.extend(seat.words.iter().map(|word| word.remove.clone()));
        for zone in &seat.zones {
            seats.extend(live(&mut std::iter::once(&zone.put).chain(&zone.shuffled)));
            cards.extend(zone.cards.iter().filter(|card| card.live).map(|card| card.input.clone()));
        }
    }
    let mut card = Vec::new();
    for row in view.card.iter().flat_map(|card| &card.rows) {
        card.extend(live(&mut row.buttons.iter()));
        card.extend(row.steppers.iter().flat_map(|stepper| stepped(stepper, rng)));
    }
    let texts = view.texts.iter().map(|text| text.remove.clone()).collect();
    let mut search = live(&mut view.search.results.iter());
    let queries: Vec<&str> = QUERIES.into_iter().filter(|query| *query != view.search.query).collect();
    search.push(EditorInput::Search(queries[rng.random_range(0..queries.len())].to_string()));
    let undo = live(&mut std::iter::once(&view.undo));
    [facts, seats, cards, card, texts, search, undo].into_iter().filter(|group| !group.is_empty()).collect()
}

/// A stepper's live "−" and "+", and a number typed near its own.
fn stepped(stepper: &Stepper, rng: &mut StdRng) -> Vec<EditorInput> {
    let mut inputs: Vec<EditorInput> = stepper.lower.iter().chain(&stepper.raise).cloned().collect();
    if let Some(Typed { field, value, min, max }) = stepper.typed {
        let near = value.clamp(min, max);
        let typed = rng.random_range(near.saturating_sub(20).max(min)..=near.saturating_add(20).min(max));
        if typed != value {
            inputs.push(EditorInput::Number(field, typed));
        }
    }
    inputs
}
