// src/game/gamestate/players.rs

use crate::{game::{gamestate::Game, player::Player}, utils::constants::id_types::PlayerId};

impl Game {
    // get a reference to a Player struct from a PlayerId
    pub fn get_player_ref(&self, player_id: PlayerId) -> Result<&Player, String> {
        self.players.iter()
            .find(|player| player.id == player_id)
            .ok_or_else(|| format!("Player with ID {} not found", player_id))
    }

    // get a mutable reference to a Player struct from a PlayerId (identical to get_player_ref, but mutable)
    pub fn get_player_mut(&mut self, player_id: PlayerId) -> Result<&mut Player, String> {
        self.players.iter_mut()
            .find(|player| player.id == player_id)
            .ok_or_else(|| format!("Player with ID {} not found", player_id))
    }
}