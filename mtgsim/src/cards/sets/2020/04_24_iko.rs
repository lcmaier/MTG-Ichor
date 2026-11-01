//! Ikoria: Lair of Behemoths (`iko`), released 2020-04-24.

use std::sync::Arc;

use crate::cards::authoring::static_replacement;
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::effects::{CounterType, ObjectSet};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::ManaCost;
use crate::types::replacement::{EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};

/// Adaptive Shimmerer — {5}
/// Creature — Insect, 0/0
///
/// Flash
/// This creature enters with three +1/+1 counters on it.
///
/// (Oracle text verified on Scryfall, 2026-09-01.)
///
/// # A 0/0 is the only board state that can falsify the ordering
///
/// CR 122.6a's counters go on inside the performer, before the entry is
/// announced and long before state-based actions next run. That ordering is a
/// claim, and every *other* card in either pool would pass whether it held or
/// not — a 3/3 that briefly reads 3/3 looks exactly like a 3/3 that reads 1/1.
/// **This one dies if the counters arrive a moment late**: a 0/0 is what
/// CR 704.5f is written about.
///
/// **Registered for `engineering-practices.md#how-many-cards-a-mechanic-owes`'s
/// sharpest finding**, which is not about counts: *a bespoke fixture can cover
/// an atom while the registered pool cannot build the same scenario*. The
/// ordering was asserted against a hand-built 0/0 before this card existed,
/// which proved the engine and proved nothing about the pool. Colorless, so
/// `random_deck` puts it in every deck in every game.
///
/// **Not in `PERFORMANCE_POOL`.** Registering a card and adding one to that pool
/// are different acts (§3), and RC-2's new engine path is already measured by
/// the two cards that are in it. This one grows the stress pool, which is read
/// as a threshold rather than a baseline.
///
/// Flash is a printed keyword the engine already enforces (`engine/cast.rs`
/// reads `KeywordFlag::Flash` for timing), so the card ships whole rather than
/// trimmed — which matters here, because a flashed-in 0/0 enters during an
/// opponent's turn and reaches the ordering from a second direction.
pub fn adaptive_shimmerer() -> Arc<CardData> {
    CardDataBuilder::new("Adaptive Shimmerer")
        .mana_cost(ManaCost::build(&[], 5))
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Insect))
        .power_toughness(0, 0)
        .keyword_flag(KeywordFlag::Flash)
        .rules_text("Flash\nThis creature enters with three +1/+1 counters on it.")
        .ability(static_replacement(
            "This creature enters with three +1/+1 counters on it.",
            ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::SourceOnly,
                Rewrite::EnterWith(EnterModsTemplate::with_counters(CounterType::PlusOnePlusOne, 3)),
            ),
        ))
        .build()
}
