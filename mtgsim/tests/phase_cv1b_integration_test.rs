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
//! 2. The announcement's half: a target or a choice is remembered with the
//!    epoch it had when chosen, and CR 608.2b compares it. Cast from hand,
//!    since the cast path is what stamps it.
//! 3. Combat's: a creature that leaves the battlefield is removed from combat
//!    (CR 506.4), so no attacker or blocker it was paired with still names it.
//! 4. A re-copy: each copy row's abilities are their own instances, so the
//!    rows a copied static ability generates apply once, and only while their
//!    copy is the one showing (item 16b).
//! 5. Copies that last: Mirrorform states no duration, so its copies last
//!    until the end of the game (CR 611.2a) and end when their subject moves.
//! 6. "Another target": a targeting filter is other than the object the
//!    text's "this" names, at the offer, the announcement and the re-check.
//! 7. Cryptoplasm, the card: an upkeep's "may" copy of another target
//!    creature, with no duration, that keeps the ability that made it.
//! 8. Item 189: a copy a counter write makes applicable takes back the
//!    first copy's exception at the count a doubler left it, not as added.

use std::sync::Arc;

use mtgsim::cards::alpha::giant_growth;
use mtgsim::cards::authoring::{enters, triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_rc_cards::chainbreaker;
use mtgsim::cards::phase_cv_cards::{self, cryptoplasm, cytoshape, mirrorform, mirrorweave};
use mtgsim::cards::phase_rd_cards::{circle_of_protection_red, mending_hands};
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::combat::resolution::assign_combat_damage;
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, EffectOrigin, Layer};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::{CounterSubject, DamageTarget, GameEvent};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::{
    get_effective_colors, get_effective_name, get_effective_power, get_effective_toughness,
    has_summoning_sickness,
};
use mtgsim::state::game_state::GameState;
use mtgsim::state::replacement_effects::RegisteredReplacementEffect;
use mtgsim::state::restrictions::RegisteredRestriction;
use mtgsim::test_support::{
    lightning_bolt, pacifism, place_vanilla_creature, put_in_hand, put_on_battlefield, put_spell_on_stack,
    set_attacking, set_blocked_by, set_blocking, setup_two_player_game, test_ctx, test_dp,
    RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectSet, PlayerSet, Primitive, SelectionFilter,
    TargetCount,
};
use mtgsim::types::ids::{AbilityId, ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{EventPattern, ReplacementDef, Rewrite};
use mtgsim::types::restriction::{Restriction, RestrictionDef};
use mtgsim::state::game_state::StepType;
use mtgsim::types::triggers::TriggerSubject;
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::types::zones::{DestructionSource, Zone, ZoneChangeCause};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve `card`'s spell effect for `controller` with `targets`, as the stack
/// would once CR 608.2b has kept them.
fn resolve_spell(game: &mut GameState, card: Arc<CardData>, controller: PlayerId, targets: &[ObjectId]) -> ObjectId {
    // The spell object, in no zone's collection: `ResolutionContext.source`
    // is the stack object, and a card in a hand would be one more card there.
    let id = game.add_object(GameObject::new(card.clone(), controller, Zone::Stack));
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

/// Empty `player`'s pool, fill it with exactly `pool`, and cast `card` from
/// hand under `ManaWindowStop`, as a shipped client does.
fn cast_from_pool(
    game: &mut GameState,
    player: PlayerId,
    card: Arc<CardData>,
    pool: &[(ManaType, u64)],
    dp: impl DecisionProvider,
) -> ObjectId {
    let id = put_in_hand(game, card, player);
    for t in [ManaType::White, ManaType::Blue, ManaType::Black, ManaType::Red, ManaType::Green, ManaType::Colorless] {
        let have = game.players[player].mana_pool.amount(t);
        if have > 0 {
            game.players[player].mana_pool.remove(t, have).unwrap();
        }
    }
    for &(t, n) in pool {
        game.players[player].mana_pool.add(t, n);
    }
    game.cast_spell(player, id, &ManaWindowStop::new(dp)).expect("castable from exactly its cost");
    id
}

fn fizzled(game: &GameState, spell: ObjectId) -> bool {
    game.recorded_events().events().any(|e| matches!(e, GameEvent::SpellFizzled { spell_id } if *spell_id == spell))
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

// ---------------------------------------------------------------------------
// 2. The announcement's half
// ---------------------------------------------------------------------------

/// CR 608.2b: "a target that's no longer in the zone it was in when it was
/// targeted is illegal". Lightning Bolt cast at the Bears; the Bears die and
/// come back before it resolves. The id still finds a creature on the
/// battlefield, and the epoch says it is not the one targeted, so Bolt does
/// not resolve. Partial: the atom's board is a delayed trigger (TR-3a's).
// COVERS-PARTIAL: ATOM-400.7-001
#[test]
fn a_spell_whose_target_left_and_came_back_does_not_resolve() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let bolt = cast_from_pool(&mut game, 0, lightning_bolt(), &[(ManaType::Red, 1)], RecordingDecisionProvider::picking(2));
    let announced: Vec<ResolvedTarget> = game.stack_entries[&bolt].chosen_targets[0].as_resolved_targets().collect();
    assert_eq!(announced, vec![ResolvedTarget::Object(bears)], "players first, then the Bears");

    leave_and_return(&mut game, bears, Zone::Graveyard, ZoneChangeCause::Destroyed);
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert!(fizzled(&game, bolt));
    assert_eq!(game.battlefield[&bears].damage_marked, 0);
    assert_eq!(game.get_object(bolt).unwrap().zone, Zone::Graveyard);
}

/// Two references to one creature, one move, and each finds out on its own
/// that the object is new: Giant Growth's row is pruned, and Bolt's target is
/// stale. Neither had to be told by the other. Partial: the atom's first
/// tracker is a delayed trigger.
// COVERS-PARTIAL: ATOM-400.7-003
#[test]
fn two_references_to_one_creature_both_end_at_one_move() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    resolve_spell(&mut game, giant_growth(), 1, &[bears]);
    let bolt = cast_from_pool(&mut game, 0, lightning_bolt(), &[(ManaType::Red, 1)], RecordingDecisionProvider::picking(2));

    leave_and_return(&mut game, bears, Zone::Exile, ZoneChangeCause::Exiled);
    assert_eq!(pt(&game, bears), (Some(2), Some(2)));
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert!(fizzled(&game, bolt));
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield, "the 2/2 is not bolted");
}

