# Handoff — `modes-plus-choice-kinds`, the design note

**Opened 2026-11-01**, on `choices/modes-plus-choice-kinds` off `d00a2abd`
(#306's merge), as `roadmap-v2.md` step 5 of "From triggers to v0.5". This
file is the design note, reviewed before any code; the design moves into the
doc decision 1 picks once the owner has decided, and the landing deletes this
file. Every count below was read against `d00a2abd`; the printed counts are
`plans/references/choice-kinds-census.py`'s, run 2026-11-01 over its cached
corpus (every Commander-legal paper card, fetched 2026-10-05, 32,115 cards),
with a second column for Tarkir: Dragonstorm Commander's 373 cards (`tdc`,
Scryfall `e:tdc -t:basic`), the five decks v0.5 plays.

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
| 154 | "another target" cannot exclude a player | **taken**, decision 3: the exclusion moves to the clause, which "each mode must target a different player" needs |
| 158 | "different from that one" has three axes | its modal axis is item 154's, taken; its copy axis stays `spell-copies`' |
| 28 | `EffectRecipient::Choose` has no resolution path, and a rider cannot name "each of them" | **taken**, decision 8 |
| 79 | CR 601.2h's payment order is the player's, and the engine takes it | **taken**, decision 6 |
| "Before card breadth" 4 | CR 613.7m's order of simultaneous timestamps is the active player's | **taken**, decision 6 |
| 240 | three type-changers' facilities; their upkeep halves wait on "sacrifice it unless you …" | the upkeep half is decision 4's (Contamination's "unless you sacrifice a creature"); the rest stays slotted where it is |
| 336, 361 | #306's decision 4: options named by what they are, and the 33 tests that answer by place | **taken**, the first PR, as #306 designed them |
| 92 | a CR 615.7 count spanning subjects with two choosers is refused | **closes as a record** (decision 5): every printed divided prevention gives each target its own share, so no count spans two choosers |

Re-counted against the tree: item 361's `picking(` uses are 176 in 36 files
(175 when measured); `EffectRecipient::` occurs 272 times in `mtgsim/src`
(item 154's sizing said 146 construction sites in September), which is why
decision 3 moves item 154's exclusion to the clause without touching the
enum's arms.

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
| **Modal**, any | 738 | 15 | Abrade |
| …a spell's modes | 474 | 9 | Prismari Command |
| …a triggered ability's modes (603.3c) | 209 | 6 | Junji, the Midnight Sky |
| …an activated ability's modes | 29 | 0 | Bow of Nylea |
| "Choose one" | 568 | 11 | Atsushi, the Blazing Sky |
| "Choose two", "three" | 51 | 2 | Prismari Command |
| "Choose one or both" | 57 | 0 | Artful Takedown |
| "Choose one or more" | 21 | 2 | Casualties of War, Sublime Epiphany |
| "Choose any number" (zero included, Rankle's ruling) | 2 | 0 | Rankle, Master of Pranks |
| "Choose up to N" | 12 | 0 | Hullbreaker Horror |
| "Choose X" | 1 | 0 | Doomsday Confluence |
| a count raised by a condition ("if you control a commander as you cast this spell, you may choose both instead") | 19 | **5** | the Will of the [clan] cycle |
| zero or exactly N ("you may choose two") | 1 | **1** | Shadrix Silverquill (ruling: "exactly zero modes or two modes") |
| the same mode more than once (700.2d) | 22 | 0 | Season of the Burrow |
| pawprints (700.2i) | 5 | 0 | Season of the Burrow |
| a mode chosen at random | 5 | 0 | Summon: Magus Sisters |
| another player chooses the mode (700.2e) | 3 | 0 | Fatal Lore |
| a mode with its own target (700.2c, f) | 570 | 12 | Abrade |
| "each mode must target a different player" (item 154) | 7 | **1** | Shadrix Silverquill |
| "choose one that hasn't been chosen this turn" | 26 | **2** | Parapet Thrasher, Teval's Judgment |
| a cost per mode: spree (700.2h), tiered | 21, 7 | 0 | — |
| entwine, escalate | 32, 9 | 0 | — |
| **"Unless [a player] [acts]"** (118.12a) | 428 | 1 | Compulsive Research |
| …pays | 292 | 0 | Mana Leak |
| …another action (sacrifices, discards, returns, …) | 138 | 1 | Contamination, Compulsive Research |
| …the payer is not the ability's controller | 232 | 1 | Mana Leak |
| …"unless any player pays", "any player may" (asked of each, 101.4) | 19 | 0 | Rhystic Study |
| ward, printed; its cost not mana; granted | 184; 52; 42 | 1; 0; 0 | Canopy Gargantuan |
| echo, cumulative upkeep (the same payment as keywords) | 130 | 0 | — |
| **a cost paid at resolution**, "may pay …. If/When you do" | 510 | 3 | Gix, Yawgmoth Praetor; Spit Flame |
| …its amount chosen as it is paid ({X}, X life, any amount; 107.3f) | 81 | 1 | Leyline Tyrant |
| *not this phase*: a "can't" with a price ("can't attack you unless their controller pays") | 40 | 1 | Ghostly Prison: a cost to attack, CR 508.1h, `combat-requirements`' |
| **Divided among targets** (601.2d) | 114 | **3** | Dragonlord Atarka (a trigger), Magma Opus, Skarrgan Hellkite (an activation) |
| …among objects not targeted (608.2d) | 10 | 0 | Feast of the Victorious Dead |
| …prevention divided as you choose (item 92) | 5 | 0 | — |
| **An order a card hands a player**, a library's "in any order" | 171 | 1 | Ponder |
| objects entering at once under one player (613.7m) | 66 | 1 | Living Death |
| CR 601.2h's payment order: no phrase; a shape read off the cost (decision 6) | — | — | Omnath, Locus of Mana with Momentous Fall (ATOM-601.2h-003) |
| **A value chosen as the effect applies** (608.2d): a color | 43 | 0 | — |
| …a creature type; a card type; a basic land type | 58; 7; 15 | 0 | — |
| …a card name (201.4) | 46 | 0 | Cabal Therapy |
| …a number (including "secretly") | 19 | **1** | Expel the Interlopers |
| …a player, not targeted (including "at random") | 47 | **1** | Territorial Hellkite |
| …a word with no rules meaning (607.2f) | 23 | 0 | Extinction Event's "odd or even" |
| …a pile (700.3) | 21 | 0 | Fact or Fiction |
| …a vote (701.38) | 35 | **1** | Selvala's Stampede |
| the same values chosen **as the permanent enters** (stored for a linked ability, CR 607.2d) | 194 | 3 | Temple of the Dragon Queen, Reflections of Littjara, Frontier Siege |

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
- Ponder's "put them back in any order" is the row's one `tdc` card; 135 of
  the 171 are "on the bottom of your library in any order".

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
- **B. `setup-architecture.md` gains a §7f.** The boundary's doc already owns
  the log, the matcher and decision 4. *Upkeep*: setup's doc becomes the
  rules home for modes and costs, which it is not otherwise: its subject is
  the scenario, the save and the boundary, and a reader of CR 700.2 would not
  look there.
- **C. Split by subject**: modes and the division in `cost-architecture.md`
  (beside 601.2f–h), the resolution's choices in `triggers-architecture.md`'s
  §6 (beside "may"), the orders where each performer is. *Upkeep*: four docs
  own one boundary's kinds, and a kind's shape is in one doc while its rule
  is in another.

**Recommendation: A.** The phase is a subsystem by the row's own test: it
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
| a fixed number of distinct modes | 619 | 13 | `ModalCount::Exactly` |
| a range ("one or both", "one or more", "up to N", "any number", zero included) | 92 | 2 | `UpTo`, `Any` (no lower bound) |
| zero or exactly N, never between | 1 | 1 | nothing |
| a count raised by a condition read as the modes are chosen | 19 | 5 | nothing |
| a count that is X | 1 | 0 | nothing |
| a mode more than once, its repeats ordered by their targets (Season's ruling) | 22 | 0 | nothing |
| a budget the modes spend (pawprints) | 5 | 0 | nothing |
| a mode's own cost (spree, tiered) and one cost per extra mode (escalate), all modes for a cost (entwine) | 69 | 0 | nothing (item 81) |
| a mode not chosen for this object's ability earlier this turn | 26 | 2 | nothing |
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
  mode chosen this turn is not listed. *Size of a list*: "choose one of three"
  is three options; the largest printed list is Casualties of War's and
  Sublime Epiphany's 31 (one or more of five), then Doomsday Confluence's
  multisets of X among three (21 at X = 5); a Season's 15. *Code*: one
  enumerator over the count rule, ~120–180 lines, one prompt shape. *Upkeep*:
  a new count rule is a new arm of the enumerator, never a new prompt.
- **C. One pick per mode, in a loop**: "add a mode or stop", re-offering what
  still fits, the way the mana window asks. Covers every row with `pick_n`
  of zero or one. *Upkeep*: a modal cast asks as many questions as it chose
  modes plus one, and the order of the picks, which the rules do not have, is
  in the log.

**Performance.** A and B ask one question for every printed "choose one",
which is 77% of modal cards; C asks two. B allocates its options per modal
announcement (a few small `Vec`s, ~30 at the worst printed card); nothing is
on `GameState`.

**What the identity is.** A mode's printed place is its identity, not a
position in the offered list: the card fixes it, and an illegal mode dropped
from the list does not renumber the others. The log writes `modes 1, 3`; an
answer naming `modes 1, 3` finds that set wherever a build lists it, by
derived equality (`setup-architecture.md#7e-pausing-at-a-decision`, decision
4b's typed matcher). A client draws each option as its modes' text, read off
the ability as the layers give it.

**The count's shape**, which B's enumerator reads, from the table and not
asked: `ModalCount` becomes the fewest and the most (the most a number, X or
"all"), whether the empty set is the only short choice (Shadrix), a
condition raising the most (the Will cycle), whether a mode may repeat, a
pawprint weight per mode, and who chooses (the controller, another player,
700.2e, or random). A mode's own cost (700.2h) is a `Mandatory` additional
cost assembled from the chosen modes (item 81's resolution), in the total at
601.2f. "Not chosen this turn" reads a history row naming the modes chosen
for that object's ability, keyed on the object's existence (Parapet
Thrasher's ruling: a new object chooses afresh), written as the modes are
announced. The keywords built on the count, spree, escalate, entwine and
tiered, are sibling keywords and wait for their own rows
(`engineering-practices.md#gaps-a-phase-finds`); none is in `tdc`. Random
modes draw from `GameState.rng` and ask nothing.

**Recommendation: B.** It is the only option where every printed count rule
is one question of one shape, and it makes 700.2a's "can't be chosen" a
property of the list rather than a rejection. Its lists stay under ~30 for
every printed card, and an AI harness reads a flat list of whole choices
directly.

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
  others. The build stores one instance list per mode beside the card's own
  (`instances` for what precedes and follows the modal node, and each mode's
  own), so the announcement concatenates the chosen modes' lists and still
  never walks the tree. **A mode is its own instance scope**: a back-reference
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

**Item 154 with it.** "Each mode must target a different player" is a
constraint across modes' instances, so the exclusion moves from the
`ObjectFilter` leaf (`OtherThanInstance`, asked only of a permanent filter) to
the clause: an instance carries "not what instance k took", which a player
or an "any target" filter can carry, and the two enumeration arms that skip
`validate_selection` apply it. ~60 lines, as item 154 sized it.

**Recommendation: A.** It keeps the effect on the stack the card's text,
which the log, a client and a copy all read, and it costs nothing a clone
carries. B buys a simpler resolution arm for a deep copy at every modal
cast and a second source of truth.

### Decision 4 — a cost paid as an effect resolves: "unless", "you may pay", ward

**The table**: the census's "unless" and payment rows. Grouped by what the
shape must do:

| The shape must express | Cards | `tdc` | Held today |
|---|---:|---:|---|
| a payment by the ability's controller, optional, then a branch on it ("you may pay {1}. If you do") | 510 | 3 | `Effect::Optional`, but no payment: an atom acts, it does not pay |
| …by another player ("unless its controller pays") | 232 | 1 | `Optional`'s chooser names another player; no payment |
| …asked of every player in turn until one pays ("unless any player pays") | 19 | 0 | nothing; triggers-architecture §6.2 files it "with its first card" |
| a payment that is not mana (sacrifice, discard, return a land, life) | 138 | 1 | `Cost`'s arms exist for casting; `plan_payment` asks the choices |
| an amount chosen as it is paid ({X}, X life, any amount) | 81 | 1 | MA-2's legal values of X, at cast only |
| ward: a trigger on becoming a target whose effect is the payment, its X read at resolution | 184 + 42 granted | 1 | TR-5c's `BecomesTarget` event; no payment |

**The problem.** CR 118.12a says "[do A] unless [P does B]" means "[P] may
[do B]. If [P doesn't], [do A]", and B is a cost paid as the effect resolves.
The resolution has `Effect::Optional { chooser, effect }` and its answer
(`CostAnswer`: `Does`, `Doesnt`, `Cant`, triggers-architecture §6.2), and
casting has the payment (`plan_payment`, the 601.2g window, `pay_costs`). What
is missing is a payment inside a resolution, and where "unless" is spelled.

- **A. CR 118.12a's own reading: a `Pay` instruction inside `Optional`.** A
  new `Primitive::Pay { costs }` pays as the resolving instruction's player,
  through the casting path's `plan_payment` and `pay_costs`, with CR 605.3a's
  mana window opened first (ATOM-605.3a-002). It answers `Does` when paid and
  `Cant` when it cannot be paid. "Unless P pays C, do E" is written
  `Sequence[Optional { chooser: P, Pay(C) }, Conditional(Doesnt, E)]`, which
  is the CR's sentence. "Any player may" is `Optional`'s chooser reaching
  each player in APNAP order (CR 101.4) until one pays. An X is chosen as it
  is paid (107.3f) from MA-2's legal values against what the payer can
  produce. *Code*: the primitive, the window at resolution, the chooser walk
  and X, ~250–400 lines; a builder (`Effect::unless(payer, costs, effect)`)
  so a card reads as its text.
- **B. A combinator, `Effect::Unless { payer, costs, effect }`.** The
  resolver asks and pays in one arm. *Upkeep*: a second spelling of what
  `Optional` and `Conditional` already say, and "you may pay {1}. If you do,
  …" (510 cards) is still A's shape, so both exist.

**What both settle.** Offered only when payable (CR 118.3), as a shockland's
payment is: a payer who cannot pay is not asked and answers `Cant`. A window
that ends with the cost unpaid is the payer declining: the mana stays in the
pool until the step ends (CR 106.4, 500.5), as on paper, and nothing is
reversed, since CR 732.1's reversal is a cast's or an activation's. The payment
is CR 608.2c's, "in the order written", so it is not routed through
`payment_order_rank`, whose doc already says why. Ward is a keyword on this
base and builds whole with it: its trigger is TR-5c's, its effect "counter
that spell or ability unless that player pays [cost]" (702.21a), its X read
as it resolves (702.21b). Echo and cumulative upkeep are sibling keywords on
the same payment and wait for their own rows.

**Recommendation: A.** It is the CR's own sentence, it adds one primitive
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

- **A. On the instance**: `TargetInstance` gains `shares: Option<Box<[u64]>>`,
  one per chosen target, asked by `allocate` over the instance's targets right
  after it is announced (minimum 1 each, the total the clause's amount), and
  the atom's amount reads its target's share. *Clone cost*: 16 bytes an
  instance, no heap for an undivided one.
- **B. On the entry**, a `divisions` list keyed by instance index. *Upkeep*:
  a second list kept in step with `chosen_targets` through every rewrite
  (a copy's new targets, 115.7f), where A's travels with its targets.
- **C. In the amount**, an `AmountExpr` that names a share by target. *Upkeep*:
  the amount reads an announcement it does not own, and an untargeted division
  has no target to key it on.

**The prompt.** `ChoiceKind::DivideAmongTargets { source }` over `allocate`,
its buckets the chosen targets (by what each is, #306's decision 4), each
bucket's least 1. An untargeted division asks the same kind as the effect
applies, once "any number of creatures" has chosen them, with the same rule.

**Item 92 closes.** Each of the 5 printed divided preventions announces each
target's share, so each target gets a shield of its own share and no count
spans two choosers; the refusal stays for a card no one has printed.

**Recommendation: A**, the shape `backlog.md` §2.20 sized.

### Decision 6 — when the engine asks an order the CR hands a player

**The problem.** CR 601.2h ("in any order") and 613.7m ("in the order of that
player's choice") hand a player an order the engine fixes today:
`payment_order_rank` pays mana, then what mutates, then what moves an object;
an entry batch timestamps in allocation order. Item 79 and "Before card
breadth" item 4 asked for one rule: ask only where the orders' outcomes
differ, and here only among orders that complete.

**The table.** No printed card names 601.2h's order; it is the rule's, on
every cost of two or more parts. Where the order can change the game:
- a part moves an object whose last-known information reads what another part
  changes: Omnath's power reads the pool, so sacrificing it before paying the
  mana draws more off Momentous Fall (ATOM-601.2h-003). Five printed
  creatures have a power or toughness that reads the pool or the life total
  (Omnath, Serra Avatar, Soul of Eternity, Roiling Horror, Enduring Angel), and
  a counter a part removes is read by any creature whose power counts it;
- a part moves an object into a zone another part takes from: a discard
  beside an exile from the graveyard.
613.7m's family is 66 cards that put several objects onto the battlefield at
once (Living Death in `tdc`), and every creation of tokens that are not alike.

- **A. Ask whenever there are two or more parts or objects.** The CR's literal
  reading, `OrderCostReductions`' precedent. *Cost*: every "{1}, {T}:" and
  "{B}, Sacrifice a creature:" activation and every kicked or sacrificing cast
  gains a question whose answers give one game; a person is asked "tap first
  or pay first" at every such activation.
- **B. Ask only where an order can change the game, by a structural test.**
  601.2h: when the payment holds a part that moves an object and another part
  that changes the pool, the payer's life or the moved object's counters, or
  two parts where one's destination is the other's source; the options are
  the orders that complete. 613.7m: when two or more of one player's entering
  objects are not alike (another card, another token def, another status),
  each player ordering their own in APNAP order. *Cost*: a structural check
  per payment and per entry batch; a question only on the boards the table
  names.
- **C. As today.** *Cost*: a choice the CR gives a player is the engine's,
  which the CR-first rule does not allow once a printed board reaches it, and
  Omnath's does.

A card's own "in any order" (Ponder, 171 cards) is a library order and already
has a kind, `ScryOrder`, built for CR 701.22a's; it is widened to
`OrderLibraryCards { source, bottom }`, a log word the first PR's format change
carries.

**Recommendation: B**, the trigger order's precedent (asked only where an
order can change the game, `triggers-architecture.md`'s §5.2 elision): it
asks every question the CR's choice can turn on and none that gives one game.

### Decision 7 — a value chosen as an effect applies (CR 608.2d)

**The table** is the census's value rows: a color 43, a creature type 58, a
card type 7, a basic land type 15, a card name 46, a number 19 (one in
`tdc`), a player not targeted 47 (one in `tdc`, chosen at random), a word 23,
a pile 21, a vote 35 (one in `tdc`); and 194 more choosing a value as a
permanent enters, for a linked ability (step 6). Today `ChoiceKind::ChooseOpponent`
is the one such kind (Akroan Horse's opponent), and `ChoiceOption` has `Color`,
`Player`, `Number` and `CounterType`.

**The problem.** Each value is one rule's question, CR 608.2d's, with a
different set of answers; a client, the AI harness and the log each need to
know what is being chosen and among what.

- **A. A kind per value**, `ChooseColor`, `ChooseCreatureType`,
  `ChooseCardType`, `ChooseBasicLandType`, `ChooseCardName`, `ChooseNumber`,
  `ChooseWord`, `SeparateIntoPiles`, `Vote`, beside `ChooseOpponent`, each
  with its `source` as subject and its options a typed arm
  (`ChoiceOption::CreatureType(Subtype)`, `CardType`, `CardName`, `Word`). *Upkeep*:
  `ChoiceKind`'s exhaustive matches gain ten arms; a client words each.
- **B. One kind, `ChooseValue { source, value: ValueKind }`**, with the same
  option arms. *Upkeep*: one arm in each exhaustive match, and a heuristic
  that keys on the kind matches the inner enum too.

**What both settle.**
- **A card name's options are the card registry's names**, never the names in
  this game: the game's list would show a player which cards sit in hidden
  zones (`information-model-design`'s concern), and every card a game can hold
  is registered. CR 201.4's "not a token's name unless also a card's" holds
  by construction.
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

**Recommendation: A.** It is `ChooseOpponent`'s precedent, `ChoiceKind`'s
doc's contract ("adding a new variant here is the only change"), and each
kind's `subject` and wording stay one match arm. B saves arms in matches that
are exhaustive on purpose.

### Decision 8 — when a selection that does not target is made (item 28, `backlog.md` §2.33)

**The problem.** `EffectRecipient::Choose` is collected as an instance of
"target" and announced at cast in `Target`'s arm (`engine/put_on_stack.rs:337`),
though CR 608.2d makes a plain "choose" at resolution and only CR 601.2b's
list is announced early. No registered card constructs a `Choose`. Item 28
adds that a rider cannot name "each of them" and that `resolve_primitive`'s
object-reaching arms read only `ctx.targets`. The census: of 36 cards saying
"as you cast this spell", 33 are a mode count raised by a condition (decision
2) and none chooses an object; so no printed card announces a non-targeting
selection early.

- **A. `Choose` leaves the instance list and is asked at resolution**: one
  `recipients_for(ctx, recipient)` resolves `Choose`, `FilteredPermanents`
  (already, `affected_permanents`) and a set the event names, and the
  object-reaching arms read through it. *Code*: ~150–250 lines, item 28's
  sizing; the instance walk and CR 608.2b's re-check lose their `Choose` legs.
- **B. `Choose` gains a timing**, `Choose { at: Announcement | Resolution }`,
  and stays in the instance list for the first. *Upkeep*: a timing no printed
  card asks, on 272 `EffectRecipient` uses.

**Recommendation: A.** The census finds no early non-targeting selection, and
CR 601.2b's list, the only early one, is modes, costs and X, each of which has
its own shape. The Black Gate ("choose a player … target creature") is a
`Choose` at resolution beside a target at cast, which A expresses as written.

## The build, sized

Four PRs, each with its consumer, in this order. Code and tests apart,
estimates from the item sizings and from #306's for decision 4.

| PR | What it builds | Consumer | Code | Tests |
|---|---|---|---:|---:|
| **`options-named-plus-division`** | #306's decision 4 (the trigger and replacement arms, the typed matcher with its cost parser, the 33 `picking(n)` uses in its first commit); decision 5's division at cast, activation and placement; decision 6's two orders; `ScryOrder` widened | the log's replay of Bridge from Below's two triggers, both orders; Arc Lightning (or Forked Bolt pooled); Omnath beside Momentous Fall as a fixture board | ~670–1,040 | ~580–1,000 |
| **`modal-spells-and-abilities`** | decisions 2 and 3: CR 700.2 whole, 603.3c's removal, the count's shapes, the per-mode instance lists, item 154's clause exclusion, a mode's own cost (item 81), the "chosen this turn" row | Abrade, Prismari Command, Casualties of War, a Will of the [clan], Shadrix Silverquill; fixtures for repeats, pawprints, 700.2e | ~660–1,000 | ~500–800 |
| **`costs-and-choices-at-resolution`** | decision 4 (`Pay`, the window at resolution, X as it is paid, "any player may", ward), decision 8's `Choose` at resolution, the untargeted division | Mana Leak, a ward creature, Leyline Tyrant's payment, Contamination's upkeep | ~600–950 | ~550–850 |
| **`values-chosen-at-resolution`** | decision 7's kinds and option arms with their log words and matcher arms, the vote's starting seat, piles | Expel the Interlopers, Selvala's Stampede, fixtures for each value no `tdc` card asks | ~540–850 | ~450–700 |

Each reads ~1,000–2,050 lines: the first and third inside the band
(1,500–2,500, `engineering-practices.md#sizing-a-phase`) at their upper
estimates, the second and fourth near its floor. The fourth could fold into
the third if the third lands low; that is the build's call at its sizing, in
the doc.

**Fuzz A/B, predicted per PR.** The first changes no option's count or order,
so the random agent's picks and the gameplay rows are `IDENTICAL`; decision 6's
questions are asked only on boards no registered card reaches, which the build
checks by running the structural test over every registered cost first. Each
later PR registers cards, so its stream moves with them, measured as
`fuzz_ab --require` does (every arm runs the new cards).

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
