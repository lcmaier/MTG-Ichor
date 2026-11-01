//! One static ability, effects of several kinds (CR 113.2c): a paragraph is
//! one ability, and one ability can make a layer change beside a "can't"
//! (Aether Tunnel's "gets +1/+0 and can't be blocked"), or two replacement
//! effects, or each under its own "as long as" (Dog Umbra). Every reader of
//! a static body takes its own kind from the list, so none is dropped.
//!
//! The fixtures write the list as the enum, as Platinum Angel's two
//! "can't"s do.

use mtgsim::cards::authoring::{as_long_as, gains_keyword, gets, this_object, StaticBody};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::objects::card_data::{AbilityDef, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_power, get_effective_toughness, has_keyword};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    creature_with_ability, place_forest, put_in_hand, put_on_battlefield, setup_game, test_dp, textless_static, vanilla_creature,
};
use mtgsim::types::card_types::{CardType, LandType, Subtype};
use mtgsim::types::effects::{
    AffectedSet, AmountExpr, Condition, Duration, Effect, EffectRecipient, ObjectFilter, PlayerFact, PlayerRef, PlayerSet, Primitive,
};
use mtgsim::types::ids::PlayerId;
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite};
use mtgsim::types::restriction::{Restriction, RestrictionDef};
use std::sync::Arc;

/// "You can't gain life."
fn cant_gain_life() -> Effect {
    Effect::Restriction(Box::new(RestrictionDef::new(Restriction::Event {
        pattern: EventPattern::GainLife,
        affected: AffectedSet::players(PlayerSet::You),
        by: None,
    })))
}

/// "If you would gain life, you gain twice that much life instead."
fn twice_the_life() -> Effect {
    Effect::Replacement(Box::new(ReplacementDef::new(EventPattern::GainLife, PlayerSet::You, Rewrite::Amount(AmountRewrite::Multiplier(2)))))
}

/// "As long as you control a Forest".
fn you_control_a_forest() -> Condition {
    Condition::Player {
        players: PlayerSet::You,
        fact: PlayerFact::ControlsPermanent(ObjectFilter::BySubtype(Subtype::Land(LandType::Forest))),
    }
}

/// A 2/2 whose one static ability is `members`.
fn two_two_with(members: Vec<Effect>) -> Arc<CardData> {
    creature_with_ability("Fixture Bear", 2, 2, textless_static(Effect::Sequence(members)))
}

/// "You gain `n` life", resolved as a spell would for `player`; how much
/// their life changed.
fn life_changed_by_gaining(game: &mut GameState, player: PlayerId, n: u64) -> i64 {
    let before = game.players[player].life_total;
    let source = put_in_hand(game, CardDataBuilder::new("Fixture").build(), player);
    let ctx = ResolutionContext::untargeted(source, player, mtgsim::events::cause::GeneratedBy::Spell);
    let effect = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(n)), EffectRecipient::Controller);
    game.resolve_effect(&effect, &ctx, &test_dp()).expect("resolving");
    game.players[player].life_total - before
}

/// A layer change and a "can't" in one ability: the lowering takes the
/// change and the prohibition gate the "can't".
#[test]
fn a_layer_change_and_a_cant_in_one_ability_both_apply() {
    let mut game = setup_game(2);
    let bear = put_on_battlefield(&mut game, two_two_with(vec![gets(this_object(), 1, 0).into_static_body(), cant_gain_life()]), 0);

    assert_eq!(get_effective_power(&game, bear), Some(3), "the layer change applies");
    assert_eq!(life_changed_by_gaining(&mut game, 0, 3), 0, "and the \"can't\" refuses the gain (CR 101.2)");
}

/// A layer change and a replacement effect in one ability: the gather takes
/// the replacement beside the row.
#[test]
fn a_layer_change_and_a_replacement_in_one_ability_both_apply() {
    let mut game = setup_game(2);
    let bear = put_on_battlefield(&mut game, two_two_with(vec![gets(this_object(), 1, 0).into_static_body(), twice_the_life()]), 0);

    assert_eq!(get_effective_power(&game, bear), Some(3));
    assert_eq!(life_changed_by_gaining(&mut game, 0, 3), 6, "the replacement doubles the gain (CR 614.1a)");
}

