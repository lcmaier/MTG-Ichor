// src/utils/constants/game_objects.rs
use std::collections::HashSet;
use crate::utils::constants::id_types::{ObjectId, PlayerId};
use crate::utils::constants::colors::Color;
use crate::utils::constants::abilities::AbilityDefinition;
use crate::utils::constants::card_types::{CardType, Supertype, Subtype};
use crate::utils::targeting::core::TargetRef;
use crate::utils::traits::zonestate::ZoneState;
use crate::utils::constants::combat::AttackTarget;

use super::costs::ManaCost;

#[derive(Debug, Clone, PartialEq)]
pub struct Characteristics {
    // Official WOTC characteristics
    pub name: Option<String>,
    pub mana_cost: Option<ManaCost>,
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
    pub creature: Option<CreatureAspect>,
    // to be implemeneted when necessary
    // pub planeswalker: Option<PlaneswalkerAspect>,
    // pub battle: Option<BattleAspect>,
}

//// Aspects

// Aspect for creature-specific fields
#[derive(Debug, Clone, PartialEq)]
pub struct CreatureAspect {
    pub summoning_sick: bool,
    pub power_modifier: i32,    // for temporary modifications
    pub toughness_modifier: i32, // same deal here

    // Cached calculated current power/toughness
    pub current_power: i32,
    pub current_toughness: i32,
    pub damage_marked: u32, // Damage marked on the creature

    // Combat state tracking
    pub attacking: Option<AttackingState>,
    pub blocking: Option<BlockingState>,
}

// Creature Aspect States
#[derive(Debug, Clone, PartialEq)]
pub struct AttackingState {
    pub target: AttackTarget,
    pub is_blocked: bool, // Whether this creature is currently blocked (needed in the case where a blocker is declared and then somehow removed from combat (e.g. removal spell, blink spell, etc)), we don't want damage going through in that case
    pub blocked_by: Vec<ObjectId>, // IDs of creatures blocking this creature
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockingState {
    pub blocking: Vec<ObjectId>, // The creature(s) this is blocking
    pub max_can_block: u32, // How many creatures this can block (1 for most, u32::MAX for "any number")
}
#[derive(Debug, Clone, PartialEq)]
pub struct StackState {
    pub controller: PlayerId,
    pub targets: Vec<TargetRef>,
    pub stack_object_type: StackObjectType,
    pub source_id: Option<ObjectId>, // For abilities, the ID of the object that created them
}
#[derive(Debug, Clone, PartialEq)]
pub struct GraveyardState;
#[derive(Debug, Clone, PartialEq)]
pub struct ExileState;
#[derive(Debug, Clone, PartialEq)]
pub struct CommandState;


// enum for the different kinds of objects that can be on the stack
// e.g. a copy of a card resolving on the stack ceases to exist, while a card resolving on the stack goes to either the battlefield or graveyard
#[derive(Debug, Clone, PartialEq)]
pub enum StackObjectType {
    Spell,              // A card cast as a spell
    CopyOfSpell,        // A copy of a spell (e.g., created by Fork)
    ActivatedAbility,   // An activated ability of a permanent
    TriggeredAbility,   // A triggered ability that has triggered
    CopyOfAbility       // A copy of an ability (e.g., created by Strionic Resonator)
}