//! Phase 2 Integration Tests: Stack, Casting, Spell Resolution, One-Shot Effects
//!
//! Tests here exercise the full casting → stack → resolution pipeline,
//! including priority passing, fizzling, and all five Phase 2 cards.

use std::sync::Arc;

use mtgsim::cards::alpha;
use mtgsim::cards::basic_lands;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::engine::resolve::ResolvedTarget;
use mtgsim::objects::card_data::CardData;
use mtgsim::objects::object::GameObject;
use mtgsim::state::battlefield::BattlefieldEntity;
use mtgsim::state::game_state::{GameState, PhaseType};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::decision::{PriorityAction, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup_two_player_game() -> GameState {
    let mut game = GameState::new(2, 20);
    game.phase = mtgsim::state::game_state::Phase::new(PhaseType::Precombat);
    game.active_player = 0;
    game
}

/// Put a card into a player's hand and register it in the game.
fn put_in_hand(game: &mut GameState, card_data: Arc<CardData>, player: usize) -> ObjectId {
    let obj = GameObject::new(card_data, player, Zone::Hand);
    let id = obj.id;
    game.add_object(obj);
    game.players[player].hand.push(id);
    id
}

/// Put a land onto the battlefield for a player.
fn put_land_on_battlefield(
    game: &mut GameState,
    land_fn: fn() -> Arc<CardData>,
    player: usize,
) -> ObjectId {
    let card_data = land_fn();
    let obj = GameObject::new(card_data, player, Zone::Battlefield);
    let id = obj.id;
    let ts = game.allocate_timestamp();
    game.add_object(obj);
    let entry = BattlefieldEntity::new(id, player, ts, 0);
    game.battlefield.insert(id, entry);
    id
}

/// Give a player some cards in their library (for draw effects).
fn fill_library(game: &mut GameState, player: usize, count: usize) {
    for _ in 0..count {
        let card = mtgsim::objects::card_data::CardDataBuilder::new("Dummy Card").build();
        let obj = GameObject::new(card, player, Zone::Library);
        let id = obj.id;
        game.add_object(obj);
        game.players[player].library.push(id);
    }
}

// ---------------------------------------------------------------------------
// Test 1: Cast and resolve Lightning Bolt targeting a player
// ---------------------------------------------------------------------------

#[test]
fn test_cast_and_resolve_lightning_bolt() {
    let mut game = setup_two_player_game();
    let bolt_id = put_in_hand(&mut game, alpha::lightning_bolt(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let decisions = ScriptedDecisionProvider::new();
    // Player 0 casts bolt targeting player 1, then both pass
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(bolt_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Player(1)]);

    // Round 1: Cast
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);
    assert!(game.stack.contains(&bolt_id));
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0);

    // Round 2: Both pass → resolve
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::StackResolved);
    assert_eq!(game.players[1].life_total, 17);
    assert!(game.stack.is_empty());

    // Bolt in graveyard
    assert_eq!(game.get_object(bolt_id).unwrap().zone, Zone::Graveyard);
    assert!(game.players[0].graveyard.contains(&bolt_id));
}

// ---------------------------------------------------------------------------
// Test 2: Cast and resolve Ancestral Recall
// ---------------------------------------------------------------------------

#[test]
fn test_cast_and_resolve_ancestral_recall() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 10);
    let recall_id = put_in_hand(&mut game, alpha::ancestral_recall(), 0);
    game.players[0].mana_pool.add(ManaType::Blue, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(recall_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Player(0)]);

    // Cast
    game.run_priority_round(&decisions).unwrap();
    assert!(game.stack.contains(&recall_id));

    // Resolve
    game.run_priority_round(&decisions).unwrap();
    assert_eq!(game.players[0].hand.len(), 3); // drew 3 cards
    assert_eq!(game.players[0].library.len(), 7);
}

// ---------------------------------------------------------------------------
// Test 3: Counterspell counters a spell on the stack
// ---------------------------------------------------------------------------

