# MTG-Ichor — Magic: The Gathering rules engine

A correctness-first MtG rules engine in Rust, built to expose clean hooks for UIs and
AI harnesses. The Comprehensive Rules are the source of truth; behavior is implemented,
not approximated. Crate lives in `mtgsim/`, edition 2024.

## Commands

```bash
cd mtgsim && cargo test                   # must stay green
cd mtgsim && cargo build --all-targets    # must print ZERO warnings — hard bar
cd mtgsim && cargo run --bin cli_play     # play a game at the terminal
cd mtgsim && cargo run --bin fuzz_games   # random-vs-random; --pool stress plays every card
python plans/specdb.py stats              # rules coverage by phase
python plans/check_claude_md.py           # this file's 200-line budget — must pass
```

## Where authority lives

| Doc | Authority |
|---|---|
| `plans/codebase-state.md` | Current state; wins over every other doc. Hand-maintained — update it as part of the work that changes it. Its **Deferred Migrations** section is the one to guard: debt owed by forward-looking scaffolding, invisible to tests until the dependent system lands. Add a line for every new stub or TODO at commit time |
| `plans/layers-architecture.md` | The layer system: type shapes, module layout, sublayer enumeration, dependency algorithm |
| `plans/replacement-architecture.md` | Replacement + prevention (CR 614–616): event vocabulary, CR 616.1 pipeline, ETB look-ahead frame, RA–RE sequencing |
| `plans/cant-effects-architecture.md` | "Can't" effects (CR 101.2, 614.17, 613.11): the six enforcement points, `RestrictionDef`, RS-1–RS-4 sequencing. Supersedes ticket L15 |
| `plans/copy-effects-architecture.md` | Copy effects (CR 707, 712, 708, 729) and Layer 1: `CopiableValues`, the four producers, CV-1–CV-7 sequencing. Supersedes ticket D5 |
| `plans/engineering-practices.md` | Process: this file's budget, the comment rule, the two card pools, phase sizing, the specdb gate |
| `plans/atomic-tests/sessions/*.md` | The spec corpus — atomic tests from a close read of the CR. Authored; never generated. (`summaries/` is an authoring trail; nothing reads it) |
| `MTG-Rules/versions/*.txt` | The CR itself. `tmnt.txt` is the baseline the engine targets |
| `plans/handoffs/*.md` | Where to resume a half-finished phase. Delete when the work lands |
| `plans/cards-unlocked-ledger.md` | Which cards each ticket unlocks. Live; its `L##`/`T##` vocabulary is defined in `plans/archive/implementation-plan-final.md` |
| `design_doc.md`, `plans/roadmap.md`, `plans/workflow-prompts.md` | Historical. Still live from them: design_doc's §636–664 algorithm (adopted by `layers-architecture.md`) and roadmap's Milestone 8 / Phase 10 v1 target shape. The §8/§11 delta-log fork was **resolved against** 2026-08-24 — trigger detection is the performed-action event stream; see `codebase-state.md`. `plans/archive/*` is superseded: do not act on it |
| `plans/references/*` | Research tooling, not rules authority. Fetch card text/rulings via Bash+curl with a UA header — Scryfall 403s `WebFetch` |

## The layer-system invariant

**Never read `card_data.{types,subtypes,supertypes,colors,keywords,abilities}` for an object
on the battlefield or the stack** — route through `oracle/characteristics.rs`. Violated at 21
sites with silently wrong behavior, so assume a new query needs a wrapper.

**Ability *indices* are part of it.** `activatable_abilities`, `priority.rs`'s re-derivation
by id and `cast.rs::activate_ability` must all index the *effective* list; migrating one alone
mis-activates silently (CR 305.7 grants abilities that exist in no `CardData`).

**Two exemptions, tagged in source and not bugs:** `// PRE-LAYER ZONE:` (cast-zone and
play-from-hand legality, before the object is a permanent) and `register_static_effects`
(circular inside `place_on_battlefield`). → `layers-architecture.md`; `codebase-state.md`.

## Registry membership is not effect existence

A continuous effect from a static ability applies only while its source still *has* that
ability, and CR 305.7 or Layer 6 can take it away without touching the registry.
`compute_characteristics` re-checks existence at **every layer**. Two rules follow; both have
already cost a redesign:

- **Don't reconcile the registry at state-mutation chokepoints** — deciding existence outside
  the layer walk needs an iteration cap and invents oscillation the CR does not have.
