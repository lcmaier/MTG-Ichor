//! CR 113.6 — which of an object's abilities function in which zone.
//!
//! > 113.6. Abilities of an instant or sorcery spell usually function only
//! > while that object is on the stack. Abilities of all other objects usually
//! > function only while that object is on the battlefield. The exceptions are
//! > as follows: …
//!
//! **A module rather than a method, because five subsystems ask and none owns
//! it** (`layers-architecture.md` §13d): static-ability registration, the cost
//! pipeline, `replacement::gather`, `restriction::predicate`, and the trigger
//! dispatcher for CR 113.6k. Beside `engine::restriction` for that module's own
//! reason: a rule several subsystems ask cannot live inside one of them.
//!
//! # What ships, and what does not
//!
//! The triage is `CLAUDE.md`'s — an arm the engine cannot apply is worse than a
//! missing one — and §13d decision 4 names a card against each deferral.
//! **113.6e, f, j and m are all `backlog.md` §2.3's**: they are about playing or
//! activating an object from somewhere other than the battlefield and the hand.
//! 113.6n has no deck-construction pass to modify, and 113.6p has no emblem.


use crate::objects::card_data::{AbilityDef, AbilityType};
use crate::types::effects::{Antecedent};
use crate::types::card_types::{CardType, CardTypes};
use crate::types::triggers::{TriggerDef, TriggerEvent, TriggerSubject};
use crate::types::zones::{Zone, ZoneSet, ZoneChangePattern};

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
pub fn functioning_zones(ability: &AbilityDef, types: &CardTypes) -> ZoneSet {
    // CR 113.6a — a CDA functions everywhere. **An agreement, not a mechanism**:
    // `layers::cda` applies CDAs in any zone without asking this, so the arm only
    // keeps a new caller from making 113.6a false (CR 604.3a(3), `CLAUDE.md`).
    if ability.is_characteristic_defining {
        return ZoneSet::ALL;
    }

    // CR 113.6d — what *that particular object* costs functions on the stack,
    // which is `CostSubject`'s split: affinity is `Itself`, Thalia's "creature
    // spells cost {1} more" is `Spells(_)` on the battlefield. Through "as long
    // as", which `as_cost_modification` sees past.
    if let Some((_, def)) = ability.effect.as_cost_modification() {
        if def.applies_to.applies_to_its_own_object() {
            return ZoneSet::STACK;
        }
        return default_zones(types);
    }

    // CR 113.6b — "An ability that states which zones it functions in
    // functions only from those zones" — and 113.6c, which is the same field
    // holding a complement (`ZoneSet::ALL.without(..)`): the
    // `Effect::FunctionsIn` at the ability's root, a static ability's (Wonder)
    // or a triggered ability's (Bridge from Below). A keyword's own rule
    // (flashback in a graveyard, CR 702.34a) and CR 113.6m's inference from a
    // cost or effect place an ability without its text stating a zone, and
    // are arms of their own when they land.
    if let Some(zones) = ability.effect.stated_zones() {
        return zones;
    }

    // CR 113.6k, **derived from the condition**, for a triggered ability that
    // states no zone: a trigger on this object moving out of zone Z functions
    // in Z when CR 603.10a looks back across the move (dies, a card leaving a
    // graveyard), and otherwise where the move can leave it (a discard, in the
    // graveyard); "from anywhere" everywhere it can be, never a
    // leaves-the-battlefield ability (CR 603.6c; Guile); any other object's,
    // on the battlefield. The ability's zones are its conditions' together,
    // and the dispatcher asks each again ([`condition_functions_in`]). What an
    // *effect* says about a zone is CR 113.6m's, not derived.
    if let Some(def) = ability.effect.as_trigger() {
        return trigger_zones(def, types);
    }

    default_zones(types)
}

/// CR 113.6k's derivation for a triggered ability — see the arm above.
fn trigger_zones(def: &TriggerDef, types: &CardTypes) -> ZoneSet {
    let arms = def.condition.events();
    if arms.is_empty() {
        return default_zones(types);
    }
    let mut zones = ZoneSet::EMPTY;
    for arm in arms {
        zones |= condition_zones(arm, types);
    }
    zones
}

