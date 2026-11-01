//! The tokens printed cards' effects create, each named for what it makes:
//! a token's characteristics are its maker's effect (CR 111.3), and one
//! token can have makers in several sets. A token dealt as a card is a
//! printing, filed by its set (Everywhere).


use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{EntersAttacking, TokenDef};
use crate::types::keywords::KeywordFlag;

/// The 4/4 white Angel with flying and vigilance Divine Visitation makes —
/// "Angel Token" by CR 111.4.
pub fn angel_token() -> TokenDef {
    TokenDef {
        name: None,
        colors: vec![Color::White],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Angel)],
        supertypes: Vec::new(),
        power: Some(4),
        toughness: Some(4),
        keyword_flags: vec![KeywordFlag::Flying, KeywordFlag::Vigilance],
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
        enters_attacking: None,
    }
}

/// A 1/1 red Human creature token, created tapped and attacking.
pub fn attacking_human_token() -> TokenDef {
    TokenDef {
        name: None,
        colors: vec![Color::Red],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Human)],
        supertypes: Vec::new(),
        power: Some(1),
        toughness: Some(1),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: true,
        enters_attacking: Some(EntersAttacking::Chosen),
    }
}

/// "a 1/1 green Elf Warrior creature token" (CR 111.4).
pub fn elf_warrior_token() -> TokenDef {
    TokenDef {
        name: None,
        colors: vec![Color::Green],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Elf), Subtype::Creature(CreatureType::Warrior)],
        supertypes: Vec::new(),
        power: Some(1),
        toughness: Some(1),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
        enters_attacking: None,
    }
}

/// A token that carries only what the effect said — CR 111.3's "a token
/// doesn't have any characteristics not defined by the spell or ability that
/// created it" — with CR 111.4's default name.
fn vanilla_token(
    color: Color,
    creature_type: CreatureType,
    power: i32,
    toughness: i32,
) -> TokenDef {
    TokenDef {
        name: None,
        colors: vec![color],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(creature_type)],
        supertypes: Vec::new(),
        power: Some(power),
        toughness: Some(toughness),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
        enters_attacking: None,
    }
}

/// "a 1/1 red Goblin creature token" — named "Goblin Token" (CR 111.4).
pub fn goblin_token() -> TokenDef {
    vanilla_token(Color::Red, CreatureType::Goblin, 1, 1)
}

/// "a 1/1 green Saproling creature token" — named "Saproling Token" (CR 111.4).
pub fn saproling_token() -> TokenDef {
    TokenDef {
        name: None,
        colors: vec![Color::Green],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Saproling)],
        supertypes: Vec::new(),
        power: Some(1),
        toughness: Some(1),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
        enters_attacking: None,
    }
}

/// "a 1/1 white Soldier creature token" — named "Soldier Token" (CR 111.4).
pub fn soldier_token() -> TokenDef {
    vanilla_token(Color::White, CreatureType::Soldier, 1, 1)
}

/// The 2/2 black Zombie Kalitas makes — named "Zombie Token" by CR 111.4,
/// since the card gives it no name.
pub fn zombie_token() -> TokenDef {
    TokenDef {
        name: None,
        colors: vec![Color::Black],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Zombie)],
        supertypes: Vec::new(),
        power: Some(2),
        toughness: Some(2),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
        enters_attacking: None,
    }
}