/// "Choose a creature. It gets +2/+2 until end of turn", the choice made as
/// the spell is cast. A choice does not fizzle (CR 115.1 is about targets),
/// so the spell resolves, and the creature chosen is gone: the one that came
/// back is another object, and the pump finds nothing.
#[test]
fn a_choice_made_at_cast_finds_nothing_once_its_object_has_moved() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let spell = cast_from_pool(&mut game, 0, chosen_pump(), &[(ManaType::Green, 1)], RecordingDecisionProvider::picking(0));
    assert!(!game.stack_entries[&spell].chosen_targets[0].is_targeted());

    leave_and_return(&mut game, bears, Zone::Hand, ZoneChangeCause::Returned);
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert!(!fizzled(&game, spell), "a choice does not fizzle");
    assert_eq!(pt(&game, bears), (Some(2), Some(2)));
    assert!(game.continuous_effects.is_empty(), "nothing left to pump");
}

/// **Fixture.** "Choose a creature. It gets +2/+2 until end of turn." No
/// printed card chooses an object as it is cast without targeting it and
/// then acts on it, so the shape is written here.
fn chosen_pump() -> Arc<CardData> {
    CardDataBuilder::new("Chosen Pump")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            rules_text: "Choose a creature. It gets +2/+2 until end of turn.".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(2), AmountExpr::Fixed(2), Duration::UntilEndOfTurn),
                EffectRecipient::Choose(SelectionFilter::Creature, TargetCount::Exactly(1)),
            ),
        })
        .build()
}

/// CR 400.3 and 400.7 together: player 0 has stolen player 1's Bears with
/// Act of Treason, and Lightning Bolt is aimed at them when they are
/// destroyed. They go to their owner's graveyard, the steal's row and the
/// Bolt's target both lose them, and the card there has none of the
/// permanent's damage or counters.
// COVERS: COMP-ZONE-TRANSITION-001
#[test]
fn a_stolen_creature_destroyed_goes_home_as_a_new_object() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    resolve_spell(&mut game, mtgsim::cards::phase_lg_cards::act_of_treason(), 0, &[bears]);
    assert_eq!(mtgsim::oracle::characteristics::get_effective_controller(&game, bears), Some(0));
    game.execute_action(
        GameAction::AddCounters { subject: CounterSubject::Object(bears), counter: CounterType::PlusOnePlusOne, n: 1, by: 0 },
        &test_ctx(),
    )
    .unwrap();
    let red = put_on_battlefield(&mut game, red_creature(1), 0);
    deal_damage(&mut game, red, DamageTarget::Object(bears), 1);
    let aim = ScriptedDecisionProvider::new();
    aim.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: bears },
        vec![ChoiceOption::Object(bears)],
    );
    let bolt = cast_from_pool(&mut game, 0, lightning_bolt(), &[(ManaType::Red, 1)], aim);
    let announced: Vec<ResolvedTarget> = game.stack_entries[&bolt].chosen_targets[0].as_resolved_targets().collect();
    assert_eq!(announced, vec![ResolvedTarget::Object(bears)]);

    destroy(&mut game, bears);
    assert!(game.players[1].graveyard.contains(&bears), "its owner's graveyard (CR 400.3)");
    assert!(!game.continuous_effects.iter().any(|row| row.affected_objects.refers_to(bears)), "the steal lost it");
    assert!(!game.battlefield.contains_key(&bears), "no permanent, so no damage and no counters");
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert!(fizzled(&game, bolt), "and so did the Bolt");
}

