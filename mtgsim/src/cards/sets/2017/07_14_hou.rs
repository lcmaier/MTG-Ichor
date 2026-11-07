//! Hour of Devastation (`hou`), released 2017-07-14.

use std::sync::Arc;

use crate::cards::authoring::{artifact, choose_one, deal_damage, destroy, spell_ability, target_creature, target_permanent};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};

/// Abrade — {1}{R}
/// Instant
/// Choose one —
/// • Abrade deals 3 damage to target creature.
/// • Destroy target artifact.
///
/// (Oracle text verified on Scryfall, 2026-11-07.)
///
/// CR 700.2's plainest shape, "choose one" of two modes that each target, so
/// the board decides which can be chosen (CR 700.2a): with no creature and no
/// artifact it is not offered at all. Pooled, as the card that puts the modal
/// offer and its castability walk in a measured game.
///
/// # The rulings, and where each is tested
///
/// Scryfall lists none for this card (2026-11-07). Its modes are
/// `phase_modes_chosen_and_resolved_integration_test`'s, cast from hand.
pub fn abrade() -> Arc<CardData> {
    let text = "Choose one —\n• Abrade deals 3 damage to target creature.\n• Destroy target artifact.";
    CardDataBuilder::new("Abrade")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, choose_one([deal_damage(target_creature(), 3), destroy(target_permanent(artifact()))])))
        .build()
}
