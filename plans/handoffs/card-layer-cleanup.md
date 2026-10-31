# Handoff — `card-layer-cleanup`, the design note

**Opened 2026-10-31** on `cards/card-layer-cleanup`, off `main` at
`f7fa8a4e` (#300's merge). `card-layer-cleanup` is step 3 of
`roadmap-v2.md`'s route ("From triggers to v0.5", the paragraph under
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
type shapes move under it, each read off the board, not proposed through
`perform_action`:

- **An ability's kind** (`AbilityType`), which priority, the announcement,
  the mana window and the why panel branch on (decision 2).
- **The fields a builder stamps**: `AbilityDef`'s `instances`, filled by
  `CardDataBuilder::build`, and the id every registry row carries, filled by
  `DurationRegistry::add` for the continuous effects (CR 613), the
  replacement effects (CR 614) and the restrictions (CR 101.2) (decision 3).
- **A decklist**, game construction beside `Game::new` and the scenario
  loader (CR 103.3: "The players' decks become their libraries"), which
  `setup-architecture.md` owns (decision 5).

Two fixes travel beside it and do change games: fixtures that wear real
cards' names (item 204: two pooled, and 13 unregistered this note's census
found) and fourteen creatures without their printed creature types (item
207). Decision 6 puts them in a PR of their own.

## The intake

`python plans/check_state_of_play.py --slotted card-layer-cleanup` lists
five open items; a sixth names the phase.

| Item | What it is | Taken by |
|---|---|---|
| "Before card breadth" item 10 (`codebase-state.md#before-card-breadth-phase-8`) | No shared ability helpers, cards filed by the engine phase that first needed them, fixtures mixed with printings, instructions written as literals | decisions 1, 2, 4 and 7 |
| "Before Layers" item 7d (`codebase-state.md#before-layers-cr-613--now-during-layers`) | A `ContinuousEffect` row carries `id: 0` until the registry overwrites it | decision 3 |
| Main item 204 | Cerulean Wisps and Moonlace, test cards with real names that do less than their text; 13 unregistered functions share the shape | decision 6 |
| Main item 207 | Fourteen registered creatures lack their printed creature types; Thornweald Archer is no Elf to Elvish Warmaster | decision 6 |
| Main item 209 | Scripted test answers are positions in the option list | its own PR, after the cleanup (the split, below); test-only, so nothing here waits on it |
| Main item 215, its third cleanup's card half (moved here by A6l, the inventory's merges) | `static_replacement` written in three card files; `intrinsic_mana_ability` beside `mana_ability_single` | decision 1's ability words: one `static_ability` and one `tap_for` |

`route-review` (2026-10-29) added two things the items do not carry: each
deck a decklist (decision 5) and `AbilityDef.instances`, the field every
literal writes and the builder overwrites (decision 3), which it moved from
the item 6 audit's cleanup (A6e).

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
| A path naming a card module | 447 in 123 files | | | brief: 369 `use` lines in 114 files |
| The dev GUI's | 14 | | | brief: 15 |
| Card files | 58 files, 19,030 lines | | | |

The card functions: 266, of which 215 are registered. `classify_cards.py`
reads 214 registered printings and one registered fixture (Loyalty Probe).
Registered nowhere are 51 card functions, 6 tokens a printed card creates
(`soldier_token`, `saproling_token`, …) and `equip`, Equip's ability word,
which the script counts as a fixture and which is an authoring word.

Counted by `literal_census.py` in the session's scratchpad (not kept: a
sizing, consumed here) and `plans/references/classify_cards.py`.

## The split

Five PRs, two of them small and slotted beside the phase. Decision 7 decides
how the third counts against the band, and the count passes the brief's
three, which needs the owner's yes.

