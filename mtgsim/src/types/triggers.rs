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
//! bottom of this file, and the matcher's `triggering_events_in`, each end
//! in an arm naming every variant, so a new arm cannot compile until it says
//! what its "that object", "that player", "that many" and occurrences are.

use std::sync::Arc;
use crate::types::restriction::SourceFilter;

use crate::engine::layers::types::EffectiveCharacteristics;
use crate::events::cause::EventCause;
use crate::events::event::{BatchId, DamageTarget, EventRecord, EventSeq, GameEvent, ResolutionStamp};
use crate::objects::card_data::{AbilityText, CardData};
use crate::state::battlefield::{AttackTarget, CostChoices, PermanentState};
use crate::state::game_state::{AbilityIdentity, GameState, PhaseType, StackEntry, StepType};
use crate::types::effects::{Condition, CounterType, Effect, EffectRecipient, ObjectFilter, PlayerRef};
use crate::types::ids::{AbilityId, DelayedTriggerId, ExtraTurnId, ObjectId, ObjectRef, PlayerId};
use crate::types::mana::ManaType;
use crate::types::zones::{DrawFilter, Zone, ZoneChangePattern};

/// CR 603.1 — "[When/Whenever/At] [trigger condition or event], [effect]".
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerDef {
    pub condition: TriggerCondition,
    /// CR 603.4 — checked as the event happens and again as the ability
    /// resolves (608.2a). `None` is an ability with no "if" clause; an `if`
    /// anywhere else in the text is ordinary `Effect::Conditional`.
    pub intervening_if: Option<Condition>,
    /// The two once-per-turn gates and "for the first time each turn" (§3.5).
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

/// The once-per-turn gates (`triggers-architecture.md` §3.5): two trackers
/// with two writers, and the def says which one a card prints.
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
    /// "One or more ... on a creature": one trigger for each object or player
    /// the window's matching records are about, so counters put on three
    /// creatures at once trigger three times (Enduring Scalelord's second
    /// ruling) and two records on one creature once. "That many" sums that
    /// subject's records, and "that creature" is the subject. Only an arm
    /// that `TriggerEvent::supports_once_per_subject` carries it
    /// (`TriggerDef::readings_its_arms_cannot_give`).
    OncePerSubject,
}

/// What a counter arm's counters are on (CR 122.1): an object, or a player.
#[derive(Debug, Clone, PartialEq)]
pub enum CountersOn {
    /// "On this creature", "on a creature you control".
    Object(TriggerSubject),
    /// "Whenever you get one or more counters": a player, or any player for
    /// `None`.
    Player(Option<PlayerRef>),
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
    /// CR 603.7c — "when that token dies": an object a delayed triggered
    /// ability refers to ([`Referred`]), each existence's own departure and no
    /// later object's (CR 400.7).
    Referred,
}

/// What an attack trigger asks of the player, planeswalker or battle
/// attacked: CR 508.3a's "attacks [a player, planeswalker, or battle]",
/// 508.3b's "is attacked" and 508.3e's "attacks [another player]".
#[derive(Debug, Clone, PartialEq)]
pub enum DefenderFilter {
    /// A player; `None` is any player, `Some(You)` the ability's controller.
    Player(Option<PlayerRef>),
    /// A planeswalker or a battle: "attacks a planeswalker you control".
    Permanent(ObjectFilter),
    /// Either: "attacks you or a planeswalker you control" (17 cards) is a
    /// player or a permanent, which the authoring word
    /// `player_or_their_planeswalker` writes.
    Or(Box<DefenderFilter>, Box<DefenderFilter>),
}

impl DefenderFilter {
    /// Whether `target`, a player, planeswalker or battle, is one this names.
    /// The asker answers the two leaves, since who "you" is and what a filter
    /// reads are the asker's: `names_player` whether a `PlayerRef` names that
    /// player, `matches_permanent` whether a planeswalker or battle matches a
    /// filter. A trigger asks with its referents (CR 508.3a, 508.3b, 508.3e),
    /// a resolution putting a creature onto the battlefield attacking with
    /// its controller (CR 508.4).
    pub fn admits(
        &self,
        target: AttackTarget,
        names_player: &dyn Fn(&PlayerRef, PlayerId) -> bool,
        matches_permanent: &dyn Fn(&ObjectFilter, ObjectId) -> bool,
    ) -> bool {
        match (self, target) {
            (DefenderFilter::Player(who), AttackTarget::Player(player)) => {
                who.as_ref().is_none_or(|who| names_player(who, player))
            }
            (DefenderFilter::Permanent(filter), AttackTarget::Planeswalker(id) | AttackTarget::Battle(id)) => {
                matches_permanent(filter, id)
            }
            (DefenderFilter::Or(either, or), _) => {
                either.admits(target, names_player, matches_permanent) || or.admits(target, names_player, matches_permanent)
            }
            (DefenderFilter::Player(_), AttackTarget::Planeswalker(_) | AttackTarget::Battle(_))
            | (DefenderFilter::Permanent(_), AttackTarget::Player(_)) => false,
        }
    }
}

/// What one triggering event of an attack trigger is (CR 603.2c): each
/// creature declared as an attacker (CR 508.3a, 508.3c, 508.3d), or each
/// player, planeswalker or battle it is declared attacking (CR 508.3b,
/// 508.3e).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackEventPer {
    Attacker,
    Defender,
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

/// An object's event role: the part it played in the event that triggered
/// an ability, which an instruction names to say which object it reads. "That
/// creature" after "[a creature] deals damage" is the damage's source, and
/// after "[a creature] is dealt damage" its recipient: CR 608.2c leaves the
/// antecedent to the text, so the card states it (`triggers-architecture.md`
/// §12, TR-5a). Always "event role", never a bare "role", which the Role
/// tokens own (CR 111.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventRole {
    /// The one object an event about one object names: the card a zone change
    /// moved, the permanent tapped or that entered, the card drawn, the spell
    /// cast or countered, the source of mana, the permanent whose control
    /// changed.
    Object,
    /// CR 120.1: what dealt the damage.
    DamageSource,
    /// CR 120.1: the object the damage was dealt to.
    DamageRecipient,
    /// CR 508.1: a creature declared as an attacker.
    Attacker,
    /// CR 506.3: the planeswalker or battle a creature was declared
    /// attacking. A player attacked is no object.
    Attacked,
    /// CR 701.3: the Aura, Equipment or Fortification.
    Attachment,
    /// CR 701.3: the object it became attached to, or was attached to.
    Host,
}

impl EventRole {
    pub const ALL: [EventRole; 7] = [
        EventRole::Object,
        EventRole::DamageSource,
        EventRole::DamageRecipient,
        EventRole::Attacker,
        EventRole::Attacked,
        EventRole::Attachment,
        EventRole::Host,
    ];
}

/// What damage is dealt to (CR 120.1), or what is attacked (CR 506.3): an
/// object, or a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectOrPlayer {
    Object(ObjectRef),
    Player(PlayerId),
}

impl ObjectOrPlayer {
    /// The object; `None` for a player.
    pub fn object(self) -> Option<ObjectRef> {
        match self {
            ObjectOrPlayer::Object(object) => Some(object),
            ObjectOrPlayer::Player(_) => None,
        }
    }
}

/// The objects a triggering event is about, each by the part it played, as the
/// existence the dispatch found (CR 400.7): the one fact a record, which names
/// its objects by id, does not carry. One variant per shape of event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggeringEventObjects {
    /// A step beginning, a life change, a player losing.
    NoObject,
    /// [`EventRole::Object`].
    Object(ObjectRef),
    /// CR 120.1: the damage's source, as the existence that dealt it, and what
    /// it was dealt to.
    Damage { source: ObjectRef, recipient: ObjectOrPlayer },
    /// CR 508.1: one creature declared as an attacker, and what it attacks.
    Attack { attacker: ObjectRef, defender: ObjectOrPlayer },
    /// CR 508.3b: one player, planeswalker or battle attacked, and every
    /// creature declared attacking it.
    Defense { defender: ObjectOrPlayer, attackers: Vec<ObjectRef> },
    /// CR 701.3: an attachment and the object it became, or was, attached to.
    Attach { attachment: ObjectRef, host: ObjectRef },
}

