// src/game/collections/deck.rs

// this file contains in-game deck functionality
use rand::rng;
use rand::seq::SliceRandom;
use crate::{cards::basic_lands::{create_basic_land, BasicLand}, utils::constants::{deck::{Deck, DeckFormat}, game_objects::{GameObj, LibraryState}, id_types::PlayerId}};

impl Deck {
    // Create a new empty deck of the specified format
    pub fn new(owner: PlayerId, deck_format: DeckFormat) -> Self {
        Deck {
            cards: Vec::new(),
            owner,
            deck_format,
        }
    }

    // Ensure deck is within size requirements
    fn is_above_min_size(&self) -> bool {
        self.cards.len() >= self.deck_format.min_size()
    }

    fn is_below_max_size(&self) -> bool {
        match self.deck_format.max_size() {
            Some(size) => self.cards.len() <= size,
            None => true // if no upper limit, any number of cards are permitted in the main deck (NOTE: might implement internal upper limit to avoid unexpected crashing)
        }
    }

    pub fn is_valid(&self) -> bool {
        self.is_above_min_size() && self.is_below_max_size()
    }

    // Shuffling
    pub fn shuffle(&mut self) {
        let mut rng = rng();
        self.cards.shuffle(&mut rng);
    }

    // get number of cards in deck
    pub fn size(&self) -> usize {
        self.cards.len()
    }

    // Put a prepared LibraryState card on the bottom of the deck
    pub fn put_card_on_bottom(&mut self, card: GameObj<LibraryState>) {
        self.cards.push(card);
    }

    // TEMP: Create a test deck
    pub fn create_test_deck(owner: PlayerId) -> Self {
        let mut deck = Deck::new(owner, DeckFormat::Limited); // the alpha deck will be 40 cards for simplicity

        // for now, add 20 of each basic land we're interested in to the test deck
        for _ in 0..20 {
            deck.put_card_on_bottom(create_basic_land(BasicLand::Forest, owner));
            deck.put_card_on_bottom(create_basic_land(BasicLand::Mountain, owner));
        }

        deck.shuffle();
        deck
    }


}