//! The small defects the `rules-hunter` hunt found (`codebase-state.md`,
//! "Found by the `rules-hunter` hunt"), each test a board that gave a wrong
//! answer before its fix.

use std::sync::Arc;

use mtgsim::cards::authoring::{
    create_replacement_from_a_chosen_source, deal_damage, each_opponent, spell_ability,
    the_next_damage_from_a_chosen_source,
};
use mtgsim::cards::printings::*;
use mtgsim::cards::registry::CardRegistry;

use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::{get_effective_controller, has_summoning_sickness};
use mtgsim::oracle::mana_helpers::activatable_abilities;
use mtgsim::scenario::Scenario;
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::*;
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::replacement::{AmountRewrite, Rewrite};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

fn bears_deck() -> Vec<Arc<CardData>> {
    (0..30).map(|_| grizzly_bears()).collect()
}

fn in_combat_step(game: &mut GameState, step: StepType) {
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(step) });
}

fn attacks(creature: ObjectId, player: PlayerId) -> ChoiceOption {
    ChoiceOption::AttackerTarget(creature, AttackTarget::Player(player))
}

fn attacking(game: &GameState, creature: ObjectId) -> Option<AttackTarget> {
    game.battlefield.get(&creature)?.attacking.as_ref().map(|info| info.target)
}

/// `effect`, resolved for `controller` from a spell of `card`'s with `targets`.
fn resolve(
    game: &mut GameState,
    card: Arc<CardData>,
    effect: &Effect,
    controller: PlayerId,
    targets: Vec<ObjectOrPlayerId>,
    dp: &dyn DecisionProvider,
) -> ObjectId {
    let spell = game.add_object(GameObject::new(card, controller, Zone::Graveyard));
    let context = ResolutionContext {
        source: spell,
        ability_source: None,
        controller,
        generated_by: GeneratedBy::Spell,
        targets: if targets.is_empty() { ChosenTargets::NONE } else { ChosenTargets::one(targets) },
        replaced_event: None,
        trigger: None,
    };
    game.resolve_effect(effect, &context, dp).unwrap();
    spell
}

/// `card`'s spell ability, resolved for `controller`.
fn resolve_card(
    game: &mut GameState,
    card: Arc<CardData>,
    controller: PlayerId,
    targets: Vec<ObjectOrPlayerId>,
    dp: &dyn DecisionProvider,
) -> ObjectId {
    let effect = card.abilities[0].effect.clone();
    resolve(game, card, &effect, controller, targets, dp)
}

fn act_of_treason_on(game: &mut GameState, caster: PlayerId, target: ObjectId) {
    resolve_card(game, act_of_treason(), caster, vec![ObjectOrPlayerId::Object(target)], &test_dp());
}

/// "Gain control of target permanent", for as long as nothing ends it.
fn steal_for_good(game: &mut GameState, thief: PlayerId, target: ObjectId) {
    let effect = Effect::Atom(
        Primitive::GainControl { to: PlayerRef::You, duration: Duration::Indefinite },
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    );
    resolve(game, grizzly_bears(), &effect, thief, vec![ObjectOrPlayerId::Object(target)], &test_dp());
}

fn three_seats() -> GameState {
    let mut game = setup_game(3);
    (0..3).for_each(|player| fill_library(&mut game, player, 20));
    game
}

fn play_to_turn(game: &mut GameState, turn: u32) {
    let dp = test_dp();
    while game.turn_number < turn {
        game.advance_turn(&dp).unwrap();
    }
}

fn offered(game: &GameState, player: PlayerId, permanent: ObjectId) -> bool {
    activatable_abilities(game, player).iter().any(|&(source, ..)| source == permanent)
}

// COVERS: ATOM-802.3-001
#[test]
fn each_attacker_chooses_its_own_defending_player() {
    let mut game = setup_game(4);
    let x = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    let y = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    in_combat_step(&mut game, StepType::DeclareAttackers);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::DeclareAttackers, vec![attacks(x, 1), attacks(y, 2)]);
    game.process_declare_attackers(&dp).unwrap();

    assert_eq!(attacking(&game, x), Some(AttackTarget::Player(1)));
    assert_eq!(attacking(&game, y), Some(AttackTarget::Player(2)));
}

