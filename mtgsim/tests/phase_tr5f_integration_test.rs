//! TR-5f — creatures put onto the battlefield attacking, and removal from
//! combat (`triggers-architecture.md` §12, TR-5f).
//!
//! 1. Removal from combat is a record, `RemovedFromCombat`, at each cause
//!    that removes a permanent and stays on the battlefield (CR 506.4,
//!    511.3), read against one board.
//! 2. "Defending player" once its creature has left combat (CR 508.5's
//!    second sentence), read off the combat's history.

use std::sync::Arc;

use mtgsim::cards::authoring::{attacks, triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::artifacts::sol_ring;
use mtgsim::cards::phase_ld_cards::march_of_the_machines;
use mtgsim::cards::phase_lg_cards::act_of_treason;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::{EventCause, GameRule, GeneratedBy};
use mtgsim::events::event::{CombatExit, EventRecord, GameEvent};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{
    fill_library, put_in_graveyard, put_on_battlefield, set_active_player, set_attacking, set_blocked_by, set_blocking, setup_game,
    setup_two_player_game, test_dp,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::oracle::characteristics::get_effective_controller;
use mtgsim::types::effects::{
    AmountExpr, Antecedent, Duration, Effect, EffectRecipient, ObjectFilter, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{ObjectId, ObjectRef, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::triggers::TriggerSubject;
use mtgsim::types::zones::Zone;
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

/// Put every waiting trigger on the stack and resolve the stack, answering
/// every prompt with `dp`.
fn resolve_all(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(dp).unwrap();
        game.perform_sba_and_triggers(dp).unwrap();
    }
}

/// `effect` resolving as `player`'s sorcery aimed at `target`, then the
/// state-based check and the control sweep that follow it.
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

/// "[Primitive] target creature".
fn at_target_creature(primitive: Primitive) -> Effect {
    Effect::Atom(primitive, EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)))
}

/// Act of Treason's resolution: `thief` gains control of `victim` until end
/// of turn, untaps it, and it gains haste.
fn steal(game: &mut GameState, thief: PlayerId, victim: ObjectId) {
    resolve_as(game, thief, &act_of_treason().abilities[0].effect, victim);
}

/// A regeneration shield on `creature`, then its destruction, which the
/// shield replaces (CR 701.19a).
fn regenerate(game: &mut GameState, player: PlayerId, creature: ObjectId) {
    resolve_as(game, player, &at_target_creature(Primitive::Regenerate), creature);
    resolve_as(game, player, &at_target_creature(Primitive::Destroy), creature);
}

/// Every `RemovedFromCombat` record, in order.
fn removals(game: &GameState) -> Vec<Vec<CombatExit>> {
    game.recorded_events()
        .events()
        .filter_map(|event| match event {
            GameEvent::RemovedFromCombat { removed } => Some(removed.clone()),
            _ => None,
        })
        .collect()
}

fn existence(game: &GameState, id: ObjectId) -> ObjectRef {
    game.object_ref(id).unwrap()
}

fn attacking(game: &GameState, id: ObjectId, player: PlayerId) -> CombatExit {
    CombatExit { object: existence(game, id), attacking: Some(AttackTarget::Player(player)), blocking: Vec::new() }
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// Advance the turn until `step` of the combat phase is the one in progress.
fn walk_to(game: &mut GameState, step: StepType) {
    let dp = test_dp();
    while !(game.phase.phase_type == PhaseType::Combat && game.phase.step == Some(step)) {
        game.advance_turn(&dp).unwrap();
    }
}

/// A 2/2 with haste that carries `effect` on "whenever this creature
/// attacks".
fn on_its_attack(name: &str, effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .color(Color::Red)
        .power_toughness(2, 2)
        .keyword_flag(KeywordFlag::Haste)
        .ability(triggered_ability("", whenever(attacks(TriggerSubject::ThisObject), effect)))
        .build()
}

// ---------------------------------------------------------------------------
// 1. The record
// ---------------------------------------------------------------------------

/// An effect that removes an attacker from combat announces it, with the
/// player it was attacking, and the attacker deals no combat damage.
// COVERS: ATOM-506.4-005
#[test]
fn an_effect_announces_what_it_removed_from_combat() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    declare(&mut game, &[bears], &[(bears, 1)]);

    resolve_as(&mut game, 1, &at_target_creature(Primitive::RemoveFromCombat), bears);
    assert!(game.battlefield[&bears].attacking.is_none());
    assert_eq!(removals(&game), vec![vec![attacking(&game, bears, 1)]]);
    let line = format!("RemovedFromCombat: [Grizzly Bears ({bears}) attacking P1]");
    assert!(format_event_log(&game).contains(&line), "no {line:?}");

    game.process_combat_damage(&test_dp(), false).unwrap();
    assert_eq!(life(&game, 1), 20, "a creature removed from combat deals no combat damage");
}

/// Regeneration's rider removes an attacking creature from combat (CR
/// 701.19a), through the same instruction, and announces it.
#[test]
fn regeneration_announces_its_removal() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    declare(&mut game, &[bears], &[(bears, 1)]);

    regenerate(&mut game, 0, bears);
    assert!(game.battlefield.contains_key(&bears), "the shield replaced the destruction");
    assert!(game.battlefield[&bears].attacking.is_none());
    assert_eq!(removals(&game), vec![vec![attacking(&game, bears, 1)]]);
}

