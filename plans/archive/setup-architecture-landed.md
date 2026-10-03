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

#### SU-4 — the replay — ✅ landed 2026-10-03

*Evicted 2026-10-03 from `plans/setup-architecture.md` §8, where the heading and a stub remain; SU-5's sizing stays there.*

### The build as sized (2026-10-03, at #216)

| SU-4, the replay: the engine's half | Where | Code | Tests |
|---|---|---|---|
| `ChoiceKind::as_str`, replacing the eight readers of `Debug` text | `ui::choice_types`; `scenario::setup`, `test_support`, devgui `prompt.rs`, four test files | 40–60 | 15–25 |
| The text: a record's header (the format, the engine), a start (a dealt game's seed, its `GameConfig` destructured with no `..`, and its decks by name; or a scenario's path, seed and text), an answer line and the outcome; written and read, each refusal naming its line; a start built into a game | `state::decision_log` | 330–470 | 120–180 |
| An option's identity, what was chosen: written, and compared by one matcher, which the replay, the setup driver and `ById` use; the scripted provider's answer by identity, `expect_choice`, which new tests use | `ui::choice_types`, `scenario::setup`, `ui::decision` | 150–230 | 80–130 |
| The replay: each line's player, kind, turn and step checked, its choice found by identity among the options this build offers and its fit through `ui::ask`'s predicates, which the validators then assert; a forced line skipped or answered where one build asks it and the other does not; the seats after the last line, or a stop naming where it diverged; a superseded replay stopped | `ui::replay`, `ui::ask` | 200–290 | 170–260 |
| The stop: why a run ended, the catcher over any run, a stopped game refused at each of the six entries; the setup driver's refusal as a stop, which `fuzz_games` reports as the game's error | `ui::decision`, `state::game`, `scenario::setup`, `bin/fuzz_games` | 100–145 | 60–100 |
| No layer audit while replaying within a session (§7.1) | `engine::layers::compute`, `state::diagnostics` | 15–25 | 15–25 |
| The dev GUI on the engine's text and the stop: its log written in the engine's words, and a refused setup line shown as a refusal rather than an engine panic | devgui `bridge.rs` | 35–50 | — |
| **SU-4** | | **870–1,270** | **460–720** |

At the last phases' rate SU-4's code is 1,310–2,670, so 1,770–3,390 in all.
Its upper half crosses 2,500, so the build measures at each commit, code and
tests apart, and stops to report when it crosses. An option's identity and
its matcher are the seam that would move into a PR of their own, ahead of the
replay, since they have consumers without it. **Its consumers**, each a test or a
client of what it builds (`engineering-practices.md` §4: every PR in a split
carries one):
- the replay test at `phase_a6g_integration_test.rs:180` moves onto the
  engine's replay, over the engine's text written and read back, at two seats
  and four;
- the setup driver and `phase_cv2a`'s `ById` find an answer through the one
  matcher, where each keeps a copy of its own today;
- SU-2's refusal test (`phase_su2_integration_test.rs:49`) catches a stop
  where it catches a panic today;
- `fuzz_games --scenario` reports a refused line as the game's error, where its
  `catch_unwind` counts a panic today;
- the dev GUI writes its log in the engine's text, so every game the window
  plays from SU-4 on is a record SU-5's `--load` reads, and it shows a refused
  setup line as the refusal it is.

Its tests beside them: a reordered option list replaying to the same game,
and a choice no longer offered stopping at its line; a scenario game with
setup actions replayed; a cast
from hand with exact mana under `ManaWindowStop`, replayed; each disagreement
(the player, the kind, the turn or step, an answer that does not fit) stopping
at its line; the stop's four reasons, and a panic that is not a stop passing
through; a stopped game refused; the audit's switch; the text's refusals and
a pinned text. Each replay plays a game capped at a few turns, as the a6g
test's eight are, timed in debug: `cargo test` runs the engine at opt-level 0
with the audit on, where a whole Commander game replays in ~35 s (§7.1).

**A/B, predicted before any arm runs.** SU-4 changes nothing a fuzz game's
outcome depends on: `fuzz_games` attaches no log, uses no replay, and meets
the stop only on a scenario's refused line, and `ui::ask`'s validators assert
the predicates they assert today, from one function each. So every gameplay
and cost row is predicted `IDENTICAL` on both pools at two seats and four,
and instructions per decision within ±0.3%, for code the compiler places
differently.

### Sized against built

Lines added, `mtgsim/` and `devgui/` with a `src` file's `#[cfg(test)]`
module counted as tests, read off #216's tip (`c3425c9`): each row its
commits' additions, so a line two commits touched counts in both, and the
totals the whole diff's.

