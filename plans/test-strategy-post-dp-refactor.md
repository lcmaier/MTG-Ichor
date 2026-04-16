# Test Strategy: Post-DP Refactor

> Written after SPECIAL-1c completion (2026-04-15). The DecisionProvider trait
> is now a 4-primitive generic interface. This document describes what that
> unlocks for testing and outlines a philosophy for new test development.

---

## 1. What Changed

### Before (old trait)
- **12+ typed methods** on `DecisionProvider` (`choose_attackers`, `choose_blockers`,
  `choose_targets`, `choose_priority_action`, `choose_discard`, etc.)
- **`PassiveDecisionProvider`** — returned hardcoded "pass everything" defaults,
  silently making decisions the test never specified
- **Silent fallback** — if a test didn't script a decision, PassiveDP picked
  *something* (empty attackers, first option, pass priority), masking bugs
- **Per-method script queues** — `ScriptedDecisionProvider` had separate
  `VecDeque<Vec<(ObjectId, AttackTarget)>>` for attackers,
  `VecDeque<Vec<(ObjectId, ObjectId)>>` for blockers, etc.

### After (4-primitive trait)
- **4 generic methods:** `pick_n`, `pick_number`, `allocate`, `choose_ordering`
- **`ChoiceKind` enum** carries semantic meaning — the *engine* knows what
  decision it's asking; the *DP* knows how to answer it
- **`ScriptedDecisionProvider`** — single queue of `(ChoiceKind, Response)` pairs.
  Every DP call must have a matching expectation. Extra expectations panic on Drop.
  Missing expectations panic on call.
- **`ask_*` free functions** — typed wrappers that build `ChoiceOption` lists,
  call the appropriate primitive, and validate/unpack the response
- **`validate_allocation`** — centralized validation with `per_bucket_mins` and
  `per_bucket_maxs`

---

## 2. What This Unlocks

### 2a. Self-Documenting Tests
Every decision a test requires is explicitly stated via `expect_*`. Reading a
test's setup tells you *exactly* what decisions the player makes and in what
order. No more "PassiveDP did something and we hope it's fine."

### 2b. Wrong-Decision Detection
If the engine asks for a decision the test didn't anticipate, ScriptedDP panics
immediately with a message saying what `ChoiceKind` was unexpected. If the test
scripts decisions that are never consumed, the `Drop` impl panics. Both
directions are covered.

### 2c. Negative / Invalid Response Testing
Since the DP is generic, we can deliberately return *invalid* responses:
- Out-of-bounds indices from `pick_n`
- Allocation that doesn't sum to total
- Allocation that violates `per_bucket_mins` or `per_bucket_maxs`
- Indices repeated in `pick_n` when uniqueness is required

The `ask_*` wrappers and `validate_allocation` should catch these with panics or
errors. We can now write `#[should_panic]` tests proving they do.

### 2d. Decision Sequence Specification
Because all decisions flow through one queue, we can write integration tests
that assert the *exact sequence* of decisions the engine asks for during a
complex operation. E.g., "casting Doom Blade should produce exactly:
PriorityAction → SelectRecipients → GenericManaAllocation" in that order.

### 2e. Option Enumeration Testing
The `ask_*` functions build `ChoiceOption` lists from game state. We can write
tests that set up a board state and assert the *options presented* to the DP
are correct — without caring what the DP chooses. This tests the `ask_*` layer
independently of any DP implementation.

### 2f. DP Implementation Property Testing
The 4 primitives have clear contracts:
- `pick_n`: returned indices must be in `[0, options.len())`, count in `[min, max]`
- `pick_number`: result in `[min, max]`
- `allocate`: sum == total, each bucket >= min, each bucket <= max (if given)
- `choose_ordering`: returned indices are a permutation of `[0, items.len())`

These are perfect candidates for property-based / fuzz testing against
`RandomDecisionProvider` — every response must satisfy the contract for any
valid input.

---

