// src/game/gamestate/casting.rs

// this file contains logic for casting spells from various zones

use crate::utils::{constants::{
    card_types::CardType, game_objects::StackState, id_types::{ObjectId, PlayerId}, turns::PhaseType, zones::Zone
}, targeting::core::TargetRef};

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
        // TODO

        // Calculate and pay the mana cost
        // TODO

        // Remove the card from the hand and put it on the stack
        let player = self.get_player_mut(player_id)?;
        let hand_card = player.remove_card_from_hand(card_id)?;

        let stack_state = StackState {
            controller: player_id,
            targets,
        };

        // convert to stack object
        let stack_object = hand_card.to_stack(stack_state);

        // put into stack data structure
        self.stack.push(stack_object);

        Ok(())
    }
}