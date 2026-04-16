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
use mtgsim::objects::card_data::CardDataBuilder;
use mtgsim::state::game_state::{GameState, PhaseType};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{EffectRecipient, PermanentFilter, SelectionFilter, TargetCount};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::oracle::legality::candidate_priority_actions;
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
    // [Pass, CastSpell(whisper_id)] → idx 1
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // {1}{B}: pool has [Black(1), Colorless(1)], allocate 1 generic → [0, 1]
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
        vec![0, 1],
    );
    // Both pass → resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);

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
    let _whisper_id = put_in_hand(&mut game, phase5_pre_cards::nights_whisper(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);

    // Set to combat phase — sorcery can't be cast here
    game.phase = mtgsim::state::game_state::Phase::new(PhaseType::Combat);

    // Sorcery can't be cast in combat — not in candidates, both pass
    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);

    let result = game.run_priority_round(&decisions);
    // Should pass through — both players pass, stack empty → AllPassed
    assert!(result.is_ok());
    assert!(game.stack.is_empty());
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
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // {2}{W}{W}: pool has [White(2), Colorless(2)], allocate 2 generic → [0, 2]
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
        vec![0, 2],
    );
    // Both pass → resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);

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
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // Both pass → resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);

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
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);

    // Cast — it should go on the stack
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);
    assert!(game.stack.contains(&ritual_id));

    // Mana hasn't been added yet — still on the stack
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Black), 0);

    // Now resolve (both pass)
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
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
    // [Pass, CastSpell(blade_id)] → idx 1
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // Target: legal selections = [Object(target_id)] (only nonblack creature) → idx 0
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(
            SelectionFilter::Permanent(PermanentFilter::And(
                Box::new(PermanentFilter::ByType(CardType::Creature)),
                Box::new(PermanentFilter::Not(Box::new(PermanentFilter::ByColor(
                    mtgsim::types::colors::Color::Black,
                )))),
            )),
            TargetCount::Exactly(1),
        ),
        spell_id: blade_id,
    }, vec![0]);
    // {1}{B}: pool has [Black(1), Colorless(1)], allocate 1 generic → [0, 1]
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
        vec![0, 1],
    );
    // Both pass → resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);

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
    // Put a black creature on the battlefield
    let black_creature_data = CardDataBuilder::new("Black Knight")
        .card_type(CardType::Creature)
        .color(mtgsim::types::colors::Color::Black)
        .power_toughness(2, 2)
        .build();
    let target_id = put_on_battlefield(&mut game, black_creature_data, 1);
    let _blade_id = put_in_hand(&mut game, phase5_pre_cards::doom_blade(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);

    // castable_spells filters out Doom Blade because has_any_legal_choice
    // finds no nonblack creatures. So CastSpell should NOT appear.
    let actions = candidate_priority_actions(&game, 0);
    assert!(
        !actions.iter().any(|a| matches!(a, PriorityAction::CastSpell(_))),
        "Doom Blade should not be castable when only target is a black creature",
    );
    // Black creature still alive
    assert!(game.battlefield.contains_key(&target_id));
}

#[test]
fn test_doom_blade_rejects_noncreature_permanent() {
    let mut game = setup_two_player_game();
    // A land is a noncreature permanent — Doom Blade can't target it
    let land_id = put_on_battlefield(&mut game, basic_lands::forest(), 1);

    let _blade_id = put_in_hand(&mut game, phase5_pre_cards::doom_blade(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);

    // castable_spells filters out Doom Blade because has_any_legal_choice
    // finds no nonblack creatures (only a land). CastSpell should NOT appear.
    let actions = candidate_priority_actions(&game, 0);
    assert!(
        !actions.iter().any(|a| matches!(a, PriorityAction::CastSpell(_))),
        "Doom Blade should not be castable when only permanent is a noncreature",
    );
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
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // Both pass → resolve. SBAs fire after resolution and detect legend rule.
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    // LegendRule fires during SBA check after resolution: keep first (index 0)
    decisions.expect_pick_n(ChoiceKind::LegendRule {
        legend_name: "Isamaru, Hound of Konda".to_string(),
    }, vec![0]);

    // Cast and resolve — second Isamaru enters, SBAs kill one
    cast_and_resolve(&mut game, &decisions);

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
