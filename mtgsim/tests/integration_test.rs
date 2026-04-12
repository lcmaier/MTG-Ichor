//! Integration tests: end-to-end game flows.
//!
//! Tests here exercise multiple subsystems together:
//! zone transitions, turn structure, mana abilities, SBAs, and events.

use std::sync::Arc;

use mtgsim::cards::basic_lands;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::{AbilityType, CardData};
use mtgsim::objects::object::GameObject;
use mtgsim::state::game_state::{GameState, PhaseType, StepType};
use mtgsim::types::ids::AbilityId;
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;

/// Build a deck of basic lands for a player
fn build_test_deck(game: &mut GameState, player_id: usize, land_fn: fn() -> Arc<CardData>, count: usize) {
    for _ in 0..count {
        let card_data = land_fn();
        let obj = GameObject::in_library(card_data, player_id);
        let id = game.add_object(obj);
        game.players[player_id].library.push(id);
    }
}

/// Get the first mana ability ID from a permanent on the battlefield
fn get_mana_ability_id(game: &GameState, permanent_id: mtgsim::types::ids::ObjectId) -> AbilityId {
    let obj = game.get_object(permanent_id).unwrap();
    obj.card_data.abilities.iter()
        .find(|a| a.ability_type == AbilityType::Mana)
        .expect("Permanent has no mana ability")
        .id
}

#[test]
fn test_full_opening_sequence() {
    let mut game = GameState::new(2, 20);

    build_test_deck(&mut game, 0, basic_lands::forest, 10);
    build_test_deck(&mut game, 1, basic_lands::mountain, 10);

    // -- Verify initial state --
    assert_eq!(game.players[0].library.len(), 10);
    assert_eq!(game.players[1].library.len(), 10);
    assert_eq!(game.turn_number, 1);
    assert_eq!(game.active_player, 0);
    assert_eq!(game.phase.phase_type, PhaseType::Beginning);

    // -- Advance through Beginning phase --
    game.advance_turn().unwrap();
    assert_eq!(game.phase.step, Some(StepType::Upkeep));

    game.advance_turn().unwrap();
    assert_eq!(game.phase.step, Some(StepType::Draw));
    assert_eq!(game.players[0].hand.len(), 1, "Player 0 should have drawn a card");
    assert_eq!(game.players[0].library.len(), 9);

    game.advance_turn().unwrap();
    assert_eq!(game.phase.phase_type, PhaseType::Precombat);
    assert_eq!(game.phase.step, None);

    // -- Play a land from hand --
    let land_id = game.players[0].hand[0];
    game.play_land(0, land_id, Zone::Hand).unwrap();

    assert!(game.battlefield.contains_key(&land_id));
    assert_eq!(game.get_object(land_id).unwrap().zone, Zone::Battlefield);
    assert!(game.players[0].hand.is_empty());
    assert_eq!(game.players[0].lands_played_this_turn, 1);

    // -- Tap the land for mana (explicit ability ID) --
    let ability_id = get_mana_ability_id(&game, land_id);
    game.activate_mana_ability(0, land_id, ability_id).unwrap();

    assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 1);
    assert!(game.battlefield.get(&land_id).unwrap().tapped);

    // -- Can't tap again --
    assert!(game.activate_mana_ability(0, land_id, ability_id).is_err());

    // -- Advance through the rest of the turn --
    for _ in 0..10 {
        game.advance_turn().unwrap();
    }

    assert_eq!(game.turn_number, 2);
    assert_eq!(game.active_player, 1);
    assert_eq!(game.phase.phase_type, PhaseType::Beginning);
    assert_eq!(game.players[0].mana_pool.total(), 0);
}

