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

use crate::types::history::{HistoryCount, ObjectFact};
use crate::types::effects::{
    AmountExpr, Antecedent, Condition, Duration, Effect, EffectRecipient, ObjectFilter, PickCount, PlayerFact, PlayerRef,
    Primitive, Selector,
};
use crate::types::triggers::{BlockOccurrence, EventRole, Multiplicity, TargetedBy, TriggerDef, TriggerEvent, TriggerLimit, TriggerSubject};

/// A set of the facts of an entry a def reads, each one a column the elision
/// compares, and the event roles of its event it names. Built like
/// `Channels`: a bit per fact, then a bit per [`EventRole`] it names, then a
/// bit per event role it needs exactly one object of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TriggerBindingReads(u32);

impl TriggerBindingReads {
    pub const NOTHING: TriggerBindingReads = TriggerBindingReads(0);
    /// "That creature", "them": an object of the triggering events.
    pub const TRIGGERING_OBJECTS: TriggerBindingReads = TriggerBindingReads(1 << 0);
    /// "That player".
    pub const PLAYER: TriggerBindingReads = TriggerBindingReads(1 << 1);
    /// "That many".
    pub const AMOUNT: TriggerBindingReads = TriggerBindingReads(1 << 2);
    /// "Its power", "its toughness": an object of the triggering event, and
    /// the record its frame comes from.
    pub const CHARACTERISTICS: TriggerBindingReads = TriggerBindingReads(1 << 3);
    /// "This object": the source the entry's origin names.
    pub const SOURCE: TriggerBindingReads = TriggerBindingReads(1 << 4);
    /// "This ability": its CR 603.2h gate and its CR 603.7h count.
    pub const ABILITY: TriggerBindingReads = TriggerBindingReads(1 << 5);
    /// "That card", "that token": what a delayed trigger refers to (CR
    /// 603.7c).
    pub const REFERRED: TriggerBindingReads = TriggerBindingReads(1 << 6);
    /// Every fact, and no event role: a def that reads everything is asked
    /// whatever its event roles are, and an event role named inside a
    /// delayed trigger's or a static ability's body is that body's own.
    const EVERYTHING: TriggerBindingReads = TriggerBindingReads(0x007F);
    /// The bit for the first event role the def names: after the seven
    /// facts, one bit per `EventRole`.
    const FIRST_EVENT_ROLE: u32 = 7;
    /// The bit for the first event role the def needs exactly one object
    /// of: after the bits for the event roles it names, one bit per `EventRole`.
    /// Counted from `EventRole::ALL`, so a new event role moves it and
    /// the two runs of bits never overlap.
    const FIRST_NEEDS_ONE: u32 = Self::FIRST_EVENT_ROLE + EventRole::ALL.len() as u32;

    /// The def names `role` of its event.
    pub fn event_role(role: EventRole) -> TriggerBindingReads {
        TriggerBindingReads(1 << (Self::FIRST_EVENT_ROLE + role as u32))
    }

    /// The def needs exactly one object in `role`'s part, and names
    /// it: "its power", "if it had", "the player it's attacking". "Them"
    /// takes every object in the part, and needs none in particular.
    fn needs_one(role: EventRole) -> TriggerBindingReads {
        TriggerBindingReads::event_role(role) | TriggerBindingReads(1 << (Self::FIRST_NEEDS_ONE + role as u32))
    }

    /// Each event role of its event the def names, which every arm of its
    /// condition must have (`TriggerEvent::event_roles`).
    pub fn event_roles(self) -> impl Iterator<Item = EventRole> {
        EventRole::ALL.into_iter().filter(move |&p| self.contains(TriggerBindingReads::event_role(p)))
    }

    /// Each event role the def needs exactly one object of, which no arm of
    /// its condition may have several of (`TriggerEvent::may_have_several`).
    pub fn event_roles_needing_one(self) -> impl Iterator<Item = EventRole> {
        EventRole::ALL.into_iter().filter(move |&p| self.contains(TriggerBindingReads::needs_one(p)))
    }

    pub fn contains(self, fact: TriggerBindingReads) -> bool {
        self.0 & fact.0 == fact.0
    }
}

