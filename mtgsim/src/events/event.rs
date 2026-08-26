use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::Zone;
use crate::types::mana::ManaType;
use crate::engine::actions::ZoneChangeCause;
use crate::engine::layers::types::EffectiveCharacteristics;
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
        /// Why the engine moved it. `(from, to)` cannot tell a sacrifice from a
        /// destruction, and 278 printed cards want that difference.
        ///
        /// **Only the replacement pipeline and the trigger matcher may branch
        /// on this.** See [`ZoneChangeCause`](crate::engine::actions::ZoneChangeCause).
        cause: ZoneChangeCause,
        /// CR 603.10a — the permanent's characteristics an instant *before* it
        /// left the battlefield. `None` when it did not leave the battlefield,
        /// because then there is nothing to look back at.
        ///
        /// Leaves-the-battlefield abilities "look back in time": the game reads
        /// the object as it existed before the event, which is how a creature
        /// with "when this dies, draw a card" still has that ability at the
        /// moment it is asked, and how a dying Aura still knows what it was
        /// enchanting. Capturing it late is not merely inaccurate, it is
        /// impossible — CR 611.2a drops every registry row the object's static
        /// abilities generated the instant it leaves, so the layer walk a
        /// moment later answers about a graveyard card.
        ///
        /// Boxed because `EffectiveCharacteristics` is the widest type in the
        /// engine and every `GameEvent` in the log would otherwise pay for it.
        lki: Option<Box<EffectiveCharacteristics>>,
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

    // --- Life ---
    LifeChanged { player_id: PlayerId, old: i64, new: i64, source: Option<ObjectId> },

    // --- Combat ---
    AttackersDeclared { attackers: Vec<ObjectId> },
    BlockersDeclared { blockers: Vec<(ObjectId, ObjectId)> },

    // --- Spells ---
    SpellCast { spell_id: ObjectId, caster: PlayerId },

    /// A stack object finished resolving (CR 608.2m).
    ///
    /// **Display only, and renamed because the old name lied.** It was
    /// `SpellResolved`, and `resolve_top_of_stack` emits it unconditionally —
    /// so it has always fired for activated abilities as well as spells, and a
    /// matcher keying "whenever a spell resolves" on it would have been wrong
    /// about every ability in the game.
    ///
    /// Nothing needs it. A spell finishing resolution is a
    /// [`Self::ZoneChange`] out of the stack with
    /// [`ZoneChangeCause::Resolved`] — which also says *where* it went, the
    /// thing this event never carried. An ability finishing is
    /// [`Self::AbilityResolved`], which carries the durable identity CR 603.7h
    /// counting needs. This one is a log line.
    StackObjectResolved { object_id: ObjectId },
    /// An activated ability finished resolving (CR 608.2m), identified by what
    /// it *is* rather than by the stack object that represented it.
    ///
    /// [`Self::StackObjectResolved`] carries the ephemeral ability object's id,
    /// which ceases to exist at resolution and therefore identifies nothing
    /// afterward.
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

    // --- Deaths: display sugar, not matchable facts ---
    //
    // All three of these are `ZoneChange { from: Battlefield, to: Graveyard }`
    // with a `cause` and an `lki` frame, said less precisely. **Nothing may
    // match a trigger on them**, for two reasons that are both structural:
    //
    // - **They partition the same event by type, and permanent types are not a
    //   partition.** A Gideon is a creature *and* a planeswalker; Circuit Mender
    //   is an artifact creature. Whichever of these events the engine chose to
    //   emit, a matcher reading it would miss every trigger keyed on the other
    //   type. The `lki` frame carries the whole type set, so one event matches
    //   everything applicable.
    // - **They name a subset without naming its boundary.** "Dies" is
    //   battlefield → graveyard; "leaves the battlefield" is battlefield →
    //   anywhere; CR 603.6c's own atom (ATOM-603.6c-001) turns on *which* zone
    //   the card went to. An event carrying only an id cannot answer that, which
    //   is why `PermanentLeftBattlefield` was deleted rather than wired up.
    //
    // `fuzz_games` counts `CreatureDied` and `ui/display.rs` prints all three.
    // That is the whole permitted consumer set.

    /// Display only — see the note above. Use the `ZoneChange`.
    CreatureDied { creature_id: ObjectId, owner: PlayerId },
    /// Display only — a planeswalker put into its owner's graveyard by
    /// SBA 704.5i (0 loyalty). Use the `ZoneChange` and its `cause`.
    PlaneswalkerDied { object_id: ObjectId, owner: PlayerId },
    /// Display only — a permanent put into its owner's graveyard by the legend
    /// rule (704.5j). Use the `ZoneChange` and its `cause`.
    LegendRuleSacrificed { object_id: ObjectId, owner: PlayerId },

    // --- Player loss ---
    PlayerLost { player_id: PlayerId, reason: LossReason },

    // --- Counters ---
    /// +1/+1 and -1/-1 counters annihilated each other on a permanent (rule 704.5q).
    CountersAnnihilated { object_id: ObjectId, pairs_removed: u32 },

    // --- Attachment SBAs ---
    /// Display only, as the deaths above — an Aura put into its owner's
    /// graveyard by SBA 704.5m/704.5n (unattached, or attached to an illegal or
    /// missing object).
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

