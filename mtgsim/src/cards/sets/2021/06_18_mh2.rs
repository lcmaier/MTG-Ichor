//! Modern Horizons 2 (`mh2`), released 2021-06-18.

use std::sync::Arc;

use crate::cards::authoring::{creatures_you_control, each, put_counters, triggered_ability};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, CounterType, Effect, PlayerRef};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit};

/// Nykthos Paragon — {4}{W}{W}
/// Enchantment Creature — Human Soldier 4/6
///
/// > Whenever you gain life, you may put that many +1/+1 counters on each
/// > creature you control. Do this only once each turn.
///
/// "Do this" is the counters, so only a yes closes CR 603.2h's gate (§6.4):
/// the walk answers `Does` for a yes and the resolution writes the gate on
/// that answer alone.
///
/// It exercises "you may" behind CR 603.2h's gate, which only a yes closes;
/// "that many" counters on each creature you control.
///
/// # The rulings, and where each is tested
///
/// All in `phase_tr2b_integration_test`.
/// - *"...As long as you haven't yet chosen to put +1/+1 counters on your
///   creatures with it, all instances of gaining life will cause Nykthos
///   Paragon's ability to trigger."* (#1) and *"Once you have chosen to put
///   +1/+1 counters on your creatures, further instances of gaining life will
///   not cause the ability to trigger."* (#3)
///   → `paragon_triggers_until_you_choose_and_then_does_not`.
/// - *"...if you control multiple Nykthos Paragons, you will be able to do
///   this once for each of them."* (#2) → `two_paragons_each_put_their_counters_once`.
/// - *"If multiple instances of the ability are on the stack, you will be able
///   to put +1/+1 counters for only one of those instances..."* (#4) and the
///   two lifelink creatures of #6
///   → `two_lifelink_creatures_trigger_paragon_twice_and_only_the_first_acts`.
/// - *"If a creature you control is dealt lethal damage at the same time that
///   you gain life, it won't receive +1/+1 counters..."* (#5)
///   → `a_creature_dealt_lethal_damage_as_you_gain_life_gets_no_counters`.
/// - *"...if a single creature you control with lifelink deals combat damage
///   to multiple creatures, players, and/or planeswalkers at the same time
///   ..., the ability will trigger only once."* (#6)
///   → `one_lifelink_creature_dealing_damage_twice_at_once_is_one_gain`.
pub fn nykthos_paragon() -> Arc<CardData> {
    let each_creature_you_control = each(creatures_you_control());
    CardDataBuilder::new("Nykthos Paragon")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 4))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Soldier))
        .power_toughness(4, 6)
        .rules_text(
            "Whenever you gain life, you may put that many +1/+1 counters on each creature you control. \
             Do this only once each turn.",
        )
        .ability(triggered_ability("Whenever you gain life, you may put that many +1/+1 counters on each creature you control. Do this only once each turn.", TriggerDef {
            condition: TriggerCondition::Event(TriggerEvent::GainsLife {
                player: Some(PlayerRef::You),
                multiplicity: Multiplicity::PerOccurrence,
            }),
            intervening_if: None,
            limit: Some(TriggerLimit::DoThisOnlyOnceEachTurn),
            effect: Effect::Optional {
                chooser: PlayerRef::You,
                effect: Box::new(put_counters(each_creature_you_control, CounterType::PlusOnePlusOne, AmountExpr::TriggeringAmount)),
            },
        }))
        .build()
}
