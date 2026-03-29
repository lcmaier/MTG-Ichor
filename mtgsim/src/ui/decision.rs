use std::collections::HashMap;

use crate::engine::resolve::ResolvedTarget;
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::GameState;
use crate::types::effects::TargetSpec;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};

/// What a player chooses to do when they have priority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PriorityAction {
    /// Pass priority without taking an action
    Pass,
    /// Cast a spell from hand
    CastSpell(ObjectId),
    /// Activate an activated ability on a permanent
    ActivateAbility(ObjectId, AbilityId),
    /// Play a land from hand
    PlayLand(ObjectId),
}

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

    /// Choose what to do when the player has priority.
    fn choose_priority_action(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> PriorityAction;

    /// Choose targets for a spell or ability being cast/activated.
    fn choose_targets(
        &self,
        game: &GameState,
        player_id: PlayerId,
        target_spec: &TargetSpec,
    ) -> Vec<ResolvedTarget>;

    /// Choose how to allocate mana from the pool to pay the generic component
    /// of a mana cost. Returns a map of ManaType → amount to spend on generic.
    ///
    /// The default implementation uses a greedy auto-allocation that spends
    /// surplus mana (after reserving for specific symbols). Real player UIs
    /// or AI can override this for full manual control.
    fn choose_generic_mana_allocation(
        &self,
        game: &GameState,
        player_id: PlayerId,
        mana_cost: &ManaCost,
    ) -> HashMap<ManaType, u64> {
        auto_allocate_generic(game, player_id, mana_cost)
            .unwrap_or_default()
    }
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

    fn choose_discard(&self, game: &GameState, player_id: PlayerId) -> Option<ObjectId> {
        // Auto-discard the last card in hand (simple default for tests)
        game.players.get(player_id)
            .and_then(|p| p.hand.last().copied())
    }

    fn choose_priority_action(&self, _game: &GameState, _player_id: PlayerId) -> PriorityAction {
        PriorityAction::Pass
    }

    fn choose_targets(&self, _game: &GameState, _player_id: PlayerId, _target_spec: &TargetSpec) -> Vec<ResolvedTarget> {
        Vec::new()
    }

    // choose_generic_mana_allocation: uses default greedy implementation
}

/// A test-oriented decision provider with pre-scripted decisions.
/// Decisions are consumed in order as they're requested.
pub struct ScriptedDecisionProvider {
    pub attack_decisions: std::cell::RefCell<Vec<Vec<(ObjectId, AttackTarget)>>>,
    pub block_decisions: std::cell::RefCell<Vec<Vec<(ObjectId, ObjectId)>>>,
    pub discard_decisions: std::cell::RefCell<Vec<Option<ObjectId>>>,
    pub priority_decisions: std::cell::RefCell<Vec<PriorityAction>>,
    pub target_decisions: std::cell::RefCell<Vec<Vec<ResolvedTarget>>>,
}

impl ScriptedDecisionProvider {
    pub fn new() -> Self {
        ScriptedDecisionProvider {
            attack_decisions: std::cell::RefCell::new(Vec::new()),
            block_decisions: std::cell::RefCell::new(Vec::new()),
            discard_decisions: std::cell::RefCell::new(Vec::new()),
            priority_decisions: std::cell::RefCell::new(Vec::new()),
            target_decisions: std::cell::RefCell::new(Vec::new()),
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

    fn choose_priority_action(&self, _game: &GameState, _player_id: PlayerId) -> PriorityAction {
        let mut decisions = self.priority_decisions.borrow_mut();
        if decisions.is_empty() {
            PriorityAction::Pass
        } else {
            decisions.remove(0)
        }
    }

    fn choose_targets(&self, _game: &GameState, _player_id: PlayerId, _target_spec: &TargetSpec) -> Vec<ResolvedTarget> {
        let mut decisions = self.target_decisions.borrow_mut();
        if decisions.is_empty() {
            Vec::new()
        } else {
            decisions.remove(0)
        }
    }

    // choose_generic_mana_allocation: uses default greedy implementation
}

/// Greedy auto-allocation of generic mana from a player's pool.
///
/// Calculates surplus mana after reserving for specific (colored) symbols,
/// then greedily assigns surplus to pay the generic component. This is the
/// default implementation used by DecisionProvider; real player UIs or AI
/// implementations can override `choose_generic_mana_allocation` for full
/// manual control.
pub fn auto_allocate_generic(
    game: &GameState,
    player_id: PlayerId,
    mana_cost: &ManaCost,
) -> Result<HashMap<ManaType, u64>, String> {
    let generic_count = mana_cost.generic_count() as u64;
    if generic_count == 0 {
        return Ok(HashMap::new());
    }

    let player = game.get_player(player_id)?;
    let pool = &player.mana_pool;

    // Calculate how much of each colored type is needed for specific symbols
    let mut specific_needed: HashMap<ManaType, u64> = HashMap::new();
    for sym in &mana_cost.symbols {
        if let ManaSymbol::Colored(mt) = sym {
            *specific_needed.entry(*mt).or_insert(0) += 1;
        }
    }

    // Calculate surplus available after paying specific costs
    let mut available: HashMap<ManaType, u64> = HashMap::new();
    for mt in &[ManaType::White, ManaType::Blue, ManaType::Black, ManaType::Red, ManaType::Green, ManaType::Colorless] {
        let in_pool = pool.amount(*mt);
        let needed = specific_needed.get(mt).copied().unwrap_or(0);
        if in_pool > needed {
            available.insert(*mt, in_pool - needed);
        }
    }

    // Greedily allocate generic from available surplus
    let mut allocation = HashMap::new();
    let mut remaining = generic_count;
    for (mt, avail) in &available {
        if remaining == 0 {
            break;
        }
        let use_amount = (*avail).min(remaining);
        if use_amount > 0 {
            allocation.insert(*mt, use_amount);
            remaining -= use_amount;
        }
    }

    if remaining > 0 {
        return Err(format!(
            "Not enough mana to pay generic cost: need {} more",
            remaining
        ));
    }

    Ok(allocation)
}
