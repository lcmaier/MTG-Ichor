//! Dominaria (`dom`), released 2018-04-27.

use std::sync::Arc;

use crate::cards::authoring::{dies, gets, host, return_to_hand, static_ability, triggered_ability, triggering_object, whenever};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, EnchantmentType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::SelectionFilter;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{EventRole, TriggerSubject};

/// Demonic Vigor — {B}
/// Enchantment — Aura
///
/// > Enchant creature
/// > Enchanted creature gets +1/+1.
/// > When enchanted creature dies, return that card to its owner's hand.
///
/// A leaves-the-battlefield ability (CR 603.10a), so it looks back: the Aura
/// stays on the battlefield, unattached, until CR 704.5m puts it into the
/// graveyard, and what it enchanted is read off the creature's departure,
/// which framed what was attached to it. "That card" is the one the creature
/// became in the graveyard (CR 603.6e, 400.7).
///
/// It exercises an Aura's "when enchanted creature dies", asked of its host's
/// departure, which framed what was attached to it.
///
/// # The ruling, and where it is tested
///
/// In `phase_tr4a_integration_test`.
/// - *"Demonic Vigor can enchant a token, but its last ability won't return
///   the token to your hand."* (#1) →
///   `demonic_vigor_triggers_for_a_token_and_returns_nothing`.
pub fn demonic_vigor() -> Arc<CardData> {
    let text = "When enchanted creature dies, return that card to its owner's hand.";
    CardDataBuilder::new("Demonic Vigor")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .rules_text("Enchant creature\nEnchanted creature gets +1/+1.\nWhen enchanted creature dies, return that card to its owner's hand.")
        .enchant_filter(SelectionFilter::Creature)
        .ability(static_ability(
            "Enchanted creature gets +1/+1.",
            gets(host(), 1, 1),
        ))
        .ability(triggered_ability(
            text,
            whenever(
                dies(TriggerSubject::Host),
                return_to_hand(triggering_object(EventRole::Object)),
            ),
        ))
        .build()
}
