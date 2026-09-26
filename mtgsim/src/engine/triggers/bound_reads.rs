//! What a trigger's def reads off its entry (`triggers-architecture.md` §5.2,
//! item 163). One player's entries of one def go on the stack unasked only
//! when they agree on every fact the def reads; a fact it does not read may
//! differ.
//!
//! Every match here is exhaustive, so a new leaf does not compile until it
//! says what it reads. A leaf that carries a definition of its own — a
//! replacement, a restriction, a copy — is read as reading everything, which
//! only ever asks.

use crate::types::effects::{
    AmountExpr, Condition, Duration, Effect, EffectRecipient, ObjectFilter, PickCount, PlayerFact, PlayerRef,
    Primitive, Selector,
};
use crate::types::triggers::{TriggerDef, TriggerLimit};

/// The facts of an entry a def reads, each one a column the elision compares.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoundReads {
    /// "That object", "that spell": the binding's subject.
    pub subject: bool,
    /// "That player".
    pub player: bool,
    /// "That many".
    pub amount: bool,
    /// "Its power", "its toughness": the subject, and the record its frame
    /// comes from.
    pub characteristics: bool,
    /// "This object": the source the entry's origin names.
    pub source: bool,
    /// "This ability": its CR 603.2h gate and its CR 603.7h count.
    pub ability: bool,
}

impl BoundReads {
    const EVERYTHING: BoundReads =
        BoundReads { subject: true, player: true, amount: true, characteristics: true, source: true, ability: true };
}

impl TriggerDef {
    /// Every fact of its entry this def reads: its effect's, its intervening
    /// "if"'s, and CR 603.2h's gate's.
    pub fn bound_reads(&self) -> BoundReads {
        let mut out = BoundReads::default();
        effect(&self.effect, &mut out);
        if let Some(intervening_if) = &self.intervening_if {
            condition(intervening_if, &mut out);
        }
        match self.limit {
            Some(TriggerLimit::DoThisOnlyOnceEachTurn) => out.ability = true,
            Some(TriggerLimit::TriggersOnlyOnceEachTurn | TriggerLimit::FirstTimeEachTurn) | None => {}
        }
        out
    }
}

fn effect(e: &Effect, out: &mut BoundReads) {
    match e {
        Effect::Atom(verb, recipient_of) => {
            primitive(verb, out);
            recipient(recipient_of, out);
        }
        Effect::Sequence(effects) => effects.iter().for_each(|e| effect(e, out)),
        Effect::Conditional(test, inner) => {
            condition(test, out);
            effect(inner, out);
        }
        Effect::Optional { chooser, effect: inner } => {
            player_ref(chooser, out);
            effect(inner, out);
        }
        Effect::ForEach(over, inner) => {
            selector(over, out);
            effect(inner, out);
        }
        Effect::Repeat(times, inner) => {
            amount(times, out);
            effect(inner, out);
        }
        // CR 603.3c's modes are chosen at placement, so a modal def is never
        // elided. The other four are static abilities' bodies.
        Effect::Modal { .. }
        | Effect::Replacement(_)
        | Effect::Restriction(_)
        | Effect::CostModification(_)
        | Effect::Triggered(_) => *out = BoundReads::EVERYTHING,
    }
}

fn recipient(r: &EffectRecipient, out: &mut BoundReads) {
    match r {
        // An instance is never elided, and "you" is one player across the
        // entries the elision compares.
        EffectRecipient::Implicit
        | EffectRecipient::Controller
        | EffectRecipient::Target(..)
        | EffectRecipient::Choose(..)
        | EffectRecipient::SameInstanceAs(_)
        | EffectRecipient::EachOf(_) => {}
        EffectRecipient::ThisObject | EffectRecipient::Host => out.source = true,
        EffectRecipient::TriggeringObject => out.subject = true,
        EffectRecipient::TriggeringPlayer => out.player = true,
        EffectRecipient::FilteredPermanents(among) | EffectRecipient::FilteredObjectsIn(among, _) => filter(among, out),
        EffectRecipient::ChosenBy(choice) => {
            recipient(&choice.chooser, out);
            for pick in &choice.picks {
                filter(&pick.filter, out);
                match &pick.count {
                    PickCount::Exactly(n) => amount(n, out),
                }
            }
        }
    }
}

fn amount(a: &AmountExpr, out: &mut BoundReads) {
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
        | AmountExpr::StartingLifeTotal => {}
        AmountExpr::CountOf(over) | AmountExpr::CardTypesAmong(over) => selector(over, out),
        AmountExpr::Plus(inner, _) | AmountExpr::Multiply(inner, _) => amount(inner, out),
        AmountExpr::SourcePower => out.source = true,
        AmountExpr::TriggeringAmount => out.amount = true,
        AmountExpr::TriggeringPower | AmountExpr::TriggeringToughness => out.characteristics = true,
    }
}

