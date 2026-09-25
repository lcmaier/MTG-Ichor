# Fuzz record — the pooled fixture tables, phase by phase

Every `**Re-recorded …**` block the project has written, moved out of
`engineering-practices.md` §3 on 2026-09-13. §3 had grown to about 2,000 lines
and roughly 1,400 of them were record; the rules the section exists for were the
shorter half of it. The rules stayed there; the numbers came here.

**This is a record, not a plan.** Nothing here schedules work, owns a decision or
grants authority to anything. Each block is what one PR measured, written after
that PR's A/B — evidence about a tree that existed on a date, never a target, a
budget, or a claim about the tree you have now. The rules that govern the
measuring stay in `engineering-practices.md` §3: which pool answers which
question, what a row may hold and what it may not (`ms/game` may not, and why),
and how to read the five bold counter rows. Read those there; read this for what
the numbers were.

**Newest first, and a phase appends at the top.** The run this file was cut from
had stopped being ordered — later phases were appended *below* LI-3 while the
LI/LH tail kept its original newest-first order — so it was re-sorted on the
move, same-date ties in merge order. **A phase that moves a pool re-records both
the `performance` and the `stress` table and adds its block here**; that rule is
`engineering-practices.md` §3's and stays there, restated here only so the next
person appending knows where the numbers go.

**Three runs were reunited with the entries that own them**, because sorting by
date is only meaningful once each entry is whole. They had drifted into the
middle of RE-7's block: LI-3's three-arm A/B (what LI-3's summary calls "the
three-arm table below", 900 lines further down and inside another entry); the
CR 704.5p fix's fixture re-record, including a table row orphaned from its own
header; and a run of instrument doctrine from the RD-3 review, which is a rule
rather than a measurement and went back to `engineering-practices.md` §3. No
sentence in any of them was changed.

**What the move preserved.** Every `**Re-recorded …**` heading is byte-identical
— architecture docs and `plans/archive/` cite them verbatim and grep for them —
and so is `### 3.1a`, which keeps its old section number for the same reason:
two live docs name it by that number, and breaking them to tidy a label is not
worth it.

**Re-recorded 2026-09-24 for TR-2a** (the histories, the gates and each
player — `triggers-architecture.md` §12, TR-2a; `codebase-state.md` items 122
and 172 closed, 171 half closed). **Pool change**: `performance` goes 94 → 96
(Vengeful Warchief, Elvish Warmaster) and `stress` 171 → 175 (plus Paladin of
Atonement and Temple Bell). The shipped columns are therefore a new baseline,
and the engine columns are the ones comparable to TR-1b's.

**Predictions, written before any arm ran.**
- Each engine step against `main` reads every gameplay row `IDENTICAL` on both
  pools.
- On `stress`, the rider's new order changes which draw line comes first, not
  which cards anyone draws.
- Lifelink's summed gain changes no life total, and no pooled card triggers on
  a single gain.
- The Layer 4 fix likely moves no game at seed 12345.
- The cost rows move up with the histories, which read a spell's types as it
  is cast.
- CPU per decision stays inside the budget, and the audit agrees on every arm.

**The A/B, five arms, `fuzz_ab.py`**: `main` at #185's merge (`71b3caa`);
**each** (`0e5f34a`, item 122, the last commit before lifelink);
**lifelink** (`925781d`); **engine** (`4c6d469`, the engine complete with the
four cards unregistered); **shipped** (`dd5d48b`). Both pools at two seats and
four, 200 games at seed 12345, timing 3×200 on `performance`, and the counter
runs audited.

| | 2 seats | 4 seats |
|---|---|---|
| gameplay rows, each, lifelink and engine vs `main`, both pools | **IDENTICAL** | **IDENTICAL** |
| `Layer walks`, `main` → each, `performance` / `stress` | 358 → 380 / 487 → 504 | 764 → 805 / 1,057 → 1,086 |
| `Candidate visits`, `main` → engine, `performance` / `stress` | 31.0 → 31.0 / 30.2 → 30.3 | 109.0 → 111.0 / 112.7 → 113.9 |
| `Life changes`, each → lifelink, `performance` | 17.4 → 17.3 | 46.2 → 46.1 |
| `µs / decision`, engine vs `main` — §3.1's budget | 30.6 → 31.7, +3.8% | 48.0 → 49.4, +2.9% |
| the same, re-taken at five rounds (three arms) | — | 49.1 → 49.8, **+1.3%**; each +0.1% |
| audit, engine arms, `performance` / `stress` | 177,397 / 209,234 agreed | 360,439 / 410,372 agreed |
| deterministic, every arm | yes | yes |

**The budget.** The three-round sitting read the engine arm over the
four-seat line, at +2.9%. The re-take at five rounds read +1.3%, with the
each arm at +0.1%, so the first reading sat inside the sitting's own spread
(~2–6%, `fuzz_ab.py`). Both are recorded here for the reviewer. The review
asked for another reading: at seven rounds the engine arm read **−0.2%** at
four seats and −2.4% at two. Three sittings of the same two binaries read
+2.9%, +1.3% and −0.2% at four seats, so TR-2a's cost is below what a timing
sitting can resolve.

**The cost rows, attributed by bisect** (each commit built alone, 200 games,
four seats, `performance`):
- **`Layer walks` +41 (+5.4%) is the histories commit** (`88d78a1`), as
  predicted: one walk per cast, for a spell the memo has not seen yet. The new
  walks are cheap. `Frames/walk` falls 19.53 → 18.60 and `ms / 1,000 walks`
  falls 3–5%, so the rows move more than the time does.
- **`Candidate visits` +0.6 is the `CastsSpell` commit** (`bf7e08e`), and
  the prediction missed it. `main` gave `SpellCast` no kind, so a cast's
  window returned at the kinds probe. With a kind, the window passes that
  probe. Where a granted or zone-map trigger exists, it then takes the
  whole-list legs, which the mask does not narrow (§11).
- **The engine arm's further +1.4 is the Layer 4 fix**, the only runtime
  change between lifelink and engine. A batch that departs Blood Moon or Urborg
  now takes a snapshot, so its window reaches the frame legs, and `Windows past
  gate` goes 70.8 → 70.9.

**Game by game**: the 200-game `--dump-events` streams of neighboring arms,
diffed per game.

| step | `performance`, 2 / 4 seats | `stress`, 2 / 4 seats |
|---|---|---|
| `main` → each | 0 / 0 | 0 / 1 |
| each → lifelink | 103 / 139, 7 / 16 of them merging | 76 / 115, 2 / 9 of them merging |
| lifelink → engine | 0 / 0 | 0 / 0 |

- **`main` → each.** The one game is four-seat `stress` game 146. P0 is
  active and controls Alms Collector, and P1's draw-two is replaced. `main`
  drew P1's card first, where CR 121.2c draws the active player's first. The
  lines are the same, reordered.
- **each → lifelink.** In every differing game, the event sequences are
  identical once the `LifeChanged` lines are removed. Every player's life
  total is also identical at every step boundary.
  - A gain now comes after its batch's other members (CR 120.4c–d).
  - A lifelinker's simultaneous gains merge (CR 702.15e): 7 / 17 fewer gain
    lines on `performance` and 2 / 13 on `stress`.

  The prediction foresaw the merge and not the move, which is the larger
  half: any lifelink damage dealt in one batch with other damage moves its
  gain.
- **lifelink → engine.** No game differs.

**The shipped arm is a pool change and is not budgeted.** CPU per decision
against `main` is −1.2% at two seats and +1.0% at four. `Triggers placed`
reads 1.2 / 3.6 on `performance` and 1.3 / 3.6 on `stress`. The two pooled
cards take slots, so fewer TR-1 triggers are dealt.

**Reachability** (`--require`, shipped, `performance`, 200 games):

| | 2 seats | 4 seats |
|---|---|---|
| Vengeful Warchief — cast / resolved / games / copies per deck | 165 / 164 / 109 (54%) / 1.44 | 123 / 123 / 104 (52%) / 2.90 |
| Elvish Warmaster | 189 / 187 / 135 (68%) / 1.43 | 119 / 118 / 99 (50%) / 2.88 |
| `Triggers placed`, with the two forced | 1.7 | 5.1 |
| board diversity | 200 of 200 | 199 of 200 |

**The audited sittings on the shipped arm found zero disagreements.**
- A thousand games at seed 777 on both pools, at two seats and four:
  864,066 / 1,772,511 dispatches on `performance`, and 999,769 / 2,015,689 on
  `stress`.
- The four TR-2a cards ×4 on `stress`: 208,865 / 390,836 dispatches, and 6.3 /
  13.4 triggers placed a game.

Before the Layer 4 fix, the four-seat `stress` sitting at seed 777 panicked in
game 889. That was the audit's first catch on a shipped pool
(`triggers-architecture.md` §4.10).

**§3 fixture rows, shipped, 50 games / seed 12345**

| | performance | stress |
|---|---|---|
| Wins by seat | 29 (58.0%) / 21 (42.0%) | 27 (54.0%) / 22 (44.0%) |
| Wins by effect | 0 | 1 |
| Avg turns | 28.7 | 31.4 |
| Spells cast | 21.8 | 22.6 |
| Lands played | 17.5 | 18.7 |
| Combat w/ atk | 9.5 | 9.3 |
| Creatures died | 6.8 | 5.0 |
| Damage events | 20.8 | 20.2 |
| Total damage | 57.3 | 59.4 |
| Life changes | 13.1 | 16.3 |
| **Layer walks** | **363** | **510** |
| **Board walks** | **223** | **313** |
| **Memo hits** | **56,579** | **97,183** |
| **Layer frames** | **3,985** | **6,565** |
| **Frames/walk** | **10.98** | **12.88** |
| **Dependency checks** | **23** | **20** |
| **Replacement gathers** | **1032** | **1267** |
| **Restriction queries** | **1034** | **1271** |
| Mana productions | 84 | 134 |
| Prevention allocations | 0.00 | 0.00 |
| Replacement prompts | 0.10 | 2.64 |
| Max batch depth | 4 | 4 |
| Decisions | 220 | 388 |
| Priority decisions | 83 | 154 |
| Triggers placed | 0.6 | 1.3 |
| Windows past gate | 23.3 | 48.1 |
| Candidate visits | 39.9 | 66.6 |
| Trigger matches | 2.0 | 2.0 |

The four-seat fixture rows are in the sitting's output (`--players 4`,
shipped), `performance` / `stress`: `Layer walks` 745 / 1,200, `Memo hits`
179,434 / 290,725, `Decisions` 431 / 775, `Triggers placed` 2.2 / 2.8, and
`Turns after a departure` 19.6 / 19.6.

**Determinism**: three `fuzz_games` runs under `MTGSIM_HASH_SEED` 1, 2 and 3
(200 games, seed 12345, shipped) were identical line for line outside `===
Timing ===`, on both pools at two seats and four. Every timing round of the
sitting also reproduced its counter run (`deterministic: yes` on all five
arms).

