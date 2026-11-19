//! The choices a step of a resolving effect reads back (CR 608.2d): each
//! name its own words point at, "the chosen number", "it", "of a type
//! chosen this way".
//!
//! Two readers ask. The resolution asks before a step acts: a step that
//! reads a choice which was never made is a part of the effect that "won't
//! happen" (CR 608.2b), and only the list can say so ahead of a verb, an
//! amount or a player word that would otherwise meet the missing name
//! midway. `cards::registry`'s test asks of every card: a name is read only
//! after the step that keeps it.
//!
//! Each function below answers for one node. Every match is exhaustive, so
//! a new place a chosen name can sit does not compile until it says what it
//! reads. **A definition a step creates is not read here**: a granted
//! ability, a replacement, a "can't" and a delayed trigger outlast the
//! resolution and its choices with it, so a name inside one is the
//! registry's to refuse, which it finds in the definition's printed form.

use std::ops::BitOr;

use crate::types::costs::Cost;
use crate::types::effects::{
    AmountExpr, AnsweredBy, ChoiceKind, Condition, EffectRecipient, ObjectFilter, ObjectSet, PickCount, PlayerFact, PlayerRef, Primitive,
    SelectionFilter, TargetCount, ValueChoice,
};
use crate::types::history::{HistoryCount, ObjectFact};

/// The names a node reads, each once, in the order met, and whether the
/// read needs the one answer: an amount and a player word read exactly one
/// ("that many", "that player may"), where a filter and a verb read every
/// answer a walk was given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoicesRead(Vec<ChoiceRead>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChoiceRead {
    kind: ChoiceKind,
    needs_one_answer: bool,
}

impl ChoicesRead {
    const NONE: ChoicesRead = ChoicesRead(Vec::new());

    fn the(kind: ChoiceKind) -> ChoicesRead {
        ChoicesRead(vec![ChoiceRead { kind, needs_one_answer: false }])
    }

    fn the_one(kind: ChoiceKind) -> ChoicesRead {
        ChoicesRead(vec![ChoiceRead { kind, needs_one_answer: true }])
    }

    pub fn kinds(&self) -> impl Iterator<Item = ChoiceKind> + '_ {
        self.0.iter().map(|read| read.kind)
    }

    /// The names read as one answer, which a walk's several cannot give.
    pub fn kinds_needing_one_answer(&self) -> impl Iterator<Item = ChoiceKind> + '_ {
        self.0.iter().filter(|read| read.needs_one_answer).map(|read| read.kind)
    }

    /// These names, less `kind`: a walk's own question may read the answers
    /// it has been given so far, which is no read of an earlier step.
    fn other_than(mut self, kind: ChoiceKind) -> ChoicesRead {
        self.0.retain(|read| read.kind != kind);
        self
    }

    /// The same names, each needing one answer: a player word's.
    fn each_needing_one_answer(mut self) -> ChoicesRead {
        self.0.iter_mut().for_each(|read| read.needs_one_answer = true);
        self
    }
}

impl BitOr for ChoicesRead {
    type Output = ChoicesRead;
    fn bitor(mut self, rhs: ChoicesRead) -> ChoicesRead {
        for read in rhs.0 {
            match self.0.iter_mut().find(|already| already.kind == read.kind) {
                Some(already) => already.needs_one_answer |= read.needs_one_answer,
                None => self.0.push(read),
            }
        }
        self
    }
}

fn all<T>(nodes: impl IntoIterator<Item = T>, reads: impl Fn(T) -> ChoicesRead) -> ChoicesRead {
    nodes.into_iter().fold(ChoicesRead::NONE, |so_far, node| so_far | reads(node))
}

/// What one instruction reads: its verb's own words, and what it acts on.
/// A player verb that takes one player reads the one chosen; a draw and a
/// life loss take each player a walk's answers chose.
pub fn atom_reads(primitive: &Primitive, recipient: &EffectRecipient) -> ChoicesRead {
    let acted_on = match recipient {
        EffectRecipient::ChosenAs(kind) if kind.is_a_player() && !acts_on_each_player(primitive) => {
            recipient_reads(recipient).each_needing_one_answer()
        }
        _ => recipient_reads(recipient),
    };
    primitive_reads(primitive) | acted_on
}

/// Whether the verb acts on every player it is handed ("each opponent
/// loses 1 life", "each player chosen this way draws a card"), where every
/// other player verb takes the one.
pub fn acts_on_each_player(primitive: &Primitive) -> bool {
    matches!(primitive, Primitive::DrawCards(_) | Primitive::LoseLife(_))
}

