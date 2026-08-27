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
| `plans/codebase-state.md` | Current state; wins over every other doc. Hand-maintained — update it as part of the work that changes it. Its **Deferred Migrations** section is the one to guard: debt owed by forward-looking scaffolding, invisible to tests until the dependent system lands. Add a line for every new stub or TODO at commit time |
| `plans/layers-architecture.md` | The layer system: type shapes, module layout, sublayer enumeration, dependency algorithm |
| `plans/replacement-architecture.md` | Replacement + prevention (CR 614–616): event vocabulary, CR 616.1 pipeline, ETB look-ahead frame, RA–RE sequencing |
| `plans/atomic-tests/sessions/*.md` | The spec corpus — atomic tests from a close read of the CR. Authored; never generated. (`summaries/` is an authoring trail; nothing reads it) |
| `MTG-Rules/versions/*.txt` | The CR itself. `tmnt.txt` is the baseline the engine targets |
| `plans/handoffs/*.md` | Where to resume a half-finished phase. Delete when the work lands |
| `plans/cards-unlocked-ledger.md` | Which cards each ticket unlocks. Live; its `L##`/`T##` vocabulary is defined in `plans/archive/implementation-plan-final.md` |
| `design_doc.md`, `plans/roadmap.md`, `plans/workflow-prompts.md` | Historical. Still live from them: design_doc's §636–664 algorithm (adopted by `layers-architecture.md`) and roadmap's Milestone 8 / Phase 10 v1 target shape. The §8/§11 delta-log fork was **resolved against** 2026-08-24 — trigger detection is the performed-action event stream; see `codebase-state.md` |
| `plans/archive/*` | Superseded. Do not act on it |
| `plans/references/*` | Research tooling, not rules authority — prints numbers a human pastes into prose; generates nothing the project reads. Fetch card text/rulings via Bash+curl with a UA header — Scryfall 403s `WebFetch` |

## The layer-system invariant

**Never read `card_data.{types,subtypes,supertypes,colors,keywords,abilities}` for an
object on the battlefield or the stack.** Printed characteristics stopped equalling
effective ones when Layer 4 landed. Route through `oracle/characteristics.rs` —
`has_type`, `has_subtype`, `has_supertype`, `has_permanent_type`, `is_creature`,
`get_effective_*`, `get_effective_abilities`. This was violated at 21 sites with
silently wrong behavior, so it is not obvious: assume any new query needs a wrapper.

Ability **indices** are part of it — `activatable_abilities` produces an index,
`priority.rs` re-derives it by id, `cast.rs::activate_ability` consumes it. All three
must index the *effective* list (CR 305.7 strips a Blood-Mooned land's printed
abilities and grants one that exists in no `CardData`); migrating one alone
mis-activates silently.

**Exemptions:** cast-zone and play-from-hand legality (`engine/cast.rs`,
`engine/zones.rs`, `oracle/legality.rs`, `oracle/mana_helpers.rs`) runs before the
object is a permanent — tagged `// PRE-LAYER ZONE:`, don't "fix" them. And
`register_static_effects` (`state/game_state.rs`) reads printed abilities on purpose:
it runs inside `place_on_battlefield`, where computing effective characteristics is
circular — safe because registration is not what decides whether an effect applies.

## Registry membership is not effect existence

A continuous effect from a static ability applies only while its source still *has*
that ability, and CR 305.7 or Layer 6 can take it away without touching the registry.
`compute_characteristics` re-checks existence at **every layer**, against the source's
frame as of the end of the previous layer (`EffectOrigin`,
`static_ability_still_exists`). Two rules follow; both have already cost a redesign:

- **Don't reconcile the registry at state-mutation chokepoints** — deciding existence
  outside the layer walk turns a structurally-terminating computation into a fixpoint
  that needs an iteration cap and invents oscillation the CR does not have.
- **The frame cache's descending layer ceiling is the termination argument**, not an
  optimization. `test_self_stripping_land_terminates_and_is_stable` overflows the
  stack without it. See `layers-architecture.md` §5.2.

## CDAs are never registry effects

CR 604.3a(3) makes "affects no other object" a *criterion* for being a
characteristic-defining ability, so every CDA applies to exactly the object that has
it — no filter, no `AffectedSet`, no row. `engine/layers/cda.rs` applies them off the
object's own effective ability list at layers 4, 5 and 7a, ahead of that layer's
registry slice; `register_static_effects` skips them and `ContinuousEffectRegistry::add`
asserts nothing lands in `Layer7aCdaPT`. Registering one as well would apply it twice.

