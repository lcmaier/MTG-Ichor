//! TR-5e — the attack declaration (`triggers-architecture.md` §12, TR-5e).
//!
//! 1. The declaration's record names what each attacker attacks (CR 508.1b).
//! 2. The readings of CR 508.3a–e and 506.5 over that record: a triggering
//!    event for each attacker, or for each player attacked with its
//!    attackers, and CR 508.2a's moment.
//! 3. "The player or planeswalker it's attacking", read as CR 608.2h reads
//!    the creature: Hellrider, and both halves of Fathom Fleet Swordjack's
//!    ruling.
//! 4. "Defending player" (CR 508.5, 802.2a): for the creature the text names,
//!    as the player acted on, in a target's filter and in an intervening
//!    "if"; and with no creature named, in a target's filter.

use std::sync::Arc;

use mtgsim::cards::authoring::{
    attacks, attacks_alone, attacks_defender, is_attacked, player_attacks, player_attacks_player, player_or_their_planeswalker,
    triggered_ability, whenever, Whose,
};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lg_cards::act_of_treason;
use mtgsim::cards::phase_tr5e_cards::hellrider;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::{ChosenTargets, FilterIdentity};
use mtgsim::events::cause::GeneratedBy;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{fill_library, put_in_graveyard, put_on_battlefield, setup_game, setup_two_player_game, test_dp};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, Antecedent, ColorChange, Condition, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, PlayerFact,
    PlayerRef, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::triggers::{DefenderFilter, EventRole, TriggerDef, TriggerEvent, TriggerSubject, TriggerTier};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::display::format_event_log;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn stocked(players: usize) -> GameState {
    let mut game = if players == 2 { setup_two_player_game() } else { setup_game(players) };
    for p in 0..players {
        fill_library(&mut game, p, 10);
    }
    game
}

/// Player 0 declares `attacks`, each attacker against its player.
/// `attackers` is every creature that can attack, in the order they entered:
/// the declaration's options are each of them against each opponent, the
/// opponents in seat order.
fn declare(game: &mut GameState, attackers: &[ObjectId], attacks: &[(ObjectId, PlayerId)]) {
    let opponents = game.players.len() - 1;
    let picks = attacks
        .iter()
        .map(|&(attacker, player)| {
            let at = attackers.iter().position(|&a| a == attacker).unwrap();
            at * opponents + (player - 1)
        })
        .collect();
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(StepType::DeclareAttackers) });
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::DeclareAttackers, picks);
    game.process_declare_attackers(&dp).unwrap();
}

/// The one declaration's record.
fn declared(game: &GameState) -> Vec<(ObjectId, AttackTarget)> {
    let records = game.recorded_events();
    let mut declarations = records.events().filter_map(|event| match event {
        GameEvent::AttackersDeclared { attackers } => Some(attackers.clone()),
        _ => None,
    });
    let attacks = declarations.next().expect("a declaration");
    assert!(declarations.next().is_none(), "one declaration");
    attacks
}

/// Put every waiting trigger on the stack and resolve the stack, answering
/// every prompt with `dp`.
fn resolve_all(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(dp).unwrap();
        game.perform_sba_and_triggers(dp).unwrap();
    }
}

/// An enchantment with one triggered ability.
fn watcher(name: &str, arm: impl Into<TriggerEvent>, effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new(name).card_type(CardType::Enchantment).ability(triggered_ability("", whenever(arm, effect))).build()
}

/// "Put a +1/+1 counter on that creature", or "on each of them".
fn a_counter_on_the_attackers() -> Effect {
    Effect::Atom(
        Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
        EffectRecipient::Triggering(EventRole::Attacker),
    )
}

/// A provider that answers one ordering prompt, `player`'s two triggers,
/// which differ in the attackers they read.
fn ordered_by(player: PlayerId) -> ScriptedDecisionProvider {
    let dp = ScriptedDecisionProvider::new();
    dp.expect_ordering(ChoiceKind::OrderTriggers { player, tier: TriggerTier::First }, vec![0, 1]);
    dp
}

fn plus_ones(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].counter_count(CounterType::PlusOnePlusOne)
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// `effect` resolving as `player`'s sorcery aimed at `target`.
fn resolve_as(game: &mut GameState, player: PlayerId, effect: &Effect, target: ObjectId) {
    let source = put_in_graveyard(game, act_of_treason(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(target)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(effect, &ctx, &test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
}

fn creature_you_control() -> ObjectFilter {
    ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ByController(PlayerRef::You)))
}

/// `n` Grizzly Bears under player 0.
fn bears(game: &mut GameState, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| put_on_battlefield(game, grizzly_bears(), 0)).collect()
}