impl BitOr for TriggerBindingReads {
    type Output = TriggerBindingReads;
    fn bitor(self, rhs: TriggerBindingReads) -> TriggerBindingReads {
        TriggerBindingReads(self.0 | rhs.0)
    }
}

impl TriggerDef {
    /// Every fact of its entry this def reads: its effect's, its intervening
    /// "if"'s, and CR 603.2h's gate's.
    pub fn trigger_binding_reads(&self) -> TriggerBindingReads {
        let gate = match self.limit {
            Some(TriggerLimit::DoThisOnlyOnceEachTurn) => TriggerBindingReads::ABILITY,
            Some(TriggerLimit::TriggersOnlyOnceEachTurn | TriggerLimit::FirstTimeEachTurn) | None => TriggerBindingReads::NOTHING,
        };
        effect(&self.effect) | self.intervening_if.as_ref().map_or(TriggerBindingReads::NOTHING, condition) | gate
    }

    /// Each event role the def names that some arm of its condition does not
    /// have (`TriggerEvent::event_roles`), or needs exactly one object of
    /// where an arm may have several (`TriggerEvent::may_have_several`): an
    /// instruction that could only ever find nothing there, which is a card
    /// written wrong. Empty for every registered card (`cards::registry`'s
    /// test).
    pub fn event_roles_its_arms_lack(&self) -> Vec<EventRole> {
        let arms = self.condition.events();
        let reads = self.trigger_binding_reads();
        let mut lacking: Vec<EventRole> =
            reads.event_roles().filter(|p| arms.iter().any(|arm| !arm.event_roles().contains(p))).collect();
        for several in reads.event_roles_needing_one().filter(|&p| arms.iter().any(|arm| arm.may_have_several(p))) {
            if !lacking.contains(&several) {
                lacking.push(several);
            }
        }
        lacking
    }

    /// The readings of its event the def asks of an arm that cannot give
    /// them: once per subject on an arm that cannot be read that way; CR
    /// 122.7's threshold on a trigger that is not a counter put read once per
    /// subject, whose records alone keep the counts it compares; a block arm
    /// whose fields no CR 509.3 reading combines; and a side of a block named
    /// by event role where the arm fixes that side to the ability's own
    /// source, which on a second arm of "blocks or becomes blocked by" is
    /// the source itself and not "that creature". Empty for every registered
    /// card (`cards::registry`'s test).
    pub fn readings_its_arms_cannot_give(&self) -> Vec<&'static str> {
        let arms = self.condition.events();
        let mut wrong = Vec::new();
        if arms.iter().any(|arm| arm.multiplicity() == Multiplicity::OncePerSubject && !arm.supports_once_per_subject()) {
            wrong.push("once per subject, on an arm that cannot be read that way");
        }
        let puts_once_per_subject =
            |arm: &TriggerEvent| matches!(arm, TriggerEvent::CountersPutOn { multiplicity: Multiplicity::OncePerSubject, .. });
        if self.intervening_if.as_ref().is_some_and(reads_a_counter_threshold) && !arms.iter().all(puts_once_per_subject) {
            wrong.push("CR 122.7's threshold, on a trigger that is not a counter put read once per subject");
        }
        // "For the first time each turn" asks a record's place in its turn,
        // which a cast, a draw, a gain and a loss have, or the turn's
        // targeting rows, which keep no spell's characteristics.
        if self.limit == Some(TriggerLimit::FirstTimeEachTurn) && !arms.iter().all(can_tell_a_first_time_each_turn) {
            wrong.push("for the first time each turn, on an arm whose record has no place in its turn");
        }
        let named = self.trigger_binding_reads();
        for arm in arms {
            let TriggerEvent::Blocks { blocker, attacker, occurrence, count, alone, .. } = arm else { continue };
            match occurrence {
                BlockOccurrence::Unblocked if *blocker != TriggerSubject::Any || count.is_some() || *alone => {
                    wrong.push("an unblocked creature's trigger that asks about a blocker")
                }
                BlockOccurrence::Block if count.is_some() => wrong.push("CR 509.3e's count, on a trigger once per block"),
                _ => {}
            }
            // An arm whose blocker (or attacker) is this creature names that side
            // "this creature". Named by event role instead, it is the other
            // creature on the other arm of "blocks or becomes blocked by", so
            // Cockatrice's "destroy that creature" written as the attacker
            // would destroy Cockatrice itself.
            let names_by_role = |role| named.contains(TriggerBindingReads::event_role(role));
            let own_side_by_role = (*blocker == TriggerSubject::ThisObject && names_by_role(EventRole::Blocker))
                || (*attacker == TriggerSubject::ThisObject && names_by_role(EventRole::Attacker));
            if own_side_by_role {
                wrong.push("its own side of a block, named by event role rather than as this object");
            }
            let other = named.contains(TriggerBindingReads::event_role(EventRole::OtherCombatant));
            if other && arm.role_as_recorded(EventRole::OtherCombatant) == EventRole::OtherCombatant {
                wrong.push("the other creature in a block, on a block arm with no side of its own");
            }
        }
        wrong
    }
}