// ---------------------------------------------------------------------------
// 3. Combat's half
// ---------------------------------------------------------------------------

/// CR 506.4: "a permanent is removed from combat if it leaves the
/// battlefield". A 3/3 blocked by two 1/1s, one of which is destroyed before
/// damage, is blocked by one creature: all its damage goes there (CR
/// 510.1c), and no division is asked between a blocker and a card in a
/// graveyard.
#[test]
fn a_blocker_that_dies_before_damage_is_out_of_the_division() {
    let mut game = setup_two_player_game();
    let attacker = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
    let (gone, stays) = (place_vanilla_creature(&mut game, 1, 1, 1, &[]), place_vanilla_creature(&mut game, 1, 1, 1, &[]));
    set_attacking(&mut game, attacker, 1);
    set_blocked_by(&mut game, attacker, vec![gone, stays]);
    set_blocking(&mut game, gone, vec![attacker]);
    set_blocking(&mut game, stays, vec![attacker]);

    destroy(&mut game, gone);
    let blocked_by = game.battlefield[&attacker].attacking.as_ref().map(|a| a.blocked_by.clone());
    assert_eq!(blocked_by, Some(vec![stays]), "removed from combat as it left");

    let assignments = assign_combat_damage(&game, &ScriptedDecisionProvider::new(), 0, false);
    let from_attacker: Vec<(DamageTarget, u64)> =
        assignments.iter().filter(|a| a.source == attacker).map(|a| (a.target, a.amount)).collect();
    assert_eq!(from_attacker, vec![(DamageTarget::Object(stays), 3)]);
}

// ---------------------------------------------------------------------------
// 4. A re-copy
// ---------------------------------------------------------------------------

/// **Fixture.** A nonlegendary 2/2 with "Creatures you control get +1/+1":
/// the donor whose copied static ability makes a re-copy observable.
fn anthem_bearer() -> Arc<CardData> {
    use mtgsim::types::effects::ObjectFilter;
    use mtgsim::types::effects::PlayerRef;
    CardDataBuilder::new("Anthem Bearer")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(mtgsim::test_support::static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(1), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(ObjectFilter::And(
                Box::new(ObjectFilter::ByType(CardType::Creature)),
                Box::new(ObjectFilter::ByController(PlayerRef::You)),
            )),
        )))
        .build()
}

/// Resolve `card` with `targets`, answering its CR 707.4 choice with `dp`.
fn resolve_spell_with(
    game: &mut GameState,
    card: Arc<CardData>,
    controller: PlayerId,
    targets: &[ObjectId],
    dp: &dyn DecisionProvider,
) {
    let id = game.add_object(GameObject::new(card.clone(), controller, Zone::Stack));
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller,
        targets: ChosenTargets::one(targets.iter().copied().map(ResolvedTarget::Object).collect()),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, dp).unwrap();
}

/// Cytoshape twice in one turn, both times making the same creature a copy
/// of the same Anthem Bearer. Each copy row puts its own instance of the
/// anthem on the creature, and only the one showing applies, so your Bears
/// get +1/+1 once. Keyed by the donor's id alone, the two copies' rows were
/// one ability twice, and both passed CR 604.2's existence check.
#[test]
fn copying_one_donor_twice_applies_its_static_ability_once() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, anthem_bearer(), 1);
    let copier = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    for _ in 0..2 {
        // The candidates are the battlefield in timestamp order: the donor first.
        resolve_spell_with(&mut game, cytoshape(), 0, &[copier], &RecordingDecisionProvider::picking(0));
    }
    assert_eq!(get_effective_name(&game, copier), "Anthem Bearer");
    assert_eq!(pt(&game, bears), (Some(3), Some(3)), "one anthem, not two");
    assert_eq!(pt(&game, copier), (Some(3), Some(3)), "its own anthem, once");
    assert_eq!(pt(&game, donor), (Some(3), Some(3)), "the donor's own, for its own controller");
}

// ---------------------------------------------------------------------------
// 5. Copies that last
// ---------------------------------------------------------------------------

fn serra_angel() -> Arc<CardData> {
    mtgsim::cards::keyword_creatures::serra_angel()
}

/// State-based actions and triggers, then the stack, until both are quiet:
/// the Wall's own entry trigger, here.
fn settle(game: &mut GameState) {
    for _ in 0..20 {
        game.perform_sba_and_triggers(&test_dp()).unwrap();
        if game.stack.is_empty() {
            return;
        }
        game.resolve_top_of_stack(&test_dp()).unwrap();
    }
    panic!("the board never settled");
}

fn counters(game: &GameState, id: ObjectId, counter: CounterType) -> u32 {
    game.battlefield[&id].counter_count(counter)
}

