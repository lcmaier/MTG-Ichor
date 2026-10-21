# clone-architecture.md — landed phases, evicted

**A record of finished work, not a plan.** Every section here sat under a ✅
heading in `plans/clone-architecture.md` and was moved out when its phase
shipped, leaving the heading, a stub and a pointer in the live doc. Nothing
here is owed and nothing here should be acted on; what each section is *for*
is the reasoning: the build as sized, what the building changed, the
measurement. `check_state_of_play.py` reads the ✅ headings that stay, and
fails when a landed section keeps more than 40 lines in the live doc
(`engineering-practices.md` §4). Later phases are appended by the PR that
lands them.

#### CL-1 — the board and the stack shared between forks — ✅ landed 2026-10-21

*Evicted 2026-10-21 from `plans/clone-architecture.md` §4, where §7 keeps the heading and a stub.*

### The build as sized (2026-10-21, at #266)

One PR, in the band (`engineering-practices.md` §4), its commits in decision
2's order. Sizes are code plus tests.

| PR | Shape | Size | Consumer | Closes |
|---|---|---|---|---|
| **CL-1, the board and the stack shared between forks** | The chunked table, entries inline or behind their own `Arc`, and the chunked list, each with its tests. `stack_entries`, `stack` and `pending_triggers` first, then `objects`, then `battlefield`, and `FitOnClone` deleted. The eight walks of shape 4 as reads, then writes. The memo's eviction at `remove_object`, and its frames in the table. Item 293's list (decision 5(b)), with its debug walk. Decision 6's (a), (b) and (c) | table ~300 and list ~200, each half tests; the five stores' fields, constructors and the scenario writer's destructure ~150; the eight walks ~100; the memo ~80; item 293 ~110; the three instruments ~350; iteration sites whose item type changes ~50: **1,350–1,750** | the wipe, the churn and every decision of the 24 games, under floor 2's proxy and floor 3 | items 183 and 293 |

**If the first ticket's count passes 2,500**, the seam is the board: the stack's
three stores, item 293 and the instruments first, then `objects`,
`battlefield`, `FitOnClone` and the memo as a second PR. Each half carries a
consumer: the wipe for the first, the churn and the token board for the
second.

**Predictions** (the probe's model, §2.0):

| Reading | Today | After CL-1 |
|---|---|---|
| `clone_bound_test` between turns, 4 seats | 53 allocations, 90.8 KB, 8.1 µs | 34, 12.1 KB, about 2.7 µs |
| every decision, 4 seats | 78, 94.2 KB | 37, 12.2 KB |
| every decision, 2 seats | 38, 47.5 KB | 28, 5.8 KB |
| the wipe, 8,000 stacked | 32,014, 10 MB, 8.2 ms | about 15 allocations, 15 KB, 5 µs |
| the churn, no objects left | 101.3 KB | under 8 KB |
| instructions per decision, the budget's board | | within ±2.5 points. Reads lose the hasher (§2.7); each write checks a count, and each permanent, stack entry or waiting trigger made allocates its `Arc`, under one a decision (§2.6). The direction is the arm's to read |
| gameplay rows, both pools | | **IDENTICAL**: no rule reads the change |
