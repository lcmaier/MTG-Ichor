use crate::types::ids::{ObjectId, PlayerId};
use crate::types::effects::CounterType;
use crate::types::zones::Zone;
use crate::types::mana::ManaType;
use crate::engine::actions::{LifeLossCause, ZoneChangeCause};
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::state::game_state::{AbilityIdentity, PhaseType, StepType};
use crate::types::triggers::{TriggerOrigin, TriggerSeq};


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
    /// CR 106.6a / 106.12 — a spell or ability produced mana, performed as
    /// `GameAction::ProduceMana`.
    ///
    /// `mana` is by type in proposal order, restricted units folded into
    /// their type's count — CR 106.6 says a restriction "doesn't affect the
    /// mana's type" — and a `Vec` rather than a `HashMap`, because
    /// `--dump-events` renders this line and a map's order is the process's.
    /// `tapped_for_mana` is CR 106.12's fact, the one CR 106.12a's "whenever a
    /// permanent is tapped for mana" triggers read off the performed event.
    ManaAdded {
        player_id: PlayerId,
        source_id: ObjectId,
        mana: Vec<(ManaType, u64)>,
        tapped_for_mana: bool,
    },

    // --- Damage ---
    /// `is_combat` is the proposal's, carried because it is not
    /// live-derivable: a triggered ability resolving during the combat damage
    /// step deals noncombat damage in that step (`codebase-state.md` "Before
    /// Triggered abilities" item 10).
    DamageDealt {
        source_id: ObjectId,
        target: DamageTarget,
        amount: u64,
        is_combat: bool,
    },

    // --- Turn structure ---
    //
    // `player` is whose phase or step it is — the proposal's field, which
    // "at the beginning of your upkeep" reads (item 10). The three `*End`
    // variants that stood here were never emitted and no printed trigger
    // reads an end: "at end of combat" is the end-of-combat step beginning
    // (CR 511.2) and "at end of turn" the end step's (CR 513.1a) — item 18.
    PhaseBegin { phase: PhaseType, player: PlayerId },
    StepBegin { step: StepType, player: PlayerId },
    TurnBegin { player: PlayerId, turn_number: u32 },

    // --- Permanents ---
    PermanentEnteredBattlefield { object_id: ObjectId, controller: PlayerId },

    // --- Life ---
    /// `cause` is `Some` for a loss — the proposal's `LifeLossCause`, which
    /// CR 727.1a's "from radiation" reads (item 10) — and `None` for a gain,
    /// which has no cause to name.
    LifeChanged {
        player_id: PlayerId,
        old: i64,
        new: i64,
        source: Option<ObjectId>,
        cause: Option<LifeLossCause>,
    },

    // --- Combat ---
    AttackersDeclared { attackers: Vec<ObjectId> },
    BlockersDeclared { blockers: Vec<(ObjectId, ObjectId)> },

    // --- Spells ---
    SpellCast { spell_id: ObjectId, caster: PlayerId },

    // There is no `SpellResolved`/`StackObjectResolved`: a spell finishing
    // resolution is a `ZoneChange` out of the stack with
    // `ZoneChangeCause::Resolved`, which also says *where* it went, and an
    // ability finishing is `AbilityResolved`, which carries the durable identity
    // CR 603.7h counting needs. One event for both would fire for abilities
    // too, and "whenever a spell resolves" keyed on it would be wrong.
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
    /// An ability triggered (CR 603.2) — the record CR 603.3b's second tier
    /// watches, emitted by the dispatcher once per queued trigger after the
    /// window it belongs to has closed (`triggers-architecture.md` §4.8).
    /// **Unstamped**: it carries no batch and no resolution, because it is a
    /// consequence of the event and not part of it, and the dispatcher reads
    /// it like any unbatched record. `caused_by` is the record that matched
    /// (the first of them, for a "one or more" trigger).
    AbilityTriggered {
        seq: TriggerSeq,
        origin: TriggerOrigin,
        controller: PlayerId,
        caused_by: EventSeq,
    },
    SpellCountered { spell_id: ObjectId, countered_by: ObjectId },
    AbilityCountered { ability_id: ObjectId, countered_by: ObjectId },
    /// Spell or ability fizzled (countered by game rules due to all targets
    /// becoming illegal). No source object — this is a game-rules counter.
    SpellFizzled { spell_id: ObjectId },

    // --- Deaths are not events of their own ---
    //
    // There is no `CreatureDied`, `PlaneswalkerDied`, `LegendRuleSacrificed` or
    // `AuraDied`. Each is a `ZoneChange { from: Battlefield, to: Graveyard }`
    // with a `cause` and an `lki` frame, said less precisely:
    //
    // - **They would partition one event by type, and permanent types are not
    //   a partition.** A Gideon is a creature *and* a planeswalker; the `lki`
    //   frame carries the whole type set, so one event answers for all of them.
    // - **They would name a subset without naming its boundary.** "Dies" is
    //   battlefield → graveyard; "leaves the battlefield" is battlefield →
    //   anywhere; and ATOM-603.6c-001 turns on *which* zone the card went to.
    //
    // A reader that wants deaths matches `ZoneChange { from: Battlefield, to:
    // Graveyard, lki, .. }` and asks the frame what died. `ui/display.rs` and
    // `fuzz_games` both do exactly that.

    // --- The game's end ---
    /// Emitted by the `GameAction::PlayerLoses` performer, so a loss a
    /// replacement effect prevented (Exquisite Archangel) or a "can't" refused
    /// (Platinum Angel) announces nothing.
    PlayerLost { player_id: PlayerId, reason: LossReason },
    /// CR 104.2b — an effect stated that this player wins. Emitted by the
    /// `GameAction::PlayerWins` performer. CR 104.2a's win — the last player
    /// standing — is not an event: it is the outcome the losses imply, recorded
    /// on `GameState::result` when the batch that performed them settles.
    PlayerWon { player_id: PlayerId },

    // --- Scry ---
    /// A player scried (CR 701.22a). `n` is the instruction's number and
    /// `looked_at` is how many cards were really there to look at.
    ///
    /// **Emitted after the process, not before it, and emitted even when the
    /// library was short.** CR 701.22d: "an ability that triggers whenever a
    /// player scries triggers after the process described in rule 701.22a is
    /// complete, even if some or all of those actions were impossible." So a
    /// scry 2 against a one-card library is still a scry and still announces
    /// one.
    ///
    /// **Both numbers, because a printed card reads each and they differ.**
    /// `n` is what the instruction said, which is CR 615.5's "that many" and
    /// what Eligeth, Crossroads Augur draws. `looked_at` is Elrond, Master of
    /// Healing's, whose ruling is explicit: its trigger "cares about the
    /// number of cards you **actually** looked at. For example, if you were
    /// supposed to scry 3 but only had two cards in your library, X would be
    /// 2." Elrond's trigger itself is critical-path item 6's.
    ///
    /// **They agree on almost every board, and `min(n, library.len())` still
    /// cannot be derived later — Opt is the counter-example, and it is in the
    /// pool.** A scry moves no card between zones, so it is tempting to
    /// recompute the count from the library's length whenever a reader wants
    /// it. But "Scry 1. Draw a card." against a **one-card library** looks at
    /// that card and then draws it: by the time a trigger is put on the stack
    /// the library holds zero, and the derivation gives 0 where the answer is 1.
    /// Anything that empties or refills a library between the scry and the read
    /// does the same. The count is a fact about an instant that has passed,
    /// which is what an event log is for — the same argument
    /// [`Self::ZoneChange`]'s `lki` frame makes one field over.
    ///
    /// **Not a zone change, even when cards moved.** A card going to the
    /// bottom of its own library does not change zones (CR 400.1's zones are
    /// the seven, and "top" and "bottom" are positions inside one), so there
    /// is nothing for `announce_zone_change` to say and this is the only line
    /// a scry writes. A scry 0 writes none at all — CR 701.22b, enforced at
    /// the proposal by `replacement::never_happens`.
    Scried { player_id: PlayerId, n: u64, looked_at: u64 },

    /// A player shuffled their library (CR 701.24a) — what "whenever a player
    /// shuffles their library" will read.
    ///
    /// **Not a zone change**, for [`Self::Scried`]'s reason: the cards were
    /// reordered inside one zone. The move a "shuffle it into its owner's
    /// library" makes is its own `ZoneChange` line, announced first
    /// (CR 701.24c), and this line follows whether or not that move happened.
    LibraryShuffled { player_id: PlayerId },

    // --- Counters ---
    /// Counters were put on or taken off a permanent or a player (CR 122.1).
    ///
    /// One event for both directions rather than two, because `added` is a
    /// signed count and a reader that cares about the direction reads its sign.
    /// It is emitted only on an actual change: `RemoveCounters` reports how
    /// many were really there (CR 701.2's "as much as it can"), and removing
    /// none announces nothing — the same transition rule CR 603.2e gives
    /// tapping. One event for both subjects too: "whenever one or more
    /// counters are put on a permanent" and "whenever you get one or more
    /// counters" read the same line and branch on the subject. Counters a
    /// permanent is *given as it enters* are part of the entry (CR 122.6) and
    /// announce nothing here.
    CountersChanged { subject: CounterSubject, counter: CounterType, added: i32 },

    /// +1/+1 and -1/-1 counters annihilated each other on a permanent (rule 704.5q).
    ///
    /// Distinct from [`Self::CountersChanged`] because CR 704.5q is a
    /// state-based action that removes both kinds at once and still writes
    /// `PermanentState` directly — it has no `GameAction` to propose through
    /// (`codebase-state.md` Deferred Migrations item 6).
    CountersAnnihilated { object_id: ObjectId, pairs_removed: u32 },

    // --- Attachment ---
    /// An Aura, Equipment or Fortification became attached to `host`
    /// (CR 701.3a), leaving `former_host` if it was attached before. Emitted
    /// by the `GameAction::Attach` performer on the transition only — an
    /// attach to the host it is already on is CR 701.3b's "does nothing" and
    /// announces nothing.
    Attached { attachment: ObjectId, host: ObjectId, former_host: Option<ObjectId> },

    // --- Attachment SBAs ---
    /// An Equipment or Fortification was detached by SBA 704.5n
    /// (attached to an illegal permanent). It stays on the battlefield.
    EquipmentDetached { equipment_id: ObjectId, former_host: ObjectId },

    // --- Multiplayer (CR 800.4a) ---
    /// An object owned by a player who has just left the game left it too.
    ///
    /// **Not a zone change, because there is no zone to name**: CR 400.11
    /// lists the seven, and outside the game is not one of them. `from` is
    /// where the object was, which the log wants and a `ZoneChange` would have
    /// carried; there is no `to`.
    ///
    /// Emitted once per object by the `GameAction::PlayerLoses` performer —
    /// CR 800.4a is "not a state-based action. It happens as soon as the
    /// player leaves the game" — and by nothing else.
    ///
    /// **`lki` is CR 603.6c, which names this event in as many words**:
    /// "leaves-the-battlefield abilities trigger when a permanent moves from
    /// the battlefield to another zone, **or when a phased-in permanent leaves
    /// the game because its owner leaves the game**". So a permanent leaving
    /// here fires them, and CR 603.10a's frame is what a matcher will read —
    /// captured before `cleanup_zone_state` retires the static abilities that
    /// produced it, the same window `perform_zone_change` has. `None` for an
    /// object that was not a permanent, which is every other zone.
    ///
    /// CR 603.6c's qualifier is the one part with no implementation: a *phased
    /// out* permanent does not trigger, and phasing (CR 702.26) is not built
    /// (`codebase-state.md`, "Phasing"). Every permanent is phased in today.
    LeftTheGame {
        object_id: ObjectId,
        owner: PlayerId,
        from: Zone,
        lki: Option<Box<EffectiveCharacteristics>>,
    },

    // --- Tokens ---
    /// A token was created (CR 111.2's first sentence, CR 701.7a) — in
    /// `zone`, which is the battlefield for a creation performed as the
    /// effect wrote it and somewhere else for one whose entry a replacement
    /// substituted: Hallowed Moonlight's "exile it instead", whose ruling is
    /// that the token "is put into exile instead and then ceases to exist".
    /// CR 704.5d reads the second kind from there.
    ///
    /// **Not a zone change.** The token came from nowhere, so there is no
    /// `from` to name, and a `ZoneChange { from: Battlefield }` for the exiled
    /// kind would be the line a leaves-the-battlefield trigger reads
    /// (`codebase-state.md` item 52).
    ///
    /// One emitter, `GameState::announce_token_created`, with two callers,
    /// each of which performed the placement it announces: the
    /// `EnterBattlefield` performer's token arm, ahead of
    /// `PermanentEnteredBattlefield` — CR 111.2 is two sentences, the token
    /// is created and then it enters — and the `CreateTokenIn` performer.
    /// **CR 111.13 is the line between a token this announces and one it
    /// does not**: a copy of a permanent spell becomes a token as it
    /// resolves and "is not 'created' for the purposes of any replacement
    /// effects or triggered abilities that refer to creating a token" — it
    /// enters from the stack with a `from`, takes the card arm, and is
    /// announced by its zone change alone.
    ///
    /// `owner` is CR 111.2's "the player who creates a token is its owner";
    /// a token created in exile has no controller to name.
    TokenCreated { object_id: ObjectId, owner: PlayerId, zone: Zone },

    /// A token in a non-battlefield zone ceased to exist (rule 704.5d).
    /// Not a zone change — the token is simply removed from the game.
    TokenCeasedToExist { object_id: ObjectId },

    // --- State-based ---
    StateBasedActionPerformed,
}

