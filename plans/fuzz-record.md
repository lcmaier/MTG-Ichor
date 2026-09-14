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
the first `AffectedSet::Host` row, and the first spell whose target
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