/// CR 802.3 gives a creature one target, so the question takes one pair per
/// creature, and an answer naming Hellrider twice is malformed, as a repeated
/// index is. There is no second declaration for it to trigger on.
#[test]
#[should_panic(expected = "declared attacking twice")]
fn a_declaration_naming_a_creature_twice_is_malformed() {
    let mut game = setup_game(3);
    let rider = put_on_battlefield(&mut game, hellrider(), 0);
    place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    in_combat_step(&mut game, StepType::DeclareAttackers);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::DeclareAttackers, vec![attacks(rider, 1), attacks(rider, 2)]);
    let _ = game.process_declare_attackers(&dp);
}

/// The most a declaration can name is one pair for each creature.
#[test]
fn a_declaration_is_bounded_by_its_creatures_and_not_its_pairs() {
    let mut game = setup_game(3);
    let rider = put_on_battlefield(&mut game, hellrider(), 0);
    in_combat_step(&mut game, StepType::DeclareAttackers);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::DeclareAttackers, vec![attacks(rider, 2)]);
    game.process_declare_attackers(&dp).unwrap();

    assert_eq!(attacking(&game, rider), Some(AttackTarget::Player(2)));
    assert_eq!(game.pending_triggers.len(), 1, "one attacker, one trigger");
}

// COVERS: ATOM-802.4-001
#[test]
fn defending_players_declare_blocks_in_apnap_order_whatever_the_attackers_timestamps() {
    let mut game = setup_game(4);
    let at_p3 = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    let at_p1 = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    let p1_blocker = place_vanilla_creature(&mut game, 1, 2, 2, &[]);
    let p3_blocker = place_vanilla_creature(&mut game, 3, 2, 2, &[]);
    set_attacking(&mut game, at_p3, 3);
    set_attacking(&mut game, at_p1, 1);
    in_combat_step(&mut game, StepType::DeclareBlockers);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::DeclareBlockers, vec![ChoiceOption::BlockerAttacker(p1_blocker, at_p1)]);
    dp.expect_choice(ChoiceKind::DeclareBlockers, vec![ChoiceOption::BlockerAttacker(p3_blocker, at_p3)]);
    game.process_declare_blockers(&dp).unwrap();

    let order: Vec<PlayerId> = dp.asked_of().into_iter().map(|(player, _)| player).collect();
    assert_eq!(order, vec![1, 3], "the older attacker's defender declares second");
    let blocked_by = |attacker| game.battlefield.get(&attacker).unwrap().attacking.as_ref().unwrap().blocked_by.clone();
    assert_eq!(blocked_by(at_p1), vec![p1_blocker]);
    assert_eq!(blocked_by(at_p3), vec![p3_blocker]);
}

/// P1's Leyline of the Void, a creature under Opalescence, in P0's hands until
/// end of turn or left with P1. P0 reaches the main phase holding ten cards, so
/// the cleanup step discards three; the answer is how many reached P0's
/// graveyard and how many were exiled.
fn discarded_beside_a_leyline(stolen: bool) -> (usize, usize) {
    let mut game = Game::new(GameConfig::test(), vec![bears_deck(), bears_deck()]).unwrap();
    let dp = ScriptedDecisionProvider::new();
    game.setup(&dp).unwrap();
    put_on_battlefield(&mut game.state, opalescence(), 1);
    let leyline = put_on_battlefield(&mut game.state, leyline_of_the_void(), 1);
    while game.state.players[0].hand.len() < 10 {
        put_in_hand(&mut game.state, grizzly_bears(), 0);
    }
    game.state.set_turn_position(Phase { phase_type: PhaseType::Precombat, step: None });
    if stolen {
        act_of_treason_on(&mut game.state, 0, leyline);
        dp.expect_choice(ChoiceKind::DeclareAttackers, vec![]);
    }
    let hand = game.state.players[0].hand.clone();
    let discards = hand[..3].iter().map(|&card| ChoiceOption::Object(card)).collect();
    dp.expect_choice(ChoiceKind::Discard { source: None }, discards);
    game.run_turn(&dp).unwrap();
    (game.state.players[0].graveyard.len(), game.state.exile.len())
}

/// CR 514.1 is "first" and CR 514.2 "second", so a steal that ends at 514.2
/// still holds for the discard: P0 controls the Leyline, and P0's graveyard is
/// not an opponent's.
// COVERS-PARTIAL: ATOM-514.1-001, ATOM-514.2-001
#[test]
fn the_cleanup_discard_comes_before_until_end_of_turn_effects_end() {
    assert_eq!(discarded_beside_a_leyline(true), (3, 0));
}

