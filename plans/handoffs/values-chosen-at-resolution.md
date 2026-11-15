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

- **`several-choosers-at-one-step`** (item 429): start from decision 5's
  second row below, the vote's and the piles' counts under "Counted", and
  the choose step as built. A step has one chooser today
  (`ChosenHow::By` a recipient naming one player); several choosers is that
  step asked of each in turn (CR 608.2e), the answers kept apart.
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
| The kinds: `ChooseColor`, `ChooseCreatureType`, `ChooseCardType`, `ChooseNumber`, and `ChooseOpponent` widened to `ChoosePlayer`; `DecisionOption::CreatureType` and `CardType`; each with its subject, `as_str`, question, option label, why, setup line, decision-log text and replay; `ui::ask`; the random agent | 140–220 | 50–90 |
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

Each is asked through `ui::ask`, names its subject (`source`, the resolving
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
