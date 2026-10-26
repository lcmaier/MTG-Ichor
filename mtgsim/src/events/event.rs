use std::num::NonZeroU32;
use std::sync::Arc;

use super::cause::EventCause;
use super::recorder::RecorderHandle;
use crate::types::ids::{DelayedTriggerId, ObjectId, ObjectOrPlayer, ObjectRef, PlayerId, UntilReturnId, ZoneChangeEpoch};
use crate::types::effects::CounterType;
use crate::types::zones::Zone;
use crate::types::mana::ManaType;
use crate::engine::actions::ZoneChangeAct;
use crate::engine::replacement::ReplacementInstanceId;
use crate::engine::last_known::LkiStore;
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::{AbilityIdentity, PhaseType, StepType};
use crate::objects::card_data::AbilityText;
use crate::types::triggers::{DelayedDuration, Lki, TriggerOrigin, TriggerId};


/// The object a move took out of a zone, as its record names it: which
/// existence of it left (an object that moves is a new object, CR 400.7) and
/// who owns it (CR 108.3).
///
/// Every object that leaves a zone has last known information, but a move's
/// record carries it only where a trigger on that move looks back in time
/// (CR 603.10a): a permanent leaving the battlefield, a card leaving a
/// graveyard, an object all players can see put into a hand or library. A
/// trigger on any other move (a card milled, a spell resolving into a
/// graveyard, a card exiled from a hand) sees the object as it is after the
/// move, so that record has no LKI, and `WithoutLki` holds the existence and
/// the owner alone. Where the record has one, the LKI names both already, so
/// `WithLki` is all it holds and neither fact is written twice. Something
/// that reads a departed object later, such as a trigger waiting to go on
/// the stack, finds its LKI in the store (`GameState::lki`), not here.
///
/// Not the mover's `Departure` (`engine::last_known`): the before-move half
/// that hands this record its LKI and keeps what the move unattaches.
#[derive(Debug, Clone, PartialEq)]
pub enum DepartedObject {
    WithLki(Lki),
    WithoutLki { object: ObjectRef, owner: PlayerId },
}

impl DepartedObject {
    /// The existence that left.
    pub fn object(&self) -> ObjectRef {
        match self {
            DepartedObject::WithLki(lki) => lki.object(),
            DepartedObject::WithoutLki { object, .. } => *object,
        }
    }

    pub fn owner(&self) -> PlayerId {
        match self {
            DepartedObject::WithLki(lki) => lki.owner(),
            DepartedObject::WithoutLki { owner, .. } => *owner,
        }
    }

    /// The frame the move carries, if it carries one.
    pub fn lki(&self) -> Option<&Lki> {
        match self {
            DepartedObject::WithLki(lki) => Some(lki),
            DepartedObject::WithoutLki { .. } => None,
        }
    }

    /// The object `object`, owned by `owner`, which left with `frame` if the
    /// move framed it.
    pub fn new(object: ObjectRef, owner: PlayerId, frame: Option<Lki>) -> DepartedObject {
        match frame {
            Some(lki) => {
                debug_assert!(lki.object() == object && lki.owner() == owner, "{} left with another object's frame", object.id);
                DepartedObject::WithLki(lki)
            }
            None => DepartedObject::WithoutLki { object, owner },
        }
    }
}

/// Game events that can be observed by triggered abilities and logging systems.
///
/// Events are emitted *after* the action occurs (past tense). They represent
/// facts about what happened, not requests for what should happen. What a
/// replacement effect reads is the proposal before it is performed
/// (`engine::actions::GameAction`, CR 614), never one of these.
#[derive(Debug, Clone, PartialEq)]
pub enum GameEvent {
    // --- Zone transitions ---
    ZoneChange {
        /// The existence that moved and its owner, and its frame: CR
        /// 603.10a's characteristics an instant *before* it moved, for a move
        /// in one of the rule's three classes (`zone_change_looks_back`):
        /// leaving the battlefield, a card leaving a graveyard, an object all
        /// players could see put into a hand or library, and for any mover a
        /// keeper reads. Unframed for any other move, which no trigger looks
        /// back at.
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
        /// The frame is behind an `Arc`, and the layer memo's own: the widest
        /// type in the engine is shared rather than copied, so a trigger
        /// binding that copies this record copies a pointer. It is what a dies
        /// trigger's "it deals damage" was (CR 608.2h).
        departed: DepartedObject,
        from: Zone,
        to: Zone,
        /// What was done (`replacement-architecture.md` §2b.2). A redirect
        /// keeps it (CR 614.6).
        ///
        /// **Only the readers [`ZoneChangeAct`](crate::engine::actions::ZoneChangeAct)
        /// names may branch on this.**
        act: ZoneChangeAct,
        /// The replacement effects that applied to this move, in the order
        /// they applied (CR 614.5), each as it applied. `None` for a move
        /// nothing replaced.
        applied: Option<Arc<[AppliedReplacement]>>,
        /// CR 400.7e's new object: the epoch the move stamped, which
        /// [`GameEvent::new_object`] pairs with the object's id. A later move
        /// makes another object, which "it" never finds (CR 603.6c).
        new_epoch: ZoneChangeEpoch,
    },

    /// A permanent became tapped (CR 701.26a).
    ///
    /// **Only on the transition.** CR 603.2e: "becomes tapped" triggers only
    /// when a permanent already on the battlefield changes from untapped to
    /// tapped — it does not fire for a redundant tap, and does not fire for a
    /// permanent that *enters* tapped. CR 701.26a agrees from the other side:
    /// only untapped permanents can be tapped at all.
    Tapped { object: ObjectRef },