/// How many attackers each waiting trigger's triggering events name, in the
/// order they triggered.
fn attackers_per_trigger(game: &GameState) -> Vec<usize> {
    game.pending_triggers.iter().map(|t| t.binding.triggering_objects(EventRole::Attacker).len()).collect()
}

// ---------------------------------------------------------------------------
// 1. The record
// ---------------------------------------------------------------------------

/// Two creatures attacking two of three opponents: the record pairs each with
/// its own, and the log line says so.
#[test]
fn the_declaration_names_what_each_attacker_attacks() {
    let mut game = stocked(4);
    let first = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let second = put_on_battlefield(&mut game, grizzly_bears(), 0);
    declare(&mut game, &[first, second], &[(first, 1), (second, 3)]);

    assert_eq!(declared(&game), vec![(first, AttackTarget::Player(1)), (second, AttackTarget::Player(3))]);
    let log = format_event_log(&game);
    let line = format!("AttackersDeclared: [Grizzly Bears ({first}) -> P1, Grizzly Bears ({second}) -> P3]");
    assert!(log.contains(&line), "no {line:?} in {log:#?}");
}

// ---------------------------------------------------------------------------
// 2. The readings
// ---------------------------------------------------------------------------

/// CR 508.3a: "whenever a creature attacks" triggers once for each creature
/// declared as an attacker, whatever it attacks, and "that creature" is each.
#[test]
fn a_creature_attacks_triggers_for_each_attacker() {
    let mut game = stocked(4);
    put_on_battlefield(&mut game, watcher("Fixture War Horn", attacks(TriggerSubject::Any), a_counter_on_the_attackers()), 2);
    let [first, second] = bears(&mut game, 2)[..] else { unreachable!() };
    declare(&mut game, &[first, second], &[(first, 1), (second, 3)]);

    assert_eq!(attackers_per_trigger(&game), vec![1, 1]);
    resolve_all(&mut game, &ordered_by(2));
    assert_eq!((plus_ones(&game, first), plus_ones(&game, second)), (1, 1));
}

/// CR 508.3a's second sentence: "whenever a creature attacks you", and
/// "attacks you or a planeswalker you control", trigger for the attacker
/// attacking its controller and not the one attacking another player.
#[test]
fn a_creature_attacks_you_triggers_for_your_attacker_only() {
    for you in [DefenderFilter::Player(Some(PlayerRef::You)), player_or_their_planeswalker(PlayerRef::You)] {
        let mut game = stocked(4);
        let attacks_you = attacks_defender(TriggerSubject::Any, you);
        put_on_battlefield(&mut game, watcher("Fixture Sentry Bell", attacks_you, a_counter_on_the_attackers()), 1);
        let [first, second] = bears(&mut game, 2)[..] else { unreachable!() };
        declare(&mut game, &[first, second], &[(first, 1), (second, 3)]);

        assert_eq!(attackers_per_trigger(&game), vec![1]);
        resolve_all(&mut game, &test_dp());
        assert_eq!((plus_ones(&game, first), plus_ones(&game, second)), (1, 0));
    }
}

/// CR 508.3b: "whenever a player is attacked" triggers once for each player
/// attacked, and "them" is the creatures attacking that player: three
/// attackers against two players make two triggers, of two attackers and one.
#[test]
fn a_player_is_attacked_triggers_for_each_player_with_its_attackers() {
    let mut game = stocked(4);
    let attacked = is_attacked(DefenderFilter::Player(None));
    put_on_battlefield(&mut game, watcher("Fixture Watchtower", attacked, a_counter_on_the_attackers()), 2);
    let [first, second, third] = bears(&mut game, 3)[..] else { unreachable!() };
    declare(&mut game, &[first, second, third], &[(first, 1), (second, 3), (third, 1)]);

    assert_eq!(attackers_per_trigger(&game), vec![2, 1], "player 1's two attackers, then player 3's one");
    resolve_all(&mut game, &ordered_by(2));
    assert_eq!([first, second, third].map(|id| plus_ones(&game, id)), [1, 1, 1]);
}

