//! Typed `ask_*` free functions — the engine-side bridge to the 4-primitive
//! `DecisionProvider` trait.
//!
//! Each function:
//! 1. Constructs a `ChoiceContext` with the appropriate `ChoiceKind`
//! 2. Packs options into `Vec<ChoiceOption>`
//! 3. Calls the appropriate DP primitive (`pick_n`, `pick_number`, `allocate`, `choose_ordering`)
//! 4. Validates the response (bounds, count, sum, permutation)
//! 5. Unpacks indices back into typed results
//!
//! Engine call sites and tests use these exclusively — never raw DP methods.
//!
//! **A prompt with one legal answer is not asked** (CR 102.2): `ask_discard`
//! returns the hand when the count takes all of it, `order_scry_group` returns
//! below two cards, `ask_select_recipients` returns a fixed count that takes
//! none or all, `forced_allocation` answers a split with one place to put the
//! remainder, and five asks assert two or more candidates and leave the single
//! case to the caller. Out of process each of those would be a round trip for
//! an answer the engine already has — and it would spend CPU against the
//! ratchet's numerator without producing a decision to count
//! (`codebase-state.md` item 145).
//!
//! The four `validate_*` helpers also carry `codebase-state.md` item 138's
//! **decision** count — a prompt with two or more legal answers, the unit
//! `engineering-practices.md` §3.1's ratchet reads CPU in. They are where it
//! belongs rather than in the 25 bodies: each runs exactly once per prompt,
//! with the candidate list and the bounds both in scope, so "two or more
//! answers" is decided once per primitive rather than once per caller.

use std::collections::HashMap;

use crate::engine::resolve::ResolvedTarget;
use crate::events::event::DamageTarget;
use crate::state::battlefield::AttackTarget;
use crate::state::diagnostics::EngineCounters;
use crate::state::game_state::GameState;
use crate::types::costs::{AdditionalCost, AlternativeCost};
use crate::types::effects::EffectRecipient;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};

use super::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use super::decision::{DecisionProvider, PriorityAction};

/// CR 102.2 — the one allocation a forced split admits, or `None` when the
/// player has something to decide.
///
/// A split is forced in exactly three shapes: nothing left once every bucket
/// holds its minimum, nothing spare once every bucket holds its maximum, and
/// one bucket free to take whatever is left. Everything else admits two
/// allocations that differ by one unit moved between two free buckets.
///
/// **Both a guard and a counter.** The `ask_*` bodies call it to answer
/// without a prompt — a round trip for an answer the engine already has costs
/// the ratchet its numerator (`codebase-state.md` item 145) — and
/// [`validate_allocation`] calls it so the decision count cannot drift from
/// what the callers skip.
///
/// **`None` is also every answer that is not well formed**, and that is load
/// bearing rather than defensive: an allocation this returns is used *without*
/// [`validate_allocation`] ever running, so anything it answers wrongly is
/// silent. A `None` falls through to the prompt and the validator, which is
/// where the loud message has always been — a caller whose minimums exceed the
/// total, whose one free bucket cannot hold the remainder, or whose bounds
/// disagree with each other gets the panic it used to get. The generic split's
/// `can_pay` precondition is a `debug_assert`, so in a release fuzz run this is
/// the only thing standing between a caller bug and an unpayable split that
/// `ManaPool::pay` refuses and CR 601.2 rewinds — `codebase-state.md` 16c's
/// failure, which this guard must not reintroduce.
fn forced_allocation(
    total: u64,
    per_bucket_mins: &[u64],
    per_bucket_maxs: Option<&[u64]>,
) -> Option<Vec<u64>> {
    if let Some(maxs) = per_bucket_maxs
        && (maxs.len() != per_bucket_mins.len()
            || per_bucket_mins.iter().zip(maxs).any(|(lo, hi)| lo > hi))
    {
        return None;
    }
    // More owed to the minimums than there is to give: infeasible, and the sum
    // assertion names it.
    let slack = total.checked_sub(per_bucket_mins.iter().sum())?;
    if slack == 0 {
        return Some(per_bucket_mins.to_vec());
    }
    let headroom = |i: usize| {
        per_bucket_maxs.map_or(u64::MAX, |maxs| maxs[i].saturating_sub(per_bucket_mins[i]))
    };
    let free: Vec<usize> = (0..per_bucket_mins.len()).filter(|&i| headroom(i) > 0).collect();
    if free.len() == 1 {
        // Only when it actually fits; a bucket asked for more than its maximum
        // is the caller's bug and belongs in the validator's message.
        if headroom(free[0]) < slack {
            return None;
        }
        let mut alloc = per_bucket_mins.to_vec();
        alloc[free[0]] += slack;
        return Some(alloc);
    }
    // Saturating: the trample split's unbounded buckets are `u64::MAX`, and a
    // plain sum of two of them overflows before it can fail the comparison.
    if let Some(maxs) = per_bucket_maxs
        && free.iter().map(|&i| headroom(i)).fold(0u64, u64::saturating_add) == slack
    {
        return Some(maxs.to_vec());
    }
    None
}

// ===========================================================================
// Validation helpers
// ===========================================================================

/// Validate pick_n response: indices in range, no duplicates, count in bounds.
fn validate_pick_n(
    indices: &[usize],
    options_len: usize,
    bounds: (usize, usize),
    context_desc: &str,
    counters: &EngineCounters,
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

    // Item 138's decision count. A `pick_n` has one legal answer only when the
    // count is fixed and that count admits one combination — take none, or
    // take every option — so a lone candidate offered as "take it or not" is a
    // decision and a forced list is not, whichever way the caller spelled it.
    if !(bounds.0 == bounds.1 && (bounds.0 == 0 || bounds.0 == options_len)) {
        counters.record_decision();
    }
}

/// Validate pick_number response: value in range.
///
/// When `GameNumber` replaces `u64` for symbolic values (`backlog.md` §2.28,
/// the loop-shortcut capture), this becomes `GameNumber::gte`/`lte` rather
/// than an integer comparison.
fn validate_pick_number(
    value: u64,
    min: u64,
    max: u64,
    context_desc: &str,
    counters: &EngineCounters,
) {
    assert!(
        value >= min && value <= max,
        "ask_{}: DP returned {} but range is [{}, {}]",
        context_desc,
        value,
        min,
        max,
    );

    // Item 138's decision count: one number to name is no decision.
    if max > min {
        counters.record_decision();
    }
}

/// Validate allocate response: length matches buckets, sum equals total,
/// each bucket >= its per-bucket minimum and <= its per-bucket maximum.
fn validate_allocation(
    alloc: &[u64],
    buckets_len: usize,
    total: u64,
    per_bucket_mins: &[u64],
    per_bucket_maxs: Option<&[u64]>,
    context_desc: &str,
    counters: &EngineCounters,
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

    if let Some(maxs) = per_bucket_maxs {
        assert_eq!(
            maxs.len(),
            buckets_len,
            "ask_{}: per_bucket_maxs length {} != buckets length {}",
            context_desc,
            maxs.len(),
            buckets_len,
        );
    }

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
        if let Some(maxs) = per_bucket_maxs {
            assert!(
                val <= maxs[i],
                "ask_{}: DP allocated {} to bucket {} but maximum is {}",
                context_desc,
                val,
                i,
                maxs[i],
            );
        }
    }

    // Item 138's decision count, off the same predicate the callers skip on —
    // a prompt that reached here at all had two or more legal allocations.
    if forced_allocation(total, per_bucket_mins, per_bucket_maxs).is_none() {
        counters.record_decision();
    }
}

