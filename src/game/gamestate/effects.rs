// src/game/gamestate/effects.rs

use crate::utils::{constants::abilities::EffectDetails, targeting::requirements::TargetingRequirement};

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
}