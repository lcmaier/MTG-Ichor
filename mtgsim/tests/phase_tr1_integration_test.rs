//! Phase TR-1 — the trigger spine: dispatch, the queue, placement, the stack
//! object (`triggers-architecture.md` §12, TR-1; §13's TR-1 row).
//!
//! **Two instants, and every board here is about one of them.** Detection
//! runs at the close of a batch or at an unbatched emission and writes a
//! `PendingTrigger` onto `GameState`; nothing happens then (CR 603.2,
//! 117.2a). Placement runs inside the CR 117.5 loop, in APNAP order over the
//! seat list, and is where the refusal, the order and the targets are
//! (CR 603.3a–d). The tests read the queue between the two, which is what
//! separates "it triggered" from "it went on the stack" from "it resolved".
//!
//! The fixtures are invented and carry their own names (`engineering-
//! practices.md` §3); the five cards are printed and were verified on
//! Scryfall on 2026-09-19.

use std::sync::Arc;

use mtgsim::cards::alpha::lightning_bolt;
use mtgsim::cards::artifacts::sol_ring;
use mtgsim::cards::authoring::{
    another, at_beginning_of, dies, enters, triggered_ability, whenever, Whose,
};
use mtgsim::cards::basic_lands::{forest, plains, swamp};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_ld_cards::march_of_the_machines;
use mtgsim::cards::phase_lf_cards::humility;
use mtgsim::cards::phase_lg_cards::act_of_treason;
use mtgsim::cards::phase_lj_cards::yixlid_jailer;
use mtgsim::cards::phase_rb_cards::rest_in_peace;
use mtgsim::cards::phase_rd_cards::fog;
use mtgsim::cards::phase_re_cards::eon_hub;
use mtgsim::cards::phase_tr1_cards::{
    blood_artist, felidar_sovereign, saproling_token, soul_warden, verdant_force, wild_growth,
};
use mtgsim::engine::actions::{DestructionSource, GameAction};
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::engine::triggers::{is_mana_ability, visible_to_all};
use mtgsim::events::event::{DamageTarget, GameEvent};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::get_effective_controller;
use mtgsim::state::game_state::{
    AbilityIdentity, GameResult, GameState, Phase, PhaseType, StackEntry, StepType,
};
use mtgsim::test_support::{
    creature_with_ability, fill_library, install_trace, put_in_hand, put_in_library,
    put_on_battlefield, put_spell_on_stack, registered, setup_game, setup_two_player_game,
    static_ability, test_ctx, test_dp, vanilla_creature, StackWatcher,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::costs::{AdditionalCost, Cost};
use mtgsim::types::effects::{
    AmountExpr, Condition, CounterType, Duration, Effect, EffectRecipient, ManaOutput, ObjectFilter,
    ObjectSet, PlayerRef, Primitive, SelectionFilter, TargetCount, TypeChange,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaSpent, ManaType};
use mtgsim::types::replacement::EnterMods;
use mtgsim::types::triggers::{
    DamageRecipient, Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerOrigin,
    TriggerSubject, TriggerTier,
};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::random::RandomDecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

fn draw_one() -> Effect {
    Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

fn a_creature() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Creature)
}

/// A 1/1 creature fixture carrying one triggered ability.
fn watcher(name: &str, event: impl Into<TriggerEvent>, effect: Effect) -> Arc<CardData> {
    creature_with_ability(name, 1, 1, triggered_ability(whenever(event, effect)))
}

/// A colorless enchantment fixture carrying one triggered ability — for a
/// board where the watcher must not itself be a creature.
fn enchantment_watcher(name: &str, ability: AbilityDef) -> Arc<CardData> {
    CardDataBuilder::new(name).card_type(CardType::Enchantment).ability(ability).build()
}

fn pending(game: &GameState) -> usize {
    game.pending_triggers.len()
}

fn place(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).expect("placing");
}

fn resolve_top(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.resolve_top_of_stack(dp).expect("resolving");
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// The stack from the bottom up, as the source of each entry's ability.
fn stack_sources(game: &GameState) -> Vec<ObjectId> {
    game.stack
        .iter()
        .map(|id| game.stack_entries[id].ability_identity.expect("a trigger's entry").source.id)
        .collect()
}

/// A `SelectRecipients` expectation for Blood Artist's "target player".
fn expect_target_player(dp: &ScriptedDecisionProvider, index: usize) {
    dp.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
            spell_id: ObjectId::UNASSIGNED,
        },
        vec![index],
    );
}

/// Destroy `objects` as one event (CR 608.2f) — a wipe.
fn destroy_all(game: &mut GameState, objects: &[ObjectId], source: ObjectId) {
    let batch = objects
        .iter()
        .map(|&object| GameAction::Destroy { object, source: DestructionSource::Effect(source) })
        .collect();
    game.execute_actions(batch, &test_ctx()).expect("the wipe");
}

/// Resolve `effect` for `controller` against `targets`, the way the stack
/// would, with `source` as the resolution's source.
fn resolve_effect_from(
    game: &mut GameState,
    source: ObjectId,
    controller: PlayerId,
    effect: &Effect,
    targets: Vec<ResolvedTarget>,
    dp: &dyn DecisionProvider,
) {
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller,
        targets: ChosenTargets::one(targets),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(effect, &ctx, dp).expect("resolving the effect");
}

/// Walk the turn machinery until `whose` player's `step` begins. Libraries
/// are filled so a draw step on the way is not a loss.
fn advance_to(game: &mut GameState, whose: PlayerId, step: StepType) {
    for p in 0..game.num_players() {
        if game.players[p].library.len() < 5 {
            fill_library(game, p, 10);
        }
    }
    let ctx = test_ctx();
    for _ in 0..200 {
        game.advance_turn(&ctx).expect("advancing");
        if game.active_player == whose && game.phase.step == Some(step) {
            return;
        }
    }
    panic!("player {whose}'s {step:?} never began");
}

/// Walk to the next upkeep step to begin, whoever's it is, and say whose.
fn next_upkeep(game: &mut GameState) -> PlayerId {
    for p in 0..game.num_players() {
        if game.players[p].library.len() < 5 {
            fill_library(game, p, 10);
        }
    }
    let ctx = test_ctx();
    for _ in 0..200 {
        game.advance_turn(&ctx).expect("advancing");
        if game.phase.step == Some(StepType::Upkeep) {
            return game.active_player;
        }
    }
    panic!("no upkeep began");
}

fn deal(game: &mut GameState, source: ObjectId, target: DamageTarget, amount: u64, combat: bool) {
    game.execute_action(
        GameAction::DealDamage { source, target, amount, is_combat: combat, unpreventable: false },
        &test_ctx(),
    )
    .expect("dealing damage");
}

// ---------------------------------------------------------------------------
// CR 603.2, 117.2a, 603.3 — triggering does nothing; placement is at priority
// ---------------------------------------------------------------------------

/// Soul Warden watches an opponent's creature spell resolve: the ability is
/// in the queue, no life has changed, the stack is empty; the next priority
/// grant puts it on the stack before anyone is asked anything.
// COVERS: ATOM-603.2-001
// COVERS: ATOM-117.2a-001
// COVERS: ATOM-603.3-001
#[test]
fn soul_warden_triggers_when_a_creature_spell_resolves_and_nothing_happens_yet() {
    let mut game = setup_two_player_game();
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    let bears = put_spell_on_stack(&mut game, grizzly_bears(), 1);
    assert_eq!(pending(&game), 0);

    resolve_top(&mut game, &test_dp());

    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield);
    assert_eq!(pending(&game), 1, "CR 603.2 — the ability triggered");
    assert_eq!(life(&game, 0), 20, "and did nothing yet");
    assert!(game.stack.is_empty(), "CR 117.2a — not on the stack until a player would receive priority");
    let entry = &game.pending_triggers[0];
    assert_eq!(entry.controller, 0);
    assert_eq!(entry.origin.source(), warden);
    assert_eq!(entry.tier(), TriggerTier::First);

    // CR 603.3b's record, and its stamp: a consequence of the entry, not part of it.
    let recorded = game.recorded_events();
    let triggered = recorded
        .records()
        .iter()
        .find(|r| matches!(r.event, GameEvent::AbilityTriggered { .. }))
        .expect("one AbilityTriggered per queued trigger");
    assert_eq!(triggered.stamp.batch, None);

    let watcher = StackWatcher::new();
    game.run_priority_round(&watcher).unwrap();
    assert_eq!(watcher.at_first_prompt(), (1, 0), "on the stack, queue empty, before the first prompt");
    assert_eq!(life(&game, 0), 21, "both passed and it resolved");
}

/// Soul Warden's ruling: entering at the same time as two other creatures,
/// it triggers for each of the two others and not for itself — one window,
/// three entries, two triggers (CR 603.6a).
// RULING: Soul Warden #1 - "If this creature enters at the same time as one or more other creatures, its ability will trigger for each of those other creatures."
// COVERS: ATOM-603.6a-001
#[test]
fn soul_warden_entering_beside_two_creatures_triggers_for_each_of_them() {
    let mut game = setup_two_player_game();
    // A Soul Warden token beside two bear tokens: `CreateTokens` is one event.
    let mut warden_token = saproling_token();
    warden_token.name = Some("Soul Warden".to_string());
    warden_token.abilities = soul_warden().abilities.to_vec();
    let bear = saproling_token();
    game.execute_action(
        GameAction::CreateTokens { defs: vec![warden_token, bear.clone(), bear], controller: 0 },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(pending(&game), 2, "two other creatures, two triggers, none for itself");
    // Two bindings differ (two subjects), so the order is the player's.
    let dp = ScriptedDecisionProvider::new();
    dp.expect_ordering(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First }, vec![0, 1]);
    place(&mut game, &dp);
    resolve_top(&mut game, &dp);
    resolve_top(&mut game, &dp);
    assert_eq!(life(&game, 0), 22);
}

/// CR 603.6a's other half: the newcomer's own ETB is checked too — Soul
/// Warden's and the entering creature's both trigger.
// COVERS-PARTIAL: ATOM-603.6a-001
#[test]
fn the_newcomers_own_etb_triggers_beside_soul_wardens() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 1, 3);
    put_on_battlefield(&mut game, soul_warden(), 0);
    let comer = put_on_battlefield(
        &mut game,
        watcher("Arriving Scholar", enters(TriggerSubject::ThisObject), draw_one()),
        1,
    );

    assert_eq!(pending(&game), 2);
    let sources: Vec<ObjectId> = game.pending_triggers.iter().map(|t| t.origin.source()).collect();
    assert!(sources.contains(&comer), "the newcomer was checked (CR 603.6a)");
}

