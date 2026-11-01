//! Urza's Legacy (`ulg`), released 1999-02-15.

use std::sync::Arc;

use crate::cards::authoring::{
    dies, draw_cards, gets, host, return_to_hand, spell_ability, spell_is_countered, static_ability, target_permanent, triggered_ability,
    triggering_object, untap, whenever, you, Whose,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, EnchantmentType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{ObjectFilter, SelectionFilter};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{EventRole, TriggerSubject};

/// Burst of Energy — {W}
/// Instant
/// Untap target permanent.
pub fn burst_of_energy() -> Arc<CardData> {
    CardDataBuilder::new("Burst of Energy")
        .card_type(CardType::Instant)
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .rules_text("Untap target permanent.")
        .ability(spell_ability(
            "Untap target permanent.",
            untap(target_permanent(ObjectFilter::All)),
        ))
        .build()
}

/// Multani's Presence — {G}
/// Enchantment
///
/// > Whenever a spell you've cast is countered, draw a card.
///
/// "A spell you've cast" is who cast it, which an effect that took control of
/// the spell does not change (`StackEntry::caster`).
///
/// It exercises a trigger on a spell being countered, looking back at who cast
/// it (CR 603.10e).
///
/// # The ruling, and where it is tested
///
/// In `phase_tr4d_integration_test`.
/// - *"A spell that doesn't resolve because its targets are illegal isn't
///   countered, so this ability won't trigger. This is a change from previous
///   rules."* (#1) → `a_spell_whose_targets_are_illegal_is_not_countered`.
pub fn multanis_presence() -> Arc<CardData> {
    let text = "Whenever a spell you've cast is countered, draw a card.";
    CardDataBuilder::new("Multani's Presence")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(
                spell_is_countered(Whose::Yours),
                draw_cards(you(), 1),
            ),
        ))
        .build()
}

/// Rancor — {G}
/// Enchantment — Aura
///
/// > Enchant creature
/// > Enchanted creature gets +2/+0 and has trample.
/// > When this Aura is put into a graveyard from the battlefield, return it
/// > to its owner's hand.
///
/// A trigger on the Aura itself going to the graveyard: "it" is the card it
/// became there (CR 400.7e), which the binding names since RH-3.
///
/// It exercises an Aura's trigger on itself going to the graveyard, finding the
/// card it became (CR 400.7e).
///
/// # The ruling, and where it is tested
///
/// In `phase_tr4c_integration_test`.
/// - *"If the creature this Aura would enchant is an illegal target by the
///   time Rancor tries to resolve, the Aura spell doesn't resolve. It won't
///   enter the battlefield, so it won't be put into a graveyard from the
///   battlefield and its ability won't trigger."* (#1) →
///   `rancor_whose_creature_left_before_it_resolved_does_not_trigger`.
pub fn rancor() -> Arc<CardData> {
    let anthem = "Enchanted creature gets +2/+0 and has trample.";
    let returns = "When this Aura is put into a graveyard from the battlefield, return it to its owner's hand.";
    CardDataBuilder::new("Rancor")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .enchant_filter(SelectionFilter::Creature)
        .rules_text("Enchant creature\nEnchanted creature gets +2/+0 and has trample.\nWhen this Aura is put into a graveyard from the battlefield, return it to its owner's hand.")
        .ability(static_ability(anthem, gets(host(), 2, 0).and_gains_keyword(KeywordFlag::Trample)))
        .ability(triggered_ability(
            returns,
            whenever(dies(TriggerSubject::ThisObject), return_to_hand(triggering_object(EventRole::Object))),
        ))
        .build()
}
