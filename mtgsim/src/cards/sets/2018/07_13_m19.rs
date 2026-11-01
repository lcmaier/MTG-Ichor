//! Core Set 2019 (`m19`), released 2018-07-13.

use std::sync::Arc;

use crate::cards::authoring::{shuffles_into_library_from_anywhere, spell_ability, take_an_extra_turn_after_this_one, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};

/// Nexus of Fate — {5}{U}{U}
/// Instant
/// "Take an extra turn after this one.
///  If Nexus of Fate would be put into a graveyard from anywhere, reveal Nexus
///  of Fate and shuffle it into its owner's library instead."
///
/// The card that is never on the battlefield, which is the sentence that
/// makes it the right second shape: its static ability functions on the
/// stack by CR 113.6b's statement, and CR 113.6's first sentence would have
/// put an instant's abilities there anyway. Resolving is the famous case —
/// CR 608.2n's move to the graveyard is replaced and the card goes back into
/// the library it will be drawn from again — and being countered (CR 701.6)
/// or discarded is the same clause from a different zone.
///
/// **Registered, not pooled**, for the reason Time Walk is not: an extra
/// turn moves `Avg turns/game` by design, which is a worse baseline rather
/// than a wider one. `Primitive::ExtraTurn` is that card's, unchanged.
///
/// "Reveal Nexus of Fate" is not modeled: the engine has no per-viewer
/// visibility to reveal against (`backlog.md` §2.9), so the reveal is the
/// same no-op it is on Darksteel Colossus and Nephalia Academy, and this
/// card is one of §2.9's tests when it lands.
pub fn nexus_of_fate() -> Arc<CardData> {
    CardDataBuilder::new("Nexus of Fate")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 5))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(
            "Take an extra turn after this one.\nIf Nexus of Fate would be put into a \
             graveyard from anywhere, reveal Nexus of Fate and shuffle it into its owner's \
             library instead.",
        )
        .ability(spell_ability("Take an extra turn after this one.", take_an_extra_turn_after_this_one(you())))
        .ability(shuffles_into_library_from_anywhere("If Nexus of Fate would be put into a graveyard from anywhere, reveal Nexus of Fate and shuffle it into its owner's library instead.", ))
        .build()
}
