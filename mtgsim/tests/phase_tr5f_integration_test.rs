//! TR-5f — creatures put onto the battlefield attacking, and removal from
//! combat (`triggers-architecture.md` §12, TR-5f).
//!
//! 1. Removal from combat is a record, `RemovedFromCombat`, at each cause
//!    that removes a permanent and stays on the battlefield (CR 506.4,
//!    511.3), read against one board.
//! 2. "Defending player" once its creature has left combat (CR 508.5's
//!    second sentence), read off the combat's history.

use std::sync::Arc;

use mtgsim::cards::authoring::{attacks, attacks_alone, is_attacked, player_attacks, triggered_ability, whenever, Whose};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::artifacts::sol_ring;
use mtgsim::cards::phase_ld_cards::march_of_the_machines;
use mtgsim::cards::phase_lg_cards::act_of_treason;
use mtgsim::cards::phase_re_cards::{divine_visitation, parallel_lives};
use mtgsim::cards::phase_tr5e_cards::hellrider;
use mtgsim::cards::phase_tr5f_cards::{attacking_human_token, hanweir_garrison, relentless_assault};
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::layers::compute_characteristics;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::{EventCause, GameRule, GeneratedBy};
use mtgsim::events::event::{CombatExit, DepartedObject, EventRecord, GameEvent};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::history_log::RowFact;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{
    fill_library, put_in_graveyard, put_in_hand, put_on_battlefield, static_ability, test_ctx, set_active_player, set_attacking, set_blocked_by, set_blocking, setup_game,
    setup_two_player_game, test_dp, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::history::ObjectFact;
use mtgsim::types::colors::Color;
use mtgsim::oracle::characteristics::get_effective_controller;
use mtgsim::types::effects::{
    AmountExpr, Antecedent, Condition, CounterType, Duration, Effect, EffectRecipient, EntersAttacking, ObjectFilter, PlayerFact,
    PlayerRef, PlayerSet, Primitive, TypeChange,
    ReturnUnder, SelectionFilter, TargetCount, TokenDef,
};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, ObjectRef, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::triggers::{ParticipantFilter, TriggerEvent, TriggerSubject};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::display::{format_event_log, format_record, log_lines};

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
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Object(target)]),
        replaced_event: None,
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
        .ability(triggered_ability("", whenever(attacks(TriggerSubject::Object(Antecedent::ThisObject)), effect)))
        .build()
}

/// An enchantment with one triggered ability.
fn watcher(name: &str, arm: impl Into<TriggerEvent>, effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new(name).card_type(CardType::Enchantment).ability(triggered_ability("", whenever(arm, effect))).build()
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
    let blocking = |id| CombatExit { object: existence(&game, id), attacking: None, blocking: vec![existence(&game, attacker)] };
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
            GameEvent::ZoneChange { from: Zone::Battlefield, departed: DepartedObject::WithLki(lki), .. } => Some(lki.attacking()),
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
        Effect::Atom(Primitive::LoseLife(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::DefendingPlayer(Antecedent::ThisObject))),
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
fn first_in(window: &[EventRecord], is: impl Fn(&GameEvent) -> bool) -> &EventRecord {
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
    let destroyed = first_in(window, |event| matches!(event, GameEvent::ZoneChange { departed, .. } if departed.object().id == march));
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
    let exit = CombatExit { object: existence(&game, ring), attacking: None, blocking: vec![existence(&game, bears)] };
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
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Object(bears)]),
        replaced_event: None,
        trigger: None,
    };
    let take = at_target_creature(Primitive::GainControl { to: PlayerRef::You, duration: Duration::WhileSourceOnBattlefield });
    game.resolve_effect(&take, &ctx, &test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(get_effective_controller(&game, bears), Some(0));
    let before = game.recorded_events().len();

    resolve_as(&mut game, 1, &at_target_creature(Primitive::Destroy), sower);
    assert_eq!(get_effective_controller(&game, bears), Some(1));
    let records = game.recorded_events();
    let window = records.records_from(before);
    let destroyed = first_in(window, |event| matches!(event, GameEvent::ZoneChange { departed, .. } if departed.object().id == sower));
    let back = first_in(window, |event| matches!(event, GameEvent::ControlChanged { lki, .. } if lki.object().id == bears));
    assert_eq!(back.stamp.batch, destroyed.stamp.batch);
}

// ---------------------------------------------------------------------------
// 4. Item 128: creatures put onto the battlefield attacking (CR 508.4)
// ---------------------------------------------------------------------------