    /// A permanent became untapped (CR 701.26b). Transition-only, as `Tapped`.
    Untapped { object: ObjectRef },

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
    ///
    CardDrawn { player_id: PlayerId, card: ObjectRef },

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
        source: ObjectRef,
        mana: Vec<(ManaType, u64)>,
        tapped_for_mana: bool,
        /// The source as it last existed, where its own mana ability's cost
        /// moved it first (CR 608.2h).
        source_lki: Option<Lki>,
    },

    // --- Damage ---
    /// `is_combat` is the proposal's, carried because it is not
    /// live-derivable: a triggered ability resolving during the combat damage
    /// step deals noncombat damage in that step (`codebase-state.md` "Before
    /// Triggered abilities" item 10).
    DamageDealt {
        /// The existence that dealt it: as it last existed where it left
        /// first, which `source_lki` frames.
        source: ObjectRef,
        recipient: ObjectOrPlayer,
        amount: u64,
        /// CR 120.10: how much of `amount` was past the damage still lethal
        /// to the permanent after the batch's earlier damage to it, so its
        /// records' excess sums to the event's. 0 for a player, and for a
        /// permanent that is neither a creature nor a planeswalker.
        excess: u64,
        is_combat: bool,
        /// The proposal's `source_lki`: the source as it last existed, when
        /// it dealt the damage after leaving its zone, so a trigger asking
        /// what dealt it reads what the damage's results read (CR 608.2h).
        source_lki: Option<Lki>,
    },
    /// A prevention effect applied, preventing `amount` of the damage that
    /// would be dealt to `target` (CR 615.13): one record each time an effect
    /// prevents some or all of one subject's damage in a batch. `by` is the
    /// effect, which "prevented this way" reads; there is no source, since one
    /// subject's damage can come from several (two attackers into a Fog).
    /// **Announced while its batch decides**, ahead of what the batch
    /// performs: the prevented damage is never performed (CR 615.6), and
    /// the application is the event.
    DamagePrevented { recipient: ObjectOrPlayer, amount: u64, by: ReplacementInstanceId },

    // --- Turn structure ---
    //
    // `player` is whose phase or step it is, which "at the beginning of your
    // upkeep" reads. No record marks an end, since no printed trigger reads
    // one: "at end of combat" is the end-of-combat step beginning (CR 511.2)
    // and "at end of turn" the end step's (CR 513.1a).
    PhaseBegin { phase: PhaseType, player: PlayerId },
    StepBegin { step: StepType, player: PlayerId },
    TurnBegin { player: PlayerId, turn_number: u32 },

    // --- Permanents ---
    PermanentEnteredBattlefield { object: ObjectRef, controller: PlayerId },

    // --- Life ---
    /// `source` is a gain's source, or the source of the damage a loss is a
    /// result of (CR 120.3a), as the existence it is or last was: `None` for
    /// one that left the game unframed, which no existence names (CR 800.4a).
    /// What caused it — an effect, a cost, combat damage
    /// — is the record's cause, which CR 727.1a's "from radiation" reads
    /// (item 10).
    LifeChanged {
        player_id: PlayerId,
        old: i64,
        new: i64,
        source: Option<ObjectRef>,
    },

    // --- Combat ---
    /// CR 508.1: each creature declared as an attacker, with what it attacks
    /// (CR 508.1b). A creature put onto the battlefield attacking is in no
    /// record: it never "attacked" (CR 508.4). What it attacks is the
    /// combat's own `AttackTarget`, a permanent by id: the record is
    /// unbatched, so its dispatch reads that permanent at the record's moment.
    AttackersDeclared { attackers: Vec<(ObjectRef, AttackTarget)> },
    /// CR 509.1: every defending player's blocks (CR 802.4), each as (blocker,
    /// attacker), and each attacking creature no creature was declared
    /// blocking, which became an unblocked creature (CR 509.1h). One record
    /// for the step, written when nobody blocks too, since CR 509.3g's
    /// "attacks and isn't blocked" reads the declaration whole.
    BlockersDeclared { blocks: Vec<(ObjectRef, ObjectRef)>, unblocked: Vec<ObjectRef> },
    /// CR 509.1h: an unblocked attacking creature became blocked by an
    /// effect, no creature blocking it. "Whenever this becomes blocked"
    /// reads it (CR 509.3c); "becomes blocked by a creature" never does
    /// (509.3d).
    BecameBlocked { attacker: ObjectRef },
    /// CR 506.4, 511.3: permanents removed from combat at once, each with
    /// what it was attacking or blocking as it was removed, which CR 508.5
    /// reads once its creature has left combat. No replaceable event
    /// (`replacement-architecture.md` §8a): announced by each cause after it
    /// removes them. A permanent that leaves the battlefield is removed by its
    /// zone change, whose frame keeps the attack, and has no record here.
    RemovedFromCombat { removed: Vec<CombatExit> },

    // --- Spells ---
    /// `spell` is the existence the cast made (CR 400.7).
    SpellCast { spell: ObjectRef, caster: PlayerId },

    // There is no `SpellResolved`/`StackObjectResolved`: a spell finishing
    // resolution is a `ZoneChange` out of the stack caused by CR 608.2n's or
    // 608.3's rule (`GameRule`), which also says *where* it went, and an
    // ability finishing is `AbilityResolved`, which carries the durable identity
    // CR 603.7h counting needs. One event for both would fire for abilities
    // too, and "whenever a spell resolves" keyed on it would be wrong.
    /// An activated or triggered ability finished resolving (CR 608.2n),
    /// identified by what it *is* rather than by the stack object that
    /// represented it, which ceases to exist as it resolves. CR 603.7h's
    /// counting — "whenever this ability resolves for the third time this
    /// turn" (Ashling the Pilgrim; Ashling, Flame Dancer) — needs the durable
    /// (source, ability) pair.
    AbilityResolved { identity: AbilityIdentity, controller: PlayerId },
    /// An activated ability became activated: its costs are paid (CR 602.2b
    /// runs 601.2i), so an activation reversed for want of them (CR 732.1)
    /// announces nothing. A mana ability is activated too (CR 605.3a), and
    /// announces this before it resolves (CR 605.3b). `from` is the zone its
    /// source was in as it was activated, and `source_lki` the source as it
    /// was then when a cost moved it, which "an ability of an artifact" reads.
    AbilityActivated {
        identity: AbilityIdentity,
        controller: PlayerId,
        mana_ability: bool,
        from: Zone,
        source_lki: Option<Lki>,
    },
    /// An object or player became the target of a spell or ability: one
    /// record per (spell or ability, distinct target), `instances` how many
    /// of its instances of "target" chose it (CR 115.9a counts each).
    /// Announced as the announcement is final, ahead of `SpellCast` (CR
    /// 601.2i) and `AbilityActivated` (602.2b), and as a triggered ability
    /// goes on the stack (603.3d). A choice that does not target announces
    /// nothing (CR 115.10a).
    ///
    /// `ability_source` is an ability's source, as the existence the ability
    /// is of, which a reader names it by: `by` is the stack object, which CR
    /// 608.2n removes. `None` for a spell. Not the whole `AbilityIdentity`,
    /// which would grow every event by half.
    Targeted { target: ObjectOrPlayer, by: ObjectRef, ability_source: Option<ObjectRef>, controller: PlayerId, instances: u32 },
    /// An ability triggered (CR 603.2) — the record CR 603.3b's second tier
    /// watches, emitted by the dispatcher once per queued trigger after the
    /// window it belongs to has closed (`triggers-architecture.md` §4.8).
    /// **Unstamped**: it carries no batch and no resolution, because it is a
    /// consequence of the event and not part of it, and the dispatcher reads
    /// it like any unbatched record. `caused_by` is the record that matched
    /// (the first of them, for a "one or more" trigger), and `None` for a
    /// state trigger, which no record caused (CR 603.8).
    AbilityTriggered {
        trigger: TriggerId,
        origin: TriggerOrigin,
        controller: PlayerId,
        caused_by: Option<EventId>,
    },
    /// A delayed triggered ability was created (CR 603.7a), by
    /// `GameState::register_delayed_trigger`, ahead of any record it can
    /// trigger on. A record for a log and a client: nothing triggers on it.
    /// `source` and `controller` are CR 603.7d–g's.
    DelayedTriggerCreated {
        id: DelayedTriggerId,
        source: ObjectRef,
        controller: PlayerId,
        rules_text: AbilityText,
        duration: DelayedDuration,
    },
    /// An "until" exile made the return it waits to make (CR 610.3), by
    /// `GameState::wait_to_return`, ahead of any record its event can be. A
    /// record for a log and a client, as [`Self::DelayedTriggerCreated`] is.
    /// `source` is the exiling spell or ability's source, `returns` what it
    /// exiled.
    UntilReturnMade { id: UntilReturnId, source: ObjectRef, controller: PlayerId, returns: Vec<ObjectRef> },
    /// A spell was countered (CR 701.6a). `lki` is the spell as it was on the
    /// stack, its entry included, which "whenever a spell is countered" looks
    /// back at (CR 603.10e): the existence it was, who cast it, its
    /// controller as it left the stack (in its owner's graveyard it has none,
    /// CR 108.4a), and what it was before it became a card in a graveyard.
    SpellCountered { countered_by: ObjectRef, lki: Lki },
    /// An activated or triggered ability was countered (CR 701.6b), named by
    /// what it is: the stack object ceased to exist as it was countered.
    AbilityCountered { identity: AbilityIdentity, controller: PlayerId, countered_by: ObjectRef },
    /// A spell doesn't resolve (CR 608.2b): every target of every instance of
    /// "target" is illegal. Its move to the graveyard, the `ZoneChange` caused
    /// by `GameRule::TargetsIllegal`, comes first. Not a countering: CR 701.6a's
    /// counter is an effect's, and "whenever a spell is countered" does not
    /// read this. `controller` as [`Self::SpellCountered`]'s, and `spell` the
    /// existence that was on the stack, named before its move.
    SpellFizzled { spell: ObjectRef, controller: PlayerId },
    /// An ability doesn't resolve (CR 608.2b), named by what it is, as
    /// [`Self::AbilityResolved`] is: CR 608.2b removes the stack object, so
    /// its id names nothing once this is read.
    AbilityFizzled { identity: AbilityIdentity, controller: PlayerId },

    // --- Deaths are not events of their own ---
    //
    // There is no `CreatureDied`, `PlaneswalkerDied`, `LegendRuleSacrificed` or
    // `AuraDied`. Each is a `ZoneChange { from: Battlefield, to: Graveyard }`
    // with an `act` and an `lki` frame, said less precisely:
    //
    // - **They would partition one event by type, and permanent types are not
    //   a partition.** A Gideon is a creature *and* a planeswalker; the `lki`
    //   frame carries the whole type set, so one event answers for all of them.
    // - **They would name a subset without naming its boundary.** "Dies" is
    //   battlefield → graveyard; "leaves the battlefield" is battlefield →
    //   anywhere; and ATOM-603.6c-001 turns on *which* zone the card went to.
    //
    // A reader that wants deaths matches `ZoneChange { from: Battlefield, to:
    // Graveyard, lki, .. }` and asks the frame what died.

    // --- The game's end ---
    /// Emitted by the `GameAction::PlayerLoses` performer, so a loss a
    /// replacement effect prevented (Exquisite Archangel) or a "can't" refused
    /// (Platinum Angel) announces nothing.
    /// Why is the record's cause: the state-based action's subrule, or the
    /// effect that said so (CR 104.3e).
    PlayerLost { player_id: PlayerId },
    /// CR 104.2b — an effect stated that this player wins. Emitted by the
    /// `GameAction::PlayerWins` performer. CR 104.2a's win — the last player
    /// standing — is not an event: it is the outcome the losses imply, recorded
    /// on `GameState::result` when the batch that performed them settles.
    PlayerWon { player_id: PlayerId },
    /// CR 104.4 — the game is a draw. Emitted by the
    /// `GameAction::GameEndsInADraw` performer; a draw CR 104.4a's
    /// simultaneous losses imply is recorded when their batch settles, as
    /// CR 104.2a's win is.
    GameEndedInADraw,

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
    /// Counters were put on a permanent or given to a player (CR 122.1,
    /// 122.6).
    ///
    /// One record kind per direction, as the proposals are (`AddCounters`,
    /// `RemoveCounters`), because the two carry different facts: a put always
    /// has a putter, and CR 122.7's "the Nth counter" compares the counts
    /// around one. One kind for both subjects: "whenever one or more counters
    /// are put on a permanent" and "whenever you get one or more counters"
    /// read the same record and differ in the subject they ask for.
    CountersPutOn {
        subject: ObjectOrPlayer,
        counter_type: CounterType,
        /// How many were put on: never 0, since putting none announces
        /// nothing.
        n: u32,
        /// The player who put them on: the instruction's controller (CR
        /// 608.2c), an entry's named player or its controller (CR 122.6a),
        /// a damage source's controller (CR 120.3d).
        by: PlayerId,
        /// The subject's count of `counter_type` after them, which CR 122.7
        /// compares with the count before (`now - n`).
        now: u32,
    },

    /// Counters were taken off a permanent or away from a player (CR 122.1).
    /// Emitted only on an actual change: `RemoveCounters` reports how many
    /// were really there (CR 609.3's "only as much as possible"), and removing none
    /// announces nothing, the same transition rule CR 603.2e gives tapping.
    CountersRemovedFrom {
        subject: ObjectOrPlayer,
        counter_type: CounterType,
        /// How many were removed, never 0.
        n: u32,
        /// The player who removed them, where an instruction, an effect or a
        /// cost did (CR 608.2c). `None` where a rule removes them and names no
        /// player: CR 704.5q, and damage to a planeswalker (CR 120.3c).
        by: Option<PlayerId>,
        /// The subject's count of `counter_type` after them.
        now: u32,
    },

    // --- Attachment ---
    /// An Aura, Equipment or Fortification became attached to `host`
    /// (CR 701.3a), leaving `former_host` if it was attached before. Emitted
    /// by the `GameAction::Attach` performer on the transition only — an
    /// attach to the host it is already on is CR 701.3b's "does nothing" and
    /// announces nothing.
    Attached { attachment: ObjectRef, host: ObjectRef, former_host: Option<ObjectRef> },

    /// An Aura, Equipment or Fortification stopped being attached to
    /// `former_host` (CR 701.3d), by `route`. Each is the existence it was at
    /// the event (CR 400.7): a host that left is gone by the time a trigger's
    /// "that permanent" is read (Grafted Wargear's first ruling). What caused
    /// it is the stamp's, not the route's.
    ///
    /// Whether each end left the battlefield in this event (CR 603.10c): for
    /// the two leaving routes the end whose departure unattached them, and
    /// the other end too when the same event takes it, such as an Aura and its
    /// host leaving with their owner (CR 800.4a). An end that left is read as
    /// it left, off the LKI store (`frame_of`).
    Unattached {
        attachment: ObjectRef,
        former_host: ObjectRef,
        route: UnattachRoute,
        attachment_left: bool,
        former_host_left: bool,
    },

    /// A permanent's controller became `to` (CR 613.1b): a control-changing
    /// effect started or ended. No performer does this, so
    /// `announce_control_changes` finds it, comparing each permanent with the
    /// controller the stream last announced; one sweep's changes are one
    /// event. `lki` is the permanent as it was before, which "when you lose
    /// control of" and "an opponent gains control of" look back to (CR
    /// 603.10d): the existence, and the controller it had, the one it went
    /// from.
    ControlChanged { to: PlayerId, lki: Lki },

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
    /// object that was not a permanent, which is every other zone: a card
    /// leaving the game from a graveyard leaves no graveyard (Erebos's Titan's
    /// third ruling). `departed` holds the frame where the move carries
    /// one, and the existence and its owner where it does not.
    ///
    /// CR 603.6c's qualifier is the one part with no implementation: a *phased
    /// out* permanent does not trigger, and phasing (CR 702.26) is not built
    /// (`codebase-state.md`, "Phasing"). Every permanent is phased in today.
    LeftTheGame {
        departed: DepartedObject,
        from: Zone,
    },

    /// A move a replacement sent back into the zone it was leaving: Darksteel
    /// Colossus milled, and shuffled into its library instead. The act it
    /// replaced still happened (CR 614.6; the card "is still milled", Jess
    /// Dunks on Zellix, Sanity Flayer), so `act` and `applied` say what was
    /// done and what replaced it, and `lki` is the card as it was, the
    /// existence and its owner. **Not a zone change**: nothing left a zone
    /// and no new object was made (CR 400.7), so it is no departure, and only
    /// an arm that names an act reads it (`triggers-architecture.md` §3.12a
    /// decision 3).
    SameZoneMove {
        zone: Zone,
        act: ZoneChangeAct,
        applied: Option<Arc<[AppliedReplacement]>>,
        lki: Lki,
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
    /// a token created in exile has no controller to name. `definition` is
    /// what the creation made it from, after any replacement substituted
    /// another (Divine Visitation), which "when you do" after "create a
    /// [token]" asks (Ajani, Nacatl Avenger's eighth ruling).
    TokenCreated { token: ObjectRef, owner: PlayerId, zone: Zone, definition: Arc<crate::types::effects::TokenDef> },

    /// A token in a non-battlefield zone ceased to exist (rule 704.5d).
    /// Not a zone change — the token is simply removed from the game.
    /// `token` is the existence that ended, named before the store lost it.
    TokenCeasedToExist { token: ObjectRef },

    // --- State-based ---
    StateBasedActionPerformed,
}

