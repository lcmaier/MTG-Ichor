// src/utils/game_obj_behavior/core.rs
use crate::utils::{constants::{card_types::CardType, game_objects::{Characteristics, GameObj}, id_types::{ObjectId, PlayerId}, zones::Zone}, traits::zonestate::ZoneState};

// Common methods for all GameObjs regardless of state
impl<S: ZoneState> GameObj<S> {
    fn id(&self) -> ObjectId { self. id }
    fn owner(&self) -> PlayerId { self.owner }
    fn characteristics(&self) -> &Characteristics { &self.characteristics }

    // check if this object has a specific card type
    pub fn has_card_type(&self, card_type: &CardType) -> bool {
        if let Some(types) = &self.characteristics.card_type {
            types.contains(card_type)
        } else {
            false
        }
    }

    // Get the current zone based on the state
    pub fn zone(&self) -> Zone {
        todo!()
    }


}