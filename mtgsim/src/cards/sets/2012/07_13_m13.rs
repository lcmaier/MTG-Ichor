//! Magic 2013 (`m13`), released 2012-07-13.

use std::sync::Arc;

use crate::cards::authoring::static_replacement;
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::PlayerSet;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite};

/// Rhox Faithmender — {3}{W}
/// Creature — Rhino Monk 1/5
///
/// > Lifelink
/// > If you would gain life, you gain twice that much life instead.
///
/// **The first consumer in Phase RE that meets a live proposal without a
/// fixture.** Lifelink's contained `GainLife` has been proposed in every
/// measured game since RB, with nothing watching it; this card is on the
/// battlefield with lifelink of its own, so it doubles the life its own combat
/// damage gains.
///
/// # The rulings, and where each is tested
///
/// - *"If you control two Rhox Faithmenders, life you gain will be multiplied by
///   four. Three ... by eight, and so on."* → **the test RE-3 was built
///   around**, and it failed as an unexpected CR 616.1 *prompt* before it could
///   fail as a number (§11 item 58).
///   → `test_two_rhox_faithmenders_quadruple`, with
///   `three_rhox_faithmenders_multiply_by_eight` for the exponent and
///   `four_is_not_two_applications_of_one_faithmender` for CR 614.6's single
///   modified event.
/// - *"If an effect sets your life total to a specific number, and that number
///   is higher than your current life total, the effect will cause you to gain
///   life equal to the difference ... 3 life and 'becomes 10' ... will actually
///   become 17."* → **no facility**: CR 119.5's set-life-total is
///   `Primitive::SetLifeTotal`, which RE-6 builds. It is the same board as
///   Alhammarret's Archive's first ruling and Skullcrack's fourth, so one
///   primitive closes three (`codebase-state.md` item 123).
pub fn rhox_faithmender() -> Arc<CardData> {
    CardDataBuilder::new("Rhox Faithmender")
        .mana_cost(ManaCost::build(&[ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Rhino))
        .subtype(Subtype::Creature(CreatureType::Monk))
        .power_toughness(1, 5)
        .keyword_flag(KeywordFlag::Lifelink)
        .rules_text("Lifelink\nIf you would gain life, you gain twice that much life instead.")
        .ability(static_replacement("If you would gain life, you gain twice that much life instead.",
            ReplacementDef::new(
                EventPattern::GainLife,
                PlayerSet::You,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            ),
        ))
        .build()
}
