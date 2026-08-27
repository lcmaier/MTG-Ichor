//! Phase RB integration tests: the CR 616.1 pipeline and its first consumers.
//!
//! RA proved the *record* was complete. RB is the first phase where a
//! replacement effect actually changes what happens, so these tests assert on
//! the modified outcome **and** on the pipeline having been entered — a
//! replacement test that passes because the effect never fired is the exact
//! failure this phase is prone to (`replacement-architecture.md` §10).

use std::sync::Arc;

use mtgsim::cards::phase_rb_cards::kalitas_traitor_of_ghet;
use mtgsim::engine::actions::{ActionContext, DestructionSource, GameAction, ZoneChangeCause};
use mtgsim::events::event::{DamageTarget, GameEvent};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::engine::layers::types::{EffectModification, Layer};
use mtgsim::test_support::{
    pass_turn, place_bare, put_on_battlefield, registered, set_attacking, set_blocked_by,
    set_blocking, setup_game, setup_two_player_game, stock_libraries, test_ctx, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::types::effects::{
    AffectedSet, CounterType, Effect, EffectRecipient, PermanentFilter, Primitive,
    SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

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

    let dp = ScriptedDecisionProvider::new();
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

// ---------------------------------------------------------------------------
// Item 5 — consumer 1: counters (CR 122.1c/d/h). No card text; 164 cards.
// ---------------------------------------------------------------------------

/// A creature whose printed static ability prevents its trip to the graveyard.
///
/// A probe, not a card. Its only job is to be a *second* candidate alongside a
/// finality counter so that CR 616.1's choice becomes reachable — which is the
/// first `DecisionProvider` call this engine has ever made about a replacement
/// effect.
fn graveyard_probe(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::ZoneChange {
                    from: Some(Zone::Battlefield),
                    to: Some(Zone::Graveyard),
                    cause: None,
                    object: None,
                },
                AffectedSet::SourceOnly,
                Rewrite::Prevent,
            ))),
        })
        .build()
}

/// Counters placed the way the engine places them, through the chokepoint.
fn add_counters(game: &mut GameState, id: ObjectId, counter: CounterType, n: u32) {
    game.execute_action(GameAction::AddCounters { object: id, counter, n }, &test_ctx())
        .unwrap();
}

/// The signed counter deltas in the log, as `(object, counter, added)`.
fn counter_changes(game: &GameState) -> Vec<(ObjectId, CounterType, i32)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::CountersChanged { object_id, counter, added } => {
                Some((*object_id, *counter, *added))
            }
            _ => None,
        })
        .collect()
}

// COVERS: ATOM-122.1d-001
#[test]
fn test_a_stun_counter_replaces_the_untap_with_removing_a_counter() {
    // > 122.1d ... "If a permanent with a stun counter on it would become
    // > untapped, instead remove a stun counter from it."
    //
    // The counter removal is the *substituted event*, which is why `Uses` has
    // no `CounterBacked` variant: modelling it as a spent use would have
    // written `BattlefieldEntity.counters` from inside the pipeline, invisible
    // to CR 614.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.battlefield.get_mut(&bear).unwrap().tapped = true;
    add_counters(&mut game, bear, CounterType::Stun, 2);

    game.execute_action(GameAction::Untap { object: bear }, &test_ctx()).unwrap();

    assert!(game.battlefield[&bear].tapped, "it did not untap");
    assert_eq!(game.battlefield[&bear].counter_count(CounterType::Stun), 1);
    assert!(
        !game.events.events().any(|e| matches!(e, GameEvent::Untapped { .. })),
        "CR 614.6 — a replaced event never happens, so nothing announced an untap"
    );
    assert_eq!(
        counter_changes(&game),
        vec![(bear, CounterType::Stun, 2), (bear, CounterType::Stun, -1)],
    );
}

// COVERS-PARTIAL: ATOM-122.1d-001
#[test]
fn test_the_last_stun_counter_comes_off_and_the_next_untap_works() {
    // "One **or more** counters create a single replacement effect", so two
    // counters are two events' worth of protection, not two applications to one
    // event — and once the last is gone the effect stops existing at gather
    // time, which is where CR 614.4 wants the question asked.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.battlefield.get_mut(&bear).unwrap().tapped = true;
    add_counters(&mut game, bear, CounterType::Stun, 1);

    game.execute_action(GameAction::Untap { object: bear }, &test_ctx()).unwrap();
    assert!(game.battlefield[&bear].tapped);
    assert_eq!(game.battlefield[&bear].counter_count(CounterType::Stun), 0);

    game.execute_action(GameAction::Untap { object: bear }, &test_ctx()).unwrap();
    assert!(!game.battlefield[&bear].tapped, "no counter left to spend");
}

// COVERS: ATOM-122.1c-001
#[test]
fn test_a_shield_counter_replaces_destruction_by_an_effect() {
    // > 122.1c ... "If this permanent would be destroyed as the result of an
    // > effect, instead remove a shield counter from it"
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let source = place_bare(&mut game, vanilla_creature(1, 1, &[]), 0);
    add_counters(&mut game, bear, CounterType::Shield, 1);

    game.execute_action(
        GameAction::Destroy { object: bear, source: DestructionSource::Effect(source) },
        &test_ctx(),
    )
    .unwrap();

    assert!(game.battlefield.contains_key(&bear));
    assert_eq!(game.battlefield[&bear].counter_count(CounterType::Shield), 0);
    assert!(zone_changes(&game).is_empty());
}

