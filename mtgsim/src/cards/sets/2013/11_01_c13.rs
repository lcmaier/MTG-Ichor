//! Commander 2013 (`c13`), released 2013-11-01.

use std::sync::Arc;

use crate::cards::authoring::{creature, static_replacement};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{CounterType, ObjectSet, PlayerSet};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite};

/// Primal Vigor — {4}{G}
/// Enchantment
///
/// > If one or more tokens would be created, twice that many of those tokens
/// > are created instead.
/// > If one or more +1/+1 counters would be put on a creature, twice that
/// > many +1/+1 counters are put on that creature instead.
///
/// **Doubling Season with `Everyone` on the token half and no controller on
/// the counter half** — "it doesn't matter who controls the tokens or the
/// creature", the ruling says, and the N-player table is where that shows.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"Everything that is specified by the effect creating the original
///   token or tokens will also be true about the additional token or
///   tokens"* → RE-4's `the_extra_tokens_are_the_same_tokens`.
/// - *"It doesn't matter who controls the tokens or the creature that the
///   +1/+1 counters are being placed on."* → four players: an opponent's
///   Raise the Alarm makes four, and a third player's Battlegrowth on a
///   fourth's creature puts two;
///   `primal_vigor_does_not_care_who_controls_the_tokens_or_the_creature`.
/// - *"Primal Vigor affects permanents that 'enter the battlefield with' a
///   certain number of counters. For example, if a creature would normally
///   enter the battlefield with three +1/+1 counters on it, it will enter
///   with six."* → Adaptive Shimmerer enters with six;
///   `primal_vigor_doubles_the_counters_a_creature_enters_with`.
/// - *"If there are two Primal Vigors on the battlefield, the number of
///   tokens or +1/+1 counters is four times the original number."* → the
///   Season's acid test on the symmetric card; `two_primal_vigors_quadruple`.
///
/// **Registered and not pooled**, for Doubling Season's reason with
/// `Everyone` on top of it.
pub fn primal_vigor() -> Arc<CardData> {
    CardDataBuilder::new("Primal Vigor")
        .mana_cost(ManaCost::build(&[ManaType::Green], 4))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If one or more tokens would be created, twice that many of those tokens are \
             created instead.\nIf one or more +1/+1 counters would be put on a creature, \
             twice that many +1/+1 counters are put on that creature instead.",
        )
        .ability(static_replacement("If one or more tokens would be created, twice that many of those tokens are created instead.",
            ReplacementDef::new(
                EventPattern::CreateTokens { kind: None },
                PlayerSet::Everyone,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            ),
        ))
        .ability(static_replacement("If one or more +1/+1 counters would be put on a creature, twice that many +1/+1 counters are put on that creature instead.", ReplacementDef::new(
            EventPattern::AddCounters { counter: Some(CounterType::PlusOnePlusOne), by: None },
            ObjectSet::battlefield_filter(creature()),
            Rewrite::Amount(AmountRewrite::Multiplier(2)),
        )))
        .build()
}
