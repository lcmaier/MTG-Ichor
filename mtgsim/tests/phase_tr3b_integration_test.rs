//! TR-3b — the returns, CR 610.3's "until", and the objects a delayed
//! trigger names (`triggers-architecture.md`, TR-3b).
//!
//! 1. `codebase-state.md` item 223's row half: a resolution's registry rows
//!    name the object that has the ability, and only the rows that end with
//!    their source end as it leaves the battlefield (CR 611.2a, 611.2b,
//!    611.3b).

use std::sync::Arc;

use mtgsim::cards::authoring::{enters, triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_controller, get_effective_power};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_on_battlefield, setup_two_player_game, test_ctx, test_dp};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{AbilityId, ObjectId, PlayerId};
use mtgsim::types::triggers::TriggerSubject;
use mtgsim::types::zones::{Zone, ZoneChangeCause};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Activate `source`'s first ability for `player` and resolve it.
fn activate_and_resolve(game: &mut GameState, player: PlayerId, source: ObjectId) {
    game.activate_ability(player, source, 0, &test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
}

/// An activated ability with no cost but `{T}`.
fn tap_ability(rules_text: &'static str, effect: Effect) -> AbilityDef {
    AbilityDef {
        rules_text: rules_text.into(),
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Activated,
        costs: vec![Cost::TapSelf],
        effect,
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    }
}

fn sacrifice(game: &mut GameState, id: ObjectId) {
    game.change_zone(id, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx()).unwrap();
}

/// "{T}: Target creature gets +2/+2 until end of turn." An artifact, so its
/// ability can be activated the turn it arrives.
fn pump_totem() -> Arc<CardData> {
    CardDataBuilder::new("Pump Totem")
        .card_type(CardType::Artifact)
        .ability(tap_ability(
            "{T}: Target creature gets +2/+2 until end of turn.",
            Effect::Atom(
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(2), AmountExpr::Fixed(2), Duration::UntilEndOfTurn),
                EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            ),
        ))
        .build()
}

/// Sower of Temptation's shape: "When this creature enters, gain control of
/// target creature an opponent controls for as long as this creature remains
/// on the battlefield."
fn tempter() -> Arc<CardData> {
    CardDataBuilder::new("Tempter")
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Faerie))
        .power_toughness(2, 2)
        .ability(triggered_ability(
            "When this creature enters, gain control of target creature an opponent controls for as long as this creature remains on the battlefield.",
            whenever(
                enters(TriggerSubject::ThisObject),
                Effect::Atom(
                    Primitive::GainControl(Duration::WhileSourceOnBattlefield),
                    EffectRecipient::Target(
                        SelectionFilter::Permanent(ObjectFilter::And(
                            Box::new(ObjectFilter::ByType(CardType::Creature)),
                            Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
                        )),
                        TargetCount::Exactly(1),
                    ),
                ),
            ),
        ))
        .build()
}

/// "{T}: Regenerate this creature." A creature put down by
/// `put_on_battlefield`, which is never summoning-sick.
fn regenerator() -> Arc<CardData> {
    CardDataBuilder::new("Regenerator")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(tap_ability(
            "{T}: Regenerate this creature.",
            Effect::Atom(Primitive::Regenerate, EffectRecipient::ThisObject),
        ))
        .build()
}

// ---------------------------------------------------------------------------
// 1. Item 223's rows
// ---------------------------------------------------------------------------

/// CR 611.2a: "gets +2/+2 until end of turn" lasts until end of turn,
/// whatever becomes of the object whose ability made it. The row names the
/// totem (CR 113.7a), so the sweep that ends a static ability's rows as its
/// source leaves must leave this one.
#[test]
fn a_pump_an_activated_ability_made_outlives_its_source() {
    let mut game = setup_two_player_game();
    let totem = put_on_battlefield(&mut game, pump_totem(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    activate_and_resolve(&mut game, 0, totem);
    assert_eq!(get_effective_power(&game, bears), Some(4));
    let sources: Vec<ObjectId> = game.continuous_effects.iter().map(|row| row.source).collect();
    assert_eq!(sources, vec![totem], "the pump's row names the totem, not the ability's stack object");

    sacrifice(&mut game, totem);
    assert_eq!(get_effective_power(&game, bears), Some(4), "CR 611.2a: the pump outlives its source");
}

/// CR 611.2b: "for as long as this creature remains on the battlefield"
/// ends as it leaves, and the stolen creature goes back to its controller.
#[test]
fn a_for_as_long_as_row_a_resolution_made_ends_as_its_source_leaves() {
    let mut game = setup_two_player_game();
    let stolen = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let tempter = put_on_battlefield(&mut game, tempter(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(get_effective_controller(&game, stolen), Some(0));

    sacrifice(&mut game, tempter);
    assert_eq!(get_effective_controller(&game, stolen), Some(1), "CR 611.2b: the duration ended with its source");
}

/// Item 95: the shield "{T}: Regenerate this creature" makes is the
/// creature's (CR 113.7a), and its row is what CR 616.1's prompt names
/// (`replacement::gather` reads the row's source).
#[test]
fn a_regeneration_shield_from_an_activated_ability_names_the_permanent() {
    let mut game = setup_two_player_game();
    let regenerator = put_on_battlefield(&mut game, regenerator(), 0);
    activate_and_resolve(&mut game, 0, regenerator);
    let sources: Vec<ObjectId> = game.replacement_effects.iter().map(|row| row.source).collect();
    assert_eq!(sources, vec![regenerator]);
}
