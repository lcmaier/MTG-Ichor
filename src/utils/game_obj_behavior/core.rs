// src/utils/game_obj_behavior/core.rs
use crate::utils::{
    constants::{
        card_types::{CardType, Subtype, Supertype}, 
        game_objects::{Characteristics, GameObj}, 
        id_types::{ObjectId, PlayerId}, 
        zones::Zone
    }, 
    traits::zonestate::ZoneState};

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

    // similar methods for subtype and supertype 
    // (consider Rime Tender, a creature with "{T}: Untap another target snow permanent.", or
    // Bladewing the Risen with "When ~ enters, you may return target Dragon permanent card from your graveyard to the battlefield."
    pub fn has_card_subtype(&self, subtype: &Subtype) -> bool {
        if let Some(subtypes) = &self.characteristics.subtype {
            subtypes.contains(subtype)
        } else {
            false
        }
    }
    pub fn has_card_supertype(&self, supertype: &Supertype) -> bool {
        if let Some(supertypes) = &self.characteristics.supertype {
            supertypes.contains(supertype)
        } else {
            false
        }
    }

}