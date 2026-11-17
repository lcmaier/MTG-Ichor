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
> unrevealed) and 432, both halves: a chosen value over a hidden zone or kept
> secret (decision 7), and a pick at random that no record shows (decision 11).
> **Instruments:** `plans/references/information-census.py`, the card count,
> committed with this note. The cost probe was a throwaway crate outside the
> tree; its readings are in "The costs, and a ceiling for each".

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
                                  └ a move that is announced writes who can still follow the card
```

Everything else is the boundary every step asks its questions across
(`setup-architecture.md#7e-pausing-at-a-decision`): what the asked seat is
handed with the question.

**Three audiences, and the design is the line between the last two.**

| Audience | Sees | Today | After |
|---|---|---|---|
| The rules | everything; a few rules ask who else can see a card (CR 603.2f, 701.9c, 406.3b) | `visible_to_all`, by zone | one query |
| The referee: whoever runs the game. Saves, replays, the trace, the scenario writer, the dev GUI in its tester's mode, a training harness | everything, on purpose | everything | everything, through a named door |
| A seat: a person's client, an AI's observation, an agent in process | what its players may see | everything | its sight |

## Counted

### The tree, today

Two read-only censuses of `4eddc8ab`, checked by hand where a decision rests
on a row.

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

These follow, and the decisions rest on them.

- **An `ObjectId` is a card's name for the whole game.** It survives every
  zone change; only `zone_change_epoch` moves (`ObjectRef`, `types/ids.rs:460`).
  So an id a seat has seen once is a handle on that card wherever it goes
  (decision 4).
- **Every name helper reads the printed card.** `printed_name`
  (`ui/display.rs:58`), `object_label` and `DecisionOption::as_logged` return
  `card_data.name` for any id in any zone, face down or not.
- **A cast is silent until CR 601.2i.** The move onto the stack at 601.2a
  and a rewind's move back announce nothing (`engine/zones.rs:37`, the
  `// CAST-ROLLBACK:` tag: "nothing may observe it"). So what a player was
  shown is written where a move is announced, never where it is performed.
- **The referee's records are whole by design**: the decision log's start holds
  every deck and the seed, a `decision` trace record every offered option,
  the trace's names table every object. They stay whole, and no seat is
  handed one. "Referee" is this note's word for whoever runs the game and
  sees all of it; "host" is taken, by an Aura's (`plans/glossary.md`).
- **A model trained on everything and played on a seat's sight needs both
  at one decision.** PerfectDou's training gives its value network every
  hand and its policy only its own seat's. A training harness is a referee:
  it holds the game, so it may build a seat's copy for the policy and a copy
  with every face for the value network, the same type from the same
  builder (decision 3). The whole log is its to read as well.
- **`GameObject` is 48 bytes with 4 of padding** (measured), and no clone
  copies it: the object store is shared between forks in chunks. So the
  record costs a clone nothing at any width (decision 2).
- **Nothing in the engine caps the number of seats today.** A set of players
  held as bits would be its first cap (decision 2).

### The rules

| Rule | What it says | Watched by |
|---|---|---|
| 400.2 | Library and hand are hidden "even if all the cards in one such zone happen to be revealed" | 603.2f, 603.10a, 400.7j |
| 401.2, 401.3, 402.3, 404.2 | a library's cards and order are hidden from everyone; a hand from everyone but its owner; any player may count either; a graveyard is open | |
| 401.4 | cards put into a library together are ordered by their owner, who "doesn't reveal the order" | 701.20d |
| 401.5, 401.6 | under a lasting reveal, a top card that changes while a spell is being cast is not shown until the cast is finished; one that stops being revealed "becomes a new object" | 601.2i |
| 406.3, 406.3b, 406.4, 601.3f | a card exiled face down is seen by no one, but a player once allowed to look "may continue to look" until it leaves exile or is shuffled in a pile; casting from among such cards needs that look | 607, 708.4 |
| 701.20a, d, e | a reveal shows a card to all players for as long as the effect needs it, a cost's until the spell leaves the stack, and one that causes a trigger until that trigger does; a reordered library card stops being revealed; "look at" is the same, to one player | 603.2f, 602.2a |
| 701.23a, b, e | a search looks at every card of the zone; a found card is not revealed unless the text says so | |
| 701.9c | a discard put into a hidden zone "without being revealed" has undefined characteristics | 732 |
| 708.2, 708.5, 708.9, 708.12 | a face-down object has only its listed characteristics; its controller may look at it; it is revealed as it leaves, when its owner leaves, and at the game's end | 707.2, 613 |
| 603.2f, 603.10a | a trigger needs its source "visible to all players"; a look-back needs "an object that all players can see" | |
| 400.7j | an effect finds an object it moved "to a public zone" | 608.2h |
| 722.4, 808.5, 810.5 | what a controlled player sees, the controlling player sees too; teammates may look at each other's hands | |

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
| Show a card to all, once | reveal, any | 1,249 | 10 | Chaos Warp | nothing: a no-op |
| | …a hand | 152 | 0 | Cabal Therapy | |
| | …the top of a library | 319 | 6 | Satyr Wayfinder | |
| | …until a card matches | 110 | 2 | Selvala's Stampede | `backlog.md` "2.46" |
| | …a card as it is drawn | 28 | 0 | God-Eternal Kefnet | |
| Show a card to all, while an effect lasts | a hand | 8 | 0 | Telepathy | nothing |
| | the top card of a library | 16 | 0 | Courser of Kruphix | |
| Show a card to one player, once | look at, any | 1,384 | 22 | Ponder | scry's prompt only |
| | …scry or surveil | 685 | 14 | Opt | asked; nothing recorded |
| | …the top of your own library, otherwise | 492 | 6 | Ponder | |
| | …another player's hand / the top of their library | 37 / 41 | 0 | Sorcerous Sight / Cruel Fate | |
| | search | 987 | 18 | Cultivate | nothing |
| | …another player's library or hand | 115 | 0 | Praetor's Grasp | |
| Show a card to one player, while an effect lasts | the top card of your library | 51 | 1 | Thundermane Dragon | nothing |
| | a face-down creature you don't control | 3 | 0 | Found Footage | |
| | cards exiled with this permanent | 4 | 0 | Grimoire Thief | |
| Move a card between two hidden places, unseen | a hand into a library | 41 | 0 | Brainstorm | moved; every seat watches |
| | a library into a hand, unrevealed | 161 | 3 | Diabolic Tutor | |
| | into a library "in any order" (CR 401.4) | 136 | 0 | Teferi's Puzzle Box | |
| | onto the bottom "in a random order" | 286 | 2 | Ureni of the Unwritten | |
| Hide a face in a public zone | cast face down: morph, megamorph, disguise | 227 | 0 | Zoetic Cavern | `face_down`, never set |
| | put there face down: manifest, cloak | 73 | 1 | Reality Shift | |
| | another way onto the battlefield or the stack face down | 52 | 0 | Ixidron | |
| | exiled face down, any | 136 | 2 | Bane Alley Broker | nothing |
| | …foretell / hideaway | 51 / 15 | 0 / 2 | Mosswort Bridge | |
| | …from another player's hand or library | 16 | 0 | Jester's Scepter | |
| | face down anywhere, a card counted once | 478 | 3 | the four families above | |
| Keep a choice from the other players | "secretly", any | 17 | 0 | the three rows below | nothing: announced |
| | …revealed in the same step, once all have chosen | 12, six of them votes | 0 | Call to the Void, Truth or Consequences | |
| | …revealed after another player's guess | 1 | 0 | The Toymaker's Trap | |
| | …kept on a permanent, revealed as a later cost | 4 | 0 | Emissary of Grudges | |
| | a vote (CR 701.38), 33 of them open | 39 | 1 | Selvala's Stampede | not built |
| Draw from the random stream | shuffle | 1,193 | 21 | Cultivate | `GameState.rng` |
| | a card taken at random / a player at random | 169 / 16 | 0 / 1 | Stupor / Territorial Hellkite | |
| | a random order | 293 | 2 | Velomachus Lorehold | |
| | a coin / a die | 71 / 92 | 0 | Karplusan Minotaur / Sword of Hours | `backlog.md` "2.31" |
| Read sight in a rule | an ability working from a hand, seen as it does | 91 | 1 | Dismantling Wave's cycling trigger | 603.2f by zone |
| | a discard put into a library (701.9c) | 2 | 0 | Library of Leng | item 130 |

