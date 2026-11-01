//! March of the Machine (`mom`), released 2023-04-21.

use std::sync::Arc;

use crate::cards::authoring::{draw_cards, has_base_power_and_toughness, spell_ability, target_creature, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};

/// Zhalfirin Shapecraft — {1}{U}
/// Instant
/// Target creature has base power and toughness 4/3 until end of turn.
/// Draw a card.
///
/// # The rulings, and where each is tested
///
/// In `phase_lb_integration_test`:
///
/// - *"Zhalfirin Shapecraft overwrites all previous effects that set the
///   creature's base power and toughness to specific values. Any power- or
///   toughness-setting effects that start to apply after the ability
///   resolves will overwrite this effect."* → CR 613.7's timestamps within
///   Layer 7b, both ways round
///   (`zhalfirin_shapecraft_and_another_setter_apply_in_timestamp_order`).
/// - *"Effects that modify a creature's power and/or toughness … will apply
///   to the creature no matter when they started to take effect."* → Bull
///   Rush and Inside Out Fixture cast before it still apply after it
///   (`test_layer_ordering_cast_in_reverse_order` and the other orders).
/// - *"Zhalfirin Shapecraft doesn't affect the creature's card types,
///   creature types, or abilities."* → Serra Angel keeps its types, flying
///   and vigilance
///   (`zhalfirin_shapecraft_leaves_types_and_abilities_alone`).
pub fn zhalfirin_shapecraft() -> Arc<CardData> {
    CardDataBuilder::new("Zhalfirin Shapecraft")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text("Target creature has base power and toughness 4/3 until end of turn.\nDraw a card.")
        .ability(spell_ability(
            "Target creature has base power and toughness 4/3 until end of turn.",
            has_base_power_and_toughness(target_creature(), 4, 3).until_end_of_turn(),
        ))
        .ability(spell_ability("Draw a card.", draw_cards(you(), 1)))
        .build()
}
