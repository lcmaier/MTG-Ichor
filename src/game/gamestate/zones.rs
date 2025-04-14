use crate::game::gamestate::core::Game;
use crate::game::game_obj::GameObj;
use crate::utils::constants::id_types::{ObjectId, PlayerId};
use crate::utils::constants::zones::Zone;
use crate::game::player::Player;


impl Game {
    // Find an object by ID in a collection (e.g. battlefield, stack, hand, etc.)
    fn find_object_by_id(&self, objects: &Vec<GameObj>, object_id: &ObjectId) -> Option<usize> {
        objects.iter().position(|obj| match obj {
            GameObj::Card { id, .. } => id == object_id,
        })
    }

    // Remove an object by ID in a collection (e.g. battlefield, stack, hand, etc.)
    fn remove_object_by_id(objects: &mut Vec<GameObj>, object_id: &ObjectId) -> Option<GameObj> {
        let mut extracted = objects.extract_if(.., |obj| match obj {
            GameObj::Card { id, .. } => id == object_id,
        });

        // return the first (and only) object found, or None if not found
        extracted.next()
    }

    // get a reference to a Player struct from a PlayerId
    pub fn get_player_ref(&self, player_id: PlayerId) -> Result<&Player, String> {
        self.players.iter()
            .find(|player| player.id == player_id)
            .ok_or_else(|| format!("Player with ID {} not found", player_id))
    }

    // get a mutable reference to a Player struct from a PlayerId (identical to get_player_ref, but mutable)
    pub fn get_player_mut(&mut self, player_id: PlayerId) -> Result<&mut Player, String> {
        self.players.iter_mut()
            .find(|player| player.id == player_id)
            .ok_or_else(|| format!("Player with ID {} not found", player_id))
    }

    
    

    // general method to move an object from one zone to another
    pub fn move_object(&mut self,
                        source_zone: Zone,
                        object_id: ObjectId,
                        destination_zone: Zone,
                        source_player_id: Option<PlayerId>,
                        destination_player_id: Option<PlayerId>,
                        additional_effects: Option<fn(&mut GameObj)>) -> Result<(), String> { // last param is juuuuust in case
        
        // extract the object by its id from the source zone
        let object = match source_zone {
            // private zones that require player ID
            Zone::Hand | Zone::Library | Zone::Graveyard => {
                let pid = source_player_id.ok_or_else(||
                    format!("Player ID required to access zone {:?} for object ID {}", source_zone, object_id))?;
                
                let player = self.get_player_mut(pid)?;

                match source_zone {
                    Zone::Hand => Game::remove_object_by_id(&mut player.hand, &object_id),
                    Zone::Library => Game::remove_object_by_id(&mut player.library, &object_id),
                    Zone::Graveyard => Game::remove_object_by_id(&mut player.graveyard, &object_id),
                    _ => unreachable!(),
                }
            },
            // public zones that don't require player ID
            Zone::Battlefield => Game::remove_object_by_id(&mut self.battlefield, &object_id),
            Zone::Stack => Game::remove_object_by_id(&mut self.stack, &object_id),
            Zone::Exile => Game::remove_object_by_id(&mut self.exile, &object_id),
            Zone::Command => Game::remove_object_by_id(&mut self.command_zone, &object_id),
        };

        // check that we found the object
        if object.is_none() {
            return Err(format!("Object with ID {} not found in zone {:?}", object_id, source_zone));
        }

        // unwrap the object since we know it exists now
        let mut object = object.unwrap();

        // update the object's zone
        match &mut object {
            GameObj::Card { zone, .. } => *zone = destination_zone.clone()
        }

        // apply any additional effects to the object
        if let Some(effect) = additional_effects {
            effect(&mut object);
        }

        // place the object in its destination zone
        match destination_zone {
            // Private zones that require a player ID
            Zone::Hand | Zone::Library | Zone::Graveyard => {
                let player = self.get_player_mut(destination_player_id.unwrap())?;
                match destination_zone {
                    Zone::Hand => player.hand.push(object),
                    Zone::Library => player.library.push(object),
                    Zone::Graveyard => player.graveyard.push(object),
                    _ => unreachable!(),
                }
            },

            // Public zones that don't require a player ID
            Zone::Battlefield => self.battlefield.push(object),
            Zone::Stack => self.stack.push(object),
            Zone::Exile => self.exile.push(object),
            Zone::Command => self.command_zone.push(object),
        }
        
        Ok(())
    }
}