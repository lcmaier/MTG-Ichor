//! Planeshift (`pls`), released 2001-02-05.

use std::sync::Arc;

use crate::cards::authoring::{colored, you_cast};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::cost_modification::{CostChange, CostModificationDef};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};

/// Thunderscape Familiar — {1}{R}
/// Creature — Kavu, 1/1
///
/// First strike
/// Black spells and green spells you cast cost {1} less to cast.
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// The other half of CR 601.2h's example. "Black spells **and** green spells"
/// is English "and" over two colors, which is set union — `Or`, the node
/// Root Maze's reading added — and "you cast" is CR 109.5's "you", the
/// source's current controller.
pub fn thunderscape_familiar() -> Arc<CardData> {
    CardDataBuilder::new("Thunderscape Familiar")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Kavu))
        .power_toughness(1, 1)
        .keyword_flag(KeywordFlag::FirstStrike)
        .rules_text("First strike\nBlack spells and green spells you cast cost {1} less to cast.")
        .ability(
            CostModificationDef::spells(
                you_cast(colored(Color::Black).or(colored(Color::Green))),
                CostChange::Reduce(ManaCost::build(&[], 1)),
            )
            .into_ability("Black spells and green spells you cast cost {1} less to cast."),
        )
        .build()
}
