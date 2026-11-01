//! Champions of Kamigawa (`chk`), released 2004-10-01.

use std::sync::Arc;

use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};

/// Isamaru, Hound of Konda - {W}
/// Legendary Creature - Dog
/// 2/2
pub fn isamaru_hound_of_konda() -> Arc<CardData> {
    CardDataBuilder::new("Isamaru, Hound of Konda")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Dog))
        .power_toughness(2, 2)
        .build()
}