| # | PR | What | Size, code plus tests (larger side) | Session |
|---|---|---|---|---|
| 1 | `card-ability-words` | Decisions 2 and 3, and decision 1's ability words; every `AbilityDef` literal in the tree rebuilt from them; the equality test | about −2,300 / +1,250 | one |
| 2 | `card-instruction-words` | Decision 1's verbs and recipients; the 238 card-file instructions rebuilt from them | about −1,400 / +950, under the band's floor | one |
| 3 | `cards-by-first-printing` | Decisions 4, 5 and 7: the move, the fixtures module, the decklist and its first reader | the move counted by decision 7; the rest about +1,300 / −900 | one, long |
| 4 | `creature-types-plus-fixture-names` | Items 204 and 207, by decision 6, with the 13 unregistered fixtures under printed names | about +150 / −40 and more per fixture that is a printing; both pools re-recorded | short |
| 5 | `scripted-answers-by-identity` | Item 209 | about 670 changed lines | one |

The estimates assume each ability's effect body stays where it is when its
literal becomes a word, so that a literal's boilerplate lines are deleted and
its effect's lines are untouched. Measured that way the 212 literals are
−2,138 / +672. Rewriting the effect bodies too would make PR 1 −3,282 /
+1,420, past the band, which is why the instructions are PR 2.

