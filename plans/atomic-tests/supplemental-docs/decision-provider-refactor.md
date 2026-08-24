# DecisionProvider Refactor: 4-Primitive Generic Trait

> Date: 2026-04-13 (updated 2026-04-14)
> Status: **Implemented.** SPECIAL-1a/b/c all shipped (April 2026); the 4-primitive trait
> is what `ui/decision.rs` has today. Kept as the design record, not as a plan.
> Tickets: SPECIAL-1a → SPECIAL-1b → SPECIAL-1c (3-way split)
>   - SPECIAL-1a: types + trait + ask functions + ScriptedDP (Medium)
>   - SPECIAL-1b: CLI, Random, Dispatch implementations (Small)
>   - SPECIAL-1c: engine call site migration + old trait deletion (Medium)
> Depends on: T18a (done). SPECIAL-1c must complete before T18b/c/d.
> Note: Designated SPECIAL (not T18e) because this is a cross-cutting architectural refactor, not a sub-ticket of the T18 casting pipeline.

---

## 1. Problem Statement

The `DecisionProvider` trait currently has 12 typed methods (9 required + 3 with defaults). Each new decision point in the game requires:
1. A new method on the trait
2. Updates to all 5 implementations (Scripted, Random, CLI, Passive, Dispatch)
3. A new queue field on `ScriptedDecisionProvider`

MtG has **far more than 23 distinct decision points** when accounting for the full card pool:
- Thoughtseize-style "choose from revealed hand"
- Choose a color, creature type, card name, CMC, power/toughness value
- Choose a counter type among those on a permanent
- Scry/surveil ordering
- Fact or Fiction pile splitting
- Stack ordering (APNAP)
- Replacement effect choice
- Trample over planeswalkers/battles (tiered allocation)
- Pay optional cost (Rhystic Study)
- And many more niche card-specific choices

Conservatively, the final count is **100+ decision methods** if each gets its own typed signature. This creates an unwieldy trait surface that makes implementing even a basic UI an enormous task.

---

## 2. Design Goals (Four Pillars)

1. **Correctness:** The engine must always receive a valid response in the correct shape. No silent defaults that could mask bugs or produce illegal game states.
2. **Extensibility:** Adding a new decision point should not require touching any existing DP implementation. One new `ChoiceKind` variant + one new `ask_*` free function.
3. **Maintainability:** The DP trait surface stays fixed at 4 methods. `ScriptedDecisionProvider` has 4 queues. New UIs implement 4 methods.
4. **Performance:** No measurable overhead. Context construction is stack-allocated enum + small heap vec. The hot path is always the DP implementation's own logic (AI evaluation, user input), not the dispatch layer.

---

## 3. Architecture

### 3.1 The Trait (4 methods, forever)

```rust
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
    /// Each bucket must receive >= per-bucket minimum (usually 1 for damage, 0 for other).
    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_min: u64,
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
```

### 3.2 ChoiceContext (semantic label, extensible via variants)

```rust
/// What kind of decision is being made. UIs use this to render appropriate
/// screens. AI agents can match on this for specialized heuristics.
/// Adding a new variant here is the ONLY change needed when a new decision
/// type is introduced — no trait methods or impl changes.
#[derive(Debug, Clone)]
pub enum ChoiceKind {
    // --- Priority & Turn Structure ---
    PriorityAction,

    // --- Combat ---
    DeclareAttackers,
    DeclareBlockers,
    AssignCombatDamage { attacker_id: ObjectId },
    AssignTrampleDamage { attacker_id: ObjectId, defending_target: DamageTarget },

    // --- Casting Pipeline (601.2) ---
    ChooseXValue { spell_name: String, x_count: u64 },
    ChooseAlternativeCost,
    ChooseAdditionalCosts,
    ChooseModes { min: usize, max: usize },
    ChooseTargets { recipient: EffectRecipient, spell_name: String },
    ChooseDistribution { total: u64, spell_name: String },
    GenericManaAllocation { mana_cost: ManaCost },

    // --- State-Based & Cleanup ---
    DiscardToHandSize,
    LegendRule { legend_name: String },

    // --- Cost Payment ---
    ChooseSacrifice { filter_description: String, count: u32 },
    CostPaymentOrder,
    PayOptionalCost { description: String },

    // --- Replacement Effects & Triggers ---
    ReplacementEffectChoice,
    StackOrdering,

    // --- Card-Specific Choices ---
    ChooseColor,
    ChooseCreatureType,
    /// Card name selection uses `pick_number` with registry index, NOT `pick_n`
    /// with 27K+ string options. See §8.7 for performance rationale.
    NameCard,
    ChooseFromRevealedHand { owner: PlayerId },
    ChooseCounterType { permanent_id: ObjectId },

    // --- Loop Shortcuts (D26, rule 727/731) ---
    /// DP proposes a shortcut; engine validates against delta log.
    /// Returned as a PriorityAction::ProposeShortcut from ChoosePriorityAction.
    /// Follow-up calls: DeclareLoopCount (iteration count),
    /// AcceptShortcut / ShortenShortcut (opponent responses). See §9.7.
    DeclareLoopCount { loop_description: String },
    AcceptShortcut { proposer: PlayerId, description: String },
    ShortenShortcut { proposer: PlayerId },
    BreakFragmentedLoop,

    // --- Alchemy/Digital ---
    DraftPick { pool_name: String },
    HeistCard,
    SpecializeColor,

    // Future variants added here. Exhaustive matching is intentional:
    // the compiler flags every match site when a variant is added.
}

/// Wrapper carrying the semantic kind. No display text — each DP impl formats
/// its own prompts by matching on `kind`. This keeps choice types pure (no
/// presentation leakage into the engine boundary).
#[derive(Debug, Clone)]
pub struct ChoiceContext {
    pub kind: ChoiceKind,
    // NOTE: No `prompt: String` field. Display formatting belongs in DP impls.
    // CLI matches on `kind` to build human-readable prompts. AI ignores prompts
    // entirely. Network serializes `kind` and lets the client format.
}
```

