use std::collections::HashMap;
use crate::engine::layers::types::Timestamp;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::effects::CounterType;

/// The counters of one kind on a permanent, with the timestamp CR 613.7c gives
/// them.
///
/// > Each counter receives a timestamp as it's put on an object or player. If
/// > that object or player already has a counter of that kind on it, each
/// > counter of that kind receives a new timestamp identical to that of the new
/// > counter.
///
/// The second sentence is what makes a single timestamp per kind *exact*: two
/// counters of the same kind can never carry different timestamps, so there is
/// nothing a per-counter list would express that this does not.
///
/// Read by the layer system: keyword counters (CR 122.1b) are layer 6 effects
/// and interleave with that layer's registry rows by timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CounterStack {
    pub count: u32,
    pub timestamp: Timestamp,
}

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
    /// The turn number when this permanent entered the battlefield.
    pub entered_battlefield_turn: u32,
    /// The turn number when the current controller gained control.
    /// Used for summoning sickness (CR 302.6): sick if control was gained on or
    /// after the turn the controller's most recent turn began — see
    /// `oracle::characteristics::has_summoning_sickness`, which owns the
    /// comparison. Convention: 0 = pregame (rule 103.6 Leylines).
    pub controller_since_turn: u32,

    // Creature-specific (only meaningful if the permanent is a creature)
    pub damage_marked: u32,
    /// Set when this creature is dealt damage by a source with deathtouch.
    /// Checked in SBA 704.5g: any nonzero damage from deathtouch is lethal.
    /// Cleared in cleanup alongside damage_marked.
    pub damaged_by_deathtouch: bool,

    // Combat state (transient, cleared at end of combat)
    pub attacking: Option<AttackingInfo>,
    pub blocking: Option<BlockingInfo>,

    // Counters (rule 122)
    pub counters: HashMap<CounterType, CounterStack>,

    /// The value of X chosen when this permanent was cast (rule 107.3f).
    /// Carried from StackEntry on resolution. None for non-X spells.
    pub x_value: Option<u64>,

    // Attachment tracking (rule 301.5, 303.4)
    /// The permanent this is attached to (for Auras, Equipment, Fortifications).
    pub attached_to: Option<ObjectId>,
    /// Permanents attached to this one (Auras, Equipment, Fortifications targeting this).
    pub attached_by: Vec<ObjectId>,
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
    pub fn new(object_id: ObjectId, controller: PlayerId, timestamp: u64, current_turn: u32) -> Self {
        BattlefieldEntity {
            object_id,
            controller,
            timestamp,
            tapped: false,
            flipped: false,
            face_down: false,
            phased_out: false,
            entered_battlefield_turn: current_turn,
            controller_since_turn: current_turn,
            damage_marked: 0,
            damaged_by_deathtouch: false,
            attacking: None,
            blocking: None,
            counters: HashMap::new(),
            x_value: None,
            attached_to: None,
            attached_by: Vec::new(),
        }
    }

    /// Clear combat state (called at end of combat step)
    pub fn clear_combat_state(&mut self) {
        self.attacking = None;
        self.blocking = None;
    }

    /// Add `n` counters of the given type, timestamped per CR 613.7c.
    ///
    /// Prefer `GameState::add_counters`, which allocates the timestamp for you.
    /// This is the low-level form, for callers that already hold one.
    ///
    /// CR 613.7c's second sentence — "if that object already has a counter of
    /// that kind on it, each counter of that kind receives a new timestamp
    /// identical to that of the new counter" — is the `entry.timestamp =`
    /// below, and it is also why one timestamp per *kind* is exact rather than
    /// a simplification: counters of one kind can never disagree.
    pub fn add_counters(&mut self, counter_type: CounterType, n: u32, timestamp: Timestamp) {
        let entry = self
            .counters
            .entry(counter_type)
            .or_insert(CounterStack { count: 0, timestamp });
        entry.count += n;
        entry.timestamp = timestamp;
    }

    /// Remove up to `n` counters of the given type. Returns the number actually removed.
    pub fn remove_counters(&mut self, counter_type: CounterType, n: u32) -> u32 {
        let Some(entry) = self.counters.get_mut(&counter_type) else {
            return 0;
        };
        let removed = entry.count.min(n);
        entry.count -= removed;
        if entry.count == 0 {
            self.counters.remove(&counter_type);
        }
        removed
    }

    /// Returns the number of counters of the given type (0 if none).
    pub fn counter_count(&self, counter_type: CounterType) -> u32 {
        self.counters.get(&counter_type).map(|s| s.count).unwrap_or(0)
    }

    /// The CR 613.7c timestamp shared by every counter of this kind, if any.
    pub fn counter_timestamp(&self, counter_type: CounterType) -> Option<Timestamp> {
        self.counters.get(&counter_type).map(|s| s.timestamp)
    }

    /// Attach this permanent to a host permanent.
    /// Sets `self.attached_to` to the host's ID.
    /// The caller is responsible for adding this permanent's ID to the host's `attached_by`.
    pub fn attach_to(&mut self, host: ObjectId) {
        self.attached_to = Some(host);
    }

    /// Detach this permanent from its host.
    /// Clears `self.attached_to`.
    /// The caller is responsible for removing this permanent's ID from the host's `attached_by`.
    pub fn detach(&mut self) {
        self.attached_to = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_entity() -> BattlefieldEntity {
        BattlefieldEntity::new(Uuid::new_v4(), 0, 1, 1)
    }

    #[test]
    fn test_add_counters() {
        let mut e = make_entity();
        e.add_counters(CounterType::PlusOnePlusOne, 3, 1);
        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 3);
        e.add_counters(CounterType::PlusOnePlusOne, 2, 2);
        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 5);
    }

    #[test]
    fn test_remove_counters() {
        let mut e = make_entity();
        e.add_counters(CounterType::PlusOnePlusOne, 3, 3);

        let removed = e.remove_counters(CounterType::PlusOnePlusOne, 2);
        assert_eq!(removed, 2);
        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 1);

        // Remove more than available — clamped at 0
        let removed = e.remove_counters(CounterType::PlusOnePlusOne, 5);
        assert_eq!(removed, 1);
        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 0);

        // Remove from empty — returns 0
        let removed = e.remove_counters(CounterType::PlusOnePlusOne, 1);
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_counter_count_default_zero() {
        let e = make_entity();
        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 0);
        assert_eq!(e.counter_count(CounterType::Flying), 0);
        assert_eq!(e.counter_count(CounterType::Loyalty), 0);
    }

    #[test]
    fn test_multiple_counter_types() {
        let mut e = make_entity();
        e.add_counters(CounterType::PlusOnePlusOne, 2, 4);
        e.add_counters(CounterType::MinusOneMinusOne, 1, 5);
        e.add_counters(CounterType::Flying, 1, 6);
        e.add_counters(CounterType::Charge, 5, 7);

        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 2);
        assert_eq!(e.counter_count(CounterType::MinusOneMinusOne), 1);
        assert_eq!(e.counter_count(CounterType::Flying), 1);
        assert_eq!(e.counter_count(CounterType::Charge), 5);
        assert_eq!(e.counter_count(CounterType::Loyalty), 0);

        // Remove one type, others unaffected
        e.remove_counters(CounterType::PlusOnePlusOne, 2);
        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 0);
        assert_eq!(e.counter_count(CounterType::MinusOneMinusOne), 1);
        assert_eq!(e.counter_count(CounterType::Flying), 1);
    }

    #[test]
    fn test_attachment_tracking_basic() {
        let host_id = Uuid::new_v4();
        let attachment_id = Uuid::new_v4();

        let mut host = BattlefieldEntity::new(host_id, 0, 1, 1);
        let mut attachment = BattlefieldEntity::new(attachment_id, 0, 2, 1);

        // Attach: set attached_to on attachment, add to host's attached_by
        attachment.attach_to(host_id);
        host.attached_by.push(attachment_id);

        assert_eq!(attachment.attached_to, Some(host_id));
        assert_eq!(host.attached_by, vec![attachment_id]);
    }

    #[test]
    fn test_detach_clears_both_sides() {
        let host_id = Uuid::new_v4();
        let attachment_id = Uuid::new_v4();

        let mut host = BattlefieldEntity::new(host_id, 0, 1, 1);
        let mut attachment = BattlefieldEntity::new(attachment_id, 0, 2, 1);

        // Attach
        attachment.attach_to(host_id);
        host.attached_by.push(attachment_id);

        // Detach
        attachment.detach();
        host.attached_by.retain(|&id| id != attachment_id);

        assert_eq!(attachment.attached_to, None);
        assert!(host.attached_by.is_empty());
    }

    #[test]
    fn test_attachment_defaults_none() {
        let e = make_entity();
        assert_eq!(e.attached_to, None);
        assert!(e.attached_by.is_empty());
    }
}
