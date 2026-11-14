//! Stronghold (`sth`), released 1998-03-02.

use std::sync::Arc;

use crate::cards::authoring::{counter_spell, its_controller, spell_ability, target_spell, the_target};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::mana::{ManaCost, ManaType};

/// Mana Leak — {1}{U}
/// Instant
///
/// > Counter target spell unless its controller pays {3}.
///
/// CR 118.12a's sentence: "its controller may pay {3}. If they don't, counter
/// target spell", the payment offered as Mana Leak resolves and only to a
/// controller who can pay (CR 118.3), who may activate mana abilities to pay
/// it (CR 605.3a).
///
/// It exercises a payment asked of the player an instance of "target" names,
/// and the mana window at resolution.
///
/// Scryfall lists no rulings (2026-11-13).
pub fn mana_leak() -> Arc<CardData> {
    let text = "Counter target spell unless its controller pays {3}.";
    let three = vec![Cost::Mana(ManaCost::build(&[], 3))];
    CardDataBuilder::new("Mana Leak")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, counter_spell(target_spell()).unless(its_controller(the_target()), three)))
        .build()
}
