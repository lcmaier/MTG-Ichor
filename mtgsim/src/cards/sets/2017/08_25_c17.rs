//! Commander 2017 (`c17`), released 2017-08-25.

use std::sync::Arc;

use crate::cards::authoring::{draw_cards, static_replacement, you_and_that_player};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::PlayerSet;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, ReplacementDef, Rewrite};

/// Alms Collector — {3}{W}
/// Creature — Cat Cleric 3/4
///
/// > Flash
/// > If an opponent would draw two or more cards, instead you and that player
/// > each draw a card.
///
/// **The only printed customer for [`EventPattern::DrawCards`]**, and the card
/// that makes the instruction event necessary rather than tidy: its own ruling
/// says to count how many times the word "draw" is used, so "draw two cards" is
/// one event this watches and two cantrips are two events it does not.
///
/// **The instruction is prevented, and one rider is "you and that player each
/// draw a card"**: one instruction to two players, which CR 121.2c performs
/// the active player's draw first (`PlayerGroup::you_and_that_player`). The
/// rider carries this effect's applied set (CR 614.5), so the draws it makes
/// are modified events this cannot apply to again. That is its second ruling:
/// Thought Reflection doubles the affected player's draw, and this does not
/// come back for it.
///
/// **Flash is not modeled** (`codebase-state.md`'s timing item), and it costs
/// this card's tests nothing: every board here puts it on the battlefield
/// before the draw, which is the only state its replacement reads.
/// # The rulings, and where each is tested
///
/// - *"[Its] replacement effect applies to an instruction to draw more than one
///   card before any replacement effects apply to individual cards drawn."* →
///   `ATOM-616.1g-001`, with Thought Reflection on the other side of the board.
///   → `alms_collector_applies_to_the_instruction_before_thought_reflection_sees_a_draw`
/// - *"Once a replacement effect has been applied to an event, it can't be
///   applied again to the resulting events ... Thought Reflection can double
///   that player's resulting card draw without [this] applying again."* →
///   **the ruling that decides the encoding**, and a test: the rider's draws
///   carry the applied set, so the board is three cards and not a loop.
///   → `alms_collector_does_not_apply_again_to_the_draws_it_produced`
/// - *"To determine whether a player is instructed to draw multiple once or
///   instructed multiple times to draw one card, count how many times the word
///   'draw' is used."* → test. Ancestral Recall (pooled) is one "draw" of three
///   and meets it; two `Primitive::DrawCards(1)` in one resolution are two
///   instructions and do not.
///   → `one_instruction_of_two_is_a_different_event_from_two_instructions_of_one`
/// - *"If an effect puts cards into a player's hand without using the word
///   'draw' at all, [it] doesn't apply."* → the same structural fact as
///   Teferi's first ruling, asserted once there.
/// - *"If two players each control [one] and an effect instructs them to each
///   draw two or more cards, the replacement effect of each ... is applied and
///   both players end up drawing two cards."* → test. Two separate
///   instructions, each meeting the other player's Collector and neither meeting
///   its own controller's. → `two_alms_collectors_facing_each_other_both_draw_two`
/// - *"If two players each control [one] and a third player would draw two or
///   more cards, the third player chooses which ... will apply."* → the
///   four-player test, and the only three-player CR 616.1 prompt reachable from
///   two printed cards. → `a_third_player_chooses_which_alms_collector_applies`
pub fn alms_collector() -> Arc<CardData> {
    CardDataBuilder::new("Alms Collector")
        .mana_cost(ManaCost::build(&[ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Cat))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(3, 4)
        .rules_text(
            "Flash\nIf an opponent would draw two or more cards, instead you and that player each draw a card.",
        )
        .ability(static_replacement("If an opponent would draw two or more cards, instead you and that player each draw a card.",
            ReplacementDef::new(
                EventPattern::DrawCards { at_least: Some(2) },
                PlayerSet::Opponents,
                Rewrite::Prevent,
            )
            .with_then(draw_cards(you_and_that_player(), 1)),
        ))
        .build()
}
