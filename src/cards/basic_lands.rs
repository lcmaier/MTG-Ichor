// src/cards/basic_lands.rs

use std::collections::{HashSet, HashMap};

use uuid::Uuid;

use crate::utils::{
    constants::{
        abilities::{
            AbilityDefinition, AbilityType, EffectDetails
        }, card_types::{
            BasicLandType, 
            CardType, 
            LandType, 
            Subtype, 
            Supertype
        }, costs::Cost, game_objects::{
            Characteristics, GameObj, LibraryState}, id_types::PlayerId}, 
    mana::ManaType};

// We need this distinction from BasicLandType because Wastes don't have a basic land type because Richard Garfield hates me specifically
#[derive(PartialEq)]
pub enum BasicLand {
    Plains,
    Island,
    Swamp,
    Mountain,
    Forest,
    Wastes
}

// Creates a basic land card (LibraryState objects can only be cards) of the specified type
pub fn create_basic_land(basic_land_variant: BasicLand, owner: PlayerId) -> GameObj<LibraryState> {
    let id = Uuid::new_v4(); // generate unique id for card

    // define the card's types (including subtype and supertype)
    let mut card_types = HashSet::new();
    card_types.insert(CardType::Land);

    let mut supertype = HashSet::new();
    supertype.insert(Supertype::Basic);

    let mut subtype = HashSet::new();
    match basic_land_variant {
        BasicLand::Plains => subtype.insert(Subtype::Land(LandType::Basic(BasicLandType::Plains))),
        BasicLand::Island => subtype.insert(Subtype::Land(LandType::Basic(BasicLandType::Island))),
        BasicLand::Swamp => subtype.insert(Subtype::Land(LandType::Basic(BasicLandType::Swamp))),
        BasicLand::Mountain => subtype.insert(Subtype::Land(LandType::Basic(BasicLandType::Mountain))),
        BasicLand::Forest => subtype.insert(Subtype::Land(LandType::Basic(BasicLandType::Forest))),
        BasicLand::Wastes => false, // No subtype for Wastes
    };

    // match card name and rules text to land type
    let (card_name, rules_text) = match basic_land_variant {
        BasicLand::Plains => ("Plains".to_string(), "T: Add {W}".to_string()),
        BasicLand::Island => ("Island".to_string(), "T: Add {U}".to_string()),
        BasicLand::Swamp => ("Swamp".to_string(), "T: Add {B}".to_string()),
        BasicLand::Mountain => ("Mountain".to_string(), "T: Add {R}".to_string()),
        BasicLand::Forest => ("Forest".to_string(), "T: Add {G}".to_string()),
        BasicLand::Wastes => ("Wastes".to_string(), "T: Add {C}".to_string())
    };


    // Create mana ability definition based on land type
    let mana_type = match basic_land_variant {
        BasicLand::Plains => ManaType::White,
        BasicLand::Island => ManaType::Blue,
        BasicLand::Swamp => ManaType::Black,
        BasicLand::Mountain => ManaType::Red,
        BasicLand::Forest => ManaType::Green,
        BasicLand::Wastes => ManaType::Colorless
    };

    let mut mana_map = HashMap::new();
    mana_map.insert(mana_type, 1); // Basic lands add 1 mana of the appropriate type


    let mana_ability = AbilityDefinition {
        id: Uuid::new_v4(),
        ability_type: AbilityType::Mana,
        costs: vec![Cost::Tap],
        effect_details: EffectDetails::ProduceMana { mana_produced: mana_map },
    };

    // add the abilities to this object's characteristics
    let abilities = vec![mana_ability];

    // Build characteristics for LibraryState card object
    let characteristics = Characteristics {
        name: Some(card_name),
        mana_cost: None, // Lands don't have a mana cost
        color: Some(HashSet::new()), // Lands are colorless
        color_indicator: None, // Basic lands don't have color indicators
        card_type: Some(card_types),
        supertype: Some(supertype),
        subtype: if subtype != HashSet::new() { Some(subtype) } else { None },
        rules_text: Some(rules_text),
        abilities: Some(abilities),
        power: None,
        toughness: None,
        loyalty: None,
        defense: None,
        hand_modifier: None,
        life_modifier: None,
    };

    // Create the GameObj for the land
    GameObj { id, owner, characteristics, state: LibraryState {} }
    
}