/// A change of control removes the creature as part of the change (CR
/// 506.4): its removal is in the change's window, after the change.
#[test]
fn a_change_of_control_announces_its_removal_after_the_change() {
    let mut game = stocked(2);
    set_active_player(&mut game, 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    set_attacking(&mut game, bears, 0);

    steal(&mut game, 0, bears);
    let records = game.recorded_events();
    let tail: Vec<&GameEvent> = records
        .events()
        .filter(|event| matches!(event, GameEvent::ControlChanged { .. } | GameEvent::RemovedFromCombat { .. }))
        .collect();
    assert!(
        matches!(tail[..], [GameEvent::ControlChanged { .. }, GameEvent::RemovedFromCombat { .. }]),
        "the change, then its removal: {tail:?}"
    );
    assert_eq!(removals(&game), vec![vec![attacking(&game, bears, 0)]]);
}

/// CR 511.3: the end of combat removes every creature in it at once, as one
/// record. Each blocker still names the attacker it blocked, since every part
/// is read before anything is removed.
// COVERS: ATOM-511.3-001
#[test]
fn the_end_of_combat_removes_everything_at_once() {
    let mut game = stocked(2);
    let attacker = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let first = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let second = put_on_battlefield(&mut game, grizzly_bears(), 1);
    walk_to(&mut game, StepType::EndCombat);
    set_attacking(&mut game, attacker, 1);
    set_blocked_by(&mut game, attacker, vec![first, second]);
    set_blocking(&mut game, first, vec![attacker]);
    set_blocking(&mut game, second, vec![attacker]);

    game.advance_turn(&test_dp()).unwrap();
    assert_eq!(game.phase.phase_type, PhaseType::Postcombat);
    let blocking = |id| CombatExit { object: existence(&game, id), attacking: None, blocking: vec![attacker] };
    assert_eq!(removals(&game), vec![vec![attacking(&game, attacker, 1), blocking(first), blocking(second)]]);
    for id in [attacker, first, second] {
        let entry = &game.battlefield[&id];
        assert!(entry.attacking.is_none() && entry.blocking.is_none(), "{id} is out of combat");
    }
}

/// CR 506.4's first cause: an attacker returned to its owner's hand leaves
/// combat with the battlefield. Its zone change announces it, its frame
/// keeping what it was attacking, and no removal record repeats it.
// COVERS: ATOM-506.4-001
#[test]
fn leaving_the_battlefield_removes_it_with_no_removal_record() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    declare(&mut game, &[bears], &[(bears, 1)]);

    resolve_as(&mut game, 1, &at_target_creature(Primitive::ReturnToHand), bears);
    assert_eq!(game.objects[&bears].zone, Zone::Hand);
    assert!(removals(&game).is_empty());
    let records = game.recorded_events();
    let left = records
        .events()
        .find_map(|event| match event {
            GameEvent::ZoneChange { from: Zone::Battlefield, lki: Some(lki), .. } => Some(lki.attacking()),
            _ => None,
        })
        .expect("the zone change");
    assert_eq!(left, Some(AttackTarget::Player(1)), "its frame keeps the attack");
    game.process_combat_damage(&test_dp(), false).unwrap();
    assert_eq!(life(&game, 1), 20);
}

// ---------------------------------------------------------------------------
// 2. "Defending player" once its creature has left combat
// ---------------------------------------------------------------------------

