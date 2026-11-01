//! Fixtures for mana (CR 106 and 605, `mana-architecture.md`).

use std::sync::Arc;

use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, LandType, Subtype};
use crate::types::mana::ManaType;

/// Five-Color Land Fixture
/// Land — Plains Island Swamp Mountain Forest
///
/// **The fuzz decks' any-color land, in Everywhere's place.** Everywhere's
/// shape, the five basic land types and their five intrinsic mana abilities
/// (CR 305.6), on a card a deck can hold: Everywhere is a token
/// (`cards::tokens::everywhere`), and no deck holds a token. `random_deck`
/// fills every land slot it has not assigned with a land that taps for all
/// five colors, and a pool that has one can pay for any card in it; a real
/// "add one mana of any color" is not expressible yet
/// (`backlog.md#219-any-color-mana--add-one-mana-of-any-color`).
///
/// It enters untapped: a tapland would push every deck's curve back a turn.
/// Under Blood Moon, which is pooled, CR 305.7 strips all five abilities and
/// leaves the one a Mountain has.
pub fn five_color_land_fixture() -> Arc<CardData> {
    CardDataBuilder::new("Five-Color Land Fixture")
        .card_type(CardType::Land)
        .subtype(Subtype::Land(LandType::Plains))
        .subtype(Subtype::Land(LandType::Island))
        .subtype(Subtype::Land(LandType::Swamp))
        .subtype(Subtype::Land(LandType::Mountain))
        .subtype(Subtype::Land(LandType::Forest))
        .mana_ability_single(ManaType::White)
        .mana_ability_single(ManaType::Blue)
        .mana_ability_single(ManaType::Black)
        .mana_ability_single(ManaType::Red)
        .mana_ability_single(ManaType::Green)
        .build()
}
