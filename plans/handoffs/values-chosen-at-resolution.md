# Handoff — `values-chosen-at-resolution`, the design note (first of two)

**Opened 2026-11-14** on `choices/value-chosen-as-an-effect-resolves` off
`7f0e97ef` (#328's merge), as `roadmap-v2.md` §3a step 5, row
`values-chosen-at-resolution`: the last design in `choices-architecture.md`.
This PR is its first half. This file is the design note, reviewed before
the build; the row's last PR deletes it. Every count and line below was
read against `7f0e97ef`. **Revised after the design stop's fresh read**
(the `owner-review` agent, the same day): decision 2 and the split's homes
are new, and decision 5 now builds on step 7's settled player words.

Decision 7's kinds were settled at #307's review (`choices-architecture.md#decision-7--a-value-chosen-as-an-effect-applies-cr-6082d`:
option A, a `ChoiceKind` per value). This note decides what is still open:
what names an earlier choice and where its picks are kept (item 426),
whether an object gets a choosing instruction of its own, how a value
reaches the instructions that read it, how a player verb finds a chosen
player, and the split.

## Where the work sits

All of it is inside one stack object's resolution, the RESOLVE step of
`engine-map.md#11-the-path-in-one-sketch`: `resolve_top_of_stack` →
`resolve_effect` → each instruction in turn. Today an instruction can choose
objects as it applies (decision 8, landed in #326: `EffectRecipient::Choose`,
asked through `SelectRecipients`), and nothing after it can name what it
chose. This PR adds three things on that step:

- **An instruction that only chooses**: a color, a creature type, a card
  type, a number, a player or objects, asked through `ui::ask` under its own
  `ChoiceKind` (decision 7's), or drawn at random.
- **A record of what each choice picked**, kept for the rest of the
  resolution (decision 1).
- **The words that read it back**: "it" and "that player" as an
  instruction's recipient, "the chosen [value]" and "the rest" in a filter
  or an amount (decision 3), and a player verb reading the player it was
  handed (decision 4).

Two places off that step change too. The filter table
(`engine/object_filter.rs`) gains two leaves, which only a resolution's own
reads answer. And every client reads the new kinds: the dev GUI's words, the
decision log, setup lines, the why panel and the random agent.

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
  re-counted at 110–180 with the store's skips and its checks.

Two more come in:

- **Item 394's power leaf.** Item 394 ("a filter compares only power, and
  only to a fixed number") is slotted to step 9's cheap gaps. Only the leaf
  Expel the Interlopers needs comes forward, power compared to an
  `AmountExpr` (~40–60, decision 3); mana value, toughness, mentor's "lesser
  power" and "tapped" stay where they are slotted, and the item narrows.
- **Item 292, proposed (decision 4).** "A one-player instruction whose
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
several choosers at one instruction, a vote's walk without its count.

**What reads the value afterwards.** A card can sit in several rows;
"lasting" is a duration or a later event in the same clause.

| Reader | Cards | Lasting | Example |
|---|---:|---:|---|
| a filter over cards in a zone (a hand, a library, a graveyard, what was revealed) | 62 | — | Cabal Therapy's "all cards with that name" |
| a filter over permanents or spells, read as the instruction applies | 67 | 39 | Expel's "power greater than or equal to the chosen number"; Kindred Dominance |
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
  Conjurer's Ban) or linked (step 6). None is writable with this PR's words.

And the word: 12 of the 17 sort each player by a word (friend or foe) or ask
each opponent, which is the several-choosers walk; "odd or even" (5) reads
mana value or power by parity, a leaf of its own.

### The objects, by what the next clause does with them

Of the 75 cards in `Choose`'s family (#326's census), 68 name the chosen
object again (item 426's word count). Read by hand for its role
(`choice-kinds-census.py --names` on the family, 2026-11-14):

| The chosen object, in the clause after the choice | About | Example |
|---|---:|---|
| what the next instruction acts on: "it gains", "put … on it", "untap it" | 13 | Final Showdown, The Elderspell, The Nipton Lottery |
| what the next instruction spares: "destroy the rest", "each creature not chosen" | 11 | Duneblast, Last One Standing, Celestial Judgment |
| what is copied: "a copy of it", "becomes a copy of that creature" | 8 | Cytoshape, Season of Weaving, Polymorphous Rush |
| named by a lasting effect or a later trigger: "is dealt to that creature instead", "whenever that creature deals combat damage this turn" | 4 | Gideon's Sacrifice, Azra Oddsmaker |
| what deals damage, or whose power is read | 2 | Crush Underfoot, Benthic Anomaly |
| the rest: a card in a hand, each player choosing, a choice with a payment | ~30 | Despise, Tunnel of Love, Mudslide |

So most object choices are a sentence of their own, and the next
instruction acts on something else: the rest of the board, a copy, a
source. The brief's chain ("choose a creature you control; it gets +2/+2
and gains trample") is printed nowhere in that phrasing (Scryfall,
`o:/choose (a|an|another) [a-z ]*creature[^.]*\. (it|that creature) (gets|gains)[^.]* and (gains|gets)/`,
0, 2026-11-14); The Nipton Lottery's run of instructions on its chosen
creature is the printed form.

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
- **CR 608.2c**: instructions are followed in the order written, read as
  English: "it" and "that player" look back.
- **CR 608.2h** and **CR 611.2c**: information is determined once, as the
  effect is applied; a continuous effect from a resolution that changes
  characteristics fixes its set then, and one that changes the rules keeps
  reading. 611.2c is the rule that *watches* this one: it is why a "can't"
  made from a chosen value must carry the value itself.
- **CR 607.2d**: on a permanent, "the chosen [value]" names the choice its
  linked ability made. That is step 6's (`linked-abilities`, 202 cards that
  choose as a permanent enters), the next row; the words this PR adds are
  the ones that row's static abilities read (decision 3).
