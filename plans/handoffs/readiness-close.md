# Design note — readiness, part 2: item 356, the gather by kind

**Opened 2026-10-30** on `audit/readiness-close`, the item 6 audit's last PR
(roadmap row A6e, readiness; `plans/handoffs/item-6-audit.md`, "Pass 4 —
readiness"). Its brief planned no design stop, but said to stop here if item
356 needed a new type, and it does: a set of the kinds of `GameAction` a
replacement pattern watches. This note is that stop. It goes when the build
lands, its decisions moved to `replacement-architecture.md`'s "3.3 Where
replacement effects come from".

## Where this sits on the event path

Every proposed event goes through `execute_actions`, and before anything
performs, `apply_replacements` runs CR 616.1's loop. Each pass of the loop asks
`gather` (`engine/replacement/gather.rs`) which replacement effects apply to the
proposal. The gather reads five sources (`replacement-architecture.md#33-where-replacement-effects-come-from`);
source 1, the static abilities of permanents, is a walk of the whole
battlefield in timestamp order, asking each permanent whether it printed a
replacement ability (`GameState::replacement_ability_sources`) and whether a
counter on it makes one (source 5, CR 122.1c/d/h). That walk, on every
proposal, is item 356.

- **Item 356** (`codebase-state.md`): the walk visits every permanent when
  only its printed sources can answer. 30.5% of instructions on the 50-Goblin
  token board (item 354, floor 1 failing there), 17.2% inclusive on the
  Commander board. #299's all-or-nothing probe read −5.0% CPU on the
  Commander board and −11.2% on the token board. The owner, at #299's review:
  **the shape is the trigger dispatcher's, not the probe's switch**, since one
  named grant or one shield counter anywhere put the probe back on the whole
  walk.
- **Item 22** (`codebase-state.md`): the counter half of the same walk,
  `any_replacement_counter`'s scan on every proposal. Sized as a set of the
  permanents holding a replacement counter and slotted to Phase 10, "measured
  before it is paid for"; decision 4 takes it here, and this PR's A/B is that
  measurement.
- The dispatcher's shape (`engine/triggers/dispatch.rs`): each printed source
  is filed in `trigger_sources` under an `EventKindMask`, the kinds of record
  its arms read; `battlefield_readers` asks the file for the window's kinds;
  `named_trigger_carriers` reads the objects a named grant or copy row
  (`ObjectSet::SourceOnly`, `Fixed`, `Host`) puts a triggered ability on,
  when the summary's `named_unattributed_trigger_kinds` meets the window's;
  only a row over a filter walks the zones it names.
  `engine-map.md#23-the-gates-and-the-three-legs-a-new-source-extends` has
  both gates side by side.

## The rest of this PR, built before the stop

None of it waits on this note:

- **Item 352** (the battlefield's ordered lists regrew on every call):
  `ObjectTable`'s walk counts its length down and reports it. `aa48651c`.
- **Item 41's fork test** over item 6's state: each branch's end state is
  compared with the original's, part by part, and a fourth gate forks while a
  delayed trigger or a return waits. 0 of 13,174 branches diverge. `3ad11939`.
- **Item 142, the panic surface**, reconciled: both runs counted alike; the
  script now keeps debug-only code apart. 60 sites in 44,314 engine lines on
  2026-09-15, 71 in 81,519 now. `514fc27a`.
- **k**, `fuzz_games --observe`: a first reading of 0.086 on the Commander
  board, MTG Arena running. `eb7d0d9e`, its rows named at the pre-review
  `0266d68e`.

Owed after the build: the A/B and the budget, the audited smoke, floor 1
re-read on an idle machine, the audit's two records and the handoff deleted.

## What the new type must express

**It counts every pattern, whatever route its effect takes**: a permanent's
static ability, an activated ability's shield, a resolution's, and a "can't"
(`Restriction::Event` reuses `EventPattern`). Under decision 2(a) every route
asks the kinds through `pattern_watches`, so the kinds must hold every
pairing any of them makes.

**Recounted 2026-10-30 at the owner's review**, by
`plans/references/replacement-kinds-census.py` over every Commander-legal
paper card (32,115; `cast-census.py`'s corpus, fetched 2026-10-05). The first
count was a Scryfall query per family that matched a word anywhere in a
card's text. So it counted triggered abilities: the discard row's 17 were 7
replacements and 10 triggers, Ajani's Last Stand among them. And it missed
every family whose wording lacks the word: madness, regeneration, protection,
and every permanent that enters tapped. The recount puts a card in a family
by the clause its replacement watches:
- the event inside a "would … instead" sentence;
- a "causes you to discard … instead" or "tapped for mana … instead" one;
- a prevention, a skip, or an entry modification;
- a keyword whose CR 702 definition is a replacement.

Reminder text is stripped first, so a keyword counts once. "If … instead"
with no event in the "if" (kicker, "if you control a Mountain") is a spell
modifying its own effect, and is not counted. Every card of each family
under 40 was read, and a seeded sample of 10 to 12 from each larger one.

**Static** is the battlefield file's population: the clause is printed on a
permanent, not in a triggered ability, not behind an activation cost, not
inside a quoted grant, and not a one-shot. It is read off the line's shape,
so it is approximate.

| What the kinds must do | Family | Cards | Static | Example | Arm today |
|---|---|---:|---:|---|---|
| **One kind** | damage prevented | 490 | 138 | Callous Giant | `DealDamage` |
| | protection (CR 702.16e) | 193 | 190 | White Knight | `DealDamage` |
| | damage replaced | 142 | 74 | Furnace of Rath | `DealDamage` |
| | dies, or put into a graveyard | 196 | 76 | Rest in Peace | `ZoneChange`, its `to` a graveyard |
| | discard; madness | 7; 61 | 7; 35 | Wilt-Leaf Liege | `ZoneChange`, its act |
| | leaves the battlefield; unearth | 15; 56 | 1; 56 | From the Catacombs | `ZoneChange` |
| | leaves the stack: flashback, jump-start, aftermath, harmonize, rebound, buyback; countered | 338; 1 | 1; 1 | Guile | `ZoneChange` from the stack, its act |
| | would enter | 16 | 12 | Containment Priest | `EnterBattlefield` |
| | others enter tapped | 27 | 24 | Archon of Emeria | `EnterBattlefield` |
| | others enter with counters | 33 | 25 | Bard Class | `EnterBattlefield` (the first count put it with the two-kind rows; an "enters with" rewrite watches only the entry) |
| | draw; dredge | 47; 14 | 35; 10 | Abundance | `DrawCard`, `DrawCards` |
| | gain life / lose life | 21 / 2 | 20 / 2 | Rain of Gore | `GainLife`, `LoseLife` |
| | tokens | 31 | 27 | Anointed Procession | `CreateTokens` |
| | skip, or begin, a turn, phase or step | 44 | 29 | Stasis | `BeginTurn`, `BeginPhase`, `BeginStep` |
| | mana | 16 | 9 | Mana Reflection | `ProduceMana` |
| | regenerate; umbra armor; "would be destroyed" | 270; 15; 3 | 18; 15; 0 | Cudgel Troll | `Destroy` |
| | untap / scry / lose the game | 2 / 2 / 4 | 2 / 2 / 3 | Eligeth, Crossroads Augur / Lich's Mirror | `Untap`, `Scry`, `PlayerLoses` |
| | can't lose / can't win the game | 11 / 9 | 6 / 5 | Platinum Angel / Abyssal Persecutor | `PlayerLoses` / `PlayerWins`, through `Restriction::Event` |
| **Two kinds** | counters put on, a player's too (energy, poison) | 31 | 29 | Hardened Scales | `AddCounters`, and an entry: "Doubling Season affects permanents that enter with counters" (ruling, 2024-11-08) |
| | can't enter | 5 | 5 | Grafdigger's Cage | `ZoneChange` whose `to` admits the battlefield, and an entry |
| **Its own entry only** | enters tapped; enters with counters; as it enters; a keyword (amplify, modular, sunburst, riot and ten more) | 761; 448; 301; 205 | 761; 415; 300; 205 | Blood Crypt | `EnterBattlefield` over `SourceOnly`, read by source 1a as the card enters. **1,587 cards have no other replacement** |
| **No arm, or no `GameAction`, yet** | life total reduced | 9 | 6 | Enduring Angel | none: damage and life loss in one pattern |
| | mana empties (CR 500.4) | 5 | 5 | Kruphix, God of Horizons | none |
| | explore, connive, learn, planeswalk, assemble, copy a spell, proliferate | 2, 1, 1, 1, 1, 1, 1 | all | Topography Tracker | none: a keyword action with no `GameAction` |
| | search / a die roll or coin flip / a mill's number | 1 / 8 / 2 | 1 / 6 / 2 | Aven Mindcensor / Barbarian Class / Bruvac the Grandiloquent | `backlog.md` §2.5 / §2.31 / a line in §2.5 at this PR's close |
| **An arm, no printed card** | becomes tapped | 0 | — | — | `Tap`, kept in the kinds as every arm is |
| **A `GameAction`, no arm, no printed card** | a shuffle, becoming blocked, a creation into a zone, attaching, unattaching, a token ceasing to exist, a draw game | 0 | — | — | none; each has a kind no pattern's kinds hold |
| **Not indexed by kind** | shield / stun / finality counters, cards that put one | 28 / 91 / 40 | | Boon of Safety | source 5, from the counter |
| | becomes a copy | 86 | 10 | Cytoshape | a named copy row (decision 3) |
| | enters as a copy; a token copy | 59; 279 | | Spark Double | filed as printed: an entry copy is the object's own |
| | protection granted to named objects / over a filter | 121 / 28 | | Mother of Runes / Akroma's Will | a named grant row (decision 3) / a `Filter` row's zones |
| | a quoted replacement granted | 21 | | Bewitching Leechcraft / Pulmonic Sliver | a named row / a `Filter` row, as the grant says |

**Reading.** Every printed family that has an arm watches one `GameAction`
kind or two, and every pairing is already one arm of `pattern_watches`: the
kinds add no vocabulary, they index what the arms pair. A zone-change pattern
watches an entry only when its destination admits the battlefield, so the
kinds read the pattern's `to`. Filed by the arm alone, the 196 "dies or put
into a graveyard" cards would be asked at every entry, each Goblin of the
token board's included.

**The largest family is one the first count missed, and it watches nothing
from the battlefield.** 1,587 cards replace only their own entry: every
tapland, every "enters with N counters", every "as this enters, choose". CR
614.12 reads that ability off the card as it enters, which is source 1a, and
a permanent already on the battlefield cannot enter again without becoming a
new object (CR 400.7). So the file leaves the entry out of a `SourceOnly`
def's kinds, and a def left with none is filed nowhere (decision 1). The
trigger side does the same for a trigger that fires only on its own object
moving: it is asked only as a newcomer, never from the zone map
(`newcomer_trigger_sources`).

Twelve families have no arm, and all but "life total reduced" have no
`GameAction` either. Each arrives with a variant, so a proposal's kind must be an exhaustive
match that a new variant cannot compile past. The rulings read for the table
set two more constraints:
- A counter pattern watches an entry (Hardened Scales, Doubling Season).
- A shield or finality counter works on a permanent that lost its abilities
  ("If that permanent loses its abilities and then would go to a graveyard,
  it will still be exiled instead", Balustrade Wurm, 2024-09-20). So the
  counter route may not hang off the ability list.

**What overrides it.** No rule lets a card change which event another effect
watches: CR 614.1 makes that the effect's text. A text-changing effect (CR
612, Layer 3) could rewrite a pattern's words, and Layer 3 is the route every
gate already leaves open (`engine-map.md#23-the-gates-and-the-three-legs-a-new-source-extends`);
no printed text-changing effect changes an event word. So no family
overrides the kinds, and the kinds are read off the printed pattern as the
other gates read the printed list.

## The battlefield leg, as built under any choice below

The candidates are the union of up to three sets: the printed sources filed
under the proposal's kind (decision 1), the objects the named rows put a
replacement ability on (decision 3), and the permanents holding a replacement
counter (decision 4). **Each permanent once, in CR 613.7 order**: the union
goes through `in_battlefield_order`, the key `battlefield_ids_ordered` sorts on,
and duplicates are dropped there, so CR 616.1's prompt offers what today's walk
offers in today's order. A permanent can be in two sets in ordinary play: one
that printed a replacement ability and then becomes a copy of another
replacement source (Cytoshape's shape) is filed as a printed source at its
entry and named by its copy row. (An entry copy, Spark Double's, is applied as
the object's own and filed as printed, so it is in one set.) Each
candidate is asked as the walk asks: its abilities, then its counters. A
`Filter` row reaching the battlefield, and the audit's `Sweep::Everything`,
still walk every permanent.

## The decisions

### 1. The kind index's type

The problem: `replacement_ability_sources` is an `IdSet`, so it says only that
a permanent printed some replacement ability. To ask "could this permanent
answer a damage event?" without reading its frame, the file must say which
kinds of proposal its printed patterns watch.

- **(a) `GameActionKind` and `GameActionKindMask`, the dispatcher's shape.** A
  `counted_enum!` with one variant per `GameAction` variant and
  `GameActionKind::of(&GameAction)`, exhaustive, beside `GameAction` in
  `engine/actions.rs`; a mask built as `EventKindMask` is (`types/triggers.rs`),
  as many words as the kinds need, its empty value spelled `EMPTY` and no
  `Default`; `EventPattern::action_kinds()`, exhaustive over the arms and
  reading `ZoneChange`'s `to`, beside `EventPattern` in
  `types/replacement.rs`. `replacement_ability_sources` becomes
  `IdMap<ObjectId, GameActionKindMask>`, filed by `register_static_effects`
  with `.or_insert(GameActionKindMask::EMPTY)`. About 110 lines. Performance:
  one bit test per source per gather in place of a frame read for every
  source. Upkeep: a new `GameAction` variant names its kind (a compile error
  until it does), and a new `EventPattern` arm its kinds.
- **(b) One generic mask over both kinds enums.** `EventKindMask` and the new
  mask share their methods, about 40 lines not written twice. But a mask
  sized from a type parameter's variant count needs an unstable feature, so
  the generic one is one `u64` with a compile-time bound, and
  `EventKindMask`'s design (it widens past 64 kinds with no edit, TR-2b's
  decision 5) is lost.
- **(c) Reuse `EventKind`**, filing each action under the record kind it
  performs. No new type, but two vocabularies conflated: a destruction, a
  draw game, a token ceasing to exist and an instruction to draw several
  cards have no record kind of their own, and the mapping would be a second
  table kept in step with the performers.
- **(d) No mask: file the printed defs**, as the zone leg does
  (`zone_replacement_ability_sources`), and ask `printed_could_apply` of each
  source per gather. No new type; but each source costs a pattern and
  affected-set check per gather, which can read the subject's frame.

**Chosen: (a)** (the owner, 2026-10-30). It is the shape the owner named, it
costs a bit test where the others cost a check or a read, and a variant
cannot be added without saying its kind.

**Added at the review, from the recount: an object's own entry is never
filed.** A `SourceOnly` def's kinds leave out `EnterBattlefield` wherever
the def is filed. Its own entry is source 1a's (CR 614.12), and on any other
leg the object cannot be the one entering (CR 400.7). A def left with no
kind is filed nowhere. A tapland on the battlefield then costs nothing,
where under the arm's kinds alone it would be asked at every entry.
Mowu's "if one or more +1/+1 counters would be put on ~" keeps
`AddCounters`. And a board whose only replacement sources are taplands
passes no gate: the gather passed its gate on 90–99% of calls at Commander
scale because a pooled tapland was a source
(`triggers-architecture.md#112-the-same-probe-at-commander-scale-and-what-it-says-is-next`).
About 15 lines, and a test that a tapland is not filed.

**The zone leg keeps its printed defs**, and under
decision 2(a) its check opens with the same kind test, since
`printed_could_apply` asks `pattern_watches`. Its def check is finer than a
kind: a Darksteel Colossus in a library watches zone changes into a
graveyard, the commonest kind there is, and the def check turns most of them
down before any frame is read. A later consumer, not built here:
`is_prohibited`'s sweep (item 44) asks `pattern_watches` too and could file
its sources the same way.

### 2. One table, not two kept in step

The problem: `pattern_watches` pairs a pattern arm with the action arms it
watches, by its match, and `action_kinds` would say the same pairing again.
If they disagree, either a card is dead behind the gate (the kinds omit a
pairing the match has) or the gate over-approximates (harmless).

- **(a) `pattern_watches` asks the kinds first** and returns false for a kind
  the pattern's kinds lack, its match refining from there. The kinds are then
  the discriminant test for the gate and the matcher both, as
  `TriggerEvent::record_kinds` is for the dispatcher and the matcher: a
  pairing the kinds omit is dead in both, so a card's own test fails, and the
  gate can never skip what the matcher would take. A "can't" asks the same
  function, so a restriction obeys the same table. Cost: one bit test per
  `pattern_watches` call. The two-kind pairings are already pinned by tests
  that would fail: `phase_re5_integration_test.rs`'s
  `hardened_scales_adds_one_to_the_counters_a_creature_enters_with` and
  `winding_constrictor_adds_one_to_the_counters_a_creature_enters_with` (a
  counter pattern on an entry), and `phase_re4_integration_test.rs`'s
  `a_token_that_cannot_enter_is_not_created` (a zone-change pattern on an
  entry, through a "can't").
- **(b) Two tables and a test** that runs every pattern arm against a sample
  action of every kind and requires the two to agree. About 70 lines of
  sample actions, and the agreement holds only for the samples.

**Chosen: (a)** (the owner, 2026-10-30),
`engineering-practices.md#one-record-never-two-kept-in-step`.

### 3. The named rows' objects on the battlefield leg

The problem: today a grant or copy row over named objects
(`RegistryScopeSummary::any_named_unattributed_replacement`) sends the gather
back to the whole walk. 86 cards make a permanent become a copy, 121 grant
protection to named objects (Mother of Runes) and 21 grant a quoted
replacement, which makes that the common case on a Commander board. While the flag is set, the zone leg
also scans every continuous-effect row on every gather, and each copy row's
captured ability list (`puts_a_replacement_ability`), to find the named
objects.

- **(a) Read each named row's objects for every proposal**, sharing the zone
  leg's scan, and put the ones on the battlefield among the candidates; the
  summary keeps its flag. Host rows are included here, which the zone leg
  leaves to the battlefield. About 15 lines. Performance: the row scan on
  every gather while the flag is set, as the zone leg pays today, plus one
  frame read (almost always a memo hit) per named object.
- **(b) File the rows by kind too, the dispatcher's way**: the flag becomes
  `named_unattributed_replacement_kinds`, a mask of the kinds the replacement
  abilities those rows put watch, as `named_unattributed_trigger_kinds` is,
  and a row is read only when its abilities watch the proposal's kind. The
  summary is already recomputed at every registry change, so a row's kinds
  are derived there, once. About 40 lines. Performance: no row scan for a
  proposal no named row's ability watches, on both legs.

**Chosen: (b)** (the owner, 2026-10-30). It is the dispatcher's shape; it
takes the row scan off the zone leg too; and its derivation runs at a
registry change, not at a gather. A row's kinds leave out its own entry, as
decision 1's file does. Protection granted by name will watch only damage
once it lands as CR 702.16e's prevention (`backlog.md` §2.6), so a Mother of
Runes activation will not open the board to the walk for a draw or a land
drop.

### 4. Where the gather finds a replacement counter

The problem: a shield, stun or finality counter makes a replacement on any
permanent (source 5, CR 122.1c/d/h). Today `any_replacement_counter` scans
the battlefield for one on every gather that passes the board gate, and the
walk then asks every permanent's counters. The scan exits early on a
permanent with no counters. But a Commander board almost always has counters
somewhere, and more each year (the owner, at review), so the early exit
fires less and less, while the three kinds the gather wants stay rare: 28, 91
and 40 cards put one. The gather needs the holders of three kinds without
visiting every other permanent.

Read today: a permanent's counters live on `PermanentState`, inside the
battlefield's shared chunks, so a fork copies none of them. They are written
by `GameState::add_counters` and `remove_counters` (`perform_action`'s two
arms and the scenario builder), by `PermanentState::entering` as a permanent
arrives with counters, and by nothing else but two tests, which put +1/+1
counters directly. The scan's doc says why it is computed: a set kept beside
the counters drifts at any write that skips it.

- **(a) An index of the three kinds' holders, behind one read and write
  path.** `GameState::replacement_counter_holders`, written by the two
  `GameState` writers, by `insert_battlefield_entity` for an arrival's
  counters, and by `cleanup_zone_state` as a permanent leaves.
  `PermanentState::counters` becomes private to `state/` and is read through
  accessors there. A write outside `state/` does not compile, and a debug
  assertion at each gather that the index is the scan's answer guards what
  `state/` itself writes. The counters stay in the shared chunks. About 40
  lines, and a sweep of about 15 readers in `src`, 6 in `tests` and the dev
  GUI's snapshot; the two tests move to `GameState::add_counters`.
  Performance: a fork pays nothing while the index is empty and one
  allocation while any permanent holds one of the three kinds. Upkeep: two
  records inside one module, kept in step by its three writers.
- **(b) A counter registry: one owner of every permanent's counters, indexed
  by kind.** `GameState.counters: CounterRegistry`, with two private halves:
  per object, kind → `CounterStack`; per kind, the objects holding one.
  Only its own `add`, `remove` and `clear` (as a permanent leaves, CR 122.2)
  write either half. `of(id)`, `count(id, kind)` and `holders(kind)` read it,
  and `PermanentState::counters` goes. Nothing outside the type can write
  either half, so the index cannot drift: one record. The gather asks
  `holders` for three kinds, and any later "which permanents have a +1/+1
  counter" reader asks the same (proliferate's choice, CR 704.5q's
  annihilation, a filter's "with a counter on it").
  Performance:
  - The per-object half leaves the battlefield's shared chunks, so a fork
    copies it unless it gets chunks of its own. As a plain map that is one
    allocation for the map and one per holder: about 10 more on a
    counter-rich four-seat board, against floor 2's 39 of 64. As an
    `ObjectTable`, which the battlefield is, it is one or two.
  - The index costs a fork one allocation per kind present.

  About 150–200 lines, and the same readers rewritten to ask by id. Scope:
  this is the counter store `route-review` (2026-10-29) slotted to
  `layers-combat-and-counters`, to be decided once with its other two
  customers:
  - counters on a card off the battlefield (item 23): 71 suspend cards put
    time counters on a card in exile, and Skullbriar keeps its counters
    across zones;
  - named counters ("Before card breadth" item 3, `CounterType::Named`).

  Built here and battlefield-only, it fixes the store's shape before those
  two exist. Built here for every zone, it brings item 23's open question
  into this PR: what CR 613.7c's counter timestamp means for a card off the
  battlefield. The PR grows to about 1,000–1,100 lines.
- **(c) Keep it computed, without the sort.** One pass over the battlefield's
  permanents per gather, collecting the holders; no drift possible. About 10
  lines. Performance: a pass over every permanent per gather, which on the
  50-Goblin board is 200 reads a proposal.

**Recommended: (a), shaped as (b)'s interface.** Its readers go through
accessors in `state/`, and its writes through `state/`'s three writers. Those
are the registry's read and write sides. When `layers-combat-and-counters`
decides the store with its other two customers, the storage moves behind
them, and the index becomes the registry's own. Built now, (b) would decide
the store with one customer of three. If you would rather have the registry
now, it would go in its own `ObjectTable`, battlefield-only, with item 23's
timestamp question filed against it.

## Size

Re-counted against the tree: item 356's **Sized:** line said 80–150 lines and
item 22's about 15. As chosen, with decision 4 as recommended:
- about 110 for the type and its tables;
- 15 for leaving an object's own entry out;
- 50 for the battlefield leg, less `any_replacement_counter`;
- 40 for the named rows by kind;
- 40 for the holders and their check;
- about 30 lines of mechanical sweep.

That is about 285 lines of code. Tests, 90–130:
- a printed source that becomes a copy of another, offered once;
- a copied replacement found beside a printed source of another kind;
- a named row whose ability watches another kind, read for none of this
  proposal's;
- a tapland on the battlefield, filed nowhere;
- a shield counter put on mid-game, one an arriving permanent brings, and the
  holder gone when its last one is removed.

The branch already holds about 420 lines of code and tests (k's flag at
about 200, twice its estimate; the fork test at about 140), so the PR lands
near 800–850. That is under the 1,500–2,500 band and above the brief's
300–450. The recount's script, about 260 lines in `plans/references/`, is a
research tool and not in that count. Decision 4(b) would add about 150–200.

## Where each piece lands after the build

- `replacement-architecture.md#33-where-replacement-effects-come-from`:
  sources 1 and 5 as built, the four decisions under it, and the recount's
  table with `plans/references/replacement-kinds-census.py` named as its
  instrument.
- `engine-map.md#23-the-gates-and-the-three-legs-a-new-source-extends`: the
  replacement row's printed leg filed by kind, its named leg by kind on the
  battlefield too.
- `codebase-state.md`: items 356 and 22 closed; item 44 notes the
  restriction gate could file the same way.
- `backlog.md` §2.5: a mill's number as an instruction a replacement watches
  (Bruvac the Grandiloquent).