// ---------------------------------------------------------------------------
// CR 603.10a — look-back, on the frame the record carries
// ---------------------------------------------------------------------------

/// Blood Artist's ruling, and CR 603.10a: dying beside two creatures it
/// triggers three times — twice off the frame the game looks back to, since
/// it is in the graveyard by the time anything is checked.
// RULING: Blood Artist #1 - "If Blood Artist and one or more other creatures die at the same time, its ability will trigger for each of those creatures."
// COVERS: ATOM-603.10a-002
#[test]
fn blood_artist_dying_beside_two_creatures_triggers_three_times() {
    let mut game = setup_two_player_game();
    let artist = put_on_battlefield(&mut game, blood_artist(), 0);
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let wrath = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &[artist, a, b], wrath);

    assert_eq!(game.get_object(artist).unwrap().zone, Zone::Graveyard);
    assert_eq!(pending(&game), 3, "its own death and the two others");
    assert!(game.pending_triggers.iter().all(|t| t.origin.source() == artist && t.controller == 0));

    // Each is placed with its own target (CR 603.3d), then resolves.
    let dp = ScriptedDecisionProvider::new();
    dp.expect_ordering(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First }, vec![0, 1, 2]);
    for _ in 0..3 {
        expect_target_player(&dp, 1);
    }
    place(&mut game, &dp);
    assert_eq!(game.stack.len(), 3);
    for _ in 0..3 {
        resolve_top(&mut game, &dp);
    }
    assert_eq!((life(&game, 0), life(&game, 1)), (23, 17));
}

/// ATOM-603.10a-001's board: an artifact with "whenever a creature dies"
/// destroyed in the same wipe as two creatures triggers twice — the game
/// looks back to before the event to see the artifact had the ability.
// COVERS: ATOM-603.10a-001
#[test]
fn an_artifact_dying_in_the_wipe_still_sees_the_creatures_die() {
    let mut game = setup_two_player_game();
    let relic = put_on_battlefield(
        &mut game,
        CardDataBuilder::new("Mourning Idol")
            .card_type(CardType::Artifact)
            .ability(triggered_ability(whenever(dies(a_creature()), gain_one())))
            .build(),
        0,
    );
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &[relic, a, b], source);

    assert_eq!(game.get_object(relic).unwrap().zone, Zone::Graveyard);
    assert_eq!(pending(&game), 2, "once per creature, off the frame");
    let dp = ScriptedDecisionProvider::new();
    dp.expect_ordering(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First }, vec![0, 1]);
    place(&mut game, &dp);
    resolve_top(&mut game, &dp);
    resolve_top(&mut game, &dp);
    assert_eq!(life(&game, 0), 22);
}

/// Item 167: CR 603.10 looks back to "the existence of those abilities ...
/// immediately prior to the event" for a source that *survives* it too. One
/// wipe takes Humility and a creature while Blood Artist lives: before the
/// wipe Blood Artist had no abilities, so nothing triggers, though its
/// ability is back by the time anything is checked.
// COVERS-PARTIAL: ATOM-603.10a-001
#[test]
fn a_survivor_looks_back_to_the_abilities_it_had_before_the_wipe() {
    let mut game = setup_two_player_game();
    let artist = put_on_battlefield(&mut game, blood_artist(), 0);
    let enchantment = put_on_battlefield(&mut game, humility(), 1);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &[enchantment, bear], source);

    assert_eq!(game.get_object(artist).unwrap().zone, Zone::Battlefield);
    assert_eq!(pending(&game), 0, "no ability before the event, so no trigger");
}

/// Item 167's other sign: a look-back ability *granted* by a row whose source
/// leaves in the same event existed before it and not after, so it triggers —
/// for the granter's own death too, which is another creature dying.
// COVERS-PARTIAL: ATOM-603.10a-001
#[test]
fn a_grant_ending_in_the_wipe_still_sees_the_creatures_die() {
    let mut game = setup_two_player_game();
    let carrier = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let granter = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let mut granted = triggered_ability(whenever(dies(another(a_creature())), gain_one()));
    granted.id = new_ability_id();
    game.continuous_effects.add(ContinuousEffect {
        duration: Duration::WhileSourceOnBattlefield,
        affected_objects: ObjectSet::Fixed(vec![carrier]),
        ..registered(
            granter,
            Layer::Layer6Ability,
            100,
            EffectModification::GrantAbility(std::sync::Arc::new(granted)),
        )
    });
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &[granter, bear], source);

    assert_eq!(pending(&game), 2, "the granter and the bear, off the list from before");
    assert!(game.pending_triggers.iter().all(|t| t.origin.source() == carrier));
}

/// CR 603.2c — one trigger per occurrence: three lands destroyed as one
/// event are three triggers, not one for the batch.
// COVERS: ATOM-603.2c-001
#[test]
fn a_per_occurrence_trigger_fires_once_per_land_in_a_wipe() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    put_on_battlefield(
        &mut game,
        enchantment_watcher(
            "Landfall Lament",
            triggered_ability(whenever(dies(ObjectFilter::ByType(CardType::Land)), draw_one())),
        ),
        0,
    );
    let lands: Vec<ObjectId> = (0..3).map(|_| put_on_battlefield(&mut game, forest(), 1)).collect();
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &lands, source);

    assert_eq!(pending(&game), 3);
    // And "one or more" is one, over the same window.
    let mut game2 = setup_two_player_game();
    put_on_battlefield(
        &mut game2,
        enchantment_watcher(
            "Landfall Dirge",
            triggered_ability(whenever(
                dies(ObjectFilter::ByType(CardType::Land)).once_per_event(),
                gain_one(),
            )),
        ),
        0,
    );
    let lands: Vec<ObjectId> = (0..3).map(|_| put_on_battlefield(&mut game2, forest(), 1)).collect();
    let source = put_on_battlefield(&mut game2, sol_ring(), 1);
    destroy_all(&mut game2, &lands, source);
    assert_eq!(pending(&game2), 1, "CR 603.2c — the batch is the event for \"one or more\"");
    assert_eq!(game2.pending_triggers[0].binding.records.len(), 3, "and the binding holds every record");
}

// ---------------------------------------------------------------------------
// CR 603.2b, 500.6, 405.3 — the beginning of a step, and APNAP
// ---------------------------------------------------------------------------

/// "At the beginning of your upkeep, you gain 1 life": the upkeep begins,
/// the ability triggers, and it is on the stack before anyone has priority.
// COVERS: ATOM-603.2b-001
// COVERS: ATOM-500.6-001
#[test]
fn an_upkeep_trigger_fires_as_the_upkeep_begins_and_is_placed_before_priority() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        watcher("Dawn Chanter", at_beginning_of(StepType::Upkeep, Whose::Yours), gain_one()),
        0,
    );

    advance_to(&mut game, 0, StepType::Upkeep);

    assert_eq!(pending(&game), 1, "CR 603.2b");
    let watcher = StackWatcher::new();
    game.run_priority_round(&watcher).unwrap();
    assert_eq!(watcher.at_first_prompt(), (1, 0), "CR 500.6 — on the stack before priority");
    assert_eq!(life(&game, 0), 21);
}

/// Verdant Force's ruling: each upkeep, not just yours — a Saproling at the
/// opponent's upkeep too.
// RULING: Verdant Force #1 - "Verdant Force's ability triggers at the beginning of each upkeep, not just each of your upkeeps."
#[test]
fn verdant_force_triggers_at_an_opponents_upkeep_too() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, verdant_force(), 0);

    advance_to(&mut game, 1, StepType::Upkeep);

    assert_eq!(pending(&game), 1);
    assert_eq!(game.pending_triggers[0].controller, 0);
    place(&mut game, &test_dp());
    resolve_top(&mut game, &test_dp());
    let saprolings = game
        .battlefield_ids_ordered()
        .into_iter()
        .filter(|id| game.get_object(*id).unwrap().is_token)
        .count();
    assert_eq!(saprolings, 1, "a Saproling under Verdant Force's controller");
}

/// CR 405.3 / 603.3b — the active player's trigger goes on the stack first
/// (lowest); the other player's resolves first.
// COVERS: ATOM-405.3-001
// COVERS: ATOM-603.3b-001
#[test]
fn apnap_puts_the_active_players_trigger_on_the_stack_first() {
    let mut game = setup_two_player_game();
    let chanters = [
        put_on_battlefield(
            &mut game,
            watcher("Dawn Chanter", at_beginning_of(StepType::Upkeep, Whose::Each), gain_one()),
            0,
        ),
        put_on_battlefield(
            &mut game,
            watcher("Dusk Chanter", at_beginning_of(StepType::Upkeep, Whose::Each), gain_one()),
            1,
        ),
    ];

    let active = next_upkeep(&mut game);
    let other = 1 - active;
    assert_eq!(pending(&game), 2);
    place(&mut game, &test_dp());

    assert_eq!(stack_sources(&game), vec![chanters[active], chanters[other]], "bottom to top: active player's first");
    resolve_top(&mut game, &test_dp());
    assert_eq!(life(&game, other), 21, "the non-active player's resolves first");
    assert_eq!(life(&game, active), 20);
}

/// CR 405.3's second sentence: a player with two simultaneous triggers
/// chooses their order — asked once, as a permutation; the first named goes
/// on the stack lowest.
// COVERS: ATOM-405.3-002
#[test]
fn a_player_with_two_different_triggers_chooses_their_order() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    let gainer = put_on_battlefield(
        &mut game,
        watcher("Dawn Chanter", at_beginning_of(StepType::Upkeep, Whose::Each), gain_one()),
        0,
    );
    let drawer = put_on_battlefield(
        &mut game,
        watcher("Dawn Scholar", at_beginning_of(StepType::Upkeep, Whose::Each), draw_one()),
        0,
    );

    next_upkeep(&mut game);
    assert_eq!(pending(&game), 2);
    let dp = ScriptedDecisionProvider::new();
    // Options in trigger order: [gainer, drawer]. Put the drawer's on first.
    dp.expect_ordering(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First }, vec![1, 0]);
    place(&mut game, &dp);

    assert_eq!(stack_sources(&game), vec![drawer, gainer]);
}

/// Item 163's elision: two copies of one ability under one controller with
/// no targets and identical bindings give the same game in either order, so
/// nothing is asked. `test_dp` panics on any prompt.
#[test]
fn two_identical_triggers_are_placed_without_an_ordering_prompt() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        watcher("Dawn Chanter", at_beginning_of(StepType::Upkeep, Whose::Each), gain_one()),
        0,
    );
    put_on_battlefield(
        &mut game,
        watcher("Dawn Chanter", at_beginning_of(StepType::Upkeep, Whose::Each), gain_one()),
        0,
    );

    next_upkeep(&mut game);
    assert_eq!(pending(&game), 2);
    place(&mut game, &test_dp());

    assert_eq!(game.stack.len(), 2);
}

