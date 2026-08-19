# DecisionProvider Middleware & Candidate Enumeration

> Date: 2026-04-14
> Status: Design sketch — not scheduled for implementation
> Context: Arose during SPECIAL-1c (engine call site migration) when discovering that `ask_*` functions require option enumeration that the engine doesn't currently perform.
> Related: `decision-provider-refactor.md` (parent design), SPECIAL-1c (current ticket)

---

## 1. The Problem

The 4-primitive `DecisionProvider` trait (`pick_n`, `pick_number`, `allocate`, `choose_ordering`) requires someone to enumerate the options before presenting them. The old typed-method trait pushed legality enumeration into each DP impl (Random, CLI, Scripted all independently queried the oracle). The new design centralizes that responsibility in the engine/oracle layer.

The hard case is **priority actions**: determining whether a player can legally cast a spell requires knowing whether they can pay its costs, which may involve arbitrarily complex mana ability activation chains. Proving castability ahead of time is equivalent to searching a branching execution tree and is intractable in the general case.

---

## 2. Candidate Enumeration Strategy

### Principle: Conservative Filtering Only

The oracle should only exclude options where it can **prove illegality from static game state reads**. Anything uncertain stays in the candidate list. The engine's execution + rollback (601.2e) handles false positives.

**Provably illegal (filter out):**

- Playing a land when `lands_played_this_turn >= max_lands_per_turn`
- Casting a sorcery-speed spell when it's not the player's main phase or the stack is non-empty
- Attacking with a creature that has defender (508.1a)
- Attacking with a creature that has summoning sickness and lacks haste
- Blocking with a tapped creature
- Targeting something that doesn't match the required type/characteristic

**Not provably determinable from reads (keep in candidate list):**