- **The frame cache's descending layer ceiling is the termination argument**, not an
  optimization. → `layers-architecture.md` §5.2.

## CDAs are never registry effects

CR 604.3a(3) makes "affects no other object" a *criterion*, so every CDA applies to exactly the
object that has it — no filter, no `AffectedSet`, no row. `engine/layers/cda.rs` applies them
off the object's own effective ability list at layers 4, 5 and 7a; registering one applies it twice.

`AbilityDef.is_characteristic_defining` asserts only what the ability's *text* satisfies. CR
604.3a(2) is provenance, owned by whoever writes the ability onto an object, and **a Layer 6
`GrantAbility` must clear it**. → `layers-architecture.md` §6.

## The chokepoint invariant

**Never mutate observable game state outside `perform_action`'s own arms.** A direct write is
invisible to CR 614 no matter how loudly it is *emitted* — the pipeline reads the proposal, not
the event. Propose with `execute_action` / `change_zone`.

- **`ZoneChange` carries a `ZoneChangeCause`, and there is no catchall.** Every mover names its
  reason; only the replacement pipeline and the trigger matcher may branch on `cause`.
- **Performers are loud; callers check legality.** That makes CR 608.2b's partial resolution
  the *caller's* job — see `Primitive::Untap`/`Destroy`.
- **"Becomes" events fire on the transition only (CR 603.2e).** A redundant tap announces nothing.
- **Routing a sweep makes its order observable** — it needs `battlefield_ids_ordered` even
  where the old direct-write loop did not.
- **One performer, one emitter, and they are different functions.** `move_object` performs and
  announces nothing; `perform_action`'s `ZoneChange` arm is the only production emitter, because
  it alone knows the `cause` and can capture the CR 603.10a LKI frame.
- **A simultaneous rule needs `execute_actions`, not a loop.** CR 704.3's single event, 704.7's
  collapse and 615.7's shield allocation are unreachable from a loop; a *nested* call joins the
  enclosing batch. One permanent exemption, tagged in `move_object`'s doc: `// CAST-ROLLBACK:`
  — CR 601.2 rewinds are not events. → `replacement-architecture.md` §2; `codebase-state.md`.

## The replacement pipeline (CR 614–616)

`apply_replacements` sits inside `execute_actions`, between the proposal and the mutation. Six
rules, each of which has already cost something. → `replacement-architecture.md` §4.1.

- **Never prompt a `DecisionProvider` with fewer than two candidates.** CR 616.1 chooses only
  among "two or more"; the short circuit is what keeps every test at zero prompts, so relaxing
  it is a design error, not a test fix.
- **A "can't" is not a replacement effect (CR 614.17).** Checked ahead of the pipeline, and it
  wins (CR 101.2) — `engine::replacement::is_blocked`, never a `ReplacementDef`.
- **A static replacement ability is discovered off the *effective* ability list, never a
  registry.** `replacement_ability_sources` gates the sweep and is sound only until Layer 1/3
  exist: **a new gather source, or a new route to that list, needs a gate leg** or it is dead.
- **Declining an optional is tracked separately from CR 614.5's applied set.** CR 903.9b is
  exempt *and* optional, so a decline recorded in the applied set is a hang.
- **Deciding is separated from performing (CR 704.3).** A batch decides every member against
  one board, then performs, then runs riders — where CR 101.4's APNAP ordering lives.
- **Riders resolve after the performed event, never mid-loop (CR 615.5)**, are unconditional
  once queued (CR 615.12), and re-enter with a fresh applied-set.

**Growth contracts, enforced in review.** `EventPattern` grows on one axis — an arm per
`GameAction` variant; `Rewrite` is a closed algebra, so a new arm needs the CR rule permitting
it; per-mechanic variety goes in `ReplacementDef.then`. **An arm the pipeline cannot apply is
worse than a missing one**, so both ship narrower than §3.2. → §2a (as built), §3.2a, §3.2b.

## Determinism at the decision boundary

**Never iterate `game.battlefield` directly where the order is observable** — go through
`battlefield_ordered` / `battlefield_ids_ordered`. A `DecisionProvider` picks by *index* and
`HashMap` order differs per process. The key is `BattlefieldEntity::timestamp` (CR 613.7's
order anyway), never `ObjectId` (v4 UUIDs); same rule for any collection reaching a choice.