impl TriggeringEventObjects {
    /// The object that played `role`, if this event has exactly one:
    /// a defender's attackers are several, and [`Self::objects`] has them.
    pub fn get(&self, role: EventRole) -> Option<ObjectRef> {
        match (self, role) {
            (TriggeringEventObjects::Object(object), EventRole::Object)
            | (TriggeringEventObjects::Damage { source: object, .. }, EventRole::DamageSource)
            | (TriggeringEventObjects::Attack { attacker: object, .. }, EventRole::Attacker)
            | (TriggeringEventObjects::Attach { attachment: object, .. }, EventRole::Attachment)
            | (TriggeringEventObjects::Attach { host: object, .. }, EventRole::Host) => Some(*object),
            (TriggeringEventObjects::Damage { recipient: object, .. }, EventRole::DamageRecipient)
            | (TriggeringEventObjects::Attack { defender: object, .. }, EventRole::Attacked)
            | (TriggeringEventObjects::Defense { defender: object, .. }, EventRole::Attacked) => object.object(),
            (
                TriggeringEventObjects::NoObject
                | TriggeringEventObjects::Object(_)
                | TriggeringEventObjects::Damage { .. }
                | TriggeringEventObjects::Attack { .. }
                | TriggeringEventObjects::Defense { .. }
                | TriggeringEventObjects::Attach { .. },
                _,
            ) => None,
        }
    }

    /// Each object, with the part it played.
    pub fn objects(&self) -> impl Iterator<Item = (EventRole, ObjectRef)> + '_ {
        // Up to two objects, then a defender's attackers.
        let (pairs, attackers): ([Option<(EventRole, ObjectRef)>; 2], &[ObjectRef]) = match self {
            TriggeringEventObjects::NoObject => ([None, None], &[]),
            TriggeringEventObjects::Object(object) => ([Some((EventRole::Object, *object)), None], &[]),
            TriggeringEventObjects::Damage { source, recipient } => (
                [Some((EventRole::DamageSource, *source)), recipient.object().map(|o| (EventRole::DamageRecipient, o))],
                &[],
            ),
            TriggeringEventObjects::Attack { attacker, defender } => {
                ([Some((EventRole::Attacker, *attacker)), defender.object().map(|o| (EventRole::Attacked, o))], &[])
            }
            TriggeringEventObjects::Defense { defender, attackers } => {
                ([defender.object().map(|o| (EventRole::Attacked, o)), None], attackers)
            }
            TriggeringEventObjects::Attach { attachment, host } => {
                ([Some((EventRole::Attachment, *attachment)), Some((EventRole::Host, *host))], &[])
            }
        };
        pairs.into_iter().flatten().chain(attackers.iter().map(|&attacker| (EventRole::Attacker, attacker)))
    }
}

/// One event that triggered an ability: one occurrence of its trigger event
/// (CR 603.2c), with the record that holds it, copied whole (§3.4), and the
/// objects it is about. Not the [`TriggerEvent`], which is what the ability
/// watches for.
#[derive(Debug, Clone, PartialEq)]
pub struct TriggeringEvent {
    pub record: EventRecord,
    pub objects: TriggeringEventObjects,
}

counted_enum! {
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
        AbilityResolved,
        Attached,
        Unattached,
        SpellCountered,
        PlayerLost,
        SameZoneMove,
        ControlChanged,
        CountersPutOn,
        CountersRemovedFrom,
    }
}

