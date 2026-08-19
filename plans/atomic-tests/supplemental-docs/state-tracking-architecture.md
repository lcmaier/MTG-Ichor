# State Tracking & Loop Detection Architecture

> **Origin:** Extracted from `session-5-audit-response.md` Q3+Q4 discussion (2026-04-05).
> **Scope:** Delta log design, trigger tracking (603.8, 603.1b, cross-turn, resolution counting), loop detection (731), and voluntary shortcut system (729/D26).
> **Status:** Architectural proposal — deferred to Phase 7 implementation. No current code depends on these decisions.

---

## Problems to solve

1. **603.8 — State triggers fire mid-resolution.** The CR Example is explicit: "Discard your hand, then draw that many cards" momentarily empties the hand, and a "whenever you have no cards in hand" trigger DOES fire. The engine must detect momentary state conditions during resolution.
2. **603.1b — Multi-condition triggers.** "Whenever you cast a creature spell AND an artifact spell in the same turn" requires tracking multiple events across a turn.
3. **Cross-turn lookback.** Paladin of Atonement: "At the beginning of each upkeep, if you lost life last turn." Alchemy cards could look back arbitrarily far.
4. **Resolution counting.** Ashling, Flame Dancer: "If this is the second time this ability has resolved this turn..."
5. **Loop detection (rule 731).** Detecting repeating game states for mandatory-loop draw declarations.

---

## Approaches considered

### Approach 1: `im` crate persistent data structures

Replace `std::HashMap`, `Vec`, etc. in GameState with `im` equivalents. Snapshots become O(1) pointer copies.

- **Pros:** Full historical GameState at every point. Condition authoring as `fn(&GameState) -> bool`. Loop detection via structural equality with shared-node short-circuiting.
- **Cons:** Pervasive refactor (all collections + call sites). 2-5x constant performance tax on ALL mutations. Overkill for trigger checking.

### Approach 2: Delta log

State-mutating methods emit structured delta entries to a game-lifetime log. Trigger matchers scan deltas.

```rust
struct GameDelta {
    turn: TurnNumber,
    phase: Phase,
    resolution_id: Option<StackObjectId>,
    action: GameAction,
}

enum GameAction {
    HandSizeChanged { player: PlayerIndex, old: usize, new: usize },
    LifeChanged { player: PlayerIndex, old: i32, new: i32 },
    ZoneTransfer { object: ObjectId, from: Zone, to: Zone },
    CounterChanged { object: ObjectId, counter_type: CounterType, old: u32, new: u32 },
    // ... other mechanically-relevant deltas
}
```

- **Pros:** Zero steady-state overhead. Small deltas. Cross-turn lookback via turn-number filtering.
- **Cons:** Loop detection expensive (must reconstruct states or pattern-match). Condition authoring in "what changed" terms.

### Approach 3: Zobrist hashing (rejected)

Classical Zobrist hashing is **not feasible for MTG.** Zobrist requires precomputing a random bitstring for every (component, value) pair. In chess: ~781 bitstrings, finite and compile-time enumerable. In MTG:
- ObjectIds are runtime-generated (tokens, copies).
- Counter counts and life totals are unbounded.
- Library ordering is a permutation of N cards.
- Continuous effects create arbitrary characteristic modifications.

Lazy Zobrist (on-demand bitstrings) has unpredictable cache growth and fragile XOR semantics.

### Approach 3 (revised): Delta log + deterministic full-state hash

**Delta log** for trigger tracking (common path, cheap). **Full-state hash** (`#[derive(Hash)]` on GameState) at quiescent points for loop detection. O(state_size) per check but infrequent (once per priority pass). Acceptable because loop detection doesn't need O(1)-per-mutation — just cheap-enough at priority-check frequency.

### Comparison table

| Concern | `im` persistent | Delta log only | Delta log + full-state hash |
|---|---|---|---|
| 603.8 state triggers | Eval on snapshots | Scan deltas | Scan deltas |
| Multi-condition triggers | Eval on snapshots | Filter deltas by turn | Filter deltas by turn |
| Cross-turn lookback | Query snapshot | Filter by turn number | Filter by turn number |
| Resolution counting | Count snapshots | Count deltas | Count deltas |
| **Loop detection (731)** | Structural equality. Good. | Reconstruct states. **Expensive.** | **Full-state hash. Good.** |
| Mutation cost | **2-5x slower** | **Zero** | **~Zero** |
| Refactor invasiveness | **High** | Moderate | Moderate |

---

## What existing simulators do

No major MTG simulator solves general loop detection algorithmically:

