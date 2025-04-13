// src/game/card.rs
use uuid::Uuid;
use std::collections::HashSet;
use crate::utils::constants::colors::Color;
use crate::utils::constants::zones::Zone;
use crate::utils::constants::card_types::{CardType, Supertype, LandType};
use crate::game::game_obj::{Characteristics, GameObj};
use crate::utils::constants::id_types::PlayerId;

// Define a temporary simplified enum for land types we need
#[derive(Debug, Clone, PartialEq)]
pub enum BasicLand {
    Plains,
    Island,
    Swamp,
    Mountain,
    Forest,
    Wastes
}

pub fn create_basic_land(land_type: BasicLand, owner: PlayerId) -> GameObj {
    let id = Uuid::new_v4(); // Generate a new unique ID for the card
    let mut card_types = HashSet::new();
    card_types.insert(CardType::Land);

    let mut supertype = HashSet::new();
    supertype.insert(Supertype::Basic);

    // we'll handle the subtype later -- tricky because Wastes doesn't have one and the others are all unique

    // match card name and rules text to land type
    let (card_name, rules_text) = match land_type {
        BasicLand::Plains => ("Plains".to_string(), "T: Add {W}".to_string()),
        BasicLand::Island => ("Island".to_string(), "T: Add {U}".to_string()),
        BasicLand::Swamp => ("Swamp".to_string(), "T: Add {B}".to_string()),
        BasicLand::Mountain => ("Mountain".to_string(), "T: Add {R}".to_string()),
        BasicLand::Forest => ("Forest".to_string(), "T: Add {G}".to_string()),
        BasicLand::Wastes => ("Wastes".to_string(), "T: Add {C}".to_string())
    };

    // Build characteristics for card object
    let characteristics = Characteristics {
        name: Some(card_name),
        mana_cost: None, // Lands don't have a mana cost
        color: Some(HashSet::new()), // Lands are colorless
        color_indicator: None, // Basic lands don't have color indicators
        card_type: Some(card_types),
        supertype: Some(supertype),
        subtype: None, // We'll handle the subtype later
        rules_text: Some(rules_text),
        abilities: None, // We'll add abilities later, just focused on creating, drawing, and playing the land for now
        power: None,
        toughness: None,
        loyalty: None,
        defense: None,
        hand_modifier: None,
        life_modifier: None,
    };

    // Create the GameObj for the land
    GameObj::Card {
        id,
        characteristics,
        zone: Zone::Library, // Lands start in the library
        owner,
        controller: Some(owner), // The owner is also the controller
    }
}