/// Wall of Omens: "When this creature enters, draw a card."
fn wall_of_omens() -> Arc<CardData> {
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    CardDataBuilder::new("Wall of Omens")
        .card_type(CardType::Creature)
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .power_toughness(0, 4)
        .ability(triggered_ability("", whenever(enters(TriggerSubject::ThisObject), draw)))
        .build()
}

/// CR 611.2a: "If no duration is stated, it lasts until the end of the
/// game." Mirrorform states none, so two turns' cleanup leave its copies,
/// and what ends one is CR 400.7: the creature that died and came back is
/// itself again, and the artifact beside it is still an Angel.
// COVERS: ATOM-611.2a-002
#[test]
fn mirrorforms_copies_last_until_their_subject_moves() {
    let mut game = setup_two_player_game();
    let angel = put_on_battlefield(&mut game, serra_angel(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let ring = put_on_battlefield(&mut game, mtgsim::cards::artifacts::sol_ring(), 0);
    resolve_spell(&mut game, mirrorform(), 0, &[angel]);
    assert_eq!(game.continuous_effects.iter().next().map(|row| row.duration), Some(Duration::Indefinite));

    mtgsim::test_support::pass_turn(&mut game);
    mtgsim::test_support::pass_turn(&mut game);
    assert_eq!((get_effective_name(&game, bears), get_effective_name(&game, ring)), ("Serra Angel".into(), "Serra Angel".into()));

    leave_and_return(&mut game, bears, Zone::Graveyard, ZoneChangeCause::Destroyed);
    assert_eq!(get_effective_name(&game, bears), "Grizzly Bears");
    assert_eq!(get_effective_name(&game, ring), "Serra Angel");
}

// RULING: Mirrorform #1 - "Because the permanents aren't entering the
//   battlefield when they become copies of the target permanent, any "When
//   [this permanent] enters" or "[this permanent] enters with" abilities of
//   the copied permanent won't apply."
/// CR 707.4's "the change doesn't cause enters-the-battlefield ... abilities
/// to trigger": a copy of Wall of Omens draws nothing, and a copy of
/// Chainbreaker gets none of its two -1/-1 counters.
#[test]
fn mirrorform_copies_enter_nothing_and_trigger_nothing() {
    let mut game = setup_two_player_game();
    mtgsim::test_support::fill_library(&mut game, 0, 3);
    mtgsim::test_support::fill_library(&mut game, 1, 3);
    let wall = put_on_battlefield(&mut game, wall_of_omens(), 1);
    let breaker = put_on_battlefield(&mut game, chainbreaker(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    settle(&mut game);
    let hand = game.players[0].hand.len();

    resolve_spell(&mut game, mirrorform(), 0, &[wall]);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(get_effective_name(&game, bears), "Wall of Omens");
    assert!(game.stack.is_empty(), "nothing entered, so nothing triggered");
    assert_eq!(game.players[0].hand.len(), hand);

    resolve_spell(&mut game, mirrorform(), 0, &[breaker]);
    assert_eq!(get_effective_name(&game, bears), "Chainbreaker");
    assert_eq!(counters(&game, bears, CounterType::MinusOneMinusOne), 0);
    assert_eq!(pt(&game, bears), (Some(3), Some(3)));
}

// RULING: Mirrorform #2 - "The permanents copy exactly what was printed on
//   the original permanent and nothing else (unless that permanent is copying
//   something else; see below). They don't copy whether that permanent is
//   tapped or untapped, whether it has any counters on it or Auras and
//   Equipment attached to it, or any non-copy effects that have changed its
//   power, toughness, types, color, and so on."
/// The donor's counter, tapped status and pump stay on the donor; the copy
/// is an untapped 4/4 white Angel, as printed.
#[test]
fn mirrorform_copies_only_the_printed_values() {
    let mut game = setup_two_player_game();
    let angel = put_on_battlefield(&mut game, serra_angel(), 1);
    game.add_counters(angel, CounterType::PlusOnePlusOne, 1);
    game.battlefield.get_mut(&angel).unwrap().tapped = true;
    resolve_spell(&mut game, giant_growth(), 1, &[angel]);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);

    resolve_spell(&mut game, mirrorform(), 0, &[angel]);
    assert_eq!(pt(&game, bears), (Some(4), Some(4)));
    assert_eq!(counters(&game, bears, CounterType::PlusOnePlusOne), 0);
    assert!(!game.battlefield[&bears].tapped);
    assert_eq!(get_effective_colors(&game, bears), std::collections::HashSet::from([Color::White]));
}

// RULING: Mirrorform #3 - "If the copied permanent is copying something else,
//   then the permanents become copies of whatever that permanent copied."
/// The donor is a Cytoshape copy of Serra Angel for the turn. Mirrorform's
/// copies are Angels, captured once (CR 707.2b), so they stay Angels after
/// cleanup has ended the donor's own copy.
#[test]
fn mirrorform_of_a_copy_copies_what_it_copied_and_keeps_it() {
    let mut game = setup_two_player_game();
    let angel = put_on_battlefield(&mut game, serra_angel(), 1);
    let donor = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    resolve_spell_with(&mut game, cytoshape(), 1, &[donor], &RecordingDecisionProvider::picking(0));
    assert_eq!(get_effective_name(&game, donor), "Serra Angel");

    resolve_spell(&mut game, mirrorform(), 0, &[donor]);
    assert_eq!(get_effective_name(&game, bears), "Serra Angel");
    mtgsim::test_support::pass_turn(&mut game);
    assert_eq!(get_effective_name(&game, donor), "Grizzly Bears", "Cytoshape's copy ended");
    assert_eq!(get_effective_name(&game, bears), "Serra Angel", "Mirrorform's did not");
    let _ = angel;
}

// RULING: Mirrorform #4 - "If the copied permanent has {X} in its mana cost,
//   X is 0."
/// The copy's cost has the {X}, and its mana value counts it as 0 (CR
/// 202.3e).
#[test]
fn mirrorform_of_an_x_permanent_has_x_as_zero() {
    use mtgsim::types::mana::ManaSymbol;
    let mut game = setup_two_player_game();
    let x_cost = ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Green)]);
    let donor = put_on_battlefield(
        &mut game,
        CardDataBuilder::new("X Creature").card_type(CardType::Creature).mana_cost(x_cost.clone()).power_toughness(2, 2).build(),
        1,
    );
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    resolve_spell(&mut game, mirrorform(), 0, &[donor]);
    let cost = mtgsim::engine::layers::compute_characteristics(&game, bears).unwrap().mana_cost.clone().expect("the copied cost");
    assert_eq!(cost, x_cost);
    assert_eq!(cost.mana_value(), 1);
}

