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
}

impl GameObject {
    /// Create a new game object in the specified zone
    pub fn new(card_data: Arc<CardData>, owner: PlayerId, zone: Zone) -> Self {
        GameObject {
            id: new_object_id(),
            owner,
            card_data,
            zone,
        }
    }

    /// Create a new game object in the library (most common creation path — building a deck)
    pub fn in_library(card_data: Arc<CardData>, owner: PlayerId) -> Self {
        Self::new(card_data, owner, Zone::Library)
    }
}