## 3. Test Classes

### Class A: Validation / Negative Tests (`#[should_panic]`)
Assert that `validate_allocation` and `ask_*` wrappers reject invalid DP
responses. These already exist for allocation (see `ask.rs` tests:
`test_validation_rejects_*`). Extend to:
- `pick_n` returning index >= options.len()
- `pick_n` returning wrong count vs bounds
- `allocate` with sum != total
- `allocate` violating per_bucket_mins
- `allocate` violating per_bucket_maxs

### Class B: Decision Sequence Tests
Integration-level tests that assert the *order and kind* of DP calls for a
game action. Use `ScriptedDecisionProvider` and verify all expectations consumed.
Examples:
- Cast a targeted spell → PriorityAction, SelectRecipients, GenericManaAllocation
- Cast a spell with X → PriorityAction, ChooseXValue, GenericManaAllocation
- Declare attackers + blockers + damage → DeclareAttackers, DeclareBlockers, AssignCombatDamage
- Legend rule fires during SBA → LegendRule
- End-of-turn discard → DiscardToHandSize

### Class C: Option Enumeration Tests
Unit tests on `ask_*` functions that set up game state, call the `ask_*`
wrapper with a mock DP (ScriptedDP returning index 0 or similar), and assert
the *options* that were presented. Examples:
- `ask_choose_attackers` — only untapped, non-summoning-sick, non-defender creatures appear
- `ask_choose_blockers` — only untapped creatures appear as blocker candidates
- `ask_choose_priority_action` — legal actions include land play only in main phase with land drop remaining
- `ask_choose_targets` with SelectionFilter — only matching permanents appear

### Class D: DP Contract Property Tests
Fuzz/property tests that generate random game states and valid inputs, call
RandomDecisionProvider, and assert the response satisfies the primitive's
contract. These catch bugs in RandomDP (which is used by the fuzz harness).
- Random `pick_n` always returns valid indices within bounds
- Random `allocate` always sums correctly and respects mins/maxs
- Random `choose_ordering` always returns a valid permutation

### Class E: Regression Tests (former PassiveDP gaps)
Tests for scenarios where PassiveDP previously masked bugs by returning
hardcoded defaults. The key pattern: "what happens when the player makes a
*specific* choice?" Examples:
- Player assigns all trample damage to first blocker (suboptimal but legal)
- Player chooses to keep the *second* legend, not the first
- Player discards a specific card (not the first one in hand)
- Player chooses 0 for X value (legal but degenerate)

---

## 4. Prompt for LLM Test Planning Session

Below is a prompt template to hand to an LLM for generating a detailed test
plan. Replace `[SNIPPET]` markers with actual code.

---

```
# Context

I'm building an MTG rules engine in Rust. Player decisions go through a
4-primitive `DecisionProvider` trait:

## Trait (4 methods)

[PASTE: DecisionProvider trait from decision.rs L343-397]

## ChoiceKind enum (what decisions exist)

[PASTE: ChoiceKind enum from choice_types.rs L23-45]

## ChoiceOption enum (what options look like)

[PASTE: ChoiceOption enum from choice_types.rs L57-82]

## ScriptedDecisionProvider (how tests work)

Tests use `ScriptedDecisionProvider` which has a single FIFO queue of
`(expected_kind, response)` pairs. You enqueue expectations before running
game logic. If the engine asks for a decision that doesn't match the front
of the queue, it panics. If the test ends with unconsumed expectations, the
Drop impl panics. This means every test explicitly declares every decision.

Key methods: `expect_pick_n(kind, indices)`, `expect_number(kind, n)`,
`expect_allocation(kind, alloc)`, `expect_ordering(kind, order)`.

Convenience: `queue_empty_turn_passes()` enqueues 16 PriorityAction passes
(8 priority points × 2 players for one turn with no attackers).

## Representative ask_* function

[PASTE: ask_choose_attackers from ask.rs — shows how options are built
from game state, passed to dp.pick_n, and unpacked]

## Representative test example

[PASTE: one existing integration test, e.g. test_cast_and_resolve_lightning_bolt
from phase2_integration_test.rs]

# Task

Design a comprehensive test plan for this system. For each test, provide:
- **Name** (Rust test function name)
- **Class** (A: validation, B: sequence, C: enumeration, D: contract, E: regression)
- **Setup** (game state + DP expectations)
- **Assertion** (what to verify)

Focus on:
1. Class A: 5-8 negative tests proving validation catches bad DP responses
2. Class B: 5-8 sequence tests proving the engine asks decisions in the right order
3. Class C: 5-8 enumeration tests proving ask_* functions present correct options
4. Class E: 3-5 regression tests for scenarios PassiveDP would have masked

Do NOT write Rust code. Write test specs only — I will implement them.
Keep specs concise: 3-4 sentences each.
```