/// Whether the engine can answer "for the first time each turn" on this arm,
/// which the registry refuses where it cannot. A cast, a draw, a life gain
/// or a life loss carries its place in the turn on its record. A targeting
/// is answered by the turn's targeting rows, which keep the target, who
/// targeted it and whether a spell or an ability did, but no spell's
/// characteristics: "the first time each turn it becomes the target of an
/// Aura spell" has no row to ask, so it is refused until rows keep them.
fn can_tell_a_first_time_each_turn(arm: &TriggerEvent) -> bool {
    match arm {
        TriggerEvent::CastsSpell { .. }
        | TriggerEvent::DrawsCard { .. }
        | TriggerEvent::GainsLife { .. }
        | TriggerEvent::LosesLife { .. } => true,
        TriggerEvent::BecomesTarget { by, .. } => !matches!(by, TargetedBy::Spell(Some(_))),
        _ => false,
    }
}

/// Whether `condition` asks CR 122.7's threshold of the triggering event.
fn reads_a_counter_threshold(condition: &Condition) -> bool {
    match condition {
        Condition::CounterCountReached { .. } => true,
        Condition::All(all) => all.iter().any(reads_a_counter_threshold),
        _ => false,
    }
}

fn effect(e: &Effect) -> TriggerBindingReads {
    match e {
        Effect::Atom(verb, recipient_of) => primitive(verb) | recipient(recipient_of),
        Effect::Sequence(effects) => effects.iter().fold(TriggerBindingReads::NOTHING, |reads, e| reads | effect(e)),
        Effect::Conditional(test, inner) => condition(test) | effect(inner),
        Effect::Optional { chooser, effect: inner } => player_ref(chooser) | effect(inner),
        Effect::Remember(inner) => effect(inner),
        Effect::ForEach(over, inner) => selector(over) | effect(inner),
        Effect::Repeat(times, inner) => amount(times) | effect(inner),
        // CR 603.3c's modes are chosen at placement, so a modal def is never
        // elided, and each mode's event roles are the def's. The other four
        // are static abilities' bodies.
        Effect::Modal { modes, .. } => modes.iter().fold(TriggerBindingReads::EVERYTHING, |reads, e| reads | effect(e)),
        Effect::Replacement(_)
        | Effect::Restriction(_)
        | Effect::CostModification(_)
        | Effect::Triggered(_) => TriggerBindingReads::EVERYTHING,
    }
}

