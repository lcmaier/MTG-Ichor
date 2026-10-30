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

Counted 2026-10-30 on Scryfall, each query under `game:paper -is:funny`,
`unique=cards`, `total_cards` off page one; a second phrasing for each family
that read zero, and for the entering families since 2024's wording drops
"the battlefield".

| What the kinds must do | Family | Query | Cards | Example | Arm today |
|---|---|---|---:|---|---|
| **One kind** | prevent damage | `o:prevent o:damage` | 523 | Abuna Acolyte | `DealDamage` |
| | damage replaced | `o:"would deal" o:damage o:instead` | 84 | Aegis of Honor | `DealDamage` |
| | dies, or put into a graveyard | `o:/would (die\|be put into [a-z ]*graveyard)/` | 193 | Rest in Peace | `ZoneChange`, its `to` a graveyard |
| | leaves the battlefield | `o:"would leave the battlefield"` | 15 | From the Catacombs | `ZoneChange` |
| | a discard | `o:"causes you to discard"` | 17 | Dodecapod | `ZoneChange`, its act |
| | enters | `o:"would enter"` | 16 | Containment Priest | `EnterBattlefield` |
| | others enter tapped | `o:/(creatures\|permanents\|lands\|artifacts)[^.]* enter tapped/` | 26 | Archon of Emeria | `EnterBattlefield` |
| | draw | `o:"would draw"` | 49 | Abundance | `DrawCard`, `DrawCards` |
| | gain life / lose life | `o:"would gain life"` / `o:"would lose life"` | 21 / 2 | Angel of Vitality | `GainLife`, `LoseLife` |
| | tokens | `o:/would (create\|be created)/ o:token` | 34 | Anointed Procession | `CreateTokens` |
| | skip a turn, phase or step | `o:/skip (your\|their\|each\|that player.s)/` | 31 | Chronatog | `BeginTurn`, `BeginPhase`, `BeginStep` |
| | mana | `o:/(would add\|would produce\|produces? (twice\|three times))/ o:mana` | 4 | Mana Reflection | `ProduceMana` |
| | untap | `o:/would (untap\|become untapped)/` | 2 | Freyalise's Winds | `Untap` |
| | destroy | `o:"would be destroyed"` | 7 | Crackling Emergence | `Destroy` |
| | scry / lose the game | `o:"would scry"` / `o:"would lose the game"` | 2 / 4 | Eligeth / Lich's Mirror | `Scry`, `PlayerLoses` |
| | can't win | `o:"can't win the game"` | 9 | Abyssal Persecutor | `PlayerWins`, through `Restriction::Event` |
| **Two kinds** | counters put on | `o:counters o:/would (be put\|put)/` | 34 | Hardened Scales | `AddCounters`, and an entry: "Doubling Season affects permanents that enter with counters" (ruling, 2024-11-08) |
| | enter with additional counters | `o:"enter with" o:additional` | 9 | Bard Class | the same two |
| | can't enter | `o:"can't enter"` | 6 | Grafdigger's Cage | `ZoneChange` whose `to` admits the battlefield, and an entry |
| **An arm, no printed card** | becomes tapped | `o:/would (tap\|become tapped)/`, `o:/can.t (become tapped\|be tapped)/` | 0, 0 | — | `Tap`, kept in the kinds as every arm is |
| **No pattern arm** | a shuffle | `o:"would shuffle"`, `o:shuffle o:instead o:would` | 0, 16 (all zone changes: Blightsteel Colossus) | — | none; nor for becoming blocked, a creation into a zone, attaching, unattaching, a token ceasing to exist, a draw game |
| **No `GameAction` arm yet** | a search | `o:"would search"` | 1 | Aven Mindcensor | `backlog.md` §2.5 |
| | a die roll | `o:/would roll/` | 7 | Barbarian Class | `backlog.md` §2.31 |
| | a mill's number | `o:"would mill"` | 2 | Bruvac the Grandiloquent | none: a line in `backlog.md` §2.5 (CR 701's keyword actions) at this PR's close |
| **Not indexed by kind** | shield / stun / finality counters | `o:"shield counter"` / `o:"stun counter"` / `o:"finality counter"` | 28 / 99 / 44 | Boon of Safety | source 5, from the counter |
| | a copy | `o:/(enter\|enters\|becomes?) as a copy of/` | 64 | Spark Double | a named copy row; an entry copy is the printed leg |
| | a grant by name | `o:/(has\|have\|gains?) "[^"]*instead/` | 12 | Bewitching Leechcraft | a named grant row, its `Host` |
| | a grant over a filter | `o:/(creatures\|permanents)[^".]* (have\|gain) "[^"]*instead/` | 1 | Scion of Halaster | a `Filter` row's zones, walked |