### 3.3 ChoiceOption (what each index represents)

```rust
/// A single selectable option presented to the DP.
#[derive(Debug, Clone)]
pub enum ChoiceOption {
    /// A game object (creature, card in hand, permanent, etc.)
    Object(ObjectId),
    /// A player
    Player(PlayerId),
    /// A game action (for priority)
    Action(PriorityAction),
    /// An attacker-target pair (for declare attackers)
    AttackerTarget(ObjectId, AttackTarget),
    /// A blocker-attacker pair (for declare blockers)
    BlockerAttacker(ObjectId, ObjectId),
    /// A cost option with human-readable label
    CostOption { index: usize, label: String },
    /// A number (for X value ranges presented as discrete options)
    Number(u64),
    /// A color
    Color(Color),
    /// A creature type
    CreatureType(String),
    /// A card name (only used in small curated lists, NOT the full registry.
    /// Full card name selection uses pick_number with registry index — see §8.7)
    CardName(String),
    /// A counter type
    CounterType(CounterType),
    /// A mana type (for generic allocation)
    ManaType(ManaType),
}
```

### 3.4 Typed `ask_*` Free Functions (engine-side bridge)

These live in a new module `ui/ask.rs` (or `engine/decisions.rs` — TBD). They are the **only** code that constructs `ChoiceContext`/`ChoiceOption` and unpacks indices. Engine call sites and tests use these exclusively.

```rust
/// Engine calls this. Returns typed (ObjectId, AttackTarget) pairs.
pub fn ask_choose_attackers(
    dp: &dyn DecisionProvider,
    game: &GameState,
    player: PlayerId,
    legal: &[(ObjectId, AttackTarget)],
) -> Vec<(ObjectId, AttackTarget)> {
    let options: Vec<ChoiceOption> = legal.iter()
        .map(|(id, t)| ChoiceOption::AttackerTarget(*id, *t))
        .collect();
    let ctx = ChoiceContext {
        kind: ChoiceKind::DeclareAttackers,
    };
    let indices = dp.pick_n(game, player, &ctx, &options, (0, legal.len()));
    indices.iter().map(|&i| legal[i]).collect()
}

/// X value for spells with {X} in their cost.
pub fn ask_choose_x_value(
    dp: &dyn DecisionProvider,
    game: &GameState,
    player: PlayerId,
    spell_name: &str,
    x_count: u64,
    max_mana: u64,
) -> u64 {
    let ctx = ChoiceContext {
        kind: ChoiceKind::ChooseXValue {
            spell_name: spell_name.to_string(),
            x_count,
        },
    };
    dp.pick_number(game, player, &ctx, 0, max_mana)
}

// ... one function per decision type. Each is ~10-15 lines.
// Adding a new decision = one new function here + one new ChoiceKind variant.
```

### 3.5 Implementation Examples

**CLI (simple UI — 4 methods, ~40 lines total):**

