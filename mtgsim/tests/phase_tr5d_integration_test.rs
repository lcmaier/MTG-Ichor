//! Phase TR-5d — prevention and excess damage (CR 615.13, 120.10).
//!
//! 1. "Whenever damage that would be dealt to you is prevented" reads one
//!    record for each time a prevention effect is applied to one recipient's
//!    damage in a batch, however many sources the damage came from.
//! 2. Selfless Squire, whose shield outlives it and whose growth reads any
//!    prevention effect.

use std::sync::Arc;

use mtgsim::cards::authoring::{damage_to_is_prevented, triggered_ability, whenever};
use mtgsim::cards::phase_rd_cards::{mending_hands, safe_passage};
use mtgsim::cards::phase_tr5d_cards::selfless_squire;
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeAct};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::{DamageTarget, GameEvent};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    pass_turn, put_in_graveyard, put_in_hand, put_on_battlefield, setup_two_player_game, test_dp, vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{CounterType, Effect, EffectRecipient, ObjectFilter, Primitive};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::triggers::{DamageRecipient, EventRole};
use mtgsim::types::zones::Zone;
use mtgsim::ui::decision::DecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve `card`'s first ability for `controller`, the way the spell would.
fn resolve_spell(game: &mut GameState, card: Arc<CardData>, controller: PlayerId, targets: Vec<ResolvedTarget>) {
    let id = put_in_hand(game, card.clone(), controller);
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller,
        generated_by: mtgsim::events::cause::GeneratedBy::Spell,
        targets: ChosenTargets::one(targets),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).expect("it resolves");
}

/// Put a card from a graveyard onto the battlefield, through the chokepoint,
/// so its entry triggers.
fn reanimate(game: &mut GameState, card: Arc<CardData>, player: PlayerId) -> ObjectId {
    let id = put_in_graveyard(game, card, player);
    game.change_zone(id, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&test_dp())).expect("it enters");
    id
}

fn damage(source: ObjectId, target: DamageTarget, amount: u64) -> GameAction {
    GameAction::DealDamage { source, target, amount, is_combat: false, unpreventable: false, source_lki: None }
}

/// `damages` dealt as one event (CR 120.4c), choices answered by `dp`.
fn deal_together(game: &mut GameState, damages: Vec<GameAction>, dp: &dyn DecisionProvider) {
    game.execute_actions(damages, &ActionContext::new(dp)).expect("the damage is dealt");
}

fn resolve_all(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).expect("SBAs and triggers");
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(dp).expect("it resolves");
        game.perform_sba_and_triggers(dp).expect("SBAs and triggers");
    }
}

/// A Selfless Squire that entered on an earlier turn: its entry's
/// prevention was made and ended with that turn (CR 514.2).
fn squire_from_last_turn(game: &mut GameState, player: PlayerId) -> ObjectId {
    let squire = put_on_battlefield(game, selfless_squire(), player);
    resolve_all(game, &test_dp());
    pass_turn(game);
    assert!(game.replacement_effects.is_empty(), "its prevention ended with that turn");
    squire
}

/// The triggers `source` has waiting to be put on the stack.
fn waiting_from(game: &GameState, source: ObjectId) -> usize {
    game.pending_triggers.iter().filter(|p| p.origin.source() == source).count()
}

fn plus_one_counters(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].counter_count(CounterType::PlusOnePlusOne)
}

