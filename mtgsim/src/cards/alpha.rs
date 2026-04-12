//! Phase 2 spell card definitions.
//!
//! Note on naming: this module is called "alpha" as a catch-all for early card
//! implementations, but not all cards here are from the Alpha set:
//! - Lightning Bolt, Ancestral Recall, Counterspell — Alpha (1993)
//! - Burst of Energy — Urza's Destiny (1999)
//! - Volcanic Upheaval — Battle for Zendikar (2015)
//!
//! Once we have enough cards to justify it, we'll reorganize into per-set
//! modules (e.g. `cards::alpha`, `cards::urzas_destiny`, `cards::bfz`).

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::*;
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};

/// Lightning Bolt — {R}
/// Instant
/// Lightning Bolt deals 3 damage to any target.
pub fn lightning_bolt() -> Arc<CardData> {
    CardDataBuilder::new("Lightning Bolt")
        .card_type(CardType::Instant)
        .color(Color::Red)
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::DealDamage(AmountExpr::Fixed(3)),
                TargetSpec::Any(TargetCount::Exactly(1)),
            ),
        })
        .build()
}

/// Ancestral Recall — {U}
/// Instant
/// Target player draws 3 cards.
pub fn ancestral_recall() -> Arc<CardData> {
    CardDataBuilder::new("Ancestral Recall")
        .card_type(CardType::Instant)
        .color(Color::Blue)
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::DrawCards(AmountExpr::Fixed(3)),
                TargetSpec::Player(TargetCount::Exactly(1)),
            ),
        })
        .build()
}

/// Counterspell — {U}{U}
/// Instant
/// Counter target spell.
pub fn counterspell() -> Arc<CardData> {
    CardDataBuilder::new("Counterspell")
        .card_type(CardType::Instant)
        .color(Color::Blue)
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 0))
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::CounterSpell,
                TargetSpec::Spell(TargetCount::Exactly(1)),
            ),
        })
        .build()
}

/// Burst of Energy — {W}
/// Instant
/// Untap target permanent.
pub fn burst_of_energy() -> Arc<CardData> {
    CardDataBuilder::new("Burst of Energy")
        .card_type(CardType::Instant)
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::Untap,
                TargetSpec::Permanent(PermanentFilter::All, TargetCount::Exactly(1)),
            ),
        })
        .build()
}

/// Volcanic Upheaval — {3}{R}
/// Instant
/// Destroy target land.
pub fn volcanic_upheaval() -> Arc<CardData> {
    CardDataBuilder::new("Volcanic Upheaval")
        .card_type(CardType::Instant)
        .color(Color::Red)
        .mana_cost(ManaCost::build(&[ManaType::Red], 3))
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::Destroy,
                TargetSpec::Permanent(
                    PermanentFilter::ByType(CardType::Land),
                    TargetCount::Exactly(1),
                ),
            ),
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::AbilityType;

    #[test]
    fn test_lightning_bolt() {
        let bolt = lightning_bolt();
        assert_eq!(bolt.name, "Lightning Bolt");
        assert!(bolt.types.contains(&CardType::Instant));
        assert_eq!(bolt.mana_cost.as_ref().unwrap().mana_value(), 1);
        assert_eq!(bolt.abilities.len(), 1);
        assert_eq!(bolt.abilities[0].ability_type, AbilityType::Spell);
    }

    #[test]
    fn test_ancestral_recall() {
        let recall = ancestral_recall();
        assert_eq!(recall.name, "Ancestral Recall");
        assert!(recall.types.contains(&CardType::Instant));
        assert_eq!(recall.mana_cost.as_ref().unwrap().mana_value(), 1);
    }

    #[test]
    fn test_counterspell() {
        let cs = counterspell();
        assert_eq!(cs.name, "Counterspell");
        assert!(cs.types.contains(&CardType::Instant));
        assert_eq!(cs.mana_cost.as_ref().unwrap().mana_value(), 2);
    }

    #[test]
    fn test_burst_of_energy() {
        let boe = burst_of_energy();
        assert_eq!(boe.name, "Burst of Energy");
        assert!(boe.types.contains(&CardType::Instant));
        assert_eq!(boe.mana_cost.as_ref().unwrap().mana_value(), 1);
    }

    #[test]
    fn test_volcanic_upheaval() {
        let vu = volcanic_upheaval();
        assert_eq!(vu.name, "Volcanic Upheaval");
        assert!(vu.types.contains(&CardType::Instant));
        assert_eq!(vu.mana_cost.as_ref().unwrap().mana_value(), 4);
    }
}
