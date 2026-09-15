//! CR 113.6 — which of an object's abilities function in which zone.
//!
//! > 113.6. Abilities of an instant or sorcery spell usually function only
//! > while that object is on the stack. Abilities of all other objects usually
//! > function only while that object is on the battlefield. The exceptions are
//! > as follows: …
//!
//! **A module rather than a method, because four subsystems ask and none owns
//! it** (`layers-architecture.md` §13d). Before LK the rule was in the tree
//! four times and scattered: `register_static_effects` spelled "a static
//! ability functions on the battlefield" as *which function calls it*,
//! `CostSubject::applies_from_battlefield` was CR 113.6d in a method (A5
//! deleted it; the arm below is where its answer lives now), and
//! `replacement::gather` and `restriction::predicate` each swept
//! `battlefield_ids_ordered` for the same unstated reason. This is the one
//! answer they share, and critical-path item 6 adds a fifth caller for
//! CR 113.6k.
//!
//! Beside `engine::restriction` for that module's own reason: a rule several
//! subsystems ask cannot live inside any one of them without the other three
//! reaching across.
//!
//! # What ships, and what does not
//!
//! Six of the fourteen subrules, and the triage is `CLAUDE.md`'s — an arm the
//! engine cannot apply is worse than a missing one. §13d decision 4 is the
//! table with a card named against each deferral; the short version is that
//! **113.6e, f, j and m are all `backlog.md` §2.3's**, because they are about
//! playing or activating an object from somewhere other than the battlefield
//! and `check_cast_legality` hard-codes `Zone::Hand`. 113.6k is item 6's,
//! 113.6n has no deck-construction pass to modify, and 113.6p has no emblem.

use std::collections::HashSet;

use crate::objects::card_data::AbilityDef;
use crate::types::card_types::CardType;
use crate::types::effects::{Condition, Effect};
use crate::types::zones::{Zone, ZoneSet};

/// The zones in which `ability` functions (CR 113.6), for an object whose
/// effective card types are `types`.
///
/// **A set rather than a predicate**, because the caller that matters needs
/// the set: `GameState::register_static_effects` asks "which zones should this
/// be registered in" once per zone change, and `RegistryScopeSummary::
/// reachable_zones` — the affected-side twin of this question — is already a
/// `ZoneSet` union for the same reason (§13c decision 2). [`functions_in`] is
/// the one-line `contains` every other caller asks.
///
/// **`types` is an input rather than a read.** CR 113.6's first sentence
/// splits on instant-or-sorcery, and `CLAUDE.md`'s layer-system invariant says
/// that answer must come from `oracle::characteristics` rather than from
/// `card_data.types` — so this module takes the types it is given and cannot
/// read the wrong ones. The two callers that run before a frame exists pass
/// printed types and say so.
pub fn functioning_zones(ability: &AbilityDef, types: &HashSet<CardType>) -> ZoneSet {
    // CR 113.6a — "Characteristic-defining abilities function everywhere, even
    // outside the game and before the game begins."
    //
    // **An agreement, not a mechanism.** `engine::layers::cda` applies a CDA
    // off the object's own effective ability list at layers 4, 5 and 7a and
    // never consults this function; `compute_non_member` walks CDAs for an
    // object in any zone at all. So 113.6a is already true and this arm exists
    // so that a *new* caller of this predicate cannot accidentally make it
    // false. CR 604.3a(3) is why a CDA is never a registry row in the first
    // place (`CLAUDE.md`).
    if ability.is_characteristic_defining {
        return ZoneSet::ALL;
    }

    // CR 113.6d — "An object's ability that allows a player to pay an
    // alternative cost rather than its mana cost or otherwise modifies what
    // that particular object costs to cast functions on the stack."
    //
    // "That particular object" is the whole of the subrule, and it is exactly
    // what `CostSubject` already distinguishes: affinity is `Itself` and
    // functions on the stack, while Thalia's "creature spells cost {1} more"
    // is `Spells(_)` and functions from the battlefield like any other static
    // ability. Through the "as long as" wrapper, which `as_cost_modification`
    // sees past for us.
    if let Some((_, def)) = ability.effect.as_cost_modification() {
        if def.applies_to.applies_to_its_own_object() {
            return ZoneSet::STACK;
        }
        return default_zones(types);
    }

    // CR 113.6b — "An ability that states which zones it functions in
    // functions only from those zones" — and 113.6c, which is the same field
    // holding a complement (`ZoneSet::ALL.without(..)`). The statement is a
    // clause of the card's text and the clause is a `Condition`; see
    // [`stated_zones`] for why that is the CR's shape rather than a
    // convenience.
    if let Effect::Conditional(condition, _) = &ability.effect {
        if let Some(zones) = stated_zones(condition) {
            return zones;
        }
    }

    default_zones(types)
}

