//! Cards for the characteristic-defining abilities phase (CR 604.3 / 613.3 /
//! 613.4a).
//!
//! Both cards here set `is_characteristic_defining` on a printed static
//! ability. Nothing else has to happen: `register_static_effects` skips CDAs,
//! and `engine::layers::cda` applies them off the object's own effective
//! ability list at Layers 4, 5 and 7a. See that module for why a CDA is not a
//! registry effect.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, ColorChange, Duration, Effect, EffectRecipient, Primitive, Selector,
};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};

/// Tarmogoyf — {1}{G}
/// Creature — Lhurgoyf
/// */1+*
/// Tarmogoyf's power is equal to the number of card types among cards in all
/// graveyards and its toughness is equal to that number plus 1.
///
/// The Layer 7a card (CR 613.4a). Three things about how it is authored:
///
/// **Printed P/T is `(0, 1)`, not `(0, 0)`.** The box reads `*/1+*`: the `*` is
/// what the CDA supplies and resolves to 0 when nothing supplies it (CR 208.2a),
/// but the `1+` is printed arithmetic and survives on its own. The Merfolk
/// Trickster ruling says so in as many words — "If the target creature has power
/// and toughness written as */* with an ability that defines its power and
/// toughness, it's 0/0 when it loses all abilities. If its power and toughness
/// are written as */*+1, it's 0/1, and so on."
///
/// This is load-bearing rather than cosmetic. Strip Tarmogoyf's abilities
/// without setting a P/T and it is a 0/1, which survives state-based actions;
/// authored as 0/0 it would die. (Under Humility the answer is 1/1 either way,
/// because Humility's Layer 7b half sets one.)
///
/// **Card types, not cards.** Ten artifact creatures in a graveyard are still
/// two card types — hence `CardTypesAmong` rather than `CountOf`.
///
/// **All graveyards, including its own.** `Selector::CardsInAllGraveyards`
/// spans every player, and a Tarmogoyf in a graveyard counts itself.
pub fn tarmogoyf() -> Arc<CardData> {
    let card_types_in_graveyards =
        || AmountExpr::CardTypesAmong(Selector::CardsInAllGraveyards);

    CardDataBuilder::new("Tarmogoyf")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Lhurgoyf))
        .power_toughness(0, 1)
        .rules_text(
            "Tarmogoyf's power is equal to the number of card types among cards in all \
             graveyards and its toughness is equal to that number plus 1.",
        )
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::SetPowerToughness(
                    card_types_in_graveyards(),
                    AmountExpr::Plus(Box::new(card_types_in_graveyards()), 1),
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::Implicit,
            ),
            is_characteristic_defining: true,
        })
        .build()
}

/// Culling Drone — {1}{B}
/// Creature — Eldrazi Drone
/// 2/2
/// Devoid (This card has no color.)
///
/// The Layer 5 CDA, and CR 702.114a says so in as many words: "Devoid is a
/// characteristic-defining ability." The colored mana cost is the point — the
/// card would be black, and the CDA overrides that in Layer 5, before anything
/// in Layer 6 could remove the ability.
///
/// **Ingest omitted.** The printed card also has Ingest, a triggered ability
/// (CR 702.113a); triggered abilities are Phase 7. It has nothing to do with
/// what this card is here to test.
pub fn culling_drone() -> Arc<CardData> {
    CardDataBuilder::new("Culling Drone")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Eldrazi))
        .subtype(Subtype::Creature(CreatureType::Drone))
        .power_toughness(2, 2)
        .rules_text("Devoid (This card has no color.)")
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ChangeColor(ColorChange::RemoveAll, Duration::WhileSourceOnBattlefield),
                EffectRecipient::Implicit,
            ),
            is_characteristic_defining: true,
        })
        .build()
}