/// The tokens on the battlefield, in the order they entered.
fn tokens(game: &GameState) -> Vec<ObjectId> {
    game.battlefield_ids_ordered().into_iter().filter(|id| game.objects[id].is_token).collect()
}

fn attack_of(game: &GameState, id: ObjectId) -> Option<AttackTarget> {
    game.battlefield.get(&id).and_then(|entry| entry.attacking.as_ref()).map(|attack| attack.target)
}

/// A 1/1 created tapped and attacking as `entering` says.
fn a_human(entering: EntersAttacking) -> TokenDef {
    TokenDef { enters_attacking: Some(entering), ..attacking_human_token() }
}

/// `n` of `def` created by `player`'s effect, its prompts answered by `dp`.
fn create(game: &mut GameState, player: PlayerId, def: TokenDef, n: u64, dp: &dyn DecisionProvider) {
    let effect = Effect::Atom(Primitive::CreateToken(def, AmountExpr::Fixed(n)), EffectRecipient::Player(PlayerRef::You));
    let source = put_in_graveyard(game, act_of_treason(), player);
    let ctx = ResolutionContext::untargeted(source, player, GeneratedBy::Spell);
    game.resolve_effect(&effect, &ctx, dp).unwrap();
}

fn at(game: &mut GameState, step: StepType) {
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(step) });
}

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You))
}

/// A 1/1 token created tapped and attacking under `player`, and its id.
fn a_token_tapped_and_attacking(game: &mut GameState, player: PlayerId) -> ObjectId {
    create(game, player, a_human(EntersAttacking::Chosen), 1, &test_dp());
    tokens(game).last().copied().unwrap()
}

/// A 1/1 creature card put from `player`'s hand onto the battlefield tapped
/// and attacking through the return door, Preeminent Captain's "put a
/// Soldier creature card from your hand onto the battlefield tapped and
/// attacking", and its id.
fn a_card_from_hand_tapped_and_attacking(game: &mut GameState, player: PlayerId) -> ObjectId {
    let soldier = CardDataBuilder::new("Fixture Hand Soldier")
        .card_type(CardType::Creature)
        .color(Color::White)
        .power_toughness(1, 1)
        .build();
    let card = put_in_hand(game, soldier, player);
    let puts = Effect::Atom(
        Primitive::ReturnToBattlefield {
            under: ReturnUnder::Owner,
            counters: Vec::new(),
            tapped: true,
            attacking: Some(EntersAttacking::Chosen),
        },
        EffectRecipient::Choose(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    resolve_as(game, player, &puts, card);
    assert_eq!(game.objects[&card].zone, Zone::Battlefield);
    card
}

/// CR 508.4 is written about any creature put onto the battlefield
/// attacking, so the rules that follow it are tested both ways in.
const WAYS_ONTO_THE_BATTLEFIELD_ATTACKING: [(&str, fn(&mut GameState, PlayerId) -> ObjectId); 2] = [
    ("a token", a_token_tapped_and_attacking),
    ("a card from a hand", a_card_from_hand_tapped_and_attacking),
];

fn entering_defender_prompts(dp: &RecordingDecisionProvider) -> usize {
    dp.kinds().iter().filter(|kind| kind.as_str() == "ChooseEnteringDefender").count()
}

/// Hanweir Garrison attacks beside Hellrider. Its two tokens enter tapped
/// and attacking player 1, with no prompt at two seats, but they were never
/// declared (CR 508.4): Hellrider triggers for the two creatures declared and
/// not for them (CR 508.3a), and neither token has an `Attacked` row.
// RULING: Hanweir Garrison #2 - the tokens are attacking, and were never declared as attacking creatures.
// COVERS: ATOM-508.4-001
#[test]
fn a_token_put_onto_the_battlefield_attacking_never_attacked() {
    let mut game = stocked(2);
    let hellrider = put_on_battlefield(&mut game, hellrider(), 0);
    let garrison = put_on_battlefield(&mut game, hanweir_garrison(), 0);
    declare(&mut game, &[hellrider, garrison], &[(hellrider, 1), (garrison, 1)]);
    assert_eq!(game.pending_triggers.len(), 3, "Hellrider's two and the Garrison's");

    let dp = RecordingDecisionProvider::picking(0);
    resolve_all(&mut game, &dp);
    let humans = tokens(&game);
    assert_eq!(humans.len(), 2);
    for &human in &humans {
        assert_eq!(attack_of(&game, human), Some(AttackTarget::Player(1)));
        assert!(game.battlefield[&human].tapped);
    }
    assert_eq!(entering_defender_prompts(&dp), 0, "one player to attack, so nothing to choose");
    mtgsim::test_support::assert_not_asked(&game, 0, "ChooseEnteringDefender", "its one legal answer");
    assert_eq!(life(&game, 1), 18, "Hellrider's two triggers, and none for the tokens");
    let attacked = |id| {
        let object = Some(existence(&game, id));
        game.history.log.rows_of_turn(game.turn_number).any(|row| row.object == object && matches!(row.fact, RowFact::Attacked(_)))
    };
    assert!(attacked(hellrider) && attacked(garrison));
    assert!(humans.iter().all(|&human| !attacked(human)), "no declaration, no `Attacked` row");
}

/// CR 508.3b and 508.3d read the declaration too: "whenever a player is
/// attacked" and "whenever you attack" trigger once each, for the Garrison,
/// and the tokens joining the attack trigger neither again.
#[test]
fn a_token_put_onto_the_battlefield_attacking_triggers_no_defender_or_player() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, watcher("Fixture Watchtower", is_attacked(ParticipantFilter::Player(PlayerSet::Everyone)), gain_one()), 0);
    put_on_battlefield(&mut game, watcher("Fixture War Drum", player_attacks(Whose::Yours), gain_one()), 0);
    let garrison = put_on_battlefield(&mut game, hanweir_garrison(), 0);
    declare(&mut game, &[garrison], &[(garrison, 1)]);

    resolve_all(&mut game, &RecordingDecisionProvider::picking(0));
    assert_eq!(tokens(&game).len(), 2);
    assert_eq!(life(&game, 0), 22, "each watcher once");
}

