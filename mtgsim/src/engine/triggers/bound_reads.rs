//! What a trigger's def reads off its entry (`triggers-architecture.md` §5.2,
//! item 163). One player's entries of one def go on the stack unasked only
//! when they agree on every fact the def reads; a fact it does not read may
//! differ.
//!
//! Each function below answers for one node of the def: the facts it reads,
//! together with its children's. Every match is exhaustive, so a new leaf does
//! not compile until it says what it reads. A leaf that carries a definition
//! of its own — a replacement, a restriction, a copy — is read as reading
//! everything, which only ever asks.

use std::ops::BitOr;

use crate::types::effects::{
    AmountExpr, Condition, Duration, Effect, EffectRecipient, ObjectFilter, PickCount, PlayerFact, PlayerRef,
    Primitive, Selector,
};
use crate::types::triggers::{TriggerDef, TriggerLimit};

/// A set of the facts of an entry a def reads, each one a column the elision
/// compares. Built like `Channels`: a bit per fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundReads(u8);

impl BoundReads {
    pub const NOTHING: BoundReads = BoundReads(0);
    /// "That object", "that spell": the binding's subject.
    pub const SUBJECT: BoundReads = BoundReads(1 << 0);
    /// "That player".
    pub const PLAYER: BoundReads = BoundReads(1 << 1);
    /// "That many".
    pub const AMOUNT: BoundReads = BoundReads(1 << 2);
    /// "Its power", "its toughness": the subject, and the record its frame
    /// comes from.
    pub const CHARACTERISTICS: BoundReads = BoundReads(1 << 3);
    /// "This object": the source the entry's origin names.
    pub const SOURCE: BoundReads = BoundReads(1 << 4);
    /// "This ability": its CR 603.2h gate and its CR 603.7h count.
    pub const ABILITY: BoundReads = BoundReads(1 << 5);
    /// "That card", "that token": what a delayed trigger refers to (CR
    /// 603.7c).
    pub const REFERRED: BoundReads = BoundReads(1 << 6);
    const EVERYTHING: BoundReads = BoundReads((1 << 7) - 1);

    pub fn contains(self, fact: BoundReads) -> bool {
        self.0 & fact.0 == fact.0
    }
}

impl BitOr for BoundReads {
    type Output = BoundReads;
    fn bitor(self, rhs: BoundReads) -> BoundReads {
        BoundReads(self.0 | rhs.0)
    }
}

impl TriggerDef {
    /// Every fact of its entry this def reads: its effect's, its intervening
    /// "if"'s, and CR 603.2h's gate's.
    pub fn bound_reads(&self) -> BoundReads {
        let gate = match self.limit {
            Some(TriggerLimit::DoThisOnlyOnceEachTurn) => BoundReads::ABILITY,
            Some(TriggerLimit::TriggersOnlyOnceEachTurn | TriggerLimit::FirstTimeEachTurn) | None => BoundReads::NOTHING,
        };
        effect(&self.effect) | self.intervening_if.as_ref().map_or(BoundReads::NOTHING, condition) | gate
    }
}

fn effect(e: &Effect) -> BoundReads {
    match e {
        Effect::Atom(verb, recipient_of) => primitive(verb) | recipient(recipient_of),
        Effect::Sequence(effects) => effects.iter().fold(BoundReads::NOTHING, |reads, e| reads | effect(e)),
        Effect::Conditional(test, inner) => condition(test) | effect(inner),
        Effect::Optional { chooser, effect: inner } => player_ref(chooser) | effect(inner),
        Effect::Remember(inner) => effect(inner),
        Effect::ForEach(over, inner) => selector(over) | effect(inner),
        Effect::Repeat(times, inner) => amount(times) | effect(inner),
        // CR 603.3c's modes are chosen at placement, so a modal def is never
        // elided. The other four are static abilities' bodies.
        Effect::Modal { .. }
        | Effect::Replacement(_)
        | Effect::Restriction(_)
        | Effect::CostModification(_)
        | Effect::Triggered(_) => BoundReads::EVERYTHING,
    }
}

fn recipient(r: &EffectRecipient) -> BoundReads {
    match r {
        // An instance is never elided, and "you" is one player across the
        // entries the elision compares.
        EffectRecipient::Implicit
        | EffectRecipient::Controller
        | EffectRecipient::Target(..)
        | EffectRecipient::Choose(..)
        | EffectRecipient::SameInstanceAs(_)
        | EffectRecipient::EachOf(_) => BoundReads::NOTHING,
        EffectRecipient::ThisObject | EffectRecipient::Host => BoundReads::SOURCE,
        EffectRecipient::TriggeringObject => BoundReads::SUBJECT,
        EffectRecipient::TriggeringPlayer => BoundReads::PLAYER,
        EffectRecipient::Referred => BoundReads::REFERRED,
        EffectRecipient::FilteredPermanents(among) | EffectRecipient::FilteredObjectsIn(among, _) => filter(among),
        EffectRecipient::ChosenBy(choice) => choice.picks.iter().fold(recipient(&choice.chooser), |reads, pick| {
            let count = match &pick.count {
                PickCount::Exactly(n) => amount(n),
            };
            reads | filter(&pick.filter) | count
        }),
    }
}

