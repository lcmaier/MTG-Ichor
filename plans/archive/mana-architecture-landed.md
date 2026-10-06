# mana-architecture.md — landed phases, evicted

**A record of finished work, not a plan.** Every section here sat under a ✅
heading in `plans/mana-architecture.md` and was moved out when its phase
shipped, leaving the heading, a stub and a pointer in the live doc. Nothing
here is owed and nothing here should be acted on; what each section is *for*
is the reasoning — the build as sized, what the building changed, the
measurement. `check_state_of_play.py` reads the ✅ headings that stay, and
fails when a landed section keeps more than 40 lines in the live doc
(`engineering-practices.md` §4). Later phases are appended by the PR that
lands them.

#### MA-1 — the inventory and the exact check — ✅ landed 2026-10-05

*Evicted 2026-10-05 from `plans/mana-architecture.md` §5 and §6's table, where §10 keeps the heading and a stub.*

### The build as sized (2026-10-05, at #224)

**§6's row.** §5's five commits: the rewind counter; `oracle::mana_supply`
(the inventory sorting only the player's permanents, the check, §3.3's
shapes for the registered cards, alternatives of different sizes and
Doubling Cube exact, §3.4's memo with its debug audit, §3.6's seam); the
gate in `can_cast` and `can_afford_ability_costs`, one inventory a priority
point; the window (§3.8); `ManaAbilityWindow`'s typed reason. A property
test: every "yes" on a random small board is a payment an exhaustive search
finds, and every "no" is not. Pools Krark-Clan Ironworks and Doubling Cube,
one card for each path it builds that no pooled card reaches, in its last
commit, after the arms read. **~600 engine, ~700 tests, ~100–250 fixtures:
1,400–1,550.** Consumer: the priority question on the budget's board, and
SU-7's review board (Grizzly Bears with one Everywhere: now short). Closes
item 162's oracle half, its offer customer.