/// CR 506.5 counts the creatures declared. A creature declared alone attacks
/// alone though a creature was put onto the battlefield attacking before the
/// declaration, and one put onto the battlefield attacking never attacks,
/// alone or otherwise: a token or a card.
#[test]
fn attacks_alone_counts_only_the_declared() {
    for (way, put_attacking) in WAYS_ONTO_THE_BATTLEFIELD_ATTACKING {
        let mut game = stocked(2);
        put_on_battlefield(&mut game, watcher("Fixture Lone Banner", attacks_alone(TriggerSubject::Filter(ObjectFilter::All)), gain_one()), 0);
        let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
        at(&mut game, StepType::BeginCombat);
        let entered = put_attacking(&mut game, 0);
        assert_eq!(attack_of(&game, entered), Some(AttackTarget::Player(1)), "{way}");

        declare(&mut game, &[bears], &[(bears, 1)]);
        resolve_all(&mut game, &test_dp());
        assert_eq!(life(&game, 0), 21, "{way}: the Bears attacked alone, and the other did not attack");
    }
}

/// Hanweir Garrison's first ruling, at four seats: each token's defender is
/// chosen as the tokens are created, and needn't be the Garrison's or the
/// other token's.
// RULING: Hanweir Garrison #1 - each token's defender is chosen as it is created, and needn't match.
#[test]
fn each_token_chooses_what_it_attacks_at_four_seats() {
    let mut game = stocked(4);
    let garrison = put_on_battlefield(&mut game, hanweir_garrison(), 0);
    declare(&mut game, &[garrison], &[(garrison, 1)]);

    let dp = ScriptedDecisionProvider::new();
    let asked = ChoiceKind::ChooseEnteringDefender { object: ObjectId::UNASSIGNED };
    dp.expect_choice(asked.clone(), vec![ChoiceOption::Player(2)]);
    dp.expect_choice(asked, vec![ChoiceOption::Player(3)]);
    resolve_all(&mut game, &dp);
    let attacks: Vec<_> = tokens(&game).into_iter().map(|token| attack_of(&game, token)).collect();
    assert_eq!(attacks, [Some(AttackTarget::Player(2)), Some(AttackTarget::Player(3))]);
    assert_eq!(attack_of(&game, garrison), Some(AttackTarget::Player(1)));
}

