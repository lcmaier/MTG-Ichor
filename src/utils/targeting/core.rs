// src/utils/targeting/core.rs

use crate::utils::constants::{card_types::{CardType, Subtype, Supertype}, id_types::{ObjectId, PlayerId}};

// TARGETING STRUCTS AND ENUMS
#[derive(Debug, Clone, PartialEq)]
pub enum TargetZone {
    Battlefield,
    Stack,
    Graveyard,
    Exile,
    // not sure if targeting effects can access other zones, will update if I am proven wrong
}

#[derive(Debug, Clone, PartialEq)]
pub enum TargetCategory {
    // Special targeting categories
    AnyDamageable,       // Any target (creatures, players, planeswalkers, battles)
    Opponent,  // Any opponent player
    // Controller, // Controller of object (not sure if I'll need this one yet)
    Permanent, // Any permanent
    Player,    // Any player
}

// Targets must be either a player or an object (this is what the spell/ability is actually targeting)
#[derive(Debug, Clone, PartialEq)]
pub enum TargetRefId {
    Player(PlayerId),
    Object(ObjectId)
}

// Represents an individual target reference 
#[derive(Debug, Clone, PartialEq)]
pub struct TargetRef {
    pub ref_id: TargetRefId,
}

impl TargetRef {
    pub fn permanent(object_id: ObjectId) -> Self {
        TargetRef {
            ref_id: TargetRefId::Object(object_id),
        }
    }

    pub fn player(player_id: PlayerId) -> Self {
        TargetRef { 
            ref_id: TargetRefId::Player(player_id) 
        }
    }
    
    pub fn is_player(&self) -> bool {
        matches!(self.ref_id, TargetRefId::Player(_))
    }
    
    pub fn is_object(&self) -> bool {
        matches!(self.ref_id, TargetRefId::Object(_))
    }
    
    pub fn get_player_id(&self) -> Option<PlayerId> {
        match self.ref_id {
            TargetRefId::Player(id) => Some(id),
            _ => None
        }
    }
    
    pub fn get_object_id(&self) -> Option<ObjectId> {
        match self.ref_id {
            TargetRefId::Object(id) => Some(id),
            _ => None
        }
    }
}

// Nested, compositional criteria system for advanced targeting
// Need to be able to handle targets like "attacking Zombie" or "activated or triggered ability you control" with grace
#[derive(Debug, Clone, PartialEq)]
pub enum TargetCriteria {
    // Card type criteria (creature/land/enchantment, supertypes and subtypes, and zones)
    CardType(CardType, TargetZone),
    Supertype(Supertype, TargetZone),
    Subtype(Subtype, TargetZone),

    // Special categories (e.g. Opponent)
    Category(TargetCategory),

    // Card states
    Tapped,
    Untapped,
    Attacking,
    Blocking,
    
    // Logical operations for composition
    And(Vec<TargetCriteria>),
    Or(Vec<TargetCriteria>),
    Not(Box<TargetCriteria>),
    
    // Other criteria as needed
    PowerToughness(PowerToughnessCriteria),
    ControlledBy(ControlCriteria),
}

// Helper structs to organize related criteria
#[derive(Debug, Clone, PartialEq)]
pub enum PowerToughnessCriteria {
    PowerEqual(i32),
    PowerLessThan(i32),
    PowerGreaterThan(i32),
    ToughnessEqual(i32),
    // etc.
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlCriteria {
    Controller(Option<PlayerId>), // None means "you" (the spell's controller)
    Owner(Option<PlayerId>),
}