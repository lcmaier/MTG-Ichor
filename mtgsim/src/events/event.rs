use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::Zone;
use crate::types::mana::ManaType;
use crate::state::game_state::{PhaseType, StepType};

use std::collections::HashMap;

/// Game events that can be observed by triggered abilities and logging systems.
///
/// Events are emitted *after* the action occurs (past tense). They represent
/// facts about what happened, not requests for what should happen.
///
/// **Replacement effects** (e.g. "if damage would be dealt, prevent it") are
/// NOT modeled as events. They will be handled by a replacement effect registry
/// that the engine consults *before* performing an action. See the design note
/// in the module docs for details.
///
/// The engine emits these; triggered abilities and logging subscribe to them.
#[derive(Debug, Clone)]
pub enum GameEvent {
    // --- Zone transitions ---
    ZoneChange {
        object_id: ObjectId,
        owner: PlayerId,
        from: Zone,
        to: Zone,
    },

    // --- Mana ---
    ManaAdded {
        player_id: PlayerId,
        source_id: ObjectId,
        mana: HashMap<ManaType, u64>,
    },

    // --- Damage ---
    DamageDealt {
        source_id: ObjectId,
        target: DamageTarget,
        amount: u64,
    },

    // --- Turn structure ---
    PhaseBegin { phase: PhaseType },
    PhaseEnd { phase: PhaseType },
    StepBegin { step: StepType },
    StepEnd { step: StepType },
    TurnBegin { player: PlayerId, turn_number: u32 },
    TurnEnd { player: PlayerId, turn_number: u32 },

    // --- Permanents ---
    PermanentEnteredBattlefield { object_id: ObjectId, controller: PlayerId },
    PermanentLeftBattlefield { object_id: ObjectId },

    // --- Life ---
    LifeChanged { player_id: PlayerId, old: i64, new: i64 },

    // --- Combat ---
    AttackersDeclared { attackers: Vec<ObjectId> },
    BlockersDeclared { blockers: Vec<(ObjectId, ObjectId)> },

    // --- Spells ---
    SpellCast { spell_id: ObjectId, caster: PlayerId },
    SpellResolved { spell_id: ObjectId },
    SpellCountered { spell_id: ObjectId },

    // --- Creatures ---
    CreatureDied { creature_id: ObjectId, owner: PlayerId },

    // --- State-based ---
    StateBasedActionPerformed,
}

/// What damage is being dealt to
#[derive(Debug, Clone, PartialEq)]
pub enum DamageTarget {
    Player(PlayerId),
    Object(ObjectId),
}

/// An event log that records game events in order.
///
/// This serves multiple purposes:
/// 1. Triggered ability checking ("when X happens" — scan recent events)
/// 2. Game history / replay
/// 3. UI display
#[derive(Debug, Clone, Default)]
pub struct EventLog {
    events: Vec<GameEvent>,
}

impl EventLog {
    pub fn new() -> Self {
        EventLog { events: Vec::new() }
    }

    pub fn emit(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    pub fn events(&self) -> &[GameEvent] {
        &self.events
    }

    /// Get events since the given index (useful for checking "what happened since last check")
    pub fn events_since(&self, index: usize) -> &[GameEvent] {
        if index >= self.events.len() {
            &[]
        } else {
            &self.events[index..]
        }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear the log (e.g., between games)
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_log_basic() {
        let mut log = EventLog::new();
        assert!(log.is_empty());

        log.emit(GameEvent::TurnBegin { player: 0, turn_number: 1 });
        log.emit(GameEvent::PhaseBegin { phase: PhaseType::Beginning });

        assert_eq!(log.len(), 2);
        assert!(!log.is_empty());
    }

    #[test]
    fn test_event_log_since() {
        let mut log = EventLog::new();
        log.emit(GameEvent::TurnBegin { player: 0, turn_number: 1 });
        log.emit(GameEvent::PhaseBegin { phase: PhaseType::Beginning });
        log.emit(GameEvent::StepBegin { step: StepType::Untap });

        let since = log.events_since(1);
        assert_eq!(since.len(), 2);

        let since_end = log.events_since(3);
        assert_eq!(since_end.len(), 0);
    }
}