#[test]
fn test_counterspell_counters_bolt() {
    let mut game = setup_two_player_game();
    let bolt_id = put_in_hand(&mut game, alpha::lightning_bolt(), 0);
    let cs_id = put_in_hand(&mut game, alpha::counterspell(), 1);
    game.players[0].mana_pool.add(ManaType::Red, 1);
    game.players[1].mana_pool.add(ManaType::Blue, 2);

    let decisions = ScriptedDecisionProvider::new();

    // Player 0 casts bolt targeting player 1
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(bolt_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Player(1)]);

    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);
    assert!(game.stack.contains(&bolt_id));

    // Player 1 responds with Counterspell targeting the bolt
    // In priority, player 0 passes, then player 1 casts counterspell
    decisions.priority_decisions.borrow_mut().push(PriorityAction::Pass); // player 0 passes
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(cs_id)); // player 1 casts
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(bolt_id)]);

    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);
    assert!(game.stack.contains(&cs_id)); // counterspell on stack
    assert!(game.stack.contains(&bolt_id)); // bolt still on stack below it

    // Both pass — Counterspell resolves (it's on top), countering the bolt
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::StackResolved);

    // Bolt should have been countered (removed from stack, in graveyard)
    assert!(!game.stack.contains(&bolt_id));
    assert_eq!(game.get_object(bolt_id).unwrap().zone, Zone::Graveyard);

    // Counterspell itself should be in graveyard too
    assert_eq!(game.get_object(cs_id).unwrap().zone, Zone::Graveyard);

    // Player 1's life should be unchanged — bolt was countered
    assert_eq!(game.players[1].life_total, 20);
}

// ---------------------------------------------------------------------------
// Test 4: Volcanic Upheaval destroys a land
// ---------------------------------------------------------------------------

#[test]
fn test_volcanic_upheaval_destroys_land() {
    let mut game = setup_two_player_game();
    let target_land = put_land_on_battlefield(&mut game, basic_lands::forest, 1);
    let upheaval_id = put_in_hand(&mut game, alpha::volcanic_upheaval(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 4); // {3}{R}

    let decisions = ScriptedDecisionProvider::new();
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(upheaval_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(target_land)]);

    // Cast
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);
    // Land is still on battlefield while upheaval is on the stack
    assert!(game.battlefield.contains_key(&target_land));

    // Resolve
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::StackResolved);

    // Land should be destroyed (in graveyard)
    assert!(!game.battlefield.contains_key(&target_land));
    assert_eq!(game.get_object(target_land).unwrap().zone, Zone::Graveyard);
    assert!(game.players[1].graveyard.contains(&target_land));
}

// ---------------------------------------------------------------------------
// Test 5: Burst of Energy untaps a tapped permanent
// ---------------------------------------------------------------------------

#[test]
fn test_burst_of_energy_untaps_land() {
    let mut game = setup_two_player_game();
    let land_id = put_land_on_battlefield(&mut game, basic_lands::plains, 0);
    // Tap the land
    game.battlefield.get_mut(&land_id).unwrap().tapped = true;
    assert!(game.battlefield.get(&land_id).unwrap().tapped);

    let burst_id = put_in_hand(&mut game, alpha::burst_of_energy(), 0);
    game.players[0].mana_pool.add(ManaType::White, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(burst_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(land_id)]);

    // Cast
    game.run_priority_round(&decisions).unwrap();
    // Resolve
    game.run_priority_round(&decisions).unwrap();

    // Land should be untapped
    assert!(!game.battlefield.get(&land_id).unwrap().tapped);
}

// ---------------------------------------------------------------------------
// Test 6: Volcanic Upheaval fizzles when target land is destroyed first
// ---------------------------------------------------------------------------

#[test]
fn test_volcanic_upheaval_fizzles_when_target_destroyed() {
    let mut game = setup_two_player_game();
    let target_land = put_land_on_battlefield(&mut game, basic_lands::forest, 1);

    // Player 0 has two Volcanic Upheavals
    let upheaval1_id = put_in_hand(&mut game, alpha::volcanic_upheaval(), 0);
    let upheaval2_id = put_in_hand(&mut game, alpha::volcanic_upheaval(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 8); // enough for both

    let decisions = ScriptedDecisionProvider::new();

    // Cast first upheaval targeting the land
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(upheaval1_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(target_land)]);
    game.run_priority_round(&decisions).unwrap();

    // Cast second upheaval targeting the same land
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(upheaval2_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(target_land)]);
    game.run_priority_round(&decisions).unwrap();

    // Stack: [upheaval1, upheaval2] — upheaval2 on top (LIFO)
    assert_eq!(game.stack.len(), 2);

    // Both pass — upheaval2 resolves, destroying the land
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::StackResolved);
    assert!(!game.battlefield.contains_key(&target_land));

    // Both pass again — upheaval1 resolves but should fizzle (target gone)
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::StackResolved);

    // upheaval1 fizzled but still goes to graveyard
    assert_eq!(game.get_object(upheaval1_id).unwrap().zone, Zone::Graveyard);
    // Stack should be empty
    assert!(game.stack.is_empty());
}

