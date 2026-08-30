# MTG-Ichor — Magic: The Gathering Rules Engine

A from-scratch Magic: The Gathering rules engine in Rust. Correctness-first: the
Comprehensive Rules are the source of truth, and behavior is *implemented*, not
approximated. The engine is UI-agnostic and does no I/O — it exists to power a GUI for
humans playing over a network, and to run headless for AI self-play across many parallel
games.

**v1 targets two use cases:** peer-to-peer human games through a GUI — specifically
**4-player Commander** — and **highly parallel AI games** over the CLI. A correct
two-player game is a checkpoint on the way, not the destination.

> **Status:** The layer system (CR 613) core is live; replacement effects (CR 614–616) are
> the phase starting now. Build is green with zero warnings.
>
> For anything more precise than that — per-rule coverage, what's stubbed, what's next —
> read [`plans/codebase-state.md`](plans/codebase-state.md). It is the single source of
> truth and it is maintained as part of the work that changes it. This README deliberately
> does not duplicate its numbers.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Game — lifecycle, setup, config, DecisionProvider dispatch │
├─────────────────────────────────────────────────────────────┤
│  ui/ — DecisionProvider trait (4 primitives) + ask_* bridge │
│  (CLI, Random, Scripted, Dispatch implementations)          │
├─────────────────────────────────────────────────────────────┤
│  engine/ — Rules engine (reads + mutates GameState)         │
│  ┌──────────┬──────────┬────────────┬─────────────────────┐ │
│  │ cast.rs  │ stack.rs │priority.rs │ targeting.rs        │ │
│  │ turns.rs │ zones.rs │ resolve.rs │ costs.rs            │ │
│  │ sba.rs   │ mana.rs  │ combat/    │ actions.rs          │ │
│  │ layers/  │keywords.rs│           │                     │ │
│  └──────────┴──────────┴────────────┴─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  oracle/ — Read-only queries over EFFECTIVE characteristics │
│  (characteristics, legality, board, mana_helpers)           │
├─────────────────────────────────────────────────────────────┤
│  state/ — GameState, GameConfig, PlayerState,               │
│           ContinuousEffectRegistry, BattlefieldEntity       │
│  objects/ — GameObject, CardData                            │
│  types/ — Enums and value types (no logic)                  │
│  events/ — EventLog for game history                        │
│  cards/ — Card definitions (data only, via CardRegistry)    │
└─────────────────────────────────────────────────────────────┘
```

### Key design principles

- **Central object store.** All game objects live in one `HashMap<ObjectId, GameObject>`;
  zones reference objects by id.
- **Single zone-transition chokepoint.** Every zone move goes through `move_object()`.
- **The engine does no I/O.** Every player decision routes through the `DecisionProvider`
  trait. The engine is pure state transforms.
- **Composable effects.** Card effects are trees built from `Primitive` variants and
  `Effect` combinators (`Atom`, `Sequence`, `Conditional`, `Modal`, …).
- **Immutable card data.** `CardData` is `Arc`-shared across instances; the layer system
  computes effective characteristics on top of printed values.
- **Action pipeline.** Observable mutations route through `execute_action(GameAction)` —
  the seam the replacement-effect pipeline hooks into.

### Three invariants worth knowing before touching the code

These are load-bearing. Each has already cost a redesign or produced silent wrong behavior,
and each is stated in full in [`CLAUDE.md`](CLAUDE.md).

1. **Never read printed characteristics for anything on the battlefield or the stack.**
   `card_data.{types,subtypes,supertypes,colors,keywords,abilities}` stopped equalling
   effective characteristics when Layer 4 landed. Route through `oracle/characteristics.rs`.
   Ability *indices* count too — a Blood-Mooned land has no printed abilities left and gains
   an intrinsic `{T}: Add {R}` that exists nowhere in its `CardData`.
2. **Registry membership is not effect existence.** A static ability's effect applies only
   while its source still has that ability, and CR 305.7 or Layer 6 can remove it without
   touching the registry — so existence is re-checked at every layer, against the previous
   layer's frame. Deciding existence outside the layer walk turns a terminating computation
   into a fixpoint that invents oscillation the CR does not have.
3. **Determinism at the decision boundary.** A `DecisionProvider` picks by *index*, so the
   order a sweep returns in is part of the decision. Sweeps that reach a choice go through
   `battlefield_ordered` / `battlefield_ids_ordered`, ordered by `BattlefieldEntity::timestamp`
   — which is CR 613.7's order anyway. Never raw `HashMap` order, and never `ObjectId`, which
   is a v4 UUID. Randomness is owned, never ambient: draw from `GameState.rng`, not
   `rand::rng()`.

---

## What's implemented

A coarse map. The per-CR-rule breakdown lives in
[`plans/codebase-state.md`](plans/codebase-state.md).

| Area | Status |
| --- | --- |
| Turn structure, all phases and steps (CR 5) | ✅ |
| Mana: pool, restrictions, persistence, context-aware spending (CR 106, 123) | ✅ |
| Casting pipeline (CR 601.2), stack and resolution (CR 608) | ✅ core; modes and activation restrictions pending |
| Priority, mana-ability windows (CR 117, 601.2g) | ✅ |
| Targeting (CR 115) | ✅ core; changing targets pending |
| Combat, including 2025 damage-assignment rules (CR 506–511) | ✅ |
| State-based actions (CR 704) | ✅ |
| Keyword abilities (CR 702) | ✅ evergreen set; infect/wither, equip, bestow pending |
| **Layer system (CR 613)** | 🟡 Layers 2, 4, 5, 6, 7a–7d live; 1 and 3 stubbed; **613.8 dependency algorithm not started** |
| Characteristic-defining abilities (CR 604.3) | ✅ |
| CR 305.7 land-type replacement (Blood Moon, Urborg) | ✅ |
| **Replacement and prevention (CR 614–616)** | ❌ stub hook only — the phase starting now |
| **Triggered abilities (CR 603)** | ❌ enum variant only |
| Commander (CR 903) | 🟡 skeleton: command zone, commander-damage SBA and accumulation |
| Multiplayer (CR 800/802) | ❌ |
| CLI play, seeded and threaded fuzz harness | ✅ |

**Cards:** 54 registered — basic and dual lands, Alpha staples, vanilla and keyword
creatures, and the layer-exercising set that arrived with Phases LB–LG (Blood Moon,
Humility, Glorious Anthem, March of the Machines, Tarmogoyf, Merfolk Thaumaturgist,
Moonlace, …). Cards are data, not engine code — see `mtgsim/src/cards/`, and
[`plans/cards-unlocked-ledger.md`](plans/cards-unlocked-ledger.md) for which ticket unlocks
what.

---

## Getting started

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)

### Build and test

```bash
cd mtgsim && cargo test
```

`cargo build --all-targets` must print **zero warnings** — a hard bar, not a preference.

### Play at the terminal

```bash
cd mtgsim && cargo run --bin cli_play
```

You play Player 0 against a random-decision bot.

### Fuzz harness

```bash
cd mtgsim && cargo run --bin fuzz_games -- --games 500 --seed 42 --threads 8
```

Flags: `--games N`, `--max-turns N`, `--seed N`, `--threads N`, `--pool
performance|stress`, `--verbose`, `--dump-events <path>`.

Two card pools. `performance` is the frozen 55 every recorded baseline was measured on
and is the default, because an A/B against a pool that moved is not an A/B; `stress` is
every registered card, which is what hunts panics and exercises effect interactions.
The harness prints which one it played.

A given `--seed` reproduces a run exactly. Three runs at one seed must agree on every line
except the two wall-clock lines; a differing turn count or outcome means process state
reached a decision, which is a bug.

### Rules-coverage queries

```bash
python plans/specdb.py stats
```

`specdb` joins the atomic-test corpus to the test suite and to the CR, so "what is covered"
is a query rather than hand-maintained prose. Also available: `next --phase`,
`show <ATOM-ID>`, `gaps --chapter N`, `orphans`, `suspicious`.

---

## DecisionProvider

The engine is completely UI-agnostic, and the trait is deliberately **narrow**: four
primitives, rather than one method per kind of decision.

| Primitive | Purpose |
| --- | --- |
| `pick_n` | Choose between *min* and *max* options from a list |
| `pick_number` | Choose a number in a range (X values, counter counts) |
| `allocate` | Distribute a quantity across recipients (damage assignment, generic mana) |
| `choose_ordering` | Order a set (trigger stacking, library arrangement) |

What a given choice *means* is carried alongside it by a `ChoiceContext` with a
`ChoiceKind` enum — `DeclareAttackers`, `AssignTrampleDamage`, `ChooseXValue`, and so on.
Adding a new decision to the engine means adding a `ChoiceKind` variant: the trait and every
implementation of it stay unchanged, and exhaustive matching makes the compiler point at
every UI site that needs a new screen.

Engine code never calls the trait directly. It goes through the typed `ask_*` free functions
in `ui/ask.rs`, which build the context, pack the options, call the right primitive, and
validate the response (bounds, counts, sums, permutations) before unpacking it into typed
results.

**Built-in implementations:** `CliDecisionProvider` (interactive stdin/stdout),
`RandomDecisionProvider` (fuzzing and bot opponents; `::seeded` for replayable runs),
`ScriptedDecisionProvider` (deterministic integration tests), and
`DispatchDecisionProvider` (routes decisions per player id).

---

## Roadmap

Dependency order — each item needs the ones above it. [`CLAUDE.md`](CLAUDE.md) →
"Critical path to v1" owns this ordering; when another doc disagrees, it wins.

| # | Scope | Status |
| --- | --- | --- |
| 1 | Layer system core → Layer 4 → static-ability effect existence | ✅ |
| 2 | Characteristic-defining abilities (CR 604.3 / 613.4a) | ✅ |
| 3 | Layer 6 — ability adding and removing (Humility) | ✅ |
| 4 | Layer 2 — control changing | ✅ |
| 5 | **Replacement and prevention effects (CR 614–616)** | 🔜 **starting now** |
| 6 | Triggered abilities (CR 603) — takes LKI and conditional statics with it | Planned |
| 7 | The CR 613.8 cluster — dependency algorithm, board-wide sequential pass, memoization | Planned |

Interleaved after 5 rather than sequenced against it: the **Commander and multiplayer
track** — cost modification (commander tax), CR 903.9a/b, `GameConfig::commander()`, CR 800
priority, turn rotation and elimination, and CR 802.

Item 7 is a hard back-stop before broad card work: a Commander-viable pool is dense in
exactly the static abilities that interact, so dependency-ordering-sensitive cards cannot be
authored until it lands.

**On the ordering:** replacement effects come *before* triggered abilities. Replacement
gates most real cards and Commander's 903.9b command-zone redirection, and triggers need to
fire on events observed after replacement has applied.

The execution plan for the current phase is
[`plans/replacement-architecture.md`](plans/replacement-architecture.md): phases RA (event
spine) → RB (pipeline) → RC (ETB replacements) → RD (damage) → RE (remaining event kinds).

---

## Project layout

```
mtgsim/src/
├── bin/            cli_play.rs, fuzz_games.rs
├── cards/          Card definitions (data only) + registry.rs
├── engine/         actions, cast, costs, mana, priority, resolve, sba,
│                   stack, targeting, turns, zones, keywords
│   ├── combat/     validation, resolution, steps, keywords
│   └── layers/     compute, types, cda, land_types  ← CR 613
├── events/         GameEvent + EventLog
├── objects/        CardData, AbilityDef, GameObject
├── oracle/         characteristics, legality, board, mana_helpers
├── state/          game, game_state, game_config, player, battlefield,
│                   continuous_effects  ← the ContinuousEffect registry
├── types/          ids, mana, effects, costs, card_types, colors,
│                   keywords, keyword_actions, zones
└── ui/             decision (trait), ask (typed bridge), choice_types,
                    cli, random, display