/// CR 506.3a: a noncreature put onto the battlefield attacking enters, and
/// is never an attacking permanent.
// COVERS: ATOM-506.3a-001
#[test]
fn a_noncreature_token_enters_and_does_not_attack() {
    let mut game = stocked(2);
    at(&mut game, StepType::DeclareAttackers);
    let artifact = TokenDef {
        types: vec![CardType::Artifact],
        subtypes: Vec::new(),
        power: None,
        toughness: None,
        ..a_human(EntersAttacking::Chosen)
    };
    create(&mut game, 0, artifact, 1, &test_dp());
    let [token] = tokens(&game)[..] else { panic!("one token") };
    assert_eq!(attack_of(&game, token), None);
    assert!(!game.combat_had_attackers);
}

/// CR 506.3b: a creature put onto the battlefield attacking under a player
/// who is not attacking enters, and is never an attacking creature.
// COVERS: ATOM-506.3b-001
#[test]
fn a_token_under_a_player_who_is_not_attacking_does_not_attack() {
    let mut game = stocked(2);
    at(&mut game, StepType::DeclareAttackers);
    create(&mut game, 1, a_human(EntersAttacking::Chosen), 1, &test_dp());
    let [token] = tokens(&game)[..] else { panic!("one token") };
    assert_eq!(attack_of(&game, token), None);
}

/// CR 506.3c and 508.4a: "attacking the same player" as a creature attacking
/// player 3, after player 3 has left the game: the token enters, and is never
/// an attacking creature.
// COVERS-PARTIAL: ATOM-506.3c-001
#[test]
fn a_token_whose_player_has_left_the_game_does_not_attack() {
    let mut game = stocked(4);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    at(&mut game, StepType::DeclareAttackers);
    set_attacking(&mut game, bears, 3);
    game.players[3].life_total = 0;
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert!(!game.in_game(3));

    let effect = Effect::Atom(
        Primitive::CreateToken(a_human(EntersAttacking::SameAs(Antecedent::ThisObject)), AmountExpr::Fixed(1)),
        EffectRecipient::Player(PlayerRef::You),
    );
    let ctx = ResolutionContext {
        ability_source: game.object_ref(bears),
        ..ResolutionContext::untargeted(bears, 0, GeneratedBy::Spell)
    };
    game.resolve_effect(&effect, &ctx, &test_dp()).unwrap();
    let [token] = tokens(&game)[..] else { panic!("one token") };
    assert_eq!(attack_of(&game, token), None);
}

/// CR 506.3d and 508.4d: put onto the battlefield attacking after blockers
/// are declared, in the declare blockers, combat damage or end of combat
/// step, it enters unblocked. In the declare blockers step it then deals its
/// combat damage.
// COVERS: ATOM-506.3d-001, ATOM-506.3d-002, ATOM-506.3d-003
#[test]
fn a_token_put_onto_the_battlefield_attacking_after_blocks_is_unblocked() {
    for step in [StepType::DeclareBlockers, StepType::CombatDamage, StepType::EndCombat] {
        let mut game = stocked(2);
        at(&mut game, step);
        create(&mut game, 0, a_human(EntersAttacking::Chosen), 1, &test_dp());
        let [token] = tokens(&game)[..] else { panic!("one token") };
        let attack = game.battlefield[&token].attacking.clone().expect("attacking");
        assert!(!attack.is_blocked && attack.blocked_by.is_empty(), "{step:?}: unblocked");
        if step == StepType::DeclareBlockers {
            game.process_combat_damage(&test_dp(), false).unwrap();
            assert_eq!(life(&game, 1), 19);
        }
    }
}

/// CR 508.8's second leg: with nothing declared, a creature put onto the
/// battlefield attacking in the beginning of combat step still makes the
/// declare blockers and combat damage steps begin, and it deals its damage,
/// a token or a card.
#[test]
fn a_creature_put_onto_the_battlefield_attacking_makes_combat_happen() {
    for (way, put_attacking) in WAYS_ONTO_THE_BATTLEFIELD_ATTACKING {
        let mut game = stocked(2);
        walk_to(&mut game, StepType::BeginCombat);
        put_attacking(&mut game, 0);
        assert!(game.combat_had_attackers, "{way}");
        walk_to(&mut game, StepType::CombatDamage);
        let records = game.recorded_events();
        let began = |step| records.events().any(|event| matches!(event, GameEvent::StepBegin { step: s, .. } if *s == step));
        assert!(began(StepType::DeclareBlockers) && began(StepType::CombatDamage), "{way}");
        game.process_combat_damage(&test_dp(), false).unwrap();
        assert_eq!(life(&game, 1), 19, "{way}");
    }
}

