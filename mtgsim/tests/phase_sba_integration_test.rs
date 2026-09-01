//! State-based actions reached the way a game reaches them — through
//! registered cards.
//!
//! `engine/sba.rs` already unit-tests every SBA here. What it cannot show is
//! that the *card pool* can build the scenario, and
//! `engineering-practices.md` §3.3 is explicit that those are two measurements:
//! "a bespoke fixture can cover an atom while the registered pool cannot build
//! the same scenario". CR 704.5q's annihilation sweep is the worked example —
//! two passing unit tests since Phase 5-Pre, and **zero occurrences across 200
//! stress games** until [`battlegrowth`] was registered.
//!
//! So every test below builds its board from registry cards, and the fixtures
//! it does use ([`vanilla_creature`]) are only ever the *other* creature on the
//! board — never the one the rule is about.

use mtgsim::cards::phase_rc_cards::{adaptive_shimmerer, chainbreaker};
use mtgsim::cards::phase_sba_cards::battlegrowth;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::actions::ZoneChangeCause;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::CardData;
use mtgsim::oracle::characteristics::{get_effective_power, get_effective_toughness};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_graveyard, put_in_hand, setup_two_player_game, test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    CounterType, EffectRecipient, PermanentFilter, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Put a card onto the battlefield without casting it.
///
/// The counters Chainbreaker enters with are CR 122.6a's, which ride the entry
/// itself rather than the cast, so this reaches the same board as a resolved
/// spell without putting a generic-mana allocation prompt in the way.
fn reanimate(game: &mut GameState, card: Arc<CardData>, player: usize) -> ObjectId {
    let id = put_in_graveyard(game, card, player);
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx())
        .expect("it enters");
    id
}

/// Cast [`battlegrowth`] from `player`'s hand at the only legal target.
///
/// Index 0 is unambiguous *because* each caller leaves exactly one creature on
/// the battlefield — `enumerate_legal_selections` returns an ordered list and a
/// test that picked out of several would be asserting on that order instead of
/// on CR 704.5q.
fn cast_battlegrowth_at_the_only_creature(game: &mut GameState, player: usize) -> ObjectId {
    let id = put_in_hand(game, battlegrowth(), player);
    game.players[player].mana_pool.add(ManaType::Green, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(
                SelectionFilter::Permanent(PermanentFilter::ByType(CardType::Creature)),
                TargetCount::Exactly(1),
            ),
            spell_id: id,
        },
        vec![0],
    );

    game.cast_spell(player, id, &decisions).expect("it is castable");
    game.resolve_top_of_stack(&decisions).expect("it resolves");
    // CR 704.3 — the check a player receiving priority after the resolution
    // would trigger. `resolve_top_of_stack` deliberately does not run it, so a
    // test that skipped this would show counters piling up and call it a rule.
    game.check_state_based_actions(&test_dp()).unwrap();
    id
}

/// Every `CountersAnnihilated` in the log, as `(object, pairs_removed)`.
fn annihilations(game: &GameState) -> Vec<(ObjectId, u32)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::CountersAnnihilated { object_id, pairs_removed } => {
                Some((*object_id, *pairs_removed))
            }
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// CR 704.5q — +1/+1 and -1/-1 counter annihilation
// ---------------------------------------------------------------------------

