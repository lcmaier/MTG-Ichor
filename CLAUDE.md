# MTG-Ichor — Magic: The Gathering rules engine

A correctness-first MtG rules engine in Rust, built to expose clean hooks for UIs and
AI harnesses. The Comprehensive Rules are the source of truth; behavior is implemented,
not approximated. Crate lives in `mtgsim/`, edition 2024.

## Commands

```bash
cd mtgsim && cargo test                  # must stay green
cd mtgsim && cargo build --all-targets   # must print ZERO warnings — hard bar
cd mtgsim && cargo run --bin cli_play    # play a game at the terminal
cd mtgsim && cargo run --bin fuzz_games  # random-vs-random batch harness
python plans/specdb.py stats             # rules coverage by phase
```

## Where authority lives

| Doc | Authority |
|---|---|
| `plans/codebase-state.md` | Current state; wins over every other doc. Hand-maintained — update it as part of the work that changes it, not in a later pass |
| `plans/layers-architecture.md` | The layer system: type shapes, module layout, sublayer enumeration, dependency algorithm |
| `plans/atomic-tests/sessions/*.md` | The spec corpus — atomic tests from a close read of the CR. Authored; never generated |
| `MTG-Rules/versions/*.txt` | The CR itself. `tmnt.txt` is the baseline the engine targets |
| `plans/handoffs/*.md` | Where to resume a half-finished phase. Delete when the work lands |
| `plans/cards-unlocked-ledger.md` | Which cards each ticket unlocks. Live; its `L##`/`T##` ticket vocabulary is defined in `plans/archive/implementation-plan-final.md` |
| `design_doc.md` (repo root) | The original design. Historical except §8/§11 (the delta-log proposal — an open fork, see `codebase-state.md` "Before Replacement effects") and the §636–664 algorithm, adopted verbatim by `layers-architecture.md` |
| `plans/roadmap.md`, `plans/workflow-prompts.md` | Historical. What still holds in the roadmap: Milestone 8 ("Commander Playable") and Phase 10 (GUI / AI API / parallel fuzz), which are the v1 target shape. Its phase graph and v1 definition are superseded by **Critical path to v1** below |
| `plans/archive/*` | Superseded. Do not act on it |
| `plans/references/scryfall-syntax.md` | External tooling reference (Scryfall search filters), not rules authority. Fetch card examples/rulings via Bash+curl with a UA header — Scryfall 403s the `WebFetch` tool |

Generated, never hand-edit — fix the source and rebuild:

- `spec.sqlite` — `python plans/specdb.py build`, from `sessions/`.
- `global-test-index.md`, `phase-index-*.md` — `python plans/atomic-tests/extract-phase-index.py`,
  run by hand. **It reads `summaries/`, not `sessions/`**, so a correction made to a
  session file does not reach the indexes. That drift channel is open and known;
  closing it (derive the indexes from `sessions/` or from the sqlite) is scheduled
  with the specdb backfill pass.

`codebase-state.md`'s **Deferred Migrations** section is the one to guard — debt owed by
forward-looking scaffolding, which doesn't surface as a test failure until the dependent
system lands. Add a line for every new stub or TODO at commit time.

## The layer-system invariant

**Never read `card_data.{types,subtypes,supertypes,colors,keywords,abilities}` for an
object on the battlefield or the stack.** Printed characteristics stopped equalling
effective ones when Layer 4 landed. Route through `oracle/characteristics.rs` —
`has_type`, `has_subtype`, `has_supertype`, `has_permanent_type`, `is_creature`,
`get_effective_*`, `get_effective_abilities`.

`abilities` joined the list in Phase LD Part B: CR 305.7 strips a Blood-Mooned land's
printed abilities and grants an intrinsic `{T}: Add {R}` that exists nowhere in its
`CardData`. Ability **indices** are part of this — `activatable_abilities` produces an
index, `priority.rs` re-derives it by id, and `cast.rs::activate_ability` consumes it. All
three must index the effective list; changing one alone mis-activates silently.

This was violated at 21 sites and produced silent wrong behavior, so it is not obvious:
assume any new query needs a wrapper.

