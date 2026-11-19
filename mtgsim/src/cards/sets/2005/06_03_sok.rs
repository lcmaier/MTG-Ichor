//! Saviors of Kamigawa (`sok`), released 2005-06-03.

use std::sync::Arc;

use crate::cards::authoring::{
    all_their, choose_number_at_least, choose_objects, lose_life, one_of_the_chosen, sacrifice, spell_ability, target_opponent, that_target,
    the_chosen_number, they_control,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, SpellType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{Condition, CostAnswer, Effect, ObjectFilter, PlayerRef, TargetCount};
use crate::types::mana::{ManaCost, ManaType};

/// Choice of Damnations — {5}{B}
/// Sorcery — Arcane
///
/// > Target opponent chooses a number. You may have that player lose that
/// > much life. If you don't, that player sacrifices all but that many
/// > permanents of their choice.
///
/// (Oracle text verified on Scryfall, 2026-11-18.)
///
/// "Chooses a number" with no most printed: any number from zero up (CR
/// 107.1b). The number is the opponent's, the "may" is yours, and the
/// permanents kept are the opponent's choice again: that many of their own,
/// and every other permanent they control sacrificed by them.
///
/// # The rulings, and where each is tested
///
/// - #1, the number first and then its controller's choice between the life
///   and the sacrifice →
///   `phase_several_choosers_at_one_step_integration_test::choice_of_damnations_has_the_opponent_lose_the_number_they_chose`.
/// - #2, "that opponent chooses that many permanents and then sacrifices
///   the rest", and a number above what they control sacrifices nothing →
///   `phase_several_choosers_at_one_step_integration_test::choice_of_damnations_has_the_opponent_keep_that_many_and_sacrifice_the_rest`.
pub fn choice_of_damnations() -> Arc<CardData> {
    let text = "Target opponent chooses a number. You may have that player lose that much life. \
                If you don't, that player sacrifices all but that many permanents of their choice.";
    let that_player = || that_target(0);
    let keeps_that_many = Effect::Sequence(vec![
        choose_objects(that_player(), ObjectFilter::All.and(they_control()), TargetCount::ExactlyAmount(the_chosen_number())),
        sacrifice(all_their(that_player(), ObjectFilter::All.other_than(one_of_the_chosen()))),
    ]);
    CardDataBuilder::new("Choice of Damnations")
        .mana_cost(ManaCost::build(&[ManaType::Black], 5))
        .color(Color::Black)
        .card_type(CardType::Sorcery)
        .subtype(Subtype::Spell(SpellType::Arcane))
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                choose_number_at_least(target_opponent(), 0),
                Effect::Optional { chooser: PlayerRef::You, effect: Box::new(lose_life(that_player(), the_chosen_number())) },
                Effect::Conditional(Condition::CostAnswer(CostAnswer::Doesnt), Box::new(keeps_that_many)),
            ]),
        ))
        .build()
}