impl EventKind {
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
            GameEvent::AbilityResolved { .. } => EventKind::AbilityResolved,
            GameEvent::Attached { .. } => EventKind::Attached,
            GameEvent::Unattached { .. } => EventKind::Unattached,
            GameEvent::SpellCountered { .. } => EventKind::SpellCountered,
            GameEvent::PlayerLost { .. } => EventKind::PlayerLost,
            GameEvent::SameZoneMove { .. } => EventKind::SameZoneMove,
            GameEvent::ControlChanged { .. } => EventKind::ControlChanged,
            GameEvent::CountersPutOn { .. } => EventKind::CountersPutOn,
            GameEvent::CountersRemovedFrom { .. } => EventKind::CountersRemovedFrom,
            GameEvent::AbilityActivated { .. }
            | GameEvent::AbilityCountered { .. }
            | GameEvent::AbilityFizzled { .. }
            | GameEvent::BlockersDeclared { .. }
            | GameEvent::DelayedTriggerCreated { .. }
            | GameEvent::PlayerWon { .. }
            | GameEvent::RemovedFromCombat { .. }
            | GameEvent::Scried { .. }
            | GameEvent::SpellFizzled { .. }
            | GameEvent::StateBasedActionPerformed
            | GameEvent::Targeted { .. }
            | GameEvent::TokenCeasedToExist { .. }
            | GameEvent::TokenCreated { .. }
            | GameEvent::UntilReturnMade { .. } => return None,
        })
    }

    /// This kind's word in a mask, and its bit within that word.
    const fn place(self) -> (usize, u64) {
        let index = self as usize;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EventKindMask([u64; EventKind::WORDS]);

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

impl TriggerDef {
    /// Whether every condition is this object itself changing zones
    /// (`TriggerSubject::ThisObject`): Guerrilla Tactics' discard, Guile's "put
    /// into a graveyard from anywhere". Only the move itself can trigger such
    /// an ability. Read after the move, it is asked where the move left the
    /// object, by the one window that made it a newcomer there; read looking
    /// back, it is asked off the move's frame. So off the battlefield it waits
    /// on `newcomer_trigger_sources` rather than the zone map every window asks.
    pub fn triggers_only_on_this_object_moving(&self) -> bool {
        let arms = self.condition.events();
        !arms.is_empty() && arms.iter().all(|arm| matches!(arm, TriggerEvent::ZoneChange { subject: TriggerSubject::ThisObject, .. }))
    }
}

/// One arm per performed event kind, in `GameEvent`'s declaration order
/// (`triggers-architecture.md` §3.3). `None` on a field means the arm does
/// not ask. The arms the registered cards and fixtures read; the rest land
/// with their phase, listed in §3.3's table.
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerEvent {
    /// Dies is `from: Battlefield, to: Graveyard`; leaves the battlefield is
    /// `from: Battlefield, to: None`; "from anywhere" is `from: None`, which
    /// CR 603.6c makes *not* a leaves-the-battlefield ability.
    ZoneChange {
        subject: TriggerSubject,
        change: ZoneChangePattern,
        /// Who or what caused it, for "a spell or ability an opponent controls
        /// causes" (Guerrilla Tactics, Sacred Ground): the cause of what the
        /// arm names, its act's or its destination's
        /// (`replacement-architecture.md` §2b.7).
        by: Option<SourceFilter>,
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
    /// this. `place` is which of the player's draws it watches ("your second
    /// card each turn"), read off the record, which took it as it was drawn.
    DrawsCard { player: Option<PlayerRef>, multiplicity: Multiplicity, place: Option<DrawFilter> },
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
    /// CR 508.3a–e and 506.5, read off the declaration's record only: a
    /// creature put onto the battlefield attacking never "attacked" (CR
    /// 508.4). "Whenever [a player] attacks" (CR 508.3d) is `OncePerEvent`
    /// over a creature that player controls.
    Attacks {
        attacker: TriggerSubject,
        defender: Option<DefenderFilter>,
        per: AttackEventPer,
        /// CR 506.5: the only creature declared as an attacker.
        alone: bool,
        multiplicity: Multiplicity,
    },
    /// "Whenever [you/a player] cast[s] a [filter] spell" (CR 601.2i). The
    /// spell is on the stack as its record is dispatched, so the filter reads
    /// it through the layer walk.
    CastsSpell { caster: Option<PlayerRef>, spell: Option<ObjectFilter> },
    /// "Whenever a [filter] spell [you've cast] is countered" (CR 701.6a),
    /// looking back (CR 603.10e): the filter reads the spell as it was on the
    /// stack, and `caster` is who cast it, which an effect that took control
    /// of the spell does not change. A spell that doesn't resolve because its
    /// targets are illegal isn't countered (Multani's Presence's ruling), and
    /// no printed trigger reads an ability being countered.
    IsCountered { spell: Option<ObjectFilter>, caster: Option<PlayerRef> },
    /// "Whenever [a player/an opponent] loses the game" (CR 603.9), for any
    /// reason, looking back (CR 603.10f): the loser's permanents leave in
    /// the same event and are asked as they were, so one the loser owned and
    /// another player controlled triggers for that player. A draw is no loss
    /// (CR 104.4a).
    PlayerLoses { player: Option<PlayerRef> },
    /// "Whenever [you/an opponent] shuffle[s] [your/their] library" (CR
    /// 701.24): one record per shuffle, a library of zero or one cards
    /// included (CR 701.24e). Cascade's random bottom is no shuffle, and emits
    /// no record (Cosi's Trickster's rulings).
    ShufflesLibrary { player: Option<PlayerRef> },
    /// "Whenever a +1/+1 counter is put on this creature" (`PerOccurrence`,
    /// one trigger per counter: Fathom Mage's ninth ruling), "one or more
    /// ... on a creature you control" (`OncePerSubject`), "one or more ... on
    /// one or more" (`OncePerEvent`). Counters a permanent enters with are
    /// put on it (CR 122.6). `by` is who put them on (CR 122.6a): "whenever
    /// you put". CR 122.7's "the Nth counter" is the ability's own
    /// intervening "if", `Condition::CounterCountReached`, as CR 714.2b
    /// words a Saga's chapter.
    CountersPutOn {
        subject: CountersOn,
        counter_type: Option<CounterType>,
        by: Option<PlayerRef>,
        multiplicity: Multiplicity,
    },
    /// "Whenever a +1/+1 counter is removed from this creature" (one per
    /// counter: Protean Hydra's fourth ruling), CR 704.5q's annihilation
    /// included (its fifth). `by` is who removed them, where a player did:
    /// a rule's removal matches no `by`.
    CountersRemovedFrom {
        subject: CountersOn,
        counter_type: Option<CounterType>,
        by: Option<PlayerRef>,
        multiplicity: Multiplicity,
    },
    /// "Whenever [an Aura] becomes attached to [this creature]" (CR 701.3a),
    /// on the transition only (CR 603.2e, 701.3b): entering attached is one
    /// (CR 303.4f), moving from another object is one, and an attach to the
    /// object it is already on is none (Siona, Captain of the Pyleas' first
    /// ruling). Read after the event, so an Aura that makes its object stop
    /// being a creature you control does not trigger Siona (her fourth
    /// ruling). "That creature" is the object it became attached to.
    BecomesAttached { attachment: TriggerSubject, host: TriggerSubject },
    /// "Whenever [this Equipment] becomes unattached from [a permanent]": every
    /// route CR 701.3d counts (Grafted Wargear's first ruling), looking back
    /// (CR 603.10c), so an Equipment that left triggers off how it was. "That
    /// permanent" is the object it was attached to, as the existence it was
    /// (CR 400.7).
    BecomesUnattached { attachment: TriggerSubject, former_host: TriggerSubject },
    /// "When you lose control of [subject]" is `from: You`; "whenever an
    /// opponent gains control of [subject] from you" adds `to: Opponent`.
    /// Looking back (CR 603.10d): the subject as it was before the change, and
    /// "you" its controller then. Losing control includes the permanent
    /// leaving the battlefield (Khârn the Betrayer's first ruling), so an arm
    /// with `to: None` reads a battlefield departure too, controlled by `from`
    /// as it left; another player gaining control does not.
    ControlChanges { subject: TriggerSubject, from: Option<PlayerRef>, to: Option<PlayerRef> },
    /// CR 603.3b's second tier, by construction: the event the dispatcher
    /// emits per queued trigger (§4.8).
    AbilityTriggers { caused_by: Option<Box<TriggerEvent>>, source: Option<ObjectFilter> },
    /// "When this ability has resolved" (CR 603.7h): the ability's own
    /// `AbilityResolved` record, which CR 608.2n's last step announces. For
    /// a delayed trigger "this ability" is the one that created it, and "the
    /// third time this turn" is the creating resolution's condition, which
    /// creates it only then (§6.5).
    AbilityResolves { identity: IdentityRef },
}

/// Which ability an `AbilityResolves` arm watches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityRef {
    /// The ability whose arm this is, or for a delayed trigger the ability
    /// that created it — the same ability of the same object, whichever
    /// instance (Ashling the Pilgrim's ruling, §6.5).
    ThisAbility,
}

/// CR 603.10a's three classes of zone change, as a property of one move:
/// leaving the battlefield, a card leaving a graveyard, and an object all
/// players can see put into a hand or library. A move in a class carries the
/// frame from before it, which is what a look-back arm reads.
///
/// `to` is `None` for leaving the game (CR 800.4a), which counts only from the
/// battlefield: CR 603.6c makes it a leaves-the-battlefield event, and a card
/// leaving the game from a graveyard is no card leaving a graveyard (Erebos's
/// Titan's third ruling).
pub fn zone_change_looks_back(from: Zone, to: Option<Zone>) -> bool {
    match to {
        None => from == Zone::Battlefield,
        Some(to) => {
            matches!(from, Zone::Battlefield | Zone::Graveyard) || (from.is_public() && matches!(to, Zone::Hand | Zone::Library))
        }
    }
}

impl TriggerEvent {
    /// CR 603.10's list, as a property of the arm's fields and never a flag a
    /// card sets (§4.3): CR 603.10a's three zone-change classes as an arm can
    /// match them (`zone_change_looks_back`), so "into a hand" with no `from`
    /// looks back at a move from a zone all players could see; becoming
    /// unattached (603.10c); a change of control (603.10d); a spell being
    /// countered (603.10e); and a player losing the game (603.10f). Phasing
    /// out (603.10b) arrives with phasing.
    pub fn looks_back(&self) -> bool {
        match self {
            TriggerEvent::ZoneChange { change, .. } => {
                matches!(change.from, Some(Zone::Battlefield | Zone::Graveyard))
                    || (matches!(change.to, Some(Zone::Hand | Zone::Library)) && change.from.is_none_or(|from| from.is_public()))
            }
            TriggerEvent::BecomesUnattached { .. }
            | TriggerEvent::ControlChanges { .. }
            | TriggerEvent::IsCountered { .. }
            | TriggerEvent::PlayerLoses { .. } => {
                true
            }
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
            | TriggerEvent::CountersPutOn { .. }
            | TriggerEvent::CountersRemovedFrom { .. }
            | TriggerEvent::BecomesAttached { .. }
            | TriggerEvent::AbilityTriggers { .. }
            | TriggerEvent::AbilityResolves { .. } => false,
        }
    }

    /// CR 603.6e's trigger: an Aura's (or any attachment's) on the permanent
    /// it is attached to leaving the battlefield, the one CR 400.7f lets find
    /// the Aura's new object in a graveyard.
    pub fn watches_its_host_leave(&self) -> bool {
        matches!(self, TriggerEvent::ZoneChange { subject: TriggerSubject::Host, change, .. } if change.from == Some(Zone::Battlefield))
    }

    /// The kinds of record this arm can match — the discriminant half of
    /// the matcher. [`Self::reads`] asks it of one record, and the
    /// dispatcher's source mask is its union over a permanent's printed
    /// arms: one table, so the two cannot disagree.
    ///
    /// CR 603.6c is why `ZoneChange` names two: a leaves-the-battlefield
    /// ability reads the departure a player leaving the game causes, and
    /// that record is `LeftTheGame` rather than a zone change (§4.2 leg 2).
    /// An arm that names an act reads a third, the act a replacement kept in
    /// its zone (`GameEvent::SameZoneMove`), since it still happened.
    pub fn record_kinds(&self) -> EventKindMask {
        match self {
            TriggerEvent::ZoneChange { change, .. } => {
                let moved = EventKindMask::of(EventKind::ZoneChange).with(EventKind::LeftTheGame);
                if change.act.is_some() { moved.with(EventKind::SameZoneMove) } else { moved }
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
            TriggerEvent::IsCountered { .. } => EventKindMask::of(EventKind::SpellCountered),
            TriggerEvent::PlayerLoses { .. } => EventKindMask::of(EventKind::PlayerLost),
            TriggerEvent::ShufflesLibrary { .. } => EventKindMask::of(EventKind::LibraryShuffled),
            TriggerEvent::CountersPutOn { .. } => EventKindMask::of(EventKind::CountersPutOn),
            TriggerEvent::CountersRemovedFrom { .. } => EventKindMask::of(EventKind::CountersRemovedFrom),
            TriggerEvent::BecomesAttached { .. } => EventKindMask::of(EventKind::Attached),
            TriggerEvent::BecomesUnattached { .. } => EventKindMask::of(EventKind::Unattached),
            TriggerEvent::ControlChanges { to, .. } => {
                let changed = EventKindMask::of(EventKind::ControlChanged);
                if to.is_none() { changed.with(EventKind::ZoneChange).with(EventKind::LeftTheGame) } else { changed }
            }
            TriggerEvent::AbilityTriggers { .. } => EventKindMask::of(EventKind::AbilityTriggered),
            TriggerEvent::AbilityResolves { .. } => EventKindMask::of(EventKind::AbilityResolved),
        }
    }

    /// Whether `record` is the kind of event this arm reads at all — the
    /// discriminant test the predicates follow.
    pub fn reads(&self, record: &GameEvent) -> bool {
        EventKind::from_record(record).is_some_and(|kind| self.record_kinds().contains(kind))
    }

    /// The object the arm is about, for the arms about one: the creature
    /// that dies, the permanent that becomes tapped, the source that deals
    /// damage. `None` for an arm about a player, a step or an ability.
    pub fn subject(&self) -> Option<&TriggerSubject> {
        match self {
            TriggerEvent::ZoneChange { subject, .. }
            | TriggerEvent::BecomesTapped { subject }
            | TriggerEvent::BecomesUntapped { subject }
            | TriggerEvent::EntersBattlefield { subject, .. }
            | TriggerEvent::ManaAdded { source: subject, .. }
            | TriggerEvent::DamageDealt { source: subject, .. }
            | TriggerEvent::Attacks { attacker: subject, .. }
            | TriggerEvent::BecomesAttached { host: subject, .. }
            | TriggerEvent::BecomesUnattached { former_host: subject, .. }
            | TriggerEvent::ControlChanges { subject, .. }
            | TriggerEvent::CountersPutOn { subject: CountersOn::Object(subject), .. }
            | TriggerEvent::CountersRemovedFrom { subject: CountersOn::Object(subject), .. } => Some(subject),
            TriggerEvent::CountersPutOn { subject: CountersOn::Player(_), .. }
            | TriggerEvent::CountersRemovedFrom { subject: CountersOn::Player(_), .. }
            | TriggerEvent::DrawsCard { .. }
            | TriggerEvent::PhaseBegins { .. }
            | TriggerEvent::StepBegins { .. }
            | TriggerEvent::TurnBegins { .. }
            | TriggerEvent::GainsLife { .. }
            | TriggerEvent::LosesLife { .. }
            | TriggerEvent::CastsSpell { .. }
            | TriggerEvent::IsCountered { .. }
            | TriggerEvent::PlayerLoses { .. }
            | TriggerEvent::ShufflesLibrary { .. }
            | TriggerEvent::AbilityTriggers { .. }
            | TriggerEvent::AbilityResolves { .. } => None,
        }
    }

    /// Whether this arm may be read once per subject: whether one event can
    /// make several of its records about one object or player, so that
    /// grouping them by it differs from reading each. Counters put on one
    /// creature by two players, or of two kinds, are two records. A death is
    /// one record per object, so per subject would be per record. The damage
    /// arm is the next to join, for "one or more creatures deal combat damage
    /// to a player" (`triggers-architecture.md` §4.4).
    pub fn supports_once_per_subject(&self) -> bool {
        match self {
            TriggerEvent::CountersPutOn { .. } | TriggerEvent::CountersRemovedFrom { .. } => true,
            TriggerEvent::ZoneChange { .. }
            | TriggerEvent::BecomesTapped { .. }
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
            | TriggerEvent::IsCountered { .. }
            | TriggerEvent::PlayerLoses { .. }
            | TriggerEvent::ShufflesLibrary { .. }
            | TriggerEvent::BecomesAttached { .. }
            | TriggerEvent::BecomesUnattached { .. }
            | TriggerEvent::ControlChanges { .. }
            | TriggerEvent::AbilityTriggers { .. }
            | TriggerEvent::AbilityResolves { .. } => false,
        }
    }

    /// The object or player `record` is about, for an arm that may be read
    /// once per subject (`supports_once_per_subject`): what a once-per-subject
    /// trigger is one of. `None` for every other arm and record.
    pub fn occurrence_subject(&self, record: &GameEvent) -> Option<crate::events::event::CounterSubject> {
        match (self, record) {
            (TriggerEvent::CountersPutOn { .. }, GameEvent::CountersPutOn { subject, .. })
            | (TriggerEvent::CountersRemovedFrom { .. }, GameEvent::CountersRemovedFrom { subject, .. }) => Some(*subject),
            _ => None,
        }
    }

    /// Whether the arm watches an object a delayed trigger refers to (CR
    /// 603.7c): "when that token dies".
    pub fn watches_referred(&self) -> bool {
        self.subject() == Some(&TriggerSubject::Referred)
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
            | TriggerEvent::Attacks { multiplicity, .. }
            | TriggerEvent::CountersPutOn { multiplicity, .. }
            | TriggerEvent::CountersRemovedFrom { multiplicity, .. } => *multiplicity,
            TriggerEvent::BecomesTapped { .. }
            | TriggerEvent::BecomesUntapped { .. }
            | TriggerEvent::ManaAdded { .. }
            | TriggerEvent::PhaseBegins { .. }
            | TriggerEvent::StepBegins { .. }
            | TriggerEvent::TurnBegins { .. }
            | TriggerEvent::CastsSpell { .. }
            | TriggerEvent::IsCountered { .. }
            | TriggerEvent::PlayerLoses { .. }
            | TriggerEvent::ShufflesLibrary { .. }
            | TriggerEvent::BecomesAttached { .. }
            | TriggerEvent::BecomesUnattached { .. }
            | TriggerEvent::ControlChanges { .. }
            | TriggerEvent::AbilityTriggers { .. }
            | TriggerEvent::AbilityResolves { .. } => Multiplicity::PerOccurrence,
        }
    }

    /// Whether one triggering event of this arm may have several objects in
    /// `role`'s part: a defender's attackers (CR 508.3b). An
    /// instruction that needs exactly one, "its power", cannot name it.
    pub fn may_have_several(&self, role: EventRole) -> bool {
        matches!(
            (self, role),
            (TriggerEvent::Attacks { per: AttackEventPer::Defender, .. }, EventRole::Attacker)
        )
    }

    /// The parts an object can play in this arm's triggering events: what an
    /// instruction may name, which a registry test checks every registered
    /// card against (TR-5a).
    pub fn event_roles(&self) -> &'static [EventRole] {
        match self {
            TriggerEvent::ZoneChange { .. }
            | TriggerEvent::BecomesTapped { .. }
            | TriggerEvent::BecomesUntapped { .. }
            | TriggerEvent::DrawsCard { .. }
            | TriggerEvent::ManaAdded { .. }
            | TriggerEvent::EntersBattlefield { .. }
            | TriggerEvent::CastsSpell { .. }
            | TriggerEvent::IsCountered { .. }
            | TriggerEvent::ControlChanges { .. }
            | TriggerEvent::CountersPutOn { subject: CountersOn::Object(_), .. }
            | TriggerEvent::CountersRemovedFrom { subject: CountersOn::Object(_), .. } => &[EventRole::Object],
            TriggerEvent::DamageDealt { .. } => &[EventRole::DamageSource, EventRole::DamageRecipient],
            TriggerEvent::Attacks { .. } => &[EventRole::Attacker, EventRole::Attacked],
            TriggerEvent::BecomesAttached { .. } | TriggerEvent::BecomesUnattached { .. } => {
                &[EventRole::Attachment, EventRole::Host]
            }
            TriggerEvent::PhaseBegins { .. }
            | TriggerEvent::StepBegins { .. }
            | TriggerEvent::TurnBegins { .. }
            | TriggerEvent::GainsLife { .. }
            | TriggerEvent::LosesLife { .. }
            | TriggerEvent::PlayerLoses { .. }
            | TriggerEvent::ShufflesLibrary { .. }
            | TriggerEvent::CountersPutOn { subject: CountersOn::Player(_), .. }
            | TriggerEvent::CountersRemovedFrom { subject: CountersOn::Player(_), .. }
            | TriggerEvent::AbilityTriggers { .. }
            | TriggerEvent::AbilityResolves { .. } => &[],
        }
    }

    /// "That player" (§3.4). `None` for an event about no player.
    pub fn player_of(&self, record: &GameEvent) -> Option<PlayerId> {
        match (self, record) {
            (TriggerEvent::ZoneChange { .. }, GameEvent::ZoneChange { owner, .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::LeftTheGame { owner, .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::SameZoneMove { owner, .. }) => Some(*owner),
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
            // The player who gained control.
            (TriggerEvent::ControlChanges { .. }, GameEvent::ControlChanged { to, .. }) => Some(*to),
            // Its controller as it left the stack, as the record keeps it.
            (TriggerEvent::IsCountered { .. }, GameEvent::SpellCountered { controller, .. }) => Some(*controller),
            (TriggerEvent::DrawsCard { .. }, GameEvent::CardDrawn { player_id, .. })
            | (TriggerEvent::ShufflesLibrary { .. }, GameEvent::LibraryShuffled { player_id })
            | (TriggerEvent::PlayerLoses { .. }, GameEvent::PlayerLost { player_id }) => Some(*player_id),
            (TriggerEvent::EntersBattlefield { .. }, GameEvent::PermanentEnteredBattlefield { controller, .. }) => {
                Some(*controller)
            }
            // The player who got them; counters on an object name no player.
            (TriggerEvent::CountersPutOn { .. }, GameEvent::CountersPutOn { subject, .. })
            | (TriggerEvent::CountersRemovedFrom { .. }, GameEvent::CountersRemovedFrom { subject, .. }) => match subject {
                crate::events::event::CounterSubject::Player(player) => Some(*player),
                crate::events::event::CounterSubject::Object(_) => None,
            },
            (TriggerEvent::AbilityTriggers { .. }, GameEvent::AbilityTriggered { controller, .. })
            | (TriggerEvent::AbilityResolves { .. }, GameEvent::AbilityResolved { controller, .. }) => {
                Some(*controller)
            }
            (TriggerEvent::BecomesTapped { .. }, GameEvent::Tapped { .. })
            | (TriggerEvent::BecomesUntapped { .. }, GameEvent::Untapped { .. })
            | (TriggerEvent::BecomesAttached { .. }, GameEvent::Attached { .. })
            | (TriggerEvent::BecomesUnattached { .. }, GameEvent::Unattached { .. })
            | (TriggerEvent::ControlChanges { .. }, GameEvent::ZoneChange { .. })
            | (TriggerEvent::ControlChanges { .. }, GameEvent::LeftTheGame { .. })
            | (TriggerEvent::Attacks { .. }, GameEvent::AttackersDeclared { .. }) => None,
            // Every arm, by name, so a new arm does not compile until it says what
            // this projection is for it; the record never matches the arm otherwise.
            (
                TriggerEvent::ZoneChange { .. }
                | TriggerEvent::BecomesTapped { .. }
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
                | TriggerEvent::IsCountered { .. }
                | TriggerEvent::PlayerLoses { .. }
                | TriggerEvent::ShufflesLibrary { .. }
                | TriggerEvent::CountersPutOn { .. }
                | TriggerEvent::CountersRemovedFrom { .. }
                | TriggerEvent::BecomesAttached { .. }
                | TriggerEvent::BecomesUnattached { .. }
                | TriggerEvent::ControlChanges { .. }
                | TriggerEvent::AbilityTriggers { .. }
                | TriggerEvent::AbilityResolves { .. },
                _,
            ) => None,
        }
    }

    /// "That many" (§3.4): the damage dealt, the mana added, the objects
    /// moved. `None` for an event with no quantity.
    pub fn amount_of(&self, record: &GameEvent) -> Option<u64> {
        match (self, record) {
            (TriggerEvent::DamageDealt { .. }, GameEvent::DamageDealt { amount, .. }) => Some(*amount),
            (TriggerEvent::ManaAdded { .. }, GameEvent::ManaAdded { mana, .. }) => {
                Some(mana.iter().map(|(_, n)| *n).sum())
            }
            (TriggerEvent::GainsLife { .. }, GameEvent::LifeChanged { old, new, .. }) => {
                Some((new - old).max(0) as u64)
            }
            // A counter is one occurrence of a per-counter arm, so "that many"
            // of it is one; the other readings sum the record's counters.
            (TriggerEvent::CountersPutOn { multiplicity, .. }, GameEvent::CountersPutOn { n, .. })
            | (TriggerEvent::CountersRemovedFrom { multiplicity, .. }, GameEvent::CountersRemovedFrom { n, .. }) => {
                Some(match multiplicity {
                    Multiplicity::PerOccurrence => 1,
                    Multiplicity::OncePerEvent | Multiplicity::OncePerSubject => u64::from(*n),
                })
            }
            (TriggerEvent::LosesLife { .. }, GameEvent::LifeChanged { old, new, .. }) => {
                Some((old - new).max(0) as u64)
            }
            // One card or object a record, so "that many" over a "one or more"
            // binding is the cards drawn, or the objects moved: "the number of
            // cards milled" counts a Darksteel Colossus kept in its library.
            (TriggerEvent::DrawsCard { .. }, GameEvent::CardDrawn { .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::ZoneChange { .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::LeftTheGame { .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::SameZoneMove { .. }) => Some(1),
            (TriggerEvent::BecomesTapped { .. }, GameEvent::Tapped { .. })
            | (TriggerEvent::BecomesUntapped { .. }, GameEvent::Untapped { .. })
            | (TriggerEvent::PhaseBegins { .. }, GameEvent::PhaseBegin { .. })
            | (TriggerEvent::StepBegins { .. }, GameEvent::StepBegin { .. })
            | (TriggerEvent::TurnBegins { .. }, GameEvent::TurnBegin { .. })
            | (TriggerEvent::EntersBattlefield { .. }, GameEvent::PermanentEnteredBattlefield { .. })
            | (TriggerEvent::Attacks { .. }, GameEvent::AttackersDeclared { .. })
            | (TriggerEvent::CastsSpell { .. }, GameEvent::SpellCast { .. })
            | (TriggerEvent::IsCountered { .. }, GameEvent::SpellCountered { .. })
            | (TriggerEvent::PlayerLoses { .. }, GameEvent::PlayerLost { .. })
            | (TriggerEvent::ShufflesLibrary { .. }, GameEvent::LibraryShuffled { .. })
            | (TriggerEvent::BecomesAttached { .. }, GameEvent::Attached { .. })
            | (TriggerEvent::BecomesUnattached { .. }, GameEvent::Unattached { .. })
            | (TriggerEvent::ControlChanges { .. }, GameEvent::ControlChanged { .. })
            | (TriggerEvent::ControlChanges { .. }, GameEvent::ZoneChange { .. })
            | (TriggerEvent::ControlChanges { .. }, GameEvent::LeftTheGame { .. })
            | (TriggerEvent::AbilityTriggers { .. }, GameEvent::AbilityTriggered { .. })
            | (TriggerEvent::AbilityResolves { .. }, GameEvent::AbilityResolved { .. }) => None,
            // Every arm, by name, so a new arm does not compile until it says what
            // this projection is for it; the record never matches the arm otherwise.
            (
                TriggerEvent::ZoneChange { .. }
                | TriggerEvent::BecomesTapped { .. }
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
                | TriggerEvent::IsCountered { .. }
                | TriggerEvent::PlayerLoses { .. }
                | TriggerEvent::ShufflesLibrary { .. }
                | TriggerEvent::CountersPutOn { .. }
                | TriggerEvent::CountersRemovedFrom { .. }
                | TriggerEvent::BecomesAttached { .. }
                | TriggerEvent::BecomesUnattached { .. }
                | TriggerEvent::ControlChanges { .. }
                | TriggerEvent::AbilityTriggers { .. }
                | TriggerEvent::AbilityResolves { .. },
                _,
            ) => None,
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
    /// The events that triggered it (CR 603.2c): one for a `PerOccurrence`
    /// trigger, every one of its window's for a `OncePerEvent` trigger, each
    /// with its record and the objects it is about.
    pub triggering_events: Vec<TriggeringEvent>,
    /// Which event matched — whose projections say what the bound facts are.
    pub event: EventIndex,
    /// The ability whose triggering this is (CR 603.3b's second tier).
    pub triggered_by: Option<TriggerSeq>,
    /// CR 603.7c — what a delayed triggered ability refers to, carried to
    /// the stack for its "that card" and "it"; empty for an object's trigger.
    pub referred: Referred,
    /// CR 400.7f — the card an Aura whose trigger watches its enchanted
    /// permanent leave (CR 603.6e) became in its owner's graveyard, which
    /// "this card" finds: put there in the same event, set at dispatch, or by
    /// CR 704.5m's check while the trigger waits, set at that move.
    ///
    /// The one field written after the ability triggered. It lives here
    /// rather than beside `kept_lki` on the three entries because the binding
    /// travels from the pending trigger to the stack and into the resolution
    /// by itself, and so would a copy of the ability (CR 707.10); on the
    /// entries, every one of those hand-offs would have to carry it.
    pub source_found_in_graveyard: Option<ObjectRef>,
}

impl TriggerBinding {
    /// The event the binding's projections read through.
    pub fn event(&self) -> &TriggerEvent {
        &self.def.condition.events()[self.event.0]
    }

    /// The triggering events' records, in the window's order.
    pub fn records(&self) -> impl Iterator<Item = &EventRecord> {
        self.triggering_events.iter().map(|event| &event.record)
    }

    /// The first triggering event's record, which caused the trigger.
    pub fn first_record(&self) -> Option<&EventRecord> {
        self.records().next()
    }

    /// The one triggering event of a `PerOccurrence` trigger, or the first of
    /// a once-per-subject trigger's, whose events are all about its one
    /// subject. `None` for a "one or more" trigger, which has several.
    pub fn triggering_event(&self) -> Option<&TriggeringEvent> {
        match self.event().multiplicity() {
            Multiplicity::PerOccurrence | Multiplicity::OncePerSubject => self.triggering_events.first(),
            Multiplicity::OncePerEvent => None,
        }
    }

    /// "Its power", "if it had": the object that played `role` in the
    /// one triggering event, as the existence the dispatch found. `None` for a
    /// "one or more" trigger, which has no single object.
    pub fn triggering_object(&self, role: EventRole) -> Option<ObjectRef> {
        self.triggering_event()?.objects.get(role)
    }

    /// "That creature", "them": each object that played `role` in each
    /// triggering event. One for a `PerOccurrence` trigger, but for a
    /// defender's attackers.
    pub fn triggering_objects(&self, role: EventRole) -> Vec<ObjectRef> {
        self.triggering_events
            .iter()
            .flat_map(|event| event.objects.objects().filter(move |&(part, _)| part == role))
            .map(|(_, object)| object)
            .collect()
    }
}

/// Where a pending trigger came from (§3.8). A reflexive trigger is
/// `Delayed`, since CR 603.12 has it follow the delayed rules; the rule-owned
/// origin lands with TR-6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerOrigin {
    /// A printed, granted or copied ability of an object.
    Object(AbilityIdentity),
    /// CR 603.7 — a delayed triggered ability, by the identity CR 603.7d–g
    /// give it: its source, and its registry number as the ability
    /// (`AbilityId::delayed`). It is no ability of that object, and CR
    /// 603.2d's "triggers additional times" does not reach it.
    Delayed(AbilityIdentity),
}

impl TriggerOrigin {
    /// The ability that triggered, by what it is.
    pub fn identity(&self) -> AbilityIdentity {
        match self {
            TriggerOrigin::Object(identity) | TriggerOrigin::Delayed(identity) => *identity,
        }
    }

    pub fn source(&self) -> ObjectId {
        self.source_ref().id
    }

    /// The source, and which existence of it (CR 400.7).
    pub fn source_ref(&self) -> ObjectRef {
        self.identity().source
    }
}

/// CR 603.7b — how many times a delayed triggered ability triggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayedDuration {
    /// "The next time": it triggers on the first event that matches and is
    /// gone. If several match at once, its controller chooses which one
    /// causes it.
    Once,
    /// "This turn": it triggers on every matching event until CR 514.2 ends
    /// the turn's "this turn" effects in the cleanup step.
    ThisTurn,
    /// CR 603.12 — a reflexive triggered ability, which follows the rules
    /// for delayed ones "except that [it is] checked immediately after being
    /// created" against what the resolution creating it has performed so
    /// far, and never waits.
    Reflexive(ReflexiveForm),
}

/// CR 603.12's two forms: "when [a player] [does or doesn't] take that
/// action". Its def's event is the action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReflexiveForm {
    /// "When you do", and "when [something happens] this way": it triggers
    /// once for each time its event occurred earlier during the resolution
    /// (CR 603.12a), or once for them all where its event is "one or more".
    Does,
    /// "When you don't": it triggers once if its event did not occur.
    Doesnt,
}

/// The resolution a delayed triggered ability is created during: what a
/// reflexive one asks about (CR 603.12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatingResolution {
    pub stamp: ResolutionStamp,
    /// "That action": the records of the last instruction before it that took
    /// an action, from its first to the one after its last. `None` when no
    /// instruction before it took one.
    pub that_action: Option<(EventSeq, EventSeq)>,
}

/// Which turn a delayed triggered ability may trigger in, as a card prints it.
/// It grows with the cards that need it: the CR does not list the ways a card
/// names a turn. "Your next upkeep" (the Pacts) needs no arm, being `whose:
/// You` on the step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayedTurn {
    /// The card names no turn, so it triggers in whichever turn its event
    /// comes in: "at the beginning of the next end step" is next turn's
    /// once this turn's has begun (Protean Hydra's sixth ruling). How many
    /// times it triggers is its `DelayedDuration`, whose `ThisTurn` is "this
    /// turn".
    NoneNamed,
    /// A turn after the one it is created in: the Ice Age cantrips' "at the
    /// beginning of the next turn's upkeep", which a second upkeep this turn
    /// (CR 500.10) is not.
    NextTurn,
    /// "That turn": the extra turn the same resolution scheduled before
    /// creating it (CR 500.7), Final Fortune's "at the beginning of that
    /// turn's end step". A skipped extra turn never comes (CR 614.10a), and
    /// the trigger goes with it.
    ThatExtraTurn,
}

/// The turns a registered delayed trigger may trigger in: its template's
/// `DelayedTurn`, fixed as it was created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerTurn {
    Any,
    /// A turn numbered past this one. A skipped turn advances no number (CR
    /// 614.10a), so the first is the next turn that begins.
    LaterThan(u32),
    /// The one extra turn (CR 500.7).
    Extra(ExtraTurnId),
}

/// What `Primitive::CreateDelayedTrigger` creates (CR 603.7), as a card
/// prints it. Its source, controller, X and turn belong to the moment it is
/// created, and the resolution fills them in (`triggers-architecture.md`
/// §3.9).
#[derive(Debug, Clone, PartialEq)]
pub struct DelayedTriggerTemplate {
    /// What it watches, its intervening "if", and what it does. Its targets
    /// are chosen as it is put on the stack (CR 603.3d), never as the spell
    /// that creates it is cast.
    pub def: Arc<TriggerDef>,
    pub duration: DelayedDuration,
    pub turn: DelayedTurn,
    /// The words the card prints for it, which no `TriggerDef` carries:
    /// what a log and a client show while it waits. Written by the card's
    /// author, as an ability's are.
    pub rules_text: AbilityText,
}

/// A delayed triggered ability waiting for its event (CR 603.7): on
/// `GameState::delayed_triggers` from the moment it is created until it
/// triggers (`Once`), its duration ends, or its turn has gone.
#[derive(Debug, Clone)]
pub struct DelayedTrigger {
    pub id: DelayedTriggerId,
    pub def: Arc<TriggerDef>,
    /// CR 603.7d–g's source, by identity (CR 400.7): the spell as it
    /// resolved, the source of the ability that created it, or the object
    /// with the static ability. "This creature" in its text is this object.
    pub source: ObjectRef,
    /// The record of the move that ended `source` as that object, once one
    /// has (CR 400.7). A departure record names the object's id and not
    /// which existence of it moved, so this is how "when this creature
    /// leaves the battlefield" knows its own departure and no later move of
    /// a new object with the same id.
    pub source_left_at: Option<EventSeq>,
    /// CR 113.7a, 608.2h — `source`'s last known information once it has
    /// left the zone it was in when this was created: kept as it left, or
    /// as this was created if it had already gone. Its trigger carries it to
    /// the stack, and its intervening "if" reads the source off it. `None`
    /// while the source is still there, and after a move nothing frames: CR
    /// 601.2a's, out of a hidden zone.
    pub source_lki: Option<Lki>,
    /// What its stack object is built from (CR 603.3's "the text of the
    /// ability that created it"), held because the source may be gone by
    /// then.
    pub source_card: Arc<CardData>,
    /// CR 603.7d–g's controller, as of the moment it was created.
    pub controller: PlayerId,
    /// "Its owner": the source's (CR 108.3), read as it was created, since
    /// the store may lose the source before the trigger reads it.
    pub owner: PlayerId,
    /// The first record performed after it was created, and the batch it was
    /// created during, if any: CR 603.7a's "won't trigger until it has
    /// actually been created". A rider (CR 615.5) creates one inside the
    /// batch it rides on, after some of that event's members are performed
    /// and before others are, and the whole event happened "just beforehand".
    pub created_at: EventSeq,
    pub created_in: Option<BatchId>,
    /// The ability that created it, for CR 603.7h's "this ability"; `None`
    /// for a spell's and a replacement effect's.
    pub created_by: Option<AbilityIdentity>,
    pub duration: DelayedDuration,
    /// CR 107.3n — the X of the spell or ability that created it.
    pub x_value: Option<u64>,
    /// The turns it may trigger in.
    pub turn: TriggerTurn,
    /// CR 601.2c's instances of "target" its def declares, announced as it
    /// is put on the stack (603.3d).
    pub instances: Vec<EffectRecipient>,
    /// Its template's words.
    pub rules_text: AbilityText,
    /// CR 603.7c — the objects and players its "that card", "it" and "that
    /// token" mean: what its creator remembered (`Effect::Remember`).
    pub referred: Referred,
}

impl DelayedTrigger {
    /// Its identity (CR 603.7d–g): the source, and its registry number as
    /// the ability.
    pub fn identity(&self) -> AbilityIdentity {
        AbilityIdentity { source: self.source, ability: AbilityId::delayed(self.id) }
    }
}

/// Whose a new delayed triggered ability is, as CR 603.7d–g say for what
/// creates it, taken at the moment it is created: what
/// `GameState::register_delayed_trigger` adds to a template.
#[derive(Debug, Clone)]
pub struct DelayedProvenance {
    pub source: ObjectRef,
    pub source_card: Arc<CardData>,
    pub controller: PlayerId,
    pub created_by: Option<AbilityIdentity>,
    pub x_value: Option<u64>,
    pub turn: TriggerTurn,
    /// The resolution creating it, if one is: a replacement's rider (CR
    /// 603.7f) is none.
    pub resolution: Option<CreatingResolution>,
    /// What caused its creation: the effect creating it, a resolution's or a
    /// rider's.
    pub cause: EventCause,
}

/// CR 603.7c — what a delayed triggered ability refers to: the objects and
/// players the instruction its creator remembered acted on
/// (`Effect::Remember`), each object as that instruction left it. The ability
/// "still affects it even if the object changes characteristics", and not
/// once it has left and returned (CR 400.7).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Referred {
    pub objects: Vec<RememberedObject>,
    pub players: Vec<PlayerId>,
}