/// A life total's change (CR 119.3), by its direction: what "gains life"
/// and "loses life" each read (CR 119.9 makes each source's its own event).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifeChange {
    Gained(u64),
    Lost(u64),
}

impl GameEvent {
    /// The life change this record is; `None` for any other record. A 0
    /// change is no event and never reaches the log (CR 119.10).
    pub fn life_change(&self) -> Option<LifeChange> {
        let GameEvent::LifeChanged { old, new, .. } = self else { return None };
        match new.cmp(old) {
            std::cmp::Ordering::Greater => Some(LifeChange::Gained(new.abs_diff(*old))),
            std::cmp::Ordering::Less => Some(LifeChange::Lost(new.abs_diff(*old))),
            std::cmp::Ordering::Equal => None,
        }
    }

    /// The card or permanent this zone change put into its destination, as an
    /// `ObjectRef`: its id with the epoch the move gave it. A moved object is a
    /// new object (CR 400.7), so this is how "return it" after a dies trigger
    /// finds the card in the graveyard, and finds nothing once it has moved
    /// again (CR 603.6c). `None` for a record that is not a zone change.
    pub fn new_object(&self) -> Option<ObjectRef> {
        match self {
            GameEvent::ZoneChange { departed, new_epoch, .. } => {
                Some(ObjectRef { id: departed.object().id, zone_change_epoch: *new_epoch })
            }
            _ => None,
        }
    }

