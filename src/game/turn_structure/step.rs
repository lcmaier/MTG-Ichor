// src/game/turn_structure/step.rs
use crate::utils::constants::turns::StepType;

impl StepType {
    // Get a name for CLI display
    pub fn name(&self) -> &'static str {
        match self {
            StepType::Untap => "Untap Step",
            StepType::Upkeep => "Upkeep Step",
            StepType::Draw => "Draw Step",
            StepType::BeginCombat => "Beginning of Combat Step",
            StepType::DeclareAttackers => "Declare Attackers Step",
            StepType::DeclareBlockers => "Declare Blockers Step",
            StepType::FirstStrikeDamage => "First Strike Combat Damage Step",
            StepType::CombatDamage => "Combat Damage Step",
            StepType::EndCombat => "End of Combat Step",
            StepType::End => "End Step",
            StepType::Cleanup => "Cleanup Step",
        }
    }

    // Check if players receive priority during this step
    pub fn players_receive_priority(&self) -> bool {
        match self {
            StepType::Untap => false,
            StepType::Cleanup => false, // Generally no, but can happen in special cases that will override this
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Step {
    pub step_type: StepType,
    pub is_completed: bool,
}

impl Step {
    pub fn new(step_type: StepType) -> Self {
        Step {
            step_type,
            is_completed: false,
        }
    }
}