/// An object remembered by identity (CR 400.7), with the record of the move
/// that ended it as that object once one has. A departure record names the
/// object's id and not which existence of it moved, so this is how "when this
/// creature leaves the battlefield" and "when that token dies" know their own
/// object's move and no later object's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RememberedObject {
    pub object: ObjectRef,
    pub left_at: Option<EventSeq>,
}

impl RememberedObject {
    /// An existence that has not moved yet.
    pub fn now(object: ObjectRef) -> Self {
        RememberedObject { object, left_at: None }
    }

    /// Whether record `seq`'s object `id` is this existence: its own
    /// departure, or, while it has not moved, the object as it is.
    pub fn is(self, game: &GameState, id: ObjectId, seq: EventSeq) -> bool {
        id == self.object.id
            && match self.left_at {
                Some(at) => at == seq,
                None => game.find(self.object).is_some(),
            }
    }

    /// The object, while it is still this existence (CR 603.7c), in a zone
    /// all players can see: one in a hand or a library can't be found (CR
    /// 400.7j).
    pub fn found(self, game: &GameState) -> Option<ObjectId> {
        game.find(self.object).filter(|object| object.zone.is_public()).map(|_| self.object.id)
    }
}

/// An object's last known information (CR 113.7a, 608.2h): the existence
/// that left (CR 400.7), its characteristics as it last existed in the zone it
/// left, and the facts about it that no layer computes. A departure record
/// carries one (CR 603.10a), and so does damage a source deals after it left.
/// A holder of a `kept_lki` that reads an object — its source, or one of its
/// triggering event's objects — keeps one when the object leaves the zone the
/// holder expected it in (`triggers-architecture.md` §6.1), since the
/// departure is a later event than the one that triggered.
///
/// One body behind one pointer: the record and every entry that keeps it
/// share it, and handing it over clones the pointer.
#[derive(Debug, Clone, PartialEq)]
pub struct Lki(Arc<LkiBody>);

