# Handoff — `practices-rules-page`, the design note

**Opened 2026-10-24**, A6e's cleanup PR 1c (`item-6-audit.md`'s cleanup
plan, row 1c). This file is the design note, reviewed before the build; the
build runs on the same branch and the PR that lands it deletes this file.
Every count below was read from the tree at `7ade063b` on 2026-10-24.

## 0. The problem

`plans/engineering-practices.md` is the process doc: how a PR is sized,
measured, commented, named and gated. It is **2,571 lines, 175 KB**, and
its rules sit inside the records that produced them: a rule is usually one
bold sentence, followed by the war story, the first application, the
measurement and the review that asked for it. Nobody reads that as a list
of rules, so nobody checks a new design against the list. The cost showed
at #285: §2c's worked example of "one collection of records, never two kept
in step" was TR-2b's per-keeper LKI lists, the very split #285's store of
last known information undid. The example blessed the design, and no one
re-read the rule against it, because the rule was a paragraph in a long
section of records.

**The goal:** every rule on one short page, about three lines and a pointer
each, the way `CLAUDE.md` states its invariants; the records and
measurements moved to where they belong; and nothing that was a rule
dropped.

Where the bulk is, by section with its subsections:

| Section | Lines | What it mostly is |
|---|---:|---|
| §3 Two card pools | 861 | the floor tables and the measuring method (§3.1, §3.2), the rulings pass (§3.4–§3.4b), and the records behind each |
| §10 GUI code | 363 | the dev GUI's review method, the random-clicks test's history, `prompt_cost`'s readings |
| §2 The comment rule | 246 | five rules, then §2.1's four sweep records with their tables |
| §4 Sizing and splitting | 221 | the band rule and a dozen sizing rules, each with its story |
| §9 Spine-phase audit | 167 | the audit's method, pass by pass, and its first instance |
| §7 Trace pages | 166 | the page format, an index of sixteen pages, and the tiers' landing records |
| §11 Invariants as gates | 138 | the three checks, the source tags, what the gate misses |
| §1, §2a–§2d, §5, §6, §8 | ~405 | mostly rules already |

## 1. Decisions

### Decision 1 — where the rules live

**The shared problem.** A rule helps only if it is read when a design is
made, and `CLAUDE.md` is the one file read every time. But it loads before
every task, it is at exactly 200 of its 200 lines, and the page this PR
writes will hold about 160 rules.

- **(a) In `CLAUDE.md`.** The page's ~400 lines would join it, with
  `check_claude_md.py --budget 600` typed to allow it. Every task then loads
  three times today's preamble, most of it about measuring and sizing, which
  most tasks never touch. Upkeep is one file. `CLAUDE.md` already states the
  dozen practice rules that bite most often (comments, names, counts in docs,
  sizing, the rules pass, the specdb gate, the dev GUI, registration) and
  points at their sections.
- **(b) A short page, linked from `CLAUDE.md`'s authority table.**
  *(Recommended.)* The page keeps the path `plans/engineering-practices.md`,
  so every citation and script that names it still lands. `CLAUDE.md`'s
  authority row is rewritten in place, one line for one, to say the page is
  every process rule in three lines and a pointer. Its other twelve pointers
  keep their section labels (decision 2) and need no edit.
- **(c) Both: the page, and a few rules promoted into `CLAUDE.md` by swap.**
  The candidate is the new rule from #285, "key a CR concept before storing
  it" (§2e below). It is a design invariant more than a process rule, and
  `CLAUDE.md` is where invariants live. But a line in costs a line out, chosen
  against the whole file, and 1d (`concept-census-and-register`) builds a
  gate for it, which enforces it better than a line does. I would take (c)
  only at 1d, if the gate leaves something a reader must be told.

### Decision 2 — section numbers

**The shared problem.** The page reorganizes the doc's content, and other
files cite its sections by number. Measured at `7ade063b`: **330 citations
on the same line as the doc's name, across 115 live files**, all of which
resolve to a heading today:

- 88 in `mtgsim/` (65 in `src`, 21 in `tests`, 2 in `Cargo.toml`), 2 in
  `devgui/`;
