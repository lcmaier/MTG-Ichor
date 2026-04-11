//! Phase 3 integration tests — creatures and combat.
//!
//! Tests the full pipeline: creature spell resolution, declare attackers,
//! declare blockers, combat damage, SBA lethal damage, and game-over via
//! combat damage.

use std::sync::Arc;

use mtgsim::cards::alpha;
use mtgsim::cards::creatures;
use mtgsim::cards::basic_lands;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::resolve::ResolvedTarget;
use mtgsim::objects::card_data::CardData;
use mtgsim::objects::object::GameObject;
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::game::{Decklist, Game, GameResult};
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{PhaseType, StepType};
use mtgsim::types::card_types::CardType;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::decision::{PassiveDecisionProvider, PriorityAction, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_test_deck(creatures: Vec<fn() -> Arc<CardData>>, lands: usize) -> Decklist {
    let mut deck: Decklist = Vec::new();
    for factory in creatures {
        deck.push(factory());
    }
    for _ in 0..lands {
        deck.push(basic_lands::forest());
    }
    deck
}

/// Advance a GameState through steps until we reach a specific phase/step.
fn advance_to_step(game: &mut Game, target_phase: PhaseType, target_step: Option<StepType>) {
    for _ in 0..200 {
        let phase = game.state.phase.phase_type;
        let step = game.state.phase.step;
        if phase == target_phase && step == target_step {
            return;
        }
        game.state.advance_turn().unwrap();
    }
    panic!("Failed to reach {:?}/{:?}", target_phase, target_step);
}

/// Place a creature directly on the battlefield for a player (bypassing casting).
/// Returns its ObjectId. The creature is NOT summoning-sick.
fn place_creature_on_battlefield(
    game: &mut Game,
    owner: PlayerId,
    card_factory: fn() -> Arc<CardData>,
) -> ObjectId {
    let data = card_factory();
    let obj = mtgsim::objects::object::GameObject::new(data, owner, Zone::Battlefield);
    let id = obj.id;
    game.state.add_object(obj);
    let ts = game.state.allocate_timestamp();
    let entry = mtgsim::state::battlefield::BattlefieldEntity::new(id, owner, ts, 0);
    game.state.battlefield.insert(id, entry);
    id
}

/// Put a card into a player's hand (for casting spells like Lightning Bolt).
fn put_in_hand(game: &mut Game, card_data: Arc<CardData>, player: PlayerId) -> ObjectId {
    let obj = GameObject::new(card_data, player, Zone::Hand);
    let id = obj.id;
    game.state.add_object(obj);
    game.state.players[player].hand.push(id);
    id
}

// ---------------------------------------------------------------------------
// Test: Registry includes Phase 3 creatures
// ---------------------------------------------------------------------------

#[test]
fn test_registry_has_phase3_creatures() {
    let registry = CardRegistry::default_registry();
    for name in &["Grizzly Bears", "Hill Giant", "Savannah Lions"] {
        let card = registry.create(name).unwrap();
        assert_eq!(card.name, *name);
        assert!(card.types.contains(&CardType::Creature));
        assert!(card.power.is_some());
        assert!(card.toughness.is_some());
    }
}

// ---------------------------------------------------------------------------
// Test: Unblocked attacker deals damage to defending player
// ---------------------------------------------------------------------------

#[test]
fn test_unblocked_attacker_deals_damage() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![
            make_test_deck(vec![], 20),
            make_test_deck(vec![], 20),
        ],
    ).unwrap();
    let passive = PassiveDecisionProvider;
    game.setup(&passive).unwrap();

    // Place a 2/2 creature on the battlefield for player 0
    let bears_id = place_creature_on_battlefield(&mut game, 0, creatures::grizzly_bears);

    // Advance to DeclareAttackers step
    advance_to_step(&mut game, PhaseType::Combat, Some(StepType::DeclareAttackers));

    // Now run through the combat phase with a scripted decision provider
    // that attacks with the bears
    let scripted = ScriptedDecisionProvider::new();
    scripted.attack_decisions.borrow_mut().push(
        vec![(bears_id, AttackTarget::Player(1))]
    );
    // No blocks (player 1 has no creatures)
    scripted.block_decisions.borrow_mut().push(vec![]);
    // Priority: all pass
    for _ in 0..20 {
        scripted.priority_decisions.borrow_mut().push(PriorityAction::Pass);
    }

    // Process declare attackers
    game.state.process_declare_attackers(&scripted).unwrap();

    // Verify attacker is tapped and attacking
    assert!(game.state.battlefield.get(&bears_id).unwrap().tapped);
    assert!(game.state.battlefield.get(&bears_id).unwrap().attacking.is_some());

    // Advance to DeclareBlockers
    game.state.advance_turn().unwrap();
    assert_eq!(game.state.phase.step, Some(StepType::DeclareBlockers));
    game.state.process_declare_blockers(&scripted).unwrap();

    // Advance to FirstStrikeDamage (no-op in Phase 3)
    game.state.advance_turn().unwrap();
    assert_eq!(game.state.phase.step, Some(StepType::FirstStrikeDamage));
    game.state.process_combat_damage(&scripted, true).unwrap();

    // Advance to CombatDamage
    game.state.advance_turn().unwrap();
    assert_eq!(game.state.phase.step, Some(StepType::CombatDamage));
    game.state.process_combat_damage(&scripted, false).unwrap();

    // Player 1 should have taken 2 damage (bears are 2/2)
    assert_eq!(game.state.players[1].life_total, 18);
}