**Every PR carries a consumer**: PR 1's and PR 2's are the card files,
PR 3's is `cli_play`'s deck, PR 4's are Elvish Warmaster and its fixtures,
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
   The owner decided at A6g's ability-names review (2026-10-02) that each
   instruction template is one named constructor in `cards/authoring/`, which
   the parser's phrase table maps to. The question is what a template is, and
   where the vocabulary stops.

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

   The printed cards, by verb (Scryfall, `game:paper -is:funny`, 2026-10-31):

   | Verb | Cards | Of which with another recipient |
   |---|---:|---|
   | Draw N cards | 3,411 | "target player draws" 104, "each player draws" 43 |
   | Deal N damage to any target | 558 | to target creature 395 |
   | Gets +N/+N until end of turn | 1,678 | "gets … and gains … until end of turn" 404 |

   Queries: `o:/\bdraws? (a|one|two|three|four|five|six|seven|x) cards?/`,
   `o:"target player draws"`, `o:"each player draws"`,
   `o:/deals \d+ damage to any target/`, `o:/deals \d+ damage to target creature/`,
   `o:/gets [+-]\d+\/[+-]\d+ until end of turn/`,
   `o:/gets [+-]\d+\/[+-]\d+ and gains [^.]* until end of turn/`.
   A verb recurs across thousands of cards, and its recipient varies within
   the verb, so the recipient is the axis a word must not fix.

   - (a) **A word per verb, its recipient, amount and duration as
     arguments, and a word per recipient.** "Target player draws three
     cards" is `draw_cards(target_player(), 3)`, Lightning Bolt
     `deal_damage(any_target(), 3)`, Giant Growth
     `modify_power_toughness(target_creature(), 3, 3, until_end_of_turn())`.
     The ability words are `spell_ability(text, effect)`,
     `static_ability(text, effect)`, `activated_ability(text, costs,
     effect)`, `mana_ability(text, costs, effect)`, today's
     `triggered_ability`, and `tap_for(mana)` for CR 305.6's ability, with
     `.characteristic_defining()` and `.only_as_sorcery()` for the two rare
     fields. *Code*: about 49 verb words and 12 recipient words, ~450 lines
     with their doc lines, and ~80 for the ability words, in
     `cards/authoring/abilities.rs`, `instructions.rs` and `recipients.rs`.
     *Performance*: none; a card is built once, and built equal.
     *Upkeep*: a card adds a word only when it prints a new verb or a new
     recipient, and a new recipient works with every verb at once. The
     parser's table is two tables, phrase to verb and phrase to recipient,
     which is how Oracle text composes them.
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
   - Combinators are the enum written out (`Effect::Sequence`, `Optional`,
     `Conditional`, `ForEach`). "X and Y until end of turn" passes its
     duration to each verb. The helper `roadmap-v2.md` §C names, which would
     write the duration once, waits for the decks' gap census to count that
     family: three cards in the tree print it ("becomes red and gains haste
     until end of turn").
   - A filter gets a word only for a phrase the text prints on two cards or
     more, as `authoring::another` already does.
   - A word lands with its first card, never ahead of it.
   - Engine code builds the abilities it makes through `AbilityDef`'s own
     constructor (decision 3), never through the card words. CR 305.6's
     ability is the one word both use (`tap_for`), so a printed basic land's
     ability and the one its land type gives stay one ability, which is
     `tap_to_add_text`'s reason today.
   - Tests may use the words and need not, except where decision 3's
     privacy leaves an ability word as the only way to build an ability.

   **Recommended: (a).** (b) is the owner's example spelled out. But
   "skip your next turn" names no recipient a card could vary, and "draw"
   does: 67 card-file instructions act on "you" through 16 verbs, and 44 on
   a target through 24.

2. **Where an activation's facts live: costs and the restriction on the two
   activated kinds.**

   `AbilityDef` carries `costs` and `activation_restriction` on every
   ability. Both mean something only on an activated ability, mana or not:
   CR 602.1 writes one as "[Cost]: [Effect.] [Activation instructions (if
   any).]", and CR 605.3 has a mana ability follow 602.2's activation. Every
   other kind writes them out empty: 164 static, spell and triggered literals
   write `costs: Vec::new()`, and 206 sites write
   `ActivationRestriction::None`. CV-2a's review (the copy track's first
   build, the owner, 2026-09-28) decided to move the restriction into
   `AbilityType::Activated`, "the one kind it means anything for". **That is
   not the one kind**: a mana ability prints restrictions too (Mox Opal's
   metalcraft), and CR 605.3 makes 602.5 apply to it. Today the field's one
   reader, `can_begin_to_activate` (`oracle/mana_helpers.rs`), refuses a mana
   ability before it reads the field.

   | What the shape must express | Cards | Example | Holds today |
   |---|---:|---|---|
   | Only as a sorcery (CR 602.5d) | 503 | Bonesplitter's equip (702.6a), Aggravated Assault | `OnlyAsSorcery` |
   | Only as an instant (602.5e) | 3 | Lion's Eye Diamond, Rhystic Cave | none |
   | Only once each turn, or no more than N times | 105 + 8 | Aggressive Mining; Fire-Belly Changeling | none |
   | Only during a turn, phase or step | 139 | Amulet of Quoz | none |
   | Only if a condition holds | 223 | Ancient Hellkite, Mox Opal | none |
   | …on a mana ability (CR 605.3) | 60 | Mox Opal, Blazemire Verge, Abzan Devotee | none: refused before the field is read |
   | Two restrictions on one ability | 57 | Akul the Unrepentant, Ashen Ghoul | none: one value |
   | Who may activate (CR 602.1b) | 40 | Aether Storm | none |
   | A loyalty ability's timing, the rule's and not the text's (CR 606.3) | 320 planeswalkers | — | not built |

   Queries: `o:"activate only"` (985 in all), `o:"activate only as a
   sorcery"`, `o:"activate only as an instant"`, `o:"activate only once each
   turn"`, `o:/activate (no more than|only) (twice|three times)/`,
   `o:"activate only during"`, `o:"activate only if"`,
   `o:/: add [^\n]*activate only/`, `o:/activate only [^\n]* and only/`,
   `o:"any player may activate"`, `t:planeswalker`. Mox Opal's one ruling
   reads its restriction as the activation begins, on the board then: with
   a second Mox Opal entering as the third artifact, neither can be
   activated until the legend rule has put one into the graveyard. That is
   CR 602.5's "can't begin to activate", the check `can_begin_to_activate`
   makes for `OnlyAsSorcery` today, and the one a mana ability's restriction
   needs at the mana window too.

   `backlog.md` §2.8 owns what the restriction grows into (a count, a step,
   a `Condition`, a functioning zone); this decision gives it one home.

   - (a) **`AbilityType::Activated { restriction }` and `AbilityType::Mana {
     restriction }`**, CV-2a's decision widened to the mana kind. *Code*: the
     54 reads of the two variants (31 `Activated`, 23 `Mana`) become
     patterns; the field leaves the 206 sites. *Performance*: none.
     *Upkeep*: `AbilityType` stays `Copy` only while the restriction is, and
     §2.8's `Condition` ends that; `costs` stays on every ability.
   - (b) **`AbilityType::Activated(Activation)` and `Mana(Activation)`, with
     `Activation { costs, restriction }`**: CR 602.1's activation as one
     value. *Code*: (a)'s reads, and the 20 reads of `.costs` (in 10 files)
     read `ability.activation()`, an `Option<&Activation>`; `AbilityType`
     stops being `Copy` now, so a read that copies it out borrows instead.
     *Upkeep*: an ability that is not activated cannot hold a cost or a
     restriction, and §2.8's growth and 602.1b's "any player may activate"
     have one home.
   - (c) **Both stay on `AbilityDef`, and the ability words write the
     defaults.** *Code*: none past decision 1. *Upkeep*: the type still says
     a static ability has costs. Nothing reads them, so nothing is wrong
     today. What (a) and (b) buy over (c) is a type that tells the truth and
     one home for §2.8. #297 showed that the first alone does not pay for a
     sweep, so the second is the payoff.

   **Recommended: (b).** The words hide the defaults under every option, so
   the call sites are the same; the difference is the type every later
   activated card is written against, and §2.8's first card lands in one
   place: the conditional family alone is 223 cards, 60 of them mana
   abilities, which (a) as CV-2a framed it would have left out.