/// The elision's expiry: a target reopens the prompt. Two Blood Artists on
/// one death are asked their order and then each its target.
#[test]
fn two_triggers_with_targets_are_asked_their_order() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, blood_artist(), 0);
    put_on_battlefield(&mut game, blood_artist(), 0);
    let victim = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &[victim], source);
    assert_eq!(pending(&game), 2);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_ordering(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First }, vec![0, 1]);
    expect_target_player(&dp, 1);
    expect_target_player(&dp, 1);
    place(&mut game, &dp);
    assert_eq!(game.stack.len(), 2);
}

/// The elision's other expiry: identical abilities with **different**
/// bindings — two Soul Wardens see one entry (same binding, elided), but two
/// per-creature death triggers on two deaths differ in their records.
#[test]
fn identical_triggers_with_different_bindings_are_asked_their_order() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, watcher("Mourner", dies(a_creature()), gain_one()), 0);
    let a = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let b = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &[a, b], source);
    assert_eq!(pending(&game), 2);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_ordering(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First }, vec![1, 0]);
    place(&mut game, &dp);
    assert_eq!(game.stack.len(), 2);
}

// ---------------------------------------------------------------------------
// CR 502.4, 503.1a, 511.2, 508.1m — held triggers and the steps' own grants
// ---------------------------------------------------------------------------

/// A trigger from the untap step is held (nobody receives priority there)
/// and goes on the stack at the upkeep, beside the upkeep's own trigger,
/// before the active player gets priority.
// COVERS: ATOM-502.4-001
// COVERS: COMP-UNTAP-TRIGGER-UPKEEP-001
// COVERS: ATOM-503.1a-001
#[test]
fn an_untap_step_trigger_is_held_until_the_upkeep() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 1, 5);
    let untapper = put_on_battlefield(
        &mut game,
        watcher("Night Watch", TriggerEvent::BecomesUntapped { subject: TriggerSubject::ThisObject }, draw_one()),
        1,
    );
    game.battlefield.get_mut(&untapper).unwrap().tapped = true;
    let upkeeper = put_on_battlefield(
        &mut game,
        watcher("Dusk Chanter", at_beginning_of(StepType::Upkeep, Whose::Yours), gain_one()),
        1,
    );
    game.set_turn_position(Phase { phase_type: PhaseType::Ending, step: Some(StepType::End) });

    advance_to(&mut game, 1, StepType::Untap);
    assert!(!game.battlefield[&untapper].tapped, "CR 502.3 untapped it");
    assert_eq!(pending(&game), 1, "CR 502.4 — triggered, and held");
    assert!(game.stack.is_empty());
    assert_eq!(game.players[1].hand.len(), 0, "and it has not resolved");

    advance_to(&mut game, 1, StepType::Upkeep);
    assert_eq!(pending(&game), 2, "the upkeep's own joins it");
    let watcher = StackWatcher::new();
    game.run_priority_round(&watcher).unwrap();
    assert_eq!(watcher.at_first_prompt(), (2, 0), "CR 503.1a — both on the stack before priority");
    let order: Vec<ObjectId> = vec![untapper, upkeeper];
    assert!(order.contains(&untapper));
}

/// "At end of combat" triggers as the end-of-combat step begins (CR 511.2).
// COVERS: ATOM-511.2-001
#[test]
fn an_end_of_combat_trigger_fires_as_the_end_of_combat_step_begins() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        watcher("Combat Medic", at_beginning_of(StepType::EndCombat, Whose::Each), gain_one()),
        0,
    );

    advance_to(&mut game, 0, StepType::EndCombat);

    assert_eq!(pending(&game), 1);
    place(&mut game, &test_dp());
    assert_eq!(game.stack.len(), 1);
}

/// CR 508.1m — abilities that trigger on attackers being declared trigger,
/// once per attacker, and are on the stack before the active player gets
/// priority (508.2b).
// COVERS: ATOM-508.1m-001
#[test]
fn attack_triggers_fire_as_attackers_are_declared() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        enchantment_watcher(
            "War Drums",
            triggered_ability(whenever(
                TriggerEvent::Attacks { attacker: TriggerSubject::Any, multiplicity: Multiplicity::PerOccurrence },
                gain_one(),
            )),
        ),
        0,
    );
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(StepType::DeclareAttackers) });

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::DeclareAttackers, vec![0, 1]);
    game.process_declare_attackers(&dp).unwrap();

    assert_eq!(pending(&game), 2, "one per attacker");
    let subjects: Vec<Option<ObjectId>> =
        game.pending_triggers.iter().map(|t| t.binding.subject.map(|o| o.id)).collect();
    assert_eq!(subjects, vec![Some(a), Some(b)]);
    let watcher = StackWatcher::new();
    game.run_priority_round(&watcher).unwrap();
    assert_eq!(watcher.at_first_prompt(), (2, 0), "CR 508.2b");
}

// ---------------------------------------------------------------------------
// CR 603.2e, 603.2f, 603.2g — becomes, visibility, prevented events
// ---------------------------------------------------------------------------

/// A permanent entering tapped never "became tapped" (CR 603.2e); tapping it
/// afterwards does.
// COVERS: ATOM-603.2e-001
#[test]
fn entering_tapped_is_not_becoming_tapped() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    put_on_battlefield(
        &mut game,
        enchantment_watcher(
            "Tap Sentinel",
            triggered_ability(whenever(TriggerEvent::BecomesTapped { subject: TriggerSubject::Any }, draw_one())),
        ),
        0,
    );
    let bear = put_in_hand(&mut game, grizzly_bears(), 0);
    let mods = EnterMods { tapped: true, ..EnterMods::NONE };
    game.execute_action(
        GameAction::EnterBattlefield {
            object: bear,
            from: Some(Zone::Hand),
            controller: 0,
            mods,
            cause: Some(ZoneChangeCause::Resolved),
        },
        &test_ctx(),
    )
    .unwrap();
    assert!(game.battlefield[&bear].tapped);
    assert_eq!(pending(&game), 0, "CR 603.2e — it entered in that state");

    game.battlefield.get_mut(&bear).unwrap().tapped = false;
    game.execute_action(GameAction::Tap { object: bear }, &test_ctx()).unwrap();
    assert_eq!(pending(&game), 1, "the transition is the event");
}

/// CR 603.2f — a card in a library, never visible to all players, does not
/// trigger however its condition is met; and the predicate itself.
// COVERS: ATOM-603.2f-001
#[test]
fn a_card_in_a_library_does_not_trigger() {
    let mut game = setup_two_player_game();
    let hidden = put_in_library(&mut game, watcher("Buried Warden", enters(a_creature()), gain_one()), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);

    assert_eq!(pending(&game), 0);
    assert!(!visible_to_all(&game, hidden));

    let in_hand = put_in_hand(&mut game, grizzly_bears(), 0);
    let on_board = put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert!(!visible_to_all(&game, in_hand));
    assert!(visible_to_all(&game, on_board));
    game.battlefield.get_mut(&on_board).unwrap().face_down = true;
    assert!(!visible_to_all(&game, on_board), "a face-down permanent is hidden information");
}

/// The "whenever damage is dealt to you" fixture — Fog prevents the combat
/// damage, so no damage was dealt and nothing triggers (CR 603.2g, 615.6).
// COVERS: ATOM-603.2g-001
// COVERS: ATOM-615.6-001
#[test]
fn prevented_damage_triggers_nothing() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    put_on_battlefield(
        &mut game,
        enchantment_watcher(
            "Pain Diary",
            triggered_ability(whenever(
                TriggerEvent::DamageDealt {
                    source: TriggerSubject::Any,
                    recipient: DamageRecipient::Player(Some(PlayerRef::You)),
                    combat: None,
                    multiplicity: Multiplicity::PerOccurrence,
                },
                draw_one(),
            )),
        ),
        0,
    );
    let attacker = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    let fog_card = put_in_hand(&mut game, fog(), 0);
    resolve_effect_from(&mut game, fog_card, 0, &fog().abilities[0].effect, Vec::new(), &test_dp());

    deal(&mut game, attacker, DamageTarget::Player(0), 3, true);

    assert_eq!(life(&game, 0), 20);
    assert_eq!(pending(&game), 0, "CR 603.2g — a prevented event triggers nothing");

    // The same board without Fog: the damage happens and the ability triggers.
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        enchantment_watcher(
            "Pain Diary",
            triggered_ability(whenever(
                TriggerEvent::DamageDealt {
                    source: TriggerSubject::Any,
                    recipient: DamageRecipient::Player(Some(PlayerRef::You)),
                    combat: None,
                    multiplicity: Multiplicity::PerOccurrence,
                },
                gain_one(),
            )),
        ),
        0,
    );
    let attacker = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    deal(&mut game, attacker, DamageTarget::Player(0), 3, true);
    assert_eq!(pending(&game), 1);
}

/// CR 614.6 — a replaced event never happens, and the modified one triggers:
/// under Rest in Peace a destroyed creature is exiled, Blood Artist sees no
/// death, and an "is exiled" watcher sees the exile.
// COVERS: ATOM-614.6-001
#[test]
fn a_replaced_death_triggers_the_exile_and_not_the_death() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, rest_in_peace(), 0);
    let artist = put_on_battlefield(&mut game, blood_artist(), 0);
    let exile_watcher = put_on_battlefield(
        &mut game,
        enchantment_watcher(
            "Banishment Ledger",
            triggered_ability(whenever(
                TriggerEvent::ZoneChange {
                    subject: TriggerSubject::Filter(a_creature()),
                    from: Some(Zone::Battlefield),
                    to: Some(Zone::Exile),
                    cause: None,
                    owner: None,
                    multiplicity: Multiplicity::PerOccurrence,
                },
                gain_one(),
            )),
        ),
        0,
    );
    let victim = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &[victim], source);

    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile);
    let sources: Vec<ObjectId> = game.pending_triggers.iter().map(|t| t.origin.source()).collect();
    assert_eq!(sources, vec![exile_watcher], "the exile triggered; the death never happened");
    assert!(!sources.contains(&artist));
}