/// CR 508.3c: "whenever you attack with a creature" is a creature its
/// controller attacks with: player 0's watcher triggers for each, and the same
/// watcher under player 1, whose creatures do not attack, not at all.
#[test]
fn you_attack_with_a_creature_triggers_for_each_of_yours() {
    let mut game = stocked(2);
    let yours = || watcher("Fixture Banner", attacks(creature_you_control()), a_counter_on_the_attackers());
    put_on_battlefield(&mut game, yours(), 0);
    put_on_battlefield(&mut game, yours(), 1);
    let [first, second] = bears(&mut game, 2)[..] else { unreachable!() };
    declare(&mut game, &[first, second], &[(first, 1), (second, 1)]);

    assert_eq!(attackers_per_trigger(&game), vec![1, 1]);
    assert!(game.pending_triggers.iter().all(|t| t.controller == 0));
}

/// CR 508.3d: "whenever you attack" triggers once however many creatures
/// attack, and "them" is every one.
#[test]
fn you_attack_triggers_once_for_the_declaration() {
    let mut game = stocked(4);
    put_on_battlefield(&mut game, watcher("Fixture Drum", player_attacks(Whose::Yours), a_counter_on_the_attackers()), 0);
    put_on_battlefield(&mut game, watcher("Fixture Drum", player_attacks(Whose::Yours), a_counter_on_the_attackers()), 1);
    let [first, second] = bears(&mut game, 2)[..] else { unreachable!() };
    declare(&mut game, &[first, second], &[(first, 1), (second, 3)]);

    assert_eq!(attackers_per_trigger(&game), vec![2], "player 0's one trigger; player 1 did not attack");
    resolve_all(&mut game, &test_dp());
    assert_eq!((plus_ones(&game, first), plus_ones(&game, second)), (1, 1));
}

/// CR 508.3e: "whenever you attack an opponent" triggers once for each
/// opponent attacked, with its attackers; "whenever an opponent attacks you"
/// triggers only for the player attacked.
#[test]
fn you_attack_a_player_triggers_for_each_player_attacked() {
    let mut game = stocked(4);
    let effect = a_counter_on_the_attackers;
    put_on_battlefield(&mut game, watcher("Fixture Raid Horn", player_attacks_player(Whose::Yours, Whose::AnOpponents), effect()), 0);
    let attacks_you = || player_attacks_player(Whose::AnOpponents, Whose::Yours);
    put_on_battlefield(&mut game, watcher("Fixture Alarm", attacks_you(), effect()), 2);
    put_on_battlefield(&mut game, watcher("Fixture Alarm", attacks_you(), effect()), 3);
    let [first, second, third] = bears(&mut game, 3)[..] else { unreachable!() };
    declare(&mut game, &[first, second, third], &[(first, 1), (second, 3), (third, 1)]);

    let mut by_controller: Vec<(PlayerId, usize)> = game
        .pending_triggers
        .iter()
        .map(|t| (t.controller, t.binding.triggering_objects(EventRole::Attacker).len()))
        .collect();
    by_controller.sort();
    assert_eq!(by_controller, vec![(0, 1), (0, 2), (3, 1)], "player 0 for players 1 and 3; player 3, attacked; not player 2");
}

/// CR 506.5: "whenever a creature attacks alone" triggers when it is the only
/// creature declared as an attacker, and not when another attacks beside it.
#[test]
fn attacks_alone_triggers_only_for_a_lone_attacker() {
    for (attackers, triggers) in [(1, 1), (2, 0)] {
        let mut game = stocked(2);
        put_on_battlefield(&mut game, watcher("Fixture Duelist's Oath", attacks_alone(TriggerSubject::Any), a_counter_on_the_attackers()), 0);
        let creatures = bears(&mut game, attackers);
        let attacks: Vec<(ObjectId, PlayerId)> = creatures.iter().map(|&id| (id, 1)).collect();
        declare(&mut game, &creatures, &attacks);
        assert_eq!(game.pending_triggers.len(), triggers, "{attackers} attacking");
    }
}

