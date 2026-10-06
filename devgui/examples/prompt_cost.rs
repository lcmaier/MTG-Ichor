//! What one prompt costs the window, read on a large board:
//!
//!     cargo run --release --example prompt_cost [-- BOARD.scenario]
//!
//! Four readings, each the median and the slowest tenth of many runs, beside
//! the allocations and bytes one run asks for:
//! - **snapshot**: `Snapshot::build`, the engine thread's work at each prompt
//!   before it sends, read with the layer memo warm (the engine has usually
//!   walked the board by the time it asks) and cold (the epoch just bumped),
//!   inside the seat's `audit_each_frame_once`; **of which the board text**,
//!   `Scenario::write`, which it builds for "Save board as scenario"; and in
//!   a debug build what the layer memo's audit costs it, read at every hit
//!   and paused;
//! - **receive**: `WindowState::receive`, the window's work as a prompt arrives;
//! - **views**: `board_view` and `prompt_view`, which `app::draw` builds again
//!   at every repaint, and the header's tools with three entries in their
//!   menu (`setup-architecture.md` §7.3);
//! - **the editor** on the same board: its view, built again at every repaint
//!   in the editor, with the advanced settings off and on; one edit at a
//!   click, which writes the board, reads it back and has the loader check
//!   it; and a line typed, added and undone (`setup-architecture.md` §7b);
//! - **the why panel**: the seat's answer at the priority question about the
//!   permanent the layers did most to and about a card in hand, each with the
//!   question's section, and the panel's view at every repaint
//!   (`setup-architecture.md` §7c).
//!
//! A number to read beside a change, not a gate (`engineering-practices.md`
//! §10.4): the time is the machine's, and the allocations are the code's.
//! The board defaults to `tests/scenarios/large.scenario`, and the prompt is
//! the one the engine asks first there.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use devgui::bridge::{GameSetup, Play, Pool, ToWindow, spawn_game};
use devgui::editor::{BoardNumber, Editor, EditorInput, Source};
use devgui::save::{Destination, Tools};
use devgui::snapshot::Snapshot;
use devgui::view_model::WindowState;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::layers::explain;
use mtgsim::scenario::Scenario;
use mtgsim::state::game_state::GameState;
use mtgsim::oracle::legality::candidate_priority_actions;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::why::{OpenQuestion, WhyAbout, why};

const RUNS: usize = 200;

