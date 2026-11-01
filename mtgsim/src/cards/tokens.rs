//! Named token definitions: a token printed as a card of its own, which the
//! registry deals like one (Everywhere, among Duskmourn's tokens), and the
//! creature tokens printed cards' effects create, each named for what it
//! makes.

use std::sync::Arc;

use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, LandType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{EntersAttacking, TokenDef};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::ManaType;

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

/// Everywhere
/// Token Land
/// This land is a Plains, Island, Swamp, Mountain, and Forest. ({T}: Add {W},
/// {U}, {B}, {R}, or {G}.)
///
/// (Scryfall, 2026-10-01.) **A token in the real game, not a card** — the one
/// Overlord of the Hauntwoods creates. It is in the registry because it is the
/// only source of every color, which is what let `fuzz_games --require` stop
/// seeding a deck's colors from the required card's; real "add one mana of any
/// color" is not expressible yet (`backlog.md` §2.19).
///
/// **Its land types are printed subtypes here, where Oracle now prints them as
/// text.** On 2026-09-03 the type line carried the five types; Oracle since
/// moved them into the text box, a characteristic-defining ability (CR 604.3)
/// that applies first in layer 4 (CR 613.3), so the base subtypes here read the
/// same under every effect in the registry. The five mana abilities are
/// CR 305.6's intrinsics, written out as the dual lands' are, since nothing
/// derives an ability from a printed subtype.
///
/// **It does not enter tapped.** "Create a *tapped* Everywhere token" is the
/// Overlord's instruction, not the token's text, so there is no `EnterWith` —
/// which is also what makes it a no-downside source where a tapland pushes
/// every deck's curve back a turn.
pub fn everywhere() -> Arc<CardData> {
    CardDataBuilder::new("Everywhere")
        .card_type(CardType::Land)
        .subtype(Subtype::Land(LandType::Plains))
        .subtype(Subtype::Land(LandType::Island))
        .subtype(Subtype::Land(LandType::Swamp))
        .subtype(Subtype::Land(LandType::Mountain))
        .subtype(Subtype::Land(LandType::Forest))
        .rules_text("This land is a Plains, Island, Swamp, Mountain, and Forest.")
        .mana_ability_single(ManaType::White)
        .mana_ability_single(ManaType::Blue)
        .mana_ability_single(ManaType::Black)
        .mana_ability_single(ManaType::Red)
        .mana_ability_single(ManaType::Green)
        .build()
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
