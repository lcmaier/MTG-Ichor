# A6g's review practices: the retroactive pass over the dev GUI

Branch `tooling/a6g-gui-review` (`roadmap-v2.md` A6g). The whole crate read at
66e2721 against the rule (logic in plain Rust under tests, egui only drawing),
`engineering-practices.md` §10.1's checklist, §2b's names, and whether a test
fails when the code breaks. Every finding is here, triaged, before any fix
lands; the PR that lands the fixes deletes this file.

**How it was read.** The plain-Rust half (`bridge`, `snapshot`, `prompt`,
`view_model`, `session` and the tests) by hand. The drawing half (`app.rs`,
`main.rs`, with `session.rs` and `bridge.rs` for threads and channels) by a
review agent given the checklist, the vendored egui 0.36.2 source to confirm
each behavior against, a budget of 40 tool calls or 15 minutes and a
stop-and-report rule: 31 calls, 13 minutes, eight findings. And by the three
instruments the PR adds, each run on the tree before any fix: the random-click
games (§10.3), the gate (§10.2) and the cost reading (§10.4).

**Triage.** *fix*: here, with a test where one can fail. *doc*: right as it
is once written down, where it is written. *defer*: real, and owned by a later
PR on the route, given a `codebase-state.md` item naming it. *design*: needs a
decision first, given an item naming the PR that decides.