// COVERS-PARTIAL: ATOM-122.1c-001
#[test]
fn test_a_shield_counter_does_not_replace_a_state_based_destruction() {
    // "**As the result of an effect**" is CR 701.8b way 1 only. A shield
    // counter saves a creature from lethal damage through its *other* half —
    // the prevention effect, which stops the damage before 704.5g can ask — so
    // reading the clause loosely would have spent the counter twice on one
    // creature.
    //
    // Damage is marked directly here rather than dealt, precisely to reach
    // 704.5g with the prevention half bypassed.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    add_counters(&mut game, bear, CounterType::Shield, 1);
    game.battlefield.get_mut(&bear).unwrap().damage_marked = 5;

    let dp = ScriptedDecisionProvider::new();
    game.check_state_based_actions(&dp).unwrap();

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Graveyard);
    assert_eq!(
        game.battlefield.get(&bear).map(|e| e.counter_count(CounterType::Shield)),
        None,
        "and the counter was never spent"
    );
}

// COVERS: ATOM-122.1c-002
#[test]
fn test_a_shield_counter_prevents_damage_and_the_rider_removes_a_counter() {
    // > 122.1c ... "If damage would be dealt to this permanent, prevent that
    // > damage and remove a shield counter from it."
    //
    // `Prevent` plus a CR 615.5 rider, not an `Instead`: CR 615.13 lets
    // triggers fire on damage *being prevented*, so the engine has to know a
    // prevention happened rather than seeing a substituted event.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let bolt = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);
    add_counters(&mut game, bear, CounterType::Shield, 1);

    game.execute_action(
        GameAction::DealDamage {
            source: bolt,
            target: DamageTarget::Object(bear),
            amount: 3,
            is_combat: false,
        },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.battlefield[&bear].damage_marked, 0, "prevented");
    assert_eq!(game.battlefield[&bear].counter_count(CounterType::Shield), 0);
    assert!(
        !game.events.events().any(|e| matches!(e, GameEvent::DamageDealt { .. })),
        "CR 614.6 — the damage event never happened"
    );
}

#[test]
fn test_the_shield_rider_runs_after_the_event_it_rides_on() {
    // §4.1a's timing contract, asserted on the *event log* rather than the end
    // state, because the end state is identical either way. The rider is queued
    // during the CR 616.1 loop and resolved after the surviving event — so its
    // counter removal is the last thing in the log, not the first.
    //
    // The event here is prevented entirely, which is the sharper case: CR
    // 615.12 makes a rider unconditional once queued, so it still runs.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let bolt = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);
    add_counters(&mut game, bear, CounterType::Shield, 1);
    let before = game.events.len();

    game.execute_action(
        GameAction::DealDamage {
            source: bolt,
            target: DamageTarget::Object(bear),
            amount: 3,
            is_combat: false,
        },
        &test_ctx(),
    )
    .unwrap();

    let after: Vec<String> =
        game.events.events().skip(before).map(|e| format!("{e:?}")).collect();
    assert_eq!(after.len(), 1, "one event: the rider's counter removal, got {after:?}");
    assert!(after[0].starts_with("CountersChanged"), "got {:?}", after[0]);
}

// COVERS: ATOM-122.1h-001
#[test]
fn test_a_finality_counter_exiles_instead_of_the_graveyard() {
    // > 122.1h ... "If this permanent would be put into a graveyard from the
    // > battlefield, exile it instead."
    //
    // Any cause, not just destruction — and the counter is *not* removed:
    // 122.1h does not say to, and CR 122.2 ends its counters when it leaves.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    add_counters(&mut game, bear, CounterType::Finality, 1);

    game.change_zone(bear, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Exile);
    assert_eq!(
        zone_changes(&game),
        vec![(bear, Zone::Battlefield, Zone::Exile, ZoneChangeCause::Exiled)],
        "one event, the modified one — CR 614.6"
    );
}

#[test]
fn test_a_finality_counter_catches_a_destruction_through_the_inner_zone_change() {
    // The two-event shape of `Destroy` earning its keep. A finality counter
    // watches the graveyard trip, not the destruction, so it can only see a
    // destroyed creature because `GameAction::Destroy`'s performer proposes the
    // inner `ZoneChange` through the pipeline — with a *fresh* applied set,
    // since a zone change is a different kind of event from a destruction
    // (§3.2d containment).
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let source = place_bare(&mut game, vanilla_creature(1, 1, &[]), 0);
    add_counters(&mut game, bear, CounterType::Finality, 1);

    game.execute_action(
        GameAction::Destroy { object: bear, source: DestructionSource::Effect(source) },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Exile);
}

// COVERS: ATOM-616.1-001
#[test]
fn test_two_replacements_on_one_event_prompt_the_affected_controller() {
    // CR 616.1 — "the affected object's controller ... chooses one to apply".
    // A finality counter and a printed `Prevent` both want this creature's trip
    // to the graveyard, so for the first time in this engine's life a
    // `DecisionProvider` is asked which replacement to apply.
    let mut game = setup_two_player_game();
    let probe = put_on_battlefield(&mut game, graveyard_probe("Graveyard Probe"), 0);
    add_counters(&mut game, probe, CounterType::Finality, 1);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::ChooseReplacementEffect { affected_object: None },
        vec![0],
    );
    let ctx = ActionContext::new(&dp);
    game.change_zone(probe, Zone::Graveyard, ZoneChangeCause::Sacrificed, &ctx)
        .unwrap();

    // Index 0 is the printed ability: `gather` reads a permanent's abilities
    // before its counters, and both come before the registry.
    assert!(game.battlefield.contains_key(&probe), "the `Prevent` was chosen");
    assert!(dp.is_empty(), "the prompt was consumed");
}

