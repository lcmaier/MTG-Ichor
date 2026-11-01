//! Kaldheim (`khm`), released 2021-02-05.

use std::sync::Arc;

use crate::cards::authoring::{
    activated_ability, another, create_tokens, each, gets, static_replacement, triggered_ability, with_subtype, you, you_control,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{AffectedSet, ObjectFilter, ObjectSet, PlayerRef, PlayerSet, TokenDef};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite, Rounding};
use crate::types::triggers::{Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit, TriggerSubject};

fn elves_you_control() -> ObjectFilter {
    with_subtype(Subtype::Creature(CreatureType::Elf)).and(you_control())
}

/// Elvish Warmaster — {1}{G}
/// Creature — Elf Warrior 2/2
///
/// > Whenever one or more other Elves you control enter, create a 1/1 green
/// > Elf Warrior creature token. This ability triggers only once each turn.
/// > {5}{G}{G}: Elves you control get +2/+2 and gain deathtouch until end of turn.
///
/// The earliest printing of "this ability triggers only once each turn", so
/// its ruling is the phrase's definition (§3.5): the gate is the trigger's,
/// written as the dispatcher queues it.
///
/// It exercises "one or more" with "triggers only once each turn"; a pump over
/// a filter, fixed as it resolves.
///
/// # The rulings, and where each is tested
///
/// - *"It doesn't matter how many Elves enter the battlefield under your
///   control. The ability creates only one Elf Warrior creature token."*
///   → `phase_tr2a_integration_test::warmaster_makes_one_token_however_many_elves_enter`.
/// - *"Once the triggered ability has triggered once during a turn, it can't
///   trigger again, even if the triggered ability is still on the stack, has
///   been countered, or has otherwise left the stack."*
///   → `phase_tr2a_integration_test::warmaster_triggers_once_a_turn_even_while_its_first_trigger_waits`.
/// - *"The activated ability affects only Elves you control as the ability
///   resolves..."* → `phase_tr2a_integration_test::warmasters_pump_is_fixed_as_it_resolves`.
pub fn elvish_warmaster() -> Arc<CardData> {
    let other_elves_you_control = TriggerEvent::EntersBattlefield {
        subject: TriggerSubject::Filter(another(with_subtype(Subtype::Creature(CreatureType::Elf)))),
        controller: Some(PlayerRef::You),
        from: None,
        was_cast: None,
        multiplicity: Multiplicity::OncePerEvent,
    };
    let pump = activated_ability(
        "{5}{G}{G}: Elves you control get +2/+2 and gain deathtouch until end of turn.",
        vec![Cost::Mana(ManaCost::build(&[ManaType::Green, ManaType::Green], 5))],
        gets(each(elves_you_control()), 2, 2).and_gains_keyword(KeywordFlag::Deathtouch).until_end_of_turn(),
    );
    CardDataBuilder::new("Elvish Warmaster")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elf))
        .subtype(Subtype::Creature(CreatureType::Warrior))
        .power_toughness(2, 2)
        .rules_text(
            "Whenever one or more other Elves you control enter, create a 1/1 green Elf Warrior creature token. \
             This ability triggers only once each turn.\n\
             {5}{G}{G}: Elves you control get +2/+2 and gain deathtouch until end of turn.",
        )
        .ability(triggered_ability("Whenever one or more other Elves you control enter, create a 1/1 green Elf Warrior creature token. This ability triggers only once each turn.", TriggerDef {
            condition: TriggerCondition::Event(other_elves_you_control),
            intervening_if: None,
            limit: Some(TriggerLimit::TriggersOnlyOnceEachTurn),
            effect: create_tokens(you(), 1, TokenDef::creature(1, 1, &[Color::Green], &[CreatureType::Elf, CreatureType::Warrior])),
        }))
        .ability(pump)
        .build()
}

/// Vorinclex, Monstrous Raider — {4}{G}{G}
/// Legendary Creature — Phyrexian Praetor 6/6
///
/// > Trample, haste
/// > If you would put one or more counters on a permanent or player, put
/// > twice that many of each of those kinds of counters on that permanent or
/// > player instead.
/// > If an opponent would put one or more counters on a permanent or player,
/// > they put half that many of each of those kinds of counters on that
/// > permanent or player instead, rounded down.
///
/// **The reader of `AddCounters::by`, and the first `Halve` over counters.**
/// Both halves are `Filter { All }` plus `Everyone` — "a permanent or player"
/// is genuinely both questions, Furnace of Rath's shape — and differ only in
/// `by`: `PlayerSet::You` against `PlayerSet::Opponents`, resolved against
/// this card's controller. "Each of those kinds" is the entry door's own
/// arithmetic, applied per kind in the mods.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"Unlike many similar effects, Vorinclex cares deeply about who is
///   putting the counters on the permanent or player to determine which of
///   its two last abilities applies."* → an opponent's Battlegrowth on your
///   creature puts half of one, rounded down — none — and yours on theirs
///   puts two; `vorinclex_reads_who_is_putting_the_counters_on`.
/// - *"If a permanent enters the battlefield with counters on it, the effect
///   causing the permanent to be given counters may specify which player
///   puts those counters on it. If the effect doesn't specify a player, the
///   object's controller puts those counters on it."* → CR 122.6a's default
///   at the entry door: your Loyalty Probe enters with six, an opponent's
///   with one; `vorinclex_reads_the_entering_controller_as_the_putter`.
/// - *"If two or more effects attempt to modify how many counters would be
///   put onto a permanent you control, you choose the order to apply those
///   effects, no matter who controls the sources of those effects."* →
///   beside your Hardened Scales, both orders;
///   `you_order_vorinclex_beside_hardened_scales`.
///
/// The "or player" half is built, not recorded: your Live Fast under your
/// Vorinclex gets four energy, and an opponent's gets one;
/// `vorinclex_doubles_and_halves_the_counters_a_player_gets`.
///
/// **Registered and not pooled**: six mana, legendary, and a body whose
/// engine path Hardened Scales opens at one.
pub fn vorinclex_monstrous_raider() -> Arc<CardData> {
    CardDataBuilder::new("Vorinclex, Monstrous Raider")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 4))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Phyrexian))
        .subtype(Subtype::Creature(CreatureType::Praetor))
        .power_toughness(6, 6)
        .keyword_flag(KeywordFlag::Trample)
        .keyword_flag(KeywordFlag::Haste)
        .rules_text(
            "Trample, haste\nIf you would put one or more counters on a permanent or player, \
             put twice that many of each of those kinds of counters on that permanent or \
             player instead.\nIf an opponent would put one or more counters on a permanent \
             or player, they put half that many of each of those kinds of counters on that \
             permanent or player instead, rounded down.",
        )
        .ability(static_replacement("If you would put one or more counters on a permanent or player, put twice that many of each of those kinds of counters on that permanent or player instead.",
            ReplacementDef::new(
                EventPattern::AddCounters { counter: None, by: Some(PlayerSet::You) },
                AffectedSet { objects: ObjectSet::battlefield_filter(ObjectFilter::All), players: PlayerSet::Everyone },
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            ),
        ))
        .ability(static_replacement("If an opponent would put one or more counters on a permanent or player, they put half that many of each of those kinds of counters on that permanent or player instead, rounded down.",
            ReplacementDef::new(
                EventPattern::AddCounters { counter: None, by: Some(PlayerSet::Opponents) },
                AffectedSet { objects: ObjectSet::battlefield_filter(ObjectFilter::All), players: PlayerSet::Everyone },
                Rewrite::Amount(AmountRewrite::Halve(Rounding::Down)),
            ),
        ))
        .build()
}
