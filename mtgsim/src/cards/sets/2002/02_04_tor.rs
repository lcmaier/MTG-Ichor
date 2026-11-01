//! Torment (`tor`), released 2002-02-04.

use std::sync::Arc;

use crate::cards::authoring::{becomes_the_target_of, mill, triggered_ability, whenever, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{TargetedBy, TriggerSubject};

/// Cephalid Aristocrat — {4}{U}
/// Creature — Octopus Noble 3/3
///
/// > Whenever this creature becomes the target of a spell or ability, mill
/// > two cards.
///
/// Any spell or ability, its controller's own included, and once for each,
/// however many of its instances chose it (CR 115.9a counts instances only
/// for "with N targets").
///
/// It exercises becoming the target of a spell or ability, once for each,
/// whoever controls it.
///
/// # The rulings
///
/// None (Scryfall, 2026-10-18). Its tests are in `phase_tr5c_integration_test`.
pub fn cephalid_aristocrat() -> Arc<CardData> {
    let text = "Whenever this creature becomes the target of a spell or ability, mill two cards.";
    CardDataBuilder::new("Cephalid Aristocrat")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 4))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Octopus))
        .subtype(Subtype::Creature(CreatureType::Noble))
        .power_toughness(3, 3)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(
                becomes_the_target_of(TriggerSubject::ThisObject, TargetedBy::SpellOrAbility, None),
                mill(you(), 2),
            ),
        ))
        .build()
}
