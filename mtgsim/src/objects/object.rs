use std::sync::Arc;

use crate::types::ids::{ObjectId, PlayerId, new_object_id};
use crate::types::zones::Zone;

use super::card_data::CardData;

/// A runtime game object — an instance of a card (or token, or copy) in the game.
///
/// This is the "live" representation. It tracks which card it is, who owns it,
/// and what zone it's currently in. All zone-specific mutable state (tapped, damage,
/// counters, etc.) lives in the GameState's zone containers, NOT here.
///
/// This separation means zone transitions are trivial (update `zone` field),
/// and zone-specific state is initialized/cleaned up by the engine's zone
/// transition logic in one centralized place.
#[derive(Debug, Clone)]
pub struct GameObject {
    /// Unique identity of this object in the game
    pub id: ObjectId,
    /// The player who owns this object (determines whose graveyard/library it goes to)
    pub owner: PlayerId,
    /// Reference to the immutable printed card definition (shared via Arc)
    pub card_data: Arc<CardData>,
    /// Current zone this object is in
    pub zone: Zone,
    /// True if this object is a token (created by an effect, not a real card)
    pub is_token: bool,
    /// True if this object is a copy of another object
    pub is_copy: bool,
}

impl GameObject {
    /// Create a new game object in the specified zone
    pub fn new(card_data: Arc<CardData>, owner: PlayerId, zone: Zone) -> Self {
        GameObject {
            id: new_object_id(),
            owner,
            card_data,
            zone,
            is_token: false,
            is_copy: false,
        }
    }

    /// Create a new game object in the library (most common creation path — building a deck)
    pub fn in_library(card_data: Arc<CardData>, owner: PlayerId) -> Self {
        Self::new(card_data, owner, Zone::Library)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;

    #[test]
    fn test_game_object_default_not_token() {
        let card = CardDataBuilder::new("Test Card").build();
        let obj = GameObject::new(card, 0, Zone::Hand);
        assert!(!obj.is_token);
    }

    #[test]
    fn test_game_object_default_not_copy() {
        let card = CardDataBuilder::new("Test Card").build();
        let obj = GameObject::new(card, 0, Zone::Hand);
        assert!(!obj.is_copy);
    }

    #[test]
    fn test_in_library_default_not_token_or_copy() {
        let card = CardDataBuilder::new("Test Card").build();
        let obj = GameObject::in_library(card, 0);
        assert!(!obj.is_token);
        assert!(!obj.is_copy);
    }
}