fn selector(s: &Selector, out: &mut BoundReads) {
    match s {
        Selector::ControlledCreatures => {}
        Selector::CreaturesInGraveyard(whose) | Selector::CardsInHand(whose) => player_ref(whose, out),
        Selector::CardsInGraveyard(whose) => {
            if let Some(whose) = whose {
                player_ref(whose, out);
            }
        }
        Selector::PermanentsMatching(among) => filter(among, out),
    }
}

fn player_ref(p: &PlayerRef, out: &mut BoundReads) {
    match p {
        PlayerRef::You | PlayerRef::Opponent | PlayerRef::Player(_) => {}
        PlayerRef::Owner => out.source = true,
    }
}

fn filter(f: &ObjectFilter, out: &mut BoundReads) {
    match f {
        ObjectFilter::All
        | ObjectFilter::ByType(_)
        | ObjectFilter::BySubtype(_)
        | ObjectFilter::BySupertype(_)
        | ObjectFilter::ByColor(_)
        | ObjectFilter::PowerLE(_)
        | ObjectFilter::Token
        | ObjectFilter::OtherThanInstance(_) => {}
        ObjectFilter::ByController(whose) | ObjectFilter::ByOwner(whose) => player_ref(whose, out),
        ObjectFilter::NotSource => out.source = true,
        ObjectFilter::And(a, b) | ObjectFilter::Or(a, b) => {
            filter(a, out);
            filter(b, out);
        }
        ObjectFilter::Not(inner) => filter(inner, out),
    }
}

fn condition(c: &Condition, out: &mut BoundReads) {
    match c {
        Condition::Player { fact, .. } => match fact {
            PlayerFact::ControlsPermanent(among) | PlayerFact::CardInGraveyard(among) => filter(among, out),
            PlayerFact::LifeAtLeast(n) | PlayerFact::LifeAtMost(n) => amount(n, out),
            PlayerFact::LibraryEmpty => {}
        },
        Condition::SpellWasKicked | Condition::SourceInZone(_) | Condition::SourceUntapped => out.source = true,
        Condition::HostMatches(among) => {
            out.source = true;
            filter(among, out);
        }
        Condition::All(all) => all.iter().for_each(|c| condition(c, out)),
        Condition::ResolvedThisTurn(_) => out.ability = true,
        // A mode makes the def modal, which reads everything; a history and
        // the walk's answer are one player's, the same for each entry.
        Condition::ModeChosen(_)
        | Condition::ThisTurn(_)
        | Condition::LastTurn(_)
        | Condition::SinceYourLastTurn(_)
        | Condition::ThisGame(_)
        | Condition::CostAnswer(_) => {}
    }
}

fn primitive(p: &Primitive, out: &mut BoundReads) {
    match p {
        Primitive::Destroy
        | Primitive::Exile
        | Primitive::Sacrifice
        | Primitive::ReturnToHand
        | Primitive::ReturnToBattlefield
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
        | Primitive::CounterAbility => {}
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
        | Primitive::CreateToken(_, n) => amount(n, out),
        Primitive::AddCounters { amount: n, by, .. } | Primitive::GetCounters { amount: n, by, .. } => {
            amount(n, out);
            player_ref(by, out);
        }
        // The source deals the damage and makes the mana, and lifelink,
        // deathtouch, protection and a mana restriction read it.
        Primitive::DealDamage { amount: n, .. } => {
            out.source = true;
            amount(n, out);
        }
        Primitive::ProduceMana(_) | Primitive::Attach | Primitive::Fight => out.source = true,
        Primitive::SetPowerToughness(p, t, lasts) | Primitive::ModifyPowerToughness(p, t, lasts) => {
            amount(p, out);
            amount(t, out);
            duration(lasts, out);
        }
        Primitive::SwitchPowerToughness(lasts)
        | Primitive::GrantKeywordFlag(_, lasts)
        | Primitive::RemoveKeywordFlag(_, lasts)
        | Primitive::GrantAbility(_, lasts)
        | Primitive::LoseAbility(_, lasts)
        | Primitive::LoseAllAbilities(lasts)
        | Primitive::ChangeColor(_, lasts)
        | Primitive::ChangeType(_, lasts)
        | Primitive::GainControl(lasts) => duration(lasts, out),
        Primitive::CreateReplacement(..) | Primitive::Restrict(..) | Primitive::Copy(..) => {
            *out = BoundReads::EVERYTHING
        }
    }
}

fn duration(d: &Duration, out: &mut BoundReads) {
    match d {
        Duration::UntilEndOfTurn | Duration::UntilYourNextTurn | Duration::Indefinite => {}
        Duration::WhileSourceOnBattlefield | Duration::WhileEnchanted | Duration::WhileEquipped => out.source = true,
    }
}
