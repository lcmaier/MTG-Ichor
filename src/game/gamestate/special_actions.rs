// src/game/gamestate/special_actions.rs
use crate::game::gamestate::Game;
use crate::utils::constants::id_types::{ObjectId, PlayerId};
use crate::utils::constants::turns::PhaseType;
use crate::utils::constants::zones::Zone;
use crate::utils::constants::card_types::CardType;

impl Game {
    // Special Action: Play a Land from your hand (counts toward land drop for turn)
    pub fn play_land_from_hand(&mut self, player_id: PlayerId, card_id: ObjectId) -> Result<(), String> {
        
        // 1. Check if it's the player's turn
        if self.active_player_id != player_id {
            return Err("You can only play lands via special action on your turn".to_string());
        }

        // 2. Check if it is a main phase
        if self.phase.phase_type != PhaseType::Precombat && self.phase.phase_type != PhaseType::Postcombat {
            let msg = format!("You can only play a land during your main phase, the current phase is {:?}", self.phase);
            return Err(msg);
        }
        
        // 3. Check if the stack is empty
        if !self.stack.is_empty() {
            return Err("You can only play lands when the stack is empty".to_string());
        }

        // 4. Check if the player has priority
        if self.priority_player_id != player_id {
            return Err("You can only play lands when you have priority".to_string());
        }

        // 5. Get the player and ensure they actually have a land drop to spend
        let player = self.get_player_mut(player_id)?;
        if player.lands_played_this_turn >= player.max_lands_this_turn {
            return Err("You have no land drops remaining this turn".to_string());
        }

        // 6. Find and validate the card in hand
        let card = player.get_card_in_hand(card_id)
            .ok_or_else(|| format!("player.get_card_in_hand() call failed, card with ID {} not found in hand", card_id))?;

        if !card.has_card_type(&CardType::Land) {
            return Err("Selected card is not a land".to_string());
        }

        // 7. Remove card from hand
        let hand_card = player.remove_card_from_hand(card_id)?;

        // 8. Increment land drop count
        player.lands_played_this_turn += 1;

        // 9. Convert to battlefield state and add to the battlefield vec
        let battlefield_card = hand_card.to_battlefield(player_id);
        self.battlefield.insert(battlefield_card.id, battlefield_card);

        Ok(())
    }
}