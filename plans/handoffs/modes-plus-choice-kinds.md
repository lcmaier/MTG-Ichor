# Handoff — `modes-plus-choice-kinds`, the design note

**Opened 2026-11-01**, on `choices/modes-plus-choice-kinds` off `d00a2abd`
(#306's merge), as `roadmap-v2.md` step 5 of "From triggers to v0.5". This
file is the design note, reviewed before any code; the design moves into
`choices-architecture.md` (decision 1) once the owner has decided the rest,
and the landing deletes this file. Every count below was read against
`d00a2abd`; the printed counts are
`plans/references/choice-kinds-census.py`'s, run 2026-11-01 over its cached
corpus (every Commander-legal paper card, fetched 2026-10-05, 32,115 cards),
with a second column for Tarkir: Dragonstorm Commander's 373 cards (`tdc`,
Scryfall `e:tdc -t:basic`, committed as `plans/references/tdc-names.txt`), the
five decks v0.5 plays. The corpus is paper; Alchemy's digital-only cards, in
scope, were counted beside it on Scryfall (the census's docstring) and add no
family the paper cards lack: 84 modal, 63 "unless", 12 divided, 61 "may pay",
16 values, and 65 drafting from a spellbook, which is decision 7's card-name
pick over a list the card names.

**What changes in a game of Magic.** A modal spell or ability ("Choose one
—", CR 700.2) can be cast, activated or put on the stack and resolves with the
modes its player chose. A cost paid as an effect resolves works: "counter it
unless its controller pays {2}" (CR 118.12a), ward (702.21), "you may pay {X}.
If you do" (107.3f). An effect divided among targets is divided as its player
announced (601.2d). Two orders the CR hands a player are asked where they can
change the game (601.2h's payment order, 613.7m's timestamps). A choice made as
an effect applies (608.2d), of a color, a creature type, a card name, a
number, a word, a pile or a vote, has one shape each. And every option the
engine offers says what it is, which #306's decision 4 designed and this
phase's first PR builds.

## Where the work sits

On the event path (`engine-map.md#11-the-path-in-one-sketch`), at four points,
and across the decision boundary at every one:

- **Announcing, CR 601.2b–d** (and 602.2b for an activation, 603.3c–d for a
  trigger put on the stack): the modes, a mode's own cost (700.2h), and the
  division among targets (601.2d). `cast_spell`
  (`engine/put_on_stack.rs:77`), `activate_ability` (`:462`) and placement's
  `place_one` (`engine/triggers/placement.rs:192`).
- **Paying, CR 601.2h**: the order of the payment's parts, which
  `payment_order_rank` fixes today (`engine/costs.rs:143`).
- **RESOLVE**: a cost paid at resolution, a value chosen as the effect
  applies, a division among objects not targeted (608.2d), an `EffectRecipient::Choose`
  answered here rather than announced (item 28 and `backlog.md`'s "2.33
  Choice versus target"), and the modes a resolution walks
  (`engine/resolve.rs:521`).
- **PERFORM**: CR 613.7m's order among objects entering at once, inside the
  entry's performer.
- **The boundary** (`setup-architecture.md#7e-pausing-at-a-decision`,
  decision 3): every kind this phase adds asks through `ui::ask`, names its
  options by what they are, and holds nothing off `GameState` past the next
  anchor (a priority prompt asked for the first time in its window). That
  rule is why this step comes before the decks: a choice kind is what a
  client, the AI harness and the decision log all see.

## The intake

`check_state_of_play.py --slotted modes-plus-choice-kinds` lists ten items
(the brief named item 240 among them; the board lists it as named, slotted
elsewhere, and item 158 as slotted elsewhere with its modal axis here), and
`--slotted divided-among-targets` one. Each is taken or re-slotted:

| Item | What it is | Here |
|---|---|---|
| 31 | `StackEntry.chosen_modes` has no writer and no reader (12 construction sites, re-counted) | **taken**, decision 3 |
| 153 | the instance walk ("each instance of 'target'") would announce the targets of modes nobody chose | **taken**, decision 3 |
| 81 | a keyworded additional cost can be mandatory: spree's per-mode costs | **taken** with CR 700.2h, decision 2; spree the keyword waits for its row (below) |
| 154 | "another target" cannot exclude a player | **taken**, decision 3: the exclusion leaves the `ObjectFilter` leaf for any filter, and "each mode must target a different player" is a rule of the modal node |
| 158 | "different from that one" has three axes | its modal axis is item 154's, taken; its copy axis stays `spell-copies`' |
| 28 | `EffectRecipient::Choose` has no resolution path, and a rider cannot name "each of them" | **taken**, decision 8 |
| 79 | CR 601.2h's payment order is the player's, and the engine takes it | **taken**, decision 6 |
| "Before card breadth" 4 | CR 613.7m's order of simultaneous timestamps is the active player's | **taken**, decision 6 |
| 240 | three type-changers' facilities; their upkeep halves wait on "sacrifice it unless you …" | the upkeep half is decision 4's (Contamination's "unless you sacrifice a creature"); the rest stays slotted where it is |
| 336, 361 | #306's decision 4: options named by what they are, and the 33 tests that answer by place | **taken**, the first PR, as #306 designed them |
| 92 | a CR 615.7 count spanning subjects with two choosers is refused | **closes as a record** (decision 5): CR 615.7 states a shield on one permanent or player, chosen for by that one's player or controller, and no count shared across subjects; every printed divided prevention gives each target its own share, and the refusal stays the answer to the shape the CR does not state |

Re-counted against the tree: item 361's `picking(` uses are 176 in 36 files
(175 when measured), of which the 33 that depend on order move; `EffectRecipient::`
occurs 272 times in `mtgsim/src` (item 154's sizing said 146 construction
sites in September), which is why decision 3 moves item 154's exclusion into
`SelectionFilter` and leaves `EffectRecipient`'s arms alone;
`ObjectFilter::OtherThanInstance` has 25 uses to re-point. Item 154 counted 8
cards printing "each mode must target a different player"; Scryfall and the
census both read 7 today, Chaos Balor's text no longer saying it.

## Counted

### The rules

- **CR 700.2** and its subrules a–i: who chooses the modes and when (a, b, e),
  a mode's own targets (c, f), the same mode more than once (d), a copy's modes
  (g), a mode's own cost (h), pawprints (i). **The rules that watch it**: CR
  601.2b announces the modes first of every announcement, before splice, costs
  and X; 603.3c removes a modal trigger none of whose modes can be chosen;
  601.2c's "a spell may require some targets only if … a particular mode was
  chosen"; 707.10's copy, which copies the modes.
- **CR 118.12 and 118.12a**: "[do something] unless [a player does something
  else]" means "[the player] may [do it]. If they don't, [do something]", and
  the action is a cost paid as the effect resolves. **Watching it**: CR 118.3
  (no player pays a cost they can't), 601.2h's "partial payments are not
  allowed", 605.3a (mana abilities may be activated "whenever a rule or effect
  asks for a mana payment, even if it's in the middle of … resolving"), 107.3f
  (an X in text and in no cost is chosen "as it resolves"), 101.4 (several
  players choosing at once choose in APNAP order), 702.21a–b (ward is a
  trigger whose effect is a CR 118.12a payment, its X read as it resolves).
- **CR 601.2d and 608.2d's last sentence**: a division among targets is
  announced with the targets, each target receiving at least one; a division
  among objects not targeted is chosen as the effect applies. **Watching
  it**: 115.7f (a change of targets cannot change the division), 608.2b (an
  illegal target is not dealt its share, and the division stands; Arc
  Lightning's ruling).
- **CR 601.2h** ("in any order") and **613.7m** ("in the order of that
  player's choice"): orders the rules hand a player.
- **CR 608.2d and 608.2e**: choices not made while casting are made as the
  effect applies; with several players, in APNAP order, step by step. The
  values the CR names: a color (105), a creature or card type (205), a card
  name (201.4: one in the Oracle card reference, never a token's alone), a
  number, a player, a word with no rules meaning (607.2f), a pile (700.3), a
  vote (701.38: each player in turn order from a named player; 701.38d's
  several votes at once).

### The printed cards

Each family is counted by its clause, never a word anywhere in the text:
reminder text stripped, a mode only under a bulleted list, an "unless" only
when a player's action follows it. One card can sit in several rows. Every
family under 40 and a sample of each larger one was read card by card.

| Family | Cards | `tdc` | Example |
|---|---:|---:|---|
| **Modal**, any | 745 | 15 | Abrade |
| …a spell's modes (spree, pawprints and tiered included) | 507 | 9 | Prismari Command |
| …a triggered ability's modes (603.3c) | 209 | 6 | Junji, the Midnight Sky |
| …an activated ability's modes | 29 | 0 | Bow of Nylea |
| "Choose one" (tiered included, CR 702.183a) | 575 | 11 | Atsushi, the Blazing Sky |
| "Choose two", "three" | 50 | 1 | Prismari Command |
| "Choose one or both" | 57 | 0 | Artful Takedown |
| "Choose one or more" (spree included, CR 702.172a) | 42 | 2 | Casualties of War, Sublime Epiphany |
| "Choose any number" (zero included, Rankle's ruling) | 2 | 0 | Rankle, Master of Pranks |
| "Choose up to N" | 12 | 0 | Hullbreaker Horror |
| a count that is X ("choose X", "choose up to X, where X is …") | 4 | 0 | Doomsday Confluence, Riku of Many Paths |
| a count the player may raise on a condition ("if you control a commander as you cast this spell, you may choose both instead") | 19 | **5** | the Will of the [clan] cycle |
| a count a condition raises (teamwork, kicked, delirium: "choose both instead") | 13 | 0 | Prophetic Titan, Inscription of Abundance |
| zero or exactly N ("you may choose two") | 1 | **1** | Shadrix Silverquill (ruling: "exactly zero modes or two modes") |
| the same mode more than once (700.2d) | 22 | 0 | Season of the Burrow; Moment of Reckoning (up to four, of two modes) |
| pawprints (700.2i) | 5 | 0 | Season of the Burrow |
| a mode chosen at random | 5 | 0 | Summon: Magus Sisters |
| another player chooses the mode (700.2e) | 3 | 0 | Fatal Lore |
| a mode with its own target (700.2c, f) | 591 | 12 | Abrade |
| "each mode must target a different player" (item 154) | 7 | **1** | Shadrix Silverquill |
| "choose one that hasn't been chosen this turn" | 11 | **2** | Parapet Thrasher, Teval's Judgment |
| "choose one that hasn't been chosen", for as long as the object exists | 11 | 0 | Demonic Pact |
| a value not chosen before by this object's ability | 2 | 0 | Garth One-Eye's card name, The Toymaker's Trap's number |
| an object chosen earlier in this resolution, excluded | 2 | 0 | Fortunate Few, Rejoin the Fight |
| a cost per mode: spree (700.2h), tiered | 21, 7 | 0 | Final Showdown; Fire Magic |
| entwine, escalate | 32, 9 | 0 | Barbed Lightning; Collective Defiance |
| **"Unless [a player] [acts]"** (118.12a) | 427 | 1 | Compulsive Research |
| …pays | 292 | 0 | Mana Leak |
| …another action (sacrifices, discards, returns, …) | 137 | 1 | Contamination, Compulsive Research |
| …the payer named by the text ("its controller", "target player", "they"), not "you" | 232 | 1 | Mana Leak; Compulsive Research's target player |
| …"unless any player pays", "any player may" (asked of each, 101.4) | 18 | 0 | Rhystic Study |
| ward, printed; its cost not mana alone; granted | 190; 52; 45 | 1; 0; 0 | Canopy Gargantuan |
| echo, cumulative upkeep (the same payment as keywords) | 130 | 0 | Albino Troll; Arctic Wolves |
| **a cost paid at resolution**, "you may pay …. If/When you do" | 460 | 2 | Spit Flame, Leyline Tyrant |
| …by a player the text names ("its controller may pay") | 21 | 1 | Gix, Yawgmoth Praetor |
| …its amount chosen as it is paid ({X}, X life, any amount; 107.3f), never fixed by the text | 46 | 1 | Leyline Tyrant |
| *not at resolution*: a payment as a permanent enters ("As this land enters, you may pay 2 life"), a replacement (614.12) | 29 | 0 | Hallowed Fountain |
| *not this phase*: a "can't" with a price ("can't attack you unless their controller pays") | 35 | 1 | Ghostly Prison: a cost to attack, CR 508.1h, `combat-requirements`' |
| **Divided among targets** (601.2d) | 113 | **3** | Dragonlord Atarka (a trigger), Magma Opus, Skarrgan Hellkite (an activation) |
| …among objects not targeted (608.2d) | 8 | 0 | Feast of the Victorious Dead, Invoke Justice |
| …prevention divided as you choose (item 92) | 5 | 0 | Remedy |
| **An order a card hands a player**, a library's "in any order" | 171 | 1 | Ponder |
| objects entering at once (613.7m) | 64 | 1 | Living Death |
| …of several players at once, each ordering their own in APNAP order (101.4) | 10 | 1 | Living Death |
| CR 601.2h's payment order: no phrase; a shape read off the cost (decision 6) | — | — | Omnath, Locus of Mana with Momentous Fall (ATOM-601.2h-003) |
| …a creature whose power or toughness reads the pool or the life total, so a payment beside its sacrifice changes what it leaves | 8 | 0 | Omnath, Serra Avatar, Death's Shadow |
| **A value chosen as the effect applies** (608.2d): a color | 26 | 0 | Brave the Elements |
| …a creature type; a card type; a basic land type | 54; 7; 15 | 0 | Bloodline Bidding; Blood Oath; Elsewhere Flask |
| …a card name (201.4) | 46 | 0 | Brain Pry |
| a card name narrowed by what the card is ("nonland", "creature", "other than a basic land"; 201.4a), at resolution or as it enters | 27 | 0 | Cabal Therapy, Meddling Mage |
| …a number (including "secretly") | 12 | **1** | Expel the Interlopers |
| …a player, not targeted (including "at random") | 38 | **1** | Territorial Hellkite |
| …a word with no rules meaning (607.2f) | 17 | 0 | Extinction Event's "odd or even" |
| …a type, keyword or color out of the few the card prints ("land or nonland", "hexproof or indestructible") | 9 | 0 | Winding Way, Linvala, Shield of Sea Gate |
| …a pile (700.3) | 21 | 0 | Fact or Fiction |
| …a vote (701.38) | 33 | **1** | Selvala's Stampede |
| the same values chosen **as the permanent enters** (stored for a linked ability, CR 607.2d), "enters under the control of an opponent of your choice" included | 202 | 3 | Temple of the Dragon Queen, Reflections of Littjara, Frontier Siege |
| the same values chosen **as an additional cost to cast it** (601.2b, paid at 601.2h): a creature type; a number | 2; 1 | 0 | Caller of the Hunt; Liquid Fire |
| the same values chosen **while casting, because the targets read them** (601.2c: "target creature cards of the creature type of your choice") | 2 | 0 | Aphetto Dredging (its ruling: "choose the creature type before choosing the targets") |
| an object chosen as an additional cost to cast it, not targeted | 2 | 0 | Close Encounter, Monstrous Emergence |
| *not this phase*: a value chosen in a mana ability ("{T}: Choose a color. Add …", 605.3b), the mana model's | 12 | 0 | Nykthos, Shrine to Nyx |
| *other times*: before the game, while drafting, as it attaches or transforms, during an untap step, or as a replacement or prevention applies | 14 | 0 | Paliano, the High City; Sanctuary Blade; Pulse of Llanowar |

Notes on the table:
- **738 modal cards, not the roadmap's 915.** The roadmap's number is the
  Forge-script card census (`plans/references/card-census/`), which counts
  each card's needs by Forge's own names. The clause count here reads the
  bulleted list under a "choose" instruction, and was read card by card on
  the 15 `tdc` hits. `tdc`'s sixteenth bulleted card, Frontier Siege, is
  "choose Khans or Dragons" as it enters, CR 607.2m's anchor words: a value
  as it enters, not a mode.
- The "as it enters" row is this phase's option shape and `linked-abilities`'
  record (step 6): the value is chosen through the same kind, and stored on
  the permanent for "the chosen color" by that step's design.
- Ponder's "put them back in any order" is the row's one `tdc` card; of the
  171, 111 put cards on the bottom of a library in any order, 24 on top
  (Brainstorm), and 36 back in any order.
- **"Hasn't been chosen" has two scopes, read in review.** The first count
  put all 26 under "this turn"; 11 are, 11 remember for as long as the object
  exists (Demonic Pact's ruling: "refers only to that specific Demonic Pact",
  whoever controls it), 2 choose a value rather than a mode, and 2 exclude
  an object chosen earlier in the same resolution. Decision 2 keeps both
  scopes in one record.
- **Every row was read again in review**, against what its words claim:
  every hit of every family under 40, two seeded samples of each larger one
  (`--sample 7` and `--sample 8`), and each hit's matching clause read in
  full across the whole family. The census's predicates were fixed where a
  row held what its words do not: a value chosen in a mana ability, before
  the game or as a permanent attaches (no resolution), a payment as a land
  enters (a replacement), an X the text fixes, a condition read as an
  "unless" ("unless you've cast"), the seven tiered cards and spree's and
  pawprints' heads, which no modal row held, and 13 count raises with no
  "you may". The corrected counts are the table's.
- **A value or an object can be chosen while casting**, as an additional cost
  (5 cards). The value is decision 7's kind asked at 601.2b; the object is a
  cost paid at 601.2h, the cost model's (`remaining-costs`), not decision 8's
  `Choose`.

## The decisions

Decisions 2 and 3 are modes; 4 is a cost paid at resolution; 5 the division;
6 the two orders; 7 and 8 a choice made as an effect applies. Decision 1 is
where all of it is written.

### Decision 1 — where the design lives

**The problem.** The design spans the cast's announcements (CR 601.2b–d), the
payment's order (601.2h), the resolution's choices (608.2d, 118.12a), a
replacement-shaped performer (613.7m) and the decision boundary. `CLAUDE.md`'s
architecture row says a new subsystem extends the row, never adds a row.

- **A. A doc of its own, `choices-architecture.md`**, on the row: "choices"
  (CR 700.2/118.12/601.2b–d/608.2d/701.38/201.4 + the two orders). It owns
  the choice kinds, the mode model, the payment at resolution and the
  division, and cites `setup-architecture.md#7e-pausing-at-a-decision` for the
  boundary's rule and the log's format. *Upkeep*: one more doc for
  `engine-map.md` to route through, and one more on the row.
- **B. `setup-architecture.md` gains a section 7f**, after "7e. Pausing at a
  decision". The boundary's doc already owns
  the log, the matcher and decision 4. *Upkeep*: setup's doc becomes the
  rules home for modes and costs, which it is not otherwise: its subject is
  the scenario, the save and the boundary, and a reader of CR 700.2 would not
  look there.
- **C. Split by subject**: modes and the division in `cost-architecture.md`
  (beside 601.2f–h), the resolution's choices in `triggers-architecture.md#6-resolution`
  (beside "may"), the orders where each performer is. *Upkeep*: four docs
  own one boundary's kinds, and a kind's shape is in one doc while its rule
  is in another.

**Decided in review: A.** The phase is a subsystem by the row's own test: it
owns CR sections no doc owns (700.2, 118.12a's payment, 608.2d's values,
701.38, 201.4), and every later choice kind (`linked-abilities`' stored
values, `resolution-recipients-and-amounts`' recipients, `permission-design`'s
offers) cites one place for the kinds' shapes. Its phases are named, not
coded (`engineering-practices.md#naming-a-phase`).

### Decision 2 — how modes are offered, and how an answer names them

**The table** is the census's modal rows. What the shape must do, grouped by
what the shape must do:

| The shape must express | Cards | `tdc` | What holds it today |
|---|---:|---:|---|
| a fixed number of distinct modes, and nothing else on this table | 546 | 7 | `ModalCount::Exactly` |
| a range ("one or both", "one or more" and spree, "up to N", "any number", zero included) | 113 | 2 | `UpTo`, `Any` (no lower bound) |
| zero or exactly N, never between | 1 | 1 | nothing |
| a count the player may raise on a condition read as the modes are chosen ("you may choose both instead") | 19 | 5 | nothing |
| a count a condition raises ("choose both instead": teamwork, kicked, delirium) | 13 | 0 | nothing |
| a count that is X ("choose X", "choose up to X") | 4 | 0 | nothing |
| a mode more than once (700.2d), as often as the count allows, past the number of modes (Moment of Reckoning: up to four, of two); a mode's copies ordered by their targets (Moment of Reckoning's ruling) | 22 | 0 | nothing |
| a budget the modes spend (pawprints) | 5 | 0 | nothing |
| a mode's own cost (spree, tiered) and one cost per extra mode (escalate), all modes for a cost (entwine) | 69 | 0 | nothing (item 81) |
| a mode this object's ability chose earlier this turn is not offered | 11 | 2 | nothing |
| a mode this object's ability chose at any time is not offered (Demonic Pact) | 11 | 0 | nothing |
| another player chooses; the game chooses at random | 3; 5 | 0 | nothing |

**The problem.** A mode is chosen once per cast (or activation, or
placement), among options whose legality is per mode (700.2a: a mode whose
targets cannot be chosen cannot be chosen) and whose count rule is one of the
rows above. Under #306's decision 2b an answer names what it chose; the
question is what one option is.

- **A. One pick over the modes**, `ChoiceOption::Mode(n)` with n the mode's
  printed place, through `pick_n` with the count's bounds. Covers the first
  two rows. The others each need more: zero-or-two is not a range, so it needs
  a rejection and a re-ask; a repeat needs a second question (an allocation
  of the count among the modes picked); pawprints a budget `pick_n` cannot
  check, so a third. *Code*: smallest for "choose one"; three prompt shapes
  for the rest. *Upkeep*: every new count rule is a new prompt or a new
  rejection.
- **B. One pick over the legal sets of modes**,
  `ChoiceOption::Modes(Vec<ModeIndex>)`: each option is one whole choice (a
  set, or with repeats a multiset in printed order), listed only if the count
  rule allows it and each of its modes is legal. One `pick_n` of exactly one,
  for every row. Zero-or-two lists the empty set and the pairs; a raised count
  lists the pairs only when the condition holds as the modes are chosen (Will
  of the Abzan's ruling); pawprints list the multisets within the budget; a
  mode already chosen is not listed. *Size of a list*: "choose one of three"
  is three options. The largest printed lists all allow repeats: Unite the
  Coalition's 126 (five of five modes), Planewide Celebration's 35 (four of
  four); then Casualties of War's and Sublime Epiphany's 31 (one or more of
  five), Rootcast Apprenticeship's 20, a Season's 16 and Moment of
  Reckoning's 15 (none to four, of two), each counting the empty choice where
  the count allows it. Doomsday Confluence's grow with X (the sets of that X
  only, C(X+2, 2) among three modes: 21 at X = 5, 66 at X = 10). *Code*: one
  enumerator over the count rule, ~120–180 lines, one prompt shape. *Upkeep*:
  a new count rule is a new arm of the enumerator, never a new prompt.
- **C. One pick per mode, in a loop**: "add a mode", re-offering only the
  modes that still lead to a legal whole choice, and "stop" beside them once
  the modes so far are one. The loop ends without asking when the count's
  most is reached, so "choose one" is one question and "choose two" two.
  *Code*: the same enumerator, run again at every pick to find what still
  leads somewhere, and a kind that carries the picks so far. *Upkeep*: the
  order of the picks, which the rules do not have, is in the log, and one
  set is reached by several orders (6 for three distinct modes, 126 sets by
  up to 120 orders each on Unite the Coalition).

**Performance.** A, B and C ask one question for every printed "choose one",
which is 77% of modal cards; for "choose two" or more, B asks one and C one
per mode. B allocates its options per modal announcement: a few small `Vec`s
for every fixed count, 126 at the widest printed list, and for a count of X
the sets of that X only. Nothing is on `GameState`.

**Castability runs the same enumerator, stopping at the first legal choice.**
Whether a modal spell can be cast (700.2a, 601.2c), a modal ability activated
(602.2b), or a modal trigger kept on the stack (603.3c) is one question:
does any choice the offer would list exist, at a cost the payer can pay.
Asking that with a shortcut is where an offer and a cast disagree, which
`cost-architecture.md#36-enumeration-must-agree-with-enforcement` rules out
for costs. The per-mode check
alone covers most of it, but not all:
- **Per mode**: a mode whose instances of "target" cannot all be chosen is
  dropped first, by the check `can_cast` already makes for each instance
  (`oracle/mana_helpers.rs:415`), run over the mode's own list;
- **The count over what is left**: a fixed count needs that many modes left,
  or any one left when it may repeat (a repeated mode may take the same target
  each time, 700.2d); a count that reaches zero (Rankle, Moment of Reckoning,
  a Season, Shadrix's "you may choose two") always has the empty choice;
- **What no per-mode check sees**: Shadrix's and its six siblings' "each mode
  must target a different player" (in a two-player game, two modes that each
  target an opponent cannot both be chosen), and a mode's own cost (spree,
  tiered, escalate: the base cost may be payable when no legal set's total is).
The enumerator holds all three, so castability asks it for its first legal
set and, where modes carry costs, checks each legal set's total until one is
payable. *Cost*: for a node with no per-mode cost, one pass to the first
legal set, which for "choose one" is the first mode with its targets; the
worst case, no legal set, walks every candidate the dropped modes leave, at
most the list's length. Once per modal card in hand per priority pass; the
fuzz's reversal count (`Actions reversed`) measures any over-offer that
remains.

**What the identity is, and the kind.** A mode's printed place is its
identity, not a position in the offered list: the card fixes it, and an
illegal mode dropped from the list does not renumber the others. It is one
type, `ModeIndex`, in every place a mode is named: `StackEntry.chosen_modes`
(`Vec<usize>` today) and `Condition::ModeChosen` (`usize` today, 8 uses) both
move to it. The question is `ChoiceKind::ChooseModes { source }`, one `pick_n`
of exactly one over `ChoiceOption::Modes(Vec<ModeIndex>)`. The log writes
`modes 1, 3`, `modes 2, 2` for a mode chosen twice, and `no modes` for the
empty set; a script names `Modes(vec![1, 3])`, found wherever a build lists it
by derived equality (`setup-architecture.md#7e-pausing-at-a-decision`,
decision 4b's typed matcher). A client draws each option as its modes' text,
read off the ability as the layers give it.

**The modal node's shape**, which B's enumerator reads, from the table and
not asked. `Effect::Modal` carries four things:
- **the count**, one enum whose arms are the table's count rules:
  `Exactly(n)`; `Range { fewest, most }` ("one or both" is 1 to 2, "one or
  more" 1 to all, "any number" 0 to all, "up to N" 0 to N; where modes may
  repeat, `most` is the printed number even past the number of modes, as
  Moment of Reckoning's none to four of two, and where they may not, a count
  past the number of modes is capped at it); `NoneOr(n)`
  (Shadrix's zero or two); `X`; `Pawprints(budget)`; and `RaisedIf { usual,
  raised, condition }` around any of them (the Will cycle; Prophetic Titan's
  delirium), the condition read once as the modes are chosen (Will of the
  Abzan's ruling). Its two printed forms are its raised count: "you may
  choose both instead" raises `Exactly(1)` to `Range { 1, 2 }`, and "choose
  both instead" to `Exactly(2)`;
- **the chooser**: the controller, an opponent (700.2e: with more than one,
  the controller first picks which, through decision 7's `ChoosePlayer`), or
  at random from `GameState.rng`, which asks nothing;
- **three rules of the node**, each a flag a plain "choose one" leaves off: a
  mode may repeat (700.2d), each mode's targets are different players (item
  154's eight-card shape, decision 3), and a mode this object's ability chose
  before is not offered, either this turn or for as long as the object exists
  (the record below);
- **the modes**, each a `Mode` holding its effect, its pawprint weight (1
  unless printed), its own cost (700.2h; none unless printed) and its
  instances of "target" (decision 3).

A plain "choose one" is `Exactly(1)`, the controller, no flags, and modes of
weight 1 and no cost. A mode's own cost is a `Mandatory` additional cost
assembled from the chosen modes (item 81's resolution), in the total at
601.2f.

**What an ability has chosen before**, the record both "hasn't been chosen"
scopes read (22 cards) and decision 7's two values that remember (Garth
One-Eye's card name, The Toymaker's Trap's number). One map on
`GameHistory`, `chosen_by_ability: IdMap<AbilityIdentity, ChosenBefore>`,
keyed as `resolutions_this_turn` is: `AbilityIdentity` carries which
existence of its object the ability is on (CR 400.7) and which grant, so a
second Demonic Pact, a copy, or the same card back from a zone starts empty,
and a control change keeps the record (Demonic Pact's rulings: "refers only
to that specific Demonic Pact", and a new controller "can choose only the
third or fourth mode"). An entry holds the turn it was last written and what
was chosen: the modes, or the value as its `ChoiceOption`. The node's flag
says which scope it reads. "This turn" reads the entry only if its turn is
now, and a write on a later turn starts it over; "hasn't been chosen" reads
it whole. It is written as the modes are announced, since a countered
ability's mode still counts as chosen (Demonic Pact's ruling), and it is
never swept: an entry whose object no longer exists is never asked for
again. *Cost*: one map lookup per announcement of such an ability, never a
walk of the history log; the map is empty in a game without one of these
cards, and an empty map clones without a heap allocation.

The record is not one of the turn's rows, which a turn's start clears, since
the longer scope must outlive the turn. Two other homes were weighed: two
maps (a turn's map cleared with the others, and a map for the object's
lifetime) give each scope its own map, at the price of two writers for one
fact; a field on the object ends with the object by itself, but
grows every object's clone for a fact 24 printed cards use, and an object
with two such abilities (a copy, a grant) still needs the map's key inside
it.

**The keywords on the count wait for `breadth-features`** (after v0.5): spree,
escalate, entwine and tiered are sibling keywords on CR 700.2
(`engineering-practices.md#gaps-a-phase-finds`), none is in `tdc`, and each is
the node above plus its keyword's cost rule. The second PR builds 700.2h's
per-mode cost with a fixture card printing one.

**2b, settled in review: for a count of X, X is asked first.** CR 601.2b
announces the modes and the value of X in one step, so the engine may ask
either first, and Doomsday Confluence's sets cannot be listed before X is
known. X is asked from MA-2's legal values (the values the payer can pay,
`mana-architecture.md`), then the sets of that size. The note had read
601.2b's sentence order as a sequence; it is one announcement, so this is
the engine's order of two questions, not a rules question.

**B against C, read again in review.** C is how a person picks modes, one at
a time; B is what the rules ask for, one whole choice. The two come apart at
the boundary. B's list holds everything C's picks could reach, so a client
draws B as C: a checkbox per mode, or a count per mode where modes repeat,
each enabled only while some listed set still holds the picks so far, and
Confirm enabled when the picks are themselves a listed set. That logic is
plain Rust over the offered list, under tests, and asks the engine nothing
more (`engineering-practices.md#the-dev-gui`). C's offer at one pick cannot
be turned back into B's list without replaying the engine's loop. An AI
harness gets a flat list under B, can split it into per-mode steps itself
(an option's modes are its content), and searches each set once, where under
C one set is reached by every order of its picks.

**Decided in review: B.** It is the only option where every printed count rule
is one question of one shape, and it makes 700.2a's "can't be chosen" a
property of the list rather than a rejection. Its lists are 126 at the widest
printed (Unite the Coalition) and grow with X only for the one card whose
count is X. A client can still draw it as C's one-at-a-time picks, and an AI
harness reads a flat list of whole choices directly.

**For the GUIs** (the owner, in review): a person should pick modes the way C
asks, one at a time, so the dev GUI draws `ChooseModes` as the
checkbox-or-count picker above, its logic plain Rust over the offered list
under tests, in the dev GUI PR that follows `modal-spells-and-abilities`. The
GUI built for players (Phase 10, `backlog.md`'s "2.38" entry) starts from
that logic. It is the feel to emulate, not a layout: the player's GUI decides
how it looks.

### Decision 3 — where the chosen modes live, and how targets and resolution read them

**The problem.** CR 601.2b chooses modes before 601.2c announces targets, and
an unchosen mode announces nothing (700.2c). The stored instance lists
(`AbilityDef::instances`, `CardData::spell_instances`, built once per card so
castability never walks the tree) see a `Modal` node as declaring nothing
(item 153). `StackEntry.chosen_modes` is `Vec<usize>` with no writer (item
31), the resolution's `Modal` arm is an error (`engine/resolve.rs:521`), and a
copy keeps the original's modes (700.2g, `spell-copies`).

- **A. The printed effect stays on the entry, and the chosen modes are its
  record.** `chosen_modes` holds the chosen modes in printed order (repeats
  adjacent, 700.2d); the resolution's `Modal` arm walks those modes and no
  others. **The per-mode instance lists live in the modal node**, one on each
  `Mode`, filled by `CardDataBuilder::build` in the pass that fills
  `AbilityDef::instances`. The tree holds instance lists in six places (the
  card, `objects/card_data.rs:57` and `:182`; the delayed trigger's template,
  `types/triggers.rs:1937`; the pending trigger, `:2190`; the dispatch,
  `engine/triggers/dispatch.rs:604`; the trigger's state,
  `engine/triggers/state.rs:44`), and each keeps its list as it is, stopping
  at the modal node; each holds or reaches the effect, so the cast, the
  activation and placement all read a chosen mode's list through the node and
  none gains a field. The announcement concatenates the chosen modes' lists
  after the node's own and never walks the tree. **A mode is its own instance scope**: a back-reference
  inside a mode (`EffectRecipient::SameInstanceAs`) counts from the mode's
  first instance, since no printed mode refers to another mode's target, and
  a repeated mode's second copy announces its own targets.
  *Clone cost*: none: `chosen_modes` is already on `StackEntry` (24 bytes,
  no heap when empty), and holds a `ModeIndex` (one byte) per chosen mode.
- **B. The entry's effect is replaced by the chosen modes at announcement**:
  a new `Effect::Sequence` of the chosen modes, so resolution and the
  instance walk need no mode logic. `chosen_modes` is kept for readers (a
  copy, a client). *Clone cost*: a fresh `Arc<Effect>` and a deep copy of the
  chosen modes' trees at every modal announcement. *Upkeep*: the effect on the
  stack is no longer the card's text, so a client drawing the stack object,
  the log and a copy each read the record rather than the effect, and the
  instance indices still need A's per-mode scope.

**Item 154 with it, in two halves.**
- **"Another target" within one scope** (Arc Trail's "any target and …
  another target", CR 115.4) cannot exclude a player today, because the
  exclusion is an `ObjectFilter` leaf (`OtherThanInstance`) and a player or
  "any target" filter carries none. The leaf is deleted and its 25 uses
  re-pointed to one wrapper any filter takes, `SelectionFilter::OtherThan {
  filter, instance }`; the two enumeration arms that push players without
  asking `validate_selection` learn to ask it. One spelling of "another
  target", ~60–90 lines.
- **"Each mode must target a different player" across modes** (Shadrix
  Silverquill and 6 more) is a rule of the modal node, the flag above: the
  card writes it once on the node, never an instance number, since which
  instances exist depends on the modes chosen. The announcement keeps a set
  of the players the chosen modes' earlier instances took and offers each
  later player-targeting instance the rest.

**Recommendation: A.** It keeps the effect on the stack the card's text,
which the log, a client and a copy all read, and it costs nothing a clone
carries. B buys a simpler resolution arm for a deep copy at every modal
cast and a second source of truth.

**A cast under A, step by step** (asked in review). Prismari Command
({1}{U}{R}, "Choose two —": 2 damage to any target; target player draws two,
then discards two; target player creates a Treasure; destroy target
artifact), with no artifact on the board:
1. At priority, castability asks the enumerator for a first legal pair:
   mode 4 is dropped, since no artifact can be targeted, and modes 1 and 2
   are a pair, so the card is offered.
2. CR 601.2a: the card moves to the stack, its effect the printed `Modal`
   node with all four modes.
3. 601.2b: `ChooseModes` lists `modes 1, 2`, `modes 1, 3` and `modes 2, 3`.
   The player answers `modes 1, 3`, and `chosen_modes` is `[1, 3]`.
4. 601.2c: the announcement reads the node's own instances (none), then mode
   1's list (any target), then mode 3's (target player), and asks them in
   that order. The entry holds two `TargetInstance`s, mode 1's first.
5. 601.2f–i: the cost is the printed one, paid through the window, and the
   spell becomes cast.
6. Resolution: CR 608.2b re-checks both instances. The `Modal` arm walks
   `[1, 3]` in printed order, giving each mode the instances that follow the
   previous mode's: which instances are whose is computed from
   `chosen_modes` and the node, never stored. If the creature has gained
   hexproof, mode 1 does nothing and mode 3 still makes the Treasure (its
   ruling: it does "as much as it can").
7. A copy copies `chosen_modes` and the instances (707.10); Redirect changes
   targets within the instances and never the modes (700.2f).

Doomsday Confluence ({X}{X}{B}, "Choose X. You may choose the same mode more
than once": each player sacrifices a nonartifact creature; create a 3/3 Dalek
with menace; each opponent discards a card):
1. Castability asks at the least X, as for any X spell
   (`oracle/mana_helpers.rs:110`): at X = 0 the choice is empty, which no
   mode's target can make illegal, so the card is offered when {B} is
   payable.
2. 601.2a: to the stack.
3. 601.2b: X first (2b), `ChooseXValue` over the values the player can pay;
   say 3. Then `ChooseModes` over the ten multisets of three among three
   modes; the answer `modes 1, 2, 2` makes `chosen_modes` `[1, 2, 2]`, a
   mode's copies adjacent in printed order.
4. 601.2c: no mode targets, so nothing is announced.
5. 601.2f: the total is {3}{3}{B}; paid.
6. Resolution: the arm walks `[1, 2, 2]` as though the modes were printed in
   that sequence (700.2d): each player sacrifices, then two Daleks.

With targets, a repeat announces one instance list per copy: Moment of
Reckoning's `modes 1, 1, 2` asks mode 1's target twice, the same permanent
allowed both times (700.2d), and the order the two were announced is their
order at resolution (its ruling).

**Once its modes are chosen, a modal spell resolves as the spell printed
with only those modes**, in printed order and a repeated mode as many times
as chosen (700.2d's "as if that mode appeared that many times in sequence"),
which is the `Modal` arm walking `chosen_modes` as a sequence. Everything
else is any spell's: CR 608.2b's re-check, and its rule that a spell whose
every target is illegal does not resolve, which takes a chosen mode with no
target down with it, as it takes the "draw a card" of "Destroy target
creature. Draw a card." Under A the unchosen modes stay on the stack object
only as text, for the log, a client and a copy; nothing at resolution reads
them.

### Decision 4 — a cost paid as an effect resolves: "unless", "you may pay", ward

**The table**: the census's "unless" and payment rows. Grouped by what the
shape must do:

| The shape must express | Cards | `tdc` | Held today |
|---|---:|---:|---|
| a payment by the ability's controller, optional, then a branch on it ("you may pay {1}. If you do") | 460 | 2 | `Effect::Optional`, but no payment: an atom acts, it does not pay |
| …by a player the text names ("unless its controller pays", "its controller may pay") | 232 + 21 | 1 + 1 | `Optional`'s chooser names another player; no payment |
| …asked of every player in turn until one pays ("unless any player pays") | 18 | 0 | nothing; `triggers-architecture.md#62-may-and-unless-cr-6035` files it "with its first card" |
| a payment that is not mana (sacrifice, discard, return a land, life) | 137 | 1 | `Cost`'s arms exist for casting; `plan_payment` asks the choices |
| an amount chosen as it is paid ({X}, X life, any amount), the text not fixing it | 46 | 1 | the legal values of X (`mana-architecture.md`'s MA-2), at cast only |
| ward: a trigger on becoming a target whose effect is the payment, its X read at resolution | 190 + 45 granted | 1 | the "becomes the target" trigger event (built with targeting's records, TR-5c); no payment |
| *not this decision's*: a payment as a permanent enters ("As this land enters, you may pay 2 life"), a replacement (CR 614.12) | 29 | 0 | `EntryOffer::Pay`, `AcceptAsObjectEnters`' offer |

**The problem.** CR 118.12a says "[do A] unless [P does B]" means "[P] may
[do B]. If [P doesn't], [do A]", and B is a cost paid as the effect resolves.
The resolution has `Effect::Optional { chooser, effect }` and its answer
(`CostAnswer`: `Does`, `Doesnt`, `Cant`;
`triggers-architecture.md#62-may-and-unless-cr-6035`), and
casting has the payment (`plan_payment`, the 601.2g window, `pay_costs`). What
is missing is a payment inside a resolution, and where "unless" is spelled.

- **A. CR 118.12a's own reading: a `Pay` instruction inside `Optional`.** A
  new `Primitive::Pay { costs }` pays as the resolving instruction's player,
  through the casting path's `plan_payment` and `pay_costs`, with CR 605.3a's
  mana window opened first (ATOM-605.3a-002), and answers `Does` when paid.
  "Unless P pays C, do E" is written
  `Sequence[Optional { chooser: P, Pay(C) }, Conditional(Doesnt, E)]`, which
  is the CR's sentence. "Any player may" is `Optional`'s chooser reaching
  each player in APNAP order (CR 101.4) until one pays. An X is chosen as it
  is paid (107.3f) from the legal values of X against what the payer can
  produce. *Code*: the primitive, the window at resolution, the chooser walk
  and X, ~250–400 lines; a builder (`Effect::unless(payer, costs, effect)`)
  so a card reads as its text.
- **B. A combinator, `Effect::Unless { payer, costs, effect }`.** The
  resolver asks and pays in one arm. *Upkeep*: a second spelling of what
  `Optional` and `Conditional` already say, and "you may pay {1}. If you do,
  …" (510 cards) is still A's shape, so both exist.

**What both settle.**
- **An unpayable offer is "doesn't".** Offered only when payable (CR 118.3),
  as a shockland's payment is. A payer who cannot pay is not asked, and the
  `Optional` answers `Doesnt`: CR 118.12's clause checks whether the player
  chose to pay an optional cost, and one who could not did not. So Mana Leak
  counters a tapped-out player's spell. `Cant` stays what `CostAnswer`'s doc
  says it is, a mandatory action that could not start.
- **The kinds.** The offer is `ChoiceKind::PayAsItResolves { source, offer }`,
  `offer` the costs as `AcceptAsObjectEnters`' `EntryOffer::Pay` carries one,
  so a client can tell a payment from a gift; it is a take or leave
  (`ask_take_or_leave`), logged and scripted as `ApplyOptionalEffect`'s is,
  asked of the payer. An amount chosen as it is paid is `ChoiceKind::ChooseAmountToPay
  { source, values }`, `ChooseXValue`'s shape (a `pick_number` over an
  interval, or `Number` options when a rule cuts a gap) with the resolving
  object as its subject, since `ChooseXValue`'s `spell_id` names a spell
  being cast.
- **A window that ends with the cost unpaid is the payer declining**: the
  mana stays in the pool until the step ends (CR 106.4, 500.5), as on paper,
  and nothing is reversed, since CR 732.1's reversal is a cast's or an
  activation's. The payment is CR 608.2c's, "in the order written", so it is
  not routed through `payment_order_rank`, whose doc already says why.
- **Ward is a keyword on this base and builds whole with it**: its trigger is
  the "becomes the target" event, its effect "counter that spell or ability
  unless that player pays [cost]" (702.21a), its X read as it resolves
  (702.21b). Echo and cumulative upkeep are sibling keywords on the same
  payment, none in `tdc`, and wait for `breadth-features`.

**Decided in review: A.** It is the CR's own sentence, it adds one primitive
where B adds a combinator beside it, and every row of the table is it with a
different chooser or cost. Item 240's upkeep halves (Contamination, Infernal
Darkness: "sacrifice this unless you sacrifice a creature") become writable.

### Decision 5 — where a division lives

**The table**: 114 cards divide among targets (3 in `tdc`: one cast, one
activation, one trigger, so 601.2d, 602.2b and 603.3d all announce a
division), 10 among objects not targeted, 5 divide prevention.

**The problem.** CR 601.2d announces a division with the targets, at least
one to each; 608.2b keeps it when a target goes illegal (that target is not
dealt its share); 115.7f keeps it across a change of targets; 608.2d chooses
an untargeted division as it applies. `backlog.md`'s "2.20 Several target
clauses on one spell" sized the targeted half: a divided `TargetCount`, a
parallel allocation, a per-target amount, ~500–700 lines with Arc Lightning.
A change of targets (115.7f) keeps each share with the target that replaces
its old one, so whatever holds the shares must move with the targets.

- **A. Paired with the targets, in one list.** `TargetInstance.chosen` becomes
  one enum, `Targets(Vec<ObjectOrPlayer>)` or `Divided(Vec<(ObjectOrPlayer,
  u64)>)`, read through a `targets()` accessor by the ~16 readers of
  `.chosen`; the atom's amount reads its target's share off the pair. A change
  of targets rewrites the pair's target and keeps its share. *Clone cost*:
  none for an undivided instance, the same `Vec` it holds today.
- **B. A parallel list**, `shares` on the instance or `divisions` on the
  entry keyed by instance. *Upkeep*: two lists matched by position, which
  every rewrite of the targets (a copy's new targets, 115.7f) must keep in
  step.
- **C. In the amount**, an `AmountExpr` that names a share by target. *Upkeep*:
  the amount reads an announcement it does not own.

**An untargeted division lives in the resolution**: CR 608.2d chooses it as
the effect applies, so it is chosen, applied and gone within the atom, held in
the resolution's walk like any value chosen there, and never on the stack
object. Feast of the Victorious Dead's counters are divided this way.

**The prompt, for both.** `ChoiceKind::DivideAmong { source }` over
`allocate`, its buckets the chosen objects and players (named by what each
is, #306's decision 4), each bucket's least 1, its total the clause's amount.
At announcement it follows the instance's targets (601.2d); at resolution it
follows the "any number of" choice it divides among (608.2d). The log writes
each bucket with its share, as an allocation does today.

**Item 92 closes as a record.** CR 615.7 states a shield on one permanent or
player, whose player or controller chooses which damage it prevents, and
states no count shared across subjects. Each of the 5 printed divided
preventions announces each target's share, so each target gets a shield of
its own share. The refusal stays the answer to the shape the CR does not
state, and the item says so in those words.

**The three options on one cast** (asked in review). Arc Lightning's 3
damage divided as Grizzly Bears 2 and the opponent 1:

| | A, paired | B, a parallel list | C, in the amount |
|---|---|---|---|
| The entry holds | `Divided[(Bears, 2), (opponent, 1)]` | `chosen: [Bears, opponent]`, `shares: [2, 1]` | `chosen: [Bears, opponent]`, and beside it `{Bears: 2, opponent: 1}` for an amount that reads a share by target |
| It resolves | each pair's target takes its share | `chosen[i]` takes `shares[i]` | each target looks up its share |
| Bears has gained hexproof (608.2b) | the re-check drops the pair, and the opponent takes 1 (its ruling: the division stands) | the re-check's survivors (`ChosenTargets`, `engine/targeting.rs:83`) drop Bears, so `shares` must drop its first entry in step, or the opponent takes 2 | the map is keyed by target, so dropping Bears is safe |
| Redirect moves Bears' 2 to Llanowar Elves (115.7f) | the pair's target is rewritten and its share stays: (Elves, 2) | `chosen[0]` is rewritten in place and `shares[0]` stays, right while no rewrite reorders the list | the map still names Bears: every rewrite of a target must also re-key it, or the Elves have no share |
| Reverberate copies it with new targets (707.10c) | the pairs are copied, then rewritten as above | both lists copied, then as above | the map copied, then re-keyed as above |

Under A every writer of the targets keeps the shares right without knowing
they exist, since the share travels in the target's slot, and the re-check's
survivors keep the pairs whole. B adds a rule every writer must keep by
position, and C one every writer must keep by identity.

**Decided in review: A**, the shape the backlog entry sized, with its two lists
made one.

### Decision 6 — when the engine asks an order the CR hands a player

**The problem.** CR 601.2h ("in any order") and 613.7m ("in the order of that
player's choice") hand a player an order the engine fixes today:
`payment_order_rank` pays mana, then what mutates, then what moves an object;
an entry batch timestamps in allocation order. Item 79 and "Before card
breadth" item 4 asked for one rule: ask only where the orders' outcomes
differ, and here only among orders that complete.

**The table.** No printed card names 601.2h's order; it is the rule's, on
every cost of two or more parts. Its shape is read off the cost, and the
census counts the one family a payment's order is visible through: 8
creatures whose power or toughness reads the pool or the life total (Omnath,
whose sacrifice before or after paying the mana draws a different number off
Momentous Fall, ATOM-601.2h-003; Serra Avatar; Death's Shadow). 613.7m's
family is 64 cards that put several objects onto the battlefield at once, 10
of them several players' objects at once, each player ordering their own in
APNAP order (Living Death in `tdc`), and every creation of tokens that are
not alike. A third order of the
same kind is already asked: CR 601.2f's cost reductions, "the player may apply
them in any order", asked whenever two apply (`OrderCostReductions`) though
with today's symbols no order changes the total
(`cost-architecture.md#34-in-any-order-they-choose--the-prompt-the-theorem-and-the-middleware`).

- **A. Ask whenever there are two or more parts or objects.** The CR's literal
  reading, `OrderCostReductions`' policy. *Cost*: every "{1}, {T}:" and
  "{B}, Sacrifice a creature:" activation and every kicked or sacrificing cast
  gains a question whose answers give one game; a person is asked "tap first
  or pay first" at every such activation.
- **B. Ask only where an order can change the game, by a property of each
  part.** Each `Cost` arm states, in one exhaustive match, what paying it
  reads and what it changes: a quantity (the pool, the payer's life, an
  object's counters) or a move (from a zone to a zone); and each `AmountExpr`
  arm states which quantity it reads, which says what a moved object's
  characteristics read. The order is asked when one part changes what another
  part reads: a part that moves an object whose effective characteristics
  read a quantity another part changes (Omnath), or a part that moves into the
  zone another takes from (a discard beside an exile from the graveyard). A
  new `Cost` or `AmountExpr` arm states its reads and writes as it is added,
  so the test grows with the types rather than with a list. The options are
  the orders that complete. 613.7m is asked when two or more of one player's
  objects receiving a timestamp at once each carry something a timestamp is
  read for (below), each player ordering their own in APNAP order.
  *Cost*: a match per part per payment, and per entry batch; a question only
  on such boards.
- **C. As today.** *Cost*: a choice the CR gives a player is the engine's,
  which the CR-first rule does not allow once a printed board reaches it, and
  Omnath's does.

**The kinds.** `ChoiceKind::OrderCostPayments { spell_or_ability_id }` over
`choose_ordering`, each item a part named as decision 4b names a cost (its
kind and its whole cost); two parts alike in every word are one cost twice,
either order the same game, and the matcher takes them in turn.
`ChoiceKind::OrderSimultaneousEntries { player }` over `choose_ordering`, its
items the entering objects by what each is; alike objects never reach it.
A card's own "in any order" (Ponder, 171 cards) is a library order and already
has a kind, `ScryOrder`, built for CR 701.22a's; it is widened to
`OrderLibraryCards { source, bottom }`, a log word the first PR's format change
carries.

**How B is kept right** (asked in review). Skipping a question is right only
if every order the player could choose gives the same game. Skipping every
such question is the goal: a question whose answers all give one game wastes
a person's time and an agent's decision. But when the engine cannot show
that the orders agree, the two ways to be wrong are not alike: an extra
question is waste, which a measurement finds and a later elision removes,
while a skipped question that mattered takes a choice the CR gives the
player, and nothing would show it. So B skips wherever it can show the
orders agree, asks where it cannot, and is checked by running it with the
shortcut off:
- **Every arm states what it reads and changes in an exhaustive match**, with
  no wildcard, so a new `Cost` or `AmountExpr` arm does not compile until it
  says; an arm that cannot say states "everything", which meets every other
  part and is always asked.
- **The argument**: two parts commute when neither reads what the other
  changes and they do not both add to one ordered thing: a library, a
  graveyard (CR 404.2 keeps its order), or the history log's rows of one
  kind, since "the first … each turn" reads which came first.
- **The audit** (`fuzz_games --audit`, beside its trigger-dispatch audit, and
  a test over every registered cost): wherever the property says a payment's
  parts commute, the payment is made in every order on a clone of the board,
  and the boards are compared whole, zones, their order and the history log
  included. A difference fails the audit with its seed.

**The two orders differ in what the audit can see.** CR 601.2h's is closed:
everything a payment changes exists when 601.2h ends, so comparing the boards
then is a complete check for every board the audit meets. CR 613.7m's is not:
a timestamp is read for as long as the object stays in its zone. There the
argument rests on the CR's list of what reads a timestamp, which is short:
the layer order of the continuous effects an object's static abilities
generate (613.7a), the Role rule's "most recent" (303.7a), and stickers'
order (123.5a, which the engine does not have). So 613.7m is asked when two or
more of one player's objects entering at once generate continuous effects
from static abilities, or are Roles on one object, and the audit compares
every object's characteristics after the entry. An ability an object gains
after it enters takes the granting effect's later timestamp (613.7a,
"whichever is later"), so it never reads the order of entry. One step is
open: whether a static ability that a later copy effect gives an object takes
the copy effect's timestamp, as a grant's does. If it does, the list above is
complete. If not, an object that becomes a copy later reads its entry order,
and 613.7m falls back to asking whenever two of one player's entering objects
are not alike. That is the owner's question, below.

The property this replaces asked whenever two entering objects were not
alike. Since 613.7m covers objects entering any zone together, that would
have asked at every board wipe for the graveyard's timestamps.

**6b. `OrderCostReductions` under the same rule.** Under B, one rule for
every order the CR hands a player means it is asked only where the order
changes the total. For reductions the argument already exists, as a proof:
`cost-architecture.md#34-in-any-order-they-choose--the-prompt-the-theorem-and-the-middleware`'s
theorem shows every order gives one total, and names its two expiry
conditions (a reduction whose amount is a hybrid symbol, CR 118.7e, and a
floored "not below" reduction). So the order is asked when two reductions
apply and one of them meets a condition, which no registered card does: the
prompt drops out of every game that meets two reductions, a fixture migration
(9 lines in 8 test files name it) and a moved random stream, in the first PR,
beside its format change. Kept as it is, the engine has two policies for one
concept.

**Decided in review: B, and 6b folded into it**, the trigger order's precedent
(asked only where an order can change the game,
`triggers-architecture.md#52-apnap-over-the-seat-list-two-tiers-the-ordering-prompt-and-its-elision`):
it asks every question the CR's choice can turn on and none that gives one
game, and states that once for the three orders.

### Decision 7 — a value chosen as an effect applies (CR 608.2d)

**The table** is the census's value rows: a color 26, a creature type 54, a
card type 7, a basic land type 15, a card name 46, a number 12 (one in
`tdc`), a player not targeted 38 (one in `tdc`, chosen at random), a word 17,
a pile 21, a vote 33 (one in `tdc`), and a type, keyword or color out of the
few the card prints 9; 202 more choosing a value as a permanent enters, for a
linked ability (step 6); 3 as an additional cost to cast the spell (Caller of
the Hunt's creature type, Liquid Fire's number); and 2 while casting, because
the targets read the value (Aphetto Dredging's "target creature cards of the
creature type of your choice", 601.2c). The kinds carry no timing: the same
`ChooseCreatureType` is asked at 601.2b, as a permanent enters, or as an
effect applies. A card that prints its few options (Winding Way's "creature
or land", Roots of Life's "Island or Swamp") asks the same kind over those
options only. Two values are no one's here: a color chosen in a mana ability
(12, "{T}: Choose a color. Add …", CR 605.3b) is the mana model's
(`chosen-color-plus-could-produce`), and 14 are chosen before the game,
while drafting, as a permanent attaches or transforms, or as a replacement
applies, each its own mechanic's. Two remember what they chose
before (Garth One-Eye's card name, The Toymaker's Trap's number), through
decision 2's record of what an ability has chosen. Today `ChoiceKind::ChooseOpponent`
is the one such kind (Akroan Horse's opponent), and `ChoiceOption` has `Color`,
`Player`, `Number` and `CounterType`.

**The problem.** Each value is one rule's question, CR 608.2d's, with a
different set of answers; a client, the AI harness and the log each need to
know what is being chosen and among what.

- **A. A kind per value**: ten new kinds, `ChooseColor`,
  `ChooseCreatureType`, `ChooseCardType`, `ChooseBasicLandType`,
  `ChooseCardName { source, filter }`, `ChooseNumber`, `ChooseWord`,
  `ChooseKeyword` (Linvala, Shield of Sea Gate's "hexproof or
  indestructible", Blitzwing's at random; found in review), `SeparateIntoPiles`
  and `Vote`, and `ChooseOpponent` widened to
  `ChoosePlayer { source }` for "choose a player" (38 cards, Territorial
  Hellkite's at random in `tdc`; its options say whether only opponents are
  offered, and it is 700.2e's "which opponent chooses the mode" too). Each has
  its `source` as subject and its options a typed arm:
  `ChoiceOption::CreatureType(CreatureType)`, `LandType(LandType)`,
  `CardType(CardType)` (the three enums `types/card_types.rs` already has),
  `Keyword(KeywordFlag)` (`types/keywords.rs`), `CardName`, `Word`.
  *Upkeep*: `ChoiceKind`'s exhaustive matches gain ten arms and rename one; a
  client words each.
- **B. One kind, `ChooseValue { source, value: ValueKind }`**, with the same
  option arms. *Upkeep*: one arm in each exhaustive match, and a heuristic
  that keys on the kind matches the inner enum too.

**What both settle.**
- **A card name's options are the card registry's names that pass the
  card's filter**, never the names in this game: the game's list would show a
  player which cards sit in hidden zones (`information-model-design`'s
  concern), and every card a game can hold is registered. 24 of the cards
  narrow the name by what the card is ("a nonland card name", 17; "a creature
  card name", "an artifact card name"): CR 201.4a reads that against the
  named card's Oracle characteristics, so the filter is over each registered
  definition as printed (`// AS PRINTED:`, the invariant's tag for exactly
  this read). CR 201.4's "not a token's name unless also a card's" holds by
  construction. An Alchemy spellbook's draft is the same pick over the list
  the card names.
- **A vote** (701.38) is asked of each player in turn order from the player
  the text names, which `backlog.md`'s "2.4 Voting, and the `DecisionProvider`
  choice shapes" found the per-player walk lacks: the walk gains its starting
  seat. A player with several votes casts them at once (701.38d): one pick of
  that many, repeats allowed.
- **A pile** (700.3) is a pick of the objects for the first pile; the rest are
  the second.
- **"Secretly"** (a number chosen by each player, then revealed) is asked in
  APNAP order like any 608.2e choice; that a later chooser does not see an
  earlier answer is the information model's, filed there.
- **At random** (a player, a mode) draws from `GameState.rng` and asks
  nothing.

**Decided in review: A.** It is `ChooseOpponent`'s precedent, `ChoiceKind`'s
doc's contract ("adding a new variant here is the only change"), and each
kind's `subject` and wording stay one match arm. B saves arms in matches that
are exhaustive on purpose.

### Decision 8 — a "choose" that does not target is asked as the effect applies (item 28, `backlog.md`'s "2.33 Choice versus target"): settled in review, a fix

**What is wrong today.** A "choose a creature" in an effect is not a
target: it is made as the effect applies (CR 608.2d), hexproof and shroud do
not stop it (they stop targeting, 702.11b, 702.18a), and a spell with no
creature to choose can still be cast. The engine has the three backwards.
`EffectRecipient::Choose` is collected with the instances of "target"
(`Effect::instances`), announced at cast in `Target`'s arm
(`engine/put_on_stack.rs:336`), and must have a legal choice before the
spell is offered (`oracle/mana_helpers.rs:443`). No registered card
constructs one, so no game plays wrong today; the first card that writes one
would. Item 28 adds that a rider cannot name "each of them" and that
`resolve_primitive`'s object-reaching arms read only `ctx.targets`.

**The fix**, item 28's: `Choose` leaves the instance list and is asked at
resolution. One `recipients_for(ctx, recipient)` resolves `Choose`,
`FilteredPermanents` (already, `affected_permanents`) and a set the event
names, and the object-reaching arms read through it. The instance walk, the
castability check and CR 608.2b's re-check lose their `Choose` legs. *Code*:
~150–250 lines, item 28's sizing. The Black Gate ("choose a player … target
creature") is a `Choose` at resolution beside a target at cast, as written.

**Why there is no second option.** The note offered a timing on `Choose`, for
a selection announced at cast. The CR announces nothing at cast that is not a
target except 601.2b's list (modes, costs, X), and the census found one more
family: an additional cost that chooses (Close Encounter's and Monstrous
Emergence's "choose a creature you control", Caller of the Hunt's creature
type). That choice is part of a cost, paid at 601.2h and owned by the cost
model (`remaining-costs`), never an effect's `Choose`.

## The build, sized

Four PRs, each with its consumer, in this order. Code and tests apart,
estimates from the item sizings and from #306's for decision 4.

| PR | What it builds | Consumer | Code | Tests |
|---|---|---|---:|---:|
| **`options-named-plus-division`** | #306's decision 4 (the trigger and replacement arms, the typed matcher with its cost parser, and in its first commit the 33 order-dependent `picking(n)` uses of the 176); decision 5's division at cast, activation and placement; decision 6's two orders and 6b's cost reductions under the same rule; `ScryOrder` widened | the log's replay of Bridge from Below's two triggers, both orders; Arc Lightning (or Forked Bolt pooled); Omnath beside Momentous Fall as a fixture board | ~680–1,050 | ~610–1,060 |
| **`modal-spells-and-abilities`** | decisions 2 and 3: CR 700.2 whole, 603.3c's removal, the node's count, chooser and rules, the per-mode instance lists, both halves of item 154, a mode's own cost (item 81), `ModeIndex`, the record of what an ability chose before, castability through the enumerator | Abrade, Prismari Command, Casualties of War, a Will of the [clan], Shadrix Silverquill, Arc Trail, Demonic Pact; fixtures for repeats past the number of modes (Moment of Reckoning's shape), pawprints, 700.2e, and a mode with its own cost (700.2h) | ~760–1,150 | ~600–920 |
| **`costs-and-choices-at-resolution`** | decision 4 (`Pay`, the window at resolution, X as it is paid, "any player may", ward), decision 8's `Choose` at resolution, the untargeted division | Mana Leak, a ward creature, Leyline Tyrant's payment, Contamination's upkeep | ~600–950 | ~550–850 |
| **`values-chosen-at-resolution`** | decision 7's kinds and option arms with their log words and matcher arms, `ChoosePlayer`, the vote's starting seat, piles | Expel the Interlopers, Selvala's Stampede, fixtures for each value no `tdc` card asks | ~560–870 | ~450–700 |

The first three read ~1,150–2,110 lines each: inside the band (1,500–2,500,
`engineering-practices.md#sizing-a-phase`) at their upper estimates and under
it at their lower. The fourth reads ~1,010–1,570, at the band's floor at best;
it could fold into the third if the third lands low, which is the build's
call at its sizing, written in the doc.

**Fuzz A/B, predicted per PR.** The first changes no option's count or order
for decision 4, but it moves the stream twice on purpose: 6b drops a prompt
from every game that meets two cost reductions, and the division's card is
registered. So it runs three arms (`main`; the branch with the division's
card unregistered, which isolates 6b and should differ from `main` only in
games that met two reductions; the branch), through `fuzz_ab --require`,
which runs every arm on the new cards. Decision 6's new questions should be
asked by no registered card, which the build checks first by running the
test over every registered cost and every registered entry batch. Each later
PR registers cards, and its stream moves with them, measured the same way.

## Found while designing

1. **The brief cited CR 500.4 for a pool emptying at a step's end**; it is
   500.5 ("then any unspent mana left in a player's mana pool empties", with
   106.4). The setup-verb commit cites 106.4 and 500.5.
2. **Costs to attack are not this phase's** though they read "unless … pays":
   "can't attack you unless their controller pays" is CR 508.1h's cost to
   attack (40 cards, Ghostly Prison in `tdc`), which `roadmap-v2.md` step 11
   puts with `combat-requirements`.
3. **Three choices are the information model's, not this phase's**: a
   "secretly" chosen value, a vote's earlier answers being known (Selvala's
   ruling says they are), and a card name's list (decision 7). Filed for
   `information-model-design` at the build.
4. **The roadmap's 915 modal cards and 16 in `tdc`** are the Forge census's;
   by clause it is 738 and 15 (the census notes above).
5. **The setup verb leaves mana a hand-written board does not have today.**
   Under #306's decision 1, `planeswalker.scenario`'s tapped Plains becomes
   `then: player 0 taps Plains for {W}`, so player 0 has {W} in the pool at
   the declare attackers step's first priority, until that step ends (CR
   500.5). Today's board has the Plains tapped and nothing in the pool. For
   the owner: whether that board should keep the floating {W}, or say what
   it means some other way.
6. **A copied static ability's timestamp** (decision 6, raised in review).
   CR 613.7a gives a continuous effect from a static ability "the same
   timestamp as the object the static ability is on, or the timestamp of the
   effect that created the ability, whichever is later". For an object that
   becomes a copy after it entered (Cytoshape on a creature already on the
   battlefield), is the copy effect "the effect that created" the abilities
   it copies? If so, 613.7m's property is complete; if not, it asks whenever
   two of one player's entering objects are not alike. For the owner, and
   read against `copy-effects-architecture.md` at the build. The case: an
   opponent's Kudo, King Among Bears ("Other creatures have base power and
   toughness 2/2") is out; your Living Death returns Godhead of Awe ("Other
   creatures have base power and toughness 1/1") and Grizzly Bears together.
   Only Godhead generates an effect, so the property does not ask, and
   Godhead's later timestamp beats Kudo: your Savannah Lions is 1/1. Later,
   Polymorphous Rush makes the Bears a copy of Kudo. If the copied ability
   takes Rush's timestamp, it is the latest and the Lions is 2/2 whatever
   Living Death's order was. If it keeps the Bears' own, the Lions is 2/2
   only if the Bears were put after Godhead, and 1/1 if before: a choice the
   engine never asked decides it.
