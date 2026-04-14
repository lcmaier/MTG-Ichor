//! Typed `ask_*` free functions — the engine-side bridge to the 4-primitive
//! `GenericDecisionProvider` trait.
//!
//! Each function:
//! 1. Constructs a `ChoiceContext` with the appropriate `ChoiceKind`
//! 2. Packs options into `Vec<ChoiceOption>`
//! 3. Calls the appropriate DP primitive (`pick_n`, `pick_number`, `allocate`, `choose_ordering`)
//! 4. Validates the response (bounds, count, sum, permutation)
//! 5. Unpacks indices back into typed results
//!
//! Engine call sites and tests use these exclusively — never raw DP methods.

use std::collections::HashMap;

use crate::engine::resolve::ResolvedTarget;
use crate::events::event::DamageTarget;
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::GameState;
use crate::types::costs::{AdditionalCost, AlternativeCost};
use crate::types::effects::EffectRecipient;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaType};

use super::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use super::decision::{GenericDecisionProvider, PriorityAction};

// ===========================================================================
// Validation helpers
// ===========================================================================

/// Validate pick_n response: indices in range, no duplicates, count in bounds.
fn validate_pick_n(
    indices: &[usize],
    options_len: usize,
    bounds: (usize, usize),
    context_desc: &str,
) {
    assert!(
        indices.len() >= bounds.0 && indices.len() <= bounds.1,
        "ask_{}: DP returned {} selections, expected {}-{}",
        context_desc,
        indices.len(),
        bounds.0,
        bounds.1,
    );

    for &idx in indices {
        assert!(
            idx < options_len,
            "ask_{}: DP returned index {} but only {} options available",
            context_desc,
            idx,
            options_len,
        );
    }

    // Check for duplicates
    let mut seen = std::collections::HashSet::new();
    for &idx in indices {
        assert!(
            seen.insert(idx),
            "ask_{}: DP returned duplicate index {}",
            context_desc,
            idx,
        );
    }
}

/// Validate pick_number response: value in range.
///
/// TODO(Phase 9): When `GameNumber` replaces `u64` for symbolic/comparative
/// values, this will need to use `GameNumber::gte`/`GameNumber::lte` instead
/// of direct integer comparison.
///
/// Not yet used — `ask_choose_x_value` delegates affordability to rollback.
/// Will be needed for future bounded `pick_number` callers (loop count, etc.).
#[allow(dead_code)]
fn validate_pick_number(value: u64, min: u64, max: u64, context_desc: &str) {
    assert!(
        value >= min && value <= max,
        "ask_{}: DP returned {} but range is [{}, {}]",
        context_desc,
        value,
        min,
        max,
    );
}

/// Validate allocate response: length matches buckets, sum equals total,
/// each bucket >= its per-bucket minimum.
fn validate_allocation(
    alloc: &[u64],
    buckets_len: usize,
    total: u64,
    per_bucket_mins: &[u64],
    context_desc: &str,
) {
    assert_eq!(
        alloc.len(),
        buckets_len,
        "ask_{}: DP returned {} allocations but {} buckets provided",
        context_desc,
        alloc.len(),
        buckets_len,
    );

    assert_eq!(
        per_bucket_mins.len(),
        buckets_len,
        "ask_{}: per_bucket_mins length {} != buckets length {}",
        context_desc,
        per_bucket_mins.len(),
        buckets_len,
    );

    let sum: u64 = alloc.iter().sum();
    assert_eq!(
        sum, total,
        "ask_{}: DP allocation sum is {} but total should be {}",
        context_desc, sum, total,
    );

    for (i, &val) in alloc.iter().enumerate() {
        assert!(
            val >= per_bucket_mins[i],
            "ask_{}: DP allocated {} to bucket {} but minimum is {}",
            context_desc,
            val,
            i,
            per_bucket_mins[i],
        );
    }
}

/// Validate choose_ordering response: valid permutation of 0..items_len.
///
/// Checks length, index range, and uniqueness. By the pigeonhole principle,
/// N unique values each in [0, N) IS a permutation of 0..N, so no explicit
/// "sequential" check is needed.
#[allow(dead_code)]
fn validate_ordering(order: &[usize], items_len: usize, context_desc: &str) {
    assert_eq!(
        order.len(),
        items_len,
        "ask_{}: DP returned {} indices but {} items to order",
        context_desc,
        order.len(),
        items_len,
    );

    let mut seen = vec![false; items_len];
    for &idx in order {
        assert!(
            idx < items_len,
            "ask_{}: DP returned index {} but only {} items",
            context_desc,
            idx,
            items_len,
        );
        assert!(
            !seen[idx],
            "ask_{}: DP returned duplicate index {} in ordering",
            context_desc,
            idx,
        );
        seen[idx] = true;
    }
}

