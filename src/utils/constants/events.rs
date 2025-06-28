// src/utils/constants/events.rs

use std::collections::HashMap;

use crate::utils::mana::ManaType;

use super::id_types::{ObjectId, PlayerId};
use crate::utils::targeting::core::TargetRef;

pub trait EventHandler {
    fn handle_event(&mut self, event: &GameEvent) -> Result<(), String>;
}

// Add this enum to define reasons for destruction
#[derive(Debug, Clone, PartialEq)]
pub enum DeathReason {
    ZeroToughness,       // Directly to graveyard, can't be regenerated
    LethalDamage,        // Destroyed, can be regenerated
    DestroyEffect,       // Destroyed by an effect like "Destroy target creature"
    Sacrifice,           // Sacrificed (not destroyed)
    LegendRule,          // Due to legend rule (not destroyed)
    // Add other reasons as needed
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


    //// DAMAGE EVENTS ////
    // Event for when damage is about to be dealt
    // (we say 'about to be' to allow for replacement and prevention effects on that damage)
    DamageAboutToBeDealt {
        source_id: ObjectId,
        target_ref: TargetRef,
        amount: u64,
    },

    // Event for when damage is actually dealt (after replacement/prevention effects are applied)
    DamageDealt {
        source_id: ObjectId,
        target_ref: TargetRef,
        amount: u64,
    },


    //// STATE-BASED ACTION EVENTS ////
    // General event for checking state-based actions
    CheckStateBasedActions,

    // Event for when a creature with toughness 0 or less is put into its owners graveyard (rule 704.5f)
    CreatureZeroToughness {
        creature_id: ObjectId,
    },

    // Event for when permanent is destroyed
    PermanentDestroyed {
        permanent_id: ObjectId,
        reason: DeathReason,
    },

    // Event for when permanent is sacrificed
    PermanentSacrificed {
        permanent_id: ObjectId,
    },

    PermanentEntersBattlefield { permanent_id: ObjectId },
    PermanentLeavesBattlefield { permanent_id: ObjectId },
    CreatureDies { creature_id: ObjectId },
    SpellCast { spell_id: ObjectId, caster_id: PlayerId },
    EndOfTurn { player_id: PlayerId },
}