---

## 5. Priority Order for Implementation

1. **Class A** first — these are small, isolated, fast to write, and catch
   validation bugs before they become integration mysteries
2. **Class C** next — option enumeration tests catch "wrong options presented"
   bugs, which are subtle and common when new cards/mechanics are added
3. **Class B** after — sequence tests provide confidence that complex flows
   (cast pipeline, combat) ask decisions in the right order
4. **Class E** last — regression tests are valuable but less urgent since the
   new ScriptedDP already catches the "unscripted decision" class of bugs
5. **Class D** ongoing — property tests against RandomDP should be added to the
   fuzz harness incrementally

---

## 6. Tickets and Conventions Landed in the Plan

As of 2026-04-16 the following changes have been made to
`plans/implementation-plan-final.md`:

- **SPECIAL-5 (D31)** — DP validation + contract property tests (Classes A + D).
  Bounded one-shot ticket. Covers `validate_allocation`, `pick_n`/`pick_number`/
  `choose_ordering` bound checks, and property tests for `RandomDecisionProvider`
  against the 4-primitive contract. Also adds debug-mode contract assertions to
  `ask_*` wrappers so the fuzz harness exercises contracts in realistic states.

- **SPECIAL-6 (D32)** — `ask_*` option enumeration tests (Class C). Initial PR
  covers `ask_choose_attackers`, `ask_choose_blockers`,
  `ask_choose_priority_action`, `ask_select_objects`, `ask_choose_legend_to_keep`.
  Living ticket: every future `ask_*` / `ChoiceKind` / `SelectionFilter` /
  `PermanentFilter` addition must add at least one Class C test in the same PR.

- **Testing Conventions** section added to `implementation-plan-final.md` just
  before Part 1. Codifies:
  - The A/B/C/D/E taxonomy as the shared vocabulary for ticket `Tests` bullets.
  - Per-ticket coverage rules (when each class is required).
  - Standing rules: write failing tests first for DP surface changes, don't
    weaken tests, no silent DP defaults, keep `ChoiceKind` exhaustive, fuzz
    harness = living Class D suite, always report full test-count breakdowns.

**Classes B and E are intentionally NOT standalone tickets.** They are enforced
per-ticket via the Testing Conventions rules and absorbed organically as each
Phase 5/6/7 ticket lands. Bulk retrofitting Class B/E onto already-passing
tests is low ROI — `ScriptedDP`'s strict queue already provides de-facto
sequence assertions, and regression scenarios are cheapest to write alongside
the feature that needs them.

### Priority adjustment after landing
Original priority (§5) was A → C → B → E → D. Post-ticketing the actionable
queue is:

1. **SPECIAL-5** (Classes A + D, one PR, highest leverage per hour).
2. **SPECIAL-6** initial landing (Class C for the five listed `ask_*` functions).
3. **Class B + E absorbed into future tickets** per Testing Conventions — not
   scheduled as standalone work.
4. **SPECIAL-6 extensions** land continuously with every new `ask_*` /
   `ChoiceKind` / filter variant.
