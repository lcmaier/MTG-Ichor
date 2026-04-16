//! Pre-Phase 3 Integration Tests: Game lifecycle, loss handling, discard,
//! draw skip, cost validation, and rollback.

mod common;

use std::sync::Arc;

use mtgsim::cards::alpha;
use mtgsim::cards::basic_lands;
use mtgsim::objects::card_data::CardData;
use mtgsim::objects::object::GameObject;
use mtgsim::state::game::{Decklist, Game, GameResult};
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameState, PhaseType};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::types::effects::{EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

use common::{put_in_hand, put_land_on_battlefield};

// ---------------------------------------------------------------------------
// Phase-specific helpers
// ---------------------------------------------------------------------------

fn make_forest() -> Arc<CardData> {
    basic_lands::forest()
}

fn make_test_decklist(count: usize) -> Decklist {
    (0..count).map(|_| make_forest()).collect()
}

// ---------------------------------------------------------------------------
// Test 1: Full game lifecycle — setup, run turns, game doesn't crash
// ---------------------------------------------------------------------------

#[test]
fn test_game_lifecycle_two_turns() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![make_test_decklist(30), make_test_decklist(30)],
    ).unwrap();

    let decisions = ScriptedDecisionProvider::new();
    game.setup(&decisions).unwrap();

    // No creatures, so no DeclareAttackers TBA fires.
    // Skip first draw to avoid discard-to-hand-size noise.
    game.state.skip_first_draw = true;

    // Turn 1: all players pass all priority (16 passes for 8 steps × 2 players)
    decisions.queue_empty_turn_passes();
    game.run_turn(&decisions).unwrap();
    assert_eq!(game.state.turn_number, 2);
    assert_eq!(game.state.active_player, 1);

    // Turn 2: player 1 draws (hand=8), cleanup discards 1
    decisions.queue_empty_turn_passes();
    decisions.expect_pick_n(ChoiceKind::DiscardToHandSize, vec![0]);
    game.run_turn(&decisions).unwrap();
    assert_eq!(game.state.turn_number, 3);
    assert_eq!(game.state.active_player, 0);

    assert!(!game.is_over());
}

// ---------------------------------------------------------------------------
// Test 2: Game ends when a player is bolted to 0 life
// ---------------------------------------------------------------------------

#[test]
fn test_game_over_bolt_to_zero() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![make_test_decklist(30), make_test_decklist(30)],
    ).unwrap();

    let dp = ScriptedDecisionProvider::new();
    game.setup(&dp).unwrap();

    // Manually reduce player 1's life to 3, then bolt them
    game.state.players[1].life_total = 3;
    // Clear hand so CastSpell(bolt) is at index 1 (no PlayLand actions)
    game.state.players[0].hand.clear();
    let bolt_id = put_in_hand(&mut game.state, alpha::lightning_bolt(), 0);
    game.state.players[0].mana_pool.add(ManaType::Red, 1);
    game.state.phase = mtgsim::state::game_state::Phase::new(PhaseType::Precombat);
    game.state.active_player = 0;

    // Script: cast bolt targeting player 1, then pass for everything else
    let scripted = ScriptedDecisionProvider::new();
    // CastSpell at index 1 in [Pass, CastSpell(bolt_id)]
    scripted.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // Target Player(1) at index 1 in [Player(0), Player(1)] for SelectionFilter::Any
    scripted.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
        spell_id: bolt_id,
    }, vec![1]);
    // CastSpell returns ActionTaken immediately (no extra pass needed)
    // Both pass → resolve + SBA (player 1 at 0 life → loses)
    scripted.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    scripted.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    // After resolve + SBA, both pass → phase ends
    scripted.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    scripted.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);

    // Cast and resolve via priority loop
    game.state.run_priority_loop(&scripted).unwrap();

    // Player 1 at 0 life → SBA should flag them as lost
    assert_eq!(game.state.players[1].life_total, 0);
    assert!(game.state.player_lost[1]);

    // Game should detect the result
    let result = game.check_game_over();
    assert_eq!(result, Some(GameResult::Winner(0)));
}

// ---------------------------------------------------------------------------
// Test 3: SBA flags player loss on 0 life (direct state manipulation)
// ---------------------------------------------------------------------------

