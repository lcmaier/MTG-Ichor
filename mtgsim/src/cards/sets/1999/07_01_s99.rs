//! Starter 1999 (`s99`), released 1999-07-01.

use std::sync::Arc;

use crate::cards::authoring::{blocks_a_creature, destroy, this_object, triggered_ability, triggering_object, whenever};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::EffectRecipient;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{EventRole, TriggerSubject};

/// Loyal Sentry — {W}
/// Creature — Human Soldier 1/1
///
/// > When this creature blocks a creature, destroy that creature and this
/// > creature.
///
/// One trigger for each creature it blocks (CR 509.3b), and one instruction
/// on both (CR 608.2f): the two are destroyed in one event.
///
/// It exercises a block read once for each attacker it blocks, and one
/// instruction on two objects.
///
/// # The ruling, and where it is tested
///
/// In `phase_tr5c_integration_test`.
/// - *"Loyal Sentry and the creature it blocks are destroyed before combat
///   damage is dealt. The blocked creature is destroyed even if Loyal Sentry
///   leaves the battlefield before its triggered ability resolves."* (#1) →
///   `loyal_sentry_destroys_both_before_combat_damage` and
///   `loyal_sentry_destroys_the_attacker_after_it_has_left`.
pub fn loyal_sentry() -> Arc<CardData> {
    let text = "When this creature blocks a creature, destroy that creature and this creature.";
    let both = EffectRecipient::Group(vec![triggering_object(EventRole::Attacker), this_object()]);
    CardDataBuilder::new("Loyal Sentry")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Soldier))
        .power_toughness(1, 1)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(blocks_a_creature(TriggerSubject::ThisObject, TriggerSubject::Any), destroy(both)),
        ))
        .build()
}
