//! Phase LB Integration Tests: Continuous Effects via Layer System
//!
//! Tests the Giant Growth pump spell end-to-end:
//! cast → resolve (registers continuous effect) → layer system computes P/T →
//! cleanup step expires UntilEndOfTurn effects.


use mtgsim::cards::alpha;
use mtgsim::cards::creatures;
use mtgsim::cards::utility_creatures;
use mtgsim::cards::phase5_pre_cards;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::oracle::characteristics::{get_effective_power, get_effective_toughness};
use mtgsim::types::effects::{EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

use mtgsim::engine::actions::ZoneChangeCause;
use mtgsim::test_support::test_ctx;
use mtgsim::test_support::{
    fill_library, pass_turn, put_in_hand, put_on_battlefield, setup_two_player_game,
};

// ---------------------------------------------------------------------------
// Test 1: Giant Growth gives +3/+3 until end of turn
// ---------------------------------------------------------------------------

// COVERS: ATOM-302.4c-001
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

// COVERS: ATOM-611.2a-001
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

// COVERS-PARTIAL: ATOM-613.4c-001
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

// COVERS-PARTIAL: ATOM-604.2-001
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

// COVERS: ATOM-611.3c-001
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
    let land_id = mtgsim::test_support::put_land_on_battlefield(&mut game, mtgsim::cards::basic_lands::forest, 0);

    // Place Glorious Anthem
    let _anthem_id = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);

    // Land has no P/T
    assert_eq!(get_effective_power(&game, land_id), None);
    assert_eq!(get_effective_toughness(&game, land_id), None);
}

// ---------------------------------------------------------------------------
// Test 8: Anthem effect removed when anthem leaves the battlefield
// ---------------------------------------------------------------------------

// COVERS: ATOM-604.2-001, ATOM-611.3b-001
#[test]
fn test_glorious_anthem_removed_on_ltb() {
    let mut game = setup_two_player_game();

    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let anthem_id = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);

    // Buffed
    assert_eq!(get_effective_power(&game, bears_id), Some(3));
    assert_eq!(game.continuous_effects.len(), 1);

    // Destroy the anthem (move to graveyard)
    game.change_zone(anthem_id, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();

    // Effect removed, creature back to base
    assert_eq!(game.continuous_effects.len(), 0);
    assert_eq!(get_effective_power(&game, bears_id), Some(2));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(2));
}

// ---------------------------------------------------------------------------
// Test 9: Two anthems stack additively
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-613.4c-001
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

// COVERS-PARTIAL: ATOM-613.4c-001, ATOM-613.1g-001
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

// ===========================================================================
// Layer 7b Tests: SetPowerToughness (Zhalfirin Shapecraft)
// ===========================================================================

// ---------------------------------------------------------------------------
// Test 11: Zhalfirin Shapecraft sets base P/T to 4/3
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-613.4b-001
#[test]
fn test_zhalfirin_shapecraft_sets_base_pt() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let spell_id = put_in_hand(&mut game, phase5_pre_cards::zhalfirin_shapecraft(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 2);

    // Base 2/2
    assert_eq!(get_effective_power(&game, bears_id), Some(2));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(2));

    let decisions = ScriptedDecisionProvider::new();

    // Cast Zhalfirin Shapecraft targeting the creature
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id,
        },
        vec![0],
    );
    // {1}{U}: pool has [Blue(2)], allocate 1 generic → [1]
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
        vec![1],
    );
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);

    // Resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(&decisions).unwrap();

    // Creature should be 4/3, and we drew a card
    assert_eq!(get_effective_power(&game, bears_id), Some(4));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(3));
    assert_eq!(game.players[0].hand.len(), 1); // drew 1 card
}

// ---------------------------------------------------------------------------
// Test 12: Inside Out switches P/T
// ---------------------------------------------------------------------------