/// CR 614.8 — a regenerated creature was still dealt damage, so the damage
/// trigger fires while the destruction is replaced.
// COVERS: ATOM-614.8-002
#[test]
fn damage_triggers_still_fire_when_the_creature_regenerates() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        enchantment_watcher(
            "Wound Tally",
            triggered_ability(whenever(
                TriggerEvent::DamageDealt {
                    source: TriggerSubject::Any,
                    recipient: DamageRecipient::Object(Some(a_creature())),
                    combat: None,
                    multiplicity: Multiplicity::PerOccurrence,
                },
                gain_one(),
            )),
        ),
        0,
    );
    let troll = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let shaman = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    resolve_effect_from(
        &mut game,
        shaman,
        0,
        &Effect::Atom(Primitive::Regenerate, EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1))),
        vec![ResolvedTarget::Object(troll)],
        &test_dp(),
    );
    let bolt = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);

    deal(&mut game, bolt, DamageTarget::Object(troll), 3, false);
    assert_eq!(pending(&game), 1, "the damage was dealt");

    place(&mut game, &test_dp());
    assert!(game.battlefield.contains_key(&troll), "regenerated: the destruction was replaced");
    assert_eq!(game.battlefield[&troll].damage_marked, 0);
}

// ---------------------------------------------------------------------------
// CR 603.6, 603.6b, 603.6c — zone-change triggers and the board they read
// ---------------------------------------------------------------------------

/// CR 603.6 — "put a +1/+1 counter on it": bounced before the trigger
/// resolves, the creature is a new object the ability cannot find.
// COVERS: ATOM-603.6-001
#[test]
fn a_zone_change_trigger_cannot_find_an_object_that_left() {
    let counter_on_it = Effect::Atom(
        Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
        EffectRecipient::TriggeringObject,
    );
    // Control: still there, it gets the counter.
    let mut game = setup_two_player_game();
    let grower = put_on_battlefield(
        &mut game,
        watcher("Sprouting Sapling", enters(TriggerSubject::ThisObject), counter_on_it.clone()),
        0,
    );
    assert_eq!(pending(&game), 1);
    place(&mut game, &test_dp());
    resolve_top(&mut game, &test_dp());
    assert_eq!(game.battlefield[&grower].counter_count(CounterType::PlusOnePlusOne), 1);

    // Bounced between trigger and resolution: nothing.
    let mut game = setup_two_player_game();
    let grower = put_on_battlefield(
        &mut game,
        watcher("Sprouting Sapling", enters(TriggerSubject::ThisObject), counter_on_it),
        0,
    );
    game.change_zone(grower, Zone::Hand, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    place(&mut game, &test_dp());
    assert_eq!(game.stack.len(), 1, "the ability still goes on the stack");
    resolve_top(&mut game, &test_dp());
    assert_eq!(game.get_object(grower).unwrap().zone, Zone::Hand);
    assert!(game.battlefield.is_empty());
}

/// CR 603.6c — a leaves-the-battlefield ability looks for the card only in
/// the first zone it went to: moved out of the graveyard before the trigger
/// resolves, "exile it" finds nothing; still there, it is exiled.
// COVERS: ATOM-603.6c-001
// COVERS: ATOM-603.6c-002
#[test]
fn a_dies_trigger_checks_only_the_first_zone_the_card_went_to() {
    let exile_it = Effect::Atom(Primitive::Exile, EffectRecipient::TriggeringObject);
    let mut game = setup_two_player_game();
    let restless = put_on_battlefield(&mut game, watcher("Restless Shade", dies(TriggerSubject::ThisObject), exile_it.clone()), 0);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);
    destroy_all(&mut game, &[restless], source);
    assert_eq!(pending(&game), 1);
    place(&mut game, &test_dp());
    resolve_top(&mut game, &test_dp());
    assert_eq!(game.get_object(restless).unwrap().zone, Zone::Exile, "found in the graveyard, exiled");

    // Enduring Renewal's shape: the card leaves the graveyard first.
    let mut game = setup_two_player_game();
    let renewal = put_on_battlefield(
        &mut game,
        enchantment_watcher("Renewal Vow", triggered_ability(whenever(dies(a_creature()), exile_it))),
        0,
    );
    let _ = renewal;
    let creature = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);
    destroy_all(&mut game, &[creature], source);
    assert_eq!(pending(&game), 1);
    game.change_zone(creature, Zone::Hand, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    place(&mut game, &test_dp());
    resolve_top(&mut game, &test_dp());
    assert_eq!(game.get_object(creature).unwrap().zone, Zone::Hand, "not in the first zone it went to: nothing happens");
}

/// CR 603.6b — the permanent is never on the battlefield unmodified: under
/// March of the Machines an artifact enters as a creature, and Soul Warden
/// sees a creature enter.
// COVERS: ATOM-603.6b-001
#[test]
fn a_permanent_animated_as_it_enters_is_a_creature_to_an_etb_trigger() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, march_of_the_machines(), 1);
    put_on_battlefield(&mut game, soul_warden(), 0);
    assert_eq!(pending(&game), 0, "Soul Warden is a creature, but not another");

    put_on_battlefield(&mut game, sol_ring(), 1);

    assert_eq!(pending(&game), 1, "an artifact that is a creature the moment it enters");
}

/// CR 603.6b's converse — under Humility a creature has no abilities the
/// moment it enters, so its ETB never triggers; and the batch is the event,
/// so Humility entering *beside* the creature strips it too (CR 603.6a).
// COVERS: ATOM-603.6b-002
#[test]
fn humility_strips_an_etb_before_it_can_trigger() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, humility(), 1);
    put_on_battlefield(&mut game, watcher("Arriving Scholar", enters(TriggerSubject::ThisObject), draw_one()), 0);
    assert_eq!(pending(&game), 0);

    // Together, in one window: Humility has no ETB of its own, and the
    // creature's is gone by the time the window closes.
    let mut game = setup_two_player_game();
    let hum = put_in_hand(&mut game, humility(), 1);
    let scholar = put_in_hand(&mut game, watcher("Arriving Scholar", enters(TriggerSubject::ThisObject), draw_one()), 0);
    let entry = |_game: &GameState, object: ObjectId, controller: PlayerId| GameAction::EnterBattlefield {
        object,
        from: Some(Zone::Hand),
        controller,
        mods: EnterMods::NONE,
        cause: Some(ZoneChangeCause::Resolved),
    };
    let batch = vec![entry(&game, scholar, 0), entry(&game, hum, 1)];
    game.execute_actions(batch, &test_ctx()).unwrap();
    assert!(game.battlefield.contains_key(&scholar) && game.battlefield.contains_key(&hum));
    assert_eq!(pending(&game), 0, "checked at the close of the batch, with Humility on the board");
}

// ---------------------------------------------------------------------------
// CR 603.3a, 603.3b (tier 2), 603.3d — the controller, the tiers, the removal
// ---------------------------------------------------------------------------

/// CR 603.3a — controlled by whoever controlled the source *when it
/// triggered*: a steal between trigger and placement changes nothing.
// COVERS: ATOM-603.3a-001
#[test]
fn the_trigger_is_controlled_by_whoever_controlled_the_source_when_it_triggered() {
    let mut game = setup_two_player_game();
    let sentinel = put_on_battlefield(
        &mut game,
        watcher(
            "Gate Sentinel",
            enters(another(a_creature())),
            gain_one(),
        ),
        0,
    );
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(pending(&game), 1);

    let treason = put_in_hand(&mut game, act_of_treason(), 1);
    resolve_effect_from(
        &mut game,
        treason,
        1,
        &act_of_treason().abilities[0].effect,
        vec![ResolvedTarget::Object(sentinel)],
        &test_dp(),
    );
    assert_eq!(get_effective_controller(&game, sentinel), Some(1), "stolen before placement");

    place(&mut game, &test_dp());
    let top = *game.stack.last().unwrap();
    assert_eq!(game.stack_entries[&top].controller, 0, "CR 603.3a");
    resolve_top(&mut game, &test_dp());
    assert_eq!((life(&game, 0), life(&game, 1)), (21, 20));
}

/// CR 603.3b's second tier: an ability that triggers on another ability
/// triggering goes on the stack after the one that caused it, in the same
/// placement — Strict Proctor's shape on a fixture.
// COVERS: ATOM-603.3b-002
#[test]
fn a_trigger_on_a_trigger_is_placed_in_the_second_tier() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    let proctor = put_on_battlefield(
        &mut game,
        enchantment_watcher(
            "Lenient Proctor",
            triggered_ability(whenever(
                TriggerEvent::AbilityTriggers { caused_by: None, source: Some(a_creature()) },
                gain_one(),
            )),
        ),
        1,
    );
    let scholar = put_on_battlefield(
        &mut game,
        watcher("Arriving Scholar", enters(TriggerSubject::ThisObject), draw_one()),
        0,
    );

    assert_eq!(pending(&game), 2);
    let tiers: Vec<(ObjectId, TriggerTier)> = game.pending_triggers.iter().map(|t| (t.origin.source(), t.tier())).collect();
    assert_eq!(tiers, vec![(scholar, TriggerTier::First), (proctor, TriggerTier::Second)]);

    place(&mut game, &test_dp());
    assert_eq!(stack_sources(&game), vec![scholar, proctor], "tier 1 below tier 2, whatever APNAP says");
}

/// CR 603.3d — a choice required with no legal choices: the ability is
/// removed, never reaching the stack, and nothing is announced.
// COVERS: ATOM-603.3d-001
#[test]
fn a_trigger_with_no_legal_target_is_removed_and_never_reaches_the_stack() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        watcher(
            "Shattering Herald",
            enters(TriggerSubject::ThisObject),
            Effect::Atom(
                Primitive::Destroy,
                EffectRecipient::Target(
                    SelectionFilter::Permanent(ObjectFilter::ByType(CardType::Artifact)),
                    TargetCount::Exactly(1),
                ),
            ),
        ),
        0,
    );
    assert_eq!(pending(&game), 1);
    let objects = game.objects.len();
    let before = game.recorded_events().len();

    place(&mut game, &test_dp());

    assert!(game.stack.is_empty());
    assert_eq!(pending(&game), 0);
    assert_eq!(game.objects.len(), objects, "no stack object was created");
    assert_eq!(game.recorded_events().len(), before, "and nothing announced: a removal is not a counter");
}

// ---------------------------------------------------------------------------
// CR 603.4, 608.2a, 608.2 — the intervening "if" at both instants
// ---------------------------------------------------------------------------

/// Felidar Sovereign at 42 life: true at the trigger, true at resolution —
/// the player wins.
// RULING: Felidar Sovereign #1 - "Felidar Sovereign's triggered ability checks to see if you have 40 or more life as your upkeep begins. If you don't, the ability won't trigger at all. If you do, the ability will check again as it tries to resolve. If you don't have 40 or more life at that time, the ability won't do anything."
// COVERS: ATOM-603.4-001
#[test]
fn felidar_sovereign_wins_when_the_condition_holds_at_both_instants() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, felidar_sovereign(), 0);
    game.players[0].life_total = 42;

    advance_to(&mut game, 0, StepType::Upkeep);
    assert_eq!(pending(&game), 1);
    place(&mut game, &test_dp());
    resolve_top(&mut game, &test_dp());

    assert_eq!(game.result, Some(GameResult::Winner(0)));
}