/// Item 16b's board, and why its sized fix would have been wrong: a turn's
/// copy laid over an indefinite one hides the older copy's anthem for the
/// turn, and when cleanup ends it, the older copy shows again and its anthem
/// with it. Dropping the older copy's rows at the re-copy would have lost
/// the anthem for good.
#[test]
fn an_indefinite_copy_shows_again_with_its_statics_when_a_later_copy_ends() {
    let mut game = setup_two_player_game();
    let bearer = put_on_battlefield(&mut game, anthem_bearer(), 1);
    let copier = put_on_battlefield(&mut game, grizzly_bears(), 0);
    resolve_spell(&mut game, mirrorform(), 0, &[bearer]);
    let other = put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(pt(&game, other), (Some(3), Some(3)), "the copied anthem");

    // Cytoshape's candidates: [bearer, copier, other]; the last is a plain Bear.
    resolve_spell_with(&mut game, cytoshape(), 0, &[copier], &RecordingDecisionProvider::picking(2));
    assert_eq!(get_effective_name(&game, copier), "Grizzly Bears");
    assert_eq!(pt(&game, other), (Some(2), Some(2)), "hidden for the turn");

    mtgsim::test_support::pass_turn(&mut game);
    assert_eq!(get_effective_name(&game, copier), "Anthem Bearer");
    assert_eq!(pt(&game, other), (Some(3), Some(3)), "and back with it");
}

// ---------------------------------------------------------------------------
// 6. "Another target"
// ---------------------------------------------------------------------------

/// **Fixture.** "{T}: Another target creature gets +1/+1 until end of turn."
/// No registered card has an activated "another target" yet; Cryptoplasm's
/// is a trigger's.
fn another_pumper() -> Arc<CardData> {
    use mtgsim::types::effects::ObjectFilter;
    CardDataBuilder::new("Another Pumper")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .card_type(CardType::Creature)
        .power_toughness(1, 1)
        .ability(AbilityDef {
            rules_text: "{T}: Another target creature gets +1/+1 until end of turn.".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![mtgsim::types::costs::Cost::TapSelf],
            effect: Effect::Atom(
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(1), Duration::UntilEndOfTurn),
                EffectRecipient::Target(
                    SelectionFilter::Permanent(mtgsim::cards::authoring::another(ObjectFilter::ByType(CardType::Creature))),
                    TargetCount::Exactly(1),
                ),
            ),
        })
        .build()
}

/// "Another target creature" is other than the permanent whose ability it
/// is (CR 113.7a). With that permanent the only creature, the ability is not
/// offered (CR 602.2b through 601.2c's "legal choices for all its targets");
/// with a second creature it is, and the announcement offers only that one.
#[test]
fn another_target_is_other_than_the_abilitys_own_source() {
    use mtgsim::oracle::legality::candidate_priority_actions;
    use mtgsim::ui::decision::PriorityAction;
    let mut game = setup_two_player_game();
    let pumper = put_on_battlefield(&mut game, another_pumper(), 0);
    let offered = |game: &GameState| {
        candidate_priority_actions(game, 0).iter().any(|a| matches!(a, PriorityAction::ActivateAbility(s, _) if *s == pumper))
    };
    assert!(!offered(&game), "it is not another creature");

    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert!(offered(&game));
    let dp = RecordingDecisionProvider::picking(0);
    game.activate_ability(0, pumper, 0, &dp).unwrap();
    let ability = *game.stack.last().unwrap();
    let announced: Vec<ResolvedTarget> = game.stack_entries[&ability].chosen_targets[0].as_resolved_targets().collect();
    assert_eq!(announced, vec![ResolvedTarget::Object(bears)]);
    assert_eq!(dp.prompts(), 0, "one legal choice is no choice");

    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(pt(&game, bears), (Some(3), Some(3)));
}