/// CR 113.6k for one trigger condition.
///
/// "All zones it can trigger from" is where the object is when the condition
/// is asked. A look-back condition is asked off the frame, in the zone the
/// object left (CR 603.10a). Any other is asked after the event (CR 603.10),
/// where the object went: the destination it names, else any zone the move
/// can leave it in, since a replacement may send it elsewhere. CR 702.29c says
/// so of a trigger on a discard ("whatever zone the card winds up in"), and
/// Yixlid Jailer's second ruling of one on a card milled into a graveyard. A
/// hidden zone is one of them: whether the card triggers there is CR
/// 603.2f's question, of the object (Library of Leng's board, which Future
/// Sight's revealed top card turns around).
fn condition_zones(arm: &TriggerEvent, types: &CardTypes) -> ZoneSet {
    match arm {
        TriggerEvent::ZoneChange {
            subject: TriggerSubject::Object(Antecedent::ThisObject),
            change: ZoneChangePattern { from: Some(zone), to, .. },
            ..
        } => {
            if arm.looks_back() {
                ZoneSet::of(*zone)
            } else {
                to.map_or(ZoneSet::ALL, ZoneSet::of).without(ZoneSet::of(*zone))
            }
        }
        TriggerEvent::ZoneChange { subject: TriggerSubject::Object(Antecedent::ThisObject), change: ZoneChangePattern { from: None, .. }, .. } => {
            ZoneSet::ALL
        }
        _ => default_zones(types),
    }
}

/// CR 113.6k's second sentence as the dispatcher asks it: does this one
/// trigger condition function *here*? An ability functions where any of its
/// conditions does, so [`functions_in`] can say yes for the ability while
/// this says no for one condition of it — Absolver Thrull's "enters" in the
/// exile its haunt condition functions in, which is the CR's own example.
/// `contains` for [`functions_in`]'s reason. A zone the ability's text
/// states, `stated`, answers for every condition of it (CR 113.6b).
pub fn condition_functions_in(stated: Option<ZoneSet>, arm: &TriggerEvent, types: &CardTypes, zone: Zone) -> bool {
    stated.unwrap_or_else(|| condition_zones(arm, types)).contains(zone)
}

/// CR 113.6 as a caller asks it: does `ability` function *here*?
///
/// **A one-line wrapper, and the justification is not that it reads
/// better**: the obvious hand-written
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
/// not noticed. `engineering-practices.md#one-line-wrappers` is the general
/// rule this is an instance of. Whether `ability` is a static ability that
/// functions in `zone` for an object of `types` (CR 113.6): what a gather of
/// static effects asks before it reads one's body.
pub(crate) fn functions_as_static(ability: &AbilityDef, types: &CardTypes, zone: Zone) -> bool {
    ability.ability_type == AbilityType::Static && functions_in(ability, types, zone)
}

