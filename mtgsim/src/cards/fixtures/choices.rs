//! Fixtures for the choices a card asks for (`choices-architecture.md`). A
//! division is announced on a cast, an activation and a trigger alike (CR
//! 601.2d, 602.2b, 603.3d), and divides damage, counters and shields.

use std::sync::Arc;

use crate::cards::authoring::{
    activated_ability, an_opponent_controls, creature, deal_damage_divided, distribute_counters, draw_cards, enters, gain_life,
    gets, planeswalker, prevent_the_next_damage_divided, spell_ability, static_ability, tap, targets, this_object, triggered_ability,
    whenever, you,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::{AdditionalCost, Cost, PaymentAct};
use crate::types::effects::{AmountExpr, Antecedent, CounterType, Effect, ObjectFilter, ObjectValue, SelectionFilter, TargetCount};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::types::triggers::TriggerSubject;

/// Omnath, Locus of Mana Fixture — {2}{G}
/// Creature — Elemental, 1/1.
///
/// Omnath's second line, its power and toughness read off the pool, which is
/// what makes CR 601.2h's order change a game: sacrificed before or after the
/// mana is paid, it last existed as a different creature (`ATOM-601.2h-003`).
/// Its first line, "You don't lose unspent green mana as steps and phases
/// end", is `mana-provenance`'s, so this is not the card. Registered nowhere:
/// the order is a question no registered card's game asks.
pub fn omnath_locus_of_mana_fixture() -> Arc<CardData> {
    let text = "This creature gets +1/+1 for each unspent green mana you have.";
    let per_green = || AmountExpr::UnspentMana(ManaType::Green);
    CardDataBuilder::new("Omnath, Locus of Mana Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Green], 2))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elemental))
        .power_toughness(1, 1)
        .rules_text(text)
        .ability(static_ability(text, gets(this_object(), per_green(), per_green())))
        .build()
}

/// Momentous Fall Fixture — {2}{G}{G}
/// Instant.
///
/// Momentous Fall's words, on its shape: a mandatory sacrifice whose object
/// the resolution reads back as it last existed (the card's 2010-06-15
/// ruling). Beside [`omnath_locus_of_mana_fixture`] it is `ATOM-601.2h-003`'s
/// board.
pub fn momentous_fall_fixture() -> Arc<CardData> {
    let text = "As an additional cost to cast this spell, sacrifice a creature.\nYou draw cards equal to the sacrificed creature's power, then you gain life equal to its toughness.";
    let sacrificed = |value| AmountExpr::ValueOf(Antecedent::PaidWith(PaymentAct::Sacrificed), value);
    CardDataBuilder::new("Momentous Fall Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 2))
        .color(Color::Green)
        .card_type(CardType::Instant)
        .rules_text(text)
        .additional_cost(AdditionalCost::Mandatory(vec![Cost::Sacrifice(creature(), 1)]))
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![draw_cards(you(), sacrificed(ObjectValue::Power)), gain_life(you(), sacrificed(ObjectValue::Toughness))]),
        ))
        .build()
}

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
