// src/utils/game_obj_behavior/zone_transitions.rs
use crate::utils::constants::game_objects::*;
use crate::utils::constants::id_types::PlayerId;
use crate::utils::constants::card_types::*;

// This file handles the logic for all default methods of moving game objects between zones (i.e. replacement effects are handled elsewhere)
// This is pure object-to-object conversion methods, think of this as the "bare metal layer" for moving GameObjs between zones
// NOTE: Currently incomplete, will be added to as new transitions are needed as the alpha expands

// Playing cards from the hand
impl GameObj<HandState> {
    // cards from the hand can go to every other zone (including command 
    // (i.e. an effect would discard your commander out of your hand, you have the option to put it into the command zone))
    pub fn to_battlefield(self, controller: PlayerId) -> GameObj<BattlefieldState> {
        let mut battlefield_state = BattlefieldState {
            tapped: false,
            flipped: false,
            face_down: false,
            phased_out: false,
            controller,
            // counters: HashMap::new(),
            // initialize all aspects to None, we populate these as needed below
            damageable: None,
            creature: None,
        };

        // Check card types and add relevant aspects
        if let Some(card_types) = &self.characteristics.card_type {
            // Add damageable aspect for creatures, planeswalkers, and battles
            if card_types.contains(&CardType::Creature) ||
               card_types.contains(&CardType::Planeswalker) ||
               card_types.contains(&CardType::Battle) {
                battlefield_state.damageable = Some(DamageableAspect { 
                    damage_marked: 0, 
                });
            }

            // Add creature aspect for creatures
            if card_types.contains(&CardType::Creature) {
                battlefield_state.creature = Some(CreatureAspect { 
                    summoning_sick: true, 
                    power_modifier: 0, 
                    toughness_modifier: 0, 
                    attacking: None, 
                    blocking: None 
                });
            }
        }
        // Create a GameObj with the populated battlefield_state
        GameObj {
            id: self.id,
            owner: self.owner,
            characteristics: self.characteristics,
            state: battlefield_state,
        }
    }
}





// Moving objects from the battlefield to other zones
impl GameObj<BattlefieldState> {
    // destroy transition (by default, moves the object to the graveyard)
    pub fn destroy(self) -> GameObj<GraveyardState> {
        GameObj { 
            id: self.id, 
            owner: self.owner, 
            characteristics: self.characteristics, 
            state: GraveyardState 
        }
    }
}






// Moving objects from the stack to other zones (usually the battlefield or graveyard as they resolve/are countered)
impl GameObj<StackState> {
    pub fn resolve_as_permanent(self, spell_controller: PlayerId) -> Result<GameObj<BattlefieldState>, String> {
        // ensure this object has a permanent type before moving it to the battlefield
        let is_permanent_type = if let Some(types) = &self.characteristics.card_type {
            types.iter().any(|t| t.is_permanent())
        } else {
            false
        };

        if !is_permanent_type {
            return Err("Can't resolve a non-permanent spell as a permanent".to_string());
        }

        // next we initialize the state for this object when it enter the battlefield
        let mut battlefield_state = BattlefieldState {
            tapped: false,
            flipped: false,
            face_down: false,
            phased_out: false,
            controller: spell_controller,
            // counters: HashMap::new()
            // initialize all aspects to None, we populate these as needed below
            damageable: None,
            creature: None,
        };

        // Check card types and add relevant aspects
        if let Some(card_types) = &self.characteristics.card_type {
            // Add damageable aspect for creatures, planeswalkers, and battles
            if card_types.contains(&CardType::Creature) ||
               card_types.contains(&CardType::Planeswalker) ||
               card_types.contains(&CardType::Battle) {
                battlefield_state.damageable = Some(DamageableAspect { 
                    damage_marked: 0, 
                });
            }

            // Add creature aspect for creatures
            if card_types.contains(&CardType::Creature) {
                battlefield_state.creature = Some(CreatureAspect { 
                    summoning_sick: true, 
                    power_modifier: 0, 
                    toughness_modifier: 0, 
                    attacking: None, 
                    blocking: None 
                });
            }
        }

        // Create a GameObj with the populated battlefield_state
        Ok(GameObj {
            id: self.id,
            owner: self.owner,
            characteristics: self.characteristics,
            state: battlefield_state,
        })


    }

    pub fn resolve_as_nonpermanent(self) -> Result<GameObj<GraveyardState>, String> {
        // ensure this is at least an instant or sorcery spell (only nonpermanent types, only sort of exception is Kindred on these, but those are still nonpermanent spells)
        let is_nonpermanent_type = if let Some(types) = &self.characteristics.card_type {
            types.iter().any(|t| !t.is_permanent())
        } else {
            false
        };

        if !is_nonpermanent_type {
            return Err("Can't resolve a spell with no nonpermanent types (Instant or Sorcery) as a nonpermanent spell".to_string());
        }

        // Otherwise we're good to move the spell to the graveyard as it resolves
        Ok(GameObj { 
            id: self.id, 
            owner: self.owner, 
            characteristics: self.characteristics, 
            state: GraveyardState 
        })
    }
}






// Library zone transitions
impl GameObj<LibraryState> {
    pub fn to_hand(self) -> GameObj<HandState> {
        GameObj { 
            id: self.id, 
            owner: self.owner, 
            characteristics: self.characteristics, 
            state: HandState {}, 
        }
    }
}





// Graveyard zone transitions
impl GameObj<GraveyardState> {
    pub fn to_hand(self) -> GameObj<HandState> {
        GameObj { 
            id: self.id, 
            owner: self.owner, 
            characteristics: self.characteristics, 
            state: HandState {} 
        }
    }
}