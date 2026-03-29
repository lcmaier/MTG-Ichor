//! Phase 3 vanilla creature card definitions.
//!
//! These creatures have no abilities — they resolve to the battlefield as
//! permanents. The combat system uses their printed power/toughness.
//!
//! Card origins:
//! - Grizzly Bears — Alpha (1993), {1}{G} 2/2
//! - Hill Giant — Alpha (1993), {3}{R} 3/3
//! - Savannah Lions — Alpha (1993), {W} 2/1

use std::sync::Arc;

use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};

/// Grizzly Bears — {1}{G}
/// Creature — Bear
/// 2/2
pub fn grizzly_bears() -> Arc<CardData> {
    CardDataBuilder::new("Grizzly Bears")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::single(ManaType::Green, 1, 1))
        .power_toughness(2, 2)
        .build()
}

/// Hill Giant — {3}{R}
/// Creature — Giant
/// 3/3
pub fn hill_giant() -> Arc<CardData> {
    CardDataBuilder::new("Hill Giant")
        .card_type(CardType::Creature)
        .color(Color::Red)
        .mana_cost(ManaCost::single(ManaType::Red, 1, 3))
        .power_toughness(3, 3)
        .build()
}

/// Savannah Lions — {W}
/// Creature — Cat
/// 2/1
pub fn savannah_lions() -> Arc<CardData> {
    CardDataBuilder::new("Savannah Lions")
        .card_type(CardType::Creature)
        .color(Color::White)
        .mana_cost(ManaCost::single(ManaType::White, 1, 0))
        .power_toughness(2, 1)
        .build()
}

/// Earth Elemental — {3}{R}{R}
/// Creature — Elemental
/// 4/5
pub fn earth_elemental() -> Arc<CardData> {
    CardDataBuilder::new("Earth Elemental")
        .card_type(CardType::Creature)
        .color(Color::Red)
        .mana_cost(ManaCost::single(ManaType::Red, 2, 3))
        .power_toughness(4, 5)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grizzly_bears() {
        let bears = grizzly_bears();
        assert_eq!(bears.name, "Grizzly Bears");
        assert!(bears.types.contains(&CardType::Creature));
        assert_eq!(bears.mana_cost.as_ref().unwrap().mana_value(), 2);
        assert_eq!(bears.power, Some(2));
        assert_eq!(bears.toughness, Some(2));
    }

    #[test]
    fn test_hill_giant() {
        let giant = hill_giant();
        assert_eq!(giant.name, "Hill Giant");
        assert!(giant.types.contains(&CardType::Creature));
        assert_eq!(giant.mana_cost.as_ref().unwrap().mana_value(), 4);
        assert_eq!(giant.power, Some(3));
        assert_eq!(giant.toughness, Some(3));
    }

    #[test]
    fn test_savannah_lions() {
        let lions = savannah_lions();
        assert_eq!(lions.name, "Savannah Lions");
        assert!(lions.types.contains(&CardType::Creature));
        assert_eq!(lions.mana_cost.as_ref().unwrap().mana_value(), 1);
        assert_eq!(lions.power, Some(2));
        assert_eq!(lions.toughness, Some(1));
    }

    #[test]
    fn test_earth_elemental() {
        let elemental = earth_elemental();
        assert_eq!(elemental.name, "Earth Elemental");
        assert!(elemental.types.contains(&CardType::Creature));
        assert_eq!(elemental.mana_cost.as_ref().unwrap().mana_value(), 5);
        assert_eq!(elemental.power, Some(4));
        assert_eq!(elemental.toughness, Some(5));
    }
}
