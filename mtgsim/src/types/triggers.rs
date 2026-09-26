//! Triggered abilities as data — CR 603's type surface
//! (`triggers-architecture.md` §3).
//!
//! A triggered ability is an [`AbilityDef`](crate::objects::card_data::AbilityDef)
//! whose effect is `Effect::Triggered(Arc<TriggerDef>)`, the shape
//! `Effect::Replacement` gave static replacement abilities: a card is data and
//! adding one touches `src/cards/*.rs` alone. **The one growth axis is
//! [`TriggerEvent`]**, one arm per performed `GameEvent` kind and no other —
//! a card needing a predicate no arm expresses is a missing field on a
//! record, never a new axis (§3.3's contract, inherited from
//! `replacement-architecture.md` §3.2a). The per-arm projections at the
//! bottom of this file are exhaustive matches with no wildcard, so a new arm
//! cannot compile until it says what its "that object", "that player", "that
//! many" and "one occurrence" are.

use std::sync::Arc;

use crate::engine::layers::types::EffectiveCharacteristics;
use crate::events::event::{DamageTarget, EventRecord, GameEvent};
use crate::objects::card_data::CardData;
use crate::state::game_state::{AbilityIdentity, PhaseType, StepType};
use crate::types::effects::{Condition, Effect, EffectRecipient, ObjectFilter, PlayerRef};
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::mana::ManaType;
use crate::types::zones::{Zone, ZoneChangeCause};

/// CR 603.1 — "[When/Whenever/At] [trigger condition or event], [effect]".
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerDef {
    pub condition: TriggerCondition,
    /// CR 603.4 — checked as the event happens and again as the ability
    /// resolves (608.2a). `None` is an ability with no "if" clause; an `if`
    /// anywhere else in the text is ordinary `Effect::Conditional`.
    pub intervening_if: Option<Condition>,
    /// The two once-per-turn gates and "for the first time each turn" (§3.5).
    /// Declared here; the gates and their writers are TR-2's.
    pub limit: Option<TriggerLimit>,
    /// Its targets are `Effect::instances` of this tree, announced at
    /// placement (603.3d); its bound facts read [`TriggerBinding`].
    pub effect: Effect,
}

/// CR 603.2's "game event or game state".
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerCondition {
    /// One performed record matches (§3.3).
    Event(TriggerEvent),
    /// CR 603.8 — no event; a predicate over live state. Declared for the
    /// shape; the state check that reads it is TR-6's, and the dispatcher
    /// refuses it until then.
    State(Condition),
    /// "Whenever [A] or [B]" — each arm matched on its own; two arms matching
    /// one record trigger once.
    AnyOf(Vec<TriggerEvent>),
}

impl TriggerDef {
    /// The kinds of record this def can match: every arm's, together.
    ///
    /// **What it is for:** `register_static_effects` files a permanent in
    /// `trigger_sources` under the union of this over its printed triggers,
    /// and the dispatcher skips the permanent for any window whose records
    /// are none of these kinds (§11). A state trigger reads no record (CR
    /// 603.8), so its set is empty.
    pub fn record_kinds(&self) -> EventKindMask {
        self.condition
            .events()
            .iter()
            .fold(EventKindMask::EMPTY, |mask, arm| mask | arm.record_kinds())
    }
}

impl TriggerCondition {
    /// The condition's events, in printed order. A state trigger has none.
    pub fn events(&self) -> &[TriggerEvent] {
        match self {
            TriggerCondition::Event(event) => std::slice::from_ref(event),
            TriggerCondition::AnyOf(events) => events,
            TriggerCondition::State(_) => &[],
        }
    }

    /// CR 603.3b's tier: second iff the condition is another ability
    /// triggering.
    pub fn tier(&self) -> TriggerTier {
        if self.events().iter().any(|e| matches!(e, TriggerEvent::AbilityTriggers { .. })) {
            TriggerTier::Second
        } else {
            TriggerTier::First
        }
    }
}

/// CR 603.3b's two-part placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriggerTier {
    /// A trigger condition that isn't another ability triggering.
    First,
    /// The remaining triggered abilities.
    Second,
}

/// The once-per-turn gates (`triggers-architecture.md` §3.5). Two trackers
/// with two writers, both TR-2's; the def carries the shape now so a card
/// file can say which it prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerLimit {
    /// CR 603.2h — "Do this only once each turn": an action-taken gate,
    /// written by the resolution that takes the action.
    DoThisOnlyOnceEachTurn,
    /// "This ability triggers only once each turn" — a triggered gate,
    /// written by the dispatcher as it queues (Elvish Warmaster's ruling).
    TriggersOnlyOnceEachTurn,
    /// "Whenever [event] for the first time each turn" — a predicate on the
    /// event, read record by record off the turn summary.
    FirstTimeEachTurn,
}

/// CR 603.2c — one trigger per matching record, or one per window in which
/// any record matches ("one or more").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Multiplicity {
    PerOccurrence,
    OncePerEvent,
}

/// "Which object" an arm is about, read against the ability's own source.
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerSubject {
    /// CR 603.6a's "[this object]": the record's subject is the source itself.
    ThisObject,
    /// "Enchanted land", "equipped creature": the source's host (CR 303.4m).
    Host,
    /// "A creature", "another creature you control". "Another" is
    /// `And(filter, NotSource)`; `NotSource` excludes the ability's own source.
    Filter(ObjectFilter),
    Any,
}