#[test]
fn test_two_turn_land_and_mana_cycle() {
    let mut game = GameState::new(2, 20);
    build_test_deck(&mut game, 0, basic_lands::forest, 10);
    build_test_deck(&mut game, 1, basic_lands::mountain, 10);

    // -- Turn 1: Player 0 --
    for _ in 0..3 {
        game.advance_turn().unwrap();
    }
    assert_eq!(game.phase.phase_type, PhaseType::Precombat);

    let land1_id = game.players[0].hand[0];
    game.play_land(0, land1_id, Zone::Hand).unwrap();
    let ability1 = get_mana_ability_id(&game, land1_id);
    game.activate_mana_ability(0, land1_id, ability1).unwrap();
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 1);

    for _ in 0..10 {
        game.advance_turn().unwrap();
    }

    // -- Turn 2: Player 1 --
    assert_eq!(game.turn_number, 2);
    assert_eq!(game.active_player, 1);

    for _ in 0..3 {
        game.advance_turn().unwrap();
    }

    let land2_id = game.players[1].hand[0];
    game.play_land(1, land2_id, Zone::Hand).unwrap();
    let ability2 = get_mana_ability_id(&game, land2_id);
    game.activate_mana_ability(1, land2_id, ability2).unwrap();
    assert_eq!(game.players[1].mana_pool.amount(ManaType::Red), 1);

    for _ in 0..10 {
        game.advance_turn().unwrap();
    }

    // -- Turn 3: Player 0 again --
    assert_eq!(game.turn_number, 3);
    assert_eq!(game.active_player, 0);

    assert!(!game.battlefield.get(&land1_id).unwrap().tapped);

    game.activate_mana_ability(0, land1_id, ability1).unwrap();
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 1);
}

#[test]
fn test_event_log_records_zone_changes() {
    let mut game = GameState::new(2, 20);
    build_test_deck(&mut game, 0, basic_lands::forest, 5);

    let initial_events = game.events.len();

    game.advance_turn().unwrap(); // Untap -> Upkeep
    game.advance_turn().unwrap(); // Upkeep -> Draw

    assert!(game.events.len() > initial_events, "Should have emitted events");

    let events_before_play = game.events.len();
    game.advance_turn().unwrap(); // Draw -> Precombat

    let land_id = game.players[0].hand[0];
    game.play_land(0, land_id, Zone::Hand).unwrap();

    assert!(game.events.len() > events_before_play, "Playing a land should emit a zone change event");
}

#[test]
fn test_card_registry_integration() {
    let registry = CardRegistry::default_registry();

    let mut game = GameState::new(2, 20);

    let land_names = ["Forest", "Mountain", "Plains", "Island", "Swamp"];
    for (i, &name) in land_names.iter().enumerate() {
        let card_data = registry.create(name).unwrap();
        let obj = GameObject::in_library(card_data, i % 2);
        let id = game.add_object(obj);
        game.players[i % 2].library.push(id);
    }

    assert_eq!(game.players[0].library.len(), 3);
    assert_eq!(game.players[1].library.len(), 2);

    let drawn_id = game.draw_card(0).unwrap().expect("Should have drawn a card");
    let drawn_obj = game.get_object(drawn_id).unwrap();
    assert_eq!(drawn_obj.card_data.name, "Swamp");
}

#[test]
fn test_sba_integration_with_turn_structure() {
    let mut game = GameState::new(2, 20);
    build_test_deck(&mut game, 0, basic_lands::forest, 10);
    build_test_deck(&mut game, 1, basic_lands::mountain, 10);

    let bears_data = mtgsim::objects::card_data::CardDataBuilder::new("Grizzly Bears")
        .mana_cost(mtgsim::types::mana::ManaCost::single(ManaType::Green, 1, 1))
        .color(mtgsim::types::colors::Color::Green)
        .card_type(mtgsim::types::card_types::CardType::Creature)
        .subtype(mtgsim::types::card_types::Subtype::Creature(
            mtgsim::types::card_types::CreatureType::Bear,
        ))
        .power_toughness(2, 2)
        .build();

    let bears = GameObject::new(bears_data.clone(), 0, Zone::Battlefield);
    let bears_id = bears.id;
    game.add_object(bears);
    game.place_on_battlefield(bears_id, 0).damage_marked = 3;

    game.check_state_based_actions_loop(&mtgsim::ui::decision::PassiveDecisionProvider).unwrap();

    assert!(!game.battlefield.contains_key(&bears_id));
    assert_eq!(game.players[0].graveyard.len(), 1);
    assert_eq!(game.get_object(bears_id).unwrap().zone, Zone::Graveyard);
}
