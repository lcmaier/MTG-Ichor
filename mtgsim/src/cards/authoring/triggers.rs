//! CR 603's authoring words (`triggers-architecture.md` §3).
//!
//! **The cause-and-owner builder is deliberately absent.**
//! `.caused_by(Sacrificed)` and `.owned_by(Opponent)` are one method each on
//! [`CountableEvent`] and they wait for the first card that prints one beside
//! another cause: none of TR-1's five does, and none of TR-2's seven.
//! Cornered Crook's "when you do" is a sacrifice and nothing else, which is
//! [`is_sacrificed`]'s one word. Until then a card that asks writes the arm out —
//! which is what the one fixture that asks (a discard, `cause:
//! Some(Discarded)`) already does, and reads correctly, because its fields
//! are `Some`. → `triggers-architecture.md` §15 item 14.

use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction};
use crate::state::game_state::StepType;
use crate::types::effects::{
    Antecedent, Condition, CounterType, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, ReturnUnder,
};
use crate::types::history::CountIs;
use crate::types::replacement::EntryCounters;
use crate::types::ids::AbilityId;
use crate::types::card_types::CardType;
use crate::types::triggers::{
    AttackEventPer, CountersOn, DefenderFilter, DelayedDuration, DelayedTriggerTemplate, DelayedTurn, EventRole, Multiplicity,
    ReflexiveForm, TriggerCondition, TriggerDef, TriggerEvent, TriggerSubject,
};
use crate::types::zones::{Zone, ZoneChangeAct, ZoneChangePattern};

