# The information model — who can see what

> **Status:** design, 2026-11-15, at its design stop: nothing here is built,
> and every numbered decision waits on the owner. `roadmap-v2.md`'s row
> `information-model-design`; the build is `information-model`, at step 8.
> Counts were read against `4eddc8ab`.
> **Graduates:** `backlog.md`'s "2.9 The information model" and "2.34 A search
> over a cloned `GameState` sees its next draw", and what "2.38 The v1 GUI"
> parked here under "What a client is handed".
> **Intake:** `codebase-state.md` item 357 (`ui/` holds three kinds of code;
> decision 9), and for the build items 130 (a discard put into a hidden zone
> unrevealed) and 432 (a chosen value over a hidden zone, and secrecy).
> **Instruments, committed with this note:**
> `plans/references/information-census.py`, the card count. The cost probe
> was a throwaway crate outside the tree; its readings are in
> "The costs, and a ceiling for each".

**What changes in a game: nothing a player sees yet.** Today every seat is
handed the whole game. This note decides how the engine answers "may this
player see this card's face?", what it remembers each player was shown, and
what a seat is handed in place of the whole game.

## Where it sits

**Mostly off the event path.** Three places touch it
(`engine-map.md#11-the-path-in-one-sketch`):

```
proposal ─ DECIDE (CR 616.1) ─ PERFORM ─ records ─ triggers ─ PLACE ─ RESOLVE
                                  │          │         │
                                  │          │         └ CR 603.2f: "visible to all players", asked per candidate
                                  │          └ each record is stamped with who saw the card it names
                                  └ a move into or inside a hidden place writes who can still follow the card
```

Everything else is the boundary every step asks its questions across
(`setup-architecture.md#7e-pausing-at-a-decision`): what the asked seat is
handed with the question.

**Three audiences, and the design is the line between the last two.**

| Audience | Sees | Today | After |
|---|---|---|---|
| The rules | everything; a few rules ask who else can see a card (CR 603.2f, 701.9c, 406.3b) | `visible_to_all`, one caller's worth | one query |
| The table: saves, replays, the trace, the scenario writer, the dev GUI in its tester's mode | everything, on purpose | everything | everything, by a named door |
| A seat: a person's client, an AI's observation, an agent in process | what its player may see | everything | its sight |

## Counted

### The tree, today

Two read-only censuses of `4eddc8ab`, spot-checked by hand where a decision
rests on a row.

| | Count | Where |
|---|---:|---|
| `DecisionProvider` implementations | 51: 18 in `mtgsim/src`, 29 in `mtgsim/tests`, 4 in the dev GUI | `ui/decision.rs:162` is the trait |
| …that never read the game, or only pass it on | 22 (8 and 14) | |
| …that read it only to name an option | 7 | `as_logged`, `position_named` |
| …that read what every player may see | 13 | the stack, the turn, the phase, a trigger's timestamp, the asked player's own mana sources |
| …that read the asked player's own hand | 1 | `fuzz_games`' `Observer` (`bin/fuzz_games.rs:1048`) |
| …that read what no player may see, or take the whole game | 8 | a fork's clone, the scenario writer, the dev GUI's `Snapshot::build`, a test's hook |
| Per-viewer queries | 0 | `Zone::is_public` (`types/zones.rs:54`) has eight callers, none takes a player |
| Writers of `PermanentState.face_down` outside tests | 0 | it is always false |
| Prompt kinds whose options are hidden cards | 4: `PriorityAction` (the hand), `Discard`, `Scry`, `OrderLibraryCards` | each asks the cards' owner; no search, look or reveal exists |
| `GameEvent` variants that name a card in a hand or library | 5: `ZoneChange`, `CardDrawn`, `LeftTheGame`, `SameZoneMove`, `AbilityActivated` from a hand | `events/event.rs:87` |

Four things follow that the decisions rest on.

- **An `ObjectId` is a card's name for the whole game.** It survives every
  zone change; only `zone_change_epoch` moves (`ObjectRef`, `types/ids.rs:460`).
  So an id a seat has seen once is a handle on that card wherever it goes
  (decision 4).
- **Every name helper reads the printed card.** `printed_name`
  (`ui/display.rs:58`), `object_label` and `DecisionOption::as_logged` return
  `card_data.name` for any id in any zone, face down or not.
- **The table's records are whole by design**: the decision log's start holds
  every deck and the seed, a `decision` trace record every offered option,
  the trace's names table every object. They stay whole (the table's), and no
  seat is handed one.
- **`GameObject` is 48 bytes with 4 of padding** (measured): a set of players
  fits in it at no cost (decision 2).

### The rules

