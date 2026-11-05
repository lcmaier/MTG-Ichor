//! The small defects the `rules-hunter` hunt found (`codebase-state.md`,
//! "Found by the `rules-hunter` hunt"), each test a board that gave a wrong
//! answer before its fix.

use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::*;
use mtgsim::types::ids::PlayerId;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

fn in_combat_step(game: &mut GameState, step: StepType) {
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(step) });
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
