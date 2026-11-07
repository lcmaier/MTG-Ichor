//! Lorwyn (`lrw`), released 2007-10-12.

use std::sync::Arc;

use crate::cards::authoring::{
    an_opponent_controls, choose_two, counter_spell, creature, draw_cards, each, put_counters, return_to_hand, spell_ability, tap,
    target_creature, target_permanent, target_spell, you,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{CounterType, Effect, ObjectFilter};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};

/// Incremental Growth — {3}{G}{G}
/// Sorcery
/// Put a +1/+1 counter on target creature, two +1/+1 counters on another
/// target creature, and three +1/+1 counters on a third target creature.
///
/// **The "another target" shape**, which CR 601.2c reaches through its
/// parenthesis rather than through a rule of its own: one object may be chosen
/// once for each instance "as long as it fits the targeting criteria", and
/// "another" is how a card writes the exclusion *into* those criteria. Each
/// later clause carries `ObjectFilter::OtherThanInstance` for every earlier one.
///
/// Registered and unpooled: three distinct creatures is a board a random game
/// reaches rarely, and the castability rule it presses on is the same one
/// Jagged Lightning presses on more cheaply.
///
/// # The rulings, and where each is tested
///
/// - *"You must choose three different targets in order to cast Incremental
///   Growth."* — **testable now**. Two creatures is not enough, and the spell
///   is not offered at all rather than being offered and rewound:
///   `incremental_growth_needs_a_third_creature_to_be_castable`.
/// - *"If some of the creatures are illegal targets as Incremental Growth tries
///   to resolve, the remaining legal targets still get the appropriate number
///   of +1/+1 counters. If all targets are illegal, Incremental Growth doesn't
///   resolve."* — **testable now**, and it is both halves of CR 608.2b in one
///   sentence: `incremental_growth_still_counters_the_creatures_that_are_left`
///   and `incremental_growth_does_not_resolve_with_every_creature_gone`.
///
/// *"You decide how many +1/+1 counters each creature will get as part of
/// casting the spell"* is not a third ruling here — the counts are printed
/// (one, two, three) and belong to the clauses, not to CR 601.2d's division.
pub fn incremental_growth() -> Arc<CardData> {
    // "another target creature", "a third target creature": a creature, and not
    // one an earlier instance took.
    let other_than = |earlier_targets: &[usize]| {
        let mut filter = creature();
        for &ix in earlier_targets {
            filter = filter.and(ObjectFilter::OtherThanInstance(ix));
        }
        target_permanent(filter)
    };
    let grow = |n: u64, recipient| {
        put_counters(recipient, CounterType::PlusOnePlusOne, n)
    };
    CardDataBuilder::new("Incremental Growth")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Sorcery)
        .rules_text(
            "Put a +1/+1 counter on target creature, two +1/+1 counters on another target \
             creature, and three +1/+1 counters on a third target creature.",
        )
        .ability(spell_ability("Put a +1/+1 counter on target creature, two +1/+1 counters on another target creature, and three +1/+1 counters on a third target creature.", Effect::Sequence(vec![
            grow(1, target_creature()),
            grow(2, other_than(&[0])),
            grow(3, other_than(&[0, 1])),
        ])))
        .build()
}

/// Knight of Meadowgrain — {W}{W}
/// Creature — Kithkin Knight
/// 2/2 First Strike, Lifelink
pub fn knight_of_meadowgrain() -> Arc<CardData> {
    CardDataBuilder::new("Knight of Meadowgrain")
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Kithkin))
        .subtype(Subtype::Creature(CreatureType::Knight))
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 0))
        .power_toughness(2, 2)
        .keyword_flag(KeywordFlag::FirstStrike)
        .keyword_flag(KeywordFlag::Lifelink)
        .rules_text("First strike\nLifelink")
        .build()
}

/// Cryptic Command — {1}{U}{U}{U}
/// Instant
/// Choose two —
/// • Counter target spell.
/// • Return target permanent to its owner's hand.
/// • Tap all creatures your opponents control.
/// • Draw a card.
///
/// (Oracle text verified on Scryfall, 2026-11-07.)
///
/// CR 700.2's "choose two": two different modes, each pair a choice, of four
/// modes two of which target and two of which do not.
///
/// # The rulings, and where each is tested
///
/// - #1, both modes chosen as it is cast, and two different ones:
///   `cryptic_command_offers_each_pair_of_different_modes`.
/// - #2, its targets are its chosen modes', and with every one illegal none
///   of its effects happens: `cryptic_command_whose_one_target_left_draws_nothing`.
pub fn cryptic_command() -> Arc<CardData> {
    let text = "Choose two —\n• Counter target spell.\n• Return target permanent to its owner's hand.\n• Tap all creatures your opponents control.\n• Draw a card.";
    CardDataBuilder::new("Cryptic Command")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue, ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(
            text,
            choose_two([
                counter_spell(target_spell()),
                return_to_hand(target_permanent(ObjectFilter::All)),
                tap(each(creature().and(an_opponent_controls()))),
                draw_cards(you(), 1),
            ]),
        ))
        .build()
}
