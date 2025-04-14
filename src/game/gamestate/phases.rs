// src/game/gamestate/phases.rs
use crate::game::gamestate::core::Game;
use crate::utils::constants::turns::{Phase, Step};

impl Game {
    // Advance the game to the next phase/step
    pub fn next_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Beginning => Phase::Precombat,
            Phase::Precombat => Phase::Combat,
            Phase::Combat => Phase::Postcombat,
            Phase::Postcombat => Phase::Ending,
            Phase::Ending => {
                // End of turn, start a new turn
                self.turn_number += 1;
                self.active_player_id = (self.active_player_id + 1) % self.players.len(); // rotate to the next player
                self.priority_player_id = self.active_player_id; // reset priority to the active player
                Phase::Beginning // reset to beginning phase for the new turn
            }
        };

        // reset step to first in the new phase
        self.step = match self.phase {
            Phase::Beginning => Some(Step::Untap),
            Phase::Combat => Some(Step::BeginCombat),
            Phase::Ending => Some(Step::End),
            _ => None, // no steps in main phases
        };

        println!("Phase changed to: {:?}", self.phase);
        if let Some(step) = &self.step {
            println!("Step: {:?}", step);
        }
    }

    // Check if there are any attacking creatures with first strike or double strike, for use in determining
    // if we need to add a first strike damage step to the combat phase
    fn attacking_first_or_double_strike_creatures(&self) -> bool {
        // will be implemented when I gather the courage to implement first strike creatures
        false
    }

    // Advance to the next step within the current phase
    pub fn next_step(&mut self) -> bool {
        let next_step = match self.phase {
            Phase::Beginning => match self.step {
                Some(Step::Untap) => Some(Step::Upkeep),
                Some(Step::Upkeep) => Some(Step::Draw),
                Some(Step::Draw) => None, // end of beginning phase steps
                _ => None,
            },
            Phase::Combat => match self.step {
                Some(Step::BeginCombat) => Some(Step::DeclareAttackers),
                Some(Step::DeclareAttackers) => Some(Step::DeclareBlockers),
                Some(Step::DeclareBlockers) => {
                    // Check if we need a first strike damage step
                    if self.attacking_first_or_double_strike_creatures() {
                        Some(Step::FirstStrikeDamage)
                    } else {
                        Some(Step::CombatDamage)
                    }
                }
                Some(Step::FirstStrikeDamage) => Some(Step::CombatDamage),
                Some(Step::CombatDamage) => Some(Step::EndCombat),
                Some(Step::EndCombat) => None, // end of combat phase steps
                _ => None,
            },
            Phase::Ending => match self.step {
                Some(Step::End) => Some(Step::Cleanup),
                Some(Step::Cleanup) => None, // end of ending phase steps
                _ => None,
            },
            _ => None, // main phases don't have steps
        };

        self.step = next_step;
        if let Some(step) = &self.step {
            println!("Step changed to: {:?}", step);
            true
        } else {
            false // no more steps in this phase, return false
        }
    }

    // Process current phase and step
    pub fn process_current_phase_and_step(&mut self) -> Result<(), String> {
        match self.phase {
            Phase::Beginning => {
                match self.step {
                    Some(Step::Untap) => self.process_untap_step(),
                    Some(Step::Upkeep) => self.process_upkeep_step(),
                    Some(Step::Draw) => self.process_draw_step(),
                    _ => Ok(()),
                }
            },
            Phase::Precombat => {
                self.process_main_phase()
            },
            Phase::Combat => {
                // skip for now, will be implemented later
                Ok(())
            },
            Phase::Postcombat => {
                self.process_main_phase()
            },
            Phase::Ending => {
                match self.step {
                    Some(Step::End) => self.process_end_step(),
                    Some(Step::Cleanup) => self.process_cleanup_step(),
                    _ => Ok(()),
                }
            },
        }
    }

    // Process each step
    fn process_untap_step(&mut self) -> Result<(), String> {
        // First, all of active player's phased-in permanents with phasing phase out, and all permanents that were phased-out with phasing phase in (rule 502.1)
        // NOT YET IMPLEMENTED

        // Second, if it's day and last turn's active player cast no spells last turn, it becomes night (rule 502.2)
        // Similarly, if it's night and last turn's active player cast 2 or more spells last turn, it becomes day (rule 502.2)
        // (if it wasn't day or night last turn, this check does not happen at all)
        // NOT YET IMPLEMENETED
        
        // Third, untap all permanents controlled by the active player (or as many as possible) (rule 502.4)
        let active_player_id = self.active_player_id;

        for permanent in self.battlefield.iter_mut() {
            let controller = permanent.get_controller();
            if controller.unwrap() == active_player_id {
                // Untap logic here
                continue;
                //permanent.untap();
            }
        }

        // Reset land played counter for active player
        let active_player = self.get_player_mut(active_player_id)?;
        active_player.reset_lands_played();

        // There is no priority in the untap step (rule 502.4) so we can move directly to the next step
        println!("Untap step completed.");
        Ok(())
    }

    fn process_upkeep_step(&mut self) -> Result<(), String> {
        // no turn based actions, but active player gets priority at the beginning of the step (rule 503.1)
        self.priority_player_id = self.active_player_id;

        // check for triggered abilities that trigger at the beginning of upkeep (rule 503.2)
        // NOT YET IMPLEMENTED

        // need to implement check for both players passing priority, which signals the end of the step
        // NOT YET IMPLEMENTED
        Ok(())
    }

    fn process_draw_step(&mut self) -> Result<(), String> {
        // active player draws a card (rule 504.1)
        let active_player = self.get_player_mut(self.active_player_id)?;
        active_player.draw_card()?;

        // active player gets priority (rule 504.2)
        self.priority_player_id = self.active_player_id;

        // need to implement check for both players passing priority, which signals the end of the step
        // NOT YET IMPLEMENTED
        Ok(())
    }

    fn process_main_phase(&mut self) -> Result<(), String> {
        // provided we're not in an archenemy game or using contraptions, a few things happen at the beginning of the main phase
        // If the active player controls a Saga(s) and it's the precombat main phase, the active player puts
        // a lore counter on each Saga they control (rule 505.4)
        // NOT YET IMPLEMENTED

        // Active player gets priority
        self.priority_player_id = self.active_player_id;

        // need to implement check for both players passing priority, which signals the end of the step
        // NOT YET IMPLEMENTED

        Ok(())
    }

    fn process_end_step(&mut self) -> Result<(), String> {
        // Active player gets priority
        self.priority_player_id = self.active_player_id;

        // need to implement check for both players passing priority, which signals the end of the step
        // NOT YET IMPLEMENTED

        Ok(())
    }

    fn process_cleanup_step(&mut self) -> Result<(), String> {
        // If active player's hand has more cards than their max hand size (usually 7), discard cards to reduce
        // hand size to that number (turn-based and doesn't use the stack)
        let mut_active_player = match self.get_player_mut(self.active_player_id) {
            Ok(player) => player,
            Err(e) => return Err(e),
        };
        
        let mut temp = 0;
        while mut_active_player.max_hand_size > mut_active_player.hand.len() as u32 && temp < 5 {
            // logic to (turn-based and not using the stack)
            // NOT YET IMPLEMENTED, below is just to make the while loop end
            temp += 1;
        }

        // Next, the following things happen simultaneously
        // all damage marked on all permanents (including phased-out permanents) is removed and "until end of turn" and "this turn" effects end (rule 514.2)
        // NOT YET IMPLEMENTED

        // Normally, no player receives priority during cleanup, but if there are any state based actions to be performed and/or triggered
        // abilities waiting to be put on the stack, those state-based actions are performed, then those triggered abilities are put on the stack,
        // then the active player gets priority. Once priority passes through all players, another cleanup step is performed 
        // (repeat until cleanup is passed through cleanly) (Rules 514.3 and 514.3a)
        // NOT YET IMPLEMENTED

        Ok(())


    }


    // handle passing priority
    pub fn pass_priority(&mut self) -> Result<bool, String> {
        // Find next player in turn order
        let player_count = self.players.len();
        let next_player_id = (self.priority_player_id + 1) % player_count;

        // if priority is passed back to active player with stack empty, marks end of step/phase (i.e., priority has been "passed around")
        if next_player_id == self.active_player_id {
            if self.stack.is_empty() {
                // move to next step, if we've run out in this phase then we move to the next phase
                if !self.next_step() {
                    self.next_phase();
                }

                // process the new phase/step
                self.process_current_phase_and_step()?;
                return Ok(true);
            }
            // otherwise, resolve the top of the stack
            // NOT YET IMPLEMENTED
        }

        // pass priority to the next player
        self.priority_player_id = next_player_id;
        println!("Priority passed to player {}", self.priority_player_id);
        Ok(false)
    }



}