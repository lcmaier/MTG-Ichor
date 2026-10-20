//! Phase TR-5d — prevention and excess damage (CR 615.13, 120.10).
//!
//! 1. "Whenever damage that would be dealt to you is prevented" reads one
//!    record for each time a prevention effect is applied to one recipient's
//!    damage in a batch, however many sources the damage came from.
//! 2. Selfless Squire, whose shield outlives it and whose growth reads any
//!    prevention effect.
//! 3. Excess damage, decided with the batch: each record's share of what the
//!    batch dealt each creature or planeswalker past lethal, read once per
//!    permanent by the damage arm and by "was dealt excess damage this turn".
//! 4. Delayed triggers of one template agree on their source by identity
//!    and by what was kept of it, and differ by their creators' X.
//! 5. The damage, and its excess, a resolution's own effect dealt ("this
//!    way"), which Goblin Negotiation reads after its damage's window.
//! 6. A look-back record's frame as the ability triggers: a control change's,
//!    and an unattach's end that left (CR 603.10c-d).

use std::sync::Arc;

use mtgsim::cards::authoring::{at_beginning_of, damage_to_is_prevented, triggered_ability, whenever, Whose};
use mtgsim::cards::phase_rd_cards::{mending_hands, safe_passage};
use mtgsim::cards::phase_tr5d_cards::{goblin_negotiation, selfless_squire};
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeAct};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::{EventCause, GeneratedBy};
use mtgsim::events::event::{DamageTarget, GameEvent};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::get_effective_power;
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::state::replacement_effects::RegisteredReplacementEffect;
use mtgsim::test_support::{
    equipment, fill_library, pass_turn, put_in_graveyard, put_in_hand, put_on_battlefield, setup_two_player_game, static_ability, test_dp, vanilla_creature,
    RecordingDecisionProvider,
};
use mtgsim::types::card_types::{ArtifactType, CardType, Subtype};
use mtgsim::types::effects::{
    AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef, PlayerSet, Primitive, SelectionFilter,
    TargetCount,
};
use mtgsim::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite};
use mtgsim::types::history::ObjectFact;
use mtgsim::types::ids::{AbilityId, ObjectId, ObjectRef, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::oracle::x_values::LegalXValues;
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::triggers::{
    DamageRecipient, DelayedDuration, DelayedProvenance, DelayedTriggerTemplate, DelayedTurn, EventRole, Multiplicity, Referred, TriggerEvent,
    TriggerSubject, TriggerTurn,
};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::display::format_record;

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

// ---------------------------------------------------------------------------
// Excess damage (CR 120.10)
// ---------------------------------------------------------------------------

/// "Whenever a [recipient] is dealt excess [noncombat] damage", read once per
/// permanent.
fn excess_arm(recipient: DamageRecipient, combat: Option<bool>) -> TriggerEvent {
    TriggerEvent::DamageDealt { source: TriggerSubject::Any, recipient, combat, excess: true, multiplicity: Multiplicity::OncePerSubject }
}

fn a_creature() -> DamageRecipient {
    DamageRecipient::Object(Some(ObjectFilter::ByType(CardType::Creature)))
}

/// An enchantment whose controller gains that much life off `arm`.
fn gains_that_much(arm: TriggerEvent) -> Arc<CardData> {
    CardDataBuilder::new("Fixture Gauge")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability(
            "",
            whenever(arm, Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringAmount), EffectRecipient::Controller)),
        ))
        .build()
}

/// A planeswalker with printed loyalty 3 and nothing else.
fn loyalty_three() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Walker").card_type(CardType::Planeswalker).loyalty(3).build()
}