/// "Attacking that player", read as the instruction resolves: "whenever this
/// creature attacks, create a token tapped and attacking defending player",
/// and "... attacking the same player", at four seats attack the player the
/// creature attacks, with no prompt.
#[test]
fn a_token_attacking_that_player_attacks_the_player_named() {
    let defending = EntersAttacking::Player {
        player: EffectRecipient::Player(PlayerRef::DefendingPlayer(Antecedent::ThisObject)),
        among: ParticipantFilter::Player(PlayerSet::Everyone),
    };
    let same = EntersAttacking::SameAs(Antecedent::ThisObject);
    for entering in [defending, same] {
        let mut game = stocked(4);
        let effect =
            Effect::Atom(Primitive::CreateToken(a_human(entering.clone()), AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You));
        let leader = put_on_battlefield(&mut game, on_its_attack("Fixture Vanguard", effect), 0);
        declare(&mut game, &[leader], &[(leader, 2)]);
        resolve_all(&mut game, &test_dp());
        let [token] = tokens(&game)[..] else { panic!("one token") };
        assert_eq!(attack_of(&game, token), Some(AttackTarget::Player(2)), "{entering:?}");
    }
}

/// Parallel Lives' second ruling: the extra tokens are tapped and attacking
/// too, and at four seats each is asked what it attacks.
#[test]
fn parallel_lives_doubles_the_attacking_tokens() {
    let mut game = stocked(4);
    put_on_battlefield(&mut game, parallel_lives(), 0);
    let garrison = put_on_battlefield(&mut game, hanweir_garrison(), 0);
    declare(&mut game, &[garrison], &[(garrison, 1)]);

    let dp = RecordingDecisionProvider::picking(0);
    resolve_all(&mut game, &dp);
    let humans = tokens(&game);
    assert_eq!(humans.len(), 4);
    assert!(humans.iter().all(|&human| attack_of(&game, human) == Some(AttackTarget::Player(1))));
    assert_eq!(entering_defender_prompts(&dp), 4, "one choice for each token");
}

/// Divine Visitation's first ruling: the Angel the creation becomes keeps
/// "tapped and attacking".
#[test]
fn divine_visitations_angels_are_tapped_and_attacking() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, divine_visitation(), 0);
    let garrison = put_on_battlefield(&mut game, hanweir_garrison(), 0);
    declare(&mut game, &[garrison], &[(garrison, 1)]);

    resolve_all(&mut game, &RecordingDecisionProvider::picking(0));
    let angels = tokens(&game);
    assert_eq!(angels.len(), 2);
    for angel in angels {
        assert_eq!(game.objects[&angel].card_data.power, Some(4), "the Angel, not the Human");
        assert!(game.battlefield[&angel].tapped);
        assert_eq!(attack_of(&game, angel), Some(AttackTarget::Player(1)));
    }
}