/// Validate choose_ordering response: valid permutation of 0..items_len.
///
/// Checks length, index range, and uniqueness. By the pigeonhole principle,
/// N unique values each in [0, N) IS a permutation of 0..N, so no explicit
/// "sequential" check is needed.
fn validate_ordering(
    order: &[usize],
    items_len: usize,
    context_desc: &str,
    counters: &EngineCounters,
) {
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

    // Item 138's decision count: one item has one order.
    if items_len >= 2 {
        counters.record_decision();
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
    dp: &dyn DecisionProvider,
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
    validate_pick_n(&index, options.len(), (1, 1), "choose_priority_action", &game.counters);
    // Item 138's split of the count above. `Pass` is always offered
    // (`engine::priority`), so a longer list is a seat with something else to
    // do — and the priority prompts that are `[Pass]` alone, 91.5% of them at
    // four seats, are what a prompt count would have measured instead.
    if legal_actions.len() > 1 {
        game.counters.record_priority_decision();
    }
    legal_actions[index[0]].clone()
}

// ===========================================================================
// Combat
// ===========================================================================

/// Choose which creatures to declare as attackers.
/// Returns a list of (attacker_id, attack_target) pairs.
pub fn ask_choose_attackers(
    dp: &dyn DecisionProvider,
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
    validate_pick_n(&indices, options.len(), (0, legal.len()), "choose_attackers", &game.counters);
    indices.iter().map(|&i| (legal[i].0, legal[i].1.clone())).collect()
}

/// Choose which creatures to declare as blockers.
/// Returns a list of (blocker_id, attacker_id) pairs.
pub fn ask_choose_blockers(
    dp: &dyn DecisionProvider,
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
    validate_pick_n(&indices, options.len(), (0, legal.len()), "choose_blockers", &game.counters);
    indices.iter().map(|&i| legal[i]).collect()
}

/// Choose how to divide an attacker's combat damage among multiple blockers.
///
/// Uses `allocate` with all-zero per-bucket minimums (2025 rules: no
/// ordering/lethal-first constraint — player freely divides).
pub fn ask_choose_attacker_damage_assignment(
    dp: &dyn DecisionProvider,
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
    let alloc = forced_allocation(power, &mins, None).unwrap_or_else(|| {
        let alloc = dp.allocate(game, player, &ctx, power, &buckets, &mins, None);
        validate_allocation(&alloc, buckets.len(), power, &mins, None, "choose_attacker_damage_assignment", &game.counters);
        alloc
    });
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
    dp: &dyn DecisionProvider,
    game: &GameState,
    player: PlayerId,
    attacker_id: ObjectId,
    blockers: &[ObjectId],
    defending_target: DamageTarget,
    power: u64,
    per_blocker_mins: &[u64],
    per_bucket_maxs: Option<&[u64]>,
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
            defending_target,
        },
    };
    let alloc = forced_allocation(power, &mins, per_bucket_maxs).unwrap_or_else(|| {
        let alloc = dp.allocate(game, player, &ctx, power, &buckets, &mins, per_bucket_maxs);
        validate_allocation(&alloc, buckets.len(), power, &mins, per_bucket_maxs, "choose_trample_damage_assignment", &game.counters);
        alloc
    });

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
    dp: &dyn DecisionProvider,
    game: &GameState,
    player: PlayerId,
    spell_id: ObjectId,
    x_count: u64,
) -> u64 {
    let ctx = ChoiceContext {
        kind: ChoiceKind::ChooseXValue { spell_id, x_count },
    };
    let value = dp.pick_number(game, player, &ctx, 0, u64::MAX);
    // Contract check: value must be in [0, u64::MAX] — tautological for u64, but
    // keeps the validate_* pattern wired in so fuzz harness exercises it. Affordability
    // is enforced by the casting pipeline rollback (601.2h), not here.
    validate_pick_number(value, 0, u64::MAX, "choose_x_value", &game.counters);
    value
}

/// Choose an alternative cost (rule 118.9).
/// Returns `None` for normal cost, or `Some(index)` for chosen alt cost.
pub fn ask_choose_alternative_cost(
    dp: &dyn DecisionProvider,
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
    validate_pick_n(&index, options.len(), (1, 1), "choose_alternative_cost", &game.counters);
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
    dp: &dyn DecisionProvider,
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
        &game.counters,
    );
    indices
}

/// Select recipients for an effect (covers both MTG "target" and non-targeting "choose").
///
/// `legal_selections` contains every legal recipient (objects AND players).
/// Returns the chosen `ResolvedTarget`s.
pub fn ask_select_recipients(
    dp: &dyn DecisionProvider,
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
    // CR 102.2 — a fixed count that takes none of the legal recipients or all
    // of them is not a choice, and one legal target for "target creature" is
    // the common shape of it. Returned in `legal_selections` order for
    // `ask_discard`'s reason: no rule gives the player that order, and the
    // choosers agreeing is worth more than an order nobody named.
    if min_selections == max_selections {
        if min_selections == 0 {
            return Vec::new();
        }
        if min_selections == legal_selections.len() {
            return legal_selections.to_vec();
        }
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
        &game.counters,
    );
    indices.iter().map(|&i| legal_selections[i]).collect()
}

/// Prompt the casting/activating player for a mana ability to activate
/// inside the 601.2g / 602.1b mana-ability window.
///
/// Bounds are `(0, 1)`: picking zero options means "stop activating" (the
/// engine exits the loop; `pay_costs` will run with the current pool and
/// fail if insufficient — caller must then roll back). Picking one option
/// means "activate this ability"; the engine applies it and re-enters the
/// loop to re-check whether the pool now covers the cost.
///
/// Options are presented as `ChoiceOption::Action(PriorityAction::ActivateAbility(...))`
/// so DPs that already pattern-match on `PriorityAction` reuse the same
/// inspection path. The `ChoiceContext` carries the `spell_or_ability_id`
/// being paid for and the `remaining_cost` after pool — middleware DPs
/// (future AutoTapDP) use that to plan tap sequences.
pub fn ask_activate_mana_ability(
    dp: &dyn DecisionProvider,
    game: &GameState,
    player: PlayerId,
    spell_or_ability_id: ObjectId,
    remaining_cost: &ManaCost,
    legal: &[(ObjectId, AbilityId)],
) -> Option<(ObjectId, AbilityId)> {
    if legal.is_empty() {
        return None;
    }
    let options: Vec<ChoiceOption> = legal
        .iter()
        .map(|(perm_id, ab_id)| {
            ChoiceOption::Action(PriorityAction::ActivateAbility(*perm_id, *ab_id))
        })
        .collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::ManaAbilityWindow {
            spell_or_ability_id,
            remaining_cost: remaining_cost.clone(),
        },
    };
    // (0, 1): 0 = decline / stop, 1 = activate one ability
    let indices = dp.pick_n(game, player, &ctx, &options, (0, 1));
    validate_pick_n(&indices, options.len(), (0, 1), "activate_mana_ability", &game.counters);
    if indices.is_empty() {
        None
    } else {
        Some(legal[indices[0]])
    }
}

/// CR 601.2f — the order in which two or more cost reductions apply.
///
/// `sources` are the reductions' sources in battlefield timestamp order, so
/// the permutation returned means the same thing in every process. One
/// reduction has no order to choose and must not reach here.
pub fn ask_order_cost_reductions(
    dp: &dyn DecisionProvider,
    game: &GameState,
    player: PlayerId,
    spell_id: ObjectId,
    sources: &[ObjectId],
) -> Vec<usize> {
    debug_assert!(sources.len() >= 2, "CR 601.2f: one reduction has no order to choose");
    let options: Vec<ChoiceOption> = sources.iter().map(|id| ChoiceOption::Object(*id)).collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::OrderCostReductions { spell_id },
    };
    let order = dp.choose_ordering(game, player, &ctx, &options);
    validate_ordering(&order, options.len(), "order_cost_reductions", &game.counters);
    order
}

