// tests/common.rs - Common test utilities and helpers

use mtgsim::game::gamestate::Game;
use mtgsim::game::player::Player;
use mtgsim::utils::constants::deck::Deck;
use mtgsim::utils::constants::turns::PhaseType;
use mtgsim::utils::constants::zones::Zone;
use mtgsim::utils::constants::game_objects::{GameObj, BattlefieldState};
use mtgsim::utils::mana::ManaType;
use mtgsim::cards::generator::ObjectGenerator;
use mtgsim::utils::constants::id_types::{PlayerId, ObjectId};

/// Sets up a basic two-player game with decks and drawn hands
pub fn setup_basic_game() -> Game {
    let mut game = Game::new();
    
    // Create players
    let mut player1 = Player::new(0, 20, 7, 1);
    let mut player2 = Player::new(1, 20, 7, 1);
    
    // Create and assign decks
    let deck1 = Deck::create_test_deck(0);
    let deck2 = Deck::create_test_deck(1);
    
    player1.set_library(deck1.cards);
    player2.set_library(deck2.cards);
    
    // Draw starting hands
    player1.draw_n_cards(7).expect("Failed to draw cards for player 1");
    player2.draw_n_cards(7).expect("Failed to draw cards for player 2");
    
    // Add players to game
    game.players.push(player1);
    game.players.push(player2);
    
    game
}

/// Sets up a game with specific cards for testing
pub fn setup_game_with_cards(
    player1_cards: Vec<&str>,
    player2_cards: Vec<&str>
) -> Game {
    let mut game = Game::new();
    
    // Create players
    let mut player1 = Player::new(0, 20, 7, 1);
    let mut player2 = Player::new(1, 20, 7, 1);
    
    // Create specific cards for player 1
    for card_name in player1_cards {
        match ObjectGenerator::create_card_in_library(card_name, 0) {
            Ok(card) => player1.library.push(card),
            Err(e) => panic!("Failed to create card {}: {}", card_name, e),
        }
    }
    
    // Create specific cards for player 2
    for card_name in player2_cards {
        match ObjectGenerator::create_card_in_library(card_name, 1) {
            Ok(card) => player2.library.push(card),
            Err(e) => panic!("Failed to create card {}: {}", card_name, e),
        }
    }
    
    game.players.push(player1);
    game.players.push(player2);
    
    game
}

/// Adds mana to a player's mana pool
pub fn add_mana_to_player(game: &mut Game, player_id: PlayerId, mana_type: ManaType, amount: u64) {
    game.get_player_mut(player_id)
        .expect("Player not found")
        .mana_pool
        .add_mana(mana_type, amount);
}

/// Finds a card in a player's hand by name
pub fn find_card_in_hand(game: &Game, player_id: PlayerId, card_name: &str) -> Option<ObjectId> {
    game.get_player_ref(player_id)
        .ok()?
        .hand
        .iter()
        .find(|card| card.characteristics.name.as_deref() == Some(card_name))
        .map(|card| card.id)
}

/// Advances the game to a specific phase
pub fn advance_to_phase(game: &mut Game, target_phase: PhaseType) {
    while game.phase.phase_type != target_phase {
        game.advance_turn().unwrap();
    }
}

/// Resolves all objects on the stack
pub fn resolve_stack(game: &mut Game) -> Result<(), String> {
    while !game.stack.is_empty() {
        game.resolve_top_of_stack()?;
    }
    Ok(())
}

/// Verifies that a player has a specific amount of life
pub fn assert_player_life(game: &Game, player_id: PlayerId, expected_life: i64) {
    let actual_life = game.get_player_ref(player_id)
        .expect("Player not found")
        .life_total;
    assert_eq!(actual_life, expected_life, 
        "Player {} life mismatch: expected {}, got {}", 
        player_id, expected_life, actual_life);
}

/// Verifies that a creature has specific power/toughness
pub fn assert_creature_stats(
    game: &Game, 
    creature_id: ObjectId, 
    expected_power: i32, 
    expected_toughness: i32
) {
    let creature = game.battlefield.get(&creature_id)
        .expect("Creature not found on battlefield");
    
    let power = creature.characteristics.power.expect("Creature has no power");
    let toughness = creature.characteristics.toughness.expect("Creature has no toughness");
    
    assert_eq!(power, expected_power, 
        "Creature power mismatch: expected {}, got {}", expected_power, power);
    assert_eq!(toughness, expected_toughness,
        "Creature toughness mismatch: expected {}, got {}", expected_toughness, toughness);
}