**Randomness is owned, never ambient.** Nothing reachable from a game may call `rand::rng()`;
draw from `GameState.rng` or the provider's own `StdRng`. `::new()` and `reseed_from_entropy`
are the deliberate opt-outs, for interactive play.

The regression is `tests/determinism_test.rs`, but two runs in one process share one
`RandomState`, so the end-to-end check is three shell `fuzz_games` runs at one seed, matching
line for line but the timing lines. → `codebase-state.md`.

## Critical path to v1

**This section owns the ordering** — `plans/roadmap.md`'s phase graph and `specdb.py`'s
`CRITICAL_PATH` point here. Reasoning and what is *done* live in `codebase-state.md`.

1–4. Layers core, CDAs, Layer 6, Layer 2 — ✅
5. Replacement effects (CR 614–616), phases RA–RE. RA and RB are in; **RC — ETB replacements
   — is next, and is the ~1,350-card unlock**
5b. "Can't" effects (CR 101.2/614.17/613.11), phases RS-1–RS-4, beside 5 rather than after.
   **RS-1 must land before RC-4**; RS-3 (combat) wants item 7 first
5c. Copy effects (CR 707/712/708/729 + Layer 1), phases CV-1–CV-7, beside 5 and 5b. **A copy
   row stores values, never a reference** — which is what keeps it off item 7. CV-2 needs RC-2
6. Triggered abilities (CR 603) — insertion point in `perform_sba_and_triggers`. Takes LKI
   formalization and conditional static abilities with it
7. The CR 613.8 cluster — dependency algorithm + board-wide sequential pass + cross-call
   memoization, as one phase. **Hard back-stop: before Phase 8 card breadth**; until it lands,
   author no dependency-ordering-sensitive cards

(Numbering is stable — other docs cite these items by number.) Interleaved after 5: the
Commander/multiplayer track — cost modification, `GameConfig::commander()`, CR 903.7, CR
800/802. **903.9a/b are done**, but unreachable until something sets `GameObject.is_commander`.

**v1 is two use cases** (owner, 2026-08-24): 4-player Commander through a GUI, and highly
parallel AI games over the CLI. Two-player Standard is a checkpoint, not the target, so
**write new systems N-player-shaped from the start**.

## Spec database

`plans/specdb.py` joins the atomic-test corpus to the test suite and the CR, so coverage is a
query rather than prose. `build` regenerates `spec.sqlite` and the index markdowns from
`sessions/` — never hand-edit those; fix the session file and rebuild.

**Annotate at write time** (`// COVERS:` above the `#[test]`, `// COVERS-PARTIAL:` when it does
not build the whole atom) and **never claim an atom a test doesn't prove**. **A phase does not
close until `owed` is clean for it** — a gate, not a report. → `engineering-practices.md` §5.

## Git workflow

One branch per unit of work → PR → merge to main. Merge commit or "Rebase and merge", never
squash — the project leans on its per-commit record.

**Size a phase before writing it, and split it in the doc, not in the moment.** PRs run
1,500–2,500 additions; RB ran to +5,475 because nobody counted first. Every PR in a split
carries at least one consumer of what it builds, and review findings go to
`plans/handoffs/<phase>-review.md`, one theme per session. → `engineering-practices.md` §4.

`gh` is installed and authenticated. Opening a PR is fine; **merging to main is the user's
call** — hand over the URL unless they say otherwise that session.

## Conventions

- Small commits; messages explain *why* — they're part of the project record.
- **Comment the *why*, and only where it is not recoverable from the code plus one rule
  number.** A war story goes in the commit message or the architecture doc, not the source.
  → `engineering-practices.md` §2.
- Don't refactor speculatively.
- Test cards in `src/cards/phase_XX_cards.rs`; integration tests in
  `tests/phase_XX_integration_test.rs`. **Register every card you write** — the frozen
  `PERFORMANCE_POOL` protects the baseline, not omission. → `engineering-practices.md` §3.
- A bugfix must be shown to fail against the pre-fix tree (`git stash push mtgsim/src`) first.

## Maintaining this file

**200 lines, hard**, checked by `python plans/check_claude_md.py`. Every invariant is at most
three lines plus a pointer — the reasoning, the war story and the rule numbers live in the
architecture doc. **Adding a section requires removing one.** No progress snapshots or counts:
this file loads into every session, so a stale claim here is worse than no claim.
→ `engineering-practices.md` §1.
