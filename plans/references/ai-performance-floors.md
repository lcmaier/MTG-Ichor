# Item 42 and a bounded history gate two of three AI floors

**v1 should hold its AI use case to three floors. Only the first holds without the event-log PR, item 42, and the third also needs TR-2a's turn history bounded.** The floors are: at least **10,000 decisions per loaded physical core-second at Commander scale**, a full-state clone of at most **10 µs**, and at most **128 KB per game state**. The throughput floor is the rate at which the 12–14 physical cores that current 8-GPU nodes ship per GPU can keep one H100 busy serving a 10-million-parameter, 128-token policy. The engine's one-thread reading of 15,600 clears it. The loaded reading is 11,700–16,000 per physical core, depending on whether hardware threads count, and the pending tap-solver re-base (about ×0.68) would move it to 7,950–10,900. How the reading is defined therefore matters as much as the engine's speed. The clone and bytes floors come from search, not self-play. The card-game standard of 10,000 iterations per decision in about a second leaves 100 µs per iteration. A Commander decision step already costs 64 µs, and replaying a tree path from the root costs about four steps, so only state-per-node search with a cheap, compact clone fits. Item 143's clone table was re-taken on this branch with its own method. Without the log, the clone floor holds (3.7–8.8 µs). The bytes floor does not: the no-log state passes 128 KB late in three of five Commander-scale games and reaches 209 KB. TR-2a's whole-game history is all of the excess; with it emptied, the state peaks at 102 KB. With the log, the clone passes 10 µs by turn 20 and reaches 131 µs and 1,176 KB at the end of a 184-turn game. The nearest full-rules MTG engines run 40–300× slower per step and copy state about 1,000× more slowly, and none of them publishes a search rate near the standard. Every workload in the notes fits the engine at these floors, provided item 42 lands first and TR-2a's history is bounded. Those workloads run from 5e9 decisions per single-server run to 1e13 on the largest runs, with 2 to 10,000 forks per searched decision.

## One floor holds today; two wait on item 42