```rust
impl DecisionProvider for CliDecisionProvider {
    fn pick_n(&self, _game: &GameState, _player: PlayerId,
              ctx: &ChoiceContext, options: &[ChoiceOption],
              bounds: (usize, usize)) -> Vec<usize> {
        println!("{}", ctx.prompt);
        for (i, opt) in options.iter().enumerate() {
            println!("  [{}] {:?}", i, opt);  // ChoiceOption derives Debug
        }
        if bounds.0 == bounds.1 {
            println!("(select exactly {})", bounds.0);
        } else {
            println!("(select {}-{})", bounds.0, bounds.1);
        }
        read_usize_list(options.len())  // existing helper
    }
    // pick_number, allocate, choose_ordering similarly simple
}
```

**Random (fuzz — 4 methods, ~30 lines total):**

```rust
impl DecisionProvider for RandomDecisionProvider {
    fn pick_n(&self, _game: &GameState, _player: PlayerId,
              _ctx: &ChoiceContext, options: &[ChoiceOption],
              bounds: (usize, usize)) -> Vec<usize> {
        let count = self.rng.borrow_mut().gen_range(bounds.0..=bounds.1);
        let mut indices: Vec<usize> = (0..options.len()).collect();
        indices.shuffle(&mut *self.rng.borrow_mut());
        indices.truncate(count);
        indices
    }
    // ...
}
```

**AI (optional context matching for specialized heuristics):**

```rust
impl DecisionProvider for AiDecisionProvider {
    fn pick_n(&self, game: &GameState, player: PlayerId,
              ctx: &ChoiceContext, options: &[ChoiceOption],
              bounds: (usize, usize)) -> Vec<usize> {
        match &ctx.kind {
            ChoiceKind::DeclareAttackers => {
                self.evaluate_attack(game, player, options)
            }
            ChoiceKind::ChooseTargets { .. } => {
                self.evaluate_targets(game, player, options)
            }
            _ => {
                // Fallback: pick randomly or use generic heuristic
                self.generic_pick(options, bounds)
            }
        }
    }
}
```

**Scripted (tests — `ChoiceKind`-aware queue, every expectation requires a kind):**

```rust
pub struct ScriptedDecisionProvider {
    queue: RefCell<VecDeque<ScriptedExpectation>>,
}

/// A single scripted expectation: what kind of decision we expect, and what to return.
pub struct ScriptedExpectation {
    /// The ChoiceKind we expect the engine to ask for (discriminant match, fields ignored).
    /// No `Any` fallback — every scripted decision must declare what it expects.
    pub expected_kind: ChoiceKind,
    pub response: ScriptedResponse,
}

pub enum ScriptedResponse {
    PickN(Vec<usize>),
    Number(u64),
    Allocation(Vec<u64>),
    Ordering(Vec<usize>),
}
```

Helper methods — each enqueues an expectation with a mandatory `ChoiceKind`:
```rust
impl ScriptedDecisionProvider {
    pub fn expect_pick_n(&self, kind: ChoiceKind, indices: Vec<usize>) { ... }
    pub fn expect_number(&self, kind: ChoiceKind, n: u64) { ... }
    pub fn expect_allocation(&self, kind: ChoiceKind, alloc: Vec<u64>) { ... }
    pub fn expect_ordering(&self, kind: ChoiceKind, order: Vec<usize>) { ... }
}
```

No `script_*` convenience methods without a kind. Every test must state what decision it expects — this is the entire point of the self-documenting design. If you're writing a test and don't know what `ChoiceKind` the engine will ask for, that's a signal to understand the engine flow better before writing the test.

When a DP method fires, it:
1. Pops the front of the queue (panics on empty: "unexpected DP call, no scripted response")
2. Asserts `discriminant(&incoming_kind) == discriminant(&expected_kind)` (variant match, payload fields ignored). Panic message includes both expected and actual kinds for clear diagnostics.
3. Asserts `response` variant matches the method (e.g., `PickN` for `pick_n`, panics on mismatch: "expected pick_n response but got Number")
4. Returns the scripted response

`Drop` impl asserts queue is empty — unconsumed expectations = test bug.

**Test readability:**
```rust
// Every decision is self-documenting — reads as a decision script
scripted.expect_pick_n(ChoiceKind::DeclareAttackers, vec![0, 2]);
scripted.expect_pick_n(ChoiceKind::ChooseTargets { .. }, vec![1]);
scripted.expect_number(ChoiceKind::ChooseXValue { .. }, 5);
```

---

## 4. Validation & Safety

### 4.1 Engine-Side Validation