#[test]
fn an_opponents_leyline_still_exiles_the_cleanup_discard() {
    assert_eq!(discarded_beside_a_leyline(false), (0, 3));
}

/// P1's Soul Warden triggers on Flickerwisp entering, and Flickerwisp's trigger,
/// placed first, targets P1's Cephalid Aristocrat, which triggers as it is
/// targeted. That trigger was not waiting when the placement began, so P1
/// places the Warden's alone and it goes on top after the state-based check.
#[test]
fn a_trigger_created_during_placement_waits_for_the_next_pass() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 10);
    fill_library(&mut game, 1, 10);
    let warden = put_on_battlefield(&mut game, soul_warden(), 1);
    let aristocrat = put_on_battlefield(&mut game, cephalid_aristocrat(), 1);
    let answers = test_dp();
    game.perform_sba_and_triggers(&answers).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&answers).unwrap();
    }
    let wisp = put_on_battlefield(&mut game, flickerwisp(), 0);

    let dp = RecordingDecisionProvider::choosing([ChoiceOption::Object(aristocrat)]);
    game.perform_sba_and_triggers(&dp).unwrap();

    let sources: Vec<Option<ObjectId>> =
        game.stack.iter().map(|id| game.stack_entries[id].ability_identity.map(|ability| ability.source.id)).collect();
    assert_eq!(sources, vec![Some(wisp), Some(warden), Some(aristocrat)], "bottom of the stack first");
    assert_eq!(dp.kinds(), vec!["SelectRecipients"], "P1 has nothing to order");
}

/// P0 steals P2's Samite Healer on turn 4, after P2's first turn. It goes back
/// at P0's cleanup, so through P1's turn P2 has not controlled it continuously
/// since P2's most recent turn began (CR 302.6), and {T} is not offered. Only
/// P2's next turn frees it.
#[test]
fn a_steal_that_ended_leaves_the_creature_summoning_sick_for_its_owner() {
    let mut game = three_seats();
    let healer = put_on_battlefield(&mut game, samite_healer(), 2);
    play_to_turn(&mut game, 4);
    assert!(offered(&game, 2, healer), "the owner's own creature, before the steal");
    act_of_treason_on(&mut game, 0, healer);

    play_to_turn(&mut game, 5);
    assert_eq!(get_effective_controller(&game, healer), Some(2));
    assert!(has_summoning_sickness(&game, healer), "P2's control was interrupted");
    assert!(!offered(&game, 2, healer));

    play_to_turn(&mut game, 6);
    assert!(!has_summoning_sickness(&game, healer), "P2's turn began with it under control");
    assert!(offered(&game, 2, healer));
}

/// At two seats an extra turn puts a turn between the steal's end and the
/// owner's next.
#[test]
fn a_steal_that_ended_before_an_extra_turn_leaves_the_creature_sick() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 20);
    fill_library(&mut game, 1, 20);
    let healer = put_on_battlefield(&mut game, samite_healer(), 1);
    play_to_turn(&mut game, 3);
    act_of_treason_on(&mut game, 0, healer);
    resolve_card(&mut game, time_walk(), 0, vec![], &test_dp());

    play_to_turn(&mut game, 4);
    assert_eq!(game.active_player, 0, "the extra turn");
    assert_eq!(get_effective_controller(&game, healer), Some(1));
    assert!(!offered(&game, 1, healer), "P1's control was interrupted");
}

/// P0 has held P2's creature since turn 4. P1 takes it for turn 8, and it goes
/// back to P0, whose older row must not restore the older clock: P0's control
/// was interrupted since P0's turn 7 began.
#[test]
fn a_steal_interrupted_by_another_restarts_the_clock_when_it_resumes() {
    let mut game = three_seats();
    let creature = place_vanilla_creature(&mut game, 2, 2, 2, &[]);
    play_to_turn(&mut game, 4);
    steal_for_good(&mut game, 0, creature);
    play_to_turn(&mut game, 8);
    act_of_treason_on(&mut game, 1, creature);

    play_to_turn(&mut game, 9);
    assert_eq!(get_effective_controller(&game, creature), Some(0));
    assert!(has_summoning_sickness(&game, creature));

    play_to_turn(&mut game, 10);
    assert!(!has_summoning_sickness(&game, creature));
}