- 12 in `CLAUDE.md`, 7 in `ci.yml`, 24 in eleven scripts;
- 15 in **pinned trace pages**, one each, which §7's rule says are never
  edited;
- 181 in 23 plans docs and the item 6 handoff, `codebase-state.md` the
  most (38), and 1 in `.gitignore`.

Beyond those: about 100 inside the doc itself, about 111 in `plans/archive/`
(accurate records of the doc as it was, not edited), and an unknown number
of bare citations ("§3.1's floors") on a line after the one naming the doc,
which no script can find.

- **(a) Stable labels.** *(Recommended.)* Every label cited today keeps a
  heading on the page with the same number. A heading's title may change; a
  label whose body moved away keeps a heading and one line saying where it
  went, so a citation lands on the rule and the pointer. New labels are
  allowed (§2e); a retired label is never reused, as a ruling's `#n` never
  is. No citation is edited, the pinned pages and the archive stay right,
  and so do the bare citations nobody can find. **The cost:** the page keeps
  the old order. A concept spread over several sections stays spread, so
  measuring is §3, §3.1 and §3.2, and naming is §2b and §4.2. A short index
  by concept at the top of the page answers that in about ten lines.
- **(b) Renumber by concept, and rewrite every citation.** The page reads in
  its natural order. The sweep touches about 110 files, 65 source comments
  among them, and cannot reach the 15 pinned pages, the archive or the bare
  citations, so those point at the wrong rule from then on, silently. Upkeep
  afterwards is the same as (a).

The script that checks every citation resolves is the same under either
(decision 4); under (a) it passes on day one, which is also how the build is
checked.

### Decision 3 — where each record goes

**The shared problem.** About 2,200 of the 2,571 lines are not rules. They
are of three kinds, each already with a home in the project:

- **Measurements** go to `plans/fuzz-record.md`, the project's dated record
  of what was measured, newest first. Each moved block keeps its date and
  goes in at its date's place. Moved: the floors' "Standing" history, the
  Cytoshape `--require` table, §3.2's tail baselines and the `ByOwner` A/B,
  the pool-change readings under "The five bold rows", the Circle of
  Protection count, §10.4's `prompt_cost` readings, §11.5's sizing of the
  checked sitting. "How to read the five bold rows" goes to `fuzz-record.md`'s
  preamble, since it is how to read that file's tables.
- **History** goes to a new `plans/archive/engineering-practices-records.md`:
  every war story, first application, review exchange and decision record,
  under its heading byte-identical, so a grep for a heading still lands.
  Moved: §1's examples, §2.1's four sweep records with their tables, the
  stories under §3, §3.3, §3.4, §3.4a (the census and the A4b sitting),
  §3.5 whole, §4's and §4.1's stories, §5.1, the "why" halves of §6 and §7,
  §7's per-phase trace decisions, §7.1's two landing records, §8's RE-7 and
  §8.1's SU-7, §9's first instance and "what the next run adds", §10.2's
  first run, and §10.3's "Since …" paragraphs.
- **Method** becomes its own doc where the method is long and read whole by
  whoever does the work. The page keeps the rule that says when, and points:
  - §7's page format and its index of pages → `plans/traces/README.md`,
    beside the pages;
  - §9's passes and their instruments → `plans/spine-phase-audit.md`, the
    doc a spine close's handoff is planned from;
  - §10.1's egui checklist and §10.3's account of what random clicks do →
    `plans/devgui-map.md`, which already says how each part of the dev GUI
    is reviewed. A review agent is handed that section instead.

  The alternative is appendices at the bottom of the page: one file, and a
  citation of §7 or §10.1 lands where it did. But the file stays near 1,000
  lines, so it is not short, and that is the failure this PR exists to fix.

**Two tables are both a rule and a record, and need a choice each.**

