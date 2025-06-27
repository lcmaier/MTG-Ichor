// src/game/gamestate/event_handlers/destruction_like_events.rs

use crate::{game::gamestate::Game, utils::constants::{events::{DeathReason, GameEvent}, id_types::ObjectId}};

impl Game {
    /// Handle when a permanent is destroyed
    pub fn handle_permanent_destroyed(&mut self, permanent_id: ObjectId, reason: DeathReason) -> Result<(), String> {
        // Find and remove the permanent from the battlefield
        let permanent = self.battlefield.remove(&permanent_id)
            .ok_or_else(|| format!("Permanent with ID {} not found on battlefield", permanent_id))?;

        // Get the name for logging
        let name = permanent.characteristics.name.clone()
            .unwrap_or_else(|| format!("Permanent {}", permanent_id));

        // Get the owner before we consume the permanent
        let owner_id = permanent.owner;
        
        // Log based on reason
        match reason {
            DeathReason::LethalDamage => {
                println!("{} is destroyed due to lethal damage", name);
            },
            DeathReason::DestroyEffect => {
                println!("{} is destroyed due to a destroy effect", name);
            },
            _ => {
                println!("{} is destroyed (reason: {:?})", name, reason);
            }
        }

        // TODO: In the future, check for regeneration effects here
        // For now, just move to graveyard

        // Convert to graveyard object and add to owner's graveyard
        let graveyard_obj = permanent.to_graveyard();
        let owner = self.get_player_mut(owner_id )?;
        owner.graveyard.push(graveyard_obj);
        
        Ok(())
    }

    /// Handle when a creature dies from having 0 or less toughness
    pub fn handle_creature_zero_toughness(&mut self, creature_id: ObjectId) -> Result<(), String> {
        // Remove the permanent from the battlefield
        let permanent = self.battlefield.remove(&creature_id)
            .ok_or_else(|| format!("Creature with ID {} not found on battlefield", creature_id))?;

        // Get the name for logging
        let name = permanent.characteristics.name.clone()
            .unwrap_or_else(|| format!("Creature {}", creature_id));
        
        // Get the owner before we consume the permanent
        let owner_id = permanent.owner;
        
        println!("{} is put into graveyard due to having 0 or less toughness", name);

        // Convert to graveyard object and add to owner's graveyard
        let graveyard_obj = permanent.to_graveyard();
        let owner = self.get_player_mut(owner_id)?;
        owner.graveyard.push(graveyard_obj);
        
        Ok(())
    }
}