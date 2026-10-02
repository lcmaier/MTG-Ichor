//! What one prompt costs the window, read on a large board:
//!
//!     cargo run --release --example prompt_cost [-- BOARD.scenario]
//!
//! Four readings, each the median and the slowest tenth of many runs, beside
//! the allocations and bytes one run asks for:
//! - **snapshot**: `Snapshot::build`, the engine thread's work at each prompt
//!   before it sends, read with the layer memo warm (the engine has usually
//!   walked the board by the time it asks) and cold (the epoch just bumped);
//!   **of which the board text**, `Scenario::write`, which it builds for
//!   "Save board as scenario";
//! - **receive**: `WindowState::receive`, the window's work as a prompt arrives;
//! - **views**: `board_view` and `prompt_view`, which `app::draw` builds again
//!   at every repaint.
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

use devgui::bridge::{GameSetup, Pool, ToWindow, spawn_game};
use devgui::snapshot::Snapshot;
use devgui::view_model::WindowState;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::scenario::Scenario;

const RUNS: usize = 200;

fn main() {
    let board = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("scenarios").join("large.scenario")
    });
    let text = std::fs::read_to_string(&board).unwrap_or_else(|e| panic!("cannot read {}: {e}", board.display()));
    let scenario = Scenario::parse(&text).unwrap_or_else(|refusal| panic!("{refusal}"));
    let mut game = scenario.build(&CardRegistry::default_registry()).unwrap_or_else(|refusal| panic!("{refusal}")).game;
    game.state.record_events();

    let engine = spawn_game(
        GameSetup { seed: scenario.seed, pool: Pool::Stress, log_path: None, scenario: Some(board.clone()) },
        Arc::new(|| {}),
    );
    let message = engine.from_engine.recv_timeout(Duration::from_secs(60)).expect("the engine's first prompt");
    let ToWindow::Prompt { snapshot, prompt, .. } = message else {
        panic!("the board's first message is not a prompt: {message:?}");
    };
    let objects = snapshot.players.iter().map(|p| p.hand.len() + p.library.len() + p.graveyard.len() + p.battlefield.len()).sum::<usize>();
    println!("{}: {objects} objects, a {} prompt of {} options", board.display(), prompt.kind, prompt.options.len());

    reading("snapshot, memo warm", || Snapshot::build(&game.state, 0));
    reading("snapshot, memo cold", || {
        game.state.bump_layer_epoch();
        Snapshot::build(&game.state, 0)
    });
    reading("  of which the board text", || Scenario::write(&game.state).to_string());
    reading("receive", || {
        let mut state = WindowState::default();
        state.receive(ToWindow::Prompt { snapshot: snapshot.clone(), prompt: prompt.clone(), yielding: None });
        state
    });
    let mut state = WindowState::default();
    state.receive(ToWindow::Prompt { snapshot, prompt, yielding: None });
    reading("views, every repaint", || (state.board_view(), state.prompt_view()));
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
