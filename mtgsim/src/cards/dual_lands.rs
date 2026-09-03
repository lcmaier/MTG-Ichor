//! The lands `fuzz_games` builds a mana base from: the ten original dual
//! lands (Alpha), and Everywhere.
//!
//! Registered so `fuzz_games` has nonbasic lands to build a mana base from. Until
//! they existed, `random_deck` filled every land slot from a colour→basic table,
//! so no deck could contain a nonbasic land and Blood Moon — which is in the card
//! pool — had nothing to affect. CR 305.7, the most intricate code in Layer 4, had
//! no random-play coverage at all.
//!
//! **Why duals specifically.** They are nonbasic (no `Basic` supertype) while
//! carrying basic land *types*, which is exactly the shape CR 305.7 cares about:
//! Blood Moon sets a dual's subtypes to Mountain, which strips its land types and
//! the mana abilities that came with them, and grants the intrinsic `{T}: Add {R}`.
//! They need no machinery that does not already exist — two mana abilities and two
//! subtypes each.
//!
//! **One modelling shortcut, recorded rather than hidden.** CR 305.6 makes a
//! land's mana abilities *intrinsic to its basic land types* — the reminder text
//! in parentheses on the printed card is not rules text. We give each dual two
//! explicit `AbilityType::Mana` abilities instead, because base characteristics
//! are read straight from `CardData` and nothing derives abilities from printed
//! subtypes. The two models agree everywhere we can currently observe: CR 305.7
//! clears `chars.abilities` wholesale before granting the new intrinsic, and
//! Humility removes mana abilities from an animated land either way (per its
//! Scryfall ruling). Revisit if an effect ever needs to distinguish an intrinsic
//! ability from a printed one.

use std::sync::Arc;

use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, LandType, Subtype};
use crate::types::mana::ManaType;

/// Build a dual land from its two basic land types.
///
/// Subtype order follows the printed type line, which is also the order
/// `land_types::basic_land_types_sorted` would produce — irrelevant to behaviour,
/// but it keeps the fixture readable next to the real card.
fn dual(name: &str, first: LandType, second: LandType) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Land)
        .subtype(Subtype::Land(first))
        .subtype(Subtype::Land(second))
        .mana_ability_single(mana_for(first))
        .mana_ability_single(mana_for(second))
        .build()
}

/// The mana a basic land type produces (CR 305.6).
fn mana_for(land_type: LandType) -> ManaType {
    match land_type {
        LandType::Plains => ManaType::White,
        LandType::Island => ManaType::Blue,
        LandType::Swamp => ManaType::Black,
        LandType::Mountain => ManaType::Red,
        LandType::Forest => ManaType::Green,
        other => panic!("{other:?} is not a basic land type and produces no mana"),
    }
}

/// Tundra — Land — Plains Island
pub fn tundra() -> Arc<CardData> {
    dual("Tundra", LandType::Plains, LandType::Island)
}

/// Underground Sea — Land — Island Swamp
pub fn underground_sea() -> Arc<CardData> {
    dual("Underground Sea", LandType::Island, LandType::Swamp)
}

/// Badlands — Land — Swamp Mountain
pub fn badlands() -> Arc<CardData> {
    dual("Badlands", LandType::Swamp, LandType::Mountain)
}

/// Taiga — Land — Mountain Forest
pub fn taiga() -> Arc<CardData> {
    dual("Taiga", LandType::Mountain, LandType::Forest)
}

/// Savannah — Land — Forest Plains
pub fn savannah() -> Arc<CardData> {
    dual("Savannah", LandType::Forest, LandType::Plains)
}

/// Scrubland — Land — Plains Swamp
pub fn scrubland() -> Arc<CardData> {
    dual("Scrubland", LandType::Plains, LandType::Swamp)
}

/// Volcanic Island — Land — Island Mountain
pub fn volcanic_island() -> Arc<CardData> {
    dual("Volcanic Island", LandType::Island, LandType::Mountain)
}

/// Bayou — Land — Swamp Forest
pub fn bayou() -> Arc<CardData> {
    dual("Bayou", LandType::Swamp, LandType::Forest)
}

