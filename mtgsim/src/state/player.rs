use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::ManaPool;

/// Per-player state in the game.
///
/// Player-owned zones (hand, library, graveyard) store ObjectIds — the actual
/// GameObjects live in GameState's central object store.
#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: PlayerId,
    pub life_total: i64,
    pub mana_pool: ManaPool,

    // Player-owned zones (ordered collections of object IDs)
    pub library: Vec<ObjectId>,
    pub hand: Vec<ObjectId>,
    pub graveyard: Vec<ObjectId>,

    // Turn-specific state
    pub max_hand_size: i32,
    pub lands_per_turn: u32,
    pub lands_played_this_turn: u32,

    // SBA flags — these are ONLY for state-based action checks (rule 704).
    // General per-turn tracking (e.g. "cast a spell this turn") should live
    // in a separate TurnTracker struct when needed.
    pub has_drawn_from_empty_library: bool,
}

impl PlayerState {
    pub fn new(id: PlayerId, starting_life: i64) -> Self {
        PlayerState {
            id,
            life_total: starting_life,
            mana_pool: ManaPool::new(),
            library: Vec::new(),
            hand: Vec::new(),
            graveyard: Vec::new(),
            max_hand_size: 7,
            lands_per_turn: 1,
            lands_played_this_turn: 0,
            has_drawn_from_empty_library: false,
        }
    }

    pub fn can_play_land(&self) -> bool {
        self.lands_played_this_turn < self.lands_per_turn
    }

    pub fn reset_lands_played(&mut self) {
        self.lands_played_this_turn = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = PlayerState::new(0, 20);
        assert_eq!(player.id, 0);
        assert_eq!(player.life_total, 20);
        assert_eq!(player.max_hand_size, 7);
        assert_eq!(player.lands_per_turn, 1);
        assert!(player.library.is_empty());
        assert!(player.hand.is_empty());
        assert!(player.graveyard.is_empty());
    }

    #[test]
    fn test_land_play_tracking() {
        let mut player = PlayerState::new(0, 20);
        assert!(player.can_play_land());

        player.lands_played_this_turn = 1;
        assert!(!player.can_play_land());

        player.reset_lands_played();
        assert!(player.can_play_land());
    }
}
