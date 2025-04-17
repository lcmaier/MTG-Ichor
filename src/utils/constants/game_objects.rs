// src/utils/constants/game_objects.rs
use std::collections::HashSet;
use crate::utils::constants::id_types::{ObjectId, PlayerId};
use crate::utils::constants::colors::Color;
use crate::utils::constants::abilities::AbilityDefinition;
use crate::utils::constants::card_types::{CardType, Supertype, Subtype};
use crate::utils::traits::zonestate::ZoneState;
use crate::utils::constants::combat::AttackTarget;

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
    pub abilities: Option<Vec<AbilityDefinition>>, 
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub loyalty: Option<i32>,
    pub defense: Option<i32>,
    pub hand_modifier: Option<i32>,
    pub life_modifier: Option<i32>,
}

// Base GameObj with common data that all states share
#[derive(Debug, Clone, PartialEq)]
pub struct GameObj<S: ZoneState> {
    pub id: ObjectId,
    pub owner: PlayerId,
    pub characteristics: Characteristics,
    pub state: S
}

// Different states for the zones with state-specific data to minimize game object overhead
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryState;
#[derive(Debug, Clone, PartialEq)]
pub struct HandState;
#[derive(Debug, Clone, PartialEq)]
pub struct BattlefieldState {
    pub tapped: bool,
    pub flipped: bool,
    pub face_down: bool,
    pub phased_out: bool,
    pub controller: PlayerId,
    // pub counters: HashMap<CounterType, u32> // to be implemented as soon as I create the CounterType
    
    // Optional *aspects* based on card type
    pub damageable: Option<DamageableAspect>,
    pub creature: Option<CreatureAspect>,
    // to be implemeneted when necessary
    // pub planeswalker: Option<PlaneswalkerAspect>,
    // pub battle: Option<BattleAspect>,
}

//// Aspects
// Aspect for objects that can have damage marked on them
#[derive(Debug, Clone, PartialEq)]
pub struct DamageableAspect {
    pub damage_marked: u32,
}

// Aspect for creature-specific fields
#[derive(Debug, Clone, PartialEq)]
pub struct CreatureAspect {
    pub summoning_sick: bool,
    pub power_modifier: i32,    // for temporary modifications
    pub toughness_modifier: i32, // same deal here

    // Combat state tracking
    pub attacking: Option<AttackingState>,
    pub blocking: Option<BlockingState>,
}

// Creature Aspect States
#[derive(Debug, Clone, PartialEq)]
pub struct AttackingState {
    pub target: AttackTarget
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockingState {
    pub blocking: Vec<ObjectId>, // Both the game and the creature itself need to keep track of what it's blocking
    pub max_can_block: u32,      // Max creatures this creature can block (I believe a Hundred-Handed One buffed with a few "extra block" abilities is the max--somewhere around to 105)
}
#[derive(Debug, Clone, PartialEq)]
pub struct StackState {
    pub controller: PlayerId,
    pub targets: Vec<String>, // placeholder for now, will need a Target type
}
#[derive(Debug, Clone, PartialEq)]
pub struct GraveyardState;
#[derive(Debug, Clone, PartialEq)]
pub struct ExileState;
#[derive(Debug, Clone, PartialEq)]
pub struct CommandState;