// ---------------------------------------------------------------------------
// 7. Cryptoplasm
// ---------------------------------------------------------------------------

/// Walk the turn machinery until `whose` player's `step` begins. Libraries
/// are filled so a draw step on the way is not a loss.
fn advance_to(game: &mut GameState, whose: PlayerId, step: StepType) {
    for p in 0..game.num_players() {
        if game.players[p].library.len() < 5 {
            mtgsim::test_support::fill_library(game, p, 10);
        }
    }
    for _ in 0..200 {
        game.advance_turn(&test_ctx()).expect("advancing");
        if game.active_player == whose && game.phase.step == Some(step) {
            return;
        }
    }
    panic!("player {whose}'s {step:?} never began");
}

/// The trigger on top of the stack, and what it targets.
fn top_targets(game: &GameState) -> (ObjectId, Vec<ResolvedTarget>) {
    let top = *game.stack.last().expect("a trigger on the stack");
    (top, game.stack_entries[&top].chosen_targets[0].as_resolved_targets().collect())
}

/// Place the upkeep trigger, choosing `target` among several candidates.
fn place_targeting(game: &mut GameState, target: ObjectId) -> ObjectId {
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: target },
        vec![ChoiceOption::Object(target)],
    );
    game.perform_sba_and_triggers(&dp).unwrap();
    assert!(dp.is_empty(), "the target was asked for as the trigger was put on the stack");
    let (trigger, targets) = top_targets(game);
    assert_eq!(targets, vec![ResolvedTarget::Object(target)]);
    trigger
}

/// Resolve the trigger on top, answering its "may".
fn resolve_may(game: &mut GameState, yes: bool) {
    let (trigger, _) = top_targets(game);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::ApplyOptionalEffect { source: trigger }, if yes { vec![0] } else { vec![] });
    game.resolve_top_of_stack(&dp).unwrap();
    assert!(dp.is_empty(), "the may was asked as it resolved");
}

/// Does `id` have Cryptoplasm's ability, whichever instance?
fn has_cryptoplasms_ability(game: &GameState, id: ObjectId) -> bool {
    let printed = cryptoplasm().abilities[0].id;
    mtgsim::oracle::characteristics::get_effective_abilities(game, id).iter().any(|a| a.id.definition() == printed.definition())
}

// RULING: Cryptoplasm #1 - "You choose the target for the triggered ability
//   when the ability is put onto the stack. You choose whether or not
//   Cryptoplasm becomes a copy of that creature when the ability resolves."
/// Cast from hand from exactly {1}{U}{U}. At its controller's upkeep the
/// target is chosen as the trigger goes on the stack and nothing else is
/// asked; "may" is asked as it resolves. A no leaves a Cryptoplasm, and next
/// upkeep a yes makes it a Serra Angel.
#[test]
fn cryptoplasm_targets_as_it_triggers_and_asks_may_as_it_resolves() {
    let mut game = setup_two_player_game();
    let angel = put_on_battlefield(&mut game, serra_angel(), 1);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    let crypto = cast_from_pool(
        &mut game,
        0,
        cryptoplasm(),
        &[(ManaType::Blue, 2), (ManaType::Colorless, 1)],
        ScriptedDecisionProvider::new(),
    );
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.get_object(crypto).unwrap().zone, Zone::Battlefield);

    advance_to(&mut game, 0, StepType::Upkeep);
    place_targeting(&mut game, angel);
    resolve_may(&mut game, false);
    assert_eq!(get_effective_name(&game, crypto), "Cryptoplasm");

    advance_to(&mut game, 0, StepType::Upkeep);
    place_targeting(&mut game, angel);
    resolve_may(&mut game, true);
    assert_eq!(get_effective_name(&game, crypto), "Serra Angel");
    assert!(has_cryptoplasms_ability(&game, crypto), "except it has this ability");
}