Each `ask_*` function validates the DP's response before returning:
- **Bounds check:** Returned indices are within `0..options.len()`
- **Count check:** Number of selections is within `(min, max)`
- **Sum check (allocation):** Total distributed equals expected total
- **Minimum check (allocation):** Each bucket gets >= per_bucket_min
- **Ordering completeness:** Returned ordering is a valid permutation

Invalid responses result in a panic (debug builds) or re-prompt (release). The engine never receives an invalid response.

### 4.2 Exhaustiveness via Compiler Enforcement

`ChoiceKind` does NOT use `#[non_exhaustive]`. This is a single-crate project — there are no downstream consumers to protect. Exhaustive matching is a feature:
- When a new `ChoiceKind` variant is added, the compiler flags **every** match site that doesn't handle it
- This is strictly better than `#[non_exhaustive]` + `_ =>` catch-all, which silently hides unhandled cases
- If the project becomes multi-crate (engine lib + UI bins), revisit this for public-API enums at that time
- Same policy applies to all other enums in the codebase (see `design_doc.md` §11, 2026-04-14 entry)

### 4.3 Priority Action Uses `pick_n` (Confirmed)

Priority action selection is `pick_n` with `bounds: (1, 1)` over the list of legal actions. The engine enumerates all legal actions via existing oracle helpers (`castable_spells`, `activatable_abilities`, `playable_lands`), wraps them as `ChoiceOption::Action` variants, adds `Pass` (always legal) and `ProposeShortcut` (always legal — validated after selection, see §9.7), and calls `pick_n`.

Selecting a special action is just choosing one option from a finite list — there's nothing structurally different about it. The option space is always enumerable: the oracle helpers already exist and are already called every priority pass by both CLI and Random DPs. Moving enumeration into `ask_choose_priority_action` centralizes it and makes all DPs simpler (they no longer need their own oracle calls).

**Performance:** `castable_spells` does mana feasibility checks — O(hand_size × mana_sources). `activatable_abilities` is O(battlefield_size). `playable_lands` is O(hand_size). All trivially fast. Already computed every priority pass in the current implementation. No new performance cost.

---

## 5. Migration Plan

### 5.1 Scope

- Delete all 12 current typed methods from `DecisionProvider` trait
- Replace with 4 generic methods
- Create `ChoiceKind`, `ChoiceContext`, `ChoiceOption` enums in `ui/choice_types.rs`
- Create `ask_*` free functions in `ui/ask.rs` (~15 functions initially, one per existing decision point)
- Rewrite all 5 DP implementations to use 4 generic methods
- Update all engine call sites to use `ask_*` functions
- Update all tests to use scripted `pick_n_queue` etc.

### 5.2 File Changes

| File | Change |
|------|--------|
| `ui/choice_types.rs` | **NEW** — `ChoiceKind`, `ChoiceContext`, `ChoiceOption` |
| `ui/ask.rs` | **NEW** — typed `ask_*` free functions |
| `ui/mod.rs` | Add `pub mod choice_types, ask` |
| `ui/decision.rs` | Rewrite trait (4 methods), rewrite `ScriptedDecisionProvider` (4 queues), rewrite `DispatchDecisionProvider` (4 forwards), delete `PassiveDecisionProvider` (replace with `DefaultDecisionProvider` that picks first/zero/even) |
| `ui/cli.rs` | Rewrite `CliDecisionProvider` (4 methods) |
| `ui/random.rs` | Rewrite `RandomDecisionProvider` (4 methods) |
| `engine/cast.rs` | Replace direct DP calls with `ask_*` calls |
| `engine/priority.rs` | Replace `choose_priority_action` with `ask_choose_priority_action` |
| `engine/combat/steps.rs` | Replace attacker/blocker/damage DP calls |
| `engine/sba.rs` | Replace legend rule DP call |
| `engine/costs.rs` | Replace mana allocation DP call |
| `engine/resolve.rs` | Update DP call sites |
| All test files | Update scripted DP usage |

### 5.3 Estimated LOC

- New code: ~400 (types + ask functions)
- Deleted code: ~600 (typed methods across 5 impls)
- Modified code: ~200 (engine call sites + tests)
- **Net: ~0 LOC change**, roughly equivalent total

### 5.4 Risk

- **Medium.** This is a wide-but-shallow refactor. Every DP call site changes, but each change is mechanical (replace `dp.choose_X(...)` with `ask_choose_X(dp, ...)`). The logic doesn't change — only the dispatch mechanism.
- **Test safety:** All 385 existing tests continue to exercise the same game logic. Only the DP dispatch layer changes.

---

## 6. Comparison to Alternatives Considered

