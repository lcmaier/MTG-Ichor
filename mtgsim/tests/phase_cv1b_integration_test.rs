//! Phase CV-1b — CR 400.7's new object, and copies that last indefinitely.
//!
//! `copy-effects-architecture.md` §5.3: a copy with no stated duration is
//! reachable by neither CR 514.2's expiry nor `remove_by_source`, so it is
//! safe only once a move ends every reference to the object that moved
//! (`codebase-state.md` item 10). The rule comes first, then the copies that
//! lean on it.
//!
//! **No registered card returns an object yet**, so the rule's tests move
//! objects by hand through `change_zone`, the door a return will use: out and
//! straight back, which is the board every reference kind has to survive.
//!
//! 1. The registries' half: a row that named the mover stops naming it, and
//!    CR 400.7a and 400.7c's exceptions for a permanent spell.

use std::sync::Arc;

use mtgsim::cards::alpha::giant_growth;
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_cv_cards::mirrorweave;
use mtgsim::cards::phase_rd_cards::{circle_of_protection_red, mending_hands};
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, EffectOrigin, Layer};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::{CounterSubject, DamageTarget};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{
    get_effective_colors, get_effective_name, get_effective_power, get_effective_toughness,
    has_summoning_sickness,
};
use mtgsim::state::game_state::GameState;
use mtgsim::state::replacement_effects::RegisteredReplacementEffect;
use mtgsim::state::restrictions::RegisteredRestriction;
use mtgsim::test_support::{
    pacifism, put_in_hand, put_on_battlefield, put_spell_on_stack, setup_two_player_game, test_ctx,
    test_dp, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{CounterType, Duration, ObjectSet, PlayerSet};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{EventPattern, ReplacementDef, Rewrite};
use mtgsim::types::restriction::{Restriction, RestrictionDef};
use mtgsim::types::zones::{DestructionSource, Zone, ZoneChangeCause};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve `card`'s spell effect for `controller` with `targets`, as the stack
/// would once CR 608.2b has kept them.
fn resolve_spell(game: &mut GameState, card: Arc<CardData>, controller: PlayerId, targets: &[ObjectId]) -> ObjectId {
    let id = put_in_hand(game, card.clone(), controller);
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller,
        targets: ChosenTargets::one(targets.iter().copied().map(ResolvedTarget::Object).collect()),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
    id
}

/// `id` leaves the battlefield for `to` and comes straight back: the shape a
/// blink or a reanimation makes, which no registered card makes yet.
fn leave_and_return(game: &mut GameState, id: ObjectId, to: Zone, cause: ZoneChangeCause) {
    game.change_zone(id, to, cause, &test_ctx()).unwrap();
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    assert_eq!(game.get_object(id).unwrap().zone, Zone::Battlefield, "the object came back");
}

fn pt(game: &GameState, id: ObjectId) -> (Option<i32>, Option<i32>) {
    (get_effective_power(game, id), get_effective_toughness(game, id))
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

fn deal_damage(game: &mut GameState, source: ObjectId, target: DamageTarget, amount: u64) {
    game.execute_action(
        GameAction::DealDamage { source, target, amount, is_combat: false, unpreventable: false },
        &test_ctx(),
    )
    .unwrap();
}

/// A red creature, for CR 609.7a's chosen source.
fn red_creature(power: i32) -> Arc<CardData> {
    CardDataBuilder::new("Red Probe")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .power_toughness(power, power)
        .build()
}

/// A resolution's row registered by hand, for a shape no registered card
/// makes: an effect on a spell on the stack, or a set with a player half.
fn resolution_row(game: &mut GameState, layer: Layer, affected: ObjectSet, modification: EffectModification) {
    let timestamp = game.allocate_timestamp();
    let turn = game.turn_number;
    game.continuous_effects.add(ContinuousEffect {
        id: 0,
        source: mtgsim::types::ids::new_object_id(),
        origin: EffectOrigin::Resolution,
        layer,
        duration: Duration::UntilEndOfTurn,
        controller: 0,
        created_on_turn: turn,
        timestamp,
        affected_objects: affected,
        modification,
    });
}

fn replacement_row(game: &mut GameState, def: ReplacementDef) {
    let turn = game.turn_number;
    game.replacement_effects.add(RegisteredReplacementEffect {
        id: 0,
        source: mtgsim::types::ids::new_object_id(),
        controller: 0,
        duration: Duration::UntilEndOfTurn,
        created_on_turn: turn,
        targets: Vec::new(),
        def,
    });
}

/// "`object` can't be destroyed" from a resolution.
fn cant_be_destroyed(game: &mut GameState, object: ObjectId) {
    let turn = game.turn_number;
    game.restrictions.add(RegisteredRestriction {
        id: 0,
        source: mtgsim::types::ids::new_object_id(),
        controller: 0,
        duration: Duration::UntilEndOfTurn,
        created_on_turn: turn,
        def: RestrictionDef::new(Restriction::Event {
            pattern: EventPattern::Destroy { source: None },
            affected_objects: ObjectSet::Fixed(vec![object]),
            affected_players: PlayerSet::Nobody,
            by: None,
        }),
    });
}

fn destroy(game: &mut GameState, object: ObjectId) {
    let by = mtgsim::types::ids::new_object_id();
    game.execute_action(GameAction::Destroy { object, source: DestructionSource::Effect(by) }, &test_ctx())
        .unwrap();
}

// ---------------------------------------------------------------------------
// 1. The registries' half
// ---------------------------------------------------------------------------

/// Item 10's Giant Growth board, as a blink: the pump, the counter and the
/// Aura were the old object's. The creature that comes back has none of
/// them, and it has not been under its controller's control since the turn
/// began (CR 302.6). Partial: the return is by hand, not Long Road Home's.
// COVERS-PARTIAL: ATOM-400.7-002
#[test]
fn a_creature_exiled_and_returned_is_a_new_object() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    resolve_spell(&mut game, giant_growth(), 0, &[bears]);
    game.execute_action(
        GameAction::AddCounters { subject: CounterSubject::Object(bears), counter: CounterType::PlusOnePlusOne, n: 1, by: 0 },
        &test_ctx(),
    )
    .unwrap();
    let aura = put_on_battlefield(&mut game, pacifism(), 1);
    game.attach(aura, bears);
    assert_eq!(pt(&game, bears), (Some(6), Some(6)), "2/2, +3/+3 and a counter");

    leave_and_return(&mut game, bears, Zone::Exile, ZoneChangeCause::Exiled);

    assert_eq!(pt(&game, bears), (Some(2), Some(2)), "no pump, no counter");
    assert!(game.continuous_effects.is_empty(), "the pump's row named only the old object");
    assert_eq!(game.battlefield[&bears].counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(game.battlefield[&aura].attached_to, None, "the Aura is left for CR 704.5m");
    assert!(has_summoning_sickness(&game, bears));
}

/// One row about two objects keeps applying to the one that stayed: a
/// Mirrorweave copy on two creatures, one of which dies and comes back.
#[test]
fn a_copy_on_two_creatures_keeps_applying_to_the_one_that_stayed() {
    let mut game = setup_two_player_game();
    let angel = put_on_battlefield(&mut game, mtgsim::cards::keyword_creatures::serra_angel(), 1);
    let (a, b) = (put_on_battlefield(&mut game, grizzly_bears(), 0), put_on_battlefield(&mut game, grizzly_bears(), 0));
    resolve_spell(&mut game, mirrorweave(), 0, &[angel]);
    assert_eq!((get_effective_name(&game, a), get_effective_name(&game, b)), ("Serra Angel".into(), "Serra Angel".into()));

    leave_and_return(&mut game, a, Zone::Graveyard, ZoneChangeCause::Destroyed);

    assert_eq!(get_effective_name(&game, a), "Grizzly Bears", "a new object, copying nothing");
    assert_eq!(get_effective_name(&game, b), "Serra Angel");
    let rows: Vec<&ContinuousEffect> = game.continuous_effects.iter().collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].affected_objects, ObjectSet::Fixed(vec![b]));
}