/// CR 614.5 — two replacement effects of one ability are two effects, each
/// with its one opportunity: a gain of 3 is doubled twice. Keyed by the
/// ability alone, the second would read as applied already, and double once.
// COVERS-PARTIAL: ATOM-614.5-001
#[test]
fn two_replacement_effects_of_one_ability_each_apply_once() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, two_two_with(vec![twice_the_life(), twice_the_life()]), 0);

    // Scripted with nothing, so a CR 616.1 prompt would fail the resolution:
    // two multipliers commute, and there is nothing to ask.
    assert_eq!(life_changed_by_gaining(&mut game, 0, 3), 12);
}

/// CR 604.2 — Dog Umbra's shape: "As long as …, [a can't]. Otherwise, [a
/// replacement]." Each member exists under its own clause, so exactly one
/// applies at a time.
#[test]
fn each_member_of_one_ability_exists_under_its_own_as_long_as() {
    let mut game = setup_game(2);
    put_on_battlefield(
        &mut game,
        two_two_with(vec![
            Effect::Conditional(you_control_a_forest(), Box::new(cant_gain_life())),
            Effect::Conditional(Condition::Not(Box::new(you_control_a_forest())), Box::new(twice_the_life())),
        ]),
        0,
    );

    assert_eq!(life_changed_by_gaining(&mut game, 0, 3), 6, "no Forest: the replacement exists, the \"can't\" does not");
    place_forest(&mut game, 0);
    assert_eq!(life_changed_by_gaining(&mut game, 0, 3), 0, "a Forest: the \"can't\" exists, the replacement does not");
}

/// CR 604.2 — Clutch of Undeath's shape, "gets +3/+3 as long as …. Otherwise,
/// it gets …": two layer changes of one ability, each under its own clause,
/// which the board flips with no zone change. Each is an effect of its own,
/// so the one that applies does not carry the set it started with to the
/// other (CR 613.6): sharing one, a Forest would give both.
#[test]
fn each_layer_change_of_one_ability_exists_under_its_own_as_long_as() {
    let mut game = setup_game(2);
    let bear = put_on_battlefield(
        &mut game,
        two_two_with(vec![
            as_long_as(you_control_a_forest(), gets(this_object(), 3, 3)),
            as_long_as(Condition::Not(Box::new(you_control_a_forest())), gets(this_object(), 1, 0)),
        ]),
        0,
    );

    assert_eq!((get_effective_power(&game, bear), get_effective_toughness(&game, bear)), (Some(3), Some(2)), "no Forest: the second change alone");
    place_forest(&mut game, 0);
    assert_eq!((get_effective_power(&game, bear), get_effective_toughness(&game, bear)), (Some(5), Some(5)), "a Forest: the first alone");
}

/// Tenza, Godo's Maul's shape: an unconditional change, then a member under
/// its own clause, each in its own layer, in one ability.
#[test]
fn a_layer_change_beside_one_under_its_own_as_long_as_each_apply_as_their_clause_says() {
    let mut game = setup_game(2);
    let bear = put_on_battlefield(
        &mut game,
        two_two_with(vec![
            gets(this_object(), 1, 1).into_static_body(),
            as_long_as(you_control_a_forest(), gains_keyword(this_object(), KeywordFlag::Trample)),
        ]),
        0,
    );

    assert_eq!(get_effective_power(&game, bear), Some(3));
    assert!(!has_keyword(&game, bear, KeywordFlag::Trample), "no Forest: no trample");
    place_forest(&mut game, 0);
    assert_eq!(get_effective_power(&game, bear), Some(3), "the unconditional change is untouched");
    assert!(has_keyword(&game, bear, KeywordFlag::Trample), "a Forest: trample");
}

/// The prohibition gate's granted leg: a Layer 6 grant of an ability with a
/// layer change and a "can't" lights the registry summary's flag that sends
/// the sweep to the creature, and the sweep finds the "can't" in the list.
/// The granted layer change is not asked about: a static ability that grants
/// a static ability over a filter registers no layer rows yet.
#[test]
fn a_granted_ability_with_several_kinds_is_found_by_the_gate_it_carries() {
    let mut game = setup_game(2);
    let granted: AbilityDef = textless_static(Effect::Sequence(vec![gets(this_object(), 1, 0).into_static_body(), cant_gain_life()]));
    let grantor = textless_static(Effect::Atom(
        Primitive::GrantAbility(Arc::new(granted), Duration::WhileSourceOnBattlefield),
        EffectRecipient::FilteredPermanents(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Creature)),
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
        )),
    ));
    put_on_battlefield(&mut game, CardDataBuilder::new("Fixture Grantor").card_type(CardType::Enchantment).ability(grantor).build(), 0);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    assert_eq!(life_changed_by_gaining(&mut game, 0, 3), 0, "the granted \"can't\" is found beside the layer change");
}