3. **The fields a literal writes and something else overwrites.**

   No printed card reaches these: an ability's id, CR 601.2c's instances of
   "target" read off its effect, and a registry row's id are identities the
   engine assigns. Three values are written at every construction site and
   replaced before anything reads them:

   - `AbilityDef.instances`, written `Vec::new()` 208 times and filled by
     `CardDataBuilder::build`. A def built outside the builder keeps the
     empty list. That is right for every such def today and, as the field's
     doc says, silently wrong for a future one with a target.
   - `AbilityDef.id`, `AbilityId::UNASSIGNED` in every card literal, which
     the builder stamps. Tests set real ids on purpose (`new_ability_id()`,
     33 sites), so the field stays writable.
   - The row id, `id: 0` on 42 lines, overwritten by
     `DurationRegistry::add`: the continuous-effect rows (35 literals and 6
     functions returning one, item 7d, some of them inheriting the id
     through `..test_support::registered(..)`), and 18 replacement and
     restriction rows.

   **3a. The ability.**

   - (a) **`instances` private to `objects::card_data`**, read through
     `instances()`, and every ability built by one constructor on the type,
     `AbilityDef::new(kind, text, effect)`, with `.with_id(id)` for a site
     that sets one. Decision 1's ability words call it. *Code*: the 7
     readers of `.instances`; every literal outside `card_data.rs` must go,
     121 card, 24 engine and 67 test, and the compiler lists each one, so
     the migration is checked, not judged; three struct-update sites
     (`..spell(..)`) become calls. *Upkeep*: no literal can write a stamped
     field again.
   - (b) **Both fields stay public**, the words writing the defaults.
     *Code*: the card literals only. *Upkeep*: engine and test literals stay
     legal, and the hazard the field's doc names stays.

   **Recommended: (a).**

   **3b. The registry row.**

   - (a) **`ContinuousEffectDraft`, as item 7d planned**: the row's fields
     less `id`, consumed by `add`, which builds the row. *Code*: a second
     struct of nine fields and its conversion; the 35 literals and 6
     functions retyped; `add`'s signature. The replacement and restriction
     rows keep `id: 0`. *Upkeep*: two field lists, kept in step by the
     conversion's exhaustive destructure.
   - (b) **The id leaves the row in all three registries.**
     `DurationRegistry` stores `Registered<T> { id, row: T }` and hands that
     out, dereferencing to the row so `row.layer` reads as it does today;
     `DurationRow` loses `id` and `set_id`. *Code*: the 42 `id: 0` lines go,
     plus the registry's internals and the signatures that take a row and
     read its id (`EffectId` is named in 25 places, `ReplacementEffectId` in
     6). *Upkeep*: one place stamps an id, and no row can be built with a
     wrong one. The cost is a wrapper with `Deref`: one more indirection
     a reader meets.
   - (c) **`EffectId::UNASSIGNED`, as `AbilityId` has**, which `add`
     asserts. *Code*: 42 lines. *Upkeep*: the meaningless field stays,
     named.

   **Recommended: (b).** Item 7d's draft fixes one registry of the three, by
   a second copy of its fields.

