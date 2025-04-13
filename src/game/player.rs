// src/game/player.rs
use crate::game::game_obj::GameObj;
use crate::utils::mana::ManaPool;
use crate::utils::constants::card_types::CardType;
use crate::utils::constants::zones::Zone;
use crate::utils::constants::id_types::{PlayerId, ObjectId};

#[derive(Debug, Clone)]
pub struct Player {
    pub id: usize,

    // player zones
    pub hand: Vec<GameObj>,
    pub library: Vec<GameObj>,
    pub graveyard: Vec<GameObj>,

    // player state
    pub life_total: i64,
    pub mana_pool: ManaPool,
    pub max_lands_this_turn: u32,
    pub lands_played_this_turn: u32,
}

impl Player {
    // Create a new player
    pub fn new(id: PlayerId, starting_life: i64, default_lands_per_turn: u32) -> Self {
        Player{
            id,
            hand: Vec::new(),
            library: Vec::new(),
            graveyard: Vec::new(),
            life_total: starting_life,
            mana_pool: ManaPool::new(),
            max_lands_this_turn: default_lands_per_turn,
            lands_played_this_turn: 0,
        }
    }


    // initialize player's library with provided deck (from game engine, randomized order)
    pub fn set_library(&mut self, cards: Vec<GameObj>) {
        self.library = cards;
    }


    // draw a card from the top of the library
    pub fn draw_card(&mut self) -> Result<(), String> {
        if self.library.is_empty() {
            // need to handle empty library case to have the player lose the game, but we'll do that later
            return Err("Library is empty".to_string());
        }

        let mut card = self.library.pop().unwrap(); // pop from the end of the library vector (top of the library)

        // update the card's zone attribute to hand
        match &mut card {
            GameObj::Card { zone, .. } => { // pattern match the card's zone and update it
                *zone = Zone::Hand;
            }
        }

        // move the card to the player's hand
        self.hand.push(card);
        Ok(())
    }

    // helper to draw multiple cards
    pub fn draw_n_cards(&mut self, n: i64) -> Result<(), String> {
        for _ in 0..n {
            match self.draw_card() {
                Ok(_) => {} // do nothing, card was drawn successfully
                Err(e) => return Err(e), // return the error if drawing fails
            }
        }
        Ok(())
    }


    // play a land from hand
    pub fn play_land(&mut self, card_id: ObjectId) -> Result<GameObj, String>{
        // check if we've played our allotment of lands this turn
        if self.lands_played_this_turn >= self.max_lands_this_turn {
            return Err("Already played a land this turn".to_string());
        }

        // find the card position in hand
        let position = self.hand.iter().position(|card| match card {
            GameObj::Card { id, .. } => *id == card_id,
        });

        let card_index = match position {
            Some(index) => index,
            None => return Err(format!("Card with ID {} not found in hand", card_id)),
        };

        // check if the card is a land
        match &self.hand[card_index] {
            GameObj::Card { characteristics, ..} => {
                if let Some(card_types) = &characteristics.card_type {
                    if !card_types.iter().any(|t| *t == CardType::Land) {
                        return Err(format!("Selected card at index {} is not a land", card_index));
                    }
                }
            }
        }


        // need to move the land from the hand to the battlefield
        // game engine will handle putting it in the right zone, we just need to update the player's state
        // first remove it from the hand
        let mut land = self.hand.remove(card_index);

        // update the land's zone to battlefield
        match &mut land {
            GameObj::Card { zone, .. } => {
                *zone = Zone::Battlefield;
            }
        }

        // increment the lands played this turn
        self.lands_played_this_turn += 1;

        Ok(land) // this result will be handled by the game engine
    }

    
    // Reset lands played at end of turn
    pub fn reset_lands_played(&mut self) {
        self.lands_played_this_turn = 0;
    }
}