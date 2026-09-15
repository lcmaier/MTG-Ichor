# MTG-Ichor — Magic: The Gathering Rules Engine

A from-scratch Magic: The Gathering rules engine in Rust. Correctness-first: the
Comprehensive Rules are the source of truth, and behavior is *implemented*, not
approximated. The engine is UI-agnostic and does no I/O — it exists to power a GUI for
humans playing over a network, and to run headless for AI self-play across many parallel
games.

**v1 targets two use cases:** peer-to-peer human games through a GUI — specifically
**4-player Commander** — and **highly parallel AI games** over the CLI. A correct
two-player game is a checkpoint on the way, not the destination.

> **Status (2026-09-15):** The layer system (CR 613) is complete but for Layer 3 and Layer
> 1b, including the CR 613.8 dependency algorithm. Replacement and prevention effects
> (CR 614–616) landed as Phases RA–RE, 2026-08-25 → 2026-09-15, and closed through an
> audit; every observable mutation is now a proposal the CR 616.1 pipeline sees before it
> happens. Next on the spine: triggered abilities (CR 603). Build is green with zero
> warnings.
>
> For anything more precise than that — per-rule coverage, what's stubbed, what's next —
> read [`plans/state-of-play.md`](plans/state-of-play.md), the generated board, and
> [`plans/codebase-state.md`](plans/codebase-state.md), the prose beside it. That file is
> the single source of truth and is maintained as part of the work that changes it. This
> README deliberately does not duplicate its numbers.

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
│           ContinuousEffectRegistry, PermanentState          │
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
- **One chokepoint.** Every observable mutation is a `GameAction` *proposal* through
  `execute_actions`; the CR 614–616 pipeline (`engine/replacement/`) sits between the
  proposal and the mutation, and the performed event is what the log — and, next,
  triggered abilities — read. A direct write is invisible to it by construction.

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
   `battlefield_ordered` / `battlefield_ids_ordered`, ordered by `PermanentState::timestamp`
   — which is CR 613.7's order anyway. Never raw `HashMap` order, and never `ObjectId`, which
   is a v4 UUID. Randomness is owned, never ambient: draw from `GameState.rng`, not
   `rand::rng()`.

---

## What's implemented

A coarse map. The per-CR-rule breakdown lives in
[`plans/codebase-state.md`](plans/codebase-state.md).

| Area | Status |
| --- | --- |
| Turn structure, all phases and steps (CR 5) | ✅ every turn, phase and step is a proposed event; extra turns and phases; skips |
| Mana: pool, restrictions, persistence, context-aware spending (CR 106, 123) | ✅ production is an event (CR 106.6a, 106.12) |
| Casting pipeline (CR 601.2), stack and resolution (CR 608) | ✅ core, with cost determination as its own pipeline (CR 601.2f); modes and most activation restrictions pending |
| Priority, mana-ability windows (CR 117, 601.2g) | ✅ |
| Targeting (CR 115) | ✅ core; changing targets pending |
| Combat, including 2025 damage-assignment rules (CR 506–511) | ✅ |
| State-based actions (CR 704) | ✅ one simultaneous batch per check |
| Keyword abilities (CR 702) | ✅ evergreen set, equip; infect/wither, bestow pending |
| **Layer system (CR 613)** | ✅ Layers 1a, 2, 4, 5, 6, 7a–7d live, the CR 613.8 dependency algorithm inside one board-wide pass; Layer 3 and Layer 1b stubbed |
| Characteristic-defining abilities (CR 604.3) | ✅ |
| CR 305.7 land-type replacement (Blood Moon, Urborg) | ✅ |
| CR 113.6 — which abilities function in which zone | 🟡 the registration leg; the replacement and restriction sweeps still visit the battlefield alone |
| **Replacement and prevention (CR 614–616)** | ✅ Phases RA–RE: the proposal chokepoint, the CR 616.1 loop, the CR 614.12 look-ahead frame, damage and prevention (CR 615), and every event kind the vocabulary derives |
| "Can't" effects (CR 101.2, 614.17) | 🟡 the spine and the event chokepoint; casting, combat and cost restrictions pending |
| Copy effects (CR 707) | 🟡 the copiable-values capture and "becomes a copy"; enters-as-a-copy, tokens, spell copies, faces, face-down pending |
| **Triggered abilities (CR 603)** | ❌ enum variant only — next on the spine |
| Commander (CR 903) | 🟡 command zone, CR 903.9a/b, commander damage; the tax, designation and `GameConfig::commander()` pending |
| Multiplayer (CR 800/802) | 🟡 any number of seats, N-player rotation, a lost player leaves the game (CR 800.4a–e); CR 802 pending |
| CLI play, seeded and threaded fuzz harness at any seat count | ✅ |

