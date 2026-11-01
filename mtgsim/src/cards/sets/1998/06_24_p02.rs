//! Portal Second Age (`p02`), released 1998-06-24.

use std::sync::Arc;

use crate::cards::authoring::{deal_damage, spell_ability, targets};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::SelectionFilter;
use crate::types::mana::{ManaCost, ManaType};

/// Jagged Lightning — {3}{R}{R}
/// Sorcery
/// Jagged Lightning deals 3 damage to each of two target creatures.
///
/// **One instance of "target" holding two choices**, which is the other half of
/// CR 601.2c's first sentence and the half no registered card had: the two
/// creatures must differ, and a board with one creature does not make the spell
/// castable. `ATOM-608.2b-005`'s card, and the cheapest witness that CR 608.2b
/// is per target — a creature that gains protection in response is skipped
/// while the other still takes its three.
///
/// Registered and unpooled: `TargetCount::Exactly(2)` walks the same
/// announcement loop Seeds of Strength already walks, so pooling it would move
/// the stream for no new path (`engineering-practices.md#cards-and-pools`).
///
/// # The rulings, and where each is tested
///
/// Scryfall lists none for this card.
pub fn jagged_lightning() -> Arc<CardData> {
    CardDataBuilder::new("Jagged Lightning")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 3))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text("Jagged Lightning deals 3 damage to each of two target creatures.")
        .ability(spell_ability(
            "Jagged Lightning deals 3 damage to each of two target creatures.",
            deal_damage(targets(2, SelectionFilter::Creature), 3),
        ))
        .build()
}
