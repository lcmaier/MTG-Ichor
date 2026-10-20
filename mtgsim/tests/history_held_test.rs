//! What a game holds for its history log (`triggers-architecture.md` §12,
//! TR-5h): the rows and the frames they keep alive, read at item 143's
//! checkpoints through each game's end, beside what one clone copies.
//!
//! The log is kept for the whole game, so what a game holds grows with its
//! events; no clone copies it (a clone copies the chain's head). **At most
//! about 5 KB a turn** is TR-5h's measurement gate, asserted at each game's
//! end. A frame is counted whole, its inline size and its heap, even where
//! the layer memo or a later row shares it, so the reading is a ceiling.
//!
//! **Release only**, as `clone_bound_test.rs` is, and on its games:
//! `fuzz_games`' deck builder and seed derivations, four seats of 100 cards at
//! 40 life, three `stress` seeds and `close_out.py`'s 20 `performance` games.

use std::collections::HashSet;
use std::sync::Arc;

use rand::rngs::StdRng;
use rand::SeedableRng;

use mtgsim::cards::random_deck::random_deck;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::layers::types::EffectiveCharacteristics;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::history_log::HistoryLog;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

#[path = "support/counting_allocator.rs"]
mod counting_allocator;
use counting_allocator::clone_cost;

/// TR-5h's gate: what a game may hold for its log, per turn played.
const MAX_BYTES_PER_TURN: u64 = 5 * 1024;

/// Item 143's checkpoints, as `clone_bound_test.rs` reads them.
const CHECKPOINTS: [u32; 10] = [1, 10, 20, 30, 40, 50, 60, 80, 100, 150];

/// `fuzz_games`' turn limit.
const MAX_TURNS: u32 = 200;

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

/// The log's rows, and each distinct frame they point to, whole.
fn held_bytes(log: &HistoryLog) -> (u64, usize) {
    let mut seen: HashSet<*const EffectiveCharacteristics> = HashSet::new();
    let mut frames = 0u64;
    for frame in log.rows_newest_first().filter_map(|row| row.frame.as_ref()) {
        if seen.insert(Arc::as_ptr(frame)) {
            let (_, heap) = clone_cost(&**frame);
            frames += (std::mem::size_of::<EffectiveCharacteristics>() + 2 * std::mem::size_of::<usize>()) as u64 + heap;
        }
    }
    (log.row_bytes_held() as u64 + frames, seen.len())
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release only, as clone_bound_test is")]
fn a_game_holds_its_log_within_the_gate() {
    // SAFETY: this binary's one test sets it before any other thread exists
    // that could read the environment.
    unsafe { std::env::set_var("MTGSIM_HASH_SEED", "1") };

    let stress = CardRegistry::default_registry();
    let performance = CardRegistry::performance_pool();
    let mut games: Vec<(&str, &CardRegistry, u64)> = vec![("stress", &stress, 12345), ("stress", &stress, 777), ("stress", &stress, 12351)];
    games.extend((12345..12365).map(|seed| ("performance", &performance, seed)));

    let mut readings = Vec::new();
    let mut over = Vec::new();
    for (pool, registry, seed) in games {
        let (mut game, dp) = commander_game(registry, seed);
        let mut turns = 0u32;
        let mut read = |game: &Game, turns: u32, at: String| -> u64 {
            let log = &game.state.history.log;
            let (held, frames) = held_bytes(log);
            let (allocations, clone_bytes) = clone_cost(&game.state);
            let per_turn = held / u64::from(turns.max(1));
            readings.push(format!(
                "{pool} {seed}, {at}: {} rows, {frames} frames, held {:.1} KB ({per_turn} B a turn); a clone {allocations} allocations, {:.1} KB",
                log.len(),
                held as f64 / 1024.0,
                clone_bytes as f64 / 1024.0,
            ));
            per_turn
        };
        while !game.is_over() && turns < MAX_TURNS {
            if CHECKPOINTS.contains(&game.state.turn_number) {
                read(&game, turns, format!("turn {}", game.state.turn_number));
            }
            game.run_turn(&dp).expect("turn");
            turns += 1;
        }
        let per_turn = read(&game, turns, format!("end, {turns} turns"));
        if per_turn > MAX_BYTES_PER_TURN {
            over.push(format!("{pool} {seed}: {per_turn} B a turn over {turns} turns"));
        }
    }
    println!("{}", readings.join("\n"));
    assert!(over.is_empty(), "a game held more than {MAX_BYTES_PER_TURN} B a turn:\n{}", over.join("\n"));
}
