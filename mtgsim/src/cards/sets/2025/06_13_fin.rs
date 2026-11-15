//! Final Fantasy (`fin`), released 2025-06-13.

use std::sync::Arc;

use crate::cards::authoring::{creatures_you_control, draw_cards, put_counters, spell_ability, target_player};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{CounterType, Effect, EffectRecipient, SelectionFilter, TargetCount};
use crate::types::mana::{ManaCost, ManaType};

/// Combat Tutorial — {2}{U}
/// Sorcery
///
/// > Target player draws two cards. Put a +1/+1 counter on up to one target
/// > creature you control.
///
/// (Oracle text verified on Scryfall, 2026-11-15.)
///
/// Two instances of "target", the first a player a verb acts on: with that
/// player gone from the game and the creature still a legal target, the
/// spell resolves and the draw is no one's (CR 608.2b).
///
/// # The rulings, and where each is tested
///
/// Scryfall lists none for this card. Its boards are
/// `phase_value_chosen_as_an_effect_resolves_integration_test::a_draw_whose_target_player_has_left_is_no_ones`
/// and `a_draw_goes_to_its_target_player`.
pub fn combat_tutorial() -> Arc<CardData> {
    let text = "Target player draws two cards. Put a +1/+1 counter on up to one target creature you control.";
    let up_to_one_creature_you_control = EffectRecipient::Target(SelectionFilter::Permanent(creatures_you_control()), TargetCount::UpTo(1));
    CardDataBuilder::new("Combat Tutorial")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                draw_cards(target_player(), 2),
                put_counters(up_to_one_creature_you_control, CounterType::PlusOnePlusOne, 1),
            ]),
        ))
        .build()
}
