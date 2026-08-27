//! Phase RB integration tests: the CR 616.1 pipeline and its first consumers.
//!
//! RA proved the *record* was complete. RB is the first phase where a
//! replacement effect actually changes what happens, so these tests assert on
//! the modified outcome **and** on the pipeline having been entered — a
//! replacement test that passes because the effect never fired is the exact
//! failure this phase is prone to (`replacement-architecture.md` §10).

use std::sync::Arc;

use mtgsim::engine::actions::{DestructionSource, GameAction, ZoneChangeCause};
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::engine::layers::types::{EffectModification, Layer};
use mtgsim::test_support::{
    place_bare, put_on_battlefield, registered, setup_two_player_game, test_ctx, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{AffectedSet, Effect};
use mtgsim::types::ids::{new_ability_id, ObjectId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::replacement::{
    EventPattern, ReplacementDef, Rewrite,
};
use mtgsim::types::zones::Zone;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Every zone change in the log, as `(object, from, to, cause)`.
fn zone_changes(game: &GameState) -> Vec<(ObjectId, Zone, Zone, ZoneChangeCause)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::ZoneChange { object_id, from, to, cause, .. } => {
                Some((*object_id, *from, *to, *cause))
            }
            _ => None,
        })
        .collect()
}

/// A creature whose printed static ability is a bare `Prevent` on destruction.
///
/// Not a real card — a probe. It is the smallest thing that proves a static
/// `Effect::Replacement` is discovered off the *effective* ability list and
/// that `register_static_effects` lets it through without trying to lower it
/// into a layer row.
fn undying_probe(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::Destroy { source: None },
                AffectedSet::SourceOnly,
                Rewrite::Prevent,
            ))),
        })
        .build()
}

// ---------------------------------------------------------------------------
// Item 4 — `GameAction::Destroy` as the outer event
// ---------------------------------------------------------------------------

// COVERS: ATOM-701.8b-001
#[test]
fn test_destroy_is_an_outer_event_that_lowers_to_a_zone_change() {
    // CR 701.8a — "to destroy a permanent, move it from the battlefield to its
    // owner's graveyard". Two events, not one, and the cause names which of
    // CR 701.8b's routes it was.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let source = place_bare(&mut game, vanilla_creature(1, 1, &[]), 0);

    game.execute_action(
        GameAction::Destroy { object: bear, source: DestructionSource::Effect(source) },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Graveyard);
    assert_eq!(
        zone_changes(&game),
        vec![(bear, Zone::Battlefield, Zone::Graveyard, ZoneChangeCause::Destroyed)],
    );
}

// COVERS-PARTIAL: ATOM-701.8b-001
#[test]
fn test_a_state_based_death_and_an_effect_death_carry_different_causes() {
    // CR 701.8b calls both "destroyed", and `DestructionSource` is what lets a
    // shield counter ("as the result of an effect") tell them apart while a
    // "dies" trigger does not have to.
    let mut game = setup_two_player_game();
    let by_effect = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let by_sba = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let source = place_bare(&mut game, vanilla_creature(1, 1, &[]), 0);

    game.execute_action(
        GameAction::Destroy { object: by_effect, source: DestructionSource::Effect(source) },
        &test_ctx(),
    )
    .unwrap();
    game.execute_action(
        GameAction::Destroy { object: by_sba, source: DestructionSource::StateBasedAction },
        &test_ctx(),
    )
    .unwrap();

    let causes: Vec<ZoneChangeCause> = zone_changes(&game).into_iter().map(|z| z.3).collect();
    assert_eq!(causes, vec![ZoneChangeCause::Destroyed, ZoneChangeCause::DestroyedBySba]);
}

#[test]
fn test_destroy_is_loud_off_the_battlefield() {
    // Performers are loud; callers check legality (CR 608.2b). Both production
    // callers filter their targets, so a lenient arm here would only be
    // somewhere for a future bug to hide.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.battlefield.remove(&bear);

    let err = game.execute_action(
        GameAction::Destroy { object: bear, source: DestructionSource::StateBasedAction },
        &test_ctx(),
    );
    assert!(err.is_err());
    assert!(zone_changes(&game).is_empty());
}

// COVERS: ATOM-614.17-001
#[test]
fn test_indestructible_is_a_cant_ahead_of_the_pipeline_not_a_replacement() {
    // CR 614.17 — "some effects state that something can't happen. These
    // effects aren't replacement effects, but follow similar rules." CR 701.8a
    // makes indestructible one of them, so the destruction is *proposed* and
    // then blocked, rather than being filtered before it exists.
    //
    // The observable difference from the pre-RB filter is that the event now
    // reaches the pipeline at all, which is what CR 614.17c's self-replacement
    // exception will need.
    let mut game = setup_two_player_game();
    let myr = place_bare(
        &mut game,
        vanilla_creature(0, 1, &[KeywordFlag::Indestructible]),
        0,
    );
    let source = place_bare(&mut game, vanilla_creature(1, 1, &[]), 0);

    game.execute_action(
        GameAction::Destroy { object: myr, source: DestructionSource::Effect(source) },
        &test_ctx(),
    )
    .unwrap();

    assert!(game.battlefield.contains_key(&myr), "indestructible: it can't be destroyed");
    assert!(zone_changes(&game).is_empty(), "the inner zone change never happened");
}