// COVERS-PARTIAL: ATOM-616.1-001
#[test]
fn test_choosing_the_other_replacement_takes_the_other_branch() {
    // The same board, the other index. Two tests rather than one because a
    // choice test that only ever picks index 0 cannot tell a real prompt from a
    // hard-coded first candidate.
    let mut game = setup_two_player_game();
    let probe = put_on_battlefield(&mut game, graveyard_probe("Graveyard Probe"), 0);
    add_counters(&mut game, probe, CounterType::Finality, 1);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::ChooseReplacementEffect { affected_object: None },
        vec![1],
    );
    let ctx = ActionContext::new(&dp);
    game.change_zone(probe, Zone::Graveyard, ZoneChangeCause::Sacrificed, &ctx)
        .unwrap();

    assert_eq!(game.get_object(probe).unwrap().zone, Zone::Exile, "the counter was chosen");
}

// COVERS: ATOM-616.1f-001
// COVERS: ATOM-616.2-001
#[test]
fn test_the_loop_re_gathers_and_the_second_effect_applies_to_the_modified_event() {
    // CR 616.1f — "once the chosen effect has been applied, this process is
    // repeated ... until there are no more left to apply" — and CR 616.2, "a
    // replacement effect can become applicable to an event as the result of
    // another replacement effect that modifies the event".
    //
    // Both at once: an exile-watcher that could not have applied to the
    // original graveyard-bound event becomes applicable only after the finality
    // counter has rewritten the destination.
    let mut game = setup_two_player_game();
    let probe = put_on_battlefield(&mut game, exile_watcher("Exile Watcher"), 0);
    add_counters(&mut game, probe, CounterType::Finality, 1);

    game.change_zone(probe, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();

    assert!(
        game.battlefield.contains_key(&probe),
        "the finality counter redirected to exile, and only then did the \
         exile-watcher have anything to prevent"
    );
    assert!(zone_changes(&game).is_empty());
    assert_eq!(
        game.battlefield[&probe].counter_count(CounterType::Finality),
        1,
        "122.1h removes no counter"
    );
}

// COVERS: ATOM-614.5-001
#[test]
fn test_a_replacement_effect_does_not_apply_to_its_own_output() {
    // CR 614.5 — "once a replacement effect has been applied to an event, it
    // can't be applied again to the resulting events". A finality counter
    // rewrites Battlefield→Graveyard into Battlefield→Exile; without the
    // applied set the re-gather would find it applicable again only if its
    // pattern still matched, so the sharper probe is an effect whose *output
    // still matches its own pattern*.
    let mut game = setup_two_player_game();
    let probe = put_on_battlefield(&mut game, self_matching_probe("Ouroboros Probe"), 0);

    game.change_zone(probe, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();

    // One application: Graveyard → Exile. A second would have sent it to
    // Exile → Exile, which `perform_action` treats as a no-op, so the
    // observable failure is the iteration cap rather than a wrong zone.
    assert_eq!(game.get_object(probe).unwrap().zone, Zone::Exile);
    assert_eq!(
        zone_changes(&game),
        vec![(probe, Zone::Battlefield, Zone::Exile, ZoneChangeCause::Exiled)],
    );
}

/// A creature that prevents its own trip to *exile*.
///
/// Only reachable as a second application: nothing sends this creature to exile
/// on its own, so it can fire only after some other replacement has rewritten
/// the destination — which is exactly CR 616.2.
fn exile_watcher(name: &str) -> Arc<CardData> {
    replacement_creature(
        name,
        ReplacementDef::new(
            EventPattern::ZoneChange {
                from: Some(Zone::Battlefield),
                to: Some(Zone::Exile),
                cause: None,
                object: None,
            },
            AffectedSet::SourceOnly,
            Rewrite::Prevent,
        ),
    )
}

/// A creature whose replacement rewrites any battlefield exit into an exile —
/// so its **own output still matches its own pattern**.
///
/// The sharp probe for CR 614.5: an effect whose output no longer matches would
/// terminate whether or not the applied set worked.
fn self_matching_probe(name: &str) -> Arc<CardData> {
    replacement_creature(
        name,
        ReplacementDef::new(
            EventPattern::ZoneChange {
                from: Some(Zone::Battlefield),
                to: None,
                cause: None,
                object: None,
            },
            AffectedSet::SourceOnly,
            Rewrite::Instead(GameActionTemplate::ZoneChangeTo {
                to: Zone::Exile,
                cause: ZoneChangeCause::Exiled,
            }),
        ),
    )
}

/// A 2/2 whose only ability is one static replacement effect.
fn replacement_creature(name: &str, def: ReplacementDef) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(def)),
        })
        .build()
}

// ---------------------------------------------------------------------------
// CR 616.1 at four players
// ---------------------------------------------------------------------------

/// A `DecisionProvider` that records which player was asked, then delegates.
///
/// `ScriptedDecisionProvider` validates the *kind* of every decision but not
/// its subject, and CR 616.1's whole content is *who* chooses. At two players
/// "the affected object's controller" and "the player who isn't active" are the
/// same answer, so the assertion only means something at four.
struct RecordingProvider<'a> {
    inner: &'a ScriptedDecisionProvider,
    asked: &'a std::cell::RefCell<Vec<PlayerId>>,
}

