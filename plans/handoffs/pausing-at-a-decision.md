# Handoff — `pausing-at-a-decision`, the design note

**Opened 2026-11-01**: step 4 of `roadmap-v2.md`'s "From triggers to v0.5",
its row in `roadmap-v2.md#the-rows-route-review-added-2026-10-29`. This file
is the design note, reviewed before anything is built. Once the owner takes
its decisions they are folded into `setup-architecture.md` as a section of
their own, which owns the save and the decision log's replay, and this file is
deleted in the same PR. Every count below was read against `91b182a2`, `main`
at `159f92bc` (#305's merge) plus this branch's first commit, which merged
item 360 into item 336. The build comes after v0.5, with the AI harness;
decision 4's part is built before the decks. **Revised at #306's first review
round (2026-11-01)**, where the owner asked questions and took no decision
yet: decision 1 recommends B, and its clone cost, which the first draft set
against C alone, is B's too; decision 2 measures D's surface and lists its
costs beyond the rewrite, and says no dev GUI code moves; decision 3 says
what it is under D; decision 4a walks through Bridge from Below. **At the
second round** the owner took decision 1 (B), leaned to B for decision 2 and
asked for B against D in full, which decision 2 now carries; decision 3 is
rewritten from the start; decision 4a keeps what the engine compares, what
the log writes and what a person sees apart, so no option is shown or logged
as an id number.

**What changes in a game: nothing a player sees.** A game becomes something
that can stop at any question, be copied there, and go on from an answer.
Three customers need that and v0.5 has none of them: a search that copies the
game at a decision to try each answer, a seat over a network whose player
takes minutes, and a harness running many games on one thread.

---

## 1. Where it sits

**Not on the event path**: no proposal, replacement or trigger changes. It is
the boundary every step of the path asks its questions across
(`engine-map.md#11-the-path-in-one-sketch`): DECIDE's CR 616.1 choice,
PLACE's trigger order and targets, RESOLVE's choices, and outside the path
the priority loop, casting's CR 601.2 announcements and combat's
declarations.

```
the rules code: priority, casting, combat, the batch's DECIDE, PLACE, RESOLVE
  └─ ui::ask — 29 functions, called at 38 places in 15 files: each validates the answer, logs it, traces it
       └─ &dyn DecisionProvider — pick_n, pick_number, allocate, choose_ordering, and seat_mode
            └─ 47 implementations: 19 in the engine, 25 in tests, 3 in the dev GUI
```

**Today the engine drives.** It calls the provider from inside its own call
stack, so whatever the rules code is in the middle of when it asks (a cast's
announced costs, the replacement effects a batch has applied) is a local on
that stack. The dev GUI's game waits on a thread of its own: `GuiSeat`
(`devgui/src/bridge.rs:405`) blocks it on a channel at every prompt, which
serves one game a window. A save reloads by replaying its decision log
(`setup-architecture.md#72-the-decisions`, decision 6: a start and every
answer, each written as what was chosen). A copy of the game resumes only at a
priority round's start (`Game::resume_turn_at_priority`, built by A4h, the PR
that made such a copy replay its original, 2026-09-16),
and `tests/priority_fork_test.rs` proves that one: every such copy, played on,
replays the original.

---

## 2. The intake

`check_state_of_play.py --slotted pausing-at-a-decision` listed four items;
the first commit merged item 360 into item 336, leaving three.

