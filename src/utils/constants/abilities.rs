// src/utils/constants/abilities.rs

use std::collections::HashMap;

use crate::{game::gamestate::Game, utils::{constants::id_types::{ObjectId, PlayerId}, mana::ManaType}};

#[derive(Debug, Clone, PartialEq)]
pub enum AbilityType {
    Mana,
    Activated,
    Triggered,
    Static,
    // Loyalty,
}

// Represents costs for activating abilities
#[derive(Debug, Clone, PartialEq)]
pub enum Cost {
    Tap,
    // Mana(Vec<ManaType>),
    // Add other costs later (sacrifice, etc.)
}

impl Cost {
    // Check if the cost can be paid
    pub fn can_pay(&self, game: &Game, permanent_id: ObjectId, player_id: PlayerId) -> Result<bool, String> {
        match self {
            Cost::Tap => {
                // we need to ensure that the associated permanent is untapped to pay a Tap cost
                // First, find the permanent
                let permanent = game.battlefield.iter()
                    .find(|obj| obj.id == permanent_id)
                    .ok_or_else(|| format!("Permanent with ID {} not found on the battlefield", permanent_id))?;

                // return true only if the permanent is untapped
                Ok(!permanent.state.tapped)
            },
        }
    }

    // Pay the cost (IMPORTANT: this method assumes you've already verified that the cost can be paid, undefined behavior when calling on an unpayable cost)
    pub fn pay(&self, game: &mut Game, permanent_id: ObjectId, player_id: PlayerId) -> Result<(), String> {
        match self {
            Cost::Tap => {
                // First, locate the permanent on the battlefield
                let mut_permanent = game.battlefield.iter_mut()
                    .find(|obj| obj.id == permanent_id)
                    .ok_or_else(|| format!("Permanent with ID {} not found on the battlefield", permanent_id))?;

                // pay the cost by tapping down the permanent
                mut_permanent.state.tapped = true;
                Ok(())
            },
        }
    }
}

// Base trait for all ability effects
pub trait Effect {
    fn apply(&self, game: &mut Game, controller_id: PlayerId, source_id: ObjectId) -> Result<(), String>;
}

// Ability definitions - These are NOT objects in the game
#[derive(Debug, Clone, PartialEq)]
pub struct AbilityDefinition {
    pub ability_type: AbilityType,
    pub costs: Vec<Cost>,
    pub effect_details: EffectDetails,
}

// Types of effects - will be expanded later
#[derive(Debug, Clone, PartialEq)]
pub enum EffectDetails  {
    //// MANA ABILITY EFFECTS
    ProduceMana {
        mana_produced: HashMap<ManaType, u64>,
    },
    // Add more effect types as needed
}

// Only activated/triggered abilities on the stack become ability objects
#[derive(Debug, Clone)]
pub struct AbilityOnStack {
    pub id: ObjectId,        // Only for stack objects
    pub source_id: ObjectId, // The object that has this ability 
    pub controller_id: PlayerId,
    pub effect_details: EffectDetails ,
}