impl DecisionProvider for RecordingProvider<'_> {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &mtgsim::ui::choice_types::ChoiceContext,
        options: &[mtgsim::ui::choice_types::ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        self.asked.borrow_mut().push(player);
        self.inner.pick_n(game, player, context, options, bounds)
    }
    fn pick_number(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &mtgsim::ui::choice_types::ChoiceContext,
        min: u64,
        max: u64,
    ) -> u64 {
        self.asked.borrow_mut().push(player);
        self.inner.pick_number(game, player, context, min, max)
    }
    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &mtgsim::ui::choice_types::ChoiceContext,
        total: u64,
        buckets: &[mtgsim::ui::choice_types::ChoiceOption],
        mins: &[u64],
        maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.asked.borrow_mut().push(player);
        self.inner.allocate(game, player, context, total, buckets, mins, maxs)
    }
    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &mtgsim::ui::choice_types::ChoiceContext,
        items: &[mtgsim::ui::choice_types::ChoiceOption],
    ) -> Vec<usize> {
        self.asked.borrow_mut().push(player);
        self.inner.choose_ordering(game, player, context, items)
    }
}

// COVERS-PARTIAL: ATOM-616.1-001
#[test]
fn test_the_chooser_is_the_affected_objects_controller_not_the_active_player() {
    // CR 616.1 — "the affected object's controller (or its owner if it has no
    // controller) or the affected player chooses". Not the controller of either
    // *effect*, and not the active player.
    //
    // Four players, because APNAP with one nonactive player is the same answer
    // as no APNAP at all (§10). Player 0 is active, the creature is player 2's,
    // and the prompt has to reach player 2.
    let mut game = setup_game(4);
    game.active_player = 0;
    let probe = put_on_battlefield(&mut game, graveyard_probe("Graveyard Probe"), 2);
    add_counters(&mut game, probe, CounterType::Finality, 1);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::ChooseReplacementEffect { affected_object: None },
        vec![1],
    );
    let asked = std::cell::RefCell::new(Vec::new());
    let recording = RecordingProvider { inner: &dp, asked: &asked };
    let ctx = ActionContext::new(&recording);
    game.change_zone(probe, Zone::Graveyard, ZoneChangeCause::Sacrificed, &ctx)
        .unwrap();

    assert_eq!(*asked.borrow(), vec![2], "player 2 controls it; player 0 is active");
    assert_eq!(game.get_object(probe).unwrap().zone, Zone::Exile);
}

// COVERS-PARTIAL: ATOM-616.1-001
#[test]
fn test_simultaneous_choices_are_offered_in_apnap_order() {
    // CR 616.1's last sentence: "if two or more players have to make these
    // choices at the same time, choices are made in APNAP order (see rule
    // 101.4)". One batch, three creatures belonging to three different players,
    // each with two applicable replacements — so three players must choose at
    // once, and the order is active-player-first then turn order.
    //
    // The batch is built player-3-first on purpose: if the pipeline simply
    // followed batch order the recorded sequence would be [3, 1, 2].
    let mut game = setup_game(4);
    game.active_player = 1;

    let mut probes = Vec::new();
    for owner in [3usize, 1, 2] {
        let id = put_on_battlefield(&mut game, graveyard_probe("Graveyard Probe"), owner);
        add_counters(&mut game, id, CounterType::Finality, 1);
        probes.push(id);
    }

    let dp = ScriptedDecisionProvider::new();
    for _ in 0..3 {
        dp.expect_pick_n(
            ChoiceKind::ChooseReplacementEffect { affected_object: None },
            vec![1],
        );
    }
    let asked = std::cell::RefCell::new(Vec::new());
    let recording = RecordingProvider { inner: &dp, asked: &asked };
    let ctx = ActionContext::new(&recording);

    let batch: Vec<GameAction> = probes
        .iter()
        .map(|&object| GameAction::ZoneChange {
            object,
            from: Zone::Battlefield,
            to: Zone::Graveyard,
            cause: ZoneChangeCause::Sacrificed,
        })
        .collect();
    game.execute_actions(batch, &ctx).unwrap();

    assert_eq!(
        *asked.borrow(),
        vec![1, 2, 3],
        "active player first, then turn order — not the order the batch was built in"
    );
}

// COVERS-PARTIAL: ATOM-704.7-001
#[test]
fn test_batch_members_each_get_their_own_applied_set() {
    // §4.2, and Kalitas's printed ruling is what pins it: CR 614.5 is per
    // *event*, and batch members are separate events. **One** replacement
    // effect, three simultaneous deaths, three applications.
    //
    // The shape matters. An effect keyed per-permanent (a counter) cannot tell
    // a shared applied set from a per-event one, because each permanent's
    // effect has its own instance id either way — a first draft of this test
    // used counters and passed under a deliberately shared set. This one has a
    // single source with an `AffectedSet::Filter`, so a shared set exiles the
    // first creature and sends the other two to the graveyard.
    let mut game = setup_two_player_game();
    let _watcher = put_on_battlefield(&mut game, all_creatures_exile_watcher(), 0);
    let ids: Vec<ObjectId> = (0..3)
        .map(|_| place_bare(&mut game, vanilla_creature(2, 2, &[]), 0))
        .collect();

    let batch: Vec<GameAction> = ids
        .iter()
        .map(|&object| GameAction::ZoneChange {
            object,
            from: Zone::Battlefield,
            to: Zone::Graveyard,
            cause: ZoneChangeCause::Sacrificed,
        })
        .collect();
    game.execute_actions(batch, &test_ctx()).unwrap();

    for id in ids {
        assert_eq!(
            game.get_object(id).unwrap().zone,
            Zone::Exile,
            "one static replacement, applied once per death"
        );
    }
}

