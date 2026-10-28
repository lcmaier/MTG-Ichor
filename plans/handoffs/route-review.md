# Route review — the outline

**The design stop for `route-review`**, the route review after triggers
(`roadmap-v2.md`'s "From triggers to Phase 8", first row). It absorbs pass 6,
scheduling, of the item 6 audit (`plans/handoffs/item-6-audit.md`). Nothing a
player sees changes: this page proposes a v1 that ends, the shapes designed
before cards are written, an order, and a home for each of the 62 open items
four slots list. §6 puts six choices to the owner; the build starts on the
answers, on this branch.

Counts marked **(C)** are the card census of 2026-10-26: Commander-legal cards
whose Forge script needs the step (`plans/references/card-census/results/census-2026-10-26.txt`).
Counts marked **(S)** are Scryfall's `total_cards`, `game:paper -is:funny`,
2026-10-27, each query under the table. "Cited by" is how many other CR
sections cite one (`plans/references/cr-coupling.py` over `tmnt.txt`).

## 1. What v1 is

- **The pool: Starter Commander Decks** (`scd`, 2022-12-02): five ready-built
  100-card decks sold together, 332 distinct cards less basics. It seats four
  from one product with a deck to spare; the four products with fewer
  distinct blockers each sell two decks (`results/precons-2026-10-26.txt`).
- **What it needs.** The census reads 190 of the 332 as authorable and 135 as
  blocked by a named step; the authoring check passed 7 of 20 "authorable"
  cards (`card-census/authoring-check.md`), so expect about a third of the
  190 to work today. Its blockers by cards: any-color mana 50, `remaining-costs`
  (discard, counters and exile as costs) 21, `information-model` 17, modes 13,
  `casting-from-other-zones` 12, "unless a player pays" 8, convoke 6, amass 6,
  `combat-requirements` 5, token copies 4, "for each" 4, and 1–3 each for six
  more steps and five more mechanics. No card in it has two faces, is face
  down, melds or mutates, and none is a battle (it predates them).
- **So v1 is** those five decks, four at a time through a GUI and in parallel
  through the harness: both use cases (the owner, 2026-08-24) on a pool that
  ends. "Every card but the genuinely obscure" (2026-08-30) comes after v1.
- **The AI player** comes after v1 (decision 2). The harness's shapes come
  before the first deck.

## 2. The shapes

A shape is a change that, made later, unbuilds something already built
(`cr-coverage-audit.md#2-the-method--the-type-surface`); an arm only adds.
Each is designed before the first deck's cards are written. What each must
hold at v1, which rules let a card override it and which families do
(`engineering-practices.md#what-a-new-type-must-express`) are each shape's own
design stop's; this page orders them.

| # | Shape | Owning doc | CR, cited by | Cards | Why a shape, not an arm | Built before the first deck |
|---|---|---|---|---:|---|---|
| 1 | **Pausing a game at a decision**: the pending decision on `GameState` (item 40's invariant), the provider's shape, every option named by what it is | `setup-architecture.md`, which owns the save and the decision log's replay | 116, 117: 27 | every game | 29 `ask_*` functions are called from 37 sites in 15 files, each waiting on the provider. A GUI over a network and a harness running games side by side need a game to stop at a decision and resume; later, it rewrites every site and every choice kind written before it | yes |
| 2 | **Modes plus the other choice kinds**: modes (700.2), "unless [a player] pays" (118.12a) and ward, X paid as it resolves, a division (601.2d), an order the CR hands a player (601.2h, 613.7m), a choice at resolution (608.2d) | shape 1's | 601: 66; 700: 8 | 915 modal (C), 823 (S); 201 "unless … pays", 198 ward; 57 pay X; 79 divided; 173 "in any order"; at most 793 choices at resolution | A choice kind is what a client, the harness and the decision log all see, and changing one is a fixture migration. Modes also reshape the stack object: chosen before targets (601.2b), kept by a copy (707.10). `Effect::Modal` is refused at resolution, and `StackEntry.chosen_modes: Vec<usize>`, written empty and read nowhere, picks modes by position: it goes, for modes named by identity | yes |
| 3 | **What one ability remembers for another** (607, linked abilities) | a doc of its own, graduating `backlog.md#22-linked-abilities-cr-607` | 607: 13 | 432 "the chosen …", 291 "as this enters, choose", 251 "was kicked", 137 "exiled with" (S, overlapping, from that entry's table) | `AbilityDef` carries no link, and each pair so far got a field of its own: `CostChoices` (kicker, bargain and the other paid choices, on the stack entry, the resolving object and the permanent), `x_value` (CR 107.3m's X) and `CastFacts` (CR 400.7d). Tribute (item 338) and "prevented this way" (item 288) are gaps. Whether each folds into one record is the design's first decision; every card before it adds another field | yes |
| 4 | **The information model** (400–408) | its design's own doc, `information-model-design` | 12 | 2,394 (C) | The GUI shows one player's view; an AI observation must not see a hidden zone. Every client reads through it, and what it keeps is forked state, which floor 3 (kilobytes per clone) bounds | yes: `information-model-design`, `information-model` |
| 5 | **Permissions and casting from other zones** (601.3, 602.2, 113.6, 903.8) | `permission-architecture.md` | 601: 66 | 1,832 (C); every commander | `can_begin_to_cast` refuses a card not in hand, so no commander can be cast. Who may cast what from where is a fact of the cast path's types | `permission-design` and `command-zone-casting` yes; `casting-from-other-zones` with the decks |
| 6 | **Attack and block requirements** (508.1d, 509.1c; costs to attack, 508.1g–h) | `cant-effects-architecture.md` | 506–511: 29 | 1,567 restrictions, 322 requirements (C) | The CR's declaration obeys the most requirements without breaking a restriction, so declaring is a choice among legal declarations, not a pick per creature: the choice and its validator change shape | yes |
| 7 | **The card and object model**: faces and characteristic sets as two independent choices (712, 709, 715, 718, 720, 708), several components for meld and mutate (729, 712.4), an attachment to a player (303.4m) | `copy-effects-architecture.md`, which owns faces (`copy-effects-architecture.md#45-cr-712--faces-transform-modal-meld`), face-down (`copy-effects-architecture.md#46-cr-708--face-down-and-layer-1b`) and merging (`copy-effects-architecture.md#6-v1-scope--everything-in-an-order`); `permission-architecture.md`'s characteristic sets; `layers-architecture.md`'s host | 712: 9; 708: 9; 729: 3 | 583 two-faced, 328 split and Adventure, 361 face down, 51 meld and mutate (C); 51 enchant a player (S) | `CardData` holds one set of characteristics and `PermanentState` one object. Faces stay behind `oracle/characteristics.rs`, the one reader the layer invariant allows; components and a host are read across the engine. Designed first, the 332 cards are written in the shape that stays | **design only**, beside `permission-design` as today's route has it. The builds are the four copy-track rows, after v1 |

Queries (S): `(o:"choose one" or o:"choose two" or o:"choose three" or o:"choose one or more" or o:"choose any number")`;
`o:/unless (that player|its controller|any player|they|its owner|an opponent|target player|that spell.s controller) pays?/`;
`kw:ward`; `(o:/may pay \{X\}/ or o:"pay any amount")`; `o:"divided as you choose"`; `o:"in any order"`;
`(o:"of your choice" or o:"of their choice" or o:"you choose" or o:"chooses a")`, an upper bound since it
counts entry and cast choices too; `(o:"enchant player" or o:"enchant opponent")`.

**Trimmed: "amounts that can go negative."** CR 107.1b turns any negative
result into zero except a power and toughness change and an effect that
doubles or sets a life total or P/T. The layer side is signed already
(`PtValue::Fixed(i32)`); what can't be written is the resolution's P/T
change, `Primitive::ModifyPowerToughness(AmountExpr, …)`, whose amount is
unsigned (764 cards print a negative P/T change (S), 612 until end of turn;
62 sites write the primitive). That is an arm on one primitive and its
lowering, and it goes to `layers-combat-and-counters` beside Layer 7c's
doubling, whose CR 701.10c needs it.

**A shape that waits:** mana provenance, each unit keeping its source
(`mana-provenance`), reshapes the pool, and no `scd` card needs it.

**Heavily cited, not shapes:** multiplayer (CR 800–811, cited by 17): turns and
priority already run over N seats, and CR 800.4f–i and 802 are
`multiplayer-rules`' arms. Commander (903) is data in `game-formats`, with
color identity and partner. Starting the game (103, cited by 15): mulligans
and CR 103.6's openers are a step before the first turn, whose decision is
shape 1's.

## 3. The order

**Before the first deck**, in order:
1. `route-review`, this PR.
2. Readiness, the item 6 audit's last two measurement PRs (§5).
3. `pausing-at-a-decision`, design then build: every choice kind after it is written in its shape.
4. `modes-plus-choice-kinds`, design then builds; `divided-among-targets` folds in.
5. `linked-abilities`.
6. `resolution-recipients-and-amounts`, the audit's row 6 on #295's design, after the shapes whose types it writes.
7. `information-model-design`, then `information-model`.
8. `permission-design`, with `card-and-object-model`, `cant-cast-play-or-activate`'s reasons and CR 610.5's grant at cast designed beside it; `command-zone-casting`, `game-formats`, `multiplayer-rules`.
9. `combat-restrictions`, `cant-pay-costs`, `combat-requirements`, then `layers-combat-and-counters`, the audit's row 7.
10. `card-layer-cleanup` and `rulings-for-every-card`, today's gates on Phase 8.

**The decks.** `starter-commander-decks` opens with a gap census: each of the
332 cards read against the tree, since the census misses parameters. It
confirms the rows the census says the decks need:
`any-color-plus-painlands-plus-signets`, `chosen-color-plus-could-produce`,
`convoke-delve-improvise`, `hybrid-and-phyrexian-payment`, `remaining-costs`,
`casting-from-other-zones`, `token-copies`, `spell-copies`, with
`mana-ability-costs` first, since it widens the check the other mana rows
extend (the owner, at #224's and #225's reviews). Then the five decks, which
own the breadth mechanics their cards need (amass, devotion, overload and the
rest), each built whole by the family rule. Then Phase 10: the GUI, with
`mana-payment-solver` for a person paying at a seat, and the harness. That is v1.

**The family rule.** When a deck's card needs a mechanic, its whole CR section
is built, with a fixture for each rule no deck card prints: one kicker card
builds CR 702.33.

**After v1**, the rest of today's route in its order: `cant-be-targeted`,
`as-though-effects`, `mana-provenance`, `ironworks-loop`, the card model's
builds (`double-faced-cards`, `split-and-adventure-style-cards`,
`face-down-and-morph`, `merging-and-meld`), `loop-detection`,
`text-changing-effects`, `breadth-features`; then Phase 9's remainder (teams
and Two-Headed Giant, CR 805 and 810; Limited); cards from more precons,
fewest blockers first (decision 1). `CLAUDE.md`'s "Phase 8 breadth → Phase 9 →
Phase 10" becomes the decks, then Phase 10, then everything after v1.

## 4. Homes

Each of the 62 items' proposed row:

- **`pausing-at-a-decision`**: 287 (a save names casts and counts the rest), first in the route-review row's audit of the scenario format and the dev GUI together against what the engine now records, since a paused game is saved in that format; 336 (an ordering option names an effect by its source).
- **`modes-plus-choice-kinds`**: 28 (a choice made at resolution), 79 (the payment order is the player's), "Before card breadth" 4 (CR 613.7m's order of the player's choice).
- **`linked-abilities`**: 288 ("prevented this way"), 335 (an "as this enters" effect can be any instruction), 338 (whether tribute was paid), 349 (a delayed trigger's "if" naming what it remembers).
- **`card-and-object-model`**: 227 and 346 (an Aura on a player, "enchanted player"), whose design names the row that builds them.
- **`multiplayer-rules`**: 342 ("an opponent" read two ways, asserting at three seats), 345 ("target opponent", 742 cards), 348 (`PlayerRef::Owner` read two ways), and item 341's one hazard, `NamedPlayers::FirstInstance` picking a player by position.
- **`resolution-recipients-and-amounts`** keeps its 14 (30, 90, 94, 106, 155, 268, 269, 272, 281, 292, 299, 307, 329, 334) and takes 347 ("target player or planeswalker", "target ability") and 350 (a resolution sweeping a graveyard).
- **`layers-combat-and-counters`** keeps its 16 (12, 86, 117, 184, 190, 205, 206, 278, 308, 322, 323, "Before Layers" 16, "Before card breadth" 1, 3, 5 and 6) and takes 23 (counters off the battlefield, beside "Before card breadth" 3's named counters: one counter store), 331 (what a static's duration means), the negative P/T arm, and 188 (two roads to one state), whose open candidates are mostly layer and combat code and which the owner placed before breadth.
- **`game-formats`**: 119 (CR 103.6's openers), with mulligans, color identity and partner; `setup-architecture.md` names the pregame steps its neighbors, so their design is written there.
- **`card-layer-cleanup`**: "Before Layers" 7d (`ContinuousEffect { id: 0 }` meaning "not yet assigned", over 44 literals) and row 8's `AbilityDef.instances` (a list the builder fills, written empty in 205 struct literals): both are written by every card.
- **`text-changing-effects`**: "Before Layers" 7 (a static granted by a static, Rune of Might's miss), the other design in the layer walk.
- **`loop-detection`**: 264 (an exponential token board, and `GameNumber`).
- **`breadth-features`**: 87 (battles and their protector, CR 310); no `scd` card is a battle.
- **`partial-damage-redirection`**, a new row after v1 graduating `backlog.md#225-partial-damage-redirection-cr-6149--one-event-becoming-two`: 125 (Exquisite Archangel), whose shape is a replacement making two events (`replacement-architecture.md`'s RD-5 section), the ruling's exile-or-graveyard choice riding on it.
- **`names-and-hygiene-sweeps`**, row 8 moved to Phase 10 as the cleanup before release: 215 (MA-1's two cleanups: the mana checks as methods on one value, its memo audits in debug builds), 239 (the `.clone()` audit), 285 (tuple returns), the naming sweep, the renames (`chars` spelled `characteristics`, no id called `seq`, the `lki_` prefix), `rust-notes`, and the route-review row's question whether `GameState`'s fields go private to their performers, for which A6q's write check stands in until then.
- **Retired** (decision 4): 340, 343, 344 and the rest of 341, one spelling per word.

**The route-review row's own list of 2026-10-13**, the rest of it: the card
parser is declined for v1 (332 cards by hand, on `card-layer-cleanup`'s
constructors); "unless pays" and modal triggers are shape 2, CR 607 shape 3,
costs to attack shape 6, meld beside the characteristic sets and foretell
before face-down shape 7; pending decisions, pausing at a prompt and the
provider's shape are shape 1; the token board for the floors goes to
readiness.

## 5. What is left of the audit

- **Readiness's two PRs** stay, next: the ratchet (decisions per loaded
  core-second, read at item 6's close, never worse per decision without a
  written reason), the three floors with the clone timer (time per clone and
  drop, by the median of five means), the fork test (item 41: a game forked at
  a priority decision replays the same), and k, an AI observation's cost per
  decision, which `information-model-design` and `pausing-at-a-decision` both
  read. Items 138 (the throughput ratchet), 142 (the panic-surface baseline)
  and 143 (the clone table) stay theirs.
- **Pass 6** is this PR (decision 3).
- **Rows 6–8 leave the audit** (decision 3). #295, row 6's design note with
  its decisions settled, closes, and the note comes to `plans/handoffs/` in
  this PR's build.
- **The handoff file** is deleted by readiness's second PR, which writes the
  audit's two records (`codebase-state.md`'s "Was critical-path item 6 done"
  heading and `spine-phase-audit.md`'s second instance).

## 6. Decisions

**1. The v1 card pool, and the order after it.** v1 has no end today: every
card but the obscure, about 32,000 legal in Commander, of which the census
suggests about one in six plays now, so every row waits on "before Phase 8".
- (a) `scd`'s five decks, in §3's order. The card model's builds,
  `cant-be-targeted`, `as-though-effects`, `mana-provenance`, `ironworks-loop`,
  `loop-detection` and battles move after v1. A family no deck needs is
  unbuilt at v1; the CR stays the customer after it.
- (b) More tables: `scd` and the next products by fewest blockers (two decks
  each, 149–192 cards apiece). Same shapes, more mana and costs.
- (c) v1 stays every card: today's route and its back-stops stand.
- After v1, (i) more products, fewest blockers first, or (ii) Alpha onward in
  release order, which meets 22 of the census's 37 steps in its first year of
  sets and so orders little.
- **Recommended: (a), then (i).**

**2. Where the AI player goes.** "Parallel AI games over the CLI" needs the
harness, an engine that stops at each decision for many games at once; it
does not say who plays.
- (a) After v1: shape 1 and k before the first deck; a player that chooses
  well is its own project on the harness.
- (b) Before v1, so one person can fill a table: a search player needs the
  information model's per-viewer redeal
  (`backlog.md#234-a-search-over-a-cloned-gamestate-sees-its-next-draw--determinization-and-what-29-does-not-do`)
  and the fork floors, and becomes a second consumer of every choice kind.
- (c) A simple heuristic player before v1, for testing only.
- **Recommended: (a).** v1's users are people at a table and other people's
  AIs; the random and scripted providers cover testing.

**3. What leaves the audit.** It planned three more cleanup PRs before any
route work, about 4,500–6,300 lines carried by its table: row 6, recipients
and amounts (14 items); row 7, layers, combat and counters (16); row 8, names
and hygiene (5, and four sweeps: the naming sweep, the renames,
`AbilityDef.instances`, the tuple returns). Shapes 2 and 3 reshape types row 6
writes, and pass 6 planned a table of every open phase with its cards and its
atoms by rule prefix.
- (a) Rows 6–8 stay, built next; the shapes then reshape their code. Pass 6's
  table is built as planned.
- (b) Rows 6 and 7 become route rows with their scope, after the shapes they
  write (§3, steps 6 and 9). Row 8 splits: what every card writes to
  `card-layer-cleanup`, the rest to Phase 10 under its name. #295 closes and
  its note moves to `main`, since a draft parked for months rebases onto a
  moved tree. Pass 6's table is §2's card counts; its atoms column is dropped,
  since each row's design stop reads its atoms and `owed` gates them.
- (c) Rows 6 and 7 dissolve into the shape rows, losing #295's settled
  decisions as one unit.
- **Recommended: (b).**

**4. One spelling per word: items 340, 341, 343, 344.** The census (#296)
found words spelled as several arms; #297 folded them and closed unmerged, one
spelling bought with longer call sites and more generic readers.
- (a) Retire 340, 343, 344 and 341's folds; keep 342, 345, 348 and 341's
  positional pick, which are hazards or behavior (§4). Their fold lines in
  `cr-coverage-audit.md#4d-words-and-their-spellings` become "spelled apart"
  and the retirements go to `codebase-state.md`.
- (b) Keep them for `names-and-hygiene-sweeps`; the sweep grows with every
  card (about 1,000 sites today by the four items' **Sized:** lines).
- (c) Sweep before the first deck so 332 cards write one spelling: #297
  again, about 1,850–2,100 lines.
- **Recommended: (a).**

**5. Whether #296's check stays.** `check_type_surface.py`'s third check fails
a new arm of a card-text type with no line in the word register naming the word it spells,
and deck work adds arms often.
- (a) Keep: a line per arm, so a second spelling is seen as it is written.
- (b) Drop the check and keep the register as a dated record. After 4(a), a second
  spelling is one we chose not to act on.
- (c) Keep a line only where an arm is read two ways (342's hazard). No
  script sees a reading, so this is review, not a check.
- **Recommended: (b),** if 4 is (a).

**6. The home rule.** Since #294's review a home is a route row still to come,
never "the first registered card that needs it", because a registration is
not a place on the route. With the decks as v1 the order is fixed, so "the
first deck that needs it" could be one.
- (a) Keep the rule: every item a named row, as §4 does. What a deck's gap
  census finds is slotted to `starter-commander-decks` by name.
- (b) Add a home: a CR family built whole at the first product in the card
  order that needs it, read off the census into a table in `roadmap-v2.md`
  that `check_state_of_play.py` reads.
- (c) Restore "the first registered card".
- **Recommended: (a).** §4 needed no new kind of home; (b) can wait for the
  order after v1, where it would earn its upkeep.

**The build after the answers:** `CLAUDE.md`'s critical path and `roadmap-v2.md`'s
"From triggers to Phase 8" rewritten to §3; rows for `pausing-at-a-decision`,
`modes-plus-choice-kinds`, `linked-abilities`, `card-and-object-model`,
`resolution-recipients-and-amounts`, `layers-combat-and-counters`,
`starter-commander-decks`, `partial-damage-redirection` and
`names-and-hygiene-sweeps`; the family rule in
`engineering-practices.md#gaps-a-phase-finds`; the 62 **Slotted:** lines; the
retirements and the register's lines; #295's note moved; under 5(b), the third check
taken out of `check_type_surface.py`; the audit's pass 6 row and its Status
table; the board. Docs and Python, about 500–800 lines: most of it the Slotted
lines (2–4 each), the rewritten table and the new rows, plus #295's note.