/// The excess on each damage record since `from`, in order.
fn excess_since(game: &GameState, from: usize) -> Vec<u64> {
    game.recorded_events()
        .events()
        .skip(from)
        .filter_map(|e| match e {
            GameEvent::DamageDealt { excess, .. } => Some(*excess),
            _ => None,
        })
        .collect()
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

// COVERS: ATOM-120.10-001
#[test]
fn a_two_two_dealt_five_is_dealt_three_excess() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, gains_that_much(excess_arm(a_creature(), None)), 0);
    let source = put_on_battlefield(&mut game, vanilla_creature(5, 5, &[]), 1);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let before = game.recorded_events().len();

    deal_together(&mut game, vec![damage(source, DamageTarget::Object(bears), 5)], &test_dp());
    assert_eq!(excess_since(&game, before), vec![3]);
    resolve_all(&mut game, &test_dp());

    assert_eq!(life(&game, 0), 23);
}

// RULING: Aegar, the Freezing Flame #2 - lethal damage takes into account
//   the damage already dealt to the creature that turn.
#[test]
fn damage_already_marked_counts_toward_lethal() {
    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    let wall = put_on_battlefield(&mut game, vanilla_creature(0, 4, &[]), 0);
    deal_together(&mut game, vec![damage(source, DamageTarget::Object(wall), 2)], &test_dp());
    let before = game.recorded_events().len();

    deal_together(&mut game, vec![damage(source, DamageTarget::Object(wall), 1)], &test_dp());
    deal_together(&mut game, vec![damage(source, DamageTarget::Object(wall), 3)], &test_dp());

    assert_eq!(excess_since(&game, before), vec![0, 2], "4 toughness, 3 marked, then 3 more past the 1 left");
}

// RULING: Aegar, the Freezing Flame #4 - 1 damage from a deathtouch source
//   is lethal to a creature, and deathtouch does nothing to a planeswalker.
#[test]
fn deathtouch_makes_one_damage_lethal_to_a_creature_and_not_to_a_planeswalker() {
    let mut game = setup_two_player_game();
    let viper = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[KeywordFlag::Deathtouch]), 1);
    let giant = put_on_battlefield(&mut game, vanilla_creature(4, 4, &[]), 0);
    let walker = reanimate(&mut game, loyalty_three(), 0);
    let before = game.recorded_events().len();

    deal_together(
        &mut game,
        vec![damage(viper, DamageTarget::Object(giant), 3), damage(viper, DamageTarget::Object(walker), 3)],
        &test_dp(),
    );

    assert_eq!(excess_since(&game, before), vec![2, 0]);
}

/// CR 120.10's "those sources together": two sources into one creature in
/// one event are one excess, read once for the permanent. A 3/3 dealt 2 and
/// 2 is dealt 1 excess, all of it on the second record.
#[test]
fn two_sources_into_one_creature_make_one_excess_read_once() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, gains_that_much(excess_arm(a_creature(), None)), 0);
    let first = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let second = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let centaur = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    let before = game.recorded_events().len();

    deal_together(
        &mut game,
        vec![damage(first, DamageTarget::Object(centaur), 2), damage(second, DamageTarget::Object(centaur), 2)],
        &test_dp(),
    );
    assert_eq!(excess_since(&game, before), vec![0, 1]);
    resolve_all(&mut game, &test_dp());

    assert_eq!(life(&game, 0), 21, "one trigger, that much is 1");
}

/// Once per permanent, not per event: two creatures each dealt excess in one
/// event are two triggers, each with its own.
#[test]
fn two_creatures_dealt_excess_are_two_triggers() {
    let mut game = setup_two_player_game();
    let gauge = put_on_battlefield(&mut game, gains_that_much(excess_arm(a_creature(), None)), 0);
    let source = put_on_battlefield(&mut game, vanilla_creature(5, 5, &[]), 1);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let elves = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);

    deal_together(
        &mut game,
        vec![damage(source, DamageTarget::Object(bears), 3), damage(source, DamageTarget::Object(elves), 4)],
        &test_dp(),
    );
    assert_eq!(waiting_from(&game, gauge), 2);
    resolve_all(&mut game, &RecordingDecisionProvider::picking(0));

    assert_eq!(life(&game, 0), 24, "1 and 3");
}

