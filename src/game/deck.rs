// src/game/deck.rs
use rand::seq::SliceRandom;
use rand::rng;
use crate::utils::constants::id_types::PlayerId;
use crate::game::game_obj::GameObj;
use crate::game::card::create_basic_land;
use crate::game::card::BasicLand;

pub struct Deck {
    pub cards: Vec<GameObj>,
    pub owner: PlayerId
}

impl Deck {
    // new empty deck
    pub fn new(owner: PlayerId) -> Self {
        Deck {
            cards: Vec::new(),
            owner
        }
    }

    // shuffle the deck
    pub fn shuffle(&mut self) {
        let mut rng = rng();
        self.cards.shuffle(&mut rng);
    }

    // get cards in library
    pub fn size(&self) -> usize {
        self.cards.len()
    }

    // test function to add basic lands to our test deck
    pub fn create_test_land_deck(owner: PlayerId) -> Self {
        let mut deck = Deck::new(owner);

        // add 5 of each basic land to the deck
        for _ in 0..5 {
            deck.cards.push(create_basic_land(BasicLand::Plains, owner));
            deck.cards.push(create_basic_land(BasicLand::Island, owner));
            deck.cards.push(create_basic_land(BasicLand::Swamp, owner));
            deck.cards.push(create_basic_land(BasicLand::Mountain, owner));
            deck.cards.push(create_basic_land(BasicLand::Forest, owner));
            deck.cards.push(create_basic_land(BasicLand::Wastes, owner));
        }

        deck
    }
}