`AbilityDef.is_characteristic_defining` asserts only what the ability's *text*
satisfies. CR 604.3a(2) is provenance, owned by whoever writes the ability onto an
object: copy and text-changing effects hand the flag along, and **a Layer 6
`GrantAbility` must clear it** — a granted ability is never a CDA.
`layers-architecture.md` §6.

## The chokepoint invariant

**Never mutate observable game state outside `perform_action`'s own arms.** A
mutation written directly is invisible to CR 614 no matter how loudly it is
*emitted* — the pipeline reads the proposal, not the event. Lifelink was the
proof: it emitted `LifeChanged` and wrote `life_total` by hand, so a census of
emission sites showed it wired up while Tainted Remedy could never have seen it.
Propose with `execute_action` / `change_zone`; let the arm do the writing.

- **`ZoneChange` carries a `ZoneChangeCause`, and there is no catchall.**
  `(from, to)` cannot tell a sacrifice from a destruction. Every mover names its
  reason; a site with nothing honest to say does not belong on the chokepoint
  (`cast.rs::rollback_cast_to_hand` is what that looks like). Nothing may branch
  on `cause` outside the replacement pipeline (`EventPattern::ZoneChange`, its
  first real reader as of Phase RB) and the trigger matcher.
- **Performers are loud; callers check legality.** `perform_action` errors for a
  tap of something not on the battlefield. That makes CR 608.2b's partial
  resolution the *caller's* job — see `Primitive::Untap`/`Destroy`.
- **"Becomes" events fire on the transition only** (CR 603.2e). A redundant tap
  succeeds and announces nothing.
- **Routing a sweep makes its order observable.** CR 616.1 prompts when two
  effects want one event, so a loop that proposes actions needs
  `battlefield_ids_ordered` even if the old direct-write loop did not.
- **One performer, one emitter, and they are different functions.**
  `move_object` performs a zone change and announces nothing;
  `perform_action`'s `ZoneChange` arm is the only production emitter of
  `GameEvent::ZoneChange`, because it is the only place that knows the
  `cause` and the only place that can capture the CR 603.10a LKI frame
  *before* the object stops being a permanent. Emitting from the performer
  is how a CR 601.2 cast rewind spent a phase in the log claiming to be a
  real move.
- **A simultaneous rule needs `execute_actions`, not a loop.** CR 704.3's
  single event, CR 704.7's same-result collapse and CR 615.7's shield
  allocation are all unreachable from a loop of `execute_action` calls.
  A batch shares one `BatchId`; a *nested* call joins the enclosing batch
  rather than opening its own, because CR 120.3f makes lifelink's gain a
  *result of* the damage and CR 120.4c/d let the one damage event occur.

One exemption, tagged in `engine/zones.rs`'s `move_object` doc and permanent:
`// CAST-ROLLBACK:` — CR 601.2 rewinds are not events. (`// REPLACEMENT-BYPASS:`
is gone; RA-3 closed its three sites with `GameState::resolving`.)

## The replacement pipeline (CR 614–616)

`apply_replacements` sits inside `execute_actions`, between the proposal and the
mutation. Six rules, each of which has already cost something:

- **Never prompt a `DecisionProvider` with fewer than two candidates.** CR 616.1
  makes a choice only among "two or more"; the engine consequence is larger than
  the rule, because every test that reaches `execute_action` now traverses this
  loop and the one-candidate short circuit is what keeps that at zero prompts.
  Relaxing it to make something work is a design error, not a test fix.
- **A "can't" is not a replacement effect (CR 614.17).** It is checked ahead of
  the pipeline and it wins (CR 101.2) — `engine::replacement::is_blocked`, never
  a `ReplacementDef`. Filtering one out at the call site instead is the shape
  indestructible had before Phase RB: observationally right, and it left two
  call sites free to disagree while making CR 614.17c unreachable.
- **A static replacement ability is discovered off the *effective* ability list,
  never from a registry.** That is the same reasoning as "registry membership is
  not effect existence", and it is what makes Humility and Blood Moon strip a
  replacement ability for free. `GameState::replacement_ability_sources` is a
  *hint* that gates the sweep, not the answer — **add a new gather source and
  you must add it to the gate**, or the source is silently dead on every board
  the gate skips.
- **Declining an optional is tracked separately from CR 614.5's applied set.**
  CR 903.9b is `exempt_from_614_5` *and* optional, so a decline recorded only in
  the applied set is re-offered forever — a hang, not a wrong answer.
