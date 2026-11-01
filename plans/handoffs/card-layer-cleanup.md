# Handoff — `card-layer-cleanup`, the design note

**Resume here: PRs 1 and 2 of the split are built: `card-ability-words`
(#301) and `card-instruction-words` (`cards/card-instruction-words`). Next
is PR 3, `cards-by-first-printing`**: decisions 4, 5 and 7, the move into
per-set files, `cards::fixtures` and `cards::tokens`, and the decklist with
its readers ("The split"). PR 2 built decision 1's verbs, recipients and
duration suffixes, decision 3b's `RowId::UNASSIGNED` (closing item 7d), and
one static ability with several kinds of effect, read by every reader of a
static body, with Platinum Angel as one ability.

**Opened 2026-10-31** on `cards/card-layer-cleanup`, off `main` at
`f7fa8a4e` (#300's merge). `card-layer-cleanup` is step 3 of
`roadmap-v2.md`'s route ("From triggers to v0.5"; its paragraph is under
`roadmap-v2.md#c-phase-8--card-breadth-643-atoms`): the card files put in
order before the rows after it register cards, and before the five Tarkir:
Dragonstorm Commander decks (`tdc`) add 373. This file is the design note,
reviewed before the build; the phase's last PR deletes it. Every count below
was read against `f7fa8a4e` by the scripts named under it.

## Where the work sits

Nowhere on the event path. A card's `CardData` is the printed definition
every object built from it shares, read before any event: the layer walk
seeds each object's frame from it (`compute_characteristics`, CR 613.1), the
announcement reads its instances of "target" (CR 601.2c), and
`can_begin_to_activate` reads an activated ability's restriction (CR 602.5).
The cleanup rebuilds the same values from shared words, files them by set,
and proves every card equal before and after, so no game changes. Three
shapes move under it, each read off the board and none proposed through
`perform_action`:

- **An ability's kind** (`AbilityType`), which priority, the announcement,
  the mana window and the why panel branch on (decision 2).
- **The values a store stamps**: `AbilityDef`'s and `CardData`'s instances
  of "target", filled by `CardDataBuilder::build`, and the id of every row in
  the continuous-effect (CR 613), replacement (CR 614) and restriction (CR
  101.2) registries, filled by `DurationRegistry::add` (decision 3).
- **A decklist**: game construction beside `Game::new` and the scenario
  loader, which already reads a commander (CR 103.3: "The players' decks
  become their libraries"). `setup-architecture.md` owns it (decision 5).

Two fixes travel beside it and do change games: fixtures that wear real
cards' names (item 204: two pooled, and 8 unregistered this note's census
found), and fourteen creatures without their printed creature types (item
207). Decision 6 puts them in a PR of their own.

## The intake

`python plans/check_state_of_play.py --slotted card-layer-cleanup` lists
five open items; a sixth names the phase.

| Item | What it is | Taken by |
|---|---|---|
| "Before card breadth" item 10 (`codebase-state.md#before-card-breadth-phase-8`) | No shared ability helpers, cards filed by the engine phase that first needed them, fixtures mixed with printings, instructions written as literals | decisions 1, 2, 4 and 7 |
| "Before Layers" item 7d (`codebase-state.md#before-layers-cr-613--now-during-layers`) | A `ContinuousEffect` row carries `id: 0` until the registry overwrites it | decision 3 |
| Main item 204 | Cerulean Wisps and Moonlace, test cards with real names that do less than their text; 8 unregistered functions share the shape | decision 6 |
| Main item 207 | Fourteen registered creatures lack their printed creature types; Thornweald Archer is no Elf to Elvish Warmaster | decision 6 |
| Main item 209 | Scripted test answers are positions in the option list | its own PR after the cleanup, as its Slotted line says; test-only, so nothing here waits on it |
| Main item 215's third cleanup, its card half | `static_replacement` written in three card files; `intrinsic_mana_ability` beside `mana_ability_single`. A6l, the duplicate-function inventory, moved this half here | decision 1's ability words: one `static_ability` and one `tap_for` |

`route-review` (2026-10-29) added two things the items do not carry: each
deck a decklist (decision 5), and `AbilityDef.instances`, the field every
literal writes and the builder overwrites (decision 3), which it moved here
from the cleanup of A6e, the audit that closed critical-path item 6.

### The re-count

Each literal's brackets matched across `mtgsim/src` and `mtgsim/tests`, with
comments and strings blanked, and a function's return type (`-> AbilityDef
{`) not counted as a literal. The brief's figures were counted another way
and differ by up to a sixth; the build re-counts by this method.

| Shape | Card files | Other `src` | Tests | Item's **Sized:** line |
|---|---:|---:|---:|---|
| `AbilityDef { .. }` | 121 literals, 2,127 lines | 24, 291 | 67, 864 | item 10: 116 on 2026-09-28 |
| `Effect::Atom(..)` | 238 literals, 1,362 lines, 49 verbs | 72, 206 | 428, 1,323 | item 10: 168 over 37 |
| `ContinuousEffect { .. }` | none | 13, 163 (+6 functions returning one) | 22, 221 | item 7d: ~26 sites, ~100 lines |
| A replacement or restriction row with `id: 0` | 1 | 6 | 11 | not counted before |
| `ActivationRestriction::None` | 206 sites in all | | | item 10: 145 |
| `instances: Vec::new()` | 208 sites in all | | | not counted before |
| A path naming a card module | 447 in 123 files | | | the brief: 369 `use` lines in 114 files |
| The dev GUI's | 14 | | | the brief: 15 |
| Card files | 58 files, 18,972 lines; 16,388 of them move, all but `registry.rs`, `authoring/`, `random_deck.rs` and `mod.rs` | | | the brief: 19,030 |

The card functions: 266, of which 215 are registered. `classify_cards.py`
reads 214 registered printings and one registered fixture (Loyalty Probe).
Registered nowhere are 51 card functions, 6 tokens a printed card creates
(`soldier_token`, `saproling_token`, …) and `equip`, Equip's ability word,
which the script counts as a fixture and which is an authoring word.

Counted by `literal_census.py` in the session's scratchpad (not kept: a
sizing, consumed here) and `plans/references/classify_cards.py`.

## The split

Five PRs, where the brief estimated three. Going past three needs the
owner's yes, which decisions 6 and 7 ask for: two of the five are small and
already slotted beside the phase.

| # | PR | What | Size, code plus tests (larger side) | Session |
|---|---|---|---|---|
| 1 | `card-ability-words` | Decisions 2 and 3a, and decision 1's ability words; every `AbilityDef` literal in the tree rebuilt from them; the equality test | built (#301): −2,560 / +1,557, 60 past the band, almost all of it deleted boilerplate | one |
| 2 | `card-instruction-words` | Decision 1's verbs, recipients and duration helper; the 238 card-file instructions rebuilt from them; decision 3b, the registry rows' id; one static ability with several kinds of effect, below | built (`cards/card-instruction-words`): +2,026 / −1,928, the words +1,335 / −1,630, the row id +178 / −94, the readers +488 / −183 | one |
| 3 | `cards-by-first-printing` | Decisions 4, 5 and 7: the move, the fixtures and tokens modules, the decklist and its readers | the move counted by decision 7; the rest about +1,400 / −900 | one, long |
| 4 | `creature-types-plus-fixture-names` | Items 204 and 207 by decision 6, and the misnamed functions that are printings registered | about +150 / −40, more per printing registered; both pools re-recorded | short |
| 5 | `scripted-answers-by-identity` | Item 209 | about 670 changed lines | one |

The estimates assume each ability's effect body stays where it is when its
literal becomes a word, so that a literal's boilerplate lines are deleted and
its effect's lines are untouched. Measured that way the 212 literals are
−2,138 / +672. Rewriting the effect bodies too would make PR 1 −3,282 /
+1,420, past the band, which is why the instructions are PR 2. **Decision 3b
moved from PR 1 to PR 2 during PR 1's build** (2026-10-31): PR 1 measured
−2,430 before decision 2, decision 3a's privacy made every test literal
part of it, and 3b, the registry rows' id, touches no ability. PR 2 sat at
the band's floor without it.

**One static ability with several kinds of effect joined PR 2 at #301's
review** (the owner, 2026-10-31). A card's paragraph is one ability (CR
113.2c), and one ability can make effects of different kinds, but every
reader of a static body takes the whole body as its one kind:
`replacement_of` for the replacement gather, `as_restriction` for the
"can't" gate, `as_cost_modification`, `as_trigger_multiplier`. A mixed
`Effect::Sequence` is dropped, loudly in tests through the layer lowering's
`debug_assert` and silently in release. So Platinum Angel's one paragraph
is written as two abilities, which is wrong wherever abilities are counted,
granted or listed. PR 2's "and" is the first word that writes a mixed body,
and about 78 printed Aura and Equipment paragraphs mix layer changes with a
"can't" (Aether Tunnel) and 42 hold two different can'ts (Arrest), Scryfall
regexes kept within one paragraph. The build: each of those readers yields
every member of its kind in a static body's list, each with its own "as
long as"; the gather's carrier filter learns the list; the layer lowering
takes the atoms and leaves the other kinds to their readers; Platinum Angel
becomes one ability. About 30 call sites. The gather is hot, so the
close-out's callgrind arm reads it.

**PR 2 as built** (2026-10-31). The words differ from decision 1's sketch
where a card asked for less: two durations (`.until_end_of_turn()`,
`.until_end_of_game()`) beside a static ability's bare form, and six
continuations (`.and_gets`, `.and_gains`, `.and_loses_all_abilities`,
`.and_has_base_power_and_toughness`, `.and_becomes_color`,
`.and_becomes_types`), each with its card; "creatures you control" is
`each(creatures_you_control())`; a replacement effect or "can't" a
resolution creates is a `CreatedEffect`, which no chain and no static
ability takes. No "and" word writes a mixed list: Platinum Angel writes
its two "can't"s as the enum, and a member word waits for the first card
with a layer change in a list (Aether Tunnel, `combat-restrictions`). A
layer change under its own "as long as" in a list is refused for now
(`codebase-state.md` main item 359). The fingerprints were rewritten once,
for Platinum Angel; what the pin owes is item 358's.

**Every PR carries a consumer**: PR 1's and PR 2's are the card files,
PR 3's is the decklists, PR 4's are Elvish Warmaster and its fixtures,
PR 5's are the tests.

## Decisions

1. **The constructor vocabulary: one word per ability kind and one per
   verb.**

   Today an ability is an eight-field literal. A card author decides three of
   its fields (its text, its effect, and an activated ability's costs), the
   builder stamps two (`id`, `instances`), and the other three are one constant
   per kind. An instruction is `Effect::Atom(primitive, recipient)`, where
   "target creature" is
   `EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1))`.
   At the review of A6g, the dev GUI's build, which gave each ability its own
   paragraph of rules text, the owner decided (2026-10-02) that each
   instruction template is one named constructor in `cards/authoring/`, and
   that the static parser's phrase table maps to the same constructors. The
   question is what a template is, and where the vocabulary stops.

   | What a word builds | Card files | Other `src` | Tests | Example | Holds today |
   |---|---:|---:|---:|---|---|
   | A spell ability (CR 113.3a) | 46 | 9 | 24 | Lightning Bolt | literal; a private `spell` in two card files |
   | A static ability (113.3d) | 56 | 7 | 21 | Glorious Anthem | literal; a private `static_ability` in four card files, `static_replacement` in three, `test_support::static_ability` |
   | An activated ability (113.3b) | 11 | 0 | 16 | Merfolk Thaumaturgist | literal |
   | An activated mana ability (605.1a) | 5 | 6 | 3 | Sol Ring; CR 305.6's | `CardDataBuilder::mana_ability_single`, `land_types::intrinsic_mana_ability`, a test helper in `phase_ld_cards` |
   | A triggered ability (113.3c) | 1 | 1 | 1 | — | `authoring::triggered_ability` builds the rest |
   | A characteristic-defining ability (604.3) or a restricted activation (602.5) | 7 and 2 | | | Tarmogoyf; Aggravated Assault | the field, written out |
   | An instruction, one verb on one recipient (CR 608.2c) | 238 over 49 verbs | 72 | 428 | "Target player draws three cards." | literal |
   | …its verb and recipient together | 129 pairs, 83 of them once | | | | |
   | A recipient (CR 115.1, 109.5) | 12 arms; "you" reaches 16 verbs, "target" 24 | | | | literal; a private `target_creature` in one card file |
   | A duration (CR 611.2a) | 46 until end of turn, 60 while the source is on the battlefield, 5 indefinite | | | | `Duration::` written out |
   | One duration over two changes to one recipient | 3 cards | "becomes red and gains haste until end of turn" | | | two atoms, the second on `SameInstanceAs(0)` |

   The printed cards, by verb and by who it acts on (Scryfall,
   `game:paper -is:funny`, 2026-10-31):

   | Verb | Any recipient | Recipients counted apart |
   |---|---:|---|
   | Draw cards | 3,708 | a target player 113, each player 50, a player the effect names ("that player", "its controller") 46 |
   | Deal damage | — | any target 675, a target creature 502, a target player 230, "each …" 505 |
   | Gets +N/+N until end of turn | 1,920 | with a second change ("and gains …", "and is …") 431, 184 of them on a target |

   Queries, each taking any amount: `o:/\bdraws? [^.]*\bcards?\b/`,
   `o:/\btarget (player|opponent) draws\b/`, `o:/\beach (player|opponent) draws\b/`,
   `o:/\b(that player|its controller|its owner|defending player|the chosen player) draws\b/`,
   `o:/deals [^.]*damage to any target/`, `o:/deals [^.]*damage to target creature/`,
   `o:/deals [^.]*damage to target (player|opponent)/`,
   `o:/deals [^.]*damage to each (opponent|player|creature|other)/`,
   `o:/gets [+-][0-9x]+\/[+-][0-9x]+ until end of turn/`,
   `o:/gets [+-][0-9x]+\/[+-][0-9x]+ and (gains|has|loses|is|becomes) [^.]* until end of turn/`.
   Blood for the Blood God!'s "draw eight cards" and The Ten Rings' "draw
   cards equal to the difference" are in the first; the first count of
   this table stopped at "seven" and at a numeral, and missed both.

   **What the counts can carry.** Each is a floor: a regex misses a pronoun
   subject ("they draw"), a sentence that names its player before the
   verb, and every phrasing nobody thought to write. So no total here
   decides anything. What decides is the pairing: each verb prints with
   several recipients, which the floors show, and which the tree counts
   exactly (67 card-file instructions act on "you" through 16 verbs, 44 on
   a target through 24). The recipient is the axis a word must not fix.

   - (a) **A word per verb, its recipient and amount as arguments, and a
     word per recipient.** "Target player draws three cards" is
     `draw_cards(target_player(), 3)`, and Lightning Bolt
     `deal_damage(any_target(), 3)`.

     **A continuous change takes its duration as a suffix, as the text
     prints it**, and a second change to the same object is a suffix too:

     | Card | Its text | Its words |
     |---|---|---|
     | Giant Growth | "Target creature gets +3/+3 until end of turn." | `gets(target_creature(), 3, 3).until_end_of_turn()` |
     | Kindled Fury | "Target creature gets +1/+0 and gains first strike until end of turn." | `gets(target_creature(), 1, 0).and_gains(FirstStrike).until_end_of_turn()` |
     | Moonlace | "Target spell or permanent becomes colorless." | `becomes_colorless(target_spell_or_permanent()).until_end_of_game()`, CR 611.2a's default when the text states none |
     | Glorious Anthem | "Creatures you control get +1/+1." | `static_ability(text, gets(creatures_you_control(), 1, 1))`: no suffix, since a static ability's effect lasts while the ability does (CR 611.3a) and the static word stamps it |

     A change word returns a change awaiting its duration, which is not an
     `Effect`: a spell that forgets "until end of turn" does not compile,
     and only `static_ability` takes one bare. The continuation methods
     follow the text's own verbs, `.and_gets`, `.and_gains`, `.and_loses`,
     `.and_is`, `.and_becomes`.

     **What the chain writes for you is the back-reference.** Kindled Fury
     has one target, chosen once as the spell is cast (CR 601.2c), and its
     second change is to that same creature. The engine holds the two
     changes as two instructions, and the second one's recipient cannot be
     "target creature" again, which would be a second target the player
     chooses. It is `SameInstanceAs(0)`: "whatever was chosen for this
     effect's first target". That pointer back at an earlier instruction's
     target is the back-reference, and today a card file writes it by hand.
     `.and_gains` puts it there: the same target as `SameInstanceAs`, or the
     same recipient when it names no target ("this creature gets … and
     gains …").

     The ability words are `spell_ability(text, effect)`,
     `static_ability(text, effect)`, `activated_ability(text, costs,
     effect)`, `mana_ability(text, costs, effect)`, today's
     `triggered_ability`, the keyword words (`equip`, today's `persist` and
     `undying`), and `tap_for(mana)` for CR 305.6's ability, with
     `.characteristic_defining()` and `.only_as_sorcery()` for the two rare
     fields. *Code*: about 49 verb words, 12 recipient words, the change
     type with its seven durations and five continuations, ~550 lines with
     their doc lines, and ~80 for the ability words, in
     `cards/authoring/abilities.rs`, `instructions.rs` and `recipients.rs`.
     *Performance*: a card is built once, and built equal; but an ability
     the engine makes in the layer walk is built on every pass, and PR 1
     measured what that and decision 2's larger kind cost: +1.28%
     instructions a decision (`fuzz-record.md`), inside the budget. *Upkeep*: a
     card adds a word only when it prints a new verb or a new recipient, and
     a new recipient works with every verb at once. The parser's table is
     two tables, phrase to verb and phrase to recipient, which is how Oracle
     text composes them.
   - (b) **A word per printed phrase, its recipient fixed**: `draw_cards(3)`
     for "you", `target_player_draws(3)`, `each_player_draws(1)`, which is
     the shape of the owner's examples (`skip_your_next_turn()`). It reads
     closest to the text at a call site. *Code*: 129 words today. *Upkeep*:
     a new recipient adds a word for each verb it pairs with, which is the
     second `CardDataBuilder` item 10 warns against.

   **Where it stops**, under (a):

   - A word returns an ability, an effect, a recipient, a duration or a
     trigger event, never a card. `CardDataBuilder` owns the card: item 10's
     line, "a helper here returns an `AbilityDef` and nothing else", widened
     to the instruction's parts.
   - **Joining one-shot instructions is the enum written out**, with no word
     of its own. Night's Whisper, "You draw two cards and lose 2 life", is
     `Effect::Sequence(vec![draw_cards(you(), 2), lose_life(you(), 2)])`;
     "you may draw a card" is `Effect::Optional { .. }` around
     `draw_cards(you(), 1)`; "if you control a Forest, …" is
     `Effect::Conditional(condition, ..)`. The card files write 40
     sequences, 12 conditionals, 7 optionals and 3 for-eaches. A one-shot's
     "and" carries nothing to the instruction after it, since each names
     its own player ("you lose 2 life"), so a `.and()` there would save only
     the `vec!`. A continuous "and" carries the recipient and the duration
     to the next change, which is the work that earns the chain.
   - A filter gets a word for a phrase two cards print. The repetition is
     the reason, as it is for `authoring::another`. The rule that asks for
     two more cards (`engineering-practices.md#cards-and-pools`) is for a
     word that becomes structure, a type or an arm the engine branches on,
     where a wrong word costs a redesign. A filter word is a name over types
     that exist, and a wrong one is inlined again.
   - A word lands with its first card, never ahead of it, and **sits in its
     file by its CR rule, not by when it landed**: each file follows the CR
     section it names (`instructions.rs` by CR 701's keyword actions in
     701's order, then the plain verbs by their rules, 119 for life, 120 for
     damage, 121 for drawing; `recipients.rs` by CR 115 and 109). A word for
     a 1993 card that lands after the `tdc` decks goes where its rule
     puts it, and every word's doc cites that rule. The alternatives were
     the order of the earliest card printing each word, which re-sorts the
     file whenever an older card arrives, and the alphabet, which is stable
     but scatters the words one rule groups.
   - The engine never calls the card words. It builds the abilities it makes
     through `AbilityDef`'s own constructor (decision 3), and CR 305.6's
     ability is `AbilityDef::tap_for(mana)` in `objects::card_data`, beside
     `tap_to_add_text`, which a card file reaches through
     `CardDataBuilder::mana_ability_single`, its one spelling. So a printed basic
     land's ability and the one its land type gives stay one ability, as
     `tap_to_add_text`'s doc asks, and no engine module depends on `cards`.
   - Tests build an ability through `AbilityDef::new` or a card word; under
     decision 3's privacy a literal no longer compiles outside
     `objects::card_data`.

   **Settled: (a)** (the owner, at #301's first review round, 2026-10-31),
   with a continuous change's duration and its "and" as suffixes, which the
   same round asked for over a prefix helper. A word sits in its file by its
   CR rule (the owner, at the second round, 2026-10-31).

2. **Where an activation's facts live: costs and the restriction on the two
   activated kinds.**

   `AbilityDef` carries `costs` and `activation_restriction` on every
   ability. Both mean something only on an activated ability, mana or not:
   CR 602.1 writes one as "[Cost]: [Effect.] [Activation instructions (if
   any).]", and a mana ability is an activated ability (CR 605.1a, "an
   activated ability is a mana ability if …"), so 602.5's restrictions reach
   it on their own terms; CR 605.3 changes only 602.2's procedure for it.
   Every other kind writes both out empty: 164 static, spell and triggered
   literals write `costs: Vec::new()`, and 206 sites write
   `ActivationRestriction::None`.

   **What this decision is not.** `permission-architecture.md#3-what-it-must-express-at-v1`
   owns what an activation may say: who may activate (602.2, there "a field
   on `AbilityDef`, read by `can_begin_to_activate`"), when beyond priority
   (602.5's limits, steps and conditions, and 602.5b's and 602.5c's
   persistence), and 606.3's loyalty limit, which is per permanent and so
   on no ability at all. `permission-design` (route step 9) designs those
   contents. This decision only chooses where they live, and moves today's
   one value there. At the review of CV-2a, a copy-effects build, the owner
   decided (2026-09-28) to move the restriction into
   `AbilityType::Activated`, "the one kind it means anything for". **It is
   not the one kind**: 60 printed mana abilities carry a restriction (Mox
   Opal, the Verge lands).

   | What `permission-design` will put there | Cards | Example | Holds today |
   |---|---:|---|---|
   | Only as a sorcery (CR 602.5d) | 503 | Bonesplitter's equip (702.6a), Aggravated Assault | `OnlyAsSorcery` |
   | Only as an instant (602.5e) | 3 | Lion's Eye Diamond, Rhystic Cave | none |
   | Only once each turn, or no more than N times | 105 + 8 | Aggressive Mining; Fire-Belly Changeling | none |
   | Only during a turn, phase or step | 139 | Amulet of Quoz | none |
   | Only if a condition holds | 223 | Ancient Hellkite, Mox Opal | none |
   | …on a mana ability | 60 | Mox Opal, Blazemire Verge, Abzan Devotee | none: `can_begin_to_activate` refuses a mana ability before it reads the field, and the mana window reads no restriction |
   | Two restrictions on one ability | 57 | Akul the Unrepentant, Ashen Ghoul | none: one value |
   | Who may activate (CR 602.1b, 602.2) | 40 | Aether Storm | none |

   Queries: `o:"activate only"` (985 in all, so 482 past sorcery timing),
   `o:"activate only as a sorcery"`, `o:"activate only as an instant"`,
   `o:"activate only once each turn"`,
   `o:/activate (no more than|only) (twice|three times)/`,
   `o:"activate only during"`, `o:"activate only if"`,
   `o:/: add [^\n]*activate only/`, `o:/activate only [^\n]* and only/`,
   `o:"any player may activate"`. The permission doc's census
   (`plans/references/cast-census.md`, 2026-10-05) read 489 limits past
   sorcery timing and 44 "any player may", a month earlier and by other
   queries. Mox Opal's one ruling reads its restriction as the activation
   begins, on the board then: with a second Mox Opal entering as the third
   artifact, neither can be activated until the legend rule has put one
   into the graveyard. That is CR 602.5's "can't begin to activate", which
   `can_begin_to_activate` asks of `OnlyAsSorcery` today; asking it of a
   mana ability in the mana window is `permission-design`'s.

   - (a) **`AbilityType::Activated { restriction }` and `AbilityType::Mana {
     restriction }`**, CV-2a's decision widened to the mana kind. *Code*: the
     54 reads of the two variants (31 `Activated`, 23 `Mana`) become
     patterns, and the field leaves the 206 sites. *Performance*: none.
     *Upkeep*: `costs` stays on every ability, and the permission doc's
     "who may activate" field stays on `AbilityDef` beside a restriction
     that is not there.
   - (b) **`AbilityType::Activated(Activation)` and `Mana(Activation)`, with
     `Activation { costs, restriction }`**: CR 602.1's activation as one
     value, holding today's costs and one restriction. *Code*: (a)'s reads,
     and the 20 reads of `.costs` (in 10 files) read `ability.costs()`, which
     answers no cost for a kind that has none to pay (CR 602.1a), or bind
     the `Activation` where they have matched the kind already;
     `AbilityType` stops being `Copy`, so a read that copies it out borrows
     instead. *Upkeep*: an ability that is not
     activated cannot hold a cost or a restriction, and `permission-design`
     grows one value: a list of restrictions for the 57, and who may
     activate in place of the field its §3 plans on `AbilityDef`.
   - (c) **Both stay on `AbilityDef`, and the ability words write the
     defaults.** *Code*: none past decision 1. *Upkeep*: the type still says
     a static ability has costs. Nothing reads them, so nothing is wrong
     today, and `permission-design` puts its fields beside them.

   What (a) and (b) buy over (c) is a type that tells the truth, and for
   (b) one home for `permission-design`'s fields. #297 showed that the
   first alone does not pay for a sweep, but the call sites here are the
   same under every option, since the words hide the defaults; what differs
   is the type every later activated card is written against.

   **Settled: (b)** (the owner, at #301's first review round, 2026-10-31),
   and with it the permission doc's §3 line for who may activate names
   `Activation` instead of `AbilityDef`.

3. **The values a literal writes and a store overwrites.**

   No printed card reaches these: an ability's id, CR 601.2c's instances of
   "target" read off an effect, and a registry row's id are values the
   engine assigns. Four are written at construction and replaced before
   anything reads them:

   - `AbilityDef.instances`, written `Vec::new()` 208 times and filled by
     `CardDataBuilder::build`. A def built outside the builder keeps the
     empty list. That is right for every such def today and, as the field's
     doc says, silently wrong for a future one with a target.
   - `CardData::spell_instances`, the same kind of value for a spell, public
     and filled by `build`, which two `CardData` literals inherit through
     struct update (`put_on_stack.rs`'s kicker test,
     `tests/phase_ma2_integration_test.rs`). Its doc gives the reason it is
     stored: it is read once per card in hand per priority pass.
   - `AbilityDef.id`, `AbilityId::UNASSIGNED` in every card literal, which
     the builder stamps. Tests set real ids on purpose (`new_ability_id()`,
     33 sites), so it stays writable, as `ObjectId` does: `GameObject::new`
     writes `ObjectId::UNASSIGNED` and `add_object` stamps it.
   - The row id, `id: 0` on 42 lines, overwritten by
     `DurationRegistry::add`: the continuous-effect rows (35 literals and 6
     functions returning one, item 7d, some of them inheriting the id
     through `..test_support::registered(..)`), and 18 replacement and
     restriction rows.

   **3a. The instances.**

   **What they are, from Magic.** A player casting Lightning Bolt is asked
   for one target, "any target": CR 601.2c's instances of the word
   "target". The engine reads that list off the ability twice. Whether an
   ability can be activated at all needs a legal choice for each instance,
   asked at every priority check (`every_instance_has_a_choice`, called from
   `can_activate` in `oracle/mana_helpers.rs`). The announcement then asks
   for each in order (`engine/put_on_stack.rs`). So each ability stores the
   list, and a spell its card's (`spell_instances`).

   Today Lightning Bolt's literal (`cards/alpha.rs`) writes the list empty,
   `instances: Vec::new()`, and `CardDataBuilder::build` overwrites it with
   `[Target(Any, Exactly(1))]`. Every card literal writes that empty list
   for the builder to replace. An ability the engine makes at run time
   without the builder keeps whatever list it was given, and one with a
   target would never be asked for it. Under (a) the card writes
   `spell_ability("Lightning Bolt deals 3 damage to any target.",
   deal_damage(any_target(), 3))`. The list is computed from the effect as
   the ability is built, and nothing outside `objects::card_data` can write
   it, so nothing can write it wrong.

   - (a) **Both lists private to `objects::card_data`**, read through
     `instances()` and `spell_instances()`. `AbilityDef::new(kind, text,
     effect)` fills the list from its effect, and `build` refills every
     def it reaches, as now; `.with_id(id)` serves a site that sets an id.
     The one write to a def's effect after construction, `functioning_in`,
     wraps it in `Effect::FunctionsIn`, whose instances are its body's
     (`Effect::for_each_instance`), so the list stays right. *Code*: the 7
     readers; every `AbilityDef` literal outside `card_data.rs` must go,
     121 card, 24 engine and 67 test, and the compiler lists each one, so
     the migration is checked, not judged; three struct-update sites
     (`..spell(..)`) become calls, and the two `CardData` struct updates
     clone the card and assign the field they change. *Upkeep*: no def is
     built with a wrong list, and
     no literal can write a stamped field again.
   - (b) **Both stay public**, the words writing the defaults. *Code*: the
     card literals only. *Upkeep*: engine and test literals stay legal, and
     the hazard the field's doc names stays.
   - (c) **Neither is stored**: each reader calls `effect.instances()`.
     *Code*: the two fields and their filling go, the 208 sites with them.
     *Performance*: `can_activate` would walk and collect every activatable
     ability's instances at every priority point (`oracle/mana_helpers.rs`),
     and a spell's at every card in hand, the cost `spell_instances`' doc
     stores the list to avoid; the close-out's instruction counts would
     price it. *Upkeep*: nothing can go stale.

   **Settled: (a)** (the owner, at #301's second review round,
   2026-10-31). (c) is the cleanest shape, but it reopens a cost the tree
   already decided against, for a staleness (a) closes.

   **3b. The row id.**

   It is assigned as it is added: `DurationRegistry::add` stamps it. The
   placeholder exists only because `id` is a field of the row the caller
   builds, and Rust makes a caller write every field, so it writes `0` for
   `add` to overwrite. Assigning it only as the row is added means the value
   the caller builds has no `id` at all. (a) and (b) do that; (c) only names
   the placeholder.

   - (a) **`ContinuousEffectDraft`, as item 7d planned**: the row's fields
     less `id`, consumed by `add`, which builds the row. *Code*: a second
     struct of nine fields and its conversion; the 35 literals and 6
     functions retyped; `add`'s signature. The replacement and restriction
     rows keep `id: 0`. *Upkeep*: two field lists, kept in step by the
     conversion's exhaustive destructure, for one registry of three.
   - (b) **The id leaves the row in all three registries.**
     `DurationRegistry` stores and hands out `Registered<T> { id, row: T }`,
     dereferencing to the row so `row.layer` reads as it does today, and
     `DurationRow` loses `id` and `set_id`. The rows are already named
     `RegisteredReplacementEffect` and `RegisteredRestriction`, which before
     registration they would not be, so they are renamed (49 uses).
     *Code*: the 42 `id: 0` lines go, plus the registry's internals, the
     renames, and the signatures that read a row's id (`EffectId` named in
     25 places, `ReplacementEffectId` in 6). *Upkeep*: no row can be built
     with an id, but a wrapper with `Deref` is one more indirection a reader
     meets, and the rows would spell "stamped by its store" differently from
     objects and abilities.
   - (c) **The engine's spelling for a value its store stamps, as objects
     and abilities have it**: `RowId` becomes a newtype with
     `RowId::UNASSIGNED`, the 42 lines write it, and `add` asserts it.
     *Code*: the newtype and its ~57 uses (`RowId` 26, `EffectId` 25,
     `ReplacementEffectId` 6), and the 42 lines. *Upkeep*: the field stays,
     named and checked, and one spelling covers all three stores; a row id
     can no longer be mistaken for a count.

   **Settled: (c)** (the owner, at #301's third review round,
   2026-10-31), revised again at the second round. The owner found
   (b) a wrapper type that clutters every registry read, and a shape that
   leaves objects and rows spelling "the store fills this in" two ways.
   (c) is that one spelling across all three stores: an object writes
   `ObjectId::UNASSIGNED`, an ability `AbilityId::UNASSIGNED`, and a row
   `EffectId::UNASSIGNED` or `ReplacementEffectId::UNASSIGNED`, through the
   aliases, each read as its own registry's id. It adds no type a reader
   meets and renames nothing. What it costs, measured: `RowId` becomes a
   newtype deriving what `ObjectId` derives plus `Ord`, since CR 613.7a's
   sub-order sorts rows by id. The 56 uses of the three names compile
   through unchanged, the counter's one `+= 1` becomes a method, and the
   42 `id: 0` lines name the placeholder. No trace, scenario, UI or dev GUI
   code prints a row id, so no output changes. What it gives up: a row
   still carries the field before it is added. `add` asserts the field is
   `UNASSIGNED`, so a row re-added with a stale id fails loudly, and item 7d
   closes as named and checked rather than removed.

   **(b), as it was recommended at the first round**, when the owner asked
   why a row carries an unassigned id at all; kept as the record of that
   answer. The caller builds a row with no id, and the registry pairs it
   with one as it adds it, in all three registries. The two row types take
   names that say what they are before they are registered (`ReplacementEffectRow`,
   `RestrictionRow`), and `ContinuousEffect` keeps its own. Item 7d's draft
   does the same for one registry of three, by a second copy of its fields.
   An ability's id stays `AbilityId::UNASSIGNED` until `build`, since tests
   set real ones and the card words write the default, so no card file
   names it.

   **What (b) changes, against today** (asked at the second round):

   - *The code*: `Registered<T> { id, row: T }`, dereferencing to `T`;
     `DurationRegistry` stores `Arc<Registered<T>>` and `add(row: T)`
     returns the id; `DurationRow` loses `id` and `set_id`, in its three
     implementations and its two test rows. The 42 `id: 0` lines go, and
     the two renames touch 49 uses. A function that reads a row's id takes
     the wrapper: the row types are named in 31 signatures in 6 files, and
     about 22 reads take an id. A function that only applies an effect keeps
     taking `&ContinuousEffect`, and can no longer read an id it has no use
     for.
   - *Readers*: `row.layer` and every other field read stays as it is,
     through `Deref`, and no code destructures a row. `Deref` on a wrapper
     that is not a pointer is the one shape a reader meets here and nowhere
     else; without it, every field read through the registry would change.
   - *Behavior*: none. Ids come from the same counter in the same order, so
     every `EffectId`, CR 613.7's id tiebreak in `effects_in_layer`, CR
     614.5's applied sets and the close-out's rows are unchanged.
   - *Performance and the clone*: none. The id moves from inside the row to
     beside it, behind the same `Arc`, so a fork shares a row as it does
     today (`clone-architecture.md`).
   - *Item 7d*: closed by another shape than its **Sized:** line's draft,
     which its close-out entry records.

   **Objects, and whether stopping at rows is halfway.** Under (c) the
   question does not arise, since every store spells its placeholder
   `UNASSIGNED`. Under (b) it did, and this was the answer. The smell item 7d
   names is a call site writing a meaningless value, and only a value built
   as a literal has one. The 42 row literals each write `id: 0`. A
   `GameObject` is built through one door, `GameObject::new`: 199 calls,
   none of which writes an id, since `new` writes `ObjectId::UNASSIGNED`
   itself and `add_object` stamps the id and the timestamp as it stores the
   object, keyed by that id, in its `ObjectTable`. After decision 3a an
   ability is built the same way, through `AbilityDef::new`. So the rule
   the tree would follow is one rule: **a value built as a literal at many
   sites has no field its store assigns; a value built through one
   constructor may keep that field, and the constructor writes it.** Objects
   and abilities are the second kind, rows the first. What objects keep is
   a state the type allows and the store never holds, an unadded object,
   seen only between `new` and `add_object`. Removing it is the object
   store's own change: the id duplicates the table's key, and dropping it
   would pass the key beside every `GameObject` read that uses its id,
   across the engine. A row could instead go the objects' way, its id
   private behind a constructor, but a continuous-effect row has nine named
   fields, and nine positional arguments read worse than the literal.

4. **The per-set layout, the fixtures, and how a test names a card.**

   | What the layout must hold | Count | Example | Holds today |
   |---|---:|---|---|
   | Printings, filed by their first paper set that is no promo, memorabilia or box set | 214 in 91 sets, 50 of them holding one card | Limited Edition Alpha (`lea`): 38 | 58 files named for engine phases or card types |
   | A set whose Scryfall code cannot name a Rust module | 15 of Scryfall's 1,004 paper sets begin with a digit; no code is a Rust keyword. In the tree, Fifth Dawn (`5dn`) alone, with 5 cards | `5dn`, Tenth Edition's `10e` | — |
   | A set whose code cannot name a file everywhere | Conflux's `con`, which Windows reserves | — | — |
   | A registered token, dealt as a land card | 1 | Everywhere, first printed among Duskmourn's tokens (`tdsk`) | `dual_lands` |
   | A token definition a printed card's effect makes (CR 111.3: its text is the effect's) | 6, three of them made by cards in two sets | the Saproling (Verdant Force, Bramble Elemental) | beside one of its makers |
   | A registered fixture | 1 | Loyalty Probe, for CR 704.5i | `phase_rd_cards` |
   | An unregistered card function | 51, 8 of them under printed cards' names | `flight_clause`; `dual_land_ub` named Underground Sea | 11 phase files |
   | …of which the fixtures pool deals | 5 | `fixture_curtain` | `phase_history_and_frame_records_cards::CASTABLE` |
   | A keyword's ability word (CR 702.6a) | 1 | `equip`, which three equipment call | `phase_lh_cards`; moved to `authoring/abilities.rs`, where an activated keyword's word sits; `persist` and `undying`, CR 603's, stay in `triggers.rs` |

   `classify_cards.py --first-printings --list`; the sets by
   `api.scryfall.com/sets`, less the digital ones; the names by
   `/cards/collection` (2026-10-31). **The script's key was wrong, and this
   note fixes it.** It took the earliest printing of any kind. So a
   prerelease promo dated days before its set (Dark Ascension's promos,
   `pdka`, before `dka`), an Arena League or FNM reprint, or a boxed reprint
   collection that came first (Salvat 2005 before Ravnica) filed six cards
   under a set that is not theirs, and its search missed the one token. It
   now takes the earliest paper printing outside a promo, memorabilia or box
   set, by the set's type and not the printing's promo flag, so Nexus of
   Fate's buy-a-box printing stays in Core Set 2019 (`m19`). It falls back
   to any paper printing for a card printed nowhere else, and finds a token
   among the extras. It also stopped counting `random_deck`'s two functions
   as fixtures. A card the rule files under a product older than its
   expansion stays there, as the rule says: Goblin Electromancer in Duel
   Decks: Izzet vs. Golgari (`ddj`), which printed it a month before Return
   to Ravnica.

   **What builds as item 10 planned, each with its one rule:**

   - **A set** is one file in a folder for its release year, named by its
     release month and day and its Scryfall code (the owner, at #301's first
     review round): Alpha is `cards/sets/1993/08_05_lea.rs`. A folder lists
     its sets in the order they came out, the order in which new abilities
     built on old ones. A file's name need not be a Rust identifier, so
     `sets/mod.rs` declares each set by its code, in release order,
     `#[path = "1993/08_05_lea.rs"] pub mod lea;`, and the registry and
     `printings` name a set by its code. A code that cannot name a module
     takes the prefix `set_` in the module name only (`set_5dn`); Conflux's
     `con` is no file name problem once the file begins with its date. Two
     sets released the same day sort by code (Tarkir: Dragonstorm's `tdm`
     and its Commander decks' `tdc`). The date is Scryfall's `released_at`
     for the set, which PR 3's `classify_cards.py --list` prints beside each
     code.
   - **A fixture** is filed in `cards::fixtures` by the architecture doc
     whose path it exists to test, and one no doc owns by the CR subject it
     exists for. By destination: `layers` 19 (the layer phases' files),
     `cost` 7, `triggers` 9 (history records and state triggers),
     `replacement` 3, and `state_based_actions` 1 (Loyalty Probe), with the
     8 misnamed functions among them until PR 4 settles each (decision 6).
   - **A token definition** is filed in `cards::tokens`, named by what it
     makes, since a token's text is its maker's effect (CR 111.3) and
     makers come from several sets. A registered token, which is dealt as a
     card, is a printing: Everywhere in `tdsk`.
   - **Registration** stays in `registry.rs`, whose comments say why each
     card is pooled; each path is updated.
   - **The phase files' module docs** do not move: what one says about a
     card goes to that card's doc comment, and the rest is the engine
     phase's history, which the architecture docs hold.
   - **The 8 functions under printed names** move into `cards::fixtures`
     as they are, and PR 4 settles each (decision 6).

   The genuine choice is **how a test or the dev GUI names a card**:

   - (a) **By its set**: `use mtgsim::cards::sets::lea::lightning_bolt`.
     *Code*: the 447 paths and the dev GUI's 14 each name the set.
     *Upkeep*: whoever writes a test looks up the card's first printing.
   - (b) **Through one flat module of printings, beside `fixtures` and
     `tokens`**: `use mtgsim::cards::printings::lightning_bolt`, where
     `printings.rs` re-exports every set (`pub use super::sets::lea::*;`, one
     line a set). *Code*: the same 447 paths, each naming the card and not
     its set, and ~95 re-export lines. *Upkeep*: a test reads what it uses,
     and whether it is a printing, a fixture or a token is visible at every
     import, which is item 10's reason for the split. Two sets defining one
     function name would be an ambiguity error where the name is used,
     which is the error wanted.

   **Settled: (b)** (the owner, at #301's first review round, 2026-10-31),
   with the sets filed by year and release date as above.

5. **The decklist: its format, and what reads it first.**

   | What a decklist must express | Cards | Example | Holds today |
   |---|---:|---|---|
   | Named cards with counts (CR 100.2a), a name on more than one line | every deck | `cli_play`'s 10 Mountains, then 8 more at its end | `Decklist = Vec<Arc<CardData>>` (`state/game.rs`), built in code by `cli_play::build_test_deck` and `random_deck` |
   | One commander, "an attribute of the card itself" (CR 903.3) | 3,761 cards can be one | each `tdc` deck | a scenario's `commander` card word (`scenario/text.rs`), which the loader stamps as `is_commander` (`scenario/build.rs`); no decklist |
   | Two commanders: partner, friends forever, choose a Background, Doctor's companion (CR 702.124) | 129 + 7 + 32 + 28, and the Backgrounds | Tymna and Thrasios | the scenario's word, on two cards |
   | A companion, revealed from outside the game (CR 702.139a, 103.2b) | 10 | Lurrus of the Dream-Den | nothing |
   | A sideboard (CR 100.4a) | constructed play | — | nothing |

   Queries: `is:commander`, `kw:partner`, `o:"friends forever"`,
   `o:"choose a background"`, `o:"doctor's companion"`, `kw:companion`.

   **What `route-review` recorded** (the owner, 2026-10-29,
   `roadmap-v2.md#c-phase-8--card-breadth-643-atoms`): "each deck is a
   decklist, a data file of its 100 names that a test loads against the
   registry, so a reprint is filed once, in its first printing's set". The
   use is settled: the five `tdc` decks. What this decision asks is the
   file's grammar, which reads it first, and when the five lists arrive.

   **The format:**

   - (a) **The deck sites' text**: a `count name` line under `Commander`,
     `Companion`, `Deck` and `Sideboard` headers, `#` comments, a name
     allowed on several lines (read in order), and Arena's and Moxfield's
     `(SET) 123` suffix accepted and ignored. It is what Scryfall, Moxfield
     and Archidekt export and Arena imports, so a product's list is pasted
     from its source and diffed against it. *Code*: a hand-written parser
     (no new dependency, as `setup-architecture.md` chose for the scenario
     format), ~70 lines and its tests. *Upkeep*: the commander designation
     is spelled twice in the files the engine reads, a scenario's
     `commander` word and a decklist's `Commander` header, read by two
     parsers that both end in the object's `is_commander`.
   - (b) **The scenario grammar's card lines**, the zone words the loader
     already reads: `library: Forest | x10` and `command: Isamaru, Hound of
     Konda | commander`, with the player taken from the seat the deck is
     dealt to. *Code*: the scenario parser, given a decklist mode with no
     player numbers. *Upkeep*: one grammar and one commander word for every
     file the engine reads, and a save could embed a decklist as it embeds a
     scenario; each list from a deck site is rewritten line by line, 500
     lines for the five `tdc` decks.

   **Where it lives**, whichever the format: `setup-architecture.md` gains a
   section, since CR 103.3 turns the decks into libraries; the parser goes
   in `mtgsim::scenario`, game construction beside `Game::new`; the files go
   in `mtgsim/decks/`. The names are a `Decklist`, the word the route and
   the deck sites use, resolved against the registry into a `Deck`, CR
   103.3's word for what `Game::new` deals: today's `Decklist` alias is
   renamed `Deck` (8 uses), and an unknown name is refused with its line.
   The commander section is parsed and held, and `game-formats` reads it
   when it designates (its Slotted line, "Before Commander" item 2, which
   is updated to name it).

   **What reads it first:**

   - (a) **`cli_play`'s deck, with the five `tdc` lists at step 13**:
     `build_test_deck`'s forty cards become `mtgsim/decks/cli-play.txt`, in
     the same order, and a test loads every file in `mtgsim/decks/` against
     the registry. The five lists, from MTGJSON's deck files (a download
     each), arrive with step 13's gap census, the first reader that wants
     the names the registry lacks.
   - (b) **The five `tdc` lists now, as the route recorded**, beside
     `cli_play`'s. The test loads each, asserts what holds before any of
     their cards is registered (100 cards, one commander, one copy of each
     card but basic lands, CR 903.5a–b), and prints the names the registry
     lacks, which step 13's census starts from.

   **Settled: format (a), reader (b)** (the owner, at #301's first review
   round, 2026-10-31).

   **Which formats may name a commander, and the variants after v0.5**
   (asked in the same round). The parser reads sections and judges none;
   whether a section is legal is the format's, and `game-formats`, the row
   that makes formats data, validates a `Decklist` against its format as
   it builds `DeckLimits`' check ("Before Commander" item 2, main item 32).
   A `Commander` section is legal only in a format whose rules designate a
   commander, which the CR gives as Commander (903), its Brawl option
   (903.12, of which Standard Brawl is a format, not a CR rule) and
   Commander Draft (903.13); every other format refuses one, with its line.
   The other variants keep objects outside the deck that a decklist would
   list: a planar deck (Planechase, CR 901), a vanguard card (902), a scheme
   deck (Archenemy, 904), conspiracies (905), an Attraction deck (717). The
   parser refuses a header it does not know. So each variant's section
   arrives with that variant's own format design, Vanguard's after v1 among
   them, and this PR builds only v0.5's four.

6. **Items 204 and 207: commits in the move's PR, or a PR of their own.**

   No printed family: both are fixes to registered cards' data. Item 204
   renames two pooled fixtures (Cerulean Wisps leaves a tapped creature
   tapped, Moonlace's text says more than it does) and registers the real
   cards only if written whole. Item 207 gives fourteen creatures their
   printed types. Each changes what both pools deal, so each owes a test
   shown failing on the tree before it and a re-record of both tables.

   **Item 204 is a family, not two cards.** Decision 4's census found 8
   unregistered card functions under 8 printed cards' names: the four
   `*_spell` stand-ins (Liquimetal Torque, Ensoul Artifact, Call to Serve,
   On Serra's Wings), Underground Sea, Inside Out, Bull Rush and Zhalfirin
   Shapecraft. `engineering-practices.md#cards-and-pools` forbids a fixture
   that wears a real name while behaving differently.

   **What each of the ten becomes** (the owner, at #301's first review
   round): most were bootstrap cards, written before fixtures were a
   practice. One the engine can express now is written whole, as its card,
   with its rulings read, and registered, since writing a card and
   registering it are one act. One it cannot express keeps its narrow shape
   as a fixture: its function takes the suffix `_fixture`, and its card name
   the same word (`cerulean_wisps_fixture`, "Cerulean Wisps Fixture"), so
   neither reads as the printed card. The four `*_spell` stand-ins are
   fixtures by purpose whatever the engine can do, since each gives a test
   a spell with a permanent's effect (`liquimetal_coating_spell`'s doc:
   "Modeled as an instant spell for testing"), and take the suffix.

   **How that bears on the choice below**: it settles what each fix is, not
   where it lands. Every one of the ten either moves a pool's stream (the
   two pooled ones, renamed or rewritten, and any card registered) or moves
   nothing (an unregistered one renamed), so they go together with item
   207. PR 3 moves the 8 into `cards::fixtures` as they are, which keeps it
   a pure move; one written whole moves again, into its set, in the fixes'
   commits.

   **What is recorded**: item 207's Slotted line says "as its own commit
   after the move" (the owner, 2026-10-02), which is (a); item 204's, "with
   `card-layer-cleanup`, which moves the fixtures".

   - (a) **Their own commits at the end of PR 3**, as recorded.
     *Close-out*: three arms (main, the move's last commit, the fixes): the
     move's rows identical, the fixes' rows moved and re-recorded.
     *Upkeep*: one PR whose identical-or-not answer depends on which commit
     is asked.
   - (b) **One PR of their own, right after the move**, ~150 lines with the
     tests, more for each card written whole. *Close-out*: PR 3's two arms
     read identical, and PR 4 re-records once for everything that moves a
     stream. Its edits land in the files they stay in: 204's fixtures are
     born in `cards::fixtures`, and the fourteen are edited in their set
     files.
   - (c) **Before PR 1**, so every cleanup PR is measured on fixed data.
     204's fixtures are then written in a phase file and moved.

   **Settled: (b)** (the owner, at #301's second review round,
   2026-10-31), which departs from both Slotted lines: the owner's later
   lesson from #297 (2026-10-27) puts behavior in its own small PR, and a
   three-arm close-out is the cost (a) carries. Both items are re-slotted
   to PR 4 at the close-out.

7. **How the move counts against the band.**

   Filing the 16,388 moving lines by first printing deletes each from its
   phase file and adds it to a set file, so `git diff --numstat` reads about
   16,400 a side: six and a half bands, though no line in a moved body
   changes. Item 10 promised that "each moved file is a pure rename and its
   history follows it". By set, a phase file scatters across many sets, so
   no file is a rename and `git log --follow` loses each card's history;
   `git log -S` and `git blame -C -C` still find a line's. The promise is
   dropped either way.

   - (a) **A moved line counts zero, and the PR shows the move is pure.**
     A throwaway script compares each card function's text in the new tree
     with its text at the base, byte for byte, and its output goes in the PR
     body. The equality test reads every card equal, and `git diff
     --color-moved` shows the rest. The band reads what is not a move
     (module files, imports, registry paths, docs, the decklists, and the
     module-doc text that becomes card doc comments), and reports the moved
     count beside it, as docs are. It is a rule for any later move too, so
     `engineering-practices.md#sizing-a-phase` gains a line saying so.
   - (b) **A moved line counts**, so the move splits into about seven PRs by
     set group. Each is a pure move, which no reviewer reads line by line
     either.

   **Recommended: (a).** The review a moved line needs is the proof that it
   moved, not a reading of it.

## What builds without a decision

- **The equality test** (PR 1, deleted at PR 3's close-out). It
  fingerprints every card function, the 215 registered, the fixtures and
  the token definitions, by a canonical `{:#?}` with each `HashSet`'s
  entries sorted, since `CardData`'s sets iterate in a per-process order.
  It compares them with a file written at the base
  (`UPDATE_CARD_FINGERPRINTS=1` rewrites it), and a mismatch names the
  card. **The file never changes through PRs 1–3.** A commit before the
  reshape replaces `AbilityDef`'s derived `Debug` with one written against
  accessors (its kind's name, `costs()`, the restriction, its text,
  effect, flag, id and instances), printing what the derive prints today.
  Decisions 2 and 3 then change the fields behind the accessors, not that
  output, so the commit that reshapes the types is checked against the
  base like every other. The derive comes back when the file goes. A
  Deferred Migrations line carries both until then.
- `classify_cards.py`'s key (decision 4), in this note's commit; PR 3 makes
  it walk `cards/sets/`, `cards/fixtures/` and `cards/tokens/`, since its
  glob reads the card directory's top level only.
- `authoring/triggers.rs`' doc line "A triggered ability: no cost, the def
  as its effect." sits above `functioning_in`; PR 1 moves it to
  `triggered_ability`, the function it describes.
- The engine-phase card files' doc comments that name a card's role
  (`registry.rs`'s and the modules') follow each card, and `CLAUDE.md`'s
  conventions line, "Test cards in `src/cards/phase_XX_cards.rs`", becomes
  the set files, `cards::fixtures` and `cards::tokens` (PR 3).
- The live docs' card-file paths, which `check_engine_map.py` reads, follow
  the move (PR 3).

## Gates and predictions

- **PRs 1–3:** `close_out.py` reads every gameplay row identical on both
  pools at two and four seats, and the equality test reads every card
  equal. A row that moves is a defect in the PR, not a re-record.
- **PR 3:** the dev GUI's build and tests, since its imports change;
  `check_module_layout.py` for the new `mod.rs` files; `check_engine_map.py`
  for the moved paths.
- **PR 4:** each fix's test fails on the tree before it; both pools'
  streams move, and the re-record goes in `fuzz-record.md`.
- **Every PR:** `cargo build --all-targets` with no warnings, `cargo test`,
  `cargo clippy --all-targets -- -D warnings`, the release tests, and the
  `check_*.py` family.

## Filed at the close-out, if the decisions stand

- Item 215's card half closed with PR 1. Item 7d closes with PR 2, item 10
  and main item 358 (the fingerprints) with PR 3; items 204 and 207 with
  PR 4, both re-slotted there by decision 6; item 209 with PR 5. Item 204's
  body already names the 8 (this note's commits).
- Decision 2 (b), done in PR 1: `permission-architecture.md#3-what-it-must-express-at-v1`'s
  who-may-activate line names `Activation`, and its when-beyond-priority
  line names it as the home `permission-design` grows.
- Decision 5: `setup-architecture.md` gains the decklist's section (PR 3),
  and "Before Commander" item 2's Slotted line names the `Decklist`'s
  commander section and `Deck`.
- Decision 7 (a): `engineering-practices.md#sizing-a-phase` gains the
  moved-line rule (PR 3), within the page's line budget.