// RULING: Cryptoplasm #2 - "The copy effect lasts indefinitely. Often, it will
//   last until it is overwritten by another copy effect (if it copies another
//   creature on a future turn, perhaps.)"
/// A Serra Angel through two cleanups and the other player's turn, then a
/// Grizzly Bears from the next upkeep's copy, which the ability it kept made.
#[test]
fn cryptoplasms_copy_lasts_until_another_overwrites_it() {
    let mut game = setup_two_player_game();
    let angel = put_on_battlefield(&mut game, serra_angel(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let crypto = put_on_battlefield(&mut game, cryptoplasm(), 0);

    advance_to(&mut game, 0, StepType::Upkeep);
    place_targeting(&mut game, angel);
    resolve_may(&mut game, true);
    advance_to(&mut game, 1, StepType::Upkeep);
    advance_to(&mut game, 0, StepType::Upkeep);
    assert_eq!(get_effective_name(&game, crypto), "Serra Angel", "two cleanups later");

    place_targeting(&mut game, bears);
    resolve_may(&mut game, true);
    assert_eq!(get_effective_name(&game, crypto), "Grizzly Bears");
    assert!(has_cryptoplasms_ability(&game, crypto));
}

// RULING: Cryptoplasm #3 - "If the creature is an illegal target when the
//   ability tries to resolve, it won't resolve. Cryptoplasm won't become a copy
//   of that creature; it remains whatever it was before."
/// The target dies and comes back before the trigger resolves: a new object
/// (CR 400.7), so an illegal target (CR 608.2b). The trigger does not
/// resolve, the "may" is never asked, and the Angel it was stays.
#[test]
fn cryptoplasm_whose_target_is_gone_stays_what_it_was() {
    let mut game = setup_two_player_game();
    let angel = put_on_battlefield(&mut game, serra_angel(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let crypto = put_on_battlefield(&mut game, cryptoplasm(), 0);
    advance_to(&mut game, 0, StepType::Upkeep);
    place_targeting(&mut game, angel);
    resolve_may(&mut game, true);

    advance_to(&mut game, 0, StepType::Upkeep);
    let trigger = place_targeting(&mut game, bears);
    leave_and_return(&mut game, bears, Zone::Graveyard, ZoneChangeCause::Destroyed);
    game.resolve_top_of_stack(&ScriptedDecisionProvider::new()).unwrap();
    assert!(!game.stack.contains(&trigger));
    assert_eq!(get_effective_name(&game, crypto), "Serra Angel");
}

// RULING: Cryptoplasm #4 - "If another creature becomes a copy of Cryptoplasm,
//   it will become a copy of whatever Cryptoplasm is currently copying (if
//   anything), plus it will have Cryptoplasm's triggered ability."
/// "Except it has this ability" is part of the copiable values (CR 707.9b),
/// so a Clone of a Cryptoplasm that is an Angel enters as an Angel with the
/// trigger, and the trigger works for the Clone's controller.
#[test]
fn a_clone_of_cryptoplasm_copies_what_it_copies_and_has_its_ability() {
    let mut game = setup_two_player_game();
    let angel = put_on_battlefield(&mut game, serra_angel(), 1);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    let crypto = put_on_battlefield(&mut game, cryptoplasm(), 0);
    advance_to(&mut game, 0, StepType::Upkeep);
    place_targeting(&mut game, angel);
    resolve_may(&mut game, true);

    let clone = mtgsim::test_support::put_in_graveyard(&mut game, phase_cv_cards::clone(), 1);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::ChooseCopySource { source: clone }, vec![ChoiceOption::Object(crypto)]);
    game.change_zone(clone, Zone::Battlefield, ZoneChangeCause::Returned, &mtgsim::engine::actions::ActionContext::new(&dp))
        .unwrap();
    assert_eq!(get_effective_name(&game, clone), "Serra Angel");
    assert!(has_cryptoplasms_ability(&game, clone));

    advance_to(&mut game, 1, StepType::Upkeep);
    assert_eq!(game.pending_triggers.len(), 1, "the Clone's copy of the trigger, at its controller's upkeep");
}

/// CR 707.4: "Some effects cause a permanent that's copying a permanent to
/// copy a different object while remaining on the battlefield. The change
/// doesn't cause enters-the-battlefield or leaves-the-battlefield abilities
/// to trigger. This also doesn't change any noncopy effects presently
/// affecting the permanent." Cryptoplasm, an Angel with Giant Growth on it,
/// becomes a Grizzly Bears: still +3/+3, and Soul Warden sees nothing enter.
// COVERS: ATOM-707.4-001
#[test]
fn a_re_copy_keeps_noncopy_effects_and_enters_nothing() {
    let mut game = setup_two_player_game();
    let angel = put_on_battlefield(&mut game, serra_angel(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let crypto = put_on_battlefield(&mut game, cryptoplasm(), 0);
    put_on_battlefield(&mut game, mtgsim::cards::phase_tr1_cards::soul_warden(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&test_dp()).unwrap();
    }
    advance_to(&mut game, 0, StepType::Upkeep);
    place_targeting(&mut game, angel);
    resolve_may(&mut game, true);

    advance_to(&mut game, 0, StepType::Upkeep);
    place_targeting(&mut game, bears);
    resolve_spell(&mut game, giant_growth(), 0, &[crypto]);
    let before = life(&game, 0);
    resolve_may(&mut game, true);

    assert_eq!(get_effective_name(&game, crypto), "Grizzly Bears");
    assert_eq!(pt(&game, crypto), (Some(5), Some(5)), "2/2 and Giant Growth");
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert!(game.stack.is_empty(), "nothing entered or left");
    assert_eq!(life(&game, 0), before);
}

/// CR 603.3d: with no other creature there is no legal target, so the
/// trigger is removed from the stack as it would be put there, and nothing is
/// asked. Cryptoplasm is not "another" creature to itself.
#[test]
fn cryptoplasm_alone_has_no_target_and_its_trigger_is_removed() {
    let mut game = setup_two_player_game();
    let crypto = put_on_battlefield(&mut game, cryptoplasm(), 0);
    advance_to(&mut game, 0, StepType::Upkeep);
    assert_eq!(game.pending_triggers.len(), 1, "it triggers");
    game.perform_sba_and_triggers(&ScriptedDecisionProvider::new()).unwrap();
    assert!(game.stack.is_empty(), "and is removed for want of a target");
    assert_eq!(get_effective_name(&game, crypto), "Cryptoplasm");
}

/// A declined "may" still announced its targets (CR 603.3d), so the instance
/// after it is read past them: "You may have target creature get +2/+2 until
/// end of turn. Target creature gets +1/+1 until end of turn", declined, pumps
/// the second creature and not the first.
#[test]
fn a_declined_may_keeps_the_next_instance_on_its_own_target() {
    let mut game = setup_two_player_game();
    let (first, second) = (put_on_battlefield(&mut game, grizzly_bears(), 0), put_on_battlefield(&mut game, grizzly_bears(), 0));
    let pump = |n: u64| {
        Effect::Atom(
            Primitive::ModifyPowerToughness(AmountExpr::Fixed(n), AmountExpr::Fixed(n), Duration::UntilEndOfTurn),
            EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
        )
    };
    let effect = Effect::Sequence(vec![
        Effect::Optional { chooser: mtgsim::types::effects::PlayerRef::You, effect: Box::new(pump(2)) },
        pump(1),
    ]);
    assert_eq!(effect.instances().len(), 2, "the may's target is an instance");

    let source = game.add_object(GameObject::new(grizzly_bears(), 0, Zone::Stack));
    let mut targets = ChosenTargets::NONE;
    targets.push(vec![ResolvedTarget::Object(first)]);
    targets.push(vec![ResolvedTarget::Object(second)]);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 0,
        targets,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::ApplyOptionalEffect { source }, vec![]);
    game.resolve_effect(&effect, &ctx, &dp).unwrap();
    assert_eq!((pt(&game, first), pt(&game, second)), ((Some(2), Some(2)), (Some(3), Some(3))));
}

// ---------------------------------------------------------------------------
// 8. Item 189
// ---------------------------------------------------------------------------

/// **Fixture.** "Creatures you control with power 4 or greater enter as a
/// copy of this creature." Essence of the Wild with a power gate, which no
/// printed copy has: the gate is what lets a counter written during the entry
/// make the copy applicable (CR 616.2).
fn essence_of_might() -> Arc<CardData> {
    use mtgsim::types::effects::{ObjectFilter, PlayerRef};
    use mtgsim::types::replacement::{CopyDonor, EntryCopyTemplate};
    let big_creatures_you_control = ObjectFilter::And(
        Box::new(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Creature)),
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
        )),
        Box::new(ObjectFilter::Not(Box::new(ObjectFilter::PowerLE(3)))),
    );
    CardDataBuilder::new("Essence of Might")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .power_toughness(6, 6)
        .ability(mtgsim::test_support::static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::battlefield_filter(big_creatures_you_control),
            Rewrite::EnterAsCopy(EntryCopyTemplate { donor: CopyDonor::ThisObject, except: Vec::new() }),
        )))))
        .build()
}

