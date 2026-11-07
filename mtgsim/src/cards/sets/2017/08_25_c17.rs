//! Commander 2017 (`c17`), released 2017-08-25.

use std::sync::Arc;

use crate::cards::authoring::{
    choose_one_or_more_each_targeting_a_different_player, chosen_by, creature, dies, discard, draw_cards, lose_life, sacrifice,
    static_replacement, target_opponent, triggered_ability, whenever, you_and_that_player,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{Choice, ChoiceScope, ChoiceSide, DiscardChooser, Pick, PlayerSet};
use crate::types::triggers::TriggerSubject;
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

/// Vindictive Lich — {3}{B}
/// Creature — Zombie Wizard 4/1
/// When this creature dies, choose one or more. Each mode must target a
/// different player.
/// • Target opponent sacrifices a creature of their choice.
/// • Target opponent discards two cards.
/// • Target opponent loses 5 life.
///
/// (Oracle text verified on Scryfall, 2026-11-07.)
///
/// "Each mode must target a different player", the rule of the modal node
/// (item 154's second half), on a dies trigger: at two seats one opponent
/// takes one mode, and at more each mode its own opponent. In Shadrix
/// Silverquill's place, whose third mode waits for `multiplayer-rules`
/// (item 415).
///
/// # The rulings, and where each is tested
///
/// - #1, no more modes than opponents, and at least one:
///   `vindictive_lich_at_two_seats_takes_one_mode`.
/// - #2, a mode for each opponent is not required:
///   `vindictive_lich_at_three_seats_offers_one_mode_or_two`.
pub fn vindictive_lich() -> Arc<CardData> {
    let text = "When this creature dies, choose one or more. Each mode must target a different player.\n• Target opponent sacrifices a creature of their choice.\n• Target opponent discards two cards.\n• Target opponent loses 5 life.";
    let sacrifices = sacrifice(chosen_by(Choice {
        chooser: target_opponent(),
        among: ChoiceScope::ChoosersPermanents,
        picks: vec![Pick::exactly(1, creature())],
        acts_on: ChoiceSide::Chosen,
    }));
    CardDataBuilder::new("Vindictive Lich")
        .mana_cost(ManaCost::build(&[ManaType::Black], 3))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Zombie))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(4, 1)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(
                dies(TriggerSubject::ThisObject),
                choose_one_or_more_each_targeting_a_different_player([
                    sacrifices,
                    discard(target_opponent(), 2, DiscardChooser::Affected),
                    lose_life(target_opponent(), 5),
                ]),
            ),
        ))
        .build()
}
