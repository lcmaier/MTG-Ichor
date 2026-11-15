//! `--seed N` means the same run, in any process.
//!
//! The fuzz harness advertises reproducibility, and the project's perf protocol
//! assumes it (on 2026-08-23, "200 games / seed 12345, back to back, ±3% band";
//! today `engineering-practices.md#cards-and-pools`'s interleaved A/B): without
//! it the two runs being compared are not doing the same amount of work, so a
//! real regression hides inside the spread and a phantom one appears.
//!
//! Three things have to hold. The randomness has to come from the seed — it
//! used to come from `rand::rng()`, which is seeded per process, so the seed
//! reached deck construction and stopped there. The ids have to come from the
//! game — an `ObjectId` is stamped from `GameState`'s own counter, so two runs
//! of one game name every object alike and their logs compare verbatim (until
//! A4g, 2026-09-16, ids were v4 UUIDs and the id column had to be stripped
//! first). And the *options* have to arrive in a fixed order: a hash map
//! hands its walker whatever order its hasher gives, which is not the game's.
//!
//! The first two are directly testable here. The third is not testable
//! end-to-end in one process: the id hasher is seeded once per process
//! (`types::ids::IdHash`), so two runs in one process see every map in the
//! same order whether or not the sweeps are ordered. CI's determinism step is
//! the end-to-end check — three `fuzz_games` runs at one seed under three
//! hasher seeds, byte-identical outside `=== Timing ===` — and here it is
//! tested at the mechanism instead: a battlefield sweep walks the object
//! table in id order, which neither the hasher nor the order of arrival
//! moves.

use mtgsim::cards::registry::CardRegistry;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::test_support::{put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx, vanilla_creature};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::zones::{Zone, ZoneChangeAct};
use mtgsim::ui::random::RandomDecisionProvider;

/// Play one short game and return its event log, rendered.
fn play_seeded_game(seed: u64) -> Vec<String> {
    play_seeded_game_with(seed, 2)
}

/// [`play_seeded_game`] at any table size. Four is where the rotation passes
/// over a player who has left (CR 800.4j/k) and the game goes on, which no
/// two-player game reaches.
fn play_seeded_game_with(seed: u64, players: usize) -> Vec<String> {
    let registry = CardRegistry::default_registry();
    let deck: Vec<_> = registry
        .card_names()
        .iter()
        .cycle()
        .take(60)
        .filter_map(|name| registry.create(name).ok())
        .collect();

    let mut game = Game::new(GameConfig::test(), vec![deck; players])
        .expect("game creation");
    game.state.record_events();
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

/// Verbatim, ids included: which card, which zone, which order, and which
/// object — the id column is the game's too, and it must match event for
/// event.
#[test]
fn test_same_seed_replays_the_same_game() {
    let first = play_seeded_game(0xFEED_BEEF);
    let second = play_seeded_game(0xFEED_BEEF);

    assert_eq!(first, second, "same seed produced two different games");
    assert!(first.len() > 20, "game was too short to prove anything");
}

/// The same claim at four seats — the first table size at which the priority
/// and turn rotations read `player_lost` for a game that continues.
#[test]
fn test_same_seed_replays_the_same_four_player_game() {
    let first = play_seeded_game_with(0xC0FF_EE42, 4);
    let second = play_seeded_game_with(0xC0FF_EE42, 4);
    assert_eq!(first, second, "same seed produced two different four-player games");
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

/// Eight creatures made in one order and put onto the battlefield in the
/// other come out of every sweep that feeds a decision in the order they were
/// made: the table's, not CR 613.7's.
#[test]
fn test_battlefield_sweeps_come_out_in_id_order() {
    let mut game = setup_two_player_game();
    let made: Vec<ObjectId> = (0..8).map(|_| put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0)).collect();
    for &creature in made.iter().rev() {
        game.change_zone(creature, Zone::Battlefield, ZoneChangeAct::Put, &test_ctx()).unwrap();
    }
    assert!(game.object_timestamp(made[0]) > game.object_timestamp(made[7]), "the newest object entered first");

    assert_eq!(game.battlefield_ids(), made);
    assert_eq!(mtgsim::oracle::board::permanents_controlled_by(&game, 0), made);
    assert_eq!(mtgsim::oracle::legality::legal_blockers(&game, 0), made);
}

/// CR 613.7e gives an Equipment a new timestamp each time it becomes attached,
/// and an attach to the host it is already on (CR 701.3b) gives it none. The
/// sweep order is the table's, so neither moves the Equipment in it.
#[test]
fn test_a_reattachment_moves_only_the_attachments_timestamp() {
    let mut game = setup_two_player_game();
    let equipment = put_on_battlefield(&mut game, mtgsim::test_support::equipment("Harness"), 0);
    let placed: Vec<ObjectId> = (0..8).map(|_| put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0)).collect();
    let swept = game.battlefield_ids();
    let stamped = game.object_timestamp(equipment);

    assert!(game.attach(equipment, placed[3]));
    let attached = game.object_timestamp(equipment);
    assert!(placed.iter().all(|&creature| game.object_timestamp(creature) < attached), "CR 613.7e: newer than every creature");
    assert!(attached > stamped);

    assert!(!game.attach(equipment, placed[3]), "CR 701.3b: the same host is not an attach");
    assert_eq!(game.object_timestamp(equipment), attached);

    assert!(game.attach(equipment, placed[5]));
    assert!(game.object_timestamp(equipment) > attached, "a new host, a new timestamp");
    assert_eq!(game.battlefield_ids(), swept, "and the sweep order never moved");
}