| Approach | Surface | Type safety | New decision cost | Impl burden |
|----------|---------|-------------|-------------------|-------------|
| **Typed methods (current)** | 12→100+ methods | Compile-time per-method | New trait method + 5 impls | O(impls × methods) |
| **Single `decide` method** | 1 method | Runtime (wrong shape = panic) | New enum variant | Match arm in every impl |
| **Two-tier trait (raw + simple)** | 100+ / 4 methods | Compile-time (both tiers) | New method + adapter entry | Adapter block is enormous |
| **4 generic methods + ask_\* (chosen)** | 4 methods | Compile-time at ask_\* boundary | New ChoiceKind + ask_\* fn | O(1) per impl |

---

## 7. Decision Record

**Decision:** Refactor `DecisionProvider` from typed methods to 4 generic primitives (`pick_n`, `pick_number`, `allocate`, `choose_ordering`) with `ChoiceContext` for semantic labeling and `ask_*` free functions for typed engine-side dispatch.

**Rationale:**
- MtG's decision space is too large for typed methods (~100+ final). The surface must be bounded.
- Simple UIs (CLI, Random) benefit from a 4-method surface. Complex impls (AI) benefit from `ChoiceContext` pattern matching for specialized heuristics.
- `ask_*` functions preserve typed engine call sites — readability and compile-time safety at the engine boundary are not sacrificed.
- `ChoiceKind` uses exhaustive matching (no `#[non_exhaustive]`) — compiler flags every match site when a variant is added.
- No performance impact. Context construction is trivial relative to DP evaluation logic.
- Eliminates the "silent default" problem: generic primitives have no sensible default, forcing every impl to actively handle requests.
- Network/serialization: `ChoiceContext`, `ChoiceOption`, and response types are trivially `#[derive(Serialize, Deserialize)]` with serde.

**Alternatives rejected:**
- Typed methods: doesn't scale to 100+ decisions
- Single `decide` enum: loses compile-time safety, shifts match complexity into every impl
- Two-tier trait: Rust orphan rules prevent clean blanket impl + direct impl coexistence; adapter struct is viable but the adapter block grows to hundreds of methods

---

## 8. Invariant Declarations

These invariants constrain the 4-method surface and must hold for the lifetime of the engine.

### 8.1 Completeness Invariant

> Every MtG decision can be decomposed into a finite sequence of `pick_n`, `pick_number`, `allocate`, and `choose_ordering` calls.

If a card introduces a decision that cannot be expressed as a composition of these four primitives, the card is added to the excluded cards list (see `design_doc.md` §11: "Explicit excluded cards list") rather than adding a 5th primitive. This is not expected to occur — the four shapes cover the full taxonomy of MtG decisions (see §8.6 below).

### 8.2 Atomicity Invariant

> Each DP call is a single atomic decision. The engine never expects the DP to maintain state between calls.

The `action_queue` pattern on DP implementations (e.g., `RefCell<VecDeque<PriorityAction>>` in `RandomDecisionProvider`) is DP-internal convenience for pre-planning multi-step sequences. The engine treats each `ask_*` call as independent. This means:
- DPs are free to be stateless (CLI reads input each time)
- DPs are free to be stateful (AI caches evaluation between calls)
- The engine can be serialized/deserialized between any two DP calls

### 8.3 Validity Invariant

> `ask_*` functions validate all DP responses before returning to the engine. The engine never receives an out-of-bounds index, a wrong-count selection, or a mismatched allocation sum.

Validation rules per primitive:
- **`pick_n`:** All returned indices in `0..options.len()`, no duplicates, count in `[min, max]`
- **`pick_number`:** Value in `[min, max]`
- **`allocate`:** Sum equals `total`, each bucket ≥ `per_bucket_min`, length matches `buckets.len()`
- **`choose_ordering`:** Returned vec is a valid permutation of `0..items.len()`

Invalid responses: panic in debug builds (`debug_assert!`), re-prompt or engine error in release.

### 8.4 Return Type Evolution Invariant

> `pick_number` may evolve from returning `u64` to returning `GameNumber` in Phase 9. This is a one-time type alias promotion, not a new method. All other return types (`Vec<usize>` for indices, `Vec<u64>` for allocation) are stable.

The `GameNumber` type progression:
- **Phase 7:** `type GameNumber = u64` (alias, zero-cost)
- **Phase 9:** `enum GameNumber { Finite(u64), Shortcut { id: LoopId, iterations: Box<GameNumber>, per_iteration: Box<GameNumber> }, Relative { base: Box<GameNumber>, multiplier: u64, offset: i64 } }`