- **Deciding is separated from performing (CR 704.3).** A batch decides every
  member's replacements against one board, then performs, then runs riders. That
  is what "checks the conditions, then performs ... as a single event" means,
  and it is where CR 101.4's APNAP ordering of simultaneous choices lives.
- **Riders resolve after the performed event, never mid-loop (CR 615.5).** They
  are unconditional once queued (CR 615.12) and re-enter with a fresh
  applied-set. During the loop nothing has happened yet, so a rider run inside
  it runs before the event it rides on.

**Growth contracts, meant to be enforced in review.** `EventPattern` grows on
exactly one axis — one arm per `GameAction` variant; a pattern it cannot express
means the missing thing is a `GameAction` variant or a field on one. `Rewrite`
is a closed algebra: a sixth arm is a claim that CR 614/615 permits an operation
the list omits, and should arrive with the rule number that says so. Per-mechanic
variety goes in `ReplacementDef.then`, which is the existing `Effect` tree.

**An arm the pipeline cannot apply is worse than a missing one.** Both enums
ship narrower than `replacement-architecture.md` §3.2 specifies, because a
variant with no application path is a card that silently does nothing — and this
tree has paid for that failure mode more than once. They are matched exhaustively
and are not `#[non_exhaustive]`, so adding one is a normal diff that fails to
compile at every reader.

## Determinism at the decision boundary

**Never iterate `game.battlefield` directly where the order is observable — go through
`GameState::battlefield_ordered` / `battlefield_ids_ordered`.** A `DecisionProvider`
picks by *index*, so sweep order is part of the decision, and `HashMap` order differs
per process. Sorting by `ObjectId` is not a fix (v4 UUIDs); the deterministic key is
`BattlefieldEntity::timestamp` — allocated once per `place_on_battlefield`, never
reassigned, and CR 613.7's order anyway. Order-irrelevant sweeps (untap-all,
clear-all-damage) may still iterate the map. Same rule for any collection whose order
reaches a choice: sort it (`card_names`, `basic_land_types_sorted`) or use a
`BTreeMap` (the legend-rule grouping).

**Randomness is owned, never ambient.** Nothing reachable from a game may call
`rand::rng()`; draw from `GameState.rng` or the provider's own `StdRng`.
`RandomDecisionProvider::seeded` and `Game::reseed` make a run replayable; `::new()`
and `reseed_from_entropy` are the deliberate opt-outs, for interactive play. An
unseeded `GameState` still uses a fixed default seed — reproducible-by-default.

The regression is `tests/determinism_test.rs`, but two runs in one process share one
`RandomState`, so the end-to-end check is three `fuzz_games` runs at one seed from the
shell: every line must match except the timing lines (`Total time`, `Time/game`,
`CPU/game`). A differing turn count or outcome means process state reached a decision.

## Critical path to v1

**This section owns the ordering** — `plans/roadmap.md`'s phase graph and `specdb.py`'s
`CRITICAL_PATH` point here. Reasoning and what's *done* live in `codebase-state.md`.

