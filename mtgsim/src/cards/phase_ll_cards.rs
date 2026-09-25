//! Phase LL — a card in a library or a hand is walked only when something
//! reads it (`layers-architecture.md` §13e).
//!
//! Fixtures, registered nowhere. Each is a clause of a printed card, or a text
//! no card prints that the CR permits, and each is the smallest board one of
//! the phase's claims needs. Oracle texts were verified on Scryfall on
//! 2026-09-25.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, PlaneswalkerType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Condition, Duration, Effect, EffectRecipient, Primitive, TypeChange};
use crate::types::ids::AbilityId;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::zones::ZoneSet;

fn static_ability(effect: Effect) -> AbilityDef {
    AbilityDef {
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect,
    }
}

/// A type change that adds and removes nothing but `add_types` and
/// `add_subtypes`.
fn adding(add_types: Vec<CardType>, add_subtypes: Vec<Subtype>) -> TypeChange {
    TypeChange {
        add_types,
        remove_types: Vec::new(),
        set_types: None,
        add_subtypes,
        remove_subtypes: Vec::new(),
        set_subtypes: None,
        add_supertypes: Vec::new(),
        remove_supertypes: Vec::new(),
        set_supertypes: None,
    }
}

/// Grist, the Hunger Tide — {1}{B}{G}
/// Legendary Planeswalker — Grist, loyalty 3
/// "As long as Grist isn't on the battlefield, it's a 1/1 Insect creature in
/// addition to its other types."
///
/// **Its first ability on its printed frame, and none of its three loyalty
/// abilities**, which wait on `backlog.md` §2.11 (no loyalty ability exists)
/// and, for the −2's reflexive trigger, TR-3. The only printed card whose
/// static ability changes its own card everywhere but the battlefield
/// (Scryfall, two phrasings, 2026-09-25).
///
/// Its rulings are the tests: "Anywhere but on the battlefield, Grist is a
/// Legendary Planeswalker Creature — Grist Insect", so on the stack it is a
/// creature spell (Essence Scatter can counter it, and Thalia, Guardian of
/// Thraben does not tax it), and once it enters it is a planeswalker and
/// nothing else. The clause is a `SourceOnly` row, which reaches Grist only
/// because Grist is a member of every pass wherever it is
/// (`layers::board::source_joins`).
pub fn grist_insect_clause() -> Arc<CardData> {
    CardDataBuilder::new("Grist's Insect Clause")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Green], 1))
        .color(Color::Black)
        .color(Color::Green)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Planeswalker)
        .subtype(Subtype::Planeswalker(PlaneswalkerType::Grist))
        .loyalty(3)
        .rules_text("As long as Grist isn't on the battlefield, it's a 1/1 Insect creature in addition to its other types.")
        .ability(static_ability(Effect::Conditional(
            // CR 113.6c — the ability says where it does not function.
            Condition::SourceInZone(ZoneSet::EVERYWHERE_BUT_BATTLEFIELD),
            Box::new(Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::ChangeType(
                        adding(vec![CardType::Creature], vec![Subtype::Creature(CreatureType::Insect)]),
                        Duration::WhileSourceOnBattlefield,
                    ),
                    EffectRecipient::ThisObject,
                ),
                Effect::Atom(
                    Primitive::SetPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(1), Duration::WhileSourceOnBattlefield),
                    EffectRecipient::ThisObject,
                ),
            ])),
        )))
        .build()
}