// RULING: Aegar, the Freezing Flame #3 - a planeswalker is dealt excess
//   damage past its loyalty.
// RULING: Aegar, the Freezing Flame #6 - a creature planeswalker takes the
//   smaller lethal amount: a 5/5 with three loyalty counters dealt 4 is dealt
//   1 excess.
#[test]
fn a_creature_planeswalker_takes_the_smaller_lethal_amount() {
    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, vanilla_creature(6, 6, &[]), 1);
    let walker = reanimate(&mut game, loyalty_three(), 0);
    let creature_walker = CardDataBuilder::new("Fixture Creature Walker")
        .card_type(CardType::Creature)
        .card_type(CardType::Planeswalker)
        .power_toughness(5, 5)
        .loyalty(3)
        .build();
    let both = reanimate(&mut game, creature_walker, 0);
    let before = game.recorded_events().len();

    deal_together(
        &mut game,
        vec![damage(source, DamageTarget::Object(walker), 5), damage(source, DamageTarget::Object(both), 4)],
        &test_dp(),
    );

    assert_eq!(excess_since(&game, before), vec![2, 1]);
}

/// Prevented damage is never dealt (CR 615.6), so it is no excess: a 2/2
/// under Mending Hands' 4, dealt 5, is dealt 1, none of it excess.
#[test]
fn prevented_damage_is_no_excess() {
    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, vanilla_creature(5, 5, &[]), 1);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    resolve_spell(&mut game, mending_hands(), 0, vec![ResolvedTarget::Object(bears)]);
    let before = game.recorded_events().len();

    deal_together(&mut game, vec![damage(source, DamageTarget::Object(bears), 5)], &test_dp());

    assert_eq!(excess_since(&game, before), vec![0]);
    assert_eq!(game.battlefield[&bears].damage_marked, 1);
}

/// "Is dealt excess noncombat damage" (Fall of Cair Andros): combat damage
/// past lethal triggers nothing.
#[test]
fn excess_noncombat_damage_reads_only_noncombat_damage() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, gains_that_much(excess_arm(a_creature(), Some(false))), 0);
    let source = put_on_battlefield(&mut game, vanilla_creature(5, 5, &[]), 1);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let elves = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let in_combat = GameAction::DealDamage {
        source,
        target: DamageTarget::Object(bears),
        amount: 5,
        is_combat: true,
        unpreventable: false,
        source_lki: None,
    };

    deal_together(&mut game, vec![in_combat, damage(source, DamageTarget::Object(elves), 3)], &test_dp());
    resolve_all(&mut game, &test_dp());

    assert_eq!(life(&game, 0), 22, "the elves' 2, not the bears' 3");
}

/// "Was dealt excess damage this turn" (Maarika, Brutal Gladiator) reads the
/// damage rows' excess; a creature dealt exactly lethal was dealt none.
#[test]
fn was_dealt_excess_damage_this_turn_reads_the_rows_excess() {
    let mut game = setup_two_player_game();
    let banner = CardDataBuilder::new("Fixture Banner")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(AmountExpr::Fixed(5), AmountExpr::Fixed(5), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(ObjectFilter::ThisTurn(ObjectFact::WasDealtExcessDamage)),
        )))
        .build();
    let source = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    let exact = put_on_battlefield(&mut game, vanilla_creature(0, 3, &[]), 0);
    let past = put_on_battlefield(&mut game, vanilla_creature(0, 2, &[]), 0);
    put_on_battlefield(&mut game, banner, 0);

    deal_together(
        &mut game,
        vec![damage(source, DamageTarget::Object(exact), 3), damage(source, DamageTarget::Object(past), 3)],
        &test_dp(),
    );

    assert_eq!(get_effective_power(&game, exact), Some(0), "dealt lethal, none past it");
    assert_eq!(get_effective_power(&game, past), Some(5), "dealt 1 past lethal");
}

