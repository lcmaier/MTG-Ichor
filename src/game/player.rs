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
    pub max_hand_size: u32,
    pub mana_pool: ManaPool,
    pub max_lands_this_turn: u32,
    pub lands_played_this_turn: u32,
}

impl Player {
    // Create a new player
    pub fn new(id: PlayerId, starting_life: i64, max_hand_size: u32, default_lands_per_turn: u32) -> Self {
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