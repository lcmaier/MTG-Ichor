use crate::state::battlefield::AttackTarget;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};

/// Abstraction for player decisions.
///
/// The engine calls methods on this trait whenever it needs a player to make
/// a choice. Implementations can be:
/// - CLI (interactive terminal play)
/// - Programmatic (for tests — pre-scripted decisions)
/// - AI (bot players)
/// - Network (remote player over a protocol)
///
/// This keeps the engine completely decoupled from input/output.
pub trait DecisionProvider {
    /// Choose which creatures to declare as attackers.
    /// Returns a list of (attacker_id, attack_target) pairs.
    fn choose_attackers(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> Vec<(ObjectId, AttackTarget)>;

    /// Choose which creatures to declare as blockers.
    /// Returns a list of (blocker_id, attacker_id) pairs.
    fn choose_blockers(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> Vec<(ObjectId, ObjectId)>;

    /// Choose a card from hand to discard (e.g., cleanup step discard to hand size).
    /// Returns the ObjectId of the card to discard.
    fn choose_discard(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> Option<ObjectId>;
}

/// A test-oriented decision provider that returns empty decisions (no attacks, no blocks).
/// Useful for tests that just need the turn loop to advance without interaction.
pub struct PassiveDecisionProvider;

impl DecisionProvider for PassiveDecisionProvider {
    fn choose_attackers(&self, _game: &GameState, _player_id: PlayerId) -> Vec<(ObjectId, AttackTarget)> {
        Vec::new()
    }

    fn choose_blockers(&self, _game: &GameState, _player_id: PlayerId) -> Vec<(ObjectId, ObjectId)> {
        Vec::new()
    }

    fn choose_discard(&self, _game: &GameState, _player_id: PlayerId) -> Option<ObjectId> {
        None
    }
}

/// A test-oriented decision provider with pre-scripted decisions.
/// Decisions are consumed in order as they're requested.
pub struct ScriptedDecisionProvider {
    pub attack_decisions: std::cell::RefCell<Vec<Vec<(ObjectId, AttackTarget)>>>,
    pub block_decisions: std::cell::RefCell<Vec<Vec<(ObjectId, ObjectId)>>>,
    pub discard_decisions: std::cell::RefCell<Vec<Option<ObjectId>>>,
}

impl ScriptedDecisionProvider {
    pub fn new() -> Self {
        ScriptedDecisionProvider {
            attack_decisions: std::cell::RefCell::new(Vec::new()),
            block_decisions: std::cell::RefCell::new(Vec::new()),
            discard_decisions: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl DecisionProvider for ScriptedDecisionProvider {
    fn choose_attackers(&self, _game: &GameState, _player_id: PlayerId) -> Vec<(ObjectId, AttackTarget)> {
        let mut decisions = self.attack_decisions.borrow_mut();
        if decisions.is_empty() {
            Vec::new()
        } else {
            decisions.remove(0)
        }
    }

    fn choose_blockers(&self, _game: &GameState, _player_id: PlayerId) -> Vec<(ObjectId, ObjectId)> {
        let mut decisions = self.block_decisions.borrow_mut();
        if decisions.is_empty() {
            Vec::new()
        } else {
            decisions.remove(0)
        }
    }

    fn choose_discard(&self, _game: &GameState, _player_id: PlayerId) -> Option<ObjectId> {
        let mut decisions = self.discard_decisions.borrow_mut();
        if decisions.is_empty() {
            None
        } else {
            decisions.remove(0)
        }
    }
}
