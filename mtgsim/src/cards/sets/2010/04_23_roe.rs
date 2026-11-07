//! Rise of the Eldrazi (`roe`), released 2010-04-23.

use std::sync::Arc;

use crate::cards::authoring::{creature, draw_cards, gain_life, spell_ability, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::costs::{AdditionalCost, Cost, PaymentAct};
use crate::types::effects::{AmountExpr, Antecedent, Effect, ObjectValue};
use crate::types::mana::{ManaCost, ManaType};

/// Momentous Fall — {2}{G}{G}
/// Instant
/// As an additional cost to cast this spell, sacrifice a creature.
/// You draw cards equal to the sacrificed creature's power, then you gain
/// life equal to its toughness.
///
/// (Oracle text verified on Scryfall, 2026-11-07.)
///
/// A mandatory sacrifice whose object the resolution reads back as it last
/// existed. Beside Omnath, Locus of Mana it is `ATOM-601.2h-003`'s board,
/// where the order of its two costs changes what it draws; no registered
/// card's power or toughness reads what its other cost changes, so no
/// registered game asks that order.
///
/// # The rulings, and where each is tested
///
/// - #1, exactly one creature sacrificed, and none means it can't be cast:
///   `momentous_fall_sacrifices_exactly_one_creature_and_needs_one`.
/// - #2, the sacrifice is paid while it is cast, so it is gone before anyone
///   can respond: `the_spell_reads_the_creature_its_cost_sacrificed`.
/// - #3, the sacrificed creature's last known information is read: the same
///   test, and `sacrificing_omnath_before_or_after_the_mana_draws_a_different_number`.
pub fn momentous_fall() -> Arc<CardData> {
    let sacrificed = |value| AmountExpr::ValueOf(Antecedent::PaidWith(PaymentAct::Sacrificed), value);
    CardDataBuilder::new("Momentous Fall")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 2))
        .color(Color::Green)
        .card_type(CardType::Instant)
        .rules_text(
            "As an additional cost to cast this spell, sacrifice a creature.\nYou draw cards equal to the sacrificed creature's power, then you gain life equal to its toughness.",
        )
        .additional_cost(AdditionalCost::Mandatory(vec![Cost::Sacrifice(creature(), 1)]))
        // The first paragraph is a cost (CR 118.8), the second the spell's one ability.
        .ability(spell_ability(
            "You draw cards equal to the sacrificed creature's power, then you gain life equal to its toughness.",
            Effect::Sequence(vec![draw_cards(you(), sacrificed(ObjectValue::Power)), gain_life(you(), sacrificed(ObjectValue::Toughness))]),
        ))
        .build()
}
