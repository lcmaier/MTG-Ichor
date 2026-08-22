//! Regression tests: engine queries must read *effective* characteristics,
//! not printed ones.
//!
//! Before Layer 4 landed, printed types and effective types were always equal,
//! so ~21 call sites read `obj.card_data.{types,subtypes,supertypes,colors}`
//! directly with no consequence. Layer 4 broke that invariant: an artifact
//! animated by Ensoul Artifact *is* a creature, but every direct reader still
//! saw a non-creature artifact.
//!
//! No existing test caught this because none crossed the layer system and the
//! consumers of it in the same scenario. These do.


use mtgsim::engine::layers::{AffectedSet, ContinuousEffect, EffectModification, EffectOrigin, Layer};
use mtgsim::objects::card_data::CardDataBuilder;
use mtgsim::oracle::characteristics::is_creature;
use mtgsim::types::card_types::{CardType, Supertype};
use mtgsim::types::effects::{
    CounterType, Duration, EffectRecipient, PermanentFilter, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::ObjectId;
use mtgsim::engine::resolve::ResolvedTarget;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

use mtgsim::test_support::{put_on_battlefield, setup_two_player_game};
use mtgsim::state::game_state::GameState;

/// Register a Layer 4 effect on a single object.
fn add_layer4(game: &mut GameState, id: ObjectId, modification: EffectModification) {
    let timestamp = game.allocate_timestamp();
    let turn = game.turn_number;
    game.continuous_effects.add(ContinuousEffect {
        id: 0,
        source: id,
        origin: EffectOrigin::Resolution,
        layer: Layer::Layer4Type,
        duration: Duration::UntilEndOfTurn,
        controller: 0,
        created_on_turn: turn,
        timestamp,
        affected: AffectedSet::Fixed(vec![id]),
        modification,
    });
}

// ---------------------------------------------------------------------------
// Targeting (engine/targeting.rs)
// ---------------------------------------------------------------------------

/// An animated artifact must be a legal "target creature".
/// This is the headline bug: `validate_creature_target` read printed types.
#[test]
fn test_animated_artifact_is_a_legal_creature_target() {
    let mut game = setup_two_player_game();
    let artifact_id = put_on_battlefield(
        &mut game,
        CardDataBuilder::new("Darksteel Ingot")
            .card_type(CardType::Artifact)
            .build(),
        0,
    );

    let recipient =
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1));
    let targets = vec![ResolvedTarget::Object(artifact_id)];

    // Before: not a creature, so targeting must fail.
    assert!(!is_creature(&game, artifact_id));
    assert!(game.validate_targets(&recipient, &targets).is_err());

    // Ensoul Artifact: "target artifact you control becomes a 5/5 creature"
    add_layer4(&mut game, artifact_id, EffectModification::AddType(CardType::Creature));

    assert!(is_creature(&game, artifact_id));
    assert!(
        game.validate_targets(&recipient, &targets).is_ok(),
        "an animated artifact is a creature and must be targetable as one"
    );
}

/// A creature that loses its creature type must stop being a legal target.
#[test]
fn test_detyped_creature_is_not_a_legal_creature_target() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(
        &mut game,
        CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build(),
        0,
    );

    let recipient =
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1));
    let targets = vec![ResolvedTarget::Object(bears_id)];
    assert!(game.validate_targets(&recipient, &targets).is_ok());

    add_layer4(
        &mut game,
        bears_id,
        EffectModification::RemoveType(CardType::Creature),
    );

    assert!(!is_creature(&game, bears_id));
    assert!(
        game.validate_targets(&recipient, &targets).is_err(),
        "a permanent that lost the creature type is not a legal creature target"
    );
}

/// `PermanentFilter::ByType` is used by every filtered effect; it must see
/// Layer 4 output too.
#[test]
fn test_permanent_filter_by_type_sees_layer4() {
    let mut game = setup_two_player_game();
    let artifact_id = put_on_battlefield(
        &mut game,
        CardDataBuilder::new("Darksteel Ingot")
            .card_type(CardType::Artifact)
            .build(),
        0,
    );

    let recipient = EffectRecipient::Target(
        SelectionFilter::Permanent(PermanentFilter::ByType(CardType::Creature)),
        TargetCount::Exactly(1),
    );
    let targets = vec![ResolvedTarget::Object(artifact_id)];
    assert!(game.validate_targets(&recipient, &targets).is_err());

    add_layer4(&mut game, artifact_id, EffectModification::AddType(CardType::Creature));

    assert!(game.validate_targets(&recipient, &targets).is_ok());
}

// ---------------------------------------------------------------------------
// State-based actions (engine/sba.rs)
// ---------------------------------------------------------------------------

/// CR 704.5j: the legend rule keys on the *effective* Legendary supertype, so
/// a permanent made legendary by an effect is subject to it.
// COVERS-PARTIAL: ATOM-205.4d-001, ATOM-704.5j-001
#[test]
fn test_legend_rule_sees_granted_legendary_supertype() {
    let mut game = setup_two_player_game();
    let make = |n: u8| {
        CardDataBuilder::new("Ornithopter")
            .card_type(CardType::Artifact)
            .card_type(CardType::Creature)
            .power_toughness(n as i32, 2)
            .build()
    };
    let a = put_on_battlefield(&mut game, make(0), 0);
    let b = put_on_battlefield(&mut game, make(0), 0);

    // Two nonlegendary permanents with the same name coexist fine.
    game.check_state_based_actions_loop(&ScriptedDecisionProvider::new()).unwrap();
    assert!(game.battlefield.contains_key(&a));
    assert!(game.battlefield.contains_key(&b));

    // On Serra's Wings style: both become legendary.
    add_layer4(&mut game, a, EffectModification::AddSupertype(Supertype::Legendary));
    add_layer4(&mut game, b, EffectModification::AddSupertype(Supertype::Legendary));

    // The legend rule now fires, so the controller must choose which to keep.
    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(
        ChoiceKind::LegendRule { legend_name: "Ornithopter".to_string() },
        vec![0],
    );
    game.check_state_based_actions_loop(&decisions).unwrap();
    let survivors = [a, b].iter().filter(|id| game.battlefield.contains_key(id)).count();
    assert_eq!(
        survivors, 1,
        "the legend rule must fire on a granted Legendary supertype"
    );
}

/// CR 704.5i: a permanent that becomes a planeswalker with no loyalty counters
/// is put into its owner's graveyard.
// COVERS-PARTIAL: ATOM-704.5i-001
#[test]
fn test_zero_loyalty_sba_sees_granted_planeswalker_type() {
    let mut game = setup_two_player_game();
    let artifact_id = put_on_battlefield(
        &mut game,
        CardDataBuilder::new("Darksteel Ingot")
            .card_type(CardType::Artifact)
            .build(),
        0,
    );

    game.check_state_based_actions_loop(&ScriptedDecisionProvider::new()).unwrap();
    assert!(game.battlefield.contains_key(&artifact_id));

    add_layer4(
        &mut game,
        artifact_id,
        EffectModification::AddType(CardType::Planeswalker),
    );
    assert_eq!(
        game.battlefield
            .get(&artifact_id)
            .unwrap()
            .counter_count(CounterType::Loyalty),
        0
    );

    game.check_state_based_actions_loop(&ScriptedDecisionProvider::new()).unwrap();
    assert!(
        !game.battlefield.contains_key(&artifact_id),
        "a planeswalker with 0 loyalty is put into its owner's graveyard (704.5i)"
    );
}