/// A defender's triggering event has several attackers, so a reader of one
/// object, "its power", cannot name them: the registry check refuses it, and
/// accepts "them".
#[test]
fn its_power_over_a_defenders_attackers_is_refused() {
    let gain = |amount| Effect::Atom(Primitive::GainLife(amount), EffectRecipient::Controller);
    let its_power = whenever(is_attacked(DefenderFilter::Player(None)), gain(AmountExpr::TriggeringPower(EventRole::Attacker)));
    assert_eq!(its_power.event_roles_its_arms_lack(), vec![EventRole::Attacker]);
    let them = whenever(is_attacked(DefenderFilter::Player(None)), a_counter_on_the_attackers());
    assert!(them.event_roles_its_arms_lack().is_empty());
    let each = whenever(attacks(TriggerSubject::Any), gain(AmountExpr::TriggeringPower(EventRole::Attacker)));
    assert!(each.event_roles_its_arms_lack().is_empty(), "an attacker's own triggering event has one");
}

// COVERS: ATOM-508.2a-001
/// CR 508.2a: "whenever a green creature attacks" asks at the declaration. A
/// blue creature turned green after it attacks triggers nothing, then or
/// later.
#[test]
fn a_creature_turned_green_after_it_attacks_triggers_nothing() {
    let mut game = stocked(2);
    let green = ObjectFilter::ByColor(Color::Green);
    put_on_battlefield(&mut game, watcher("Fixture Verdant Banner", attacks(green), a_counter_on_the_attackers()), 0);
    let drake = CardDataBuilder::new("Fixture Blue Drake").card_type(CardType::Creature).color(Color::Blue).power_toughness(2, 2).build();
    let drake = put_on_battlefield(&mut game, drake, 0);
    declare(&mut game, &[drake], &[(drake, 1)]);
    assert!(game.pending_triggers.is_empty(), "blue as it was declared");

    let make_green = Effect::Atom(
        Primitive::ChangeColor(ColorChange::Set([Color::Green].into_iter().collect()), Duration::UntilEndOfTurn),
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    );
    let ctx = ResolutionContext {
        source: drake,
        ability_source: None,
        controller: 0,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(drake)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&make_green, &ctx, &test_dp()).unwrap();
    assert!(mtgsim::oracle::characteristics::get_effective_colors(&game, drake).contains(&Color::Green), "green now");
    resolve_all(&mut game, &test_dp());
    assert_eq!(plus_ones(&game, drake), 0);

    // The same board with a green creature: the watcher does trigger.
    let mut game = stocked(2);
    put_on_battlefield(&mut game, watcher("Fixture Verdant Banner", attacks(ObjectFilter::ByColor(Color::Green)), a_counter_on_the_attackers()), 0);
    let bear = put_on_battlefield(&mut game, grizzly_bears(), 0);
    declare(&mut game, &[bear], &[(bear, 1)]);
    resolve_all(&mut game, &test_dp());
    assert_eq!(plus_ones(&game, bear), 1);
}

// ---------------------------------------------------------------------------
// 3. The player or planeswalker it's attacking
// ---------------------------------------------------------------------------

// RULING: Hellrider #1 - "For each attacking creature, Hellrider will deal
//   damage to the corresponding player or planeswalker."
/// Hellrider and two Grizzly Bears attack three players: each trigger deals
/// its 1 damage to the player its own creature attacks.
#[test]
fn hellrider_deals_each_attackers_damage_to_what_it_attacks() {
    let mut game = stocked(4);
    let rider = put_on_battlefield(&mut game, hellrider(), 0);
    let [first, second] = bears(&mut game, 2)[..] else { unreachable!() };
    let before = [1, 2, 3].map(|p| life(&game, p));
    declare(&mut game, &[rider, first, second], &[(rider, 2), (first, 1), (second, 3)]);
    assert_eq!(game.pending_triggers.len(), 3, "one for each creature its controller attacks with");

    let dp = ScriptedDecisionProvider::new();
    dp.expect_ordering(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First }, vec![0, 1, 2]);
    resolve_all(&mut game, &dp);
    assert_eq!([1, 2, 3].map(|p| before[p - 1] - life(&game, p)), [1, 1, 1]);
}

