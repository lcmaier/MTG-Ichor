//! Exodus (`exo`), released 1998-06-15.

use std::sync::Arc;

use crate::cards::authoring::{another, creature, enters, gain_life, triggered_ability, whenever, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};

/// Soul Warden — {W}
/// Creature — Human Cleric 1/1
///
/// > Whenever another creature enters, you gain 1 life.
///
/// The matcher's card: CR 603.6a's "another" is the `NotSource` leaf, and
/// the batch close is what makes two creatures entering together two
/// triggers rather than one and none.
///
/// It exercises the matcher at a batch's close: CR 603.6a's "another", two
/// entries in one window.
///
/// # The rulings, and where each is tested
///
/// - *"If this creature enters at the same time as one or more other
///   creatures, its ability will trigger for each of those other creatures."*
///   → `phase_tr1_integration_test::soul_warden_entering_beside_two_creatures_triggers_for_each_of_them`.
pub fn soul_warden() -> Arc<CardData> {
    CardDataBuilder::new("Soul Warden")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(1, 1)
        .rules_text("Whenever another creature enters, you gain 1 life.")
        .ability(triggered_ability("Whenever another creature enters, you gain 1 life.", whenever(
            enters(another(creature())),
            gain_life(you(), 1),
        )))
        .build()
}