    /// The last known information this record keeps of its source, which had
    /// left before the event: a damage source gone before its damage (CR
    /// 608.2h), an ability's source a cost moved before it was activated (CR
    /// 602.2b), or a mana source its own mana ability's cost moved before the
    /// mana was made. `None` where the source was where the event found it.
    /// Not [`Self::lki`], which is the object the event itself moved.
    pub fn source_lki(&self) -> Option<&Lki> {
        match self {
            GameEvent::DamageDealt { source_lki, .. }
            | GameEvent::AbilityActivated { source_lki, .. }
            | GameEvent::ManaAdded { source_lki, .. } => source_lki.as_ref(),
            _ => None,
        }
    }

    /// The last known information this record carries for `id`, its object
    /// as it last existed before the event (CR 113.7a): a source gone before
    /// it (`source_lki`), the object the event itself moved (`lki`), or an
    /// unattach's end that left in the event, as `store` keeps it. `None` for
    /// an object the record keeps no frame of.
    pub fn frame_of<'a>(&'a self, id: ObjectId, store: &'a LkiStore) -> Option<&'a Lki> {
        match self {
            GameEvent::Unattached { attachment, former_host, attachment_left, former_host_left, .. } => {
                [(attachment, attachment_left), (former_host, former_host_left)]
                    .into_iter()
                    .find(|&(end, left)| *left && end.id == id)
                    .and_then(|(end, _)| store.of(*end))
            }
            _ => self.source_lki().or_else(|| self.lki().map(|(lki, ..)| lki)).filter(|lki| lki.object().id == id),
        }
    }

    /// The source of the ability this record is about, as the existence that
    /// activated or triggered it.
    pub fn ability_source(&self) -> Option<ObjectRef> {
        match self {
            GameEvent::AbilityTriggered { origin, .. } => Some(origin.source_ref()),
            GameEvent::AbilityActivated { identity, .. }
            | GameEvent::AbilityResolved { identity, .. }
            | GameEvent::AbilityCountered { identity, .. }
            | GameEvent::AbilityFizzled { identity, .. } => Some(identity.source),
            _ => None,
        }
    }