/// Plateau — Land — Mountain Plains
pub fn plateau() -> Arc<CardData> {
    dual("Plateau", LandType::Mountain, LandType::Plains)
}

/// Tropical Island — Land — Forest Island
pub fn tropical_island() -> Arc<CardData> {
    dual("Tropical Island", LandType::Forest, LandType::Island)
}

/// Everywhere
/// Token Land — Plains Island Swamp Mountain Forest
/// ({T}: Add {W}, {U}, {B}, {R}, or {G}.)
///
/// (Scryfall, 2026-09-03.) **A token in the real game, not a card** — the one
/// Overlord of the Hauntwoods creates. It is in the registry because it is the
/// only source of every color, which is what let `fuzz_games --require` stop
/// seeding a deck's colors from the required card's; real "add one mana of any
/// color" is not expressible yet (`backlog.md` §2.19). Test-only, like the
/// rest of this module.
///
/// **Its text box is reminder text, not rules text** (CR 207.2). The type line
/// is the whole object and the five mana abilities are CR 305.6's intrinsics,
/// written out for the reason in this module's doc; `rules_text` carries the
/// reminder text so the display shows what the token shows, and not the
/// `{T}: Add {W}.` the builder would otherwise invent from the first ability.
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
        .rules_text("({T}: Add {W}, {U}, {B}, {R}, or {G}.)")
        .mana_ability_single(ManaType::White)
        .mana_ability_single(ManaType::Blue)
        .mana_ability_single(ManaType::Black)
        .mana_ability_single(ManaType::Red)
        .mana_ability_single(ManaType::Green)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::phase_ld_cards;
    use crate::oracle::characteristics::{get_effective_abilities, get_effective_subtypes};
    use crate::test_support::{put_on_battlefield, setup_two_player_game};
    use crate::types::card_types::Supertype;
    use crate::types::effects::{Effect, Primitive};

    /// The interaction these cards exist to make reachable in random play, using
    /// the same factories the registry hands to `fuzz_games` rather than a
    /// hand-built fixture.
    ///
    /// CR 305.7: setting a land's subtypes to a basic land type strips its old
    /// land types and the abilities that came with them, and grants the intrinsic
    /// mana ability for the new type.
    #[test]
    fn blood_moon_turns_a_dual_into_a_mountain_that_taps_for_red() {
        let mut game = setup_two_player_game();
        let sea = put_on_battlefield(&mut game, underground_sea(), 0);

        let produced = |game: &_, id| -> Vec<ManaType> {
            get_effective_abilities(game, id)
                .iter()
                .filter_map(|a| match &a.effect {
                    Effect::Atom(Primitive::ProduceMana(out), _) => Some(out.mana[0].0),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(produced(&game, sea), vec![ManaType::Blue, ManaType::Black]);

        put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 1);

        assert_eq!(
            get_effective_subtypes(&game, sea)
                .into_iter()
                .collect::<Vec<_>>(),
            vec![Subtype::Land(LandType::Mountain)],
            "CR 305.7: the old land types are gone"
        );
        assert_eq!(
            produced(&game, sea),
            vec![ManaType::Red],
            "blue and black went with the land types, and red came with Mountain"
        );
    }

    #[test]
    fn duals_are_nonbasic_lands_with_two_types_and_two_mana_abilities() {
        for card in [
            tundra(), underground_sea(), badlands(), taiga(), savannah(),
            scrubland(), volcanic_island(), bayou(), plateau(), tropical_island(),
        ] {
            assert!(card.types.contains(&CardType::Land), "{}", card.name);
            // The whole reason these exist: CR 305.8 makes a land nonbasic on the
            // supertype alone, which is what Blood Moon's filter selects.
            assert!(
                !card.supertypes.contains(&Supertype::Basic),
                "{} must be nonbasic to be a Blood Moon target",
                card.name
            );
            assert_eq!(card.subtypes.len(), 2, "{}", card.name);
            assert_eq!(card.abilities.len(), 2, "{}", card.name);
        }
    }
}
