// src/effects/mana_effects

use std::collections::HashMap;

use crate::utils::{constants::abilities::{AbilityDefinition, Cost}, mana::ManaType};
use crate::utils::constants::abilities::EffectDetails;

impl AbilityDefinition {
    // mana ability to tap the associated object (permanent, hopefully) for n mana of a specific type
    pub fn tap_for_mana(mana_type: ManaType, amount: u64) -> Self {
        let mut mana_map = HashMap::new();
        mana_map.insert(mana_type, amount);

        let mut cost_vec = Vec::new();
        cost_vec.push(Cost::Tap);

        AbilityDefinition { 
            ability_type: crate::utils::constants::abilities::AbilityType::Mana, 
            costs: cost_vec, 
            effect_details: EffectDetails::ProduceMana { 
                mana_produced: mana_map 
            } 
        }
    }
}