| Item | What it is | Taken by |
|---|---|---|
| 287 | A save writes each cast by its card's name, and draws, deaths, attacks and damage only as counts, so a card that narrows one of those counts by what its objects were reads rows a reloaded board does not have | decision 1, which closes it with the writer change it chooses; slotted to the build after v0.5 |
| 336 | CR 616.1's replacement choice and CR 603.3b's trigger order offer each effect as its source, so two effects of one source are one option twice, told apart only by place. Item 360 (#305's filing of the same gap with the trigger order, the thirteen scripted answers that name such options by place, and the typed matcher the owner chose at #305) is merged into it | decision 4; re-slotted to `modes-plus-choice-kinds` |
| 361 | `RecordingDecisionProvider::picking(n)`, a test policy that takes option n of every list, has 33 uses in tests that fail when the lists are reversed | decision 4; re-slotted to `modes-plus-choice-kinds` |

Read as the build's subject, not this PR's, as the brief says: item 40 (the
decision-site invariant, "no decision site may hold outcome-bearing state that
is not in `GameState`", and its two violators, the CR 614.5 sets and the CR
601.2 announcement), item 140 (a priority round cannot be resumed mid-round,
since its pass count and current player are loop locals) and item 233 (three
prompts in the trigger code that hold what they have decided off `GameState`).
Decision 3 says which of them the build still owes.

---

## 3. Counted

### 3.1 The tree

| | Count | Where |
|---|---:|---|
| `ask_*` functions | 29 | `mtgsim/src/ui/ask.rs` |
| their call sites | 38, in 15 files | 14 under `engine/`, and `state/game.rs` |
| `ChoiceKind` variants | 31 | `ui/choice_types.rs:39` |
| `ChoiceOption` variants | 13 | `ui/choice_types.rs:495` |
| `DecisionProvider` implementations | 47: 19 engine, 25 tests, 3 dev GUI | 10 of the engine's 19 are decorators that forward to an inner provider; the 3 in the dev GUI are `GuiSeat`, `HandOver` and `AtTheQuestion` |
| scenario files | 14, every one with a `step` line | `mtgsim/scenarios/` (6), `devgui/tests/scenarios/` (8) |
| files that parse a scenario | 11 integration-test files, `fuzz_games`, and the dev GUI's editor, bridge, session and two test harnesses | |

The brief's numbers hold: 29 sites, 31 kinds, 13 option shapes, 47
implementations.

### 3.2 The rules

- **CR 117.3–117.4**, a priority round: a player who acts receives priority
  again, and the round ends when every player in the game passes in
  succession. A priority prompt asked for the first time in its window holds
  nothing outcome-bearing on the stack. Item 41 (a fork at a priority
  boundary, and the test that requires it to replay the original) proves it
  for a round's start, and item 140 says the same of the rest of a round. So
  it is where a copy of the game can resume from: a round's start today, any
  first ask once item 140's entry point lands. This note calls it an
  **anchor**.
- **CR 601.2a–i and 732.1**: casting is a sequence of announcements, and an
  action that cannot be completed is reversed. The reversal is why the
  announcement lives on the stack today: nothing is committed until 601.2i.
- **CR 616.1 and 603.3b**: the affected player chooses among replacement
  effects, and a player orders their triggered abilities. Both offer effects,
  and two effects can share a source (decision 4).
- **The rules that watch a pending decision.** CR 722.3 and 722.5: while a
  player controls another, the controlled player is still the one whose
  decision it is, and the controller makes it. So a question names whose
  decision it is, and which seat answers is routed. No plan owns CR 722 yet;
  this branch files it (§8). CR 728.1a: a subgame "temporarily discontinue[s]"
  the main game, which resumes when the subgame ends. That is the CR's own
  pause, inside a resolution (§8).

### 3.3 The printed cards

**Decision 1's families**: what a turn's history narrows by object. Scryfall,
`game:paper -is:funny`, unique cards, 2026-11-01:

| What the card reads | Cards | Example | Held today |
|---|---:|---|---|
| a death this turn | 97 | Bloodcrazed Paladin | `ControlledCreaturesDied`, a count the scenario writes |
| …narrowed by what died (nontoken, a type, put into a graveyard from the battlefield) | 13 | Caller of the Claw, Gadrak, the Crown-Scourge | the count only: a reloaded board has no rows to narrow (item 287) |
| a draw this turn | 37 | Archmage Ascension | `CardsDrawn`, a count |
| an attack this turn | 108 | Alesha, Who Laughs at Fate | `AttackersDeclared`, a count; an object's attack is reported, not written |
| damage dealt this turn, by or to an object | 126 | Aggravate | reported for the turn's combat; rows not written |
| damage or life to a player this turn | 225 | Acolyte's Reward | `DamageTaken`, `LifeLost`, counts |
| an object that entered this turn | 101 | Armory Mice | `arrived this turn` sets the entry turn; no entry row |
| a permanent that left the battlefield this turn | 20 | Elegy Acolyte | not written |
| blocked or was blocked this turn | 9 | Joven's Ferrets | reported |
| became a target this turn | 2 | Runesword | reported |
| a permanent that attacked during its controller's **last turn** | 4 | Giant Turtle, Goblin Rock Sled, Halls of Mist, Tangle Kelp | not written, not reported |
| a player who attacked you during their **last turn** | 3 | Avenge, O-Kagachi, Vengeful Kami, Weathered Sentinels | the count of attackers declared, with no player attacked |
| a creature that entered **since your last turn ended** | 1 | Premature Burial | `arrived turn N`, the permanent's entry turn |
| a cast of a commander from the command zone **this game** | 21 | Commander's Insignia, Liesa, Shroud of Dusk | a field whose word waits for `command-zone-casting` (`setup-architecture.md#52-later-and-the-growth-contract`) |
| the first spell, or a named spell, cast **this game** | 3 | First Contact, Once Upon a Time, Approach of the Second Sun | the spells-cast count, and `cast <card>` |
| what an object has dealt damage to **this game** | 1 | The Fallen | not written, not reported |

Queries: `o:/died this turn/`; `o:/graveyard from the battlefield this turn/`
and `o:/(nontoken|artifact|legendary|enchantment|[0-9] or greater)[^.]* died
this turn/`; `o:/(drawn|drew)[^.]* this turn/`; `o:/attacked[^.]* this turn/`;
`o:/dealt damage[^.]* this turn/`; `o:/(damage|life)[^.]* (dealt to|you've been
dealt|you were dealt|opponents? (was|were) dealt)[^.]* this turn/`;
`o:/entered( the battlefield)?[^.]* this turn/`; `o:/left the battlefield this
turn/`; `o:/(blocked or was blocked|that blocked|which blocked|creature
blocked)[^.]* this turn/`; `o:/(became the target|targeted)[^.]* this turn/`;
`o:/(died|entered|attacked|drew|dealt damage)[^.]*(last turn|since your last
turn)/` (8, split by hand into the three rows above);
`o:/(died|entered|attacked|drew|dealt damage|cast)[^.]*this game/` (25, split
the same way). Rulings read: Gadrak's ("counts creatures that died during the entire
turn, even ones that died before Gadrak entered") says the rows predate the
reader, which is why a board must carry them.

**Decision 4's families**: two effects of one source offered in one question.
The same date and filter:

| How one source comes to offer two effects | Cards | Example | Held today |
|---|---:|---|---|
| two triggered abilities of one object, triggering together | — (a shape, not a phrase) | Bridge from Below: its ruling says "you choose the order in which Bridge from Below's triggered abilities resolve" | `OrderTriggers`' options are `Object(source)` twice |
| one ability triggering twice on one event | 337 | Soul Warden with two creatures entering | the same, asked only where the order can change the game |
| a granted triggered ability beside a printed one | 259 | grants of `"whenever …"` | the same |
| a granted replacement ability, or a keyword that is one | 30 | Jund's devour 5 on Thunder-Thrash Elder (CR 614.13b's example) | `ChooseReplacementEffect`'s options are `Object(source)` twice |
| a counter's replacement beside the permanent's own | 72 | finality counters (44), shield counters (28) | the same |
| one source's ability activated twice, two rows | 58 | Words of Worship: its ruling says "you can choose which one to apply" | the same |
| a copy of an object with either | 365 | Spark Double | the same |

Queries: `o:/whenever (another creature|a creature|another nontoken
creature)( you control)? (enters|dies)/`; `o:/(have|has|gains?) "(whenever|when|at
the beginning)/`; `o:/(have|has|gains?) "(if|as)[^"]* would/` (10) and
`o:/(have|has|gains?) (devour|undying|persist|riot|amplify|unleash|modular|bloodthirst)/`
(20); `o:"finality counter"`; `o:"shield counter"`; `o:"the next time"
o:/\}[^.]*: /`; `o:/(enters|enter) as a copy|becomes a copy|create a token
that's a copy/`. So two options from one source is the common case of a busy
board, not an edge.

**CR 722's family**, for the routing in decision 2: `(o:/(gain )?control (of
)?(target|that|each|an|the other) (player|opponent)/ or o:/controls the other
during/) -o:"control of target creature"`, 12: Mindslaver, Sorin Markov,
Emrakul, the Promised End, Opposition Agent among them.

Decisions 2 and 3 have no printed family: they are the engine's interface and
where its state is kept, which no card reaches. Their customers are the 31
choice kinds and every one added later, counted above.

---

## 4. Measured

A throwaway probe outside the tree (a crate in the session's scratchpad,
depending on `mtgsim` by path; release, `MTGSIM_HASH_SEED=1`): `fuzz_games`'
own deals (`random_deck`, `RandomStreams`), its default stack
(`ManaWindowStop` over the random agent), 20 games a board from seed 12345.
It times the engine between prompts and asks two questions of every game.

**What re-running from the last anchor would cost** if a game, instead of
keeping its stack, re-ran from a copy taken at its last anchor through every
answer since, at every prompt that is not an anchor (decision 2's option C).
**What a copy at each anchor costs**, timed by cloning the state there.

| Board | Prompts a game (priority) | Engine a game | Re-running from the anchor | A copy at each anchor |
|---|---|---:|---:|---:|
| 2 seats, 60 cards, `performance` | 185 (66) | 5.6 ms | **+70.8%** of the game, 2.8 answers re-played a prompt | 1.13 µs, +1.3% |
| 4 seats, 60 cards | 326 (115) | 15.1 ms | **+79.1%**, 2.8 | 1.63 µs, +1.2% |
| Commander scale: 4 seats, 100 cards, 40 life | 477 (166) | 31.1 ms | **+77.4%**, 2.9 | 1.70 µs, +0.9% |

Anchors at round starts only, what the tree resumes from today, read the same
within 5 points (+71.4%, +82.4%, +82.9%), since with this stack the engine
takes a pass itself wherever `Pass` is all a seat can do: at Commander scale 2
of a game's 166 asked priority prompts are mid-round.

Two readings follow. **Re-running to step a game is out**: +77% of a
Commander game's CPU would take floor 1 (decisions per loaded core-second,
`engineering-practices.md#the-floors`) from 15,180 to about 8,500, under its
10,000. **Re-running to fork a game is cheap**: one fork at a prompt that is
not an anchor costs its prefix once, 35 µs at two seats, 58 at four and 79 at
Commander scale, about one decision's engine time (65 µs), plus a 1.7 µs copy.

