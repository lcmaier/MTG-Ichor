# MTG-Ichor — Magic: The Gathering rules engine

A correctness-first MtG rules engine in Rust, built to expose clean hooks for UIs and
AI harnesses. The Comprehensive Rules are the source of truth; behavior is implemented,
not approximated. Crate lives in `mtgsim/`, edition 2024.

## Commands

```bash
cd mtgsim && cargo test                   # must stay green
cd mtgsim && cargo build --all-targets    # must print ZERO warnings — hard bar
cd mtgsim && cargo run --bin cli_play     # play at the terminal; --bin fuzz_games is random-vs-random, --pool stress plays every card, --audit checks every trigger dispatch
python plans/specdb.py stats              # rules coverage by phase
python plans/check_claude_md.py && python plans/check_module_layout.py && python plans/check_glossary.py && python plans/check_state_of_play.py --check && python plans/check_rulings.py --check && python plans/check_string_literals.py && python plans/check_type_surface.py && python plans/check_clippy_allows.py && python plans/check_engine_map.py && python plans/check_egui_only_draws.py && python plans/check_invariants.py && python plans/check_practices_page.py   # all must pass
```

## Where authority lives

| Doc | Authority |
|---|---|
| `plans/state-of-play.md` | **Read this first when picking up work.** Generated board — the critical path verbatim, which phases each architecture doc records as landed, card/test counts, the Deferred Migrations debt, and every half-finished phase in `plans/handoffs/`. **Never hand-edit**: `python plans/check_state_of_play.py --write` regenerates it, `--check` fails CI when it is stale, and `--flight` prints branches and open PRs |
| `plans/codebase-state.md` | Current state; wins over every other doc. Hand-maintained — update it as part of the work that changes it. Its **Deferred Migrations** section is the one to guard: debt owed by forward-looking scaffolding, invisible to tests until the dependent system lands. Add a line for every new stub or TODO at commit time |
| `plans/*-architecture.md` | One per CR subsystem; each owns its type shapes, phase codes and open questions. `layers` (613) · `replacement` (614–616, `RA`–`RE`) · `cant-effects` (101.2/614.17/613.11, `RS-*`, supersedes L15) · `copy-effects` (707/712/708/729 + Layer 1, `CV-*`, supersedes D5) · `cost` (601.2f–h/118.7–9/613.11's cost half/903.8, `CM-*`/`CP-*`, supersedes L15's cost half) · `triggers` (603 + 117.5/405.3/500.6/605.1b/608.2a/800.4d, the CR 603.10 reader, `TR-*`) · `setup` (103, the scenario loader, the save, `SU-*`) · `permission` (601.3's allows/602.2/602.5/113.6/305.1–2/606.3, `PM-*`) · `mana` (106/605, 601.2g–h's existence half, `MA-*`) · `clone` (what a fork shares, floors 2 and 3 mid-turn, `CL-*`). **Start at `plans/engine-map.md`**: one event through all of them, and where each invariant below lives. **A new subsystem extends this row, never adds one** |
| `plans/cr-coverage-audit.md` | Whether the plan can *express* the frozen CR: the type-surface method, its calibration bar, the concept register, and the findings register. `specdb.py orphaned`/`audit` confirm it; they are not the instrument |
| `plans/backlog.md` | Everything off the critical path — one entry per mechanic: the surface that can't express the rule, rough size, what it blocks. **Not designs** — a mechanic graduates out to an architecture doc. Excluded from `orphaned`'s ownership set on purpose; §1 says why |
| `plans/engineering-practices.md` | **Every process rule, a sentence or two and a pointer each**: comments, names, sizing, measuring, cards and rulings, the gates and their tags. Cited by heading anchor (`engineering-practices.md#the-close-out`), never by number. Its reasoning is `plans/archive/engineering-practices-records.md`; the measurements are `plans/fuzz-record.md`, newest-first, a block per phase |
| `plans/atomic-tests/sessions/*.md` | The spec corpus — atomic tests from a close read of the CR. Authored; never generated. (`summaries/` is an authoring trail; nothing reads it) |
| `MTG-Rules/versions/*.txt` | The CR itself. `tmnt.txt` is the baseline the engine targets |
| `plans/handoffs/*.md` | Where to resume a half-finished phase. Delete when the work lands |
| `plans/cards-unlocked-ledger.md` | Which cards each ticket unlocks. Live — but its `L##`/`T##` labels are the **archived** plan's vocabulary, **not a queue**: a `T##` names work, never schedules it. Never cite one as the owner of anything; the owner is `backlog.md` §2, an architecture doc, or the critical path |
| `plans/roadmap-v2.md` | The route narrative: why the spine is ordered, card stakes per segment, milestones, sizing. Ordering authority stays with **Critical path to v1** below |
| `design_doc.md`, `plans/roadmap.md`, `plans/workflow-prompts.md`, `plans/archive/*` | **Historical — do not act on them** (`archive/*-landed.md` hold shipped phases' evicted bodies: accurate records, still not plans). Two things stayed live: design_doc's §636–664 algorithm (adopted by `layers-architecture.md`) and roadmap's D## deferred-item tables. The §8/§11 delta-log fork was resolved 2026-08-24 — trigger detection is the performed-action event stream (`codebase-state.md`) |
| `plans/references/*` | Research tooling, not rules authority. Fetch card text/rulings via Bash+curl with a UA header — Scryfall 403s `WebFetch` |

## The layer-system invariant

**Never read `card_data.{types,subtypes,supertypes,colors,keyword_flags,abilities}` for an object
in any zone** — route through `oracle/characteristics.rs`, and assume a new query needs a wrapper.

**Ability *indices* are part of it.** `activatable_abilities`, `priority.rs`'s re-derivation
by id and `put_on_stack.rs::activate_ability` must all index the *effective* list; migrating one alone
mis-activates silently (CR 305.7 grants abilities that exist in no `CardData`).

**An exemption is a tag in source, and `check_invariants.py` fails a read without one:** `// PRE-LAYER ZONE:`,
`// AS PRINTED:`, `// NO ROW REACHES:`, each in `engineering-practices.md#invariant-gates`. → `layers-architecture.md`.

## Registry membership is not effect existence

A continuous effect from a static ability applies only while its source still *has* that
ability, and CR 305.7 or Layer 6 can take it away without touching the registry.
`compute_characteristics` re-checks existence at **every layer**. Two rules follow; both have
already cost a redesign:

- **Don't reconcile the registry at state-mutation chokepoints** — deciding existence outside
  the layer walk needs an iteration cap and invents oscillation the CR does not have.
- **Nothing inside the layer pass calls the top-level entry**: a member's frame is read off the live board, and only a
  non-member's walk recurses, at a strictly lower ceiling (`Board::frame_of`). That is the termination argument, not an optimization. → `layers-architecture.md` §5.2.

## CDAs are never registry effects

CR 604.3a(3) makes "affects no other object" a *criterion*, so every CDA applies to exactly the
object that has it — no filter, no `ObjectSet`, no row. `engine/layers/cda.rs` applies them
off the object's own effective ability list at layers 4, 5 and 7a; registering one applies it twice.

`AbilityDef.is_characteristic_defining` asserts only what the ability's *text* satisfies. CR
604.3a(2) is provenance, owned by whoever writes the ability onto an object, and **a Layer 6
`GrantAbility` must clear it**. → `layers-architecture.md` §6.

## The chokepoint invariant

**Never mutate observable game state outside `perform_action`'s own arms.** A direct write is
invisible to CR 614 no matter how loudly it is *emitted* — the pipeline reads the proposal, not
the event. Propose with `execute_action` / `change_zone`; `check_invariants.py` fails any other write (`engineering-practices.md#invariant-gates`).

- **A `ZoneChange` names its `ZoneChangeAct` and every proposal its `EventCause`; no catchall.** Only
  the pipeline, the trigger matcher, the history writer and a resolution counting its own records ("discarded this way") branch on `act`; every emitter names its record's cause.
- **Performers are loud; callers check legality.** That makes CR 608.2b's partial resolution
  the *caller's* job — see `Primitive::Untap`/`Destroy`.
- **"Becomes" events fire on the transition only (CR 603.2e).** A redundant tap announces nothing.
- **Routing a sweep makes its order observable** — it needs `battlefield_ids_ordered` even
  where the old direct-write loop did not.
- **One performer, one emitter, and they are different functions.** `move_object` performs and
  announces nothing; `announce_zone_change` is the only emitter, and each caller performed the move
  it announces. A performer nests a proposal only if the outer event is real without it → §11 item 20.
- **A simultaneous rule needs `execute_actions`, not a loop.** CR 704.3's single event, 704.7's
  collapse and 615.7's shield allocation are unreachable from a loop; a *nested* call joins the
  enclosing batch unless the CR makes its events no *result of* the outer one (`// AUXILIARY-MOVE:`,
  CR 614.13). Two permanent write tags: `// CAST-ROLLBACK:` (CR 601.2a's move, silent both ways, loud at 601.2i) and `// NOT REPLACEABLE:` (a write no replacement effect can watch for, CR 614.1). → `replacement-architecture.md` §2.
- **A decision site may not hold outcome-bearing state off `GameState`.** → `codebase-state.md` item 40.

## The replacement pipeline (CR 614–616)

`apply_replacements` sits inside `execute_actions`, between the proposal and the mutation. Six
rules, each of which has already cost something. → `replacement-architecture.md` §4.1.

- **Never prompt a `DecisionProvider` with fewer than two candidates.** CR 616.1 chooses only
  among "two or more"; the short circuit is what keeps every test at zero prompts, so relaxing
  it is a design error, not a test fix.
- **A "can't" is not a replacement effect (CR 614.17).** Checked ahead of the pipeline, and it
  wins (CR 101.2) — `engine::restriction::is_prohibited`, never a `ReplacementDef`.
- **A static replacement ability is discovered off the *effective* ability list, never a
  registry.** One gate per reader of that list (`engine-map.md` §2.3), three legs each — printed, granted, copied; Layer 3 is the route left.
  **A new gather source, or a new route to that list, needs a leg on every gate** or it is dead.
- **Declining an optional is tracked separately from CR 614.5's applied set.** CR 903.9b is
  exempt *and* optional, so a decline recorded in the applied set is a hang.
- **Deciding is separated from performing (CR 704.3).** A batch decides every member against
  one board, then performs, then runs riders — where CR 101.4's APNAP ordering lives.
- **Riders resolve after the performed event, never mid-loop (CR 615.5)**, are unconditional
  once queued (CR 615.12), and carry the replaced event's applied set (CR 614.5) — a fresh one loops.

**Growth contracts, enforced in review.** `EventPattern` grows on one axis — an arm per
`GameAction` variant; `Rewrite` is a closed algebra, so a new arm needs the CR rule permitting
it; per-mechanic variety goes in `ReplacementDef.then`. **An arm the pipeline cannot apply is
worse than a missing one**, so both ship narrower than §3.2. → §2a (as built), §3.2a, §3.2b.

## Determinism at the decision boundary

**Never iterate a `GameState` hash field where the order is observable** — go through `battlefield_ordered` /
`battlefield_ids_ordered`, or sort; `check_invariants.py` fails one untagged. A `DecisionProvider` picks by *index* and
`HashMap` order differs per process. The key is `PermanentState::timestamp` — CR 613.7's, which 613.7e
reassigns from the same process-independent counter — never `ObjectId`; same for any collection reaching a choice, log **or count**.

**Randomness is owned, never ambient.** Nothing reachable from a game may call `rand::rng()`, and `clippy.toml`
refuses it; draw from `GameState.rng` or the provider's own `StdRng`. `::new()` and `reseed_from_entropy`
for interactive play, and an unseeded `fuzz_games` run's seed, are the deliberate opt-outs, each under `#[expect]`.

The regression is `tests/determinism_test.rs`; CI's three runs under three `MTGSIM_HASH_SEED`s are the end-to-end check.

## Critical path to v1

**This section owns the ordering and says nothing about progress** — landed status is derived onto
`state-of-play.md` from the architecture docs' ✅ headings, and a second copy here would go stale between
merges. Route order; numbers are stable labels; `roadmap-v2.md` §3a is the same route with the *why*.

1–5, 6a, 6b, 7, 7a — the layer system, replacement effects (CR 614–616) and CR 113.6's zones, in place. → `layers-architecture.md`, `replacement-architecture.md`
5b. "Can't" effects (CR 101.2/614.17/613.11): the phases its doc names, beside the spine; `cant-be-targeted` any time
5c. Copy effects (CR 707/712/708/729 + Layer 1): the phases its doc names, beside the spine; `merging-and-meld` back-stopped before Phase 8
6. Triggered abilities (CR 603): the rest of `roadmap-v2.md` §3a's A rows, then `information-model-design`, reviewed before any build
**After 6, in order** (`roadmap-v2.md` §3a, "From triggers to Phase 8", has each step's why):
`route-review`; four-player Commander end to end — `cant-be-targeted`; `cant-cast-play-or-activate`; `permission-design` with a card's characteristic sets designed beside it;
`command-zone-casting`; `game-formats`; `multiplayer-rules`; `information-model` — then what Phase 8 is gated on: `combat-restrictions`, `cant-pay-costs`, `combat-requirements`; `as-though-effects`'s permission half;
`hybrid-and-phyrexian-payment`, `remaining-costs`; `mana-ability-costs`, `any-color-plus-painlands-plus-signets`, `chosen-color-plus-could-produce`, `convoke-delve-improvise`, `mana-payment-solver`; `mana-provenance`; `ironworks-loop`;
`casting-from-other-zones`; `double-faced-cards`, then `split-and-adventure-style-cards`; `token-copies`, `spell-copies`; `face-down-and-morph`; `merging-and-meld`; `divided-among-targets`; `loop-detection`.
Then Phase 8 breadth → Phase 9 formats and multiplayer → Phase 10: GUI, AI harness, parallel play.

**v1 is two use cases** (owner, 2026-08-24): 4-player Commander through a GUI, and highly parallel
AI games over the CLI. Two-player Standard is a checkpoint, not the target — so **write new systems N-player-shaped from the start**.

## Spec database

`plans/specdb.py` joins the corpus to the test suite and the CR; `build` regenerates `spec.sqlite`
and the index markdowns — never hand-edit those, fix the session file and rebuild.
**Annotate at write time** (`// COVERS:` above the `#[test]`, `// COVERS-PARTIAL:` when it does not
build the whole atom), **never claim an atom a test doesn't prove**, and **a phase does not close
until `owed` is clean for it** — a gate, not a report. → `engineering-practices.md#the-spec-database`.

## Git workflow

One branch per unit of work → PR → merge to main; merge commit or "Rebase and merge", **never
squash** — the project leans on its per-commit record. **Size a phase before writing it, and split
it in the doc, not in the moment** → `engineering-practices.md#sizing-a-phase` owns the 1,500–2,500 band, the
one-consumer-per-PR rule and where review findings go. `gh` is installed; opening a PR is fine, but
**merging to main is the user's call** — hand over the URL unless they say otherwise that session.
A build closes with `/pr-handover`, a review round runs as `/review-round`, and the next PR's brief is `/next-brief`.

## Conventions

- Small commits; messages explain *why* — they're part of the project record.
- **Comment the *why*, and only where it is not recoverable from the code plus one rule
  number.** A war story goes in the commit message or the architecture doc, not the source.
  → `engineering-practices.md#comments`.
- **A count or `file:line` in a doc is generated, or it drives a decision**; otherwise describe
  it. → `engineering-practices.md#counts-in-docs`.
- Don't refactor speculatively. American spelling, gated by `check_glossary.py`.
- **Edit files with the Edit tool.** A counted sweep goes through `plans/replace_exact.py`, never a heredoc'd splice script.
- **Name for the call site:** the use alone must say what a name means, since its doc comment is
  not there. → `engineering-practices.md#names`.
- **Before deferring a rules question, read the rule that *watches* the one you are
  implementing** — it is in another chapter. → `engineering-practices.md#the-rules-pass`.
- Test cards in `src/cards/phase_XX_cards.rs`; integration tests in `tests/phase_XX_integration_test.rs`.
- **Dev GUI: logic is plain Rust under tests, egui only draws**; a GUI PR is reviewed by `engineering-practices.md#the-dev-gui`.
  **Register every card you write, and read its rulings** — `check_rulings.py` gates the second, a
  phase opening a new engine path adds one to `PERFORMANCE_POOL`. → `engineering-practices.md#cards-and-pools` and `engineering-practices.md#rulings`.
- A bugfix must be shown to fail against the pre-fix tree (`git stash push mtgsim/src`) first.

## Maintaining this file

**200 lines, hard**, checked by `python plans/check_claude_md.py`. Every invariant is at most
three lines plus a pointer — the reasoning, the war story and the rule numbers live in the
architecture doc. **Adding a section requires removing one**, and the removal is chosen against the
whole file, not against whatever happens to sit next to the edit. **No progress snapshots, counts or
✅ marks** — this file loads before every task, a stale claim here is worse than no claim, and status
belongs to `state-of-play.md`, derived from the architecture docs. → `engineering-practices.md#line-budgets`.