/// Hellrider's trigger waiting for Grizzly Bears, which attack player 1.
fn bears_attack_beside_hellrider() -> (GameState, ObjectId) {
    let mut game = stocked(2);
    let rider = put_on_battlefield(&mut game, hellrider(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    declare(&mut game, &[rider, bears], &[(bears, 1)]);
    assert_eq!(declared(&game), vec![(bears, AttackTarget::Player(1))], "Hellrider stays home");
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(game.stack.len(), 1, "the trigger is on the stack");
    (game, bears)
}

// RULING: Fathom Fleet Swordjack #5 - "If Fathom Fleet Swordjack leaves the
//   battlefield after its triggered ability triggers but before it resolves,
//   the ability will resolve as normal. The player or planeswalker it was
//   attacking will be dealt damage."
/// The attacker destroyed in response: the player it was attacking, as it
/// last existed, is dealt the damage.
#[test]
fn an_attacker_that_left_the_battlefield_still_names_what_it_attacked() {
    let (mut game, bears) = bears_attack_beside_hellrider();
    let destroy = Effect::Atom(Primitive::Destroy, EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)));
    resolve_as(&mut game, 1, &destroy, bears);
    assert!(!game.battlefield.contains_key(&bears));
    let before = life(&game, 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(before - life(&game, 1), 1);
}

// RULING: Fathom Fleet Swordjack #5 - "On the other hand, if it remains on
//   the battlefield but leaves combat before its triggered ability resolves,
//   no player or planeswalker is dealt damage."
/// The attacker stolen in response: a change of control removes it from
/// combat (CR 506.4), so it attacks nothing and nobody is dealt damage.
#[test]
fn an_attacker_that_left_combat_names_nothing() {
    let (mut game, bears) = bears_attack_beside_hellrider();
    resolve_as(&mut game, 1, &act_of_treason().abilities[0].effect, bears);
    assert!(game.battlefield[&bears].attacking.is_none(), "removed from combat");
    let before = [0, 1].map(|p| life(&game, p));
    resolve_all(&mut game, &test_dp());
    assert_eq!([0, 1].map(|p| life(&game, p)), before);
}

// ---------------------------------------------------------------------------
// 4. "Defending player"
// ---------------------------------------------------------------------------

/// A 2/2 creature with haste and one triggered ability.
fn attacking_watcher(name: &str, def: TriggerDef) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .color(Color::Red)
        .power_toughness(2, 2)
        .keyword_flag(KeywordFlag::Haste)
        .ability(triggered_ability("", def))
        .build()
}

fn defending_player_of(creature: Antecedent) -> PlayerRef {
    PlayerRef::DefendingPlayer(creature)
}

fn its_attacker() -> Antecedent {
    Antecedent::Triggering(EventRole::Attacker)
}

fn loses_one(creature: Antecedent) -> Effect {
    Effect::Atom(Primitive::LoseLife(AmountExpr::Fixed(1)), EffectRecipient::DefendingPlayer(creature))
}

/// "Tap target creature [defending player] controls".
fn tap_target_creature_of(defending: PlayerRef) -> Effect {
    let theirs = ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ByController(defending)));
    Effect::Atom(Primitive::Tap, EffectRecipient::Target(SelectionFilter::Permanent(theirs), TargetCount::Exactly(1)))
}

