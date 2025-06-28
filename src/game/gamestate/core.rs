use std::collections::HashMap;

// src/game/gamestate/core.rs
use crate::game::player::Player;
use crate::game::turn_structure::phase::{self, next_phase_type};
use crate::game::turn_structure::{phase::Phase, step::Step};
use crate::utils::constants::abilities::AbilityType;
use crate::utils::constants::combat::{AttackDeclaration, BlockDeclaration, CombatDamageAssignment};
use crate::utils::constants::effect_context::EffectContext;
use crate::utils::constants::events::{EventHandler, GameEvent};
use crate::utils::constants::game_objects::{AttackingState, BattlefieldState, BlockingState, CommandState, ExileState, GameObj, StackObjectType, StackState};
use crate::utils::constants::id_types::{ObjectId, PlayerId};
use crate::utils::constants::turns::PhaseType;

#[derive(Debug, Clone)]
pub struct Game {
    pub players: Vec<Player>,
    pub active_player_id: usize, // the active player is the one whose turn it is (by definition), so this doubles as a turn player index
    pub priority_player_id: usize,
    pub turn_number: u32,
    pub phase: Phase,
    // global zones (Player zones hand, library, graveyard are within Player struct)
    pub stack: Vec<GameObj<StackState>>, // stack of objects (spells, abilities, etc.)
    pub battlefield: HashMap<ObjectId, GameObj<BattlefieldState>>, // battlefield objects (creatures, enchantments, tokens, etc.)
    pub exile: Vec<GameObj<ExileState>>,
    pub command_zone: Vec<GameObj<CommandState>>,

    // Combat tracking
    pub attacks_declared: bool, // whether attacks have been declared this combat
    pub blockers_declared: bool, // whether blockers have been declared this combat

    // API-based combat declarations (optional, used instead of default CLI UI if present)
    pub pending_attack_declarations: Option<Vec<AttackDeclaration>>,
    pub pending_block_declarations: Option<Vec<BlockDeclaration>>,
    pub pending_damage_assignments: Option<Vec<CombatDamageAssignment>>,

    // Context tracking for effects
    pub effect_context: EffectContext,
}

impl Game {
    // Create a new game
    pub fn new() -> Self {
        Game {
            players: Vec::new(),
            active_player_id: 0,
            priority_player_id: 0,
            turn_number: 1,
            phase: Phase::new(PhaseType::Beginning),
            stack: Vec::new(),
            battlefield: HashMap::new(),
            exile: Vec::new(),
            command_zone: Vec::new(),
            attacks_declared: false,
            blockers_declared: false,
            pending_attack_declarations: None,
            pending_block_declarations: None,
            pending_damage_assignments: None,
            effect_context: EffectContext::new(),
        }
    }

    /// Check state-based actions whenever a player would get priority
    /// This should be called after any game action that could trigger SBAs
    pub fn check_state_based_actions_if_needed(&mut self) -> Result<(), String> {
        self.handle_event(&GameEvent::CheckStateBasedActions)
    }
    
    /// Give priority to a player, checking SBAs first
    pub fn give_priority(&mut self, player_id: PlayerId) -> Result<(), String> {
        // First check state-based actions
        self.check_state_based_actions_if_needed()?;
        
        // Then set priority
        self.priority_player_id = player_id;
        Ok(())
    }

    // Advance the gamestate to the next phase or step
    pub fn advance_turn(&mut self) -> Result<(), String> {
        // If we're in a phase with steps, attempt to advance to the next step
        if self.phase.has_steps() {
            if let Some(current_step) = &self.phase.current_step {
                // store current step type for StepEnded game event handler
                let old_step_type = current_step.step_type;

                if self.phase.next_step() {
                    // If next_step() returned true, emit the step ended event
                    self.handle_event(&GameEvent::StepEnded { step_type: old_step_type })?;
                    return self.process_current_phase();
                }
            }
        }

        // We do a similar thing with the phase end as we did with the step end
        // Store the current phase before advancing
        let old_phase_type = self.phase.phase_type;
        // If this phase doesn't have steps or we couldn't reach a next step (because we were in the last step of the previous phase)
        // we move to the next phase
        let next_phase_type = next_phase_type(&self.phase.phase_type);

        // Emit the phase ended event
        self.handle_event(&GameEvent::PhaseEnded { phase_type: old_phase_type })?;

        // If we're moving from Ending phase to Beginning phase, we are starting a new turn
        if self.phase.phase_type == PhaseType::Ending && next_phase_type == PhaseType::Beginning {
            self.turn_number += 1;
            self.active_player_id = (self.active_player_id + 1) % self.players.len();
        }

        self.phase = Phase::new(next_phase_type);
        
        // we have successfully updated to the new phase/step, now we process it.
        self.process_current_phase()
    }

