use crate::events::event::GameEvent;
use crate::state::battlefield::BattlefieldEntity;
use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::Zone;

/// Centralized zone transition logic.
///
/// ALL object movement between zones goes through this module.
/// This is the single place where:
/// - Objects are removed from their old zone's collection
/// - Objects are added to their new zone's collection
/// - Zone-specific state is initialized or cleaned up
///
/// This replaces v1's scattered `to_battlefield()`, `to_graveyard()`, etc. methods.

impl GameState {
    /// Move a game object from one zone to another.
    ///
    /// This is the fundamental zone transition operation. All higher-level operations
    /// (draw, play land, cast spell, destroy, etc.) ultimately call this.
    pub fn move_object(&mut self, id: ObjectId, to: Zone) -> Result<(), String> {
        let from = {
            let obj = self.get_object(id)?;
            obj.zone
        };

        if from == to {
            return Ok(()); // no-op
        }

        // Remove from old zone's collection
        self.remove_from_zone_collection(id, from)?;

        // Clean up zone-specific state for the old zone
        self.cleanup_zone_state(id, from);

        // Add to new zone's collection
        self.add_to_zone_collection(id, to)?;

        // Initialize zone-specific state for the new zone
        self.init_zone_state(id, to)?;

        // Update the object's zone field
        let owner = self.get_object(id)?.owner;
        let obj = self.get_object_mut(id)?;
        obj.zone = to;

        // Emit zone change event
        self.events.emit(GameEvent::ZoneChange {
            object_id: id,
            owner,
            from,
            to,
        });

        Ok(())
    }

    /// Draw a card: move top of library to hand.
    pub fn draw_card(&mut self, player_id: PlayerId) -> Result<ObjectId, String> {
        let player = self.get_player(player_id)?;

        if player.library.is_empty() {
            // Rule 704.5b: player attempted to draw from empty library
            let player_mut = self.get_player_mut(player_id)?;
            player_mut.has_drawn_from_empty_library = true;
            return Err(format!("Player {} tried to draw from an empty library", player_id));
        }

        // Top of library = last element in the Vec
        let card_id = {
            let player = self.get_player(player_id)?;
            *player.library.last().unwrap()
        };

        self.move_object(card_id, Zone::Hand)?;
        Ok(card_id)
    }

    /// Draw N cards for a player
    pub fn draw_cards(&mut self, player_id: PlayerId, count: u64) -> Result<Vec<ObjectId>, String> {
        let mut drawn = Vec::new();
        for _ in 0..count {
            match self.draw_card(player_id) {
                Ok(id) => drawn.push(id),
                Err(e) => return Err(e),
            }
        }
        Ok(drawn)
    }

    /// Play a land to the battlefield (special action, not a spell).
    ///
    /// The `from` parameter specifies which zone the land is being played from.
    /// Normally this is `Zone::Hand`, but continuous effects can allow playing
    /// lands from other zones (e.g. graveyard via Crucible of Worlds).
    pub fn play_land(&mut self, player_id: PlayerId, card_id: ObjectId, from: Zone) -> Result<(), String> {
        let obj = self.get_object(card_id)?;
        if obj.zone != from {
            return Err(format!("Card is not in {:?}", from));
        }
        if obj.owner != player_id {
            return Err("Can only play your own lands".to_string());
        }
        if !obj.card_data.types.contains(&CardType::Land) {
            return Err("This card is not a land".to_string());
        }

        // Check land drop limit
        let player = self.get_player(player_id)?;
        if !player.can_play_land() {
            return Err("Already played maximum lands this turn".to_string());
        }

        // Move to battlefield
        self.move_object(card_id, Zone::Battlefield)?;

        // Increment land drop counter
        let player = self.get_player_mut(player_id)?;
        player.lands_played_this_turn += 1;

        Ok(())
    }

    // --- Internal helpers ---

    /// Remove an object ID from the zone's collection
    fn remove_from_zone_collection(&mut self, id: ObjectId, zone: Zone) -> Result<(), String> {
        match zone {
            Zone::Library => {
                let owner = self.get_object(id)?.owner;
                let player = self.get_player_mut(owner)?;
                if let Some(pos) = player.library.iter().position(|&x| x == id) {
                    player.library.remove(pos);
                    Ok(())
                } else {
                    Err(format!("Object {} not found in player {}'s library", id, owner))
                }
            }
            Zone::Hand => {
                let owner = self.get_object(id)?.owner;
                let player = self.get_player_mut(owner)?;
                if let Some(pos) = player.hand.iter().position(|&x| x == id) {
                    player.hand.remove(pos);
                    Ok(())
                } else {
                    Err(format!("Object {} not found in player {}'s hand", id, owner))
                }
            }
            Zone::Battlefield => {
                self.battlefield.remove(&id);
                Ok(())
            }
            Zone::Graveyard => {
                let owner = self.get_object(id)?.owner;
                let player = self.get_player_mut(owner)?;
                if let Some(pos) = player.graveyard.iter().position(|&x| x == id) {
                    player.graveyard.remove(pos);
                    Ok(())
                } else {
                    Err(format!("Object {} not found in player {}'s graveyard", id, owner))
                }
            }
            Zone::Stack => {
                if let Some(pos) = self.stack.iter().position(|&x| x == id) {
                    self.stack.remove(pos);
                    Ok(())
                } else {
                    Err(format!("Object {} not found on stack", id))
                }
            }
            Zone::Exile => {
                if let Some(pos) = self.exile.iter().position(|&x| x == id) {
                    self.exile.remove(pos);
                    Ok(())
                } else {
                    Err(format!("Object {} not found in exile", id))
                }
            }
            Zone::Command => {
                if let Some(pos) = self.command.iter().position(|&x| x == id) {
                    self.command.remove(pos);
                    Ok(())
                } else {
                    Err(format!("Object {} not found in command zone", id))
                }
            }
        }
    }

