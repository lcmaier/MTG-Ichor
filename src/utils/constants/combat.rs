use crate::utils::constants::id_types::{ObjectId, PlayerId};

#[derive(Debug, Clone)]
pub struct AttackingCreature {
    pub creature_id: ObjectId,
    pub attack_target_id: ObjectId,
    pub attack_target_type: AttackTarget,
}
#[derive(Debug, Clone, PartialEq)]
pub enum AttackTarget {
    Player(PlayerId),
    Planeswalker(ObjectId),
    Battle(ObjectId),
}

#[derive(Debug, Clone)]
pub struct BlockingCreature {
    pub creature_id: ObjectId,
    pub blocking: Vec<ObjectId>, // some creatures can block multiple creatures
    pub max_can_block: u32,
}