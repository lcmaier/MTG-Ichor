//! Phase LC Integration Tests: Layer 5 Color-Changing Effects
//!
//! Tests color-changing spells and static abilities end-to-end:
//! cast → resolve (registers ContinuousEffect at Layer5Color) →
//! layer system computes effective colors → cleanup step expires effects.


use mtgsim::cards::authoring::{color, creature, each, gets, static_ability, you_control};
use mtgsim::cards::{fixtures, printings};
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::oracle::characteristics::{get_effective_colors, get_effective_power, get_effective_toughness};
use mtgsim::oracle::legality::candidate_priority_actions;
use mtgsim::types::colors::Color;
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

use mtgsim::engine::actions::ZoneChangeAct;
use mtgsim::test_support::test_ctx;
use mtgsim::test_support::{creature_with_ability, fill_library, put_in_hand, put_on_battlefield, setup_two_player_game, test_dp};

/// Helper: cast a targeted creature spell from hand and resolve it.
fn cast_and_resolve_targeted_spell(
    game: &mut mtgsim::state::game_state::GameState,
    decisions: &ScriptedDecisionProvider,
    _spell_id: mtgsim::types::ids::ObjectId,
    cast_index: usize,
) {
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![cast_index]);
    // No target to script: one legal target on these boards, so the choice
    // is forced and no prompt is made.
    let result = game.run_priority_round(decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);

    // Both pass → resolve. The caster is asked while another spell in hand is
    // castable; the opponent has `Pass` alone, which the engine takes.
    if candidate_priority_actions(game, game.active_player).len() > 1 {
        decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    }
    game.run_priority_round(decisions).unwrap();
}

// ===========================================================================
// Test 1: Cerulean Wisps sets creature to blue
// ===========================================================================

