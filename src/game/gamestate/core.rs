use crate::game::player::Player;
use crate::game::turn_structure::phase::next_phase_type;
use crate::game::turn_structure::{phase::Phase, step::Step};
use crate::utils::constants::combat::{AttackingCreature, BlockingCreature};
use crate::utils::constants::game_objects::{BattlefieldState, CommandState, ExileState, GameObj, StackState};
use crate::utils::constants::turns::PhaseType;
use crate::utils::constants::zones::Zone;
use crate::utils::constants::id_types::{ObjectId, PlayerId};


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
        }
    }

    // Helper to get obj ID from index in a specific zone -- this is the ONLY place we should be using indexes to access objects, ObjectId EVERYWHERE else
    pub fn get_obj_id_from_index(&self, player_id: PlayerId, zone: &Zone, index: usize) -> Result<ObjectId, String> {
        match zone {
            Zone::Hand => {
                let player = self.get_player_ref(player_id)?;
                if index >= player.hand.len() {
                    return Err(format!("Index {} out of bounds for hand", index));
                }
                Ok(player.hand[index].id)
            },
            Zone::Library => {
                let player = self.get_player_ref(player_id)?;
                if index >= player.library.len() {
                    return Err(format!("Index {} out of bounds for library", index));
                }
                Ok(player.library[index].id)
            },
            Zone::Graveyard => {
                let player = self.get_player_ref(player_id)?;
                if index >= player.graveyard.len() {
                    return Err(format!("Index {} out of bounds for graveyard", index));
                }
                Ok(player.graveyard[index].id)
            },
            Zone::Battlefield => {
                if index >= self.battlefield.len() {
                    return Err(format!("Index {} out of bounds for battlefield", index));
                }
                Ok(self.battlefield[index].id)
            },
            Zone::Stack => {
                if index >= self.stack.len() {
                    return Err(format!("Index {} out of bounds for stack", index));
                }
                Ok(self.stack[index].id)
            },
            Zone::Exile => {
                if index >= self.exile.len() {
                    return Err(format!("Index {} out of bounds for exile", index));
                }
                Ok(self.exile[index].id)
            },
            Zone::Command => {
                if index >= self.command_zone.len() {
                    return Err(format!("Index {} out of bounds for command zone", index));
                }
                Ok(self.command_zone[index].id)
            },
        }    
    }

    // Advance the gamestate to the next phase or step
    pub fn advance_turn(&mut self) -> Result<(), String> {
        // If we're in a phase with steps, attempt to advance to the next step
        if self.phase.has_steps() {
            if self.phase.next_step() {
                // If next_step() returned Ok(()), the internal state has already been updated to the new step/phase, so we simply evaluate it
                return self.process_current_phase()
            }
        }

        // If this phase doesn't have steps or we couldn't reach a next step (because we were in the last step of the previous phase)
        // we move to the next phase
        let next_phase_type = next_phase_type(&self.phase.phase_type);

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
}