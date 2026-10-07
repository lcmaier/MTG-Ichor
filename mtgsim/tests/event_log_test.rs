//! The event log's lines, as `--dump-events` and the dev GUI's log read
//! them: formatted after the event, when an object a line names may be gone.
//!
//! An ability on the stack is an object that ceases to exist as it resolves,
//! doesn't resolve, or is countered (CR 608.2b, 608.2n, 701.6b), so a line
//! naming it by that object reads as a bare id. Each test reads the log once
//! the objects are gone, and asserts on its words, which every reader sees.

use std::sync::Arc;

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_a4i_cards::seeds_of_strength;
use mtgsim::cards::phase_cv_cards::cryptoplasm;
use mtgsim::cards::phase_tr1_cards::soul_warden;
use mtgsim::cards::utility_creatures::merfolk_thaumaturgist;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{
    fill_library, lightning_bolt, put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx, test_dp,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{AmountExpr, Duration, Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount};
use mtgsim::types::ids::{AbilityId, ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
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
        game.advance_turn(&test_ctx()).expect("advancing");
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

/// A spell's line says why too, and names the card in its graveyard.
#[test]
fn a_spell_that_does_not_resolve_says_why() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let bolt = bolt_at(&mut game, 0, bears);
    game.change_zone(bears, Zone::Hand, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    resolve_top(&mut game);

    let log = format_event_log(&game);
    let line = format!("SpellFizzled: Lightning Bolt ({bolt}) doesn't resolve: every target is illegal (CR 608.2b)");
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
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(ability)]),
        replaced_amount: None,
        damage_prevented: None,
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