/// The amounts a choose step reads before it asks: how many it chooses,
/// and what they must add up to. What it chooses among and who chooses are
/// the step's own to find missing (a choice that chooses nothing), where an
/// amount never chosen leaves it no question to ask.
pub fn amounts_a_choice_reads(what: &ValueChoice) -> ChoicesRead {
    match what {
        ValueChoice::Objects { count, total, .. } => {
            count_reads(count) | total.as_ref().map_or(ChoicesRead::NONE, |total| amount_reads(total.is.amount()))
        }
        ValueChoice::Player { .. } | ValueChoice::Color | ValueChoice::CreatureType | ValueChoice::CardType(_) | ValueChoice::Number { .. } => {
            ChoicesRead::NONE
        }
    }
}

pub(crate) fn recipient_reads(recipient: &EffectRecipient) -> ChoicesRead {
    match recipient {
        EffectRecipient::ChosenAs(kind) => ChoicesRead::the(*kind),
        EffectRecipient::Target(among, count) | EffectRecipient::Choose(among, count) => selection_reads(among) | count_reads(count),
        EffectRecipient::Group(members) => all(members, recipient_reads),
        EffectRecipient::FilteredPermanents(among) | EffectRecipient::FilteredObjectsIn(among, _) => filter_reads(among),
        EffectRecipient::ChosenBy(choice) => {
            recipient_reads(&choice.chooser)
                | all(&choice.picks, |pick| match &pick.count {
                    PickCount::Exactly(count) => filter_reads(&pick.filter) | amount_reads(count),
                    PickCount::All => filter_reads(&pick.filter),
                })
        }
        EffectRecipient::Implicit
        | EffectRecipient::Controller
        | EffectRecipient::ThisObject
        | EffectRecipient::Host
        | EffectRecipient::SameInstanceAs(_)
        | EffectRecipient::EachOf(_)
        | EffectRecipient::Triggering(_)
        | EffectRecipient::TriggeringPlayer
        | EffectRecipient::Referred
        | EffectRecipient::AttackedBy(_)
        | EffectRecipient::DefendingPlayer(_) => ChoicesRead::NONE,
    }
}

fn selection_reads(among: &SelectionFilter) -> ChoicesRead {
    match among {
        SelectionFilter::Permanent(filter) => filter_reads(filter),
        SelectionFilter::Another(inner) => selection_reads(inner),
        SelectionFilter::Creature
        | SelectionFilter::Player(_)
        | SelectionFilter::Any
        | SelectionFilter::Spell
        | SelectionFilter::DamageSource => ChoicesRead::NONE,
    }
}

fn count_reads(count: &TargetCount) -> ChoicesRead {
    match count {
        TargetCount::UpToAmount(amount) | TargetCount::ExactlyAmount(amount) => amount_reads(amount),
        TargetCount::Divided(divided) => count_reads(&divided.among) | amount_reads(&divided.total),
        TargetCount::Exactly(_) | TargetCount::UpTo(_) | TargetCount::Between(..) | TargetCount::AnyNumber => ChoicesRead::NONE,
    }
}

fn filter_reads(filter: &ObjectFilter) -> ChoicesRead {
    match filter {
        ObjectFilter::ChosenAs(kind) => ChoicesRead::the(*kind),
        ObjectFilter::WithValue { is, .. } => amount_reads(is.amount()),
        ObjectFilter::ThisTurn(fact) | ObjectFilter::ThisCombat(fact) => object_fact_reads(fact),
        ObjectFilter::ByController(whose) | ObjectFilter::ByOwner(whose) => player_ref_reads(whose),
        ObjectFilter::And(a, b) | ObjectFilter::Or(a, b) => filter_reads(a) | filter_reads(b),
        ObjectFilter::Not(inner) => filter_reads(inner),
        ObjectFilter::All
        | ObjectFilter::ByType(_)
        | ObjectFilter::BySubtype(_)
        | ObjectFilter::BySupertype(_)
        | ObjectFilter::ByColor(_)
        | ObjectFilter::WithCounters { .. }
        | ObjectFilter::Token
        | ObjectFilter::NotSource
        | ObjectFilter::OneOf(_) => ChoicesRead::NONE,
    }
}

pub(crate) fn object_set_reads(set: &ObjectSet) -> ChoicesRead {
    match set {
        ObjectSet::Filter { filter, .. } => filter_reads(filter),
        ObjectSet::SourceOnly | ObjectSet::Host | ObjectSet::Fixed(_) => ChoicesRead::NONE,
    }
}