- Mana affordability (requires speculative mana ability activation)
- Complex additional cost payability (sacrifice a creature — do they have one they're willing to sacrifice?)
- Conditional legality that depends on choices not yet made

### The Contract

> The candidate list is an **overapproximation** of legal actions, never an underapproximation. False positives are harmless (engine rejects via rollback). False negatives are bugs (player can never take an action they're entitled to).

### Retry Behavior

When `RandomDecisionProvider` picks a candidate that turns out illegal, the engine rejects it and re-prompts. To avoid spinning on a candidate list full of false positives:

- Mark failed candidates as ineligible for the remainder of that priority window
- Bound retries (e.g., 3× the candidate list size) with fallback to `Pass`
- Log excessive retries as a diagnostic signal that the candidate filter needs tightening

---

## 3. Where Legality Lives

### Oracle (read-only, no mutation)

All candidate enumeration queries belong in `oracle/`. They are pure reads against `&GameState`:

- `playable_lands(game, player)` — exact (land drop count is static state)
- `legal_attackers(game, player)` — exact (tap state, summoning sickness, defender are static)
- `legal_blockers(game, player)` — exact (tap state, blocking restrictions are static)
- `candidate_spells(game, player)` — **overapproximation**: passes timing checks, may include unaffordable spells
- `candidate_abilities(game, player)` — same: timing-legal, affordability uncertain
- `legal_discard_targets(game, player)` — exact (hand contents are known)
- `legal_legend_choices(game, player, legend_name)` — exact

The oracle **never mutates game state**. This is a load-bearing invariant. If a query would require speculative execution to answer, it returns a conservative overapproximation instead.

### Engine (executes, validates, rolls back)

The engine attempts to execute the DP's choice. If execution fails (illegal target, can't pay costs, etc.), it rolls back via the 601.2e mechanism and re-prompts the DP. The engine is the correctness backstop — oracle filtering is a UX optimization, not a safety mechanism.

### Heuristic Affordability (`find_mana_sources`)

The existing `find_mana_sources` greedy algorithm in `oracle/mana_helpers.rs` is a best-effort filter. It catches the common case (enough untapped lands of the right colors) but misses complex mana ability chains. It is:

- **Useful** as a UX hint (CLI can show "affordable" spells, AI can prioritize likely-castable options)
- **Not authoritative** — the engine's execution + rollback is the real check
- **Clearly documented** as heuristic, not exact

---

## 4. Middleware DecisionProvider Architecture

### Motivation

The engine should never make choices on behalf of a player — it executes and validates. But some mechanical decisions (mana payment sequencing, trivial trigger ordering) are tedious for human players and irrelevant to AI training. These belong in a **wrapping DP** layer that sits between the engine and the "real" DP.

### Pattern

```
Engine ←→ AutoPayDP(inner: CliDP) ←→ Human
Engine ←→ AutoPayDP(inner: AiDP)  ←→ AI Policy
Engine ←→ RawDP                   ←→ AI (training on full action space)
```

A middleware DP implements `DecisionProvider`, holds an `inner: Box<dyn DecisionProvider>`, and intercepts specific `ChoiceKind` variants. Everything it doesn't handle passes through to `inner` unchanged.

```rust
struct AutoPayDP {
    inner: Box<dyn DecisionProvider>,
    // configuration, solver state, etc.
}

impl DecisionProvider for AutoPayDP {
    fn pick_n(&self, game, player, context, options, bounds) -> Vec<usize> {
        match &context.kind {
            ChoiceKind::GenericManaAllocation { .. } => {
                // Solve mana payment automatically
                self.auto_solve_mana(game, player, context, options, bounds)
            }
            _ => self.inner.pick_n(game, player, context, options, bounds),
        }
    }
    // pick_number, allocate, choose_ordering: delegate to inner
}
```

### Composability

Wrappers compose via nesting. Each handles one concern:

```
Engine → AutoPayDP → AutoYieldDP → AutoOrderTriggersDP → InnerDP
```

- **AutoPayDP**: Intercepts `GenericManaAllocation`, solves tap sequences. The hard one — requires a mana solver (Phase 5+).
- **AutoYieldDP**: Intercepts `PriorityAction` in known-pass situations (e.g., opponent's upkeep with no instants in hand). Arena's "pass until end" / "pass until response" behavior.
- **AutoOrderTriggersDP**: Intercepts `StackOrdering` for trivial cases (single trigger, all triggers have identical effect).

### Toggling

Each wrapper can be toggled off, causing it to pass all decisions through to `inner`. This is Arena's "full control" toggle generalized per-concern:

- Full control = all wrappers bypassed, inner DP sees every decision
- Default = wrappers handle mechanical tedium
- Per-wrapper toggle = "I want to auto-pay but manually order triggers"

### The Arena Problem and Why This Is Hard

Arena's auto-tapper is frustrating because it makes **locally correct but globally wrong** choices. It taps your only blue source for a 1G spell when you have a UU spell in hand. Solving this requires the auto-pay wrapper to consider:

- The player's hand (what else might they want to cast?)
- The player's plan (which the wrapper doesn't know)
- Opponent's board state (do they need to hold up countermagic mana?)

This is fundamentally a strategy question, not a mechanical one. A good auto-payer needs some degree of lookahead or player-expressed preferences ("prefer to keep blue open"). This is why it's Phase 5+ — the wrapper interface is simple, the solver behind it is not.

### Relationship to Existing Code

`DispatchDecisionProvider` is already this pattern applied to player routing (player 1 → CLI, player 2 → AI). Middleware wrappers are the same concept applied to decision-type routing. The engine doesn't care how many layers deep the stack is — it talks to one `dyn DecisionProvider`.

---

## 5. What This Means for SPECIAL-1c (Current Ticket)

None of this needs to be built now. The current ticket scope is:

1. Build candidate enumeration functions in `oracle/` using conservative filtering
2. Wire `ask_*` functions to use those candidates
3. Migrate engine call sites from `decisions.choose_*()` to `ask_*()`

The middleware architecture is a future concern. The important thing is that the current refactor **doesn't preclude it**: the 4-primitive trait, the `ChoiceContext` enum, and the `DispatchDecisionProvider` pattern all compose cleanly with wrapping DPs. No architectural changes needed later — just new wrapper types.

---

## 6. Open Questions (Deferred)

- **Mana solver design**: Greedy? Constraint-based? SAT solver? How deep does lookahead go? What's the performance budget?
- **Auto-payer preference system**: How does a player express "keep blue open"? Per-game setting? Per-spell override? Learned from play patterns?
- **Wrapper ordering**: Does the order of middleware composition matter? (Probably not for the initial set, but could matter if wrappers interact.)
- **Training mode**: AI training may want the raw action space (every tap decision) rather than the abstracted one. The `RawDP` bypass handles this, but training infrastructure needs to know which mode it's in.
- **Serialization**: If the game is serialized between DP calls (§8.2 atomicity invariant), do wrapper DPs need to serialize their internal state? AutoPayDP probably doesn't (stateless per-call). AutoYieldDP might (it tracks yield-until conditions).