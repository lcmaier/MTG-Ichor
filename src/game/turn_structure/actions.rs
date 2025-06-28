// src/game/turn_structure/actions.rs
use crate::game::gamestate::Game;
use crate::game::ui::combat::{AttackerUI, BlockerUI, DamageAssignmentUI};
use crate::utils::constants::combat::{AttackTarget, CombatDamageAssignment, DamageRecipient};
use crate::utils::constants::events::{EventHandler, GameEvent};
use crate::utils::constants::game_objects::{AttackingState, BlockingState};
use crate::utils::constants::id_types::ObjectId;
use crate::utils::constants::turns::{PhaseType, StepType};
use crate::utils::targeting::core::{TargetRef, TargetRefId};

// Game impls for moving through the turns
impl Game {
    // Process the current phase and step
    pub fn process_current_phase(&mut self) -> Result<(), String> {
        match self.phase.phase_type {
            PhaseType::Beginning => {
                if let Some(ref step) = self.phase.current_step {
                    match step.step_type {
                        StepType::Untap => self.process_untap_step(),
                        StepType::Upkeep => self.process_upkeep_step(),
                        StepType::Draw => self.process_draw_step(),
                        _ => Err(format!("Invalid step {:?} for Beginning phase", step.step_type)),
                    }
                } else {
                    Err("Beginning phase must have a step".to_string())
                }
            },
            PhaseType::Precombat => self.process_main_phase(),
            PhaseType::Combat => {
                if let Some(ref step) = self.phase.current_step {
                    match step.step_type {
                        StepType::BeginCombat => self.process_begin_combat_step(),
                        StepType::DeclareAttackers => self.process_declare_attackers_step(),
                        StepType::DeclareBlockers => self.process_declare_blockers_step(),
                        StepType::FirstStrikeDamage => self.process_first_strike_damage_step(),
                        StepType::CombatDamage => self.process_combat_damage_step(),
                        StepType::EndCombat => self.process_end_combat_step(),
                        _ => Err(format!("Invalid step {:?} for Combat phase", step.step_type)),
                    }
                } else {
                    Err("Combat phase must have a step".to_string())
                }
            },
            PhaseType::Postcombat => self.process_main_phase(),
            PhaseType::Ending => {
                if let Some(ref step) = self.phase.current_step {
                    match step.step_type {
                        StepType::End => self.process_end_step(),
                        StepType::Cleanup => self.process_cleanup_step(),
                        _ => Err(format!("Invalid step {:?} for Ending phase", step.step_type)),
                    }
                } else {
                    Err("Ending phase must have a step".to_string())
                }    
            }
        }
    }


    // STEP PROCESSING METHODS

    fn process_untap_step(&mut self) -> Result<(), String> {
        // 1. Switch phasing state of permanents with phasing active player controls (rule 502.1)
        // 2. Do day/night checks per rule 502.2
        // 3. Untap as many permanents as possible (this may involve the active player making choices about which permanents to untap) 
        // ABOVE NOT YET IMPLEMENTED
        // 4. Reset land play count
        let active_player = self.get_player_mut(self.active_player_id)?;
        active_player.reset_lands_played();

        // This isn't in rule 502 but rule 302.6 explains summoning sickness--so we update that state in all creatures currently under the active player's control
        for permanent in self.battlefield.values_mut() {
            if permanent.state.controller == self.active_player_id {
                if let Some(creature) = &mut permanent.state.creature {
                    if creature.summoning_sick {
                        creature.summoning_sick = false;
                    }
                }
            }
        }

        // No player gets priority during untap step
        println!("Untap step completed.");
        Ok(())
    }

    fn process_upkeep_step(&mut self) -> Result<(), String> {
        // Active player gets priority at the beginning of this step per rule 503.1
        self.priority_player_id = self.active_player_id;

        // NOTE: Need to implement functionality to check for upkeep-based triggers somewhere, if not here
        Ok(())
    }

