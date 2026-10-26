# Handoff — the item 6 audit

**Opened 2026-10-22**, the day PR #269 (TR-7) merged and the last build
phase of critical-path item 6, triggered abilities, landed with it. This is
the audit `engineering-practices.md#the-spine-phase-audit` runs at a spine item's close, the
second after the post-RE audit (`git show 341ebf9:plans/handoffs/post-re-audit.md`).
It covers `roadmap-v2.md` rows A6e (item 6's close audit) and A6h (the docs
audit), which sit together on item 6's line before `information-model-design`. This file is the
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

- The plan merged as #270 (`c91657e1`). Discovery, session 1, ran on
  `audit/item-6-discovery` off that merge and wrote the work-lists under
  passes 1, 2, 3 and 5. It touched this file and the regenerated board, and
  nothing else.
- `plans/handoffs/` was empty before this file.
- Pass 1, the close-out, ran in session 2 as one PR, #272, on
  `audit/item-6-close-out` (§9). Its work-list below is done; what each entry
  became is in the PR's body and in the places §3 names. The passes after it
  start from their own work-lists.
- Pass 2, the eviction, ran in session 3 as one PR, #273, on
  `audit/item-6-eviction` (§9); its body has the line counts section by
  section. Every moved heading stayed byte-identical except the TR-4 and
  TR-5 roll-ups, which gained their last sub-phase's ✅ date so the board
  stops listing them as not yet landed (the owner, at #273's review).
- Pass 3's hygiene half ran in session 4 as one PR, #274, on
  `audit/item-6-hygiene` (§9). The model pass's rows to file are items
  301–315 in `codebase-state.md`, and five more joined items 213, 265, 291 and
  292, so A6e's cleanup, session 6, sizes against numbered items. Its row
  pointers in the work-list below are still `45d4a130`'s lines; the items
  cite functions.
- Pass 5's first PR, the contracts, ran in session 5 as #275 on
  `audit/item-6-docs-contracts` (§9). Done: `CLAUDE.md`'s 6 rows and
  `engine-map.md`'s 12; every row the six contract-change readers found in
  another doc, source comments included; the 15 contested rows among those,
  each fixed or given a reason in #275's body; `engine-map.md` §1.2 read
  against a probe of its board, and the page re-pinned to `22113bbb`. The
  doc-reader rows that share a section with a contract row were left for
  PRs 2 and 3 (replacement §2a's `EventPattern`, `ReplacementClass` and
  `Uses` rows, copy §2.1's Tier C). Their work-list rows quote the text as
  it was before #275, so re-read the section before fixing.
- Pass 5's second PR, the older subsystem docs, ran in session 5 as #276
  on `audit/item-6-docs-older` (§9). Done: `layers-`, `replacement-`,
  `cant-effects-`, `copy-effects-` and `cost-architecture.md` — their 69
  remaining work-list rows and the two halves #275 left (replacement §3.1's
  table, cost §3.2's CM-0 paragraph), all fixed; the 11 contested rows, each
  dispositioned in #276's body; and every unread section listed under "What
  the readers did not reach" for these five, read. Docs only. Left for PR 3:
  the cost gather's zone leg, which has no `codebase-state.md` item (A5's
  predicate landed at LK; the replacement leg is RF's and the restriction
  leg item 146's); three stale source doc comments
  (`ContinuousEffectRegistry::effects_in_layer`'s "Not yet CR 613.8",
  `CopiableValues`' and `any_control_changing`'s fast-path pointers); and
  `clone-architecture.md`'s "no code" status line, the last of the
  convention #275 left to a sweep (replacement, cant-effects and copy now
  name their landed phases).
- Pass 5's third PR, the newer subsystem docs and the registers, ran in
  session 5 as #277 on `audit/item-6-docs-registers` (§9). Done:
  `triggers-`, `setup-`, `permission-`, `mana-` and
  `clone-architecture.md`'s remaining work-list rows; the registers'
  (`codebase-state.md`, `backlog.md`, `roadmap-v2.md`,
  `engineering-practices.md`, `cr-coverage-audit.md`; the glossary's and
  the census's confirmed fixed by #275); the source doc comments, the three
  #276 left among them; the contested rows, each dispositioned in #277's
  body; triggers §4.9–§6, §9 and §10 and mana §2, §3.3–§3.6 and §3.9–§9,
  read; pass 3's dedup claims, annotated where they stand; item 265 on
  `permission-design`'s row; and item 316, the cost gather's zone leg, filed and slotted
  at `game-formats`. Found beside the rows: item 72 had moved to `ironworks-loop` while five
  `backlog.md` lines and the mana and cost docs still placed it with item
  6. Left for PR 4: item 185 and the two trace pages.
- Pass 5's fourth PR, item 185, ran in session 5 as #278 on
  `audit/item-6-elisions` (§9). Done: the inventory, `plans/elisions.md`
  (forced prompts, elisions and skipped work, each with its claim, premise
  and what checks it; the forced prompts' "CR 102.2" corrected throughout);
  every forced prompt decided in `ui/ask.rs`, and each forced prompt and
  elision taken recorded in the game's log (`events::unasked`), drawn by
  `format_event_log` and not by the dev GUI's `log_lines`; audits for the
  replacement, restriction and cost gates, the control sweep and the
  castability fold under `--audit` (`engine::audit`), and a debug check of
  the trigger elisions' premise. Item 185 closed, item 317 filed. Decided
  at its design review (the owner): the record a note on the event
  recorder; every forced prompt and elision recorded but the priority pass;
  the inventory a page of its own, the engine map staying an entry point;
  forced prompts decided in `ui/ask.rs`; the checks built here, and the two
  trace pages split to PR 5.
- Pass 5's fifth PR, the two trace pages, ran in session 5 as #279 on
  `audit/item-6-trace-pages` (§9). Done:
  `plans/traces/tr-4d-control-is-announced-by-a-sweep.html` and
  `tr-5h-every-history-read-is-a-scan-of-one-log.html`, each from traced
  runs at `50f0b8b8` with every fix it names taken out once in a scratchpad
  copy of the crate; linked from `triggers-architecture.md` §12's two
  headings, `codebase-state.md`'s "Found after TR-4d" and "Found by TR-5h"
  entries and `trace-pages.md`'s table, and §18's "owed" bullet
  gone. TR-5h's damage-fact and gate boards are its own fixtures, since no
  registered card reads a damage fact or the log from a layer, and its page
  shows a record's place in its turn reading #265's tally. **Pass 5 is
  done.**
- The card-homed triage, session 5b, ran as one PR, #280, on
  `audit/item-6-card-homed-triage` (§9). All 51 items have a disposition in
  their `**Slotted:**` lines and a verdict re-dated 2026-10-23: 31 built
  before breadth in A6e's cleanup, 14 re-slotted to rows, 5 kept with their
  cards, and item 109 closed, found fixed. The triage section below has the
  builds by theme and the pool question priced.
- Pass 5c, the names for the phases ahead, ran in session 5c as one PR,
  #282, on `audit/item-6-phase-names` (§9). Done: every phase and route row
  still to come is cited by its name, hyphenated in backticks
  (`engineering-practices.md#naming-a-phase`), in the docs and in source, test and dev
  GUI comments; the retired codes are `plans/archive/route-codes-renamed.md`;
  the board reads the names, and every open item's home is the one it had
  (149 route-homed, 5 card-homed, 12 records, 0 homeless). Decided at its
  review (the owner): the hexproof phase split in two, `game-formats` widened
  to the 60-card constructed formats, what is left of Phase 9 put to
  `route-review`, and no check for a retired code.
- A6e's cleanup, session 6, opened on `audit/item-6-cleanup-plan` off
  #282's merge. Its plan is pass 3's "The cleanup's plan": 60 items and 40
  merge rows in eight themed PRs, each item's **Slotted:** line naming its
  PR; it merged as #283, with the four decisions as recommended. The first
  PR, `history-and-frame-records`, ran as #284 on
  `audit/history-and-frame-records` off #283's merge (§9). Done: items 286,
  297, 303, 305, 306, 310, 311, 312, 313 and 314 and "Before Triggered
  abilities" 6 closed, items 278 and 279 built and narrowed, the 12 merge
  rows, item 247's T17 log half, and `--pool fixtures` in the checked
  sitting. Items 313 and 314's readings were not judgment calls: the CR's
  cost order and the printed cards' rulings decide them. Left to another
  PR: CR 509.4's blocking entry, item 278's remainder, which needs the block
  arm to say which side it watches, re-slotted to
  `layers-combat-and-counters`. Added at its review (the owner,
  2026-10-24): `last-known-information-store`, item 318, the PR after it.
  It ran as #285 on `audit/last-known-information-store`: the design note
  first, its four decisions taken at its review (`triggers-architecture.md`
  §3.11), then the build on the same branch. Item 318 closed. The next PR,
  `practices-rules-page` (1c), ran as #286 on `audit/practices-rules-page`:
  the design note first, its four decisions taken at its review, then the
  build on the same branch. Done: `engineering-practices.md` is a page of
  rules, each a sentence or two and a pointer, cited by heading anchor and
  never by number; the doc as it stood is
  `plans/archive/engineering-practices-records.md`, under a table mapping
  each old number to its anchor; the trace-page format, the spine audit's
  passes and the egui checklist have docs of their own; the floor readings
  are `fuzz-record.md`'s; every citation was rewritten, and
  `check_practices_page.py` fails a number or an anchor that names no
  heading. The next PR, `concept-census-and-register` (1d), ran as #287 on
  `audit/concept-census-and-register`: the design note first, its six
  decisions taken at its review, then the build on the same branch. Done:
  the census of the 758 fields the game holds, read from the tree with
  `plans/references/concept-census.py`; the register,
  `cr-coverage-audit.md#4c-concepts-and-their-owners`, 21 concepts each with
  its key, owner and holders; `check_type_surface.py`'s second half, which
  fails a holder the register does not list; the
  `ObjectInformation::Characteristics` arm naming its moment (`FrameContext`);
  and nine findings filed as items 319–327 into five cleanup PRs, with item
  30's objects that paid re-slotted from a card to
  `resolution-recipients-and-amounts`. The next PR,
  `resolution-context-rider` (2), ran as #288 on
  `audit/resolution-context-rider`. Done: item 137 closed;
  `ResolutionContext`'s two rider numbers one
  `replaced_event: Option<ReplacedEvent>`, an enum whose damage arm alone
  has a prevented amount, which the pipeline's `Rider` carries too; and the 101 literals that wrote
  both `None`s swept, 37 of them to `ResolutionContext::untargeted`. The
  next PR, `trigger-matcher-merges` (3), ran as #289 on
  `audit/trigger-matcher-merges`, item 317 its last commit. Done: items 247,
  301, 302, 304, 309, 315, 317, 319, 320 and 321 closed, and the merge rows,
  each dispositioned in #289's body. Item 320 ran long and was split in the
  doc: a move's records and the three always-framed ones name their
  existence, and the other payloads' ids are item 328, since converting them
  moves the stream. Item 329, a rider's missing ability source, filed to
  `resolution-recipients-and-amounts`. At #289's review (2026-10-25) the
  owner split `one-fact-one-type` in two, since a session's length follows
  how many separate fixes it holds: item 328 is `record-existences` (3b),
  with its own `differ` A/B, and goes first, because item 291's
  object-or-player type is built over its result. The next PR,
  `record-existences` (3b), ran as #290 on `audit/record-existences`. Done:
  item 328 closed. Every record names each object by the existence the
  event was about, and the matcher, the history writer and a resolution's
  created tokens read it; the A/B moved neither pool, so its verdict stood.
  What a creature attacks stays the combat's `AttackTarget`, by id, the
  question the brief left to the session. The next PR,
  `one-fact-one-type` (4), ran as #291 on `audit/one-fact-one-type`. Done:
  items 291, 296, 326 and 327 closed and rows 46 and 64 merged. An object
  or a player is `ObjectOrPlayerId` by id and `ObjectOrPlayer` by
  reference, in `types::ids`; what an effect is affecting is one
  `AffectedSet`; `Selector` is an `ObjectSet`. A zone statement left
  `Condition` for `Effect::FunctionsIn` at every ability's root, which a
  trigger's binding carries for the dispatch and the resolution to ask (the
  owner, at the build and its review, superseding `layers-architecture.md`
  §13d decision 1b). The next PR is `durations-and-control` (5), then
  `batch-legality-and-entry-choices` (5b): row 5 split in two at #293's
  review, which takes the cleanup one PR past its planned four (the owner,
  2026-10-25).

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
**134** numbered items: 127 in the main run (167–296, with 218, 221 and 277
unused) and 7 in "Before Triggered abilities". 82 are closed, **52 open** —
37 unreachable, 10
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
   {X} spell offered at its least legal X; `cost-architecture.md`'s `hybrid-and-phyrexian-payment` and
   `remaining-costs` assumptions predate both.

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

**The sessions** (the owner's question at #270's review). Discovery is
read-only, so all three workflows run first, in one session, before any
fix: the passes then start from lists rather than each opening with a
fan-out, and nothing a fix session writes is read mid-discovery. The fix
sessions are ordinary ones, one PR at a time in the main folder:

| Session | Kind | Passes | PRs |
|---|---|---|---|
| 1. Discovery | ultracode: the three workflows, and pass 2's citation script | 1, 3 and 5's lists, written into this file | 1, docs |
| 2. Close-out | plain | 1 | 2–3 |
| 3. Eviction | plain; a script and a reader for doubtful citations | 2 | 1 |
| 4. Hygiene and CI | plain | 3's sweep | 1–2 |
| 5. The docs audit's fixes | plain, a session per PR | 5 | 4 (pass 5, re-cut at #274's merge) |
| 5b. The card-homed triage | plain; a single verifier at most | the triage (§4) | 1, docs |
| 5c. Names for the phases ahead | plain; the design reviewed first | pass 5c | 1 |
| 6. A6e's cleanup | plain, a session per themed PR | 3's cleanup, with the triage's builds | 8, by the plan in pass 3 (2026-10-23), and three more at #284's review: the store of last known information, the rules page and the concept census |
| 7. Readiness | plain; measurement | 4 | 2 |
| 8. Scheduling | plain | 6 | 1 |

A fix session may still spawn a single verifier for a doubtful finding; it
runs no fan-out of its own. Sessions 4 and 5 commute. A finding whose
pointers a later fix moves is re-read by the session that fixes it, which
re-reads what it touches anyway.

**Then each fix is its own commit on a sequential branch** in the main
folder, shown to fail first where it is a bug, one PR at a time through the
owner's review. Readiness and scheduling are measurement and judgment and
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
  sits before `information-model-design`? — audited YYYY-MM-DD` heading in `codebase-state.md`,
  pointers not prose, written when this file goes; and
  `spine-phase-audit.md` gains the second instance's per-pass record and what the next run adds.

## 4. The passes

Order: the close-out gates the rest; the eviction opens second; hygiene and
the docs audit commute; the card-homed triage follows hygiene's sweep; the
naming of the phases ahead follows the docs audit and the triage; A6e's
cleanup follows hygiene's model pass, the triage and the naming, and takes the
triage's builds; readiness reads the tree the cleanup leaves; scheduling is
last.

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

#### Work-list (discovery, 2026-10-22)

These lists come from Workflow A, `item6-discovery-open-items`. Eleven
finders ran, each with about 35 tool calls: eight item batches and three
readers. Then two independent refuters checked each changed verdict, one
re-running the finder's command and one finding its own evidence. A change
survives only if neither refuter refutes it. The batch for items 279–296
held 13 items, about 2.6 tool calls each, so it was re-run as two finders
at full budget. Their results are the ones below, and how the two runs
compare is noted.

`fuzz_games` below is `mtgsim/target/release/fuzz_games.exe`, which was
built after `main`'s last source commit. Its trace header reads
`02e4e5d8d680-dirty`, but `git diff 02e4e5d8 HEAD -- mtgsim/src
mtgsim/Cargo.toml` is empty. Each run used `--threads 4`, and the scripts
that count a dump sit in the discovery session's scratchpad.

**Counts.**

| | Entries | Changed | Survived | Refuted | Unverified |
|---|---:|---:|---:|---:|---:|
| Items (53) | 53 | 9 | 4 | 5 | 0 |
| §13's deferred atoms (37) | 37 | 12 | 3 | 9 | 0 |
| §15's findings (15) and §16's bullets (12) | 27 | 15 | 7 | 8 | 0 |
| Item 96's two atoms | 2 | 1 | 1 | 0 | 0 |
| **Total** | **119** | **37** | **15** | **22** | **0** |

Every finder logged what it dropped (93 notes). Some kinds of
check were dropped across the board:
- Scryfall counts in `**Sized:**` lines were not re-queried. They size the
  work but don't decide reach.
- Line estimates were not compiled, since cargo was barred.
- No test boards were searched for the four wrong-today items, because a
  fuzz run reached each of them.
- Debug-only loud paths (the layer memo audit, frameless rows) were not
  exercised.
- No item's per-site counts were re-read one site at a time.

Two finders went over budget: the wrong-today batch (40 calls) and the
§15–§16 reader (47).

**The four wrong-today items, reproduced.** All four are still wrong. None
changed class.

| Item | Was | Now, and the command | Disposition |
|---|---|---|---|
| 59 | 2026-09-03: 27 entries; 11 exiled a creature card first; 12 died to CR 704.5f; 6 both | 16 entries in 11 of 40 games; 8 exiled first; 15 died at once to CR 704.5f; 7 both, in 5 games. Sutured Ghoul is still 0/0 with no CDA (`phase_rc_cards.rs:741`). `fuzz_games --pool stress --require "Sutured Ghoul" --games 40 --seed 12345 --dump-events`, then a script that pairs each entry with the Ghoul's prior graveyard→exile moves and a `ZeroToughness` move within five events | Keep; re-date with today's counts. B7 is unlanded; the size holds |
| 204 | 2026-10-01: Cerulean Wisps leaves a tapped target tapped | 39 Wisps resolved with a target; 5 left it tapped with no `Untapped` (game 1: Merfolk Thaumaturgist #90, tapped at event 406, resolved at 441, untapped only at 490 in the untap step). `fuzz_games --pool performance --require "Cerulean Wisps" --games 40 --seed 2026 --dump-events`. Moonlace's half was checked from source only (`phase_lc_cards.rs:79-81`) | Keep; re-date with the count. C0 is unlanded |
| 207 | 2026-10-02: Thornweald Archer is no Elf, in the stress pool | Both pools now: Elvish Warmaster and the Archer are both in `PERFORMANCE_POOL` (`registry.rs:463`, `:91`; Warmaster since `70188c40`). In 60 stress games, 4 of 4 Archers entering under a Warmaster triggered nothing (2 triggers on real Elves); in 60 performance games, 3 of 3 (9 on Elves). All 14 creatures still have no subtype line. `fuzz_games --pool stress\|performance --require "Elvish Warmaster,Thornweald Archer" --games 60 --seed 2026 --dump-events` | Keep; re-date; "the stress pool" becomes both pools, and the **Sized:** line's re-record covers both pools' streams |
| 211 | 2026-10-05: Krark-Clan Ironworks is never offered at priority | 0 of 165 priority questions to its controller offered Ironworks; 226 of 226 mana windows did. `fuzz_games --pool stress --require "Krark-Clan Ironworks" --games 20 --seed 2026 --trace`. The item's exact case (a sacrifice in response to its removal) was not staged | Keep; re-date with the count. MA-6 is unlanded |

**Changed verdicts that survived both refuters.**

| Item | Was | Now | Disposition |
|---|---|---|---|
| 197 | unreachable: no registered effect puts a card onto the battlefield uncast; slotted to the first PR that builds `ReturnToBattlefield` | Unreachable for a new reason. TR-3b built the return (`7030a47c`, #234), and Flickerwisp, Banishing Light (`a8fc1da8`) and Strangleroot Geist (`5699bc9a`) now put cards onto the battlefield uncast. But each returns a permanent card, and the only CR 304.4/307.4 check is the scenario loader's (`scenario/build.rs:259-262`): none in `return_objects`, `entry_proposal` (`actions.rs:2193`), `move_object` or `perform_entry`. Wingrattle Scarecrow's persist is a fourth road | Rewrite the verdict. The slot's PR landed without the check, so either do a small fix here (~15–25 lines: the check in `entry_proposal`, beside its CR 800.4b check, where every road meets, plus a fixture that returns an instant) or re-slot it. Stale beside it: `phase_rc_cards.rs:434` ("`ReturnToBattlefield` is a stub"), `backlog.md:2961`, `:3089`, `setup-architecture.md:2746` |
| 217 | unreachable: every arm the wildcard in `amount_reads` covers reads nothing or is refused | Unreachable, but the reason is false. `c5d89ef8` (item 280's PR, #262) gave the layer `evaluate_amount` a `History` arm (`compute.rs:909`), and `amount_reads`' `_ => {}` (`layers/board.rs:810`) still swallows it. No registered static carries `AmountExpr::History` | Small fix here, ~12–15 lines: list the wildcard's arms, `History` reading `out.source \|= you_channel`. Drop the re-slot alternative, since item 280 is closed. Also update `codebase-state.md:7836` and `references/feedback-loops.md:160-163`, `:277` |
| 230 | unreachable; sized ~40 lines plus a leg whose dispatch result is dropped and whose provider is optional | Verdict holds: Banishing Light is the only registered "until", and `until_has_happened` still refuses any other event loudly. The leg is half stale: since `2a2e5e21`, `emit_record` returns `dispatch_unbatched`'s result (`trace.rs:888`). The optional provider holds (`emit_event` passes `None`, `trace.rs:848`) | Keep, re-dated; re-size the leg. Strike "whose result is dropped" and the "attackers declared" example, since attacks have a provider since `66b0796f` (`combat/steps.rs:89`). Only a cast lacks one (`put_on_stack.rs:314`) |
| 264 | reachable — not wrong: the CR's answer, intractable; reproduced by forcing Bridge from Below, 200 games at seed 12345 (game 12414) | Same class, but the reproduction is dead: the forced run now finishes with 0 turn-limit hits, 12414 included, because pool changes reshuffled every seed's decks. Primal Vigor, Verdant Force and the copy cards are still registered. A refuter's scenario still hangs: N Primal Vigors plus Verdant Force run 0.9 s at 10, 5.0 s at 11, 33.6 s at 12 and past 40 s at 15 | Rewrite the verdict with no known seed, and cite the scenario reproduction instead. Stays with A6s (§5) |

**Changes refuted.** In each case the old verdict stands, re-dated. The
refuters' corrections are listed so the close-out re-reads them instead of
re-deriving them.

| Item | Proposed | Why it was refuted |
|---|---|---|
| 239 | Re-size the `.clone()` audit to today's counts (387 in `src/`, from 192) and add `engine/triggers/` as a hot path | The counts reproduce (calibrated at `3a4c94d4`: exactly 192/121/35/17/9), but "a hot path" does not: 111.8 candidate visits and 6 placements a game, against 63,179 memo hits and 951 gathers. Re-size from the counts alone. The stale "before Phase 6's triggers" line goes too |
| 208 | Re-size the crate sweep to ~2,700 lines (2,732 `mtgsim::` uses in 128 files) and say the directory half holds | The crate half reproduces exactly (2,614 lines in `src`+`tests`, over the band). The directory half does not: the item's 20 scripts is `\bmtgsim\b` over `.py`/`.sh`, which gives 23 now. CI lines went from 2 to 4, trace pages from 7 to 9, and 17 Rust lines name `../mtgsim/`. `backlog.md:2588` ties the item to Phase 10's design |
| 215 | Re-size: the bare `OnceCell<ManaSupply>` "now threads five signatures, up from three" | Nothing grew. All five signatures were there at the commit that wrote the item (`2b0ad1e0`); the item counted only the twins. There are four twin pairs, not three (`can_activate`). And `x_values.rs:154` builds its own cell for `mana_payable`, whose debug audit never rewinds counters. The third cleanup's move to items 242/243/246 and C0 holds, so §5 and pass 3 should say "215's two cleanups" |
| 247 | Re-size, naming the sites the TR-4a and TR-5c widenings added; and rename T14's `for_each_naming_entry` to `for_each_lki_keeper` in the inventory | The new name is stale too (`bd402f37` renamed it `lki_keepers`, `dispatch.rs:1104`). The site list misses the rows `duplicate-inventory.md:560` (TR-5h) and `:609` (TR-5d) route to the item, and T15/T16/T21's sites. Re-size at A6e's sizing step from the inventory's whole list |
| 279 | Re-slot (drop the landed TR-5h half) and re-size, since TR-5h writes `LeftZone` and `Cast` rows, so each fact would be ~15 + ~30, not ~45 + ~70 | The re-size fails: "was cast" was ~15 + ~30 already, and its writer is `PermanentState::cast` (`battlefield.rs:103`), not TR-5h's `Cast` row, which keys the spell on the stack (`history.rs:229-233`) and never matches the permanent, since every arrival bumps the epoch (`zones.rs:257`). Both runs refuted the re-size. Both reproduced the rest: `ObjectFact` has seven variants (the heading says four), Relentless Assault is the only card that reads one, and `GatedWalkInput::Histories` has no references. Also, the Slotted label is 68 characters, past the board's 60-character stop, so `SLOTTED_RE` reads the whole paragraph as the slot |

**Unchanged, re-dated.** Each keeps its class, owner and size, and its
proof is the command its finder ran. Some carry a small wording fix.

| Item | Class | The proof | Small fix proposed |
|---|---|---|---|
| 170 | reachable — not wrong | `grep -n 'fn ask_order_triggers' -A10 ui/ask.rs`: still `ChoiceOption::Object(source)` per entry. Hellrider reaches it (`tests/phase_tr5e_integration_test.rs`, `hellrider_deals_each_attackers_damage_to_what_it_attacks`) | Name Hellrider's test as the registered board |
| 184 | unreachable | `grep` for the shape's eight named cards in `registry.rs`: none registered. The per-function scan of `FilteredPermanents` and `FilteredObjectsIn` found 208 cards with a one-line grep, which misses the 7 multi-line registrations (215 are registered) | Re-scan all 215 (the item says 175); name the shape's cards |
| 185 | nothing owed | `git log -S filter_is_mods_invariant`: the premise site the body names was removed by `7fd936e7` | 1–2 lines: name its successor, once pass 5 confirms which |
| 188 | nothing owed | `git log -S"fn can_be_a_blocker"` and `grep check_cost_resource(`: three candidates were closed elsewhere (items 242 and 243, and `a0cf62bb`) | ~6 lines recording them, so A6h's survey does not re-confirm them |
| 190 | unreachable | The five registered CDA cards each define both sides | — |
| 195 | unreachable | `grep -rn Layer3 src`: only the ordering slot, the declaration and the why panel's label | The field to own is now `AbilityText.words` (`fc119315`), not a bare `rules_text` |
| 196 | unreachable | `grep -rni "two.headed\|struct Team\|GameVariant" src`: nothing | `roadmap-v2.md` §D has no row that models teams (CR 805/810), so the slot's "the PR that models teams" rests on "CR 800's remaining subrules". Scheduling question for pass 6 |
| 203 | unreachable | `grep -rnoi "flashback\|jump-start\|retrace\|unearth" src/cards`: comments only; `can_begin_to_cast` refuses outside the hand | The slot names PM-0 to design and PM's later builds to build; say both |
| 205 | unreachable | `grep -rn every_creature_type src`: unit tests only | "One reader with a test" means one engine reader; `ui/display.rs:298` also reads it |
| 206 | unreachable | `grep -rn "RemoveType\|SetTypes"`: test boards only, no card | — |
| 209 | reachable — not wrong | `expect_pick_n` and the like: 333 positional answers in 70 files, beside 46 `expect_choice` answers in 17 | Re-count to 333 in 70 (~670 lines, one PR). The debt grew by 34 sites since 2026-10-03 although `expect_choice` exists |
| 213 | unreachable | `git log --since=2026-10-02 -p -- registry.rs`: none of the 34 cards registered since has a mana ability or a mana trigger | — |
| 219 | unreachable | `grep -rn "Effect::Conditional(" src/cards`: 12 card sites, all statics, none with a target | The walk is now `Effect::for_each_instance` (`effects.rs:2519`) |
| 220 | unreachable | `GainsThisAbility`'s one user is Cryptoplasm's trigger; Dimir Doppelganger and Lazav are unregistered | — |
| 227 | unreachable | `grep -rin "enchant player" src/cards`: none; `stack.rs:197-212` still errs on a non-object target | Add TR-4b's readers to the size (`EnterMods.attached_to`, the attachment records) |
| 233 | unreachable | `sed -n 60,82p triggers/placement.rs` and the clone tests: since CL-1 (`4ef103e4`), `clone_bound_test` and `fork_copies_test` clone at every decision of two or more options, but nothing *resumes* a fork mid-round | Reword "forks" to "resumes a fork" |
| 237 | unreachable | `grep -rn 'ObjectFilter::(Has\|With)?Keyword'`: no keyword leaf. CR 113.6k's trigger conditions have landed and read no keyword | Rewrite the last sentence: madness under B11 is the one remaining candidate |
| 240 | unreachable | `grep -n -E 'register\("(Hall of Gemstone\|Naked Singularity\|Harvest Mage)'`: none | Contamination and Infernal Darkness are still fixtures after item 6, so "register the day item 6 owns their upkeep halves" no longer names what blocks them |
| 256 | unreachable | `grep -rn -i "Copy(Ability\|Spell)" src`: no copy primitive | — |
| 257 | unreachable | `grep -rn -i "Keyword::Storm" src/cards`: none | The heading "has no slot" is stale; it is slotted to CV-4 (pass 5) |
| 265 | unreachable | `castable_spells` still enumerates only the hand | 1 line for pass 5: point PM-0's row (`permission-architecture.md:227`) at item 265 |
| 268 | unreachable | `grep -rn TriggeringPlayer src/cards`: no card authors it; its only author is a test fixture | The slot names no route row, so it is met by registering a card. Flag it for item 284's check, or re-slot it at A6s |
| 269 | reachable — not wrong | `awk '/pub enum AmountExpr/,/^}/'`: still seven per-object variants; none of the 16 cards since 2026-10-16 lacks a pairing | — |
| 271 | unreachable | `grep -rn AnyDefendingPlayer src tests`: one test only | — |
| 272 | unreachable | `sed -n 510,520p engine/resolve.rs`: `ForEach` still refuses at resolution | — |
| 273 | unreachable | `grep -rn 'XOffer::Set' src tests`: tests only | 2 lines: CR 606.6's cut is an interval; the set form comes from a target "with mana value X" |
| 274 | unreachable | `filter_reads` has no `MANA_COST` arm; `gather.rs`'s debug assert stands | — |
| 278 | unreachable | `combat/steps.rs:187` is still the only writer of a block event | — |
| 281 | unreachable | `grep -rn 'EffectRecipient::Group' mtgsim/src mtgsim/tests`: one card use (Loyal Sentry, an attacker and this object). `resolve_group_member` (`resolve.rs:2395`) refuses every other member; Hunting Kavu is unregistered | — |
| 283 | unreachable | `grep -rn -E 'Loyalty[A-Za-z]*\(\|LoyaltyCost' mtgsim/src`: no loyalty cost. Loyalty Probe has no abilities; CP-2 is unlanded | — |
| 284 | reachable — not wrong | `CARD_HOME_RE` (`check_state_of_play.py:199`) only classifies a home, and nothing compares a slot's named cards with the registry. The board counts 53 card-homed items | ~3 lines: the Slotted line names A6e's cleanup (session 6), not the hygiene sweep; replace the worked example (item 280, closed) with a live one; 53, not 55 |
| 285 | reachable — not wrong | A census of tuple-returning signatures finds 84 pairs today against 76 at TR-5c, by the same method (+8) | The count becomes about 88 by the item's own method (it said ~80) |
| 286 | unreachable | `grep -rn -E 'HistoryCount\|AmountExpr::History' src/cards`: the one `HistoryCount` has `of: None` | — |
| 287 | unreachable | `sed -n 437,470p scenario/build.rs`: a scenario's stated rows still carry no frame, and the debug assert guards them | The Slotted line notes that A6s's row already takes the item into its scenario audit |
| 288 | unreachable | `grep` for the six cards in `registry.rs`: only Selfless Squire, reading any prevention | — (stays with A6s, §5) |
| 291 | reachable — not wrong | `grep -rwo` for `Selector` and the four id enums: all three duplications stand | `set_affects` has 3 call sites, not "4 callers"; the counts are re-done at A6e's first ticket |
| 292 | unreachable | `grep -rniE 'target (player\|opponent)' src/cards`: every registered "target player" card has one target | 12 call sites, not "13 callers" |
| 294 | unreachable | `grep -rnE 'is_copy\s*(=\|:)\s*true' src tests`: nothing sets it | — |
| 295 | unreachable | `grep -rn 'InherentAbility::' src tests`: none; `TriggerOrigin::Rule` is never built | Optional: the slot cites `triggers-architecture.md:4534`'s Phase 9 list, since `roadmap-v2.md`'s Phase 9 never names the monarch; or a `backlog.md` entry |
| 296 | unreachable | `grep -rn SourceInZone`: 6 card uses, none under `Not`; the census matches exactly | — |

For items 279–296, the first run and the re-run agree on every class. The
first run also proposed a re-size for 284, which its refuters rejected.

**§13's 37 deferred atoms.** All the counts reproduce against
`plans/atomic-tests/spec.sqlite`: 133 atoms, 40 deferred and 37 uncovered.
`owed --phase "Phase 7" --all` lists exactly the 37. One sentence is wrong.
"The 13 of them still ticketed NEW" is false: none is ticketed NEW. The 13
are "Deferred to …" tickets (13 `- **Ticket:** Deferred to` lines across
the session files), which is why `owed` reads clean. Fix the wording. The
row sits at line 4332; the heading is at 4312.

25 are kept as they are. 12 had proposed changes: 3 survived and 9 were
refuted. Grouped by the fact they share:

| Atoms | Proposal | Outcome | Disposition |
|---|---|---|---|
| 605.3a-002; 603.12a-001, 702.21a-001 | Re-own the "unless [a player] pays" atoms. CP-1's row (`cost-architecture.md:1031`) names no "unless" payment, and `:1121` calls Rhystic Study's "item 6's", so the two docs point at each other | 605.3a-002 survived. The other two were refuted only on the target: all four of their refuters name A6s (`roadmap-v2.md:258` lists "unless [a player] pays") | Re-own all three to A6s together. Also fix the same stale pointer at `triggers-architecture.md:2036`, `:2934`, `:4530`, `replacement-architecture.md:526`, `:1008`, and `session-7b.md:785`, `:792` |
| 707.7-001 | Re-own: CR 707.7 copies a linked pair, and CR 607 is `backlog.md` §2.2's | Survived | Re-own to §2.2, slotted B7 (item 59). `roadmap-v2.md:324`'s "CV-1b's slice is 707.2, 707.3 and 707.7" is stale |
| 610.5-001, -002 | Re-own: the ticket's `backlog.md` §2.1 graduated on 2026-09-07 | -002 survived; -001 was refuted on "no doc carries convoke", since `mana-architecture.md:736` puts convoke at MA-5 | Re-own both. The payment half goes to MA-5. The 610.5 half (a grant at cast that outlives its source) has no owner in any doc, so A6s gives it one |
| 707.10b-001 | Re-own: a circular deferral between the copy doc and item 6 | Refuted. `c1174618` (A6p) fixed CV-4's row (copy doc `:1362`) on 2026-10-12 | Keep at CV-4. The stale prose is at copy doc `:892-895`, `:912`, `:1628-1632`, and `roadmap-v2.md:324` ("its 39 CR 707.10b clauses are item 6's"). For pass 5 |
| 707.9g-001 | Claim now, since TR-3c landed | Refuted: 707.9g's link is CR 603.11's, not 603.12's reflexive trigger (`tmnt.txt:5625`), and 603.11 is §2.2's | Keep deferred, with §2.2 decided at A6s |
| 208.2b-001, -002 | Re-own: "nothing names the entry choice" | Refuted: `replacement-architecture.md:1581` and `:1663` (§3.5, RG) name CR 613.2a's "as … enters" P/T placement at 1a, with no phase | Keep. The owner is that row, which needs a phase (pass 5, or A6s) |
| 702.176a-003 | Re-own: §13 and the ticket disagree | Refuted: both say Phase 8 (`backlog.md:300`, `cast-census.md:462`) | Keep |
| 608.2j-001 | Claim now: the vocabulary exists | Refuted: `Effect::ForEach` refuses at resolution (`resolve.rs:517-518`) | Keep. Or claim with `Atom(Destroy, FilteredPermanents(And(ByType(Creature), ByColor(Black))))`, which resolves through the sweep arm (`resolve.rs:358`); the close-out decides (~60–80 lines) |

**§15's 15 findings and §16's 12 bullets.** Both counts reproduce.

| Entry | Finding | Outcome | Disposition |
|---|---|---|---|
| §15.1 | Closed as built: `battlefield_readers` sorts on timestamp, and `candidates_now` puts the other legs in CR 613.7 order | Survived; three hash seeds matched | Close. A refuter adds that `zone_trigger_sources` is a second `IdMap` leg (`dispatch.rs:1260`), also sorted there |
| §15.2 | Partly closed: TR-5g queues and emits N, but no test puts a CR 603.3b tier-2 watcher under a multiplier | Survived | Rewrite. Then either write the ~20-line test (Panharmonicon plus the Lenient Proctor fixture at `tr1:1095`) as a close-out fix, or file it |
| §15.3 | Closed: the `CAST-ROLLBACK` tag is at both sites (`put_on_stack.rs:130`, `:732`) | Survived | Close. A refuter adds that `rollback_cast_to_hand` always returns the card to hand. That is safe while only hand casts exist (`NotInHand`), and nothing tracks it. A candidate item, re-read first |
| §15.4 | Closed by TR-4d's decision 1 | **Refuted**: `add_resolution_row` (`resolve.rs:1778`) re-keeps frames once a steal row exists. Ray of Command (registered, pooled) writes untap, `GainControl` and haste with no batch between, so the record's LKI may show the thief as controller | Keep open. **Reproduce first**: the refuter read the code and did not run it |
| §15.5 | Dredge has no owner anywhere | Unchanged | Move it to `backlog.md` §2 |
| §15.6 | Closed by TR-3b (`Duration::Indefinite`; Sneak Attack's ruling as fixtures) | Survived | Close. One ruling, quoted by two tests (`tr3b:397`, `:414`) |
| §15.7 | Closed by TR-2a: no prune | Survived | Close. Cite TR-5h too, since the store it measured is gone |
| §15.8 | Closed: one binding field | Survived | Close. Item 137's "~48" is now 98–108 literals, so A6e re-sizes it |
| §15.9, §15.10, §15.11 | Still open (CV-7; `backlog.md` §2.9 and B4; CP-2) | Unchanged | Keep open. For §15.11, confirm CP-2's row names `Cost::Discard` and `Cost::ExileFromGraveyard` |
| §15.12 | Move the `ObjectRef` and miracle notes into §2.2, owned by A6s | Refuted: §2.2 is slotted B7 (item 59), A6s only decides, and 607.2c is cited where 607.2a is meant | Keep; fix the citation. The close-out decides the move |
| §15.13 | Rewrite: three prompts hold state off `GameState` (item 233) | Refuted: TR-5f added a fourth, `ChooseEnteringDefender` (`actions.rs:2265`), inside a batch and not in item 233 | Rewrite with four, and decide whether item 233 takes it |
| §15.14 | Rewrite: `ZoneChangeCause` is gone and the cause is `act` | Refuted: the cause is `by: Option<SourceFilter>` (RH-1's `741b6d65`), and Guerrilla Tactics also writes `act` and `by` longhand | Rewrite naming `by` and `act`. A shorthand's name must not collide with `ReplacementDef::caused_by` |
| §15.15 | `SourcePower` is refused at resolution | Unchanged | Move to a `codebase-state.md` item with Reachability and Slotted lines |
| §16 bullets 1, 4, 5, 6, 8, 9, 10 | Still open, text true (modes at §2.7 by A6s; monarch at Phase 9; phasing at 'Before Triggered abilities' item 6; B4; CV-4; §2.12 and RE-10; B8) | Unchanged | Keep. Bullet 4 is common in Commander, so a `backlog.md` entry is worth weighing; bullet 9's RE-10 status was not checked |
| §16 bullet 2 | "Unless [a player] pays" points at cost §3.10, which never named it | Survived | Rewrite: no owner yet, A6s gives it one. With the same pointer at §6.2 and the copies above |
| §16 bullet 3 | Move miracle into §2.2 | Refuted: item 288 already names Phyrexian Vindicator as the 607.2h/603.11 trigger customer, and `layers-architecture.md:2167`'s "(item 6)" leads nowhere | Keep; fix the two pointers |
| §16 bullet 7 | Rewrite to B14's two-iteration comparison | Refuted: §4.9's amendment still keeps Tier 2's hash, and CR 731.3 and 731.5 stay in §2.28 | Rewrite to "B14, graduating §2.28", keeping 731.3/731.5 and both of TR-6's counts (`sba.rs:570`, `dispatch.rs:647`) |
| §16 bullet 11 | Closed by #188 and item 179 | Refuted: #188 bounded the window, but the history log grows by design (item 179's restatement; §3.10, "Nothing truncates the log") | Rewrite, don't delete |
| §16 bullet 12 | Point at `permission-architecture.md` §6/§7 | Refuted: that doc has no §7, its §6 is sizing, and no CR 116 action makes a delayed trigger. The pregame reveals are item 119's | Rewrite to item 119. The same stale §2.8 pointer sits at `triggers-architecture.md:716`, `delayed.rs:52`, `tests/phase_tr3a_integration_test.rs:808` |

**Item 96.** The promotion survived. `BOUNDARY-DEF-615.1a-001`'s unproven
member is the out-of-set one, a trigger on damage dealt. The engine has the
path now: `TriggerEvent::DamageDealt` (`types/triggers.rs:766`) and
`GainLife(TriggeringAmount)`. The helpers are in
`tests/phase_tr5d_integration_test.rs` (`gains_that_much` at `:280`,
`damage_to_is_prevented`).

The close-out writes a ~40-line two-member test, marked `// COVERS`, beside
them, and shows it fails against a broken engine first. Then it rewords the
`PARTIAL` comment at `types/replacement.rs:2208` and closes item 96: about 50
lines in all. Notes from the refuters:
- `damage_to_is_prevented` needs `.into()`.
- Mending Hands cast at a creature (`tr5d:440`) is closer to the atom than
  a hand-registered shield.
- The atom's "target creature" becomes "a creature", since
  `DamageRecipient` has no referred arm. The test's doc comment says so.

The item's text is stale too: Phase 6 shipped on 2026-09-15, and `owed` is
now 8, not 9. `COMP-614-616-DOUBLE-REPLACEMENT-001` is still covered.

**A fifth wrong-today item, from pass 3's discovery.** Row 57 of pass 3's
work-list: the log names a departed copy by its card. It was reproduced in
stress play and kept by both refuters. The close-out files it as a
`codebase-state.md` item with a dated `reachable — wrong today` verdict and
a slot, so the board's bolded row counts it and it stays scheduled.

**For the close-out to reproduce before acting.** Refuters raised these as
side findings, and no one has verified them:
- §15.4's Ray of Command board.
- §15.3's rollback zone, under CR 732.1's reversal.
- §15.13's fourth prompt.
- Item 264's Primal Vigor scenario, which hangs at about 12 copies. That
  matters to CI's stress step only if a deal builds it.

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

**Its cost is citations:** **183** of the doc's sections in tracked
`plans/` files (109 outside the archive; 181 and 107 before this file added
two) and **89** in `src/` and `tests/`, plus 2 written without the
backtick; §12 alone **85**. Citations are written with a backtick before the `§`. A
script lists every one against the heading it lands on, and an agent reads
only those whose target moved. Headings stay byte-identical, every moved
section leaves its heading and a pointer, and `check_glossary.py` runs on
the result.

#### Work-list (discovery, 2026-10-22)

From a script, not a workflow: `plans/references/trigger-citations.py`,
run as `python plans/references/trigger-citations.py . --list` from the
repository root. It reads every tracked
file under `plans/`, `mtgsim/src`, `mtgsim/tests` and `devgui/`. A citation
is the file name closed by a backtick, then `§` and a token. It also counts
follow-on tokens in the same phrase (`§4.2 and §4.3`) and, by heuristic,
the doc's own bare self-citations. Each one is landed on its heading and
classed against the table above.

**Totals at `c91657e1`.**

| Where | Citations |
|---|---:|
| `plans/` outside the archive | 109 (107 at `45d4a130`; this file adds 2) |
| `plans/archive/` | 74 |
| `mtgsim/src` | 70 |
| `mtgsim/tests` | 19 |
| `devgui/` | 0 |
| **Total, with the backtick** | **272** |
| Written without the backtick (`fuzz_games.rs:1178`, `phase_tr1b_integration_test.rs:169`, both `§4.10`) | 2 |
| Follow-on tokens in the same phrase | 33 |
| The doc's bare self-citations (heuristic; a few are other docs' sections) | ~295 |

These work-lists add four more citations of their own, so a run after #271
reads 113 outside the archive and 276 in all. The pass re-runs the script
on its own tree before it moves anything.

Counting both, §12 takes 85 + 12 follow-on. By the target's class, counting
both (305) with self-citations apart:

| Class | Citations | Self |
|---|---:|---:|
| moves whole (§0, §1, §2.1, §8, §14, §17) | 2 | 7 |
| partly moves (§4.9, §11.1, §12, §15) | 105 | 35 |
| stays | 198 | 253 |

One token lands on no heading yet: `§18`, this file's planned
retrospective (§6, decision 5).

**The moved-target citations**, outside the doc, split by the words that
follow each token. "Doubtful" means the words do not say which part is
meant. Those are the lines the pass's reader reads.

| Target | What the words name | Count | Where |
|---|---|---:|---|
| §14 | the whole section (moves) | 2 | `codebase-state.md:9499`; `archive/codebase-state-closed.md:7123` |
| §12 | a re-count or a roll-up (moves) | 2 | `archive/roadmap-v2-landed.md:382, 406` |
| §12 | a phase stub (stays) | 36 | 19 in `src/` and `tests/`, 17 in `plans/` (the script's `--list`) |
| §12 | doubtful | 59 | 20 in `src/` and `tests/`, mostly card-file headers (`phase_tr*_cards.rs:1`); 39 in `plans/`, 27 of them in the archive |
| §4.9 | doubtful (B14's input or the draw) | 3 | `engine/triggers/dispatch.rs:64`, `tests/phase_tr6_integration_test.rs:2`, `archive/codebase-state-closed.md:7540` |
| §11.1 | doubtful (its reading or the section) | 2 | `fuzz-record.md:4281, 4482` |
| §15 | findings 4, 5 and 14, each moving only if pass 1 closes it | 3 | `archive/codebase-state-closed.md:2366` (4), `tests/phase_tr4c_integration_test.rs:131` (5), `cards/authoring/triggers.rs:11` (14) |

In pass 1's work-list, §15's findings 4 and 14 stay, since their proposed
closes were refuted. Finding 5's disposition is a move to `backlog.md` §2,
so its one citation re-points if the close-out makes that move. The 7
self-citations of whole-moving sections and the 35 of partly-moving ones are
the doc's own. The pass re-points them as it moves the text.

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
`ContinuousEffectDraft` refactor, ~100), 215 (MA-1's two cleanups left; the
third went to items 242, 243, 246 and C0 on 2026-10-10), 239
(the crate-wide `.clone()` audit, with a fuzz A/B), 284 (~80 lines of
Python and a CI step), 269 (seven per-object `AmountExpr` variants, with the
naming sweep), `AbilityDef.instances`, the naming sweep itself, and the
model pass's survivors, folded into band-sized PRs by theme
(`engineering-practices.md#sizing-a-phase`). Each that moves a pool carries its own
A/B; each that changes a hot path, its own callgrind arm. The plan below
places each.

#### The cleanup's plan (session 6, 2026-10-23)

Sized on `50c459d4`, #282's merge. `check_state_of_play.py --slotted A6e`
lists 63 open items. Items 138, 142 and 143 are readiness's (pass 4), which
leaves 60:
- the card-homed triage's 31 builds;
- the model pass's 16 filed items, 297 (row 57) and 301–315;
- item 317;
- the 12 more that A6e's `roadmap-v2.md` row gives the cleanup: 137,
  215, 239, 247, 269, 284, 285, 291, 292, 296, 300 and "Before Layers" 7d.

With them go the work-list's 40 merge rows, which no item holds; the naming
sweep and `AbilityDef.instances` from A6e's `roadmap-v2.md` row; and
`--pool fixtures`. Each item's **Slotted:** line names its PR, so a
session's intake is `check_state_of_play.py --slotted <name>`.

**How each is sized.** An item is sized from its own **Sized:** line and a
row from its figure, both read against the tree. Where a build's line gives
no test size, its tests count as many lines as its code (#280's rule).
*Sized* is that sum. *Carried* is the code at 1.5–2×, the overrun sizings
have run, plus the tests; a counted sweep is carried as counted. The band is
`engineering-practices.md#sizing-a-phase`'s 1,500–2,500, code plus tests. "Pools" says
whether `performance` and `stress` are predicted to move. Every close-out
runs the A/B and the budget; a PR that moves a pool also owes the
attribution. "Hot path" names the path whose callgrind read the budget is
about.

| # | PR | What it delivers | Items | Merge rows | Sized | Carried | Pools | Hot path |
|---|---|---|---|---|---:|---:|---|---|
| 1 | `history-and-frame-records` | Every record written, each keeping the object as it was, three of the six record checks, and the fixture pool | 278, 279, 286, 297, 303, 305, 306, 310, 311, 312, 313, 314, "Before Triggered abilities" 6; `--pool fixtures` | 28, 29, 31, 32, 35, 36, 42, 45, 48, 49, 61, 62 | ~1,370 | ~1,700–2,000 | identical: no registered card reads a new row, frame or fact, and the fixtures pool apart | a row per zone change, last known information on a departure row, a `GameEvent` field; and a held-memory reading |
| 1b | `last-known-information-store` | Last known information in one store, keyed by existence (added at #284's review) | 318 | — | its design note's | — | its decisions' | a frame per move, and the clone floors |
| 1c | `practices-rules-page` | `engineering-practices.md` as a short page of rules, each three lines and a pointer, its records and measurements moved to the archive and the subsystem docs (added at #284's review) | — | — | docs | — | identical | none |
| 1d | `concept-census-and-register` | Each CR concept that names a thing, every type and field that holds it and its one owner; a gate that fails a new field holding an owned concept without a register line (added at #284's review) | — | — | its census's | — | identical | none |
| 2 | `resolution-context-rider` | `ResolutionContext`'s two rider numbers as one optional value | 137 | — | ~130 | ~130–160 | identical | none |
| 3 | `trigger-matcher-merges` | The trigger matcher's facts, each written once | 247, 301, 302, 304, 309, 315, 319, 320, 321, and 317 (decision 2) | 2, 3, 8, 9, 10, 12, 13, 15, 17, 18, 19, 20, 21, 22, 23, 24, 25, 30, 53, 54, 55, 59, 67, 68; and 5 | ~910 | ~1,250–1,600; ~1,350–1,800 with 317 | identical for the merges; with 317 both pools differ, since Blood Artist's order prompts go: a three-arm A/B | the dispatch, and the close-out's audited smoke (`engineering-practices.md#the-close-out`) |
| 3b | `record-existences` | Every record names its object by existence (split from 4 at #289's review) | 328 | — | ~300 | ~450–600 | both differ: a trigger names the existence its event was about, which is another where an object moved again in its own window; an A/B attributed to those windows | the dispatch's subject reads |
| 4 | `one-fact-one-type` | One type for each thing written as several, and `Condition` holding predicates only | 291, 296, 326, 327 | 46 | ~1,550–1,800 | ~1,550–1,850 | identical | the replacement gather's membership test |
| 5 | `durations-and-control` | "For as long as" ended with its object, the control sweep's announcements and record, a recipient for `GainControl`, and CR 514.2's wipe asking a restriction (split from `batches-durations-and-control` at #293's review; item 148 to `ironworks-loop` there) | 17, 20, 300, 324, 325, "Before Layers" 13 | — | ~670–965 | ~810–1,390 | both differ: 300's wide form announces Ray of Command's steal before its delayed trigger is created, attributed game by game | the duration sweep per departure, and the settlement per row a resolution writes |
| 5b | `batch-legality-and-entry-choices` | Phase 2 skips a stale member; an entry event's auxiliary moves and costs decided as they apply and performed as one batch (split from 5 at #293's review) | 26, 61, 62, 63, 134, 330, 332 (63 and 332 from first-card slots at #293's review) | — | ~730–1,030 | ~910–1,550 | identical | phase 2's check per member |
| 6 | `resolution-recipients-and-amounts` | The triage's recipient, target and amount builds | 30, 90, 94, 106, 155, 268, 269, 272, 281, 292, 299, 307, 329, 334 (334 filed at #294's review) | 38 | ~1,230, ~1,340–1,410 with 334 | ~1,500–1,800, ~1,640–2,080 with 334 | identical | castability's assignment (155) |
| 7 | `layers-combat-and-counters` | The triage's layer, combat and counter builds, one record check among them | 12, 86, 117, 184, 190, 205, 206, 308, 322, 323, "Before Layers" 16, "Before card breadth" 1, 3, 5 and 6 (12 from a first-card slot at #293's review) | 41, 44, 50 | ~1,330 | ~1,650–2,000 | identical | the layer walk |
| 8 | `names-and-hygiene-sweeps` | Names that read at the call site, and the hygiene pass's last items | "Before Layers" 7d, 215, 239, 285 (284 closed as moot at #294's review); the naming sweep; `AbilityDef.instances` | — | ~830–1,200 | ~1,200–2,200 | identical | 239's clones, 215's shared checks at priority |

That is **60 items and 40 merge rows in 9 PRs**, about 11,000–14,000
lines carried, since `record-existences` was split from
`one-fact-one-type` at #289's review. 1d's census (#287, 2026-10-24) added
ten items to five of them, each in its row above: about 400–750 lines, which
leave every row in the band (`trigger-matcher-merges` about 1,570–2,210
carried with 317, the largest). Item 291 does not pass the band
whole, so it is not split. Item 137 is under the band, a PR of its own on
item 124's precedent that a sweep does not ride inside a rules change.

**What each PR takes.**

1. `history-and-frame-records`.
   - Item 297: the log names a departed copy by its card where the line
     above names it as it left. The fix is one `GameEvent::frame_of`, and
     item 247's T17 log half goes with it. Reachable and wrong today, the
     board's row.
   - Item 278: an effect that makes a block writes its record (CR 509.3c),
     with ATOM-509.4-001's blocking entry. **Check:** every block the combat
     holds has its record.
   - Item 279: the zone-change "this turn" facts, "was cast" and "came under
     your control", read off the history log's rows. **Check:** each zone
     change a batch performs writes its row.
   - Item 286: a departure row keeps its record's last known information,
     which knows a token. **Check:** no history read asks the store about an
     object its row says departed.
   - Item 306 (row 34): "entered this turn" stored once. It is decided here,
     because 279 adds the first row readers.
   - "The first time each turn": item 303 (row 7) lists the facts placed in
     a player's turn once, and item 314 (row 65) makes its two answers one
     after the CR reading.
   - What a record keeps of an object that is gone:
     - item 305 (row 26): an unattach frames each end that left;
     - item 311 (row 58): a `LeftTheGame` binding carries its object;
     - item 312 (row 60): a card exiled from a hidden zone and returned gets
       its frame, folded into rows 61 and 62's one rule for which moves carry
       one;
     - item 313 (row 63): a sacrificed source's tapped status, one answer
       after the CR reading.
   - Item 310 (row 56): a sentinel test that the two lists of the objects a
     record names agree.
   - "Before Triggered abilities" 6: CR 603.6c's phased-in qualifier, at the
     frame's `if`.
   - `--pool fixtures`: every registered card plus the castable fixtures, in
     the `fuzz-checked` job, where the checks fire.
   - The merge rows:
     - 28, 29, 31, 32, 35 and 36: the history log's turns and counts, each
       written once;
     - 42, 61 and 62: a mover's before-move half, and which moves carry a
       frame;
     - 45, 48 and 49: the log's words for a replacement, a name and a
       delayed trigger.
1b. `last-known-information-store`, added at #284's review (the owner,
   2026-10-24). Item 318: CR 113.7a's last known information, kept today on
   records, proposals, captures, each keeper's list, history rows and two
   payment and departure structs, read through about fifteen functions,
   becomes one store keyed by existence. Its design note comes first, with
   three decisions: which moves write it, its shape against the clone
   floors, and its lifetime.
1c. `practices-rules-page`, added at #284's review (the owner,
   2026-10-24). `engineering-practices.md` is 175 KB, and its records and
   measurements bury its rules: its last-known-information example
   blessed the split item 318 undoes, and nothing asked the global
   question. The rules become a short page, each three lines and a
   pointer as `CLAUDE.md`'s are; the records and measurements move to the
   archive and to the subsystem docs they belong to, as the fuzz tables did
   to `fuzz-record.md`; a rule a script can check is listed as a gate to
   build. After the store, so the rule on one record is rewritten once.
   - Taken at #286's review (the owner, 2026-10-24): the short page at the
     same path; citations by heading anchor, not stable numbers, since a
     number says nothing about where it leads; the trace-page doc named
     `trace-pages.md` for what it holds; and the gate.
   - Asked at #285's review (the owner, 2026-10-24): whether the rules
     belong in `CLAUDE.md` instead. `CLAUDE.md` is at its 200-line limit,
     so the page is linked from it, unless the owner chooses otherwise at
     1c's review.
   - The page's example of a design that mirrors the CR is
     `departure_read` in `engine/triggers/dispatch.rs`, which the owner
     named at #285's review: an arm that looks back reads a departing
     object as its record framed it, and any other arm reads it as it is
     now, two match arms for CR 603.10 and 603.6c.
1d. `concept-census-and-register`, added at #284's review (the owner,
   2026-10-24). The model pass asked whether a fact is computed twice, a
   question about functions; nobody asked whether a CR concept is held in
   several places, which is how last known information came to twenty
   fields. The census extends `cr-coverage-audit.md`'s type-surface method
   from `GameState`'s fields to every type that holds a CR concept that
   names a thing, each with its one owner; a concept with two is a finding,
   slotted to the PR whose types it touches. The register is a gate in the
   `check_*.py` family: a new field holding an owned concept fails without
   its line. Before `one-fact-one-type`, whose scope its findings set, and
   `trigger-matcher-merges`.
   - The question the census asks first of each concept is its key, before
     its storage: TR-2b chose where last known information lived by how
     long each keeper needed it, and the key CR 400.7 gives it, the
     existence, came only with item 318.
   - Candidates named at #285's review (the owner, 2026-10-24), each to
     be confirmed or cleared by the census:
     - survivor frames: an object as it was before an event it survived
       (`ControlChanged`'s and `SameZoneMove`'s frames, `LookBackSnapshot`,
       `frame_before_control_change`), held per record;
     - which record ended an existence, held by each keeper that reads it
       (`DelayedTrigger.source_left_at`, `RememberedObject.left_at`), the
       shape the LKI store replaced;
     - item 30's "what paid" for a cost, which `pay_costs` returns only as
       mana, so a sacrificed creature is read back off the batch;
     - `ObjectInformation::Characteristics`, the catch-all its doc names
       as one: characteristics the asker holds, for five moments today (a
       spell being cast, CR 601.2f; a permanent about to enter, CR 614.12;
       a survivor's list from just before a batch, CR 603.10; a history
       row's object as the event left it; a live list already walked). The
       census decides whether it stays one arm, honestly named, or splits
       (moved here at #285's merge, the owner, 2026-10-24):
       - first, whether any filter on those paths reads a fact beyond the
         characteristics (tapped, attacking), which the arm reads live:
         now, where the characteristics are from another moment (a
         survivor's tapped state after the batch, an entering permanent's
         in its old zone);
       - then the shape. One arm leaves a reader that cares one answer for
         all five: `TriggerSubject::Host` answers "not attached", right for
         an entering permanent and wrong for a walked live list, which no
         caller sends there today and nothing stops. One arm per moment
         makes every reader decide. The recommendation at #285's review
         was a moment tag on the one arm, which readers that do not care
         ignore and readers that do must match.
   - Taken at #287's review (the owner, 2026-10-24): the concepts read from
     the tree, not from the CR, since a concept no holder holds cannot have
     two owners; a script and one reader, no fan-out (one agent read the
     pilot slice for 190,609 tokens, six times this session's reading); the
     register as `cr-coverage-audit.md`'s section 4c; the gate per holding
     type, never per field name, so a rename passes, in
     `check_type_surface.py`; the arm handing its two callers their `Lki` and
     tagging the rest with its moment; and one holder per key, with a
     record's values keyed by its event. The owner also ruled ante (CR 407)
     out of every design argument.
   - Done: the register's 21 concepts, the arm's moment (`FrameContext`), and
     items 319–327 in five PRs below. Survivor frames cleared: one concept,
     owned by the record or the snapshot its moment belongs to, with two
     hand-overs that hold the value before its owner exists.
2. `resolution-context-rider`. Item 137: every resolution that is not a CR
   615.5 rider writes two `None`s about riders. It becomes one optional
   value, at 99 literals (48 when the item was filed).
   - Done: 101 literals at `a0e4c51e`, 30 now `untargeted(..)` calls, 7
     taking the rest from it and 62 writing `replaced_event: None`; the close-out
     read identical, and −0.34% per decision, an inlining shift
     (`fuzz-record.md`).
3. `trigger-matcher-merges`.
   - Item 247, the inventory's T-entries, each named for what it merges:
     - T1: a trigger's tail, the first arm, the limits and the intervening
       "if";
     - T2: the queue (row 2);
     - T3: CR 603.2c's fold (rows 21 and 24);
     - T4: CR 603.7a's "existed before" (row 25);
     - T6: the candidate for an object (row 3);
     - T7: the multiplicities and record kinds (rows 17 and 53);
     - T8: whether an extra turn is still coming, which the waiting panel
       reads;
     - T9: rows 8 and 30, and whether an ability is triggered;
     - T12's other sites: a gone source's owner;
     - T14: a waiting entry's objects. Its dispatch half is gone, so it is
       re-read;
     - T15: which object a non-filter `ObjectSet` names;
     - T16: the "as long as" peel;
     - T19: a cost list's mana part;
     - T20: row 19;
     - T21: first strike asked three times, and declared attackers derived
       twice.
   - Item 301 (row 1): a state trigger's "not left the stack" walks
     `lki_keepers`' legs, the resolving one included.
   - Item 302 (row 4): the dispatch gate's legs come from one constructor,
     with today's legs. Decision 3 places the zone leg's filter.
   - Item 304 (row 11): `event_roles` is narrowed to what each arm's events
     carry, with a debug assert.
   - Item 309 (row 52): a template naming a referred object its arm never
     remembers is refused when authored.
   - Item 315 (row 66): `AbilityTriggers`' "caused by" goes through
     `triggering_events_in`.
   - Item 317: CR 603.3b's elision stops asking the order of one ability's
     agreeing triggers (decision 2).
   - The merge rows: 9, 10, 12, 13, 15, 18, 20, 22, 23, 54, 55, 59, 67 and
     68, as the work-list says. Row 5 merges as a `PendingTrigger` method.
     Row 27's two-line tidy and row 69's comments ride along.
   - Done (#289): the ten items closed and every row dispositioned in its
     body; +1,684 lines of code and tests. The close-out read the merges
     identical on both pools at two seats and four, and 317's moves each at
     a dropped Blood Artist (or Flickerwisp) order prompt
     (`fuzz-record.md`). Item 320 split in the doc: its other payloads are
     item 328, below.
3b. `record-existences`, split from `one-fact-one-type` at #289's review.
   - Item 328, split from item 320 at #289: the record payloads that still
     name an object by its id, each an `ObjectRef`. It moves the stream, so
     its A/B is attributed game by game to the windows where an object
     moved twice. Ahead of `one-fact-one-type`, whose object-or-player type
     by reference is built over its result.
   - Done (#290): item 328 closed; +607/−471 lines of code and tests, the
     types in five commits by family that read identical, then the
     readers. Neither pool moved (`fuzz-record.md`): no registered card
     puts a record and a later move of its object in one batch. The
     records' recipients and subjects are the by-reference
     `ObjectOrPlayer`, which item 291 merges with the rest.
4. `one-fact-one-type`.
   - Item 291, three things each written as several types:
     - `Selector` becomes an `ObjectSet`;
     - `ResolvedTarget`, `DamageTarget`, `CounterSubject` and `EventSubject`
       become one object-or-player type by id, and `TargetRef` and
       `ObjectOrPlayer` one by reference;
     - the affected pair becomes one type (CR 614.1).
     Row 64 folds the four matchers of an object-or-player slot with it.
     Counted on the tree, about 1,010 lines name the six id types and about
     140 write a player-only pair (71 `NO_OBJECTS`, 63
     `affecting_players`).
   - Item 296: CR 113.6b/c's zone statements move out of `Condition`, so a
     `Not(SourceInZone(..))` cannot state a zone silently.
   - Row 46: the `PlayerRef` that names a player for an entering object,
     merged keeping the refusal CR 616.1b needs.
   - Done (#291): the four items closed and rows 46 and 64 merged in
     twelve commits, the type merges first, each reading identical; item
     291 stayed whole. Two calls the plan did not make: `EventSubject` is
     an `Option<ObjectOrPlayerId>`, `None` for an event about the game
     (a draw), and `DamageDealt` and `DamagePrevented` name their
     `recipient`, since one type for every role leaves the field's name to
     say which. Item 326's holder is `PlayerHistory`, so a clone allocates
     once fewer.
5. `durations-and-control` (split from `batches-durations-and-control` at
   #293's review, whose design note,
   `plans/handoffs/batches-durations-and-control.md`, holds both halves'
   decisions).
   - Item 17: "for as long as" ends with the object it names. The
     replacement and restriction registries are swept, the watched object is
     read by existence, and the control leg is built. **Check:** no such row
     outlives that object.
   - Item 20: CR 514.2's wipe asks `Restriction::RuleDoesNotApply`, an axis
     keyed by the rule whose action stops, with the state-based check asking
     it for every CR 704.5 subrule.
   - "Before Layers" 13: "an opponent gains control", chosen at resolution
     above two players, through one function every instruction asks.
   - Item 300: a change of control waiting to be announced is announced
     before each continuous effect a resolution writes (the wide form).
   - Items 324 and 325: one record of what the last control sweep announced,
     and `controller_since_turn` gone.
   - Item 148: re-slotted to `ironworks-loop` at #293's review (the owner,
     2026-10-25). Its card targets an exiled card, a surface built with the
     graveyard targets that row already builds.
   - Done (#293): the six items closed in six commits, the control sweep's
     two records first, so item 17's control leg reads the reshaped one. Two
     calls the plan did not make: a resolution whose watched object has
     left, become a new object or changed hands registers nothing (CR
     611.2b's "never starts"), and the wide announcement settles once per
     effect, not per row, since a type change's sibling rows are one effect.
5b. `batch-legality-and-entry-choices`.
   - Item 26: batch phase 2 skips a member that is no longer legal (CR
     608.2b). Its check: a member that passed performs without `Err`.
   - Items 61 and 62: devour's sacrifices are decided as each applies and
     performed as one batch when phase 1 ends, counted by what will be
     performed (CR 701.21a), and chosen by the entering creature's
     controller (CR 109.5). **Check:** one entry event's auxiliary moves
     write one batch.
   - Item 134: an entry choice's cost, chosen under CR 614.12b's combined
     payability and paid with those moves.
   - Item 330 (added after #290, the owner): how a trigger's filter reads an
     object its own event moved. Both questions were answered at #293's
     review: a filter reads the object as the event found it.
   - Items 63 and 332 (re-slotted from "with the first registered card" at
     #293's review): devour by a count (Thromok the Insatiable), and
     tribute's choice of opponent, beside this PR's entry choices.
6. `resolution-recipients-and-amounts`.
   - Items 90 and 106: a resolution-created row keeps its targets for its
     rider, and `RetargetSpec::ToFixed` reads them.
   - Item 94: prevention rows per player.
   - Item 155: castability asks for an assignment of every "target" at
     once.
   - Item 268: "that player" names its participant.
   - Item 272: `Effect::ForEach` over players.
   - Item 281: `EffectRecipient::Group` takes a target, and each member
     fizzles alone.
   - Item 292: a one-player instruction whose player is gone does nothing.
     Row 39's two more sites go with it.
   - Item 334 (filed at #294's review, the owner: fixed before the audit
     closes): its object twin, a target an earlier instruction moved is no
     longer the object chosen (CR 400.7), so the instruction does nothing
     to it.
   - Items 299 and 269: "equal to its power" is read at resolution, and an
     object's value becomes one `AmountExpr::ValueOf`.
   - Item 307 (row 40): entering attacking is asked of the attacking
     player.
   - Row 38: one `subjects_from_context` for the recipients that do not
     target.
   - Item 329, filed at #289: a replacement's rider names its source, so
     `ExileUntil` in a rider can read "this".
7. `layers-combat-and-counters`.
   - Item 12 (re-slotted from "with the first registered card" at #293's
     review): a creature with no toughness, beside "Before card breadth"
     1's noncreature P/T, the station cards' rulings deciding the arm.
   - "Before Layers" 16 and item 184: one effect group per atom, and CR
     613.6's lock as the union within its layer.
   - "Before card breadth" 1: a noncreature permanent has no P/T (CR
     208.3).
   - "Before card breadth" 3: named counters, and CR 122.1a's other +X/+Y
     kinds.
   - "Before card breadth" 5: Layer 7c's doubling (CR 701.10a).
   - "Before card breadth" 6: multi-block damage (CR 510.1d). **Check:** a
     blocker's damage goes only among the attackers it blocks.
   - Item 86: infect, wither and toxic.
   - Item 117: land drops reset as the turn begins (CR 305.2).
   - Item 190: a CDA that defines one side of P/T.
   - Items 205 and 206: "every creature type", and CR 205.1a's correlated
     subtypes.
   - Item 308 (row 47): a condition's "you" read through one channel list.
   - The merge rows: 41 (Aura, Equipment, Fortification by effective
     subtype), 44 (counters of a kind) and 50 (a condition's source,
     tapped).
8. `names-and-hygiene-sweeps`.
   - The naming sweep: about 30 functions whose names are unclear where
     they are called. 289 free functions are called from another file today,
     against about 200 at #243's review, at a median of 2 call sites. The 30
     are listed at the PR's first ticket.
   - Three rules the owner set at #285's review (2026-10-24), each swept
     here:
     - `chars` is spelled `characteristics`, since a programmer reads it as
       characters first: about 400 lines in `src/` and `tests/` name a local,
       a field or a parameter `chars`, beside the accessors #285 renamed;
     - no id is called `seq`, not even a local (widened at #289's review):
       an `EventId`'s, left over from `EventSeq`, about 200 lines, the
       matchers' parameters, the locals and `EventRecord.seq` among them.
       The trigger's id, `TriggerId`, was renamed whole at #289;
     - a name that says last known information with other words in it
       takes the `lki_` prefix, as `lki_as_read_by` does.
   - Item 285: the tuple returns, sorted by `engineering-practices.md#names`'s
     rule 10.
   - `AbilityDef.instances`: the list the builder fills, taken out of the
     231 struct literals that write it, 205 of them empty.
   - "Before Layers" 7d: `ContinuousEffectDraft`, over 44 literals.
   - Item 215, MA-1's two cleanups:
     - the checks as methods on one value holding the inventory;
     - the memo audits taken over a clone, in debug builds.
   - Item 239: the `.clone()` audit, 390 sites. The 43 in `engine/layers/`
     are read first, then the 30 in `engine/replacement/`.
   - Item 284: closed as moot at #294's review (the owner, 2026-10-26),
     when the slot "with the first registered card" was retired.
   - Four rules of `engineering-practices.md` a script could check, slotted
     here at #286's review, each to `check_practices_page.py` or a check
     beside it:
     - `#comments`: a source comment cites the CR rule, never a plans item
       number, a doc's private label or a phase code, once the old lines in
       `src/` are swept;
     - `#names`: no bare pronoun, `Other` or `EMPTY` as a variant name, read
       off the enums;
     - `#trace-pages`: a page's `<head>` names its commit and CR version, and
       an architecture doc and `codebase-state.md` each link it;
     - `#the-spine-phase-audit`: every `ci.yml` step carries a comment giving
       its reason.

**Placed elsewhere, each with its reason.**
- Items 138, 142 and 143: readiness, pass 4.
- Rows kept by the work-list's reasons: 6, 16, 33, 37, 43 and 51, the last
  because no PR here touches the trace readers.
- Rows that joined an item with another home:
  - row 14 and row 10's prediction, item 213 at `mana-ability-costs`;
  - row 42's cast hand-over, item 265 at `permission-design`;
  - rows 39 and 64, items 292 and 291 above.

**Why this order.** Records first, for three reasons:
- it holds the one wrong-today item the audit filed;
- three of the six records with their checks land in it, so the fixture
  pool lands with consumers;
- item 306 decides the "entered this turn" store before item 279 adds its
  first readers.

The store of last known information follows it (#284's review), before
anything else reads or writes the fact again; then the rules page, so
§2c is rewritten after the store, and the concept census, before the
reshapes whose scope it may change. Item 137 comes next: it is
small, and every build after it writes one
`None` instead of two. Then the reshapes come before the builds, so the
builds read and write what stays:
- the matcher merges, which 17's control leg, 268 and 272 read;
- items 291 and 296, the types the builds write (decision 4).

The sweeps come last. The names settle once every build has landed, and
239's reading meets the hot paths as readiness will measure them.

**Decisions at the plan's review.**
1. **The order**: records first, as above, or the matcher merges first.
   Recommended: records first.
2. **Item 317**: it can ride `trigger-matcher-merges` as its last commit,
   with a three-arm A/B (main, the merges, the merges with 317), or stay
   its own PR as its slot says. Recommended: ride, one review fewer. The
   slot predates the sizing, and the third arm keeps the merges' "no
   behavior moved" readable.
3. **Item 302**: its slot puts readiness's pricing of the zone leg first,
   but readiness reads the tree the cleanup leaves. Recommended:
   `trigger-matcher-merges` builds the one constructor with today's legs,
   and readiness prices the zone leg's filter on it.
4. **Item 291**: before the builds, so they write the types that stay and
   the rename sweeps no new code, or after them, so one sweep covers the
   builds' references too, about 50–100 lines more. Recommended: before.

#### Work-list (discovery, 2026-10-22)

These come from Workflow B, `item6-discovery-model-pass`. Eleven finders
ran, each with about 35 tool calls:
- the five areas A–E, with `duplicate-inventory.md` §2a's prompt;
- two readers of the window's hunks in the shared files above (H1:
  `resolve.rs`, `actions.rs`, `types/effects.rs`, `game_state.rs`,
  `events/event.rs`; H2: `ui/display.rs`, `ui/why.rs`, `layers/board.rs`,
  `replacement/pipeline.rs`, `layers/condition.rs`, `layer_memo.rs`). The
  brief named nine finders and this paragraph's "plus the window's hunks"
  put these two in;
- the three cross-area readers, and one for `triggering_events_in`.

No finder saw `similar_functions.py`'s report or the inventory. Each finding
went to three refuters, each with one lens:
1. Is it truly one fact?
2. Do the copies already disagree? Show the input.
3. Does a test pin either copy?

A finding survived when two of the three did not refute it. Then a separate
dedup step ran `python plans/similar_functions.py` once (2,724 functions;
29 near-copies, 5 one-shape, 33 same-inputs, 42 same-composition and 27
same-test entries), read the inventory's §3.1–§3.10, folded internal
duplicates and marked each row. A follow-up, `item6-discovery-model-pass-reach`,
then checked whether a registered card reaches each row whose copies
disagree. Its "wrong today" verdicts each went to two refuters.

**Counts.** 119 found, 100 survived, 19 refuted. The 100 fold into **69
rows: 33 new, 35 known, 1 already merged.** 27 of the rows' copies already
disagree on some input. The follow-up found **one reachable and wrong today, row
57**, and both its refuters agreed. Six more are reachable but show no
wrong answer (rows 3, 26, 34, 45, 58, 63), and 20 are unreachable. Every
divergence was confirmed by reading both places.

By disposition:

| Disposition | Rows |
|---|---|
| File a new `codebase-state.md` item | 16: rows 1, 4, 7, 11, 26, 34, 40, 47, 52, **57**, 58, 60, 63, 65, 66 and 56's test |
| Join an existing item | 3: row 14 (item 213), row 39 (292), row 64 (291); plus parts of rows 10 (213) and 42 (265) |
| Merge in A6e's cleanup | 40 rows |
| Keep | 8 |
| Merge or keep | 2: rows 5 and 46 |

The 40 merge rows sum to about 620–670 lines as sized, before tests and
fixtures.

A calibration note for the next run: the test lens refuted one finding of
119. So in practice a finding survived unless lenses 1 and 2 both refuted
it, and the 84% survival rate measures those two lenses. The next run
should give the third lens a question it can answer "no" to.

What was dropped:
- Each finder read its own files in full, but the matchers outside its
  range only in excerpts.
- The two hunk readers read the largest trigger-phase hunks and named the
  commits they did not read. In `ui/why.rs` that is everything outside four
  hunks (`d1dceb9c`, `253233c3`, `02d6e845`, `793154b7` among them).
- The dedup step read §6 of the inventory only through a keyword grep.
- Rows H1, H2, X2 and X3#6 rest on the lenses' reading of the code, not the
  dedup step's.
- Every divergence comes from reading the code. Nothing was run but the
  follow-up's fuzz sittings.

**`TriggerEvent`'s eleven tables (§6, decision 2): a small fix, not a
design.** The counts hold: 30 variants, eleven exhaustive matches, and the
test at `types/triggers.rs:2144` covering 22 arms. The eight it misses are
Blocks, BecomesTarget, ActivatesAbility, IsCountered, PlayerLoses,
CountersPutOn, CountersRemovedFrom and ControlChanges.

The model pass confirmed one fact across the tables: which record kinds an
arm reads. `record_kinds` already owns that (row 53). The other tables
answer different questions from different fields, and the finding that
took "the eleven as one" was refuted 2 of 3 as not one fact. The work is a
`reads()` gate at the head of `triggering_events_in` plus a pairing test
over every admitted (arm, record kind), about 45 lines. Rows 17, 18, 22, 52,
54, 55 and 67 are the same kind of small fix inside `types/triggers.rs`.

Two things would be the design decision 2 describes: folding the tables
into one per-arm declaration, and row 11's `event_shape` type. Nothing found
here calls for either.

**One tension to weigh.** Row 18 survived: CR 603.10a's classes written
once for a move and once for an arm. Finding A#8 took the same pair whole
and was refuted 0 of 3: the `None` lifts differ on purpose, CR 603.6c
included, and tests pin the difference. Row 18's merge keeps those lifts in
`looks_back` and shares only the case that names both zones.

**The copies that already disagree.** Paths are under `mtgsim/src/`.

| Row | Fact | Places | Where they differ | Reach (2026-10-22) | Disposition | Size |
|---|---|---|---|---|---|---|
| 1 | Whether a triggered ability has triggered and not left the stack (CR 603.8, 608.2n), three walks | `triggers/state.rs:177`; `triggers/dispatch.rs:1104` `lki_keepers`, `:879` | A state trigger whose resolution emits an event while its state holds queues again: the resolving entry sits in `self.resolving`, which `state.rs` does not walk | unreachable: Emperor Crocodile, the only registered state trigger, sacrifices its own source. 1,000 stress games (`--require "Emperor Crocodile" --seed 101`) show no repeat | **File an item**; merge the walks onto `lki_keepers`' legs in A6e | ~25 + a test |
| 2 | Queueing a trigger: the CR 603.2h insert, the seq, the push and `caused_by` | `dispatch.rs:785`; `state.rs:67`; `triggers/delayed.rs:507` (`:515`), `:146` | A declined "when you don't" is announced as caused by record #0, the opening draw; `state.rs:95` says `None` | unreachable: only the Withheld Tithe fixture uses `when_you_dont`; Cornered Crook uses `when_you_do` | Merge as item 247's T2, one helper over `Option<EventId>`, failing first on the tr3c board | ~50 |
| 3 | The `TriggerCandidate` built for an object | `dispatch.rs:1417`, `:1401`, `:1454`, `:1491`; `triggers/audit.rs:181` (`:222`); `state.rs:134` | A gone object's card: the dispatcher's `frame_card`, the audit's bare builder | reachable — not wrong: a steal (Ray of Command, Act of Treason) and the owner's loss, at 3+ seats, produce it; the audit never compares or places the card (`--players 4 --require "Act of Treason,Ray of Command" --audit --seed 3`) | Merge as T6 | ~30–45 |
| 6 | A targeting matching a `BecomesTarget` arm's "by" and controller | `dispatch.rs:2130`, `:1880`; `triggers/history.rs:319`; `types/triggers.rs:245`; `triggers/binding_reads.rs:215` | Only on a definition the registry refuses | unreachable: Cephalid Aristocrat is the only registered arm | **Keep**: §3.8's reason holds | ~12 optional |
| 10 | A source that left before its event, read as it last existed | `dispatch.rs:1989`, `:2108`, `:2412`, `:2633`, `:2648`; `triggers/history.rs:281` | The mana prediction (`dispatch.rs:2447`) reads the source now; the real activation reads it as gone | unreachable: no registered `{T}` mana ability moves its own source | Merge (`SubjectRead::of_source`); the prediction's `lki: None` into item 213 (MA-7) | ~15 |
| 11 | Which event roles an instruction may name | `types/triggers.rs:1281`, `:1244`, `:448`; `dispatch.rs:2576`, `:2084`, `:2299` | Three arms declare a role no event of theirs carries, so the registry check passes a card that finds nothing | unreachable: no registered card names `DamageRecipient` or `Attacked` | **File an item**: narrow `event_roles` by field, fix `dispatch.rs:2571`'s comment, add a debug assert. An `event_shape` type would be decision 2's design | ~20 (`event_shape` 60–100) |
| 14 | Whether a permanent's triggered mana ability triggers on a production | `dispatch.rs:1762`, `:2418`; `oracle/mana_supply.rs:953` | The inventory passes no referents for an intervening "if", and never predicts an `ActivatesAbility` arm: two under-offers | unreachable: Wild Growth has neither shape | Into item 213 (MA-7) as two more shapes | ~40–80 |
| 18 | CR 603.10a's classes, for a move and for an arm | `types/triggers.rs:959`, `:977`; `dispatch.rs:1037`, `:1187`, `:446` | `{from: None, to: Hand}` looks back, but a Library→Hand move carries no frame | unreachable: no registered arm has that shape | Merge for the both-zones case only (the tension above) | ~10 + a test |
| 25 | CR 603.7a: an event from before a waiting effect, or from its own batch, never satisfies it | `triggers/delayed.rs:572`, `:267`; `types/zones.rs:34`; `engine/returns.rs:167` | A rider that runs `ExileUntil`, then sacrifices in the same batch: `is_due` answers at once | unreachable: only a replacement rider can; Banishing Light's is a stack trigger | Merge as T4 (`created_in`), failing first on a rider fixture | ~8 + a fixture |
| 26 | How each end of an unattach was just before (CR 603.10c) | `triggers/binding.rs:33`; `dispatch.rs:2187`, `:2539`; `engine/actions.rs:2109` | When both ends leave in one event, the record frames one | reachable — not wrong: Grafted Wargear and its host leave together when a player leaves a four-seat game (CR 800.4a), but the Wargear reads no `Attachment` role (`--players 4 --require "Grafted Wargear" --seed 2601`, game 3) | **File an item**: a frame per departed end | ~20 + a `GameEvent` field |
| 34 | Whether an object entered this turn, stored twice | `state/battlefield.rs:233`; `engine/object_filter.rs:279`; `triggers/history.rs:249` | Both ways: `test_support.rs:633` resets the field after the row; a scenario's "arrived this turn" sets the field with no row | reachable — not wrong: a scenario permanent loaded "arrived this turn" (Grizzly Bears) disagrees, but nothing reads the rows and no registered card uses `ObjectFact::Entered` | **File an item**: one store, decided before the first row reader (§3.10's owner) | ~20 |
| 39 | The one player a recipient names at resolution | `engine/resolve.rs:2993`, `:2822` | A `TriggeringPlayer` on an arm with no player: the controller draws, against `Ok(None)` | unreachable: only a tr5c fixture builds `TriggeringPlayer` | Item 292 holds it; add `named_player`'s `Ok(None)` and `EntersAttacking::Player` | item 292's ~60–80 |
| 40 | CR 506.2/506.3b: entering attacking only under the attacking player | `actions.rs:2262`; `state/game_state.rs:1651`; `combat/steps.rs:273` | The prompt reads the event's controller, the performer the effective one | unreachable: Hanweir Garrison's tokens enter under the attacking player | **File an item**; then an `is_attacking_player` beside `is_defending_player` | ~8 + the caller + a fixture |
| 42 | A mover's before-move half (frame, keepers, unattach) | `actions.rs:2052`; `engine/leaving.rs:112`, `:214`; `engine/put_on_stack.rs:309` (`:134`) | A cast from the graveyard hands a waiting trigger no frame; an exile does | unreachable: casts come from the hand only | Merge the two performers' half in A6e. The cast hand-over joins item 265 (PM-0); it needs `lki_now`, since a cast runs outside a batch | ~12; ~5 + a test |
| 44 | How many counters of a kind an object has | `layers/condition.rs:173`; `object_filter.rs:316`, `:49`; `layers/compute.rs:724` | `AtMost(0)` of an object with no LKI and no entry: the filter counts 0, the condition answers false (drifted at `ec2fbdd4`) | unreachable: persist and undying's `AtMost(0)` sit on a dies arm, which looks back | Merge on the filter's 0, failing first | ~10 + a test |
| 45 | The log's words for the replacements on a move | `ui/display.rs:948`, `:962` | A `SameZoneMove` line drops " (redirected)" | reachable — not wrong: Cephalid Aristocrat milling Darksteel Colossus (`--pool performance --require "Darksteel Colossus,Cephalid Aristocrat" --seed 45`); "stays in Library … replaced by" is still true | Merge `replaced_by_words`; the marker is the owner's one-line call | ~10 |
| 46 | Which `PlayerRef` names a player for an entering object | `replacement/pipeline.rs:2980`, `:3206` | Deliberately: with several opponents one asks (CR 616.1b), one refuses | unreachable | Merge keeping the `Err` (`pipeline.rs:1164` reads it with `.ok()`), or keep | ~25 |
| 47 | Which channels a condition reads to resolve CR 109.5's "you" | `layers/condition.rs:377`; `layers/board.rs:874` | A layer-2 conditional static whose "you" moves: `depends_on` is false, so CR 613.8a's order is missed | unreachable: no registered static lowers to a Layer 2 row | **File an item**: a dependency-order gap, with a Layer 2 fixture | ~5 + a fixture |
| 52 | Whether an arm reads a referred object (CR 603.7c) | `types/triggers.rs:1201`, `:1095`; `resolve.rs:914`; `dispatch.rs:2176`, `:2183`, `:2299` | A template naming `Referred` in a field `subject()` does not return, with nothing remembered, is made empty and never triggers, with no error | unreachable: Ray of Command's is a `ControlChanges` subject and it remembers | **File an item**: authoring validation, failing first | ~10–40 + a ~25-line test |
| 57 | Which frame names an object in the log | `ui/display.rs:874` (`:957`, `:1030`, `:1142`), `:1016`, `:970`, `:983`, `:1101`, `:1137`, `:905`; `events/event.rs:545`, `:558`; `triggers/binding.rs:26` | A departed copy is named by its card where the line above names it as it left | **reachable — wrong today**, both refuters agreeing: `fuzz_games --pool stress --require "Clone,Rancor,Holy Strength,Bonesplitter,Angelic Destiny" --copies 3 --games 100 --seed 57 --dump-events`, game 14: "621 ZoneChange: Blood Artist (Clone, #61) … Destroyed", then "622 Unattached: Angelic Destiny (#30) from Clone (#61)". Also `AbilityActivated` and `AbilityResolved` for Clone as Samite Censer-Bearer (seed 5700, games 10 and 38), and `ManaAdded` for Clone as Krark-Clan Ironworks under March of the Machines (seed 5757, game 199). The dev GUI shows these lines | **File a `codebase-state.md` item, reachable — wrong today**, slotted so the board's wrong-today row stays scheduled: A6e's cleanup, or a small fix sooner. The fix is one `GameEvent::frame_of(id)` that `record_frame` and `as_it_left` both read; log fixtures move (`event_log_test`, the trace pages) | ~25 + fixtures |
| 58 | Which existence a move that makes no new object names (`LeftTheGame`, `SameZoneMove`) | `dispatch.rs:2622`; `triggers/history.rs:269`, `:273`; `triggers/binding.rs:45` | A `LeftTheGame` binding carries no object, since the store entry is gone before the emit | reachable — not wrong: Banishing Light's "until" and Ray of Command's delayed arm match it in four-seat play, but neither reads the object (`--players 4 --require "Banishing Light" --seed 58`, game 16) | **File an item**, failing first on a four-seat fixture | ~8 + a ~60-line test |
| 60 | Which movers a keeper reads that the batch frames | `dispatch.rs:1045`, `:1199`, `:1071`; `actions.rs:2052` | A card in exile from a hidden zone, returned to the battlefield, gets no frame; the same card into the graveyard does | unreachable: Flickerwisp and Banishing Light return what they exiled from the battlefield, which no keeper reads | **File an item** with the fixture; fold the predicate into A6e | ~5 + a ~40-line test; ~25 |
| 61 | Which moves carry a frame the batch took | `dispatch.rs:1037`, `:1187`; `actions.rs:2046`; `leaving.rs:112` | One direction only: an unreplaced move within one zone is framed, never read, then truncated | unreachable | Merge `zone_change_looks_back_at`; the reach check found the over-capture covers every such move, not only Exile→Exile | ~12 |
| 63 | An activated ability's source's LKI once a cost moved it | `put_on_stack.rs:444` (`:463`; `mana.rs:67`); `dispatch.rs:1025`, `:1199` | `{T}`, sacrifice: `AbilityActivated` records the source untapped, the departure frame tapped | reachable — not wrong: Mind Stone and Dark Sphere produce it (`--require "Mind Stone,Dark Sphere" --seed 63`); nothing a registered card has reads `was_tapped` off the record | **File an item**: a CR read of 602.2b and 605.3a against 113.7a | ~30 + a ~40-line test |
| 65 | "The first time each turn", answered two ways | `dispatch.rs:1880` (`:1881-1889`), `:2130`; `triggers/history.rs:57`, `:180`; `binding_reads.rs:209`; `types/zones.rs:304` | A filtered `CastsSpell` arm, an instant then a creature: the ordinal refuses the creature | unreachable: Vengeful Warchief's arm is unfiltered | **File an item**: a rules-reading question against §3.5's "first of its kind" | ~40–80 + fixtures |
| 66 | "The event that caused this trigger was arm X" | `dispatch.rs:2158`, `:1613` | The `AbilityTriggers` arm checks the record's kind, a multiplier the full predicates | unreachable: nothing builds `AbilityTriggers` with `caused_by: Some` | **File an item**: route through `triggering_events_in`, or drop the field | ~5–10 + a fixture |
| 69 | Splitting a combat declaration's pairs into one side's occurrences | `dispatch.rs:2086` (`:2072`), `:2309` (`:2321`) | Attacks and blocks order their groups by different rules | unreachable: no registered `AttackOccurrence::Defender` arm | **Keep**: both orders are deterministic and the CR fixes none; record each in a comment | ~15 |

**The copies that agree: merge in A6e's cleanup** unless the row says
otherwise. "Known" names where the inventory or the report already lists
the fact; each row cites the place.

| Row | Fact | Places | Status | Disposition | Size |
|---|---|---|---|---|---|
| 4 | Which objects may carry an ability reading a kind mask (the gate's legs, put together by each caller of `candidates_now`) | `triggers/dispatch.rs:735` `detect`, `:1340` `find_trigger_matches`, `:1252` `look_back_frames`; `triggers/state.rs:106` | new | **File an item** (which filtering the zone-source leg uses is a dispatch-gate performance question). Fix the stale comment at `dispatch.rs:754-756` in the sweep: the map is keyed by object, not by ability | ~25 |
| 5 | One player's waiting triggers of one tier | `triggers/placement.rs:63`, `:97` | known (report, same test) | Merge as a `PendingTrigger` method, or keep | ~4 |
| 7 | The placed kinds: which facts carry a place in their player's turn, so which arms can tell "the first time each turn" | `triggers/binding_reads.rs:209`; `triggers/history.rs:57`; `state/history.rs:41` `PLACED_FACTS`; `dispatch.rs:1889` | known (§3.7–§3.9; `PLACED_FACTS` is a fourth copy) | **File an item.** §3.7's keep covers one direction only: an arm the gate admits but `placed_among` gives no place silently never fires. One `TriggerEvent → Option<TurnFact>` map, or a test that the three lists agree | ~20 |
| 8 | A `LifeChanged` is a gain or a loss, and by how much (CR 119.3, 119.9) | `dispatch.rs:2013`, `:2016`; `triggers/history.rs:241`, `:342`; `types/triggers.rs:1426` | known (T9, T21, §3.8) | Merge: `GameEvent::life_change()` | ~15–20 |
| 9 | Where an entering permanent came from, and none for a token (CR 111.2) | `dispatch.rs:2554` `entry_origin`; `triggers/history.rs:249` | new | Merge: `draft_history_rows` calls `entry_origin` | ~10–15 |
| 12 | A candidate's `TriggerReferents` | `dispatch.rs:1574`, `:1777`, `:2439` | new | Merge: `TriggerCandidate::referents` | ~10 |
| 13 | A set of permanents in CR 613.7 order | `dispatch.rs:1303`, `:1515`, `:922`; `oracle/board.rs:22`; `state/game_state.rs:1392` | new | Merge: `GameState::in_battlefield_order(ids)`, with `gather.rs`'s distinct-key debug assert | ~25 |
| 15 | One hit made a `MatchedTrigger`, with `is_mana_ability` computed twice per hit | `dispatch.rs:1710`, `:1738` (`:1677`, `:1700`) | new | Merge: one constructor, mana computed once | ~15 |
| 16 | CR 603.2h's once-each-turn key, at the trigger and at resolution | `dispatch.rs:1878`; `triggers/placement.rs:158`; `engine/stack.rs:166` | known (T5) | **Keep**: both moments are deliberate (Nykthos Paragon's ruling) | ~8 optional |
| 17 | Which arms carry CR 603.2c's multiplicity, read by a getter and a setter | `types/triggers.rs:1206`; `cards/authoring/triggers.rs:193` | known (T7, §3.9, §6 near-copies 17) | Merge as item 247's T7: `multiplicity_mut`, so the setter loses its `unreachable!`, plus a round-trip test | ~45 |
| 19 | Each `TriggeringEventObjects` variant's roles | `types/triggers.rs:448` `get`, `:485` `objects` | known (T20) | Merge as T20: a private `pairs()` | ~−25 |
| 20 | Which multiplicities have one triggering event for "it" | `types/triggers.rs:1577`; `dispatch.rs:1847` | new | Merge: `Multiplicity::has_one_triggering_event()` | ~8 |
| 21 | CR 603.2c's fold of a window's matches into triggers | `dispatch.rs:1706`; `triggers/delayed.rs:449` | known (T3) | Merge as T3: one fold | ~30 |
| 22 | What one occurrence of a counters arm is | `dispatch.rs:2364`; `types/triggers.rs:1431`; `triggers/binding.rs:174` | new | Merge: one `Multiplicity` method returning (events, each) | ~10–15 |
| 23 | An existence remembered with the departure that ended it (CR 400.7), its "only once" rule written three times | `types/triggers.rs:1751`, `:1843`; `triggers/delayed.rs:221`, `:370` | known (report, same test) | Merge: `DelayedTrigger.source` becomes a `RememberedObject`. It touches the scenario save and load, which nobody has read | ~20–40 |
| 24 | The `MatchedTrigger` a delayed entry makes | `triggers/delayed.rs:168`, `:434` | known (T3) | Merge as T3: `matched_of` | ~15 |
| 27 | The resolving ability's source as it last existed | `triggers/delayed.rs:85`; `engine/resolve.rs:2875`; `triggers/binding.rs:245`, `:231` | already merged (T13, #251) | Keep, or a two-line tidy dropping the redundant guard in `delayed.rs` | ~3 |
| 28 | Where a draw falls in the turn and the draw step | `engine/zones.rs:320`, `:336`; `triggers/history.rs:180`, `:191`, `:155` | known (§3.8, whose keep's premise no longer holds: `place_in_turn` reads the `TurnTally`) | Merge: `next_draw_ordinals` on `count_in_turn + 1` | ~20 |
| 29 | Turn 0 counts toward nothing (CR 103) | `state/history_log.rs:445`; `triggers/history.rs:182`; `zones.rs:324`; `scenario/write.rs:310` | known (§3.8) | Merge: a named counted-turn notion | ~10 |
| 30 | CR 603.7h's "the same ability, whichever instance" | `triggers/history.rs:152`, `:202`; `scenario/build.rs:632`; `dispatch.rs:2168`; `scenario/write.rs:642` | known (T9; the loader is a fourth copy, the writer a fifth dependent) | Merge: `AbilityIdentity::ignoring_grant()` | ~10 |
| 31 | Which row kind a `TurnFact` counts | `state/history_log.rs:166`, `:188` | known (§3.8 kept) | Merge with a debug assert. §3.8's keep misses a fact counted on two row kinds (Revolt's) | ~5 |
| 32 | `TurnFact`'s variants and their count | `types/history.rs:26`, `:55` (`COUNT` is the literal 9); `scenario/text.rs:386` | known (§3.9) | Merge: `counted_enum!`, `every_turn_fact` → `TurnFact::ALL` | ~15 |
| 33 | The record in a window whose LKI is a given existence | `triggers/history.rs:216`; `dispatch.rs:2539` | known (L3: the method is `GameEvent::lki()` now) | **Keep**: what remains is one `find_map` over two record sets with different fallbacks | ~8–10 |
| 35 | The log's rows for a span of turns | `state/history_log.rs:313`, `:447`; `scenario/write.rs:310` | new | Merge: a private `rows_of_turns(range)` | ~6 |
| 36 | Which turn a scenario lands each span on | `scenario/build.rs:543`, `:581` | known (§3.8 kept; §6 same composition 40) | Merge one span-to-turn helper; each keeps its own `ThisGame` rule, the reason §3.8 kept them | ~10–15 |
| 37 | A player's count of a placed fact on one turn, incremental and re-summed | `state/history.rs:57`; `state/history_log.rs:435` | new | **Keep**: a cache over one rule, `added_to_count`. Optional debug cross-check | 0 (~5) |
| 38 | What a non-targeting recipient names from the context alone | `resolve.rs:289`, `:2395`, `:2833` | known (§3.7 kept) | Merge: one `subjects_from_context`, each caller filtering it (§3.7's two questions as two filters) | ~40, net −15 |
| 41 | Aura, Equipment or Fortification by effective subtype (CR 303.4h, 704.5p) | `engine/actions.rs:641`; `engine/sba.rs:481` | new | Merge: one predicate in `oracle/characteristics.rs` | ~10 |
| 43 | CR 508.4's candidates for `EntersAttacking::Chosen` | `resolve.rs:2791`; `actions.rs:2345` | new | **Keep**: the rule is once, in `what_can_be_attacked` | 0 |
| 48 | The log's short name for a player or object | `ui/display.rs:928`, `:932`, `:1002`, `:1063`, `:1077` | known (T29; §6 composition 14) | Merge: `player_in_log` at all 41 `P{n}` sites; rename the shadowing closure | ~15+ |
| 49 | A delayed trigger's name in the log | `ui/display.rs:891`, `:973`, `:977`, `:988`; `devgui/src/view_model.rs:1081` | new | Merge: extend `ability_word` | ~6 |
| 50 | The source's tapped status as a condition reads it | `layers/condition.rs:157`, `:164` | new | Merge: a local `source_tapped()` (not `CurrentOrLastKnown`, which loses the look-ahead) | ~8 |
| 51 | A trace record is a state trigger's answer | `ui/why.rs:613`; `ui/what_happened.rs:53`, `:162` | known (report, composition) | **Keep** unless A6e already touches the trace readers | ~6 |
| 53 | **The tables as one**: which record kinds each arm reads, paired again in five places, several ending in a silent catch-all | `types/triggers.rs:1042`, `:1325`, `:1418`, `:1095`, `:1281`, `:1188`; `dispatch.rs:1909` (catch-all `:2290`), `:2622` (`:2637`), `:1613` | known (T7; §6 one shape 2; this pass's reader 1) | Merge as item 247's T7: `arm.reads(&record.event)` opens `triggering_events_in`; the act re-check at `:1955` goes; a pairing test over every admitted (arm, kind) | ~45 |
| 54 | Which arms may be read once per subject, and their subject | `types/triggers.rs:1149`, `:1188`; `dispatch.rs:1729`; `delayed.rs:456` | new | Merge: `occurrence_subject` exhaustive, `supports_once_per_subject` derived | ~30 |
| 55 | CR 603.10's look-back list, hard-coded again for four arms | `types/triggers.rs:976`; `dispatch.rs:488`, `:2033`, `:2183`, `:2194`, `:446`; `binding.rs:26` | new | Merge: a `SubjectRead` constructor keyed on `looks_back`, or keep (CR 603.10c–e fix the three) | ~15 |
| 56 | Which objects a record names, for the names it keeps | `events/event.rs:572`; `ui/display.rs:924`; `state/trace.rs:895` | new (this pass's reader 2) | **File an item**: a per-variant sentinel-name test pinning the two lists; no merge, since `format_event` needs roles | ~50 test |
| 59 | An unattach names each end as the record holds it; `subject_after`'s `Unattached` branch is dead | `dispatch.rs:2700`, `:2681` | new | Merge, or delete the dead branch | ~5 |
| 62 | A departure of an object with no battlefield entry yet looks back at nothing | `dispatch.rs:1031`, `:1186` | new | Merge into row 61's `zone_change_looks_back_at` | ~6 |
| 64 | An "object or player" slot matched by four hand-written matchers | `dispatch.rs:2385`, `:2121`, `:2453`; `types/triggers.rs:277`, `:186`, `:224` | known (item 291) | Item 291's work: merge `CountersOn` and `TargetSubject`; `DefenderFilter` keeps its own | ~25 (+40) |
| 67 | The zone-change arm's shared questions, once per record kind | `dispatch.rs:1921`, `:1936`, `:1951`; `types/triggers.rs:1046`; `dispatch.rs:2205` | new | Merge one owner test; rides row 53's gate | ~15 |
| 68 | The two counter arms' shared questions | `dispatch.rs:2219`, `:2231` | new | Merge: `counter_record_matches` | ~12 |

**What the dedup found the inventory and the docs get wrong.** These are
for pass 5 and for A6e's sizing:
- T2's "for an empty match, which no match is": `when_not_done` builds one.
- §3.5's T4 "no rider makes one": a fixture rider can.
- §3.7's "refused loudly, never answered wrong": one direction is silent
  (row 7).
- §3.8's draw-place keep: its premise is gone (row 28).
- Item 290's closing claim: only one end carries its frame (row 26).
- `triggers-architecture.md:3708` and `5accb643`'s "`looks_back` reads
  `zone_change_looks_back`": it does not (row 18).
- The comment at `dispatch.rs:754-756` (row 4).

**The 19 refuted.** A#4, A#6, A#7, A#8, B#8, B#16, C#11, D#6, D#8, E#15,
H1#7, H2#9, X1#10, X2#8, X2#9, X3#1, X3#7, X4#2 and X4#4. Lenses 1 and 2
refuted eighteen, as different facts on purpose or copies with different
jobs. The test lens alone let them stand. All three lenses refuted A#8. The
workflow's journal keeps their texts.

### The card-homed triage

**Added by the owner at #272's review (2026-10-22).** 51 open items, by the
board's count on 2026-10-22, are slotted "with the first registered card"
that needs them. Card breadth would meet each inside a card's PR, unannounced:
engine work, and often a bug on a path no registered card has driven. Fuzz
cannot see those paths. §15.4's bug lived through every `stress` run with
Ray of Command pooled, because no registered card read the victim's side of
the change.

**Goal:** each card-homed item read once and given one of three dispositions:
1. **Build before breadth.** A fixture test of the CR's answer (the CR is
   the customer, not a printed card). Where the item writes a record, also a
   consistency check on it that a debug test or an `--audit` run trips. That
   kind of check is what would have caught §15.4 with no reader: with "a
   change looks back to a frame its loser controlled" asserted, all four of
   TR-4d's Ray of Command tests fail on the pre-fix tree.
2. **A route row**, for an item that needs a design: re-slotted to the row
   whose work it is. Items 273 and 274 to B13, which builds the first filter
   that reads a mana value, is the first candidate.
3. **Stays card-homed**, with the reason a card, not the CR, decides its
   shape.

**Whether to register fixtures** in a fuzz pool is the triage's question to
price: fuzz drives a registered fixture, but it flags only loud failures, so
a fixture earns its place in a pool through the consistency checks above.
The cost is in the readings: a third pool beside `performance` and `stress`
keeps the real-card numbers clean.

**Sizes.** The read is one docs PR. About 25 of the items are sized at 60
lines or less, so building them is about 1,000–1,500 lines with their tests,
one or two band-sized PRs that ride A6e's cleanup sessions. The ones that
need a design go to rows and build nothing here. Item 284's check, in the
cleanup, then guards whatever stays card-homed.

**Done (#280, 2026-10-23).** Each item was read once, its verdict re-dated
and its disposition written into its `**Slotted:**` line; the PR's body has
the table, item by item, with the reason and, for a build, its fixture and
check. Re-counted, 28 of the 51 state a size of 60 lines or less, 17 more,
and 6 give none in lines. The board's card-homed count went from 51 to 5.

- **Built before breadth: 31,** slotted to A6e's cleanup, about 3,500 lines
  with tests by their own `**Sized:**` lines. Six write a record and carry a
  consistency check in debug builds: 17 (no registry row outlives the object
  its duration names), 61 (one entry event's auxiliary moves in one batch),
  "Before card breadth" 6 (a blocker's damage only among the attackers it
  blocks), 278 (a block record for every block), 279 (a row for every zone
  change) and 286 (no history read asks the store about a departed object).
  Two themes of about a band each: resolution, replacement and batches (17,
  20, 26, 61, 62, 90 with 106, 94, 134, 148, 155, 268, 272, 281, 299 and
  "Before Layers" 13), about 1,800 lines; layers, combat, counters and the
  history ("Before Layers" 16 with 184, "Before card breadth" 1, 3, 5 and 6,
  "Before Triggered abilities" 6, 86, 117, 190, 205, 206, 278, 279 and 286),
  about 1,700. Sizings have run 1.5–2× on code, so the cleanup's sizing may
  cut a third PR.
- **To a row: 14.** A6s places nine, each a design: 23, 28, 79 with "Before
  card breadth" 4 (one question: when an order the CR hands a player is
  asked), 87, 119, 125, 227 and "Before Layers" 7, which leaves
  `roadmap-v2.md` §3a's off-the-route sentence. B13 takes 274 and its
  partner 273, which had read as route-homed only through a parenthetical
  CP-2; MA-7 takes 133, MA-4 and MA-3 take 240, PM-0 takes 147 and CP-2
  takes 283.
- **Kept with their cards: 5,** each with its reason: 12, 63, 80, 127 and
  135. Item 284's check guards them.
- **Closed: item 109,** found fixed by #239 (item 229) and #250.

**The pool question: a third pool (the owner, 2026-10-23, at #280's
review).** `--pool fixtures` is every registered card plus the registered
fixtures, run only in the `fuzz-checked` job, where the consistency checks
fire: about 14 s for 50 games at four threads by that job's own stress
reading, and about 40–60 lines (a `CardPool` arm in `fuzz_games`, fixtures
registered apart from `default_registry`, a row in
`engineering-practices.md#cards-and-pools`'s table). `stress` stays real cards, and the fixture pool keeps no table.
It lands in the cleanup with the first build PR whose fixture carries a
check, so the pool has a consumer; only castable fixtures join it.

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

**Re-cut at #274's merge (the owner, 2026-10-22).** Four PRs, a session
each: (1) the contracts — `CLAUDE.md`, `engine-map.md`, and every row the
contract-change readers found, with `engine-map.md` §1.2's unread steps;
(2) layers, replacement, cant-effects, copy and cost, with their contested
rows and unread sections; (3) triggers, setup, permission, mana and clone,
the registers (`codebase-state.md`, `backlog.md`, `roadmap-v2.md`,
`engineering-practices.md`, the glossary, the censuses) and the source doc
comments, with pass 3's six claims for this pass; (4) item 185 and the two
trace pages, split in two if it passes the band. **Item 185 is built, not
only written:** the inventory, each elision with its claim, its premise as a
property and what holds it, and **an engine record at each elision stating
its reason**, kept in a game's log at the level of a land tapped for mana —
in the log, and not drawn in the dev GUI's (the owner's words). **Item 188
left this pass** for A6s, the route review, which places it among the rows
before Phase 8's card breadth: confirming a candidate is a read of the code
and then a merge, which is cleanup work, not a doc fix.

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

**Two trace pages, from pass 1.** The close-out decided each TR phase's
trace page (`triggers-architecture.md` §12, each heading). TR-4d's control
sweep and TR-5h's history log change how a read is answered, and neither
has a page: they are owed, and the owner earmarked both for this pass at
#272's review. They are written in this pass's PRs at
`trace-pages.md`'s budget, two to three hours each.

#### Work-list (discovery, 2026-10-22)

These come from Workflow C, `item6-discovery-docs-vs-tree`. Nineteen
readers ran, each with about 35 tool calls:
- one per architecture doc, ten, plus `engine-map.md`;
- one per contract change in §1's six, each reading every doc that states
  it;
- one for `CLAUDE.md`'s invariants against the six;
- one that collected the "For A6h" lists from the bodies of #171–#269.
  Those were saved from `gh pr view`; #184 and #218 do not exist.

Each claim a reader called stale went to two refuters. One re-ran the
reader's evidence. The other looked for what makes the doc right: the same
thing under another name, a later qualifying section, a dated record, a
plan. A claim survived only if neither refuted it. Where one did, it is
listed below as contested.

A compression step then wrote each survivor as one row and folded exact
repeats, the same claim found by a doc reader and a contract reader. It
kept every survivor and added nothing; the refuters' better corrections
replace the reader's where they gave one. "Stale" means the doc states
something about the engine that the tree no longer bears out. Dated
records, plans and CR quotations were left alone. `plans/archive/*`,
`fuzz-record.md` and the generated board were not audited.

**Counts.** The readers checked 929 claims and found them true. They called
268 stale: **208 survived**, 36 are contested and 24 were refuted. The 208
fold into 190 rows (18 repeats), and the 36 into 35.

| Doc | Survived | Rows | Contested | Refuted |
|---|---:|---:|---:|---:|
| `CLAUDE.md`, and `engine-map.md` §2.7 | 7 | 6 | 0 | 2 |
| `engine-map.md` | 13 | 12 | 6 | 1 |
| `layers-architecture.md` | 22 | 21 | 4 | 1 |
| `replacement-architecture.md` | 22 | 18 | 5 | 1 |
| `cant-effects-architecture.md` | 13 | 13 | 1 | 0 |
| `copy-effects-architecture.md` | 23 | 23 | 2 | 2 |
| `cost-architecture.md` | 19 | 14 | 1 | 5 |
| `triggers-architecture.md` | 26 | 23 | 2 | 1 |
| `setup-`, `permission-` and `mana-architecture.md` | 16 | 15 | 4 | 2 |
| `clone-architecture.md` | 9 | 7 | 7 | 2 |
| `codebase-state.md` | 12 | 12 | 2 | 2 |
| `roadmap-v2.md`, `backlog.md`, `engineering-practices.md`, `glossary.md`, `cast-census.md`, `cr-coverage-audit.md` | 16 | 16 | 1 | 4 |
| Source doc comments | 10 | 10 | 1 | 1 |
| **Total** | **208** | **190** | **36** | **24** |

**The "For A6h" lists.** They are in #200 (eight items), #210, #223 and
#243 (the `place_on_battlefield` name: 43 places by #243's count, 41 now). The other mentions are
not list items. #197 and #198 schedule item 188's code survey with A6h, and
#209, #210, #246 and #269 name A6h in passing.

Of the listed disagreements:
- 22 were called stale. 17 survived and are rows below, 2 are contested,
  and 3 were refuted as dated records.
- The 17 still standing include `CLAUDE.md`'s "two gates, three legs"
  and `FrameCache` lines, `codebase-state.md` item 40's pointers and its
  TL;DR counts, and `roadmap-v2.md` A4d, B3, B5 and §9.
- The rest is the collector's own reading, which went to no refuter. 3
  have been fixed since: #210's two `roadmap-v2.md` phrases (`9ee2fba8`,
  and a rewrap) and #223's `ui/why.rs:197` comment (`02d6e845`, SU-8).
- 1 is partly fixed: #200 item 4's "Five architecture docs" became
  "Eight", but there are ten.
- Several stand only as dated records: #200 item 2's `FrameCache` in
  `replacement-architecture.md` §5, §5d and RC-4, and `layers-architecture.md`
  `:1734`; #200 item 1's `copy-effects-architecture.md:989-991`; #210's
  triggers §17 and its dated-prose list in `roadmap-v2.md` §2, §3a and §3b.
  The pass may annotate them. Under the run's rule they are not stale.

Of #243's 41 `place_on_battlefield` names, 8 were sampled. The row is
contested, because some name `test_support`'s helper on purpose.

**What the readers did not reach**, because at about 35 calls a reader takes
the contract sections first:
- **`replacement-architecture.md`:** §3.2c–§3.2d, §3.5, §4.1a–§4.4, §5–§5e,
  §8, §10 and §11.
- **`layers-architecture.md`:** §13a–§13b and most of §12's records.
- **`cant-effects-architecture.md`:** §6.1–§6.3 and §9, its findings, the
  likeliest home of further "today" claims.
- **`cost-architecture.md`:** §3.11's walkthrough and §7–§12.
- **`triggers-architecture.md`:** §4.9–§6, §9, §10, §13, §15 and §16, past
  a name-existence grep over §4.9–§10. Pass 1's readers covered §13, §15
  and §16; pass 2 moves §0–§2.1, §8, §11.1, §12's re-counts, §14 and §17.
- **`mana-architecture.md`:** §2, §3.3–§3.6 and §3.9–§9.
- **`engine-map.md`:** §1.2's replacement, copy and layers steps (read in
  #275).
- **The contract readers:** each grep hit read only as a line, past its
  budget, as their logs list. Most such hits were dated records.

These sections are this pass's to read before its first PR closes. The
discovery logs name them, with line ranges, in the workflow's journal.

**The survivors.** Paths are under `mtgsim/src/`. "The doc says" is the
doc's own words or a close paraphrase, and the correction is the text as it
would be rewritten in place. A `CLAUDE.md` correction must not add a line
(the 200-line budget): each one stays on its line. Many rows here are a
stale count or pointer. A correction follows
`engineering-practices.md#counts-in-docs` (added in #271): it drops the number, or describes the thing, rather
than writing today's value in.

**`CLAUDE.md`, and `engine-map.md` §2.7** (6)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| CLAUDE.md §The layer-system invariant (37-38) | "**Never read `card_data.{types,subtypes,supertypes,colors,keyword_flags,abilities}` for an object on the battlefield or the stack** — route through `oracle/characteristics.rs`" | engine/layers/compute.rs:357 (compute_non_member); engine/layers/board.rs:1882 (left_out_zones); state/game_state.rs:2088 (CDA applied "in every zone (CR 604.3)"); plans/check_invariants.py:75-76 (DIRECT_READ has no zone filter) | "**Never read `card_data.{types,subtypes,supertypes,colors,keyword_flags,abilities}` for an object in any zone** — route through `oracle/characteristics.rs`, and assume a new query needs a wrapper." Shorter than the current line, so it fits the budget. engine-map.md §2.7 row 1 should read "Characteristics of an object in any zone". |
| CLAUDE.md §Registry membership is not effect existence (56) | "The frame cache's descending layer ceiling is the termination argument, not an optimization. → `layers-architecture.md` §5.2." | engine/layers/board.rs:107-111 (Board::sub keyed (ObjectId, ceiling)), :412 (Board::frame_of); no FrameCache anywhere in src; plans/layers-architecture.md:669-671 ("Until LI-1 (2026-09-06) the argument was a descending layer ceiling") | "**Nothing inside the layer pass calls the top-level entry**: a member's frame is read off the live board, one application per iteration, and only a non-member's walk recurses, at a strictly lower ceiling (`Board::frame_of`). That is the termination argument, not an optimization. → `layers-architecture.md` §5.2." |
| CLAUDE.md §The chokepoint invariant, first bullet (75-76) | "A `ZoneChange` names its `ZoneChangeAct` and every proposal its `EventCause`; no catchall. Only the pipeline and the trigger matcher branch on `act`; every emitter names its record's cause." | engine/resolve.rs:2619-2623 (AmountExpr::DiscardedThisWay filters on act Discarded); engine/triggers/history.rs:260-263 (history writer); the rule is restated at types/zones.rs:217-218 | "Only the pipeline, the trigger matcher, the history writer and a resolution counting its own records (\"discarded this way\") branch on `act`; every emitter names its record's cause." Keep "branch on", not "read" (ui/display.rs prints act). No line added, so the 200-line budget holds; types/zones.rs:217-219 and replacement-architecture.md:1070-1072 change in step. |
| CLAUDE.md §The chokepoint invariant (88) | "One permanent chokepoint exemption, `// CAST-ROLLBACK:`: CR 601.2a's move is silent both ways, loud at 601.2i." | plans/check_invariants.py:70 (`"write": ("CAST-ROLLBACK", "NOT REPLACEABLE")`), :390; `// NOT REPLACEABLE:` sites engine/keywords.rs:45, engine/layer_settlement.rs:68, engine/resolve.rs:1519, :1532, engine/sba.rs:203, engine/triggers/control.rs:160, engine/turns.rs:369, :417, types/replacement.rs:1629, :1633 | "Two permanent write tags: `// CAST-ROLLBACK:` (CR 601.2a's move, silent both ways, loud at 601.2i) and `// NOT REPLACEABLE:` (a write no replacement effect can watch for, CR 614.1). → `replacement-architecture.md` §2." Same line, wider; no line is added. Same claim also at replacement-architecture.md:130 and cost-architecture.md:875. |
| CLAUDE.md §The replacement pipeline (CR 614–616) (102) | "Two gates, three legs each — printed, granted, copied; Layer 3 is the route left." | engine/replacement/gather.rs:464; engine/restriction/predicate.rs:112-116 (is_prohibited); engine/cost_determination/gather.rs:59-63 (cost_modifications_for); engine/triggers/dispatch.rs:14 (zone_trigger_sources' granted and copied legs), :1511 (trigger-multiplier probe) | "Five gates — replacement, restriction, cost modification, trigger dispatch, CR 603.2d's trigger-multiplier count — three legs each: printed, granted, copied; Layer 3 is the route left." (The other refuter's alternative: "One gate per reader of that list (`engine-map.md` §2.3), three legs each — printed, granted, copied; Layer 3 is the route left.") |
| CLAUDE.md §Determinism at the decision boundary (124-125) | "`::new()` and `reseed_from_entropy` are the deliberate opt-outs, for interactive play, each under `#[expect]`." | bin/fuzz_games.rs:1142-1144 (`#[expect(clippy::disallowed_methods, ...)]` over `rand::rng().random::<u64>()`, the master seed of a run with no --seed); clippy.toml:7-8 | "`::new()` and `reseed_from_entropy`, for interactive play, and an unseeded `fuzz_games` run's seed are the deliberate opt-outs, each under `#[expect]`." Line 125 goes from 74 to about 125 characters; no line is added. |

**`engine-map.md`** (12)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| engine-map §1.1 The path in one sketch (37) | `RIDERS    phase 3 → resolve_rider (CR 615.5)    [replacement]`, sketched before SETTLE and DETECT, as a phase of the batch | engine/actions.rs:895-896 (execute_batch: dispatch_batch, then execute_riders); engine/actions.rs:1268 (execute_riders); engine/actions.rs:1375 (resolve_riders); engine/triggers/dispatch.rs:654-656 (check_state_triggers ends the dispatch); engine/actions.rs:1016-1017 (CR 614.13's batch) | Move the line below STATE, above PLACE: "RIDERS    after the dispatch → execute_riders: the riders as a batch of their own, dispatched at its close (CR 615.5, 603.4), for any batch that dispatches its own window (the outermost, or CR 614.13's auxiliary); a batch that joined an enclosing window runs them in resolve_riders, inside it   [replacement]" |
| engine-map §1.2 The walk, step 12 (82) | "12 \| Phase 3: no rider was queued \| replacement \| `resolve_rider` `engine/actions.rs:1082`": riders resolve in a batch phase 3, ahead of step 13's dispatch | engine/actions.rs:1048 (execute_batch_inner: two phases, returns its riders); engine/actions.rs:1268 (execute_riders), called at engine/actions.rs:896 after dispatch_batch at :895; engine/triggers/dispatch.rs:716, 727 (steps 14-15 run inside dispatch_batch) | Move the step after step 15, before 16, and renumber: "\| 16 \| No rider was queued, so once the window is dispatched (steps 13-15), `execute_riders` returns at once and opens no batch of its own \| replacement \| `execute_riders` `engine/actions.rs`, called after `dispatch_batch` in `execute_actions` \| `replacement-architecture.md` §4.1a \|". No line numbers unless the header is re-pinned. |
| engine-map §1.2 The walk, step 13 (83) | "The outermost close dispatches the window: histories first, then the gate reads `trigger_sources` ...", located at `dispatch_after_batch` `engine/actions.rs:682`, `dispatch_batch` ... | engine/actions.rs:895 (execute_batch calls dispatch_batch directly); engine/actions.rs:903, 976 (dispatch_after_batch, called only from execute_actions_decomposing) | "\| `execute_batch`'s close `engine/actions.rs:895`, `dispatch_batch` `engine/triggers/dispatch.rs:574`, `advance_history` `engine/triggers/history.rs:75`, `detect` `engine/triggers/dispatch.rs:735`, `battlefield_readers` `engine/triggers/dispatch.rs:922` \|". These are today's lines: re-pin the header with the fix, or keep the pin-era numbers and change only the name. |
| engine-map §1.3 The seams, S12 (106) | S12's Where names `dispatch_after_batch` `engine/actions.rs:682` for the close where "a batch that queued riders dispatches what it performed before them", the riders then their own batch (`execute_riders`) | engine/actions.rs:895-896 (execute_batch: dispatch_batch, then execute_riders); engine/actions.rs:976 (execute_actions_decomposing, dispatch_after_batch's only caller) | Where: `execute_batch` `engine/actions.rs:840` (the window dispatched at `engine/actions.rs:895`), then `execute_riders` `engine/actions.rs:1268`. Not `execute_batch` at :895: check_engine_map.py looks for the name within two lines of the cited one, and :895 holds dispatch_batch, so --drift would list it. |
| engine-map §2.1 The modules and what each owns, state row (148) | "`state/layer_memo.rs`, `state/history.rs`, `state/diagnostics.rs`, `state/trace.rs` \| the layer memo; the game's bounded history, each player's rows in it; ..."; neither `state/history_log.rs` nor `state/shared_stores.rs` is named | state/history.rs:1-11 (GameHistory at :120: the log, each PlayerHistory, the per-ability "this turn" sets); state/history_log.rs:1-10 (:7 "Nothing truncates it"); state/shared_stores.rs:1 | First column adds `state/history_log.rs`; the cell becomes "the layer memo; `GameHistory`: the history log (one row per event a card can ask about, kept for the whole game in `Arc`-chained chunks), each player's `PlayerHistory` beside it (own turns, draw-step draws, the newest turn's tally), and each ability's "this turn" facts; seed-deterministic work counters; the trace sink and `emit_event`". Add a row: `state/shared_stores.rs` \| `ObjectTable` and `SharedList`, the chunked stores a fork shares (objects, battlefield, stack, stack entries, waiting triggers, memo frames). |
| engine-map §2.1 The modules and what each owns, ui row (155) | "`ui/decision.rs`, `ui/ask.rs`, `ui/choice_types.rs` \| the four-method `DecisionProvider`; ..." | ui/decision.rs:162 (pub trait DecisionProvider); ui/decision.rs:212 (seat_mode, defaulted) | "the five-method `DecisionProvider` (`pick_n`, `pick_number`, `allocate`, `choose_ordering`, and the defaulted `seat_mode`); ..." |
| engine-map §2.2 The chokepoint's arms (199) | "Three doors sit beside `execute_action` `engine/actions.rs:598`: `execute_actions` for a simultaneous event, `execute_actions_decomposing` ... and `execute_actions_new_batch` ... for `// AUXILIARY-MOVE:` (CR 614.13)." | engine/actions.rs:831 (execute_caused_actions); its callers engine/sba.rs:540, engine/returns.rs:90 | "Four doors sit beside `execute_action` `engine/actions.rs:598`: `execute_actions` for a simultaneous event, `execute_caused_actions` `engine/actions.rs:831` for one whose members carry causes of their own (CR 704.3's state-based actions, CR 610.3d's returns), `execute_actions_decomposing` `engine/actions.rs:723` for CR 121.2's draws, and `execute_actions_new_batch` `engine/actions.rs:765` for `// AUXILIARY-MOVE:` (CR 614.13)." |
| engine-map §2.3 The gates, and the three legs a new source extends (209-222) | Each reader skips its sweep behind a gate with three legs, and "a new route onto the list needs a leg on every gate"; the table lists five gates: replacement, restriction, cost, trigger, state trigger | engine/triggers/dispatch.rs:1509 (trigger_multiplier_sources_now); state/game_state.rs:608 (trigger_multiplier_sources, written at :2083); state/continuous_effects.rs:185-186 (RegistryScopeSummary's any_granted_trigger_multiplier, any_copied_trigger_multiplier) | Add a row: "\| trigger multiplier (CR 603.2d) \| `count_additional_triggers` `engine/triggers/dispatch.rs:1529`, over `trigger_multiplier_candidates` `engine/triggers/dispatch.rs:1480`; the gate at `trigger_multiplier_sources_now` `engine/triggers/dispatch.rs:1509` \| `trigger_multiplier_sources` \| `any_granted_trigger_multiplier`, `any_copied_trigger_multiplier`; a departure's CR 603.10a frame and a look-back snapshot are two more probes \|" |
| engine-map §2.3 The gates, trigger row (221) | "\| trigger \| `detect` ...; the gate at `engine/triggers/dispatch.rs:446` \| `trigger_sources`, `zone_trigger_sources` \| ...": two printed stores | state/game_state.rs:756 (newcomer_trigger_sources), written at state/game_state.rs:2030 in register_static_effects (state/game_state.rs:1984); probed at engine/triggers/dispatch.rs:748 via newcomer_trigger_carriers (engine/triggers/dispatch.rs:975) | Printed: `trigger_sources`, `zone_trigger_sources`, `newcomer_trigger_sources` (off the battlefield, a printed trigger whose every event arm is the object itself moving, `TriggerDef::triggers_only_on_this_object_moving`; asked only of the window that brought the object there, via `newcomer_trigger_carriers`). Also: `detect` is at `engine/triggers/dispatch.rs:735` and its gate at `:741-766`, not 430/446. |
| engine-map §2.7 Every CLAUDE.md invariant, where it lives, rows for characteristics, no observable mutation, observable order, randomness (291-316) | The Lives-at column names what each rule protects (`perform_action` `engine/actions.rs:1153`, `battlefield_ordered`, `GameState::rng`, `tests/determinism_test.rs`), not the gates CLAUDE.md says enforce them | plans/check_invariants.py:67-71 (TAGS printed/unordered/write), :384-391 (advice per check); clippy.toml:9-24 (disallowed-methods/types) | Only the characteristics row needs a fix: its tag list gains `// NO ROW REACHES:` (`prints_cost_ability` `engine/cost_determination/gather.rs:208`), the tags `plans/check_invariants.py`'s printed check accepts (the line-295 fix). The other three rows are true; to cite gates, add `engineering-practices.md#invariant-gates` to their Doc column, never `PERFORMERS` in Lives-at (check_engine_map.py:162-165 fails it). |
| engine-map §2.7 Every CLAUDE.md invariant, where it lives (295) | Characteristics row: "exemptions tagged `// PRE-LAYER ZONE:` and `// AS PRINTED:` (`printed_faces` `ui/display.rs:160`), and `register_static_effects` ...", naming two tags | engine/cost_determination/gather.rs:209, engine/triggers/audit.rs:155 (`// NO ROW REACHES:`) | "exemptions tagged `// PRE-LAYER ZONE:`, `// AS PRINTED:` (`printed_faces` `ui/display.rs:160`) and `// NO ROW REACHES:` (`prints_cost_ability` `engine/cost_determination/gather.rs:208`, `objects_that_may_trigger` `engine/triggers/audit.rs:154`), and `register_static_effects` `state/game_state.rs:1583`", keeping the name-plus-file:line form that check_engine_map.py's NAMED pattern checks. |
| engine-map §2.7 Every CLAUDE.md invariant, where it lives (312) | "\| Riders run after the performed event, carrying its applied set \| phase 3 `engine/actions.rs:1032`; `Rider::lineage` `engine/replacement/pipeline.rs:76` \|" | engine/actions.rs:1048 (execute_batch_inner: no phase 3); engine/actions.rs:1268 (execute_riders, called at :896, :1017); engine/actions.rs:1375 (resolve_riders, called at :872, :966); engine/replacement/pipeline.rs:81 (Rider::lineage) | "\| Riders run after the performed event, carrying its applied set \| `execute_riders` `engine/actions.rs:1268` (a batch that dispatches its own window: its riders as a batch of their own, after that dispatch) and `resolve_riders` `engine/actions.rs:1375` (a nested or decomposed batch's, inside the window it joined); `Rider::lineage` `engine/replacement/pipeline.rs:81` \| `replacement-architecture.md` §4.1a \|". These are today's lines, so re-pin the header with this row. |

**`layers-architecture.md`** (21)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| layers §3.2 `EffectiveCharacteristics` (95-148) | Keywords are `AbilityDef::Keyword(KeywordAbility)` variants, with no separate `keywords` set; fields include `abilities: Vec<AbilityDef>`, `loyalty: Option<u32>`, `defense`, `text`, `face_down: bool` | engine/layers/types.rs:318-338 (struct EffectiveCharacteristics); card_data.rs:112 (AbilityDef is a struct, not an enum) | Fields: name, mana_cost, colors: HashSet<Color>, types: CardTypes, subtypes: Subtypes, supertypes: HashSet<Supertype>, keyword_flags: HashSet<KeywordFlag> (CR 702 presence-only keywords, the second Layer 6 channel), abilities: Arc<Vec<AbilityDef>>, power/toughness/loyalty: Option<i32>, controller, control_since_turn. color_indicator, defense, text and face_down are not built. |
| layers §3.3 `ContinuousEffect` (154-213) | Fields include `affected: ObjectSet` and a stored `filter_reads: HashSet<CharacteristicCategory>`, "pre-computed at effect creation"; `Duration::WhileSourceActive` effects are removed | engine/layers/types.rs:267-289 (struct ContinuousEffect); engine/layers/board.rs:745 (filter_reads), :946 (effect_channels); types/effects.rs:538 (Duration, no WhileSourceActive) | Fields: id, source, origin: EffectOrigin, layer, duration, controller (CR 611.2c; one refuter says the cite is wrong, since controller is what resolves "your" in UntilYourNextTurn), created_on_turn, timestamp, affected_objects: ObjectSet, modification. Reads are not stored: board::effect_channels derives `Reads { source, members }` per layer onto `Application.reads` for the CR 613.8 check. Drop line 213's "What *is* stored is `filter_reads`". Rows end through ContinuousEffect::ends_with_its_source: every StaticAbility row (CR 611.3b), or a Duration::ends_with_its_source row (CR 611.2b). |
| layers §3.4 "One leaf table for both askers", opening paragraph and asker table (254-263) | "Two functions answer each `ObjectFilter` case for one object, and two more answer 'does this `PlayerRef` name player p'"; the table lists `targeting::object_matches_filter_with` returning `Result<bool, FilterRefusal>` | engine/targeting.rs:669-691 (GameState::object_matches_filter -> bool, calls object_filter::evaluate); compute.rs:834; engine/object_filter.rs:157 (FilterRefusal is private) | Before #250, two functions answered each `ObjectFilter` case and two more answered the `PlayerRef` question. The table is that starting point; `object_matches_filter_with` and its `Result<bool, FilterRefusal>` are gone. Today `compute::object_matches_filter` and `GameState::object_matches_filter` both call `object_filter::evaluate` and return `bool`, and `dispatch::player_ref_is` and `SourceFilter::matches` both ask `object_filter::player_ref_names`. |
| layers §3.4 "One leaf table for both askers", The shape (265-270) | `engine::object_filter::evaluate(game, filter, id, frame, referents) -> bool`; FilterReferents answers `you()`, `source()`, `source_owner()`, `earlier_targets()`; one predicate, `player_ref_names(who, player, referents)` | engine/object_filter.rs:215-222 (evaluate; one reader cites :221-228), :34-56 (trait FilterReferents), :361-366 (player_ref_names) | `evaluate(game, filter, id, frame, last_known, referents) -> bool`, with `frame: &impl Fn() -> Option<&EffectiveCharacteristics>` and `last_known: Option<&Lki>`, which gives the owner (CR 108.3) and counters (CR 122.1) once the store has lost the object. Referents: `you()`, `source()`, `source_owner()`, `earlier_targets()`, and since TR-5e `defending_player(game, creature)` (CR 508.5). Two defaulted hooks: `counters_on` (the walk overrides it to read the entity it is building) and `note_gated_read` (used only by the walk). `player_ref_names(game, who, player, referents) -> Option<bool>` returns None when `who` names the source's owner and there is no source. |
| layers §3.5 `EffectModification` + `CharacteristicCategory` (296-402) | Variants include `TurnFaceDown`, `CopyFrom { copiable: CopiableValues }`, `SetText(String)`, `GrantKeyword`/`RemoveKeyword(KeywordFlag)`, `GrantAbility(Box<AbilityDef>)`, `ModifyPowerToughness { power: i32, toughness: i32 }`; plus `enum CharacteristicCategory` and `Layer::category()` | engine/layers/types.rs:126-150 (enum EffectModification); engine/layers/board.rs:642 (writes_of into Channels) | As built (types.rs:126-203): CopyFrom(Arc<CopiableValues>) (Layer 1a, CR 613.2a); SetController(PlayerRef); Add/RemoveType(CardType), SetTypes(CardTypes); Add/RemoveSubtype(Subtype), SetSubtypes(Subtypes); Add/RemoveSupertype(Supertype), SetSupertypes(HashSet<Supertype>); AddColor, SetColors, RemoveAllColors; GrantKeywordFlag/RemoveKeywordFlag; GrantAbility(Arc<AbilityDef>); LoseAbility(AbilityId); LoseAllAbilities; Set/ModifyPowerToughness { power: PtValue, toughness: PtValue }; SwitchPowerToughness. There is no face-down variant: CV-6 builds 1b from PermanentState.face_down with no row (copy-effects §4.6). There is no SetText, since Layer 3 is not modeled. CharacteristicCategory and Layer::category() do not exist; the static check is board::writes_of into Channels (board.rs:642). |
| layers §3.6 `Duration` (481-516) | `Duration` is WhileSourceActive, UntilEndOfTurn, UntilYourNextTurn, UntilEndOfCombat, WhileCondition(ExpirationPredicate), UntilTargetLeavesBattlefield(ObjectId) | types/effects.rs:538-545 (enum Duration) | pub enum Duration { UntilEndOfTurn, UntilYourNextTurn, WhileSourceOnBattlefield, WhileEnchanted, WhileEquipped, Indefinite } (types/effects.rs). UntilEndOfCombat, WhileCondition, UntilTargetLeavesBattlefield and ExpirationPredicate are not built. |
| layers §3.7 `ContinuousEffectRegistry` (518-540) | `struct ContinuousEffectRegistry { effects: Vec<ContinuousEffect>, next_effect_id: EffectId }`; `remove(&mut self, id: EffectId) -> Option<ContinuousEffect>` | state/continuous_effects.rs:402-405, :521, :526 | `{ effects: DurationRegistry<ContinuousEffect>, summary: RegistryScopeSummary }`. The Vec, the id counter and the CR 514.2 expiry hooks live in DurationRegistry (state/duration_registry.rs). `add` assigns the id; `next_id()` reads the next one without advancing it. Every write goes through `mutating`, which rebuilds the summary. Rows are Arc<ContinuousEffect>, kept in (layer, timestamp, id) order and served by effects_in_layer. Removal: remove(id) -> Option<Arc<_>>, remove_by_source (called only from tests), remove_rows_ending_with, remove_static_by_source, remove_references_to (CR 400.7), retire_earlier_copies_of (CR 707.4), remove_expired_at_cleanup and remove_expired_at_turn_start. retime_static_rows (CR 613.7a/e) edits rows in place. |
| layers §4 Module Layout (544-576) | Lists `dependency.rs` (hybrid dependency algorithm), `lifecycle.rs` (register_effect / deregister_effect helpers), and `types.rs` holding Duration, CharacteristicCategory, ObjectSet, Timestamp, CopiableValues, AbilityOrigin | engine/layers/mod.rs:12-21; types/effects.rs:327, :538; engine/layers/copy.rs:67; types/ids.rs:32 | engine/layers/: types.rs (Layer, EffectId, PtValue, EffectModification, EffectOrigin, EffectGroup, ContinuousEffect, EffectiveCharacteristics; it re-exports ObjectSet, defined at types/effects.rs:327, and Timestamp, defined at types/ids.rs:32; Duration is defined at types/effects.rs:538 and imported, not re-exported). board.rs: the board pass and CR 613.8 (resolve_order_within_layer, depends_on over Channels, next_ready). compute.rs: compute_characteristics, the memo entry, compute_non_member. Also cda.rs, condition.rs, copy.rs (CopiableValues), explain.rs, intrinsic.rs, land_types.rs, lookahead.rs. There is no AbilityOrigin. Registration: register_static_effects (game_state.rs:1984), register_copied_static_effects (:2152), resolve.rs register_granted_static_effects (:1817), and continuous_effects.add. Removal goes through ContinuousEffectRegistry's own methods. |
| layers §5.1 Zone scope — hidden-zone fast path (632, 641-642) | The struct sketch gives reachable_zones "plus eight gate flags added by Layer 2, RS-1, CV-1, CM-1 and RE-8" (line 632's "carried eight fields by RE-8" is a dated record and still holds) | state/continuous_effects.rs:68-254 (struct RegistryScopeSummary) | Fix line 641 only; line 632 stays. New text: "// ...plus fifteen more fields: the eight gate flags that stood at RE-8 (RF has since folded the replacement pair into `unattributed_replacement_zones` and `any_named_unattributed_replacement`), the name-writing flag (#203), and six from the trigger track (TR-1's `unattributed_trigger_zones`, item 167's `ability_list_sources`, TR-3a's and TR-6's event-kind masks, TR-5g's trigger-multiplier pair); `state/continuous_effects.rs` is the list". |
| layers §7 Layer 1 Sublayers (755, 766) | "`EffectModification::CopyFrom(Box<CopiableValues>)`, the phase's one new arm"; the face-down note says that in Layer 1a `compute_characteristics` reads the flag and synthesizes face-down characteristics, and that `TurnFaceDown` effects flip the flag | engine/layers/types.rs:127 (CopyFrom(Arc<...CopiableValues>)); state/battlefield.rs:68 (face_down field, read by nothing in engine/layers) | `EffectModification::CopyFrom(Arc<CopiableValues>)`. Face-down is unbuilt: PermanentState.face_down exists, but no layer reads it and no TurnFaceDown arm exists. CV-6 will make 1b derive CR 708.2a's characteristics from the flag. |
| layers §8 Timestamps, item 1 (774) | "On ETB — `PermanentState.timestamp = game.next_timestamp()`"; "the field is already populated (`state/battlefield.rs:80-84`); Phase LA starts *reading* it" | state/game_state.rs:2510 (obj.timestamp = self.allocate_timestamp()), :1453-1454 (insert_battlefield_entity stamps from the object), :1469-1474 (set_object_timestamp writes both) | On entering any zone (CR 613.7d), GameObject.timestamp is allocated from GameState.next_timestamp (move_object, plus token and setup creation). The battlefield entry's PermanentState.timestamp is a copy that insert_battlefield_entity stamps from the object, kept for the ordered sweeps. static_effect_timestamp reads the object's value (§13d decision 2). |
| layers §10 Where effects get added (853); §13d decision 3's table (2536-2538) | "`GameState::place_on_battlefield` → `register_static_effects`"; the table gives `register_static_effects(id, controller, zone)`, one function with two callers: A `place_on_battlefield`, B `move_object`'s tail | state/game_state.rs:1666-1678 (make_permanent), :1984 (register_static_effects, 4 parameters); engine/zones.rs:205-209 (register_off_the_battlefield) | §10: `GameState::make_permanent` → `register_static_effects(id, controller, Zone::Battlefield, arrived_on)`, reached from both `perform_entry` and `create_on_battlefield`. Every other zone registers through `engine/zones.rs::register_off_the_battlefield`, with the owner as "you" (CR 108.4); its callers are `arrive_in_zone` (`move_object`'s tail) and `create_in_zone`. §13d table: `(id, controller, zone, arrived_on)`; caller A is `make_permanent`, caller B is `register_off_the_battlefield`. Drop the emblem clause, because nothing creates an emblem. |
| layers §10 Where effects get removed (862-871) | Expiry runs in `engine/turns.rs::cleanup_step` (UntilEndOfTurn), `turns.rs::begin_untap` (UntilYourNextTurn) and `combat/steps.rs::end_of_combat` (UntilEndOfCombat); the SBA loop handles WhileCondition; there is an UntilTargetLeavesBattlefield row | engine/turns.rs:341-343 (expire_until_your_next_turn), :384/:428 (on_step_begin → remove_expired_at_cleanup) | Source leaves the battlefield → zones.rs::cleanup_zone_state → remove_rows_ending_with. Source leaves another zone → remove_static_by_source. CR 514.2 cleanup → turns.rs on_step_begin → remove_expired_at_cleanup. Start of that player's turn (UntilYourNextTurn) → turns.rs expire_until_your_next_turn → remove_expired_at_turn_start. No end-of-combat, condition or target-leaves durations exist. |
| layers §10 Invariant (871); §15.1 "Emblem / commander-source effects" (file says 762; a refuter puts it at 2762) | The registry "should never contain effects whose source has left its functional zone"; `remove_by_source` "runs on every zone change out of battlefield"; §15.1 says it "handles this" with `zone_of_function = {Command, Battlefield}` | engine/zones.rs:520 (remove_rows_ending_with), :573 (remove_static_by_source); state/continuous_effects.rs:526 (remove_by_source, called only from tests) | A resolution's rows outlive their source (CR 611.2a). Only rows whose duration ends with their source are removed: through remove_rows_ending_with when the source leaves the battlefield, and remove_static_by_source (static rows only) when it leaves another zone. remove_by_source has no production caller. Functioning zones come from engine/zone_function.rs::functioning_zones, not a zone_of_function field. |
| layers §12, 7a — epoch memoization, "A row's condition reads more than the row" (1021-1032) | Four too-frequent inputs (life totals, tapped status, combat, histories with the turn number) "are `GatedWalkInput`s" since TR-5f's review; "a filter (`ObjectFilter::AttackedThisTurn`) and a CDA alike" | state/layer_memo.rs:89-107 (enum GatedWalkInput); types/effects.rs:259 (ObjectFilter::ThisTurn(ObjectFact)) | ...are GatedWalkInputs: LifeTotals, TappedStatus, Combat, TurnNumber, CombatCount, and Rows(RowKind), one per history row kind (cast, drew, life, entered, left, dealt damage, attacked, blocked, ...). ... a filter (`ObjectFilter::ThisTurn(ObjectFact)`) and a CDA alike. |
| layers §13d decision 1a (2278-2285) | `functioning_zones(ability: &AbilityDef, types: &HashSet<CardType>) -> ZoneSet`; `functions_in(ability: &AbilityDef, types: &HashSet<CardType>, zone: Zone) -> bool` | engine/zone_function.rs:45, :201 | pub fn functioning_zones(ability: &AbilityDef, types: &CardTypes) -> ZoneSet; pub fn functions_in(ability: &AbilityDef, types: &CardTypes, zone: Zone) -> bool. Change only the parameter type and keep the doc comments. |
| layers §13d decision 4, the 113.6d row (2569); The pieces (2661) | "Already answered by `CostSubject::applies_from_battlefield`; it becomes this predicate's arm and the method delegates"; The pieces: "`CostSubject::applies_from_battlefield` delegates to the predicate" | engine/zone_function.rs:45 (functioning_zones' 113.6d arm); no applies_from_battlefield in src | Row 2569: "Was answered by `CostSubject::applies_from_battlefield`; it became `functioning_zones`' arm, which takes the ability through `as_cost_modification` and returns `ZoneSet::STACK` when `applies_to_its_own_object` holds, else CR 113.6's default. The method was deleted (0146cc75), not left delegating; `applies_to_its_own_object` stays." In The pieces, "Three rows moved" becomes four, and the `CostSubject` row says the method was deleted, with the 113.6d gate in `register_static_effects` replacing its one caller. |
| layers §14 Testing Strategy (2748-2749) | "Unit tests in `compute.rs` and `dependency.rs`"; "Integration tests in `tests/phaseL{A,B,C,D}_integration_test.rs`" | engine/layers/mod.rs:12-21 (no dependency module); mtgsim/tests/phase_lb_integration_test.rs … phase_ll_integration_test.rs | **Unit tests** in `compute.rs` (algorithm correctness on synthetic `CardDataBuilder` objects) and `board.rs` (the CR 613.8 pass, `depends_on`, on real-card boards: Blood Moon, Urborg, Opalescence, Ashaya); `cda.rs`, `condition.rs`, `land_types.rs` and `lookahead.rs` have their own. **Integration tests** in `tests/phase_l{b,c,d,e,f,g,h,i,i2,i3,j,k,l}_integration_test.rs` (there is no `phase_la`) and `tests/layer_aware_queries_test.rs`. |
| layers §15.1 WhileCondition predicate AST — Built 2026-09-06 by LI-3 (2813) | "A new leaf is three edits, not two: the variant, the `holds` arm, and the `condition_reads` arm"; "The Phase LC gate's count is 9 of ~25" | types/effects.rs (enum Condition, 18 variants), :717 (reads_entry_state); engine/zone_function.rs:280 (stated_zones, exhaustive over Condition); engine/layers/condition.rs:93 (holds); engine/layers/board.rs:831 (condition_reads); engine/triggers/binding_reads.rs:353 (condition_reads) | **A new leaf is six edits**: the variant, plus one arm in each of the five exhaustive matches over `Condition`: `holds` (engine/layers/condition.rs), `board::condition_reads`, the trigger binding's `condition_reads` (engine/triggers/binding_reads.rs), `Condition::reads_entry_state` (types/effects.rs) and `stated_zones` (LK decision 1b). The reads arms are the ones to remember: a missing one gives a wrong *order*, not a wrong value. Phase LC gate: 16 leaves of ~25; the enum has 18 variants, including the combinators `All` and `Not`. |
| layers §15.2 Still open, item 1 (2817) | "`compute_characteristics` caching at call-site. No caching for Phase LA ... Frame cache (§5.2) is required for correctness and is the only cache." | state/layer_memo.rs; engine/layers/compute.rs:301 (compute_characteristics_uncached beside the memoized entry), :103-111 (memo served by epoch); state/game_state.rs:1500, :1528 (bump_layer_epoch) | Closed 2026-09-03 (critical-path item 7a, §12). `compute_characteristics` is memoized across calls in `GameState::layer_memo` (state/layer_memo.rs). Each frame is stamped with the layer epoch (`GameState::layer_epoch` plus the registry's mutation count) and is served only at that epoch. A debug build checks each hit against `compute_characteristics_uncached`; under `audit_each_frame_once` it checks each frame once. §5.2's per-pass `(ObjectId, ceiling)` non-member cache is not a cross-call cache: it bounds only the non-member CDA recursion. Since LI-1 the sequential pass is the termination argument. |
| layers §15.2 Still open, item 2 (2819) | "Copy-effect snapshot shape (Layer 1b). `CopiableValues` is referenced as a type in §3.5 but not defined. Needs a concrete struct ... Resolve at Phase LD kickoff." | engine/layers/copy.rs:67 (pub struct CopiableValues) | Closed by CV-1: CopiableValues is defined in engine/layers/copy.rs (copy-effects-architecture.md §3.2 owns its shape), and it is Layer 1a, not 1b. |

**`replacement-architecture.md`** (18)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| replacement §2a The shipped types, `ReplacementDef` row (160; cited as 161 and 162) | "One replacement effect as data, nine fields in declaration order: `pattern` + `affected_objects` + `rewrite` + `then` + `class` + `uses` + `is_regeneration` + `exempt_from_614_5` + `optional`" | types/replacement.rs:51-173 (struct ReplacementDef; `affected_players` :80, `by` :111) | One replacement effect as data, eleven fields in declaration order: `pattern` + `affected_objects` + `affected_players` + `by` (the cause, §2b.7) + `rewrite` + `then` + `class` + `uses` + `is_regeneration` + `exempt_from_614_5` + `optional` |
| replacement §2a The shipped types, `EventPattern` row (161; cited as 163) | "Which events an effect watches. Ships 6 arms: `DealDamage`, `ZoneChange`, `Untap`, `Tap`, `Destroy`, `CounterChange`" | types/replacement.rs:204-587 (enum EventPattern: 20 arms, no `CounterChange`) | Which events an effect watches. 20 arms: `DealDamage`, `ZoneChange`, `Untap`, `Tap`, `Destroy`, `DrawCards`, `DrawCard`, `GainLife`, `LoseLife`, `EnterBattlefield`, `AddCounters`, `RemoveCounters`, `BeginTurn`, `BeginPhase`, `BeginStep`, `Scry`, `PlayerLoses`, `PlayerWins`, `ProduceMana`, `CreateTokens` (RB's `CounterChange` split into `AddCounters`/`RemoveCounters`) |
| replacement §2a The shipped types, `Rewrite` row (163; cited as 163 and 165) | "What the effect does to the event. **Ships 2 arms, not §3.2b's 5**: `Prevent` and `Instead(GameActionTemplate)` \| **closed** — `Amount`, redirection and the rest land with the phase that can apply them" | types/replacement.rs:854-1018 (enum Rewrite, 8 arms: `Prevent` :860, `Instead` :869, `EnterWith` :889, `EnterAfterMoving` :926, `EnterUnderControlOf` :949, `EnterAsCopy` :966, `Amount` :987, `Retarget` :1017); :1888 (`ReplacementClass::from_rewrite` matches all eight) | What the effect does to the event. Eight arms: `Prevent`, `Instead(GameActionTemplate)`, `Amount`, `Retarget` (CR 614.9), `EnterWith`, `EnterAfterMoving` (CR 614.13), `EnterUnderControlOf` (CR 616.1b), `EnterAsCopy` (CR 616.1c) \| **closed** — a new arm needs the CR rule that permits it (§3.2b). Or mark the whole table RB-dated and drop its RH-era rows |
| replacement §2a The shipped types, `GameActionTemplate` row (164) | "**`ZoneChangeTo` names only a destination** and keeps the event's `cause`, since a redirected act is still that act (...): `cause` is the act, `to` where it went." | engine/replacement/pipeline.rs:2626-2632 (substitute keeps `act`); engine/actions.rs:246 (`act: ZoneChangeAct` on `GameAction::ZoneChange`); types/replacement.rs `GameActionTemplate` (8 arms) | **`ZoneChangeTo` names only a destination** and keeps the event's `act` (and the proposal's cause), since a redirected act is still that act (CR 614.6, 701.9c, 701.17c, 608.2c's example): `act` is the act, `to` where it went. It carried its own act until 2026-09-24 (`codebase-state.md` item 176) |
| replacement §2a The shipped types, `ReplacementClass` row (167 as cited) | "**closed** — all five ship; only `Other` has a producer" | types/replacement.rs:1888-1891 (`from_rewrite`: `EnterUnderControlOf` → `ControlChanging`, `EnterAsCopy` → `CopyAsEnters`), :1953 (`ReplacementDef::new` calls it); cards/phase_cv_cards.rs:343, :416 (Clone, Spark Double); mtgsim/tests/phase_rc4_integration_test.rs:167 (Defector fixture) | **closed** — all five ship; the class is derived, not authored (`ReplacementClass::from_rewrite`, called by `ReplacementDef::new`): `CopyAsEnters` has printed producers (Clone, Spark Double), `ControlChanging` only a test fixture (`tests/phase_rc4_integration_test.rs`'s Defector), and every other rewrite is `Other`; `SelfReplacement` (§11 item 3) and `BackFaceUp` (CV-5) have none |
| replacement §2a The shipped types, `Uses` row (166; cited as 168) | "`Static` or `Once`. `CounterBacked` did not survive contact with the CR (§3.2) \| **closed** — `Shield(u64)` is CR 615.7's and lands with RD" | types/replacement.rs:1919 (enum Uses), :1943 (`NextDamage(u64)`) | `Static`, `Once` or `NextDamage(u64)` (CR 615.7, landed in RD-2; named `Shield(u64)` in the design, §3.3 item 4). `CounterBacked` did not survive contact with the CR (§3.2) \| **closed** |
| replacement §2a One action, end to end: the phase 1/phase 2 diagram (197 as cited, 189 per a refuter) and 211–213 | "`announce_zone_change` is the one ZoneChange emitter — three callers, each of which performed the move it announces ... ├─ is_blocked? → CR 614.17 \"can't\" wins" | engine/replacement/pipeline.rs:456 (`prohibition(...)`), :469, :484-488 (dropped unless a self-replacement applies); restriction/predicate.rs:96 (`fn is_blocked` removed in RS-1, 68bfdada); the callers: engine/actions.rs:1638 and :1804 via `perform_zone_change` (:2065), engine/put_on_stack.rs:312; actions.rs:2121-2125 (the source's own three-caller count) | Only the diagram line is stale; keep "three callers", optionally spelled out: the `ZoneChange` and `EnterBattlefield` arms, both through `perform_zone_change`, and `cast_spell` at CR 601.2i. The diagram line becomes: ├─ prohibition() → CR 614.17 "can't" wins (101.2); the event is dropped unless a self-replacement applies (614.17c)  restriction/predicate.rs |
| replacement §2b.3 Who or what caused it: the four rule enums (436–454; cited as 444–454) | "Each of the four rule enums has one variant per rule the engine performs, and no catchall"; the `GameRule` list (`OpeningHand` ... `StateBasedCheck` ..., and the records no action makes) names 13 variants | events/cause.rs:186-232 (enum GameRule, 17 variants): :221 `ControlChanges`, :224 `CombatEnds`, :228 `CombatantTypeChanged`, :231 `MandatoryLoop` | After `StateBasedCheck` (704.3's one event, whose members are each their own subrule's) add `ControlChanges` (613.1b: a change of control, announced by a sweep that no performer proposes), `CombatEnds` (511.3's removal from combat), `CombatantTypeChanged` (506.4: an attacker or blocker that stops being a creature), `MandatoryLoop` (104.4b's draw), then the records no action makes (... a triggered ability put on the stack with its targets, 603.3) |
| replacement §3.1 `GameAction` — the replaceable-event vocabulary, the table (1037–1058) | "`ZoneChange { cause }` \| 400.6 \| RA \| `cause` is new and required"; "`Destroy { object, source }` ... performs an inner `ZoneChange{cause: Destroyed}`"; "`AddCounters { target, kind, n, source }`"; "`PlayerLoses { player, reason }`"; four variants have no row | engine/actions.rs:138 (enum GameAction, 26 variants), :242 `ZoneChange { object, from, to, act }`, :307 `AddCounters { subject, counter, n, by }`, :359 `Destroy { object }`, :530 `PlayerLoses { player }`; rowless: :269 `Attach`, :287 `Unattach`, :334 `RemoveCounters`, :459 `TokenCeasesToExist`, :599 `GameEndsInADraw`; :1350, :1202 (`BatchMember::inner_move`); :391 (`EnterBattlefield` with `from`, `act`); :1655, :1713 (Tap/Untap emit) | `ZoneChange { object, from, to, act }` (act caller-set, required; the cause is the proposal's `EventCause`); `Destroy { object }` — outer; its inner `ZoneChange { to: Graveyard, act: Destroyed }` is decided with it and performed right after (`BatchMember::inner_move`), which of 701.8b's ways being the cause; `AddCounters { subject, counter, n, by }` / `RemoveCounters`; `PlayerLoses { player }` (reason folded into the cause, RH-2); add rows for `Attach`, `Unattach`, `TokenCeasesToExist`, `GameEndsInADraw`; fix the 7-variant "Today:" lead-in. Also stale: `EnterBattlefield` (`from`, `act`), `CreateTokens` (`attacking`), Untap/Tap "currently silent", ProduceMana's "silent writers" |
| replacement §3.1 `GameAction` table, `PlayerLoses` row (1056; cited as 1052) | "\| `PlayerLoses { player, reason }` / `PlayerWins { player }` \| 104.2b, 104.3e, 704.5a–c, 704.7 \| RE \|" | engine/actions.rs:530 and :1940 (`GameAction::PlayerLoses { player }`), :1949 (`GameEvent::PlayerLost { player_id: player }`, `ctx.cause`), :586 (`PlayerWins { player }`) | \| `PlayerLoses { player }` / `PlayerWins { player }` \| 104.2b, 104.3e, 704.5a–c, 704.7 \| RE \| added by §8a's audit (2026-08-24); RE-6, where the four SBA loops become batch members; its reason folded into the cause at RH-2 (§2b.11, decision 12) \| |
| replacement §3.1 `GameAction`, the three rules for `act` (1070–1071) | "**Nothing may branch on `act` outside the replacement pipeline and the trigger matcher.** It is not a general-purpose tag; a third reader is a third place for it to drift." | engine/resolve.rs:2619-2623 (`AmountExpr::DiscardedThisWay` filters on `act: ZoneChangeAct::Discarded`); engine/triggers/history.rs:260-263 (skips `ZoneChangeAct::Cast \| ZoneChangeAct::Drawn`) | **Only the replacement pipeline, the trigger matcher, the history (`triggers/history.rs`, which drafts no row for a cast or a draw) and a resolution's own "discarded this way" count (`AmountExpr::DiscardedThisWay` in `engine/resolve.rs`) branch on `act`.** It is not a general-purpose tag; each further reader is another place for it to drift. (Keep "branch on", not "read": display.rs reads `act` to print it.) |
| replacement §3.2a `EventPattern`, the field vocabulary (1192–1194) | "Within an arm, constraints on the event's fields reuse existing vocabulary — `ObjectFilter`, `PlayerRef`, `ZoneChangeCause`, `CardType` — rather than inventing per-mechanic predicates." | types/zones.rs:224 (enum `ZoneChangeAct`), :265, :268 (`ZoneChangePattern` and its `act`); types/replacement.rs:247-251 (`EventPattern::ZoneChange { change: ZoneChangePattern, .. }`), :428-431 ("A PlayerSet and not a PlayerRef"), :91-111 (the cause filter is `ReplacementDef::by`) | Within an arm, constraints on the event's fields reuse existing vocabulary — `ObjectFilter`, `PlayerSet`, `ZoneChangePattern` (its `act: ZoneChangeAct`), `CardType` — rather than inventing per-mechanic predicates. What caused an event is not an arm's field: it is `ReplacementDef::by`, the same `SourceFilter` a "can't" uses. |
| replacement §3.2b `Rewrite` — a closed algebra (1232–1280: the sketch 1238–1250, the closing rule 1278) | The sketch has five arms: `EnterWith(EnterMods)` modifies "(tapped, counters, controller, copy-of)"; `Retarget(RetargetSpec)` is "CR 614.9 redirection, CR 616.1b control-changing"; "A sixth arm is a claim that CR 614/615 permits an operation this list omits" | types/replacement.rs:854-1018 (enum Rewrite, 8 arms: `EnterWith(EnterModsTemplate)` :889, `EnterAfterMoving` :926, `EnterUnderControlOf` :949, `EnterAsCopy(EntryCopyTemplate)` :966, `Retarget` :1017), :958-960 (`EnterAsCopy` "Not an EnterWith either"), :1888-1900 (`from_rewrite` files `Retarget` as `Other`: 616.1b is about entering) | Eight arms: `Prevent`; `Amount`; `Retarget` (CR 614.9 redirection only, class `Other`); `EnterWith(EnterModsTemplate)` (614.1c/d: status, counters, characteristic edits; no controller or copy-of); `EnterAfterMoving` (614.13); `EnterUnderControlOf` (616.1b, `ControlChanging`); `EnterAsCopy` (614.1c/616.1c, `CopyAsEnters`, replacing an earlier copy); `Instead`. "The other four arms" becomes seven; "A ninth arm is a claim that CR 614–616 permits…" |
| replacement §3.5 The entry state, the feeds table's `counters` rows (1624–1627; cited as 1627–1630) | "`counters`, +1/+1 \| with \| power and toughness up, at 7c (CR 122.1a) \| `PowerLE` ... `counters`, any other kind \| with \| nothing \| none" — the "Leaves it can unmatch" column never names `WithCounters` | engine/replacement/pipeline.rs:1690-1700 (`EntryWrites::unmatches`, arm `ObjectFilter::WithCounters { kind, count }`), :1546-1550 (`counter_kinds`) | Every counters row's "Leaves it can unmatch" cell also names `WithCounters` counting that kind or every kind (`kind: None`): `AtMost` directly, `AtLeast` under a `Not`, `Exactly` either way; "any other kind \| with \| nothing \| none" becomes "\| with \| no characteristic \| `WithCounters` as above \|" |
| replacement §3.5 `CharacteristicEdit` — one vocabulary, two placements, the table (1658–1664) | The table lists `GainsAbility(AbilityDef)`, `GainsKeyword(KeywordFlag)` ... `Name(String)`, `ManaCost(Option<ManaCost>)`, `Loyalty(i32)` ("1a; `Name` **built** (CV-2b)") but no `GainsThisAbility`; `Colors`/`ManaCost`/`Loyalty` unmarked | types/effects.rs:1570-1593 (enum CharacteristicEdit), :1586 (`GainsThisAbility`); cards/phase_cv_cards.rs:477 (Cryptoplasm); engine/resolve.rs:2043-2046 (`with_this_ability`); copy.rs:176 (refused elsewhere) | Add a row: `GainsThisAbility` \| none: refused at every entry (`has_entry_placement`; `CopiableValues::except`), since only an ability's resolution has an ability to name \| 1a for an ability's resolution copy only (CV-1b, Cryptoplasm): `with_this_ability` makes it a `GainsAbility` of the resolving def at the capture. Mark `Colors`, `ManaCost` and `Loyalty` as arms not yet in the enum |
| replacement §4.1 The CR 616.1 loop, the pseudocode (1689–1715; 1696, 1712–1714) | "applied: HashSet<ReplacementInstanceId> = inherited # 3.2d lineage ... if chosen.optional && !ask_apply(dp, chooser, chosen): ... chosen.consume_use(game) # Uses::Once (Shield: RD)" | engine/replacement/pipeline.rs:401 (`applied: AppliedReplacements`), :405 (`declined`, unchanged), :571 (`ask_take_or_leave`), :1968-2001 (`consume_use`); engine/replacement/instance.rs:72-77 (`AppliedReplacements`, an ordered Vec); ui/ask.rs:912; gather.rs:176 (`chooser_for`); types/replacement.rs:1943 (`NextDamage`) | `applied: AppliedReplacements = inherited  # ordered, CR 614.5; kept on the record (§2b.4)`; `chooser = chooser_for(game, ev)`; `if chosen.optional && !chosen.rewrite.asks_its_own_may() && !ask_take_or_leave(..)`: `applied.decline(chosen.instance)`, `declined.insert(..)`, `continue` (no `consume_use`); `if !chosen.exempt_from_614_5: applied.apply(..)`; `consume_use(game, chosen, outcome)  # Uses::Once only if it took effect; Uses::NextDamage by the amount prevented (CR 615.7)` |
| replacement §6 Engine interaction points, the `ActionContext` plumbing (2476–2485) | "pub struct ActionContext<'a> { pub dp: &'a dyn DecisionProvider, /// The resolution this action belongs to, if any. ... pub resolution: Option<&'a ResolutionContext>, }" | engine/actions.rs:62-65 (`struct ActionContext { dp, cause: EventCause }`); engine/resolve.rs:139 (`resolving()` builds the cause from `resolution.cause()`) | pub struct ActionContext<'a> { pub dp: &'a dyn DecisionProvider, /// What caused the proposal (§2b.3); every emitter stamps it. pub cause: EventCause, } — the resolution reaches it as `EventCause::Effect`'s `ResolutionStamp` (RH-1) |
| replacement §7 DecisionProvider surface (2502–2505; 2506 per a refuter) | In `ui/choice_types.rs`, CR 616.1's prompt is "`ChooseReplacementEffect { affected: ReplacementSubject, event: EventSummary }`" | ui/choice_types.rs:129 (`ChooseReplacementEffect { affected_object: Option<ObjectId> }`); ui/ask.rs:1077 (built there); no `ReplacementSubject` or `EventSummary` in src | `ChooseReplacementEffect { affected_object: Option<ObjectId> },  // None when the event is about the choosing player rather than an object` |

**`cant-effects-architecture.md`** (13)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| cant-effects §1 Why now, point 3 (135-137) | "`Hexproof`, `Shroud`, `Menace` and `Intimidate` are constructible and grantable — and two of them are printed by `ui/display.rs`" | ui/display.rs:134,136,138,140 (keyword_name names all four) | "... constructible and grantable — and all four are printed by `ui/display.rs::keyword_name` — but no code reads any of them outside `display.rs`." |
| cant-effects §2.3 What the engine enforces today (229-240) | Indestructible is checked at "`is_blocked`, `pipeline.rs:53`" (✅ one hardcoded arm); can't be regenerated at "`push_if_applicable`, `gather.rs:228`", "✅ via a `GameState` `HashSet`" | engine/restriction/predicate.rs:337-358 (keyword_restrictions synthesizes indestructible); engine/replacement/gather.rs:637,650 (push_if_applicable asks is_prohibited with Query::ApplyReplacement) | Mark the table as the authoring-time picture (2026-08-27), or update the rows: indestructible is a keyword-derived `Restriction::Event` built in `engine/restriction/predicate.rs::keyword_restrictions`; can't-be-regenerated is a registered `Restriction::ApplyReplacement` asked at `gather.rs::push_if_applicable`. Neither `is_blocked` nor the `HashSet` exists (RS-1, 2026-08-31). |
| cant-effects §2.3 What the engine enforces today (234-236) | Defender at "`validate_attackers`, `validation.rs:187`" (✅ hardcoded); flying vs. non-reach blocker at "`can_block`, `validation.rs:313`" (✅ hardcoded); Hexproof, Shroud at "`validate_targets`, `targeting.rs:39`" | oracle/legality.rs:45 (can_attack's Defender check); engine/combat/validation.rs:279 (can_block's flying check); engine/targeting.rs:384 (validate_targets), :437 (the hexproof/shroud comment) | Rows read: Defender — `oracle/legality.rs::can_attack` (called by `validate_attackers`), ✅ hardcoded; Flying vs. non-reach blocker — `can_block`, `engine/combat/validation.rs`, ✅ hardcoded; Hexproof, Shroud — `targeting.rs::validate_targets`, ❌ flag exists, comment names RS-2. |
| cant-effects §3.3 `Restriction` — the arms (408-458); also §4.1 Change (688-690) | `Event { pattern: EventPattern, affected: ObjectSet, by: Option<SourceFilter> }`; `ApplyReplacement { kind, to: ObjectSet, to_players: PlayerSet }`; §4.1's indestructible `Restriction::Event { pattern: EventPattern::Destroy { source: None }, affected: ObjectSet::SourceOnly }` | types/restriction.rs:86-106 (Event { pattern, affected_objects, affected_players, by }), :124-144 (ApplyReplacement { kind, to_objects, to_players }); engine/restriction/predicate.rs:350-355 | `Event { pattern: EventPattern, affected_objects: ObjectSet, affected_players: PlayerSet, by: Option<SourceFilter> }` and `ApplyReplacement { kind: ReplacementKindFilter, to_objects: ObjectSet, to_players: PlayerSet }`; §4.1: `Restriction::Event { pattern: EventPattern::Destroy { source: None }, affected_objects: ObjectSet::SourceOnly, affected_players: PlayerSet::Nobody, by: None }`. |
| cant-effects §3.5 The predicate, the gate (555-558) | "The same instrument as `replacement_ability_sources`: a set maintained at `place_on_battlefield` and `cleanup_zone_state`, over-approximating" | state/game_state.rs:1984,2069-2070 (register_static_effects inserts into restriction_ability_sources); engine/zones.rs:515,531 (cleanup_zone_state removes); test_support.rs:927 (place_on_battlefield is a test helper); engine/restriction/predicate.rs:114-117 (the gate's three legs plus the registry) | `GameState::restriction_ability_sources`, filled by `register_static_effects` on entry (via `perform_entry`/`make_permanent`), cleared by `cleanup_zone_state` on leaving; over-approximates (a stripped ability costs a walk, never an answer). Gate adds `any_granted_restriction`, `any_copied_restriction`; sweep skipped when all three static legs are empty; predicate returns early only when the registry is empty too. |
| cant-effects §3.7 'Why it lands on this document' (613-614) | "`is_prohibited` has seven call sites and none is in `engine/combat`." | engine/replacement/gather.rs:650; pipeline.rs:456, :2032, :3095, :3379; engine/resolve.rs:2335 | "`is_prohibited` and `prohibition` have six call sites between them (gather, four in the pipeline, resolve's candidate filter) and none is in `engine/combat`." |
| cant-effects §3.7 'Why it lands on this document' (614-618; a refuter places it at 624-627) and half 2 (639-641) | `validate_attackers` reads `entry.tapped` and asks `has_keyword` for Defender inline; `can_block` reads `entry.tapped`; RS-3a rewrites "exactly" the two validators, `can_block` and the two enumerators | oracle/legality.rs:30-46 (can_attack: entry.tapped :38, Defender :45), :55-61 (can_be_a_blocker: entry.tapped :61), :147-163 (legal_attackers/legal_blockers filter through them); engine/combat/validation.rs:165 (validate_attackers calls can_attack), :264 (can_block calls can_be_a_blocker) | `oracle/legality.rs::can_attack` reads `entry.tapped` and asks `has_keyword` for Defender; `can_be_a_blocker` reads `entry.tapped`. `validate_attackers`, `legal_attackers` ask `can_attack`; `legal_blockers` and `can_block` (before its pair checks) ask `can_be_a_blocker`; `validate_blockers` reaches it via `can_block`. RS-3a's seam: those two helpers, `can_block`'s pair checks, the two set-level checks. Change the closing "the same four functions" too. |
| cant-effects §3.7 The mirror, CR 609.4, constraint half 1 (627-630; refuters place it at 640-642) | "Today `is_prohibited` returns `bool` and `restriction/predicate.rs` returns `true` on the first match, discarding the attribution." | engine/restriction/predicate.rs:54-70 (Prohibition { source, by: ProhibitedBy::{Keyword, StaticAbility(&str), RegisteredEffect} }), :96 (pub(crate) fn prohibition -> Option<Prohibition>) | `is_prohibited` returns `bool`, but since SU-8 (2026-10-06) it is the `is_some` of `restriction::prohibition`, which returns the first match in sweep order (keywords, battlefield sweep, registry) as a `Prohibition`: source object plus keyword, static ability text or registered effect. It names one restriction, so a Defender-suppressing consult must still see past a Defender-derived match. |
| cant-effects §4.1 Tier 2, Cost (702-705); also §3.5's sketch (541) and §5.2 (1219, 1236) | "`gather`'s two halves of 'does this apply' — `watches` and `affects` (named `pattern_matches` / `shield_contains` when this was written; renamed closing `rb-review.md` C6)" | engine/replacement/gather.rs:735 (pub(crate) fn set_affects), :792 (pub(crate) fn pattern_watches) | §4.1: `pattern_watches` and `set_affects` (`pattern_matches`/`shield_contains` when written, `watches`/`affects` after `rb-review.md` C6, `pub(crate)` under these names at RS-1 so the restriction predicate shares them). §3.5: `Query::Event { action, cause, lookahead }` (predicate.rs:34-46). §5.2: `gather` takes a fifth `frame: &EntryFrame` (gather.rs:202-208); the re-ask is `prohibition` (predicate.rs:96), called at pipeline.rs:456 with `lookahead: Some(&frame)`. |
| cant-effects §4.5 Tier 1d — targeting, Site (955-958) | "plus `legality.rs::enumerate_legal_selections` and `has_any_legal_choice`, because CR 601.2c makes an illegal target an illegal *cast*" | oracle/mana_helpers.rs:394-415 (fn every_instance_has_a_choice, 'this is the single `has_any_legal_choice` it replaced') | "plus `legality.rs::enumerate_legal_selections` and `mana_helpers.rs::every_instance_has_a_choice` (CR 601.2c's check over every instance of "target"), because CR 601.2c makes an illegal target an illegal *cast*" |
| cant-effects §4.6 Tier 1e — costs, the Cost projection table (993-1010) | `Cost` is "a closed 10-variant enum and every variant maps": `Tap`/`Untap` -> `GameAction::Tap`/`Untap`; `SacrificeSelf`/`Sacrifice(..)` -> `ZoneChange { cause: Sacrificed }`; `Discard(..)` -> `ZoneChange { ..., cause: Discarded }` | types/costs.rs:14 (TapSelf), :16 (UntapSelf), :28 (ExileFromGraveyard(ObjectFilter, u32)); engine/actions.rs:246 (ZoneChange's `act: ZoneChangeAct`) | The count holds (10 variants). Table: `TapSelf`/`UntapSelf` -> `GameAction::Tap`/`Untap`; `SacrificeSelf`/`Sacrifice(..)` -> `ZoneChange { act: Sacrificed }`; `Discard(..)` -> `ZoneChange { from: Hand, to: Graveyard, act: Discarded }`; new row `ExileFromGraveyard(..)` -> `ZoneChange { from: Graveyard, to: Exile, act: Put }` (types/zones.rs:234-241 files CR 701.13a under `Put`). Line 310's `cause: Sacrificed` -> `act: Sacrificed`. |
| cant-effects §4.8 Tier 5 (1124-1127); also §2.4 third bullet (263-266) and §4.9 Scoping (1186-1187) | "'If they can't, X' needs `Effect::Conditional`, which `resolve.rs` currently errors on." §2.4: "These need `Effect::Conditional`, which is unimplemented and belongs to Phase 6." | engine/resolve.rs:450-457 (Effect::Conditional(condition, inner) resolves through resolution_condition_holds) | `Effect::Conditional` resolves (CR 608.2c; resolve.rs:450, `resolution_condition_holds`). Single-player "if you can't, X" is `Effect::Conditional(Condition::CostAnswer(CostAnswer::Cant), X)` (CR 118.12; `Primitive::Sacrifice` sets `Cant`, resolve.rs:1502; tests/phase_tr2b_integration_test.rs:345). Unbuilt: Plaguecrafter's per-player answer (one `last_cost_answer`; `Effect::ForEach` errors, resolve.rs:517), which §4.9 should say; §2.4's 149 "unless" clauses wait on `RestrictionDef`'s `unless: Option<Condition>`. |
| cant-effects §4.9 Never prompt for a choice a restriction forbids, 'Where it goes' (1157-1159) | "Into candidate enumeration — `enumerate_legal_selections` and the resolution-time selection path — as one `is_prohibited` call per candidate." | engine/resolve.rs:2323-2342 (GameState::admits, the one candidate filter); oracle/legality.rs (no restriction consult) | Into the resolution-time selection path only: `resolve.rs`'s `choose_as_it_applies` keeps a candidate only if `GameState::admits` lets it through, one `is_prohibited` call per candidate against the event the verb would propose; a named sacrifice uses the same `admits`. `enumerate_legal_selections` does not consult it; targeting's consult is RS-2's, asked by the enumeration's targeting callers (§4.5). |

**`copy-effects-architecture.md`** (23)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| copy-effects §Header, Status line (3-5) | "**Status:** design, authored 2026-08-29 in answer to `codebase-state.md` ... found this gap. No code written yet." | engine/layers/copy.rs:67 (`pub struct CopiableValues`); engine/layers/types.rs:147 (`CopyFrom` arm) | Rewrite only the last sentence in place, keeping the provenance clause ("in answer to `codebase-state.md` → ... found this gap."): "CV-1, CV-2a, CV-2b and CV-1b have landed (§7a–§7d); CV-3 to CV-7 are unbuilt." |
| copy-effects §1.1 The CR decides it (83-84) and §3.1's last bullet | The copiable values are "`compute_to_ceiling(game, id, END_OF_LAYER_1, cache)`, materialized into an owned snapshot ... `compute.rs`'s existing frame-cache entry point"; §3.1's last bullet names the same deleted function | engine/layers/copy.rs:287-302 (`copiable_values` calls `board::frame_at_ceiling`, engine/layers/board.rs:1966); engine/layers/copy.rs:306 (`copiable_values_on_battlefield` uses `compute_board_to`, board.rs:1720) | §1.1: "`layers::copy::copiable_values(game, id)`: `board::frame_at_ceiling(game, id, END_OF_LAYER_1)` taken into an owned `CopiableValues` at the moment the copy effect starts to apply (and `copiable_values_on_battlefield` for every permanent in one `compute_board_to` pass)"; drop "existing frame-cache entry point". §3.1: replace "a future `compute_to_ceiling` change that raised the default ceiling" with "a capture taken past `END_OF_LAYER_1`, or a sublayer added to `LAYER_ORDER` without moving that const (the `debug_assert` in `copiable_values` pins it)". |
| copy-effects §1.2 One mechanism, not two (97) | "**The answer is a Layer 1a row for every population that has copiable values at all**" | state/battlefield.rs:132,148-151 (`PermanentState::entered_as: EnteredAs { copy, edits }`); engine/layers/board.rs:1045 (`push_own(OwnApplication::EnteredAsCopy, CopyFrom(..))` from state, no registry row) | "**The answer is layer 1a's `CopyFrom` for every permanent that copies another object**: a row for a copy that begins after entry (Tier C), and the permanent's own `PermanentState::entered_as`, applied at its timestamp, for one that entered as a copy (Tier B). A token copy's values are its own `CardData` (§3.3, producer 3)." |
| copy-effects §2.1 The mechanism table, Tier B row (189) | Tier B (enters as a copy, a CR 616.1c replacement; 707.5, 616.1c; 69 cards) has Built? cell "❌ (bucket ships, no producer)" | types/replacement.rs:966 (`Rewrite::EnterAsCopy`, the producer; class `CopyAsEnters` at :1891); cards/phase_cv_cards.rs:343,416 (Clone, Spark Double built on it) | Built? cell becomes "✅ CV-2a, CV-2b (`Rewrite::EnterAsCopy`; Clone and Spark Double; the cards in §4.1a's 'Not in CV-2' table are not built yet)"; other cells unchanged. |
| copy-effects §2.5 What the engine has today (276-290); §1.4 item 3 (~160) | `Layer::Layer1Copy` (types.rs:43): "nothing produces an effect in it"; `CopyOnEnter`/`BackFaceUp` (replacement.rs:324,326): "no producer"; `GameObject.is_copy` "written nowhere, read nowhere"; copiable-values capture ❌ | engine/layers/types.rs:52 (`Layer1Copy`: "Only 1a has producers: CopyFrom rows ... PermanentState::entered_as"); types/replacement.rs:1870 (`CopyAsEnters`), produced by `Rewrite::EnterAsCopy` (cards/phase_cv_cards.rs:343); engine/leaving.rs:209 (reads `is_copy`); engine/layers/copy.rs:287 (capture); engine/resolve.rs:1228 (`CreateToken`) | Rewrite §2.5: `Layer1Copy` (types.rs:52) has 1a producers (`CopyFrom` rows from `Primitive::Copy`, entry copies from `PermanentState::entered_as`), 1b none; `LAYER_ORDER` (compute.rs:48) lacks the 1a/1b split (CV-6); `CopyAsEnters` (renamed from `CopyOnEnter`, replacement.rs:1870) comes from `Rewrite::EnterAsCopy` (:1891), `BackFaceUp` (:1872) has none until CV-5; `is_copy` (objects/object.rs:30) has no writer, read at leaving.rs:209 and scenario/write.rs:481; `CreateToken` (resolve.rs:1228) proposes `GameAction::CreateTokens` from a `TokenDef`, no token-copy path yet (CV-3); the capture is `layers::copy::copiable_values` (copy.rs:287) at `END_OF_LAYER_1`. Delete "Nothing here is wrong; it is all correctly-shaped scaffolding with no producer." §1.4 item 3: prefix "At authoring (2026-08-29)," or past tense, since CV-2a gave `CopyAsEnters` a producer. |
| copy-effects §3.2 CopiableValues — the type surface (351-383) | Fields `types: HashSet<CardType>`, `subtypes: HashSet<Subtype>`, `keywords: HashSet<KeywordFlag>`, `abilities: Vec<AbilityDef>`, and `back_face: Option<Box<CopiableValues>>` for CR 707.8a | engine/layers/copy.rs:67-97 (`types: CardTypes`, `subtypes: Subtypes`, `keyword_flags: HashSet<KeywordFlag>`, `abilities: Arc<Vec<AbilityDef>>`, `loyalty: Option<i32>`; no `back_face`) | Fields as built: name, `mana_cost: Option<ManaCost>`, colors, `types: CardTypes`, `subtypes: Subtypes`, supertypes, `keyword_flags`, `abilities: Arc<Vec<AbilityDef>>` (shared, written only through `Arc::make_mut`), power, toughness, loyalty (CR 707.2, CR 306.5b). `back_face` is not on the type; CV-5 adds it with CR 712 faces. |
| copy-effects §3.2 (388) and §4.7 'The other two summary flags' (1058-1063) | "`compute.rs`'s `any_control_changing` fast path (`compute.rs:389`, `characteristics.rs:51`) ... `any_multi_row_group` (`compute.rs:203`)" | state/continuous_effects.rs:68 (`any_multi_row_group`), :88 (`any_control_changing`) on `RegistryScopeSummary`; read at oracle/characteristics.rs:168 and oracle/board.rs:17 | §3.2: the fast path is `RegistryScopeSummary::any_control_changing` (state/continuous_effects.rs:88), gated in `get_effective_controller` at oracle/characteristics.rs:168, which returns `compute::base_controller`. §4.7: `any_multi_row_group` (continuous_effects.rs:68, set at :270 when two rows share an `EffectGroup`, read at engine/layers/board.rs:297); `any_control_changing` gates at characteristics.rs:168, oracle/board.rs:17 and engine/triggers/control.rs:57,86,112. Same stale wording at engine/layers/copy.rs:62; continuous_effects.rs:76 names a deleted `compute::effective_controller`. |
| copy-effects §3.2 'The one arm this adds to EffectModification' (395-401) | "/// Boxed for the same reason `GrantAbility` is: this is the largest payload in the enum and the enum is matched at every layer. CopyFrom(Box<CopiableValues>)," | engine/layers/types.rs:147 (`CopyFrom(Arc<...CopiableValues>)`); engine/layers/types.rs:186 (`GrantAbility(Arc<AbilityDef>)`) | "/// Behind an `Arc`, as `GrantAbility` is. A pointer, because this is the largest payload in the enum and stored inline it would enlarge every row the layer walk iterates. An `Arc` rather than a `Box`, because the registry is cloned with every fork (§9 item 4). CopyFrom(Arc<CopiableValues>)," |
| copy-effects §3.3 The four producers (419-423) and §7 table, CV-3 row (1359-1361) | "exactly as `token_card_data` already builds one from a `TokenDef` (`engine/resolve.rs:1450`)"; CV-3 row: "**1** constructor beside `token_card_data` (`resolve.rs:1450`); **1** `Primitive::CreateToken` arm" | types/effects.rs:1297 (`TokenDef::card_data`), called at engine/actions.rs:2318 in `create_tokens` (actions.rs:2299); engine/resolve.rs:1228 (`Primitive::CreateToken` proposes `GameAction::CreateTokens`) | §3.3: "exactly as `TokenDef::card_data` (`types/effects.rs:1297`) already builds one from a `TokenDef`; `create_tokens` (`engine/actions.rs:2299`) calls it as it performs `GameAction::CreateTokens`." CV-3 row: "**1** constructor beside `TokenDef::card_data` (`types/effects.rs:1297`, called by `create_tokens`, `engine/actions.rs:2299`); **1** `Primitive::CreateToken` arm (`resolve.rs:1228`, which proposes `GameAction::CreateTokens`)". Drop "not a new resolve arm": how a copy token reaches `CreateTokens` is CV-3's decision, not yet made. |
| copy-effects §3.5 What is deliberately not a copy effect (478-480); §9 item 5 (1608) | `is_copy` "has no reader today and CV-1 should not invent one"; §9 item 5: "a flag with no reader has no cost" | engine/leaving.rs:209 (CR 707.10 "not a card" leg reads `obj.is_copy`); scenario/write.rs:470-481 | "It has no writer yet. Its one engine reader is the CR 707.10 'not a card' leg in `engine/leaving.rs`'s CR 800.4a stack sweep, plus the scenario writer's report. That reader needs `is_copy` set on every copy of a spell and never on an object represented by a card; a cast copy of a card (CR 707.12) also satisfies it. So CV-4's first writer must give the flag that meaning, or migrate the reader if it deletes the flag." Same fix for §2.5's "read nowhere" (283). |
| copy-effects §4.1a 'Not in CV-2, and the facility each waits for' table, Wall of Stolen Identity row (739) | Row "CR 707.9g's linked reflexive trigger \| Wall of Stolen Identity \| reflexive triggers (TR-3)" | types/triggers.rs:1662 (`Reflexive(ReflexiveForm)`), :1668 (`enum ReflexiveForm`); types/effects.rs:1656-1673 (`CopyException`, four arms, none for 707.9g) | Row: "CR 707.9g's linked triggered ability \| Wall of Stolen Identity \| a `CopyException` arm for 707.9g; CR 603.11's link from the replacement to its 'when you do' trigger (linked abilities, `backlog.md` §2.2), since TR-3c's `DelayedDuration::Reflexive` is CR 603.12's, made by a resolution, and does not reach a printed linked trigger; and a 'for as long as you control' `Duration`". (No `TriggerDuration` type exists.) |
| copy-effects §4.2 'The choice site, and what carries the chosen id' (823-826) | Cytoshape gets "its own decision site and its own `ChoiceKind::ChooseCopySource { spell_id }`" | ui/choice_types.rs:258 (`ChooseCopySource { source: ObjectId }`); ui/ask.rs:1268 | "its own decision site and its own `ChoiceKind::ChooseCopySource { source }`" |
| copy-effects §4.4 Tiers D and E (892-895) and §9 item 7 (1626-1633) | "a CR 603.7h identity question and it lands with triggers (critical-path item 6), so CV-4 covers spell copies and defers *ability* copies (39 clauses) with a cite"; §9 item 7: "39 clauses, deferred to critical-path item 6" | No ability-copy path in src (no `fn copy_of`) after TR-1..TR-7; state/game_state.rs:136 (`StackEntry.ability_identity`, the identity a copy would keep) | §4.4: "That is a CR 603.7h identity question: the copy keeps the original's `StackEntry.ability_identity` (RA-2's `AbilityIdentity`) and advances the same resolution count (`triggers-architecture.md` §6.5). No TR phase builds ability copies, and §6.5 and §13 there give them to CV-4, so CV-4 covers spell copies and *ability* copies (39 clauses) (§7's row, corrected 2026-10-12, A6p)." §9 item 7: "39 clauses, CV-4's (§7; `triggers-architecture.md` §6.5 and §13), named here so CV-4 keeps §6.5's sentence." Line 912's "(707.10b, item 6)" loses "item 6". |
| copy-effects §4.7 Leg 1 — gather's gate, 'The fix, specified' (1002-1027) | Gate rests on "`RegistryScopeSummary::any_granted_replacement` (Layer 6 grants, `continuous_effects.rs:73`)" (`gather.rs:143`); "The fix, specified: extend `RegistryScopeSummary` with a third flag" — `pub any_copied_replacement: bool` | state/continuous_effects.rs:133 (`unattributed_replacement_zones: ZoneSet`), :141 (`any_named_unattributed_replacement: bool`), covering grant and copy rows alike; engine/replacement/gather.rs:269-280 (gate); engine/zones.rs:515 (`cleanup_zone_state`) | As built: one leg for every unprinted route — `RegistryScopeSummary::unattributed_replacement_zones` (a `Filter` grant or copy row, by zone) and `any_named_unattributed_replacement` (a named-set row) — read by `gather`'s gate beside `replacement_ability_sources` and `zone_replacement_ability_sources`. |
| copy-effects §4.7 Leg 2 — register_static_effects (1029-1031) | "`game_state.rs:737` 'Reads printed abilities on purpose'" — deliberately, to avoid circularity inside `place_on_battlefield` | state/game_state.rs:1984 (`register_static_effects`), its comment at ~1995; test_support.rs:927 (`place_on_battlefield`, now only a test helper) | `register_static_effects` (`state/game_state.rs`) reads the abilities the object arrived with, printed or an entry copy's (CR 707.5), never a frame; its `// PRE-LAYER ZONE:` tag says why: the registry it builds is the layer walk's input. `perform_entry` runs it for the battlefield and `move_object` for every other zone, so only a permanent that becomes a copy later (CR 707.4) needs `register_copied_static_effects`. |
| copy-effects §4.7 Leg 3 — anything Phase 6 adds (1053-1056) | "Triggered-ability registration will want the same ETB scan and must take the effective list or a copy's triggers will be dead. Named here because critical-path item 6 has not started" | engine/triggers/dispatch.rs:12,27 (dispatch reads the effective ability list via `compute_characteristics`); state/continuous_effects.rs:186 (`any_copied_trigger_multiplier`) | "Leg 3 — triggered abilities, as built (item 6)": `register_static_effects` files `GameState::trigger_sources` from the abilities a permanent entered with, printed or its entry copy's (CR 707.5); a later copy reaches dispatch via `triggered_ability_kinds`' `CopyFrom` arm into `unattributed_trigger_zones` (`Filter` row) or `named_unattributed_trigger_kinds` (named row, read by `named_trigger_carriers`); dispatch reads the effective list via `compute_characteristics`; CR 603.2d's multiplier has the copied twin `any_copied_trigger_multiplier`. (The other refuter's text says `trigger_sources` records printed abilities only; settle against game_state.rs:2003-2028.) |
| copy-effects §5.3 The CR 400.7 trap, "Resolved by CV-1b" note (1190) and closing paragraph (1236) | "`move_object` prunes the mover from every row that names it, beside `remove_by_source` and never instead of it"; line 1236: the teardown "must run *alongside* `remove_by_source`" | engine/zones.rs:520 (`cleanup_zone_state` calls `remove_rows_ending_with`); engine/zones.rs:485-486; state/duration_registry.rs:207-210 | "`move_object` prunes the mover from every row that names it, beside `remove_rows_ending_with` (the battlefield sweep that ends a static ability's rows and a resolution's rows that end with their source) and never instead of it." Line 1236 should name the same function (the evidence notes part 1 at 1169 names the old sweep too). |
| copy-effects §5.3 The CR 400.7 trap, twelve-exceptions note (1198) | "Two keep a row today's cards can make, and `break_references_to` spares them; five are a permission's, and the prune spares nothing for them yet." | engine/zones.rs:500-505 (spares only the continuous registry's rows, stack to battlefield); engine/zones.rs:82-95 (the 400.7c re-point) | "Two keep a row today's cards can make: `break_references_to` spares 400.7a's continuous rows, and `move_object` re-points 400.7c's prevention shield at the permanent; five are a permission's, and the prune spares nothing for them yet." |
| copy-effects §5.3 The CR 400.7 trap, twelve-exceptions table, row 400.7c (1205) | "400.7c \| prevention of damage from a permanent spell \| the prune spares it (`prevents_damage_from`) \| ✅" | engine/zones.rs:504 (`self.replacement_effects.remove_references_to(id, \|_\| false)`); engine/zones.rs:82-95 (`move_object` re-points the shield); types/replacement.rs:668 (`pub object: Option<ObjectRef>`) | "400.7c \| prevention of damage from a permanent spell \| the shield names its chosen source by identity (`SourcePattern.object`, an `ObjectRef`), which the prune never reaches; `move_object` re-points it at the permanent (`prevents_damage_from`) \| ✅" |
| copy-effects §5.3 'The tempting shortcut' (1210) | "`ContinuousEffect.source` is documented (`layers/types.rs:174`) as the object whose static ability generates the effect" | engine/layers/types.rs:269 (the field; its own doc at :268 says only "The object that generates this effect."); engine/layers/types.rs:229 (`EffectOrigin::StaticAbility`'s doc, the text once at :174) | "`ContinuousEffect.source` is documented (`engine/layers/types.rs`, `EffectOrigin::StaticAbility`'s doc: 'generated by a static ability of `source`') as the object whose static ability generates the effect" |
| copy-effects §5.4 Layer 1's sublayers are inverted in four places (1242-1282); §4.6 (~964); §9 item 3 (~1575) | "Three places in this repository say the opposite — that 1a is face-down and 1b is copy ... **No code is wrong**, because `Layer1Copy` collapses both and nothing produces an effect in either"; layers §7 and `compute.rs:27` named as owed | engine/layers/types.rs:43-52 ("CR 613.2a is 1a, copy; CR 613.2b is 1b, face-down"; "Only 1a has producers"); plans/layers-architecture.md:64-68 (`Layer1aCopy`, `Layer1bFaceDown`) and :756; compute.rs no longer carries the inverted sentence | Recast §5.4, §4.6:966-967 and §9 item 3 as a record: found 2026-08-29, corrected by CV-1 at the four named sites (layers-architecture.md §7's bullets :755-756 and `Layer` block :64-68, codebase-state.md, compute.rs's `LAYER_ORDER` doc, now :42). Order is 1a copy, 1b face-down; 1a has producers (`CopyFrom` rows from `Primitive::Copy`, entry copies via `PermanentState::entered_as`), 1b none until CV-6; the one integer is `layers::copy::END_OF_LAYER_1`, pinned by `copiable_values`'s debug_assert. Still owed in layers-architecture.md: :766, :93, :139/:142, :306/:313, :2819 (:1425-1426 is Phase LD's historical scope). |
| copy-effects §10 Explicitly out of scope (1714-1715) | "**Text-changing effects (Layer 3)** — deferred indefinitely at 25 cards (`engine/layers/types.rs`)" | engine/layers/types.rs:55-57 ("Unbuilt, and owed: about 20 vintage-legal cards need it") | "**Text-changing effects (Layer 3)** — unbuilt and owed: about 20 vintage-legal cards need it (`engine/layers/types.rs`, `codebase-state.md` item 195). It is slotted as Phase 8's line item C2 (`roadmap-v2.md` §C), designed in `layers-architecture.md`. It is also §4.7's *other* missing gate leg, so the rule in §4.7 is written to cover it in advance." |
| copy-effects §11 Documents this owes, 'Owed but deliberately not written here' (1733-1750) | Owed: layers-architecture.md §7 "and its `Layer` code block at :64–68" (sublayer inversion); `compute.rs:27` (same inversion); `game_state.rs:249` (scope the "between them the gate is sound" claim) | plans/layers-architecture.md:64-68 (already `Layer1aCopy`/`Layer1bFaceDown`); engine/layers/types.rs:43-52 (right order); state/game_state.rs:505-517 (`replacement_ability_sources` doc names its other legs; no "gate is sound" anywhere) | Narrow the list to what remains. Still owed: layers-architecture.md's inversion at :766 (§7's implementation note), :139/:142, :306/:313, :1425-1426 (the other refuter lists :93 instead of :1425-1426), and §6's table (:716-729) lacks a CR 707.9d pointer; specdb.py's `owed` default `ticket LIKE 'NEW%'` (:990; §9 item 11), its own PR; replacement-architecture.md §9's RC-4 bucket list (not re-verified). Discharged: compute.rs's `LAYER_ORDER` comment, the `CopyAsEnters` rename and phase cite (types/replacement.rs:1855-1870), `replacement_ability_sources`' scoping (game_state.rs:509-512), cant-effects' closing paragraph (:1573-1583, still spells `CopyOnEnter`). §9 item 3 (:1577-1578) drops compute.rs:27. |

**`cost-architecture.md`** (14)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| cost Header, Status (5) | "each is answered where it lands and listed in §11). CM-0 and CM-1 built." | types/cost_modification.rs:110, :178 (CM-2's arms); engine/costs.rs:141 (CM-3's payment_order_rank); ui/mana_window_stop.rs (CM-4); the doc's own stubs at cost-architecture.md:1353, :1371, :1400 mark CM-2 to CM-4 ✅ | "...listed in §11). CM-0 through CM-4 built. CP-1 and CP-2 are sized slots (§6)." |
| cost §3.1, the third gate leg (243-247) | "It is exact today, because `compute_non_member` applies no registry rows": a non-member's list *is* its printed list, so no Layer 6 grant or Layer 1 copy reaches a stack spell or hand card | engine/layers/compute.rs:332-341, :345-363, :429-450 (compute_non_member applies `notes`); engine/layers/board.rs:1819-1822 (PassMembership::LeftOut), :1882-1892 (left_out_zones), :1958-1960 (a stack spell a row reaches is a pass member) | The leg is sound but no longer exact by construction. Since LJ a `Filter` row naming the stack makes a spell a pass member; since LL a left-out hand or library card gets each reaching row through `compute_non_member`'s notes. It rests on `any_granted_cost_modification`/`any_copied_cost_modification`, which open the sweep and make the gather compute the spell's frame (`codebase-state.md` item 75). |
| cost §3.1, "A cost modification is discovered, not registered" (257-265) | `register_static_effects` inserts a source only when `ability.effect` *is* `Effect::Replacement`/`Effect::Restriction`; `Effect::Conditional(cond, Replacement)` is never inserted, so a conditional replacement or restriction static "is inert today" | state/game_state.rs:2057, :2069 (register_static_effects); types/effects.rs:2591 (replacement_body), :2602 (as_restriction), both through with_its_condition (:2555); replacement/gather.rs:604-610; restriction/predicate.rs:145 | All three gates see through the wrapper: `register_static_effects` files through `Effect::replacement_body`, `Effect::as_restriction` and `Effect::as_cost_modification`, each peeling `Conditional` with `with_its_condition`. The sweeps (`replacement_of`, `is_prohibited`, the cost gather) peel too and ask the condition at the proposal (RE-6, 2026-09-12; `codebase-state.md` item 73). Trinisphere's "as long as" is seen by every leg. |
| cost §3.2, the type-surface code block (284, 294-295) | The block shows `Itself` ("the spell this ability is on (CR 113.6d/e, 702.41a): CM-2") and `ReduceGeneric(AmountExpr)` ("cost {X} less, where X is …": CM-2) as commented arms not yet in the enum | types/cost_modification.rs:110 (CostSubject::Itself), :178 (CostChange::ReduceGeneric(AmountExpr)) | Write both arms live. In `CostSubject`: `/// This spell — CR 113.6d, 702.41a (CM-2). Itself,`. In `CostChange`: `/// "cost {N} less, where N is …" — generic only, read through settled_amount (CM-2). ReduceGeneric(AmountExpr),`. Leave only `ActivatedAbilities` and `not_below` as commented arms. |
| cost §3.3, the pipeline as built, step 1 Merge (340-343) | "Every `Cost::Mana` in the assembled list — the base or alternative cost, an X expansion, a kicker's mana — becomes **one** mana component." | engine/cost_determination/total.rs:221 (ManaComponent.x_symbol_count, X kept a variable), :197 (locked_at(x)), :53 (total.locked_at(x_value)) | Every `Cost::Mana` (the base or alternative cost and a kicker's mana) becomes **one** mana component. The base's X stays a variable through steps 3–5 (`ManaComponent::x_symbol_count`, read by no modification); step 6 evaluates it at the announced X (`TotalCostInX::locked_at`); `total_cost_in_x` is the total 601.2b's announcement of X reads. (A refuter: it omits that an additional cost's X stays a pip, total.rs:248.) |
| cost §3.6, enumeration must agree with enforcement (549-550) | "`find_mana_sources` then reasons about the previewed component as it did about the printed one." (the sentence before it still holds) | oracle/mana_helpers.rs:160-169 (can_cast_with: preview_mana_cost, then mana_payable, then ManaSupply::covers); oracle/mana_supply.rs:11-14 | `mana_payable` then asks whether the pool or the inventory pays the previewed component: `oracle::mana_supply`'s Hall's-condition check, exact since MA-1 (`mana-architecture.md` §3.2). For an X spell it is asked at the least legal X (MA-2, §3.7). |
| cost §3.8, Commander tax (644-645) | "*a commander* — `is_commander` is set nowhere in production ("Before Commander" item 2), and `can_begin_to_cast` admits only the hand." | scenario/build.rs:247 (sets is_commander from the `Commander` card word); the hand half holds (oracle/mana_helpers.rs:81) | *a commander*: `is_commander` is set only by the scenario loader's `Commander` card word (`scenario/build.rs`), with no `GameConfig` designation and no deck-built game setting it, and `can_begin_to_cast` admits only the hand. |
| cost §3.12, paying is deciding then performing, rank table (899) | \| 1 \| `Tap`, `Untap`, `PayLife`, counters \| | engine/costs.rs:144-148 (payment_order_rank); types/costs.rs:14, 16 | \| 1 \| `TapSelf`, `UntapSelf`, `PayLife`, `RemoveCounters`, `AddCounters` \| |
| cost §4, "Two predicates, one per gate" (973-981); also §8 item 6a (1207) | "`applies_from_battlefield` decides whether `register_static_effects` records a permanent as a source at all — an affinity creature is a source of nothing"; 6a: "Both fixed by asking `CostSubject` a question (`applies_from_battlefield`, `applies_to_its_own_object`)" | state/game_state.rs:2047, :2077 (gates on engine::zone_function::functions_in, then as_cost_modification); types/cost_modification.rs, doc comment above applies_to_its_own_object; engine/zone_function.rs:57-60 | One predicate on `CostSubject` and one zone rule: `register_static_effects` records a battlefield source only when `engine::zone_function::functions_in` says the ability functions there (CR 113.6), which derives "not on the battlefield" for an `Itself` subject from `applies_to_its_own_object`, so an affinity creature sources nothing. That method is also source 2's gate leg; CR 602.2b's activated abilities will answer `false`. Amend 6a likewise. |
| cost §4, "What the spell-side match reads" (983-985); the entry also names §3.2's CM-0 paragraph (318-319) | "`object_matches_filter_in_frame(spell, filter, you, &frame)` — `permanent_matches_filter_in_frame` until CM-0 — with `frame = compute_characteristics(game, spell)`." | engine/cost_determination/gather.rs:176; engine/targeting.rs:728 (object_matches_filter_of_source) | `object_matches_filter_of_source(spell, filter, source_controller, source, Some(HeldFrame::Characteristics(frame)))` with `frame = compute_characteristics(game, spell)`: the one `ObjectFilter` leaf table (`layers-architecture.md` §3.4), asked for the cost ability's source. It was `object_matches_filter_in_frame` from CM-0 until 2026-10-14, `permanent_matches_filter_in_frame` before. (A refuter calls 318-319 a dated CM-0 plan, a record, not stale.) |
| cost §6, CP-1 row (1031) | "`{Q}` exists as `Cost::Untap` and its atoms want annotations." | types/costs.rs:16 (Cost::UntapSelf) | "`{Q}` exists as `Cost::UntapSelf` and its atoms want annotations." |
| cost §6, CP-2 row (1032) | "with X announced from every cost that holds it (CR 107.3a), since `cast_spell` reads X from the mana cost alone and Toxic Deluge's is in an additional cost" | engine/put_on_stack.rs:185-191 (total_cost_in_x; has_x also reads total_in_x.mana.x_symbol_count()); engine/cost_determination/total.rs:161-171, :67-70 and :248-255 (an additional cost's {X} stays a pip) | "...(CR 107.3a), since `cast_spell` announces X only from the chosen base, the mana cost or an alternative cost's own mana (`total_cost_in_x`, `ManaComponent::add_base`), never from an additional cost: its {X} stays a symbol nothing pays, and Toxic Deluge's X is an additional cost's life payment, which `PayLife(u64)` cannot hold." (Both proposed corrections wrongly had additional costs' mana supply X.) |
| cost §8 Findings, item 1 (1160-1166) | "**A conditional replacement or restriction static is inert** (§3.1): `register_static_effects` matches the body without peeling `Effect::Conditional`, and neither gather does either." | state/game_state.rs:2057, :2069; types/effects.rs:2591, :2602; replacement/gather.rs:603-611; restriction/predicate.rs:145; continuous_effects.rs:321, :336; registry.rs:1108 (Laboratory Maniac) | "1. **A conditional replacement or restriction static was inert** (§3.1). ✅ closed at RE-6 (2026-09-12), `codebase-state.md` item 73." It now files through `Effect::replacement_body`/`Effect::as_restriction` (the `as_cost_modification` peel); the gathers read `replacement_of` and `as_restriction`, asking the condition at the proposal. Laboratory Maniac is the first registered card with one; #249 gave the granted and copied legs `as_restriction` last. |
| cost §8 Findings, item 7 (1213-1220) | "**A registry row cannot reach a non-member, so CR 113.6e's second sentence has nothing to build on**": `compute_non_member` "applies CDAs and no rows at all", so printed lists make source 2's printed leg *exact* | engine/layers/compute.rs:332-341, :345-363, :429-450; engine/layers/board.rs:1882-1892, :1958-1960; phase_rf_cards.rs:243 (Hollow Hands strips a hand card) | Retitle "CR 113.6e's second sentence has nothing to grant with". Since LJ a `Filter` row naming HAND or STACK reaches the card: a pass member, or left out and given the pass's notes after its CDAs by `compute_non_member` (LL). Source 2's printed leg (`prints_cost_ability`) is exact only OR'd with the two summary flags. 113.6e waits only on a granting card, PM-0 (`codebase-state.md` item 75). Amend "exact rather than an over-approximation" too. |

**`triggers-architecture.md`** (23)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| triggers §3.3 TriggerEvent table, CardDrawn row (302) | `DrawsCard { player: Option<PlayerRef>, multiplicity }` — never a library-to-hand `ZoneChange` (121.5) | types/triggers.rs:758 (DrawsCard; enum at :734) | `DrawsCard { player: Option<PlayerRef>, multiplicity, place: Option<DrawFilter> }` — never a library-to-hand `ZoneChange` (121.5); `place` is which of the player's draws it watches ("your second card each turn"), read off the record's place in its turn and draw step (item 254). Lands column: "TR-2b; `place` A6r" (not TR-6) |
| triggers §3.3 TriggerEvent table, ManaAdded row (303) | `ManaAdded { source: Option<ObjectFilter>, tapped_for_mana: Option<bool>, mana: Option<ManaType> }` | types/triggers.rs:760-764 (ManaAdded; enum at :734) | `ManaAdded { source: TriggerSubject, tapped_for_mana: Option<bool>, mana: Option<ManaType> }` |
| triggers §3.3 TriggerEvent table, DamageDealt row (304) | `DamageDealt { source: Option<SourcePattern>, recipient: DamageRecipient, combat: Option<bool>, excess: bool, multiplicity }` | types/triggers.rs:766-775 (DamageDealt; enum at :734) | `DamageDealt { source: TriggerSubject, recipient: DamageRecipient, combat: Option<bool>, excess: bool, multiplicity }` |
| triggers §3.3 the count after the table (335; lead-in 275) | "thirty-five performed kinds after TR-5, thirty-one with an arm and four with none"; the table is the projection "in GameEvent's declaration order" | events/event.rs:26-521 (enum GameEvent, 41 variants); types/triggers.rs:603-647 (`EventKind::from_record`, `None` for 10) | Forty-one performed kinds: thirty-one with an arm (LeftTheGame and SameZoneMove via the ZoneChange arm, SameZoneMove only for an arm naming an act, §3.12a decision 3), ten with none. Add rows for RemovedFromCombat (CR 506.4/511.3's record, read by the turn history, engine/triggers/history.rs:312 — not a log record), DelayedTriggerCreated, UntilReturnMade, GameEndedInADraw, SameZoneMove. Drop "in GameEvent's declaration order" (275); fix the "Thirty-four variants, four deleted ... one split" arithmetic |
| triggers §3.4 TriggerBinding code block (374-397) | `pub struct TriggerBinding { pub records: Vec<EventRecord>, pub event: EventIndex, pub subject: Option<ObjectRef>, pub triggered_by: Option<TriggerId>, }` | types/triggers.rs:1528-1555 | `pub struct TriggerBinding { pub def: Arc<TriggerDef>, pub triggering_events: Vec<TriggeringEvent>, pub event: EventIndex, pub triggered_by: Option<TriggerId>, pub referred: Referred, pub source_found_in_graveyard: Option<ObjectRef> }` |
| triggers §3.4 "The projections live on the arm" (~412) | `TriggerEvent` carries three methods — `subject_of(&GameEvent) -> Option<ObjectId>`, `player_of(..) -> Option<PlayerId>`, `amount_of(..) -> Option<u64>` | types/triggers.rs:1188 (`occurrence_subject`), :1281 (`event_roles`), :1325 (`player_of`), :1418 (`amount_of`); engine/triggers/dispatch.rs:1909 (`triggering_events_in`), :2576 (`triggering_event_objects`) | `TriggerEvent` carries two projections, `player_of(&GameEvent) -> Option<PlayerId>` and `amount_of(..) -> Option<u64>`; `event_roles()` names the parts its objects play, and the dispatcher's `triggering_events_in` builds each record's `TriggeringEventObjects` by `EventRole`. `subject_of` went at TR-5a; `occurrence_subject` is CR 603.2c's once-per-subject key, not a projection |
| triggers §3.4 readers paragraph (421-425); §4.4 OncePerEvent bullet (1856) | `EffectRecipient::TriggeringObject` is `subject` checked against the live object's epoch ...; `AmountExpr::TriggeringPower` reads the live object; (§4.4) `TriggeringAmount` is `amount_of` summed over them, and `subject` is `None` | types/effects.rs:883 (`EffectRecipient::Triggering(EventRole)`), :76 (`AmountExpr::TriggeringPower(EventRole)`); types/triggers.rs:1528-1543 (TriggerBinding, no `subject`), :1577 (`triggering_event()`); engine/triggers/binding.rs:156 (`bound_objects`), :213 | §3.4: `EffectRecipient::Triggering(role)` is each object that played `role` in each triggering event (`TriggerBinding::triggering_objects`), kept only while still that object (`bound_objects`: id and `zone_change_epoch`, CR 603.6, 400.7) and, for a zone change, in a public zone (CR 400.7e); `TriggeringPlayer` is `player_of` on the first record; `TriggeringAmount` sums `amount_of` over the records; `TriggeringPower(role)` reads `triggering_object`: the record's frame when it framed it, the live object while there, else the entry's `kept_lki` (CR 608.2h). §4.4: the binding holds each triggering event with its record, `TriggeringAmount` summed over them, no single triggering object (`triggering_event()` is `None`) |
| triggers §3.7 PendingTrigger and the queue (587-620; GameState block 617-619) | `pub is_state_trigger: bool` ("a state trigger stays armed-off until its stack object leaves"); on GameState `pub pending_triggers: Vec<PendingTrigger>`, `pub next_trigger_id: u64` | types/triggers.rs:2026 (PendingTrigger); state/game_state.rs:705-706; state/shared_stores.rs:144-146 (PendingTrigger `CopiedAlone`); engine/triggers/state.rs:177 | `PendingTrigger { seq, origin, controller, source_card, instances: Vec<EffectRecipient>, binding, kept_lki: Vec<Lki>, x_value: Option<u64> }`, no flag: a state trigger's one-shot is read off the queue and the stack (§4.5's TR-6 amendment). On GameState: `pub pending_triggers: SharedList<PendingTrigger>, // state::shared_stores: chunks of 32 a fork shares, each trigger CopiedAlone` and `pub(crate) next_trigger_id: u64` |
| triggers §3.10 history log, RowFact list (852) | `DealtDamage { to, amount, combat }` | state/history_log.rs:81 | `DealtDamage { to, amount, excess, combat }` |
| triggers §3.11 Lki, PermanentFacts block and reader table (~976-1000) | "The body is private, and the readers on the type are its whole surface"; `struct PermanentFacts { tapped: bool, counters: Vec<(CounterType, u32)>, attached_to: Option<ObjectId>, attached_by: Vec<ObjectId>, }` | types/triggers.rs:1906-1914 (PermanentFacts), :1982 (`counters_of`), :1992 (`attacking`), :1999 (`entered_turn`) | PermanentFacts gains `entered_turn: u32` and `attacking: Option<AttackTarget>` (CR 506.3); LkiBody shows `entry: Option<Arc<StackEntry>>` (TR-4d). Table: `counters(kind)` row keeps only `AmountExpr::SourceCounters` (engine/resolve.rs:2580); new rows `counters_of(Option<CounterType>)` (`None` = every kind; `Condition::CounterCount`, `ObjectFilter::WithCounters`, CR 122), `attacking()` (`CurrentOrLastKnown::attacking` for `EffectRecipient::AttackedBy`, defending player, CR 506.3, 508.5), `entered_turn()` (`ObjectFact::Entered`), `entry()` (dispatch.rs:2034's caster, CR 603.10e) |
| triggers §3.12 table, item 17 (1083) | the `EnterBattlefield` performer announces one `CountersChanged` per `EntryCounters` row after `PermanentEnteredBattlefield`, inside the batch, with a new `by: Option<PlayerId>` on the record | events/event.rs:374-387 (CountersPutOn, `by: PlayerId` at :383); state/game_state.rs:1596-1632 (`announce_entry_counters`) | the `EnterBattlefield` performer (`perform_entry`'s `announce_entry_counters`) announces one `CountersPutOn` per `EntryCounters` row with `n > 0` after `PermanentEnteredBattlefield`, inside the batch, `by: PlayerId` (CR 122.6a's putter: the row's named player, else the controller it entered under). Announced, never proposed, so CR 614.16's doublers see nothing twice; CR 122.7's "before" is the record's `now - n` (not "live count minus `added`"); `now` counts that entry's rows of the kind so far. Same stale wording at 1197, 3116 |
| triggers §3.12 the stream's records, the added record (1086-1087) | **`AbilityTriggered { seq, identity: TriggerOrigin, controller, caused_by: usize }`**, the event CR 603.3b's second tier watches (§4.8) | events/event.rs:239-243 (`EventId` at :741) | **`AbilityTriggered { seq, origin: TriggerOrigin, controller, caused_by: Option<EventId> }`** (`None` for a state trigger, §4.5) |
| triggers §4.2 Who is asked, leg 3 (1611) | `zone_trigger_sources: IdMap<ObjectId, Vec<AbilityId>>` | state/game_state.rs:748 (decl), :2032 (write); engine/triggers/state.rs:116 (read) | `zone_trigger_sources: IdMap<ObjectId, EventKindMask>`: each object filed against the record kinds read by its printed triggered defs that function where it is, a `State` condition's `BoardState` bit included; a def that triggers only on its own object moving files it on `newcomer_trigger_sources` instead. Also §11 (~3198) "keyed by ability rather than by kind" is stale |
| triggers §4.2 "The gate, three legs on each of two sets" (1676-1682) | written by `place_on_battlefield`, removed by `cleanup_zone_state` ... `puts_a_triggered_ability(def)` is the predicate beside `puts_a_replacement_ability` | state/game_state.rs:2028 (`register_static_effects` writes `trigger_sources`; via `perform_entry` → `make_permanent`, :1678); state/continuous_effects.rs:287, :362 (`triggered_ability_kinds`); test_support.rs:927 (`place_on_battlefield`, a test helper) | written by `register_static_effects` as `perform_entry` (→ `make_permanent`) registers the arriving permanent, removed by `cleanup_zone_state` ... `triggered_ability_kinds(effect)`, `puts_a_replacement_ability`'s twin, returns the record kinds a grant or copy row's triggers read (a mask, not a yes or no). A `Filter` row ORs into `unattributed_trigger_zones`/`unattributed_trigger_kinds`; a named row (`SourceOnly`/`Fixed`/`Host`) into `named_unattributed_trigger_kinds` (gate: dispatch.rs:952). Also "or a trigger multiplier" is stale: multipliers go in `trigger_multiplier_sources` (game_state.rs:2083) |
| triggers §4.2 "Why those four, and why four is enough" (1714) | sufficient **for the origins TR-1 ships** — `TriggerOrigin` has one built arm; the two unbuilt ones are the gate's next legs, the delayed registry (TR-3, §4.6) and the state check (TR-6, §4.5) | types/triggers.rs:1608 (TriggerOrigin), :1627 (`InherentAbility`); engine/triggers/delayed.rs:305 (`detect_delayed`, called at engine/triggers/dispatch.rs:722); engine/triggers/state.rs:96, :107, :177 | together they are sufficient for an object's triggers. `TriggerOrigin` has two built arms, `Object` and `Delayed`; `Rule` is declared over an uninhabited `InherentAbility`, no trigger until the monarch (Phase 9). The delayed registry is the dispatch's own leg beside the gate (TR-3a, §4.6, `detect_delayed`). The state check (TR-6, §4.5) has no event and no window; it reads the gate's three legs through `EventKind::BoardState`, a kind no record carries |
| triggers §4.3 Look-back (1774) | the "if" clause reads the frame where it names the object (persist's "if it had no -1/-1 counters", off `Status.counters`) | types/triggers.rs:1976 (`Lki::counters`); engine/layers/condition.rs:176 (`lki.counters_of`) | (persist's "if it had no -1/-1 counters", off the `Lki`'s `counters_of(kind)`, its `PermanentFacts.counters`): `Condition::CounterCount` at engine/layers/condition.rs:176, delegating to `Lki::counters`; `PermanentFacts` replaced the design's `Status` at TR-4a |
| triggers §4.4 Multiplicity, per-subject bullet (~1866-1872) | "The damage arm is the next to need it": "one or more creatures you control deal combat damage to a player" triggers once per player dealt combat damage at once ... "No registered card reads it yet; the first one does." | types/triggers.rs:1149-1151 (`supports_once_per_subject`), :1194 (`occurrence_subject`, keyed on the recipient) | The damage arm carries it too since TR-5d (`supports_once_per_subject`), keyed on the recipient (`occurrence_subject`): "is dealt excess damage" reads it once per permanent; "whenever one or more creatures you control deal combat damage to a player" reads it the same way, once per player, its subject the player (Pyrewild Shaman's and Frostcliff Siege's rulings; 32 cards, Scryfall `o:"one or more creatures you control deal combat damage to a player"`). No registered card reads either yet |
| triggers §4.5 State triggers, 'One-shot' paragraph (1926-1930); §7 table row (3075) | a triggered state trigger is in `state_triggers_armed_off: IdSet<AbilityIdentity>` until its stack object leaves; `GameState::trigger_left_stack(identity)` is the one writer that re-arms it; §7 row lists both as field and writer | engine/triggers/state.rs:177 (`state_trigger_has_not_left_the_stack`) | §4.5: a state trigger that has triggered does not trigger again while an instance waits in the pending queue or is on the stack; the check reads both (`state_trigger_has_not_left_the_stack`, engine/triggers/state.rs) and keeps no set, so every way of leaving, CR 603.3d's removal included, is the answer changing; keep Immortal Coil, Emperor Crocodile. §7 row: Field "none materialized: read off `pending_triggers` and the stack"; Writer "none of its own (the queue's and the stack's)"; Readers "the state check". Check "the four sets" after the table |
| triggers §4.6 Delayed and reflexive (2019-2021) | 603.7c is the `refs` list: "exile it at the beginning of the next end step" names an `ObjectRef { id, zone_change_epoch }` and finds nothing if the epoch moved (CR 400.7) | types/triggers.rs:1796 (`referred: Referred`), :1837-1846 (RememberedObject) | 603.7c is the entry's `referred` (`Effect::Remember`, §3.9): "exile it at the beginning of the next end step" names a `RememberedObject` (an `ObjectRef` and the move that ended it) and finds nothing once that object has moved (CR 400.7) |
| triggers §6.3 The bound facts, LKI, and CR 603.6's "unable to be found" (2973-2975) | `EffectRecipient::TriggeringObject` resolves against `ObjectRef`: the object at that id whose `zone_change_epoch` still equals the binding's | types/effects.rs:883 (`Triggering(EventRole)`; the cited :706 is `Antecedent::Triggering`); engine/triggers/binding.rs:148-164 (`bound_objects`) | `EffectRecipient::Triggering(role)` resolves through `GameState::bound_objects`: each `ObjectRef` the binding took at dispatch, found while its `zone_change_epoch` still matches, and for a zone change's trigger only in a public zone (CR 400.7e) |
| triggers §7 The trackers, history row (3072) | "this turn" quantities, "last turn", "your last turn", "this game" \| `PlayerHistory`'s two rows, total and snapshot (§3.10) \| the dispatcher, record by record | state/history.rs:24-36 (PlayerHistory), :113-139 (GameHistory); state/history_log.rs:39 (HistoryRow); engine/triggers/history.rs:75 (`advance_history`) | \| ... \| the history log's rows (`GameHistory.log: HistoryLog`, §3.10), summed over a span; `PlayerHistory` keeps "your last turn", the draw step's count, and a tally of the newest turn's placed facts (casts, draws, life-gain and life-loss events) \| `engine::triggers::history` (`advance_history`), record by record as each event ends: at the window's dispatch, or for a drawn card as its draw ends \| `Condition::ThisTurn/LastTurn/SinceYourLastTurn/ThisGame`, `FirstTimeEachTurn` \| |
| triggers §7 The trackers, 603.7h row (3076) | `GameState.resolutions_this_turn`, keyed by the ability (§6.5) | state/history.rs:139; state/game_state.rs:725 | `GameHistory.resolutions_this_turn` (on `GameState.history`), keyed by `(ObjectRef, AbilityId)`. Same stale path in §6.5 at 3003 |
| triggers §7 The trackers, delayed-trigger refs row (3080) | \| the object a delayed trigger refers to (603.7c) \| `DelayedTrigger.refs: Vec<ObjectRef>` \| the producer, from the records its instruction performed (§3.9) \| the delayed check, the resolution \| | types/triggers.rs:1745-1796 (DelayedTrigger, `referred: Referred`), :1832-1846 (Referred, RememberedObject); engine/triggers/delayed.rs:218-228 (`note_delayed_source_left`); engine/resolve.rs:508, :921 | \| the objects and players a delayed trigger refers to (603.7c) \| `DelayedTrigger.referred: Referred { objects: Vec<RememberedObject>, players }`, each object an `ObjectRef` with `left_at`, the record of the move that ended it (CR 400.7) \| `Effect::Remember`, from the records its creator's instruction performed (§3.9), handed to `create_delayed_trigger`; each `left_at` by the movers (`note_delayed_source_left`) \| the delayed check, the resolution \|. Also the `DelayedTrigger.created` row (~3084) is stale: code has `created_at`/`created_in` |

**`setup-`, `permission-` and `mana-architecture.md`** (15)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| mana §3.1 "The mana check runs last" (228-231) | "`can_cast` asks timing, targets and the other costs before mana; `can_afford_ability_costs` walks the printed cost list, mana first on most cards, and asks targets after." | oracle/mana_helpers.rs:329-358 (`can_activate_as_its_controller`: each non-mana cost, then targets, then `mana_payable` last); no `can_afford_ability_costs` anywhere in the tree | `can_cast` asks timing, targets and the other costs before mana, and since MA-1 so does `can_activate` (through `can_activate_as_its_controller`): timing, each non-mana cost, targets, then the mana. Before MA-1, `can_afford_ability_costs` walked the printed cost list, mana first on most cards, and asked targets after. |
| mana §3.2 "How it runs" (267-272); §7 decision 3 (823-825) | The inventory keeps one table: for each of the 64 type sets (128 once `{S}` needs a seventh bit), how much mana reaches it, built once with a subset-sum transform (384 additions); §7: "Hall's condition over a 64-entry table". | oracle/mana_supply.rs:1178-1185 (`SplitSupply`, `free: Vec<(ManaTypes, u64)>`), :1276-1287 (`SplitSupply::feasible` sums the list over each subset of `demand.kinds()`); no 64/128-slot array or transform | The inventory keeps its free mana as a list of (type set, amount) (`SplitSupply::free`). For each nonempty subset of the cost's colored pip kinds, `feasible` adds up the mana whose type set meets that subset and compares it with the pips asked, so k distinct colored pip kinds need at most 2^k − 1 comparisons. Decision 3 changes the same way: "Hall's condition over the inventory's (type set, amount) list". |
| mana §3.3 "Sacrifice-fed sources" and "The cost's own consumption" (375-380) | "Today the check counts the sacrifice free (143)"; Chainbreaker with Citanul Hierophants' grant cannot tap itself for its own `{3}`; "today's greedy counts it, and it is among the probe's 44." | oracle/mana_supply.rs:158-160 (an outlet used once per permanent it can sacrifice), :696-703 (`ReservedByCosts::fodder_left` takes the cost's own sacrifices out first), :313-316 (`covers` reserves the action's own costs); no `find_mana_sources` in the tree | Before MA-1 the greedy counted the sacrifice free (143); MA-1's check takes the cost's own sacrifices out of an outlet's fodder, and a self-sacrificing source that the cost's own sacrifice must take is still read loosely (`codebase-state.md` item 213). ... Before MA-1 the greedy counted it, and it was among the probe's 44 (§2.3); MA-1's check leaves it out. |
| mana §3.6 Item 33's seam (436-440, 445-447); §4 families table, "Restricted mana" row (732) | An entry carries its restriction and the check takes the purpose; refused entries leave the table, and a second table is built when the inventory holds any; "What this design builds: the entry's restriction and the purpose argument"; §4: "the seam in MA-1". | oracle/mana_supply.rs:152-156 (`SupplyEntry { permanent, ways }`, no restriction), :313 (`ManaSupply::covers(&self, game, cost, non_mana)`, no purpose), :797-798 (restricted mana "is not counted"), :1715 (test `restricted_mana_is_not_counted`) | MA-1 built neither a restriction on an entry nor a purpose argument: the inventory does not count restricted mana (`ManaOutput::special`) and reads only the pool's unrestricted mana, so never offers a payment needing it. The entry's restriction, the purpose argument on `covers` and the per-purpose table come with item 33's build (`roadmap-v2.md` B9). §4 row's Build cell: "item 33's build (B9); MA-1 leaves restricted mana uncounted". |
| mana §3.6 "The pool" (442-444) | "**The pool** is read through one function, today `ManaPool::available()`. Item 33's per-unit record replaces it with the units the purpose may spend" | oracle/mana_supply.rs:231 (`ManaSupply::read`: `state.mana_pool.unrestricted_by_type()`); types/mana.rs:868 (`fn unrestricted_by_type`); `.available()` left only at engine/costs.rs:296 and a test | **The pool** is read through one function, today `ManaPool::unrestricted_by_type()`, which leaves restricted mana out (the inventory's pool entry and `remaining_cost_after_pool` both call it; the payment's generic split still reads `available()`). Item 33's per-unit record replaces it with the units the purpose may spend, which `amount_for` already computes for the special atoms. |
| mana §3.13 The why of the three mana prompts, `ManaAbilityWindow` bullet (694-695) | "**`ManaAbilityWindow`:** the refusals are the inventory's, one `CannotPay` per mana ability it did not make an entry of, saying why." | ui/why.rs:683-704 (`window_refusals`: `can_pay_costs(..).err()` on each mana ability the window's options leave out; never reads `ManaSupply`), :708-713 (`window_offers`) | **`ManaAbilityWindow`:** the refusals are the window's. For each of the source's mana abilities the window does not offer (its options come from `ManaAbilityWindowOffer`), the `CannotPay` its costs return when they cannot be paid, saying why. A permanent whose abilities can't be activated at all gets that one refusal instead (`ui/why.rs`' `window_refusals`). |
| permission §2 What the engine has (93-94) | "`StackEntry.cast_from` records the zone a spell was cast from (CR 903.8's tax reads it)." | state/game_state.rs:114-118 (doc comment: CR 903.8 commander tax "neither implemented yet"), field at :124; the reader is engine/stack.rs:69 (`ResolvingObject.cast` -> `CastFacts`); 903.8 appears in src only in comments, now and at 0481f00 | `StackEntry.cast_from` records the zone a spell was cast from. The resolution carries it into a permanent's `CastFacts` (`engine/stack.rs`), and the trigger history's `RowFact::Cast` reads it (`engine/triggers/history.rs`). CR 903.8's tax, which B2 builds, will read it. |
| permission §4 Seams, "Item 162" bullet (162-164) | "Item 211's mana abilities at a priority question (CR 605.3a) are item 162's offer, not this document's." | plans/codebase-state.md:8360-8372 (item 211 slotted to MA-6, shape C, decided by the owner 2026-10-05 at #224's review); still unbuilt: oracle/mana_helpers.rs:236-239 (`CannotActivate::ManaAbility`), :300 (`can_begin_to_activate`) | Item 211's mana abilities at a priority question (CR 605.3a) are MA-6's (`mana-architecture.md` §3.11's shape C: one "add mana" entry for a seat that asks), not this document's. |
| permission §6 Sizing and the phase plan, "Slot" (230-233) | PM-0 beside RS-2, which is "beside A, after A6c's item 10" (`roadmap-v2.md` §3a); the first build lands before B2, "which comes first after triggers". | plans/roadmap-v2.md:260 ("Nothing runs beside A any more."), :277-283 (route: A6s, RS-2, PM-0, PM-1, B2); CLAUDE.md "Critical path to v1" (A6s; RS-2; PM-0; PM-1; B2) | **Slot** (`roadmap-v2.md` §3a, "From triggers to Phase 8"; `CLAUDE.md` owns the order): after triggers, the route review A6s comes first. Then RS-2, with PM-0 designing its reasons beside it so each check's reasons are designed once (the owner, 2026-10-05), and a card's characteristic sets designed beside PM-0. Then PM-1, CR 903.8's command zone, which B2 needs, then B2. PM's later builds (flashback and the other cast-from-elsewhere families) have their own step, after B13 and before CV-5. |
| setup §2 What a scenario is, "Plain state is written" (177-179) | "`PlayerHistory` is materialized, and 'nothing derives these from the event stream' (`state/history.rs`), so writing a count is like writing a life total." | state/history.rs:3-8 (module doc: every count is a sum over the history log's rows), :24-30 (`PlayerHistory` holds `own_turn`, `previous_own_turn` and the draw-step draws, no counts); state/history_log.rs:95 (`RowFact::Stated`) | A player's history is such a count, a sum over the history log's rows (`state/history_log.rs`), which one writer, `engine::triggers::history`, writes as each event ends. A scenario writes each count as a `RowFact::Stated` row, that player's on the span's turn, so writing a count is like writing a life total. |
| setup §3.1 What a scenario's permanent gets, "One road" (250-253) | "`place_on_battlefield`, the `EnterBattlefield` performer, splits at the seam between state and event"; its state half (entity, counters' 613.7c timestamps, memo bump, `register_static_effects`) is the door both use; the performer is that half plus the announcement. | state/game_state.rs:1563 (`GameState::perform_entry`, calls `make_permanent` at :1570), :1666 (`make_permanent`); test_support.rs:927 (`place_on_battlefield`, a test helper calling `game.perform_entry(.., EventCause::Harness)` at :933) | `GameState::perform_entry`, the `EnterBattlefield` performer, splits at the seam between state and event. Its state half, `make_permanent` (the entity, the counters' CR 613.7c timestamps, the memo bump, `register_static_effects`), is the door both use. The construction door, `GameState::create_on_battlefield` (`engine/zones.rs`), is that half alone. The performer is that half, then what only play writes (the announced controller, CR 508.4's entering attacking), then the announcement. `test_support::place_on_battlefield` is a test helper over `perform_entry`. (Also stale, same name, at line 2688 in SU-1's "What shipped".) |
| setup §3.1 "Every other write goes through the door play uses" (273-274) | "The turn goes through `set_turn_position`, `begin_turn` and `begin_turn_history`, the one writers." | state/game_state.rs:1116 (`GameState::begin_turn`, which calls `self.history.turn_began()` and `PlayerHistory::record_own_turn`); no `begin_turn_history` anywhere | The turn goes through `set_turn_position` and `begin_turn`, the one writers; `begin_turn` also begins the turn's history (`GameHistory::turn_began`, `PlayerHistory::record_own_turn`). |
| setup §4 The format, names, and the writer, "Why these characters" (371-373) | "a counter kind as devgui's `counter_label` prints it, a table the build moves into the engine so the parser, the writer and the window read one copy." | types/effects.rs:1389-1394 (`CounterType::name`, "One table for everything that prints or parses a kind"), :1423 (`CounterType::named`); devgui/src/editor.rs:1348 and devgui/src/snapshot.rs:184 read `kind.name()`; no `counter_label` anywhere | a counter kind as `CounterType::name` prints it (`types/effects.rs`), the one table the parser (`CounterType::named`), the writer and the window read. |
| setup §7c The why panel, "The rules pass", Events bullet (2016-2019) | "**CR 704.3** watches every state-based death, and the cause names its rule (`ZoneChangeCause::DestroyedBySba` is 704.5g, `ZeroToughness` is 704.5f)." | events/cause.rs:43 (`EventCause::StateBasedAction(StateBasedAction)`), :150-181 (`enum StateBasedAction`: `ZeroToughness` for 704.5f, `LethalDamage` for 704.5g and 704.5h, at :150-163); types/zones.rs:227, :241; no `ZoneChangeCause` or `DestroyedBySba` in src or tests | **CR 704.3** watches every state-based death, and the cause names its rule (`EventCause::StateBasedAction(StateBasedAction::LethalDamage)` is 704.5g and 704.5h, with act `Destroyed`; `StateBasedAction::ZeroToughness` is 704.5f, with act `Put`). |
| setup §7c.2 The surfaces, sketched (2303-2305) | "// As built, `Ok` holds the mana sources it would tap, which `castable_spells` returns. pub fn can_cast(game, player, card) -> Result<Vec<ManaSource>, CannotCast>;" | oracle/mana_helpers.rs:104 (`pub fn can_cast(..) -> Result<(), CannotCast>`), :174 (`pub fn castable_spells(..) -> Vec<ObjectId>`) | // oracle (SU-7): one check for the enumeration and the enforcement. As built, // `Ok` carries nothing and `castable_spells` returns the castable cards' ids. pub fn can_cast(game: &GameState, player: PlayerId, card: ObjectId) -> Result<(), CannotCast>; |

**`clone-architecture.md`** (7)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| clone §0 Where this sits (43-46) | "Today a clone deep-copies every object: a permanent's 400 bytes, a stack entry's 408 bytes and its heap, and a waiting trigger's 264 bytes and its three heap blocks." | state/shared_stores.rs:166-170 (ObjectTable::clone copies the chunk pointers), :350-354 (SharedList::clone); state/game_state.rs:320-343,705 | Before CL-1 a clone deep-copied every object (a permanent's 400 bytes, a stack entry's 408 bytes and its heap, a waiting trigger's 264 bytes and its three heap blocks). Since CL-1 (§7) a clone copies one pointer per chunk of 32. A fork's first write into a chunk copies that chunk, and a permanent's state, a stack entry or a waiting trigger, each behind its own Arc, is copied alone and only when written. |
| clone §1 What a clone copies today / §1.1 The stores (58-75) | §1 "today": `objects` is `FitOnClone<IdMap<ObjectId, GameObject>>`, `stack`/`pending_triggers` are `Vec`s, memo frames a `RefCell<IdMap<...>>`, "never evicted"; `FitOnClone` (`types/ids.rs:450`) is item 183's half fix; the history log is "the tree's one store shared in chunks" | state/game_state.rs:320,327,329,343,705 (ObjectTable/SharedList fields); state/layer_memo.rs:37-46 and :272-273 (frames: RefCell<ObjectTable<StoredFrame>>, fn forget); state/game_state.rs:2523-2525 (remove_object calls layer_memo.forget) | Retitle §1 "What a clone copied before CL-1 (on `b86c59c1`)". Add under §1.1: since CL-1 (§7) all six are `state::shared_stores` types. `objects`, `battlefield`, `stack_entries` and the memo's frames (`RefCell<ObjectTable<StoredFrame>>`) are `ObjectTable`s, and `stack` and `pending_triggers` are `SharedList`s. `remove_object` drops a removed object's frame (`LayerMemo::forget`). `FitOnClone` is gone (decision 4), so the history log is no longer the tree's one store shared in chunks. |
| clone §1.3 The battlefield's 31 engine writers, by shape (110-124) | `aura_put_into_graveyard_by_sba` (`engine/triggers/dispatch.rs:852`) and `for_each_lki_keeper` (`:1040`) "take `&mut` of every waiting trigger and every stack entry, whether or not they write one." | engine/triggers/dispatch.rs:879-913 (aura_put_into_graveyard_by_sba reads, then writes only the matches), :1104 (lki_keepers), :1128 (kept_lki_slot), :1199 (hand_over_lki); for_each_lki_keeper is gone; engine/sba.rs:197-207; engine/turns.rs:408-421; engine/triggers/control.rs:111 (settle_announced_controllers, not :98) | "The stack side had shape 4 too, before CL-1": both took `&mut` of every waiting trigger and stack entry. Since CL-1 each reads first and writes only what matches (decision 3). The aura check (`engine/triggers/dispatch.rs:879`) collects its matches, then writes those. `for_each_lki_keeper` is gone. Its walk is now `lki_keepers` (`:1104`, by `&`), read by `objects_lki_is_kept_for` (`:1071`) and `hand_over_lki` (`:1199`), which writes only matched keepers through `kept_lki_slot` (`:1128`). The battlefield's four sweeps also read first: `engine/sba.rs:201`, `engine/turns.rs:411`, `engine/triggers/control.rs:56`, `:111`. |
| clone §3 Decision 1, row C Upkeep (337) | C's "outer vector spans every id minted, 8 bytes per 32 ids: 2 KB after the 8,000-trigger wipe, 250 KB after a million ids" | state/shared_stores.rs:229-244 (ObjectTable::remove drops an emptied chunk and pops every chunk past the last live one) | The outer vector spans the ids up to the highest live one, 8 bytes per 32 ids (an emptied chunk is dropped, and the trailing ones with it): 2 KB on the 8,000-trigger wipe, 250 KB if an object a million ids in is live. |
| clone §3 Decision 1, "How an entry is held, under C" (341-345) | Small entries with no heap of their own sit inline in the chunk: "an object (48 B) and a memo frame (24 B), whose chunk copy is 1.5 KB or less." | state/layer_memo.rs:75-78 (StoredFrame { epoch: u64, frame: Arc<EffectiveCharacteristics> }); state/shared_stores.rs:128-131 ("The layer memo's frame: an epoch and a pointer, 16 bytes") | "an object (48 B) and a memo frame (16 B: an epoch and a pointer, with no key, since a chunk's slot is the id), whose chunk copy is 1.5 KB or less" |
| clone §Decision 7, rule 3 (575-577) | "A fork iterates as its parent does. Today a `FitOnClone` rebuild gives the fork a different hash order from its parent's. Nothing may observe it, but C removes the possibility." | state/shared_stores.rs:166-170 (a clone copies the chunk vector as is; no rebuild); FitOnClone absent from mtgsim/src | "A fork iterates as its parent does." Before CL-1 a `FitOnClone` rebuild gave the fork a different hash order from its parent's. CL-1's tables walk in id order and a clone copies them as they are, so the possibility is gone. |
| clone §7 CL-1, "Where it built otherwise than designed" (667) | "The holding is a type parameter, so a typed read (`&PermanentState`) and the tests' field writes after `get_mut` kept their call sites." | state/shared_stores.rs:53-56 (trait ForkCopy { type Held: Holding<Item = Self>; }), :161-163 (pub struct ObjectTable<T: ForkCopy> { chunks: Vec<Option<Arc<TableChunk<T::Held>>>>, ... }) | The holding is the item type's associated type (`ForkCopy::Held`), so a table is generic over the item alone, and a typed read (`&PermanentState`) and the tests' field writes after `get_mut` kept their call sites. |

**`codebase-state.md`** (12)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| codebase-state §TL;DR, "Code size" (10) | "**Code size:** 62,241 lines of Rust across 118 `src/` files, plus 32,712 in `tests/`. 1,537 tests, 0 warnings." | the `src/` tree at c91657e1: 190 .rs files, 101,656 lines; `tests/`: 111 files, 71,077 lines; state-of-play.md's Counts table: 2,754 `#[test]` | **Code size** (at c91657e1, 2026-10-22): 101,656 lines of Rust across 190 `src/` files, plus 71,077 across 111 files in `tests/`; the test count is `state-of-play.md`'s generated `#[test]` count (2,754 at c91657e1), 0 warnings. (Same line: `performance`, 124 cards; `stress`, every registered card, 215.) "0 warnings" was not rechecked, because cargo was not allowed. |
| codebase-state §TL;DR, replacement-effects bullet (15) | "Every observable mutation is a `GameAction` proposal (22 kinds) through one chokepoint" | engine/actions.rs:138-637 (`pub enum GameAction`, 26 variants; Unattach, TokenCeasesToExist, ShuffleLibrary and GameEndsInADraw were added after 1b34975e) | "Every observable mutation is a `GameAction` proposal (26 kinds) through one chokepoint". Line 147's "22 kinds" is stale in the same way. |
| codebase-state §TL;DR, "What is next on the spine" (21; both refuters find it on 20) | "**What is next on the spine:** the triggers architecture doc and critical-path item 6 ... item 139 with the fork test, A4b's rulings ledger and A4c's trace sink remain" | plans/roadmap-v2.md:238-258 (A6 ✅ 2026-09-24 through A6d ✅ 2026-10-22; A6e, A6h, A6f, A6s open); state/trace.rs:71 (`TraceSink`) | "**What is next on the spine:** item 6's close audit (A6e), the docs audit (A6h) and the information-model design (A6f), then the route review A6s (`roadmap-v2.md` §3a A)." The same line's "RS-2 and CV-2 beside" is stale too (CV-2a/b landed, roadmap-v2.md:260). |
| codebase-state §TL;DR, architecture-docs bullet (22) | "**Eight architecture docs own their subsystems:** `layers-architecture.md`, ..., `setup-architecture.md` and `permission-architecture.md`" | plans/mana-architecture.md, plans/clone-architecture.md (`ls plans/*-architecture.md` lists ten); CLAUDE.md's authority row lists mana and clone | "**Ten architecture docs own their subsystems:** `layers-architecture.md`, `replacement-architecture.md`, `cant-effects-architecture.md`, `copy-effects-architecture.md`, `cost-architecture.md`, `triggers-architecture.md`, `setup-architecture.md`, `permission-architecture.md`, `mana-architecture.md` and `clone-architecture.md`" |
| codebase-state §CR coverage table, CR 3 / 209 row (93) | "\| 209 \| Loyalty (for PW) \| ✅ ETB counter init + 0-loyalty SBA, ... \| `engine/sba.rs` (704.5i), `state/game_state.rs` (init_etb_counters), `engine/actions.rs` \|" | engine/layers/intrinsic.rs:1-41 (`add_intrinsic_entry_abilities`, which makes CR 306.5b's "enters with" loyalty a synthesized ability), called at engine/layers/compute.rs:481 and engine/layers/board.rs:1774; `init_etb_counters` has 0 hits | "\| 209 \| Loyalty (for PW) \| ✅ CR 306.5b's "enters with" loyalty is an ability the walk gives every planeswalker frame, gathered and ordered by CR 616.1 + 0-loyalty SBA ... \| `engine/sba.rs` (704.5i), `engine/layers/intrinsic.rs`, `engine/actions.rs` \|" |
| codebase-state §item 40 (decision sites), Violator 1 (2085) | "**Violator 1 — `pipeline.rs:101/115/119`, the CR 614.5 sets.** `applied`, `declined` and `exempt_applied` live in `apply_replacements`' frame" | engine/replacement/pipeline.rs:401 (`applied`), :405 (`declined`), :409 (`exempt_applied`), inside `apply_replacements` (:380) | "**Violator 1 — `pipeline.rs:401/405/409`, the CR 614.5 sets.**" Also stale: the next sentence's `ask_apply_optional_replacement` is now `ask_take_or_leave` with `ChoiceKind::ApplyOptionalReplacement` (pipeline.rs:571), and engine-map.md:288/310 still cite :339/:343. |
| codebase-state §item 40, Violator 2 (2096) | "The four `rollback_cast_to_hand` call sites are the evidence" | engine/put_on_stack.rs:163, 197, 215, 270, 280, 292 | "The six `rollback_cast_to_hand` call sites are the evidence". The Sized paragraph's "the four rewind sites" (2117) is stale in the same way. |
| codebase-state §Deferred Migrations, item 75 (2716-2717; the 2026-09-14 LJ re-derivation, 2729-2737) | "The literal *reachability* sentence survives on a technicality — an object a row reaches is a **member**, so nothing puts a row on a non-member"; `compute_non_member` "is now the walk of an object **no row names**" | engine/layers/board.rs:1819-1822 (`PassMembership::LeftOut`), :1882 (`left_out_zones`); engine/layers/compute.rs:332-341, :427-447 (applies each noted row's modification) | Add LL's re-derivation (2026-09-25): a hand or library card that a row reaches is left out of the pass (`board::left_out_zones`). It stays a member only if the guard keeps its zone in (a dynamic row, or a same-layer pair `Channels` cannot rule out), a `Fixed` row names it, or it joins as a source. `compute_non_member` walks a left-out card and applies each reaching row's `RowNote` (`PassMembership::LeftOut`), so rows do reach non-members. The opening's "applies CDAs and no rows at all" holds only for `PassMembership::NonMember`. A spell a STACK row reaches is still a member. Source 2's printed leg still rests on the two summary flags. |
| codebase-state §"Found by the post-RE audit, pass 3" (heading 6057), the events sentence (6564-6566) | "a visual effect keys on the performed stream — `ZoneChange` with its catchall-free `ZoneChangeCause`, `PlayerLost`, `Tapped`, `ManaAdded`" | events/event.rs:28-38 (`ZoneChange.act: ZoneChangeAct`), :796-804 (`EventStamp.cause: EventCause`); `ZoneChangeCause` has 0 hits | "a visual effect keys on the performed stream — `ZoneChange` with its catchall-free `ZoneChangeAct` and its stamp's `EventCause`, `PlayerLost`, `Tapped`, `ManaAdded`" |
| codebase-state §item 148, "Exile is the one zone the Colossus family's tests do not move from" (6853-6870) | "no `ZoneChangeCause` names that move. Pull from Eternity ... is the card, and the no-catchall rule makes it bring its cause"; **Slotted:** with that card, "which brings its `ZoneChangeCause`" | types/zones.rs:234-241 (`ZoneChangeAct::Put`); events/cause.rs:26 (`EventCause::Effect`); effects.rs:1759 (`Primitive`, which has no put-into-graveyard verb) | The reason is a missing card. Since RH-1 the move needs no new act or cause: it is `ZoneChangeAct::Put` with `EventCause::Effect`. The card brings only a `Primitive` that puts an object into its owner's graveyard, and none exists today. **Sized:** the card, its `Primitive` arm, and one Colossus-in-exile test. **Slotted:** with Pull from Eternity, which brings that `Primitive` and the Colossus test. The same stale premise is in mtgsim/tests/phase_rf_integration_test.rs:325-328. |
| codebase-state §item 162, the tap solver's two halves, opening paragraph (7285-7288) | "The second customer is `castable_spells`' affordability, a heuristic overapproximation today (`find_mana_sources`), whose over-offers are the CR 732.1 rewinds A4h made the retry loop state-dependent for (item 139)" | oracle/mana_helpers.rs:23-31 (`mana_payable`: the pool, then `ManaSupply::covers`), :167, :174-187 (`castable_spells`); oracle/mana_supply.rs:11-14; `find_mana_sources` has 0 hits | "The second customer is `castable_spells`' affordability, exact on the registered cards since MA-1 (#225): `oracle::mana_supply`'s Hall's-condition check (`ManaSupply::covers`, through `mana_payable`) replaced `find_mana_sources`' greedy overapproximation, whose over-offers were the CR 732.1 rewinds A4h made the retry loop state-dependent for (item 139) and the enumeration lever 4 priced at 19.2% (item 138)." Item 213 owns the shapes the check still reads one way. The Sized paragraph's "`find_mana_sources` refuses hybrid" (~7305) is stale too. |
| codebase-state §Deferred Migrations, item 185 ("The engine's elisions are argued one site at a time") (7786-7793) | "`pipeline::ordering_cannot_change_outcome` spreads its premises over `classify`, `commutes`, `shared_clauses_hold`, `filter_is_mods_invariant`, `EventPattern::reads_the_amount` and `EnterModsTemplate::is_fixed`" | engine/replacement/pipeline.rs:1538 (`struct EntryWrites`, the feeds table), :1664 (`EntryWrites::unmatches`); there is no `filter_is_mods_invariant` in the tree | "... spreads its premises over `classify`, `commutes`, `shared_clauses_hold`, the feeds table (`EntryWrites`, `replacement-architecture.md` §3.5; `filter_is_mods_invariant` until RG), `EventPattern::reads_the_amount` and `EnterModsTemplate::is_fixed` ..." The item's "the predicate's doc comment" is now `EntryWrites`'. |

**`roadmap-v2.md`, `backlog.md`, `engineering-practices.md`, `glossary.md`, `cast-census.md`, `cr-coverage-audit.md`** (16)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| backlog §2.11 Loyalty abilities, Verdict (829-830) | "CR 704.5i's zero-loyalty SBA is implemented and tested (`engine/sba.rs:277`, with `ZoneChangeCause::ZeroLoyalty`)" | engine/sba.rs:373 (`batch.push((sba_put_into_graveyard(id), StateBasedAction::ZeroLoyalty))`); `ZoneChangeCause` has 0 hits | CR 704.5i's zero-loyalty SBA is implemented and tested (`engine/sba.rs`, act `Put`, cause `StateBasedAction::ZeroLoyalty`) |
| backlog §2.22 decorator table, row 6 (1506) | Row 6, "tap solver, oracle half": "nothing — a query: a covering set for `remaining_cost`" … "**the one row with algorithmic legwork**, below; designed in `mana-architecture.md` (#224), MA-1" | oracle/mana_supply.rs:18 ("It answers whether a payment exists, never which") | designed in `mana-architecture.md` (#224): the existence check landed in MA-1 (#225), and the covering set, the solver, is MA-6 (§3.12) |
| backlog §2.22 "Rows 6 and 7, the tap solver, framed" (1629-1636) | "The other customer is **`castable_spells`' affordability**, which is a heuristic overapproximation today (… `find_mana_sources`)" … "the harness prints no rewind count today" | oracle/mana_helpers.rs:160-169; oracle/mana_supply.rs:11-14 | The other customer was **`castable_spells`' affordability**, a heuristic overapproximation (`find_mana_sources`) until MA-1 (#225) replaced it with `oracle::mana_supply`'s exact check, Hall's condition (`mana-architecture.md` §3.2), exact for every registered card; the shapes it still reads one way are `codebase-state.md` item 213. The rewinds it removed are what `fuzz_games` prints as "Actions reversed" (22.4 a game fell to 0.0). What remains for the solver is the payment itself, MA-6 (§3.12). |
| backlog §2.22 "This is the one row with algorithmic legwork" (1661-1667) | "`find_mana_sources` returns `None` on any hybrid, Phyrexian or X symbol — the heuristic the matching replaces is a one-unit-per-source greedy that gives up on half the symbol alphabet." | oracle/mana_supply.rs:11-16 (`covers`: Gale/Hall condition over entries with amounts); types/mana.rs:185-196 | Before MA-1 (#225), `find_mana_sources` was a one-unit-per-source greedy that returned `None` on any hybrid, Phyrexian or X symbol. MA-1 replaced it with `ManaSupply::covers`, exact for existence (Gale's/Hall's condition over entries with amounts), but it still refuses what `ManaCost::demand` cannot tally: hybrid and Phyrexian symbols and an activated ability's {X} (CP-1's), {S} (item 33's); a spell's X is expanded first by `with_x_as`. It answers whether a payment exists, never which; the solver that returns a covering set is MA-6's. |
| backlog §2.22 row 9, the reversal (item 72) (1705-1709) | "*Reverse all* is, and cheaply: `GameState` is `Clone` and a no-log clone is 4–6 µs (item 42's measurement)" … restores it, "truncating `events` to the snapshot's length." | state/game_state.rs:778-784 (`pub events: EventWindow`, flushed when the outermost dispatch returns); events/recorder.rs:10-14 (a RecorderHandle's Clone is the fork; the clone's stream is its own) | … `GameState` is `Clone` and a clone is a few microseconds (CL-1's reading, `clone-architecture.md`: 1.8 µs between turns at worst, 4.5–4.7 µs under an 8,000-trigger stack, against floor 2's 10 µs in `engineering-practices.md#the-floors`), so the engine takes one at the window's first activation and, if the player takes CR 732.1's offer at the rewind, restores it. Nothing needs truncating: the clone carries `events`, a window flushed when the outermost dispatch returns, and its recorder handle is a fork, so the restored state's stream ends at the snapshot. (The refuters disagree on where 1.8 µs is read: engineering-practices:728 §3.1 vs clone-architecture:684.) |
| cast-census §5 Mana abilities, the MANA table's last column and the prose (368-384, 410-414) | "The check misreads both: it counts Ironworks' sacrifice as free and Sol Ring's `{C}{C}` as one mana. The last column is item 162's (`oracle/mana_helpers.rs::find_mana_sources`, `available_mana_sources`)." | oracle/mana_supply.rs:121-133, 243-267 (entries carry each way's mana; sacrifices and fodder via `sacrifice_candidates` and `SacrificeOutlet`) | The check read both wrongly before MA-1: it counted Ironworks' sacrifice as free and Sol Ring's `{C}{C}` as one mana. The last column is that check (`find_mana_sources`, gone since #225). `oracle::mana_supply` counts Sol Ring's two, and Ironworks once per artifact it can sacrifice (`mana-architecture.md` §3.1, §3.3). The column text lives in cast-census.py:538-576: fix it there, then regenerate. |
| cast-census §8 mana symbols, the prose after the SYMBOLS block (506-507) | "Blaze and Protean Hydra are the registered X spells." | cards/registry.rs:1035 (`registry.register("Goblin Negotiation", phase_tr5d_cards::goblin_negotiation)`); cards/phase_tr5d_cards.rs:107-111 (mana cost {X}{R}{R}) | Blaze, Protean Hydra and Goblin Negotiation are the registered X spells. |
| cr-coverage-audit §4 The sweep, "Rows added by the re-sweep" table, rows `ReplacementEffectRegistry` and `TargetInstance` (232, 244) | 244: "`TargetInstance` *(pass 3)* \| a target that left and came back (CR 400.7) \| main item 10's `target_epochs`"; 232: "a chosen source (CR 609.7a); targets held across a move (CR 400.7) \| feature; main item 10" | engine/targeting.rs:27 (`pub chosen: Vec<TargetRef>`), :41 (`enum TargetRef { Object(ObjectRef), Player }`); state/replacement_effects.rs:105 (`pub targets: Vec<TargetRef>`); types/replacement.rs:668 (`SourcePattern.object`); no `target_epochs` in src | 244: \| `TargetInstance` *(pass 3)* \| a target that left and came back (CR 400.7) \| **built** — `TargetRef` carries the epoch, compared at CR 608.2b (main item 10, closed by CV-1b, #230) \|. 232, keeping the CR column: \| `ReplacementEffectRegistry` \| a chosen source (CR 609.7a); targets held across a move (CR 400.7) \| **built** — `targets: Vec<TargetRef>` (main item 10, closed by CV-1b, #230); a chosen source is `SourcePattern.object`, an `ObjectRef` the resolution fills (TR-3c, #239) \|. Line 248 (`RestrictionRegistry`) and §4a 305/347 name item 10 the same way (unchecked). |
| trace-pages.md Trace pages, the page table (1562-1575) | "Three existed when the practice was written down, and they are the template:" over a ten-row table (rc-4b … cv-2b) | plans/traces/a4i-a-target-belongs-to-an-instance.html, ma-1-a-payment-is-a-matching-not-a-count.html, tr-1-a-trigger-is-matched-at-the-close.html, tr-5g-a-multiplier-is-read-where-its-trigger-looks.html (14 pages; the table lists ten) | Add four rows, each with its "What it proves" cell, in landing order: `a4i-a-target-belongs-to-an-instance.html` (A4i) and `tr-1-a-trigger-is-matched-at-the-close.html` (TR-1) after rf; `ma-1-a-payment-is-a-matching-not-a-count.html` (MA-1, six boards) and `tr-5g-a-multiplier-is-read-where-its-trigger-looks.html` (TR-5g) after cv-2b. Line 1562: "Three existed when the practice was written down (RC-4b, CV-1, RC-5); every page since is listed here:". Same early count: "Both existing pages" (1559/1560), "the two above" (1578). (Refuter 2 drafted the four cells: route-and-practice.json, for-a6h#10.) |
| engineering-practices.md#trace-pages Trace pages, "Pinned, never maintained" (1626) | "**Pinned, never maintained.** The first line of the file is an HTML comment naming the commit and the frozen CR version." | plans/traces/*.html line 6 (every page, viewer.html included) | **Pinned, never maintained.** The first comment in the file, the one in its `<head>` right after the charset and viewport `<meta>` lines, names the commit and the frozen CR version. Same paragraph (1631): "a stale page has to say so on its own first line" → "in its own head". |
| glossary **matcher** entry (624-625) | "Besides the replacement pipeline, the only code that may branch on a zone change's `act`." | engine/resolve.rs:2622 (`act: ZoneChangeAct::Discarded`); engine/triggers/history.rs:260 (`act: ZoneChangeAct::Cast \| ZoneChangeAct::Drawn, .. } => {}`) | Besides the replacement pipeline, one of the few places that branch on a zone change's `act`. The others are the history, which drafts no row for a cast or a draw, and a resolution's "discarded this way" count (`AmountExpr::DiscardedThisWay`). (A refuter found the same stale claim at CLAUDE.md:76 and replacement:1070.) |
| roadmap-v2 §3 The spine, item 5 (115) | "**RC — ETB replacements — is next, and is the largest single unlock before triggers: ~1,350 cards**" | codebase-state:15 ("Replacement effects (CR 614–616) — ✅ complete, Phases RA–RE … critical-path item 5 closed with RE-9"); roadmap-v2:636 ("RC 6 ✅") | **RC — ETB replacements — was the largest single unlock before triggers (~1,350 cards, counted 2026-08-31); it landed in six PRs, RC-1–RC-5 with RC-4b (2026-09-01 → 2026-09-03), and item 5 closed with RE-9.** (There is no RC-6.) |
| roadmap-v2 §3a A, row A4d, the Rust notes (241) | "what the borrow checker forced (the accessor pair, `FrameCache`'s overlay in place of a clone, `ActionContext`'s plumbing)" | engine/layers/board.rs:346 (`Board::entity`), :977 (`rows_in_layer`); no `FrameCache` in src | what the borrow checker forced (the accessor pair, `Board`'s look-ahead overlay in place of a clone, `ActionContext`'s plumbing) |
| roadmap-v2 §3a B, row B3 (321) | "the priority loop, turn rotation and APNAP are already modulo `num_players`; CR 800.4 (a player leaves) and 802's defending-player choice are not" | codebase-state:13 ("Not started: CR 802's defending player and CR 800.4f–h's choices by a departed player"); cards/phase_re_cards.rs:151 (RE-6, 800.4j); bin/fuzz_games.rs:466 (CR 800.4a) | the priority loop, turn rotation and APNAP are already modulo `num_players`, and a player leaves (CR 800.4a–e, j, k, m; RE-6, RE-7); CR 800.4f–i (a cost, a choice or the last known information of a departed player) and 802's defending-player choice are not. Also stale: the "Do this" cell's "with a `--players 4` fuzz mode" (RE-6 landed that mode). |
| roadmap-v2 §3a B, row B5 (323) | "RS-2 waits for A4i and for main item 10 with CV-1b (B6), by item 10's own \"before RS-2\"" | roadmap-v2:245 (row A6c ✅ 2026-10-06, PR #230 — CV-1b with item 10) | RS-2's prerequisites, A4i and main item 10 with CV-1b (A6c), have landed, as item 10's own "before RS-2" asked (or drop the sentence: the RS-2 row in "From triggers to Phase 8" already states it) |
| roadmap-v2 §9 Watch items (658-659) | "**Item 6's true size** — the dominant risk on the route. Unsized." / "**§5's two amendments** — advisory until `CLAUDE.md` adopts them." | triggers §12 (TR-1–TR-7; 23 TR-phase headings); roadmap-v2:567 ("Status: adopted 2026-09-04"), :238-254 (A6 through A6d ✅) | Drop both bullets. Item 6's build has landed (A6d, 2026-10-22, roadmap-v2:254); its size lives in §8's Item 6 row and triggers §12, what remains is A6e (:255), and §10 bars state claims here. If one must stay: "**Item 6's close audit** — A6e, about 10–15 PRs, `plans/handoffs/item-6-audit.md`." Both §5 amendments were adopted 2026-09-04 (:567). Separately, §8's "19 since TR-5e's review" now counts 21. |

**Source doc comments** (10)

| Where | The doc says | The code | The correction |
|---|---|---|---|
| engine/actions.rs §`GameAction::EnterBattlefield` doc (383-386) | "`mods` starts as whatever the *rules* say the permanent enters with (`GameState::default_enter_mods`) and accumulates through [`Rewrite::EnterWith`] as CR 616.1f iterates." | engine/actions.rs:2203 (`entry_proposal` seeds `mods: EnterMods::NONE`); engine/layers/intrinsic.rs:1,41 (`add_intrinsic_entry_abilities`: CR 306.5b is a gathered ability); types/replacement.rs:966 (`EnterAsCopy` also writes the mods) | `mods` starts empty (`EnterMods::NONE`, `entry_proposal`) plus what the effect putting the permanent there says ("tapped", "attacking", an Aura's host); it accumulates as CR 616.1f iterates: `EnterWith` merges, `EnterAfterMoving` adds CR 614.13's counters, `Amount` scales counters (CR 122.6), `EnterAsCopy` sets the copy. CR 306.5b's loyalty is a `layers::intrinsic` ability, gathered like any printed "enters with". |
| engine/put_on_stack.rs §`run_mana_ability_window`'s failure branch (690-692) | "Enumeration said this was legal but activation failed — likely staleness or a `can_pay_ability_costs` over-approximation bug." | oracle/mana_supply.rs (`ManaAbilityWindowOffer`, which re-asks each ability's costs through `can_pay_costs`, per mana-architecture §3.8); no `fn can_pay_ability_costs` in src | Enumeration said this was legal but activation failed: likely a stale offer, or an over-approximation in `ManaAbilityWindowOffer::options`'s cost recheck (`can_pay_costs`, or its untapped-and-not-summoning-sick shortcut for a {T}-only cost). |
| engine/triggers/mod.rs §module doc (31-32) | "[`binding`] is what the resolution reads back: the bound facts as indices into the event log, resolved through the matched arm's projections" | types/triggers.rs:1536 (`TriggerBinding.triggering_events: Vec<TriggeringEvent>`, each with its record); engine/triggers/binding.rs:3-6 (records copied at dispatch) | [`binding`] is what the resolution reads back: the bound facts as the records the trigger matched, copied at dispatch, read through the matched arm's projections. |
| events/event.rs §`AppliedReplacement.redirected` field doc (785-787) | "Whether it changed where the move goes: CR 614.6's modified event is then the replacement's, and so is \"who exiled it\"." | ui/display.rs:952 (the only reader of `.redirected`); PR #248 commit 9d4ad49f ("a redirected move's destination is the resolving spell's") | Whether it changed where the move goes, which the log shows. It does not change who sent it there: that is still the record's cause, a redirect included (`replacement-architecture.md` §2b.4). |
| events/event.rs §`EventWindow` doc (897-899) | "What the rules read from the past is materialized instead: the turn summaries (§3.10), and the records a trigger binds (§3.4)." | state/history.rs:3-5 (every count is a sum over the history log's rows); plans/triggers-architecture.md:4005-4006 (TR-5h: `TurnSummary` and the other summaries are gone) | What the rules read from the past is materialized instead: the game's history (`state::history`, §3.10), meaning the history log's rows and each player's own turns, and the records a trigger binds (§3.4). Siblings at events/event.rs:822-883 and types/triggers.rs:164 use the same retired word. |
| events/recorder.rs §module doc (4-6) | "The engine keeps a record only until its last reader has run ([`EventWindow`]), so a game's history is not game state." | state/game_state.rs:719-725 (`pub history: GameHistory`, "What the game remembers"); state/history.rs:10-11 ("[`GameHistory`] holds it all: the log, ...") | The engine keeps a record only until its last reader has run ([`EventWindow`]), so the performed stream is not game state: what a card can ask of the past is `GameState.history` (`state::history`), the history log's rows and the facts kept beside them. |
| types/effects.rs §`EffectRecipient::FilteredPermanents` doc (907-914) | Read by the ETB hook and at resolution by primitives acting on several objects: `CreateReplacement` one row per permanent (CR 615.11), `DealDamage` one batch member each. "All three resolve it the same way" | engine/resolve.rs:354-361 (`Effect::Atom` with `FilteredPermanents` hands every primitive `affected_permanents`), :2415-2430 (`affected_permanents`), :1594 (`GainControl`) | Keep the ETB hook; at **resolution** every primitive receives `affected_permanents` (CR 611.2c's set, fixed as the effect begins): `CreateReplacement` one row each (CR 615.11), `DealDamage` and `Untap` one batch member each, a characteristic or control change writes that fixed set into its rows; `Restrict` keeps the filter, read at each query (CR 611.2c's second sentence). |
| types/replacement.rs §`pub enum Rewrite` doc (840-851) | "The full algebra, each arm with the rule that permits it" (a six-row table) ... "Every arm has a customer, so the enum is the algebra as claimed" | types/replacement.rs:926 `EnterAfterMoving(AuxiliaryMove)` and :966 `EnterAsCopy(EntryCopyTemplate)`, both arms of the enum and both missing from the table | Add the two missing rows: `\| EnterAfterMoving(..) \| 614.13 \|` and `\| EnterAsCopy(..) \| 616.1c, 707.5, 614.1c \|`. |
| types/replacement.rs §`GameActionTemplate::ZoneChangeTo` doc (~1700-1710; also engine/replacement/pipeline.rs:2635 comment, :2645-2646 error text) | "Send the event's object somewhere else instead, keeping its `from` and its `cause`." An entry turned into a zone change "keeps the entry's cause by the same rule, since a return or a land play is read together with `to`." | engine/replacement/pipeline.rs:2626-2643 (substitute keeps `act`); types/zones.rs:234-241 (a return is `Put`) | "keeping its `from` and its `act`" ... "keeps the entry's act by the same rule, since a `Put` or a land play is read together with `to`." pipeline.rs: "with the entry's act"; error: "the entry names no act". |
| types/zones.rs §`ZoneChangeAct` doc, "Three rules" (218-219; also events/event.rs:36-37, types/replacement.rs:248-250) | "Nothing may branch on it outside the replacement pipeline and the trigger matcher. A third reader is a third place for it to drift." | engine/resolve.rs:2619-2623; engine/triggers/history.rs:260-263 | Name the readers it has: the replacement pipeline, the trigger matcher, the history's row writer and `AmountExpr::DiscardedThisWay`; a new reader joins that list in review. |

**Contested: one refuter of two refuted each.** The fix session reads each one
before acting on it.

| Where | The doc says | The proposed correction | The refuter's objection |
|---|---|---|---|
| engine-map §1.2 The walk, row 11 (81) | The move ends every reference to it: "rows that named it pruned, save a permanent spell's under 400.7a and 400.7c" | "(CR 400.7: rows that named it pruned, save a permanent spell's continuous rows under 400.7a; a prevention shield watching the spell is re-pointed at the permanent under 400.7c)" | "Save" means "except", and both exceptions hold for the cited `move_object`: zones.rs:82-95 keeps the 400.7c shield and re-points it, as #239's comment (zones.rs:488-492) frames it. Only pins drifted. |
| engine-map §2.1 The modules and what each owns (126) | "143 files under `mtgsim/src`; `mod.rs` files declare and re-export only" | 190 files, plus rows for the unnamed `scenario/`, `state/decision_log/`, `history_log.rs`, `shared_stores.rs`, `layer_settlement.rs`, `layers/explain.rs`, `mana_supply.rs`, `x_values.rs` and five ui files | The page is pinned to 75b1860, where 143 is exact; §3 reports unnamed files rather than failing them, so the fix is a §3 refresh and re-pin. The list also misses `engine/triggers/control.rs`. |
| engine-map §2.1 The modules and what each owns (131) | The chokepoint row: "`GameAction`, `ActionContext`, `execute_actions`' three phases, `perform_action`'s arms, `change_zone`, entry and token proposals" | Replace "`execute_actions`' three phases" with "a batch's two phases (decide, perform) and its riders after them (`execute_riders`, `resolve_riders`)" | The line names `execute_actions`, which through `execute_batch` still does three steps: decide (actions.rs:1101), perform (:1176), riders (resolve_riders :872, execute_riders :896), matching the project's own "phase 3: RIDERS". |
| engine-map §2.1 The modules and what each owns, engine/triggers row (136) | The row names dispatch.rs, delayed.rs, placement.rs, binding.rs, binding_reads.rs, history.rs and audit.rs, but not state.rs or control.rs | Add "`state.rs` CR 603.8's state check; `control.rs` CR 603.10d's change of control, announced by a sweep" | Every named file still does what it says; state.rs and control.rs are only absent, which §3 reports rather than fails until the next refresh; state.rs is named at line 222. |
| engine-map §2.2 The chokepoint's arms, EnterBattlefield row (184) | "`EnterBattlefield` \| the move (or a token's creation), then `perform_entry`, which announces a `CountersPutOn` for each counter row (CR 122.6) \| —" | Append "an Aura entering attached (`EnterMods::attached_to`) is then attached, with `Attached`" (actions.rs:1826) | Everything it says holds (actions.rs:1795-1821, game_state.rs:1563-1595); the attach proposes nothing, so "—" stands; rewritten in fae1dbb0 after 05d2802d, it is a selective summary: incomplete, not stale. |
| engine-map §2.7 Every CLAUDE.md invariant, where it lives, Lives-at column (291-316) | The Lives-at file:line pointers (compute.rs:94, actions.rs:1153, :1776, phase 3 :1032, game_state.rs:660, ids.rs:268, ...); only 3 of 31 sit at the cited line | Re-cite each at today's line (e.g. compute.rs:103, actions.rs:1457, :2129, game_state.rs:812, ids.rs:310); phase 3 becomes `execute_riders` actions.rs:1268; the can't check is `prohibition` pipeline.rs:456 | The header pins every file:line to 75b1860, where 35 of 38 hold, and §3 reports drift until a refresh re-pins. A refresh should still carry two renames: `prohibition` and `execute_riders`. |
| layers §3.1 `Layer` / `Sublayer` (57-93) | The `Layer` enum lists `Layer1aCopy` and `Layer1bFaceDown` (CR 613.2a/b) as separate sublayers | `Layer1Copy, Layer2Control, …, Layer7dSwitchPT`: one Layer 1 slot until CV-6 splits it; line 93's face-down "Layer-1a" is actually 1b, and unbuilt | §3.1 is the frozen target surface. §7 line 762 already says Layer1Copy collapses both sublayers until CV-6, and copy-effects:1364 plans the split. Lines 93/766's "1a" for face-down is a separate error. |
| layers §3.4 `ObjectSet` (222-247) | Block shows `Filter(SelectionFilter)`, `Fixed(Vec<ObjectId>)`; amendment says "As shipped, the variant is `Filter { filter: ObjectFilter }`" | `ObjectSet { SourceOnly, Filter { filter: ObjectFilter, zones: ZoneSet }, Fixed(Vec<ObjectId>), Host }`; zones from LJ, Host from LH; battlefield_filter/filter_in | The block is the 2026-05-20 design, which the 2026-08-23 amendment marks as unshipped. The amendment was true when written, and zones (§13c decision 3) and Host (LH-1) are recorded elsewhere in the doc. |
| layers §13d decision "No `playable_from` parameter" (2302-2303) | "§11 item 9's sizing and `roadmap-v2.md` A5 both name CR 113.6e's 'any zone from which it could be played or cast'" | "§11 item 9's sizing names CR 113.6e's 'any zone from which it could be played or cast'" (A5 dropped) | It is LK's dated design record (2026-09-14). A5 named 113.6e then; fb0db4ea removed it 09-16. The cited contradictions are records. The fix is wrong too: §11 item 9's sizing never quotes 113.6e. |
| layers §15.2 item 4 (2844-2854, 2878-2880) | `get_effective_abilities` "can keep returning `Vec<AbilityDef>`"; the `EffectId` tiebreak is required because `remove_by_source` uses `swap_remove` | Returns Arc<Vec<AbilityDef>>; rows are sorted (layer, timestamp, id) and removal is an order-preserving retain, so the tiebreak does not exist to repair swap_remove | This closed item is a set of dated blocks, each true when written: the Arc came 09-16, and swap_remove went after the 08-21 update. The 08-23 block supersedes it. The tiebreak still stands for shared timestamps. |
| replacement §2a The shipped types, `GameActionTemplate` row (164; cited as 166) | "Ships `ZoneChangeTo` (three RB customers ...) and `RemoveCountersFromAffected` (two — the shield and stun counters)" | Eight arms: `ZoneChangeTo`, `RemoveCountersFromAffected`, `DrawCards`, `GainLife`, `LoseLife`, `CreateTokens`, `PlayerWins`, `ProduceMana` (RB shipped the first two) | A record of RB: §2a's heading says "every type Phase RB shipped", the row says "three RB customers", later sections record the other six arms; it concedes line 164 was edited 2026-09-24 |
| replacement §3.2 `ReplacementDef`, the (shipped) block (1163–1168) | "`Shield(u64)` survives and belongs to RD. RB ships `Uses { Static, Once }`", under an undated "// (shipped)" block | `Shield(u64)` survived as `NextDamage(u64)`, built in RD-2; the block becomes `// (shipped) pub enum Uses { Static, Once, NextDamage(u64) }` | A record of RB, written once at its landing (f44d7bcb, 2026-08-26); "belongs to RD" was a plan RD carried out, and §3.3 item 4 and RD-2's Shipped line already record `NextDamage` |
| replacement §3.5 The entry state, "One constructor." (1575–1578) | "`PermanentState::entering` is the only function that turns an `EnterMods` into an entity, for the performer and the look-ahead alike ... A new member is a field on `EnterMods`, one line there" | `entering` turns a replacement's half of `EnterMods` (status, counters, copy, edits) into an entity; `attached_to` (303.4f) and `attacking` (508.4) are applied after it by the performer (`perform_entry`), unpredicted by the look-ahead (other refuter: `attached_to` is the `EnterBattlefield` arm's, actions.rs:1826) | The claim is about construction, which holds: `entering` is still the only `PermanentState` constructor, its two callers (lookahead.rs:65, game_state.rs:1674) differ only in timestamps; attacking and attachment come after |
| replacement §6 Engine interaction points, the table's lifelink, `place_on_battlefield` and `StackEntry` rows (2445–2470 per a refuter) | `apply_lifelink`: "the gain becomes an `execute_action(GainLife)` proposal"; `place_on_battlefield` "becomes the *performer*"; `StackEntry`: "add `cast_from: Zone`" | `engine/keywords.rs::add_lifelink_gain`; the `EnterBattlefield` arm of `perform_action` is the performer (the function is gone); `StackEntry` \| `cast_from: Option<Zone>` (Some exactly for a spell) | A Site \| Change \| Phase plan table, every row true when written, its RA/RC phases landed: mark the rows ✅ or evict it, don't rename; `place_on_battlefield` still exists (test_support.rs:927) |
| replacement §8a Known-missing kinds, named now: countering a spell (2663–2667) | Countering is "a `ZoneChange` out of the stack with `ZoneChangeCause::Countered`", which "`EventPattern::ZoneChange { cause }` has watched since RB" | ... with `act: ZoneChangeAct::Countered`, which `EventPattern::ZoneChange`'s `change.act` watches (an act field since RH-1; watched since RB) | A dated record: one of four rows "Re-read at the post-RE audit (2026-09-15)", exactly true then; RH-1's rename (21b808a2, 2026-10-11) is recorded at lines 263 and 1063; the substance holds |
| cant-effects §4.7 Tier 3, 'Built by RD-4' paragraph (1098-1102) | "`Restriction::Event` deliberately did not" take a player set, since no expressible printed "can't" concerns a player-subject event | `Restriction::Event` did not at RD-4; RE-3 (2026-09-12) gave it `affected_players: PlayerSet`, for "players can't gain life", unioned with `affected_objects` by `set_affects`. | A dated record, not a claim about today: opens "Built by RD-4 (2026-09-09)", past tense, true at 87ccd1e2 (no GainLife/LoseLife arms); RE-3's later field does not contradict a record of RD-4. |
| copy-effects §5.1 What RC-4 actually needs (1084-1098); §9 item 9 (~1645); §11 owed list (~1745) | `CopyOnEnter` "moves to CV-2"; replacement.rs:316's RC-B cite "should be corrected"; `CopyOnEnter` is "named backwards", rename now | The 616.1c bucket is `CopyAsEnters` (renamed from `CopyOnEnter`, produced by `Rewrite::EnterAsCopy`, CV-2a); RC-B cite gone; mark §9 item 9 Done; drop §11's line | Records of the 2026-08-29 design pass, not claims about today: the header dates it, §9 item 9 says "this pass ships no .rs changes", §11 is unmaintained, §7b already records the rename |
| copy-effects §9 item 2 (1567-1573) | "**Done** by 2026-09-28: the comment, now at `types/replacement.rs:1661`, names CV-2." | Done by 2026-09-28: RC-B cite gone; the comment above `ReplacementClass` now says only `SelfReplacement` has no producer and `BackFaceUp` waits on CV-5 | A dated closure note, exactly true when written (de7a333c: :1661 read "`CopyOnEnter` waits on CV-2"); 28c02178 later rewrote it; no RC-B cite remains, so only "now at :1661" is outdated |
| cost §3.3, step 4 Reductions (362-366) | A `not_below` reduction "additionally stops at its own floor", a clamp on the step, not on the total | Not built: when its subject lands (§3.10), a `not_below` reduction will stop at its own floor, a step clamp; no arm carries it today. | The doc already says it is unbuilt: §3.2 comments out `Reduce { not_below }`, §3.4 and §3.10 defer it, §6's CM-1 row says "no `not_below`"; "as built" predates CM-1. |
| triggers §4.8 The trace's sixth emit point, nesting limits (2149, 2164) | `BATCH_NESTING_LIMIT` (32, `actions.rs:38`) ... `DISPATCH_NESTING_LIMIT` (16, now `dispatch.rs:62`) | `actions.rs:42` and `dispatch.rs:65`; or name the constants without line numbers | Both pointers sit in dated amendments (2026-09-22; the 2026-10-21 draft) and were true then; "now" means as of the draft; the values and behavior (32 returns Err, 16 draws) still hold |
| triggers §7 The trackers, fact table last row (3085) | `TriggerBinding` (the matched records, copied whole; the matched event; the subject's epoch) | `TriggerBinding` (each triggering event: its record and the objects it is about, by role, as `ObjectRef`s taken at dispatch; the matched event); reader: the resolution, through the arm's projections | The row names no fields and each part still holds: `TriggeringEvent.record` copied whole, `event: EventIndex`, ObjectRefs carry the epoch; it only omits other roles' objects, incomplete, not false |
| mana §3.1 "The mana check runs last" (229-231) | `can_afford_ability_costs` "walks the printed cost list, mana first on most cards, and asks targets after." | `can_cast` and `can_activate` (`can_activate_as_its_controller`) both ask timing, targets and the other costs before mana. Before MA-1, `can_afford_ability_costs` walked the printed cost list mana first. | §3.1 is MA-1's design, dated by the status header (2026-10-05, numbers on f4df90a); mana_helpers.rs:327-350 builds its mana-last decision, and the removed function's half describes f4df90a, as §2.1 does. (Same claim survived as doc:mana#2.) |
| mana §3.2 symbol table, `{X}` row (~281), with §3.7 (~457) | "\| `{X}` \| none at the gate (§3.7) \| yes \|" | A spell's `{X}` is expanded to its least legal X (0, or 1 generic under "X can't be 0") before the check; an unexpanded `{X}`, an activated ability's, is refused until CP-1 announces it. | §3.7 says the check tests the least legal X, as mana_helpers.rs:154-160 does; the unexpanded `{X}` falls under the next line, "Until a payment path pays a symbol, the check refuses it"; CP-1 owns it. |
| mana §9 Found while designing, "`ManaCost` holds generic mana as one symbol per mana" (909-913) | 601.2f's step 1 expands X with `repeat_n(Generic, x_value * x_count)` (`total.rs`) and `apply_reduction` rebuilds generic the same way, so a cost allocates a symbol per mana. | The arithmetic keeps generic as a count (`ManaComponent`, `total.rs`; reductions too), but `ManaComponent::at` and `ManaCost::with_x_as` still write one `Generic` symbol per mana. | A dated record under §9's "Found by MA-2's plan (2026-10-17, at #259)", written 58 minutes before f5b02eed; its point still holds, and §3.7's as-built (521-529) and backlog.md:2024-2025 already describe the count. |
| setup §3 Where it lives, options table (227) and recommended shape (233-236) | `Scenario::parse(&text)?.build(&registry)?` returns a `Game`; sketch: `let mut game = scenario.build(..)?; game.resume(&dp)?;` | Returns a `BuiltScenario`, the `Game` and its `SetupActions`; sketch: `let BuiltScenario { mut game, setup } = scenario.build(..)?; game.resume(&SetupDriver::new(setup, &seats))?;` | §3 is the design-time decision table (2026-09-30, status "design"); its decision, an engine module `mtgsim::scenario`, holds, and §5.3 and §8 SU-2 already qualify the return type. |
| clone §Header, Status (3) | "**Status:** design, 2026-10-21, for `roadmap-v2.md` B10; no code." | "**Status:** landed. Designed 2026-10-21 (#266) for `roadmap-v2.md` B10; its one build, CL-1, landed 2026-10-21 (#267, §7)." §1–§2's numbers were measured on `b86c59c1`, before CL-1. | A dated record of the design, which the same header qualifies ("CL-1 landed 2026-10-21 (§7)", added on purpose in b613148a). replacement, triggers, copy-effects and cant-effects still say "No code" after their builds landed. |
| clone §Header, Companions (27-28) | "`triggers-architecture.md` §4, the dispatcher's queue and placement, which write two of the stores." | "`triggers-architecture.md` §4, the dispatcher's queue, and §5, placement, which write two of the stores." | The engine claim holds (dispatch.rs:829,834 queue; placement.rs:35-102 drains). Only the cross-doc section number is off, and it was wrong from 14db570a, not drift: worth a tidy-up, not a stale-claim fix. |
| clone §1.1 The stores (71-75) | "`FitOnClone` (`types/ids.rs:450`) is item 183's half fix ... The history log ... is the tree's one store shared in chunks" | `FitOnClone` was item 183's half fix; CL-1 removed it (decision 4). The history log was then the one chunked store; since CL-1 `state/shared_stores.rs` holds the others. | §1 is the pre-build baseline of a dated design, qualified by the status line, §7 ("Before") and decision 4 ("drop it"). b613148a kept §1–§3 on purpose (engineering-practices.md#landing-a-phase). Owed eviction, not a one-paragraph fix. |
| clone §Decision 5, "What the check does today" (471-484) | "each check walks every object in the game to find the few that moved (`engine/sba.rs:137` and `:295`)." | "What the check did before CL-1": each check walked every object. Since CL-1 both rules read `arrivals_since_last_sba_check` (`engine/sba.rs:133`), and the walk is kept in debug builds only (`:141`). | A dated design record ("design, 2026-10-21", "CL-1 landed"). The pointers are b86c59c1's, decision 5's "Taken: (b)" already keeps the debug walk, and §7 closes item 293. |
| clone §Decision 5, "The list" (486-489) | "Every move into a zone goes through one function, `arrive_in_zone` (`engine/zones.rs:250`), which already stamps each move with its epoch." | `arrive_in_zone` (`engine/zones.rs:256`) | The claim holds: it is the only writer of zone and epoch, and :250 still lands in its own doc comment. 250 was exact at b86c59c1 in a dated doc, so :256 is a cosmetic retarget, not a fix. |
| clone §Decision 7, rule 4 (578-581) | "which `a_fork_reading_a_left_out_card_leaves_the_originals_memo_as_it_was` already asserts for today's map." | "which `a_fork_reading_a_left_out_card_leaves_the_originals_memo_as_it_was` (`state/layer_memo.rs:787`) asserts on the frames' table." | Pre-build design text: "today's map" is the doc's name for the pre-CL-1 baseline, and §7 records that the frames moved into chunks. The test at layer_memo.rs:787 still asserts rule 4. |
| codebase-state (+ nine other plans docs) §the EnterBattlefield entry (556; replacement 2046-2047) | `GameAction::EnterBattlefield { object, controller, mods }` ... "performed by `place_on_battlefield`" | "performed by `perform_entry` (state/game_state.rs)", at each occurrence that means the engine's entry performer rather than test_support's helper. | Line 556 is a dated RC-2 ledger record and was true then (RH-1 renamed the performer on 2026-10-11). Swapping only the name leaves today's shape false. replacement 2046-47 is a 2026-08-24 problem statement that §5d qualifies. |
| codebase-state §Before Triggered abilities, item 3 (closed LKI formalization) stub (5094) | "The reader is `engine::triggers::binding` — `bound_object`, `bound_player` and `bound_amount` over the record" | Name `bound_objects` (by role, over each triggering event's `ObjectRef`s), `bound_player` and `bound_amount`; CR 603.6's "unable to be found" and CR 400.7 as one epoch comparison. | It is a closed item's stub, word for word archive/codebase-state-closed.md:1565 under "CLOSED 2026-09-19 (TR-1)". `bound_object` was the real name then (renamed in c9e06a95, 2026-10-16). Editing the stub would split it from the archived verdict. |
| glossary Words that are not about one subsystem, **referents** (576-581) | **referents**: "you" (CR 109.5), "this" (CR 113.7a) and its owner, and the instances of "target" announced so far | Append "and the player a named attacking creature's 'defending player' means (CR 508.5)", per `FilterReferents::defending_player` (object_filter.rs:40-43) | The "antecedent" entry (glossary:685-690) already covers it: "defending player" names an antecedent, read as CR 608.2h reads that object, not a text's word for itself; the trait also carries counters_on. |
| engine/layers/explain.rs §`AppliedBy::EnteredWith` doc (68-69; also ui/why.rs:932, types/effects.rs:1617-1630, board.rs:552/1049) | "An edit it entered with (CR 614.1c)"; why.rs and effects.rs say the same, though §3.5 puts edits in the "as" half | "An edit it entered as (CR 614.1c)"; why.rs "what it entered as"; effects.rs "enters as" / "an entry edit"; renaming `EnteredWith` is a code change for the owner. | "Entered with" names the edit by `Rewrite::EnterWith` (replacement.rs:889), which writes every entry edit, Mutant included; the name dates from SU-6, after RG: a vocabulary mismatch with §3.5, not a false fact. |

**The 24 refuted.** In each, both refuters found the doc right or the claim
a dated record. Two were `CLAUDE.md` lines.
- "Checked ahead of the pipeline" is the project's own name for the site,
  the top of each CR 616.1f iteration, before the gather. `pipeline.rs:453`,
  the glossary and `cant-effects-architecture.md` §1 use the same words.
- Critical-path item 6's "A6–A6f" reads the route table in order, as
  `roadmap-v2.md:204-207` says, from row A6 down to A6f.

The journal keeps every text.

### Pass 5c — names for the phases ahead

**Added by the owner at #274's merge (2026-10-22), and scoped the same day:**
the phases and route rows still to come get names that say what they
deliver. Not the numbered items, and not the landed phases. Today the road
ahead reads `RS-2; PM-0; PM-1; B2; B3; B4 … RS-3a, RS-4, RS-3b; B8; CP-1,
CP-2; MA-7, MA-3–MA-6; B9; B13 … CV-5; CV-3, CV-4; CV-6; CV-7; A4l; B14`
(`CLAUDE.md`'s critical path): two-letter tracks, letter rows and suffixes
a reader has to look up, and the PR titles inherit them ("TR-5c: blocks,
targeting and activation", "A6m, PR 1 of 2"). Card breadth adds more.

**Counted on `21e3c1d4`** (`git grep` for each code as a word, live docs
being `plans/`, `CLAUDE.md` and `README.md` with the archive, the corpus and
the generated board out): 27 codes — the board's 17 "named, not yet landed"
phases, `PM-1`, and `roadmap-v2.md`'s `A6s`, `A4l`, `B2`, `B3`, `B4`, `B8`,
`B9`, `B13` and `B14` — about 970 mentions in the docs and 60 in source and
test comments, over 55 files. `RS-2` alone is 136.

**The work:**
- **A naming scheme**, decided first and reviewed: what a phase's name is
  (the subsystem and what it delivers, as the board's Phase column already
  reads: "Combat, the predicate half"), whether a short handle survives
  beside it for branches and greps, and how a phase split later is named
  (`engineering-practices.md#naming-a-phase`).
- **A mapping table**, old code to new name, kept where a reader of an old
  commit message finds it.
- **The sweep**, mechanical once the table exists, since each code is one
  token with one meaning. The gates move with it: `check_state_of_play.py`'s
  `ROUTE_TOKEN_RE` reads a `**Slotted:**` line's route token, and its board
  reads phases off headings.
- **Untouched:** landed phase codes (`TR-5h`, `CV-1b`, `RH-3` …), which live in
  commit messages, PR titles and `plans/archive/` that nobody rewrites, and
  the numbered items.

**Sized:** one PR, the design opened as a draft and reviewed before the
sweep goes on the same branch: ~1,000 changed lines for the sweep, inside
the band. **Order:** after pass 5's PRs and the card-homed triage, which
edit the same lines; before A6e's cleanup, so the cleanup's PRs and pass 6's
scheduling table are written in the new names.

#### The design (2026-10-23, reviewed at #282)

**Re-counted on `305e6195`** (#280's merge), the same way: the 27 codes are
1,228 mentions in the live docs and 43 in source, test and dev GUI comments,
over 61 files, on 874 distinct lines. `RS-2` alone is 149. The plan's figures
predate pass 5 and the triage, whose `**Slotted:**` lines were written in
these codes. The board reads 149 route-homed, 5 card-homed, 12 records, 0
homeless and **5** reachable — wrong today at that commit (the brief said 4;
the board is current at `305e6195`, so the check is "unchanged from 5").

**The problem the scheme solves.** `engineering-practices.md#naming-a-phase` (#225's
review) already puts a phase's plain name first in a title and its code
after. The docs kept citing the code anyway, 1,228 times, because a name of
six words does not fit where a reference goes: a table cell, a `**Slotted:**`
line, the critical path, a source comment's "until RS-3". The board's
machinery also needs one exact token per phase: `ROUTE_TOKEN_RE` finds a
`**Slotted:**` line's homes by shape, and `--slotted` greps for one. So the
reference form has to stay short and exact; what changes is that it is made
of words.

**The scheme.**
- **A phase still to come, and a route row still to come, is cited by its
  name: two to four lowercase words saying what it delivers, hyphenated, in
  backticks** — `combat-restrictions`, `game-formats`. Wherever the old
  code stood: prose, tables, `**Slotted:**` lines, source comments, the
  critical path. A title or a ✅ heading writes the same words with spaces
  and a capital, then a dash and what a reader needs beside them ("Combat
  restrictions — menace, evasion, landwalk"); a heading the board reads keeps
  the backticked form first (below). A branch is `<subsystem>/<name>`
  (`cant-effects/combat-restrictions`), the subsystem being the owning
  architecture doc's, or `route/` for a roadmap row.
- **Names use the CR's words where the CR has them** (CR 508.1c's
  "restrictions", 508.1d's "requirements") and a mechanic's keyword where one
  names the work (`convoke-delve-improvise`); no CR numbers, no abbreviations
  beyond `CLAUDE.md`'s three, and unique across the route.
- **No new codes.** A phase written from here on is named, never coded; the
  families (`RS`, `CV`, `CP`, `MA`, `PM`, `TR` …) close, and their letters
  stay only on landed phases. A track is cited by its subsystem in prose
  ("the mana phases", "permission's later builds"), never by its letters.
- **A split names each part for what it delivers**: `combat-restrictions`
  and `combat-requirements`, not a part 1 and a part 2, and no suffix. The
  whole's name goes to the record below, or stays as prose for the pair.
- **Landed codes are untouched** — commit messages, PR titles, the board's
  landed table and `plans/archive/` read in them, and nobody rewrites those.
  A phase that lands under a name keeps the name; its ✅ heading carries it.

This replaces §4.2's second and third bullets in the sweep; the first (name
first) and fourth (the generated index) stand.

**The scope's edges** — the codes the plan did not list, each in or out:

| Code | Mentions | | Why |
|---|---:|---|---|
| `B1` cost, `B5` (RS-3a, RS-4, RS-3b), `B6` (CV-3 … CV-7), `B11` permission, `B12` mana | 40, 11, 17, 51, 4 | in, as prose | Rows that group a track. Their phases get names, so a row ID left beside them is the one code a reader still decodes. Cited by subsystem, no hyphenated name; `B11`'s three `**Slotted:**` lines name `permission-design`, which sizes the builds |
| `RS-3`, the combat pair | 18 + 13 in source | in | The family of two named phases; each mention names the one it means, or both |
| `PM`, `RS`, `CV`, `CP` as a track's letters | 54 | in where forward | "PM's later builds" is a forward step on the critical path; a mention of a landed track stays |
| `B7`, the lattice | 18 | in | Forward, and seven `**Slotted:**` lines home on it, so it needs a name the board reads |
| `A6f`, the information-model design | 24 | in | Forward, on item 6's line, not in flight; pairs with `B4`'s build |
| `A4d`, the Rust notes | 9 | in | Forward, and missing from the plan's list |
| `C0`–`C2`, Phase 8's named PRs | 48, 15, 10 | in | Forward route rows cited by code, four `**Slotted:**` lines on `C0` |
| `A6e`, `A6h` | 118, 28 | out | The audit in flight; they land as codes |
| Phase 8, 9, 10; `§D`, `§E` | — | out | Already named ("Phase 9 formats and multiplayer"); numbered like the items |

With the edges the sweep is about 1,500 mentions on 1,075 lines over 69
files.

**The names**, old code to new, are `plans/archive/route-codes-renamed.md`:
a record, since every live mention moves and only a reader of an old commit
message, PR title or archive page still meets the codes. `git grep RS-2`
finds it. §4.2 and the board's "Phases by name" point to it. **No glossary
entries**: `check_glossary.py`'s first assertion wants every term in
`mtgsim/src`, and a phase name is the route's word, not the code's.

**The gates that move**, all in `check_state_of_play.py` but the last two:
- `ROUTE_TOKEN_RE` gains a backticked hyphenated name. One the route does not
  define reads as unknown, so a mistyped name alone leaves an item homeless
  and fails `--check`; a file name like `` `cant-effects-architecture.md` `` is
  not one, the dot ending the match.
- `route_ids` collects the names from the three places that define them:
  the architecture docs' sizing tables and headings, `roadmap-v2.md`'s rows
  (a forward row's first cell becomes its name, landed rows keep their IDs),
  and ✅ headings.
- `DEFINED_RE`, `LANDED_HEADING_RE` with `CODE_RE`, and the ✅-heading reader
  beside them accept a backticked name where a code stands
  (`` | **`token-copies` — a second constructor …** | ``,
  `` ### `token-copies` — token copies — ✅ 2026-… ``). `PM-1 onward`, which
  the board missed, became `command-zone-casting`'s row and
  `casting-from-other-zones`'.
- `--slotted` takes a name (`--slotted combat-restrictions`); the board's
  example and the index's "Code" column, read "for older text", gain a
  "Cited as" column for the phases not yet landed.
- `selftest` gains a name-homed line, a backticked non-name and a landed
  name.
- `plans/references/comment-census.py`'s `PHASE_CODE` gains the backticked
  names, so §2.1's census counts "until `combat-restrictions`" as it counted
  "until RS-3".
- `plans/references/cast-census.py`'s owner keys (`"RS-2"`, `"B2"`, `"PM"`)
  are renamed and its page edited to match; it fetches from Scryfall, so the
  page is not regenerated.

**The check that the homes held** is per item, not by total: the board's
`home()` for every open item before and after, by a probe, identical. A
mistyped name reads as no token, so a total could hide two items swapping.

**`CLAUDE.md`'s critical path** reads in the names inside 200 lines: the
"After 6" block went from four lines to six.

**Sized:** the sweep's ~1,075 lines, about as many additions, plus the
gates (~150 with the selftest), §4.2 (~20) and this design and the record
(~200): about 1,450 additions, inside the band, so one PR. Source comments
are 53 of those lines, the only ones §4's code-and-tests count sees. **Lines
are not re-wrapped**: a name runs longer than its code, so some lines pass
the docs' width, and a re-wrap would re-flow whole paragraphs and bury the
substitutions in the diff.

**Decided at #282's review (the owner, 2026-10-23):**
1. **The cited form**: the hyphenated name in backticks.
2. **The edges**: all in but `A6e`, `A6h` and the numbered phases.
3. **The names**, with five changes and two consequences for the route:
   - `cant-target-or-cast` **split** into `cant-be-targeted` and
     `cant-cast-play-or-activate`. The owner asked whether the two belonged
     in one phase; they share the read side's query and nothing else, not a
     site, a consumer or a design partner, since the casting half's checks are
     where `permission-design`'s permissions land. `cant-effects-architecture.md`
     §7's row became two.
   - `commander-format` **widened** to `game-formats`: Commander and the five
     60-card constructed formats as data, a card's legality read from a table
     refreshed off Scryfall. It pulls Phase 9's format half ahead of Phase 8;
     match structure stays out, being the tournament rules'.
   - **What is left of Phase 9** once `game-formats` and `multiplayer-rules`
     come forward is `route-review`'s to decide (its row says so).
   - `remaining-costs` for `discard-and-counter-costs` ("counter" is both a
     noun and a verb in the CR); `any-color-plus-painlands-plus-signets` for
     `any-color-and-riders` ("rider" is the replacement pipeline's word
     already); `chosen-color-plus-could-produce`; `breadth-features` and
     `card-layer-cleanup` for the two ⚠ names.
   - Two route steps that had no name got one: `casting-from-other-zones`
     (permission's later builds) and `split-and-adventure-style-cards` (the
     other characteristic sets), each a row in its doc.
4. **The retired-code check: not built.** The owner plans to pare `plans/`
   down after v1, and the harm a stray code can do is a `**Slotted:**` line
   that loses its home, which `--check` already fails as homeless.

### Pass 6 — scheduling

**Goal:** one table, §9's shape: each open track phase and lattice entry,
what it unlocks in cards (the ledger and the docs' consumer lists, no
Scryfall count re-derived) and in atoms (`spec.sqlite` by rule prefix, the
prefixes named), what it needs, and which of `information-model-design`'s and `route-review`'s questions needs
it first; then a proposed order for what follows. §3b re-derived; `rust-notes`'s
open artifact, the Rust notes, placed. The pass proposes; the owner
decides. One PR, which writes §3's two records and deletes this file.

## 5. A6e's items, by pass

| Item | What | Pass |
|---|---|---|
| 96 | promote `BOUNDARY-DEF-615.1a-001` | 1 — closed by #272 |
| 138, 142, 143 | the throughput target, the panic-surface baseline, the clone table | 4 |
| 137, 7d, 215 (its two cleanups), 239, 247, 269, 284, 285, 291, 292, 296, 297, 300 | the cleanup; 297 is pass 3's row 57 and 300 the owner's at #272's review, both filed by pass 1 | 3 |

A6h's are 185 and 188 (pass 5). `route-review`'s, 264 and 288, stay with `route-review`.

## 6. Decisions for the owner

**All five accepted as recommended (the owner, 2026-10-22, at #270's
review).** Decision 4 is applied: A6e's row reads about 10–15 PRs and A6h's
2–4.

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
`backlog.md`'s and `engineering-practices.md`'s growth by subsystem; the old doc's §3
subsections paragraph by paragraph (`archive/engineering-practices-records.md`); whether `panic_surface.py`'s two runs
counted alike. Each pass reads what it needs of these.

## 8. Briefs

Each pass's session starts from its section above and this file's §2 and
§3. A brief adds only the branch name and what the previous pass's PR
changed.

## 9. Status

| Pass | PR | State |
|---|---|---|
| Plan | #270 | merged 2026-10-22 |
| Discovery (session 1: Workflows A, B and C, pass 2's script) | #271 | the work-lists under passes 1, 2, 3 and 5 |
| 1. Close-out (2–3 PRs) | #272 | one PR: the side findings reproduced; items 297–299 filed; 53 verdicts re-dated; §13, §15 and §16 dispositioned; items 96, 197 and 217 and §15.4 fixed; `owed` clean; the TL;DR, the CR 603 row and `triggers-architecture.md` §18 |
| 2. Eviction (1) | #273 | one PR: the whole sections and §4.9's `loop-detection` input moved, the 22 amendments collapsed into the text they overruled (−239 lines), 8 outside citations re-pointed; `triggers-architecture.md` 4,736 → 3,853, the archive 5,618 → 7,067 |
| 3. Hygiene and CI (1–2), then A6e's cleanup (8) | #274, #283, #284, #285, #286, #287, #288, #289, #290, #291 | the hygiene half, one PR: `comment-census.py`, §2.1's fourth application (history blocks 100 → 77, the outgrown comments corrected); every `ci.yml` step commented and `history_held_test` given a step; clippy re-counted, all five allows kept; the 1.88 floor re-measured; items 301–315 filed; `--suggest` triaged, seven terms defined; the README brought to item 6's close, and §9's close-out now lists it (the owner, at #274's review). A6e's cleanup, #283: the plan, 60 items and 40 merge rows in eight themed PRs, each item's **Slotted:** line naming its PR. Its first PR, `history-and-frame-records`, #284: 11 items closed, 278 and 279 built and narrowed (278's CR 509.4 entry to `layers-combat-and-counters`), the 12 merge rows, `--pool fixtures`. PR 1b, `last-known-information-store`, #285: item 318 closed, one LKI store keyed by existence, the keepers' lists and the second captures gone, the four view types folded into `ObjectInformation`. PR 1c, `practices-rules-page`, #286: the practices doc a page of rules cited by heading anchor, under `check_practices_page.py`. PR 1d, `concept-census-and-register`, #287: the census of the fields the game holds, the register of 21 concepts in `cr-coverage-audit.md` §4c and `check_type_surface.py`'s half that keeps it, the Characteristics arm's `FrameContext`, items 319–327 filed into five cleanup PRs and item 30 re-slotted. PR 2, `resolution-context-rider`, #288: item 137 closed, `ResolutionContext`'s rider numbers one optional `ReplacedEvent`, 101 literals swept. PR 3, `trigger-matcher-merges`, #289: items 247, 301, 302, 304, 309, 315, 317, 319, 320 and 321 closed and the merge rows, item 317 its last commit; item 320's other payloads split to item 328, item 329 filed. PR 3b, `record-existences`, #290: item 328 closed, every record naming its objects by existence and a trigger remembering it; neither pool moved. PR 4, `one-fact-one-type`, #291: items 291, 296, 326 and 327 closed, rows 46 and 64 merged; one type each for an object or a player, what an effect affects and which objects an amount counts; a zone statement out of `Condition` |
| The card-homed triage (1, then its builds in the cleanup) | #280 | one PR: the 51 items dispositioned in their `**Slotted:**` lines with verdicts re-dated, 31 builds for A6e's cleanup with their fixtures and six consistency checks, 14 to rows (`route-review`, `ironworks-loop`, `mana-ability-costs`, `chosen-color-plus-could-produce` and `any-color-plus-painlands-plus-signets`, `permission-design`, `remaining-costs`), 5 kept with their cards, item 109 closed, item 273 to `ironworks-loop`; the fixture pool priced for the owner |
| 4. Readiness (2) | | |
| 5. Docs audit, A6h (5) | #275, #276, #277, #278, #279 | re-cut at #274's merge; item 188 to `route-review`. PR 1, the contracts: `CLAUDE.md`, `engine-map.md` (re-pinned to `22113bbb`, §1.2 read) and every contract-change row, contested ones dispositioned. PR 2, the older subsystem docs (#276): layers, replacement, cant-effects, copy and cost — every remaining row fixed, 11 contested dispositioned, the unread sections read. PR 3, the newer subsystem docs, the registers and the source comments (#277): every remaining row fixed, the contested ones and pass 3's dedup claims dispositioned, the unread sections read, item 316 filed. PR 4, item 185 (#278): the inventory `plans/elisions.md`, the record of each question not asked, the shortcuts' audits; item 185 closed, item 317 filed; the trace pages split out. PR 5, the two trace pages (#279): TR-4d's control sweep and TR-5h's history log, from traced runs at `50f0b8b8`, linked from §12, `codebase-state.md` and `engineering-practices.md#trace-pages`. Done |
| 5c. Names for the phases ahead (1) | #282 | added at #274's merge; one PR: the design reviewed, then the sweep — the scheme in `engineering-practices.md#naming-a-phase`, the record of the codes, the board reading names with every item's home unchanged, the hexproof phase split, `game-formats` widened, Phase 9's remainder put to `route-review` |
| 6. Scheduling (1) | | |
