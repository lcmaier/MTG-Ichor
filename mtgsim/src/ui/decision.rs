use std::cell::RefCell;
use std::collections::VecDeque;
use std::mem::discriminant;

use crate::oracle::characteristics::get_effective_toughness;
use crate::state::game_state::GameState;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};

use super::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};

/// What a player chooses to do when they have priority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PriorityAction {
    /// Pass priority without taking an action
    Pass,
    /// Cast a spell from a zone it can be cast from
    CastSpell(ObjectId),
    /// Activate an activated ability on a permanent
    ActivateAbility(ObjectId, AbilityId),
    /// Play a land from a zone it could be played from
    PlayLand(ObjectId),
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

// ===========================================================================
// DecisionProvider implementation for Dispatch
// ===========================================================================

/// A decision provider that dispatches to different providers per player.
///
/// Enables any combination of human/bot/network players in a single game.
/// Each player is assigned a `Box<dyn DecisionProvider>` at construction time.
/// All `DecisionProvider` methods route through `dp_for(player_id)`.
pub struct DispatchDecisionProvider {
    providers: Vec<Box<dyn DecisionProvider>>,
}

impl DispatchDecisionProvider {
    /// Create a new dispatcher from a list of generic providers, one per player.
    pub fn new(providers: Vec<Box<dyn DecisionProvider>>) -> Self {
        DispatchDecisionProvider { providers }
    }

    fn dp_for(&self, player_id: PlayerId) -> &dyn DecisionProvider {
        &*self.providers[player_id]
    }
}

impl DecisionProvider for DispatchDecisionProvider {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        self.dp_for(player).pick_n(game, player, context, options, bounds)
    }

    fn pick_number(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        min: u64,
        max: u64,
    ) -> u64 {
        self.dp_for(player).pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.dp_for(player).allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        self.dp_for(player).choose_ordering(game, player, context, items)
    }
}

// ===========================================================================
// 4-PRIMITIVE DECISION PROVIDER TRAIT
// ===========================================================================

/// The 4-primitive decision provider trait.
///
/// Every MtG decision decomposes into one of these four shapes:
/// - `pick_n`: select N items from a list (attackers, targets, discard, etc.)
/// - `pick_number`: choose a number in a range (X value, loop count, etc.)
/// - `allocate`: distribute a total across buckets (damage, mana, etc.)
/// - `choose_ordering`: reorder a list (scry, stack ordering, etc.)
///
/// `ChoiceContext` carries the semantic kind so impls can specialize.
pub trait DecisionProvider {
    /// Pick N items from a list of options. Bounds: (min, max) selections.
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize>;

    /// Pick a number in an inclusive range.
    fn pick_number(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        min: u64,
        max: u64,
    ) -> u64;

    /// Distribute a total across buckets. Sum of returned vec must equal total.
    /// `per_bucket_mins` has one entry per bucket: bucket[i] must receive >= per_bucket_mins[i].
    /// `per_bucket_maxs`, if Some, has one entry per bucket: bucket[i] must receive <= per_bucket_maxs[i].
    /// For uniform minimums, pass `&vec![min; buckets.len()]`.
    /// Pass `None` for maxs when no upper bound is needed.
    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64>;

    /// Order a list of items. Returns indices in desired order.
    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize>;
}

// ---------------------------------------------------------------------------
// Scripted DecisionProvider for tests
// ---------------------------------------------------------------------------

/// A single scripted expectation: what kind of decision we expect, and what to return.
#[derive(Debug)]
pub struct ScriptedExpectation {
    /// The ChoiceKind we expect the engine to ask for (discriminant match, fields ignored).
    /// No `Any` fallback — every scripted decision must declare what it expects.
    pub expected_kind: ChoiceKind,
    pub response: ScriptedResponse,
}

/// The response shape for a scripted expectation.
#[derive(Debug)]
pub enum ScriptedResponse {
    PickN(Vec<usize>),
    Number(u64),
    Allocation(Vec<u64>),
    Ordering(Vec<usize>),
}

/// A test-oriented generic decision provider with a single queue of
/// `ScriptedExpectation`s. Each expectation pairs a mandatory `ChoiceKind`
/// (discriminant-matched) with a `ScriptedResponse`.
///
/// Every test must state what decision it expects — this is the entire point
/// of the self-documenting design.
pub struct ScriptedDecisionProvider {
    queue: RefCell<VecDeque<ScriptedExpectation>>,
}

impl ScriptedDecisionProvider {
    pub fn new() -> Self {
        ScriptedDecisionProvider {
            queue: RefCell::new(VecDeque::new()),
        }
    }

    /// Enqueue a pick_n expectation.
    pub fn expect_pick_n(&self, kind: ChoiceKind, indices: Vec<usize>) {
        self.queue.borrow_mut().push_back(ScriptedExpectation {
            expected_kind: kind,
            response: ScriptedResponse::PickN(indices),
        });
    }