// COVERS: ATOM-613.4d-001
#[test]
fn test_inside_out_switches_pt() {
    let mut game = setup_two_player_game();
    // Use a creature with asymmetric P/T: Hill Giant 3/3 won't show the switch.
    // Use Earth Elemental 4/5 instead.
    let creature_data = mtgsim::objects::card_data::CardDataBuilder::new("Earth Elemental")
        .card_type(mtgsim::types::card_types::CardType::Creature)
        .power_toughness(4, 5)
        .build();
    let creature_id = put_on_battlefield(&mut game, creature_data, 0);
    let spell_id = put_in_hand(&mut game, phase5_pre_cards::inside_out(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 2);

    // Base 4/5
    assert_eq!(get_effective_power(&game, creature_id), Some(4));
    assert_eq!(get_effective_toughness(&game, creature_id), Some(5));

    let decisions = ScriptedDecisionProvider::new();

    // Cast Inside Out targeting the creature
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id,
        },
        vec![0],
    );
    // {1}{U}: pool has [Blue(2)], allocate 1 generic → [1]
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
        vec![1],
    );
    game.run_priority_round(&decisions).unwrap();

    // Resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(&decisions).unwrap();

    // Creature should be 5/4 (swapped), and we drew a card
    assert_eq!(get_effective_power(&game, creature_id), Some(5));
    assert_eq!(get_effective_toughness(&game, creature_id), Some(4));
    assert_eq!(game.players[0].hand.len(), 1);
}

// ---------------------------------------------------------------------------
// Test 13: Bull Rush gives +2/+0
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-613.4c-001
#[test]
fn test_bull_rush_pumps_power() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let spell_id = put_in_hand(&mut game, phase5_pre_cards::bull_rush(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let decisions = ScriptedDecisionProvider::new();

    // Cast Bull Rush
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id,
        },
        vec![0],
    );
    game.run_priority_round(&decisions).unwrap();

    // Resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(&decisions).unwrap();

    // 2+2 = 4 power, toughness stays 2
    assert_eq!(get_effective_power(&game, bears_id), Some(4));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(2));
}

// ===========================================================================
// Layer Ordering Tests: 7b → 7c → 7d
// ===========================================================================

/// Helper: cast a targeted spell from hand and resolve it.
/// `generic_alloc` — if the spell has generic mana cost, provide the allocation vector.
fn cast_and_resolve_targeted_spell(
    game: &mut mtgsim::state::game_state::GameState,
    decisions: &ScriptedDecisionProvider,
    spell_id: mtgsim::types::ids::ObjectId,
    cast_index: usize,
    target_index: usize,
    generic_alloc: Option<Vec<u64>>,
) {
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![cast_index]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id,
        },
        vec![target_index],
    );
    if let Some(alloc) = generic_alloc {
        decisions.expect_allocation(
            ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
            alloc,
        );
    }
    let result = game.run_priority_round(decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);

    // Both pass → resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(decisions).unwrap();
}

// ---------------------------------------------------------------------------
// Test 14: All three spells on same creature → P/T 3/6 regardless of order
// Cast order: Shapecraft (7b), Bull Rush (7c), Inside Out (7d)
// ---------------------------------------------------------------------------

#[test]
fn test_layer_ordering_7b_7c_7d_cast_in_layer_order() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let shapecraft_id = put_in_hand(&mut game, phase5_pre_cards::zhalfirin_shapecraft(), 0);
    let bull_rush_id = put_in_hand(&mut game, phase5_pre_cards::bull_rush(), 0);
    let inside_out_id = put_in_hand(&mut game, phase5_pre_cards::inside_out(), 0);
    fill_library(&mut game, 0, 5);
    // Provide plenty of mana: Blue for Shapecraft+Inside Out, Red for Bull Rush
    game.players[0].mana_pool.add(ManaType::Blue, 10);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let decisions = ScriptedDecisionProvider::new();

    // Shapecraft {1}{U}: available [Blue(10), Red(1)] → allocate 1 generic from Blue
    cast_and_resolve_targeted_spell(&mut game, &decisions, shapecraft_id, 1, 0, Some(vec![1, 0]));
    // After 7b: base set to 4/3
    assert_eq!(get_effective_power(&game, bears_id), Some(4));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(3));

    // Bull Rush {R}: no generic cost
    cast_and_resolve_targeted_spell(&mut game, &decisions, bull_rush_id, 1, 0, None);
    // After 7b+7c: 4+2=6, 3+0=3
    assert_eq!(get_effective_power(&game, bears_id), Some(6));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(3));

    // Inside Out {1}{U}: available [Blue(8)] (Red spent) → allocate 1 from Blue
    cast_and_resolve_targeted_spell(&mut game, &decisions, inside_out_id, 1, 0, Some(vec![1]));
    // After 7b+7c+7d: swap(6,3) = 3/6
    assert_eq!(get_effective_power(&game, bears_id), Some(3));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(6));
}

