//! Zendikar (`zen`), released 2009-10-02.

use std::sync::Arc;

use crate::cards::authoring::{
    at_beginning_of, card_type, choose_card_type, chosen_by, each_player, put_counters, sacrifice, shuffles_their_library, the_chosen_card_type,
    this_object, triggered_ability, whenever, win_the_game, you, you_have, Whose,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Choice, ChoiceScope, ChoiceSide, CounterType, Effect, Pick, PlayerFact, PlayerRef};
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
            intervening_if: Some(you_have(PlayerFact::LifeAtLeast(AmountExpr::Fixed(40)))),
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
        .subtype(Subtype::Creature(CreatureType::Vampire))
        .subtype(Subtype::Creature(CreatureType::Shaman))
        .color(Color::Black)
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 1))
        .power_toughness(2, 3)
        .keyword_flag(KeywordFlag::Flying)
        .keyword_flag(KeywordFlag::Lifelink)
        .keyword_flag(KeywordFlag::Deathtouch)
        .rules_text("Flying\nDeathtouch\nLifelink")
        .build()
}

/// World Queller — {3}{W}{W}
/// Creature — Avatar 4/4
///
/// > At the beginning of your upkeep, you may choose a card type. If you do,
/// > each player sacrifices a permanent of their choice of that type.
///
/// (Oracle text verified on Scryfall, 2026-11-15.)
///
/// A card type chosen as the triggered ability resolves (CR 205.2a,
/// 608.2d), and read by each player's edict: the candidates are their
/// permanents of that type. "You may choose" is the whole of it, since
/// choosing is what "if you do" asks about.
///
/// # The rulings, and where each is tested
///
/// - *"You may choose any card type. Choosing anything other than artifact,
///   battle, creature, enchantment, kindred, land, or planeswalker will
///   result in no permanents being sacrificed, though, ..."* (#1) →
///   `phase_value_chosen_as_an_effect_resolves_integration_test::world_queller_may_choose_a_type_no_permanent_has`.
/// - *"You don't choose a card type until the ability resolves. Once you
///   choose a type, it's too late for players to respond."* (#2) →
///   `phase_value_chosen_as_an_effect_resolves_integration_test::world_queller_keeps_a_card_type_for_each_players_edict`.
/// - *"First you choose which permanent you'll sacrifice ..., then each other
///   player in turn order does the same, then all chosen permanents are
///   sacrificed at the same time."* (#3) →
///   `phase_value_chosen_as_an_effect_resolves_integration_test::world_queller_asks_each_player_in_turn_order`.
pub fn world_queller() -> Arc<CardData> {
    let text = "At the beginning of your upkeep, you may choose a card type. If you do, each player sacrifices a permanent of their choice of that type.";
    let each_sacrifices_one_of_the_type = sacrifice(chosen_by(Choice {
        chooser: each_player(),
        among: ChoiceScope::ChoosersPermanents,
        picks: vec![Pick::exactly(1, card_type(the_chosen_card_type()))],
        acts_on: ChoiceSide::Chosen,
    }));
    CardDataBuilder::new("World Queller")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Avatar))
        .power_toughness(4, 4)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(
                at_beginning_of(StepType::Upkeep, Whose::Yours),
                Effect::Optional {
                    chooser: PlayerRef::You,
                    effect: Box::new(Effect::Sequence(vec![choose_card_type(you()), each_sacrifices_one_of_the_type])),
                },
            ),
        ))
        .build()
}