    /// Enqueue a pick_number expectation.
    pub fn expect_number(&self, kind: ChoiceKind, n: u64) {
        self.queue.borrow_mut().push_back(ScriptedExpectation {
            expected_kind: kind,
            response: ScriptedResponse::Number(n),
        });
    }

    /// Enqueue an allocate expectation.
    pub fn expect_allocation(&self, kind: ChoiceKind, alloc: Vec<u64>) {
        self.queue.borrow_mut().push_back(ScriptedExpectation {
            expected_kind: kind,
            response: ScriptedResponse::Allocation(alloc),
        });
    }

    /// Enqueue a choose_ordering expectation.
    pub fn expect_ordering(&self, kind: ChoiceKind, order: Vec<usize>) {
        self.queue.borrow_mut().push_back(ScriptedExpectation {
            expected_kind: kind,
            response: ScriptedResponse::Ordering(order),
        });
    }

    /// Pop the front expectation and validate. Panics if queue is empty or
    /// kind doesn't match — every DP call must have a corresponding expectation.
    fn pop_and_validate(&self, actual_kind: &ChoiceKind, method_name: &str) -> ScriptedResponse {
        let mut queue = self.queue.borrow_mut();
        let expectation = match queue.pop_front() {
            Some(e) => e,
            None => panic!(
                "ScriptedDecisionProvider: unexpected {} call (kind: {:?}), no scripted response in queue",
                method_name, actual_kind
            ),
        };

        assert_eq!(
            discriminant(&expectation.expected_kind),
            discriminant(actual_kind),
            "ScriptedDecisionProvider: kind mismatch — expected {:?}, got {:?}",
            expectation.expected_kind,
            actual_kind,
        );

        expectation.response
    }

    /// Returns true if the queue is empty (all expectations consumed).
    pub fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    /// Returns the number of remaining expectations.
    pub fn remaining(&self) -> usize {
        self.queue.borrow().len()
    }

    /// Enqueue all priority passes for one full empty turn (no creatures,
    /// no spells cast). The turn structure yields 8 priority points where
    /// both players pass:
    ///
    ///   Upkeep, Draw, Precombat, BeginCombat, DeclareAttackers,
    ///   EndCombat, Postcombat, End
    ///
    /// = 8 × 2 players = 16 passes total.
    ///
    /// DeclareBlockers / FirstStrikeDamage / CombatDamage are skipped
    /// when no attackers are declared (rule 508.8). Untap and Cleanup
    /// never grant priority.
    pub fn queue_empty_turn_passes(&self) {
        for _ in 0..16 {
            self.expect_pick_n(
                crate::ui::choice_types::ChoiceKind::PriorityAction,
                vec![0],
            );
        }
    }
}

impl Drop for ScriptedDecisionProvider {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            let remaining = self.queue.borrow().len();
            assert_eq!(
                remaining, 0,
                "ScriptedDecisionProvider dropped with {} unconsumed expectation(s) — \
                 test bug: scripted expectations were enqueued but never consumed by DP calls",
                remaining,
            );
        }
    }
}

impl DecisionProvider for ScriptedDecisionProvider {
    fn pick_n(
        &self,
        _game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        _options: &[ChoiceOption],
        _bounds: (usize, usize),
    ) -> Vec<usize> {
        match self.pop_and_validate(&context.kind, "pick_n") {
            ScriptedResponse::PickN(indices) => indices,
            other => panic!(
                "ScriptedDecisionProvider: pick_n called but scripted response is {:?}, expected PickN",
                other
            ),
        }
    }

    fn pick_number(
        &self,
        _game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        _min: u64,
        _max: u64,
    ) -> u64 {
        match self.pop_and_validate(&context.kind, "pick_number") {
            ScriptedResponse::Number(n) => n,
            other => panic!(
                "ScriptedDecisionProvider: pick_number called but scripted response is {:?}, expected Number",
                other
            ),
        }
    }

    fn allocate(
        &self,
        _game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        _total: u64,
        _buckets: &[ChoiceOption],
        _per_bucket_mins: &[u64],
        _per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        match self.pop_and_validate(&context.kind, "allocate") {
            ScriptedResponse::Allocation(alloc) => alloc,
            other => panic!(
                "ScriptedDecisionProvider: allocate called but scripted response is {:?}, expected Allocation",
                other
            ),
        }
    }

    fn choose_ordering(
        &self,
        _game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        _items: &[ChoiceOption],
    ) -> Vec<usize> {
        match self.pop_and_validate(&context.kind, "choose_ordering") {
            ScriptedResponse::Ordering(order) => order,
            other => panic!(
                "ScriptedDecisionProvider: choose_ordering called but scripted response is {:?}, expected Ordering",
                other
            ),
        }
    }
}
