// src/game/collections/deck.rs

// this file contains in-game deck functionality
use rand::rng;
use rand::seq::SliceRandom;
use crate::{cards::{basic_lands::{create_basic_land, BasicLand}, generator::ObjectGenerator}, utils::constants::{deck::{Deck, DeckFormat}, game_objects::{GameObj, LibraryState}, id_types::PlayerId}};

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

    pub fn create_lightning_bolt_deck(owner: PlayerId) -> Self {
        let mut deck = Deck::new(owner, DeckFormat::Limited);
        for _ in 0..20 {
            deck.put_card_on_bottom(create_basic_land(BasicLand::Mountain, owner));
            match ObjectGenerator::create_card_in_library(&"Lightning Bolt", owner) {
                Ok(card) => deck.put_card_on_bottom(card),
                Err(e) => panic!("Error creating Lightning Bolt: {}", e),
            }
        }
        deck.shuffle();
        deck
    }


}


// UNIT TESTS
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deck_creation() {
        let deck = Deck::new(0, DeckFormat::Limited);
        
        assert_eq!(deck.owner, 0);
        assert_eq!(deck.deck_format, DeckFormat::Limited);
        assert!(deck.cards.is_empty());
        assert_eq!(deck.size(), 0);
    }
    
    #[test]
    fn test_deck_validation_limited() {
        let mut deck = Deck::new(0, DeckFormat::Limited);
        
        // Limited requires minimum 40 cards
        assert!(!deck.is_valid()); // Empty deck is invalid
        
        // Add 39 cards - still invalid
        for _ in 0..39 {
            deck.put_card_on_bottom(create_basic_land(BasicLand::Forest, 0));
        }
        assert!(!deck.is_valid());
        assert!(!deck.is_above_min_size());
        
        // Add one more to reach 40 - now valid
        deck.put_card_on_bottom(create_basic_land(BasicLand::Forest, 0));
        assert!(deck.is_valid());
        assert!(deck.is_above_min_size());
        assert!(deck.is_below_max_size());
    }
    
    #[test]
    fn test_put_card_on_bottom() {
        let mut deck = Deck::new(0, DeckFormat::Limited);
        
        let card1 = create_basic_land(BasicLand::Mountain, 0);
        let card1_id = card1.id.clone();
        deck.put_card_on_bottom(card1);
        
        assert_eq!(deck.size(), 1);
        assert_eq!(deck.cards[0].id, card1_id);
        
        let card2 = create_basic_land(BasicLand::Forest, 0);
        let card2_id = card2.id.clone();
        deck.put_card_on_bottom(card2);
        
        assert_eq!(deck.size(), 2);
        assert_eq!(deck.cards[1].id, card2_id); // Second card at bottom
    }
    
    #[test]
    fn test_shuffle() {
        let mut deck = Deck::new(0, DeckFormat::Limited);
        
        // Create a deck with distinguishable cards
        for i in 0..40 {
            if i < 20 {
                deck.put_card_on_bottom(create_basic_land(BasicLand::Mountain, 0));
            } else {
                deck.put_card_on_bottom(create_basic_land(BasicLand::Forest, 0));
            }
        }
        
        // Check initial ordering - first 20 are Mountains
        let initial_first_card_type = deck.cards[0].characteristics.name.clone();
        
        // Shuffle multiple times to ensure randomization
        let mut shuffled_differently = false;
        for _ in 0..10 {
            deck.shuffle();
            if deck.cards[0].characteristics.name != initial_first_card_type {
                shuffled_differently = true;
                break;
            }
        }
        
        assert!(shuffled_differently, "Deck should be shuffled into different order");
        assert_eq!(deck.size(), 40); // Size unchanged
    }
    
    #[test]
    fn test_create_test_deck() {
        let deck = Deck::create_test_deck(0);
        
        assert_eq!(deck.owner, 0);
        assert_eq!(deck.size(), 40); // 20 Mountains + 20 Forests
        assert!(deck.is_valid());
        
        // Count each land type
        let mut mountains = 0;
        let mut forests = 0;
        
        for card in &deck.cards {
            match card.characteristics.name.as_deref() {
                Some("Mountain") => mountains += 1,
                Some("Forest") => forests += 1,
                _ => panic!("Unexpected card in test deck"),
            }
        }
        
        assert_eq!(mountains, 20);
        assert_eq!(forests, 20);
    }
    
    #[test]
    fn test_create_lightning_bolt_deck() {
        let deck = Deck::create_lightning_bolt_deck(0);
        
        assert_eq!(deck.owner, 0);
        assert_eq!(deck.size(), 40); // 20 Mountains + 20 Lightning Bolts
        assert!(deck.is_valid());
        
        // Count each card type
        let mut mountains = 0;
        let mut bolts = 0;
        
        for card in &deck.cards {
            match card.characteristics.name.as_deref() {
                Some("Mountain") => mountains += 1,
                Some("Lightning Bolt") => bolts += 1,
                _ => panic!("Unexpected card in lightning bolt deck"),
            }
        }
        
        assert_eq!(mountains, 20);
        assert_eq!(bolts, 20);
    }
}