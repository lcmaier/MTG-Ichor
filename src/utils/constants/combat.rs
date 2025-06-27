use crate::utils::{constants::id_types::{ObjectId, PlayerId}, targeting::core::TargetCriteria};

#[derive(Debug, Clone)]
pub struct AttackDeclaration {
    pub attacker_id: ObjectId,
    pub target: AttackTarget,
}

#[derive(Debug, Clone)]
pub struct BlockDeclaration {
    pub blocker_id: ObjectId,
    pub attacker_id: ObjectId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttackTarget {
    Player(PlayerId),
    Planeswalker(ObjectId),
    Battle(ObjectId),
}

// Unified target type for damage assignment
#[derive(Debug, Clone, PartialEq)]
pub enum DamageRecipient {
    Creature(ObjectId),
    Player(PlayerId),
    Planeswalker(ObjectId),
    Battle(ObjectId),
}

// For tracking combat damage assignment
#[derive(Debug, Clone)]
pub struct CombatDamageAssignment {
    pub source_id: ObjectId,
    pub target_id: DamageRecipient,
    pub amount: u32,
    pub is_first_strike: bool,
    pub is_trample: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockingRequirement {
    // Requires at least N creatures to block (e.g. menace keyword is defined as setting this to 2, there are several creatures that need 3 or more blockers, etc)
    MinimumBlockers(u32),
    
    // Requires blocking by a specific type of creature or other criteria
    RequiredBlockerType(TargetCriteria),

    // Can't be blocked by a specific criteria of creature (e.g. flying keyword means "can't be blocked by creatures without flying or reach", or "can't be blocked by creatures with flying")
    CannotBeBlockedBy(TargetCriteria),
}

#[derive(Debug, Clone)]
pub struct AttackingCreature {
    pub creature_id: ObjectId,
    pub attack_target_id: ObjectId,
    pub attack_target_type: AttackTarget,
    pub blocking_requirements: Vec<BlockingRequirement>, // for particular requirements like "must be blocked by a Dalek" or "must be blocked by 2 or more creatures" (menace)
}

#[derive(Debug, Clone)]
pub struct BlockingCreature {
    pub creature_id: ObjectId,
    pub blocking: Vec<ObjectId>, // some creatures can block multiple creatures
    pub max_can_block: u32,
}