**Exemption:** cast-zone and play-from-hand legality (`engine/cast.rs`, `engine/zones.rs`,
`oracle/legality.rs`, `oracle/mana_helpers.rs`) runs before the object is a permanent.
Those sites are tagged `// PRE-LAYER ZONE:`. Don't "fix" them.

**Second exemption:** `register_static_effects` (`state/game_state.rs`) reads printed
abilities on purpose — it runs inside `place_on_battlefield`, before the object's own
effect is registered, so computing effective characteristics there is circular. This is
safe because registration is not what decides whether an effect applies; see below.

## Registry membership is not effect existence

A continuous effect from a static ability applies only while its source still *has* that
ability, and CR 305.7 or Layer 6 can take it away without touching the registry. So
`compute_characteristics` re-checks existence at **every layer**, against the source's
frame as of the end of the previous layer (`EffectOrigin`, `static_ability_still_exists`).

Two rules follow, and both have already cost a redesign:

- **Don't reconcile the registry at state-mutation chokepoints.** Deciding existence
  outside the layer walk turns a structurally-terminating computation into a fixpoint that
  needs an iteration cap and invents oscillation the CR does not have.
- **The frame cache's descending layer ceiling is the termination argument**, not an
  optimization. `test_self_stripping_land_terminates_and_is_stable` overflows the stack if
  the existence check asks at the full ceiling. See `layers-architecture.md` §5.2.

## CDAs are never registry effects

CR 604.3a(3) makes "affects no other object" a *criterion* for being a
characteristic-defining ability, so every CDA applies to exactly the object that has it —
no filter, no `AffectedSet`, no row. `engine/layers/cda.rs` applies them off the object's
own effective ability list at layers 4, 5 and 7a, ahead of that layer's registry slice;
`register_static_effects` skips them and `ContinuousEffectRegistry::add` asserts nothing
lands in `Layer7aCdaPT`. Registering one as well would apply it twice.

`AbilityDef.is_characteristic_defining` asserts only what the ability's *text* satisfies.
CR 604.3a(2) is provenance and belongs to whoever writes the ability onto an object: a copy
or text-changing effect hands the flag along (correct), and **a Layer 6 `GrantAbility` must
clear it** — a granted ability is never a CDA. `layers-architecture.md` §6.

## Determinism at the decision boundary

**Never iterate `game.battlefield` directly where the order is observable — go through
`GameState::battlefield_ordered` / `battlefield_ids_ordered`.** A `DecisionProvider` picks
by *index*, so the order a sweep returns in is part of the decision. `HashMap` reseeds
`RandomState` per process, so a raw sweep hands the AI a different action list every run
and `fuzz_games --seed N` stops reproducing. Sorting by `ObjectId` is not a fix: ids are
v4 UUIDs. The deterministic key is `BattlefieldEntity::timestamp` — allocated once per
`place_on_battlefield`, never reassigned, and CR 613.7's order anyway. Order-irrelevant
sweeps (untap-all, clear-all-damage) may still iterate the map. Same rule for any other
collection whose order reaches a choice: `cards/registry.rs::card_names` sorts,
`engine/layers/land_types.rs::basic_land_types_sorted` sorts, and `sba.rs`'s legend-rule
grouping is a `BTreeMap`.

**Randomness is owned, never ambient.** `rand::rng()` is seeded from the OS; anything
reachable from a game must draw from `GameState.rng` (shuffles, and later coin flips) or
from the provider's own `StdRng`. `RandomDecisionProvider::seeded` and `Game::reseed` are
what make a run replayable; `::new()` and `reseed_from_entropy` are the deliberate
opt-outs, for interactive play. A `GameState` nobody reseeds still uses a fixed default
seed — reproducible-by-default, so tests that shuffle don't drift.

The regression lives in `tests/determinism_test.rs`. Note what it cannot cover: two runs
inside one process share one `RandomState`, so they agree whether or not the sweeps are
ordered. The end-to-end check is three `fuzz_games` runs at one seed from the shell: every line
must match except the two wall-clock lines (`Total time`, `Time/game`), which are machine
noise. A differing turn count or outcome means process state reached a decision.

## Critical path to v1