/// Player 0's Fixture Warcaller, a 2/2 with haste, carries "whenever a
/// creature you control attacks, defending player loses 1 life", with
/// "defending player" read for `creature`. The Warcaller attacks player 1 and
/// Grizzly Bears attack player 3, so the trigger fires twice, once for each.
/// Returns how much life players 1, 2 and 3 lost once both resolve.
fn warcaller_and_bears_attack(creature: Antecedent, dp: &dyn DecisionProvider) -> [i64; 3] {
    let mut game = stocked(4);
    let def = whenever(attacks(creature_you_control()), loses_one(creature));
    let warcaller = put_on_battlefield(&mut game, attacking_watcher("Fixture Warcaller", def), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let before = [1, 2, 3].map(|p| life(&game, p));
    declare(&mut game, &[warcaller, bears], &[(warcaller, 1), (bears, 3)]);
    assert_eq!(game.pending_triggers.len(), 2);
    resolve_all(&mut game, dp);
    [1, 2, 3].map(|p| before[p - 1] - life(&game, p))
}

/// CR 508.5's two clauses can disagree at four seats, and the text says which
/// creature it means (CR 608.2c). "Defending player" for the trigger's
/// attacker: the Warcaller's trigger makes player 1 lose life, and the Bears'
/// makes player 3. The two triggers read different creatures, so player 0
/// orders them.
#[test]
fn defending_player_of_the_attacker_is_each_attackers_defender() {
    assert_eq!(warcaller_and_bears_attack(its_attacker(), &ordered_by(0)), [1, 0, 1]);
}

/// The same board with "defending player" for the source, "an ability of an
/// attacking creature": both triggers ask about the Warcaller, which attacks
/// player 1, so player 1 loses 2. The triggers read the same creature, so
/// nothing asks their order.
#[test]
fn defending_player_of_the_source_is_one_player_for_every_trigger() {
    assert_eq!(warcaller_and_bears_attack(Antecedent::ThisObject, &test_dp()), [2, 0, 0]);
}

/// "Whenever this creature attacks, tap target creature defending player
/// controls" (60 cards), and the same with the trigger's attacker (9 cards,
/// equipment and "a creature you control attacks"): at four seats only the
/// creature of the player attacked is a legal target, so it is the one
/// chosen, at the trigger's placement.
#[test]
fn a_targets_defending_player_is_read_at_placement() {
    let this_creatures = whenever(attacks(TriggerSubject::ThisObject), tap_target_creature_of(defending_player_of(Antecedent::ThisObject)));
    let its_attackers = whenever(attacks(creature_you_control()), tap_target_creature_of(defending_player_of(its_attacker())));
    for (def, attacked) in [(this_creatures, 2), (its_attackers, 3)] {
        let mut game = stocked(4);
        let theirs: Vec<ObjectId> = (1..4).map(|p| put_on_battlefield(&mut game, grizzly_bears(), p)).collect();
        let attacker = put_on_battlefield(&mut game, attacking_watcher("Fixture Binder", def), 0);
        declare(&mut game, &[attacker], &[(attacker, attacked)]);
        resolve_all(&mut game, &test_dp());
        let tapped: Vec<bool> = theirs.iter().map(|id| game.battlefield[id].tapped).collect();
        let expected: Vec<bool> = (1..4).map(|p| p == attacked).collect();
        assert_eq!(tapped, expected, "player {attacked}'s creature, the one defending player controls");
    }
}

/// "Whenever this creature attacks, if defending player controls a creature,
/// you gain 2 life", the source destroyed in response: CR 603.4 asks again at
/// resolution, and "defending player" is still the player it was attacking,
/// off its last known information (CR 508.5, 608.2h).
#[test]
fn an_intervening_if_reads_the_defending_player_of_a_creature_that_left() {
    let mut def = whenever(attacks(TriggerSubject::ThisObject), Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(2)), EffectRecipient::Controller));
    let creature = ObjectFilter::ByType(CardType::Creature);
    def.intervening_if =
        Some(Condition::DefendingPlayer { of: Antecedent::ThisObject, fact: PlayerFact::ControlsPermanent(creature) });
    let mut game = stocked(4);
    put_on_battlefield(&mut game, grizzly_bears(), 3);
    let source = put_on_battlefield(&mut game, attacking_watcher("Fixture Raider", def), 0);
    declare(&mut game, &[source], &[(source, 3)]);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(game.stack.len(), 1, "player 3 controls a creature as it attacks");

    let destroy = Effect::Atom(Primitive::Destroy, EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)));
    resolve_as(&mut game, 1, &destroy, source);
    let before = life(&game, 0);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0) - before, 2);
}

// "Defending player" with no creature to settle which (CR 802.2a): Yare's
// "target creature defending player controls". During player 0's combat
// every opponent is a defending player (CR 802.2), so a creature any of them
// controls is a legal target and player 0's is not; outside combat, nobody is
// a defending player.
#[test]
fn any_defending_player_is_each_opponent_of_the_attacking_player_in_combat() {
    let mut game = stocked(4);
    let creatures: Vec<ObjectId> = (0..4).map(|p| put_on_battlefield(&mut game, grizzly_bears(), p)).collect();
    let theirs =
        ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ByController(PlayerRef::AnyDefendingPlayer)));
    let target = EffectRecipient::Target(SelectionFilter::Permanent(theirs), TargetCount::Exactly(1));
    let legal = |game: &GameState| -> Vec<bool> {
        creatures
            .iter()
            .map(|&id| game.validate_targets(&target, &[ResolvedTarget::Object(id)], 1, FilterIdentity::NONE).is_ok())
            .collect()
    };
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(StepType::BeginCombat) });
    assert_eq!(legal(&game), vec![false, true, true, true], "before anything attacks");
    game.set_turn_position(Phase { phase_type: PhaseType::Postcombat, step: None });
    assert_eq!(legal(&game), vec![false; 4]);
}
