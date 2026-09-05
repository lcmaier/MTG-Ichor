//! Phase LH — attachment as a layers input (`layers-architecture.md` §13a).
//!
//! The first registered Aura. Every faithful Aura's text is about its host,
//! and until LH-1 no `AffectedSet` could name the host: `static_affected_set`
//! lowered "creatures you control" and "this permanent" and nothing else, and
//! `register_static_effects` runs before the resolution attaches the Aura, so
//! even a captured set would have been empty. `AffectedSet::Host` is the arm
//! this card is the consumer of.
//!
//! It is also the card that makes CR 608.3b reachable. Three functions used
//! to derive a spell's recipient from its *effect* — the CR 601.2c castability
//! pre-check, the CR 601.2c target selection and the CR 608.2b fizzle — and an
//! Aura has no spell ability, so none of them could see `enchant_filter`
//! (`codebase-state.md` Deferred Migrations item 8). The fix could not be shown
//! failing until a registered card carried one.

use std::sync::Arc;

use crate::objects::card_data::{ActivationRestriction, AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{ArtifactType, CardType, EnchantmentType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, PermanentFilter, PlayerRef, Primitive,
    SelectionFilter, TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::keywords::KeywordFlag;
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
/// `AffectedSet::Host`, so the first row whose membership is a
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
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ModifyPowerToughness(
                    AmountExpr::Fixed(1),
                    AmountExpr::Fixed(2),
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::Host,
            ),
        })
        .build()
}

/// "Equip {N}" — CR 702.6a's activated ability, spelled out: "{N}: Attach this
/// permanent to target creature you control. Activate only as a sorcery."
///
/// The recipient carries the whole of "creature you control", so CR 608.2b's
/// re-check at resolution is what CR 301.5b means by "control of the creature
/// matters ... when it resolves": a creature stolen in response is an illegal
/// target and the Equipment doesn't move (CR 701.3b). The equip ability
/// carries no equip-specific quality (CR 702.6c); that is a card-breadth
/// variant of this filter, not a new shape.
pub fn equip(generic: u8) -> AbilityDef {
    AbilityDef {
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::OnlyAsSorcery,
        id: new_ability_id(),
        ability_type: AbilityType::Activated,
        costs: vec![Cost::Mana(ManaCost::build(&[], generic))],
        effect: Effect::Atom(
            Primitive::Attach,
            EffectRecipient::Target(
                SelectionFilter::Permanent(PermanentFilter::And(
                    Box::new(PermanentFilter::ByType(CardType::Creature)),
                    Box::new(PermanentFilter::ByController(PlayerRef::You)),
                )),
                TargetCount::Exactly(1),
            ),
        ),
    }
}

/// Bonesplitter — {1}
/// Artifact — Equipment
///
/// Equipped creature gets +2/+0.
/// Equip {1}
///
/// (Oracle text verified on Scryfall, 2026-09-04. Mirrodin; many reprints.)
///
/// # Why this one
///
/// The first registered Equipment, and the consumer of `Primitive::Attach`,
/// `GameAction::Attach` and `ActivationRestriction::OnlyAsSorcery` — the
/// reattachment path that makes CR 613.7e observable at all, since an Aura
/// attaches once. **Asymmetric** for the reason Holy Strength's doc gives:
/// +2/+0 fails an assertion if the row is transposed or applied twice, where
/// Leonin Scimitar's and Short Sword's +1/+1 pass both. One generic mana to
/// cast and one to equip is the cheapest that leaves.
///
/// # What it makes reachable
///
/// - **CR 702.6a / 602.5d** — an activated ability with a timing restriction,
///   the first in the pool; the random agent is offered it only at sorcery
///   speed.
/// - **CR 613.7e / 701.3c** — a permanent whose CR 613.7 timestamp changes
///   after it entered, which is why `BattlefieldEntity.timestamp` split from
///   `entry_timestamp` (`layers-architecture.md` §13a, LH-2).
/// - **CR 704.5p** — the Equipment detach SBA, measured at **0** in the
///   2026-09-01 fuzz re-audit because nothing could attach an Equipment.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// A new primitive, a new `GameAction`, and the first activation restriction:
/// three engine paths no other pooled card opens, and §3 asks that each be
/// measured rather than assumed.
pub fn bonesplitter() -> Arc<CardData> {
    CardDataBuilder::new("Bonesplitter")
        .card_type(CardType::Artifact)
        .subtype(Subtype::Artifact(ArtifactType::Equipment))
        .mana_cost(ManaCost::build(&[], 1))
        .rules_text("Equipped creature gets +2/+0.
Equip {1}")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ModifyPowerToughness(
                    AmountExpr::Fixed(2),
                    AmountExpr::Fixed(0),
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::Host,
            ),
        })
        .ability(equip(1))
        .build()
}

/// Skyhook Harness (invented) — {1}
/// Artifact — Equipment
///
/// Equipped creature has flying.
/// Equip {1}
///
/// A test fixture, never registered. CR 613.7e is unobservable through
/// Bonesplitter alone — Layer 7c additions commute — so the rule is pinned
/// with a Layer 6 grant against Humility's "lose all abilities": the
/// Equipment enters first, Humility second, and only the timestamp the equip
/// gives it (CR 613.7e) lets its grant apply after Humility.
pub fn equipment_granting_flying() -> Arc<CardData> {
    CardDataBuilder::new("Skyhook Harness")
        .card_type(CardType::Artifact)
        .subtype(Subtype::Artifact(ArtifactType::Equipment))
        .mana_cost(ManaCost::build(&[], 1))
        .rules_text("Equipped creature has flying.\nEquip {1}")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::GrantKeywordFlag(KeywordFlag::Flying, Duration::WhileSourceOnBattlefield),
                EffectRecipient::Host,
            ),
        })
        .ability(equip(1))
        .build()
}