/// The return door takes the same words: "return this card from your
/// graveyard to the battlefield tapped and attacking" (Phoenix Chick's
/// shape), during combat. It was never declared, so it never attacked (CR
/// 508.4).
#[test]
fn a_card_returned_tapped_and_attacking_is_tapped_and_attacking() {
    let mut game = stocked(2);
    let bears = put_in_graveyard(&mut game, grizzly_bears(), 0);
    at(&mut game, StepType::DeclareAttackers);
    let returns = Effect::Atom(
        Primitive::ReturnToBattlefield { under: ReturnUnder::Owner, counters: Vec::new(), tapped: true, attacking: Some(EntersAttacking::Chosen) },
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    resolve_as(&mut game, 0, &returns, bears);
    assert_eq!(attack_of(&game, bears), Some(AttackTarget::Player(1)));
    assert!(game.battlefield[&bears].tapped);
    let bears_now = Some(existence(&game, bears));
    assert!(!game.history.log.rows_of_turn(game.turn_number).any(|row| row.object == bears_now && matches!(row.fact, RowFact::Attacked(_))));
}

/// Hanweir Garrison's third ruling: another attack trigger goes on the stack
/// beside its trigger, in either order, and a targeted one chooses its
/// target as it goes, before the tokens exist (CR 603.3d), so it cannot
/// choose them.
// RULING: Hanweir Garrison #3 - another attack trigger in either order, and it can't target the tokens.
#[test]
fn an_attack_trigger_beside_it_cannot_target_the_tokens() {
    for order in [0, 1] {
        let mut game = stocked(2);
        let counter_on_target = Effect::Atom(
            Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
            EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
        );
        let captain = put_on_battlefield(&mut game, on_its_attack("Fixture Captain", counter_on_target), 0);
        let garrison = put_on_battlefield(&mut game, hanweir_garrison(), 0);
        declare(&mut game, &[captain, garrison], &[(captain, 1), (garrison, 1)]);
        resolve_all(&mut game, &RecordingDecisionProvider::picking(order));
        let humans = tokens(&game);
        assert_eq!(humans.len(), 2);
        let counters = |id| game.battlefield[&id].counter_count(CounterType::PlusOnePlusOne);
        assert!(humans.iter().all(|&human| counters(human) == 0), "order {order}");
        assert_eq!(counters(captain) + counters(garrison), 1);
    }
}

// ---------------------------------------------------------------------------
// 5. The log leaves a rule's sweep implicit (decisions 12 and 16)
// ---------------------------------------------------------------------------

/// Player 0's tapped Grizzly Bears attack in turn 1 and are removed from
/// combat as it ends (CR 511.3), and player 0's next untap step, in turn 3,
/// untaps them and a tapped Forest-like fixture (CR 502.3). Returns the game
/// at turn 3's upkeep.
fn a_combat_then_an_untap_step(watching: Option<Arc<CardData>>) -> GameState {
    let mut game = stocked(2);
    if let Some(watching) = watching {
        put_on_battlefield(&mut game, watching, 0);
    }
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let land = put_on_battlefield(&mut game, CardDataBuilder::new("Fixture Grove").card_type(CardType::Land).build(), 0);
    walk_to(&mut game, StepType::EndCombat);
    set_attacking(&mut game, bears, 1);
    for id in [bears, land] {
        game.battlefield.get_mut(&id).unwrap().tapped = true;
    }
    let dp = test_dp();
    while !(game.turn_number == 3 && game.phase.step == Some(StepType::Upkeep)) {
        game.advance_turn(&dp).unwrap();
    }
    game
}

fn lines_in(game: &GameState, records: &[&EventRecord]) -> Vec<String> {
    records.iter().map(|record| format_record(game, record)).collect()
}

/// The end of combat's removal and the untap step's untaps are records, which
/// the full event log keeps, and no line in the game's log.
#[test]
fn the_log_leaves_a_rules_sweep_implicit() {
    let game = a_combat_then_an_untap_step(None);
    let recorded = game.recorded_events();
    let all: Vec<&EventRecord> = recorded.records().iter().collect();
    let everything = lines_in(&game, &all);
    let shown = lines_in(&game, &log_lines(&game, recorded.records()));
    let is_sweep = |line: &String| line.starts_with("RemovedFromCombat") || line.starts_with("Untapped");
    assert_eq!(everything.iter().filter(|line| is_sweep(line)).count(), 3, "the removal and two untaps: {everything:#?}");
    assert!(!shown.iter().any(is_sweep), "{shown:#?}");
    assert_eq!(shown.len(), everything.len() - 3);
}

/// A sweep a triggered ability read is a line: "whenever a permanent you
/// control becomes untapped" reads each untap, and the untap step's event
/// shows whole, while the end of combat, which nothing read, stays implicit.
#[test]
fn a_sweep_a_trigger_reads_is_a_line() {
    let yours = TriggerSubject::Filter(ObjectFilter::ByController(PlayerSet::You));
    let watching = watcher("Fixture Dawn Bell", TriggerEvent::BecomesUntapped { subject: yours }, gain_one());
    let game = a_combat_then_an_untap_step(Some(watching));
    let recorded = game.recorded_events();
    let shown = lines_in(&game, &log_lines(&game, recorded.records()));
    assert_eq!(shown.iter().filter(|line| line.starts_with("Untapped")).count(), 2, "{shown:#?}");
    assert!(!shown.iter().any(|line| line.starts_with("RemovedFromCombat")));
}

// ---------------------------------------------------------------------------
// 6. Relentless Assault: "creatures that attacked this turn"
// ---------------------------------------------------------------------------

/// Relentless Assault's effect resolving as `player`'s spell.
fn relentless_assault_resolves(game: &mut GameState, player: PlayerId) {
    let card = relentless_assault();
    let source = put_in_graveyard(game, card.clone(), player);
    let ctx = ResolutionContext::untargeted(source, player, GeneratedBy::Spell);
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
}

/// Advance the turn until a main phase is the one in progress.
fn walk_to_postcombat_main(game: &mut GameState) {
    let dp = test_dp();
    while game.phase.phase_type != PhaseType::Postcombat {
        game.advance_turn(&dp).unwrap();
    }
}

/// Hellrider and Hanweir Garrison attack, and the Garrison's tokens join them
/// tapped. In the postcombat main phase Relentless Assault untaps the two
/// that attacked, and not the tokens, which never attacked (CR 508.4). In the
/// second combat Hellrider attacks again and its trigger fires again
/// (Relentless Assault's fourth ruling).
// RULING: Relentless Assault #4 - attack triggers trigger during each combat that turn.
#[test]
fn relentless_assault_untaps_what_attacked_and_attack_triggers_fire_again() {
    let mut game = stocked(2);
    let hellrider = put_on_battlefield(&mut game, hellrider(), 0);
    let garrison = put_on_battlefield(&mut game, hanweir_garrison(), 0);
    declare(&mut game, &[hellrider, garrison], &[(hellrider, 1), (garrison, 1)]);
    resolve_all(&mut game, &RecordingDecisionProvider::picking(0));
    let humans = tokens(&game);
    assert_eq!(life(&game, 1), 18);

    walk_to_postcombat_main(&mut game);
    relentless_assault_resolves(&mut game, 0);
    assert!(!game.battlefield[&hellrider].tapped && !game.battlefield[&garrison].tapped);
    assert!(humans.iter().all(|human| game.battlefield[human].tapped), "the tokens never attacked");

    let dp = test_dp();
    while !(game.phase.phase_type == PhaseType::Combat && game.phase.step == Some(StepType::DeclareAttackers)) {
        game.advance_turn(&dp).unwrap();
    }
    let attack = ScriptedDecisionProvider::new();
    attack.expect_pick_n(ChoiceKind::DeclareAttackers, vec![0]);
    game.process_declare_attackers(&attack).unwrap();
    assert_eq!(game.pending_triggers.len(), 1, "Hellrider's, in the second combat");
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 1), 17);
    let hellrider_now = Some(existence(&game, hellrider));
    let attacks = game.history.log.rows_newest_first().filter(|row| row.object == hellrider_now && matches!(row.fact, RowFact::Attacked(_))).count();
    assert!(attacks >= 2, "an `Attacked` row in each combat");
}

