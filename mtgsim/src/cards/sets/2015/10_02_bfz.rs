//! Battle for Zendikar (`bfz`), released 2015-10-02.

use std::sync::Arc;

use crate::cards::authoring::{becomes_colorless, destroy, land, spell_ability, static_ability, target_permanent, this_object};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};

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
        .rules_text("Devoid\nIngest")
        .ability(static_ability(
            "Devoid",
            becomes_colorless(this_object()),
        ).characteristic_defining())
        .build()
}

/// Volcanic Upheaval — {3}{R}
/// Instant
/// Destroy target land.
pub fn volcanic_upheaval() -> Arc<CardData> {
    CardDataBuilder::new("Volcanic Upheaval")
        .card_type(CardType::Instant)
        .color(Color::Red)
        .mana_cost(ManaCost::build(&[ManaType::Red], 3))
        .rules_text("Destroy target land.")
        .ability(spell_ability(
            "Destroy target land.",
            destroy(target_permanent(land())),
        ))
        .build()
}
