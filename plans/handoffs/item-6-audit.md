# Handoff — the item 6 audit

**Opened 2026-10-22**, the day PR #269 (TR-7) merged and the last build
phase of critical-path item 6, triggered abilities, landed with it. This is
the audit `engineering-practices.md` §9 runs at a spine item's close, the
second after the post-RE audit (`git show 341ebf9:plans/handoffs/post-re-audit.md`).
It covers `roadmap-v2.md` rows A6e (item 6's close audit) and A6h (the docs
audit), which sit together on item 6's line before A6f. This file is the
plan and the contract: the board lists it under half-finished work until
the last pass lands, each pass's PR updates §9's status block, and **the PR
that lands the last pass deletes it**.

An audit reads, measures and gives each finding a disposition. It ships
docs and small fixes; anything larger is scheduled work (§6, decision 3).
Every count below was read from the tree at `45d4a130` (#269's merge) on
2026-10-22 by read-only scouts, then checked by a second set of agents told
to refute each number; the ones that did not reproduce were corrected. A
pass re-reads what it touches: a number here is a starting point.

## 0. Where things stand

- Branch `audit/item-6-plan`, off `main` at `45d4a130`. This file and the
  regenerated board, nothing else.
- `plans/handoffs/` was empty before this file.

## 1. Why this audit is bigger than the last

**The track is not bigger; the window around it is.** Elapsed time is from
the first PR's opening to the last one's merge.

| Track | PRs | Lines added | Days |
|---|---:|---:|---:|
| `replacement/`, to item 5's close (#58–#140) | 28 | +70,529 | 21 |
| `triggers/` (#171–#269) | 34 | +71,251 | 34 |

The window #171–#269 holds **96** merged PRs, **+164,056** lines. **62** are
not on `triggers/` branches, and eight of the 34 that are, are not build
phases: the survey and the design doc (#171, #172), the survey's design gaps
and the cast-facts capture (#180, #181), and four A6 steps (#241, the LKI
rename; #242, RH's design; #246, A6p's records contract; #248, A6r's fixes).

| Class | PRs | Which |
|---|---:|---|
| Prerequisites the trigger plan required | 11 | #188 (bounded state), #195, #197, #199, #230 (CV-2a, RG, CV-2b, CV-1b), #243–#245 (RH), #259 (MA-2), #266, #267 (B10, CL-1) |
| Fixes to older subsystems that trigger work exposed | 6 | #185 replacement, #190 and #194 state and clone, #191 layers (LL), #198 turns, #265 the trigger placement's own cost |
| The mana track's first phase | 2 | #224, #225 (MA-1's design and build, found by A4k before the window) |
| Tooling, the dev GUI, process | 28 | A6g's dev GUI, SU-1–SU-8, A6j, #187; #205 is an engine fix the dev GUI found |
| Audits, measurements and docs | 15 | A6k (#235–#237), A6l (#240), A6m (#249, #250), A6q (#247), #189 (item 181's measurement), the type-surface re-sweep, the censuses, the route docs |

**The debt grew with the window.** `codebase-state.md` gained **49** dated
"Found by" headings (52 counting two "Found at" and one "Found after") and
**130** numbered items: 78 closed, **52 open** — 37 unreachable, 10
reachable but not wrong, 2 nothing owed (185 and 188, A6h's), 3 reachable
and wrong today. The board's totals: 347 items, 150 open, **4 wrong today**
(59, 204, 207, 211), 0 unstated, 0 homeless.

**The contracts other docs state moved.** Five detours changed a rule a
subsystem doc states, and each landed in pieces:

1. **RH (#242–#245), replacement.** The zone-change record:
   `replacement-architecture.md` §2b, about 800 lines over four PRs, and
   `CLAUDE.md`'s chokepoint bullets (every `ZoneChange` names its act and
   every proposal its cause; a batch's members are decided against one
   board). #185 and #248 changed `substitute` and redirection beside it.
2. **CV-1b (#230), copy, permission, triggers.** A move ends every reference
   made before it (CR 400.7).
3. **RG (#197), replacement and copy.** CR 614.1c's "enters with" and
   "enters as" are one shape, which `Rewrite::EnterAsCopy` builds on.
4. **LL (#191) and A6m (#249, #250), layers.** Hidden zones walked only when
   read; the `ObjectFilter` leaf tables merged; CR 611.2c's affected set.
5. **MA-1 (#225) and MA-2 (#259), mana and cost.** The exact check, and an
   {X} spell offered at its least legal X; `cost-architecture.md`'s CP-1 and
   CP-2 assumptions predate both.

And a sixth outside those docs: the state and clone storage (#188's event
window, #190, #194, #267), against `CLAUDE.md`'s determinism and chokepoint
sections and `clone-architecture.md`.

**The calibration.** The post-RE audit ran **13 PRs** (#141–#154) in about
21.5 hours: readiness filed the most items (6), hygiene took the most PRs
(4) and cut the most lines. A6k, the midpoint audit (#235–#237), ran 3 PRs
in about 22 hours. The model pass came at A6l (#240) and found **11** copies
the script did not list, 3 of them already disagreeing, for about 355k
tokens over 14k lines. This audit predicts **13–20 PRs over several days**,
the sizes in §4, against the 1 PR each that A6e's and A6h's rows carry
(§6, decision 4).

## 2. How the work is split

**Discovery fans out; fixing stays sequential.** Three passes (1, 3 and 5)
open with a read-only discovery workflow, multi-agent, the owner's opt-in at
the plan's review: breadth-first reading where one context misses things,
and where a claim needs an independent check before it becomes work. Pass
2's citation read is a script, with agents only for the doubtful lines.
The workflows' rules:

- **Read-only.** No agent edits the tree, switches a branch or runs a build;
  the working copy is shared with the session that fixes.
- **Find, then refute.** Each finding goes to independent verifiers
  prompted to refute it, with distinct lenses where it can fail more than
  one way (is the fact really computed twice; do the copies already
  disagree; does a test pin either). Only survivors become work.
- **A budget and a stop rule per agent**, and every dropped or unread area
  logged, so no report reads as "covered everything" when it did not.
- **The output is a list, not a diff**: each survivor with its pointers, its
  proposed disposition (fix here, merge in a cleanup PR, file an item, keep
  with a reason) and its size. The list goes into this file's pass section.

**Then each fix is its own commit on a sequential branch** in the main
folder, shown to fail first where it is a bug, one PR at a time through the
owner's review. Readiness and scheduling are measurement and judgement and
take no workflow. Expected cost: 1–2M tokens per discovery workflow,
verification included.

## 3. Where findings land

As §9 and the post-RE audit set it:

- **A deferred finding** is a numbered item in `codebase-state.md`'s
  Deferred Migrations under `### Found by the item 6 audit, <pass>
  (2026-…)`, with its `**Reachability (date):**`, `**Sized:**` and
  `**Slotted:**` lines (a slot that names a route row, never a bare owner).
- **A mechanic with no surface** is a `backlog.md` §2 entry.
- **A stale claim** is rewritten where it stands, never appended to.
- **A rule** goes to `engineering-practices.md`.
- **The run's record** is a `### Was critical-path item 6 done, and what
  sits before A6f? — audited YYYY-MM-DD` heading in `codebase-state.md`,
  pointers not prose, written when this file goes; and `engineering-practices.md`
  §9 gains the second instance's per-pass record and what the next run adds.

## 4. The passes

Order: the close-out gates the rest; the eviction opens second; hygiene and
the docs audit commute; A6e's cleanup follows hygiene's model pass;
readiness reads the tree the cleanup leaves; scheduling is last.

### Pass 1 — the close-out

**Goal:** item 6's done-checklist read off the tree and every entry given a
disposition; the summaries made true. Two or three PRs.

**Instruments:** the board first (`check_state_of_play.py --write`, then
`--check`; its two bolded rows are the work, and both should read zero
unstated and every wrong-today item scheduled with an owner);
`specdb.py build`, then `owed --phase "Phase 7"` (the stored database read
clean on 2026-10-22, 0 uncovered `NEW`, but it predates two commits that
added coverage, so the gate is the rebuilt one).

**Discovery workflow: the open items, re-derived.** One agent per batch of
the 52 window items still open and the 4 wrong-today items, re-deriving
each verdict against `main` (does a registered card or a test board now
reach it; has its owner landed; is its size still true), each wrong-today
item reproduced, a refuter for each changed verdict. Item 233, the midpoint
audit's prompts that hold decisions off `GameState`, is among them. Plus
three readers: `triggers-architecture.md` §13's 37 deferred atoms against
their owners (the copy group's CV-2a and CV-2b have landed); §15's 15
findings and §16's 12 bullets against what landed (at least §15's items 2,
4, 6, 7, 11, 14 and 15, and §16's loop and event-log bullets); and item 96,
A6e's atom pair (promote `BOUNDARY-DEF-615.1a-001`).

**Produces:** every changed verdict rewritten in place with a dated line;
A6e's row carrying all 15 of its items, each routed to the pass its
`**Slotted:**` line names (below); §13, §15 and §16 dispositioned; the
trace-page decision on each TR heading (§9's addition, §7); the TL;DR and
the CR 603 row of `codebase-state.md` rewritten, not appended; and the
retrospective, "The track in hindsight", pointers not prose, under 150
lines, written once and not maintained (§6, decision 5 places it) —
including the window's lesson, that the track's own size matched
replacement's and the detours did not. Fixes are their own commits, shown
to fail first; one that moves a pool is its own PR with a `fuzz-record.md`
block.

### Pass 2 — the eviction

**Goal:** `triggers-architecture.md` to its live contract. One PR.

The doc is **4,577 lines** (it grew at every merge: 1,672 at #172) and its
archive 5,618. The 21 phase stubs already meet the ≤40-line rule. What
moves to `plans/archive/triggers-architecture-landed.md`, planned section by
section with line counts before anything moves:

| Section | Lines | Why |
|---|---:|---|
| §0, §1 | ~110 | pre-build reasoning |
| §2.1 | 20 | a stale snapshot: it counts 34 `GameEvent` and 23 `GameAction` variants, against 41 and 26 |
| §8 | 32 | its "Today/Change" table is built and its line pointers drifted |
| §11.1's reading | 47 | a measurement, kept in `fuzz-record.md` |
| §12's two re-counts and its TR-4/TR-5 roll-ups | ~160 | sizing records |
| §14 | 69 | the survey's sixteen questions, answered |
| §17 | 25 | the documents owed, delivered |
| §4.9's design input for B14 | ~260 | the archive, under a pointer from B14's row; §4.9 keeps the draw's ~70 |
| §15's findings that pass 1 closes | — | each leaving its `N. **title**` line |

About **720 lines** by whole sections. Beyond them, **22 amendment
paragraphs** (paragraphs that open `**Amended`, 359 lines) sit beside the
text they overrule; the pass collapses each into the text as it now stands
and measures what that saves. §3–§6 are 2,860 lines of contract and stay.

**Its cost is citations:** **181** of the doc's sections in tracked
`plans/` files (107 outside the archive) and **91** in `src/` and `tests/`;
§12 alone **85**. Citations are written with a backtick before the `§`. A
script lists every one against the heading it lands on, and an agent reads
only those whose target moved. Headings stay byte-identical, every moved
section leaves its heading and a pointer, and `check_glossary.py` runs on
the result.

### Pass 3 — hygiene and CI

**The comment sweep and CI**, one or two PRs: §2.1's two grep tiers and the
history-word block list over every file the window touched (non-card `src/`
comment lines went from 21,800 to 24,220 since A6k; the history-word count
is flat at 100); `TODO`s re-owned; `check_glossary.py --suggest` triaged;
clippy counted, then decided against `check_clippy_allows.py`'s table;
the toolchain floor (`rust-version = "1.88"`) re-measured or its reason
recorded; and `ci.yml`'s steps read for a comment giving each its reason
(it went from 9 steps at the post-RE audit to 25, and about 9 have none).

**Discovery workflow: the model pass**, one fact computed twice. Finders
use `plans/references/duplicate-inventory.md` §2a's prompt and are shown
neither `similar_functions.py`'s report nor the inventory's earlier lists;
reconciling against those (the five per-PR runs, §3.6–§3.10, and §3.2's
facts already assigned to A6e) is a dedup step after finding. The trigger
code reads as five areas of comparable size, each whole:

| Area | Files | Lines |
|---|---|---:|
| A. Dispatch front | `engine/triggers/dispatch.rs` 1–1276 (types, entry and queue, LKI capture), `control.rs`, `placement.rs`, `state.rs`, `mod.rs` | ~1,950 |
| B. Dispatch back | `dispatch.rs` 1277–2792 (candidates, matching, the arm predicates, event objects), `audit.rs` | ~1,840 |
| C. Vocabulary | `types/triggers.rs`, `cards/authoring/triggers.rs` | 2,794 |
| D. Binding and delayed | `binding.rs`, `binding_reads.rs`, `delayed.rs`, `engine/returns.rs`, `engine/zone_function.rs` | 2,430 |
| E. History | `engine/triggers/history.rs`, `state/history.rs`, `state/history_log.rs`, `types/history.rs`, `state/shared_stores.rs` | 2,054 |

Plus the window's hunks in the shared files the track grew, lines added
over the window's commits: `engine/resolve.rs` (+2,426),
`ui/display.rs` (+1,615), `engine/actions.rs` (+1,459), `types/effects.rs`
(+1,284), `state/game_state.rs` (+1,266), `events/event.rs` (+1,148),
`engine/layers/board.rs` (+1,110), `replacement/pipeline.rs` (+1,090),
`layers/condition.rs` (+914), `state/layer_memo.rs` (+614), `ui/why.rs`
(new). Not all of that is the trigger track's; the readers take the hunks
whose commits name a trigger phase.

Three cross-area readers take what no area holds whole:

1. **`TriggerEvent`'s eleven tables.** Its 30 variants are matched
   exhaustively by 11 functions: eight in `types/triggers.rs` (`looks_back`,
   `record_kinds`, `subject`, `supports_once_per_subject`, `multiplicity`,
   `event_roles`, `player_of`, `amount_of`), two in `dispatch.rs`
   (`triggering_events_in`, `triggering_event_objects`) and one in
   `authoring/triggers.rs` (`once_per_event`). A new event costs eleven
   arms. The test at `types/triggers.rs:2144` checks `record_kinds` against
   `GameEvent`'s `from_record` for 22 of the 30 arms, which links this
   reader to the next.
2. **"Which objects this event names".** `GameEvent`'s 41 variants are
   matched by `from_record` (`types/triggers.rs:603`),
   `for_each_object_named` (`events/event.rs:572`) and `format_event`
   (`ui/display.rs:924`), and in part by `single_object` (`dispatch.rs:2622`)
   and `draft_history_rows` (`engine/triggers/history.rs:207`).
3. **The LKI capture cluster**, `dispatch.rs` 1025–1289: fourteen small
   functions whose names overlap (`capture_lki`, `capture_frame`,
   `capture_of`, `lki_now`, `last_known_information`, `take_named_lki`,
   `take_captured_lki`, `hand_over_lki` among them).

And `triggering_events_in` (`dispatch.rs:1909`), 384 lines, the subsystem's
longest function, read for what it decides more than once. The survivors
join `similar_functions.py`'s whole report as this pass's list of record.

**A6e's cleanup**, after the model pass, because its merges may reshape the
same types. Sizing it is its first act, with every item in: 291 (three
things written as several types: ~920 references and ~250 constructor
sites, split along its three seams if it passes the band), 285 (about 80
tuple returns), 292 (~60–80 lines), 296 (~100 lines), 247 (the trigger tail
applied, queued and folded, ~200–350), 137 (`ResolutionContext`'s rider
fields, a PR of its own by its line), Before Layers 7d (the
`ContinuousEffectDraft` refactor, ~100), 215 (MA-1's three cleanups), 239
(the crate-wide `.clone()` audit, with a fuzz A/B), 284 (~80 lines of
Python and a CI step), 269 (seven per-object `AmountExpr` variants, with the
naming sweep), `AbilityDef.instances`, the naming sweep itself, and the
model pass's survivors, folded into band-sized PRs by theme
(`engineering-practices.md` §4). Each that moves a pool carries its own
A/B; each that changes a hot path, its own callgrind arm.

### Pass 4 — readiness

**Goal:** is the engine on track for the harness use case? Two PRs.
Measurement, no workflow, on the tree A6e's cleanup leaves.

- **The ratchet**, its first duty: decisions per loaded core-second on both
  boards at four seats and one thread, recorded as a dated reading beside
  item 138's first. It may not read worse per decision without a written
  reason in the record.
- **The census provider**, item 138's counters.
- **§3.1's three floors**, with **the clone timer** and its counting
  allocator: item 143's table extended with this close's row, allocations
  and bytes beside time, and time by the median of five means of 2,000.
- **The fork record-and-replay**, item 41's test, over the window's new
  state: the histories, the delayed registry, the returns, the shared
  stores.
- **The three-run contention read**, `--threads 1`, 8 and 16 at 200 games
  (item 138's lever 7), beside the post-RE reading.
- **Callgrind** instructions per decision, the lever list re-ranked by
  instruction share (TR-7's engine arm read 0.6695 M on the Commander
  board).
- **`panic_surface.py`**, its two runs reconciled: a scout read the engine
  at 79,898 lines against 44,342 on 2026-09-15, and 76 release-active sites
  against 60, without checking that both runs counted alike. Item 142 holds
  the baseline.
- **k**, an AI observation's cost per decision, with
  `plans/references/ai-performance-floors.md`'s throwaway probe, before A6f.

Items 138, 142 and 143 are this pass's. Expect the most new items again,
each sized.

### Pass 5 — the docs audit, A6h

**Goal:** each fact stated once where it lives, superseded text corrected
where it stands (A6h's row), with items 185 (the inventory of elisions)
and 188 (two roads to one state). Two to four PRs.

**Discovery workflow: the docs against the tree.** One reader per
architecture doc for its stated contract — types, invariants, decisions a
later phase reads — checked against the code; one reader per contract change
in §1's six, reading every doc that states it; one for `CLAUDE.md`'s
invariants against the same six. A refuter per claim called stale. Inputs:
PR #200's "For A6h" list (8 disagreements) and every later PR body that
carries one (`git log` mentions A6h 7 times in the window; a `gh` search
did not find them, so the pass reads the bodies). The docs that grew past
300 lines in the window: `codebase-state.md` +1,855,
`replacement-architecture.md` +1,034, `engineering-practices.md` +874,
`backlog.md` +765, `cr-coverage-audit.md` +482, `copy-effects-architecture.md`
+409; and the new ones, `setup-architecture.md` (2,765),
`mana-architecture.md` (1,009), `clone-architecture.md` (693),
`engine-map.md` (334), `devgui-map.md` (310) and
`permission-architecture.md` (234). `fuzz-record.md` (+4,577) is a record
and is not audited. `roadmap-v2.md`'s rows are re-checked against the
2026-10-02 cut.

### Pass 6 — scheduling

**Goal:** one table, §9's shape: each open track phase and lattice entry,
what it unlocks in cards (the ledger and the docs' consumer lists, no
Scryfall count re-derived) and in atoms (`spec.sqlite` by rule prefix, the
prefixes named), what it needs, and which of A6f's and A6s's questions needs
it first; then a proposed order for what follows. §3b re-derived; A4d's
open artifact, the Rust notes, placed. The pass proposes; the owner
decides. One PR, which writes §3's two records and deletes this file.

## 5. A6e's items, by pass

| Item | What | Pass |
|---|---|---|
| 96 | promote `BOUNDARY-DEF-615.1a-001` | 1 |
| 138, 142, 143 | the throughput target, the panic-surface baseline, the clone table | 4 |
| 137, 7d, 215, 239, 247, 269, 284, 285, 291, 292, 296 | the cleanup | 3 |

A6h's are 185 and 188 (pass 5). A6s's, 264 and 288, stay with A6s.

## 6. Decisions for the owner

1. **The docs audit inside this audit.** A6h is its own row; the plan runs
   it as pass 5 under one handoff, since both rows close item 6 and the
   contract changes are the same list. Recommended.
2. **`TriggerEvent`'s eleven tables.** If the model pass confirms one fact
   in several tables, folding them is a design (the arm's growth contract),
   not a cleanup: filed as an item with a slot, unless it sizes as a small
   fix.
3. **The cleanup is scheduled work riding this handoff, not audit fixes.**
   §9 limits an audit to docs and small fixes. A6e's row and its items'
   `**Slotted:**` lines are the schedule, so pass 3's cleanup PRs are that
   scheduled work, sized to the band, and the "small fix" bar holds for
   everything else a pass finds. Recommended: the alternative, cleanup after
   scheduling, would schedule against a tree the cleanup then changes.
4. **The rows' sizes.** A6e's row says 1 PR and A6h's 1; this plan amends
   them to its estimates (§1) when its review settles.
5. **The retrospective's home.** `triggers-architecture.md` §18, "The track
   in hindsight", in `replacement-architecture.md` §14's shape. Recommended:
   it survives the eviction, being written after it is planned.

## 7. What the scouting did not cover

The clippy count; `similar_functions.py`'s current report; the TL;DR and
the CR row's text; the bodies of the 34 trigger PRs for older-subsystem
fixes made inside a phase (TR-5d's CR 615.13, TR-4d's item 131);
`backlog.md`'s and `engineering-practices.md`'s growth by subsystem; §3's
subsections paragraph by paragraph; whether `panic_surface.py`'s two runs
counted alike. Each pass reads what it needs of these.

## 8. Briefs

Each pass's session starts from its section above and this file's §2 and
§3. A brief adds only the branch name and what the previous pass's PR
changed.

## 9. Status

| Pass | PR | State |
|---|---|---|
| Plan | — | this file |
| 1. Close-out (2–3 PRs) | | |
| 2. Eviction (1) | | |
| 3. Hygiene and CI (1–2), then A6e's cleanup (3–6) | | |
| 4. Readiness (2) | | |
| 5. Docs audit, A6h (2–4) | | |
| 6. Scheduling (1) | | |
