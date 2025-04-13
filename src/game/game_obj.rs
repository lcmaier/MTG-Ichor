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