/// How many of `mana_cost`'s symbols must be paid with `mana_type` specifically.
///
/// `ManaCost::build` writes `{C}` as `ManaSymbol::Colorless`, but
/// `from_symbols` admits `Colored(ManaType::Colorless)` for the same pip, so
/// both spellings count.
fn pips_owed(mana_cost: &ManaCost, mana_type: ManaType) -> u64 {
    mana_cost
        .symbols
        .iter()
        .filter(|s| match s {
            ManaSymbol::Colored(t) => *t == mana_type,
            ManaSymbol::Colorless => mana_type == ManaType::Colorless,
            _ => false,
        })
        .count() as u64
}

/// Choose how to allocate mana from the pool to pay generic mana.
/// Returns a map of ManaType → amount.
///
/// Each bucket's maximum is `available − pips of that type`, not the pool's
/// amount: the generic part is paid out of the same pool the specific pips are
/// (CR 601.2h), so a bucket capped at the pool's amount lets a DP name a split
/// that `ManaPool::pay` then refuses, and CR 601.2 rewinds the whole cast for
/// it. Every other `ask_*` offers only legal choices, and a DP should not have
/// to know payment law to answer "which mana" — `codebase-state.md` 16c,
/// `backlog.md` §2.18.
///
/// Only `Colored`, `Colorless` and `Generic` symbols reach here — `can_pay`
/// refuses a cost containing any other — so no mode question arises. When
/// hybrid payment lands, a twobrid or hybrid whose chosen mode is a colored
/// half is one more pip of that type and folds into the same tally.
pub fn ask_choose_generic_mana_allocation(
    dp: &dyn DecisionProvider,
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
    let maxs: Vec<u64> = available_types
        .iter()
        .map(|(mt, amt)| amt.saturating_sub(pips_owed(mana_cost, *mt)))
        .collect();
    // `can_pay` has passed by the time this is asked, and it checked both
    // halves of the same inequality: every type covers its own pips, and what
    // is left over covers the generic count. So the clamped maxima always sum
    // to at least `generic_count` and a feasible answer exists — asserted
    // rather than trusted, because a caller that skipped the check would
    // otherwise reach `validate_allocation` with no legal answer to give.
    debug_assert!(
        maxs.iter().sum::<u64>() >= generic_count,
        "generic split of {}: clamped maxima {:?} cannot reach {} — caller skipped can_pay",
        mana_cost,
        maxs,
        generic_count,
    );
    let alloc = forced_allocation(generic_count, &mins, Some(&maxs)).unwrap_or_else(|| {
        let alloc = dp.allocate(game, player, &ctx, generic_count, &buckets, &mins, Some(&maxs));
        validate_allocation(
            &alloc,
            buckets.len(),
            generic_count,
            &mins,
            Some(&maxs),
            "choose_generic_mana_allocation",
            &game.counters,
        );
        alloc
    });

    // per_bucket_maxs already enforces that each allocation leaves every pip
    // its own mana — no post-hoc check needed.

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

/// CR 704.6d / 903.9a — may this commander go to the command zone?
///
/// A **state-based action**, not a replacement effect: current Oracle splits
/// CR 903.9, and only 903.9b (hand or library) is a replacement, so the
/// graveyard half never needed the replacement pipeline at all.
pub fn ask_commander_to_command_zone(
    dp: &dyn DecisionProvider,
    game: &GameState,
    owner: PlayerId,
    commander: ObjectId,
) -> bool {
    let options = vec![ChoiceOption::Object(commander)];
    let ctx = ChoiceContext {
        kind: ChoiceKind::CommanderToCommandZoneSba { commander },
    };
    let picked = dp.pick_n(game, owner, &ctx, &options, (0, 1));
    validate_pick_n(&picked, options.len(), (0, 1), "commander_to_command_zone", &game.counters);
    !picked.is_empty()
}

/// CR 701.9b — which cards the affected player discards.
///
/// > 701.9b By default, effects that cause a player to discard a card allow the
/// > affected player to choose which card to discard.
///
/// **One helper for both producers**, CR 514.1's turn-based action and a
/// resolving effect's "discards two cards": `source` is the whole difference
/// and it is only on the prompt. The other two shapes 701.9b names do not come
/// through here — "at random" reads `GameState::rng` at the performer, and
/// "another player chooses" has no card yet.
///
/// Returns the chosen cards **in hand order**, whatever order the provider
/// picked them in, so the graveyard they land in is ordered the way the hand
/// was. Nothing in CR 701.9 gives the discarding player that order, and the two
/// choosers agreeing is worth more than an order no rule names.
///
/// Two ways nothing is asked, both CR 102.2's "a choice that is forced is not
/// made": an empty hand, and a count that takes the whole hand. The second is
/// what a Mind Rot against a one-card hand is, and it is common.
pub fn ask_discard(
    dp: &dyn DecisionProvider,
    game: &GameState,
    player: PlayerId,
    hand: &[ObjectId],
    count: usize,
    source: Option<ObjectId>,
) -> Vec<ObjectId> {
    // CR 101.3 — an effect does as much as it can, which for a hand shorter
    // than the count is the whole hand.
    let count = count.min(hand.len());
    if count == 0 {
        return Vec::new();
    }
    if count == hand.len() {
        return hand.to_vec();
    }
    let options: Vec<ChoiceOption> = hand.iter().map(|id| ChoiceOption::Object(*id)).collect();
    let ctx = ChoiceContext { kind: ChoiceKind::Discard { source } };
    let mut picked = dp.pick_n(game, player, &ctx, &options, (count, count));
    validate_pick_n(&picked, options.len(), (count, count), "discard", &game.counters);
    picked.sort();
    picked.into_iter().map(|i| hand[i]).collect()
}

/// CR 701.22a — where the cards a scry looked at go.
///
/// > 701.22a To "scry N" means to look at the top N cards of your library, then
/// > put any number of them on the bottom of your library in any order and the
/// > rest on top of your library in any order.
///
/// `looked_at` is the cards, **top-most first**, and there are `looked_at.len()`
/// of them rather than N: a library shorter than the instruction has fewer, and
/// CR 701.22d ("even if some or all of those actions were impossible") is what
/// makes that a scry anyway rather than a failure.
///
/// Returns `(top, bottom)`, each ordered top-most first within its own group,
/// which is what the caller writes back into the library.
///
/// **Up to three prompts, and CR 102.2 removes the ones with one answer.**
/// "Any number of them" is the first, bounds `(0, k)`; "in any order" is the
/// other two, asked only of a group holding two or more cards. So Opt — scry 1
/// — asks exactly once and never orders, which is every scry a registered card
/// makes today.
pub fn ask_scry(
    dp: &dyn DecisionProvider,
    game: &GameState,
    player: PlayerId,
    looked_at: &[ObjectId],
    n: u64,
    source: Option<ObjectId>,
) -> (Vec<ObjectId>, Vec<ObjectId>) {
    // **Not a scry 0** — CR 701.22b makes that no event at all, and
    // `replacement::never_happens` drops it before any performer runs. This is
    // a scry of one or more against an **empty library**, which CR 701.22d
    // says still happens ("even if some or all of those actions were
    // impossible"): the event is announced, and there is simply nothing to
    // ask about. Silent for CR 102.2's reason, the same one that skips the
    // ordering prompts below.
    if looked_at.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let options: Vec<ChoiceOption> =
        looked_at.iter().map(|id| ChoiceOption::Object(*id)).collect();
    let bounds = (0, looked_at.len());
    let ctx = ChoiceContext { kind: ChoiceKind::Scry { source, n } };
    let to_bottom = dp.pick_n(game, player, &ctx, &options, bounds);
    validate_pick_n(&to_bottom, options.len(), bounds, "scry", &game.counters);

    let mut bottom: Vec<ObjectId> = Vec::with_capacity(to_bottom.len());
    let mut top: Vec<ObjectId> = Vec::with_capacity(looked_at.len() - to_bottom.len());
    for (i, id) in looked_at.iter().enumerate() {
        if to_bottom.contains(&i) {
            bottom.push(*id);
        } else {
            top.push(*id);
        }
    }
    (order_scry_group(dp, game, player, top, source, false),
     order_scry_group(dp, game, player, bottom, source, true))
}

/// CR 701.22a's "in any order", for one of [`ask_scry`]'s two groups.
fn order_scry_group(
    dp: &dyn DecisionProvider,
    game: &GameState,
    player: PlayerId,
    group: Vec<ObjectId>,
    source: Option<ObjectId>,
    bottom: bool,
) -> Vec<ObjectId> {
    if group.len() < 2 {
        return group;
    }
    let options: Vec<ChoiceOption> = group.iter().map(|id| ChoiceOption::Object(*id)).collect();
    let ctx = ChoiceContext { kind: ChoiceKind::ScryOrder { source, bottom } };
    let order = dp.choose_ordering(game, player, &ctx, &options);
    validate_ordering(&order, options.len(), "scry_order", &game.counters);
    order.into_iter().map(|i| group[i]).collect()
}

// ===========================================================================
// Replacement effects (CR 616.1)
// ===========================================================================

/// Choose which of several applicable replacement or prevention effects to
/// apply (CR 616.1).
///
/// **The caller must not call this with fewer than two candidates.** CR 616.1
/// only makes a choice when "two or more ... are attempting to modify the way
/// an event affects an object or player", and the engine-side consequence is
/// larger than the rule: every existing `ScriptedDecisionProvider` test now
/// reaches the pipeline, and the one-candidate short circuit is what keeps that
/// at zero new prompts.
///
/// The options are the candidates' **source objects**, which is lossy when one
/// permanent has two applicable replacement abilities — the index is what
/// selects, and the source is what a UI has to render. A richer option would
/// need `ChoiceOption` to carry rules text, and no card in the pool needs it
/// yet.
pub fn ask_choose_replacement(
    dp: &dyn DecisionProvider,
    game: &GameState,
    chooser: PlayerId,
    affected_object: Option<ObjectId>,
    sources: &[ObjectId],
) -> usize {
    game.counters.record_replacement_prompt();
    assert!(
        sources.len() >= 2,
        "ask_choose_replacement: CR 616.1 makes a choice only among two or more \
         applicable effects; called with {}",
        sources.len(),
    );
    let options: Vec<ChoiceOption> = sources.iter().map(|s| ChoiceOption::Object(*s)).collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::ChooseReplacementEffect { affected_object },
    };
    let index = dp.pick_n(game, chooser, &ctx, &options, (1, 1));
    validate_pick_n(&index, options.len(), (1, 1), "choose_replacement", &game.counters);
    index[0]
}

