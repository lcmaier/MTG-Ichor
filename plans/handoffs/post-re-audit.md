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

- Branch `audit/post-re-plan`, off `main` at e4dae3c (RE-9's merge), PR #141.
  This file and the regenerated board, nothing else. The owner answered §6's
  five decisions on the PR the same day; the answers are recorded there and
  folded into §3 and §4.
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
material is fresh: **§14 of `replacement-architecture.md`, "The phase in
hindsight"** (§6 decision 1), pointers not prose, since §11's findings
already hold the detail item by item.

**And the eviction that goes with it** (the same decision): the owner wants
"a lot of that doc" moved to `plans/archive/replacement-architecture-landed.md`.
The live doc is 6,497 lines and the archive's landed doc 4,933; each RE PR
already evicted its own phase body under the ≤40-line stub rule, so what is
left is the pre-build reasoning — §1's verdict, §8b and §8c's sizing, §9's
two design checks (361 and 336 lines), §5b's corrections, and the closed
majority of §11's 108 findings. The rule for what stays: what a reader
changing the pipeline needs — §2a as built, §3's type surface, §4's loop,
§5d's overlay as built, §6, §7, §8 and §8a, the open §11 items, §12, §13,
§14. Plan it section by section before moving anything, and if it is a PR
of its own, open it second.

**Size:** one sitting of reading, one docs PR, up to three small fix commits,
and possibly a second docs PR for the eviction. The brief is §8.

**Done 2026-09-15**, branch `audit/close-out`, four commits (PR #142; the number is
in §7). (1) The board's instrument: "reachable but not wrong today" had been
read as a wrong answer (items 118, 131), RD-1's prose list at column 0 had
been counted as three items, and 88, 116 and 121 had verdicts whose first
word the board does not read — 186 items, 3 wrong today, 0 unstated; and
`SHIPPED_PHASES` gained Phase 6. (2) The Phase 6 `owed` triage, `backlog.md`
§3.3's second block: 50 uncovered, 21 `NEW` → 0; four atoms annotated (two
with one assertion added), two tests written (CR 400.7c whole, CR 611.2c
partial), twenty-two tickets re-filed with their owner and the old words kept.
(3) Item 118 fixed — one proposal inside the 514.3a loop, shown to fail
first, A/B IDENTICAL at two and four seats on both pools, so no
`fuzz-record.md` block. (4) The docs: `replacement-architecture.md` §8a's
four kinds dispositioned (turned face up → CV-6; dice → `backlog.md` §2.31;
search → §2.5's producer; countering → closed, a zone change with a cause),
§11 items 3 (deferred, sized, fixture-first), 4 (scheduled, A5's third PR)
and 14 (deferred, with the reading), §12 re-read and amended in place, §13
current, RE-1's and RE-10's trace decisions on the heading, §14 written
(~110 lines); `codebase-state.md`'s TL;DR rewritten and the CR 614–616 row
✅ with its "not yet" list, items 59/60/122/131 scheduled with owners, item
134 (CR 614.12b) opened; `roadmap-v2.md` §E's two-player sentence and A6's
CR 121.2c seam; `copy-effects-architecture.md` §4.6's CR 614.1e line. Two
numbers the brief had wrong, for the record: §11 holds **99** numbered
findings, not 108, and the missing kinds' counts are **3** turned-face-up
cards (not 2) and **7** legal dice replacements (of 9).

### Pass 1b — the eviction of `replacement-architecture.md` (planned 2026-09-15)

Sized against the tree at the close-out, section by section, before anything
moves — the live doc is ~6,700 lines with §14 and the audit's amendments,
the archive 4,933. The rule for what stays is §3's: what a reader changing
the pipeline needs.

| Section | Lines | Moves? | The stub keeps |
|---|---:|---|---|
| §0 The budget | 70 | stays — the doc's own rule | — |
| §1 Verdict | 76 | **moves** | the heading, the one-sentence verdict and its date, → archive |
| §2, §2a | 136 | stay — the spine, the types as built | — |
| §3.1–§3.4 | 543 | stay — the type surface; §3.2c's evidence table is the closed-algebra proof and stays with §3.2b | — |
| §4.1–§4.4 | 353 | stay — the loop | — |
| §5, §5a, §5c, §5d | 350 | stay — the frame as built | — |
| §5b | 54 | **moves** | the Thassa-boundary sentence `layers-architecture.md` §13b and §13c cite |
| §6, §7, §8 | 110 | stay | — |
| §8a | ~250 | stays, as amended | — |
| §8b, §8c | 132, 184 | **move** — sizing method, breadth axes | §8c's stub keeps its two rules cited from outside — "two customers before a leaf" (nine citations: `backlog.md`, `codebase-state.md`, `engineering-practices.md` §5, three source files) and "an arm the pipeline cannot apply is worse than a missing one" |
| §9 phase stubs, RC's "Why four", RD's and RE's "Why", "Out of", "Cut", "Trace page", "Exit criteria" | ~650 | stay — the decisions a reader of the stubs needs | — |
| §9 RD design check | 362 | **moves** | the heading and the seven decisions' one-line titles |
| §9 RD-5, RD "Measured" | 78, 25 | **move** — a closed gate and a table `fuzz-record.md` holds | the heading and the gate's verdict line |
| §9 RE design check | 337 | **moves** | the heading and the nine decisions' one-line titles |
| §9 RE "Measured" | 212 | **moves** — `fuzz-record.md` holds the tables | the heading and the two-line summary |
| §10 Testing | 43 | stays | — |
| §11 head | ~180 | the table stays; the `ZoneChangeCause` derivation (~165) **moves** | the catchall ban's one sentence |
| §11 items 1–99 | ~2,100 | the three open (3, 4, 14) stay; the 96 closed **move**, under the same twenty `### Found by …` headings in the archive | every heading, and under each one line per item — number, bold title, → archive — so `grep '^90\. \*\*'` still lands in the live doc (65 citations of "§11 item N" outside it) |
| §12, §13, §14 | ~200 | stay | — |

**Size: ~3,500 lines out, ~250 back in stubs; live ~3,400, archive ~8,400.**
One docs PR, mechanical but not blind: every heading byte-identical so
`grep` finds it, `check_glossary.py` on the result, and the eleven outside
citations of §5b and §8c read against their stubs. **Opens second, after
pass 1 merges, so the stubs point at a §14 that exists on `main`.**

**Done 2026-09-15 (#151). Live 6,725 → 3,401, archive 4,933 → 8,647**,
against this table's "~3,400 / ~8,400" — 3,324 lines out and 234 back in
stubs. Every "moves" row matched its span. Three of this table's numbers did
not, and the tree is what they are corrected against: §11's closed items sit
under **seventeen** `### Answered …` / `### Found by …` headings, not twenty;
the `ZoneChangeCause` derivation is **66** lines, not ~165 (the ~165 was §11's
whole head, whose other 128 lines are items 1–7, five of which moved); and
**"an arm the pipeline cannot apply is worse than a missing one" is not a
sentence in §8c** — it is `CLAUDE.md`'s, argued at §3.2a/§3.2b and quoted by
§9's RD-4 stub and "Cut, and argued", all of which stay — so §8c's stub keeps
one rule, "two customers before a variant", and not two. Verified: every line
removed from the live doc is present in the archive's additions, the only
archive text that is not a moved line is a heading or an *Evicted* note, each
moved heading has the same count in both files, and all 99 §11 items still
carry a `N. **title**` line so every outside citation resolves.

### Pass 2 — hygiene and CI

**Goal:** the §2.1 comment sweep by its own instrument, the twelve `TODO`s,
and the CI gaps.

**Reads:** `engineering-practices.md` §2, §2.1, §6; `ci.yml`.

**Produces:** the numbered-claim grep run over `src/` and `plans/`, each hit
re-derived and dated or rewritten as history; restating comments deleted
file by file in the files the grep touches; the twelve `TODO`s either given
a live owner (a backlog section, a critical-path item) or deleted;
`check_glossary.py` added to CI; **clippy run once and counted before
anything is decided** (§6 decision 2 — a first run on 62K lines will produce
hundreds of findings, so adoption is a scoped choice made on the count, not
a flag flip); a decision on `cargo fmt --check`; the 1.98 pin either measured
down or left with its comment. The CI half is its own small PR.

**Size:** one PR for CI, one or two for comments, mechanical.

**Done 2026-09-15**, two PRs off `main` after #142. **CI (`audit/ci`, PR
#143):** clippy run once and counted — 114 sites, 26 lints, 48 files on
1.98.0, not "hundreds" — then adopted at `-D warnings` with a `[lints.clippy]`
table allowing five lints with a reason each — none a refactor the code
owes — and the other 90 sites fixed (54 by `--fix`, 17 of them let-chains); `check_glossary.py` beside the
other three checks; `cargo fmt --check` not adopted, its cost (148 of 165
files, 2,441 hunks, no `rustfmt.toml`) written into the workflow for the
owner's switch; the 1.98 pin measured down to a floor of **1.88** (every
target, every test; the let-chains are why 1.85 cannot build it), recorded as
`rust-version` and the pin kept at 1.98 because the lint set was counted
there. A/B IDENTICAL on both pools at two and four seats. **Comments
(`audit/comments`, PR #144):** the §2.1 sweep by its three steps, with
the counts and what they meant recorded in `engineering-practices.md` §2.1 as
the instrument's first application — the wide grep is mostly "about" as a
preposition, so a tighter tier is written down beside it; 13 undated
measurements dated from blame or re-derived, one future-tense doc made
history, one never-derived estimate retired; every one of the twelve `TODO`s
given a live owner in place of the archived label or deleted, which filed one
new owner (`backlog.md` §2.32, mulligans — the CR map had the mulligan under
103.4 and it is 103.5 in the frozen CR); twelve British spellings fixed and
their forms added to the glossary gate; three stale line-number pointers in
`codebase-state.md` replaced with function names. The first cut was read as
a rewrite rather than a cut (97 comment lines out, 96 in), so §2.1's step 3
was then applied in full: every inline comment block in the files the two PRs
touch, read against the line beneath it — 144 lines out of 20 source files,
50 in where an archived label (`T12b`, `SPECIAL-8`, "Phase 6") needed a live
owner instead. One PR, not two: the whole diff is comment lines, well under
the split threshold.

**Then option 2, the same day**, after the owner read that result as still
too small: history narration out of every comment kind and §2's four-line
signal enforced on inline blocks, over every non-card source file, sized
against the tree first and split at §4's band — four stacked PRs (#146, #147,
#148, #149), 17,278 → 15,806 comment lines, twelve outgrown
comments corrected on the way, and one gate interaction (the glossary anchored
on a name the sweep deleted). `engineering-practices.md` §2.1's second record
has the numbers and the instrument that actually worked.

### Pass 3 — parallel-play readiness

**Goal:** answer whether the engine is on track for the AI-harness use case,
against a stated target, with a ranked lever list — not code.

**Reads:** `fuzz-record.md`'s newest block; `codebase-state.md` items 40, 41,
42, 67, 69, 136; `layers-architecture.md` §12; `roadmap-v2.md` §E; §4 below.

**Produces:** the target, written down; a profile at four seats and one
thread on the stress pool, with the clone question read off it; the panic
surface separated into engine and test; a state-clone measurement at
Commander scale (four 100-card decks) extending §4's table; **the fork
model is decided** (§4, §6 decision 4 — priority boundaries), so what this
pass owes is the enumeration of the inner asks and the cost of each way of
answering them, and item 41's fork test promoted from "probably sound" to a
requirement; the `DecisionProvider` and `ChoiceContext` serialization
question answered (which fields, which crate, what it costs on the
straight-line path); the target proposed in the metric §4 names, for the
owner to set; each lever sized and ranked against it. Findings land in
Deferred Migrations with reachability and size; the batched decision
boundary §4 describes is the seed of the harness's own architecture doc,
which is Phase 10's and not this pass's.

**Size:** one sitting of measurement, one docs PR.

**Done 2026-09-15**, branch `audit/pass-3-parallel-play`, one docs PR (the
number is in §7) plus one instrument, `plans/panic_surface.py`. Measured
rather than argued, with three throwaway probes around `fuzz_games`' own
decks and streams (draw for draw, so the games are the fixture tables'): a
counting provider for the decision census, a clone timer with a counting
allocator, and item 41's fork test — record every answer, clone at a round
start, replay, compare with ids masked. What landed where: **the target** —
`codebase-state.md` item 138, 20,000 decisions per core-second at four
seats on `performance` and 10,000 at Commander scale against 10,000 and
7,950 today, a decision being a prompt with two or more options (91.5% of
priority prompts are forced passes), the twelfth and thirteenth counters
sized at ~80 lines and not built, and the levers ranked — worker scaling
first for a batch (6.0× on eight cores at +31% CPU a game, measured), §12's
oracle traffic and the per-prompt candidate enumeration next pending the
profile, the event-log window first for the fork use case, item 136's fast
path last; **the profile, owed** — no sampling profiler runs unprivileged on
this machine, so a symbolized binary was built into `mtgsim/target-prof/`
and the recipe is below; **the panic surface** — item 142, 60 release-active
engine sites of four kinds, against the 375 unit-test unwraps the raw count
had folded in; **the clone at Commander scale** — item 143, §4's table
extended with bytes and allocations, 5–6 µs and 69–92 KB a clone without
the log, 125 µs and 786 KB with it; **the inner asks** — `backlog.md`
§2.22's table, 24 asks in three classes plus the residual (about 2–7 a game
at four seats: CR 616.1's affected player, a targeted discard, a legend
rule or commander SBA on someone else's permanent), with three of the 24
turn-based actions' own boundaries rather than parameters of anything;
**item 41** promoted to a requirement with its RNG question decided (a fork
carries both streams; determinization reseeds the branch's provider), and
the test itself found the one thing on the stack that should not be — item
139, the retry re-prompt's stale candidate list (779 forks, 741 identical,
all 38 divergences that one mechanism) — and the one entry point missing,
item 140; **serialization** — item 141, 36 types and 788 lines in the
closure, free on the straight-line path and dearer than the engine per
prompt out of process, with the `&GameState` parameter kept as the
observation hook; **item 69** closed and evicted. Three things this pass's
brief had wrong, corrected where they sat: `ChoiceOption` has 12 variants,
not 13; §4's "hundreds of decisions" is 2,544 prompts and 513 decisions;
and §4's "the provider sees a `ChoiceContext`, not the state" — every
method takes `&GameState`.

**The profile recipe, for whoever holds an administrator prompt.** `samply`
(installed 2026-09-15; a Mozilla tool, and `cargo install --locked samply`
is the hygienic spelling next time) over the symbolized build:
`CARGO_PROFILE_RELEASE_DEBUG=2 cargo build --release --bin fuzz_games
--target-dir target-prof` in `mtgsim/`, then from an elevated PowerShell
`samply record --save-only --rate 4000 -o prof_stress4.json.gz
.\target-prof\release\fuzz_games.exe --games 200 --seed 12345 --threads 1
--players 4 --pool stress`. The saved file is Firefox Profiler JSON, which
a script reads for inclusive and self time by function; the reading goes to
`layers-architecture.md` §12 and re-orders item 138's list. Docker and WSL
were considered for callgrind and declined for the toolchain surface each
adds; the counters are the exact half of the instrument already.

### Pass 4 — scheduling

**Goal:** what sits between now and the triggers doc, and in what order.

**Reads:** `cant-effects-architecture.md` §7.1, `copy-effects-architecture.md`
§7.1, `layers-architecture.md` §13c, `backlog.md` §2.24, `roadmap-v2.md` §3a
rows A4b, A4c, A6, and pass 1's and pass 3's outputs.

**Produces:** one table — each open track phase, what it unlocks in cards
and atoms, what it needs, and which of the triggers doc's questions (the
four problems `state-tracking-architecture.md` names, CR 603.10a's
visibility seam) needs it first; a proposed order for the between-phases
slot; and **the audit written up as a recurring practice** — §6 decision 5,
decided — as a §9 of `engineering-practices.md`, with this run as its first
instance, the cadence (each spine-phase close), the passes, and what each
pass's instrument is. The owner decides the order; this pass only proposes
it.

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

**The fork point — decided (owner, 2026-09-15).** Decisions are asked
through `DecisionProvider` *inside* engine calls, mid-resolution and
mid-batch, while the engine holds `&mut GameState`; the provider sees a
`ChoiceContext`, not the state [**corrected by pass 3, 2026-09-15:** the
trait hands every one of its four methods `game: &GameState`
(`ui/decision.rs`), and `RandomDecisionProvider` reads it for its tap
preference and its X value; the `ChoiceContext` is the *prompt*, and the
state parameter is the observation hook — `codebase-state.md` item 141
decides to keep it]. Of the two shapes — fork only at priority
boundaries and answer inner asks from a policy, or a suspend-at-decision mode
in the engine — the owner chose **priority boundaries**, and for a reason
stronger than the engineering one: an RL agent should never be handed a view
it cannot act on, so the observation boundary and the action boundary are
the same boundary, and that boundary is priority. Three consequences for the
engine, all pass 3's to write down:

1. `codebase-state.md` item 41's priority-boundary fork test is a
   requirement, no longer "probably sound today".
2. Every inner ask — targets, modes, payment, a replacement choice, an
   ordering — is either a *parameter of the action chosen at priority* (the
   action space is complete legal actions, sub-choices resolved) or answered
   by a policy the harness supplies; it is never a separate observation.
   [**Pass 3, 2026-09-15:** not a fact about the tree — `PriorityAction` is
   four variants carrying no sub-choice, and targets, modes, X, the costs
   and the payment are separate asks made after the action is chosen; the
   24 asks are classified in `backlog.md` §2.22's table, and three of them
   — attackers, blockers, damage assignment — are turn-based actions' own
   decisions, boundaries of their own rather than parameters of anything.]
   Pass 3 enumerates which asks are which. The residual to design for, not
   the rule: an ask that lands on the *other* player mid-resolution (CR
   616.1's affected player choosing among two or more, and CR 603.3b's
   ordering once triggers exist).
3. The observation is built from the per-viewer query (`backlog.md` §2.9),
   which is why that entry sits before Phase 8 on the route.

**The GPU question (owner, 2026-09-15): is "cores" even the right metric,
when a researcher will want GPUs for higher parallelism — and does that mean
bespoke code per vendor?** No to both, for a structural reason rather than a
tooling one. A GPU runs thousands of threads in lockstep on one instruction
stream; a branch that goes two ways inside a warp serializes it, and dynamic
allocation, hash maps, trait objects and recursion are either unavailable or
run at a small fraction of CPU speed. A CR-faithful rules engine is the
opposite shape: every event is a branch, every board is a different-length
cascade through the pipeline, and its state is `HashMap`s of `Arc`s. The
board games that do run on a GPU (Pgx's chess and Go in JAX, Brax, the GPU
Atari emulators) were written as fixed-size tensor states with masked
arithmetic in place of branches; putting this engine there would mean a
second engine, and it would not be this one.

What a researcher does with a GPU is run the **agent** on it — the policy
network — while the **environment** runs on CPU cores, many copies at once,
observations batched to the GPU and actions returned. That is the shape of
every large-scale RL system built on a complex environment (IMPALA,
AlphaStar over StarCraft II, EnvPool, Sample Factory), and the vendor
question disappears inside the researcher's framework: PyTorch and JAX
abstract CUDA, ROCm and Metal, and nothing in this repository ever addresses
a GPU. So cores are the engine's metric: throughput per core at four seats,
and how many cores — on one machine or many — a run can keep busy.

What the engine owes that architecture, and this is the concrete
consequence: a **batched decision boundary**. Advance N games each to its
next priority boundary, hand back N observations in a fixed encoding, accept
N actions — the vectorized-environment interface, which is the fork model
above with a batch dimension. Its pieces are all named already: the
per-viewer query is the observation, `ChoiceContext` serialized is the
prompt, `DecisionProvider`'s four methods are the action, and the binding (a
Python module over the crate, or a socket protocol) is Phase 10's harness
doc. Order of magnitude from today's numbers, for pass 3 to replace with a
measurement: a four-seat random game is ~45 ms of CPU across on the order of
hundreds of decisions, so the engine advances a game between decisions in
roughly 100 µs; a small policy net batched over a thousand games is about a
millisecond on a GPU; so a few dozen cores keep one GPU fed, and the engine's
job is to stay off the profile. [**Measured by pass 3, 2026-09-15
(`codebase-state.md` item 138):** a four-seat `performance` game is 2,544
provider prompts, of which **513** offer two or more options — 188 at
priority, the other 2,031 priority prompts being forced passes, and 325
inner asks — in 51 ms of CPU: 20 µs between prompts, **~100 µs between
decisions**, 10,000 decisions per core-second; at Commander scale 692 in
87 ms, 7,950 per core-second. So the estimate's µs was right and its count
was an order of magnitude low, and "a few dozen cores" becomes about a
hundred per GPU at today's rate, or fifty at the proposed target. The
harness rule that falls out: the batched boundary advances past a forced
prompt without an observation, or 92% of what it ships is a view with one
legal action — exactly the view §4's rule forbids handing an agent.]

**Still open, and pass 3's to measure or propose:**

- Memory per fork at Commander scale (four 100-card decks, ~40 permanents),
  and the clone at that scale — the table above is 60-card random decks.
  **Measured 2026-09-15, `codebase-state.md` item 143**, same method, the
  table extended with bytes and allocations per clone: four 100-card decks
  at 40 life, `stress`, seed 12345 — 400 objects, 41 permanents at turn 40
  and 64 at turn 80; **5.3–6.1 µs and 69–92 KB a clone with the log out,
  21–125 µs and 227–786 KB with it** (258 → 1,662 allocations); the log is
  the whole of the growth, as item 42 says, and at 60 cards the no-log clone
  reads 2.2–4.5 µs and 43–67 KB, so the board's size costs a fork about a
  microsecond and the game's length costs it a hundred.
- The throughput target, which nothing states, in the metric the paragraph
  above settles: **decisions per core-second at four seats** — the unit an
  RL loop consumes — with random providers, as a floor the fixture table can
  watch. Pass 3 proposes the number; the owner sets it. **Proposed
  2026-09-15, item 138: 20,000 at four seats on the 60-card `performance`
  board and 10,000 at Commander scale, against 10,000 and 7,950 today**,
  with the instrument that reads it — a decision is a prompt with two or
  more options, counted by a twelfth `EngineCounters` cell and read beside
  `CPU/game` — sized at ~50 lines and not built (§3's rule: measurement and
  docs, no engine code).

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

## 6. Decisions — answered by the owner on PR #141, 2026-09-15

Five were named when the plan was written; four are decided and one is
half-decided. The passes above carry each answer.

1. **Where the retrospective lives — decided.** §14 of
   `replacement-architecture.md`, "The phase in hindsight", under 150 lines,
   pointers not prose — **and** a larger eviction of that doc to
   `plans/archive/replacement-architecture-landed.md` alongside it. Pass 1;
   the section-by-section rule is in §3.
2. **Clippy — decided in method.** Run once and count before anything is
   decided; the count decides. Pass 2.
3. **The throughput target — metric decided, number open.** Cores are the
   engine's metric and the GPU is the agent's (§4); the unit is decisions
   per core-second at four seats. Pass 3 proposes the number; the owner sets
   it. The one thing still open in this file.
4. **The fork model — decided.** Priority boundaries, for the RL reason in
   §4: no view an agent cannot act on. Pass 3 enumerates the inner asks and
   promotes item 41's test to a requirement.
5. **Recurring — decided.** Yes, at each spine-phase close. Pass 4 writes it
   as `engineering-practices.md` §9 with this run as the first instance.

## 7. Status

| Pass | State | PR |
|---|---|---|
| Plan | this file, decisions answered | #141 |
| 1 — close-out | ✅ done 2026-09-15 — §3's "Done" block; item 118 fixed in it | #142 |
| 1b — the eviction | ✅ done 2026-09-15 — §3's table and the re-count under it; live 6,725 → 3,401, archive 4,933 → 8,647 | #151 |
| 2 — hygiene and CI | ✅ done 2026-09-15 — §3's "Done" block; clippy counted at 114 and gated, the floor measured at 1.88, the §2.1 sweep's first record; then the option-2 sweep (history out, four-line signal on inline blocks) over every non-card source file, §2.1's second record | #143 (CI), #144 (comments; merged into `audit/ci`, re-landed on main by #145), option 2: #146, #147, #148, #149, stacked in that order |
| 3 — parallel-play readiness | ✅ done 2026-09-15 — §3's "Done" block; the target proposed (`codebase-state.md` item 138), the fork test's one finding (item 139), item 69 closed, the ask table in `backlog.md` §2.22; the four-seat `stress` profile owed, recipe in §3's block | #153 |
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
> RE-10 are unnamed. `replacement-architecture.md` is 6,497 lines;
> `plans/archive/replacement-architecture-landed.md` is 4,933; every RD and
> RE phase body is already a ≤40-line stub, so the eviction below is the
> pre-build reasoning and the closed findings, not the phase bodies.
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
> 8. The retrospective: §14 of `replacement-architecture.md`, "The phase in
>    hindsight", under 150 lines, pointers not prose (handoff §6 decision 1,
>    decided).
> 9. The eviction plan for `replacement-architecture.md`, section by
>    section, before anything moves: what a reader changing the pipeline
>    needs stays live (§2a, §3, §4, §5d, §6, §7, §8, §8a, the open §11
>    items, §12, §13, §14); the pre-build reasoning (§1, §5b, §8b, §8c, §9's
>    two design checks) and §11's closed findings move to the landed doc
>    under the ≤40-line stub rule, each stub keeping the heading `grep`
>    finds. Size it; if it is a PR of its own, open it second, after the
>    close-out merges, so the stubs point at a §14 that exists.
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

## 9. Pass 2 brief

Written 2026-09-15 at pass 1's close, with every number read from the tree
that day. Paste this to start pass 2. Read this file first; it may be newer
than the paste.

> **Pass 2 of the post-RE audit — hygiene and CI.** Two PRs, both off `main`
> after #142 merges: `audit/ci` first (small), then `audit/comments`
> (mechanical; may split in two). `plans/handoffs/post-re-audit.md` is the
> contract; §3's pass 2 and §5 say what lands where.
>
> **Read first, in this order:** `engineering-practices.md` §2 and §2.1 (the
> comment rule and its instrument — the three-step sweep is the procedure);
> `.github/workflows/ci.yml` end to end *including its comments* — each step's
> reason is written there and the PR must not un-reason one; `CLAUDE.md`'s
> "Commands" (the four checks that must pass) and "Conventions"; the
> `codebase-state.md` Deferred Migrations header, "Item ids are section-scoped"
> (how to cite what a `TODO` points at); handoff §6 decision 2 (clippy: run
> once, count, then decide).
>
> **Verified against the tree, 2026-09-15:**
> - CI runs nine steps: build with `-D warnings`, test, the harness build, both
>   pools at 200 games, three-run determinism, the `CLAUDE.md` budget, module
>   layout, the board check. **Absent:** `check_glossary.py` (which `CLAUDE.md`
>   lists among the four that must pass), `cargo clippy`, `cargo fmt --check`.
>   The toolchain is pinned at 1.98.0 with a comment saying it is "not a
>   measured MSRV"; edition 2024's floor is 1.85.
> - **Clippy, run once** (1.98.0, `--all-targets`, the default lint set):
>   **114 sites, 26 lints, 48 files** — 82 in the library, 9 more in its unit
>   tests, 4 in `fuzz_games`, 19 across seven integration-test targets. The
>   top six: `collapsible_if` 37, `unnecessary_get_then_check` 16,
>   `too_many_arguments` 8, `empty_line_after_doc_comments` 6,
>   `doc_lazy_continuation` 6, `large_enum_variant` 4. Fifty-four of the
>   library's 82 are `--fix` suggestions; one `#[allow(clippy::…)]` exists.
>   "Hundreds" was wrong, so the question the count decides (§6 decision 2)
>   is *which* lints go to `-D` and which are allowed with a reason, not
>   whether.
> - **`cargo fmt --check`: 148 of 165 Rust files would change, 2,441 hunks.**
>   No `rustfmt.toml`. Adopting it means one whole-tree reformat commit first,
>   which rewrites every file's blame.
> - **The §2.1 instrument** — `grep -rnE "(roughly|about|one in|[0-9]+%|
>   [0-9]+ of [0-9]+|measured|counted)"` over comment lines: **456 in `src/`**
>   (133 of them in `cards/`) across 65 files; 140 in `tests/`; 1,031 in live
>   `plans/` (254 in `codebase-state.md`, 259 in `replacement-architecture.md`).
>   Most `plans/` hits already carry a date by convention, and the instrument's
>   step 2 is "re-derive or date" — an already-dated claim is done, an undated
>   one is the work. Comment lines are 17,016 of 49,653 non-card `src/` lines
>   (34%). The last sweep was the RB review's themes C and E at PR #62; #142
>   is 80 PRs later against a 15–20 cadence.
> - **Twelve `TODO`s in `src/`**, so none is rediscovered:
>   `combat/resolution.rs:150` ("Phase 4/5"); `combat/steps.rs:35` (the
>   Cartesian product's scaling); `layers/board.rs:832` (other P/T counter
>   kinds); `sba.rs:284` and `:511` (two that say a TODO *no longer* exists);
>   `targeting.rs:541` ("once T22 lands" — hexproof/shroud); `turns.rs:315`
>   and `:413` (`T12c`, `BlanketPersistenceSet`); `oracle/mana_helpers.rs:68`
>   (producer preference); `state/game.rs:121` (London mulligan);
>   `ui/ask.rs:71` ("Phase 9", `GameNumber`) and `:1277` (a `choose_ordering`
>   test). `T##` and "Phase N" are the archived plan's vocabulary, which
>   `CLAUDE.md` says is not a queue.
>
> **The checklist — every entry gets a disposition:**
> 1. **The CI PR** (`audit/ci`): `check_glossary.py` beside the other three
>    Python checks; a clippy step at the scope the count decides — the honest
>    first cut is `cargo clippy --all-targets -- -D warnings` with a
>    `[lints.clippy]` table in `Cargo.toml` allowing, one line of reason each,
>    the lints judged not worth their diff (`too_many_arguments` and
>    `large_enum_variant` are the candidates: they are design, not hygiene),
>    and the rest fixed in the same PR; `cargo fmt --check`'s cost stated (148
>    files, 2,441 hunks, one reformat commit) and the switch left to the owner
>    unless told; the 1.98 pin either measured down (one build and test on
>    1.85, the result written into the comment) or left with the comment
>    amended to say when it was measured. Every CI step keeps a comment giving
>    its reason, as the nine have.
> 2. **The comment PR(s)** (`audit/comments`): the §2.1 sweep by its three
>    steps — run the instrument, re-derive or date each undated `src/` hit
>    (start with the 323 outside `cards/`; the card files' 133 are mostly
>    Scryfall censuses that carry their date already), rewrite the stale ones
>    as history, delete restating comments in the files the grep touches —
>    file by file, never a blind sweep (§2's own words). Split by directory
>    when a PR passes ~1,500 lines of diff (`engineering-practices.md` §4).
>    The `plans/` half: undated claims only, live docs only.
> 3. **The twelve `TODO`s:** each gets a live owner written into the comment
>    in place of the archived label — a `backlog.md` §2 entry, a critical-path
>    item, or a Deferred Migrations item — or is deleted where the code has
>    moved past it (the two in `sba.rs` say so themselves).
> 4. **The instrument's own record:** `engineering-practices.md` §2.1 gains
>    this run's date and counts as its first recorded application, the way
>    `backlog.md` §3.3 records `owed`'s.
>
> **Binding rules:** comment the why only where the code plus one rule number
> does not recover it; do not refactor — a clippy fix that changes a signature
> is a refactor, and is allowed with a reason rather than fixed; American
> spelling; a behavior change is not this pass — a clippy fix that alters
> behavior is its own commit, shown to fail first, and one that moves a pool is
> its own PR with a `fuzz-record.md` block. Both pools' counters IDENTICAL to
> `main` at 2 and 4 seats (`plans/fuzz_ab.py --rounds 0`) is the check that
> nothing moved.
>
> **Exit:** CI green on each PR with the new steps in it; `cargo build
> --all-targets` zero warnings and `cargo test` green (capture the whole log
> to a file and grep it — a piped summary loses a FAILED line); the four
> checks each pass on their own exit code; the instrument re-run and its
> count recorded in §2.1; handoff §7's pass 2 row updated with the PR
> numbers; PRs opened, merge left to the owner.

## 10. Pass 4 brief

Written 2026-09-15 at pass 3's close, every number read from the tree that
day. Paste this to start pass 4. Read this file first; it may be newer than
the paste.

> **Pass 4 of the post-RE audit — scheduling.** One docs PR,
> `audit/pass-4-scheduling`, off `main` after pass 3's PR merges.
> `plans/handoffs/post-re-audit.md` is the contract: §3's pass 4 block is
> the goal, §5 says where findings land, §6 decision 5 is the
> recurring-audit decision, and **this is the last pass, so its PR deletes
> the handoff** (the file's own rule, first paragraph) and writes the
> audit's record as a `### … — audited 2026-09-15` heading in
> `codebase-state.md` (§5's last bullet). The two after-the-passes
> artifacts — the codebase map and the Rust notes — need a home for their
> status before the file goes: a `backlog.md` §2 entry or a `roadmap-v2.md`
> row, the owner's call.
>
> **Read first, in this order:** `cant-effects-architecture.md` §7.1 (the
> Track R/S reading; its rows 9, 11, 12, 14 are RS-2, RS-4, RS-3a, RS-3b,
> all open and all unblocked since item 7 landed on 2026-09-06);
> `copy-effects-architecture.md` §7.1 and its phase table (CV-1b and
> CV-2–CV-7 open; CV-7 back-stopped before Phase 8); `layers-architecture.md`
> §13c's list of open layers items; `backlog.md` §2.24 ("as though", 287
> cards, sized as a `Permission` mirror of `Restriction`, no doc), §2.9 (the
> information model, before Phase 8's reveal cards and before Phase 10),
> §2.20 (several target clauses, back-stopped before RS-2 and before A6's
> first PR), §2.22 (the middleware census, now with pass 3's ask table);
> `roadmap-v2.md` §3a rows A4b (the rulings ledger, "1 + a sitting", the
> owner's between-phases slot), A4c (the trace sink, 1 PR, ordering-free)
> and A6 (the triggers doc, unsized — "write its doc first"); pass 1's
> outputs (§3's pass 1 block; `codebase-state.md`'s "Found by the post-RE
> audit (2026-09-15)") and pass 3's (items 138–143; `backlog.md` §2.22's
> table); `engineering-practices.md` §8, for the shape a numbered section
> of that file takes.
>
> **Verified against the tree, 2026-09-15:** no triggers architecture doc
> exists. `atomic-tests/supplemental-docs/state-tracking-architecture.md`
> numbers **five** problems, not the four §3's pass 4 block says — CR
> 603.8's mid-resolution state triggers, 603.1b's multi-condition triggers,
> cross-turn lookback, resolution counting, and CR 731's loop detection,
> the last owned by `backlog.md` §2.28 — and CR 603.10a's visibility seam
> and CR 121.2c's ordering (main item 122) are A6's two named seams.
> `plans/handoffs/` holds this file alone. Pass 3 left three engine items
> scheduled against the harness rather than the spine — item 139 (~5 lines,
> its own PR with an A/B, carrying item 41's test), item 140 (~80 lines,
> after 139), item 138's two counters (~80 lines, no A/B) — and none gates
> the triggers doc.
>
> **The checklist:**
> 1. **The table** — one row per open track phase and lattice entry: RS-2,
>    RS-3a, RS-3b, RS-4; CV-1b, CV-2–CV-7; A4b, A4c; §2.24, §2.9, §2.20,
>    §2.22's census; pass 3's items 138–140. Columns: what it unlocks in
>    cards (cite `cards-unlocked-ledger.md` and each doc's consumer lists;
>    re-derive no Scryfall count) and in atoms (`specdb.py`), what it needs,
>    and which of the triggers doc's questions — the five problems above,
>    the two seams — needs it first.
> 2. **A proposed order for the between-phases slot** — the owner decides;
>    this pass proposes. A4b already holds the slot; say what sits beside
>    it and what waits for the doc.
> 3. **The audit as a recurring practice** — `engineering-practices.md` §9,
>    with this run as the first instance: the cadence (each spine-phase
>    close, §6 decision 5); the passes (close-out, eviction, hygiene and CI,
>    readiness, scheduling); and each pass's instrument by name — the board
>    and `specdb owed --phase`; the ≤40-line stub rule and the archive;
>    §2.1's grep tiers and the clippy count; the census provider, the clone
>    timer, the fork record-and-replay, `plans/panic_surface.py` and the
>    three-run contention read; the table this pass produces.
> 4. **The record** — the `### … — audited 2026-09-15` heading in
>    `codebase-state.md`, pointers not prose; then delete this file.
>
> **Binding rules:** docs only; no count re-derived that a doc already
> carries with a date; American spelling; a number in a heading is a number
> that will be wrong.
>
> **Exit:** one docs PR; the four checks each on their own exit code;
> `check_state_of_play.py --write` (deleting the handoff moves the board's
> half-finished list) then `--check`; merge left to the owner.
