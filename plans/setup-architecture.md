# Game setup — CR 103's doors, the scenario loader, and the save

> **Status:** design, 2026-09-30, revised 2026-10-01 over three review rounds
> on PR #204, which carries the design alone. It covers `roadmap-v2.md` A6g's
> scenario work: the scenario loader as phase **SU-1**, setup actions as
> **SU-2**, and a board editor as **SU-3**, each its own PR. Item 194's PR,
> before SU-1, built CR 103.8's derivation (§1), SU-1 the loader, the writer
> and the dev GUI's start, and SU-2 setup actions (§8, both landed
> 2026-10-01). SU-3, the board editor, is designed in §7b and landed
> 2026-10-03. A6g's tools (undo, the save and
> savestates) are designed in §7.1–§7.3, decided over two review rounds on
> #212 (2026-10-03). Their design delta, after SU-3 (2026-10-03): §7.1's tree
> read again, §7.2's decision 6 (the records at scale, for v1), and §8's
> re-size, which builds them as two PRs (the owner, 2026-10-03): **SU-4** (the
> replay, the engine's), landed 2026-10-03, and **SU-5** (the tools, the
> window's), landed 2026-10-04.
> **The why panel**, A6g's next item, is designed in §7c and decided at #220
> (the owner, 2026-10-05, every recommendation taken): SU-6 to SU-8. **SU-6**
> (what the layers did, and the panel, #221) landed 2026-10-05.
> **Authority:** how a game is built before its first event, and what makes a
> built game reproducible: CR 103's dealt game (`Game::new`, `Game::setup`),
> the second door this adds (a described board), and the save. Where this
> contradicts `codebase-state.md`, that file wins on what exists; this file
> wins on what is being built. `CLAUDE.md` owns the ordering.
> **Companions:** `replacement-architecture.md` §2 (the chokepoint, which
> construction comes before); `triggers-architecture.md` §4.1 (why an
> announcement outside a batch dispatches); `codebase-state.md` items 41 and
> 140 (resuming at a priority prompt), 188 (two roads to one state), 193 (the
> replay hazard), 194 (CR 103.8a, closed by the first-draw PR) and 196 (CR
> 103.8b's teams).
> **Neighbors, owned elsewhere until they are designed here:** CR 103.5's
> mulligans (`backlog.md` §2.32), CR 103.6's opening-hand actions (item 119).

---

## 0. The decisions

Each is argued in the section named; each section gives the options with their
code shape, cost and upkeep.

1. **A scenario is a board described at rest** (§2). *At rest* means the state
   at the start of a step's priority round, with the stack empty and nothing
   waiting to trigger. **What the engine stores as plain state is written
   down:** cards in their zones, a permanent's status, counters and damage,
   life totals, and each player's history counts. **What a spell or an effect
   created, carrying a source or a duration, is played from the board** through
   the engine's own code: the stack, resolved effects, delayed triggers, extra
   turns. **A save is a start plus the decision log**, where the start is a
   dealt game (seed, pool, decks) or a scenario. So a save can be "scenario +
   log".
2. **It lives in the engine**, `mtgsim::scenario`, always compiled, as game
   construction beside `Game::new` (§3). The GUI, integration tests and
   `fuzz_games` call one `Scenario::build`. Its battlefield door is the entry
   performer's state half, split off the announcement, so the performer and the
   loader share one road (item 188). **No source tag:** construction comes
   before every event, and the loader holds no `DecisionProvider`, so it cannot
   propose, prompt or perform.
3. **A text file, parsed by hand in the engine**, with no new dependency, over a
   typed `Scenario` value (§4). **Cards are named, never numbered:** a tag in
   square brackets tells two cards with one name apart. **A writer** turns any
   game state back into a scenario file and lists what it could not write, so a
   board can be played to or loaded, tweaked and saved. Load refuses what the
   engine cannot represent consistently, naming the line and the rule. What the
   rules would correct (0 toughness, an Aura attached to nothing) is the game's:
   CR 117.5 performs it before the first priority, and the log shows it. **No
   SBA check at load.**
4. **v1's vocabulary** (§5): N players, the turn position, life, player
   counters and history counts, the five zones of cards, and permanents with
   controller, order, tapped, arrival, counters, damage, attachments and
   combat; commander designation and damage; a player who has left. **The
   writer reads every field of the state structs with no `..`**, so a field the
   engine adds does not compile until it is written, reported as unwritable, or
   named as the engine's own.
5. **Two streams from the scenario's seed** (§6): the game's and the agents'.
   One engine function derives them, with `fuzz_games`' salts moved in
   unchanged. A `shuffled` library exists for hand-written files; the writer
   lists every library in its order and never shuffles one.
6. **The GUI** (§7) takes `--scenario <file>`, has Reload and "Save board as
   scenario", and shows a load error in the window. The decision log embeds the
   scenario, so the log is a save. **Savestates**, positions the tester sets
   and moves between, go in A6g's tools PR (the one that builds undo and the
   save) as bookmarks in the log (the owner, 2026-10-01). The tools' decisions
   are §7.2's, settled 2026-10-03: a run stops by a typed unwind, an undo
   returns to the window's last prompt, the dev GUI seats no agent, the save is
   a journal beside the log, and the exported test is dropped. The records at
   scale are decision 6, from SU-4's design: the engine owns a record's text,
   each answer recorded by what was chosen so that a later build replays it
   until the game diverges, and each writer owns its files.
7. **Setup actions** (§5.3): any seat's actions, played in order from the board
   before the tester takes over, which is how a deep stack is built; they
   resolve nothing, so a resolved effect is their stack and the seats' passes
   (SU-2's build). They do not depend on what the tester does, so every reload
   reaches the same situation. One driver answers their prompts by matching
   options' ids. **SU-2, the next PR** (the owner, 2026-10-01). **No script
   for a seat during play** (the owner, the same day): it breaks as soon as
   play leaves the line it was written for. A fixed line is a regression test,
   which the log replays exactly; exploring is a person playing both seats.
8. **This file is where the design lives** (§9): CR 103 is a subsystem with
   three open items and no document, and the loader is its second door.
   `CLAUDE.md`'s architecture row gains one entry on its existing line.
9. **Building one from scratch** (§7a): a template and samples, refusals that
   say what to change, a list of cards in development that a scenario can name
   before they are registered (in SU-1, the owner, 2026-10-01), setup actions
   for what only an effect makes, the growth contract for a new mechanic's
   state, and a board editor in the dev GUI, **SU-3**, before the tools (the
   owner, 2026-10-03), designed in §7b.
10. **The why panel** (§7c, decided 2026-10-05). A right-click
    asks why of an object, and a panel beside the board answers: what the
    layers did to it, and why it is or is not among the open question's
    options. From the trace, it also says what an event did and which
    triggered abilities were asked about it. A question about now is the
    engine's, asked at the seat. A question about then comes from a replay
    to the open question with the sink on. Three PRs, SU-6 to SU-8.

**Size** (§8): SU-1's code ~1,010–1,360 lines and tests ~710–980; SU-2
~400–650 in all, built at 640 and 314. **A/B:** `IDENTICAL` predicted for each,
since no path a fuzz game runs changes behavior.

---

## 1. Where it sits

On the event path, nowhere, and that is the point. A scenario is built before
the first event, as `Game::new` fills the libraries before the first shuffle.
Play then enters at a step's priority round, through item 41's entry point,
`Game::resume_turn_at_priority`, which skips the step's turn-based actions
because the board already shows them done. Its first act is CR 117.5's
state-based action check, then triggers, then the active player's priority (CR
117.3a). From there it is an ordinary game. The writer runs the other way, from
a game at any prompt back to a file.

```
Game::new + Game::setup   CR 103: libraries, shuffle, hands, first turn ──┐
Scenario::build           a board at the start of a priority round, ──────┤
                          then SU-2's setup actions                        ▼
            Game ── run, or resume_turn_at_priority ── CR 117.5's check ── priority …
              └── Scenario::write, at any prompt ──▶ a scenario file, and what it could not write
```

**CR 103.8's first draw is derived where a game is built** (item 194, the
first-draw PR, 2026-10-01). `Game::new` sets `skip_first_draw` from
`state::game::starting_player_skips_first_draw`, which answers CR 103.8a and
103.8c from the seat count the game begins with, as CR 800.1 reads it
(`GameState::is_multiplayer`). `begin_step` reads the flag and refuses that
draw step at its proposal, since a skipped step is passed "as though it didn't
exist" (CR 500.11): it begins nothing and grants no priority. The answer keys
on the players, never on a format: a two-player Commander game skips the draw,
and four players with 60-card decks do not. So no config states it.
`GameConfig::first_player_draws` is an override, `None` in every constructor
but `test()`, whose `Some(true)` keeps a test's hand sizes independent of the
seat count; the clients build their games from `GameConfig::unrestricted()`. A
scenario that starts in turn 1 before the draw step calls the same function
(§5.1). CR 103.8b's Two-Headed Giant team plugs in at the function's input once
Phase 9 models teams (`codebase-state.md` item 196); neither caller changes.

---

## 2. What a scenario is

**The problem.** Testing an interaction needs a chosen board. The dev GUI
starts only from a seed (`devgui/src/bridge.rs` deals two random 60-card decks
from a pool), so a board is hunted across seeds.

| | **R. A recipe**: seed, decks, a scripted opening | **D. A described board** |
|---|---|---|
| Code | none new: `Game::new`, `setup`, a scripted provider | a loader, ~700 lines (§8) |
| Authoring | find a seed that deals the cards, then script every answer of every seat until the board appears; library order and a turn-7 life total come only from play | write the board down, save it from a game (§4.3), or build it in the editor (§7a) |
| Upkeep | answers are indices into option lists, so an engine change that adds or reorders an option anywhere upstream re-scripts every recipe past it | a vocabulary that grows with the engine's stored state (§5.2) |
| Risk | none new | building a state no play reaches (item 188); answered in §3 by building through the engine's own doors, and checked in §4.3 against played boards |

**Recommendation: D.** One rule draws the line between what a file writes and
what is played.

- **Plain state is written.** That is anything the engine stores as a value
  with no source and no duration: a field (tapped, counters, damage, a life
  total, a card's zone, the turn), or a count it keeps. A player's history is
  such a count: spells cast this turn, life gained, attackers declared.
  `PlayerHistory` is materialized, and "nothing derives these from the event
  stream" (`state/history.rs`), so writing a count is like writing a life
  total.
- **What an effect created is played.** That means a spell or ability on the
  stack, a registry row from a resolution (Giant Growth's +3/+3, Act of
  Treason's control, a regeneration shield, a "can't"), a delayed trigger, or
  an extra turn. Writing those down would rebuild what `cast_spell` or a
  resolution computes (601.2's targets per instance, the rows a resolution
  lowers to), which is item 188's second road. A stack entry written by hand
  is the fixture that hid Skullcrack's target for four phases. Played through
  the engine, they come out exactly as play makes them.

**A scenario never needs what happened before it.** It needs the state at its
start, and every piece of plain state, history included, is written. What it
cannot write is only what writing would duplicate engine code for, and those
are played: by the window or an agent, after §5.3's setup actions have put
them on the stack before the tester takes over.

**The save.** A save is a *start* and the decision log. The start is a dealt
game (seed, pool, and the decks the log already records) or a scenario (its
text, embedded so the save outlives edits to the file). Replaying builds the
start and answers from the log; undo (the tools PR) is the replay without the last
answer. So a save is "scenario + log", and a dealt game is the other kind of
start. An exact replay needs the start, every answer in order and the same
engine; since item 193 no seat's mode changes what it is offered.

**Two snapshots of a regular game follow.** The save is exact at any prompt,
because it replays. The board written as a scenario (§4.3) is editable, exact
for everything it writes, and lists what it could not write.

---

## 3. Where it lives

**The problem** is item 188: code that builds one state in two places drifts.
Today's board builders are `test_support`'s, behind the `test-support` feature,
and they are two builders that disagree by design (the module doc says so).
Measured with a throwaway probe on 2026-09-30, deleted after the reading:

- `put_on_battlefield` announces each arrival outside any batch, so the trigger
  dispatcher runs. Soul Warden then two Grizzly Bears: **2 triggers waiting and
  5 events recorded** before anyone has priority.
- `place_bare` registers nothing. Glorious Anthem and Grizzly Bears: **0
  registry rows**, and the Bears are 2/2.

Neither builds a board at rest.

| | **A. An engine module, `mtgsim::scenario`** | **B. Promote `test_support`** | **C. devgui code** |
|---|---|---|---|
| Code shape | `Scenario::parse(&text)?.build(&registry)?` returns a `Game` | drop the feature gate | the GUI writes `GameState` through its `pub` surface |
| Cost | ~700 lines that ship with the engine, none on a game's hot path | ~930 lines of fixtures ship: inline cards with test ids, `test_ctx` leaking a provider per call, the recording providers | no `pub(crate)` door is reachable (`create_in_zone`, the registration), and tests and `fuzz_games` cannot call it |
| Upkeep | one loader for three callers, inside the crate's invariants | the GUI picks one of two idioms and neither is at rest; fixing either changes what ~1,600 `put_on_battlefield` and 82 `place_bare` calls do under the tests that rely on it | the second road item 188 names, in a crate reviewed by running it |

**Recommendation: A.**

```rust
let scenario = Scenario::parse(&text)?;              // or a `Scenario` value built in Rust
let mut game = scenario.build(&CardRegistry::default_registry())?;
game.resume(&dp)?;                                   // resume_turn_at_priority, then run
```

### 3.1 What a scenario's permanent gets

| | `put_on_battlefield` | `place_bare` | a scenario's permanent |
|---|---|---|---|
| in the store, with CR 613.7d's timestamp | yes | yes | yes, in the file's order |
| entity from `PermanentState::entering` | yes | no: `PermanentState::new` | yes |
| static abilities registered, every gate filed (CR 113.6, 604, 613) | yes | **no** | yes |
| CR 306.5b's intrinsic entry counters | yes | no | yes, unless the file states that kind |
| arrival announced, triggers dispatched | **yes** | no | **no**: it is at rest |
| summoning sickness | not sick | not sick | as stated: arrived before this turn (the default) or this turn |

**One road.** `place_on_battlefield`, the `EnterBattlefield` performer, splits
at the seam between state and event. Its state half (the entity, the counters'
CR 613.7c timestamps, the memo bump, `register_static_effects`) becomes the
door both use; the performer is that half plus the announcement. The loader's
door is the construction counterpart of `create_in_zone`, which refuses the
battlefield today because during play an arrival is an entry. At construction
it is not, and the door refuses once a turn has begun: the loader builds at
turn 0, as `Game::setup` deals the opening hands at turn 0, and begins the
stated turn last.

**The arrival turn is a parameter of the state half.** CR 302.6's clock is the
entity's `controller_since_turn`, and a static Layer 2 effect's clock is its
row's `created_on_turn` (the controller arm in `engine/layers/compute.rs`). So
a Control Magic–shaped Aura that arrived this turn leaves the creature it
steals sick, and one that arrived earlier does not. `PermanentState::entering`
and `register_static_effects` take the turn, and the performer passes the
current one, so play is unchanged: ~15 lines over five call sites.

**Every other write goes through the door play uses.** Cards off the
battlefield go through `create_in_zone`, `Game::new`'s door, which registers
what functions there (CR 113.6). Counters go through `GameState::add_counters`,
in the file's order. An attachment goes through `GameState::attach` at its own
line, so CR 613.7e's new timestamp sits in the file's order; its host must be
listed first. The turn goes through `set_turn_position`, `begin_turn` and
`begin_turn_history`, the one writers. The plain fields with no door (tapped,
damage, combat, life, lands played, the history counts) are written directly,
as `engine/combat/steps.rs` writes combat; none is a layer-walk input, so none
needs a bump.

### 3.2 The exemption

The chokepoint invariant forbids mutating *observable* state outside
`perform_action`'s arms, because CR 614 reads proposals. Nothing observes a
scenario's writes. The loader holds no `DecisionProvider`, so it builds no
`ActionContext` and can neither propose, prompt nor perform, and the state it
returns has never emitted an event (asserted: the event sequence is still at
its start). That is construction, which the tree already does without a tag:
`Game::new`'s `create_in_zone`, and `Game::setup`'s opening draws, "safe by
construction rather than by exemption". **So no source tag.** One sentence goes
into `replacement-architecture.md` §2 beside `// CAST-ROLLBACK:`, and
`CLAUDE.md` is unchanged. SU-2's setup actions are not construction: they are
played, through the chokepoint like any other action.

The layer invariant holds without an exemption too: the loader reads no printed
characteristic of a permanent, and a check that needs one (who controls an
attacker) runs after the board is built, through `oracle/characteristics.rs`.
The one printed read is CR 304.4 and 307.4, "instants can't enter the
battlefield", asked of a card before it is a permanent, so it carries
`// PRE-LAYER ZONE:`.

### 3.3 Tests after this

Existing tests keep their builders. The ~1,600 `put_on_battlefield` and 82
`place_bare` calls build exactly what each test was written against, and
moving them is a refactor that changes what some of them test (one that relies
on an announced arrival, say). New tests choose. A scenario reads better when
the test is about a board (several permanents, a turn position, combat), and a
test that fails on one can be opened in the dev GUI as it is, since
`--scenario` takes the same text. `test_support`'s builders stay for what a
scenario does not do: a fixture card built inline (a scenario names one only if
the test registers it in the registry it passes), and `place_bare`'s deliberate
"nothing registered". Whether `put_on_battlefield` should stop announcing is
item 188's survey (A6h).

---

## 4. The format, names, and the writer

| | **T. Text, parsed by hand in the engine** | **S. TOML or RON through serde** | **B. Rust builders only** |
|---|---|---|---|
| Code | ~250 lines of parser | ~60 lines of derives | none |
| Dependencies | none | serde and a format crate in the engine: the owner's call, and it pre-empts `backlog.md` §2.38's wire format (Phase 10) | none |
| Errors | ours: the line, and the rule that refuses | syntax from the crate; the semantic ones (an unknown card, an ambiguous name) still ours | the compiler's, for shape; ours at build |
| Editing a scenario | edit, click Reload | edit, click Reload | edit, rebuild the engine and devgui |
| Upkeep | a new word is a row in the table the parser and the writer walk (§5.2) | a new word is a field | a new word is a method |
| Regression export | `include_str!` the file | the same | generate Rust |

**Recommendation: T**, over a typed `Scenario` that Rust may build directly
when a test wants to. No builder methods in v1: a test writes its scenario
inline as text.

The shape (the grammar is settled in the build):

```
# Holy Strength on an attacker, Wall of Stone blocking, Humility arrived this turn
players 2
seed 7
turn 3
active 0
step declare blockers

player 0: life 18
player 0 this turn: attackers declared 1
player 1: life 20, poison 2
hand 0: Lightning Bolt
library 0: Mountain                         # top first
library 0: Forest | x10
library 1 shuffled: Forest | x20
graveyard 0: Savannah Lions                 # each card on top of the last

battlefield: Glorious Anthem | controller 0
battlefield: Grizzly Bears [a] | controller 0, tapped, attacking player 1
battlefield: Grizzly Bears [b] | controller 0
battlefield: Holy Strength | controller 0, attached to Grizzly Bears [a]
battlefield: Wall of Stone | controller 1, blocking Grizzly Bears [a]
battlefield: Loyalty Probe | controller 1, loyalty 1
battlefield: Humility | controller 1, arrived this turn
exile: Lightning Bolt | owner 1
command: Isamaru, Hound of Konda | owner 0, commander
```

Most lines state one card and what differs from the default (§5.1), so a file
is about as long as the board it describes.

**Why these characters.** Scryfall, `name:/x/ game:paper`, unique cards,
2026-09-30 and 10-01: a comma is in 3,798 card names, so a line holds one card
and a comma never separates names; `|`, `#` and `[` are in none, so they open
the attributes, a comment and a tag; a semicolon is in one (TL;DR); `(` is in
45; 11 names begin with a digit, so a count is `x10` after the bar and never
`10 Forest`. A name runs from the head's colon to the tag or the bar, so
"Circle of Protection: Red" parses (only the first colon is the head's). Words
reuse the spellings already shown to a person: a step as `format_phase` prints
it, and a counter kind as devgui's `counter_label` prints it, a table the build
moves into the engine so the parser, the writer and the window read one copy.

### 4.1 How a bad board fails

| Class | Example | At load |
|---|---|---|
| Not a card | `Grizly Bears` | refused: the line and the name, "Grizly Bears is not registered" |
| A reference that does not resolve | `attached to Grizzly Bears` with no Bears on the battlefield, or with two and no tag; a host listed after its attachment | refused, naming the lines |
| A state no sequence of events reaches | an attacker in a main phase (CR 506.4, 511.3); a blocker of a creature that is not attacking; an attacker attacking its own controller (CR 508.1b; a control change removes it from combat, 506.4); declare blockers with no attacker (CR 508.8); an instant on the battlefield (CR 304.4); a card owned by a player who has left (CR 800.4a); the untap or cleanup step, where no player receives priority (CR 502.4, 514.3); turn 1's draw step in a two-player game, which CR 103.8a skips as though it didn't exist (500.11) | refused, naming the rule |
| A state the rules correct | a 0-toughness creature (CR 704.5f), an Aura attached to nothing (704.5m), ten poison counters (704.5c), two legends with one name (704.5j) | built; CR 117.5 performs it before the first priority, and the log shows it |

**SBAs are not checked at load.** A check is an event, the resumed game's first
record. CR 704.5j asks a player which legend to keep, and the loader holds no
provider. And a board whose state-based actions act is how one is tested.

### 4.2 Names, not ids

An `ObjectId` does not exist until the loader creates the object, and the
loader mints ids in file order, so `#12` would mean "the twelfth card created".
An author would count every card above it, the twenty Forests included, and
adding a line would renumber every later one. A name is what the author already
has.

So a reference is a card's name, which must be unique among the objects that
reference can mean: an attachment's host among the battlefield's permanents, a
commander among the commanders. When two share it, each takes a tag in square
brackets, on its own line and in every reference to it, and the loader refuses
an ambiguous reference by naming both lines. The tag is the author's choice of
text; the writer uses `[a]`, `[b]`, … and only for a name that occurs twice.

Forge's puzzle files, 374 of them in its repository, are the precedent. Cards
are named, libraries are listed top first, attributes follow a `|`, and only
cards in a relationship carry an author-chosen id: `Isochron Scepter|Id:9|Imprinting:8`
beside `Accumulated Knowledge|Id:8|ExiledWith:9`.

### 4.3 The writer

**The problem.** Writing a whole board by hand is tedious. A tester who has
reached an interesting board in a game, or has loaded a scenario and played
into one, wants to keep it, edit it and load it again.

**The shape.** `Scenario::write(&GameState)` returns a scenario's text and the
list of what it could not write. The dev GUI calls it from a "Save board as
scenario" button at any prompt. The writer:

- reads every field of `GameState`, `PlayerState`, `PermanentState` and
  `GameObject` through a destructure with no `..` (§5.2's growth contract), so
  each field is written, reported, or named as the engine's own (the memo, the
  gates, the event window);
- writes each library in its order, top first, and never as `shuffled`;
- writes its lines in CR 613.7 timestamp order across the zones, since the
  loader stamps in line order; a library's cards are stamped in its own order
  instead. A counter kind or an attachment stamped later than its permanent's
  line gets a line of its own at its place. So a timestamp comparison a rule
  makes, such as a flying counter put on after Humility arrived, reads the same
  after loading;
- tags a name only where two objects share it;
- reports what §2 says is played, rather than dropping it silently: the stack,
  a resolution's rows, a delayed trigger, an extra turn. It also reports the
  fields §5.2 lists as waiting (a copy, a token, a face-down permanent). The
  report goes at the top of the file as comments, and into the window.

A written board resumes at the start of the step's priority round where it was
written, with the active player to act (§5.3). **Its randomness is fresh.**
`StdRng` keeps its state private, so the stream's position cannot be written.
The written seed gives a deterministic game, not the original's continuation;
that is the save's job (§2).

**The writer is also the loader's test.** Played games reach boards no
hand-written test thinks of. A test plays a few dozen seeded games with the
random agent. At every round start whose board the writer reports nothing for,
the test writes the board, loads it, and compares it with the original over
every written field and every object's computed characteristics. That checks
the loader against play rather than against itself, at the scale item 188's
risk calls for. The boards skipped are counted by reason, and that count orders
the vocabulary's next words. Tokens and copies are the likely first.

---

## 5. The vocabulary, sized

A *word* is anything a scenario file can say: a line's head (`hand 0:`) or an
attribute after the bar (`tapped`).

### 5.1 v1, as built

**The one table of the grammar**: the parser (`scenario/text.rs`) reads it,
`Display` writes it, and `mtgsim/scenarios/template.scenario` shows every
word in use. One line states one thing, and `#` starts a comment. A line that
names a card puts the name after the head's colon (only the first colon is the
head's, so "Circle of Protection: Red" parses), then an optional tag in square
brackets, then `|` and its words, separated by commas. A word that names
another card takes the rest of the line, so it comes last
(`CardWord::names_a_card`, which `Display` writes last). A count is `xN` after
the bar, never `N Forest`, since a name may begin with a digit. A setup action
(`then:`, §5.3) gives each answer a segment of its own after a bar instead: a
bar is in no card's name, and a target's name may hold a comma.

Defaults in the last column apply when the file says nothing. A reference
(`<card>`) is a name, with its tag where it has one (§4.2).

| Word | CR | Writes | Default |
|---|---|---|---|
| `players N` | 102.1 | `GameState::new(N)` | 2 |
| `starting life L` | 103.4 | `starting_life` and each life total | 20 |
| `seed S` | — | §6's two streams | 0 |
| `turn T`, `active A` | 500.1, 102.1 | `begin_turn` and `begin_turn_history` for the turns the natural rotation over the players in the game gives, ending with A's turn T; `turn_rotation`; `priority_player` | 1, player 0 |
| `step …` | 500.1, 117.3a | `set_turn_position`; `attacks_declared` from the combatants | precombat main |
| *(who has priority)* | 117.3a | the active player, at the round's start | not a word: §5.3 |
| *(the first draw)* | 103.8 | `skip_first_draw`, from `starting_player_skips_first_draw` (§1) | derived |
| `player p: life N` | 119 | `life_total` | starting life |
| `player p: poison N`, `energy N`, … | 122.1 | `PlayerState::add_counters` | none |
| `player p: lands played N` | 305.2 | `lands_played_this_turn` | 0 |
| `player p: left the game` | 104.5, 800.4a | `player_lost`; refused if p owns or controls anything, is active, or leaves fewer than two in the game | in the game |
| `player p: commander damage N from <card>` | 903.10a | `commander_damage_taken` | none |
| `player p this turn:`, `last turn:`, `this game:` (and `since your last turn:`, which reads and is refused: the rows derive it), each with `spells cast N`, `<type> spells cast N`, `cards drawn N`, `life gained N`, `life gain events N`, `life lost N`, `life loss events N`, `damage taken N`, `creatures died N`, `attackers declared N` | — | `PlayerHistory`'s rows, one word per `TurnFact`; "this game" defaults to the other two rows' sum, and "since your last turn" is derived from the rows | zero |
| `this turn: <card> \| triggered`, `resolved N`, `took its once-each-turn action`, each after an optional `ability N` (its printed place, needed when the card has two that could be meant) | 603.2h, 603.7h | `triggered_this_turn`, `resolutions_this_turn`, `action_taken_this_turn` | none |
| `hand p:`, `library p:`, `graveyard p:` | 402, 401, 404 | `create_in_zone`: a library top first; a graveyard bottom first, each card on top of the last, since lines are stamped in file order (§4.3) | empty |
| `library p shuffled:` | 401, 701.24 | then `shuffle_library` from the game's stream | — |
| `exile:`, `command:` | 406, 408 | `create_in_zone`, with `owner` | empty |
| `commander` | 903.3 | `GameObject::is_commander` | no |
| `xN` | — | N copies of the line's card, each its own object; refused with a tag | 1 |
| `battlefield:` | 613.7d | the door (§3.1), in the file's order | — |
| `controller p`, `owner p` | 110.2b, 108.3 | the entity's default controller; the object's owner | each the other |
| `tapped` | 110.5 | `tapped` | untapped |
| `arrived this turn`, `arrived turn N` | 302.6 | the door's arrival turn. CR 302.6 measures from the controller's own most recent turn, so a non-active player's creature that arrived on their last turn is still sick, which "this turn" alone cannot say (the build, 2026-10-01) | arrived before the first turn |
| `<kind> N`, a counter | 122.1, 613.7c, 306.5b | `add_counters`, in order; a stated kind replaces its intrinsic count | the intrinsic entry counters |
| `counters: <card> \| <kind> N` | 613.7c | a counter kind stamped at its own line (§4.3) | — |
| `damage N` | 120.6 | `damage_marked` | 0 |
| `attached to <card>` | 301.5, 303.4, 613.7e | `attach`, at its line | — |
| `attacking player p`, `attacking <card>`, `blocked` | 506, 508.1, 509.1h | `attacking` | — |
| `blocking <card>` | 509.1a | `blocking`, and each attacker's `blocked_by` in the file's order. One attacker per blocker: a word that names a card takes the rest of its line, since a name may hold a comma, and no registered card blocks two | — |
| `dealt first-strike damage` | 510.4 | `dealt_first_strike_damage`; refused before the first-strike damage step, and kept to the end of combat as the engine keeps it | no |
| `then: player p casts <card>`, `then: player p activates <card>` | 117.1, 601.2, 602.2 | a setup action (§5.3), played from the board in file order: a cast from p's hand, or an activation of an ability of a permanent p controls | none |
| `\| targeting <card>`, `\| targeting player p` | 115.1, 601.2c | the line's answers to CR 601.2c's choices, a target or a "choose": each choice the spell or ability asks takes the next segments it offers, so they read in the card's own order. A card here is a permanent, a card in a graveyard or exile, or a spell an earlier line casts | — |
| *(an X)* | 107.3a, 601.2b | not a word: no X spell is offered at priority, since `ManaPool::can_pay` and `find_mana_sources` read no X (`codebase-state.md`'s CR 107 row), and none is registered. The driver names `ChooseXValue` among the questions no line answers, so the PR that offers one adds `x N` | — |
| `\| ability N` | 602.1 | the ability's place among the permanent's abilities as the layers give them, 1 for the first; never a mana ability, whose mana would wait in a pool no word writes (§5.2) | the permanent's one activated ability |
| *(a mode)* | 700.2 | not a word: nothing asks for a mode, since `Effect::Modal` cannot resolve (`backlog.md` §2.7). The setup driver matches every `ChoiceKind` with no wildcard, so the PR that adds the prompt adds `mode N` | — |

### 5.2 Later, and the growth contract

**Played, never written:** the stack (CR 405, and 601.2's choices), effects
with a source and a duration (Giant Growth's row, Act of Treason's control,
prevention and regeneration shields, a "can't"), delayed triggers (CR 603.7),
and extra turns and phases (500.7, 500.8). Triggers waiting to be put on the
stack have no word either, since none is waiting at a round's start (CR 117.5).
**Road in v1:** play them from the board, in the window's seat. **Road from
SU-2:** setup actions (§5.3) put them on the stack, and the seats' passes
resolve them.

**Fields whose word waits for something:**

| Field | CR | Waits for | Road meanwhile |
|---|---|---|---|
| entered as a copy (`entered_as`) | 707.5, 614.1c | ~60–100 lines to capture a named card's or another object's copiable values through the copy code; first in line if §4.3's count says so | cast the Clone |
| tokens | 111.1 | a name for a token: today each card file defines its own `TokenDef`; first in line if §4.3's count says so | play the card that makes it |
| X, how it was cast, cost choices | 107.3f, 400.7d, 707.10 | the first static ability that reads one at rest | cast it |
| the mana pool | 106.4 | `codebase-state.md` main item 33, whose provenance decides what a unit of mana is | tap the lands |
| face down, phased out, flipped | 708, 702.26, 710 | the PR that builds each system | — |
| counters on a card off the battlefield | 122.1, 702.62 | `GameObject` carrying counters (suspend's time counters) | — |
| how many times a commander was cast from the command zone | 903.8 | the PR that counts it for commander tax | — |
| exiled face down | 406.3 | CR 708's PR | — |
| "since your last turn" past two seats | — | a word for the counts a player's last turn ended at; the loader derives them from two rows, exact at two seats and the round trip's commonest skip at four (1,312 of 1,901 boards) | the save |
| turns off the natural rotation: a player who left after taking turns, an extra turn | 302.6, 500.7 | a word for each player's most recent turn; the loader rotates over the players in the game | the save |

**The growth contract.** The format grows with the engine's stored state, never
with its cards or its effects. A new card, trigger or effect needs no word,
because effects are played. That is what §2's rule buys: writing effects down
would make this surface grow with every effect the engine learns. A new field
does need a word, and the writer's destructure makes that a compile error until
it is answered in one of three ways:
- **written:** a row in the table the parser and the writer both walk, holding
  the word's spelling, how the writer reads it and how the loader writes it,
  plus a round-trip test, ~30–50 lines;
- **reported** as unwritable;
- **named** as the engine's own.

The four structs hold about 100 fields today.

### 5.3 Priority, the stack, and setup actions

v1 resumes every scenario at a round's start, where CR 117.3a gives the active
player priority. A stack, and a non-active player about to act, come from play.
**SU-2 adds setup actions**: a list of actions, each naming its seat, played
from the board in order before anyone else is asked:

```
then: player 0 casts Lightning Bolt | targeting Grizzly Bears [b]
then: player 1 casts Giant Growth | targeting Grizzly Bears [b]
then: player 1 activates Merfolk Thaumaturgist | targeting Grizzly Bears [b]
```

- The seat holding priority passes until the next line's seat holds it. So a
  stack of ten is ten lines and nothing else: the opponent's responses are
  lines like any other, and nobody scripts a pass. While a line is left, the
  driver stops every seat at every priority point, so none is passed over
  unseen.
- Each line's own prompts are answered from the line: its targets by name or
  player, its ability by place; nothing asks for a mode or an X yet (§5.1).
  Costs are paid from the board's untapped lands, as a seat's `AutoPayer` over
  `ManaWindowStop` pays them: the window taps a source that makes a pip still
  owed, by the random agent's preference taking its first source rather than
  a random one and the line's own permanent last, closes once paid, and the
  generic split is the first the caps allow. (`AutoPayer` itself only orders
  cost reductions; the tap is the driver's, until `backlog.md` §2.18's
  solver.) A deep stack needs as many lands as its spells cost; floating mana
  waits for item 33's provenance.
- What can be checked before play fails the load, naming the line: a name
  that means nothing or two things, a card not in the seat's hand, a
  permanent another player controls, an ability that is not an activated one.
  What only play shows is refused in play, naming the line: an action the
  engine does not offer when its seat holds priority, which is how a line no
  seat reaches before the stack would resolve is refused rather than played a
  turn later; an action it rewinds once picked (CR 732.1); a target the choice
  does not offer, one too few or one too many; and a question no line answers
  (a trigger's, a replacement's, a "may", a cost choice). A `DecisionProvider`
  cannot stop a game, so that refusal is a panic, shown in the window as the
  engine's; the tools PR's "stop" answer replaces it.
- When the list is spent, the next prompt goes to whoever plays that seat: the
  window or an agent.

**Setup actions are part of the situation, not of the play.** They run before
the tester is asked anything and read nothing the tester does, so every reload
replays them identically and the tester tries a different line from the same
board each time. The driver that plays them, `SetupDriver`, is a
`DecisionProvider` over the seats' own: the loader resolves each named card to
its id through the table it built the board with (`Scenario::build` returns
both), and the driver picks the option carrying that id:
`ChoiceOption::Action(CastSpell(id))` or `ActivateAbility(id, ability)`,
`Object(id)`, `Player(p)`. It matches by id rather than by text, so a change in
how options are worded does not break a file. Every action still goes through
`cast_spell` and `activate_ability`, so the stack is exactly what play builds:
targets per instance, costs, cast triggers.

**Setup actions resolve nothing** (the build, 2026-10-01). Every line is
played before the stack it builds could resolve, since resolving needs every
seat to pass and the line's seat never does, so the seats' passes resolve it
once the tester takes over. A resolved effect (a creature under Act of
Treason, a regeneration shield) is its line and those passes. A line that
resolves the top of the stack before the tester is asked would build one
outright: ~40 lines with its test, and it weakens the load's checks for every
line after it, since a resolution can move a card or a permanent's control.
Open for the owner at SU-2's review.

**Why not write the stack down instead.** It would need the same lines (who
cast what, targeting what, in which order), plus everything casting decides
that the lines leave to the engine. And a stack written by hand is the second
road §2 rules out.

**No script for a seat during play** (the owner, 2026-10-01, at the second
review round). One was proposed: a seat taking listed actions against a live
opponent, each at the first prompt where it could. It fails at the scenario's
main use. A scenario is replayed to try different options, and a scripted
response written for one line answers the wrong spell, or nothing, once play
leaves that line. What it would have served is covered elsewhere: a fixed line
is a regression test, which the decision log replays exactly (§7); exploring
the opponent's side is a person playing both seats (§7); and building the
situation is setup actions.

**Why SU-2 and not SU-1.** The driver, the action words and their tests come to
~400–650 lines, and SU-1 with the writer comes to ~1,720–2,340, so together
they cross the band's 2,500. SU-2 can follow SU-1 directly, ahead of playable,
and needs nothing from item 193: a setup action the engine refuses is refused,
naming its line.

### 5.4 N players

The engine side is N-seat from the start: each seat's zones, one battlefield
with controllers, attack targets naming a player, a planeswalker or a battle
(CR 508.1b), blockers controlled by the attacked player (509.1a), players who
have left (800.4a), commander designation and damage. The GUI loads two-seat
scenarios, since the bridge builds two providers, until the seats PR, which
lifts that with four seats in it (§7b's decision 4, the owner, 2026-10-03).
Tests and `fuzz_games` load any count.

---

## 6. Determinism

**The question:** the same scenario and the same answers must play the same
game.

**The loader is deterministic by order.** Ids, CR 613.7 timestamps, counter
timestamps and each zone's order come from the file's order through the
state's own counters, and the loader iterates no map. A registered card's
ability ids derive from its name (`AbilityId::printed`), so nothing depends on
the process.

**Two streams.** A dealt game has three: the decks, the game's and the agents'.
A scenario's decks are described, so two remain:

1. **The game's**, `GameState.rng`, which draws every shuffle (CR 701.24),
   random discard (701.9b) and coin flip (705), and at load the order of any
   library marked `shuffled`, in seat order. **`shuffled` is for a
   hand-written file** whose author does not care about a library's order
   (twenty Forests, or a pool to draw from at random), and for `fuzz_games
   --scenario`, where each game should draw differently. Its order comes from
   the file's seed, so the same file and seed give the same order in every
   process. **The writer never writes `shuffled`:** a saved board lists every
   library in its order.
2. **The agents'**, `RandomDecisionProvider::seeded`: one per agent seat in the
   GUI, one for every seat in `fuzz_games`, as today.

**One function derives both from the seed**, `RandomStreams::from_seed`, with
`fuzz_games`' two XOR salts moved into the engine at their values, so no
fixture row moved. The dev GUI's dealt start used the seed, +1 and +2 until A6g's
review practices PR converged it (#208, finding 23), ahead of the tools PR this
section had planned it for, so a fuzz game's printed seed deals that game's
decks and shuffle in the window.

**What a replay needs.** The log records every seat's answers since A6g's
playable PR, written by the engine with the passes it makes itself
(`state::decision_log`), so a replay that has every seat stop at every priority
point needs no agent stream (`tests/phase_a6g_integration_test.rs` replays a
game from its log). The streams matter only for playing on past the log's end,
and a replay does not depend on the agent's policy. **What can still break it:**
a sweep that leaks map order, which `CLAUDE.md`'s three runs under three
`MTGSIM_HASH_SEED`s catch; item 193's blacklist, the other, left the engine
with playable. `fuzz_games
--scenario` (§8) puts a scenario-started game under that check.

---

## 7. The GUI, and the regression test

| Way in, or out | Size | What it gives |
|---|---|---|
| **An argument**: `cargo run -- --scenario ../mtgsim/scenarios/x.scenario [--seed N]`, the flag overriding the file's seed | ~30 lines | the board, and the same board with a different agent |
| **Reload**: build the game again from the same file | ~25 lines | edit, reload, no relaunch; the old engine thread unwinds when its channel closes, as a closed window ends it today |
| **Save board as scenario**: the writer, at the current prompt | ~30 lines | keep a board reached in play; edit it; load it |
| **A list** of `mtgsim/scenarios/` in the header | ~40 lines | picking without the command line |
| **A file picker**: a dialog crate (`rfd`) in devgui | ~20 lines and a dependency with code per platform | picking anywhere on disk |

**Recommendation: the argument, Reload and Save now; the list once there are
more than a handful of files; no picker.** A load error shows where the panic
panel is, with the file's line, and Reload tries again, so a typo does not need
a relaunch. The decision log of a scenario game starts with `scenario <path>`,
`seed N` and the scenario's text verbatim, so the log is a save (§2) even after
the file changes.

**Seats.** The window played seat 0 and the random agent the others until the
seats PR (#214, 2026-10-03); playable's PR had built the seat's controls and
kept the window at seat 0 (2026-10-02). **The window plays every seat, and the
agent has left the dev GUI** (the owner, 2026-10-03, over the tools' two
review rounds): reaching a
situation on a big Commander board takes specific choices from each seat, an
agent's random ones get in the way, and the dev GUI is for building cards and
debugging, not for play against an opponent. `fuzz_games` keeps the agent.
With no script for a seat during play (§5.3), playing every seat is also how a
tester explores both sides of an interaction from one board. What it takes:
- every seat gets the window's stack of decorators and its own yield; full
  control stays one switch;
- a prompt names the seat it asks, which `Prompt` does not carry today, and
  `WINDOW_SEAT`'s seven uses (the bridge and the view model) go;
- the tests that play the window's part, the random clicks and the rule the
  review pictures are drawn by, answer for every seat, so the pictures redraw;
- **past two seats**, the owner's call, taken at SU-3's design: `--players N`
  for a dealt game and a scenario's own count, where today the bridge deals
  two decks and refuses a scenario that is not two-seat
  (`build_scenario_game`), though
  `mtgsim/scenarios/four-seats-commander.scenario` already loads in the
  engine.

Sized ~200–300 lines with tests at two seats, and ~+100–150 past two. It
came before the tools, as a PR of its own ahead of SU-3 with four seats in it
(§7b's decision 4, the owner, 2026-10-03), re-sized in §8: **built by the
seats PR** (#214), whose record is in `plans/archive/roadmap-v2-landed.md`,
A6g.

**Savestates** (the owner's suggestion, placed in the tools PR by the owner,
2026-10-01) are positions the tester sets during play and moves between, like a
video game's save slots. They belong there because they are its replay with
more than one stop. A savestate is a bookmark at an answer in the decision log,
and moving to one replays the start and the log up to it, exactly, at any
prompt. A cloned `GameState` would resume only at a round's start (item 140).
Playing on from an earlier bookmark starts a branch, and the save keeps every
branch. A savestate can also be written out as a scenario, with the writer's
report. §7.1–§7.3 design them with undo and the save, and §8 sizes the
build.

### 7.1 The tools: where they sit, and what the tree has

> **Status:** design, 2026-10-02, decided over two review rounds on #212
> (2026-10-03). Its tree read again at SU-4's design (2026-10-03), against
> `fb1767a`, #215's merge. The build, SU-4 and SU-5, comes after SU-3 (§8).

**Where it sits.** On the decision boundary's far side, beside the decision
log. Since playable (#211) the engine writes every answer to the log, whoever
gave it, and marks the passes it makes itself `forced` (`state::decision_log`).
Every tool here reads that record. A replay is a `DecisionProvider` that
answers from it, so the engine cannot tell a replayed answer from a live one,
and no proposal, event or trigger changes. The one change to the boundary
itself is the stop (decision 1), which lets a run return to its caller from
inside a prompt.

```
a start ── build ──▶ Game ── run ── each prompt ──▶ the replay: the log's next line
(dealt, or                                           └─ the log spent ─▶ the seats (the window), or a stop (a test)
 a scenario)        every answer ──▶ the engine's log ──▶ the session's save
```

**What the tree has** (read 2026-10-03 at `fb1767a`; first read 2026-10-02):
- **The log's text is the dev GUI's.** `DecisionLog` (`devgui/src/bridge.rs:336`)
  writes it from the engine's `LoggedDecision`: the answer line at `:375`,
  `answer N [turn T, <phase> — <step>] player P <Kind> <Answer>[ forced]`, whose
  answer is `LoggedAnswer`'s `Debug` text (`Picks([0])`, `Number(2)`,
  `Allocation(…)`, `Order(…)`); a dealt start, `seed`, `pool` by its `Debug`
  and a `deck` line a seat; a scenario's at `:360–368`, `scenario <path>`,
  `seed` and the file's text between `begin scenario text` and `end scenario
  text`; and `outcome` by `Debug`. The engine's `state::decision_log` (120
  lines) holds `LoggedAnswer`, `LoggedDecision` and `DecisionLogHandle`, and no
  text. No line names the format or the engine, a dealt start records no
  `GameConfig`, and nothing reads a log back.
- **A kind's name is read off its `Debug` text in eight places**, not the three
  this section first counted. Six split the name off: the dev GUI's
  `kind_name` (`devgui/src/prompt.rs:132`), `SetupDriver::refuse_question`
  (`scenario/setup.rs:205`), and four test helpers, `variant`
  (`tests/phase_a6g_integration_test.rs:65`) and `variant_name`
  (`phase_cv2a_integration_test.rs:109`, `phase_cv2b_integration_test.rs:124`,
  `prompt_subject_test.rs:48`). Two in `test_support.rs` keep the whole text
  and match its start: `RecordingDecisionProvider::kinds`, whose 27 uses in 8
  files compare with `starts_with`, and `RejectionRecorder::rejected_at`. A
  prefix is looser than a name, since `Scry` begins `ScryOrder`, though no
  test matches `Scry` today. `ChoiceKind::as_str` replaces all eight: not
  `name`, which the CR uses for a card's name (CR 201) and for choosing one
  (201.4), the owner at #216's review.
  `ChoiceKind` still has 27 variants.
- **The replay exists only as a test**,
  `a_game_replayed_from_its_log_is_the_same_game`
  (`tests/phase_a6g_integration_test.rs:180`). It answers every line from a log
  kept in memory, with every seat stopping at every priority point, and
  asserts the same event log.
- **A recorded answer is a position in the option list**, wherever one is
  recorded: the dev GUI's log, and 299 scripted test answers in 54 files
  (`expect_pick_n` 262, `expect_allocation` 27, `expect_ordering` 10). Two
  places already find an answer by what it is, each with a matcher of its
  own: the setup driver finds a line's action and targets among the options
  by id (`scenario/setup.rs:234`, `:287`), and `phase_cv2a_integration_test.rs`'s
  `ById` picks objects by id.
- **An answer that does not fit is an engine panic.** `ui::ask`'s four
  `check_*` functions assert it (`ui/ask.rs:149`, `:224`, `:286`, `:385`), so a
  replay that hands a prompt the wrong answer ends as a validator's panic, not
  as a line that disagrees.
- `ScriptedDecisionProvider` matches a kind by its discriminant
  (`ui/decision.rs:404`) and ignores the player. Its `Drop` (`:435`), which
  skips while unwinding, is still the engine's only one. No `Cargo.toml` sets
  `panic = "abort"`.
- **A run ends early under `catch_unwind` in three places**: `fuzz_games`, once
  a game (`src/bin/fuzz_games.rs:927`); the dev GUI's engine thread
  (`devgui/src/bridge.rs:110`), which takes `WindowGone` as a superseded game's
  end; and SU-2's refusal test (`tests/phase_su2_integration_test.rs:49`),
  which catches the setup driver's `panic!` (`scenario/setup.rs:200`). Met in
  play, the dev GUI shows that refusal as "The engine thread panicked".
- **The memo audit** is `compute::audit_memo_hit`
  (`engine/layers/compute.rs:193`), with `audit_left_out` (`:219`) beside it
  for a card the pass leaves out, both compiled on `debug_assertions`.
- **The commit a record would name** is `state::trace::COMMIT`
  (`state/trace.rs:54`), which `mtgsim/stamp_commit.rs` stamps at build: twelve
  hex digits, `-dirty` after them when the tree has uncommitted edits,
  `unknown` without git.
- **The dev GUI's debug build runs the engine at opt-level 1** since the seats
  PR (§7b's decision 5), so the costs below are read again under it.
- **`codebase-state.md` item 200 is the tools'.** The dev GUI settles a
  scenario's seed at launch, so Reload ignores an edited `seed` line, and a log
  it cannot open shows as an engine panic. Slotted to the tools PR at ~30–50
  lines, and not in §8's first sizing.

**Measured** (a throwaway probe, 2026-10-02: eight dealt games on the
performance pool, two seeded random agents, each played to its end and then
replayed from its log). A game ran 18–40 turns and 465–884 lines, of which
141–284 were not forced. A whole game replays in 3–6 ms in release and in
0.7–2.9 s in debug at opt-level 0, where the layer memo's audit is most of
the cost.

**What a debug window can do about it** (the owner's question at review,
2026-10-03; measured the same day, the probe's games and the owner's machine):

| Build | Replay, a line | Whole game | Engine rebuilt after a change |
|---|---|---|---|
| debug before the seats PR: opt-level 0, the memo audit on | 2,527 µs | ~1.8 s | 2.4 s; the dev GUI's whole rebuild 6.7 s |
| debug, the audit off | 58 µs | ~40 ms | 2.4 s |
| debug, the engine at opt-level 1, the audit on | 312 µs | ~0.2 s | 3.4 s |
| debug, the engine at opt-level 2, the audit on | 279 µs | ~0.2 s | 13.5 s |
| release: no audit | ~6 µs | ~4 ms | the dev GUI's whole rebuild 15.4 s |

So the audit, which re-walks the layers at every memo hit, is about 98% of a
debug replay, and the same audit is why `engineering-practices.md` §10.4
measured 125 ms a prompt for the snapshot in debug. Two levers keep the debug
build and its audit:
- **The engine at opt-level 1 in the dev GUI's debug build**, two lines in
  `devgui/Cargo.toml` (`[profile.dev.package.mtgsim] opt-level = 1`): every
  engine call in the window about 8× faster with the audit still on, for about
  a second more per engine rebuild and a first build of ~23 s instead of ~14.
  It leaves `cargo test` in `mtgsim/` as it is. Opt-level 2 is no faster here,
  and its rebuild is four times as slow. **Built by the seats PR** (§7b's
  decision 5), with `engineering-practices.md` §10.4's `prompt_cost` read
  before and after.
- **No audit while replaying within a session.** An undo or a move to a
  savestate replays answers this window played earlier in the same process,
  so in the same build, and in a debug build with the audit on. The audit is
  off for that replay and back on at the hand-over: a debug-only switch,
  ~15–25 lines, about 170× on a Commander board's replay (below). **A load
  keeps the audit on**, since its lines may have been played by another
  build: decision 6's fix-and-reload loop, where the engine was changed after
  the save was written, makes that the usual case, and those lines are then
  replayed under the changed engine for the first time. A load in debug pays
  the audit once, up to ~5–7 s on a Commander board. (Corrected at SU-4's
  design: the first draft switched the audit off for every replay, on the
  premise that every replayed answer had been audited when first played,
  which holds within a session and not across one.)

**Read again at SU-4's design** (a throwaway probe, 2026-10-03, the owner's
machine: `fuzz_games`' dealing and its bot stack, ten games a board from seed
12345 on the performance pool, each played to its end and replayed from its
log with every seat stopped at every priority point). Every rebuild in §7.3
replays from the start, so a whole game's replay is what a late one waits:

| Build | Two seats, 60 cards: 820 lines a game | Four seats, 100 cards, 40 life: 2,766 lines | A line |
|---|---:|---:|---:|
| release | 6.5 ms | 21.8 ms | 8 µs |
| the dev GUI's debug engine with the debug checks off | 9.9 ms | 32.2 ms | 12 µs |
| the dev GUI's debug engine as built: opt-level 1, the checks on | 1.04 s | 5.42 s | 1.3–2.0 ms |

A replay costs what the game it replays cost, within 5%. A line's cost in
debug rises with the board, since the audit walks the board again at each
memo hit: the first three of those seeds, shorter games, read 0.19 s a game
at two seats, the 2026-10-02 probe's 0.2 s above, and 7.3 s at four, where
opt-level 0 read 0.97 s and 34.9 s. So late in a Commander game a debug
window waits about 5–7 s for a rebuild with the audit on and about 30 ms with
it off, and a release window about 20 ms.

Release stays what `engineering-practices.md` §10.4 says it is for, a large
board: it rebuilds the dev GUI in 15.4 s after an engine change against
debug's 6.7, and it runs without the audit, which found the engine bug in
`engineering-practices.md` §10.3. Checkpoints, a clone kept at each round
start and replayed from, would cut a late undo to a turn's replay, at
~100–200 lines and item 140's entry point; with both levers an undo late in a
Commander game waits ~30 ms in debug, so they are not needed.

### 7.2 The decisions

#### Decision 1 — how a run stops at the log's end

**The problem.** A `DecisionProvider` must return an answer and cannot say
"stop". The window does not need to: a replay that reaches its last line hands
the next prompt to the seats. Three things do. The setup driver's refusal
(§5.3) is a panic today, which the window shows as an engine bug. A replay that
disagrees with its log (another question at a line, or an answer the prompt
cannot take) must say which line. And a replay that an undo or a move
supersedes should end at its next answer rather than play on to its last line
(§7.3).

| | **A. A typed unwind** | **B. A stop through the boundary** |
|---|---|---|
| Code | a `Stop` value naming why (the log spent, a line that disagrees, a setup line refused, a replay superseded), raised with `std::panic::resume_unwind`; one engine function catches that payload and no other, and takes the run it wraps as a closure, since a game is entered six ways (`Game::setup`, `run`, `run_turn`, `resume`, `resume_turn_at_priority`, and `GameState::run_priority_round`) and each caller has its own: `game.until_stopped(\|game\| game.resume(&driver))`; the setup driver's `refuse` raises it instead of `panic!`; ~80–110 lines | every `DecisionProvider` method returns `Result<_, Stop>`, every `ask_*` call site passes it up, and `run`'s `Result<_, String>` gains a typed error; 33 provider impls (engine, tests, dev GUI) and 33 `ask_*` call sites in 15 files, ~300–600 lines changed |
| Performance | nothing on a game's path; a stop is one unwind | one branch per answer |
| Upkeep | control flow by unwinding, as the tree already ends a run early; it relies on `panic = "unwind"` and on no engine `Drop` writing `GameState`, both true today. Work in flight at the prompt is dropped, so a stopped game is read and never continued: the run methods refuse one | the stop is in the types and nothing unwinds; every provider written from now on handles it |

**Decided: A** (the owner, 2026-10-03). A stop leaves the state as it was at
the prompt, which is what the window shows there and what `Scenario::write`
writes. Continuing it would play a game missing what the unwound frames held
(a batch's decided members, a cast's remaining steps), which is why the run
methods refuse a stopped game rather than trusting every caller.

#### Decision 2 — what one undo removes

**The problem.** A person's click is one line among many. In the probe's games
about three lines in ten are choices, and the rest are passes the engine made
itself. Some choices at the window's seat are its decorators', not the
person's: `ManaWindowStop` closing a paid window, `AutoPayer` ordering
reductions, `AutoYield` passing. Whatever undo removes, the replay hands the
next prompt to the seats, and anything a seat answers without asking the
person is answered again at once.

| | **A. The log's last line** | **B. The window's last prompt** | **C. The window seat's last chosen line** |
|---|---|---|---|
| What the person sees | usually nothing: a forced pass, an agent's or a decorator's answer is given again at once, and the window shows the prompt it showed before, so undo is pressed until one of theirs comes back | the question they last answered, asked again, with everything after it removed and played again live | as B where they answered that line; where a decorator did, it answers again, and the window shows the prompt it showed before |
| Code | drop one line | the session records where each window prompt fell (the log's length when it was asked, one number a prompt), and undo goes back to the last | from the log alone |
| After a reload | the same | the record is in the save (decision 4) | the same |

**Decided: B** (the owner, 2026-10-03). Where the window was asked is the
session's record, not the game's: `engineering-practices.md` §10's rule keeps
who answered out of the log, so it goes in the save beside the savestates.
The window plays every seat (§7's "Seats"), so undo goes back to the last
prompt whichever seat it was for, and steps back through the whole table's
choices a prompt at a time. A yield ends at an undo,
since it was set at a prompt the undo removes; full control is the window's
and stays, as Reload keeps it. Where undo stops, at a scenario's first prompt,
after a load and during a replay, is §7.3's.

#### Decision 3 — the other seats under replay: closed, no agent in the dev GUI

**Closed at the second review round (the owner, 2026-10-03): the dev GUI
seats no agent, at least for v1.** "The devgui is for building cards and
debugging issues, not for normal play", so the window plays every seat (§7's
"Seats"), and the question this decision asked never arises: what an agent's
random stream should be after a replay hands over to it.

The options it weighed, for the record: answer the agent's lines from the log
and restart the agent at the hand-over; ask the agent again and check it
against the log; or answer from the log and key the agent's randomness by the
log's length, so that it continues as the original did. Nor does the question
arise in v1's GUI replaying a finished game, or in an AI harness replaying a
recorded one: both answer every line from the log and nothing plays on, and a
search forks a clone rather than replaying (item 140). The keyed agent is the
road if a later client ever seats an agent that plays on past a replay.

#### Decision 4 — where savestates and branches live, and loading a save

**The problem.** A save is a start and every answer (§2). With undo and
savestates it becomes a tree. An undo or a move to an earlier savestate starts
a branch once play goes on, the save keeps every branch, and a savestate names
a place on one. The tree also holds where the window was asked (decision 2),
which is the session's record and not the game's.

| | **A. Lines in the log file** | **B. A save file beside the log** | **C. A tree of logs** |
|---|---|---|---|
| Shape | the log becomes a journal: the engine's answer lines, with the session's lines between them for a window prompt, a savestate and a move back; reading it applies the moves to rebuild the tree | the log stays the engine's record of the line of play the window is on, written again by each rebuild's replay. The save, `<log>.save`, is the session's journal: the start, every answer as the engine wrote it, and the session's own lines | a folder for each session: each branch a whole log, the start and every answer from it, and an index of the savestates, the window prompts and which branch left which |
| Writing | appended and flushed a line at a time, as the log is today, so a crash loses nothing | the same, into the save | each branch's log as today; the index written again at each change |
| Reading | one parser; every reader of the game's record (a bug report, a test written from it) applies the moves first | a log is one line of play and replays alone, as now; the save's reader rebuilds the tree | every log replays alone; the index is a save file under another name |
| `engineering-practices.md` §10 | who answered (the window's prompts) sits in the log | the log is untouched | the log is untouched |

**Decided: B, the save a journal** (the owner, 2026-10-03). The engine's
thread appends to it, since that thread sees every line in order and knows
when it asks the window; a savestate is written at the click, under the lock
the record is written through, while the engine's thread waits at the open
prompt (as built at SU-5, where the design had it a reply like "stop
yielding"); and a superseded game's writer is shut before the next one
starts, since a game still running toward its next prompt would otherwise
append to the new line. Loading is §7.3's.

**At scale, for v1** (the owner, 2026-10-03), left open here to be settled in
SU-4's design before its build, is decision 6: the folders, what a record
says about what wrote it, a file per game or a batch, an index, and pruning,
for the dev GUI, v1's GUI and the AI harness.

#### Decision 5 — the exported test: dropped

**Dropped at the second review round (the owner, 2026-10-03).** The export
was a button that wrote the game on screen out as a Rust test: its start, an
`expect_*` line per answer and a check on the result, the shape §7 sketched
and `roadmap-v2.md` A6g listed with the tools. It had too many moving parts
for what it buys, and a generator with a bug writes wrong tests, which are
worse than none, with nothing placed to catch them. A regression test found in
the window is written by hand, from the log the window keeps. The
`ChoiceKind` built from a logged name, which §7 owed the export, goes with it;
`ChoiceKind::as_str` stays, since the replay compares kinds by it.

#### Decision 6 — the records at scale, for v1

> **Status:** decided at #216's review (the owner, 2026-10-03): its first
> question as D, and the four after it as proposed, the dev GUI's answers and
> the options Phase 10 starts from. It settles decision 4's "At scale, for
> v1".

**The problem.** Three writers will keep records of play, at three scales.
The dev GUI keeps a few a session, to replay and to attach to a bug report.
v1's GUI keeps a player's games to replay (`backlog.md` §2.38), hundreds over
months. The AI harness plays games by the thousand a minute and opens few of
them (Phase 10, `roadmap-v2.md` §E). Each record is a start and every answer,
in the engine's text (§7.3's table), and the engine's replay is what reads one
back. Where the bytes go, how a record is found again and when it is thrown
away are each writer's, as layout is a client's (`engineering-practices.md`
§10). So this decision draws one line: what the engine's text carries so that
every writer can do its part, which SU-4 builds, and the writers' own choices,
of which SU-5 builds the dev GUI's and Phase 10's design starts the other two
from the options below.

**Measured** (a throwaway probe, 2026-10-03, the owner's machine, release:
`fuzz_games`' dealing and its bot stack, twenty games a board from seed 12345
on the performance pool, each line formatted as the dev GUI writes it today):

| | Two seats, 60 cards | Four seats, 100 cards, 40 life |
|---|---:|---:|
| lines a game | 766 (379–1,602) | 3,349 (1,968–5,805) |
| of which forced | 69% | 81% |
| bytes a game | 61 KB (at most 130) | 274 KB (at most 473) |
| a game's CPU with no log | 6.1–6.4 ms | 35.9–36.7 ms |
| the lines formatted into memory | +4–5% (0.3–0.4 µs a line) | +2–3% (0.2–0.4 µs a line) |
| written to a file flushed a line at a time, as the dev GUI writes | +104–112% | +51–52% |
| written to a file through a buffer | +67–72% | +16–17% |

Creating an empty file costs 0.4–1.7 ms. The rest of a file's cost is the
operating system's, here Windows, so a harness reads its own on its hosts. At
about 28 Commander games a second a core, a harness that kept every record
would write about 7.5 MB a second a core.

**1. What a record says, and how another engine reads it.** A record can be
replayed by an engine built after it: the dev GUI's fix-and-reload loop (play
to a bug, save, fix the engine, load the save to watch the fix) does that on
purpose, and v1's GUI would after a release. How that engine reads a record
turns on how an answer is recorded. A position in the option list is exact
within one build and wrong across two without saying so: a build that lists
the options in another order makes option 1 Play Forest where it was Cast
Lightning Bolt, with the same player, kind, turn and step, so no check on
those sees it.

| | **A. As today** | **B. A header; another engine refused** | **C. A header; another engine replayed by position** | **D. A header; each answer recorded by what was chosen** |
|---|---|---|---|---|
| Shape | a record begins with its start, and an answer is a position | `decision log 1`, then `engine <commit>` from `state::trace::COMMIT`; a reader refuses a format it does not know, and a commit not its own | B's header; another commit is replayed, and the first line whose question disagrees names both commits | B's header; each answer is recorded as what was chosen, not where it sat: the card by its id and its printed name, as a scenario names it, the player, the action, the attack or block, the cost by its keyword and text, the number, color, counter or mana type. The replay finds that option in whatever list this build offers |
| Across builds | misread, silently | refused, which ends the fix-and-reload loop at its last step; and a commit proves less than it seems, since `-dirty` keeps one name across uncommitted edits | replayed, but a reordered list plays another game with no stop | replayed while the game is the same, through a reordered or longer list, and stopped at the first answer whose question or choice the engine no longer offers |
| Code | none | ~25–40 lines | B's | B's, and an option's identity written and compared by one matcher, ~130–200 with the replay's use of it |
| Upkeep | — | the format's number goes up when an older reader could not read the new text, which a pinned text shows in review | B's | B's; a new `ChoiceOption` variant does not compile until its identity is written, since the writer matches exhaustively |

**Decided: D** (the owner, 2026-10-03, at #216's review). Replay is exact
within a build and best effort across builds, where a fixed rule changes the
game anyway, and nothing is refused. Within a build each identity is found at
its own position, so undo, savestates and the tests replay exactly. Across
builds:
- a question with one legal answer (a forced line) is skipped when this build
  does not ask it, and answered when it asks one the record lacks, so an
  engine elision such as A6j's does not end older records;
- at the first answer whose question or choice the engine no longer offers,
  the replay stops (decision 1) and says where: "diverged at answer 341: it
  chose #12 Lightning Bolt, which is not offered". The window plays on from
  that board, and a test fails there;
- randomness follows the start's seed, so a build that draws it differently
  shuffles differently from there, which shows as a divergence at the first
  choice no longer possible; accepted (the owner);
- the engine line informs, in a bug report and in a divergence's message.

The writer reads an object's printed name and no characteristics, as the log
reads nothing through the layers (`state::decision_log`). The format line
names no crate, so item 208's rename strands no record. The matcher is the one
the setup driver and `ById` each keep a copy of (§7.1), and SU-4 moves both
onto it.

**2. Where each writer's files go.** The engine names no path. It formats
lines into the sink its writer gives it, so how a record is stored never
reaches the engine.

| Writer | Where | A record's name | Built by |
|---|---|---|---|
| the dev GUI | a scenario's games in its board's folder, `boards/<board>/`, and a dealt game's in `logs/` (§7b's decision 3); each save beside its log, as `<log>.save` | the seed, `-players-P` past two seats, `-2` on for a Reload. A load plays on in a new pair named as a Reload's, beside the loaded one, which it never writes | SU-3, and the save SU-5 |
| v1's GUI | the platform's per-user data folder (`%APPDATA%`, `~/Library/Application Support`, `$XDG_DATA_HOME`), which Tauri, the stack `backlog.md` §2.38 recommends, names. The other options: a folder the player picks, or one beside the program, which an installed program cannot write | the date and time the game began, and its table | Phase 10 |
| the harness | a folder per run, holding the run's configuration, the engine's commit among it, and the records the run kept. The other option: one folder for every run, each record's name carrying its run's | the game's number and seed | Phase 10 |

**3. A file per game, a batch per worker, or no file.**

| | **A. A file per game** | **B. A batch per worker** | **C. Kept on demand** |
|---|---|---|---|
| Shape | the dev GUI's: a file a game, flushed a line at a time, so a game that panics leaves its whole record | records back to back in one file per worker thread, which a reader splits at each record's format line | a game a seed replays writes nothing: its seed, its table and the engine's commit replay it, and one worth opening, such as a failure or a sample, is played again with the log attached, as `fuzz_games` is today (`--trace-game`, `--dump-events`). A game whose seats a seed cannot replay, such as a learned policy whose weights move or a remote agent, formats its lines into memory and writes them only if the game is kept |
| Performance at Commander scale | 274 KB a game, and +16–52% of its CPU here | the same bytes, and a file created per worker rather than per game | nothing for a seeded game; +2–3% in memory for another |
| Upkeep | at the harness's rate, about 7.5 MB a second a core | a splitter, ~15 lines in the engine when a batch is first written | the run's seeds, table and commit kept, which the harness needs to replay anything |

**Decided: A for the dev GUI, which has it, and for v1's GUI; C for the
harness, with B for what it keeps if that is many.** What SU-4 builds for all
three: the engine formats each line into its writer's sink and opens no file;
a record begins with its format line, so records can sit back to back and be
split again; and the text is a function of the game alone, with no clock in
it, so two records of one game are the same bytes but for the engine line and
the path a scenario was read from, and a diff of two engines' records is a
diff of the game. The splitter waits for its first writer.

**4. An index for browsing many records.**

| | **A. None: names and headers** | **B. An index file per folder** | **C. A database** |
|---|---|---|---|
| Shape | a browser lists the folder, and for more than a name reads a record's first lines (format, engine, start) and its last (the outcome) | a row a record (the file, start, seed, seats, turns, outcome), appended at each game's end | a table a record, in SQLite or the like |
| Performance | a few lines read a file at each browse; nothing while playing | a write a game | a write a game, and a dependency |
| Upkeep | none, since nothing is kept in step | a second record of each game, kept in step with the files by hand, which a deleted or half-written file leaves wrong (`engineering-practices.md` §2c) | a schema, and a migration at each change to it |

**Decided: A for the dev GUI**, which loads by path (`--load`) and is browsed
with the system's file browser. v1's GUI starts from A and moves to C if a
player's history wants search. The harness's index is its run's report,
written once at the run's end, naming the records it kept and why.

**5. Pruning.**

| | **A. None** | **B. By age or count** | **C. All but the marked** |
|---|---|---|---|
| Shape | the folders are git-ignored, and a person deletes what they no longer want | at start, records past a count or an age are deleted | as B, sparing a record with a savestate or a mark to keep it |
| Size | a log is 61–274 KB a game, and its save as much again or more, with its branches | bounded | bounded |
| Upkeep | none | deletes the save a bug report needed | a mark to set, and a rule that reads it |

**Decided: A for the dev GUI.** v1's GUI gets a setting, its design's; a
harness run's folder goes with the run.

**So the dev GUI, the first consumer, takes**: D's header and answers, its
folders as SU-3 built them, a file a game, no index and no pruning. v1's GUI
and the harness choose at Phase 10's design, from these options, and
`backlog.md` §2.38 and `roadmap-v2.md` §E point here.

### 7.3 What the window shows, and where each surface lives

**Loading.** The dev GUI writes a save beside every game's log, so `devgui
--load FILE` takes a save; a plain log loads as a save with one line of play
and no record of where the window was asked. The window replays the line the
save ended on, saying how many of its answers it has replayed while it does,
and then asks the next question; a load in debug keeps the audit on (§7.1),
so on a Commander board that takes seconds. Play goes on in a new log and save
beside the loaded pair, named as a Reload's are, and the loaded files are
never written (decision 6). The game's header gains **Undo answer**, named
apart from the editor's own Undo, which SU-3 put in the editor's header;
Savestate, which marks the open prompt and names it for its turn and step;
and a menu of the savestates and the line most recently left, a "back to where
I was", each replayed on a click. The save keeps every line, and the menu
lists only those, so lines played and abandoned do not crowd it (the owner,
at #216's review). They sit beside Reload, after the Play | Edit switch. A
savestate is written out as a scenario by moving to it and clicking
"Save board as scenario". Reload is unchanged: a scenario read again is a new
start, so a new save.

**Where undo stops** (the owner's question at review, 2026-10-03):
- **At the window's first prompt** there is nothing to undo, and the button is
  off with a line saying so. In a scenario that prompt comes after its setup
  actions, which belong to the start (§5.3), not to the play: going further
  back is Reload, which plays them again to the same prompt.
- **Right after a load**, undo walks back into the save. The save records
  where the window was asked, so the first press returns to the window's
  previous prompt in the line it loaded, as it would have in the session that
  wrote it. A plain log has no such record, so its undo starts with the first
  answer given after loading.
- **While a replay runs**, Undo and the menu stay live. Each press moves the
  target back one more window prompt and starts the replay again, and the
  replay it supersedes stops at its next answer (decision 1). Three presses go
  back three prompts and wait for one replay. Savestate is off, since no
  prompt is open.

**`engineering-practices.md` §10's question** for each new surface: would a
second display client compute the same thing from the same facts?

| Surface | Home | Why |
|---|---|---|
| `ChoiceKind::as_str` | the engine, `ui::choice_types` | the log, the prompt and the setup driver's refusal spell a kind one way |
| An option's identity, and the one matcher that finds it in a list | the engine, `ui::choice_types` | a recorded answer, a setup line and a test's script name what was chosen alike (decision 6) |
| A record's header, a start's text and a line's, written and read | the engine, `state::decision_log` | every client's log and save say the same words and what wrote them, and one parser reads them (decision 6) |
| The replay | the engine, a provider in `ui` | a second client's undo replays the same way |
| The stop | the engine, `ui::decision` and `Game` | it crosses the boundary |
| The save's journal and tree; where the window was asked | the dev GUI | which prompts reach a person is one client's seat stack; it moves to the engine when a second client reads saves (`backlog.md` §2.38) |
| The replaying status, the buttons, the menu | the dev GUI's drawing | how it looks |

---

## 7a. Building a scenario from scratch

**The goal** (the owner, 2026-10-01): a developer sets up an obscure board to
test a new card or mechanic, without a game to save one from. Five things stand
in the way, and each has a step with its slot.

1. **Knowing what to write** (SU-1). `mtgsim/scenarios/` holds a commented
   template naming every word and its default, and a handful of samples (combat
   with an Aura, Humility against Opalescence, a planeswalker, four seats with
   a commander). A test loads each file, so none goes stale. Names are exact,
   as the writer writes them; a name that matches no registered card is
   refused with its line, which is all a typo needs. A refusal says what to
   change as well as what is wrong: "Grizzly Bears is attacking in the
   precombat main phase; attackers exist from the declare attackers step to
   the end of combat (CR 506.4, 511.3); set `step declare attackers` or
   later."
2. **A card that is not registered yet** (SU-1, ~15–25 lines). The project
   registers a card only once the engine plays it, because `determinism_test`
   and every `--pool stress` game play the whole registry. A card under
   development therefore has no name a scenario can use, which is the moment a
   developer most wants one. A list of cards in development beside the
   registry fixes that: scenario loading reads it, and the fuzz pools and
   `determinism_test` do not. A card leaves the list in the commit that
   registers it. A test can do the same today by registering a fixture in the
   registry it passes.
3. **State only an effect makes** (SU-2). A stack ten deep: setup actions
   play it from the board. A creature under Act of Treason, a regeneration
   shield: setup actions cast them, and the seats' first passes resolve them
   (§5.3). The kinds an effect leaves behind as stored state (a token, a copy)
   get words in the order §4.3's count gives.
4. **A new mechanic's own state** (the growth contract, §5.2). The PR that adds
   a field adds its word, because the writer does not compile until it does,
   so a mechanic's state can be set up the day the mechanic lands. The monarch
   (CR 724), the initiative (725) and day and night (730) will each arrive with
   a word.
5. **Typing a board in at all** (SU-3, §7b, built at #215). A board editor in the
   dev GUI: search a registered name, put the card in a zone, then click its
   controller, status, counters, attachment and combat. It edits the same
   `Scenario` value the parser builds and saves it through `Display`, so it
   adds no third road. **Before the tools** (the owner, 2026-10-03, at #212's
   review), since it makes every later test board quick to build, a four-seat
   Commander board above all. It had moved up to follow SU-2 at #206's review,
   and back at SU-2's.

So a board reaches a scenario three ways, all into one `Scenario` value:
written as text (SU-1), saved from a game and edited (SU-1's writer), or built
in the editor (SU-3, §7b).

---

## 7b. The board editor (SU-3)

> **Status:** built in SU-3 (#215, §8's ✅ section), 2026-10-03. It grows §7a's
> item 5. The owner decided 1 and 4 at the first review round (2026-10-03):
> one window, and the seats PR ahead of SU-3 with four seats in it. At the
> second, 2 and 3: the board's own words, the rest to follow as advanced
> settings, and a `boards/` folder with a folder per board, the owner's
> direction. At the third, 5: the engine at opt-level 1 in the dev GUI's
> debug build, in the seats PR. The rest is settled by §7a and
> `engineering-practices.md` §10, or is this design's own and reviewed with it.

**Where it sits.** Nowhere on the event path: before the first event, on the
start's side of `Scenario::build`, beside the parser and the writer. It is a
third way *into* the one `Scenario` value, and adds no way to build a board
from it. Nothing in this section runs in a game: the editor, the seats change
and decision 5's lever are the dev GUI's, and the engine gains only a listing
and some spellings made public, so no path a game runs changes (§8's A/B).

```
a file ── parse ─┐                           ┌─ Display ─▶ a file, or the clipboard
a game ── write ─┼─▶ Scenario ◀── a click ───┤
an empty board ──┘    │    └─ undo: the boards before it
                      └─ build ─▶ the loader's refusal, marked on its card; or Play: Game::resume …
```

**What the tree has** (read 2026-10-03):
- The value has nine fields: players, starting life, seed, turn, active,
  step, the player words, the card lines and the setup actions. A card line
  is `CardLine { kind, card, copies, words }` over 8 `LineKind`s and 15
  `CardWord`s, and a player's line is one of 6 `PlayerWord`s. §5.1's table
  has 35 rows, four of them not words (who has priority, the first draw, an
  X, a mode).
- `parse` numbers each item by its line, `write` numbers every item 0, and
  `Display` writes no numbers.
- `CardRegistry::card_names()` lists the 181 registered cards, sorted. A card
  in development (`IN_DEVELOPMENT`, `registry.rs:479`, empty today) can be
  created by name, and nothing lists it.
- The counter kinds a permanent can carry are the engine's own (`CounterType`:
  +1/+1, −1/−1, loyalty, the keyword counters such as flying). They are listed
  by `CounterType::ALL` and spelled by `CounterType::name`, both public, and
  the editor's counter control reads them. A card word's text
  (`card_word_text`), the step words (`turn_positions`, `position_word`) and
  the tag spelling (`tag_letters`) are private to the scenario module.
- The dev GUI: 2,737 lines in `src` and 682 in `tests`; `--scenario`, Reload
  and "Save board as scenario", which writes `logs/<stem>-turn-T.scenario`;
  `Snapshot::board_text`, the written board, built at every prompt. The window
  is seat 0 (`WINDOW_SEAT`, seven lines), the agent is built at
  `bridge.rs:181`, two decks are dealt at `:153`, and a scenario that is not
  two-seat is refused at `:204`. (The seats PR changed all four: decision 4.)

**Measured** (a throwaway probe, 2026-10-03, the owner's machine, the median
of 30): a board written and read back (`Display`, then `parse`), and built.

| Board | Objects | Round trip, debug | `build`, debug | `build`, release |
|---|---:|---:|---:|---:|
| `template.scenario` | 25 | 0.11 ms | 0.18 ms | 0.03 ms |
| `four-seats-commander.scenario` | 34 | 0.04 ms | 0.07 ms | 0.01 ms |
| `holy-strength.scenario` | 42 | 0.07 ms | 0.15 ms | 0.03 ms |
| `large.scenario` (§10.4's) | 188 | 0.20 ms | 0.62 ms | 0.16 ms |

So the loader can check the board at every click, at the click and on the
window's thread (`engineering-practices.md` §10.1, items 1 and 2): under a
millisecond in debug on the largest board the tree has.

### 7b.1 The model

**Plain Rust under tests, `devgui/src/editor.rs`, and egui only draws it**
(§10). An `Editor` holds:
- **the board**, a `Scenario`;
- **undo**, a stack of the boards before it: a click that changes the board
  pushes the board it changed, and Undo pops one;
- **the loader's verdict** on the board: `build`'s refusal, or none;
- where the board came from and where it saves (decision 3), the card being
  edited, and the search.

**A click makes a new board, renumbered through its text.** Each
`EditorInput` edits a copy of the board. The copy is written and read back,
`Scenario::parse(&board.to_string())`, before it replaces the board and is
checked. Three things follow:
- The editor never holds a board its own text cannot say, so Save and Play
  see exactly the board the editor shows.
- Each item's line number is its line in that text, so the line a refusal
  names (`ScenarioError::line`) is a card, a player's line or a game fact the
  editor can mark, with no table of its own.
- A board whose text does not read back is refused with the parser's message,
  and the edit is not made. A name holding `|`, `#` or `[`, which no
  registered card has (§4), is the one way to reach that.

**The check is `build`.** Its refusals say what to change (§4.1). The window
shows the refusal under the board and marks the card it names. The editor
checks nothing itself.

**A reference is a click on the card it names.** Attached to, attacking a
permanent and blocking each name another card, and the editor writes that
card's name and tag. Two permanents sharing a name, or two commanders, need a
tag in every reference to either (§4.2). So when a card is put on the
battlefield or made a commander, and its name is shared there, it takes the
first unused letter in the writer's spelling. The card it shares the name
with takes one too if it has none, and so does every reference to that card:
attached to, attacking, blocking, the `counters:` and `this turn:` lines, and
commander damage. A tag never changes after that, so a reference keeps naming
its card. A tag left on a name no longer shared is harmless, since a tagged
name still matches only itself. A setup action's name is not rewritten, since a
setup action may name a card in a graveyard or a spell. If the new card makes
one ambiguous, the loader refuses it, naming its line and saying to tag it.

**Order is the file's, and the rules read it.** On the battlefield the lines'
order is CR 613.7d's timestamps (§3.1). A library is listed top first and a
graveyard bottom first. The editor lists each zone in its lines' order. A card
moves up or down in its zone, or to either end. A new card goes last, which is
the latest arrival, the bottom of a library and the top of a graveyard.
**Attaching** a card to a permanent listed below it moves the card's line just
below its host. The loader needs the host first, and CR 613.7e gives an
attached Aura or Equipment a new timestamp anyway.

**Where a card is put says whose it is.** A seat's hand, library and
graveyard hold cards that seat owns, and a seat's battlefield holds what it
controls, written `controller p`, which the owner defaults to (§5.1). Exile
and the command zone are listed under each seat as the cards it owns there,
written `owner p`, which the loader requires. So each "+" names a seat, and
an owner other than the controller is a word on the card.

**Moving a card in the editor takes the words that no longer fit.** These
are the person's clicks on the board being edited; no file changes until
Save. Say Grizzly Bears is on player 0's battlefield, tapped and attacking,
with Holy Strength attached to it, and the person moves the Bears to player
0's hand. A card in a hand is not tapped and attacks nothing, so the Bears'
line loses the words only a permanent has (controller, tapped, arrival,
counters, damage, attachment, combat) and becomes `hand 0: Grizzly Bears`.
Holy Strength's `attached to Grizzly Bears` goes too, since it would name a
permanent that is no longer there, and removing the Bears does the same. That
leaves Holy Strength attached to nothing, which §4.1 builds and the game's
first check puts into the graveyard (CR 704.5m), as play would. Undo brings
all of it back.

**Search** filters every name a scenario can use by a case-insensitive
substring: the registered cards, then the cards in development, marked as
such. Those need a listing, `CardRegistry::names_in_development()`, a few
lines. `card_names` stays as it is, since `random_deck` and the fuzz pools draw
from it.

**Naming a card in play searches another list** (the owner's question at
review). Pithing Needle's controller chooses a card name, and Petrified
Hamlet's a land card name. CR 201.4 allows the name of any card in the Oracle
reference, about 27,000 names, most of them not registered, and 201.4a filters
them by characteristics. That choice has no shape in `DecisionProvider` yet
(`backlog.md` §2.4, unowned), and its list is not the registry. What it shares
with the editor is the client's half: a box that narrows a long list of names
as the person types. So the editor's search is a view-model piece over any
list of names, which the window's prompt for that choice can take when §2.4
lands. In both cases the list is the engine's.

**A card shows its line**: its name and tag, then its words as the file spells
them (`tapped · +1/+1 2 · attacking player 1`), so `card_word_text` becomes
`CardWord`'s `Display`. It does not show the layers' output. A board the loader
refuses is not a game, and one it accepts becomes a game only when Play starts
it. What the board is as a game (its power and toughness under Humility, who
is summoning sick) is Play's: one click, and the window shows it at its first
prompt.

**The window's controls:**
- the game's facts: players, starting life, seed, turn, the active player and
  the step (`turn_positions` and `position_word`, made public), and each
  player's life;
- each seat's zones: battlefield, hand, library, graveyard, and what it owns
  in exile and the command zone. Each card is a button showing its line, as
  the play view draws a card, and a click on it opens its words below. A
  zone's "+" puts the name chosen in the search there;
- the card being edited, its words as controls:
  - a seat chooser for controller and owner;
  - toggles for tapped, commander, blocked and dealt first-strike damage;
  - arrival: before the game, this turn, or turn N;
  - a counter for each kind in `CounterType::ALL`, each with − and +;
  - damage;
  - for attached to, attacking and blocking, a button that waits for the
    click on the card it names;
  - then copies (`xN`) off the battlefield, the zone, up and down, and Remove;
- Undo, Play, Save, and the file it saves to (decision 3).

**§10's question** for each new surface: would a second client compute the
same thing from the same facts?

| Surface | Home | Why |
|---|---|---|
| The value, `parse`, `Display`, `build`, the refusal | the engine, `scenario` | as built in SU-1 and SU-2 |
| The names a scenario can use | the engine, `CardRegistry` | every client searches the same names |
| A card word's text, the step words, the tag spelling | the engine, `scenario` | the grammar's spellings: the editor shows what the file will say |
| The search's filter, undo, the tagging and ordering rules, the card buttons | the dev GUI | one client's way of editing a value; none of it reaches a game |

### 7b.2 The decisions

#### Decision 1 — one window, or a separate editor

**The problem.** A board is built to be played. Testing a card is a loop:
build a board, play to the interaction, look, change the board, play again.
And a board reached in play is the start of the next one. Where the editor sits
sets how many steps each turn of that loop takes.

| | **A. One window, two modes** | **B. A window that only edits** |
|---|---|---|
| Shape | Edit beside Play, a switch in the header. "Play this board" saves the board to its file (decision 3) and starts it as Reload starts a file. Switching back to Edit returns to the board as it was, undo and all. At any prompt, "Edit this board" opens the board the game is at through `Scenario::write`, which the snapshot already carries, with the writer's report shown | `devgui --edit FILE` opens a window that only edits, and Save writes the file. A second window, `devgui --scenario FILE`, plays it, and Reload picks up each save. A board reached in play comes back through "Save board as scenario" and `--edit` |
| Code | `Session` keeps an `Editor` beside its game, and a mode. About 60 lines more than B | a second `eframe::App`, chosen at launch, ~40 lines; the session is unchanged |
| Performance | the dev GUI's alone, since nothing here runs in a game: the game waits at its prompt while the person edits, its thread blocked on `recv`, as it is now between clicks | the same |
| Upkeep | one header gains a switch; the two views share the item and zone drawing | two windows kept in step by hand: each turn of the loop is Save, switch windows, Reload |

**Decided: A** (the owner, 2026-10-03). Each way round the loop is one click,
and "Edit this board" is the written board the window already has. To be
looked at again once built, if one window proves too busy to read.

#### Decision 2 — what the first editor edits

**The problem.** A scenario file can say 31 kinds of thing, §5.1's words. The
question is which of them the first editor gives a control (a toggle, a
number, a click on a card) and which it leaves to typing in the file. Each
control is code and tests, and some are lists: a setup action is a seat, a
card and its targets in order.

**By example.** Take a board like §4's: Holy Strength on an attacking Grizzly
Bears, with Wall of Stone blocking. Both options build all of it by clicking:
- the turn and the step;
- Grizzly Bears on player 0's battlefield, set tapped and attacking player 1;
- Holy Strength there too, attached by a click on the Bears;
- Wall of Stone on player 1's side, blocking by a click on the Bears;
- a Lightning Bolt in player 0's hand.

Now add two things: "player 0 has cast a spell this turn" (a history count,
row 13) and "a Lightning Bolt on the stack targeting the Wall" (a setup
action, rows 31–32: `then: player 0 casts Lightning Bolt | targeting Wall of
Stone`). Under A those two lines are typed into the saved file, which then
opens in the editor again. The editor shows them as text, keeps them on every
save and can remove them, but has no control to change them. Under B the
editor has a spells-cast count on each player, and a list of setup actions,
each built from a seat, a card in its hand and targets clicked in order.

| | **A. The board's own words** | **B. Every word** |
|---|---|---|
| §5.1's rows clicked | 21: the game's facts (1–5), life (8), the zones and their lines (15–17, 19, 20), `commander` (18), and a permanent's words (21–24, 26–30): controller and owner, tapped, arrival, counters, damage, attachment, combat, dealt first-strike damage | 31: A's, and player counters (9), lands played (10), left the game (11), commander damage (12), history (13), `this turn:` (14), `counters:` lines (25), and setup actions with their targets and ability (31, 32, 34) |
| The rest | 10 rows shown as their text: under the player, on the card's button, or listed under the board (`counters:` and `this turn:` lines, setup actions), each removable. Edited in the file, which then opens again | none |
| Code and tests | ~1,500–2,200 (§8) | ~2,100–3,100. The ten rows add ~600–900, setup actions the most (~250): a list of lines, each a seat, a verb, a card from that seat's hand or board, an ability, and targets clicked in order |
| Upkeep | a word a later PR adds (§5.2's growth contract) is kept and shown as text until a board needs a control for it | each word a later PR adds owes a control and its test in that PR |

**Decided: A** (the owner, 2026-10-03), with B to follow as the editor's
**advanced settings**: a switch that shows controls for the ten rows A leaves
as text. Those controls can be typeable, a field that takes any line of the
grammar and reads it with the parser (~40–60 lines), and clickable, a control
per row at the sizes §8 gives (~600–900 in all). A6g's row slots them after
the "why" panel. A four-seat Commander board is built from A's words: four
players at 40 life, commanders in the command zone and on the battlefield,
and combat. Commander damage (12) and a player who has left (11), which
`four-seats-commander.scenario` uses, are the likeliest first advanced rows.

#### Decision 3 — where a built board is saved, and how a file comes back

**The problem.** A built board is kept to play again, to commit as a sample or
a test board, or to paste into a test, which writes its scenario inline as text
(§4). An existing file must open in the editor: the template, a sample, a test
board, a board saved from a game. `Display` writes the board and none of its
comments, so a file saved over loses its comments. The template is mostly
comments.

**It is one corner of a wider question**: §7.2 decision 4's "At scale, for
v1", where each of the dev GUI's files goes (the owner, at review). The dev
GUI writes two kinds of file, told apart by who reads them:
- **Records of play.** A decision log per game start (`logs/seed-41.log`,
  `-2` for a Reload), and a save beside each from SU-4. They are written as a
  game is played, and a replay reads them. There is one per game, and v1's GUI
  and the AI harness will write them by the thousand. The folders, headers, batching,
  index and pruning that §7.2 leaves to SU-4's design are all about these.
- **Boards.** A board saved from a game ("Save board as scenario", written to
  `logs/<stem>-turn-T.scenario` today), and from SU-3 a board built in the
  editor. A person keeps, edits and loads them. There are few, and the good
  ones are committed as samples (`mtgsim/scenarios/`) or review boards
  (`devgui/tests/scenarios/`). Neither v1 writer §7.2 names writes boards:
  v1's GUI keeps a player's games, and the harness, like `fuzz_games
  --scenario` today, reads boards and writes none.

**So the first part: boards get a folder of their own**, `boards/` beside
`logs/`, which both saves write to and which git ignores. Today a board saved
into `logs/` shows in `git status`, since only `*.log` is ignored there. Then
`logs/` holds only records of play: SU-4 lays them out for v1 without boards
among them, and pruning old records never takes a board. About 10 lines, and
"Save board as scenario" moves with it.

**The second part: how Save writes inside it.**

| | **A. A new file every time** | **B. Its own file** | **C. A name typed in the window** |
|---|---|---|---|
| Shape | Save writes the first unused `boards/<stem>-N.scenario`, where the stem names where the board came from; nothing is overwritten | a board opened from `boards/` saves back to its file, keeping its leading comment block. Any other board (a new one, one from a game, a committed sample or test board) saves first to a new file in `boards/`, as A does, and that file is then its own | a name and a folder (`boards/`, `mtgsim/scenarios/`, `devgui/tests/scenarios/`), typed in the header |
| Code | ~15 lines, `saved_board_path`'s rule | ~35 | ~60 |
| Risk | a board worked on over an afternoon leaves a file per save | a comment written into a board's body by hand is lost at its next save | a committed file overwritten, comments and all, by a mistyped name |

**Common to every option:**
- **Copy as text** puts the board's text on the clipboard (egui's
  `copy_text`, ~5 lines), for a test's inline scenario.
- **A file comes back** four ways. `devgui --edit [FILE]` starts in the editor,
  on an empty two-seat board if no file is given. "Edit this board" works at
  any prompt. In a scenario game, "Edit the scenario" opens the editor's own
  board if Play started it, or else the file, read again with its setup
  actions. And a list in the header names the boards in `boards/` and the
  `.scenario` files under `mtgsim/scenarios/` and `devgui/tests/scenarios/`,
  ~40 lines. §7 deferred that list until there were more than a handful of
  files, and there are eleven committed ones.

**Decided: `boards/`, and B inside it** (the owner, 2026-10-03), with Copy as
text and the list. Working on one board writes one file, and the editor never
writes a committed file, so a board becomes a sample by a copy that its PR
reviews.

**A folder per board** (the owner's direction at the second round, designed
here). Each board gets a folder in `boards/`, named for the board, holding the
board and the record of every game played from it:

```
devgui/
  boards/
    holy-strength/
      holy-strength.scenario   the board, as Save or Play last wrote it
      seed-0.log               a game played from it; Reload's is seed-0-2.log
      seed-0.log.save          SU-4's save, beside its log
    board-3/                   a new board, named at its first Save or Play
  logs/
    seed-41.log                a dealt game, which starts from no board
```

- **The board is the key, as the log already says.** Each log names its start
  in its header and holds the board's text, so the folder shows what the log
  records, and a log read alone still replays (§2).
- **Play saves first**, then starts the board from its file as Reload does. So
  every game in a folder sits beside the board it began from or a later edit
  of it, each log keeping the text it began from. Play is live only while the
  loader accepts the board.
- **A new board is named `board-N`**, the first unused, at its first Save or
  Play, and is renamed by renaming its folder and its file.
- **"Save board as scenario"** makes a new board, so a new folder, named for
  the game's start and turn (`holy-strength-turn-3/`), with a leading comment
  naming the log it came from.
- **A committed board** played with `--scenario` (a sample, a review board)
  records its games in `boards/<its stem>/`, which holds no board file until
  the editor saves one there; the committed file is never written. Two
  committed boards with one stem share a folder, and each log still names its
  file.
- **A dealt game** has no board, so its records stay in `logs/`, named for the
  seed as today.

Deleting or sharing a board's folder takes its history with it. For v1 it
gives SU-4's design one fact: in the dev GUI, records are grouped by the start
they replay from. Where v1's GUI and the harness put theirs stays SU-4's
(§7.2). About 20–40 lines over a flat `boards/`, and starting Play from a file
drops the start from text that decision 1's first draft needed.

#### Decision 4 — the window playing every seat, and past two seats

**The problem.** §7's "Seats" settled that the window plays every seat and
the agent leaves the dev GUI (2026-10-03). Two questions were left open:
whether that change rides in SU-3 or in a PR ahead of it, and whether the dev
GUI goes past two seats now. The editor builds a board of any seat count the
loader takes, but the bridge refuses one that is not two-seat
(`bridge.rs:204`). So the four-seat Commander board, the reason the owner moved
SU-3 up, could be built and not played. A6g's row lists four seats out of
scope.

**What the change is**, at any seat count:
- every seat gets the window's stack of decorators and a yield of its own, and
  full control stays one switch;
- the prompt names the seat it asks, and `WINDOW_SEAT` goes;
- **the words that assumed one seat name it** (the owner, at review). "Pass
  until my next turn" becomes "Pass until Player 2's next turn", for the seat
  being asked, and so do the header's "Your decision" and "you win" and the
  board's "(you)". Each seat's yield already measures from that seat
  (`Yields::holding` takes the player), so only the words change;
- **where each seat is drawn**: the board stacks the seats one above another,
  and today the window's own seat is the bottom one, beside the prompt. That
  order stays at every prompt and the seat being asked is marked, rather than
  the asked seat moving to the bottom, which would reshuffle the board at
  every pass of priority. This is a layout choice, so it is the client's
  (§10);
- the tests that play the window's part answer for every seat, and the review
  pictures are drawn again.

Past two seats it adds:
- `--players N` for a dealt game, dealt as `fuzz_games --players N` deals, so a
  four-seat fuzz game's printed seed deals its decks;
- a scenario's own seat count, with the refusal at `bridge.rs:204` gone;
- tests at four seats: a dealt four-seat game and
  `four-seats-commander.scenario`, each played by rule to its end. Each is
  timed in debug, and runs in release only if it is slow, as
  `clone_bound_test` does.

| | **A. A PR ahead, four seats in it** | **B. Inside SU-3, four seats in it** | **C. A PR ahead, two seats; four later** |
|---|---|---|---|
| Size | ~200–320 with tests, a review of its own (§8); SU-3 stays ~1,500–2,200 | SU-3 ~1,700–2,520, at the band's top before any overrun | ~150–240; four seats a later PR of ~50–80 |
| What lands first | four-seat boards written by hand play at once, and the editor's Play plays any board it builds | everything at SU-3's merge | the editor's four-seat boards cannot be played until the later PR |
| Review | a small GUI PR: the pictures and a click script | the largest GUI PR yet, reviewed by its pictures | two small PRs |

**Decided: A** (the owner, 2026-10-03), as the seats PR: A6g tooling like
playable, with no SU code, since it is the dev GUI's and not CR 103's.
Decision 5's lever rides in it. **Built by the seats PR** (#214), ahead of
SU-3's build.

#### Decision 5 — the engine at opt-level 1 in the dev GUI's debug build

**The problem, from the start.** A build sets two separate switches, which
`cargo` sets together for debug and for release:

| | How hard the compiler optimizes (`opt-level`) | The debug checks (`debug_assertions`), the layer memo's audit among them |
|---|---|---|
| a debug build (`cargo run`, `cargo test`), today | 0: the code as written; quick to compile, slow to run | on |
| a release build (`cargo run --release`) | 3: fully optimized; slow to compile, fast to run | off |
| **the lever**: the engine, in the dev GUI's debug build | 1: lightly optimized | on, unchanged |

Optimization changes how fast the code runs, never what it does. Under the
lever the engine keeps the same cache, the same audit and the same checks, and
computes the same answers, faster. The audit belongs to the second switch,
which the lever leaves on.

**The check that matters here is the layer memo's audit.** An object's
characteristics through the layers (CR 613: its types, power and toughness,
abilities and controller after every continuous effect) take a walk over the
effects to work out, and the engine asks for them constantly, at every rule
check and every prompt. So the engine keeps each object's last answer, the
layer memo, and serves it again until something a walk reads changes. Each
such change bumps an epoch number, which retires every kept answer. A write
that forgets the bump would leave an old answer in use, a wrong power or
controller, and nothing would say so. So at every answer the memo serves, a
debug build also walks the layers afresh, and panics if the two differ
(`compute::audit_memo_hit`). That caught a real bug in §10.3's random clicks:
a player's loss bumped nothing. It is why cards are tested in a debug window,
and it is also why that window is slow. Each served answer costs a full walk
anyway, which is about 98% of a debug engine call (§7.1) and 125 ms a prompt
on a large board (§10.4). A release build has no audit.

**The choice is yes or no** to two lines in `devgui/Cargo.toml`:
`[profile.dev.package.mtgsim] opt-level = 1`. When the dev GUI is built in
debug, they compile the engine crate with light optimization, and everything
else stays as debug has it: the audit and every other debug assertion, debug
info, the overflow checks. Only the engine runs faster: §7.1 measured every
engine call about 8× faster with the audit on, and an engine rebuild 2.4 →
3.4 s. The lever applies only to builds of the dev GUI. Release builds, and
the engine's own `cargo test` in `mtgsim/`, which reads its own manifest, are
the same either way. **Why now:** the seats PR makes the window answer every
seat, so the dev GUI's tests, which play whole games through the window, do
more work.

**Measured today**: the dev GUI's CI test step, `cargo test --locked --lib
--test headless_game --test random_clicks`, each arm in a target directory of
its own (2026-10-03, the owner's machine).

| | Cold build of the test targets | The tests, two runs | of which `random_clicks` |
|---|---:|---:|---:|
| opt-level 0, as today | 50.0 s | 17.9 s, 16.5 s | 15.3 s, 14.8 s |
| the engine at opt-level 1 | 48.8 s | 4.7 s, 4.0 s | 3.4 s, 3.3 s |

A cold build is no slower, since eframe's dependencies are most of it. CI's
devgui job ran 71 s at #211's merge (the build 14 s, the tests 27 s), beside the
check job's 133 s.

| | **A. In the seats PR** | **B. Not now** |
|---|---|---|
| Shape | the two lines, with their reason | — |
| Performance | a debug window's engine calls ~8× faster with the audit on; the dev GUI's tests ~4× faster, which pays for answering every seat; an engine rebuild ~1 s slower | the seats PR's tests at opt-level 0: the step, ~17 s here today, grows with the prompts every seat adds, the four-seat games most |
| Upkeep | the audit stays; a debugger may show some of the engine's values as optimized away; `cargo test` in `mtgsim/` is unchanged; `main.rs`'s advice (debug to test cards, release for a large board) is read again | — |

**Decided: A** (the owner, 2026-10-03), in the seats PR. `prompt_cost` is
read before and after in the debug build, since its §10.4 reading is release,
which the lever does not touch, and the CI step's time is quoted from the PR's
run.

**Logged for the dev GUI audit** (the owner, the same day). The lever makes
the engine faster, but most of a debug window's cost is still the layer memo's
audit, about 98% of a debug engine call (§7.1). So the audit that ends A6g's
row looks at what the audit costs the window, and what the window can do about
it without losing what the audit catches. One answer is already designed:
SU-4's switch that skips the audit while replaying (§7.1).

### 7b.3 What SU-3 and the seats PR changed that §7.1–§7.3 name

Each PR body lists these under "for SU-4" (§8). As built, read again at SU-4's
design against `fb1767a`:
- **a folder per board** (decision 3, `boards::Folders::game_log`): a
  scenario game's log is `boards/<stem>/seed-N.log`, a `--scenario` launch's
  too, and a dealt game's is `logs/seed-N.log`, or `seed-N-players-P.log`
  past two seats; a Reload takes the next free `-2`, `-3` beside it
  (`session::own_log`). So the save beside each log is in one folder or the
  other, and `--load` reads both. Play starts from the board's file as Reload
  does, so the tools meet no new kind of start, and §7.3's "a scenario read
  again is a new start" holds for each Play;
- **a `GuiSeat` and a stack of decorators per seat** (the seats PR, #214),
  `WINDOW_SEAT` gone, the prompt naming its seat: §7.2's decision 2 ("the
  window's last prompt, whichever seat it was for") is built against these;
- **the session**: a game is optional, `Session::setup` and its engine `None`
  until one starts, as under `--edit`; `launch::read` returns a `Start`, a
  game or the editor, which `--load` joins as a third; the header's `message`
  replaces `saved`; and `Input` is no longer `Copy`, since the search's text
  rides in it;
- **the header's controls**: the Play | Edit switch, then the mode's own, and
  Open… in both. The game's: its status, the log's path, Full control,
  Reload, "Save board as scenario", "Edit this board" and "Edit the scenario".
  The editor's: its own Undo, Play, Save and Copy as text. The tools' Undo
  answer, Savestate and menu sit in the game's header beside Reload, and the
  game's Undo reads as its own (§7.3);
- **`--edit [FILE]`** beside `--scenario FILE` and the tools' `--load FILE`.

---

## 7c. The why panel

> **Status:** design, 2026-10-05, decided at #220 the same day: the owner
> took every recommendation in §7c.1. It is `roadmap-v2.md` A6g's next
> item, "a 'why' panel fed by the trace sink", read against `7f8532a`
> (#219's merge). The build is SU-6 to SU-8 (§8). SU-6 built the layers'
> section and the panel, 2026-10-05, and SU-7 the question's section, the
> same day (§8's ✅ sections). What each build changed here is amended where
> it stands, and listed in the archive's entry for it.

**What it is for.** When the window shows something surprising, the tester has
two ways to find out why: read the engine's code, or write a trace page by hand
(`engineering-practices.md` §7). The panel answers in the window instead. A
right-click on a card, a permanent or a player opens it beside the board. It
says which rules and effects made the thing look the way it does.

- **"Why is Serra Angel a 1/1 with no abilities?"** On
  `mtgsim/scenarios/humility-opalescence.scenario`, the panel lists what each
  layer did to the Angel, in the order the layers applied it. As printed, it
  is a 4/4 with flying and vigilance. In layer 6, Humility (#22), "All
  creatures lose all abilities and have base power and toughness 1/1.", takes
  flying and vigilance. In layer 7b, the same ability takes power and
  toughness from 4/4 to 1/1. Below that, it lists what reached the Angel's
  zone and did not apply to it: Opalescence (#21) at layers 4 and 7b, which
  affects "each other non-Aura enchantment", and the Angel is not one. Each
  line carries its rule (CR 613.1f, 613.4b), and each card a line names links
  to that card's own why.
- **"Why can't I cast Grizzly Bears?"** On the review board `main.scenario`
  with Everywhere edited to a Mountain, the mana Player 0 can make, one red,
  does not cover {1}{G} (CR 601.2g–h). With Everywhere itself the Bears are
  offered: the enumeration counts its five mana abilities as five sources,
  `codebase-state.md` item 162, whose design is the PR after SU-7. On the opponent's turn it would say instead that a creature
  spell is cast only in its controller's main phase with the stack empty (CR
  117.1a, 302.1). At a declare-blockers question it would say "Grizzly Bears
  can't block Serra Angel: the Angel has flying, and the Bears have neither
  flying nor reach (CR 702.9b)".
- **"Why didn't Grizzly Bears die?"** A right-click on a log line asks why of
  that event. The panel shows the event's batch as the engine decided it: the
  damage proposed, each CR 616.1 iteration with its candidates (a prevention
  shield, say), the one applied and what it rewrote the event to, and what
  was performed. When a "can't" stopped the event, it names that too.
- **"Why didn't Soul Warden trigger?"** The same right-click, on "Grizzly Bears
  enters the battlefield", lists every triggered ability the engine asked
  about that event. Each one either matched or was refused, and the panel says
  by what: its condition, its intervening "if" (CR 603.4), or a once-each-turn
  limit. For a match, it says whether the trigger went on the stack or was
  removed because it had no legal target (CR 603.3d).

**The dev GUI's words**, as `plans/devgui-map.md` uses them:
- The **window** plays every **seat**. A seat is a player's chair: the
  `DecisionProvider` the engine asks for that player's choices.
- A **question** is a prompt, a choice the engine is waiting on. The **open
  question** is the one on screen.
- The **snapshot** is the board, copied as plain data at each question.
- The **view model** turns the snapshot into what the window draws.
- The **bridge** carries each question from the engine's thread to the window,
  and the answer back.
- A **replay** builds the game again from its start and plays the recorded
  answers into it. Undo answer works this way (§7.1).
- The **sink** (`state::trace`) is the trace sink: an observer that writes one
  JSON record per line at the engine's emit points, and is off unless a sink
  is attached.

And four engine words the design leans on:
- A **frame** is an object's characteristics as the layers computed them.
- The **layer memo** keeps each object's last frame until something a layer
  walk reads changes. A walk runs only when the memo has no frame to serve,
  which is a **memo miss**.
- An **application** is `board.rs`'s word for one thing a layer applies: one
  effect's rows in that layer, one CDA, one counter, or what an object entered
  as.
- The **key order** is the order a layer sorts its applications in before any
  dependency: CDAs first, then by timestamp (CR 613.3, 613.7). CR 613.8 lets an
  application wait out of it.

**Where it sits.** In two places, and that is the design's first fact.
- A question about **now** is a read of the state at the open question, as the
  snapshot is. Characteristics and options are this kind. Nothing on the
  event path changes.
- A question about **then** is answered from the sink's records. Events and
  triggers are this kind. The emit points write the records along the event
  path as it happens: the batch's proposals, each CR 616.1 iteration, the
  performed events, the dispatch's matcher, and placement.

```
now:   the open question ── its seat ── the engine explains X ─────────────────▶ the panel
then:  a replay of the window's line, sink on: batch · CR 616.1 · perform · dispatch · placement
                                               └── the records ──▶ the engine words them ──▶ the panel
```

**What the tree has** (read 2026-10-05 at `7f8532a`):
- **The sink writes 12 kinds of record from six emit points**
  (`state/trace.rs`, `engine/trace_records.rs`):
  - `batch`: the proposals, and their CR 616.1 subject groups;
  - `pipeline`: one CR 616.1 iteration, with the candidates, the chooser, the
    choice, the rewrite and each member's result;
  - `batch_end`: what each proposal became, or `null` if it was dropped;
  - `event`: the performed event, in `ui::display::format_event`'s words;
  - `trigger`: one matcher decision. It names the record asked about and the
    ability, and says whether it matched or which predicate refused it;
  - `pending`: placement, or its refusal (CR 800.4d's departed controller, CR
    603.3d's no legal choice);
  - `layer_walk`: one top-level walk, giving the answer's name, types, power,
    toughness and controller;
  - `decision` and `priority_rejected`: the prompt, and an action it refused;
  - and `game`, `object` and `fork`.

  **Two of the four kinds of why are in it**: the event's batch and the
  dispatch's matcher. **Two are not.** A `layer_walk` records the answer but
  not the applications that produced it. It is also written only on a memo
  miss, so the frame on screen may have no record at all. And nothing records
  why an option is *not* offered, since the enumeration drops a candidate
  without a word.
- **The layer pass already has a recorder, for one test.**
  `compute_board_traced` (`engine/layers/board.rs:1589`) hands
  `resolve_order_within_layer` (`:1550`) a list. For one layer, it fills that
  list with a `TraceStep` (`:1294`): each application's source, and the
  members it reached. `layer_4_order` (`:1810`) is its only caller.
  `perform` (`:1301`), through `row_affected` (`:1198`), already decides three
  things for each row:
  - whether its ability is gone (CR 604.2, `Affected::Gone`);
  - whether CR 613.6 locked its set (`Locked`);
  - or else, which members its filter matches now (`Fresh`).
- **The options are enumerated by checks that keep no reason.** Two of them
  have a twin that does.
  - `candidate_priority_actions` (`oracle/legality.rs:137`) offers
    `playable_lands` (`:38`), `castable_spells` (`oracle/mana_helpers.rs:135`)
    and `activatable_abilities` (`:333`). Each is a chain of `continue`s.
  - `passes_timing_check` (`mana_helpers.rs:302`) "mirrors"
    `check_cast_legality` (`engine/put_on_stack.rs:654`), which returns its
    reason as a `String`.
  - Combat already has the shape this design wants. `can_block`
    (`engine/combat/validation.rs:280`) returns a `CombatError`: the
    pre-filter keeps the `Ok`s and the validator reports the `Err`. Then
    `ui::display::combat_error` (`ui/display.rs:501`) words each error with
    its rule.
  - The attack side has its checks twice. `legal_attackers`
    (`oracle/legality.rs:82`) filters silently, and `validate_attackers`
    (`validation.rs:151`) returns the `CombatError`.
- **The window's seat can carry a request while a question is open.**
  `GuiSeat::ask` (`devgui/src/bridge.rs:334`) waits on the window's replies.
  It already carries one out in place without closing the question: "Stop
  yielding" (`:347`).
- **The window's line can be replayed to the open question.** The save holds
  every answer, and `Save::line_to` (`devgui/src/save.rs:193`) gives the line
  to any place. A `Replay` with no seats behind it stops the run with
  `Stop::LogSpent` at the next question (§7.2's decision 1). That leaves the
  state as that question showed it.

**Breadth, counted.** Every kind serves cards. The counts say how many, and
which rules.

The registry, from a probe that classified each card's definition (2026-10-05):

| A why about … | Registered (181) | Of them in the performance pool (101) | What puts a card there |
|---|---:|---:|---|
| options: cast, play, activate, attack, block, target | 181 | 101 | every card is cast or played; 77 are creatures, 36 choose targets or objects, 15 have activated abilities and 24 mana abilities |
| characteristics, through the layers | 47 | 33 | 23 static abilities with layer rows, 26 resolutions that make a continuous effect, 11 counters, 5 copies, 5 CDAs, 2 attach |
| events changed: replacement, prevention, "can't" | 66 | 18 | 46 static replacements, 16 made by a resolution, 4 "can't" |
| triggers | 11 | 7 | a triggered ability |

Magic, from Scryfall (2026-10-05, `total_cards` with ` game:paper -is:funny`
appended to each query):

| | Cards | Query |
|---|---:|---|
| every card | 32,381 | `-t:token` |
| a triggered ability | 14,149 | `(o:"when " or o:"whenever " or o:"at the beginning of")` |
| a target | 12,212 | `o:"target"` |
| a power or toughness change | 5,233 | `(o:"gets +" or o:"get +" or o:"gets -" or o:"get -" or o:"base power")` |
| a "can't" | 1,830 | `o:"can't"` |
| a replacement | 554 | `o:/would.*instead/` |
| a prevention | 522 | `o:"prevent"` |

The CR (`tmnt.txt`), counting numbered rules and subrules per section:
- **options:** 601 (28), 602 (21), 117 (22), 305 (13), 508 (40), 509 (25),
  115 (28), 732 (4);
- **characteristics:** 613 (43), 604 (10), 611 (15), 122 (21), 707 (34);
- **events:** 614 (39), 615 (17), 616 (11), 704 (41), 608 (26);
- **triggers:** 603 (49), and 113.6's functioning zones.

So options touch every card, and characteristics touch the subtlest rules:
CR 613.8's dependencies, CR 613.6's locked sets and CR 604.2's existence.
Triggers are the largest share of Magic, 44% of it, and TR-3 to TR-7 build
them next; `roadmap-v2.md` A6g's row calls those phases "where visual testing
pays most". Events changed are 37% of the registry today, and under a tenth of
Magic.

**The rules pass** (`engineering-practices.md` §8). For each kind: the rule
that owns it, the rule that watches it, and the CR's own words for what the
panel reports.
- **Options.** CR 601.2 owns casting. **CR 601.3** watches it: "A player can
  begin to cast a spell only if a rule or effect allows that player to cast it
  and no rule or effect prohibits that player from casting it". **CR 732.1**
  catches what 601.3 lets through: an action that is begun and cannot be
  completed is reversed. So a missing option has two tiers, and the panel must
  say which.
  - **Never offered.** The enumeration's checks decide this tier: CR 601.3's
    "can begin", 117.1a's timing, 305.2's land drop, and 508.1a and 509.1a's
    attackers and blockers.
  - **Offered, then reversed** (CR 732.1). A cost not paid in CR 601.2g's
    window is the usual case. The sink records it as `priority_rejected`, and
    the re-asked question already says it in `ui::display::rejection`'s words.

  The enumeration over-approximates on purpose (`oracle/legality.rs`'s
  header). An option it offers may still be reversed, but one it drops must
  be impossible. So a "why not offered" reason is always a static one.
- **Characteristics.** CR 613 owns them. **CR 604.2** and **611.3b** watch an
  effect's existence: a static ability's effect exists while its source has
  the ability. **CR 613.6** fixes the set an effect applies to where it
  starts, and **CR 613.8** reorders a layer by dependency. Each of these is a
  reason an effect did or did not reach an object, and the pass already
  decides each.
- **Events.** CR 614–616 own how an event is modified. **CR 614.17** and
  **101.2** watch them: a "can't" is checked ahead of every replacement and
  wins. So "why didn't it die?" is as often a "can't" (indestructible, CR
  702.12b) as a replacement. **CR 704.3** watches every state-based death, and
  the cause names its rule (`ZoneChangeCause::DestroyedBySba` is 704.5g,
  `ZeroToughness` is 704.5f). **CR 608.2b** is an event that never happens: a
  spell whose every target is illegal does not resolve.
- **Triggers.** CR 603 owns them. **CR 113.6** watches which abilities function
  in which zone, and an ability that does not function there is never asked.
  **CR 603.10** looks back in time for a leaves-the-battlefield trigger. **CR
  603.3d** removes a trigger that has no legal choice as it is put on the
  stack.

**Measured.** A throwaway probe on the owner's machine, 2026-10-05. The random
agent played each board to its end, with the decision log recorded as the dev
GUI records it. Then the game was replayed from its start to its last question. The
debug rows use the dev GUI's debug engine: opt-level 1 with its debug checks on,
as `devgui/Cargo.toml` builds it. There were ten games a board. The release
sitting ran straight after a build, so its absolute times may read high.

| Board | Answer lines a game | A live game: sink off → on | Trace a game | Replay to the last question, audits paused: sink off → on |
|---|---:|---|---:|---|
| `large.scenario` (188 objects, 2 seats), debug | 126–375 | 2.2–6.2 s → 2.1–6.5 s (−5% to +8%) | 252–680 KB | 9–27 ms → 13–34 ms |
| the same, release | | 4.5–11.9 ms → 6.2–15.7 ms (+6% to +51%) | | 4.2–10.2 ms → 5.5–15.2 ms |
| two seats, 60 cards, debug | 379–1,602 | 0.03–3.8 s → 0.04–3.7 s (−3% to +12%) | 0.5–2.5 MB | 3–40 ms → 7–62 ms |
| the same, release | | 1.6–18 ms → 4.5–36 ms (+78% to +181%) | | 1.4–17 ms → 4.8–35 ms |
| four seats, 100 cards, 40 life, debug | 1,968–3,465 | 1.8–11.4 s → 1.7–11.8 s (−7% to +6%) | 1.9–3.2 MB | 33–81 ms → 52–114 ms |
| the same, release | | 16–34 ms → 30–54 ms (+48% to +98%) | | 15–34 ms → 29–56 ms |

- **In the debug window the sink costs nothing measurable.** The layer memo's
  audit is about 98% of a debug game (§7.1). The sink's own work can be read
  off the paused replays, where the audit is off: 2–7 ms a game on the large
  board, 5–22 ms at two seats and 19–36 ms at four. That is under 1% of a live
  debug game on the large board and at four seats, and 0.6–4% at two seats,
  all inside the run-to-run noise. The one exception was the shortest game, at
  33 ms, where it was 15%. In release the sink adds 6% to 181% to the engine's
  own work, which is still 2–11 µs an answer.
- **A replay to the open question with the sink on takes about as long as a
  click.** In debug it is 13–34 ms on the large board, and at most 114 ms late
  in a four-seat game. A question halfway through a game replays in a third to
  a half of that.
- **A trace is 4–8 records an answer line**, and 0.25–3.2 MB a game.
- **One layer pass on the large board costs 29 µs in release and 46 µs in
  debug**, with the memo cold. The priority question's candidates cost 39–40 µs
  in release. In debug they cost 39 ms with the memo's audit, and 86 µs
  without it.

### 7c.1 The decisions

#### Decision 1 — what a why is about

**The problem.** The four kinds differ in when their facts exist, and in what
the engine has today. Characteristics and options are questions about the
open question's board. Events and triggers are about a moment before it. The
sink holds the facts for events and triggers, and none for characteristics or
options.

| | **A. Characteristics only** | **B. Now: characteristics and options** | **C. Then: events and triggers** (the row's "fed by the trace sink") | **D. All four, in three PRs** |
|---|---|---|---|---|
| Answers | "why is it a 1/1" | that, and "why can't I cast, attack or block with it" | "why did this happen this way", "why didn't it trigger" | all four of the brief's examples |
| The engine adds | the pass's recorder, and its words | that, and the enumeration's reasons | a reader of the sink's lines, and one field on `pipeline` | all three |
| Code, then tests | ~460–690, ~200–300 | ~750–1,140, ~350–520 | ~480–745, ~250–330 | ~1,050–1,615, ~500–750 |

The sizes include the panel itself, ~180–270 lines of each option's code,
which D builds once (§8 itemizes them). They are design estimates, and SU-1
to SU-5 ran 1.0–2.5× their estimates on code.

**Decided: D** (the owner, 2026-10-05). Build it as SU-6 (characteristics, and the panel), SU-7
(options) and SU-8 (events and triggers). Each answers a surprise the brief
names, and the measured costs make each cheap at a click.
- **Characteristics and options are engine surfaces a second client wants
  too.** v1's GUI (`backlog.md` §2.38) will want "why can't I cast this" as
  much as the dev GUI does. That is `engineering-practices.md` §10's test for
  an engine surface.
- **Characteristics come first.** The layer system is the subsystem the
  hand-written trace pages explain most (item 7, LI-1, CV-1, CV-2b, RG). The
  hover already shows its answer but not how it was reached, and its recorder
  exists.
- **Triggers come last of the three, but only by dependency.** The trace half
  needs the replay and the reader, and all three PRs land before TR-3.

**The trimmed option is A or C alone**, at the row's ~400–700, with the other
kinds filed as items with their slots.

#### Decision 2 — where the facts come from

**The problem.** The sink records what was consulted, in what order, and with
what answer (`engineering-practices.md` §7.1), and it records only where an
emit point sits. For each kind there are three places the facts could come
from: the sink as it stands, new records, or an engine query asked at the open
question.

| Kind | The sink as it stands | New records | An engine query |
|---|---|---|---|
| Characteristics | `layer_walk`: the answer, with no applications, and nothing on a memo hit | a record for each application in each layer, in every traced pass: ~5–50 a pass, and still nothing for the frame a memo hit serves | **the pass run again for one object, with a recorder**: ~0.05 ms, and the memo is untouched |
| Options | `decision` lists what was offered, and nothing says what was not | a record for each dropped candidate, at every traced priority question | **the enumeration's own checks, each naming its reason** |
| Events | **`batch`, `pipeline`, `batch_end`, `event`**: the proposals, each CR 616.1 iteration, and what was performed | **one field**: `pipeline` names a member a "can't" blocked (CR 614.17), and the "can't"'s source | deciding a past event again at the open question reads today's board, not that one's |
| Triggers | **`trigger`, `pending`**: each ability asked about a record, matched or refused by what, and its placement | — | the same as for events |

**Decided: an engine query for the two kinds about now, and the sink for
the two about then** (the owner, 2026-10-05).
- A record for each application would put work in every traced pass for a
  question asked about one object, and it would still miss the frame a memo
  hit serves.
- A query is the pass itself, with a recorder watching one object. It has the
  dispatch audit's shape from TR-1b, "the same code with its shortcuts off";
  here the shortcut is the memo.
- Past events need the sink. Deciding one again at the open question would
  read the board as it is now.

What the engine adds, each under an A/B predicted `IDENTICAL` on every counter:
1. **`layers::explain(game, id)`** (SU-6). It is `run_pass` with a recorder for
   one object, in place of `compute_board_traced`'s list for one layer, and
   `compute_non_member` with the same for a card no pass holds. It records
   each application in each layer, in the order applied:
   - what it is: an effect's rows, a CDA, a counter, or what the object
     entered as;
   - its source and its timestamp;
   - what it did to the watched object. It applied, with the frame before and
     after. Or it did not, because its filter or its zones exclude the object.
     Or it is gone (CR 604.2), or locked out (CR 613.6);
   - and whether it waited (CR 613.8): an application applied ahead of one
     earlier in the key order says so.

   Hypotheticals (`depends_on`'s journaled `perform`) record nothing. A debug
   assertion holds the explanation's result equal to the frame the memo
   serves.
2. **The enumeration's reasons** (SU-7). Each check in `playable_lands`,
   `castable_spells` and `activatable_abilities` becomes a function that
   returns why it refuses. The enumeration keeps its `Ok`s, and the why
   reports its reasons, as `can_block` already does.
   - `legal_attackers` takes `validate_attackers`' per-creature check the same
     way, so the attack side has one road, as the block side does.
   - `check_cast_legality`'s `String`s become the same typed reasons, so
     enumeration and enforcement share one check. That is `codebase-state.md`
     item 188's class: two roads to one answer.
   - `ui::display` words each reason with its rule, beside `combat_error`.

   As SU-7 built it: `play_land` and `activate_ability` ask the same checks
   too, the pairs item 188 names beside `check_cast_legality`, and a cost's
   resource check (`can_pay_costs`) returns `CannotPay`. Each family's reason
   is the first its check refuses, in the order the enumeration asked. The
   section answers at every question whether it offers the object or the
   player, and why not only at the three questions these checks build (the
   priority question and the two declarations). A target's reasons are RS-2's,
   which rewrites target legality (the owner, 2026-10-05).
3. **A reader of the sink's lines** (SU-8), in `state::trace` beside the
   writer, as its inverse. It gives a record's kind and fields, for the panel
   to select by. So the format has one owner, and a round-trip test against
   `Record` holds the two halves equal.
4. **`pipeline` names what a "can't" blocked** (SU-8). `is_prohibited`
   (`engine/restriction/predicate.rs:60`) gains a form that returns the
   restriction's source, and the iteration record writes it for each member.

**Every why comes back as one value, `ui::why::Why`**: a title, and sections
of lines, each line with its words, its rule, and the objects it names. The
engine words it, from the explanation, the reasons or the records. So the
window draws a list and never a case per mechanic, as A6g's row requires:
"the GUI draws only the engine's generic surfaces".

#### Decision 3 — when the facts are gathered

**The problem.** A question about now needs the state at the open question. A
question about then needs the sink's records up to it. Today the window's game
runs with no sink attached.

| | **A. Live: a sink on every game the window plays** | **B. On demand: a replay to the open question, with the sink on** | **C. Now at the seat, then by a replay** |
|---|---|---|---|
| Questions about now | asked at the seat, while the question is open | answered on the replayed game, stopped at the open question | **asked at the seat**, which answers while the question stays open, as it carries out "Stop yielding" |
| Questions about then | read from the game's own records, kept in memory | read from the replay's records | **read from a replay's records**, built again at the click |
| Cost, debug | within the run-to-run noise during play, but 0.25–3.2 MB kept a game and the records parsed at each click | a replay a click: 13–34 ms on the large board, at most 114 ms late in a four-seat game | at most 0.1 ms a question about now, and a replay a question about then |
| Cost, release | +6% to +181% of the engine's own work, 2–11 µs an answer | 5–56 ms a click | at most 0.1 ms a question about now, and 5–56 ms a question about then |
| Upkeep | every game the window plays carries an observer and its memory, and each Undo or savestate rebuilds the record from its replay | Undo's rebuild path, without the hand-over (where a replay gives the game to the seats): one more thread, over the line the save already has | both paths, each the cheaper one for its kind of question |

**Decided: C** (the owner, 2026-10-05).
- **A question about now is answered where the state already is**, at the
  cost of a pass.
- **A question about then pays for a replay at the click.** That is the
  brief's own suggestion, and the measurements put it at about a click's wait.
  Nothing about live play changes, and the window keeps no trace.
- **The replay is Undo answer's rebuild (§7.3), with three differences**: the
  sink is attached, the line is the window's current one, and no seat stands
  behind it, so it stops at the open question (`Stop::LogSpent`). It runs on a
  thread of its own while the game's own thread keeps waiting at the question.
  Its audits are paused, as an undo's are, since this build checked every
  answer on the line as it was given.

**Asking at the seat is safe, because the explanation only reads.**
- Its reads can fill the layer memo at the current epoch, with the frames a
  later read would compute anyway. The memo's audit holds the two equal.
- Its reads also count in the diagnostics, which the dev GUI shows nowhere.
- The test plays a game with every object's why asked at every question, and
  the same game with none asked. They must have the same events and the same
  decision log.
- The memo's audit is not paused for it in debug. SU-6 measured a why at a
  question at 1.9 ms on the large board in debug, audit and all, beside the
  snapshot's own 33 ms there (`prompt_cost`, 2026-10-05), so a pause would
  save nothing a tester sees.

#### Decision 4 — what the window shows, and how a tester gets there

**Getting there.**
- A right-click on anything the board draws as an object or a player asks why
  of it: a permanent, a card in any zone, a stack object, or a player's line.
  SU-6 built it for objects, since the layers say nothing about a player. A
  player's line comes with SU-7, whose section is the first with something to
  say about one: whether the player is among the open question's options.
- A left click keeps its meaning, so a why never answers the question by
  accident. So the settling beat, the 0.3 s after a question arrives when the
  window drops clicks (`codebase-state.md` item 201), has no need to drop a
  why.
- In SU-8, a right-click on a log line asks why of that event.
- Each object named in the panel's lines links to its own why, and a Back
  button retraces the path taken.

**The panel follows its object.** While the panel is open, each new question's
message carries the why of the object it shows. The seat answers it beside the
snapshot, for about 0.1 ms a question. So the panel is never stale: following a
creature through combat shows each change as it happens. A question about then
is answered once, at its click, and marked with the question it was asked at.

**What it shows**, in sections:
1. **At this question** (SU-7). It says whether the object is among the open
   question's options, and as which. If not, it says why and on which tier:
   never offered, or offered and then reversed (CR 732.1).
2. **What the layers did** (SU-6). The printed card comes first. Then each
   application that reached the object, in the order applied, with its layer,
   source, timestamp, the ability's own text, and what it changed ("Power and
   toughness from 4/4 to 1/1": SU-6 wrote "from … to …", since the window's
   font has no arrow). Then each application that reached the object's zone
   but not the object, with the reason. Last comes the result, which matches
   the hover.
3. **What happened** (SU-8). For an event, it shows the event's batch as the
   trace viewer draws it (`plans/traces/viewer.html`), and every triggered
   ability asked about it, matched or refused and by what.

**Where it goes.** A panel on the window's left, beside the board, opened by
the right-click and closed by its ×. The board stays in view, and the log and
the stack keep the right side. The other choice was a floating window, which
can cover the very cards it explains. **Decided as recommended** (the owner,
2026-10-05). Layout is the client's to decide (`engineering-practices.md`
§10), and nothing in the engine depends on it.

#### Decision 5 — where this design lives

| | Home | Fit |
|---|---|---|
| **a** | **this file, §7c**, beside the tools | the panel is built from the tools' parts (§7.1–§7.3): the save's line, the replay and its stop, the seat's loop. `devgui-map.md` §9 already sends a reader to this file for the tools, and the panel's PRs take this file's codes, as SU-4 and SU-5 did |
| b | `engineering-practices.md` §7, beside the trace pages | the panel is tier 2's reader (§7.1), but that file is process, and a design with decisions and phases does not belong in it |
| c | a new doc on `CLAUDE.md`'s architecture row | the row holds one doc per CR subsystem. This would be its first doc that is not one, which §9 declined for the loader |
| d | each engine surface in its subsystem's doc, and the panel here | the layer recorder would go in `layers-architecture.md`, and the reasons beside `cost-architecture.md` §3.6's "enumeration and enforcement must agree". That is five places to read one design |

**Decided: a** (the owner, 2026-10-05). Each other doc gets one line where
an invariant lives:
- at SU-6's build, `layers-architecture.md` §9 gains that the recorder is
  the pass, with no second walk. The design named §13b, LI's landed plan;
  the line went beside LI-2's loop in §9, which is what the recorder watches;
- at SU-7's, `cost-architecture.md` §3.6 gains that the reasons are one check,
  shared by the enumeration and the enforcement;
- at SU-8's, `engineering-practices.md` §7.1 gains a pointer here, as tier 2's
  reader in the window.

`CLAUDE.md` does not change.

### 7c.2 The surfaces, sketched

Each is a shape for the build to refine, not a type list to copy.

```rust
// engine::layers (SU-6): the pass, recording for one object.
pub fn explain(game: &GameState, id: ObjectId) -> Option<LayerExplanation>;
// One step: the application, its layer, timestamp and source, and what it did to `id`:
// Applied { before, after }, NotMatched, NotInItsZones, Gone (CR 604.2) or LockedOut (CR 613.6).

// oracle (SU-7): one check for the enumeration and the enforcement. As built,
// `Ok` holds the mana sources it would tap, which `castable_spells` returns.
pub fn can_cast(game: &GameState, player: PlayerId, card: ObjectId) -> Result<Vec<ManaSource>, CannotCast>;
let castable = hand.iter().filter(|&&card| can_cast(game, player, card).is_ok());

// ui::why: what every client draws.
pub fn why(game: &GameState, about: WhyAbout, at: Option<&OpenQuestion>) -> Why;
pub struct WhyLine { pub text: String, pub rule: Option<&'static str>, pub names: Vec<ObjectId> }
```

**The bridge** (SU-6). The window sends `Reply::Why(Some(about))`, and the seat
stores what the panel shows. It answers at once with `ToWindow::Why`, and while
the panel stays open it puts `why: Option<Box<Why>>` in each `ToWindow::Prompt`.
`Reply::Why(None)` closes the panel. In SU-8, `Session` starts the replay
thread for a question about then, and its answer arrives as `ToWindow::Why`
too. A newer replay supersedes an older one, as an undo's does.

**When no question is open**, the right-click is off, with a line saying why,
as Savestate is. That covers the engine playing between questions, a replay on
its way, and a game that has ended. After SU-8, a game that has ended answers
through the replay of its whole line. As SU-6 built it, an object's hover
says so, and an open panel keeps its answer at the last question, its links
off, with a line saying why.

**What each PR's review runs**, in Magic terms:
- **SU-6.** Load `humility-opalescence.scenario`, then right-click Serra
  Angel. The panel says 1/1 with no abilities: layer 6 takes flying and
  vigilance, layer 7b sets base 1/1, and Opalescence does not apply to it
  because it is not an enchantment. Right-click Humility: Opalescence makes it
  a creature at layer 4, its own ability strips it at layer 6, and at 7b
  Opalescence's 4/4 comes before Humility's 1/1 by timestamp (CR 613.7). Swap
  the two lines in the editor, Play, and ask again.
- **SU-7.** On `main.scenario`, edit Everywhere to a Mountain, Play, and
  right-click Grizzly Bears in the hand: the mana is short. (A Mountain keeps
  Lightning Bolt castable, so the window stops at the priority question.) (With Everywhere
  the Bears are offered: item 162, amended 2026-10-05.) Pass to Player 1's
  turn and ask again: now it is the timing. On `blocks.scenario`, edit in a
  Serra Angel attacking and right-click Wall of Stone at the block: it cannot
  block the flyer. Right-click a player's line: whether the question offers
  that player.
- **SU-8.** On `bolt-into-giant-growth.scenario`, let the stack resolve, then
  right-click the log line where Lightning Bolt deals its damage. The panel
  lists that event's batch, its CR 616.1 iterations and the triggers asked
  about it. Then right-click the Bears: Giant Growth at layer 7c and the
  Thaumaturgist's switch at 7d say why the damage was or was not lethal.

**Random clicks** (`engineering-practices.md` §10.3) gain right-clicks on the
board, and clicks on the panel's links. A why click must change what the panel
shows or refresh it, and the engine must answer every why without panicking.
In debug that runs the explanation's equality assertion against the memo across
whole random games, which is the recorder's widest test.

### 7c.3 Found while designing

1. **A debug build's trace carries records a release build's does not.**
   `check_order_invariance` (`engine/replacement/pipeline.rs:1597`) is a
   debug-only self-check, and its `gather` builds a CR 614.12 look-ahead. That
   look-ahead writes a `layer_walk` record (`membership: "entering"`) and
   counts its walks. Release returns before any of it. The probe's two-seat
   seed 12353 wrote 412 walks in release and 417 in debug. The diff is those
   five records alone, and every other game matched byte for byte. The sink's
   own module doc says why that matters: a debug trace that differs from a
   release one breaks regenerating a page's spine from a test. Filed as
   `codebase-state.md` item 210, slotted to SU-8, the first PR that reads a
   debug engine's trace in the window.

---

## 8. The build, sized

**The editor's advanced settings** (§7b.2's decision 2, B; A6g's row, after
the "why" panel), sized at SU-3's design, row by row: player counters
~45–65, lands played ~20–30, left the game ~20–30, commander damage ~50–80,
history ~110–160, `this turn:` ~90–130, `counters:` lines ~60–90, setup
actions ~210–310: ~600–900 in all, the typeable field that reads any line of
the grammar first at ~40–60. SU-3's own code came in at 1.5–2.1 times its
sizing, nearly all of it in the dev GUI, which may run looser than the engine
(the owner, 2026-10-03; §8's ✅ section).

**The why panel** (§7c, decided 2026-10-05) is three PRs, in the order
§7c.1's decision 1 set. Each size gives code, then tests, at
this design's resolution. SU-1 to SU-5 ran 1.0–2.5× their code estimates.

### SU-8 — what happened, from the trace

**The engine, ~190–295.**
- the reader of the sink's lines, ~90–130;
- `is_prohibited`'s form that names the restriction, and the `pipeline`
  record's field, ~25–45;
- the why's sections for an event and for an object's triggers, ~70–110;
- item 210's fix, ~5–10;
- **every question kind's line, ~80–140** (`codebase-state.md` item 212's
  census, the owner, 2026-10-05). `ui::why::refusals` matches every
  `ChoiceKind` without a wildcard. Each kind says what it ranges over, so a
  why asked about anything else says the question does not range over it. Two
  kinds are answered from the trace: `ChooseReplacementEffect`'s candidates,
  from the `pipeline` record, and `OrderTriggers`', from the `trigger`
  records. Three get a reason from their own filter:
  `ChooseEnteringController` (CR 800.4a), `ChooseAuxiliaryZoneChange` (CR
  614.13a and 101.2) and `ChooseCopySource` (the copy effect's filter). The
  other filtered kinds' reasons land with their owners: `SelectRecipients`
  with RS-2; `ManaAbilityWindow`, `ChooseXValue` and `GenericManaAllocation`
  with item 162's build; `ChooseSacrificeForCost` with
  `cost-architecture.md` CP-2. `plans/references/cast-census.md` §9 has the
  table, read off the enum and asserted against it. `refusals`' doc comment
  still cites item 212, now closed, and this PR rewrites it.

**The dev GUI, ~110–180.**
- the replay thread for a question about then, with its stop and its
  supersession, ~70–110;
- each log line's event number in the snapshot, and the right-click on it,
  ~30–50;
- the sections drawn, ~10–20.

**Tests, ~150–230.**
- the reader's round trip against every record kind;
- an event's batch read back on `bolt-into-giant-growth.scenario`;
- a "can't" named: an indestructible creature with lethal damage;
- a trigger refused by its intervening "if";
- in the headless tests, the replay stopping at the open question;
- every `ChoiceKind` answers a why without panicking, and a departed
  player is named as the reason at `ChooseEnteringController`, ~30–50.

**A/B.** No record is written in a fuzz game unless `--trace` asks for it.
Predicted `IDENTICAL` on every counter, and instructions within ±0.3%. With
`--trace`, the `pipeline` record's new field is the only change in its text.

**In all:** ~1,050–1,615 lines of code and ~500–750 of tests, as sized at
#220. SU-6 landed at 1,062 and 622, and SU-7 at 1,042 and 606. SU-8, as
sized above, is ~380–615 more and ~180–280 of tests; item 212's census added
the question kinds' lines (~80–140 and ~30–50).

### SU-7 — why an option is not offered — ✅ landed 2026-10-05

**What shipped.** §7c's second PR (#222). Each option the window offers has one
check that the enumeration and the enforcement both ask, and it returns why
it refuses: `can_begin_to_cast` and `can_cast` (`CannotCast`), `can_play_land`
(`CannotPlayLand`), `can_activate_its_abilities`, `can_begin_to_activate` and
`can_activate` (`CannotActivate`), `can_attack` (`CombatError`, as `can_block`
already did), with `SorceryTiming` for CR 307.1's three questions and
`CannotPay` for a cost (CR 118.3). `ui::display` words each reason with its
rule. The why's first section, **At this question**, says whether the open
question offers the object or the player, and as what; if not, why, on two
tiers: never offered, from the checks, or offered and then reversed
(CR 732.1). In the dev GUI a player's line asks too.

**What moved on the way in.** `play_land` and `activate_ability` share the
checks too, and the cast now refuses a land (CR 305.9). A target's reasons
are RS-2's. The click script edits Everywhere to a Mountain, since one
Everywhere counts as five mana sources (`codebase-state.md` item 162, whose
design doc is the next PR), and item 211 records that the priority question
never offers a mana ability. It landed at +1,042 code and +606 tests,
against 310–485 and 150–220 sized.

**Measured.** `close_out.py` with three arms against #221's merge: the behavior arm
(every commit but the last code one) `IDENTICAL` on every row, gameplay and
cost, both pools, two seats and four; the shipped arm `IDENTICAL` on every
gameplay row, with `Memo hits` down 57–165 a run, the one query its last
commit drops (the owner's split). Instructions per decision −0.55%, against
−1.0% ± 0.3 predicted: the error `String`s no longer allocated saved the
1.09% priced, and the per-ability check, now a call where the old loop
skipped an ability inline, took back 0.56. `prompt_cost` on the large board:
a why at a question 38.7 µs median in release, a card in hand's 6.9 µs.

→ `plans/archive/setup-architecture-landed.md`, "SU-7" (the build as sized,
sized against built, and what the build changed in the design).

### SU-6 — the why panel, and what the layers did — ✅ landed 2026-10-05

**What shipped.** §7c's first PR (#221), as decided at #220. `layers::explain(game,
id)` (`engine/layers/explain.rs`) runs the walk `compute_characteristics`
runs for the object, by its pass membership, and hands it a `Recorder` where
the game's passes hand `None`. It records each application in each layer as
it applies: what it affected, and what it did to the object, applied with the
frame before and after or missed, by its set, by CR 604.2 or by CR 613.6.
Each says what it waited for (CR 613.8), and CR 306.5b's loyalty ability is
a step of its own. A debug assertion holds the answer to the walk's.
`ui::why::why(game, id)` words it: the printed card, each layer's changes in
the order applied, what reached the object's zone and missed it, and the
result, each line with its rule and a link for each object it names. In the
dev GUI a right-click on an object asks why at the open question, the seat
answers beside the snapshot, and the panel on the window's left follows its
object from question to question, with links, Back and ×, kept across
Undo and an unchanged board's Reload.

**What moved on the way in.** Each step keeps every object it affected, which
the panel names and LI-2's one-layer test hook now reads. `OwnApplication` names
which of a member's own applications it is. A change reads "from 4/4 to
1/1", since the window's font has no arrow. A player's line moved to SU-7,
and the memo's audit is not paused in debug (§7c.1's decisions 3 and 4,
amended where they stand). It landed at +1,062 code and +622 tests, against
460–690 and 200–300 sized.

**Measured.** `close_out.py` against #219's merge: each counter file
byte-identical outside `=== Timing ===`, both pools, two seats and four.
Instructions per decision +0.50%, past the ±0.3% predicted and inside §3.1's
2.5 points: about 0.10 in the recorder's checks on the game's path, about
0.40 in functions whose source this PR does not change. `prompt_cost` on the
large board: a why at a question 38 µs median in release, 1.9 ms in debug;
the panel's view 1.5 µs a repaint.

→ `plans/archive/setup-architecture-landed.md`, "SU-6" (the build as sized,
sized against built, and what the build changed in the design).

### SU-5 — the tools — ✅ landed 2026-10-04

**What shipped.** §7.1–§7.3's window half, as decided at #212 and #216.
The save, `<log>.save` beside each decision log (`devgui/src/save.rs`): a
journal in the engine's words and the session's, its answers numbered in
the order taken, so the engine reads them as one record, and the session's
three lines, where the window was asked, a savestate, a move. The record
(`bridge::Record`): one engine thread writes it at a time, a rebuild's
taking it over from the one before, whose replay stops at its next answer,
and a replay's answers held until it hands the game to the seats, where the
log is written again. Undo answer: a fresh game replayed to the window's
previous question, the memo's audits paused, off at the first question
with a line saying why, each press during a replay one question further
back. Savestate, named for its turn and step, and the menu of the
savestates and "Back to where I was". `--load FILE`, a save or a plain log,
replayed with the audits on and counted, play going on in a new pair beside
it, and a line that diverges shown and played on from the answer before.
Item 200, both halves.

**What moved on the way in.** A savestate is written at the click under the
record's lock rather than relayed to the engine's thread: that thread waits
at the open prompt, and the lock orders the lines. The game's buttons come
first after the Play | Edit switch, where nothing before them changes width,
so a double click on Undo answer cannot land on Reload. A loaded line that
diverges is built again and replayed to the answer before it, a stopped game
never continuing; a record that ended inside its setup actions replays as far
as they got. It landed at +1,162 code and +784 tests, all in the dev GUI,
against 670–980 and 300–440 sized.

**Measured.** No engine file changed, so no arms ran. `prompt_cost` in
release, two sittings interleaved with `main`'s: every reading's allocations
and bytes as `main`'s and its times within noise, and the header's tools
0.1–0.2 µs and 6 allocations a repaint. The dev GUI's CI test step 24.0–24.1 s
against `main`'s 23.2–23.4 s, the tools' eight tests 0.9 s of it.

→ `plans/archive/setup-architecture-landed.md`, "SU-5" (the build as sized,
sized against built, and what the build changed in the design).

### SU-4 — the replay — ✅ landed 2026-10-03

**What shipped.** §7.1–§7.3's engine half, as decided at #216.
`ChoiceKind::as_str`, the trace sink's spelling of a kind moved onto the
kind, replacing eight readers of `Debug` text. `ChoiceOption::as_logged`, an
option as what it is, and `position_of`, the one matcher the setup driver, the
cv2 test providers and the scripted provider's new `expect_choice` find an
option by. `ui::ask`'s checks as predicates the validators assert and the
replay shares. The stop: `Stop`, raised from inside a prompt;
`Game::until_stopped`, which catches it and nothing else; a stopped game
refused at every run entry; the setup driver's refusal raised as one, which
`fuzz_games` and the dev GUI report as the scenario's. The log's text in the
engine (`state::decision_log`): `decision log 1` and the engine's commit, a
`GameStart` written, read and built, an answer a line naming what it chose,
the outcome; `BuiltStart::play` and `replay`. `ui::replay`: each line checked
and its choice found by what it is, a forced line skipped or supplied where
two builds differ, then the seats or a stop, and a `ReplayControl` for another
thread. The memo's audits paused for a replay within a session and resumed at
the hand-over. The dev GUI writes the engine's text and plays its start
through `BuiltStart`.

**What moved on the way in.** `as_str` is the trace sink's match moved, not a
ninth copy; exact names exposed two stale test literals, one of which left a
test unable to fail, and a list of variants one short. A scenario's replay
answers the setup driver's lines from its record, with no driver. A start is
read only within a record, so SU-5's journal hands the reader its engine
lines. It landed at +1,259 code and +680 tests in the engine and +101 in the
dev GUI, against 835–1,220, 460–720 and ~35–50 sized: 1.0–1.5 times on code,
under the 1.5–2.1 the last phases ran at.

**Measured** (`fuzz-record.md`, the SU-4 block). Every gameplay and cost row
byte-identical to `main`'s on both pools at two seats and four, as predicted;
instructions per decision −0.39%, past the ±0.3% predicted on the cheap side,
from code placement: a probe arm restoring the validators' old duplicate check
read +0.03%. In debug the a6g replay test runs in 0.28 s with its replay's
audits paused, and the dev GUI's CI step reads 22.8 s.

→ `plans/archive/setup-architecture-landed.md`, "SU-4" (the build as sized,
sized against built, and what the build changed in the design).

### SU-3 — the board editor — ✅ landed 2026-10-03

**What shipped.** §7b as decided. `devgui/src/editor.rs`: an `Editor` over a
`Scenario`, undo a stack of boards, each edit made on a copy written and read
back, and `build`'s refusal marking the card, player word or setup action it
names. The board's own words have controls (decision 2's A); the other ten
rows are shown as text, each removable. Tags are given once a name is shared
and never changed, an Aura moves below its host, and a card leaving the
battlefield keeps only the words its new line has, the references that named
only it going with it. `search.rs` narrows any list of names. One window
(decision 1): Play | Edit, Play saving the board and starting it from its
file, "Edit this board", "Edit the scenario", `--edit [FILE]`. A folder per
board (decision 3, `boards.rs`), git-ignored, with Copy as text and the
header's Open… list. The engine gained `names_in_development`, `CardWord`'s
and `PlayerWord`'s Display, and public `turn_positions`, `position_word` and
`tag_letters`.

**What moved on the way in.** The battlefield is one zone in the editor's
order, numbered across the seats, so Humility can be put ahead of
Opalescence; a line of several cards moves one copy at a time; the editor's
random clicks found two clicks it offered that changed nothing (a move past
identical lines, a second copy of a tagged card), now not offered; a name
holding `#` reads back as another board with no parse error, so the read-back
compares the text written again; turning `commander` off drops the
commander damage that named only it. It landed at +2,164 code and +621 tests
against ~1,015–1,430 and ~495–765 sized, all but +77 of it in the dev GUI.

**Measured** (`fuzz-record.md`, the SU-3 block). Every counter file is
byte-identical to `main`'s on both pools at two seats and four, as
predicted, and instructions per decision +0.14%. On the 188-object board
the editor's view costs 19 µs a repaint and an edit 142 µs a click in
release, 104 µs and 278 µs in debug, inside §7b's 0.82 ms
(`engineering-practices.md` §10.4); the dev GUI's CI test step reads about
24 s, as before.

→ `plans/archive/setup-architecture-landed.md`, "SU-3" (the build as sized,
sized against built, and what the build changed in the design).

### SU-2 — setup actions — ✅ landed 2026-10-01

**What shipped.** The `then:` line in §5.1's table: `then: player p casts
<card>` or `… activates <card>`, each answer a segment of its own after a bar
(`targeting <card>`, `targeting player p`, `ability N`). `Scenario::build`
returns a `BuiltScenario`, the board and its `SetupActions`, each name
resolved through the loader's own table and what can be checked refused
there, naming the line. `SetupDriver`, a `DecisionProvider` over the seats'
own, plays the lines as §5.3 says; `fuzz_games --scenario` and the dev GUI
play through it, the GUI before its first prompt and again on Reload.
`mtgsim/scenarios/bolt-into-giant-growth.scenario` is §5.3's stack, and the
template casts a Bolt, so CI's determinism step plays a setup action under its
three hasher seeds. Item 198 rode along in its own commit.

**What moved on the way in.** `mode N` and `x N` are words that wait: nothing
asks for a mode, and no X spell is offered at priority (§5.1). `AutoPayer`
orders reductions and taps nothing, so the driver taps, by the random agent's
preference. A line no seat reaches is refused when its seat holds priority
without its action, since the driver stops every seat. Setup actions resolve
nothing, and a line that resolves the stack is open (§5.3). It landed at
+954 code and tests against ~400–650 sized.

**Measured** (`fuzz-record.md`, the SU-2 block). Both arms play every gameplay
and cost row byte-identically to `main` on both pools at two seats and four,
as predicted; instructions per decision +0.28% for SU-2 and +0.47% with item
198's fix, past the ±0.1 predicted, from where the compiler inlines rather
than from work (the fix's battlefield pass is 0.06%).

→ `plans/archive/setup-architecture-landed.md`, "SU-2" (the build as sized,
sized against built, and what the build changed in the design).

### SU-1 — the scenario loader — ✅ landed 2026-10-01

**What shipped.** The entry performer's state half, `make_permanent`, which
`place_on_battlefield` and the construction door `create_on_battlefield` share,
with the arrival turn a parameter of both it and `register_static_effects`.
`mtgsim::scenario`: the grammar as §5.1's one table (a module doc's copy moved here at
review), `Scenario::parse`,
`build` (§4.1's refusals, saying what to change), `write` (the four structs
destructured with no `..`, and its report) and `Display`. `Game::resume`;
`RandomStreams::from_seed`, with `fuzz_games`' salts; the registry's list of cards in
development; `CounterType::name`. `mtgsim/scenarios/`: a template naming every
word and four samples. `fuzz_games --scenario`, under CI's three hasher seeds.
The dev GUI's `--scenario`, Reload, "Save board as scenario", a refusal in the
window and the log's header, its review pictures drawn from three boards.

**What moved on the way in.** The design gained `arrived turn N` (CR 302.6
reads the controller's own most recent turn), lost the list after `blocking`
(one attacker per blocker, since a reference takes the rest of its line), and
lists a graveyard bottom first; the rest is in the archive. It landed at
+3,181 code and tests against ~1,720–2,340 sized; the owner kept it whole when
it crossed 2,500 at the writer.

**Measured** (`fuzz-record.md`, the SU-1 block). `main` and the last code
commit play every gameplay and cost row byte-identically on both pools at two
seats and four, as predicted; instructions per decision −0.23%. The round
trip compared 1,527 written boards at two seats and 47 at four with no
difference. Its skips order the next words: at two seats the mana pool,
copies, tokens and a resolution's rows; at four, "since your last turn" (§5.2).

→ `plans/archive/setup-architecture-landed.md`, "SU-1" (the build as sized,
sized against built, and what the build changed in the design).

---

## 9. Where this design lives

`CLAUDE.md`'s authority table has no row for tooling, and the file is at 197 of
200 lines.

| | Home | Cost to `CLAUDE.md` | Fit |
|---|---|---|---|
| **a** | **this file, as CR 103's doc**: the dealt start, the scenario and the save | one entry on the architecture row's existing line: 0 lines | the row is "one per CR subsystem", and CR 103 is one, with three open items (194, 119, `backlog.md` §2.32) and no doc |
| b | `scenario-architecture.md`, the loader alone | the same entry, 0 lines | the row's first doc that is not a CR subsystem |
| c | `roadmap-v2.md` A6g's row and the module's doc | none | the row is a route narrative of ~1,000 words already, and a design does not belong in one |

**Recommendation: a.** The row's new entry: `setup` (103, the scenario loader,
the save, `SU-*`).

---

## 10. Found while designing

1. **Neither test builder builds a board at rest** (§3's probe). Not changed
   here (§3.3). Whether `put_on_battlefield` should stop announcing is item
   188's survey (A6h), which now has the construction door to compare against.
2. **Two seed derivations** (§6). **Converged at #208** (finding 23), ahead of
   the tools PR.
3. **CR 304.4 and 307.4 have no check on the entry path**: nothing stops an
   instant entering the battlefield, and the loader is the first to refuse one.
   Whether a registered card can put a non-permanent card there
   (`ReturnToBattlefield` is the primitive to read) is checked in the build and
   filed as an item with its reachability and slot. **Answered by SU-1:** none
   can, and the check is `codebase-state.md` item 197.
4. **`shuffle_library` cites CR 701.20 for shuffling.** In `tmnt.txt` 701.20 is
   Reveal and Shuffle is 701.24. A one-word fix that rides with the build.
   **Fixed in SU-1.**

---

## 11. Out of scope

Hidden information (B4; the dev GUI shows every card); four seats in the GUI
(the seats PR, §7b's decision 4);
save, undo and savestates (A6g's tools, SU-4 and SU-5); item 193 (playable); the
stack and resolved effects as written state (§2), which SU-2's setup actions
play instead; a script for a seat during play (§5.3, dropped at review); the
board editor (SU-3, §7b); CR 103.5's mulligans and CR 103.6's opening-hand
actions, named in the header and owned elsewhere. For the why panel (§7c): a
why in `cli_play`, which `ui::why` makes possible since it is the engine's,
but which nothing builds; and a why about an earlier question's board, which is
Undo answer or a savestate away.
