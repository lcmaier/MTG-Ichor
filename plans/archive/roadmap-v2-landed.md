# roadmap-v2.md — landed work's records, evicted

**A record of finished work, not a plan.** A row of `plans/roadmap-v2.md`
keeps what is left, why it sits where it does and its scope. What a landed PR
built, the owner's dated decisions along the way and the sizes it was
estimated and built at move here verbatim when the row is cut (the owner,
2026-10-02, at A6g's ability names: the A6g row had grown to about 7,000
characters in one cell, a PR at a time). §3a's other landed rows followed the
same day, a section each below A6g's in the table's order. Nothing here is owed
and nothing here should be acted on; `backlog.md` §2.38 re-sizes v1's GUI from
A6g's numbers.

## A6g — the dev GUI, as its row stood on 2026-10-02

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A6g row, which keeps the
PRs left, the rule and the scope. The two cells, unchanged.*

### Do this

**The dev GUI** — a barebones testing client for card and fuzz work that sees every card; not v1's GUI, which is `backlog.md` §2.38. **Staged by the owner (2026-09-30) as a spike and three PRs.** **The spike** (`tooling/a6g-dev-gui-spike`, a draft): the engine on a worker thread under `DispatchDecisionProvider`, seat 0 a `GuiSeat` sending an owned snapshot (every characteristic off `compute_characteristics`, never `card_data`) and the prompt over a channel; a plain-Rust view model that handles the four primitives with no case per `ChoiceKind`; a thin egui layer; a decision log of seed, decks and answers; a headless whole-game test; review pictures drawn by `egui_kittest`. `devgui/` beside `mtgsim/`, a path dependency with its own lockfile and target, so the engine keeps its one dependency and every `mtgsim/target` path stands. **Re-ordered at #202's review (the owner, 2026-09-30), and grown at #204's (2026-10-01): ten PRs after the spike, in this order (SU-3 moved up and the audit added at #206's review; the review practices added, ability names split out of playable and SU-3 moved back at SU-2's), each named for what it delivers rather than numbered, since a split renumbers a list.** **Display fixes** (item 191), ✅ 2026-09-30 (PR #203): a copy named through the layers in the hover text, every keyword flag printed, the board printer nothing called deleted, and each log line naming an object as it was when its event happened, the trace's, the dev GUI's and the dumps' read off one record (`codebase-state.md` item 191, archived). **The scenario loader (SU-1)**, ✅ 2026-10-01: a board described at rest and built through the engine's construction doors, emitting nothing; written back from any game with what it cannot write reported; played by tests, `fuzz_games --scenario` and the dev GUI (`--scenario`, Reload, "Save board as scenario", a refusal shown in the window); a list of cards in development beside the registry (`setup-architecture.md` §8's ✅ section). Main item 194, the two-player first draw, landed first (PR #205). **Setup actions (SU-2)**, ✅ 2026-10-01 (PR #207): `then:` lines, any seat's casts and activations, played from the board by `SetupDriver` before the tester is asked anything, their names resolved by the loader and what can be checked refused at load; they build a stack and resolve nothing, and a line that would resolve it is open; no script for a seat during play (the owner, 2026-10-01). Item 198, deathtouch damage read by one state-based action check, rode along (`setup-architecture.md` §8's ✅ section). **The GUI review practices and a retroactive pass**, ✅ 2026-10-01 (PR #208; the owner's call at SU-2's review, since GUI code is the part of the tree the owner can least review line by line): `engineering-practices.md` §10. The rule, logic in plain Rust under tests and egui only drawing, is `check_egui_only_draws.py` in the check family; whole games played by seeded random clicks (`devgui/tests/random_clicks.rs`) assert that every game ends, every answer is accepted and every click the window offers does something; a cost reading, `examples/prompt_cost.rs`, prices a prompt on a 188-object board (about 0.2 ms in release and 125 ms in debug, where the layer memo's audit runs; 27 µs a repaint); and a checklist for egui code, which each GUI PR's review reads. The pass read the whole crate against all four: 26 findings, 19 fixed in the PR, 2 written down, and 5 given `codebase-state.md` items 199–202 with their slots. The random-click games' first run found an engine bug, a loss that bumped no layer epoch, fixed with an A/B (`fuzz-record.md`). **Ability names**, ✅ 2026-10-01 (PR #209), split out of playable at the same review: each ability carries its own paragraph of its card's Oracle text, `AbilityDef::rules_text` (CR 113.2c), the owner's call at the design review over a description written from the effect tree, which paraphrases; the field is the one the static parser (`engineering-practices.md` §3.4a) fills from the paragraph it reads, and every registered card's rules text was made Oracle's first, reminder text aside (CR 207.2), which 50 of the 180 real cards' was not. `ui::display` words each prompt's question, each option's label and each keyword's name for both clients (`codebase-state.md` item 202, archived), so the mana window says "Everywhere (#20) · {T}: Add {W}." where it said "ability 0", tapping a five-color land is not a memory game, and the CLI no longer prints its options with `{:?}`. **Playable**, after A6j: `FullControl` and `AutoYield` over the seat (its one-answer shortcut went in A6j, with main item 164); an engine surface the spike found missing, a reason on a re-asked prompt (CR 509.1c's blocker retry and CR 732.1's rewind re-ask in silence), which carries main item 193: the priority window's blacklist leaves the engine, the re-asked prompt says what failed, and the bot skips it; the card as printed beside the hover, which shows each ability's own text off the effective list since ability names, labeled as printed: a named exemption to "never `card_data`" (the owner's example: Arena swaps the two on a toggle at inspection), with both faces of a double-faced card side by side once CV-5 builds them (the owner, 2026-10-01); keyboard shortcuts. The CI job beside `check` landed with the spike (#201). **The tools:** undo as a replay without the last answer; a save as a start (a dealt game's seed and decks, or a scenario) and the decision log; savestates, positions the tester sets and moves between, as bookmarks in that log replayed to exactly (the owner, 2026-10-01; `setup-architecture.md` §7); and that log exported as `ScriptedDecisionProvider` expectations for a regression test. **The board editor (SU-3)**: a scenario built by clicking, editing the same `Scenario` value the parser builds (`setup-architecture.md` §7a); moved up to follow SU-2 at #206's review and back after the tools at SU-2's (the owner, 2026-10-01), so the window names what it shows before boards are built in it. **A "why" panel fed by the trace sink.** **A dev GUI audit, last** (the owner, 2026-10-01, at #206's review, since GUI code is the hardest of the tree to review line by line): the whole crate read against the review path's rule, logic in plain Rust under tests and egui only drawing, with its names and its tests audited as #206's were; the first PR after the "why" panel, reading what the PRs after the retroactive pass added. **The rule: the GUI draws only the engine's generic surfaces** — the layer output, `ChoiceKind::subject()`, the options' ids, `ui::display`'s formatters, the trace sink — never a case per mechanic; what it cannot draw from them is an engine surface to add, not client logic. **Out of scope for all of A6g:** four seats, hidden information (B4's seat toggle when B4 lands; once `backlog.md` §2.9's information model lands, the dev GUI keeps a perfect-information toggle for the tester), card art, animation, drag-and-drop, and anything specific to one mechanic

### Why here