    /// The last known information this record carries (CR 603.10), with the
    /// zone its object left and its owner: a zone change's or a leaving's
    /// (603.10a), a countered spell's (603.10e), and `None` for every other
    /// record and for a move no look-back reads.
    pub fn lki(&self) -> Option<(&Lki, Zone, PlayerId)> {
        match self {
            GameEvent::ZoneChange { departed: DepartedObject::WithLki(lki), from, .. }
            | GameEvent::LeftTheGame { departed: DepartedObject::WithLki(lki), from } => Some((lki, *from, lki.owner())),
            GameEvent::SpellCountered { lki, .. } => Some((lki, Zone::Stack, lki.owner())),
            _ => None,
        }
    }

    /// Calls `each` with every object this event names, as it is when the
    /// event is announced: the objects whose names its record keeps where a
    /// copy makes one other than its card's ([`NamesAsAnnounced`]). A zone
    /// change names its object as it was in the zone it left instead, off its
    /// look-back frame or its card, so it calls nothing here.
    pub fn for_each_object_named(&self, mut each: impl FnMut(ObjectId)) {
        use GameEvent::*;
        match self {
            Tapped { object }
            | Untapped { object }
            | PermanentEnteredBattlefield { object, .. }
            | TokenCreated { token: object, .. }
            | TokenCeasedToExist { token: object }
            | CardDrawn { card: object, .. }
            | ManaAdded { source: object, .. }
            | SpellCast { spell: object, .. }
            | SpellFizzled { spell: object, .. }
            | CountersPutOn { subject: ObjectOrPlayer::Object(object), .. }
            | CountersRemovedFrom { subject: ObjectOrPlayer::Object(object), .. }
            | DamageDealt { source: object, recipient: ObjectOrPlayer::Player(_), .. }
            | DamagePrevented { recipient: ObjectOrPlayer::Object(object), .. }
            | BecameBlocked { attacker: object } => each(object.id),
            AbilityActivated { identity, .. } | AbilityResolved { identity, .. } | AbilityFizzled { identity, .. } => {
                each(identity.source.id)
            }
            AbilityCountered { identity, countered_by, .. } => {
                each(identity.source.id);
                each(countered_by.id);
            }
            Targeted { target, by, ability_source, .. } => {
                if let ObjectOrPlayer::Object(object) = target {
                    each(object.id);
                }
                each(ability_source.unwrap_or(*by).id);
            }
            AbilityTriggered { origin, .. } => each(origin.source()),
            DelayedTriggerCreated { source, .. } => each(source.id),
            UntilReturnMade { source, returns, .. } => {
                each(source.id);
                for returned in returns {
                    each(returned.id);
                }
            }
            LifeChanged { source, .. } => {
                if let Some(source) = source {
                    each(source.id);
                }
            }
            DamageDealt { source, recipient: ObjectOrPlayer::Object(recipient), .. } => {
                each(source.id);
                each(recipient.id);
            }
            SpellCountered { countered_by, lki } => {
                each(lki.object().id);
                each(countered_by.id);
            }
            SameZoneMove { lki, .. } | ControlChanged { lki, .. } => each(lki.object().id),
            Unattached { attachment, former_host, .. } => {
                each(attachment.id);
                each(former_host.id);
            }
            Attached { attachment, host, former_host } => {
                each(attachment.id);
                each(host.id);
                if let Some(former_host) = former_host {
                    each(former_host.id);
                }
            }
            // Each attacker, and the planeswalker or battle it attacks.
            AttackersDeclared { attackers } => {
                for &(attacker, target) in attackers {
                    each(attacker.id);
                    if let Some(attacked) = target.permanent() {
                        each(attacked);
                    }
                }
            }
            BlockersDeclared { blocks, unblocked } => {
                for &(blocker, attacker) in blocks {
                    each(blocker.id);
                    each(attacker.id);
                }
                for attacker in unblocked {
                    each(attacker.id);
                }
            }
            // Each permanent removed, and what it attacked or blocked.
            RemovedFromCombat { removed } => {
                for exit in removed {
                    each(exit.object.id);
                    if let Some(attacked) = exit.attacking.and_then(AttackTarget::permanent) {
                        each(attacked);
                    }
                    for attacker in &exit.blocking {
                        each(attacker.id);
                    }
                }
            }
            ZoneChange { .. }
            | LeftTheGame { .. }
            | CountersPutOn { .. }
            | CountersRemovedFrom { .. }
            | DamagePrevented { recipient: ObjectOrPlayer::Player(_), .. }
            | PhaseBegin { .. }
            | StepBegin { .. }
            | TurnBegin { .. }
            | PlayerLost { .. }
            | PlayerWon { .. }
            | GameEndedInADraw
            | Scried { .. }
            | LibraryShuffled { .. }
            | StateBasedActionPerformed => {}
        }
    }
}

/// One permanent a [`GameEvent::RemovedFromCombat`] removed, as it was in
/// combat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatExit {
    /// The existence removed (CR 400.7).
    pub object: ObjectRef,
    /// What it was attacking, or `None` for a blocker, or for an attacker
    /// whose planeswalker or battle had left combat first (CR 506.4c).
    pub attacking: Option<AttackTarget>,
    /// The attackers it was blocking.
    pub blocking: Vec<ObjectRef>,
}

/// Which of CR 701.3d's ways an attachment stopped being attached to its
/// host. In the first two both stay on the battlefield, and the name says
/// what the attachment is attached to afterwards; in the last two one of
/// them left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnattachRoute {
    /// To nothing: CR 701.3d's "unattach" instruction, or CR 704.5n's or
    /// 704.5p's state-based action.
    AttachedToNothing,
    /// To a different object (CR 701.3c).
    AttachedElsewhere,
    /// The attachment left the battlefield.
    AttachmentLeft,
    /// The object it was attached to left the battlefield.
    HostLeft,
}

/// Which performed event a record is. Ids are handed out in the order events
/// are performed, so a smaller id happened earlier, and the trace prints it
/// as `index`. Never a position in a buffer: the window flushes and the
/// recorder forks, and a record keeps its id through both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventId(pub usize);

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
///
/// Not ordered: the moves and payments a batch's entries decide on are
/// numbered after that batch and performed before it, so a larger id is not a
/// later event. The stream's order is [`EventId`]'s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BatchId(pub u64);

