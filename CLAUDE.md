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
| `plans/codebase-state.md` | Current state; wins over every other doc. **Generated**, so regenerate it rather than hand-patching when it drifts |
| `plans/layers-architecture.md` | The layer system: type shapes, module layout, sublayer enumeration, dependency algorithm |
| `plans/atomic-tests/sessions/*.md` | The spec corpus — atomic tests from a close read of the CR. Authored; never generated |
| `MTG-Rules/versions/*.txt` | The CR itself. `tmnt.txt` is the baseline the engine targets |
| `plans/handoffs/*.md` | Where to resume a half-finished phase. Delete when the work lands |
| `plans/roadmap.md`, `plans/workflow-prompts.md` | Historical. Only the roadmap's Tier-1 v1.0 definition and phase graph still hold |
| `plans/archive/*` | Superseded. Do not act on it |

Generated, never hand-edit: `global-test-index.md`, `phase-index-*.md`, `spec.sqlite`.
Fix the source and rebuild.

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
effect is registered, so computing effective characteristics there is circular.

## Critical path to v1

Dependency order — each needs the ones above. For what's *done*, read
`codebase-state.md` and run `specdb stats`.

1. Layers (CR 613) core → Layer 4 Part B (`AbilityOrigin` + CR 305.7 ability stripping)
2. Layer 6 — ability adding/removing (Humility)
3. Layer 2 — control changing
4. CR 613.8 dependency algorithm (ordering is timestamp-only today)
5. Replacement effects (CR 614–616) — stub hook at `engine/actions.rs:86-89`
6. Triggered abilities (CR 603) — insertion point at `engine/priority.rs:235`

Phase 8 (card breadth) and Phase 9 (formats) are **not** on the v1 path.

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

Per-phase branches → PR → merge to main. Branch names name the single phase they carry.
**Don't rename branches or push a name the user didn't choose.**

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
