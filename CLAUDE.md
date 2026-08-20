# MTG-Ichor — Magic: The Gathering rules engine

A correctness-first MtG rules engine in Rust, built to be fast and to expose clean
hooks for UIs and AI harnesses. The Comprehensive Rules are the source of truth;
behavior is implemented, not approximated.

Crate lives in `mtgsim/`. Edition 2024. ~22,600 lines across 68 `.rs` files.

## Commands

```bash
cd mtgsim && cargo test            # full suite — must stay green
cd mtgsim && cargo build --all-targets   # must produce ZERO warnings
cd mtgsim && cargo run --bin cli_play    # play a game at the terminal
cd mtgsim && cargo run --bin fuzz_games  # random-vs-random batch harness
python plans/specdb.py stats       # rules-coverage by phase
```

**Zero warnings is a hard bar.** `cargo build --all-targets` must print none before
any commit.

## Document hierarchy — read in this order

| Doc | Authority |
|---|---|
| `plans/codebase-state.md` | **Current state. Wins over every other doc** — but it is a *generated* audit snapshot, not an authored one. See the note below |
| `plans/layers-architecture.md` | Authoritative for the layer system: type shapes, module layout, sublayer enumeration, dependency algorithm |
| `plans/atomic-tests/sessions/*.md` | The spec corpus — 1,753 atomic tests from a close read of the CR. Authored, never generated |
| `MTG-Rules/versions/*.txt` | The CR itself. Ground truth. `tmnt.txt` = baseline, effective 2026-02-27 |
| `plans/handoffs/*.md` | Transient. Where to resume a half-finished phase. Delete when the work lands |
| `plans/roadmap.md` | **Historical.** Only its Tier-1 v1.0 definition and phase dependency graph are still valid — it carries a staleness banner |
| `plans/workflow-prompts.md` | **Historical.** Written for Windsurf/Cascade, an earlier toolchain. Ignore the tool-specific parts |
| `plans/archive/*` | Superseded. Do not act on it |

`plans/atomic-tests/global-test-index.md` and `phase-index-*.md` are generated from
the session summaries. Don't hand-edit them.

**codebase-state.md is generated, and that changes how to treat it.** It is an audit
snapshot produced by reading the code, so it is authoritative *about the moment it was
produced* and perishable after that — it sat four months stale once, reporting 433 tests
and "Layers not started" long after Layers had landed. When it drifts, **regenerate it
from the code rather than hand-patching prose**; patching is what let it drift. The
generating prompt was never recorded — capture it in `plans/` the next time it is
regenerated so the process is repeatable.

Its **Deferred Migrations** section is the part worth guarding: debt owed by
forward-looking scaffolding, which does not surface as a test failure until the
dependent system lands. Add a line there at commit time for every new stub or TODO.

## Where the project is

**Don't trust any progress claim in this file — check the code.** For what is actually
done, read `plans/codebase-state.md` and run `python plans/specdb.py stats`. The
*ordering* below is durable; the completion state is not, so it is deliberately not
recorded here.

Critical path to v1, in dependency order — each item needs the ones above it:

0. **Layers (CR 613) core** — registry, `EffectiveCharacteristics`, `compute_characteristics`, Layers 7b/7c/7d, 5, 4
1. **Layer 4 Part B** — `AbilityOrigin` + CR 305.7 ability stripping (see `plans/handoffs/ld-layer4.md`)
2. **Layer 6** — ability adding/removing (Humility)
3. **Layer 2** — control changing
4. **CR 613.8 dependency algorithm** — ordering is timestamp-only today
5. **Replacement effects (CR 614–616)** — stub hook at `engine/actions.rs:86-89`
6. **Triggered abilities (CR 603)** — insertion point at `engine/priority.rs:235`

Phase 8 (card breadth) and Phase 9 (formats) are **not** on the v1 path.

## The layer-system invariant

**Never read `obj.card_data.{types,subtypes,supertypes,colors,keywords}` for an
object on the battlefield or the stack.** Printed characteristics stopped equalling
effective characteristics when Layer 4 landed. Route through `oracle/characteristics.rs`:

```rust
has_type(game, id, CardType::Creature)      // not card_data.types.contains(..)
has_subtype(game, id, &subtype)
has_supertype(game, id, Supertype::Legendary)
has_permanent_type(game, id)
is_creature(game, id)
get_effective_power / _toughness / _colors / _types / _subtypes / _supertypes / _name
```

This bug class was real: an artifact animated by Ensoul Artifact couldn't be chosen
as "target creature", and a permanent made legendary never triggered the legend rule.
Regression tests live in `mtgsim/tests/layer_aware_queries_test.rs`.

**The one exemption:** cast-zone and play-from-hand legality — `engine/cast.rs`,
`engine/zones.rs`, `oracle/legality.rs`, `oracle/mana_helpers.rs`. Those run before
the object is a permanent, so layers have nothing to contribute. Those sites are
tagged `// PRE-LAYER ZONE:` in source. Don't "fix" them.

## Spec database

`plans/specdb.py` joins the atomic-test corpus to the Rust test suite and to the CR
so coverage is a query, not hand-maintained prose (which went four months stale
once already).

```bash
python plans/specdb.py build                              # rebuild after any change
python plans/specdb.py next --phase "Phase 5-Layers" --rule 613
python plans/specdb.py show ATOM-305.7-002                # a ticket you can implement
python plans/specdb.py gaps --chapter 6                   # CR rules the corpus never examined
python plans/specdb.py orphans                            # typo check on annotations
```

**Annotate at write time.** When a test proves an atom, say so directly above `#[test]`:

```rust
// COVERS: ATOM-305.7-001
// COVERS-PARTIAL: ATOM-613.4c-001
#[test]
fn test_something() { ... }
```

`COVERS` = the test builds the atom's whole scenario. `COVERS-PARTIAL` = it exercises
the rule but not that scenario. **Never claim an atom a test doesn't actually prove** —
a false link is worse than a blank, because the whole point is a number you can trust.
A test with no matching atom is normal: structural/plumbing tests have no CR content,
and coverage here measures *rules* coverage, not project completeness.

Read `stats` per-phase. The TOTAL row is noise — half the corpus is Phase 8/9 work
that isn't on the v1 path.

`spec.sqlite` is derived and gitignored. Never hand-edit it; fix a session file or an
annotation and rebuild.

## Git workflow

Per-phase branches → PR → merge to main. Branch names name the single phase they
carry (`feature/phase-LD-layer-4`, `chore/spec-db`). **Don't rename branches to
generalized names or push a name the user didn't choose.**

Use **merge commits or "Rebase and merge"**, not squash — this project leans on its
written record and squashing discards per-commit messages.

**`git log main..HEAD` lies about what's merged.** Squash-merges create a new SHA with
the same content, so already-merged commits still appear "ahead". Always check content:

```bash
git diff --stat origin/main HEAD          # the true delta
git merge-base --is-ancestor <sha> origin/main
```

`gh` is **not installed**. After pushing, hand over a compare URL:
`https://github.com/lcmaier/MTG-Ichor/compare/main...<branch>?expand=1`

## Conventions

- Small commits. Don't refactor speculatively.
- Commit messages explain *why*, not just what — they're part of the project record.
- Test cards go in `src/cards/phase_XX_cards.rs`; integration tests in
  `tests/phase_XX_integration_test.rs`.
- Every new forward-looking stub or TODO gets a line in the **Deferred Migrations**
  section of `plans/codebase-state.md` at commit time. That section is hand-written and
  is the most valuable thing in the doc set — nothing can derive it.
- When a fix claims to repair a bug, prove it: verify the new tests fail against the
  pre-fix tree (`git stash push mtgsim/src`) before committing.

## Maintaining this file

Keep it **durable**. Invariants, conventions, commands, and doc precedence belong here;
progress snapshots and counts do not, because they rot silently and this file is loaded
into every session — a stale claim here is worse than no claim.

Update it when something *structural* changes: a new invariant (like the layer-system
rule above), a workflow change, a new tool, a shift in doc authority. Don't update it
because a phase finished — `codebase-state.md` and `specdb` answer that, and they are
derived from the code rather than from memory.