/// A creature with "if a creature would be put into a graveyard from the
/// battlefield, exile it instead" — Kalitas's shape without Kalitas's filter.
///
/// One source, an `AffectedSet::Filter` over other objects: the only shape in
/// which a batch-wide applied set is distinguishable from a per-event one.
fn all_creatures_exile_watcher() -> Arc<CardData> {
    replacement_creature(
        "Exile Warden",
        ReplacementDef::new(
            EventPattern::ZoneChange {
                from: Some(Zone::Battlefield),
                to: Some(Zone::Graveyard),
                cause: None,
                object: None,
            },
            AffectedSet::Filter { filter: PermanentFilter::ByType(CardType::Creature) },
            Rewrite::Instead(GameActionTemplate::ZoneChangeTo {
                to: Zone::Exile,
                cause: ZoneChangeCause::Exiled,
            }),
        ),
    )
}

// ---------------------------------------------------------------------------
// The untap step (CR 502.1, 703.4c) — the turn-based action stun counters ride
// ---------------------------------------------------------------------------

// COVERS: ATOM-502.3-001
// COVERS: ATOM-703.4c-001
#[test]
fn test_every_permanent_the_active_player_controls_untaps_as_one_event() {
    // CR 502.3 / 703.4c — "the active player determines which permanents they
    // control will untap. Then they untap them all simultaneously." Normally
    // all of them, and *simultaneously* is not decorative: the untap step is
    // one batch, so every `Untapped` event it emits carries one `BatchId`,
    // which is what CR 603.2c's "whenever one or more permanents untap" will
    // read.
    //
    // These two atoms were listed as ALREADY-IMPLEMENTED with no test since the
    // corpus was written. RB is what makes them worth having one: each untap is
    // now a replaceable proposal, so "all of them, at once" is a claim about
    // the pipeline and not just about a loop.
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 5);
    let mine: Vec<ObjectId> = (0..3)
        .map(|_| place_bare(&mut game, vanilla_creature(2, 2, &[]), 0))
        .collect();
    let theirs = place_bare(&mut game, vanilla_creature(2, 2, &[]), 1);
    for id in mine.iter().chain(std::iter::once(&theirs)) {
        game.battlefield.get_mut(id).unwrap().tapped = true;
    }

    // Player 1's turn ends, player 0's begins — untap step runs for player 0.
    game.active_player = 1;
    pass_turn(&mut game);

    for id in &mine {
        assert!(!game.battlefield[id].tapped, "the active player's permanents untapped");
    }
    assert!(game.battlefield[&theirs].tapped, "CR 502.1 untaps only the active player's");

    let batches: std::collections::HashSet<_> = game
        .events
        .records()
        .iter()
        .filter(|r| matches!(r.event, GameEvent::Untapped { .. }))
        .map(|r| r.stamp.batch)
        .collect();
    assert_eq!(batches.len(), 1, "one event, not three: CR 502.1 says simultaneously");
}

// COVERS-PARTIAL: ATOM-122.1d-001
#[test]
fn test_a_stun_counter_survives_a_real_untap_step() {
    // The same rule as the direct-proposal test, reached through the turn-based
    // action instead. Worth its own test because the untap step is where
    // CR 122.1d is actually met, and because routing that sweep through the
    // pipeline is what made its *order* observable — the sweep builds its batch
    // from `battlefield_ids_ordered` for exactly this reason.
    //
    // This does **not** cover ATOM-502.3-002. That atom's mechanism is a
    // "doesn't untap during your untap step" *continuous effect* (Frost Titan,
    // Winter Orb), which is a different thing from a CR 614 replacement: the
    // permanent simply is not chosen to untap, and no event is replaced. It
    // stays open.
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 5);
    let stunned = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let free = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.battlefield.get_mut(&stunned).unwrap().tapped = true;
    game.battlefield.get_mut(&free).unwrap().tapped = true;
    add_counters(&mut game, stunned, CounterType::Stun, 1);

    game.active_player = 1;
    pass_turn(&mut game);

    assert!(game.battlefield[&stunned].tapped, "the stun counter ate the untap");
    assert_eq!(game.battlefield[&stunned].counter_count(CounterType::Stun), 0);
    assert!(!game.battlefield[&free].tapped, "its neighbour untapped normally");
}

// ---------------------------------------------------------------------------
// Item 6 — consumer 2: regeneration (CR 701.19)
// ---------------------------------------------------------------------------

/// Resolve `Primitive::Regenerate` against one permanent, the way an activated
/// ability's resolution would.
fn regenerate(game: &mut GameState, source: ObjectId, target: ObjectId) {
    let dp = ScriptedDecisionProvider::new();
    let rctx = ResolutionContext {
        source,
        controller: 0,
        targets: vec![ResolvedTarget::Object(target)],
    };
    game.resolve_effect(
        &Effect::Atom(Primitive::Regenerate, any_permanent()),
        &rctx,
        &dp,
    )
    .unwrap();
}

