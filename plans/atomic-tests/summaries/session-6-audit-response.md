# Session 6 Audit Response — Design Discussions

> **Date:** 2026-04-06
> **Scope:** Design questions raised during the session-6 atomic test audit
> **Companion file:** `session-6.md` (atomic tests with inline audit notes)

---

## Table of Contents

1. [Dependency System as Its Own "Thing"](#1-dependency-system)
2. [611.2b — Multiplayer ETB Control Change Subtlety](#2-6112b-multiplayer)
3. [611.2c / 613.11 — Effect Discrimination: Permanent vs Game-Rule](#3-6112c-effect-discrimination)
4. [612.3 — Granted Abilities vs Text-Changing Effects](#4-6123-granted-abilities)
5. [612.8 — Name Restoration on Effect Expiry](#5-6128-name-restoration)
6. [613.7a — Timestamp Design for Granted Static Abilities](#6-6137a-timestamp-design)
7. [613.8b — Dependency Loop Detection Algorithm](#7-6138b-dependency-loops)
8. [614.12 / 614.13 — ETB Look-Ahead Design Hurdles](#8-61412-etb-look-ahead)
9. [616 — Replacement/Prevention Interaction Design](#9-616-replacement-prevention)
10. [616.1g — Outer/Inner Event Containment](#10-6161g-event-containment)
11. [616.2 — Replacement Effect Compositionality](#11-6162-compositionality)
12. [Cross-System Integration: How These Fit Together](#12-cross-system-integration)
13. [Trigger Classification: Event-Driven vs State-Based](#13-trigger-classification)
14. [Applicability System: What Does an Ability Affect?](#14-applicability-system)

---

## 1. Dependency System

**Question:** Should we handle the dependency system as its own "thing"? Hard to write tests that are meaningfully atomic.

### Analysis

The dependency system (613.8–613.8c) is fundamentally a **meta-algorithm** that operates on top of the layer system. It doesn't produce observable game state changes on its own — it only changes the *order* in which effects are applied within a layer/sublayer. This makes it structurally different from most testable rules.

### Recommendation

**Yes, treat it as a first-class subsystem** with its own module (`engine/layers/dependency.rs` or similar). Reasons:

1. **Isolatable logic:** The dependency check is a pure function: `(effects_in_layer, game_state) → ordered_effects`. It can be unit-tested independently of the full layer stack.
2. **Graph algorithm:** It's fundamentally a topological sort with cycle detection (Kahn's algorithm). This is well-understood and testable in isolation.
3. **Atomic tests are *integration* tests:** The ATOM tests in session-6.md (613.8a-001 Blood Moon+Urborg, 613.8b-001 circular fallback, 613.8c-001 re-evaluation) verify observable outcomes. The dependency module itself should have **unit tests** that verify:
   - Correct edge detection (condition a: same layer; condition b: changes text/existence/applies-to/effect; condition c: CDA guard)
   - Correct topological ordering
   - Cycle detection → timestamp fallback
   - Re-evaluation after each application

### Proposed API

```rust
/// Determine application order for effects within a single layer/sublayer.
/// Returns effects in the order they should be applied.
pub fn resolve_dependency_order(
    effects: &[ContinuousEffect],
    game_state: &GameState,
    layer: Layer,
) -> Vec<&ContinuousEffect> {
    // 1. Build dependency graph: for each pair (A, B), check if A depends on B
    // 2. Topological sort (Kahn's algorithm)
    // 3. If cycle detected among remaining nodes, sort those by timestamp
    // 4. Return ordered list
}

/// Check if effect A depends on effect B per 613.8a.
fn is_dependent(a: &ContinuousEffect, b: &ContinuousEffect, state: &GameState) -> bool {
    // (a) Same layer/sublayer — guaranteed by caller
    // (b) Applying B would change text/existence/applies-to/effect of A
    // (c) Neither is CDA, or both are CDAs
    ...
}
```

### Testing Strategy

- **Unit tests** on `resolve_dependency_order` and `is_dependent` with synthetic effects
- **Integration tests** via the ATOM tests (Blood Moon+Urborg, circular dependency, re-evaluation)
- **Fuzz tests** with randomly generated effect sets to verify no panics/infinite loops

---

## 2. 611.2b — Multiplayer ETB Control Change Subtlety

**Observation:** Consider a multiplayer scenario: Player A controls Master Thief, triggers ETB targeting Player B's artifact. Before resolution, Player C steals Master Thief. The triggered ability's controller is still Player A (controller at time of trigger generation), so when it resolves, Player A gains control of Player B's artifact — not Player C.

### Impact on Engine

This is already handled correctly if we follow the rule for "controller of a triggered ability" (see CR 603.3a — "The controller of a triggered ability on the stack is the player who controlled the ability's source at the time it triggered"). Our engine needs to stamp the controller on triggered abilities at trigger time, not look it up dynamically at resolution.

### Design Decision

- `TriggeredAbilityOnStack` must store `controller_at_trigger_time: PlayerId`
- Resolution uses this stored controller, not the current controller of the source permanent
- This is straightforward and aligns with the existing `ObjectRef` epoch-stamp model

### Test Note

ATOM-611.2b-001 already tests the "duration never starts" case. A new test should be added for the multiplayer control-change-before-resolution scenario when triggered abilities are implemented in Phase 7. Noted as a deferred test.

---

## 3. 611.2c / 613.11 — Effect Discrimination: Permanent vs Game-Rule

**Question:** How will the engine discriminate between effects that apply to permanents (characteristic-modifying) and effects that modify the game rules?

### Proposal: Marker on Ability Definition

Similar to CDAs, each continuous effect gets a classification marker at card registration time:

```rust
enum ContinuousEffectKind {
    /// Modifies object characteristics (P/T, color, type, abilities, name, control)
    /// Applied in Layers 1-7
    CharacteristicModifying,
    
    /// Modifies rules of the game ("creatures can't attack", "spells cost more")
    /// Applied after L1-L7 per 613.11
    GameRuleModifying,
    
    /// Cost modification (special sub-case of GameRuleModifying)
    /// Follows 601.2f pipeline: increases → reductions → Trinisphere
    CostModification { kind: CostModKind },
}

enum CostModKind {
    Increase,
    Reduction,
    Floor, // Trinisphere
}
```

### 4 Pillars Assessment

| Pillar | Assessment |
|--------|-----------|
| **Speed** | O(1) dispatch per effect — just check the enum variant. No runtime classification overhead. |
| **Correctness** | Classification is done at card registration, reviewed per card. Errors are caught in card-specific tests. |
| **Extensibility** | New effect kinds can be added as enum variants without changing the layer engine. |
| **Maintainability** | Single source of truth per card. The marker is adjacent to the effect definition, making it easy to audit. |

### How It Works

1. **Card registration:** When a card is loaded, each static/continuous ability is tagged with its `ContinuousEffectKind`.
2. **Layer engine:** Processes `CharacteristicModifying` effects through L1-L7 in the normal layer/sublayer/timestamp/dependency pipeline.
3. **Post-layer:** After L1-L7 completes, processes `GameRuleModifying` effects in timestamp order.
4. **Cost pipeline:** `CostModification` effects are processed during 601.2f in the order: increases → reductions → floors.

### Lock-in Implications (611.2c)

For resolving spell/ability effects:
- `CharacteristicModifying`: affected set locked at resolution (611.2c first clause)
- `GameRuleModifying`: affected set NOT locked — applies dynamically (611.2c second clause)
- Mixed effects: each part classified independently (ATOM-611.2c-003)

---

## 4. 612.3 — Granted Abilities vs Text-Changing Effects

**Question:** Text-changing effects in L3 should NOT modify abilities granted by external effects. This throws a wrench in treating text-changing as simple ability/characteristic modification in a different layer.

### Problem

If we naively apply L3 text-changing to all text on a permanent, it would incorrectly modify externally-granted abilities. Per 612.3, text-changing should only affect the object's own printed text (and copy-derived text), not text from external grants.

### Proposed Solution: Demarcate Granted Abilities

Each ability on a permanent carries an `origin` tag:

```rust
enum AbilityOrigin {
    /// Part of the card's printed text (or copy-derived text)
    Intrinsic,
    /// Granted by an external continuous effect
    Granted { source_id: ObjectId, effect_timestamp: Timestamp },
}
```

**L3 text-changing effects only process abilities with `AbilityOrigin::Intrinsic`.**

### Alternative Considered: Separate Text Storage

Store "own text" and "granted text" in separate fields. Rejected because:
- Complicates the ability lookup API (callers need to merge two lists)
- Doesn't naturally handle the case where a granted ability is later removed (need to track in both places)
- The origin tag approach is simpler and more general

### Assessment

The `AbilityOrigin` tag approach is:
- **Correct:** Faithfully implements 612.3
- **Extensible:** The origin tag is useful for other rules too (e.g., "remove all abilities granted by Auras")
- **Minimal overhead:** One extra enum field per ability instance

---

## 5. 612.8 — Name Restoration on Effect Expiry

**Question:** If a name-setting effect expires, how does the engine restore the old name?

### Answer: Layer System Handles This Automatically

The layer system (613.5) continuously re-evaluates from scratch:

1. Start with printed/copiable values (including the printed name)
2. Apply all active continuous effects in layer order
3. If the name-setting effect is no longer active (expired, source left), it's simply not in the effect list
4. Result: the name reverts to the printed value

**No special "undo" or "restore" mechanism is needed.** This is one of the core benefits of the "recalculate from scratch" approach to the layer system.

### Implementation Note

This means the engine must NOT mutate the base characteristics in-place. Instead:
- `GameObject.printed_characteristics` — immutable (set at creation/copy)
- `oracle::characteristics::get_effective_name(state, obj_id)` — computed by running the full layer pipeline

This aligns with the existing `oracle/` module architecture.

---

## 6. 613.7a — Timestamp Design for Granted Static Abilities

**Question:** How will the engine apply 613.7a properly? (Static ability timestamp = later of object's timestamp vs the effect that created the ability.)

### Proposal

Each `ContinuousEffect` on an object stores:

```rust
struct ContinuousEffect {
    /// Timestamp of the object that generates this effect
    object_timestamp: Timestamp,
    /// Timestamp of the external effect that granted this ability (if any)
    grant_timestamp: Option<Timestamp>,
    // ...
}

impl ContinuousEffect {
    fn effective_timestamp(&self) -> Timestamp {
        match self.grant_timestamp {
            Some(gt) => std::cmp::max(self.object_timestamp, gt),
            None => self.object_timestamp,
        }
    }
}
```

When a Rune of Flight (Aura) grants "Equipped creature has flying" to an Equipment:
- The Equipment's `object_timestamp` is T_hammer
- The `grant_timestamp` is T_rune
- `effective_timestamp()` = max(T_hammer, T_rune) = T_rune (if Rune entered later)

When the Equipment gets a new timestamp (e.g., re-attached per 613.7e):
- `object_timestamp` updates to T_new
- `grant_timestamp` stays T_rune
- All effects preserve their relative order per the second sentence of 613.7a

### Edge Case: Re-Timestamp Propagation

Per 613.7a second sentence: "If the ability-granting effect has the later timestamp and the object receives a new timestamp, that object's old static abilities and the granted one all receive new timestamps at the same time."

### ⚠️ APNAP Tension (Audit feedback)

The sub-ordering index approach ("all effects get T_new but preserve relative order") creates a tension with the APNAP rule for simultaneous timestamps (613.7d). **Clarification:**

- **APNAP (613.7d)** governs ordering between **different objects** that receive timestamps simultaneously (e.g., two permanents entering at once under different players' control). Active player's objects get earlier timestamps.
- **Sub-ordering (613.7a)** governs ordering of **effects on the same object** that all receive a new timestamp at the same moment. These are all controlled by the same player, so APNAP doesn't apply.

The two systems don't conflict — they operate at different scopes:

| Scope | Mechanism | Example |
|-------|-----------|--------|
| Same object, multiple effects | Sub-ordering index (preserve relative order) | Rune of Flight + Colossus Hammer on same creature |
| Different objects, simultaneous | APNAP | Two creatures ETB at same time from different players |

**Implementation:** `Timestamp` becomes a composite: `(global_sequence: u64, sub_index: u16)`. The `global_sequence` participates in APNAP ordering. The `sub_index` preserves intra-object relative order and is only compared when `global_sequence` values are equal AND the effects are on the same object.

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Timestamp {
    /// Global monotonic sequence number. APNAP ordering assigns
    /// active player's objects earlier sequence numbers.
    global_seq: u64,
    /// Intra-object sub-ordering. Only meaningful when comparing
    /// effects on the same object with the same global_seq.
    sub_index: u16,
}
```

`Ord` derivation gives us correct lexicographic comparison: `global_seq` first, then `sub_index`. This is correct because effects on different objects will have different `global_seq` values (APNAP assigns them), and effects on the same object use `sub_index` to break ties. Still not *elegant* — it's an inherent complexity cost of 613.7a — but it's correct and the `Ord` derive makes it invisible at call sites.

---

## 7. 613.8b — Dependency Loop Detection Algorithm

**Question:** Circular dependency detection and handling.

### Algorithm: Kahn's Topological Sort

```
function resolve_order(effects, state):
    // Build directed graph: edge A→B means "A depends on B" (B must apply first)
    graph = build_dependency_graph(effects, state)
    
    // Kahn's algorithm
    in_degree = compute_in_degrees(graph)
    queue = effects where in_degree == 0, sorted by timestamp
    result = []
    
    while queue is not empty:
        next = queue.pop_front()  // earliest timestamp among ready nodes
        result.push(next)
        for each neighbor of next:
            in_degree[neighbor] -= 1
            if in_degree[neighbor] == 0:
                insert neighbor into queue (sorted by timestamp)
    
    // Remaining nodes (in_degree > 0) form cycles
    cycle_nodes = effects not in result, sorted by timestamp
    result.extend(cycle_nodes)
    
    return result
```

### Key Properties

1. **Non-circular dependencies:** Resolved correctly by topological order
2. **Circular dependencies:** Detected as nodes with remaining in-degree > 0; fall back to timestamp (613.8b)
3. **Re-evaluation (613.8c):** After applying each effect, rebuild the graph for remaining effects and re-run. This is O(n²) in the worst case but n is typically very small (< 20 effects in a layer).

### Complexity

- Typical case: 2-5 effects per layer → trivial
- Worst case: ~20 effects (heavily animated board) → still fast (microseconds)
- No need for caching across evaluations — the full recalculation is cheap enough

---

## 8. 614.12 / 614.13 — ETB Look-Ahead Design Hurdles

**Question:** These represent significant design hurdles for representing complex board states. Need additional design consideration.

### The Problem

When a permanent is entering the battlefield, replacement effects need to evaluate it **as it would exist on the battlefield** — before it's actually there. This requires:

1. **Look-ahead evaluation (614.12):** Run the layer system on a hypothetical version of the permanent that includes:
   - The permanent's own static abilities **that affect only itself** (see Self-Only Qualifier below)
   - All existing continuous effects that would apply to it
   - But NOT the permanent's own effect on other permanents (it's not there yet)
   - Already-applied replacement modifications (counters, tapped status, etc.)

2. **Auxiliary zone changes (614.13):** The entering permanent's ETB replacement may sacrifice/exile other objects as part of entering. These zone changes happen during the replacement, before the permanent is officially on the battlefield.

### ⚠️ The Self-Only Qualifier (614.12 — missed in initial analysis)

The CR is explicit: ETB replacement effects "may come from the permanent itself **if they affect only that permanent** (as opposed to a general subset of permanents that includes it)."

This means:
- **"This creature enters tapped"** (Scarwood Treefolk) → affects only itself → **included** in look-ahead
- **"Permanents enter tapped"** (Orb of Dreams) → general subset → **excluded** from its own look-ahead (Orb enters untapped)
- **"Creatures you control enter with a +1/+1 counter"** (Master Biomancer) → general subset → applies to OTHER creatures entering, NOT to Master Biomancer itself

**Implementation: Derive from existing ability targeting (no new enum)**

~~Previously proposed `EtbReplacementScope { SelfOnly, GeneralSubset }` enum.~~ On reflection, this is unnecessary — the distinction is already captured by the ability's targeting/applicability definition, which the card definition system must encode anyway:

- An ability that references `Target::Self` / `Applicability::This` → affects only itself → **self-only**
- An ability that references `Applicability::Matching(predicate)` → category-based → **general subset**

The check during look-ahead becomes:

```rust
// During look-ahead, when collecting effects from the entering permanent:
fn is_self_only_etb(ability: &Ability) -> bool {
    // Self-referencing abilities: "this creature enters tapped",
    // "As ~ enters, choose a color", "~ enters with N counters"
    ability.is_etb_replacement() && ability.applicability().is_self_referencing()
}
```

`is_self_referencing()` is derivable from the targeting/applicability data that already exists on every ability definition. This same distinction is needed for:
- **612.3:** text-changing effects only modify intrinsic text (self-referencing vs granted)
- **614.12:** look-ahead inclusion (self-only vs general subset)
- **Continuous effect generation:** self-only effects generate from the object, general effects generate from the controller's board presence

So rather than a new enum, we get this behavior from the existing `Applicability` type that every ability already carries. Card definitions don't get more complex.

During look-ahead:
- Abilities from the entering permanent where `is_self_only_etb()` → **included**
- Abilities from the entering permanent where NOT `is_self_only_etb()` → **excluded** (the permanent isn't on the battlefield yet to generate category-based effects)
- All abilities from *other* permanents already on the battlefield → **included** (they already exist)

### Proposed Architecture

```rust
/// Evaluate a permanent's characteristics as it would exist on the battlefield
/// without actually placing it there. Used for 614.12 look-ahead.
fn evaluate_look_ahead(
    state: &GameState,
    entering_object: &GameObject,
    already_applied_replacements: &[ReplacementEffect],
) -> LookAheadCharacteristics {
    // 1. Create a temporary "phantom" entry in the battlefield
    // 2. Collect applicable effects:
    //    a. entering_object's own SelfOnly static abilities
    //    b. all existing continuous effects from OTHER permanents
    //    c. already-applied replacement modifications (counters, etc.)
    //    d. EXCLUDE: entering_object's GeneralSubset abilities
    //    e. EXCLUDE: entering_object's effects on OTHER permanents
    // 3. Run layer system on the phantom
    // 4. Return computed characteristics
    // 5. Remove phantom entry
}
```

### Auxiliary Zone Changes (614.13) — Dispatch Pattern

The `process_etb_auxiliary_zone_changes` function is a **passthrough dispatch** — a match on the replacement effect type, with each arm handling the specific zone-change logic for that mechanic:

```rust
/// Process ETB replacement that causes auxiliary zone changes.
/// Dispatches on replacement effect type.
fn process_etb_auxiliary_zone_changes(
    state: &mut GameState,
    entering_object: &mut GameObject,
    replacement: &ReplacementEffect,
    simultaneous_entries: &[ObjectId],  // 614.13a: can't choose these
    already_chosen: &mut HashSet<ObjectId>,  // 614.13b: can't re-choose
) -> Vec<ZoneChange> {
    match &replacement.kind {
        EtbReplacement::Devour { devour_n } => {
            // Present sacrifice choices, validate, execute, add counters
            let sacrificed = choose_devour_sacrifices(
                state, entering_object, *devour_n,
                simultaneous_entries, already_chosen,
            );
            already_chosen.extend(sacrificed.iter().map(|zc| zc.object_id));
            sacrificed
        }
        EtbReplacement::Exploit => {
            // "You may sacrifice a creature"
            let sacrificed = choose_exploit_sacrifice(
                state, entering_object,
                simultaneous_entries, already_chosen,
            );
            // ... 
            sacrificed
        }
        EtbReplacement::ShockLand { life_cost } => {
            // "Pay 2 life or enter tapped" — no zone changes,
            // but interacts with Ashiok (614.13c)
            vec![]
        }
        // ... other ETB replacement types
    }
}
```

Each arm handles:
1. Present choices to player (via DP)
2. Validate against 614.13a (can't choose entering/simultaneous objects)
3. Validate against 614.13b (can't re-choose across multiple ETB replacements)
4. Execute zone changes
5. Update entering_object based on results

This is extensible — adding a new ETB replacement type means adding a new match arm.

### Simultaneous Entry: Living Death and Mass ETB

When N permanents enter simultaneously (Living Death, Collected Company, etc.):

1. **Each gets its own independent look-ahead** against the *pre-entry* board state. Creature A's look-ahead does NOT see creatures B–N (none of them are on the battlefield yet).
2. **Entering creatures can't affect each other's entry.** If one of the 10 is a Humility, its continuous effect doesn't exist during look-ahead — it's not on the battlefield yet. Only effects from permanents that already existed pre-entry are included.
3. **"Can't enter" effects must already exist.** Grafdigger's Cage must be on the battlefield *before* Living Death resolves. An entering creature can't generate a "can't enter" effect that blocks its siblings.
4. **Algorithm:** For each of the N entering permanents, run `evaluate_look_ahead(current_board_state, permanent_i)` independently. O(N × layer_cost). Look-aheads don't interact — they could conceptually run in parallel (though we'd execute sequentially for simplicity).
5. **616.1 ordering still applies** for choosing which replacement effects to apply to each entering permanent, but the look-ahead inputs are independent.

### Stack-to-Battlefield Zone Transition

The look-ahead input is **the spell object as it currently exists on the stack**, not the printed card text. By the time we reach the "move to battlefield" step:

1. The spell has already been affected by any stack-targeting continuous effects (text-changing in L3, ability-granting in L6, etc.) — these modifications are baked into the `GameObject` on the stack.
2. The `GameObject` carries these modifications forward into the phantom entry.
3. Stack-specific continuous effects ("Spells cost {1} more") naturally drop out during look-ahead because the layer engine is zone-aware — it evaluates for the battlefield zone, not the stack zone.
4. Effects that granted abilities to the spell on the stack: the ability is already part of the `GameObject`'s ability list, so it persists through the zone change.

No special handling needed — the layer engine's zone filtering handles this. The key invariant: **look-ahead starts from `entering_object` (the spell's current `GameObject`), not from the card's printed definition.**

### Open Questions

- **Performance:** Look-ahead runs the full layer system. For mass ETB (N=10), this is 10 × layer_cost. Mitigation: layer engine results for the static board state can be cached and reused across all N look-aheads (only the phantom entry varies).
- **Circular references:** A permanent's look-ahead sees existing effects, but those effects may themselves depend on what's entering. Resolved by the rule: entering permanents don't generate effects yet, so no circularity.
- **614.13c (Ashiok):** When an ETB replacement causes milling/exile from library, the entering card itself is excluded from that effect even though it's technically still in the library at that moment. The dispatch needs to know which card is "in transit."

---

## 9. 616 — Replacement/Prevention Interaction Design

**Question:** Any major design considerations for the prevention/replacement interactions?

### Key Design Decisions

1. **Unified replacement pipeline:** Both replacement and prevention effects go through the same pipeline. The distinction is semantic (replacement modifies the event, prevention reduces/eliminates damage), but the ordering/application mechanism is identical.

2. **Priority ordering (616.1a-e):** The engine needs a priority system:
   ```
   Priority 1: Self-replacements (616.1a)
   Priority 2: Control-changing replacements (616.1b) — ETB only
   Priority 3: Copy-becoming replacements (616.1c) — ETB only
   Priority 4: Back-face-up replacements (616.1d) — ETB only, deferred
   Priority 5: All others (616.1e) — player choice
   ```

3. **Iterative application (616.1f):** After applying one replacement, re-check for applicable replacements on the modified event. This is a loop that terminates because:
   - Each replacement applies at most once (614.5)
   - The set of applicable replacements can only shrink or change (not grow unboundedly)

4. **"Can't" override (614.17 / META-101.2):** Not a replacement, but needs to be checked in the same pipeline. "Can't" effects are checked before replacement application and override any "can" permission.

### Proposed Pipeline

```
function apply_replacements(event):
    applied = {}
    loop:
        applicable = find_applicable_replacements(event) - applied
        if applicable is empty: break
        
        // Check "can't" effects first
        if event is blocked by "can't": return null_event
        
        // Priority ordering per 616.1a-e
        chosen = select_by_priority(applicable, event.affected_controller)
        
        // Apply chosen replacement
        event = chosen.apply(event)
        applied.add(chosen)
        
        // Check for newly applicable replacements (616.2)
        // Loop continues
    
    return event
```

---

## 10. 616.1g — Outer/Inner Event Containment

**Question:** How does the engine distinguish "outer" vs "inner" events?

### Proposal: Event Hierarchy via Decomposition

Events have a containment relationship, but **the inner events are not stored inside the outer event**. Instead, the outer event is pure data, and a `decompose` step *creates* the inner events after outer-level replacements have been applied.

```rust
/// TokenDef is pure data — characteristics, abilities, etc.
/// It does NOT contain GameEvents.
struct TokenDef {
    name: String,
    colors: ColorSet,
    types: TypeLine,
    power: Option<u32>,
    toughness: Option<u32>,
    abilities: Vec<Ability>,
}

enum GameEvent {
    CreateTokens { count: usize, token_def: TokenDef },
    EnterBattlefield { object: ObjectId },
    DealDamage { source: ObjectId, target: DamageTarget, amount: u32 },
    // ...
}
```

A `CreateTokens` event contains a `count` and a `TokenDef`. After outer replacements (e.g., Doubling Season changes count from 1 to 2), the engine **creates** N `EnterBattlefield` events — they are generated by the decomposition logic, not unwrapped from a nested struct.

### Why NOT nest events

If `TokenDef` contained `Vec<GameEvent>`, you'd get:
- Recursive event types that are hard to pattern-match for replacement effects
- Replacement effects needing to "reach inside" nested events
- Difficult-to-debug scenarios where inner events are modified before the outer event's replacements run
- Ownership/lifetime headaches with mutable references into nested structures

### Implementation

```rust
fn process_event(state: &mut GameState, event: GameEvent) -> EventResult {
    // 1. Apply replacements to the outer event
    let modified = apply_replacements(state, event);
    
    // 2. Execute the outer event and decompose into sub-events.
    //    decompose() CREATES new events; it doesn't unwrap them.
    //    CreateTokens { count: 2, def } → create 2 token objects →
    //    emit [EnterBattlefield { obj_1 }, EnterBattlefield { obj_2 }]
    let sub_events = execute_and_decompose(state, modified);
    
    // 3. Process each sub-event (which may have its own replacements)
    for sub in sub_events {
        process_event(state, sub);  // recursive
    }
}

/// Decomposition logic: knows which event types produce sub-events.
fn execute_and_decompose(state: &mut GameState, event: GameEvent) -> Vec<GameEvent> {
    match event {
        GameEvent::CreateTokens { count, token_def } => {
            // Create N token game objects (not yet on battlefield)
            let token_ids: Vec<ObjectId> = (0..count)
                .map(|_| state.create_token_object(&token_def))
                .collect();
            // Each token needs to enter the battlefield
            token_ids.into_iter()
                .map(|id| GameEvent::EnterBattlefield { object: id })
                .collect()
        }
        GameEvent::EnterBattlefield { object } => {
            // Terminal event — actually place on battlefield
            state.move_to_battlefield(object);
            vec![]
        }
        // ... other event types
    }
}
```

This naturally handles 616.1g nesting: outer replacements (Doubling Season) fire on `CreateTokens`, inner replacements (Voice of All's "choose a color") fire on each `EnterBattlefield`. The recursion handles arbitrary depth, and each level is a flat `GameEvent` — no nested event trees to debug.

---

## 11. 616.2 — Replacement Effect Compositionality

**Question:** Does the replacement effect system need to be compositional?

### Answer: Yes, inherently

Rule 616.2 explicitly states that a replacement can **become applicable** as a result of another replacement modifying the event. This means the system must:

1. After each replacement application, re-scan for newly applicable replacements
2. A replacement that watches for event type X can fire if a previous replacement changed the event into type X
3. Example: "gain life → draw cards" then "draw cards → return from graveyard" is a two-step chain

### Design Implication

The iterative loop in the pipeline (Section 9 above) already handles this. The key is that `find_applicable_replacements` is called **after every application**, not just once at the start. This is the compositionality.

### Termination Guarantee

The loop terminates because:
- Each replacement applies at most once per event (614.5)
- The total number of replacement effects is finite
- Therefore the loop runs at most N iterations (N = number of replacement effects in the game)

In practice, chains rarely exceed 2-3 links.

---

## 12. Cross-System Integration: How These Fit Together

> **Cross-reference:** `state-tracking-architecture.md` covers the delta log, loop detection, and trigger tracking systems. This section maps how the session-6 systems (layers, replacements, events) connect to those.

### The Core Pipeline

Every game action flows through this pipeline:

```
Action
  → Replacement pipeline (614/615/616) — may modify, prevent, or redirect
    → Event execution — mutates GameState
      → Delta log emission (state-tracking-architecture.md) — records what changed
        → Layer system recalculation (613) — recomputes effective characteristics
          → State trigger scan (603.8) — checks deltas for trigger conditions
            → SBA check — uses oracle/ for effective characteristics
```

### Where Each System Lives

| System | Module | Inputs | Outputs |
|--------|--------|--------|---------|
| **Layer engine** | `engine/layers/` | Active continuous effects, GameState | Effective characteristics (via oracle/) |
| **Dependency resolver** | `engine/layers/dependency.rs` | Effects in a layer | Ordered application sequence |
| **Replacement pipeline** | `engine/replacements/` | GameEvent + active replacement/prevention effects | Modified GameEvent (or null) |
| **Event decomposition** | `engine/events/` | Resolved outer GameEvent | Vec of inner GameEvents |
| **Delta log** | `engine/delta_log.rs` (from state-tracking-arch) | Executed GameActions | Structured delta entries |
| **Trigger scanner** | `engine/triggers/` (Phase 7) | Delta log entries | Pending triggered abilities |
| **Loop detector** | `engine/loop_detect.rs` (from state-tracking-arch) | Forced-action counter + state hashes | LoopStatus |

### Key Integration Points

**1. Replacement pipeline → Delta log**

When a replacement modifies an event, the delta log must record **what actually happened**, not the original event:
- Original: "deal 3 damage to Player A"
- After replacement: "deal 6 damage to Player A" (doubled)
- Delta log records: `DamageDealt { amount: 6, ... }`
- Trigger scanner sees 6, not 3

If a replacement completely replaces an event (exile instead of die), the original event type **never appears** in the delta log. Only the replacement event does. This is how 614.6 ("the original event never happens") integrates with trigger tracking.

**2. Layer recalculation → Trigger scanner**

After any game action, the layer system recalculates. If characteristics change as a result (e.g., creature's toughness drops to 0), this may:
- Generate new SBA checks (lethal damage, 0 toughness)
- Generate state-trigger conditions ("whenever a creature has power 4 or greater")

The delta log captures the characteristic change; the trigger scanner reads it.

**3. ETB look-ahead → Layer engine**

`evaluate_look_ahead` calls the layer engine in a read-only mode (phantom entry, no delta emission). This is the only place where the layer engine runs without emitting deltas.

**4. Event decomposition → Replacement pipeline (recursive)**

`process_event` is recursive: outer event → replacements → execute → decompose → inner events → replacements → execute → ... Each level emits its own deltas.

**5. Loop detection → Replacement pipeline**

The forced-action counter (Tier 1 loop detection from state-tracking-architecture.md) integrates with the replacement pipeline: if a replacement effect causes a mandatory loop (e.g., replacement creates a trigger that creates another replacement), the counter increments. The replacement pipeline itself doesn't need to know about loop detection — it's checked at the priority-pass level.

### Shared Infrastructure

Both systems need:
- `GameState` with `#[derive(Hash, PartialEq)]` — for loop detection (Tier 2/3) and D26 shortcut validation
- `GameAction` / `GameDelta` enum — delta log entries that both trigger scanning and shortcut validation consume
- `Timestamp` composite type — used by layer engine, dependency resolver, and replacement ordering
- `ObjectId` / `ObjectRef` epoch-stamp — used everywhere for object identity across zone changes

### Phase Ordering

These systems build on each other:

```
Phase 5: Layer engine + dependency resolver + oracle/ routing
         (foundation — everything else reads effective characteristics from here)
              ↓
Phase 6: Replacement pipeline + event decomposition + ETB look-ahead
         (uses layer engine for look-ahead; modifies events before execution)
              ↓
Phase 7: Delta log + trigger scanner + loop detector (Tier 1)
         (records what Phase 6 produced; scans for trigger conditions)
              ↓
Phase 9: Loop detection Tier 2/3 + D26 shortcuts
         (uses full-state hash + delta transcripts from Phase 7)
```

The key insight: **Phase 5 (layers) and Phase 6 (replacements) are the load-bearing walls.** If these are correct, Phase 7 trigger tracking is straightforward (scan deltas), and loop detection is an orthogonal concern that reads from the same data.

---

## 13. Trigger Classification: Event-Driven vs State-Based

> **Cross-reference:** `state-tracking-architecture.md` §603.8 and the delta log design. This section defines the two trigger categories and how they're classified at card registration time.

### Two Categories, Same Scan Point

Both trigger types are scanned at the same point in the pipeline (after event execution + delta emission), but they match against different things:

| Type | CR Basis | Matches Against | Example |
|------|----------|----------------|---------|
| **Event-driven** | 603.1 | "Did event X just happen?" — pattern-match most recent deltas | "Whenever a creature enters the battlefield" |
| **State-based** | 603.8 | "Did condition Y become true?" — evaluate condition, compare to previous | "Whenever you have no cards in hand" |

Both are checked after every game action. The difference is what they inspect:
- **Event-driven:** Scan the most recent batch of deltas for matching `GameAction` patterns. No historical comparison needed.
- **State-based:** Evaluate a boolean condition against the current `GameState`. Fire only if the condition transitioned from false→true (detected by comparing against previous evaluation or by finding a delta that could have caused the transition).

### Card Registration Classification

At card definition time, each triggered ability is tagged:

```rust
enum TriggerKind {
    /// "Whenever [event]" — fires when a matching delta appears.
    /// The pattern describes which GameAction variants to match.
    EventDriven {
        watches_for: GameActionPattern,
    },
    
    /// "Whenever [condition is true]" / "At [time], if [condition]" —
    /// fires when the condition transitions to true.
    StateBased {
        condition: TriggerCondition,
    },
}
```

`GameActionPattern` is a predicate over `GameAction` variants:
```rust
enum GameActionPattern {
    ZoneTransfer { to: Option<Zone>, from: Option<Zone>, object_filter: Option<ObjectFilter> },
    LifeChanged { player_filter: Option<PlayerFilter>, direction: Option<LifeDirection> },
    DamageDealt { source_filter: Option<ObjectFilter>, target_filter: Option<DamageTargetFilter> },
    SpellCast { spell_filter: Option<ObjectFilter> },
    // ... other patterns matching GameAction variants from delta log
}
```

`TriggerCondition` is a function-like predicate over `GameState`:
```rust
enum TriggerCondition {
    /// "Whenever you have no cards in hand"
    HandEmpty { player: PlayerRef },
    /// "Whenever a creature you control has power 4 or greater"
    PermanentMatches { filter: ObjectFilter, condition: CharacteristicPredicate },
    /// Generic fallback for unusual conditions
    Custom(fn(&GameState, PlayerId) -> bool),
}
```

### How the Scanner Works

`scan_triggers` is a **building block** called by the game loop at specific points — it doesn't drive the game loop itself. See "Trigger Lifecycle" below for when it's called.

```rust
/// Scan recent deltas for triggers that should fire.
/// Called by the game loop after each resolution and after SBAs.
/// Returns NEW triggers to put on the stack; does NOT touch the stack itself.
fn scan_triggers(
    state: &GameState,
    recent_deltas: &[GameDelta],
    active_triggers: &[RegisteredTrigger],
) -> Vec<PendingTrigger> {
    let mut pending = vec![];
    
    for trigger in active_triggers {
        match &trigger.kind {
            TriggerKind::EventDriven { watches_for } => {
                for delta in recent_deltas {
                    if watches_for.matches(&delta.action) {
                        pending.push(PendingTrigger::new(trigger, delta));
                    }
                }
            }
            TriggerKind::StateBased { condition } => {
                // With Approach B: state triggers also scan deltas,
                // looking for transition deltas rather than current state.
                // See "Delta Requirements" below.
                for delta in recent_deltas {
                    if condition.matches_transition(delta) {
                        pending.push(PendingTrigger::new_state(trigger));
                    }
                }
            }
        }
    }
    
    pending
}
```

### Trigger Lifecycle: Stack Interleaving

`scan_triggers` does NOT lose triggers on the stack. Triggers already on the stack are `TriggeredAbilityOnStack` objects — they're part of the stack, not the scanning system. The lifecycle is:

```
1. Something resolves (spell, ability, or trigger)
   → Deltas emitted during resolution
   → Layer recalculation

2. SBA check loop:
   a. Check state-based actions (lethal damage, 0 toughness, etc.)
   b. If any SBAs performed → new deltas emitted → loop back to 2a
   c. When no SBAs remain → proceed

3. Trigger scan:
   a. scan_triggers(all_deltas_since_last_scan) → Vec<PendingTrigger>
   b. For simultaneous triggers: order by APNAP (603.3b)
   c. Place on stack (controller chooses order among their own)
   d. Clear the delta batch for this scan

4. Priority pass:
   a. Active player gets priority
   b. If they pass, NAP gets priority
   c. If all pass on empty stack → next phase/step
   d. If anyone acts (cast/activate) → go to 1 when it resolves
   e. If all pass with non-empty stack → top of stack resolves → go to 1
```

The critical point: **step 3 happens after EVERY resolution**, not just once. So:

- Trigger A is on the stack, resolves → step 1 emits deltas
- Step 3 scans those deltas → finds Trigger B → puts B on stack (on top)
- Step 4: priority passes → Trigger B resolves → step 1 emits deltas
- Step 3 scans → maybe Trigger C fires → and so on

Triggers already on the stack from earlier are untouched — they sit below the new triggers and resolve after them. No triggers are lost because the scan only *adds* to the stack; it never removes or reorders existing entries.

### Key Design Points

1. **Classification is static.** A triggered ability is always event-driven or always state-based. Determined by card text pattern:
   - "Whenever [noun] [verbs]" → event-driven (creature enters, player casts, damage is dealt)
   - "Whenever [condition]" / "At [time], if [condition]" → state-based (hand empty, life below threshold)

2. **603.8 mid-resolution firing and Approach B.** Per `state-tracking-architecture.md`, deltas are emitted per sub-action during resolution. The 603.8 "momentary hand-empty" case:

   If we evaluated the condition against *post-resolution* state (hand refilled), the state trigger would NOT fire. But 603.8 says it DOES fire. Two approaches:
   - **Approach A:** Evaluate state triggers after *each sub-action delta*. Correct per 603.8 but expensive (trigger scan after every sub-action).
   - **Approach B:** State triggers scan deltas for "did the condition become true at any point." If `HandSizeChanged { old: 3, new: 0 }` appears anywhere in the delta batch, fire the trigger even if a later delta reversed it.

   **Approach B is recommended** — O(deltas) cost, aligns with the delta log design. State triggers effectively become "event-driven triggers that watch for state-transition deltas," which means the scanner uses a single code path for both types.

3. **Delta requirements for state triggers (no saved GameState needed).** Deltas must carry enough information to detect transitions *without* needing a previous `GameState`. This means deltas record **(old, new)** pairs for state values:

   ```rust
   enum GameDelta {
       HandSizeChanged { player: PlayerId, old: usize, new: usize },
       LifeTotalChanged { player: PlayerId, old: i32, new: i32 },
       CharacteristicChanged {
           object: ObjectId,
           field: CharacteristicField,
           // old/new are opaque but comparable
           old_hash: u64,
           new_hash: u64,
       },
       ZoneTransfer { object: ObjectId, from: Zone, to: Zone },
       // ...
   }
   ```

   The `(old, new)` pattern means:
   - **No previous GameState saved.** The delta itself records the transition.
   - **No delta reversal needed.** We never need to "undo" a delta to reconstruct past state. (This is good because not all actions are reversible — library shuffles destroy ordering information, random choices are non-deterministic, etc.)
   - **Transition detection is local:** `old != new` tells us something changed, and the specific values tell us *what* the transition was (e.g., hand went to 0).

   The engine records `old` values at the point of mutation (cheap — just read the current value before writing the new one). This is O(1) per mutation.

4. **Integration with delta log.** Both trigger kinds consume the same `GameDelta` stream. The delta log is the single source of truth for "what happened." This aligns with `state-tracking-architecture.md` §603.8.

---

## 14. Applicability System: What Does an Ability Affect?

**Question:** Multiple design decisions reference an `Applicability` type that doesn't exist in the plan yet. What is it, and where does it fit?

### The Problem

Every ability in the engine needs to know *what it applies to*. Currently, card definitions encode this implicitly. But multiple systems now depend on being able to ask "does this ability target/affect itself, or a category?":

| System | Question Asked | Section |
|--------|---------------|---------|
| **612.3** text-changing | "Is this ability intrinsic (apply text-change) or granted (skip)?" | §4 |
| **614.12** ETB look-ahead | "Does this ETB replacement affect only itself, or a general subset?" | §8 |
| **613 layers** | "Which objects does this continuous effect apply to?" | §1, §3 |
| **Dependency system** | "Would applying effect B change which objects effect A applies to?" | §7 |
| **Trigger scanning** | "Which events/objects does this trigger watch for?" | §13 |
| **Replacement pipeline** | "Does this replacement apply to this event?" | §9 |

### Proposal: `Applicability` as a First-Class Type

```rust
/// Describes what set of objects/events an ability applies to.
/// Every ability carries one of these.
enum Applicability {
    /// Affects only the object that has this ability.
    /// "This creature enters tapped", "~ gets +1/+1", "As ~ enters, choose..."
    SelfRef,
    
    /// Affects a filtered subset of objects.
    /// "Creatures you control get +1/+1", "Permanents enter tapped"
    ObjectFilter(ObjectFilter),
    
    /// Affects a player or set of players.
    /// "Each opponent loses 1 life", "You gain 2 life"
    PlayerFilter(PlayerFilter),
    
    /// Affects a specific event type (for replacement/trigger matching).
    /// "If a creature would die", "Whenever a creature enters"
    EventFilter(GameActionPattern),
}

/// Predicate for filtering game objects.
struct ObjectFilter {
    controller: Option<PlayerRef>,  // "you control", "an opponent controls"
    zone: Option<Zone>,             // "on the battlefield", "in a graveyard"
    type_filter: Option<TypeFilter>, // "creature", "artifact", "permanent"
    characteristic_pred: Option<CharacteristicPredicate>, // "with power 4+", "that's red"
}

/// Relative player references (resolved at evaluation time).
enum PlayerRef {
    You,          // the controller of this ability
    Opponent,     // any single opponent
    AllOpponents, // each opponent
    AnyPlayer,    // any player
    TargetPlayer, // chosen target (for targeted abilities)
}
```

### How Systems Use It

**614.12 self-only check:**
```rust
fn is_self_only_etb(ability: &Ability) -> bool {
    ability.is_etb_replacement()
        && matches!(ability.applicability(), Applicability::SelfRef)
}
```

**612.3 text-changing scope:**
```rust
fn should_apply_text_change(ability: &Ability, target_obj: ObjectId, source_obj: ObjectId) -> bool {
    // Text-changing only affects intrinsic text, not granted abilities.
    // Intrinsic abilities have origin == Intrinsic (from §4).
    // If the ability is on the same object and is SelfRef, it's intrinsic text.
    ability.origin == AbilityOrigin::Intrinsic
}
```

**Layer system — collecting applicable effects:**
```rust
fn applies_to_object(effect: &ContinuousEffect, obj: &GameObject, state: &GameState) -> bool {
    match &effect.applicability {
        Applicability::SelfRef => effect.source_id == obj.id,
        Applicability::ObjectFilter(filter) => filter.matches(obj, state),
        _ => false, // player/event applicability not relevant for object effects
    }
}
```

**Dependency system (613.8a condition b):**
```rust
/// Would applying effect B change which objects effect A applies to?
fn changes_applicability(a: &ContinuousEffect, b: &ContinuousEffect, state: &GameState) -> bool {
    match &a.applicability {
        Applicability::SelfRef => {
            // SelfRef never changes applicability — it always applies to its source
            false
        }
        Applicability::ObjectFilter(filter) => {
            // Does B change characteristics that filter checks?
            // e.g., B changes types → A filters by type → dependency
            filter.depends_on_characteristics_changed_by(b)
        }
        _ => false,
    }
}
```

### Where It Lives

`Applicability` is a **shared type** defined in the card/ability definition layer (not in the engine). It's populated at card registration time and consumed read-only by all the systems above.

Proposed location: `src/cards/types.rs` or a new `src/cards/applicability.rs` alongside the existing card definition infrastructure. The `ObjectFilter` and `PlayerRef` types are also used by the trigger system (§13) and replacement pipeline (§9), so they should be in a shared location.

### Relationship to Existing Targeting

The current codebase has some targeting logic for spells (choose targets during casting). `Applicability` is **not the same as targeting** — it's the static description of what an ability *affects*, not the runtime choice of what it's *aimed at*:

- **Targeting** (rule 115): Runtime choice, can be illegal, checked on resolution. "Target creature gets +1/+1."
- **Applicability**: Static description, always valid, checked by layer engine / replacement pipeline. "Creatures you control get +1/+1" — no target, just a filter.

Some abilities have both: "Target creature you control gets +1/+1 and other creatures you control get +0/+1." The targeting system handles the first part, `Applicability::ObjectFilter` handles the second.

### Assessment

| Pillar | Assessment |
|--------|------------|
| **Correctness** | Centralizes "what does this affect" in one place, reducing inconsistency across systems |
| **Extensibility** | New filter types (e.g., "cards in exile with mana value 3+") just add fields to `ObjectFilter` |
| **Maintainability** | One type to audit per card, used by 6+ systems. Changes to filtering logic are localized |
| **Speed** | `SelfRef` is O(1). `ObjectFilter` is O(objects) but already required by every system that uses it |

---

## Summary of Design Decisions Needed Before Implementation

| Priority | Topic | Decision Needed |
|----------|-------|-----------------|
| **P0** | Dependency system architecture | Module structure, API design, Kahn's algorithm implementation |
| **P0** | Effect discrimination (611.2c/613.11) | `ContinuousEffectKind` enum, card registration tagging |
| **P0** | Layer system recalculation model | Confirm "recalculate from scratch" vs incremental |
| **P0** | Timestamp composite type | `(global_seq, sub_index)` structure; interaction with APNAP |
| **P0** | Applicability system | `Applicability` enum + `ObjectFilter` — shared type for all ability-scope queries |
| **P1** | Granted ability origin tracking (612.3) | `AbilityOrigin` enum on ability instances |
| **P1** | ETB self-only qualifier (614.12) | Derive from `Applicability::SelfRef` (no new enum) |
| **P1** | Replacement pipeline architecture (616) | Unified pipeline with priority ordering |
| **P1** | Event decomposition (616.1g) | `execute_and_decompose()` — creates inner events, no nesting |
| **P1** | Trigger lifecycle integration | `scan_triggers` called after every resolution; APNAP stack ordering |
| **P2** | ETB look-ahead (614.12) | Phantom entry approach, SelfRef filtering, perf |
| **P2** | ETB auxiliary dispatch (614.13) | Match-arm dispatch per replacement type |
| **P2** | Delta log format | `(old, new)` pairs on state-change deltas; no GameState saving or reversal |
| **P2** | Trigger classification (603.1/603.8) | `TriggerKind` enum, delta-based scanning for both types |