// ===========================================================================
// Priority & Turn Structure
// ===========================================================================

/// Choose what to do when the player has priority.
///
/// The engine enumerates all legal actions and passes them as options.
/// Returns the chosen `PriorityAction`.
pub fn ask_choose_priority_action(
    dp: &dyn GenericDecisionProvider,
    game: &GameState,
    player: PlayerId,
    legal_actions: &[PriorityAction],
) -> PriorityAction {
    let options: Vec<ChoiceOption> = legal_actions
        .iter()
        .map(|a| ChoiceOption::Action(a.clone()))
        .collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::PriorityAction,
    };
    let index = dp.pick_n(game, player, &ctx, &options, (1, 1));
    validate_pick_n(&index, options.len(), (1, 1), "choose_priority_action");
    legal_actions[index[0]].clone()
}

// ===========================================================================
// Combat
// ===========================================================================

/// Choose which creatures to declare as attackers.
/// Returns a list of (attacker_id, attack_target) pairs.
pub fn ask_choose_attackers(
    dp: &dyn GenericDecisionProvider,
    game: &GameState,
    player: PlayerId,
    legal: &[(ObjectId, AttackTarget)],
) -> Vec<(ObjectId, AttackTarget)> {
    if legal.is_empty() {
        return Vec::new();
    }
    let options: Vec<ChoiceOption> = legal
        .iter()
        .map(|(id, t)| ChoiceOption::AttackerTarget(*id, t.clone()))
        .collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::DeclareAttackers,
    };
    let indices = dp.pick_n(game, player, &ctx, &options, (0, legal.len()));
    validate_pick_n(&indices, options.len(), (0, legal.len()), "choose_attackers");
    indices.iter().map(|&i| (legal[i].0, legal[i].1.clone())).collect()
}

/// Choose which creatures to declare as blockers.
/// Returns a list of (blocker_id, attacker_id) pairs.
pub fn ask_choose_blockers(
    dp: &dyn GenericDecisionProvider,
    game: &GameState,
    player: PlayerId,
    legal: &[(ObjectId, ObjectId)],
) -> Vec<(ObjectId, ObjectId)> {
    if legal.is_empty() {
        return Vec::new();
    }
    let options: Vec<ChoiceOption> = legal
        .iter()
        .map(|(blocker, attacker)| ChoiceOption::BlockerAttacker(*blocker, *attacker))
        .collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::DeclareBlockers,
    };
    let indices = dp.pick_n(game, player, &ctx, &options, (0, legal.len()));
    validate_pick_n(&indices, options.len(), (0, legal.len()), "choose_blockers");
    indices.iter().map(|&i| legal[i]).collect()
}

/// Choose how to divide an attacker's combat damage among multiple blockers.
///
/// Uses `allocate` with all-zero per-bucket minimums (2025 rules: no
/// ordering/lethal-first constraint — player freely divides).
pub fn ask_choose_attacker_damage_assignment(
    dp: &dyn GenericDecisionProvider,
    game: &GameState,
    player: PlayerId,
    attacker_id: ObjectId,
    blockers: &[ObjectId],
    power: u64,
) -> Vec<(ObjectId, u64)> {
    let buckets: Vec<ChoiceOption> = blockers
        .iter()
        .map(|id| ChoiceOption::Object(*id))
        .collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::AssignCombatDamage { attacker_id },
    };
    let mins = vec![0u64; buckets.len()];
    let alloc = dp.allocate(game, player, &ctx, power, &buckets, &mins);
    validate_allocation(&alloc, buckets.len(), power, &mins, "choose_attacker_damage_assignment");
    blockers
        .iter()
        .zip(alloc.iter())
        .filter(|(_, dmg)| **dmg > 0)
        .map(|(id, dmg)| (*id, *dmg))
        .collect()
}