// ---------------------------------------------------------------------------
// Test: Blocked creature — both die from lethal damage (SBA)
// ---------------------------------------------------------------------------

#[test]
fn test_blocked_creatures_trade() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![
            make_test_deck(vec![], 20),
            make_test_deck(vec![], 20),
        ],
    ).unwrap();
    let passive = PassiveDecisionProvider;
    game.setup(&passive).unwrap();

    // Player 0: 2/2 attacker
    let attacker = place_creature_on_battlefield(&mut game, 0, creatures::grizzly_bears);
    // Player 1: 2/2 blocker
    let blocker = place_creature_on_battlefield(&mut game, 1, creatures::grizzly_bears);

    advance_to_step(&mut game, PhaseType::Combat, Some(StepType::DeclareAttackers));

    let scripted = ScriptedDecisionProvider::new();
    scripted.attack_decisions.borrow_mut().push(
        vec![(attacker, AttackTarget::Player(1))]
    );
    scripted.block_decisions.borrow_mut().push(
        vec![(blocker, attacker)]
    );

    // Declare attackers
    game.state.process_declare_attackers(&scripted).unwrap();

    // Declare blockers
    game.state.advance_turn().unwrap();
    game.state.process_declare_blockers(&scripted).unwrap();

    // Verify attacker is marked as blocked
    let att_info = game.state.battlefield.get(&attacker).unwrap().attacking.as_ref().unwrap();
    assert!(att_info.is_blocked);
    assert_eq!(att_info.blocked_by, vec![blocker]);

    // First strike damage (no-op)
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, true).unwrap();

    // Combat damage
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, false).unwrap();

    // Both should have 2 damage marked (lethal for 2-toughness creatures)
    assert_eq!(game.state.battlefield.get(&attacker).unwrap().damage_marked, 2);
    assert_eq!(game.state.battlefield.get(&blocker).unwrap().damage_marked, 2);

    // Run priority loop — SBAs fire automatically (rule 117.5), killing both
    for _ in 0..10 {
        scripted.priority_decisions.borrow_mut().push(PriorityAction::Pass);
    }
    game.state.run_priority_loop(&scripted).unwrap();

    // Both creatures should be in graveyard, not on battlefield
    assert!(!game.state.battlefield.contains_key(&attacker));
    assert!(!game.state.battlefield.contains_key(&blocker));

    // No player life lost (damage was to creatures)
    assert_eq!(game.state.players[0].life_total, 20);
    assert_eq!(game.state.players[1].life_total, 20);
}

// ---------------------------------------------------------------------------
// Test: Bigger creature survives combat
// ---------------------------------------------------------------------------

