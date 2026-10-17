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
    AmountExpr, Antecedent, Condition, Duration, Effect, EffectRecipient, ObjectFilter, PickCount, PlayerFact, PlayerRef,
    Primitive, Selector,
};
use crate::types::triggers::{EventRole, TriggerDef, TriggerLimit};

/// A set of the facts of an entry a def reads, each one a column the elision
/// compares, and the event roles of its event it names. Built like
/// `Channels`: a bit per fact, then a bit per [`EventRole`] it names, then a
/// bit per event role it needs exactly one object of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundReads(u32);

impl BoundReads {
    pub const NOTHING: BoundReads = BoundReads(0);
    /// "That creature", "them": an object of the triggering events.
    pub const TRIGGERING_OBJECTS: BoundReads = BoundReads(1 << 0);
    /// "That player".
    pub const PLAYER: BoundReads = BoundReads(1 << 1);
    /// "That many".
    pub const AMOUNT: BoundReads = BoundReads(1 << 2);
    /// "Its power", "its toughness": an object of the triggering event, and
    /// the record its frame comes from.
    pub const CHARACTERISTICS: BoundReads = BoundReads(1 << 3);
    /// "This object": the source the entry's origin names.
    pub const SOURCE: BoundReads = BoundReads(1 << 4);
    /// "This ability": its CR 603.2h gate and its CR 603.7h count.
    pub const ABILITY: BoundReads = BoundReads(1 << 5);
    /// "That card", "that token": what a delayed trigger refers to (CR
    /// 603.7c).
    pub const REFERRED: BoundReads = BoundReads(1 << 6);
    /// Every fact, and no event role: a def that reads everything is asked
    /// whatever its event roles are, and an event role named inside a
    /// delayed trigger's or a static ability's body is that body's own.
    const EVERYTHING: BoundReads = BoundReads(0x007F);
    /// The bit for the first event role the def names: after the seven
    /// facts, one bit per `EventRole`.
    const FIRST_EVENT_ROLE: u32 = 7;
    /// The bit for the first event role the def needs exactly one object
    /// of: after the bits for the event roles it names, one bit per `EventRole`.
    /// Counted from `EventRole::ALL`, so a new event role moves it and
    /// the two runs of bits never overlap.
    const FIRST_NEEDS_ONE: u32 = Self::FIRST_EVENT_ROLE + EventRole::ALL.len() as u32;

    /// The def names `role` of its event.
    pub fn event_role(role: EventRole) -> BoundReads {
        BoundReads(1 << (Self::FIRST_EVENT_ROLE + role as u32))
    }

    /// The def needs exactly one object in `role`'s part, and names
    /// it: "its power", "if it had", "the player it's attacking". "Them"
    /// takes every object in the part, and needs none in particular.
    fn needs_one(role: EventRole) -> BoundReads {
        BoundReads::event_role(role) | BoundReads(1 << (Self::FIRST_NEEDS_ONE + role as u32))
    }

    /// Each event role of its event the def names, which every arm of its
    /// condition must have (`TriggerEvent::event_roles`).
    pub fn event_roles(self) -> impl Iterator<Item = EventRole> {
        EventRole::ALL.into_iter().filter(move |&p| self.contains(BoundReads::event_role(p)))
    }

    /// Each event role the def needs exactly one object of, which no arm of
    /// its condition may have several of (`TriggerEvent::may_have_several`).
    pub fn event_roles_needing_one(self) -> impl Iterator<Item = EventRole> {
        EventRole::ALL.into_iter().filter(move |&p| self.contains(BoundReads::needs_one(p)))
    }

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

