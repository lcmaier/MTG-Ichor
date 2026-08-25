//! Phase LC Integration Tests: Layer 5 Color-Changing Effects
//!
//! Tests color-changing spells and static abilities end-to-end:
//! cast → resolve (registers ContinuousEffect at Layer5Color) →
//! layer system computes effective colors → cleanup step expires effects.


use mtgsim::cards::creatures;
use mtgsim::cards::phase_lc_cards;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::oracle::characteristics::{get_effective_colors, get_effective_power, get_effective_toughness};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

use mtgsim::test_support::test_ctx;
use mtgsim::test_support::{fill_library, put_in_hand, put_on_battlefield, setup_two_player_game};

/// Helper: cast a targeted creature spell from hand and resolve it.
fn cast_and_resolve_targeted_spell(
    game: &mut mtgsim::state::game_state::GameState,
    decisions: &ScriptedDecisionProvider,
    spell_id: mtgsim::types::ids::ObjectId,
    cast_index: usize,
    target_index: usize,
) {
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![cast_index]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id,
        },
        vec![target_index],
    );
    let result = game.run_priority_round(decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);

    // Both pass → resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(decisions).unwrap();
}

// ===========================================================================
// Test 1: Cerulean Wisps sets creature to blue
// ===========================================================================

#[test]
fn test_cerulean_wisps_sets_color_to_blue() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let wisps_id = put_in_hand(&mut game, phase_lc_cards::cerulean_wisps(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 1);

    // Verify base color is green
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Green));
    assert!(!colors.contains(&Color::Blue));

    let decisions = ScriptedDecisionProvider::new();
    cast_and_resolve_targeted_spell(&mut game, &decisions, wisps_id, 1, 0);

    // Creature should now be blue (not green)
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Blue));
    assert!(!colors.contains(&Color::Green));
    assert_eq!(colors.len(), 1);

    // Wisps is in graveyard
    assert_eq!(game.get_object(wisps_id).unwrap().zone, Zone::Graveyard);

    // Drew a card (Sequence: ChangeColor + DrawCards)
    assert_eq!(game.players[0].hand.len(), 1);

    // One continuous effect registered
    assert_eq!(game.continuous_effects.len(), 1);
}

// ===========================================================================
// Test 2: Color change expires at cleanup → reverts to printed colors
// ===========================================================================

// COVERS-PARTIAL: ATOM-611.2a-001
#[test]
fn test_color_change_expires_at_cleanup() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let wisps_id = put_in_hand(&mut game, phase_lc_cards::cerulean_wisps(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 1);

    let decisions = ScriptedDecisionProvider::new();
    cast_and_resolve_targeted_spell(&mut game, &decisions, wisps_id, 1, 0);

    // Blue now
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Blue));
    assert!(!colors.contains(&Color::Green));

    // Advance to cleanup (9 steps from precombat main)
    for _ in 0..9 {
        game.advance_turn().unwrap();
    }

    // Verify we're in cleanup
    assert_eq!(game.phase.phase_type, mtgsim::state::game_state::PhaseType::Ending);
    assert_eq!(game.phase.step, Some(mtgsim::state::game_state::StepType::Cleanup));

    // Effect expired — back to green
    assert_eq!(game.continuous_effects.len(), 0);
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Green));
    assert!(!colors.contains(&Color::Blue));
}

// ===========================================================================
// Test 3: Moonlace makes creature colorless
// ===========================================================================

#[test]
fn test_moonlace_makes_creature_colorless() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let moonlace_id = put_in_hand(&mut game, phase_lc_cards::moonlace(), 0);
    game.players[0].mana_pool.add(ManaType::Blue, 1);

    // Base: green
    assert!(get_effective_colors(&game, bears_id).contains(&Color::Green));

    let decisions = ScriptedDecisionProvider::new();
    cast_and_resolve_targeted_spell(&mut game, &decisions, moonlace_id, 1, 0);

    // Now colorless
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.is_empty());
}

// ===========================================================================
// Test 4: Chromatic Ward (static) adds red to your creatures
// ===========================================================================