// ---------------------------------------------------------------------------
// Test 15: Same result when cast in REVERSE order (7d, 7c, 7b)
// Proves the layer system applies in fixed order, not cast order.
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-613.4d-004
//
// The atom's mechanism exactly: Bull Rush's +2/+0 is created *after* the switch
// and still applies before it, so the pump lands on the unswitched side and the
// switch happens last (2/2 -> 4/2 -> 2/4). Partial because the atom's board is a
// 1/3 under two modifiers (+0/+1 and +5/+0, expecting 4/6) and this is a 2/2
// under one.
//
// The annotation used to sit on `test_layer_ordering_7b_7c_7d_cast_in_layer_order`
// as a full COVERS, which is the one arrangement that cannot show this: casting
// in layer order means nothing is ever added after the switch. Caught by
// `specdb suspicious` (2026-08-24).
#[test]
fn test_layer_ordering_cast_in_reverse_order() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    // Add to hand in CAST ORDER so index 1 always picks the next intended spell
    let inside_out_id = put_in_hand(&mut game, phase5_pre_cards::inside_out(), 0);
    let bull_rush_id = put_in_hand(&mut game, phase5_pre_cards::bull_rush(), 0);
    let shapecraft_id = put_in_hand(&mut game, phase5_pre_cards::zhalfirin_shapecraft(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 10);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let decisions = ScriptedDecisionProvider::new();

    // Inside Out {1}{U}: available [Blue(10), Red(1)] → allocate 1 from Blue
    cast_and_resolve_targeted_spell(&mut game, &decisions, inside_out_id, 1, 0, Some(vec![1, 0]));
    // Only 7d active: swap base 2/2 → still 2/2 (symmetric!)
    assert_eq!(get_effective_power(&game, bears_id), Some(2));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(2));

    // Bull Rush {R}: no generic
    cast_and_resolve_targeted_spell(&mut game, &decisions, bull_rush_id, 1, 0, None);
    // 7c: 2+2=4, 2+0=2; then 7d: swap → 2/4
    assert_eq!(get_effective_power(&game, bears_id), Some(2));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(4));

    // Shapecraft {1}{U}: available [Blue(8)] (Red spent) → allocate 1 from Blue
    cast_and_resolve_targeted_spell(&mut game, &decisions, shapecraft_id, 1, 0, Some(vec![1]));
    // 7b: set 4/3; 7c: +2/+0 → 6/3; 7d: swap → 3/6
    assert_eq!(get_effective_power(&game, bears_id), Some(3));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(6));
}

// ---------------------------------------------------------------------------
// Test 16: Cast order 7c, 7d, 7b — same final result 3/6
// ---------------------------------------------------------------------------

#[test]
fn test_layer_ordering_cast_7c_7d_7b() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    // Add to hand in CAST ORDER so index 1 always picks the next intended spell
    let bull_rush_id = put_in_hand(&mut game, phase5_pre_cards::bull_rush(), 0);
    let inside_out_id = put_in_hand(&mut game, phase5_pre_cards::inside_out(), 0);
    let shapecraft_id = put_in_hand(&mut game, phase5_pre_cards::zhalfirin_shapecraft(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 10);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let decisions = ScriptedDecisionProvider::new();

    // Bull Rush {R}: no generic
    cast_and_resolve_targeted_spell(&mut game, &decisions, bull_rush_id, 1, 0, None);
    // Only 7c: 2+2=4, 2+0=2
    assert_eq!(get_effective_power(&game, bears_id), Some(4));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(2));

    // Inside Out {1}{U}: available [Blue(10)] (Red spent) → allocate 1 from Blue
    cast_and_resolve_targeted_spell(&mut game, &decisions, inside_out_id, 1, 0, Some(vec![1]));
    // 7c: 4/2; 7d: swap → 2/4
    assert_eq!(get_effective_power(&game, bears_id), Some(2));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(4));

    // Shapecraft {1}{U}: available [Blue(8)] → allocate 1 from Blue
    cast_and_resolve_targeted_spell(&mut game, &decisions, shapecraft_id, 1, 0, Some(vec![1]));
    // 7b: 4/3; 7c: +2/+0 → 6/3; 7d: swap → 3/6
    assert_eq!(get_effective_power(&game, bears_id), Some(3));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(6));
}