pub fn functions_in(ability: &AbilityDef, types: &CardTypes, zone: Zone) -> bool {
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
fn default_zones(types: &CardTypes) -> ZoneSet {
    if types.contains(&CardType::Instant) || types.contains(&CardType::Sorcery) {
        ZoneSet::STACK
    } else {
        ZoneSet::BATTLEFIELD
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::{AbilityType, ActivationRestriction};
    use crate::types::card_types::{LandType, Subtype};
    use crate::types::cost_modification::{CostChange, CostModificationDef};
    use crate::types::effects::{
        PlayerRef, AmountExpr, Condition, Duration, Effect, EffectRecipient, ObjectFilter, PlayerFact, PlayerSet, Primitive,
    };
    use crate::types::ids::new_ability_id;
    use crate::types::mana::ManaCost;

    fn ability(effect: Effect) -> AbilityDef {
        AbilityDef {
            rules_text: "".into(),
            id: new_ability_id(),
            instances: Vec::new(),
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

    fn types_of(kinds: &[CardType]) -> CardTypes {
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
        let wonder = ability(Effect::FunctionsIn(
            ZoneSet::GRAVEYARD,
            Box::new(Effect::Conditional(
                Condition::Player {
                    players: PlayerSet::You,
                    fact: PlayerFact::ControlsPermanent(ObjectFilter::BySubtype(Subtype::Land(LandType::Island))),
                },
                Box::new(anthem()),
            )),
        ));
        let creature = types_of(&[CardType::Creature]);
        assert_eq!(functioning_zones(&wonder, &creature), ZoneSet::GRAVEYARD);
        assert!(functions_in(&wonder, &creature, Zone::Graveyard));
        assert!(
            !functions_in(&wonder, &creature, Zone::Battlefield),
            "CR 113.6b — 'only from those zones', so Wonder on the battlefield grants nothing"
        );
    }

    /// An "as long as" states no zone, so it leaves the default alone: Kird
    /// Ape's shape is a battlefield static with one.
    #[test]
    fn a_condition_that_names_no_zone_leaves_the_default() {
        let kird_ape = ability(Effect::Conditional(
            Condition::Player {
                players: PlayerSet::You,
                fact: PlayerFact::ControlsPermanent(ObjectFilter::BySubtype(Subtype::Land(LandType::Forest))),
            },
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
        let squee_shaped = ability(Effect::FunctionsIn(ZoneSet::GRAVEYARD | ZoneSet::EXILE, Box::new(anthem())));
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
        let grist_shaped = ability(Effect::FunctionsIn(ZoneSet::EVERYWHERE_BUT_BATTLEFIELD, Box::new(anthem())));
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
            .into_ability_while("", Condition::SourceUntapped)
            .effect,
        );
        assert_eq!(
            functioning_zones(&trinisphere_shaped, &types_of(&[CardType::Artifact])),
            ZoneSet::BATTLEFIELD
        );
    }

    /// A triggered ability on this card itself changing zones
    /// (`TriggerSubject::Object(Antecedent::ThisObject)`), "when this card is put into `to` from
    /// `from`", gaining 1 life.
    fn when_this_card_moves(from: Zone, to: Option<Zone>, act: Option<crate::types::zones::ZoneChangeAct>) -> AbilityDef {
        let arm = TriggerEvent::ZoneChange {
            subject: TriggerSubject::Object(Antecedent::ThisObject),
            change: ZoneChangePattern { from: Some(from), to, act },
            by: None,
            owner: None,
            multiplicity: crate::types::triggers::Multiplicity::PerOccurrence,
        };
        ability(Effect::Triggered(std::sync::Arc::new(crate::cards::authoring::whenever(
            arm,
            Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You)),
        ))))
    }

    /// CR 113.6k on a trigger about this card changing zones. A move CR
    /// 603.10a looks back across functions where the card was; any other
    /// functions wherever the move can leave it.
    #[test]
    fn a_trigger_on_this_card_moving_functions_where_it_is_asked() {
        let creature = types_of(&[CardType::Creature]);
        let instant = types_of(&[CardType::Instant]);
        // Dies, and a card leaving a graveyard: the zone left.
        assert_eq!(functioning_zones(&when_this_card_moves(Zone::Battlefield, Some(Zone::Graveyard), None), &creature), ZoneSet::BATTLEFIELD);
        assert_eq!(functioning_zones(&when_this_card_moves(Zone::Graveyard, None, None), &creature), ZoneSet::GRAVEYARD);
        // Into a hand from exile, which all players see: the zone left.
        assert_eq!(functioning_zones(&when_this_card_moves(Zone::Exile, Some(Zone::Hand), None), &creature), ZoneSet::EXILE);
        // Narcomoeba's, from the library: the graveyard it went to (Yixlid
        // Jailer's second ruling).
        assert_eq!(functioning_zones(&when_this_card_moves(Zone::Library, Some(Zone::Graveyard), None), &creature), ZoneSet::GRAVEYARD);
        // Guerrilla Tactics', a discard: wherever the card can wind up (CR
        // 702.29c), the graveyard, Rest in Peace's exile or Library of Leng's
        // library, and never the hand it left. Whether it can trigger in the
        // library is CR 603.2f's question, asked of the object, and not this
        // one (`plans/atomic-tests/supplemental-docs/603-2f-complexity.md`).
        let discard = when_this_card_moves(Zone::Hand, None, Some(crate::types::zones::ZoneChangeAct::Discarded));
        let wound_up = functioning_zones(&discard, &instant);
        assert!(wound_up.contains(Zone::Graveyard) && wound_up.contains(Zone::Exile) && wound_up.contains(Zone::Library));
        assert!(!wound_up.contains(Zone::Hand), "not the zone it left");
    }
}
