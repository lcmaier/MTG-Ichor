// src/game/gamestate/effects/zone_effects.rs
use crate::{game::gamestate::Game, utils::constants::{id_types::{ObjectId, PlayerId}, zones::Zone}};

impl Game {
    // General Effect method: Put an object from a source zone onto the battlefield
    pub fn _put_object_onto_battlefield(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        source_zone: Zone,
        tapped: bool,
        controller: PlayerId,
    ) -> Result<(), String> {
        match source_zone {
            Zone::Hand => {
                // get the player struct
                let player = self.get_player_mut(player_id)?;

                // Remove the card object from the player's hand
                let hand_card = player.remove_card_from_hand(object_id)?;

                // Convert to the battlefield state
                let mut battlefield_obj = hand_card.to_battlefield(controller);

                // Apply tapped state if specified by the calling effect
                if tapped {
                    battlefield_obj.state.tapped = true;
                }

                // Finally, add the battlefield_obj to the battlefield vec
                self.battlefield.push(battlefield_obj);

                Ok(())
            },
            // will implement other source zones as they become needed
            _ => Err(format!("Moving objects from {:?} to battlefield not yet implemented", source_zone))
        }
    }

    // CONVENIENCE METHODS
    
}