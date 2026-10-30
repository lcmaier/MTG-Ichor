//! Floors 2 and 3 in CI: what one clone of a Commander game allocates, at item
//! 143's checkpoints through the game's end and **at every decision**
//! (`engineering-practices.md#the-floors`; `clone-architecture.md` decision 6).
//! A search clones mid-turn too, where the stack and the triggers waiting hold
//! what between turns they do not: TR-5g's games reached 78 allocations at a
//! 22-trigger ordering prompt while every checkpoint read under 64.
//!
//! A search that stores states clones one at every iteration, so a clone has
//! to stay cheap and small however long the game runs
//! (`plans/references/ai-performance-floors.md`). This test asserts the
//! portable proxies: **at most 64 allocations** (floor 2's CI form) and **at
//! most 128 KB** (floor 3), counted exactly by `support/counting_allocator.rs`.
//! **Time is machine-bound**, so the second test, ignored, reads it at the
//! readiness pass (`spine-phase-audit.md`): `cargo test --release --test
//! clone_bound_test -- --ignored --nocapture`.
//!
//! **Its own binary.** The global allocator counts only while the measuring
//! thread has switched it on, and its counts are one pair of atomics, so the
//! two tests take turns on [`ONE_AT_A_TIME`] and nothing else in the process
//! reaches the count.
//!
//! **Release only**, as CI's own step runs it (`cargo test --release --test
//! clone_bound_test`): about two seconds there, and 675 s in a debug build
//! (three games), where the layer memo's audit re-walks every memo hit
//! (`compute_characteristics`).
//!
//! **The games are `fuzz_games`'**: its deck builder and its three seed
//! derivations, four seats of 100 cards at 40 life (`deal_commander_game`).
//! Three are `stress` seeds. 12345 and 777 are item 143's; 12351 is game 7 of
//! the 100-game sweep at 12345, which ran 184 turns on the tree this test was
//! written on, and a long game is what shows a size bounded by the board
//! rather than the turn count. The `stress` pool grows with every registered
//! card, so its games change with it; the test reads whatever checkpoints they
//! reach. **Then `close_out.py`'s 20 `performance` games**, the board the
//! budget is read on, so the floors are checked where the budget is, and the
//! same 20 on **the token board**, 50 Goblins a seat (`fuzz_games --tokens`),
//! where floor 1 fails (`codebase-state.md` item 354). `fuzz_games --pool
//! <pool> --players 4 --deck-size 100 --life 40 --seed <seed> --games 1` plays
//! any of them but for the first player's draw.
//!
//! **The fourth game is 12351 again, with Teferi's clause under player 0 from
//! setup on** (`phase_lj_cards::teferi_flash_clause`, `codebase-state.md` item
//! 181). A row reaching every zone but the battlefield makes the memo hold a
//! frame for every card in the game, and its `And` filter's two boxes are cloned
//! with the registry, so it is the heavier clone of item 181's two fixtures.

use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

use mtgsim::cards::phase_lj_cards::teferi_flash_clause;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{create_goblins_for_each_seat, deal_commander_game, put_on_battlefield, AtEachPrompt};
use mtgsim::ui::choice_types::ChoiceContext;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

#[path = "support/counting_allocator.rs"]
mod counting_allocator;
use counting_allocator::clone_cost;

#[path = "support/clone_timer.rs"]
mod clone_timer;
use clone_timer::clone_micros;

/// Floor 2's CI proxy.
const MAX_ALLOCATIONS: u64 = 64;
/// Floor 3.
const MAX_BYTES: u64 = 128 * 1024;

/// Item 143's checkpoints: the state between two `run_turn` calls, named by
/// the turn about to be played, and the game's end.
const CHECKPOINTS: [u32; 10] = [1, 10, 20, 30, 40, 50, 60, 80, 100, 150];

/// `fuzz_games`' turn limit.
const MAX_TURNS: u32 = 200;

/// The token board's Goblins a seat.
const GOBLINS_A_SEAT: usize = 50;

/// The decisions the timing test clones at in each group: the heaviest by
/// bytes (`clone-architecture.md` decision 6(d)).
const HEAVIEST: usize = 20;

