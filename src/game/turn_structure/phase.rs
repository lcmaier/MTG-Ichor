// src/game/turn_structure/phase.rs
use crate::utils::constants::turns::{PhaseType, StepType};
use crate::game::turn_structure::step::Step;

#[derive(Debug, Clone)]
pub struct Phase {
    pub phase_type: PhaseType,
    pub current_step: Option<Step>,
    pub is_completed: bool, // to ease state transitions elsewhere we explicitly track if the phase is ready to be transitioned
}

// helper function to get the next phase type in the (normal) turn sequence
pub fn next_phase_type(current: &PhaseType) -> PhaseType {
    match current {
        PhaseType::Beginning => PhaseType::Precombat,
        PhaseType::Precombat => PhaseType::Combat,
        PhaseType::Combat => PhaseType::Postcombat,
        PhaseType::Postcombat => PhaseType::Ending,
        PhaseType::Ending => PhaseType::Beginning, // Cycles back to beginning for next turn
    }
}

impl Phase {
    // Create a new phase struct
    pub fn new(phase_type: PhaseType) -> Self {
        let current_step = match phase_type { // get the first step of the phase we're creating
            PhaseType::Beginning => Some(Step::new(StepType::Untap)),
            PhaseType::Combat => Some(Step::new(StepType::BeginCombat)),
            PhaseType::Ending => Some(Step::new(StepType::End)),
            _ => None, // Main phases don't have steps
        };

        Phase {
            phase_type,
            current_step,
            is_completed: false,
        }
    }

    // logic for handing step transition within a phase -- important that calling this UPDATES the current_phase to whatever the calculated next_phase is
    pub fn next_step(&mut self) -> bool {
        if let Some(step) = &mut self.current_step {
            // if we have a current step, mark it as completed
            step.is_completed = true;

            // determine the next step's type
            let next_step_type = match (self.phase_type, step.step_type) {
                (PhaseType::Beginning, StepType::Untap) => Some(StepType::Upkeep),
                (PhaseType::Beginning, StepType::Upkeep) => Some(StepType::Draw),
                (PhaseType::Beginning, StepType::Draw) => None, // end of beginning phase

                (PhaseType::Combat, StepType::BeginCombat) => Some(StepType::DeclareAttackers),
                (PhaseType::Combat, StepType::DeclareAttackers) => Some(StepType::DeclareBlockers),
                (PhaseType::Combat, StepType::DeclareBlockers) => {
                    // TODO: logic to check if we need a first strike damage step
                    // for now go directly to regular combat damage
                    Some(StepType::CombatDamage)
                },
                (PhaseType::Combat, StepType::FirstStrikeDamage) => Some(StepType::CombatDamage),
                (PhaseType::Combat, StepType::CombatDamage) => Some(StepType::EndCombat),
                (PhaseType::Combat, StepType::EndCombat) => None, // end of combat phase

                (PhaseType::Ending, StepType::End) => Some(StepType::Cleanup),
                (PhaseType::Ending, StepType::Cleanup) => None, // End of ending phase (and turn)

                _ => {
                    // no steps in Main phases, here for match statement completeness
                    // This branch should never be called (the parent if should filter any main phases out), so we'll put an error statement here for debugging purposes
                    println!("Warning! A branch in the match statement for the next_step method of the Phase struct was called that shouldn't be possible, please investigate in src/game/turn_structure/phases.rs");
                    None
                },
            };

            if let Some(step_type) = next_step_type {
                // We successfully found a next step in the match statement, update the current_step
                self.current_step = Some(Step::new(step_type));
                true
            } else {
                // No more steps in this phase
                self.is_completed = true;
                false
            }
        } else {
            // No steps in this phase (Main phase)
            self.is_completed = true;
            false
        }
    }


    // HELPER FUNCTIONS
    // helper for game engine to see if this phase still has steps
    pub fn has_steps(&self) -> bool {
        self.current_step.is_some()
    }

    // Helper to get the current step name
    pub fn current_step_name(&self) -> Option<&'static str> {
        self.current_step.as_ref().map(|step| step.step_type.name())
    }
}