#[derive(Debug, PartialEq)]
struct LkiBody {
    object: ObjectRef,
    frame: Arc<EffectiveCharacteristics>,
    /// CR 108.3. The store loses a token at CR 704.5d and an object whose
    /// owner leaves at CR 800.4a, and this is then the one place it is kept.
    owner: PlayerId,
    /// CR 707.10's decisions about the spell it was.
    cost_choices: CostChoices,
    /// `None` for an object that left a zone other than the battlefield.
    permanent: Option<PermanentFacts>,
    /// The stack object it was, for a spell or ability that left the stack: what
    /// was chosen for it and who cast it (CR 603.10e, 707.10).
    entry: Option<Arc<StackEntry>>,
}

/// What a permanent had that its characteristics do not carry: its tapped
/// status (CR 110.5), its counters (CR 122), its attachments both ways (CR
/// 301.5, 303.4) and what it was attacking (CR 506.3).
#[derive(Debug, PartialEq)]
struct PermanentFacts {
    tapped: bool,
    attacking: Option<AttackTarget>,
    /// By kind, in `CounterType`'s order.
    counters: Vec<(CounterType, u32)>,
    attached_to: Option<ObjectId>,
    attached_by: Vec<ObjectId>,
}

impl Lki {
    /// `object` as it is now, about to leave: `frame` is its characteristics,
    /// `permanent` its entry while it is on the battlefield, and `entry` its
    /// stack entry while it is on the stack.
    pub fn new(
        object: ObjectRef,
        frame: Arc<EffectiveCharacteristics>,
        owner: PlayerId,
        cost_choices: CostChoices,
        permanent: Option<&PermanentState>,
        entry: Option<&StackEntry>,
    ) -> Lki {
        let permanent = permanent.map(|entry| {
            // ORDER-FREE: sorted next.
            let mut counters: Vec<(CounterType, u32)> =
                entry.counters.iter().map(|(&kind, stack)| (kind, stack.count)).filter(|&(_, n)| n > 0).collect();
            counters.sort_unstable_by_key(|&(kind, _)| kind);
            PermanentFacts {
                tapped: entry.tapped,
                attacking: entry.attacking.as_ref().map(|attack| attack.target),
                counters,
                attached_to: entry.attached_to,
                attached_by: entry.attached_by.clone(),
            }
        });
        let entry = entry.map(|entry| Arc::new(entry.clone()));
        Lki(Arc::new(LkiBody { object, frame, owner, cost_choices, permanent, entry }))
    }