/// The registry refuses excess read per record, whose records each hold a
/// share of the event's, and excess dealt to a player, who is dealt none.
#[test]
fn excess_read_per_record_or_of_a_player_is_refused() {
    let refused = |arm: TriggerEvent| {
        let gain = Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringAmount), EffectRecipient::Controller);
        whenever(arm, gain).readings_its_arms_cannot_give().len()
    };
    let per_record = TriggerEvent::DamageDealt {
        source: TriggerSubject::Any,
        recipient: a_creature(),
        combat: None,
        excess: true,
        multiplicity: Multiplicity::PerOccurrence,
    };
    assert_eq!(refused(per_record), 1);
    assert_eq!(refused(excess_arm(DamageRecipient::Player(None), None)), 1);
    assert_eq!(refused(excess_arm(a_creature(), None)), 0);
}

/// The log line names the excess where there is some.
#[test]
fn the_damage_log_line_names_its_excess() {
    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, vanilla_creature(5, 5, &[]), 1);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let before = game.recorded_events().len();

    deal_together(&mut game, vec![damage(source, DamageTarget::Object(bears), 5)], &test_dp());

    let recorded = game.recorded_events();
    let lines: Vec<String> = recorded.records()[before..].iter().map(|record| format_record(&game, record)).collect();
    assert!(lines.iter().any(|line| line.contains("for 5, 3 excess")), "{lines:?}");
}

// ---------------------------------------------------------------------------
// Delayed triggers of one template (item 275)
// ---------------------------------------------------------------------------

/// "At the beginning of the next end step, [effect]", once.
fn at_the_next_end_step(effect: Effect) -> DelayedTriggerTemplate {
    DelayedTriggerTemplate {
        def: Arc::new(whenever(at_beginning_of(StepType::End, Whose::Each), effect)),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NoneNamed,
        rules_text: "".into(),
    }
}

/// `template` made by `source`, the existence `source` names, for player 0,
/// with its creator's `x`.
fn make_delayed(game: &mut GameState, template: &DelayedTriggerTemplate, source: ObjectRef, card: Arc<CardData>, x: Option<u64>) {
    let provenance = DelayedProvenance {
        source,
        source_card: card,
        controller: 0,
        created_by: None,
        x_value: x,
        turn: TriggerTurn::Any,
        resolution: None,
        cause: EventCause::Harness,
    };
    game.create_delayed_trigger(template, provenance, Referred::default()).expect("it is made");
}

/// The end step's delayed triggers, put on the stack and resolved under `dp`.
fn end_step(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.execute_action(GameAction::BeginStep { step: StepType::End, player: 0 }, &ActionContext::new(&test_dp()))
        .expect("the step begins");
    resolve_all(game, dp);
}

/// CR 107.3n: two delayed triggers of one template and one source differ in
/// their creators' X, so the order of "draw X cards" with X = 2 and X = 5 is
/// still asked, though nothing else about them differs.
#[test]
fn delayed_draws_of_two_xs_still_ask_their_order() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 10);
    let card = vanilla_creature(1, 1, &[]);
    let source = put_on_battlefield(&mut game, Arc::clone(&card), 0);
    let source = game.object_ref(source).expect("it is there");
    let draw_x = at_the_next_end_step(Effect::Atom(Primitive::DrawCards(AmountExpr::X), EffectRecipient::Controller));
    make_delayed(&mut game, &draw_x, source, Arc::clone(&card), Some(2));
    make_delayed(&mut game, &draw_x, source, card, Some(5));
    let hand = game.players[0].hand.len();

    let dp = RecordingDecisionProvider::picking(0);
    end_step(&mut game, &dp);

    assert_eq!(dp.kinds(), vec!["OrderTriggers"]);
    assert_eq!(game.players[0].hand.len(), hand + 7);
}

