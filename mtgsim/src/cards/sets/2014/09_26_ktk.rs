//! Khans of Tarkir (`ktk`), released 2014-09-26.

use std::sync::Arc;

use crate::cards::authoring::{create_tokens, creature, spell_ability, static_replacement, you, you_control};
use crate::cards::tokens::goblin_token;
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{CounterType, ObjectSet};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite};

/// Hardened Scales — {G}
/// Enchantment
///
/// > If one or more +1/+1 counters would be put on a creature you control,
/// > that many plus one +1/+1 counters are put on it instead.
///
/// **`AmountRewrite::Plus`'s second kind** — RD-3 landed the arm for Torbran's
/// damage — and the first additive counter replacement. It does not commute
/// with a multiplier, which is why beside Doubling Season the affected
/// permanent's controller is asked, and why beside a second Scales the
/// prompt is asked too though its outcome is one (`backlog.md` §2.29).
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"If a creature you control would enter the battlefield with a number of
///   +1/+1 counters on it, it enters with that many plus one instead."* → the
///   entry door on Master Biomancer's grant;
///   `hardened_scales_adds_one_to_the_counters_a_creature_enters_with`.
/// - *"If two or more effects attempt to modify how many counters would be
///   put on a creature you control, you choose the order to apply those
///   effects, no matter who controls the sources of those effects."* → an
///   opponent's Primal Vigor beside your Scales, and the creature's
///   controller is asked — 1 → 2 → 3 or 1 → 2 → 4;
///   `the_creatures_controller_orders_scales_beside_an_opponents_vigor`.
/// - *"Each additional Hardened Scales you control will increase the number
///   of +1/+1 counters placed on a creature you control by one."* → two
///   rows, each applied once; `two_hardened_scales_add_two`.
///
/// **Pooled**, `PERFORMANCE_POOL` +1 — its comment there says why.
pub fn hardened_scales() -> Arc<CardData> {
    CardDataBuilder::new("Hardened Scales")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If one or more +1/+1 counters would be put on a creature you control, that many \
             plus one +1/+1 counters are put on it instead.",
        )
        .ability(static_replacement("If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on it instead.", ReplacementDef::new(
            EventPattern::AddCounters { counter: Some(CounterType::PlusOnePlusOne), by: None },
            ObjectSet::battlefield_filter(creature().and(you_control())),
            Rewrite::Amount(AmountRewrite::Plus(1)),
        )))
        .build()
}

/// Hordeling Outburst — {1}{R}{R}
/// Sorcery
///
/// > Create three 1/1 red Goblin creature tokens.
///
/// Raise the Alarm's shape one wider, and a sorcery: the second plural
/// creation, so that "the batch is the creation's order" (CR 613.7m's
/// decision point, not asked — `replacement-architecture.md` §9, RE
/// decision 3) is asserted on more than a pair.
///
/// Scryfall lists no rulings (2026-09-13). **Registered and not pooled** —
/// the same engine path as the Alarm at `{1}{R}{R}`.
pub fn hordeling_outburst() -> Arc<CardData> {
    CardDataBuilder::new("Hordeling Outburst")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text("Create three 1/1 red Goblin creature tokens.")
        .ability(spell_ability("Create three 1/1 red Goblin creature tokens.",
            create_tokens(you(), 3, goblin_token()),
        ))
        .build()
}