// ---------------------------------------------------------------------------
// Test 17: All effects expire at cleanup
// ---------------------------------------------------------------------------

#[test]
fn test_layer_effects_expire_at_cleanup() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let shapecraft_id = put_in_hand(&mut game, phase5_pre_cards::zhalfirin_shapecraft(), 0);
    let bull_rush_id = put_in_hand(&mut game, phase5_pre_cards::bull_rush(), 0);
    let inside_out_id = put_in_hand(&mut game, phase5_pre_cards::inside_out(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 10);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let decisions = ScriptedDecisionProvider::new();

    // Cast all three
    cast_and_resolve_targeted_spell(&mut game, &decisions, shapecraft_id, 1, 0, Some(vec![1, 0]));
    cast_and_resolve_targeted_spell(&mut game, &decisions, bull_rush_id, 1, 0, None);
    cast_and_resolve_targeted_spell(&mut game, &decisions, inside_out_id, 1, 0, Some(vec![1]));

    // 3/6 as expected
    assert_eq!(get_effective_power(&game, bears_id), Some(3));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(6));
    assert_eq!(game.continuous_effects.len(), 3);

    // Advance to cleanup (9 steps from precombat main)
    for _ in 0..9 {
        game.advance_turn().unwrap();
    }

    // All effects expired, back to base 2/2
    assert_eq!(game.continuous_effects.len(), 0);
    assert_eq!(get_effective_power(&game, bears_id), Some(2));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(2));
}

// ---------------------------------------------------------------------------
// Test 13: Merfolk Thaumaturgist — Layer 7d from a registered permanent
//
// `inside_out` above proves the *primitive*; it can never prove the pool,
// because its printed cost is hybrid ({1}{U/R}) and it is therefore a fixture
// that will not be registered. The Thaumaturgist is the registered 7d source,
// and it reaches the layer through the activated-ability path rather than a
// spell — the first `AbilityType::Activated` ability in `CardRegistry`, so
// `run_priority_round`'s ActivateAbility arm gets its first pool-level test
// here too.
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-613.4d-001
//
// Partial: the atom's board is a switch on a creature, which
// test_inside_out_switches_pt builds exactly. This one differs in how the
// effect is created (activated ability, tap cost) rather than in what 7d does.
#[test]
fn test_merfolk_thaumaturgist_switches_pt_from_an_activated_ability() {
    let mut game = setup_two_player_game();

    // Placed before the Thaumaturgist so it holds the earlier timestamp and is
    // therefore index 0 in the target list — `battlefield_ordered` is what the
    // DP picks by, and the ordering is the point of that guarantee.
    let elemental = put_on_battlefield(&mut game, creatures::earth_elemental(), 0);
    let thaumaturgist =
        put_on_battlefield(&mut game, utility_creatures::merfolk_thaumaturgist(), 0);

    assert_eq!(get_effective_power(&game, elemental), Some(4));
    assert_eq!(get_effective_toughness(&game, elemental), Some(5));

    let decisions = ScriptedDecisionProvider::new();

    // [0] Pass, [1] activate the Thaumaturgist — nothing else on this board can
    // be activated, cast or played.
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id: thaumaturgist,
        },
        vec![0],
    );
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);

    assert!(
        game.battlefield[&thaumaturgist].tapped,
        "the {{T}} cost is paid on activation (CR 602.2b), not on resolution"
    );
    assert_eq!(
        get_effective_power(&game, elemental),
        Some(4),
        "and nothing has happened yet — the ability is still on the stack"
    );

    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(&decisions).unwrap();

    assert_eq!(get_effective_power(&game, elemental), Some(5), "Layer 7d");
    assert_eq!(get_effective_toughness(&game, elemental), Some(4));
    assert_eq!(
        (get_effective_power(&game, thaumaturgist), get_effective_toughness(&game, thaumaturgist)),
        (Some(1), Some(2)),
        "it targeted the Elemental, not itself"
    );

    pass_turn(&mut game);
    assert_eq!(
        (get_effective_power(&game, elemental), get_effective_toughness(&game, elemental)),
        (Some(4), Some(5)),
        "until end of turn, so cleanup takes it back"
    );
}
