# Handoff — the process PR

**Opened 2026-11-06** on `route/v0.5-sizing`. This is the queue for the PR that
lands beside the CI review (`plans/spine-phase-audit.md#the-ci-review`) when a
merged PR's number crosses #320. Process changes wait here and land together
there, a bug in a gate being the one exception, so an engine PR carries none.
The landing of that PR empties this file: a new queue opens with the next
change.

## Decided, to build

**The spec database becomes a reading list and a checklist when a phase starts,
not a closing gate** (the owner, in the brief for `v0.5-sizing`).

- Reword `CLAUDE.md`'s Spec database section and
  `engineering-practices.md#the-spec-database`: "a phase does not close until
  `owed` is clean" goes, and each phase reads its atoms at intake.
- Have `/next-brief` and `/design-note` read them at intake, with
  `python plans/specdb.py next --rule <its rules>`. It reads every phase and
  shows every atom since this PR; its rule is a prefix, so `702.11` also matches
  `702.110` to `702.119`.
- Make `owed` say what it checked. It filters to atoms ticketed `NEW`, then
  prints "every atom in a shipped phase is covered or was explicitly deferred"
  when that list is empty, which it has not checked. It lists 8 today.
- The board's "This is not `specdb owed`" table calls it a gate and needs the
  same rewording, in `check_state_of_play.py`'s `render`.

**A half row's ✅.** The board reads a row of the Sizing table as landed from a
✅ heading, a roadmap row, or its own State cell. A row that is half a phase or
a §C paragraph has only the last, so `/pr-handover` should say to set it.

**A row's deck cards.** From step 9's close each row registers the deck cards it
unlocks (`roadmap-v2.md`, "From triggers to v0.5"), so `/next-brief` should name
them and the PR that registers them.

**Deck cards ready on the board** (the owner, at #315). `state-of-play.md` shows
how many of the 385 could be written today, read off
`plans/references/card-census/results/gap-census-records.jsonl` by
`gap_census.py`'s `needs_left`, beside the registered count, and each step's
actual PRs against its range, so progress reads as cards and a step past its range
says to re-forecast. Stdlib and file reads only, like the rest of the board.

**Owed by the gap census, missed at #315.** `backlog.md` §2.37 gives it the parity
table of `EventPattern` against `TriggerEvent`, in `triggers-architecture.md`, about
an hour. The brief's intake read `--slotted`, which reads items and no backlog
entry, so a backlog Owner line naming a row never reached it. The intake should read
both.

**Three rules from #315's close** (the owner, 2026-11-06):

- **A deck card places a finding, never excuses one.** Every finding is filed and
  slotted, and a deck card decides only when in v0.5's order. The unbuild test comes
  first, and a design reads the mechanic's whole CR family whichever card brought it.
- **A held-out draw at each step's close:** 30 seeded cards from outside the lists,
  read for a need with no home, so overfitting to the decks is a number. The first,
  at #315's close, found 4–5 of 30.
- **`backlog.md` §2.37's slot matrix moves to v0.5's close,** ahead of card breadth
  and custom cards, as a measure and not a fill. That entry's Owner line and the
  route's "After v0.5" list need saying so.

## Open, for the owner

1. **Whether a new gate or rule raised in review defaults to the next PR.**
2. **When the next audit runs:** the hunt at each subsystem's close and one full
   audit at v0.5, each with a PR budget, or another cadence. `roadmap-v2.md`'s
   Sizing section assumes a budget for its low case: 1.5 merged PRs for each route
   PR, against 2.0 measured.