#[test]
fn test_bigger_creature_survives() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![
            make_test_deck(vec![], 20),
            make_test_deck(vec![], 20),
        ],
    ).unwrap();
    let passive = PassiveDecisionProvider;
    game.setup(&passive).unwrap();

    // Player 0: 3/3 attacker (Hill Giant)
    let attacker = place_creature_on_battlefield(&mut game, 0, creatures::hill_giant);
    // Player 1: 2/2 blocker (Grizzly Bears)
    let blocker = place_creature_on_battlefield(&mut game, 1, creatures::grizzly_bears);

    advance_to_step(&mut game, PhaseType::Combat, Some(StepType::DeclareAttackers));

    let scripted = ScriptedDecisionProvider::new();
    scripted.attack_decisions.borrow_mut().push(
        vec![(attacker, AttackTarget::Player(1))]
    );
    scripted.block_decisions.borrow_mut().push(
        vec![(blocker, attacker)]
    );

    game.state.process_declare_attackers(&scripted).unwrap();
    game.state.advance_turn().unwrap();
    game.state.process_declare_blockers(&scripted).unwrap();
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, true).unwrap();
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, false).unwrap();

    // Hill Giant takes 2 damage (not lethal for 3 toughness)
    assert_eq!(game.state.battlefield.get(&attacker).unwrap().damage_marked, 2);
    // Bears take 3 damage (lethal for 2 toughness)
    assert_eq!(game.state.battlefield.get(&blocker).unwrap().damage_marked, 3);

    // Run priority loop — SBAs fire, killing Bears but not Hill Giant
    for _ in 0..10 {
        scripted.priority_decisions.borrow_mut().push(PriorityAction::Pass);
    }
    game.state.run_priority_loop(&scripted).unwrap();

    // Bears die, Hill Giant survives
    assert!(game.state.battlefield.contains_key(&attacker));
    assert!(!game.state.battlefield.contains_key(&blocker));
}

// ---------------------------------------------------------------------------
// Test: Overkill damage — Bears (2/2) vs Lions (2/1), both die
// ---------------------------------------------------------------------------

#[test]
fn test_overkill_damage_both_die() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![
            make_test_deck(vec![], 20),
            make_test_deck(vec![], 20),
        ],
    ).unwrap();
    let passive = PassiveDecisionProvider;
    game.setup(&passive).unwrap();

    // Player 0: Grizzly Bears (2/2)
    let bears = place_creature_on_battlefield(&mut game, 0, creatures::grizzly_bears);
    // Player 1: Savannah Lions (2/1)
    let lions = place_creature_on_battlefield(&mut game, 1, creatures::savannah_lions);

    advance_to_step(&mut game, PhaseType::Combat, Some(StepType::DeclareAttackers));

    let scripted = ScriptedDecisionProvider::new();
    scripted.attack_decisions.borrow_mut().push(
        vec![(bears, AttackTarget::Player(1))]
    );
    scripted.block_decisions.borrow_mut().push(
        vec![(lions, bears)]
    );

    game.state.process_declare_attackers(&scripted).unwrap();
    game.state.advance_turn().unwrap();
    game.state.process_declare_blockers(&scripted).unwrap();
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, true).unwrap();
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, false).unwrap();

    // Bears take 2 damage (lethal for 2 toughness)
    assert_eq!(game.state.battlefield.get(&bears).unwrap().damage_marked, 2);
    // Lions take 2 damage — overkill for 1 toughness, but damage is still marked
    assert_eq!(game.state.battlefield.get(&lions).unwrap().damage_marked, 2);

    // Run priority loop — SBAs fire, both should die
    for _ in 0..10 {
        scripted.priority_decisions.borrow_mut().push(PriorityAction::Pass);
    }
    game.state.run_priority_loop(&scripted).unwrap();

    // Both creatures dead
    assert!(!game.state.battlefield.contains_key(&bears));
    assert!(!game.state.battlefield.contains_key(&lions));

    // No player life lost (all damage was creature-to-creature)
    assert_eq!(game.state.players[0].life_total, 20);
    assert_eq!(game.state.players[1].life_total, 20);
}

// ---------------------------------------------------------------------------
// Test: No attackers declared — no combat damage
// ---------------------------------------------------------------------------

#[test]
fn test_no_attackers_no_damage() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![
            make_test_deck(vec![], 20),
            make_test_deck(vec![], 20),
        ],
    ).unwrap();
    let passive = PassiveDecisionProvider;
    game.setup(&passive).unwrap();

    let _bears = place_creature_on_battlefield(&mut game, 0, creatures::grizzly_bears);

    // Run an entire turn with passive decisions (no attacks)
    game.run_turn(&passive).unwrap();

    // No damage should have been dealt
    assert_eq!(game.state.players[0].life_total, 20);
    assert_eq!(game.state.players[1].life_total, 20);
}