/// When something waiting for an event was created, as a position in the
/// event stream: a delayed trigger (CR 603.7a) or the return an "until" exile
/// waits to make (CR 610.3). Not a CR 613.7 timestamp, which orders
/// permanents and effects; this orders records. "Won't trigger until it has actually been created, even if
/// its trigger event occurred just beforehand": a record before it is no
/// event it waits for, and neither is one of the batch it was created
/// during. A rider (CR 615.5) creates one inside its own batch, on both
/// sides of its records, and that whole event happened just beforehand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatedAt {
    /// The first record performed after it was created.
    pub first_record: EventId,
    pub batch: Option<BatchId>,
}

impl CreatedAt {
    /// The position something created now has: the next record's id, and
    /// the batch open now, if one is.
    pub fn now(window: &EventWindow) -> CreatedAt {
        CreatedAt { first_record: window.next_seq(), batch: window.current_batch() }
    }

    /// Whether it was created before the event `record` is part of.
    pub fn is_before(self, record: &EventRecord) -> bool {
        record.seq >= self.first_record && !self.batch.is_some_and(|batch| record.stamp.batch == Some(batch))
    }
}

/// Which resolution an event belongs to (CR 608.2).
///
/// `source` is the stack object that was resolving: a spell's card, which keeps
/// its id as it moves to the stack (CR 601.2a), or an ability's own object, a
/// fresh id that CR 608.2n destroys. A card cast twice stamps two resolutions
/// alike, so a reader of what one resolution did bounds it by the mark taken
/// as it began ([`EventWindow::resolution_records`]).
///
/// **Who reads it.** It is [`EventCause::Effect`]'s payload. On the performed
/// record it is how a resolution reads back what its own instructions did:
/// `Effect::Remember`'s objects (CR 603.7c). It is also provenance, wanted by
/// the CR 731 loop-detection transcripts and useful in a log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolutionStamp {
    pub source: ObjectId,
    pub controller: PlayerId,
}

/// One replacement effect that applied to an event, as it applied (CR 614.5;
/// `replacement-architecture.md` §2b.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppliedReplacement {
    pub instance: ReplacementInstanceId,
    /// What its effect's own events are caused by: its source and its
    /// controller as it applied, which may have left or changed hands since.
    pub by: EventCause,
    /// Whether it changed where the move goes, for the log. Who exiled it is
    /// still the record's cause, the resolving spell's or ability's, a redirect
    /// included (`replacement-architecture.md` §2b.4).
    pub redirected: bool,
}

/// The context every emitted event is stamped with.
///
/// An envelope rather than fields on each `GameEvent` variant: it is the same
/// two facts for every kind of event, and a variant that forgets to carry them
/// fails silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventStamp {
    /// The batch this event was performed as part of. `None` for an event
    /// emitted outside `GameState::execute_actions` — a spell being cast, an
    /// activation, an ability triggering.
    pub batch: Option<BatchId>,
    /// Who or what caused it (`replacement-architecture.md` §2b.3), as its
    /// emitter names it: for a performed proposal, the proposal's cause.
    pub cause: EventCause,
}

/// One performed event: what happened, where in the stream, and the context
/// it happened in. A [`TriggerBinding`] copies the records it binds, so a
/// record outlives the window it was read in wherever a trigger needs it
/// (`triggers-architecture.md` §3.4).
///
/// [`TriggerBinding`]: crate::types::triggers::TriggerBinding
#[derive(Debug, Clone, PartialEq)]
pub struct EventRecord {
    /// Its place in the stream.
    pub seq: EventId,
    /// What happened.
    pub event: GameEvent,
    /// See [`EventStamp`].
    pub stamp: EventStamp,
    /// See [`NamesAsAnnounced`].
    pub names: NamesAsAnnounced,
    /// Where the history placed it, once it has counted it.
    pub(crate) places: RecordPlaces,
}

/// A record's places, written when the history counts it: as its window
/// is dispatched, or, for a card of a draw instruction, as that card's draw
/// ends. Four bytes a place, so the record keeps its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecordPlaces {
    in_turn: Option<NonZeroU32>,
    in_draw_step: Option<NonZeroU32>,
    counted: bool,
}

impl RecordPlaces {
    /// A record the history has not placed yet, which is every record as it
    /// is pushed.
    pub(crate) const NOT_COUNTED: RecordPlaces = RecordPlaces { in_turn: None, in_draw_step: None, counted: false };

    fn counted(in_turn: Option<u64>, in_draw_step: Option<u64>) -> Self {
        let place = |n: u64| NonZeroU32::new(u32::try_from(n).unwrap_or(u32::MAX));
        RecordPlaces { in_turn: in_turn.and_then(place), in_draw_step: in_draw_step.and_then(place), counted: true }
    }
}

/// The names an event's objects were announced under, where one was not its
/// card's: a copy's (CR 707.2), a face-down permanent's, which is none (CR
/// 708.2a), or one a text-changing effect wrote (CR 612.5–612.9). `None` on
/// nearly every record. Kept because the log and the trace format a record
/// after its objects may have changed; no rule reads it. Only those: every
/// other object's name is its card's, which the store keeps, so recording it
/// would cost a read and an allocation per object per event.
pub type NamesAsAnnounced = Option<Arc<Vec<(ObjectId, String)>>>;

impl EventRecord {
    /// The batch this event was performed as part of, if any.
    pub fn batch(&self) -> Option<BatchId> {
        self.stamp.batch
    }

    /// The resolution whose effect caused this event, if one did.
    pub fn resolution(&self) -> Option<ResolutionStamp> {
        self.stamp.cause.resolution()
    }

    /// Its place among its player's records of the same kind this turn, 1
    /// for the first: what "the first time each turn" reads
    /// (`triggers-architecture.md` §3.5). Only the kinds each player's turn
    /// tally counts have one (`state::history`) — a cast, a draw, a gain, a
    /// loss — and read by any trigger that matches the record then or later
    /// in the resolution (CR 603.12).
    pub fn place_in_turn(&self) -> Option<u64> {
        self.places.in_turn.map(|n| u64::from(n.get()))
    }

    /// A draw's place among its player's draws in their own draw step in
    /// progress (CR 504): what "the first one you draw in each of your draw
    /// steps" reads. `None` for any other record.
    pub fn place_in_draw_step(&self) -> Option<u64> {
        self.places.in_draw_step.map(|n| u64::from(n.get()))
    }

    /// Whether the history has counted it.
    pub(crate) fn counted(&self) -> bool {
        self.places.counted
    }
}