/// A creature nobody took stays free across the same turns.
#[test]
fn a_creature_nobody_stole_is_not_summoning_sick_on_another_players_turn() {
    let mut game = three_seats();
    let healer = put_on_battlefield(&mut game, samite_healer(), 2);
    play_to_turn(&mut game, 5);
    assert!(offered(&game, 2, healer));
}

fn board(text: &str) -> Game {
    Scenario::parse(text).unwrap().build(&CardRegistry::default_registry()).unwrap().game
}

#[test]
fn a_board_keeps_when_control_last_moved_through_a_write_and_a_load() {
    let text = "players 2\nturn 4\nactive 0\nbattlefield: Grizzly Bears | controller 1, control since turn 3";
    let game = board(text);
    assert!(has_summoning_sickness(&game.state, game.state.battlefield_ids_ordered()[0]));

    let written = Scenario::write(&game.state).to_string();
    assert!(written.contains("control since turn 3"), "{written}");
    let loaded = board(&written);
    assert!(has_summoning_sickness(&loaded.state, loaded.state.battlefield_ids_ordered()[0]));
}

/// The clock reads only against the controller's most recent turn, so one from
/// before it is the default's answer.
#[test]
fn a_clock_from_before_the_controllers_turn_is_not_written() {
    let game = board("players 2\nturn 4\nactive 1\nbattlefield: Grizzly Bears | controller 1, control since turn 3");
    assert!(!has_summoning_sickness(&game.state, game.state.battlefield_ids_ordered()[0]));
    assert!(!Scenario::write(&game.state).to_string().contains("control since"));
}

#[test]
fn a_board_refuses_a_clock_older_than_the_permanent() {
    let text = "players 2\nturn 4\nactive 1\nbattlefield: Grizzly Bears | controller 1, arrived this turn, control since turn 2";
    let refusal = Scenario::parse(text).unwrap().build(&CardRegistry::default_registry()).map(|_| ());
    assert!(refusal.is_err(), "turn 2 is before it arrived");
}

/// Two 3/3s attacking P1, the first named for P1's Reflect Damage.
fn two_attackers_one_reflected(game: &mut GameState) -> (ObjectId, ObjectId, ObjectId) {
    let sent_back = place_vanilla_creature(game, 0, 3, 3, &[]);
    let kept = place_vanilla_creature(game, 0, 3, 3, &[]);
    set_attacking(game, sent_back, 1);
    set_attacking(game, kept, 1);
    let choosing = RecordingDecisionProvider::choosing([ChoiceOption::Object(sent_back)]);
    let reflect = resolve_card(game, reflect_damage(), 1, vec![], &choosing);
    (sent_back, kept, reflect)
}

/// Two attackers hit P1 beside Ghosts of the Innocent and Furnace of Rath, and
/// P1's Reflect Damage sends one back at P0. P1 chooses it first, and the two
/// events then affect two players, so CR 616.1 has each choose for their own,
/// the active player first.
#[test]
fn a_redirect_leaves_each_affected_player_choosing_for_their_own_event() {
    let mut game = setup_two_player_game();
    let ghosts = put_on_battlefield(&mut game, ghosts_of_the_innocent(), 1);
    let furnace = put_on_battlefield(&mut game, furnace_of_rath(), 1);
    let (.., reflect) = two_attackers_one_reflected(&mut game);

    let dp = ScriptedDecisionProvider::new();
    let chooses = ChoiceKind::ChooseReplacementEffect { affected_object: None };
    dp.expect_choice(chooses.clone(), vec![ChoiceOption::Object(reflect)]);
    dp.expect_choice(chooses.clone(), vec![ChoiceOption::Object(ghosts)]);
    dp.expect_choice(chooses.clone(), vec![ChoiceOption::Object(furnace)]);
    game.process_combat_damage(&dp, false).unwrap();

    let asked: Vec<PlayerId> = dp.asked_of().into_iter().map(|(player, _)| player).collect();
    assert_eq!(asked, vec![1, 0, 1], "P1, then each for their own, the active player first");
    assert_eq!(game.players[0].life_total, 20 - 2, "P0 halves then doubles its 3");
    assert_eq!(game.players[1].life_total, 20 - 3, "P1 doubles then halves its 3");
}

