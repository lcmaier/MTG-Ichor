//! Phase 2 Integration Tests: Stack, Casting, Spell Resolution, One-Shot Effects
//!
//! Tests here exercise the full casting → stack → resolution pipeline,
//! including priority passing, fizzling, and all five Phase 2 cards.

mod common;

use mtgsim::cards::alpha;
use mtgsim::cards::basic_lands;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::types::effects::{EffectRecipient, PermanentFilter, SelectionFilter, TargetCount};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

use common::{setup_two_player_game, put_in_hand, put_land_on_battlefield, fill_library};

// ---------------------------------------------------------------------------
// Test 1: Cast and resolve Lightning Bolt targeting a player
// ---------------------------------------------------------------------------

#[test]
fn test_cast_and_resolve_lightning_bolt() {
    let mut game = setup_two_player_game();
    let bolt_id = put_in_hand(&mut game, alpha::lightning_bolt(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let decisions = ScriptedDecisionProvider::new();
    // CastSpell(bolt_id) at index 1 in [Pass, CastSpell(bolt_id)]
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // Target Player(1) at index 1 in [Player(0), Player(1)] for Any
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
        spell_id: bolt_id,
    }, vec![1]);
    // Round 1: Cast (returns ActionTaken immediately)
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);
    assert!(game.stack.contains(&bolt_id));
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0);

    // Round 2: Both pass → resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
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
    // CastSpell(recall_id) at index 1 in [Pass, CastSpell(recall_id)]
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // Target Player(0) at index 0 in [Player(0), Player(1)] for Player
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
        spell_id: recall_id,
    }, vec![0]);
    // Cast (returns ActionTaken immediately)
    game.run_priority_round(&decisions).unwrap();
    assert!(game.stack.contains(&recall_id));

    // Resolve: both pass
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
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
    // CastSpell at index 1 in [Pass, CastSpell(bolt_id)]
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // Target Player(1) at index 1 in [Player(0), Player(1)] for Any
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
        spell_id: bolt_id,
    }, vec![1]);
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);
    assert!(game.stack.contains(&bolt_id));

    // Player 1 responds with Counterspell targeting the bolt
    // After ActionTaken, priority returns to caster (player 0) who passes
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    // Player 1 casts: CastSpell(cs_id) at index 1 in [Pass, CastSpell(cs_id)]
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // Target bolt at index 0 in [Object(bolt_id)] for Spell
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(SelectionFilter::Spell, TargetCount::Exactly(1)),
        spell_id: cs_id,
    }, vec![0]);

    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);
    assert!(game.stack.contains(&cs_id)); // counterspell on stack
    assert!(game.stack.contains(&bolt_id)); // bolt still on stack below it

    // Both pass — Counterspell resolves
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
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
    // CastSpell at index 1 in [Pass, CastSpell(upheaval_id)]
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // Target land at index 0 in [Object(target_land)] for Permanent(ByType(Land))
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(
            SelectionFilter::Permanent(PermanentFilter::ByType(mtgsim::types::card_types::CardType::Land)),
            TargetCount::Exactly(1),
        ),
        spell_id: upheaval_id,
    }, vec![0]);
    // {3}{R}: 3 generic from Red pool → [3]
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
        vec![3],
    );
    // Cast (returns ActionTaken immediately)
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);
    // Land is still on battlefield while upheaval is on the stack
    assert!(game.battlefield.contains_key(&target_land));

    // Resolve: both pass
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
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
    // CastSpell at index 1 in [Pass, CastSpell(burst_id)]
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    // Target land at index 0 in [Object(land_id)] for Permanent(All)
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(
            SelectionFilter::Permanent(PermanentFilter::All),
            TargetCount::Exactly(1),
        ),
        spell_id: burst_id,
    }, vec![0]);
    // Cast (returns ActionTaken immediately)
    game.run_priority_round(&decisions).unwrap();
    // Resolve: both pass
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
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

    let upheaval_recipient = EffectRecipient::Target(
        SelectionFilter::Permanent(PermanentFilter::ByType(mtgsim::types::card_types::CardType::Land)),
        TargetCount::Exactly(1),
    );

    // Cast first upheaval targeting the land
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: upheaval_recipient.clone(),
        spell_id: upheaval1_id,
    }, vec![0]);
    // {3}{R}: 3 generic from Red pool → [3]
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
        vec![3],
    );
    game.run_priority_round(&decisions).unwrap();

    // Cast second upheaval targeting the same land
    // After ActionTaken, priority returns to caster (player 0)
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: upheaval_recipient.clone(),
        spell_id: upheaval2_id,
    }, vec![0]);
    // {3}{R}: 3 generic from Red pool → [3]
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
        vec![3],
    );
    game.run_priority_round(&decisions).unwrap();

    // Stack: [upheaval1, upheaval2] — upheaval2 on top (LIFO)
    assert_eq!(game.stack.len(), 2);

    // Both pass — upheaval2 resolves, destroying the land
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::StackResolved);
    assert!(!game.battlefield.contains_key(&target_land));

    // Both pass again — upheaval1 resolves but should fizzle (target gone)
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
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
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(
            SelectionFilter::Permanent(PermanentFilter::All),
            TargetCount::Exactly(1),
        ),
        spell_id: burst_id,
    }, vec![0]);
    game.run_priority_round(&decisions).unwrap();

    // Player 1 responds with Volcanic Upheaval targeting the same land
    // After ActionTaken, priority returns to caster (player 0) who passes
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    // Player 1 casts: CastSpell(upheaval_id) at idx 1 in [Pass, CastSpell(upheaval_id)]
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(
            SelectionFilter::Permanent(PermanentFilter::ByType(mtgsim::types::card_types::CardType::Land)),
            TargetCount::Exactly(1),
        ),
        spell_id: upheaval_id,
    }, vec![0]);
    // {3}{R}: 3 generic from Red pool → [3]
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
        vec![3],
    );
    game.run_priority_round(&decisions).unwrap();

    // Stack: [Burst of Energy, Volcanic Upheaval] — Upheaval on top (LIFO)
    assert_eq!(game.stack.len(), 2);

    // Both pass — Upheaval resolves, destroying the land
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::StackResolved);
    assert!(!game.battlefield.contains_key(&target_land));
    assert_eq!(game.get_object(target_land).unwrap().zone, Zone::Graveyard);

    // Both pass — Burst of Energy resolves but fizzles (target gone)
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
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
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
        spell_id: bolt_id,
    }, vec![1]);
    game.run_priority_round(&decisions).unwrap();

    // Cast recall second (goes on top)
    // After ActionTaken, priority returns to caster (player 0)
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
        spell_id: recall_id,
    }, vec![1]);
    game.run_priority_round(&decisions).unwrap();

    assert_eq!(game.stack.len(), 2);

    // Resolve top: Recall draws 3 for player 1 (both pass)
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(&decisions).unwrap();
    assert_eq!(game.players[1].hand.len(), 3);
    assert_eq!(game.players[1].life_total, 20); // bolt hasn't resolved yet

    // Resolve next: Bolt deals 3 to player 1 (both pass)
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
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
    // Player 0 casts bolt
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
        spell_id: bolt_id,
    }, vec![1]);
    // Round 2: both pass → resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    // Round 3: both pass → phase ends
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);

    // run_priority_loop will: cast → pass pass → resolve → pass pass → phase ends
    game.run_priority_loop(&decisions).unwrap();

    assert_eq!(game.players[1].life_total, 17);
    assert!(game.stack.is_empty());
}
