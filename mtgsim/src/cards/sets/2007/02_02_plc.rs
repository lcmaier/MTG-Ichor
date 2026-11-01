//! Planar Chaos (`plc`), released 2007-02-02.

use std::sync::Arc;

use crate::cards::authoring::{activated_ability, becomes_types, each, land, static_ability, switch_power_and_toughness, target_creature};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, LandType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::TypeChange;
use crate::types::mana::{ManaCost, ManaType};

/// Merfolk Thaumaturgist — {2}{U}
/// Creature — Merfolk Wizard (1/2)
/// `{T}: Switch target creature's power and toughness until end of turn.`
///
/// The pool's only Layer 7d source, and the reason it is a creature rather than
/// the instant that already existed as a fixture: `fixtures::inside_out_fixture`
/// simplifies a hybrid cost the engine cannot express, so it cannot be
/// registered without lying about the card (see its doc comment).
///
/// It is also the first `AbilityType::Activated` ability in the registry —
/// every other registered ability is a spell, a mana ability, or a static — so
/// it is what gives `activate_ability`'s stack path, its target selection, and
/// its rollback arms their first random-play exposure. The tap cost drags CR
/// 302.6 in with it.
pub fn merfolk_thaumaturgist() -> Arc<CardData> {
    CardDataBuilder::new("Merfolk Thaumaturgist")
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Merfolk))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .color(Color::Blue)
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .power_toughness(1, 2)
        .rules_text("{T}: Switch target creature's power and toughness until end of turn.")
        .ability(activated_ability(
            "{T}: Switch target creature's power and toughness until end of turn.",
            vec![Cost::TapSelf],
            switch_power_and_toughness(target_creature()).until_end_of_turn(),
        ))
        .build()
}

/// Urborg, Tomb of Yawgmoth
/// Legendary Land
/// Each land is a Swamp in addition to its other land types.
///
/// (Oracle text verified on Scryfall, 2026-09-06.)
///
/// The existence dependency (CR 613.8a(b)), on the card whose ruling states
/// it: "If an effect such as that of Magus of the Moon causes Urborg to lose
/// its abilities by setting it to a basic land type not in addition to its
/// other types, it won't turn lands into Swamps, no matter in what order
/// those effects started to apply" (2021-03-19). Urborg is a Legendary Land
/// and so nonbasic (CR 305.8 — the supertype alone decides), so Blood Moon
/// makes it a Mountain and CR 305.7 strips the very ability that generates
/// this effect. Applying Blood Moon changes whether Urborg's effect *exists*,
/// so Urborg waits for Blood Moon whatever their timestamps, and by then it
/// has nothing to apply.
///
/// No printed mana ability: Urborg taps for {B} only because it is itself a
/// land and so a Swamp under its own effect, which is CR 305.6's intrinsic
/// ability arriving through the additive clause of CR 305.7.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The pool's first card whose effect can depend on another's — Blood Moon
/// has been in the pool since Phase LD — and a land, so every deck that
/// draws it drops it. `random_deck` fills nonbasic land slots from the
/// registry's nonbasic lands, which is how it reaches a game at all. That
/// makes the dependency loop's slow path (`board::next_ready`) live in
/// a measured game, where before this card every layer in the pool was
/// pairwise independent under the static check.
pub fn urborg_tomb_of_yawgmoth() -> Arc<CardData> {
    CardDataBuilder::new("Urborg, Tomb of Yawgmoth")
        .supertype(Supertype::Legendary)
        .card_type(CardType::Land)
        .rules_text("Each land is a Swamp in addition to its other land types.")
        .ability(static_ability(
            "Each land is a Swamp in addition to its other land types.",
            becomes_types(each(land()), TypeChange::adding(&[], &[], &[Subtype::Land(LandType::Swamp)])),
        ))
        .build()
}
