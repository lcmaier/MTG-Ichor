//! Phase LB Integration Tests: Continuous Effects via Layer System
//!
//! Tests the Giant Growth pump spell end-to-end:
//! cast → resolve (registers continuous effect) → layer system computes P/T →
//! cleanup step expires UntilEndOfTurn effects.

mod common;

use mtgsim::cards::alpha;
use mtgsim::cards::creatures;
use mtgsim::cards::phase5_pre_cards;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::oracle::characteristics::{get_effective_power, get_effective_toughness};
use mtgsim::types::effects::{EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

use common::{put_in_hand, put_on_battlefield, setup_two_player_game};

// ---------------------------------------------------------------------------
// Test 1: Giant Growth gives +3/+3 until end of turn
// ---------------------------------------------------------------------------

#[test]
fn test_giant_growth_pumps_creature() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let growth_id = put_in_hand(&mut game, alpha::giant_growth(), 0);
    game.players[0].mana_pool.add(ManaType::Green, 1);

    // Verify base P/T
    assert_eq!(get_effective_power(&game, bears_id), Some(2));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(2));

    let decisions = ScriptedDecisionProvider::new();

    // Cast Giant Growth: pick_n for priority action (index 1 = CastSpell)
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // Target: the creature (index 0 in legal targets list)
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id: growth_id,
        },
        vec![0],
    );
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);
    assert!(game.stack.contains(&growth_id));

    // Both players pass → resolve Giant Growth
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::StackResolved);

    // Giant Growth is now in graveyard
    assert_eq!(game.get_object(growth_id).unwrap().zone, Zone::Graveyard);

    // Creature should be 5/5
    assert_eq!(get_effective_power(&game, bears_id), Some(5));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(5));

    // Verify a continuous effect was registered
    assert_eq!(game.continuous_effects.len(), 1);
}

// ---------------------------------------------------------------------------
// Test 2: Giant Growth expires at cleanup step
// ---------------------------------------------------------------------------

#[test]
fn test_giant_growth_expires_at_cleanup() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let growth_id = put_in_hand(&mut game, alpha::giant_growth(), 0);
    game.players[0].mana_pool.add(ManaType::Green, 1);

    let decisions = ScriptedDecisionProvider::new();

    // Cast and resolve Giant Growth (same as test 1)
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id: growth_id,
        },
        vec![0],
    );
    game.run_priority_round(&decisions).unwrap();

    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(&decisions).unwrap();

    // Creature is 5/5
    assert_eq!(get_effective_power(&game, bears_id), Some(5));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(5));

    // Advance through turn to cleanup step.
    // We're in Precombat main. Steps to cleanup:
    //   Precombat → Combat (BeginCombat, DeclareAttackers, DeclareBlockers,
    //               FirstStrikeDamage, CombatDamage, EndCombat) → Postcombat → Ending (End, Cleanup)
    // Precombat → Combat/BeginCombat = 1
    // BeginCombat → DeclareAttackers = 1
    // DeclareAttackers → DeclareBlockers = 1
    // DeclareBlockers → FirstStrikeDamage = 1
    // FirstStrikeDamage → CombatDamage = 1
    // CombatDamage → EndCombat = 1
    // EndCombat → (end of Combat) → Postcombat = 1
    // Postcombat → Ending/End = 1
    // End → Cleanup = 1
    // Total: 9 advances from Precombat to Cleanup
    for _ in 0..9 {
        game.advance_turn().unwrap();
    }

    // Verify we're in cleanup
    assert_eq!(game.phase.phase_type, mtgsim::state::game_state::PhaseType::Ending);
    assert_eq!(game.phase.step, Some(mtgsim::state::game_state::StepType::Cleanup));

    // Effect should be removed
    assert_eq!(game.continuous_effects.len(), 0);

    // Creature should be back to 2/2
    assert_eq!(get_effective_power(&game, bears_id), Some(2));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(2));
}

// ---------------------------------------------------------------------------
// Test 3: Two Giant Growths stack additively
// ---------------------------------------------------------------------------

#[test]
fn test_two_giant_growths_stack() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let growth1_id = put_in_hand(&mut game, alpha::giant_growth(), 0);
    let growth2_id = put_in_hand(&mut game, alpha::giant_growth(), 0);
    game.players[0].mana_pool.add(ManaType::Green, 2);

    let decisions = ScriptedDecisionProvider::new();

    // Cast and resolve first Giant Growth
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id: growth1_id,
        },
        vec![0],
    );
    game.run_priority_round(&decisions).unwrap();
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(&decisions).unwrap();

    assert_eq!(get_effective_power(&game, bears_id), Some(5));

    // Cast and resolve second Giant Growth
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id: growth2_id,
        },
        vec![0],
    );
    game.run_priority_round(&decisions).unwrap();
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(&decisions).unwrap();

    // Should be 2 + 3 + 3 = 8/8
    assert_eq!(get_effective_power(&game, bears_id), Some(8));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(8));
    assert_eq!(game.continuous_effects.len(), 2);

    // Advance to cleanup — both expire
    for _ in 0..9 {
        game.advance_turn().unwrap();
    }

    assert_eq!(game.continuous_effects.len(), 0);
    assert_eq!(get_effective_power(&game, bears_id), Some(2));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(2));
}

