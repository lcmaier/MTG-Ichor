//! Cards for Phase RE-9 — mana (`replacement-architecture.md` §9).
//!
//! **The mana production event gets its first watchers.** Until RE-9 the two
//! places that added mana wrote the pool directly and nothing could see
//! them; `GameAction::ProduceMana` is that event, and these are the printed
//! replacement effects CR 106.6a and CR 106.12b describe. Three cards, on
//! two axes:
//!
//! | Card | What it is the first of | CR |
//! |---|---|---|
//! | [`mana_reflection`] | a multiplier over a production | 106.6a |
//! | [`nyxbloom_ancient`] | the second factor the arm has seen | 106.6a |
//! | [`deep_water`] | a production retyped, with a filter on the permanent | 106.12b |
//!
//! Every one of them says "tap … for mana", which is CR 106.12's definition
//! — *"to activate a mana ability of that permanent that includes the {T}
//! symbol in its activation cost"* — and so every one of them is
//! `EventPattern::ProduceMana { tapped_for_mana: Some(true), .. }`. The
//! definition is what makes each card's second ruling true without a line of
//! code: a spell's production and a triggered mana ability's are not tapping
//! a permanent for mana, and neither proposes `true`.
//!
//! # What is not here, and what each waits for
//!
//! Fourteen printed cards replace "tapped for mana" (Scryfall, 2026-09-15).
//! **Virtue of Strength** is the third multiplier ("if you tap a *basic land*
//! for mana … three times") and an Adventure card, which the engine has no
//! second face for. **Contamination** and **Infernal Darkness** are Deep
//! Water's shape as statics, and each carries an upkeep half — a trigger,
//! cumulative upkeep — that is critical-path item 6's; both are fixtures in
//! `tests/phase_re9_integration_test.rs`, where the CR is the customer. **Hall
//! of Gemstone** wants a color chosen at upkeep, **Naked Singularity** a map
//! from basic land type to color, **Harvest Mage** a choice inside the
//! substitution — three facilities, none of them this phase's. **False
//! Dawn** — "spells and abilities you control that would add colored mana
//! instead add that much white mana" — is the one printed watcher that does
//! not say "tapped" and would be the pattern's `None`; its second sentence is
//! a payment rule (spend white as any color) that `ManaPool` does not have.
//! The eight "whenever you tap … for mana, add an additional" cards are CR
//! 605.1b's triggered mana abilities and item 6's.
//!
//! # What a random deck can draw
//!
//! **Mana Reflection is the pooled card**, and it is why this PR exists: a
//! six-drop static on the hottest path in the engine, so that from the turn
//! it resolves every land tap is a gather with a match. Nyxbloom Ancient
//! stays out — the same path at seven mana with a 5/5 trample body that
//! changes combat — and Deep Water stays out as an activated `{U}` the random
//! agent would spend on nothing; its reachability is a `--require` row.

use std::sync::Arc;

use crate::objects::card_data::{
    AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder,
};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PatternFill, PlayerRef,
    PlayerSet, Primitive,
};
use crate::types::ids::new_ability_id;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{
    AmountRewrite, EventPattern, GameActionTemplate, ReplacementDef, Rewrite, TemplateAmount,
};