pub(crate) fn amount_reads(amount: &AmountExpr) -> ChoicesRead {
    match amount {
        AmountExpr::ChosenAs(kind) => ChoicesRead::the_one(*kind),
        AmountExpr::CountOf(over) | AmountExpr::CardTypesAmong(over) => object_set_reads(over),
        AmountExpr::Plus(inner, _) | AmountExpr::Multiply(inner, _) => amount_reads(inner),
        AmountExpr::History(_, history) => history_count_reads(history),
        AmountExpr::X
        | AmountExpr::Fixed(_)
        | AmountExpr::AffectedManaValue
        | AmountExpr::TargetPower
        | AmountExpr::TargetToughness
        | AmountExpr::DamageDealtThisWay
        | AmountExpr::ExcessDamageDealtThisWay
        | AmountExpr::DiscardedThisWay
        | AmountExpr::ReplacedAmount
        | AmountExpr::UnspentMana(_)
        | AmountExpr::DamagePrevented
        | AmountExpr::StartingLifeTotal
        | AmountExpr::ValueOf(..)
        | AmountExpr::SourcePower
        | AmountExpr::SourceCounters(_)
        | AmountExpr::TriggeringAmount
        | AmountExpr::TriggeringPower(_)
        | AmountExpr::TriggeringToughness(_) => ChoicesRead::NONE,
    }
}

pub fn player_ref_reads(player: &PlayerRef) -> ChoicesRead {
    match player {
        PlayerRef::NamedBy(recipient) => recipient_reads(recipient).each_needing_one_answer(),
        PlayerRef::You
        | PlayerRef::Opponent
        | PlayerRef::Owner
        | PlayerRef::Player(_)
        | PlayerRef::DefendingPlayer(_)
        | PlayerRef::AnyDefendingPlayer
        | PlayerRef::AnyPlayer
        | PlayerRef::Named(_) => ChoicesRead::NONE,
    }
}

pub fn condition_reads(condition: &Condition) -> ChoicesRead {
    match condition {
        Condition::Player { fact, .. } | Condition::DefendingPlayer { fact, .. } => player_fact_reads(fact),
        Condition::HostMatches(among) => filter_reads(among),
        Condition::All(clauses) => all(clauses, condition_reads),
        Condition::Not(clause) => condition_reads(clause),
        Condition::ThisTurn(history, _)
        | Condition::LastTurn(history, _)
        | Condition::SinceYourLastTurn(history, _)
        | Condition::ThisGame(history, _) => history_count_reads(history),
        Condition::SpellWasKicked
        | Condition::SourceUntapped
        | Condition::SourceTapped
        | Condition::CounterCount { .. }
        | Condition::CounterCountReached { .. }
        | Condition::ResolvedThisTurn(_)
        | Condition::ModeChosen(_)
        | Condition::CostAnswer(_) => ChoicesRead::NONE,
    }
}

fn player_fact_reads(fact: &PlayerFact) -> ChoicesRead {
    match fact {
        PlayerFact::ControlsPermanent(among) | PlayerFact::CardInGraveyard(among) => filter_reads(among),
        PlayerFact::LifeAtLeast(amount) | PlayerFact::LifeAtMost(amount) => amount_reads(amount),
        PlayerFact::LibraryEmpty | PlayerFact::HandEmpty => ChoicesRead::NONE,
    }
}

fn object_fact_reads(fact: &ObjectFact) -> ChoicesRead {
    match fact {
        ObjectFact::DealtDamageTo { player, .. } => player.as_ref().map_or(ChoicesRead::NONE, player_ref_reads),
        ObjectFact::Attacked
        | ObjectFact::Blocked
        | ObjectFact::WasBlocked
        | ObjectFact::Entered
        | ObjectFact::WasDealtDamage
        | ObjectFact::WasDealtExcessDamage
        | ObjectFact::Died
        | ObjectFact::PutHere { .. }
        | ObjectFact::WasCast
        | ObjectFact::CameUnderControl => ChoicesRead::NONE,
    }
}

fn history_count_reads(history: &HistoryCount) -> ChoicesRead {
    history.of.as_deref().map_or(ChoicesRead::NONE, filter_reads)
}

fn cost_reads(cost: &Cost) -> ChoicesRead {
    match cost {
        Cost::Sacrifice(among, _) | Cost::Discard(among, _) | Cost::ExileFromGraveyard(among, _) => filter_reads(among),
        Cost::Mana(_)
        | Cost::PayLife(_)
        | Cost::TapSelf
        | Cost::UntapSelf
        | Cost::SacrificeSelf
        | Cost::RemoveCounters(..)
        | Cost::AddCounters(..) => ChoicesRead::NONE,
    }
}