// ===========================================================================
// Anthem Tests: Glorious Anthem static ability
// ===========================================================================

// ---------------------------------------------------------------------------
// Test 4: Glorious Anthem gives +1/+1 to creatures you control on ETB
// ---------------------------------------------------------------------------

#[test]
fn test_glorious_anthem_buffs_creatures_on_etb() {
    let mut game = setup_two_player_game();

    // Place a creature first
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    assert_eq!(get_effective_power(&game, bears_id), Some(2));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(2));

    // Place Glorious Anthem — static effect registers on ETB
    let _anthem_id = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);

    // Creature should now be 3/3
    assert_eq!(get_effective_power(&game, bears_id), Some(3));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(3));

    // Verify a continuous effect was registered
    assert_eq!(game.continuous_effects.len(), 1);
}

// ---------------------------------------------------------------------------
// Test 5: Anthem applies to creatures that enter after the anthem
// ---------------------------------------------------------------------------

#[test]
fn test_glorious_anthem_buffs_creatures_entering_later() {
    let mut game = setup_two_player_game();

    // Place Glorious Anthem first
    let _anthem_id = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);
    assert_eq!(game.continuous_effects.len(), 1);

    // Now place a creature — should immediately get +1/+1
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    assert_eq!(get_effective_power(&game, bears_id), Some(3));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(3));
}

// ---------------------------------------------------------------------------
// Test 6: Anthem only affects creatures you control (not opponent's)
// ---------------------------------------------------------------------------

#[test]
fn test_glorious_anthem_only_your_creatures() {
    let mut game = setup_two_player_game();

    // Player 0's creature
    let my_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    // Player 1's creature
    let opp_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);

    // Player 0 places Glorious Anthem
    let _anthem_id = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);

    // Only player 0's creature gets the buff
    assert_eq!(get_effective_power(&game, my_bears), Some(3));
    assert_eq!(get_effective_toughness(&game, my_bears), Some(3));
    assert_eq!(get_effective_power(&game, opp_bears), Some(2));
    assert_eq!(get_effective_toughness(&game, opp_bears), Some(2));
}

// ---------------------------------------------------------------------------
// Test 7: Anthem does not buff non-creature permanents
// ---------------------------------------------------------------------------

#[test]
fn test_glorious_anthem_ignores_noncreatures() {
    let mut game = setup_two_player_game();

    // Place a land (not a creature)
    let land_id = common::put_land_on_battlefield(&mut game, mtgsim::cards::basic_lands::forest, 0);

    // Place Glorious Anthem
    let _anthem_id = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);

    // Land has no P/T
    assert_eq!(get_effective_power(&game, land_id), None);
    assert_eq!(get_effective_toughness(&game, land_id), None);
}

// ---------------------------------------------------------------------------
// Test 8: Anthem effect removed when anthem leaves the battlefield
// ---------------------------------------------------------------------------

#[test]
fn test_glorious_anthem_removed_on_ltb() {
    let mut game = setup_two_player_game();

    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let anthem_id = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);

    // Buffed
    assert_eq!(get_effective_power(&game, bears_id), Some(3));
    assert_eq!(game.continuous_effects.len(), 1);

    // Destroy the anthem (move to graveyard)
    game.change_zone(anthem_id, Zone::Graveyard).unwrap();

    // Effect removed, creature back to base
    assert_eq!(game.continuous_effects.len(), 0);
    assert_eq!(get_effective_power(&game, bears_id), Some(2));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(2));
}

// ---------------------------------------------------------------------------
// Test 9: Two anthems stack additively
// ---------------------------------------------------------------------------

#[test]
fn test_two_anthems_stack() {
    let mut game = setup_two_player_game();

    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let _anthem1 = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);
    let _anthem2 = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);

    // 2 + 1 + 1 = 4/4
    assert_eq!(get_effective_power(&game, bears_id), Some(4));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(4));
    assert_eq!(game.continuous_effects.len(), 2);
}

// ---------------------------------------------------------------------------
// Test 10: Anthem + Giant Growth combine correctly
// ---------------------------------------------------------------------------

#[test]
fn test_anthem_plus_pump_spell() {
    let mut game = setup_two_player_game();

    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let _anthem_id = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);
    let growth_id = put_in_hand(&mut game, alpha::giant_growth(), 0);
    game.players[0].mana_pool.add(ManaType::Green, 1);

    // Base 2/2 + anthem +1/+1 = 3/3
    assert_eq!(get_effective_power(&game, bears_id), Some(3));

    let decisions = ScriptedDecisionProvider::new();

    // Cast Giant Growth targeting the creature
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id: growth_id,
        },
        vec![0],
    );
    game.run_priority_round(&decisions).unwrap();

    // Resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(&decisions).unwrap();

    // 2 base + 1 anthem + 3 growth = 6/6
    assert_eq!(get_effective_power(&game, bears_id), Some(6));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(6));
    assert_eq!(game.continuous_effects.len(), 2); // anthem + growth
}