| Rule | What it says | Watched by |
|---|---|---|
| 400.2 | Library and hand are hidden "even if all the cards in one such zone happen to be revealed" | 603.2f, 603.10a, 400.7j |
| 401.2, 401.3, 402.3, 404.2 | a library's cards and order are hidden from everyone; a hand from everyone but its owner; any player may count either; a graveyard is open | |
| 401.4 | cards put into a library together are ordered by their owner, who "doesn't reveal the order" | 701.20d |
| 401.5, 401.6 | a top card revealed for as long as an effect lasts is not shown while a spell is being cast, and one that stops being revealed "becomes a new object" | 601.2i |
| 406.3, 406.3b, 406.4, 601.3f | a card exiled face down is seen by no one, but a player once allowed to look "may continue to look" until it leaves exile or is shuffled in a pile; casting from among such cards needs that look | 607, 708.4 |
| 701.20a, d, e | a reveal shows a card to all players for as long as the effect needs it; a reordered library card stops being revealed; "look at" is the same, to one player | 603.2f |
| 701.23a, b, e | a search looks at every card of the zone; a found card is not revealed unless the text says so | |
| 701.9c | a discard put into a hidden zone "without being revealed" has undefined characteristics | 732 |
| 708.2, 708.5, 708.9, 708.12 | a face-down object has only its listed characteristics; its controller may look at it; it is revealed as it leaves, when its owner leaves, and at the game's end | 707.2, 613 |
| 603.2f, 603.10a | a trigger needs its source "visible to all players"; a look-back needs "an object that all players can see" | |
| 400.7j | an effect finds an object it moved "to a public zone" | 608.2h |
| 722.4, 808.5, 810.5 | what a controlled player sees, the controlling player sees; teammates may look at each other's hands | |

**The CR defines sight and never memory.** It says who is shown a card and
for how long. What a player still knows afterwards is outside it, and it is
what a client draws and a search must not exceed. Decisions 1 and 2 are that
second thing.

### The printed cards

`python plans/references/information-census.py`, over the Commander-legal
paper corpus (32,115 cards, fetched 2026-10-05), read 2026-11-15. A card is
counted by the clause that does the thing, reminder text stripped; a keyword
off the card's keyword list. Families under 40 were read by hand, and twelve
of each larger one. `tdc` is the five decks' lists.

| What the model must do | Family | Cards | `tdc` | Example | Holds today |
|---|---|---:|---:|---|---|
| Show a card to all, once | reveal, any | 1,249 | 10 | Thoughtseize | nothing: a no-op |
| | …a hand | 152 | 0 | Cabal Therapy | |
| | …the top of a library | 319 | 6 | | |
| | …until a card matches | 110 | 2 | Selvala's Stampede | `backlog.md` "2.46" |
| | …a card as it is drawn | 28 | 0 | God-Eternal Kefnet | |
| Show a card to all, while an effect lasts | a hand | 8 | 0 | Telepathy | nothing |
| | the top card of a library | 16 | 0 | Courser of Kruphix | |
| Show a card to one player, once | look at, any | 1,384 | 22 | Ponder | scry's prompt only |
| | …scry or surveil | 685 | 14 | Opt | asked; nothing recorded |
| | …the top of your own library, otherwise | 492 | 6 | Ponder | |
| | …another player's hand / the top of their library | 37 / 41 | 0 | Gitaxian Probe / Cruel Fate | |
| | search | 987 | 18 | Cultivate | nothing |
| | …another player's library or hand | 115 | 0 | Praetor's Grasp | |
| Show a card to one player, while an effect lasts | the top card of your library | 51 | 1 | Thundermane Dragon | nothing |
| | a face-down creature you don't control | 3 | 0 | Found Footage | |
| | cards exiled with this permanent | 4 | 0 | Grimoire Thief | |
| Move a card between two hidden places, unseen | a hand into a library | 41 | 0 | Brainstorm | moved; every seat watches |
| | a library into a hand, unrevealed | 161 | 3 | Diabolic Tutor | |
| | into a library "in any order" (CR 401.4) | 136 | 0 | | |
| | onto the bottom "in a random order" | 286 | 2 | | |
| Hide a face in a public zone | cast face down: morph, megamorph, disguise | 227 | 0 | Exalted Angel | `face_down`, never set |
| | put there face down: manifest, cloak | 73 | 1 | Reality Shift | |
| | another way onto the battlefield or the stack face down | 52 | 0 | Ixidron | |
| | exiled face down, any | 136 | 2 | | nothing |
| | …foretell / hideaway | 51 / 15 | 0 / 2 | Mosswort Bridge | |
| | …from another player's hand or library | 16 | 0 | Jester's Scepter | |
| Keep a choice from the other players | "secretly" | 17 | 0 | Call to the Void | nothing: announced |
| | a vote (CR 701.38): 33 open, 6 secret | 39 | 1 | Selvala's Stampede | not built |
| Draw from the random stream | shuffle | 1,193 | 21 | | `GameState.rng` |
| | a card taken at random / a player at random | 169 / 16 | 0 / 1 | Territorial Hellkite | |
| | a random order | 293 | 2 | | |
| | a coin / a die | 71 / 92 | 0 | | `backlog.md` "2.31" |
| Read sight in a rule | an ability working from a hand, seen as it does | 91 | 1 | cycling triggers | 603.2f by zone |
| | a discard put into a library (701.9c) | 2 | 0 | Library of Leng | item 130 |

**Rulings read.** Call to the Void (2022-06-10): each player keeps the choice
"secret until all players simultaneously reveal". Selvala's Stampede
(2016-08-23): "each player will know the votes of players who voted
beforehand". Courser of Kruphix (2021-03-19): a new top card is not revealed
"until you finish" casting, and of several cards put on top only "the one
new top card" is. Jester's Scepter (2006-07-15): "You can see the cards that
are exiled… but no one else can". Praetor's Grasp (2011-06-01): "Other
players, including the card's owner, can't look at the card".

**The clients.** Every surface below serves a person at the four-seat GUI and
an AI client alike, since both are a seat. The redeal (decision 6) serves an
AI's search alone. The door to the whole game serves neither: it is the
table's, for tests, the dev GUI's tester and the scenario writer.

