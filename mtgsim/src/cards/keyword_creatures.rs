//! Phase 4 keyword creature card definitions.
//!
//! French-vanilla (keywords only, no other abilities) and atomic (single keyword)
//! creatures used for testing keyword implementations.
//!
//! Card origins:
//! - Serra Angel — Alpha (1993), {3}{W}{W} 4/4 Flying, Vigilance
//! - Thornweald Archer — Future Sight (2007), {1}{G} 2/1 Reach, Deathtouch
//! - Raging Cougar — Portal (1997), {2}{R} 2/2 Haste
//! - Wall of Stone — Alpha (1993), {1}{R}{R} 0/8 Defender
//! - Elvish Archers — Alpha (1993), {1}{G} 2/1 First Strike
//! - Ridgetop Raptor — Legions (2003), {3}{R} 2/1 Double Strike
//! - War Mammoth — Alpha (1993), {3}{G} 3/3 Trample
//! - Knight of Meadowgrain — Lorwyn (2007), {W}{W} 2/2 First Strike, Lifelink
//! - Rhox War Monk — Shards of Alara (2008), {G}{W}{U} 3/4 Lifelink
//! - Giant Spider — Alpha (1993), {3}{G} 2/4 Reach
//! - Vampire Nighthawk — Zendikar (2009), {1}{B}{B} 2/3 Flying, Lifelink, Deathtouch

use std::sync::Arc;

use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::keywords::KeywordAbility;
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};

/// Serra Angel — {3}{W}{W}
/// Creature — Angel
/// 4/4 Flying, Vigilance
pub fn serra_angel() -> Arc<CardData> {
    CardDataBuilder::new("Serra Angel")
        .card_type(CardType::Creature)
        .color(Color::White)
        .mana_cost(ManaCost::single(ManaType::White, 2, 3))
        .power_toughness(4, 4)
        .keyword(KeywordAbility::Flying)
        .keyword(KeywordAbility::Vigilance)
        .build()
}

/// Thornweald Archer — {1}{G}
/// Creature — Elf Archer
/// 2/1 Reach, Deathtouch
pub fn thornweald_archer() -> Arc<CardData> {
    CardDataBuilder::new("Thornweald Archer")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::single(ManaType::Green, 1, 1))
        .power_toughness(2, 1)
        .keyword(KeywordAbility::Reach)
        .keyword(KeywordAbility::Deathtouch)
        .build()
}

/// Raging Cougar — {2}{R}
/// Creature — Cat
/// 2/2 Haste
pub fn raging_cougar() -> Arc<CardData> {
    CardDataBuilder::new("Raging Cougar")
        .card_type(CardType::Creature)
        .color(Color::Red)
        .mana_cost(ManaCost::single(ManaType::Red, 1, 2))
        .power_toughness(2, 2)
        .keyword(KeywordAbility::Haste)
        .build()
}

/// Wall of Stone — {1}{R}{R}
/// Creature — Wall
/// 0/8 Defender
pub fn wall_of_stone() -> Arc<CardData> {
    CardDataBuilder::new("Wall of Stone")
        .card_type(CardType::Creature)
        .color(Color::Red)
        .mana_cost(ManaCost::single(ManaType::Red, 2, 1))
        .power_toughness(0, 8)
        .keyword(KeywordAbility::Defender)
        .build()
}

/// Elvish Archers — {1}{G}
/// Creature — Elf Archer
/// 2/1 First Strike
pub fn elvish_archers() -> Arc<CardData> {
    CardDataBuilder::new("Elvish Archers")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::single(ManaType::Green, 1, 1))
        .power_toughness(2, 1)
        .keyword(KeywordAbility::FirstStrike)
        .build()
}

/// Ridgetop Raptor — {3}{R}
/// Creature — Dinosaur
/// 2/1 Double Strike
pub fn ridgetop_raptor() -> Arc<CardData> {
    CardDataBuilder::new("Ridgetop Raptor")
        .card_type(CardType::Creature)
        .color(Color::Red)
        .mana_cost(ManaCost::single(ManaType::Red, 1, 3))
        .power_toughness(2, 1)
        .keyword(KeywordAbility::DoubleStrike)
        .build()
}

/// War Mammoth — {3}{G}
/// Creature — Elephant
/// 3/3 Trample
pub fn war_mammoth() -> Arc<CardData> {
    CardDataBuilder::new("War Mammoth")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::single(ManaType::Green, 1, 3))
        .power_toughness(3, 3)
        .keyword(KeywordAbility::Trample)
        .build()
}

/// Knight of Meadowgrain — {W}{W}
/// Creature — Kithkin Knight
/// 2/2 First Strike, Lifelink
pub fn knight_of_meadowgrain() -> Arc<CardData> {
    CardDataBuilder::new("Knight of Meadowgrain")
        .card_type(CardType::Creature)
        .color(Color::White)
        .mana_cost(ManaCost::single(ManaType::White, 2, 0))
        .power_toughness(2, 2)
        .keyword(KeywordAbility::FirstStrike)
        .keyword(KeywordAbility::Lifelink)
        .build()
}

/// Rhox War Monk — {G}{W}{U}
/// Creature — Rhino Monk
/// 3/4 Lifelink
pub fn rhox_war_monk() -> Arc<CardData> {
    CardDataBuilder::new("Rhox War Monk")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .color(Color::White)
        .color(Color::Blue)
        .mana_cost(ManaCost { symbols: vec![
            ManaSymbol::Colored(ManaType::Green),
            ManaSymbol::Colored(ManaType::White),
            ManaSymbol::Colored(ManaType::Blue),
        ] })
        .power_toughness(3, 4)
        .keyword(KeywordAbility::Lifelink)
        .build()
}