/// CR 400.7a: an effect that changed a permanent spell's characteristics
/// continues to apply to the permanent it becomes. The color change is
/// registered by hand; no registered card targets a spell with one. And a
/// "can't" on the spell does not carry over: 400.7a–c name no restriction.
// COVERS-PARTIAL: ATOM-400.7a-001
#[test]
fn a_permanent_spell_keeps_its_color_change_and_loses_its_restriction() {
    let mut game = setup_two_player_game();
    let spell = put_spell_on_stack(&mut game, grizzly_bears(), 0);
    let black = std::collections::HashSet::from([Color::Black]);
    resolution_row(&mut game, Layer::Layer5Color, ObjectSet::Fixed(vec![spell]), EffectModification::SetColors(black.clone()));
    cant_be_destroyed(&mut game, spell);

    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.get_object(spell).unwrap().zone, Zone::Battlefield);
    assert_eq!(get_effective_colors(&game, spell), black, "CR 400.7a");
    assert!(game.restrictions.is_empty(), "no exception names a restriction");

    destroy(&mut game, spell);
    assert_eq!(game.get_object(spell).unwrap().zone, Zone::Graveyard);
}

/// CR 609.7a's chosen source is chosen once. Circle of Protection: Red's
/// shield watches the creature it chose; once that creature has died and
/// come back it is a new object, and the shield watches nothing.
#[test]
fn a_shield_against_a_chosen_source_ends_when_the_source_moves() {
    let mut game = setup_two_player_game();
    let circle = put_on_battlefield(&mut game, circle_of_protection_red(), 0);
    let red = put_on_battlefield(&mut game, red_creature(4), 1);
    // CR 609.7a's candidates are the battlefield in timestamp order: [circle, red].
    let dp = RecordingDecisionProvider::picking(1);
    let ctx = ResolutionContext {
        source: circle,
        ability_source: game.object_ref(circle),
        controller: 0,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&circle_of_protection_red().abilities[0].effect, &ctx, &dp).unwrap();
    assert_eq!(game.replacement_effects.len(), 1);

    leave_and_return(&mut game, red, Zone::Graveyard, ZoneChangeCause::Destroyed);
    assert!(game.replacement_effects.is_empty(), "the source it chose is gone");

    deal_damage(&mut game, red, DamageTarget::Player(0), 4);
    assert_eq!(life(&game, 0), 16, "the returned creature is not the chosen source");
}

