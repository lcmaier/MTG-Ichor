use crate::game::player::Player;
use crate::game::turn_structure::phase::{self, next_phase_type};
use crate::game::turn_structure::{phase::Phase, step::Step};
use crate::utils::constants::combat::{AttackingCreature, BlockingCreature};
use crate::utils::constants::effect_context::EffectContext;
use crate::utils::constants::events::{EventHandler, GameEvent};
use crate::utils::constants::game_objects::{BattlefieldState, CommandState, ExileState, GameObj, StackState};
use crate::utils::constants::turns::PhaseType;
use crate::utils::constants::zones::Zone;
use crate::utils::constants::id_types::{ObjectId, PlayerId};
use crate::utils::constants::card_types::CardType;

#[derive(Debug, Clone)]
pub struct Game {
    pub players: Vec<Player>,
    pub active_player_id: usize, // the active player is the one whose turn it is (by definition), so this doubles as a turn player index
    pub priority_player_id: usize,
    pub turn_number: u32,
    pub phase: Phase,
    // global zones (Player zones hand, library, graveyard are within Player struct)
    pub stack: Vec<GameObj<StackState>>, // stack of objects (spells, abilities, etc.)
    pub battlefield: Vec<GameObj<BattlefieldState>>, // battlefield objects (creatures, enchantments, tokens, etc.)
    pub exile: Vec<GameObj<ExileState>>,
    pub command_zone: Vec<GameObj<CommandState>>,

    // Combat tracking
    pub attacking_creatures: Vec<AttackingCreature>, // creatures attacking this turn
    pub blocking_creatures: Vec<BlockingCreature>, // creatures blocking this turn

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
            turn_number: 0,
            phase: Phase::new(PhaseType::Beginning),
            stack: Vec::new(),
            battlefield: Vec::new(),
            exile: Vec::new(),
            command_zone: Vec::new(),
            attacking_creatures: Vec::new(),
            blocking_creatures: Vec::new(),
            effect_context: EffectContext::new(),
        }
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
        // need to clone the value up here so we can pass the spell/ability's controller to the resolution functionk
        let top_obj_clone = top_object.clone();

        // Resolve it based on card type (permanents go to battlefield, nonpermanents go to graveyard)
        if let Some(card_types) = &top_object.characteristics.card_type {
            if card_types.contains(&CardType::Instant) || card_types.contains(&CardType::Sorcery) {
                top_object.resolve_as_nonpermanent()?;
            } else {
                top_object.resolve_as_permanent(top_obj_clone.state.controller)?;
            }
        } else {
            // must be an ability on the stack
            top_object.resolve_as_ability()?;
        }
    
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
                // Implementation would be in a separate method
                Ok(()) // Placeholder for now
            },
            GameEvent::PermanentDestroyed { permanent_id, reason } => {
                // Implementation would be in a separate method
                Ok(()) // Placeholder for now
            },
            GameEvent::PermanentSacrificed { permanent_id } => {
                // Implementation would be in a separate method
                Ok(()) // Placeholder for now
            },
        }
    }
}