// ---------------------------------------------------------------------------
// Test: Summoning-sick creature cannot attack
// ---------------------------------------------------------------------------

#[test]
fn test_summoning_sick_cannot_attack() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![
            make_test_deck(vec![], 20),
            make_test_deck(vec![], 20),
        ],
    ).unwrap();
    let passive = PassiveDecisionProvider;
    game.setup(&passive).unwrap();

    // Place creature that IS summoning sick
    let data = creatures::grizzly_bears();
    let obj = mtgsim::objects::object::GameObject::new(data, 0, Zone::Battlefield);
    let bears_id = obj.id;
    game.state.add_object(obj);
    game.state.place_on_battlefield(bears_id, 0); // entered this turn = summoning sick

    advance_to_step(&mut game, PhaseType::Combat, Some(StepType::DeclareAttackers));

    let scripted = ScriptedDecisionProvider::new();
    scripted.attack_decisions.borrow_mut().push(
        vec![(bears_id, AttackTarget::Player(1))]
    );

    // Should fail validation
    let result = game.state.process_declare_attackers(&scripted);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("summoning sickness"));
}

// ---------------------------------------------------------------------------
// Test: Combat damage kills player (game over)
// ---------------------------------------------------------------------------

#[test]
fn test_combat_damage_kills_player() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![
            make_test_deck(vec![], 20),
            make_test_deck(vec![], 20),
        ],
    ).unwrap();
    let passive = PassiveDecisionProvider;
    game.setup(&passive).unwrap();

    // Set player 1 to 2 life so a single 2/2 attack kills them
    game.state.players[1].life_total = 2;

    let bears_id = place_creature_on_battlefield(&mut game, 0, creatures::grizzly_bears);

    advance_to_step(&mut game, PhaseType::Combat, Some(StepType::DeclareAttackers));

    let scripted = ScriptedDecisionProvider::new();
    scripted.attack_decisions.borrow_mut().push(
        vec![(bears_id, AttackTarget::Player(1))]
    );
    scripted.block_decisions.borrow_mut().push(vec![]);

    game.state.process_declare_attackers(&scripted).unwrap();
    game.state.advance_turn().unwrap();
    game.state.process_declare_blockers(&scripted).unwrap();
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, true).unwrap();
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, false).unwrap();

    // Player 1 at 0 life
    assert_eq!(game.state.players[1].life_total, 0);

    // Run priority loop — SBAs fire, flagging player 1 as lost
    for _ in 0..10 {
        scripted.priority_decisions.borrow_mut().push(PriorityAction::Pass);
    }
    game.state.run_priority_loop(&scripted).unwrap();
    assert!(game.state.player_lost[1]);

    // Game should detect winner
    let result = game.check_game_over();
    assert_eq!(result, Some(GameResult::Winner(0)));
}

// ---------------------------------------------------------------------------
// Test: Combat state cleared at end of combat
// ---------------------------------------------------------------------------

#[test]
fn test_combat_state_cleared_after_combat() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![
            make_test_deck(vec![], 20),
            make_test_deck(vec![], 20),
        ],
    ).unwrap();
    let passive = PassiveDecisionProvider;
    game.setup(&passive).unwrap();

    let bears_id = place_creature_on_battlefield(&mut game, 0, creatures::grizzly_bears);

    advance_to_step(&mut game, PhaseType::Combat, Some(StepType::DeclareAttackers));

    let scripted = ScriptedDecisionProvider::new();
    scripted.attack_decisions.borrow_mut().push(
        vec![(bears_id, AttackTarget::Player(1))]
    );
    scripted.block_decisions.borrow_mut().push(vec![]);
    for _ in 0..20 {
        scripted.priority_decisions.borrow_mut().push(PriorityAction::Pass);
    }

    game.state.process_declare_attackers(&scripted).unwrap();

    // Verify attacking state is set
    assert!(game.state.battlefield.get(&bears_id).unwrap().attacking.is_some());
    assert!(game.state.attacks_declared);

    // Advance through rest of combat to postcombat
    // DeclareBlockers
    game.state.advance_turn().unwrap();
    game.state.process_declare_blockers(&scripted).unwrap();
    // FirstStrikeDamage
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, true).unwrap();
    // CombatDamage
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, false).unwrap();
    // EndCombat
    game.state.advance_turn().unwrap();
    assert_eq!(game.state.phase.step, Some(StepType::EndCombat));
    // Advance past combat phase to postcombat
    game.state.advance_turn().unwrap();
    assert_eq!(game.state.phase.phase_type, PhaseType::Postcombat);

    // Combat state should be cleared
    assert!(game.state.battlefield.get(&bears_id).unwrap().attacking.is_none());
    assert!(!game.state.attacks_declared);
    assert!(!game.state.blockers_declared);
    assert!(game.state.blocker_damage_divisions.is_empty());
}

