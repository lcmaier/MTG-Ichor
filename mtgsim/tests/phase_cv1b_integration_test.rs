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

use std::sync::Arc;

use mtgsim::cards::alpha::giant_growth;
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_cv_cards::{cytoshape, mirrorweave};
use mtgsim::cards::phase_rd_cards::{circle_of_protection_red, mending_hands};
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::combat::resolution::assign_combat_damage;
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, EffectOrigin, Layer};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::{CounterSubject, DamageTarget, GameEvent};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
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
use mtgsim::types::zones::{DestructionSource, Zone, ZoneChangeCause};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

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
    let announced: Vec<ResolvedTarget> = game.stack_entries[&bolt].chosen_targets[0].targets().collect();
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