/// Giant Spider — {3}{G}
/// Creature — Spider
/// 2/4 Reach
pub fn giant_spider() -> Arc<CardData> {
    CardDataBuilder::new("Giant Spider")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::single(ManaType::Green, 1, 3))
        .power_toughness(2, 4)
        .keyword(KeywordAbility::Reach)
        .build()
}

/// Vampire Nighthawk — {1}{B}{B}
/// Creature — Vampire Shaman
/// 2/3 Flying, Lifelink, Deathtouch
pub fn vampire_nighthawk() -> Arc<CardData> {
    CardDataBuilder::new("Vampire Nighthawk")
        .card_type(CardType::Creature)
        .color(Color::Black)
        .mana_cost(ManaCost::single(ManaType::Black, 2, 1))
        .power_toughness(2, 3)
        .keyword(KeywordAbility::Flying)
        .keyword(KeywordAbility::Lifelink)
        .keyword(KeywordAbility::Deathtouch)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::keywords::KeywordAbility;

    #[test]
    fn test_serra_angel() {
        let card = serra_angel();
        assert_eq!(card.name, "Serra Angel");
        assert_eq!(card.power, Some(4));
        assert_eq!(card.toughness, Some(4));
        assert!(card.keywords.contains(&KeywordAbility::Flying));
        assert!(card.keywords.contains(&KeywordAbility::Vigilance));
        assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 5);
    }

    #[test]
    fn test_thornweald_archer() {
        let card = thornweald_archer();
        assert_eq!(card.name, "Thornweald Archer");
        assert_eq!(card.power, Some(2));
        assert_eq!(card.toughness, Some(1));
        assert!(card.keywords.contains(&KeywordAbility::Reach));
        assert!(card.keywords.contains(&KeywordAbility::Deathtouch));
        assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 2);
    }

    #[test]
    fn test_raging_cougar() {
        let card = raging_cougar();
        assert_eq!(card.name, "Raging Cougar");
        assert_eq!(card.power, Some(2));
        assert_eq!(card.toughness, Some(2));
        assert!(card.keywords.contains(&KeywordAbility::Haste));
        assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 3);
    }

    #[test]
    fn test_wall_of_stone() {
        let card = wall_of_stone();
        assert_eq!(card.name, "Wall of Stone");
        assert_eq!(card.power, Some(0));
        assert_eq!(card.toughness, Some(8));
        assert!(card.keywords.contains(&KeywordAbility::Defender));
        assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 3);
    }

    #[test]
    fn test_elvish_archers() {
        let card = elvish_archers();
        assert_eq!(card.name, "Elvish Archers");
        assert_eq!(card.power, Some(2));
        assert_eq!(card.toughness, Some(1));
        assert!(card.keywords.contains(&KeywordAbility::FirstStrike));
        assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 2);
    }

    #[test]
    fn test_ridgetop_raptor() {
        let card = ridgetop_raptor();
        assert_eq!(card.name, "Ridgetop Raptor");
        assert_eq!(card.power, Some(2));
        assert_eq!(card.toughness, Some(1));
        assert!(card.keywords.contains(&KeywordAbility::DoubleStrike));
        assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 4);
    }

    #[test]
    fn test_war_mammoth() {
        let card = war_mammoth();
        assert_eq!(card.name, "War Mammoth");
        assert_eq!(card.power, Some(3));
        assert_eq!(card.toughness, Some(3));
        assert!(card.keywords.contains(&KeywordAbility::Trample));
        assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 4);
    }

    #[test]
    fn test_knight_of_meadowgrain() {
        let card = knight_of_meadowgrain();
        assert_eq!(card.name, "Knight of Meadowgrain");
        assert_eq!(card.power, Some(2));
        assert_eq!(card.toughness, Some(2));
        assert!(card.keywords.contains(&KeywordAbility::FirstStrike));
        assert!(card.keywords.contains(&KeywordAbility::Lifelink));
        assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 2);
    }

    #[test]
    fn test_rhox_war_monk() {
        let card = rhox_war_monk();
        assert_eq!(card.name, "Rhox War Monk");
        assert_eq!(card.power, Some(3));
        assert_eq!(card.toughness, Some(4));
        assert!(card.keywords.contains(&KeywordAbility::Lifelink));
        assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 3);
    }

    #[test]
    fn test_giant_spider() {
        let card = giant_spider();
        assert_eq!(card.name, "Giant Spider");
        assert_eq!(card.power, Some(2));
        assert_eq!(card.toughness, Some(4));
        assert!(card.keywords.contains(&KeywordAbility::Reach));
        assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 4);
    }

    #[test]
    fn test_vampire_nighthawk() {
        let card = vampire_nighthawk();
        assert_eq!(card.name, "Vampire Nighthawk");
        assert_eq!(card.power, Some(2));
        assert_eq!(card.toughness, Some(3));
        assert!(card.keywords.contains(&KeywordAbility::Flying));
        assert!(card.keywords.contains(&KeywordAbility::Lifelink));
        assert!(card.keywords.contains(&KeywordAbility::Deathtouch));
        assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 3);
    }
}
