# setup-architecture.md — landed phases, evicted

**A record of finished work, not a plan.** Every section here sat under a ✅
heading in `plans/setup-architecture.md` and was moved out when its phase
shipped, leaving the heading, a stub and a pointer in the live doc. Nothing
here is owed and nothing here should be acted on; what each section is *for*
is the reasoning — the build as sized, what the building changed, the
measurement. `check_state_of_play.py` reads the ✅ headings that stay, and
fails when a landed section keeps more than 40 lines in the live doc
(`engineering-practices.md` §4). Later phases are appended by the PR that
lands them.

#### SU-1 — the scenario loader — ✅ landed 2026-10-01

*Evicted 2026-10-01 from `plans/setup-architecture.md` §8, where the heading and a stub remain.*

### The build as sized (2026-10-01, before the build)

Each commit is measured as code and tests apart.

1. **The doors.** The performer's state half and the construction door; the
   arrival turn; `intrinsic_entry_mods` moved from `test_support` into the
   engine, both calling one copy; counter-kind names moved into the engine,
   devgui's labels reading them. Tests: the door emits nothing, registers rows
   and gives loyalty.
2. **`mtgsim::scenario`'s types, parser and errors**, names and tags included.
   Tests: each error class.
3. **The loader**, its checks, the resume entry, the streams function, and the
   list of cards in development (§7a). Tests: one per word in §5.1; load emits
   nothing; a scenario game played twice is one game; the template and each
   sample under `mtgsim/scenarios/` load.
4. **The writer**, its destructure and its report, and the round-trip test over
   played boards (§4.3).
5. **`fuzz_games --scenario`**, droppable (~40 lines): random games from a
   board, and the three-hash-seed check from one.
6. **devgui:** the start, the argument, Reload, Save, load errors and the log
   header. Tests: the headless game from a scenario; the review pictures drawn
   from scenarios, with no seed hunted.
7. **Docs:** this file's ✅ section, `codebase-state.md` items for §10's
   findings with their slots, `roadmap-v2.md` A6g's row as built (it names
   SU-1 to SU-3 in their slots since this design's review), the
   `fuzz-record.md` block.

**A/B.** The performer's split and the arrival parameter carry the same values
in play, so `close_out.py` runs once and predicts `IDENTICAL` on both pools,
with item 194 merged first since it moves every two-seat game. **Review path:**
the PR body sorts files by how to review them, carries a click script in Magic
terms, and shows the pictures.

### Sized against built

Measured at each part's commit against the one before it; the parts' sums
count a line edited twice twice, so the whole is read off `main`.

| Part | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---:|---:|---:|---:|
| 1. Doors | 60–90 | 166 | 60–90 | 38 |
| 2. Parser | 250–310 | 603 | 130–170 | 83 |
| 3. Loader, checks, resume, streams, cards in development | 340–440 | 594 | 320–400 | 339 |
| 4. Writer and round trip | 180–250 | 707 | 140–220 | 265 |
| 5. `fuzz_games` | 30–50 | 49 | — | 0 |
| 6. devgui | 150–220 | 198 | 60–100 | 149 |
| **SU-1, against `main`** | **~1,010–1,360** | **2,307** | **~710–980** | **874** |

