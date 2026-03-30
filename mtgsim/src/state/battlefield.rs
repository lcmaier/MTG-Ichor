use crate::types::ids::{ObjectId, PlayerId};

/// Battlefield-specific state for a permanent.
///
/// This is stored separately from the GameObject itself — the object just knows
/// it's on the battlefield (via its `zone` field), and the engine looks up its
/// BattlefieldEntity here for mutable state like tapped/damage/counters.
#[derive(Debug, Clone)]
pub struct BattlefieldEntity {
    pub object_id: ObjectId,
    pub controller: PlayerId,

    /// Timestamp for the layer system (rule 613.7).
    /// Permanents that entered the battlefield earlier have lower timestamps.
    /// Used to order continuous effects within the same layer/sublayer.
    pub timestamp: u64,

    // Permanent state
    pub tapped: bool,
    pub flipped: bool,
    pub face_down: bool,
    pub phased_out: bool,
    pub summoning_sick: bool,

    // Creature-specific (only meaningful if the permanent is a creature)
    pub damage_marked: u32,
    /// Set when this creature is dealt damage by a source with deathtouch.
    /// Checked in SBA 704.5g: any nonzero damage from deathtouch is lethal.
    /// Cleared in cleanup alongside damage_marked.
    pub damaged_by_deathtouch: bool,
    pub power_modifier: i32,
    pub toughness_modifier: i32,

    // Combat state (transient, cleared at end of combat)
    pub attacking: Option<AttackingInfo>,
    pub blocking: Option<BlockingInfo>,

    // Counters (future: HashMap<CounterType, u32>)
}

#[derive(Debug, Clone)]
pub struct AttackingInfo {
    pub target: AttackTarget,
    pub is_blocked: bool,
    pub blocked_by: Vec<ObjectId>,
}

#[derive(Debug, Clone)]
pub struct BlockingInfo {
    pub blocking: Vec<ObjectId>,
}

#[derive(Debug, Clone)]
pub enum AttackTarget {
    Player(PlayerId),
    Planeswalker(ObjectId),
    Battle(ObjectId),
}

impl BattlefieldEntity {
    pub fn new(object_id: ObjectId, controller: PlayerId, timestamp: u64) -> Self {
        BattlefieldEntity {
            object_id,
            controller,
            timestamp,
            tapped: false,
            flipped: false,
            face_down: false,
            phased_out: false,
            summoning_sick: true,
            damage_marked: 0,
            damaged_by_deathtouch: false,
            power_modifier: 0,
            toughness_modifier: 0,
            attacking: None,
            blocking: None,
        }
    }

    /// Clear combat state (called at end of combat step)
    pub fn clear_combat_state(&mut self) {
        self.attacking = None;
        self.blocking = None;
    }
}
