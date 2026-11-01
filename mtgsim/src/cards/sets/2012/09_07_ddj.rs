//! Duel Decks: Izzet vs. Golgari (`ddj`), released 2012-09-07.

use std::sync::Arc;

use crate::cards::authoring::you_cast;
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::cost_modification::{CostChange, CostModificationDef};
use crate::types::effects::ObjectFilter;
use crate::types::mana::{ManaCost, ManaType};

/// Goblin Electromancer — {U}{R}
/// Creature — Goblin Wizard, 2/2
///
/// Instant and sorcery spells you cast cost {1} less to cast.
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// The reduction position. Its rulings: the effect "reduces only generic mana
/// in the spell's total cost" (CR 118.7a), and "two Goblin Electromancers
/// will make instant and sorcery spells you cast cost {2} less to cast" —
/// which is also the board on which CR 601.2f's ordering prompt is asked at
/// all, and the reason the stress pool wants this card more than once.
pub fn goblin_electromancer() -> Arc<CardData> {
    CardDataBuilder::new("Goblin Electromancer")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Red], 0))
        .color(Color::Blue)
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Goblin))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(2, 2)
        .rules_text("Instant and sorcery spells you cast cost {1} less to cast.")
        .ability(
            CostModificationDef::spells(
                you_cast(ObjectFilter::ByType(CardType::Instant).or(ObjectFilter::ByType(CardType::Sorcery))),
                CostChange::Reduce(ManaCost::build(&[], 1)),
            )
            .into_ability("Instant and sorcery spells you cast cost {1} less to cast."),
        )
        .build()
}
