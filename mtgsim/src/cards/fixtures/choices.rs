//! Fixtures for the choices a card asks for (`choices-architecture.md`). A
//! division is announced on a cast, an activation and a trigger alike (CR
//! 601.2d, 602.2b, 603.3d), and divides damage, counters and shields.

use std::sync::Arc;

use crate::cards::authoring::{
    activated_ability, an_opponent_controls, creature, deal_damage_divided, distribute_counters, enters, planeswalker,
    prevent_the_next_damage_divided, spell_ability, tap, targets, triggered_ability, whenever,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{AmountExpr, CounterType, Effect, ObjectFilter, SelectionFilter, TargetCount};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::types::triggers::TriggerSubject;

/// Mogg Mob Fixture — {2}{R}
/// Creature — Goblin, 3/3.
///
/// A division on an activation (CR 602.2b), three among up to three, so two
/// targets ask. Sacrificing it is the cost, so the amount and the shares are
/// fixed before its source is gone, and the source deals the damage as it last
/// existed (CR 113.7a).
pub fn mogg_mob_fixture() -> Arc<CardData> {
    let text = "Sacrifice this creature: It deals 3 damage divided as you choose among one, two, or three targets.";
    CardDataBuilder::new("Mogg Mob Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Goblin))
        .power_toughness(3, 3)
        .rules_text(text)
        .ability(activated_ability(text, vec![Cost::SacrificeSelf], deal_damage_divided(SelectionFilter::Any, TargetCount::Between(1, 3), 3)))
        .build()
}

/// Dragonlord Atarka Fixture — {5}{R}{G}
/// Creature — Dragon, 8/8, unlegendary.
///
/// A division on a trigger (CR 603.3d), among "any number" of a filtered
/// target, the shape `tdc` prints.
pub fn dragonlord_atarka_fixture() -> Arc<CardData> {
    let text = "When this creature enters, it deals 5 damage divided as you choose among any number of target creatures and/or planeswalkers your opponents control.";
    let theirs = SelectionFilter::Permanent(creature().or(planeswalker()).and(an_opponent_controls()));
    CardDataBuilder::new("Dragonlord Atarka Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Green], 5))
        .color(Color::Red)
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Dragon))
        .power_toughness(8, 8)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(enters(TriggerSubject::ThisObject), deal_damage_divided(theirs, TargetCount::AnyNumber, 5)),
        ))
        .build()
}

/// Magma Opus Fixture — {4}{R}, a sorcery: its first two sentences.
///
/// Two instances of "target", one divided: CR 601.2d comes after 601.2c, so
/// the second instance is named before the first is divided.
pub fn magma_opus_fixture() -> Arc<CardData> {
    let text = "Magma Opus Fixture deals 4 damage divided as you choose among any number of targets. Tap two target permanents.";
    CardDataBuilder::new("Magma Opus Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Red], 4))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                deal_damage_divided(SelectionFilter::Any, TargetCount::AnyNumber, 4),
                tap(targets(2, SelectionFilter::Permanent(ObjectFilter::All))),
            ]),
        ))
        .build()
}

/// Elven Rite Fixture — {1}{G}, a sorcery.
///
/// Counters divided, the other half of CR 601.2d's "damage or counters".
pub fn elven_rite_fixture() -> Arc<CardData> {
    let text = "Distribute two +1/+1 counters among one or two target creatures.";
    CardDataBuilder::new("Elven Rite Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            distribute_counters(SelectionFilter::Creature, TargetCount::Between(1, 2), CounterType::PlusOnePlusOne, 2),
        ))
        .build()
}

/// Remedy Fixture — {1}{W}, an instant.
///
/// A prevention divided: each target a shield of its own share (CR 615.7).
pub fn remedy_fixture() -> Arc<CardData> {
    let text = "Prevent the next 5 damage that would be dealt this turn to any number of targets, divided as you choose.";
    CardDataBuilder::new("Remedy Fixture")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, prevent_the_next_damage_divided(SelectionFilter::Any, TargetCount::AnyNumber, 5)))
        .build()
}

/// Rolling Thunder Fixture — {X}{R}, a sorcery.
///
/// A total the X announced before the targets sets (CR 601.2b, 601.2d).
pub fn rolling_thunder_fixture() -> Arc<CardData> {
    let text = "Rolling Thunder Fixture deals X damage divided as you choose among any number of targets.";
    CardDataBuilder::new("Rolling Thunder Fixture")
        .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Red)]))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(text, deal_damage_divided(SelectionFilter::Any, TargetCount::AnyNumber, AmountExpr::X)))
        .build()
}

/// Meteor Swarm Fixture — {X}{R}, a sorcery.
///
/// A count the X sets and a fixed total: past 8 targets, no X can be divided
/// at least one each, and the announcement rewinds.
pub fn meteor_swarm_fixture() -> Arc<CardData> {
    let text = "Meteor Swarm Fixture deals 8 damage divided as you choose among X target creatures.";
    CardDataBuilder::new("Meteor Swarm Fixture")
        .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Red)]))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            deal_damage_divided(SelectionFilter::Creature, TargetCount::ExactlyAmount(AmountExpr::X), 8),
        ))
        .build()
}
