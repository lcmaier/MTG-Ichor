# The spine-phase audit

When a critical-path item closes, it is audited before the next one starts
(`engineering-practices.md#the-spine-phase-audit`, the owner's decision at the
post-RE audit, 2026-09-15). This doc is how: the passes, each with its named
instruments, planned in a `plans/handoffs/` file that the last pass's PR
deletes, each pass its own PR. The run's own record is a
`### … — audited YYYY-MM-DD` heading in `codebase-state.md`, pointers and
not prose, written when the handoff goes.

**Why it is a practice and not a checklist in the closing PR.** The first run
found "done" defined in five places and collected in none, `owed` never run
for the phase, the comment sweep eighty PRs overdue, and the performance
levers ranked by argument rather than by a profile. None is a bug a test
catches; each is a claim about the tree that went stale with nothing
re-reading it. The person closing a phase believes its claims, so the
audit is its own sitting, run while the material is fresh.

## The passes

In this order: the close-out gates the rest, the eviction opens second,
hygiene and readiness commute, and scheduling needs the close-out and
readiness.

1. **The close-out.** The phase's done-checklist read off the tree, every
   entry closed, scheduled with an owner, or deferred with a dated
   reachability line and a size. Instruments:
   - **the board**: `check_state_of_play.py --write`, then `--check`, whose
     bolded rows are the work;
   - **`specdb owed --phase "<the phase>"`**, once `SHIPPED_PHASES` holds it,
     a gate: the phase is not closed until it is clean;
   - the doc's own findings list, known-missing table and out-of-scope list,
     re-read against what landed;
   - the summaries (`codebase-state.md`'s TL;DR and CR row, the README's
     status block, implemented table and roadmap rows), rewritten in place,
     never appended to.

   It produces the retrospective in `replacement-architecture.md` §14's
   shape: pointers, under 150 lines, written once. A fix found here is shown
   to fail first; one that moves a pool is its own PR with a `fuzz-record.md`
   block.
2. **The eviction.** The phase doc's pre-build reasoning and closed findings
   move to `plans/archive/<doc>-landed.md` under the 40-line stub rule,
   planned section by section with line counts before anything moves: every
   heading byte-identical so a grep still lands, every closed item leaving its
   `N. **title**` line so every outside citation resolves, every outside
   citation of a moved section read against its stub, and `check_glossary.py`
   on the result. It opens after the close-out merges, so the stubs point at a
   retrospective on `main`.
3. **Hygiene and CI.**
   - **The comment sweep** (`engineering-practices.md#the-comment-sweep`):
     `plans/references/comment-census.py` at both ends.
   - Every `TODO` given a live owner or deleted.
   - **Clippy counted before it is decided**: `cargo clippy --all-targets`,
     the count deciding which lints go to `-D` and which are allowed with a
     reason each. The toolchain pin measured. Every step in `ci.yml` carries
     a comment giving its reason.
   - **Helpers built more than once**: `plans/similar_functions.py`, whose
     token scan lists near-identical functions and whose keys (inputs, pairs
     of reads in a row, equality tests) list what functions in different
     files share; its docstring says what it cannot see. `--against main`
     leaves what a branch added, and `close_out.py` prints that at every
     close.
   - **A model pass** beside it: a subagent not shown the report reads the
     code for one fact computed twice, with
     `plans/references/duplicate-inventory.md` §2a's prompt, over the files a
     branch changed at every close and over a spine phase's whole subsystem at
     its close. It is not repeatable, so the script stays the list of record.
     Each copy is routed to one helper or kept with its reason.
4. **Readiness.** Is the engine on track for the harness use case? **Its first
   duty is the ratchet's re-reading** (`engineering-practices.md#the-budget`):
   decisions per core-second on both boards at four seats and one thread, a
   dated reading beside the previous close's, never worse per decision
   without a written reason; then the three floors
   (`engineering-practices.md#the-floors`). Instruments:
   - **the census provider**, counting decisions where they reach the agent;
   - **the clone timer** with a counting allocator, at item 143's checkpoints;
   - **the fork record-and-replay**: record every answer, clone at a round
     start, replay, compare the logs verbatim (item 41's test);
   - **`plans/panic_surface.py`**: the engine's release-active panic surface,
     apart from the unit-test tails; a surface growing faster than the engine
     is the finding;
   - **the three-run contention read**: `--threads 1`, 8 and 16 at 200 games;
   - **callgrind under WSL** (`layers-architecture.md` §12's instrument
     paragraph, `plans/profile/`): the instruction count is the reading that
     travels between machines, and the milliseconds stay in this pass.

   It produces the lever list re-ranked by instruction share, each lever sized
   in its item.
5. **Scheduling.** One table: each open track phase and lattice entry, what it
   unlocks in cards (the ledger and the docs' consumer lists, no Scryfall
   count re-derived) and in atoms (`spec.sqlite` by rule prefix), what it
   needs, and which of the next spine item's design questions needs it landed
   first; then a proposed order. The pass proposes; the owner decides.
   `roadmap-v2.md` §3a's A table is the first instance.

## Across the passes

- **Docs and small fixes only.** Anything larger becomes a scheduled item with
  an owner.
- **No count is re-derived that a doc already carries with a date**, and a
  number in a heading is a number that will be wrong.
- **A pass re-reads what it touches.** Every number in a pass's brief is a
  starting point, and the brief says so about itself.
- **Discovery may fan out; fixing stays sequential.** A read-only discovery
  workflow finds, then independent verifiers try to refute each finding, each
  agent with a budget and a stop rule, and only survivors become work; the
  output is a list, never a diff (`plans/handoffs/item-6-audit.md` §2).
- **At the trace-page decision**, each phase of the item is decided on its
  heading (`engineering-practices.md#trace-pages`), and
  `check_glossary.py --suggest` is triaged.

## The hygiene pass at a long item's midpoint

A critical-path item longer than about eight PRs gets pass 3 halfway through
(the owner, 2026-10-08), planned in a handoff and dispositioned as the close's
passes are, with one more question: a shape review of the code the rest of
the item will extend. Which lists hand a position across a call or a
mutation; which facts the rules fix at a moment the code recomputes later;
where matching is done outside the one matcher; and what a per-event path
scans, read off a profile. Item 6's was A6k.

## The runs

- **The post-RE audit**, 2026-09-15, at critical-path item 5's close:
  `git show 341ebf9:plans/handoffs/post-re-audit.md`, and `codebase-state.md`,
  "Was critical-path item 5 done, and what sits before item 6? — audited
  2026-09-15". Where each pass's output went:
  `archive/engineering-practices-records.md#9-the-spine-phase-audit--a-close-is-audited-not-declared`.
- **The item 6 audit**, opened 2026-10-22: `plans/handoffs/item-6-audit.md`.
