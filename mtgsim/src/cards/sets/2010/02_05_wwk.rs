//! Worldwake (`wwk`), released 2010-02-05.

use std::sync::Arc;

use crate::cards::authoring::{gets, spell_ability, target_creature};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};

/// Bull Rush — {R}
/// Instant
/// Target creature gets +2/+0 until end of turn.
///
/// # The rulings, and where each is tested
///
/// Scryfall lists none for this card (2026-11-01). Its Layer 7c tests are
/// `phase_lb_integration_test`'s, beside Zhalfirin Shapecraft and Inside Out
/// Fixture in every cast order.
pub fn bull_rush() -> Arc<CardData> {
    CardDataBuilder::new("Bull Rush")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text("Target creature gets +2/+0 until end of turn.")
        .ability(spell_ability(
            "Target creature gets +2/+0 until end of turn.",
            gets(target_creature(), 2, 0).until_end_of_turn(),
        ))
        .build()
}
