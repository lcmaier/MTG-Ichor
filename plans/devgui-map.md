# The dev GUI map

**Where to start reading the dev GUI** (`devgui/`), the window a tester uses to
build a board, play every seat of a game on it, and watch the engine answer.
It names the parts, follows one question from the engine to the window and
back, and says how each part is reviewed. The rules for GUI code are
`engineering-practices.md` §10; the window's tools are `setup-architecture.md`
§7. Read at `5d900c1`, SU-6's last code commit, 2026-10-05.

## 1. What it is

The engine plays a whole game in one call and stops only to ask a seat for a
choice: a seat is a `DecisionProvider`, the one interface every answer in the
engine comes through. A window cannot stop, since it repaints whenever the
mouse moves. So the game runs on a thread of its own, and the window plays
every seat by answering the questions that thread sends it. The window is for
building and debugging cards, not for play against an opponent, so no agent
sits in it.

## 2. The parts

| Part | File | What it does | Reviewed |
|---|---|---|---|
| The command line | `launch.rs`, `main.rs` | reads `--seed`, `--players`, `--scenario`, `--edit` and `--load`, and opens the window | `launch.rs` closely, `main.rs` by running |
| The session | `session.rs` | one game and one board: Play, Reload, Undo answer, Savestate and the menu, `--load`, "Save board as scenario", the editor. Every click lands here first | closely |
| The bridge | `bridge.rs` | runs the game on the engine's thread, replaying a line first when a rebuild or a load asks. Its seats carry each question to the window and the answer back, and it writes the game's record, the log and the save, one thread at a time | closely |
| The save | `save.rs` | the journal beside each log: every line played, where the window was asked, the savestates, each move; the tree Undo answer and the menu read | closely |
| The snapshot | `snapshot.rs` | the board copied as plain data at each question, read through the layers, so the window never touches the live game | closely |
| The prompt | `prompt.rs` | the question as plain data: who is asked, what, and each option's label and the cards it names | closely |
| The view model | `view_model.rs` | turns a snapshot and a prompt into what the window shows, and clicks into an answer | closely |
| The board editor | `editor.rs`, `boards.rs`, `search.rs` | a board built by clicking and saved as a scenario, a folder per board (SU-3) | closely |
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
   window, and waits.
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

## 5. Who computes what

From `engineering-practices.md` §10:
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
  played from it, `seed-N.log`, named for the seed the game played;
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
  it (§10.3).
- `tests/tools.rs`: Undo answer, savestates and the menu, the save and
  `--load`, on short boards through the session, with a replay superseded
  at the bridge.
- `tests/why_panel.rs`: the why panel through the session on the Humility
  sample: a right-click answered at the open question, the panel following
  its object to the next question and through Undo answer, its links, Back
  and close.
- `tests/screenshots.rs`: the window drawn offscreen at the review boards,
  as the pictures in `tests/snapshots/`. A PR shows each picture it changed,
  old beside new.
- `examples/prompt_cost.rs`: what one question costs the window (§10.4). A PR
  that touches the snapshot or the view model quotes it before and after.

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

SU-6 (`setup-architecture.md` §7c) built its first part: a right-click on a
card, a permanent or a stack object asks why it is the way it is, and a panel
on the window's left shows what the layers did to it, line by line, with each
line's rule.

- **Asked at the seat.** `WindowState::input` turns the right-click into
  `Reply::Why`, which the session sends as it sends an answer. The seat
  waiting at the open question (`GuiSeat::ask`) carries it out without
  closing the question, as it carries out "Stop yielding": it asks the engine
  (`ui::why::why`, which reads `engine::layers::explain`) and sends the answer
  back as `ToWindow::Why`. A why answers nothing, so it keeps the answer in
  progress and need not wait out the beat after a prompt arrives.
- **The panel follows its object.** The seat keeps the object the panel
  shows, and each later question's message carries its why. The session
  keeps the panel across Undo answer, a savestate and Reload, whose games
  number objects alike, and closes it at `--load`.
- **The words are the engine's.** The panel lays out the engine's sections
  and lines; `WindowState::why_view` adds a link for each object a line
  names, Back, and a close, and `app.rs` draws them (`why_panel`). With no
  question open the panel keeps its last answer, its links off, and says so,
  and an object's hover says a right-click asks nothing (`Item::why_hint`).
- **What comes next:** SU-7 adds why an option is or is not offered at the
  open question, and a player's line to right-click, and SU-8 what an event
  did and which triggered abilities were asked about it, read from a
  replay's trace (§7c).