/// Whom damage was dealt to, for "is dealt damage" and "deals damage to".
#[derive(Debug, Clone, PartialEq)]
pub enum DamageRecipient {
    Any,
    /// A player; `None` is any player, `Some(You)` the ability's controller.
    Player(Option<PlayerRef>),
    /// A permanent; `None` is any permanent.
    Object(Option<ObjectFilter>),
}

/// The kind of record a trigger arm reads - one variant per `GameEvent`
/// variant any [`TriggerEvent`] can match, and none for the records no arm
/// can (`Scried`, `TokenCreated`, ...).
///
/// **The one table the matcher's discriminant test is written from.**
/// [`TriggerEvent::reads`] and the dispatcher's source mask
/// (`triggers-architecture.md` §11) both ask [`TriggerEvent::record_kinds`]
/// and [`EventKind::from_record`], so they cannot disagree about which arm
/// reads which record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    ZoneChange,
    LeftTheGame,
    Tapped,
    Untapped,
    ManaAdded,
    DamageDealt,
    PhaseBegin,
    StepBegin,
    TurnBegin,
    LifeChanged,
    EnteredBattlefield,
    AttackersDeclared,
    AbilityTriggered,
    SpellCast,
    CardDrawn,
    LibraryShuffled,
}

impl EventKind {
    /// How many kinds there are: the last variant's index, plus one. A kind
    /// added after it moves this, and `bit` asserts it did.
    pub const COUNT: usize = EventKind::LibraryShuffled as usize + 1;

    /// How many words an [`EventKindMask`] needs to hold every kind. One while
    /// the kinds fit in 64 (`triggers-architecture.md` §12, TR-2b's decision
    /// 5), and a new kind past that grows the mask with no edit to it.
    pub const WORDS: usize = EventKind::COUNT.div_ceil(64);

    /// This record's kind, or `None` for a record no trigger arm reads.
    /// Exhaustive, so a new `GameEvent` variant does not compile until it
    /// says whether an arm reads it — a wildcard would file it as unread, and
    /// the mask would drop it without a word.
    pub fn from_record(record: &GameEvent) -> Option<EventKind> {
        Some(match record {
            GameEvent::ZoneChange { .. } => EventKind::ZoneChange,
            GameEvent::LeftTheGame { .. } => EventKind::LeftTheGame,
            GameEvent::Tapped { .. } => EventKind::Tapped,
            GameEvent::Untapped { .. } => EventKind::Untapped,
            GameEvent::ManaAdded { .. } => EventKind::ManaAdded,
            GameEvent::DamageDealt { .. } => EventKind::DamageDealt,
            GameEvent::PhaseBegin { .. } => EventKind::PhaseBegin,
            GameEvent::StepBegin { .. } => EventKind::StepBegin,
            GameEvent::TurnBegin { .. } => EventKind::TurnBegin,
            GameEvent::LifeChanged { .. } => EventKind::LifeChanged,
            GameEvent::PermanentEnteredBattlefield { .. } => EventKind::EnteredBattlefield,
            GameEvent::AttackersDeclared { .. } => EventKind::AttackersDeclared,
            GameEvent::AbilityTriggered { .. } => EventKind::AbilityTriggered,
            GameEvent::SpellCast { .. } => EventKind::SpellCast,
            GameEvent::CardDrawn { .. } => EventKind::CardDrawn,
            GameEvent::LibraryShuffled { .. } => EventKind::LibraryShuffled,
            GameEvent::AbilityActivated { .. }
            | GameEvent::AbilityCountered { .. }
            | GameEvent::AbilityResolved { .. }
            | GameEvent::Attached { .. }
            | GameEvent::BlockersDeclared { .. }
            | GameEvent::CountersAnnihilated { .. }
            | GameEvent::CountersChanged { .. }
            | GameEvent::EquipmentDetached { .. }
            | GameEvent::PlayerLost { .. }
            | GameEvent::PlayerWon { .. }
            | GameEvent::Scried { .. }
            | GameEvent::SpellCountered { .. }
            | GameEvent::SpellFizzled { .. }
            | GameEvent::StateBasedActionPerformed
            | GameEvent::TokenCeasedToExist { .. }
            | GameEvent::TokenCreated { .. } => return None,
        })
    }

    /// This kind's word in a mask, and its bit within that word.
    const fn place(self) -> (usize, u64) {
        let index = self as usize;
        assert!(index < EventKind::COUNT, "a kind past EventKind::COUNT: move the constant");
        (index / 64, 1 << (index % 64))
    }
}

/// A set of [`EventKind`]s - §11's lever, pre-approved there and built when
/// the reading asked for it. What a source is filed under in
/// `GameState::trigger_sources`, and what a window's records OR into.
///
/// A hand-rolled bitmask following [`crate::types::zones::ZoneSet`], for the
/// same reason it is one, and as many words wide as [`EventKind::WORDS`]
/// says: while the kinds fit in 64 it is one word, the same instructions as a
/// `u64`, and a kind past that widens it without an edit here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventKindMask([u64; EventKind::WORDS]);

