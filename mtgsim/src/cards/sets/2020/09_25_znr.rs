//! Zendikar Rising (`znr`), released 2020-09-25.

use std::sync::Arc;

use crate::cards::authoring::{
    becomes_types, creature, each, has_base_power_and_toughness, land, non, static_ability, this_object, token, you_control,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, LandType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, ObjectSet, TypeChange};
use crate::types::mana::{ManaCost, ManaType};

/// Ashaya, Soul of the Wild — {3}{G}{G}
/// Legendary Creature — Elemental
/// */*
/// Ashaya's power and toughness are each equal to the number of lands you
/// control.
/// Nontoken creatures you control are Forest lands in addition to their
/// other types. (They're still affected by summoning sickness.)
///
/// (Oracle text verified on Scryfall, 2026-09-06.)
///
/// The applies-to dependency (CR 613.8a(b)) on a printed card: applying
/// Ashaya makes creatures nonbasic lands, which changes what Blood Moon
/// applies to, so Blood Moon waits for Ashaya — "if there are nontoken
/// creatures", as the judge answer in `plans/references/` puts it, because
/// the dependency is decided against the board being built. **No ruling
/// covers Ashaya beside Blood Moon**; the expected answer in the tests is
/// derived from the CR and says so: Ashaya first, then Blood Moon makes the
/// creature-lands Mountains that lose their abilities (CR 305.7), Ashaya's
/// own CDA included, so Ashaya is 0/0.
///
/// Two abilities. The first is a CDA (CR 604.3a) and counts lands *as of
/// layer 7a* — after its own second ability has made it a land — which is
/// its 2020-09-25 ruling: "it's affected by its second ability and thus its
/// first ability counts itself". The second lowers to two layer-4 rows,
/// `AddType(Land)` and `AddSubtype(Forest)`, and it is why the pass orders
/// *effects* rather than rows (`board::Kind::Effect`).
///
/// In `stress` only: the shape is Urborg's, measured there, and five mana
/// is a rare cast in a random game.
pub fn ashaya_soul_of_the_wild() -> Arc<CardData> {
    let lands_you_control = AmountExpr::CountOf(ObjectSet::battlefield_filter(land().and(you_control())));
    let nontoken_creatures_you_control = creature().and(you_control()).and(non(token()));

    CardDataBuilder::new("Ashaya, Soul of the Wild")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 3))
        .color(Color::Green)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elemental))
        // `*/*` — the CDA supplies both numbers, in every zone (CR 208.2a).
        .power_toughness(0, 0)
        .rules_text("Ashaya's power and toughness are each equal to the number of lands you control.\nNontoken creatures you control are Forest lands in addition to their other types.")
        .ability(static_ability(
            "Ashaya's power and toughness are each equal to the number of lands you control.",
            has_base_power_and_toughness(this_object(), lands_you_control.clone(), lands_you_control),
        ).characteristic_defining())
        .ability(static_ability(
            "Nontoken creatures you control are Forest lands in addition to their other types.",
            becomes_types(each(nontoken_creatures_you_control), TypeChange::adding(&[CardType::Land], &[Subtype::Land(LandType::Forest)], &[])),
        ))
        .build()
}
