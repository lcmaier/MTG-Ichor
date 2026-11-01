//! Ravnica Allegiance (`rna`), released 2019-01-25.

use std::sync::Arc;

use crate::cards::authoring::{
    activated_ability, at_beginning_of, counters_put_on, creature, put_counters, target_permanent, this_object, triggered_ability, whenever,
    win_the_game, you, you_control, Whose,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{AmountExpr, Antecedent, Condition, CounterType};
use crate::types::history::CountIs;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{TriggerCondition, TriggerDef};

/// Simic Ascendancy — {G}{U}
/// Enchantment
///
/// > {1}{G}{U}: Put a +1/+1 counter on target creature you control.
/// > Whenever one or more +1/+1 counters are put on a creature you control,
/// > put that many growth counters on this enchantment.
/// > At the beginning of your upkeep, if this enchantment has twenty or more
/// > growth counters on it, you win the game.
///
/// The growth counters are its own: the put is read once for each creature
/// (`once_per_object`), and "that many" is that creature's. The win is CR
/// 603.4's intervening "if" over the counters on it, asked as the upkeep
/// begins and again as the ability resolves.
///
/// It exercises a put read once per creature, that many counters of a kind no
/// rule reads, and a win behind an intervening "if" over them.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr5b_integration_test`.
/// - *"An ability that triggers when counters are put on a permanent will
///   trigger if that permanent somehow enters the battlefield with those
///   counters."* (#1) →
///   `simic_ascendancy_counts_the_counters_a_creature_enters_with`.
/// - *"If Simic Ascendancy doesn't have twenty or more growth counters on it
///   as your upkeep begins, its last ability won't trigger. You can't take
///   any actions during your turn before your upkeep begins."* (#2) →
///   `simic_ascendancy_asks_for_twenty_as_the_upkeep_begins`.
/// - *"If the last ability does trigger, but counters are removed from Simic
///   Ascendancy so it has fewer than twenty remaining on it, you won't win
///   the game."* (#3) →
///   `simic_ascendancy_does_not_win_once_its_counters_fall_below_twenty`.
/// - *"If the last ability does trigger, but Simic Ascendancy leaves the
///   battlefield, use the number of counters it had on it immediately before
///   it left the battlefield to determine whether you win the game."* (#4) →
///   `simic_ascendancy_wins_off_the_count_it_had_as_it_left`.
pub fn simic_ascendancy() -> Arc<CardData> {
    let activated_text = "{1}{G}{U}: Put a +1/+1 counter on target creature you control.";
    let growth_text = "Whenever one or more +1/+1 counters are put on a creature you control, put that many growth counters on this enchantment.";
    let win_text =
        "At the beginning of your upkeep, if this enchantment has twenty or more growth counters on it, you win the game.";
    let growth = put_counters(this_object(), CounterType::Growth, AmountExpr::TriggeringAmount);
    let win = TriggerDef {
        condition: TriggerCondition::Event(at_beginning_of(StepType::Upkeep, Whose::Yours)),
        intervening_if: Some(Condition::CounterCount {
            object: Antecedent::ThisObject,
            kind: Some(CounterType::Growth),
            count: CountIs::AtLeast(20),
        }),
        limit: None,
        effect: win_the_game(you()),
    };
    CardDataBuilder::new("Simic Ascendancy")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Blue], 0))
        .color(Color::Green)
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text(&format!("{activated_text}\n{growth_text}\n{win_text}"))
        .ability(activated_ability(
            activated_text,
            vec![Cost::Mana(ManaCost::build(&[ManaType::Green, ManaType::Blue], 1))],
            put_counters(target_permanent(creature().and(you_control())), CounterType::PlusOnePlusOne, 1),
        ))
        .ability(triggered_ability(
            growth_text,
            whenever(counters_put_on(creature().and(you_control()), CounterType::PlusOnePlusOne).once_per_object(), growth),
        ))
        .ability(triggered_ability(win_text, win))
        .build()
}