What the sizing left out, by part: the grammar table in the module doc (moved to
the design's §5.1 at review, 2026-10-01) and
the refusal messages that say what to change (2 and 3); the writer's
destructures, which name 97 fields a line each, and the report's ~30 checks
(4); `CounterType::name` and the move of `intrinsic_entry_mods`, counted
again where it landed (1). The tests came in near the sizing. The total
crossed the band's 2,500 at the writer's commit, before its tests; the
owner kept the PR whole there (2,515, with ~3,000–3,300 projected).

### What the build changed in the design

- **`arrived turn N`** beside `arrived this turn`: CR 302.6 measures from the
  controller's own most recent turn, so a non-active player's creature that
  arrived on their last turn is still sick, and "this turn" cannot say so.
  The writer writes an arrival only where that comparison still reads it.
- **One attacker per blocker.** A word that names a card takes the rest of
  its line, since a name may hold a comma; `blocking <card>, …` could not
  parse. No registered card blocks two.
- **`this turn: <card> | …`**, the bar like every other card line, and
  `ability N` by printed place for a card with two abilities it could mean.
- **A graveyard is listed bottom first.** §4.3's rule stamps lines in file
  order and each card goes on top; the example's "top first" contradicted it.
- **`dealt first-strike damage`** is refused before the first-strike damage
  step, not off it: the engine keeps the set to the end of combat.
- **Two refusals the design did not list:** turn 1's draw step at two seats
  (CR 103.8a, 500.11, from the brief), and a library that mixes shuffled and
  listed lines.
- **`cast` and `x_value` are named, not reported:** nothing reads them once the
  entry has been dispatched. `cost_choices` is reported: a condition reads it
  at rest.
- **The round trip's first run found a loader bug:** CR 508.8 skips declare
  blockers and the damage steps, and the loader also refused the end of
  combat step without attackers. Fixed before the writer's commit.

#### SU-2 — setup actions — ✅ landed 2026-10-01

*Evicted 2026-10-01 from `plans/setup-architecture.md` §8, where the heading and a stub remain.*

### The build as sized (2026-10-01, before the build)

§8 sized SU-2 at the driver ~180–260 lines, the action words ~80–120 and tests
~150–270, ~400–650 in all. The brief asked for a re-sizing before the build,
counting the arms its six open questions implied; it came to code ~570–735 and
tests ~370–530 in the band, and ~60–90 more in devgui, with SU-1's factor of
1.9 in mind.

### Sized against built

Measured at each part's commit, `mtgsim/src` and `mtgsim/tests` with a
`src` file's `#[cfg(test)]` module counted as tests; the whole is read off
`main`.

| Part | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---:|---:|---:|---:|
| 1. The words (`board.rs`, `text.rs`, `mod.rs`) | 80–120 | 121 | — | 31 |
| 2. The loader's resolution and refusals (`build.rs`) | in the driver's | 129 | — | — |
| 3. The driver (`setup.rs`), `fuzz_games` | 180–260 | 374 | 150–270 | 229 |
| 4. devgui | — | 16 | — | 31 |
| 5. Item 198 | 10–20 | 16 | a test | 54 |
| **SU-2, against `main`** | **~260–380** | **640** | **~150–270** | **314** |

The band's figure is +954 −47; devgui adds +47 −9 and one picture. What the
sizing left out: the names resolved in the loader rather than the driver (the
brief's one road for names), with their refusals; the match over all 22
`ChoiceKind`s, which a new question has to pass; the tap and the generic
split; and the refusal messages and doc comments, as SU-1's did. The
re-sizing came within its range, with the six open questions' arms counted.

### What the build changed in the design

- **`mode N` and `x N` wait.** Nothing asks for a mode: `Effect::Modal`
  cannot resolve (`backlog.md` §2.7). No X spell is offered at priority:
  `ManaPool::can_pay` and `find_mana_sources` read no X (`codebase-state.md`'s
  CR 107 row), so no game reaches CR 107.3a's question, and `AmountExpr::X`
  errors at resolution besides. `x N` was built in the first commit and left
  in the second, when its test found the window never offered the spell. The
  driver names `ChooseXValue` among the questions no line answers.
- **`AutoPayer` taps nothing.** It orders cost reductions; the window's pick
  is the driver's, the random agent's preference taking its first source, and
  the line's own permanent last: Mind Stone's ability otherwise taps the stone
  for its own {1} and rewinds.
- **The refusal of a line no seat reaches comes one prompt early.** The driver
  stops every seat at every priority point while a line is left, so a seat
  with `Pass` alone is asked rather than passed over, and the line is refused
  when its seat holds priority without its action.
- **Setup actions resolve nothing.** §0, §2, §5.2 and §7a had said they build
  a resolved effect; with two verbs and that refusal, they build a stack, and
  the seats' passes resolve it. A line that resolves the stack is open.
- **Names resolve at the load.** The design had the driver resolve each name;
  the loader does, through the table it built the board with, and hands the
  ids over beside the game, so its checks are refusals at load rather than in
  play. A target is a permanent, a card in a graveyard or exile, or a spell
  an earlier line casts, so a library's twenty Forests need no tags.