fn recipient(r: &EffectRecipient) -> TriggerBindingReads {
    match r {
        // An instance is never elided, and "you" is one player across the
        // entries the elision compares.
        EffectRecipient::Implicit
        | EffectRecipient::Controller
        | EffectRecipient::Target(..)
        | EffectRecipient::Choose(..)
        | EffectRecipient::SameInstanceAs(_)
        | EffectRecipient::EachOf(_) => TriggerBindingReads::NOTHING,
        EffectRecipient::ThisObject | EffectRecipient::Host => TriggerBindingReads::SOURCE,
        EffectRecipient::Triggering(p) => TriggerBindingReads::TRIGGERING_OBJECTS | TriggerBindingReads::event_role(*p),
        EffectRecipient::TriggeringPlayer => TriggerBindingReads::PLAYER,
        EffectRecipient::AttackedBy(creature) | EffectRecipient::DefendingPlayer(creature) => creature_read(*creature),
        EffectRecipient::Referred => TriggerBindingReads::REFERRED,
        EffectRecipient::Group(members) => members.iter().fold(TriggerBindingReads::NOTHING, |reads, member| reads | recipient(member)),
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
fn creature_read(creature: Antecedent) -> TriggerBindingReads {
    match creature {
        Antecedent::ThisObject => TriggerBindingReads::SOURCE,
        Antecedent::Triggering(p) => TriggerBindingReads::TRIGGERING_OBJECTS | TriggerBindingReads::needs_one(p),
    }
}

fn amount(a: &AmountExpr) -> TriggerBindingReads {
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
        | AmountExpr::StartingLifeTotal => TriggerBindingReads::NOTHING,
        AmountExpr::CountOf(over) | AmountExpr::CardTypesAmong(over) => selector(over),
        AmountExpr::Plus(inner, _) | AmountExpr::Multiply(inner, _) => amount(inner),
        AmountExpr::History(_, count) => history_count(count),
        AmountExpr::SourcePower | AmountExpr::SourceCounters(_) => TriggerBindingReads::SOURCE,
        AmountExpr::TriggeringAmount => TriggerBindingReads::AMOUNT,
        AmountExpr::TriggeringPower(p) | AmountExpr::TriggeringToughness(p) => {
            TriggerBindingReads::CHARACTERISTICS | TriggerBindingReads::needs_one(*p)
        }
    }
}

fn selector(s: &Selector) -> TriggerBindingReads {
    match s {
        Selector::ControlledCreatures => TriggerBindingReads::NOTHING,
        Selector::CreaturesInGraveyard(whose) | Selector::CardsInHand(whose) => player_ref(whose),
        Selector::CardsInGraveyard(whose) => whose.as_ref().map_or(TriggerBindingReads::NOTHING, player_ref),
        Selector::PermanentsMatching(among) => filter(among),
    }
}

fn player_ref(p: &PlayerRef) -> TriggerBindingReads {
    match p {
        PlayerRef::You | PlayerRef::Opponent | PlayerRef::Player(_) | PlayerRef::AnyDefendingPlayer => TriggerBindingReads::NOTHING,
        PlayerRef::Owner => TriggerBindingReads::SOURCE,
        PlayerRef::DefendingPlayer(creature) => creature_read(*creature),
    }
}

fn filter(f: &ObjectFilter) -> TriggerBindingReads {
    match f {
        ObjectFilter::All
        | ObjectFilter::ByType(_)
        | ObjectFilter::BySubtype(_)
        | ObjectFilter::BySupertype(_)
        | ObjectFilter::ByColor(_)
        | ObjectFilter::PowerLE(_)
        | ObjectFilter::WithCounters { .. }
        | ObjectFilter::Token
        | ObjectFilter::OtherThanInstance(_) => TriggerBindingReads::NOTHING,
        ObjectFilter::ThisTurn(fact) | ObjectFilter::ThisCombat(fact) => object_fact(fact),
        ObjectFilter::ByController(whose) | ObjectFilter::ByOwner(whose) => player_ref(whose),
        ObjectFilter::NotSource => TriggerBindingReads::SOURCE,
        ObjectFilter::And(a, b) | ObjectFilter::Or(a, b) => filter(a) | filter(b),
        ObjectFilter::Not(inner) => filter(inner),
    }
}

fn condition(c: &Condition) -> TriggerBindingReads {
    match c {
        Condition::Player { fact, .. } => player_fact(fact),
        Condition::DefendingPlayer { of, fact } => creature_read(*of) | player_fact(fact),
        Condition::SpellWasKicked
        | Condition::SourceInZone(_)
        | Condition::SourceUntapped
        | Condition::SourceTapped
        | Condition::CounterCount { object: Antecedent::ThisObject, .. } => TriggerBindingReads::SOURCE,
        Condition::HostMatches(among) => TriggerBindingReads::SOURCE | filter(among),
        // "It had": the object, as the record it left with framed it.
        Condition::CounterCount { object: Antecedent::Triggering(p), .. } => {
            TriggerBindingReads::CHARACTERISTICS | TriggerBindingReads::needs_one(*p)
        }
        Condition::All(all) => all.iter().fold(TriggerBindingReads::NOTHING, |reads, c| reads | condition(c)),
        Condition::ResolvedThisTurn(_) => TriggerBindingReads::ABILITY,
        // The counts its records left, as "that many" reads their counters.
        Condition::CounterCountReached { .. } => TriggerBindingReads::AMOUNT,
        Condition::ThisTurn(count, _)
        | Condition::LastTurn(count, _)
        | Condition::SinceYourLastTurn(count, _)
        | Condition::ThisGame(count, _) => history_count(count),
        // A mode makes the def modal, which reads everything; the walk's
        // answer is one player's, the same for each entry.
        Condition::ModeChosen(_) | Condition::CostAnswer(_) => TriggerBindingReads::NOTHING,
    }
}

fn object_fact(f: &ObjectFact) -> TriggerBindingReads {
    match f {
        ObjectFact::Attacked | ObjectFact::Blocked | ObjectFact::WasBlocked | ObjectFact::Entered | ObjectFact::WasDealtDamage => {
            TriggerBindingReads::NOTHING
        }
        ObjectFact::DealtDamageTo { player, .. } => player.as_ref().map_or(TriggerBindingReads::NOTHING, player_ref),
    }
}

/// What a history count's filter reads; its rows are one player's, the same
/// for each entry.
fn history_count(count: &HistoryCount) -> TriggerBindingReads {
    count.of.as_deref().map_or(TriggerBindingReads::NOTHING, filter)
}

fn player_fact(f: &PlayerFact) -> TriggerBindingReads {
    match f {
        PlayerFact::ControlsPermanent(among) | PlayerFact::CardInGraveyard(among) => filter(among),
        PlayerFact::LifeAtLeast(n) | PlayerFact::LifeAtMost(n) => amount(n),
        PlayerFact::LibraryEmpty => TriggerBindingReads::NOTHING,
    }
}

fn primitive(p: &Primitive) -> TriggerBindingReads {
    match p {
        // The return watches the source, and CR 610.3a/b asks whether it has
        // already left: two sources' exiles differ in whose leaving returns.
        Primitive::ExileUntil { .. } => TriggerBindingReads::SOURCE,
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
        | Primitive::ExtraPhases { .. }
        | Primitive::Regenerate
        | Primitive::RemoveFromCombat
        | Primitive::RemoveAllDamage
        | Primitive::Tap
        | Primitive::Untap
        | Primitive::CounterSpell
        | Primitive::CounterAbility
        | Primitive::Counter => TriggerBindingReads::NOTHING,
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
        Primitive::DealDamage { amount: n, .. } => TriggerBindingReads::SOURCE | amount(n),
        Primitive::ProduceMana(_) | Primitive::Attach | Primitive::Fight => TriggerBindingReads::SOURCE,
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
        | Primitive::CreateDelayedTrigger(_) => TriggerBindingReads::EVERYTHING,
    }
}

fn duration(d: &Duration) -> TriggerBindingReads {
    match d {
        Duration::UntilEndOfTurn | Duration::UntilYourNextTurn | Duration::Indefinite => TriggerBindingReads::NOTHING,
        Duration::WhileSourceOnBattlefield | Duration::WhileEnchanted | Duration::WhileEquipped => TriggerBindingReads::SOURCE,
    }
}

#[cfg(test)]
mod tests {
    use super::TriggerBindingReads;
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
        let reads = exile_that_creature_until_this_leaves().trigger_binding_reads();
        assert!(reads.contains(TriggerBindingReads::TRIGGERING_OBJECTS), "the exiled creature");
        assert!(reads.contains(TriggerBindingReads::SOURCE), "the source whose leaving returns it");
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
        assert_eq!(def.trigger_binding_reads(), TriggerBindingReads::TRIGGERING_OBJECTS | TriggerBindingReads::event_role(EventRole::Object));
    }
}
