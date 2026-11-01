//! Named token definitions: a token printed as a card of its own, which the
//! registry deals like one. A creature token an effect describes ("a 1/1
//! white Soldier creature token") is no definition of its own: its maker
//! writes it with `TokenDef::creature`, since a token's characteristics are
//! that effect's (CR 111.3).

use std::sync::Arc;

use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, LandType, Subtype};
use crate::types::mana::ManaType;

/// Everywhere
/// Token Land
/// This land is a Plains, Island, Swamp, Mountain, and Forest. ({T}: Add {W},
/// {U}, {B}, {R}, or {G}.)
///
/// (Scryfall, 2026-10-01.) **A token in the real game, not a card** — the one
/// Overlord of the Hauntwoods creates. It is in the registry because it is the
/// only source of every color, which is what let `fuzz_games --require` stop
/// seeding a deck's colors from the required card's; real "add one mana of any
/// color" is not expressible yet (`backlog.md` §2.19).
///
/// **Its land types are printed subtypes here, where Oracle now prints them as
/// text.** On 2026-09-03 the type line carried the five types; Oracle since
/// moved them into the text box, a characteristic-defining ability (CR 604.3)
/// that applies first in layer 4 (CR 613.3), so the base subtypes here read the
/// same under every effect in the registry. The five mana abilities are
/// CR 305.6's intrinsics, written out as the dual lands' are, since nothing
/// derives an ability from a printed subtype.
///
/// **It does not enter tapped.** "Create a *tapped* Everywhere token" is the
/// Overlord's instruction, not the token's text, so there is no `EnterWith` —
/// which is also what makes it a no-downside source where a tapland pushes
/// every deck's curve back a turn.
pub fn everywhere() -> Arc<CardData> {
    CardDataBuilder::new("Everywhere")
        .card_type(CardType::Land)
        .subtype(Subtype::Land(LandType::Plains))
        .subtype(Subtype::Land(LandType::Island))
        .subtype(Subtype::Land(LandType::Swamp))
        .subtype(Subtype::Land(LandType::Mountain))
        .subtype(Subtype::Land(LandType::Forest))
        .rules_text("This land is a Plains, Island, Swamp, Mountain, and Forest.")
        .mana_ability_single(ManaType::White)
        .mana_ability_single(ManaType::Blue)
        .mana_ability_single(ManaType::Black)
        .mana_ability_single(ManaType::Red)
        .mana_ability_single(ManaType::Green)
        .build()
}