- **XMage:** `Watchers` pattern-observer + `GameState.copy()` deep clones. Loop detection: heuristic counter → prompt players to vote draw. Buggy/exploitable (GitHub #6212).
- **MTGO:** Heuristic counter → offer draw after threshold. Fails on unusual loops ("LSV breaks MTGO").
- **MTG Arena / GRE:** Heuristic action counting + pattern detection. Declares draws after repeated mandatory patterns. Still has edge cases.
- **Forge:** No loop detection. Hard iteration cap.

**Takeaway:** Industry standard is "count repeated actions, declare draw after threshold." MTG is Turing-complete (Churchill et al., 2019, arXiv:1904.09828), making general halting detection formally unsolvable.

---

## Pregame sweep optimization

A pregame `RelevantEffects` analysis could prune which `GameAction` variants are emitted. Good for delta log efficiency. Less useful for loop detection (needs full state regardless). Breaks for conjure/wish cards — fall back to "track everything." Deferrable, no architectural impact.

---

## Recommendation: Delta log + tiered loop detection

**Trigger tracking:** Delta log. Settled.

**Loop detection (731):** Tiered:

### Tier 1 — Forced-action counter

Count consecutive forced actions without meaningful player decisions.

**Forced action** = engine action with no DP call, or DP called with only 1 legal option:
- SBAs (no DP call)
- Single pending trigger placement (no ordering choice)
- Trigger resolution with no choices (no "may," ≤1 legal target, no modes)
- Priority pass where only legal action is "pass"

**Meaningful player decision** = DP call with 2+ legal options. Resets counter.

**Example — 3× Oblivion Ring:**
1. O-Ring C ETB: 2 legal targets → meaningful decision → counter resets. Player picks A.
2. O-Ring A LTB: no choices → counter: 1.
3. O-Ring B ETB: 1 target (forced) → counter: 2.
4. Continue. Counter hits threshold → `PotentialMandatoryLoop`.

**Threshold:** Configurable, default ~15. Low enough for good UX, high enough to avoid false positives on legitimate complex board states.

```rust
struct LoopDetector {
    consecutive_forced_actions: u32,
    threshold: u32, // default 15
}

impl LoopDetector {
    fn on_action(&mut self, had_meaningful_choice: bool) -> LoopStatus {
        if had_meaningful_choice {
            self.consecutive_forced_actions = 0;
            LoopStatus::Normal
        } else {
            self.consecutive_forced_actions += 1;
            if self.consecutive_forced_actions >= self.threshold {
                LoopStatus::PotentialMandatoryLoop
            } else {
                LoopStatus::Normal
            }
        }
    }
}
```

### Tier 2 — Full-state hash at quiescent points

`#[derive(Hash)]` on GameState. At quiescent points (empty stack + priority), hash and check `HashSet<u64>`. Catches long-period or subtle loops missed by Tier 1.

### Tier 3 — Full deep comparison on hash collision

`#[derive(PartialEq)]` confirmation. Store last N full states. Eliminates false positives. Almost never runs.

---

## Three categories of loops

| Category | Example | Detection | Resolution |
|---|---|---|---|
| **A — Mandatory, no net change** | 3× Oblivion Ring | Tier 1 counter / Tier 2 hash | Forced draw (731.4) |
| **B — Mandatory, accumulating** | Forced infinite draw triggers | Tier 1 counter | Fast-forward to termination or draw |
| **C — Voluntary, accumulating** | Splinter Twin combo | Player declares shortcut | D26: validate + bulk-apply |
| **D — Fragmented, voluntary** | "{0}: gains flying" vs "{0}: loses flying" | Tier 2 hash match | Active player must choose differently (731.3) |

Categories A and B are handled by the tiered detection system. Category C is handled by D26. Category D is detected by Tier 2 (full-state hash match at quiescent points) but resolved differently from A—see loop resolution logic below.

---

## Loop resolution logic (731.3, 731.5, 731.6)

> Added from Session 9B audit (2026-04-09). These rules constrain *how* loops are resolved once detected.

Tier 1/2 detect loops. The resolution path depends on which category:

### 731.3 — Fragmented loops (Category D)

When Tier 2 detects a hash match involving voluntary actions from multiple players (each player independently took an optional action), this is a **fragmented loop**, not a mandatory loop. Per 731.3:

- The active player (or first involved player in turn order) must make a **different choice** to break the loop.
- This is NOT a draw — it’s a forced choice change.
- The nonactive player gets the "final say" — the game state settles in their favor.

Tier 1 does NOT catch these because both actions involve meaningful player decisions (DP called with 2+ options), so the forced-action counter keeps resetting.

**Implementation:** When Tier 2 fires and the delta log transcript shows the repeating segment contains DP calls with 2+ options from multiple players, classify as Category D. Prompt the active player with their normal options but **exclude the action they took in the previous iteration** (or mark it as loop-perpetuating).

### 731.5 — Involved-object constraint

When resolving any loop (Category A or D), the engine **cannot force a player to use objects not involved in the loop** to break it. For example, a player controlling Seal of Cleansing cannot be forced to sacrifice it to end an Oblivion Ring loop.

**Implementation:** The loop detector must track which `ObjectId`s appear in the delta log transcripts for the repeating segment. When prompting for a loop-breaking action (Category D) or evaluating whether a mandatory loop is truly inescapable (Category A), only actions involving those objects count.

```rust
struct DetectedLoop {
    category: LoopCategory,
    involved_objects: HashSet<ObjectId>,
    involved_players: Vec<PlayerIndex>,
    repeating_segment: Range<usize>,  // indices into delta log
}

enum LoopCategory { Mandatory, Fragmented }
```

### 731.6 — Unless-clause constraint

In a loop containing "[A] unless [B]," no player can be forced to perform [B] to break the loop. If no player voluntarily chooses [B], [A] becomes mandatory.

**Consequence:** If the loop detector identifies an unless-clause in the repeating segment, it must NOT auto-select [B] as the loop-breaking action. If [A] is mandatory and creates an inescapable loop, the game is a draw (Category A). The DP may *offer* [B] as an option, but cannot *force* it.

**Implementation:** The `Effect::Conditional` / unless-clause representation must be inspectable so the loop resolver can identify which branch is the unless-branch and avoid forcing it.

---

## D26 — Voluntary shortcut system (rule 729)

### "Execute first, then declare" pattern

The player executes the loop through normal gameplay (all decisions via existing DP interface). After two iterations, declares "repeat N more times." Engine validates using delta log transcripts.

**Why not "declare first":** DP would need to reference objects that don't yet exist, breaking the core invariant that every DP method receives `&GameState` and returns choices about currently-existing objects. Speculative APIs or template languages add complexity without benefit — the engine would execute internally to validate anyway.

### Concrete flow

1. **Iteration 1:** Player executes loop normally. Delta log captures action transcript.
2. **Iteration 2:** Player executes again. Delta log captures second transcript.
3. **Declaration:** DP signals shortcut:
```rust
fn declare_shortcut(
    &self,
    state: &GameState,
    recent_actions: &[GameAction],
) -> Option<ShortcutDeclaration>;

struct ShortcutDeclaration {
    loop_boundary_index: usize,  // boundary between iteration 1 and 2 in delta log
    iterations: GameNumber,       // how many more
}
```
4. **Engine validation:**
   - Extract two transcripts from delta log.
   - Compare structurally (same `GameAction` sequence, same relative object references).
   - Compute per-iteration net delta.
   - Check self-termination (e.g., Devoted Druid without Vizier → toughness decreasing → cap N).
   - If valid: bulk-apply `net_delta × N`, emit `ShortcutApplied` to delta log.
   - If invalid: reject, player continues manually.

### Why two iterations

One sample can't confirm stability. Two iterations prove the sequence is repeatable:
- **Devoted Druid + Vizier:** Both produce {G}, transcripts match → stable.
- **Devoted Druid without Vizier:** Transcripts match structurally but toughness decreasing → engine caps N.
- **Token that alters next iteration:** Transcripts won't match → rejected.
- **Splinter Twin:** Both create hasty token + untap Twin → stable. Bulk-create N tokens.

### Shared infrastructure

Both Tier 2/3 and D26 need `#[derive(Hash)]` and `#[derive(PartialEq)]` on `GameState` and `GameAction`. D26 additionally uses delta log transcripts. No extra infrastructure beyond what's already planned.

### DP intelligence is separate from engine correctness

Simple DP: never shortcuts, plays until `MAX_TRIGGER_ITERATIONS`. Smart DP: recognizes patterns after 2 iterations. Smarter DP: pre-evaluates combo availability. Engine role is fixed: validate + bulk-apply.

---

## The formal boundary

| Problem | Solvable? | How? |
|---|---|---|
| Mandatory, no net change (731.4) | **Yes, automatically.** | Tiers 1-3 → forced draw |
| Mandatory, accumulating | **Yes, automatically.** | Tier 1 + natural termination |
| Fragmented, voluntary (731.3) | **Yes, automatically.** | Tier 2 hash → active player must choose differently |
| Voluntary, accumulating (729) | **Yes, with player input.** | D26 shortcut validation |
| General automatic detection without player input | **No — halting problem.** | Formally unsolvable (MTG is Turing-complete) |

No simulator solves the last row. Pragmatic fallbacks: `MAX_TRIGGER_ITERATIONS` (Phase 7), full `GameNumber` + `LoopDeclaration` (Phase 9).

---

## 603.8: No interleaving needed

With the delta log, resolution doesn't interleave state-trigger checks per sub-action:
1. Each sub-action emits a delta.
2. After full resolution, state triggers scan deltas (filtered by `resolution_id`).
3. Matching triggers → `pending_triggers` queue.

Single post-resolution pass, not per-sub-action.

---

## Casting rollback (601.2e)

Separate concern. Clone GameState before 601.2a, restore on failure. Delta log doesn't change this. Deferred to T18.

---

## Motivating card examples

- **603.8:** "Whenever you have no cards in hand, draw a card" + "Discard your hand, draw that many" → delta captures `HandSizeChanged { new: 0 }`.
- **603.1b:** Paladin of Atonement → query previous turn's `LifeChanged` deltas.
- **603.7h:** Ashling, Flame Dancer → count `AbilityResolved` deltas this turn.
- **731:** 3× Oblivion Ring → Tier 1 counter catches immediately.