- **The floor table**, which `check_state_of_play.py` parses (`floors()`,
  between `<!-- floors: begin -->` and `<!-- floors: end -->`) to draw the
  board's floor readings. Its `Limit` column is a rule and its `Reading`,
  `Read on` and `Where` columns are a measurement.
  - **(a) Split it.** *(Recommended.)* The page's floors table holds each
    floor's limit, definition and instrument; `fuzz-record.md` opens with
    "The latest floor readings", each floor's reading, date and where it was
    read, replaced in place as today. The script reads the limits from the
    page and the readings from `fuzz-record.md`, joins them by floor, and
    fails when the two lists of floors differ. About 20 lines of script, in
    the same commit as the move. Each fact is written once, which is §2c's
    own rule.
  - **(b) Keep the whole table on the page.** No script change. The page
    then carries a measurement that changes at every reading.
  - **(c) Move the whole table to `fuzz-record.md`.** The limits then sit in
    a record, and the page's floor definitions repeat them: two copies kept
    in step.
- **§3.4b's register**, the three rows where the engine follows the CR over
  a ruling, or one reading over another. `check_rulings.py` and
  `references/feedback-loops.py` both read it under its `### 3.4b ` heading.
  **Recommended: it stays on the page.** Each row is a standing decision a
  builder must follow, as the source tags in §11.1 are, it is three lines of
  markdown, and leaving it moves no script.

### Decision 4 — which rules become gates

**Built here (recommended):**

1. **Every citation resolves.** A new `plans/check_practices_page.py`, in
   the `check_*.py` family, on `CLAUDE.md`'s check line and in CI. It finds
   every `engineering-practices.md §X` in tracked files outside
   `plans/archive/`, and every `§X` inside the page itself, and fails on a
   label with no heading. Today all 330 resolve, so it is green before and
   after the build, which is how the build is checked.
2. **The page has a line budget.** The same script fails when the page is
   over its budget, set at the build to the page's size rounded up (about
   400) and raised only by a typed `--budget N`, as `CLAUDE.md`'s is. The
   doc reached 2,571 lines because nothing counted it, and that is §1's own
   argument.

`CLAUDE.md`'s check line gains `&& python plans/check_practices_page.py`.
The line count stays 200.

**Could be checked by a script, slotted elsewhere:**

| Rule | The check | Slotted to |
|---|---|---|
| §2: say the CR rule, never a plans item number, in source | a grep for "item N", `L7`-style labels and phase codes in comments; the old lines in `src/` must be swept first | `names-and-hygiene-sweeps` |
| §2b: no bare pronoun or `Other` as a name | a variant-name list, read from the enums | `names-and-hygiene-sweeps` |
| §7: a trace page is pinned and linked from two places | its `<head>`'s first comment names a commit and a CR version; an architecture doc and `codebase-state.md` each link it | `names-and-hygiene-sweeps`, or here if you want it (about 50 lines) |
| §9: every CI step carries a comment giving its reason | each `ci.yml` step preceded by a comment | `names-and-hygiene-sweeps` |
| §2c and §2e: one owner per CR concept, keyed first | the concept register | 1d, `concept-census-and-register` |