**Review round 1** (the owner's review of #186). The round's head was run
against the first head, both pools, two seats and four. Counters were
`IDENTICAL`, and the audit agreed on the same dispatches. The round only
refactored: it changed the history's shape, the recipient's shape, names,
and the turn the opening hands are drawn in. **Review round 2** (lifelink's
gain from the batch's own damage, no state) was also `IDENTICAL` against round
1's head on both pools at two seats and four.

**Re-recorded 2026-09-24 for item 176** (a redirect keeps the act —
`codebase-state.md` item 176; `GameActionTemplate::ZoneChangeTo` names only a
destination). **No pool change**, so there is no new fixture table:
`performance` stays at 94 and `stress` at 171. What moves is a label on the
performed record, and one prompt.

**Expected, as the brief stated it before the run:** a `differ` only where a
redirect moves. Academy's CR 616.1 prompt now comes in both orders under
Leyline of the Void, and a move that Rest in Peace or Leyline redirects changes
its label.

**The A/B, `fuzz_ab.py`**: `main` at #183's merge (`ebe9266`) against the fix
(`c7b9a1b`), 200 games, seed 12345, both pools, timing 3×200 on `performance`.
**Counters `IDENTICAL` on both pools**, `Replacement prompts` included (0.14 /
1.80), and the audit agreed on both arms (177,397 / 209,234 dispatches).
Timing: 6.94 → 6.96 ms/game median (+0.3%), µs/decision 29.5 → 29.6,
deterministic on both arms. The counters cannot see a label, and seed 12345's
200 `stress` games never dealt a discard that Leyline and an Academy both
watched with Leyline chosen first.

**Game by game**, the 200-game `--dump-events` streams diffed per game. A line
counts as the same event when only its `[cause]` differs:

| board | games differing | lines, cause only | any other difference |
|---|---|---|---|
| `performance` | 7 of 200 | 10 | 0 |
| `stress` | 77 of 200 | 478 | 0 |

Every line is a redirected move that now carries its act. On `performance`,
all 10 are `Hand -> Library` from `[PutIntoLibrary]` to `[Discarded]`: a Nexus
of Fate or a Darksteel Colossus discarded and shuffled in instead. On `stress`:

- `Battlefield -> Exile`, from `[Exiled]`: 202 `DestroyedBySba`,
  25 `Sacrificed`, 14 `Destroyed`, 7 `ZeroToughness`, 5 `AuraSba`;
- `Stack -> Exile`: 147 `Resolved`, 3 `Countered`. `Stack -> Library`:
  32 `Resolved`, Nexus of Fate;
- `Hand -> Exile`: 30 `Discarded`. `Hand -> Library`: 11 `Discarded`;
- `Hand -> Exile`: 2 `PlayedAsLand`, a Dryad Arbor played under Containment
  Priest, which is decision 2's other act that is its destination.

**The Leyline board**: `--pool stress --require "Leyline of the Void,Nephalia
Academy,Mind Rot,Hymn to Tourach" --copies 4`, 200 games, seed 12345, `main`
against the fix. `Replacement prompts` goes 1.90 → 2.31 per game. 169 of 200
games differ, 142 by labels only, and 27 change course. Each was attributed at
its first divergence that is not a label:

- **15** at a discard Leyline was chosen for, where Academy is now offered next
  and accepted: `Hand -> Exile [Exiled]` becomes
  `Hand -> Library [Discarded]`;
- **10** after an earlier discard of that shape where Academy was offered and
  declined. The outcome matches `main`'s but for the label, and the prompt
  spent a draw of the random provider's stream, so a later choice differs;
- **2** after a Nexus of Fate or Darksteel Colossus discard (games 164 and
  174), where the discarding player's Academy is now offered after the card's
  own redirect. It is the same CR 616.1f reader.

**Determinism**: three `fuzz_games` runs, `stress`, 200 games, seed 12345,
under `MTGSIM_HASH_SEED` 1, 2 and 3, were identical line for line except the
timing lines.

**Re-recorded 2026-09-22 for TR-1b** (the dispatch audit — `triggers-architecture.md`
§4.10 and §12's TR-1b row — with main item 174's fix as its first commit, the
dispatcher's counts as three rows, and the first Commander-scale callgrind
reading). **No pool change**, so there is no new fixture table: `performance`
stays at 94 and `stress` at 171. What this block records is four arms'
attribution, the audit's sittings, what it costs, and a baseline.

**Predictions, written before any arm ran.** 174's fix against `main`:
gameplay moving only where a batch departs Humility ahead of a creature whose
look-back ability it had removed (or March ahead of an artifact it animated),
each diverging game attributed; the cost rows moving down, since the frame
became a memo read before the batch performs where it was an uncached board walk per
departure. The audit arm unaudited against 174's: `IDENTICAL`. The audit arm
audited against itself unaudited: `IDENTICAL` counters. The counts arm
against the audit arm: the three new rows and nothing else. Every audited
sitting: zero disagreements.

**The A/B, four arms, `fuzz_ab.py`**: `main` at #178's merge (`28a1d18`),
**fix174** (`e25ceb7`), **audit** (`e083bbd`), **counts** (`45d32e7`). Both
pools, two seats and four, 200 games, seed 12345, timing 3×200 on
`performance`. The sitting now audits its counter runs on every arm whose
binary has `--audit` (audit and counts), and its timing rounds never.

| arm | counters | µs / decision vs `main`, two seats / four |
|---|---|---|
| fix174 vs `main` | every gameplay row identical; six cost rows move, all down | −1.9% / −5.9% |
| audit (audited) vs fix174 (unaudited) | **IDENTICAL** on both pools at two seats and four | −0.8% / −6.1% vs `main` |
| counts vs audit, both audited | **IDENTICAL** outside the three new rows | −0.7% / −6.5% vs `main` |

The six rows, `performance` / `stress`: at two seats `Layer walks` 366 → 358 /
494 → 487, `Board walks` 245 → 237 / 317 → 310, `Memo hits` 64,525 → 64,559 /
101,673 → 101,707, `Layer frames` 4,748 → 4,551 / 7,071 → 6,905, `Frames/walk`
12.96 → 12.71 / 14.32 → 14.19, `Dependency checks` 30 → 29 / 18 → 17; at four
seats 815 → 764 / 1,107 → 1,057, 540 → 489 / 643 → 593, 196,290 → 196,412 /
284,988 → 285,114, 16,765 → 14,922 / 21,574 → 19,750, 20.56 → 19.53 /
19.48 → 18.69, 106 → 92 / 129 → 117. The four-seat saving is the larger
because a player leaving departs a whole board, and each permanent was one
uncached walk.

**What moves in fix174, game by game** — the 200-game `--dump-events` streams
of `main` and fix174 diffed per game. `performance`: 0 of 200 at either seat
count. `stress`: 1 of 200 at two seats and 5 of 200 at four, and in **every
one the only difference is a frame's type annotation** — seven lines in six
games, `Creature` → `Creature Land` (one `Artifact Creature` →
`Artifact Creature Land`), where Ashaya, Soul of the Wild ("nontoken creatures
you control are Forest lands") left in the same batch ahead of the creature.
The fix frames the creature before the event, a land; `main` framed it after
Ashaya left. Named by name at two seats and, for the three four-seat games
whose owners had left, by replaying each alone under a turn cap (seeds 12386,
12504, 12523). No trigger and no decision moved: every watcher in those games
reads "a creature", which both frames are. So the prediction's shape held and
its example did not: the shipped pools reach item 174's appearance half
through Ashaya, not March, and reach its trigger half not at all.

**Reachability** — `--require "Humility,Blood Artist" --copies 4`, 200 games,
`main` against fix174 game by game. On `stress` **1 of 200 at two seats and 1
of 200 at four change an answer**, both item 174's board exactly: one
state-based check after a Pyroclasm (seed 12413) and after combat (seed
12502) moves Humility ahead of a Blood Artist, and `main` triggers that
Blood Artist on every death in the event where fix174 triggers it on none.
Four more four-seat games differ by the Ashaya annotation only. On
`performance` 0 of 200 at both seat counts. `Triggers placed` reads 2.0 and
4.5 on both arms: one game's three triggers are below the row's precision.

**The audited sittings** — `fuzz_games --audit` on the counts arm, 200 games,
seed 12345, **zero disagreements on every board**:

| board | dispatches answered twice | triggers agreed | `Triggers placed` |
|---|---|---|---|
| `performance`, two seats / four | 177,397 / 360,439 | 538 / 1,219 | 1.5 / 4.2 |
| `stress`, two seats / four | 209,234 / 410,372 | 331 / 965 | 1.0 / 3.7 |
| Commander scale (four seats, 100 cards, 40 life), `performance` / `stress` | 493,427 / 556,543 | 1,657 / 1,623 | 4.7 / 5.3 |
| `stress`, Humility and Blood Artist ×4, two seats / four | 248,868 / 557,494 | 567 / 1,118 | 2.0 / 4.5 |
| `performance`, the three TR-1 cards ×8, two seats / four | 263,880 / 602,841 | 13,228 / 36,662 | 47.5 / 149.6 |

Audited against unaudited, the audit arm's output outside `=== Timing ===` is
identical line for line on both pools at two seats and four (by hand), and
every timing round of the sitting above, unaudited, reproduced its audited
counter run (`deterministic: yes` on all four arms at both seat counts).

**The audit bites** — recorded, not committed; each a one-line switch in a
release build, the tree restored after. **Item 167's snapshot off**
(`departs_an_ability_list_source` answering false): the forced Humility
board on `stress` panics in game 175 (seed 12519): Humility and Parallel
Lives die in one state-based check, and the dispatcher triggers the surviving
Blood Artist twice where the reference says none. **Item 174's capture
off** (the move walking the board it finds): the same board panics in game
69 at two seats and game 158 at four, Humility moved ahead of a Blood Artist
and the dispatcher triggering on each death. **F1 as it stood** (CR 113.6
asked of the object, and the per-condition check round 1 added to
`match_def` off with it — `match_def` is shared with the reference, and the
per-condition check alone masks the per-def one on Dread, whose two
abilities have one condition each): `the_audit_agrees_over_a_zone_map_card`
panics at the damage window, naming Dread's damage trigger as the
dispatcher's alone, before the fixture's own assertion.

**What the audit costs** (decision 2: every zone, libraries included, unless
the cost said otherwise). CPU per game, serial, audited against unaudited:
7.7 → 85.0 ms at two seats (×11), 23.0 → 314.9 ms at four (×14), 39.3 → 750.3
ms at Commander scale (×19, 50 games). A threaded 200-game audited sitting is
3 s at two seats, 8 s at four and about 20 s at Commander scale, which is
what an A/B's counter runs now pay per arm and pool. Not prohibitive, so
libraries stay in reach.

**The dispatcher's rows** read 25.5 / 31.0 / 2.7 (`Windows past gate`,
`Candidate visits`, `Trigger matches`) on `performance` at two seats and
70.8 / 109.0 / 6.1 at four; with eight copies each of the three TR-1 cards at
two seats, 189.0 / 712.9 / 66.1. `triggers-architecture.md` §11.1's throwaway
probe read 25 / 30 / 2.7 and 189 / 707 / 66.1 on the first two boards.
Windows and matches reproduce it; candidates are counted per ability list,
and counting distinct objects instead gives 30.6 and 708.9, so the per-list
definition is part of the gap and the rest is not attributed.

**Callgrind at Commander scale — the baseline.** `plans/profile/`'s scripts
(new in this PR; the board is their argument), `--games 200 --seed 12345
--pool performance --players 4 --deck-size 100 --life 40`, `main` and the
head (`45d32e7`) built and run in WSL the same hour; each arm's counters
identical native and under valgrind. **Totals: 99,927,135,440 instructions
on `main`, 95,957,039,123 on the head (−3.97%).** The whole difference is
the CR 603.10a capture: `compute_characteristics_uncached` is 4.00 G (4.01%)
on `main`, 13,414 calls, and has no row on the head, where the frame is a
memo read before the batch performs. Inclusive rows, first row per function, down to 6% of
the head:

| function | `main` (G) | head (G) | head % |
|---|---|---|---|
| `Game::run_turn` | 98.12 | 94.16 | 98.1% |
| `GameState::run_priority_round` | 83.37 | 79.39 | 82.7% |
| `GameState::check_state_based_actions` | 39.03 | 35.22 | 36.7% |
| `layers::compute::compute_characteristics` | 33.21 | 33.25 | 34.6% |
| `layers::board::compute_board_to` | 22.49 | 22.39 | 23.3% |
| `GameState::execute_batch_inner` | 26.32 | 22.32 | 23.3% |
| `replacement::pipeline::apply_replacements` | 19.80 | 19.72 | 20.5% |
| `oracle::legality::candidate_priority_actions` | 18.33 | 18.34 | 19.1% |
| `replacement::gather::gather` | 17.04 | 16.99 | 17.7% |
| `oracle::characteristics::is_creature` | 15.88 | 15.90 | 16.6% |
| `GameState::execute_actions` | 14.33 | 13.97 | 14.6% |
| `oracle::mana_helpers::available_mana_sources` | 12.47 | 12.47 | 13.0% |
| `oracle::mana_helpers::castable_spells` | 12.11 | 12.11 | 12.6% |
| `layers::board::perform` | 12.49 | 10.89 | 11.4% |
| `GameState::drain` | 10.41 | 10.42 | 10.9% |
| `GameState::resolve_top_of_stack` | 10.04 | 9.91 | 10.3% |
| `GameState::battlefield_ordered` | 9.85 | 9.85 | 10.3% |
| `GameState::change_zone` | 8.87 | 8.85 | 9.2% |
| `__rust_alloc` | 9.75 | 8.81 | 9.2% |
| `RandomState::hash_one::<&CardType>` | 8.94 | 8.56 | 8.9% |
| `GameState::run_mana_ability_window` | 8.22 | 8.22 | 8.6% |
| `GameState::cast_spell` | 8.12 | 8.09 | 8.4% |
| `__rust_dealloc` | 8.01 | 7.66 | 8.0% |
| `GameState::battlefield_ids_ordered` | 7.61 | 7.53 | 7.8% |
| `oracle::mana_helpers::find_mana_sources` | 7.32 | 7.32 | 7.6% |
| `driftsort_main::<(ObjectId, &PermanentState)>` | 7.27 | 7.27 | 7.6% |
| `GameState::perform_sba_and_triggers` | 6.75 | 6.70 | 7.0% |
| `layers::board::write_affected` | 7.38 | 6.43 | 6.7% |
| `oracle::characteristics::get_effective_abilities` | 6.37 | 6.38 | 6.6% |
| `oracle::mana_helpers::activatable_abilities` | 6.05 | 6.05 | 6.3% |
| `layers::board::row_affected` | 6.61 | 5.76 | 6.0% |

The dispatcher itself (`dispatch_inner`, which `find_matches` inlines into on
the head) is 1.72 G on `main` and 1.73 G on the head, 1.8%: §11.2's 2% at this
scale, now an instruction count. Two rows no phase has named sit in the top
third: the sort behind `battlefield_ordered` (10.3% inclusive, 7.6% of it
`driftsort` over `(ObjectId, &PermanentState)`) and SipHash over the type set
(`hash_one::<&CardType>`, 8.9%). They are a reading, not a plan.

**Determinism** holds under three `MTGSIM_HASH_SEED`s, line for line outside
`=== Timing ===`, `stress`, 60 games, seed 99, at two seats and four, audited
and unaudited.

**Review round 1, same PR** (the owner's review of #179, 2026-09-23). The
audit above was a second matcher; the review rebuilt it as the dispatcher's
own loop run over a candidate set with no shortcut in it
(`triggers-architecture.md` §4.10), fixed the one bug that matcher had found
in the shared loop (CR 603.2c: an ability matched through a survivor's two
lists triggered once per list), and tidied three things. **Prediction,
written before the sitting:** the round's head (`1a09270`) against the first
head's engine (`45d32e7`; `aef30d6` changed no code) `IDENTICAL` on every row,
audited counter runs included, since the CR 603.2c fix is unreachable on the
pools and the rest is refactoring or an audit that leaves no trace.
**Result:** `IDENTICAL` on both pools at two seats and four, 200 games, seed
12345, the audit agreeing on the same 177,397 / 209,234 / 360,439 / 410,372
dispatches; CPU per decision −0.6% and −0.7%, inside the spread;
deterministic on every arm. On `stress`, audited and unaudited are identical
line for line at both seat counts.

**What the audit costs now**, serial CPU per game audited against unaudited:
7.0 → 15.4 ms at two seats (×2.2, was ×11), 22.6 → 53.2 ms at four (×2.3,
was ×14), 38.4 → 102.9 ms at Commander scale (×2.7, was ×19, 50 games). A
threaded audited sitting is 1–6 s on every board below, where the
Commander-scale one took 23 s. Most of the difference is that an object no
row reaches and that printed no triggered ability is skipped before a frame
is computed, which is almost every card in a library or a hand.

**Zero disagreements** on the same boards as above — both pools at two seats
and four, Commander scale on both pools, Humility and Blood Artist ×4 on
`stress`, the three TR-1 cards ×8 on `performance` — with the same dispatch
and trigger counts. **It still bites**: item 167's snapshot off panics in
game 175 at two seats (and seven games at four), item 174's capture off in
game 69 at two seats and game 158 at four, the same games the first audit
named. **F1's demonstration no longer reproduces**: CR 113.6 asked per
ability is inside the loop the two answers share. Determinism holds under
three hasher seeds, audited and unaudited, at two seats and four.

**Re-recorded 2026-09-22 for the TR-1 review, theme E** (provenance ids —
`triggers-architecture.md` §3.6's amendment as built — and main item 167's
look-back snapshot, §4.3). **No pool change**: `performance` stays at 94 and
`stress` at 171, and every row of the 2026-09-19 tables still reads. What
this block records is two arms' attribution and one cost.

**Predictions, written before any arm ran.** Provenance against `main`:
`IDENTICAL` on every row, because the mana window dedupes on the definition
and the elision's def key equals the old id key for the five pooled
triggers. Both against provenance: gameplay identical except where Humility
leaves in a batch beside a death while a Blood Artist survives; the cost
rows moving only where an ability list's source leaves while a Blood Artist
is on the battlefield; CPU inside the spread for both.

**The A/B, three arms, `fuzz_ab.py`**: `main` at #177's head (`073f348`),
provenance at `83dcc58`, both at the next commit. Both pools, two seats and
four, 200 games, seed 12345, timing 3×200 on `performance`.

| arm | vs `main`, counters | CPU/game vs `main`, two seats / four |
|---|---|---|
| provenance | **IDENTICAL** on both pools at two seats and four | −0.3% / +1.1% |
| both | **IDENTICAL** at two seats; at four, cost rows and three games' play (below) | −0.1% / +0.1% |

**The first sitting missed one prediction, and the miss was the agent's.**
Provenance read one game in 200 different on `performance` at two seats
(seed 12543): `ui/random.rs`' mana preference counted a permanent's
flexibility per `(permanent, ability)` entry, so under two Citanul
Hierophants every creature's two instances of `{T}: Add {G}` counted twice
where they had collapsed into one key, and the agent's least-flexible pick
moved. Keyed on the definition, as the window is, in provenance's own
commit; the re-run above is `IDENTICAL`, and 0 of 200 games differ replayed
one by one.

**What moves in "both", game by game** — each of the 200 games replayed
alone on both arms. At two seats 10 `performance` and 3 `stress` games
differ, each by 1–4 `Memo hits` and nothing else, the snapshot's own memo
reads, below the table's precision. At four seats 24 and 10 games differ in
the cost rows alone, and **three change an answer, all three item 167's
board**, named off each game's trace on the provenance arm: seeds 12352 and
12457 on `performance`, where one state-based check kills a creature and
makes P2 lose, P2's Humility leaves the game in the same event (CR 800.4a)
and a surviving Blood Artist no longer triggers on the death; and seed
12366 on `stress`, an Opalescence-animated Humility dying in combat beside
Parallel Lives.

**Reachability** — `--require "Humility,Blood Artist" --copies 4`, 200
games, seed 12345, both arms. On `performance` no game changes an answer at
either seat count, since the pool can remove Humility only by its
controller leaving, while 125 and 172 games take a snapshot (cost rows
only). On `stress` 2 of 200 at two seats and 6 of 200 at four change an
answer, `Triggers placed` 2.0 → 2.0 and 4.7 → 4.5. Item 167's verdict held:
reachable, wrong, rare.

**The owner's condition on the two-half `AbilityId`** (§3.6: kept if the CPU
line shows no cost of the wider key, a field on `AbilityDef` otherwise):
the provenance arm is −0.3% and +1.1% per game, inside the sitting's
spread at both seat counts, so the id keeps its two halves.

**Determinism** holds under three `MTGSIM_HASH_SEED`s, line for line outside
`=== Timing ===`: `stress`, 60 games, seed 99, at two seats and four, and on
the forced Humility and Blood Artist board at four.

**Review round 1, same PR** (the owner's review of #178). Two code changes,
each its own arm against the one before. **A candidate is one ability list**
(`42ccd0e`): a survivor's list from before is its own candidate beside its
live one, rather than rows tagged with where they came from. It reads
`IDENTICAL` to the previous head on every row, diagnostics included, on
both pools at two seats and four and on the forced Humility and Blood
Artist `stress` board at both seat counts, where snapshots are taken; CPU
+0.6% and +1.4%, inside the spread. **The snapshot reads every printed
trigger source** (`6fdca3b`), not only those whose kinds a look-back arm
reads, since that filter was a second table of `TriggerEvent::looks_back`
that TR-4's new look-back classes would have fallen out of. Gameplay is
identical everywhere, and only engine-work rows move, by one or two
`Memo hits` a game on average and one `Layer frames` on `stress` at two
seats. Determinism holds under three hasher seeds at four seats, on the
shipped `stress` pool and on the forced board.

**Re-recorded 2026-09-22 for the TR-1 review, theme C** (the matcher — F1's
CR 113.6 fix, #16's flatten, `triggers-architecture.md` §11's kind mask, #38,
and `fuzz_games --copies N`). **No pool change**, so there is no new fixture
table: `performance` stays at 94 and `stress` at 171, and every row of the
2026-09-19 tables above still reads. What this block records is a cost
reading and one row of a gate that could not be met.

**The A/B, three arms, `fuzz_ab.py` against `main` at #175's head.** Both
pools, two seats and four, 200 games, seed 12345.

| arm | vs `main`, counters |
|---|---|
| F1 + #16's flatten (`d39b9ae`) | **IDENTICAL** on both pools at two seats and four |
| plus §11's mask (shipped) | every gameplay counter identical; **six engine-work rows move, all down** |

The six are `Layer walks`, `Board walks`, `Memo hits`, `Layer frames`,
`Frames/walk` and `Dependency checks` — on `performance` at two seats
372 → 366, 251 → 245, 64,786 → 64,524, 4,861 → 4,748, 13.08 → 12.96,
31 → 30; the same shape on `stress` and at four seats.
`Triggers placed`, `Decisions`, `Priority decisions`, `Replacement gathers`,
`Restriction queries`, `Mana productions` and every game-content row are
identical everywhere. **This is the one gate row the phase could not meet and
should not have been asked to**: those six rows are `state/diagnostics.rs`'s
cost model, the mask exists to stop asking a source that cannot match, and
the two cannot both hold. The F1-only arm is what carries "no behavior
moved", and it carries it byte for byte.

**The dispatcher's own sitting** is `triggers-architecture.md` §11.1 — a
throwaway `Instant` build, before and after, on the `--copies` boards the
2026-09-20 reading used. The headline: candidate visits per game 297 → 30 on
the shipped pool and 10,131 → 707 with eight copies each of the three, with
`matches` and `Triggers placed` unchanged on every board. Whole-game CPU on
the heavy boards −11% at both seat counts; on the shipped pool −0.4% and
−2.7%, inside the sitting's spread.

**The instrument is now in the tree.** `--copies N` reproduces the retired
`--stuff N` probe's count columns exactly (243 / 297 / 2.7 at zero copies,
1,252 / 10,131 / 66.1 at eight, two seats), so the 2026-09-20 reading and
this one are one sitting rather than two builds that no longer exist. The
`Instant` half is still a throwaway; §3 refuses to store a timer and is
right to.

**Determinism** holds under three `MTGSIM_HASH_SEED`s, line for line outside
`=== Timing ===`, `stress`, 60 games, seed 99.

**Review round 1, same PR** (the owner's review of #176: Dread for the
invented "Ichorid" fixture, the candidate types renamed and folded, the dead
tag guard removed for a registry invariant, the mask selecting sources before
ordering them, the mask API renamed, and CR 113.6k asked per condition). No
pool change and nothing a pooled card can reach, so the claim is the
strongest one there is: the round's head against the previous head
(`f88d421`) is **IDENTICAL on every row, diagnostics included**, on both
pools at two seats and four and on the eight-copy board; against `main` the
same six cost rows move as above. Determinism holds under three hasher seeds
at four seats, which is the case that iterates `trigger_sources` to select
readers.

**The Commander-scale sitting** is `triggers-architecture.md` §11.2 — the
probe widened to time the replacement gather and the restriction check. At
four seats, 100 cards and 40 life the dispatcher is 2% of CPU on the shipped
pool and 7–9% with the three forced; **the replacement gather is 14–16% on
every board and passes its gate on 90–99% of calls**, which makes it the
next lever of the three sweeps. It is not this PR's.

**Re-recorded 2026-09-19 for TR-1** (the trigger spine — `triggers-architecture.md`
§12, TR-1; `codebase-state.md` "Before Triggered abilities" items 1, 3, 7, 9,
10 and 18 closed). **Pool change**: `performance` 91 → 94 (Soul Warden, Blood
Artist, Wild Growth) and `stress` 166 → 171 (plus Verdant Force and Felidar
Sovereign), so the shipped columns are a new baseline and the engine column
is the one comparable to A4c's.

**Three arms, two seat counts.** `main` (bec4c04); **engine** — this tree
with the pools exactly as `main` has them (the five cards unregistered, the
pool array at 91), which is the arm §11's prediction was written about;
**shipped** — the tree as merged. Each at `--players 2` and `--players 4`,
both pools, `--rounds 3 --games 200`.

**The probe first, as the brief asked: is the gate really empty on the old
pools?** A fourth binary — the engine arm plus a `panic!` the moment a
dispatch passes the gate and reaches the matcher — ran 200 games on each pool
at each seat count: no panic, `Triggers placed 0.0` on all four. So on the old
pools every dispatch is the five set probes plus the per-departure frame scan
(`archive/triggers-architecture-landed.md`, TR-1 note 4), and nothing else.

| | 2 seats | 4 seats |
|---|---|---|
| engine vs `main`, every counter, both pools | **IDENTICAL** | **IDENTICAL** |
| `µs / decision`, engine vs `main` — §3.1's budget | 27.5 → 27.8, **+1.0%** | 46.8 → 47.4, **+1.5%** |
| shipped vs `main`, both pools | differ | differ |
| `Triggers placed`, shipped, `performance` / `stress` | 1.5 / 1.0 | 4.2 / 3.7 |
| `Decisions`, `main` → shipped, `performance` | 223 → 235 | 440 → 469 |
| `µs / decision`, shipped vs `main` — recorded, a pool change | +6.2% | +4.6% |
| `Wins by effect`, shipped | 1 (`performance`), 1 (`stress`) | 0, 4 |
| deterministic across rounds, three hasher seeds | yes / yes / yes | yes / yes / yes |

**The engine arm is inside the budget and the counters say why.** At
identical counters — every gameplay counter and every engine-work counter
`IDENTICAL` on all four pool-and-seat cells — the cost is the gate and the
scan, +1.0% and +1.5% CPU per decision against a run-to-run spread §3.1 puts
at ~2.4%. `Layer walks`, `Board walks` and `Memo hits` did not move, which is
the cost model's claim in §11: with no trigger source no candidate is walked.

**The shipped arm is a pool change and is not budgeted; its numbers are the
new fixture.** `Triggers placed` is the phase's own row: Soul Warden, Blood
Artist and Wild Growth between them put 1.5 abilities a game on the stack at
two seats and 4.2 at four, and Wild Growth's resolutions never reach the row —
CR 605.4a keeps them off the stack. `Wins by effect` is Felidar Sovereign
(registered, not pooled, in `stress`) and Blood Artist's drain at two seats.
`Replacement prompts` on four-seat `stress` went 3.40 → 9.55 because Blood
Artist's "target player loses 1 life" and Rest in Peace share boards; a
reading, not a regression. `main`'s one `Hit turn limit` on four-seat `stress`
is `main`'s (A4c's block reads the same game).

**Reachability** (`--require`, shipped, `performance`, 200 games):

| | 2 seats | 4 seats |
|---|---|---|
| Soul Warden — cast / resolved / games / copies per deck | 216 / 216 / 135 (68%) / 1.43 | 143 / 143 / 118 (59%) / 2.83 |
| Blood Artist | 212 / 209 / 136 (68%) / 1.45 | 151 / 151 / 124 (62%) / 2.87 |
| Wild Growth | 204 / 202 / 131 (66%) / 1.42 | 126 / 124 / 105 (52%) / 2.85 |
| `Triggers placed`, with the three forced | 4.8 | 13.5 |
| board diversity | 197 of 200 (98%) | 195 of 200 (98%) |

`Triggers placed` is the placement count and Wild Growth's stackless
resolutions are not in it; per-card trigger counts are the ledger's next
instrument, not this one's.

**§3 fixture rows, shipped, 50 games / seed 12345**

| | performance | stress |
|---|---|---|
| Wins by seat | 27 (54.0%) / 22 (44.0%) | 26 (52.0%) / 24 (48.0%) |
| Wins by effect | 1 | 0 |
| Avg turns | 30.4 | 28.7 |
| Spells cast | 23.1 | 20.8 |
| Lands played | 18.1 | 17.2 |
| Combat w/ atk | 10.6 | 8.7 |
| Creatures died | 6.9 | 4.9 |
| Damage events | 22.1 | 19.7 |
| Total damage | 62.3 | 54.4 |
| Life changes | 15.9 | 16.8 |
| **Layer walks** | **351** | **428** |
| **Board walks** | **238** | **274** |
| **Memo hits** | **62,059** | **69,977** |
| **Layer frames** | **4,632** | **4,957** |
| **Frames/walk** | **13.20** | **11.59** |
| **Dependency checks** | **14** | **9** |
| **Replacement gathers** | **1107** | **1097** |
| **Restriction queries** | **1110** | **1100** |
| Mana productions | 82 | 114 |
| Prevention allocations | 0.02 | 0.02 |
| Replacement prompts | 0.22 | 1.26 |
| Max batch depth | 4 | 5 |
| Decisions | 221 | 314 |
| Priority decisions | 83 | 124 |
| Triggers placed | 1.9 | 0.7 |

The four-seat fixture rows are in the sitting's output (`--players 4`,
shipped): `Layer walks` 813 / 1,186, `Memo hits` 195,321 / 316,833,
`Decisions` 455 / 798, `Triggers placed` 4.4 / 2.8, `Turns after a departure`
21.0 / 20.7, `Departed-owned permanents` 0.0 / 0.0.

**Re-recorded 2026-09-18 for A4c** (the trace sink — `roadmap-v2.md` row
A4c; `codebase-state.md` "Before Triggered abilities" item 5 closed). **No pool
change**: `performance` 91 and `stress` 166, as A4q left them, so every column
here is comparable to the A4q block below. The record exists because the row's
check is that **no counter moves**: the sink is an observer, and an observer
that moved one would have become a participant.

**Three arms, two seat counts.** `main` (2958c1e); **off** (4a2a803) — the
sink compiled in and not attached, which is the binary every untraced game
runs; **on** — the same binary under `--trace DIR`, every game written to its
own file. Each at `--players 2` and `--players 4`, both pools, `--rounds 3
--games 200`; `plans/fuzz_ab.py --arm-args` is what the on arm needed.

| | 2 seats | 4 seats |
|---|---|---|
| off vs `main`, every counter, both pools | **IDENTICAL** | **IDENTICAL** |
| on vs `main`, every counter, both pools | **IDENTICAL** | **IDENTICAL** |
| `µs / decision`, off vs `main` — §3.1's budget | 28.9 → 29.1, **+0.8%** | 51.4 → 51.8, **+0.7%** |
| `µs / decision`, on vs `main` — recorded, not budgeted | 28.9 → 67.0, +131.5% | 51.4 → 101.0, +96.5% |
| CPU/game median, main → off → on | 6.45 → 6.50 → 14.93 ms | 22.62 → 22.78 → 44.44 ms |
| one traced `performance` game, seed 12345 | 3,091 records, 0.65 MB | 7,533 records, 1.6 MB |
| the on arm's 200 `performance` games | 206 MB | 465 MB, 1.3–4.8 MB a game |
| deterministic across rounds, three hasher seeds | yes / yes / yes | yes / yes / yes |
| one traced game under three `MTGSIM_HASH_SEED`s | byte-identical | byte-identical |

**Off is one branch per emit point, and it reads at the spread's floor.**
+0.8% and +0.7% per decision, against a run-to-run spread §3.1 puts at
~2.4% — not distinguishable from zero, and inside the 2.5-point budget either
way. That is what "every payload built behind the branch" buys: the untraced
path renders nothing, sorts nothing and takes no lock.

**On is the price of writing, and it is not budgeted.** A traced two-seat
game writes about 3,000 records and two thirds of a megabyte, a four-seat
game 7,500 and a megabyte and a half, through a `BufWriter` behind a `Mutex`.
The `pipeline` record is the largest kind — one per CR 616.1 iteration, the
event rendered as proposed and as it left — and the `event` record's text is
`format_event`'s, which resolves every name. A sink asked for one batch, or
one kind, would cost proportionally less; nothing selects yet because nothing
has asked for less.

**Every counter `IDENTICAL` on all four arms is the row's whole claim.** The
gameplay rows say the sink decided nothing; the diagnostic rows — `Layer
walks`, `Memo hits`, `Decisions` — say it *asked* nothing, which is the finer
check: a `layer_walk` record built by re-querying the memoized entry would
have moved `Memo hits`, and this one reads the frame the walk just computed.
The four-seat `stress` `Hit turn limit: 1` is the standing one A4n, A4p and
A4q's blocks carry, identical on all three arms.

**The §3 fixture rows, as shipped**: the `main` arm reproduced A4q's two
tables digit for digit at both seat counts, and the off and on arms
reproduced `main`'s. The tables are A4q's, below, and are not repeated.

**Re-recorded 2026-09-18 for A4q** (identical clauses that read no earlier
instance are checked once — `roadmap-v2.md` row A4q; A4i's review, theme I.2's
third rider, split out of A4n). **No pool change**: `performance` 91 and
`stress` 166, the cards A4i left, so every column here is comparable to the A4n
and A4p blocks below. The record exists because a counter moves — one counter,
the one the fold is about.

