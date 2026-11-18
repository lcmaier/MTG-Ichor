# Handoff — `values-chosen-at-resolution`, the design note (first of three)

**Opened 2026-11-14** on `choices/value-chosen-as-an-effect-resolves` off
`7f0e97ef` (#328's merge), as `roadmap-v2.md` §3a step 5, row
`values-chosen-at-resolution`: the last design in `choices-architecture.md`.
This file is the design note, reviewed before the build; the row's last PR
deletes it. Every count and line below was read against `7f0e97ef`.

**Revised after the design stop's fresh read** (the `owner-review` agent,
2026-11-14), and **at #330's first review round (2026-11-15)**:

- **Decisions 4 and 5 are settled, both A** (the owner): item 292 comes
  forward, and the row is three PRs.
- **Decisions 1 to 3 are rewritten.** The first draft sorted its shapes by
  how a card's sentences are built: a choice "in a sentence of its own" was
  one type and a choice "inside the verb's clause" another. The review asked
  for what the game piece does instead. There is one shape now: a step
  chooses, the answer is kept, later steps use it.
- **The number comparison is one generic leaf**, where the first draft added
  `PowerAtLeast` alone.
- **At the second round (2026-11-15), decision 1 is settled: A**, a name the
  card gives.
- **At the third round (2026-11-15), decision 2 is settled: A**, every
  choice is a choose step. The owner asked why a name is text and not an
  enum, and decision 1 proposed an enum for the name's type.
- **At the fourth round (2026-11-15), the name is an enum** (the owner).
- **At the fifth round (2026-11-15), decision 3 is settled: A**, looked up
  where the step runs.
- **At the sixth round (2026-11-15), the number comparison is one filter
  with a labelled comparison.** Every decision is taken, and the build
  starts.