```

Integration tests live in `mtgsim/tests/`, one file per phase.

---

## Documentation map

Authority order — when two docs disagree, the higher one wins.

| Doc | Authoritative for |
| --- | --- |
| [`plans/codebase-state.md`](plans/codebase-state.md) | **Current state.** Beats every other doc, this README included |
| [`CLAUDE.md`](CLAUDE.md) | Invariants, conventions, commands, critical-path ordering |
| [`plans/layers-architecture.md`](plans/layers-architecture.md) | The layer system: type shapes, module layout, dependency algorithm |
| [`plans/replacement-architecture.md`](plans/replacement-architecture.md) | Replacement and prevention (CR 614–616): event vocabulary, the CR 616.1 pipeline, phase sequencing |
| [`plans/atomic-tests/sessions/`](plans/atomic-tests/sessions/) | The spec corpus — atomic tests from a close read of the CR. Authored, never generated |
| [`MTG-Rules/versions/`](MTG-Rules/versions/) | The CR itself; `tmnt.txt` is the baseline the engine targets |
| [`plans/cards-unlocked-ledger.md`](plans/cards-unlocked-ledger.md) | Which cards each ticket unlocks |
| [`plans/engineering-practices.md`](plans/engineering-practices.md) | Process: `CLAUDE.md`'s line budget, the comment rule, the two card pools, phase sizing, the specdb gate |
| [`design_doc.md`](design_doc.md) | The original design. **Historical**, except its §636–664 algorithm, adopted verbatim by `layers-architecture.md` |
| `plans/archive/` | Superseded. Do not act on it |

`spec.sqlite` and the generated index files under `plans/atomic-tests/` come from
`specdb.py build` — never hand-edit them; fix the session file and rebuild.

---

## Contributing

- Small commits. Commit messages explain *why* — they are part of the project record.
- One branch per unit of work → PR → merge with a merge commit, never squash. This project
  leans on its written record, and squashing discards per-commit messages.
- A bugfix must be shown to fail against the pre-fix tree before it is committed.
- New cards go in `mtgsim/src/cards/`, integration tests in `mtgsim/tests/`.
- Annotate tests with `// COVERS:` / `// COVERS-PARTIAL:` atom ids at write time. Never
  claim an atom a test does not prove — a false link is worse than a blank.

---

## License

See [LICENSE](LICENSE).
