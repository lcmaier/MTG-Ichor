//! Floors 2 and 3 on the boards no seed reaches (`clone-architecture.md`
//! decision 6): a stack 8,000 triggers deep and the queue that fills it, and a
//! game whose tokens came and went. `clone_bound_test` reads the seeds' games
//! at every decision; this reads the depths and the history beside them, under
//! the same bounds: **at most 64 allocations** (floor 2's CI form) and **at
//! most 128 KB** (floor 3), counted exactly by `support/counting_allocator.rs`.
//! The second test, ignored, times the same boards at the readiness pass:
//! `cargo test --release --test clone_fixture_test -- --ignored --nocapture`.
//!
//! **Its own binary**, for the reason `clone_bound_test` gives: the global
//! allocator counts only while the measuring thread has switched it on, and
//! the two tests take turns on [`ONE_AT_A_TIME`]. **Release only**, as CI's
//! step runs it: the wipe dispatches and places 8,000 triggers, which a debug
//! build's layer-memo audit re-walks.

use std::sync::Mutex;

use mtgsim::cards::phase_tr1_cards::blood_artist;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::oracle::characteristics::is_creature;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{create_goblins_for_each_seat, put_on_battlefield, setup_game};
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

/// The two tests share the allocator's counts.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Every creature on the battlefield destroyed in one batch.
fn wipe(game: &mut GameState, dp: &ManaWindowStop<RandomDecisionProvider>) {
    let creatures: Vec<_> = game.battlefield_ids_ordered().into_iter().filter(|&id| is_creature(game, id)).collect();
    game.execute_actions(creatures.into_iter().map(|object| GameAction::Destroy { object }).collect(), &ActionContext::new(dp)).unwrap();
}

/// Builds each board and hands it to `read`, named.
fn each_board(mut read: impl FnMut(&GameState, &str)) {
    // `codebase-state.md` item 183's wipe: forty death watchers ("whenever a
    // creature dies, target player loses 1 life and you gain 1 life") over
    // four seats, 160 goblins, one destruction of all 200, so 8,000 triggers.
    let dp = ManaWindowStop::new(RandomDecisionProvider::seeded(1));
    let mut game = setup_game(4);
    for player in &mut game.players {
        player.life_total = 10_000_000;
    }
    for i in 0..40 {
        put_on_battlefield(&mut game, blood_artist(), i % 4);
    }
    create_goblins_for_each_seat(&mut game, 40, &dp);
    game.pending_triggers.clear();
    read(&game, "200 permanents");
    wipe(&mut game, &dp);
    assert_eq!(game.pending_triggers.len(), 8_000, "each watcher sees all 200 die");
    read(&game, "8,000 triggers waiting");
    game.perform_sba_and_triggers(&dp).unwrap();
    assert_eq!(game.stack.len(), 8_000);
    read(&game, "8,000 triggers stacked");

    // Twenty rounds of 200 tokens made, walked and destroyed, on four seats
    // and no cards: what the game keeps of objects that are gone.
    let mut game = setup_game(4);
    for _ in 0..20 {
        create_goblins_for_each_seat(&mut game, 50, &dp);
        wipe(&mut game, &dp);
        game.perform_sba_and_triggers(&dp).unwrap();
    }
    assert!(game.objects.is_empty(), "every token ceased to exist (CR 704.5d)");
    read(&game, "after 4,000 tokens came and went");
}

/// Takes this test's turn and pins the id hasher's seed, as `clone_bound_test`
/// does.
fn take_the_allocator() -> std::sync::MutexGuard<'static, ()> {
    let turn = ONE_AT_A_TIME.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    // SAFETY: both tests set it holding the lock, before either builds a game,
    // and nothing else in this binary reads the environment.
    unsafe { std::env::set_var("MTGSIM_HASH_SEED", "1") };
    turn
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release only: CI runs it with --release")]
fn a_clone_on_a_deep_stack_and_after_a_token_churn_stays_under_the_floors() {
    let _turn = take_the_allocator();
    let mut readings = Vec::new();
    each_board(|game, at| {
        let (allocations, bytes) = clone_cost(game);
        readings.push((format!("{at}: {allocations} allocations, {:.1} KB", bytes as f64 / 1024.0), allocations, bytes));
    });
    let lines: Vec<&str> = readings.iter().map(|(line, _, _)| line.as_str()).collect();
    println!("{}", lines.join("\n"));
    let over: Vec<&str> = readings
        .iter()
        .filter(|(_, allocations, bytes)| *allocations > MAX_ALLOCATIONS || *bytes > MAX_BYTES)
        .map(|(line, _, _)| line.as_str())
        .collect();
    assert!(over.is_empty(), "a clone passed {MAX_ALLOCATIONS} allocations or {} KB:\n{}", MAX_BYTES / 1024, over.join("\n"));
}

/// Floor 2's own bound on the same boards: the median of five means of 2,000
/// clone-and-drops each. Printed, never asserted, as `clone_bound_test`'s.
#[test]
#[ignore = "a wall-clock reading for the readiness pass: cargo test --release --test clone_fixture_test -- --ignored --nocapture"]
fn a_clone_on_a_deep_stack_and_after_a_token_churn_takes_microseconds() {
    let _turn = take_the_allocator();
    each_board(|game, at| println!("{at}: {:.2} µs", clone_micros(game, 2_000)));
}
