//! Khans of Tarkir (`ktk`), released 2014-09-26.

use std::sync::Arc;

use crate::cards::authoring::{
    choose_objects, create_tokens, creature, destroy, each, one_of_the_chosen, spell_ability, static_replacement, you, you_control,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType};
use crate::types::colors::Color;
use crate::types::effects::{CounterType, Effect, ObjectSet, TargetCount, TokenDef};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite};

/// Duneblast — {4}{W}{B}{G}
/// Sorcery
///
/// > Choose up to one creature. Destroy the rest.
///
/// (Oracle text verified on Scryfall, 2026-11-15.)
///
/// Up to one creature chosen as the spell resolves, none of them a target
/// (CR 115.10a, 608.2d), and "the rest" every creature other than it.
///
/// # The rulings, and where each is tested
///
/// - *"You decide which creature to spare as Duneblast resolves. This choice
///   doesn't target the creature. If you don't choose a creature, then all
///   creatures will be destroyed."* (#1) →
///   `phase_value_chosen_as_an_effect_resolves_integration_test::duneblast_spares_the_creature_chosen_and_destroys_the_rest`.
pub fn duneblast() -> Arc<CardData> {
    let text = "Choose up to one creature. Destroy the rest.";
    CardDataBuilder::new("Duneblast")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::Black, ManaType::Green], 4))
        .color(Color::White)
        .color(Color::Black)
        .color(Color::Green)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                choose_objects(you(), creature(), TargetCount::UpTo(1)),
                destroy(each(creature().other_than(one_of_the_chosen()))),
            ]),
        ))
        .build()
}

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
            create_tokens(you(), 3, TokenDef::creature(1, 1, &[Color::Red], &[CreatureType::Goblin])),
        ))
        .build()
}
