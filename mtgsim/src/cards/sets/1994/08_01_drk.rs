//! The Dark (`drk`), released 1994-08-01.

use std::sync::Arc;

use crate::cards::authoring::{
    activated_ability, basic, becomes_types, create_replacement, create_replacement_from_a_chosen_source, each, land, non, static_ability,
    tapped_for_mana, the_next_damage_from_a_chosen_source, you_control,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, LandType, Subtype, Subtypes};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{PlayerSet, TypeChange};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, GameActionTemplate, ReplacementDef, Rewrite, Rounding, TemplateAmount};

/// Blood Moon — {2}{R}
/// Enchantment
/// Nonbasic lands are Mountains.
/// (Static ability: filter = lands without Basic supertype, SetSubtypes({Mountain}))
pub fn blood_moon() -> Arc<CardData> {
    let mountain_set = Subtypes::from([Subtype::Land(LandType::Mountain)]);

    let nonbasic_land_filter = land().and(non(basic()));

    CardDataBuilder::new("Blood Moon")
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Enchantment)
        .rules_text("Nonbasic lands are Mountains.")
        .ability(static_ability(
            "Nonbasic lands are Mountains.",
            becomes_types(each(nonbasic_land_filter), TypeChange { set_subtypes: Some(mountain_set), ..TypeChange::NONE }),
        ))
        .build()
}

/// Dark Sphere — {0}
///
/// > {T}, Sacrifice this artifact: The next time a source of your choice would
/// > deal damage to you this turn, prevent half that damage, rounded down.
///
/// The prevention half of RD-1's rounding decision, reached from a **registry
/// row** rather than from a static ability — and the board that makes
/// `consume_use`'s order matter: half of 1 rounded down is 0, so the row
/// prevents nothing and CR 609.7b leaves it for the next instance. RD-2's
/// `a_once_prevention_that_prevents_nothing_is_not_used_up_dark_sphere_is_rd_3s`
/// is the fixture this card replaces.
///
/// # The ruling (Scryfall, 2026-09-08), and where it is tested
///
/// - *If two or more of these effects would apply, you apply them sequentially.
///   So if the source would deal 5 damage after two of these abilities have
///   resolved, the first one prevents 2 damage, reducing it to 3 damage, then
///   the second one prevents a further 1 damage, reducing the total damage
///   dealt to 2.* → two rows, each applied once (CR 614.5), and 5 → 3 → 2.
pub fn dark_sphere() -> Arc<CardData> {
    CardDataBuilder::new("Dark Sphere")
        .mana_cost(ManaCost::build(&[], 0))
        .card_type(CardType::Artifact)
        .rules_text(
            "{T}, Sacrifice this artifact: The next time a source of your choice would deal \
             damage to you this turn, prevent half that damage, rounded down.",
        )
        .ability(activated_ability("{T}, Sacrifice this artifact: The next time a source of your choice would deal damage to you this turn, prevent half that damage, rounded down.",
            vec![Cost::TapSelf, Cost::SacrificeSelf],
            create_replacement_from_a_chosen_source(the_next_damage_from_a_chosen_source(
                None,
                Rewrite::Amount(AmountRewrite::PreventHalf(Rounding::Down)),
            ))
            .until_end_of_turn(),
        ))
        .build()
}

/// Deep Water — {U}{U}
/// Enchantment
///
/// > {U}: Until end of turn, if you tap a land you control for mana, it
/// > produces {U} instead of any other type.
///
/// **CR 106.12b's "of a specific type", and the one of its six printed cards
/// that registers whole today.** An activated ability whose effect is Fog's
/// shape — `Primitive::CreateReplacement` until end of turn — with
/// `Instead(ProduceMana { Blue, ReplacedAmount })`: every unit retyped, the
/// amount kept. "A land you control" is the pattern's `source` filter, asked
/// of the tapped permanent at the proposal, and it is on the pattern rather
/// than in `affected_objects` because the event's subject is the player.
///
/// # The rulings, and where each is tested
///
/// - *"The amount of mana produced is unchanged, but it will all be {U}."* →
///   a land producing two of one color produces two blue
///   (`deep_water_keeps_the_amount_and_changes_the_type`), and a restricted
///   unit keeps its restriction with its new color
///   (`a_retyped_restricted_unit_keeps_its_restriction`).
/// - *"Deep Waters affects lands you control when it resolves and any lands
///   you gain control of this turn."* → the filter is evaluated at each
///   proposal, not captured when the row is made: a land that arrives under
///   your control after the activation is retyped
///   (`deep_water_reaches_a_land_gained_after_the_activation`).
///
/// Beside Mana Reflection the two commute — retype-then-double and
/// double-then-retype are one event — so a Forest under both adds {U}{U} with
/// no CR 616.1 prompt (`deep_water_and_mana_reflection_commute`).
pub fn deep_water() -> Arc<CardData> {
    CardDataBuilder::new("Deep Water")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text(
            "{U}: Until end of turn, if you tap a land you control for mana, it produces {U} instead of any other type.",
        )
        .ability(activated_ability(
            "{U}: Until end of turn, if you tap a land you control for mana, it produces {U} instead of any other type.",
            vec![Cost::Mana(ManaCost::build(&[ManaType::Blue], 0))],
            create_replacement(ReplacementDef::new(
                tapped_for_mana(Some(land().and(you_control()))),
                PlayerSet::You,
                Rewrite::Instead(GameActionTemplate::ProduceMana { mana_type: ManaType::Blue, amount: TemplateAmount::ReplacedAmount }),
            ))
            .until_end_of_turn(),
        ))
        .build()
}
