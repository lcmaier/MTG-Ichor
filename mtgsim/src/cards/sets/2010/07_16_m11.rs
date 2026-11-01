//! Magic 2011 (`m11`), released 2010-07-16.

use std::sync::Arc;

use crate::cards::authoring::{activated_ability, creature, draw_cards, each_player, scry, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::mana::{ManaCost, ManaType};

/// Temple Bell — {3}
/// Artifact
///
/// > {T}: Each player draws a card.
///
/// One instruction to every player, which CR 121.2c performs in APNAP order:
/// the active player draws first. No rulings.
///
/// It exercises "each player draws", in APNAP order (CR 121.2c).
pub fn temple_bell() -> Arc<CardData> {
    CardDataBuilder::new("Temple Bell")
        .mana_cost(ManaCost::build(&[], 3))
        .card_type(CardType::Artifact)
        .rules_text("{T}: Each player draws a card.")
        .ability(activated_ability(
            "{T}: Each player draws a card.",
            vec![Cost::TapSelf],
            draw_cards(each_player(), 1),
        ))
        .build()
}

/// Viscera Seer — {B}
/// Creature — Vampire Wizard, 1/1
///
/// > Sacrifice a creature: Scry 1.
///
/// The sacrifice is a cost (CR 602.2b), so the creature is gone before the
/// ability is on the stack and its death triggers go above it.
///
/// It exercises an activated ability whose cost sacrifices a chosen creature.
///
/// # Not in `PERFORMANCE_POOL`
///
/// A random agent takes the activation at nearly every priority and
/// sacrifices its own creatures with it, so the pool's boards would thin out
/// (the owner, at #269's review). `stress`, which drives the engine every way
/// it can be driven, plays it.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr7_integration_test`.
/// - *It can sacrifice itself* (#1) → `viscera_seer_can_sacrifice_itself`.
/// - *A creature sacrificed after blockers are declared deals no combat
///   damage* (#2) → `a_blocked_creature_sacrificed_to_the_seer_deals_no_combat_damage`.
pub fn viscera_seer() -> Arc<CardData> {
    let text = "Sacrifice a creature: Scry 1.";
    CardDataBuilder::new("Viscera Seer")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Vampire))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(1, 1)
        .rules_text(text)
        .ability(activated_ability(
            text,
            vec![Cost::Sacrifice(creature(), 1)],
            scry(you(), 1),
        ))
        .build()
}
