//! Phase LH — attachment as a layers input (`layers-architecture.md` §13a).
//!
//! The first registered Aura. Every faithful Aura's text is about its host,
//! and until LH-1 no `AffectedSet` could name the host: `static_affected_set`
//! lowered "creatures you control" and "this permanent" and nothing else, and
//! `register_static_effects` runs before the resolution attaches the Aura, so
//! even a captured set would have been empty. `AffectedSet::AttachedToSource`
//! is the arm this card is the consumer of.
//!
//! It is also the card that makes CR 608.3b reachable. Three functions used
//! to derive a spell's recipient from its *effect* — the CR 601.2c castability
//! pre-check, the CR 601.2c target selection and the CR 608.2b fizzle — and an
//! Aura has no spell ability, so none of them could see `enchant_filter`
//! (`codebase-state.md` Deferred Migrations item 8). The fix could not be shown
//! failing until a registered card carried one.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, EnchantmentType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Duration, Effect, EffectRecipient, Primitive, SelectionFilter};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};

/// Holy Strength — {W}
/// Enchantment — Aura
///
/// Enchant creature
/// Enchanted creature gets +1/+2.
///
/// (Oracle text verified on Scryfall, 2026-09-04. Alpha; Magic 2011 printing.)
///
/// # Why this one out of 26
///
/// Scryfall lists 26 Auras whose entire oracle text is "Enchant creature" and
/// one P/T sentence, and they differ only in cost and sign. Two things pick
/// this one. **The bonus is positive**, so the host survives the attach and
/// the Aura stays on the battlefield to be the *subject* of CR 704.5m/n
/// rather than dying with a host it killed (Dead Weight, Weakness). **And it
/// is asymmetric**: +1/+2 is the smallest bonus where a row that swapped
/// power for toughness, or applied twice, would fail an assertion — River's
/// Favor's +1/+1 would pass both. One white pip is the cheapest that leaves.
///
/// # What it makes reachable
///
/// - **CR 704.5m/n** — the Aura SBAs, measured at **0** across 200 stress games
///   in the 2026-09-01 fuzz re-audit because no registered card was an Aura.
/// - **CR 303.4a / 601.2c** — an Aura spell's target is defined by its enchant
///   ability, not by a spell ability, and this is the first spell in the pool
///   whose recipient comes from `enchant_filter`.
/// - **CR 608.3b** — a permanent spell whose target is gone fizzles. Item 8.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The first static ability in the pool that lowers to
/// `AffectedSet::AttachedToSource`, so the first row whose membership is a
/// `battlefield` read per candidate per layer rather than a filter match. That
/// is a new arm in `effect_applies_to`, and §3 asks that a new engine path be
/// measured rather than assumed.
pub fn holy_strength() -> Arc<CardData> {
    CardDataBuilder::new("Holy Strength")
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .rules_text("Enchant creature\nEnchanted creature gets +1/+2.")
        // CR 702.5a — "Enchant creature" is the Aura's targeting restriction,
        // and CR 303.4a makes it the spell's target.
        .enchant_filter(SelectionFilter::Creature)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ModifyPowerToughness(
                    AmountExpr::Fixed(1),
                    AmountExpr::Fixed(2),
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::AttachedToSource,
            ),
        })
        .build()
}