/// Choose how to divide a trampling attacker's damage among blockers and
/// the defending player/planeswalker.
///
/// `per_blocker_mins[i]` is the minimum damage that must be assigned to
/// blocker i (1 if deathtouch, else toughness − damage_marked). The
/// defending target bucket has minimum 0. The engine pre-computes these.
///
/// Returns `(blocker_assignments, overflow_to_defender)`.
pub fn ask_choose_trample_damage_assignment(
    dp: &dyn GenericDecisionProvider,
    game: &GameState,
    player: PlayerId,
    attacker_id: ObjectId,
    blockers: &[ObjectId],
    defending_target: DamageTarget,
    power: u64,
    per_blocker_mins: &[u64],
) -> (Vec<(ObjectId, u64)>, u64) {
    // Buckets: one per blocker + one for the defending target (min 0)
    let mut buckets: Vec<ChoiceOption> = blockers
        .iter()
        .map(|id| ChoiceOption::Object(*id))
        .collect();
    match &defending_target {
        DamageTarget::Player(pid) => buckets.push(ChoiceOption::Player(*pid)),
        DamageTarget::Object(oid) => buckets.push(ChoiceOption::Object(*oid)),
    }

    // Build per-bucket minimums: blocker mins + 0 for the defending target
    let mut mins: Vec<u64> = per_blocker_mins.to_vec();
    mins.push(0); // defending target has no minimum

    let ctx = ChoiceContext {
        kind: ChoiceKind::AssignTrampleDamage {
            attacker_id,
            defending_target: defending_target.clone(),
        },
    };
    let alloc = dp.allocate(game, player, &ctx, power, &buckets, &mins);
    validate_allocation(&alloc, buckets.len(), power, &mins, "choose_trample_damage_assignment");

    let blocker_assignments: Vec<(ObjectId, u64)> = blockers
        .iter()
        .zip(alloc.iter())
        .filter(|(_, dmg)| **dmg > 0)
        .map(|(id, dmg)| (*id, *dmg))
        .collect();
    let overflow = alloc[blockers.len()];

    (blocker_assignments, overflow)
}

// ===========================================================================
// Casting Pipeline (601.2)
// ===========================================================================

/// Choose the value of X for a spell with {X} in its mana cost.
///
/// The DP sees `(min=0, max=u64::MAX)` — affordability is NOT checked here.
/// If the player picks an X they can't afford, the casting pipeline rolls
/// back the entire cast attempt at payment time (601.2h). Each DP is free
/// to use game state to self-limit (e.g. Random DP checks pool + sources).
pub fn ask_choose_x_value(
    dp: &dyn GenericDecisionProvider,
    game: &GameState,
    player: PlayerId,
    spell_id: ObjectId,
    x_count: u64,
) -> u64 {
    let ctx = ChoiceContext {
        kind: ChoiceKind::ChooseXValue { spell_id, x_count },
    };
    let value = dp.pick_number(game, player, &ctx, 0, u64::MAX);
    // Only reject truly impossible values (negative handled by u64 type).
    // Affordability is enforced by rollback, not here.
    value
}

/// Choose an alternative cost (rule 118.9).
/// Returns `None` for normal cost, or `Some(index)` for chosen alt cost.
pub fn ask_choose_alternative_cost(
    dp: &dyn GenericDecisionProvider,
    game: &GameState,
    player: PlayerId,
    available: &[AlternativeCost],
) -> Option<usize> {
    if available.is_empty() {
        return None;
    }
    // Options: index 0 = "pay normal cost", indices 1..=N = alt costs
    let mut options: Vec<ChoiceOption> = vec![ChoiceOption::NormalCost];
    for cost in available.iter() {
        options.push(ChoiceOption::AlternativeCost(cost.clone()));
    }
    let ctx = ChoiceContext {
        kind: ChoiceKind::ChooseAlternativeCost,
    };
    let index = dp.pick_n(game, player, &ctx, &options, (1, 1));
    validate_pick_n(&index, options.len(), (1, 1), "choose_alternative_cost");
    let chosen = index[0];
    if chosen == 0 {
        None
    } else {
        Some(chosen - 1)
    }
}

/// Choose which additional costs to pay (rule 118.8).
/// Returns indices into `available`.
pub fn ask_choose_additional_costs(
    dp: &dyn GenericDecisionProvider,
    game: &GameState,
    player: PlayerId,
    available: &[AdditionalCost],
) -> Vec<usize> {
    if available.is_empty() {
        return Vec::new();
    }
    let options: Vec<ChoiceOption> = available
        .iter()
        .map(|cost| ChoiceOption::AdditionalCost(cost.clone()))
        .collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::ChooseAdditionalCosts,
    };
    let indices = dp.pick_n(game, player, &ctx, &options, (0, available.len()));
    validate_pick_n(
        &indices,
        options.len(),
        (0, available.len()),
        "choose_additional_costs",
    );
    indices
}