// ---------------------------------------------------------------------------
// Test 6b: Burst of Energy fizzles when Volcanic Upheaval destroys its target
// ---------------------------------------------------------------------------

#[test]
fn test_burst_of_energy_fizzles_after_upheaval_destroys_target() {
    let mut game = setup_two_player_game();
    let target_land = put_land_on_battlefield(&mut game, basic_lands::forest, 0);
    // Tap the land so Burst of Energy has a meaningful target
    game.battlefield.get_mut(&target_land).unwrap().tapped = true;

    let upheaval_id = put_in_hand(&mut game, alpha::volcanic_upheaval(), 1);
    let burst_id = put_in_hand(&mut game, alpha::burst_of_energy(), 0);
    game.players[1].mana_pool.add(ManaType::Red, 4); // {3}{R} for Upheaval
    game.players[0].mana_pool.add(ManaType::White, 1); // {W} for Burst

    let decisions = ScriptedDecisionProvider::new();

    // Player 0 casts Burst of Energy targeting their own tapped land
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(burst_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(target_land)]);
    game.run_priority_round(&decisions).unwrap();

    // Player 1 responds with Volcanic Upheaval targeting the same land
    decisions.priority_decisions.borrow_mut().push(PriorityAction::Pass); // player 0 passes
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(upheaval_id)); // player 1 casts
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(target_land)]);
    game.run_priority_round(&decisions).unwrap();

    // Stack: [Burst of Energy, Volcanic Upheaval] — Upheaval on top (LIFO)
    assert_eq!(game.stack.len(), 2);

    // Both pass — Upheaval resolves, destroying the land
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::StackResolved);
    assert!(!game.battlefield.contains_key(&target_land));
    assert_eq!(game.get_object(target_land).unwrap().zone, Zone::Graveyard);

    // Both pass — Burst of Energy resolves but fizzles (target gone)
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::StackResolved);

    // Burst fizzled — it's in the graveyard, stack is empty
    assert_eq!(game.get_object(burst_id).unwrap().zone, Zone::Graveyard);
    assert!(game.stack.is_empty());
}

// ---------------------------------------------------------------------------
// Test 7: Stack ordering — LIFO resolution
// ---------------------------------------------------------------------------

#[test]
fn test_stack_lifo_order_bolt_then_recall() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 1, 10);

    let bolt_id = put_in_hand(&mut game, alpha::lightning_bolt(), 0);
    let recall_id = put_in_hand(&mut game, alpha::ancestral_recall(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);
    game.players[0].mana_pool.add(ManaType::Blue, 1);

    let decisions = ScriptedDecisionProvider::new();

    // Cast bolt first (goes on stack first = bottom)
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(bolt_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Player(1)]);
    game.run_priority_round(&decisions).unwrap();

    // Cast recall second (goes on top)
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(recall_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Player(1)]);
    game.run_priority_round(&decisions).unwrap();

    assert_eq!(game.stack.len(), 2);

    // Resolve top: Recall draws 3 for player 1
    game.run_priority_round(&decisions).unwrap();
    assert_eq!(game.players[1].hand.len(), 3);
    assert_eq!(game.players[1].life_total, 20); // bolt hasn't resolved yet

    // Resolve next: Bolt deals 3 to player 1
    game.run_priority_round(&decisions).unwrap();
    assert_eq!(game.players[1].life_total, 17);
}

// ---------------------------------------------------------------------------
// Test 8: Registry creates all Phase 2 cards
// ---------------------------------------------------------------------------

#[test]
fn test_registry_has_phase2_cards() {
    let registry = CardRegistry::default_registry();
    for name in &[
        "Lightning Bolt",
        "Ancestral Recall",
        "Counterspell",
        "Burst of Energy",
        "Volcanic Upheaval",
    ] {
        let card = registry.create(name);
        assert!(card.is_ok(), "Card '{}' should be in registry", name);
    }
}

// ---------------------------------------------------------------------------
// Test 9: Full priority loop with single spell
// ---------------------------------------------------------------------------

#[test]
fn test_priority_loop_cast_and_resolve() {
    let mut game = setup_two_player_game();
    let bolt_id = put_in_hand(&mut game, alpha::lightning_bolt(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let decisions = ScriptedDecisionProvider::new();
    // Player 0 casts, then all pass for the rest
    decisions.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(bolt_id));
    decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Player(1)]);

    // run_priority_loop will: cast → pass pass → resolve → pass pass → phase ends
    game.run_priority_loop(&decisions).unwrap();

    assert_eq!(game.players[1].life_total, 17);
    assert!(game.stack.is_empty());
}