/// The two tests share the allocator's counts.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// One of the games, and the group its readings are reported in.
struct Board<'a> {
    group: &'static str,
    name: String,
    registry: &'a CardRegistry,
    seed: u64,
    row: Option<fn() -> Arc<CardData>>,
    goblins_a_seat: usize,
}

/// The games, in order: the 24 the test was written with, then the token board.
fn boards<'a>(stress: &'a CardRegistry, performance: &'a CardRegistry) -> Vec<Board<'a>> {
    let board = |group, pool: &str, registry, seed, row: Option<fn() -> Arc<CardData>>, goblins_a_seat| {
        let with = if row.is_some() { " with Teferi's clause" } else { "" };
        let goblins = if goblins_a_seat > 0 { format!(", {goblins_a_seat} Goblins a seat") } else { String::new() };
        Board { group, name: format!("{pool} {seed}{with}{goblins}"), registry, seed, row, goblins_a_seat }
    };
    let mut boards = vec![
        board("the 24 games", "stress", stress, 12345, None, 0),
        board("the 24 games", "stress", stress, 777, None, 0),
        board("the 24 games", "stress", stress, 12351, None, 0),
        board("the 24 games", "stress", stress, 12351, Some(teferi_flash_clause), 0),
    ];
    // `close_out.py`'s `--games 20 --seed 12345 --pool performance`.
    boards.extend((12345..12365).map(|seed| board("the 24 games", "performance", performance, seed, None, 0)));
    boards.extend((12345..12365).map(|seed| board("the token board", "performance", performance, seed, None, GOBLINS_A_SEAT)));
    boards
}

/// Plays `board` to its end or the turn limit, calling `at_decision` at each
/// decision that reaches the agent and `at_checkpoint` at each checkpoint and
/// at the end.
fn play(board: &Board, at_decision: impl Fn(&GameState, &ChoiceContext), mut at_checkpoint: impl FnMut(&GameState, String)) {
    let (mut game, agents) = deal_commander_game(board.registry, board.seed);
    let dp = ManaWindowStop::new(AtEachPrompt::new(RandomDecisionProvider::seeded(agents), at_decision));
    game.setup(&dp).expect("setup");
    if let Some(card) = board.row {
        put_on_battlefield(&mut game.state, card(), 0);
    }
    if board.goblins_a_seat > 0 {
        create_goblins_for_each_seat(&mut game.state, board.goblins_a_seat, &dp);
    }
    let mut turns = 0;
    while !game.is_over() && turns < MAX_TURNS {
        if CHECKPOINTS.contains(&game.state.turn_number) {
            at_checkpoint(&game.state, format!("turn {}", game.state.turn_number));
        }
        game.run_turn(&dp).expect("turn");
        turns += 1;
    }
    at_checkpoint(&game.state, format!("end, {turns} turns"));
}

/// Takes this test's turn and pins the id hasher's seed.
///
/// Bytes vary with the hasher's seed: a map doubles one checkpoint earlier
/// under some seeds (item 143's re-take). `types::ids` reads the variable once,
/// when the first id map is built, which is after this.
fn take_the_allocator() -> std::sync::MutexGuard<'static, ()> {
    let turn = ONE_AT_A_TIME.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    // SAFETY: both tests set it holding the lock, before either builds a game,
    // and nothing else in this binary reads the environment.
    unsafe { std::env::set_var("MTGSIM_HASH_SEED", "1") };
    turn
}

/// The prompt's kind, short.
fn named(context: &ChoiceContext) -> String {
    format!("{:?}", context.kind).chars().take(48).collect()
}

