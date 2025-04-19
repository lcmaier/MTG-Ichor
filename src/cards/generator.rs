// cards/generator.rs

use uuid::Uuid;

use crate::utils::constants::{game_objects::{GameObj, LibraryState}, id_types::PlayerId};
use super::registry::CARD_CHARACTERISTICS;

// generates game objects in vaious zones
pub struct ObjectGenerator;

impl ObjectGenerator {
    pub fn create_card_in_library(card_name: &str, owner: PlayerId) -> Result<GameObj<LibraryState>, String> {
        // acquire the card's characteristics from the registry
        if let Some(characteristics_fn) = CARD_CHARACTERISTICS.get(card_name) {
            Ok(GameObj { 
                id: Uuid::new_v4(), 
                owner, 
                characteristics: characteristics_fn(), 
                state: LibraryState {} }
            )
        } else {
            Err(format!("Name '{}' not found in card registry", card_name))
        }
    }

    // other zones as needed
}