**Moved ahead of the triggers build by the owner (2026-09-29)**: after the map and A6j, before A6i. **The spike ran ahead of A6j (the owner, 2026-09-30)** as a measurement, answering a one-legal-answer prompt in the seat until main item 164 moves it into the engine. TR-1 to TR-2b already make boards worth watching, and TR-3 to TR-7 (delayed and reflexive triggers, the look-back list, combat's shapes, state triggers, the Ironworks loop) are where visual testing pays most, so the client exists before they are built rather than after. It is still the boundary's second real client before Phase 8 adds `ChoiceKind`s, so `codebase-state.md` main item 141's unmet payload rule (`SelectRecipients`' `EffectRecipient`, the cost options' `Cost` trees) shows up while it is still one fix. It sees every card on purpose, so B4's back-stop, which is about a GUI that renders one player's view, does not bind it, and when B4's query lands the GUI is its first consumer (a seat toggle). **After main items 161 and 164 (A6j)**, `backlog.md` §2.22's "before the GUI": a GUI seat needs auto-yield, the full-control switch and no `[Pass]`-only prompts from its first game. Homed here for A4d's reason: tooling with a slot question. **Size, measured by the spike (2026-09-30):** 1,653 lines of code and 487 of tests, built in about half an hour after about two and a half of design; the bridge, the snapshot, the prompt and the view model, about 1,390 of those lines, carry into v1. The three PRs, judged from the spike's parts: playable ~400–600 lines of code plus ~60 in the engine, the tools ~600–900, the "why" panel ~400–700, each with ~150–300 of tests. The review practices and the retroactive pass, ~200–400 lines with their tests before what the pass finds, built at about 1,080 added in code, tests and tools, about half of it the pass's fixes; ability names, ~150–250 with tests, taking playable's ~60 engine lines, built at 826 lines of code with 328 of tests after three review rounds, once each ability carried its card's text and its paragraph: about 190 of the code is the new field where an ability is built, and 73 is card text. The scenario work, sized in `setup-architecture.md` §8: SU-1 ~1,010–1,360 lines of code, built at 2,307 with 874 of tests; SU-2 ~260–380, built at 640 with 314 of tests; SU-3 ~350–600 with its tests, likely low by the factor of about 2 both ran at. A6g is ~3,050–3,850 lines of code with the spike before the scenario work, inside the old guess of 3,400–5,200, and ~4,670–6,190 with it; the display fixes are in neither figure; §2.38 re-sizes from these numbers when it is designed

### Playable, ✅ 2026-10-02 (PR #211)

**Built:** `FullControl` and `AutoYield` over the window's seat, composed as `cli_play`'s, with a header switch the window flips from its own thread and yields set at the priority prompt they answer, and "Stop yielding" at any prompt while one holds; main item 193, a re-ask that names what the engine rejected (`ChoiceContext::rejected`, a `Rejection`), the engine no longer filtering, the random agent skipping its window's rejections, `SeatMode::person` gone and one hang guard, `REJECTION_LIMIT`; the decision log written by the engine (`state::decision_log`), every answer and the engine's own passes, a forced line marked; the card as printed beside the hover, `ui::display::printed_faces`, under a third named exemption, `// AS PRINTED:`; keyboard shortcuts, and item 201's beat. Found on the way by the random clicks, once full control asked at combat damage's priority points: `Scenario::write` panicked on a blocker whose attacking token had ceased to exist.

**Decided at the design review (the owner, 2026-10-02):** (a) the reason rides on `ChoiceContext`, typed and coarse, the last rejection only; (b) one hang guard for both loops, an error at the thousandth rejected answer; (c) item 201's beat, over dropping a double click's second half or waiting for the pointer to move; (d) the exemption named in `CLAUDE.md`, tagged `// AS PRINTED:`; (e) the decision log is the engine's, the same whatever answered: a seat's mode is never in it, since it says nothing about the game and tells a reader what the player expects, and the engine's pass is a line like the seat's. A log of every choice, the engine's passes included, was the owner's; the length was judged not to matter, and a filtered view is a filter on the forced mark when something reads it.

**Sized and built:** the row's ~400–600 lines of code with ~150–300 of tests, items 193 (~80–150) and 201 (~20–40) beside it; built at 2,252 changed lines of code and tests, 1,697 of them in source files with their unit tests and 555 in the test directories, `ChoiceContext`'s 52-literal sweep among them: about twice the code estimate, as SU-1 and SU-2 ran.

### The seats, ✅ 2026-10-03 (PR #214)

**Built:** every seat the window's, each with `cli_play`'s stack (`FullControl` over `AutoYield` over `AutoPayer` over `ManaWindowStop` over a `GuiSeat`) and a yield of its own, full control one switch, and the random agent gone from the dev GUI; a `Prompt` naming the player it asks, so `WINDOW_SEAT` went; the words that assumed one seat naming it ("Player 1 to decide", "Player 1: You have priority", "Pass until Player 1's next turn", "(to decide)" on the board, "Player 0 wins"); the seats drawn in one order at every prompt; `--players N`, dealt as `fuzz_games --players N` deals, and a scenario played at its own seat count; the engine at opt-level 1 in the dev GUI's debug build. Tests: a dealt four-seat game's opening hands (CR 103.8c) with every seat asked; `four-seats-commander.scenario` played by rule to its end without asking the seat that left (CR 800.4a), and under random clicks; a yield one click in twenty in the random clicks; the headless whole game on seed 6; the blockers picture asking for two options.

**Decided:** at SU-3's design (#213, `setup-architecture.md` §7b): decision 4, a PR of its own ahead of SU-3 with four seats in it, and decision 5, the opt-level lever in it, with the audit's share of a debug window's cost logged for the dev GUI audit (the owner, 2026-10-03). At that design's review the owner asked that a yield name its seat.

**Sized and built:** ~200–320 lines with tests (`setup-architecture.md` §8); built at +146 lines of code and +150 of tests against `main`, the docs beside.

**Measured** (the owner's machine, debug): the snapshot on the large board 121.7 → 29.5 ms with the lever (`engineering-practices.md` §10.4); the dev GUI's CI test step 16.5–17.9 s on `main`, 4.0–4.7 s with the lever alone, and about 25 s with every seat the window's, 23.4 s of it the random clicks. The window now builds a snapshot at every seat's prompts, where the agent answered seat 1's for nothing.

### The board editor (SU-3), ✅ 2026-10-03 (PR #215)

**Built:** a scenario built by clicking, beside the game in one window (`setup-architecture.md` §7b): an editor over the parser's `Scenario` value, each edit made on a copy written and read back, undo a stack of boards, the loader's refusal marking the card it names and Play waiting on it; the board's own words as controls and the other ten rows as text; tags, an Aura moved below its host, the words a card leaving the battlefield no longer has; a search over any list of names; Play saving the board and starting it from its file, "Edit this board", "Edit the scenario", `--edit [FILE]`; a folder per board under a git-ignored `boards/`, its games' logs beside it, with Copy as text and an Open… list. Tests: the editor's rules, its own random clicks (3,000 clicks, every one changing the board or the selection, each board its own text read back, Undo walking back to the start), and the session's Play, "Edit this board", "Edit the scenario", Save and logs, with a four-seat Commander board built in the editor and played; two pictures.

**Decided:** at the design (#213, the owner, 2026-10-03): one window; the board's own words, the rest to follow as the editor's advanced settings; `boards/` with a folder per board, the editor never writing a committed file. At the build, that the dev GUI may run over its sizing (the owner, 2026-10-03).

**Sized and built:** ~1,500–2,200 lines with tests (`setup-architecture.md` §8); built at +2,164 lines of code and +621 of tests against `main`, all but +77 of them in the dev GUI, the docs beside (`plans/archive/setup-architecture-landed.md`, "SU-3").

**Measured:** every fuzz counter file byte-identical to `main`'s, instructions per decision +0.14% (`fuzz-record.md`); on the large board the editor's view 19 µs a repaint and an edit 142 µs a click in release, 104 µs and 278 µs in debug; the dev GUI's CI test step about 24 s, as before.

### The replay (SU-4), ✅ 2026-10-03 (PR #217)

**Built:** the tools' engine half (`setup-architecture.md` §7.1–§7.3, §8). The decision log's text in the engine: a record opens with its format and the engine's commit, then its start, written, read and built into a game, then an answer a line naming what it chose rather than where the option sat, then its outcome. A replay answers from it, exact within a build and following the game across builds, a question with one legal answer skipped or supplied where two builds differ, and stops at a line it cannot follow, saying why. The stop, a typed unwind one function catches, and the setup driver's refusal raised as one. `ChoiceKind::as_str`, and an option logged by what it is with one matcher for replays, setup lines and tests' scripts. The memo's debug audits paused for a replay within a session. The dev GUI writes the engine's text and plays its start through the engine.

**Decided:** at the design delta (#216, the owner, 2026-10-03): two PRs along the engine's line; decision 6, the records at scale, with each answer recorded by what was chosen so that a later build replays a record until the game diverges, and nothing refused; `as_str` over `name`, which is a card's in the CR; the savestate menu's shape for SU-5; item 208, the engine crate's name, between Phase 8 and the v1 release; the sweep of the tests' scripted answers beside C0 (item 209).

**Sized and built:** ~1,330–1,990 lines with tests (`setup-architecture.md` §8); built at +1,259 lines of code and +680 of tests in the engine and +101 in the dev GUI, the docs beside (`plans/archive/setup-architecture-landed.md`, "SU-4").

**Measured:** every fuzz counter file byte-identical to `main`'s, instructions per decision −0.39%, from code placement (`fuzz-record.md`); the a6g replay test 0.28 s in debug with its replay's audits paused; the dev GUI's CI test step 22.8 s.

### The tools (SU-5), ✅ 2026-10-04 (PR #219)

**Built:** the tools' window half (`setup-architecture.md` §7.1–§7.3, §8). The save, `<log>.save` beside each decision log: a journal of every line the session played, where the window was asked, the savestates and each move, in the engine's words and the session's (`devgui/src/save.rs`). The record: one engine thread writes the log and the save at a time, a rebuild's taking over from the one before, whose replay stops at its next answer, and a replay's answers held until it hands the game to the seats. Undo answer, a fresh game replayed to the window's previous question, off at the first question with a line saying why; three presses during a replay wait for one. Savestate, named for its turn and step, and a menu of the savestates and "Back to where I was". `--load FILE`, a save or a plain log, replayed with the memo's audits on and counted, play going on in a new pair beside it; a line that diverges shown and played on from the answer before it. Item 200: a scenario's seed read from its file at each start, and a log that cannot be made refused in the window. Tests: undo across a target choice against a game played straight there, through `Scenario::write`; three presses landing three questions back; a superseded replay sending the window nothing; a savestate and a branch through a save and a load; a plain log's load; a divergence; item 200's two fixes, each failing on the tree before it. Three new pictures and eleven redrawn.

**Decided:** at the design (#212, #216, the owner, 2026-10-03): decisions 1 to 6, the menu's shape, and the split along the engine's line; at the build (the owner, 2026-10-04), no design PR, a gap to be brought back before working around it, and the dev GUI free to run over its sizing. None was found; a savestate is written at the click rather than relayed as a reply, the same order by the record's lock (`setup-architecture.md` §7.2, decision 4, amended where it stands).

**Sized and built:** ~970–1,420 lines with tests (`setup-architecture.md` §8); built at +1,162 lines of code and +784 of tests, all in the dev GUI, the docs beside (`plans/archive/setup-architecture-landed.md`, "SU-5").

**Measured:** no engine file changed, so no arms; `prompt_cost` in release as `main`'s, every reading's allocations and bytes the same, the header's tools 0.1–0.2 µs a repaint; the dev GUI's CI test step 24.0–24.1 s against `main`'s 23.2–23.4 s.

### What the layers did (SU-6), ✅ 2026-10-05

**Built:** the "why" panel's first part (`setup-architecture.md` §7c, §8). `layers::explain`, the layer pass for one object with a recorder where the game's passes hand none: each application in each layer as it applies, what it reached and what it did to the object, applied with the frame before and after or missed by its set, by CR 604.2 or by CR 613.6, with what it waited for (CR 613.8), and CR 306.5b's loyalty ability a step of its own; a debug assertion holding its answer to the walk's. `ui::why`, one value every client draws: the printed card, each layer's changes in the order applied, what reached the object's zone and missed it, and the result, each line with its rule and a link for each object it names. In the dev GUI, a right-click on an object asks why at the open question; the seat answers beside the snapshot and follows the object at each question after; the panel on the window's left has links, Back and ×, and is kept across Undo and an unchanged board's Reload; a hover says when a right-click asks nothing. Tests: Serra Angel under Humility; Humility and Opalescence in both orders; Urborg waiting for Blood Moon and gone by its turn; an anthem that misses; a CDA and counters; a card off the battlefield; a game with every why asked against the same game with none; the panel's round trip, Back and links, and a Reload keeping it only on the same board; random clicks with right-clicks; a picture.

**Decided:** at the design (#220, the owner, 2026-10-05), every recommendation: all four kinds of why in three PRs, SU-6 to SU-8; an engine query for a question about now and the trace sink for a question about then; now asked at the seat, then by a replay with the sink on; a right-click into a panel on the window's left that follows its object; the design in `setup-architecture.md` §7c. At the build, a player's line moved to SU-7, the layers having nothing to say about a player.

**Sized and built:** ~660–990 lines with tests (`setup-architecture.md` §8); built at +1,062 lines of code and +622 of tests, 820 and 318 of them in the engine, the docs beside (`plans/archive/setup-architecture-landed.md`, "SU-6").

**Measured:** every fuzz counter file byte-identical to `main`'s, instructions per decision +0.50%, about 0.10 of it the recorder's checks and the rest code placement (`fuzz-record.md`); on the large board a why at a question 38 µs in release and 1.9 ms in debug, and the panel 1.5 µs a repaint; the dev GUI's CI test step 14.4–15.0 s against `main`'s 28.9–30.9 s, the random clicks playing other games since a why's draw moved their stream (with the draw taken out, `main`'s games in 24.3 s against 27.2 s).

## A1 — the CR 704.5d token-order leak

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A1 row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-04** — the CR 704.5d token-order leak (`codebase-state.md` main item 6)

### Why here

the matcher reads that log; five lines and a test, closed in 83333e9

## A2 — LH-1 and LH-2, attachment as a layers input

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A2 row, which keeps its date, its PRs and what it delivered. The two cells, unchanged.*

### Do this

✅ **LH-1, LH-2** (2026-09-04, 2026-09-05) — attachment as a layers input (critical-path 6b)

### Why here

`layers-architecture.md` §13a: Aura and Equipment triggers need `attached_to` as a fact the effective-ability list reads; had to precede 7

## A3 — the CR 613.8 cluster

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A3 row, which keeps its date, its PRs and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-06** — the CR 613.8 cluster (7), with the `Condition` AST for conditional statics

### Why here

`layers-architecture.md` §13b: the board-wide pass is the frame conditional statics and CR 603.4's intervening-ifs both evaluate against, so the trigger phase adds a reader, not a language; unblocked RS-3b

## A4 — RD, then RE

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4 row, which keeps its date, its PRs and what it delivered. The two cells, unchanged.*

### Do this

✅ **RD, then RE** (2026-09-08 → 2026-09-15) — the rest of critical-path item 5, closed with RE-9 and audited the same day

### Why here

`replacement-architecture.md` §9 and §14; RD's CR 120.3 decomposition is what "whenever you lose life" sees, RE's `CreateTokens`, `PlayerLoses` and draw replacement are what 249 token-watchers and 391 draw-watchers see post-replacement; the audit's record is `codebase-state.md`, "Was critical-path item 5 done, and what sits before item 6? — audited 2026-09-15"

## A5 — CR 113.6, which abilities function in which zone

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A5 row, which keeps its date, its PRs and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-14** — CR 113.6, which abilities function in which zone (critical-path 6a): the `ObjectSet` rename, then LJ, then LK

### Why here

the one facility four docs named and none owned, three PRs in the order the owner set at RE-8's close; what shipped narrower and wider than this row assumed is `layers-architecture.md` §13c and §13d, `replacement-architecture.md` §11 items 4 and 9, and `codebase-state.md` item 119 (unblocked). The three-way split held under the build: *which abilities function where* was this row, *casting from a non-hand zone* is `backlog.md` §2.3, *a continuous effect reaching a non-battlefield object* was LJ

## A4e — item 138's two counters

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4e row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-16 (PR #155)** — item 138's two counters, `decisions` and `priority_decisions`, with their rows in `fuzz_games` and `plans/fuzz_ab.py`

### Why here

first in the slot because it costs nothing and changes what every later A/B reads: CPU per decision, the ratchet's own unit (`engineering-practices.md` §3.1). What it cost that the row did not predict: the stated definition and the measured one disagreed by 30 forced prompts a game (`codebase-state.md` item 138's correction and the new item 145), and the Commander-scale board had to become two harness flags before its rows could be re-read. Item 145's fix rode along on the owner's call at review — a forced prompt is CPU spent against the ratchet's own numerator — which cost 40 scripted tests their expectations and made the PR a stream-moving one with a `fuzz-record.md` block

## A5b — the gather's zone leg (RF)

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A5b row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-16 (RF)** — the gather's zone leg: `replacement::gather` reads static replacement abilities where CR 113.6 says they function, off a second candidate set the registration doors keep; LK had landed the registration leg only (`replacement-architecture.md` §11 item 4, closed)

### Why here

critical-path 6a's remainder and the spine's own next step, so it preceded A6's first PR. Darksteel Colossus rather than Blightsteel — infect is unbuilt — with Nexus of Fate as the stack-shaped second card; the affected side's zone check replaced the `debug_assert` and corrected three registered rows; the card's shuffle needed `GameAction::ShuffleLibrary`, the first in-game writer of a library's order. `replacement-architecture.md` §9, "Phase RF"

## A4f — lever 1, the ability list behind an `Arc`

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4f row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-16 (PR #157)** — lever 1, the ability list behind an `Arc` (`codebase-state.md` main items 67 and 138; the reading is `layers-architecture.md` §12)

### Why here

22.5% of a four-seat `stress` game's instructions (`layers-architecture.md` §12, 2026-09-15); answer-preserving, ~100–150 lines plus 13 call sites; A/B `IDENTICAL` on both pools at two and four seats, then a callgrind re-read against §12's reading. The owner's preference (2026-09-15) is the two levers before the doc so the trigger phase's own A/Bs are cleaner, and the engineering half of that: the matcher's per-event sweep is `get_effective_abilities` per object per event, the exact path this turns from a `Vec` clone into a refcount

## A4g — lever 2, process-stable ids

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4g row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-16 (PR #158)** — lever 2, process-stable ids in place of the v4 UUIDs (`codebase-state.md` main item 144, closed and archived, with item 149 opened beside it; `plans/id-hasher.md`)

### Why here

22.1% of instructions hash 16-byte keys with SipHash; the decision replaces the keys rather than only the hasher: `ObjectId` a `u64` stamped in `add_object` beside the timestamp, `AbilityId` derived from the card the way the v5 intrinsic site already does, a one-line mixing hasher, the `uuid` dependency gone. It halves every id, retires the fuzz-dump and fork-test masks, and removes the engine's one ambient-randomness call; the cost is that map iteration order becomes process-stable, so the CI determinism step seeds the hasher per run (~10 lines, in the same PR). About 200–300 lines, one PR, A/B'd as two arms — the type swap, `IDENTICAL` expected, then the hasher — with a callgrind re-read. Two things item 144 leaves the PR to decide: how a granted ability's id is minted, and whether the two ids become newtypes. Both levers before A4h, because an instruction count travels only at identical games and A4h moves the stream

## A4h — item 139 and item 41's fork test

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4h row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-17 (PR #159)** — item 139, the retry re-ask's stale list, carrying item 41's fork test (`codebase-state.md` item 139 archived, 41 recorded as met for round starts, 150 opened and closed; `fuzz-record.md`'s A4h block)

### Why here

~5 lines in `engine/priority.rs`; it moves the random agent's stream, so its own A/B and a `fuzz-record.md` block; the test is ~250 lines beside `tests/determinism_test.rs` plus `#[derive(Clone)]` on the random provider. After the levers for the reason above; **before A6's first PR** because the test is main item 40's check — the pending trigger queue must be on `GameState`, the one design constraint with a deadline. **What it cost that the row did not predict: two fixes, not one.** Item 41's test could not go green on item 139 alone — building it found item 150, an `activatable_abilities` that never applied CR 601.2c, so a blacklisted equip ability came straight back and the blacklist rather than the board was deciding the prompt. Across the test's 192-game sweep: 119 branches failed to replay on `main`, 38 with item 139's fix alone, 92 with item 150's alone, 0 with both. No fixture migration — no scripted test reaches the retry loop. The A/B needed a fourth arm, since a stream-moving PR has no identical counters to read §3.1's budget at, and the overhead lands at +0.7% to +3.9% CPU per decision, at the budget's line rather than inside it

## A4b — the rulings ledger

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4b row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **tool done 2026-09-17** — the rulings ledger (`engineering-practices.md` §3.4a): `plans/check_rulings.py`, `plans/rulings-ledger.json`, and the check line's fifth script. **The reading is a standing queue, not a phase**, and A4b read its head

### Why here

**Scheduled here by the owner (2026-09-08): after replacements, between phases.** It gates nothing and nothing gates it, which is exactly why it needed a slot rather than a "whenever". The re-count the row asked for: **330 rulings over 160 real printings, 125 on the 90 pooled cards** — not the ~250 a projection from the 2026-09-08 census gives, because the carrying rate rose from 46% to 60% as well as the card count. What the row did not predict: the gate had to be scoped (a card is owed once it is `read`, or once it is registered after the ledger existed) or it would have failed on all 330 the day it landed, and the escape had to be five kinds rather than one, because each becomes untrue a different way. **The sitting found no defect**, and §3.4a says why that is structural rather than luck: a count-ordered queue puts the cards each phase was built around at its head. **Remaining: 308 rulings over 93 cards, 103 of them on 38 pooled cards**, tracked in the ledger and printed by `--queue`. **Pace set by the owner 2026-09-17: the pooled 103 takes a slot between phases whenever one is free — about five sittings at A4b's rate, no deadline — and the off-pool 205 is not scheduled.** Between phases rather than during one for the reason §3.4a gave A4b's own slot: a bug this finds in a pooled card is an engine fix that moves the random agent's stream and owes its own A/B, which is cheaper between phases than inside one. Nothing forces the pace, because the backlog is frozen — see `codebase-state.md` item 151. Still **before item 6's first card PR** — that phase's registrations are the first large batch the pass (§3.4) will read, and now the gate is what makes it happen

## A4i — CR 601.2c's instances of "target"

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4i row, which keeps its date, its PRs and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-17** — CR 601.2c's instances of "target" (`backlog.md` §2.20, graduated and struck; `codebase-state.md` items 152–156; `fuzz-record.md`'s A4i block; trace page `plans/traces/a4i-a-target-belongs-to-an-instance.html`; the review's nine themes and where each went, `codebase-state.md`'s A4i section)

### Why here

its own back-stop (2026-09-04): **before RS-2 and before A6's first PR** — CR 603.3d chooses a trigger's targets per instance as the ability goes on the stack, and RS-2's `validate_targets` builds on the same shape. **What the row did not predict, three things.** The consumer is not a bite spell: Rabid Bite needs `AmountExpr::TargetPower` and a damage source that is a target, two facilities of its own, where Seeds of Strength's three identical "target creature" clauses are the case no structural rule can decide and cost nothing else. The entry's "a recipient per atom (or per instance)" had a third shape it did not name — Incremental Growth's "**another** target creature", which CR 601.2c reaches through "as long as it fits the targeting criteria" rather than through a rule, and which is the biggest population §2.20 counted (36 cards on that phrase alone); it is `ObjectFilter::OtherThanInstance`. And **CR 601.2d is split out** as A4l rather than shipped: it is a second mechanism, not a second clause, and a second consumer. Six of the eight atoms; the A/B was `IDENTICAL` on `performance` and found a live bug on `stress` — Skullcrack's three damage never landed when the card was cast, because the old first-atom rule read `Controller` off its first restriction and announced no target at all

## A4j — `ChoiceKind::subject()`

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4j row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-18** (PR #168) — **`ChoiceKind::subject()`**, matched without a wildcard, with `tests/prompt_subject_test.rs` walking whole games at two seats and four (`backlog.md` §2.21, struck; `codebase-state.md` item 141's watch item closed; `describe()` was built and taken out in review — the strings are the client's, and `ui/cli.rs`'s `prompt_line` is exhaustive instead). Three variants gained the id they lacked — `ChooseAlternativeCost`, `ChooseAdditionalCosts`, `GenericManaAllocation`; `LegendRule` was counted as a fourth and is a legitimate `None`, since CR 704.5j names no member of the group. Every gameplay counter `IDENTICAL`, both pools, two seats and four — no `fuzz-record.md` block

### Why here

~150 lines, any time before A6's first PR, so the phase's new `ChoiceKind`s (CR 603.3b's ordering, "may" triggers) decide their subject at birth against an exhaustive match; the payload rule item 141 records is what a trigger prompt must obey

## A4m — `EngineCounters` is `Diagnostics`

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4m row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-18** (PR #165) — **`EngineCounters` is `Diagnostics`, and `GameState.counters` is `game.diagnostics`** (`codebase-state.md` item 132, struck and archived; **no `fuzz-record.md` block, because nothing moved**)

### Why here

A naming fix the owner asked for on 2026-09-17, after A4i and A4h both rubbed against it. `GameState.counters` was `state::diagnostics::EngineCounters` — layer walks, memo hits, decisions — while `PermanentState.counters` and `PlayerState.counters` are CR 122's +1/+1 and poison, and `Primitive::AddCounters`, `CounterType` and `EntryCounters` are all the second kind. **The CR owns the word** (`CLAUDE.md`'s naming rule), so the diagnostics moved and the counters stayed. The name was the PR's one decision and the owner took `Diagnostics` over `codebase-state.md` item 132's proposed `EngineMeters`, because the module is already `state::diagnostics`. **The check held:** `IDENTICAL` on both pools at two seats and at four against a `main` arm built at 573ca8b, and raw `fuzz_games` output byte-identical between the arms apart from the four timing lines; the printed row labels and `fuzz_ab.py`'s `ROWS` table are untouched, which is what keeps every recorded table greppable. **What it cost that this row did not predict:** the sweep was **120 lines across 21 files**, not "72 sites across 12". The row counted the type and the `record_*`/accessor calls and so missed both halves of the same thing — `ui/ask.rs` passes `&game.counters` as an *argument* at 25 `validate_*` call sites and builds 11 `EngineCounters::default()`s in its own unit tests (48 lines in that file, not 19), and seven integration test files read the accessors for 34 more. What made the sweep safe was not the count but the anchor: every substitution required one of the 27 recorders and accessors after `.counters.`, which two `HashMap`s one struct over cannot satisfy

## A4n — the instance list, computed once and stored

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4n row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-18** (PR #166) — **The instance list is computed once, in `CardDataBuilder::build`, and stored** (A4i's review, themes H and I.2, closed; `fuzz-record.md`'s A4n block; the third rider is row A4q)

### Why here

`spell_instances` / `effect_instances` allocate a `Vec` and deep-clone each clause on every call, and `castable_spells` calls one per card in hand per priority check. **Measured at A4i's review: 366 calls and 325 clause clones per game** — a thread-local probe, 50 games at seed 12345, single-threaded, `performance` pool — derived more often than the layer system walks (340), against 226 decisions. Not an A4i regression: the code it replaced cloned one recipient per call, and A4i made it a `Vec` plus *n*, with *n* = 1 for every card but Incremental Growth — which is why every arm read `IDENTICAL`. **The fix is not deriving at all**: the list is a pure function of the card and `CardData` is built once behind an `Arc`, so compute it in `CardDataBuilder::build()`, store it on `CardData` and each `AbilityDef`, and return `&[EffectRecipient]`. Zero allocation, zero clone, and it retires the re-derivation invariant `resolve_effect` currently rests on rather than restating it. A field, two accessors, ~40 call sites taking a slice; owes an A/B whose prediction is `IDENTICAL` counters at a lower CPU reading. Any time; before A6's first PR is enough, since that phase multiplies the call rate by every trigger it puts on the stack. **Three riders from the audit (2026-09-17, theme I.2).** `resolve_effect_at`, the one site that indexes the flat buffer, calls `to_vec()` on the instance slice — a heap allocation per targeting atom per resolution that theme C's buffer was built to remove and that the borrow checker does not need (the slice passes through; verified with `cargo check`, reverted): two lines, same A/B. The precomputed list cannot be filled only in `CardDataBuilder::build()`: card files construct `AbilityDef` as struct literals, and a Layer 6 `GrantAbility` carries one inside a `Primitive` that never meets the builder, so it is filled where every definition is born or computed lazily. **Amended 2026-09-18, reading the tree before the work started:** that rider is half wrong and the other half is bigger than it says. `build()` already walks the set the precompute needs — `AbilityId::UNASSIGNED`'s doc: it replaces the id "on every def it can reach from the card — the printed list first, then every def nested in an effect (a granted ability, a token's abilities)" — so a def nested in a *card's* effect does meet the builder, and only a def synthesized at runtime (`AbilityId::on_object`) escapes it. But `AbilityDef` has six fields and **160 struct literals name every one** (108 in `src/cards`, 27 elsewhere in `src`, 25 in `tests`), so a seventh field is ~160 mechanical edits however it is filled, and nothing uses `..Default::default()`. And the call sites are **13, not ~40** — of the four `effect_instances` ones, `mana_helpers:399` already holds an `&AbilityDef` and `registry:987` is a cold registration check, but **`resolve.rs:112` takes an `&Effect` and nothing else**, so where the resolution path reads its stored list is a decision this row does not make. And identical clauses that read no earlier instance are checked once — Seeds of Strength, pooled, runs three `has_legal_choices` battlefield scans per priority pass per copy in hand for one answer **What shipped, and what the row did not predict (2026-09-18).** `AbilityDef::instances` and `CardData::spell_instances`, filled by `build()`'s existing id-stamping walk — the Aura rule (CR 303.4a) moved from `spell_instances` the function into `build()`, and `cards::registry`'s test re-derives both lists independently for every registered card, which is the gate on a derived field. The walk itself is `Effect::instances` now, with two callers: the builder and that test. **The literal count was 133, not 160**: the grep behind 160 counted 25 `-> AbilityDef {` function signatures and the struct definition, and a scripted insert after each literal's `id:` line landed all 133 (91 in `src/cards`, 19 elsewhere in `src`, 23 in tests). **The resolution path reads the announcement, not a def.** An ability on the stack has no `CardData` def in hand, an Aura's one instance sits in no effect tree, and CR 615.5's rider has neither — so `resolve_effect_with_announced_targets` takes the entry's `chosen_targets` as its clause list (`targeting::DeclaredInstances::Announced`), the invariant that a second walk numbers the atoms as the first did is retired rather than restated, and the bare `resolve_effect` a test or a rider calls reads the tree lazily, only for a `SameInstanceAs` atom. Rider 1 rode: the instance slice passes through to `resolve_primitive` without `to_vec()`; the two `to_vec()`s further down `resolve.rs` are a registry row taking ownership of its targets, a different class, and stayed. **The check held and the lower reading did not show**: `IDENTICAL` on both pools at two seats and at four against a `main` arm built at 879f114, and `µs / decision` inside the sitting's own spread at both seat counts — ~366 allocations per ~6,700 µs game is below what a timing sitting can read, which the row's own "cost-model finding, not a timing one" already said; the instrument that found it, the per-game call count, is what moved, and the `fuzz-record.md` block carries both numbers. Rider 3 is split out as row A4q, because it changes how many battlefield scans run and that moves the counters this row's claim needs identical.

## A4o — "counter target spell" and an activated ability

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4o row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-18** (PR #163) — **`SelectionFilter::Spell` accepted an activated ability on the stack, and the pool had both halves** (`codebase-state.md` item 159, struck; `fuzz-record.md`'s A4o block)

### Why here

**Found by A4i's audit (2026-09-17) and reproduced with a fixture.** `validate_spell_target` checks stack membership only, the enumeration yields every stack id and A4i's `has_legal_choices` arm counts every stack id; only the sibling `DamageSource` arm filters on `is_spell`. Counterspell and Merfolk Thaumaturgist's activated ability are both in `PERFORMANCE_POOL`, so a random game casts Counterspell at the ability — forced, no prompt, since the ability is the only other stack object — and `Primitive::CounterSpell` moves the ephemeral ability object to its owner's graveyard: the fixture ends with **two Merfolk Thaumaturgist objects, one on the battlefield and one in the graveyard**. First in the slot because it is live in the measured games. The fix is the `is_spell` filter in the validator, the enumeration arm and the count arm, ~20 lines, and the regression casts from hand; it changes what `castable_spells` offers whenever an ability is on the stack, so it moves the random agent's stream and owes its own A/B and a `fuzz-record.md` block. **Before A6**, whose triggered abilities are the same ephemeral stack objects and would be counterable by the same mistake. **What the row did not predict, two things.** The three sites are three, but they do not change the same thing: the count arm withdraws the offer, while the enumeration arm only shortens the candidate list — which is the whole difference whenever a real spell sits on the stack beside the ability, and the attribution needed both probed before every diverging game was accounted for. And the row said "a random game casts Counterspell at the ability" with Merfolk Thaumaturgist as the example; the measured runs countered **nine** different sources' abilities into graveyards — 26 phantom cards over the four A/B arms, Chainbreaker and Bonesplitter the most frequent — so it was the equip and the `{T}:` ability generally, not the one card. The prediction that did hold is `differ` on both pools at both seat counts, with the cost inside noise

## A4p — seats that have left the game

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4p row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-18** (PR #164) — **The `Player` and `Any` arms count and offer seats that have left the game** (`codebase-state.md` item 160, struck; `fuzz-record.md`'s A4p block)

### Why here

**Pre-existing, and A4i's count logic inherited it.** `num_players()` is the player vector's length, which CR 800.4a never shrinks; `enumerate_legal_selections_upto` yields `0..num_players()` for `Player` and `Any`, and `has_legal_choices` counts the same. Reproduced at four seats with seat 3 departed: both filters offer `Player(3)`; `validate_targets` refuses it for `Player` (a cast the oracle offered and the engine rewinds — item 139's class) and **accepts it for `Any`**, because `validate_any_target` never asks `in_game`, so "any target" damage resolves against a player who is not in the game. Three edits, ~15 lines: the player iterator filters on `in_game`, the two count arms count in-game seats, `validate_any_target` gains the check the `Player` validator has — and that validator's comment claiming a departed seat is "not offered at CR 601.2c" comes true. Two-player streams cannot move, since a two-player departure ends the game (CR 104.2a), so the A/B prediction is `IDENTICAL` on both pools at two seats and the four-seat `stress` arm is where it differs. **Before A6 and before Phase 9**: v1 is four seats, and a trigger's "target opponent" reads the same arms. **What the row did not predict, three things.** The A/B prediction named the four-seat `stress` arm as the one that would differ; four-seat `performance` differs too and by nearly as much — 65 diverging games of 200 against `stress`' 68 — because Lightning Bolt is pooled on both. **Only the enumeration ever changed an answer in the 400 measured games:** every registered card with a `Player` or `Any` instance declares `TargetCount::Exactly(1)`, so both count arms are asked `n = 1` and answer `true` while any seat remains — that half of the fix is correct and unreachable at once, and what would reach it is a card with two instances of "target player" and two seats left. And the damage did **not** come through CR 608.2b, which is where the row put it: the eleven Lightning Bolts `main` resolved against a departed seat were aimed at one *at announcement*, by an enumeration that still listed it, so on the fixed arm `validate_any_target` never had a departed seat to refuse. That validator is the back-stop for the narrower window — a seat that leaves between announcement and resolution — which did not open once in 400 games and is why it needed a fixture rather than a fuzz run

## A4q — identical clauses checked once

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4q row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-18** (PR #167) — **Identical clauses that read no earlier instance are checked once** (A4i's review, theme I.2's third rider, split out of A4n on 2026-09-18; `fuzz-record.md`'s A4q block)

### Why here

`every_instance_has_a_choice` asks `has_legal_choices` once per instance, and Seeds of Strength — pooled — carries three identical `Target(Creature, Exactly(1))` clauses that read no earlier instance, so one castability question costs three battlefield scans per copy in hand per priority pass; each scan is a `validate_selection` per candidate, which is a layer query. The fold is in `oracle/mana_helpers.rs`: a clause equal to an earlier one that nothing feeds forward to reuses that clause's answer, ~15 lines. **Not a rider on A4n**, because it changes how many scans run, which moves memo hits and possibly layer walks — the counters A4n's `IDENTICAL` claim needs still — so it owes its own A/B, with the prediction that only `Memo hits` moves and moves down, and `--require "Seeds of Strength"` to widen the reading. Any time; before A6 for the reason A4n gave, since triggers multiply the call rate. **What shipped, and what the row did not predict (2026-09-18).** The fold is a `reusable_from` cursor and a `contains` over the clauses already answered in that range — no side table, so a one-instance card, which is every card in the pool but one, pays nothing at all and the change allocates nothing. **The prediction held on all four arms**: every gameplay counter `IDENTICAL` on both pools at two seats and at four, with `Memo hits` the only row that moved and down in all four cells (−0.46% and −0.76% on `performance`, −0.15% and −0.20% on `stress`), and −1.42% with `--require "Seeds of Strength"` forcing the card into every deck. **`Layer walks` did not move, and the row's "possibly layer walks" was the right worry about the wrong shape**: a fold across *identical* clauses leaves the surviving clause walking the same candidates and filling the memo with exactly them, so only the repeat hits go away — a fold across clauses with different filters is the one that would trade a hit for a walk, and the reusable range is not where those live. **And the card set is one card, checked rather than assumed**: a probe over `default_registry` walked every `AbilityDef::instances` and `CardData::spell_instances` for a clause equal to an earlier one in the reusable range and named Seeds of Strength alone out of 166 — Incremental Growth's three clauses each exclude the ones before them and Plague Spores' two name a creature and a land — so `stress`'s narrower gap is dilution and nothing is owed in `codebase-state.md`. The fixture measures one `castable_spells` in layer queries against a **one-clause twin of the same cost** rather than asserting an absolute count, because an absolute would have to know what the rest of the castability check spends on a board; it reads 3 against the control's 1 on the pre-fix tree

## A4k — the middleware census

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4k row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-18** (PR #169) — **`backlog.md` §2.22 rewritten as the middleware census**: three rules, nine rows sized and sequenced, five non-rows named, and the one code change it names — `AutoPayer`'s unreachable forced-split branch retired, and at the owner's review the rest of the payer scheduled into the engine as an elision, full control reshaped as a switch above the stack, the solver's preference left to the client and the reversal settled as keep-or-reverse-all by a clone (`codebase-state.md` main items 161–165; items 72, 84 and 145 updated in place; §2.35, the "why can't" oracle query, filed from A4j's review)

### Why here

a doc sitting, sequenced ahead of full control by the owner (2026-09-08); beside the doc, because CR 603.3b's ordering prompt joins the census's residual the day triggers exist and the doc classifies it at birth — done: **C**, residual, the census's row 8, with item 47's elision precedent for identical triggers. **What the sitting did not predict, three things.** The decision that looked like the row's whole code change was the smallest of three jobs the payer had, and the two that matter are absences rather than answers — never taking a split with surplus, and the engine's own guard since A4e — so the retirement is ~50 lines out and the rule it leaves is the census's first. Two of the ten candidate rows were not middleware at all (auto-sacrifice is a harness's own provider, staged payment a client's buffer), and one thing that was not on the list is the largest forced prompt left: the `[Pass]`-only priority prompt, 91.5% of priority prompts at four seats, the engine's by that same rule, with a 148-fixture migration behind ~10 lines (main item 164). And the tree's citation for the forced-choice rule, "CR 102.2", is the two-player-opponent rule in `tmnt.txt` — a comment fix, recorded. Every gameplay counter `IDENTICAL`, both pools, two seats and four — no `fuzz-record.md` block

## A4c — the trace sink

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4c row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-18** (PR #170) — **the trace sink**, tier 2 of `engineering-practices.md` §7.1: `mtgsim/src/state/trace.rs`, five emit points (the batch, the CR 616.1 iteration, the layer walk, the decision boundary, the performed event), JSON lines with no `serde`, `fuzz_games --trace DIR`, `cli_play --trace PATH`, `test_support::install_trace`; `plans/trace_spine.py` and `plans/traces/viewer.html` render a spine and mark where the argument goes. `codebase-state.md` "Before Triggered abilities" item 5 closed and archived, item 9 opened for the dispatcher's own emit point; main item 42's stream half rode (`--dump-events` is a projection of a trace) and its window did not; main item 166 holds the two-version diff; `fuzz-record.md`'s A4c block

### Why here

**Out of A6's first PR and into its own (the owner, 2026-09-08)**: A6 is 4–6 PRs of new subsystem, and "why did this fire, or not" wants answering before that phase starts. **The six decisions, as taken.** "Off" is one branch on an `Option` at each emit point with the payload built behind it, no cargo feature, so the A/B compared one binary with the sink off and on against `main`: every gameplay and diagnostic counter `IDENTICAL` on both pools at two seats and four, the off arm inside §3.1's budget. The sink lives behind a handle whose hand-written `Clone` is the fork marker, never a buffer on the state. No `serde`; a builder that escapes strings. The decision boundary is the four `validate_*` helpers, the way item 138's counters landed, plus a `priority_rejected` record in the retry loop — which is what answers A4h's question: the offered list, the rejection with its error, the re-ask. Item 42's performed-event stream reaches the sink through one `emit_event` door; the in-state window stays unbounded. The two-version diff is out, and the format is shaped for it (`seq`, `branch`, a header with the seed and `stamp_commit.rs`'s commit). **What the row did not predict.** The Rust is ~1,000 lines against item 5's 300–400: half of `trace.rs` is module doc and the JSON builder, the 25 `ask_*` call sites carry the prompt to the validators, and the test asserts a page's spine row by row. The consumer — rd-2's Trace A regenerated from its test — found two things the pinned page says and today's engine does not: the batch numbers the attacker's assignments 0 and 1, and the rider's batch inherits the applied set since RE-2; that is what a spine diff is for, and the page stays pinned. The sink on costs +131% CPU per game at two seats, about a megabyte of records a game — recorded, not budgeted. Trace determinism under three hasher seeds held first time at both seat counts, because the one map the sink reads, the object store, is sorted by id before it is written

## A6 — the triggers architecture doc and item 6's first phases

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A6 row, which keeps its date, its PRs and what it delivered; the rest of critical-path item 6 is rows A6b, A6d and A6e. The two cells, unchanged.*

### Do this

**The triggers architecture doc**, then **critical-path item 6**

### Why here

52 CR 603 atoms plus ~80 more Phase 7 atoms in CR 610/608/724/707/605; **the five problems and three seams §3b names are the bar** — P1–P5 from `atomic-tests/supplemental-docs/state-tracking-architecture.md`, S1 CR 603.10a's look-back list written about *visibility* (the doc consumes `Zone::is_public()`, zero callers, and does **not** pull `backlog.md` §2.9 forward), S2 CR 121.2c's recipient ordering (main item 122, RE-2's Alms Collector the customer; the doc decides between item 122's two shapes beside its own APNAP ordering), S3 per-instance ability identity (main item 149: `AbilityIdentity`'s `(source, ability)` pair cannot tell two instances of one ability on one object apart since A4g; the doc decides whether CR 603.7h counts the instance or the ability, and the identity gains the index if the instance). The first PR carries LKI's consumers, `cast_by` (main 9), CR 605.1b (main 11) and CR 707.10b's ability identity. **What waits for the doc, because the doc names its fields:** item 42's per-player turn summaries (P2–P4), §2.28's mandatory half (P5), item 122 (S2), the LKI reader ("Before Triggered abilities" item 3), CR 800.4d's refusal (item 7), CR 603.6c's qualifier (item 6), CV-4's CR 707.10b half. Written before CV-7, on the single-component `PermanentState`, so it says which of its reads key on the permanent and which on a component (CR 729.3d). **One constraint inherited from A4o (2026-09-18):** "counter target spell" now asks `is_spell_on_stack`, so a triggered ability is uncounterable by it for free — as long as this phase puts triggers on the stack as the same ephemeral objects activated abilities use, with `is_spell: false`. If the doc chooses a different shape, it owns the filter question; either way the complement Stifle's class wants is that predicate negated. **Surveyed 2026-09-18 — step 1 done** (`plans/references/trigger-survey.md`, regenerated by the script beside it; `codebase-state.md` "Before Triggered abilities" item 2 closed, items 10–18 opened): every event CR 603.1b–603.12a names against the corpus and the 14,149 paper cards that carry a trigger, and the printed distribution against the performed event that would carry it. **The doc's opening list, from the survey.** Nine gaps the stream has today, each an item — three performers drop a proposal field (whose step, combat or not, the life loss's cause; item 10), the attack record has no defender (11), no event announces a target being chosen, ward's 195 cards among the 312 (12), none a control change (13), the LKI frame carries no status (persist, undying, 603.6e; 14) and is captured for battlefield departures only (603.10a's other two classes; 15), none a prevention applied (615.13; 16), entry counters announce nothing (122.6; 17), three variants are dead (18). And sixteen corner cases the survey keeps as questions (§6): CR 513.2's "next", 608.2b's fizzle against "is countered", life loss per source or per batch, 122.6's record, unattach's three routes, the source-less inherent triggers (724.2/725.2/727.1), when a state trigger is re-checked, 603.6a's batch boundary, the entry's `from`, which copies are cast (707.10 against 707.12), one spell naming one permanent twice, 605.1b's stackless mana trigger, "from anywhere" without a frame, delayed triggers' identity and provenance (603.7c–g), game-scoped lookback against the two-turn window, and the two once-per-turn gates (603.2h against "this ability triggers only once each turn"). **The owner's review of the survey (2026-09-18) corrected two "not searched" rows** — Avatar Aang prints CR 603.1b's form and Wan Shi Tong's family prints 603.10a's third class — and added the last three questions. **Step 2 done 2026-09-18 — the doc is `plans/triggers-architecture.md`, and it sizes the phase at six PRs**: TR-1 the spine (the dispatch at a batch's close, the queue on `GameState`, placement in APNAP order with 603.3b's tiers, the stack object with `is_spell: false`; items 1, 3, 7, 9, 10, 18; Soul Warden, Blood Artist, Verdant Force, Wild Growth, Felidar Sovereign), TR-2 the per-player histories and the two once-per-turn gates (P2–P4, S2's `EachPlayer` recipient, "may"), TR-3 delayed, reflexive and "until" triggers (603.7, 603.12, 610.3), TR-4 the look-back list and the widened frame (items 13–15, `Appearance`), TR-5 combat's shapes, targeting, counters, prevention and the multiplier (items 11, 12, 16, 17; 603.2d), TR-6 state triggers and the loop detector's Tier 1 (P1, P5). Every one of the sixteen questions is answered in its §14, the 133 atoms are owed by phase in its §13, and S3 is decided: the instance and the object's epoch join `AbilityIdentity`, and CR 603.7h counts the pair (Ashling the Pilgrim's ruling). The owner reviewed the doc (PR #172) before TR-1. **TR-1 landed 2026-09-19** — the spine: the dispatch at both doors, the queue on `GameState`, placement in APNAP order with the tiers, the stack object, CR 605.4a's stackless mana trigger, `PermanentState.cast`, items 1, 3, 7, 9, 10 and 18 closed; five cards, three pooled (91 → 94); fifty tests, §13's TR-1 row clean; the engine arm `IDENTICAL` with +1.0%/+1.5% CPU per decision (`fuzz-record.md`, TR-1). What the sizing did not predict is `archive/triggers-architecture-landed.md`'s nine notes, and main item 167 is the one wrong answer it filed. **The TR-1 review's five themes and where each went** (PRs #174–#178, 2026-09-20 to 09-22): `codebase-state.md`'s "Found by TR-1" section — item 167 closed by theme E's look-back snapshot. **TR-1b, the dispatch audit, is next, before TR-2** (the owner, 2026-09-22): a slow reference matcher beside the dispatcher, compared at every dispatch in an audited run, with item 174's frame-capture fix as its first commit (`triggers-architecture.md` §4.10, §12). Then TR-2. **TR-2 split at its re-count, and TR-2a landed 2026-09-24** — the histories, the gates and each player: items 122 and 172 closed and 171 half; four cards, two pooled (94 → 96); thirty-seven tests, §13's TR-2a row clean; every gameplay row `IDENTICAL` on the four engine arms, +1.3% CPU per decision at four seats (`fuzz-record.md`, TR-2a). It landed over the band, at +2,875 in code and tests, with "each player" moved forward from TR-2b. **TR-2b is next**: "may", CR 118.12's answer and the `departed` frames. **Item 176's zone-change record design goes between TR-3 and TR-4** (the owner, 2026-09-24, on #185's review): the record's act, performer, redirecting replacement, the object a move made (item 177) and the moment each fact is taken at (item 175), designed as one unit, since seven of the eleven items opened since TR-1 sat on that surface. TR-4 widens the record; no card in TR-2 or TR-3 reads what it settles (§12)

## A6a — the bounded-state PR

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A6a row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-25** (PR #188; `codebase-state.md` items 42 and 179, closed and archived) — **the bounded-state PR** (the owner, 2026-09-25): `codebase-state.md` item 42, no retained log, trigger bindings copying their bound facts at dispatch; item 179, TR-2a's history bounded; the clone probe committed, with CI checks on allocations and bytes per clone; a throwaway probe of the observation cost k, and of a naive redeal (`backlog.md` §2.34) at item 143's checkpoints, with the first decision after it read against a warm and a cold layer memo

### Why here

floors 2 and 3 (`engineering-practices.md` §3.1) wait on it; before TR-2b, so no later phase builds on a whole-log read or a whole-game history

## A6b — TR-2b

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A6b row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-26** (PR #193) — **TR-2b**, whose first commit folds the "your" conditions into one variant with `players: PlayerSet`. **After two fixes, each its own PR.** Item 180's, registry rows shared across a fork: floor 2's CI proxy broke on the budget's own board (✅ 2026-09-25, `codebase-state.md` item 180, archived). Then item 181's, a layers PR: the measurement's brief put a fix ahead of TR-2b if floor 1 broke on a board with a row reaching the hidden zones, and it did (✅ 2026-09-25, LL, `layers-architecture.md` §13e, `codebase-state.md` items 181 and 182, archived)

### Why here

`triggers-architecture.md` §12, re-counted 2026-09-24; it builds the complete `Primitive::Sacrifice` and the wider `EventKindMask` every later phase reads

## A4d — the engine map, then the Rust notes

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A4d row, which keeps the Rust notes, still open, and why they sit there. The two cells, unchanged.*

### Do this

✅ **The engine map, done 2026-09-29** (`plans/engine-map.md`, held to the tree by `plans/check_engine_map.py`), then **the Rust notes**, still open — the post-RE audit's two after-the-passes artifacts (2026-09-15). **Moved up and widened by the owner at CV-2b's design review (2026-09-29)**, to get bearings across triggers, copies and replacement at once. The map is `engineering-practices.md` §7.1's tier 3: the structure (the modules and what each owns, the chokepoint's arms, the three gate legs, the look-ahead frame, the decision sites `codebase-state.md` main item 40 tracks, and the ten-line check that its arm list matches the enum), and now also one event's path from proposal to triggers, naming the subsystem that owns each step and every seam where one reads another's output, each linked to its architecture doc section and trace page. The notes are `plans/references/rust-through-the-engine.md`: the same tour annotated for a reader coming from another language — why the chokepoint is a function and not a trait, what the borrow checker forced (the accessor pair, `FrameCache`'s overlay in place of a clone, `ActionContext`'s plumbing), where `Arc` sits and why, what `Cell` buys `Diagnostics`, the `test-support` feature in `Cargo.toml`

### Why here

Next after CV-2b, ahead of the rest of A6c (the owner: "probably soon"; movable). The map first, since the notes annotate it, and the notes any time after. Before the triggers phase (A6d), so it is not the first subsystem the map absorbs after the fact; a day to draw, then minutes per refresh (§7.1)

## A6j — items 161, 164 and 192

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A6j row, which keeps its date, its PR and what it delivered. The two cells, unchanged.*

### Do this

✅ **done 2026-09-30 (PR #202)** — **items 161, 164 and 192, as one PR** (the owner, 2026-09-29; 192 added by the brief): the engine takes `Pass` itself when the priority list is `[Pass]` alone, unless the seat's `SeatMode` says it stops at every priority point (main item 164); full control, a `FullControl<D, R>` switch above a person's stack that also makes the seat stop there, and `AutoYield<D>`, neither shipping without the other (main item 161); a person's canceled action offered again and charged to no budget (main item 192). The dev GUI's one-answer shortcut went with item 164

### Why here

`backlog.md` §2.22's "before the GUI", and all three were back-stopped before A6g: a GUI seat needs auto-yield, the switch and no `[Pass]`-only prompts from its first game. **What the row did not predict, three things.** The migration ran at its count, 72 failing tests and the 148 + 112 expectations behind them, but twelve tests whose subject is every priority grant could not migrate, and four CR 104.1 boards would have passed vacuously, an agent's strict script no longer seeing a grant the CR does not make; so `SeatMode`'s stop is a field any provider may set, beside the person's. The GUI seat's shortcut had answered only `[Pass]`-only priority prompts, 9,330 and 11,187 across 40 seeds a pool, so all of it went. And the code came to 476 lines added, 391 net, against 300–450: the two new decorators are 278 of them, about half of it the trait's forwarding, five methods each. Every gameplay counter `IDENTICAL`, both pools, two seats and four; instructions per decision −0.75% (`fuzz-record.md`)

## A6c — CV-2a, the entry state, CV-2b, then CV-1b with item 10

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's A6c row, which keeps CV-1b with item 10; its why cell stands. Its Do this cell, unchanged.*

### Do this

**CV-2a, the entry state, CV-2b, then CV-1b with item 10 (CR 400.7)** — CV-2a ✅ 2026-09-28 (`copy-effects-architecture.md` §7b); the entry state ✅ 2026-09-28 (RG, `replacement-architecture.md` §3.5); CV-2 was split in two at its design review, one PR per card; CV-2b ✅ 2026-09-29 (`copy-effects-architecture.md` §7c, its vocabulary §4.1a). **The entry state** was added at CV-2a's review (the owner, 2026-09-28): CR 614.1c's "enters with" and "enters as" formalized as one shape, with one constructor for the entering permanent, a table of what each `EnterMods` field feeds for the prompt-skipping check, a `CharacteristicEdit` vocabulary whose first placement is Master Biomancer's Mutant (`backlog.md` §2.30), "enters untapped" with one printed card (§2.40), and CR 306.5b gathered rather than seeded (`codebase-state.md` main item 186). Designed first, for review; CV-2b then places the same edits inside a copy

## B1 — cost modification

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's B1 row, which keeps CP-1 and why it sits there. The two cells, unchanged.*

### Do this

**Cost modification** — `plans/cost-architecture.md`: **CM-0 to CM-4 ✅** — CM-0 and CM-1 2026-09-07, CM-2 (the spell's own cost abilities) and CM-3 (lock-in's payment side) 2026-09-07, CM-4 (the mana window and the payer) 2026-09-08; **CP-1 (payment) remains**, sized there

### Why here

commander tax runs through it; RS-4 reads better after it; the 25 unjudged CR 601 atoms get their verdicts here (§7); CM-3 and CM-4 went before A6 because the Ironworks loop is item 6's integration test, and CP-1 goes any time

## B6 — CV-3 … CV-7

*Evicted 2026-10-02 from `plans/roadmap-v2.md` §3a's B6 row, which keeps CV-3 to CV-7; what it said about CV-2, which landed on row A6c, is cut. Its why cell, unchanged.*

### Why here

CV-7 (merging, CR 729) is back-stopped before Phase 8 because a multi-component permanent is a fact every later phase would otherwise code against **Added 2026-09-15 (pass 4):** CV-2 is unblocked (RC-2 and RC-4 landed) and first in the chain CV-2 → main item 10 with CV-1b → RS-2; CV-4 is free at any point and its 39 CR 707.10b clauses are item 6's; CV-6 after CV-5 and unsized on purpose (count `put_on_stack.rs`'s alternative-cost sites first), with "Before Layers" item 10's 1a/1b split and `replacement-architecture.md` §8a's turned-face-up event kind; CV-7 its own design pass. What the doc owes each: its LKI section decides against item 10's shape — the `Fixed`-set prune and `target_epochs` — whether or not item 10 has landed (CV-1b); CR 707.5's ETB-of-a-copy becomes a test (CV-2); CR 726's day/night reads the previous turn's spell count, P3's tracker (CV-5); which reads key on the permanent and which on a component before CV-7 (CR 729.3d). Atoms: `copy-effects-architecture.md` §8's 101 (2026-08-29), four of CR 707's partial since CV-1 — CV-1b's slice is 707.2, 707.3 and 707.7; CV-2's 707.5, 707.6 and 707.9, thirteen with the two composites; CV-4's the nine under 707.10–707.12; CV-5's the 34 of 712 plus 710's 3; CV-6's 708's 10; CV-7's 729's 19 plus 712.4's 3; no 707.1 atom exists for CV-3. **Added 2026-09-28 (RG):** CV-6 carries `codebase-state.md` item 187, since its face-down status is the first status the layer walk reads by design: a status the walk reads bumps the layer epoch when it changes, the tapped status included.
