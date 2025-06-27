// src/game/gamestate/event_handlers/damage_events

use crate::{game::gamestate::Game, utils::{constants::id_types::ObjectId, targeting::core::{TargetRef, TargetRefId}}};

impl Game {
    // Handle damage about to be dealt (for replacement effects)
    pub fn handle_damage_about_to_be_dealt(&mut self, source_id: ObjectId, target_ref: &TargetRef, amount: u64) -> Result<(), String> {
        // This is where damage replacement effects would be processed
        // For the alpha version, we can just pass these through without any changes
        return self.handle_damage_dealt(source_id, target_ref, amount)
    }

    // Handle damage being dealt
    pub fn handle_damage_dealt(&mut self, source_id: ObjectId, target_ref: &TargetRef, amount: u64) -> Result<(), String> {
        // Damage can be dealt to players and multiple types of permanents, but is handled differently in each case
        match &target_ref.ref_id {
            TargetRefId::Player(player_id) => {
                // Deal damage to a player
                let player = self.get_player_mut(*player_id)?;
                player.life_total -= amount as i64;
                println!("Player {} takes {} damage. Life total is now {}", player_id, amount, player.life_total);
                Ok(())
            },
            TargetRefId::Object(object_id) => {
                // Get the object from the battlefield (only objects on the battlefield can be dealt damage--you can't Lightning Bolt a Counterspell)
                let permanent = self.battlefield.get_mut(object_id)
                    .ok_or_else(|| format!("Object with ID {} not found on battlefield", object_id))?;

                // Deal damage to the permanent only if it's a damageable object (Creature, Planeswalker, or Battle)
                if let Some(creature) = &mut permanent.state.creature {
                    creature.damage_marked += amount as u32;

                    // Get the name for display
                    let default = &"Unknown".to_string();
                    let name = permanent.characteristics.name.as_ref()
                        .unwrap_or(default);
                
                    println!("Permanent {} takes {} damage. Marked damage is now {}", 
                            name, amount, creature.damage_marked);
                    Ok(())
                } else { // TODO: handle Planeswalkers and Battles
                    return Err(format!("Object with ID {} cannot be dealt damage", object_id));
                }
            }
        }
    }
}