    // Handle passing priority
    pub fn pass_priority(&mut self) -> Result<bool, String> {
        let player_count = self.players.len();
        let next_player_id = (self.priority_player_id + 1) % player_count;

        // If priority would pass to the active player with an empty stack, advance to the next step/phase
        if next_player_id == self.active_player_id && self.stack.is_empty() {
            self.advance_turn()?;
            return Ok(true);
        }

        // If both players have passed priority and stack is not empty, resolve the top of stack
        if next_player_id == self.active_player_id && !self.stack.is_empty() {
            println!("Both players passed priority with non-empty stack. Resolving top spell/ability...");
            self.resolve_top_of_stack()?;
            // After resolution, active player gets priority
            self.priority_player_id = self.active_player_id;
            return Ok(true);
        }    

        // Otherwise, pass priority to the next player
        self.priority_player_id = next_player_id;
        println!("Priority passed to player {}", self.priority_player_id);
        Ok(false)
    }

    // Handle resolving the spell/ability on top of the stack
    pub fn resolve_top_of_stack(&mut self) -> Result<(), String> {
        // Ensure the stack is nonempty
        if self.stack.is_empty() {
            return Err("Cannot resolve top of stack: Stack is empty".to_string());
        }

        // Pop the top spell/ability from the stack
        let top_object = self.stack.pop().unwrap();
        // Need to clone the value here so we can pass the spell/ability's controller to the resolution function
        let controller_id = top_object.state.controller;
        let owner_id = top_object.owner;

        // Process the object based on its stack_object type
        match top_object.state.stack_object_type {
            StackObjectType::Spell => {
                // Evaluate based on permanent vs nonpermanent (as this determines what zone the object will go to next)
                if let Some(card_types) = &top_object.characteristics.card_type {
                    let is_permanent = card_types.iter().any(|t| t.is_permanent());
                    if is_permanent {
                        // Resolve as permanent
                        match top_object.resolve_as_permanent(controller_id) {
                            Ok(permanent) => {
                                println!("Resolving spell as permanent: {:?}", permanent);
                                self.battlefield.insert(permanent.id, permanent);
                            },
                            Err(e) => return Err(format!("Error resolving spell as permanent: {}", e)),
                        }
                    } else {
                        // Resolve as nonpermanent (goes to graveyard)
                        println!("Resolving spell as nonpermanent: {:?}", top_object);
                        // Process spell effects of this spell
                        if let Some(abilities) = &top_object.characteristics.abilities {
                            for ability in abilities {
                                if ability.ability_type == AbilityType::Spell {
                                    // process this spell ability's effect(s)
                                    self.process_effect(&ability.effect_details, &top_object.state.targets, controller_id, Some(top_object.id))?;
                                }
                            }
                        }
                        // Move nonpermanent to graveyard
                        match top_object.resolve_as_nonpermanent() {
                            Ok(graveyard_obj) => {
                                // spell resolves to its owner's graveyard
                                let owner = self.get_player_mut(owner_id)?;
                                owner.graveyard.push(graveyard_obj);
                            },
                            Err(e) => return Err(format!("Error resolving spell as nonpermanent: {}", e)),
                        }
                    }
                } else {
                    return Err("Spell without card types cannot be resolved".to_string())
                }
            },
            _ => todo!()
        }
        // check state based actions after resolving the top of stack
        self.check_state_based_actions_if_needed()?;
        Ok(())
        
    }
}




impl EventHandler for Game {
    fn handle_event(&mut self, event: &GameEvent) -> Result<(), String> {
        match event {
            GameEvent::ManaAbilityActivated { source_id, player_id } => {
                self.handle_mana_ability_activated(*source_id, *player_id)
            },
            GameEvent::ManaAdded { source_id, player_id, mana_types } => {
                self.handle_mana_added(*source_id, *player_id, mana_types)
            },
            GameEvent::PhaseEnded { phase_type } => {
                self.handle_phase_ended(*phase_type)
            },
            GameEvent::StepEnded { step_type } => {
                self.handle_step_ended(*step_type)
            },
            GameEvent::DamageAboutToBeDealt { source_id, target_ref, amount } => {
                self.handle_damage_about_to_be_dealt(*source_id, target_ref, *amount)
            },
            GameEvent::DamageDealt { source_id, target_ref, amount } => {
                self.handle_damage_dealt(*source_id, target_ref, *amount)
            },
            GameEvent::CheckStateBasedActions => {
                self.handle_check_state_based_actions()
            },
            GameEvent::CreatureZeroToughness { creature_id } => {
                self.handle_creature_zero_toughness(*creature_id)
            },
            GameEvent::PermanentDestroyed { permanent_id, reason } => {
                self.handle_permanent_destroyed(*permanent_id, reason.clone())
            },
            _ => Err(format!("Unhandled game event: {:?}", event)),
        }
    }
}


