//! Core Set 2021 (`m21`), released 2020-07-03.

use std::sync::Arc;

use crate::cards::authoring::{draw_two_instead, static_replacement};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, Supertype};
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::zones::DrawFilter;

/// Teferi's Ageless Insight — {2}{U}{U}
/// Legendary Enchantment
///
/// > If you would draw a card except the first one you draw in each of your
/// > draw steps, draw two cards instead.
///
/// `Some(DrawFilter::NotFirstInDrawStep)` watches every draw but the first one
/// drawn in its controller's draw step, counted as each is drawn. So beside a
/// Thought Reflection this draws **three** in the draw step: the Reflection
/// doubles the step's one draw, the doubled instruction's first card is the
/// step's first, which this excepts, and its second is the card this doubles.
/// And when Words of Worship takes the step's draw, the next card drawn in that
/// step is its first (CR 702.94a counts the same way), so this excepts it.
///
/// Legendary, so the two-copy board is Thought Reflection's.
/// # The rulings, and where each is tested
///
/// - *"If a spell or ability causes you to put a card into your hand without
///   specifically using the word 'draw,' it's not a card drawn."* →
///   structurally true and asserted: a `ZoneChangeAct` that is not `Drawn`
///   proposes no draw at all.
///   → `a_card_put_into_hand_is_not_drawn_and_no_draw_replacement_sees_it`
/// - *"If two or more replacement effects would apply to a card-drawing event,
///   the player drawing the card chooses the order."* → as Thought Reflection's.
/// - *"Because [it] is legendary, it's unlikely that one player will control
///   two. However, if that happens, each card that player would draw after the
///   first will result in four cards being drawn."* → the legend rule makes this
///   Thought Reflection's test. This card's own board is the one decision 1
///   named: beside a Thought Reflection in the draw step it draws **three**.
///   → `teferi_beside_thought_reflection_draws_three_in_the_draw_step`, with
///   `teferi_excepts_the_draw_steps_first_card` and
///   `teferi_doubles_a_draw_that_is_not_the_draw_steps` either side of it.
pub fn teferis_ageless_insight() -> Arc<CardData> {
    CardDataBuilder::new("Teferi's Ageless Insight")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .supertype(Supertype::Legendary)
        .rules_text(
            "If you would draw a card except the first one you draw in each of your draw steps, draw two cards instead.",
        )
        .ability(static_replacement("If you would draw a card except the first one you draw in each of your draw steps, draw two cards instead.", draw_two_instead(Some(DrawFilter::NotFirstInDrawStep))))
        .build()
}
