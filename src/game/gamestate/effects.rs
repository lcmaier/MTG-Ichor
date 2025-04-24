// src/game/gamestate/effects.rs

use crate::utils::{constants::{abilities::EffectDetails, events::{EventHandler, GameEvent}, id_types::{ObjectId, PlayerId}}, targeting::{core::TargetRef, requirements::TargetingRequirement}};

use super::Game;

impl Game {
    // get all targeting requirements for an effect
    pub fn get_targeting_requirements(&self, effect: &EffectDetails) -> Vec<Vec<TargetingRequirement>> {
        match effect {
            // Easy ones -- non-recursive (one-shot) effects
            EffectDetails::DealDamage { amount: _, target_requirement } => {
                if let Some(req) = target_requirement {
                    vec![vec![req.clone()]]
                } else {
                    vec![]
                }
            },
            EffectDetails::Conditional { condition: _, if_true, if_false } => {
                let mut all_requirements = Vec::new();

                // Add the true branch requirements (guaranteed to exist)
                all_requirements.extend(self.get_targeting_requirements(if_true));

                // Add the false branch if it exists
                if let Some(false_effect) = if_false {
                    all_requirements.extend(self.get_targeting_requirements(false_effect));
                }

                all_requirements
            },
            // other effects don't have targets
            _ => vec![],
        }
    }


    // Process an effect based on its type
    pub fn process_effect(&mut self, effect: &EffectDetails, targets: &Vec<TargetRef>, controller_id: PlayerId, source_id: Option<ObjectId>) -> Result<(), String> {
        match effect {
            EffectDetails::DealDamage { amount, target_requirement: _ } => { // we ignore target_requirements bc if we're processing an effect, they've already been checked
                // Ensure we have a target
                if targets.is_empty() {
                    return Err("Damage effect requires a target but none was provided".to_string());
                }

                // unwrap the source ID, since this is an effect that always(?) has a source (either a permanent on the battlefield or a spell/ability on the stack--all GameObjs)
                let obj_id = source_id.unwrap();

                // Create an "about to deal damage" event (for replacement effects) for all targets of the damage effect
                for target in targets {
                    let predamage_event = GameEvent::DamageAboutToBeDealt { source_id: obj_id, target_ref: target.clone(), amount: *amount };
                    // process the "about to deal damage" event
                    self.handle_event(&predamage_event)?;

                    // then create the actual damage event
                    let damage_event = GameEvent::DamageDealt { source_id: obj_id, target_ref: target.clone(), amount: *amount };
                    // and process it
                    self.handle_event(&damage_event)?;
                }

                Ok(())
            },
            EffectDetails::Sequence(effects) => {
                // process each effect in sequence
                for sub_effect in effects {
                    self.process_effect(sub_effect, targets, controller_id, source_id)?;
                }
                Ok(())
            },
            _ => todo!()
        }
    }
}