// ---------------------------------------------------------------------------
// Test: Marked damage persists between phases — bolt in 2nd main kills
// ---------------------------------------------------------------------------
//
// Earth Elemental (4/5) is blocked by Savannah Lions (2/1).
// After combat: Elemental has 2 damage marked, Lions is dead.
// In postcombat main phase, Lightning Bolt deals 3 more to Elemental.
// Total damage 2 + 3 = 5 ≥ 5 toughness → Elemental dies via SBA.

#[test]
fn test_damage_persists_bolt_in_second_main_kills() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![
            make_test_deck(vec![], 20),
            make_test_deck(vec![], 20),
        ],
    ).unwrap();
    let passive = PassiveDecisionProvider;
    game.setup(&passive).unwrap();

    // Player 0: Earth Elemental (4/5)
    let elemental = place_creature_on_battlefield(&mut game, 0, creatures::earth_elemental);
    // Player 1: Savannah Lions (2/1)
    let lions = place_creature_on_battlefield(&mut game, 1, creatures::savannah_lions);

    // Advance to DeclareAttackers
    advance_to_step(&mut game, PhaseType::Combat, Some(StepType::DeclareAttackers));

    let scripted = ScriptedDecisionProvider::new();
    scripted.attack_decisions.borrow_mut().push(
        vec![(elemental, AttackTarget::Player(1))]
    );
    scripted.block_decisions.borrow_mut().push(
        vec![(lions, elemental)]
    );

    // Declare attackers
    game.state.process_declare_attackers(&scripted).unwrap();
    // Declare blockers
    game.state.advance_turn().unwrap();
    game.state.process_declare_blockers(&scripted).unwrap();
    // First strike damage (no-op)
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, true).unwrap();
    // Combat damage
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, false).unwrap();

    // Elemental: 2 damage marked (from Lions' 2 power), not lethal on 5 toughness
    assert_eq!(game.state.battlefield.get(&elemental).unwrap().damage_marked, 2);
    // Lions: 4 damage marked (from Elemental's 4 power), lethal on 1 toughness
    assert_eq!(game.state.battlefield.get(&lions).unwrap().damage_marked, 4);

    // Run priority → SBAs kill Lions, Elemental survives
    for _ in 0..10 {
        scripted.priority_decisions.borrow_mut().push(PriorityAction::Pass);
    }
    game.state.run_priority_loop(&scripted).unwrap();

    assert!(game.state.battlefield.contains_key(&elemental));
    assert!(!game.state.battlefield.contains_key(&lions));

    // Advance through EndCombat to Postcombat main phase
    advance_to_step(&mut game, PhaseType::Postcombat, None);

    // Verify damage is STILL marked on Elemental (persists between phases)
    assert_eq!(game.state.battlefield.get(&elemental).unwrap().damage_marked, 2);

    // Put Lightning Bolt in player 1's hand and give them red mana
    let bolt_id = put_in_hand(&mut game, alpha::lightning_bolt(), 1);
    game.state.players[1].mana_pool.add(ManaType::Red, 1);

    // Player 1 casts bolt targeting Elemental
    let bolt_scripted = ScriptedDecisionProvider::new();
    bolt_scripted.priority_decisions.borrow_mut().push(PriorityAction::Pass); // player 0 passes
    bolt_scripted.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(bolt_id)); // player 1 casts
    bolt_scripted.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(elemental)]);
    // After cast, both pass to resolve
    for _ in 0..10 {
        bolt_scripted.priority_decisions.borrow_mut().push(PriorityAction::Pass);
    }

    // Run priority loop: cast bolt → resolve → SBAs fire
    game.state.run_priority_loop(&bolt_scripted).unwrap();

    // Elemental now has 2 (combat) + 3 (bolt) = 5 damage ≥ 5 toughness → dead
    assert!(!game.state.battlefield.contains_key(&elemental));
    assert_eq!(game.state.get_object(elemental).unwrap().zone, Zone::Graveyard);
}