/// Relentless Assault's first ruling: resolving outside a main phase, it
/// still untaps, and creates no combat or main phase.
// RULING: Relentless Assault #1 - the phases only if it resolves during a main phase.
#[test]
fn relentless_assault_adds_no_phase_outside_a_main_phase() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    declare(&mut game, &[bears], &[(bears, 1)]);
    let phases = game.turn_plan.phases.len();
    relentless_assault_resolves(&mut game, 0);
    assert!(!game.battlefield[&bears].tapped, "it attacked, so it untaps");
    assert_eq!(game.turn_plan.phases.len(), phases, "no phase created in the combat phase");
}

/// Relentless Assault's third ruling: cast in a main phase of an opponent's
/// turn, it untaps that opponent's creatures that attacked, and the combat it
/// adds is that opponent's turn's.
// RULING: Relentless Assault #3 - on an opponent's turn, that opponent's creatures untap and that opponent attacks again.
#[test]
fn relentless_assault_on_an_opponents_turn_is_that_players_combat() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    declare(&mut game, &[bears], &[(bears, 1)]);
    walk_to_postcombat_main(&mut game);
    let phases = game.turn_plan.phases.len();

    relentless_assault_resolves(&mut game, 1);
    assert!(!game.battlefield[&bears].tapped, "player 0's creature attacked this turn");
    assert_eq!(game.turn_plan.phases.len(), phases + 2);
    assert_eq!(game.active_player, 0, "the additional combat is player 0's turn's");
}

// ---------------------------------------------------------------------------
// 7. What a static's condition reads, kept fresh
// ---------------------------------------------------------------------------

/// A 1/1 creature with "as long as [condition], [effect on itself]".
fn while_it(name: &str, condition: Condition, effect: Primitive) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .color(Color::Green)
        .power_toughness(1, 1)
        .ability(static_ability(Effect::Conditional(condition, Box::new(Effect::Atom(effect, EffectRecipient::Object(Antecedent::ThisObject))))))
        .build()
}

/// +1/+1 while the static's condition holds.
fn plus_one() -> Primitive {
    Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(1), Duration::WhileSourceOnBattlefield)
}

fn power(game: &GameState, id: ObjectId) -> Option<i32> {
    compute_characteristics(game, id).and_then(|frame| frame.power)
}

