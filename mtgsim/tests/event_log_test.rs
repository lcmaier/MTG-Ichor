//! The event log's lines, as `--dump-events` and the dev GUI's log read
//! them: formatted after the event, when an object a line names may be gone.
//!
//! An ability on the stack is an object that ceases to exist as it resolves,
//! doesn't resolve, or is countered (CR 608.2b, 608.2n, 701.6b), so a line
//! naming it by that object reads as a bare id. Each test reads the log once
//! the objects are gone, and asserts on its words, which every reader sees.

use std::sync::Arc;

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_cv_cards::clone;
use mtgsim::cards::phase_lh_cards::holy_strength;
use mtgsim::cards::phase_tr2a_cards::paladin_of_atonement;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::cards::phase_rd_cards::{gisela_blade_of_goldnight, mending_hands, pinpoint_avalanche, pyroclasm, safe_passage};
use mtgsim::cards::registry::CardRegistry;
use mtgsim::cards::phase_a4i_cards::seeds_of_strength;
use mtgsim::cards::phase_cv_cards::cryptoplasm;
use mtgsim::cards::phase_tr1_cards::soul_warden;
use mtgsim::cards::phase_tr3a_cards::blessed_wine;
use mtgsim::cards::phase_tr3b_cards::banishing_light;
use mtgsim::cards::utility_creatures::merfolk_thaumaturgist;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::GameEvent;
use mtgsim::scenario::Scenario;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{
    fill_library, lightning_bolt, put_in_graveyard, put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx, test_dp,
    RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{AmountExpr, Duration, Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount};
use mtgsim::types::ids::{AbilityId, ObjectId, PlayerId, UntilReturnId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::triggers::{DelayedDuration, EventKind};
use mtgsim::types::zones::{Zone, ZoneChangeAct};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::display::format_event_log;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Walk the turn machinery until `whose` player's `step` begins. Libraries
/// are filled so a draw step on the way is not a loss.
fn advance_to(game: &mut GameState, whose: PlayerId, step: StepType) {
    for p in 0..game.num_players() {
        if game.players[p].library.len() < 5 {
            fill_library(game, p, 10);
        }
    }
    for _ in 0..200 {
        game.advance_turn(&test_dp()).expect("advancing");
        if game.active_player == whose && game.phase.step == Some(step) {
            return;
        }
    }
    panic!("player {whose}'s {step:?} never began");
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

/// `card` cast by `player` from exactly `pool`, each instance of "target"
/// answered with the next of `targets`.
fn cast_at(game: &mut GameState, player: PlayerId, card: Arc<CardData>, pool: &[(ManaType, u64)], targets: &[ChoiceOption]) -> ObjectId {
    let aim = ScriptedDecisionProvider::new();
    for target in targets {
        aim.expect_choice(
            ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
            vec![target.clone()],
        );
    }
    cast_from_pool(game, player, card, pool, aim)
}

/// Lightning Bolt cast by `player` at `target`.
fn bolt_at(game: &mut GameState, player: PlayerId, target: ObjectId) -> ObjectId {
    cast_at(game, player, lightning_bolt(), &[(ManaType::Red, 1)], &[ChoiceOption::Object(target)])
}

fn resolve_top(game: &mut GameState) {
    game.resolve_top_of_stack(&test_dp()).expect("resolving");
    game.perform_sba_and_triggers(&test_dp()).expect("the state-based actions and triggers after it");
}

// ---------------------------------------------------------------------------
// An ability that doesn't resolve, and one countered
// ---------------------------------------------------------------------------

/// The dev GUI's board: Cryptoplasm's upkeep trigger targets the Bears, and
/// Lightning Bolt kills them in response. The trigger doesn't resolve (CR
/// 608.2b), and its line, read once the ability has ceased to exist, says
/// what it was, whose, and why.
#[test]
fn an_ability_that_does_not_resolve_is_named_by_its_source() {
    let mut game = setup_two_player_game();
    let crypto = put_on_battlefield(&mut game, cryptoplasm(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    advance_to(&mut game, 0, StepType::Upkeep);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    let trigger = *game.stack.last().expect("Cryptoplasm's trigger, its one target the Bears");

    bolt_at(&mut game, 1, bears);
    resolve_top(&mut game);
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Graveyard);
    resolve_top(&mut game);
    assert!(game.stack.is_empty() && !game.objects.contains_key(&trigger), "the ability ceased to exist");

    let log = format_event_log(&game);
    let line = format!("AbilityFizzled: Cryptoplasm ({crypto})'s ability [P0] doesn't resolve: every target is illegal (CR 608.2b)");
    assert!(log.contains(&line), "no {line:?} in {log:#?}");
    assert!(!log.iter().any(|l| l.starts_with("SpellFizzled")), "an ability is not a spell: {log:#?}");
}

/// A spell's line says why too, and whose it was, which the card in its
/// graveyard no longer says (CR 108.4a).
#[test]
fn a_spell_that_does_not_resolve_says_why() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let bolt = bolt_at(&mut game, 0, bears);
    game.change_zone(bears, Zone::Hand, ZoneChangeAct::Put, &test_ctx()).unwrap();
    resolve_top(&mut game);

    let log = format_event_log(&game);
    let line = format!("SpellFizzled: Lightning Bolt ({bolt}) [P0] doesn't resolve: every target is illegal (CR 608.2b)");
    assert!(log.contains(&line), "no {line:?} in {log:#?}");
}

/// Lightning Bolt countered (CR 701.6a): its line says whose it was, as a
/// countered ability's does.
#[test]
fn a_countered_spell_says_whose_it_was() {
    let mut game = setup_two_player_game();
    let bolt = cast_at(&mut game, 0, lightning_bolt(), &[(ManaType::Red, 1)], &[ChoiceOption::Player(1)]);
    let counter = put_in_hand(&mut game, lightning_bolt(), 1);
    let ctx = ResolutionContext {
        source: counter,
        ability_source: None,
        controller: 1,
        generated_by: mtgsim::events::cause::GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(bolt)]),
        replaced_event: None,
        trigger: None,
    };
    let effect = Effect::Atom(Primitive::CounterSpell, EffectRecipient::Target(SelectionFilter::Spell, TargetCount::Exactly(1)));
    game.resolve_effect(&effect, &ctx, &test_dp()).unwrap();
    assert_eq!(game.get_object(bolt).unwrap().zone, Zone::Graveyard);

    let log = format_event_log(&game);
    let line = format!("SpellCountered: Lightning Bolt ({bolt}) [P0] countered by Lightning Bolt ({counter})");
    assert!(log.contains(&line), "no {line:?} in {log:#?}");
}

/// Soul Warden's trigger countered by an effect that counters abilities (CR
/// 701.6b): the countered ability is named by its source, as the one that
/// doesn't resolve is.
#[test]
fn a_countered_ability_is_named_by_its_source() {
    let mut game = setup_two_player_game();
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    let ability = *game.stack.last().expect("Soul Warden's trigger");
    let stifle = put_in_hand(&mut game, lightning_bolt(), 1);
    let ctx = ResolutionContext {
        source: stifle,
        ability_source: None,
        controller: 1,
        generated_by: mtgsim::events::cause::GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(ability)]),
        replaced_event: None,
        trigger: None,
    };
    let counter = Effect::Atom(Primitive::CounterAbility, EffectRecipient::Target(SelectionFilter::Spell, TargetCount::Exactly(1)));
    game.resolve_effect(&counter, &ctx, &test_dp()).unwrap();
    assert!(game.stack.is_empty());

    let log = format_event_log(&game);
    let line = format!("AbilityCountered: Soul Warden ({warden})'s ability [P0] countered by Lightning Bolt ({stifle})");
    assert!(log.contains(&line), "no {line:?} in {log:#?}");
}

