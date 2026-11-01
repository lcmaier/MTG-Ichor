//! Time Spiral (`tsp`), released 2006-10-06.

use std::sync::Arc;

use crate::cards::authoring::static_replacement;
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};

/// Scarwood Treefolk — {3}{G}
/// Creature — Treefolk, 3/5
/// "This creature enters tapped."
///
/// **The Jailer's partner, and the half of ATOM-614.12-001 that asks the
/// question.** The atom's board is the two of them, the Treefolk in a
/// graveyard, and the assertion is that reanimating it puts it onto the
/// battlefield **tapped** — even though, sitting in the graveyard, the Jailer
/// had taken its abilities away.
///
/// CR 614.12 is why: an entry replacement is checked against the permanent as
/// it *would exist on the battlefield*, and on the battlefield the Jailer does
/// not reach it. `Board::in_zones_or_entering` is that sentence in one line —
/// an entering object is admitted by a row iff the row reaches the
/// battlefield, whatever zone it is still sitting in.
///
/// A creature rather than `printings::idyllic_beachfront`'s land, and
/// that matters: the land's "enters tapped" is stripped by Blood Moon through
/// CR 305.7, which is a *battlefield*-scoped effect and so tests the opposite
/// leg. This one is only reachable through a graveyard.
pub fn scarwood_treefolk() -> Arc<CardData> {
    CardDataBuilder::new("Scarwood Treefolk")
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Treefolk))
        .power_toughness(3, 5)
        .rules_text("This creature enters tapped.")
        .ability(static_replacement(
            "This creature enters tapped.",
            crate::types::replacement::ReplacementDef::new(
                crate::types::replacement::EventPattern::EnterBattlefield { cast: None },
                crate::types::effects::ObjectSet::SourceOnly,
                crate::types::replacement::Rewrite::EnterWith(
                    crate::types::replacement::EnterModsTemplate::tapped(),
                ),
            ),
        ))
        .build()
}
