use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::Zone;
use crate::types::mana::ManaType;
use crate::state::game_state::{AbilityIdentity, PhaseType, StepType};

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

    /// A permanent became tapped (CR 701.26a).
    ///
    /// **Only on the transition.** CR 603.2e: "becomes tapped" triggers only
    /// when a permanent already on the battlefield changes from untapped to
    /// tapped — it does not fire for a redundant tap, and does not fire for a
    /// permanent that *enters* tapped. CR 701.26a agrees from the other side:
    /// only untapped permanents can be tapped at all.
    Tapped { object_id: ObjectId },

    /// A permanent became untapped (CR 701.26b). Transition-only, as `Tapped`.
    Untapped { object_id: ObjectId },

    /// A player drew a card (CR 121.1).
    ///
    /// **Not redundant with the `ZoneChange` it accompanies.** CR 121.5: moving
    /// cards from library to hand *without the word "draw"* means the player has
    /// not drawn them, and that difference is trigger-visible — 106 cards say
    /// "whenever you draw a card" and 54 count "your second card". A tutor and a
    /// draw produce the same library→hand `ZoneChange`; only this event
    /// separates them.
    ///
    /// Not emitted when the library was empty: nothing was drawn, and CR 704.5b
    /// handles the attempt as a state-based action instead.
    CardDrawn { player_id: PlayerId, card_id: ObjectId },

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
    LifeChanged { player_id: PlayerId, old: i64, new: i64, source: Option<ObjectId> },

    // --- Combat ---
    AttackersDeclared { attackers: Vec<ObjectId> },
    BlockersDeclared { blockers: Vec<(ObjectId, ObjectId)> },

    // --- Spells ---
    SpellCast { spell_id: ObjectId, caster: PlayerId },
    SpellResolved { spell_id: ObjectId },
    /// An activated ability finished resolving (CR 608.2m), identified by what
    /// it *is* rather than by the stack object that represented it.
    ///
    /// `SpellResolved` carries the ephemeral ability object's id, which ceases
    /// to exist at resolution and therefore identifies nothing afterward.
    /// CR 603.7h counting — "whenever this ability resolves for the third time
    /// this turn" (Ashling the Pilgrim; Ashling, Flame Dancer) — needs the
    /// durable (source, ability) pair, which is why this event exists
    /// alongside it rather than replacing it.
    AbilityResolved { identity: AbilityIdentity, controller: PlayerId },
    /// An activated ability was put onto the stack (CR 602.2a).
    AbilityActivated { identity: AbilityIdentity, controller: PlayerId },
    SpellCountered { spell_id: ObjectId, countered_by: ObjectId },
    AbilityCountered { ability_id: ObjectId, countered_by: ObjectId },
    /// Spell or ability fizzled (countered by game rules due to all targets
    /// becoming illegal). No source object — this is a game-rules counter.
    SpellFizzled { spell_id: ObjectId },

    // --- Creatures ---
    CreatureDied { creature_id: ObjectId, owner: PlayerId },

    // --- Permanents put into graveyard by SBA ---
    /// A planeswalker was put into its owner's graveyard by SBA (704.5i, 0 loyalty).
    PlaneswalkerDied { object_id: ObjectId, owner: PlayerId },
    /// A permanent was put into its owner's graveyard by the legend rule (704.5j).
    LegendRuleSacrificed { object_id: ObjectId, owner: PlayerId },

    // --- Player loss ---
    PlayerLost { player_id: PlayerId, reason: LossReason },

    // --- Counters ---
    /// +1/+1 and -1/-1 counters annihilated each other on a permanent (rule 704.5q).
    CountersAnnihilated { object_id: ObjectId, pairs_removed: u32 },

    // --- Attachment SBAs ---
    /// An Aura was put into its owner's graveyard by SBA 704.5m/704.5n
    /// (unattached or attached to an illegal/missing object).
    AuraDied { object_id: ObjectId, owner: PlayerId },
    /// An Equipment or Fortification was detached by SBA 704.5p
    /// (attached to a non-creature). Equipment stays on battlefield.
    EquipmentDetached { equipment_id: ObjectId, former_host: ObjectId },

    // --- Tokens ---
    /// A token in a non-battlefield zone ceased to exist (rule 704.5d).
    /// Not a zone change — the token is simply removed from the game.
    TokenCeasedToExist { object_id: ObjectId },

    // --- State-based ---
    StateBasedActionPerformed,
}

/// Why a player lost the game (for event logging).
#[derive(Debug, Clone, PartialEq)]
pub enum LossReason {
    /// Life total reached 0 or below (rule 704.5a)
    LifeReachedZero,
    /// Attempted to draw from an empty library (rule 704.5b)
    DrawnFromEmptyLibrary,
    /// Accumulated 10 or more poison counters (rule 704.5c)
    PoisonCounters,
    /// Dealt 21 or more combat damage by a single commander (rule 704.5)
    CommanderDamage,
}

/// What damage is being dealt to
#[derive(Debug, Clone, PartialEq)]
pub enum DamageTarget {
    Player(PlayerId),
    Object(ObjectId),
}

/// Identifies a set of events performed as one.
///
/// Three rules need an event *set* rather than an event: CR 704.3 ("performs
/// all applicable state-based actions simultaneously as a single event"),
/// CR 510.2 (combat damage), and CR 502.1 (the untap step). Every event a
/// batch emits carries the same `BatchId`, which is what CR 603.2c/603.6a's
/// "one or more" phrasing needs — "whenever one or more creatures die" fires
/// once for the batch, not once per member.
///
/// **Nothing reads this in RA.** Phase 6's trigger matcher is the customer;
/// RA's job is that the grouping exists and is recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BatchId(pub u64);

