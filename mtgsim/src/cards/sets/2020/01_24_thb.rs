//! Theros Beyond Death (`thb`), released 2020-01-24.

use std::sync::Arc;

use crate::cards::authoring::{static_replacement, tapped_for_mana};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::PlayerSet;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, ReplacementDef, Rewrite};

/// Nyxbloom Ancient — {4}{G}{G}{G}
/// Enchantment Creature — Elemental, 5/5
///
/// > Trample
/// > If you tap a permanent for mana, it produces three times as much of
/// > that mana instead.
///
/// **The second factor the arm has seen**, and a creature, so the ability
/// exists only while the body does. Registered and unpooled: the same path
/// as Mana Reflection at seven mana, with a 5/5 trample body that would move
/// the combat rows by design.
///
/// # The rulings, and where each is tested
///
/// Its four rulings are Mana Reflection's four with the number changed, so
/// three of them are tested once, above. The fourth is its own: *"The
/// effects of multiple Nyxbloom Ancients are cumulative. For example, if you
/// have two Nyxbloom Ancients on the battlefield, you'll get nine times the
/// original amount and type of mana."* → nine, no prompt
/// (`two_nyxbloom_ancients_produce_nine_times_the_mana`) — and the two cards
/// together are six, the mixed pair the commuting cell was written for
/// (`a_reflection_and_an_ancient_produce_six_times_the_mana`).
pub fn nyxbloom_ancient() -> Arc<CardData> {
    CardDataBuilder::new("Nyxbloom Ancient")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green, ManaType::Green], 4))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elemental))
        .power_toughness(5, 5)
        .keyword_flag(KeywordFlag::Trample)
        .rules_text(
            "Trample\nIf you tap a permanent for mana, it produces three times as much of that mana instead.",
        )
        .ability(static_replacement("If you tap a permanent for mana, it produces three times as much of that mana instead.",
            ReplacementDef::new(
                tapped_for_mana(None),
                PlayerSet::You,
                Rewrite::Amount(AmountRewrite::Multiplier(3)),
            ),
        ))
        .build()
}