| Part | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---:|---:|---:|---:|
| `ChoiceKind::as_str` and the eight readers it replaced | 40–60 | 44 | 15–25 | 46 |
| An option as what it is, the one matcher, `expect_choice` | 150–230 | 120 | 80–130 | 108 |
| The text: the format, the engine, the start written, read and built, the answer lines, the outcome | 330–470 | 575 | 120–180 | 142 |
| The replay, with `ui::ask`'s checks as predicates | 200–290 | 390 | 170–260 | 245 |
| The stop, and the setup driver's refusal as one | 100–145 | 106 | 60–100 | 122 |
| The audits paused within a session | 15–25 | 40 | 15–25 | 41 |
| The dev GUI's log in the engine's text, its start built through `BuiltStart` | 35–50 | 55 | — | 21 |
| **SU-4** | **870–1,270** | **1,322** | **460–720** | **718** |

The engine's own code is +1,259 against 835–1,220, its tests +680; the dev
GUI +101 with its bridge 39 lines shorter net. What the sizing left out: the
text's module doc, 24 lines of which moved from the old file into `mod.rs`
and count as added; the reader's refusals, which say what was expected where;
the predicates' error text, which keeps the validators' messages word for
word; `GameConfig` destructured and read a line a field. The text row ran
1.2–1.7 times its sizing and the replay row 1.3–2.0 with the predicates in it;
the rest came in near or under their rows.

### What the build changed in the design

1. **`as_str` moved, not added.** The trace sink kept an exhaustive match of
   the names (`trace_records::choice_kind_name`); `ChoiceKind::as_str` is that
   match moved onto the kind, so the trace, the log and a refusal spell a kind
   one way.
2. **Exact names exposed stale tests.** `"ChooseReplacement"` matched only as
   a prefix of `ChooseReplacementEffect`; `"DiscardToHandSize"` named a kind
   RE-8 renamed `Discard`, so `phase_re6`'s "no cleanup discard is asked"
   could not fail; `prompt_subject_test`'s list of every variant held 26 of
   27, its count hard-coded, with `OrderTriggers` missing from its list of
   kinds asked without a subject. All three fixed in the first commit.
3. **A scenario's replay runs no setup driver.** Its record holds the answers
   the driver gave, so `BuiltStart::replay` answers them from the log with the
   rest, and `BuiltStart::play` runs the driver. A record that ends inside its
   setup actions, as one refused there does, replays to its last line, and
   SU-5's load decides what a person sees then.
4. **A start is read within a record.** §8's risk said the reader would read a
   start apart from a record. It reads a line apart (`AnswerLine::read`) and a
   start only within `decision_log::read`, so SU-5's journal hands the reader
   its engine lines and reads its own.
5. **The audits' pause is the caller's.** The replay resumes the audits at the
   hand-over; a run that stops leaves them as its caller set them, since a
   stopped game is read and never continued.
6. **The dev GUI builds its start through the engine.** The bridge's own
   dealing and scenario build went with its log text: it makes a `GameStart`,
   builds it and plays it through `BuiltStart`, so the start a record names is
   the one the window played.
7. **`ReplayControl`** carries the lines answered and the supersede flag
   across threads, for SU-5's replaying count and an undo during a replay.

#### SU-3 — the board editor — ✅ landed 2026-10-03

*Evicted 2026-10-03 from `plans/setup-architecture.md` §8, where the heading and a stub remain, with the seats PR's sizing, which was made beside it.*

### The build as sized (2026-10-03, at the design)

**SU-3, the board editor, re-sized at its design (2026-10-03)** with doc
comments and messages counted, for decision 2's A and decisions 1 and 3 as
recommended (§7b.2). §7a's ~500–900 and the A6g row's ~350–600 came before the
design. SU-1's code came in at about 1.9 times its sizing and SU-2's at about
2.0, each from what its sizing left out (the archive's tables), while SU-1's
devgui part came in inside its range (198 against 150–220). Most of SU-3 is
devgui code like that part, and its refusals are the loader's.

| SU-3 piece | Where | Code | Tests |
|---|---|---|---|
| The names in development listed; `CardWord`'s `Display`; the step words and the tag spelling public | `cards::registry`, `scenario::text`, `scenario::write` | 25–40 | 15–25 |
| The editor: the board, undo, the renumbering, the check and its mark; each input; tags, order, attaching, leaving the battlefield; the search | devgui `editor.rs` | 400–550 | 260–380 |
| The editor's view: the facts, the seats and zones, the card buttons, the card's controls, the words shown as text, the search's results | devgui `editor.rs` | 250–330 | 100–150 |
| Session and launch: the switch, Play from the board's file, "Edit this board", "Edit the scenario", Save, Copy as text, the list, `--edit`; a folder per board | devgui `session.rs`, `bridge.rs`, `launch.rs` | 120–190 | 90–160 |
| The drawing | devgui `app.rs` | 220–320 | two pictures, 30–50 |
| **SU-3** | | **1,015–1,430** | **495–765** |

The editor's tests include its own random clicks (§10.3's shape): random
inputs from what its view offers, checking after each that the board is its
own text read back, that each offered click changes the board or the
selection, and that undo walks back to the start.

