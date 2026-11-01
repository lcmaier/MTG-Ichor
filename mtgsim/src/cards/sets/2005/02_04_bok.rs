//! Betrayers of Kamigawa (`bok`), released 2005-02-04.

use std::sync::Arc;

use crate::cards::authoring::{any_target, prevent_the_next_n_damage_this_turn, spell_ability};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};

/// Mending Hands — {W}
///
/// > Prevent the next 4 damage that would be dealt to any target this turn.
///
/// The plain CR 615.7 shield: one row, `NextDamage(4)`, the target filled in at
/// resolution as an object or a player, gone when the count reaches zero or at
/// the cleanup step (CR 615.3's "until they're used up or their duration has
/// expired"). Scryfall lists no rulings (2026-09-08), so its tests are the
/// rule's own: the count depletes per point across events (`ATOM-615.7-001`), a
/// 4 against 5 lets 1 through, two attackers into a shielded player are one
/// `allocate` prompt whose every answer prevents the same total
/// (`ATOM-615.7-002`), and a shield made after the damage prevents nothing
/// (`ATOM-615.4-001`).
///
/// **The pooled card of the phase**, because it is the first registry row a
/// pooled damage event meets and the first CR 615.7 prompt a fuzz game can
/// reach. Healing Salve was the canonical printing and is modal.
pub fn mending_hands() -> Arc<CardData> {
    CardDataBuilder::new("Mending Hands")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text("Prevent the next 4 damage that would be dealt to any target this turn.")
        .ability(spell_ability("Prevent the next 4 damage that would be dealt to any target this turn.",
            prevent_the_next_n_damage_this_turn(any_target(), 4),
        ))
        .build()
}
