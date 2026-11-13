# Handoff — the process PR

**Opened 2026-11-13** on `process/ci-review-at-320`, as that PR emptied the
queue before it. This is the queue for the PR that lands beside the CI review
(`plans/spine-phase-audit.md#the-ci-review`) when a merged PR's number crosses
#340. Process changes wait here and land together there, a bug in a gate being
the one exception, so an engine PR carries none. A new gate or rule raised in
review joins it unless the owner says now. The landing of that PR empties this
file: a new queue opens with the next change.

## Decided, to build

**The first held-out draw from a fetch, at step 5's close.**
`plans/references/card-census/held_out_draw.py` has run on a made-up card file
only. The draw at #315 was read by hand and its needs were named and not filed:
emblems, populate, a target that is a player or a planeswalker, villainous
choice, and perhaps a spend restriction on {X}. The step 5 draw files what it
finds, and those five where they hold
(`plans/references/card-census/README.md#the-held-out-draws`).

**The nightly's start times, read at the next CI review.** Its cron moved from
07:23 to 00:23 UTC on a trial: each of its first eight runs started 6 h 46 min
to 10 h 15 min late. If the runs after this PR's merge still start between 9 am
and 1 pm Eastern, no cron moves them, and the choice is a scheduler outside
GitHub calling `workflow_dispatch`, or a midday run
(`.github/workflows/nightly.yml`'s opening comment).

## Open, for the owner

Nothing yet. The two questions this queue opened with were answered in #321's
review and built there: the hygiene pass runs at the midpoint of the run to
v0.5, after step 9, and a backlog entry with no Owner line fails the board's
check.
