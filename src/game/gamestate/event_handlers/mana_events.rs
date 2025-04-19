// src/game/gamestate/event_handlers/mana_events.rs

use std::collections::HashMap;

use crate::{game::gamestate::Game, utils::{constants::{abilities::{AbilityType, EffectDetails}, events::{EventHandler, GameEvent}, id_types::{ObjectId, PlayerId}, turns::{PhaseType, StepType}}, mana::ManaType}};

impl Game {
    pub fn handle_mana_ability_activated(&mut self, source_id: ObjectId, controller_id: PlayerId, ) -> Result<(), String> {
        // locate the permanent in the battlefield vector
        // Find the permanent to check conditions
        let permanent = self.battlefield.iter()
            .find(|obj| obj.id == source_id)
            .ok_or_else(|| format!("Permanent with ID {} not found on battlefield", source_id))?;

        // check controller -- you can't activate mana abilities of permanents you don't control
        if permanent.state.controller != controller_id {
            return Err(format!("handle_mana_ability_activated was called with controller_id {}, but the permanent it was called on belongs to a different player.", controller_id).to_string());
        }

        // get the ability and any costs to pay (I'd hope there's a cost for a mana ability sheesh)
        let (costs, mana_produced) = if let Some(abilities) = &permanent.characteristics.abilities {
            let mana_ability = abilities.iter()
                .find(|ability| ability.ability_type == AbilityType::Mana)
                .ok_or_else(|| "handle_mana_ability_activated called on permanent with no mana abilities".to_string())?;

            if let EffectDetails::ProduceMana { mana_produced } = &mana_ability.effect_details {
                (mana_ability.costs.clone(), mana_produced.clone())
            } else {
                return Err("Invalid mana ability effect".to_string());
            }
        } else {
            return Err("handle_mana_ability_activated called on permanent with no abilities".to_string());
        };

        // Ensure we can pay all costs
        for cost in &costs {
            if !cost.can_pay(self, source_id, controller_id)? {
                return Err(format!("Cannot pay cost {:?}", cost));
            }
        }

        // Passed cost check, now pay the costs
        for cost in costs {
            cost.pay(self, source_id, controller_id)?;
        }

        // Now that the costs are paid, add the ability's mana
        let mana_event = GameEvent::ManaAdded { 
            source_id, 
            player_id: controller_id, 
            mana_types: mana_produced 
        };

        // Process the mana event and return
        self.handle_event(&mana_event)?;
        Ok(())
    }

    

    // Handle adding mana to a player's mana pool
    pub fn handle_mana_added(&mut self, source_id: ObjectId, player_id: PlayerId, mana_types: &HashMap<ManaType, u64>) -> Result<(), String> {
        // Add the mana to the player's mana pool
        let player = self.get_player_mut(player_id)?;

        for (mana_type, amount) in mana_types {
            player.mana_pool.add_mana(mana_type.clone(), *amount);
            println!("Added {} {:?} mana to player {}'s mana pool", 
                    amount, mana_type, player_id);
        }

        Ok(())
    }

    // Handle end of phase
    pub fn handle_phase_ended(&mut self, phase_type: PhaseType) -> Result<(), String> {
        // Empty mana pools at the end of phases
        self.empty_mana_pools();
        Ok(())
    }
    
    // Handle end of step
    pub fn handle_step_ended(&mut self, step_type: StepType) -> Result<(), String> {
        // Empty mana pools at the end of steps
        self.empty_mana_pools();
        Ok(())
    }
    
    // Helper method to empty all players' mana pools
    pub fn empty_mana_pools(&mut self) {
        for player in &mut self.players {
            player.mana_pool.empty();
        }
    }
}