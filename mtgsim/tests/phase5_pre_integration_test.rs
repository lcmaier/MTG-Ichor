//! Phase 5-Pre Integration Tests: New cards and engine features.
//!
//! Tests exercise the full cast → resolve → SBA pipeline for the phase5_pre
//! cards: Night's Whisper, Doom Blade, Angel's Mercy, Dark Ritual, and
//! Isamaru (legend rule).

mod common;

use mtgsim::cards::basic_lands;
use mtgsim::cards::creatures;
use mtgsim::cards::phase5_pre_cards;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::engine::resolve::ResolvedTarget;
use mtgsim::objects::card_data::CardDataBuilder;
use mtgsim::state::game_state::{GameState, PhaseType};
use mtgsim::types::card_types::CardType;
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::decision::{PriorityAction, ScriptedDecisionProvider};

use common::{setup_two_player_game, put_in_hand, put_on_battlefield, fill_library};

// ---------------------------------------------------------------------------
// Phase-specific helpers
// ---------------------------------------------------------------------------

fn cast_and_resolve(
    game: &mut GameState,
    decisions: &ScriptedDecisionProvider,
) {
    // Round 1: Cast
    let result = game.run_priority_round(decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);

    // Round 2: Both pass → resolve
    let result = game.run_priority_round(decisions).unwrap();
    assert_eq!(result, PriorityResult::StackResolved);
}

// ---------------------------------------------------------------------------
// Night's Whisper — Draw 2, lose 2 life (sorcery)
// ---------------------------------------------------------------------------

#[test]
fn test_nights_whisper_draw_and_life_loss() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 10);
    let whisper_id = put_in_hand(&mut game, phase5_pre_cards::nights_whisper(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(whisper_id));

    cast_and_resolve(&mut game, &decisions);

    // Drew 2 cards
    assert_eq!(game.players[0].hand.len(), 2);
    assert_eq!(game.players[0].library.len(), 8);
    // Lost 2 life
    assert_eq!(game.players[0].life_total, 18);
    // Sorcery in graveyard
    assert_eq!(game.get_object(whisper_id).unwrap().zone, Zone::Graveyard);
}

#[test]
fn test_nights_whisper_sorcery_speed_wrong_phase() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 10);
    let whisper_id = put_in_hand(&mut game, phase5_pre_cards::nights_whisper(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);

    // Set to combat phase — sorcery can't be cast here
    game.phase = mtgsim::state::game_state::Phase::new(PhaseType::Combat);

    let decisions = ScriptedDecisionProvider::new();
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(whisper_id));

    let result = game.run_priority_round(&decisions);
    // Should fail — sorcery during combat
    assert!(result.is_err() || game.stack.is_empty());
}

// ---------------------------------------------------------------------------
// Angel's Mercy — Gain 7 life
// ---------------------------------------------------------------------------

#[test]
fn test_angels_mercy_gains_life() {
    let mut game = setup_two_player_game();
    let mercy_id = put_in_hand(&mut game, phase5_pre_cards::angels_mercy(), 0);
    game.players[0].mana_pool.add(ManaType::White, 2);
    game.players[0].mana_pool.add(ManaType::Colorless, 2);

    let decisions = ScriptedDecisionProvider::new();
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(mercy_id));

    cast_and_resolve(&mut game, &decisions);

    assert_eq!(game.players[0].life_total, 27);
    assert_eq!(game.get_object(mercy_id).unwrap().zone, Zone::Graveyard);
}

// ---------------------------------------------------------------------------
// Dark Ritual — Add {B}{B}{B} (spell, NOT a mana ability)
// ---------------------------------------------------------------------------

#[test]
fn test_dark_ritual_adds_three_black_mana() {
    let mut game = setup_two_player_game();
    let ritual_id = put_in_hand(&mut game, phase5_pre_cards::dark_ritual(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(ritual_id));

    cast_and_resolve(&mut game, &decisions);

    // Started with 1B, spent 1B to cast, gained 3B → 3B in pool
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Black), 3);
    assert_eq!(game.get_object(ritual_id).unwrap().zone, Zone::Graveyard);
}