/// Spark Double copies a Grizzly Bears and so enters with its +1/+1 counter
/// (CR 707.9e); Doubling Season makes it two, and a 4/4 entering is now one
/// Essence of Might copies (CR 616.2). The later copy means "the exception's
/// effect doesn't happen", and the two counters were that effect, the one
/// added and the one it was doubled into. Taken back as added, one would
/// stay on a 7/7.
#[test]
fn a_later_copy_takes_back_the_counters_a_doubler_made_of_the_exception() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    put_on_battlefield(&mut game, mtgsim::cards::phase_re_cards::doubling_season(), 0);
    put_on_battlefield(&mut game, essence_of_might(), 0);

    let spark = mtgsim::test_support::put_in_graveyard(&mut game, phase_cv_cards::spark_double(), 0);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::ChooseCopySource { source: spark }, vec![ChoiceOption::Object(bears)]);
    game.change_zone(spark, Zone::Battlefield, ZoneChangeCause::Returned, &mtgsim::engine::actions::ActionContext::new(&dp))
        .unwrap();
    assert!(dp.is_empty());

    assert_eq!(get_effective_name(&game, spark), "Essence of Might");
    assert_eq!(counters(&game, spark, CounterType::PlusOnePlusOne), 0);
    assert_eq!(pt(&game, spark), (Some(6), Some(6)));
}