| Floor | Today (engine's readings) | Rests on | Instrument | Item 42 first? |
|---|---|---|---|---|
| **1. ≥ 10,000 decisions per loaded physical core-second, Commander scale** (≤ 100 µs of engine CPU per decision) | 15,600 on one thread (64 µs per decision). Loaded: 11,700 with one worker per physical core, ≈ 16,000 with all 16 threads; both come from a contention read that predates the last two levers. About ×0.68 once the tap solver lands | Feeding one H100 at 30% of dense BF16 (~116,000 passes/s of a 10M-parameter, 128-token policy) from the 12–14 physical cores per GPU of current 8-GPU nodes | **Exists**: `fuzz_games` counters, `fuzz_ab.py`, and the readiness pass (dated DPCS, the 1/8/16-thread contention read, callgrind). **Add**: callgrind instructions per decision, recorded beside the first reading | **No** |
| **2. Full-state clone ≤ 10 µs** at every checkpoint of a Commander-scale game through its end. Portable form: ≤ 1/6 of one decision's engine CPU. CI proxy: ≤ 64 allocations | Measured on this branch. Without the log: 3.7–8.8 µs (7.1 µs with the history emptied too), 19–50 allocations. With it: 8.8–11.6 µs by turn 20, up to 131 µs and 1,479 allocations at the end of a 184-turn game | 10,000 search iterations per decision in about 1 s (ISMCTS; MTG ensemble determinization) gives 100 µs per iteration, and a Commander step takes 64 µs of it. AlphaZero spends about 50 µs per simulation | **To build**: commit the counting-allocator clone probe; add a CI test on allocations per clone | **Yes** |
| **3. ≤ 128 KB per state** (deep size of a clone) at every checkpoint. A state's size must be bounded by its board's high-water mark, never by the turn count | Measured on this branch. Without the log: 36–209 KB, over 128 KB late in three of five Commander-scale games; TR-2a's history is 99–107 KB of the two long games' ends. With the history emptied: ≤ 102 KB. With the log: up to 1,176 KB | Stored-state search at 5,000–10,000 nodes within ~2 GB per hardware thread; 256 MB bot caps; archives of restorable states | **To build**: the same probe; a CI test on exact bytes per clone, with the hash seed pinned | **Yes**, plus a bound on TR-2a's whole-game history |

The split puts the owner's profile into concrete terms: item 42 ranks nowhere for straight-line throughput and first for forking. Model-free self-play (the DouZero, AlphaHoldem and ByteRL tier) steps each game forward and never copies it, so it needs floor 1 alone. AlphaZero-style training, ISMCTS and determinized rollouts either copy a state on every iteration or replay to one, so they also need floors 2 and 3. Those are the methods that most MTG AI projects in the notes use: Forge's simulation AI, XMage's minimax and MCTS players, Magarena, MageZero and the 2012 ensemble-determinization work.

## The big picture: fast enough, and what decides research-grade from here

**The engine is already the fastest full-rules MTG simulator by a wide margin.** It is 40–300× faster per step and, once the log is out of `GameState`, roughly 700–1,000× faster per clone. Its per-core rate sits in the same band as the NetHack Learning Environment (14,000–39,000 steps per core-second), which is serious RL research territory. Speed is no longer what decides whether it is research-grade. What decides it is keeping the headroom through the triggers phase and the tap solver, and making sure the research interface does not throw it away.

**Is it enough, by workload** [D, from the re-taken clone and the 15,600 reading]:

| Workload | What it needs | Where the engine is | Verdict |
|---|---|---|---|
| Model-free self-play (DouZero, AlphaHoldem, ByteRL scale: ~5e9 decisions on one server) | ~10,000 decisions per loaded core-second | 15,600 on one thread. A DouZero-sized run is about half a day of engine time on this 8-core desktop, about an hour on one 8-GPU server | Enough. A server's own cores keep its GPUs busy for any policy of 10M parameters or more |
| AlphaZero-style training (800 simulations per decision) | A clone that is small next to a 64 µs step | ~73 µs per stored-state simulation without the log. Over ~270 searched decisions, that is ~16 core-seconds per Commander game, against ~1.9 H100-seconds of evaluation at 10M × 128, so ~8 cores per GPU | Enough after item 42. With the log in, or by replaying from the root, 12–31 cores per GPU: up to ~2.5× what a server has |
| Card-game tree search (10,000 iterations per decision, the MTG literature's standard) | ~100 µs per iteration | ~0.73 s per decision on full rules, about what the 2012 paper's toy simulator (lands and vanilla creatures) took | Enough for play and evaluation. Training on it at millions of games is too heavy on any engine |
| Deck evaluation (2M–36M games per study) | Games per core-hour | ~44 ms per Commander game, so 36M games is ~440 core-hours | Far more than enough |

**What changes in the plan:**
1. **Item 42 moves up.** It was filed as "not urgent", but it gates both search floors. With the log leaving `GameState` entirely, it is a small PR. It should get a slot on the route before Phase 10's harness, rather than "whenever the first fork harness shows up".
2. **Bound TR-2a's `PlayerHistory` in the same PR.** This is the survey's one new engine finding. One PR that makes the state bounded by the board would switch floors 2 and 3 on:
   - the log out of `GameState`;
   - the history bounded to about five rows per player;
   - the committed clone probe;
   - a CI test on allocations per clone.
3. **Adopt floor 1 as the ratchet's backstop, and expect it to bind soon.** After the tap solver's re-base, the one-thread reading is ~10,600, 6% above the floor, and the triggers phase adds cost on top. The levers to spend then are all open in the latest profile. Each is 8–19% of instructions inclusive; they overlap and do not add:
   - the timestamp sorts (16.8%, ~30 lines);
   - the allocator (18.2% self, and up to a quarter to a third of a fully loaded machine);
   - the state-based-action sweep's per-permanent lookups (16.8%);
   - the pre-check for priority prompts with nothing to do (19.2%);
   - the frame's SipHash type sets (8.6 G, ~11%).

   The readiness pass at item 6's close re-takes the scaling read, which predates the two big levers, and records callgrind instructions per decision.

**What research-grade needs beyond speed, in order of risk:**
1. **The harness boundary (Phase 10).** It is the biggest risk to the throughput researchers actually get. EnvPool's numbers show the binding alone can cost 15×: Python subprocesses against an in-process thread pool. Item 141 found per-prompt serialization out of process costs more than the engine. The batched decision boundary, an in-process binding and suppressed forced prompts are requirements, not polish.
2. **Observation encoding.** Its cost relative to a step, k, is unmeasured, and floor 1 effectively scales by (1 + k).
3. **Search that does not cheat.** Today a search over a clone sees opponents' hands and the next draw. The fix is `backlog.md` §2.34's determinization. Search also needs item 140's mid-round fork entry and a way to search over targets and modes, which the priority-boundary fork model either folds into the action or leaves to a harness policy.
4. **Card breadth.** At 175 cards, whether real decks run will decide adoption more than microseconds will.

**Where the information model goes.** Today §2.9 is row B4. It can sit anywhere in A or B, with a hard back-stop before Phase 8's face-down and reveal cards and before any Phase 10 work. RE-8's close (2026-09-14) declined to pull it forward, and the trigger survey (2026-09-18) read no reason to. This survey adds three things:
- **k is floor 1's largest unmeasured factor.** k needs no information model to bound. A throwaway probe that builds a naive Commander observation (the effective characteristics of every public object, one hand, hidden-zone counts) over today's omniscient state, using `Zone::is_public()`, and times it per decision would measure it. The visibility mask adds little on top.
- **§2.34 found §2.9, as planned, insufficient for honest search.** A query says who may see an object. A search also needs a per-viewer knowledge record, written by reveal, look and scry, and a redeal from it.
- **That knowledge record is forked state.** It must fork with the game, so it lives on `GameState`, and floor 3 constrains its shape: per-viewer bits per object, bounded by the board, never a per-viewer event history. With the log piped out it cannot be derived from events later, so it has to be materialized at the chokepoint, as the history is.

**So the design moves up and the build does not.**
- Measure k with the probe now. If k ≪ 1, nothing else about the ordering changes.
- Merge §2.9 and §2.34 into one design (query, knowledge record, redeal). Give it a slot right after item 6's audit, ahead of Phase 8, in place of "anywhere before the back-stop". That follows the pattern mana provenance (B9) already set: a design pass the owner reviews first, then the build.
- The build keeps its back-stop. Its cost grows with every reveal, look and scry path written before it, which is the retrofit the back-stop exists to cap.

**What not to do:**
- **No GPU port.** GPU simulators exist only for fixed-shape games like chess and Go.
- **No make/unmake undo journal.** A 10 µs clone is ~14% of a step.
- **No chasing tiny-policy ratios.** A 1M-parameter policy needs extra CPU servers on any engine, and the literature ran 100–600 cores per GPU.

## Item 143, re-taken on this branch: the history is what grows

**Method.** The probe is a throwaway crate in the scratchpad that depends on this branch's `mtgsim` by path (3b1880e, tree clean). It follows item 143's method, recovered from that probe's session:
- Bytes and allocations are read from one held clone under a counting allocator.
- Time is the mean of 2,000 clone-and-drop iterations. Here it is the median of five such means.
- "No log" is the same state after `events.clear()`. A third variant also empties every player's `PlayerHistory`.
- A checkpoint is the state between two `run_turn` calls.

Decks are `fuzz_games`' `random_deck` verbatim. Every game was checked against `fuzz_games` at the same seed and flags, and all seven matched in length and winner.

The board is `fuzz_games`' `--deck-size 100 --life 40 --players 4`. Item 143's original rows used the old probe's own recipe (singleton nonlands, 37 lands), which item 138 records as unreproducible, so those are different games and are not compared row by row. Seeds 12345 and 777 now play games of 70 and 49 turns. The longest game of each pool's 100-game sweep at seed 12345 is added: `stress` game 7 (184 turns) and `performance` game 43 (168 turns).

**Commander scale** (µs is the median of five means; KB is 1,024 bytes):

| game | turn | objects | on bf | µs | KB | allocs | µs, no log | KB, no log | allocs, no log | µs, no log or history | KB, no log or history |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `stress` 12345 | 1 | 400 | 0 | 4.2 | 42.9 | 21 | 3.8 | 36.9 | 20 | 3.9 | 36.9 |
| | 40 | 400 | 45 | 23.1 | 240.2 | 247 | 6.8 | 95.5 | 50 | 6.0 | 66.7 |
| | 60 | 300 | 47 | 32.6 | 359.3 | 410 | 6.3 | **131.3** | 40 | 5.3 | 87.5 |
| | 70, over | 100 | 19 | 40.6 | 430.0 | 580 | 4.3 | **136.9** | 33 | 3.1 | 87.7 |
| `stress` 777 | 40 | 303 | 35 | 19.0 | 260.2 | 222 | 5.5 | 95.2 | 39 | 4.7 | 66.5 |
| | 49, over | 103 | 15 | 31.2 | 347.7 | 417 | 3.7 | 96.2 | 30 | 2.8 | 63.4 |
| `stress` game 7 (seed 12351) | 40 | 400 | 36 | 22.6 | 251.8 | 242 | 6.2 | 96.7 | 40 | 5.3 | 67.9 |
| | 80 | 300 | 31 | 43.4 | 464.1 | 511 | 6.5 | 126.9 | 43 | 4.9 | 71.2 |
| | 100 | 200 | 28 | 58.4 | 569.9 | 639 | 6.6 | **139.9** | 40 | 4.7 | 73.5 |
| | 150 | 200 | 41 | 103.2 | 865.0 | 1,040 | 8.6 | **159.4** | 48 | 6.9 | 74.0 |
| | 184, over | 100 | 47 | 130.6 | 1,175.9 | 1,479 | 7.7 | **194.5** | 45 | 4.8 | 95.8 |
| `performance` 12345 | 40 | 300 | 31 | 26.5 | 260.1 | 314 | 5.6 | 94.1 | 42 | 4.9 | 66.2 |
| | 64, over | 100 | 19 | 44.2 | 395.2 | 554 | 4.1 | 101.9 | 29 | 3.0 | 61.4 |
| `performance` game 43 (seed 12387) | 40 | 400 | 29 | 17.5 | 202.0 | 121 | 6.4 | 95.0 | 46 | 5.8 | 66.4 |
| | 80 | 400 | 48 | 26.4 | 343.7 | 199 | 7.6 | **128.7** | 49 | 6.4 | 69.5 |
| | 100 | 400 | 61 | 46.0 | 444.0 | 278 | **8.8** | **165.6** | 50 | 7.1 | 92.1 |
| | 150 | 201 | 55 | 82.6 | 707.6 | 743 | 8.4 | **198.1** | 43 | 5.7 | 98.3 |
| | 168, over | 101 | 44 | 102.6 | 834.9 | 1,022 | 7.3 | **209.0** | 34 | 4.7 | 101.9 |

**60 cards, `performance`, seed 12345** (item 143's join rows):

| seats | turn | objects | on bf | µs | KB | allocs | µs, no log | KB, no log | allocs, no log |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 4 | 40 | 180 | 31 | 19.0 | 236.8 | 218 | 4.7 | 91.9 | 39 |
| 4 | 49, over | 60 | 16 | 28.1 | 307.8 | 387 | 3.4 | 95.7 | 28 |
| 2 | 30 | 120 | 22 | 12.3 | 149.8 | 192 | 3.0 | 44.6 | 28 |
| 2 | 43, over | 120 | 32 | 21.3 | 222.8 | 282 | 3.8 | 64.1 | 35 |

**Where the no-log bytes go**, KB, from cloning each field alone under the same counter:

| state | objects map | history | battlefield map | stack, entries, exile, command | players' zones and counters | registries, source sets, trigger bookkeeping | crate-private rest (layer memo, …) | total |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| turn 1 (any game) | 28.5 | 0.0 | 0.0 | 0.0 | 4.3 | 2.3–3.9 | 0.8 | 36–38 |
| `stress` game 7, turn 100 | 28.5 | 66.4 | 23.2 | 10.6 | 3.0 | 1.8 | 6.3 | 139.9 |
| `stress` game 7, end (184) | 28.5 | 98.8 | 46.4 | 10.8 | 2.1 | 1.7 | 6.3 | 194.5 |
| `performance` game 43, end (168) | 28.5 | 107.2 | 46.3 | 10.5 | 2.2 | 1.9 | 12.5 | 209.0 |

What the re-take changes:
- **Floor 2 holds without the log, with less headroom than item 143 showed.** The worst median is 8.8 µs (`performance` game 43, turn 100), and one rep reached 9.3 µs (`stress` game 7, turn 150): 12% under the floor. With the history emptied too, the worst is 7.1 µs. With the log, the clone is already 8.8–11.6 µs at turn 20.
- **Floor 3 fails without the log, and the history is the whole excess.** The state passes 128 KB from turn 60, 80 and 100 of the three longer games and ends the two long ones at 195 and 209 KB. `PlayerHistory` is one 192-byte row per player per game turn, 523 and 568 rows there, which is 99 and 107 KB. With it emptied, no board passes 102 KB.
- **The other growing term is map capacity, bounded by the board's high-water mark.** The battlefield map keeps the bucket count of the largest board it has held: 23 KB up to about 56 permanents and 46 KB beyond. It stays at 46 KB as the board shrinks (46.1 KB with 19 permanents at the end of seed 12345's game). The objects map is a flat 28.5 KB for 400 objects.
- **The log grows about 5.3 KB a turn at four seats.** It is 981 of the 1,176 KB and 1,434 of the 1,479 allocations at the end of the 184-turn game.
- **Allocations are exact across hash seeds; bytes are not.** Under `MTGSIM_HASH_SEED` 1 through 8 (and unset), allocation counts were identical at every checkpoint of `stress` game 7. Bytes differed under seeds 3 and 5, where the battlefield map doubled one checkpoint earlier (+11.5 KB at turn 30).
- **`GameState` is 2,064 bytes inline**, up from item 143's 1,536. The turn-1 clone is smaller than item 143's, 36–38 KB against 46 KB, which is A4g's 8-byte ids.

## The nearest MTG engines trail by 40–300× per step and ~1,000× per copy

[D] marks my own arithmetic on cited figures. "Engine's readings" means MTG-Ichor's own measurements on the reference machine (Ryzen 7 7700X, 8 cores and 16 threads, 31.5 GB).

| System | Unit counted | Per-core rate | Core-µs per unit | Clone or copy cost | Memory per instance | Kind | Hardware |
|---|---|---|---|---|---|---|---|
| **MTG-Ichor** (Rust; engine's readings) | decision with ≥ 2 options | 15,600 (Commander); 20,400 (60-card) | 64; 49 | 3.7–8.8 µs without log; 17–43 µs at turns 40–80 and up to 131 µs at game end with it | 36–209 KB without log (history up to 107 KB of it); up to 1,176 KB with it | env only, random agents | Ryzen 7 7700X, 1 thread |
| **Forge** (Java) ([ADR-0002](https://github.com/Tyrathalis/anvil/blob/main/docs/decisions/ADR-0002-fork-api-gate-resolution.md), [census](https://github.com/Tyrathalis/anvil/blob/main/docs/design/callback-census-results.md)) | AI callback, including one-option prompts and AI thinking time | ~199 at 1 worker; ~52 per worker at 16 workers [D] | ~5,000; ~19,000 loaded [D] | GameCopier: median 6 ms, p99 9 ms | 2 GB fixed heap per worker; ~2.3 GB allocated per game | end to end, heuristic AI | Ryzen 9 7950X; 1v1 Commander precons |
| **XMage** via MageZero (Java) ([README](https://github.com/WillWroble/MageZero), [setup](https://github.com/WillWroble/MageZero/blob/main/setup_guide.md), [FAQ](https://github.com/WillWroble/MageZero/blob/main/faq_goals.md)) | MCTS simulation, including network wait | ~150 per thread | ≤ 6,700 [D] | unpublished; deep copy of the whole game per node ([GameImpl.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/game/GameImpl.java)) | `-Xmx24g` for ≤ 6 games ≈ 4 GB each [D]; server default `-Xmx1024m` ([startServer.sh](https://github.com/magefree/mage/blob/master/Mage.Server/release/startServer.sh)) | end to end, search | 4 GHz; 2-player 60-card |
| **argentum** (Kotlin, immutable state) ([training-data](https://github.com/wingedsheep/argentum-engine/blob/main/docs/ai/training-data.md), [perf](https://github.com/wingedsheep/argentum-engine/blob/main/backlog/engine-performance.md)) | random action including enumeration; AI decision | ~367 per CPU-s [D]; AI decision mean 2.0 ms | ~2,700; 2,000 | "O(1) fork" claimed; 1,448 quiet-state simulations per thread-s | 812.6 KiB allocated per AI decision; 344 MiB peak heap | env only (random); end to end (AI) | Apple M1 Pro, 8 threads |
| **Magarena** MCTS (Java) ([#235](https://github.com/magarena/magarena/issues/235)) | iteration: full copy plus random playout to game end | ~13–48 per physical core [D] | ~21,000–74,000 | at most one iteration | unpublished | search | 2015 laptop, 4C/8T, 2.2 GHz |
| **Toy MTG** (C#; lands and vanilla creatures only) ([Cowling 2012](https://eprints.whiterose.ac.uk/id/eprint/75050/1/EnsDetMagic.pdf)) | full-game rollout | ~8,900–31,200 per CPU-s [D] | ~32–112 | unpublished | unpublished | search, including tree | Xeon X5460 |
| **SabberStone** (C#) ([changelog](https://github.com/HearthSim/SabberStone/wiki/Changelog)) | random game, `Options()` + `Process()` only | ~3,560 games [D], an upper bound; **conflicts** with ~200 games/s ([Świechowski](https://arxiv.org/abs/1808.04794)) | ~281 per game | ≈ 10.4× one `Process`, ≈ 6.2× one random step; no µs published | ≥ ~11 KB from tag tables alone [D from code] ([EntityData.cs](https://github.com/HearthSim/SabberStone/blob/master/SabberStoneCore/src/Model/Entities/EntityData.cs)) | env only | Ryzen 5 2600, 1 thread |
| **Pokémon**: Showdown / PokaiEngine / libpkmn ([PokaiTrainer](https://arxiv.org/abs/2608.29197), [pkmn](https://github.com/pkmn/engine/blob/main/docs/TESTING.md)) | VGC turn; Gen-1 battle | ~500 / ~12,500 turns; ~51,000 battles (libpkmn) | 2,000 / 80; 19.5 per battle | ~2 ms per sampled Showdown turn including restore; ~81 µs for PokaiEngine [D] | unpublished | env only | 1 Node process; EPYC 7B12 |
| **Tales of Tribute** (C#) ([arXiv](https://arxiv.org/abs/2305.08234)) | random game | 118–400 [D] | 2,500–8,500 | unpublished | "negligible" | env only | i5-9300H / i7-12700H, 1 thread |
| **sts_lightspeed** (C++) ([README](https://github.com/gamerpuppy/sts_lightspeed)) | random playout (combat or full run: unstated) | 12,500 per thread [D] | 80 | unpublished | unpublished | env only (claim) | 16 threads, CPU unstated |
| **RLCard** (Python) ([arXiv](https://arxiv.org/abs/1910.04376)) | player action, including forced moves | ~714 (Dou Dizhu) to ~12,500 (Leduc), 1 process | 80–1,400 | `step_back` snapshots, never benchmarked | unpublished | env only, random | Xeon Silver 4116 |
| **Hanabi LE** ([arXiv](https://arxiv.org/abs/1902.00506)) | turn | ~10,000 [D] | ~100 | unpublished | unpublished | env only | "a CPU" |
| **OpenSpiel** (C++/Python) ([spiel.h](https://github.com/google-deepmind/open_spiel/blob/master/open_spiel/spiel.h), [mcts.cc](https://github.com/google-deepmind/open_spiel/blob/master/open_spiel/algorithms/mcts.cc), [Pgx](https://arxiv.org/html/2303.17503)) | — | unpublished. Via Python on a 256-core DGX-A100: ≤ 1/10 of one A100 running Pgx | — | virtual deep copy; MCTS clones once per simulation | unpublished | — | — |
| **Pgx** (JAX) ([arXiv](https://arxiv.org/html/2303.17503), [chess](https://github.com/sotetsuk/pgx/blob/main/pgx/_src/games/chess.py), [Go](https://github.com/sotetsuk/pgx/blob/main/pgx/_src/games/go.py)) | vectorized state transition | ≥ 1e5 per A100 for the slowest game | ~10 GPU-µs [D] | none needed (immutable) | chess ~46 KB (30 KB of it the observation), 19×19 Go ~25 KB [D from source] | env only | A100, batch 1024 |
| **EnvPool** (C++) ([README](https://github.com/sail-sg/envpool/blob/main/README.md), [arXiv](https://arxiv.org/abs/2206.10558)) | Atari agent step = 4 frames | ~1,045 per thread and ~2,090 per physical core (DGX); 3,156 for a single env | ~317–957 | unpublished | unpublished | env only | DGX-A100 128C/256T; Ryzen 9 5950X |
| **NLE** (C) ([arXiv](https://arxiv.org/abs/2006.13760), [PufferLib](https://arxiv.org/html/2406.12905)) | keypress-level action | 14,400 (2020); 29,000–39,000 (2024, one core) | 26–69 | no clone facility found | unpublished; a private `.so` copy per instance | env only | i7 2.9 GHz laptop; i9-14900K |

**Units flatter every comparator.** No source counts only decisions with two or more options. Forge's 1v1 Commander precon game makes a mean of 776 AI callbacks, 437 of them priority picks ([census](https://github.com/Tyrathalis/anvil/blob/main/docs/design/callback-census-results.md)). A four-seat Commander-scale game here makes 692 filtered decisions. RLCard and NetHack also count forced moves. Forge and XMage publish only end-to-end rates with the AI's thinking time included. That makes argentum's ~367 random actions per CPU-second the cleanest full-rules comparator, and it is 42× below the Commander reading. Every MTG figure is two-player except one pathological Forge four-player token game, which ran 39 turns in 102.2 s ([PR #11314](https://github.com/Card-Forge/forge/pull/11314)).

**Contention costs the comparators more than this engine.** Forge's per-worker rate falls from ~199 to ~52 callbacks per second between 1 and 16 workers, a 26% per-worker efficiency. The Anvil authors blame boost clocks, the split L3 cache and ~2.3 GB of allocation per game ([ADR-0002](https://github.com/Tyrathalis/anvil/blob/main/docs/decisions/ADR-0002-fork-api-gate-resolution.md)). MageZero's search is "limited by heavy heap usage" at 8 threads ([README](https://github.com/WillWroble/MageZero)). The engine's own contention read, by contrast, kept 75% efficiency at 8 workers (engine's readings).

**Three figures conflict.** SabberStone's release notes imply about 3,560 random games per core-second, while Świechowski et al. report about 200 games per second on a high-end PC, 18× lower. The release-note figure times only `Options()` and `Process()` on one thread with logging off, so it is an upper bound. The 200 comes with no method, version or thread count. Neither figure is per decision. Forge's copy cost is a 6 ms median over 2,000 forks in ADR-0002, but 11–30 ms in Anvil's earlier smoke test ([fork-fidelity test](https://github.com/Tyrathalis/anvil/blob/main/docs/design/fork-fidelity-test.md)). MageZero's "~75 sims/s" at 8 threads does not say whether it is per thread or in total.

**Relative to each engine's own step, clone cost ranks the same on any hardware.** SabberStone's clone costs about 6.2 random steps [D], and Forge's copy about 1.2 of its own callbacks [D: 6 ms ÷ 5.0 ms]. This engine's clone costs at most 0.14 of a decision without the log (8.8 ÷ 64). With the log it costs 0.4–0.7 of a decision at turn 80 and up to 2.0 at the end of a 184-turn game (130.6 µs against 64 µs), so the late-game clone is then relatively costlier than Forge's. For memory, nobody publishes bytes per live state. Without the log, the engine's 36–209 KB (36–102 KB with the history emptied) sits at the scale of a Pgx chess state (~46 KB), while the Java MTG engines budget gigabytes of heap per game.

## Research workloads want billions of decisions and up to 10,000 forks each

### Self-play volume spans 1e6 to 1e13 decisions per run

| Tier | Run | Volume (every action counted) | Wall clock and hardware | End-to-end rate per core-s | Engine CPU at 10,000/core-s (time on one DGX H100's 112 cores) |
|---|---|---|---|---|---|
| MTG, academic | Causal-RL MTG benchmark ([arXiv 2605.06066](https://arxiv.org/abs/2605.06066)) | ≥ 1e6 steps per opponent per seed, 7 seeds | 5e5 steps ≈ 2 h; RTX 3090 + 12-core Ryzen 9 5900X | ~5.8 [D] | 100 core-s per 1e6 steps |
| MTG, hobby | MageZero ([README](https://github.com/WillWroble/MageZero), [FAQ](https://github.com/WillWroble/MageZero/blob/main/faq_goals.md)) | 1,000-game generations; ~20K games "enough for most decks" | ~250 games/h on 13 threads, 300-simulation MCTS | 0.0053 games per thread-s [D] | bound by search (next table) |
| Single server | DouZero ([arXiv](https://arxiv.org/abs/2106.06135)) | ~5e9 timesteps | ~10 days; 48 CPUs + 4× 1080 Ti | 60–121 [D] | 139 core-h (1.2 h) |
| Single server | AlphaHoldem ([AAAI](https://cdn.aaai.org/ojs/20394/20394-13-24407-1-2-20220628.pdf)) | 6.55e9 samples (~2.7e9 hands) | 3 days; 64 cores + 8× TITAN V | 395 [D] | 182 core-h (1.6 h) |
| Mid | ByteRL, LOCM ([arXiv](https://arxiv.org/abs/2303.04096)) | ~1.3e11 observations [D] | ~72 h; 24 V100 + 14,400 cores | ~35 [D] | 3,600 core-h (32 h) |
| Mid | Ataraxos, Stratego ([arXiv](https://arxiv.org/abs/2511.07312)) | 2.08e11 steps; 163M games | 1 week; 16 H100 with a GPU simulator, under $8,000 | GPU-simulated | 5,800 core-h (52 h) |
| Flagship | DeepNash ([arXiv](https://arxiv.org/abs/2206.15378), [Sokota App. J](https://arxiv.org/abs/2511.07312)) | ~5.5e9 games [D] | 2–3 months; 1,024 TPU nodes | not stated | — |
| Flagship | OpenAI Five ([arXiv](https://arxiv.org/abs/1912.06680), [2019 post](https://web.archive.org/web/2019id_/https://openai.com/blog/openai-five-defeats-dota-2-world-champions/), [2018 post](https://web.archive.org/web/2018id_/https://blog.openai.com/openai-five/)) | ~1.07e13 timesteps [D] | 10 months; 128,000 cores + 256 P100 (2018 configuration) | 3.9 [D] | 34 core-years (110 days) |
| Perfect-information search | AlphaZero chess ([arXiv](https://arxiv.org/abs/1712.01815)); KataGo ([arXiv](https://arxiv.org/abs/1902.10565)) | 44M games; 4.2M games | 9 h on 5,000 TPUv1; 19 days on ~27 V100 | — | bound by search |

Every run in the table counts every agent action, forced ones included. Treating one of its steps as one engine decision is therefore a unit conversion, not an equivalence. At 692 decisions per Commander game, the single-server tier is about 7–10 million games [D]; at MageZero's ~250 games per hour, that would take 3–4 years [D].

At floor 1, one DGX H100's 112 cores cover the engine side of each tier quickly: under two hours for the single-server tier, one to two days for the ByteRL and Ataraxos tier, and under four months for OpenAI Five's volume. Engine CPU volume limits none of these runs. The published end-to-end rates of 4–400 steps per core-second sit 25–2,500× below the floor because inference, search and learning share the same cores. Hanabi's environment, for example, runs about 10,000 turns per core-second on its own but only ~100–190 end to end ([Hanabi LE](https://arxiv.org/abs/1902.00506), [SAD](https://arxiv.org/abs/1912.02288)).

### Search turns one decision into 2 to 10,000 forks

The engine inputs come from the engine's readings: 64 µs per Commander decision step, and the re-taken clone, at most 8.8 µs without the log or 43.4–130.6 µs from turn 80 of the 184-turn game to its end with it. Two assumptions are mine. Replay uses ISMCTS's mean node depth of 4.1, the only depth figure in the notes ([ISMCTS](https://eprints.whiterose.ac.uk/id/eprint/75048/1/CowlingPowleyWhitehouse2012.pdf)). Playouts assume half of a 692-decision game remains. A stored-state iteration costs one clone plus one step; a replay iteration costs one clone plus 4.1 steps.

| Budget per decision (source) | Forks | Store states, no log | Store states, log | Replay from root, no log |
|---|---|---|---|---|
| Gumbel search, 2–32 simulations ([thesis](https://discovery.ucl.ac.uk/id/eprint/10167022/2/ivo_danihelka_thesis.pdf)) | 2–32 | 0.15–2.3 ms | 0.21–6.2 ms | 0.54–8.7 ms |
| KataGo, 225–400 expected visits ([arXiv](https://arxiv.org/abs/1902.10565)) | 225–400 | 16–29 ms | 24–78 ms | 61–108 ms |
| AlphaZero/MuZero board games, 800 ([AlphaZero](https://arxiv.org/abs/1712.01815), [MuZero](https://arxiv.org/abs/1911.08265)) | 800 | 58 ms | 86–156 ms | 217 ms |
| XMage minimax cap, 5,000 nodes ([ComputerPlayer6](https://github.com/magefree/mage/blob/master/Mage.Server.Plugins/Mage.Player.AI.MAD/src/mage/player/ai/ComputerPlayer6.java)) | 5,000 | 0.36 s | 0.54–0.97 s | 1.4 s |
| ISMCTS / MTG 2012 budget, 10,000, truncated with leaf evaluation ([ISMCTS](https://eprints.whiterose.ac.uk/id/eprint/75048/1/CowlingPowleyWhitehouse2012.pdf), [Cowling 2012](https://eprints.whiterose.ac.uk/id/eprint/75050/1/EnsDetMagic.pdf)) | 10,000 | 0.73 s | 1.1–1.9 s | 2.7 s |
| The same budget with playouts to game end (40 × 250) | 10,000 + 40 roots | ~220 s | ~220 s | ~220 s |

Three shapes recur. **Learned-model search** (MuZero, EfficientZero, Gumbel) forks nothing: it steps a latent state stored in each node ([MuZero](https://arxiv.org/abs/1911.08265), [mctx](https://github.com/google-deepmind/mctx/blob/main/mctx/_src/tree.py)).

**Real-simulator AlphaZero-style search** re-simulates from one root copy per simulation. OpenSpiel's MCTS clones the state on every simulation ([mcts.cc](https://github.com/google-deepmind/open_spiel/blob/master/open_spiel/algorithms/mcts.cc)), and KataGo resets each playout to the root board ([search.cpp](https://github.com/lightvector/KataGo/blob/master/cpp/search/search.cpp)). That is cheap when a step is a board update and expensive when a step is a 64 µs rules resolution.

**Card-game determinized search** runs 10,000 iterations, classically with playouts to game end, which would cost about 220 core-seconds per Commander decision. Modern card-game agents therefore truncate: at the end of the turn with a learned evaluator ([Świechowski](https://arxiv.org/abs/1808.04794)), at a depth threshold ([Zhang & Buro](https://skatgame.net/mburo/ps/cig17-hsai.pdf)), or after 2–3 Diplomacy phases ([SearchBot](https://arxiv.org/abs/2010.02923)). Once playouts are truncated, the clone becomes the recurring engine cost.

The MTG engines' own AIs already store a full game per node. XMage keeps a `Game` in every MCTS node ([MCTSNode.java](https://github.com/magefree/mage/blob/master/Mage.Server.Plugins/Mage.Player.AIMCTS/src/mage/player/ai/MCTSNode.java)). Forge copies the game once per evaluated option, to depth 3 ([SimulationController.java](https://github.com/Card-Forge/forge/blob/master/forge-ai/src/main/java/forge/ai/simulation/SimulationController.java)). Both pay about 6 ms per copy. Poker and Diplomacy search work differently: they iterate over an explicit subgame, with 100–1,000 CFR iterations in poker ([ReBeL](https://arxiv.org/abs/2007.13544)) and 256–4,096 regret-matching iterations in Diplomacy ([SearchBot](https://arxiv.org/abs/2010.02923)).

Competition time limits translate directly into stored-state iterations without the log [D]:

| Limit (source) | Iterations without the log |
|---|---|
| Hanabi: 40 ms per decision ([Goodman](https://arxiv.org/abs/1902.06075)) | ~550 |
| LOCM: 100–200 ms per turn ([referee](https://github.com/acatai/Strategy-Card-Game-AI-Competition/blob/master/referee1.2-java/src/main/java/com/codingame/game/engine/Constants.java)) | 1,400–2,700 |
| Botzone: 1 s ([wiki](https://wiki.botzone.org.cn/index.php?title=Bot/en)) | ~13,700 (5,100 late in a long game with the log) |
| Hearthstone competition: 30 s per turn ([rules](https://hearthstoneai.github.io/rules.html)) | ~410,000 |

The Hearthstone limit is disputed: the 2019 introductory paper says 60 s ([arXiv 1906.04238](https://arxiv.org/abs/1906.04238)).

### Fast environments need a dozen cores per GPU, not hundreds

| System | Inference runs on | CPUs per accelerator | End-to-end agent steps per core-s |
|---|---|---|---|
| Ape-X ([arXiv](https://arxiv.org/abs/1803.00933)) | CPU actors | 376 per P100 | ~35 |
| IMPALA, distributed ([arXiv](https://arxiv.org/abs/1802.01561)) | CPU actors | 150–500 per P100 | ~125 |
| SEED RL ([arXiv](https://arxiv.org/abs/1910.06591)) | TPU | 27–208 per TPU v3 core | ~144–305 |
| Sample Factory ([arXiv](https://arxiv.org/abs/2006.11751)) | GPU | 36 per RTX 2080 Ti | ~290–1,020 |
| moolib default / Cleanba ([arXiv](https://arxiv.org/abs/2310.00036)) | GPU | 10 / ~6 per A100 | n/a |
| DouZero ([arXiv](https://arxiv.org/abs/2106.06135)) | GPU | 12 logical CPUs per 1080 Ti | 60–121 |
| OpenAI Five ([arXiv](https://arxiv.org/abs/1912.06680)) | forward-pass GPUs | ~100–160 per forward-pass GPU | 3.9 |
| AlphaStar ([paper](https://storage.googleapis.com/deepmind-media/research/alphastar/AlphaStar_unformatted.pdf)) | TPU v3 | ~262 physical cores per 8-core device | ~6 |
| JueWu 2019 / 2020 ([1v1](https://arxiv.org/abs/1912.09729), [5v5](https://arxiv.org/abs/2011.12692)) | CPU / GPU | 375 per P40 / ~109 per GPU | n/a |
| Tencent Hearthstone ([arXiv](https://arxiv.org/abs/2303.05197)) | not stated | 244 per V100 | n/a |
| ByteRL, LOCM ([arXiv](https://arxiv.org/abs/2303.04096)) | not stated | 600 per V100 | ~35 |

Current 8-GPU nodes ship **12–14 physical cores per GPU**:

| Node | Physical cores per GPU |
|---|---|
| DGX H100 / H200 / B200 ([DGX H100](https://docs.nvidia.com/dgx/dgxh100-user-guide/introduction-to-dgxh100.html), [DGX B200](https://docs.nvidia.com/dgx/dgxb200-user-guide/introduction-to-dgxb200.html)) | 14 |
| AWS p5 family (96 cores, 192 vCPU) ([AWS](https://docs.aws.amazon.com/ec2/latest/instancetypes/ac.html)) | 12 |
| Azure ND H100 v5 ([Azure](https://learn.microsoft.com/en-us/azure/virtual-machines/sizes/gpu-accelerated/ndh100v5-series)) | 12 |
| GCP A3/A4 (208–224 vCPU) ([GCP](https://cloud.google.com/compute/docs/accelerator-optimized-machines)) | ~13–14 |
| DGX A100 ([DGX A100](https://docs.nvidia.com/dgx/dgxa100-user-guide/introduction-to-dgxa100.html)) | 16 |
| AWS p4d and GCP A2 | 6 |

For comparison, single-GPU cloud shapes offer 12–26 vCPUs ([GCP GPUs](https://cloud.google.com/compute/docs/gpus)) and a high-end desktop has 16 cores ([AMD](https://www.amd.com/en/products/processors/desktops/ryzen/9000-series/amd-ryzen-9-9950x.html)).

**Derived cores per H100, inference only, at U = 30%** [D]. The ">14" column marks rows the node's own cores cannot feed at 15,600.

| Parameters × tokens | FLOPs per pass | Passes/s per H100 | Cores/GPU at 20,400 | at 15,600 | at 10,000 (floor) | > 14 at 15,600? |
|---|---|---|---|---|---|---|
| 1M × 64 | 1.28e8 | 2,319,000 | 113.7 | 148.7 | 231.9 | yes (10.6×) |
| 1M × 128 | 2.56e8 | 1,160,000 | 56.8 | 74.3 | 116.0 | yes (5.3×) |
| 1M × 512 | 1.02e9 | 289,900 | 14.2 | 18.6 | 29.0 | yes (1.33×) |
| 10M × 64 | 1.28e9 | 231,900 | 11.4 | 14.9 | 23.2 | marginal (1.06×) |
| 10M × 128 | 2.56e9 | 116,000 | 5.7 | 7.4 | 11.6 | no |
| 10M × 512 | 1.02e10 | 29,000 | 1.4 | 1.9 | 2.9 | no |
| 100M × 64 | 1.28e10 | 23,190 | 1.14 | 1.49 | 2.3 | no |
| 100M × 128 | 2.56e10 | 11,600 | 0.57 | 0.74 | 1.2 | no |
| 100M × 512 | 1.02e11 | 2,900 | 0.14 | 0.19 | 0.29 | no |

**How the table is built.** Passes per second = U × 989.5 TFLOPS ÷ (2 × parameters × tokens), which is the leading term of Kaplan et al.'s forward cost ([Kaplan](https://arxiv.org/abs/2001.08361)). The H100 SXM's dense BF16 peak is taken as half its sparse figure of 1,979 TFLOPS, as the A100's 312/624 pair shows ([H100](https://www.nvidia.com/en-us/data-center/h100/), [A100](https://www.nvidia.com/en-us/data-center/a100/)). U = 30% is an assumption; it sits above the 5–20% that PufferLib reports for ordinary RL stacks ([PufferLib 2.0](https://rlj.cs.umass.edu/2025/papers/RLJ_RLC_2025_151.pdf)). Cores per GPU = passes per second ÷ DPCS.

**The project's earlier envelope checks out.** It said a thousand games per GPU-millisecond needs about 100 cores at 10,000 DPCS. A rate of 1e6 passes per second means parameters × tokens ≈ 1.5e8 at 30%, which needs ~64 cores at 15,600 DPCS and ~49 at 20,400.

**A node's own 14 cores per GPU suffice** once parameters × tokens ≥ ~6.8e8 at 15,600 DPCS. Examples: 10M parameters over at least 68 tokens, or 1M over 680. If the same GPU also trains, the threshold falls to ~1.7e8. That is because learning costs 6N per token against inference's 2N, so a GPU that learns from every decision spends 4× the inference FLOPs at sample reuse 1 and 7× at reuse 2.

**The literature's large ratios came from slow environments.** Its 100–600 cores per accelerator served environments that delivered 6–300 agent steps per core-second. At 15,600, the same GPUs would need 1/50 to 1/2,600 as many cores [D]. Only a small policy (≤ ~1M parameters, ≤ ~128 tokens) on a dedicated inference GPU needs more CPU than the node carries.

## Each floor traces to one workload and one instrument

### Floor 1: 10,000 decisions per loaded core-second, no item 42 needed

| Step | Arithmetic | Source |
|---|---|---|
| FLOPs per forward pass | 2 × 1e7 parameters × 128 tokens = 2.56e9 | [Kaplan et al. 2020](https://arxiv.org/abs/2001.08361) |
| H100 SXM dense BF16 peak | 1,979 TFLOPS "with sparsity" ÷ 2 = 989.5 TFLOPS | [H100](https://www.nvidia.com/en-us/data-center/h100/), [A100](https://www.nvidia.com/en-us/data-center/a100/) |
| Passes per second at 30% | 0.30 × 989.5e12 ÷ 2.56e9 ≈ 116,000 | assumption U = 30% (band 10–50%) |
| Physical cores per GPU | 12 (AWS p5 family, Azure ND H100 v5) to 14 (DGX H100/H200/B200) | [AWS](https://docs.aws.amazon.com/ec2/latest/instancetypes/ac.html), [Azure](https://learn.microsoft.com/en-us/azure/virtual-machines/sizes/gpu-accelerated/ndh100v5-series), [DGX H100](https://docs.nvidia.com/dgx/dgxh100-user-guide/introduction-to-dgxh100.html) |
| Rate needed to feed the GPU | 116,000 ÷ 14 = 8,300 to 116,000 ÷ 12 = 9,700 per core | [D] |
| Floor | rounded up to 10,000 (≤ 100 µs of engine CPU per decision); at 10,000 the node's cores keep the GPU fed up to U = 31–36% | [D] |

The base case is deliberately demanding. The notes' policy grid runs from 1M to 100M parameters over 64–512 tokens. 10M is the small end of the token-set policies: Chessformer's 6M-parameter model ([arXiv 2409.12272](https://arxiv.org/abs/2409.12272)), JueWu's 9M teacher models ([arXiv 2011.12692](https://arxiv.org/abs/2011.12692)) and Metamon's 15M Small model ([arXiv 2504.04395](https://arxiv.org/abs/2504.04395)). **It is not the small end of game agents generally.** Their MLP and CNN-LSTM policies run 0.15–2M parameters ([PufferLib 2.0](https://rlj.cs.umass.edu/2025/papers/RLJ_RLC_2025_151.pdf), [IMPALA](https://arxiv.org/abs/1802.01561)). This corrects the draft's framing, not its number.

128 tokens is the low end of the notes' inferred 128–512 for a Commander board; the engine's boards carry 16–64 permanents mid-game, plus hands, graveyards and the stack. Bigger boards lower the floor. An inference-only GPU at 30% is also near the top of current practice.

| Change from the base case | Floor implied (decisions per physical core-second) |
|---|---|
| Base: 10M × 128, inference-only H100, U = 30%, 12–14 cores per GPU | 8,300–9,700 → **10,000** |
| U = 10% / 50% | 2,800–3,200 / 13,800–16,100 |
| 64 / 256 / 512 tokens | 16,600–19,300 / 4,100–4,800 / 2,100–2,400 |
| 1M / 100M parameters at 128 tokens | 82,800–96,600 / 830–970 |
| Same GPU also trains (÷4 at sample reuse 1, ÷7 at reuse 2) | 1,200–2,400 |
| Scores each legal action, ~20 options ([Chessformer](https://arxiv.org/abs/2409.12272)) | ~410–480 |
| 800-simulation search per decision | ~10–12 |
| Three of four seats scripted (only the learner's decisions reach the GPU) | 33,000–39,000 |
| Featurization and IPC cost k × the engine's time per decision | × (1 + k) |
| One RTX 4090 (BF16, 0.167× an H100 ([Ada](https://images.nvidia.com/aem-dam/Solutions/geforce/ada/nvidia-ada-gpu-architecture.pdf))) with a 16-core desktop: 10M / 1M × 128 | ~1,200 / ~12,100 |

Tiny policies are beyond any node's own cores. A 1M-parameter, 128-token policy would need 83,000–97,000 decisions per core-second from the node, which is why published small-policy systems either add CPU nodes or write their environments in C. PufferLib's Ocean environments exceed 1M steps per second per core ([PufferLib 2.0](https://rlj.cs.umass.edu/2025/papers/RLJ_RLC_2025_151.pdf)).

**Today's reading needs one correction.** The draft took the loaded rate as 15,600 × 0.75 = 11,700, the efficiency of 8 workers on 8 physical cores. But "fully loaded" means every hardware thread is busy. The GPU nodes' 12–14 cores per GPU are physical cores with two threads each: AWS lists 2 threads per core on p5 ([AWS](https://docs.aws.amazon.com/ec2/latest/instancetypes/ac.html)), and GCP's vCPUs are hyper-threads ([GCP](https://cloud.google.com/compute/docs/accelerator-optimized-machines)). The 16-thread read gave an 8.2× speed-up, so the machine-wide rate per physical core is 8.2 × 15,600 ÷ 8 ≈ **16,000** [D].

The floor's reading should therefore be defined as one-thread DPCS × (16-thread speed-up ÷ physical cores). Both factors come from the 2026-09-15 read, which was taken before the shared ability list (−31% instructions) and u64 ids (−39%) landed, so the next readiness pass must re-read them. After the tap-solver re-base, the one-thread reading falls to ~10,600. The loaded reading falls to ~10,900 on this definition, or ~7,950 without SMT. The floor survives the middleware with about 9% to spare, which is too little to absorb the triggers phase without a lever.

**The instrument already exists.** `fuzz_games`' decision counters produce DPCS, and `fuzz_ab.py` enforces the 2.5% per-PR budget in µs per decision. The spine-close readiness pass records DPCS on both boards, runs the 1/8/16-thread contention read and runs callgrind. One addition is needed: record callgrind instructions per decision in the same sitting as the floor's first loaded reading, so that the floor can travel between machines. The current Commander-scale baseline is 95.96 G instructions for 200 games, the engine's reading of 2026-09-22.

| What moves the reading | Effect | Is it engine speed? |
|---|---|---|
| Tap solver or auto-payer middleware | ×~0.68: at 60 cards, 270 of 464 decisions remain, carrying 85% of the instructions | no; decisions get fewer and heavier |
| Triggers phase | lower: more work per decision | partly |
| Counting hardware threads | ×1.37 between the 11,700 and 16,000 readings | no; definition |
| Allocator contention | moves the loaded factor; a global-allocator swap is a ranked lever | partly |
| Agent | random-legal Forge games ran a median 35 turns against 19 for heuristic play ([ADR-0003](https://github.com/Tyrathalis/anvil/blob/main/docs/decisions/ADR-0003-m0-closeout.md)); a policy shifts the decision mix | no |
| Pool, seed, format | 20,400 at 60 cards against 15,600 at Commander scale | no |
| Machine | timings never cross machines; callgrind instructions do | no |

Item 42 is not needed for this floor. The owner's profile ranks it nowhere for straight-line throughput.

### Floor 2: a 10 µs clone, reachable only once the log leaves GameState

**The budget comes from card-game search.** ISMCTS chose 10,000 iterations per decision because its authors expected that to take about one second in an efficient implementation ([ISMCTS §V-B](https://eprints.whiterose.ac.uk/id/eprint/75048/1/CowlingPowleyWhitehouse2012.pdf)). The MTG ensemble-determinization paper spent the same 10,000 as 40 determinizations × 250 simulations ([Cowling, Ward & Powley 2012](https://eprints.whiterose.ac.uk/id/eprint/75050/1/EnsDetMagic.pdf)). That gives 100 µs per iteration [D]. AlphaZero's chess self-play spent 800 simulations in about 40 ms per move ([arXiv 1712.01815, Table S3](https://arxiv.org/abs/1712.01815)), or 50 µs each [D].

A stored-state iteration costs one clone plus one decision step, and a Commander step costs 64 µs. With a 10 µs clone the iteration costs about 74 µs, inside the budget, and the clone takes 10% of it. A 125 µs clone alone exceeds the budget. Replaying instead costs 4.1 × 64 = 262 µs per iteration before any clone [D]; 4.1 is ISMCTS's mean node depth on *Lord of the Rings: The Confrontation*. Go-Explore found that restoring saved states instead of replaying cut simulated steps at least tenfold ([arXiv 1901.10995](https://arxiv.org/abs/1901.10995)).

Storing states is therefore the only route to the standard budget at Commander step costs, and it puts a clone on every iteration. The same holds for fork-per-rollout methods such as SPARTA, which ran about 1e5 rollouts per game ([arXiv 1912.02318](https://arxiv.org/abs/1912.02318)).

**The floor is ≤ 10 µs per full-state clone on the reference machine at item 143's checkpoints.** Those are turns 1, 10, 20, 30, 40, 50, 60, 80, 100 and 150 and game end. The games are seeds 12345 and 777 plus the longest game of each pool's 100-game sweep, all on `fuzz_games`' `--deck-size 100 --life 40 --players 4` board. Each reading is the median of five means of 2,000 clone-and-drops, in a release build on one thread. Re-taken on this branch, the worst no-log reading is 8.8 µs (one rep 9.3 µs), 12% under the floor. With the history emptied too, it is 7.1 µs.

On other machines the floor takes a portable form: at most 1/6 of one decision's engine CPU, read in the same sitting (10 ÷ 64 ≈ 1/6.4). In CI it takes a proxy form: at most 64 allocations per clone, against 19–50 today.

Allocations make the better proxy for three reasons:
- Time tracks allocations more closely than bytes. In seed 12345's game, the 137 KB game-end clone takes 4.3 µs at 33 allocations, while the 96 KB turn-40 clone takes 6.8 µs at 50.
- The counts were identical under eight hash seeds, while bytes were not.
- They are independent of the machine and unchanged by a global-allocator swap, because the counter sees requested sizes.

**No MTG comparator is close to 10 µs.** Forge's median copy is 600× the floor and rebuilds every card from its script ([GameCopier.java](https://github.com/Card-Forge/forge/blob/master/forge-ai/src/main/java/forge/ai/simulation/GameCopier.java)). XMage deep-copies every card, the last-known-information maps and the whole state ([GameImpl.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/game/GameImpl.java)). The fastest published full-rules MTG search rate, argentum's 1,448 quiet-state simulations per thread-second ([training-data](https://github.com/wingedsheep/argentum-engine/blob/main/docs/ai/training-data.md)), is about 7× short of 10,000 per second.

**The instrument must be built.** Item 143's table came from a throwaway probe that each readiness pass rebuilds. Committing it means a `#[global_allocator]` that wraps `std::alloc::System` with atomic counters, using std only and adding no dependency. It should live in its own integration-test binary, so no other test shares the allocator, and hold a single test, so the counts stay exact. That test asserts allocations per clone at every checkpoint. The timing stays in the readiness pass, where machine-bound readings belong. The existing fork record-and-replay test covers fidelity, not cost.

**Item 42 must land first.** With the log in the state, the clone costs 8.8–11.6 µs at turn 20, 17.5–26.5 µs at turn 40 and 26.4–43.4 µs at turn 80. It reaches 130.6 µs at the end of the 184-turn game and 19.0 µs at turn 40 of a 60-card four-seat game. Today the floor is met only by a probe that clears the log. No harness may clear the log mid-game: pending and stacked triggers reference its records by id, and the dispatcher reads the current batch's suffix (`triggers-architecture.md` §3.4). TR-2a has already moved the "this turn" and "last turn" reads onto per-player turn summaries (§3.10), and every whole-log read left in the engine is in a unit-test tail. What item 42 has left is taking the log out of the state, down to a bounded window or to nothing, plus the sink. Either shape has to keep the records that pending and stacked triggers reference by id (§3.4).

Item 42 is necessary but not sufficient. Forks happen at priority boundaries today, and mid-round priority prompts await item 140. A search that branches on inner asks, which are 424 of a Commander game's 692 decisions, must fold them into a compound action or replay to them.

The reading can also move without the engine getting slower. Item 42's window adds bytes and allocations back. The triggers phase adds state, and TR-2a's history already costs a late clone 1.4–2.9 µs. The portable form loosens as decisions get heavier: the tap solver takes a decision to ~94 µs, which would allow a ~16 µs clone. The absolute 10 µs therefore governs on the reference machine.

### Floor 3: 128 KB per state, bounded by the board's high-water mark rather than the turn count

| Workload | States held | At 102 KB (no log, history emptied) | At 128 KB (floor) | At 209 KB (today, no log, end of a 168-turn game) | At 1,176 KB (log, end of a 184-turn game) | Memory available |
|---|---|---|---|---|---|---|
| Stored-state search at XMage's 5,000-node cap | 5,000 | 0.51 GB | 0.64 GB | 1.0 GB | 5.9 GB | ~2 GB per hardware thread (31.5 GB ÷ 16) |
| Stored-state search at 10,000 iterations | 10,000 | 1.0 GB | 1.28 GB | 2.1 GB | 11.8 GB | same |
| The same search on all 192 vCPUs of an AWS p5.48xlarge | 1.92M | 196 GB | 246 GB | 401 GB | 2.3 TB | 2,048 GiB ([AWS](https://docs.aws.amazon.com/ec2/latest/instancetypes/ac.html)) |
| Bot sandbox: Botzone ([wiki](https://wiki.botzone.org.cn/index.php?title=Bot/en)), LOCM ([COG 2019](https://jakubkowalski.tech/Projects/LOCM/COG19/)), Tales of Tribute ([arXiv](https://arxiv.org/abs/2305.08234)) | states that fit | ~2,500 | ~2,000 | ~1,200 | ~220 | 256 MB |
| 1,000 concurrent straight-line games | 1,000 | 102 MB | 128 MB | 209 MB | 1.2 GB | 31.5 GB |

**The budget is search memory.** At the floor, a 10,000-node stored-state search fits within the reference machine's memory per hardware thread. Today's no-log peak just overruns it, and with the log it overruns it about six times. Go-Explore-style archives, which keep one restorable state per cell, scale the same way ([arXiv 1901.10995](https://arxiv.org/abs/1901.10995)). With the history bounded, the floor leaves 26% of headroom over the measured 102 KB peak for the triggers phase and item 42's window.

For comparison:

| Comparator | Size | Source |
|---|---|---|
| Pgx fixed-shape states | ~46 KB (chess), ~25 KB (19×19 Go) [D from source] | [Pgx chess](https://github.com/sotetsuk/pgx/blob/main/pgx/_src/games/chess.py), [Pgx Go](https://github.com/sotetsuk/pgx/blob/main/pgx/_src/games/go.py) |
| SabberStone tag tables alone | ≥ ~11 KB [D from code] | [EntityData.cs](https://github.com/HearthSim/SabberStone/blob/master/SabberStoneCore/src/Model/Entities/EntityData.cs) |
| C++ Hearthstone MCTS | ~27 KB per iteration [D: 8 GB ÷ 300K] | [peter1591](https://github.com/peter1591/hearthstone-ai) |
| Lc0 stateless search nodes | 64 bytes each | [node.h](https://github.com/LeelaChessZero/lc0/blob/master/src/search/classic/node.h) |
| Forge | one game per 2 GB fixed heap | [ADR-0002](https://github.com/Tyrathalis/anvil/blob/main/docs/decisions/ADR-0002-fork-api-gate-resolution.md) |

**Measured on this branch, the floor fails without the log, and TR-2a's history is the whole excess.** The no-log state passes 128 KB from turn 60 of seed 12345's 70-turn game (131 KB), turn 80 of `performance` game 43 (129 KB) and turn 100 of `stress` game 7 (140 KB). It ends the two long games at 209 and 195 KB. `PlayerHistory` keeps one `TurnSummary` per player for every turn of the game (`state/history.rs`; `triggers-architecture.md` §3.10's "Whole game, not two turns"). Each row is `[u64; 24]`, 192 bytes, and the 568 and 523 rows at those ends are 107 and 99 KB. With every player's history emptied, no board passes 102 KB.

§3.10 deferred pruning the history "until a reading says it should", and this is that reading. A bound needs no whole-game rows. "This game" is a running total. "Last turn" is the previous row. "Since the beginning of your last turn" is two per-player accumulators, reset as each own turn begins. Together that is five rows per player, ~3.8 KB at four seats, which is close to item 42's original "two turns deep per player" design.

**The second growing term is map capacity, and it is bounded.** The battlefield map keeps the bucket count of the largest board it has held: 23 KB up to about 56 permanents and 46 KB beyond. That is why the rule says "high-water mark" rather than "board".

**The instrument is the same committed probe**, plus a CI test on exact bytes per clone at each checkpoint through game end on both seeds, with `MTGSIM_HASH_SEED` pinned. Pinning is required, not a precaution: bytes differed under two of eight hash seeds, where the battlefield map doubled one checkpoint early. Allocation counts did not differ.

**Item 42 must land first, and it is not enough.** The log is 981 of the 1,176 KB and 1,434 of the 1,479 allocations at the end of the 184-turn game. Whether item 42 bounds the log to a window or moves all of it out of `GameState`, the no-log columns above are what remains. TR-2a's history is the term in them that still grows with the turn count.

Straight-line batches do not bind this floor; search does. A thousand concurrent games need about 100–210 MB without the log and up to 1.2 GB with it, against 31.5 GB. The reading also moves with:
- the card pool;
- the seat count;
- the hash seed, by one map doubling;
- until the history is bounded, the game's length.

## GPU utilization carries the widest band; clone and byte figures go unpublished

| Missing figure | Where it is missing | What that leaves |
|---|---|---|
| Clone µs | XMage (a 2011 benchmark's number was never published ([commit](https://github.com/magefree/mage/commit/c02d453a4))), OpenSpiel, EnvPool, NLE, RLCard, robomage (has the tooling); SabberStone gives only a ratio | Floor 2 rests on search budgets, not on a peer clone figure. Forge's 6 ms is the only measured MTG copy |
| Bytes per live state | every framework; Pgx only by derivation from source; the Java engines publish heap budgets | Floor 3 rests on memory budgets |
| Decisions with ≥ 2 options | every source | per-step comparisons flatter the comparators |
| Four-player throughput | every MTG engine except one Forge token game | no Commander-scale comparator exists |
| Achieved GPU utilization for batched small-policy inference | no measurement | floor 1 spans 2,800–16,100 over U = 10–50% |
| Actor-side featurization and IPC per decision | not measured for this engine | floor 1 scales by 1 + k |
| Tokens per Commander observation | inferred only, 128–512 | floor 1 halves per doubling |
| Search path depth | one figure (4.1) | the replay column rests on it |
| Contention after the two levers | the engine's own reading | the loaded reading sits at 11,700–16,000 |

GPU utilization is both the widest band and the hardest to measure from here. The floor scales linearly with it, so the 10–50% band moves it 5×, and no source measures batched inference of 1–100M-parameter policies on an H100. Policy size moves the floor 100× across the notes' grid, and tokens move it 4× across 128–512. Training on the inference GPU divides it by 4–7, and featurization multiplies it by an unknown factor.

All of these live on the GPU side or in the harness. Floor 1 should therefore be re-derived once the Phase 10 AI harness can measure k and U. Floors 2 and 3 rest only on quantities the engine can measure today.

## Conclusion

The research use case inverts the throughput profile's priorities. For self-play, the engine already produces decisions faster than an 8-GPU node can consume them, for any token-set policy of at least 10M parameters over 128 tokens. The literature's hundreds of cores per GPU were the cost of environments running 6–300 steps per second, not a target to match. What a serious researcher meets first is the fork. At 64 µs per Commander step, the standard card-game budget is reachable only by storing states. Storing states is affordable only with a clone under ~10 µs and a state under 128 KB. Measured on this branch, the clone meets that only without the event log. The state meets it only without both the log and TR-2a's whole-game history. Item 42 is therefore the cheapest gate on the use case, and its emit door is already in place. Bounding the history is the other half of floor 3.

The floors are more exposed to changes of definition than to engine regressions. Middleware removes ~42% of the decisions at 60 cards without changing speed. Counting hardware threads moves the loaded reading by 37%. Featurization and utilization live outside the engine entirely. Each floor needs its definition fixed before its first dated reading: which decisions count, how many threads, which checkpoints, seeds and card pool. After item 42, the first failure is not a timing; it is floor 3. Without the log, TR-2a's history, at 192 bytes per player per turn, takes the state past 128 KB from turn 60–100 of the longer Commander games and to 209 KB at the end of a 168-turn one. Bounded, the state peaks at 102 KB.
