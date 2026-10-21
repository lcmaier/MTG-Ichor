//! Floors 2 and 3 on the boards no seed reaches (`clone-architecture.md`
//! decision 6): a stack 8,000 triggers deep and the queue that fills it, and a
//! game whose tokens came and went. `clone_bound_test` reads the seeds' games
//! at every decision; this reads the depths and the history beside them, under
//! the same bounds: **at most 64 allocations** (floor 2's CI form) and **at
//! most 128 KB** (floor 3), counted exactly by `support/counting_allocator.rs`.
//!
//! **Its own binary, with one test**, for the reason `clone_bound_test` gives:
//! the global allocator counts only while the measuring thread has switched it
//! on. **Release only**, as CI's step runs it: the wipe dispatches and places
//! 8,000 triggers, which a debug build's layer-memo audit re-walks.

use mtgsim::cards::phase_re_cards::goblin_token;
use mtgsim::cards::phase_tr1_cards::blood_artist;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::oracle::characteristics::is_creature;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_on_battlefield, setup_game};
use mtgsim::types::effects::TokenDef;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

#[path = "support/counting_allocator.rs"]
mod counting_allocator;
use counting_allocator::clone_cost;

/// Floor 2's CI proxy.
const MAX_ALLOCATIONS: u64 = 64;
/// Floor 3.
const MAX_BYTES: u64 = 128 * 1024;

/// `n` goblin tokens for each of four seats.
fn tokens_for_each_seat(game: &mut GameState, n: usize, dp: &ManaWindowStop<RandomDecisionProvider>) {
    let goblins: Vec<TokenDef> = vec![goblin_token(); n];
    for player in 0..4 {
        let create = GameAction::CreateTokens { defs: goblins.clone(), controller: player, attacking: None };
        game.execute_action(create, &ActionContext::new(dp)).unwrap();
    }
}

/// Every creature on the battlefield destroyed in one batch.
fn wipe(game: &mut GameState, dp: &ManaWindowStop<RandomDecisionProvider>) {
    let creatures: Vec<_> = game.battlefield_ids_ordered().into_iter().filter(|&id| is_creature(game, id)).collect();
    game.execute_actions(creatures.into_iter().map(|object| GameAction::Destroy { object }).collect(), &ActionContext::new(dp)).unwrap();
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release only: CI runs it with --release")]
fn a_clone_on_a_deep_stack_and_after_a_token_churn_stays_under_the_floors() {
    // SAFETY: this binary's one test sets it before any other thread exists
    // that could read the environment.
    unsafe { std::env::set_var("MTGSIM_HASH_SEED", "1") };
    let mut readings = Vec::new();
    let mut read = |game: &GameState, at: &str| {
        let (allocations, bytes) = clone_cost(game);
        readings.push((format!("{at}: {allocations} allocations, {:.1} KB", bytes as f64 / 1024.0), allocations, bytes));
    };

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
    tokens_for_each_seat(&mut game, 40, &dp);
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
        tokens_for_each_seat(&mut game, 50, &dp);
        wipe(&mut game, &dp);
        game.perform_sba_and_triggers(&dp).unwrap();
    }
    assert!(game.objects.is_empty(), "every token ceased to exist (CR 704.5d)");
    read(&game, "after 4,000 tokens came and went");

    let lines: Vec<&str> = readings.iter().map(|(line, _, _)| line.as_str()).collect();
    println!("{}", lines.join("\n"));
    let over: Vec<&str> = readings
        .iter()
        .filter(|(_, allocations, bytes)| *allocations > MAX_ALLOCATIONS || *bytes > MAX_BYTES)
        .map(|(line, _, _)| line.as_str())
        .collect();
    assert!(over.is_empty(), "a clone passed {MAX_ALLOCATIONS} allocations or {} KB:\n{}", MAX_BYTES / 1024, over.join("\n"));
}