## The model

**Two questions, kept apart.**

1. **Sight, now** (the CR's): may this player look at this card's face at
   this moment? Derived each time it is asked, from the zone, the owner, a
   face-down permanent's controller, an effect in progress, and an effect
   that lasts. One query, beside `oracle/characteristics.rs`, as
   `backlog.md` "2.9" sited it.
2. **Shown and followed** (memory): which players were shown this card and
   could follow it since? Stored on the card, written where the card moves or
   is shown.

A seat's copy shows a face when either says yes, and says which.

**The rule that writes the record: a move a player can follow.** A player
follows a move when they can tell which card moved: it left a public zone, or
a known place in a library (the top, the bottom), or they chose it from their
own hand, or it was revealed as it moved.

| A card… | Who is on its record afterwards |
|---|---|
| arrives face up in a public zone | everyone (nothing is stored; the zone answers) |
| is bounced from the battlefield to a hand | everyone: all saw it go |
| is drawn | its owner, and anyone who knew the top card |
| is revealed, or looked at | the players shown, added to who was there |
| goes from a hand to a library or face down, unrevealed (Brainstorm, a morph) | the mover alone; **and every other player is taken off every card left in that hand**, since each could have been the one that went |
| is among library cards reordered where others cannot see the order | the player who ordered them (CR 401.4, 701.20d) |
| is in a library that is shuffled, or goes to the bottom "in a random order" | nobody |

The last-but-two row is the model's one subtle rule. An opponent knows your
hand is A, B and C; you put one back unseen. Dropping only that card from
their copy of your hand tells them which it was. So an unseen departure
clears what they knew of the whole hand.

## The binding table

| State | Bound | Live or latched | Stored | Read by | Ends |
|---|---|---|---|---|---|
| who was shown a card and has followed it (`face_shown_to`) | when it is revealed, looked at, or moves where some can follow | latched | the object's record (decision 2) | a seat's copy, the redeal, CR 406.3's continued look | its pile is shuffled or reordered; an unseen move; it arrives face up in a public zone |
| a reveal or look in progress (CR 701.20a) | as the instruction runs, or as a cost reveals | latched for the effect | the resolving record; for a cost's reveal, the stack entry | the sight query, CR 603.2f, 701.9c | the resolution ends, or the spell leaves the stack |
| sight from an effect that lasts (Telepathy, Courser of Kruphix, Found Footage) | never: read off the effect each time | live | nowhere | the sight query | with the effect, by CR 611's own rules |
| that a choice is secret | at the choose step, off the card's word | latched | on the kept choice | a seat's copy of the question, the record and the kept list | the step that reveals it |
| a face-down object's real face | `card-and-object-model`'s (decision 8) | | | the sight query alone | CR 708.9 |
| who saw the card a record names | as the record is written | latched | the record's stamp, off `GameState` | a seat's copy of the stream | never; the stream is the table's |

## Decisions

Numbered from 1. Three topics the brief listed are not choices and sit under
"Settled by the rules" after decision 10: a card followed into a hidden zone,
card names over hidden zones, and the rules' own readers.

### Decision 1 — what the engine remembers a player was shown

**Retrofit cost.** Low while nothing reads it: the record's meaning is fixed
by the move rule's dozen lines and the seat's copy. After the dev GUI and an
AI's observation are built on it, a change moves what every seat's copy
holds and every redeal.

**The problem.** A reveal lasts a moment (CR 701.20a), but a player goes on
knowing the card. A client draws that, and a search may use exactly that and
no more (`backlog.md` "2.34": the redeal "must keep every known card in
place"). The record must fork with the game, so it is on `GameState`, and
floor 3 says it grows with the board, never with the turn count
(`plans/references/ai-performance-floors.md`).

- **A. Cards a player can point at.** One set of players on each card: those
  shown it who have followed it since, by the table above. An unseen
  departure clears a hand for those who did not see it. *Code*: the move
  rule at the one place a card arrives in a zone, and four clears.
  *Performance*: a few bit operations a move. *Upkeep*: one rule; a new verb
  that moves a card says only whether it reveals. *What it loses*: a player
  who knew three cards of a hand, after one unseen departure, knows two of
  three remain. The engine forgets all three. A client still has them in its
  own record of what it was shown, and may draw them as "seen earlier".
- **B. Cards a player has seen in each hidden zone, with what has left
  unseen.** A list per viewer and zone of faces seen there, and a count of
  unseen departures since each. Exact for a hand. *Code*: the lists, their
  upkeep on every departure, and a seat's copy that draws "one of these has
  gone". *Performance*: a list for each viewer of each hand and library, 32
  at four seats, and each one that is not empty is an allocation a clone,
  against floor 2's 64 and today's 39. *Upkeep*: a second kind of
  knowing ("possibly here") in every reader, and a redeal that samples under
  "two of these three".
- **C. Nothing stored: sight alone**, and each client keeps its own memory.
  No redeal can then be built from a fork, which the floors' survey refused
  (2026-09-25).

**Recommended: A.** It is never wrong about what it shows, which is the
property a seat's copy and a search both need, and what it forgets a client
can keep for itself. B's exactness is a belief model, and where those stop is
a research question the engine should not own.

### Decision 2 — where the record is stored

**Retrofit cost.** Small: one field and its handful of writers, behind the
query. No game, log or save moves, since the table's records do not hold it.

**The problem.** Floors 2 and 3 price every fork
(`engineering-practices.md#the-floors`): 64 allocations and 128 KB a clone,
read at every decision by `clone_bound_test`. The latest reading is 39 and
8.7 KB. The object store is shared between forks in chunks of 32
(`clone-architecture.md`, decision 1), so a clone copies a pointer a chunk,
whatever an object holds.

| | A clone, 4 seats · 2 seats | A shuffle | A fork's write |
|---|---|---|---|
| **A. A set of players on `GameObject`** (`face_shown_to`, 16 bits in the record's 4 bytes of padding) | +0 allocations, +0 bytes · the same | reads each card of the library, writes the few with a bit set: about 0.1 µs for 99 cards, derived from the store's read cost | copies the card's chunk, 1.5 KB, as any move of it does |
| **B. A's set with an epoch beside it**, each hidden zone counting its shuffles | +0, +0, if the stamp fits; 8 bytes an object in the shared chunks otherwise | one write | the same, and a compare a query |
| **C. A list on each hidden zone** of (card, players) | +1 allocation for each list that is not empty: up to 8 · 4 | clears a list | the list's own copy |
| **D. A table of its own**, keyed by id, shared in chunks | +1 allocation once any card is marked | reads the library | a small chunk of its own |

`backlog.md` "2.9"'s input was B: "stamp knowledge facts with an epoch… which
spares a sweep over each fact". The sweep it spares is a read of cards the
shuffle is already rewriting the order of, and only a card with a bit set is
written. A's cost is the read.

**Recommended: A.** Zero allocations and zero bytes a clone at any seat
count up to 16, no second structure to keep in step with the card, and a
shuffle that costs a tenth of a microsecond more. B if a library of
thousands ever shuffles often, which is a one-field change behind the same
query.

### Decision 3 — what a decision provider is handed

**Retrofit cost.** The highest here, and it grows: every implementation's
four methods. 51 today; each row before step 8 adds test providers. Changing
the answer after the build is the same sweep again.

**The problem.** Four of the trait's five methods take `&GameState`
(`ui/decision.rs:162`). `codebase-state.md` item 141, the serialized
boundary, kept the parameter "until that query exists". Subtracting from the
whole game leaks each new field until someone remembers it; adding to nothing
fails closed (`backlog.md` "2.9", inputs).

- **A. Keep `&GameState`**, and make honesty the harness's: an adapter
  builds an AI's observation through the sight query. *Code*: none.
  *Upkeep*: every agent in process (the random agent, each decorator, a
  future heuristic) can read any hand by a typo, and nothing fails.
- **B. A seat's sight, read on demand.** `SeatSight<'_>` is the game and a
  player, two words, private to its module. Its methods are the only way
  through: the stack, the turn, a hand it may see, a card's face if sight or
  the record allows. A provider that never looks pays nothing. The table's
  instruments (a fork's clone, the scenario writer, the dev GUI's tester,
  the audit) take the whole game through one named method,
  `the_whole_table()`, which `check_invariants.py` allows only under
  `tests/`, `test_support.rs`, the dev GUI and a tagged site. *Code*: the
  type and about twenty accessors; 51 signatures swapped by
  `plans/replace_exact.py`; the 13 public readers moved onto accessors; the
  8 whole-game readers onto the door. *Performance*: nothing a prompt.
  *Upkeep*: a new thing an agent wants to read is a new accessor, reviewed
  as an addition.
- **C. An owned copy built at every prompt.** No borrow crosses the
  boundary. *Performance*: k at every decision for every stack, the default
  agents included: 0.125 today, so floor 1's 12,465 reads about 11,080
  (measured, below). *Upkeep*: instruments need a second channel anyway.

**How the 51 move under B** (from the census):

| What it reads | Count | Change |
|---|---:|---|
| nothing, or passes it on | 22 | the parameter's type |
| names of options | 7 | `as_logged` stays the table's (a script matches typed options without it); a seat's label goes through sight |
| the public board | 13 | accessors: `stack()`, `turn_number()`, `phase()`, `pending_triggers()`, the asked player's mana sources |
| its own hand | 1 | `fuzz_games`' `Observer`: the first consumer, and k's real reading |
| the whole game | 8 | `the_whole_table()` |

**Recommended: B.** It is the only one that fails closed and costs nothing
at a prompt. It leaves `setup-architecture.md#7e-pausing-at-a-decision`'s
decision 2 as taken: a paused game hands a client the owned copy
(decision 5) built from the same accessors, "on the game's stack at the
question".

### Decision 4 — what a seat's copy names a card by

**Retrofit cost.** High once a client exists: every object a client is
handed and every answer it sends is named this way. Nothing changes inside
the engine either way.

**The problem.** An id follows a card through every zone. A creature is
bounced, so every player knows that card is in its owner's hand. The owner
casts a face-down spell. If the spell carries the id the creature had, every
client knows what it is; at a table no one would. The same id links a card
revealed on top of a library to the card played five turns later.

- **A. The engine's id everywhere**, as today. Leaks every face-down object
  whose card a seat has met, and makes a policy that tracks ids stronger
  than one that plays the game.
- **B. By existence.** A seat's copy names an object by the tick of the move
  that made it (`zone_change_epoch`, minted one a move at
  `engine/zones.rs:259`), which CR 400.7 says is a new object. A record of a
  move a seat can follow gives both names, so a client still animates one
  card from a hand to the battlefield. A move it cannot follow gives only
  the new one. An object that has never moved (a token as it is made, an
  ability on the stack, a card still in its opening library) is named by the
  id it was made with, which has been nowhere else. *Code*: one function in
  the copy's builder. An answer names an option as the copy named it, and
  the engine finds its place among the options it offered.
- **C. The id for a face-up object, a made-up handle for a face-down one.**
  Closes the morph and leaves the library link; two naming rules in every
  client.

**Recommended: B.** One rule, which is the CR's own, and it cannot leak
since the name holds nothing about the card. An agent in process still reads
ids through `SeatSight`: it is trusted code, and the copy is where the names
change. `backlog.md` "2.36" (how an object is named in text) keeps its slot;
this decides the machine name only.

### Decision 5 — events and answers, as a seat sees them

**Retrofit cost.** Medium: a stamp on each record and one function. The
table's stream, the trace and every fixture table are unchanged under A.

**The problem.** `CardDrawn` names the card, and so does a `ZoneChange` from
a library to a hand; the log line every seat reads is "P1 drew Counterspell".
Who may see a card must be answered as of the event, since by the time a
client reads the record the card has moved on. The stream left `GameState`
with `codebase-state.md` item 42, so a per-viewer history costs no floor
wherever it is kept.

- **A. One stream, each record stamped with who saw the card it names**, as
  it is written (a set of players, four bytes). A seat's copy of a record is
  built by one function: the record as it is when the seat is in the stamp,
  and with the card left out ("P1 drew a card") when not. An answer another
  seat gave goes through the same function option by option, and a kind
  marked secret (decision 7) is held back whole until its reveal. *Upkeep*:
  a new record kind that names a card says which of its objects the stamp is
  about.
- **B. A stream per seat, written as the event happens.** N copies of every
  record; each sink decides at once and keeps no stamp. A fork's recorder
  branches N streams.
- **C. A second event vocabulary for clients**, written out by hand beside
  `GameEvent`. Every variant is spelled twice and kept in step.

**Recommended: A.** One writer, one projection, and the table's stream stays
the trace's and the fixture tables'. The wire form of a record is Phase 10's
(`backlog.md` "2.38"), built from A's projection.

**What the table keeps whole**: the decision log (every deck, the seed,
every answer), the trace, the scenario writer. A seat is never handed one. A
save a player keeps of their own game is the table's record, so it holds
every hand: where v1 keeps it is `backlog.md` "2.38"'s open question 5.

### Decision 6 — the redeal for a forked search

**Retrofit cost.** Low: one function over the record, with no client before
the AI harness.

**The problem.** A fork holds the true game. A search over it knows its next
draw (`backlog.md` "2.34"). Before each branch the fork is rebuilt from what
the searcher knows: every card the searcher cannot place is shuffled among
the places it could be.

- **A. The engine's.** `GameState::redeal_as_seen_by(player)`, on a fork:
  the searcher's own library, among the slots it does not know; each other
  player's hand and library together; a face-down object's card with a
  hidden card of its owner that could have been put there the same way (any
  card for a manifest, a card with morph or disguise for a face-down cast).
  It draws on the fork's own `rng`, so a branch is seeded and replays. One
  state a branch, every seat's hidden cards sampled from the searcher's
  view, as "2.34" settled.
- **B. The harness's**, over a public list of what is pinned. Each harness
  writes the exchange rules again, and the face-down rule is a rules
  question.

**Recommended: A**, built after v0.5 beside the AI harness, its only
client (see "The build, sized"). Its test is "2.34"'s: a searcher over many
samples predicts its next draw no better than chance.

### Decision 7 — a choice kept from the other players

**Retrofit cost.** Low now; `several-choosers-at-one-step` is the next row
and writes whichever shape is taken.

**The problem.** 17 cards choose "secretly, then reveal". Each player
chooses; nobody sees another's choice until all are shown. A vote is the
opposite, and its ruling says so: later voters know earlier votes.

- **A. A mark on the choose step, and the engine asks in turn as it does
  now.** The step says its choices are secret and where they are revealed.
  Each answer is kept as any choice is. Until the reveal, no other seat's
  copy holds it: not the question's words, not the record of the answer, not
  the kept list. Asking in turn then shows nothing that asking at once would
  hide.
- **B. Ask every chooser at once**: a new boundary shape, several seats
  pending together. It is what the card literally says, and with A's
  withholding it changes nothing a player can observe but how long they
  wait.

**Recommended: A.** What `several-choosers-at-one-step` must record is the
mark and the reveal step, no more. A choice kept on a permanent in secret
(Emissary of Grudges: "as this enters, secretly choose an opponent", then
"Reveal the player you chose:" as a cost) is `linked-abilities`' kept value
carrying the same mark.

### Decision 8 — a face-down card's real face: what this asks of `card-and-object-model`

**Retrofit cost.** High if left to chance: every reader of an object's
printed card is a possible leak, and there are name helpers that read it
today for any id.

**The problem.** A face-down permanent has only its listed characteristics
(CR 708.2), yet its controller may look at it (708.5), it is revealed as it
leaves (708.9), and a reveal for information reads it ignoring continuous
effects (708.12). 478 printed cards put a card face down somewhere, two of
them in the decks' lists beside Reality Shift: Mosswort Bridge and Windbrisk
Heights, whose hideaway exiles one.

- **A. The object keeps its card, and face down is a status the readers
  honor.** `card_data` stays the real card, as `PermanentState.face_down`
  implies today. Every printed read must then check the status, and
  `check_invariants.py`'s printed-read rule gains a face-down leg.
- **B. The real face is behind sight.** While an object is face down, its
  record's card is what the rules say it is (a 2/2 with no name), and the
  real card is held where only the sight query and CR 708.9's reveal read
  it. A reader that forgets sees the face-down card, which is the right
  answer by accident.

**Recommended: B as a requirement**, with where the real card is held left
to that design. This note fixes three things for it: the look is the sight
query's (the controller by CR 708.5, a lasting grant such as Found
Footage's, and the record); a card exiled face down carries the same record,
which is CR 406.3's "may continue to look"; and the query answers CR 406.3b
and 601.3f ("only if they can look at the face-down card").

### Decision 9 — where `ui/`'s three kinds of code live (`codebase-state.md` item 357)

**Retrofit cost.** None in behavior: moves, at any time. What a move costs
is its sweep of import paths, counted on the tree: the question boundary
(`ask`, `decision`, `decision_types`) is imported by 156 files, the policies
by 73, the words by 43.

**The problem.** `ui/` holds the engine's question boundary, the policies a
seat chooses, and the words a client shows a person, about 9,900 lines. The
third is what a viewer sees, so it takes a seat's sight; the second must not
reach past it.

**The hazard is closed without a move.** What keeps a policy or a word from
reading past a seat's sight is `SeatSight`'s private field, and privacy is a
module's: it holds wherever the other files sit. So a move's payoff is a
name and a smaller folder, not a hazard.

- **A. Three modules now**: `decision/` (asking, the kinds, the trait,
  `SeatSight`), `agents/` (the random agent and the decorators), `words/`
  (`display`, `why`, `what_happened`, `waiting`). A PR of moves alone, about
  176 files' imports through `plans/replace_exact.py`, counted zero in the
  band and read by its script.
- **B. Leave `ui/` and add `SeatSight` to it; split at Phase 10**, with
  `backlog.md` "2.38"'s open question 6 and item 208, which redraw the crate
  boundary and move these files anyway. One sweep where A then C is two.
- **C. Separate crates now.** The engine's own output borrows `display`'s
  words today, so the words cannot leave until the trace's reader does.

**Recommended: B.** A is consistency alone and costs a sweep the Phase 10
decision repeats. Item 357 is then re-slotted to Phase 10 beside item 208,
with this finding: the words and the policies each take a `SeatSight`, so
the split's seam is already in their signatures.

### Decision 10 — what `backlog.md` "2.38" parked here

The owner put five things on this design (2026-11-04 to 2026-11-14). Four
have a home; one is a choice.

| Parked | Disposition |
|---|---|
| The structure a client is handed | Taken: a seat's copy is three parts, the board through sight (decision 3), what happened since (decision 5), and the question, each object named by existence (decision 4). Its wire form stays Phase 10's |
| One table per `DecisionKind`: its name, log word, scenario word and fields | Taken in the build's `a-seats-copy` PR, generated from the code, since the copy's question needs each kind's fields listed |
| A setup line's word for a division, and `ability N` | Re-slotted to that table's PR: both are words of the scenario form, which the table lists and nothing here shapes |
| What an attack declaration answers with | Re-slotted to `combat-requirements`, which reshapes the declaration (`roadmap-v2.md`, step 11) |
| The error a client is handed | Stays with `plans/handoffs/process-pr.md`, which makes a branched-on error a typed enum. What this adds: an error that names a card goes through decision 5's function like a record |

**The choice: do an agent's answers name options instead of places?**
`setup-architecture.md#7e-pausing-at-a-decision`'s decision 2b took identity
for a client's answers and left a provider in process answering by place.
"2.38" asked to take the place out of the trait too.

**Retrofit cost.** Every implementation's return values, once. Decision 3
already rewrites every signature, so taken together it is one sweep, and
apart it is two.

- **A. With decision 3's sweep, by reference.** `pick_n` returns
  `Vec<&DecisionOption>` out of the slice it was handed, `allocate` each
  bucket with its amount, `choose_ordering` the items in order. A provider
  can only return what it was offered; no clone, no index out of range, and
  `allocate`'s parallel list is gone. Each implementation that makes an
  answer itself changes about a line a method; the 14 that only pass one on
  change their return types. Counted at the PR's intake.
- **B. Leave places in the trait**, as 2b took.

**Recommended: A.** The cost is lowest in the PR that already touches every
implementation.

### Settled by the rules

- **A card followed into a hidden zone** (`linked-abilities`,
  `resolution-recipients-and-amounts`). CR 400.7j finds an object only in a
  public zone. So a later effect never finds a card in a hand or a library:
  `RememberedObject::found` already refuses, and the recipients note's
  decision 1 fixes `ChosenTargets`, which follows by id today. Inside one
  resolution an instruction may act on the card it just moved ("draw a
  card, then reveal it", CR 121.6c), through the move's own record, and the
  instruction's player is the one who can point at it. What the rows record
  is the move's new existence, which the record already holds.
- **A card exiled face down, found later** (`linked-abilities`). Exile is
  public, so the link finds the card (CR 406.6, 607); its face is the sight
  query's. A lasting "may look at cards exiled with this" (four cards) is
  read off the link while it lasts, and written to the card's record as the
  card is exiled and as its source changes controller, which is CR 406.3's
  continued look for all but a grant gained later.
- **Card names over hidden zones** (`values-kept-past-the-resolution`). The
  list offered is the registry's names, never this game's cards
  (`choices-architecture.md`, decision 7). The registry a game is built from
  is public and a decklist is not: a seat's copy of the game's start holds
  its own deck alone. The engine reads the hand for "all cards with that
  name" as it reads anything; what the row records is the reveal, a step of
  its own, which is what puts those cards on every player's record.
- **The rules' readers.** `visible_to_all` (`engine/triggers/dispatch.rs:248`)
  becomes the query's "shown to every player": the zone, a face-down status,
  a reveal in progress, a lasting reveal. That is CR 603.2f as
  `triggers-architecture.md` S1 left it. CR 701.9c reads the same question
  of the discard's move (item 130). CR 401.6's "becomes a new object" is the
  lasting-reveal PR's.
- **Each random draw named by its kind.** A draw from `GameState.rng` says
  what it was for: a shuffle, a card at random, a player at random, an
  order, a coin, a die, the redeal. A replay and `loop-detection` read the
  kind; no player is shown the stream.

## What each of the four rows records

| Row | Records | So that the model |
|---|---|---|
| `several-choosers-at-one-step` | on a choose step: that its choices are secret, and the step that reveals them (decision 7). A vote records nothing | withholds a secret answer from every other seat's copy until the reveal |
| `values-kept-past-the-resolution` | a reveal as its own step before a filter reads a hand or a library; names offered from the registry | puts the revealed cards on every player's record, and offers no list that shows a deck |
| `card-and-object-model`, the design | where the real face is held, behind sight (decision 8); a face-down mark for a card in exile | answers CR 708.5, 708.9, 406.3 and 406.3b from one query |
| `linked-abilities` | the cards exiled with a permanent, each with whether it is face down; a secret kept value's mark | reads a lasting look off the link |
| `resolution-recipients-and-amounts` | nothing new: a target or "that card" by existence, as its note's decision 1 already has it | never follows a card into a hand or a library |

## The costs, and a ceiling for each

Every recorded reading was taken again before a ceiling was set on it. The
probe was a throwaway crate outside the tree on `clone_bound_test`'s
Commander games (`performance`, seeds 12345 to 12349), release, one thread,
medians of five means.

| Cost | On record | Read 2026-11-15 | Ceiling, and what it admits and refuses |
|---|---|---|---|
| **The observation, k** (`fuzz_games --observe`, 20 games, seed 12345) | 0.106 (7.7 µs against 66.3), then 0.100 at readiness | **0.125**: 6.7 µs an observation, 69 rows, against 53.4 µs of engine a decision | **A seat's copy costs at most 0.025 more than the naive observation in the same sitting, and floor 1 over (1 + k) stays above its limit.** Admits the query on each row (0.003, measured) and a name and a reason a row (about 0.014, derived at 10 ns). Refuses a second layer read a row (0.125). At the latest floor 1 reading, 12,465, the limit is k under 0.246 |
| **The record, a clone** | "bits on each object stay under 1 KB with no allocation" | `GameObject` is 48 bytes, 4 of them padding | **Zero allocations and zero bytes a clone** at every decision, on `clone_bound_test`. A reveal in progress may add one allocation at a prompt inside its own resolution |
| **The redeal, a fork** | 1.7 to 6.1 µs, "about one clone again" | **0.8 to 7.9 µs**, against a clone of 0.7 to 1.9; with a lookup a card, the same within 0.5 | **A clone and its redeal together stay inside floor 2's 10 µs** at every checkpoint. Today 1.7 to 8.8, worst on turn 1, when 40 cards cross between hands and libraries and their chunks are copied |
| **The memo after a redeal** | cold costs "one board walk more, 10–40 µs from mid-game" | one walk of what a seat's copy lists, cold less warm: **11 to 62 µs** from turn 20, 1.5 on turn 1 | **A redeal leaves the memo warm unless a registry row reaches a zone it deals across**, one compare on `RegistryScopeSummary::reachable_zones`, held by a test that counts layer frames with and without a redeal. Cold, it is one walk, which this design does not bound and `layers-architecture.md`'s LL does |
| **The query, a decision** | "the visibility query's instructions per decision", unmeasured | 2.2 ns an object; 167 ns a decision over the 75 objects a copy lists | **At most 5 ns an object on a board with no lasting sight effect.** Admits the measured 2.2 and one count check. Refuses a layer read (97 ns a row) or a registry scan an object |

**What moved since the record.** The clone fell to a fifth with CL-1, so the
redeal is now about five clones and no longer one; the pair still fits the
floor. k rose because the engine got faster and the pool grew: the
observation itself is 6.7 µs against 7.7.

**What k does not price.** Under decision 3's B a provider that never looks
pays nothing, so floor 1's own reading, on the default agents, does not move
with this build. k is paid by a harness that builds a copy at every
decision.

## The build, sized

A count set at a design stop is read with its top as the expected size
(`references/sizing-calibration.md`). Code and tests apart; cards and docs
are beside the band.

| PR | Shape | Consumer | Code | Tests |
|---|---|---|---:|---:|
| **`seat-sight` — the query, and what a provider is handed** | the sight query for the zone defaults and a face-down permanent; `SeatSight` and its accessors; the 51 implementations onto it, with decision 10's answers if taken; `the_whole_table()` and its gate | `fuzz_games`' `Observer`, the random agent, every decorator; atoms 400.2, 401.2, 401.3, 402.3-001, 404.2, 406.3-001 | 600–950 | 400–650 |
| **`shown-and-followed` — the record, reveal and look** | `face_shown_to` and the move rule; reveal and look as actions with their records; a reveal in progress; CR 603.2f and 701.9c reading the query (item 130) | Nephalia Academy's reveal; God-Eternal Kefnet's fixture gains its reveal; a Brainstorm-shaped fixture for the cleared hand; atoms 402.3-002, 402.3-003, 701.20a | 600–900 | 550–850 |
| **`search-and-library-walks` — search, and the top of a library looked through** | CR 701.23's search with its prompt; "look at the top N, put one here and the rest there"; revealing until a card matches (`backlog.md` "2.46") | the decks' search and look cards: the gap census has 39 deck cards needing the model and 20 of them needing nothing else, registered after it | 550–850 | 450–700 |
| **`a-seats-copy` — what a client is handed** | the owned copy: the board through sight, what happened since with each record as the seat sees it, the question; names by existence; a secret answer withheld; the words taking a sight; the kinds table | `fuzz_games --observe` builds it, which is k's reading; a test that no seat's copy of a seeded game holds a card its player was not shown | 550–850 | 450–700 |
| **`devgui-one-seat`** | the window drawn from one seat's copy, with the tester's whole table as a switch | a person at the dev GUI; its own review path | 300–600 | |
| **`lasting-sight` — a hand or a top card shown while an effect lasts** | sight read off an effect; CR 401.5's wait while casting and 401.6's new object | Courser of Kruphix, Telepathy; the one deck card that looks at its top card any time; atom 402.3-004 | 300–500 | 300–450 |
| **`redeal-for-a-seat`**, after v0.5 with the AI harness | decision 6, and the memo's test | the harness's search; "no better than chance" | 250–400 | 250–400 |

**Six PRs at step 8, one of them the dev GUI's, and one after v0.5.** The
first four read 1,000–1,600, 1,150–1,750, 1,000–1,550 and 1,000–1,550, each
under the band's top; `lasting-sight` is under its floor and may ride
`search-and-library-walks` if that lands small. `roadmap-v2.md` carried 3–6
before any design.

**Face-down and secret choices are built in their own rows** on this
design's query: `face-down-and-morph` at step 16, the secret mark in
`several-choosers-at-one-step`, with `a-seats-copy` doing the withholding.

## Atoms and their boards

| Atom | Board | Covered by |
|---|---|---|
| ATOM-400.2-001 | a card in each zone; the query asked for a player who owns none | `seat-sight` |
| ATOM-401.2-001, 401.3-001 | a library's cards refused to every player, its count given | `seat-sight` |
| ATOM-402.3-001 | two hands; each player's sight of both | `seat-sight` |
| ATOM-404.2-001, 406.3-001 | a graveyard and a face-up exiled card, seen by all | `seat-sight` |
| ATOM-402.3-002, ATOM-701.20a-001 | a three-seat reveal of a hand, seen by both others while the effect runs, and on their record after | `shown-and-followed` |
| ATOM-402.3-003 | a three-seat look: the looker sees, the third seat does not | `shown-and-followed` |
| ATOM-402.3-004 | a hand revealed while an enchantment stays, and not after | `lasting-sight` |
| ATOM-406.3-002 | a card exiled face down, seen by none; then by a player granted the look | `face-down-and-morph`, on `card-and-object-model`'s mark: not this build's, since nothing exiles face down before it |
| CR 708's eight, BOUNDARY-708.12-001 | | `face-down-and-morph`; the query is their reader |

CR 603.2f and 603.10a have no uncovered atom; their reader changes body and
its tests stand.

## The route

`information-model-design` moves from step 8 to the front of what is left of
step 5, ahead of `several-choosers-at-one-step`, since four rows before
step 8 each record something this decides. `information-model` stays at
step 8 with its count from the table above. `CLAUDE.md`'s critical path,
`roadmap-v2.md`'s step table and Sizing rows, and the board are changed in
this PR.

## Found while designing

1. **`printed_name` names a face-down or hidden card for any caller.** Latent
   while nothing is face down and every seat sees everything; `seat-sight`
   leaves it the table's and gives a seat a label through sight.
2. **Floor 1's room for k halved.** `backlog.md` "2.9" has k failing floor 1
   above 0.56, against 15,600; at the latest reading it is 0.246.
3. **The dev GUI's `Snapshot::build` writes the whole board as scenario text
   at every prompt**, every library in order. It is the tester's, and
   `devgui-one-seat` keeps it behind the switch.
4. **The tick alone does not name an object.** An ability on the stack takes
   its source's epoch (`engine/put_on_stack.rs:655`), and an object that has
   never moved has none. So decision 4's name is the tick for an object that
   has moved and the id it was made with for one that has not.
5. **A lasting look gained after a card is exiled** is the one place the
   record falls short of CR 406.3's "may continue to look": no card prints
   it, and the query still answers while the grant lasts.
6. **`check_invariants.py`'s printed-read rule does not see `ui/`'s name
   helpers** as a seat's leak, since each is tagged for a record no rule
   reads. Decision 9's `words/` takes a sight, which is what closes it.