/// CR 113.6 as a caller asks it: does `ability` function *here*?
///
/// **A one-line wrapper with one caller, and the justification is not that it
/// reads better** — an earlier draft of this comment claimed "every caller but
/// registration asks this shape" and there is only the one caller
/// (`register_static_effects`). It is here because the obvious hand-written
/// form is wrong in a way that compiles:
///
/// ```ignore
/// functioning_zones(a, t) == ZoneSet::of(Zone::Graveyard)   // WRONG
/// functioning_zones(a, t).contains(Zone::Graveyard)         // right
/// ```
///
/// **A statement names a `ZoneSet`, not a zone**, and Squee, the Immortal is
/// the printed card that makes the difference bite: "You may cast this card
/// from your graveyard or from exile" (Scryfall, verified 2026-09-14) is one
/// statement over two zones, so `==` answers `false` for both of them. Wonder
/// names one zone today, which is exactly when the bug would be introduced and
/// not noticed. `engineering-practices.md` §2a is the general rule this is an
/// instance of.
pub fn functions_in(ability: &AbilityDef, types: &HashSet<CardType>, zone: Zone) -> bool {
    functioning_zones(ability, types).contains(zone)
}

/// CR 113.6's first sentence: the stack for an instant or sorcery, the
/// battlefield for everything else.
///
/// Read off the object's *types*, not off `ability.ability_type`, because the
/// CR does: a static "this spell can't be countered" on an instant functions
/// on the stack (CR 113.6g) and the default arm is already what says so, which
/// is why 113.6g needs no arm of its own. A card that is both — CR 205.1b
/// forbids it, and Layer 4 cannot produce it from a permanent — would take the
/// stack, which is the honest reading of "an instant or sorcery spell".
fn default_zones(types: &HashSet<CardType>) -> ZoneSet {
    if types.contains(&CardType::Instant) || types.contains(&CardType::Sorcery) {
        ZoneSet::STACK
    } else {
        ZoneSet::BATTLEFIELD
    }
}

