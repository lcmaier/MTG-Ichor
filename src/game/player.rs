// src/game/player.rs
use crate::utils::constants::game_objects::{GameObj, GraveyardState, HandState, LibraryState};
use crate::utils::mana::ManaPool;
use crate::utils::constants::card_types::CardType;
use crate::utils::constants::zones::Zone;
use crate::utils::constants::id_types::{PlayerId, ObjectId};

#[derive(Debug, Clone)]
pub struct Player {
    pub id: usize,

    // player zones
    pub hand: Vec<GameObj<HandState>>,
    pub library: Vec<GameObj<LibraryState>>,
    pub graveyard: Vec<GameObj<GraveyardState>>,

    // player state
    pub life_total: i64,
    pub max_hand_size: i32,
    pub mana_pool: ManaPool,
    pub max_lands_this_turn: u32,
    pub lands_played_this_turn: u32,
}

impl Player {
    // Create a new player
    pub fn new(id: PlayerId, starting_life: i64, max_hand_size: i32, default_lands_per_turn: u32) -> Self {
        Player{
            id,
            hand: Vec::new(),
            library: Vec::new(),
            graveyard: Vec::new(),
            life_total: starting_life,
            max_hand_size, 
            mana_pool: ManaPool::new(),
            max_lands_this_turn: default_lands_per_turn,
            lands_played_this_turn: 0,
        }
    }


    // initialize player's library with provided deck (from game engine, randomized order)
    pub fn set_library(&mut self, cards: Vec<GameObj<LibraryState>>) {
        self.library = cards;
    }


    // draw a card from the top of the library
    pub fn draw_card(&mut self) -> Result<(), String> {
        if self.library.is_empty() {
            // need to handle empty library case to have the player lose the game, but we'll do that later
            return Err("Library is empty".to_string());
        }

        let card = self.library.pop().unwrap(); // pop from the end of the library vector (top of the library)

        // Convert the LibraryState to HandState
        let hand_card = card.to_hand();

        // move the card to the player's hand
        self.hand.push(hand_card);
        Ok(())
    }

    // helper to draw multiple cards
    pub fn draw_n_cards(&mut self, n: u64) -> Result<(), String> {
        for _ in 0..n {
            match self.draw_card() {
                Ok(_) => {} // do nothing, card was drawn successfully
                Err(e) => return Err(e), // return the error if drawing fails
            }
        }
        Ok(())
    }

    pub fn show_hand(&mut self) -> Result<(), String> {
        println!("\nYour hand:");
        for (i, card) in self.hand.iter().enumerate() {
            if let Some(name) = &card.characteristics.name {
                if let Some(rules_text) = &card.characteristics.rules_text {
                    println!("{}: {} - {}", i + 1, name, rules_text);
                } else {
                    println!("{}: {} - (No rules text)", i + 1, name);
                }
            } else {
                println!("{}: ERROR: UNKNOWN CARD", i + 1);
            }
        }

        Ok(())
    }

    // Find card in hand by ID and return a reference to the HandState GameObj
    pub fn get_card_in_hand(&self, card_id: ObjectId) -> Option<&GameObj<HandState>> {
        self.hand.iter().find(|card| card.id == card_id)
    }

    // Get a mutable reference to a card in hand by ID
    pub fn get_card_in_hand_mut(&mut self, card_id: ObjectId) -> Option<&mut GameObj<HandState>> {
        self.hand.iter_mut().find(|card| card.id == card_id)
    }

    // Remove a card from hand by ID
    pub fn remove_card_from_hand(&mut self, card_id: ObjectId) -> Result<GameObj<HandState>, String> {
        let mut extracted = self.hand.extract_if(.., |card| card.id == card_id);

        // get first (and should be only) matching card
        if let Some(card) = extracted.next() {
            Ok(card)
        } else {
            Err(format!("Card with ID {} not found in hand", card_id))
        }
    }

    
    // Reset lands played at end of turn
    pub fn reset_lands_played(&mut self) {
        self.lands_played_this_turn = 0;
    }

    // Increase max land count for this turn
    pub fn increase_land_drop_limit(&mut self, amount: u32) {
        self.max_lands_this_turn += amount;
    }
}


