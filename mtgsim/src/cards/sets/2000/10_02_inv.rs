//! Invasion (`inv`), released 2000-10-02.

use std::sync::Arc;

use crate::cards::authoring::{
    choose_color, color, creature, destroy, draw_cards, each, land, non, return_to_hand, scry, spell_ability, the_chosen_color,
    target_permanent, you,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::Effect;
use crate::types::mana::{ManaCost, ManaType};

/// Opt — {U}
/// Instant
///
/// > Scry 1. (Look at the top card of your library. You may put that card on
/// > the bottom.)
/// >
/// > Draw a card.
///
/// **The scry producer, and CR 608.2c's "A, then B" in one resolution.** The
/// two instructions are an `Effect::Sequence`, so the scry is complete before
/// the draw is proposed — which is what makes the board with Eligeth,
/// Crossroads Augur say something: under Eligeth the first instruction
/// becomes a draw too, and the card draws two.
///
/// A scry 1 asks exactly one question — which is CR 701.22a's "put any number
/// of them on the bottom" with one card — and never asks the ordering one,
/// because a group of one has no order to choose.
///
/// **The reminder text's "look at" is the same non-event as Nephalia Academy's
/// "reveal"**: this engine's decision providers see the whole board, so
/// looking changes nothing until §2.9 exists.
///
/// No printed rulings (Scryfall, 2026-09-14).
pub fn opt() -> Arc<CardData> {
    CardDataBuilder::new("Opt")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text("Scry 1.\nDraw a card.")
        .ability(spell_ability(
            "Scry 1.",
            scry(you(), 1),
        ))
        .ability(spell_ability(
            "Draw a card.",
            draw_cards(you(), 1),
        ))
        .build()
}

/// Plague Spores — {4}{B}{R}
/// Sorcery
/// Destroy target nonblack creature and target land. They can't be regenerated.
///
/// **Two instances with different criteria**, and `ATOM-608.2b-002`'s card: the
/// CR's own example of one object satisfying both clauses is an artifact
/// creature land, and when it turns black the creature clause goes illegal
/// while the land clause does not. That board is what proves the two rules of
/// CR 608.2b are different rules.
///
/// **"They" is two atoms, not one.** An atom reads its own instance, so the
/// clause is written once per destroy. That is not a workaround: CR 701.19c
/// is about each permanent's destruction, and a single atom spanning two
/// instances would need a recipient the CR never asks for. Each destroy
/// carries the clause itself, so it covers that destruction and no other.
///
/// Registered and unpooled: the board needs March of the Machines and Moonlace
/// Fixture together, which a random game does not assemble.
///
/// # The rulings, and where each is tested
///
/// Scryfall lists none for this card.
pub fn plague_spores() -> Arc<CardData> {
    CardDataBuilder::new("Plague Spores")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Red], 4))
        .color(Color::Black)
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text("Destroy target nonblack creature and target land. They can't be regenerated.")
        .ability(spell_ability("Destroy target nonblack creature and target land. They can't be regenerated.", Effect::Sequence(vec![
            destroy(target_permanent(creature().and(non(color(Color::Black))))).cant_be_regenerated(),
            destroy(target_permanent(land())).cant_be_regenerated(),
        ])))
        .build()
}

/// Wash Out — {3}{U}
/// Sorcery
///
/// > Return all permanents of the color of your choice to their owners'
/// > hands.
///
/// (Oracle text verified on Scryfall, 2026-11-15.)
///
/// A color chosen as the spell resolves (CR 105.4, 608.2d), then every
/// permanent of it, whoever controls it.
///
/// # The rulings, and where each is tested
///
/// Scryfall lists none for this card. Its board is
/// `phase_value_chosen_as_an_effect_resolves_integration_test::wash_out_returns_the_permanents_of_the_color_chosen_as_it_resolves`.
pub fn wash_out() -> Arc<CardData> {
    let text = "Return all permanents of the color of your choice to their owners' hands.";
    CardDataBuilder::new("Wash Out")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 3))
        .color(Color::Blue)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(text, Effect::Sequence(vec![choose_color(you()), return_to_hand(each(color(the_chosen_color())))])))
        .build()
}
