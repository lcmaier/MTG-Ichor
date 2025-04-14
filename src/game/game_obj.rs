// src/game/game_obj.rs

use std::collections::HashSet;
use crate::utils::constants::id_types::{ObjectId, PlayerId};
use crate::utils::constants::colors::Color;
use crate::utils::constants::zones::Zone;
use crate::utils::constants::card_types::{
    CardType, Supertype, ArtifactType, EnchantmentType, LandType, PlaneswalkerType, SpellType, CreatureType, PlanarType, DungeonType, BattleType, Subtype
};

// define characteristics as per rule 109.3
#[derive(Debug, Clone, PartialEq)]
pub struct Characteristics {
    pub name: Option<String>,
    pub mana_cost: Option<String>,
    pub color: Option<HashSet<Color>>,
    pub color_indicator: Option<HashSet<Color>>,
    pub card_type: Option<HashSet<CardType>>,
    pub supertype: Option<HashSet<Supertype>>,
    pub subtype: Option<HashSet<Subtype>>,
    pub rules_text: Option<String>,
    pub abilities: Option<Vec<String>>, // placeholder, we'll want a proper Ability type once we get to object abilities
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub loyalty: Option<i32>,
    pub defense: Option<i32>,
    pub hand_modifier: Option<i32>,
    pub life_modifier: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameObj {
    Card {
        id: ObjectId,
        characteristics: Characteristics,
        zone: Zone,
        owner: PlayerId,
        controller: Option<PlayerId>
    },
}

impl GameObj {
    // Get the owner of this game object
    pub fn get_owner(&self) -> PlayerId {
        match self {
            GameObj::Card { owner, .. } => *owner,
        }
    }

    // Get the controller of this game object
    pub fn get_controller(&self) -> Option<PlayerId> {
        match self {
            GameObj::Card { controller, .. } => *controller,
        }
    }

    // Get the zone the object is in
    pub fn get_zone(&self) -> &Zone {
        match self {
            GameObj::Card { zone, .. } => zone,
            // Add other variants here when you implement them
        }
    }

    // Check if the game object is in a specific zone
    pub fn is_in_zone(&self, zone_to_check: &Zone) -> bool {
        match self {
            GameObj::Card { zone, .. } => zone == zone_to_check,
            // Add other variants here when you implement them
        }
    }

    // Update the zone
    pub fn set_zone(&mut self, new_zone: Zone) {
        match self {
            GameObj::Card { zone, .. } => *zone = new_zone,
            // Add other variants here when you implement them
        }
    }
}