    /// The existence that left.
    pub fn object(&self) -> ObjectRef {
        self.0.object
    }

    /// The characteristics it last had.
    pub fn chars(&self) -> &EffectiveCharacteristics {
        &self.0.frame
    }

    /// The same, shared.
    pub fn frame(&self) -> &Arc<EffectiveCharacteristics> {
        &self.0.frame
    }

    pub fn owner(&self) -> PlayerId {
        self.0.owner
    }

    pub fn cost_choices(&self) -> &CostChoices {
        &self.0.cost_choices
    }

    /// Whether it was tapped; `None` for an object that was not a permanent.
    pub fn was_tapped(&self) -> Option<bool> {
        self.0.permanent.as_ref().map(|p| p.tapped)
    }

    /// How many `kind` counters it had (CR 122.8, 122.9), none for an object
    /// that was not a permanent.
    pub fn counters(&self, kind: CounterType) -> u32 {
        let Some(permanent) = &self.0.permanent else { return 0 };
        permanent.counters.iter().find(|(k, _)| *k == kind).map_or(0, |&(_, n)| n)
    }

    /// How many counters of `kind` it had, or of every kind for `None`.
    pub fn counters_of(&self, kind: Option<CounterType>) -> u32 {
        match (kind, &self.0.permanent) {
            (Some(kind), _) => self.counters(kind),
            (None, Some(permanent)) => permanent.counters.iter().map(|&(_, n)| n).sum(),
            (None, None) => 0,
        }
    }