**Alchemy, which that corpus leaves out** (it is paper cards). Scryfall's
`total_cards` for each query, 2026-11-17; the rules are
`MTG-Rules/Alchemy-Derived-Rules/`.

| What the model must do | Family, and its query | Cards | Example |
|---|---|---:|---|
| Know that a hidden card was changed, and by whom it is known | a change that follows the card ("perpetually"), `o:perpetually` | 244 | Davriel's Withering |
| | …made to a card in a hand, `o:"in your hand"` | 65 | Blooming Cactusfolk |
| | …to a card its controller picks from their hand, `o:/choose an? [a-z ]*card in your hand/` | 34 | Bloodsprout Talisman |
| | …to a card picked at random from a hand or library | 7 | Fuel Tank Feaster |
| | …to a card in a library, `o:"library"` | 40 | Goblin Morale Sergeant, whose changed card goes "into the top five cards of your library at random" |
| | …to a card in another player's hand | 4 | Lumbering Lightshield, which has it revealed first |
| Move a card out of a library unseen, with no shuffle | seek, `kw:seek` | 122 | Discover the Formula |
| Bring a card into a hidden zone from outside the game | conjure, `kw:conjure`; a spellbook, `o:"spellbook"` | 190; 65 | Sarkhan, Wanderer to Shiv; Charged Conjuration |

**Rulings read.** Call to the Void (2022-06-10): each player keeps the choice
"secret until all players simultaneously reveal". Selvala's Stampede
(2016-08-23): "each player will know the votes of players who voted
beforehand". Courser of Kruphix (2021-03-19): a new top card is not revealed
"until you finish" casting, and of several cards put on top only "the one
new top card" is. Jester's Scepter (2006-07-15): "You can see the cards that
are exiled… but no one else can". Praetor's Grasp (2011-06-01): "Other
players, including the card's owner, can't look at the card".

**The clients.** A person at the four-seat GUI and an AI client are both a
seat, and are handed the same thing: the owned copy of decision 5. The
engine's own agents in process (the random agent, the decorators) read
`SeatSight` directly. The redeal (decision 6) serves an AI's search alone.
The door to the whole game serves no client: it is the referee's, for tests,
the dev GUI's tester and the scenario writer.

## The model

**Two questions, kept apart.**