/// CR 615.7 — which of several simultaneous sources' damage a "prevent the
/// next N damage" effect prevents.
///
/// `buckets` are `(damage source, amount)` in batch order; the answer is one
/// share per bucket, each at most that source's amount, summing to the smaller
/// of `remaining` and the damage on offer. **Two or more buckets, or nothing to
/// ask** — the caller handles one source by preventing `min(remaining,
/// amount)` unasked, and the assertion is the same one `ask_choose_replacement`
/// makes about CR 616.1.
pub fn ask_allocate_next_damage(
    dp: &dyn DecisionProvider,
    game: &GameState,
    chooser: PlayerId,
    source: ObjectId,
    remaining: u64,
    buckets: &[(ObjectId, u64)],
) -> Vec<u64> {
    assert!(
        buckets.len() >= 2,
        "ask_allocate_next_damage: CR 615.7 chooses only among two or more          sources; called with {}",
        buckets.len(),
    );
    let options: Vec<ChoiceOption> =
        buckets.iter().map(|(id, _)| ChoiceOption::Object(*id)).collect();
    let maxs: Vec<u64> = buckets.iter().map(|(_, amount)| *amount).collect();
    let mins = vec![0; buckets.len()];
    let total = remaining.min(maxs.iter().sum());
    let ctx = ChoiceContext {
        kind: ChoiceKind::AllocateNextDamage { source, remaining },
    };
    forced_allocation(total, &mins, Some(&maxs)).unwrap_or_else(|| {
        let alloc = dp.allocate(game, chooser, &ctx, total, &options, &mins, Some(&maxs));
        validate_allocation(&alloc, options.len(), total, &mins, Some(&maxs), "allocate_next_damage", &game.counters);
        alloc
    })
}

/// Ask whether to apply a "you **may** ... instead" replacement effect
/// (CR 614.1a).
///
/// Declining is not free: CR 614.5 gives the effect one opportunity per event,
/// and being offered it *is* the opportunity. The caller marks it applied
/// either way and consumes a use only on acceptance.
pub fn ask_apply_optional_replacement(
    dp: &dyn DecisionProvider,
    game: &GameState,
    chooser: PlayerId,
    affected_object: Option<ObjectId>,
    candidate: &crate::engine::replacement::ReplacementInstance,
) -> bool {
    game.counters.record_replacement_prompt();
    let options = vec![ChoiceOption::Object(candidate.source)];
    let ctx = ChoiceContext {
        kind: ChoiceKind::ApplyOptionalReplacement {
            affected_object,
            source: candidate.source,
        },
    };
    let picked = dp.pick_n(game, chooser, &ctx, &options, (0, 1));
    validate_pick_n(&picked, options.len(), (0, 1), "apply_optional_replacement", &game.counters);
    !picked.is_empty()
}

/// Choose which of several opponents a permanent enters under (CR 616.1b's
/// `Rewrite::EnterUnderControlOf(PlayerRef::Opponent)` — Xantcha's "an opponent
/// of your choice").
///
/// **Two or more candidates, like [`ask_choose_replacement`]**: with one
/// opponent CR 102.2 leaves nothing to choose and the caller does not ask.
/// CR 614.12a puts the choice before the permanent enters, which holds here
/// because the CR 616.1 loop that asks runs ahead of the performer.
pub fn ask_choose_entering_controller(
    dp: &dyn DecisionProvider,
    game: &GameState,
    chooser: PlayerId,
    object: ObjectId,
    candidates: &[PlayerId],
) -> PlayerId {
    assert!(
        candidates.len() >= 2,
        "ask_choose_entering_controller: a choice needs two or more opponents; called with {}",
        candidates.len(),
    );
    let options: Vec<ChoiceOption> = candidates.iter().map(|p| ChoiceOption::Player(*p)).collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::ChooseEnteringController { object },
    };
    let index = dp.pick_n(game, chooser, &ctx, &options, (1, 1));
    validate_pick_n(&index, options.len(), (1, 1), "choose_entering_controller", &game.counters);
    candidates[index[0]]
}