/// "Whenever this creature attacks, defending player loses 1 life."
fn warcaller() -> Arc<CardData> {
    on_its_attack(
        "Fixture Warcaller",
        Effect::Atom(Primitive::LoseLife(AmountExpr::Fixed(1)), EffectRecipient::DefendingPlayer(Antecedent::ThisObject)),
    )
}

/// The Warcaller attacks player 3 of four, and `leave` takes it out of combat
/// while its trigger waits. Returns the life players 1, 2 and 3 lost once
/// the trigger resolves.
fn warcaller_leaves_combat(leave: impl Fn(&mut GameState, ObjectId)) -> [i64; 3] {
    let mut game = stocked(4);
    let warcaller = put_on_battlefield(&mut game, warcaller(), 0);
    declare(&mut game, &[warcaller], &[(warcaller, 3)]);
    assert_eq!(game.pending_triggers.len(), 1);
    leave(&mut game, warcaller);
    assert!(game.battlefield.get(&warcaller).is_none_or(|entry| entry.attacking.is_none()), "it left combat");
    let before = [1, 2, 3].map(|p| life(&game, p));
    resolve_all(&mut game, &test_dp());
    [1, 2, 3].map(|p| before[p - 1] - life(&game, p))
}

/// CR 508.5's second sentence: "defending player" for a creature no longer
/// attacking is the player it was attacking before it was removed from
/// combat. At four seats nothing else could say which: the Warcaller is
/// stolen by player 2, removed by player 1's effect, or regenerated, and in
/// each case player 3 loses the life.
#[test]
fn defending_player_after_a_removal_is_the_player_it_was_attacking() {
    let stolen = warcaller_leaves_combat(|game, warcaller| steal(game, 2, warcaller));
    let removed = warcaller_leaves_combat(|game, warcaller| {
        resolve_as(game, 1, &at_target_creature(Primitive::RemoveFromCombat), warcaller)
    });
    let regenerated = warcaller_leaves_combat(|game, warcaller| regenerate(game, 0, warcaller));
    assert_eq!([stolen, removed, regenerated], [[0, 0, 1]; 3]);
}

/// Stolen, which removes it from combat, and then killed, both before its
/// trigger resolves: its last known information keeps no attack, and the
/// history's removal row for that existence names player 3.
#[test]
fn a_creature_stolen_then_killed_still_names_its_former_defender() {
    let lost = warcaller_leaves_combat(|game, warcaller| {
        steal(game, 2, warcaller);
        resolve_as(game, 1, &at_target_creature(Primitive::Destroy), warcaller);
        assert_eq!(game.objects[&warcaller].zone, Zone::Graveyard);
    });
    assert_eq!(lost, [0, 0, 1]);
}

// ---------------------------------------------------------------------------
// 3. The settlement: a combatant that stops being a creature
// ---------------------------------------------------------------------------

fn destroy_target_enchantment() -> Effect {
    let enchantment = SelectionFilter::Permanent(ObjectFilter::ByType(CardType::Enchantment));
    Effect::Atom(Primitive::Destroy, EffectRecipient::Target(enchantment, TargetCount::Exactly(1)))
}

/// The first record in `window` that `is`, and the batch it was performed in.
fn first_in<'a>(window: &'a [EventRecord], is: impl Fn(&GameEvent) -> bool) -> &'a EventRecord {
    window.iter().find(|record| is(&record.event)).expect("the record")
}

/// March of the Machines makes Sol Ring a 1/1 artifact creature, which
/// attacks. Destroying March ends that, and CR 506.4 removes the artifact from
/// combat as it stops being a creature: in the destruction's window, as part
/// of the event that caused it, so the window's triggers read the board
/// without it. It deals no combat damage.
// COVERS: ATOM-506.4-003
#[test]
fn an_attacker_that_stops_being_a_creature_leaves_combat() {
    let mut game = stocked(2);
    let march = put_on_battlefield(&mut game, march_of_the_machines(), 0);
    let ring = put_on_battlefield(&mut game, sol_ring(), 0);
    declare(&mut game, &[ring], &[(ring, 1)]);
    let before = game.recorded_events().len();

    resolve_as(&mut game, 1, &destroy_target_enchantment(), march);
    assert!(game.battlefield[&ring].attacking.is_none(), "CR 506.4: it stopped being a creature");
    let records = game.recorded_events();
    let window = records.records_from(before);
    let destroyed = first_in(window, |event| matches!(event, GameEvent::ZoneChange { object_id, .. } if *object_id == march));
    let removal = first_in(window, |event| matches!(event, GameEvent::RemovedFromCombat { .. }));
    assert_eq!(removal.stamp.batch, destroyed.stamp.batch, "one event with its cause");
    assert_eq!(removal.stamp.cause, EventCause::GameRule(GameRule::CombatantTypeChanged));
    assert_eq!(removal.event, GameEvent::RemovedFromCombat { removed: vec![attacking(&game, ring, 1)] });

    game.process_combat_damage(&test_dp(), false).unwrap();
    assert_eq!(life(&game, 1), 20);
}

