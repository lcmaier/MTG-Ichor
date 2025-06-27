// src/utils/constants/abilities.rs

use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    game::{gamestate::Game, player::Player}, 
    utils::{
        constants::{
            costs::{Cost, ManaCost}, id_types::{ObjectId, PlayerId}
        },
    mana::ManaType, targeting::requirements::TargetingRequirement}};

use super::{card_types::CreatureType, game_objects::{BattlefieldState, GameObj}};

#[derive(Debug, Clone, PartialEq)]
pub enum AbilityType {
    Mana,
    Activated,
    Triggered,
    Static,
    Spell,
    // Loyalty,
}

impl Cost {
    // Check if the cost can be paid
    pub fn can_pay(&self, game: &Game, player_id: PlayerId, permanent_id: Option<ObjectId>) -> Result<bool, String> {
        match self {
            Cost::Tap => {
                // we need to ensure that the associated permanent is untapped to pay a Tap cost
                // First, get the permanent
                if permanent_id == None {
                    return Err("A tap payment needs a permanent_id to tap, received none".to_string())
                }
                let obj_id = permanent_id.unwrap();
                // Next, find the permanent
                let permanent = game.battlefield.get(&obj_id)
                    .ok_or_else(|| format!("Permanent with ID {} not found on the battlefield", obj_id))?;

                // return true only if the permanent is untapped
                Ok(!permanent.state.tapped)
            },
            Cost::Mana(mana_cost) => {
                // get the player and check their mana pool
                let player = match game.get_player_ref(player_id) {
                    Ok(p) => p,
                    Err(_) => return Ok(false),
                };
                println!("{:?}", mana_cost);
                println!("{:?}", player.mana_pool);

                // check specific mana type costs (Green, White, etc--everything except generic mana, which requires additional handling)
                if !player.mana_pool.has_mana(ManaType::White, mana_cost.white as u64) ||
                   !player.mana_pool.has_mana(ManaType::Blue, mana_cost.blue as u64) ||
                   !player.mana_pool.has_mana(ManaType::Black, mana_cost.black as u64) ||
                   !player.mana_pool.has_mana(ManaType::Red, mana_cost.red as u64) ||
                   !player.mana_pool.has_mana(ManaType::Green, mana_cost.green as u64) ||
                   !player.mana_pool.has_mana(ManaType::Colorless, mana_cost.colorless as u64) {
                    return Ok(false);
                }

                // create a clone of the player's mana pool to subtract specific costs (to see if generic can be paid afterwards)
                let mut casting_pool = player.mana_pool.clone();
                casting_pool.remove_mana(ManaType::White, mana_cost.white as u64)?;
                casting_pool.remove_mana(ManaType::Blue, mana_cost.blue as u64)?;
                casting_pool.remove_mana(ManaType::Black, mana_cost.black as u64)?;
                casting_pool.remove_mana(ManaType::Red, mana_cost.red as u64)?;
                casting_pool.remove_mana(ManaType::Green, mana_cost.green as u64)?;
                casting_pool.remove_mana(ManaType::Colorless, mana_cost.colorless as u64)?;

                // Now that we've checked all specific costs, get total remaining mana and check if we can pay generic cost
                if casting_pool.get_generic_mana() < mana_cost.generic as u64 {
                    return Ok(false);
                }
                // If we pass all the checks, then this player can pay this mana cost
                Ok(true)
            }
        }
    }