/// A static ability whose effect is a replacement effect — never a resolution,
/// so it carries no `Duration` and is re-derived off the source's *effective*
/// ability list on every gather.
fn static_replacement(def: ReplacementDef) -> AbilityDef {
    AbilityDef {
        id: new_ability_id(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect: Effect::Replacement(Box::new(def)),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    }
}

/// CR 106.12's "tapped for mana", optionally of a permanent matching `source`.
fn tapped_for_mana(source: Option<ObjectFilter>) -> EventPattern {
    EventPattern::ProduceMana { tapped_for_mana: Some(true), source }
}

/// Mana Reflection — {4}{G}{G}
/// Enchantment
///
/// > If you tap a permanent for mana, it produces twice as much of that mana
/// > instead.
///
/// **The pooled card, and CR 106.6a's first customer.** `Amount(Multiplier(2))`
/// over every unit the production carries — the plain ones scaled, the
/// restricted atoms repeated, which is the rule's "any restrictions … will
/// apply to all mana produced". `PlayerSet::You` is "if *you* tap".
///
/// # The rulings, and where each is tested
///
/// - *"You're 'tapping a permanent for mana' only if you're activating a mana
///   ability of that permanent that includes the {T} symbol in its cost. A
///   mana ability produces mana as part of its effect."* → CR 106.12, read off
///   the activation cost: a Forest is doubled, Dark Ritual is not
///   (`dark_ritual_is_not_tapping_a_permanent_for_mana`), and neither is
///   Krark-Clan Ironworks' sacrifice
///   (`a_mana_ability_without_a_tap_symbol_is_not_doubled`).
/// - *"If an ability triggers 'whenever you tap' something for mana and
///   produces mana, that triggered mana ability won't be affected by Mana
///   Reflection."* → CR 605.1b's triggered mana abilities are critical-path
///   item 6's; the day one exists its own production proposes
///   `tapped_for_mana: false` by the same definition, and this ruling is
///   that PR's test rather than this one's.
/// - *"Mana Reflection doesn't produce any mana itself. Rather, it causes
///   permanents you tap for mana to produce more mana. If the mana ability of
///   that permanent puts any restrictions or riders on the mana it produces,
///   that will apply to all the mana it produces this way."* →
///   `ATOM-106.6a-001`, on a fixture land whose {G} is spendable only on
///   creature spells: two restricted atoms, no free one
///   (`a_restricted_production_doubles_into_two_restricted_units`).
/// - *"The effects of multiple Mana Reflections are cumulative. For example,
///   if you have two Mana Reflections on the battlefield, you'll get four
///   times the original amount and type of mana. If you have three, you'll
///   get eight times the mana, and so on."* → four, and the prompt is the
///   first assertion: two multipliers commute, so CR 616.1 has nothing to ask
///   (`two_mana_reflections_quadruple_with_no_prompt`,
///   `three_mana_reflections_multiply_by_eight`).
pub fn mana_reflection() -> Arc<CardData> {
    CardDataBuilder::new("Mana Reflection")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 4))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text("If you tap a permanent for mana, it produces twice as much of that mana instead.")
        .ability(static_replacement(
            ReplacementDef::new(
                tapped_for_mana(None),
                ObjectSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Nyxbloom Ancient — {4}{G}{G}{G}
/// Enchantment Creature — Elemental, 5/5
///
/// > Trample
/// > If you tap a permanent for mana, it produces three times as much of
/// > that mana instead.
///
/// **The second factor the arm has seen**, and a creature, so the ability
/// exists only while the body does. Registered and unpooled: the same path
/// as Mana Reflection at seven mana, with a 5/5 trample body that would move
/// the combat rows by design.
///
/// # The rulings, and where each is tested
///
/// Its four rulings are Mana Reflection's four with the number changed, so
/// three of them are tested once, above. The fourth is its own: *"The
/// effects of multiple Nyxbloom Ancients are cumulative. For example, if you
/// have two Nyxbloom Ancients on the battlefield, you'll get nine times the
/// original amount and type of mana."* → nine, no prompt
/// (`two_nyxbloom_ancients_produce_nine_times_the_mana`) — and the two cards
/// together are six, the mixed pair the commuting cell was written for
/// (`a_reflection_and_an_ancient_produce_six_times_the_mana`).
pub fn nyxbloom_ancient() -> Arc<CardData> {
    CardDataBuilder::new("Nyxbloom Ancient")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green, ManaType::Green], 4))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elemental))
        .power_toughness(5, 5)
        .keyword_flag(KeywordFlag::Trample)
        .rules_text(
            "Trample\nIf you tap a permanent for mana, it produces three times as much of that mana instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                tapped_for_mana(None),
                ObjectSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(3)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Deep Water — {U}{U}
/// Enchantment
///
/// > {U}: Until end of turn, if you tap a land you control for mana, it
/// > produces {U} instead of any other type.
///
/// **CR 106.12b's "of a specific type", and the one of its six printed cards
/// that registers whole today.** An activated ability whose effect is Fog's
/// shape — `Primitive::CreateReplacement` until end of turn — with
/// `Instead(ProduceMana { Blue, ReplacedAmount })`: every unit retyped, the
/// amount kept. "A land you control" is the pattern's `source` filter, asked
/// of the tapped permanent at the proposal, and it is on the pattern rather
/// than in `affected_objects` because the event's subject is the player.
///
/// # The rulings, and where each is tested
///
/// - *"The amount of mana produced is unchanged, but it will all be {U}."* →
///   a land producing two of one color produces two blue
///   (`deep_water_keeps_the_amount_and_changes_the_type`), and a restricted
///   unit keeps its restriction with its new color
///   (`a_retyped_restricted_unit_keeps_its_restriction`).
/// - *"Deep Waters affects lands you control when it resolves and any lands
///   you gain control of this turn."* → the filter is evaluated at each
///   proposal, not captured when the row is made: a land that arrives under
///   your control after the activation is retyped
///   (`deep_water_reaches_a_land_gained_after_the_activation`).
///
/// Beside Mana Reflection the two commute — retype-then-double and
/// double-then-retype are one event — so a Forest under both adds {U}{U} with
/// no CR 616.1 prompt (`deep_water_and_mana_reflection_commute`).
pub fn deep_water() -> Arc<CardData> {
    CardDataBuilder::new("Deep Water")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text(
            "{U}: Until end of turn, if you tap a land you control for mana, it produces {U} instead of any other type.",
        )
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::Mana(ManaCost::build(&[ManaType::Blue], 0))],
            effect: Effect::Atom(
                Primitive::CreateReplacement(
                    Box::new(
                        ReplacementDef::new(
                            tapped_for_mana(Some(ObjectFilter::And(
                                Box::new(ObjectFilter::ByType(CardType::Land)),
                                Box::new(ObjectFilter::ByController(PlayerRef::You)),
                            ))),
                            ObjectSet::NO_OBJECTS,
                            Rewrite::Instead(GameActionTemplate::ProduceMana {
                                mana_type: ManaType::Blue,
                                amount: TemplateAmount::ReplacedAmount,
                            }),
                        )
                        .affecting_players(PlayerSet::You),
                    ),
                    Duration::UntilEndOfTurn,
                    PatternFill::Authored,
                ),
                EffectRecipient::Implicit,
            ),
        })
        .build()
}
