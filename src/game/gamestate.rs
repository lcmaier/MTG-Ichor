// src/game/gamestate.rs
use crate::player::Player;
use crate::game::game_obj::GameObj;
use crate::utils::constants::turns::{Phase, Step};

// game struct
pub struct Game {
    pub players: Vec<Player>,
    pub active_player_index: usize, // the active player is the one whose turn it is (by definition), so this doubles as a turn player index
    pub priority_player_index: usize,
    pub turn_number: u32,
    pub phase: Phase,
    pub step: Option<Step>,

    // global zones (Player zones like hand, library, graveyard are within Player struct)
    pub stack: Vec<GameObj>, // stack of objects (spells, abilities, etc.)
    pub battlefield: Vec<GameObj>, // battlefield objects (creatures, enchantments, tokens, etc.)
    pub exile: Vec<GameObj>,
    pub command_zone: Vec<GameObj>,
}