`GameNumber` implements `PartialOrd` (not `Ord`) because two `Relative` values from unrelated loops are incomparable. Simple DPs return `GameNumber::Finite(n)`. The shortcut system constructs compositional variants. See §9 for full integration analysis.

### 8.5 Composition Invariant

> Complex decisions (multi-stage choices, conditional follow-ups) are expressed as sequential `ask_*` calls from the engine, not as a single compound DP call.

Examples:
- Cryptic Command (choose 2 modes, then choose targets per mode): `ask_choose_modes` → `ask_choose_targets` × N
- Fact or Fiction (reveal 5, opponent splits into 2 piles, caster picks a pile): `ask_pick_pile` (opponent) → `ask_pick_n` with 2 options (caster)
- Kicker with conditional target: `ask_choose_additional_costs` → conditionally `ask_choose_targets`

The engine orchestrates the sequence. The DP sees each step independently.

### 8.6 Coverage Audit

| Primitive | Shape | Covers |
|-----------|-------|--------|
| `pick_n` | Select N indices from M options, bounds (min, max) | Priority action, attackers, blockers, targets, discard, legend rule, sacrifice, modes, color, creature type, card name, counter type, accept/reject, loop break, Fact or Fiction piles (selected = pile A, rest = pile B), Thoughtseize-style revealed hand pick, Gifts Ungiven search |
| `pick_number` | Choose a value in [min, max] | X value, loop count declaration, "choose a number" effects (e.g. Menacing Ogre), `GameNumber` values in Phase 9 |
| `allocate` | Distribute total across buckets with per-bucket minimum | Combat damage assignment, trample (including over planeswalkers/battles — more buckets, same primitive), generic mana allocation, damage distribution (Arc Lightning), life distribution, +1/+1 counter distribution |
| `choose_ordering` | Return permutation of indices | Scry/surveil ordering, stack ordering (APNAP), cost payment order, triggered ability ordering, library ordering (e.g. Brainstorm put-back) |

**Edge cases verified:**
- **Partition (Fact or Fiction):** `pick_n` — selected indices = pile A, complement = pile B
- **Multi-group partition (hypothetical 3+ piles):** Sequential `pick_n` calls — "pick pile 1", "pick pile 2 from remainder", etc. No known MtG card requires 3+ piles in a single decision.
- **Boolean (Rhystic Study "pay {1}?"):** `pick_n` with `bounds: (1,1)` and 2 options (Yes, No)
- **Free-text input (choose a card name):** Uses `pick_number` with `CardRegistry` index — see §8.7. NOT `pick_n` with 27K+ options.
- **Multi-dimensional allocation (trample over planeswalkers + battles):** Single `allocate` with N+1 buckets (N blockers + defending entity). Tiered trample (over planeswalkers, over battles) just adds more buckets.
- **Tree/graph selection:** Does not exist in MtG's decision model. All choices are flat.

### 8.7 Card Name Selection: Performance-Aware Design

> "Choose a card name" effects (Pithing Needle, Meddling Mage, Cabal Therapy, etc.) require selecting from the full pool of legal MtG card names — currently ~27,000 unique names, growing with every set.

**Problem:** Passing `Vec<ChoiceOption::CardName(String)>` with 27K entries to `pick_n` is wasteful. Allocation of 27K strings per invocation, serialization over network, and UI rendering of the full list are all performance bottlenecks. Arena's half-second freeze on "name a card" effects is almost certainly this problem.

**Solution:** Card name selection uses `pick_number` with `ChoiceKind::NameCard`, NOT `pick_n` with enumerated options.

The engine maintains a `CardRegistry` (`Arc<CardRegistry>` on `GameState`, scaffolded in Phase 5-Pre). Card names map to integer IDs. The `ask_name_card` function:

```rust
pub fn ask_name_card(
    dp: &dyn DecisionProvider,
    game: &GameState,
    player: PlayerId,
) -> CardNameId {
    let registry = game.card_registry();
    let ctx = ChoiceContext { kind: ChoiceKind::NameCard };
    let id = dp.pick_number(game, player, &ctx, 0, registry.len() as u64 - 1);
    CardNameId(id as usize)
}
```

Each DP impl handles the lookup/UI locally:
- **CLI:** Text prompt with prefix-search/autocomplete against `game.card_registry()`. O(log N) binary search per keystroke.
- **Random:** `rng.gen_range(0..registry.len())`
- **AI:** Picks from a precomputed shortlist of strategically relevant names
- **Network/GUI:** Sends `ChoiceKind::NameCard` to the client. Client renders its own card name picker with local trie/fuzzy search. Only the integer ID returns over the wire.