**Decision 2's B** adds, row by row: player counters ~45–65, lands played
~20–30, left the game ~20–30, commander damage ~50–80, history ~110–160,
`this turn:` ~90–130, `counters:` lines ~60–90, setup actions ~210–310: ~600–900
in all, so ~2,100–3,100 for SU-3. Decided A, these rows are the editor's
advanced settings, slotted on A6g's row after the "why" panel (§7b.2,
decision 2).

**The seats PR** (§7's "Seats", decision 4), with decision 5's lever:

| Seats PR piece | Where | Code | Tests |
|---|---|---|---|
| A stack of decorators and a yield per seat; the prompt names its seat; N decks; the agent and the two-seat refusal gone | devgui `bridge.rs`, `prompt.rs` | 50–80 | — |
| `--players N` | devgui `launch.rs` | 15–25 | 10–15 |
| The asked seat named and marked; seats in a fixed order; the outcome by seat | devgui `view_model.rs`, `app.rs` | 30–50 | 25–40 |
| The tests answering every seat; a dealt four-seat game and the four-seat sample played by rule | devgui tests | — | 60–100 |
| The engine at opt-level 1 in debug | `devgui/Cargo.toml` | 3–5 | — |
| **The seats PR** | | **~100–160** | **~95–155** |

Its review pictures are drawn again, since a prompt names its seat. Of its
~200–320, four seats are ~50–80: `--players`, and the four-seat games.

**A/B, predicted before any arm runs.** Neither PR changes what a fuzz game
runs. The seats PR touches no engine file. SU-3's engine changes are a new
listing and spellings made public, read by nothing `fuzz_games` reaches. So
every gameplay and cost row is predicted `IDENTICAL` on both pools at two
seats and four, and SU-3's instructions per decision within ±0.3%, for code
the compiler places differently.

### Sized against built

Lines added, `mtgsim/` and `devgui/` with a `src` file's `#[cfg(test)]`
module counted as tests, read off `main` (`7832f22`).

| Part | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---:|---:|---:|---:|
| The names in development, the spellings public, a card line written straight into its formatter | 25–40 | 50 | 15–25 | 27 |
| The editor: its model and its view (`editor.rs`), and the search (`search.rs`) | 650–880 | 1,379 | 360–530 | 328 |
| Session and launch: the switch, Play, "Edit this board", "Edit the scenario", Save, the list, `--edit`, the folders (`boards.rs`) | 120–190 | 376 | 90–160 | 225 |
| The drawing (`app.rs`), and `prompt_cost`'s editor readings | 220–320 | 359 | 30–50 | 41 |
| **SU-3, against `main`** | **1,015–1,430** | **2,164** | **495–765** | **621** |

The editor's random clicks (`tests/random_clicks.rs`) are its tests' 120,
in the second row. What the sizing left out: a type for each kind of
control in the view, each with its doc comment; thirty inputs, each a
variant with its line; the rules' helpers for references, tags and order; the
counter controls, one per kind; the folders as a module of their own; and
the read-back check. As SU-1's and SU-2's code ran at about twice its sizing,
SU-3's ran at 1.5–2.1 times, all but 50 lines of it in the dev GUI. The band
crossed 2,500 at the session's commit; the owner had said a dev GUI overage
may run (2026-10-03).

### What the build changed in the design

1. **The battlefield is one zone in the editor's order.** §7b.1 said a card
   moves up or down in its zone; across the seats that left Humility unable
   to arrive before Opalescence when each was its seat's only permanent, so
   the battlefield's order is one, its cards numbered across the seats, and
   the other zones are each seat's own.
2. **A line of several moves one copy.** Moving a card out of `library 0:
   Forest | x10` takes one Forest, so a line on the battlefield is always one
   permanent, which a tag can name.
3. **The random clicks found two offered clicks that changed nothing.** A
   move past only identical lines (two `Forest | x5` lines, one put back by
   hand) and a second copy of a tagged card, which the grammar refuses. A move
   is offered only when it passes a line that says something else, and a
   tagged card's copies have no "+".
4. **The read-back compares the text.** A name holding `#` reads back as
   another board, the comment cut off, with no parse error, so an edit is
   made only when the board read back writes the same text.
5. **`commander` off drops the commander damage that named only it**, as
   leaving the battlefield drops the words naming only a permanent: an edit
   that takes a card out of a reference's reach takes that reference.
6. **A fourth spelling made public**: `PlayerWord`'s Display, the player
   words the editor shows as text. And `CardLine`'s Display writes each word
   into its formatter: the snapshot's board text read 838 allocations on
   `main`, 904 with `CardWord`'s Display through `to_string`, and 601 now.
7. **The build's choices** the design left to it: a committed file's first
   save in a session makes a board, the first of `<stem>`, `<stem>-2`, …
   with no board file, and later saves write that board; "Edit this board"
   and "Save board as scenario" name theirs for the game's start and turn, a
   dealt game's `seed-N-turn-T`; a file's leading comment block, the comment
   and blank lines above its first line, is written back above the board.

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