**Two arms, two seat counts.** `main` (bf7bfb0, post-#166) and `new` (dbc57dd),
each at `--players 2` and `--players 4`, both pools, `--rounds 3 --games 200`.

| | new vs main |
|---|---|
| every **gameplay** counter, both pools, two seats and four | **IDENTICAL** |
| `Memo hits`, `performance`, two seats / four seats | 58,274 → **58,004**, −0.46% / 183,117 → **181,724**, −0.76% |
| `Memo hits`, `stress`, two seats / four seats | 70,914 → **70,806**, −0.15% / 257,253 → **256,736**, −0.20% |
| `Layer walks`, every pool and seat count | **unchanged** — `performance` 343 and 791, `stress` 428 and 1,081 |
| `µs / decision`, `performance`, two seats / four seats | 27.2 → 26.9, −1.3% / 47.0 → 47.2, +0.3% |
| CPU/game median, two seats / four seats | 6.07 → 5.99 ms / 20.70 → 20.77 ms |
| deterministic across rounds, three hasher seeds | yes / yes |

**`IDENTICAL` on everything the game does, `Memo hits` down, was the whole
prediction and it held on all four arms.** A castability answer the fold does
not change cannot change a game, and none did: same wins, same turns, same
spells cast, same decisions, and the four-seat `stress` `Hit turn limit: 1`
that A4n's and A4p's blocks also carry. What went away is the asking.

**`Layer walks` did not move, and the row's "may move with it" was the right
worry about the wrong shape.** The concern was that a scan the fold skips is a
frame the memo never gets filled with, so some later question would walk where
it used to hit. It cannot happen for *this* fold: the clauses it collapses are
identical, so the surviving clause walks the same candidates the skipped ones
would have and fills the memo with exactly them — only the repeat hits behind
it go away. A fold across clauses with *different* filters is the one that
would trade a hit for a walk, and this is not that fold.

**Attributed to one card, forced.** With `--require "Seeds of Strength"` in
every deck — 1.44 copies, cast 188, resolved 186, in 124 of 200 games (62%),
those four numbers identical on both arms — the same `performance` gap at two
seats is **58,504 → 57,675, −1.42%**. Three times the unforced −0.46%, which
is what a per-card effect does when the card goes from a share of the draws to
a guaranteed copy in every deck.

**And it is one card in the registry, not just one in the pool.** A probe over
`default_registry` — every card built, every `AbilityDef::instances` and
`CardData::spell_instances` walked for a clause equal to an earlier one in the
reusable range — names **Seeds of Strength alone**, out of 166. The other two
registered cards with several instances do not fold and should not: Incremental
Growth's three clauses each exclude the ones before them, and Plague Spores'
two name a creature and a land. So `stress`'s narrower gap is dilution — one
card competing for 36 deck slots against 166 names rather than 91 — and not a
second shape behaving differently. Nothing is owed in `codebase-state.md` for
that reason.

**The §3 fixture rows, as shipped** (50 games / seed 12345, both pools). The
`main` arm reproduced A4p's two tables digit for digit, which is the check that
the baseline is the baseline; the tables below are those tables with `Memo hits`
moved and nothing else. At two seats 56,661 → 56,556 on `performance` (−0.19%)
and 63,025 → 62,943 on `stress` (−0.13%); at four, 178,408 → 176,892 (−0.85%)
and 269,509 → 269,118 (−0.15%).

| | performance | stress |
|---|---|---|
| Wins by seat | 16 (32.0%) / 34 (68.0%) | 27 (54.0%) / 23 (46.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 28.3 | 28.1 |
| Spells cast | 22.1 | 19.2 |
| Lands played | 16.8 | 17.0 |
| Combat w/ atk | 9.8 | 7.3 |
| Creatures died | 6.5 | 4.3 |
| Damage events | 22.3 | 15.8 |
| Total damage | 59.9 | 44.1 |
| Life changes | 14.5 | 11.7 |
| **Layer walks** | **339** | **414** |
| **Board walks** | **228** | **243** |
| **Memo hits** | **56,556** | **62,943** |
| **Layer frames** | **4,162** | **4,382** |
| **Frames/walk** | **12.29** | **10.58** |
| **Dependency checks** | **21** | **26** |
| **Replacement gathers** | **1029** | **1037** |
| **Restriction queries** | **1031** | **1039** |
| Mana productions | 82 | 104 |
| Prevention allocations | 0.02 | 0.04 |
| Replacement prompts | 0.14 | 0.56 |
| Max batch depth | 4 | 5 |
| Decisions | 225 | 279 |
| Priority decisions | 84 | 112 |

**And the four-player table:**

| | performance | stress |
|---|---|---|
| Wins by seat | 24 (48.0%) / 14 (28.0%) / 10 (20.0%) / 2 (4.0%) | 16 (32.0%) / 17 (34.0%) / 13 (26.0%) / 4 (8.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 58.6 | 65.5 |
| Spells cast | 43.3 | 43.4 |
| Lands played | 35.3 | 37.7 |
| Combat w/ atk | 24.6 | 24.0 |
| Creatures died | 14.3 | 11.5 |
| Damage events | 53.7 | 53.4 |
| Total damage | 149.0 | 158.5 |
| Life changes | 38.9 | 42.5 |
| Turns after a departure | 19.3 | 18.7 |
| Departed-owned permanents | 0.0 | 0.0 |
| **Layer walks** | **768** | **1,160** |
| **Board walks** | **496** | **640** |
| **Memo hits** | **176,892** | **269,118** |
| **Layer frames** | **14,706** | **20,463** |
| **Frames/walk** | **19.14** | **17.64** |
| **Dependency checks** | **96** | **131** |
| **Replacement gathers** | **2184** | **2625** |
| **Restriction queries** | **2189** | **2630** |
| Mana productions | 158 | 249 |
| Prevention allocations | 0.00 | 0.04 |
| Replacement prompts | 1.54 | 2.50 |
| Max batch depth | 5 | 5 |
| Decisions | 446 | 709 |
| Priority decisions | 166 | 281 |

**Re-recorded 2026-09-18 for A4n** (the instance list is computed once, in
`CardDataBuilder::build`, and stored — `roadmap-v2.md` row A4n; A4i's review,
themes H and I.2). **No pool change**: `performance` 91 and `stress` 166, the
cards A4i left, so every column here is comparable to the A4p block below —
and **both arms reproduce A4p's fixture tables to the digit**, two seats and
four, both pools, which is why no table is repeated here. The record exists
because the number the finding was measured in moved, and the timing did not.

**Two arms, two seat counts, three sittings each.** `main` (879f114, the merge
base, post-#165) and `new` (9f6eb55), each at `--players 2` and `--players 4`,
both pools, `--rounds 3 --games 200`.

| | new vs main |
|---|---|
| every counter, both pools, **two** seats | **IDENTICAL** |
| every counter, both pools, **four** seats | **IDENTICAL** |
| `µs / decision`, `performance`, two seats, three sittings | 30.0 → 30.3 (+0.9%), 29.3 → 28.6 (−2.3%), 28.4 → 27.9 (−1.7%) |
| `µs / decision`, `performance`, four seats, three sittings | 49.3 → 49.5 (+0.5%), 49.6 → 49.3 (−0.6%), 50.4 → 48.4 (−4.0%) |
| CPU/game median, first sitting, two seats / four seats | 6.69 → 6.75 ms / 21.69 → 21.79 ms |
| deterministic across rounds, three hasher seeds | yes / yes |

**`IDENTICAL` was the prediction and it held everywhere.** The stored list is
the walk's own answer — `cards::registry`'s new test compares the two for every
registered card — so no game could play differently, and none did: every
counter on every arm, and the four-seat `Hit turn limit: 1` on `stress` that
A4p's block also carried.

**The lower CPU reading the row predicted is not readable, and the row said
it would not be.** The median of three sittings is −1.7% per decision at two
seats and −0.6% at four, inside a spread the script itself puts at ~2–6%; the
sign is the predicted one in four sittings of six and the size is the noise
floor's. That is what "a cost-model finding, not a timing one" meant: ~366
allocations against a ~6,700 µs game sit below what an interleaved sitting can
resolve. A timing table cannot show this change, so the instrument that found
it is the one that records it.

**The instrument, re-read on both arms.** A counter on the walk itself —
`effect_instances` on `main`, `Effect::instances` on `new` — 50 games at seed
12345, `--threads 1`, printed per game beside `Decisions`:

| derivations of the instance list, per game | performance | stress |
|---|---|---|
| `main`, on the cast / activate / resolve paths | 365.7 | 512.7 |
| `new`, on those paths | **0** | **0** |
| `new`, inside `CardDataBuilder::build` | 659.7 | 1031.6 |

The `main` reading reproduces A4i's review to the decimal (366, against 339
layer walks and 225 decisions on the same board). The zero is structural rather
than measured: the walk has two callers left, the builder and the registry test,
and the play-path readers take `&[EffectRecipient]` off the def or the card.

**The third row is the harness, not the engine, and it is outside the timer.**
`fuzz_games::random_deck` calls `registry.create(name)` — a fresh `build()` —
for every slot of every deck and again for every registered name in each of
its two name filters, so a two-seat `performance` game constructs roughly 480
cards, each build now walking each def's effect once. `run_one_game` starts its
clock after the decks exist, so none of it reaches `CPU/game`; and the walk
allocates nothing for a def that declares no instance, which is most of them.
It is recorded because a reader of the probe would otherwise see the count go
*up*, and because a harness that builds one registry per process — the GUI, the
AI harness — pays it once.

**Re-recorded 2026-09-18 for A4p** (a seat that has left the game is not a
target — `codebase-state.md` item 160, struck; `roadmap-v2.md` row A4p).
**No pool change**: `performance` 91 and `stress` 166, the cards A4i left, so
every column here is comparable to the A4o block below. The record exists
because the fix changes what CR 601.2c enumerates for the `Player` and `Any`
filters from the first elimination of a game on — and above two seats, that is
every game.

**Four arms — two trees, two seat counts.** `main` (c0b52ca, post-#163) and
`fix` (b544a30), each at `--players 2` and `--players 4`, both pools,
`--rounds 3 --games 200`.

| | fix vs main |
|---|---|
| every counter, both pools, **two** seats | **IDENTICAL** |
| every counter, both pools, **four** seats | **differ** — attributed below |
| `µs / decision`, `performance`, two seats | 29.2 → 29.7, **+1.5%** |
| `µs / decision`, `performance`, four seats | 51.4 → 52.2, **+1.5%** |
| CPU/game median, two seats / four seats | 6.52 → 6.62 ms / 22.73 → 22.97 ms |
| deterministic across rounds, three hasher seeds | yes / yes |

**The two-seat `IDENTICAL` is the rule, not luck.** CR 104.2a ends a two-player
game the moment a player leaves, so `in_game` is false for a seat only in a
window the game does not survive: at two seats the filter has nothing to
remove. Both arms also reproduced the A4o block's recorded two-seat fixture
table to the digit, which is the check that the baseline is the baseline.
Neither µs/decision reading is a cost — both sit inside the sitting's own ~2–6%
spread, and the two seat counts moved by the same +1.5% while their CPU/game
medians moved by +1.5% and +1.1%.

**Attributed game by game rather than assumed.** Both arms dumped 200 games per
pool at four seats (`--dump-events --threads 1`), diffed per game; then a probe
build of the `fix` tree replayed all 200 games one at a time by their per-game
seeds (`game_seed = master + game_num`). The probe carries a `println!` at each
of the **four** sites whose answer this fix can change (the enumeration's offer
list, the two count arms, and `validate_any_target`), printing only when the two
answers differ and returning the fixed one either way. The probe's own
200-game dump is byte-identical to the `fix` arm's on both pools, which is what
makes it a probe of that arm rather than a fifth tree.

| | diverging games | games where an answer changed | changed answers |
|---|---|---|---|
| four seats, `performance` | 65 of 200 | 100 | 189 |
| four seats, `stress` | 68 of 200 | 113 | 343 |

**Every diverging game is a game where an answer changed**, on both pools, with
no diverging game left over. The converse does not hold and should not: a
shorter candidate list the agent would have passed on anyway leaves the stream
where it was, which is why 189 changed answers produce 65 forks. All 400 games
have a departure — a four-player game ends when three seats are gone — so the
population the fix can reach is every game, and it reaches half of them.

**Only the enumeration ever changed an answer, and that is a finding about the
pool rather than about the fix.** Every registered card with a
`Player` or `Any` instance declares `TargetCount::Exactly(1)`, so both count
arms are asked `n = 1` and answer `true` while any seat remains: the count
half of the fix is correct and unreachable at once, and what would reach it is
a card with two instances of "target player" and two seats left. The
`validate_any_target` half never fired either, because on the `fix` arm the
enumeration withholds the seat and CR 608.2b's re-check never meets a departed
one. The whole measured difference is the offer list.

**The defect was live in the measured games, and this is what it cost.** A
second probe, this one on the `main` tree — a `println!` at the `DamageDealt`
performer when the target player is not in the game, dump byte-identical to
`main`'s — names every one of them: **11 Lightning Bolts resolved against a
player who had left the game**, 10 over 10 games on `performance` and one on
`stress`, for 3, 6 and 12 damage (the pool's damage doublers are why the
numbers are not all 3). The `fix` arm manufactures **zero** on both pools,
which is the same claim `tests/phase_a4p_integration_test.rs` makes on one
board. The sibling half cost nothing observable: "target player" was offered
and then refused by `validate_player_target`, so the agent lost a priority
action to a rewind and no effect landed.

**The §3 fixture rows, as shipped** (50 games / seed 12345, both pools). The
two-seat table is unchanged from A4o's, digit for digit:

| | performance | stress |
|---|---|---|
| Wins by seat | 16 (32.0%) / 34 (68.0%) | 27 (54.0%) / 23 (46.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 28.3 | 28.1 |
| Spells cast | 22.1 | 19.2 |
| Lands played | 16.8 | 17.0 |
| Combat w/ atk | 9.8 | 7.3 |
| Creatures died | 6.5 | 4.3 |
| Damage events | 22.3 | 15.8 |
| Total damage | 59.9 | 44.1 |
| Life changes | 14.5 | 11.7 |
| **Layer walks** | **339** | **414** |
| **Board walks** | **228** | **243** |
| **Memo hits** | **56,661** | **63,025** |
| **Layer frames** | **4,162** | **4,382** |
| **Frames/walk** | **12.29** | **10.58** |
| **Dependency checks** | **21** | **26** |
| **Replacement gathers** | **1029** | **1037** |
| **Restriction queries** | **1031** | **1039** |
| Mana productions | 82 | 104 |
| Prevention allocations | 0.02 | 0.04 |
| Replacement prompts | 0.14 | 0.56 |
| Max batch depth | 4 | 5 |
| Decisions | 225 | 279 |
| Priority decisions | 84 | 112 |

**And the four-player table** (50 games / seed 12345), which is the one that
moves:

| | performance | stress |
|---|---|---|
| Wins by seat | 24 (48.0%) / 14 (28.0%) / 10 (20.0%) / 2 (4.0%) | 16 (32.0%) / 17 (34.0%) / 13 (26.0%) / 4 (8.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 58.6 | 65.5 |
| Spells cast | 43.3 | 43.4 |
| Lands played | 35.3 | 37.7 |
| Combat w/ atk | 24.6 | 24.0 |
| Creatures died | 14.3 | 11.5 |
| Damage events | 53.7 | 53.4 |
| Total damage | 149.0 | 158.5 |
| Life changes | 38.9 | 42.5 |
| Turns after a departure | 19.3 | 18.7 |
| Departed-owned permanents | 0.0 | 0.0 |
| **Layer walks** | **768** | **1,160** |
| **Board walks** | **496** | **640** |
| **Memo hits** | **178,408** | **269,509** |
| **Layer frames** | **14,706** | **20,463** |
| **Frames/walk** | **19.14** | **17.64** |
| **Dependency checks** | **96** | **131** |
| **Replacement gathers** | **2184** | **2625** |
| **Restriction queries** | **2189** | **2630** |
| Mana productions | 158 | 249 |
| Prevention allocations | 0.00 | 0.04 |
| Replacement prompts | 1.54 | 2.50 |
| Max batch depth | 5 | 5 |
| Decisions | 446 | 709 |
| Priority decisions | 166 | 281 |

Errors, panics and `Uncast resolved` are 0 on every arm and every run. The one
turn-limit hit is four-seat `stress` **game 159 (seed 12503)**, the long game
A4o's block already identified — both arms reach the cap on it, and the fix
does not touch what makes it long.

**Re-recorded 2026-09-18 for A4o** ("counter target spell" may not name an
activated ability — `codebase-state.md` item 159, struck; `roadmap-v2.md` row
A4o). **No pool change**: `performance` 91 and `stress` 166, the cards A4i left,
so every column here is comparable to the block below. The record exists anyway
because the fix moves the random agent's stream — it changes what
`castable_spells` offers and what CR 601.2c enumerates whenever an activated
ability is on the stack, and both halves of that are in the pool.

**Four arms — two trees, two seat counts.** `main` (8e5a8d8, post-#162) and
`fix` (2613f77), each at `--players 2` and `--players 4`, both pools,
`--rounds 3 --games 200`.

| | fix vs main |
|---|---|
| every counter, both pools, **two** seats | **differ** — predicted, and attributed below |
| every counter, both pools, **four** seats | **differ** — same |
| `µs / decision`, `performance`, two seats | 28.5 → 28.7, **+0.5%** |
| `µs / decision`, `performance`, four seats | 51.5 → 49.3, **−4.2%** |
| CPU/game median, two seats / four seats | 6.36 → 6.39 ms / 22.81 → 21.80 ms |
| deterministic across rounds, three hasher seeds | yes / yes |

**`differ` is the finding, not a failure**, and it is the one case §3.1's
`IDENTICAL` prediction does not apply to: the fix removes a *candidate action*
from a priority window, so the agent's `random_range(0..n)` draws against a
shorter list and every game that reaches such a window forks. Both µs/decision
readings sit inside the sitting's own ~2–6% spread and neither is a cost; the
four-seat −4.2% is spread, not a speedup.

**Attributed game by game rather than assumed.** Both arms dumped 200 games per
pool per seat count (`--dump-events --threads 1`), diffed per game; then a probe
build of the `fix` tree — a `println!` at each of the two changed answers, the
count arm and the enumeration arm, returning the fixed value either way so the
probe plays the fix arm's line — replayed all 200 games one at a time by their
per-game seeds.

| | diverging games | games where a `Spell` answer changed | phantom cards `main` made |
|---|---|---|---|
| two seats, `performance` | 11 of 200 | 22 | 10, over 9 games |
| two seats, `stress` | 10 of 200 | 20 | 6, over 6 games |
| four seats, `performance` | 14 of 200 | 58 | 3, over 3 games |
| four seats, `stress` | 21 of 200 | 53 | 7, over 7 games |

**Every diverging game is a game where the filter's answer changed**, in all
four arms, with no diverging game left over. The converse does not hold and
should not: a shorter option list the agent would have passed on anyway leaves
the stream where it was, which is why 22 changed answers produce 11 forks.
**Both** changed answers had to be probed to get the containment — the count
arm alone leaves four-seat `stress` game 124 unexplained, because with a real
spell on the stack beside the ability the count is ≥ 1 either way and only the
announcement's candidate list shrinks.

**The defect was live in the measured games, and this is what it cost.** In
`main`, 26 stack objects across the four arms were countered without ever having
been cast — an activated ability's ephemeral object moved to a graveyard by
`Primitive::CounterSpell`, landing there as a second copy of its source's card
while the source stood on the battlefield. Chainbreaker, Bonesplitter, Merfolk
Thaumaturgist, Samite Healer, Mind Stone, Words of Worship, Deep Water,
Circle of Protection: Red, Aggravated Assault. The `fix` arm manufactures **zero**
in all four arms, which is the same claim `tests/phase_a4o_integration_test.rs`
makes on one board.

**The §3 fixture rows, as shipped** (50 games / seed 12345, both pools):

| | performance | stress |
|---|---|---|
| Wins by seat | 16 (32.0%) / 34 (68.0%) | 27 (54.0%) / 23 (46.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 28.3 | 28.1 |
| Spells cast | 22.1 | 19.2 |
| Lands played | 16.8 | 17.0 |
| Combat w/ atk | 9.8 | 7.3 |
| Creatures died | 6.5 | 4.3 |
| Damage events | 22.3 | 15.8 |
| Total damage | 59.9 | 44.1 |
| Life changes | 14.5 | 11.7 |
| **Layer walks** | **339** | **414** |
| **Board walks** | **228** | **243** |
| **Memo hits** | **56,661** | **63,025** |
| **Layer frames** | **4,162** | **4,382** |
| **Frames/walk** | **12.29** | **10.58** |
| **Dependency checks** | **21** | **26** |
| **Replacement gathers** | **1029** | **1037** |
| **Restriction queries** | **1031** | **1039** |
| Mana productions | 82 | 104 |
| Prevention allocations | 0.02 | 0.04 |
| Replacement prompts | 0.14 | 0.56 |
| Max batch depth | 4 | 5 |
| Decisions | 225 | 279 |
| Priority decisions | 84 | 112 |

**And the four-player table** (50 games / seed 12345):

| | performance | stress |
|---|---|---|
| Wins by seat | 23 (46.0%) / 16 (32.0%) / 9 (18.0%) / 2 (4.0%) | 14 (28.0%) / 18 (36.0%) / 12 (24.0%) / 6 (12.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 59.6 | 65.1 |
| Spells cast | 43.7 | 43.4 |
| Lands played | 35.5 | 37.7 |
| Combat w/ atk | 25.2 | 23.8 |
| Creatures died | 14.4 | 11.6 |
| Damage events | 55.0 | 54.3 |
| Total damage | 154.2 | 159.5 |
| Life changes | 40.0 | 42.8 |
| Turns after a departure | 20.2 | 18.4 |
| Departed-owned permanents | 0.0 | 0.0 |
| **Layer walks** | **780** | **1,152** |
| **Board walks** | **505** | **632** |
| **Memo hits** | **183,848** | **266,714** |
| **Layer frames** | **15,168** | **20,383** |
| **Frames/walk** | **19.45** | **17.69** |
| **Dependency checks** | **115** | **132** |
| **Replacement gathers** | **2227** | **2601** |
| **Restriction queries** | **2231** | **2606** |
| Mana productions | 160 | 247 |
| Prevention allocations | 0.00 | 0.02 |
| Replacement prompts | 1.52 | 2.52 |
| Max batch depth | 5 | 5 |
| Decisions | 456 | 694 |
| Priority decisions | 169 | 273 |

Errors, panics and turn-limit hits are 0 on every arm and every run but one:
four-seat `stress` **game 159 (seed 12503)**, which both arms hit at the cap and
which is the long game RF's block already identified — 246 turns at
`--max-turns 600`, not a loop.

**Re-recorded 2026-09-17 for A4i** (CR 601.2c's instances of "target" —
`backlog.md` §2.20, graduated; `codebase-state.md` items 152–156).
**Pool change**: `performance` 90 → **91** (Seeds of Strength) and `stress`
161 → **166**, so the `card` column below is a new baseline and is compared to
nothing. The `rule` column is not: it is the engine change with no card
registered, at 90 and 161, which is what makes an engine reading possible
across a pool change at all (§3.1).

**Three arms.** `main` (aafb79a); **rule** (72617d5 + ee67476) — the instance
model and the six migrated effects, no card registered, both pools the size
`main` has them; **card** (3ce6186) — the five cards, Seeds of Strength pooled.

| | rule vs main |
|---|---|
| every counter, `performance`, 200 games | **IDENTICAL** |
| every counter, `stress`, 200 games | **differ** — and that is the finding below |
| `µs / decision`, three sittings | 30.3 (+2.4%), 29.7 (+0.6%), 30.3 (+0.4%) |
| CPU/game median, first sitting | 6.96 → 7.13 ms |
| deterministic across rounds, three hasher seeds | yes / yes |

`IDENTICAL` on `performance` was the prediction and it held: no card in the
pool announced a second instance, so the loop is entered with n = 1 everywhere
and n = 1 is the straight line it replaced. The median over three sittings is
**+0.6% µs/decision**, inside §3.1's 2.5-point budget, against a run-to-run
spread §3.1 itself puts at ~2.4% — which is what the first sitting's +2.4% is.

**What the A/B found, and it is not a cost.** `stress` differs on wins, turns,
spells and decisions with the *same 161-card pool on both arms*, so a
registered-but-unpooled card was playing differently. It is **Skullcrack**:
three atoms, the first two `EffectRecipient::Controller`, and the pre-A4i rule
took a `Sequence`'s first atom's recipient as the whole spell's — so the cast
announced no target and its three damage went nowhere. Registered since RE-3
and never caught, because RE-3's tests stage a `ResolutionContext` with the
target written in by hand. `codebase-state.md` item 152; shown to fail against
`main` before the fix, and the regression casts from hand.

**And a cost the first sitting found, fixed before the second.**
`every_instance_has_a_choice` fed each instance's legal choices forward so the
next clause could exclude them, for every spell rather than only the "another
target" family — enumerating every candidate where `has_legal_choices` would
have stopped at the first. The arms played identically game for game and
**memo hits moved 63,427 → 65,649, +3.5% per game**: no game reached a
different board, the engine just asked more questions to get to the same one.
That is the shape §3.1's `IDENTICAL` prediction exists to catch, and it is why
a difference is a finding rather than a tolerance.

The `card` arm's re-record, `performance`, 200 games at seed 12345, pool 91:
CPU/game median **6.25 ms**, **28.0 µs/decision**, 343 layer walks, 57,816
memo hits, 223 decisions, 28.7 average turns. Reachability, `--require`:
Seeds of Strength **cast 190, resolved 188, in 124 of 200 games (62%)**, and
the two that did not resolve are CR 608.2b. `stress`, 166 cards: 0 errors,
0 panics, 0 turn-limit hits.

**The review's round, 2026-09-17** (themes A, B, C, F of A4i's review; the
handoff was evicted with A4n and `codebase-state.md`'s A4i section is the index). **No pool change** — `performance` 91 and
`stress` 166, as the block above left them, so these columns are comparable to
it. Two arms: **preC** (c731e55, A4i as reviewed) and **postC** (3ea4cb5, the
flat `ChosenTargets`, the one-pass `surviving_targets`, and the bounded
castability enumeration). Theme A's renames sit between them and are
behaviour-free.

| | postC vs preC |
|---|---|
| every counter, `performance`, 200 games | **IDENTICAL** |
| every gameplay counter, `stress`, 200 games | **IDENTICAL** |
| `Memo hits`, `stress` | **71,580 → 71,469, −0.16%** |
| `µs / decision`, `performance` | 29.3 → 29.0, **−1.1%** |
| CPU/game median, `performance` | 6.54 → 6.47 ms |
| deterministic across rounds, three hasher seeds | yes / yes |

**The only counter that moved is the one the change was about, and it moved
down.** Same games, same turns, same decisions, fewer memoized layer queries:
the castability check's feed-forward used to enumerate every legal candidate and
keep the first `n`, and now stops at `n`. It runs only for a clause that a later
clause excludes — the "another target" family — which today is Incremental
Growth alone, registered and unpooled, hence `performance` `IDENTICAL` and
`stress` down a little.

**Attributed rather than assumed.** With `--require "Incremental Growth"`
putting the card in every deck, the same gap is **71,184 → 70,593, −0.83%** —
five times wider, which is what a per-card effect does when the card goes from
a third of games to all of them.

It is the mirror image of the +3.5% the first sitting found: that was the
feed-forward running where nothing read it, this is the same loop no longer
reading more than it needs.

**Re-recorded 2026-09-16 for A4h** (the retry re-ask's stale list —
`codebase-state.md` item 139, closed, and item 150 beside it; item 41's fork
test). **No pool change**: `performance` 90 and `stress` 161, as A4g left
them, so every column below is comparable to A4g's. Recorded because the
stream moves in every game after its first rejected action.

**Four arms, and the third is the one the budget is read on.** `main`
(9c3c8e6); **inert** (43ae182) — `#[derive(Clone)]` on the random provider and
`Game::resume_turn_at_priority`, no engine change; **cost** — a throwaway
build of the inert arm that does A4h's extra work on every re-ask and throws
the answer away, so it plays `main`'s games while paying A4h's bill; **new**
(HEAD) — both fixes and the test.

| | 2 seats | 4 seats |
|---|---|---|
| inert vs `main`, every counter, both pools | **IDENTICAL** | **IDENTICAL** |
| cost vs `main`, every gameplay counter, both pools | **IDENTICAL**; the layer diagnostics move, because the probe calls the engine (`Layer walks` 352 → 364 at 50 games) | **IDENTICAL**, same exception |
| new vs `main`, both pools | differ, by construction | differ, by construction |
| **`µs / decision`, cost vs inert — the overhead** | **+2.8%**, **+0.7%** | **+3.9%**, **+2.8%**, **+2.1%** |
| `µs / decision`, new vs `main` | 28.7 → 30.3, **+5.6%** | 48.9 → 51.2, **+4.7%** |
| `ms / 1,000 walks`, cost | −1.3% vs `main`, −2.7% vs inert | −0.7% vs `main`, −0.5% / −1.2% vs inert |
| CPU/game median, main → inert → cost → new | 6.78 → 6.75 → 6.93 → 7.13 ms | 22.31 → 22.02 → 22.89 → 23.42 ms |
| `Decisions` a game, `performance` | 236 → 235 | 456 → 457 |
| `Priority decisions` a game, `performance` | 94 → **89** | 182 → **172** |
| deterministic across rounds, three hasher seeds | yes / yes / yes / yes | yes / yes / yes / yes |

**The overhead is one extra enumeration per re-ask, and the cost arm is what
isolates it.** §3.1's budget is 2.5 points of CPU per decision *at identical
counters*, and a stream-moving PR has none — so an arm that does A4h's work
without A4h's effect is the only honest read. Five sittings, three rounds or
five: **+0.7% and +2.8% at two seats, +2.1%, +2.8% and +3.9% at four**. That
is the budget's line, over it as often as under, against a run-to-run spread
§3.1 itself puts at ~2.4% — so this block says why rather than claiming the
gate, and the reviewer decides.

**Why it is worth paying.** What it buys is item 40's invariant at the
priority boundary: the prompt is a function of `GameState`, so a clone taken
at one can be resumed, which is the property the AI track is built on and the
one `tests/priority_fork_test.rs` now asserts every run. Where it goes is the
re-ask path only, which exists because the candidate list is an
overapproximation by contract
(`dp-middleware-and-candidate-enumeration.md` §2). What removes it is Phase
10's exact action space — item 140's third option — which retires the re-asks
and this cost together.

`ms / 1,000 walks` moves the *other* way in all five (−0.5% to −2.7%): the
cost is more layer walks, not slower ones, because
`candidate_priority_actions` reads every permanent's effective abilities. A
calibration the cost arm gave for free — an earlier build enumerating on
*every* iteration rather than only on a re-ask read **+21.3%**, which is what
one whole extra `candidate_priority_actions` per priority window costs. A4h
pays about a tenth of that. And ordering inside item 150's check is worth
1.5 points on its own: with `has_any_legal_choice` ahead of the cost check
the shipped arm read +7.2% at two seats, and +5.6% behind it.

**The rest of the shipped arm's delta is a different game, not overhead.**
At two seats on `performance` the new arm casts 23.0 spells a game against
22.3, plays 18.3 lands against 17.9 and runs 30.8 turns against 30.4 — a
busier board, walked more (`Layer frames` 4,496 → 4,726). It is also a
*better-behaved* one: `Priority decisions` — the prompts offering more than
`Pass` — fall 94 → 89 at two seats and 182 → 172 at four, which is item 150's
check taking the equip abilities no creature could receive off the list.

**No callgrind row.** The stream moved, so an instruction count would be a
new baseline rather than a comparison; §12's reading stays A4g's.

**Which games diverge, and why the event dump cannot attribute them.** 40
games, two seats, `performance`, `--dump-events` both arms, split per game:
**36 of 40 differ** — 35 of them already with item 139's fix alone. The first
difference is never *at* a rejection in the dump, and it cannot be: a cast
that `castable_spells` offered and 601.2g could not pay taps nothing and
emits no event, so the re-ask the fix changed is invisible and the divergence
first surfaces at the next decision that happens to differ. The instrument
that *can* see it is the fork test, which compares the offered lists at the
prompt — all 119 of its pre-fix failures name a list the branch could not
rebuild, none a silent divergence after one.

**One fresh turn-limit hit, and it is a long game.** Four seats, `stress`:
`Hit turn limit` 1 → 2. `main`'s is game 159 (seed 12503); the new one is game
111 (seed 12455), and at `--max-turns 600` it finishes — P1 wins in 242
turns, against 246 for the pre-existing one. Neither is a loop.

**The §3 fixture rows (50 games, two seats), new arm.**

| | performance | stress |
|---|---|---|
| Wins by seat | 24 (48.0%) / 26 (52.0%) | 21 (42.0%) / 29 (58.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 27.6 | 32.2 |
| Spells cast | 21.9 | 22.4 |
| Lands played | 17.1 | 18.1 |
| Combat w/ atk | 10.0 | 10.9 |
| Creatures died | 6.1 | 4.8 |
| Damage events | 20.6 | 25.3 |
| Total damage | 61.1 | 59.1 |
| Life changes | 14.4 | 19.9 |
| **Layer walks** | **349** | **490** |
| **Board walks** | **229** | **331** |
| **Memo hits** | **54,467** | **133,051** |
| **Layer frames** | **4,133** | **8,338** |
| **Frames/walk** | **11.83** | **17.01** |
| **Dependency checks** | **10** | **4** |
| **Replacement gathers** | **1010** | **1417** |
| **Restriction queries** | **1012** | **1419** |
| Mana productions | 82 | 142 |
| Prevention allocations | 0.00 | 0.00 |
| Replacement prompts | 0.30 | 1.28 |
| Max batch depth | 5 | 5 |
| Decisions | 221 | 415 |
| Priority decisions | 82 | 176 |

**Re-recorded 2026-09-16 for A4g** (process-stable ids —
`codebase-state.md` item 144, closed; `layers-architecture.md` §12, the
second 2026-09-16 re-read). **No pool change**: `performance` 90 and `stress`
161, as RF left them, so every column below is comparable to RF's. Recorded
because a counter moved, and the block says which, where, and why.

**Three arms, one per commit.** `main` (a41fcc2); **swap** (a8931ea) — the
type swap alone, `ObjectId` from the state's counter and `AbilityId` derived
from the card, every id-keyed map still on `RandomState`; **hash** (87bee60)
— `types::ids::IdHash` on all 27 id-keyed declarations, seeded per process,
with `fuzz_ab.py` giving each timing round its own seed from this PR on.

| | 2 seats | 4 seats |
|---|---|---|
| hash vs swap, every counter, both pools | **IDENTICAL** | **IDENTICAL** |
| swap vs main, `performance` | **IDENTICAL** at 200 games and in the 50-game §3 run | differ: **5 games of 200** (10, 34, 116, 123, 179); nine rows move by a digit — `Memo hits` 179,038 → 179,113, `Layer frames` 15,314 → 15,316, `Dependency checks` 117 → 118, gathers 2227 → 2228, restriction queries 2231 → 2232, and four gameplay averages by 0.1 |
| swap vs main, `stress` | differ: **1 game of 200** (36); `Memo hits` 80,380 → 80,330, `Layer frames` 5,772 → 5,769, `Frames/walk` 12.50 → 12.49 | differ: **2 games of 200** (36, 179); `Memo hits` 273,206 → 273,265, `Layer frames` 21,435 → 21,437, `Dependency checks` 171 → 170, gathers 2597 → 2598, restriction queries 2602 → 2604, total damage 149.4 → 149.5 |
| `µs / decision`, main → swap → hash | 43.9 → 41.1 → **29.0** (−6.6% / **−34.0%**) | 76.9 → 68.9 → **48.2** (−10.4% / **−37.3%**) |
| CPU/game median, main → swap → hash | 10.37 → 9.69 → 6.84 ms | 35.07 → 31.44 → 22.00 ms |
| CPU/game p99 median | 36.34 → 33.97 → 22.98 ms | 87.22 → 81.27 → 57.36 ms |
| deterministic across rounds, three hasher seeds | yes / yes / yes | yes / yes / yes |

**The moved games are one board, and it is not an order leak.** The hasher
arm reads identical to the swap arm on every counter at every seat count on
both pools, with a different `MTGSIM_HASH_SEED` per timing round — so no
sweep leaks iteration order, which was the difference the two-arm design was
built to catch. What moves is the swap itself, and it is the decision item
144 made: a printed `AbilityId` is derived from the card name and the def's
ordinal, so two copies of one card share their ids where two runs of the
factory used to mint two sets. On every one of the six diverging games the
acting player controls **two Citanul Hierophants** at the divergence ("creatures
you control have '{T}: Add {G}'"): each creature carries two grants of one
ability under one id, `enumerate_activatable_mana_abilities` dedupes by
`(ObjectId, AbilityId)` and offers one candidate where `main` offered two, and
the random agent's uniform pick over a shorter list lands on a different
option — a different color off an Everywhere, or a Forest for an Everywhere.
Attributed, not guessed: a probe build that appended a per-build nonce to the
card name (restoring per-copy uniqueness and nothing else) read `main` to the
digit on both boards. CR 113.10b calls the two grants two instances of one
ability and `LoseAbility` already removes both (phase LF's test), so the
shorter list is the reading the ids now carry; activating either instance
taps the same creature for the same mana. `codebase-state.md` item 149 holds
what it leaves open (per-instance counting, CR 603.7h) — a note for the
triggers doc, nothing owed here.

**The §3 fixture rows (50 games, two seats).** `performance` is byte-identical
to RF's run. `stress` moves because game 36 is inside the first fifty:
`Layer walks` 487 → 486, `Memo hits` 91,715 → 91,515, `Layer frames`
6,542 → 6,529, `Frames/walk` 13.44 → 13.43, `Dependency checks` 11 → 10,
gathers 1237 → 1236, restriction queries 1239 → 1238, mana productions
134 → 133, `Decisions` 386 → 385, priority decisions 165 → 164, total damage
49.7 → 49.6; every other row as RF recorded it.

**Where the milliseconds went, and the ratchet.** The swap alone is the
16-byte key becoming 8 bytes under the same SipHash: −6.6% per decision at two
seats, −10.4% at four. The hasher is the rest: −34.0% and −37.3%, the largest
per-decision drop the record holds after A4f's, on a board whose decisions
per game are unchanged to the digit on `performance` at two seats. Both arms
are inside §3.1's 2.5 points by a wide margin, in the right direction. The
instruction reading that travels is §12's.

**Re-recorded 2026-09-16 for RF** (the gather's zone leg —
`replacement-architecture.md` §9, Phase RF; §3.3's source 2). `PERFORMANCE_POOL`
+1 — Darksteel Colossus, 89 → 90 — and the stress pool +2 (159 → 161: Nexus of
Fate and the pooled one), so **both tables are a re-record and neither column
is comparable to A4e's**.

**Three arms, and the middle one is the engine's reading.** `main` (1fe9a14),
the leg with both cards registered and unpooled, and the pool entry on top —
**as measured at the review commit (ca88adf)**, which added the printed-def
precheck; the landing's own sitting is kept below it, because the difference
between the two is the review's finding.

| | 2 seats | 4 seats |
|---|---|---|
| engine vs `main`, `performance`, every counter | **IDENTICAL** | **one row**: `Memo hits` 189,560 → 189,566, every other row identical — attributed below |
| engine vs `main`, `stress` | differ, by construction (registration moves the decks) | differ, by construction |
| pooled vs `main`, both pools | differ, by construction | differ, by construction |
| `µs / decision`, engine vs `main` | 68.7 → 66.7, **−2.9%** | 121.1 → 119.2, **−1.6%** |
| `µs / decision`, pooled vs `main` | 68.7 → 68.6, −0.1% | 121.1 → 114.9, −5.1% |
| `Layer walks`, `performance`, main → engine → pooled | 367 → 367 → 363 | 809 → 809 → 787 |
| `Frames/walk`, `performance` | 12.61 → 12.61 → 12.38 | 20.29 → 20.29 → 19.46 |
| `Replacement gathers`, `performance` | 1081 → 1081 → 1097 | 2270 → 2270 → 2227 |

**The engine half is inside §3.1's 2.5 points at both seat counts.** A board
that plays no zone-functioning card has an empty candidate map, and the leg
costs it one `is_empty` per gather. **The pooled half now costs nothing the
walk row can see**: the pooled arm walks *fewer* frames than `main` because
its games are different games, not because a Colossus is cheaper than
nothing. The cost scales with the number of such cards in play, not with the
size of the libraries, which is what a candidate map buys over a zone walk.

**The six memo hits are attributed, not guessed.** A fourth arm — the review
commit with one change reverted, `puts_a_replacement_ability` matching only a
bare `Effect::Replacement` body as the two old bools did — is byte-identical
to `main` at four seats on every line outside `=== Timing ===`, and the engine
arm differs from it on that one line alone. So the six are the wrapper peel:
a **copied** Laboratory Maniac ("if you would draw a card while your library
has no cards in it", a `Conditional` body, pooled beside Cytoshape) now
lights the gate its wrapper had hidden it from, and the battlefield sweep
walks that board's permanents — six memo hits, no new frame — in the few
games where the copy exists. That was a silent gap: the copied ability was
never gathered (`cost-architecture.md` §8 item 1 named the shape for the cost
gate). No draw from an empty library met it in 200 games, so no other row
moved.

**At landing (2026-09-16, before the review), the same three arms read**:
engine vs `main` IDENTICAL at both seat counts, +0.6% / −1.2% per decision;
pooled `Layer walks` 367 → **521** at two seats and 809 → **1,365** at four,
`Frames/walk` 12.61 → 8.93 and 20.29 → 11.65, per decision +3.9% / −0.3%. One
non-member frame per gather per Colossus in a hand or a library, on every
event — the shape decision 1 predicted and the review declined to accept;
decision 7 (the printed-def precheck) is what took the rows back to `main`'s.

**Reachability** (`--require "Darksteel Colossus,Nexus of Fate"` on the pooled
binary, `--pool stress` because Nexus is unpooled, 200 games, seed 12345,
one copy of each forced into every deck):

| | 2 seats | 4 seats |
|---|---|---|
| Darksteel Colossus — cast / resolved / games with a cast / copies per deck | 45 / 45 / 38 (19%) / 1.25 | 68 / 68 / 60 (30%) / 2.46 |
| Nexus of Fate — cast / resolved / games with a cast / copies per deck | 138 / 137 / 89 (44%) / 1.28 | 127 / 127 / 88 (44%) / 2.51 |

One of the two-seat Nexuses was countered, which is the stack-side
replacement reached in a random game; every resolved Nexus is CR 608.2n's
move replaced. Errors, panics and turn-limit hits are 0 on every arm and
every run but the one four-seat `stress` game A4e already recorded at the
cap.

**The §3 fixture rows, as shipped** (50 games / seed 12345, both pools):

| | performance | stress |
|---|---|---|
| Wins by seat | 20 (40.0%) / 30 (60.0%) | 27 (54.0%) / 23 (46.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 29.3 | 30.8 |
| Spells cast | 22.2 | 22.5 |
| Lands played | 17.7 | 18.0 |
| Combat w/ atk | 10.0 | 9.0 |
| Creatures died | 6.1 | 4.7 |
| Damage events | 21.6 | 19.8 |
| Total damage | 67.6 | 49.7 |
| Life changes | 13.4 | 15.9 |
| **Layer walks** | **352** | **487** |
| **Board walks** | **237** | **316** |
| **Memo hits** | **58,513** | **91,715** |
| **Layer frames** | **4,203** | **6,542** |
| **Frames/walk** | **11.94** | **13.44** |
| **Dependency checks** | **6** | **11** |
| **Replacement gathers** | **1063** | **1237** |
| **Restriction queries** | **1066** | **1239** |
| Mana productions | 83 | 134 |
| Prevention allocations | 0.00 | 0.00 |
| Replacement prompts | 0.34 | 1.16 |
| Max batch depth | 5 | 5 |
| Decisions | 233 | 386 |
| Priority decisions | 91 | 165 |

**And the four-player table** (50 games / seed 12345):

| | performance | stress |
|---|---|---|
| Wins by seat | 22 (44.0%) / 15 (30.0%) / 11 (22.0%) / 2 (4.0%) | 18 (36.0%) / 20 (40.0%) / 9 (18.0%) / 3 (6.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 57.4 | 66.6 |
| Spells cast | 42.4 | 47.3 |
| Lands played | 34.5 | 38.6 |
| Combat w/ atk | 23.9 | 25.5 |
| Creatures died | 13.8 | 11.9 |
| Damage events | 51.2 | 58.3 |
| Total damage | 148.5 | 149.7 |
| Life changes | 37.4 | 42.0 |
| Turns after a departure | 18.6 | 21.2 |
| Departed-owned permanents | 0.0 | 0.0 |
| **Layer walks** | **773** | **1,230** |
| **Board walks** | **496** | **705** |
| **Memo hits** | **169,819** | **292,195** |
| **Layer frames** | **14,505** | **23,859** |
| **Frames/walk** | **18.77** | **19.39** |
| **Dependency checks** | **144** | **96** |
| **Replacement gathers** | **2150** | **2775** |
| **Restriction queries** | **2155** | **2780** |
| Mana productions | 151 | 279 |
| Prevention allocations | 0.00 | 0.06 |
| Replacement prompts | 1.22 | 3.40 |
| Max batch depth | 5 | 5 |
| Decisions | 437 | 821 |
| Priority decisions | 175 | 341 |

**Re-recorded 2026-09-16 for A4e** (item 138's two decision counters and item
145's forced-prompt guard, PR #155). **The pool did not change** — 89 and 159,
the same cards — and this is a re-record anyway, which is the unusual part and
the reason it is here: the guard stops the engine asking a question with one
legal answer, a skipped prompt is a skipped RNG draw for
`RandomDecisionProvider::allocate`, and every seed-deterministic row moves
because the games that follow are different games.

**Three arms, and the middle one is the proof.** `main` (be65deb), the
counters-only commit, and the guard on top:

| | 2 seats | 4 seats |
|---|---|---|
| counters vs `main`, both pools, every row | **IDENTICAL** | **IDENTICAL** |
| guard vs `main`, both pools | differ, by construction | differ, by construction |
| `Decisions`, `performance` | ? → 266 → 236 | ? → 483 → 464 |
| `Priority decisions`, `performance` | ? → 103 → 96 | ? → 188 → 188 |
| `Layer walks`, `performance` | 394 → 394 → 367 | 793 → 793 → 809 |
| `Avg turns`, `performance` | 32.1 → 32.1 → 30.0 | 61.0 → 61.0 → 60.9 |

The walk row moves *up* at four seats and down at two, which is what a moved
stream looks like: nothing here measures a cost change, and the guard's own
saving is 30 provider round trips a game — about 15 µs in process, and out of
process the difference between shipping 2,544 prompts and 2,514.

**One four-seat `stress` game reaches the 200-turn cap**, where none did on
`main`. At `--max-turns 600` that run's longest game is 234 turns and the cap
is never reached, so it is a long game rather than a hang — RE-9 recorded the
same shape for its pooled arm.

**The six-board decision reading** this record's instrument exists for is
`codebase-state.md` item 138's table, not repeated here; the headline is
**8,790 decisions per core-second** on the 60-card `performance` board at four
seats and **6,420** at Commander scale (`--deck-size 100 --life 40 --players
4`, new flags in this PR).

**The §3 fixture rows, as shipped** (50 games / seed 12345, both pools):

| | performance | stress |
|---|---|---|
| Wins by seat | 29 (58.0%) / 21 (42.0%) | 24 (48.0%) / 25 (50.0%) |
| Wins by effect | 0 | 1 |
| Avg turns | 29.9 | 30.5 |
| Spells cast | 23.4 | 23.3 |
| Lands played | 18.1 | 17.7 |
| Combat w/ atk | 10.5 | 10.1 |
| Creatures died | 6.9 | 5.2 |
| Damage events | 21.7 | 23.2 |
| Total damage | 60.7 | 54.0 |
| Life changes | 14.3 | 15.6 |
| **Layer walks** | **367** | **451** |
| **Board walks** | **248** | **289** |
| **Memo hits** | **59,089** | **79,362** |
| **Layer frames** | **4,448** | **5,658** |
| **Frames/walk** | **12.12** | **12.55** |
| **Dependency checks** | **21** | **25** |
| **Replacement gathers** | **1071** | **1187** |
| **Restriction queries** | **1073** | **1190** |
| Mana productions | 80 | 120 |
| Prevention allocations | 0.00 | 0.08 |
| Replacement prompts | 0.12 | 1.18 |
| Max batch depth | 5 | 5 |
| Decisions | 238 | 333 |
| Priority decisions | 96 | 138 |

**And the four-player table** (50 games / seed 12345), which RE-7 established
is diffed against its own predecessor and never against a two-player arm:

| | performance | stress |
|---|---|---|
| Wins by seat | 17 (34.0%) / 22 (44.0%) / 8 (16.0%) / 3 (6.0%) | 18 (36.0%) / 22 (44.0%) / 8 (16.0%) / 2 (4.0%) |
| Avg turns | 59.9 | 67.7 |
| Spells cast | 45.2 | 48.0 |
| Lands played | 36.1 | 38.9 |
| Combat w/ atk | 25.5 | 26.0 |
| Creatures died | 14.9 | 13.5 |
| Damage events | 54.4 | 59.9 |
| Total damage | 159.5 | 151.5 |
| Life changes | 37.1 | 41.9 |
| Turns after a departure | 22.2 | 19.9 |
| Departed-owned permanents | 0.0 | 0.0 |
| **Layer walks** | **814** | **1,275** |
| **Board walks** | **529** | **702** |
| **Memo hits** | **188,573** | **287,180** |
| **Layer frames** | **16,046** | **23,833** |
| **Frames/walk** | **19.71** | **18.70** |
| **Dependency checks** | **116** | **122** |
| **Replacement gathers** | **2273** | **2763** |
| **Restriction queries** | **2278** | **2770** |
| Mana productions | 156 | 262 |
| Prevention allocations | 0.00 | 0.08 |
| Replacement prompts | 2.52 | 3.62 |
| Max batch depth | 5 | 6 |
| Decisions | 467 | 793 |
| Priority decisions | 186 | 331 |

**Re-recorded 2026-09-15 for RE-9** (CR 106.6a's mana production event and
CR 106.12's "tapped for mana"; `replacement-architecture.md` §9, the last of
RE's ten PRs). `PERFORMANCE_POOL` +1 — Mana Reflection, 88 → 89 — and the
stress pool +5 (154 → 159: Nyxbloom Ancient, Deep Water, Pale Moon, Doubling
Cube and the pooled one), so **both tables are a re-record and neither column
is an engine reading.** The A/B sitting ran at 157; Pale Moon and Doubling
Cube were registered at the PR's review, unpooled, so the `performance`
columns below are the sitting's to the byte and the `stress` columns were
re-recorded at 159 afterwards.

**The review's two probe arms, and the number the phase was actually
worried about.** The sitting's engine arm measures a proposal with nothing
watching it; a third arm held the game fixed and made a mana replacement
*apply* on every tap — one `Indefinite` no-op `Multiplier(1)` row per player
from a source in exile — and a fourth gated `gather` for `ProduceMana` the
way §8's event-kind bitmask would. Against the engine arm at `--rounds 7`:

| | 2 seats | 4 seats |
|---|---|---|
| a mana replacement applying on every tap, CPU/game | 13.95 → 14.32 ms (**+2.7%**) | 47.57 → 48.61 ms (**+2.2%**) |
| the event-kind gate for `ProduceMana`, CPU/game | 13.95 → 13.97 (+0.1%) | 47.57 → 47.29 (−0.6%) |
| Replacement gathers, applying arm | 1063 → 1144 | 2,163 → 2,312 |
| every gameplay and layer row | identical | identical |

The applying arm's rounds sit clear of the engine arm's; the gated arm's
straddle them. So a board with a mana doubler pays about 2.5 points for the
pipeline's work per tap, and the gate returns nothing for this event kind —
the +1.2% a proposal costs is the chokepoint's fixed per-event work, not the
sweep (`codebase-state.md` item 136 names the lever that would touch it). The
engine reading is the third arm, this branch with the three cards
unregistered, whose counters are `main`'s on every gameplay and layer row at
both seat counts by construction; what it adds is one row and one movement:
`Mana productions`, **new**, and `Replacement gathers` up by that many.

**The row this PR exists to read, and it read as predicted.** Every mana
production is a proposal now, so every land tap is a gather — RE-1's shape,
one proposal per unit that also gathers — and the question the section asked
in advance was whether the hottest path in the engine could carry it without
§8's event-kind gate. `Mana productions` is the denominator that question is
read against, added to the table here and to `fuzz_ab.py` in this sitting
(the `main` column reads `?` for it once, by construction):

| | 2 seats | 4 seats |
|---|---|---|
| CPU/game median (engine arm) | 13.82 → 13.98 ms (**+1.2%**) | 44.51 → 44.83 ms (**+0.7%**) |
| ms / 1,000 walks | 38.18 → 38.62 (+1.2%) | 57.21 → 57.62 (+0.7%) |
| CPU/turn p50 | 0.410 → 0.410 | 0.690 → 0.700 |
| Replacement gathers | 983 → 1063 | 2,012 → 2,163 |
| Mana productions | ? → 81 | ? → 151 |
| Memo hits | 59,133 → 59,397 (+0.4%) | 175,453 → 176,255 (+0.5%) |
| Layer walks / frames / frames-per-walk | 362 / 4,538 / 12.54, identical | 778 / 15,246 / 19.59, identical |

Rounds straddle at both seat counts (two seats: `main` 13.65–14.00, engine
13.79–14.03), and the number is under the 2.5-point gate §11 item 54 named,
so **the gate was not built**. The pooled arm is the bigger board a six-drop
that doubles mana makes — two seats: turns 29.6 → 32.1, spells 22.8 → 24.3,
walks 362 → 394, `Frames/walk` 12.54 → 13.96, CPU/game **+20.1%** with the walk
flat per *frame* (3.04 → 3.02 ms per thousand) — and at four seats +4.0%, a
four-player game being the bigger board already. `--require`: Mana Reflection
cast 119 / resolved 118 in **82 of 200 games (41%)**, 1.46 copies/deck, 100%
board diversity. One four-player `stress` game on the pooled arm ran to the
200-turn cap; at `--max-turns 600` it ends at 208 with nothing else moved, and
`main`'s longest four-player `stress` game is 194.

**The §3 fixture rows, as shipped** (50 games / seed 12345, both pools, the
`pooled` arm):

| | performance | stress |
|---|---|---|
| Wins by seat | 25 (50.0%) / 25 (50.0%) | 23 (46.0%) / 27 (54.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 33.0 | 31.8 |
| Spells cast | 25.6 | 23.9 |
| Lands played | 19.3 | 18.4 |
| Combat w/ atk | 11.5 | 10.6 |
| Creatures died | 8.2 | 6.2 |
| Damage events | 24.2 | 24.1 |
| Total damage | 67.3 | 57.7 |
| Life changes | 15.4 | 16.9 |
| **Layer walks** | **401** | **491** |
| **Board walks** | **279** | **303** |
| **Memo hits** | **73,622** | **82,252** |
| **Layer frames** | **5,856** | **6,009** |
| **Frames/walk** | **14.61** | **12.25** |
| **Dependency checks** | **46** | **9** |
| **Replacement gathers** | **1221** | **1243** |
| **Restriction queries** | **1223** | **1245** |
| Mana productions | 90 | 123 |
| Prevention allocations | 0.00 | 0.06 |
| Replacement prompts | 0.38 | 2.74 |
| Max batch depth | 5 | 5 |

The `main` arm's same rows, for the pool these replace: performance
358 / 242 / 54,349 / 4,201 / 11.74 / 11 / 927 / 930; stress
476 / 282 / 74,632 / 5,567 / 11.70 / 21 / 1026 / 1028. The engine arm's
`Mana productions` at 50 games: 80 and 115.

**The four-player table** (50 games / seed 12345, `--players 4`, both pools,
the `pooled` arm):

| | performance | stress |
|---|---|---|
| Wins by seat | 23 (46.0%) / 19 (38.0%) / 7 (14.0%) / 1 (2.0%) | 11 (22.0%) / 19 (38.0%) / 14 (28.0%) / 6 (12.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 60.9 | 64.5 |
| Spells cast | 44.4 | 47.4 |
| Lands played | 36.1 | 37.9 |
| Combat w/ atk | 25.2 | 25.3 |
| Creatures died | 14.9 | 11.4 |
| Damage events | 53.7 | 61.0 |
| Total damage | 153.9 | 149.9 |
| Life changes | 39.2 | 43.6 |
| Turns after a departure | 21.6 | 21.8 |
| Departed-owned permanents | 0.0 | 0.0 |
| **Layer walks** | **801** | **1,156** |
| **Board walks** | **531** | **677** |
| **Memo hits** | **186,728** | **269,381** |
| **Layer frames** | **15,778** | **22,920** |
| **Frames/walk** | **19.71** | **19.84** |
| **Dependency checks** | **156** | **143** |
| **Replacement gathers** | **2270** | **2677** |
| **Restriction queries** | **2274** | **2682** |
| Mana productions | 155 | 260 |
| Prevention allocations | 0.00 | 0.06 |
| Replacement prompts | 2.54 | 2.98 |
| Max batch depth | 4 | 6 |

The `main` arm's same rows at four seats: performance
788 / 515 / 178,289 / 15,083 / 19.15 / 147 / 2059 / 2064; stress
1,216 / 644 / 256,962 / 19,940 / 16.40 / 111 / 2354 / 2359.

Every arm `deterministic: yes` at both seat counts on both pools — seven
timing rounds each identical to the threaded counters run outside
`=== Timing ===`; zero errors and zero panics everywhere; zero turn limits at
two seats and the one four-seat tail above.

**Re-recorded 2026-09-14 for LK** (CR 113.6, which abilities function in which
zone; `layers-architecture.md` §13d). `PERFORMANCE_POOL` +1 — Wonder, 87 → 88
— and the stress pool +1 (152 → 153, the pooled one; `exiled_ancestor` is a
fixture registered nowhere), so **both tables are a re-record and neither
column is an engine reading.** Wonder resolves in **45% of games** at 20 games
/ seed 12345, and dies like any other creature, so the board carries a
graveyard source for the rest of the game once it does.

**The headline number of this sitting is not in the tables, and the tables are
why.** Registering a card changes the decks, so the two-arm counters differ and
a timing delta cannot be attributed — games got 3.6% shorter here and CPU/game
went *down* 3.8%, which says nothing about the engine. The sitting therefore
built a **third arm with Wonder unregistered**, whose counters are
byte-identical to `main` by construction (`replacement-architecture.md` §11
item 80's technique). That arm read **+16.5%**, which is how
`layers-architecture.md` §13d decision 2's field move was caught: moving
CR 613.7's timestamp off `PermanentState` onto `GameObject` put it one
`HashMap` hop away from `battlefield_ordered` and `battlefield_ids_ordered`,
which run ~5,700 times a game over ~16 permanents (`codebase-state.md` item
77). Post-fix the same arm reads **−2.5%**, counters still identical on both
pools.

**Build the unregistered arm first.** The pool-moving arm cannot tell a 16%
regression from a shorter game, and in this sitting it very nearly did not: the
two-arm read was +5.2%, which is inside the documented spread and would have
shipped.

| | 2 seats | 4 seats |
|---|---|---|
| CPU/game median | 14.73 → 14.17 ms (−3.8%) | 54.23 → 49.33 ms (−9.0%) |
| ms / 1,000 walks | 39.49 → 39.14 (−0.9%) | 63.65 → 63.41 (−0.4%) |
| Layer walks | 373 → 362 | 852 → 778 |
| Layer frames | 4,646 → 4,538 | 16,847 → 15,246 |
| Frames/walk | 12.46 → 12.54 (+0.6%) | 19.78 → 19.59 (−1.0%) |
| Memo hits | 61,908 → 59,133 | 193,334 → 175,453 |

**Frames/walk is the row LJ's block said to watch, and it says the prediction
held.** LJ moved it +1.1% at two seats and +4.1% at four, and the ratio was the
shape of its cost: it scaled with how much of the *named zone* exists. LK puts
a **source** off the battlefield instead, so its cost is per *row* — one
`compute_non_member` walk and cache insert per layer per zone-functioning row —
and should not scale with seat count at all. It does not: +0.6% at two seats
and −1.0% at four, with the four-seat number moving the other way. Every other
row here is the pool, not the engine.

**The §3 fixture rows, as shipped** (50 games / seed 12345, both pools, the
`new` arm):

| | performance | stress |
|---|---|---|
| Wins by seat | 24 (48.0%) / 26 (52.0%) | 30 (60.0%) / 20 (40.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 28.3 | 30.9 |
| Spells cast | 21.8 | 21.5 |
| Lands played | 17.0 | 17.9 |
| Combat w/ atk | 9.5 | 9.9 |
| Creatures died | 6.3 | 4.8 |
| Damage events | 20.4 | 20.7 |
| Total damage | 56.6 | 58.3 |
| Life changes | 13.5 | 15.4 |
| Turns after a departure | ? | ? |
| Departed-owned permanents | ? | ? |
| **Layer walks** | **358** | **441** |
| **Board walks** | **242** | **286** |
| **Memo hits** | **54,349** | **77,676** |
| **Layer frames** | **4,201** | **5,599** |
| **Frames/walk** | **11.74** | **12.70** |
| **Dependency checks** | **11** | **17** |
| **Replacement gathers** | **927** | **1063** |
| **Restriction queries** | **930** | **1066** |
| Prevention allocations | 0.02 | 0.06 |
| Replacement prompts | 0.00 | 0.98 |
| Max batch depth | 5 | 5 |

The `main` arm's same rows, for the pool these replace: performance
376 / 247 / 59,404 / 4,687 / 12.47 / 37 / 999 / 1002; stress
512 / 307 / 89,245 / 6,335 / 12.38 / 27 / 1167 / 1171.

Determinism: three shell `fuzz_games` runs at seed 777, 40 games, identical
line for line but the timing lines (`CLAUDE.md`).

**Re-recorded 2026-09-14 for LJ** (the zone-reaching `ObjectSet`;
`layers-architecture.md` §13c). `PERFORMANCE_POOL` +1 — Yixlid Jailer, 86 → 87
— and the stress pool +2 (150 → 152: Scarwood Treefolk and the pooled one), so
**both tables are a re-record and neither column is an engine reading.**

**This is the first real reading of item 9's `reachable_zones` argument**, which
was that the facility costs nothing until a zone-reaching card is played. Half
of that is structural and needs no measurement: with the mask at `BATTLEFIELD`
the seed's zone loop does not run and `membership` answers as before, so an
ordinary board pays exactly zero (`tests/phase_lj_integration_test.rs` asserts
it). The half worth measuring is what a board that *does* play one pays, and
the pool is what makes that a measured game rather than a fixture — the Jailer
resolves in **130 of 200 games (65%)** at seed 12345.

| | 2 seats | 4 seats |
|---|---|---|
| CPU/game median | 14.84 → 14.94 ms (**+0.7%**) | 55.01 → 56.21 ms (**+2.2%**) |
| ms / 1,000 walks | 39.79 → 40.05 (+0.7%) | 67.25 → 65.97 (**−1.9%**) |
| Layer walks | 373 → 373 | 818 → 852 (+4.2%) |
| Layer frames | 4,596 → 4,646 (+1.1%) | 15,545 → 16,847 (+8.4%) |
| Frames/walk | 12.33 → 12.46 (+1.1%) | 19.01 → 19.78 (+4.1%) |
| Memo hits | 61,262 → 61,908 | 191,269 → 193,334 |

**Read the two middle rows together, because they say different things.** At
four seats the CPU/game delta is +2.2% — the bottom of the documented ~2–6%
spread — while **ms per 1,000 walks went down 1.9%**. The walk did not get
slower. There are more of them (+4.2%) and each frames more objects (+4.1%),
which is precisely the zone members being seeded: from the turn the Jailer
resolves, every card in every graveyard is a member of every pass, and
graveyards only grow. That is the cost the facility has, it is confined to the
one zone the card names, and it is what the mask exists to keep confined — the
unguarded version is every library too, which at four seats is ~400 members a
pass instead of ~40.

**Frames/walk is the row to watch in the next phase that widens a `ZoneSet`.**
It moved 4.1% for a card reaching graveyards at four seats and 1.1% at two,
and the ratio between those two numbers is the shape of the cost: it scales
with how much of the named zone exists, not with how many rows name it.

**The §3 fixture rows, as shipped** (50 games / seed 12345, both pools, the
`new` arm):

| | performance | stress |
|---|---|---|
| Wins by seat | 23 (46.0%) / 27 (54.0%) | 28 (56.0%) / 21 (42.0%) |
| Wins by effect | 0 | 1 |
| Avg turns | 29.6 | 33.0 |
| Spells cast | 23.7 | 24.6 |
| Lands played | 17.7 | 18.9 |
| Combat w/ atk | 10.9 | 11.1 |
| Creatures died | 7.3 | 7.0 |
| Damage events | 23.1 | 26.1 |
| Total damage | 63.4 | 69.5 |
| Life changes | 15.7 | 16.5 |
| Turns after a departure | ? | ? |
| Departed-owned permanents | ? | ? |
| **Layer walks** | **376** | **512** |
| **Board walks** | **247** | **307** |
| **Memo hits** | **59,404** | **89,245** |
| **Layer frames** | **4,687** | **6,335** |
| **Frames/walk** | **12.47** | **12.38** |
| **Dependency checks** | **37** | **27** |
| **Replacement gathers** | **999** | **1167** |
| **Restriction queries** | **1002** | **1171** |
| Prevention allocations | 0.00 | 0.00 |
| Replacement prompts | 0.26 | 0.84 |
| Max batch depth | 4 | 5 |

The `main` arm's same rows, for the pool these replace: performance
369 / 240 / 61,692 / 4,622 / 12.53 / 19 / 1003 / 1005; stress
447 / 281 / 75,034 / 5,331 / 11.94 / 20 / 1076 / 1079.

Both arms `deterministic: yes`; zero errors and zero panics on both pools at
both seat counts. Counters differ from `main` by construction — the pool moved,
so the decks moved — which is why no counter row above is an engine reading.

**Re-recorded 2026-09-14 for RE-8** (CR 701.9's discard and CR 701.22's scry;
`replacement-architecture.md` §9). `PERFORMANCE_POOL` +2 — Mind Rot and Opt,
84 → 86 — and the stress pool +5 (145 → 150: Hymn to Tourach, Nephalia
Academy, Eligeth, Crossroads Augur, and the two pooled ones), so **both tables
are a re-record and neither column is an engine reading**.

**The engine's reading took a fifth arm, and it is the one worth keeping.**
`main`; **engine**, this branch with the five cards unregistered (84 / 145);
**registered**, the old pool (84 / 150); **pooled**, as shipped (86 / 150); and
**engine-oldcleanup**, the engine arm with CR 514.1's one-card-at-a-time
cleanup loop restored. **That fifth arm is `IDENTICAL` to `main` outside
`=== Timing ===` on both pools at two seats and at four** — so two new
producers, `ReplacementDef::by`'s third clause in `applies_to`, a `GameAction`
variant with its three exhaustive arms and a `TemplateAmount` on a template
cost a board with nothing watching them **nothing at all**, which is what §9
predicted of the middle arm. Everything the engine arm *does* move is the
cleanup reshape: the rule is one turn-based action over "enough cards" and the
engine asked one card at a time, so the fix spends the random agent's RNG
differently from that turn on. Counted game by game, **23 of 200 games on
`performance` and 29 of 200 on `stress`** reach a cleanup discard of two or
more; the aggregates move by tenths (avg turns 31.4 → 31.3, gathers
1060 → 1052, walks 384 → 383, prompts 1.14 → 1.16).

**The pooled column is a re-record and a *smaller* board, which is the
direction to read it in.** Two nonland cards in a 36-slot deck dilute what was
there: `Replacement prompts` 1.14 → **0.57** per game on `performance` at two
seats is Hardened Scales meeting a second Scales less often, not a prompt that
stopped being asked, and `Replacement gathers` 1060 → 1018 and avg turns
31.4 → 30.4 say the same thing. At four seats, prompts 2.42 → 1.95 and gathers
2150 → 2135. On `stress` all five cards are in every deck and the board moves
much further: gathers 2345 → 2213 at four seats, prompts 5.64 → 6.79.

**CPU flat or down on every arm, both seat counts.** Two seats, medians of
three interleaved rounds: `main` 16.65 ms, engine 16.26 (−2.3%), registered
16.40 (−1.5%), pooled 15.16 (−8.9%); `CPU/turn p50` 0.430 → 0.440 / 0.440 /
0.430. Four seats, medians of two: 55.07 ms → +0.7% (engine-oldcleanup), +0.8%
(engine), −2.0% (registered), −2.0% (pooled); `CPU/turn p50` 0.775 → 0.785 /
0.770 / 0.770 / 0.805. The pooled −8.9% is the shorter game and not a faster
walk — `ms / 1,000 queries` is −1.6%. **Reachability**, `--require`, 200 games
— `performance`: Mind Rot cast 173, resolved 172, in **129 of 200 games
(64%)**, copies/deck 1.43; Opt cast 210, resolved 209, in **139 of 200 (70%)**,
1.52 — unchanged by the counter fix below, `performance` holding none of the
three cards that replace a resolution's last move. `stress`: Hymn to Tourach cast 149, resolved
**146** (re-read at the review: the first pass counted `cause == Resolved`,
and seventeen Hymns had resolved under a Leyline of the Void, which replaces
CR 608.2m's graveyard move — §11 item 91), in **102 of 200 (51%)**, 1.28;
Nephalia Academy cast 221, resolved 221, in **149 of 200 (74%)**, 1.31;
**Eligeth, Crossroads Augur** cast 121, resolved 121, in **99 of 200 (50%)**,
1.23 — the first `--require` row this project has for a card whose name
contains a comma, which the now-repeatable flag is what bought. Zero errors,
zero panics and **zero turn limits** on every arm, both pools, both seat
counts; the longest `stress` game at two seats is `main`'s own 155 turns, which
the engine arm reproduces and the registered decks reshuffle away (78).
`deterministic: yes` on every arm; three shell runs at one seed, `--threads 1`,
both pools and both seat counts, `IDENTICAL` outside `=== Timing ===`.

| | performance (86 cards) | stress (150 cards) |
|---|---|---|
| P0 / P1 | 24 (48.0%) / 26 (52.0%) | 20 (40.0%) / 30 (60.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 29.5 | 31.0 |
| Spells cast | 23.1 | 22.9 |
| Lands played | 18.0 | 18.3 |
| Combat w/ atk | 10.6 | 10.3 |
| Creatures died | 6.6 | 5.1 |
| Damage events | 22.3 | 22.4 |
| Total damage | 64.0 | 56.0 |
| Life changes | 15.9 | 15.4 |
| **Layer walks** | **369** | **447** |
| **Board walks** | **240** | **281** |
| **Memo hits** | **61,692** | **75,034** |
| **Layer frames** | **4,622** | **5,331** |
| **Frames/walk** | **12.53** | **11.94** |
| **Dependency checks** | **19** | **20** |
| **Replacement gathers** | **1003** | **1076** |
| **Restriction queries** | **1005** | **1079** |
| Prevention allocations | 0.00 | 0.02 |
| Replacement prompts | 0.22 | 2.94 |
| Max batch depth | 4 | 5 |

**The four-player table, re-recorded** — the pool moved, so this one moves with
it. `engine-oldcleanup` is `IDENTICAL` to `main` here too, on both pools, so
the attribution holds at both seat counts; `engine` and `registered` produce
identical counters on `performance`, as a registration that does not touch that
pool should. `Replacement prompts` 3.08 → 2.02 on `performance` and 4.94 →
9.60 on `stress` at 50 games — the same dilution one way and the five new
cards the other. No turn limit and no draw on any arm.

| 4 players, 50 games / seed 12345 | performance (86 cards) | stress (150 cards) |
|---|---|---|
| Wins by seat | 21 (42.0%) / 13 (26.0%) / 12 (24.0%) / 4 (8.0%) | 23 (46.0%) / 12 (24.0%) / 11 (22.0%) / 4 (8.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 57.5 | 66.9 |
| Spells cast | 43.6 | 48.7 |
| Lands played | 35.2 | 38.3 |
| Combat w/ atk | 23.4 | 26.7 |
| Creatures died | 14.7 | 13.7 |
| Damage events | 49.6 | 59.4 |
| Total damage | 155.7 | 153.0 |
| Life changes | 33.8 | 40.5 |
| Turns after a departure | 19.8 | 20.8 |
| Departed-owned permanents | 0.0 | 0.0 |
| **Layer walks** | **816** | **1,154** |
| **Board walks** | **516** | **667** |
| **Memo hits** | **170,069** | **266,040** |
| **Layer frames** | **14,588** | **21,183** |
| **Frames/walk** | **17.88** | **18.36** |
| **Dependency checks** | **113** | **126** |
| **Replacement gathers** | **1980** | **2437** |
| **Restriction queries** | **1985** | **2444** |
| Prevention allocations | 0.00 | 0.08 |
| Replacement prompts | 2.02 | 9.60 |
| Max batch depth | 4 | 5 |

**Re-recorded 2026-09-13 for RE-5** (CR 614.16's counter half, 122.1, 122.6,
122.6a; `replacement-architecture.md` §9). `PERFORMANCE_POOL` +1 — Hardened
Scales, 83 → 84 — and the stress pool +6 (139 → 145: Doubling Season,
Vorinclex, Monstrous Raider, Winding Constrictor, Live Fast, Primal Vigor),
so **both tables are a re-record and neither column is an engine reading**.
The engine's reading is the four arms (`replacement-architecture.md` §11
item 80): `main`; **engine**, the branch with the cards *unregistered*;
**registered**, the old pool; **pooled**, as shipped. **Re-measured after
the review's theme A (2026-09-14) — the named putter and the split pattern
arms — and every row was identical; re-recorded after theme B the same day
— the commutation table — whose cells the random agent reaches rarely**:
`Replacement prompts` 2.14 → 2.12 at two seats and 3.12 → 3.08 at four on
`performance`, 5.04 → 4.94 on `stress` at four, and the four-player rows
re-routed by those few games; at 200 games the pooled row reads 1.14 → 1.14
and 2.44 → 2.42. The prompts the pool still asks are not counter pairs — two
Guardian Seraphs (`PreventUpTo` beside `PreventUpTo`, the table's next
cell) and devour beside Master Biomancer (opaque by design). The engine arm
stays `IDENTICAL` to `main` everywhere but `stress` at four seats, where the
one new cell in `main`'s registry — Divine Visitation beside Parallel Lives —
re-routes a game or two (prompts 11.28 → 11.27, gathers 2385 → 2385). CPU
flat on all four arms, both times.

**Engine and registered `IDENTICAL` to `main` outside `=== Timing ===` on
`performance` at two seats and at four, and the engine arm `IDENTICAL` on
`stress` at both** (200 games / seed 12345) — the counter pattern's entry
door is an arm no def in the old pool or in `main`'s registry reaches, the
`Amount` legs are never entered, and the subject enum changes no proposal's
count. CPU/game +0.1% (engine) and +1.1% (registered) at two seats, −0.5%
and −1.5% at four; `CPU/turn p50` 0.400 → 0.400 and 0.710 → 0.710 — flat.

**The pooled column is a re-record, and the row the section said would move
is the one that did.** `Replacement prompts` 0.74 → 1.14 per game at two
seats and 1.83 → 2.44 at four on `performance`: Hardened Scales beside a
second Scales, and beside Master Biomancer at an entry's second iteration —
additive pairs with one outcome, asked because the predicate has no shape
for them yet (`backlog.md` §2.29). `Replacement gathers` 1043 → 1060 at two
seats, the sweep Scales opens on every `AddCounters` and counter-bearing
entry; avg turns 30.7 → 31.4, total damage 69.3 → 59.0, a different board.
CPU/game −0.4% and +1.3%, `ms / 1,000 queries` −2.4% and −0.7%: flat.
**Reachability**, `--require`, 200 games — `performance`: Hardened Scales
cast 226, resolved 226, in **136 of 200 games (68%)**, copies/deck 1.54;
`stress`: Doubling Season cast 119, resolved 119, in **97 of 200 (48%)**, 1.29; Winding Constrictor cast 150, resolved 148, in **112 of 200 (56%)**, 1.23; Live Fast cast 133, resolved 119, in **91 of 200 (46%)**, 1.26; Primal Vigor cast 136, resolved 136, in **101 of 200 (50%)**, 1.33. Zero errors, zero panics, every arm; the two
two-seat `stress` turn limits are `main`'s own stall (seeds 12386 and
12538), reproduced line for line by the engine arm and reshuffled away by
the registered decks.

| | performance (84 cards) | stress (145 cards) |
|---|---|---|
| P0 / P1 | 28 (56.0%) / 22 (44.0%) | 25 (50.0%) / 25 (50.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 31.4 | 31.4 |
| Spells cast | 25.1 | 22.1 |
| Lands played | 19.0 | 17.6 |
| Combat w/ atk | 11.6 | 9.3 |
| Creatures died | 7.7 | 4.5 |
| Damage events | 24.2 | 20.0 |
| Total damage | 60.9 | 57.3 |
| Life changes | 15.9 | 17.1 |
| **Layer walks** | **395** | **498** |
| **Board walks** | **256** | **327** |
| **Memo hits** | **67,768** | **103,220** |
| **Layer frames** | **5,098** | **6,975** |
| **Frames/walk** | **12.92** | **14.00** |
| **Dependency checks** | **16** | **11** |
| **Replacement gathers** | **1085** | **1090** |
| **Restriction queries** | **1087** | **1092** |
| Prevention allocations | 0.00 | 0.04 |
| Replacement prompts | 2.12 | 1.80 |
| Max batch depth | 6 | 5 |

**The four-player table, re-recorded** — the pool moved, so this one moves
with it. Engine and registered are `IDENTICAL` to `main` on `performance`
here too, and the pooled column is the same re-record: `Replacement
prompts` 1.83 → 2.44 at 200 games, `Layer walks` 839 → 826, gathers
2145 → 2145, CPU/game +1.3% (`CPU/turn p50` 0.710 → 0.685). No turn limit
and no draw on any arm at four seats. Three shell runs at one seed and
`--players 4`: `IDENTICAL` outside `=== Timing ===` on both pools.

| 4 players, 50 games / seed 12345 | performance (84 cards) | stress (145 cards) |
|---|---|---|
| Wins by seat | 22 (44.0%) / 19 (38.0%) / 6 (12.0%) / 3 (6.0%) | 24 (48.0%) / 14 (28.0%) / 9 (18.0%) / 3 (6.0%) |
| Wins by effect | 0 | 0 |
| Avg turns | 59.2 | 61.5 |
| Spells cast | 44.1 | 44.5 |
| Lands played | 35.9 | 36.5 |
| Combat w/ atk | 24.2 | 23.9 |
| Creatures died | 15.0 | 11.2 |
| Damage events | 51.2 | 57.4 |
| Total damage | 144.2 | 166.8 |
| Life changes | 36.1 | 42.6 |
| Turns after a departure | 20.9 | 18.4 |
| Departed-owned permanents | 0.0 | 0.0 |
| **Layer walks** | **818** | **1,083** |
| **Board walks** | **529** | **618** |
| **Memo hits** | **182,914** | **251,639** |
| **Layer frames** | **15,586** | **19,482** |
| **Frames/walk** | **19.05** | **17.99** |
| **Dependency checks** | **116** | **227** |
| **Replacement gathers** | **2066** | **2258** |
| **Restriction queries** | **2070** | **2264** |
| Prevention allocations | 0.00 | 0.02 |
| Replacement prompts | 3.08 | 4.94 |
| Max batch depth | 6 | 6 |

**Re-recorded 2026-09-13 for RE-4** (CR 614.16's token half;
`replacement-architecture.md` §9) — **re-measured after its review**, whose
three themes moved the engine (a rider carries its lineage; an exit beside
enters-with effects is not asked; the creation pattern's kind and its
template) and the registry (Divine Visitation, Bard, King of Dale).
`PERFORMANCE_POOL` +2 — Parallel Lives and Raise the Alarm, 81 → 83 — and the
stress pool +6 (133 → 139), so **both tables are a re-record and neither
column is an engine reading**. The engine's reading is the arms, and there
are four now (`replacement-architecture.md` §11 item 80): `main`; **engine**,
the branch with the cards *unregistered*, so `main`'s registry in both pools;
**registered**, the old pool; **pooled**, as shipped.

**Both `performance` pools: engine and registered `IDENTICAL` to `main`
outside `=== Timing ===` and the two rows the new binary prints**, at two
seats and at four (200 games / seed 12345). Nothing in the 81 creates a token,
meets a rider across a Reflection, or puts an exit beside an enters-with, so
the review's three themes cost the pool nothing and change no seeded stream.
CPU/game −0.4% and +0.1% at two seats, −1.0% and +0.2% at four; `CPU/turn
p50` 0.390 → 0.390 and 0.720 → 0.720 — flat.

**The engine arm on `stress` is the reading the middle arm could not give,
and it is exact.** At two seats 188 of 200 games are byte-identical to
`main`; eleven differ by exactly one `TokenCreated` line per Zombie Kalitas
makes (26 lines across them); and **one game diverges** — seed 12441 (game
97), Containment Priest and Root Maze beside a Dryad Arbor, the one-exit
prompt theme C stopped asking, after which the random agent's stream is a
different game. That one game is `Layer walks` 521 → 519 and the win split
103/97 → 102/98. **No two-seat game was re-routed by theme A**: the boards
with a Collector and a Reflection (games 150, 153, 168, 182) differ only by
their creation lines. Four seats: `Layer walks` 1,240 → 1,240, gathers
2517 → 2513, frames 23,502 → 23,476, seats 91/53/41/14 → 92/54/39/14 — the
same handful of re-routed games.

**Two rows are new, and both are baselines from here.** `Replacement
prompts` — CR 616.1 questions actually put to a player, per game — is 0.49 on
`performance` at two seats (0.74 pooled) and 2.38 at four (1.83 pooled), 2.54
on the engine arm's `stress` at two seats and 23.80 at four; it is the row a
phase that widens `ordering_cannot_change_outcome` moves, and nothing else
should. `Max batch depth` — the deepest nesting any game reached — is **7**,
on `stress` at both seat counts (6 on `performance`), across 1,600 games;
`engine::actions::BATCH_NESTING_LIMIT` is 32, which is that with headroom.

*The pooled column is a re-record and a bigger board.* Two 1/1 Soldiers a
cast are two permanents that attack, block, die and are walked: at 200 games
on `performance`, `Replacement gathers` 1002 → 1043, `Layer walks` 373 → 386,
avg turns 29.9 → 30.7, total damage 57.8 → 69.3, max turns 72 → 95 (the p99,
43 → 78 ms). CPU/game +11.0% this sitting and +12.9% and +9.0% the two before,
`ms / 1,000 queries` +1.0%, `CPU/turn p50` 0.390 → 0.400: more game, not a
slower walk. **Reachability**, `--require`, 200 games — `performance`:
Parallel Lives cast 168, resolved 168, in **116 of 200 games (58%)**,
copies/deck 1.54; Raise the Alarm 201 / 201 in **140 (70%)**, 1.49; `stress`:
Divine Visitation 131 / 131 in **98 (49%)**, 1.30; Bard, King of Dale
135 / 135 in **94 (47%)**, 1.29. Zero errors, zero panics, every arm.

| | performance (83 cards) | stress (139 cards) |
|---|---|---|
| P0 / P1 | 26 (52.0%) / 24 (48.0%) | 29 (58.0%) / 20 (40.0%) |
| Avg turns | 30.1 | 35.6 |
| Spells cast | 23.3 | 25.3 |
| Lands played | 18.3 | 19.4 |
| Combat w/ atk | 10.8 | 13.6 |
| Creatures died | 7.5 | 6.7 |
| Damage events | 23.1 | 27.5 |
| Total damage | 68.3 | 72.4 |
| Life changes | 14.5 | 19.0 |
| **Layer walks** | **389** | **663** |
| **Board walks** | **249** | **463** |
| **Memo hits** | **62,489** | **312,875** |
| **Layer frames** | **4,545** | **18,214** |
| **Frames/walk** | **11.69** | **27.48** |
| **Dependency checks** | **23** | **45** |
| **Replacement gathers** | **1021** | **1513** |
| **Restriction queries** | **1023** | **1518** |
| Prevention allocations | 0.02 | 0.00 |
| Replacement prompts | 0.62 | 18.66 |
| Max batch depth | 4 | 5 |

**The `stress` column's missing game is a turn limit**, and there are two in
200: seeds 12386 and 12538, both the Circle of Protection: Red and Words of
Worship stall — a two-seat endgame the random agent cannot end, RE-3's and
RD-3's cards on decks the two new registrations reshuffled, with no token in
either game's last four hundred events (74 Circle activations in one, 58
Words in the other). Neither is an engine reading, for RE-1's reason.

**The four-player table, re-recorded** — the pool moved, so this one moves
with it. Engine and registered are `IDENTICAL` to `main` on `performance`
here too, and the pooled column is the same bigger board: gathers 2102 → 2145
at 200 games, avg turns 61.0 → 61.4, CPU/game −4.5% (a different board;
`Dependency checks` 171 → 111). No turn limit and no draw on any arm at four
seats: the loop RE-4's landing found at seed 12523 is theme A's fixture now,
and the decks it ran on have moved. Three shell runs at one seed and
`--players 4` identical outside `=== Timing ===` on both pools.

| 4 players, 50 games / seed 12345 | performance (83 cards) | stress (139 cards) |
|---|---|---|
| Wins by seat | 24 (48%) / 13 (26%) / 9 (18%) / 4 (8%) | 25 (50%) / 11 (22%) / 10 (20%) / 4 (8%) |
| Wins by effect | 0 | 0 |
| Avg turns | 55.8 | 65.5 |
| Spells cast | 42.0 | 48.5 |
| Lands played | 33.5 | 38.7 |
| Combat w/ atk | 22.5 | 26.9 |
| Creatures died | 14.7 | 12.5 |
| Damage events | 50.2 | 59.6 |
| Total damage | 156.0 | 155.8 |
| Life changes | 35.5 | 41.6 |
| Turns after a departure | 18.5 | 21.3 |
| Departed-owned permanents | 0.0 | 0.0 |
| **Layer walks** | **786** | **1,303** |
| **Board walks** | **498** | **719** |
| **Memo hits** | **161,430** | **316,380** |
| **Layer frames** | **14,310** | **24,881** |
| **Frames/walk** | **18.21** | **19.10** |
| **Dependency checks** | **90** | **154** |
| **Replacement gathers** | **1922** | **2532** |
| **Restriction queries** | **1926** | **2542** |
| Prevention allocations | 0.00 | 0.04 |
| Replacement prompts | 2.82 | 19.18 |
| Max batch depth | 5 | 7 |

At 200 games the same run reads: `performance` avg turns 61.4, wins by seat
96/59/31/14, turns after a departure 21.9, gathers 2145, `Layer walks` 839,
CPU/game median 49.69 ms with `CPU/turn p50` 0.74 ms; `stress` avg turns
63.6, 89/51/40/20, 20.8, gathers 2385, `Layer walks` 1,189. The stress column
is not an engine reading, for RE-1's reason: the registry grew by six.

**Re-recorded 2026-09-13 for RE-7** (CR 800.4a; `replacement-architecture.md`
§9). `PERFORMANCE_POOL` unchanged at 81 and the stress pool at 133 — RE-7
registers no card — so unlike the re-records before it, **this whole table is an
engine reading and not a pool one**, and it is the first four-player diff the
project has taken.

**The two-player table is not re-recorded, and that is the reading rather
than an omission.** RE-7 is CR 800.4, which CR 800.1 scopes to a game that
*began* with more than two players, so both two-player pools are byte-identical
to `main` outside `=== Timing ===` — there is no number to write down. The two
tables are not alternatives: **a phase re-records whichever ones it moves**, and
from RE-6 on that is a question with two answers rather than one. A phase that
changes a two-player path re-records the two-player table; a phase whose rules only
exist at three or more re-records this one; a phase that moves the pool
re-records both, because the decks changed under each.

**The row it exists to move: "Departed-owned permanents" 32.2 → 0.0 and
34.2 → 0.0** at 200 games, 32.7/32.8 → 0.0/0.0 at the 50 below.
`codebase-state.md` item 108 closed with it. **Every engine-work row moves and
one number says why: `Frames/walk` 21.22 → 18.69.** A walk fills the whole
working set (LI-1), and the working set is a board about 32 permanents smaller,
so each one got cheaper: `Memo hits` −11.8%, `Layer frames` −8.0%,
`Dependency checks` 307 → 171. CPU/game median **−15.8% and −14.0%** across
two sittings, `ms / 1,000 walks` −19.4% and −17.7%, `CPU/turn p50` 0.860 →
0.800 and 0.840 → 0.800. **That is not a speed-up the engine earned**; it is a board it
stopped carrying, and it is the size of what item 108 was costing every
four-player number measured before it.

**`Layer walks` is the one row that goes *up*, 802 → 838, and it is CR 603.6c's
price.** A permanent leaving the game carries the CR 603.10a frame a
leaves-the-battlefield trigger will read, and the frame is an uncached walk
each — about 33 per game across three departures, +4.5%, and **zero at two
seats**. It buys back more than it costs here and would not on a wider board;
recording it as a rate rather than as a total is what makes that checkable
later.

`Replacement gathers` 2104 → 2102 and `Restriction queries` 2107 → 2107 —
flat, as predicted: this PR adds no proposal. **Both two-player pools are
`IDENTICAL` outside `=== Timing ===`**, which is CR 800.1's doing rather than a
gate's: the section is about what two-player games cannot do.

**The gameplay rows below are a different board, not a delta** — the departed
players' creatures stopped blocking and attacking — so `Avg turns`, `Total
damage` and the seat shares moved and should be read as a fresh fixture.
**Both of `stress`'s outliers resolved, and reading them before the averages
is what found them**: `Hit turn limit` 1 → 0 (seed 12413 ran to turn 200 with
**99** departed-owned permanents on the board and now ends at 196), `Draw`
1 → 0 (seed 12492's CR 104.4a draw is a game that diverged, not a rule that
changed: with three seats' boards gone it ends at turn 43 instead of 46), and
**`Wins by effect` 0 → 1** — seed 12410 is Thought Reflection beside Laboratory
Maniac in a draw step, the first card the last in the library and the second
replaced by the win. RE-6's "games ended by a win: zero on every table" loses
its zero here. Zero errors and zero panics on both pools, both arms; three
shell runs at one seed and `--players 4` identical outside `=== Timing ===`.

| 4 players, 50 games / seed 12345 | performance (81 cards) | stress (133 cards) |
|---|---|---|
| Wins by seat | 27 (54%) / 9 (18%) / 10 (20%) / 4 (8%) | 26 (52%) / 15 (30%) / 6 (12%) / 3 (6%) |
| Wins by effect | 0 | 0 |
| Avg turns | 63.8 | 62.7 |
| Spells cast | 47.2 | 45.0 |
| Lands played | 37.9 | 36.4 |
| Combat w/ atk | 26.5 | 23.7 |
| Creatures died | 16.3 | 9.2 |
| Damage events | 56.5 | 51.4 |
| Total damage | 162.2 | 138.2 |
| Life changes | 39.0 | 34.5 |
| Turns after a departure | 23.9 | 20.0 |
| **Departed-owned permanents** | **0.0** | **0.0** |
| **Layer walks** | **899** | **1,112** |
| **Board walks** | **569** | **601** |
| **Memo hits** | **201,330** | **224,139** |
| **Layer frames** | **17,314** | **17,982** |
| **Frames/walk** | **19.26** | **16.17** |
| **Dependency checks** | **137** | **153** |
| **Replacement gathers** | **2238** | **2220** |
| **Restriction queries** | **2243** | **2226** |
| Prevention allocations | 0.00 | 0.02 |

At 200 games the same run reads: `performance` avg turns 61.0, wins by seat
95/58/34/13, turns after a departure 20.8, departed-owned permanents 0.0,
gathers 2102, `Layer walks` 838, CPU/game median 56.27 ms with `CPU/turn p50`
0.80 ms; `stress` avg turns 66.2, 91/53/41/14 and one win by effect, 22.6 and
0.0, gathers 2517, `Layer walks` 1,240 — where `main`'s was 1,239, so the
frame's cost and the smaller board cancel almost exactly on the wider pool.
**Do not carry the milliseconds across sittings**: the same two binaries read
58.87/48.00 ms in one sitting and 66.85/56.27 in the next, which is §8's point
about a stored ms, and the ratio is the finding.

**What a seat costs, measured in one sitting on one binary so the ratio means
something** (200 games / seed 12345, `performance`, `--threads 1`): two players
**14.94 ms/game** and `CPU/turn p50` **0.430 ms** over 29.9 turns; four players
**56.70 ms/game** and **0.800 ms** over 61.0. So a four-player game costs
**3.8×** a two-player one, lasts **2.0×** as long, and its turn is **1.86×** as
expensive — the last figure being the board being wider, and the one RE-7 moved
(it was 2× when item 108 left 32 permanents on the table). **This pair is the
number v1 reads**, since `CLAUDE.md` names four-player Commander and highly
parallel CLI games as the two use cases, and it is worth re-taking whenever a
phase claims to be flat: "flat at two seats" has been true of a phase that was
not flat at four.

**Re-recorded 2026-09-12 for RE-6** — `PERFORMANCE_POOL` +1 (Laboratory
Maniac, 80 → 81) and the stress pool +4 (129 → 133). **The middle arm moves
exactly the row §9 predicted and nothing else in gameplay**: `Replacement
gathers` 999 → 1000 and `Restriction queries` 1001 → 1002 per game on
`performance` — the loss, proposed once per game — with every gameplay row
identical to `main` and `Layer walks` identical. `Memo hits` 61,652 → 61,444
(−0.3%) is the one other movement, and it is CR 104.1: a player who had just
lost used to keep receiving priority until the phase ended, and the questions
their random agent asked in that tail are gone. So the middle arm reads
`differ` outside `=== Timing ===` on both pools and the check is the
aggregates, which are identical to the printed digit. CPU/game −1.0% and −1.7%
across two sittings, `ms / 1,000 walks` −1.0% and −1.7%, `CPU/turn p50`
0.440 → 0.440 and 0.490 → 0.480: flat, and the two sittings' medians (15.7
and 17.5 ms) are §8's reminder about stored milliseconds.

*The pooled column is a re-record, not a speed-up.* Laboratory Maniac is a 2/2
for three displacing a slot's share of costlier cards: `Layer walks` 385 → 373,
`Layer frames` 4,629 → 4,487, `Dependency checks` 41 → 34, CPU/game −4.7% and
−5.0%. **Reachability:** `--require "Laboratory Maniac"` on `performance`, 200
games / seed 12345: cast 203, resolved 201, in **117 of 200 games (58%)**,
copies/deck 1.57, board diversity 100%. **Games ended by a win: zero** on
every table at 200 games; the path that makes decking a win is walked in 58%
of games and reached its end in none at two seats — a 30-turn game does not
empty a 60-card library. Zero errors, zero panics.

| | performance (81 cards) | stress (133 cards) |
|---|---|---|
| P0 / P1 | 30 (60.0%) / 20 (40.0%) | 26 (52.0%) / 24 (48.0%) |
| Avg turns | 30.8 | 30.4 |
| Spells cast | 24.3 | 22.6 |
| Lands played | 18.9 | 18.0 |
| Combat w/ atk | 10.9 | 9.5 |
| Creatures died | 7.3 | 4.5 |
| Damage events | 22.9 | 20.9 |
| Total damage | 58.9 | 52.1 |
| Life changes | 15.2 | 15.7 |
| **Layer walks** | **388** | **490** |
| **Board walks** | **250** | **289** |
| **Memo hits** | **62,231** | **76,188** |
| **Layer frames** | **4,774** | **5,534** |
| **Frames/walk** | **12.31** | **11.30** |
| **Dependency checks** | **32** | **20** |
| **Replacement gathers** | **1043** | **1045** |
| **Restriction queries** | **1045** | **1047** |
| Prevention allocations | 0.00 | 0.02 |

**The stress column is not an engine reading**, for RE-1's reason: the
registry grew by four, so the arms play different decks from `main` there.

**The four-player table (first recorded 2026-09-12, RE-6) — RE-7's baseline,
never diffed against a two-player arm.** `fuzz_games --players 4`, the same
seed and the same shape as the table above, plus the two rows only a table of
three or more can move. `python plans/fuzz_ab.py --players 4 --arm
re6=<binary>` prints all of it; RE-7 runs it with two arms and diffs.
**Departed-owned permanents** is `codebase-state.md` item 108's wrong answer
counted — permanents a player who has left still owns when the game ends —
and RE-7 zeroes it. Zero errors and zero panics on both pools. Two `stress`
games of 200 are worth a sentence each: one ran to its 200th turn and ended
there with a win (seed 12413, all four seats took about fifty turns), and one
is **CR 104.4a's draw** — at turn 46 the two survivors dealt each other lethal
combat damage in one damage step, both losses were members of one check, and
the batch settled a draw rather than crowning whichever performed second
(seed 12492), which is the per-batch settlement doing in a random game what
its test says. Three shell runs at one seed line-for-line outside
`=== Timing ===`. **Read the first version of this table as a warning**: its
first run had avg turns 86.6 and total damage 496 per game, because a departed
seat was still an attack target and the random agent had been hitting empty
chairs for a hundred turns — CR 506.2, fixed in RE-6
(`replacement-architecture.md` §11 item 66). A wider table's first number is a
measurement of the harness until the harness is checked.

| 4 players, 50 games / seed 12345 | performance (81 cards) | stress (133 cards) |
|---|---|---|
| Wins by seat | 24 (48%) / 12 (24%) / 9 (18%) / 5 (10%) | 25 (50%) / 14 (28%) / 7 (14%) / 4 (8%) |
| Wins by effect | 0 | 0 |
| Avg turns | 62.7 | 61.9 |
| Spells cast | 45.0 | 43.6 |
| Lands played | 37.4 | 36.2 |
| Combat w/ atk | 26.1 | 23.7 |
| Creatures died | 16.0 | 8.6 |
| Damage events | 55.8 | 51.9 |
| Total damage | 159.2 | 137.6 |
| Life changes | 38.9 | 35.9 |
| Turns after a departure | 22.9 | 19.2 |
| **Departed-owned permanents** | **32.7** | **32.8** |
| **Layer walks** | **848** | **1,067** |
| **Board walks** | **520** | **546** |
| **Memo hits** | **223,086** | **244,752** |
| **Layer frames** | **18,326** | **18,965** |
| **Frames/walk** | **21.60** | **17.77** |
| **Dependency checks** | **185** | **207** |
| **Replacement gathers** | **2185** | **2191** |
| **Restriction queries** | **2189** | **2197** |
| Prevention allocations | 0.00 | 0.06 |

At 200 games the same run reads: `performance` avg turns 61.2, wins by seat
87/63/36/14, turns after a departure 21.0, departed-owned permanents 32.2,
gathers 2104, CPU/game median 75.0 ms with `CPU/turn p50` 0.96 ms — a
four-player game costs about four and a half times a two-player one and lasts
twice as long, so the turn is twice as expensive, which is the board being
twice as wide; `stress` avg turns 66.3, 80/57/49/13 and one draw, 22.8 and
34.2, gathers 2502.

**Re-recorded 2026-09-12 for RE-3** — `PERFORMANCE_POOL` +1 (Rhox Faithmender,
79 → 80) and the stress pool +6 (123 → 129). **The middle arm is identical to
`main` outside `=== Timing ===` on both pools**, which is the byte-level form of
§9's prediction: two `EventPattern` arms, an `AmountRewrite` variant, two
templates and a field on `Restriction::Event` cost a game nothing until a card
watches one. CPU median 15.73 → 15.49 ms (−1.5%; main 15.36–16.11, middle
15.32–15.77 — straddling, so flat). Seven rounds, on RE-2's rule.

*The shipped arm's CPU/game is up and it is the card, which the per-unit rows
are what say.* Two sittings put CPU/game at **+4.0% and +2.1%** — both inside
§8's 2–6% spread, both with the arms' rounds barely straddling, and the 1.9
points between them is what §8 already says a stored ms number is. What is
stable across both is the reading that matters: `ms / 1,000 walks` **+0.2% then
−1.6%**, and `CPU/turn p50` **+2.3% then 0.0%**. The walk did not get slower;
there are more of them. Rhox Faithmender is a 1/5 lifelinker that doubles its own
lifelink, so at 200 games `performance` runs 29.6 → 29.8 turns, spells 22.8 →
23.4, damage events 19.6 → 21.3, total damage 56.1 → 61.1, life changes 12.9 →
13.9, layer walks 371 → 385. `Replacement gathers` **974 → 999 (+25)** is one
per doubled gain, and the honest reading is that **the sweep was already running
on every lifelink gain and finding nothing** — this phase gave it something to
find rather than giving it somewhere new to look. Per turn: 32.9 → 33.5
gathers, 0.450 → 0.460 `CPU/turn p50`, which is the slot.

| | performance (80 cards) | stress (129 cards) |
|---|---|---|
| P0 / P1 | 29 (58.0%) / 21 (42.0%) | 30 (60.0%) / 20 (40.0%) |
| Avg turns | 31.9 | 30.8 |
| Spells cast | 24.9 | 22.0 |
| Lands played | 19.2 | 18.2 |
| Combat w/ atk | 11.6 | 9.3 |
| Creatures died | 7.8 | 4.9 |
| Damage events | 23.9 | 20.1 |
| Total damage | 62.2 | 55.6 |
| Life changes | 15.4 | 14.6 |
| **Layer walks** | **406** | **494** |
| **Board walks** | **264** | **296** |
| **Memo hits** | **67,873** | **77,925** |
| **Layer frames** | **5,107** | **5,922** |
| **Frames/walk** | **12.58** | **11.98** |
| **Dependency checks** | **32** | **13** |
| **Replacement gathers** | **1086** | **1058** |
| **Restriction queries** | **1089** | **1063** |
| Prevention allocations | 0.00 | 0.04 |

**Reachability.** `--require "Rhox Faithmender"` on `performance`, 200 games /
seed 12345: cast 176, resolved 174, in **114 of 200 games (57%)**, copies/deck
1.52 — between Eon Hub's 64% at five mana and Thought Reflection's 46% at seven,
which is where a four-drop belongs. **Reach is not the number that mattered
here**, and §9 said which was: it is the only RE consumer that needs no second
card to do anything, because Knight of Meadowgrain and Vampire Nighthawk were
already pooled and lifelink's contained gain was already a proposal.
Zero errors and zero panics. The other five were forced through `stress` the
same way, since three of them are the phase's kind-changing substitutions:
Tainted Remedy 164/164 in 119 games (60%), Words of Worship 172/172 in 117
(58%), Ali from Cairo 161/160 in 112 (56%), Alhammarret's Archive 148/148 in 106
(53%), Skullcrack 215/196 in 131 (66%) — zero errors and zero panics on each.
Skullcrack's 19 unresolved casts are CR 608.2b doing its job: it targets, and a
target that has left is a spell that does not resolve.

**The stress column is not an engine reading**, for RE-1's and RE-2's reason:
`default_registry` grew by six, so the middle and shipped arms play different
decks from `main` on that pool. `Avg turns` 29.0 → 32.3 and `Memo hits` +44% are
the six new cards being drawn.

**Re-run at the PR's review, and the stress column moved by a hair — the second
instance of RE-2's rule, and the first that was predicted.** The review added
`ordering_cannot_change_outcome`'s fourth shape, which suppresses CR 616.1's
prompt between two identical substitutions. *Answer-preserving is not
stream-preserving*: `RandomDecisionProvider::pick_n` draws from its own `StdRng`
on every prompt, so a game in which two Tainted Remedies met one life gain lost
a draw and shifted from there. `performance` is **identical to the digit** — no
pooled card carries two identical `Instead` statics — and `stress` moves `Avg
turns` 30.9 → 30.8, gathers 1062 → 1058, `Memo hits` −0.4%: less than one game's
worth, which is the check the rule prescribes. The table above is the re-run.
RC-4's entry suppression had the same property and nothing said so; this is the
line that says it.

**Re-recorded 2026-09-11 for RE-2** — `PERFORMANCE_POOL` +1 (Thought
Reflection, 78 → 79) and the stress pool +4 (119 → 123). **The middle arm is
free, and the shipped arm is faster than `main`** — both readings need a
sentence, and neither says what it looks like it says.

*The engine's share, `performance`, 200 games / seed 12345.* Every
seed-dependent gameplay row is **identical to `main`** — turns 31.6, spells
23.6, lands 18.4, combats 10.1, deaths 7.1, damage events 21.0, total damage
59.1, life changes 13.7, wins 117/83 — and so is every layer row: walks 375,
board walks 249, frames 4,549, frames/walk 12.13, dependency checks 24, all
unchanged. `Memo hits` 64,069 → 64,160 (+0.14%). What moved is
**`Replacement gathers` 1003 → 1037 and `Restriction queries` 1006 → 1039,
both +34** at 31.6 turns a game — one per turn for the draw step's instruction
plus about two per game for the pool's cantrips, which is §9's prediction
("+1 per draw instruction, so roughly +1 per turn plus one per cantrip") to
within a rounding. The new proposal is the *outer*; the inner is the event that
was already being proposed, which is why the count moves by one per instruction
rather than by one per card drawn.

*And it costs nothing measurable.* CPU/game median **16.15 → 16.23 ms, +0.5%**
(seven interleaved rounds, `--threads 1`; main 15.74–16.62, middle
15.92–17.03). **Seven rounds and not three, because three said +6.0% and the
three were wrong**: one `main` round came in at 20.32 ms against its own
15.97 median, and a 27% outlier in a three-round median is a 6% answer. The
rule that follows is worth more than the number — **when an arm's rounds
straddle another arm's, raise `--rounds` before writing the delta down**, and
`fuzz_ab.py --rounds 7 --no-fixtures` re-runs the timing block alone in about
70 seconds. RE-1 cost +5.0% for +447 gathers; +34 gathers costing +0.5% is the
same per-proposal price, which is the cross-check that says both numbers are
real.

| | performance (79 cards) | stress (123 cards) |
|---|---|---|
| P0 / P1 | 26 (52.0%) / 24 (48.0%) | 29 (58.0%) / 21 (42.0%) |
| Avg turns | 28.2 | 28.2 |
| Spells cast | 22.7 | 21.0 |
| Lands played | 17.7 | 17.4 |
| Combat w/ atk | 9.4 | 9.5 |
| Creatures died | 6.4 | 4.4 |
| Damage events | 20.3 | 19.9 |
| Total damage | 58.4 | 51.8 |
| Life changes | 13.4 | 13.6 |
| **Layer walks** | **362** | **469** |
| **Board walks** | **234** | **260** |
| **Memo hits** | **54,867** | **70,035** |
| **Layer frames** | **4,424** | **4,945** |
| **Frames/walk** | **12.22** | **10.55** |
| **Dependency checks** | **12** | **55** |
| **Replacement gathers** | **932** | **986** |
| **Restriction queries** | **935** | **988** |
| Prevention allocations | 0.02 | 0.02 |

**The shipped arm's −5.0% is the game getting shorter, not the engine getting
faster, and every absolute counter on it has to be read that way.** At 200
games `performance` goes 31.6 turns → 29.6, spells 23.6 → 22.8, and
`Replacement gathers` **1003 → 974 — down, on the arm that adds a card**. Per
turn it is the middle arm's: 31.7 for `main`, 32.8 for middle, 32.9 for the
shipped arm, so Thought Reflection's own contribution to the sweep is about a
tenth of a gather per turn and the 63-gather drop is two fewer turns. CPU/game
median 16.15 → 15.35 (**−5.0%**) is the same arithmetic; `CPU/turn p50` is
0.440 → 0.450 (**+2.3%**), which is what the slot actually costs. A card that
draws extra cards ends games sooner, and a per-game counter cannot tell that
from an engine that got cheaper — **so a pooled card that changes game length
is read per turn**, which no earlier pool addition had forced.

**Reachability.** `--require "Thought Reflection"` on `performance`, 200 games
/ seed 12345: cast 133, resolved 131, in **91 of 200 games (46%)**, copies/deck
1.58. Seven mana is the most any pooled card has cost and this is the number
that was measured rather than argued: Eon Hub reaches 64% at five. Zero errors
and zero panics. The other three were forced through `stress` the same way and
are recorded because two of them can loop if their encoding is wrong
(`replacement-architecture.md` §11 items 42 and 53): Alms Collector 134/134 in
98 games (49%), Teferi's Ageless Insight 144/143 in 99 (50%), Notion Thief
130/129 in 91 (46%) — zero errors and zero panics on each.

**The stress column is not an engine reading this time either**, for RE-1's
reason: `default_registry` grew by four, so the middle and shipped arms play
different decks from `main` on that pool and are byte-identical to each other.
`Avg turns` 29.1 → 29.0 and `Dependency checks` 33 → 46 are the new cards being
drawn, not the pipeline.

**Re-run at the PR's review (2026-09-12), and the `performance` column moved by
a hair — which is worth a paragraph, because the change was supposed to be
answer-preserving and in the sense that matters it was.** Suppressing CR 616.1's
prompt between two draw doublers (§11 item 55) changes no rules answer: the
total is the product either way. It changes one thing the fuzz harness can see —
`RandomDecisionProvider::pick_n` draws from its own `StdRng` on every prompt, so
a prompt that no longer happens is one fewer draw and that game's decision
stream shifts from there. At 200 games the aggregate rows are identical (turns
29.6, layer walks 371) and `Restriction queries` moves by **1**; at 50 the
smaller sample shows it, which is why the table above is the re-run. Stress is
unchanged to the digit, because no `stress` deck put two draw doublers on one
battlefield in these 50 games.

**The rule that generalizes**, and it applies to every suppression this codebase
has: *answer-preserving is not stream-preserving.* An A/B whose arms differ by a
prompt cannot be read as "byte-identical or the change is wrong" — the check is
that the **gameplay aggregates** hold and the counters move by less than a game.

**Re-recorded 2026-09-11 for RE-1** — `PERFORMANCE_POOL` +1 (Eon Hub,
77 → 78) and the stress pool +5 (114 → 119). **The first re-recording whose
middle arm is not free**, and the two halves are worth separating.

*The engine's share, `performance`, 200 games / seed 12345.* Every
seed-dependent gameplay row is **identical to `main`** — turns, spells, lands,
combats, deaths, damage events, total damage, life changes, wins — and so is
every layer row: walks 363, board walks 241, frames 4,262, frames/walk 11.73,
dependency checks 36, all unchanged. Three real behaviour changes moved none of
them, which is what the prediction said they would do: turn 1's untap step now
runs (on an empty battlefield, with a land-drop count already at zero),
CR 508.8's three combat steps no longer happen (they granted no priority and
ran no turn-based action when they did), and "until your next turn" expires at
the turn's begin rather than at the untap step's (the same instant while
nothing in either pool skips the untap step).

What moved is the proposals: **`Replacement gathers` 507 → 954** and
**`Restriction queries` 509 → 957**, both +447 per game — one turn, five phases
and eight steps per turn with no attackers, eleven with, at 30.2 turns a game.
§9's RE-1 bullet predicted "about sixteen per turn … ~+450 per game" and it
landed at +14.8. **`Memo hits` 58,262 → 59,191 (+1.6%) is the row the
prediction got wrong**, and the correction is worth keeping: `gather`'s fast
path stops the *walk*, not the *query*, and `is_prohibited` asks one per
proposal, which the memo answers. A flat `Layer walks` beside a moved
`Memo hits` is what "the gate held" looks like, not a contradiction.

*And it costs.* CPU/game median **12.92 → 13.57 ms, +5.0%** (five interleaved
rounds, `--threads 1`, cleanly separated: main 12.90–12.99, middle 13.55–13.66).
A fourth arm with a hard-coded event-kind gate in `gather` — the shape §8's
answer-preserving lever would have — measured **13.24, +2.5%**, which attributes
exactly half the cost to the sweep and half to the chokepoint itself: the batch,
the APNAP sort, `is_prohibited`, the grouping and the event. The second half is
not recoverable by any gate and is what the three begin events cost.
`replacement-architecture.md` §9 records the decision that followed.

| | performance (78 cards) | stress (119 cards) |
|---|---|---|
| P0 / P1 | 30 (60.0%) / 20 (40.0%) | 27 (54.0%) / 23 (46.0%) |
| Avg turns | 30.1 | 26.8 |
| Spells cast | 22.9 | 20.6 |
| Lands played | 17.9 | 16.3 |
| Combat w/ atk | 10.1 | 8.3 |
| Creatures died | 6.9 | 3.6 |
| Damage events | 20.9 | 17.2 |
| Total damage | 59.2 | 47.5 |
| Life changes | 14.3 | 12.5 |
| **Layer walks** | **351** | **429** |
| **Board walks** | **233** | **242** |
| **Memo hits** | **58,082** | **58,549** |
| **Layer frames** | **4,163** | **4,249** |
| **Frames/walk** | **11.85** | **9.91** |
| **Dependency checks** | **14** | **21** |
| **Replacement gathers** | **949** | **868** |
| **Restriction queries** | **951** | **871** |
| Prevention allocations | 0.02 | 0.00 |

At 200 games the shipped arm is `performance` walks 363 → 375 and gathers
954 → 1003 — Eon Hub opens the per-permanent sweep on every begin proposal for
as long as it is on the battlefield, which is the first pooled card whose
static watches something the *turn machinery* proposes rather than something a
spell does. CPU/game median 12.99 → 14.80 (+13.9% against `main`, +7.9% against
the middle arm), which is the ordinary price of a slot: §3.1a measured Bone
Splinters at +13.8% and rejected Altar's Reap at +20.2%. Zero errors and zero
panics on every arm and pool; three shell runs at one seed identical outside
the timing lines.

**Reachability.** `--require "Eon Hub"` on `performance`, 200 games / seed
12345: cast 211, resolved 209, in **129 of 200 games (64%)**, copies/deck
1.57 — so the two-copy board is common, and CR 616.1's prompt between two
applicable skips on one upkeep is answered by the random agent in a measured
game rather than only in a fixture. Zero errors and zero panics on that run.
Unforced, the shipped arm's `Replacement gathers` is 1003 against the middle
arm's 954, which is the card reaching the battlefield without being pushed.

**The stress column is not an engine reading this time, and saying so is the
point.** `--pool stress` draws from `default_registry`, which RE-1 grew by
five, so its decks are different in the middle arm as well as the shipped one
— Yawgmoth's Bargain gives the random agent a use for its life total and it
decks itself, which takes `Avg turns` 31.2 → 29.1 and `Total damage` 60.9 →
48.0. That is the card doing exactly what §9 predicted and why it is
registered and not pooled; the engine delta is the `performance` column, where
the decks are byte-identical.

**Re-recorded 2026-09-09 for RD-3** — `PERFORMANCE_POOL` +1 (Guardian Seraph,
75 → 76) and the stress pool +8 (102 → 110). **The engine's share is zero, and
this is the cleanest reading the protocol has produced:** the middle arm —
RD-3's engine with `registry.rs` and both pools unchanged — is *byte-identical*
to `main` outside `=== Timing ===` at 200 games on both pools, so every
movement below is the pool's. A widened `EventPattern` arm costs nothing until
a def writes the new fields (`replacement-architecture.md` §11 item 32).

| | performance (76 cards) | stress (110 cards) |
|---|---|---|
| P0 / P1 | 23 (46.0%) / 27 (54.0%) | 29 (58.0%) / 21 (42.0%) |
| Avg turns | 31.1 | 32.1 |
| Spells cast | 23.8 | 23.4 |
| Lands played | 18.4 | 18.8 |
| Combat w/ atk | 10.4 | 9.5 |
| Creatures died | 7.7 | 5.4 |
| Damage events | 21.3 | 19.9 |
| Total damage | 62.4 | 55.0 |
| Life changes | 14.4 | 12.5 |
| **Layer walks** | **374** | **514** |
| **Board walks** | **242** | **284** |
| **Memo hits** | **61,239** | **73,967** |
| **Layer frames** | **4,427** | **5,262** |
| **Frames/walk** | **11.84** | **10.23** |
| **Dependency checks** | **29** | **78** |
| **Replacement gathers** | **534** | **565** |
| **Restriction queries** | **536** | **568** |
| Prevention allocations | 0.02 | 0.00 |

At 200 games the shipped arm is `performance` walks 368 → 361 and gathers
506 → 508 — a source-side `ObjectFilter` on every damage event is free at this
board size — against `stress` walks 437 → 496 and gathers 513 → 563, which is
eight cards in every deck. CPU/game median 14.10 → 14.15 ms (+0.4%); zero
errors and zero panics on every arm and pool; three shell runs at one seed
identical outside the timing lines.

**Re-recorded 2026-09-09 for RD-2** — `PERFORMANCE_POOL` +1 (Mending Hands,
74 → 75) and the stress pool +4 (98 → 102). The table gains a row,
**Prevention allocations**, which `plans/fuzz_ab.py` prints from here on: CR
615.7's allocation prompts per game — a *reachability* count and not a cost,
which is why it is not bold, and `?` on any binary older than RD-2. Every
other movement below is the pool's: with the registry and pools unchanged,
RD-2's engine (the `loop` arm) reproduces RD-1's 50-game `performance` table
to the digit — 372 walks, 4,602 frames, 518 gathers — and the item-29
suppression moves it by one game's worth (373 / 4,612 / 519); on `stress`
both differ by a hair in the two-Furnace games (448 → 446 → 448 walks). The
shipped columns are a different game set, because a one-mana instant in every
white deck reshuffles every deck; read them as RD-1's were read.

| | performance (75 cards) | stress (102 cards) |
|---|---|---|
| P0 / P1 | 26 (52.0%) / 24 (48.0%) | 26 (52.0%) / 24 (48.0%) |
| Avg turns | 32.3 | 26.8 |
| Spells cast | 23.9 | 21.2 |
| Lands played | 18.7 | 16.4 |
| Combat w/ atk | 10.8 | 8.4 |
| Creatures died | 7.8 | 4.9 |
| Damage events | 22.6 | 17.5 |
| Total damage | 65.4 | 52.4 |
| Life changes | 14.4 | 12.5 |
| **Layer walks** | **399** | **419** |
| **Board walks** | **258** | **237** |
| **Memo hits** | **67,128** | **50,792** |
| **Layer frames** | **4,717** | **4,023** |
| **Frames/walk** | **11.82** | **9.59** |
| **Dependency checks** | **16** | **18** |
| **Replacement gathers** | **561** | **445** |
| **Restriction queries** | **562** | **448** |
| Prevention allocations | 0.00 | 0.00 |

The engine's share, by the four-arm protocol (`replacement-architecture.md`
§9, RD-2 as landed): gathers and walks flat on both middle arms; the middle
arms differ from `main` outside the timing block in three and four of 200
`performance` games, every one with two Furnaces of Rath on the battlefield —
one CR 616.1 prompt where a two-member batch had two, then none where the
suppression removes it — and in nothing else; CPU/game −0.8% and −0.9% on the
middle arms, +2.1% shipped. Reachability is the `Prevention allocations` row
and the `--require` counts in that section: 0.02 per `stress` game unforced,
0.05 with the three unpooled cards forced.

**Re-recorded 2026-09-08 for RD-1** — `PERFORMANCE_POOL` +1 (Furnace of Rath,
73 → 74) and the stress pool +5. Every row moved, and that is what a damage
doubler in every red deck does: games end sooner (avg turns 28.9 → 30.0 on
`performance` is the *other* direction and is deck-mix noise at 50 games; the
200-game run has 31.0 → 29.7), total damage per game rises 59.4 → 63.8, and
CPU/game falls **6.4%** because there is less game to play.

| | performance (74 cards) | stress (98 cards) |
|---|---|---|
| P0 / P1 | 27 (54.0%) / 23 (46.0%) | 20 (40.0%) / 30 (60.0%) |
| Avg turns | 30.0 | 27.2 |
| Spells cast | 23.5 | 21.0 |
| Lands played | 17.9 | 16.7 |
| Combat w/ atk | 9.7 | 9.2 |
| Creatures died | 7.0 | 4.1 |
| Damage events | 20.8 | 19.8 |
| Total damage | 63.8 | 56.3 |
| Life changes | 14.4 | 13.3 |
| **Layer walks** | **372** | **448** |
| **Board walks** | **245** | **245** |
| **Memo hits** | **60,807** | **56,268** |
| **Layer frames** | **4,602** | **4,374** |
| **Frames/walk** | **12.36** | **9.77** |
| **Dependency checks** | **22** | **18** |
| **Replacement gathers** | **518** | **480** |
| **Restriction queries** | **520** | **483** |

The `stress` column moved by a hair between the first recording and this one
(449 → 448 walks, 4,414 → 4,374 frames) — the review's `Primitive::Mill` fix,
which turns a mill of N into one batch instead of N and so bumps the layer
epoch once rather than N times. Nothing a game can see changes;
`performance` is untouched because Angel of Suffering is not in it.

Reachability, 200 `stress` games with Loyalty Probe forced into every deck:
cast 206, resolved 204, **in 133 games (66%)**, 1.49 copies per deck. It is
`{2}` and colorless, which is why the number is that high and why the fixture
is worth registering: **CR 704.5i fires 4 times in 400 unforced `stress`
games** — three Probes bolted to zero, and one Merfolk Thaumaturgist that
Cytoshape turned into a copy of a Probe and which died on the spot, because
CR 707.2 does not copy counters. That state-based action had measured 0 at
every game count since it was written.

**Re-recorded 2026-09-08 for the CR 704.5p fix** (`codebase-state.md` item 82).
No card and no pool change — a bug fix and, riding with it, the largest engine
speed-up the project has measured. **Every row that moved is the engine's.**

The bug is one sentence of CR 704.5p that was never implemented (an Equipment
that *becomes* a creature stayed attached). The speed-up is unrelated to it and
was found while fixing it: both attachment sweeps asked their `has_subtype`
questions *before* reading `attached_to`, so three characteristics frames were
computed for every permanent on the battlefield, every state-based-action
check, to answer a question about the handful that were attached. Reading the
field first is the whole change.

| | main | fixed |
|---|---:|---:|
| Memo hits, `performance` (200 games) | 99,530 | **62,215** (−37.5%) |
| Memo hits, `stress` | 97,783 | **61,424** (−37.2%) |
| CPU/game median (200 games, ×3) | 16.09 ms | **14.29 ms** (−11.2%) |
| ms / 1,000 walks | 42.57 | 37.70 (−11.4%) |
| Layer walks / frames | 378 / 4,504 | 379 / 4,510 |

**Read `ms/1,000 queries` carefully here: it rises 41.8%, and that is the
speed-up rather than a regression.** Queries are walks plus memo hits, so a
37% fall in memo hits shrinks the denominator faster than the numerator falls.
This is the case §3.1 warns about — "fewer questions walked" is a different
finding from "the walk got slower" — and it is the first time the project has
produced it. CPU/game and memo hits are the rows to read.

One behavioural row moves, and it is the bug: total damage 56.6 → 56.5 per
`performance` game, which is an equipped Bonesplitter detaching under March of
the Machines and no longer granting +2/+0. Zero errors and zero panics on both
pools; three shell runs at one seed identical outside the timing lines.

| | performance (73 cards) | stress (94 cards) |
|---|---|---|
| P0 / P1 | 28 (56.0%) / 22 (44.0%) | 25 (50.0%) / 25 (50.0%) |
| Avg turns | 28.9 | 28.2 |
| Spells cast | 22.3 | 22.6 |
| Lands played | 17.5 | 17.0 |
| Combat w/ atk | 9.9 | 10.1 |
| Creatures died | 6.6 | 4.5 |
| Damage events | 21.7 | 22.7 |
| Total damage | 59.3 | 58.6 |
| Life changes | 14.3 | 14.8 |
| **Layer walks** | **371** | **434** |
| **Board walks** | **235** | **251** |
| **Memo hits** | **55,610** | **57,618** |
| **Layer frames** | **4,267** | **4,547** |
| **Frames/walk** | **11.50** | **10.48** |
| **Dependency checks** | **10** | **42** |
| **Replacement gathers** | **475** | **485** |
| **Restriction queries** | **477** | **487** |

**Re-recorded 2026-09-08 for CM-3** (lock-in's payment side;
`cost-architecture.md`). One new card in `performance` — **Bone Splinters**,
72 → 73 — and six in `stress` (Bone Splinters, Altar's Reap, Thunderscape
Familiar, Krark-Clan Ironworks, Foundry Inspector, Mind Stone; 88 → 94).

**`cost-architecture.md` §6 said CM-3 "opens no new path a pooled card would
measure". The A/B says the first half and disproves the second.** The middle
arm — CM-3's engine with all five cards registered and the *old* pool — is
`IDENTICAL` to `main` on `performance` at 200 games, so the payment order,
the plan/pay split, the mandatory-cost announcement and the castability gate
cost the pool nothing and change no seeded stream. But `Cost::Sacrifice`
*is* a new engine path, and no card in the 72 could reach it: that is the
`PERFORMANCE_POOL` doc's own failure mode, "a gated subsystem no card in the
pool could open", and its rule is that a phase which opens a path adds one
card deliberately. So the pool gains one and §6's second clause was wrong.

The pooled card opens four things at once — the first non-mana cost paid
through `pay_costs`, the first mandatory additional cost (CR 118.8b), the
first castability answer that turns on something other than the mana cost,
and the first payment prompt that is not an allocation. The other five stay
registered and out: the Familiar and the Inspector open the path Thalia
already opens, and the Ironworks pair's window is CM-4's to measure once the
window stops closing early.

**Which card is a separate question from which path, and it was measured.**
Altar's Reap was pooled first, because CR 601.2h's example is written on it.
It costs **+20.2%** CPU/game, and it draws two cards, so the games it makes
are bigger. Bone Splinters opens the identical set of paths, sacrifices the
same way, and destroys a creature instead of drawing two: **+13.8%** in the
same sitting, +11.9% in a second. Altar's Reap stays registered — its tests
are the rule's — and the pool carries Bone Splinters.

### 3.1a What a pooled card costs is mostly the *slot*, measured 2026-09-08

The obvious reading of the paragraph above is that Altar's Reap's card draw is
the cost and a leaner card avoids it. **Half right, and the other half is the
more useful number.** A third arm settles it: `Cobbled Wings` — already
registered, opening *no new engine path at all*, since Bonesplitter is pooled
and equip is the same code — as the 73rd card instead.

| 73rd card | opens a new path | CPU/game vs `main` | ms / 1,000 queries |
|---|---|---:|---:|
| *(none — 72)* | — | +0.0% | 0.154 |
| Cobbled Wings (inert control) | no | **+14.0%** | 0.165 |
| Bone Splinters | yes | **+11.9%** | 0.158 |
| Altar's Reap | yes | +20.2% (prior sitting) | 0.161 |

**A card that does nothing new costs as much as Bone Splinters does.** So
~12–14% is what *a pool slot* costs, not what a mechanic costs: a 73rd
playable card changes deck composition, boards get bigger, and the layer walk
covers more objects per walk (frames 3,998 → 4,542 for the inert control,
which introduces no rows of its own). Only the surcharge above that line is
attributable to a card, and Altar's Reap's ~6 points is one; Bone Splinters is
indistinguishable from the control.

Three things follow, and they are what to quote the next time this comes up:

- **Choosing a leaner card is worth doing and worth about 6 points.** Between
  two cards that open the same path, take the cheaper board. Past that there is
  nothing to optimise: the slot is the cost.
- **This is not an engine regression and no engine work removes it.**
  ms/1,000 queries — the cost of a unit of work rather than of a game — moves
  2.8% for Bone Splinters and 7.2% for a card with no mechanic, both inside
  the sitting's 2–6% spread. The games got bigger; nothing got slower. Every
  CM phase's middle arm has been byte-identical to `main`.
- **The growth is the price of representativeness, and it is bounded by how
  often a phase opens a genuinely new path** — three times across CM-1, CM-2
  and CM-3. A pool that stopped growing would go back to measuring a shrinking
  fraction of the engine, which is the failure the freeze was lifted for.

**And the A/B sitting is not the development bottleneck it feels like**:
three arms, both pools, counters, fixture rows and three interleaved timing
rounds is **32 seconds** of wall clock (2026-09-08). What costs minutes is
building one release binary per arm and orchestrating them, which is
`--rounds`-independent. If a sitting ever does need to be cheaper, `--rounds 2`
or `--games 100` halves the timing block at the price of a wider spread; that
knob is there and has not been needed.

Reachability, 200 games with Bone Splinters forced into every `performance`
deck: cast 193, resolved 110, in 84 games (42%), **1.58 copies per deck**.
Casts and not resolutions is the number that matters here — the sacrifice is
paid at CR 601.2h whether or not the spell later resolves — and 193 against
Altar's Reap's 223 is 87% of the payment-path exercise for two-thirds of the
cost. Zero errors, zero panics and zero `Uncast resolved` in every arm on both
pools, which is the statement that matters most for a new payment arm.

| | performance (73 cards) | stress (94 cards) |
|---|---|---|
| P0 / P1 | 28 (56.0%) / 22 (44.0%) | 26 (52.0%) / 24 (48.0%) |
| Avg turns | 28.9 | 28.1 |
| Spells cast | 22.3 | 22.5 |
| Lands played | 17.5 | 16.9 |
| Combat w/ atk | 9.9 | 10.1 |
| Creatures died | 6.6 | 4.5 |
| Damage events | 21.7 | 22.7 |
| Total damage | 59.4 | 58.8 |
| Life changes | 14.3 | 14.8 |
| **Layer walks** | **371** | **431** |
| **Board walks** | **235** | **248** |
| **Memo hits** | **88,901** | **89,949** |
| **Layer frames** | **4,265** | **4,445** |
| **Frames/walk** | **11.49** | **10.32** |
| **Dependency checks** | **10** | **38** |
| **Replacement gathers** | **475** | **481** |
| **Restriction queries** | **477** | **483** |

**Re-recorded 2026-09-07 for CM-2** (the spell's own cost abilities;
`cost-architecture.md`). One new card in `performance` — Myr Enforcer, 71 → 72
— and two in `stress` (Myr Enforcer and Frogmite; 86 → 88). The middle arm —
CM-2's engine, both cards registered, the *old* `PERFORMANCE_POOL` — is
`IDENTICAL` to `main` on `performance` outside the timing block, so every
movement below is the pool's.

**It was not identical on the first run, and that is the whole reason the arm
exists.** +5 layer walks and +5 layer frames per 200 games, with every gameplay
counter unchanged — five non-member frames computed and thrown away. The cause:
source 2's gate asked "does this card print a cost ability", which is true of a
**Thalia in hand**, so her frame was computed at every castability preview and
then refused because her subject is other spells. The gate now asks the
subject. Nobody would have found five walks by reading the diff, and nobody
needed to.

Timing, 200 games, three interleaved rounds: CPU/game 16.63 ms `main` → 16.50
ms registered (−0.8%, inside the sitting's ~2–6% spread) → 15.01 ms pooled;
`deterministic` yes in all three arms, and three shell runs at one seed match
line for line on both pools. **The pooled column is a re-record, not a
speed-up**: never A/B a number across a pool change.

Reachability, 200 games: Myr Enforcer cast 167, resolved 166, in 113
`performance` games (56%), **1.74 copies per deck** — so the board where one
Enforcer counts the one already on the battlefield is routine rather than
contrived. `stress` with Trinisphere forced beside it: 158 / 157 in 108 games
(54%) and 193 / 192 in 136 games (68%), ~1.5 copies per deck each, which is
affinity's reduction and a direct-total effect meeting on one spell.

| | performance (72 cards) | stress (88 cards) |
|---|---|---|
| P0 / P1 | 27 (54.0%) / 23 (46.0%) | 24 (48.0%) / 26 (52.0%) |
| Avg turns | 28.4 | 30.0 |
| Spells cast | 21.5 | 22.9 |
| Lands played | 17.3 | 17.7 |
| Combat w/ atk | 9.3 | 10.5 |
| Creatures died | 6.3 | 4.2 |
| Damage events | 20.3 | 23.7 |
| Total damage | 47.6 | 59.6 |
| Life changes | 13.5 | 15.2 |
| **Layer walks** | **352** | **433** |
| **Board walks** | **220** | **254** |
| **Memo hits** | **80,262** | **99,787** |
| **Layer frames** | **3,602** | **4,734** |
| **Frames/walk** | **10.24** | **10.94** |
| **Dependency checks** | **11** | **73** |
| **Replacement gathers** | **449** | **527** |
| **Restriction queries** | **451** | **530** |

**Re-recorded 2026-09-07 for CM-1** (cost modification; `cost-architecture.md`).
One new card in `performance` — Thalia, Guardian of Thraben, 70 → 71 — and
three in `stress` (Thalia, Goblin Electromancer, Trinisphere; 83 → 86). **Every
movement below is the pool's, and this time that was checked by construction
rather than argued**: a third arm, the CM-1 engine with the three cards
registered but the *old* `PERFORMANCE_POOL`, reproduces `main`'s 200-game
`performance` run byte for byte outside the timing block (`fuzz_ab.py`:
`unpooled vs main outside Timing: IDENTICAL`), because a cost pipeline gated on
an empty source set is not there; on `stress` that arm and the pooled arm are
identical to each other, so pooling Thalia moved `performance` and nothing else.
Timing, 200 games, three interleaved rounds: CPU/game 15.67 → 15.70 ms
(+0.2%), ms per 1,000 walks 46.2 → 43.0, deterministic in all three arms.

Reachability, 200 games (`--require`, which since this phase re-joins a card
name that contains its own separator — the flag split "Thalia, Guardian of
Thraben" into two names nobody had registered): Thalia cast 210, resolved 210,
in 131 `performance` games (66%), 1.66 copies per deck; with Humility forced
beside her, Thalia in 138 games (69%) and Humility in 105 (52%), which is the
stripped-source path in a measured game. `stress`, Goblin Electromancer and
Trinisphere forced: 180 cast / 178 resolved in 126 games (63%) and 193 / 191 in
131 games (66%), ~1.5 copies per deck each.

| | performance (71 cards) | stress (86 cards) |
|---|---|---|
| P0 / P1 | 32 (64.0%) / 18 (36.0%) | 26 (52.0%) / 24 (48.0%) |
| Avg turns | 29.3 | 31.4 |
| Spells cast | 22.1 | 23.7 |
| Lands played | 17.3 | 18.4 |
| Combat w/ atk | 10.4 | 11.1 |
| Creatures died | 6.4 | 5.6 |
| Damage events | 21.4 | 25.3 |
| Total damage | 56.1 | 67.4 |
| Life changes | 14.7 | 15.8 |
| **Layer walks** | **363** | **467** |
| **Board walks** | **235** | **270** |
| **Memo hits** | **91,298** | **105,142** |
| **Layer frames** | **4,253** | **5,021** |
| **Frames/walk** | **11.70** | **10.76** |
| **Dependency checks** | **14** | **114** |
| **Replacement gathers** | **479** | **549** |
| **Restriction queries** | **482** | **552** |

**Re-recorded 2026-09-06 for LI-3** (conditional statics;
`layers-architecture.md` §13b). One new card in `performance` — Kird Ape,
69 → 70 — and none in `stress`, and no new row. **Every movement below is
the pool's**: with the registry and pools unchanged, LI-3's engine
reproduces LI-2's table byte for byte on both pools, at 50 games and at
200 (the middle arm of the three-arm table below), because no *registered*
card's static ability had a condition before Kird Ape and the new branch
in the existence check therefore never ran.

| | performance (70 cards) | stress (81 cards) |
|---|---|---|
| P0 / P1 | 28 (56.0%) / 22 (44.0%) | 27 (54.0%) / 23 (46.0%) |
| Avg turns | 29.3 | 30.3 |
| Spells cast | 22.9 | 23.2 |
| Lands played | 17.7 | 17.7 |
| Combat w/ atk | 10.0 | 9.8 |
| Creatures died | 6.8 | 5.1 |
| Damage events | 22.5 | 22.3 |
| Total damage | 57.2 | 57.8 |
| Life changes | 15.1 | 14.5 |
| **Layer walks** | **321** | **361** |
| **Board walks** | **232** | **269** |
| **Memo hits** | **87,860** | **102,836** |
| **Layer frames** | **4,074** | **4,975** |
| **Frames/walk** | **12.70** | **13.79** |
| **Dependency checks** | **21** | **44** |
| **Replacement gathers** | **476** | **522** |
| **Restriction queries** | **479** | **524** |

**Three arms again (2026-09-06, LI-3).** `plans/fuzz_ab.py`, one sitting:
`main` at 9011d42 (A), LI-3's engine with the registry and both pools
unchanged (B), and LI-3 as shipped (C).

| | A: main | B: engine, pools unchanged | C: LI-3 |
|---|---|---|---|
| performance / stress, 200 games, outside `=== Timing ===` | — | **identical** | differs — pool |
| performance layer walks / board walks (50 games) | 334 / 241 | 334 / 241 | 321 / 232 |
| performance frames, frames/walk (50 games) | 4,296, 12.85 | 4,296, 12.85 | 4,074, 12.70 |
| performance dependency checks (50 games) | 23 | 23 | 21 |
| performance CPU/game median (200 games, ×3) | 15.72 ms | 15.89 ms (+1.1%) | 16.01 ms (+1.8%) |
| performance ms / 1,000 questions (walks + hits) | 0.163 | 0.164 (+1.1%) | 0.162 (−0.2%) |
| performance CPU/game p99 median | 45.36 ms | 45.68 ms | 70.31 ms |
| stress dependency checks (200 games) | 87 | 87 | 72 |

**B is identical to A this time, not "identical the new row aside".**
Every counter, every behavioural row and every cost row matches at 50 and
at 200 games on both pools; the three serial timing rounds are identical
line for line outside `=== Timing ===`; and on 40-game `--dump-events`
streams with the id masks applied, the whole stream is byte-identical on
both pools — 0 of 40 games differ. That is what a conditional-existence
clause predicts for a pool with no conditional card: `Effect::Conditional`
is an arm no registered ability's body reaches, so `condition::holds` is
never called and `condition_reads` adds no channel. The claim the A/B can
make is therefore narrow and exact — **the change is inert until a
conditional card is in the pool** — and every row that moves in C is Kird
Ape's.

Then C. 40 of 40 games differ on each pool, the first divergence in
`performance` being event 18, `Keldon Warlord ... Library -> Hand` against
`Kird Ape ... Library -> Hand`: the registry's sorted name list grew by
one, so `random_deck` draws different cards from the same seeded stream.
Nothing in those diffs is the engine's. The p99 column is the one to read
carefully — 45.36 → 70.31 ms is a *different set of games*, not a slower
engine, since B's p99 sits on A's; CPU/game median moves +1.8%, inside the
sitting's spread, and ms per 1,000 questions is flat to slightly down.

**Re-recorded 2026-09-06 for LI-2** (CR 613.8a/b/c, the dependency loop;
`layers-architecture.md` §13b). One new card in `performance` — Urborg,
Tomb of Yawgmoth, 68 → 69 — and three in `stress` — Urborg, Opalescence,
Ashaya, Soul of the Wild, 78 → 81 — and a new bold row, **`Dependency
checks`**: the CR 613.8a hypotheticals a game ran, which are the pairs the
static channel check could not settle. **Every movement below is the
pool's**: with the registry and pools unchanged, LI-2's engine reproduces
LI-1's table row for row on both pools (the middle arm of the three-arm
table), so what moved is Urborg dropping into `performance` decks and, in
`stress`, three new names in the list `random_deck` draws from — every
stress game's deck changed, and that column is not comparable to LI-1's
beyond the fact that the engine did not move it.

| | performance (69 cards) | stress (81 cards) |
|---|---|---|
| P0 / P1 | 26 (52.0%) / 24 (48.0%) | 30 (60.0%) / 20 (40.0%) |
| Avg turns | 31.2 | 28.5 |
| Spells cast | 23.9 | 21.7 |
| Lands played | 18.4 | 17.4 |
| Combat w/ atk | 11.0 | 9.6 |
| Creatures died | 7.2 | 4.1 |
| Damage events | 23.7 | 20.8 |
| Total damage | 58.0 | 54.6 |
| Life changes | 16.6 | 14.1 |
| **Layer walks** | **334** | **334** |
| **Board walks** | **241** | **247** |
| **Memo hits** | **95,297** | **93,311** |
| **Layer frames** | **4,296** | **4,452** |
| **Frames/walk** | **12.85** | **13.34** |
| **Dependency checks** | **23** | **63** |
| **Replacement gathers** | **513** | **483** |
| **Restriction queries** | **516** | **486** |

**Three arms, and the middle one is `main` (2026-09-06, LI-2).**
`plans/fuzz_ab.py`, one sitting: `main` at a6f2ed8 (A), LI-2's engine with
the registry and pools unchanged (B), and LI-2 as shipped (C).

| | A: main | B: engine, pools unchanged | C: LI-2 |
|---|---|---|---|
| performance, 200 games, outside `=== Timing ===` | — | identical (the new row aside) | differs — pool |
| performance layer walks / board walks (50 games) | 328 / 236 | 328 / 236 | 334 / 241 |
| performance frames, frames/walk (50 games) | 4,289, 13.08 | 4,289, 13.08 | 4,296, 12.85 |
| performance dependency checks (50 games) | — | 13 | 23 |
| performance CPU/game median (200 games, ×3) | 15.42 ms | 15.89 ms (+3.0%) | 15.72 ms (+1.9%) |
| performance ms / 1,000 questions (walks + hits) | 0.155 | 0.160 (+3.0%) | 0.163 (+4.8%) |
| performance CPU/game p99 median | 48.12 ms | 49.25 ms | 45.45 ms |
| stress, 200 games | — | identical (the new row aside) | differs — pool |
| stress dependency checks (200 games) | — | 7 | 87 |

Read B first, because it is the finding. **The engine change alone changes
no game in either pool**: every counter and every behavioural row matches
A at 50 and at 200 games, the three serial timing rounds are identical
line for line outside `=== Timing ===`, and on 40-game `--dump-events`
streams with the id masks applied, 0 of 40 games differ on `performance`
and 0 of 40 on `stress`. That is CR 613.8's prediction for this pool: its
only dependency-shaped pairs are Humility beside a creature's static
ability, and the dependency's answer there was already timestamp order's
in both directions (LI-1's flipped pin). B's 8 hypotheticals per game at
200 games are exactly those pairs, each confirming a dependency that
changes nothing. (`fuzz_ab.py` prints "differ" for B against A because the
new counter row is a new line; the raw outputs under `--out` minus that
line are what "identical" means here.)

Then C. 29 of 40 `performance` games and 40 of 40 `stress` games differ
from A, every one first at a `Library -> Hand` draw event: the registry's
sorted name list changed, so `random_deck` draws different cards from the
same seeded stream. Nothing in those diffs is the engine's. The cost rows
move with the pool too — `Dependency checks` 13 → 23 on `performance` is
Blood Moon meeting Urborg — and CPU/game is flat inside the sitting's
spread: B +3.0% and C +1.9% against A, with round 3 reading B *faster*
than A. The loop's fast path is why: a layer whose applications are
pairwise independent under the channel check costs N² bit-ands and no
hypothetical, which is every layer of every board that has no
dependency-shaped card.

**How to read `Dependency checks` from now on.** It is the CR 613.8 loop's
slow path — pairs the static check could not settle, each a frame clone
per member the other application reaches, applied and taken back. Zero
means every pair in every layer was settled statically. Read it beside
`Board walks`: checks per board walk is how many pairs per pass reached
the expensive half, and a rise against unchanged board walks means a
dependency-shaped card started meeting another more often.

**Re-recorded 2026-09-06 for LI-1** (the board-wide sequential pass;
`layers-architecture.md` §13b). No new card and no pool change — the
consumer, Humility beside Citanul Hierophants, was already in both pools —
so for once **every row is the engine's**, and the table gains a row,
`Board walks`: the layer walks that computed the whole working set at once.
Since LI-1 a miss for a permanent walks the whole board and fills the memo
for every member, so `Layer walks` is the number of *boards* computed plus
the walks of objects no row can reach, and `Frames/walk` reads near the
board's size.

| | performance (68 cards) | stress (78 cards) |
|---|---|---|
| P0 / P1 | 29 (58.0%) / 21 (42.0%) | 31 (62.0%) / 19 (38.0%) |
| Avg turns | 30.7 | 33.1 |
| Spells cast | 23.4 | 24.3 |
| Lands played | 18.3 | 18.9 |
| Combat w/ atk | 10.1 | 11.6 |
| Creatures died | 6.6 | 4.0 |
| Damage events | 21.8 | 25.6 |
| Total damage | 53.6 | 60.1 |
| Life changes | 15.7 | 17.4 |
| **Layer walks** | **328** | **382** |
| **Board walks** | **236** | **286** |
| **Memo hits** | **95,005** | **122,108** |
| **Layer frames** | **4,289** | **5,682** |
| **Frames/walk** | **13.08** | **14.88** |
| **Replacement gathers** | **505** | **602** |
| **Restriction queries** | **507** | **604** |

**Two arms, and the second legitimately differs from `main` (2026-09-06,
LI-1).** `plans/fuzz_ab.py`, one sitting: `main` at 650633f (A) and LI-1
(B). There is no "engine, pool unchanged" arm distinct from the shipped one,
because the pool did not change; and B is not A's stream, because the pooled
board's answer did — a creature under Humility no longer taps for the
Hierophants' {G}.

| | A: main | B: LI-1 |
|---|---|---|
| performance, 200 games, outside `=== Timing ===` | — | differs |
| performance layer walks / board walks (50 games) | 2,425 / — | 328 / 236 |
| performance frames, frames/walk (50 games) | 3,755, 1.55 | 4,289, 13.08 |
| performance CPU/game median (200 games, ×3) | 14.56 ms | 14.81 ms (+1.7%) |
| performance ms / 1,000 questions (walks + hits) | 0.146 | 0.149 (+2.1%) |
| performance CPU/game p99 median | 48.28 ms | 46.58 ms |
| stress, 200 games | — | differs |

Read the divergence first, because it is the finding. On 40-game
`--dump-events` streams with the id masks applied, **one game in forty
differs on each pool** — `performance` game 38, `stress` game 20 — and in
both, Humility and Citanul Hierophants had entered the battlefield before
the first divergent event, which is a choice made by index (a mana source
tapped for a payment; a blocker) from a list in which a creature under
Humility no longer offers the granted ability. Every other game is
byte-identical to `main`'s. The behavioural rows move by that one game per
pool: `performance` by a tenth of a turn, `stress` by more because its game
20 diverged early (event 521 of 814) and ran to 1,297 events.

Then the cost rows, which are the pass's shape rather than its price. Layer
walks fell 7× because one board walk fills the memo for every member where
each member used to miss on its own; frames rose 14% because a pass builds one frame
per member where a walk built 1.55; the two together are the +1.7% of
CPU/game and the +2.1% per question, both inside the sitting's spread
(round 1 read +0.5%, round 2 +4.7%, round 3 +1.7%). The look-ahead's cost —
a pass per entry where it was one walk — is in there and did not show;
`layers-architecture.md` §13b names the two answer-preserving levers if a
later board makes it show.

**The `main` arm has to be built at the commit the worktree is synced to.**
The first sitting compared against a `fuzz_games.exe` built the day before
#102's last commits, and every game differed from its first draw because the
decks did. `git worktree list` says where the source is; only a
`cargo build --release --bin fuzz_games` in that worktree says where the
binary is.

**How to read `Board walks` from now on.** A layer walk is a board walk or
the walk of an object no row can reach (`board::membership`); board walks ×
the working set is the bulk of `Layer frames`. A board-walk count that rises
against unchanged layer walks means the engine started asking about
permanents at more epochs; frames rising against unchanged board walks means
the boards got bigger.

**Re-recorded 2026-09-06 for LH-2** (CR 613.7e and Equip;
`layers-architecture.md` §13a). Two new cards: Bonesplitter in both pools —
the first Equipment, the first `Primitive::Attach` / `GameAction::Attach`,
and the first activation restriction — and Cobbled Wings in `stress` only,
since it opens no path Bonesplitter does not. That moves `performance` from
67 cards to 68 and `stress` from 76 to 78. **The rows are the pool's and a
bugfix's, not the walk's**: the four arms below say which.

| | performance (68 cards) | stress (78 cards) |
|---|---|---|
| P0 / P1 | 29 (58.0%) / 21 (42.0%) | 31 (62.0%) / 19 (38.0%) |
| Avg turns | 30.8 | 32.3 |
| Spells cast | 23.5 | 23.7 |
| Lands played | 18.3 | 18.6 |
| Combat w/ atk | 10.1 | 11.4 |
| Creatures died | 6.6 | 4.0 |
| Damage events | 21.8 | 25.1 |
| Total damage | 53.6 | 59.1 |
| Life changes | 15.7 | 17.2 |
| **Layer walks** | **2,425** | **3,384** |
| **Memo hits** | **93,215** | **113,452** |
| **Layer frames** | **3,755** | **4,629** |
| **Frames/walk** | **1.55** | **1.37** |
| **Replacement gathers** | **506** | **579** |
| **Restriction queries** | **508** | **581** |

**Four arms this time, because the consumer found a bug (2026-09-06,
LH-2).** `plans/fuzz_ab.py`, one sitting: `main` (A); LH-2's engine with the
`activate_ability` allocation fix reverted and both new cards unregistered
(B, "clean"); the engine as shipped, both registered and Bonesplitter not
pooled (B′); and LH-2 shipped (C). B is the "engine, pool unchanged" arm the
protocol asks for. B′ exists because the fix — an activated ability with a
generic pip had never been payable, so it was blacklisted at every
activation — changes what Chainbreaker, a pooled card, does in a game.

| | A: main | B: clean engine | B′: engine + fix | C: shipped |
|---|---|---|---|---|
| performance, 200 games, outside `=== Timing ===` | — | **A's, but `Memo hits` +4** | differs | differs |
| performance walks (50 games) | 2,421 | 2,421 | 2,543 | 2,425 |
| performance frames/walk | 1.44 | 1.44 | 1.43 | 1.55 |
| performance CPU/game median (200 games, ×3) | 13.16 ms | 13.23 ms (+0.5%) | 13.65 ms (+3.7%) | 14.92 ms (+13.4%) |
| performance ms / 1,000 walks | 5.454 | 5.483 (+0.5%) | 5.695 (+4.4%) | 5.637 (+3.4%) |
| stress, 200 games | — | A's, but `Memo hits` +3 | **identical to C** | — |

Read B against A first. Its event stream *is* `main`'s — 40-game dumps
identical after the id masks, and the sweeps keying on a timestamp CR 613.7e
now reassigns moved no pick in 200 games — and the extra memo hits are the
new `can_pay_costs` pre-check in `activate_ability` asking cached questions.
The walk itself is `main`'s code: a re-stamp at attach time
(`retime_static_rows`) costs one registry pass per equip, not a per-frame
sort, and B's +0.5% is inside the sitting's own noise (round 2 had B faster
than A). B′ against B is Chainbreaker's ability resolving at all (85 times
per 40 games, from 0): more stack, more resolutions. C against B′ is the
card: an Equipment on the battlefield in 140 of 200 games, whose `Host` row
walks its source for the existence check. That is the frames/walk
1.43 → 1.55 and the +13.4% of CPU/game, of which +3.4% is per walk — the
shape LH-1 measured for Holy Strength at 1.39 → 1.44, on a card cast about
three times as often because it is a colorless {1}. The first version of
this PR read the rows' timestamps live in the walk instead, and the same
board then cost +9.5% per 1,000 walks; §13a records why that shape was
replaced.

**Re-recorded 2026-09-04 for LH-1** (the Aura host becomes addressable;
`layers-architecture.md` §13a). One new card, Holy Strength, in both pools —
the first `ObjectSet::Host` row, and the first spell whose target
is its enchant ability rather than a spell ability — which moves `performance`
from 66 cards to 67 and `stress` from 75 to 76. **Every row here is the pool's
and none is the engine's**, measured the way RC-5 measured it, below.

| | performance (67 cards) | stress (76 cards) |
|---|---|---|
| P0 / P1 | 30 (60.0%) / 20 (40.0%) | 32 (64.0%) / 18 (36.0%) |
| Avg turns | 31.9 | 28.3 |
| Spells cast | 24.6 | 21.1 |
| Lands played | 19.0 | 17.1 |
| Combat w/ atk | 11.7 | 10.1 |
| Creatures died | 7.7 | 3.4 |
| Damage events | 26.0 | 22.3 |
| Total damage | 66.2 | 56.1 |
| Life changes | 18.0 | 15.0 |
| **Layer walks** | **2,421** | **1,962** |
| **Memo hits** | **94,202** | **75,607** |
| **Layer frames** | **3,496** | **2,697** |
| **Frames/walk** | **1.44** | **1.37** |
| **Replacement gathers** | **527** | **452** |
| **Restriction queries** | **530** | **454** |

**The engine's share is zero again, by the same three-arm protocol
(2026-09-04, LH-1).** `plans/fuzz_ab.py`, one sitting: `main` (A), LH-1's
engine with `PERFORMANCE_POOL` exactly as `main` had it (B), and LH-1 shipped
(C).

| | A: main | B: engine, pool unchanged | C: shipped |
|---|---|---|---|
| performance, 200 games, outside `=== Timing ===` | — | **byte-identical to A** | differs |
| performance walks (50 games) | 2,550 | 2,550 | 2,421 |
| performance frames/walk | 1.39 | 1.39 | 1.44 |
| performance CPU/game median (200 games, ×3) | 12.84 ms | 12.78 ms (−0.5%) | 12.15 ms (−5.4%) |
| performance ms / 1,000 walks | 5.116 | 5.092 (−0.5%) | 5.035 (−1.6%) |
| stress, 200 games | — | **identical to C** | — |

Read as RC-5's was read. B against A is byte-identical on `performance`, so
the new `effect_applies_to` arm, the `StackEntry` field and the recipient
helper cost the pool nothing when no card reaches them; B against C is
identical on `stress`, because a registered card is in that pool whether or
not it joined the measured one. **The shipped column's −5.4% is the game, not
the walk**: a +1/+2 Aura in roughly a third of decks ends games sooner (33.3 →
31.8 turns over 200 games), so fewer walks happen, while `ms / 1,000 walks`
moves −1.6% — inside the sitting's spread, and not a finding. `Frames/walk`
rises on `performance` (1.39 → 1.44) for the reason RC-5 gave for Master
Biomancer: a row scoped to one host puts a sub-frame under its source. And
`stress` loses a death a game (4.4 → 3.4) because a creature wearing +1/+2
survives combats it used to lose — the one behavioural row LH-1 moves on its
own.

*Previous values, 2026-09-03 (performance 66, stress 75 — RC-5; reproduced to
the digit by this sitting's `main` arm, which is the check that the re-record
is the cards and not the machine): performance 27/23, 34.5 turns, 24.9 spells,
20.0 lands, 12.0 combats, 7.9 deaths, 24.3 damage events, 54.7 damage, 16.3
life changes, 2,550 walks / 102,077 memo hits / 3,555 frames / 1.39 per walk /
567 gathers / 570 queries; stress 28/22, 30.6, 22.3, 18.2, 10.2, 4.4, 24.0,
56.9, 17.0, 2,214 / 87,597 / 3,116 / 1.41 / 504 / 506. RC-5's own three-arm
A/B read engine +0.7% and shipped +1.0% CPU/game against its `main`, B
byte-identical to A on `performance` and to C on `stress` — the same partition
this sitting reproduces. Before that, 2026-09-03 (performance 64, stress 72 —
before RC-5's three cards; reproduced to the digit by RC-5's `main` arm): performance 28/22, 34.6
turns, 26.0 spells, 20.3 lands, 13.1 combats, 8.2 deaths, 26.7 damage events,
58.4 damage, 18.4 life changes, 2,663 walks / 105,963 memo hits / 3,818 frames /
1.43 per walk / 584 gathers / 585 queries; stress 24/26, 34.4, 25.3, 19.7, 11.6,
3.5, 25.6, 54.4, 17.5, 2,719 / 108,699 / 3,612 / 1.33 / 599 / 600. Before that,
2026-09-03 (performance 64, stress 72 — the Everywhere pool,
mana base and agent, before the epoch memo; every row but the three cost rows
is unchanged by it, and `Memo hits` did not exist): performance 108,626 walks
/ 161,827 frames / 1.49 per walk / 584 gathers / 585 queries; stress 111,418 /
152,428 / 1.37 / 599 / 600 — reproduced to the digit on `main` at 90692f7 in
the same sitting. Before that, 2026-09-03 (performance 63 → 64, stress 71 → 72
— the same PR, before the random agent learned to tap for the pip it owes):
performance
30/20, 36.0 turns, 22.2 spells, 20.6 lands, 11.2 combats, 5.2 deaths, 22.3
damage events, 50.8 damage, 16.5 life changes, 108,423 walks / 158,203 frames /
1.46 per walk / 650 gathers / 651 queries; stress 31/19, 36.6, 21.7, 20.7,
11.0, 3.2, 22.2, 48.2, 17.9, 114,839 / 148,874 / 1.30 / 694 / 696. Before
that, 2026-09-02 (pools unchanged at 63 / 71; a cast whose payment fails now
rewinds instead of resolving unpaid, 16c): performance 25/25, 30.6
turns, 26.4 spells, 19.2 lands, 11.8 combats, 6.7 deaths, 24.9 damage events,
50.9 damage, 16.8 life changes, 104,622 walks / 146,052 frames / 1.40 per walk /
588 gathers / 590 queries; stress 25/25, 30.8, 25.1, 18.8, 11.8, 5.6, 25.2,
58.9, 16.9, 100,674 / 137,318 / 1.36 / 578 / 582 — reproduced to the digit on
`main` at 6dedaf8 before the Everywhere re-record. Before that, 2026-09-02
(performance 62 → 63, stress 68 → 71; Cytoshape,
Mirrorweave and Mirrorform, CV-1): performance 23/27, 28.8 turns, 21.0 spells,
18.4 lands, 11.0 combats, 6.7 deaths, 24.2 damage events, 49.8 damage, 16.8 life
changes, 93,914 walks / 136,338 frames / 1.45 per walk / 531 gathers / 533
queries; stress 27/23, 28.3, 19.6, 17.6, 10.5, 4.4, 23.4, 55.3, 15.7, 88,252 /
117,612 / 1.33 / 514 / 517 — reproduced to the digit on `main` at 650a263
before the 16c re-record. Before that, 2026-09-02 (performance 60 → 61, Root
Maze, RC-3): performance
23/27, 31.7 turns, 23.0 spells, 19.8 lands, 12.4 combats, 7.8 deaths, 25.7 damage
events, 52.2 damage, 17.0 life changes, 108,632 walks / 135,449 frames / 1.25
per walk / 669 gathers / 670 queries; stress 30/20, 29.8, 21.8, 18.1, 11.6, 4.5,
25.8, 58.8, 16.5, 98,843 / 118,256 / 1.20 / 618 / 618. Before that, 2026-09-01
(performance 59 → 60, Battlegrowth and Adaptive
Shimmerer): performance 25/25, 30.4 turns, 22.1 spells, 19.1 lands, 11.9
combats, 7.6 deaths, 25.0 damage events, 52.3 damage, 17.0 life changes, 99,877
walks / 123,802 frames / 1.24 per walk / 626 gathers / 627 queries; stress 26/24,
29.6, 21.4, 18.1, 11.1, 4.7, 24.7, 57.4, 15.5, 93,245 / 108,422 / 1.16 / 600 /
601. Before that (performance 57 → 59, RC-2): performance 25/25,
31.8 turns, 22.8 spells, 19.7 lands, 12.7 combats, 8.6 deaths, 26.7 damage
events, 53.3 damage, 17.6 life changes, 108,902 walks / 135,893 frames / 1.25
per walk / 669 gathers / 670 queries; stress 27/23, 30.6, 21.9, 18.6, 12.1, 4.8,
27.5, 60.5, 17.3, 101,929 / 127,209 / 1.25 / 648 / 649. Before that (performance
55 → 57, RS-1): performance 29/21, 28.4 turns, 21.0 spells, 18.1 lands, 11.2
combats, 5.9 deaths, 22.8 damage events, 49.1 damage, 16.6 life changes; stress
28/22, 28.8, 20.9, 17.9, 12.1, 3.8, 25.5, 57.0, 19.2.*