// COVERS-PARTIAL: ATOM-604.2-001
#[test]
fn test_chromatic_ward_static_adds_red() {
    let mut game = setup_two_player_game();

    // Player 0's green creature
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    // Player 1's green creature
    let opp_bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);

    // Place Chromatic Ward — static effect registers on ETB
    let _ward_id = put_on_battlefield(&mut game, phase_lc_cards::chromatic_ward(), 0);

    // Player 0's creature: green + red
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Green));
    assert!(colors.contains(&Color::Red));
    assert_eq!(colors.len(), 2);

    // Player 1's creature: still just green (not affected)
    let opp_colors = get_effective_colors(&game, opp_bears_id);
    assert!(opp_colors.contains(&Color::Green));
    assert!(!opp_colors.contains(&Color::Red));
}

// ===========================================================================
// Test 5: Chromatic Ward applies to creatures entering after it
// ===========================================================================

// COVERS-PARTIAL: ATOM-611.3c-001
#[test]
fn test_chromatic_ward_applies_to_later_creatures() {
    let mut game = setup_two_player_game();

    // Enchantment first
    let _ward_id = put_on_battlefield(&mut game, phase_lc_cards::chromatic_ward(), 0);

    // Creature enters after
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);

    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Green));
    assert!(colors.contains(&Color::Red));
}

// ===========================================================================
// Test 6: Chromatic Ward effect removed when ward leaves the battlefield
// ===========================================================================

// COVERS-PARTIAL: ATOM-611.3b-001
#[test]
fn test_chromatic_ward_removed_on_ltb() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let ward_id = put_on_battlefield(&mut game, phase_lc_cards::chromatic_ward(), 0);

    // Green + red
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Red));
    assert_eq!(game.continuous_effects.len(), 1);

    // Destroy the ward
    game.change_zone(ward_id, Zone::Graveyard, &test_ctx()).unwrap();

    // Back to just green
    assert_eq!(game.continuous_effects.len(), 0);
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Green));
    assert!(!colors.contains(&Color::Red));
    assert_eq!(colors.len(), 1);
}

// ===========================================================================
// Test 7: Layer 5 color change is independent of Layer 7 P/T effects
// ===========================================================================

#[test]
fn test_color_change_independent_of_pt_pump() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let wisps_id = put_in_hand(&mut game, phase_lc_cards::cerulean_wisps(), 0);
    let growth_id = put_in_hand(&mut game, mtgsim::cards::alpha::giant_growth(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    game.players[0].mana_pool.add(ManaType::Green, 1);

    let decisions = ScriptedDecisionProvider::new();

    // Cast Cerulean Wisps → creature becomes blue
    cast_and_resolve_targeted_spell(&mut game, &decisions, wisps_id, 1, 0);

    // Cast Giant Growth → creature gets +3/+3
    cast_and_resolve_targeted_spell(&mut game, &decisions, growth_id, 1, 0);

    // Color should be blue (L5), P/T should be 5/5 (L7c)
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Blue));
    assert!(!colors.contains(&Color::Green));
    assert_eq!(get_effective_power(&game, bears_id), Some(5));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(5));
}

// ===========================================================================
// Test 8: Two color-changing spells — later one overrides (SetColors)
// ===========================================================================

// COVERS-PARTIAL: ATOM-613.7-001
#[test]
fn test_two_set_colors_later_overrides() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let wisps1_id = put_in_hand(&mut game, phase_lc_cards::cerulean_wisps(), 0);
    let moonlace_id = put_in_hand(&mut game, phase_lc_cards::moonlace(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 2);

    let decisions = ScriptedDecisionProvider::new();

    // Cast Cerulean Wisps → becomes blue
    cast_and_resolve_targeted_spell(&mut game, &decisions, wisps1_id, 1, 0);
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Blue));

    // Cast Moonlace → becomes colorless
    cast_and_resolve_targeted_spell(&mut game, &decisions, moonlace_id, 1, 0);
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.is_empty());

    assert_eq!(game.continuous_effects.len(), 2);
}

// ===========================================================================
// Test 9: Chromatic Ward does not affect non-creature permanents
// ===========================================================================

#[test]
fn test_chromatic_ward_ignores_noncreatures() {
    let mut game = setup_two_player_game();

    // Place a land
    let land_id = mtgsim::test_support::put_land_on_battlefield(&mut game, mtgsim::cards::basic_lands::forest, 0);

    // Place Chromatic Ward
    let _ward_id = put_on_battlefield(&mut game, phase_lc_cards::chromatic_ward(), 0);

    // Land should still just be green (from its card data... though basic lands are typically colorless)
    let colors = get_effective_colors(&game, land_id);
    assert!(!colors.contains(&Color::Red)); // not a creature, not affected
}