/// CR 707.4 — choose the permanent a copy effect captures its values from.
///
/// **Two or more candidates, always.** With one the choice is forced and the
/// caller takes it without a prompt (CR 102.2's shape, as
/// `ask_choose_entering_controller` uses it). The assert is what keeps that a
/// caller obligation rather than a convention, and it is why registering
/// Cytoshape adds no prompt to any existing scripted test.
/// CR 614.13a — choose the objects an entry replacement also moves.
///
/// > 614.13a While applying an effect that modifies how a permanent enters the
/// > battlefield, you may have to choose a number of objects that will also
/// > change zones.
///
/// **Called with one candidate as well as with many, and that is not the
/// CR 616.1 rule.** `ask_choose_replacement`'s two-candidate floor is about
/// choosing *between effects*, where one candidate leaves nothing to decide.
/// Here a single candidate is still a real choice — take it or not — because
/// the count's floor is zero. It is skipped only when the candidate list is
/// empty, where there is genuinely nothing to ask.
///
/// `max` is the caller's clamp: the smaller of the effect's own bound and the
/// number of candidates, so CR 101.3's "only the possible portion" is applied
/// before the prompt rather than after it.
pub fn ask_choose_auxiliary_zone_change(
    dp: &dyn DecisionProvider,
    game: &GameState,
    chooser: PlayerId,
    entering: ObjectId,
    source: ObjectId,
    to: crate::types::zones::Zone,
    candidates: &[ObjectId],
    max: usize,
) -> Vec<ObjectId> {
    assert!(
        !candidates.is_empty(),
        "ask_choose_auxiliary_zone_change: nothing to choose from for {}",
        entering,
    );
    let options: Vec<ChoiceOption> =
        candidates.iter().map(|id| ChoiceOption::Object(*id)).collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::ChooseAuxiliaryZoneChange { entering, source, to },
    };
    let indices = dp.pick_n(game, chooser, &ctx, &options, (0, max));
    validate_pick_n(&indices, options.len(), (0, max), "choose_auxiliary_zone_change", &game.counters);
    // Sorted, so the batch is built in candidate order however the provider
    // returned its picks — the order the moves are performed in is observable
    // (a graveyard is ordered), and it must not depend on a DP's whim.
    let mut picked: Vec<usize> = indices;
    picked.sort_unstable();
    picked.into_iter().map(|i| candidates[i]).collect()
}

pub fn ask_choose_copy_source(
    dp: &dyn DecisionProvider,
    game: &GameState,
    chooser: PlayerId,
    spell_id: ObjectId,
    candidates: &[ObjectId],
) -> ObjectId {
    assert!(
        candidates.len() >= 2,
        "ask_choose_copy_source: a choice needs two or more candidates; called with {}",
        candidates.len(),
    );
    let options: Vec<ChoiceOption> =
        candidates.iter().map(|id| ChoiceOption::Object(*id)).collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::ChooseCopySource { spell_id },
    };
    let index = dp.pick_n(game, chooser, &ctx, &options, (1, 1));
    validate_pick_n(&index, options.len(), (1, 1), "choose_copy_source", &game.counters);
    candidates[index[0]]
}

/// CR 609.7a — choose the source of damage a prevention or replacement effect
/// names, as the effect is created.
///
/// `source` is the object whose effect is asking; `candidates` is
/// `SelectionFilter::DamageSource`'s enumeration, permanents then stack
/// spells.
///
/// **Only called with two or more**, for [`ask_choose_copy_source`]'s reason:
/// with one candidate the choice is forced and the caller takes it without
/// asking anyone.
pub fn ask_choose_damage_source(
    dp: &dyn DecisionProvider,
    game: &GameState,
    chooser: PlayerId,
    source: ObjectId,
    candidates: &[ObjectId],
) -> ObjectId {
    assert!(
        candidates.len() >= 2,
        "ask_choose_damage_source: a choice needs two or more candidates; called with {}",
        candidates.len(),
    );
    let options: Vec<ChoiceOption> =
        candidates.iter().map(|id| ChoiceOption::Object(*id)).collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::ChooseDamageSource { source },
    };
    let index = dp.pick_n(game, chooser, &ctx, &options, (1, 1));
    validate_pick_n(&index, options.len(), (1, 1), "choose_damage_source", &game.counters);
    candidates[index[0]]
}

/// Choose which permanents pay a `Cost::Sacrifice` (CR 601.2h, 701.21a).
///
/// `candidates` is every permanent the payer controls that matches the cost's
/// filter, in `battlefield_ids_ordered`; `count` is how many the cost takes.
///
/// **Only called with more candidates than the cost needs.** With exactly
/// `count` the payment is forced and the caller sacrifices them without
/// asking anyone — `CLAUDE.md`'s "never prompt with fewer than two
/// candidates", counted against the choice rather than against the list.
pub fn ask_choose_sacrifice_for_cost(
    dp: &dyn DecisionProvider,
    game: &GameState,
    player: PlayerId,
    spell_or_ability_id: ObjectId,
    count: u32,
    candidates: &[ObjectId],
) -> Vec<ObjectId> {
    let n = count as usize;
    assert!(
        candidates.len() > n,
        "ask_choose_sacrifice_for_cost: {} candidates for {} sacrifices is forced",
        candidates.len(),
        n,
    );
    let options: Vec<ChoiceOption> =
        candidates.iter().map(|id| ChoiceOption::Object(*id)).collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id, count },
    };
    let picks = dp.pick_n(game, player, &ctx, &options, (n, n));
    validate_pick_n(&picks, options.len(), (n, n), "choose_sacrifice_for_cost", &game.counters);
    picks.into_iter().map(|i| candidates[i]).collect()
}

