//! Phase TR-4d — control, countering, loss, and the records' missing facts
//! (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-15 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Multani's Presence | a trigger on a spell being countered, looking back at who cast it (CR 603.10e) | no |

use std::sync::Arc;

use crate::cards::authoring::{spell_is_countered, triggered_ability, whenever, Whose};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive};
use crate::types::mana::{ManaCost, ManaType};

/// Multani's Presence — {G}
/// Enchantment
///
/// > Whenever a spell you've cast is countered, draw a card.
///
/// "A spell you've cast" is who cast it, which an effect that took control of
/// the spell does not change (`StackEntry::caster`).
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
                Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller),
            ),
        ))
        .build()
}
