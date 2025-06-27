// src/cards/green/creature/grizzly_bears.rs

use std::collections::HashSet;

use crate::utils::constants::{card_types::CardType, costs::ManaCost, game_objects::Characteristics};
use crate::utils::constants::card_types::{CreatureType::Bear, Subtype::Creature};

pub fn grizzly_bears_characteristics() -> Characteristics {
    
    Characteristics {
        name: Some("Grizzly Bears".to_string()),
        mana_cost: Some(ManaCost::green(1, 1)), // 1G
        color: Some(HashSet::from([crate::utils::constants::colors::Color::Green])),
        color_indicator: None,
        card_type: Some(HashSet::from([CardType::Creature])),
        supertype: None,
        subtype: Some(HashSet::from([Creature(Bear)])),
        rules_text: Some("".to_string()), // Grizzly Bears is vanilla, i.e. no rules text. We submit Some("") for ease of display and congruence between all creatures (since some are vanilla and our code will often unwrap the rules text along with other characteristics)
        abilities: None,
        power: Some(2),
        toughness: Some(2),
        loyalty: None,
        defense: None,
        hand_modifier: None,
        life_modifier: None,
    }
}