fn amount(a: &AmountExpr) -> BoundReads {
    match a {
        AmountExpr::Fixed(_)
        | AmountExpr::X
        | AmountExpr::AffectedManaValue
        | AmountExpr::TargetPower
        | AmountExpr::TargetToughness
        | AmountExpr::DamageDealtThisWay
        | AmountExpr::ReplacedAmount
        | AmountExpr::UnspentMana(_)
        | AmountExpr::DamagePrevented
        | AmountExpr::StartingLifeTotal => BoundReads::NOTHING,
        AmountExpr::CountOf(over) | AmountExpr::CardTypesAmong(over) => selector(over),
        AmountExpr::Plus(inner, _) | AmountExpr::Multiply(inner, _) => amount(inner),
        AmountExpr::SourcePower => BoundReads::SOURCE,
        AmountExpr::TriggeringAmount => BoundReads::AMOUNT,
        AmountExpr::TriggeringPower | AmountExpr::TriggeringToughness => BoundReads::CHARACTERISTICS,
    }
}

fn selector(s: &Selector) -> BoundReads {
    match s {
        Selector::ControlledCreatures => BoundReads::NOTHING,
        Selector::CreaturesInGraveyard(whose) | Selector::CardsInHand(whose) => player_ref(whose),
        Selector::CardsInGraveyard(whose) => whose.as_ref().map_or(BoundReads::NOTHING, player_ref),
        Selector::PermanentsMatching(among) => filter(among),
    }
}

fn player_ref(p: &PlayerRef) -> BoundReads {
    match p {
        PlayerRef::You | PlayerRef::Opponent | PlayerRef::Player(_) => BoundReads::NOTHING,
        PlayerRef::Owner => BoundReads::SOURCE,
    }
}

fn filter(f: &ObjectFilter) -> BoundReads {
    match f {
        ObjectFilter::All
        | ObjectFilter::ByType(_)
        | ObjectFilter::BySubtype(_)
        | ObjectFilter::BySupertype(_)
        | ObjectFilter::ByColor(_)
        | ObjectFilter::PowerLE(_)
        | ObjectFilter::Token
        | ObjectFilter::OtherThanInstance(_) => BoundReads::NOTHING,
        ObjectFilter::ByController(whose) | ObjectFilter::ByOwner(whose) => player_ref(whose),
        ObjectFilter::NotSource => BoundReads::SOURCE,
        ObjectFilter::And(a, b) | ObjectFilter::Or(a, b) => filter(a) | filter(b),
        ObjectFilter::Not(inner) => filter(inner),
    }
}

fn condition(c: &Condition) -> BoundReads {
    match c {
        Condition::Player { fact, .. } => match fact {
            PlayerFact::ControlsPermanent(among) | PlayerFact::CardInGraveyard(among) => filter(among),
            PlayerFact::LifeAtLeast(n) | PlayerFact::LifeAtMost(n) => amount(n),
            PlayerFact::LibraryEmpty => BoundReads::NOTHING,
        },
        Condition::SpellWasKicked
        | Condition::SourceInZone(_)
        | Condition::SourceUntapped
        | Condition::SourceTapped
        | Condition::SourceHasCounters { .. } => BoundReads::SOURCE,
        Condition::HostMatches(among) => BoundReads::SOURCE | filter(among),
        Condition::All(all) => all.iter().fold(BoundReads::NOTHING, |reads, c| reads | condition(c)),
        Condition::ResolvedThisTurn(_) => BoundReads::ABILITY,
        // A mode makes the def modal, which reads everything; a history and
        // the walk's answer are one player's, the same for each entry.
        Condition::ModeChosen(_)
        | Condition::ThisTurn(_)
        | Condition::LastTurn(_)
        | Condition::SinceYourLastTurn(_)
        | Condition::ThisGame(_)
        | Condition::CostAnswer(_) => BoundReads::NOTHING,
    }
}