// UNIT TESTS
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cards::basic_lands::{create_basic_land, BasicLand}, utils::constants::turns::StepType};
    
    #[test]
    fn test_game_creation() {
        let game = Game::new();
        
        assert_eq!(game.active_player_id, 0);
        assert_eq!(game.priority_player_id, 0);
        assert_eq!(game.turn_number, 1);
        assert_eq!(game.phase.phase_type, PhaseType::Beginning);
        assert!(game.players.is_empty());
        assert!(game.stack.is_empty());
        assert!(game.battlefield.is_empty());
    }
    
    #[test]
    fn test_add_players() {
        let mut game = Game::new();
        
        let player1 = Player::new(0, 20, 7, 1);
        let player2 = Player::new(1, 20, 7, 1);
        
        game.players.push(player1);
        game.players.push(player2);
        
        assert_eq!(game.players.len(), 2);
        assert_eq!(game.players[0].id, 0);
        assert_eq!(game.players[1].id, 1);
    }
    
    #[test]
    fn test_get_player_ref() {
        let mut game = Game::new();
        game.players.push(Player::new(0, 20, 7, 1));
        game.players.push(Player::new(1, 20, 7, 1));
        
        let player = game.get_player_ref(0);
        assert!(player.is_ok());
        assert_eq!(player.unwrap().id, 0);
        
        let invalid_player = game.get_player_ref(2);
        assert!(invalid_player.is_err());
    }
    
    #[test]
    fn test_get_player_mut() {
        let mut game = Game::new();
        game.players.push(Player::new(0, 20, 7, 1));
        
        {
            let player = game.get_player_mut(0).unwrap();
            player.life_total = 15;
        }
        
        assert_eq!(game.players[0].life_total, 15);
    }
    
    #[test]
    fn test_advance_to_next_phase() {
        let mut game = Game::new();
        game.players.push(Player::new(0, 20, 7, 1));
        game.players.push(Player::new(1, 20, 7, 1));

        // Both players need a card in library to prevent deckout in draw phase
        for player in &mut game.players {
            let mut library = Vec::new();
            library.push(create_basic_land(BasicLand::Forest, player.id));
            player.set_library(library);
        }
        
        assert_eq!(game.phase.phase_type, PhaseType::Beginning);
        // 3 steps in Beginning phase: Untap, Upkeep, Draw
        for _ in 0..3 {
            game.advance_turn().unwrap();
            // print current phase and step for debugging
            println!("Phase: {:?}, Step: {:?}", game.phase.phase_type, game.phase.current_step);
        }
        assert_eq!(game.phase.phase_type, PhaseType::Precombat);
        // No steps in Precombat phase, so it should advance to Combat
        game.advance_turn().unwrap();
        assert_eq!(game.phase.phase_type, PhaseType::Combat);
        // 6 steps in Combat phase: BeginCombat, DeclareAttackers, DeclareBlockers, FirstStrikeDamage, CombatDamage, EndCombat
        for _ in 0..6 {
            game.advance_turn().unwrap();
            // print current phase and step for debugging
            println!("Phase: {:?}, Step: {:?}", game.phase.phase_type, game.phase.current_step);
        }

        assert_eq!(game.phase.phase_type, PhaseType::Postcombat);
        // No steps in Postcombat phase, so it should advance to Ending
        game.advance_turn().unwrap();
        assert_eq!(game.phase.phase_type, PhaseType::Ending);
    }
    
    #[test]
    fn test_turn_cycle() {
        let mut game = Game::new();
        game.players.push(Player::new(0, 20, 7, 1));
        game.players.push(Player::new(1, 20, 7, 1));

        // Both players need a card in library to prevent deckout in draw phase
        for player in &mut game.players {
            let mut library = Vec::new();
            library.push(create_basic_land(BasicLand::Forest, player.id));
            player.set_library(library);
        }
        
        // Complete a full turn cycle
        for _ in 0..13 { // 5 phases in a turn, 13 total steps (3 in beginning, 6 in combat, 2 main phases, and 2 ending)
            game.advance_turn().unwrap();
            // print current phase and step for debugging
            println!("Phase: {:?}, Step: {:?}", game.phase.phase_type, game.phase.current_step);
        }
        
        // Should now be player 2's turn
        assert_eq!(game.turn_number, 2);
        assert_eq!(game.active_player_id, 1);
        assert_eq!(game.phase.phase_type, PhaseType::Beginning);

        // Do it again and ensure it cycles correctly
        for _ in 0..13 {
            game.advance_turn().unwrap();
        }
        
        // Should now be player 1's turn again
        assert_eq!(game.turn_number, 3);
        assert_eq!(game.active_player_id, 0);
        assert_eq!(game.phase.phase_type, PhaseType::Beginning);
    }
}