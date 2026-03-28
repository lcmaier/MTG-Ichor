use uuid::Uuid;

/// Unique identifier for a game object (card, token, copy, ability on stack, etc.)
pub type ObjectId = Uuid;

/// Player identifier — index into the players array
pub type PlayerId = usize;

/// Unique identifier for an ability definition on a card
pub type AbilityId = Uuid;

/// Generate a new unique ObjectId
pub fn new_object_id() -> ObjectId {
    Uuid::new_v4()
}

/// Generate a new unique AbilityId
pub fn new_ability_id() -> AbilityId {
    Uuid::new_v4()
}
