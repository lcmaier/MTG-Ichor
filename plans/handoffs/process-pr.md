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

**Each crate's `rust-version` follows the pin** (the owner, at #322's merge,
2026-11-13), rather than staying at a measured floor. A newer feature such as
the never type raises the real floor, and nothing notices until the next
re-measure; nobody builds the tree on an older compiler. Both `Cargo.toml`s
take the pinned version, and the comments on the pin in `ci.yml` and on
`rust-version` in `mtgsim/Cargo.toml` stop asking for a floor to be read, as
does the CI review's last bullet in `plans/spine-phase-audit.md#the-ci-review`.

**A pin move lands in this PR** (the owner, at #322's merge, 2026-11-13). When
the CI review finds a newer Rust out, the move rides here instead of costing a
PR of its own: neither reads performance, and the next engine PR's baseline is
built on the new compiler either way. The same bullet says so. Rust ships every
six weeks and the review has come every 8 to 13 days (#280 to #320), so a
release still lands within about two. An unpinned `stable` and a
`rust-toolchain.toml` were weighed at #322 and not proposed: every warning is an
error here, so each release's lints would fail whichever PR was open, and an A/B
arm built at an older commit would compile with that commit's compiler.

## Open, for the owner

**Should the mana pool join the write gate's fields?** Raised at
`mana-provenance`'s design (2026-11-13). `check_invariants.py`'s `EVENT_FIELDS`
does not list a player's mana pool, so `turns.rs` empties pools and `costs.rs`
spends from them outside `perform_action` with no tag. That design routes the
emptying through the chokepoint and tags the spending
(`mana-architecture.md#decision-5-mana-kept-past-a-steps-end-and-mana-lost`
and its decision 4). The gate would then keep it so: a pool written anywhere
but those two places fails. If yes, it lands with `mana-kept-and-lost`, the row
that removes the last untagged write.

**Should a design note's proposed names be audited before the owner reads
them?** Raised at #323's review (2026-11-13), where the owner asked for most of
the note's proposed types to be renamed: bare `Spend`, `Rider`, `Source` and
`Pip` words, two beside names the tree already has (`Rider` in the replacement pipeline,
`SourceFilter` in the "can't" registry). The rule exists
(`engineering-practices.md#names`), and the `owner-review` agent's first check
is that rule, with "rider" among its own examples. But at a design stop the
agent applies checks 2, 4, 6, 12 and 17 only, and `/design-note` has no step
for names, so a note's code blocks reach the owner unaudited. Proposed: the
agent's design-stop list gains check 1, read over every type, field and arm
the note proposes, and `/design-note`'s step 3 gains a line: each proposed
name carries its subsystem's noun and is grepped against `src` for a taken or
a near name. If yes, it lands in the next process PR.

Nothing else open. The question raised at #322 was answered at its merge, as the two
entries above. The two this queue opened with were answered in #321's review and
built there: the hygiene pass runs at the midpoint of the run to v0.5, after
step 9, and a backlog entry with no Owner line fails the board's check.