fn any_permanent() -> EffectRecipient {
    EffectRecipient::Target(
        SelectionFilter::Permanent(PermanentFilter::All),
        TargetCount::Exactly(1),
    )
}

/// A creature whose printed static ability regenerates it every time
/// (CR 701.19b) — a Drudge Skeletons that needs no mana.
fn static_regenerator(name: &str) -> Arc<CardData> {
    replacement_creature(
        name,
        ReplacementDef::new(
            EventPattern::Destroy { source: None },
            AffectedSet::SourceOnly,
            Rewrite::Prevent,
        )
        .regeneration(),
    )
}

// COVERS: ATOM-701.19a-001
// COVERS: ATOM-614.8-001
// COVERS: ATOM-701.8c-001
#[test]
fn test_a_regeneration_shield_replaces_a_destruction() {
    // CR 701.19a — "the next time [permanent] would be destroyed this turn,
    // instead remove all damage marked on it and its controller taps it. If
    // it's an attacking or blocking creature, remove it from combat."
    //
    // All four clauses, because the rider is the whole rule and a test that
    // only checked survival would pass against a bare `Prevent`.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let killer = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);
    let blocker = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);
    set_attacking(&mut game, bear, 1);
    set_blocked_by(&mut game, bear, vec![blocker]);
    set_blocking(&mut game, blocker, vec![bear]);
    game.battlefield.get_mut(&bear).unwrap().damage_marked = 2;

    regenerate(&mut game, killer, bear);
    game.execute_action(
        GameAction::Destroy { object: bear, source: DestructionSource::Effect(killer) },
        &test_ctx(),
    )
    .unwrap();

    assert!(game.battlefield.contains_key(&bear), "instead of being destroyed");
    assert_eq!(game.battlefield[&bear].damage_marked, 0, "remove all damage marked on it");
    assert!(game.battlefield[&bear].tapped, "its controller taps it");
    assert!(game.battlefield[&bear].attacking.is_none(), "remove it from combat");
    assert!(
        game.battlefield[&blocker].blocking.as_ref().map(|b| b.blocking.is_empty()).unwrap_or(true),
        "CR 506.4 — and its blocker stops blocking it"
    );
}

#[test]
fn test_regeneration_that_did_not_heal_would_die_on_the_next_check() {
    // Why the rider's damage clause is load-bearing rather than flavour: the
    // shield is `Uses::Once`, so a creature that survived with lethal damage
    // still marked meets CR 704.5g on the very next state-based check with
    // nothing left to spend.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let killer = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);
    game.battlefield.get_mut(&bear).unwrap().damage_marked = 2;

    regenerate(&mut game, killer, bear);
    let dp = ScriptedDecisionProvider::new();
    game.check_state_based_actions(&dp).unwrap();

    assert!(game.battlefield.contains_key(&bear));
    assert_eq!(game.battlefield[&bear].damage_marked, 0);

    // Second check: nothing to destroy, because the damage is gone.
    assert!(!game.check_state_based_actions(&dp).unwrap());
    assert!(game.battlefield.contains_key(&bear));
}

// COVERS: ATOM-701.19a-002
#[test]
fn test_a_shield_protects_once_and_the_second_destruction_gets_through() {
    // "The **next** time" — `Uses::Once`, consumed by removing the registry
    // row, so the second destruction in the same turn finds nothing.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let killer = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);

    regenerate(&mut game, killer, bear);
    assert_eq!(game.replacement_effects.len(), 1);

    game.execute_action(
        GameAction::Destroy { object: bear, source: DestructionSource::Effect(killer) },
        &test_ctx(),
    )
    .unwrap();
    assert!(game.battlefield.contains_key(&bear));
    assert_eq!(game.replacement_effects.len(), 0, "the use was consumed");

    game.execute_action(
        GameAction::Destroy { object: bear, source: DestructionSource::Effect(killer) },
        &test_ctx(),
    )
    .unwrap();
    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Graveyard);
}

// COVERS: ATOM-701.19b-001
#[test]
fn test_static_regeneration_applies_every_time() {
    // CR 701.19b — "if the effect of a *static ability* regenerates a
    // permanent, it replaces destruction with an alternate effect **each
    // time**". `Uses::Static`, so nothing is consumed and the second
    // destruction is replaced too.
    let mut game = setup_two_player_game();
    let skeleton = put_on_battlefield(&mut game, static_regenerator("Drudge Probe"), 0);
    let killer = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);

    for _ in 0..3 {
        game.execute_action(
            GameAction::Destroy {
                object: skeleton,
                source: DestructionSource::Effect(killer),
            },
            &test_ctx(),
        )
        .unwrap();
        assert!(game.battlefield.contains_key(&skeleton));
    }
    assert!(zone_changes(&game).is_empty());
}

// COVERS: ATOM-701.19c-001
#[test]
fn test_cant_be_regenerated_withholds_the_shield_without_destroying_it() {
    // CR 701.19c — "effects that say that a permanent can't be regenerated
    // don't preclude such abilities from being activated or such spells from
    // being cast; rather, they cause regeneration shields to **not be
    // applied**."
    //
    // Both halves asserted, because only the second one distinguishes this
    // reading from the naive one: the shield is still in the registry
    // afterwards, unspent.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let killer = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);

    regenerate(&mut game, killer, bear);
    let dp = ScriptedDecisionProvider::new();
    let rctx = ResolutionContext {
        source: killer,
        controller: 1,
        targets: vec![ResolvedTarget::Object(bear)],
    };
    game.resolve_effect(
        &Effect::Atom(Primitive::CantBeRegenerated, any_permanent()),
        &rctx,
        &dp,
    )
    .unwrap();

    game.execute_action(
        GameAction::Destroy { object: bear, source: DestructionSource::Effect(killer) },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Graveyard);
    assert_eq!(
        game.replacement_effects.len(),
        1,
        "the shield was withheld, not spent — CR 701.19c blocks application, not creation"
    );
}

