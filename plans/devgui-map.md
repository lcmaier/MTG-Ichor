# The dev GUI map

**Where to start reading the dev GUI** (`devgui/`), the window a tester uses to
build a board, play every seat of a game on it, and watch the engine answer.
It names the parts, follows one question from the engine to the window and
back, and says how each part is reviewed. The rules for GUI code are
`engineering-practices.md` §10; the window's tools are `setup-architecture.md`
§7. Read at `2140218`, #217's tip, 2026-10-04.

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
| The command line | `launch.rs`, `main.rs` | reads `--seed`, `--players`, `--scenario` and `--edit`, and opens the window | `launch.rs` closely, `main.rs` by running |
| The session | `session.rs` | one game and one board: Play, Reload, "Save board as scenario", the editor. Every click lands here first | closely |
| The bridge | `bridge.rs` | runs the game on the engine's thread. Its seats carry each question to the window and the answer back, and it attaches the decision log's file | closely |
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
  next question ends its thread quietly (`WindowGone`); or a panic, which the
  window shows as an engine bug. A scenario line the engine cannot play is
  shown as the file's refusal, not as a bug.

## 5. Who computes what

From `engineering-practices.md` §10:
- **A fact only the rules compute**, such as characteristics, the legal
  options or a prompt's subject, is the engine's. The snapshot reads it, and
  the window never computes a rule.
- **Wording every reader sees alike**, such as names, questions and option
  labels, is `ui::display`'s, in the engine crate.
- **How it looks**, such as layout, color and what is hidden, is the
  window's.

**The decision log is the engine's.** The engine writes every answer, whoever
gave it (you, a decorator, or the engine's own pass), so a game writes the
same record whatever its seats were set to. The bridge only names the file.

## 6. Files on disk

From `devgui/`, where `cargo run` runs the window:
- `boards/<board>/` holds a board's file and the decision log of every game
  played from it, `seed-N.log`;
- `logs/seed-N-players-P.log` is a dealt game's decision log.

## 7. Tests and measures

- `tests/headless_game.rs`: the bridge and the session with no window, another
  thread playing the window's part. Most of the window's behavior is tested
  here.
- `tests/random_clicks.rs`: whole games played by seeded random clicks on
  anything the window offers. Every click must change the answer or complete
  it (§10.3).
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

SU-4 (#217) put the record and the replay in the engine (`state::decision_log`,
`ui::replay`). The bridge writes the engine's text and starts every game
through `GameStart`, so each game the window plays can be replayed. SU-5 adds
the window's side through the bridge and the session: Undo, a fresh game
replayed to your previous question; savestates; `--load`; and a `.save`
journal beside each log.
