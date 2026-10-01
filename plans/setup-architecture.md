# Game setup — CR 103's doors, the scenario loader, and the save

> **Status:** design, 2026-09-30, revised 2026-10-01 over three review rounds
> on PR #204, which carries the design alone. It covers `roadmap-v2.md` A6g's
> scenario work: the scenario loader as phase **SU-1**, setup actions as
> **SU-2**, and a board editor as **SU-3**, each its own PR. Item 194's PR,
> before SU-1, built CR 103.8's derivation (§1), SU-1 the loader, the writer
> and the dev GUI's start, and SU-2 setup actions (§8, both landed
> 2026-10-01); SU-3 is not built.
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
   and moves between, go in A6g's tools PR (the one that builds undo, the save
   and the export) as bookmarks in the log (the owner, 2026-10-01).
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
   state, and a board editor in the dev GUI, **SU-3**, right after SU-2 (moved
   ahead of playable and the tools PR at #206's review, the owner, 2026-10-01).

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
start. An exact replay needs the start, every answer in order, the same engine,
and until item 193 lands the same seat modes (the priority window's blacklist
filters the offered list by who sits at the seat, so the same index means a
different action).

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
scenarios, since four seats in the GUI are out of A6g's scope and the bridge
builds two providers. Tests and `fuzz_games` load any count.

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

**One function derives both from the seed.** Today there are two derivations:
the dev GUI's dealt start uses the seed, +1 and +2, and `fuzz_games` uses the
seed and two XOR salts. The scenario takes `fuzz_games`' salts, moved into the
engine with the same values, so no fixture row moves. The dev GUI's dealt start
converges on it in the tools PR, where the save pins the streams. Converging now
would redraw the review pictures, which seed 33's stream picks, for nothing.

**What a replay needs.** Once the log records every seat's answers (the tools PR's
decision, recommended there), a replay needs no agent stream. The streams
matter only for playing on past the log's end, and a replay stops depending on
the agent's policy. **What can still break it:** item 193's blacklist, which
playable removes before the tools PR, and a sweep that leaks map order, which
`CLAUDE.md`'s three runs under three `MTGSIM_HASH_SEED`s catch. `fuzz_games
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

**Seats.** The window plays seat 0, and the random agent plays the others. To
choose the opponent's responses too, the window plays every seat (~60–90
lines: a prompt names its player, and "(you)" follows the seat being asked).
With no script for a seat during play (§5.3), that is how a tester explores
both sides of an interaction from one board. Playable's PR builds the seat's
controls and is its natural home.

**Savestates** (the owner's suggestion, placed in the tools PR by the owner,
2026-10-01) are positions the tester sets during play and moves between, like a
video game's save slots. They belong there because they are its replay with
more than one stop. A savestate is a bookmark at an answer in the decision log,
and moving to one replays the start and the log up to it, exactly, at any
prompt. A cloned `GameState` would resume only at a round's start (item 140).
Playing on from an earlier bookmark starts a branch, and the save keeps every
branch. ~100–200 lines on top of undo. A savestate can also be written out as a
scenario, with the writer's report.

**A scenario and a log as a regression test**, the shape the tools PR's
export writes:

```rust
#[test]
fn holy_strength_under_humility() {
    let scenario = Scenario::parse(include_str!("../scenarios/holy-strength.scenario")).unwrap();
    let mut game = scenario.build(&CardRegistry::default_registry()).unwrap();
    let script = ScriptedDecisionProvider::new();
    script.expect_pick_n(ChoiceKind::PriorityAction, vec![2]); // answer 1, seat 0
    // … one line per logged answer, every seat's
    game.resume(&script).unwrap();
    assert_eq!(/* the board, or the event log */);
}
```

What the tools PR owes it, decided there: a log that records every seat (today it
records seat 0's, and the agent's replay only from its seed); a run that stops
at the log's end, since a `DecisionProvider` cannot answer "stop", which is
also what replaces the setup driver's refusal in play, a panic until then
(§5.3); and a `ChoiceKind` built from a logged name (`SelectRecipients` carries
fields the scripted provider ignores).

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
5. **Typing a board in at all** (SU-3). A board editor in the dev GUI: search a
   registered name, put the card in a zone, then click its controller, status,
   counters, attachment and combat. It edits the same `Scenario` value the
   parser builds and saves through the writer, so it adds no third road. It is
   a plain-Rust editor model with tests under a thin egui layer, the GUI review
   path's shape, at ~500–900 lines with its tests. **Right after SU-2** (the
   owner, at #206's review, 2026-10-01), ahead of playable and the tools PR:
   a board is built by clicking before one is saved from a game. It needs only
   the `Scenario` value and the writer, both SU-1's; the editor's undo is its
   own model's, so the tools PR's replay is not needed to build a board.

So a board reaches a scenario three ways, all into one `Scenario` value:
written as text (SU-1), saved from a game and edited (SU-1's writer), or built
in the editor (SU-3).

---

## 8. The build, sized

**SU-3**, the board editor: ~500–900 lines (§7a). SU-1's code came in at about
1.9 times its sizing and SU-2's at about 2.0, their tests near theirs (the
archive's tables say where), so re-size before the build, counting doc
comments and refusal messages as neither sizing did.

### SU-2 — setup actions — ✅ landed 2026-10-01

**What shipped.** The `then:` line in §5.1's table: `then: player p casts
<card>` or `… activates <card>`, each answer a segment of its own after a bar
(`targeting <card>`, `targeting player p`, `ability N`). `Scenario::build`
returns a `BuiltScenario`, the board and its `SetupActions`, each name
resolved through the loader's own table and what can be checked refused
there, naming the line. `SetupDriver`, a `DecisionProvider` over the seats'
own, plays the lines as §5.3 says; `fuzz_games --scenario` and the dev GUI
play through it, the GUI before its first prompt and again on Reload.
`mtgsim/scenarios/bolt-into-giant-growth.scenario` is §5.3's stack, played by
CI's determinism step under three hasher seeds, and the template casts a
Bolt. Item 198 rode along in its own commit.

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
2. **Two seed derivations** (§6), converged by the tools PR.
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

Hidden information (B4; the dev GUI shows every card); four seats in the GUI;
save, undo, savestates and export (A6g's tools PR); item 193 (playable); the
stack and resolved effects as written state (§2), which SU-2's setup actions
play instead; a script for a seat during play (§5.3, dropped at review); the
board editor (SU-3, §7a); CR 103.5's mulligans and CR 103.6's opening-hand
actions, named in the header and owned elsewhere.