    fn process_draw_step(&mut self) -> Result<(), String> {
        // Active player draws a card
        let active_player = self.get_player_mut(self.active_player_id)?;
        active_player.draw_card()?;

        // Then active player gets priority
        self.priority_player_id = self.active_player_id;
        Ok(())
    }

    fn process_main_phase(&mut self) -> Result<(), String> {
        // Implementation for main phase
        self.priority_player_id = self.active_player_id;
        Ok(())
    }

    // COMBAT STEP HANDLING
    fn process_begin_combat_step(&mut self) -> Result<(), String> {
        // This is where "At the beginning of combat" triggers fire, something like:
        // let begin_combat_event = GameEvent::BeginCombat {
        //     active_player_id: self.active_player_id
        // };
        // self.handle_event(&begin_combat_event)?;

        self.priority_player_id = self.active_player_id;
        Ok(())
    }

    fn process_declare_attackers_step(&mut self) -> Result<(), String> {
        println!("\n=== Declare Attackers Step ===");

        // Get attack decisions from the UI
        let decisions = AttackerUI::get_attack_decisions(self)?;
        // Apply the attack declarations
        for declaration in decisions {
            self.declare_attacker(declaration.attacker_id, declaration.target)?;
        }

        self.attacks_declared = true; // Mark that attackers have been declared
        self.priority_player_id = self.active_player_id; // Active player gets priority after declaring attackers
        Ok(())
    }

    fn process_declare_blockers_step(&mut self) -> Result<(), String> {
        println!("\n=== Declare Blockers Step ===");

        
    // Check if there are any attackers
    let has_attackers = self.battlefield.values().any(|obj| {
        obj.state.creature.as_ref()
            .and_then(|c| c.attacking.as_ref())
            .is_some()
    });
    
    if !has_attackers {
        println!("No attackers declared. Ending declare blockers step.");
        self.priority_player_id = self.active_player_id;
        return Ok(());
    }

    // Get block decisions from the UI
    let decisions = BlockerUI::get_block_decisions(self)?;
    
    // Apply the block declarations
    for declaration in decisions {
        self.declare_blocker(declaration.blocker_id, declaration.attacker_id)?;
    }

    self.priority_player_id = self.active_player_id;
    Ok(())
    }

    fn process_first_strike_damage_step(&mut self) -> Result<(), String> {
        // Will be implemented later
        self.priority_player_id = self.active_player_id;
        Ok(())
    }

    fn process_combat_damage_step(&mut self) -> Result<(), String> {
        println!("\n=== Combat Damage Step ===");

        // Process regular combat damage
        self.process_combat_damage(false)?;

        // Check state-based actions for lethal damage
        self.handle_event(&GameEvent::CheckStateBasedActions)?;
        self.priority_player_id = self.active_player_id;
        Ok(())
    }

    fn process_end_combat_step(&mut self) -> Result<(), String> {
        // clear attacking and blocking states
        self.attacks_declared = false;
        self.blockers_declared = false;
        for (_, obj) in &mut self.battlefield {
            if let Some(creature_aspect) = &mut obj.state.creature {
                // Reset attacking state
                creature_aspect.attacking = None;
                // Reset blocking state
                creature_aspect.blocking = None;
            }
        }
        
        self.priority_player_id = self.active_player_id;
        Ok(())
    }

    // END STEP HANDLING
    fn process_end_step(&mut self) -> Result<(), String> {
        self.priority_player_id = self.active_player_id;
        Ok(())
    }

    fn process_cleanup_step(&mut self) -> Result<(), String> {
        // 1. Discard to hand size
        let active_player = self.get_player_mut(self.active_player_id)?;
        // check if hand size > max
        while active_player.hand.len() > active_player.max_hand_size as usize {
            // discarding to hand size logic
        }

        // Normally no priority during cleanup
        Ok(())
    }