// ---------------------------------------------------------------------------
// What a spell or ability targets
// ---------------------------------------------------------------------------

/// `line`, and the line after it, `next`, in `log`.
fn assert_followed_by(log: &[String], line: &str, next: &str) {
    let at = log.iter().position(|l| l == line).unwrap_or_else(|| panic!("no {line:?} in {log:#?}"));
    assert_eq!(log.get(at + 1).map(String::as_str), Some(next), "{log:#?}");
}

/// Lightning Bolt at the Bears, then at a player: each target is announced
/// as the spell becomes cast (CR 601.2i), on the line before the cast's.
#[test]
fn a_spells_target_is_announced_as_it_becomes_cast() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let at_bears = bolt_at(&mut game, 0, bears);
    let at_player = cast_at(&mut game, 0, lightning_bolt(), &[(ManaType::Red, 1)], &[ChoiceOption::Player(1)]);
    resolve_top(&mut game);
    resolve_top(&mut game);

    let log = format_event_log(&game);
    let cast = |bolt: ObjectId| format!("SpellCast: P0 casts Lightning Bolt ({bolt})");
    assert_followed_by(&log, &format!("Targeted: Grizzly Bears ({bears}) by Lightning Bolt ({at_bears}) [P0]"), &cast(at_bears));
    assert_followed_by(&log, &format!("Targeted: P1 by Lightning Bolt ({at_player}) [P0]"), &cast(at_player));
}

