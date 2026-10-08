# The dev GUI map

**Where to start reading the dev GUI** (`devgui/`), the window a tester uses to
build a board, play every seat of a game on it, and watch the engine answer.
It names the parts, follows one question from the engine to the window and
back, and says how each part is reviewed. The rules for GUI code are
`engineering-practices.md` §10; the window's tools are `setup-architecture.md`
§7. Read at `7b7a8bc`, the dev GUI audit's last code commit, 2026-10-06.

## 1. What it is

The engine plays a whole game in one call and stops only to ask a seat for a
choice: a seat is a `DecisionProvider`, the one interface every answer in the
engine comes through. A window cannot stop, since it repaints whenever the
mouse moves. So the game runs on a thread of its own, and the window plays
every seat by answering the questions that thread sends it. The window is for
building and debugging cards, not for play against an opponent, so no agent
sits in it. It sees every card, and it is not v1's GUI (`backlog.md` §2.38).
**Out of scope** (A6g's row, the owner): hidden information, which a
perfect-information toggle for the tester can show once `backlog.md` §2.9's
model lands; card art, animation, drag-and-drop; and anything specific to one
mechanic.

## 2. The parts

| Part | File | What it does | Reviewed |
|---|---|---|---|
| The command line | `launch.rs`, `main.rs` | reads `--seed`, `--players`, `--scenario`, `--edit` and `--load`, and opens the window | `launch.rs` closely, `main.rs` by running |
| The session | `session.rs` | one game and one board: Play, Reload, Undo answer, Savestate and the menu, `--load`, "Save board as scenario", the editor. Every click lands here first, and one in the beat after the window was replaced is dropped | closely |
| The bridge | `bridge.rs` | runs the game on the engine's thread, replaying a line first when a rebuild or a load asks. Its seats carry each question to the window and the answer back, and it writes the game's record, the log and the save, one thread at a time | closely |
| The save | `save.rs` | the journal beside each log: every line played, where the window was asked, the savestates, each move; the tree Undo answer and the menu read | closely |
| The why's replay | `why_replay.rs` | a why about the past: the game built again, the window's line replayed into it with the trace sink on, and the why read at the open question from the trace (SU-8) | closely |
| The snapshot | `snapshot.rs` | the board copied as plain data at each question, read through the layers, so the window never touches the live game | closely |
| The prompt | `prompt.rs` | the question as plain data: who is asked, what, and each option's label and the cards it names | closely |
| The view model | `view_model.rs` | turns a snapshot and a prompt into what the window shows, and clicks into an answer | closely |
| The board editor | `editor.rs`, `boards.rs`, `search.rs` | a board built by clicking and saved as a scenario, a folder per board (SU-3), or as a new board under a name typed (#231); behind its Advanced switch, a line of the file typed in and each player's rows as controls (§11) | closely |
| The drawing | `app.rs` | lays out what the view model and the editor built, and reports clicks. It decides nothing | against §10.1's checklist, and by running |

The line between them is a gate: only `app.rs`, `main.rs` and
`tests/screenshots.rs` may name egui, and none of those may name the engine
(`plans/check_egui_only_draws.py`). Everything else is plain Rust under
tests, read the way engine code is read.

## 3. One question, there and back

Player 0 has priority in their main phase, with Lightning Bolt in hand.

1. **The engine asks.** On the engine's thread, the game reaches player 0's
   priority and calls seat 0.
2. **The seat's decorators may answer first.** Each seat is the bridge's
   `GuiSeat` wrapped in the decorators `cli_play` uses, helpers that answer
   some questions for you: `AutoYield` passes while a yield you set runs,
   `AutoPayer` pays what a cost leaves no choice in, and `ManaWindowStop`
   closes the mana window once a cost is paid (CR 601.2g). Full control
   turns them off, so every question reaches you.
3. **The bridge sends it.** `GuiSeat` copies the board (`Snapshot::build`)
   and the question (`Prompt`), sends both as `ToWindow::Prompt`, wakes the
   window, and waits. Nothing writes the game while its question is open, so
   the seat reads it with the layer memo's debug audit checking each frame
   once (`GameState::audit_each_frame_once`, `engineering-practices.md`
   §10.4), where a debug build would otherwise walk the board again at every
   read.
4. **The window shows it.** `WindowState::receive`, the view model, builds
   what to draw: Lightning Bolt marked clickable, a Pass button. `app.rs`
   draws it.
5. **You click.** `app.rs` reports the click as an `Input`. `Session::input`
   hands it to `WindowState::input`, which returns a `Reply` once the answer
   is complete, here "cast Lightning Bolt", and the session sends that to the
   engine's thread.
6. **The engine goes on.** The seat returns the answer, the engine checks it
   (`ui::ask`) and writes a decision-log line through the writer the bridge
   attached, such as `answer 12 [turn 3, precombat main] player 0
   PriorityAction picks cast Lightning Bolt (#12)`, and it casts the spell.
   Bolt's targets are the next question, which travels the same way.

## 4. Two threads

- **The window's thread** runs `app.rs` at every repaint and never waits. It
  reads the engine's messages with `try_recv`.
- **The engine's thread** runs one game, and waits at every question.
- **What ends the engine's thread:** the game's end; the window closing, or
  Reload starting another game, which drops the channel, so the old game's
  next question ends its thread quietly (`WindowGone`); a rebuild taking the
  game's record over (Undo answer, the menu), whose replay then stops at its
  next answer (`Stop::Superseded`) and which writes nothing more; or a panic,
  which the window shows as an engine bug. A scenario line the engine cannot
  play is shown as the file's refusal, not as a bug.
- **One writer of the record.** Each engine thread writes the log and the
  save through a `Writer`; taking the record over shuts the one before
  under the record's lock, so a game still running toward its next question
  cannot append to the line that replaced it. The window's thread writes a
  savestate and a move under the same lock, at the click.
- **A third, while a why about the past is read** (§10): a replay of the
  window's line on a thread of its own, which writes no record. It ends with
  its answer, sent over a channel of its own; a newer request supersedes it,
  and it stops at its next answer; and an answer the window no longer waits
  on goes nowhere.

## 5. Who computes what

From `engineering-practices.md` §10, and A6g's rule (the owner): **the window
draws only the engine's generic surfaces**, the layer output,
`ChoiceKind::subject()`, the options' ids, `ui::display`'s formatters and the
trace sink, never a case per mechanic. What it cannot draw from them is an
engine surface to add, not client logic.
- **A fact only the rules compute**, such as characteristics, the legal
  options or a prompt's subject, is the engine's. The snapshot reads it, and
  the window never computes a rule.
- **Wording every reader sees alike**, such as names, questions and option
  labels, is `ui::display`'s, in the engine crate.
- **How it looks**, such as layout, color and what is hidden, is the
  window's.

**The decision log is the engine's.** The engine words every answer, whoever
gave it (you, a decorator, or the engine's own pass), so a game writes the
same record whatever its seats were set to. The bridge only puts its lines in
the files. The save's own lines, where the window was asked, the savestates
and the moves, are the window's: which questions reach a person is one
client's choice, so they stay out of the log (`engineering-practices.md` §10).

## 6. Files on disk

From `devgui/`, where `cargo run` runs the window:
- `boards/<board>/` holds a board's file and the decision log of every game
  played from it, `seed-N.log`, named for the seed the game played. Save
  writes a board's own file, and Save as a new board's, under the name
  typed, numbered as a taken name is;
- `logs/seed-N-players-P.log` is a dealt game's decision log;
- beside each log, its save, `seed-N.log.save`: the session's journal of
  every line it played, where the window was asked, the savestates set and
  each move back (`save.rs`). The log is the one line the window is on,
  written again from the start at each Undo answer or move; the save keeps
  every line. `devgui --load` takes either and plays on in a new pair beside
  it, `seed-N-2.log` and its save, never writing the files it loaded.

## 7. Tests and measures

- `tests/headless_game.rs`: the bridge and the session with no window, another
  thread playing the window's part. Most of the window's behavior is tested
  here.
- `tests/random_clicks.rs`: whole games played by seeded random clicks on
  anything the window offers. Every click must change the answer or complete
  it (§10.3). The board editor's clicks too, with no game, the advanced
  settings' among them: each must change the board or what is shown.
- `tests/tools.rs`: Undo answer, savestates and the menu, the save and
  `--load`, on short boards through the session, with a replay superseded
  at the bridge.
- `tests/why_panel.rs`: the why panel through the session on the Humility
  sample: a right-click answered at the open question, the panel following
  its object to the next question and through Undo answer, its links, Back
  and close, and a Reload keeping it only on the same board. Since SU-8, a
  log line's why read from a replay stopped at the open question, a newer
  request replacing one on its way, and a finished game answering from its
  whole line.
- `tests/screenshots.rs`: the window drawn offscreen at the review boards,
  as the pictures in `tests/snapshots/`. A PR shows each picture it changed,
  old beside new. CI draws them and compares none, since its renderer is not
  the owner's machine, so a board that stops reaching its picture's question
  fails there (`codebase-state.md` item 216).
- `examples/prompt_cost.rs`: what one question costs the window (§10.4), and
  in a debug build what the layer memo's audit costs it. A PR that touches
  the snapshot or the view model quotes it before and after.

## 8. Reviewing a GUI PR

1. Find the parts it touches in §2. Most PRs touch the plain-Rust ones: read
   them as engine code, asking whether a test fails when the code breaks.
2. Read changes to `app.rs` against §10.1's checklist: per-frame work, never
   waiting, where state lives, repaints, how threads end, errors in words,
   widget ids, and a click landing on a screen that just changed.
3. Run the PR's click script, written in Magic terms, and look at its
   pictures. The check only a person can make is whether the window shows the
   game right.

## 9. Where the tools sit

SU-4 (#217) put the record's text and the replay in the engine
(`state::decision_log`, `ui::replay`); SU-5 (#219) built the window's side
(`setup-architecture.md` §7.1–§7.3). Every game the window plays is a start
and a line of answers, so any question it was asked can be asked again by
building the game afresh and replaying the answers before it.

- **Undo answer** goes back to the window's previous question: the session
  finds the last place the window was asked in the save, builds the game
  again and replays the line to it, with the layer memo's debug audits paused
  (this build checked those answers as you gave them), and the engine asks
  that question again. Off at the first question, with a line saying why.
  While a replay runs, the header counts its answers, and each press moves
  the target back one more question; the replay it replaces stops at its next
  answer.
- **Savestate** marks the open question's place in the save, named for its
  turn and step. The **Savestates** menu lists the savestates and "Back to
  where I was", the place the window's line last left; a click replays to it.
  The save keeps every line played, and the menu lists only these.
- **`--load FILE`** replays the line a save, or a plain log, ended on, with
  the audits on (another build may have written it), counting as it goes,
  then asks the next question. A line this build no longer takes says where
  it stopped, and the game plays on from the answer before it.
- The buttons come first in the header, after the Play | Edit switch, so a
  replay's count and the board's line changing width never move them under
  the pointer.

## 10. The why panel

SU-6 (#221, `setup-architecture.md` §7c) built its first part: a right-click on a
card, a permanent or a stack object asks why it is the way it is, and a panel
on the window's left shows what the layers did to it, line by line, with each
line's rule. SU-7 added the panel's first section, **At this question**: whether
the open question offers the object, and as what, and if it does not, why. A
player's line can be right-clicked too, and its why is that section alone.

- **Asked at the seat.** `WindowState::input` turns the right-click into
  `Reply::Why`, which the session sends as it sends an answer. The seat
  waiting at the open question (`GuiSeat::ask`) carries it out without
  closing the question, as it carries out "Stop yielding": it asks the engine
  (`ui::why::why`, which reads `engine::layers::explain` and, for the
  question's section, the checks the options were built by) and sends the
  answer back as `ToWindow::Why`. The seat hands it the open question too: who
  is asked, the `ChoiceContext` and the options, as the engine gave them. A why answers nothing, so it keeps the answer in
  progress and need not wait out the beat after a prompt arrives.
- **The panel follows its object.** The seat keeps the object or player the
  panel shows (`BoardRef`), and each later question's message carries its why. The session
  keeps the panel across Undo answer, a savestate and a Reload of the same
  start, whose games number objects alike, and closes it at any other
  start, `--load` among them.
- **The words are the engine's.** The panel lays out the engine's sections
  and lines; `WindowState::why_view` adds a link for each object a line
  names, Back, and a close, and `app.rs` draws them (`why_panel`). With no
  question open the panel keeps its last answer, its links off, and says so,
  and an object's hover says a right-click asks nothing (`Item::why_hint`).
- **Two tiers of "not offered".** *Never offered* is what the checks refuse,
  each line with its rule: the mana is short (CR 601.2h), it is not your turn
  (CR 117.1a), the Angel has flying (CR 702.9b). *Offered, then reversed* is
  an answer the engine took and undid at this question (CR 732.1), which the
  re-asked question's `rejected` names.
- **What happened (SU-8).** A right-click on a log line asks what its event
  did. Each line carries its event's number (`snapshot::LogLine`). The panel
  shows the event's batch as the engine decided it: the proposals, each CR
  616.1 iteration that met an effect or a "can't", and a batch after it that
  performed nothing, which is where a destruction a "can't" stopped shows.
  Then every triggered ability asked about the event, matched or refused and
  by what. Every question kind's line says what the question ranges over.
- **A question about the past is a replay's.** The engine reads a past event
  from its trace (`ui::why::why_from_trace`), and the window keeps no trace,
  so the session replays the window's line on a thread of its own with the
  sink on: `why_replay`, Undo answer's rebuild with three differences. The
  sink is on, nothing is recorded, and the seat behind the line reads the why
  at the first question the line does not answer, the open one, then stops the
  run. The game's own thread waits at its question meanwhile, and the panel
  says it is reading the trace. A why at a question whose options the trace
  explains (CR 616.1's choice, CR 603.3b's order) goes the same way. So does
  any why once the game is over, from the whole line. The window marks the
  request it waits on, `reading_the_trace`, and takes only that one's answer,
  so a newer request supersedes an older one.

## 11. The board editor's advanced settings

The editor clicks most of a board together, and shows ten kinds of line as
text: a player's counters, lands played, leaving the game, commander damage,
history counts, a card's `this turn:` and `counters:` lines, and setup
actions (`setup-architecture.md` §7b.2, decision 2). The header's
**Advanced** switch (#227) shows controls for the first four, and a field
for any line of the file. All of it is `editor.rs`, plain Rust, with
`app.rs` drawing it.

- **The field** (`Editor::add_typed_line`) puts the line last in the board's
  text and reads the whole text back, as every click's edit is read back,
  so Undo takes it out. Only the parser refuses a line: the board stays as
  it was, and the field keeps the line with the parser's words under it. A
  line the loader refuses goes in, as any click's edit does, and the
  refusal shows under the board.
- **The player rows** (`Editor::player_rows`): a toggle for leaving the
  game, and counts for lands played, poison, energy, and the commander
  damage from each commander on the board. Each count is what the loader
  will make of the words, so it adds poison words and keeps the last
  commander damage word, and a click leaves one word. `has_control` says
  which words a control shows; the rest stay text.
- **Not built:** controls for history, `this turn:` and `counters:` lines,
  and for setup actions, which the owner left off the route at the dev GUI
  audit since the field writes every row (`backlog.md` §2.42).

## 12. The Waiting panel

What the game is holding for later, under the side panel's shared zones (#233):
each delayed triggered ability waiting for its event, with its card's words,
whose it is, whether it fires once or each time this turn and the turn it can
trigger in, then each extra turn in the order it will be taken. The engine
answers it (`ui::waiting::what_is_waiting`, under `mtgsim/tests/waiting_test.rs`),
the snapshot carries it, `view_model.rs` makes it a zone of rows, and `app.rs`
draws that zone as it draws Exile: a collapsing header, closed until opened,
shown only while something waits. A row goes when its trigger fires or
expires, or its turn is taken, skipped, or lost with its player.
`tests/scenarios/waiting.scenario` is its board and `waiting.png` its picture.
TR-3b's "until" returns are in the engine's view already
(`Waiting::until_returns`) and wait for the dev GUI PR that draws them;
prevention and regeneration shields, and skipped steps, would join it as
kinds of row, not panels of their own.