Sizes, for the clone costs below: `GameState` 2,752 bytes, `StackEntry` 392,
`PendingTrigger` 240, `ChoiceContext` 152, `GameAction` 136, `ChoiceKind`
104, `ChoiceOption` 56, `ReplacementInstanceId` 40. Floor 2's latest reading
is 39 allocations and 5.6 µs a clone against 64 and 10; floor 3's 8.7 KB
against 128.

---

## 5. The decisions

Decision 3 follows from decision 2's answer, and decision 1's writer keeps
its copy where decision 2 puts one.

### Decision 1 — what a board written from a game says, and where a scenario starts

**The audit.** What the engine records, against what a scenario writes and
what the dev GUI shows (`setup-architecture.md#5-the-vocabulary-sized` has the
grammar):

| What the engine records | The scenario | The dev GUI |
|---|---|---|
| plain state: zones, a permanent's status, counters, damage, attachments, combat roles, life, player counters, lands played, commander damage, a player who has left | written | shown; the editor has a control for most, and its typed field writes the rest |
| the history's counts, nine facts over this turn, last turn and the game | written as counts | shown as text rows, written by the field (`backlog.md`'s "2.42 The board editor's other advanced settings") |
| a cast's row, with the spell's frame as it was cast | `cast <card>`, reported when the printed card would not reload as it was | a text row |
| an ability's counts this turn (triggered, resolved, once-each-turn) | `this turn:` lines | a text row |
| a draw's, a death's, an entry's, a departure's, damage's, an attack's, a block's and a targeting's row, each naming its objects | counted (draws, deaths, damage, attacks) or nothing (entries: `arrived this turn` sets the turn); this turn's combat and targeting rows reported | not shown: the why panel and "what happened" read the trace's events |
| last turn's rows, by object | nothing, and nothing reported | not shown |
| the stack, waiting triggers, a resolution's rows, delayed triggers, "until" returns, extra turns and phases | reported: played, never written | shown |
| the mana pool | reported, waiting for item 33's provenance (what a unit of mana remembers of its source) | shown |

So a board written mid-turn loses what this turn's rows say about objects,
and item 287's sizing adds a word per fact. The owner's question at #305
(are setup lines that granular, or does a scenario start at a turn's beginning
and play the decisions to the point?) sets the other road beside it.

**A. A word per narrowed fact** (item 287 as sized). `died <card>`,
`drew <card>`, `entered <card>`, `left <card>`, `<card> attacked`,
`<card> blocked`, `<card> dealt N damage to …`, `<card> became a target`, each
a history row stated with its card's printed frame, as `cast <card>` is: ~40
lines with its test, ~300–450 for the eight, plus B's three words below for
the spans before this turn, and a text row each in the editor. *Upkeep*: the
grammar grows with the history log, every phase that
records a new fact adds a word, and a token's row cannot be named, since no
registry holds a token. Setup lines stay as they are.

**B. A scenario starts at a turn's beginning, and the turn is played.** A
board is the state before the untap step. CR 514.2's cleanup has ended every
"this turn" effect and removed all damage, the stack is empty and combat is
over, so nothing about the turn exists to be written. What happened since is
played: by the window's seats, a test's script, or, in a board saved from a
game, the decision log's answer lines. `step` stays, meaning every seat
passes, declaring nothing, until that step's first priority, where the setup
lines play, so a main-phase board reads as it does today with its draw step
drawn. `attacking`, `blocking`, `blocked`, `damage N`, `dealt first-strike
damage`, `arrived this turn`, the `this turn:` lines and the "this turn"
counts go, and with them the loader's refusals of states no play reaches (an
attacker in a main phase, a blocker of a creature not attacking, `setup-architecture.md#41-how-a-bad-board-fails`).
A board **written from a game** is that turn's beginning plus the turn's
answers since: a save, a scenario and a log, which `--load` already replays.
*Code*: the copy is decision 2's paused game's,
taken on its own stack between two `run_turn` calls (item 143's checkpoint:
the state just before a turn runs), beside its anchors, so nothing new hooks
`GameState` and one thing keeps "a copy and the answers since". The event
stream's `TurnBegin` record cannot serve, since a record is read after the
fact and a copy must be taken at the moment. `Scenario::write` given that copy
and the answers since it writes the pair; the dev GUI's "Save board as
scenario" writes the board and its save beside it, and the editor opens the
board. A turn's beginning still carries the spans before it, and §3.3 counts
what they narrow: three words, `attacked last turn` on a permanent's line (4
cards), `player q last turn: attacked player p` (3), and `<card> has dealt
damage to player p` over the game (1, The Fallen). Premature Burial's entry
is `arrived turn N`'s already, and the 21 commander-cast cards wait on the
field `command-zone-casting` counts. ~200–330 lines in the engine with tests,
~50–100 in the dev GUI. *Migration*: the mid-turn words are in 9 of the 14
scenario files (`attacking` 7, `this turn:` 7, `blocking` 4, `damage` 3,
`arrived this turn` 2), in the inline boards of four integration-test files
(SU-1's loader tests hold most of them, then SU-7's, A6r's and TR-5h's), and
in the board editor's mid-turn controls. ~400–700 lines changed, more deleted
than added: an estimate from those counts, not a measurement. *Upkeep*: the
writer's report shrinks to what outlives a turn (delayed triggers, returns, a
resolution's long rows), and the grammar holds only the state a turn's
beginning holds.

**C. Both doors.** B, and a hand-written board also keeps today's mid-turn
grammar, any step's round start, with none of B's migration. *Upkeep*: the
mid-turn words, their editor controls and the loader's refusals stay, for
boards the writer no longer produces.

