use crate::types::ids::{ObjectId, PlayerId};
use crate::types::effects::CounterType;
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

    // There is no `SpellResolved`/`StackObjectResolved` either, and it went for
    // the same reason plus one of its own: `resolve_top_of_stack` emitted it
    // unconditionally, so it fired for activated abilities as well as spells and
    // a matcher keying "whenever a spell resolves" on it would have been wrong
    // about every ability in the game. A spell finishing resolution is a
    // `ZoneChange` out of the stack with `ZoneChangeCause::Resolved`, which also
    // says *where* it went; an ability finishing is `AbilityResolved`, which
    // carries the durable identity CR 603.7h counting needs.
    /// An activated ability finished resolving (CR 608.2n), identified by what
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

    // --- Deaths are not events of their own ---
    //
    // There is no `CreatureDied`, `PlaneswalkerDied`, `LegendRuleSacrificed` or
    // `AuraDied`. Each was a `ZoneChange { from: Battlefield, to: Graveyard }`
    // with a `cause` and an `lki` frame, said less precisely, and each was
    // deleted rather than documented as display-only, for two structural
    // reasons and one measured one:
    //
    // - **They partition one event by type, and permanent types are not a
    //   partition.** A Gideon is a creature *and* a planeswalker; Circuit Mender
    //   is an artifact creature. Whichever event the engine emitted, a reader
    //   would miss every trigger keyed on the other type. The `lki` frame
    //   carries the whole type set, so one event answers for all of them.
    // - **They named a subset without naming its boundary.** "Dies" is
    //   battlefield → graveyard; "leaves the battlefield" is battlefield →
    //   anywhere; and ATOM-603.6c-001 turns on *which* zone the card went to.
    //   An event carrying one id cannot answer that. (`PermanentLeftBattlefield`
    //   was deleted for the same reason, having never had an emitter at all.)
    // - **The redundancy was hiding a bug.** `CreatureDied` was emitted only
    //   from the state-based-action sites, so a creature killed by a spell
    //   produced none, and `fuzz_games` undercounted deaths at 5.3 per game
    //   where the zone changes say 6.2.
    //
    // A reader that wants deaths matches `ZoneChange { from: Battlefield, to:
    // Graveyard, lki, .. }` and asks the frame what died. `ui/display.rs` and
    // `fuzz_games` both do exactly that.

    // --- Player loss ---
    PlayerLost { player_id: PlayerId, reason: LossReason },

    // --- Counters ---
    /// Counters were put on or taken off a permanent (CR 122.1).
    ///
    /// One event for both directions rather than two, because `added` is a
    /// signed count and a reader that cares about the direction reads its sign.
    /// It is emitted only on an actual change: `RemoveCounters` reports how
    /// many were really there (CR 701.2's "as much as it can"), and removing
    /// none announces nothing — the same transition rule CR 603.2e gives
    /// tapping.
    CountersChanged { object_id: ObjectId, counter: CounterType, added: i32 },

    /// +1/+1 and -1/-1 counters annihilated each other on a permanent (rule 704.5q).
    ///
    /// Distinct from [`Self::CountersChanged`] because CR 704.5q is a
    /// state-based action that removes both kinds at once and still writes
    /// `BattlefieldEntity` directly — it has no `GameAction` to propose through
    /// (`codebase-state.md` Deferred Migrations item 6).
    CountersAnnihilated { object_id: ObjectId, pairs_removed: u32 },

    // --- Attachment SBAs ---
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
/// batch emits carries the same `BatchId`, which is what **CR 603.2c** needs:
/// "an ability triggers only once each time its trigger event occurs. However,
/// it can trigger repeatedly if one event contains multiple occurrences." The
/// batch is that boundary. "Whenever one or more creatures die" takes the whole
/// batch as its trigger event and fires once; "whenever a creature dies" fires
/// once per death inside it.
///
/// **Nothing reads this in RA.** Phase 6's trigger matcher is the customer;
/// RA's job is that the grouping exists and is recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BatchId(pub u64);

