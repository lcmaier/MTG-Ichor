# Handoff — the post-RE audit

**Opened 2026-09-15**, the day PR #140 (RE-9) merged and critical-path item 5
closed with it. Twenty-four replacement PRs landed between 2026-08-25 and
2026-09-15, interleaved with CM-0–CM-4, LH, LI, LJ, LK, CV-1 and RS-1, and the
owner asked for a comprehensive audit before the next spine item (A4c, then
the triggers doc). This file is the plan and the contract: the board lists it
under half-finished work until the last pass lands, each pass's PR updates
§7's status block, and **the PR that lands the last pass deletes it**.

An audit here means reading, measuring and giving each finding a disposition.
It ships docs and small fixes; anything larger becomes a scheduled item with
an owner, and the schedule is the owner's call.

## 0. Where things stand

- Branch `audit/post-re-plan`, off `main` at e4dae3c (RE-9's merge). This file
  and the regenerated board, nothing else.
- Every count below was read from the tree on 2026-09-15. A pass re-reads
  what it touches; a number here is a starting point, not a claim about the
  tree the pass finds.

## 1. The owner's list, evaluated

Nine topics were proposed. Six are audits, one is scheduling, one is a
written artifact rather than an audit, and one folds into another.

| Topic | Verdict | What the tree says (2026-09-15) |
|---|---|---|
| Replacement effects are *actually* done | **Keep; it is the gate.** | "Done" is defined in the tree and never collected in one place. `replacement-architecture.md` §11 has 108 numbered findings and three still marked **Open** — 3 (CR 614.15 self-replacement has a bucket and no producer), 4 (effects functioning off the battlefield, deferred past RE), 14 (does declining CR 903.9b exhaust its CR 614.5 exemption). §8a's known-missing-events table has four rows the 22-variant `GameAction` still lacks (turned face up, dice, search, counter). §12's out-of-scope list is stale in one direction: CR 614.12b was parked on cost modification, and CM-1–CM-4 have landed. §13 says the owed documents are ✅ **through RB**. `codebase-state.md`'s CR 614–616 row is 🟡. And the spec gate was never run for this phase: `specdb owed` scopes to three older phases by default — `state-of-play.md` says so itself — and Phase 6 stands at **127 atoms: 55 full, 22 partial, 50 without a test**. `owed --phase "Phase 6"` exists and has not been run at a close |
| `.clone()` calls | **Fold into performance.** | 193 in 62,241 lines of source; the densest files are `engine/resolve.rs` (24), `state/game_state.rs` (16), `engine/replacement/pipeline.rs` (15), `engine/cast.rs` (13), `engine/layers/compute.rs` (12). The count is low; what matters is what is cloned on a hot path, and a profile answers that where a grep cannot. The known levers are already on the record: the frame clone per dependency check (`fuzz-record.md`'s "Dependency checks" row), `codebase-state.md` item 67 (`Arc<Vec<AbilityDef>>`), item 136 (the chokepoint's fixed per-event cost), `layers-architecture.md` §12's 7a residual |
| General performance for parallel AI play | **Keep, and set the target first.** | No throughput or per-game budget is stated anywhere in `plans/`. The latest single-thread readings (RE-9's block, `fuzz-record.md`): **13.98 ms CPU/game at 2 seats, 44.83 ms at 4**; the harness scales ~6.7× on eight cores. `roadmap-v2.md` §E still says every number the project owns is two-player, which has been false since RE-7's four-seat tables. `serde` appears nowhere in `Cargo.toml` or `src/`; "serde-serializable by design" means designed so that it could be. The owner's answer to what the harness must do is in §4, with a measurement |
| A full replacement-effects trace page, with how it was built and what was learned | **Reshape.** | `engineering-practices.md` §7 makes trace pages per phase and pinned to a commit; a whole-system page is what §7.1 calls **tier 3, the codebase map**, "unblocked since the entry-hop fix landed (2026-09-02); a day to draw", and never drawn. The retrospective is evolving narrative and does not belong inside a pinned page. RE's trace-page decisions are recorded for RE-2 (yes) and RE-3 through RE-9 (no); the heading names neither RE-1 nor RE-10 |
| Comment hygiene | **Keep; overdue by the project's own rule.** | §2.1 sets the sweep at every 15–20 PRs. The last was the RB review's themes C and E at PR #62 (merged 2026-08-30); #140 is 78 PRs later. Comment lines are **17,012 of 49,645** non-card source lines (34%; `engine/` alone 32%). Twelve `TODO`s in `src/` cite the archived plan's vocabulary (`T12c`, `T22`, "Phase 4/5", "Phase 9"), which `CLAUDE.md` says is not a queue |
| Learning Rust through the project's design | **Not an audit; a written artifact.** | Nothing like it exists (the phrase "borrow checker" appears only in `design_doc.md`, which is historical). It wants the same tour the codebase map wants, so it rides behind that page rather than competing with the audit for a sitting. Home: `plans/references/`, which holds research tooling, not authority |
| CI/CD sufficiency | **Keep; one small PR.** | `.github/workflows/ci.yml` runs nine steps: build with `-D warnings`, test, fuzz both pools at 200 games, three-run determinism, the CLAUDE.md budget, module layout, and the board check. **Absent:** `check_glossary.py`, which `CLAUDE.md` lists among the four that must pass; `cargo clippy`, mentioned nowhere in the tree though one `#[allow(clippy::…)]` exists; `cargo fmt --check`, with no `rustfmt.toml`; the toolchain pin (1.98) is "not a measured MSRV" by its own comment. The spec database is out **on purpose** (owner, 2026-08-31) and stays out |
| The in-progress tracks (copiable values, can't effects, "as though") | **Scheduling, not auditing; one item with the next row.** | `CLAUDE.md` already places 5b and 5c beside the spine. RS-1 landed; RS-2, RS-3a, RS-3b, RS-4 are open, and the gate that held RS-3b (item 7) landed 2026-09-06, so nothing on either track blocks the triggers doc. CV-1 landed; CV-2–CV-7 open, CV-7 back-stopped before Phase 8. "As though" is `backlog.md` §2.24 — 287 cards, sized as a `Permission` mirror of `Restriction`, no architecture doc |
| Backlog and deferred analysis | **Keep, with the board as the instrument.** | `codebase-state.md` is 6,175 lines and its Deferred Migrations section is **5,961 of them (96%)**, 189 numbered items. The board flags **five reachable and wrong today** — 59 (Sutured Ghoul's P/T), 60 (Master Biomancer's Mutant clause), 118 (CR 514.3a's repeated cleanup announces nothing), 122 (CR 121.2c's draw order), 131 (a substituted zone change overwrites the replaced event's `cause`) — three of them RE's own — and **five with reachability not stated**: 1, 2, 88, 116, 121. The last triage was 2026-09-03, re-audited 2026-09-09 |

## 2. What the list was missing

- **Documentation staleness — and it goes first.** `CLAUDE.md` says
  `codebase-state.md` "wins over every other doc", and its TL;DR claims
  ~34,100 lines of Rust, 914 tests, "RC is under way" and "the dependency
  algorithm (CR 613.8) is not implemented". The tree has 62,241 source lines,
  1,534 tests, RE closed and LI-2/LI-3 landed. Every pass below produces
  findings, and today there is no trustworthy summary to put them under.
  `replacement-architecture.md` §13 and `roadmap-v2.md` §E are stale the same
  way. Total live `plans/` markdown is 28,386 lines beside 11,353 of archive,
  against 62,241 lines of Rust.
- **Panic surface.** A panic in a training batch is a lost game, and
  `fuzz_games` already exits 1 on one. Non-card, non-binary `src/` carries
  **341 `.unwrap()`, 21 `.expect(`, 14 `panic!`/`unreachable!`** — a count
  that includes the in-file unit tests, so the engine's own share is smaller
  and the pass must separate them. Belongs to pass 3.
- **The two v1 hooks.** The crate exists "to expose clean hooks for UIs and
  AI harnesses", and neither hook is on the list. `DecisionProvider` is four
  methods (`pick_n`, `pick_number`, `allocate`, `choose_ordering`) with nine
  implementations; `ChoiceContext` is one struct; `roadmap-v2.md` §9 already
  watches the trait's shape because Phase 10 serializes it. Belongs to pass 3.

## 3. The passes

Four passes, each its own PR, in this order. Pass 1 is sequential and gates the
rest; passes 2 and 3 are independent of each other; pass 4 needs 1 and 3.
Then two artifacts that need the whole picture. The roadmap reserves a
between-phases slot here for A4b (the rulings ledger); the audit shares it.

### Pass 1 — the close-out (the gate)

**Goal:** collect the done-checklist and give every entry a disposition —
closed, scheduled with an owner, or deferred with a dated reachability line
and a size. Then make the summaries true.

**Reads:** `replacement-architecture.md` §8a, §11 (the three Open items),
§12, §13; `codebase-state.md`'s TL;DR and CR 6 row; the board's two bolded
rows; `specdb owed --phase "Phase 6"`; §9's RE "Trace page" heading.

**Produces:** the checklist with dispositions, recorded under a dated
`### Found by the post-RE audit (2026-09-…)` heading in Deferred Migrations
for anything deferred; `codebase-state.md`'s TL;DR rewritten (not appended)
and the CR 614–616 row moved to ✅ with its own "not yet" list; §13 brought
current; §12 re-read against what landed since; the Phase 6 `owed` list
triaged atom by atom (covered elsewhere, owed, or deferred with an id); the
five wrong-today items each fixed or scheduled; the five unstated items each
given a verdict line. Fixes are their own commits, each shown to fail first
per `CLAUDE.md`; a fix that moves a pool is its own PR with a `fuzz-record.md`
block. The retrospective the owner asked for is written here, while the
material is fresh — see §6 for where.

**Size:** one sitting of reading, one docs PR, up to three small fix commits.
The brief is §8.

### Pass 2 — hygiene and CI

**Goal:** the §2.1 comment sweep by its own instrument, the twelve `TODO`s,
and the CI gaps.

**Reads:** `engineering-practices.md` §2, §2.1, §6; `ci.yml`.

**Produces:** the numbered-claim grep run over `src/` and `plans/`, each hit
re-derived and dated or rewritten as history; restating comments deleted
file by file in the files the grep touches; the twelve `TODO`s either given
a live owner (a backlog section, a critical-path item) or deleted;
`check_glossary.py` added to CI; a decision on clippy (a first run will
produce hundreds of findings on 62K lines, so adoption is a scoped choice,
not a flag flip) and on `cargo fmt --check`; the 1.98 pin either measured
down or left with its comment. The CI half is its own small PR.

**Size:** one PR for CI, one or two for comments, mechanical.

### Pass 3 — parallel-play readiness

**Goal:** answer whether the engine is on track for the AI-harness use case,
against a stated target, with a ranked lever list — not code.

**Reads:** `fuzz-record.md`'s newest block; `codebase-state.md` items 40, 41,
42, 67, 69, 136; `layers-architecture.md` §12; `roadmap-v2.md` §E; §4 below.

**Produces:** the target, written down; a profile at four seats and one
thread on the stress pool, with the clone question read off it; the panic
surface separated into engine and test; a state-clone measurement at
Commander scale (four 100-card decks) extending §4's table; the fork model
decided from the numbers (§4 recommends one); the `DecisionProvider` and
`ChoiceContext` serialization question answered (which fields, which
crate, what it costs on the straight-line path); each lever sized and
ranked against the target. Findings land in Deferred Migrations with
reachability and size; the fork model, if it holds, is the seed of the
harness's own architecture doc, which is Phase 10's and not this pass's.

**Size:** one sitting of measurement, one docs PR.

### Pass 4 — scheduling

**Goal:** what sits between now and the triggers doc, and in what order.

**Reads:** `cant-effects-architecture.md` §7.1, `copy-effects-architecture.md`
§7.1, `layers-architecture.md` §13c, `backlog.md` §2.24, `roadmap-v2.md` §3a
rows A4b, A4c, A6, and pass 1's and pass 3's outputs.

**Produces:** one table — each open track phase, what it unlocks in cards
and atoms, what it needs, and which of the triggers doc's questions (the
four problems `state-tracking-architecture.md` names, CR 603.10a's
visibility seam) needs it first; a proposed order for the between-phases
slot; and a recommendation on whether this audit becomes a recurring
practice at each spine-phase close (a §9 of `engineering-practices.md`,
with this run as its first instance). The owner decides the order; this
pass only proposes it.

**Size:** one docs PR.

### After the passes

- **The codebase map** — `engineering-practices.md` §7.1's tier 3, drawn
  with the CR 614 pipeline at its center. It absorbs the owner's "full
  replacement trace page": the chokepoint's arms, the three gate legs, the
  pipeline's loop, the look-ahead frame, the decision sites item 40 tracks.
  With the ten-line check §7.1 asks for, so the arm list cannot rot.
- **The Rust notes** — `plans/references/rust-through-the-engine.md` or
  similar: why the chokepoint is a function and not a trait, what the
  borrow checker forced (the accessor pair, `FrameCache`'s overlay instead
  of a clone, `ActionContext`'s plumbing), where `Arc` sits and why, what
  `Cell` buys `EngineCounters`, the `test-support` feature trick in
  `Cargo.toml`. Written for a reader coming from another language, after the
  map, because it is the same tour annotated.

## 4. The harness question, answered with a number

**The owner's requirement (2026-09-15):** the harness should be as flexible as
possible as a research tool for professionals — both high throughput and a
fast make/unmake path — and the worry was that a full `GameState` clone is
"probably just too slow outright".

**Measured the same day**, on a throwaway probe (not committed): random
decks from `performance_pool()`, `RandomDecisionProvider`, `GameState::clone()`
timed over 2,000 clones at each checkpoint, release build, one thread.
"no log" is the same state with `events.clear()` first.

| Seats, seed | Turn | Objects | On battlefield | Events logged | µs / clone | µs / clone, no log |
|---|---:|---:|---:|---:|---:|---:|
| 2, 12345 | 0 | 120 | 0 | 31 | 1.9 | 1.4 |
| 2, 12345 | 20 | 120 | 23 | 727 | 27.0 | 2.3 |
| 2, 12345 | 29 (over) | 120 | 32 | 1,198 | 25.9 | 2.7 |
| 4, 12345 | 0 | 240 | 0 | 59 | 3.1 | 2.6 |
| 4, 12345 | 20 | 240 | 19 | 548 | 7.7 | 3.8 |
| 4, 12345 | 40 | 180 | 32 | 1,344 | 21.5 | 3.7 |
| 4, 777 | 30 | 240 | 28 | 859 | 12.1 | 4.0 |
| 4, 777 | 60 | 180 | 33 | 2,136 | 32.7 | 3.5 |

**Reading.** A clone is **2–4 µs flat** once the event log is out of it,
at any turn and either seat count, and every microsecond above that is the
log — which is `codebase-state.md` **item 42** ("`EventLog` is on `GameState`
and grows monotonically"), already on the record as the window the harness
needs. Against 44.8 ms of CPU for a whole four-seat game, a fork costs
roughly one ten-thousandth of a game, and the mutation between two decisions
(a resolution, its replacements, the SBA batch) costs more than the fork
does. `CardData` is already behind `Arc`, so a clone copies object *state*,
not cards; `LayerMemo` is `RefCell<HashMap<_, (u64, Arc<_>)>>`, cheap to
copy and correct to copy, since the epoch travels with it.

**So the answer is: both are realistic, and the fast path is clone-fork, not
undo-journal.** A chess-shaped incremental unmake is not the right model
for a CR-faithful engine — one decision cascades through arbitrary
mutations across every registry, the memo epoch and the RNG, and an undo
record would have to invert all of it, for a saving bounded by the 3 µs a
clone already costs. The chokepoint invariant is what makes a journal
*possible*; item 42 is what makes a clone *cheap*; and the second is the
smaller build. What the trigger phase must not do is add outcome-bearing
state anywhere but `GameState` (item 40's invariant) — that is the one
design constraint with a deadline.

**Still open, and pass 3's to measure or decide:**

- Memory per fork at Commander scale (four 100-card decks, ~40 permanents),
  and the clone at that scale — the table above is 60-card random decks.
- **Where a search forks.** Decisions are asked through `DecisionProvider`
  *inside* engine calls, mid-resolution and mid-batch, while the engine holds
  `&mut GameState`; the provider sees a `ChoiceContext`, not the state. Two
  shapes: fork only at priority boundaries (item 41's test, "probably sound
  today") and answer inner asks from a policy; or a suspend-at-decision mode
  in the engine. The first is the v1 recommendation — most inner asks are
  orderings and replacement choices — and the second is what "flexible for
  professionals" may eventually require. Name the cost of each; do not build
  either in the audit.
- The straight-line throughput target, which nothing states. Suggested shape:
  games per core-second at four seats with random providers, as a floor the
  fixture table can watch — the harness's own decision cost will dominate any
  real training run, so the engine's job is to stay off the profile.

## 5. Where findings land

The way the project already records them, nothing new:

- A deferred finding: a numbered item under a dated `### Found by the post-RE
  audit` heading in `codebase-state.md`'s Deferred Migrations, with a
  `**Reachability (date):**` line and a `**Sized:**` line, per the board's
  rules.
- A finding without a surface: a `backlog.md` §2 entry.
- A stale claim: rewritten in place, never appended to.
- A rule the audit changes: `engineering-practices.md`, and `CLAUDE.md` only
  if a section can be removed for it.
- The audit's own record, when this file is deleted: a `### … — audited
  2026-09-…` heading in `codebase-state.md`, the shape "Was the critical path
  complete? — audited 2026-08-27" already uses.

## 6. Decisions left to the owner

Named here, not decided. Each pass that meets one asks and proceeds on the
recommendation if no answer has arrived.

1. **Where the retrospective lives.** Recommendation: a short §14 of
   `replacement-architecture.md`, "The phase in hindsight" — under 150
   lines, pointers not prose, since §11's findings already hold the detail
   item by item. Alternative: the top of
   `plans/archive/replacement-architecture-landed.md`, which nothing reads.
2. **Clippy.** Adopt with a scoped allow-list, adopt only for new code via a
   CI diff gate, or leave out. Recommendation: run it once in pass 2, count,
   and decide on the count.
3. **The throughput target** (§4's last bullet), before pass 3 profiles.
4. **The fork model** (§4), after pass 3's numbers.
5. **Whether the audit recurs** at each spine-phase close (pass 4 proposes).

## 7. Status

| Pass | State | PR |
|---|---|---|
| Plan | this file | — |
| 1 — close-out | not started | |
| 2 — hygiene and CI | not started | |
| 3 — parallel-play readiness | not started | |
| 4 — scheduling | not started | |
| Codebase map | not started | |
| Rust notes | not started | |

## 8. Pass 1 brief

Paste this to start pass 1. Read this file first; it may be newer than the
paste.

> **Pass 1 of the post-RE audit — the close-out.** Branch `audit/close-out`,
> off `main`. `plans/handoffs/post-re-audit.md` is the contract; §3's pass 1
> and §5 say what lands where.
>
> **Read first, in this order:** `plans/state-of-play.md` (the two bolded
> debt rows); `replacement-architecture.md` §11 items 3, 4 and 14, then §8a
> from "Known-missing kinds", then §12, then §13; `codebase-state.md`'s TL;DR
> and the CR 614–616 row in the CR 6 table; §9's RE "Trace page" heading;
> `engineering-practices.md` §5 and §5.1 (the `owed` policy).
>
> **Verified against the tree, 2026-09-15:** `GameAction` has 22 variants and
> lacks §8a's turned-face-up, dice, search-a-library and counter-a-spell rows;
> discard is `ZoneChange { cause: Discarded }` with RE-8's producer. §11
> has 108 numbered findings, three Open. §12 has six bullets; its CR 614.12b
> bullet was parked on cost modification, which landed as CM-1–CM-4. §13
> reads "✅ through RB". Phase 6: 127 atoms, 55 full, 22 partial, 50
> uncovered; `python plans/specdb.py owed --phase "Phase 6"` is the query and
> has never been run at a close. The board's reachable-and-wrong items are
> 59, 60, 118, 122, 131; its unstated items are 1, 2, 88, 116, 121. The
> TL;DR claims 914 tests and ~34,100 lines; the tree has 1,534 and 62,241.
> RE's trace-page heading names RE-2 (yes) and RE-3–RE-9 (no); RE-1 and
> RE-10 are unnamed.
>
> **The checklist — every entry gets one of three dispositions** (closed,
> scheduled with an owner, or deferred with a dated reachability line and a
> size):
> 1. §11 items 3, 4, 14.
> 2. §8a's four missing event kinds, each against its card count and its
>    owning phase (CV-6 for face-down; CR 705 is ❌ in the CR map).
> 3. §12's six bullets, re-read against what landed since 2026-08-24.
> 4. The 50 uncovered Phase 6 atoms, one line each: covered by a test that
>    lacks its `// COVERS:` line (add it), owed (a test, in this PR if under
>    an hour), or deferred with the atom id and the reason.
> 5. Items 59, 60, 118, 122, 131: fix or schedule. A fix is its own commit,
>    shown to fail first (`git stash push mtgsim/src`); a fix that moves a
>    pool is its own PR with a `fuzz-record.md` block.
> 6. Items 1, 2, 88, 116, 121: a `**Reachability (2026-09-…):**` line each.
> 7. RE-1's and RE-10's trace-page decision, recorded on the heading.
> 8. The retrospective (handoff §6 decision 1; proceed on the recommendation
>    if unanswered).
>
> **Then make the summaries true:** rewrite `codebase-state.md`'s TL;DR in
> place; move the CR 614–616 row to ✅ with its own "not yet" list; bring
> §13 current; strike or amend §12's stale bullet; `roadmap-v2.md` §E's
> "every number is two-player" sentence.
>
> **Binding rules:** never claim an atom a test does not prove; comment the
> why only where the code plus one rule number does not recover it; do not
> refactor; American spelling; a bugfix fails first. No A/B unless a fix
> moves a pool.
>
> **Exit:** every checklist entry carries a disposition in the tree;
> `python plans/specdb.py build` then `owed --phase "Phase 6"` shows only
> atoms with a recorded deferral; the four checks each pass on their own exit
> code (`check_claude_md.py`, `check_module_layout.py`, `check_glossary.py`,
> `check_state_of_play.py --check`, after `--write`); `cargo build
> --all-targets` prints zero warnings and `cargo test` is green; handoff §7's
> pass 1 row updated with the PR number; PR opened, merge left to the owner.
