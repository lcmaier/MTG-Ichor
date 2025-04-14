use crate::game::player::Player;
use crate::game::game_obj::GameObj;
use crate::utils::constants::combat::{AttackingCreature, BlockingCreature};
use crate::utils::constants::turns::{Phase, Step};
use crate::utils::constants::zones::Zone;
use crate::utils::constants::id_types::{ObjectId, PlayerId};


pub struct Game {
    pub players: Vec<Player>,
    pub active_player_id: usize, // the active player is the one whose turn it is (by definition), so this doubles as a turn player index
    pub priority_player_id: usize,
    pub turn_number: u32,
    pub phase: Phase,
    pub step: Option<Step>,

    // global zones (Player zones hand, library, graveyard are within Player struct)
    pub stack: Vec<GameObj>, // stack of objects (spells, abilities, etc.)
    pub battlefield: Vec<GameObj>, // battlefield objects (creatures, enchantments, tokens, etc.)
    pub exile: Vec<GameObj>,
    pub command_zone: Vec<GameObj>,

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
            phase: Phase::Beginning,
            step: None, // None to denote pregame (mulligans, pregame actions, etc.)
            stack: Vec::new(),
            battlefield: Vec::new(),
            exile: Vec::new(),
            command_zone: Vec::new(),
            attacking_creatures: Vec::new(),
            blocking_creatures: Vec::new(),
        }
    }

    // Helper to get card ID from index in a specific zone -- this is the ONLY place we should be using indexes to access cards, ObjectId everywhere else
    pub fn get_card_id_from_index(&self, player_id: PlayerId, zone: &Zone, index: usize) -> Result<ObjectId, String> {
        match zone {
            Zone::Hand => {
                let player = self.get_player_ref(player_id)?;
                if index >= player.hand.len() {
                    return Err(format!("Index {} out of bounds for hand", index));
                }
                match &player.hand[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
            Zone::Library => {
                let player = self.get_player_ref(player_id)?;
                if index >= player.library.len() {
                    return Err(format!("Index {} out of bounds for library", index));
                }
                match &player.library[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
            Zone::Graveyard => {
                let player = self.get_player_ref(player_id)?;
                if index >= player.graveyard.len() {
                    return Err(format!("Index {} out of bounds for graveyard", index));
                }
                match &player.graveyard[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
            Zone::Battlefield => {
                if index >= self.battlefield.len() {
                    return Err(format!("Index {} out of bounds for battlefield", index));
                }
                match &self.battlefield[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
            Zone::Stack => {
                if index >= self.stack.len() {
                    return Err(format!("Index {} out of bounds for stack", index));
                }
                match &self.stack[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
            Zone::Exile => {
                if index >= self.exile.len() {
                    return Err(format!("Index {} out of bounds for exile", index));
                }
                match &self.exile[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
            Zone::Command => {
                if index >= self.command_zone.len() {
                    return Err(format!("Index {} out of bounds for command zone", index));
                }
                match &self.command_zone[index] {
                    GameObj::Card { id, .. } => Ok(*id),
                }
            },
        }
    }
}