/// Cryptoplasm's upkeep trigger: its target is announced as it goes on the
/// stack (CR 603.3d), and the line names it by its source once it has ceased
/// to exist.
#[test]
fn a_triggered_abilitys_target_is_announced_as_it_goes_on_the_stack() {
    let mut game = setup_two_player_game();
    let crypto = put_on_battlefield(&mut game, cryptoplasm(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    advance_to(&mut game, 0, StepType::Upkeep);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    let line = format!("Targeted: Grizzly Bears ({bears}) by Cryptoplasm ({crypto})'s ability [P0]");
    assert_eq!(format_event_log(&game).last(), Some(&line), "the last thing that happened");

    bolt_at(&mut game, 1, bears);
    resolve_top(&mut game);
    resolve_top(&mut game);
    assert!(game.stack.is_empty());
    assert!(format_event_log(&game).contains(&line));
}

/// Merfolk Thaumaturgist's "{T}: Switch target creature's power and
/// toughness": its target is announced once the tap is paid, as the ability
/// becomes activated (CR 602.2b runs 601.2i).
#[test]
fn an_activated_abilitys_target_is_announced_once_its_cost_is_paid() {
    let mut game = setup_two_player_game();
    let thaumaturgist = put_on_battlefield(&mut game, merfolk_thaumaturgist(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    activate_at(&mut game, thaumaturgist, bears).expect("a tap it can pay");

    let log = format_event_log(&game);
    let targeted = format!("Targeted: Grizzly Bears ({bears}) by Merfolk Thaumaturgist ({thaumaturgist})'s ability [P0]");
    assert_followed_by(&log, &targeted, &format!("AbilityActivated: Merfolk Thaumaturgist ({thaumaturgist}) [P0]"));
    let tapped = log.iter().position(|l| *l == format!("Tapped: Merfolk Thaumaturgist ({thaumaturgist})"));
    assert!(tapped.is_some() && tapped < log.iter().position(|l| *l == targeted), "the cost first: {log:#?}");
}

/// The same ability for {R}, with no mana to pay it: the activation is
/// reversed (CR 732.1), so nothing says it was activated or what it targeted.
#[test]
fn an_activation_reversed_for_its_cost_announces_nothing() {
    let mut game = setup_two_player_game();
    let thaumaturgist = put_on_battlefield(&mut game, thaumaturgist_for_red(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    assert!(activate_at(&mut game, thaumaturgist, bears).is_err(), "nothing pays the red");
    assert!(game.stack.is_empty());

    let log = format_event_log(&game);
    assert!(!log.iter().any(|l| l.starts_with("AbilityActivated") || l.starts_with("Targeted")), "{log:#?}");
}

/// Seeds of Strength with the Bears, the one creature, chosen for each of
/// its three instances (CR 115.3 lets one object be chosen once for each):
/// one line, which says how many instances chose it (CR 115.9a counts each).
#[test]
fn one_target_chosen_for_several_instances_is_one_line() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let seeds = cast_at(
        &mut game,
        0,
        seeds_of_strength(),
        &[(ManaType::Green, 1), (ManaType::White, 1)],
        &[],
    );

    let log = format_event_log(&game);
    let targeted: Vec<&String> = log.iter().filter(|l| l.starts_with("Targeted")).collect();
    let line = format!("Targeted: Grizzly Bears ({bears}) by Seeds of Strength ({seeds}) [P0], chosen for 3 instances");
    assert_eq!(targeted, [&line]);
}

/// A creature chosen as the spell is cast, not targeted: no line, since a
/// choice that does not use the word "target" is not one (CR 115.10a).
#[test]
fn a_choice_that_does_not_target_announces_nothing() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    let pump = cast_at(&mut game, 0, chosen_pump(), &[(ManaType::Green, 1)], &[]);
    assert!(!game.stack_entries[&pump].chosen_targets[0].is_targeted());

    let log = format_event_log(&game);
    assert!(log.iter().any(|l| l.starts_with("SpellCast")));
    assert!(!log.iter().any(|l| l.starts_with("Targeted")), "{log:#?}");
}

/// Activate `source`'s activated ability at `target`.
fn activate_at(game: &mut GameState, source: ObjectId, target: ObjectId) -> Result<(), String> {
    let abilities = mtgsim::oracle::characteristics::get_effective_abilities(game, source);
    let index = abilities.iter().position(|a| a.ability_type == AbilityType::Activated).expect("an activated ability");
    let aim = ScriptedDecisionProvider::new();
    aim.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
        vec![ChoiceOption::Object(target)],
    );
    game.activate_ability(0, source, index, &ManaWindowStop::new(aim))
}

/// **Fixture.** Merfolk Thaumaturgist's ability for {R} instead of {T}, so
/// its cost is one an empty pool cannot pay.
fn thaumaturgist_for_red() -> Arc<CardData> {
    CardDataBuilder::new("Thaumaturgist for Red")
        .card_type(CardType::Creature)
        .power_toughness(1, 2)
        .ability(AbilityDef {
            rules_text: "{R}: Switch target creature's power and toughness until end of turn.".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::Mana(ManaCost::build(&[ManaType::Red], 0))],
            effect: Effect::Atom(
                Primitive::SwitchPowerToughness(Duration::UntilEndOfTurn),
                EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            ),
        })
        .build()
}

/// **Fixture.** "Choose a creature. It gets +2/+2 until end of turn", the
/// choice made as it is cast (`phase_cv1b_integration_test.rs`'s).
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
// A delayed trigger, from its creation to its resolution
// ---------------------------------------------------------------------------

/// The owner's Blessed Wine log: between the spell resolving and its draw a
/// turn later, the log said nothing, and the draw's lines named the trigger
/// `#0`. Its creation now has a line saying what it will do (CR 603.7a), and
/// the lines a turn later name the same trigger by the number that line gave
/// it.
#[test]
fn a_delayed_trigger_is_announced_as_it_is_created_and_named_when_it_triggers() {
    let mut game = setup_two_player_game();
    let wine = cast_from_pool(&mut game, 0, blessed_wine(), &[(ManaType::White, 1), (ManaType::Colorless, 1)], test_dp());
    resolve_top(&mut game);
    let created = format!(
        "DelayedTriggerCreated: Blessed Wine ({wine})'s delayed trigger 1 [P0], once: \"Draw a card at the beginning of the next turn's upkeep.\""
    );
    let log = format_event_log(&game);
    assert_eq!(log.iter().filter(|l| l.starts_with("DelayedTriggerCreated")).count(), 1, "{log:#?}");
    assert!(log.contains(&created), "no {created:?} in {log:#?}");

    advance_to(&mut game, 1, StepType::Upkeep);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    resolve_top(&mut game);

    let log = format_event_log(&game);
    let triggered = format!("AbilityTriggered: Blessed Wine ({wine})'s delayed trigger 1 [P0]");
    let resolved = format!("AbilityResolved: Blessed Wine ({wine})'s delayed trigger 1 [P0]");
    for line in [&triggered, &resolved] {
        assert!(log.contains(line), "no {line:?} in {log:#?}");
    }
    let order: Vec<usize> =
        [&created, &triggered, &resolved].iter().map(|line| log.iter().position(|l| l == *line).unwrap()).collect();
    assert!(order.windows(2).all(|w| w[0] < w[1]), "created, then triggered, then resolved: {order:?}");
}

/// The record comes before anything the trigger could read, so a trigger
/// created during an event never sees its own creation (CR 603.7a), and it
/// is no record a trigger reads at all.
#[test]
fn a_delayed_triggers_creation_is_recorded_once_and_triggers_nothing() {
    let mut game = setup_two_player_game();
    cast_from_pool(&mut game, 0, blessed_wine(), &[(ManaType::White, 1), (ManaType::Colorless, 1)], test_dp());
    resolve_top(&mut game);

    let creations: Vec<_> = game
        .recorded_events()
        .records()
        .iter()
        .filter(|r| matches!(r.event, GameEvent::DelayedTriggerCreated { .. }))
        .map(|r| r.seq)
        .collect();
    assert_eq!(creations.len(), 1);
    assert!(game.delayed_triggers[0].created_at.first_record > creations[0], "created after its own record");
    assert_eq!(EventKind::from_record(&GameEvent::DelayedTriggerCreated {
        id: game.delayed_triggers[0].id,
        source: game.delayed_triggers[0].source,
        controller: 0,
        rules_text: game.delayed_triggers[0].rules_text,
        duration: DelayedDuration::Once,
    }), None);
}

// ---------------------------------------------------------------------------
// An "until" return, from its making to the return
// ---------------------------------------------------------------------------

/// Item 234: Banishing Light's exile makes a return that waits for the Light
/// to leave (CR 610.3), and the log says so as it is made, by the number
/// the Waiting panel lists it under. The record comes before anything the
/// return's event could be, and nothing triggers on it.
#[test]
fn an_until_return_is_announced_as_it_is_made() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let light = put_on_battlefield(&mut game, banishing_light(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    resolve_top(&mut game);
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Exile);

    let made = format!("UntilReturnMade: Banishing Light ({light})'s return 1 [P0]: Grizzly Bears ({bears})");
    let log = format_event_log(&game);
    assert_eq!(log.iter().filter(|l| l.starts_with("UntilReturnMade")).count(), 1, "{log:#?}");
    assert!(log.contains(&made), "no {made:?} in {log:#?}");
    let records = game.recorded_events();
    let record = records.records().iter().find(|r| matches!(r.event, GameEvent::UntilReturnMade { .. })).unwrap();
    assert_eq!(game.until_returns[0].id, UntilReturnId(1));
    assert!(game.until_returns[0].created_at.first_record > record.seq, "made after its own record");
    assert_eq!(EventKind::from_record(&record.event), None);
}

// ---------------------------------------------------------------------------
// What a prevention effect prevented
// ---------------------------------------------------------------------------

/// Protean Hydra on the battlefield with `counters` +1/+1 counters, put
/// there by the scenario loader so its entry needs no X.
fn hydra_with(counters: u32) -> (GameState, ObjectId) {
    let text = format!(
        "library 0: Forest | x8\nlibrary 1: Mountain | x8\nbattlefield: Protean Hydra | controller 0, +1/+1 {counters}\n"
    );
    let scenario = Scenario::parse(&text).unwrap_or_else(|refusal| panic!("{refusal}"));
    let mut game = scenario.build(&CardRegistry::default_registry()).unwrap_or_else(|refusal| panic!("{refusal}")).game.state;
    game.record_events();
    let hydra = game.battlefield_ids_ordered()[0];
    (game, hydra)
}

/// Safe Passage, then Lightning Bolt at its caster: the whole 3 is
/// prevented and nothing is dealt (CR 615.6), so the prevention's own record
/// is the only line about the damage (CR 615.13). It names the effect by its
/// spell, and comes before the Bolt goes to the graveyard.
#[test]
fn a_prevention_with_no_rider_is_announced_where_no_damage_is_dealt() {
    let mut game = setup_two_player_game();
    let passage = cast_from_pool(&mut game, 1, safe_passage(), &[(ManaType::White, 3)], test_dp());
    resolve_top(&mut game);
    let bolt = cast_at(&mut game, 0, lightning_bolt(), &[(ManaType::Red, 1)], &[ChoiceOption::Player(1)]);
    resolve_top(&mut game);

    assert_eq!(game.players[1].life_total, 20);
    let log = format_event_log(&game);
    let prevented = format!("DamagePrevented: 3 to P1 by Spell of Safe Passage ({passage}) [P1]");
    let at = log.iter().position(|l| *l == prevented).unwrap_or_else(|| panic!("no {prevented:?} in {log:#?}"));
    assert!(log[at + 1].starts_with(&format!("ZoneChange: Lightning Bolt ({bolt}) [P0] Stack -> Graveyard")), "{log:#?}");
    assert!(!log.iter().any(|l| l.starts_with("DamageDealt")), "nothing was dealt: {log:#?}");
}

/// Mending Hands' 4 under two Bolts: the first is prevented whole, and the
/// second is prevented by the 1 left and dealt the rest (CR 615.7), so its
/// record says 1 and the 2 dealt follows it, in the same batch.
#[test]
fn a_shield_that_prevents_part_of_the_damage_announces_the_part() {
    let mut game = setup_two_player_game();
    let hands = cast_at(&mut game, 1, mending_hands(), &[(ManaType::White, 1)], &[ChoiceOption::Player(1)]);
    resolve_top(&mut game);
    bolt_at_player(&mut game, 1);
    resolve_top(&mut game);
    let second = bolt_at_player(&mut game, 1);
    resolve_top(&mut game);

    assert_eq!(game.players[1].life_total, 18);
    let log = format_event_log(&game);
    let by = format!("by Spell of Mending Hands ({hands}) [P1]");
    assert!(log.contains(&format!("DamagePrevented: 3 to P1 {by}")), "{log:#?}");
    assert_followed_by(&log, &format!("DamagePrevented: 1 to P1 {by}"), &format!("DamageDealt: Lightning Bolt ({second}) -> P1 for 2"));
}

/// Pyroclasm into Safe Passage: one record for each creature whose damage
/// it prevented, the subject being the unit (CR 615.13, 616.1), and the
/// Bears it does not protect are dealt their 2.
#[test]
fn one_effect_preventing_damage_to_two_creatures_announces_each() {
    let mut game = setup_two_player_game();
    let mine = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let theirs = [put_on_battlefield(&mut game, grizzly_bears(), 1), put_on_battlefield(&mut game, grizzly_bears(), 1)];
    let passage = cast_from_pool(&mut game, 1, safe_passage(), &[(ManaType::White, 3)], test_dp());
    resolve_top(&mut game);
    let clasm = cast_from_pool(&mut game, 0, pyroclasm(), &[(ManaType::Red, 2)], test_dp());
    resolve_top(&mut game);

    let log = format_event_log(&game);
    for bears in theirs {
        let line = format!("DamagePrevented: 2 to Grizzly Bears ({bears}) by Spell of Safe Passage ({passage}) [P1]");
        assert_eq!(log.iter().filter(|l| **l == line).count(), 1, "{line:?} in {log:#?}");
    }
    assert!(log.contains(&format!("DamageDealt: Pyroclasm ({clasm}) -> Grizzly Bears ({mine}) for 2")), "{log:#?}");
    assert_eq!(log.iter().filter(|l| l.starts_with("DamageDealt")).count(), 1, "{log:#?}");
}

/// The dev GUI's note: Lightning Bolt on Protean Hydra. The prevention is
/// announced, and its rider's removal of that many counters follows it
/// (CR 615.5). The prevention files under the kind `DamageIsPrevented`
/// reads (CR 615.13).
#[test]
fn protean_hydras_prevention_is_announced_before_its_counters_come_off() {
    let (mut game, hydra) = hydra_with(5);
    bolt_at(&mut game, 1, hydra);
    resolve_top(&mut game);

    let log = format_event_log(&game);
    let removed = format!("CountersRemovedFrom: 3 +1/+1 counter(s) removed from Protean Hydra ({hydra}) by P0 (now 2)");
    let prevented = format!("DamagePrevented: 3 to Protean Hydra ({hydra}) by StaticAbility of Protean Hydra ({hydra}) [P0]");
    assert_followed_by(&log, &prevented, &removed);
    let records = game.recorded_events();
    let record = records.records().iter().find(|r| matches!(r.event, GameEvent::DamagePrevented { .. })).unwrap();
    assert_eq!(EventKind::from_record(&record.event), Some(EventKind::DamagePrevented));
    assert!(record.stamp.batch.is_some(), "announced inside the Bolt's damage batch");
}

/// Pinpoint Avalanche's damage can't be prevented (CR 615.12): the Hydra's
/// effect applies and prevents none of it, so no record says it did, and
/// the 4 is dealt.
#[test]
fn unpreventable_damage_announces_no_prevention() {
    let (mut game, hydra) = hydra_with(5);
    // The Hydra is the one creature, so the target is asked of nobody.
    let avalanche = cast_from_pool(&mut game, 1, pinpoint_avalanche(), &[(ManaType::Red, 5)], test_dp());
    resolve_top(&mut game);

    let log = format_event_log(&game);
    assert!(!log.iter().any(|l| l.starts_with("DamagePrevented")), "{log:#?}");
    assert!(log.contains(&format!("DamageDealt: Pinpoint Avalanche ({avalanche}) -> Protean Hydra ({hydra}) for 4")), "{log:#?}");
}

/// Lightning Bolt cast by player 0 at `player`.
fn bolt_at_player(game: &mut GameState, player: PlayerId) -> ObjectId {
    cast_at(game, 0, lightning_bolt(), &[(ManaType::Red, 1)], &[ChoiceOption::Player(player)])
}

// ---------------------------------------------------------------------------
// A copy, named as the event found it
// ---------------------------------------------------------------------------

/// Clone put onto the battlefield from `player`'s graveyard, entering as a
/// copy of the one creature on the battlefield.
fn clone_of_the_creature(game: &mut GameState, player: PlayerId) -> ObjectId {
    let clone = put_in_graveyard(game, clone(), player);
    let dp = RecordingDecisionProvider::picking(0);
    game.change_zone(clone, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&dp)).unwrap();
    clone
}

/// Clone as Paladin of Atonement, with Holy Strength on it, Bolted. Every
/// line about the creature that died names it as it last existed (CR
/// 113.7a), as its zone change does: the Aura falling off, its dies trigger,
/// the life that trigger gains and its resolution. None names it as the
/// Clone card in the graveyard.
#[test]
fn a_copy_that_died_is_named_as_it_left_on_every_line_about_it() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, paladin_of_atonement(), 1);
    let clone = clone_of_the_creature(&mut game, 0);
    let aura = put_on_battlefield(&mut game, holy_strength(), 0);
    game.execute_action(GameAction::Attach { attachment: aura, host: clone }, &test_ctx()).unwrap();
    bolt_at(&mut game, 1, clone);
    resolve_top(&mut game);
    resolve_top(&mut game);
    assert_eq!(game.players[0].life_total, 23, "its toughness as it last existed, 1 and Holy Strength's 2");

    let log = format_event_log(&game);
    let paladin = format!("Paladin of Atonement (Clone, {clone})");
    let died = log.iter().position(|l| l.starts_with(&format!("ZoneChange: {paladin}"))).unwrap_or_else(|| panic!("{log:#?}"));
    let after: Vec<&String> = log[died..].iter().filter(|l| l.contains(&format!("{clone})"))).collect();
    for line in [
        format!("Unattached: Holy Strength ({aura}) from {paladin} (HostLeft)"),
        format!("LifeChanged: P0 20 -> 23 (source: {paladin})"),
        format!("AbilityResolved: {paladin} [P0]"),
    ] {
        assert!(after.contains(&&line), "no {line:?} in {log:#?}");
    }
    assert!(after.iter().any(|l| l.starts_with(&format!("AbilityTriggered: {paladin} [P0]"))), "{log:#?}");
    assert!(after.iter().all(|l| l.contains(&paladin)), "a line names it as the card: {after:#?}");
}

/// Clone as Gisela, Blade of Goldnight halves a Bolt at its controller's
/// Bears, the Gisela it copied gone. The prevention names its source as it
/// is, a Gisela, not as its card.
#[test]
fn a_prevention_by_a_copy_names_the_copy() {
    let mut game = setup_two_player_game();
    let gisela = put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 1);
    let clone = clone_of_the_creature(&mut game, 0);
    game.change_zone(gisela, Zone::Graveyard, ZoneChangeAct::Put, &test_ctx()).unwrap();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    bolt_at(&mut game, 1, bears);
    resolve_top(&mut game);

    let log = format_event_log(&game);
    let line = format!("DamagePrevented: 2 to Grizzly Bears ({bears}) by StaticAbility of Gisela, Blade of Goldnight (Clone, {clone}) [P0]");
    assert!(log.contains(&line), "no {line:?} in {log:#?}");
}