/// Select recipients for an effect (covers both MTG "target" and non-targeting "choose").
///
/// `legal_selections` contains every legal recipient (objects AND players).
/// Returns the chosen `ResolvedTarget`s.
pub fn ask_select_recipients(
    dp: &dyn GenericDecisionProvider,
    game: &GameState,
    player: PlayerId,
    recipient: &EffectRecipient,
    spell_id: ObjectId,
    legal_selections: &[ResolvedTarget],
    min_selections: usize,
    max_selections: usize,
) -> Vec<ResolvedTarget> {
    if legal_selections.is_empty() {
        return Vec::new();
    }
    let options: Vec<ChoiceOption> = legal_selections
        .iter()
        .map(|t| match t {
            ResolvedTarget::Object(id) => ChoiceOption::Object(*id),
            ResolvedTarget::Player(id) => ChoiceOption::Player(*id),
        })
        .collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::SelectRecipients {
            recipient: recipient.clone(),
            spell_id,
        },
    };
    let indices = dp.pick_n(game, player, &ctx, &options, (min_selections, max_selections));
    validate_pick_n(
        &indices,
        options.len(),
        (min_selections, max_selections),
        "select_recipients",
    );
    indices.iter().map(|&i| legal_selections[i]).collect()
}

/// Choose how to allocate mana from the pool to pay generic mana.
/// Returns a map of ManaType → amount.
pub fn ask_choose_generic_mana_allocation(
    dp: &dyn GenericDecisionProvider,
    game: &GameState,
    player: PlayerId,
    mana_cost: &ManaCost,
    available_types: &[(ManaType, u64)],
    generic_count: u64,
) -> HashMap<ManaType, u64> {
    if generic_count == 0 || available_types.is_empty() {
        return HashMap::new();
    }
    let buckets: Vec<ChoiceOption> = available_types
        .iter()
        .map(|(mt, _)| ChoiceOption::ManaType(*mt))
        .collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::GenericManaAllocation {
            mana_cost: mana_cost.clone(),
        },
    };
    let mins = vec![0u64; buckets.len()];
    let alloc = dp.allocate(game, player, &ctx, generic_count, &buckets, &mins);
    validate_allocation(
        &alloc,
        buckets.len(),
        generic_count,
        &mins,
        "choose_generic_mana_allocation",
    );

    // Validate each allocation doesn't exceed the available amount
    for (i, amount) in alloc.iter().enumerate() {
        let (mt, available) = available_types[i];
        assert!(
            *amount <= available,
            "ask_choose_generic_mana_allocation: allocated {} of {:?} but only {} available",
            *amount,
            mt,
            available,
        );
    }

    available_types
        .iter()
        .zip(alloc.iter())
        .filter(|(_, a)| **a > 0)
        .map(|((mt, _), a)| (*mt, *a))
        .collect()
}

// ===========================================================================
// State-Based & Cleanup
// ===========================================================================

/// Choose a card to discard (cleanup step discard-to-hand-size).
/// Returns the ObjectId of the chosen card, or None if hand is empty.
pub fn ask_choose_discard(
    dp: &dyn GenericDecisionProvider,
    game: &GameState,
    player: PlayerId,
    hand: &[ObjectId],
) -> Option<ObjectId> {
    if hand.is_empty() {
        return None;
    }
    let options: Vec<ChoiceOption> = hand.iter().map(|id| ChoiceOption::Object(*id)).collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::DiscardToHandSize,
    };
    let index = dp.pick_n(game, player, &ctx, &options, (1, 1));
    validate_pick_n(&index, options.len(), (1, 1), "choose_discard");
    Some(hand[index[0]])
}

