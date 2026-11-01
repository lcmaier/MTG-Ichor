//! Portal (`por`), released 1997-05-01.

use std::sync::Arc;

use crate::cards::authoring::{any_target, deal_damage, discard, spell_ability, target_player};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, DiscardChooser};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};

/// Blaze — {X}{R}
/// Sorcery
/// "Blaze deals X damage to any target."
///
/// No rulings (Scryfall, 2026-10-17).
///
/// It exercises the plain X spell: the gate at the least X, CR 601.2b's legal
/// values, 601.2f's expansion, and the announced X read at resolution.
pub fn blaze() -> Arc<CardData> {
    CardDataBuilder::new("Blaze")
        .card_type(CardType::Sorcery)
        .color(Color::Red)
        .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Red)]))
        .rules_text("Blaze deals X damage to any target.")
        .ability(spell_ability(
            "Blaze deals X damage to any target.",
            deal_damage(any_target(), AmountExpr::X),
        ))
        .build()
}

/// Mind Rot — {2}{B}
/// Sorcery
///
/// > Target player discards two cards.
///
/// **CR 701.9b's default chooser, and the pool's first discard outside the
/// cleanup step.** Two cards rather than one is what makes the batch
/// observable: a discard of two is one event with two members, which is what
/// CR 603.2c's "whenever one or more cards are discarded" will read once
/// rather than twice.
///
/// No printed rulings (Scryfall, 2026-09-14). The ruling it is tested against
/// belongs to another card: Notion Thief's *"the opponent still discards"* was
/// asserted in RE-2 against a fixture draw-then-discard resolution, because
/// `Primitive::Discard` did not exist. It is a printed board now — an opponent
/// draws under a Notion Thief and then Mind Rot takes two cards — and the
/// fixture goes.
pub fn mind_rot() -> Arc<CardData> {
    CardDataBuilder::new("Mind Rot")
        .mana_cost(ManaCost::build(&[ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Sorcery)
        .rules_text("Target player discards two cards.")
        .ability(spell_ability("Target player discards two cards.", discard(target_player(), 2, DiscardChooser::Affected)))
        .build()
}

/// Raging Cougar — {2}{R}
/// Creature — Cat
/// 2/2 Haste
pub fn raging_cougar() -> Arc<CardData> {
    CardDataBuilder::new("Raging Cougar")
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Cat))
        .color(Color::Red)
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .power_toughness(2, 2)
        .keyword_flag(KeywordFlag::Haste)
        .rules_text("Haste")
        .build()
}