// UNIT TESTS
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cards::basic_lands::{create_basic_land, BasicLand}, utils::mana::ManaType};

    #[test]
    fn test_player_creation() {
        let player = Player::new(0, 20, 7, 1);

        assert_eq!(player.id, 0);
        assert_eq!(player.life_total, 20);
        assert_eq!(player.max_hand_size, 7);
        assert_eq!(player.max_lands_this_turn, 1);
        assert_eq!(player.lands_played_this_turn, 0);
        assert!(player.hand.is_empty());
        assert!(player.library.is_empty());
        assert!(player.graveyard.is_empty());
    }

    #[test]
    fn test_set_library() {
        let mut player = Player::new(0, 20, 7, 1);
        let mut cards = Vec::new();
        
        // Create 5 test cards
        for _ in 0..5 {
            cards.push(create_basic_land(BasicLand::Forest, player.id));
        }
        
        player.set_library(cards.clone());
        assert_eq!(player.library.len(), 5);
    }

    #[test]
    fn test_draw_card_success() {
        let mut player = Player::new(0, 20, 7, 1);
        let mut cards = Vec::new();
        
        // Create test deck
        for _ in 0..3 {
            cards.push(create_basic_land(BasicLand::Mountain, player.id));
        }
        
        player.set_library(cards);
        assert_eq!(player.library.len(), 3);
        assert_eq!(player.hand.len(), 0);
        
        // Draw a card
        let result = player.draw_card();
        assert!(result.is_ok());
        assert_eq!(player.library.len(), 2);
        assert_eq!(player.hand.len(), 1);
    }

    #[test]
    fn test_draw_card_empty_library() {
        let mut player = Player::new(0, 20, 7, 1);
        
        // Try to draw from empty library
        let result = player.draw_card();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Library is empty");
    }

    #[test]
    fn test_draw_n_cards() {
        let mut player = Player::new(0, 20, 7, 1);
        let mut cards = Vec::new();
        
        // Create test deck with 10 cards
        for _ in 0..10 {
            cards.push(create_basic_land(BasicLand::Forest, player.id));
        }
        
        player.set_library(cards);
        
        // Draw 7 cards
        let result = player.draw_n_cards(7);
        assert!(result.is_ok());
        assert_eq!(player.library.len(), 3);
        assert_eq!(player.hand.len(), 7);
    }

    #[test]
    fn test_draw_n_cards_not_enough() {
        let mut player = Player::new(0, 20, 7, 1);
        let mut cards = Vec::new();
        
        // Create test deck with only 3 cards
        for _ in 0..3 {
            cards.push(create_basic_land(BasicLand::Mountain, player.id));
        }
        
        player.set_library(cards);
        
        // Try to draw 5 cards
        let result = player.draw_n_cards(5);
        assert!(result.is_err());
        assert_eq!(player.hand.len(), 3); // Should have drawn 3 cards before failing
        assert_eq!(player.library.len(), 0);
    }

    #[test]
    fn test_land_play_tracking() {
        let mut player = Player::new(0, 20, 7, 1);
        
        assert_eq!(player.lands_played_this_turn, 0);
        assert_eq!(player.max_lands_this_turn, 1);
        
        // Simulate playing a land
        player.lands_played_this_turn += 1;
        assert_eq!(player.lands_played_this_turn, 1);
    }
    
    #[test]
    fn test_mana_pool_operations() {
        let mut player = Player::new(0, 20, 7, 1);
        
        // Test adding mana
        player.mana_pool.add_mana(ManaType::Red, 3);
        assert!(player.mana_pool.has_mana(ManaType::Red, 3));
        assert!(!player.mana_pool.has_mana(ManaType::Red, 4));
        
        // Test removing mana
        let result = player.mana_pool.remove_mana(ManaType::Red, 2);
        assert!(result.is_ok());
        assert!(player.mana_pool.has_mana(ManaType::Red, 1));
        assert!(!player.mana_pool.has_mana(ManaType::Red, 2));
        
        // Test removing too much mana
        let result = player.mana_pool.remove_mana(ManaType::Red, 2);
        assert!(result.is_err());
        
        // Test emptying mana pool
        player.mana_pool.empty();
        assert!(!player.mana_pool.has_mana(ManaType::Red, 1));
    }
}