#[test]
fn test_sba_flags_player_loss_zero_life() {
    let mut game = GameState::new(2, 20);
    game.players[0].life_total = 0;

    let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
    assert!(performed);
    assert!(game.player_lost[0]);
    assert!(!game.player_lost[1]);
}

// ---------------------------------------------------------------------------
// Test 4: SBA flags player loss on empty library draw
// ---------------------------------------------------------------------------

#[test]
fn test_sba_flags_player_loss_empty_library() {
    let mut game = GameState::new(2, 20);
    game.players[1].has_drawn_from_empty_library = true;

    let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
    assert!(performed);
    assert!(!game.player_lost[0]);
    assert!(game.player_lost[1]);
}

// ---------------------------------------------------------------------------
// Test 5: Both players lose simultaneously = Draw
// ---------------------------------------------------------------------------

#[test]
fn test_both_players_lose_is_draw() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![make_test_decklist(20), make_test_decklist(20)],
    ).unwrap();

    game.state.player_lost[0] = true;
    game.state.player_lost[1] = true;

    assert_eq!(game.check_game_over(), Some(GameResult::Draw));
}

// ---------------------------------------------------------------------------
// Test 6: Discard to hand size during cleanup
// ---------------------------------------------------------------------------

#[test]
fn test_discard_to_hand_size() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![make_test_decklist(30), make_test_decklist(30)],
    ).unwrap();

    let decisions = ScriptedDecisionProvider::new();
    game.setup(&decisions).unwrap();

    // Player 0 has 7 cards after setup. Give them 3 more to force discard.
    for _ in 0..3 {
        let card = make_forest();
        let obj = GameObject::new(card, 0, Zone::Hand);
        let id = obj.id;
        game.state.add_object(obj);
        game.state.players[0].hand.push(id);
    }
    assert_eq!(game.state.players[0].hand.len(), 10);

    // Run a turn — cleanup step should discard down to 7.
    // No creatures → no DeclareAttackers TBA. Skip first draw would remove
    // the draw, but this test IS about discard so we keep the draw.
    // 10 cards in hand + 1 draw = 11 → discard 4 to reach max_hand_size 7.
    // Discard happens one card at a time (pick_n with 1 pick per call).
    decisions.queue_empty_turn_passes();
    for _ in 0..4 {
        decisions.expect_pick_n(ChoiceKind::DiscardToHandSize, vec![0]);
    }
    game.run_turn(&decisions).unwrap();

    // After turn completes, player 0 should have max_hand_size cards
    assert_eq!(game.state.players[0].hand.len(), 7);
}

// ---------------------------------------------------------------------------
// Test 7: First-player draw skip (standard config)
// ---------------------------------------------------------------------------

#[test]
fn test_first_player_draw_skip() {
    let config = GameConfig::standard();
    let mut game = Game::new(
        config,
        vec![make_test_decklist(60), make_test_decklist(60)],
    ).unwrap();

    let decisions = ScriptedDecisionProvider::new();
    game.setup(&decisions).unwrap();

    // Both players drew 7 cards during setup
    let hand_after_setup_p0 = game.state.players[0].hand.len();
    let lib_after_setup_p0 = game.state.players[0].library.len();
    assert_eq!(hand_after_setup_p0, 7);
    assert_eq!(lib_after_setup_p0, 53);

    // skip_first_draw should be set
    assert!(game.state.skip_first_draw);

    // Run turn 1 for player 0 — draw step should be SKIPPED.
    // No creatures → no DeclareAttackers TBA.
    // skip_first_draw is already true (standard config), so draw is skipped
    // and hand stays at 7 (no discard needed).
    decisions.queue_empty_turn_passes();
    game.run_turn(&decisions).unwrap();

    // Player 0 did NOT draw → hand should still be 7
    // (the draw was skipped, and cleanup discards to 7 anyway)
    assert_eq!(game.state.players[0].hand.len(), 7);
    // Library should be the same as after setup (no card drawn)
    assert_eq!(game.state.players[0].library.len(), 53);

    // skip_first_draw should now be false
    assert!(!game.state.skip_first_draw);
}

// ---------------------------------------------------------------------------
// Test 8: can_pay_costs pre-check prevents stranded cards on stack
// ---------------------------------------------------------------------------

