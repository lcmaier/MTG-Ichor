// src/utils/constants/events.rs

use std::collections::HashMap;

use crate::utils::mana::ManaType;

use super::id_types::{ObjectId, PlayerId};

pub trait EventHandler {
    fn handle_event(&mut self, event: &GameEvent) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub enum GameEvent {
    //// MANA EVENTS ////
    // Activated Mana Ability Event
    ManaAbilityActivated {
        source_id: ObjectId,
        player_id: PlayerId,
    },
    // Event to add mana to a player's mana pool
    ManaAdded {
        source_id: ObjectId,
        player_id: PlayerId,
        mana_types: HashMap<ManaType, u64>,
    },

    //// PHASE/STEP EVENTS ////
    // Phase/step transition events
    PhaseEnded {
        phase_type: crate::utils::constants::turns::PhaseType,
    },
    StepEnded {
        step_type: crate::utils::constants::turns::StepType,
    },

}