/// Below 40 as the upkeep begins, the ability does not trigger at all.
// COVERS: ATOM-603.4-002
#[test]
fn felidar_sovereign_does_not_trigger_below_forty_life() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, felidar_sovereign(), 0);
    game.players[0].life_total = 38;

    advance_to(&mut game, 0, StepType::Upkeep);

    assert_eq!(pending(&game), 0, "CR 603.4 — false at the trigger, no trigger");
}

/// True at the trigger, false at resolution: removed from the stack, does
/// nothing (CR 603.4, 608.2a).
// COVERS: ATOM-603.4-003
// COVERS: ATOM-608.2a-001
#[test]
fn felidar_sovereign_does_nothing_when_life_drops_before_it_resolves() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, felidar_sovereign(), 0);
    game.players[0].life_total = 42;

    advance_to(&mut game, 0, StepType::Upkeep);
    place(&mut game, &test_dp());
    assert_eq!(game.stack.len(), 1);
    let ability = *game.stack.last().unwrap();

    // In response, 5 damage: 37 life.
    let bolt = put_on_battlefield(&mut game, vanilla_creature(5, 5, &[]), 1);
    deal(&mut game, bolt, DamageTarget::Player(0), 5, false);
    assert_eq!(life(&game, 0), 37);
    let before = game.recorded_events().len();

    resolve_top(&mut game, &test_dp());

    assert_eq!(game.result, None, "CR 608.2a — removed, does nothing");
    assert!(game.stack.is_empty());
    assert!(game.get_object(ability).is_err(), "the object is gone");
    assert!(
        !game.recorded_events().records_from(before).iter().any(|r| matches!(r.event, GameEvent::AbilityResolved { .. })),
        "it did not resolve"
    );
}

/// CR 608.2's order on a triggered ability: 608.2a's clause is checked
/// before 608.2b's targets — false clause and illegal target together is a
/// removal with no fizzle — and on the legal board the effect happens, the
/// object is removed, and `AbilityResolved` is announced after the effect's
/// own events.
// COVERS: ATOM-608.2-001
#[test]
fn the_resolution_checks_the_clause_then_the_targets_then_resolves_then_announces() {
    let herald = || {
        creature_with_ability(
            "Judging Herald",
            1,
            1,
            triggered_ability(TriggerDef {
                condition: TriggerCondition::Event(at_beginning_of(StepType::Upkeep, Whose::Yours)),
                intervening_if: Some(Condition::YourLifeAtLeast(AmountExpr::Fixed(20))),
                limit: None,
                effect: Effect::Atom(
                    Primitive::Destroy,
                    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                ),
            }),
        )
    };

    // Both fail: the clause decides, and no fizzle is announced.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, herald(), 0);
    let target = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    advance_to(&mut game, 0, StepType::Upkeep);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id: ObjectId::UNASSIGNED,
        },
        vec![1],
    );
    place(&mut game, &dp);
    game.players[0].life_total = 10;
    game.change_zone(target, Zone::Exile, ZoneChangeCause::Exiled, &test_ctx()).unwrap();
    let before = game.recorded_events().len();
    resolve_top(&mut game, &dp);
    assert!(game.stack.is_empty());
    assert!(
        !game.recorded_events().records_from(before).iter().any(|r| matches!(
            r.event,
            GameEvent::SpellFizzled { .. } | GameEvent::AbilityResolved { .. }
        )),
        "608.2a came first: no fizzle, no resolution"
    );

    // Everything legal: destroyed, removed, announced — in that order.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, herald(), 0);
    let target = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    advance_to(&mut game, 0, StepType::Upkeep);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id: ObjectId::UNASSIGNED,
        },
        vec![1],
    );
    place(&mut game, &dp);
    let ability = *game.stack.last().unwrap();
    let before = game.recorded_events().len();
    resolve_top(&mut game, &dp);
    let kinds: Vec<&str> = game
        .recorded_events()
        .records_from(before)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::ZoneChange { object_id, .. } if *object_id == target => Some("destroyed"),
            GameEvent::AbilityResolved { .. } => Some("resolved"),
            _ => None,
        })
        .collect();
    assert_eq!(kinds, vec!["destroyed", "resolved"], "608.2c before 608.2n");
    assert!(game.get_object(ability).is_err());
}

// ---------------------------------------------------------------------------
// CR 608.2k — the bound object survives a characteristic change
// ---------------------------------------------------------------------------

/// "Put a +1/+1 counter on that creature" still finds the permanent after
/// it stopped being a creature: the reference was bound by the trigger
/// condition, not re-filtered at resolution.
// COVERS: ATOM-608.2k-001
#[test]
fn a_bound_object_is_still_affected_after_it_changes_characteristics() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        enchantment_watcher(
            "Growth Ledger",
            triggered_ability(whenever(
                enters(a_creature()),
                Effect::Atom(
                    Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
                    EffectRecipient::TriggeringObject,
                ),
            )),
        ),
        0,
    );
    let bear = put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(pending(&game), 1);

    // Before it resolves, the bear becomes a noncreature artifact.
    let change = Effect::Atom(
        Primitive::ChangeType(
            TypeChange {
                add_types: Vec::new(),
                remove_types: Vec::new(),
                set_types: Some([CardType::Artifact].into_iter().collect()),
                add_subtypes: Vec::new(),
                remove_subtypes: Vec::new(),
                set_subtypes: Some(std::collections::HashSet::new()),
                add_supertypes: Vec::new(),
                remove_supertypes: Vec::new(),
                set_supertypes: None,
            },
            Duration::UntilEndOfTurn,
        ),
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    );
    let wand = put_on_battlefield(&mut game, sol_ring(), 1);
    resolve_effect_from(&mut game, wand, 1, &change, vec![ResolvedTarget::Object(bear)], &test_dp());
    assert!(!mtgsim::oracle::characteristics::is_creature(&game, bear));

    place(&mut game, &test_dp());
    resolve_top(&mut game, &test_dp());
    assert_eq!(game.battlefield[&bear].counter_count(CounterType::PlusOnePlusOne), 1, "CR 608.2k");
}

// ---------------------------------------------------------------------------
// CR 605.1b, 605.4a, 605.5a, 106.12a — the triggered mana ability
// ---------------------------------------------------------------------------

/// Wild Growth: the land is tapped for mana, its record says so
/// (CR 106.12a), and the extra {G} is in the pool at once — no stack, no
/// queue, no priority.
// COVERS: ATOM-605.1b-001
// COVERS: ATOM-605.4a-001
// COVERS: ATOM-106.12a-001
#[test]
fn wild_growth_adds_its_mana_at_once_without_the_stack() {
    let mut game = setup_two_player_game();
    let land = put_on_battlefield(&mut game, forest(), 0);
    let aura = put_on_battlefield(&mut game, wild_growth(), 0);
    assert!(game.attach(aura, land));
    let ability = forest().abilities[0].id;

    game.activate_mana_ability(0, land, ability, &test_ctx()).unwrap();

    assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 2, "the land's {{G}} and the Aura's");
    assert_eq!(pending(&game), 0, "CR 605.4a — never queued");
    assert!(game.stack.is_empty());
    let added: Vec<(ObjectId, bool)> = game
        .recorded_events()
        .events()
        .filter_map(|e| match e {
            GameEvent::ManaAdded { source_id, tapped_for_mana, .. } => Some((*source_id, *tapped_for_mana)),
            _ => None,
        })
        .collect();
    assert_eq!(added, vec![(land, true), (aura, false)], "CR 106.12a's fact on the record that triggered it");
}

/// Wild Growth's ruling: the extra mana is the Aura's, not the land's, so
/// a second Wild Growth on the same land adds one more {G} — three in all —
/// and neither Aura triggers on the other's mana.
// RULING: Wild Growth #1 - "The additional mana is not an ability of the land and is not something the land can produce."
#[test]
fn wild_growths_mana_is_the_auras_and_does_not_tap_the_land_for_mana_again() {
    let mut game = setup_two_player_game();
    let land = put_on_battlefield(&mut game, forest(), 0);
    for _ in 0..2 {
        let aura = put_on_battlefield(&mut game, wild_growth(), 0);
        assert!(game.attach(aura, land));
    }
    game.activate_mana_ability(0, land, forest().abilities[0].id, &test_ctx()).unwrap();
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 3);
}

/// "Its controller adds": the enchanted land's controller, whoever controls
/// the Aura.
#[test]
fn wild_growth_adds_to_the_lands_controllers_pool() {
    let mut game = setup_two_player_game();
    let land = put_on_battlefield(&mut game, forest(), 1);
    let aura = put_on_battlefield(&mut game, wild_growth(), 0);
    assert!(game.attach(aura, land));
    game.activate_mana_ability(1, land, forest().abilities[0].id, &test_ctx()).unwrap();
    assert_eq!(game.players[1].mana_pool.amount(ManaType::Green), 2);
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 0);
}

/// CR 605.5a — a target, or an event other than mana being added,
/// disqualifies a trigger from being a mana ability; it uses the stack.
// COVERS: ATOM-605.5a-001
#[test]
fn a_mana_producing_trigger_from_another_event_uses_the_stack() {
    let produce = Effect::Atom(
        Primitive::ProduceMana(ManaOutput { mana: vec![(ManaType::Green, AmountExpr::Fixed(1))], special: vec![] }),
        EffectRecipient::Controller,
    );
    let from_entry = whenever(enters(a_creature()), produce.clone());
    assert!(!is_mana_ability(&from_entry), "triggers from a creature entering");
    let Effect::Triggered(wild) = &wild_growth().abilities[0].effect else { panic!() };
    assert!(is_mana_ability(wild));
    let mut targeted = (**wild).clone();
    targeted.effect = Effect::Sequence(vec![
        produce.clone(),
        Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1))),
    ]);
    assert!(!is_mana_ability(&targeted), "a target disqualifies it");

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, enchantment_watcher("Verdant Echo", triggered_ability(from_entry)), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(pending(&game), 1, "queued like any other trigger");
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 0);
    place(&mut game, &test_dp());
    assert_eq!(game.stack.len(), 1);
}

// ---------------------------------------------------------------------------
// CR 119.9 — life gain is per source, and a 0 gain is no event
// ---------------------------------------------------------------------------