/// CR 113.7a: one delayed trigger made while its source was there and one
/// made after it left keep different information about it, so the order of
/// "put two +1/+1 counters on it" is asked; two made while it stays are not
/// (Protean Hydra's, in `phase_tr5b_integration_test`).
#[test]
fn delayed_triggers_on_either_side_of_their_source_leaving_ask_their_order() {
    let mut game = setup_two_player_game();
    let card = vanilla_creature(1, 1, &[]);
    let id = put_on_battlefield(&mut game, Arc::clone(&card), 0);
    let source = game.object_ref(id).expect("it is there");
    let grow_it = at_the_next_end_step(Effect::Atom(
        Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(2), by: PlayerRef::You },
        EffectRecipient::ThisObject,
    ));
    make_delayed(&mut game, &grow_it, source, Arc::clone(&card), None);
    game.change_zone(id, Zone::Graveyard, ZoneChangeAct::Destroyed, &ActionContext::new(&test_dp())).expect("it dies");
    make_delayed(&mut game, &grow_it, source, card, None);

    let dp = RecordingDecisionProvider::picking(0);
    end_step(&mut game, &dp);

    assert_eq!(dp.kinds(), vec!["OrderTriggers"]);
}

// ---------------------------------------------------------------------------
// The damage, and its excess, dealt this way (item 289)
// ---------------------------------------------------------------------------

/// Cast `card` from player 0's hand with X = `x`, its one target the one
/// creature on the board, paying from a pool of red mana, and resolve it.
fn cast_with_x(game: &mut GameState, card: Arc<CardData>, x: u64) {
    let id = put_in_hand(game, card, 0);
    game.players[0].mana_pool.add(ManaType::Red, x + 2);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_number(ChoiceKind::ChooseXValue { spell_id: id, values: Box::new(LegalXValues::zero_through(x)) }, x);
    game.cast_spell(0, id, &dp).expect("it is castable");
    game.resolve_top_of_stack(&dp).expect("it resolves");
}

/// "Target creature", as the card's instruction names it.
fn target_creature() -> EffectRecipient {
    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1))
}

/// Player 0's Goblin tokens.
fn goblins(game: &GameState) -> usize {
    game.battlefield_ids_ordered()
        .into_iter()
        .filter(|id| game.objects[id].card_data.name == "Goblin Token" && game.battlefield[id].controller == 0)
        .count()
}

/// The card's own text: X damage past a 2/2's lethal 2 is a Goblin each.
#[test]
fn goblin_negotiation_makes_a_goblin_for_each_point_of_excess() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    cast_with_x(&mut game, goblin_negotiation(), 5);

    assert_eq!(goblins(&game), 3);
}

/// Damage within lethal is no excess, so no tokens.
#[test]
fn goblin_negotiation_within_lethal_makes_no_goblins() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    cast_with_x(&mut game, goblin_negotiation(), 2);

    assert_eq!(goblins(&game), 0);
}

// RULING: Goblin Negotiation #1 - excess damage is damage past lethal, the
//   damage already marked on the creature taken into account.
#[test]
fn goblin_negotiation_counts_the_damage_already_marked() {
    let mut game = setup_two_player_game();
    let pinger = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    let wall = put_on_battlefield(&mut game, vanilla_creature(0, 4, &[]), 1);
    deal_together(&mut game, vec![damage(pinger, DamageTarget::Object(wall), 3)], &test_dp());

    // The pinger is a second creature, so the target is asked: the wall.
    let id = put_in_hand(&mut game, goblin_negotiation(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 5);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_number(ChoiceKind::ChooseXValue { spell_id: id, values: Box::new(LegalXValues::zero_through(3)) }, 3);
    dp.expect_choice(ChoiceKind::SelectRecipients { recipient: target_creature(), spell_id: id }, vec![ChoiceOption::Object(wall)]);
    game.cast_spell(0, id, &dp).expect("it is castable");
    game.resolve_top_of_stack(&dp).expect("it resolves");

    assert_eq!(goblins(&game), 2, "1 was still lethal, so 2 of 3 is excess");
}

// RULING: Goblin Negotiation #2 - with deathtouch, 1 damage is lethal, so the
//   rest is excess whatever the creature's toughness.
#[test]
fn goblin_negotiation_with_deathtouch_makes_all_but_one_excess() {
    let deathtouch_negotiation = Arc::new({
        let mut card = (*goblin_negotiation()).clone();
        card.keyword_flags.insert(KeywordFlag::Deathtouch);
        card
    });
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, vanilla_creature(4, 4, &[]), 1);

    cast_with_x(&mut game, deathtouch_negotiation, 4);

    assert_eq!(goblins(&game), 3);
}