// ---------------------------------------------------------------------------
// Test: Damage clears at cleanup — bolt next upkeep doesn't kill
// ---------------------------------------------------------------------------
//
// Same setup: Earth Elemental (4/5) blocked by Savannah Lions (2/1).
// After combat: Elemental has 2 damage marked.
// Advance through cleanup (rule 514.2: damage removed) into next turn.
// In upkeep, Lightning Bolt deals 3 to Elemental.
// Only 3 damage on 5 toughness → Elemental survives.

#[test]
fn test_damage_clears_at_cleanup_bolt_next_turn_survives() {
    let config = GameConfig::test();
    let mut game = Game::new(
        config,
        vec![
            make_test_deck(vec![], 20),
            make_test_deck(vec![], 20),
        ],
    ).unwrap();
    let passive = PassiveDecisionProvider;
    game.setup(&passive).unwrap();

    // Player 0: Earth Elemental (4/5)
    let elemental = place_creature_on_battlefield(&mut game, 0, creatures::earth_elemental);
    // Player 1: Savannah Lions (2/1)
    let lions = place_creature_on_battlefield(&mut game, 1, creatures::savannah_lions);

    // Advance to DeclareAttackers
    advance_to_step(&mut game, PhaseType::Combat, Some(StepType::DeclareAttackers));

    let scripted = ScriptedDecisionProvider::new();
    scripted.attack_decisions.borrow_mut().push(
        vec![(elemental, AttackTarget::Player(1))]
    );
    scripted.block_decisions.borrow_mut().push(
        vec![(lions, elemental)]
    );

    // Run combat
    game.state.process_declare_attackers(&scripted).unwrap();
    game.state.advance_turn().unwrap();
    game.state.process_declare_blockers(&scripted).unwrap();
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, true).unwrap();
    game.state.advance_turn().unwrap();
    game.state.process_combat_damage(&scripted, false).unwrap();

    // Run priority → SBAs kill Lions
    for _ in 0..10 {
        scripted.priority_decisions.borrow_mut().push(PriorityAction::Pass);
    }
    game.state.run_priority_loop(&scripted).unwrap();

    assert!(game.state.battlefield.contains_key(&elemental));
    assert_eq!(game.state.battlefield.get(&elemental).unwrap().damage_marked, 2);

    // Now advance through the rest of this turn (postcombat, end, cleanup)
    // and into the next turn's upkeep. Cleanup clears damage (rule 514.2).
    // We need enough priority passes for each step.
    for _ in 0..40 {
        scripted.priority_decisions.borrow_mut().push(PriorityAction::Pass);
    }

    // Advance to next turn's Upkeep step
    // Turn structure: EndCombat → Postcombat → End(EndStep) → End(Cleanup) → Beginning(Untap) → Beginning(Upkeep)
    advance_to_step(&mut game, PhaseType::Beginning, Some(StepType::Upkeep));

    // Verify damage was cleared during cleanup
    assert_eq!(game.state.battlefield.get(&elemental).unwrap().damage_marked, 0);

    // Put Lightning Bolt in player 1's hand and give them red mana
    let bolt_id = put_in_hand(&mut game, alpha::lightning_bolt(), 1);
    game.state.players[1].mana_pool.add(ManaType::Red, 1);

    // It's now player 1's turn (active player rotated after cleanup).
    // Active player gets priority first, so player 1 casts immediately.
    let bolt_scripted = ScriptedDecisionProvider::new();
    bolt_scripted.priority_decisions.borrow_mut().push(PriorityAction::CastSpell(bolt_id)); // player 1 (active) casts
    bolt_scripted.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(elemental)]);
    for _ in 0..10 {
        bolt_scripted.priority_decisions.borrow_mut().push(PriorityAction::Pass);
    }

    // Run priority loop: cast bolt → resolve → SBAs check
    game.state.run_priority_loop(&bolt_scripted).unwrap();

    // Elemental has only 3 damage (bolt) on 5 toughness → survives
    assert!(game.state.battlefield.contains_key(&elemental));
    assert_eq!(game.state.battlefield.get(&elemental).unwrap().damage_marked, 3);
}