Dependency order — each needs the ones above. Ordering only; the reasoning for
it, and what's *done*, live in `codebase-state.md` (Deferred Migrations) and
`specdb stats`.

**This section owns the ordering.** `plans/roadmap.md`'s phase graph and
`specdb.py`'s `CRITICAL_PATH` point here; when they disagree, this wins.

1. Layers (CR 613) core → Layer 4 Part B → static-ability effect existence ✅
2. CDAs (CR 604.3 / 613.3 / 613.4a Layer 7a) — precedes 613.8, which reads CDA-ness ✅
3. Layer 6 — ability adding/removing (Humility) ✅
4. Layer 2 — control changing ✅
5. Replacement effects (CR 614–616) — stub hook in `execute_action`. Gates most real
   cards *and* Commander's 903.9 command-zone redirection
6. Triggered abilities (CR 603) — insertion point in `perform_sba_and_triggers`. Takes
   LKI formalization and conditional static abilities with it
7. The CR 613.8 cluster — dependency algorithm + the board-wide sequential pass
   (`codebase-state.md` item 8 step 4) + cross-call memoization
   (`layers-architecture.md` §12), as one phase. **Hard back-stop: before Phase 8 card
   breadth**, because a Commander-viable pool is dense in exactly the statics that
   interact. Until it lands, author no dependency-ordering-sensitive cards

Interleaved after 5 rather than sequenced against it: the Commander and multiplayer
track — cost modification (commander tax), 903.9, `GameConfig::commander()`, CR 800
priority/turn rotation/elimination, CR 802.

**v1 is two use cases** (owner, 2026-08-24): peer-to-peer human games through a GUI —
specifically **4-player Commander** — and **highly parallel AI games** over the CLI. A
correct two-player Standard game is a checkpoint on the way, not the target. So
multiplayer (CR 800/802), the Commander skeleton, and enough card breadth to make a
Commander game are all on the path. **Write new systems N-player-shaped from the
start** — CR 616.1's replacement ordering and CR 603's APNAP queue both take a player
set in the CR, and retrofitting a two-player assumption is the expensive path.

## Spec database

`plans/specdb.py` joins the atomic-test corpus to the test suite and to the CR, so
coverage is a query rather than hand-maintained prose.

```bash
python plans/specdb.py build                    # rebuild after any change
python plans/specdb.py next --phase "Phase 6"   # ticket queue
python plans/specdb.py show ATOM-305.7-002      # one ticket, implementable
python plans/specdb.py gaps --chapter 6         # CR rules the corpus never examined
python plans/specdb.py orphans                  # typo check on annotations
```

Annotate at write time, directly above `#[test]`: `// COVERS:` when the test builds the
atom's whole scenario, `// COVERS-PARTIAL:` when it exercises the rule but not that
scenario. **Never claim an atom a test doesn't prove** — a false link is worse than a
blank. Tests with no matching atom are normal; this measures rules coverage, not project
completeness. Read `stats` per phase; the TOTAL row is noise.

## Git workflow

One branch per unit of work → PR → merge to main.

Merge with a merge commit or "Rebase and merge", never squash — this project leans on
its written record and squashing discards per-commit messages.

`git log main..HEAD` lies after a squash-merge: same content, new SHA, so merged commits
still look ahead. Check content instead — `git diff --stat origin/main HEAD`.

`gh` is installed and authenticated. Opening a PR is fine; **merging to main is the
user's call** — hand over the URL unless they say otherwise that session. If gh reports
it isn't logged in, ask the user to run `gh auth login`; never handle their credentials.

## Conventions

- Small commits. Don't refactor speculatively.
- Commit messages explain *why* — they're part of the project record.
- Test cards in `src/cards/phase_XX_cards.rs`; integration tests in
  `tests/phase_XX_integration_test.rs`.
- A bugfix must be shown to fail against the pre-fix tree
  (`git stash push mtgsim/src`) before it's committed.

## Maintaining this file

Keep it durable: invariants, conventions, commands, where authority lives. No progress
snapshots or counts — this file loads into every session, so a stale claim here is worse
than no claim. Update on structural change (a new invariant, a workflow change, a shift
in doc authority), not because a phase finished.