/// "The damage dealt this way" is the damage dealt, after prevention (CR
/// 615.6): a fixture's 6 into Mending Hands' 4 deals 2, and gains 2.
#[test]
fn the_damage_dealt_this_way_is_what_prevention_left() {
    let siphon = Effect::Sequence(vec![
        Effect::Atom(
            Primitive::DealDamage { amount: AmountExpr::Fixed(6), unpreventable: false },
            EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
        ),
        Effect::Atom(Primitive::GainLife(AmountExpr::DamageDealtThisWay), EffectRecipient::Controller),
    ]);
    let fixture = fixture_sorcery(siphon);
    let mut game = setup_two_player_game();
    let giant = put_on_battlefield(&mut game, vanilla_creature(7, 7, &[]), 1);
    resolve_spell(&mut game, mending_hands(), 1, vec![ResolvedTarget::Object(giant)]);

    let id = put_in_hand(&mut game, fixture, 0);
    let dp = ScriptedDecisionProvider::new();
    game.cast_spell(0, id, &dp).expect("it is castable");
    game.resolve_top_of_stack(&dp).expect("it resolves");

    assert_eq!(game.battlefield[&giant].damage_marked, 2);
    assert_eq!(life(&game, 0), 22, "the 2 dealt, not the 6 proposed");
}

/// "Deals 2 damage to you" (Char's second half, a painland's) is dealt to
/// the controller, who is no chosen target.
#[test]
fn damage_to_you_is_dealt_to_the_controller() {
    let hurt_me = Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::Fixed(2), unpreventable: false },
        EffectRecipient::Controller,
    );
    let mut game = setup_two_player_game();
    let id = put_in_hand(&mut game, fixture_sorcery(hurt_me), 0);
    let dp = ScriptedDecisionProvider::new();
    game.cast_spell(0, id, &dp).expect("it is castable");
    game.resolve_top_of_stack(&dp).expect("it resolves");

    assert_eq!(life(&game, 0), 18);
}

/// A rider acting during the resolution is its replacement's effect (CR
/// 615.5), not this resolution's: player 1's shield prevents the fixture's 3,
/// and its rider deals 2 to the replaced event's subject, player 1. Nothing
/// was dealt this way.
#[test]
fn a_riders_damage_during_the_resolution_is_not_dealt_this_way() {
    let at_a_player = EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1));
    let siphon = Effect::Sequence(vec![
        Effect::Atom(Primitive::DealDamage { amount: AmountExpr::Fixed(3), unpreventable: false }, at_a_player.clone()),
        Effect::Atom(Primitive::GainLife(AmountExpr::DamageDealtThisWay), EffectRecipient::Controller),
    ]);
    let mut game = setup_two_player_game();
    let shield = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let turn = game.turn_number;
    game.replacement_effects.add(RegisteredReplacementEffect {
        id: 0,
        source: shield,
        controller: 1,
        generated_by: GeneratedBy::Spell,
        duration: Duration::UntilEndOfTurn,
        created_on_turn: turn,
        targets: Vec::new(),
        def: ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            ObjectSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::PreventRemaining),
        )
        .affecting_players(PlayerSet::You)
        .next_damage(3)
        .with_then(Effect::Atom(
            Primitive::DealDamage { amount: AmountExpr::Fixed(2), unpreventable: false },
            at_a_player.clone(),
        )),
    });

    let id = put_in_hand(&mut game, fixture_sorcery(siphon), 0);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::SelectRecipients { recipient: at_a_player, spell_id: id }, vec![ChoiceOption::Player(1)]);
    game.cast_spell(0, id, &dp).expect("it is castable");
    game.resolve_top_of_stack(&dp).expect("it resolves");

    assert_eq!(life(&game, 1), 18, "the rider dealt its 2");
    assert_eq!(life(&game, 0), 20, "the fixture's 3 were prevented");
}

