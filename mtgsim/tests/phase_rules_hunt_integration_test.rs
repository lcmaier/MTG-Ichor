//! The small defects the `rules-hunter` hunt found (`codebase-state.md`,
//! "Found by the `rules-hunter` hunt"), each test a board that gave a wrong
//! answer before its fix.

use std::sync::Arc;

use mtgsim::cards::printings::*;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::objects::card_data::CardData;
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::{get_effective_controller, has_summoning_sickness};
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::scenario::Scenario;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::*;
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;

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

/// CR 802.3 gives a creature one target, so a declaration naming it twice is
/// illegal and declared again, and Hellrider triggers once.
#[test]
fn a_creature_declared_at_two_players_is_asked_again() {
    let mut game = setup_game(3);
    let rider = put_on_battlefield(&mut game, hellrider(), 0);
    in_combat_step(&mut game, StepType::DeclareAttackers);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::DeclareAttackers, vec![attacks(rider, 1), attacks(rider, 2)]);
    dp.expect_choice(ChoiceKind::DeclareAttackers, vec![attacks(rider, 1)]);
    let seat = RejectionRecorder::new(dp);
    game.process_declare_attackers(&seat).unwrap();

    let asked = seat.rejected_at("DeclareAttackers");
    assert_eq!(asked.len(), 2, "the illegal declaration is declared again");
    assert!(asked[0].is_none() && asked[1].is_some(), "the second ask names what it rejected");
    assert_eq!(attacking(&game, rider), Some(AttackTarget::Player(1)));
    assert_eq!(game.pending_triggers.len(), 1, "one attacker, one trigger");
}

// COVERS: ATOM-802.4-001
#[test]
fn defending_players_declare_blocks_in_apnap_order_whatever_the_attackers_timestamps() {
    let mut game = setup_game(4);
    let at_p3 = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    let at_p1 = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    place_vanilla_creature(&mut game, 1, 2, 2, &[]);
    place_vanilla_creature(&mut game, 3, 2, 2, &[]);
    set_attacking(&mut game, at_p3, 3);
    set_attacking(&mut game, at_p1, 1);
    in_combat_step(&mut game, StepType::DeclareBlockers);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::DeclareBlockers, vec![]);
    dp.expect_choice(ChoiceKind::DeclareBlockers, vec![]);
    game.process_declare_blockers(&dp).unwrap();

    let order: Vec<PlayerId> = dp.asked_of().into_iter().map(|(player, _)| player).collect();
    assert_eq!(order, vec![1, 3], "the older attacker's defender declares second");
}

/// Act of Treason's effect, resolved for `caster` on `target`.
fn act_of_treason_on(game: &mut GameState, caster: PlayerId, target: ObjectId) {
    let card = act_of_treason();
    let spell = game.add_object(GameObject::new(card.clone(), caster, Zone::Graveyard));
    let context = ResolutionContext {
        source: spell,
        ability_source: None,
        controller: caster,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Object(target)]),
        replaced_event: None,
        trigger: None,
    };
    let effect = card.abilities[0].effect.clone();
    game.resolve_effect(&effect, &context, &ScriptedDecisionProvider::new()).unwrap();
}

/// P1's Leyline of the Void, a creature under Opalescence, in P0's hands until
/// end of turn or left with P1. P0 runs a turn holding nine cards, and the
/// cleanup step's discard answers how many reached P0's graveyard and how many
/// were exiled.
fn discarded_beside_a_leyline(stolen: bool) -> (usize, usize) {
    let mut game = Game::new(GameConfig::test(), vec![bears_deck(), bears_deck()]).unwrap();
    let dp = ScriptedDecisionProvider::new();
    game.setup(&dp).unwrap();
    put_on_battlefield(&mut game.state, opalescence(), 1);
    let leyline = put_on_battlefield(&mut game.state, leyline_of_the_void(), 1);
    while game.state.players[0].hand.len() < 9 {
        put_in_hand(&mut game.state, grizzly_bears(), 0);
    }
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
fn a_trigger_born_during_placement_waits_for_the_next_pass() {
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

/// P0 steals P2's creature with Act of Treason. It goes back at P0's cleanup,
/// so through P1's turn P2 has not controlled it continuously since P2's most
/// recent turn began (CR 302.6), and only P2's next turn frees it.
#[test]
fn a_steal_that_ended_leaves_the_creature_summoning_sick_for_its_owner() {
    let mut game = setup_game(3);
    (0..3).for_each(|player| fill_library(&mut game, player, 10));
    let creature = place_vanilla_creature(&mut game, 2, 2, 2, &[]);
    assert!(!has_summoning_sickness(&game, creature));
    act_of_treason_on(&mut game, 0, creature);

    let dp = test_dp();
    while game.active_player != 1 {
        game.advance_turn(&dp).unwrap();
    }
    assert_eq!(get_effective_controller(&game, creature), Some(2));
    assert!(has_summoning_sickness(&game, creature), "P2's control was interrupted");

    while game.active_player != 2 {
        game.advance_turn(&dp).unwrap();
    }
    assert!(!has_summoning_sickness(&game, creature), "P2's turn began with it under control");
}

/// A creature nobody took stays free across the same turns.
#[test]
fn a_creature_nobody_stole_is_not_summoning_sick_on_another_players_turn() {
    let mut game = setup_game(3);
    (0..3).for_each(|player| fill_library(&mut game, player, 10));
    let creature = place_vanilla_creature(&mut game, 2, 2, 2, &[]);
    let dp = test_dp();
    while game.active_player != 1 {
        game.advance_turn(&dp).unwrap();
    }
    assert!(!has_summoning_sickness(&game, creature));
}

/// A board that says when control last moved reads that way, and writes it back.
#[test]
fn a_board_states_when_control_last_moved() {
    let text = "players 2\nturn 4\nactive 0\nbattlefield: Grizzly Bears | controller 1, control since turn 3";
    let scenario = Scenario::parse(text).unwrap();
    let game = scenario.build(&CardRegistry::default_registry()).unwrap().game;
    let bears = game.state.battlefield_ids_ordered()[0];
    assert_eq!(game.state.battlefield.get(&bears).unwrap().control_began_turn, 3);
    let written = Scenario::write(&game.state).to_string();
    assert!(written.contains("control since turn 3"), "{written}");
}

/// Two attackers hit P1 beside Ghosts of the Innocent and Furnace of Rath, and
/// P1's Reflect Damage sends one back at P0. P1 chooses it first, and the two
/// events then affect two players, so CR 616.1 has each choose for their own,
/// the active player first.
#[test]
fn a_redirect_leaves_each_affected_player_choosing_for_their_own_event() {
    let mut game = setup_two_player_game();
    let sent_back = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
    let kept = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
    set_attacking(&mut game, sent_back, 1);
    set_attacking(&mut game, kept, 1);
    let ghosts = put_on_battlefield(&mut game, ghosts_of_the_innocent(), 1);
    let furnace = put_on_battlefield(&mut game, furnace_of_rath(), 1);
    let reflect = put_in_hand(&mut game, reflect_damage(), 1);
    let card = reflect_damage();
    let context = ResolutionContext {
        source: reflect,
        ability_source: None,
        controller: 1,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![]),
        replaced_event: None,
        trigger: None,
    };
    let choosing = RecordingDecisionProvider::choosing([ChoiceOption::Object(sent_back)]);
    game.resolve_effect(&card.abilities[0].effect, &context, &choosing).unwrap();

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