/// A triggered ability: no cost, the def as its effect.
pub fn triggered_ability(rules_text: &'static str, def: TriggerDef) -> AbilityDef {
    AbilityDef {
        rules_text: rules_text.into(),
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
/// matcher reads as other than the ability's own source. In a targeting
/// filter it is "another target creature", other than the same object.
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

/// A counter arm, one trigger per counter until told otherwise: "whenever a
/// +1/+1 counter is put on this creature" triggers once for each (Fathom
/// Mage's ninth ruling). Built by [`counters_put_on`] and
/// [`counters_removed_from`] only, so the three readings of how many
/// triggers an event makes are its methods' and no other arm's.
pub struct CounterEvent {
    direction: CounterDirection,
    subject: CountersOn,
    counter_type: Option<CounterType>,
    by: Option<PlayerRef>,
}

/// Which of the two counter records a [`CounterEvent`] reads.
enum CounterDirection {
    PutOn,
    RemovedFrom,
}

impl CounterEvent {
    /// "One or more [kind] counters are put on [a creature]": once for each
    /// object the counters are on (Enduring Scalelord's second ruling).
    pub fn once_per_object(self) -> TriggerEvent {
        self.counted(Multiplicity::OncePerSubject)
    }

    /// "One or more ... on one or more [creatures]": once for the event.
    pub fn once_per_event(self) -> TriggerEvent {
        self.counted(Multiplicity::OncePerEvent)
    }

    /// "Whenever [who] put[s]" or "remove[s]": the record's player (CR
    /// 122.6a for a put).
    pub fn by(self, who: PlayerRef) -> CounterEvent {
        CounterEvent { by: Some(who), ..self }
    }

    fn counted(self, multiplicity: Multiplicity) -> TriggerEvent {
        let CounterEvent { direction, subject, counter_type, by } = self;
        match direction {
            CounterDirection::PutOn => TriggerEvent::CountersPutOn { subject, counter_type, by, multiplicity },
            CounterDirection::RemovedFrom => TriggerEvent::CountersRemovedFrom { subject, counter_type, by, multiplicity },
        }
    }
}

impl From<CounterEvent> for TriggerEvent {
    fn from(event: CounterEvent) -> Self {
        event.counted(Multiplicity::PerOccurrence)
    }
}

/// "Whenever a [kind] counter is put on [subject]"; `None` is any kind.
/// Counters it enters with are put on it too (CR 122.6).
pub fn counters_put_on(subject: impl Into<TriggerSubject>, counter_type: impl Into<Option<CounterType>>) -> CounterEvent {
    CounterEvent {
        direction: CounterDirection::PutOn,
        subject: CountersOn::Object(subject.into()),
        counter_type: counter_type.into(),
        by: None,
    }
}

/// "Whenever a [kind] counter is removed from [subject]"; `None` is any kind.
pub fn counters_removed_from(
    subject: impl Into<TriggerSubject>,
    counter_type: impl Into<Option<CounterType>>,
) -> CounterEvent {
    CounterEvent {
        direction: CounterDirection::RemovedFrom,
        subject: CountersOn::Object(subject.into()),
        counter_type: counter_type.into(),
        by: None,
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
            | TriggerEvent::Attacks { multiplicity, .. }
            | TriggerEvent::CountersPutOn { multiplicity, .. }
            | TriggerEvent::CountersRemovedFrom { multiplicity, .. } => {
                *multiplicity = Multiplicity::OncePerEvent
            }
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
            | TriggerEvent::AbilityResolves { .. } => {
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
        change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None },
        by: None,
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
        change: ZoneChangePattern { from: Some(Zone::Battlefield), to: None, act: None },
        by: None,
        owner: None,
        multiplicity: Multiplicity::PerOccurrence,
    })
}

/// "[Subject] is sacrificed" (CR 701.21a), wherever a replacement sends it:
/// the event CR 603.12's "when you do" names after "you may sacrifice …",
/// with the subject "you" controlled as it left.
pub fn is_sacrificed(subject: impl Into<TriggerSubject>) -> CountableEvent {
    CountableEvent(TriggerEvent::ZoneChange {
        subject: subject.into(),
        change: ZoneChangePattern { from: Some(Zone::Battlefield), to: None, act: Some(ZoneChangeAct::Sacrificed) },
        by: None,
        owner: None,
        multiplicity: Multiplicity::PerOccurrence,
    })
}

/// Persist (CR 702.79a): "When this permanent is put into a graveyard from
/// the battlefield, if it had no -1/-1 counters on it, return it to the
/// battlefield under its owner's control with a -1/-1 counter on it."
pub fn persist() -> AbilityDef {
    returns_unless_it_had("Persist", CounterType::MinusOneMinusOne)
}

/// Undying (CR 702.93a): persist's mirror, on +1/+1 counters.
pub fn undying() -> AbilityDef {
    returns_unless_it_had("Undying", CounterType::PlusOnePlusOne)
}

/// The shape both keywords expand to. "It had" is read as the permanent left
/// (CR 603.10a), at the trigger and again at resolution (CR 603.4), and "it"
/// is the card it became in the graveyard (CR 400.7e), which comes back as a
/// new object with one counter on it (CR 122.6).
fn returns_unless_it_had(keyword: &'static str, counter: CounterType) -> AbilityDef {
    let returns = Primitive::ReturnToBattlefield {
        under: ReturnUnder::Owner,
        counters: vec![EntryCounters { counter, n: 1, by: None }],
        tapped: false,
        attacking: None,
    };
    triggered_ability(keyword, TriggerDef {
        condition: TriggerCondition::Event(dies(TriggerSubject::ThisObject).into()),
        intervening_if: Some(Condition::CounterCount { object: Antecedent::Triggering(EventRole::Object), kind: Some(counter), count: CountIs::AtMost(0) }),
        limit: None,
        effect: Effect::Atom(returns, EffectRecipient::Triggering(EventRole::Object)),
    })
}

/// CR 603.12's "When you do, [effect]": a reflexive trigger on `event`, the
/// action the instruction before it took. It is made as the resolution
/// reaches it and checked then, once, against that action's records, so it
/// triggers once for each time the action did `event` (603.12a) and never on
/// a later event: it does not wait. "When [something happens] this way" is
/// the same check.
pub fn when_you_do(event: impl Into<TriggerEvent>, effect: Effect, rules_text: &'static str) -> DelayedTriggerTemplate {
    reflexive(ReflexiveForm::Does, event.into(), effect, rules_text)
}

/// CR 603.12's "When you don't, [effect]": the same check as [`when_you_do`],
/// triggering once if the action did not do `event`.
pub fn when_you_dont(event: impl Into<TriggerEvent>, effect: Effect, rules_text: &'static str) -> DelayedTriggerTemplate {
    reflexive(ReflexiveForm::Doesnt, event.into(), effect, rules_text)
}

fn reflexive(form: ReflexiveForm, event: TriggerEvent, effect: Effect, rules_text: &'static str) -> DelayedTriggerTemplate {
    DelayedTriggerTemplate {
        def: std::sync::Arc::new(whenever(event, effect)),
        duration: DelayedDuration::Reflexive(form),
        turn: DelayedTurn::NoneNamed,
        rules_text: rules_text.into(),
    }
}

/// "Whenever [whose player] draw[s] a card" (CR 121.1): one trigger per card
/// drawn.
pub fn draws_a_card(whose: Whose) -> CountableEvent {
    CountableEvent(TriggerEvent::DrawsCard { player: whose.player_ref(), multiplicity: Multiplicity::PerOccurrence, place: None })
}

/// "Whenever a spell [whose player]'s cast is countered" (CR 701.6a): "a spell
/// you've cast" is `Whose::Yours`, read off who cast it (CR 603.10e).
pub fn spell_is_countered(cast_by: Whose) -> TriggerEvent {
    TriggerEvent::IsCountered { spell: None, caster: cast_by.player_ref() }
}

/// "When you lose control of [subject]" (CR 603.10d), which leaving the
/// battlefield is too (Khârn the Betrayer's first ruling).
pub fn you_lose_control_of(subject: impl Into<TriggerSubject>) -> TriggerEvent {
    TriggerEvent::ControlChanges { subject: subject.into(), from: Some(PlayerRef::You), to: None }
}

/// "Whenever an opponent gains control of [subject] from you" (CR 603.10d).
pub fn an_opponent_gains_control_of(subject: impl Into<TriggerSubject>) -> TriggerEvent {
    TriggerEvent::ControlChanges { subject: subject.into(), from: Some(PlayerRef::You), to: Some(PlayerRef::Opponent) }
}

/// "Whenever [whose player] loses the game" (CR 603.9): "a player" is
/// `Whose::Each`, "an opponent" `Whose::AnOpponents`.
pub fn loses_the_game(whose: Whose) -> TriggerEvent {
    TriggerEvent::PlayerLoses { player: whose.player_ref() }
}

/// "Whenever [whose player] shuffle[s] their library" (CR 701.24).
pub fn shuffles_their_library(whose: Whose) -> TriggerEvent {
    TriggerEvent::ShufflesLibrary { player: whose.player_ref() }
}

/// "Whenever [attachment] becomes attached to [host]" (CR 701.3a, 603.2e).
pub fn becomes_attached(attachment: impl Into<TriggerSubject>, host: impl Into<TriggerSubject>) -> TriggerEvent {
    TriggerEvent::BecomesAttached { attachment: attachment.into(), host: host.into() }
}

/// "Whenever [attachment] becomes unattached from [former host]" (CR
/// 701.3d, 603.10c).
pub fn becomes_unattached(attachment: impl Into<TriggerSubject>, former_host: impl Into<TriggerSubject>) -> TriggerEvent {
    TriggerEvent::BecomesUnattached { attachment: attachment.into(), former_host: former_host.into() }
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

/// "Whenever [subject] attacks" (CR 508.3a): one trigger for each creature
/// declared as an attacker, so "whenever a creature you control attacks" is
/// also CR 508.3c's "whenever you attack with a creature".
pub fn attacks(subject: impl Into<TriggerSubject>) -> CountableEvent {
    CountableEvent(attack_event(subject.into(), None, AttackEventPer::Attacker))
}

/// "Whenever [subject] attacks [defender]" (CR 508.3a's second sentence):
/// "whenever a creature attacks you or a planeswalker you control".
pub fn attacks_defender(subject: impl Into<TriggerSubject>, defender: DefenderFilter) -> CountableEvent {
    CountableEvent(attack_event(subject.into(), Some(defender), AttackEventPer::Attacker))
}

/// "Whenever [subject] attacks alone" (CR 506.5): the only creature declared
/// as an attacker.
pub fn attacks_alone(subject: impl Into<TriggerSubject>) -> TriggerEvent {
    TriggerEvent::Attacks {
        attacker: subject.into(),
        defender: None,
        per: AttackEventPer::Attacker,
        alone: true,
        multiplicity: Multiplicity::PerOccurrence,
    }
}

/// "Whenever [defender] is attacked" (CR 508.3b): one trigger for each
/// player, planeswalker or battle that one or more creatures are declared
/// attacking, whose "them" are those creatures.
pub fn is_attacked(defender: DefenderFilter) -> TriggerEvent {
    attack_event(TriggerSubject::Any, Some(defender), AttackEventPer::Defender)
}

/// "Whenever [whose player] attacks" (CR 508.3d): one trigger for a
/// declaration with one or more creatures that player controls in it, whose
/// "them" are those creatures.
pub fn player_attacks(whose: Whose) -> TriggerEvent {
    CountableEvent(attack_event(creature_controlled_by(whose), None, AttackEventPer::Attacker)).once_per_event()
}

/// "Whenever [whose player] attacks [attacked player]" (CR 508.3e): one
/// trigger for each such player one or more creatures the first player
/// controls are declared attacking. A planeswalker or battle attacked is not
/// the player.
pub fn player_attacks_player(whose: Whose, attacked: Whose) -> TriggerEvent {
    let defender = DefenderFilter::Player(attacked.player_ref());
    attack_event(creature_controlled_by(whose), Some(defender), AttackEventPer::Defender)
}

/// "[A player] or a planeswalker [that player] controls", as what is
/// attacked: "whenever a creature attacks you or a planeswalker you control".
pub fn player_or_their_planeswalker(who: PlayerRef) -> DefenderFilter {
    let their_planeswalker = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Planeswalker)),
        Box::new(ObjectFilter::ByController(who.clone())),
    );
    DefenderFilter::Or(Box::new(DefenderFilter::Player(Some(who))), Box::new(DefenderFilter::Permanent(their_planeswalker)))
}

fn attack_event(attacker: TriggerSubject, defender: Option<DefenderFilter>, per: AttackEventPer) -> TriggerEvent {
    TriggerEvent::Attacks { attacker, defender, per, alone: false, multiplicity: Multiplicity::PerOccurrence }
}

/// "A creature [whose player] controls"; any creature for `Whose::Each`.
fn creature_controlled_by(whose: Whose) -> TriggerSubject {
    match whose.player_ref() {
        None => TriggerSubject::Any,
        Some(player) => TriggerSubject::Filter(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Creature)),
            Box::new(ObjectFilter::ByController(player)),
        )),
    }
}
