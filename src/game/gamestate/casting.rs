// src/game/gamestate/casting.rs

// this file contains logic for casting spells from various zones

use uuid::Uuid;

use crate::utils::{constants::{
    abilities::AbilityType, card_types::CardType, costs::Cost, game_objects::{Characteristics, GameObj, StackObjectType, StackState}, id_types::{ObjectId, PlayerId}, turns::PhaseType, zones::Zone
}, targeting::{core::TargetRef, requirements::TargetingRequirement}};

use super::Game;

impl Game {
    pub fn cast_spell(&mut self, player_id: PlayerId, card_id: ObjectId, zone: Zone, targets: Vec<TargetRef>) -> Result<(), String> {
        // can only cast spells when you have priority
        if self.priority_player_id != player_id {
            return Err("You can only cast spells when you have priority".to_string());
        }

        match zone {
            Zone::Hand => self.cast_spell_from_hand(player_id, card_id, targets),
            // other casting locations to be implemented as needed
            _ => Err(format!("Cannot cast from {:?} zone", zone))
    }

    }
    
    pub fn cast_spell_from_hand(&mut self, player_id: PlayerId, card_id: ObjectId, targets: Vec<TargetRef>) -> Result<(), String> {
        // Get the player and find the card
        let player = self.get_player_ref(player_id)?;
        let card = player.get_card_in_hand(card_id)
            .ok_or_else(|| format!("Card with ID {} not found in hand", card_id))?;

        // Verify it's a spell and not a land (you can't cast a land)
        if card.has_card_type(&CardType::Land) {
            return Err("Lands cannot be cast".to_string());
        }

        // spells that aren't instants are subject to sorcery-speed timing restrictions (rule 117.1a)
        if !card.has_card_type(&CardType::Instant) {
            // stack must be empty and it must be that player's main phase
            if self.active_player_id != player_id || 
                !self.stack.is_empty() || 
                (self.phase.phase_type != PhaseType::Precombat && self.phase.phase_type != PhaseType::Postcombat) {
                    return Err("You can only cast non-instant spells during your main phase when the stack is empty".to_string());
                }
        }

        // Validate targets, if any
        if let Some(abilities) = &card.characteristics.abilities {
            for ability in abilities {
                if ability.ability_type == AbilityType::Spell {
                    // Get targeting requirements from the ability's effect
                    let target_requirements = self.get_targeting_requirements(&ability.effect_details);
                    
                    // Validate that the provided targets satisfy the requirements
                    if !self.validate_targets(&target_requirements, &targets, player_id) {
                        return Err("Invalid targets for this spell".to_string());
                    }
                }
            }
        }

        // Calculate and pay the mana cost
        if let Some(mana_cost) = &card.characteristics.mana_cost {
            // Create a dummy Cost::Mana to reuse existing cost payment logic
            let mana_cost_to_pay = Cost::Mana(mana_cost.clone());
            
            // Check if player can pay
            if !mana_cost_to_pay.can_pay(self, player_id, None)? {
                return Err("You don't have enough mana to cast this spell".to_string());
            }
            
            // Pay the cost
            mana_cost_to_pay.pay(self, player_id, None)?;
        }

        // Remove the card from the hand and put it on the stack
        let player = self.get_player_mut(player_id)?;
        let hand_card = player.remove_card_from_hand(card_id)?;

        let stack_state = StackState {
            controller: player_id,
            targets,
            stack_object_type: StackObjectType::Spell,
            source_id: None,
        };

        // convert to stack object
        let stack_object = hand_card.to_stack(stack_state);

        // put into stack data structure
        self.stack.push(stack_object);

        println!("Spell cast successfully and placed on the stack");
        Ok(())
    }

    pub fn activate_ability_on_battlefield(&mut self, player_id: PlayerId, permanent_id: ObjectId, ability_id: Uuid, ability_text: String, targets: Vec<TargetRef>) -> Result<(), String> {
        // Check if player has priority
        if self.priority_player_id != player_id {
            return Err("You can only activate abilities when you have priority".to_string());
        }

        // Find the permanent
        let permanent = self.battlefield.iter()
            .find(|obj| obj.id == permanent_id)
            .ok_or_else(|| format!("Permanent with ID {} not found on battlefield", permanent_id))?.clone();


        // Check controller
        if permanent.state.controller != player_id {
            return Err("You can only activate abilities of permanents you control".to_string());
        }

        // Get the ability
        let ability = if let Some(abilities) = &permanent.characteristics.abilities {
            if let Some(matched_ability) = abilities.iter().find(|a| a.id == ability_id) {
                matched_ability
            } else {
                return Err(format!("Ability id {} out of bounds", ability_id));
            }
        } else {
            return Err("Permanent has no abilities".to_string());
        };

        // Ensure it's an activated ability
        if ability.ability_type != AbilityType::Activated {
            return Err("This is not an activated ability".to_string());
        }
        
        // TODO: Check timing restrictions (e.g., sorcery speed). By default abilities can be activated at instant speed

        // Validate targets
        let target_requirements = self.get_targeting_requirements(&ability.effect_details);
        if !self.validate_targets(&target_requirements, &targets, player_id) {
            return Err("Invalid targets for this ability".to_string());
        }

        // Check and pay all costs
        for cost in &ability.costs {
            if !cost.can_pay(self, player_id, Some(permanent_id))? {
                return Err(format!("Cannot pay cost {:?}", cost));
            }
        }

        for cost in &ability.costs {
            cost.pay(self, player_id, Some(permanent_id))?;
        }


        // Create characteristics
        let ability_characteristics = Characteristics {
            name: None,
            mana_cost: None,
            color: None,
            color_indicator: None,
            card_type: None,
            supertype: None,
            subtype: None,
            rules_text: Some(ability_text),
            abilities: Some(vec![ability.clone()]), // only things the stack object inherits are the text and ability of the associated activated ability
            power: None,
            toughness: None,
            loyalty: None,
            defense: None,
            hand_modifier: None,
            life_modifier: None
        };

        // Create the stack object
        let stack_object = GameObj {
            id: Uuid::new_v4(),
            owner: player_id,
            characteristics: ability_characteristics,
            state: StackState {
                controller: player_id,
                targets,
                stack_object_type: StackObjectType::ActivatedAbility,
                source_id: Some(permanent_id),
            }
        };

        // and push onto the stack
        self.stack.push(stack_object);

        println!("Ability activated successfully and placed on the stack");
        Ok(())
    }

    fn validate_targets(&self, requirements: &Vec<Vec<TargetingRequirement>>, targets: &Vec<TargetRef>, caster_id: PlayerId) -> bool {
        // If there are no targeting requirements, no targets should be provided
        if requirements.is_empty() {
            return targets.is_empty();
        }

        // For the alpha, we assume there's only a single requirement
        // TODO: write requirement matching logic for multirequirement effects
        if requirements.len() == 1 && requirements[0].len() == 1 {
            let req = &requirements[0][0];

            // ensure the number of targets is correct
            if targets.len() < req.min_targets as usize || targets.len() > req.max_targets as usize {
                return false;
            }
            
            // Verify each target satisfies the criteria
            for target in targets {
                if !req.criteria.is_satisfied_by(self, target, caster_id) {
                    return false;
                }
            }
            return true;
        }
        panic!("Multirequirement spells not yet supported!")
    }
}