//! March of the Machine: The Aftermath (`mat`), released 2023-05-12.

use std::sync::Arc;

use crate::cards::authoring::{
    at_beginning_of, choose_objects, creatures_you_control, divided, gain_life, put_counters, the_chosen_objects, triggered_ability, you,
    Whose,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Condition, CounterType, Effect, PlayerSet, TargetCount};
use crate::types::history::{CountIs, HistoryCount, HistorySpan, TurnFact};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{TriggerCondition, TriggerDef};

/// Feast of the Victorious Dead — {W}{B}
/// Enchantment
///
/// > At the beginning of your end step, if one or more creatures died this
/// > turn, you gain that much life and distribute that many +1/+1 counters
/// > among any number of creatures you control.
///
/// (Oracle text verified on Scryfall, 2026-11-14.)
///
/// A division among objects not targeted (CR 608.2d): the trigger announces
/// nothing as it is put on the stack, and its controller chooses the
/// creatures and divides the counters as it resolves, each chosen creature
/// receiving at least one. Two steps: the choice, which keeps each
/// creature with its share, and the counters put on the chosen by those
/// shares; the amount the second prints is the total the first divided.
/// "That much" and "that many" are the deaths counted then, every player's
/// (morbid's sum), so a creature that dies in response counts, though the
/// intervening "if" needed one before.
pub fn feast_of_the_victorious_dead() -> Arc<CardData> {
    let text = "At the beginning of your end step, if one or more creatures died this turn, you gain that much life and distribute that many +1/+1 counters among any number of creatures you control.";
    let died = HistoryCount { players: PlayerSet::Everyone, fact: TurnFact::ControlledCreaturesDied, of: None };
    let that_many = AmountExpr::History(HistorySpan::ThisTurn, Box::new(died.clone()));
    CardDataBuilder::new("Feast of the Victorious Dead")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::Black], 0))
        .color(Color::White)
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(text, TriggerDef {
            condition: TriggerCondition::Event(at_beginning_of(StepType::End, Whose::Yours)),
            intervening_if: Some(Condition::ThisTurn(died, CountIs::AtLeast(1))),
            limit: None,
            effect: Effect::Sequence(vec![
                gain_life(you(), that_many.clone()),
                choose_objects(you(), creatures_you_control(), divided(TargetCount::AnyNumber, that_many.clone())),
                put_counters(the_chosen_objects(), CounterType::PlusOnePlusOne, that_many),
            ]),
        }))
        .build()
}