| # | Where | Finding | Found by | Triage |
|---|---|---|---|---|
| 1 | `mtgsim` `engine/actions.rs`, `PlayerLoses` | A loss writes a layer-walk input without bumping the epoch. A layer condition names only players still in the game (`condition.rs`), so a frame from before the loss is served after it. At two seats, every read after a loss: the window's last snapshot of a game seat 0 lost holding Kird Ape panics in a debug build, so the window says "The engine panicked" where it should say "Game over". At four seats the leaving seat's cards move and bump, unless it owns nothing | random clicks, stress pool, seed 2 | fix: one bump, and `a_loss_bumps` |
| 2 | `mtgsim` `engine/actions.rs`, `GainLife`, `LoseLife` | Life totals are a layer-walk input too (`PlayerFact::LifeAtLeast`, `LifeAtMost` in a static ability's condition), and a life change bumps nothing. Unreachable: no registered static ability reads a life total (Felidar Sovereign's is a trigger's intervening "if") | reading 1's cause | defer: a `codebase-state.md` item beside 187, the same summary-bit fix |
| 3 | `main.rs` | The command line is read in a drawing file, untested, and naming the engine (`Scenario::parse` for the default seed): a bad `--seed` or `--pool` panics, a misspelled flag or one missing its value is ignored, and `--pool` beside `--scenario` is ignored | the gate; the agent (E7) | fix: `launch.rs`, plain Rust with tests |
| 4 | `main.rs`, `bridge.rs` | A scenario's seed is read once, at launch: Reload ignores an edited `seed` line, and a file that did not parse at launch pins the seed at 0 | the agent (E7) | defer: the tools PR, whose "a save as a start" reshapes `GameSetup` |
| 5 | `bridge.rs` `GuiSeat::answer` | Reload ends the old engine thread through `expect`, so the panic hook prints "the window closed with a prompt open" to the terminal, as if something broke; so does the cost reading's exit | the agent (E2) | fix: unwind with a marker the hook does not print |
| 6 | `main.rs`, `session.rs`, `bridge.rs` `DecisionLog` | Every start writes one decision-log path. Reload truncates the record of the game it replaces, the record most wanted right after something looked wrong; a superseded thread still playing can write its outcome into the new game's file; a second window on one seed overwrites the first's | the agent (E3) | fix: each start gets its own file, numbered as saved boards are |
| 7 | `app.rs` `zone_view` | A zone's `CollapsingHeader` takes its id from its label, which carries the count: an opened graveyard snaps shut when a card arrives, and an opened library at every draw | the agent (E4) | fix: a stable key on `ZoneView` |
| 8 | `app.rs`, the log | `show_rows` is given one text line's height and a log line wider than the panel wraps, so the log scrolls in jumps and the scrollbar is sized wrong | the agent (E5) | fix: one line a row, the whole line on hover |
| 9 | `app.rs` | After an engine panic the window names neither the decision log that reproduces it nor a next step | the agent (E6) | fix |
| 10 | `app.rs` | A failed "Save board as scenario" is drawn as a successful one is | the agent (E6) | fix |
| 11 | `bridge.rs` `DecisionLog::open` | A decision log that cannot be created reads as an engine panic | the agent (E6) | defer: the tools PR, which owns the log |
| 12 | `app.rs` `draw`; `snapshot.rs` | The board and prompt views are built again at every repaint, and the board's scenario text at every prompt | the agent (E8); by hand | doc: read at 27 µs a repaint and 60 µs a prompt on a 188-object board, in release (§10.4) |
| 13 | the debug build | A prompt costs about 125 ms on that board in a debug build, about 640 times its release cost: the layer memo's debug audit walks the board again at every memo hit, and the snapshot hits the memo several times a permanent. The audit is what found 1 | the cost reading | doc: `main.rs`'s usage says when to run in release |
| 14 | `app.rs` | The second click of a double click, or a held key's repeat, can answer a prompt that replaced the first under the pointer; keyboard focus may pass to the next prompt's button by position (not traced) | the agent (E1) | design: playable, which adds keyboard shortcuts |
| 15 | `view_model.rs` `Input::Option` | `Input::Option(2)` reads as Rust's `Option` at every call site | by hand | fix: `Input::OptionButton` |
| 16 | `view_model.rs` `Input::Adjust` | `Input::Adjust(0, true)`: the `bool` says nothing where it is written | by hand | fix: `OneMore`, `OneFewer` |
| 17 | `view_model.rs` `PromptView`, `OptionButton` | Three positional tuples: `number: (u64, u64, u64)`, `done: (String, bool)`, `amount: (u64, bool, bool)` | by hand | fix: named structs |
| 18 | `app.rs` `draw` | A `SessionHeader` parameter is named `session`, beside the `Session` type | by hand | fix |
| 19 | `bridge.rs`, `snapshot.rs` | One count, two names: `events_shown` and `events_seen` | by hand | fix |
| 20 | `view_model.rs`, `snapshot.rs` | A permanent's detail line names what it blocks and what it is attached to by bare id ("blocking #17"), and a pair's rule line names its half by id ("now click what #7 goes with"), where the board says "Hill Giant (#17)" | by hand | fix; the pictures change |
| 21 | `view_model.rs` `Marks`, `PromptView` | A click the window offers can do nothing: a permanent that several options start at (a land's five mana abilities) is outlined and its click is ignored; so is a placed option's button in an ordering, an option's button once the picks are full, and "Start over" with nothing chosen | by hand | fix: offer only a click that changes the answer, and the random-click games assert it |
| 22 | `view_model.rs` `WindowState::input` | Its comment says `app` acts on Reload and Save; `Session::input` does | by hand | fix |
| 23 | `bridge.rs` `play` | A dealt game derives its shuffle and agent seeds as seed + 1 and seed + 2, where `fuzz_games` and the scenario path use `RandomStreams::from_seed`: the window at seed N deals `fuzz_games`' decks for that game but not its shuffle | by hand | fix: one derivation, so a fuzz game opens in the window by its seed |
| 24 | `snapshot.rs` `words`, `prompt.rs` `question` | Two surfaces the engine should own: keyword names derived from `Debug` beside `ui::display`'s private `keyword_name`, and a question per `ChoiceKind` beside `ui/cli.rs`'s `prompt_line` | by hand | defer: ability names, which makes `ui::display`'s wording public |
| 25 | `prompt.rs` `option_view` | Nothing tests the order of a pair's refs (attacker then what it attacks, blocker then what it blocks), and the rule-played games click buttons, so a swapped pair passes every test | by hand | fix: a test |
| 26 | `app.rs` `side_panel` | The drawing decides to show an empty stack by its label's text (`starts_with("Stack")`) | by hand | fix |

**Order of the remaining commits**, each green: the engine fix (1, with 2's
item), the random-click games, `launch.rs` (3), the gate, the cost reading
(12 and 13 written down), then the fixes grouped by part: names (15–19, 22),
what the window names (20), clicks that do something (21), threads and logs
(5, 6), ids and rows (7, 8, 26), errors (9, 10), seeds (23), the pairs' test
(25); and last the items for 2, 4 with 11, 14 and 24, the A6g row as built,
and this file deleted.