**Performance:** O(1) for the DP call itself. The expensive part (search/autocomplete/rendering) is pushed to each DP impl where it can be optimized per-platform. The `CardRegistry` lives in `Arc` — no copying, shared read access.

**Why not `pick_n` with a curated shortlist?** Some "name a card" effects have restrictions ("name a nonland card") which would reduce the list, but many don't (Pithing Needle names *any* card). Even restricted lists can have 10K+ entries. The `pick_number` + registry pattern works uniformly for all cases — the `ask_name_card` function can accept an optional `filter: Option<fn(&CardData) -> bool>` and pass a filtered registry size as `max`.

**The `ChoiceOption::CardName(String)` variant still exists** for small curated lists (e.g., Gifts Ungiven searches 4 cards from a library — the engine passes 4 `CardName` options via `pick_n`). It is NOT used for unbounded "name any card" effects.

---

## 9. GameNumber & Loop Shortcut Integration

> Added 2026-04-13. Confirms that `GameNumber` (D26, rule 727/731) integrates smoothly with the 4-primitive DP surface. No tradeoffs or scope compromises required.

### 9.1 Why GameNumber Exists

MtG allows players to declare arbitrarily large loop iteration counts. The canonical example:
1. Astral Dragon ETB copies Parallel Lives (a token doubler)
2. Each copy doubles future token creation
3. Result: 2↑↑N tokens (tetration-scale numbers — digit count exceeds atoms in the observable universe)

A `u64` cannot represent these quantities. `GameNumber` is a symbolic arithmetic type that represents and compares such values without materializing them.

### 9.2 GameNumber Does Not Require a New DP Primitive

`GameNumber` is a **value type**, not a **decision shape**. It changes what `pick_number` returns, not how decisions are structured:

| Phase | `pick_number` returns | `GameNumber` type |
|-------|-----------------------|-------------------|
| Phase 7 (stub) | `u64` | `type GameNumber = u64` (alias) |
| Phase 9 (full) | `GameNumber` | `enum GameNumber { Finite(u64), Shortcut { .. }, Relative { .. } }` |

The Phase 7→9 transition is a type alias promotion. Simple DPs return `GameNumber::Finite(n)`. The shortcut system constructs compositional variants internally.

### 9.3 Loop Shortcuts Map to Existing Primitives

D26 (rule 727/731) introduces these DP interactions, all covered by existing primitives:

| Interaction | Primitive | ChoiceKind |
|-------------|-----------|------------|
| "How many iterations?" (731.1b, 731.2a) | `pick_number` | `DeclareLoopCount { loop_description }` |
| "Accept this shortcut?" (731.2b) | `pick_n` (2 options: accept/shorten) | `AcceptShortcut { proposer, description }` |
| "At what point do you interrupt?" (731.2b) | `pick_n` from breakpoints | `ShortenShortcut { proposer, breakpoints }` |
| "Break this loop — which action changes?" (731.3) | `pick_n` from legal alternatives | `BreakFragmentedLoop { loop_objects }` |

### 9.4 GameNumber Propagation Through the Engine

Once a `GameNumber` value enters the system (via `pick_number` → `ask_declare_loop_count`), it propagates through engine internals:
- **Token counts:** "Create N tokens" where N is a `GameNumber`
- **Life totals:** "Gain life equal to creatures you control" → `GameNumber` life gain
- **Damage:** "Deal damage equal to power" where power is a `GameNumber` from counter accumulation
- **Comparisons:** "Is my life total greater than yours?" → `GameNumber::partial_cmp` returns `Some(Ordering)` for related values, `None` for incomparable values from unrelated loops

All of this is engine-internal arithmetic. The DP is only involved at the point of choosing the iteration count.

### 9.5 PartialOrd, Not Ord

`GameNumber` implements `PartialOrd` because:
- `Finite(5) < Finite(10)` → `Some(Less)` ✅
- `Shortcut { iterations: 1_000_000, .. } > Finite(999_999)` → `Some(Greater)` ✅
- `Relative { base: A, multiplier: 2, offset: 0 }` vs `Relative { base: B, multiplier: 3, offset: 0 }` where A and B are from unrelated loops → `None` (incomparable) ✅

This is correct per MtG rules: two players declaring independent loop counts results in "my number" vs "your number" comparisons that the rules resolve via APNAP sequential declaration ordering (the non-active player declares last and can always declare "your number + 1", winning the race). The `Relative` variant enables exactly this: `Relative { base: opponent_number, multiplier: 1, offset: 1 }` = "whatever they said, plus one."