// ---------------------------------------------------------------------------
// A look-back record's frame, as the ability triggers (item 290)
// ---------------------------------------------------------------------------

/// A sorcery for player 0 whose one ability is `effect`.
fn fixture_sorcery(effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new("Fixture Sorcery")
        .card_type(CardType::Sorcery)
        .ability(AbilityDef {
            rules_text: "".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect,
        })
        .build()
}

/// "Whenever this Equipment becomes unattached from a creature, you gain life
/// equal to that creature's power": the creature died, so it is read as it
/// last existed on the battlefield (CR 603.10c, 608.2h), with the
/// Equipment's +3/+0, off the frame the unattach's record carries.
#[test]
fn an_unattach_from_a_creature_that_died_reads_it_as_it_left() {
    let boost = static_ability(Effect::Atom(
        Primitive::ModifyPowerToughness(AmountExpr::Fixed(3), AmountExpr::Fixed(0), Duration::WhileSourceOnBattlefield),
        EffectRecipient::Host,
    ));
    let gain = whenever(
        TriggerEvent::BecomesUnattached { attachment: TriggerSubject::ThisObject, former_host: TriggerSubject::Any },
        Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringPower(EventRole::Host)), EffectRecipient::Controller),
    );
    let sword = CardDataBuilder::new("Fixture Sword")
        .card_type(CardType::Artifact)
        .subtype(Subtype::Artifact(ArtifactType::Equipment))
        .ability(boost)
        .ability(triggered_ability("", gain))
        .build();
    let mut game = setup_two_player_game();
    let host = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let sword = put_on_battlefield(&mut game, sword, 0);
    game.execute_action(GameAction::Attach { attachment: sword, host }, &ActionContext::new(&test_dp())).expect("it attaches");
    assert_eq!(get_effective_power(&game, host), Some(5));

    game.change_zone(host, Zone::Graveyard, ZoneChangeAct::Destroyed, &ActionContext::new(&test_dp())).expect("it dies");
    resolve_all(&mut game, &test_dp());

    assert_eq!(life(&game, 0), 25, "its power as it left, 5, not the card's 2");
}

/// An Equipment's unattach, its host destroyed, carries the host's frame as
/// it left (CR 603.10c), so a trigger reading the former host reads it as it
/// was on the battlefield.
#[test]
fn an_unattach_whose_host_left_carries_the_hosts_frame() {
    let mut game = setup_two_player_game();
    let host = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let sword = put_on_battlefield(&mut game, equipment("Fixture Sword"), 0);
    game.execute_action(GameAction::Attach { attachment: sword, host }, &ActionContext::new(&test_dp())).expect("it attaches");
    let as_it_was = game.object_ref(host).expect("it is there");
    let before = game.recorded_events().len();

    game.change_zone(host, Zone::Graveyard, ZoneChangeAct::Destroyed, &ActionContext::new(&test_dp())).expect("it dies");

    let recorded = game.recorded_events();
    let frames: Vec<Option<ObjectRef>> = recorded
        .events()
        .skip(before)
        .filter_map(|e| match e {
            GameEvent::Unattached { lki, .. } => Some(lki.as_ref().map(|lki| lki.object())),
            _ => None,
        })
        .collect();
    assert_eq!(frames, vec![Some(as_it_was)]);
}