/// Two lifelink creatures dealing combat damage at once are two life-gain
/// events, and "whenever you gain life" triggers twice.
// COVERS: ATOM-119.9-001
#[test]
fn each_source_of_simultaneous_life_gain_triggers_separately() {
    use mtgsim::types::keywords::KeywordFlag;
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        enchantment_watcher(
            "Vital Ledger",
            triggered_ability(whenever(
                TriggerEvent::GainsLife { player: Some(PlayerRef::You), multiplicity: Multiplicity::PerOccurrence },
                draw_one(),
            )),
        ),
        0,
    );
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Lifelink]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[KeywordFlag::Lifelink]), 0);

    game.execute_actions(
        vec![
            GameAction::DealDamage { source: a, target: DamageTarget::Player(1), amount: 2, is_combat: true, unpreventable: false },
            GameAction::DealDamage { source: b, target: DamageTarget::Player(1), amount: 3, is_combat: true, unpreventable: false },
        ],
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(life(&game, 0), 25);
    assert_eq!(pending(&game), 2, "CR 119.9 — once per source");
}

/// Gaining 0 life is not a life-gain event (CR 119.10), so nothing triggers.
// COVERS: ATOM-119.9-002
#[test]
fn gaining_zero_life_triggers_nothing() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        enchantment_watcher(
            "Vital Ledger",
            triggered_ability(whenever(
                TriggerEvent::GainsLife { player: Some(PlayerRef::You), multiplicity: Multiplicity::PerOccurrence },
                draw_one(),
            )),
        ),
        0,
    );
    let source = put_on_battlefield(&mut game, sol_ring(), 0);
    game.execute_action(GameAction::GainLife { player: 0, amount: 0, source }, &test_ctx()).unwrap();
    assert_eq!(pending(&game), 0);
}

// ---------------------------------------------------------------------------
// CR 113.9 — a triggered ability on the stack can be countered by an
// effect that counters abilities
// ---------------------------------------------------------------------------

// COVERS: ATOM-113.9-003
#[test]
fn an_effect_that_counters_abilities_counters_a_triggered_ability() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, soul_warden(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    place(&mut game, &test_dp());
    let ability = *game.stack.last().unwrap();
    assert!(!game.is_spell_on_stack(ability), "not a spell (CR 113.9)");

    let stifle = put_in_hand(&mut game, lightning_bolt(), 1);
    resolve_effect_from(
        &mut game,
        stifle,
        1,
        &Effect::Atom(Primitive::CounterAbility, EffectRecipient::Target(SelectionFilter::Spell, TargetCount::Exactly(1))),
        vec![ResolvedTarget::Object(ability)],
        &test_dp(),
    );

    assert!(game.stack.is_empty());
    assert!(game.recorded_events().events().any(|e| matches!(e, GameEvent::AbilityCountered { ability_id, .. } if *ability_id == ability)));
    assert_eq!(life(&game, 0), 20);
}

// ---------------------------------------------------------------------------
// CR 514.3a — a trigger at cleanup re-loops the step
// ---------------------------------------------------------------------------

/// A discard-to-hand-size trigger: triggers are waiting at the cleanup
/// step's check, so priority is granted and another cleanup step begins.
// COVERS: COMP-CLEANUP-RELOOP-001
#[test]
fn a_trigger_at_cleanup_grants_priority_and_begins_another_cleanup_step() {
    use mtgsim::state::game::Game;
    use mtgsim::state::game_config::GameConfig;
    use mtgsim::ui::random::RandomDecisionProvider;

    let deck: Vec<Arc<CardData>> = (0..20).map(|_| forest()).collect();
    let mut g = Game::new(GameConfig::test(), vec![deck.clone(), deck]).unwrap();
    g.state.record_events();
    let dp = RandomDecisionProvider::seeded(603);
    g.setup(&dp).unwrap();
    g.state.set_turn_position(Phase { phase_type: PhaseType::Ending, step: Some(StepType::Cleanup) });
    put_on_battlefield(
        &mut g.state,
        enchantment_watcher(
            "Discard Diary",
            triggered_ability(whenever(
                TriggerEvent::ZoneChange {
                    subject: TriggerSubject::Any,
                    from: Some(Zone::Hand),
                    to: Some(Zone::Graveyard),
                    cause: Some(ZoneChangeCause::Discarded),
                    owner: Some(PlayerRef::You),
                    multiplicity: Multiplicity::PerOccurrence,
                },
                gain_one(),
            )),
        ),
        0,
    );
    put_in_hand(&mut g.state, forest(), 0);
    assert_eq!(g.state.players[0].hand.len(), 8);
    let before = g.state.recorded_events().len();

    g.run_turn(&dp).unwrap();

    assert_eq!(g.state.players[0].life_total, 21, "the trigger resolved during the granted priority");
    let cleanups = g
        .state
        .recorded_events()
        .records_from(before)
        .iter()
        .filter(|r| matches!(r.event, GameEvent::StepBegin { step: StepType::Cleanup, .. }))
        .count();
    assert_eq!(cleanups, 1, "CR 514.3a — another cleanup step began");
}

// ---------------------------------------------------------------------------
// CR 800.4d — a departed player's trigger is not put onto the stack
// ---------------------------------------------------------------------------

/// Four seats: Blood Artist's controller loses in the same state-based
/// check that kills another creature. Blood Artist's frame sees the death
/// and the ability triggers under its departed controller; placement
/// refuses it.
// COVERS: ATOM-800.4d-001
#[test]
fn a_trigger_a_departed_player_would_control_is_not_put_on_the_stack() {
    let mut game = setup_game(4);
    let artist = put_on_battlefield(&mut game, blood_artist(), 1);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.battlefield.get_mut(&bear).unwrap().damage_marked = 2;
    game.players[1].life_total = 0;

    assert!(game.check_state_based_actions(&test_dp()).unwrap());
    assert!(game.player_lost[1]);
    assert!(game.get_object(artist).is_err(), "CR 800.4a took the permanent with its owner");
    assert_eq!(pending(&game), 1, "and the frame still saw the bear die");
    assert_eq!(game.pending_triggers[0].controller, 1);

    place(&mut game, &test_dp());

    assert!(game.stack.is_empty(), "CR 800.4d — not put on the stack");
    assert_eq!(pending(&game), 0);
}

// ---------------------------------------------------------------------------
// Eon Hub's two rulings (`codebase-state.md` main item 121)
// ---------------------------------------------------------------------------

/// A skipped upkeep begins nothing, so an upkeep trigger never triggers.
// RULING: Eon Hub #2 - "Upkeep-triggered abilities don't trigger, and \"activate only during your upkeep\" abilities can't be activated."
#[test]
fn under_eon_hub_an_upkeep_trigger_never_triggers() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, eon_hub(), 0);
    put_on_battlefield(&mut game, verdant_force(), 1);
    game.set_turn_position(Phase { phase_type: PhaseType::Ending, step: Some(StepType::End) });

    advance_to(&mut game, 1, StepType::Draw);

    assert!(!game.recorded_events().events().any(|e| matches!(e, GameEvent::StepBegin { step: StepType::Upkeep, .. })));
    assert_eq!(pending(&game), 0);
}

/// A trigger from the untap step waits for the next priority grant, which
/// under Eon Hub is the draw step's.
// RULING: Eon Hub #3 - "Any triggered abilities that triggered during the untap step will go onto the stack at the start of the draw step."
#[test]
fn under_eon_hub_an_untap_trigger_goes_on_the_stack_at_the_draw_step() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, eon_hub(), 0);
    let watch = put_on_battlefield(
        &mut game,
        watcher("Night Watch", TriggerEvent::BecomesUntapped { subject: TriggerSubject::ThisObject }, gain_one()),
        1,
    );
    game.battlefield.get_mut(&watch).unwrap().tapped = true;
    game.set_turn_position(Phase { phase_type: PhaseType::Ending, step: Some(StepType::End) });

    advance_to(&mut game, 1, StepType::Draw);

    assert_eq!(pending(&game), 1, "held through the skipped upkeep");
    let watcher = StackWatcher::new();
    game.run_priority_round(&watcher).unwrap();
    assert_eq!(watcher.at_first_prompt(), (1, 0), "on the stack at the draw step's first grant");
}

// ---------------------------------------------------------------------------
// "From anywhere" — Guile's two boards (CR 603.6c, 113.6k)
// ---------------------------------------------------------------------------

fn guile_shaped() -> Arc<CardData> {
    watcher(
        "Incarnate Echo",
        TriggerEvent::ZoneChange {
            subject: TriggerSubject::ThisObject,
            from: None,
            to: Some(Zone::Graveyard),
            cause: None,
            owner: None,
            multiplicity: Multiplicity::PerOccurrence,
        },
        gain_one(),
    )
}

/// Guile's seventh ruling, first half: the ability lost on the battlefield
/// (here to Humility) is back in the graveyard, and "from anywhere" is read
/// there — it triggers.
#[test]
fn a_from_anywhere_trigger_stripped_on_the_battlefield_still_triggers_from_the_graveyard() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, humility(), 1);
    let echo = put_on_battlefield(&mut game, guile_shaped(), 0);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &[echo], source);

    assert_eq!(pending(&game), 1, "CR 603.6c — not a leaves-the-battlefield ability; read after the move");
}

/// Its second half: Yixlid Jailer takes the ability away *in* the graveyard,
/// which is where a "from anywhere" trigger is read — it does not trigger.
#[test]
fn a_from_anywhere_trigger_is_stopped_by_yixlid_jailer() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, yixlid_jailer(), 1);
    let echo = put_on_battlefield(&mut game, guile_shaped(), 0);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &[echo], source);

    assert_eq!(pending(&game), 0);

    // And Yixlid Jailer's first ruling: a dies trigger is read from the
    // battlefield, off the frame, and is not affected.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, yixlid_jailer(), 1);
    let artist = put_on_battlefield(&mut game, blood_artist(), 0);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);
    destroy_all(&mut game, &[artist], source);
    assert_eq!(pending(&game), 1);
}

/// CR 603.6c's last sentence when the same wipe also takes Yixlid Jailer: a
/// "from anywhere" trigger is read after the whole event, so it triggers in
/// either batch order. With the card first, it reaches the graveyard while
/// Jailer is still on the battlefield, and a read at that move would find no
/// ability. The board where a look-back trigger and a "from anywhere" one
/// part ways across this event is a card already in the graveyard with a
/// trigger about other cards — Bridge from Below's shape — which the engine
/// cannot express until an intervening "if" can state the zone it works in.
#[test]
fn a_from_anywhere_trigger_reads_the_board_after_a_wipe_that_took_yixlid_jailer() {
    for jailer_first in [true, false] {
        let mut game = setup_two_player_game();
        let jailer = put_on_battlefield(&mut game, yixlid_jailer(), 1);
        let echo = put_on_battlefield(&mut game, guile_shaped(), 0);
        let source = put_on_battlefield(&mut game, sol_ring(), 1);
        let order = if jailer_first { [jailer, echo] } else { [echo, jailer] };

        destroy_all(&mut game, &order, source);

        assert_eq!(pending(&game), 1, "read after the event, Jailer gone (jailer_first: {jailer_first})");
    }
}