**Cards:** [`plans/state-of-play.md`](plans/state-of-play.md) carries the count, registered
and pooled. Basic and dual lands, Alpha staples, vanilla and keyword creatures, the
layer-exercising set from Phases LB–LK (Blood Moon, Humility, Glorious Anthem, Tarmogoyf,
Wonder, …) and the replacement track's consumers (Leyline of the Void, Kalitas, Furnace of
Rath, Doubling Season, Thought Reflection, Mana Reflection, …). Cards are data, not engine
code — see `mtgsim/src/cards/`, and
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

Flags: `--games N`, `--max-turns N`, `--seed N`, `--threads N`, `--players N`, `--pool
performance|stress`, `--require NAMES`, `--no-auto-pay`, `--verbose`,
`--dump-events <path>`.

Two card pools. `performance` is a frozen pool every recorded baseline was measured on
and is the default, because an A/B against a pool that moved is not an A/B; `stress` is
every registered card, which is what hunts panics and exercises effect interactions.
The harness prints which one it played and how many cards it holds. The record of every
measurement, newest first, is [`plans/fuzz-record.md`](plans/fuzz-record.md);
`plans/fuzz_ab.py` runs one sitting of the A/B.

A given `--seed` reproduces a run exactly. Three runs at one seed must agree on every line
except the two wall-clock lines; a differing turn count or outcome means process state
reached a decision, which is a bug.

### Rules-coverage queries

```bash
python plans/specdb.py stats
```

`specdb` joins the atomic-test corpus to the test suite and to the CR, so "what is covered"
is a query rather than hand-maintained prose. Also available: `next --phase`,
`show <ATOM-ID>`, `gaps --chapter N`, `orphans`, `suspicious`, and `owed` — the gate a
phase does not close until it is clean.

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
| 7 | The CR 613.8 cluster — dependency algorithm, board-wide sequential pass, memoization | ✅ 2026-09-06 |
| 5 | **Replacement and prevention effects (CR 614–616)** — Phases RA–RE | ✅ **2026-09-15** |
| 6a | CR 113.6 — which abilities function in which zone | 🟡 registration leg landed; the sweeps' zone leg is one PR away |
| 6 | **Triggered abilities (CR 603)** — takes LKI with it | 🔜 **next** — its architecture doc is written first |

Beside the spine, not sequenced against it: **"can't" effects** (RS-1 landed; RS-2–RS-4
open), **copy effects** (CV-1 landed; CV-2–CV-7 open), and the **Commander and
multiplayer track** — cost modification landed (CM-0–CM-4), CR 903.9a/b landed, N-seat
games run; the commander tax, `GameConfig::commander()`, designation and CR 802 remain.

**On the ordering:** replacement effects came *before* triggered abilities on purpose.
Triggers fire on events that *did* happen, so the performed-event stream had to be
post-replacement truth first — which it is now.
[`plans/replacement-architecture.md`](plans/replacement-architecture.md) §14 is that
phase in hindsight; what item 6 inherits from it is listed there.

---

## Project layout

```
mtgsim/src/
├── bin/            cli_play.rs, fuzz_games.rs
├── cards/          Card definitions (data only), one file per phase + registry.rs
├── engine/         actions (the chokepoint), cast, costs, mana, priority,
│                   resolve, sba, stack, targeting, turns, zones, keywords,
│                   leaving, zone_function (CR 113.6)
│   ├── combat/            validation, resolution, steps, keywords
│   ├── cost_determination/ gather, total  ← CR 601.2f
│   ├── layers/            board, compute, lookahead, condition, copy,
│   │                      cda, land_types, types  ← CR 613
│   └── replacement/       gather, instance, lookahead, pipeline  ← CR 614–616
├── events/         GameEvent + EventLog
├── objects/        CardData, AbilityDef, GameObject
├── oracle/         characteristics, legality, board, mana_helpers
├── state/          game, game_state, game_config, player, battlefield,
│                   duration_registry  ← shared by the three registries:
│                   continuous_effects, replacement_effects, restrictions;
│                   layer_memo, diagnostics
├── types/          ids, mana, effects, costs, cost_modification, replacement,
│                   restriction, card_types, colors, keywords, keyword_actions, zones
└── ui/             decision (trait), ask (typed bridge), choice_types,
                    cli, random, display, auto_payer, mana_window_stop
```

