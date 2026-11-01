//! Weatherlight (`wth`), released 1997-06-09.

use std::sync::Arc;

use crate::cards::authoring::{activated_ability, draw_cards, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::costs::Cost;
use crate::types::mana::{ManaCost, ManaType};

/// Mind Stone — {2}
/// Artifact
///
/// {T}: Add {C}.
/// {1}, {T}, Sacrifice this artifact: Draw a card.
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// **The CR 732.1 board.** Its second ability adds no mana, so it was never a
/// mana ability under either rules text: activating it uses the stack and
/// opens the same 601.2g window a cast does. Sacrifice Mind Stone to
/// Krark-Clan Ironworks *inside* that window and the activation can no longer
/// pay its own cost — its own source is eaten out from under it.
///
/// What the engine does with that is nothing, and deliberately: the cost is
/// checked before any of it is paid, so the activation rewinds with no
/// payment to cancel, and the Ironworks activation — legal when it happened —
/// stands with its mana. Offering CR 732.1's *reversal* of that mana ability
/// is `codebase-state.md` item 72's, and where the surviving trigger goes on
/// the stack is critical-path item 6's.
///
/// The cost list is printed mana-first, which is the convention and not a
/// rule; [`self_eating_engine`] is the fixture that prints it the other way.
pub fn mind_stone() -> Arc<CardData> {
    CardDataBuilder::new("Mind Stone")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Artifact)
        .rules_text("{T}: Add {C}.\n{1}, {T}, Sacrifice this artifact: Draw a card.")
        .mana_ability_single(ManaType::Colorless)
        .ability(activated_ability(
            "{1}, {T}, Sacrifice this artifact: Draw a card.",
            vec![
                Cost::Mana(ManaCost::build(&[], 1)),
                Cost::TapSelf,
                Cost::SacrificeSelf,
            ],
            draw_cards(you(), 1),
        ))
        .build()
}
