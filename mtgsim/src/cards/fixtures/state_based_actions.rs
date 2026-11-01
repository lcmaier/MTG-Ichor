//! Fixtures for state-based actions (CR 704), which no architecture doc owns.

use std::sync::Arc;

use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::mana::ManaCost;

/// Loyalty Probe — a fixture planeswalker, and CR 120.3c's consumer.
///
/// **Not a printed card**: a printed planeswalker's loyalty abilities have no
/// `AbilityType` and no activation path, so a real one would be a card with
/// dead abilities wearing a real name. Printed loyalty 3, no abilities at all,
/// `{2}` so any deck can cast it.
///
/// Three is chosen rather than five: `SelectionFilter::Any` enumerates
/// planeswalkers, so a random agent's Lightning Bolt finds this one, and three
/// damage takes it to exactly zero — which is what makes **CR 704.5i**
/// reachable from a fuzz game for the first time (the state-based actions'
/// phase measured it at 0 across 200 stress games, blocked on "loyalty
/// abilities + CR 120.3c"; RD-1 supplies the second half and the first is
/// combat's and Phase 8's).
///
/// It has no subtype. CR 205.3j gives every printed planeswalker one, and every
/// name in `PlaneswalkerType` belongs to a real character — so borrowing one
/// would put an invented card under a real planeswalker's name, which is
/// exactly the thing `engineering-practices.md#cards-and-pools` keeps out of
/// the registry.
///
/// Attacking one stays out of reach: `combat::validation` refuses planeswalker
/// attack targets, and that is combat's phase rather than this one's.
pub fn loyalty_probe() -> Arc<CardData> {
    CardDataBuilder::new("Loyalty Probe")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Planeswalker)
        .loyalty(3)
        .rules_text("")
        .build()
}