fn primitive_reads(primitive: &Primitive) -> ChoicesRead {
    match primitive {
        // What it chooses among, and who chooses. A walk's own name is the
        // answers it has been given so far ("that hasn't been chosen this
        // way"), which the walk opens before its first question; one
        // chooser reading its own name reads an earlier step's.
        Primitive::ChooseValue { kind, what, how } => {
            let among = amounts_a_choice_reads(what)
                | match what {
                    ValueChoice::Objects { among, .. } => selection_reads(among),
                    // "A second player": other than the players those names kept.
                    ValueChoice::Player { other_than, .. } => all(other_than, |kind| ChoicesRead::the(*kind)),
                    ValueChoice::Color | ValueChoice::CreatureType | ValueChoice::CardType(_) | ValueChoice::Number { .. } => ChoicesRead::NONE,
                };
            let chooser = match &how.answered_by {
                AnsweredBy::Player(chooser) => recipient_reads(chooser),
                AnsweredBy::EachPlayerForThemselves | AnsweredBy::AtRandom => ChoicesRead::NONE,
            };
            match how.walk {
                Some(_) => among.other_than(*kind) | chooser,
                None => among | chooser,
            }
        }
        Primitive::Pay { others, .. } => all(others, cost_reads),
        // "Add that much {R}": each color's amount.
        Primitive::ProduceMana(output) => all(&output.mana, |(_, amount)| amount_reads(amount)),
        Primitive::Mill(amount)
        | Primitive::PutTopCardsIntoHand(amount)
        | Primitive::Discard(amount, _)
        | Primitive::GainLife(amount)
        | Primitive::LoseLife(amount)
        | Primitive::SetLifeTotal(amount)
        | Primitive::DrawCards(amount)
        | Primitive::Scry(amount)
        | Primitive::Surveil(amount)
        | Primitive::RemoveCounters(_, amount)
        | Primitive::CreateToken(_, amount)
        | Primitive::DealDamage { amount, .. } => amount_reads(amount),
        Primitive::AddCounters { amount, by, .. } | Primitive::GetCounters { amount, by, .. } => amount_reads(amount) | player_ref_reads(by),
        Primitive::SetPowerToughness(power, toughness, _) | Primitive::ModifyPowerToughness(power, toughness, _) => {
            amount_reads(power) | amount_reads(toughness)
        }
        Primitive::GainControl { to, .. } => player_ref_reads(to),
        // "Until that enchantment leaves": the object it names beside the
        // one it exiles. The event it waits for is a definition of its own.
        Primitive::ExileUntil { refers_to, .. } => refers_to.as_ref().map_or(ChoicesRead::NONE, recipient_reads),
        // A definition it creates outlasts the resolution (the module's
        // note); the rest name nothing chosen in their own words.
        Primitive::GrantAbility(..)
        | Primitive::CreateReplacement(..)
        | Primitive::Restrict(..)
        | Primitive::Copy { .. }
        | Primitive::CreateDelayedTrigger(_)
        | Primitive::Destroy { .. }
        | Primitive::Exile
        | Primitive::Sacrifice
        | Primitive::ReturnToHand
        | Primitive::ReturnToBattlefield { .. }
        | Primitive::PutOnTopOfLibrary
        | Primitive::PutOnBottomOfLibrary
        | Primitive::ShuffleIntoLibrary
        | Primitive::ShuffleLibrary
        | Primitive::LoseGame
        | Primitive::WinGame
        | Primitive::ExtraTurn
        | Primitive::ExtraPhases { .. }
        | Primitive::Regenerate
        | Primitive::RemoveFromCombat
        | Primitive::BecomeBlocked
        | Primitive::RemoveAllDamage
        | Primitive::Tap
        | Primitive::Untap
        | Primitive::CounterSpell
        | Primitive::CounterAbility
        | Primitive::Counter
        | Primitive::Attach
        | Primitive::Fight
        | Primitive::SwitchPowerToughness(_)
        | Primitive::GrantKeywordFlag(..)
        | Primitive::RemoveKeywordFlag(..)
        | Primitive::LoseAbility(..)
        | Primitive::LoseAllAbilities(_)
        | Primitive::ChangeColor(..)
        | Primitive::ChangeType(..) => ChoicesRead::NONE,
    }
}