#[test]
fn test_cerulean_wisps_sets_color_to_blue() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, printings::grizzly_bears(), 0);
    let wisps_id = put_in_hand(&mut game, printings::cerulean_wisps(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 1);

    // Verify base color is green
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Green));
    assert!(!colors.contains(&Color::Blue));

    let decisions = ScriptedDecisionProvider::new();
    cast_and_resolve_targeted_spell(&mut game, &decisions, wisps_id, 1);

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
    let bears_id = put_on_battlefield(&mut game, printings::grizzly_bears(), 0);
    let wisps_id = put_in_hand(&mut game, printings::cerulean_wisps(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 1);

    let decisions = ScriptedDecisionProvider::new();
    cast_and_resolve_targeted_spell(&mut game, &decisions, wisps_id, 1);

    // Blue now
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Blue));
    assert!(!colors.contains(&Color::Green));

    // Six positions from the precombat main phase: nothing attacked, so
    // CR 508.8's three combat steps are refused at the proposal site.
    for _ in 0..6 {
        game.advance_turn(&test_dp()).unwrap();
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
    let bears_id = put_on_battlefield(&mut game, printings::grizzly_bears(), 0);
    let moonlace_id = put_in_hand(&mut game, fixtures::moonlace_fixture(), 0);
    game.players[0].mana_pool.add(ManaType::Blue, 1);

    // Base: green
    assert!(get_effective_colors(&game, bears_id).contains(&Color::Green));

    let decisions = ScriptedDecisionProvider::new();
    cast_and_resolve_targeted_spell(&mut game, &decisions, moonlace_id, 1);

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
    let bears_id = put_on_battlefield(&mut game, printings::grizzly_bears(), 0);
    // Player 1's green creature
    let opp_bears_id = put_on_battlefield(&mut game, printings::grizzly_bears(), 1);

    // Place Chromatic Ward — static effect registers on ETB
    let _ward_id = put_on_battlefield(&mut game, fixtures::chromatic_ward(), 0);

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
    let _ward_id = put_on_battlefield(&mut game, fixtures::chromatic_ward(), 0);

    // Creature enters after
    let bears_id = put_on_battlefield(&mut game, printings::grizzly_bears(), 0);

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
    let bears_id = put_on_battlefield(&mut game, printings::grizzly_bears(), 0);
    let ward_id = put_on_battlefield(&mut game, fixtures::chromatic_ward(), 0);

    // Green + red
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Red));
    assert_eq!(game.continuous_effects.len(), 1);

    // Destroy the ward
    game.change_zone(ward_id, Zone::Graveyard, ZoneChangeAct::Destroyed, &test_ctx()).unwrap();

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
    let bears_id = put_on_battlefield(&mut game, printings::grizzly_bears(), 0);
    let wisps_id = put_in_hand(&mut game, printings::cerulean_wisps(), 0);
    let growth_id = put_in_hand(&mut game, mtgsim::cards::printings::giant_growth(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    game.players[0].mana_pool.add(ManaType::Green, 1);

    let decisions = ScriptedDecisionProvider::new();

    // Cast Cerulean Wisps → creature becomes blue
    cast_and_resolve_targeted_spell(&mut game, &decisions, wisps_id, 1);

    // Cast Giant Growth → creature gets +3/+3
    cast_and_resolve_targeted_spell(&mut game, &decisions, growth_id, 1);

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
    let bears_id = put_on_battlefield(&mut game, printings::grizzly_bears(), 0);
    let wisps1_id = put_in_hand(&mut game, printings::cerulean_wisps(), 0);
    let moonlace_id = put_in_hand(&mut game, fixtures::moonlace_fixture(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 2);

    let decisions = ScriptedDecisionProvider::new();

    // Cast Cerulean Wisps → becomes blue
    cast_and_resolve_targeted_spell(&mut game, &decisions, wisps1_id, 1);
    let colors = get_effective_colors(&game, bears_id);
    assert!(colors.contains(&Color::Blue));

    // Cast Moonlace → becomes colorless
    cast_and_resolve_targeted_spell(&mut game, &decisions, moonlace_id, 1);
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
    let land_id = mtgsim::test_support::put_land_on_battlefield(&mut game, mtgsim::cards::printings::forest, 0);

    // Place Chromatic Ward
    let _ward_id = put_on_battlefield(&mut game, fixtures::chromatic_ward(), 0);

    // Land should still just be green (from its card data... though basic lands are typically colorless)
    let colors = get_effective_colors(&game, land_id);
    assert!(!colors.contains(&Color::Red)); // not a creature, not affected
}

// ===========================================================================
// Cerulean Wisps as printed, and its rulings
// ===========================================================================

/// "Untap that creature": the target the color change took.
#[test]
fn cerulean_wisps_untaps_the_creature_it_targets() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, printings::grizzly_bears(), 0);
    game.execute_action(GameAction::Tap { object: bears_id }, &test_ctx()).unwrap();
    let wisps_id = put_in_hand(&mut game, printings::cerulean_wisps(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    assert!(game.battlefield[&bears_id].tapped);

    cast_and_resolve_targeted_spell(&mut game, &ScriptedDecisionProvider::new(), wisps_id, 1);

    assert!(!game.battlefield[&bears_id].tapped, "untapped by the Wisps");
    assert_eq!(get_effective_colors(&game, bears_id), [Color::Blue].into());
}

// RULING: Cerulean Wisps #1 - "An effect that changes a permanent's colors overwrites all its old colors ..."
#[test]
fn cerulean_wisps_makes_a_three_color_creature_just_blue() {
    let mut game = setup_two_player_game();
    let monk = put_on_battlefield(&mut game, printings::rhox_war_monk(), 0);
    let wisps_id = put_in_hand(&mut game, printings::cerulean_wisps(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    assert_eq!(get_effective_colors(&game, monk).len(), 3, "green, white and blue as printed");

    cast_and_resolve_targeted_spell(&mut game, &ScriptedDecisionProvider::new(), wisps_id, 1);

    assert_eq!(get_effective_colors(&game, monk), [Color::Blue].into(), "blue alone, though it was blue before");
}

// RULING: Cerulean Wisps #2 - "Changing a permanent's color won't change its text. ..."
/// Wilt-Leaf Liege's shape, one color: "Green creatures you control get
/// +1/+1" on a green creature. Turned blue, it no longer pumps itself, and
/// its text still pumps a green creature.
#[test]
fn a_creature_turned_blue_still_pumps_green_creatures() {
    let mut game = setup_two_player_game();
    let anthem = creature_with_ability(
        "Green Anthem Bearer",
        2,
        2,
        static_ability("Green creatures you control get +1/+1.", gets(each(creature().and(color(Color::Green)).and(you_control())), 1, 1)),
    );
    let bearer = put_on_battlefield(&mut game, anthem, 0);
    let wisps_id = put_in_hand(&mut game, printings::cerulean_wisps(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    assert_eq!(get_effective_power(&game, bearer), Some(3), "green, so it pumps itself");

    cast_and_resolve_targeted_spell(&mut game, &ScriptedDecisionProvider::new(), wisps_id, 1);
    let bears_id = put_on_battlefield(&mut game, printings::grizzly_bears(), 0);

    assert_eq!(get_effective_power(&game, bearer), Some(2), "blue now, so not a green creature");
    assert_eq!(get_effective_power(&game, bears_id), Some(3), "its text still says green");
}

// ===========================================================================
// Moonlace Fixture lasts until end of turn, as its text says
// ===========================================================================

/// A target creature, colorless until end of turn and green again at the
/// cleanup: the duration its text states, where Moonlace's effect never ends.
#[test]
fn moonlace_fixture_is_colorless_until_end_of_turn() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, printings::grizzly_bears(), 0);
    let moonlace_id = put_in_hand(&mut game, fixtures::moonlace_fixture(), 0);
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    cast_and_resolve_targeted_spell(&mut game, &ScriptedDecisionProvider::new(), moonlace_id, 1);
    assert!(get_effective_colors(&game, bears_id).is_empty());

    for _ in 0..6 {
        game.advance_turn(&test_dp()).unwrap();
    }
    assert_eq!(game.phase.step, Some(mtgsim::state::game_state::StepType::Cleanup));
    assert_eq!(get_effective_colors(&game, bears_id), [Color::Green].into(), "the effect ended with the turn");
}