#[test]
fn test_the_sba_sweep_does_not_spin_on_an_indestructible_creature() {
    // CR 704.3 repeats the check only "if any state-based actions are
    // performed". An indestructible creature with lethal damage marked meets
    // 704.5g's condition every single check, so a sweep that counted the
    // *proposal* rather than the performance would re-check forever. This is
    // the customer that earned `execute_actions` its return value back.
    let mut game = setup_two_player_game();
    let myr = place_bare(
        &mut game,
        vanilla_creature(0, 1, &[KeywordFlag::Indestructible]),
        0,
    );
    game.battlefield.get_mut(&myr).unwrap().damage_marked = 5;

    let dp = mtgsim::ui::decision::ScriptedDecisionProvider::new();
    let performed = game.check_state_based_actions(&dp).unwrap();

    assert!(!performed, "nothing was performed, so 704.3 must not re-check");
    assert!(game.battlefield.contains_key(&myr));
}

// ---------------------------------------------------------------------------
// Item 3 — the pipeline, exercised through a printed static ability
// ---------------------------------------------------------------------------

// COVERS: ATOM-614.6-001
#[test]
fn test_a_static_replacement_ability_prevents_the_event() {
    // CR 614.6 — "the modified event occurs instead of the original event", and
    // a `Prevent` leaves nothing to occur.
    let mut game = setup_two_player_game();
    let probe = put_on_battlefield(&mut game, undying_probe("Undying Probe"), 0);
    let source = place_bare(&mut game, vanilla_creature(1, 1, &[]), 0);

    game.execute_action(
        GameAction::Destroy { object: probe, source: DestructionSource::Effect(source) },
        &test_ctx(),
    )
    .unwrap();

    assert!(game.battlefield.contains_key(&probe));
    assert!(zone_changes(&game).is_empty());
}

// COVERS: ATOM-614.4-001
#[test]
fn test_humility_strips_a_replacement_ability_and_the_creature_dies() {
    // Two rules at once, and they are the reason source 1 of §3.3 is a sweep of
    // *effective* ability lists rather than a registry scan.
    //
    // CR 614.4 — "a replacement effect ... must exist before the appropriate
    // event occurs" — is asked at the instant the event is proposed, so an
    // ability Layer 6 has taken away is not there to ask about. And the
    // fast-path hint set still names this creature, which is the point: it
    // over-approximates in the safe direction, costing a layer walk rather than
    // producing a wrong answer.
    let mut game = setup_two_player_game();
    let probe = put_on_battlefield(&mut game, undying_probe("Undying Probe"), 0);
    let source = place_bare(&mut game, vanilla_creature(1, 1, &[]), 0);

    let ts = game.allocate_timestamp();
    game.continuous_effects.add(registered(
        probe,
        Layer::Layer6Ability,
        ts,
        EffectModification::LoseAllAbilities,
    ));

    assert!(
        game.replacement_ability_sources.contains(&probe),
        "the hint is stale-positive here, and the gate has to survive that"
    );

    game.execute_action(
        GameAction::Destroy { object: probe, source: DestructionSource::Effect(source) },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(
        game.get_object(probe).unwrap().zone,
        Zone::Graveyard,
        "the shield was gone before the event, so nothing replaced it"
    );
}

#[test]
fn test_a_static_replacement_body_registers_no_layer_rows() {
    // CR 614.1a's replacement effects are not continuous effects.
    // `register_static_effects` has to let the body through without trying to
    // lower it — and without tripping its own loud "lowers to no layer rows"
    // assert, which is what a `_ =>` arm would have done in a debug build.
    let mut game = setup_two_player_game();
    let before = game.continuous_effects.len();
    let probe = put_on_battlefield(&mut game, undying_probe("Undying Probe"), 0);

    assert_eq!(game.continuous_effects.len(), before, "no rows");
    assert!(
        game.replacement_ability_sources.contains(&probe),
        "but it is recorded as a gather candidate, or the fast path would skip it"
    );
}

#[test]
fn test_leaving_the_battlefield_retires_the_gather_hint() {
    let mut game = setup_two_player_game();
    let probe = put_on_battlefield(&mut game, undying_probe("Undying Probe"), 0);
    assert!(game.replacement_ability_sources.contains(&probe));

    game.change_zone(probe, Zone::Exile, ZoneChangeCause::Exiled, &test_ctx()).unwrap();
    assert!(!game.replacement_ability_sources.contains(&probe));
}
