// src/game/turn_structure/actions.rs
use crate::game::gamestate::Game;
use crate::utils::constants::turns::{PhaseType, StepType};

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
    // (to be implemented in detail later)
    fn process_begin_combat_step(&mut self) -> Result<(), String> {
        self.priority_player_id = self.active_player_id;
        Ok(())
    }

    fn process_declare_attackers_step(&mut self) -> Result<(), String> {
        // Will be implemented later
        self.priority_player_id = self.active_player_id;
        Ok(())
    }

    fn process_declare_blockers_step(&mut self) -> Result<(), String> {
        // Will be implemented later
        self.priority_player_id = self.active_player_id;
        Ok(())
    }

    fn process_first_strike_damage_step(&mut self) -> Result<(), String> {
        // Will be implemented later
        self.priority_player_id = self.active_player_id;
        Ok(())
    }

    fn process_combat_damage_step(&mut self) -> Result<(), String> {
        // Will be implemented later
        self.priority_player_id = self.active_player_id;
        Ok(())
    }

    fn process_end_combat_step(&mut self) -> Result<(), String> {
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
}