impl Default for EventKindMask {
    fn default() -> Self {
        EventKindMask::EMPTY
    }
}

impl EventKindMask {
    pub const EMPTY: EventKindMask = EventKindMask([0; EventKind::WORDS]);
    /// Every kind: as a filter, it keeps every source that reads anything.
    pub const ALL: EventKindMask = EventKindMask([u64::MAX; EventKind::WORDS]);

    pub const fn of(kind: EventKind) -> EventKindMask {
        EventKindMask::EMPTY.with(kind)
    }

    pub const fn with(self, kind: EventKind) -> EventKindMask {
        let (word, bit) = kind.place();
        let mut words = self.0;
        words[word] |= bit;
        EventKindMask(words)
    }

    pub fn contains(self, kind: EventKind) -> bool {
        let (word, bit) = kind.place();
        self.0[word] & bit != 0
    }

    /// Whether the two sets share a kind - the dispatcher's whole question of
    /// a source: could anything it printed read anything this window carries?
    pub fn intersects(self, other: EventKindMask) -> bool {
        self.0.iter().zip(other.0.iter()).any(|(a, b)| a & b != 0)
    }

    pub fn is_empty(self) -> bool {
        self.0.iter().all(|&word| word == 0)
    }
}

impl std::ops::BitOr for EventKindMask {
    type Output = EventKindMask;
    fn bitor(mut self, other: EventKindMask) -> EventKindMask {
        self |= other;
        self
    }
}

impl std::ops::BitOrAssign for EventKindMask {
    fn bitor_assign(&mut self, other: EventKindMask) {
        for (word, theirs) in self.0.iter_mut().zip(other.0) {
            *word |= theirs;
        }
    }
}

/// One arm per performed event kind, in `GameEvent`'s declaration order
/// (`triggers-architecture.md` §3.3). `None` on a field means the arm does
/// not ask. TR-1 ships the arms its cards and fixtures read; the rest land
/// with their phase and are listed in §3.3's table.
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerEvent {
    /// Dies is `from: Battlefield, to: Graveyard`; leaves the battlefield is
    /// `from: Battlefield, to: None`; "from anywhere" is `from: None`, which
    /// CR 603.6c makes *not* a leaves-the-battlefield ability.
    ZoneChange {
        subject: TriggerSubject,
        from: Option<Zone>,
        to: Option<Zone>,
        cause: Option<ZoneChangeCause>,
        owner: Option<PlayerRef>,
        multiplicity: Multiplicity,
    },
    /// Transition-only by the record's own contract (CR 603.2e).
    BecomesTapped { subject: TriggerSubject },
    BecomesUntapped { subject: TriggerSubject },
    /// "Whenever [you/a player] draw[s] a card" (CR 121.1): one occurrence per
    /// card, so a draw of three triggers three times (Psychosis Crawler's
    /// ruling). "That card" is the one drawn. A card put into a hand without
    /// the word "draw" is not drawn (CR 121.5); it is a `ZoneChange`, and never
    /// this.
    DrawsCard { player: Option<PlayerRef>, multiplicity: Multiplicity },
    /// CR 106.12a's "tapped for mana" reads `tapped_for_mana`.
    ManaAdded {
        source: TriggerSubject,
        tapped_for_mana: Option<bool>,
        mana: Option<ManaType>,
    },
    /// "Is dealt damage", "deals damage", "deals combat damage to a player".
    DamageDealt {
        source: TriggerSubject,
        recipient: DamageRecipient,
        combat: Option<bool>,
        multiplicity: Multiplicity,
    },
    /// "At the beginning of [your/each] [phase]". `whose: None` is each.
    PhaseBegins { phase: PhaseType, whose: Option<PlayerRef> },
    /// "At the beginning of [your/each] upkeep"; "at end of combat" is
    /// `EndCombat` (CR 511.2) and "at the beginning of the end step" `End`.
    StepBegins { step: StepType, whose: Option<PlayerRef> },
    TurnBegins { whose: Option<PlayerRef> },
    /// "Whenever [you/a player] gain[s] life" — CR 119.9 makes each source's
    /// gain its own event, and the record is per source; a 0 gain is no event
    /// (119.10) and never reaches the log. One of two arms over `LifeChanged`,
    /// split by the sign; the loss is [`Self::LosesLife`]'s.
    GainsLife { player: Option<PlayerRef>, multiplicity: Multiplicity },
    /// "Whenever [you/a player] lose[s] life": the loss half of the sign
    /// split, one occurrence per record (§14 question 3). Damage, a payment
    /// and an effect all lose life (Vengeful Warchief's first ruling).
    LosesLife { player: Option<PlayerRef>, multiplicity: Multiplicity },
    /// CR 603.6a. `from` and `cast` are joined from the same object's zone
    /// change in the window and from `PermanentState.cast` (§4.4).
    EntersBattlefield {
        subject: TriggerSubject,
        controller: Option<PlayerRef>,
        from: Option<Zone>,
        was_cast: Option<bool>,
        multiplicity: Multiplicity,
    },
    /// CR 508.3a's plain shape, "whenever a creature attacks" — one attacker
    /// is one occurrence. TR-5 widens this to the five shapes with item 11's
    /// defender on the record.
    Attacks { attacker: TriggerSubject, multiplicity: Multiplicity },
    /// "Whenever [you/a player] cast[s] a [filter] spell" (CR 601.2i). The
    /// spell is on the stack as its record is dispatched, so the filter reads
    /// it through the layer walk.
    CastsSpell { caster: Option<PlayerRef>, spell: Option<ObjectFilter> },
    /// "Whenever [you/an opponent] shuffle[s] [your/their] library" (CR
    /// 701.24): one record per shuffle, a library of zero or one cards
    /// included (CR 701.24e). Cascade's random bottom is no shuffle, and emits
    /// no record (Cosi's Trickster's rulings).
    ShufflesLibrary { player: Option<PlayerRef> },
    /// CR 603.3b's second tier, by construction: the event the dispatcher
    /// emits per queued trigger (§4.8).
    AbilityTriggers { caused_by: Option<Box<TriggerEvent>>, source: Option<ObjectFilter> },
}

