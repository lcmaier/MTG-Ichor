//! Shared helpers for integration tests.
//!
//! Contains phase-agnostic setup utilities used across multiple test files.
//! Phase-specific helpers (e.g. `cast_and_resolve`, `advance_to_step`) remain
//! in their respective test files.

use std::sync::Arc;

use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::state::battlefield::BattlefieldEntity;
use mtgsim::state::game_state::{GameState, PhaseType};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::zones::Zone;

/// Create a minimal two-player game in precombat main phase with player 0 active.
#[allow(dead_code)]
pub fn setup_two_player_game() -> GameState {
    let mut game = GameState::new(2, 20);
    game.phase = mtgsim::state::game_state::Phase::new(PhaseType::Precombat);
    game.active_player = 0;
    game
}

/// Put a card into a player's hand and register it in the game.
#[allow(dead_code)]
pub fn put_in_hand(game: &mut GameState, card_data: Arc<CardData>, player: usize) -> ObjectId {
    let obj = GameObject::new(card_data, player, Zone::Hand);
    let id = obj.id;
    game.add_object(obj);
    game.players[player].hand.push(id);
    id
}

/// Put a land onto the battlefield for a player (from a factory function).
#[allow(dead_code)]
pub fn put_land_on_battlefield(
    game: &mut GameState,
    land_fn: fn() -> Arc<CardData>,
    player: usize,
) -> ObjectId {
    let card_data = land_fn();
    put_on_battlefield(game, card_data, player)
}

/// Put any permanent onto the battlefield for a player.
#[allow(dead_code)]
pub fn put_on_battlefield(
    game: &mut GameState,
    card_data: Arc<CardData>,
    player: usize,
) -> ObjectId {
    let obj = GameObject::new(card_data, player, Zone::Battlefield);
    let id = obj.id;
    let ts = game.allocate_timestamp();
    game.add_object(obj);
    let entry = BattlefieldEntity::new(id, player, ts, 0);
    game.battlefield.insert(id, entry);
    id
}

/// Give a player some dummy cards in their library (for draw effects).
#[allow(dead_code)]
pub fn fill_library(game: &mut GameState, player: usize, count: usize) {
    for _ in 0..count {
        let card = CardDataBuilder::new("Dummy Card").build();
        let obj = GameObject::new(card, player, Zone::Library);
        let id = obj.id;
        game.add_object(obj);
        game.players[player].library.push(id);
    }
}