/// Which resolution an event belongs to (CR 608.2).
///
/// `source` is the stack object that was resolving, and it identifies the
/// *resolution* rather than the card, because this engine gives every stack
/// object a fresh v4 `ObjectId`: one per cast, and one per activation for an
/// ability's ephemeral object, which CR 608.2n destroys rather than recycles.
/// That is an engine property, not a rule.
///
/// **Who reads it.** RB's pipeline is the first: CR 614.15 self-replacement
/// effects belong to the resolving spell or ability rather than to any registry,
/// so `apply_replacements` has to know which resolution proposed an action in
/// order to find them. That lookup uses `ActionContext::resolution`, the
/// *proposal* side. Stamping it on the performed record as well is provenance —
/// specified by the event-stream design in `codebase-state.md`, wanted by the
/// CR 731 loop-detection transcripts, and useful in a log. **No trigger matcher
/// needs it today**, and this comment should not be read as claiming one does.
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
    /// Stamped onto everything emitted while it is installed.
    ///
    /// **Ambient, and deliberately so.** The alternative is an
    /// `emit_with(event, stamp)` at all 45 emission sites, which is 45 chances
    /// to forget and no way to notice — the stamp has no test of its own at most
    /// of them. Scoping it to `execute_actions` instead means the context is
    /// established once per performed action, by the one function that knows it.
    ///
    /// This is not the ambient state `CLAUDE.md` bans. That rule is about
    /// `rand::rng()`: an ambient *source* silently changes the game's outcome
    /// and destroys replayability. This is an ambient *label* on a record —
    /// it changes what the log says about a mutation, never whether the mutation
    /// happens — and its lifetime is a single `open_batch`/`close_batch` pair
    /// that `execute_actions` opens and closes on the same code path.
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
    /// one.** CR 120.3f and 120.4 are why it has to: lifelink's life gain is one
    /// of the damage's *results*, CR 120.4c processes the results, and CR 120.4d
    /// then lets the one damage event occur. Lifelink proposes from inside
    /// `perform_action(DealDamage)`, so a second batch id would split one event
    /// into two. The same shape generalizes to CR 120.3's results-of-damage
    /// decomposition in Phase RD.
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

    /// Open a batch that does **not** join an enclosing one.
    ///
    /// `GameState::execute_actions_new_batch` is the only caller and owns the
    /// rules argument for why CR 614.13's auxiliary moves are not a result of
    /// the event they are nested inside.
    pub(crate) fn open_new_batch(&mut self, resolution: Option<ResolutionStamp>) -> EventStamp {
        let previous = self.stamp;
        let id = BatchId(self.next_batch);
        self.next_batch += 1;
        self.stamp = EventStamp { batch: Some(id), resolution };
        previous
    }

    /// Close a batch, restoring what [`Self::open_batch`] returned.
    pub(crate) fn close_batch(&mut self, previous: EventStamp) {
        self.stamp = previous;
    }

    /// The log. `records()` is the whole of it; [`Self::events`] is a
    /// convenience over the same data for readers that do not want the stamp.
    ///
    /// **Nothing in the engine reads either yet.** Every caller today is a test,
    /// `ui/display.rs`, or `fuzz_games`, and that is expected rather than a
    /// smell: RA built the record, and Phase 6's trigger matcher is its first
    /// production consumer. Keep the surface small until then — an earlier draft
    /// of this API also had an `events_from`, which was two call sites of sugar
    /// over `records_from` and is gone.
    pub fn records(&self) -> &[EventRecord] {
        &self.records
    }

    /// Just the events, for readers that do not care which batch or resolution
    /// they came from — the display path, and most assertions.
    pub fn events(&self) -> impl Iterator<Item = &GameEvent> + '_ {
        self.records.iter().map(|r| &r.event)
    }

    /// The records since `index` — pass an earlier [`Self::len`] to ask "what
    /// happened since I last looked".
    ///
    /// Clamped rather than sliced, so a stale mark returns nothing instead of
    /// panicking. That matters for the trigger matcher, whose mark is taken
    /// before a resolution that may clear the log.
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
        // CR 120.3f makes lifelink's gain a result of the damage, and CR 120.4d
        // lets the one damage event occur after its results are processed. The
        // gain is proposed from inside the damage's performance, so it must not
        // open a batch of its own.
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