impl TriggerEvent {
    /// CR 603.10's list, as a property of the arm's fields and never a flag a
    /// card sets (§4.3): leaves-the-battlefield and dies (603.10a's first
    /// class). The other classes arrive with their arms.
    pub fn looks_back(&self) -> bool {
        match self {
            TriggerEvent::ZoneChange { from, .. } => *from == Some(Zone::Battlefield),
            TriggerEvent::BecomesTapped { .. }
            | TriggerEvent::BecomesUntapped { .. }
            | TriggerEvent::DrawsCard { .. }
            | TriggerEvent::ManaAdded { .. }
            | TriggerEvent::DamageDealt { .. }
            | TriggerEvent::PhaseBegins { .. }
            | TriggerEvent::StepBegins { .. }
            | TriggerEvent::TurnBegins { .. }
            | TriggerEvent::GainsLife { .. }
            | TriggerEvent::LosesLife { .. }
            | TriggerEvent::EntersBattlefield { .. }
            | TriggerEvent::Attacks { .. }
            | TriggerEvent::CastsSpell { .. }
            | TriggerEvent::ShufflesLibrary { .. }
            | TriggerEvent::AbilityTriggers { .. } => false,
        }
    }

    /// The kinds of record this arm can match — the discriminant half of
    /// the matcher. [`Self::reads`] asks it of one record, and the
    /// dispatcher's source mask is its union over a permanent's printed
    /// arms: one table, so the two cannot disagree.
    ///
    /// CR 603.6c is why `ZoneChange` names two: a leaves-the-battlefield
    /// ability reads the departure a player leaving the game causes, and
    /// that record is `LeftTheGame` rather than a zone change (§4.2 leg 2).
    pub fn record_kinds(&self) -> EventKindMask {
        match self {
            TriggerEvent::ZoneChange { .. } => {
                EventKindMask::of(EventKind::ZoneChange).with(EventKind::LeftTheGame)
            }
            TriggerEvent::BecomesTapped { .. } => EventKindMask::of(EventKind::Tapped),
            TriggerEvent::BecomesUntapped { .. } => EventKindMask::of(EventKind::Untapped),
            TriggerEvent::DrawsCard { .. } => EventKindMask::of(EventKind::CardDrawn),
            TriggerEvent::ManaAdded { .. } => EventKindMask::of(EventKind::ManaAdded),
            TriggerEvent::DamageDealt { .. } => EventKindMask::of(EventKind::DamageDealt),
            TriggerEvent::PhaseBegins { .. } => EventKindMask::of(EventKind::PhaseBegin),
            TriggerEvent::StepBegins { .. } => EventKindMask::of(EventKind::StepBegin),
            TriggerEvent::TurnBegins { .. } => EventKindMask::of(EventKind::TurnBegin),
            TriggerEvent::GainsLife { .. } | TriggerEvent::LosesLife { .. } => {
                EventKindMask::of(EventKind::LifeChanged)
            }
            TriggerEvent::EntersBattlefield { .. } => {
                EventKindMask::of(EventKind::EnteredBattlefield)
            }
            TriggerEvent::Attacks { .. } => EventKindMask::of(EventKind::AttackersDeclared),
            TriggerEvent::CastsSpell { .. } => EventKindMask::of(EventKind::SpellCast),
            TriggerEvent::ShufflesLibrary { .. } => EventKindMask::of(EventKind::LibraryShuffled),
            TriggerEvent::AbilityTriggers { .. } => EventKindMask::of(EventKind::AbilityTriggered),
        }
    }

    /// Whether `record` is the kind of event this arm reads at all — the
    /// discriminant test the predicates follow.
    pub fn reads(&self, record: &GameEvent) -> bool {
        EventKind::from_record(record).is_some_and(|kind| self.record_kinds().contains(kind))
    }