4. **The per-set layout, the fixtures, and how a test names a card.**

   | What the layout must hold | Count | Example | Holds today |
   |---|---:|---|---|
   | Printings, filed by their first paper set that is no promo, memorabilia or box set | 214 in 91 sets, 50 of them holding one card | `lea`: 38 | 58 files named for engine phases or card types |
   | A set whose Scryfall code begins with a digit, which a Rust module name cannot | 15 of Scryfall's 1,004 paper sets; in the tree, `5dn` alone, with 5 cards | Fifth Dawn, `5dn`; Tenth Edition, `10e` | — |
   | A registered token, dealt as a land card | 1 | Everywhere (`tdsk`) | `dual_lands` |
   | A token a printed card creates (CR 111.1) | 6 | `saproling_token` | beside its card |
   | A registered fixture | 1 | Loyalty Probe | `phase_rd_cards` |
   | An unregistered card function | 51, 13 of them under 9 printed cards' names | `flight_clause`; five variants named Altar's Reap, `dual_land_ub` named Underground Sea | 11 phase files |
   | …of which the fixtures pool deals | 5 | `fixture_curtain` | `phase_history_and_frame_records_cards::CASTABLE` |
   | A keyword's ability word (CR 702.6a) | 1 | `equip`, which three equipment call | `phase_lh_cards`; moves to `authoring` beside `persist` |

   `classify_cards.py --first-printings --list`; the sets by
   `api.scryfall.com/sets`, less the digital ones; the names by
   `/cards/collection` (2026-10-31). **The script's key was wrong, and this
   note fixes it.** It took the earliest printing of any kind, so a
   prerelease promo dated days before its set (`pdka` before `dka`), an
   Arena League or FNM reprint, or a boxed reprint collection that came
   first (Salvat 2005 before Ravnica) filed six cards under a set that is
   not theirs, and its search missed the one token. It now takes the
   earliest paper printing outside a promo, memorabilia or box set, a
   promo-flagged card inside its own set staying there (Nexus of Fate,
   `m19`); falls back to any paper printing for a card printed nowhere
   else; and finds a token among the extras (Everywhere, `tdsk`). It also
   stopped counting `random_deck`'s two functions as fixtures. A card the
   rule files under a product older than its expansion stays there, as the
   rule says: Goblin Electromancer in `ddj`, which printed it a month before
   Return to Ravnica.

   **What builds as item 10 planned, with no choice in it**: one file per
   set under `cards/sets/`, named by its Scryfall code, which is what the
   import pipeline will emit; a code that begins with a digit takes the
   prefix `set_` (`set_5dn.rs`), one rule; `cards::fixtures`, one file per
   architecture doc whose path the fixture exists to test (`layers`,
   `replacement`, `cant_effects`, `copy`, `cost`, `triggers`, `mana`),
   since a fixture's reason is a path and not a printing; a token beside the
   card that creates it, and a registered token in its own token set
   (`tdsk`); registration staying in `registry.rs`, whose comments say why
   each card is pooled, with each path updated. The phase files' module
   docs do not move: what one says about a card goes to that card's doc
   comment, and the rest is the engine phase's history, which the
   architecture docs hold.

   The genuine choice is **how a test or the dev GUI names a card**:

   - (a) **By its set**: `use mtgsim::cards::sets::lea::lightning_bolt`.
     *Code*: the 447 paths and the dev GUI's 14 each name the set.
     *Upkeep*: whoever writes a test looks up the card's first printing.
   - (b) **Through one flat module of printings and one of fixtures**:
     `use mtgsim::cards::printings::lightning_bolt`, where `printings.rs`
     re-exports every set (`pub use super::sets::lea::*;`, one line a set).
     *Code*: the same 447 paths, each naming the card and not its set, and
     ~100 re-export lines. *Upkeep*: a test reads what it uses, and a
     printing or fixture is visible at every import, which is item 10's
     reason for the split. Two sets defining one function name would be an
     ambiguity error where the name is used, which is the error wanted.

   **Recommended: (b).** Filing serves change (item 10), and a test is not
   where a set changes.