/// One entry in the event log: what happened, plus the context it happened in.
///
/// The context is an envelope rather than fields on each `GameEvent` variant
/// because it is the same two facts for every kind of event, and a variant that
/// forgets to carry them fails silently.
#[derive(Debug, Clone)]
pub struct EventRecord {
    /// What happened.
    pub event: GameEvent,
    /// The batch this event was performed as part of, if any. `None` for an
    /// event emitted outside [`GameState::execute_actions`] — a phase
    /// beginning, a spell being cast, an activation.
    pub batch: Option<BatchId>,
}

/// An event log that records game events in order.
///
/// This serves multiple purposes:
/// 1. Triggered ability checking ("when X happens" — scan recent events)
/// 2. Game history / replay
/// 3. UI display
#[derive(Debug, Clone, Default)]
pub struct EventLog {
    records: Vec<EventRecord>,
    /// The batch currently being performed, stamped onto everything emitted
    /// while it is open. See [`Self::open_batch`].
    current_batch: Option<BatchId>,
    /// Allocator for [`BatchId`]. Monotonic, never reused within a game.
    next_batch: u64,
}

impl EventLog {
    pub fn new() -> Self {
        EventLog::default()
    }

    pub fn emit(&mut self, event: GameEvent) {
        self.records.push(EventRecord { event, batch: self.current_batch });
    }

    /// Open a batch, returning the value to hand back to [`Self::close_batch`].
    ///
    /// **A nested call joins the enclosing batch rather than opening a new
    /// one.** CR 702.15b is why it has to: lifelink's life gain is proposed
    /// from inside `perform_action(DealDamage)` and happens *simultaneously
    /// with that damage*, so it belongs to the damage's batch. The same shape
    /// generalizes to CR 120.3's results-of-damage decomposition in Phase RD.
    pub(crate) fn open_batch(&mut self) -> Option<BatchId> {
        let previous = self.current_batch;
        if previous.is_none() {
            self.current_batch = Some(BatchId(self.next_batch));
            self.next_batch += 1;
        }
        previous
    }

    /// Close a batch, restoring what [`Self::open_batch`] returned.
    pub(crate) fn close_batch(&mut self, previous: Option<BatchId>) {
        self.current_batch = previous;
    }

    /// The full records, with their batch context.
    pub fn records(&self) -> &[EventRecord] {
        &self.records
    }

    /// Just the events, for callers that do not care which batch they came from.
    pub fn events(&self) -> impl Iterator<Item = &GameEvent> + '_ {
        self.records.iter().map(|r| &r.event)
    }

    /// The events emitted since `index` — pass an earlier [`Self::len`].
    pub fn events_from(&self, index: usize) -> impl Iterator<Item = &GameEvent> + '_ {
        self.records_from(index).iter().map(|r| &r.event)
    }

    /// Get records since the given index (useful for checking "what happened since last check")
    pub fn records_from(&self, index: usize) -> &[EventRecord] {
        if index >= self.records.len() {
            &[]
        } else {
            &self.records[index..]
        }
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Clear the log (e.g., between games)
    pub fn clear(&mut self) {
        self.records.clear();
        self.current_batch = None;
        self.next_batch = 0;
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

        let since = log.records_from(1);
        assert_eq!(since.len(), 2);

        let since_end = log.records_from(3);
        assert_eq!(since_end.len(), 0);
    }

    #[test]
    fn test_events_outside_a_batch_carry_no_batch_id() {
        let mut log = EventLog::new();
        log.emit(GameEvent::TurnBegin { player: 0, turn_number: 1 });
        assert_eq!(log.records()[0].batch, None);
    }

    #[test]
    fn test_a_batch_stamps_every_event_it_emits() {
        let mut log = EventLog::new();
        let outer = log.open_batch();
        log.emit(GameEvent::Tapped { object_id: uuid::Uuid::nil() });
        log.emit(GameEvent::Untapped { object_id: uuid::Uuid::nil() });
        log.close_batch(outer);
        log.emit(GameEvent::StateBasedActionPerformed);

        let b = log.records()[0].batch.expect("inside a batch");
        assert_eq!(log.records()[1].batch, Some(b), "one batch, one id");
        assert_eq!(log.records()[2].batch, None, "closing restores the outer context");
    }

    #[test]
    fn test_a_nested_batch_joins_the_enclosing_one() {
        // CR 702.15b — lifelink's gain is proposed from inside the damage's
        // performance and is simultaneous with it, so it must not open a batch
        // of its own.
        let mut log = EventLog::new();
        let outer = log.open_batch();
        log.emit(GameEvent::StateBasedActionPerformed);
        let inner = log.open_batch();
        log.emit(GameEvent::StateBasedActionPerformed);
        log.close_batch(inner);
        log.emit(GameEvent::StateBasedActionPerformed);
        log.close_batch(outer);

        let b = log.records()[0].batch.expect("inside a batch");
        assert!(log.records().iter().all(|r| r.batch == Some(b)),
                "a nested batch joins the enclosing one rather than opening its own");
    }

    #[test]
    fn test_separate_batches_get_separate_ids() {
        let mut log = EventLog::new();
        let prev = log.open_batch();
        log.emit(GameEvent::StateBasedActionPerformed);
        log.close_batch(prev);
        let prev = log.open_batch();
        log.emit(GameEvent::StateBasedActionPerformed);
        log.close_batch(prev);

        assert_ne!(log.records()[0].batch, log.records()[1].batch);
    }
}