    /// The arm's multiplicity field, for the arms that carry one.
    pub fn multiplicity(&self) -> Multiplicity {
        match self {
            TriggerEvent::ZoneChange { multiplicity, .. }
            | TriggerEvent::DrawsCard { multiplicity, .. }
            | TriggerEvent::DamageDealt { multiplicity, .. }
            | TriggerEvent::GainsLife { multiplicity, .. }
            | TriggerEvent::LosesLife { multiplicity, .. }
            | TriggerEvent::EntersBattlefield { multiplicity, .. }
            | TriggerEvent::Attacks { multiplicity, .. } => *multiplicity,
            TriggerEvent::BecomesTapped { .. }
            | TriggerEvent::BecomesUntapped { .. }
            | TriggerEvent::ManaAdded { .. }
            | TriggerEvent::PhaseBegins { .. }
            | TriggerEvent::StepBegins { .. }
            | TriggerEvent::TurnBegins { .. }
            | TriggerEvent::CastsSpell { .. }
            | TriggerEvent::ShufflesLibrary { .. }
            | TriggerEvent::AbilityTriggers { .. } => Multiplicity::PerOccurrence,
        }
    }

    /// "That object" — the record's subject (§3.4). `None` for an event about
    /// no object.
    pub fn subject_of(&self, record: &GameEvent) -> Option<ObjectId> {
        match (self, record) {
            (TriggerEvent::ZoneChange { .. }, GameEvent::ZoneChange { object_id, .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::LeftTheGame { object_id, .. })
            | (TriggerEvent::BecomesTapped { .. }, GameEvent::Tapped { object_id })
            | (TriggerEvent::BecomesUntapped { .. }, GameEvent::Untapped { object_id })
            | (TriggerEvent::EntersBattlefield { .. }, GameEvent::PermanentEnteredBattlefield { object_id, .. }) => {
                Some(*object_id)
            }
            (TriggerEvent::ManaAdded { .. }, GameEvent::ManaAdded { source_id, .. }) => Some(*source_id),
            (TriggerEvent::DrawsCard { .. }, GameEvent::CardDrawn { card_id, .. }) => Some(*card_id),
            // "That creature" on a damage trigger is the permanent dealt damage,
            // the way Fungusaur reads it; the source is `player_of`'s question
            // only through its controller.
            (TriggerEvent::DamageDealt { .. }, GameEvent::DamageDealt { target, .. }) => match target {
                DamageTarget::Object(id) => Some(*id),
                DamageTarget::Player(_) => None,
            },
            (TriggerEvent::PhaseBegins { .. }, GameEvent::PhaseBegin { .. })
            | (TriggerEvent::StepBegins { .. }, GameEvent::StepBegin { .. })
            | (TriggerEvent::TurnBegins { .. }, GameEvent::TurnBegin { .. })
            | (TriggerEvent::GainsLife { .. }, GameEvent::LifeChanged { .. })
            | (TriggerEvent::LosesLife { .. }, GameEvent::LifeChanged { .. })
            | (TriggerEvent::Attacks { .. }, GameEvent::AttackersDeclared { .. })
            | (TriggerEvent::ShufflesLibrary { .. }, GameEvent::LibraryShuffled { .. })
            | (TriggerEvent::AbilityTriggers { .. }, GameEvent::AbilityTriggered { .. }) => None,
            // "That spell".
            (TriggerEvent::CastsSpell { .. }, GameEvent::SpellCast { spell_id, .. }) => Some(*spell_id),
            _ => None,
        }
    }

    /// "That player" (§3.4). `None` for an event about no player.
    pub fn player_of(&self, record: &GameEvent) -> Option<PlayerId> {
        match (self, record) {
            (TriggerEvent::ZoneChange { .. }, GameEvent::ZoneChange { owner, .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::LeftTheGame { owner, .. }) => Some(*owner),
            (TriggerEvent::ManaAdded { .. }, GameEvent::ManaAdded { player_id, .. }) => Some(*player_id),
            (TriggerEvent::DamageDealt { .. }, GameEvent::DamageDealt { target, .. }) => match target {
                DamageTarget::Player(pid) => Some(*pid),
                DamageTarget::Object(_) => None,
            },
            (TriggerEvent::PhaseBegins { .. }, GameEvent::PhaseBegin { player, .. })
            | (TriggerEvent::StepBegins { .. }, GameEvent::StepBegin { player, .. })
            | (TriggerEvent::TurnBegins { .. }, GameEvent::TurnBegin { player, .. }) => Some(*player),
            (TriggerEvent::GainsLife { .. }, GameEvent::LifeChanged { player_id, .. })
            | (TriggerEvent::LosesLife { .. }, GameEvent::LifeChanged { player_id, .. }) => Some(*player_id),
            (TriggerEvent::CastsSpell { .. }, GameEvent::SpellCast { caster, .. }) => Some(*caster),
            (TriggerEvent::DrawsCard { .. }, GameEvent::CardDrawn { player_id, .. })
            | (TriggerEvent::ShufflesLibrary { .. }, GameEvent::LibraryShuffled { player_id }) => Some(*player_id),
            (TriggerEvent::EntersBattlefield { .. }, GameEvent::PermanentEnteredBattlefield { controller, .. }) => {
                Some(*controller)
            }
            (TriggerEvent::AbilityTriggers { .. }, GameEvent::AbilityTriggered { controller, .. }) => {
                Some(*controller)
            }
            (TriggerEvent::BecomesTapped { .. }, GameEvent::Tapped { .. })
            | (TriggerEvent::BecomesUntapped { .. }, GameEvent::Untapped { .. })
            | (TriggerEvent::Attacks { .. }, GameEvent::AttackersDeclared { .. }) => None,
            _ => None,
        }
    }

    /// "That many" (§3.4): the damage dealt, the mana added. `None` for an
    /// event with no quantity.
    pub fn amount_of(&self, record: &GameEvent) -> Option<u64> {
        match (self, record) {
            (TriggerEvent::DamageDealt { .. }, GameEvent::DamageDealt { amount, .. }) => Some(*amount),
            (TriggerEvent::ManaAdded { .. }, GameEvent::ManaAdded { mana, .. }) => {
                Some(mana.iter().map(|(_, n)| *n).sum())
            }
            (TriggerEvent::GainsLife { .. }, GameEvent::LifeChanged { old, new, .. }) => {
                Some((new - old).max(0) as u64)
            }
            (TriggerEvent::LosesLife { .. }, GameEvent::LifeChanged { old, new, .. }) => {
                Some((old - new).max(0) as u64)
            }
            // One card a record, so "that many" over a "one or more" binding is
            // the cards drawn.
            (TriggerEvent::DrawsCard { .. }, GameEvent::CardDrawn { .. }) => Some(1),
            (TriggerEvent::ZoneChange { .. }, GameEvent::ZoneChange { .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::LeftTheGame { .. })
            | (TriggerEvent::BecomesTapped { .. }, GameEvent::Tapped { .. })
            | (TriggerEvent::BecomesUntapped { .. }, GameEvent::Untapped { .. })
            | (TriggerEvent::PhaseBegins { .. }, GameEvent::PhaseBegin { .. })
            | (TriggerEvent::StepBegins { .. }, GameEvent::StepBegin { .. })
            | (TriggerEvent::TurnBegins { .. }, GameEvent::TurnBegin { .. })
            | (TriggerEvent::EntersBattlefield { .. }, GameEvent::PermanentEnteredBattlefield { .. })
            | (TriggerEvent::Attacks { .. }, GameEvent::AttackersDeclared { .. })
            | (TriggerEvent::CastsSpell { .. }, GameEvent::SpellCast { .. })
            | (TriggerEvent::ShufflesLibrary { .. }, GameEvent::LibraryShuffled { .. })
            | (TriggerEvent::AbilityTriggers { .. }, GameEvent::AbilityTriggered { .. }) => None,
            _ => None,
        }
    }

    /// What one occurrence is (CR 603.2c): a record for every kind this
    /// phase ships, and an attacker for the attack shape.
    pub fn occurrences_of(&self, record: &GameEvent) -> u32 {
        match (self, record) {
            (TriggerEvent::Attacks { .. }, GameEvent::AttackersDeclared { attackers }) => attackers.len() as u32,
            (TriggerEvent::ZoneChange { .. }, GameEvent::ZoneChange { .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::LeftTheGame { .. })
            | (TriggerEvent::BecomesTapped { .. }, GameEvent::Tapped { .. })
            | (TriggerEvent::BecomesUntapped { .. }, GameEvent::Untapped { .. })
            | (TriggerEvent::DrawsCard { .. }, GameEvent::CardDrawn { .. })
            | (TriggerEvent::ManaAdded { .. }, GameEvent::ManaAdded { .. })
            | (TriggerEvent::DamageDealt { .. }, GameEvent::DamageDealt { .. })
            | (TriggerEvent::PhaseBegins { .. }, GameEvent::PhaseBegin { .. })
            | (TriggerEvent::StepBegins { .. }, GameEvent::StepBegin { .. })
            | (TriggerEvent::TurnBegins { .. }, GameEvent::TurnBegin { .. })
            | (TriggerEvent::GainsLife { .. }, GameEvent::LifeChanged { .. })
            | (TriggerEvent::LosesLife { .. }, GameEvent::LifeChanged { .. })
            | (TriggerEvent::EntersBattlefield { .. }, GameEvent::PermanentEnteredBattlefield { .. })
            | (TriggerEvent::CastsSpell { .. }, GameEvent::SpellCast { .. })
            | (TriggerEvent::ShufflesLibrary { .. }, GameEvent::LibraryShuffled { .. })
            | (TriggerEvent::AbilityTriggers { .. }, GameEvent::AbilityTriggered { .. }) => 1,
            _ => 0,
        }
    }
}

/// Which of a condition's events matched — an index into
/// [`TriggerCondition::events`], ordered as the condition lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventIndex(pub usize);

/// Monotonic per game: the order key within one player's triggers and the
/// handle a tier-2 trigger's "that ability" resolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TriggerSeq(pub u64);

/// The bound facts of a trigger, filled at dispatch and carried onto the
/// `PendingTrigger` and then the `StackEntry` (§3.4). **The matched records,
/// copied whole**: the window that held them flushes when the dispatch returns,
/// so the binding keeps its own, each fact once and its frame behind the
/// memo's `Arc`, rather than a projection that could drop one.
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerBinding {
    /// The def, shared: cloned out of the source's effective list once at
    /// dispatch, so a Humility landing between triggering and placement
    /// cannot un-trigger it (CR 113.7a).
    pub def: Arc<TriggerDef>,
    /// The records that matched: one for a `PerOccurrence` trigger, every
    /// matching record of the window for a `OncePerEvent` one.
    pub records: Vec<EventRecord>,
    /// Which event matched — whose projections say what the bound facts are.
    pub event: EventIndex,
    /// The subject's epoch at dispatch. `None` for an event about no object
    /// and for a `OncePerEvent` binding.
    pub subject: Option<ObjectRef>,
    /// The ability whose triggering this is (CR 603.3b's second tier).
    pub triggered_by: Option<TriggerSeq>,
}

impl TriggerBinding {
    /// The event the binding's projections read through.
    pub fn event(&self) -> &TriggerEvent {
        &self.def.condition.events()[self.event.0]
    }
}

/// Where a pending trigger came from (§3.8). The delayed, reflexive and
/// rule-owned origins land with TR-3 and TR-6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerOrigin {
    /// A printed, granted or copied ability of an object.
    Object(AbilityIdentity),
}

impl TriggerOrigin {
    pub fn source(&self) -> ObjectId {
        self.source_ref().id
    }

    /// The source, and which existence of it (CR 400.7).
    pub fn source_ref(&self) -> ObjectRef {
        match self {
            TriggerOrigin::Object(identity) => identity.source,
        }
    }
}

/// An object's last known information (CR 113.7a, 608.2h), kept by an entry
/// that names it — its source, or its trigger's subject — when the object
/// leaves the zone the entry expected it in (`triggers-architecture.md`
/// §6.1). The binding's records cannot hold it, since the departure is a later
/// event than the one that triggered. TR-4a makes the frame a
/// `LastKnownInformation`.
#[derive(Debug, Clone, PartialEq)]
pub struct DepartedFrame {
    pub object: ObjectRef,
    pub frame: Arc<EffectiveCharacteristics>,
}

/// An ability that has triggered and not yet been put onto the stack
/// (CR 603.2, 117.2a). On `GameState` and nowhere else — `codebase-state.md`
/// item 40's invariant, which the fork test enforces.
#[derive(Debug, Clone)]
pub struct PendingTrigger {
    pub seq: TriggerSeq,
    pub origin: TriggerOrigin,
    /// CR 603.3a — the player who controlled the source as it triggered.
    pub controller: PlayerId,
    /// What the stack object is built from — the source's card, held here
    /// because the source may be gone by placement (a dies trigger's is in a
    /// graveyard; a `LeftTheGame` source is not in the store at all). CR 603.3
    /// gives the object "no other characteristics" and nothing reads its types.
    pub source_card: Arc<CardData>,
    /// CR 601.2c's instances of "target" the def declares, in printed order —
    /// what placement announces (603.3d).
    pub instances: Vec<EffectRecipient>,
    pub binding: TriggerBinding,
    /// CR 603.8's one-shot state trigger; armed by TR-6.
    pub is_state_trigger: bool,
    /// The frames of the objects it names that have left since it triggered.
    pub departed: Vec<DepartedFrame>,
}

impl PendingTrigger {
    /// CR 603.3b's tier, derived from the def the binding carries.
    ///
    /// Not a field: `TriggerCondition::tier` is the only definition of the
    /// answer, and a copy taken at dispatch is a second one that can only
    /// ever agree or be wrong.
    pub fn tier(&self) -> TriggerTier {
        self.binding.def.condition.tier()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event::EventSeq;
    use crate::state::game_state::AbilityIdentity;
    use crate::types::ids::{AbilityId, ObjectRef};

    /// One record of every kind a trigger arm can read, named, plus one
    /// (`Scried`) that no arm reads.
    fn sample_records() -> Vec<(&'static str, GameEvent)> {
        let id = ObjectId::UNASSIGNED;
        let identity = AbilityIdentity {
            source: ObjectRef { id, zone_change_epoch: 0 },
            ability: AbilityId::printed("x", 0),
        };
        vec![
            ("ZoneChange", GameEvent::ZoneChange {
                object_id: id,
                owner: 0,
                from: Zone::Battlefield,
                to: Zone::Graveyard,
                cause: ZoneChangeCause::Destroyed,
                lki: None,
            }),
            ("LeftTheGame", GameEvent::LeftTheGame { object_id: id, owner: 0, from: Zone::Battlefield, lki: None }),
            ("Tapped", GameEvent::Tapped { object_id: id }),
            ("Untapped", GameEvent::Untapped { object_id: id }),
            ("ManaAdded", GameEvent::ManaAdded { player_id: 0, source_id: id, mana: Vec::new(), tapped_for_mana: false }),
            ("DamageDealt", GameEvent::DamageDealt {
                source_id: id,
                target: DamageTarget::Player(0),
                amount: 1,
                is_combat: false,
            }),
            ("PhaseBegin", GameEvent::PhaseBegin { phase: PhaseType::Precombat, player: 0 }),
            ("StepBegin", GameEvent::StepBegin { step: StepType::Upkeep, player: 0 }),
            ("TurnBegin", GameEvent::TurnBegin { player: 0, turn_number: 1 }),
            ("LifeChanged", GameEvent::LifeChanged { player_id: 0, old: 20, new: 21, source: None, cause: None }),
            ("PermanentEnteredBattlefield", GameEvent::PermanentEnteredBattlefield { object_id: id, controller: 0 }),
            ("AttackersDeclared", GameEvent::AttackersDeclared { attackers: vec![id] }),
            ("AbilityTriggered", GameEvent::AbilityTriggered {
                seq: TriggerSeq(0),
                origin: TriggerOrigin::Object(identity),
                controller: 0,
                caused_by: EventSeq(0),
            }),
            ("SpellCast", GameEvent::SpellCast { spell_id: id, caster: 0 }),
            ("CardDrawn", GameEvent::CardDrawn { player_id: 0, card_id: id }),
            ("LibraryShuffled", GameEvent::LibraryShuffled { player_id: 0 }),
            ("Scried", GameEvent::Scried { player_id: 0, n: 1, looked_at: 1 }),
        ]
    }

    /// Which records each arm reads, written out by hand, one line per arm.
    ///
    /// `reads` is not a table of its own: it asks the arm's `record_kinds`
    /// and the record's `EventKind::from_record`, the two halves the
    /// dispatcher's source mask is built from. This pins what those halves
    /// answer together, so a wrong entry in either fails here. The list is
    /// the one `reads` held as a 13-pair `matches!` before the mask.
    #[test]
    fn each_arm_reads_exactly_the_records_it_names() {
        let this = TriggerSubject::ThisObject;
        let each = Multiplicity::PerOccurrence;
        let expected: Vec<(TriggerEvent, &[&str])> = vec![
            (
                TriggerEvent::ZoneChange { subject: this.clone(), from: None, to: None, cause: None, owner: None, multiplicity: each },
                &["ZoneChange", "LeftTheGame"],
            ),
            (TriggerEvent::BecomesTapped { subject: this.clone() }, &["Tapped"]),
            (TriggerEvent::BecomesUntapped { subject: this.clone() }, &["Untapped"]),
            (TriggerEvent::DrawsCard { player: None, multiplicity: each }, &["CardDrawn"]),
            (TriggerEvent::ManaAdded { source: this.clone(), tapped_for_mana: None, mana: None }, &["ManaAdded"]),
            (
                TriggerEvent::DamageDealt { source: this.clone(), recipient: DamageRecipient::Any, combat: None, multiplicity: each },
                &["DamageDealt"],
            ),
            (TriggerEvent::PhaseBegins { phase: PhaseType::Precombat, whose: None }, &["PhaseBegin"]),
            (TriggerEvent::StepBegins { step: StepType::Upkeep, whose: None }, &["StepBegin"]),
            (TriggerEvent::TurnBegins { whose: None }, &["TurnBegin"]),
            (TriggerEvent::GainsLife { player: None, multiplicity: each }, &["LifeChanged"]),
            (TriggerEvent::LosesLife { player: None, multiplicity: each }, &["LifeChanged"]),
            (
                TriggerEvent::EntersBattlefield { subject: this.clone(), controller: None, from: None, was_cast: None, multiplicity: each },
                &["PermanentEnteredBattlefield"],
            ),
            (TriggerEvent::Attacks { attacker: this, multiplicity: each }, &["AttackersDeclared"]),
            (TriggerEvent::CastsSpell { caster: None, spell: None }, &["SpellCast"]),
            (TriggerEvent::ShufflesLibrary { player: None }, &["LibraryShuffled"]),
            (TriggerEvent::AbilityTriggers { caused_by: None, source: None }, &["AbilityTriggered"]),
        ];
        let records = sample_records();
        for (arm, reads) in &expected {
            for (name, record) in &records {
                assert_eq!(arm.reads(record), reads.contains(name), "{arm:?} reading a {name} record");
            }
        }
    }

    /// The mask is as wide as the kinds need and no wider: one word for the
    /// sixteen there are, every kind in it, and none of them in the empty set.
    #[test]
    fn the_mask_holds_every_kind_in_one_word() {
        assert_eq!(EventKind::WORDS, 1);
        let mut all = EventKindMask::EMPTY;
        for (_, record) in sample_records() {
            if let Some(kind) = EventKind::from_record(&record) {
                assert!(!EventKindMask::EMPTY.contains(kind));
                all |= EventKindMask::of(kind);
                assert!(all.contains(kind) && EventKindMask::ALL.contains(kind));
            }
        }
        assert!(all.intersects(EventKindMask::of(EventKind::LibraryShuffled)));
        assert!(!all.is_empty() && EventKindMask::EMPTY.is_empty());
    }

    /// A def's kinds are its arms' together — what `register_static_effects`
    /// files a source under. A state trigger reads no record (CR 603.8),
    /// whatever its condition asks.
    #[test]
    fn a_defs_record_kinds_are_its_arms_together() {
        let tapped_or_untapped = TriggerDef {
            condition: TriggerCondition::AnyOf(vec![
                TriggerEvent::BecomesTapped { subject: TriggerSubject::ThisObject },
                TriggerEvent::BecomesUntapped { subject: TriggerSubject::ThisObject },
            ]),
            intervening_if: None,
            limit: None,
            effect: Effect::Sequence(Vec::new()),
        };
        let kinds = tapped_or_untapped.record_kinds();
        assert!(kinds.contains(EventKind::Tapped));
        assert!(kinds.contains(EventKind::Untapped));
        assert!(!kinds.contains(EventKind::ZoneChange));

        let state = TriggerDef {
            condition: TriggerCondition::State(Condition::SpellWasKicked),
            ..tapped_or_untapped
        };
        assert!(state.record_kinds().is_empty());
    }
}