- **CR 603.7c**: a delayed trigger refers to what the instruction that
  created it named, players included: Faramir, Prince of Ithilien's "that
  player" reaches its delayed trigger through `Effect::Remember`, once a
  choosing instruction notes what it chose.
- **CR 105.4** (one of the five colors), **205.3m** (the creature types,
  Time Lord the one of two words), **205.2a** (the card types), **700.2e**
  (another player choosing a mode, which `ChoosePlayer` also asks).
- **CR 400.7**: an object chosen and then moved is a new object, which "it"
  does not follow unless a rule says so; the picks are kept by existence
  (`ObjectRef`), as step 7's decision 1 keeps targets.

## The shapes the decisions name

```rust
// types/effects.rs
/// Which choice a word names: the place, from 0 in printed order, of the
/// instruction that made it among the effect's choosing instructions.
pub struct ChoiceSite(u8);

enum Primitive {
    // …
    /// CR 608.2d — choose, as the instruction applies; nothing else happens.
    /// Its atom's recipient is `Implicit`.
    ChooseValue { value: ValueChoice, how: ChosenHow },
}

pub enum ValueChoice {
    Color,                                  // CR 105.4's five
    CreatureType,                           // CR 205.3m's list
    CardType(Option<Vec<CardType>>),        // CR 205.2a's, or a printed few (Season of Gathering)
    Number { least: u64, most: u64 },
    Player(PlayerSet),
    Objects(SelectionFilter, TargetCount),  // decision 2
}

pub enum ChosenHow {
    /// The player this recipient names chooses: `Controller`, or a target
    /// player, an instance of "target" as an edict's chooser is.
    By(EffectRecipient),
    /// Drawn from `GameState.rng`; nothing asked (decision 7).
    AtRandom,
}

// The words (decision 3), each naming a site.
EffectRecipient::SameChoiceAs(ChoiceSite)   // "it", "that player"
ObjectFilter::OfChosenValue(ChoiceSite)     // "of the chosen color"; for objects, "is one of the chosen"
ObjectFilter::PowerAtLeast(AmountExpr)      // item 394's leaf
AmountExpr::ChosenNumber(ChoiceSite)        // "the chosen number", "that many"

// state/game_state.rs, on ResolvingObject, beside `x_value`
pub choices_made: Vec<Option<ChoiceMade>>,  // by site; `None` for a site the walk passed

pub enum ChoiceMade {
    Objects { chosen: Vec<ObjectRef>, shares: Vec<u64> },  // `shares` empty unless divided
    Player(PlayerId),
    Color(Color),
    CreatureType(CreatureType),
    CardType(CardType),
    Number(u64),
}
```

