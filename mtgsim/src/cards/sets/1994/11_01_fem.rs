//! Fallen Empires (`fem`), released 1994-11-01.

use std::sync::Arc;

use crate::cards::authoring::{discard, spell_ability, target_player};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::DiscardChooser;
use crate::types::mana::{ManaCost, ManaType};

/// Hymn to Tourach — {B}{B}
/// Sorcery
///
/// > Target player discards two cards at random.
///
/// **CR 701.9b's second chooser, and the engine's first "at random" anything
/// that is not a shuffle.** The cards come from `GameState::rng` and never from
/// `rand::rng()` (`CLAUDE.md`: randomness is owned, never ambient), which is
/// what `tests/determinism_test.rs` covers by construction — its decks are
/// built from `default_registry`, so a registered Hymn is in every seeded game
/// it plays, and an ambient draw would make the second run of a process differ
/// from the first.
///
/// **No `DecisionProvider` is asked at all**, which is `ATOM-701.9b-001`'s own
/// expected result and not an optimization: the rule gives the affected player
/// no choice to make.
///
/// No printed rulings (Scryfall, 2026-09-14).
pub fn hymn_to_tourach() -> Arc<CardData> {
    CardDataBuilder::new("Hymn to Tourach")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Sorcery)
        .rules_text("Target player discards two cards at random.")
        .ability(spell_ability("Target player discards two cards at random.", discard(target_player(), 2, DiscardChooser::AtRandom)))
        .build()
}
