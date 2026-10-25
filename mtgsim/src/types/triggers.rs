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
use crate::engine::replacement::EventSubject;
use crate::events::event::{CounterSubject, CreationPoint, DamageTarget, EventRecord, EventId, GameEvent, LifeChange, ResolutionStamp};
use crate::objects::card_data::{AbilityText, CardData};
use crate::state::battlefield::{AttackTarget, CostChoices, PermanentState};
use crate::state::game_state::{AbilityIdentity, GameState, PhaseType, StackEntry, StepType};
use crate::types::effects::{Condition, CounterType, Effect, EffectRecipient, ObjectFilter, PlayerRef, TokenDef};
use crate::types::ids::{AbilityId, DelayedTriggerId, ExtraTurnId, ObjectId, ObjectRef, PlayerId};
use crate::types::history::TurnFact;
use crate::types::mana::ManaType;
use crate::types::zones::{DrawFilter, Zone, ZoneChangeAct, ZoneChangePattern};

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
    /// CR 603.8 — no event; a predicate over live state. The state check asks
    /// it (`engine::triggers::state`); the dispatcher refuses it.
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
    /// 603.8): its set is `BoardState`, which only the state check asks.
    pub fn record_kinds(&self) -> EventKindMask {
        match &self.condition {
            TriggerCondition::State(_) => EventKindMask::of(EventKind::BoardState),
            condition => condition.events().iter().fold(EventKindMask::EMPTY, |mask, arm| mask | arm.record_kinds()),
        }
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

/// CR 603.2d — "if [an event] causes a triggered ability of [a permanent] to
/// trigger, that ability triggers an additional time": the body of a static
/// ability, read off its source's effective ability list as the dispatch
/// queues what a window triggered.
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerMultiplierDef {
    /// "If an artifact or creature entering causes": the event a triggering
    /// event must be, or any of several ("entering or leaving"), asked of
    /// the record with this ability's own referents and the frames its own
    /// arm reads. `None` names no event (Roaming Throne).
    pub caused_by: Option<TriggerCondition>,
    /// "A triggered ability of a permanent you control": the permanent whose
    /// ability triggers, whose "you" is this ability's controller.
    pub abilities_of: ObjectFilter,
    /// "An additional time" is 1.
    pub additional: u32,
}

impl TriggerMultiplierDef {
    /// The static ability whose body this is, which a card file writes.
    pub fn into_ability(self, rules_text: &'static str) -> crate::objects::card_data::AbilityDef {
        crate::objects::card_data::AbilityDef {
            rules_text: rules_text.into(),
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: crate::objects::card_data::AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::TriggerMultiplier(Box::new(self)),
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            is_characteristic_defining: false,
        }
    }

    /// What the count cannot read, which the registry refuses: a state for
    /// the event, since CR 603.2d counts the triggers an event causes and a
    /// state trigger has none (CR 603.8), and no additional time.
    pub fn refusal(&self) -> Option<&'static str> {
        if matches!(self.caused_by, Some(TriggerCondition::State(_))) {
            Some("a state is no event that causes an ability to trigger")
        } else if self.additional == 0 {
            Some("it adds no time")
        } else {
            None
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
    /// event, read record by record off the history: the record's place in
    /// the turn.
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

impl Multiplicity {
    /// Whether a trigger read this way has one triggering event for "it":
    /// one per occurrence, or a subject's events, which are all about it. A
    /// "one or more" trigger has several and no single one.
    pub fn has_one_triggering_event(self) -> bool {
        match self {
            Multiplicity::PerOccurrence | Multiplicity::OncePerSubject => true,
            Multiplicity::OncePerEvent => false,
        }
    }

    /// What a counter record of `n` counters is to an arm read this way: how
    /// many events it holds, and how many counters each is. A counter is an
    /// occurrence of a per-counter arm (Protean Hydra's fourth ruling), so
    /// the record is `n` events of one; the other readings take it as one
    /// event of `n`.
    pub fn counter_occurrences(self, n: u32) -> (usize, u64) {
        match self {
            Multiplicity::PerOccurrence => (n as usize, 1),
            Multiplicity::OncePerEvent | Multiplicity::OncePerSubject => (1, u64::from(n)),
        }
    }
}

/// What a once-per-subject reading groups an arm's records by (CR 603.2c,
/// §4.4): the object or player the counters are on, or the damage's
/// recipient.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OccurrenceSubject {
    CountersOn,
    DamageRecipient,
}

/// The arms that carry CR 603.2c's multiplicity, and its field: the one
/// list, for the reading and for the authoring setter.
macro_rules! multiplicity_field {
    ($arm:expr) => {
        match $arm {
            TriggerEvent::ZoneChange { multiplicity, .. }
            | TriggerEvent::DrawsCard { multiplicity, .. }
            | TriggerEvent::DamageDealt { multiplicity, .. }
            | TriggerEvent::DamageIsPrevented { multiplicity, .. }
            | TriggerEvent::GainsLife { multiplicity, .. }
            | TriggerEvent::LosesLife { multiplicity, .. }
            | TriggerEvent::EntersBattlefield { multiplicity, .. }
            | TriggerEvent::Attacks { multiplicity, .. }
            | TriggerEvent::Blocks { multiplicity, .. }
            | TriggerEvent::BecomesTarget { multiplicity, .. }
            | TriggerEvent::CountersPutOn { multiplicity, .. }
            | TriggerEvent::CountersRemovedFrom { multiplicity, .. }
            | TriggerEvent::CreatesToken { multiplicity, .. } => Some(multiplicity),
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
            | TriggerEvent::ActivatesAbility { .. }
            | TriggerEvent::AbilityTriggers { .. }
            | TriggerEvent::AbilityResolves { .. }
            | TriggerEvent::Scries { .. } => None,
        }
    };
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

impl TriggerSubject {
    /// Whether it names one object, not any of a kind: this object, its host,
    /// or the object a delayed trigger refers to.
    pub fn names_one_object(&self) -> bool {
        match self {
            TriggerSubject::ThisObject | TriggerSubject::Host | TriggerSubject::Referred => true,
            TriggerSubject::Filter(_) | TriggerSubject::Any => false,
        }
    }
}

/// What becomes the target (CR 115.1): an object, or a player.
#[derive(Debug, Clone, PartialEq)]
pub enum TargetSubject {
    /// "This creature", "a permanent you control".
    Object(TriggerSubject),
    /// "You", or any player for `None`.
    Player(Option<PlayerRef>),
}

/// What it becomes the target of: "a spell or ability", "a spell", "an Aura
/// spell", "an ability" (CR 115.1a-d).
#[derive(Debug, Clone, PartialEq)]
pub enum TargetedBy {
    SpellOrAbility,
    /// A spell, of the filter's kind when it has one, read on the stack.
    Spell(Option<ObjectFilter>),
    /// An activated or triggered ability.
    Ability,
}

impl TargetedBy {
    /// Whether a spell (`by_spell`) or an ability is what it asks for. A
    /// spell's kind, where it names one, is read on the stack apart.
    pub fn admits(&self, by_spell: bool) -> bool {
        match self {
            TargetedBy::SpellOrAbility => true,
            TargetedBy::Spell(_) => by_spell,
            TargetedBy::Ability => !by_spell,
        }
    }
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

    /// Whether a planeswalker or a battle, an object, can be what it names.
    pub fn admits_a_permanent(&self) -> bool {
        match self {
            DefenderFilter::Player(_) => false,
            DefenderFilter::Permanent(_) => true,
            DefenderFilter::Or(either, or) => either.admits_a_permanent() || or.admits_a_permanent(),
        }
    }
}

/// What one occurrence of an attack trigger is (CR 603.2c): each creature
/// declared as an attacker (CR 508.3a, 508.3c, 508.3d), or each player,
/// planeswalker or battle it is declared attacking (CR 508.3b, 508.3e).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackOccurrence {
    Attacker,
    Defender,
}

/// What one occurrence of a block trigger is (CR 603.2c), each CR 509.3
/// reading's "triggers once for each".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockOccurrence {
    /// CR 509.3a: each blocking creature, however many it blocks.
    Blocker,
    /// CR 509.3b, 509.3d: each blocking creature and each creature it blocks.
    Block,
    /// CR 509.3c: each blocked creature, however many block it.
    Blocked,
    /// CR 509.3g: each attacking creature no creature was declared blocking.
    Unblocked,
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
    /// CR 506.4's attacking creature: one declared as an attacker (CR 508.1),
    /// or, in a block, one that is attacking however it came to.
    Attacker,
    /// CR 509.1g: a creature declared as a blocker.
    Blocker,
    /// "Whenever this creature blocks or becomes blocked by a creature,
    /// destroy that creature": the creature on the other side of a block
    /// from the ability's own source, the attacker it blocks or the blocker
    /// blocking it, which the arm that matched says.
    OtherCombatant,
    /// CR 506.3: the planeswalker or battle a creature was declared
    /// attacking. A player attacked is no object.
    Attacked,
    /// CR 701.3: the Aura, Equipment or Fortification.
    Attachment,
    /// CR 701.3: the object it became attached to, or was attached to.
    Host,
    /// CR 115.1: the object that became the target.
    Target,
    /// CR 115.1: the spell or ability it became the target of, as the stack
    /// object it is: Frost Titan's "counter that spell or ability".
    SpellOrAbility,
}

impl EventRole {
    pub const ALL: [EventRole; 11] = [
        EventRole::Object,
        EventRole::DamageSource,
        EventRole::DamageRecipient,
        EventRole::Attacker,
        EventRole::Blocker,
        EventRole::OtherCombatant,
        EventRole::Attacked,
        EventRole::Attachment,
        EventRole::Host,
        EventRole::Target,
        EventRole::SpellOrAbility,
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
    /// CR 615.13: the object whose damage a prevention effect prevented. The
    /// record names no source, since the damage can come from several.
    Prevention { recipient: ObjectRef },
    /// CR 508.1: one creature declared as an attacker, and what it attacks.
    Attack { attacker: ObjectRef, defender: ObjectOrPlayer },
    /// CR 508.3b: one player, planeswalker or battle attacked, and every
    /// creature declared attacking it.
    Defense { defender: ObjectOrPlayer, attackers: Vec<ObjectRef> },
    /// CR 701.3: an attachment and the object it became, or was, attached to.
    Attach { attachment: ObjectRef, host: ObjectRef },
    /// CR 509.3b, 509.3d: one block, its blocker and the attacker it blocks.
    Block { blocker: ObjectRef, attacker: ObjectRef },
    /// CR 509.3a: one blocking creature, and the attackers it blocks that
    /// the arm counts.
    Blocking { blocker: ObjectRef, attackers: Vec<ObjectRef> },
    /// CR 509.3c: one blocked creature, and the blockers blocking it that
    /// the arm counts.
    BlockedBy { attacker: ObjectRef, blockers: Vec<ObjectRef> },
    /// CR 509.3g: an attacking creature no creature was declared blocking.
    Unblocked { attacker: ObjectRef },
    /// CR 115.1: what became the target, and the spell or ability whose
    /// target it became.
    Target { target: ObjectOrPlayer, by: ObjectRef },
}

impl TriggeringEventObjects {
    /// The object that played `role`, if this event has exactly one:
    /// a defender's attackers are several, and [`Self::objects`] has them.
    pub fn get(&self, role: EventRole) -> Option<ObjectRef> {
        self.pairs().0.into_iter().flatten().find(|&(part, _)| part == role).map(|(_, object)| object)
    }

    /// Each object, with the part it played.
    pub fn objects(&self) -> impl Iterator<Item = (EventRole, ObjectRef)> + '_ {
        let (pairs, (role, several)) = self.pairs();
        pairs.into_iter().flatten().chain(several.iter().map(move |&object| (role, object)))
    }

    /// Each variant's parts, the one table of them: up to two objects with
    /// one part each, then the several of one part (a defender's attackers,
    /// a blocker's attackers, a blocked creature's blockers), which
    /// [`Self::get`] does not name.
    fn pairs(&self) -> ([Option<(EventRole, ObjectRef)>; 2], (EventRole, &[ObjectRef])) {
        match self {
            TriggeringEventObjects::NoObject => ([None, None], (EventRole::Attacker, &[])),
            TriggeringEventObjects::Object(object) => ([Some((EventRole::Object, *object)), None], (EventRole::Attacker, &[])),
            TriggeringEventObjects::Damage { source, recipient } => (
                [Some((EventRole::DamageSource, *source)), recipient.object().map(|o| (EventRole::DamageRecipient, o))],
                (EventRole::Attacker, &[]),
            ),
            TriggeringEventObjects::Prevention { recipient } => {
                ([Some((EventRole::DamageRecipient, *recipient)), None], (EventRole::Attacker, &[]))
            }
            TriggeringEventObjects::Attack { attacker, defender } => (
                [Some((EventRole::Attacker, *attacker)), defender.object().map(|o| (EventRole::Attacked, o))],
                (EventRole::Attacker, &[]),
            ),
            TriggeringEventObjects::Defense { defender, attackers } => {
                ([defender.object().map(|o| (EventRole::Attacked, o)), None], (EventRole::Attacker, attackers))
            }
            TriggeringEventObjects::Attach { attachment, host } => (
                [Some((EventRole::Attachment, *attachment)), Some((EventRole::Host, *host))],
                (EventRole::Attacker, &[]),
            ),
            TriggeringEventObjects::Block { blocker, attacker } => (
                [Some((EventRole::Blocker, *blocker)), Some((EventRole::Attacker, *attacker))],
                (EventRole::Attacker, &[]),
            ),
            TriggeringEventObjects::Blocking { blocker, attackers } => {
                ([Some((EventRole::Blocker, *blocker)), None], (EventRole::Attacker, attackers))
            }
            TriggeringEventObjects::BlockedBy { attacker, blockers } => {
                ([Some((EventRole::Attacker, *attacker)), None], (EventRole::Blocker, blockers))
            }
            TriggeringEventObjects::Unblocked { attacker } => {
                ([Some((EventRole::Attacker, *attacker)), None], (EventRole::Attacker, &[]))
            }
            TriggeringEventObjects::Target { target, by } => (
                [target.object().map(|o| (EventRole::Target, o)), Some((EventRole::SpellOrAbility, *by))],
                (EventRole::Attacker, &[]),
            ),
        }
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
    /// can (`AbilityCountered`, `TokenCeasedToExist`, ...).
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
        DamagePrevented,
        PhaseBegin,
        StepBegin,
        TurnBegin,
        LifeChanged,
        EnteredBattlefield,
        AttackersDeclared,
        BlockersDeclared,
        BecameBlocked,
        Targeted,
        AbilityActivated,
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
        TokenCreated,
        Scried,
        /// No record: a state trigger's condition (CR 603.8), which reads the
        /// board rather than an event. A state trigger's source is filed under
        /// it, so the state check finds it on every leg the dispatcher's gate
        /// has, and no window, whose kinds are its records', ever asks it.
        BoardState,
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
            GameEvent::DamagePrevented { .. } => EventKind::DamagePrevented,
            GameEvent::PhaseBegin { .. } => EventKind::PhaseBegin,
            GameEvent::StepBegin { .. } => EventKind::StepBegin,
            GameEvent::TurnBegin { .. } => EventKind::TurnBegin,
            GameEvent::LifeChanged { .. } => EventKind::LifeChanged,
            GameEvent::PermanentEnteredBattlefield { .. } => EventKind::EnteredBattlefield,
            GameEvent::AttackersDeclared { .. } => EventKind::AttackersDeclared,
            GameEvent::BlockersDeclared { .. } => EventKind::BlockersDeclared,
            GameEvent::BecameBlocked { .. } => EventKind::BecameBlocked,
            GameEvent::Targeted { .. } => EventKind::Targeted,
            GameEvent::AbilityActivated { .. } => EventKind::AbilityActivated,
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
            GameEvent::TokenCreated { .. } => EventKind::TokenCreated,
            GameEvent::Scried { .. } => EventKind::Scried,
            GameEvent::AbilityCountered { .. }
            | GameEvent::AbilityFizzled { .. }
            | GameEvent::DelayedTriggerCreated { .. }
            | GameEvent::PlayerWon { .. }
            | GameEvent::GameEndedInADraw
            | GameEvent::RemovedFromCombat { .. }
            | GameEvent::SpellFizzled { .. }
            | GameEvent::StateBasedActionPerformed
            | GameEvent::TokenCeasedToExist { .. }
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
        /// "Is dealt excess damage" (CR 120.10): only damage past lethal, and
        /// "that much" is the excess. Read once per permanent, whose records'
        /// excess is the event's, so the registry refuses it read per record
        /// or of a player.
        excess: bool,
        multiplicity: Multiplicity,
    },
    /// "Whenever damage that would be dealt to you is prevented" (CR 615.13):
    /// once each time a prevention effect is applied to one recipient's
    /// damage in a batch, however many sources it came from. "That many" is
    /// the damage that application prevented.
    DamageIsPrevented { recipient: DamageRecipient, multiplicity: Multiplicity },
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
        occurrence: AttackOccurrence,
        /// CR 506.5: the only creature declared as an attacker.
        alone: bool,
        multiplicity: Multiplicity,
    },
    /// CR 509.3a–g, read off the declaration of blockers (CR 509.1) at its
    /// dispatch, which is CR 509.3f's moment. `blocker` and `attacker` are
    /// the two sides' filters. Read once for each blocking creature or each
    /// blocked one, the other side's filter picks which of its creatures
    /// count, and the occurrence needs one of them, or `count`'s many (CR
    /// 509.3e).
    Blocks {
        blocker: TriggerSubject,
        attacker: TriggerSubject,
        occurrence: BlockOccurrence,
        /// CR 509.3e: how many of the other side's creatures match.
        count: Option<crate::types::history::CountIs>,
        /// CR 506.5: the only creature declared as a blocker, across every
        /// defending player (CR 802.4).
        alone: bool,
        multiplicity: Multiplicity,
    },
    /// "Whenever [this creature/you] become[s] the target of a [spell or
    /// ability] [an opponent controls]" (CR 115.1, 601.2c, 602.2b, 603.3d):
    /// once for each spell or ability, however many of its instances chose it
    /// (Frost Titan's ruling). `controller` is whose spell or ability it is.
    /// "For the first time each turn" (valiant) is the def's limit, read off
    /// the turn's targeting rows.
    BecomesTarget { target: TargetSubject, by: TargetedBy, controller: Option<PlayerRef>, multiplicity: Multiplicity },
    /// "Whenever [a player] activates an ability of [a permanent] [that isn't
    /// a mana ability]" (CR 602.2), off the record its costs announce: a
    /// mana ability's included (CR 605.3a), which Harsh Mentor's intervening
    /// "if it isn't a mana ability" excludes as `mana_ability: Some(false)`.
    /// The source is read as it was on the battlefield (Harsh Mentor's second
    /// ruling): off its frame when a cost moved it. Loyalty abilities are
    /// `remaining-costs`'s, and the arm asks nothing of them yet.
    ActivatesAbility { player: Option<PlayerRef>, source: TriggerSubject, mana_ability: Option<bool> },
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
    /// "Whenever you create one or more tokens" (CR 111.2, 701.7a): one
    /// record per token, so "one or more" is `OncePerEvent` and "that many"
    /// is one a record. A reflexive "when you do" after "create a [token]"
    /// names its definition, so a creation a replacement changed to another
    /// is not "that action" (Ajani, Nacatl Avenger's eighth ruling). A copy
    /// of a permanent spell is no creation (CR 111.13) and writes no record.
    CreatesToken { owner: Option<PlayerRef>, definition: Option<Box<TokenDef>>, multiplicity: Multiplicity },
    /// "Whenever you scry" (CR 701.22): one record per scry, and none for a
    /// scry 0 (CR 701.22b). "The number of cards looked at while scrying this
    /// way" is the record's `looked_at` (Elrond, Master of Healing's first
    /// ruling).
    Scries { player: Option<PlayerRef> },
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
    ///
    /// An arm that names both zones is the one move, and asks
    /// `zone_change_looks_back` of it. One that leaves a zone unnamed is
    /// lifted on purpose: "into a hand" from anywhere looks back at a move
    /// from a zone all players could see, and "from anywhere" is no
    /// leaves-the-battlefield ability (CR 603.6c).
    pub fn looks_back(&self) -> bool {
        match self {
            TriggerEvent::ZoneChange { change: ZoneChangePattern { from: Some(from), to: Some(to), .. }, .. } => {
                zone_change_looks_back(*from, Some(*to))
            }
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
            | TriggerEvent::DamageIsPrevented { .. }
            | TriggerEvent::PhaseBegins { .. }
            | TriggerEvent::StepBegins { .. }
            | TriggerEvent::TurnBegins { .. }
            | TriggerEvent::GainsLife { .. }
            | TriggerEvent::LosesLife { .. }
            | TriggerEvent::EntersBattlefield { .. }
            | TriggerEvent::Attacks { .. }
            | TriggerEvent::Blocks { .. }
            | TriggerEvent::BecomesTarget { .. }
            | TriggerEvent::ActivatesAbility { .. }
            | TriggerEvent::CastsSpell { .. }
            | TriggerEvent::ShufflesLibrary { .. }
            | TriggerEvent::CountersPutOn { .. }
            | TriggerEvent::CountersRemovedFrom { .. }
            | TriggerEvent::BecomesAttached { .. }
            | TriggerEvent::AbilityTriggers { .. }
            | TriggerEvent::AbilityResolves { .. }
            | TriggerEvent::CreatesToken { .. }
            | TriggerEvent::Scries { .. } => false,
        }
    }

    /// CR 603.6e's trigger: an Aura's (or any attachment's) on the permanent
    /// it is attached to leaving the battlefield, the one CR 400.7f lets find
    /// the Aura's new object in a graveyard.
    pub fn watches_its_host_leave(&self) -> bool {
        matches!(self, TriggerEvent::ZoneChange { subject: TriggerSubject::Host, change, .. } if change.from == Some(Zone::Battlefield))
    }

    /// The act a zone-change arm names: "whenever you sacrifice". CR 701
    /// defines a keyword action as the move it makes (CR 701.8a, 701.21a), so
    /// the act is written on the zone change's record and pattern and on no
    /// other arm. CR 603.2d's count tells "whenever you sacrifice" from
    /// "whenever a creature dies" by it (Teysa Karlov's fifth ruling).
    pub fn named_act(&self) -> Option<ZoneChangeAct> {
        if let TriggerEvent::ZoneChange { change, .. } = self { change.act } else { None }
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
            TriggerEvent::DamageIsPrevented { .. } => EventKindMask::of(EventKind::DamagePrevented),
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
            // CR 509.3c's "becomes blocked" by an effect as well as by a
            // declaration.
            TriggerEvent::Blocks { .. } => EventKindMask::of(EventKind::BlockersDeclared).with(EventKind::BecameBlocked),
            TriggerEvent::BecomesTarget { .. } => EventKindMask::of(EventKind::Targeted),
            TriggerEvent::ActivatesAbility { .. } => EventKindMask::of(EventKind::AbilityActivated),
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
            TriggerEvent::CreatesToken { .. } => EventKindMask::of(EventKind::TokenCreated),
            TriggerEvent::Scries { .. } => EventKindMask::of(EventKind::Scried),
        }
    }

    /// The fact whose records a "first time each turn" on this arm counts its
    /// record's place among, the player's own this turn (§3.5): a cast, a
    /// draw, a gain or a loss of life (CR 119.9 makes each its own event).
    /// `None` for an arm no record of which has a place. The history's
    /// writer places exactly these (`state::history::PLACED_FACTS`), and the
    /// registry admits "the first time" only on them or a targeting.
    pub fn placed_fact(&self) -> Option<TurnFact> {
        match self {
            TriggerEvent::CastsSpell { .. } => Some(TurnFact::SpellsCast),
            TriggerEvent::DrawsCard { .. } => Some(TurnFact::CardsDrawn),
            TriggerEvent::GainsLife { .. } => Some(TurnFact::LifeGainEvents),
            TriggerEvent::LosesLife { .. } => Some(TurnFact::LifeLossEvents),
            TriggerEvent::ZoneChange { .. }
            | TriggerEvent::BecomesTapped { .. }
            | TriggerEvent::BecomesUntapped { .. }
            | TriggerEvent::ManaAdded { .. }
            | TriggerEvent::DamageDealt { .. }
            | TriggerEvent::DamageIsPrevented { .. }
            | TriggerEvent::PhaseBegins { .. }
            | TriggerEvent::StepBegins { .. }
            | TriggerEvent::TurnBegins { .. }
            | TriggerEvent::EntersBattlefield { .. }
            | TriggerEvent::Attacks { .. }
            | TriggerEvent::Blocks { .. }
            | TriggerEvent::BecomesTarget { .. }
            | TriggerEvent::ActivatesAbility { .. }
            | TriggerEvent::IsCountered { .. }
            | TriggerEvent::PlayerLoses { .. }
            | TriggerEvent::ShufflesLibrary { .. }
            | TriggerEvent::CountersPutOn { .. }
            | TriggerEvent::CountersRemovedFrom { .. }
            | TriggerEvent::BecomesAttached { .. }
            | TriggerEvent::BecomesUnattached { .. }
            | TriggerEvent::ControlChanges { .. }
            | TriggerEvent::AbilityTriggers { .. }
            | TriggerEvent::AbilityResolves { .. }
            | TriggerEvent::CreatesToken { .. }
            | TriggerEvent::Scries { .. } => None,
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
            // "Becomes blocked by a creature" is about the attacker it names
            // (CR 509.3d); every other block is about its blocker.
            TriggerEvent::Blocks { occurrence: BlockOccurrence::Block, blocker, attacker, .. }
                if attacker.names_one_object() && !blocker.names_one_object() =>
            {
                Some(attacker)
            }
            TriggerEvent::ZoneChange { subject, .. }
            | TriggerEvent::BecomesTapped { subject }
            | TriggerEvent::BecomesUntapped { subject }
            | TriggerEvent::EntersBattlefield { subject, .. }
            | TriggerEvent::ManaAdded { source: subject, .. }
            | TriggerEvent::DamageDealt { source: subject, .. }
            | TriggerEvent::Attacks { attacker: subject, .. }
            | TriggerEvent::Blocks { occurrence: BlockOccurrence::Blocked | BlockOccurrence::Unblocked, attacker: subject, .. }
            | TriggerEvent::Blocks { occurrence: BlockOccurrence::Blocker | BlockOccurrence::Block, blocker: subject, .. }
            | TriggerEvent::BecomesTarget { target: TargetSubject::Object(subject), .. }
            | TriggerEvent::ActivatesAbility { source: subject, .. }
            | TriggerEvent::BecomesAttached { host: subject, .. }
            | TriggerEvent::BecomesUnattached { former_host: subject, .. }
            | TriggerEvent::ControlChanges { subject, .. }
            | TriggerEvent::CountersPutOn { subject: CountersOn::Object(subject), .. }
            | TriggerEvent::CountersRemovedFrom { subject: CountersOn::Object(subject), .. } => Some(subject),
            TriggerEvent::CountersPutOn { subject: CountersOn::Player(_), .. }
            | TriggerEvent::CountersRemovedFrom { subject: CountersOn::Player(_), .. }
            | TriggerEvent::BecomesTarget { target: TargetSubject::Player(_), .. }
            | TriggerEvent::DamageIsPrevented { .. }
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
            | TriggerEvent::AbilityResolves { .. }
            | TriggerEvent::CreatesToken { .. }
            | TriggerEvent::Scries { .. } => None,
        }
    }

    /// Whether this arm may be read once per subject: whether one event can
    /// make several of its records about one object or player, so that
    /// grouping them by it differs from reading each. Counters put on one
    /// creature by two players, or of two kinds, are two records, and so is
    /// damage to one creature from two sources, which "is dealt excess damage"
    /// reads once ("one or more creatures deal combat damage to a player"
    /// will too, `triggers-architecture.md` §4.4). A death is one record per
    /// object, so per subject would be per record.
    pub fn supports_once_per_subject(&self) -> bool {
        self.occurrence_subject_part().is_some()
    }

    /// The part of this arm's records a once-per-subject reading groups by.
    fn occurrence_subject_part(&self) -> Option<OccurrenceSubject> {
        match self {
            TriggerEvent::CountersPutOn { .. } | TriggerEvent::CountersRemovedFrom { .. } => Some(OccurrenceSubject::CountersOn),
            TriggerEvent::DamageDealt { .. } => Some(OccurrenceSubject::DamageRecipient),
            TriggerEvent::ZoneChange { .. }
            | TriggerEvent::BecomesTapped { .. }
            | TriggerEvent::BecomesUntapped { .. }
            | TriggerEvent::DrawsCard { .. }
            | TriggerEvent::ManaAdded { .. }
            | TriggerEvent::DamageIsPrevented { .. }
            | TriggerEvent::PhaseBegins { .. }
            | TriggerEvent::StepBegins { .. }
            | TriggerEvent::TurnBegins { .. }
            | TriggerEvent::GainsLife { .. }
            | TriggerEvent::LosesLife { .. }
            | TriggerEvent::EntersBattlefield { .. }
            | TriggerEvent::Attacks { .. }
            | TriggerEvent::Blocks { .. }
            | TriggerEvent::BecomesTarget { .. }
            | TriggerEvent::ActivatesAbility { .. }
            | TriggerEvent::CastsSpell { .. }
            | TriggerEvent::IsCountered { .. }
            | TriggerEvent::PlayerLoses { .. }
            | TriggerEvent::ShufflesLibrary { .. }
            | TriggerEvent::BecomesAttached { .. }
            | TriggerEvent::BecomesUnattached { .. }
            | TriggerEvent::ControlChanges { .. }
            | TriggerEvent::AbilityTriggers { .. }
            | TriggerEvent::AbilityResolves { .. }
            | TriggerEvent::CreatesToken { .. }
            | TriggerEvent::Scries { .. } => None,
        }
    }

    /// The object or player `record` is about, for an arm that may be read
    /// once per subject (`supports_once_per_subject`): what a once-per-subject
    /// trigger is one of, the counters' object or player, or the damage's
    /// recipient. `None` for every other arm, and for a record it does not
    /// read.
    pub(crate) fn occurrence_subject(&self, record: &GameEvent) -> Option<EventSubject> {
        match (self.occurrence_subject_part()?, record) {
            (OccurrenceSubject::CountersOn, GameEvent::CountersPutOn { subject, .. } | GameEvent::CountersRemovedFrom { subject, .. }) => {
                Some(EventSubject::from(*subject))
            }
            (OccurrenceSubject::DamageRecipient, GameEvent::DamageDealt { target, .. }) => Some(EventSubject::from(*target)),
            (OccurrenceSubject::CountersOn | OccurrenceSubject::DamageRecipient, _) => None,
        }
    }

    /// Whether the arm watches an object a delayed trigger refers to (CR
    /// 603.7c): "when that token dies". The creating resolution remembers
    /// the objects for this field alone.
    pub fn watches_referred(&self) -> bool {
        self.subject() == Some(&TriggerSubject::Referred)
    }

    /// Whether any field of the arm, its subject or another, names an
    /// object a delayed trigger refers to; a `caused_by` arm's too.
    pub fn names_referred(&self) -> bool {
        let fields: [Option<&TriggerSubject>; 2] = match self {
            TriggerEvent::ZoneChange { subject, .. }
            | TriggerEvent::BecomesTapped { subject }
            | TriggerEvent::BecomesUntapped { subject }
            | TriggerEvent::ManaAdded { source: subject, .. }
            | TriggerEvent::DamageDealt { source: subject, .. }
            | TriggerEvent::EntersBattlefield { subject, .. }
            | TriggerEvent::Attacks { attacker: subject, .. }
            | TriggerEvent::BecomesTarget { target: TargetSubject::Object(subject), .. }
            | TriggerEvent::ActivatesAbility { source: subject, .. }
            | TriggerEvent::ControlChanges { subject, .. }
            | TriggerEvent::CountersPutOn { subject: CountersOn::Object(subject), .. }
            | TriggerEvent::CountersRemovedFrom { subject: CountersOn::Object(subject), .. } => [Some(subject), None],
            TriggerEvent::Blocks { blocker: one, attacker: other, .. }
            | TriggerEvent::BecomesAttached { attachment: one, host: other }
            | TriggerEvent::BecomesUnattached { attachment: one, former_host: other } => [Some(one), Some(other)],
            TriggerEvent::AbilityTriggers { caused_by, .. } => return caused_by.as_deref().is_some_and(TriggerEvent::names_referred),
            TriggerEvent::BecomesTarget { target: TargetSubject::Player(_), .. }
            | TriggerEvent::CountersPutOn { subject: CountersOn::Player(_), .. }
            | TriggerEvent::CountersRemovedFrom { subject: CountersOn::Player(_), .. }
            | TriggerEvent::DrawsCard { .. }
            | TriggerEvent::DamageIsPrevented { .. }
            | TriggerEvent::PhaseBegins { .. }
            | TriggerEvent::StepBegins { .. }
            | TriggerEvent::TurnBegins { .. }
            | TriggerEvent::GainsLife { .. }
            | TriggerEvent::LosesLife { .. }
            | TriggerEvent::CastsSpell { .. }
            | TriggerEvent::IsCountered { .. }
            | TriggerEvent::PlayerLoses { .. }
            | TriggerEvent::ShufflesLibrary { .. }
            | TriggerEvent::AbilityResolves { .. }
            | TriggerEvent::CreatesToken { .. }
            | TriggerEvent::Scries { .. } => [None, None],
        };
        fields.into_iter().flatten().any(|subject| *subject == TriggerSubject::Referred)
    }

    /// How the arm counts its occurrences (CR 603.2c): its multiplicity
    /// field, and per occurrence for an arm that has none.
    pub fn multiplicity(&self) -> Multiplicity {
        multiplicity_field!(self).copied().unwrap_or(Multiplicity::PerOccurrence)
    }

    /// The arm's multiplicity field, to set; `None` for an arm that has none.
    pub fn multiplicity_mut(&mut self) -> Option<&mut Multiplicity> {
        multiplicity_field!(self)
    }

    /// Whether one triggering event of this arm may have several objects in
    /// `role`'s part: a defender's attackers (CR 508.3b). An
    /// instruction that needs exactly one, "its power", cannot name it.
    pub fn may_have_several(&self, role: EventRole) -> bool {
        let several = match self {
            TriggerEvent::Blocks { occurrence: BlockOccurrence::Blocker, .. } => EventRole::Attacker,
            TriggerEvent::Blocks { occurrence: BlockOccurrence::Blocked, .. } => EventRole::Blocker,
            TriggerEvent::Attacks { occurrence: AttackOccurrence::Defender, .. } => EventRole::Attacker,
            _ => return false,
        };
        self.role_as_recorded(role) == several
    }

    /// The event role an instruction's `role` is stored under in this arm's
    /// triggering events. They are the same but for "that creature" of
    /// "blocks or becomes blocked by", which no event stores: it is the
    /// attacker on one arm and the blocker on the other (`other_combatant`).
    pub fn role_as_recorded(&self, role: EventRole) -> EventRole {
        match role {
            EventRole::OtherCombatant => self.other_combatant().unwrap_or(role),
            _ => role,
        }
    }

    /// The role of the creature across a block from the ability's own source
    /// (CR 509.1): the attacker on an arm whose blocker is the source, the
    /// blocker on one whose attacker is. `None` for an unblocked creature,
    /// which has none, and for every other arm.
    fn other_combatant(&self) -> Option<EventRole> {
        match self {
            TriggerEvent::Blocks { occurrence: BlockOccurrence::Unblocked, .. } => None,
            TriggerEvent::Blocks { blocker: TriggerSubject::ThisObject, .. } => Some(EventRole::Attacker),
            TriggerEvent::Blocks { attacker: TriggerSubject::ThisObject, .. } => Some(EventRole::Blocker),
            _ => None,
        }
    }

    /// The parts an object can play in this arm's triggering events: what an
    /// instruction may name, which a registry test checks every registered
    /// card against. Read off the arm's fields, so an arm that watches a
    /// player declares no object in the player's place: damage to a player
    /// has no recipient object (CR 120.1), and a player attacked is no object
    /// attacked (CR 506.3).
    pub fn event_roles(&self) -> &'static [EventRole] {
        match self {
            TriggerEvent::DamageDealt { recipient: DamageRecipient::Player(_), .. } => &[EventRole::DamageSource],
            TriggerEvent::DamageIsPrevented { recipient: DamageRecipient::Player(_), .. } => &[],
            TriggerEvent::Attacks { defender: Some(defender), .. } if !defender.admits_a_permanent() => &[EventRole::Attacker],
            TriggerEvent::ZoneChange { .. }
            | TriggerEvent::BecomesTapped { .. }
            | TriggerEvent::BecomesUntapped { .. }
            | TriggerEvent::DrawsCard { .. }
            | TriggerEvent::ManaAdded { .. }
            | TriggerEvent::EntersBattlefield { .. }
            | TriggerEvent::CastsSpell { .. }
            | TriggerEvent::IsCountered { .. }
            | TriggerEvent::ControlChanges { .. }
            | TriggerEvent::ActivatesAbility { .. }
            | TriggerEvent::CountersPutOn { subject: CountersOn::Object(_), .. }
            | TriggerEvent::CountersRemovedFrom { subject: CountersOn::Object(_), .. }
            | TriggerEvent::CreatesToken { .. } => &[EventRole::Object],
            TriggerEvent::DamageDealt { .. } => &[EventRole::DamageSource, EventRole::DamageRecipient],
            TriggerEvent::DamageIsPrevented { .. } => &[EventRole::DamageRecipient],
            TriggerEvent::Attacks { .. } => &[EventRole::Attacker, EventRole::Attacked],
            TriggerEvent::Blocks { occurrence: BlockOccurrence::Unblocked, .. } => &[EventRole::Attacker],
            TriggerEvent::Blocks { .. } if self.other_combatant().is_some() => {
                &[EventRole::Blocker, EventRole::Attacker, EventRole::OtherCombatant]
            }
            TriggerEvent::Blocks { .. } => &[EventRole::Blocker, EventRole::Attacker],
            TriggerEvent::BecomesTarget { target: TargetSubject::Object(_), .. } => &[EventRole::Target, EventRole::SpellOrAbility],
            TriggerEvent::BecomesTarget { target: TargetSubject::Player(_), .. } => &[EventRole::SpellOrAbility],
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
            | TriggerEvent::AbilityResolves { .. }
            | TriggerEvent::Scries { .. } => &[],
        }
    }

    /// "That player" (§3.4). `None` for an event about no player.
    pub fn player_of(&self, record: &GameEvent) -> Option<PlayerId> {
        match (self, record) {
            (TriggerEvent::ZoneChange { .. }, GameEvent::ZoneChange { owner, .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::LeftTheGame { owner, .. }) => Some(*owner),
            (TriggerEvent::ZoneChange { .. }, GameEvent::SameZoneMove { lki, .. }) => Some(lki.owner()),
            (TriggerEvent::ManaAdded { .. }, GameEvent::ManaAdded { player_id, .. }) => Some(*player_id),
            (TriggerEvent::DamageDealt { .. }, GameEvent::DamageDealt { target, .. })
            | (TriggerEvent::DamageIsPrevented { .. }, GameEvent::DamagePrevented { target, .. }) => match target {
                DamageTarget::Player(pid) => Some(*pid),
                DamageTarget::Object(_) => None,
            },
            (TriggerEvent::PhaseBegins { .. }, GameEvent::PhaseBegin { player, .. })
            | (TriggerEvent::StepBegins { .. }, GameEvent::StepBegin { player, .. })
            | (TriggerEvent::TurnBegins { .. }, GameEvent::TurnBegin { player, .. }) => Some(*player),
            (TriggerEvent::GainsLife { .. }, GameEvent::LifeChanged { player_id, .. })
            | (TriggerEvent::LosesLife { .. }, GameEvent::LifeChanged { player_id, .. }) => Some(*player_id),
            (TriggerEvent::CastsSpell { .. }, GameEvent::SpellCast { caster, .. }) => Some(*caster),
            // Whose spell or ability it is: ward's "that player", Frost
            // Titan's "its controller".
            (TriggerEvent::BecomesTarget { .. }, GameEvent::Targeted { controller, .. }) => Some(*controller),
            // Who activated it: Harsh Mentor's "that player".
            (TriggerEvent::ActivatesAbility { .. }, GameEvent::AbilityActivated { controller, .. }) => Some(*controller),
            // The player who gained control.
            (TriggerEvent::ControlChanges { .. }, GameEvent::ControlChanged { to, .. }) => Some(*to),
            // Its controller as it left the stack, as its frame keeps it.
            (TriggerEvent::IsCountered { .. }, GameEvent::SpellCountered { lki, .. }) => Some(lki.characteristics().controller),
            (TriggerEvent::DrawsCard { .. }, GameEvent::CardDrawn { player_id, .. })
            | (TriggerEvent::ShufflesLibrary { .. }, GameEvent::LibraryShuffled { player_id })
            | (TriggerEvent::PlayerLoses { .. }, GameEvent::PlayerLost { player_id }) => Some(*player_id),
            (TriggerEvent::EntersBattlefield { .. }, GameEvent::PermanentEnteredBattlefield { controller, .. }) => {
                Some(*controller)
            }
            // The player who got them; counters on an object name no player.
            (TriggerEvent::CountersPutOn { .. }, GameEvent::CountersPutOn { subject, .. })
            | (TriggerEvent::CountersRemovedFrom { .. }, GameEvent::CountersRemovedFrom { subject, .. }) => match subject {
                CounterSubject::Player(player) => Some(*player),
                CounterSubject::Object(_) => None,
            },
            (TriggerEvent::AbilityTriggers { .. }, GameEvent::AbilityTriggered { controller, .. })
            | (TriggerEvent::AbilityResolves { .. }, GameEvent::AbilityResolved { controller, .. }) => {
                Some(*controller)
            }
            // Who created it, its owner (CR 111.2); who scried.
            (TriggerEvent::CreatesToken { .. }, GameEvent::TokenCreated { owner, .. }) => Some(*owner),
            (TriggerEvent::Scries { .. }, GameEvent::Scried { player_id, .. }) => Some(*player_id),
            (TriggerEvent::BecomesTapped { .. }, GameEvent::Tapped { .. })
            | (TriggerEvent::BecomesUntapped { .. }, GameEvent::Untapped { .. })
            | (TriggerEvent::BecomesAttached { .. }, GameEvent::Attached { .. })
            | (TriggerEvent::BecomesUnattached { .. }, GameEvent::Unattached { .. })
            | (TriggerEvent::ControlChanges { .. }, GameEvent::ZoneChange { .. })
            | (TriggerEvent::ControlChanges { .. }, GameEvent::LeftTheGame { .. })
            | (TriggerEvent::Attacks { .. }, GameEvent::AttackersDeclared { .. })
            | (TriggerEvent::Blocks { .. }, GameEvent::BlockersDeclared { .. } | GameEvent::BecameBlocked { .. }) => None,
            // Every arm, by name, so a new arm does not compile until it says what
            // this projection is for it; the record never matches the arm otherwise.
            (
                TriggerEvent::ZoneChange { .. }
                | TriggerEvent::BecomesTapped { .. }
                | TriggerEvent::BecomesUntapped { .. }
                | TriggerEvent::DrawsCard { .. }
                | TriggerEvent::ManaAdded { .. }
                | TriggerEvent::DamageDealt { .. }
                | TriggerEvent::DamageIsPrevented { .. }
                | TriggerEvent::PhaseBegins { .. }
                | TriggerEvent::StepBegins { .. }
                | TriggerEvent::TurnBegins { .. }
                | TriggerEvent::GainsLife { .. }
                | TriggerEvent::LosesLife { .. }
                | TriggerEvent::EntersBattlefield { .. }
                | TriggerEvent::Attacks { .. }
                | TriggerEvent::Blocks { .. }
                | TriggerEvent::BecomesTarget { .. }
                | TriggerEvent::ActivatesAbility { .. }
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
                | TriggerEvent::AbilityResolves { .. }
                | TriggerEvent::CreatesToken { .. }
                | TriggerEvent::Scries { .. },
                _,
            ) => None,
        }
    }

    /// "That many" (§3.4): the damage dealt, the mana added, the objects
    /// moved. `None` for an event with no quantity.
    pub fn amount_of(&self, record: &GameEvent) -> Option<u64> {
        match (self, record) {
            (TriggerEvent::DamageDealt { excess: true, .. }, GameEvent::DamageDealt { excess, .. }) => Some(*excess),
            (TriggerEvent::DamageDealt { excess: false, .. }, GameEvent::DamageDealt { amount, .. })
            | (TriggerEvent::DamageIsPrevented { .. }, GameEvent::DamagePrevented { amount, .. }) => Some(*amount),
            (TriggerEvent::ManaAdded { .. }, GameEvent::ManaAdded { mana, .. }) => {
                Some(mana.iter().map(|(_, n)| *n).sum())
            }
            (TriggerEvent::GainsLife { .. }, GameEvent::LifeChanged { .. }) => {
                Some(match record.life_change() {
                    Some(LifeChange::Gained(n)) => n,
                    Some(LifeChange::Lost(_)) | None => 0,
                })
            }
            // A counter is one occurrence of a per-counter arm, so "that many"
            // of it is one; the other readings sum the record's counters.
            (TriggerEvent::CountersPutOn { multiplicity, .. }, GameEvent::CountersPutOn { n, .. })
            | (TriggerEvent::CountersRemovedFrom { multiplicity, .. }, GameEvent::CountersRemovedFrom { n, .. }) => {
                Some(multiplicity.counter_occurrences(*n).1)
            }
            (TriggerEvent::LosesLife { .. }, GameEvent::LifeChanged { .. }) => {
                Some(match record.life_change() {
                    Some(LifeChange::Lost(n)) => n,
                    Some(LifeChange::Gained(_)) | None => 0,
                })
            }
            // The cards looked at, which a short library makes fewer than
            // the scry's number (Elrond, Master of Healing's first ruling).
            (TriggerEvent::Scries { .. }, GameEvent::Scried { looked_at, .. }) => Some(*looked_at),
            // One card or object a record, so "that many" over a "one or more"
            // binding is the cards drawn, or the objects moved: "the number of
            // cards milled" counts a Darksteel Colossus kept in its library.
            (TriggerEvent::DrawsCard { .. }, GameEvent::CardDrawn { .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::ZoneChange { .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::LeftTheGame { .. })
            | (TriggerEvent::ZoneChange { .. }, GameEvent::SameZoneMove { .. })
            | (TriggerEvent::CreatesToken { .. }, GameEvent::TokenCreated { .. }) => Some(1),
            (TriggerEvent::BecomesTapped { .. }, GameEvent::Tapped { .. })
            | (TriggerEvent::BecomesUntapped { .. }, GameEvent::Untapped { .. })
            | (TriggerEvent::PhaseBegins { .. }, GameEvent::PhaseBegin { .. })
            | (TriggerEvent::StepBegins { .. }, GameEvent::StepBegin { .. })
            | (TriggerEvent::TurnBegins { .. }, GameEvent::TurnBegin { .. })
            | (TriggerEvent::EntersBattlefield { .. }, GameEvent::PermanentEnteredBattlefield { .. })
            | (TriggerEvent::Attacks { .. }, GameEvent::AttackersDeclared { .. })
            | (TriggerEvent::Blocks { .. }, GameEvent::BlockersDeclared { .. } | GameEvent::BecameBlocked { .. })
            | (TriggerEvent::BecomesTarget { .. }, GameEvent::Targeted { .. })
            | (TriggerEvent::ActivatesAbility { .. }, GameEvent::AbilityActivated { .. })
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
                | TriggerEvent::DamageIsPrevented { .. }
                | TriggerEvent::PhaseBegins { .. }
                | TriggerEvent::StepBegins { .. }
                | TriggerEvent::TurnBegins { .. }
                | TriggerEvent::GainsLife { .. }
                | TriggerEvent::LosesLife { .. }
                | TriggerEvent::EntersBattlefield { .. }
                | TriggerEvent::Attacks { .. }
                | TriggerEvent::Blocks { .. }
                | TriggerEvent::BecomesTarget { .. }
                | TriggerEvent::ActivatesAbility { .. }
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
                | TriggerEvent::AbilityResolves { .. }
                | TriggerEvent::CreatesToken { .. }
                | TriggerEvent::Scries { .. },
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
    /// rather than on the three entries because the binding
    /// travels from the pending trigger to the stack and into the resolution
    /// by itself, and so would a copy of the ability (CR 707.10); on the
    /// entries, every one of those hand-offs would have to carry it.
    pub source_found_in_graveyard: Option<ObjectRef>,
}

impl TriggerBinding {
    /// The event the binding's projections read through. `None` for a state
    /// trigger, which no event triggered (CR 603.8).
    pub fn event(&self) -> Option<&TriggerEvent> {
        self.def.condition.events().get(self.event.0)
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
        let one = self.event()?.multiplicity().has_one_triggering_event();
        self.triggering_events.first().filter(|_| one)
    }

    /// "Its power", "if it had": the object that played `role` in the
    /// one triggering event, as the existence the dispatch found. `None` for a
    /// "one or more" trigger, which has no single object.
    pub fn triggering_object(&self, role: EventRole) -> Option<ObjectRef> {
        self.triggering_event()?.objects.get(self.event()?.role_as_recorded(role))
    }

    /// "That creature", "them": each object that played `role` in each
    /// triggering event. One for a `PerOccurrence` trigger, but for a
    /// defender's attackers.
    pub fn triggering_objects(&self, role: EventRole) -> Vec<ObjectRef> {
        let Some(event) = self.event() else { return Vec::new() };
        let role = event.role_as_recorded(role);
        self.triggering_events
            .iter()
            .flat_map(|event| event.objects.objects().filter(move |&(part, _)| part == role))
            .map(|(_, object)| object)
            .collect()
    }
}

/// Where a pending trigger came from (§3.8). A reflexive trigger is
/// `Delayed`, since CR 603.12 has it follow the delayed rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerOrigin {
    /// A printed, granted or copied ability of an object.
    Object(AbilityIdentity),
    /// CR 603.7 — a delayed triggered ability, by the identity CR 603.7d–g
    /// give it: its source, and its registry number as the ability
    /// (`AbilityId::delayed`). It is no ability of that object, and CR
    /// 603.2d's "triggers additional times" does not reach it.
    Delayed(AbilityIdentity),
    /// A triggered ability a rule gives the game rather than an object has:
    /// the monarch's "at the beginning of the monarch's end step, that player
    /// draws a card" (CR 724.2), the initiative's (CR 725.2). An arm of its
    /// own, never a fake source on `Object` (§14, question 6).
    Rule(InherentAbility),
}

/// A triggered ability no object has, which a rule owns (CR 724.2, 725.2).
/// No variant until its first designation, the monarch, lands with its
/// format (Phase 9), so no trigger has `TriggerOrigin::Rule` yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InherentAbility {}

impl TriggerOrigin {
    /// The ability that triggered, by what it is.
    pub fn identity(&self) -> AbilityIdentity {
        match self {
            TriggerOrigin::Object(identity) | TriggerOrigin::Delayed(identity) => *identity,
            TriggerOrigin::Rule(inherent) => match *inherent {},
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
    pub that_action: Option<(EventId, EventId)>,
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
    pub source_left_at: Option<EventId>,
    /// What its stack object is built from (CR 603.3's "the text of the
    /// ability that created it"), held because the source may be gone by
    /// then.
    pub source_card: Arc<CardData>,
    /// CR 603.7d–g's controller, as of the moment it was created.
    pub controller: PlayerId,
    /// "Its owner": the source's (CR 108.3), read as it was created, since
    /// the store may lose the source before the trigger reads it.
    pub owner: PlayerId,
    /// CR 603.7a's "won't trigger until it has actually been created".
    pub created: CreationPoint,
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
    pub left_at: Option<EventId>,
}

impl RememberedObject {
    /// An existence that has not moved yet.
    pub fn now(object: ObjectRef) -> Self {
        RememberedObject { object, left_at: None }
    }

    /// Whether record `seq`'s object `id` is this existence: its own
    /// departure, or, while it has not moved, the object as it is.
    pub fn is(self, game: &GameState, id: ObjectId, seq: EventId) -> bool {
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
/// left, and the facts about it that no layer computes. The LKI store keeps
/// one for each existence that ended with its move framed
/// (`engine::last_known`), and a departure record carries the same one (CR
/// 603.10a), as does damage a source deals after it left. A survivor's frame
/// from just before an event takes this shape too: a control change's, and a
/// move a replacement kept in its zone.
///
/// One body behind one pointer, which the store and every record that
/// carries it share.
#[derive(Debug, Clone, PartialEq)]
pub struct Lki(Arc<LkiBody>);

#[derive(Debug, PartialEq)]
struct LkiBody {
    object: ObjectRef,
    frame: Arc<EffectiveCharacteristics>,
    /// CR 108.3. The store loses a token at CR 704.5d and an object whose
    /// owner leaves at CR 800.4a, and this is then the one place it is kept.
    owner: PlayerId,
    /// CR 111.1: whether it was a token, which no characteristic says and
    /// the object store forgets once CR 704.5d has removed it.
    is_token: bool,
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
/// 301.5, 303.4), what it was attacking (CR 506.3), and the turn it entered.
#[derive(Debug, PartialEq)]
struct PermanentFacts {
    tapped: bool,
    entered_turn: u32,
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
        is_token: bool,
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
                entered_turn: entry.entered_battlefield_turn,
                attacking: entry.attacking.as_ref().map(|attack| attack.target),
                counters,
                attached_to: entry.attached_to,
                attached_by: entry.attached_by.clone(),
            }
        });
        let entry = entry.map(|entry| Arc::new(entry.clone()));
        Lki(Arc::new(LkiBody { object, frame, owner, is_token, cost_choices, permanent, entry }))
    }

    /// The bytes its body holds beside its frame, which a held-memory
    /// reading adds to the frames it counts (`HistoryLog::row_bytes_held`).
    pub fn bytes_beside_its_frame(&self) -> usize {
        let permanent = self.0.permanent.as_ref().map_or(0, |p| {
            p.counters.capacity() * std::mem::size_of::<(CounterType, u32)>()
                + p.attached_by.capacity() * std::mem::size_of::<ObjectId>()
        });
        std::mem::size_of::<LkiBody>() + 2 * std::mem::size_of::<usize>() + permanent
    }

    /// The existence that left.
    pub fn object(&self) -> ObjectRef {
        self.0.object
    }

    /// The characteristics it last had.
    pub fn characteristics(&self) -> &EffectiveCharacteristics {
        &self.0.frame
    }

    /// The same, shared.
    pub fn frame(&self) -> &Arc<EffectiveCharacteristics> {
        &self.0.frame
    }

    pub fn owner(&self) -> PlayerId {
        self.0.owner
    }

    /// Whether it was a token (CR 111.1).
    pub fn was_token(&self) -> bool {
        self.0.is_token
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

    /// The turn it entered the battlefield: "a creature that entered this
    /// turn" asked of one that has left. `None` for an object that was not a
    /// permanent.
    pub fn entered_turn(&self) -> Option<u32> {
        self.0.permanent.as_ref().map(|p| p.entered_turn)
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

    /// Whether it is among `player`'s triggers of `tier`, the ones CR 603.3b
    /// has that player put on the stack together.
    pub fn is_placed_with(&self, tier: TriggerTier, player: PlayerId) -> bool {
        self.tier() == tier && self.controller == player
    }
}

/// One arm of every kind and one record of every kind, for the tests that
/// pair them: which records an arm reads (`record_kinds`), and that the
/// matcher has a case for each pair it admits (`triggering_events_in`).
#[cfg(test)]
pub(crate) mod samples {
    use super::*;
    use crate::state::game_state::AbilityIdentity;
    use crate::types::ids::{AbilityId, ObjectRef};

    /// A frame for the records that carry one: a 2/2 as it was on the
    /// battlefield.
    pub(crate) fn lki() -> Lki {
        let mut game = crate::test_support::setup_two_player_game();
        let bear = crate::test_support::put_on_battlefield(&mut game, crate::test_support::vanilla_creature(2, 2, &[]), 0);
        game.lki_now(bear).unwrap_or_else(|| panic!("{bear} is on the battlefield"))
    }

    /// One record of every kind a trigger arm can read, named, plus one
    /// (`TokenCeasedToExist`) that no arm reads. `lki` is the frame of the
    /// records that carry one.
    pub(crate) fn records(lki: Lki) -> Vec<(&'static str, GameEvent)> {
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
            ("ManaAdded", GameEvent::ManaAdded { player_id: 0, source_id: id, mana: Vec::new(), tapped_for_mana: false, source_lki: None }),
            ("DamageDealt", GameEvent::DamageDealt {
                source_id: id,
                target: DamageTarget::Player(0),
                amount: 1,
                excess: 0,
                is_combat: false,
                source_lki: None,
            }),
            ("PhaseBegin", GameEvent::PhaseBegin { phase: PhaseType::Precombat, player: 0 }),
            ("StepBegin", GameEvent::StepBegin { step: StepType::Upkeep, player: 0 }),
            ("TurnBegin", GameEvent::TurnBegin { player: 0, turn_number: 1 }),
            ("LifeChanged", GameEvent::LifeChanged { player_id: 0, old: 20, new: 21, source: None }),
            ("PermanentEnteredBattlefield", GameEvent::PermanentEnteredBattlefield { object_id: id, controller: 0 }),
            ("AttackersDeclared", GameEvent::AttackersDeclared { attackers: vec![(id, AttackTarget::Player(1))] }),
            ("BlockersDeclared", GameEvent::BlockersDeclared { blocks: vec![(id, id)], unblocked: Vec::new() }),
            ("BecameBlocked", GameEvent::BecameBlocked { attacker: id }),
            ("AbilityTriggered", GameEvent::AbilityTriggered {
                seq: TriggerSeq(0),
                origin: TriggerOrigin::Object(identity),
                controller: 0,
                caused_by: Some(EventId(0)),
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
                attachment_left: false,
                former_host_left: false,
            }),
            ("DamagePrevented", GameEvent::DamagePrevented {
                target: DamageTarget::Player(0),
                amount: 1,
                by: crate::engine::replacement::ReplacementInstanceId::StaticAbility(identity.source, identity.ability),
            }),
            ("Scried", GameEvent::Scried { player_id: 0, n: 1, looked_at: 1 }),
            ("TokenCreated", GameEvent::TokenCreated {
                object_id: id,
                owner: 0,
                zone: Zone::Battlefield,
                definition: Arc::new(crate::cards::phase_re_cards::goblin_token()),
            }),
            ("Targeted", GameEvent::Targeted {
                target: crate::engine::targeting::TargetRef::Object(identity.source),
                by: id,
                ability_source: None,
                controller: 0,
                instances: 1,
            }),
            ("AbilityActivated", GameEvent::AbilityActivated { identity, controller: 0, mana_ability: false, from: Zone::Battlefield, source_lki: None }),
            ("SpellCountered", GameEvent::SpellCountered { countered_by: id, lki: lki.clone() }),
            ("PlayerLost", GameEvent::PlayerLost { player_id: 0 }),
            ("SameZoneMove", GameEvent::SameZoneMove { zone: Zone::Library, act: ZoneChangeAct::Milled, applied: None, lki: lki.clone() }),
            ("ControlChanged", GameEvent::ControlChanged { to: 1, lki }),
            ("CountersPutOn", GameEvent::CountersPutOn {
                subject: CounterSubject::Object(id),
                counter_type: CounterType::PlusOnePlusOne,
                n: 1,
                by: 0,
                now: 1,
            }),
            ("CountersRemovedFrom", GameEvent::CountersRemovedFrom {
                subject: CounterSubject::Object(id),
                counter_type: CounterType::PlusOnePlusOne,
                n: 1,
                by: Some(0),
                now: 0,
            }),
            ("TokenCeasedToExist", GameEvent::TokenCeasedToExist { object_id: id }),
        ]
    }

    /// One arm of each kind, and two more where a field changes which kinds
    /// it reads (an act, and losing control to nobody), each with the records
    /// it reads, written out by hand.
    pub(crate) fn arms() -> Vec<(TriggerEvent, &'static [&'static str])> {
        let this = TriggerSubject::ThisObject;
        let each = Multiplicity::PerOccurrence;
        let anywhere = ZoneChangePattern { from: None, to: None, act: None };
        vec![
            (
                TriggerEvent::ZoneChange { subject: this.clone(), change: anywhere, by: None, owner: None, multiplicity: each },
                &["ZoneChange", "LeftTheGame"],
            ),
            (
                TriggerEvent::ZoneChange {
                    subject: this.clone(),
                    change: ZoneChangePattern { act: Some(ZoneChangeAct::Milled), ..anywhere },
                    by: None,
                    owner: None,
                    multiplicity: each,
                },
                &["ZoneChange", "LeftTheGame", "SameZoneMove"],
            ),
            (TriggerEvent::BecomesTapped { subject: this.clone() }, &["Tapped"]),
            (TriggerEvent::BecomesUntapped { subject: this.clone() }, &["Untapped"]),
            (TriggerEvent::DrawsCard { player: None, multiplicity: each, place: None }, &["CardDrawn"]),
            (TriggerEvent::ManaAdded { source: this.clone(), tapped_for_mana: None, mana: None }, &["ManaAdded"]),
            (
                TriggerEvent::DamageDealt { source: this.clone(), recipient: DamageRecipient::Any, combat: None, excess: false, multiplicity: each },
                &["DamageDealt"],
            ),
            (TriggerEvent::DamageIsPrevented { recipient: DamageRecipient::Any, multiplicity: each }, &["DamagePrevented"]),
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
                TriggerEvent::Attacks { attacker: this.clone(), defender: None, occurrence: AttackOccurrence::Attacker, alone: false, multiplicity: each },
                &["AttackersDeclared"],
            ),
            (
                TriggerEvent::Blocks {
                    blocker: TriggerSubject::Any,
                    attacker: this.clone(),
                    occurrence: BlockOccurrence::Blocked,
                    count: None,
                    alone: false,
                    multiplicity: each,
                },
                &["BlockersDeclared", "BecameBlocked"],
            ),
            (
                TriggerEvent::BecomesTarget {
                    target: TargetSubject::Object(this.clone()),
                    by: TargetedBy::SpellOrAbility,
                    controller: None,
                    multiplicity: each,
                },
                &["Targeted"],
            ),
            (TriggerEvent::ActivatesAbility { player: None, source: this.clone(), mana_ability: None }, &["AbilityActivated"]),
            (TriggerEvent::CastsSpell { caster: None, spell: None }, &["SpellCast"]),
            (TriggerEvent::IsCountered { spell: None, caster: None }, &["SpellCountered"]),
            (TriggerEvent::PlayerLoses { player: None }, &["PlayerLost"]),
            (TriggerEvent::ShufflesLibrary { player: None }, &["LibraryShuffled"]),
            (
                TriggerEvent::CountersPutOn { subject: CountersOn::Object(this.clone()), counter_type: None, by: None, multiplicity: each },
                &["CountersPutOn"],
            ),
            (
                TriggerEvent::CountersRemovedFrom { subject: CountersOn::Object(this.clone()), counter_type: None, by: None, multiplicity: each },
                &["CountersRemovedFrom"],
            ),
            (TriggerEvent::BecomesAttached { attachment: TriggerSubject::Any, host: TriggerSubject::Any }, &["Attached"]),
            (
                TriggerEvent::BecomesUnattached { attachment: TriggerSubject::Any, former_host: TriggerSubject::Any },
                &["Unattached"],
            ),
            (TriggerEvent::ControlChanges { subject: this.clone(), from: None, to: None }, &["ControlChanged", "ZoneChange", "LeftTheGame"]),
            (TriggerEvent::ControlChanges { subject: this, from: None, to: Some(PlayerRef::You) }, &["ControlChanged"]),
            (TriggerEvent::AbilityTriggers { caused_by: None, source: None }, &["AbilityTriggered"]),
            (TriggerEvent::AbilityResolves { identity: IdentityRef::ThisAbility }, &["AbilityResolved"]),
            (TriggerEvent::CreatesToken { owner: None, definition: None, multiplicity: each }, &["TokenCreated"]),
            (TriggerEvent::Scries { player: None }, &["Scried"]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Which records each arm reads, written out by hand, one line per arm.
    ///
    /// `reads` is not a table of its own: it asks the arm's `record_kinds`
    /// and the record's `EventKind::from_record`, the two halves the
    /// dispatcher's source mask is built from. This pins what those halves
    /// answer together, so a wrong entry in either fails here. Every arm is
    /// in the list (`samples::arms`).
    #[test]
    fn each_arm_reads_exactly_the_records_it_names() {
        let records = samples::records(samples::lki());
        let expected = samples::arms();
        for (arm, reads) in &expected {
            for (name, record) in &records {
                assert_eq!(arm.reads(record), reads.contains(name), "{arm:?} reading a {name} record");
            }
        }
    }

    /// An arm that watches a player declares no event role a player fills:
    /// damage to a player has no recipient object (CR 120.1), and a player
    /// attacked is no object attacked (CR 506.3), so "that creature" there is
    /// refused at the registry rather than finding nothing at resolution.
    #[test]
    fn an_arm_that_watches_a_player_declares_no_object_in_its_place() {
        let each = Multiplicity::PerOccurrence;
        let you = || Some(PlayerRef::You);
        let to_you = TriggerEvent::DamageDealt {
            source: TriggerSubject::Any,
            recipient: DamageRecipient::Player(you()),
            combat: None,
            excess: false,
            multiplicity: each,
        };
        assert_eq!(to_you.event_roles(), &[EventRole::DamageSource]);
        let prevented_to_you = TriggerEvent::DamageIsPrevented { recipient: DamageRecipient::Player(you()), multiplicity: each };
        assert_eq!(prevented_to_you.event_roles(), &[] as &[EventRole]);
        let attacks_you = TriggerEvent::Attacks {
            attacker: TriggerSubject::Any,
            defender: Some(DefenderFilter::Player(you())),
            occurrence: AttackOccurrence::Attacker,
            alone: false,
            multiplicity: each,
        };
        assert_eq!(attacks_you.event_roles(), &[EventRole::Attacker]);
        let attacks_anything = TriggerEvent::Attacks {
            attacker: TriggerSubject::Any,
            defender: Some(DefenderFilter::Or(Box::new(DefenderFilter::Player(you())), Box::new(DefenderFilter::Permanent(ObjectFilter::All)))),
            occurrence: AttackOccurrence::Attacker,
            alone: false,
            multiplicity: each,
        };
        assert_eq!(attacks_anything.event_roles(), &[EventRole::Attacker, EventRole::Attacked]);
    }

    /// CR 603.10a's classes, asked of an arm that names both zones, are the
    /// move's own: the arm looks back exactly when its move carries a frame.
    #[test]
    fn an_arm_naming_both_zones_looks_back_as_its_move_does() {
        let zones = [Zone::Library, Zone::Hand, Zone::Battlefield, Zone::Graveyard, Zone::Stack, Zone::Exile, Zone::Command];
        for from in zones {
            for to in zones {
                let arm = TriggerEvent::ZoneChange {
                    subject: TriggerSubject::Any,
                    change: ZoneChangePattern { from: Some(from), to: Some(to), act: None },
                    by: None,
                    owner: None,
                    multiplicity: Multiplicity::PerOccurrence,
                };
                assert_eq!(arm.looks_back(), zone_change_looks_back(from, Some(to)), "{from:?} to {to:?}");
            }
        }
    }

    /// The mask is as wide as the kinds need and no wider: one word for the
    /// nineteen there are, every kind in it, and none of them in the empty set.
    #[test]
    fn the_mask_holds_every_kind_in_one_word() {
        assert_eq!(EventKind::WORDS, 1);
        let mut all = EventKindMask::EMPTY;
        for (_, record) in samples::records(samples::lki()) {
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
    /// whatever its condition asks: it is filed under the one kind no record
    /// carries, which no window meets.
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
        assert_eq!(state.record_kinds(), EventKindMask::of(EventKind::BoardState));
        assert!(!state.record_kinds().intersects(kinds));
    }
}