/// Which resolution an event belongs to (CR 608.2).
///
/// `source` is the stack object that was resolving, and that object identifies
/// the *resolution* rather than the card: a spell has one stack object, and an
/// activated ability gets a fresh ephemeral one per activation (CR 608.2m). So
/// "did this happen during the resolution that also did X" is answerable
/// without a separate id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolutionStamp {
    pub source: ObjectId,
    pub controller: PlayerId,
}

/// The context every emitted event is stamped with.
///
/// An envelope rather than fields on each `GameEvent` variant: it is the same
/// two facts for every kind of event, and a variant that forgets to carry them
/// fails silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EventStamp {
    /// The batch this event was performed as part of. `None` for an event
    /// emitted outside `GameState::execute_actions` — a phase beginning, a
    /// spell being cast, an activation.
    pub batch: Option<BatchId>,
    /// The resolution that proposed it. `None` for a turn-based action, a
    /// state-based action, cost payment, or combat damage — none of which
    /// belongs to a resolution.
    pub resolution: Option<ResolutionStamp>,
}

/// One entry in the event log: what happened, and the context it happened in.
#[derive(Debug, Clone)]
pub struct EventRecord {
    /// What happened.
    pub event: GameEvent,
    /// See [`EventStamp`].
    pub stamp: EventStamp,
}

impl EventRecord {
    /// The batch this event was performed as part of, if any.
    pub fn batch(&self) -> Option<BatchId> {
        self.stamp.batch
    }

    /// The resolution that proposed this event, if any.
    pub fn resolution(&self) -> Option<ResolutionStamp> {
        self.stamp.resolution
    }
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
    /// Stamped onto everything emitted while it is installed. See
    /// [`Self::open_batch`].
    stamp: EventStamp,
    /// Allocator for [`BatchId`]. Monotonic, never reused within a game.
    next_batch: u64,
}

impl EventLog {
    pub fn new() -> Self {
        EventLog::default()
    }

    pub fn emit(&mut self, event: GameEvent) {
        self.records.push(EventRecord { event, stamp: self.stamp });
    }

    /// Open a batch, returning the stamp to hand back to [`Self::close_batch`].
    ///
    /// **A nested call joins the enclosing batch rather than opening a new
    /// one.** CR 702.15b is why it has to: lifelink's life gain is proposed
    /// from inside `perform_action(DealDamage)` and happens *simultaneously
    /// with that damage*, so it belongs to the damage's batch. The same shape
    /// generalizes to CR 120.3's results-of-damage decomposition in Phase RD.
    ///
    /// `resolution` is not inherited the same way — it comes from the proposing
    /// `ActionContext` every time, because the honest answer to "which
    /// resolution emitted this" is the one that proposed it. In practice the
    /// nested call carries the enclosing context, so the two rules agree.
    pub(crate) fn open_batch(&mut self, resolution: Option<ResolutionStamp>) -> EventStamp {
        let previous = self.stamp;
        let batch = previous.batch.or_else(|| {
            let id = BatchId(self.next_batch);
            self.next_batch += 1;
            Some(id)
        });
        self.stamp = EventStamp { batch, resolution };
        previous
    }

    /// Close a batch, restoring what [`Self::open_batch`] returned.
    pub(crate) fn close_batch(&mut self, previous: EventStamp) {
        self.stamp = previous;
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
        self.stamp = EventStamp::default();
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
        assert_eq!(log.records()[0].batch(), None);
    }

    #[test]
    fn test_a_batch_stamps_every_event_it_emits() {
        let mut log = EventLog::new();
        let outer = log.open_batch(None);
        log.emit(GameEvent::Tapped { object_id: uuid::Uuid::nil() });
        log.emit(GameEvent::Untapped { object_id: uuid::Uuid::nil() });
        log.close_batch(outer);
        log.emit(GameEvent::StateBasedActionPerformed);

        let b = log.records()[0].batch().expect("inside a batch");
        assert_eq!(log.records()[1].batch(), Some(b), "one batch, one id");
        assert_eq!(log.records()[2].batch(), None, "closing restores the outer context");
    }

    #[test]
    fn test_a_nested_batch_joins_the_enclosing_one() {
        // CR 702.15b — lifelink's gain is proposed from inside the damage's
        // performance and is simultaneous with it, so it must not open a batch
        // of its own.
        let mut log = EventLog::new();
        let outer = log.open_batch(None);
        log.emit(GameEvent::StateBasedActionPerformed);
        let inner = log.open_batch(None);
        log.emit(GameEvent::StateBasedActionPerformed);
        log.close_batch(inner);
        log.emit(GameEvent::StateBasedActionPerformed);
        log.close_batch(outer);

        let b = log.records()[0].batch().expect("inside a batch");
        assert!(log.records().iter().all(|r| r.batch() == Some(b)),
                "a nested batch joins the enclosing one rather than opening its own");
    }

    #[test]
    fn test_separate_batches_get_separate_ids() {
        let mut log = EventLog::new();
        let prev = log.open_batch(None);
        log.emit(GameEvent::StateBasedActionPerformed);
        log.close_batch(prev);
        let prev = log.open_batch(None);
        log.emit(GameEvent::StateBasedActionPerformed);
        log.close_batch(prev);

        assert_ne!(log.records()[0].batch(), log.records()[1].batch());
    }
}