#[test]
fn test_cant_be_regenerated_does_not_withhold_other_replacements() {
    // The flag is narrow by construction: `is_regeneration` has exactly one
    // reader, and a finality counter on the same creature is untouched by it.
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let killer = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);
    add_counters(&mut game, bear, CounterType::Finality, 1);

    let dp = ScriptedDecisionProvider::new();
    let rctx = ResolutionContext {
        source: killer,
        controller: 1,
        targets: vec![ResolvedTarget::Object(bear)],
    };
    game.resolve_effect(
        &Effect::Atom(Primitive::CantBeRegenerated, any_permanent()),
        &rctx,
        &dp,
    )
    .unwrap();

    game.execute_action(
        GameAction::Destroy { object: bear, source: DestructionSource::Effect(killer) },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Exile);
}

#[test]
fn test_an_unspent_shield_expires_at_end_of_turn() {
    // "This turn" — `Duration::UntilEndOfTurn`, expiring through the same
    // CR 514.2 cleanup hook the continuous-effect registry uses.
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 5);
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let killer = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);

    regenerate(&mut game, killer, bear);
    assert_eq!(game.replacement_effects.len(), 1);

    pass_turn(&mut game);
    assert_eq!(game.replacement_effects.len(), 0);
}

#[test]
fn test_a_shield_dies_with_the_permanent_that_made_it() {
    // `cleanup_zone_state` retires replacement rows by source, the way it
    // retires continuous effects (CR 611.2a).
    let mut game = setup_two_player_game();
    let bear = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);
    let maker = place_bare(&mut game, vanilla_creature(1, 1, &[]), 0);

    regenerate(&mut game, maker, bear);
    assert_eq!(game.replacement_effects.len(), 1);

    game.change_zone(maker, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();
    assert_eq!(game.replacement_effects.len(), 0);
}

// ---------------------------------------------------------------------------
// Item 7 — consumer 3: Kalitas, Traitor of Ghet
// ---------------------------------------------------------------------------

/// Every token on the battlefield, by name.
fn tokens(game: &GameState) -> Vec<String> {
    let mut names: Vec<String> = game
        .battlefield
        .keys()
        .filter_map(|id| {
            let obj = game.objects.get(id)?;
            obj.is_token.then(|| obj.card_data.name.clone())
        })
        .collect();
    names.sort();
    names
}

#[test]
fn test_kalitas_exiles_an_opponents_dying_creature_and_makes_a_zombie() {
    // "If a nontoken creature an opponent controls would die, instead exile
    // that card and create a 2/2 black Zombie creature token."
    //
    // Both halves: the `Instead` retargets the destination, and the CR 615.5
    // rider makes the token afterwards.
    let mut game = setup_two_player_game();
    let _kalitas = put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);
    let victim = place_bare(&mut game, vanilla_creature(2, 2, &[]), 1);

    game.change_zone(victim, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();

    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile);
    assert_eq!(tokens(&game), vec!["Zombie".to_string()]);
    assert_eq!(
        zone_changes(&game),
        vec![(victim, Zone::Battlefield, Zone::Exile, ZoneChangeCause::Exiled)],
    );
}

#[test]
fn test_kalitas_ignores_its_controllers_own_creatures() {
    // "An opponent controls". `PlayerRef::Opponent` is a predicate — controlled
    // by someone who isn't you — which is CR 102.2 at two players and CR 102.3's
    // set in multiplayer, without the type having to lie.
    let mut game = setup_two_player_game();
    let _kalitas = put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);
    let mine = place_bare(&mut game, vanilla_creature(2, 2, &[]), 0);

    game.change_zone(mine, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();

    assert_eq!(game.get_object(mine).unwrap().zone, Zone::Graveyard);
    assert!(tokens(&game).is_empty());
}

#[test]
fn test_kalitas_ignores_tokens() {
    // "A **nontoken** creature", and the leaf this card forced into
    // `PermanentFilter`. CR 707.2 excludes tokenness from copiable values, so
    // it is a property of the `GameObject` that no layer walk can reach — which
    // is why no combination of the existing leaves could have expressed it.
    //
    // Also the one clause with a rules *reason* rather than a flavour one: a
    // token that Kalitas exiled would cease to exist (CR 704.5d) and the
    // exchange would make an unbounded loop with any token-death engine.
    let mut game = setup_two_player_game();
    let kalitas = put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);
    let opponent_token = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);
    game.objects.get_mut(&opponent_token).unwrap().is_token = true;

    game.change_zone(opponent_token, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();

    assert_eq!(game.get_object(opponent_token).unwrap().zone, Zone::Graveyard);
    assert!(tokens(&game).iter().all(|n| n != "Zombie"), "no Zombie was made");
    assert!(game.battlefield.contains_key(&kalitas));
}

