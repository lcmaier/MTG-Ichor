//! Limited Edition Beta (`leb`), released 1993-10-04.

use std::sync::Arc;

use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, LandType, Subtype};
use crate::types::mana::ManaType;

/// Volcanic Island — Land — Island Mountain
pub fn volcanic_island() -> Arc<CardData> {
    CardDataBuilder::new("Volcanic Island")
        .card_type(CardType::Land)
        .subtype(Subtype::Land(LandType::Island))
        .subtype(Subtype::Land(LandType::Mountain))
        .mana_ability_single(ManaType::Blue)
        .mana_ability_single(ManaType::Red)
        .build()
}