/// The same for a blocker: the animated Sol Ring blocking Grizzly Bears
/// leaves combat when March goes, and the Bears stay blocked by nothing (CR
/// 509.1h), so they deal no damage.
#[test]
fn a_blocker_that_stops_being_a_creature_leaves_and_its_attacker_stays_blocked() {
    let mut game = stocked(2);
    let march = put_on_battlefield(&mut game, march_of_the_machines(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let ring = put_on_battlefield(&mut game, sol_ring(), 1);
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(StepType::DeclareBlockers) });
    game.combat_had_attackers = true;
    set_attacking(&mut game, bears, 1);
    set_blocked_by(&mut game, bears, vec![ring]);
    set_blocking(&mut game, ring, vec![bears]);

    resolve_as(&mut game, 0, &destroy_target_enchantment(), march);
    assert!(game.battlefield[&ring].blocking.is_none());
    let info = game.battlefield[&bears].attacking.as_ref().expect("still attacking");
    assert!(info.is_blocked && info.blocked_by.is_empty(), "CR 509.1h: blocked, by nothing");
    let exit = CombatExit { object: existence(&game, ring), attacking: None, blocking: vec![bears] };
    assert_eq!(removals(&game), vec![vec![exit]]);
    game.process_combat_damage(&test_dp(), false).unwrap();
    assert_eq!(life(&game, 1), 20);
}

/// The board the fuzz probe found at four seats (`performance` seed 12525):
/// the player whose March of the Machines animates the attacking player's
/// artifact loses, and leaves the game (CR 800.4a) with March, so the
/// artifact stops being a creature and leaves combat.
#[test]
fn a_player_leaving_with_the_animator_takes_the_attacker_out_of_combat() {
    let mut game = stocked(4);
    put_on_battlefield(&mut game, march_of_the_machines(), 2);
    let ring = put_on_battlefield(&mut game, sol_ring(), 0);
    declare(&mut game, &[ring], &[(ring, 1)]);

    game.players[2].life_total = 0;
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert!(!game.in_game(2));
    assert!(game.battlefield[&ring].attacking.is_none());
    assert_eq!(removals(&game), vec![vec![attacking(&game, ring, 1)]]);
}

/// Decision 13's other half: a control change a batch causes is settled at
/// the batch's close, so it is announced in the batch's window. Player 0
/// takes the Bears for as long as its Fixture Sower remains; destroying the
/// Sower gives them back in the destruction's window.
#[test]
fn a_change_of_control_a_batch_causes_is_announced_in_its_window() {
    let mut game = stocked(2);
    let sower = put_on_battlefield(&mut game, CardDataBuilder::new("Fixture Sower").card_type(CardType::Creature).power_toughness(1, 1).build(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let ctx = ResolutionContext {
        source: sower,
        ability_source: game.object_ref(sower),
        controller: 0,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(bears)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    let take = at_target_creature(Primitive::GainControl(Duration::WhileSourceOnBattlefield));
    game.resolve_effect(&take, &ctx, &test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(get_effective_controller(&game, bears), Some(0));
    let before = game.recorded_events().len();

    resolve_as(&mut game, 1, &at_target_creature(Primitive::Destroy), sower);
    assert_eq!(get_effective_controller(&game, bears), Some(1));
    let records = game.recorded_events();
    let window = records.records_from(before);
    let destroyed = first_in(window, |event| matches!(event, GameEvent::ZoneChange { object_id, .. } if *object_id == sower));
    let back = first_in(window, |event| matches!(event, GameEvent::ControlChanged { object_id, .. } if *object_id == bears));
    assert_eq!(back.stamp.batch, destroyed.stamp.batch);
}