**Reading.** Every printed family watches one `GameAction` kind or two, and
every pairing is already one arm of `pattern_watches`: the kinds add no
vocabulary, they index what the arms pair. A zone-change pattern watches an
entry only when its destination admits the battlefield, so the kinds read
the pattern's `to`: filed by the arm alone, the 193 "dies or put into a
graveyard" patterns would be asked at every entry, each Goblin of the token
board's included. The three families with no `GameAction` arm each arrive
with a variant, so a proposal's kind must be an exhaustive match a new
variant cannot compile past. The rulings read for the table set two more
constraints: a counter pattern watches an entry (Hardened Scales, Doubling
Season), and a shield or finality counter works on a permanent that lost its
abilities ("If that permanent loses its abilities and then would go to a
graveyard, it will still be exiled instead", Balustrade Wurm, 2024-09-20), so
the counter route may not hang off the ability list.

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

**Recommended: (a).** It is the shape the owner named, it costs a bit test
where the others cost a check or a read, and a variant cannot be added
without saying its kind. **The zone leg keeps its printed defs**, and under
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

**Recommended: (a)**, `engineering-practices.md#one-record-never-two-kept-in-step`.

### 3. The named rows' objects on the battlefield leg

The problem: today a grant or copy row over named objects
(`RegistryScopeSummary::any_named_unattributed_replacement`) sends the gather
back to the whole walk. 64 copy cards and 12 grants by name make that the
common case wherever a Clone resolves. While the flag is set, the zone leg
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

**Recommended: (b).** It is the dispatcher's shape, the owner's ask; it takes
the row scan off the zone leg too; and its derivation runs at a registry
change, not at a gather. (a) is smaller, but leaves a per-gather scan on
every board with a Clone.

### 4. The permanents holding a replacement counter

The problem: a shield, stun or finality counter makes a replacement on any
permanent (source 5), and today `any_replacement_counter` scans the
battlefield for one on every gather that passes the board gate, and the walk
then asks every permanent. Its doc records why it is computed rather than
cached: a set kept beside the counters drifts at any write that skips it.
Item 22 sized the set and found it sound since TR-5b took CR 704.5q's counter
annihilation through the `RemoveCounters` arm. Read today: a permanent's
counters are written by `GameState::add_counters` and `remove_counters`
(`perform_action`'s two arms and the scenario builder), by
`PermanentState::entering` as a permanent arrives with counters, and by
nothing else but two tests, which put +1/+1 counters directly.

- **(a) A set, with the write path narrowed so no other writer compiles.**
  `GameState::replacement_counter_holders`, written by the two `GameState`
  writers, by `insert_battlefield_entity` for an arriving permanent's
  counters, and by `cleanup_zone_state` as a permanent leaves.
  `PermanentState::counters` becomes private to `state/`, read through a
  `counters()` accessor, and `PermanentState`'s own two writers
  `pub(in crate::state)`, so a write anywhere outside `state/` does not
  compile. About 40 lines, and a mechanical sweep of the field's readers
  (about 15 in `src` outside `state/`, 6 in `tests`, the dev GUI's snapshot) and
  the two tests moved to `GameState::add_counters`. A debug assertion at each
  gather that the set is the scan's answer guards what `state/` itself writes.
- **(b) The same set, kept at the writers by convention.** About 40 lines and
  no sweep; drift is caught by the debug assertion, which runs in every debug
  test and in CI's two `release-checked` fuzz steps (`ci.yml`, "debug
  assertions on"), and by the audit's whole walk under `--audit`, but a new
  writer compiles silently.
- **(c) Keep it computed, without the sort.** One pass over the battlefield's
  permanents per gather, collecting the holders; no drift possible. About 10
  lines. Performance: a pass over every permanent per gather, which on the
  50-Goblin board is 200 reads a proposal.

**Recommended: (a).** The old doc's fear was a writer that skips the set, and
(a) makes that writer fail to compile instead of failing a fuzz run. The set
costs a fork nothing while it is empty and one allocation while any permanent
holds a replacement counter, against floor 2's 39 of 64 allocations a clone;
the close's floors re-read it.

## Size

Re-counted against the tree: item 356's **Sized:** line said 80–150 lines and
item 22's about 15. As recommended: about 110 for the type and its tables,
50 for the battlefield leg (less `any_replacement_counter`), 40 for the named
rows by kind, 40 for the holders and their check, and about 30 lines of
mechanical sweep: about 270 lines of code. Tests, 80–120: a printed source
that becomes a copy of another, offered once; a copied replacement found beside a printed
source of another kind; a named row whose ability watches another kind read
for none of this proposal's; a shield counter put on mid-game, one an
arriving permanent brings, and the holder gone when its last one is removed.
The branch already holds about 420 lines of code and tests (k's flag at about
200, twice its estimate; the fork test at about 140), so the PR lands near
770–810, under the 1,500–2,500 band and above the brief's 300–450.

## Where each piece lands after the build

- `replacement-architecture.md#33-where-replacement-effects-come-from`:
  sources 1 and 5 as built, the four decisions under it.
- `engine-map.md#23-the-gates-and-the-three-legs-a-new-source-extends`: the
  replacement row's printed leg filed by kind, its named leg by kind on the
  battlefield too.
- `codebase-state.md`: items 356 and 22 closed; item 44 notes the
  restriction gate could file the same way.
- `backlog.md` §2.5: a mill's number as an instruction a replacement watches
  (Bruvac the Grandiloquent).