**§5, measuring MA-1: a change that moves the agent's stream.** MA-1
changes what the priority question offers, so `new` against `main` reads a
different game, and §3.1's budget, at identical counters, cannot be read
off it (`cost-architecture.md` §6's CM-4 note, item 138). The PR's commits
are ordered so that each arm answers one question:

1. **The rewind counter, first.** A `Diagnostics` cell counted where
   `run_priority_round` takes an `Err` (`engine/priority.rs`), the row
   "Actions reversed" (CR 732.1's word) in `fuzz_games`, and its `ROWS` entry
   in `plans/fuzz_ab.py`. Arm *counter*: **IDENTICAL** on every gameplay row,
   and the new row reads 22.4 a game on the budget's board.
2. **The inventory and the check, unused by the gate.** Arm *inert*:
   **IDENTICAL**.
3. **The window reads the inventory** (§3.8). Arm *window*: **IDENTICAL**,
   about −0.7 points.
4. **The cost arm**, a throwaway patch on commit 3 in a `C:/w/arms`
   worktree, never a commit. The gate takes the inventory once per candidate
   list and answers with today's greedy, run over the inventory's per-ability
   list so its answers are today's. It also runs the exact check,
   black-boxed. Counters **IDENTICAL**. Instructions are the budget's
   reading, **predicted −2.3 points**: the new check's 30.3K plus the old
   greedy's few hundred a cost, against 45.6K. A cost arm that ran the old
   scans as well would read +5% and fail the budget for a reason that
   ships nowhere.
5. **The behavior commit**: the gate answers with the exact check, and
   `find_mana_sources` goes. Arm *shipped*: **differ**, predicted:
   "Actions reversed" falls from 22.4 to 0 on the budget's board, and about
   30 fewer decisions a game before the stream moves (317 prompts forced, 448
   re-asks gone and 134 points gained, over 20 games). Every other row moves
   with the games.

**Fixtures.** Scripted answers are positions (item 209). A test whose board
offered an over-offered card ahead of its answer renumbers. SU-7's test
`phase_su7_integration_test.rs:93` flips on purpose: "item 162's loose
offer; when it is fixed, this board says the mana is short". Count with
`cargo test --no-fail-fast` on commit 5, as A6j did. **The ratchet** re-bases
on commit 5 (item 138: a change that moves the stream re-bases it), and the
`fuzz-record.md` block says so.

### Sized against built

Lines added, a `src` file's `#[cfg(test)]` module counted as tests, read off
the PR's last code commit (`2e16467`) against #224's merge (`a1815e3`).

| Part | Where | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---|---:|---:|---:|---:|
| The rewind counter, and the diagnostics' rewind the watchers' debug audit needs | `diagnostics.rs`, `priority.rs`, `fuzz_games.rs`, `fuzz_ab.py` | ~600, together | 68 | | |
| The inventory, the check, §3.3's shapes, §3.4's watchers with their memo and audit, the window's offer, the agent's source list | `oracle/mana_supply.rs`, `replacement/gather.rs` and `mod.rs`, `layer_memo.rs` | (above) | 1,328 | ~700, together | 645, the module's own with the property test |
| The gate, one inventory a priority point; `find_mana_sources` and its helpers out (−325) | `mana_helpers.rs`, `legality.rs`, `costs.rs` | (above) | 101 | | 1 |
| The window, its why, the agent's view | `put_on_stack.rs`, `ui/why.rs`, `ui/random.rs` | (above) | 70 | | 2 |
| The pool | `registry.rs`, `phase_re9_cards.rs` | (above) | 16 | | |
| The under-offer boards, the window's refusal, a decorator | `phase_ma1`, `phase_su7`, `phase_su2`, `trace_sink_test.rs` | | | (above) | 240 |
| Fixtures: `castable_spells` returns ids | eight test files | | | 100–250 | 15 |
| **MA-1, the whole diff** | | **~600** | **1,583** | **~800–950** | **903** |

Code ran at 2.6 times its sizing, 2.0 net of the 400 lines it took out, and
tests at 1.0. Of `mana_supply.rs`'s 1,265 lines of code, 214 are comments.
Three things the sizing left out:
- **What a tap makes, read off the board**, about 440 lines: the watchers'
  scan with its memo and audit, the fold of the replacement effects over
  every order, triggered mana, and the retypes §3.4 had left for a later
  card. The sizing priced the check; the production it checks against was
  most of the module.
- **The window's offer and the agent's view**, about 140. §3.8 had the
  agent's scan as "not this design's"; it had to read the production too
  (item 4 below).
- **The cost's own consumption**, about 70: a `{T}` or a sacrifice in the
  cost being paid takes its permanent's mana out of the supply, and the
  outlets' fodder with it.

The band stayed under 2,500 at every commit: 2,486 in all.

### What the build changed in the design

1. **Retypes are read** (§3.4, amended). The design held that ignoring one
   can only over-offer; a Forest under Deep Water makes `{U}` and not `{G}`,
   so it errs both ways, and Deep Water and Pale Moon are registered. A
   resolution's replacement effects are read at each inventory, not memoized
   by epoch: making one moves no layer input.
2. **The Cube is a transform of the demand** (§3.3, amended), not an
   enumeration of the entries that pay its `{3}`: a pip needs `ceil(n / f)`
   of its type left after the input, the cost `ceil(total / f)` of any. The
   design's example board paid `{W}{U}` without the Cube, so the amended
   section has one that does not.
3. **What Ironworks sacrifices after the Cube's doubling pays undoubled**
   (`c07ef44`, found writing item 213's list against the registered cards):
   the Cube itself, then the last Ironworks, go after the doubling or not at
   all. The first build doubled them: with eight Plains, Ironworks and the
   Cube it offered `{15}` where the board makes fourteen.
4. **The window offers what the check counts** (§3.8, amended), so a payment
   the check found is one the window can make: Doubling Cube, which no window
   offered before. The random agent's view reads each ability's production
   on the board as the check does; without it, it declined windows a payment
   covered, a Wild Growth land's `{G}` among them.
5. **The window re-asks only costs** (§3.8, as built): it takes its offer
   once, one entry per ability definition, re-asks each ability's costs at
   each prompt (a `{T}`-only one by its tapped flag), and reads the offer
   again when the epoch moves.
6. **Copies of a way are capped, and only when every way is the same
   size** (`c07ef44`). Twenty Everywheres under Mana Reflection against
   three colored kinds were 1,771 leaves a check; past what one type's pips
   need, a copy pays only generic. A first draft capped ways of different
   sizes too, and a fixed test holds that case.
7. **Decisions 4 and 5 as taken.** `can_cast` returns `Ok(())`, and
   `ManaSource` lives in `mana_supply` as the agent's per-ability list. An
   ability's mana is checked last, after its other costs in printed order
   and its targets; SU-2's Mind Stone line now reads "the engine does not
   offer this", since an over-offer was the only natural board for its
   "reversed" refusal.
8. **The fixtures were three tests, not 100–250 lines**: SU-7's flip (on
   purpose), SU-2's line (item 7), and a trace test that waited for a random
   over-offer to reach a rejection, whose seat now declines its first
   window, the one rejection an exact offer cannot see coming. The 15 lines
   in eight files are `castable_spells`' return type.
9. **The cost arm read twice.** Before any arm, `d495df4` kept the
   supply's split once an inventory: the check had rebuilt it at each of
   about 40,000 checks, nearly a point the design's price did not have. On
   `d495df4` the arm read +0.72%, against −3.0 predicted: the inventory cost about 29K instructions a take, where the
   prototype priced 18.4K, and the patch's greedy walked the per-ability
   list, not the design's counts. `eabdc9e` takes the inventory without
   allocating for what is not there, the patch moved to counts per type,
   and the arm read −1.96% (`fuzz-record.md`, MA-1's block).
