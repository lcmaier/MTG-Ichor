//! Floors 2 and 3 in CI: what one clone of a Commander game allocates, at item
//! 143's checkpoints through the game's end (`engineering-practices.md` §3.1).
//!
//! A search that stores states clones one at every iteration, so a clone has
//! to stay cheap and small however long the game runs
//! (`plans/references/ai-performance-floors.md`). Time is machine-bound and
//! read at the readiness pass; this test asserts the portable proxies:
//! **at most 64 allocations** (floor 2's CI form) and **at most 128 KB** (floor
//! 3), counted exactly by the allocator below.
//!
//! **Its own binary, with one test.** The global allocator counts only while
//! the measuring thread has switched it on, so nothing else in the process
//! could reach the count, and one test means one thread to switch it.
//!
//! **Release only**, as CI's own step runs it (`cargo test --release --test
//! clone_bound_test`): 0.26 s there, and 675 s in a debug build, where the
//! layer memo's audit re-walks every memo hit (`compute_characteristics`).
//!
//! **The games are `fuzz_games`'**: its deck builder and its three seed
//! derivations, four seats of 100 cards at 40 life on the `stress` pool.
//! Seeds 12345 and 777 are item 143's; 12351 is game 7 of the 100-game sweep at
//! 12345, which ran 184 turns on the tree this test was written on, and a long
//! game is what shows a size bounded by the board rather than the turn count.
//! The `stress` pool grows with every registered card, so its games change
//! with it; the test reads whatever checkpoints they reach, and `fuzz_games
//! --pool stress --players 4 --deck-size 100 --life 40 --seed <seed> --games 1`
//! plays the same game.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use rand::rngs::StdRng;
use rand::SeedableRng;

use mtgsim::cards::random_deck::random_deck;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::GameState;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

/// `System`, counting the allocations a thread asks for while it has
/// [`COUNTING`] on. Requested sizes, so the count is the program's and not an
/// allocator's rounding, and the same under any allocator.
struct CountingAllocator;

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

fn count(size: usize) {
    // `try_with`: a thread being torn down has no flag left to read.
    if COUNTING.try_with(Cell::get).unwrap_or(false) {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(size as u64, Ordering::Relaxed);
    }
}

// SAFETY: every call is forwarded to `System` unchanged; the counting beside it
// touches only atomics and a const-initialized thread-local, and allocates
// nothing.
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

/// Floor 2's CI proxy.
const MAX_ALLOCATIONS: u64 = 64;
/// Floor 3.
const MAX_BYTES: u64 = 128 * 1024;

/// Item 143's checkpoints: the state between two `run_turn` calls, named by
/// the turn about to be played, and the game's end.
const CHECKPOINTS: [u32; 10] = [1, 10, 20, 30, 40, 50, 60, 80, 100, 150];

/// `fuzz_games`' turn limit.
const MAX_TURNS: u32 = 200;

/// The allocations and bytes of one clone of `state`, held until counted.
fn clone_cost(state: &GameState) -> (u64, u64) {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    BYTES.store(0, Ordering::Relaxed);
    COUNTING.with(|on| on.set(true));
    let held = state.clone();
    COUNTING.with(|on| on.set(false));
    let cost = (ALLOCATIONS.load(Ordering::Relaxed), BYTES.load(Ordering::Relaxed));
    drop(held);
    cost
}

/// `fuzz_games`' game `game_seed` on the Commander board, set up.
fn commander_game(registry: &CardRegistry, game_seed: u64) -> (Game, ManaWindowStop<RandomDecisionProvider>) {
    let mut deck_rng = StdRng::seed_from_u64(game_seed);
    let shuffle_seed = game_seed ^ 0x9E37_79B9_7F4A_7C15;
    let dp_seed = game_seed ^ 0xD1B5_4A32_D192_ED03;
    let decks: Vec<Vec<Arc<CardData>>> = (0..4).map(|_| random_deck(registry, &mut deck_rng, &[], 1, 100)).collect();
    let mut config = GameConfig::test();
    config.starting_life = 40;
    let mut game = Game::new(config, decks).expect("game creation");
    game.reseed(shuffle_seed);
    let dp = ManaWindowStop::new(RandomDecisionProvider::seeded(dp_seed));
    game.setup(&dp).expect("setup");
    (game, dp)
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release only: CI runs it with --release")]
fn a_commander_clone_stays_under_the_allocation_and_byte_floors() {
    // Bytes vary with the id hasher's seed: a map doubles one checkpoint
    // earlier under some seeds (item 143's re-take). `types::ids` reads the
    // variable once, when the first id map is built, which is below.
    // SAFETY: this binary's one test sets it before any other thread exists
    // that could read the environment.
    unsafe { std::env::set_var("MTGSIM_HASH_SEED", "1") };

    let registry = CardRegistry::default_registry();
    let mut readings = Vec::new();
    let mut over = Vec::new();
    for seed in [12345u64, 777, 12351] {
        let (mut game, dp) = commander_game(&registry, seed);
        let mut turns = 0;
        let mut read = |game: &Game, at: String| {
            let (allocations, bytes) = clone_cost(&game.state);
            let line = format!("seed {seed}, {at}: {allocations} allocations, {:.1} KB", bytes as f64 / 1024.0);
            if allocations > MAX_ALLOCATIONS || bytes > MAX_BYTES {
                over.push(line.clone());
            }
            readings.push(line);
        };
        while !game.is_over() && turns < MAX_TURNS {
            if CHECKPOINTS.contains(&game.state.turn_number) {
                read(&game, format!("turn {}", game.state.turn_number));
            }
            game.run_turn(&dp).expect("turn");
            turns += 1;
        }
        read(&game, format!("end, {turns} turns"));
    }
    println!("{}", readings.join("\n"));
    assert!(
        over.is_empty(),
        "a clone passed {MAX_ALLOCATIONS} allocations or {} KB:\n{}\n\nevery reading:\n{}",
        MAX_BYTES / 1024,
        over.join("\n"),
        readings.join("\n"),
    );
}
