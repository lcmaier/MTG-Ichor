use crate::utils::constants::id_types::{ObjectId, PlayerId};

pub struct AttackingCreature {
    pub creature_id: ObjectId,
    pub attack_target_id: ObjectId,
    pub attack_target_type: AttackTarget,
}

pub enum AttackTarget {
    Player(PlayerId),
    Planeswalker(ObjectId),
    Battle(ObjectId),
}

pub struct BlockingCreature {
    pub creature_id: ObjectId,
    pub blocking: Vec<ObjectId>, // some creatures can block multiple creatures
    pub max_can_block: u32,
}