#[test]
fn test_kalitas_does_not_replace_its_own_death() {
    // `AffectedSet::Filter` is CR 614.12's "a general subset of permanents", and
    // the subset is the *opponent's* creatures. `SourceOnly` would have made
    // Kalitas immortal, which a test asserting only "the victim was exiled"
    // would never have noticed.
    let mut game = setup_two_player_game();
    let kalitas = put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);

    game.change_zone(kalitas, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();

    assert_eq!(game.get_object(kalitas).unwrap().zone, Zone::Graveyard);
    assert!(tokens(&game).is_empty());
}

#[test]
fn test_kalitas_catches_a_creature_destroyed_by_lethal_damage() {
    // "Dies" is CR 700.4 — a permanent put into a graveyard from the
    // battlefield — not CR 701.8's "destroyed". Kalitas therefore has to see a
    // CR 704.5g death, which reaches it only because `GameAction::Destroy`'s
    // performer proposes the inner `ZoneChange` through the pipeline.
    //
    // A `cause` on Kalitas's pattern would narrow it to one route and every
    // other kind of death would slip past; this test and the sacrifice test
    // above are the pair that catches that.
    let mut game = setup_two_player_game();
    let _kalitas = put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);
    let victim = place_bare(&mut game, vanilla_creature(2, 2, &[]), 1);
    game.battlefield.get_mut(&victim).unwrap().damage_marked = 2;

    let dp = ScriptedDecisionProvider::new();
    game.check_state_based_actions(&dp).unwrap();

    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile);
    assert_eq!(tokens(&game), vec!["Zombie".to_string()]);
}

// COVERS: ATOM-704.7-001
#[test]
fn test_kalitas_simultaneous_deaths_each_exile_and_make_a_zombie() {
    // **Kalitas's own printed ruling, and §4.2's acid test.** When several
    // opponent creatures die at the same time, every one of those cards is
    // exiled and each makes a Zombie: one static replacement, applied once *per
    // death*, because CR 614.5 is per event and batch members are separate
    // events.
    //
    // Under a batch-wide applied set the first creature is exiled and the rest
    // reach the graveyard — and no other Phase RB consumer can tell the two
    // apart, because a counter-derived effect is keyed per permanent either way.
    let mut game = setup_two_player_game();
    let _kalitas = put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);
    let victims: Vec<ObjectId> = (0..4)
        .map(|_| {
            let id = place_bare(&mut game, vanilla_creature(2, 2, &[]), 1);
            game.battlefield.get_mut(&id).unwrap().damage_marked = 2;
            id
        })
        .collect();

    // The SBA sweep gathers all four against one board and performs them as one
    // batch (CR 704.3), which is the only way the question comes up at all.
    let dp = ScriptedDecisionProvider::new();
    game.check_state_based_actions(&dp).unwrap();

    for victim in &victims {
        assert_eq!(
            game.get_object(*victim).unwrap().zone,
            Zone::Exile,
            "every one of those cards is exiled"
        );
    }
    assert_eq!(tokens(&game).len(), 4, "and each makes a Zombie");
    assert!(
        game.players[1].graveyard.is_empty(),
        "nothing reached the graveyard"
    );
}

#[test]
fn test_the_kalitas_rider_runs_after_the_exile_it_rides_on() {
    // §4.1a's timing contract on the card that motivated it: CR 615.5 puts the
    // rest of the effect "immediately afterward", so the Zombie enters the
    // battlefield *after* the creature it replaced has left it. Asserted on the
    // event log, because the end state is identical either way — and the order
    // is what a leaves-the-battlefield trigger will read.
    let mut game = setup_two_player_game();
    let _kalitas = put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);
    let victim = place_bare(&mut game, vanilla_creature(2, 2, &[]), 1);
    let before = game.events.len();

    game.change_zone(victim, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();

    let kinds: Vec<&'static str> = game
        .events
        .events()
        .skip(before)
        .map(|e| match e {
            GameEvent::ZoneChange { .. } => "zone-change",
            GameEvent::PermanentEnteredBattlefield { .. } => "token-entered",
            _ => "other",
        })
        .collect();
    assert_eq!(kinds, vec!["zone-change", "token-entered"]);
}

#[test]
fn test_kalitas_and_a_finality_counter_are_one_choice_not_two_exiles() {
    // Two replacement effects want one death and both send it to exile, so the
    // *outcome* cannot tell them apart — which is exactly why this test asserts
    // on the Zombie. CR 616.1 makes the affected creature's controller (the
    // opponent!) choose one, and only Kalitas's makes a token.
    //
    // The chooser being the opponent is the sharp part: it is the controller of
    // the *affected object*, not of either effect, so the player Kalitas is
    // punishing is the one who decides whether it gets its Zombie.
    let mut game = setup_two_player_game();
    let _kalitas = put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);
    let victim = place_bare(&mut game, vanilla_creature(2, 2, &[]), 1);
    add_counters(&mut game, victim, CounterType::Finality, 1);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::ChooseReplacementEffect { affected_object: None },
        vec![1],
    );
    let asked = std::cell::RefCell::new(Vec::new());
    let recording = RecordingProvider { inner: &dp, asked: &asked };
    let ctx = ActionContext::new(&recording);
    game.change_zone(victim, Zone::Graveyard, ZoneChangeCause::Sacrificed, &ctx)
        .unwrap();

    assert_eq!(*asked.borrow(), vec![1], "the affected creature's controller chooses");
    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile);
    assert!(
        tokens(&game).is_empty(),
        "index 1 is the finality counter — the battlefield sweep offers Kalitas \
         first because Kalitas entered first"
    );
}
