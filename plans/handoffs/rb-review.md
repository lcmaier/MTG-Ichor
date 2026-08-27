# PR #62 (Phase RB) — review findings ledger

**Delete this file when the last row is closed.** Per `CLAUDE.md`'s authority
table, a handoff is where to resume half-finished work and does not outlive it.

## Why this file exists

PR #62 is +5,475 / 33 files. A review of it will produce a lot of findings at
once, and the failure mode is not the *number* — it is carrying all of them in
one working context and fixing a dozen unrelated things across nine commits in a
single pass. Quality degrades on the later ones, and it is easy to lose one
silently.

**The file breaks that coupling.** Findings land here as they are made; fix
sessions start cold from this file and need nothing from the conversation that
produced them. A session that closes one bucket is a coherent unit of work; a
session that closes one item in each of nine files is the thing this exists to
prevent.

## How to use it

1. **Capture everything first, fix nothing.** Add a row per finding, verdict
   blank. Reviewing and fixing in the same pass is what produces the scatter.
2. **Triage into one of three buckets** before any code changes:
   - `fix` — a real defect in what RB shipped. Belongs in this PR.
   - `defer` — real, but belongs to a later phase. Gets a
     `codebase-state.md` Deferred Migrations entry and **leaves this PR**.
   - `doc` — the code is right and a claim written about it is wrong. Cheap,
     and worth separating because doc fixes need no test and no re-measurement.
   Only `fix` is urgent, and on past form it is the smallest of the three.
3. **One session per bucket** — or, if `fix` is large, one session per *area*,
   because findings cluster on the nine commits by construction. "Everything in
   `gather.rs`" is coherent. "One thing everywhere" is not.
4. **Close a row by naming the commit that did it**, so a later session can tell
   a fixed row from an unread one.
5. Re-run the phase's gates once per session, not once per finding:
   `cargo build --all-targets` (zero warnings), `cargo test`, `fuzz_games
   --games 50 --seed 12345` against the recorded baseline, and `specdb orphans`
   / `suspicious` if any `COVERS` line moved.

**The RB baseline to compare against** (unchanged from pre-RB, which is the
point): P0 28 (56.0%) / P1 22 (44.0%), spells 20.8, lands 17.9, combat 11.4,
creatures died 5.6, damage events 24.4, total damage 53.6, life changes 18.0.
744 tests, 0 warnings. Perf 13.04 ms/game at `--games 200 --seed 12345`.

## Already known and deliberately not fixed in RB

Seeded so a review does not spend findings re-discovering them. Each is recorded
in a doc already; if a review disagrees with the *decision*, that is a finding —
if it merely re-notices the gap, it is not.

| # | Item | Where it is recorded | Why not in RB |
|---|---|---|---|
| K1 | CR 614.15 self-replacement has a `ReplacementClass` bucket and no producer; `ResolutionContext` still has three fields | §11 item 3, `codebase-state.md` | Lands with the first card that needs it; building the mechanism first is what §0 warns against |
| K2 | CR 614.17c's blocked path therefore always drops the event | §9's RB block | Correct today — the only class that could survive a block has nothing in it |
| K3 | §3.3 source 2 (static abilities in other zones) still unsized | §11 item 4 | **RB was asked to size this and did not.** The shape answer fell out (zone parameter on one loop + item 9's `GameObject` timestamp); the card count is still owed |
| K4 | CR 704.5q counter annihilation still writes `BattlefieldEntity` directly | Deferred Migrations item 6 | Removes two counter kinds at once and would have to join the SBA batch |
| K5 | CR 704.7's same-result collapse does not reach player loss | Deferred Migrations item 6 | Needs `GameAction::PlayerLoses` (Phase RE) |
| K6 | CR 704.7's dedupe lives in the SBA sweep, not `execute_actions` as §4.2 specifies | §9's RB block | The sweep is what knows CR order for naming the cause |
| K7 | Neither half of CR 903.9 is reachable in a real game | `codebase-state.md` CR 9 table | Nothing outside tests sets `GameObject.is_commander`; that is CR 903.7's designation hook |
| K8 | All of CR 615's prevention machinery is absent | `cards-unlocked-ledger.md` Part 3 | Phase RD |

## Findings

| # | Area / commit | Finding | Verdict | Closed by |
|---|---|---|---|---|
| | | | | |

## Notes for whoever fixes these

- **`git checkout --` on a file with uncommitted work loses it.** This bit
  during RB's mutation checks and cost a re-apply of `game_state.rs`. Copy the
  file aside instead — `cp src/x.rs /tmp/x.bak`, mutate, `cp` back.
- **Mutation-check any assertion a fix adds or changes.** RB shipped one vacuous
  test that a mutation caught: the per-batch-member applied-set test used
  counter-derived effects, which are keyed per permanent, so a deliberately
  shared applied set changed nothing. It had to be rewritten in Kalitas's shape
  — one source, an `AffectedSet::Filter`, N events.
- **A `COVERS` link is a claim.** If a fix changes what a test proves, re-read
  the atom with `specdb show` before leaving the annotation alone. RB's review
  pass found ten links that were overstated and two that were simply wrong.
