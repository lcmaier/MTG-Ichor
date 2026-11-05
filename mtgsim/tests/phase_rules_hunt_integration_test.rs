//! The small defects the `rules-hunter` hunt found (`codebase-state.md`,
//! "Found by the `rules-hunter` hunt"), each test a board that gave a wrong
//! answer before its fix.

use mtgsim::cards::printings::*;
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::*;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;

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