/// Choose which legendary permanent to keep (rule 704.5j legend rule).
pub fn ask_choose_legend_to_keep(
    dp: &dyn DecisionProvider,
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
    validate_pick_n(&index, options.len(), (1, 1), "choose_legend_to_keep", &game.counters);
    legendaries[index[0]]
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::game_state::GameState;
    use crate::ui::decision::ScriptedDecisionProvider;

    fn test_game_state() -> GameState {
        GameState::new(2, 20)
    }

    // --- pick_n basic ---

    #[test]
    fn test_pick_n_returns_correct_index() {
        let dp = ScriptedDecisionProvider::new();
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
        let dp = ScriptedDecisionProvider::new();
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
        let dp = ScriptedDecisionProvider::new();
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
        let dp = ScriptedDecisionProvider::new();
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
            None,
        );
        assert_eq!(blockers, vec![(blocker_a, 2), (blocker_b, 1)]);
        assert_eq!(overflow, 2);
    }

    #[test]
    #[should_panic(expected = "DP allocated 1 to bucket 0 but minimum is 2")]
    fn test_trample_rejects_below_per_bucket_min() {
        let dp = ScriptedDecisionProvider::new();
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
            None,
        );
    }

    // --- the generic split's clamped maxima ---
    //
    // The clamp is pinned from both sides, because subtracting the pips *twice*
    // would be as wrong as not subtracting them at all. Below: a color whose
    // pips claim all of it is offered zero, and a color with surplus beyond its
    // pips is still offered the surplus. `validate_allocation` names the number
    // in its panic, so the first half needs no instrumentation — the DP tries
    // the split the old prompt allowed, and the message says what the max was.

    /// `{1}{G}{U}` against one Green, one Blue and one Red: the Green is spoken
    /// for by the `{G}` pip, so putting the generic mana there is not on the
    /// menu. The old prompt offered the pool's amounts — [1, 1, 1] — which is
    /// exactly how a DP named a split `ManaPool::pay` refused and CR 601.2
    /// rewound the cast for (`codebase-state.md` 16c).
    ///
    /// With both pips clamped to zero the Red is the only bucket left, so the
    /// engine answers without asking and the scripted provider is never
    /// touched.
    #[test]
    fn test_generic_split_refuses_a_color_its_own_pips_need() {
        let dp = ScriptedDecisionProvider::new();
        let game = test_game_state();
        let cost = ManaCost::build(&[ManaType::Green, ManaType::Blue], 1);
        // Buckets are `ManaType`-ordered: Blue, Red, Green. Bucket 2 is the Green.
        let available = [(ManaType::Blue, 1), (ManaType::Red, 1), (ManaType::Green, 1)];

        let alloc = ask_choose_generic_mana_allocation(&dp, &game, 0, &cost, &available, 1);

        assert_eq!(alloc, HashMap::from([(ManaType::Red, 1)]));
    }

    /// The same cost from `{G}{G}{U}`: the second Green is surplus, so it is
    /// still offered and still pays. This is the other side of the clamp — it
    /// subtracts the pips once, not to zero.
    #[test]
    fn test_generic_split_offers_a_color_s_surplus_beyond_its_pips() {
        let dp = ScriptedDecisionProvider::new();
        let game = test_game_state();
        let cost = ManaCost::build(&[ManaType::Green, ManaType::Blue], 1);
        let available = [(ManaType::Blue, 1), (ManaType::Green, 2)];
        // The surplus Green is the only bucket with room, so this is the forced
        // answer rather than a scripted one.

        let alloc = ask_choose_generic_mana_allocation(&dp, &game, 0, &cost, &available, 1);

        assert_eq!(alloc, HashMap::from([(ManaType::Green, 1)]));
    }

    /// A `{C}` pip is clamped the same way a colored one is — `ManaPool::pay`
    /// makes no distinction, and `ManaCost::build` writes it as
    /// `ManaSymbol::Colorless` rather than `Colored(ManaType::Colorless)`.
    ///
    /// The clamp leaves one bucket able to take the generic mana, so the answer
    /// is forced and no provider sees it: a `ScriptedDecisionProvider` with
    /// nothing scripted would panic on any call, which is what says the prompt
    /// is gone as well as that the clamp held.
    #[test]
    fn test_generic_split_clamps_a_colorless_pip() {
        let dp = ScriptedDecisionProvider::new();
        let game = test_game_state();
        let cost = ManaCost::build(&[ManaType::Colorless], 1);
        // Red, then Colorless. Bucket 1 is the Colorless, and the {C} pip has it.
        let available = [(ManaType::Red, 1), (ManaType::Colorless, 1)];

        let alloc = ask_choose_generic_mana_allocation(&dp, &game, 0, &cost, &available, 1);

        assert_eq!(alloc, HashMap::from([(ManaType::Red, 1)]));
    }

    /// The random agent does not clamp for itself, so this is the statement
    /// that the engine's maxima are what keeps every split payable.
    ///
    /// **The board gives it a real choice**, which is what makes this a test of
    /// the agent rather than of `forced_allocation`: `{1}{G}{U}` against two
    /// Blue, one Red and two Green leaves every bucket a surplus of one, so the
    /// provider is asked. Over 50 seeds no answer ever spends mana a pip is
    /// owed — one unit, and never more than the surplus of the type it
    /// lands on.
    #[test]
    fn test_random_dp_generic_split_spends_only_the_surplus() {
        use crate::ui::random::RandomDecisionProvider;

        let game = test_game_state();
        let cost = ManaCost::build(&[ManaType::Green, ManaType::Blue], 1);
        let available = [(ManaType::Blue, 2), (ManaType::Red, 1), (ManaType::Green, 2)];
        let surplus = HashMap::from([
            (ManaType::Blue, 1u64),
            (ManaType::Red, 1),
            (ManaType::Green, 1),
        ]);
        for seed in 0..50u64 {
            let dp = RandomDecisionProvider::seeded(seed);
            let alloc = ask_choose_generic_mana_allocation(&dp, &game, 0, &cost, &available, 1);
            assert_eq!(alloc.values().sum::<u64>(), 1, "seed {seed}");
            for (mana_type, amount) in &alloc {
                assert!(amount <= &surplus[mana_type], "seed {seed}: {mana_type:?} over surplus");
            }
        }
    }

    // --- ask_choose_attackers roundtrip ---

    #[test]
    fn test_ask_choose_attackers_roundtrip() {
        let dp = ScriptedDecisionProvider::new();
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
        let dp = ScriptedDecisionProvider::new();
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
        let dp = ScriptedDecisionProvider::new();
        let game = test_game_state();
        dp.expect_pick_n(ChoiceKind::PriorityAction, vec![5]);
        let actions = vec![PriorityAction::Pass, PriorityAction::Pass];
        let _ = ask_choose_priority_action(&dp, &game, 0, &actions);
    }

    #[test]
    #[should_panic(expected = "DP returned 2 selections, expected 1-1")]
    fn test_validation_rejects_wrong_count() {
        let dp = ScriptedDecisionProvider::new();
        let game = test_game_state();
        dp.expect_pick_n(ChoiceKind::PriorityAction, vec![0, 1]);
        let actions = vec![PriorityAction::Pass, PriorityAction::Pass];
        let _ = ask_choose_priority_action(&dp, &game, 0, &actions);
    }

    #[test]
    #[should_panic(expected = "allocation sum is 4 but total should be 3")]
    fn test_validation_rejects_bad_allocation_sum() {
        let dp = ScriptedDecisionProvider::new();
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
        let dp = ScriptedDecisionProvider::new();
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
        let dp = ScriptedDecisionProvider::new();
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
        let dp = ScriptedDecisionProvider::new();
        let game = test_game_state();
        let actions = vec![PriorityAction::Pass];
        let _ = ask_choose_priority_action(&dp, &game, 0, &actions);
    }

    // --- DispatchDecisionProvider routing ---

    #[test]
    fn test_dispatch_routes_by_player() {
        use crate::ui::decision::DispatchDecisionProvider;

        let dp0 = ScriptedDecisionProvider::new();
        let dp1 = ScriptedDecisionProvider::new();

        // Player 0 should get action index 0, player 1 should get index 1
        dp0.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
        dp1.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);

        let dispatch = DispatchDecisionProvider::new(vec![
            Box::new(dp0),
            Box::new(dp1),
        ]);

        let game = test_game_state();
        let actions = vec![
            PriorityAction::Pass,
            PriorityAction::PlayLand(crate::types::ids::new_object_id()),
        ];

        // Player 0 chooses index 0 → Pass
        let result0 = ask_choose_priority_action(&dispatch, &game, 0, &actions);
        assert!(matches!(result0, PriorityAction::Pass));

        // Player 1 chooses index 1 → PlayLand
        let result1 = ask_choose_priority_action(&dispatch, &game, 1, &actions);
        assert!(matches!(result1, PriorityAction::PlayLand(_)));
    }

    // --- RandomDecisionProvider generic trait ---

    #[test]
    fn test_random_pick_n_respects_bounds() {
        use crate::ui::random::RandomDecisionProvider;

        let dp = RandomDecisionProvider::new();
        let game = test_game_state();

        let ctx = ChoiceContext {
            kind: ChoiceKind::DeclareAttackers,
        };
        let options = vec![
            ChoiceOption::Object(crate::types::ids::new_object_id()),
            ChoiceOption::Object(crate::types::ids::new_object_id()),
            ChoiceOption::Object(crate::types::ids::new_object_id()),
        ];

        // Run multiple times to exercise random selection
        for _ in 0..20 {
            let result = dp.pick_n(&game, 0, &ctx, &options, (1, 2));
            assert!(
                !result.is_empty() && result.len() <= 2,
                "pick_n returned {} selections, expected 1-2",
                result.len()
            );
            for &idx in &result {
                assert!(idx < options.len(), "index {} out of range", idx);
            }
            // Check no duplicates
            let mut unique = result.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(unique.len(), result.len(), "duplicates in pick_n result");
        }
    }

    #[test]
    fn test_random_pick_n_exact_bounds() {
        use crate::ui::random::RandomDecisionProvider;

        let dp = RandomDecisionProvider::new();
        let game = test_game_state();

        let ctx = ChoiceContext {
            kind: ChoiceKind::PriorityAction,
        };
        let options = vec![
            ChoiceOption::Action(PriorityAction::Pass),
            ChoiceOption::Action(PriorityAction::Pass),
        ];

        // When bounds are (1,1), always returns exactly 1
        for _ in 0..10 {
            let result = dp.pick_n(&game, 0, &ctx, &options, (1, 1));
            assert_eq!(result.len(), 1);
        }
    }

    #[test]
    fn test_random_pick_number_x_value_self_limits() {
        use crate::ui::random::RandomDecisionProvider;

        let dp = RandomDecisionProvider::new();
        let game = test_game_state(); // 2 players, 20 life, no permanents
        let spell_id = crate::types::ids::new_object_id();

        let ctx = ChoiceContext {
            kind: ChoiceKind::ChooseXValue { spell_id, x_count: 1 },
        };

        // No mana in pool, no lands on battlefield → reasonable max is 0
        // So the result should be min (0)
        for _ in 0..10 {
            let result = dp.pick_number(&game, 0, &ctx, 0, u64::MAX);
            // With empty pool and no lands, should return 0
            assert_eq!(result, 0, "X value should be 0 with no mana sources");
        }
    }

    #[test]
    fn test_random_allocate_sums_correctly() {
        use crate::ui::random::RandomDecisionProvider;

        let dp = RandomDecisionProvider::new();
        let game = test_game_state();

        let ctx = ChoiceContext {
            kind: ChoiceKind::AssignCombatDamage {
                attacker_id: crate::types::ids::new_object_id(),
            },
        };
        let buckets = vec![
            ChoiceOption::Object(crate::types::ids::new_object_id()),
            ChoiceOption::Object(crate::types::ids::new_object_id()),
        ];
        let mins = vec![0, 0];

        for _ in 0..20 {
            let alloc = dp.allocate(&game, 0, &ctx, 5, &buckets, &mins, None);
            assert_eq!(alloc.len(), 2);
            assert_eq!(alloc.iter().sum::<u64>(), 5, "allocate sum must equal total");
        }
    }

    #[test]
    fn test_random_allocate_respects_minimums() {
        use crate::ui::random::RandomDecisionProvider;

        let dp = RandomDecisionProvider::new();
        let game = test_game_state();

        let ctx = ChoiceContext {
            kind: ChoiceKind::AssignTrampleDamage {
                attacker_id: crate::types::ids::new_object_id(),
                defending_target: crate::events::event::DamageTarget::Player(1),
            },
        };
        let buckets = vec![
            ChoiceOption::Object(crate::types::ids::new_object_id()),
            ChoiceOption::Player(1),
        ];
        let mins = vec![3, 0]; // First bucket needs at least 3

        for _ in 0..20 {
            let alloc = dp.allocate(&game, 0, &ctx, 5, &buckets, &mins, None);
            assert_eq!(alloc.len(), 2);
            assert_eq!(alloc.iter().sum::<u64>(), 5);
            assert!(alloc[0] >= 3, "bucket 0 must have at least 3, got {}", alloc[0]);
        }
    }

    // ===========================================================================
    // SPECIAL-5: Class A — validator negative tests (direct calls to validate_*)
    // ===========================================================================

    #[test]
    #[should_panic(expected = "DP returned 2 allocations but 3 buckets provided")]
    fn test_validation_rejects_wrong_bucket_count() {
        let alloc = vec![1u64, 2];
        let mins = vec![0u64; 3];
        validate_allocation(&alloc, 3, 3, &mins, None, "test", &EngineCounters::default());
    }

    #[test]
    #[should_panic(expected = "per_bucket_mins length 2 != buckets length 3")]
    fn test_validation_rejects_mismatched_mins_length() {
        let alloc = vec![1u64, 1, 1];
        let mins = vec![0u64; 2]; // wrong length
        validate_allocation(&alloc, 3, 3, &mins, None, "test", &EngineCounters::default());
    }

    #[test]
    #[should_panic(expected = "per_bucket_maxs length 2 != buckets length 3")]
    fn test_validation_rejects_mismatched_maxs_length() {
        let alloc = vec![1u64, 1, 1];
        let mins = vec![0u64; 3];
        let maxs = vec![3u64; 2]; // wrong length
        validate_allocation(&alloc, 3, 3, &mins, Some(&maxs), "test", &EngineCounters::default());
    }

    #[test]
    #[should_panic(expected = "DP allocated 5 to bucket 1 but maximum is 3")]
    fn test_validation_rejects_above_per_bucket_max() {
        let alloc = vec![0u64, 5];
        let mins = vec![0u64, 0];
        let maxs = vec![10u64, 3];
        validate_allocation(&alloc, 2, 5, &mins, Some(&maxs), "test", &EngineCounters::default());
    }

    /// The three shapes `forced_allocation` answers, and the four it refuses.
    ///
    /// The refusals are the half that matters: an answer it gives is used
    /// without `validate_allocation` running at all, so an infeasible input it
    /// answered would be a silent wrong split — `codebase-state.md` 16c's
    /// failure, reached from the other side.
    #[test]
    fn test_forced_allocation_answers_only_a_feasible_unique_split() {
        // Nothing left over the minimums.
        assert_eq!(forced_allocation(3, &[1, 2], None), Some(vec![1, 2]));
        // One bucket free to take the remainder.
        assert_eq!(forced_allocation(3, &[0, 0], Some(&[0, 5])), Some(vec![0, 3]));
        // Nothing spare under the maxima.
        assert_eq!(forced_allocation(5, &[0, 0], Some(&[2, 3])), Some(vec![2, 3]));

        // A real choice: two free buckets with room to spare.
        assert_eq!(forced_allocation(1, &[0, 0], Some(&[1, 1])), None);
        assert_eq!(forced_allocation(2, &[0, 0], None), None);
        // The minimums already exceed the total.
        assert_eq!(forced_allocation(1, &[1, 1], None), None);
        // The one free bucket cannot hold the remainder.
        assert_eq!(forced_allocation(5, &[0, 0], Some(&[0, 2])), None);
        // Bounds that disagree, and bounds that do not line up with the buckets.
        assert_eq!(forced_allocation(1, &[2, 0], Some(&[1, 4])), None);
        assert_eq!(forced_allocation(1, &[0, 0], Some(&[1])), None);
    }

    #[test]
    #[should_panic(expected = "DP returned duplicate index 1")]
    fn test_validation_rejects_duplicate_pick_n_index() {
        validate_pick_n(&[1, 1], 3, (2, 2), "test", &EngineCounters::default());
    }

    #[test]
    #[should_panic(expected = "DP returned 0 selections, expected 1-2")]
    fn test_validation_rejects_count_below_min() {
        validate_pick_n(&[], 5, (1, 2), "test", &EngineCounters::default());
    }

    #[test]
    #[should_panic(expected = "DP returned 5 but range is [0, 3]")]
    fn test_validation_pick_number_above_max() {
        validate_pick_number(5, 0, 3, "test", &EngineCounters::default());
    }

    #[test]
    #[should_panic(expected = "DP returned 1 but range is [3, 10]")]
    fn test_validation_pick_number_below_min() {
        validate_pick_number(1, 3, 10, "test", &EngineCounters::default());
    }

    #[test]
    #[should_panic(expected = "DP returned 2 indices but 3 items to order")]
    fn test_validation_ordering_wrong_length() {
        validate_ordering(&[0, 1], 3, "test", &EngineCounters::default());
    }

    #[test]
    #[should_panic(expected = "DP returned duplicate index 1 in ordering")]
    fn test_validation_ordering_duplicate() {
        validate_ordering(&[0, 1, 1], 3, "test", &EngineCounters::default());
    }

    #[test]
    #[should_panic(expected = "DP returned index 3 but only 3 items")]
    fn test_validation_ordering_index_oob() {
        validate_ordering(&[0, 1, 3], 3, "test", &EngineCounters::default());
    }

    // ===========================================================================
    // SPECIAL-5: Class D — RandomDecisionProvider contract property tests
    // ===========================================================================
    //
    // These run 200 iterations each, exercising the 4-primitive contract with
    // randomized valid inputs. A seeded RNG drives input generation so failures
    // are reproducible; the DP itself uses thread RNG for responses.

    #[test]
    fn test_random_dp_pick_n_contract_property() {
        use crate::ui::random::RandomDecisionProvider;
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        use rand::Rng;

        let dp = RandomDecisionProvider::new();
        let game = test_game_state();
        let mut seeded = StdRng::seed_from_u64(0xD1CE_C04E);

        for _ in 0..200 {
            let n: usize = seeded.random_range(1..=8);
            let lo: usize = seeded.random_range(0..=n);
            let hi: usize = seeded.random_range(lo..=n);
            let options: Vec<ChoiceOption> = (0..n)
                .map(|_| ChoiceOption::Object(crate::types::ids::new_object_id()))
                .collect();
            let ctx = ChoiceContext { kind: ChoiceKind::PriorityAction };
            let result = dp.pick_n(&game, 0, &ctx, &options, (lo, hi));

            // Contract: length within bounds, indices in range, no duplicates
            assert!(result.len() >= lo && result.len() <= hi,
                "pick_n len {} not in [{}, {}]", result.len(), lo, hi);
            let mut seen = vec![false; n];
            for &i in &result {
                assert!(i < n, "index {} >= options_len {}", i, n);
                assert!(!seen[i], "duplicate index {}", i);
                seen[i] = true;
            }
        }
    }

    #[test]
    fn test_random_dp_pick_number_contract_property() {
        use crate::ui::random::RandomDecisionProvider;
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        use rand::Rng;

        let dp = RandomDecisionProvider::new();
        let game = test_game_state();
        let mut seeded = StdRng::seed_from_u64(0xCAFE_F00D);

        // Use a non-X ChoiceKind so pick_number uses the general branch (not
        // the X-value self-limiting branch which clamps to game state).
        let ctx = ChoiceContext { kind: ChoiceKind::PriorityAction };

        for _ in 0..200 {
            let min: u64 = seeded.random_range(0..=50);
            let max: u64 = seeded.random_range(min..=min + 100);
            let result = dp.pick_number(&game, 0, &ctx, min, max);
            assert!(result >= min && result <= max,
                "pick_number {} not in [{}, {}]", result, min, max);
        }
    }

    #[test]
    fn test_random_dp_allocate_contract_property() {
        use crate::ui::random::RandomDecisionProvider;
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        use rand::Rng;

        let dp = RandomDecisionProvider::new();
        let game = test_game_state();
        let mut seeded = StdRng::seed_from_u64(0xBEEF_F00D);

        for _ in 0..200 {
            let n: usize = seeded.random_range(1..=6);
            // Per-bucket mins and maxs, with maxs >= mins, and a feasible total.
            let mut mins = Vec::with_capacity(n);
            let mut maxs = Vec::with_capacity(n);
            let mut min_sum: u64 = 0;
            let mut max_sum: u64 = 0;
            for _ in 0..n {
                let lo: u64 = seeded.random_range(0..=3);
                let hi: u64 = seeded.random_range(lo..=lo + 5);
                mins.push(lo);
                maxs.push(hi);
                min_sum += lo;
                max_sum += hi;
            }
            let total: u64 = if min_sum == max_sum { min_sum }
                else { seeded.random_range(min_sum..=max_sum) };

            let buckets: Vec<ChoiceOption> = (0..n)
                .map(|_| ChoiceOption::Object(crate::types::ids::new_object_id()))
                .collect();
            let ctx = ChoiceContext {
                kind: ChoiceKind::AssignCombatDamage {
                    attacker_id: crate::types::ids::new_object_id(),
                },
            };
            let alloc = dp.allocate(&game, 0, &ctx, total, &buckets, &mins, Some(&maxs));

            assert_eq!(alloc.len(), n, "alloc len mismatch");
            let sum: u64 = alloc.iter().sum();
            assert_eq!(sum, total, "alloc sum {} != total {}", sum, total);
            for (i, &v) in alloc.iter().enumerate() {
                assert!(v >= mins[i], "bucket {} value {} < min {}", i, v, mins[i]);
                assert!(v <= maxs[i], "bucket {} value {} > max {}", i, v, maxs[i]);
            }
        }
    }

    #[test]
    fn test_random_dp_ordering_contract_property() {
        use crate::ui::random::RandomDecisionProvider;
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        use rand::Rng;

        let dp = RandomDecisionProvider::new();
        let game = test_game_state();
        let mut seeded = StdRng::seed_from_u64(0xFACE_B00C);

        let ctx = ChoiceContext { kind: ChoiceKind::PriorityAction };

        for _ in 0..200 {
            let n: usize = seeded.random_range(0..=10);
            let items: Vec<ChoiceOption> = (0..n)
                .map(|_| ChoiceOption::Object(crate::types::ids::new_object_id()))
                .collect();
            let order = dp.choose_ordering(&game, 0, &ctx, &items);
            assert_eq!(order.len(), n, "ordering length mismatch");
            let mut seen = vec![false; n];
            for &i in &order {
                assert!(i < n, "ordering index {} >= n {}", i, n);
                assert!(!seen[i], "duplicate index {} in ordering", i);
                seen[i] = true;
            }
        }
    }

    #[test]
    fn test_random_choose_ordering_is_permutation() {
        use crate::ui::random::RandomDecisionProvider;

        let dp = RandomDecisionProvider::new();
        let game = test_game_state();

        let ctx = ChoiceContext {
            kind: ChoiceKind::PriorityAction, // placeholder kind
        };
        let items = vec![
            ChoiceOption::Object(crate::types::ids::new_object_id()),
            ChoiceOption::Object(crate::types::ids::new_object_id()),
            ChoiceOption::Object(crate::types::ids::new_object_id()),
        ];

        for _ in 0..20 {
            let order = dp.choose_ordering(&game, 0, &ctx, &items);
            assert_eq!(order.len(), items.len());
            let mut sorted = order.clone();
            sorted.sort();
            assert_eq!(sorted, vec![0, 1, 2], "ordering must be a permutation");
        }
    }

}
