// src/cards/red/instant/lightning_bolt.rs

use std::collections::HashSet;

use uuid::Uuid;

use crate::utils::{
    constants::{
        abilities::{AbilityDefinition, AbilityType, EffectDetails}, card_types::CardType, colors::Color, costs::ManaCost, game_objects::{Characteristics, GameObj, HandState, LibraryState}, id_types::PlayerId, zones::Zone}, targeting::requirements::TargetingRequirement, traits::zonestate::ZoneState};

pub fn lightning_bolt_characteristics() -> Characteristics {
    let mut card_types = HashSet::new();
    card_types.insert(CardType::Instant);
    
    let mut color = HashSet::new();
    color.insert(Color::Red);
    
    // Define effect(s)
    let effect_details = EffectDetails::DealDamage { 
        amount: 3, 
        target_requirement: Some(TargetingRequirement::any_target(1)) 
    };
    
    Characteristics {
        name: Some("Lightning Bolt".to_string()),
        mana_cost: Some(ManaCost::red(1, 0)),
        color: Some(color),
        color_indicator: None,
        card_type: Some(card_types),
        supertype: None,
        subtype: None,
        rules_text: Some("~ deals 3 damage to any target.".to_string()),
        abilities: Some(vec![AbilityDefinition {
            id: Uuid::new_v4(),
            ability_type: AbilityType::Spell,
            costs: vec![], // No additional cost beyond mana cost
            effect_details,
        }]),
        power: None,
        toughness: None,
        loyalty: None,
        defense: None,
        hand_modifier: None,
        life_modifier: None,
    }
}