    /// Add an object ID to the zone's collection
    fn add_to_zone_collection(&mut self, id: ObjectId, zone: Zone) -> Result<(), String> {
        let owner = self.get_object(id)?.owner;

        match zone {
            Zone::Library => {
                let player = self.get_player_mut(owner)?;
                player.library.push(id);
                Ok(())
            }
            Zone::Hand => {
                let player = self.get_player_mut(owner)?;
                player.hand.push(id);
                Ok(())
            }
            Zone::Battlefield => {
                // BattlefieldEntity is created in init_zone_state
                Ok(())
            }
            Zone::Graveyard => {
                let player = self.get_player_mut(owner)?;
                player.graveyard.push(id);
                Ok(())
            }
            Zone::Stack => {
                self.stack.push(id);
                Ok(())
            }
            Zone::Exile => {
                self.exile.push(id);
                Ok(())
            }
            Zone::Command => {
                self.command.push(id);
                Ok(())
            }
        }
    }

    /// Initialize zone-specific state when entering a zone
    fn init_zone_state(&mut self, id: ObjectId, zone: Zone) -> Result<(), String> {
        if zone == Zone::Battlefield {
            let obj = self.get_object(id)?;
            let controller = obj.owner; // default controller is owner
            let entry = BattlefieldEntity::new(id, controller);
            self.battlefield.insert(id, entry);
        }
        Ok(())
    }

    /// Clean up zone-specific state when leaving a zone
    fn cleanup_zone_state(&mut self, id: ObjectId, zone: Zone) {
        if zone == Zone::Battlefield {
            self.battlefield.remove(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::game_state::GameState;
    use crate::types::card_types::*;
    use crate::types::mana::ManaType;
    use crate::types::zones::Zone;

    fn make_forest() -> crate::objects::card_data::CardData {
        CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Forest))
            .mana_ability_single(ManaType::Green)
            .build()
    }

    #[test]
    fn test_draw_card() {
        let mut game = GameState::new(2, 20);

        // Put a forest in player 0's library
        let forest = GameObject::in_library(make_forest(), 0);
        let forest_id = game.add_object(forest);
        game.players[0].library.push(forest_id);

        // Draw it
        let drawn = game.draw_card(0).unwrap();
        assert_eq!(drawn, forest_id);
        assert!(game.players[0].library.is_empty());
        assert_eq!(game.players[0].hand.len(), 1);
        assert_eq!(game.players[0].hand[0], forest_id);

        // Verify the object's zone was updated
        let obj = game.get_object(forest_id).unwrap();
        assert_eq!(obj.zone, Zone::Hand);
    }

    #[test]
    fn test_draw_from_empty_library() {
        let mut game = GameState::new(2, 20);
        let result = game.draw_card(0);
        assert!(result.is_err());
        assert!(game.players[0].has_drawn_from_empty_library);
    }

    #[test]
    fn test_play_land() {
        let mut game = GameState::new(2, 20);

        // Create a forest in hand
        let forest = GameObject::new(make_forest(), 0, Zone::Hand);
        let forest_id = game.add_object(forest);
        game.players[0].hand.push(forest_id);

        // Play it
        game.play_land(0, forest_id, Zone::Hand).unwrap();

        assert!(game.players[0].hand.is_empty());
        assert!(game.battlefield.contains_key(&forest_id));
        assert_eq!(game.players[0].lands_played_this_turn, 1);

        let obj = game.get_object(forest_id).unwrap();
        assert_eq!(obj.zone, Zone::Battlefield);

        // Should not be able to play a second land
        let forest2 = GameObject::new(make_forest(), 0, Zone::Hand);
        let forest2_id = game.add_object(forest2);
        game.players[0].hand.push(forest2_id);

        let result = game.play_land(0, forest2_id, Zone::Hand);
        assert!(result.is_err());
    }

    #[test]
    fn test_zone_transition_battlefield_to_graveyard() {
        let mut game = GameState::new(2, 20);

        // Create a forest on the battlefield
        let forest = GameObject::new(make_forest(), 0, Zone::Battlefield);
        let forest_id = game.add_object(forest);
        let entry = crate::state::battlefield::BattlefieldEntity::new(forest_id, 0);
        game.battlefield.insert(forest_id, entry);

        // Move to graveyard
        game.move_object(forest_id, Zone::Graveyard).unwrap();

        assert!(!game.battlefield.contains_key(&forest_id));
        assert_eq!(game.players[0].graveyard.len(), 1);
        assert_eq!(game.get_object(forest_id).unwrap().zone, Zone::Graveyard);
    }
}