1. **Sight, now** (the CR's): may this player look at this card's face at
   this moment? Derived each time it is asked, from the zone, the owner, a
   face-down permanent's controller, a reveal or look in progress, and an
   effect that lasts. One query, beside `oracle/characteristics.rs`, as
   `backlog.md` "2.9" sited it.
2. **Known and followed** (memory): which players know this card's face and
   have been able to follow it since? Stored on the card, written where a
   move or a reveal is announced.

A seat's copy shows a face when either says yes, and says which.

**Knowing a card is not knowing what has been done to it since.** A player
was shown Hullbreaker Horror in its owner's hand. Then Fuel Tank Feaster
gives "a random creature card with the greatest mana value among creature
cards in your hand" a lasting "costs {1} less". The player still knows the
Horror is there, and does not know whether it was the card picked. So a
change that follows a card (Alchemy's "perpetually") carries its own set of
players who know of it, written by the same rule as the card's: who could
tell which card it was made to, and afterwards anyone the card is shown to.
A seat's copy draws a card it knows with the changes it knows. Where the
change is held is `card-and-object-model`'s (decision 8).

**The rule that writes the record: a move a player can follow.** A player
follows a move when they could see the card's face as it left, by sight or
by the record, and can tell which card moved: it left a public zone, or a
place in a library everyone can name (the top, the bottom), or they chose it
from their own hand, or it was revealed as it moved.

| A card… | Who is on its record afterwards |
|---|---|
| arrives face up in a public zone | everyone (nothing is stored; the zone answers) |
| is bounced from the battlefield to a hand | everyone: all saw it go |
| is drawn | its owner, and anyone who could see the top card: by the record, or by Courser of Kruphix |
| is revealed, or looked at | the players shown, added to who was there |
| goes from a hand to a library or face down, unrevealed (Brainstorm, a morph) | the mover alone; **and every other player is taken off every card left in that hand**, since each could have been the one that went |
| is among library cards reordered where others cannot see the order | the player who ordered them (CR 401.4, 701.20d) |
| is taken from a library from a place no one can name, unrevealed (Alchemy's seek, with no shuffle after) | its owner, who sees it arrive; **and every other player is taken off every card of that library**, as with a hand |
| is in a library that is shuffled, or goes to the bottom "in a random order", or is put at a random place among its top cards | nobody |
| is brought in from outside the game into a hidden zone (conjure) | everyone when the text names the card; its owner alone when it is picked from a spellbook |
| is put on the stack by a cast, before CR 601.2i | as it was: a cast in progress is announced to nobody, and a rewound one never was |
| is shown by an effect that lasts | the players it shows, written at each state-based check (decision 1) |

The fifth row is the model's one subtle rule. An opponent knows your hand is
A, B and C; you put one back unseen. Dropping only that card from their copy
of your hand tells them which it was. So an unseen departure clears what
they knew of the whole hand.

**The names**, each read against `engineering-practices.md#names` and
searched for in `mtgsim/src`:

```rust
// types/ids.rs — a set of players. Its width is decision 2's.
pub struct PlayerBits(u64);

// objects/object.rs
pub face_known_to: PlayerBits,   // know which card this is, and have followed it since
pub sighting: Sighting,          // what a seat's copy calls this card; a new one each time
                                 // it moves or players lose track of it (decision 4)
pub struct Sighting(u32);

// oracle/sight.rs
pub fn face_seen_by(game: &GameState, player: PlayerId, id: ObjectId) -> Option<SeenBecause>;
pub fn shown_to_all(game: &GameState, id: ObjectId) -> bool;   // CR 603.2f, 701.9c
pub enum SeenBecause { PublicZone, OwnHand, Controls, Revealed, LookingAt, WhileItLasts, Known }

// the boundary: what a provider is handed (decision 3)
pub struct SeatSight<'a> { game: &'a GameState, sees_for: PlayerBits }   // both private
impl SeatSight<'_> { pub fn whole_game(&self) -> &GameState }            // the referee's door

// what a client is handed (decisions 4 and 5)
pub struct Observation { pub board: Vec<SeenObject>, pub happened: Vec<SeenRecord>, pub question: SeenQuestion }
pub struct SeenObject { pub name: Sighting, pub zone: Zone, pub face: Option<SeenFace>, pub because: Option<SeenBecause> }
// SeenFace: the card, both faces of a double-faced one, and the lasting changes this seat knows of

// the performers and their records
GameAction::Reveal { cards, by }        GameEvent::Revealed { cards, by }
GameAction::LookAt { cards, player }    GameEvent::LookedAt { cards, player }

// events/event.rs — who saw the card a record names, as it was written
EventStamp { card_seen_by: PlayerBits, .. }
```

`PlayerSet` is taken, by an effect's scope (`types/effects.rs:671`). The dev
GUI's `SeatView` (`devgui/src/view_model.rs:970`) is its drawing of a seat
and keeps its name. `fuzz_games`' `ObservationRow` becomes one flat encoding
of `Observation`. A seat's copy lists another player's hand as a count, plus
a `SeenObject` for each card of it the seat's players know.

## The binding table

| State | Bound | Live or latched | Stored | Read by | Ends |
|---|---|---|---|---|---|
| who knows a card's face and has followed it (`face_known_to`) | when it is revealed, looked at, or moves where some can follow | latched | the object's record (decision 2) | a seat's copy, the redeal, CR 406.3's continued look | its pile is shuffled or reordered; its own unseen move, or another card's from the same hand; it arrives face up in a public zone |
| who knows of a change that follows a card (Alchemy's "perpetually") | as the change is made: those who can tell which card it was made to; then anyone the card is shown to | latched | on the change, with the card (`card-and-object-model`'s, decision 8) | a seat's copy of a card it knows | never: the change lasts the game |
| a reveal or look in progress (CR 701.20a, e) | as the instruction runs, as a cost reveals, as a card's ability is activated from a hand (602.2a), or as a reveal causes a trigger | latched | the resolving record; the stack entry, for a cost's and an activation's; the waiting or stacked trigger the reveal caused | the sight query, the move rule, CR 603.2f, 603.10a, 701.9c | the resolution ends; the spell or ability leaves the stack; the trigger leaves the stack, or is not put on it by the next priority |
| sight from an effect that lasts (Telepathy, Courser of Kruphix, Found Footage, "cards exiled with") | never: read off the effect each time | live | nowhere; what it shows is written to `face_known_to` at each state-based check | the sight query | with the effect, by CR 611's own rules; what was written stays |
| a cast in progress (CR 601.2a to 601.2h) | — | — | nothing new: the card's record is untouched until 601.2i announces the move | — | a rewind (CR 732.1) leaves no mark |
| that a choice is secret | at the choose step, off the card's word | latched | on the kept choice | a seat's copy of the question, the record and the kept list | the step, the guess or the cost that reveals it |
| that an exiled card is face down, and a face-down object's real face | `card-and-object-model`'s (decision 8) | | the object's, in one place | the sight query alone | CR 708.9, 406.3 |
| who saw the card a record names | as the record is written | latched | the record's stamp, off `GameState` | a seat's copy of the stream | never; the stream is the referee's |
| the players a seat sees for | when the seat is asked: its own, one it controls (CR 722.4), a teammate (808.5, 810.5) | live | nowhere | `SeatSight` | |

## Decisions

Numbered from 1. Topics the brief listed that are not choices are under
"Settled by the rules", after decision 11.

### Decision 1 — what the engine remembers a player was shown

**Retrofit cost.** Low while nothing reads it: the record's meaning is fixed
by the move table and the seat's copy. After the dev GUI and an AI's
observation are built on it, a change moves what every seat's copy holds and
every redeal.

**The problem.** A reveal lasts a moment (CR 701.20a), but a player goes on
knowing the card. A client draws that, and a search may use exactly that and
no more (`backlog.md` "2.34": the redeal "must keep every known card in
place"). The record must fork with the game, so it is on `GameState`, and
floor 3 says it grows with the board, never with the turn count
(`plans/references/ai-performance-floors.md`).

- **A. Cards a player can point at.** One set of players on each card: those
  who know its face and have followed it since, by the table above. *Code*:
  the move rule where a move is announced, and the clears. *Performance*: a
  few bit operations a move. *Upkeep*: one rule; a new verb that moves a
  card says only whether it reveals. *What it loses*, each a case where a
  player knows a set and not a place:
  - a hand after an unseen departure (41 cards make one): a player who knew
    three cards knows two of the three remain, and the engine forgets all
    three;
  - cards put on the bottom "in a random order" (286 cards, 2 in the decks):
    the player who looked knows which cards are down there, the record says
    nobody, and their own redeal may deal one back to the top;
  - cards another player ordered out of sight (136 cards).

  A client still has each in its own record of what it was shown, and may
  draw them as "seen earlier".
- **B. Cards a player has seen in each hidden zone, with what has left
  unseen.** A list for each viewer of each hand and library, and a count of
  unseen departures since each card was seen. Exact for all three losses.
  *Code*: the lists, their upkeep on every departure, and a seat's copy that
  draws "one of these has gone". *Performance*: 32 lists at four seats, each
  one that is not empty an allocation a clone, against floor 2's 64 and
  today's 39. *Upkeep*: a second kind of knowing ("possibly here") in every
  reader, and a redeal that samples under "two of these three".
- **C. Nothing stored: sight alone**, and each client keeps its own memory.
  No redeal can then be built from a fork, which the floors' survey refused
  (2026-09-25).

- **D. A snapshot kept with each player** (the owner's, design review
  round 1, for the Hullbreaker Horror board): each player holds a copy of
  each card as it was shown to them. A change made out of their sight never
  reaches their copy, so that board is right by construction. *Code*: a list
  on each player of (card, what was shown), written at each reveal and each
  followed move, and dropped by the same clears as A. *Performance*: a list
  a player, so up to one allocation a seat a clone; under Telepathy, a new
  snapshot of every card of the hand each time one changes. *Upkeep*: a
  snapshot goes stale against changes everyone can see. An effect that
  makes every card in a hand cost less is public, a player who knows the
  Horror is there knows its new cost, and the snapshot still shows the old
  one, unless each change says whether it was seen. That is A's rule again,
  kept beside a second copy of the card.

**Recommended: A, with each lasting change carrying who knows of it.** A
never shows more than a player knows, which is the property a seat's copy
and a search both need, and what it forgets a client can keep for itself.
B's exactness is a belief model, and where those stop is a research
question the engine should not own.

**The Hullbreaker Horror board, under A.** The card's record says who knows
the Horror is in that hand. The Feaster's change has a record of its own:
the hand's owner alone, since nobody else can tell which card was picked.
An opponent's copy shows the Horror at its printed cost. When the Horror is
cast, the spell is shown to all, every change on it with it, and everyone
sees the discount then. A change everyone can attribute (Blooming
Cactusfolk's "nonland cards in your hand", Lumbering Lightshield's revealed
card) is known to everyone who knows the card. Arena draws the Feaster's
change on the revealed card, which tells the opponent which card was picked;
this does not. Three of the 244 cards' shapes need care in the build and
are counted above: a pick at random (7), a pick its controller makes out of
sight (34), and a change to a card in a library (40).

**Two parts of A that change what a client sees**, each with its other way:

- **A lasting effect's sight is written at each state-based check.** While
  Telepathy stays, the query reads it live. At each check the engine writes
  onto `face_known_to` what such effects show now, reading first and writing
  only what is missing. So when Telepathy leaves, its controller still knows
  the hand, as at a table, and CR 406.3's "may continue to look" holds for a
  look gained at any time. *The other way*: write nothing, and a seat's copy
  drops the hand the moment the enchantment leaves. *Cost*: one more set of
  sources for the check to walk (`lasting-sight`), nothing on a board with
  no such permanent.
- **A cast taken back shows nothing.** The record is written where a move is
  announced, so a spell is seen at CR 601.2i, and one rewound before it
  (CR 732.1) never was: `fuzz_games` counts those as "Actions reversed".
  *The other way*: an announced spell is seen at 601.2a, which is what a
  table sees, and makes every agent's failed attempt show a card; it would
  change the `// CAST-ROLLBACK:` tag's rule that a rewind is unobservable.

### Decision 2 — where the record is stored

**Retrofit cost.** Small: two fields and their handful of writers, behind
the query. No game, log or save moves, since the referee's records do not hold
them.

**The problem.** Floors 2 and 3 price every fork
(`engineering-practices.md#the-floors`): 64 allocations and 128 KB a clone,
read at every decision by `clone_bound_test`. The latest reading is 39 and
8.7 KB. The object store is shared between forks in chunks of 32
(`clone-architecture.md`, decision 1), so a clone copies a pointer a chunk,
whatever an object holds.

| | A clone, 4 seats · 2 seats | A shuffle | A fork's write |
|---|---|---|---|
| **A. On `GameObject`**: `face_known_to` and `sighting` | +0 allocations, +0 bytes · the same | reads each card of the library, writes the few with a bit set: about 0.1 µs for 99 cards, derived from the store's read cost | copies the card's chunk, as any move of it does: 1.5 KB today, 1.8 KB at 56 bytes an object |
| **B. A's set with an epoch beside it**, each hidden zone counting its shuffles | +0 allocations, +0 bytes; 8 bytes more an object in the shared chunks | one write | the same, and a compare a query |
| **C. A list on each hidden zone** of (card, players) | +1 allocation for each list that is not empty: up to 8 · 4 | clears a list | the list's own copy |
| **D. A table of its own**, keyed by id, shared in chunks | +1 allocation once any card is marked | reads the library | a small chunk of its own |

`backlog.md` "2.9"'s input was B: "stamp knowledge facts with an epoch… which
spares a sweep over each fact". The sweep it spares is a read of cards the
shuffle is already reordering, and only a card with a bit set is written.

**Recommended: A.** Zero allocations and zero bytes a clone, no second
structure to keep in step with the card, and a shuffle that costs a tenth of
a microsecond more. B if a library of thousands ever shuffles often, which
is a change behind the same query.

**How wide the set of players is** (the owner, design review round 1: "don't
we want arbitrary N?"). The first draft fitted sixteen seats into the
record's padding, which would have been the engine's first seat cap. The set
is one type everywhere it is used (the card, a record's stamp, a seat's
players), so its width is one line:

| | Seats | `GameObject` | A clone | Past the cap |
|---|---|---|---|---|
| (a) 16 bits | 16 | 48 bytes, as today | +0, +0 | setup refuses the game |
| (b) 64 bits | 64 | 56 bytes: 3.2 KB more in the shared store at 400 objects, once a game | +0, +0 | setup refuses the game |
| (c) 64 bits inline, a heap block past them | any | 64 bytes | +0, +0 up to 64 seats; past that, an allocation for each marked card of a chunk a fork copies | nothing is refused |

Sizes are derived from the measured layout (44 bytes of fields today).
*Recommended: (b).* No clone pays for it, sixty-four seats is past anything
the floors or either GUI is sized for, and (c) is the same type's insides if
a game ever needs it. (a) saved 8 bytes an object that no fork copies.

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
- **B. A seat's sight, read on demand.** `SeatSight<'_>` is the game and the
  players the seat sees for, both private to its module. Its methods are the
  only way through: the stack, the turn, a hand it may see, a permanent's
  effective characteristics, a card's face if sight or the record allows. A
  provider that never looks pays nothing. The referee's instruments (a fork's
  clone, the scenario writer, the dev GUI's tester, the audit) take the
  whole game through one named method, `whole_game()`, which
  `check_invariants.py` allows only under `tests/`, `test_support.rs`, the
  dev GUI and a tagged site. *Code*: the type and its accessors; 51
  signatures swapped by `plans/replace_exact.py`; the 13 public readers
  moved onto accessors; the 8 whole-game readers onto the door.
  *Performance*: nothing a prompt. *Upkeep*: a new thing a seat may read is
  a new accessor, reviewed as an addition.
- **C. An owned copy built at every prompt.** No borrow crosses the
  boundary. *Performance*: k at every decision for every stack, the default
  agents included: 0.125 today, so floor 1's 12,465 reads about 11,080
  (measured, below). *Upkeep*: instruments need a second channel anyway.

**How the 51 move under B** (from the census):

| What it reads | Count | Change |
|---|---:|---|
| nothing, or passes it on | 22 | the parameter's type |
| names of options | 7 | `as_logged` stays the referee's (a script matches typed options without it); a seat's label goes through sight |
| the public board | 13 | accessors every seat has: `stack()`, `turn_number()`, `phase()`, `pending_triggers()`, a permanent's effective abilities |
| its own hand | 1 | `fuzz_games`' `Observer`: the first consumer |
| the whole game | 8 | `whole_game()` |

**Three things B fixes with it.**

- **A sight is a seat's, and a seat may see for several players.** Its own;
  a player it controls, whose eyes it has too (CR 722.4), built with
  Mindslaver's family (`backlog.md` "2.51"); a teammate, in the variants
  that allow it (CR 808.5, 810.5), built in `multiplayer-rules`. The set is
  in the type from the first PR, so neither is a signature change.
- **No accessor is added for one agent.** The random agent's mana preference
  is the one public read that is its own (`ui/random.rs:70`, over
  `oracle::mana_supply`). It is rebuilt in `ui/random` over a permanent's
  effective abilities, which every seat has.
- **A learner reads the copy, not the sight.** An accessor takes and returns
  engine ids, so an agent in process that keeps ids across prompts can
  follow a card decision 4 says it could not. The engine's own agents do not
  learn. The v1 AI client is handed `Observation`, in process or out of it,
  and pays k for it; `codebase-state.md` item 141's "in-process binding" is
  the copy written into the harness's buffer.

**Recommended: B.** It is the only one that fails closed and costs nothing
at a prompt. It leaves `setup-architecture.md#7e-pausing-at-a-decision`'s
decision 2 as taken: a paused game hands a client the owned copy built from
the same accessors, "on the game's stack at the question".

### Decision 4 — what a seat's copy names a card by

**Retrofit cost.** High once a client exists: every object a client is
handed and every answer it sends is named this way. Nothing changes inside
the engine's rules either way.

**The problem.** An id follows a card through every zone. A creature is
bounced, so every player knows that card is in its owner's hand. The owner
casts a face-down spell. If the spell carries the id the creature had, every
client knows what it is; at a table no one would. The same id links a
creature put on top of its library to the card drawn after a shuffle.

**What a name must do.** A seat may connect two sightings of a card only if
it could point at the card the whole time between.

- **A. The engine's id everywhere**, as today. Leaks every face-down object
  whose card a seat has met, and makes a policy that tracks ids stronger
  than one that plays the game.
- **B. A derived id that is cleared at every information boundary** (the
  owner's words for it, design review round 1). Each object carries a
  `sighting`, a number from one counter of the game's. It gets a new one
  whenever a seat could lose track of it: each time it changes zones, each
  time its record is cleared (a shuffle, a reorder out of sight, an unseen
  departure), and once for every card at the opening shuffle, so that no
  name holds a deck slot. Both writes ride a write that is already
  happening. A seat's copy names objects by sighting and nothing else. Two
  rules make it hold:
  1. A copy names a card in a hidden zone only for a seat that is on its
     record or sees it now. Any other is a count.
  2. A record of a boundary gives a seat the old sighting beside the new one
     only when the boundary leaves that seat on the card's record. So a
     client still animates one card from a hand to the battlefield, and a
     card bounced, shuffled away and drawn again arrives under a name no
     seat can connect.

  An answer names an option as the copy named it, and the engine finds its
  place among the options it offered. `zone_change_epoch` stays what it is,
  the rules' CR 400.7 existence; a sighting is a client's name and no rule
  reads it.
- **C. The id for a face-up object, a made-up handle for a face-down one.**
  Closes the morph and leaves the library link; two naming rules in every
  client.

**Recommended: B.** One rule, which is CR 400.7's and 701.20d's own (a new
object when it moves, and when it is reordered out of sight). `backlog.md`
"2.36" (how an object is named in text) keeps its slot; this decides the
machine name only. It is what makes decision 3's B hold against a client
that is trying to cheat: a seat's copy holds no number that says anything
about a card the seat was not shown.

### Decision 5 — events and answers, as a seat sees them

**Retrofit cost.** Medium: a stamp on each record and one function. The
referee's stream, the trace and every fixture table are unchanged under A.

**The problem.** `CardDrawn` names the card, and so does a `ZoneChange` from
a library to a hand; the log line every seat reads is "P1 drew Counterspell".
Who may see a card must be answered as of the event, since by the time a
client reads the record the card has moved on. The stream left `GameState`
with `codebase-state.md` item 42, so a per-viewer history costs no floor
wherever it is kept.

- **A. One stream, each record stamped with who saw the card it names**, as
  it is written (`card_seen_by`, the same set of players). A seat's copy of a
  record is built by one function: the record as it is when one of the
  seat's players is in the stamp, and with the card left out ("P1 drew a
  card") when not. An answer another seat gave goes through the same
  function option by option, and a secret one (decision 7) is held back
  whole until its reveal. *Upkeep*: a new record kind that names a card says
  which of its objects the stamp is about.
- **B. A stream per seat, written as the event happens.** N copies of every
  record; each sink decides at once and keeps no stamp. A fork's recorder
  branches N streams.
- **C. A second event vocabulary for clients**, written out by hand beside
  `GameEvent`. Every variant is spelled twice and kept in step.

**Recommended: A.** One writer, one projection, and the referee's stream stays
the trace's and the fixture tables'. The wire form of a record is Phase 10's
(`backlog.md` "2.38"), built from A's projection.

**What the referee keeps whole**: the decision log (every deck, the seed,
every answer), the trace, the scenario writer. A seat is never handed one. A
save a player keeps of their own game is the referee's record, so it holds
every hand: where v1 keeps it is `backlog.md` "2.38"'s open question 5.

### Decision 6 — the redeal for a forked search

**Retrofit cost.** Low: one function over the record, with no client before
the AI harness.

**The problem.** A fork holds the true game. A search over it knows its next
draw (`backlog.md` "2.34"). Before each branch the fork is rebuilt from what
the searcher knows: every card the searcher cannot place is shuffled among
the places it could be.

- **A. The engine's.** `GameState::redeal_as_seen_by(player, seed)`, on a
  fork: the searcher's own library, among the slots it does not know; each
  other player's hand and library together; a face-down object's card with a
  hidden card of its owner that could have been put there the same way (any
  card for a manifest, a card with morph or disguise for a face-down cast).
  One state a branch, every seat's hidden cards sampled from the searcher's
  view, as "2.34" settled. Two things are part of it:
  - **The caller gives the seed.** Every fork of one decision carries the
    same `rng`, so drawing on it would deal every branch alike. A branch is
    then replayed from its seed.
  - **Its writes are no event of the game**, so they are outside
    `perform_action`, as a cast's rollback is. They take a tag of their own,
    `// REDEAL:`, which `check_invariants.py` allows in that one function.
- **B. The harness's**, over a public list of what is pinned. Each harness
  writes the exchange rules again, and the face-down rule is a rules
  question.

**Recommended: A**, built after v0.5 beside the AI harness, its only
client (see "The build, sized"). Its test is "2.34"'s: a searcher over many
samples predicts its next draw no better than chance.

### Decision 7 — a choice kept from the other players

**Retrofit cost.** Low now; `several-choosers-at-one-step` is the next row
and writes whichever shape is taken.

**The problem.** 17 cards choose "secretly", six of them votes, and the
reveal comes three ways: in the same step once every chooser has chosen
(12, Call to the Void and the six secret votes), after another player's
guess (The Toymaker's Trap), and as a later cost that may never be paid (4,
Emissary of Grudges). The other 33 votes are open, and their ruling says so:
later voters know earlier votes.

- **A. A mark on the choose step, and the engine asks in turn as it does
  now.** The step says its choices are secret; a vote can carry the mark as
  any choice can. Each answer is kept as any choice is. Until it is
  revealed, no other seat's copy holds it: not the question's words, not the
  record of the answer, not the kept list. The reveal is a step of its own
  ("then those choices are revealed"), or for the four kept on a permanent
  the cost that names it ("Reveal the player you chose:"). Asking in turn
  then shows nothing that asking at once would hide.
- **B. Ask every chooser at once**: a new boundary shape, several seats
  pending together. It is what the card literally says, and with A's
  withholding it changes nothing a player can observe but how long they
  wait.

**Recommended: A.** What `several-choosers-at-one-step` records is the mark,
on a vote too, and the reveal step. The Toymaker's Trap's reveal is the same
step after its guess. A choice kept on a permanent in secret is
`linked-abilities`' kept value carrying the same mark.

### Decision 8 — what a card hides in plain sight: what this asks of `card-and-object-model`

**Retrofit cost.** High if left to chance: every reader of an object's
printed card is a possible leak, and there are name helpers that read it
today for any id.

**The problem, from the table.** You cast a morph creature face down. Every
player sees a 2/2 with no name. You may pick it up and look at it; nobody
else may; when it dies everyone is shown what it was (CR 708.2, 708.5,
708.9). The engine has to keep the real card somewhere while every rule
treats the object as the 2/2. 478 printed cards put a card face down
somewhere, three of them in the decks' lists: Reality Shift, and Mosswort
Bridge and Windbrisk Heights, whose hideaway exiles one.

- **A. The object still points at its real card, and a flag says it is face
  down.** That is what `PermanentState.face_down` implies today. Then every
  piece of code that reads the card, to print its name, list its abilities
  or draw it, must remember to check the flag first. One that forgets shows
  the real card. `check_invariants.py`'s printed-read rule would gain a
  face-down leg to catch them.
- **B. The object points at what the rules say it is, and the real card is
  put away.** While it is face down, the object's card is the 2/2 with no
  name. The real card is held in a second place that only the sight query
  and CR 708.9's reveal open. Code that forgets to check shows the 2/2,
  which is the right answer.

**Recommended: B as a requirement**, with where the real card is held left
to that design. This note fixes six things for it:

- **face down is a status, apart from which face a double-faced card has
  up.** A double-faced card manifested, cast or exiled face down shows
  neither face: it has the face-down characteristics, cannot transform, and
  turns up front face first (CR 712.15, 712.15a, 712.17). A double-faced
  permanent cannot be turned face down at all (712.16). So "which face is
  up" and "face down" are two facts on the object, and a seat that may look
  is shown the card, which is both faces. The corpus has 388 transforming
  and 98 modal double-faced cards, and any of them can be manifested;
- **a change that follows the card carries who knows of it** (decision 1's
  Hullbreaker Horror board). Perpetual changes are that design's to store;
  this asks that each one holds a set of players beside it, and that a seat's
  copy of a card is the card with the changes that seat knows. The same
  goes for a card brought in by conjure, which is a card like any other
  once it is in a zone;

- the look is the sight query's: the controller by CR 708.5, a lasting grant
  such as Found Footage's, and the record;
- a card exiled face down carries the same record, which is CR 406.3's "may
  continue to look";
- the query answers CR 406.3b and 601.3f ("only if they can look at the
  face-down card");
- **that an exiled card is face down is the object's, in one place.**
  `linked-abilities` lists the cards exiled with a permanent and holds no
  flag of its own, whichever row first exiles a card face down.

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
reading past a seat's sight is `SeatSight`'s private fields, and privacy is
a module's: it holds wherever the other files sit. So a move's payoff is a
name and a smaller folder, not a hazard.

- **A. Three modules now**: `decision/` (asking, the kinds, the trait,
  `SeatSight`), `policies/` (the random agent and the decorators), `words/`
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

The owner put five things on this design (2026-11-04 to 2026-11-14), and a
sixth that is a choice.

| Parked | Disposition |
|---|---|
| The structure a client is handed | Taken: `Observation`, in "The model". Three parts: the board through sight (decision 3), what happened since (decision 5), and the question, each object named by decision 4. Its wire form stays Phase 10's |
| One table per `DecisionKind`: its name, log word, scenario word and fields | Taken in the build's `a-seats-copy` PR, generated from the code, since the copy's question needs each kind's fields listed |
| A setup line's word for a division, and `ability N` | Re-slotted to that table's PR: both are words of the scenario form, which the table lists and nothing here shapes |
| What an attack declaration answers with | Re-slotted to `combat-requirements`, which reshapes the declaration (`roadmap-v2.md`, step 11) |
| The error a client is handed | Stays with `plans/handoffs/process-pr.md`, which makes a branched-on error a typed enum. What this adds: an error that names a card goes through decision 5's function like a record |

**The choice: do an agent's answers name options instead of places?**
`setup-architecture.md#7e-pausing-at-a-decision`'s decision 2b took identity
for a client's answers and left a provider in process answering by place.
"2.38" asked to take the place out of the trait too.

**Retrofit cost.** Every implementation's return values, and every test
that asserts a place. Counted: 51 implementations, and the three methods
that return places are called at 168 sites, many of them a test asserting a
place. "2.38"'s "about 100 providers" is not reproduced; a search for
`fn pick_n` that also catches `fn pick_number` reads 104.

- **A. With decision 3's sweep, by reference.** `pick_n` returns
  `Vec<&DecisionOption>` out of the slice it was handed, `allocate` each
  bucket with its amount, `choose_ordering` the items in order. A provider
  can only return what it was offered; no clone, no index out of range, and
  `allocate`'s parallel list is gone. It adds about 350–600 touched lines to
  `seat-sight`, which already opens every one of those files.
- **B. Leave places in the trait**, as 2b took. `seat-sight` stays at its
  smaller size.
- **C. A PR of its own, after `seat-sight`.** The same lines, and a second
  pass through the same files.

**Recommended: A.** The files are open once. "The build, sized" gives
`seat-sight` both ways.

### Decision 11 — a draw from the random stream, and who is told

**Retrofit cost.** Low: the draw sites are few (the shuffle, a discard at
random, a player at random since #330), and a record kind is an addition.

**The problem.** Two things, both `backlog.md` "2.9"'s input ("name each
random draw by its kind") and item 432's other half. A pick at random asks
nobody, so nothing is logged: a client learns that Territorial Hellkite
chose a player from what the next step does. And a replay or
`loop-detection` reading the stream cannot say what a draw was for.

- **A. Every draw goes through one function that takes its kind** (a
  shuffle, a card at random, a player at random, an order, a coin, a die,
  the redeal), and a draw whose result the players see writes a record of
  it: the player chosen, the coin's face, the die's number. A shuffle's and
  an order's results stay unseen, as today. *Code*: the function, the kind,
  one record kind; about 60–100 lines in `a-seats-copy`, whose stream it
  joins.
- **B. The kinds without the record.** A replay can read the stream; a
  client still infers a pick.
- **C. Neither**, until dice and coins are built (`backlog.md` "2.31").

**Recommended: A.** A pick the players watched is something that happened,
and the stream is where a seat reads what happened.

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
  read off the link while it lasts, and written to the card's record at each
  state-based check as any lasting sight is.
- **Card names over hidden zones** (`values-kept-past-the-resolution`). The
  list offered is the registry's names, never this game's cards
  (`choices-architecture.md`, decision 7). The registry a game is built from
  is public and a decklist is not: a seat's copy of the game's start holds
  its own deck alone. The engine reads the hand for "all cards with that
  name" as it reads anything; what the row records is the reveal, a step of
  its own, which is what puts those cards on every player's record.
- **The rules' readers.** `visible_to_all` (`engine/triggers/dispatch.rs:248`)
  becomes `shown_to_all`: the zone, a face-down status, a reveal in
  progress, a lasting reveal. That is CR 603.2f as
  `triggers-architecture.md` S1 left it. CR 701.9c reads the same question
  of the discard's move (item 130). CR 401.6's "becomes a new object" is
  `lasting-sight`'s.

## What each row before step 8 records

| Row | Records | So that the model |
|---|---|---|
| `several-choosers-at-one-step` | on a choose step, a vote included: that its choices are secret, and the step that reveals them (decision 7). An open vote records nothing | withholds a secret answer from every other seat's copy until the reveal |
| `values-kept-past-the-resolution` | a reveal as its own step before a filter reads a hand or a library; names offered from the registry | puts the revealed cards on every player's record, and offers no list that shows a deck |
| `card-and-object-model`, the design | where the real face is held, behind sight; that an exiled card is face down, in one place; face down apart from which face is up; and on each change that follows a card, who knows of it (decision 8) | answers CR 708.5, 708.9, 406.3 and 406.3b from one query, and draws a known card with only the changes its viewer knows |
| `linked-abilities` | the cards exiled with a permanent, as a list of cards and nothing more; a secret kept value's mark | reads a lasting look off the link |
| `resolution-recipients-and-amounts` | nothing new: a target or "that card" by existence, as its note's decision 1 already has it | never follows a card into a hand or a library |

## The costs, and a ceiling for each

Every recorded reading was taken again before a ceiling was set on it. The
probe was a throwaway crate outside the tree on `clone_bound_test`'s
Commander games (`performance`, seeds 12345 to 12349), release, one thread,
medians of five means.

| Cost | On record | Read 2026-11-15 | Ceiling, and what it admits and refuses |
|---|---|---|---|
| **The observation's board, k** (`fuzz_games --observe`, 20 games, seed 12345) | 0.106 (7.7 µs against 66.3), then 0.100 at readiness | **0.125**: 6.7 µs an observation, 69 rows, against 53.4 µs of engine a decision | **`Observation`'s board costs at most 0.025 more than the naive observation in the same sitting, and floor 1 over (1 + k) stays above its limit.** Admits the query on each row (0.003, measured) and a name and a reason a row (about 0.014, derived at 10 ns). Refuses a second layer read a row (0.125). At the latest floor 1 reading, 12,465, the limit is k under 0.246 |
| **The observation's other two parts** | none | none: the naive probe builds the board alone | **No ceiling yet.** What happened since and the question are read at `a-seats-copy`'s close-out, and a ceiling is set on that reading |
| **The record, a clone** | "bits on each object stay under 1 KB with no allocation" | `GameObject` is 48 bytes, 4 of them padding | **Zero allocations and zero bytes a clone** at every decision, on `clone_bound_test`. A reveal in progress may add one allocation at a prompt inside it, and `lasting-sight`'s set of sources one on a board that holds such a permanent |
| **The redeal, a fork** | 1.7 to 6.1 µs, "about one clone again" | **0.8 to 7.9 µs**, against a clone of 0.7 to 1.9; with a lookup a card, the same within 0.5 | **A clone and its redeal together stay inside floor 2's 10 µs** at every checkpoint. Today 1.7 to 8.8, worst on turn 1, when 40 cards cross between hands and libraries and their chunks are copied |
| **The memo after a redeal** | cold costs "one board walk more, 10–40 µs from mid-game" | one walk of what a seat's copy lists, cold less warm: **11 to 62 µs** from turn 20, 1.5 on turn 1 | **A redeal leaves the memo warm unless a registry row reaches a zone it deals across**, one compare on `RegistryScopeSummary::reachable_zones`, held by a test that counts layer frames with and without a redeal. Cold, it is one walk, which this design does not bound and `layers-architecture.md`'s LL does |
| **The query, a decision** | "the visibility query's instructions per decision", unmeasured | 2.2 ns an object; 167 ns a decision over the 75 objects a copy lists | **At most 5 ns an object on a board with no lasting sight effect.** Admits the measured 2.2 and one count check. Refuses a layer read (97 ns a row) or a registry scan an object |

**What moved since the record.** The clone fell to a fifth with CL-1, so the
redeal is now about five clones and no longer one; the pair still fits the
floor. k rose because the engine got faster and the pool grew: the
observation itself is 6.7 µs against 7.7.

**Who pays k.** Under decision 3's B a provider that never looks pays
nothing, so floor 1's own reading, on the default agents, does not move with
this build. A harness that hands a learner `Observation` at every decision
pays it.

## The build, sized

A count set at a design stop is read with its top as the expected size
(`references/sizing-calibration.md`). Code and tests apart; cards and docs
are beside the band.

| PR | Shape | Consumer | Code | Tests |
|---|---|---|---:|---:|
| **`seat-sight` — the query, and what a provider is handed** | `PlayerBits`; the sight query for the zone defaults and a face-down permanent; `SeatSight`, its accessors and the players it sees for; the 51 implementations onto it; `whole_game()` and its gate | `fuzz_games`' `Observer`, the random agent, every decorator; atoms 400.2, 401.2, 401.3, 402.3-001, 404.2, 406.3-001 | 600–950; with decision 10's answers 800–1,300 | 400–650; with them 550–900 |
| **`shown-and-followed` — the record, reveal and look** | `face_known_to` and the move rule, written where a move is announced; reveal and look as actions with their records; a reveal in progress and its three holders; CR 603.2f and 701.9c reading the query (item 130) | Brainstorm, registered with its rulings, for the cleared hand; Nephalia Academy's reveal; God-Eternal Kefnet's fixture gains its reveal and its trigger's hold; atoms 401.4, 402.3-002, 402.3-003, 701.20a, and 701.9c's revealed half | 600–900 | 550–850 |
| **`search-and-library-walks` — search, and the top of a library looked through** | CR 701.23's search with its prompt; "look at the top N, put one here and the rest there"; revealing until a card matches (`backlog.md` "2.46") | Cultivate and Ponder in this PR; the gap census has 39 deck cards needing the model and 20 of them needing nothing else, registered after it; atoms 701.23a, b, d | 550–850 | 450–700 |
| **`a-seats-copy` — what a client is handed** | `Observation`: the board through sight, what happened since with each record as the seat sees it, the question; decision 4's names; a secret answer withheld; a random pick's record and the draw's kinds (decision 11); the words taking a sight; the kinds table | `fuzz_games --observe` builds it, which is k's reading; a test that no seat's copy of a seeded game holds a card its players were not shown | 600–950 | 450–700 |
| **`devgui-one-seat`** | the window drawn from one seat's `Observation`, with the tester's whole game as a switch | a person at the dev GUI; its own review path | 300–600 | |
| **`lasting-sight` — a hand or a top card shown while an effect lasts** | sight read off an effect, and written at each state-based check; CR 401.5's wait while casting and 401.6's new object | Courser of Kruphix, Telepathy; Thundermane Dragon's look, the one deck card; atoms 402.3-004, 401.5-001 to 003 | 350–550 | 300–450 |
| **`redeal-for-a-seat`**, after v0.5 with the AI harness | decision 6, and the memo's test | the harness's search; "no better than chance" | 250–400 | 250–400 |

**Five route PRs at step 8, the dev GUI's beside them, and one after v0.5.**
The first four read 1,000–1,600 (1,350–2,200 with decision 10's answers),
1,150–1,750, 1,000–1,550 and 1,050–1,650, each under the band's top;
`lasting-sight` is under its floor and may ride `search-and-library-walks`
if that lands small. `roadmap-v2.md` carried 3–6 before any design.

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
| ATOM-401.4-001 | two cards put on top in their owner's order: the owner on both records, a player who knew them on neither | `shown-and-followed` |
| ATOM-402.3-002, ATOM-701.20a-001 | a three-seat reveal of a hand, seen by both others while the effect runs, and on their record after | `shown-and-followed` |
| ATOM-402.3-003 | a three-seat look: the looker sees, the third seat does not | `shown-and-followed` |
| ATOM-701.9c-001 | a discard put on top of a library unrevealed is shown to no one; the same revealed is shown to all | `shown-and-followed`, partly: the cost that becomes illegal waits for `remaining-costs`' `Cost::Discard` |
| ATOM-701.23a-001, b-001, d-001 | a search that sees the whole library, may fail to find a card with a stated quality, and must find "a card" | `search-and-library-walks` |
| ATOM-402.3-004 | a hand revealed while an enchantment stays, and known after it leaves | `lasting-sight`. The atom expects the hand hidden again at once; under decision 1 sight ends and the record stays, and the test states both |
| ATOM-401.5-001, 002, 003 | a top card revealed; a new one not shown until a cast is finished | `lasting-sight` |
| ATOM-406.3-002, ATOM-601.3f-001, 002 | a card exiled face down, seen by none, then by a player granted the look, who alone may cast it | not this build's: `face-down-and-morph`, on `card-and-object-model`'s mark, since nothing exiles face down before it. The query is their reader |
| ATOM-722.4-001 | a controlling player sees the controlled player's hand | not this build's: Mindslaver's family, after v0.5 (`backlog.md` "2.51"). `SeatSight`'s set of players is where it lands |
| CR 708's eight, BOUNDARY-708.12-001 | | `face-down-and-morph` |

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
   leaves it the referee's and gives a seat a label through sight.
2. **Floor 1's room for k halved.** `backlog.md` "2.9" has k failing floor 1
   above 0.56, against 15,600; at the latest reading it is 0.246.
3. **The dev GUI's `Snapshot::build` writes the whole board as scenario text
   at every prompt**, every library in order. It is the tester's, and
   `devgui-one-seat` keeps it behind the switch.
4. **The rules' existence tick cannot be a client's name.** An object that
   has never moved has none: a token as it is made, and an ability on the
   stack, whose reference carries its source's
   (`engine/put_on_stack.rs:655`). And an id handed out as a deck loads is a
   deck slot, the same in every game with that list. So decision 4 gives
   each object a number of its own, first minted at the opening shuffle.
5. **`check_invariants.py`'s printed-read rule does not see `ui/`'s name
   helpers** as a seat's leak, since each is tagged for a record no rule
   reads. The words taking a `SeatSight` is what closes it.
6. **ATOM-402.3-004 folds sight and memory together**: it expects a hand
   "hidden again" when Telepathy leaves. The CR ends the reveal; it does not
   say the players forget. The atom is not amended; its test states the two
   apart.