**Clone cost**: none for A. For B and C the same: one copy a turn, kept by
the paused game, at the probe's 1.1–1.7 µs, a few microseconds in a game of
tens of milliseconds, and only in a game that saves boards. (The first draft
set this against C alone; B's writer needs the same copy.)

**Setup lines and alike options.** Under B or C, setup lines stay what they are
(an action, its targets, its X, its ability's place), since what happens
later in a turn is the log's lines, not setup's. Whether a setup line refuses
an option that reads like another: under decision 4 no two offered options
read alike, but for two offered costs alike in every word, which are one cost
twice (decision 4b). So one rule holds for a script, a setup line and a
replay: an answer naming what two offered options are takes either when they
are one cost twice, and is refused as an engine bug otherwise.

**What the pair does with what outlives a turn.** A delayed trigger made last
turn ("at the beginning of your next upkeep"), an "until" return still
waiting, an extra turn: the board cannot write them
(`setup-architecture.md#52-later-and-the-growth-contract` owns that list, and
says each is played), so the writer reports them at the top of the file as it
does today, and the pair's replay stops at the first answer the reloaded
board no longer reaches, naming it (`setup-architecture.md#72-the-decisions`,
decision 6). The pair is exact for the turn in progress less what that report
names.

**Taken: B** (the owner, 2026-11-01, at #306's second review round). The
recommendation, revised at the first round, which asked what C buys over B: C buys only the hand-written mid-turn door, and
the dev GUI's use says a situation is reached by playing it, not by writing it
mid-turn. B closes item 287 with three words for the spans before a turn,
where A needs those and eight more, it shrinks the grammar and the loader,
and a board saved from a game is exact for the turn in progress but for what
its report names. The performance is the same for B and C.

### Decision 2 — the `DecisionProvider`'s shape: which drives which

**The problem.** A paused game returns a question to whoever drives it and
goes on from an answer. Today the engine drives: it calls the provider from
inside its stack, so the stack is where a question's in-flight state lives.
Item 140 measured a thread blocked on a channel, the dev GUI's mechanism, at
1.1–1.7 µs a decision on Windows and 12–13 µs on a Linux VM, against 64 µs
of engine a Commander decision. A search forks at any decision, a network
seat waits minutes, a harness runs many games on one thread.

**A. As today: the engine drives, and each client brings its own thread.**
The dev GUI's `GuiSeat` and channel, written again by every client. A copy
resumes only at an anchor. *Code*: none. *Upkeep*: each client's bridge is its
own (the dev GUI's `spawn_game`, `GuiSeat` and `HandOver` are ~330 lines), and
many games on one thread is out.

**B. The engine ships the paused game: the client drives it, and the engine
drives its providers inside.** A `PausedGame` runs the game on a stack of its
own and hands the client where it stopped: `start(game, seats) -> Paused`,
`answer(Answer) -> Paused`, where `Paused` is `Asking(Question)`,
`Ended(GameResult)` or `Halted(Halt)`. The third is the engine's existing
`Halt` (`state/game.rs:414`): the run's own error, or a `Stop` a provider
raised (a replay that diverged or ran out, a setup line refused, a replay
superseded), which a fork's replay and a scenario's setup lines can both
raise inside a paused game. A halted paused game refuses `answer`, as every
run entry refuses a stopped game today (`setup-architecture.md#72-the-decisions`,
decision 1). The question is the four primitives' arguments, owned: the
player whose decision it is, the `ChoiceKind`, the last rejection, and the
options with their bounds, the number's range, the buckets with their
limits, or the items. **No dev GUI code moves into the engine.** The
question and answer types are written new in the engine, at its standards
and under its review; they say what `devgui/src/prompt.rs`'s `Primitive`
(which method asked, with its bounds) and `Answer` (picks, a number, an
allocation, an order) say today, so the dev GUI's PR deletes those two and
uses the engine's, and nothing has two spellings. Inside, the engine is
unchanged: the seats a client answers are one new provider that hands the
question out and waits, beside any decorator (`ManaWindowStop` over it closes
a paid window without asking anyone). The stack is suspended on a thread
first, std only: the same technique the dev GUI's bridge uses, written again
in the engine rather than moved, since the bridge was built as a debugging
tool and never reviewed to the engine's bar. It moves to a stackful coroutine
once item 140's spike, with the AI harness, picks one. A client that forks asks the paused game to keep a copy
at each anchor (`with_forks()`): `fork()` copies the last one and replays the
answers since, each recorded as the typed option it chose (decision 4), not
its place (§4: 1.7 µs at an anchor, plus 35–79 µs at any other prompt). An
observation is built on the game's stack at the question, by a function the
client gives, so nothing copies the state to show it. *Code*: ~370–600 lines
and ~350–550 of tests (§6).

**C. The client drives, and the engine re-runs.** No suspended stack: each
answer re-runs from a copy at the last anchor through the answers since, and
stops at the next question. *Code*: ~200–300 lines, no thread and no
coroutine. *Performance*: §4's +71–83% of a game.

**D. The client drives an explicit state machine.** Every frame between the
turn loop and a prompt kept on `GameState` as a resumable state (the priority
round, the cast's sequence, the batch's DECIDE loop, a resolution's place in
its instructions, placement, the state-based checks, combat), so a copy at any
prompt resumes by itself, and a paused game is a `GameState` and nothing
else. *Code*: measured on the tree, 150 functions in 29 files take a provider
or a context that carries one, in ~23,600 lines of engine code outside tests;
the core of it is `resolve.rs` (41 functions), `actions.rs` (25),
`replacement/pipeline.rs` (11), `turns.rs` (9), `state/game.rs` (8),
`triggers/dispatch.rs` (7) and `put_on_stack.rs` (6). Each such function
becomes a step that saves its locals into a frame and returns when it needs an
answer, and the recursion (the effect walk, a batch inside a batch, a rider's
own resolution, a dispatch inside a dispatch) becomes an explicit stack of
frames on `GameState`. No precedent in the tree calibrates the size; RA-1
threaded a parameter through the engine, not its control flow. The order of
magnitude is several band-sized PRs, likely five to ten, and a one-subsystem
spike should size it before it is chosen. *Performance*: a fork anywhere is
one copy, 1.7 µs, where B pays up to 79 µs more at a prompt that is not an
anchor. *Upkeep, beyond the rewrite*:
- the rules code stops reading like the CR: `cast_spell` reads CR 601.2a to
  601.2i top to bottom today (`engine/put_on_stack.rs:77`), and becomes a
  `match` over a cast's step, each arm restoring and saving its locals;
- every later choice kind is written as states and transitions, in each of
  the route's steps to v1 (modes, linked abilities, recipients, permission,
  combat requirements), where under B it is one `ask_*` call;
- a frame cannot hold a borrow of `GameState`, so code that holds a card's
  data across a question today re-reads it by id after;
- the rewrite runs through the code every card resolves through, and the
  route waits for it.
What D buys: a fork anywhere at a copy's price, no thread, coroutine or
`unsafe`, and the shape research APIs ask for (OpenSpiel's `clone` and
`apply_action` at any decision) with nothing in between. B serves that API
too, its `clone` at a prompt off an anchor costing the 35–79 µs replay. A
search's rollout plays the game on: from the middle of a Commander game
(§4: 477 prompts a game) that is ~240 decisions at ~65 µs each, ~15 ms, so
the difference is about half a percent of a rollout.

| | A | B | C | D |
|---|---|---|---|---|
| Per prompt, 4 seats | a thread switch | a thread switch, then a coroutine's (tens of ns); a copy at each anchor, 1.6–1.7 µs, only with forks | +79% of the game | nothing |
| Per prompt, 2 seats | the same | the same; 1.1 µs at each anchor | +71% | nothing |
| A fork | at an anchor only | anywhere: 1.7 µs, plus 35–79 µs off an anchor | anywhere | anywhere, 1.7 µs |
| Many games on one thread | no | with the coroutine | yes | yes |
| Floor 1 | holds | holds (−1% with forks on) | **fails**: 15,180 → ~8,500 | holds |

**How the 47 implementations move.** Under B, none moves. The engine gains
one, the seat that hands a question out. A provider in process (the random
agent, a script, the CLI) still plays through `Game::run(&provider)` with
nothing in between, and a paused game answers with one through a single
adapter that calls the right primitive for a question. The dev GUI's
`GuiSeat` and `HandOver` go once its bridge drives a paused game, in the dev
GUI's PR. Under D the implementations keep their signatures too, called by a
driver loop on each question, and what changes is every `ask_*` site and the
150 functions above them.

**What each leaves to decide.** Under B: the suspension (a thread now; a
coroutine against the thread at the harness, by item 140's spike), and
decision 3's choice of what `GameState` keeps. Under D: decision 3 is
forced (everything), but each subsystem's frame is a design of its own in the
rewrite: the cast's steps, the batch's DECIDE state, the resolution's walk,
the dispatch, combat's steps. Decision 2b and a halted game's answer are the
same under both, since a client answers a question either way.

**Who answers.** The question names the player whose decision it is (CR
722.3). The seat that answers is routed by `DispatchDecisionProvider`, which
under CR 722.5 will read who controls that player, a fact `GameState` gains
with Mindslaver's family (§8); under CR 722.8 one seat then answers for two
players. The question carries no answering seat of its own.

**Recommendation: B, on a thread first.** It is the only option that serves
all three customers with no rewrite, it keeps the trait and every
implementation, and its one cost that grows, a fork off an anchor, is paid by
the search that asked for it. D buys a fork up to 79 µs cheaper, about half
a percent of a rollout, for a rewrite of the engine's control flow and a cost
on every later choice kind. C fails floor 1 by measurement.

**B against D, in full** (asked for at #306's second review round). The
client-facing shape is the same for both: start a game, read the question,
answer it, fork it. What differs is how the engine keeps its place while it
waits.

| | B: the paused game keeps its stack | D: an explicit state machine |
|---|---|---|
| **The engine's code** | unchanged; ~370–600 new lines and ~350–550 of tests beside it | 150 functions in 29 files rewritten as resumable steps, their recursion as a frame stack; five to ten band-sized PRs, uncalibrated |
| **Reading the rules code** | stays CR-shaped: `cast_spell` reads 601.2a to 601.2i | each sequence a `match` over its step, saving and restoring its locals; the CR's order is in the transitions |
| **Every later choice kind** | one `ask_*` call where the rule asks | its states and transitions, and the frame of every function it is reached through |
| **Risk of the change itself** | new code beside the engine; the engine's paths are the same | the code every card resolves through, rewritten; regressions there are the expensive kind, and the fixture tables are the net |
| **Determinism** | the engine's code is unchanged, so its invariant is; a fork's replay rests on it, and the fork test checks it | rebuilt with the rewrite, and the same fork test checks it |
| **Per question, in play** | a thread switch: 1.1–1.7 µs on Windows, 12–13 µs on a Linux VM (item 140), against ~65 µs of engine a decision; a coroutine's, tens of nanoseconds, once item 140's spike picks one | a return from the step and a call back in, not measured; but every resumable call keeps its locals in a frame on the heap rather than the machine stack, on every path, asked or not, which is not measured either |
| **A fork** | at a priority prompt, one copy (1.1–1.7 µs); elsewhere, the copy and a replay of the answers since (35 µs at two seats, 58 at four, 79 at Commander scale) | anywhere, one copy, whose frames add to floor 2's allocations |
| **What a fork costs a search** | a rollout from mid-game is ~240 decisions, ~15 ms at Commander scale: the replay is about half a percent of one. A search that expands a tree over the inner questions themselves, with no rollouts, pays about one more decision of engine a node | nothing beyond the copy |
| **A paused game's memory** | a thread's stack (reserved, committed as used) or a coroutine's (sized by the engine's deepest nesting, to be measured); fine for hundreds of games a server, coroutines past that | a `GameState` and its frames, kilobytes |
| **Many games on one thread** | needs the coroutine: a small dependency or a little platform code with `unsafe` at the switch | native |
| **A seat over a network** | the game waits in the server's memory; a lost process replays its saves | the same; a `GameState` could be written to disk instead, once `serde` covers every type it holds, a large job of its own (item 141: `serde` appears nowhere in the tree) |
| **An AI research API** (OpenSpiel's `clone` and `apply_action` at any decision) | served: `clone` at a non-priority question is the copy and the replay | served natively |
| **Debugging** | a panic's backtrace runs through the rules code that raised it | a backtrace shows the step dispatcher; where the engine was is in the frames' data |
| **What exists that it fits** | the typed `Stop` unwind, `Replay`, the decision log, `resume_turn_at_priority`, the fork test | the same pieces, each re-checked against the rewrite |
| **Nesting the CR asks for** | a subgame (CR 728) or a cast inside a resolution (Panglacial Wurm) runs on the stack as any call does | a subgame is a `GameState` held in its parent's frame; each such case is a frame design |
| **Time to v0.5** | nothing before v0.5; one engine PR and one dev GUI PR after it | the rewrite before or beside the route's next steps, or every choice kind the route adds until then written twice |
| **Changing course later** | B to D stays possible behind the same client-facing shape, but each choice kind added meanwhile is one more to convert | D to B would discard the rewrite |

What would make B the wrong call is a search whose work is mostly forks at
inner questions rather than rollouts: there each node costs about two
decisions of engine where D's costs one. What would make D the wrong call is
everything else in the table, which every later phase pays. As far as I know,
open-source engines for complex card games keep the engine-calls-player shape
and give each game a thread, and explicit state machines are the norm in
research frameworks whose games are small enough to write that way; that is
background, not a measurement of either.

**2b. How a client program's answer names what it chose.** This is the
answer a client *program* sends back to the paused game, never what a person
sees: a person sees each option drawn with its name and, for an ability, its
text (decision 4a's three layers), and clicks one. The program then tells the
engine which one.
- **By place in the question it answers**: the trait's shape and the dev
  GUI's `Answer` today, "the third option", `Picks(vec![2])`. The client holds
  the question, so a place in it is unambiguous at that instant. The paused
  game turns each into its typed option for the fork and the log.
- **By identity**: the option itself sent back, as decision 4 types it, found
  by decision 4's matcher, the way a script, a setup line and a replay name an
  option. The paused game turns it into a place for the trait's call, a lookup
  among the question's options. A client that answers late, against a
  question re-derived after a server restarted, names what it meant.

**Recommendation: by identity**: answers are named by what they chose
everywhere a client or a file writes one, and the place stays inside the
engine, between `ui::ask` and the provider.

### Decision 3 — what a pending decision holds on `GameState`, and the rule every later choice kind follows

**What the decision is about, from the start** (rewritten at #306's second
review round, which asked for it). `GameState` is the game as data: the
zones, the permanents, life totals, the stack. Copying it is how a game is
forked. But while the engine is in the middle of something, part of what it
knows is not in `GameState`: it is in the local variables of the functions
running at that moment. That part is the **in-flight state**.

A cast shows it. You cast Lightning Bolt. The engine moves Bolt to the stack
(CR 601.2a), asks whether you pay an alternative cost (601.2b), and then asks
for a target (601.2c). While it waits for the target, `GameState` shows Bolt
on the stack with no stack entry yet; the costs you announced are local
variables in `cast_spell`, and so is the fact that a cast is under way at
all. Copy `GameState` at that question and the copy holds a Bolt on the stack
that nothing describes and nothing will finish casting. A copy taken there is
not a game anyone can go on with.

Four places in the engine hold in-flight state across a question (the table
below): a cast's announcements (item 40's second "violator", item 40 being
the rule that a decision site holds nothing outcome-bearing off `GameState`),
CR 616.1's record of which replacement effects have already applied (its
first), three prompts in the trigger code (item 233), and the priority
round's count of how many players have passed in a row (item 140).

**Why this was going to be a migration, and why B changes it.** The plan
until now, your rule of 2026-10-08, was to move each of these into
`GameState`, so that a copy taken at any question could go on by itself.
Decision 2's B makes that unnecessary. A paused game keeps its own call
stack, locals and all, so pausing and answering never need the in-flight
state in `GameState`. A fork does not copy at the question either: it copies
at the last **anchor**, a priority prompt asked for the first time in its
window, where nothing is in flight (a cast has finished or not begun, the
batch has performed), and replays the answers since, which rebuilds the
in-flight state exactly as play built it.

So the question left is narrow: **which questions count as anchors**, the
places a copy can go on from directly. Today only the first priority prompt
of a round does (`Game::resume_turn_at_priority` starts a round at the
active player). A priority prompt later in the same round, after the active
player passed and the next player is asked, does not, because the pass count
is a local. The candidates, with what each would cost on `GameState`:

| In-flight state | Where it lives | What keeping it on `GameState` would cost |
|---|---|---|
| CR 614.5's applied and declined sets, the exempt slot, the group's rewritten members (item 40, violator 1) | `apply_replacements`' locals, `engine/replacement/pipeline.rs:456` | ~80–120 lines, plus the members, which CV-2a (entering as a copy) added to the sizing; an estimated 3–6 allocations and 0.5–2 KB a copy at a CR 616.1 prompt, none elsewhere (a `GameAction` is 136 bytes) |
| CR 601.2b–d's announcement, before the push (violator 2) | `cast_spell`'s locals, `engine/put_on_stack.rs:121` to `:223` | ~200–300 lines; a `StackEntry`-sized record, 392 bytes and its heap, at an announcement prompt |
| CR 603.3d's targets, 603.7b's cause, 303.4f's host (item 233) | `place_one`, `queue_delayed`, `return_until` | ~100–150 lines; a `PendingTrigger`, 240 bytes, at a placement prompt |
| the round's pass count and current player (item 140) | `run_priority_round`'s locals, `engine/priority.rs:57` | ~60 lines; nothing, since both derive from `priority_player` and the turn order |

**(a) All of it**, item 40's plan with items 233 and 140: ~450–650 lines.
*What it buys*: a copy at one of those questions could go on by itself, but
only where every function above it could be re-entered from `GameState` too.
That holds before a round's first priority (trigger placement, the
state-based checks), and not inside a cast or a resolution: moving Bolt's
announced costs into `GameState` still leaves "which step of the cast is
next" in `cast_spell`'s place in the call stack. Making that re-enterable is
decision 2's D. So (a) pays only under D. *Clone cost*: in the table, at the
questions that hold it; 8 bytes a field elsewhere.

**(b) The round's place only.** A priority round that can start at any seat,
its pass count worked out from who holds priority rather than counted in a
local (item 140, ~60 lines). Then every priority prompt asked for the first
time in its window is an anchor, and a fork there is a 1.1–1.7 µs copy. A
re-ask after a reversed action (CR 732.1: you chose a spell you could not pay
for, and are asked again) is not one, since its question carries what was
rejected; a fork there replays from the window's first ask, which rebuilds
it. Item 41's test (forks at round starts replay the original) gains the
mid-round forks and CR 514.3a's cleanup re-loop. *Clone cost*: nothing.
Items 40 and 233 close as superseded: the paused game keeps that state on its
stack, and a fork rebuilds it by replay.

**(c) Nothing.** Anchors stay at round starts; a fork at a later priority
prompt in the round replays the passes since its start. *Clone cost*:
nothing. By §4, the agents' stack meets 0–2 such prompts a game, since the
engine passes for a seat whose only option is to pass; but a seat that stops
at every priority point (a person, and an AI that acts at instant speed)
meets mostly those, and each of its forks would replay.

**Under decision 2's D there is no choice here.** Every row of the table goes
on `GameState`, and with it everything above each prompt: the resolution's
place in its effect tree, the batch's undecided groups and queued riders, the
cast's remaining steps, the dispatch in progress. That is (a) and the rest of
the rewrite. The 2026-10-08 rule stands and widens from the trigger code's
prompts to every prompt, and the gate is the same fork test.

**(b) and (c) withdraw a ruling of yours.** The rule that a prompt keeps
what it has decided on `GameState` is the owner's (2026-10-08, at A6k's
review, a triggers-phase PR:
`triggers-architecture.md#56-a-prompt-keeps-what-it-has-decided-on-gamestate`),
and the roadmap row gives it as this step's reason: "every choice kind after
it keeps its state on `GameState`". It was right for a design that resumes
from `GameState` alone. §4's measurement is what moves it: stepping a game
from `GameState` alone (C) costs +71–83%, so a paused game keeps its stack,
and then the rule buys a cheaper fork at a few prompts and nothing else. Item
40 named a second customer, a self-hosted server surviving a dropped
connection. A dropped connection loses no state, since the game lives in the
server's process; a lost process recovers its games from their saves, the
start and every answer, replayed (a Commander game in ~22 ms in release,
`setup-architecture.md#71-the-tools-where-they-sit-and-what-the-tree-has`),
which re-derives the question as a fork does.

**The rule every later choice kind follows**, which replaces that rule under
(b) or (c). It is what keeps "copy at the anchor, replay the answers" exact
for every question the route adds (modes, a choice at resolution, an order
the CR hands a player):
1. Every question goes through `ui::ask`, so the paused game hands every
   question out at one place. (Kinds share functions there: 29 for 31 kinds.)
2. Its options are named by what they are (decision 4), and its `ChoiceKind`
   names its subject (`ChoiceKind::subject`, exhaustive).
3. What it offers is a function of `GameState` and the answers since the last
   anchor: never of a provider's state or a hash order, which the determinism
   invariant already says, and nothing it holds outlives the next priority
   prompt. A question may keep its in-flight state in locals, as `cast_spell`
   does; what it may not do is keep state off `GameState` past the next
   priority prompt (a delayed trigger waits on `GameState`, as it does
   today), since a copy at that prompt would not have it.

The gate is the build's fork test (§6), which forks at every decision of
seeded games at two seats and four and requires each branch to replay the
original. It is item 41's test with "every round start" widened to "every
decision". A choice kind that broke rule 3 would fail it.

**Recommendation: (b), withdrawing the 2026-10-08 rule.** It is item 140's
60 lines, it makes rule 3 exactly true at every first ask, and it closes items
40 and 233 without the 450–650 lines whose only use is decision 2's D.
Placement's drain shape (the waiting queue loses an entry only as it is
placed) stays where it is built, since it costs nothing there, and is no
longer required of new prompts.

### Decision 4 — every option named by what it is

**The problem.** A decision log, a test's script, a scenario's setup line and
a client each name an option. Today an option is written as words
(`ChoiceOption::as_logged`), and the one matcher (`position_named`) compares
those words. Two failures follow. CR 616.1's and CR 603.3b's options are their
sources, so two effects of one source read alike, and the log writes `(1 of 2)`
(item 336; §3.3 counts the families). Matching turns on rendering, so a change
to how an option prints is a change to which option an answer means. The
owner chose the typed matcher at #305; what is open is how an effect is named,
and how the two cost shapes are named.

**4a. How an effect is named.**

**The worked example.** Bridge from Below is in your graveyard. A nontoken
creature of yours and a creature of an opponent's die in one event, so both
of its abilities trigger: one makes a Zombie, the other exiles Bridge. Its
ruling says you choose the order, and the order matters: the Zombie ability
checks "if this card is in your graveyard" again as it resolves, so it must
resolve first to make anything. Today `OrderTriggers` offers
`[Object(Bridge), Object(Bridge)]`. A client shows "Bridge from Below" twice,
the log writes `Bridge from Below (#31) (1 of 2)`, and only the list's order
says which ability is which.

- **A. As today**: `Object(source)`, and its place among alike options. A
  replay across builds that lists them in another order applies the other
  effect without diverging.
- **B. Its source and its ability**: `Effect { source, ability }`. This splits
  Bridge from Below's two triggers and the Elder's printed devour from the
  granted one. It does not split Words of Worship's two rows (one source, one
  ability), a counter's effect (no ability) or one ability triggering twice.
