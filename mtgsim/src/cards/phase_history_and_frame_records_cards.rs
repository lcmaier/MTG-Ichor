//! A6e's cleanup, `history-and-frame-records`: the castable fixtures whose
//! builds carry a debug check, which `--pool fixtures` plays in the checked
//! sitting (`engineering-practices.md#cards-and-pools`). Invented, named as
//! fixtures, and never in `default_registry`: each stands for a family of
//! printed cards no registered card covers yet.
//!
//! | Fixture | Stands for | The check it reaches |
//! |---|---|---|
//! | Fixture Curtain | Curtain of Light's "target unblocked attacking creature becomes blocked" | an effect's block has its record |
//! | Fixture Token Tally | "for each token creature that died this turn" | no history read asks the store about a departed object |
//! | Fixture Mourning Ink | "cards in graveyards that died this turn" | each zone change writes its row |
//! | Fixture Mourner's Tithe | "whenever a creature leaves the battlefield, ... its power" | a departure's frame binds its object |
//! | Fixture First Creature's Tithe | "your first creature spell each turn" | the placed facts agree |

use std::sync::Arc;

use crate::cards::authoring::{
    becomes_a_blocked_creature, creature, each_in, gain_life, is_also_color, spell_ability, static_ability, target_creature, token, you,
};
use crate::cards::authoring::triggers::{leaves_the_battlefield, triggered_ability, whenever};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, ObjectFilter, PlayerRef, PlayerSet,
};
use crate::types::history::{HistoryCount, HistorySpan, ObjectFact, TurnFact};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{EventRole, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit, TriggerSubject};
use crate::types::zones::ZoneSet;

/// Every castable fixture, by name, as `CardRegistry::fixture_pool` registers
/// them.
pub const CASTABLE: [(&str, fn() -> Arc<CardData>); 5] = [
    ("Fixture Curtain", fixture_curtain),
    ("Fixture Token Tally", fixture_token_tally),
    ("Fixture Mourning Ink", fixture_mourning_ink),
    ("Fixture Mourner's Tithe", fixture_mourners_tithe),
    ("Fixture First Creature's Tithe", fixture_first_creature_tithe),
];

/// Fixture Curtain — {W}, instant: "Target creature becomes blocked." CR
/// 509.1h's status, which only an unblocked attacking creature takes.
pub fn fixture_curtain() -> Arc<CardData> {
    let text = "Target creature becomes blocked.";
    CardDataBuilder::new("Fixture Curtain")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, becomes_a_blocked_creature(target_creature())))
        .build()
}

/// Fixture Token Tally — {1}, instant: "You gain 1 life for each token
/// creature you controlled that died this turn." The tokens are known only
/// by their departures' rows once CR 704.5d has removed them.
pub fn fixture_token_tally() -> Arc<CardData> {
    let text = "You gain 1 life for each token creature you controlled that died this turn.";
    let died = HistoryCount { players: PlayerSet::You, fact: TurnFact::ControlledCreaturesDied, of: Some(Box::new(token())) };
    CardDataBuilder::new("Fixture Token Tally")
        .mana_cost(ManaCost::build(&[], 1))
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, gain_life(you(), AmountExpr::History(HistorySpan::ThisTurn, Box::new(died)))))
        .build()
}

/// Fixture Mourning Ink — {1}{B}, enchantment: "Cards in graveyards that died
/// this turn are black." Each card its death made (CR 700.4, 400.7).
pub fn fixture_mourning_ink() -> Arc<CardData> {
    let text = "Cards in graveyards that died this turn are black.";
    CardDataBuilder::new("Fixture Mourning Ink")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(static_ability(text, is_also_color(each_in(ObjectFilter::ThisTurn(ObjectFact::Died), ZoneSet::GRAVEYARD), Color::Black)))
        .build()
}

/// Fixture Mourner's Tithe — {2}, enchantment: "Whenever a creature leaves
/// the battlefield, you gain life equal to its power." Its power as it last
/// existed (CR 608.2h), a creature leaving the game with its owner included
/// (CR 603.6c).
pub fn fixture_mourners_tithe() -> Arc<CardData> {
    let text = "Whenever a creature leaves the battlefield, you gain life equal to its power.";
    let creature = TriggerSubject::Filter(creature());
    let gain = gain_life(you(), AmountExpr::TriggeringPower(EventRole::Object));
    CardDataBuilder::new("Fixture Mourner's Tithe")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(text, whenever(leaves_the_battlefield(creature), gain)))
        .build()
}

/// Fixture First Creature's Tithe — {1}{G}, enchantment: "Whenever you cast
/// your first creature spell each turn, you gain 1 life." The first of the
/// kind it names, over the whole turn (Kalamax, the Stormsire's ruling).
pub fn fixture_first_creature_tithe() -> Arc<CardData> {
    let text = "Whenever you cast your first creature spell each turn, you gain 1 life.";
    let first_creature = TriggerDef {
        condition: TriggerCondition::Event(TriggerEvent::CastsSpell {
            caster: Some(PlayerRef::You),
            spell: Some(creature()),
        }),
        intervening_if: None,
        limit: Some(TriggerLimit::FirstTimeEachTurn),
        effect: gain_life(you(), 1),
    };
    CardDataBuilder::new("Fixture First Creature's Tithe")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(text, first_creature))
        .build()
}