    // COMBAT HELPER METHODS
    /// Declare a single creature as an attacker (helper for process_declare_attackers_step)
    fn declare_attacker(&mut self, attacker_id: ObjectId, target: AttackTarget) -> Result<(), String> {
        let attacker = self.battlefield.get_mut(&attacker_id)
            .ok_or_else(|| format!("Attacker with ID {} not found on battlefield", attacker_id))?;
        
        // Tap the creature (this is the cost of attacking)
        attacker.state.tapped = true;
        
        // Set the attacking state
        if let Some(creature_aspect) = &mut attacker.state.creature {
            creature_aspect.attacking = Some(AttackingState {
                target,
                blocked_by: Vec::new(),
                is_blocked: false, // Initially not blocked
            });
        } else {
            return Err(format!("Object {} is not a creature", attacker_id));
        }
        
        Ok(())
    }

    /// Declare a single creature as a blocker (helper for process_declare_blockers_step)
    fn declare_blocker(&mut self, blocker_id: ObjectId, attacker_id: ObjectId) -> Result<(), String> {
        // First, verify the attacker exists and is attacking
        let attacker = self.battlefield.get_mut(&attacker_id)
            .ok_or_else(|| format!("Attacker with ID {} not found on battlefield", attacker_id))?;
        
        if let Some(creature_aspect) = &mut attacker.state.creature {
            if let Some(attacking_state) = &mut creature_aspect.attacking {
                // Add this blocker to the attacker's blocked_by list
                attacking_state.blocked_by.push(blocker_id);
            } else {
                return Err(format!("Creature {} is not attacking", attacker_id));
            }
        } else {
            return Err(format!("Object {} is not a creature", attacker_id));
        }

        // Now update the blocker's state
        let blocker = self.battlefield.get_mut(&blocker_id)
            .ok_or_else(|| format!("Blocker with ID {} not found on battlefield", blocker_id))?;
        
        if let Some(creature_aspect) = &mut blocker.state.creature {
            // Initialize blocking state if it doesn't exist
            if creature_aspect.blocking.is_none() {
                creature_aspect.blocking = Some(BlockingState {
                    blocking: Vec::new(),
                    max_can_block: 1, // Default: can block one creature
                });
            }
            
            // Add the attacker to the list of creatures this blocker is blocking
            if let Some(blocking_state) = &mut creature_aspect.blocking {
                blocking_state.blocking.push(attacker_id);
            }
        } else {
            return Err(format!("Object {} is not a creature", blocker_id));
        }
        
        Ok(())
    }


    /// Processes combat damage assignment and resolution
    fn process_combat_damage(&mut self, is_first_strike: bool) -> Result<(), String> {
        // Get damage assignments from UI or API
        let assignments = if let Some(api_assignments) = self.pending_damage_assignments.take() {
            api_assignments // for now we take the assignments from the API as is
        } else {
            // Get assignments from built-in CLI UI
            let ui_assignments = DamageAssignmentUI::get_damage_assignments(self)?;
            ui_assignments
        };

        // Deal all damage simultaneously
        self.deal_combat_damage(assignments)?;

        Ok(())
    }

    /// Deal combat damage assignments
    fn deal_combat_damage(&mut self, assignments: Vec<CombatDamageAssignment>) -> Result<(), String> {
        println!("Dealing {} damage assignments", assignments.len());
        
        for assignment in assignments {
            // Create the target reference for the damage recipient
            let target_ref = match assignment.target_id {
                DamageRecipient::Player(player_id) => TargetRef {
                    ref_id: TargetRefId::Player(player_id)
                },
                DamageRecipient::Creature(creature_id) => TargetRef {
                    ref_id: TargetRefId::Object(creature_id)
                },
                DamageRecipient::Planeswalker(pw_id) => TargetRef {
                    ref_id: TargetRefId::Object(pw_id)
                },
                DamageRecipient::Battle(battle_id) => TargetRef {
                    ref_id: TargetRefId::Object(battle_id)
                },
            };

            // Send them to the damage handler
            self.handle_event(&GameEvent::DamageAboutToBeDealt { 
                source_id: assignment.source_id, 
                target_ref: target_ref.clone(), 
                amount: assignment.amount as u64 
            })?;
        }
        Ok(())
    }
}