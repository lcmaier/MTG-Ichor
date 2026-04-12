use std::collections::HashMap;

use crate::engine::resolve::ResolvedTarget;
use crate::events::event::DamageTarget;
use crate::oracle::characteristics::get_effective_toughness;
use crate::oracle::mana_helpers::ManaSource;
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

    /// Choose how to divide an attacker's combat damage among multiple blockers.
    /// Called when an attacker with `power` is blocked by 2+ creatures.
    ///
    /// Under 2025 rules (510.1c), the attacking player freely divides damage
    /// among blocking creatures with no ordering or lethal-first constraint.
    ///
    /// Returns a Vec<(blocker_id, damage_amount)> that must:
    /// - Sum to `power`
    /// - Only target blockers in `blockers`
    fn choose_attacker_damage_assignment(
        &self,
        game: &GameState,
        player_id: PlayerId,
        attacker_id: ObjectId,
        blockers: &[ObjectId],
        power: u64,
    ) -> Vec<(ObjectId, u64)>;

    /// Choose how to divide a trampling attacker's damage among blockers and
    /// the defending player/planeswalker.
    ///
    /// Each blocker must be assigned at least lethal damage (1 if `has_deathtouch`,
    /// else toughness − damage_marked). The engine validates the result.
    ///
    /// Returns `(blocker_assignments, overflow_to_defender)` where the total
    /// must equal `power`.
    fn choose_trample_damage_assignment(
        &self,
        game: &GameState,
        player_id: PlayerId,
        attacker_id: ObjectId,
        blockers: &[ObjectId],
        defending_target: DamageTarget,
        power: u64,
        has_deathtouch: bool,
    ) -> (Vec<(ObjectId, u64)>, u64);

    /// Choose which legendary permanent to keep when the legend rule (704.5j)
    /// finds duplicates. The controller has multiple legendaries with the same name;
    /// they choose one to keep and the rest go to the graveyard.
    ///
    /// TODO: This is structurally "choose 1 from N objects." Future needs (stack
    /// ordering, scry top/bottom, modal choices) will likely motivate a generic
    /// `choose_n_from_m` method. Defer generalization until the second instance
    /// arises, then batch-refactor all impls.
    fn choose_legend_to_keep(
        &self,
        game: &GameState,
        player_id: PlayerId,
        legendaries: &[ObjectId],
    ) -> ObjectId;

    /// Choose how to allocate mana from the pool to pay the generic component
    /// of a mana cost. Returns a map of ManaType → amount to spend on generic.
    fn choose_generic_mana_allocation(
        &self,
        game: &GameState,
        player_id: PlayerId,
        mana_cost: &ManaCost,
    ) -> HashMap<ManaType, u64>;
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

    fn choose_attacker_damage_assignment(&self, game: &GameState, _player_id: PlayerId, _attacker_id: ObjectId, blockers: &[ObjectId], power: u64) -> Vec<(ObjectId, u64)> {
        default_damage_assignment(game, blockers, power)
    }

    fn choose_trample_damage_assignment(&self, game: &GameState, _player_id: PlayerId, _attacker_id: ObjectId, blockers: &[ObjectId], _defending_target: DamageTarget, power: u64, has_deathtouch: bool) -> (Vec<(ObjectId, u64)>, u64) {
        default_trample_assignment(game, blockers, power, has_deathtouch)
    }

    fn choose_legend_to_keep(
        &self,
        _game: &GameState,
        _player_id: PlayerId,
        legendaries: &[ObjectId],
    ) -> ObjectId {
        legendaries[0]
    }

    fn choose_generic_mana_allocation(&self, game: &GameState, player_id: PlayerId, mana_cost: &ManaCost) -> HashMap<ManaType, u64> {
        auto_allocate_generic(game, player_id, mana_cost)
            .unwrap_or_default()
    }
}