fn lose_life(game: &mut GameState, player: PlayerId, amount: u64) {
    game.execute_action(GameAction::LoseLife { player, amount, damage_source: None }, &test_ctx()).unwrap();
}

fn at_most_ten_life() -> Condition {
    Condition::Player { players: PlayerSet::You, fact: PlayerFact::LifeAtMost(AmountExpr::Fixed(10)) }
}

/// Serra Ascendant's shape: a static reading its controller's life total
/// answers anew once the total moves, though a life change moves no other
/// walk input.
#[test]
fn a_static_reading_a_life_total_answers_anew_as_it_changes() {
    let mut game = stocked(2);
    let creature = put_on_battlefield(&mut game, while_it("Fixture Low Ebb", at_most_ten_life(), plus_one()), 0);
    assert_eq!(power(&game, creature), Some(1));
    lose_life(&mut game, 0, 15);
    assert_eq!(power(&game, creature), Some(2), "at 5 life the condition holds");
}

/// The gate's half: with no frame reading a life total, a life change moves
/// no epoch, so every frame stays served.
#[test]
fn a_life_change_no_static_reads_keeps_every_frame() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(power(&game, bears), Some(2));
    let epoch = game.layer_epoch();
    lose_life(&mut game, 0, 3);
    assert_eq!(game.layer_epoch(), epoch);
}

/// "As long as this is tapped": a tap answers anew.
#[test]
fn a_static_reading_its_tapped_status_answers_anew_as_it_taps() {
    let mut game = stocked(2);
    let creature = put_on_battlefield(&mut game, while_it("Fixture Bowed Head", Condition::SourceTapped, plus_one()), 0);
    assert_eq!(power(&game, creature), Some(1));
    game.execute_action(GameAction::Tap { object: creature }, &test_ctx()).unwrap();
    assert_eq!(power(&game, creature), Some(2));
}

/// "As long as defending player controls a permanent": its attack is the
/// write, and a creature not attacking has no defending player.
#[test]
fn a_static_reading_its_defending_player_answers_anew_as_it_attacks() {
    let mut game = stocked(2);
    let condition = Condition::Player {
        players: PlayerSet::Player(PlayerRef::DefendingPlayer(Antecedent::ThisObject)),
        fact: PlayerFact::ControlsPermanent(ObjectFilter::All),
    };
    let creature = put_on_battlefield(&mut game, while_it("Fixture Raider", condition, plus_one()), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    assert_eq!(power(&game, creature), Some(1));
    declare(&mut game, &[creature], &[(creature, 1)]);
    assert_eq!(power(&game, creature), Some(2));
}

/// "Creatures that attacked this turn get +1/+0": the declaration's history
/// row is the write.
#[test]
fn a_static_naming_what_attacked_this_turn_answers_anew_as_it_attacks() {
    let mut game = stocked(2);
    let attacked = ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ThisTurn(ObjectFact::Attacked)));
    let banner = CardDataBuilder::new("Fixture War Banner")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(0), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(attacked),
        )))
        .build();
    put_on_battlefield(&mut game, banner, 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(power(&game, bears), Some(2));
    declare(&mut game, &[bears], &[(bears, 1)]);
    assert_eq!(power(&game, bears), Some(3));
}

/// The settlement sees it too: an attacker that is no creature while its
/// controller has 10 or less life leaves combat as the life is lost (CR
/// 506.4), since the loss moves the epoch the settlement is gated on.
#[test]
fn a_life_loss_that_ends_an_attackers_creature_type_removes_it_from_combat() {
    let mut game = stocked(2);
    let not_a_creature = Primitive::ChangeType(
        TypeChange { remove_types: vec![CardType::Creature], ..TypeChange::NONE },
        Duration::WhileSourceOnBattlefield,
    );
    let creature = put_on_battlefield(&mut game, while_it("Fixture Fading Spirit", at_most_ten_life(), not_a_creature), 0);
    declare(&mut game, &[creature], &[(creature, 1)]);
    assert_eq!(attack_of(&game, creature), Some(AttackTarget::Player(1)));

    lose_life(&mut game, 0, 15);
    assert_eq!(attack_of(&game, creature), None, "no creature at 5 life, so out of combat");
    let removed = game.recorded_events().records().iter().any(|record| {
        matches!(record.event, GameEvent::RemovedFromCombat { .. })
            && record.stamp.cause == EventCause::GameRule(GameRule::CombatantTypeChanged)
    });
    assert!(removed);
}
