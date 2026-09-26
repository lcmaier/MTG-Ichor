//! CR 603's authoring words (`triggers-architecture.md` §3).
//!
//! **The cause-and-owner builder is deliberately absent.**
//! `.caused_by(Sacrificed)` and `.owned_by(Opponent)` are one method each on
//! [`CountableEvent`] and they wait for the first card that prints one: none
//! of TR-1's five does, and none of TR-2's seven.
//! Until then a card that asks writes the arm out — which is what the one
//! fixture that asks (a discard, `cause: Some(Discarded)`) already does, and
//! reads correctly, because its fields are `Some`. → `triggers-
//! architecture.md` §15 item 14.

use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction};
use crate::state::game_state::StepType;
use crate::types::effects::{Effect, ObjectFilter, PlayerRef};
use crate::types::ids::AbilityId;
use crate::types::triggers::{
    Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerSubject,
};
use crate::types::zones::Zone;

/// A triggered ability: no cost, the def as its effect.
pub fn triggered_ability(def: TriggerDef) -> AbilityDef {
    AbilityDef {
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Triggered,
        costs: Vec::new(),
        effect: Effect::Triggered(std::sync::Arc::new(def)),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    }
}

/// "Whenever [event], [effect]" with no "if" and no limit. A card with an
/// intervening "if" (CR 603.4) or a once-each-turn gate (603.2h) writes the
/// [`TriggerDef`] out, because those two fields are the whole of it.
pub fn whenever(event: impl Into<TriggerEvent>, effect: Effect) -> TriggerDef {
    TriggerDef {
        condition: TriggerCondition::Event(event.into()),
        intervening_if: None,
        limit: None,
        effect,
    }
}

/// "Another [filter]" (CR 603.6a): the filter beside `NotSource`, which the
/// matcher reads as other than the ability's own source.
pub fn another(filter: ObjectFilter) -> ObjectFilter {
    ObjectFilter::And(Box::new(filter), Box::new(ObjectFilter::NotSource))
}

/// So a card writes `dies(a_creature())` where the arm wants a subject.
/// Nothing in the engine converts one — this is the authoring surface's
/// sugar and lives with it, and rustdoc still lists the impl on
/// [`TriggerSubject`] for a reader who starts at the type.
impl From<ObjectFilter> for TriggerSubject {
    fn from(filter: ObjectFilter) -> Self {
        TriggerSubject::Filter(filter)
    }
}

/// Whose step, phase, draw or shuffle a card means — the word for the
/// `Option`s on a [`TriggerEvent`] where `None` is a *distributive* reading
/// rather than an unasked question. "At the beginning of each upkeep" is `whose: None` at
/// the type, and a card that spells it `None` tells the next reader
/// "nobody's upkeep". Everywhere else `None` does mean the arm does not
/// ask, and a card says that by not calling a constructor that sets it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Whose {
    /// "Each upkeep" — every player's, whoever's turn it is.
    Each,
    /// "Your upkeep" — the ability's controller's.
    Yours,
    /// "Each opponent's upkeep".
    AnOpponents,
}

impl Whose {
    /// The type-level spelling the matcher reads (`dispatch::player_ref_is`).
    fn player_ref(self) -> Option<PlayerRef> {
        match self {
            Whose::Each => None,
            Whose::Yours => Some(PlayerRef::You),
            Whose::AnOpponents => Some(PlayerRef::Opponent),
        }
    }
}

/// "At the beginning of [whose] [step]".
pub fn at_beginning_of(step: StepType, whose: Whose) -> TriggerEvent {
    TriggerEvent::StepBegins { step, whose: whose.player_ref() }
}

/// A trigger event whose arm carries CR 603.2c's multiplicity, and therefore
/// the only thing that can be told [`once_per_event`](Self::once_per_event).
/// The constructors below are its only source and every arm they build has
/// that field, so "whenever one or more permanents become tapped" — an arm
/// that has none — fails to compile, rather than panicking in a card file or
/// silently staying per-occurrence.
pub struct CountableEvent(TriggerEvent);