/// A test-oriented decision provider with pre-scripted decisions.
/// Decisions are consumed in order as they're requested.
pub struct ScriptedDecisionProvider {
    pub attack_decisions: std::cell::RefCell<Vec<Vec<(ObjectId, AttackTarget)>>>,
    pub block_decisions: std::cell::RefCell<Vec<Vec<(ObjectId, ObjectId)>>>,
    pub discard_decisions: std::cell::RefCell<Vec<Option<ObjectId>>>,
    pub priority_decisions: std::cell::RefCell<Vec<PriorityAction>>,
    pub target_decisions: std::cell::RefCell<Vec<Vec<ResolvedTarget>>>,
    pub damage_assignment_decisions: std::cell::RefCell<Vec<Vec<(ObjectId, u64)>>>,
    pub trample_damage_assignment_decisions: std::cell::RefCell<Vec<(Vec<(ObjectId, u64)>, u64)>>,
}

impl ScriptedDecisionProvider {
    pub fn new() -> Self {
        ScriptedDecisionProvider {
            attack_decisions: std::cell::RefCell::new(Vec::new()),
            block_decisions: std::cell::RefCell::new(Vec::new()),
            discard_decisions: std::cell::RefCell::new(Vec::new()),
            priority_decisions: std::cell::RefCell::new(Vec::new()),
            target_decisions: std::cell::RefCell::new(Vec::new()),
            damage_assignment_decisions: std::cell::RefCell::new(Vec::new()),
            trample_damage_assignment_decisions: std::cell::RefCell::new(Vec::new()),
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

    fn choose_attacker_damage_assignment(
        &self,
        game: &GameState,
        _player_id: PlayerId,
        _attacker_id: ObjectId,
        blockers: &[ObjectId],
        power: u64,
    ) -> Vec<(ObjectId, u64)> {
        let mut decisions = self.damage_assignment_decisions.borrow_mut();
        if decisions.is_empty() {
            default_damage_assignment(game, blockers, power)
        } else {
            decisions.remove(0)
        }
    }

    fn choose_trample_damage_assignment(
        &self,
        game: &GameState,
        _player_id: PlayerId,
        _attacker_id: ObjectId,
        blockers: &[ObjectId],
        _defending_target: DamageTarget,
        power: u64,
        has_deathtouch: bool,
    ) -> (Vec<(ObjectId, u64)>, u64) {
        let mut decisions = self.trample_damage_assignment_decisions.borrow_mut();
        if decisions.is_empty() {
            default_trample_assignment(game, blockers, power, has_deathtouch)
        } else {
            decisions.remove(0)
        }
    }

    fn choose_legend_to_keep(
        &self,
        _game: &GameState,
        _player_id: PlayerId,
        legendaries: &[ObjectId],
    ) -> ObjectId {
        // Keep the first one (most recently added to the scripted decisions,
        // or just the first in the list as a default)
        legendaries[0]
    }

    fn choose_generic_mana_allocation(&self, game: &GameState, player_id: PlayerId, mana_cost: &ManaCost) -> HashMap<ManaType, u64> {
        auto_allocate_generic(game, player_id, mana_cost)
            .unwrap_or_default()
    }
}

/// Convenience: default damage assignment for an attacker blocked by multiple creatures.
///
/// Assigns lethal damage to each blocker in listed order, then puts all excess
/// on the last living blocker. This is a common *strategic* choice, not a rules
/// requirement — under 2025 rules (510.1c), the player may divide freely.
/// Concrete `DecisionProvider` implementations call this explicitly — the trait
/// itself has no default.
pub fn default_damage_assignment(
    game: &GameState,
    blockers: &[ObjectId],
    power: u64,
) -> Vec<(ObjectId, u64)> {
    let mut result = Vec::new();
    let mut remaining = power;

    let alive: Vec<ObjectId> = blockers.iter()
        .copied()
        .filter(|id| game.battlefield.contains_key(id))
        .collect();

    for (i, blocker_id) in alive.iter().enumerate() {
        if remaining == 0 {
            break;
        }
        let toughness = get_effective_toughness(game, *blocker_id).unwrap_or(0);
        let damage_marked = game.battlefield.get(blocker_id)
            .map(|e| e.damage_marked)
            .unwrap_or(0);
        let lethal = if toughness > damage_marked as i32 {
            (toughness - damage_marked as i32) as u64
        } else {
            0
        };

        let is_last = i == alive.len() - 1;
        let assign = if is_last {
            // Last blocker gets all remaining damage
            remaining
        } else {
            remaining.min(lethal)
        };

        if assign > 0 {
            result.push((*blocker_id, assign));
            remaining -= assign;
        }
    }

    result
}

/// Default trample damage assignment: assign lethal to each blocker in order,
/// then overflow to the defending player.
///
/// If `has_deathtouch` is true, lethal damage is 1 (rule 702.2c).
/// Otherwise, lethal = toughness − damage_marked.
pub fn default_trample_assignment(
    game: &GameState,
    blockers: &[ObjectId],
    power: u64,
    has_deathtouch: bool,
) -> (Vec<(ObjectId, u64)>, u64) {
    let mut result = Vec::new();
    let mut remaining = power;

    let alive: Vec<ObjectId> = blockers.iter()
        .copied()
        .filter(|id| game.battlefield.contains_key(id))
        .collect();

    for blocker_id in &alive {
        if remaining == 0 {
            break;
        }
        let lethal = if has_deathtouch {
            let damage_marked = game.battlefield.get(blocker_id)
                .map(|e| e.damage_marked)
                .unwrap_or(0);
            if damage_marked > 0 { 0 } else { 1 }
        } else {
            let toughness = get_effective_toughness(game, *blocker_id).unwrap_or(0);
            let damage_marked = game.battlefield.get(blocker_id)
                .map(|e| e.damage_marked)
                .unwrap_or(0);
            if toughness > damage_marked as i32 {
                (toughness - damage_marked as i32) as u64
            } else {
                0
            }
        };

        let assign = remaining.min(lethal);
        if assign > 0 {
            result.push((*blocker_id, assign));
            remaining -= assign;
        }
    }

    // Remaining damage tramples through to the defending player
    (result, remaining)
}

// ---------------------------------------------------------------------------
// Shared helpers for DecisionProvider implementations
// ---------------------------------------------------------------------------

/// Queue mana ability activations followed by a CastSpell action.
///
/// Both CLI and Random DPs use an internal `RefCell<VecDeque<PriorityAction>>`
/// plan queue. When a player wants to cast a spell that needs land taps, we
/// queue `ActivateAbility` for each mana source, then `CastSpell`. The first
/// action is returned immediately; the rest are drained on subsequent
/// `choose_priority_action` calls.
pub fn queue_tap_and_cast(
    queue: &std::cell::RefCell<std::collections::VecDeque<PriorityAction>>,
    sources: &[ManaSource],
    card_id: ObjectId,
) -> PriorityAction {
    let mut q = queue.borrow_mut();

    // Queue mana ability activations (skip the first — we'll return it directly)
    for source in sources.iter().skip(1) {
        q.push_back(PriorityAction::ActivateAbility(
            source.permanent_id,
            source.ability_id,
        ));
    }

    // Queue the cast spell action after all taps
    q.push_back(PriorityAction::CastSpell(card_id));

    // Return the first action immediately
    if let Some(first) = sources.first() {
        PriorityAction::ActivateAbility(first.permanent_id, first.ability_id)
    } else {
        // No tapping needed — cast directly
        q.pop_front().unwrap_or(PriorityAction::Pass)
    }
}

/// Check if a previously-queued action is still valid given the current game state.
///
/// This is a best-effort heuristic, not a full legality check. It catches the
/// most common staleness cases (wrong zone, already tapped, wrong controller)
/// without reimplementing full engine validation. False positives (action passes
/// this check but fails in the engine) are caught by the engine's own checks
/// and produce errors that the DP can handle.
///
/// If one queued action is stale, the entire plan should be discarded — later
/// actions assumed the earlier ones would succeed (e.g. a CastSpell queued
/// after ActivateAbility assumes the mana will be available).
pub fn is_action_still_valid(game: &GameState, player_id: PlayerId, action: &PriorityAction) -> bool {
    match action {
        PriorityAction::Pass => true,
        PriorityAction::PlayLand(card_id) => {
            // Card must still be in hand and owned by player
            game.objects.get(card_id)
                .map(|o| o.owner == player_id && o.zone == crate::types::zones::Zone::Hand)
                .unwrap_or(false)
        }
        PriorityAction::CastSpell(card_id) => {
            // Card must still be in hand and owned by player
            game.objects.get(card_id)
                .map(|o| o.owner == player_id && o.zone == crate::types::zones::Zone::Hand)
                .unwrap_or(false)
        }
        PriorityAction::ActivateAbility(permanent_id, _ability_id) => {
            // Permanent must still be on battlefield and controlled by player.
            // Note: we don't check tapped state here because some abilities
            // (e.g. sacrifice) don't require untapping.
            game.battlefield.get(permanent_id)
                .map(|e| e.controller == player_id)
                .unwrap_or(false)
        }
    }
}

/// A decision provider that dispatches to different providers per player.
///
/// Enables any combination of human/bot/network players in a single game.
/// Each player is assigned a `Box<dyn DecisionProvider>` at construction time.
/// All `DecisionProvider` methods route through `dp_for(player_id)`.
pub struct DispatchDecisionProvider {
    providers: Vec<Box<dyn DecisionProvider>>,
}

impl DispatchDecisionProvider {
    /// Create a new dispatcher from a list of providers, one per player.
    /// Provider at index 0 handles player 0, index 1 handles player 1, etc.
    pub fn new(providers: Vec<Box<dyn DecisionProvider>>) -> Self {
        DispatchDecisionProvider { providers }
    }

    fn dp_for(&self, player_id: PlayerId) -> &dyn DecisionProvider {
        &*self.providers[player_id]
    }
}

impl DecisionProvider for DispatchDecisionProvider {
    fn choose_attackers(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> Vec<(ObjectId, AttackTarget)> {
        self.dp_for(player_id).choose_attackers(game, player_id)
    }

    fn choose_blockers(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> Vec<(ObjectId, ObjectId)> {
        self.dp_for(player_id).choose_blockers(game, player_id)
    }

    fn choose_discard(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> Option<ObjectId> {
        self.dp_for(player_id).choose_discard(game, player_id)
    }

    fn choose_priority_action(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> PriorityAction {
        self.dp_for(player_id).choose_priority_action(game, player_id)
    }

    fn choose_targets(
        &self,
        game: &GameState,
        player_id: PlayerId,
        target_spec: &TargetSpec,
    ) -> Vec<ResolvedTarget> {
        self.dp_for(player_id).choose_targets(game, player_id, target_spec)
    }

    fn choose_attacker_damage_assignment(
        &self,
        game: &GameState,
        player_id: PlayerId,
        attacker_id: ObjectId,
        blockers: &[ObjectId],
        power: u64,
    ) -> Vec<(ObjectId, u64)> {
        self.dp_for(player_id).choose_attacker_damage_assignment(
            game, player_id, attacker_id, blockers, power,
        )
    }

    fn choose_trample_damage_assignment(
        &self,
        game: &GameState,
        player_id: PlayerId,
        attacker_id: ObjectId,
        blockers: &[ObjectId],
        defending_target: DamageTarget,
        power: u64,
        has_deathtouch: bool,
    ) -> (Vec<(ObjectId, u64)>, u64) {
        self.dp_for(player_id).choose_trample_damage_assignment(
            game, player_id, attacker_id, blockers, defending_target, power, has_deathtouch,
        )
    }

    fn choose_legend_to_keep(
        &self,
        game: &GameState,
        player_id: PlayerId,
        legendaries: &[ObjectId],
    ) -> ObjectId {
        self.dp_for(player_id).choose_legend_to_keep(game, player_id, legendaries)
    }

    fn choose_generic_mana_allocation(
        &self,
        game: &GameState,
        player_id: PlayerId,
        mana_cost: &ManaCost,
    ) -> HashMap<ManaType, u64> {
        self.dp_for(player_id).choose_generic_mana_allocation(game, player_id, mana_cost)
    }
}

/// Convenience: greedy auto-allocation of generic mana from a player's pool.
///
/// Calculates surplus mana after reserving for specific (colored) symbols,
/// then greedily assigns surplus to pay the generic component. Concrete
/// `DecisionProvider` implementations call this explicitly — the trait
/// itself has no default.
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