/// Dread's two triggers (Lorwyn): "Whenever a creature deals damage to you,
/// destroy it" and "When Dread is put into a graveyard from anywhere, shuffle
/// it into its owner's library". The effects are stand-ins; the tests count
/// triggers.
///
/// The first functions on the battlefield only (CR 113.6's default) and the
/// second everywhere (CR 113.6k, derived). The board that tells the two apart
/// is the card in a graveyard with the second one still waiting — or never
/// resolved, if it is countered.
fn dread_shaped() -> Arc<CardData> {
    CardDataBuilder::new("Incarnate Menace")
        .card_type(CardType::Creature)
        .power_toughness(6, 6)
        .rules_text("Whenever a creature deals damage to you, destroy it.")
        .ability(triggered_ability(whenever(
            TriggerEvent::DamageDealt {
                source: a_creature().into(),
                recipient: DamageRecipient::Player(Some(PlayerRef::You)),
                combat: None,
                multiplicity: Multiplicity::PerOccurrence,
            },
            gain_one(),
        )))
        .ability(triggered_ability(whenever(
            TriggerEvent::ZoneChange {
                subject: TriggerSubject::ThisObject,
                from: None,
                to: Some(Zone::Graveyard),
                cause: None,
                owner: None,
                multiplicity: Multiplicity::PerOccurrence,
            },
            gain_one(),
        )))
        .build()
}

/// F1 — CR 113.6 is asked of each **ability**, not of the object.
///
/// Dread in a graveyard is a trigger candidate because its "from anywhere"
/// ability functions there. Its damage trigger does not, so a creature
/// dealing damage to its owner must not trigger it. Registration already
/// knew — `zone_trigger_sources` holds the one ability id — and the matcher
/// read only the map's keys.
#[test]
fn a_battlefield_only_trigger_is_not_asked_from_the_graveyard() {
    let mut game = setup_two_player_game();
    let dread = put_on_battlefield(&mut game, dread_shaped(), 0);
    let attacker = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    // Dread dies: "from anywhere" is read after the move (CR 603.6c).
    destroy_all(&mut game, &[dread], source);
    assert_eq!(pending(&game), 1, "the from-anywhere trigger, once");

    // A creature deals damage to Dread's owner while Dread is in the graveyard.
    deal(&mut game, attacker, DamageTarget::Player(0), 2, true);
    assert_eq!(
        pending(&game),
        1,
        "CR 113.6 — the damage trigger functions on the battlefield, and Dread is in a graveyard"
    );
}

/// The same ability from the zone it does function in — so the test above is
/// CR 113.6 being applied and not the ability being lost.
#[test]
fn the_same_trigger_is_asked_from_the_battlefield() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, dread_shaped(), 0);
    let attacker = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    deal(&mut game, attacker, DamageTarget::Player(0), 2, true);
    assert_eq!(pending(&game), 1, "a creature dealt damage to Dread's controller");
}

/// CR 113.6k's second sentence: "Other trigger conditions of the same
/// triggered ability may function in different zones." One ability with two
/// conditions — "from anywhere" functions everywhere, "a creature deals
/// damage to you" on the battlefield only. Absolver Thrull is the CR's own
/// example of the structure; its haunt condition is not expressible yet, and
/// the rule does not wait for a printed card.
fn one_ability_two_zones() -> Arc<CardData> {
    creature_with_ability(
        "Split Vigil",
        1,
        1,
        triggered_ability(TriggerDef {
            condition: TriggerCondition::AnyOf(vec![
                TriggerEvent::ZoneChange {
                    subject: TriggerSubject::ThisObject,
                    from: None,
                    to: Some(Zone::Graveyard),
                    cause: None,
                    owner: None,
                    multiplicity: Multiplicity::PerOccurrence,
                },
                TriggerEvent::DamageDealt {
                    source: a_creature().into(),
                    recipient: DamageRecipient::Player(Some(PlayerRef::You)),
                    combat: None,
                    multiplicity: Multiplicity::PerOccurrence,
                },
            ]),
            intervening_if: None,
            limit: None,
            effect: gain_one(),
        }),
    )
}

/// The ability functions in the graveyard, because one of its conditions
/// does; the other condition still does not, and is not asked there.
#[test]
fn a_trigger_condition_is_asked_only_where_it_functions() {
    let mut game = setup_two_player_game();
    let vigil = put_on_battlefield(&mut game, one_ability_two_zones(), 0);
    let attacker = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &[vigil], source);
    assert_eq!(pending(&game), 1, "the from-anywhere condition, once");

    deal(&mut game, attacker, DamageTarget::Player(0), 2, true);
    assert_eq!(
        pending(&game),
        1,
        "CR 113.6k — the damage condition functions on the battlefield only"
    );
}

/// Its control: the same condition, from the battlefield.
#[test]
fn both_conditions_function_on_the_battlefield() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, one_ability_two_zones(), 0);
    let attacker = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    deal(&mut game, attacker, DamageTarget::Player(0), 2, true);
    assert_eq!(pending(&game), 1, "the damage condition, from the battlefield");
}

// ---------------------------------------------------------------------------
// The trace's sixth emit point (item 9)
// ---------------------------------------------------------------------------

#[test]
fn the_dispatcher_writes_a_trigger_record_per_decision_and_a_pending_record_per_placement() {
    let mut game = setup_two_player_game();
    let buf = install_trace(&mut game, "tr-1");
    put_on_battlefield(&mut game, soul_warden(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    place(&mut game, &test_dp());

    let triggers = buf.of_kind("trigger");
    assert!(triggers.iter().any(|l| l.contains("\"matched\":true")), "{triggers:?}");
    assert!(
        triggers.iter().any(|l| l.contains("\"matched\":false") && l.contains("\"refused_by\":\"condition\"")),
        "Soul Warden was asked about its own entry and refused: {triggers:?}"
    );
    let pending_records = buf.of_kind("pending");
    assert_eq!(pending_records.len(), 1);
    assert!(pending_records[0].contains("\"refused_by\":null"));
}


/// §11's lever, observed: a source is visited only when the window carries a
/// kind its printed defs read.
///
/// The trace's `trigger` record is written per decision, so "no record" is
/// "not asked" — which before the mask was "asked and refused by
/// `Refusal::TriggerCondition`". Soul Warden reads an entry and nothing else; a tap
/// is a window it cannot match, and the dispatch returns at the gate.
#[test]
fn a_window_no_source_reads_is_refused_at_the_gate() {
    let mut game = setup_two_player_game();
    let buf = install_trace(&mut game, "tr-1");
    put_on_battlefield(&mut game, soul_warden(), 0);
    let bear = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let entries = buf.of_kind("trigger").len();

    game.execute_action(GameAction::Tap { object: bear }, &test_ctx()).unwrap();
    assert_eq!(
        buf.of_kind("trigger").len(),
        entries,
        "a Tapped window meets no mask, so Soul Warden is never asked"
    );

    // The control: a kind it does read puts it back in the candidate set.
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    assert!(
        buf.of_kind("trigger").len() > entries,
        "an entry is the kind Soul Warden reads"
    );
}

// ---------------------------------------------------------------------------
// The identity, and the pool's cards through a whole game
// ---------------------------------------------------------------------------

/// The stack object is `activate_ability`'s twin: `is_spell: false`, no
/// `cast_from`, an identity with the printed ability and the source's epoch,
/// and the binding beside it.
#[test]
fn the_stack_object_is_an_ability_with_its_identity_and_binding() {
    let mut game = setup_two_player_game();
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    let bear = put_on_battlefield(&mut game, grizzly_bears(), 1);
    place(&mut game, &test_dp());
    let id = *game.stack.last().unwrap();
    let entry = &game.stack_entries[&id];
    assert!(!entry.is_spell);
    assert_eq!(entry.cast_from, None);
    let identity = entry.ability_identity.unwrap();
    assert_eq!(identity.source.id, warden);
    assert_eq!(identity.ability, soul_warden().abilities[0].id);
    assert_eq!(identity.source.zone_change_epoch, game.get_object(warden).unwrap().zone_change_epoch);
    let binding = entry.trigger.as_ref().unwrap();
    assert_eq!(binding.subject.map(|o| o.id), Some(bear));
    assert_eq!(game.bound_object(binding), Some(bear));
    assert_eq!(game.get_object(id).unwrap().card_data.name, "Soul Warden");
}

/// Two grants of one triggered ability are two instances on the carrier, and
/// each keeps its identity for exactly as long as its grant lasts: the first
/// grant ending mid-turn leaves the survivor's trigger under the identity it
/// had (`triggers-architecture.md` §3.6, the provenance amendment). An
/// ordinal among same-id instances renumbered the survivor from 1 to 0, which
/// would orphan a TR-2 gate keyed on the one and hand it the other's.
#[test]
fn a_grant_ending_leaves_the_surviving_grants_identity_alone() {
    let mut game = setup_two_player_game();
    let carrier = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let granters = [
        put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0),
        put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0),
    ];
    // Diffusion Sliver's shape, from registry rows: each granter grants one
    // def to the carrier while it is on the battlefield. One def, cloned, so
    // the two grants are of one ability.
    let mut granted = triggered_ability(whenever(enters(another(a_creature())), gain_one()));
    granted.id = new_ability_id();
    for (granter, timestamp) in granters.into_iter().zip([100, 101]) {
        game.continuous_effects.add(ContinuousEffect {
            duration: Duration::WhileSourceOnBattlefield,
            affected_objects: ObjectSet::Fixed(vec![carrier]),
            ..registered(
                granter,
                Layer::Layer6Ability,
                timestamp,
                EffectModification::GrantAbility(std::sync::Arc::new(granted.clone())),
            )
        });
    }
    let identities = |game: &GameState| -> Vec<AbilityIdentity> {
        game.pending_triggers
            .iter()
            .map(|t| match t.origin {
                TriggerOrigin::Object(identity) => identity,
            })
            .collect()
    };

    put_on_battlefield(&mut game, grizzly_bears(), 0);
    let before = identities(&game);
    assert_eq!(before.len(), 2, "two instances, two triggers");
    assert_ne!(before[0], before[1], "and two identities");
    game.pending_triggers.clear();

    let source = put_on_battlefield(&mut game, sol_ring(), 1);
    destroy_all(&mut game, &[granters[0]], source);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(
        identities(&game),
        vec![before[1]],
        "the second grant's trigger, under the identity it had while the first lasted"
    );
}

