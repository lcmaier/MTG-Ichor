//! Cost modification (CR 601.2f, CR 613.11's cost half) — the type surface.
//!
//! A cost modification is a static ability's effect on what a spell costs to
//! cast: "Noncreature spells cost {1} more to cast", "Instant and sorcery
//! spells you cast cost {1} less to cast", Trinisphere. It is discovered the
//! way a replacement effect or a "can't" is — off its source's *effective*
//! ability list, at the moment CR 601.2f determines a total cost — and it is
//! never a registry row: it has no layer and applies to no object
//! (`plans/cost-architecture.md` §3.1). The arithmetic and the order live in
//! `engine::cost_determination`; this module is data, so a card file can
//! write one without reaching into the engine.
//!
//! # The growth contract
//!
//! [`CostSubject`] says *which* spells; it grows one arm per CR 601.2f/602.2b
//! subject the pipeline can determine a cost for — activated abilities
//! (CR 602.2b) are the one arm named and not yet built, and it lands with its
//! first consumer. [`CostChange`] is the three positions of CR 601.2f's order
//! and nothing else; a dynamic amount ("cost {X} less, where X is …") is the
//! reduction position with an [`AmountExpr`], read through
//! `engine::layers::compute::settled_amount` (§3.7). Matched exhaustively, deliberately not
//! `#[non_exhaustive]`: an arm the pipeline cannot apply is a card that
//! silently does nothing (`CLAUDE.md`, the growth contracts).

use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction};
use crate::types::effects::{AmountExpr, Condition, Effect, ObjectFilter};
use crate::types::ids::new_ability_id;
use crate::types::mana::ManaCost;

/// One cost-modifying effect: what it applies to, and what it does.
#[derive(Debug, Clone, PartialEq)]
pub struct CostModificationDef {
    /// Which spells the effect applies to.
    pub applies_to: CostSubject,
    /// What it does to the total cost — and so where in CR 601.2f's order it
    /// is applied.
    pub change: CostChange,
}

impl CostModificationDef {
    /// "[Spells matching `filter`] cost … to cast."
    pub fn spells(filter: ObjectFilter, change: CostChange) -> Self {
        CostModificationDef { applies_to: CostSubject::Spells(filter), change }
    }

    /// "This spell costs … to cast" — the ability is on the object being cast
    /// (CR 113.6d). What [`CardDataBuilder::affinity_for`] writes.
    ///
    /// [`CardDataBuilder::affinity_for`]: crate::objects::card_data::CardDataBuilder::affinity_for
    pub fn itself(change: CostChange) -> Self {
        CostModificationDef { applies_to: CostSubject::Itself, change }
    }

    /// The static ability whose whole text is this modification — what a
    /// card file writes for Thalia or Goblin Electromancer.
    pub fn into_ability(self) -> AbilityDef {
        self.into_ability_body(None)
    }

    /// "As long as [condition], …" — the same ability with the clause on it:
    /// Trinisphere's "as long as this artifact is untapped". The condition
    /// is read against the settled board when a cost is determined.
    pub fn into_ability_while(self, condition: Condition) -> AbilityDef {
        self.into_ability_body(Some(condition))
    }

    fn into_ability_body(self, condition: Option<Condition>) -> AbilityDef {
        let body = Effect::CostModification(Box::new(self));
        AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: match condition {
                Some(condition) => Effect::Conditional(condition, Box::new(body)),
                None => body,
            },
            activation_restriction: ActivationRestriction::None,
            is_characteristic_defining: false,
        }
    }
}

/// Which spells a cost modification applies to.
#[derive(Debug, Clone, PartialEq)]
pub enum CostSubject {
    /// Spells whose characteristics match the filter, read off the spell's
    /// frame — on the stack at CR 601.2f, in hand for the castability
    /// preview — with "you" resolved to the source's *current* controller
    /// (CR 109.5). The spell's own controller is its caster (CR 601.2a), which
    /// is what `ByController(PlayerRef::You)` compares against: "spells you
    /// cast" is the source's controller casting.
    ///
    /// `ObjectFilter` rather than a spell-specific filter because the leaves
    /// are characteristic predicates and a spell has every characteristic a
    /// permanent has (CR 601.2a) — the type was renamed for exactly this
    /// the type was renamed for exactly this consumer (CM-0).
    Spells(ObjectFilter),
    /// **This spell** — an ability on the object being cast, modifying what
    /// *that particular object* costs (CR 113.6d). Affinity is the
    /// population: "Affinity for [text]" means "This spell costs {1} less to
    /// cast for each [text] you control" (CR 702.41a), which is
    /// [`CostChange::ReduceGeneric`] over a count.
    ///
    /// No filter and no controller to resolve — the subject is an identity
    /// test against the spell whose cost is being determined, which only the
    /// spell's own ability list can satisfy (a permanent on the battlefield
    /// is not the object being cast). "You", where the change needs one, is
    /// the spell's controller: its caster (CR 601.2a).
    Itself,
}

/// What a cost modification does to the mana component of a total cost.
///
/// Three arms because CR 601.2f has three positions: increases are added,
/// then reductions are subtracted in the order the player chooses (floored at
/// {0}), then "any effects that directly affect the total cost are applied".
/// A signed delta could not say which position it was.
#[derive(Debug, Clone, PartialEq)]
pub enum CostChange {
    /// "cost {N} more to cast" — a cost increase. The symbols are added to the
    /// mana component as printed: {1} adds one generic, {W} adds a white pip.
    Increase(ManaCost),
    /// "cost {N} less to cast" — a cost reduction, applied under CR 118.7a–d:
    /// generic reduces only generic; a colored or colorless symbol removes its
    /// own pip first and spills to generic when there is none left (118.7b–d);
    /// snow reduces generic (118.7g). Generic saturates at zero at every step
    /// (601.2f).
    ///
    /// Hybrid and Phyrexian symbols are not representable as a reduction yet
    /// — the pipeline refuses them in debug rather than guessing at 118.7e —
    /// and "this effect can't reduce the mana in that cost to less than one
    /// mana" is a clamp every printed carrier puts on an *ability* cost
    /// (`cost-architecture.md` §3.10), so it arrives with that subject.
    Reduce(ManaCost),
    /// "cost {N} less to cast, where N is …" — a reduction of *generic* mana
    /// (CR 118.7a) by an amount read off the board when the cost is
    /// determined. Affinity's shape (CR 702.41a's "{1} less for each [text]
    /// you control" is one generic reduction of that count, since generic
    /// saturates at zero either way), and Golden-Tail Trainer's.
    ///
    /// Evaluated through `engine::layers::compute::settled_amount` against
    /// the finished board CR 613.11 puts cost effects after: `CountOf`
    /// enumerates the real battlefield and "you" is the source's controller
    /// (`cost-architecture.md` §3.7). An amount with no static evaluator
    /// reduces nothing — the assert is `evaluate_amount`'s — rather than
    /// guessing a number.
    ReduceGeneric(AmountExpr),
    /// CR 601.2f's "effects that directly affect the total cost" — the mana
    /// component's mana value is raised to N with generic mana, after every
    /// increase and reduction, and never lowered.
    ///
    /// One printed card: Trinisphere, "each spell that would cost less than
    /// three mana to cast costs three mana to cast", for which the CR's
    /// sentence exists. The arm is the third position in the order, and a
    /// custom card can take it; nothing about it generalizes.
    TotalAtLeast(u8),
}
