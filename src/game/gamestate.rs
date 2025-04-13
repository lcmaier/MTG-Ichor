// src/game/gamestate.rs
use crate::game::player::Player;
use crate::game::game_obj::GameObj;
use crate::utils::constants::turns::{Phase, Step};
use crate::utils::constants::zones::Zone;
use crate::utils::constants::id_types::{ObjectId, PlayerId};
use crate::utils::constants::card_types::CardType;

// game struct
pub struct Game {
    pub players: Vec<Player>,
    pub active_player_id: usize, // the active player is the one whose turn it is (by definition), so this doubles as a turn player index
    pub priority_player_id: usize,
    pub turn_number: u32,
    pub phase: Phase,
    pub step: Option<Step>,

    // global zones (Player zones like hand, library, graveyard are within Player struct)
    pub stack: Vec<GameObj>, // stack of objects (spells, abilities, etc.)
    pub battlefield: Vec<GameObj>, // battlefield objects (creatures, enchantments, tokens, etc.)
    pub exile: Vec<GameObj>,
    pub command_zone: Vec<GameObj>,
}

// Basic Methods
impl Game {
    // Create a new game
    pub fn new() -> Self {
        Game {
            players: Vec::new(),
            active_player_id: 0,
            priority_player_id: 0,
            turn_number: 0,
            phase: Phase::Beginning,
            step: None, // None to denote pregame (mulligans, pregame actions, etc.)
            stack: Vec::new(),
            battlefield: Vec::new(),
            exile: Vec::new(),
            command_zone: Vec::new(),
        }
    }

    // Helper to get card ID from index in a specific zone -- this is the ONLY place we should be using indexes to access cards, ObjectId everywhere else
    pub fn get_card_id_from_index(&self, player_id: PlayerId, zone: &Zone, index: usize) -> Result<ObjectId, String> {
        match zone {
            Zone::Hand => {
                let player = self.get_player_ref(player_id)?;
                if index >= player.hand.len() {
                    return Err(format!("Index {} out of bounds for hand", index));
                }
                match &player.hand[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
            Zone::Library => {
                let player = self.get_player_ref(player_id)?;
                if index >= player.library.len() {
                    return Err(format!("Index {} out of bounds for library", index));
                }
                match &player.library[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
            Zone::Graveyard => {
                let player = self.get_player_ref(player_id)?;
                if index >= player.graveyard.len() {
                    return Err(format!("Index {} out of bounds for graveyard", index));
                }
                match &player.graveyard[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
            Zone::Battlefield => {
                if index >= self.battlefield.len() {
                    return Err(format!("Index {} out of bounds for battlefield", index));
                }
                match &self.battlefield[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
            Zone::Stack => {
                if index >= self.stack.len() {
                    return Err(format!("Index {} out of bounds for stack", index));
                }
                match &self.stack[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
            Zone::Exile => {
                if index >= self.exile.len() {
                    return Err(format!("Index {} out of bounds for exile", index));
                }
                match &self.exile[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
            Zone::Command => {
                if index >= self.command_zone.len() {
                    return Err(format!("Index {} out of bounds for command zone", index));
                }
                match &self.command_zone[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
        }
    }
}





// Zone Management Methods
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


// Special Action Methods
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
        // if self.phase != Phase::Precombat || self.phase != Phase::Postcombat {
        //     return Err("You can only play a land during your main phase".to_string());
        // }

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

