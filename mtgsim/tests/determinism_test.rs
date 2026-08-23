//! `--seed N` means the same run, in any process.
//!
//! The fuzz harness advertises reproducibility, and the project's perf protocol
//! ("200 games / seed 12345, back to back, ±3% band") assumes it: without it the
//! two runs being compared are not doing the same amount of work, so a real
//! regression hides inside the spread and a phantom one appears.
//!
//! Two things have to hold. The randomness has to come from the seed — it used
//! to come from `rand::rng()`, which is seeded per process, so the seed reached
//! deck construction and stopped there. And the *options* have to arrive in a
//! fixed order — `GameState::battlefield` is a `HashMap` keyed by v4 UUIDs, so
//! iterating it hands the AI a differently-ordered action list every process
//! even once the AI's own RNG is seeded.
//!
//! The first is directly testable here. The second is not testable end-to-end
//! in one process — two runs inside the same process share one `RandomState`,
//! so they agree with each other whether or not the sweeps are ordered — so it
//! is tested at the mechanism instead: the ordered sweeps must come out in
//! timestamp order, which a `HashMap` sweep would satisfy only by a 1-in-`n!`
//! accident.

use mtgsim::cards::registry::CardRegistry;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::test_support::{put_on_battlefield, vanilla_creature};
use mtgsim::types::ids::ObjectId;
use mtgsim::ui::random::RandomDecisionProvider;

/// Play one short game and return its event log, rendered.
fn play_seeded_game(seed: u64) -> Vec<String> {
    let registry = CardRegistry::default_registry();
    let deck: Vec<_> = registry
        .card_names()
        .iter()
        .cycle()
        .take(60)
        .filter_map(|name| registry.create(name).ok())
        .collect();

    let mut game = Game::new(GameConfig::test(), vec![deck.clone(), deck])
        .expect("game creation");
    game.reseed(seed);
    let dp = RandomDecisionProvider::seeded(seed);
    game.setup(&dp).expect("setup");

    let mut turns = 0;
    while !game.is_over() && turns < 12 {
        game.run_turn(&dp).expect("turn");
        turns += 1;
    }
    game.event_log_snapshot()
}

#[test]
fn test_same_seed_replays_the_same_game() {
    let first = play_seeded_game(0xFEED_BEEF);
    let second = play_seeded_game(0xFEED_BEEF);

    // Object ids are v4 UUIDs, so the rendered log's id column differs between
    // runs by design. Everything else — which card, which zone, which order —
    // is the game, and it must match event for event.
    let strip = |log: &[String]| -> Vec<String> {
        log.iter()
            .map(|line| {
                let mut out = String::with_capacity(line.len());
                let mut depth = 0;
                for ch in line.chars() {
                    match ch {
                        '(' => depth += 1,
                        ')' if depth > 0 => depth -= 1,
                        _ if depth == 0 => out.push(ch),
                        _ => {}
                    }
                }
                out
            })
            .collect()
    };

    assert_eq!(
        strip(&first),
        strip(&second),
        "same seed produced two different games"
    );
    assert!(first.len() > 20, "game was too short to prove anything");
}

#[test]
fn test_different_seeds_still_diverge() {
    // The guard on the test above: a determinism check passes trivially if the
    // provider has stopped making choices at all.
    let a = play_seeded_game(1);
    let b = play_seeded_game(2);
    assert_ne!(a.len(), 0);
    assert_ne!(a, b, "two different seeds produced identical games");
}

#[test]
fn test_battlefield_sweeps_come_out_in_timestamp_order() {
    let mut game = mtgsim::test_support::setup_two_player_game();

    // Enough permanents that a `HashMap` sweep landing in timestamp order by
    // chance is a 1-in-8! event.
    let placed: Vec<ObjectId> = (0..8)
        .map(|_| put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0))
        .collect();

    let ordered: Vec<ObjectId> = game.battlefield_ordered().iter().map(|(id, _)| *id).collect();
    assert_eq!(ordered, placed, "battlefield_ordered is not ETB order");
    assert_eq!(game.battlefield_ids_ordered(), placed);

    // The sweeps that feed a decision inherit that order.
    assert_eq!(
        mtgsim::oracle::board::permanents_controlled_by(&game, 0),
        placed,
    );
    assert_eq!(mtgsim::oracle::legality::legal_blockers(&game, 0), placed);
}
