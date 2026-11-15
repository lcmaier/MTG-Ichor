//! Fallout (`pip`), released 2024-03-08.

use std::sync::Arc;

use crate::cards::authoring::{
    at_random, choose_objects, creature, destroy, each, gain_control_of, gains_keyword, one_of_the_chosen, spell_ability, the_chosen_objects,
    untap,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{Effect, TargetCount};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};

/// The Nipton Lottery — {2}{B}{R}
/// Sorcery
///
/// > Choose a creature at random. You gain control of that creature until
/// > end of turn. Untap it. It gains haste until end of turn. Then destroy
/// > all other creatures.
///
/// (Oracle text verified on Scryfall, 2026-11-15.)
///
/// A creature chosen at random as the spell resolves (CR 608.2d), nobody
/// asked, then four steps that name it: three act on "that creature" and the
/// last on every other.
///
/// # The rulings, and where each is tested
///
/// - *"The creature chosen at random isn't targeted, so a creature with
///   shroud or an opponent's creature with hexproof can be chosen this way."*
///   (#1) and *"Players can't take actions between the time the creature is
///   randomly chosen and the time at which the rest of the effects occur."*
///   (#2) → `phase_value_chosen_as_an_effect_resolves_integration_test::the_nipton_lottery_acts_on_the_creature_drawn_and_destroys_the_rest`.
pub fn the_nipton_lottery() -> Arc<CardData> {
    let text = "Choose a creature at random. You gain control of that creature until end of turn. Untap it. It gains haste until end of turn. Then destroy all other creatures.";
    CardDataBuilder::new("The Nipton Lottery")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Red], 2))
        .color(Color::Black)
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                choose_objects(at_random(), creature(), TargetCount::Exactly(1)),
                gain_control_of(the_chosen_objects()).until_end_of_turn(),
                untap(the_chosen_objects()),
                gains_keyword(the_chosen_objects(), KeywordFlag::Haste).until_end_of_turn(),
                destroy(each(creature().other_than(one_of_the_chosen()))),
            ]),
        ))
        .build()
}
