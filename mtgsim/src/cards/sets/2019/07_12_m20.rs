//! Core Set 2020 (`m20`), released 2019-07-12.

use std::sync::Arc;

use crate::cards::authoring::{put_counters, this_object, triggered_ability};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{CounterType, PlayerRef};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit};

/// Vengeful Warchief — {4}{B}
/// Creature — Orc Warrior 4/4
///
/// > Whenever you lose life for the first time each turn, put a +1/+1 counter
/// > on this creature. (Damage causes loss of life.)
///
/// "The first time each turn" is the loss record's place among this turn's
/// losses (§3.5), so two losses in one batch are the first and the second.
///
/// It exercises "for the first time each turn": a record's place in its turn.
///
/// # The rulings, and where each is tested
///
/// - *"A player loses life if they pay life."*
///   → `phase_tr2a_integration_test::warchief_counts_paid_life_as_lost_life`.
/// - *"You put only one +1/+1 counter on Vengeful Warchief, no matter how much
///   life you lost."* → the same test.
/// - *"If you pay life to cast a spell or activate an ability, you don't put a
///   +1/+1 counter on Vengeful Warchief until after you've finished casting
///   that spell or activating that ability. You put the counter on Vengeful
///   Warchief before that spell or ability resolves."*
///   → `phase_tr2a_integration_test::warchiefs_counter_goes_on_after_the_activation_and_before_it_resolves`.
/// - *"If Vengeful Warchief comes under your control after you've already lost
///   life in a turn, its ability can't trigger during that turn."*
///   → `phase_tr2a_integration_test::a_warchief_that_arrives_after_the_first_loss_waits_for_next_turn`.
pub fn vengeful_warchief() -> Arc<CardData> {
    CardDataBuilder::new("Vengeful Warchief")
        .mana_cost(ManaCost::build(&[ManaType::Black], 4))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Orc))
        .subtype(Subtype::Creature(CreatureType::Warrior))
        .power_toughness(4, 4)
        .rules_text("Whenever you lose life for the first time each turn, put a +1/+1 counter on this creature.")
        .ability(triggered_ability("Whenever you lose life for the first time each turn, put a +1/+1 counter on this creature.", TriggerDef {
            condition: TriggerCondition::Event(TriggerEvent::LosesLife {
                player: Some(PlayerRef::You),
                multiplicity: Multiplicity::PerOccurrence,
            }),
            intervening_if: None,
            limit: Some(TriggerLimit::FirstTimeEachTurn),
            effect: put_counters(this_object(), CounterType::PlusOnePlusOne, 1),
        }))
        .build()
}