**Already gates, unchanged:** §1 (`check_claude_md.py`), §2b item 9
(clippy's `default_trait_access`), §3.4 and §3.4b (`check_rulings.py`), §4's
40-line stub, ✅ heading and homeless-item rules (`check_state_of_play.py`),
§6 (`check_module_layout.py`), §10.2 (`check_egui_only_draws.py`), §11
(`check_invariants.py`, clippy's `disallowed-methods`).

**Not gateable, and why:** §2d's counts (no shape tells a count that decides
something from one that rots); §3's "a fixture must not wear a real card's
name" (needs an offline list of every printed name, which 1d's census may
bring); §4's band (the reviewer decides an overage); §4.2's "no new codes"
(the owner chose no check, at #282).

## 2. The page, section by section

Labels kept as today. The second column is what stays on the page as a rule;
everything else moves as decision 3 says.

| § | Lines | What stays as rules | Moves |
|---|---:|---|---|
| preamble | 14 | what the page is, and an index by concept (new) | the founding note → archive |
| 1 | 55 | 200 lines, hard; three lines and a pointer; adding needs removing; a war story is not an invariant; status is derived, not written; choose the cut against the whole file; a raise is typed | the examples → archive |
| 2 | 31 | comment the why; one cite beats a paragraph; the rule, never an item number; war stories to commits and docs; doc comments exempt from brevity only | the RB framing → archive |
| 2.1 | 215 | a sweep, not a budget, every 15–20 PRs and at a close that moved a measurable; `comment-census.py` at both ends with `--rev`; date or re-derive, then delete what the code says; read the subsystem's doc comments for a count or a tense; no comment-length rule in `CLAUDE.md` | four application records and their tables → archive; `comment-census.py`'s calibration cite repointed |
| 2a | 41 | the three reasons a wrapper earns its place; otherwise inline; the reason goes in its doc comment; a chokepoint is a different question | the review finding → archive |
| 2b | 60 | the ten checks, one line each; one spelling per concept; retrofit where a PR reaches | the examples shrink to one each |
| 2c | 15 | one record, never two kept in step; a derived collection dropped at once is not a copy; **rewritten example**: the store of last known information keyed by existence, and `departure_read` (`engine/triggers/dispatch.rs`), whose two match arms are CR 603.10's two readings of a departing object, as its record framed it for an arm that looks back and as it is now for any other | TR-2b's `kept_lki` example, retired |
| 2d | 31 | a count is generated or decides something; otherwise described; where it bites; work-lists exempt | the 208-claim finding → archive |
| 2e (new) | — | **key a CR concept before storing it**: ask what the CR keys it by and who owns it before where it lives or how long; TR-2b chose by lifetime, and CR 400.7's key, the existence, came only with #285 → 1d for its gate | — |
| 3 | 129 + 128 | the pools table; register the card; the performance pool is representative and grows one card per new engine path with a re-record; two more cards before a word goes structural; a fixture may be invented, never wearing a real name or cited as Magic; the pool measures cost, `--require` coverage, read per copy, once per phase that adds a card; say the pool; the tables are fixtures, no `ms/game`; a cost row is a fixture, so an unordered count is a determinism bug; measure an activation, not a cast; no behavioral cap on the agent; repeatable activations are not kept out | the freeze story, the Cytoshape table, how to read the bold rows, the pool readings, Circle of Protection, item 104 → `fuzz-record.md` and archive |
| 3.1 | 122 | each pool's question; never A/B across a pool change; the close-out checklist; the budget (2.5 points of instructions per decision); instructions, not time, and why in one line; the three floors' limits, definitions and instruments; floor 1 under a named stack; determinism outside `=== Timing ===` | the Standing column and the latest readings → `fuzz-record.md` (decision 3) |
| 3.2 | 79 | a mean hides a cliff, so read p50/p99/max and the turn tail beside the game tail; time compares only within one sitting; 200 games when the tail is read; a turn tail climbing under a steady median is the signal | the baselines and the `ByOwner` A/B → `fuzz-record.md` |
| 3.3 | 65 | ask the rule's axis, not a quota: three tiers; shape over count; a gap register is re-measured, not inherited | the Kalitas finding → archive |
| 3.4 | 71 | every registered card gets a rulings pass, three answers; in the card's doc comment under its heading; format-variant rulings dropped; then ask what the filter has that no test varies | CM-3's outing → archive |
| 3.4a | 193 | the ledger and its gate: a linked test per ruling, five dispositions, `Card #n` never reused, drift fails, scoped; the end state is every official card tested; cited cards | the census, the build record and the queue's head → archive; the decisions' detail stays in `check_rulings.py`'s docstring |
| 3.4b | 37 | the five-rule policy; **the register** (decision 3) | — |
| 3.5 | 37 | none: a note, unscheduled | whole → archive, one line kept |
| 4 | 119 | size before writing, split in the doc; the band counts code and tests, docs beside, net of generated files; hunt the gaps card by card; read the intake with `--slotted`; a consumer in every PR of a split; review findings to a handoff by theme; `git log main..HEAD` lies; an arm the PR's type opens ships under ~80 lines; the CR is the customer; a gap in the phase's own subsystem ships while the band holds, else one of two slots; a landed phase marks its ✅ heading; a landed section is a ≤40-line stub | the RB story and each rule's origin → archive; "sub-phases are numbered" is superseded by §4.2's "no new codes" and is not carried |
| 4.1 | 64 | the standing question's two halves; when to ask it, as a list; a premise that enumerates shapes breaks, so put a property on the type whose arms grow | the three corrections → archive |
| 4.2 | 38 | a phase has a plain name; one still to come is cited hyphenated in backticks; no new codes; a row's ID orders, never names; the index is generated | the origin → archive |
| 5 | 38 + 31 | annotate at write time; never claim what a test doesn't prove; `owed` clean before a close, armed by `SHIPPED_PHASES`; triage a fact or a feature; a defect with no atom is the review's | §5.1's Phase 6 history → archive |
| 6 | 39 | a `mod.rs` declares and re-exports, a macro excepted; a sibling named for what it does; a one-file module is fine | the why → archive |
| 7 | 105 + 61 | when to write a page (a phase that changes how a read is answered); its examples are the findings; pinned, never maintained; linked from two places; tier 2 makes a spine, never the argument; tier 3 is `engine-map.md` | the format and index → `traces/README.md`; the trace decisions and landing records → archive |
| 8 | 46 + 39 | the three greps before deferring; the corpus too; for a new type, the three v1 questions, answered where the type is designed | RE-7 and SU-7 → archive |
| 9 | 167 | a spine close is audited before the next item starts, in passes each its own PR from a handoff; findings land where the project records them; docs and small fixes only; a pass re-reads what it touches; the midpoint hygiene pass for an item over ~8 PRs | the passes → `spine-phase-audit.md`; the first instance → archive |
| 10 | 56 + 307 | logic in plain Rust, egui only draws; three things in a GUI PR's body; the decision log is the engine's; the three kinds of display work and the review's question; a drawing file is reviewed against the egui checklist; the gate; `prompt_cost`'s before and after quoted by a PR that touches the snapshot or the view model | the checklist and random clicks → `devgui-map.md`; the readings → `fuzz-record.md`; the history → archive |
| 11 | 138 | the three checks; the tags table; a count walking the layers is not order-free; a hot path pays for order only while tracing; a new performer is a row; a closing PR takes its `KNOWN` entry off; what the gate does not see; randomness by clippy; the checked sitting, and why a guard test is `cfg(debug_assertions)` | the A6q story and the sitting's sizing → archive and `fuzz-record.md` |

The preamble's authority paragraph names three docs for the engine
invariants; it is rewritten to point at `CLAUDE.md`'s table.

## 3. What the build must not break

- **`check_state_of_play.py`'s `floors()`** reads the floor table. It moves
  in the commit that moves the table, and `--check` passes on it.
- **`check_rulings.py` and `references/feedback-loops.py`** read §3.4b's
  register under its heading. Under decision 3's recommendation the register
  stays and neither changes.
- **The comment census's docstring** cites §2.1 for its calibration
  readings; it is repointed to the archive.
- **`CLAUDE.md`** changes two lines in place: the authority row and the check
  line. It stays at 200, and all eleven checks pass, plus the new one.
- **No code moves**, so the pools are identical and no A/B is owed.

## 4. Size, and the commits

A docs PR, so the band does not apply. Under decisions 2 and 3 as
recommended: the page about 400 lines, written new; about 1,400 lines into
the archive file, about 300 into `fuzz-record.md`, about 450 across the
three method docs, and about 200 lines of script. No citation is edited.

The build's commits, one concept each: the records out to the archive; the
measurements out to `fuzz-record.md`, the floor table and its script
together; the method docs; the page written as rules, §2c and §2e among
them; the gate and its CI step; `CLAUDE.md`'s two lines; the board.

## 5. Done when

- Every rule on the page in about three lines and a pointer, and the PR body
  maps every old section to its new home.
- `check_state_of_play.py --check` passes on the moved floor table; every
  section citation resolves under the new gate; all of `CLAUDE.md`'s checks
  pass.
- Pools identical; no A/B.
- `item-6-audit.md`'s 1c row names the PR. This file is deleted.
