//! Land tokens. One so far, and it is in the registry for the mana base.
//!
//! **Why a token is in a card registry.** `fuzz_games::random_deck` builds a
//! deck's mana from what the registry holds, and until this file every source
//! in it made one or two colours. That is why `--require` used to seed a deck's
//! colours from the required card's: a forced `{1}{G}{U}` in a mono-red deck
//! is included and never cast. A land that taps for any colour is the other
//! half of the fix — forced insertion plus any-colour mana casts the card
//! against a *random* board rather than a hand-picked one — and "add one mana
//! of any color" is not expressible today (`ManaType` has no any-colour
//! variant; `mana.rs::resolve_mana_effect` accepts only `ProduceMana` atoms).
//! Everywhere sidesteps that: five basic land types, five intrinsic abilities,
//! no rider. `backlog.md` owns the real any-colour producers.
//!
//! **Not a card, and the fixture rule that lets it in.** A token is not a card
//! (CR 111.1), so nothing here is registered as one in the sense a printed card
//! is. `engineering-practices.md` §3's rule for fixtures is the one that
//! applies: an invented fixture is fine so long as it does not wear a real
//! name while behaving differently, and is never cited as evidence about
//! printed Magic. Everywhere wears its real name and behaves as its text says.

use std::sync::Arc;

use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, LandType, Subtype};
use crate::types::mana::ManaType;

/// Everywhere
/// Token Land — Plains Island Swamp Mountain Forest
///
/// This land is a Plains, Island, Swamp, Mountain, and Forest.
/// ({T}: Add {W}, {U}, {B}, {R}, or {G}.)
///
/// (The token Overlord of the Hauntwoods creates. Oracle text verified on
/// Scryfall, 2026-09-03.)
///
/// # What it does not do
///
/// **It does not enter tapped.** "Create a *tapped* Everywhere token" is the
/// Overlord's creation instruction, not the token's own text, so the token
/// itself has no `EnterWith` — a copy of it, or one put onto the battlefield
/// by anything else, enters untapped. This is also what makes it the pool's
/// no-downside any-colour source, where a tapland would push every deck's
/// curve back a turn (RC-2's re-record measured exactly that).
///
/// # Why the mana abilities are written out
///
/// CR 305.6 makes them intrinsic to the five basic land types, and the closest
/// model to the token's text — a self-scoped Layer 4 static adding the five
/// subtypes, with 305.6 supplying the mana — is expressible today:
/// `AffectedSet::SourceOnly` is the recipient and `land_types::apply_add_subtype`
/// grants an intrinsic ability on a gain. It is not used because that grant is
/// *only* on a gain, deliberately, so that Urborg hitting a real Swamp does
/// not double its `{B}`; a printed basic type carries its ability on the
/// `CardData` instead, which is `dual_lands.rs`'s documented shortcut, and this
/// is the same land with five types instead of two. So: five printed subtypes,
/// five explicit abilities, and CR 305.7 strips all five under Blood Moon the
/// way it strips a dual's two (`token_lands_test.rs`).
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