/// Eye for an Eye's words and shape, with "each opponent" for "that source's
/// controller", which no rider can name yet: its own damage waits as a rider
/// beside an event that still happens.
fn retaliating_eye() -> Arc<CardData> {
    let text = "The next time a source of your choice would deal damage to you this turn, instead that \
                source deals that much damage to you and this spell deals that much damage to each \
                opponent.";
    CardDataBuilder::new("Retaliating Eye")
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(
            text,
            create_replacement_from_a_chosen_source(
                the_next_damage_from_a_chosen_source(None, Rewrite::Amount(AmountRewrite::Multiplier(1)))
                    .with_then(deal_damage(each_opponent(), AmountExpr::ReplacedAmount)),
            )
            .until_end_of_turn(),
        ))
        .build()
}

/// A 3/3 attacks P1 beside Ghosts of the Innocent, and P1's Retaliating Eye names
/// it. The eye turns the event into two pieces of damage. Applied first, it
/// leaves Ghosts to reach both; applied second, its "that much" is already
/// halved, and Ghosts has had its one opportunity (CR 614.5). Either way each
/// player takes 1.
#[test]
fn a_rider_gives_the_same_damage_whichever_effect_applies_first() {
    for eye_first in [true, false] {
        let mut game = setup_two_player_game();
        let ghosts = put_on_battlefield(&mut game, ghosts_of_the_innocent(), 1);
        let attacker = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
        set_attacking(&mut game, attacker, 1);
        let choosing = RecordingDecisionProvider::choosing([ChoiceOption::Object(attacker)]);
        let eye = resolve_card(&mut game, retaliating_eye(), 1, vec![], &choosing);

        let dp = ScriptedDecisionProvider::new();
        let first = if eye_first { eye } else { ghosts };
        dp.expect_choice(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![ChoiceOption::Object(first)]);
        game.process_combat_damage(&dp, false).unwrap();

        assert_eq!(game.players[0].life_total, 20 - 1, "the rider's half, eye first: {eye_first}");
        assert_eq!(game.players[1].life_total, 20 - 1, "the attacker's half, eye first: {eye_first}");
    }
}

/// The same across a split: the eye's rider waits, Reflect Damage sends the other
/// attacker back, and Ghosts of the Innocent applies in each player's group,
/// after the rider was queued, so it reaches the rider's 3 as well.
#[test]
fn a_rider_queued_before_a_redirect_meets_what_applied_after_it() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, ghosts_of_the_innocent(), 1);
    let (_, kept, reflect) = two_attackers_one_reflected(&mut game);
    let choosing = RecordingDecisionProvider::choosing([ChoiceOption::Object(kept)]);
    let eye = resolve_card(&mut game, retaliating_eye(), 1, vec![], &choosing);

    let dp = ScriptedDecisionProvider::new();
    let chooses = ChoiceKind::ChooseReplacementEffect { affected_object: None };
    dp.expect_choice(chooses.clone(), vec![ChoiceOption::Object(eye)]);
    dp.expect_choice(chooses.clone(), vec![ChoiceOption::Object(reflect)]);
    game.process_combat_damage(&dp, false).unwrap();

    assert_eq!(game.players[0].life_total, 20 - 1 - 1, "the reflected 3 halved, and the rider's 3 halved");
    assert_eq!(game.players[1].life_total, 20 - 1, "the kept 3 halved");
}

/// Ghosts of the Innocent's ruling: "If damage is redirected, it's only halved
/// once." P1 applies Ghosts first, so it has had its opportunity when P0's group
/// forms, and the reflected 3 stays 1.
// RULING: Ghosts of the Innocent #5
#[test]
fn damage_redirected_after_it_was_halved_is_not_halved_again() {
    let mut game = setup_two_player_game();
    let ghosts = put_on_battlefield(&mut game, ghosts_of_the_innocent(), 1);
    two_attackers_one_reflected(&mut game);

    let dp = ScriptedDecisionProvider::new();
    let chooses = ChoiceKind::ChooseReplacementEffect { affected_object: None };
    dp.expect_choice(chooses.clone(), vec![ChoiceOption::Object(ghosts)]);
    game.process_combat_damage(&dp, false).unwrap();

    assert_eq!(game.players[0].life_total, 20 - 1);
    assert_eq!(game.players[1].life_total, 20 - 1);
}