5. **The decklist: its format, where it lives, and its first reader.**

   | What a decklist must express | Cards | Example | Holds today |
   |---|---:|---|---|
   | Named cards with counts (CR 100.2a) | every deck | 30 Forest | `Decklist = Vec<Arc<CardData>>` (`state/game.rs`), built in code by `cli_play::build_test_deck` and `random_deck` |
   | One commander, "an attribute of the card itself" (CR 903.3) | 3,761 cards can be one | each `tdc` deck | nothing: `game-formats` adds the designation |
   | Two commanders: partner, friends forever, choose a Background, Doctor's companion (CR 702.124) | 129 + 7 + 32 + 28, and the Backgrounds | Tymna and Thrasios | nothing |
   | A companion, revealed from outside the game (CR 702.139a, 103.2b) | 10 | Lurrus of the Dream-Den | nothing |
   | A sideboard (CR 100.4a) | constructed play | — | nothing |

   Queries: `is:commander`, `kw:partner`, `o:"friends forever"`,
   `o:"choose a background"`, `o:"doctor's companion"`, `kw:companion`.

   **The format:**

   - (a) **The deck sites' text**: a `count name` line per card under
     `Commander`, `Companion`, `Deck` and `Sideboard` headers, with `#`
     comments. It is what Scryfall, Moxfield and Archidekt export and Arena
     imports, so a printed product's list is pasted from its source and
     diffed against it. *Code*: a hand-written parser (no new dependency,
     as `setup-architecture.md` chose for the scenario format), ~60 lines
     and its tests.
   - (b) **The scenario grammar's card lines**: `commander: Eshki
     Dragonclaw`, `deck: Forest | x30`. One grammar for every file the
     engine reads, and a save could embed a decklist as it embeds a scenario.
     A list from a deck site is rewritten line by line.

   **Where it lives** whichever the format: `setup-architecture.md` gains a
   section, since CR 103.3 turns the decks into libraries. The parser goes in
   `mtgsim::scenario`, game construction beside `Game::new`, and the files in
   `mtgsim/decks/`. The parsed file is `DecklistText`, sections of named
   entries; `DecklistText::deck(&registry)` resolves the main deck into
   today's `Decklist`, refusing an unknown name with its line. The commander
   section is parsed and held, and `game-formats` reads it when it puts its
   designation on `Decklist` (its Slotted line, "Before Commander" item 2).

   **The first reader:**

   - (a) **`cli_play`'s deck**: `build_test_deck`'s forty cards become
     `mtgsim/decks/cli-play.txt`, in the same order, and a test loads every
     file in `mtgsim/decks/` against the registry. The five `tdc` lists
     arrive with step 13's gap census, the first reader that wants the names
     the registry lacks.
   - (b) **The five `tdc` lists now**, with a test that resolves each and
     reports the unregistered names. Until step 17 registers the cards, that
     test either is ignored or ratchets a count every later row changes.
   - (c) **`fuzz_games --decks`**, one decklist per seat, which is v0.5's
     table over the CLI. It plays nothing until the decks' cards exist, so it
     comes with step 17.

   **Recommended: format (a), first reader (a).**

6. **Items 204 and 207: commits in the move's PR, or a PR of their own.**

   No printed family: both are fixes to registered cards' data. Item 204
   renames two pooled fixtures (Cerulean Wisps leaves a tapped creature
   tapped, Moonlace's text says more than it does) and registers the real
   cards only if written whole. Item 207 gives fourteen creatures their
   printed types. Each changes what both pools deal, so each owes a test
   shown failing on the tree before it and a re-record of both tables.

   **Item 204 is a family, not two cards.** Decision 4's census found 13
   unregistered card functions under 9 printed cards' names: five Altar's
   Reap variants, the four `*_spell` stand-ins (Liquimetal Torque, Ensoul
   Artifact, Call to Serve, On Serra's Wings), Underground Sea, Inside Out,
   Bull Rush and Zhalfirin Shapecraft. `engineering-practices.md#cards-and-pools`
   forbids a fixture that wears a real name while behaving differently.
   PR 4 reads each against its card. One that differs takes an invented
   name, which changes no game since none is dealt. One that matches is a
   printing, filed in its set and, if the engine can play it, registered
   with its rulings read, which is a pool change in the same re-record.

   - (a) **Their own commits at the end of PR 3.** *Close-out*: three arms
     (main, the move's last commit, the fixes): the move's rows identical,
     the fixes' rows moved and re-recorded. *Upkeep*: one PR whose
     identical-or-not answer depends on which commit is asked.
   - (b) **One PR of their own, right after the move**, ~150 lines with the
     tests, more for each of the 13 that turns out a printing. *Close-out*: PR 3's two arms read identical, and PR 4 re-records
     once for both. Its edits land in the files they stay in: 204's
     fixtures are born in `cards::fixtures`, and the fourteen are edited in
     their set files.
   - (c) **Before PR 1**, so every cleanup PR is measured on fixed data.
     204's fixtures are then written in a phase file and moved.

   **Recommended: (b).** Behavior goes in its own small PR (#297's lesson),
   and it lands after the move so it is written once.

7. **How the move counts against the band.**

   Filing 19,030 lines of card files by first printing deletes each line
   from its phase file and adds it to a set file, so `git diff --numstat`
   reads about 19,000 a side: eight bands, though no line in a moved body
   changes.

   - (a) **A moved line counts zero, and the PR shows the move is pure.**
     A throwaway script compares each card function's text in the new tree
     with its text at the base, byte for byte, and its output goes in the PR
     body. The equality test reads every card equal, and `git diff
     --color-moved` shows the rest. The band reads what is not a move
     (module files, imports, registry paths, docs, the decklist, and the
     module-doc text that becomes card doc comments), and reports the moved
     count beside it, as docs are.
   - (b) **A moved line counts**, so the move splits into about eight PRs by
     set group. Each is a pure move, which no reviewer reads line by line
     either.

   **Recommended: (a).** The review a moved line needs is the proof that it
   moved, not a reading of it.

## What builds without a decision

- **The equality test** (PR 1, deleted at PR 3's close-out). It
  fingerprints every card function, the 215 registered and the fixtures,
  by a canonical `{:#?}`: each `HashSet`'s entries sorted, since `CardData`'s
  sets iterate in a per-process order. It compares the fingerprints with a
  file the base wrote (`UPDATE_CARD_FINGERPRINTS=1` rewrites it), and a
  mismatch names the card. **The order of commits is what makes it a
  proof**: every literal is rewritten into a word while the types stay
  fixed, so the file never changes; then decisions 2 and 3 change the types
  behind the words, in a commit that touches no card file and regenerates
  the file. A Deferred Migrations line carries it until it goes.
- `classify_cards.py`'s key (decision 4), in this note's commit.
- The engine-phase card files' doc comments that name a card's role
  (`registry.rs`'s and the modules') follow each card, and `CLAUDE.md`'s
  conventions line, "Test cards in `src/cards/phase_XX_cards.rs`", becomes
  the set files and `cards::fixtures` (PR 3).
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

- Item 10, item 7d and item 215's card half close with PR 3; items 204 and
  207 with PR 4; item 209 with PR 5. Item 204's body gains the 13
  unregistered functions in this note's commit, so its **Sized:** line is
  the family's.
- `backlog.md` §2.8 points at `Activation` (decision 2) as the restriction's
  home.
- `setup-architecture.md` gains the decklist's section (PR 3).
- `codebase-state.md` "Before Commander" item 2 names `DecklistText`'s
  commander section as what `game-formats` reads.