Each kind's authoring helper names its word for the call site
(`of_the_chosen_color(site)`, `of_the_chosen_creature_type(site)`), all
building the one leaf, and `cards::registry`'s test checks that each word
names a site of a kind it reads.

## Decisions

### Decision 1 — what names an earlier choice, and where its picks are kept (item 426)

**Retrofit cost if chosen wrong.** Every card that chooses and then names
the choice is written against this, and `linked-abilities` (step 6) reads
the same words. Changing the reference later is a sweep over every such
card.

**The problem both options share.** A choice is made by one instruction
and read by later ones. Something must name *which* choice a later word
reads, and something must keep the picks until then. The text names them
by what was chosen ("it", "that player", "the chosen color"), and one
ability can make two choices of one kind (Tunnel of Love's two creatures,
"the chosen creatures"; Gluntch's three players; Benevolent Offering's two
paragraphs, each "Choose an opponent").

**What a choice site is, in both.** Every instruction that makes a CR
608.2d choice: `ChooseValue`, a `Choose` recipient (decision 2), and an
edict's `ChosenBy`, whose picks are kept the same way, so that none becomes
a site later and moves every number after it. Counted in printed order
across a spell's spell abilities, as `SameInstanceAs` counts instances
(`card_data::spell_effect`: Benevolent Offering's two paragraphs are sites 0
and 1); a mode is its own scope. A "you may" is no site (its answer is CR
118.12's), nor a mode (chosen at cast), nor a payment's amount (the
resolution's X).

- **A. By the site's place, as `SameInstanceAs` names a target.**
  `SameChoiceAs(site)` is "it" and "that player" as an instruction's
  recipient; decision 3's words name a site the same way. The picks are kept
  on the resolving object, `ResolvingObject.choices_made`
  (`state/game_state.rs:181`), as `x_value` keeps CR 107.3f's X chosen as
  the effect resolves: every reader that holds the resolution's context
  reaches it there, `evaluate_amount` and `resolve_player_ref` included,
  with no new argument. The walk's `choice_cursor`, beside
  `instance_cursor` (`engine/resolve.rs:164`), says which site an
  instruction is; a branch not taken (a false "if", a declined "may") moves
  it past the branch's sites and leaves them `None`, so an instruction
  naming one does nothing. A choosing instruction notes what it chose for an
  `Effect::Remember` around it (`note_acted_on`), which carries a chosen
  player to a delayed trigger (CR 603.7c). `ContinuousChange` writes
  `SameChoiceAs(ChoiceSite(0))` after a `Choose`, as it writes
  `SameInstanceAs(0)` after a target, and the refusal goes.
  *Code*: 110–180. *Performance*: nothing for an effect that makes no
  choice; an empty `Vec` clones without allocating. *Upkeep*: a site the
  author counts, checked by `cards::registry`'s tests as `SameInstanceAs` is
  (each names a site before it, in its scope, of a kind its reader takes),
  with a twin of `a_chained_change_refers_to_the_target_its_first_change_declared`
  (`cards/registry.rs:1816`) for a chain on a choice.
- **B. A binding: the choice scopes the instructions after it, and the
  words name it by kind.** `Effect::Choosing { choice, then }`; "the chosen
  number" is the innermost enclosing choice of a number, with no site.
  Reads like the card. *Upkeep*: a new `Effect` combinator every walk takes
  (`instances`, the registry's checks, the resolver), decision 8's `Choose`
  recipient a second shape beside it, and two choices of one kind in one
  scope (Tunnel of Love) needing a site after all.

**Where the picks live, a sub-choice inside A.** On the walk instead, beside
`last_cost_answer`, every reader would be handed the walk: the resolution's
`evaluate_amount` has 28 call sites, and `resolve_player_ref` and
`matches_for_effect` take only the context. A CR 615.5 rider and a mana
ability resolve no stack object, so a choice in one is refused, loudly, as X
is: no printed rider chooses, and the 12 mana abilities that choose a color
are `chosen-color-plus-could-produce`'s (step 14), which gives them their
home.

**Recommendation: A**, the picks on the resolving object. It is
`SameInstanceAs`' shape and the brief's; and a site's place is the clause's
identity, fixed by the card, the same in every process and fork, never a
position in a list the engine builds as it runs.

### Decision 2 — whether an object is chosen by an instruction of its own

**Retrofit cost if chosen wrong.** The 75 object cards are written one way
or the other; Final Showdown is the one registered card that moves.

**The problem.** Decision 8 built a choice of objects as the recipient of
the instruction that acts on them: Final Showdown's "Choose a creature you
control. It gains indestructible" is `gains_keyword(chosen_permanent(…))`.
That holds the 13 cards whose next instruction acts on the chosen object.
The other two thirds of the table above choose in a sentence of their own
and then act on something else: the rest of the board (Duneblast), a copy
(Cytoshape), a source (Crush Underfoot). With nothing acting on the chosen
object, there is no recipient to carry the choice.

- **A. Both, by the text's shape.** A choice printed as its own sentence is
  `ChooseValue(Objects(…))`, and the instructions after it name it
  (`SameChoiceAs`, or `Not(OfChosenValue(site))` for "the rest"). A choice
  inside the instruction that acts on what is chosen ("distribute three
  counters among any number of creatures you control", Feast of the
  Victorious Dead) stays decision 8's `Choose` recipient. Both are sites. A
  value printed inline (Wash Out's "permanents of the color of your choice")
  is a `ChooseValue` just before its reader, since a filter cannot ask: the
  choice is made as that instruction applies either way (CR 608.2c). Final
  Showdown moves to the first form; Feast stays. A copy effect's donor keeps
  its own kind, chosen by the copy as it applies (`ChooseCopySource`,
  Cytoshape as built). *Code*: 20–40, with objects at random free
  (`ChosenHow::AtRandom`: Last One Standing, The Nipton Lottery). *Upkeep*:
  two spellings for two printed shapes; which one a card takes is read off
  its text, which no registry check sees, so it is a review rule.
- **B. One spelling: every object choice is `ChooseValue`.** Feast's
  "distribute" becomes a choice with its shares kept at the site, then a
  put that reads them. `Choose` leaves cards and stays only as
  `SelectRecipients`' label, as the edict's prompt uses it
  (`engine/resolve.rs:2486`). *Code*: 60–110: Feast and #326's division
  fixtures rewritten, `recipients_for`'s `Choose` arm moved into the
  instruction. *Upkeep*: one spelling; a printed "distribute among" becomes
  two instructions where the card has one.
- **C. Objects wait.** Only `Choose` and `SameChoiceAs`; "the rest", copies
  and sources wait for a later row. *Code*: none. *Upkeep*: 19 of the
  family's printed shapes stay unwritable after the subsystem closes.

**Recommendation: A.** It writes each card as it is printed, the way the
value kinds already split into a sentence of their own and inline, and
keeps #326's division as built.

### Decision 3 — how a chosen value reaches what reads it

**Retrofit cost if chosen wrong.** Each reader type gains its words now,
and step 6's linked abilities answer the same words off a permanent; a
shape that serves only the resolution would be redone there.

| Reader | Cards | Under A | Built here |
|---|---:|---|---|
| a filter over permanents, read as the instruction applies | ~57 | `OfChosenValue(site)`: the object is the chosen color, has the chosen creature or card type, or is one of the chosen objects; item 394's `PowerAtLeast(AmountExpr)` with `ChosenNumber` | yes: Expel and the fixtures |
| an edict's pick filter | 1 | the same leaf, read by the edict's candidates | yes: World Queller's model |
| an amount | 10 | `ChosenNumber(site)` | yes |
| the player as a payer, a chooser or who gains control | — | `PlayerRef::NamedBy(SameChoiceAs(site))`, read as the instruction applies, as #324's payer is | yes |
| the player in a filter ("creatures that player controls") or a group ("you and that player") | 4 + | step 7's `PlayerRef::Named(PlayerAntecedent)` and `PlayerGroup.named`, with a chosen arm (decision 4) | no: step 7's second PR |
| a filter over cards in a zone | 62 | the same leaves; the search or reveal is `information-model`'s | no |
| an effect that outlasts the resolution and reads the value later: a "can't", a requirement, a later trigger | ~10 | the effect holds the value it was made with | no: refused; the split's third row |
| a change's own parameter, and the inline ones | 47 + 34 | the change holds the value | no: refused; the split's third row |

**Which reads are a resolution's.** Today `TextReferents` is one asker for
every reader outside the layer walk: a target's legality
(`engine/targeting.rs:713`), an effect's text read through
`object_matches_filter_of_source` (`:727`), a restriction's set at each
query (`engine/restriction/predicate.rs:284`), a replacement's "caused by"
(`engine/replacement/gather.rs:763`) and a resolution's filter
(`engine/resolve.rs:2602`). So the words need a field that says an asker is
the resolution's: **`FilterIdentity.resolution: Option<&ResolutionContext>`**
(`engine/targeting.rs:230`), set only by `filter_identity_for`
(`engine/resolve.rs:2609`), which `matches_for_effect`, `permanents_matching`
and a choice's or an edict's candidates already use. Every other asker
builds its identity with `None`, and the leaves refuse there with a refusal
of their own (`FilterRefusal::NotAResolution`), a malformed question. A
"can't" made by a resolution never holds a word (refused as it is made), so
it never reads a later resolution's choices.

- **A. Each word is answered where it is read, as "you" is.** A resolution
  answers off its resolving object: `evaluate_amount` reads `ChosenNumber`,
  `player_named_by` (`engine/resolve.rs:3059`) reads `SameChoiceAs`, and the
  filter table asks through `FilterReferents` (`engine/object_filter.rs:35`)
  for `chosen_value(site)` and `amount(expr)`, which `TextReferents` answers
  only when its identity is a resolution's. `OfChosenValue` reads the
  characteristics through the table's frame, as every leaf does. An effect
  that outlasts the resolution is written with the values it read as it is
  made, the narrow write-in the refused rows need, built with their first
  cards. *Code*: 130–200 with item 394's leaf. *Upkeep*: a new reader is one
  leaf in the type that reads it; step 6 answers the same leaves off a
  permanent's linked choice in the layer walk's referents.
- **B. The resolution writes the value in before any reader sees it.**
  Each instruction naming a choice is rewritten as it is reached ("of the
  chosen color" to `ByColor(Red)`, the number to `Fixed(7)`), so every
  reader and every effect it makes holds plain values. *Code*: a rewrite
  over every type a word can sit in, and over each of `Primitive`'s ~60
  arms that holds one. *Upkeep*: every new leaf or arm needs its rewrite;
  and step 6's static abilities cannot be rewritten ("the last chosen
  color" changes while the ability exists), so they need A's leaves anyway.

**Recommendation: A**, with the write-in where an effect outlasts the
resolution, and only there.

**Item 394's twin.** `PowerAtLeast(AmountExpr)` beside `PowerLE(i32)`
reads as two shapes for one question. The build renames `PowerLE` to
`PowerAtMost` (24 lines, a counted sweep through `replace_exact.py`), its
`i32` kept until item 394's other leaves bring the amount; say if you would
rather it waited for them.

### Decision 4 — a player verb's player: item 292 with item 425

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
2026-10-26) makes every player word one reader, `Player(PlayerAntecedent)`
on both enums, with "nobody" an answer that does nothing, which is item
292's fix; it keeps an instance's own word, so "that player" after a target
stays `SameInstanceAs`. A chosen player's "that player" as a recipient is
`SameChoiceAs`, the same kind of word. Its other two places, a filter's
"that player controls" and a group's "you and that player", are
`PlayerAntecedent`'s, which is not built yet: they get a chosen arm there.

- **A. Item 292 comes forward, whole.** One reader returns the player the
  route found, for every recipient that names one (a target, a chosen
  player, a trigger's player, a remembered one, defending player), the
  controller only for `Implicit` and `Controller`, and nobody otherwise:
  each of the 13 verbs does nothing for nobody, as `DealDamage` already
  does (CR 608.2b's "won't affect", 603.6's "unable to be found"). Row 39's
  two sites, `named_player` and `EntersAttacking::Player`, answer nobody the
  same way. `cards::registry` gains a check that no player verb has a
  recipient naming only objects. This is step 7's reader in step 7's shape;
  its second PR changes which arms feed it as it folds `TriggeringPlayer`
  into `PlayerAntecedent`, as it would have anyway, and loses item 292's
  60–80. *Code*: 60–100.
- **B. The chosen player's arm only.** `SameChoiceAs` joins the arms that
  read the slice; item 292 stays with step 7, its verdict changed to
  "reachable, wrong today". *Code*: ~10.

**Recommendation: A.** It is the same function in the shape step 7 settled,
and B would leave a known wrong answer reachable through two more route PRs.
Either way the chosen arm of `PlayerAntecedent` is filed into step 7's note.

`player_the_instruction_names` (`engine/resolve.rs:2961`) keeps asking
Akroan Horse's "an opponent gains control" as the instruction applies, under
`ChoosePlayer`: an implicit choice inside a word the instruction reads,
where `ChooseValue` is the printed "choose a player".

### Decision 5 — the split, and where the rest goes

The design's table gives the row 560–870 code and 450–700 tests, plus item
426's 80–150 and item 394's leaf; at the table's 1.7× that is 1,700–2,700
for the row. Re-counted piece by piece for this half. A count set at a
design stop is read with its top as the expected size
(`references/sizing-calibration.md#set-at-a-design-stop`: 20 rows, a mean
of 1.06 times the top, 0.53–1.70), so no ratio goes on these.

| Piece | Code | Tests |
|---|---:|---:|
| The kinds: `ChooseColor`, `ChooseCreatureType`, `ChooseCardType`, `ChooseNumber`, and `ChooseOpponent` widened to `ChoosePlayer`; `ChoiceOption::CreatureType` and `CardType`; each with its subject, `as_str`, question, option label, why, setup line, decision-log text and replay; `ui::ask`; the random agent | 140–220 | 50–90 |
| `ChooseValue`, `ValueChoice` and `ChosenHow`: each kind's options (105.4's five, 205.3m's list with its missing fifteen, 205.2a's or a printed few, a number's range, a player set, an object filter) and at random | 170–260 | 100–170 |
| Decision 1: the sites, the store, the cursor and its skips, `SameChoiceAs`, `Remember`'s note, the chain, the registry's checks | 110–180 | 80–130 |
| Decision 2's A: Final Showdown to its own sentence | 20–40 | 20–40 |
| Decision 3: `OfChosenValue`, `ChosenNumber`, `PowerAtLeast` and the rename, the resolution's identity and its refusal, `NamedBy`'s arm | 130–200 | 70–120 |
| Decision 4: item 292 and item 425, the Lich and its twin, item 292's two boards | 60–100 | 60–100 |
| Expel the Interlopers registered and pooled; the fixtures below | 130–190 | 130–210 |
| The dev GUI's words and tests | 15–40 | 15–30 |
| **This half** | **775–1,230** | **525–890** |

**1,300–2,120.** `CreatureType` goes under `counted_enum!` for its list,
which re-indents its ~300 variants: a whitespace-only change, read with
`git diff -w` and counted apart.

- **A. Three rows.** *This PR*: every kind above, made by you, by a player
  the text names, or at random, with decisions 1 to 4: 1,300–2,120. *The
  second* (the brief's): several choosers at one instruction — votes
  (701.38, 33 cards, the per-player walk's starting seat), piles (700.3,
  21), the 16 "each player chooses" values and their readers ("a type
  chosen this way"), the word kind (12 of its 17 sort or ask each player;
  "odd or even"'s parity leaf), a choice excluding an earlier one's pick
  (Gluntch, Fortunate Few), and an open "choose a number" (Void, Squee's
  Revenge), whose unbounded pick needs a policy for the random agent and the
  GUI; a guess of 1,000–1,600. *A third, new*, `values-kept-past-the-resolution`:
  the write-in for an effect that outlasts the resolution (~10), a change's
  parameter from a choice (47) and the inline ones (34), with the two kinds
  read only that way (a basic land type, a keyword) and the card name (its
  catalog, and whether its names are the process's registry or the game's,
  since a game is built from a `CardRegistry` passed in, `scenario/build.rs`);
  a guess of 1,200–2,000, which may split at its brief. The subsystem closes
  with the third.
- **B. Two rows, as the brief.** Every one-chooser kind here, the card name,
  the word, the keyword and the basic land type each shown by a fixture whose
  reader no printed card has (or by a branch per keyword under a condition,
  a second spelling of a change's parameter for two cards): about
  1,620–2,640, past the band's top. The refused rows still need a home, so
  the second PR grows by the third row's work.

**Recommendation: A.** This PR stays in the band with a printed model for
every board, and every refused row has a row that owns it. The intake writes
the split into `choices-architecture.md`'s "The build, sized" table before
the build, which also moves Selvala's Stampede, a vote, off this row.

## The binding table

| State | Bound | Live or latched | Stored | Read by | Ends |
|---|---|---|---|---|---|
| what a choice site picked | as its instruction applies (CR 608.2d) | latched: determined once (608.2h); objects kept by existence (`ObjectRef`, CR 400.7) and found only while each is the same object | `ResolvingObject.choices_made`, by site | `SameChoiceAs`, `OfChosenValue`, `ChosenNumber`, `NamedBy(SameChoiceAs)` | with the resolving object (CR 608.2n); a mode's on leaving the mode |
| which site an instruction is | as the walk reaches it | live | `ResolutionWalk.choice_cursor` | the store's writer | each walk; saved and restored around a mode, as `instance_cursor` is |
| that a filter is read by a resolution | at each read | live | `FilterIdentity.resolution` | the two leaves, through the referents | with the read |
| a value an effect keeps past the resolution | — | — | not built: refused | — | the third row |

## The kinds, as decision 7 settled them

Each is asked through `ui::ask`, names its subject (`source`, the resolving
object), and is answered unasked when one option is legal. Clients: a
person at the four-seat GUI and an AI client, both reading the typed
options; the dev GUI words each now.

| Kind | Options | Asked by |
|---|---|---|
| `ChooseColor { source }` | `ChoiceOption::Color`, the five (CR 105.4) | `pick_n` |
| `ChooseCreatureType { source }` | `ChoiceOption::CreatureType`, CR 205.3m's list | `pick_n` |
| `ChooseCardType { source }` | `ChoiceOption::CardType`, CR 205.2a's or a printed few | `pick_n` |
| `ChooseNumber { source, values }` | `values` an `XOffer::Interval`, as `ChooseXValue` and `ChooseAmountToPay` spell a range (`oracle/x_values.rs:87`) | `pick_number` |
| `ChoosePlayer { source }`, widened from `ChooseOpponent` | `ChoiceOption::Player`, the players the choice admits: Akroan Horse's opponent, CR 700.2e's chooser of a mode, and "choose a player" | `pick_n` |
| objects | decision 8's `SelectRecipients`, unchanged | `pick_n` |

## What builds without a decision

- **`CreatureType` gains CR 205.3m's missing fifteen** (Bison, Drix,
  Echidna, Hedgehog, Hero, Kangaroo, Lemur, Lobster, Moogle, Platypus, Qu,
  Sorcerer, Symbiote, Utrom, Villain): `tmnt.txt` lists 316 and the enum
  301, and a choice offers the rule's list. `CreatureType` goes under
  `counted_enum!` for `ALL`.
- **Expel the Interlopers** in a new `cards/sets/2023/09_08_woe.rs`, its
  first printing (Wilds of Eldraine), registered and pooled, its ruling in
  `rulings-ledger.json`: `ChooseValue { Number { least: 0, most: 10 } }`,
  then a destroy of `creature().and(PowerAtLeast(ChosenNumber(ChoiceSite(0))))`.
- **The fixtures**, in `cards::fixtures::choices`, each naming the printed
  card it models:
  - a color: Wash Out, "Return all permanents of the color of your choice
    to their owners' hands";
  - a creature type: Kindred Dominance, "Choose a creature type. Destroy all
    creatures that aren't of the chosen type";
  - a card type: World Queller, "you may choose a card type. If you do, each
    player sacrifices a permanent of their choice of that type", the edict's
    pick filter reading the site;
  - a player you choose: Gluntch's "Choose a second player to draw a card"
    and "choose a third player to create two Treasure tokens", a draw and a
    token's controller (its rule that the three differ goes to the second
    row);
  - a player at random: Truth or Consequences' last sentence, "choose an
    opponent at random. [It] deals 3 damage to that player", without its
    vote;
  - objects in a sentence of their own, at random: The Nipton Lottery,
    "Choose a creature at random. You gain control of that creature until
    end of turn. Untap it. It gains haste until end of turn. Then destroy
    all other creatures";
  - objects you choose, and the rest: Duneblast, "Choose up to one
    creature. Destroy the rest";
  - the brief's chain, "choose a creature you control; it gets +2/+2 and
    gains trample until end of turn", which no card prints in that phrasing:
    a fixture of `ContinuousChange`'s shape, choosing once.

## Atoms and their boards

`python plans/specdb.py next --rule` for each rule the brief names, read
2026-11-14.

| Atom | What it asks | Planned test |
|---|---|---|
| `ATOM-608.2d-001` (partly covered) | choices not made while casting are announced as the effect applies, and no illegal option is chosen | Expel's board: the number asked as it resolves, 0 through 10 offered (its ruling); stays `COVERS-PARTIAL`, its own board being a "you may sacrifice" |
| `ATOM-608.2d-002` (partly covered) | an untargeted division gives each at least one | not this PR's: #326's division, unchanged under decision 2's A |
| `ATOM-608.2e-001` | several players' choices in APNAP order, then the action at once | the second row's: several choosers at one instruction |
| `ATOM-105.4-001` | a color choice offers the five, and refuses "colorless" | the Wash Out board: the offer equals the five colors. A sixth answer cannot be written (`Color` has five arms and no colorless), which the test's comment says rather than claiming a refusal it never reaches |
| `ATOM-205.3m-001` | Human is a creature type, Forest is not | the Kindred Dominance board: the offer equals CR 205.3m's list as `tmnt.txt` prints it, read by the test, Human and Time Lord among it; Forest is a `LandType` and cannot be offered, said the same way |
| 201.4, 700.2e | — | covered already |

And the boards the binding table and decision 4 owe:

- **The picks are kept by existence (CR 400.7)**: a chosen creature moved
  between the choice and the instruction that names it is not found, and
  that instruction does nothing. A rule test, no printed card.
- **The player boards run at three seats**, so "choose an opponent" has two
  candidates: one asked, one drawn from the seeded `rng`.
- **Vindictive Lich at three seats**, failing first, and its twin.

## Gates and predictions

The gates that can fail: `cargo build --all-targets` with zero warnings;
`cargo test`; `cargo clippy --all-targets -- -D warnings` on both crates;
the dev GUI's tests, since `ChoiceKind` gains arms the window words; every
`check_*.py`, `check_rulings.py` for Expel included.

**The A/B, three arms** (`fuzz_ab --require`): `main`; the branch with Expel
unregistered; the branch. The middle arm asks no new question, so it is
predicted `IDENTICAL` to `main` except in a game where a player verb's
player is gone (decision 4, at three seats or more), which its dumps will
name game by game; the branch moves with Expel alone.

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
- Step 7's notes (`resolution-recipients-and-amounts`): `PlayerAntecedent`'s
  chosen arm for "that player controls" and "you and that player"; and
  `Antecedent`'s chosen-object arm, twin to its `TargetOfInstance`, for
  Crush Underfoot's "it deals damage equal to its power".
- `token-copies`: a token copy of a chosen object (Season of Weaving,
  Calamity, Galloping Inferno).
- `information-model`: a chosen value read over a hidden zone (62 filters,
  most of the card names), and The Toymaker's Trap's secret number.
- Step 9's cheap gaps, item 396: "a player with the most life" as a choice's
  admission (The Black Gate).
- `chosen-color-plus-could-produce`: a choice in a mana ability, which
  resolves no stack object.
- Item 394 narrowed to its remaining leaves; items 425 and 426 closed; item
  292 closed under decision 4's A.