### 9.6 LoopDeclaration Type

The `ask_declare_loop_count` function in Phase 9 will accept `LoopDeclaration` responses:

```rust
pub enum LoopDeclaration {
    /// A concrete number of iterations
    Concrete(u64),
    /// "Match opponent's declaration + N"
    MatchPlusN { target_declaration: LoopId, offset: u64 },
}
```

The engine converts `LoopDeclaration` into `GameNumber` internally. The DP never constructs `GameNumber` directly — it declares intent, and the engine builds the symbolic representation. This keeps the DP interface clean while enabling full symbolic arithmetic in the engine.

### 9.7 DP-Initiated Loop Shortcut Protocol

> The standard engine flow is engine-driven: engine calls DP, DP responds. But loop shortcuts (rule 731) invert this — the *player* recognizes the loop pattern and proposes the shortcut. This section describes how the DP initiates shortcut proposals within the existing 4-method surface.

**The "who drives?" problem:** The engine can't know in advance that a deterministic loop exists. The player (DP) recognizes the combo and wants to say "I've been doing the same thing — execute it N times." But the engine is the one prompting the DP for input, not the other way around.

**Solution: `ProposeShortcut` as a priority action.**

```rust
pub enum PriorityAction {
    CastSpell(ObjectId),
    ActivateAbility(ObjectId, usize),
    PlayLand(ObjectId),
    PassPriority,
    /// Player claims a repeatable loop exists. Engine validates against delta log.
    ProposeShortcut(ShortcutProposal),
}

pub struct ShortcutProposal {
    /// The action sequence forming one loop iteration (for engine verification).
    pub loop_body: Vec<PriorityAction>,
    /// How many times to repeat (filled in after engine validates, via follow-up pick_number).
    /// None at proposal time; set by the DeclareLoopCount follow-up call.
    pub iterations: Option<LoopDeclaration>,
}
```

`ProposeShortcut` is always a legal priority action option — it appears alongside cast/activate/pass in every `pick_n(ChoosePriorityAction)` call. The DP constructs the proposal with a `loop_body` describing one iteration's action sequence.

**Full protocol (5 steps, all existing primitives):**

1. **DP proposes:** `pick_n(PriorityAction)` → DP returns index of `ProposeShortcut(proposal)`
2. **Engine validates:** Compares `proposal.loop_body` against recent delta log entries. If the last N deltas don't match the claimed loop body pattern, the engine rejects the proposal (returns to step 1 — DP must choose a different action). Validation uses action sequence matching, ignoring timestamps/object IDs that change between iterations.
3. **Engine asks iteration count:** `pick_number(DeclareLoopCount)` → DP returns iteration count (as `u64` in Phase 7, `LoopDeclaration` in Phase 9)
4. **Engine asks opponents:** `pick_n(AcceptShortcut)` → each opponent returns accept or shorten. If shorten: `pick_n(ShortenShortcut)` → opponent picks a breakpoint from the loop body.
5. **Engine fast-forwards:** Applies the loop's net state delta × iterations. Two strategies:
   - **Verify-then-multiply:** Execute one more iteration to confirm the delta matches, then multiply the net change symbolically.
   - **Full unroll with cap:** For loops that modify game state non-linearly (e.g., Astral Dragon doubling), unroll with `GameNumber` tracking. The engine doesn't materialize 2↑↑N tokens — it stores the count as a `GameNumber::Shortcut` value.

**Phased rollout:**
- **Phase 7 (safety net):** Engine-detected only. The engine runs loop detection on the delta log (periodic state hashing, forced-action counter). When it detects a mandatory loop (731.4 — all actions mandatory, no player choices), it declares a draw. Iteration cap (`MAX_TRIGGER_ITERATIONS`) prevents infinite execution. No DP proposal mechanism needed — this is purely a safety net.
- **Phase 9 (full):** Both DP-proposed (the protocol above) and engine-detected shortcuts. DP proposals are needed for AI/competitive play where the player recognizes the combo before the engine's heuristic does. Engine detection serves as both a fallback and a mandatory-loop-draw detector.

**Key safety property:** The DP cannot claim a loop that didn't actually happen. The engine's delta log validation (step 2) is the gatekeeper. A malicious/buggy DP that proposes a fake loop gets rejected — the engine simply returns to the priority action prompt.

**No new DP primitives required.** The proposal is a `PriorityAction` variant (returned from `pick_n`). The iteration count is `pick_number`. Opponent acceptance is `pick_n`. All within the existing 4-method surface.