1–4. Layers core, CDAs, Layer 6, Layer 2 — ✅ (2026-05 → 2026-08)
5. Replacement effects (CR 614–616) — execute from `plans/replacement-architecture.md`
   (phases RA–RE). **RA (the event spine) and RB (the CR 616.1 pipeline, with
   counters, regeneration and Commander's 903.9 pair) landed 2026-08-25/26. RC —
   ETB replacements — is next, and is the ~1,350-card unlock.**
6. Triggered abilities (CR 603) — insertion point in `perform_sba_and_triggers`. Takes
   LKI formalization and conditional static abilities with it
7. The CR 613.8 cluster — dependency algorithm + board-wide sequential pass +
   cross-call memoization, as one phase. **Hard back-stop: before Phase 8 card
   breadth.** Until it lands, author no dependency-ordering-sensitive cards

(Numbering is stable — other docs cite these items by number.)

Interleaved after 5: the Commander/multiplayer track — cost modification (commander
tax), `GameConfig::commander()`, CR 903.7 designation, CR 800/802. **903.9a/b are
done** (Phase RB), but unreachable until something sets `GameObject.is_commander`.

**v1 is two use cases** (owner, 2026-08-24): 4-player Commander through a GUI, and
highly parallel AI games over the CLI. Two-player Standard is a checkpoint, not the
target. **Write new systems N-player-shaped from the start** — retrofitting a
two-player assumption is the expensive path.

## Spec database

`plans/specdb.py` joins the atomic-test corpus to the test suite and the CR, so
coverage is a query rather than prose. `build` regenerates `spec.sqlite` and the index
markdowns from `sessions/` — never hand-edit those; fix the session file and rebuild.

```bash
python plans/specdb.py build                    # rebuild after any corpus change
python plans/specdb.py next --phase "Phase 6"   # ticket queue
python plans/specdb.py show ATOM-305.7-002      # one ticket, implementable
python plans/specdb.py gaps --chapter 6         # CR rules the corpus never examined
python plans/specdb.py orphans                  # COVERS ids that match no atom
python plans/specdb.py suspicious               # links that exist but look wrong
python plans/specdb.py owed                     # what a shipped phase left behind
```

Annotate at write time, directly above `#[test]`: `// COVERS:` when the test builds
the atom's whole scenario, `// COVERS-PARTIAL:` otherwise. **Never claim an atom a
test doesn't prove** — a false link is worse than a blank. `suspicious` is a smell
test: a hit means read it; silence proves nothing. Tests with no atom are normal —
this measures rules coverage, not completeness. Read `stats` per phase; TOTAL is noise.

**A phase does not close until `owed` is clean for it.** Every atom in the phase is
covered, or explicitly deferred with a reason written down. This is a gate, not a
report: Phase 5-Pre shipped carrying 223 atoms and zero coverage, one of which
specified the CR 400.7 `zone_change_epoch` field by name, and nothing asked — so the
design was lost for two years and rediscovered by hand. Add the phase to
`SHIPPED_PHASES` in `specdb.py` when it lands; that is what arms the gate.

Triage what `owed` reports as a **fact** or a **feature**, because they have opposite
economics. A *fact* — object identity, who cast this, an object's characteristics an
instant ago — is unrecoverable if not captured at the moment it exists, and adding it
later means re-threading every system built in between; record it on the first
customer. A *feature* — a filter leaf, an enum arm — is a normal diff whenever it
lands, so defer it freely and apply the two-customers guard. Count cards to decide
when to build a feature; never to decide whether to record a fact. Phase RA was, in
its entirety, a facts phase.

## Git workflow

One branch per unit of work → PR → merge to main. Merge commit or "Rebase and merge",
never squash — the project leans on its per-commit record.

**Size a phase before writing it, and split it in the doc, not in the moment.**
The project's implementation PRs run 1,500–2,500 additions; the ones that went
badly went past that. RB shipped at +5,475 across 33 files — 2.2× the largest
before it — and the cause was not the decision to keep it whole but that nobody
counted first. RA was split into three because someone counted call sites and
wrote a *Measured size* column; RB got nine bullets and no measurement, so it
ran until it was done. Sub-phases are numbered `RA-1`, `RC-2`, not lettered.

Two rules learned from that:

- **Every PR in a split carries at least one consumer of what it builds.** The
  tempting seam is "engine first, consumers after", and it is wrong: RB's
  pipeline commit was 1,306 lines with zero integration tests, because the
  consumers are what make a pipeline testable — and its one real defect was
  reachable only from the *last* consumer. Splitting relocates that risk rather
  than removing it, so plan for a later PR fixing an earlier one.
- **Review findings go to `plans/handoffs/<phase>-review.md`, not into a
  session.** Capture everything before fixing anything, triage into fix / defer
  / doc, then close one bucket per session starting cold from the file. A dozen
  unrelated fixes carried in one context is where quality degrades. `git log main..HEAD` lies
after a squash-merge (same content, new SHA); check content instead:
`git diff --stat origin/main HEAD`.

`gh` is installed and authenticated. Opening a PR is fine; **merging to main is the
user's call** — hand over the URL unless they say otherwise that session. If gh isn't
logged in, ask the user to run `gh auth login`; never handle their credentials.

## Conventions

- Small commits; messages explain *why* — they're part of the project record.
- Don't refactor speculatively.
- Test cards in `src/cards/phase_XX_cards.rs`; integration tests in
  `tests/phase_XX_integration_test.rs`.
- A bugfix must be shown to fail against the pre-fix tree
  (`git stash push mtgsim/src`) before it's committed.

## Maintaining this file

Keep it durable: invariants, conventions, commands, where authority lives. No progress
snapshots or counts — this file loads into every session, so a stale claim here is
worse than no claim. Update on structural change (a new invariant, a workflow change,
a shift in doc authority), not because a phase finished.