fn main() {
    let board = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("scenarios").join("large.scenario")
    });
    let text = std::fs::read_to_string(&board).unwrap_or_else(|e| panic!("cannot read {}: {e}", board.display()));
    let scenario = Scenario::parse(&text).unwrap_or_else(|refusal| panic!("{refusal}"));
    let mut game = scenario.build(&CardRegistry::default_registry()).unwrap_or_else(|refusal| panic!("{refusal}")).game;
    game.state.record_events();

    let setup = GameSetup { seed: None, pool: Pool::Stress, players: scenario.players, scenario: Some(board.clone()) };
    let start = setup.start().unwrap_or_else(|refusal| panic!("{refusal}"));
    let engine = spawn_game(Play::new(start), Arc::new(|| {}));
    let message = engine.from_engine.recv_timeout(Duration::from_secs(60)).expect("the engine's first prompt");
    let ToWindow::Prompt { snapshot, prompt, .. } = message else {
        panic!("the board's first message is not a prompt: {message:?}");
    };
    let objects = snapshot.players.iter().map(|p| p.hand.len() + p.library.len() + p.graveyard.len() + p.battlefield.len()).sum::<usize>();
    println!("{}: {objects} objects, a {} prompt of {} options", board.display(), prompt.kind, prompt.options.len());
    let (prompt_kind, asked) = (prompt.kind.clone(), prompt.player);

    // As the seat builds it, every frame audited once in a debug build.
    let at_the_seat = |state: &GameState| state.audit_each_frame_once(|| Snapshot::build(state, 0));
    reading("snapshot, memo warm", || at_the_seat(&game.state));
    reading("snapshot, memo cold", || {
        game.state.bump_layer_epoch();
        at_the_seat(&game.state)
    });
    reading("  of which the board text", || Scenario::write(&game.state).to_string());
    if cfg!(debug_assertions) {
        // What the audit costs it: at every hit, as outside a seat, and none.
        reading("  audited at every hit", || Snapshot::build(&game.state, 0));
        game.state.pause_layer_audit();
        reading("  the audit paused", || Snapshot::build(&game.state, 0));
        game.state.resume_layer_audit();
    }
    reading("receive", || {
        let mut state = WindowState::default();
        state.receive(ToWindow::Prompt { snapshot: snapshot.clone(), prompt: prompt.clone(), yielding: None, why: None });
        state
    });
    let mut state = WindowState::default();
    state.receive(ToWindow::Prompt { snapshot, prompt, yielding: None, why: None });
    reading("views, every repaint", || (state.board_view(), state.prompt_view()));
    let destinations = vec![
        Destination::Savestate { at: 4, name: "Turn 2 · Precombat Main".to_string() },
        Destination::Savestate { at: 11, name: "Turn 3 · Declare Blockers".to_string() },
        Destination::Left(19),
    ];
    state.tools = Tools { undo_open: true, undo_answered: true, savestate_here: false, destinations, current: 23 };
    reading("tools view, every repaint", || state.tools_view());
    // The why panel (`setup-architecture.md` §7c): the seat's answer at the
    // board's priority question, about the permanent the layers did most to,
    // and about the first card in the asked seat's hand, whose question
    // section asks the cast check; and the panel's view, which `app::draw`
    // builds again at every repaint while it is open.
    let busiest = game.state.battlefield_ids_ordered().into_iter().max_by_key(|id| {
        explain(&game.state, *id).map_or(0, |explanation| explanation.steps.len())
    });
    let in_hand = game.state.players[asked].hand.first().copied();
    match (prompt_kind.as_str(), busiest, in_hand) {
        ("PriorityAction", Some(busiest), Some(in_hand)) => {
            let options: Vec<ChoiceOption> =
                candidate_priority_actions(&game.state, asked).into_iter().map(ChoiceOption::Action).collect();
            let context = ChoiceContext::new(ChoiceKind::PriorityAction);
            let question = OpenQuestion { player: asked, context: &context, options: &options };
            let at_the_seat = |about| game.state.audit_each_frame_once(|| why(&game.state, about, Some(&question)));
            reading("why, at a question", || at_the_seat(WhyAbout::Object(busiest)));
            reading("why of a card in hand, at a question", || at_the_seat(WhyAbout::Object(in_hand)));
            (state.why_path, state.why) = (vec![WhyAbout::Object(busiest)], Some(at_the_seat(WhyAbout::Object(busiest))));
            reading("why view, every repaint", || state.why_view());
        }
        _ => println!("the why panel: not read, since it needs a priority question first, a permanent and a card in hand"),
    }

    let mut editor = Editor::open(&text, Source::File(board), CardRegistry::default_registry()).unwrap_or_else(|refusal| panic!("{refusal}"));
    let opened = &editor;
    reading("editor view, every repaint", || opened.view());
    let mut life = 0;
    reading("an edit, at a click", || {
        life = 1 - life;
        editor.input(EditorInput::Number(BoardNumber::Life(0), 10 + life));
    });
    editor.input(EditorInput::Advanced(true));
    let advanced = &editor;
    reading("editor view, advanced on, every repaint", || advanced.view());
    reading("a line typed, added and undone, at clicks", || {
        editor.input(EditorInput::TypedLine("player 1: poison 3".to_string()));
        editor.input(EditorInput::AddTypedLine);
        editor.input(EditorInput::Undo);
    });
}

/// Runs `work` [`RUNS`] times and prints its median and slowest tenth, with
/// what its last run allocated, by when a warm memo is warm. What it returns
/// is dropped outside the timing.
fn reading<T>(name: &str, mut work: impl FnMut() -> T) {
    let mut times = Vec::with_capacity(RUNS);
    let mut counted = (0, 0);
    for run in 0..RUNS {
        let started = Instant::now();
        let (made, allocations) = counting(run == RUNS - 1, &mut work);
        times.push(started.elapsed());
        drop(made);
        if run == RUNS - 1 {
            counted = allocations;
        }
    }
    times.sort();
    let micros = |at: usize| times[at].as_secs_f64() * 1e6;
    println!(
        "{name:<28} {:>9.1} µs median {:>9.1} µs p90 {:>7} allocations {:>9} bytes",
        micros(RUNS / 2),
        micros(RUNS * 9 / 10),
        counted.0,
        counted.1
    );
}

/// `work`'s result, and the allocations and bytes it asked for when `count`.
fn counting<T>(count: bool, work: &mut impl FnMut() -> T) -> (T, (u64, u64)) {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    BYTES.store(0, Ordering::Relaxed);
    COUNTING.with(|on| on.set(count));
    let made = work();
    COUNTING.with(|on| on.set(false));
    (made, (ALLOCATIONS.load(Ordering::Relaxed), BYTES.load(Ordering::Relaxed)))
}

/// `System`, counting what this thread asks for while it has [`COUNTING`] on:
/// requested sizes, the way `mtgsim/tests/support/counting_allocator.rs`
/// counts a clone's.
struct CountingAllocator;

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

fn count(size: usize) {
    if COUNTING.try_with(Cell::get).unwrap_or(false) {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(size as u64, Ordering::Relaxed);
    }
}

// SAFETY: every call is forwarded to `System` unchanged; the counting beside
// it touches only atomics and a const-initialized thread-local.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        count(new_size);
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