**The first row is built** (PR #330, 2026-11-15), as
`choices-architecture.md`'s stub records under
`value-chosen-as-an-effect-resolves`, with where it differs from this note.
What is left for this file's two other rows:

- **`several-choosers-at-one-step`** (item 429): **its design note is
  this file's last section**,
  [the second row](#the-second-row--several-choosers-at-one-step-the-design-note),
  opened 2026-11-17 and stopped for the owner's review.
- **`values-kept-past-the-resolution`** (item 428): start from decision 3's
  refusals and decision 5's third row. The registry's test
  (`every_chosen_name_read_is_kept_by_a_step_before_it`) names each place a
  kept name is refused because it outlasts the resolution; that list is the
  third row's intake.

The shapes below are the note's. As built, each choose word takes its
chooser first (`choose_number(you(), 0, 10)`), a kept choice is read with
`the_chosen_number()`, `color(the_chosen_color())`, `the_chosen_objects()`
and their siblings, and the enum the note calls `ChoiceName` is `ChoiceKind`
(the question types became `Decision*` in #331).

## The model: choose, keep, use

A resolving spell or ability is a short list of steps, run in the order
printed (CR 608.2c). This PR adds one kind of step and one small store.

- **Choose.** A step asks a player for something, or draws it at random,
  and does nothing else.
- **Keep.** The answer goes into the resolution's list of choices, under a
  name the card gives it. The list is empty when the spell or ability starts
  to resolve, and is thrown away when it finishes.
- **Use.** A later step reads a name wherever it would otherwise hold a
  plain value: what it acts on, a number, or a filter's color, type or set
  of objects.

| Card | Its steps |
|---|---|
| Expel the Interlopers | choose a number from 0 to 10, kept as `number` → destroy each creature with power at least `number` |
| Duneblast | choose up to one creature, kept as `spared` → destroy each creature not in `spared` |
| Feast of the Victorious Dead, its counters | choose any number of creatures you control and split the counters among them, kept as `fed` → put each one's share on it |
| Gluntch, the Bestower, its draw | choose a player, kept as `drawer` → `drawer` draws a card |

**What of a card's English still reaches the engine**, each because a rule
turns on it:

- the order of the steps (CR 608.2c);
- "target" against "choose": a target is announced as the spell is cast and
  checked again as it resolves (CR 115, 608.2b), a choice is made as its
  step runs (608.2d). That split is #326's;
- who chooses: you, another player the card names, or no one, at random.

Nothing else does. Whether the card says "it", "that creature", "the chosen
creature" or "the rest", and whether it chooses in a sentence of its own or
inside the verb's ("distribute … among"), the steps are the same.

## Where the work sits

All of it is inside one stack object's resolution, the RESOLVE step of
`engine-map.md#11-the-path-in-one-sketch`: `resolve_top_of_stack` →
`resolve_effect` → each step in turn. Today a step can choose objects as it
applies (decision 8, landed in #326: `EffectRecipient::Choose`, asked
through `SelectRecipients`), and nothing after it can name what it chose.

Off that step, the filter table (`engine/object_filter.rs`) gains the
generic number comparison and a plain "one of these objects" leaf, and
every client reads the new kinds: the dev GUI's words, the decision log,
setup lines, the why panel and the random agent.

## The intake

`python plans/check_state_of_play.py --slotted values-chosen-at-resolution`
lists two items and no backlog entries:

- **Item 425, "a player chosen as an effect applies is refused"**
  (`codebase-state.md`, Deferred Migrations). `chosen_by_the_controller`
  (`engine/resolve.rs:662`) refuses a `Choose` over players, and no choice
  reaches `resolve_player_for_self` (`engine/resolve.rs:3461`). Taken:
  decisions 1 and 4. Sized ~40–80; inside decision 4's 60–100.
- **Item 426, "'it' after a choice made as the effect applies has no
  word"**, which makes the registry refuse a chained change on a choice
  (`cards/authoring/instructions.rs:382`). Taken: decision 1. Sized ~80–150;
  re-counted at 70–120.

Two more come in:

- **Item 394, most of it.** Item 394 ("a filter compares only power, and
  only to a fixed number") is slotted to step 9's cheap gaps. Expel needs
  power compared to the chosen number. Decision 3 builds that as the one
  leaf every such comparison is, so power and toughness, each three ways and
  against any amount, land here; mana value follows when the type that says
  which number is read gains it (step 7), and "tapped" stays slotted.
- **Item 292, settled (decision 4).** "A one-player instruction whose
  player is gone acts on its controller instead", slotted to step 7's
  `resolution-recipients-and-amounts`, its second PR. Its verdict of
  2026-10-22 says unreachable because every registered card that targets a
  player has that one target. Vindictive Lich (#319, 2026-11-07) changed
  that.

## Counted

### The cards

Every number here is `plans/references/chosen-value-readers-census.py`
(committed with this note) over `cast-census.py`'s cache of the
Commander-legal paper corpus, fetched 2026-10-05, 32,115 cards; read
2026-11-14. Its population is `choice-kinds-census.py`'s "value at
resolution" rows with piles and votes left out: **223 cards**. Every row
under 40 was read by hand, and so were the 37 player cards.

| Kind | Cards | In `tdc` | Example |
|---|---:|---:|---|
| a creature type | 54 | 0 | Kindred Dominance |
| a card name | 46 | 0 | Cabal Therapy |
| a player, not targeted | 38 | 1 (at random) | Gluntch, the Bestower; Territorial Hellkite |
| a color | 26 | 0 | Wash Out |
| a word with no rules meaning | 17 | 0 | Extinction Event's "odd or even" |
| a basic land type | 15 | 0 | Terraformer |
| a number | 12 | 1 | Expel the Interlopers |
| a type, keyword or color out of a printed few | 9 | 0 | Linvala, Shield of Sea Gate's "hexproof or indestructible" |
| a card type | 7 | 0 | World Queller |

**Who chooses.** The controller on 188; another player the text names on 6
("target player chooses a card name", Petra Sphinx; "target opponent chooses
a number", Choice of Damnations); no one, at random, on 13 (12 players,
Blitzwing's keyword); each player or each opponent in turn on 16 (CR 608.2e:
Patriarch's Bidding, friend or foe, Seize the Spotlight). The last are
several choosers at one step, a vote's walk without its count.

**What reads the value afterwards.** A card can sit in several rows;
"lasting" is a duration or a later event in the same clause.

| Reader | Cards | Lasting | Example |
|---|---:|---:|---|
| a filter over cards in a zone (a hand, a library, a graveyard, what was revealed) | 62 | — | Cabal Therapy's "all cards with that name" |
| a filter over permanents or spells, read as the step applies | 67 | 39 | Expel's "power greater than or equal to the chosen number"; Kindred Dominance |
| …of those, a "can't", a requirement or a later trigger that reads the filter after the resolution (CR 611.2c's second sentence) | about 10, by hand | all | The Black Gate, Conjurer's Ban, Twins of Discord, Zombie Boa |
| the verb's player: "that player draws", "you and that player each" | 49 (37 of the 38 players) | 26 | Gluntch's "choose a second player to draw a card" |
| a condition: "if that card has the chosen name", "for each player who chose" | 27 | 10 | Predict |
| an amount: "that many", "that much" | 10 | 3 | By Invitation Only |
| a change's own parameter: "protection from the chosen color", "becomes that type", "gains that ability" | 47 | 44 | Akroma's Blessing; Terraformer; Linvala |
| inline: chosen inside the instruction that uses it | 34 | — | Mistform Dreamer's "becomes the creature type of your choice"; Wash Out's "permanents of the color of your choice" |

Three kinds have no reader this PR can build with a printed card:

- **A basic land type**: every one of the 15 is read by a change ("each land
  you control becomes that type", "landwalk of the chosen type"), and the
  six other printed "choose a basic land type" choose as a permanent enters
  (Scryfall, `o:"choose a basic land type" -o:"becomes"`, 6, 2026-11-14).
- **A keyword out of a printed few**: Linvala's and Blitzwing's "gains that
  ability", a change's parameter.
- **A card name**: 38 of the 46 read a hidden zone (`information-model`'s),
  and the other 8 are lasting ("spells with the chosen name can't be cast",
  Conjurer's Ban) or linked (step 6). None is writable with this PR's steps.

And the word: 12 of the 17 sort each player by a word (friend or foe) or ask
each opponent, which is the several-choosers walk; "odd or even" (5) reads
mana value or power by parity, a predicate of its own.

### The objects, by what the next step does with them

Of the 75 cards whose controller chooses a permanent as an effect applies
(#326's census), 68 name the chosen object again. Read by hand for what the
card then does with it (`choice-kinds-census.py --names` on the family,
2026-11-14):

| The chosen object, after the choice | About | Example |
|---|---:|---|
| is what the next step acts on: "it gains", "put … on it", "untap it" | 13 | Final Showdown, The Elderspell, The Nipton Lottery |
| is what the next step spares: "destroy the rest", "each creature not chosen" | 11 | Duneblast, Last One Standing, Celestial Judgment |
| is copied: "a copy of it", "becomes a copy of that creature" | 8 | Cytoshape, Season of Weaving, Polymorphous Rush |
| is named by a lasting effect or a later trigger: "is dealt to that creature instead" | 4 | Gideon's Sacrifice, Azra Oddsmaker |
| deals damage, or has its power read | 2 | Crush Underfoot, Benthic Anomaly |
| the rest: a card in a hand, each player choosing, a choice with a payment | ~30 | Despise, Tunnel of Love, Mudslide |

So most object choices are used for something other than "the next verb
acts on it". The brief's chain ("choose a creature you control; it gets
+2/+2 and gains trample") is printed nowhere in that phrasing (Scryfall,
`o:/choose (a|an|another) [a-z ]*creature[^.]*\. (it|that creature) (gets|gains)[^.]* and (gains|gets)/`,
0, 2026-11-14); The Nipton Lottery's run of steps on its chosen creature is
the printed form.

### A number read off an object, and compared

Scryfall, `game:paper -is:funny`, `total_cards` for each phrase, read
2026-11-15. By phrase, so a condition's "if its power is 4 or greater" is
counted beside a filter's; the grid's shape is the point, not a cell.

| | N or less | N or greater | exactly N | against an amount ("less than", "greater than", "equal to" something read) |
|---|---:|---:|---:|---:|
| power | 176 | 250 | — | 53 |
| toughness | 25 | 24 | — | 15 |
| mana value | 406 | 107 | 76 | 110 |

Queries: `o:/power [0-9x]+ or less/`, `o:/power [0-9x]+ or greater/`, the
same two for `toughness` and `mana value`, `o:/with mana value [0-9x]+[ ,.](?!or)/`,
`o:/power (less than|greater than)/`, `o:/toughness (less than|greater than)/`,
`o:/mana value (less than|greater than|equal to)/`. Beside them:
`o:/with (lesser|greater) (power|toughness|mana value)/`, 30 (another
object's number as the amount, Legion Warboss's mentor), and
`o:/(mana value|power) of the chosen quality/`, 7 (odd or even, a different
predicate).

### The rulings read

- Expel the Interlopers (2023-09-01): "The numbers you may choose include 0
  and 10."
- Territorial Hellkite (2017-08-25): "If a player can't be attacked, that
  player can still be chosen at random."
- The Black Gate (2023-06-16): the chosen player "needs only to have the
  most life … as The Black Gate's last ability resolves", and the creature
  "can't be blocked by any creature the chosen player controls, including
  creatures that weren't on the battlefield when The Black Gate's ability
  resolved". A value is fixed as it is chosen, and an effect that outlasts
  the resolution reads the value it was made with.
- Gluntch, the Bestower (2022-06-10): "All three players must be different
  players, although any one of them can be you." A choice excluding an
  earlier one's pick, as Fortunate Few's does.

### The rules

- **CR 608.2d**: choices not made while casting are announced "while
  applying the effect", and no illegal or impossible option may be chosen.
- **CR 608.2c**: instructions are followed in the order written.
- **CR 608.2h** and **CR 611.2c**: information is determined once, as the
  effect is applied; a continuous effect from a resolution that changes
  characteristics fixes its set then, and one that changes the rules keeps
  reading. 611.2c is the rule that *watches* this one: it is why a "can't"
  made from a chosen value must carry the value itself.
- **CR 607.2d**: on a permanent, "the chosen [value]" names the choice its
  linked ability made. That is step 6's, `linked-abilities`; "Linked
  abilities" below says what this note fixes for it.
- **CR 603.7c**: a delayed trigger refers to what the step that created it
  named, players included: Faramir, Prince of Ithilien's "that player"
  reaches its delayed trigger through `Effect::Remember`, once a choose step
  notes what it chose.
- **CR 105.4** (one of the five colors), **205.3m** (the creature types,
  Time Lord the one of two words), **205.2a** (the card types), **700.2e**
  (another player choosing a mode, which `ChoosePlayer` also asks).
- **CR 400.7**: an object chosen and then moved is a new object, which a
  later step does not find unless a rule says so; chosen objects are kept by
  existence (`ObjectRef`), as step 7's decision 1 keeps targets.

## The shapes

What the decisions below recommend, in one place.

```rust
// types/effects.rs

/// The name a choice is kept under, so a later step can read it. Local to
/// one spell or ability. Nearly every card keeps one thing of a kind, so
/// the names are the kinds; a card that keeps two of a kind gets a second
/// name here, with that card (Gluntch's second and third players).
pub enum ChoiceKind { Color, CreatureType, CardType, Number, Player, Objects }

enum Primitive {
    // …
    /// Choose something as this step runs (CR 608.2d) and keep it under
    /// `name`. Nothing else happens.
    ChooseValue { name: ChoiceKind, what: ValueChoice, how: ChosenHow },
}

/// What a choose step asks for.
pub enum ValueChoice {
    Color,                                  // one of the five colors
    CreatureType,                           // one creature type
    CardType(Option<Vec<CardType>>),        // one card type, or one of a printed few ("artifact or enchantment")
    Number { least: u64, most: u64 },       // "a number between 0 and 10"
    Player(PlayerSet),                      // "a player", "an opponent"
    Objects(SelectionFilter, TargetCount),  // "up to one creature"; a divided count also splits an amount among them
}

/// Who makes the choice.
pub enum ChosenHow {
    By(EffectRecipient),  // you, or a target player
    AtRandom,             // drawn from the game's seeded rng; nobody is asked
}

/// What a choose step kept: the answer to a `ValueChoice`, arm for arm.
pub enum ChosenValue {
    Color(Color),
    CreatureType(CreatureType),
    CardType(CardType),
    Number(u64),
    Player(PlayerId),
    Objects { chosen: Vec<ObjectRef>, shares: Vec<u64> },  // `shares` is empty unless an amount was split
}

// A later step reads a name where it would otherwise hold a plain value.
EffectRecipient::ChosenAs(ChoiceKind)  // act on the objects, or the player, kept under the name
AmountExpr::ChosenAs(ChoiceKind)       // the number kept under the name
ObjectFilter::ChosenAs(ChoiceKind)     // an object of the chosen color or type, or one of the chosen objects

// "Creatures with power 3 or greater", "toughness 2 or less": one filter
// for every number read off an object and compared with another number,
// shaped as the counter filter is (`WithCounters { kind, count }`).
ObjectFilter::WithValue { of: ObjectValue, is: ValueIs }
pub enum ValueIs { AtLeast(AmountExpr), AtMost(AmountExpr), Exactly(AmountExpr), LessThan(AmountExpr) }

// state/game_state.rs, on the record of the spell or ability now resolving,
// beside the X chosen as it resolves
pub choices: Vec<(ChoiceKind, ChosenValue)>,
```

Two cards written with them, in the helpers cards use today (`each(filter)`
is "each permanent the filter describes"; `.and` joins two conditions on
one object, as `creature().and(you_control())` is "creature you control"):

```rust
// Expel the Interlopers
sequence([
    choose_number(0, 10),                                                // kept as Number
    destroy(each(creature().and(power_at_least(chosen(Number))))),      // each creature with power at least that
])

// Duneblast
sequence([
    choose_objects(creature(), up_to(1)),                                // kept as Objects
    destroy(each(creature().other_than(chosen(Objects)))),              // each creature other than the one kept
])
```

`other_than` is a new two-line helper for "not one of these", so the
second step reads as the card does; it builds the same filter `.and(non(…))`
would.

## Decisions

### Decision 1 — what a later step names a choice by (item 426)

**Retrofit cost if chosen wrong.** Every card that chooses and then uses
the choice is written against the key. Changing it later is a sweep over
those cards and their tests; no game, log or saved decision moves, since a
key never leaves the card's definition.

**What happens, start to finish.**

1. A spell or ability starts to resolve. The engine already keeps one
   record for whatever is resolving right now (`ResolvingObject`,
   `state/game_state.rs:181`); it holds, among other things, the X chosen as
   an effect resolves (`x_value`, `:201`). The record gains an empty list of
   choices.
2. A choose step asks its question and adds one entry: a key and what was
   picked.
3. A later step that names the key reads the entry.
4. The spell or ability finishes resolving, and the record goes, the list
   with it (CR 608.2n).

It lives on that record because every step is already handed the
resolution's context, which finds the record: nothing new is passed around.
(Kept as a local of the resolver's walk instead, the list would be handed
down to 28 call sites of the amount reader alone.) A spell that makes no
choice pays nothing: an empty list allocates nothing, at resolution or when
a fork copies the game. A CR 615.5 rider and a mana ability resolve no stack
object, so a choice in one is refused, loudly, as an X chosen there is
today: no printed rider chooses, and the 12 mana abilities that choose a
color are `chosen-color-plus-could-produce`'s (step 14).

**The one real choice is the key.** Three cards show the difference:

| | Expel the Interlopers | Gluntch: three players, each used by the next sentence, all different | Cultural Exchange: two sets of creatures, then swapped |
|---|---|---|---|
| **A. a name the card gives** | `number` | `first`, `second`, `third`; the second choice excludes `first` | `theirs`, `ours` |
| **B. the kind of thing, latest wins** | "the number" | "the player" works sentence by sentence; "different" needs every player chosen so far | needs "the first objects" and "the second", a position after all |
| **C. a counted place** | choice 0 | choices 0, 1, 2 | choices 0 and 1 |

- **A. A name the card gives**, `ChoiceName(&'static str)`. A choose step
  writes its name; a step that runs again replaces its own entry (Season of
  Weaving's mode chosen twice). A name nothing wrote this time, because its
  step sat in a branch not taken, reads as nothing: a step acting on it does
  nothing and a filter matches nothing; an amount with no value is a card
  bug and an error. `cards::registry`'s test, which builds every card,
  checks that each name read is written earlier in the same ability, by one
  step, of a kind its reader takes. *Code*: 70–120. *Performance*: a list of
  a few entries, searched by name; nothing when empty. *Upkeep*: a name is a
  string, so a typo fails that test, not the compiler; inserting or
  reordering steps never re-points a reader.
- **B. The kind of thing chosen, the latest winning.** No names, and the
  compiler checks the kind. Two picks of one kind alive at once need a
  second selector, "all so far" or "the nth", which is a position again
  (Cultural Exchange, Tunnel of Love's "the chosen creatures", Gluntch's
  three different players). *Upkeep*: a card that inserts a choice of the
  same kind between a choice and its reader changes what the reader finds.
- **C. A counted place.** The first draft's `ChoiceSite`: the engine numbers
  an ability's choose steps in printed order from 0, and a reader gives the
  number, as `SameInstanceAs(usize)` names a target today. No strings.
  *Upkeep*: a number the author counts; a branch not taken must still
  advance the count; and a step inserted later re-points every reader after
  it, silently where two choices are of one kind.

**Settled: A, a name the card gives** (the owner, at #330's second review
round, 2026-11-15). It is the cache the first round pictured: a key, a
value, a short life. It reads at the call site (`chosen("spared")`), and it
is the answer step 7's decision 3 took for "the sacrificed creature", named
by its act and not by its cost's place, for the reason C fails here.

**The name's type: settled, a closed enum** (the owner, at #330's fourth
review round, 2026-11-15). The owner asked at the third round why the name
is text and not an enum: a text key passed around feels brittle.

- **Text**, `ChoiceName(&'static str)`, as A was first written. Each card
  makes up its own word (`"spared"`), with no engine change per card. A
  misspelled name is caught by the registry's test, not the compiler, and
  no tool finds a name's uses.
- **A closed enum** (taken), as under "The shapes". The names are the
  engine's list, starting with one per kind of thing chosen, which covers
  every card that keeps one thing of a kind. A card that keeps two of a
  kind at once gets a second named variant, added with that card; it is
  still a name, never a position. The compiler checks the spelling, and a
  name's uses can be found. The registry's test still checks that a name is
  written before it is read, by a choice of the kind its reader takes. What
  is given up: the name says the kind ("Objects"), not the card's reason
  for it ("spared").

*Retrofit cost*: none now; after the build, a sweep over each card that
keeps a choice.

### Decision 2 — one shape for a choice of objects

**Retrofit cost if chosen wrong.** The 75 object cards are written one way
or the other. Two registered cards and #326's fixtures move under A.

**What the game piece does.** Duneblast: the player picks up to one
creature; every other creature is destroyed. Final Showdown's second mode:
the player picks a creature they control; it gains indestructible. Feast of
the Victorious Dead: the player picks creatures they control and splits the
counters among them; each gets its share. Each is "choose, then use".

**What #326 built.** The choosing is done by the recipient of the step that
acts on what is chosen (`EffectRecipient::Choose`): Final Showdown is
`gains_keyword(chosen_permanent(…))`, one step that both asks and acts. That
fits only the 13 cards whose next step acts on the chosen object; Duneblast
has no step that acts on its creature at all.

- **A. One shape: every choice is a choose step.** `ChooseValue` with
  `ValueChoice::Objects` asks through #326's `SelectRecipients` and its
  division, unchanged, and keeps the objects with their shares; the steps
  after read the name. Final Showdown and Feast become two steps each. No
  card writes `EffectRecipient::Choose` again: it stays only as the
  question's label on `SelectRecipients`, as the edict's prompt already uses
  it (`engine/resolve.rs:2486`), until that payload's own cleanup. Objects
  chosen at random come free (Last One Standing, The Nipton Lottery).
  *Code*: 60–110: `recipients_for`'s `Choose` arm and
  `chosen_by_the_controller` move behind the step, the two cards and #326's
  fixtures are rewritten. *Upkeep*: one way to write a choice.
- **B. Two shapes, by the card's sentence** (the first draft's
  recommendation): a choice in a sentence of its own is a choose step, a
  choice inside the verb's sentence stays the recipient. *Code*: 20–40.
  *Upkeep*: two ways to write one thing, picked by reading grammar, which no
  check can see.
- **C. Objects wait.** Only today's recipient, plus a way to name it again.
  *Upkeep*: "the rest", copies and sources stay unwritable after the
  subsystem closes.

**Settled: A** (the owner, at #330's third review round, 2026-11-15). A
copy effect's donor keeps its own question (`ChooseCopySource`, Cytoshape
as built), and an edict's "a creature of their choice" stays as built: its
pick is the sacrifice (CR 701.21a).

### Decision 3 — how a step reads a kept value

**Retrofit cost if chosen wrong.** Confined to the resolver and the filter
table; no card changes, since a card writes `chosen("number")` either way.

**The direct way.** A step that names a choice looks it up as it runs, and
everything downstream gets a plain value:

| Where the name sits | What the step does with it | Cards |
|---|---|---:|
| as what the step acts on (`EffectRecipient::ChosenAs`) | hands the verb the kept objects, or the kept player | the 13 objects; 37 players |
| as a number (`AmountExpr::ChosenAs`) | the amount reader returns the kept number | 10 |
| inside a filter (`ObjectFilter::ChosenAs`, or an amount in a comparison) | just before the resolution matches the filter, it writes the kept values in: "of the chosen color" becomes "red", "not chosen" becomes "not one of these three objects", "power at least the chosen number" becomes "power at least 7" | ~57, the 11 "the rest" among them |

- **A. Looked up where the step runs.** Two lookups (what a step acts on,
  an amount) and one small function over `ObjectFilter` that returns the
  filter with its names filled, called at the four places a resolution
  matches a filter: a sweep (`permanents_matching`, `engine/resolve.rs:2676`),
  a count, a choice's candidates and an edict's. The filter table never
  hears about choices. It gains one plain leaf, "one of these objects", for
  a filled set. Amounts in a comparison are read at the same moment, which
  is CR 608.2h's "determined only once, when the effect is applied". An
  effect that outlasts the resolution (a "can't", a replacement) gets its
  filter filled the same way as it is made, so it holds the value it was
  made with: that is the third row's work, and the function is this one.
  *Code*: 110–180 with the comparison below. *Upkeep*: a new place a name can
  sit is one arm of the fill.
- **B. Asked through the filter table** (the first draft's recommendation).
  The table's leaves ask their asker for "the chosen value", so the bundle
  the table is asked with gains two methods and a field marking a
  resolution's reads, and the other askers a refusal. *Upkeep*: every asker
  of the table learns about choices in order to refuse them.

**Settled: A** (the owner, at #330's fifth review round, 2026-11-15). The
first draft priced A as a rewrite of every instruction; that was wrong. Only
a filter needs writing into, because only a filter is handed to code the
resolution does not own.

**Why `TextReferents` exists**, since the first draft leaned on it. A
filter is stored as the card wrote it: "creatures you control", "another
creature". "You" and "this" are not fixed: on a static ability "you" is
whoever controls the permanent now (CR 109.5), on a spell it is the caster.
So whoever asks "does this object match?" must also say who "you" is and
which object "this" is, and `TextReferents` (`engine/object_filter.rs:69`)
is that small bundle. It is what lets one table of leaves serve targeting,
costs, "can't" effects, replacements and resolution. Under A this PR does
not touch it.

**The comparison: settled, one filter with a labelled comparison** (the
owner, at #330's sixth review round, 2026-11-15, on how each spelling read
over six cards). The first draft added `PowerAtLeast` beside the existing
`PowerLE(i32)`. The grid above is one question, a number read off an object
compared with another number, so it is one filter,
`WithValue { of, is }`, shaped as the counter filter already is. `of` is
the type amounts already use for "its power" (`ObjectValue`: power and
toughness today; step 7 adds mana value and counters, and the filter takes
them with no change). `is` says how it is compared and with what: at least,
at most, exactly or less than, each carrying any amount, a printed number,
X or the chosen number. Exact matches are printed (Void's "mana value equal
to that number") and so is a strict one (mentor's "power less than this
creature's power"), which is why a pair of at-least and at-most filters,
the review's first suggestion, would have grown to four. Built here: power
and toughness, all four comparisons, and `PowerLE` folded in (its five
readers in the engine and 24 lines that name it). The table compares plain
numbers; an asker that can read amounts fills them first, as above. Cards
write it through helpers: `power_at_least(3)`, `toughness_at_most(2)`.

### Decision 4 — a player verb's player: item 292 with item 425

**Settled: A** (the owner, at #330's first review round, 2026-11-15).

**What is wrong today.** `resolve_player_for_self` (`engine/resolve.rs:3461`)
finds the one player for the 13 verbs item 292 lists: a mill, the top cards
put into a hand, a discard, a scry, a life gain, a life total set, a loss
and a win of the game, an extra turn, a token's controller and counters a
player gets, and, through `each_player_acted_on` for one player, a draw and
a life loss. For "target player", "that player" and "defending player" it
returns the first player its slice holds, and the controller when the slice
is empty; every other recipient falls to the controller too.

**It is reachable now.** Vindictive Lich (`cards/sets/2017/08_25_c17.rs:114`,
in the stress pool) has three modes, each targeting a different opponent.
At three seats or more, if one targeted opponent leaves the game before the
trigger resolves, that mode resolves with an empty instance (CR 608.2b,
`engine/targeting.rs:777`), and "target opponent loses 5 life" costs the
Lich's controller 5 life. Derived from the code path; the build's first
test shows it failing first (`plans/fails_before_fix.py`), with its twin:
the surviving opponent's mode still resolves.

**How it fits step 7's settled words.** Step 7's decision 4
(`handoffs/resolution-recipients-and-amounts.md`, settled at #295,
2026-10-26) makes every player word one reader, with "nobody" an answer
that does nothing, which is item 292's fix. A chosen player as what a step
acts on is `ChosenAs`, read as a target player is. Its other two places, a
filter's "that player controls" and a group's "you and that player", are
that design's player words, not built yet: they get a chosen arm there.

- **A. Item 292 comes forward, whole.** One reader returns the player the
  step was handed, for every recipient that names one (a target, a chosen
  player, a trigger's player, a remembered one, defending player), the
  controller only for "you", and nobody otherwise: each of the 13 verbs
  does nothing for nobody, as `DealDamage` already does (CR 608.2b's "won't
  affect", 603.6's "unable to be found"). Row 39's two sites, `named_player`
  and `EntersAttacking::Player`, answer nobody the same way.
  `cards::registry` gains a check that no player verb has a recipient naming
  only objects. Step 7's second PR changes which recipients feed the reader,
  as it would have anyway, and loses item 292's 60–80. *Code*: 60–100.
- **B. The chosen player's arm only**; item 292 stays with step 7, its
  verdict changed to "reachable, wrong today". *Code*: ~10.

`player_the_instruction_names` (`engine/resolve.rs:2961`) keeps asking
Akroan Horse's "an opponent gains control" as the step applies, under
`ChoosePlayer`: a choice the rules imply inside one step, with nothing kept.

### Decision 5 — the split, and where the rest goes

**Settled: A, three rows** (the owner, at #330's first review round,
2026-11-15). The intake writes the split into `choices-architecture.md`'s
"The build, sized" table before the build, which also moves Selvala's
Stampede, a vote, off this row.

The design's table gives the row 560–870 code and 450–700 tests, plus item
426's 80–150 and item 394's leaf; at the table's 1.7× that is 1,700–2,700
for the row. Re-counted piece by piece for this PR. A count set at a design
stop is read with its top as the expected size
(`references/sizing-calibration.md#set-at-a-design-stop`: 20 rows, a mean
of 1.06 times the top, 0.53–1.70), so no ratio goes on these.

| Piece | Code | Tests |
|---|---:|---:|
| The kinds: `ChooseColor`, `ChooseCreatureType`, `ChooseCardType`, `ChooseNumber`, and `ChooseOpponent` widened to `ChoosePlayer`; `DecisionOption::CreatureType` and `CardType`; each with its subject, `as_str`, question, option label, why, setup line, decision-log text and replay; `decision::ask`; the random agent | 140–220 | 50–90 |
| The choose step: `ChooseValue`, `ValueChoice`, `ChosenHow`; each kind's options (the five colors, CR 205.3m's list with its missing fifteen, the card types or a printed few, a number's range, a player set, an object filter) and at random | 170–260 | 100–170 |
| Decision 1: the list on the resolving record, names, the lookups, `Remember`'s note, the chain, the registry's checks | 70–120 | 60–100 |
| Decision 2: the object choice behind the step; Final Showdown, Feast and #326's fixtures rewritten | 60–110 | 40–80 |
| Decision 3: the fill, the "one of these objects" leaf, `WithValue` with `PowerLE` folded in | 110–180 | 80–130 |
| Decision 4: item 292 and item 425, the Lich and its twin, item 292's two boards | 60–100 | 60–100 |
| Expel the Interlopers registered and pooled; the fixtures below | 130–190 | 130–210 |
| The dev GUI's words and tests | 15–40 | 15–30 |
| **This PR** | **755–1,220** | **535–910** |

**1,290–2,130.** `CreatureType` goes under `counted_enum!` for its list,
which re-indents its ~300 variants: a whitespace-only change, read with
`git diff -w` and counted apart.

- **A. Three rows.** *This PR*: a color, a creature type, a card type, a
  number, a player and objects, chosen by you, by a player the text names,
  or at random, with decisions 1 to 4. *The second* (the brief's): several
  choosers at one step — votes (701.38, 33 cards, the per-player walk's
  starting seat), piles (700.3, 21), the 16 "each player chooses" values and
  their readers ("a type chosen this way"), the word kind (12 of its 17 sort
  or ask each player; "odd or even"'s predicate), a choice excluding an
  earlier one's pick (Gluntch, Fortunate Few), and an open "choose a number"
  (Void, Squee's Revenge), whose unbounded pick needs a policy for the
  random agent and the GUI; a guess of 1,000–1,600. *A third, new*,
  `values-kept-past-the-resolution`: an effect that outlasts the resolution
  holding the value it was made with (~10), a change's parameter from a
  choice (47) and the inline ones (34), with the two kinds read only that
  way (a basic land type, a keyword) and the card name (its catalog, and
  whether its names are the process's registry or the game's, since a game
  is built from a `CardRegistry` passed in, `scenario/build.rs`); a guess of
  1,200–2,000, which may split at its brief. The subsystem closes with the
  third.
- **B. Two rows, as the brief.** Every one-chooser kind here, four of them
  shown only by a fixture no printed card backs: about 1,620–2,640, past
  the band's top.

## Linked abilities: what this note fixes for step 6, and what it leaves

The review asked whether `linked-abilities` should come into this design.
**No, with one thing fixed here.**

*What it is.* CR 607: "As this enters, choose a color" on one ability and
"creatures of the chosen color get +1/+1" on another of the same permanent
(202 cards choose as a permanent enters). The choice is kept on the
permanent, for as long as it stays, and belongs to that pair of abilities.
It is a row of its own, a design and three builds, and holds more than
chosen values: the cards exiled with a permanent, the cost paid as it
entered.

*Why it stays out.* Its questions are about where a permanent keeps a
record and how long, which this PR never meets: this list lives for one
resolution. Under decision 3's A the two share little: what a kept value is
(`ChosenValue`) and the function that writes kept values into a filter.

*What is fixed here for it.* Both are written against "a list of kept
choices", never against the resolution, so step 6 can hand the same
function a permanent's list. And a name may turn out to be the link CR
607.2d describes, the same name written by one ability and read by another
of the same card; that is step 6's decision, and nothing here forecloses it.

## The binding table

| State | Bound | Live or latched | Stored | Read by | Ends |
|---|---|---|---|---|---|
| what a choose step kept | as the step runs (CR 608.2d) | latched: determined once (608.2h); objects kept by existence (`ObjectRef`, CR 400.7) and found only while each is the same object | the resolving record's `choices`, by name | a step acting on it, an amount, a filter as it is filled | with the resolving record (CR 608.2n) |
| a value an effect keeps past the resolution | — | — | not built: refused | — | the third row |

## The kinds, as decision 7 settled them

Each is asked through `decision::ask`, names its subject (`source`, the resolving
object), and is answered unasked when one option is legal. Clients: a
person at the four-seat GUI and an AI client, both reading the typed
options; the dev GUI words each now.

| Kind | Options | Asked by |
|---|---|---|
| `ChooseColor { source }` | `DecisionOption::Color`, the five (CR 105.4) | `pick_n` |
| `ChooseCreatureType { source }` | `DecisionOption::CreatureType`, CR 205.3m's list | `pick_n` |
| `ChooseCardType { source }` | `DecisionOption::CardType`, CR 205.2a's or a printed few | `pick_n` |
| `ChooseNumber { source, values }` | `values` an `XOffer::Interval`, as `ChooseXValue` and `ChooseAmountToPay` spell a range (`oracle/x_values.rs:87`) | `pick_number` |
| `ChoosePlayer { source }`, widened from `ChooseOpponent` | `DecisionOption::Player`, the players the choice admits: Akroan Horse's opponent, CR 700.2e's chooser of a mode, and "choose a player" | `pick_n` |
| objects | decision 8's `SelectRecipients`, unchanged | `pick_n` |

## What builds without a decision

- **`CreatureType` gains CR 205.3m's missing fifteen** (Bison, Drix,
  Echidna, Hedgehog, Hero, Kangaroo, Lemur, Lobster, Moogle, Platypus, Qu,
  Sorcerer, Symbiote, Utrom, Villain): `tmnt.txt` lists 316 and the enum
  301, and a choice offers the rule's list. `CreatureType` goes under
  `counted_enum!` for `ALL`.
- **Expel the Interlopers** in a new `cards/sets/2023/09_08_woe.rs`, its
  first printing (Wilds of Eldraine), registered and pooled, its ruling in
  `rulings-ledger.json`, written as under "The shapes".
- **The fixtures**, in `cards::fixtures::choices`, each naming the printed
  card it models:
  - a color: Wash Out, "Return all permanents of the color of your choice
    to their owners' hands";
  - a creature type: Kindred Dominance, "Choose a creature type. Destroy all
    creatures that aren't of the chosen type";
  - a card type: World Queller, "you may choose a card type. If you do, each
    player sacrifices a permanent of their choice of that type", the edict's
    candidates filled from the kept type;
  - a player you choose: Gluntch's "Choose a second player to draw a card"
    and "choose a third player to create two Treasure tokens", a draw and a
    token's controller (its rule that the three differ goes to the second
    row);
  - a player at random: Truth or Consequences' last sentence, "choose an
    opponent at random. [It] deals 3 damage to that player", without its
    vote;
  - objects at random, then several steps on them: The Nipton Lottery,
    "Choose a creature at random. You gain control of that creature until
    end of turn. Untap it. It gains haste until end of turn. Then destroy
    all other creatures";
  - objects you choose, and the rest: Duneblast, "Choose up to one
    creature. Destroy the rest";
  - the brief's chain, "choose a creature you control; it gets +2/+2 and
    gains trample until end of turn", which no card prints in that phrasing:
    one choose step and a two-change `ContinuousChange` reading its name.

## Atoms and their boards

`python plans/specdb.py next --rule` for each rule the brief names, read
2026-11-14.

| Atom | What it asks | Planned test |
|---|---|---|
| `ATOM-608.2d-001` (partly covered) | choices not made while casting are announced as the effect applies, and no illegal option is chosen | Expel's board: the number asked as it resolves, 0 through 10 offered (its ruling); stays `COVERS-PARTIAL`, its own board being a "you may sacrifice" |
| `ATOM-608.2d-002` (partly covered) | an untargeted division gives each at least one | #326's tests, moved with Feast to the choose step (decision 2) |
| `ATOM-608.2e-001` | several players' choices in APNAP order, then the action at once | the second row's: several choosers at one step |
| `ATOM-105.4-001` | a color choice offers the five, and refuses "colorless" | the Wash Out board: the offer equals the five colors. A sixth answer cannot be written (`Color` has five arms and no colorless), which the test's comment says rather than claiming a refusal it never reaches |
| `ATOM-205.3m-001` | Human is a creature type, Forest is not | the Kindred Dominance board: the offer equals CR 205.3m's list as `tmnt.txt` prints it, read by the test, Human and Time Lord among it; Forest is a `LandType` and cannot be offered, said the same way |
| 201.4, 700.2e | — | covered already |

And the boards the binding table and decision 4 owe:

- **Chosen objects are kept by existence (CR 400.7)**: a chosen creature
  moved between the choice and the step that names it is not found, and
  that step does nothing. A rule test, no printed card.
- **The player boards run at three seats**, so "choose an opponent" has two
  candidates: one asked, one drawn from the seeded `rng`.
- **Vindictive Lich at three seats**, failing first, and its twin.

## Gates and predictions

The gates that can fail: `cargo build --all-targets` with zero warnings;
`cargo test`; `cargo clippy --all-targets -- -D warnings` on both crates;
the dev GUI's tests, since `DecisionKind` gains arms the window words; every
`check_*.py`, `check_rulings.py` for Expel included.

**The A/B, three arms** (`fuzz_ab --require`): `main`; the branch with Expel
unregistered; the branch. The middle arm asks no new question, so it is
predicted `IDENTICAL` to `main` except in a game where a player verb's
player is gone (decision 4, at three seats or more), which its dumps will
name game by game; the branch moves with Expel alone. Final Showdown and
Feast ask the same questions in the same order under decision 2's A, so
they move nothing.

## Filed at the close-out, if the decisions stand

Each with its home:

- The second row (several choosers): the 16 "each player chooses" values,
  the word kind, a choice excluding an earlier pick (Gluntch, Fortunate
  Few), an open "choose a number" (Void, Squee's Revenge).
- The third row, `values-kept-past-the-resolution`: an effect that outlasts
  the resolution and reads a chosen value (The Black Gate, Conjurer's Ban,
  Twins of Discord, Zombie Boa); a change's parameter from a choice
  (Akroma's Blessing, Terraformer, Linvala) and the inline ones (Mistform
  Dreamer); a chosen object a lasting effect or a later trigger names
  (Gideon's Sacrifice, Azra Oddsmaker); the basic land type and keyword
  kinds; the card name, its catalog's question, and Garth One-Eye's "a card
  name that hasn't been chosen", decision 7's record of what an ability
  chose before.
- Step 7's notes (`resolution-recipients-and-amounts`): its player words'
  chosen arm, for "that player controls" and "you and that player"; and the
  chosen object as the thing a number is read off or the thing that deals
  damage, for Crush Underfoot's "it deals damage equal to its power".
- `token-copies`: a token copy of a chosen object (Season of Weaving,
  Calamity, Galloping Inferno).
- `information-model`: a chosen value read over a hidden zone (62 filters,
  most of the card names), and The Toymaker's Trap's secret number.
- Step 9's cheap gaps, item 396: "a player with the most life" as a choice's
  admission (The Black Gate).
- `chosen-color-plus-could-produce`: a choice in a mana ability, which
  resolves no stack object.
- Item 394 narrowed to mana value, which arrives with step 7's amounts, and
  "tapped"; items 425 and 426 closed; item 292 closed.

---

# The second row — `several-choosers-at-one-step`, the design note

**Opened 2026-11-17** on `choices/several-choosers-at-one-step` off
`6f1f13f1` (#333's merge), as `roadmap-v2.md` §3a step 5's row
`several-choosers-at-one-step`, the second of this file's three. Every line
cited below was read against `6f1f13f1`, and every Scryfall count on
2026-11-17 under `game:paper -is:funny`, `total_cards`, unique cards.

**Revised after the design stop's fresh read** (the `owner-review` agent,
2026-11-17). Fixed in place:

- a walk opens its kept choice empty, so Fortunate Few's first chooser is
  offered every candidate, and an amount reads exactly one answer
  (decision 1);
- `ChosenHow` is a walk plus who answers, the vote a mark on the walk, so
  no case is spelled twice, and each order is built in the PR of its first
  card (decision 2);
- "they" is the player an answer is for, which also covers Gluntch's and
  Choice of Damnations' single chooser (decision 3);
- the players who gave an answer are a narrowing of `PlayerGroup`, which
  only a resolution reads, not a `PlayerSet` arm (decision 4);
- every vote asks `Vote`, as `choices-architecture.md`'s decision 7
  settled, and a voter's count of votes waits for `player-scoped-values`
  (decision 5);
- a choose step's words are written on the step and checked against its
  rules text, since Gluntch's cannot be cut by line (decision 7);
- item 435's exhaustive list replaces the registry's scan of a step's
  printed form, which could not see the new readers;
- CR 800.4g, the rule that watches a chooser who has left the game, is
  decision 11;
- names, the census lines and the filings the fresh read corrected.

Put to the owner: decision 10's option D (the piles off the route, since
no deck card is a pile), and whether each one-card arm is the right general
shape (decisions 2 and 4, and Gluntch's exclusion).

## Where this row sits

Still inside one stack object's resolution, the RESOLVE step of
`engine-map.md#11-the-path-in-one-sketch`: `resolve_top_of_stack` →
`resolve_effect` → each step in turn. The first row built the choose step:
it asks one player (`ChosenHow::By`, read by `chooser_named_by`,
`engine/resolve.rs:780`, which errs at `:804` when its recipient names
several), or draws at random, and keeps one value under a name
(`ResolvingObject::choices`, `state/game_state.rs:205`; `keep_under`,
`engine/resolve.rs:820`). This row lets that step ask several players, in
the order the CR names, keeps each answer with its player, and lets a later
step read the answers together or by a selection.

Three neighbors bound it:

- **The edict** (`EffectRecipient::ChosenBy`, `choose_as_it_applies`,
  `engine/resolve.rs:2698`) already walks a group in APNAP order, "each
  player sacrifices a creature of their choice", and stays as built: its pick
  is the sacrifice (CR 701.21a; the first row's decision 2).
- **Step 7, `resolution-recipients-and-amounts`**, settled
  `Effect::ForEach(Over::Players(..), body)` and `PlayerAntecedent` (its
  note's decisions 4 and 5) and has built neither: `Effect::ForEach` is
  refused at `engine/resolve.rs:521`. A reader that runs a body once per
  answer ("for each player who chose money, you and that player each create
  a Treasure token") is that construct's, and waits for it.
- **Step 8, `information-model`**, hides a secret answer from the other
  seats. This row writes the mark and the reveal its decision 7 took
  (`handoffs/information-model-design.md#decision-7--a-choice-kept-from-the-other-players`).

## The intake

`python plans/check_state_of_play.py --slotted several-choosers-at-one-step`
lists three items, and no backlog entry names this row as its owner:

- **Item 429, "one choose step has one chooser"**: a value each player or
  each opponent chooses, a word from a printed few, a choice excluding an
  earlier one's pick (Gluntch, the Bestower; Fortunate Few), and an open
  "choose a number" (Void, Squee's Revenge, Choice of Damnations). Taken:
  decisions 1, 2, 3, 6 and 8, and "What builds without a decision".
- **Item 435, "an amount or a player word that reads a choice never made is
  an error"**: CR 608.2b's "any part of the effect that requires that
  information won't happen" holds today where a verb, a filter or a later
  choose step can tell, and an amount or a player word errs. Taken: "What
  builds without a decision", each step's list of the names it reads. Sized
  ~120–200; re-counted below.
- **Item 437, "two choices of one kind from one source are one question to
  a client"**: a choose step's question carries its kind and its source,
  so Gluntch's three player choices read alike to an AI client. Taken:
  decision 7's label. Sized ~40–80; re-counted with the label.

`backlog.md#24-voting-and-the-decisionprovider-choice-shapes`' finding, that
the per-player walk has no starting seat, is decision 2's. Its owner line
names `modes-plus-choice-kinds`, this row's parent.

**One disagreement between the plans, settled here.** Step 7's decision 5
filed friend or foe at `multiplayer-rules`, as a body of its per-player
`ForEach`; the first row's note slotted the word kind here. Both hold: the
word, the walk that asks it and "each friend" as a group are this row's,
and a friend's own action that reads their own player ("each friend puts a
+1/+1 counter on each creature they control") is a `ForEach` body, filed
with step 7.

## The rules' atoms

`python plans/specdb.py next --rule` for each rule the brief names, read
2026-11-17.

| Atom | What it asks | Planned test |
|---|---|---|
| `ATOM-608.2e-001` | each step's choices in APNAP order, then that step processed at once | Divine Reckoning at four seats on another player's turn: the active player is asked first, then turn order, and every creature not chosen is destroyed in one event. `COVERS-PARTIAL`: its board is "each player discards a card, then each player draws", whose discard asks through `DiscardChooser`, not a choose step |
| `ATOM-101.4-001` | APNAP choices for "each player sacrifices a creature", then one event | the edict's walk, which already does this (`choose_as_it_applies`): a two-seat board that records the order asked and reads one `CreatureDied` batch. `COVERS` |
| `ATOM-101.4c-001` | Smallpox's four steps, each asked APNAP, never batched across steps | `COVERS-PARTIAL` on Harsh Mercy beside an edict fixture: the type walk finishes before any creature is destroyed; Smallpox's discard and its land sacrifice are the discard chooser's |
| `ATOM-700.3a-001` | each object in exactly one pile | the third PR's (decision 9): Do or Die's separation, its answer refused when an object is in both piles or neither. `COVERS-PARTIAL`: its board is Fact or Fiction, whose cards sit in a library (filed below) |
| `ATOM-700.3c-001` | objects in piles do not change zones while the piles are made | the third PR's: Do or Die's board asserts no zone change between the separation and the destroy. `COVERS-PARTIAL`, as above |
| CR 701.38 | nothing uncovered | the second PR's boards name CR 701.38a–c directly: Plea for Power's walk from a player who is not active, its tie (the ruling's "more than one choice received the most votes"), Council's Judgment asked as a vote and not a pick. 701.38d's two votes at once are step 10's, with Brago's Representative (decision 5) |

## Counted

### What several choosers look like on the cards

Grouped by what the shapes must do. Queries under the table.

| What the cards do | Cards | Example | Holds today |
|---|---:|---|---|
| each player or each opponent chooses, each for themselves | 59 | Harsh Mercy, Divine Reckoning | one chooser (`engine/resolve.rs:804`) |
| …of those, objects relative to the chooser ("a creature they control", "they don't control") | 32 | Divine Reckoning, Call to the Void | `ChoiceScope::ChoosersPermanents` on the edict only |
| …read back as "the rest" | 18 | Divine Reckoning, Cataclysm | `other_than(the_chosen_objects())` over one chooser's |
| starting with a named player, in turn order | 41: 25 votes, 16 others | Council's Judgment; Whims of the Fates, Thieves' Auction | nothing |
| a named player first, then each other player in APNAP order | 1 | Fortunate Few | nothing |
| one player answers for each player ("for each player, choose", "for each player, you choose") | 16: friend or foe 5, an object of that player's 10, piles 1 | Pir's Whim; The Eternal Wanderer; Make an Example | nothing |
| a vote (CR 701.38) | 33: open 27, secret 6 | Plea for Power; Mob Verdict | nothing |
| …what is voted for: a word, objects, a player, a color | 26, 4, 2, 1 | Tyrant's Choice; Council's Judgment; Círdan the Shipwright; Council Guardian | — |
| a card that changes voting: an additional vote, "whenever players finish voting", choosing how each player votes | 3, 3, 1 | Brago's Representative; Grudge Keeper; Illusion of Choice | — |
| piles (CR 700.3) | 27 printing "piles", 3 more "a face-down pile and a face-up pile" | Fact or Fiction, Do or Die | nothing |
| …each player's own piles | 3 | Make an Example, Bend or Break, Whims of the Fates | — |
| …three piles; piles in combat | 1; 2 | Whims of the Fates; Camouflage, Raging River | — |
| …one pile face down | 9 | Curator of Destinies | — |
| a choice made secretly | 18: 12 revealed in the same step | Menacing Ogre, Prisoner's Dilemma | nothing |
| "choose a number" with no top printed | 10 | Void, Choice of Damnations, Wheel of Misfortune's "0 or greater" | `XOffer::Interval` has a top |
| a choice excluding what this resolution chose before | 3 | Gluntch, the Bestower; Dack Fayden, Helping Hand; Fortunate Few | nothing |

**What reads several answers afterwards.** By hand over the cards above; a
card can sit in two rows.

| Reader | Cards | Example | Holds today |
|---|---:|---|---|
| any answer: "a type chosen this way", "each creature chosen this way", "the rest" | about 40 | Harsh Mercy, Divine Reckoning, Call to the Void | `ChosenAs` over one answer |
| the players who gave one answer: "each friend", "each foe", "each player who chose silence", "each player with the highest number" | 11 | Regna's Sanction, Prisoner's Dilemma, Menacing Ogre | nothing |
| a count of answers: "for each strength vote", "the number of truth votes" | 11 | Lieutenants of the Guard, Capital Punishment | nothing |
| the answer that wins: "if time gets more votes", "or the vote is tied", "with the most votes or tied" | 18 | Plea for Power, Council's Judgment | nothing |
| a number picked out: "the highest number", "that much" | 3 | Menacing Ogre, Wheel of Misfortune, Itazura, Lingering Wick | nothing |
| whether every answer is one: "if each opponent chose silence" | 1 | Prisoner's Dilemma | nothing |
| a player's own answer, read inside a later choice: "for each opponent, you choose one of their piles" | 3 | Make an Example, Bend or Break, Whims of the Fates | nothing |
| a player's own answer, read by a step acting on that player or their object: "returns cards of that type", "the color its controller chose", "the creature they chose" | 3 | Grave Sifter, Selective Obliteration, Order of Succession | nothing; step 7's `ForEach` and `PlayerAntecedent` |
| each answer with its player, one at a time: "for each money vote, … owned by the voter", "for each player who chose fame" | 6 | Expropriate, Elrond of the White Council, Master of Ceremonies, Seize the Spotlight, Mob Verdict, Círdan | `Effect::ForEach`, step 7's |

Queries: `o:/each (player|opponent) (secretly )?chooses/` (59);
`o:/each (player|opponent) (secretly )?(chooses|votes for) [^.]*\bthey (control|don.t control)\b/`
(32); `o:/(destroy|exile|sacrifices?|return)[^.]* the rest\b/ o:/each (player|opponent) chooses/`
(18); `o:"starting with you, each player"` (41) and `o:/starting with you/ -o:vote`
(16); `o:"for each player, choose"` (13) and
`o:/for each (player|opponent), you choose/` (3); `o:"each friend"` (5);
`o:"votes for"` (33), `o:"secretly votes"` (6), `o:"gets more votes"` (14),
`o:/for each [a-z ]+ vote\b/` (11), `o:"most votes"` (4),
`o:/votes? (they|you) received/` (2), `o:"the voter"` (2),
`o:vote -o:"votes for"` (8, Basri's Aegis a false hit on "Devoted");
`o:piles` (27), `o:"two piles"` (19), `o:"three piles"` (1),
`o:pile -o:piles` (12, of which Atris, Fortune's Favor and Sauron's Ransom
make piles); `o:secretly` (18),
`o:/then (those|the) (votes|choices|numbers|cards) are revealed/` (12);
`o:/choose[s]? a number(\.| or greater| greater than 0)/` (7) with Wheel of
Misfortune, Itazura and Mindblaze read by hand;
`o:/(each player|each opponent|for each player|each friend|each foe|player) (who|with the) (chose|highest|lowest|voted)/`
(10, of which 5 read an answer and the rest a life total or a finished
vote); `o:/(its controller|their controller|that player|they) chose\b/` (3).
The vote, pile and value populations are
`choice-kinds-census.py`'s too: 33, 21 and 16, the census missing the
piles a card makes from a face-down and a face-up half.

### The rulings read

- **Council's Judgment** (2020-08-07): "Because the votes are cast in turn
  order, each player will know the votes of players who voted beforehand";
  each player must vote and can't abstain; "None of the candidate permanents
  are targeted"; "In the unusual case that no permanents have any votes,
  nothing is exiled."
- **Plea for Power** (2014-05-29): "'the vote is tied' refers only to when
  there is more than one choice that received the most votes … 3 votes to 1
  vote to 1 vote, the vote isn't tied."
- **Brago's Representative** (2014-05-29): cumulative, "you'll vote three
  times per vote" with two; "You make all your votes at the same time"; only
  spells and abilities that use the word "vote".
- **Fortunate Few** (2017-08-25): "After you choose …, each other player
  does so in turn order starting from the player whose turn it is. Each
  player will know the choice made by previous players."
- **Divine Reckoning** (2011-09-22): "Starting with the player whose turn it
  is, each player chooses a creature in turn order."
- **Pir's Whim** (2018-06-08): you choose for yourself too; friends act
  before foes, each group's choices in turn order from the active player,
  and nothing happens between.
- **Make an Example** (2022-04-29): "'Their chosen pile' means the pile of
  their permanents that you chose."
- **Liliana of the Veil** (2022-09-09): "A pile can be empty. If the player
  chooses an empty pile, no permanents will be sacrificed."
- **Call to the Void** (2022-06-10): each player writes their choices down
  and all reveal at once; "If a creature is chosen more than once, it is
  destroyed only once."
- **Wheel of Misfortune** (2020-11-10): the numbers kept secret until all
  are revealed; "if the four players … reveal 4, 7, 7, and 7, Wheel of
  Misfortune will deal 7 damage to each of the last three players".
- **Menacing Ogre** (2004-10-04): "You have to choose zero or a positive
  number."
- **Choice of Damnations** (2005-06-01): the opponent chooses that many
  permanents and sacrifices the rest; a number greater than what they
  control sacrifices nothing.

### The rules

- **CR 608.2e**: each step's choices are made in APNAP order, then that step
  is processed at once, then the next step's. **CR 101.4**, with 101.4b (a
  player knows the earlier players' choices) and 101.4c (one player's
  several choices are made in the order written).
- **CR 701.38a–d**: a vote starts with a specified player and goes in turn
  order (a); its options are objects, words with no rules meaning, or other
  values (b); "voting" means only an actual vote (c); a player's several
  votes all happen at once (d).
- **CR 700.3a–d**: each object is in exactly one pile; a pile is not an
  object; objects in piles do not change zones, a graveyard keeping its
  order; a pile may be empty.
- **CR 107.1, 107.1b**: numbers are integers, and no player chooses a
  negative one. That is the whole bound on an open "choose a number".
- **CR 608.2b** and **608.2h**, as the first row read them: a choice never
  made is information that cannot be found, and an answer is fixed when
  given.
- **The rules that watch these**: CR 603.2 and 603.3b, since "whenever
  players finish voting" triggers on the vote as one event (Grudge Keeper,
  filed below); CR 616.1's and 101.4's own order for a batch's replacement
  choices, which a walk's single event then meets unchanged; CR 406.3, an
  exiled face-down card, which the face-down pile cards lean on.

## The shapes

What the decisions below recommend, in one place. Helpers in the cards'
style follow.

```rust
// state/game_state.rs, on ResolvingObject (decision 1)
pub choices: Vec<KeptChoice>,

// types/effects.rs

/// What one choose step kept: an answer for each player it asked, in the
/// order they answered (CR 101.4). Opened empty as the step starts, so its
/// own later questions read the answers so far; one chooser keeps one
/// answer.
pub struct KeptChoice { pub kind: ChoiceKind, pub answers: Vec<KeptAnswer> }

/// One answer and the player it is for: the walk's player, or the one
/// chooser. `None` for a draw at random that is for nobody.
pub struct KeptAnswer { pub for_player: Option<PlayerId>, pub value: ChosenValue }

/// Decision 2. Who answers, and over which players. `walk: None` is one
/// answer, as every first-row step is.
pub struct ChosenHow { pub walk: Option<ChoiceWalk>, pub answered_by: AnsweredBy }

/// "Each player", "each opponent", "starting with you, each player": an
/// answer for each player of the group, in `order`. `is_a_vote` marks CR
/// 701.38's vote, which only a vote's readers and changers see (701.38c).
pub struct ChoiceWalk { pub players: PlayerGroup, pub order: WalkOrder, pub is_a_vote: bool }

pub enum WalkOrder {
    Apnap,                            // CR 101.4, 608.2e
    TurnOrderFrom(EffectRecipient),   // "starting with you" (CR 701.38a)
    FirstThenApnap(EffectRecipient),  // "choose …, then each other player chooses"
}

pub enum AnsweredBy {
    Player(EffectRecipient),  // "choose", "target opponent chooses", "for each player, choose"
    EachForThemselves,        // "each player chooses"; a walk only
    AtRandom,                 // "at random"; nobody is asked
}

// Decision 3: "they" in "a creature they control", the player the answer
// is for: the walk's player, or the one chooser. A first arm on step 7's
// settled antecedent type.
PlayerRef::Named(PlayerAntecedent::ChoosingFor)

// Decision 4: a later step reads any answer through today's `ChosenAs`,
// or a selection of them:
pub struct AnswerSelection { pub kind: ChoiceKind, pub which: AnswerIs }
pub enum AnswerIs {
    Word(ChoiceWord),  // "each friend", "for each strength vote"
    Highest,           // "the highest number"
    MostChosen,        // "the most votes or tied for most votes"
}
PlayerGroup { relation, named, answered: Option<AnswerSelection> }  // "each foe", "each player with the highest number"
AmountExpr::CountOfAnswers(AnswerSelection)       // "for each strength vote"
AmountExpr::NumberAnswered(AnswerSelection)       // "the highest number", "that much"
ObjectFilter::AnsweredObject(AnswerSelection)     // "each permanent with the most votes"
Condition::IsAmong(PlayerRef, PlayerGroup)        // "if you are one of those players"
Condition::EveryAnswer(AnswerSelection)           // "if each opponent chose silence"
Condition::WinsTheVote(ChoiceKind, ChoiceWord)    // "if time gets more votes"

// Decision 6: a word with no rules meaning, a variant per printed word,
// added with its card, kept as `ChoiceKind::Word`.
pub enum ChoiceWord { Friend, Foe, Silence, Snitch, Time, Knowledge, Strength, Numbers /* … */ }
ValueChoice::Word(&'static [ChoiceWord])
ChosenValue::Word(ChoiceWord)
DecisionKind::ChooseWord { label }, DecisionOption::Word(ChoiceWord)

// Decision 5: every vote is one question kind, whatever is voted for.
DecisionKind::Vote { label: ChoiceLabel, vote_count: u32 }

// Decision 7: the mark, the reveal, and the question's label.
Primitive::ChooseValue { kind, what, how, secret: bool }
Primitive::RevealChoices(ChoiceKind)  // "then those choices are revealed"
GameEvent::ChoicesRevealed { source: ObjectId, kind: ChoiceKind, answers: Vec<KeptAnswer> }
pub struct ChoiceLabel { pub source: ObjectId, pub words: &'static str, pub secret: bool }
DecisionKind::ChooseColor { label: ChoiceLabel }  // and each choose kind, in place of `source`

// Decision 8: an open "choose a number".
ValueChoice::Number { least: u64, most: Option<u64> }
XOffer::NoTop { at_least: u64 }

// Decision 9: piles.
ValueChoice::Piles { of: ObjectFilter, count: u32 }   // "separate … into two piles"
ValueChoice::OneOfThePiles                            // "the pile of that player's choice"
ChosenValue::Piles(Vec<Vec<ObjectRef>>)               // kept as `ChoiceKind::Separation`
// the pile chosen is kept as `ChosenValue::Objects` under `ChoiceKind::ChosenPile`
DecisionKind::SeparateIntoPiles { label, pile: u32 }, DecisionKind::ChoosePile { label },
DecisionOption::Pile(Vec<ObjectId>)
```

Four cards written with them:

```rust
// Harsh Mercy, its "can't be regenerated" ahead of the destroy as
// Plague Spores writes it
sequence([
    choose_creature_type(each_player_chooses()),                      // APNAP, an answer each
    cant_regenerate(each(creature().and(non(creature_type(the_chosen_creature_type()))))),
    destroy(each(creature().and(non(creature_type(the_chosen_creature_type()))))),
])

// Divine Reckoning
sequence([
    choose_objects(each_player_chooses(), creature().and(they_control()), exactly(1)),
    destroy(each(creature().and(non(one_of_the_chosen())))),          // the rest: every answer's
])

// Menacing Ogre's trigger
sequence([
    choose_number_at_least(each_player_chooses(), 0).secretly(),
    reveal_the_choices(ChoiceKind::Number),
    lose_life(each_player_who_answered(Number, Highest), the_highest_number_chosen()),
    Effect::Conditional(you_are_among(each_player_who_answered(Number, Highest)), Box::new(put_counters(this_object(), PlusOnePlusOne, 2))),
])

// Plea for Power: "if knowledge gets more votes or the vote is tied" is
// "time does not win", with two options
sequence([
    vote(starting_with(you()), [Time, Knowledge]),
    Effect::Conditional(wins_the_vote(Time), Box::new(take_an_extra_turn_after_this_one(you()))),
    Effect::Conditional(Condition::Not(Box::new(wins_the_vote(Time))), Box::new(draw_cards(you(), 3))),
])
```

`creature_type(the_chosen_creature_type())` reads every answer, so over
four answers it is "a type chosen this way"; with one, it is the first
row's "the chosen type". `one_of_the_chosen()` is the same: every answer's
objects. `the_chosen_number()` is not: an amount reads exactly one answer
(decision 1), and "the highest number" says which.

## Decisions

Each in `/review-round`'s format. Retrofit cost first, then the options,
then the recommendation.

### Decision 1 — how a step with several choosers keeps its answers

**Retrofit cost if chosen wrong.** The kept list is read in four places the
first row built (`kept_under`, `kept_as_recipients`, `filter_as_read_now`,
the amount reader) and every reader this note adds. Changing the shape
after the build is a sweep over those readers; no card, log or save
moves, since nothing outside one resolution sees the list.

**What the keep must hold.** From the census table: one answer from each
of several players (59 choosing, 33 voting); several answers from one
player at once, a voter's several votes (CR 701.38d) or one player's two
picks (Call to the Void's "a creature they control and a creature they
don't control", CR 101.4c); an answer for each player made by one player
(13, friend or foe); and who gave or got each answer, which every
"players who" reader needs and which step 8 stamps (each secret answer
"seen by its chooser alone").

- **A. Every kept choice is a list of answers, each with its player.**
  `KeptChoice { kind, answers }`, `KeptAnswer { for_player, value }`, as
  under "The shapes". One chooser keeps a list of one, so the first row's
  choices are a special case and not a second shape. A reader of "the
  chosen color" reads every answer: one answer is today's behavior, several
  is "a color chosen this way". A voter's two votes are two answers with one
  player. Two edges are fixed here, from the fresh read:
  - **A walk opens its kept choice, empty, before its first question**, and
    adds each answer as it is given. So its own filter and options read the
    answers so far (CR 101.4b): Fortunate Few's "that hasn't been chosen
    this way" offers you every candidate, then leaves your pick out for the
    next player. "Kept, no answers yet" is the empty set, so "the rest" over
    a walk nobody answered is everything; "never made" (CR 608.2b: that part
    won't happen) stays what a one-chooser step leaves when it has no one to
    ask (decision 11). The registry's walk records a step's name before it
    reads that step's own filter.
  - **An amount reads exactly one answer.** "The chosen number", "that
    many" read the one answer a one-chooser step keeps; after a walk or a
    vote the registry's test refuses it, and the card says which number
    (`NumberAnswered`, decision 4).

  *Code*: 80–130, the four readers moved to the list. *Performance*:
  one more `Vec` per kept choice, empty unless a step ran; a clone copies
  the same entries plus their players. *Upkeep*: one shape for every choose
  step.
- **B. A per-player arm on the value.** `ChosenValue::Each(Vec<(PlayerId,
  ChosenValue)>)` for a step with several choosers; one chooser stays as
  today. *Code*: 60–100. *Upkeep*: every reader matches the new arm beside
  the old ones, and the type admits an `Each` inside an `Each`, which a
  constructor has to refuse.
- **C. One entry per answer**, keyed by name and player:
  `Vec<(ChoiceKind, Option<PlayerId>, ChosenValue)>`. *Code*: 60–100.
  *Upkeep*: "a step that runs again replaces its own entry" becomes "drop
  every entry of the name", and nothing keeps a step's answers together for
  the reveal that names them.

**Recommended: A.** It is decision 2's "one shape" of the first row again:
the step keeps answers, and one answer is the common case of many. It is
also what step 8 needs: the player each answer belongs to is the stamp.

### Decision 2 — who answers, for whom, and in what order

**Retrofit cost if chosen wrong.** Every card with several choosers writes
its chooser through it, and the dev GUI's prompt tests read the order.
After the build, a sweep over those cards and their tests.

**The walks the cards print** (the census table): each player or each
opponent for themselves in APNAP order (CR 101.4, 608.2e, Divine
Reckoning's ruling); from a named player in turn order (CR 701.38a, and 16
cards that are not votes: Whims of the Fates, join forces, Thieves'
Auction); a named player and then the others in APNAP order (Fortunate
Few's ruling, one card); one player answering for each player (16: 13
"for each player, choose" and 3 "for each player, you choose", Make an
Example among them); and a chosen direction (Order of Succession, one card,
filed below).

- **A. Two parts: which players are walked, and who answers.**
  `ChosenHow` becomes a struct, `{ walk, answered_by }`, as under "The
  shapes". `walk` is `None` for one answer (every first-row step, written
  as today through `From<EffectRecipient>`), or a `ChoiceWalk`: the group
  the cards already use (`PlayerGroup`, "each player", "each opponent",
  "you and target opponent"), its `WalkOrder` (`Apnap`, `TurnOrderFrom`,
  `FirstThenApnap`) and whether it is a vote (decision 5). `answered_by` is
  a player (you; a target opponent; under a walk, one player for each, as
  friend or foe and Make an Example's "for each opponent, you choose"),
  each player for themselves (a walk only, the registry refusing it
  without one), or at random. A vote is a walk, so a later card that
  changes who answers a vote (Illusion of Choice's "you choose how each
  player votes") changes `answered_by` and nothing else. The registry's
  test refuses a walk over a group no player can be in. *Code*: 130–210,
  the first row's `ChosenHow::By` matches moved. *Upkeep*: a new order is an
  arm with its card.
- **B. Arms on the enum, as the note first drew it.** `ForEachPlayer {
  players, order, answered_by }` and `Vote { first }` beside `By` and
  `AtRandom`. *Upkeep*: the fresh read found each case spelled twice:
  `answered_by`'s player beside `By`, its random beside `AtRandom`, and the
  vote's `first: Option` beside the walk's order, its `None` silently
  APNAP.
- **C. The order on the group.** `By(EffectRecipient::EachOf(group))`, as
  the edict's chooser is written today, with a starting player on
  `PlayerGroup`. *Upkeep*: `PlayerGroup` is also what a draw, damage and a
  life loss read, unordered (CR 608.2f's APNAP is theirs); an order on it is
  a field every other reader ignores.

**Recommended: A.** Each order and each way of answering is built in the
PR of its first card, so no arm lands without one (the fresh read's
point): the first PR builds `Apnap`, `FirstThenApnap`, one player and
each for themselves; the second `TurnOrderFrom` (the votes) and one
player answering for each (the friend-or-foe fixture); the third at random
for each (Whims of the Fates). Two of these have one printed card each, so
the owner is asked whether the general shape is right:

- `FirstThenApnap`: Fortunate Few. The other way is two choose steps (you,
  then each other player) whose answers a later reader joins, which needs
  a second name for "objects" and a reader that unions two names.
- At random for each player: Whims of the Fates (`o:"piles at random"`, 1).
  The other way is a choose step per player, which needs the per-player
  step that is step 7's `ForEach`.

The 16 cards where one player answers for each, read by hand: friend or
foe (5); an object of each player's, targeted (Kitesail Larcenist,
Afterlife from the Loam, Guff Rewrites History, Vaevictis Asmadi), which
are step 7's per-player targets; an object of each player's, not targeted
(6): The Eternal Wanderer and Unstable Glyphbridge behind a static or a
room this row does not build, Ghouls' Night Out and Breach the Multiverse
in a graveyard, Tragic Arrogance with four picks a player, and Winnowing
with a reader per player; and Make an Example's piles.

### Decision 3 — what "they" names in "a creature they control"

**Retrofit cost if chosen wrong.** The 32 cards choosing relative to the
chooser, the one-chooser cards that print "they" (Gluntch's "they put two
+1/+1 counters on a creature they control", Choice of Damnations' "that
player sacrifices all but that many permanents of their choice"), and the
untargeted "for each player, choose … that player controls" cards are
written against it.

**The problem.** A filter's "you" is the controller (CR 109.5). In "each
player chooses a creature they control", "they" is the player the walk is
on; in Gluntch, with no walk, it is the one chooser; in "for each player,
choose a creature that player controls" it is the walk's player while you
choose; and in Council's Judgment's "each player votes for a nonland
permanent you don't control" it stays "you". The same filter needs a
second referent, and in every case it is one player: the player the
answer is for, which is what `KeptAnswer::for_player` holds.

- **A. One antecedent, "the player the answer is for", on step 7's
  settled type.** `PlayerRef::Named(PlayerAntecedent::ChoosingFor)`:
  `filter_as_read_now` is handed the player each question is for and
  writes it in as `PlayerRef::Player`. `PlayerAntecedent` is step 7's
  settled type (its note's decision 4, (a)); this row builds it with this
  one arm, and step 7 adds `TheEventsPlayer`, `ControllerOf`, `OwnerOf` and
  `ForEachPlayer`, a body's player, beside it. *Code*: 40–70.
- **B. Step 7's `ForEachPlayer`**, with a one-chooser step read as a walk
  of one. *Upkeep*: Gluntch's chooser is not a "for each", and a body of
  step 7's `ForEach` that holds a choose step would have two players named
  by one word.
- **C. Spell it from what names the chooser**:
  `PlayerRef::NamedBy(ChosenAs(Player))` for Gluntch, a walk word for the
  rest. *Upkeep*: with the edict's `ChoiceScope::ChoosersPermanents`, three
  spellings of "the chooser's permanents".

**Recommended: A.** The edict keeps `ChoosersPermanents` as built; the
three plans its doc comments carry for it ("Cataclysm's four", "another
player's objects", "the rest") are reworded in the first PR to name the
choose step as their home (filed below). Beside the new arm,
`PlayerRef::NamedBy(Box<EffectRecipient>)` already exists, for Mana Leak's
"its controller" and ward's "that player"; step 7's fold of it is filed
there with both uses, and until then the two near names sit side by side.

### Decision 4 — how a later step reads several answers

**Retrofit cost if chosen wrong.** The readers are arms on four enums and
the cards that write them; after the build, a sweep over those cards.

**The readers the cards print** are the second census table: any answer;
the players who gave one answer; a count of answers; the winning answer; a
number picked out; whether every answer is one; and a player's own answer,
inside a later choice or by a step acting on that player.

- **A. Any answer through today's `ChosenAs`, and one selection type for
  the rest.** `ChosenAs` on the three enums reads every answer, as the
  shapes' Harsh Mercy shows (an amount excepted, decision 1). Everything
  else names a selection, `AnswerSelection { kind, which: AnswerIs }` (a
  printed word, the highest, the most chosen), and the place that reads it
  says what it wants of it: the players who gave it, how many there are,
  the number it is, the objects, or a condition. The players go on
  `PlayerGroup`, as a field `answered` that narrows the group to the
  players whose answer is the one selected, and not on `PlayerSet`: the
  fresh read found `PlayerSet::contains` answers from two ids with no board
  and no error path, and the replacement pipeline reads it, so an arm only a
  resolution can fill would be a silent `false` anywhere else. A group is
  read only by a resolution (`players_in`, `engine/resolve.rs:3467`), so the
  edict's chooser and the life loss's recipient take the narrowed group
  unchanged. A player's own answer is read inside a later choose step over
  the same group (Make an Example's pick of "one of their piles", a second
  step with its own walk, the third PR's) and nowhere else; read by a step
  acting on that player it is a `ForEach` body, filed with step 7. *Code*:
  90–150. *Upkeep*: a new selection is an arm of `AnswerIs`, read by every
  place at once.
- **B. An arm per printed phrase.** `Friends`, `Foes`, `VotesFor(word)`,
  `GetsMoreVotes(word)`, `HighestNumber`, each where it is read. *Code*:
  about the same. *Upkeep*: about a dozen arms over four enums, each with
  its own fill and its own registry check, and "the highest number" as a
  group and as an amount are two arms that must agree.

**Recommended: A.** The `Condition::WinsTheVote` arm is the one vote word
kept apart, since "gets more votes" is a comparison between the options
and not a selection of answers: one option alone with the most votes (the
ruling's tie), never a count.

Two of A's arms have one printed card each, so the owner is asked whether
the general shape is right:

- `Condition::EveryAnswer`: Prisoner's Dilemma's "if each opponent chose
  silence" (`o:/if each (opponent|player) (chose|voted)/`, 1).
- `Condition::IsAmong`: Menacing Ogre's "if you are one of those players"
  (`o:"one of those players"`, 2; the other, Power Play, is about who
  starts the game). `Condition::Player { players, fact }`
  (`types/effects.rs:870`) already asks a fact over a player set; it takes
  a `PlayerSet`, which decision 4 keeps the selection off, so the arm is
  new.

### Decision 5 — the vote (CR 701.38)

**Retrofit cost if chosen wrong.** The 33 vote cards and the cards that
change voting are written against it. Low now: nothing votes.

**What the rules and cards ask.** A vote is asked of every player, from a
named player in turn order (a); its options are words, objects, players or
a color (b); it is distinguishable from a choice, since "voting" means
only a vote (c) and Brago's Representative, Grudge Keeper and Illusion of
Choice read only a vote; and a voter with several votes casts them at once
(d), repeats allowed, the later voters seeing all of them.

**Settled already**: every vote is asked as one kind, `Vote`, and a
voter's several votes are "one pick of that many, repeats allowed"
(`choices-architecture.md`'s decision 7, "Decided in review: A" and "What
both settle"). The note's first draft asked a single vote as the kind of
what it is for, which the fresh read caught: Council's Judgment's vote
would have been a plain `SelectRecipients` pick, and the same vote a `Vote`
once its voter had two. It is `Vote` for every vote now.

- **A. A choose step whose walk is a vote.** `ChoiceWalk { is_a_vote:
  true, .. }` over every player in the game, from the named player in turn
  order (`TurnOrderFrom`), or APNAP where no player is named (the six secret
  councils), each for themselves. What is voted for is the step's
  `ValueChoice`, unchanged: a word, objects, a player, a color. Every vote
  asks `DecisionKind::Vote { label, vote_count }`, its options typed by what
  is voted for (`DecisionOption::Word`, `Object`, `Player`, `Color`). One
  vote is a pick of one; several are the provider's existing `allocate`
  over the options (the votes are the total, each option a bucket), which
  is "repeats allowed, all at once" (CR 701.38d). The answers are kept as
  any choose step's, one per vote. *Code*: 170–270 with the winner and the
  most chosen, under (a) below. *Upkeep*: a card that changes voting reads
  one field.
- **B. A primitive of its own**, `Primitive::Vote { options, first }`, its
  own kept value and readers. *Upkeep*: the walk, the keep, the reveal and
  every reader written twice.

**Recommended: A.**

**Where a voter's count of votes comes from** is the one open part. Brago's
Representative's "while voting, you get an additional vote" is a value a
permanent gives its controller, which is `player-scoped-values`' effective
value query (step 10, `backlog.md`'s "2.15"): the land count, maximum hand
size and player hexproof are its first readers.

- **(a) The count waits for step 10.** Every voter has one vote here; the
  question carries `vote_count` from the start, so no client changes when
  it grows. Step 10 answers the count from its query, builds the several
  votes' `allocate` ask with it, and registers Brago's Representative,
  Ballot Broker and The Valeyard, CR 701.38d's boards with them. A
  Deferred Migrations line records it. *Code*: as decision 10's table.
- **(b) Build the count now**, Brago's Representative's static read off the
  effective ability list by the walk alone: a reader of a player-scoped
  value written ahead of the query that owns them, which step 10 then
  moves. About 70–110 more in the second PR, with the `allocate` ask and
  Brago's Representative registered.

**Recommended: (a).** "Whenever players finish voting" (three cards) is a
trigger event, and Illusion of Choice's "you choose how each player votes"
is a change to `answered_by`; both are filed below.

### Decision 6 — a word with no rules meaning

**Retrofit cost if chosen wrong.** Every word a card prints is written
against it; a later change is a sweep over those cards. No log moves: a
word is offered and logged by its label.

**The words the cards print.** A vote's options (26 word votes, CR
701.38b's "words with no rules meaning that are each connected to a
different effect"); friend or foe (5); each opponent's "money, friends, or
secrets", "fame or fortune", "silence or snitch" (3). "Odd or even" (5) and
"left or right" (5) are printed the same way and have rules meaning: a
parity of a number, and a direction around the table (CR 101.4's "turn
order"). They are not this decision's (see decision 10).

- **A. A closed enum, `ChoiceWord`**, one variant per printed word, added
  with the card that prints it, as `CreatureType` holds CR 205.3m's list.
  `ValueChoice::Word(&'static [ChoiceWord])` offers the card's few, and
  `DecisionOption::Word` names each. The compiler checks a reader's word,
  and a word's uses can be found. *Code*: 110–170 with `ChooseWord`
  through every client.
- **B. The card's own text**, `&'static str`, checked by the registry's
  test against the step's list. *Upkeep*: a misspelled reader fails a test,
  not the compiler.

**Recommended: A**, the first row's fourth round over again (a name is an
enum, not text).

### Decision 7 — the secret mark, the reveal, and what a choose question carries

**Retrofit cost if chosen wrong.** The label replaces `source` on the five
choose kinds the first row built: 44 pattern sites in five test files, 37
in `mtgsim/src` and 4 in `devgui/src`, mechanical either way. The mark and
the reveal are new.

**Settled already**: the mark goes on the choose step, a vote included,
and the reveal is a step of its own that writes a record every player sees
(the information model's decision 7, taken A on 2026-11-17). Hiding waits
for step 8. What is open is the shape, and where item 437's words go.

- **A. A label on every choose question, the mark on the step, the reveal
  a step that writes an event.** `ChooseValue` gains `secret`; the question
  it asks carries `ChoiceLabel { source, words, secret }` in place of
  `source`: the step's own words, and the mark. The words are written on
  the step by the card (`.worded("Choose a second player to draw a
  card")`), and the registry's test checks that every choose step has
  words and that they are found in its ability's rules text. A cut by
  line, as a mode's words are (`ModalNode::take_words_from`,
  `types/effects.rs:1298`), finds nothing for Gluntch, whose three choices
  share one line and one of which ("they put two +1/+1 counters on a
  creature they control") never says "choose", which is item 437's own
  case. `Primitive::RevealChoices(kind)` writes `GameEvent::ChoicesRevealed`
  with the answers and their players. The registry's test refuses a read
  of a secret answer that no reveal step comes before (Menacing Ogre's
  "each player with the highest number" after its reveal; The Toymaker's
  Trap's guess, which reads nothing, before it). A choose step over objects
  asks `SelectRecipients`, whose label is item 434's, slotted to
  `card-and-object-model`; until it lands the registry refuses a secret
  choose step over objects (Call to the Void, Malik, none registered here),
  and that item can take this type. *Code*: 120–190, the mechanical sites
  counted apart.
- **B. The mark on the step only; the words a field on each kind.** The
  questions are unchanged until step 8 threads the mark through. *Upkeep*:
  step 8 then reshapes every choose question the way A does now, and a
  client meanwhile cannot tell a secret question from an open one.

**Recommended: A.** One place a client reads which step asks, what it
says and whether its answer is kept from the other seats.

### Decision 8 — an open "choose a number"

**Retrofit cost if chosen wrong.** Low: the offer's type and two clients.

**The problem.** Ten cards print "choose a number" with no top, and CR
107.1b bounds it below at zero only. Today the offer is
`XOffer::Interval { at_least, at_most }` (`oracle/x_values.rs:87`); the
random agent draws uniformly over what it is given, and draws 0 through 20
only when handed exactly `0..u64::MAX` (`autopilot/random.rs:362`); the dev
GUI's number field has a top (`NumberField`, `devgui/src/view_model.rs:1285`).
The clients are a person at the four-seat GUI and an AI client; the random
agent serves the fuzz runs.

- **A. The offer says it has no top.** `ValueChoice::Number`'s `most`
  becomes an `Option`, and the offer `XOffer::NoTop { at_least }` (named
  apart from the `at_least()` method and `Interval`'s field beside it),
  asked as `pick_number(min, u64::MAX)`. The random agent's policy, which lives in
  `autopilot/random` and not in the engine, draws from `min` through
  `min + 20` for any open offer; the dev GUI's field takes any number typed
  and its "+" never stops. *Code*: 40–70.
- **B. The engine bounds it from the board** (Void's highest mana value on
  the battlefield, say). Squee's Revenge and Choice of Damnations have no
  board bound, and a policy per card in the engine is what decision 2 of
  the first row kept out.

**Recommended: A.** A number past 20 is for a client that knows why it
wants one.

### Decision 9 — piles (CR 700.3)

**Retrofit cost if chosen wrong.** The pile cards and the two questions
(`SeparateIntoPiles`, `ChoosePile`). Nothing separates today.

**Settled already**: a pile is asked as a pick of the objects for the
first pile, the rest the second (`choices-architecture.md`'s decision 7,
"What both settle"). Whims of the Fates' three piles are two such picks.
What is open is what is kept and how "that pile" and "the other" read.

- **A. Two steps: separate, then choose a pile.** `ValueChoice::Piles { of,
  count }` keeps `ChosenValue::Piles`, the objects of each pile, each pile
  allowed to be empty (CR 700.3d) and every object in exactly one (CR
  700.3a), kept as `ChoiceKind::Separation`; `ValueChoice::OneOfThePiles`
  asks among them, each pile offered by its objects (never by its place),
  and keeps the pile chosen as `ChoiceKind::ChosenPile`, the two names told
  apart by more than a letter. "That pile" is `ChosenAs(ChosenPile)`, "the
  other" is the separation's objects not in it. Under a walk the piles are
  each player's own (decision 2), so Make an Example's "for each opponent,
  you choose one of their piles", a second step walking the same group,
  offers that opponent's. Separating moves nothing (CR 700.3c). *Code*:
  180–280, and 40–70 more for its two questions through every client.
- **B. One step that separates and picks**, keeping `{ chosen, rest }`.
  *Upkeep*: Hostile Negotiations turns a pile face up between the two, and
  Make an Example's separators and chooser differ by player, so the one
  step needs every place a card can put something between them.

**Recommended: A.** A face-down pile (9 cards) is a pile whose objects one
player has not seen, which is step 8's record; all nine also read a
library or exile, filed with Fact or Fiction below.

### Decision 10 — the split

**Retrofit cost if chosen wrong.** None in code; a PR past the band costs
the review.

Re-counted piece by piece. A count set at a design stop is read with its
top as the expected size (`references/sizing-calibration.md#set-at-a-design-stop`),
and the first row came to 2,876 against 1,290–2,130.

| Piece | Code | Tests | PR |
|---|---:|---:|---|
| Decision 1: the keep, the first row's readers moved to read every answer, an amount reading one | 80–130 | 30–60 | 1 |
| Decision 2: `ChosenHow`'s two parts, `Apnap` and `FirstThenApnap`, one player and each for themselves, each answer kept as given | 120–190 | 90–140 | 1 |
| Decision 3: the player an answer is for, in a filter | 40–70 | 30–50 | 1 |
| Decision 8: the open number, the agent, the GUI field | 40–70 | 30–50 | 1 |
| Decision 11: a chooser who has left the game (CR 800.4g), failing first | 30–60 | 30–50 | 1 |
| Item 435: each step's list of what it reads, which replaces the registry's scan of a step's printed form; its board failing first | 100–160 | 50–80 | 1 |
| Gluntch's three different players, Fortunate Few's exclusion | 30–50 | 30–50 | 1 |
| Harsh Mercy, Divine Reckoning, Fortunate Few, Gluntch, Choice of Damnations registered | 150–250 | 200–300 | 1 |
| The dev GUI's tests | 20–40 | 10–30 | 1 |
| **The first PR** | **610–1,020** | **500–810** | **1,110–1,830** |
| Decision 7: the mark, the reveal, the label with item 437's words | 120–190 | 60–100 | 2 |
| Decision 6: the word, `ChooseWord` through every client | 110–170 | 60–100 | 2 |
| Decision 4: `AnswerSelection`, the group, the count, the number, the conditions | 90–150 | 70–120 | 2 |
| Decision 5: the vote, one each, the winner and the most chosen | 170–270 | 130–210 | 2 |
| Decision 2's `TurnOrderFrom` and one player answering for each | 20–40 | 20–40 | 2 |
| Menacing Ogre, Prisoner's Dilemma, Plea for Power, Council's Judgment, Lieutenants of the Guard registered; the friend-or-foe fixture | 170–280 | 230–340 | 2 |
| `Vote` through every client; the dev GUI | 60–100 | 30–60 | 2 |
| **The second PR** | **740–1,200** | **600–970** | **1,340–2,170** |
| Decision 9: the piles, both questions through every client | 220–350 | 140–240 | 3 |
| Decision 2's at random for each player | 20–30 | 20–30 | 3 |
| Do or Die, Make an Example, Whims of the Fates registered | 90–150 | 120–180 | 3 |
| **The third PR** | **330–530** | **280–450** | **610–980** |

**3,060–4,980 for the row**, so about 5,000 read by its top, against the
brief's 1,160–1,880 (the Sizing row's 1,000–1,600 with items 435 and 437),
with the secret mark not counted there. The guess counted the 16 "each
player chooses" values; the census finds 59 such cards and 41 walks from a
named player, and the readers' table is new.

Where the line between the first two PRs falls is set by their consumers.
The word and every selection of answers (decisions 4 and 6) have no
printed card the first PR can write: each one that prints them either
chooses secretly (Menacing Ogre, Prisoner's Dilemma), votes, or acts per
player through step 7's `ForEach` (every friend-or-foe card, Master of
Ceremonies). So they ride with the mark and the vote.

- **A. Three PRs**, as the table: *the walk and its answers*
  (`several-choosers-at-one-step`, the name kept), then *secret choices,
  words and votes* (`secret-choices-and-votes`), then *piles*
  (`separate-into-piles`). The third is under the band (1,500–2,500,
  `engineering-practices.md#sizing-a-phase`); it is whole, and nothing
  else of the subsystem is ready to ride with it.
- **B. Two PRs, the piles with the walk**: 1,720–2,810 and 1,340–2,170.
  Each pile card needs only the walk, so the first can carry them; it is
  310 past the band's top as read.
- **C. Fold the parity predicate in.** "Odd or even" (5 cards) is a
  comparison of a number read off an object; four of the five read mana
  value, which `ObjectValue` gains at step 7. Filed there under A, B and D;
  folded into the second PR here it adds about 120, with Zimone's
  Hypothesis, which reads power.
- **D. The first two now, the piles off the route** (the fresh read's
  question). v0.5's five decks hold one card in this row's families,
  Selvala's Stampede, a vote that also waits for `information-model`'s
  reveal; no pile card is in them. The piles become a backlog entry with
  this note's decision 9 as its design, built with the wider card work
  after v0.5. The choices subsystem then closes with
  `values-kept-past-the-resolution` short of CR 700.3, which the plans
  record.

**Recommended: A, with parity filed at step 7.** The piles are small and
whole, and CR 700.3 is the subsystem's (the CR is the customer, not a deck
list); D is the owner's call if the route is to reach v0.5 sooner. The
route's Sizing table and `choices-architecture.md#5-the-build-sized` gain
the new rows at the build's first commit.

### Decision 11 — a chooser who has left the game (CR 800.4g)

**Retrofit cost if chosen wrong.** One function, `chooser_named_by`
(`engine/resolve.rs:780`), and the boards that pin it; at four seats a
player leaving is ordinary, so a wrong answer reaches games.

**The rules.** CR 800.4g: "If an object requires a player who has left the
game to make a choice other than whether to pay a cost, the controller of
the object chooses another player to make that choice. If the original
choice was to be made by an opponent of the controller of the object, that
player chooses another opponent if possible." CR 800.4f does the same for a
cost: it isn't paid. CR 608.2b: a target that is no longer legal "won't be
affected by parts of a resolving spell's effect for which they're illegal",
and "any part of the effect that requires that information won't happen".
And CR 104.3a: a player may concede at any time, so a walk can lose a
player partway.

**Today** `chooser_named_by` answers `None` both for a target that went
illegal and for a named player gone from the game (`:775`–`:779`), and the
step chooses nothing. The first row read 608.2b for both; it did not read
800.4g, which is the rule that watches it (found by the fresh read).

| The chooser who has left | Example | Today | The rules |
|---|---|---|---|
| a target player | "target opponent chooses a number", with a second target keeping the spell resolving | nothing chosen | 608.2b: that part won't happen; 800.4g read alone would hand it on |
| a player named otherwise | Gluntch's chosen player; a trigger's "that player chooses" whose player left before it resolved | nothing chosen | 800.4g: the controller picks another player (another opponent where the first was one), who chooses |
| a player of a walk | "each player chooses", one conceding partway | the walk reads the players in the game | they are no longer "each player"; the walk reads its group as it reaches each player, and an answer given before they left stays kept but is read only for players in the game |

- **A. 608.2b for a target, 800.4g for every other named chooser.** A
  target who left is an illegal target, and 608.2b runs first: the part
  that needs the target's choice won't happen, so the object no longer
  requires that player to choose and 800.4g does not reach it. Item 435's
  board stands as the item states it. A named chooser who is not a target
  and has left: the controller is asked `ChoosePlayer` among the other
  players (the other opponents, if any, when the first was an opponent),
  and that player answers. *Code*: 30–60, the first row's answer for a
  player gone changed and shown failing first.
- **B. 800.4g for every chooser who has left, targets included.** The
  illegal target's choice passes to another player. Item 435's board then
  shows a substitute choosing.
- **C. Today's: nobody chooses**, in every case.

**Recommended: A**, as a reading the owner rules on: neither rule's text
settles a target who has left, and no ruling was found. The fails-first
board for the second row of the table is a fixture whose trigger names
"that player" as its chooser.

## The binding table

| State | Bound | Live or latched | Stored | Read by | Ends |
|---|---|---|---|---|---|
| a step's answers, each with its player | the kept choice opened empty as the step starts; each answer as it is given, in the walk's order (CR 608.2d, 101.4) | latched (CR 608.2h); objects by existence (`ObjectRef`, CR 400.7) | `ResolvingObject::choices`, one `KeptChoice` per name | later steps, by any answer or a selection; the same step's later questions (CR 101.4b) | with the resolving record (CR 608.2n) |
| the walk's group | as the walk reaches each player: the group read against the players still in the game then | live | not stored: `players_in` asked each time | the walk | at the step's end |
| the player an answer is for | as the walk reaches each player, or as a one-chooser step asks | live for that question only | the walk, a local of `choose_value` | the step's own filter ("they control") and options ("one of their piles") | at the next question |
| a voter's number of votes | as the walk reaches the voter | read then, never kept (CR 701.38d) | not stored: one, until step 10's effective-value query answers it (decision 5) | the vote's question | at the next voter |
| the secret mark | written on the card | fixed | the step, and each question it asks | step 8's stamp; the registry's test | — |
| a reveal | as the reveal step runs | latched: a record | the event stream, `ChoicesRevealed` | every seat (step 8), the log | never: history |
| the piles | as the separator answers | latched | `choices`, as `Separation` | the pick of a pile; "that pile" and "the other" | with the resolving record |

## What builds without a decision

- **Item 435: each step's list of the names it reads.** One function per
  node, `choices_read`, each match exhaustive, built as
  `engine/triggers/binding_reads.rs` builds a trigger's facts, so a new
  reader arm does not compile until it says what it reads. Before an atom
  acts, its list is asked: a name read and not kept means that part of the
  effect won't happen (CR 608.2b), so the atom does nothing. The amount
  reader and the player word then never meet a missing name except on a
  card the registry refuses. The two-target board the item names: a spell
  whose second target keeps it resolving after the target player who was to
  choose left the game, written as a fixture (decision 11's A). **It
  replaces the registry's scan** (`chosen_name_problems`,
  `cards/registry.rs:1916`), which finds a read by matching `ChosenAs(` in a
  step's printed form and so could not see the second PR's readers; it
  lands in the first PR, before they arrive.
- **Gluntch's three different players.** `ValueChoice::Player` gains the
  names whose answers it leaves out ("a second player", the ruling's "must
  be different players"), and `ChoiceKind` gains `SecondPlayer` and
  `ThirdPlayer` with the card, as the first row's decision 1 said a card
  keeping two of a kind would. One printed card wants the exclusion
  (`o:"choose a second player"`, 1; the census row's third, Dack Fayden,
  Helping Hand, is found by `o:/choose (a|an) (second|third|different)
  (player|opponent)/` and not read here), so the owner is asked whether a
  list of names to leave out is the right general shape, beside the other
  way: a `PlayerSet` arm, which decision 4's reason keeps off that type.
- **Fortunate Few's "that hasn't been chosen this way"** is
  `non(one_of_the_chosen())` in the step's own filter, read each time the
  walk reaches a player: each answer is kept as it is given.
- **Divine Reckoning's flashback** is `AlternativeCost::Flashback`, built.
- **Choice of Damnations' "all but that many"**: the target opponent
  chooses that many permanents they control and sacrifices the rest, its
  ruling's reading, with the open number (decision 8) and "if you don't".

## The cards

Each in its first printing's set file, registered with its rulings in
`rulings-ledger.json`; none is a fixture. Read against the tree for every
piece each needs.

| PR | Card | First printing | What it proves |
|---|---|---|---|
| 1 | Harsh Mercy | Onslaught, `2002/10_07_ons.rs` | a value from each player, read as any answer |
| 1 | Divine Reckoning | Innistrad, `2011/09_30_isd.rs` | objects relative to the chooser, "the rest" |
| 1 | Fortunate Few | Commander 2017, `2017/08_25_c17.rs` | you first, then APNAP; an exclusion read from the walk |
| 1 | Gluntch, the Bestower | Battle for Baldur's Gate, a new `2022/06_10_clb.rs` | three different players chosen one at a time |
| 1 | Choice of Damnations | Saviors of Kamigawa, a new `2005/06_03_sok.rs` | an open number, chosen by a target opponent |
| 2 | Prisoner's Dilemma | Murders at Karlov Manor Commander, a new `2024/02_09_mkc.rs` | secret words from each opponent, the players who gave an answer, every answer one |
| 2 | Menacing Ogre | Onslaught | secret open numbers, the reveal, the highest |
| 2 | Plea for Power | Conspiracy, a new `2014/06_06_cns.rs` | a word vote from you, the winner, a tie |
| 2 | Council's Judgment | Conspiracy | an object vote, the most chosen |
| 2 | Lieutenants of the Guard | Conspiracy: Take the Crown, a new `2016/08_26_cn2.rs` | counted votes on an entering trigger |
| 3 | Do or Die | Invasion, `2000/10_02_inv.rs` | two piles, the other player's pick |
| 3 | Make an Example | New Capenna Commander, a new `2022/04_29_ncc.rs` (its promo printing, PNCC, is the same day) | each opponent's piles, you choosing for each |
| 3 | Whims of the Fates | Born of the Gods, a new `2014/02_07_bng.rs` | three piles, a pile each at random, from you in turn order |

The one fixture, in the second PR's `cards::fixtures::choices`, is
invented: "For each player, choose friend or foe. Each friend draws a
card. Each foe loses 2 life", the one-chooser-for-each walk on the verbs a
group already takes (a draw, a life loss). Every printed friend-or-foe
card gives a friend an action read per player, which is step 7's
`ForEach`. Brago's Representative registers at step 10 under decision 5's
(a), or in the second PR under its (b).

**What a new question moves.** No registered card asks a new question
or loses one, so the 50-game fixture rows move only with a card pooled
(`--fixtures`). The label (decision 7) changes 44 pattern sites in tests,
mechanically. One card per new engine path joins `PERFORMANCE_POOL`:
Divine Reckoning for the walk in the first PR, Council's Judgment for the
vote in the second, Make an Example for piles in the third, each measured
for its stress distortion before it is pooled.

## Gates and predictions

The gates that can fail: `cargo build --all-targets` with zero warnings;
`cargo clippy --all-targets -- -D warnings` on both crates; `cargo test`;
the dev GUI's build and tests, since `DecisionKind` gains arms and the
label; every `check_*.py` by exit code, `check_rulings.py` for each card;
`python plans/fails_before_fix.py` for item 435's board.

**The A/B, three arms per PR** (`fuzz_ab --require`): `main`; the branch
with its cards unregistered; the branch. The middle arm asks no new
question and moves no answer, so it is predicted `IDENTICAL` to `main`:
the keep's reshape and the walk change nothing for one chooser, item 435
changes only a step whose name was never kept, which no registered card
reaches, and the label adds no prompt. Decision 11's A changes a game only
where a non-target chooser has left before its step, which the dumps will
name game by game. The branch moves with its cards.

## Filed at the close-out, if the decisions stand

- Step 7, `resolution-recipients-and-amounts`: a player's own answer read
  by a step acting on that player (Grave Sifter, Selective Obliteration,
  Order of Succession's "the creature they chose"); each answer with its
  player, one at a time (Expropriate, Elrond, Master of Ceremonies, Seize
  the Spotlight, Mob Verdict, Círdan); a friend's or foe's own action
  (Pir's Whim, Regna's Sanction, Virtus's Maneuver, Khorvath's Fury,
  Zndrsplt's Judgment); "odd or even" with mana value (Extinction Event,
  Mutinous Massacre, Thanos, Zimone's Hypothesis); Wheel of Misfortune's
  "each player who didn't choose the lowest number", a group's complement
  and `AnswerIs`'s lowest; `PlayerRef::NamedBy`'s fold, for both its uses
  (Mana Leak's "its controller", which is `ControllerOf`, and ward's "that
  player", the triggering player); and its decision 5's note that friend or
  foe's word and walk landed here.
- `player-scoped-values` (step 10): a voter's count of votes from its
  effective-value query, the several votes' `allocate` ask, and Brago's
  Representative, Ballot Broker and The Valeyard (decision 5's (a)).
- `information-model`: a pile one player has not seen (the face-down
  piles); Fact or Fiction and the 18 other pile cards whose objects sit in
  a library or exile, which wait for "the top N cards" as a set beside
  Selvala's Stampede's reveal; a secret choose step over objects, with
  item 434's label (Call to the Void, Malik).
- **Several picks by one answerer** ("an artifact, a creature, an
  enchantment, and a land"; "a creature they control and a creature they
  don't control", CR 101.4c): a choose step's arm with its first card
  (Cataclysm, Balance's lands, Tragic Arrogance, Call to the Void), filed
  as an item. The first PR rewords the three plans on the edict's `Choice`
  (`types/effects.rs:1504`, `:1511`, `:1519`: "Cataclysm's four", "another
  player's objects", "the rest") to name the choose step as their home.
- The backlog's voting entry (`backlog.md#24-voting-and-the-decisionprovider-choice-shapes`)
  rewritten to what is left: "whenever players finish voting" as a trigger
  event (Grudge Keeper, Erestor of the Council, Model of Unity), and
  Illusion of Choice's "you choose how each player votes", a change to a
  vote's `answered_by`.
- `combat-restrictions`: Fight or Flight, Stand or Fall and Raging River's
  piles, read by an attack or block restriction; Camouflage's blocking.
- The third row, `values-kept-past-the-resolution`: Council Guardian's
  color vote read by protection; Order of Succession and Aminatou's "left or
  right", a direction around the table; The Toymaker's Trap's "a number
  that hasn't been chosen", across resolutions.
- Item 429 closed by the second PR, which brings its word; item 435 by
  the first; item 437 by the second.