fn kilobytes(bytes: u64) -> f64 {
    bytes as f64 / 1024.0
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release only: CI runs it with --release")]
fn a_commander_clone_stays_under_the_allocation_and_byte_floors() {
    let _turn = take_the_allocator();
    let stress = CardRegistry::default_registry();
    let performance = CardRegistry::performance_pool();
    let mut readings = Vec::new();
    let mut over = Vec::new();
    let mut decision_count = 0;
    for board in boards(&stress, &performance) {
        let decisions = Cell::new(0usize);
        let worst = RefCell::new((0, 0, String::new()));
        let over_at_decisions = RefCell::new(Vec::new());
        let at_decision = |game: &GameState, prompt: &ChoiceContext| {
            let (allocations, bytes) = clone_cost(game);
            decisions.set(decisions.get() + 1);
            let at = || format!("turn {} {}: {allocations} allocations, {:.1} KB", game.turn_number, named(prompt), kilobytes(bytes));
            if allocations > MAX_ALLOCATIONS || bytes > MAX_BYTES {
                over_at_decisions.borrow_mut().push(at());
            }
            let mut worst = worst.borrow_mut();
            if allocations > worst.0 || (allocations == worst.0 && bytes > worst.1) {
                *worst = (allocations, bytes, at());
            }
        };
        play(&board, at_decision, |game, at| {
            let (allocations, bytes) = clone_cost(game);
            let line = format!("{}, {at}: {allocations} allocations, {:.1} KB", board.name, kilobytes(bytes));
            if allocations > MAX_ALLOCATIONS || bytes > MAX_BYTES {
                over.push(line.clone());
            }
            readings.push(line);
        });
        decision_count += decisions.get();
        let (allocations, _, at) = worst.into_inner();
        if allocations > 0 {
            readings.push(format!("{}, worst of its decisions, {at}", board.name));
        }
        over.extend(over_at_decisions.into_inner().into_iter().map(|line| format!("{}, {line}", board.name)));
    }
    readings.push(format!("{decision_count} decisions read"));
    println!("{}", readings.join("\n"));
    assert!(
        over.is_empty(),
        "a clone passed {MAX_ALLOCATIONS} allocations or {} KB:\n{}\n\nevery reading:\n{}",
        MAX_BYTES / 1024,
        over.join("\n"),
        readings.join("\n"),
    );
}

/// Floor 2's own bound, a clone of at most 10 µs, read where the test above
/// reads its proxy: at every checkpoint, and at each group's 20 decisions
/// heaviest by bytes, each held until the games end and then timed. The
/// median of five means of 2,000 clone-and-drops, with the counting allocator
/// present and switched off. Printed, never asserted: a wall-clock reading
/// moves with the machine (`plans/fuzz-record.md`, the floors' readings).
#[test]
#[ignore = "a wall-clock reading for the readiness pass: cargo test --release --test clone_bound_test -- --ignored --nocapture"]
fn a_commander_clone_takes_microseconds() {
    let _turn = take_the_allocator();
    let stress = CardRegistry::default_registry();
    let performance = CardRegistry::performance_pool();
    let boards = boards(&stress, &performance);
    let mut groups: Vec<&'static str> = boards.iter().map(|board| board.group).collect();
    groups.dedup();
    for group in groups {
        let mut checkpoints: Vec<(f64, String)> = Vec::new();
        // (bytes, where, the state), heaviest first.
        let heaviest: RefCell<Vec<(u64, String, GameState)>> = RefCell::new(Vec::new());
        for board in boards.iter().filter(|board| board.group == group) {
            let at_decision = |game: &GameState, prompt: &ChoiceContext| {
                let (_, bytes) = clone_cost(game);
                let mut heaviest = heaviest.borrow_mut();
                if heaviest.len() < HEAVIEST || heaviest.last().is_some_and(|lightest| bytes > lightest.0) {
                    let at = format!("{}, turn {} {}", board.name, game.turn_number, named(prompt));
                    heaviest.push((bytes, at, game.clone()));
                    heaviest.sort_by_key(|heavy| std::cmp::Reverse(heavy.0));
                    heaviest.truncate(HEAVIEST);
                }
            };
            play(board, at_decision, |game, at| checkpoints.push((clone_micros(game, 2_000), format!("{}, {at}", board.name))));
        }
        let decisions: Vec<(f64, String)> =
            heaviest.into_inner().into_iter().map(|(bytes, at, state)| (clone_micros(&state, 2_000), format!("{at}, {:.1} KB", kilobytes(bytes)))).collect();
        for (what, mut times) in [("checkpoints", checkpoints), ("decisions heaviest by bytes", decisions)] {
            times.sort_by(|a, b| a.0.total_cmp(&b.0));
            let (Some(median), Some(worst)) = (times.get(times.len() / 2), times.last()) else { continue };
            println!("{group}, {} {what}: median {:.2} µs, worst {:.2} µs ({})", times.len(), median.0, worst.0, worst.1);
        }
    }
}