#[test]
fn test_dark_ritual_uses_stack_not_mana_ability() {
    // Dark Ritual is an instant that produces mana via a spell effect.
    // It goes on the stack and can be responded to (unlike a mana ability).
    let mut game = setup_two_player_game();
    let ritual_id = put_in_hand(&mut game, phase5_pre_cards::dark_ritual(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(ritual_id));

    // Cast — it should go on the stack
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);
    assert!(game.stack.contains(&ritual_id));

    // Mana hasn't been added yet — still on the stack
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Black), 0);

    // Now resolve
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::StackResolved);
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Black), 3);
}

// ---------------------------------------------------------------------------
// Doom Blade — Destroy target nonblack creature
// ---------------------------------------------------------------------------

#[test]
fn test_doom_blade_destroys_nonblack_creature() {
    let mut game = setup_two_player_game();
    let target_id = put_on_battlefield(
        &mut game, creatures::grizzly_bears(), 1,
    );
    let blade_id = put_in_hand(&mut game, phase5_pre_cards::doom_blade(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(blade_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(target_id)]);

    cast_and_resolve(&mut game, &decisions);

    // Creature destroyed
    assert_eq!(game.get_object(target_id).unwrap().zone, Zone::Graveyard);
    assert!(!game.battlefield.contains_key(&target_id));
    // Spell in graveyard
    assert_eq!(game.get_object(blade_id).unwrap().zone, Zone::Graveyard);
}

#[test]
fn test_doom_blade_rejects_black_creature() {
    let mut game = setup_two_player_game();
    // Put a black creature on the battlefield — use a creature and add black color
    let black_creature_data = CardDataBuilder::new("Black Knight")
        .card_type(CardType::Creature)
        .color(mtgsim::types::colors::Color::Black)
        .power_toughness(2, 2)
        .build();
    let target_id = put_on_battlefield(&mut game, black_creature_data, 1);
    let blade_id = put_in_hand(&mut game, phase5_pre_cards::doom_blade(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(blade_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(target_id)]);

    // Casting should fail — target is a black creature
    let result = game.run_priority_round(&decisions);
    assert!(result.is_err());
    // Black creature still alive
    assert!(game.battlefield.contains_key(&target_id));
}

#[test]
fn test_doom_blade_rejects_noncreature_permanent() {
    let mut game = setup_two_player_game();
    // A land is a noncreature permanent — Doom Blade can't target it
    let land_id = put_on_battlefield(&mut game, basic_lands::forest(), 1);

    let blade_id = put_in_hand(&mut game, phase5_pre_cards::doom_blade(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(blade_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(land_id)]);

    // Casting should fail — target is not a creature
    let result = game.run_priority_round(&decisions);
    assert!(result.is_err());
    assert!(game.battlefield.contains_key(&land_id));
}

// ---------------------------------------------------------------------------
// Isamaru — Legend rule: second copy triggers SBA
// ---------------------------------------------------------------------------

#[test]
fn test_isamaru_legend_rule_second_copy_dies() {
    let mut game = setup_two_player_game();

    // Place first Isamaru on the battlefield directly
    let first_id = put_on_battlefield(
        &mut game, phase5_pre_cards::isamaru_hound_of_konda(), 0,
    );

    // Cast second Isamaru from hand
    let second_id = put_in_hand(
        &mut game, phase5_pre_cards::isamaru_hound_of_konda(), 0,
    );
    game.players[0].mana_pool.add(ManaType::White, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(second_id));

    // Cast and resolve — second Isamaru enters the battlefield
    cast_and_resolve(&mut game, &decisions);

    // Both should be on the battlefield momentarily, but SBAs run during
    // the next priority check. Run the SBA loop explicitly.
    game.check_state_based_actions_loop(&decisions).unwrap();

    // ScriptedDecisionProvider keeps the first legendary (legendaries[0]).
    // One Isamaru should remain, one should be in the graveyard.
    let first_on_bf = game.battlefield.contains_key(&first_id);
    let second_on_bf = game.battlefield.contains_key(&second_id);
    assert!(
        first_on_bf ^ second_on_bf,
        "Exactly one Isamaru should survive the legend rule"
    );

    // The other should be in the graveyard
    let survivor = if first_on_bf { first_id } else { second_id };
    let deceased = if first_on_bf { second_id } else { first_id };
    assert!(game.battlefield.contains_key(&survivor));
    assert_eq!(game.get_object(deceased).unwrap().zone, Zone::Graveyard);
}