/// Choose which legendary permanent to keep (rule 704.5j legend rule).
pub fn ask_choose_legend_to_keep(
    dp: &dyn GenericDecisionProvider,
    game: &GameState,
    player: PlayerId,
    legend_name: &str,
    legendaries: &[ObjectId],
) -> ObjectId {
    assert!(
        !legendaries.is_empty(),
        "ask_choose_legend_to_keep: no legendaries provided"
    );
    let options: Vec<ChoiceOption> = legendaries
        .iter()
        .map(|id| ChoiceOption::Object(*id))
        .collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::LegendRule {
            legend_name: legend_name.to_string(),
        },
    };
    let index = dp.pick_n(game, player, &ctx, &options, (1, 1));
    validate_pick_n(&index, options.len(), (1, 1), "choose_legend_to_keep");
    legendaries[index[0]]
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::game_state::GameState;
    use crate::ui::decision::GenericScriptedDecisionProvider;

    fn test_game_state() -> GameState {
        GameState::new(2, 20)
    }

    // --- pick_n basic ---

    #[test]
    fn test_pick_n_returns_correct_index() {
        let dp = GenericScriptedDecisionProvider::new();
        let game = test_game_state();
        dp.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);

        let actions = vec![
            PriorityAction::Pass,
            PriorityAction::PlayLand(crate::types::ids::new_object_id()),
        ];
        let result = ask_choose_priority_action(&dp, &game, 0, &actions);
        assert!(matches!(result, PriorityAction::PlayLand(_)));
    }

    // --- pick_number basic ---

    #[test]
    fn test_pick_number_returns_value() {
        let dp = GenericScriptedDecisionProvider::new();
        let game = test_game_state();
        let spell_id = crate::types::ids::new_object_id();
        dp.expect_number(
            ChoiceKind::ChooseXValue { spell_id, x_count: 1 },
            5,
        );
        let result = ask_choose_x_value(&dp, &game, 0, spell_id, 1);
        assert_eq!(result, 5);
    }

    // --- allocate basic ---

    #[test]
    fn test_allocate_returns_distribution() {
        let dp = GenericScriptedDecisionProvider::new();
        let game = test_game_state();
        let id_a = crate::types::ids::new_object_id();
        let id_b = crate::types::ids::new_object_id();
        dp.expect_allocation(
            ChoiceKind::AssignCombatDamage { attacker_id: id_a },
            vec![2, 1],
        );
        let result = ask_choose_attacker_damage_assignment(&dp, &game, 0, id_a, &[id_a, id_b], 3);
        assert_eq!(result, vec![(id_a, 2), (id_b, 1)]);
    }

    // --- allocate with per-bucket minimums (trample) ---

    #[test]
    fn test_trample_allocation_respects_per_bucket_mins() {
        let dp = GenericScriptedDecisionProvider::new();
        let game = test_game_state();
        let attacker = crate::types::ids::new_object_id();
        let blocker_a = crate::types::ids::new_object_id();
        let blocker_b = crate::types::ids::new_object_id();
        // 5 power, blocker A needs 2 lethal, blocker B needs 1 lethal
        // DP assigns: 2 to A, 1 to B, 2 overflow to player
        dp.expect_allocation(
            ChoiceKind::AssignTrampleDamage {
                attacker_id: attacker,
                defending_target: DamageTarget::Player(1),
            },
            vec![2, 1, 2],
        );
        let (blockers, overflow) = ask_choose_trample_damage_assignment(
            &dp, &game, 0, attacker,
            &[blocker_a, blocker_b],
            DamageTarget::Player(1),
            5,
            &[2, 1], // per-blocker lethal minimums
        );
        assert_eq!(blockers, vec![(blocker_a, 2), (blocker_b, 1)]);
        assert_eq!(overflow, 2);
    }

    #[test]
    #[should_panic(expected = "DP allocated 1 to bucket 0 but minimum is 2")]
    fn test_trample_rejects_below_per_bucket_min() {
        let dp = GenericScriptedDecisionProvider::new();
        let game = test_game_state();
        let attacker = crate::types::ids::new_object_id();
        let blocker = crate::types::ids::new_object_id();
        // DP violates: assigns only 1 to blocker that needs 2 lethal
        dp.expect_allocation(
            ChoiceKind::AssignTrampleDamage {
                attacker_id: attacker,
                defending_target: DamageTarget::Player(1),
            },
            vec![1, 4],
        );
        let _ = ask_choose_trample_damage_assignment(
            &dp, &game, 0, attacker,
            &[blocker],
            DamageTarget::Player(1),
            5,
            &[2], // blocker needs at least 2
        );
    }

    // TODO: Add choose_ordering test when the first effect that needs it
    // (scry, stack ordering, etc.) is implemented.

    // --- ask_choose_attackers roundtrip ---

    #[test]
    fn test_ask_choose_attackers_roundtrip() {
        let dp = GenericScriptedDecisionProvider::new();
        let game = test_game_state();
        let id_a = crate::types::ids::new_object_id();
        let id_b = crate::types::ids::new_object_id();

        dp.expect_pick_n(ChoiceKind::DeclareAttackers, vec![0, 1]);
        let legal = vec![
            (id_a, AttackTarget::Player(1)),
            (id_b, AttackTarget::Player(1)),
        ];
        let result = ask_choose_attackers(&dp, &game, 0, &legal);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, id_a);
        assert_eq!(result[1].0, id_b);
    }

    // --- ask_choose_x_value roundtrip ---

    #[test]
    fn test_ask_choose_x_value_roundtrip() {
        let dp = GenericScriptedDecisionProvider::new();
        let game = test_game_state();
        let spell_id = crate::types::ids::new_object_id();
        dp.expect_number(
            ChoiceKind::ChooseXValue { spell_id, x_count: 1 },
            3,
        );
        let result = ask_choose_x_value(&dp, &game, 0, spell_id, 1);
        assert_eq!(result, 3);
    }

    // --- Validation tests ---

    #[test]
    #[should_panic(expected = "DP returned index 5 but only 2 options available")]
    fn test_validation_rejects_out_of_bounds() {
        let dp = GenericScriptedDecisionProvider::new();
        let game = test_game_state();
        dp.expect_pick_n(ChoiceKind::PriorityAction, vec![5]);
        let actions = vec![PriorityAction::Pass, PriorityAction::Pass];
        let _ = ask_choose_priority_action(&dp, &game, 0, &actions);
    }

    #[test]
    #[should_panic(expected = "DP returned 2 selections, expected 1-1")]
    fn test_validation_rejects_wrong_count() {
        let dp = GenericScriptedDecisionProvider::new();
        let game = test_game_state();
        dp.expect_pick_n(ChoiceKind::PriorityAction, vec![0, 1]);
        let actions = vec![PriorityAction::Pass, PriorityAction::Pass];
        let _ = ask_choose_priority_action(&dp, &game, 0, &actions);
    }

    #[test]
    #[should_panic(expected = "allocation sum is 4 but total should be 3")]
    fn test_validation_rejects_bad_allocation_sum() {
        let dp = GenericScriptedDecisionProvider::new();
        let game = test_game_state();
        let id_a = crate::types::ids::new_object_id();
        let id_b = crate::types::ids::new_object_id();
        dp.expect_allocation(
            ChoiceKind::AssignCombatDamage { attacker_id: id_a },
            vec![2, 2],
        );
        let _ = ask_choose_attacker_damage_assignment(&dp, &game, 0, id_a, &[id_a, id_b], 3);
    }

    // --- ScriptedDP kind mismatch ---

    #[test]
    #[should_panic(expected = "kind mismatch")]
    fn test_scripted_wrong_kind_panics() {
        let dp = GenericScriptedDecisionProvider::new();
        let game = test_game_state();
        dp.expect_pick_n(ChoiceKind::DeclareAttackers, vec![0]);
        // Engine asks for PriorityAction but we expected DeclareAttackers
        let actions = vec![PriorityAction::Pass];
        let _ = ask_choose_priority_action(&dp, &game, 0, &actions);
    }

    // --- ScriptedDP unconsumed expectations ---

    #[test]
    #[should_panic(expected = "unconsumed expectation")]
    fn test_scripted_unconsumed_panics() {
        let dp = GenericScriptedDecisionProvider::new();
        dp.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
        dp.expect_pick_n(ChoiceKind::DeclareAttackers, vec![0]);
        // Only consume one
        let game = test_game_state();
        let actions = vec![PriorityAction::Pass];
        let _ = ask_choose_priority_action(&dp, &game, 0, &actions);
        // dp drops here with 1 remaining expectation → panic
    }

    // --- ScriptedDP empty queue ---

    #[test]
    #[should_panic(expected = "no scripted response in queue")]
    fn test_scripted_empty_queue_panics() {
        let dp = GenericScriptedDecisionProvider::new();
        let game = test_game_state();
        let actions = vec![PriorityAction::Pass];
        let _ = ask_choose_priority_action(&dp, &game, 0, &actions);
    }
}