Integration tests live in `mtgsim/tests/`, one file per phase.

---

## Documentation map

Authority order — when two docs disagree, the higher one wins.

| Doc | Authoritative for |
| --- | --- |
| [`plans/state-of-play.md`](plans/state-of-play.md) | **The board** — generated, checked in CI: what landed, the counts, the debt, the open handoffs. Read first when picking up work |
| [`plans/codebase-state.md`](plans/codebase-state.md) | **Current state.** Beats every other doc, this README included; its Deferred Migrations section is the debt register |
| [`CLAUDE.md`](CLAUDE.md) | Invariants, conventions, commands, critical-path ordering |
| [`plans/layers-architecture.md`](plans/layers-architecture.md) | The layer system: type shapes, module layout, dependency algorithm |
| [`plans/replacement-architecture.md`](plans/replacement-architecture.md) | Replacement and prevention (CR 614–616): event vocabulary, the CR 616.1 pipeline, the phases as landed, §14 in hindsight |
| [`plans/cant-effects-architecture.md`](plans/cant-effects-architecture.md) | "Can't" effects (CR 101.2 / 614.17 / 613.11) |
| [`plans/copy-effects-architecture.md`](plans/copy-effects-architecture.md) | Copy effects (CR 707 / 712 / 708 / 729) and Layer 1 |
| [`plans/cost-architecture.md`](plans/cost-architecture.md) | Cost determination and modification (CR 601.2f–h, 118, 903.8) |
| [`plans/backlog.md`](plans/backlog.md) | Every mechanic off the critical path, one entry each: the surface that can't express it, size, what it blocks |
| [`plans/roadmap-v2.md`](plans/roadmap-v2.md) | The route narrative: why the spine is ordered as it is, stakes per segment |
| [`plans/handoffs/`](plans/handoffs/) | Half-finished work; a file here is an open plate |
| [`plans/atomic-tests/sessions/`](plans/atomic-tests/sessions/) | The spec corpus — atomic tests from a close read of the CR. Authored, never generated |
| [`MTG-Rules/versions/`](MTG-Rules/versions/) | The CR itself; `tmnt.txt` is the baseline the engine targets |
| [`plans/cards-unlocked-ledger.md`](plans/cards-unlocked-ledger.md) | Which cards each ticket unlocks |
| [`plans/engineering-practices.md`](plans/engineering-practices.md) | Process: `CLAUDE.md`'s line budget, the comment rule, the two card pools, phase sizing, the specdb gate |
| [`plans/glossary.md`](plans/glossary.md) | The vocabulary — what this codebase means by *subject group*, *rider*, *leg*, and the seven words that mean more than one thing |
| [`design_doc.md`](design_doc.md) | The original design. **Historical**, except its §636–664 algorithm, adopted verbatim by `layers-architecture.md` |
| `plans/archive/` | Superseded. Do not act on it |

`spec.sqlite` and the generated index files under `plans/atomic-tests/` come from
`specdb.py build` — never hand-edit them; fix the session file and rebuild.

`plans/glossary.md` is checked by `plans/check_glossary.py`: a term that stops
appearing in `mtgsim/src`, a watched word with no definition, or a word with two
meanings carrying one of them fails the build.

---

## Contributing

- Small commits. Commit messages explain *why* — they are part of the project record.
- One branch per unit of work → PR → merge with a merge commit, never squash. This project
  leans on its written record, and squashing discards per-commit messages.
- A bugfix must be shown to fail against the pre-fix tree before it is committed.
- New cards go in `mtgsim/src/cards/`, integration tests in `mtgsim/tests/`.
- A word the codebase invents gets an entry in [`plans/glossary.md`](plans/glossary.md)
  and a line on `check_glossary.py`'s watch-list, in the commit that coins it.
- Annotate tests with `// COVERS:` / `// COVERS-PARTIAL:` atom ids at write time. Never
  claim an atom a test does not prove — a false link is worse than a blank.

---

## License

See [LICENSE](LICENSE).