/// The performed stream's records until their last reader has run, and the
/// context the next one is stamped with (`triggers-architecture.md` §4.1).
///
/// **A window, not a log** (`codebase-state.md` item 42). A record is read by
/// the dispatch whose window it belongs to and by the dispatches nested inside
/// that one: an entry's own zone change, the record a tier-2 trigger's
/// `AbilityTriggered` names. Nothing reads it after the outermost of those
/// returns, which is where [`Self::flush`] runs, so at a decision outside a
/// batch this holds nothing and a clone copies nothing. What the rules read
/// from the past is materialized instead: the history log's rows
/// (`state::history_log`, §3.10), and the records a trigger binds (§3.4). What wants the whole stream attaches a
/// recorder, and the flush hands it every record in order.
#[derive(Debug, Clone, Default)]
pub struct EventWindow {
    records: Vec<EventRecord>,
    /// The sequence number the next record takes.
    next_seq: usize,
    /// The batch the next record joins; `None` outside every batch.
    ///
    /// **Ambient, and deliberately so.** Which batch a record belongs to is
    /// the batch's fact, not its emitter's: an emitter deep inside a performer
    /// cannot know whether it is the outermost event or a result nested in
    /// one (CR 120.4c). Scoping it to `execute_actions` means it is
    /// established once per batch, by the one function that knows it. The
    /// cause is the emitter's, and every emitter names it.
    ///
    /// This is not the ambient state `CLAUDE.md` bans. That rule is about
    /// `rand::rng()`: an ambient *source* silently changes the game's outcome
    /// and destroys replayability. This is an ambient *label* on a record —
    /// it changes what a record says about a mutation, never whether the mutation
    /// happens — and its lifetime is a single `open_batch`/`close_batch` pair
    /// that `execute_actions` opens and closes on the same code path.
    open: Option<BatchId>,
    /// Allocator for [`BatchId`]. Monotonic, never reused within a game.
    next_batch: u64,
    /// Where a flushed record goes, if anything wants the whole stream.
    recorder: Option<RecorderHandle>,
}

impl EventWindow {
    pub fn new() -> Self {
        EventWindow::default()
    }

    /// The stamp a record caused by `cause` carries: the open batch, if there
    /// is one, and the cause its emitter names.
    pub(crate) fn stamp_caused_by(&self, cause: EventCause) -> EventStamp {
        EventStamp { batch: self.current_batch(), cause }
    }

    /// Record `event` under `stamp`.
    pub(crate) fn push(&mut self, event: GameEvent, stamp: EventStamp, names: NamesAsAnnounced) {
        self.records.push(EventRecord {
            seq: EventId(self.next_seq),
            event,
            stamp,
            names,
            places: RecordPlaces::NOT_COUNTED,
        });
        self.next_seq += 1;
    }

    /// Mark record `seq` counted by the history, with its places.
    pub(crate) fn note_counted(&mut self, seq: EventId, in_turn: Option<u64>, in_draw_step: Option<u64>) {
        let Some(first) = self.records.first().map(|r| r.seq) else { return };
        if let Some(record) = seq.0.checked_sub(first.0).and_then(|i| self.records.get_mut(i)) {
            record.places = RecordPlaces::counted(in_turn, in_draw_step);
        }
    }

    /// The record at `seq`, or `None` for one the window no longer holds.
    pub fn record(&self, seq: EventId) -> Option<&EventRecord> {
        let first = self.records.first()?.seq;
        self.records.get(seq.0.checked_sub(first.0)?)
    }

    /// The records from `mark` on — pass an earlier [`Self::next_seq`] to ask
    /// "what has been performed since". A dispatch's mark is taken inside the
    /// window it reads, so nothing it asks for has been flushed.
    pub fn records_since(&self, mark: EventId) -> &[EventRecord] {
        let first = self.records.first().map_or(self.next_seq, |r| r.seq.0);
        debug_assert!(mark.0 >= first, "a mark from before the window's last flush");
        &self.records[mark.0.saturating_sub(first).min(self.records.len())..]
    }

    /// Every record the window holds, oldest first.
    pub fn held(&self) -> &[EventRecord] {
        &self.records
    }

    /// The records resolution `stamp` performed from `mark` on, oldest first:
    /// what a later instruction of the same resolution reads back. The
    /// window holds a resolution's records until it ends
    /// (`GameState::flush_window_unless_nested`), and a nested dispatch's
    /// records carry no stamp or another resolution's, so they are not
    /// among them.
    pub fn resolution_records(&self, stamp: ResolutionStamp, mark: EventId) -> impl Iterator<Item = &EventRecord> {
        self.records_since(mark).iter().filter(move |r| r.resolution() == Some(stamp))
    }

    /// The sequence number the next emitted record will carry.
    pub fn next_seq(&self) -> EventId {
        EventId(self.next_seq)
    }

    /// The batch the next emitted event joins — the trace sink's join key
    /// between a batch record and the events performed inside it.
    pub fn current_batch(&self) -> Option<BatchId> {
        self.open
    }

    /// Open a batch, returning what to hand back to [`Self::close_batch`].
    ///
    /// **A nested call joins the enclosing batch rather than opening a new
    /// one.** CR 120.3 and 120.4 are why it has to: a damage event's results
    /// (the life loss, the counters, lifelink's gain) are processed as part of
    /// that one event (CR 120.4c) before it occurs (CR 120.4d). They are
    /// proposed from inside the damage's batch, so a second batch id would
    /// split one event into two.
    pub(crate) fn open_batch(&mut self) -> Option<BatchId> {
        let previous = self.open;
        let id = previous.unwrap_or_else(|| {
            let id = BatchId(self.next_batch);
            self.next_batch += 1;
            id
        });
        self.open = Some(id);
        previous
    }

    /// Open a batch that does **not** join an enclosing one.
    ///
    /// Two callers, each with the rules argument for why its events are not a
    /// result of the event they are nested inside:
    /// `GameState::decide_just_before_entry` for CR 614.13's other objects that
    /// change zones, right before the entry, and `GameState::execute_riders`
    /// for a replacement's rider, which CR 615.5 puts "immediately afterward".
    pub(crate) fn open_new_batch(&mut self) -> Option<BatchId> {
        let previous = self.open;
        let id = BatchId(self.next_batch);
        self.next_batch += 1;
        self.open = Some(id);
        previous
    }

    /// Open `id` again, a batch [`Self::open_new_batch`] numbered earlier: the
    /// events an entry's replacements decide on in phase 1 are performed under
    /// it in phase 2.
    pub(crate) fn reopen_batch(&mut self, id: BatchId) -> Option<BatchId> {
        self.open.replace(id)
    }

    /// Close a batch, restoring what [`Self::open_batch`] returned.
    pub(crate) fn close_batch(&mut self, previous: Option<BatchId>) {
        self.open = previous;
    }