/// The headline: two registered cards, and the sweep runs.
///
/// `COVERS-PARTIAL` rather than `COVERS`, and the gap is worth naming. The atom
/// is 3 +1/+1 against 2 -1/-1 in **one** check, so `min` removes two pairs at
/// once. The registered pool cannot build that: CR 704.3 checks state-based
/// actions every time a player would receive priority, which is after every
/// resolution, so a second Battlegrowth always arrives at a board the sweep has
/// already cleaned. `min(1, n)` is what real play reaches, and asserting
/// otherwise here would be asserting on a fixture again. The full atom stays
/// with the unit test in `engine/sba.rs` that builds it directly.
// COVERS-PARTIAL: ATOM-122.3-001
#[test]
fn test_battlegrowth_and_chainbreaker_reach_counter_annihilation() {
    let mut game = setup_two_player_game();

    // Chainbreaker: a 3/3 that enters with two -1/-1 counters, so the board
    // sees a 1/1 and the -1/-1 half of CR 704.5q is on the battlefield.
    let chain = reanimate(&mut game, chainbreaker(), 0);
    assert_eq!(
        game.battlefield[&chain].counter_count(CounterType::MinusOneMinusOne),
        2
    );
    assert!(
        annihilations(&game).is_empty(),
        "one sign alone is not CR 704.5q's condition"
    );

    cast_battlegrowth_at_the_only_creature(&mut game, 0);

    // The +1/+1 counter landed and was annihilated against one -1/-1, leaving
    // the other. Both counts, because `min` has to take from each kind.
    let entry = &game.battlefield[&chain];
    assert_eq!(entry.counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(entry.counter_count(CounterType::MinusOneMinusOne), 1);
    assert_eq!(annihilations(&game), vec![(chain, 1)]);

    // Layer 7d reads what survived: a 3/3 with one -1/-1 counter left.
    assert_eq!(get_effective_power(&game, chain), Some(2));
    assert_eq!(get_effective_toughness(&game, chain), Some(2));
}

/// The sweep runs again on the next pair, which is what makes it a *state*-based
/// action rather than a one-shot on the entry.
#[test]
fn test_a_second_battlegrowth_annihilates_the_last_counter() {
    let mut game = setup_two_player_game();

    let chain = reanimate(&mut game, chainbreaker(), 0);
    cast_battlegrowth_at_the_only_creature(&mut game, 0);
    cast_battlegrowth_at_the_only_creature(&mut game, 0);

    let entry = &game.battlefield[&chain];
    assert_eq!(entry.counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(entry.counter_count(CounterType::MinusOneMinusOne), 0);
    assert_eq!(
        annihilations(&game),
        vec![(chain, 1), (chain, 1)],
        "one pair per check, twice — not one check removing two"
    );
    assert_eq!(
        get_effective_power(&game, chain),
        Some(3),
        "printed 3/3 with nothing left on it"
    );
}

/// A third Battlegrowth has nothing to annihilate against, and the counter
/// stays. The negative half of the rule, on the same board as the positive one.
#[test]
fn test_a_lone_plus_one_counter_survives_the_sweep() {
    let mut game = setup_two_player_game();

    let chain = reanimate(&mut game, chainbreaker(), 0);
    for _ in 0..3 {
        cast_battlegrowth_at_the_only_creature(&mut game, 0);
    }

    let entry = &game.battlefield[&chain];
    assert_eq!(
        entry.counter_count(CounterType::PlusOnePlusOne),
        1,
        "CR 704.5q removes pairs, and there is no -1/-1 left to pair with"
    );
    assert_eq!(annihilations(&game).len(), 2, "only the first two found a pair");
    assert_eq!(get_effective_power(&game, chain), Some(4));
}

/// Battlegrowth targets *any* creature, which is what makes the pairing work in
/// a random game: Chainbreaker is colorless and as likely to be across the
/// table as under it.
#[test]
fn test_battlegrowth_reaches_an_opponents_chainbreaker() {
    let mut game = setup_two_player_game();

    let chain = reanimate(&mut game, chainbreaker(), 1);
    cast_battlegrowth_at_the_only_creature(&mut game, 0);

    assert_eq!(annihilations(&game), vec![(chain, 1)]);
    assert_eq!(
        game.battlefield[&chain].counter_count(CounterType::MinusOneMinusOne),
        1
    );
}

/// A creature with only +1/+1 counters is not CR 704.5q's condition, however
/// many it has. Adaptive Shimmerer enters with three, and none of them moves.
#[test]
fn test_one_sign_alone_never_annihilates() {
    let mut game = setup_two_player_game();

    let shimmerer = reanimate(&mut game, adaptive_shimmerer(), 0);
    // A second creature, so the board is not a special case of one.
    reanimate(&mut game, vanilla_creature(2, 2, &[]), 1);
    game.check_state_based_actions(&test_dp()).unwrap();

    assert_eq!(
        game.battlefield[&shimmerer].counter_count(CounterType::PlusOnePlusOne),
        3
    );
    assert!(annihilations(&game).is_empty());
    assert_eq!(
        get_effective_toughness(&game, shimmerer),
        Some(3),
        "a 0/0 alive only on its counters — CR 704.5f would take it otherwise"
    );
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// Both halves of CR 704.5q are in the pool `fuzz_games --pool stress` plays.
///
/// This is the assertion §3.3 actually asks for — "when a rule needs two
/// objects, check that two *registered* cards can produce them". Adaptive
/// Shimmerer is here because its own doc comment claimed it "grows the stress
/// pool" while `registry.rs` never registered it, which is the same class of
/// silent gap one level down.
#[test]
fn test_both_counter_signs_are_registered() {
    let registry = CardRegistry::default_registry();
    for name in ["Battlegrowth", "Chainbreaker", "Adaptive Shimmerer"] {
        assert!(
            registry.create(name).is_ok(),
            "{name} is not registered, so no fuzz game can draw it"
        );
    }
}