    // Pay the cost (IMPORTANT: this method assumes you've already verified that the cost can be paid (with can_pay), undefined behavior when calling on an unpayable cost)
    pub fn pay(&self, game: &mut Game, player_id: PlayerId, permanent_id: Option<ObjectId>) -> Result<(), String> {
        match self {
            Cost::Tap => {
                // This payment type requires a permanent id (to tap)
                if permanent_id == None {
                    return Err("A tap payment needs a permanent_id to tap, received none".to_string())
                }
                let obj_id = permanent_id.unwrap();
                // Next, locate the permanent on the battlefield
                let mut_permanent = game.battlefield.get_mut(&obj_id)
                    .ok_or_else(|| format!("Permanent with ID {} not found on the battlefield", obj_id))?;

                // pay the cost by tapping down the permanent
                mut_permanent.state.tapped = true;
                Ok(())
            },
            Cost::Mana(mana_cost) => {
                // we get a mutable reference to the player's mana pool and remove all specific mana costs
                let player = game.get_player_mut(player_id)?;

                player.mana_pool.remove_mana(ManaType::White, mana_cost.white as u64)?;
                player.mana_pool.remove_mana(ManaType::Blue, mana_cost.blue as u64)?;
                player.mana_pool.remove_mana(ManaType::Black, mana_cost.black as u64)?;
                player.mana_pool.remove_mana(ManaType::Red, mana_cost.red as u64)?;
                player.mana_pool.remove_mana(ManaType::Green, mana_cost.green as u64)?;
                player.mana_pool.remove_mana(ManaType::Colorless, mana_cost.colorless as u64)?;

                // to pay generic costs, for now we just remove mana as needed in CWUBRG order
                // TODO: Implement a more sophisticated "smart payment" system
                let mut generic_remaining = mana_cost.generic as u64;
                if generic_remaining > 0 {
                    let mana_left = player.mana_pool.get_available_mana();
                    // get how much colorless is in the mana pool and subtract it from the generic_remaining
                    let colorless_left = mana_left.get(&ManaType::Colorless).unwrap_or(&0);
                    if colorless_left >= &generic_remaining {
                        // we can satisfy the requirement with colorless mana
                        player.mana_pool.remove_mana(ManaType::Colorless, generic_remaining as u64)?;
                        // paid all specific and generic mana costs, so we've satisfied the ManaCost and can return successfully
                        return Ok(())
                    }
                    // subtract the amount of colorless left from the generic_remaining and deplete all colorless mana in the pool
                    generic_remaining -= colorless_left;
                    player.mana_pool.remove_mana(ManaType::Colorless, *colorless_left)?;


                    // Repeat for the colors in WUBRG order...
                    let white_left = mana_left.get(&ManaType::White).unwrap_or(&0);
                    if white_left >= &generic_remaining {
                        player.mana_pool.remove_mana(ManaType::White, generic_remaining as u64)?;
                        return Ok(())
                    }
                    generic_remaining -= white_left;
                    player.mana_pool.remove_mana(ManaType::White, *white_left)?;

                    let blue_left = mana_left.get(&ManaType::Blue).unwrap_or(&0);
                    if blue_left >= &generic_remaining {
                        player.mana_pool.remove_mana(ManaType::Blue, generic_remaining as u64)?;
                        return Ok(())
                    }
                    generic_remaining -= blue_left;
                    player.mana_pool.remove_mana(ManaType::Blue, *blue_left)?;
                    
                    let black_left = mana_left.get(&ManaType::Black).unwrap_or(&0);
                    if black_left >= &generic_remaining {
                        player.mana_pool.remove_mana(ManaType::Black, generic_remaining as u64)?;
                        return Ok(())
                    }
                    generic_remaining -= black_left;
                    player.mana_pool.remove_mana(ManaType::Black, *black_left)?;

                    let red_left = mana_left.get(&ManaType::Red).unwrap_or(&0);
                    if red_left >= &generic_remaining {
                        player.mana_pool.remove_mana(ManaType::Red, generic_remaining as u64)?;
                        return Ok(())
                    }
                    generic_remaining -= red_left;
                    player.mana_pool.remove_mana(ManaType::Red, *red_left)?;

                    let green_left = mana_left.get(&ManaType::Green).unwrap_or(&0);
                    if green_left >= &generic_remaining {
                        player.mana_pool.remove_mana(ManaType::Green, generic_remaining as u64)?;
                        return Ok(())
                    }
                    // paid as much as we could, if there's still generic mana then we must return an error
                    return Err("Not enough mana in mana pool to pay cost!".to_string())
                } else {
                    // no generic cost, so we've already paid everything we need to
                    Ok(())
                }
            }
        }
    }
}

// Ability definitions - These are NOT objects in the game
#[derive(Debug, Clone, PartialEq)]
pub struct AbilityDefinition {
    pub id: Uuid,
    pub ability_type: AbilityType,
    pub costs: Vec<Cost>,
    pub effect_details: EffectDetails,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    TODO
    // Add as needed
}

// Types of effects - will be expanded later
#[derive(Debug, Clone, PartialEq)]
pub enum EffectDetails  {
    // Recursive sequence type for multi-step effects (stuff like "Discard any number of cards. Then..." or "Draw a card, then you may discard a card")
    Sequence(Vec<EffectDetails>),

    // Recursive conditional type for conditional effects ("If you discarded a nonland card this way..."-style effects)
    Conditional {
        condition: Condition,
        if_true: Box<EffectDetails>,
        if_false: Option<Box<EffectDetails>>,
    },
    
    //// MANA ABILITY EFFECTS
    ProduceMana {
        mana_produced: HashMap<ManaType, u64>,
    },

    //// DAMAGE ABILITIES
    DealDamage {
        amount: u64,
        target_requirement: Option<TargetingRequirement>, // Not all damage requires a target, see "Solar Blaze"
    },
    // Add more effect types as needed
}

// Only activated/triggered abilities on the stack become ability objects
#[derive(Debug, Clone)]
pub struct AbilityOnStack {
    pub id: ObjectId,        // Only for stack objects
    pub source_id: ObjectId, // The object that has this ability 
    pub controller_id: PlayerId,
    pub effect_details: EffectDetails ,
}