    /// What it was attacking as it left (CR 506.4's removal comes with the
    /// move); `None` for an object that was not an attacking permanent.
    pub fn attacking(&self) -> Option<AttackTarget> {
        self.0.permanent.as_ref().and_then(|p| p.attacking)
    }

    /// What it was attached to.
    pub fn host(&self) -> Option<ObjectId> {
        self.0.permanent.as_ref().and_then(|p| p.attached_to)
    }

    /// Whether `attachment` was attached to it: an Aura's "when enchanted
    /// creature dies" asks its host's departure (CR 603.10a), since the
    /// Aura that stays was detached by then.
    pub fn had_attached(&self, attachment: ObjectId) -> bool {
        self.0.permanent.as_ref().is_some_and(|p| p.attached_by.contains(&attachment))
    }

    /// The stack object it was; `None` for an object that was not on the
    /// stack.
    pub fn entry(&self) -> Option<&StackEntry> {
        self.0.entry.as_deref()
    }
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
    /// The last known information of the objects it names that have left
    /// since it triggered.
    pub kept_lki: Vec<Lki>,
    /// CR 107.3n — a delayed trigger's X, its creating spell's or ability's,
    /// which its stack object carries; `None` for an object's trigger.
    pub x_value: Option<u64>,
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
    use crate::types::zones::ZoneChangeAct;
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
                act: ZoneChangeAct::Destroyed,
                lki: None,
                applied: None,
                new_epoch: 1,
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
                source_lki: None,
            }),
            ("PhaseBegin", GameEvent::PhaseBegin { phase: PhaseType::Precombat, player: 0 }),
            ("StepBegin", GameEvent::StepBegin { step: StepType::Upkeep, player: 0 }),
            ("TurnBegin", GameEvent::TurnBegin { player: 0, turn_number: 1 }),
            ("LifeChanged", GameEvent::LifeChanged { player_id: 0, old: 20, new: 21, source: None }),
            ("PermanentEnteredBattlefield", GameEvent::PermanentEnteredBattlefield { object_id: id, controller: 0 }),
            ("AttackersDeclared", GameEvent::AttackersDeclared { attackers: vec![(id, AttackTarget::Player(1))] }),
            ("AbilityTriggered", GameEvent::AbilityTriggered {
                seq: TriggerSeq(0),
                origin: TriggerOrigin::Object(identity),
                controller: 0,
                caused_by: EventSeq(0),
            }),
            ("SpellCast", GameEvent::SpellCast { spell_id: id, caster: 0 }),
            ("CardDrawn", GameEvent::CardDrawn { player_id: 0, card_id: id }),
            ("LibraryShuffled", GameEvent::LibraryShuffled { player_id: 0 }),
            ("AbilityResolved", GameEvent::AbilityResolved { identity, controller: 0 }),
            ("Attached", GameEvent::Attached { attachment: id, host: id, former_host: None }),
            ("Unattached", GameEvent::Unattached {
                attachment: identity.source,
                former_host: identity.source,
                route: crate::events::event::UnattachRoute::HostLeft,
            }),
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
                TriggerEvent::ZoneChange { subject: this.clone(), change: ZoneChangePattern { from: None, to: None, act: None }, by: None, owner: None, multiplicity: each },
                &["ZoneChange", "LeftTheGame"],
            ),
            (TriggerEvent::BecomesTapped { subject: this.clone() }, &["Tapped"]),
            (TriggerEvent::BecomesUntapped { subject: this.clone() }, &["Untapped"]),
            (TriggerEvent::DrawsCard { player: None, multiplicity: each, place: None }, &["CardDrawn"]),
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
            (
                TriggerEvent::Attacks { attacker: this, defender: None, per: AttackEventPer::Attacker, alone: false, multiplicity: each },
                &["AttackersDeclared"],
            ),
            (TriggerEvent::CastsSpell { caster: None, spell: None }, &["SpellCast"]),
            (TriggerEvent::ShufflesLibrary { player: None }, &["LibraryShuffled"]),
            (TriggerEvent::AbilityTriggers { caused_by: None, source: None }, &["AbilityTriggered"]),
            (TriggerEvent::AbilityResolves { identity: IdentityRef::ThisAbility }, &["AbilityResolved"]),
            (TriggerEvent::BecomesAttached { attachment: TriggerSubject::Any, host: TriggerSubject::Any }, &["Attached"]),
            (
                TriggerEvent::BecomesUnattached { attachment: TriggerSubject::Any, former_host: TriggerSubject::Any },
                &["Unattached"],
            ),
        ];
        let records = sample_records();
        for (arm, reads) in &expected {
            for (name, record) in &records {
                assert_eq!(arm.reads(record), reads.contains(name), "{arm:?} reading a {name} record");
            }
        }
    }

    /// The mask is as wide as the kinds need and no wider: one word for the
    /// nineteen there are, every kind in it, and none of them in the empty set.
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
