//! Strixhaven: School of Mages (`stx`), released 2021-04-23.

use std::sync::Arc;

use crate::cards::authoring::ward;
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};

/// Waterfall Aerialist — {3}{U}
/// Creature — Djinn Wizard, 3/1
///
/// > Flying
/// > Ward {2}
///
/// Ward (CR 702.21a) is a triggered ability whose effect is CR 118.12a's
/// payment: "counter that spell or ability unless that player pays {2}",
/// that player the opponent whose spell or ability targeted it.
///
/// It exercises a payment a trigger asks of another player, and the
/// trigger's counter of a spell or of an ability.
///
/// # The rulings, and where each is tested
///
/// - *"If a player casts a spell that targets multiple permanents their
///   opponent controls with ward, each of those ward abilities will trigger.
///   If that player doesn't pay for all of them, the spell will be
///   countered."* (#1) → `a_spell_targeting_two_ward_creatures_pays_for_each`.
pub fn waterfall_aerialist() -> Arc<CardData> {
    CardDataBuilder::new("Waterfall Aerialist")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 3))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Djinn))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(3, 1)
        .rules_text("Flying\nWard {2}")
        .keyword_flag(KeywordFlag::Flying)
        .ability(ward("Ward {2}", vec![Cost::Mana(ManaCost::build(&[], 2))]))
        .build()
}