/// Why a player lost the game.
///
/// Carried by `GameAction::PlayerLoses` and read by nothing that decides: every
/// printed "would lose the game" replacement and every printed "can't lose"
/// applies to every reason (`replacement-architecture.md` §9, RE decision 5),
/// so `EventPattern::PlayerLoses` has no field for it and this is the log's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LossReason {
    /// Life total reached 0 or below (rule 704.5a)
    LifeReachedZero,
    /// Attempted to draw from an empty library (rule 704.5b)
    DrawnFromEmptyLibrary,
    /// Accumulated 10 or more poison counters (rule 704.5c)
    PoisonCounters,
    /// Dealt 21 or more combat damage by a single commander (rule 704.6c)
    CommanderDamage,
    /// An effect said so (rule 104.3e) — `Primitive::LoseGame`.
    Effect,
}

/// What damage is being dealt to.
///
/// `Copy` because CR 614.9's redirection moves one of these into a
/// rewritten proposal while the original is still being read for the rule's
/// "or from" leg, and both halves are ids.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageTarget {
    Player(PlayerId),
    Object(ObjectId),
}

/// What counters are being put on or taken off — CR 122.1's "a marker placed
/// on an object or player".
///
/// [`DamageTarget`]'s shape, for the same reason it has one: the two halves are
/// ids, CR 616.1's chooser is the object's controller for one and the player
/// for the other, and a `PlayerState` holds counters of the same
/// `CounterType` a permanent does because CR 701.34a's proliferate sweeps
/// "permanents and/or players" in one pass. `Copy` because a rewritten
/// proposal carries it while the original is still read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CounterSubject {
    Object(ObjectId),
    Player(PlayerId),
}

