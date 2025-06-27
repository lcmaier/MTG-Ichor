// src/utils/game_obj_behavior/creatures.rs

use crate::{game::gamestate::Game, utils::constants::id_types::ObjectId};

impl Game {
    // Call this whenever power and toughness are directly modified (so no counters, only pure stat changes)
    fn update_creature_stats(&mut self, creature_id: ObjectId) -> Result<(), String> {
        // Locate the creature on the battlefield
        // NOTE: This assumes stat changes can only happen to creatures on the battlefield, which is true in paper but not for alchemy
        // TODO: Update this to handle alchemy ("perpetual" stat changes can happen in hand, library, etc)
        let creature = self.battlefield.get_mut(&creature_id)
            .ok_or_else(|| format!("Creature with ID {} not found on battlefield", creature_id))?;

        // Skip if this isn't a creature/has no P/T
        let base_power = match creature.characteristics.power {
            Some(p) => p,
            None => return Ok(()),
        };

        let base_toughness = match creature.characteristics.toughness {
            Some(t) => t,
            None => return Ok(()),
        };

        // Update cached stats if we have a creature aspect
        if let Some(creature_aspect) = &mut creature.state.creature {
            // start with base stats
            let mut current_power = base_power;
            let mut current_toughness = base_toughness;

            // Add modifiers
            current_power += creature_aspect.power_modifier;
            current_toughness += creature_aspect.toughness_modifier;

            // Add counter modifiers to P/T
            // Not yet implemented...

            // Update the cached values in the creature aspect
            creature_aspect.current_power = current_power;
            creature_aspect.current_toughness = current_toughness;
        }

        Ok(())
    }

    // Wrapper methods to update power or toughness and call update_creature_stats
    fn modify_creature_power(&mut self, creature_id: ObjectId, amount: i32) -> Result<(), String> {
        let creature = self.battlefield.get_mut(&creature_id)
            .ok_or_else(|| format!("Creature with ID {} not found on battlefield", creature_id))?;

        // Check if the creature has a creature aspect
        if let Some(creature_aspect) = &mut creature.state.creature {
            // Update the power modifier
            creature_aspect.power_modifier += amount;
        } else {
            return Err(format!("Creature with ID {} does not have a creature aspect", creature_id));
        }

        // update cached values
        self.update_creature_stats(creature_id)?;
        Ok(())
    }

    fn modify_creature_toughness(&mut self, creature_id: ObjectId, amount: i32) -> Result<(), String> {
        let creature = self.battlefield.get_mut(&creature_id)
            .ok_or_else(|| format!("Creature with ID {} not found on battlefield", creature_id))?;

        // Check if the creature has a creature aspect
        if let Some(creature_aspect) = &mut creature.state.creature {
            // Update the toughness modifier
            creature_aspect.toughness_modifier += amount;
        } else {
            return Err(format!("Creature with ID {} does not have a creature aspect", creature_id));
        }

        // update cached values
        self.update_creature_stats(creature_id)?;
        Ok(())
    }

    // Getters that don't need to recalculate
    fn get_creature_power(&self, creature_id: ObjectId) -> Result<i32, String> {
        let creature = self.battlefield.get(&creature_id)
            .ok_or_else(|| format!("Creature with ID {} not found on battlefield", creature_id))?;
        
        if let Some(aspect) = &creature.state.creature {
            Ok(aspect.current_power)
        } else {
            Err(format!("Object with ID {} is not a creature", creature_id))
        }
    }

    fn get_creature_toughness(&self, creature_id: ObjectId) -> Result<i32, String> {
        let creature = self.battlefield.get(&creature_id)
            .ok_or_else(|| format!("Creature with ID {} not found on battlefield", creature_id))?;
        
        if let Some(aspect) = &creature.state.creature {
            Ok(aspect.current_toughness)
        } else {
            Err(format!("Object with ID {} is not a creature", creature_id))
        }
    }
}