/// The amounts of the preventions announced since `from`, in order.
fn prevented_since(game: &GameState, from: usize) -> Vec<(DamageTarget, u64)> {
    game.recorded_events()
        .events()
        .skip(from)
        .filter_map(|e| match e {
            GameEvent::DamagePrevented { target, amount, .. } => Some((*target, *amount)),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Selfless Squire and the prevention arm (CR 615.13)
// ---------------------------------------------------------------------------

// RULING: Selfless Squire #1 - its first ability's prevention lasts after it
//   leaves the battlefield.
#[test]
fn selfless_squires_prevention_lasts_after_it_leaves() {
    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    let squire = reanimate(&mut game, selfless_squire(), 0);
    resolve_all(&mut game, &test_dp());
    assert_eq!(game.replacement_effects.len(), 1, "the entry made the prevention");

    game.change_zone(squire, Zone::Graveyard, ZoneChangeAct::Destroyed, &ActionContext::new(&test_dp())).expect("it dies");
    let before = game.recorded_events().len();
    deal_together(&mut game, vec![damage(source, DamageTarget::Player(0), 3)], &test_dp());

    assert_eq!(game.players[0].life_total, 20, "prevented with the Squire gone");
    assert_eq!(prevented_since(&game, before), vec![(DamageTarget::Player(0), 3)]);
}

// RULING: Selfless Squire #2 - any prevention effect triggers its second
//   ability, not only its first ability's.
#[test]
fn selfless_squire_grows_from_any_prevention_of_damage_to_you() {
    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    let squire = squire_from_last_turn(&mut game, 0);
    resolve_spell(&mut game, mending_hands(), 0, vec![ResolvedTarget::Player(0)]);

    deal_together(&mut game, vec![damage(source, DamageTarget::Player(0), 3)], &test_dp());
    assert_eq!(waiting_from(&game, squire), 1);
    resolve_all(&mut game, &test_dp());

    assert_eq!(game.players[0].life_total, 20);
    assert_eq!(plus_one_counters(&game, squire), 3, "that many");
}

/// CR 615.13's "each time a prevention effect is applied": two effects that
/// each prevent part of one damage are two applications, so two triggers,
/// each with its own "that many". Two Mending Hands of 4 against 6: the
/// first chosen prevents 4 and the second 2, whichever is chosen first.
#[test]
fn two_prevention_effects_on_one_damage_are_two_triggers() {
    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, vanilla_creature(6, 6, &[]), 1);
    let squire = squire_from_last_turn(&mut game, 0);
    resolve_spell(&mut game, mending_hands(), 0, vec![ResolvedTarget::Player(0)]);
    resolve_spell(&mut game, mending_hands(), 0, vec![ResolvedTarget::Player(0)]);
    let before = game.recorded_events().len();

    let dp = RecordingDecisionProvider::picking(0);
    deal_together(&mut game, vec![damage(source, DamageTarget::Player(0), 6)], &dp);
    assert_eq!(prevented_since(&game, before), vec![(DamageTarget::Player(0), 4), (DamageTarget::Player(0), 2)]);
    assert_eq!(waiting_from(&game, squire), 2);
    resolve_all(&mut game, &dp);

    assert_eq!(plus_one_counters(&game, squire), 6);
}

/// One effect applied to two sources' damage to one player is one
/// application (the pipeline's subject group), so one trigger whose "that
/// many" is the total.
#[test]
fn one_prevention_over_two_sources_is_one_trigger() {
    let mut game = setup_two_player_game();
    let first = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let second = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    let squire = squire_from_last_turn(&mut game, 0);
    resolve_spell(&mut game, safe_passage(), 0, Vec::new());

    deal_together(
        &mut game,
        vec![damage(first, DamageTarget::Player(0), 2), damage(second, DamageTarget::Player(0), 3)],
        &test_dp(),
    );
    assert_eq!(waiting_from(&game, squire), 1);
    resolve_all(&mut game, &test_dp());

    assert_eq!(plus_one_counters(&game, squire), 5);
}

/// Safe Passage applied to you and to your creature in one event is two
/// applications, one to each recipient, and only the first is damage that
/// would be dealt to you.
#[test]
fn a_prevention_for_your_creature_is_not_damage_to_you() {
    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let squire = squire_from_last_turn(&mut game, 0);
    resolve_spell(&mut game, safe_passage(), 0, Vec::new());
    let before = game.recorded_events().len();

    deal_together(
        &mut game,
        vec![damage(source, DamageTarget::Player(0), 3), damage(source, DamageTarget::Object(bears), 2)],
        &test_dp(),
    );
    assert_eq!(prevented_since(&game, before).len(), 2);
    assert_eq!(waiting_from(&game, squire), 1);
    resolve_all(&mut game, &test_dp());

    assert_eq!(plus_one_counters(&game, squire), 3);
}

/// "Whenever damage that would be dealt to a creature is prevented, tap that
/// creature": an object recipient is the event's damage recipient, as on the
/// damage arm.
#[test]
fn a_prevented_creatures_event_role_is_the_damage_recipient() {
    let arm = damage_to_is_prevented(DamageRecipient::Object(Some(ObjectFilter::ByType(CardType::Creature))));
    let def = whenever(arm, Effect::Atom(Primitive::Tap, EffectRecipient::Triggering(EventRole::DamageRecipient)));
    assert!(def.event_roles_its_arms_lack().is_empty());
    let watcher = CardDataBuilder::new("Fixture Watcher")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", def))
        .build();

    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    put_on_battlefield(&mut game, watcher, 1);
    resolve_spell(&mut game, safe_passage(), 0, Vec::new());

    deal_together(&mut game, vec![damage(source, DamageTarget::Object(bears), 2)], &test_dp());
    resolve_all(&mut game, &test_dp());

    assert!(game.battlefield[&bears].tapped, "that creature");
    assert_eq!(game.battlefield[&bears].damage_marked, 0);
}

