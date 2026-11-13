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

**The midpoint hygiene pass gains a docs leg** (the owner, 2026-11-13), limited
to the facts the gates and the board read: ✅ headings, the Sizing and forecast
tables, Slotted lines, backlog Owner lines and the gap census's records. The
docs pass ran only at v0.5's close. `plans/spine-phase-audit.md`'s midpoint
section and the practices page say so.

**A backlog Owner line naming only landed rows fails the board's check** (the
owner, 2026-11-13), as a Slotted line does. Four live entries do today and are
re-homed or marked shipped first: §2.12, §2.17, §2.22 and §2.44. It is the
lapse that lost §2.37's parity table.

**Each Sizing row names its subsystem, as a column** (the owner, 2026-11-13),
and the board's "When each subsystem closes" reads it in place of the docs'
headings. 20 of the 42 rows to come are defined in no architecture doc, and
five have no design stop ahead to name them: `resolution-recipients-and-amounts`,
`game-formats`, `multiplayer-rules`, `player-scoped-values` and
`layers-combat-and-counters`. Their subsystems would never show as closed, the
first at step 7, so this lands at #340 and no later.

## Open, for the owner

**Whether the engine's floor follows the pin, and whether a pin move rides
here.** The owner asked at the move to 1.100.0 (2026-11-13) whether the
project should keep up with each Rust release. It already does, through the
CI review's last bullet: the review has come every 8 to 13 days (#280 to
#320) against Rust's six weeks, so a release is picked up within about two.
An unpinned `stable` was weighed and not proposed: every warning is an
error here, so each release's new lints would fail whichever PR was open.
Two changes are open. `rust-version` could follow the pin rather than stay at
the measured 1.88, since a newer feature such as the never type raises the
real floor and nothing notices until the next re-measure; following the pin
drops that re-measure from every move. And a pin move could land in this PR
when a release is out at a multiple of 20, rather than in a PR of its own,
since neither reads performance. A `rust-toolchain.toml` was also weighed and
not proposed: an A/B arm built at an older commit would compile with that
commit's compiler.

The two questions this queue opened with were answered in #321's review and
built there: the hygiene pass runs at the midpoint of the run to v0.5, after
step 9, and a backlog entry with no Owner line fails the board's check.