/// The zones a condition *states* the ability functions in (CR 113.6b/c), or
/// `None` if it states none.
///
/// # Why the statement lives in the condition
///
/// CR 113.6b is about an ability "that **states** which zones it functions
/// in", and on every card that has one the statement is already a clause with
/// nowhere else to live: Wonder's "as long as this card is in your graveyard"
/// sits beside "and you control an Island", Bridge from Below's is CR 603.4's
/// intervening "if", Leyline of the Void's is the pre-game check
/// `codebase-state.md` item 119 owns. A field on `AbilityDef` would record the
/// conclusion *beside* the clause that states it and the two could disagree
/// (§13d decision 1b, which is where the field was rejected).
///
/// # What this does *not* see: a zone the CR states and the card does not
///
/// CR 113.6b is about the **card's** text, and two other things place an
/// ability without the card saying so:
///
/// - **A keyword's own rule.** Flashback functions in a graveyard because
///   CR 702.34a says so, not because the card prints "from your graveyard".
///   Madness is the same in hand. When those land, the statement is a property
///   of the *keyword* and belongs in an arm above this one, keyed on the
///   keyword rather than on the condition — not in `stated_zones`, which
///   would have nothing to read. (Both are §13d decision 4's deferrals for a
///   different reason: `backlog.md` §2.3 blocks casting from a non-hand zone
///   at all.)
/// - **CR 113.6m's inference**, which reads an ability's *cost or effect* —
///   "exile this card from your graveyard" functions only in a graveyard — and
///   is a fourth thing again. Also deferred, with Reassembling Skeleton.
///
/// So this function answers 113.6b and 113.6c and nothing else, and the arms
/// above it answer the subrules that key on something other than text.
///
/// # The invariant that keeps this bounded
///
/// **A `SourceInZone` leaf is a zone statement only at the top of the
/// condition, or as a direct member of a top-level `All`. Anywhere else it is
/// an ordinary predicate.** Concretely: `Conditional(SourceInZone(GY), body)`
/// and `Conditional(All([SourceInZone(GY), ControlPermanent(Island)]), body)`
/// both state the graveyard; `Conditional(All([All([SourceInZone(GY)])]), body)`
/// states nothing, and neither would a clause under a future `Or` or `Not`.
/// **A card cannot reach the nested form by accident** — there is no card text
/// that wraps a conjunction in a conjunction — so the rule costs nothing to
/// honour and buys the bound.
///
/// That bound is what makes the read syntactic, which is §13c decision 3's
/// requirement for any reach: one level, no search. It is sound today because
/// `Condition` has no `Not` — there is no complement to recover by abstract
/// interpretation, so nothing has to widen to "all zones" to stay sound.
///
/// The match below is exhaustive with no wildcard, so the day `Not` or `Or`
/// lands the compiler stops the build until it answers here. The honest answer
/// for `Not` is `None` — a negated zone clause is not a statement of where the
/// ability functions, and `ZoneSet::without` is how a card says the complement.
fn stated_zones(condition: &Condition) -> Option<ZoneSet> {
    match condition {
        Condition::SourceInZone(zones) => Some(*zones),
        // One clause of a conjunction, and at most one.
        //
        // **This is not a limit of one zone — it is a limit of one
        // *statement*.** Squee, the Immortal ("You may cast this card from
        // your graveyard or from exile", Scryfall 2026-09-14) names two zones
        // and is written `SourceInZone(GRAVEYARD | EXILE)`, one clause, which
        // this returns whole. What the assert catches is two *separate*
        // clauses in a conjunction, which reads "in the graveyard **and** in
        // exile" and is unsatisfiable — an authoring slip where the card
        // wanted the union. Caught rather than silently resolved by taking the
        // first, because taking the first would make Squee half-work.
        Condition::All(clauses) => {
            let mut found: Option<ZoneSet> = None;
            for clause in clauses {
                if let Condition::SourceInZone(zones) = clause {
                    debug_assert!(
                        found.is_none(),
                        "two zone statements on one ability: CR 113.6b names \
                         the zones an ability functions in once, and a second \
                         clause would silently pick one"
                    );
                    found = Some(*zones);
                }
            }
            found
        }
        // Everything else is an ordinary predicate. `CardInGraveyard` is the
        // near miss and stays one on purpose: it is about some *other*
        // object's zone, which places nothing.
        Condition::ControlPermanent(_)
        | Condition::OpponentControlsPermanent(_)
        | Condition::CardInGraveyard(_)
        | Condition::LifeAtLeast(_)
        | Condition::LifeAtMost(_)
        | Condition::HostMatches(_)
        | Condition::SourceUntapped
        | Condition::LibraryEmpty
        | Condition::SpellWasKicked
        | Condition::ModeChosen(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::{AbilityType, ActivationRestriction};
    use crate::types::card_types::{LandType, Subtype};
    use crate::types::cost_modification::{CostChange, CostModificationDef};
    use crate::types::effects::{
        AmountExpr, Duration, EffectRecipient, ObjectFilter, Primitive,
    };
    use crate::types::ids::new_ability_id;
    use crate::types::mana::ManaCost;

    fn ability(effect: Effect) -> AbilityDef {
        AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect,
            activation_restriction: ActivationRestriction::None,
            is_characteristic_defining: false,
        }
    }

    fn anthem() -> Effect {
        Effect::Atom(
            Primitive::ModifyPowerToughness(
                AmountExpr::Fixed(1),
                AmountExpr::Fixed(1),
                Duration::WhileSourceOnBattlefield,
            ),
            EffectRecipient::FilteredPermanents(ObjectFilter::All),
        )
    }

    fn types_of(kinds: &[CardType]) -> HashSet<CardType> {
        kinds.iter().copied().collect()
    }

    /// CR 113.6's first sentence, both halves.
    #[test]
    fn the_default_is_the_battlefield_and_the_stack_for_a_spell() {
        let a = ability(anthem());
        assert_eq!(functioning_zones(&a, &types_of(&[CardType::Creature])), ZoneSet::BATTLEFIELD);
        assert_eq!(functioning_zones(&a, &types_of(&[CardType::Enchantment])), ZoneSet::BATTLEFIELD);
        assert_eq!(functioning_zones(&a, &types_of(&[CardType::Instant])), ZoneSet::STACK);
        assert_eq!(functioning_zones(&a, &types_of(&[CardType::Sorcery])), ZoneSet::STACK);
    }

    /// CR 113.6g needs no arm: "this spell can't be countered" is a static
    /// ability on an instant, and the default arm already puts it on the
    /// stack without reading what it says.
    #[test]
    fn a_static_on_an_instant_functions_on_the_stack_by_the_default_arm() {
        let cant_be_countered = ability(anthem());
        assert!(functions_in(&cant_be_countered, &types_of(&[CardType::Instant]), Zone::Stack));
        assert!(!functions_in(
            &cant_be_countered,
            &types_of(&[CardType::Instant]),
            Zone::Battlefield
        ));
    }

    /// CR 113.6a — everywhere, and the flag is the whole test.
    #[test]
    fn a_cda_functions_everywhere() {
        let mut cda = ability(anthem());
        cda.is_characteristic_defining = true;
        assert_eq!(functioning_zones(&cda, &types_of(&[CardType::Creature])), ZoneSet::ALL);
        // Including on a spell, where the default arm would have said `STACK`
        // and CR 604.3's "even outside the game" says otherwise.
        assert_eq!(functioning_zones(&cda, &types_of(&[CardType::Instant])), ZoneSet::ALL);
        for zone in ZoneSet::ALL.iter() {
            assert!(functions_in(&cda, &types_of(&[CardType::Creature]), zone));
        }
    }

    /// CR 113.6b — Wonder's clause, and the negative half that makes it a
    /// rule rather than a default.
    #[test]
    fn a_stated_zone_replaces_the_default() {
        let wonder = ability(Effect::Conditional(
            Condition::All(vec![
                Condition::SourceInZone(ZoneSet::GRAVEYARD),
                Condition::ControlPermanent(ObjectFilter::BySubtype(Subtype::Land(
                    LandType::Island,
                ))),
            ]),
            Box::new(anthem()),
        ));
        let creature = types_of(&[CardType::Creature]);
        assert_eq!(functioning_zones(&wonder, &creature), ZoneSet::GRAVEYARD);
        assert!(functions_in(&wonder, &creature, Zone::Graveyard));
        assert!(
            !functions_in(&wonder, &creature, Zone::Battlefield),
            "CR 113.6b — 'only from those zones', so Wonder on the battlefield grants nothing"
        );
    }

    /// A condition that states no zone leaves the default alone: Kird Ape's
    /// shape is a battlefield static with an "as long as".
    #[test]
    fn a_condition_that_names_no_zone_leaves_the_default() {
        let kird_ape = ability(Effect::Conditional(
            Condition::ControlPermanent(ObjectFilter::BySubtype(Subtype::Land(LandType::Forest))),
            Box::new(anthem()),
        ));
        assert_eq!(
            functioning_zones(&kird_ape, &types_of(&[CardType::Creature])),
            ZoneSet::BATTLEFIELD
        );
    }

    /// **One statement, two zones** — the case that makes [`functions_in`] a
    /// wrapper rather than a re-spelling. Squee, the Immortal's shape.
    #[test]
    fn a_statement_may_name_more_than_one_zone() {
        let squee_shaped = ability(Effect::Conditional(
            Condition::SourceInZone(ZoneSet::GRAVEYARD | ZoneSet::EXILE),
            Box::new(anthem()),
        ));
        let creature = types_of(&[CardType::Creature]);
        assert!(functions_in(&squee_shaped, &creature, Zone::Graveyard));
        assert!(functions_in(&squee_shaped, &creature, Zone::Exile));
        assert!(!functions_in(&squee_shaped, &creature, Zone::Battlefield));
        // And the shape a caller reaches for instead, which is why the
        // wrapper exists: `==` is right for a one-zone statement and wrong
        // for this one, in both directions.
        assert_ne!(functioning_zones(&squee_shaped, &creature), ZoneSet::GRAVEYARD);
        assert_ne!(functioning_zones(&squee_shaped, &creature), ZoneSet::EXILE);
    }

    /// CR 113.6c — "an ability that states which zones it doesn't function in
    /// functions everywhere except for the specified zones". The same field,
    /// holding the complement `ZoneSet` already spells.
    #[test]
    fn a_stated_complement_is_the_same_field() {
        let grist_shaped = ability(Effect::Conditional(
            Condition::SourceInZone(ZoneSet::EVERYWHERE_BUT_BATTLEFIELD),
            Box::new(anthem()),
        ));
        let creature = types_of(&[CardType::Creature]);
        assert_eq!(functioning_zones(&grist_shaped, &creature), ZoneSet::EVERYWHERE_BUT_BATTLEFIELD);
        assert!(!functions_in(&grist_shaped, &creature, Zone::Battlefield));
        assert!(functions_in(&grist_shaped, &creature, Zone::Graveyard));
        assert!(functions_in(&grist_shaped, &creature, Zone::Exile));
        assert!(functions_in(&grist_shaped, &creature, Zone::Hand));
    }

    /// CR 113.6d — the subject decides, not the card type. Affinity is on a
    /// creature and still functions on the stack; Thalia's is on a creature
    /// and functions from the battlefield.
    #[test]
    fn a_cost_ability_splits_on_its_subject() {
        let creature = types_of(&[CardType::Creature]);

        let affinity = ability(Effect::CostModification(Box::new(CostModificationDef::itself(
            CostChange::ReduceGeneric(AmountExpr::Fixed(1)),
        ))));
        assert_eq!(functioning_zones(&affinity, &creature), ZoneSet::STACK);

        let thalia = ability(Effect::CostModification(Box::new(CostModificationDef::spells(
            ObjectFilter::All,
            CostChange::Increase(ManaCost::build(&[], 1)),
        ))));
        assert_eq!(functioning_zones(&thalia, &creature), ZoneSet::BATTLEFIELD);
    }

    /// The "as long as" wrapper does not hide the subject — `CostSubject` is
    /// read through it, and a cost ability's zone is 113.6d's whatever
    /// condition it carries.
    #[test]
    fn a_conditional_cost_ability_still_splits_on_its_subject() {
        let trinisphere_shaped = ability(
            CostModificationDef::spells(
                ObjectFilter::All,
                CostChange::Increase(ManaCost::build(&[], 1)),
            )
            .into_ability_while(Condition::SourceUntapped)
            .effect,
        );
        assert_eq!(
            functioning_zones(&trinisphere_shaped, &types_of(&[CardType::Artifact])),
            ZoneSet::BATTLEFIELD
        );
    }

    /// The read is **one level deep**, and a clause nested below that states
    /// nothing — the invariant `stated_zones` documents, asserted rather than
    /// left to the comment.
    ///
    /// The board here is `All([All([SourceInZone(GRAVEYARD)])])`, which no
    /// card text produces: a conjunction inside a conjunction is a thing an
    /// author writes by accident or a future transformation produces, never a
    /// thing a card says. The point of pinning it is the *other* direction —
    /// the day `Condition` grows `Not` or `Or`, the rule for where a zone
    /// clause counts is already written down and tested, rather than being
    /// re-derived by whoever adds the combinator.
    #[test]
    fn a_zone_clause_below_the_top_level_states_nothing() {
        let nested = ability(Effect::Conditional(
            Condition::All(vec![Condition::All(vec![Condition::SourceInZone(
                ZoneSet::GRAVEYARD,
            )])]),
            Box::new(anthem()),
        ));
        assert_eq!(
            functioning_zones(&nested, &types_of(&[CardType::Creature])),
            ZoneSet::BATTLEFIELD,
            "one level, by design: a deeper walk is the search §13c decision 3 refuses"
        );
    }

    #[test]
    #[should_panic(expected = "two zone statements on one ability")]
    fn two_zone_statements_on_one_ability_are_loud() {
        let confused = ability(Effect::Conditional(
            Condition::All(vec![
                Condition::SourceInZone(ZoneSet::GRAVEYARD),
                Condition::SourceInZone(ZoneSet::HAND),
            ]),
            Box::new(anthem()),
        ));
        let _ = functioning_zones(&confused, &types_of(&[CardType::Creature]));
    }
}