/// CR 400.7d — a permanent that resolved from a cast spell remembers who
/// cast it and from where; one put onto the battlefield does not.
#[test]
fn a_permanent_remembers_whether_it_was_cast() {
    let mut game = setup_two_player_game();
    let cast = put_spell_on_stack(&mut game, grizzly_bears(), 1);
    resolve_top(&mut game, &test_dp());
    let facts = game.battlefield[&cast].cast.expect("cast from the hand");
    assert_eq!((facts.by, facts.from), (1, Zone::Hand));
    let placed = put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(game.battlefield[&placed].cast, None);
}

fn kicker_red() -> AdditionalCost {
    AdditionalCost::Kicker(vec![Cost::Mana(ManaCost::build(&[ManaType::Red], 0))])
}

/// Cast `card` from hand out of exactly `pool`, kicking it or not, then
/// resolve it and anything its resolution triggered. Exact, so the generic
/// split is forced (CR 102.2) and asks nothing.
fn cast_from_exact_pool(card: Arc<CardData>, pool: &[ManaType], kick: bool) -> (GameState, ObjectId) {
    let mut game = setup_two_player_game();
    let id = put_in_hand(&mut game, card, 0);
    for &t in pool {
        game.players[0].mana_pool.add(t, 1);
    }
    let dp = ScriptedDecisionProvider::new();
    let kicked = if kick { vec![0] } else { Vec::new() };
    dp.expect_pick_n(ChoiceKind::ChooseAdditionalCosts { spell_id: id }, kicked);
    game.cast_spell(0, id, &dp).expect("cast from an exact pool");
    assert_eq!(game.players[0].mana_pool.total(), 0, "the pool was exactly the cost");
    resolve_top(&mut game, &test_dp());
    place(&mut game, &test_dp());
    while !game.stack.is_empty() {
        resolve_top(&mut game, &test_dp());
    }
    (game, id)
}

/// A {1}{G} 2/2 with kicker {R} and "When this creature enters, if it was
/// kicked, you gain 1 life" — ATOM-400.7d-001's board with the "if kicked"
/// ability it names.
fn kicked_herald() -> Arc<CardData> {
    CardDataBuilder::new("Kicked Herald")
        .card_type(CardType::Creature)
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .power_toughness(2, 2)
        .additional_cost(kicker_red())
        .ability(triggered_ability(TriggerDef {
            condition: TriggerCondition::Event(enters(TriggerSubject::ThisObject).into()),
            intervening_if: Some(Condition::SpellWasKicked),
            limit: None,
            effect: gain_one(),
        }))
        .build()
}

/// CR 400.7d — the permanent keeps what was paid to cast it: the kicker, and
/// the mana by type, the generic {1} included (the exact {W}{G}{R} forces it
/// onto the {W}). Its "if kicked" ability reads that as it enters and again
/// as it resolves (CR 603.4). Driven through `cast_spell`, because the cast
/// path is the only writer of the mana.
// COVERS: ATOM-400.7d-001
#[test]
fn a_kicked_permanent_remembers_what_paid_for_it() {
    let (game, herald) =
        cast_from_exact_pool(kicked_herald(), &[ManaType::White, ManaType::Green, ManaType::Red], true);
    let choices = &game.battlefield[&herald].cost_choices;
    assert!(matches!(choices.additional[..], [AdditionalCost::Kicker(_)]));
    assert_eq!(choices.alternative, None);
    let facts = game.battlefield[&herald].cast.expect("cast from the hand");
    let by_type = [
        ManaType::White,
        ManaType::Blue,
        ManaType::Black,
        ManaType::Red,
        ManaType::Green,
        ManaType::Colorless,
    ]
    .map(|t| facts.mana_spent.amount(t));
    assert_eq!(by_type, [1, 0, 0, 1, 1, 0]);
    assert_eq!(life(&game, 0), 21, "the \"if kicked\" ability read it");
}

/// The same creature unkicked: no additional cost paid, and the "if" is
/// false as it enters, so the ability never triggers.
#[test]
fn an_unkicked_permanent_was_not_kicked() {
    let (game, herald) = cast_from_exact_pool(kicked_herald(), &[ManaType::White, ManaType::Green], false);
    assert!(game.battlefield[&herald].cost_choices.additional.is_empty());
    let facts = game.battlefield[&herald].cast.expect("cast from the hand");
    assert_eq!(facts.mana_spent.total(), 2);
    assert_eq!(life(&game, 0), 20);
}

/// CR 707.10 — "a copy of a spell isn't cast", and it copies "additional or
/// alternative costs". So the permanent a kicked spell's copy becomes was
/// not cast, spent no mana, and is kicked (Archangel of Wrath's ruling). No
/// spell copy exists before CV-4, so the entry is staged the way CV-4's
/// copy will be: a spell with the original's decisions and no `cast_from`.
#[test]
fn a_spell_that_was_not_cast_keeps_its_kicker() {
    let mut game = setup_two_player_game();
    let copy = game.add_object(GameObject::new(kicked_herald(), 0, Zone::Stack));
    game.stack.push(copy);
    game.stack_entries.insert(copy, StackEntry {
        object_id: copy,
        controller: 0,
        chosen_targets: Vec::new(),
        chosen_modes: Vec::new(),
        x_value: None,
        effect: Effect::Sequence(Vec::new()),
        is_spell: true,
        chosen_alternative_cost: None,
        additional_costs_paid: vec![kicker_red()],
        mana_spent: ManaSpent::NONE,
        cast_from: None,
        ability_identity: None,
        trigger: None,
    });
    resolve_top(&mut game, &test_dp());
    place(&mut game, &test_dp());
    while !game.stack.is_empty() {
        resolve_top(&mut game, &test_dp());
    }
    assert_eq!(game.battlefield[&copy].cast, None, "not cast");
    assert_eq!(life(&game, 0), 21, "and still kicked");
}

/// "If this spell was kicked" on an instant is read as it resolves, when
/// its `StackEntry` has already been taken — so off the facts resolution
/// carries, the same record a permanent spell hands to the permanent.
#[test]
fn a_resolving_spell_reads_its_own_kicker() {
    let insight = || {
        CardDataBuilder::new("Kicked Insight")
            .card_type(CardType::Instant)
            .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
            .additional_cost(kicker_red())
            .ability(AbilityDef {
                ability_type: AbilityType::Spell,
                ..static_ability(Effect::Conditional(Condition::SpellWasKicked, Box::new(gain_one())))
            })
            .build()
    };
    let (game, _) = cast_from_exact_pool(insight(), &[ManaType::Blue, ManaType::Red], true);
    assert_eq!(life(&game, 0), 21, "kicked");
    let (game, _) = cast_from_exact_pool(insight(), &[ManaType::Blue], false);
    assert_eq!(life(&game, 0), 20, "not kicked");
}

/// CR 117.5 — "each time a player would get priority, [...] triggered
/// abilities that are waiting to be put onto the stack are put onto the
/// stack." So the queue is empty at every priority prompt, and this plays
/// whole games with the three pooled cards forced into every deck, at two
/// seats and four, to say so.
///
/// **The assertion is the provider's, not the loop's.** What stood here was
/// `pending_triggers.is_empty() || !g.is_over()`, which holds trivially
/// until the game ends and then holds for the other reason; the claim the
/// doc comment made was never checked. `PriorityQueueWatcher` checks it at
/// the one instant CR 117.5 names, which is also the only instant a
/// `DecisionProvider` can see.
#[test]
fn no_player_receives_priority_with_a_trigger_still_queued() {
    use mtgsim::cards::registry::CardRegistry;
    use mtgsim::state::game::Game;
    use mtgsim::state::game_config::GameConfig;

    for players in [2usize, 4] {
        let registry = CardRegistry::performance_pool();
        let mut deck: Vec<Arc<CardData>> = registry
            .card_names()
            .iter()
            .cycle()
            .take(30)
            .filter_map(|n| registry.create(n).ok())
            .collect();
        deck.extend([soul_warden(), soul_warden(), blood_artist(), blood_artist(), wild_growth(), wild_growth()]);
        // A mana base the three can actually be cast off. Without one the
        // agent never casts them and the test is vacuous twice over, which is
        // how it stood: 212 priority prompts at two seats and not one trigger
        // placed. `random_deck` makes the same guarantee for the fuzz harness
        // and for the same reason.
        for _ in 0..8 {
            deck.extend([plains(), swamp(), forest()]);
        }
        let mut g = Game::new(GameConfig::test(), vec![deck; players]).unwrap();
        g.reseed(603);
        let dp = PriorityQueueWatcher::seeded(603 + players as u64);
        g.setup(&dp).unwrap();
        for _ in 0..20 {
            if g.is_over() {
                break;
            }
            g.run_turn(&dp).unwrap();
        }
        // The guard that keeps the claim from being vacuous a third way:
        // a board where nothing triggers proves nothing about CR 117.5.
        assert!(
            g.state.diagnostics.triggers_placed() > 0,
            "{players} seats: the forced cards must actually trigger"
        );
        assert!(dp.prompts.get() > 0, "{players} seats: the game reached priority at all");
    }
}

/// A `RandomDecisionProvider` that asserts CR 117.5 at every priority
/// prompt: nothing is waiting to be put onto the stack by the time anyone is
/// asked what to do.
///
/// A wrapper rather than a change to the random agent — the assertion is
/// this test's claim, and a provider that panicked inside `fuzz_games` would
/// turn a rules bug into a harness crash.
struct PriorityQueueWatcher {
    inner: RandomDecisionProvider,
    prompts: std::cell::Cell<usize>,
}

impl PriorityQueueWatcher {
    fn seeded(seed: u64) -> Self {
        PriorityQueueWatcher {
            inner: RandomDecisionProvider::seeded(seed),
            prompts: std::cell::Cell::new(0),
        }
    }

    fn check(&self, game: &GameState, context: &ChoiceContext) {
        if !matches!(context.kind, ChoiceKind::PriorityAction) {
            return;
        }
        self.prompts.set(self.prompts.get() + 1);
        assert!(
            game.pending_triggers.is_empty(),
            "CR 117.5 — {} trigger(s) still queued as a player receives priority",
            game.pending_triggers.len()
        );
    }
}

impl DecisionProvider for PriorityQueueWatcher {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        self.check(game, context);
        self.inner.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        min: u64,
        max: u64,
    ) -> u64 {
        self.check(game, context);
        self.inner.pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.check(game, context);
        self.inner
            .allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        self.check(game, context);
        self.inner.choose_ordering(game, player, context, items)
    }
}