impl CountableEvent {
    /// CR 603.2c's "one or more": the window is the event, so the ability
    /// triggers once however many records match it.
    ///
    /// The two groups below are [`TriggerEvent::multiplicity`]'s own, read
    /// as a setter — a new arm has to join one of them here as well as
    /// there.
    pub fn once_per_event(mut self) -> TriggerEvent {
        match &mut self.0 {
            TriggerEvent::ZoneChange { multiplicity, .. }
            | TriggerEvent::DrawsCard { multiplicity, .. }
            | TriggerEvent::DamageDealt { multiplicity, .. }
            | TriggerEvent::GainsLife { multiplicity, .. }
            | TriggerEvent::LosesLife { multiplicity, .. }
            | TriggerEvent::EntersBattlefield { multiplicity, .. }
            | TriggerEvent::Attacks { multiplicity, .. } => {
                *multiplicity = Multiplicity::OncePerEvent
            }
            TriggerEvent::BecomesTapped { .. }
            | TriggerEvent::BecomesUntapped { .. }
            | TriggerEvent::ManaAdded { .. }
            | TriggerEvent::PhaseBegins { .. }
            | TriggerEvent::StepBegins { .. }
            | TriggerEvent::TurnBegins { .. }
            | TriggerEvent::CastsSpell { .. }
            | TriggerEvent::ShufflesLibrary { .. }
            | TriggerEvent::AbilityTriggers { .. } => {
                unreachable!("only an arm with a multiplicity becomes a CountableEvent")
            }
        }
        self.0
    }
}

impl From<CountableEvent> for TriggerEvent {
    fn from(countable: CountableEvent) -> Self {
        countable.0
    }
}

/// "Whenever [subject] dies" — CR 700.4's term, "is put into a graveyard
/// from the battlefield", which in `tmnt.txt` carries no creature
/// restriction: this is the word for any permanent. Titania, Protector of
/// Argoth still prints the long phrase for lands, so Phase 8's parser maps
/// both spellings here.
pub fn dies(subject: impl Into<TriggerSubject>) -> CountableEvent {
    CountableEvent(TriggerEvent::ZoneChange {
        subject: subject.into(),
        from: Some(Zone::Battlefield),
        to: Some(Zone::Graveyard),
        cause: None,
        owner: None,
        multiplicity: Multiplicity::PerOccurrence,
    })
}

/// "Whenever [subject] leaves the battlefield" — `to: None`, the CR 603.6c
/// ability. [`dies`] is its graveyard half and looks back off the same rule;
/// "from anywhere" is neither, and writes the arm out.
pub fn leaves_the_battlefield(subject: impl Into<TriggerSubject>) -> CountableEvent {
    CountableEvent(TriggerEvent::ZoneChange {
        subject: subject.into(),
        from: Some(Zone::Battlefield),
        to: None,
        cause: None,
        owner: None,
        multiplicity: Multiplicity::PerOccurrence,
    })
}

/// "Whenever [whose player] draw[s] a card" (CR 121.1): one trigger per card
/// drawn.
pub fn draws_a_card(whose: Whose) -> CountableEvent {
    CountableEvent(TriggerEvent::DrawsCard { player: whose.player_ref(), multiplicity: Multiplicity::PerOccurrence })
}

/// "Whenever [whose player] shuffle[s] their library" (CR 701.24).
pub fn shuffles_their_library(whose: Whose) -> TriggerEvent {
    TriggerEvent::ShufflesLibrary { player: whose.player_ref() }
}

/// "Whenever [subject] enters" (CR 603.6a). Which zone it came from and
/// whether it was cast are unasked.
pub fn enters(subject: impl Into<TriggerSubject>) -> CountableEvent {
    CountableEvent(TriggerEvent::EntersBattlefield {
        subject: subject.into(),
        controller: None,
        from: None,
        was_cast: None,
        multiplicity: Multiplicity::PerOccurrence,
    })
}
