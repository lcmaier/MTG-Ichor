// src/game/gamestate/special_actions.rs
use crate::game::gamestate::Game;
use crate::utils::constants::id_types::{ObjectId, PlayerId};
use crate::game::game_obj::GameObj;
use crate::utils::constants::turns::Phase;
use crate::utils::constants::zones::Zone;
use crate::utils::constants::card_types::CardType;

impl Game {
    // Special Action: Play a Land from your hand
    pub fn play_land_from_hand(&mut self, player_id: PlayerId, card_id: ObjectId) -> Result<(), String> {

        // check if the stack is empty
        if !self.stack.is_empty() {
            return Err("Cannot play a land while the stack is not empty".to_string());
        }

        // check if the player has priority
        if self.priority_player_id != player_id {
            return Err("You do not have priority to play a land".to_string());
        }

        // check if it is a main phase
        // println!("(self.phase != Phase::Precombat) == {:?}, (self.phase != Phase::Postcombat) == {:?}", self.phase != Phase::Precombat, self.phase != Phase::Postcombat);
        if self.phase != Phase::Precombat && self.phase != Phase::Postcombat {
            let msg = format!("You can only play a land during your main phase, the current phase is {:?}", self.phase);
            return Err(msg);
        }

        // create scope for mutable Player reference, need to do checks before moving the land (which requires a mutable borrow)
        {
            // get the player
            let player = self.get_player_mut(player_id)?;

            // check if the player has already played their land(s) this turn
            if player.lands_played_this_turn >= player.max_lands_this_turn {
                return Err("Already played maximum number of lands this turn".to_string());
            }

            // verify the card exists in the player's hand and that it is a land
            let card_in_hand = player.hand.iter().find(|card| match card {
                GameObj::Card { id, .. } => *id == card_id
            });

            let is_land = match card_in_hand {
                Some(GameObj::Card { characteristics, .. }) => {
                    if let Some(card_types) = &characteristics.card_type {
                        card_types.contains(&CardType::Land)
                    } else {
                        false
                    }
                },
                None => return Err(format!("Card with ID {} not found in hand", card_id)),
            };

            if !is_land {
                return Err(format!("Card with ID {} is not a land", card_id));
            }
        }

        // now that we know the card is a land, that we've descoped the Player mutable reference, and that all the checks have passed, 
        // we can move the land from the player's hand to the battlefield
        self.move_object(
            Zone::Hand, 
            card_id, 
            Zone::Battlefield, 
            Some(player_id), 
            None, // battlefield is public zone, so no player ID needed
            None)?; // no additional effects for playing a land
        
        // increment the player's lands played this turn
        let player = self.get_player_mut(player_id)?;
        player.lands_played_this_turn += 1;

        Ok(())
    }
}