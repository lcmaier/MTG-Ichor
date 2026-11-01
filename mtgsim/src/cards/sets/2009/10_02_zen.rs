//! Zendikar (`zen`), released 2009-10-02.

use std::sync::Arc;

use crate::cards::authoring::{
    at_beginning_of, put_counters, shuffles_their_library, this_object, triggered_ability, whenever, win_the_game, you, Whose,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Condition, CounterType, Effect, PlayerFact, PlayerRef, PlayerSet};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{TriggerCondition, TriggerDef};

/// Cosi's Trickster — {U}
/// Creature — Merfolk Wizard 1/1
///
/// > Whenever an opponent shuffles their library, you may put a +1/+1 counter
/// > on this creature.
///
/// It exercises "whenever an opponent shuffles their library"; a "may" on this
/// creature.
///
/// # The rulings, and where each is tested
///
/// - *"...triggers when an opponent shuffles their library because that player
///   was instructed to do so by a spell or ability that specifically contains
///   the word 'shuffle'..."* (#1) and *"If an opponent's library is empty or
///   has just a single card in it ..., Cosi's Trickster's ability will still
///   trigger."* (#3)
///   → `phase_tr2b_integration_test::cosis_trickster_sees_an_opponents_shuffle_even_of_an_empty_library`.
/// - *"The cascade ability doesn't cause a player to shuffle their library..."*
///   (#2) → a `no-registered-card` disposition in the ledger: nothing
///   registered cascades, and no primitive puts cards on the bottom in a
///   random order.
pub fn cosis_trickster() -> Arc<CardData> {
    CardDataBuilder::new("Cosi's Trickster")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Merfolk))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(1, 1)
        .rules_text("Whenever an opponent shuffles their library, you may put a +1/+1 counter on this creature.")
        .ability(triggered_ability("Whenever an opponent shuffles their library, you may put a +1/+1 counter on this creature.", whenever(
            shuffles_their_library(Whose::AnOpponents),
            Effect::Optional {
                chooser: PlayerRef::You,
                effect: Box::new(put_counters(this_object(), CounterType::PlusOnePlusOne, AmountExpr::Fixed(1))),
            },
        )))
        .build()
}

/// Felidar Sovereign — {4}{W}{W}
/// Creature — Cat Beast 4/6
///
/// > Vigilance (Attacking doesn't cause this creature to tap.)
/// > Lifelink (Damage dealt by this creature also causes you to gain that much life.)
/// > At the beginning of your upkeep, if you have 40 or more life, you win the game.
///
/// CR 603.4's intervening "if": checked as the upkeep begins, and again as
/// the ability resolves (608.2a). "Your upkeep" is `Whose::Yours`.
///
/// It exercises CR 603.4's intervening "if" at both instants, `WinGame`.
///
/// # The rulings, and where each is tested
///
/// - *"Felidar Sovereign's triggered ability checks to see if you have 40 or
///   more life as your upkeep begins. If you don't, the ability won't trigger
///   at all. If you do, the ability will check again as it tries to resolve.
///   If you don't have 40 or more life at that time, the ability won't do
///   anything."* → the three tests
///   `phase_tr1_integration_test::felidar_sovereign_wins_when_the_condition_holds_at_both_instants`,
///   `felidar_sovereign_does_not_trigger_below_forty_life` and
///   `felidar_sovereign_does_nothing_when_life_drops_before_it_resolves`.
/// - The Two-Headed Giant ruling is a format-variant disposition in the
///   ledger: CR 810 is not built.
pub fn felidar_sovereign() -> Arc<CardData> {
    CardDataBuilder::new("Felidar Sovereign")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 4))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Cat))
        .subtype(Subtype::Creature(CreatureType::Beast))
        .power_toughness(4, 6)
        .keyword_flag(KeywordFlag::Vigilance)
        .keyword_flag(KeywordFlag::Lifelink)
        .rules_text("Vigilance\nLifelink\nAt the beginning of your upkeep, if you have 40 or more life, you win the game.")
        .ability(triggered_ability("At the beginning of your upkeep, if you have 40 or more life, you win the game.", TriggerDef {
            condition: TriggerCondition::Event(at_beginning_of(StepType::Upkeep, Whose::Yours)),
            intervening_if: Some(Condition::Player {
                players: PlayerSet::You,
                fact: PlayerFact::LifeAtLeast(AmountExpr::Fixed(40)),
            }),
            limit: None,
            effect: win_the_game(you()),
        }))
        .build()
}

/// Vampire Nighthawk — {1}{B}{B}
/// Creature — Vampire Shaman
/// 2/3 Flying, Lifelink, Deathtouch
pub fn vampire_nighthawk() -> Arc<CardData> {
    CardDataBuilder::new("Vampire Nighthawk")
        .card_type(CardType::Creature)
        .color(Color::Black)
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 1))
        .power_toughness(2, 3)
        .keyword_flag(KeywordFlag::Flying)
        .keyword_flag(KeywordFlag::Lifelink)
        .keyword_flag(KeywordFlag::Deathtouch)
        .rules_text("Flying\nDeathtouch\nLifelink")
        .build()
}