#[test]
fn test_cast_spell_rollback_on_insufficient_mana() {
    let mut game = GameState::new(2, 20);
    game.phase = mtgsim::state::game_state::Phase::new(PhaseType::Precombat);
    game.active_player = 0;

    // Give player 0 a bolt in hand but NO mana
    let bolt_id = put_in_hand(&mut game, alpha::lightning_bolt(), 0);

    let decisions = ScriptedDecisionProvider::new();
    // Target Player(1) at index 1 in [Player(0), Player(1)] for SelectionFilter::Any
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
        spell_id: bolt_id,
    }, vec![1]);

    // Casting should fail
    let result = game.cast_spell(0, bolt_id, &decisions);
    assert!(result.is_err());

    // Card should be back in hand, NOT stranded on the stack
    assert_eq!(game.get_object(bolt_id).unwrap().zone, Zone::Hand);
    assert!(game.players[0].hand.contains(&bolt_id));
    assert!(game.stack.is_empty());
    assert!(game.stack_entries.is_empty());
}

// ---------------------------------------------------------------------------
// Test 9: can_pay_costs read-only validation
// ---------------------------------------------------------------------------

#[test]
fn test_can_pay_costs_validates_correctly() {
    use mtgsim::types::costs::Cost;
    use mtgsim::types::mana::ManaCost;

    let mut game = GameState::new(2, 20);

    // Set up a forest on battlefield
    let land_id = put_land_on_battlefield(&mut game, basic_lands::forest, 0);

    // Tap cost should be payable (untapped land)
    assert!(game.can_pay_costs(&[Cost::Tap], 0, land_id).is_ok());

    // Tap the land manually
    game.battlefield.get_mut(&land_id).unwrap().tapped = true;
    assert!(game.can_pay_costs(&[Cost::Tap], 0, land_id).is_err());

    // Mana cost check
    game.players[0].mana_pool.add(ManaType::Red, 1);
    let red_cost = Cost::Mana(ManaCost::build(&[ManaType::Red], 0));
    assert!(game.can_pay_costs(&[red_cost.clone()], 0, land_id).is_ok());

    let two_red = Cost::Mana(ManaCost::build(&[ManaType::Red, ManaType::Red], 0));
    assert!(game.can_pay_costs(&[two_red], 0, land_id).is_err());

    // Life cost check
    assert!(game.can_pay_costs(&[Cost::PayLife(20)], 0, land_id).is_ok());
    assert!(game.can_pay_costs(&[Cost::PayLife(21)], 0, land_id).is_err());
}

// ---------------------------------------------------------------------------
// Test 10: CounterSpell cleans up stack_entries
// ---------------------------------------------------------------------------

#[test]
fn test_counterspell_cleans_up_stack_entries() {
    let mut game = GameState::new(2, 20);
    game.phase = mtgsim::state::game_state::Phase::new(PhaseType::Precombat);
    game.active_player = 0;

    // Player 0 casts bolt
    let bolt_id = put_in_hand(&mut game, alpha::lightning_bolt(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let decisions = ScriptedDecisionProvider::new();
    // Bolt target: Player(1) at index 1 in [Player(0), Player(1)] for SelectionFilter::Any
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
        spell_id: bolt_id,
    }, vec![1]);
    // Counterspell target: Object(bolt_id) at index 0 in [Object(bolt_id)] for SelectionFilter::Spell
    let cs_id = put_in_hand(&mut game, alpha::counterspell(), 1);
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(SelectionFilter::Spell, TargetCount::Exactly(1)),
        spell_id: cs_id,
    }, vec![0]);
    game.cast_spell(0, bolt_id, &decisions).unwrap();
    assert!(game.stack_entries.contains_key(&bolt_id));

    // Player 1 casts counterspell targeting bolt
    game.players[1].mana_pool.add(ManaType::Blue, 2);
    game.cast_spell(1, cs_id, &decisions).unwrap();

    // Resolve counterspell (top of stack)
    game.resolve_top_of_stack(&ScriptedDecisionProvider::new()).unwrap();

    // Bolt's stack entry should be cleaned up
    assert!(!game.stack_entries.contains_key(&bolt_id));
    // Bolt should be in graveyard
    assert_eq!(game.get_object(bolt_id).unwrap().zone, Zone::Graveyard);
    // Stack should have just the counterspell's post-resolution state (empty after CS resolves)
    // Actually counterspell itself was resolved and removed too
    assert!(!game.stack.contains(&bolt_id));
}