/// A prevention shield on a creature is about that object: Mending Hands
/// prevents nothing for the creature that comes back.
#[test]
fn a_shield_on_a_creature_ends_when_it_moves() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let red = put_on_battlefield(&mut game, red_creature(3), 1);
    resolve_spell(&mut game, mending_hands(), 0, &[bears]);

    leave_and_return(&mut game, bears, Zone::Hand, ZoneChangeCause::Returned);
    assert!(game.replacement_effects.is_empty());

    deal_damage(&mut game, red, DamageTarget::Object(bears), 3);
    assert_eq!(game.battlefield[&bears].damage_marked, 3, "nothing prevented it");
}

/// "Prevent all damage that would be dealt to you and target creature": the
/// creature's half goes with it, and the row keeps protecting you.
#[test]
fn a_shield_for_you_and_a_creature_keeps_protecting_you() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let red = put_on_battlefield(&mut game, red_creature(3), 1);
    replacement_row(
        &mut game,
        ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            ObjectSet::Fixed(vec![bears]),
            Rewrite::Prevent,
        )
        .affecting_players(PlayerSet::You),
    );

    leave_and_return(&mut game, bears, Zone::Graveyard, ZoneChangeCause::Destroyed);
    let row = game.replacement_effects.iter().next().expect("the player half keeps the row");
    assert_eq!(row.def.affected_objects, ObjectSet::Fixed(Vec::new()));

    deal_damage(&mut game, red, DamageTarget::Object(bears), 3);
    deal_damage(&mut game, red, DamageTarget::Player(0), 3);
    assert_eq!(game.battlefield[&bears].damage_marked, 3, "the new creature is not protected");
    assert_eq!(life(&game, 0), 20, "you still are");
}

/// A "can't" on a creature is about that object too: once it has left and
/// come back, it can be destroyed.
#[test]
fn a_restriction_on_a_creature_ends_when_it_moves() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    cant_be_destroyed(&mut game, bears);
    destroy(&mut game, bears);
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield, "the restriction holds");

    leave_and_return(&mut game, bears, Zone::Hand, ZoneChangeCause::Returned);
    assert!(game.restrictions.is_empty());
    destroy(&mut game, bears);
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Graveyard);
}

/// The common move names no row and changes no registry: a draw.
#[test]
fn a_move_no_row_names_leaves_every_registry_alone() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    resolve_spell(&mut game, giant_growth(), 0, &[bears]);
    let card = mtgsim::test_support::put_in_library(&mut game, red_creature(1), 0);
    let before = game.continuous_effects.mutations();

    game.draw_card(0, &test_ctx()).unwrap();
    assert_eq!(game.get_object(card).unwrap().zone, Zone::Hand);
    assert_eq!(game.continuous_effects.mutations(), before);
    assert_eq!(pt(&game, bears), (Some(5), Some(5)));
}