fn primitive(p: &Primitive) -> BoundReads {
    match p {
        // The return watches the source, and CR 610.3a/b asks whether it has
        // already left: two sources' exiles differ in whose leaving returns.
        Primitive::ExileUntil { .. } => BoundReads::SOURCE,
        Primitive::Destroy
        | Primitive::Exile
        | Primitive::Sacrifice
        | Primitive::ReturnToHand
        | Primitive::ReturnToBattlefield(_)
        | Primitive::PutOnTopOfLibrary
        | Primitive::PutOnBottomOfLibrary
        | Primitive::ShuffleIntoLibrary
        | Primitive::ShuffleLibrary
        | Primitive::LoseGame
        | Primitive::WinGame
        | Primitive::ExtraTurn
        | Primitive::ExtraPhases(_)
        | Primitive::Regenerate
        | Primitive::RemoveFromCombat
        | Primitive::RemoveAllDamage
        | Primitive::Tap
        | Primitive::Untap
        | Primitive::CounterSpell
        | Primitive::CounterAbility => BoundReads::NOTHING,
        Primitive::Mill(n)
        | Primitive::PutTopCardsIntoHand(n)
        | Primitive::Discard(n, _)
        | Primitive::GainLife(n)
        | Primitive::LoseLife(n)
        | Primitive::SetLifeTotal(n)
        | Primitive::DrawCards(n)
        | Primitive::Scry(n)
        | Primitive::Surveil(n)
        | Primitive::RemoveCounters(_, n)
        | Primitive::CreateToken(_, n) => amount(n),
        Primitive::AddCounters { amount: n, by, .. } | Primitive::GetCounters { amount: n, by, .. } => {
            amount(n) | player_ref(by)
        }
        // The source deals the damage and makes the mana, and lifelink,
        // deathtouch, protection and a mana restriction read it.
        Primitive::DealDamage { amount: n, .. } => BoundReads::SOURCE | amount(n),
        Primitive::ProduceMana(_) | Primitive::Attach | Primitive::Fight => BoundReads::SOURCE,
        Primitive::SetPowerToughness(p, t, lasts) | Primitive::ModifyPowerToughness(p, t, lasts) => {
            amount(p) | amount(t) | duration(lasts)
        }
        Primitive::SwitchPowerToughness(lasts)
        | Primitive::GrantKeywordFlag(_, lasts)
        | Primitive::RemoveKeywordFlag(_, lasts)
        | Primitive::GrantAbility(_, lasts)
        | Primitive::LoseAbility(_, lasts)
        | Primitive::LoseAllAbilities(lasts)
        | Primitive::ChangeColor(_, lasts)
        | Primitive::ChangeType(_, lasts)
        | Primitive::GainControl(lasts) => duration(lasts),
        Primitive::CreateReplacement(..)
        | Primitive::Restrict(..)
        | Primitive::Copy { .. }
        | Primitive::CreateDelayedTrigger(_) => BoundReads::EVERYTHING,
    }
}

fn duration(d: &Duration) -> BoundReads {
    match d {
        Duration::UntilEndOfTurn | Duration::UntilYourNextTurn | Duration::Indefinite => BoundReads::NOTHING,
        Duration::WhileSourceOnBattlefield | Duration::WhileEnchanted | Duration::WhileEquipped => BoundReads::SOURCE,
    }
}

#[cfg(test)]
mod tests {
    use super::BoundReads;
    use crate::cards::authoring::{enters, leaves_the_battlefield, whenever};
    use crate::types::card_types::CardType;
    use crate::types::effects::{Effect, EffectRecipient, ObjectFilter, Primitive, ReturnUnder};
    use crate::types::triggers::{TriggerDef, TriggerSubject};

    /// "Whenever a creature enters, exile it until this leaves the
    /// battlefield": the exile reads the entering creature, and its return
    /// watches the source. Two entries of it from two sources must not be
    /// placed unasked, since the order decides whose leaving returns the
    /// creature (item 232).
    fn exile_that_creature_until_this_leaves() -> TriggerDef {
        whenever(
            enters(TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature))),
            Effect::Atom(
                Primitive::ExileUntil {
                    until: Box::new(leaves_the_battlefield(TriggerSubject::ThisObject).into()),
                    refers_to: None,
                    under: ReturnUnder::Owner,
                },
                EffectRecipient::TriggeringObject,
            ),
        )
    }

    #[test]
    fn an_exile_until_reads_its_source() {
        let reads = exile_that_creature_until_this_leaves().bound_reads();
        assert!(reads.contains(BoundReads::SUBJECT), "the exiled creature");
        assert!(reads.contains(BoundReads::SOURCE), "the source whose leaving returns it");
    }

    /// The control: a plain exile of the same creature reads only it.
    #[test]
    fn a_plain_exile_reads_only_its_object() {
        let def = whenever(
            enters(TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature))),
            Effect::Atom(Primitive::Exile, EffectRecipient::TriggeringObject),
        );
        assert_eq!(def.bound_reads(), BoundReads::SUBJECT);
    }
}