/// A record's position in the log — its monotonic sequence number, which is
/// today the index and is the trace's `seq`. What a [`TriggerBinding`]
/// points at instead of copying the record: `EventLog::record(seq)` is the
/// read, and no record a pending or stacked trigger references may be
/// evicted (`triggers-architecture.md` §3.4).
///
/// [`TriggerBinding`]: crate::types::triggers::TriggerBinding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventSeq(pub usize);

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
/// The trigger matcher (critical path item 6) is the customer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BatchId(pub u64);

/// Which resolution an event belongs to (CR 608.2).
///
/// `source` is the stack object that was resolving, and it identifies the
/// *resolution* rather than the card, because this engine gives every stack
/// object a fresh `ObjectId`: one per cast, and one per activation for an
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

    /// Emit with no batch and no resolution whatever is ambient — for a
    /// record that is a consequence of an event rather than part of it
    /// (`GameEvent::AbilityTriggered`, §4.8).
    pub(crate) fn emit_unstamped(&mut self, event: GameEvent) {
        self.records.push(EventRecord { event, stamp: EventStamp::default() });
    }

    /// The record at `seq`, or `None` for a sequence the log does not hold.
    pub fn record(&self, seq: EventSeq) -> Option<&EventRecord> {
        self.records.get(seq.0)
    }

    /// The sequence number the next emitted record will carry.
    pub fn next_seq(&self) -> EventSeq {
        EventSeq(self.records.len())
    }

    /// The stamp the next emitted event will carry — the trace sink's join
    /// key between a batch record and the events performed inside it.
    pub fn current_stamp(&self) -> EventStamp {
        self.stamp
    }

    /// Open a batch, returning the stamp to hand back to [`Self::close_batch`].
    ///
    /// **A nested call joins the enclosing batch rather than opening a new
    /// one.** CR 120.3 and 120.4 are why it has to: a damage event's results
    /// (the life loss, the counters, lifelink's gain) are processed as part of
    /// that one event (CR 120.4c) before it occurs (CR 120.4d). They are
    /// proposed from inside the damage's batch, so a second batch id would
    /// split one event into two.
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
    /// The trigger matcher (critical path item 6) is the production consumer;
    /// keep the surface small until it lands.
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
        log.emit(GameEvent::PhaseBegin { phase: PhaseType::Beginning, player: 0 });

        assert_eq!(log.len(), 2);
        assert!(!log.is_empty());
    }

    #[test]
    fn test_event_log_since() {
        let mut log = EventLog::new();
        log.emit(GameEvent::TurnBegin { player: 0, turn_number: 1 });
        log.emit(GameEvent::PhaseBegin { phase: PhaseType::Beginning, player: 0 });
        log.emit(GameEvent::StepBegin { step: StepType::Untap, player: 0 });

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
        log.emit(GameEvent::Tapped { object_id: ObjectId::UNASSIGNED });
        log.emit(GameEvent::Untapped { object_id: ObjectId::UNASSIGNED });
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
