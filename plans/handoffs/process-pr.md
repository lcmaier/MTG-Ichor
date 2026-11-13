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

## Open, for the owner

1. **Whether the hygiene pass at a midpoint applies to the run to v0.5.** The
   rule reads "a critical-path item longer than about eight PRs gets the
   hygiene pass at its midpoint" (`plans/spine-phase-audit.md`), and the owner's
   cadence of 2026-11-13 names the hunt at each subsystem's close and the whole
   audit at v0.5. Read onto the route, the midpoint is about step 10 of 18. Left
   as written until the owner says.