    /// Hand every held record to the recorder, or drop it when none is
    /// attached. The numbering carries on.
    pub(crate) fn flush(&mut self) {
        match &self.recorder {
            Some(recorder) => recorder.extend(self.records.drain(..)),
            None => self.records.clear(),
        }
    }

    pub(crate) fn attach_recorder(&mut self, recorder: RecorderHandle) {
        self.recorder = Some(recorder);
    }

    pub(crate) fn recorder(&self) -> Option<&RecorderHandle> {
        self.recorder.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_carries_its_place_in_the_stream() {
        let mut window = EventWindow::new();
        assert!(window.held().is_empty());

        window.push(GameEvent::TurnBegin { player: 0, turn_number: 1 }, window.stamp_caused_by(EventCause::Harness), None);
        window.push(GameEvent::PhaseBegin { phase: PhaseType::Beginning, player: 0 }, window.stamp_caused_by(EventCause::Harness), None);

        assert_eq!(window.held().len(), 2);
        assert_eq!(window.held()[1].seq, EventId(1));
        assert_eq!(window.next_seq(), EventId(2));
    }

    /// CR 603.7a: a record of the batch something was created in, on either
    /// side of it, came before it; a record after that batch did not.
    #[test]
    fn what_is_created_in_a_batch_is_after_every_record_of_it() {
        let mut window = EventWindow::new();
        let outer = window.open_batch();
        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(EventCause::Harness), None);
        let created_at = CreatedAt::now(&window);
        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(EventCause::Harness), None);
        window.close_batch(outer);
        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(EventCause::Harness), None);

        let after: Vec<bool> = window.held().iter().map(|record| created_at.is_before(record)).collect();
        assert_eq!(after, vec![false, false, true]);
    }

    #[test]
    fn records_since_a_mark_are_the_ones_after_it() {
        let mut window = EventWindow::new();
        window.push(GameEvent::TurnBegin { player: 0, turn_number: 1 }, window.stamp_caused_by(EventCause::Harness), None);
        window.push(GameEvent::PhaseBegin { phase: PhaseType::Beginning, player: 0 }, window.stamp_caused_by(EventCause::Harness), None);
        window.push(GameEvent::StepBegin { step: StepType::Untap, player: 0 }, window.stamp_caused_by(EventCause::Harness), None);

        assert_eq!(window.records_since(EventId(1)).len(), 2);
        assert_eq!(window.records_since(EventId(3)).len(), 0);
    }

    /// A flush empties the window and the numbering carries on, so a record
    /// is found by its number and never by where it sits.
    #[test]
    fn a_flushed_window_keeps_numbering_where_it_left_off() {
        let mut window = EventWindow::new();
        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(EventCause::Harness), None);
        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(EventCause::Harness), None);
        window.flush();
        assert!(window.held().is_empty());
        assert_eq!(window.record(EventId(1)), None, "a flushed record is gone");

        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(EventCause::Harness), None);
        assert_eq!(window.held()[0].seq, EventId(2));
        assert_eq!(window.record(EventId(2)).map(|r| r.seq), Some(EventId(2)));
        assert_eq!(window.records_since(EventId(2)).len(), 1);
    }

    #[test]
    fn test_events_outside_a_batch_carry_no_batch_id() {
        let mut window = EventWindow::new();
        window.push(GameEvent::TurnBegin { player: 0, turn_number: 1 }, window.stamp_caused_by(EventCause::Harness), None);
        assert_eq!(window.held()[0].batch(), None);
    }

    #[test]
    fn test_a_batch_stamps_every_event_it_emits() {
        let mut window = EventWindow::new();
        let outer = window.open_batch();
        let object = ObjectRef { id: ObjectId::UNASSIGNED, zone_change_epoch: 0 };
        window.push(GameEvent::Tapped { object }, window.stamp_caused_by(EventCause::Harness), None);
        window.push(GameEvent::Untapped { object }, window.stamp_caused_by(EventCause::Harness), None);
        window.close_batch(outer);
        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(EventCause::Harness), None);

        let b = window.held()[0].batch().expect("inside a batch");
        assert_eq!(window.held()[1].batch(), Some(b), "one batch, one id");
        assert_eq!(window.held()[2].batch(), None, "closing restores the outer context");
    }

    #[test]
    fn test_a_nested_batch_joins_the_enclosing_one() {
        // CR 120.3f makes lifelink's gain a result of the damage, and CR 120.4d
        // lets the one damage event occur after its results are processed. The
        // gain is proposed from inside the damage's performance, so it must not
        // open a batch of its own.
        let mut window = EventWindow::new();
        let outer = window.open_batch();
        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(EventCause::Harness), None);
        let inner = window.open_batch();
        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(EventCause::Harness), None);
        window.close_batch(inner);
        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(EventCause::Harness), None);
        window.close_batch(outer);

        let b = window.held()[0].batch().expect("inside a batch");
        assert!(window.held().iter().all(|r| r.batch() == Some(b)),
                "a nested batch joins the enclosing one rather than opening its own");
    }

    #[test]
    fn test_separate_batches_get_separate_ids() {
        let mut window = EventWindow::new();
        let prev = window.open_batch();
        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(EventCause::Harness), None);
        window.close_batch(prev);
        let prev = window.open_batch();
        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(EventCause::Harness), None);
        window.close_batch(prev);

        assert_ne!(window.held()[0].batch(), window.held()[1].batch());
    }

    /// Two records of one batch keep the causes their emitters named: CR
    /// 704.3's one event, whose members are each their own rule's action.
    /// Outside every batch a record joins none and keeps its cause the same way.
    #[test]
    fn a_record_keeps_its_emitters_cause_inside_a_batch_and_out() {
        use crate::events::cause::{GameRule, StateBasedAction};
        let mut window = EventWindow::new();
        let (legend, zero) = (StateBasedAction::LegendRule, StateBasedAction::ZeroToughness);
        let outer = window.open_batch();
        for rule in [legend, zero] {
            let stamp = window.stamp_caused_by(EventCause::StateBasedAction(rule));
            window.push(GameEvent::StateBasedActionPerformed, stamp, None);
        }
        window.close_batch(outer);
        let check = EventCause::GameRule(GameRule::StateBasedCheck);
        window.push(GameEvent::StateBasedActionPerformed, window.stamp_caused_by(check), None);

        let [first, second, after] = window.held() else { panic!("three records") };
        assert_eq!(first.batch(), second.batch(), "one batch");
        assert!(first.batch().is_some());
        assert_eq!(first.stamp.cause, EventCause::StateBasedAction(legend));
        assert_eq!(second.stamp.cause, EventCause::StateBasedAction(zero));
        assert_eq!(after.stamp, EventStamp { batch: None, cause: check });
    }
}