    /// Each event role the def names that some arm of its condition does not
    /// have (`TriggerEvent::event_roles`), or needs exactly one object of
    /// where an arm may have several (`TriggerEvent::may_have_several`): an
    /// instruction that could only ever find nothing there, which is a card
    /// written wrong. Empty for every registered card (`cards::registry`'s
    /// test).
    pub fn event_roles_its_arms_lack(&self) -> Vec<EventRole> {
        let arms = self.condition.events();
        let reads = self.bound_reads();
        let mut lacking: Vec<EventRole> =
            reads.event_roles().filter(|p| arms.iter().any(|arm| !arm.event_roles().contains(p))).collect();
        for several in reads.event_roles_needing_one().filter(|&p| arms.iter().any(|arm| arm.may_have_several(p))) {
            if !lacking.contains(&several) {
                lacking.push(several);
            }
        }
        lacking
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
        // elided, and each mode's event roles are the def's. The other four
        // are static abilities' bodies.
        Effect::Modal { modes, .. } => modes.iter().fold(BoundReads::EVERYTHING, |reads, e| reads | effect(e)),
        Effect::Replacement(_)
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
        EffectRecipient::Triggering(p) => BoundReads::TRIGGERING_OBJECTS | BoundReads::event_role(*p),
        EffectRecipient::TriggeringPlayer => BoundReads::PLAYER,
        EffectRecipient::AttackedBy(creature) | EffectRecipient::DefendingPlayer(creature) => creature_read(*creature),
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

/// A creature the def names by `Antecedent` and reads a fact of as it
/// is now: its attack.
fn creature_read(creature: Antecedent) -> BoundReads {
    match creature {
        Antecedent::ThisObject => BoundReads::SOURCE,
        Antecedent::Triggering(p) => BoundReads::TRIGGERING_OBJECTS | BoundReads::needs_one(p),
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
        AmountExpr::SourcePower | AmountExpr::SourceCounters(_) => BoundReads::SOURCE,
        AmountExpr::TriggeringAmount => BoundReads::AMOUNT,
        AmountExpr::TriggeringPower(p) | AmountExpr::TriggeringToughness(p) => {
            BoundReads::CHARACTERISTICS | BoundReads::needs_one(*p)
        }
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
        PlayerRef::You | PlayerRef::Opponent | PlayerRef::Player(_) | PlayerRef::AnyDefendingPlayer => BoundReads::NOTHING,
        PlayerRef::Owner => BoundReads::SOURCE,
        PlayerRef::DefendingPlayer(creature) => creature_read(*creature),
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
        | ObjectFilter::WithCounters { .. }
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
        Condition::Player { fact, .. } => player_fact(fact),
        Condition::DefendingPlayer { of, fact } => creature_read(*of) | player_fact(fact),
        Condition::SpellWasKicked
        | Condition::SourceInZone(_)
        | Condition::SourceUntapped
        | Condition::SourceTapped
        | Condition::CounterCount { object: Antecedent::ThisObject, .. } => BoundReads::SOURCE,
        Condition::HostMatches(among) => BoundReads::SOURCE | filter(among),
        // "It had": the object, as the record it left with framed it.
        Condition::CounterCount { object: Antecedent::Triggering(p), .. } => {
            BoundReads::CHARACTERISTICS | BoundReads::needs_one(*p)
        }
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

fn player_fact(f: &PlayerFact) -> BoundReads {
    match f {
        PlayerFact::ControlsPermanent(among) | PlayerFact::CardInGraveyard(among) => filter(among),
        PlayerFact::LifeAtLeast(n) | PlayerFact::LifeAtMost(n) => amount(n),
        PlayerFact::LibraryEmpty => BoundReads::NOTHING,
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
        | Primitive::ReturnToBattlefield { .. }
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
    use crate::types::triggers::{EventRole, TriggerDef, TriggerSubject};

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
                EffectRecipient::Triggering(EventRole::Object),
            ),
        )
    }

    #[test]
    fn an_exile_until_reads_its_source() {
        let reads = exile_that_creature_until_this_leaves().bound_reads();
        assert!(reads.contains(BoundReads::TRIGGERING_OBJECTS), "the exiled creature");
        assert!(reads.contains(BoundReads::SOURCE), "the source whose leaving returns it");
    }

    /// "Whenever a creature dies, destroy the creature it dealt damage to": a
    /// dies record has no damage recipient, and the def says so.
    #[test]
    fn an_event_role_the_arm_lacks_is_named() {
        let def = whenever(
            crate::cards::authoring::dies(TriggerSubject::Any),
            Effect::Atom(Primitive::Destroy, EffectRecipient::Triggering(EventRole::DamageRecipient)),
        );
        assert_eq!(def.event_roles_its_arms_lack(), vec![EventRole::DamageRecipient]);
        assert!(exile_that_creature_until_this_leaves().event_roles_its_arms_lack().is_empty());
    }

    /// The control: a plain exile of the same creature reads only it.
    #[test]
    fn a_plain_exile_reads_only_its_object() {
        let def = whenever(
            enters(TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature))),
            Effect::Atom(Primitive::Exile, EffectRecipient::Triggering(EventRole::Object)),
        );
        assert_eq!(def.bound_reads(), BoundReads::TRIGGERING_OBJECTS | BoundReads::event_role(EventRole::Object));
    }
}
