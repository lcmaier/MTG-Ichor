//! Legions (`lgn`), released 2003-02-03.

use std::sync::Arc;

use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};

/// Ridgetop Raptor — {3}{R}
/// Creature — Dinosaur
/// 2/1 Double Strike
pub fn ridgetop_raptor() -> Arc<CardData> {
    CardDataBuilder::new("Ridgetop Raptor")
        .card_type(CardType::Creature)
        .color(Color::Red)
        .mana_cost(ManaCost::build(&[ManaType::Red], 3))
        .power_toughness(2, 1)
        .keyword_flag(KeywordFlag::DoubleStrike)
        .rules_text("Double strike")
        .build()
}
