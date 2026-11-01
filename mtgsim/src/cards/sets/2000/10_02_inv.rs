//! Invasion (`inv`), released 2000-10-02.

use std::sync::Arc;

use crate::cards::authoring::{cant_for, color, creature, destroy, draw_cards, land, non, scry, spell_ability, target_permanent, that_target, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{AffectedSet, Effect};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::restriction::{ReplacementKindFilter, Restriction};

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
/// regeneration restriction is written once per clause. That is not a
/// workaround: CR 701.19c's effect is about each permanent, and a single atom
/// spanning two instances would need a recipient the CR never asks for.
///
/// Registered and unpooled: the board needs March of the Machines and Moonlace
/// Fixture together, which a random game does not assemble.
///
/// # The rulings, and where each is tested
///
/// Scryfall lists none for this card.
pub fn plague_spores() -> Arc<CardData> {
    let cant_regenerate = |recipient| {
        cant_for(
            recipient,
            Restriction::ApplyReplacement {
                kind: ReplacementKindFilter::Regeneration,
                // `Primitive::Restrict` overwrites the object set with the
                // resolution's targets — this atom's instance.
                affected: AffectedSet::NOTHING,
            },
        )
        .until_end_of_turn()
    };
    CardDataBuilder::new("Plague Spores")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Red], 4))
        .color(Color::Black)
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text("Destroy target nonblack creature and target land. They can't be regenerated.")
        .ability(spell_ability("Destroy target nonblack creature and target land. They can't be regenerated.", Effect::Sequence(vec![
            // The shields are stripped before the destruction, or CR 701.19a's
            // replacement would have already applied by the time the
            // restriction existed.
            cant_regenerate(target_permanent(creature().and(non(color(Color::Black))))),
            cant_regenerate(target_permanent(land())),
            destroy(that_target(0)),
            destroy(that_target(1)),
        ])))
        .build()
}