- **C. The effect's own identity.** `ChoiceOption::Replacement(ReplacementInstanceId)`
  is CR 614.5's key, which the pipeline already gathers, applies and records
  by: a registry row, an effect of an ability on an object with its place
  among the ability's effects, a counter kind's effect, a game rule's.
  `ChoiceOption::Trigger(TriggerId)` is a waiting trigger's id, minted per
  game in trigger order. **A minted id is exact within a build and says
  nothing across two**: a registry row's id and a `TriggerId` are counters,
  so another build that mints in another order would swap two of them
  silently, the failure A has. So the words carry what makes the instance
  this one, kept as a check the way a card's name is kept beside `#7`: a
  trigger's words name the objects its binding names (what triggered it,
  CR 603.2c's occurrences), and a registry row's name the stack object whose
  resolution made it. Bridge's two become `trigger 14: Bridge from Below
  (#31) ability 1, for Grizzly Bears (#7)` and `trigger 15: Bridge from Below
  (#31) ability 2, for Hill Giant (#9)`; the others read
  `Thunder-Thrash Elder (#12) ability 7 effect 1`,
  `Words of Worship (#9) row 3, made by #41`, `Grizzly Bears (#7) finality
  counter`.
  A build that mints in another order then fails the check and diverges
  loudly. What is left is two instances alike in every word: the same source,
  ability and triggering objects, which is as far as the log's convention
  goes for two cards of one name. Every arm is an id or a small enum, within
  item 141's payload rule (a prompt names things by id and CR vocabulary,
  never an engine AST); a `ReplacementInstanceId` is 40 bytes, under
  `ChoiceOption`'s 56. *Code*: the two arms with their words and labels in
  both clients; `ask_choose_replacement` and `ask_order_triggers` pass the
  instances; the unasked notes (an elided order) name them; the thirteen
  alike answers named outright, and CV-2b's test provider's `then_index`
  (`tests/phase_cv2b_integration_test.rs:103`, its one use at `:1133` on
  `ChooseReplacementEffect`); `expect_choice_among_alike`,
  `expect_order_among_alike` and the log's `(n of m)` go. ~80–150 lines and
  the tests.

**Recommendation: C.** Only C names every family §3.3 counts.

**4b. The typed matcher, and the two cost shapes.** As the owner chose at
#305: `ChoiceOption` derives `PartialEq`; the log reader parses each answer's
words into a typed option once (`Grizzly Bears (#7)` is `Object(#7)`, the
name kept as a check that the id still names that card); scripts, setup lines
and the replay compare typed values; the log's text changes only by 4a's
words. Eleven of today's shapes and 4a's two parse plainly. The two cost
shapes do not:
`AlternativeCost` and `AdditionalCost` carry `Cost` trees (item 141 names them
as the payload rule's other offenders), and their words, `alternative
Flashback {1}{R}`, are a label that prints the keyword and, when every part is
mana, the mana.

The shapes to name: `AlternativeCost`'s seven variants (Flashback, Overload,
Dash, Escape, Evoke, Bestow, `Custom`) and `AdditionalCost`'s seven that are
offered (Kicker, Buyback, Entwine, Casualty, Bargain, Strive, `Custom`; a
`Mandatory` cost is never offered, CR 601.2b).

- **(i) Parse the label back into the tree.** Possible only where the label
  is the whole cost. A non-mana part prints the keyword alone, so Escape's
  "exile five other cards" cannot be read back.
- **(ii) A cost's typed identity is its variant and the variant's own
  values**: the existing enums' variant, and no new enum. The complete CR 702
  `KeywordName` enum that `codebase-state.md` records as wanted (the residue
  of "Keywords are abilities", under "Before Layers" item 10) would name the
  keyword variants again but not `Custom`, so the variant is the one spelling
  that covers all fourteen. For
  a variant carrying `Cost`s, the identity adds the mana among them,
  which the log writes even where the label shows the keyword alone; Casualty
  its N; Bargain and Strive nothing more; `Custom` its text. The rest of the
  words are kept as a check, as a name is. ~40 lines beyond the matcher: the
  variant's word and a mana parser for `{1}{R}`.
- **(iii) Compare the cost shapes by words**, the one textual corner left.

**Two offered costs alike under (ii).** No cost is granted today, so a spell
offers only its printed costs, and none prints two of one variant with one
mana (two kickers differ by mana). Granted costs arrive with
`casting-from-other-zones`. Checked on Scryfall (2026-11-01): Underworld
Breach grants escape "equal to the card's mana cost plus exile three other
cards", so beside Uro, Titan of Nature's Wrath's printed escape it offers
{1}{G}{U} against {G}{G}{U}{U}, and beside Underworld Rage-Hound's {1}{R}
against {3}{R}: both pairs differ by mana. A pair alike in variant, mana and
every other word is one cost twice, and either pick is the same game, which
is decision 1's one exception. `permission-design`, which gives an offered
cost its source, can add the source to the identity then.

**Recommendation: (ii).** The matcher with its parsers, ~150–250 lines and
the tests.

**4c. `picking(n)`** (item 361). It is a test's policy, not a script, and the
33 uses that depend on order break whenever an engine change reorders a list.
`modes-plus-choice-kinds` reorders and lengthens lists (a mode is an option,
and a choice kind gains options). So the 33 move to scripted answers named by
identity in that PR's first commit, at item 361's ~150–250 lines, and
`picking(n)` stays for the 142 uses no order reaches.

**4d. When it is built.** Decision 4 is independent of pausing, and
`modes-plus-choice-kinds` (step 5) already names modes by identity in place of
`StackEntry.chosen_modes`' positions. One log-format change and one fixture
migration is cheaper than two. **Recommendation**: 4a, 4b and 4c are built
in `modes-plus-choice-kinds`' first PR, ~380–650 lines in all, and items 336
and 361 are re-slotted there.

---

## 6. The build, sized

**Before the decks, in `modes-plus-choice-kinds`' first PR** (decision 4):
the two effect arms, the typed matcher with its cost parser, and the 33
`picking(n)` uses, ~380–650 lines with tests. The PR's own sizing places it in
the band. Its consumer is the decision log: a replay of a game with Bridge
from Below's two triggers, both orders.

**After v0.5, with the AI harness** (decisions 1–3):

| PR | Piece | Code | Tests |
|---|---|---:|---:|
| **engine** | the paused game: its question and answer types, the seat that hands a question out, on a thread | ~250–400 | ~200–300 |
| | the fork: a copy at each anchor, the answers since, the replay; its consumer, the fork test at every decision of seeded games at two seats and four, each branch replaying the original's event log and end state | ~120–200 | ~150–250 |
| | item 140's round started at any seat, and CR 514.3a's re-loop; item 41's test gains the mid-round forks | ~60 | ~60–80 |
| | the paused game's copy at each turn's start, the writer's turn-start save, and decision 1's three words for the spans before a turn (closing item 287) | ~140–230 | ~60–100 |
| | decision 1's B: `step` re-read as the step to pass to, the mid-turn words and their refusals out of the grammar and the loader, the engine's six scenario files and four test files' inline boards migrated | ~50–100 added, more deleted | ~100–200 changed |
| | items 40 and 233 closed as superseded | docs | |
| | **the engine PR** | **~620–990** | **~570–930** |
| **dev GUI** | the bridge drives a paused game (`spawn_game`'s thread, `GuiSeat`, `HandOver` go); "Save board as scenario" writes the turn's beginning and its save; the editor's mid-turn controls go and its eight test scenarios migrate | ~250–450 | ~100–200 |
| with the harness | the coroutine, chosen by item 140's spike against the thread, with the binding floor item 140 proposes | sized there | |

The engine PR reads ~1,190–1,920: inside the band (1,500–2,500,
`engineering-practices.md#sizing-a-phase`) at its upper estimate, under it at
its lower, so it is one PR. The paused game and the fork are one consumer's
two halves. Fuzz A/B: `IDENTICAL` predicted. Every fuzz game runs
`run_priority_round`, which item 140 rewrites to start at any seat and derive
its pass count; the derived count is today's at every round, since a round a
fuzz game plays still starts at the active player, so the gameplay rows and
the cost counters read byte-identical, which is what the A/B checks. The
paused game is a path no fuzz game takes: `fuzz_games` keeps calling
`Game::run` directly.

---

## 7. What lands where, once the decisions are taken

- `setup-architecture.md` gains a section, "Pausing at a decision", holding
  the decisions as taken, the audit's table, §3.3's census with its queries
  and date, and the build's sizing; its status block and its section 11
  ("Out of scope") are updated; this file is deleted.
- `fuzz-record.md` gains a block for §4's probe: the boards, the seed, the
  re-running and copy readings, newest first.
- `roadmap-v2.md`'s "The excluded tail" gains §8's finding 4 on its
  Subgames line.
- `codebase-state.md`: item 287 re-slotted to the build after v0.5; items 336
  and 361 re-slotted to `modes-plus-choice-kinds`; items 40, 140 and 233 gain
  a dated line pointing at the new section (closed at the build if (b) is
  taken).
- `triggers-architecture.md#56-a-prompt-keeps-what-it-has-decided-on-gamestate`
  points at decision 3's rule.
- `roadmap-v2.md`'s `pausing-at-a-decision` row says what the build is, and
  the `modes-plus-choice-kinds` row gains decision 4.
- `permission-architecture.md` gains the line about an offered cost's source
  (decision 4b).

---

## 8. Found while designing

1. **Re-running a game from an anchor at every answer is measured, not
   argued** (§4): +71–83% of a game. Item 140 had set "a replay per step" aside
   as re-simulating too much, meaning a replay from the game's start; from the
   last anchor it is still too much for stepping, and cheap for a fork.
2. **The agents' stack asks almost no mid-round priority prompts**: 0 of 66 a
   game at two seats, 1 of 115 at four, 2 of 166 at Commander scale, since the
   engine passes for a seat whose only answer is `Pass`. Item 140's 1,827 of
   2,799 were read with every seat stopping at every priority point. Its
   sizing stands; its weight is a person's or an instant-speed agent's.
3. **CR 722, controlling another player, has no owner.** 12 cards (§3.3), and
   `cr-coverage-audit.md` already notes Emrakul, the Promised End waiting on it.
   Filed as `backlog.md`'s "2.51 Controlling another player" on this branch,
   with the routing of a question to the seat that answers (decision 2) as
   one of its two halves: CR 722.5's controller answers, 722.8's controller
   still answers for themself, so one seat answers for two players, and
   722.7's controlling effect may restrict or require actions.
4. **CR 728's subgame is a paused game inside a resolution.**
   `roadmap-v2.md`'s "excluded tail" lists Shahrazad among cards that break
   the process model. Under decision 2's B a subgame is a `Game` played on
   the resolving game's stack, its questions handed out by the same paused
   game, so the process model holds it. Not a reason to take the card in; a
   note for whoever reopens the tail.
