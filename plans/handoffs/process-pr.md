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

## Open, for the owner

1. **Whether a new gate or rule raised in review defaults to the next PR.**
2. **When the next audit runs:** the hunt at each subsystem's close and one full
   audit at v0.5, each with a PR budget, or another cadence. `roadmap-v2.md`'s
   Sizing section assumes a budget for its low case: 1.5 merged PRs for each route
   PR, against 2.0 measured.
