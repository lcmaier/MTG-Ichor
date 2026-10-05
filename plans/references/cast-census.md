# How cards are cast, activated and paid for — the census

> **Status:** run 2026-10-05 against `0481f00` (#222's merge), for
> `codebase-state.md` item 212, which this closes (#223). **Authority:** every count
> is `plans/references/cast-census.py`'s, spliced in between this file's
> markers by `--write`. The statuses and owners are a reading of the tree and
> the docs by hand. They live in the script's tables and change only there.
> On what exists, `codebase-state.md` wins. **Re-run:** fetching the corpus
> takes about 7 minutes (184 pages), the counts about a minute, and the
> classification 2 seconds.

## 0. The question, and how to read the tables

`cr-coverage-audit.md` asks whether the plan can express the CR. This
census asks the same question from the card side, for one area: every way a
Commander-legal card is cast, played, activated or paid for. Each family maps
to the engine surface that would have to express it, with a status:

- **built**: the engine plays it;
- **owned and slotted**: a document names the missing surface and the route
  slots it;
- **unowned**: neither. Every family found unowned leaves this census
  with an owner and a slot, or an exclusion on the record (§1).

The corpus:

<!-- cast-census: begin CORPUS -->
`legal:commander -t:token game:paper -is:funny`: 32,115 cards (`total_cards` 32,115).
<!-- cast-census: end CORPUS -->

Four tables partition a unit, so their columns sum:

- a **cost piece**: one piece of an activated ability's cost, of an "As an
  additional cost to cast this spell" clause, or of a "rather than pay this
  spell's mana cost" clause (§3);
- an **activated ability**: a "[cost]: [effect]" line, printed or quoted in
  a grant (§4);
- a **mana ability**, by what it produces (§5);
- a **clause**: one sentence that grants or denies a way to cast or play (§6).

Keyword families (§7) and mana symbols (§8) count cards. Reminder text is
removed before anything is read, and a card's own name is written `~`.

**Every number sits beside its query.** A partition's number is its bucket's
pattern over the cached corpus, and each table lists its patterns. A card
count is Scryfall's `total_cards`, with the query in the row. `--residual` is
empty at this run: every cost piece and every mana ability lands in a bucket.

<!-- cast-census: begin DATE -->
Counted 2026-10-05: the corpus fetched 2026-10-05, the counts cache's newest entry 2026-10-05, the tree as of the run.
<!-- cast-census: end DATE -->

## 1. The families by status, and what changed in ownership

<!-- cast-census: begin SUMMARY -->
| table | built | owned and slotted | found unowned | ... given an owner here | ... excluded here | still unowned |
|---|---:|---:|---:|---:|---:|---:|
| activation costs | 6 | 3 | 41 | 40 | 1 | 0 |
| who activates | 1 | 0 | 2 | 2 | 0 | 0 |
| when | 3 | 0 | 4 | 4 | 0 | 0 |
| where | 1 | 0 | 3 | 3 | 0 | 0 |
| mana production | 5 | 2 | 4 | 4 | 0 | 0 |
| what feeds a mana ability | 3 | 0 | 0 | 0 | 0 | 0 |
| casting and playing | 0 | 3 | 7 | 7 | 0 | 0 |
| keyword families | 3 | 6 | 14 | 14 | 0 | 0 |
| mana symbols | 2 | 6 | 1 | 1 | 0 | 0 |
| mana by phrase | 1 | 5 | 1 | 1 | 0 | 0 |
| **all** | **25** | **25** | **77** | **76** | **1** | **0** |
<!-- cast-census: end SUMMARY -->

The owner settled the homes on 2026-10-05, at this census's review. Each
assignment is written where it lives:

- **The permission half → `permission-architecture.md`, a new document**
  (`roadmap-v2.md` B11). It is designed beside RS-2, so each check's reasons
  are designed once, and built before B2, since a commander is cast from the
  command zone. It graduates `backlog.md` §2.3, §2.8 and §2.11, and "Before
  card breadth" item 2. It is the first consumer of §2.15's effective-value
  query, through the land count `can_play_land` reads. The permission half is CR 601.3's
  "allows", 602.2 (who may activate), 602.5 (when), 113.6 (from which zone),
  305.1–2 (land play) and 606.3 (loyalty). Its families: casting from a
  graveyard, from exile, from a library's top or from the command zone (3,497
  possible commanders); casting while something resolves; casting a card the
  caster does not own; abilities that work in a hand or a graveyard;
  activation limits; who may activate; loyalty abilities (311 planeswalkers);
  and land play from other zones.
- **The cost actions → `cost-architecture.md` CP-2, a new row**
  (`roadmap-v2.md` B1, before C). It takes the four arms that are neither
  checked nor paid, with their payment prompts, and the payment of a loyalty
  cost. It also takes the shapes the arms need: an amount that can be X, a
  choice between two costs, tapping or untapping other permanents, counters on
  another permanent, and the count a multikicker announces. The single-arm
  actions (energy, behold, reveal, return to hand, mill, collect evidence,
  forage, waterbend, unattach...) each get one arm with their first card, in
  C. `cr-coverage-audit.md` §4 ruled a `Cost` arm additive.
- **Mana production with no owner → `codebase-state.md` item 162's design**,
  the next PR: one mana of any color, Treasure makers, painland riders, "the
  chosen color" and "could produce". §12 lists what that design inherits.
- **Two payment symbols → CP-1**: the `{C/W}`-style hybrid that CR 107.4e
  added, and an activated ability's `{X}`, which the engine never asks for.
- **The question kinds → SU-8 and each kind's owner** (§9).
- **Breadth → C, Phase 8.** These are a card over surfaces the row names: the
  keyword alternative and additional costs whose arm exists or is `Custom`,
  exert, a die roll as a cost, CR 701.9b's third chooser, an opponent choosing
  targets (CR 601.6), and the Defilers' additional cost (a `CostChange` arm).
- **Excluded: ticket counters**, 22 cards. They come from Attractions and
  stickers, which `roadmap-v2.md` §7 puts out of scope.

## 2. The rules pass — what each surface must express at v1

`engineering-practices.md` §8's habit, with the question item 212 adds. For
each surface: the rule that owns it, the rules that watch it, and what it
must express at v1. Read in CR 601.2, 602, 118, 106, 605, 113.6, 116 and 305,
with 107 and 702 where those send.

- **`Cost`** (CR 118, 601.2f–h, 602.2b, 701). A cost is "an action or
  payment" (118.1), and 601.2f's list ends "and so on". At v1, `Cost` must
  express every action a cost names:
  - with its amount, a number or X (107.3a);
  - as a choice between two costs ("pay {4} or sacrifice");
  - with each payment's choice (which card, which permanent, which counter)
    asked before any payment is performed (601.2h; `cost-architecture.md`
    §3.12).

  **Watching:** 118.3 (the resources); 614.17b, under which a "can't" makes a
  cost unpayable (RS-4); 118.11, under which a cost whose action is replaced
  is still paid; 118.10 (one payment per cost); 601.2h's two groups; 732.1.
- **`AbilityDef`'s activation fields** (CR 602). At v1 they must express:
  - who may activate (602.2, "unless the object specifically says
    otherwise");
  - when (602.5: as a sorcery, as an instant, once each turn, during a step,
    if a condition holds);
  - from which zone (113.6b, j, m);
  - whether it is a mana ability (605.1a).

  602.5b keeps a use limit through a change of control. 602.5c keeps an
  acquired ability's limit to that instance. **Watching:** 602.5a (summoning
  sickness); 605.3a (mana abilities at a priority question, item 211); 606.3
  (a loyalty ability: main phase, empty stack, once a turn per permanent,
  whoever activated it); 113.6; 613.1's layers, which grant and remove abilities, so
  every field is read off the effective list.
- **Casting and land play** (CR 601.3, 305.1). At v1 these must express a
  permission per player, object and zone, granted by a rule or an effect:
  - the hand by default (601.2: "usually the hand");
  - the command zone, for a commander its player owns (903.8);
  - a graveyard (flashback, 702.34a, and its siblings);
  - exile, after a special action (116.2f, h, k) or during a resolution
    (608.2g: cascade, discover);
  - a library's top;
  - and another player's cards, so ownership belongs inside the permission,
    not in front of it.

  A permission may carry a cost (118.9) and a consequence: flashback exiles
  the card wherever it would go. **Watching:** 601.3's prohibition half
  (RS-2); 601.3a–f (choices that change a quality; conditional flash;
  face-down cards in exile); 601.5; 117.1a's timing; 305.2b and 305.3 ("for
  any reason"); 113.6e–f, under which the ability that permits it functions
  in the zone it permits from.
- **Mana payment** (CR 106, 107.4, 601.2g–h, 605). At v1 it must express:
  - every symbol in 107.4, including the colorless hybrids `{C/W}`;
  - every production 106 and 605 name: "any color" chosen at resolution,
    amounts, several mana at once, "could produce" (106.7), restricted
    (106.6), not emptying (106.4), triggered (605.1b), replaced (106.12b);
  - every payment method: convoke (702.51a), improvise (702.126a), delve
    (702.66a), emerge (702.119a), assist (702.132a), offering (702.48a), and
    "as though it were mana of any color" (609.4b).

  **Watching:** 601.2g and 605.3a (the window, and mana abilities at a
  priority question); 106.4; 601.2f's lock-in; 732.1; 400.7d (what paid,
  item 30).
- **The question kinds** (`ChoiceKind`). At v1, the why must say for every
  question what it ranges over, and why a candidate was dropped, in the CR
  rule that drops it. The rules: 115 and 601.2c for a target, 701.21a for a
  sacrifice, 614.13a for an auxiliary move, 707.6 for a copy's choice, 800.4a
  for a player who left, and 510.1c and 702.19b for a damage bound.

### 2.1 The rules a card may override

CR 101.1 first: "Whenever a card's text directly contradicts these rules,
the card takes precedence." Then every rule in the area that names its own
exception ("unless the object specifically says otherwise", "usually",
"normally", "by default"), with the card families that override it:

<!-- cast-census: begin OVERRIDES -->
| CR | the default | what a card says instead | cards | query (plus `BASE`) | surface | status | owner |
|---|---|---|---:|---|---|---|---|
| 601.2 | a spell is cast from its caster's hand ("usually the hand") | a permission names another zone | 485 | `o:/(cast\|play)[^.]* from (your\|a\|their) (graveyard\|exile)\|from the top of your library/` | `can_begin_to_cast` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) |
| 601.3 | no player may cast a card that a rule or effect does not permit | "you may cast" a card you don't own | 16 | `(o:"you don't own" or o:"opponents own")` | `can_begin_to_cast` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) |
| 117.1a | a noninstant spell waits for a main phase and an empty stack | flash | 608 | `kw:flash` | `is_instant_or_has_flash` | built |  |
| 601.3b | a spell without flash is cast at sorcery timing | "as though it had flash" | 86 | `(o:"as though it had flash" or o:"as though they had flash")` | a permission (CR 609.4) | owned | `backlog.md` §2.24, "as though" (`roadmap-v2.md` B8, before C) |
| 305.1 | a land is played from its owner's hand | play lands from a graveyard, a library's top or exile | 52 | `o:/play (lands\|land cards)[^.]* from/` | `can_play_land` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) |
| 305.2 | one land a turn | play additional lands | 39 | `o:"additional land"` | `lands_per_turn` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) |
| 602.2 | only an object's controller activates its abilities ("unless the object specifically says otherwise") | any player, or only an opponent, may activate | 44 | `(o:"any player may activate" or o:"opponents may activate")` | `can_activate_its_abilities` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) |
| 602.5 | an ability may be activated any time its controller has priority | "Activate only ..." | 964 | `o:"activate only"` | `ActivationRestriction`, `OnlyAsSorcery` alone | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) |
| 602.5a | a creature's {T} or {Q} ability waits out summoning sickness | haste, or "as though it had haste" | 642 | `(kw:haste or o:"as though it had haste" or o:"as though they had haste")` | `has_summoning_sickness` | built |  |
| 113.6 | a permanent's abilities work on the battlefield and a spell's on the stack | an ability that names its zone, or whose cost moves its object | 372 | `o:/~ from your (graveyard\|hand)/` | `CannotActivate::NotOnBattlefield` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) |
| 106.4 | mana empties between steps and phases | mana that does not empty | 24 | `(o:"don't lose this mana" or o:"doesn't empty" or o:"don't empty")` | `ManaPersistence` | owned | `codebase-state.md` item 33, mana provenance (`roadmap-v2.md` B9, before C) |
| 107.4a | colored mana pays only its own color | spend mana as though it were any color | 50 | `o:"as though it were mana of any"` | `ManaPool::pay` | owned | `backlog.md` §2.24, "as though" (`roadmap-v2.md` B8, before C) |
| 106.6 | mana is spent on anything | "spend this mana only" | 165 | `o:"spend this mana only"` | `ManaRestriction` | owned | `codebase-state.md` item 33, mana provenance (`roadmap-v2.md` B9, before C) |
| 118.9 | a spell's mana cost is paid | a printed alternative cost ("rather than pay") | 155 | `o:"rather than pay"` | `AlternativeCost::Custom`; its pieces are §3's | built |  |
| 118.9 | a spell's mana cost is paid | an effect's "without paying its mana cost" | 316 | `o:"without paying"` | a permission's cost | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) |
| 118.8 | only the costs a spell prints | an additional cost another object imposes (the Defilers) | 7 | `o:"additional cost" -o:"as an additional cost to cast this spell"` | `CostChange` has no such arm | **owned** (found unowned) | `cost-architecture.md` §6: one `CostChange` arm with its first card, in C |
| 601.6 | the caster chooses the targets | "an opponent chooses target" | 2 | `o:"opponent chooses target"` | `ask_select_recipients`'s player | owned | `roadmap-v2.md` C, Phase 8 breadth: a card is a normal diff over surfaces named in the row |
| 701.9b | the discarding player chooses | "at random", or another player chooses | 84 | `o:/discard[^.]* at random\|chooses a card[^.]* discards/` | `DiscardChooser` | **owned** (found unowned) | `roadmap-v2.md` C, Phase 8 breadth: a card is a normal diff over surfaces named in the row |
<!-- cast-census: end OVERRIDES -->

Four rules in the area deny an override in their own words, and a surface may
encode them as fixed: 305.2b and 305.3 ("for any reason"), 601.2b ("can't
apply two alternative methods of casting or two alternative costs"), and
118.6's unpayable cost (no mana cost), which only an alternative cost gets
past (118.6a).

## 3. Costs — `Cost`'s arms

<!-- cast-census: begin COSTS -->
| family | CR | surface | status | owner | pieces | cards | in a spell's cost |
|---|---|---|---|---|---:|---:|---:|
| a choice between two costs ("pay {4} or sacrifice") | 118.1 | no arm: a cost is one action | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 52 | 52 | 46 |
| a cost paid any number of times | 601.2b | 601.2b's announcement cannot ask for a number | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 2 | 2 | 2 |
| {T} | 107.5 | `Cost::TapSelf` | built |  | 5,371 | 4,275 | 0 |
| {Q} | 107.6 | `Cost::UntapSelf` | built |  | 18 | 18 | 0 |
| a loyalty symbol, [+N] [−N] [0] | 606.4 | no loyalty ability | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 882 | 311 | 0 |
| ticket counters, {TK} | 107.17a | no arm: a player's counters | **excluded** (found unowned) | `roadmap-v2.md` §7, the excluded tail | 22 | 22 | 0 |
| mana with a Phyrexian symbol | 107.4f | `ManaSymbol::Phyrexian`, unpaid | owned | `cost-architecture.md` CP-1 (`roadmap-v2.md` B1, any time) | 38 | 38 | 0 |
| mana with a hybrid or {2/W} symbol | 107.4e | `ManaSymbol::Hybrid`, `MonoHybrid`, unpaid | owned | `cost-architecture.md` CP-1 (`roadmap-v2.md` B1, any time) | 121 | 110 | 0 |
| mana with {S} | 107.4h | `ManaSymbol::Snow`, unpaid | owned | `codebase-state.md` item 33, mana provenance (`roadmap-v2.md` B9, before C) | 37 | 35 | 0 |
| mana with {X} | 107.3a | `Cost::Mana`; an ability's X is never asked | **owned** (found unowned) | `cost-architecture.md` CP-1 (`roadmap-v2.md` B1, any time) | 148 | 145 | 0 |
| mana | 601.2g | `Cost::Mana` | built |  | 5,636 | 5,098 | 20 |
| an amount of mana the board decides | 107.3 | `Cost::Mana(ManaCost)` holds symbols | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 2 | 2 | 0 |
| pay {E} | 107.14 | no arm: a player's counters | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 58 | 52 | 0 |
| pay N life | 119.4 | `Cost::PayLife` | built |  | 159 | 155 | 5 |
| pay X life, half your life | 119.4 | `Cost::PayLife(u64)` holds a number | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 12 | 12 | 7 |
| sacrifice ~ | 701.21a | `Cost::SacrificeSelf` | built |  | 1,150 | 1,118 | 0 |
| sacrifice X, any number or half | 701.21a | `Cost::Sacrifice(_, u32)` holds a number | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 29 | 29 | 15 |
| sacrifice the enchanted or equipped permanent | 701.21a | no `ObjectFilter` for it | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 3 | 3 | 0 |
| sacrifice N permanents | 701.21a | `Cost::Sacrifice` | built |  | 841 | 813 | 122 |
| discard ~ | 113.6m | no arm, and the ability works in a hand | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 73 | 72 | 0 |
| discard your hand | 701.9a | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 12 | 12 | 1 |
| discard N cards | 701.9a | `Cost::Discard`, unchecked | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 348 | 342 | 42 |
| exile ~ from your graveyard | 113.6j | no arm, and the ability works in a graveyard | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 79 | 79 | 0 |
| exile N cards from a graveyard | 701.13 | `Cost::ExileFromGraveyard`, unchecked | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 105 | 105 | 19 |
| exile cards from your hand | 701.13 | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 37 | 37 | 26 |
| exile cards from the top of your library | 701.13 | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 8 | 7 | 0 |
| exile ~ | 701.13 | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 59 | 59 | 0 |
| exile a permanent you control | 701.13 | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 14 | 14 | 3 |
| put an exiled card into a graveyard (processors) | 406.3 | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 7 | 7 | 1 |
| remove N counters from ~ | 122.1 | `Cost::RemoveCounters`, unchecked | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 249 | 235 | 0 |
| remove X or any number of counters | 122.1 | `Cost::RemoveCounters(_, u32)` holds a number | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 56 | 56 | 0 |
| remove counters from another permanent | 122.1 | no arm: the arm names the source | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 46 | 45 | 0 |
| put N counters on ~ | 122.1 | `Cost::AddCounters`, unchecked | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 11 | 10 | 0 |
| put counters on another permanent, blight | 701.68 | no arm: the arm names the source | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 16 | 15 | 7 |
| tap N untapped permanents you control | 701.26a | no arm | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 201 | 191 | 12 |
| untap N tapped permanents | 701.26b | no arm | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 5 | 3 | 0 |
| return ~ to its owner's hand | 118.1 | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 13 | 13 | 0 |
| return a permanent you control to its owner's hand | 118.1 | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 44 | 44 | 16 |
| behold a [quality] | 701.4a | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 12 | 12 | 12 |
| reveal cards from your hand | 701.20a | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 35 | 35 | 12 |
| put cards on top or bottom of a library | 118.1 | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 5 | 5 | 0 |
| mill N | 701.17 | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 6 | 6 | 0 |
| exert ~ | 701.43 | no arm | **owned** (found unowned) | `roadmap-v2.md` C, Phase 8 breadth: a card is a normal diff over surfaces named in the row | 8 | 8 | 0 |
| collect evidence N | 701.59 | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 15 | 14 | 8 |
| forage | 701.61 | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 2 | 2 | 0 |
| waterbend {N} | 701.67 | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 25 | 25 | 9 |
| unattach an Equipment | 701.3d | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 9 | 9 | 0 |
| roll a die | 706.1 | no arm | **owned** (found unowned) | `roadmap-v2.md` C, Phase 8 breadth: a card is a normal diff over surfaces named in the row | 1 | 1 | 0 |
| a choice announced as a cost (a type, a number, a keyword) | 601.2b | no arm | **owned** (found unowned) | `cost-architecture.md` §6: one `Cost` arm with its first card, in C | 4 | 4 | 3 |
| **all pieces** | | | | | **16,086** | | |

Each row's pattern, matched against the lowercased piece, in row order:

- a choice between two costs ("pay {4} or sacrifice"): `\bor (pay\|exile\|sacrifice\|discard\|return\|tap\|untap\|reveal\|remove\|put\|collect evidence\|blight\|behold\|waterbend\|forage\|mill\|exert\|unattach\|roll\|choose)\b\|\} or \{`
- a cost paid any number of times: `any number of times`
- {T}: `^\{t\}$`
- {Q}: `^\{q\}$`
- a loyalty symbol, [+N] [−N] [0]: `^[+−-]?(\d+\|x)$`
- ticket counters, {TK}: `^(\{tk\})+$`
- mana with a Phyrexian symbol: `^(pay )?(\{[^}]+\})*\{[wubrg](/[wubrg])?/p\}`
- mana with a hybrid or {2/W} symbol: `^(pay )?(\{[^}]+\})*\{[wubrg2]/[wubrg]\}`
- mana with {S}: `^(pay )?(\{[^}]+\})*\{s\}`
- mana with {X}: `^(pay )?(\{[^}]+\})*\{x\}`
- mana: `^(pay )?(\{(\d+\|[wubrgc])\})+$`
- an amount of mana the board decides: `^pay \{\d\} for each\|^pay .*mana cost\|^pay \{x\}`
- pay {E}: `^pay .*\{e\}`
- pay N life: `^pay (\d+\|one\|two\|three\|four\|five\|six\|seven\|eight\|ten) life$`
- pay X life, half your life: `^pay .*life`
- sacrifice ~: `^sacrifice ~$`
- sacrifice X, any number or half: `^sacrifice (?:x\|any number of\|one or more\|all\|half\|that many\|cards equal\|up to)`
- sacrifice the enchanted or equipped permanent: `^sacrifice (enchanted\|equipped\|the creature\|the permanent\|it\b\|attached\|[a-z]+$)`
- sacrifice N permanents: `^sacrifice (?:a\|an\|one\|two\|three\|four\|five\|six\|seven\|eight\|nine\|ten\|eleven\|twelve\|thirteen\|fourteen\|fifteen\|twenty\|another\|\d+)`
- discard ~: `^discard ~`
- discard your hand: `^discard your hand`
- discard N cards: `^discard `
- exile ~ from your graveyard: `^exile ~ from your graveyard`
- exile N cards from a graveyard: `^exile .*(from\|of) (your\|a\|a single) graveyard`
- exile cards from your hand: `^exile .*from your hand`
- exile cards from the top of your library: `^exile .*of your library`
- exile ~: `^exile ~$`
- exile a permanent you control: `^exile .*you control`
- put an exiled card into a graveyard (processors): `^put .*(from exile\|exiled with ~) into`
- remove N counters from ~: `^remove (a\|an\|one\|two\|three\|four\|five\|six\|seven\|eight\|nine\|ten\|eleven\|twelve\|thirteen\|fourteen\|fifteen\|twenty\|another\|\d+) .*counters? from ~$`
- remove X or any number of counters: `^remove (x\|any number of\|one or more\|all\|half\|that many\|cards equal\|up to\|one or more) .*counters?`
- remove counters from another permanent: `^remove .*counters? from`
- put N counters on ~: `^put .*counters? on ~$`
- put counters on another permanent, blight: `^put .*counters? on\|^blight`
- tap N untapped permanents you control: `^tap `
- untap N tapped permanents: `^untap `
- return ~ to its owner's hand: `^return ~ to`
- return a permanent you control to its owner's hand: `^return `
- behold a [quality]: `^behold\|^choose a .* you control or\|^reveal .* or choose`
- reveal cards from your hand: `^reveal `
- put cards on top or bottom of a library: `^put .*library`
- mill N: `^mill `
- exert ~: `^exert `
- collect evidence N: `^collect evidence`
- forage: `^forage`
- waterbend {N}: `^waterbend`
- unattach an Equipment: `^unattach`
- roll a die: `^roll `
- a choice announced as a cost (a type, a number, a keyword): `^choose `
<!-- cast-census: end COSTS -->

Ten arms, and the cards ask for about fifty actions. Six arms are built.
**The four that are neither checked nor paid are among the largest families
in the table:**

- discarding cards, 342 cards;
- removing counters from the cost's source, 235;
- exiling from a graveyard, 105;
- putting counters on the source, 10, which loyalty costs also need.

Each piece's "in a spell's cost" count is the part that comes from an
additional or alternative cost, not an activated ability.

## 4. Activated abilities — who, when, where

<!-- cast-census: begin ABILITIES -->
10,955 activated abilities on 8,596 cards, 443 of them quoted in a grant; 2,222 are mana abilities (CR 605.1a). Who and when read the effect's activation instructions (CR 602.1b); where reads the cost and the effect.

| who | CR | surface | status | owner | abilities | cards | pattern |
|---|---|---|---|---|---:|---:|---|
| any player may activate | 602.2 | no `AbilityDef` field | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 42 | 39 | `\bany player may activate` |
| only an opponent may activate | 602.2 | no `AbilityDef` field | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 5 | 5 | `\b(only your opponents\|only an opponent\|any opponent\|each opponent) may activate` |
| its controller (CR 602.2's default) | 602.2 | `can_activate_its_abilities` | built |  | 10,908 | 8,553 | the rest |

| when | CR | surface | status | owner | abilities | cards | pattern |
|---|---|---|---|---|---:|---:|---|
| once a game (exhaust and the like) | 602.5b | no `ActivationRestriction` arm | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 9 | 9 | `^exhaust\b\|activate (this ability \|each exhaust ability )?only once\.\|activate only once\b(?! each)` |
| once or twice each turn | 602.5b | no `ActivationRestriction` arm | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 129 | 127 | `only once each turn\|only twice each turn\|no more than (once\|twice\|\w+ times) each turn\|only (once\|twice) each round` |
| only if a condition holds | 602.5 | no `ActivationRestriction` arm; `Condition` exists | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 218 | 215 | `activate (this ability \|these abilities \|each .{0,30} )?only (if\|while)\b` |
| only during a turn, a step or combat | 602.5 | no `ActivationRestriction` arm | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 133 | 133 | `activate (this ability )?only (during\|before\|any time you could cast an instant during)` |
| only as a sorcery | 602.5d | `ActivationRestriction::OnlyAsSorcery` | built |  | 489 | 482 | `activate (this ability \|each .{0,30} )?only as a sorcery` |
| only as an instant | 602.5e | the default, as `AbilityType::Activated` | built |  | 3 | 3 | `activate (this ability )?only as an instant` |
| any time with priority (the default) | 602.1 | the default | built |  | 9,974 | 7,880 | the rest |

| where | CR | surface | status | owner | abilities | cards | pattern |
|---|---|---|---|---|---:|---:|---|
| from a graveyard | 113.6j | `CannotActivate::NotOnBattlefield` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 200 | 200 | `(exile\|return\|put\|cast\|shuffle) ~ (onto the battlefield )?from your graveyard\|~ is in your graveyard` |
| from a hand | 113.6m | `CannotActivate::NotOnBattlefield` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 96 | 95 | `^discard ~\|discard ~[,:]\|reveal ~ from your hand\|(exile\|put\|return\|cast) ~ (onto the battlefield )?from your hand\|~ is in your hand` |
| from exile | 113.6b | `CannotActivate::NotOnBattlefield` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 2 | 2 | `(put\|return\|cast) ~ (onto the battlefield )?from exile\|activate only if ~ is (exiled\|in exile)` |
| on the battlefield (CR 113.6's default) | 113.6 | the default | built |  | 10,657 | 8,345 | the rest |
<!-- cast-census: end ABILITIES -->

Each axis partitions every ability. An ability with two limits ("only as a
sorcery and only once each turn") lands in whichever comes first in row order.

## 5. Mana abilities — what they produce, and what feeds them

<!-- cast-census: begin MANA -->
| what it produces | CR | surface | plays | owner | abilities | cards | the affordability check (item 162) |
|---|---|---|---|---|---:|---:|---|
| with "spend this mana only" | 106.6 | `ManaRestriction`; `ManaPool::pay` reads the plain pool | owned | `codebase-state.md` item 33, mana provenance (`roadmap-v2.md` B9, before C) | 153 | 151 | counts it as unrestricted |
| mana that does something when spent, or does not empty | 106.6 | `ManaGrant`, `ManaPersistence`; no payment path reads them | owned | `codebase-state.md` item 33, mana provenance (`roadmap-v2.md` B9, before C) | 8 | 8 | counts its mana |
| with a non-mana instruction in the same ability (painlands) | 605.1a | `resolve_mana_effect` resolves `ProduceMana` atoms alone | **owned** (found unowned) | `codebase-state.md` item 162's design, the next PR | 129 | 129 | counts its mana |
| a type another permanent "could produce" | 106.7 | no `ManaOutput` form | **owned** (found unowned) | `codebase-state.md` item 162's design, the next PR | 14 | 14 | skips it |
| the color or type chosen as it entered | 607.2d | no record of the choice | **owned** (found unowned) | `codebase-state.md` item 162's design, the next PR | 32 | 32 | skips it |
| one mana of any color or type | 106.1b | no `ManaOutput` form | **owned** (found unowned) | `codebase-state.md` item 162's design, the next PR | 391 | 380 | skips it |
| doubles the mana already in the pool | 106.4 | `AmountExpr::UnspentMana` | built |  | 1 | 1 | skips it |
| an amount the board decides | 106.1 | `AmountExpr`, where it names the count | built |  | 60 | 60 | skips it: `Fixed` alone |
| one of two or three listed types | 106.1 | one ability per type, as Everywhere is written | built |  | 381 | 380 | counts each type as a source |
| several mana at once ({C}{C}, {W}{U}) | 106.1 | `ManaOutput` | built |  | 156 | 155 | counts one |
| one mana of one type | 106.1 | `Primitive::ProduceMana` | built |  | 897 | 854 | counts it |

Patterns, in row order, against the effect from its first "add" or "choose a color", activation instructions removed (for the cost, against the cost):

- with "spend this mana only": `spend this mana only\|this mana can't be spent`
- mana that does something when spent, or does not empty: `when (you\|that player) spend\|spent this way\|spend this mana\|don't lose this mana\|doesn't empty`
- with a non-mana instruction in the same ability (painlands): `^add [^.]*\.\s+\S`
- a type another permanent "could produce": `could produce`
- the color or type chosen as it entered: `chosen color\|the chosen type\|of the chosen\|noted type\|circled colors\|exiled cards?'s colors`
- one mana of any color or type: `any color\|any one color\|any type\|any combination of colors\|in any combination\|any colors\|of any color\|of that color\|of either of\|any of (the\|its\|~'s)`
- doubles the mana already in the pool: `^double the amount`
- an amount the board decides: `for each\|equal to\|an amount of\|that much\|x mana\|\{x\}`
- one of two or three listed types: `^add \{[wubrgc]\}(, \{[wubrgc]\})*,? or \{[wubrgc]\}\|^choose a color\b`
- several mana at once ({C}{C}, {W}{U}): `^add (\{[wubrgc]\}){2,}\|^add (two\|three) mana`
- one mana of one type: `^add \{[wubrgc]\}\.?$`

| what its cost feeds it | CR | surface | plays | owner | abilities | cards | the affordability check (item 162) |
|---|---|---|---|---|---:|---:|---|
| costs mana (a filter, a Signet) | 605.1a | `Cost::Mana`, paid from the pool | built |  | 175 | 170 | skips it unless the pool already holds the cost |
| costs a sacrifice (Ironworks, a Treasure) | 605.1a | `Cost::Sacrifice` | built |  | 143 | 143 | counts it free, the sacrificed source included |
| costs a tap, life or nothing | 605.1a | `available_mana_sources` | built |  | 1,904 | 1,724 | counts it |

Patterns, in row order, against the effect from its first "add" or "choose a color", activation instructions removed (for the cost, against the cost):

- costs mana (a filter, a Signet): `\{(\d+\|[wubrgcx]\|[wubrg]/[wubrgp])\}`
- costs a sacrifice (Ironworks, a Treasure): `sacrifice`
- costs a tap, life or nothing: the rest
<!-- cast-census: end MANA -->

**"Plays" and the affordability check are different questions**, and the
calibration (§10) is what showed it. Krark-Clan Ironworks and Sol Ring play
correctly. The check misreads both: it counts Ironworks' sacrifice as free and
Sol Ring's `{C}{C}` as one mana. The last column is item 162's
(`oracle/mana_helpers.rs::find_mana_sources`, `available_mana_sources`).

## 6. Casting and playing — permission, ownership, timing

<!-- cast-census: begin CAST -->
| family | CR | surface | status | owner | clauses | cards | Scryfall, cards | query (plus `BASE`) |
|---|---|---|---|---|---:|---:|---:|---|
| cast without paying its mana cost | 118.9 | no permission, and no effect-provided alternative cost | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 315 | 315 | 316 | `o:"without paying"` |
| cast or play a card its caster does not own | 601.3 | `can_begin_to_cast` asks owner = caster | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 29 | 29 | 30 | `(o:"you don't own" or o:"opponents own" or o:"an opponent owns")` |
| cast or play from the top of a library | 601.3 | `CannotCast::NotInHand` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 89 | 88 | 94 | `o:/(cast\|play)[^.]*from the top of/` |
| cast ~ from a graveyard (no keyword) | 601.3 | `CannotCast::NotInHand` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 39 | 39 | 39 | `o:/(cast\|play) ~ from (your\|a) graveyard/` |
| cast or play other cards from a graveyard | 601.3 | `CannotCast::NotInHand` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 246 | 241 | 300 | `o:/(cast\|play)[^.]*from (your\|a\|their) graveyard/` |
| cast or play a card from exile | 601.3 | `CannotCast::NotInHand` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 490 | 450 | 464 | `o:/may (cast\|play) (it\|that card\|those cards\|them\|the exiled card\|cards exiled\|one of\|a card exiled\|spells? from among)\|(cast\|play)[^.]*from exile/` |
| cast as though it had flash | 601.3b | a row granting flash; CR 609.4's permission | owned | `backlog.md` §2.24, "as though" (`roadmap-v2.md` B8, before C) | 91 | 90 | 86 | `(o:"as though it had flash" or o:"as though they had flash")` |
| cast ~ only at a time or if a condition holds | 601.3 | RS-2's `Cast` arm, `SourceOnly` | owned | `cant-effects-architecture.md` RS-2 (beside A, after A6c) | 66 | 66 | 66 | `o:"cast this spell only"` |
| a player can't cast | 601.3 | RS-2's `Cast` arm | owned | `cant-effects-architecture.md` RS-2 (beside A, after A6c) | 101 | 99 | 93 | `o:"can't cast"` |
| play an additional land | 305.2 | `PlayerState::lands_per_turn`, nothing sets it | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 36 | 36 | 39 | `o:"additional land"` |

The clause patterns, in row order, against one sentence with `~` for the card's own name; a sentence lands in the first that matches, so the clause column partitions:

- cast without paying its mana cost: `without paying (its\|their\|that spell's\|~'s) mana costs?`
- cast or play a card its caster does not own: `(you don't own\|your opponents own\|an opponent owns\|cards? (your )?opponents? own)`
- cast or play from the top of a library: `(cast\|play)[^.]{0,60}from the top of (your\|their\|a\|that player's) library\|top card of your library[^.]{0,40}you may (cast\|play)`
- cast ~ from a graveyard (no keyword): `(cast\|play) ~ from (your\|a) graveyard`
- cast or play other cards from a graveyard: `(cast\|play)[^.]{0,80}from (your\|a\|their\|an opponent's\|that player's) graveyards?`
- cast or play a card from exile: `(you may\|may) (cast\|play) (it\|that card\|those cards\|them\|the exiled card\|cards exiled\|one of\|a card exiled\|spells? from among)\|(cast\|play)[^.]{0,60}from exile\|exiled with ~`
- cast as though it had flash: `as though (it\|they) had flash\|any time you could cast an instant`
- cast ~ only at a time or if a condition holds: `cast (~\|this spell) only`
- a player can't cast: `can't (be )?cast`
- play an additional land: `play (an\|two\|three\|any number of) additional lands?\|play any number of lands`
<!-- cast-census: end CAST -->

The clause column partitions, so a sentence that names exile and a
graveyard counts once. The Scryfall column counts cards by a looser query and
overlaps.

## 7. Keyword families

<!-- cast-census: begin KEYWORDS -->
| family | keywords, cards each | CR | surface | status | owner | cards | Scryfall | query (plus `BASE`) |
|---|---|---|---|---|---|---:|---:|---|
| cast from a graveyard | flashback 215, escape 32, retrace 17, jump-start 12, aftermath 27, disturb 25, mayhem 16, harmonize 11 | 601.3 | `AlternativeCost::Flashback`, `Escape`; `CannotCast::NotInHand` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 355 | 355 | `(kw:"flashback" or kw:"escape" or kw:"retrace" or kw:"jump-start" or kw:"aftermath" or kw:"disturb" or kw:"mayhem" or kw:"harmonize")` |
| exiled by a special action, cast later | foretell 51, plot 39, suspend 70 | 116.2f | no special action but a land play | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 160 | 160 | `(kw:"foretell" or kw:"plot" or kw:"suspend")` |
| cast while something resolves | cascade 37, discover 30, ripple 5, rebound 34, madness 61, miracle 17 | 608.2g | no cast during a resolution | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 183 | 183 | `(kw:"cascade" or kw:"discover" or kw:"ripple" or kw:"rebound" or kw:"madness" or kw:"miracle")` |
| cast from the command zone (a commander) | -- | 903.8 | `CannotCast::NotInHand`; `is_commander` is B2's | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | -- | 3,497 | `is:commander` |
| cast face down | morph 153, megamorph 31, disguise 43 | 708.4 | no face-down spell | owned | `copy-effects-architecture.md` CV-6, face-down (`roadmap-v2.md` B6) | 227 | 227 | `(kw:"morph" or kw:"megamorph" or kw:"disguise")` |
| cast with other characteristics | prototype 19 | 718.3 | no alternative characteristics | owned | `backlog.md` §2.41, the copy track (`roadmap-v2.md` B6) | 19 | 19 | `(kw:"prototype")` |
| an alternative cost with an arm | evoke 35, dash 22, overload 27, bestow 43 | 118.9 | `AlternativeCost` arm; each needs a second half | **owned** (found unowned) | `roadmap-v2.md` C, Phase 8 breadth: a card is a normal diff over surfaces named in the row | 127 | 127 | `(kw:"evoke" or kw:"dash" or kw:"overload" or kw:"bestow")` |
| an alternative cost with no arm | emerge 15, spectacle 11, surge 11, prowl 10, blitz 17, mutate 34, freerunning 12, impending 5, warp 32, sneak 27, web-slinging 10, awaken 15, cleave 12 | 118.9 | `AlternativeCost::Custom` names it; each needs a second half | **owned** (found unowned) | `roadmap-v2.md` C, Phase 8 breadth: a card is a normal diff over surfaces named in the row | 211 | 211 | `(kw:"emerge" or kw:"spectacle" or kw:"surge" or kw:"prowl" or kw:"blitz" or kw:"mutate" or kw:"freerunning" or kw:"impending" or kw:"warp" or kw:"sneak" or kw:"web-slinging" or kw:"awaken" or kw:"cleave")` |
| kicker | kicker 226 | 702.33 | `AdditionalCost::Kicker`, `Condition::SpellWasKicked` | built |  | 226 | 226 | `(kw:"kicker")` |
| an additional cost with an arm | buyback 39, entwine 32, casualty 15, bargain 20, strive 20 | 118.8 | `AdditionalCost` arm; each needs a second half | **owned** (found unowned) | `roadmap-v2.md` C, Phase 8 breadth: a card is a normal diff over surfaces named in the row | 126 | 126 | `(kw:"buyback" or kw:"entwine" or kw:"casualty" or kw:"bargain" or kw:"strive")` |
| an additional cost paid any number of times | multikicker 19, replicate 19, squad 15 | 601.2b | 601.2b's announcement cannot ask for a number | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 53 | 53 | `(kw:"multikicker" or kw:"replicate" or kw:"squad")` |
| an additional cost with no arm | splice 30, conspire 11, escalate 9, offspring 20, gift 25, tiered 7 | 118.8 | `AdditionalCost::Custom` names it | **owned** (found unowned) | `roadmap-v2.md` C, Phase 8 breadth: a card is a normal diff over surfaces named in the row | 102 | 102 | `(kw:"splice" or kw:"conspire" or kw:"escalate" or kw:"offspring" or kw:"gift" or kw:"tiered")` |
| spree | spree 21 | 702.172 | a mode's own additional cost | owned | `codebase-state.md` item 81, spree | 21 | 21 | `(kw:"spree")` |
| pay with permanents or cards | convoke 104, improvise 23, delve 28, assist 16, offering 6, waterbend 28 | 702.51 | no payment but mana | owned | `codebase-state.md` item 162's design, the next PR | 204 | 204 | `(kw:"convoke" or kw:"improvise" or kw:"delve" or kw:"assist" or kw:"offering" or kw:"waterbend")` |
| a cost reduction | affinity 74, undaunted 5 | 702.41 | `CostChange::ReduceGeneric` | built |  | 79 | 79 | `(kw:"affinity" or kw:"undaunted")` |
| reads the mana spent | sunburst 16, converge 28, adamant 18 | 702.44 | `ManaSpent` by type | owned | `codebase-state.md` item 30, what paid (`ManaSpent` by type) | 62 | 62 | `(kw:"sunburst" or kw:"converge" or kw:"adamant")` |
| an activated ability that works in a hand | cycling 394, typecycling 94, landcycling 91, basic landcycling 36, channel 37, ninjutsu 35, commander ninjutsu 1, transmute 14, forecast 11, bloodrush 13, reinforce 11 | 113.6m | `CannotActivate::NotOnBattlefield` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 516 | 516 | `(kw:"cycling" or kw:"typecycling" or kw:"landcycling" or kw:"basic landcycling" or kw:"channel" or kw:"ninjutsu" or kw:"commander ninjutsu" or kw:"transmute" or kw:"forecast" or kw:"bloodrush" or kw:"reinforce")` |
| an activated ability that works in a graveyard | unearth 56, embalm 15, eternalize 12, encore 26, scavenge 14 | 113.6m | `CannotActivate::NotOnBattlefield` | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 123 | 123 | `(kw:"unearth" or kw:"embalm" or kw:"eternalize" or kw:"encore" or kw:"scavenge")` |
| tap creatures with total power N | crew 181, saddle 30, station 31 | 702.122 | no arm: tap N untapped creatures | **owned** (found unowned) | `cost-architecture.md` CP-2 (`roadmap-v2.md` B1, before C) | 242 | 242 | `(kw:"crew" or kw:"saddle" or kw:"station")` |
| an ability with a use limit | boast 18, exhaust 36, level up 25, outlast 13 | 602.5b | no `ActivationRestriction` arm | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | 92 | 92 | `(kw:"boast" or kw:"exhaust" or kw:"level up" or kw:"outlast")` |
| equip and its kin | equip 601, fortify 2, reconfigure 17 | 702.6 | `ActivationRestriction::OnlyAsSorcery` | built |  | 620 | 620 | `(kw:"equip" or kw:"fortify" or kw:"reconfigure")` |
| a planeswalker's loyalty abilities | -- | 606.3 | no loyalty ability | **owned** (found unowned) | `permission-architecture.md` (`roadmap-v2.md` B11: designed beside RS-2, built before B2) | -- | 311 | `t:planeswalker` |
| a cost paid as a trigger resolves | cumulative upkeep 80, echo 50 | 118.12 | CR 118.12's payment (TR-2b); age counters, item 3 | owned | `roadmap-v2.md` C, Phase 8 breadth: a card is a normal diff over surfaces named in the row | 130 | 130 | `(kw:"cumulative upkeep" or kw:"echo")` |
<!-- cast-census: end KEYWORDS -->

The family rows group keywords by the surface each one waits on. The first
missing surface decides the owner; a keyword's other half (evoke's sacrifice
trigger, dash's return, bestow's Aura) is the card's, in C.

## 8. Mana symbols, and mana by phrase

<!-- cast-census: begin SYMBOLS -->
| a mana cost with | CR | surface | status | owner | cards | Scryfall | query (plus `BASE`) |
|---|---|---|---|---|---:|---:|---|
| a {C/W}-style hybrid | 107.4e | no `ManaSymbol` arm | **owned** (found unowned) | `cost-architecture.md` CP-1 (`roadmap-v2.md` B1, any time) | 1 | 1 | `m:/\{C\/[WUBRG]\}/` |
| a hybrid Phyrexian symbol | 107.4f | `ManaSymbol::HybridPhyrexian`, unpaid | owned | `cost-architecture.md` CP-1 (`roadmap-v2.md` B1, any time) | 4 | 4 | `m:/\{[WUBRG]\/[WUBRG]\/P\}/` |
| a Phyrexian symbol | 107.4f | `ManaSymbol::Phyrexian`, unpaid | owned | `cost-architecture.md` CP-1 (`roadmap-v2.md` B1, any time) | 32 | 72 | `is:phyrexian` |
| a {2/W} symbol | 107.4e | `ManaSymbol::MonoHybrid`, unpaid | owned | `cost-architecture.md` CP-1 (`roadmap-v2.md` B1, any time) | 20 | 20 | `m:/\{2\/[WUBRG]\}/` |
| a hybrid symbol | 107.4e | `ManaSymbol::Hybrid`, unpaid | owned | `cost-architecture.md` CP-1 (`roadmap-v2.md` B1, any time) | 582 | 600 | `is:hybrid` |
| {S} | 107.4h | `ManaSymbol::Snow`, unpaid | owned | `codebase-state.md` item 33, mana provenance (`roadmap-v2.md` B9, before C) | 2 | 2 | `m:{S}` |
| {X} | 107.3a | `ChooseXValue` asked; `find_mana_sources` refuses X, so never offered | owned | `codebase-state.md` item 162's design, the next PR | 527 | 534 | `m:{X}` |
| {C} | 107.4c | `ManaSymbol::Colorless` | built |  | 27 | 28 | `m:{C}` |
| colored and generic only | 107.4a | `ManaSymbol::Colored`, `Generic` | built |  | 30,920 | -- | the rest |

A card lands in the first row its mana cost matches, so the cards column partitions the corpus; the Scryfall column counts every card the query matches, overlaps included.
<!-- cast-census: end SYMBOLS -->

**527 cards have `{X}` in their mana cost, and the priority question never
offers one.** `ManaPool::can_pay` returns `false` for an `{X}` symbol, and
`find_mana_sources` returns `None`, so `can_cast` answers `ManaShort` before
the cast that would ask `ChooseXValue` begins. No registered card has an X
cost, so no game meets it today. It is item 162's.

<!-- cast-census: begin PHRASES -->
| family | CR | surface | status | owner | cards | query (plus `BASE`) |
|---|---|---|---|---|---:|---|
| makes a Treasure (one mana of any color) | 111.10a | no `ManaOutput` form | **owned** (found unowned) | `codebase-state.md` item 162's design, the next PR | 347 | `o:"treasure token"` |
| a triggered mana ability | 605.1b | built (TR-1); affordability skips it | owned | `codebase-state.md` item 162's design, the next PR | 19 | `(o:"tapped for mana" o:"adds") or o:"is tapped for mana, its controller adds"` |
| replaces the mana a permanent produces | 106.12b | built (RE-9); affordability skips it | owned | `codebase-state.md` item 162's design, the next PR | 10 | `(o:"tapped for mana" o:"instead") or o:"produces twice"` |
| a spell or non-mana ability that adds mana | 106.3 | `Primitive::ProduceMana` off a resolution | built |  | 19 | `(t:instant or t:sorcery) o:/^add \{/` |
| spend mana as though it were any color or type | 609.4b | no payment path reads it | owned | `backlog.md` §2.24, "as though" (`roadmap-v2.md` B8, before C) | 49 | `o:"mana as though it were mana of any"` |
| mana that does not empty | 106.4 | `ManaPersistence`, no payment path reads it | owned | `codebase-state.md` item 33, mana provenance (`roadmap-v2.md` B9, before C) | 24 | `(o:"don't lose this mana" or o:"doesn't empty" or o:"don't empty")` |
| mana that does something when spent | 106.6 | `ManaGrant`, no payment path reads it | owned | `codebase-state.md` item 33, mana provenance (`roadmap-v2.md` B9, before C) | 8 | `(o:"spend this mana" -o:"spend this mana only")` |
<!-- cast-census: end PHRASES -->

## 9. The question kinds

<!-- cast-census: begin QUESTIONS -->
27 kinds, parsed from `pub enum ChoiceKind` in `mtgsim/src/ui/choice_types.rs`; `ui::why::refusals` answers 3 with a check (`DeclareAttackers`, `DeclareBlockers`, `PriorityAction`), read off its arms in `mtgsim/src/ui/why.rs`. Of the 24 it does not, 7 are **filtered**: a check drops a candidate the question could have named. 5 are **bounds**: the options are amounts, and the why explains a limit. 12 are **no**: every candidate is offered.

| `ChoiceKind` | its options | filtered | what a why reads | owner | slot |
|---|---|---|---|---|---|
| `PriorityAction` | lands, spells, abilities, and Pass | yes | `can_play_land`, `can_cast`, `can_activate` (SU-7) | built (SU-7) | -- |
| `DeclareAttackers` | creatures and what each may attack | yes | `can_attack` (SU-7) | built (SU-7) | -- |
| `DeclareBlockers` | creatures and what each may block | yes | `can_block` (SU-7) | built (SU-7) | -- |
| `AssignCombatDamage` | amounts across the creatures blocking it | bounds | CR 510.1c's lethal-damage bound | `setup-architecture.md` §7c | SU-8 |
| `AssignTrampleDamage` | amounts across blockers and the player | bounds | CR 702.19b's lethal-damage bound | `setup-architecture.md` §7c | SU-8 |
| `ChooseXValue` | a number | bounds | the bound the enumeration sets on X | `codebase-state.md` item 162 | item 162's build |
| `ChooseAlternativeCost` | the mana cost and each printed alternative | no | -- | `setup-architecture.md` §7c | SU-8 |
| `ChooseAdditionalCosts` | each optional additional cost | no | a mandatory cost is in the total, not offered (CR 118.8) | `setup-architecture.md` §7c | SU-8 |
| `SelectRecipients` | the objects or players a target or choice may name | yes | `validate_selection`'s reasons, typed | `cant-effects-architecture.md` RS-2 | RS-2 |
| `GenericManaAllocation` | a split of the generic part across the pool | bounds | the clamp that keeps each pip's mana | `codebase-state.md` item 162 | item 162's build |
| `OrderCostReductions` | an order of every reduction that applies | no | -- | `setup-architecture.md` §7c | SU-8 |
| `ManaAbilityWindow` | the mana abilities that can pay now | yes | `can_pay_costs`'s `CannotPay`, per ability | `codebase-state.md` item 162 | item 162's build |
| `ChooseSacrificeForCost` | the permanents that can pay the cost | yes | CR 701.21a and the cost's filter | the cost phase that pays the cost | the cost phase |
| `ChooseReplacementEffect` | the replacement effects that apply | yes | the gather's candidates, from the trace | `setup-architecture.md` §7c | SU-8 |
| `OrderTriggers` | an order of one player's pending triggers | no | the dispatch's matcher, from the trace | `setup-architecture.md` §7c | SU-8 |
| `ApplyOptionalReplacement` | yes or no | no | -- | `setup-architecture.md` §7c | SU-8 |
| `ApplyOptionalEffect` | yes or no | no | -- | `setup-architecture.md` §7c | SU-8 |
| `AllocateNextDamage` | amounts across the damage sources | bounds | CR 615.7's count | `setup-architecture.md` §7c | SU-8 |
| `ChooseDamageSource` | every permanent and spell | no | -- | `setup-architecture.md` §7c | SU-8 |
| `ChooseEnteringController` | the opponents still in the game | yes | CR 800.4a | `setup-architecture.md` §7c | SU-8 |
| `ChooseAuxiliaryZoneChange` | the objects CR 614.13a/b and CR 101.2 allow | yes | the filter and the restriction that dropped it | `setup-architecture.md` §7c | SU-8 |
| `ChooseCopySource` | the permanents the copy effect may copy | yes | the copy effect's filter | `setup-architecture.md` §7c | SU-8 |
| `CommanderToCommandZoneSba` | yes or no | no | -- | `setup-architecture.md` §7c | SU-8 |
| `Discard` | the cards in the hand | no | -- | `setup-architecture.md` §7c | SU-8 |
| `Scry` | the cards looked at | no | -- | `setup-architecture.md` §7c | SU-8 |
| `ScryOrder` | an order of one pile | no | -- | `setup-architecture.md` §7c | SU-8 |
| `LegendRule` | the legendary permanents sharing the name | no | -- | `setup-architecture.md` §7c | SU-8 |
<!-- cast-census: end QUESTIONS -->

**The shape at v1** (the owner, 2026-10-05). SU-8 gives every kind a line
saying what the question ranges over. It is an exhaustive match in
`ui::why::refusals`, so a new `ChoiceKind` decides its line at birth. SU-8
also reads the trace for two kinds: the replacement candidates and the
trigger order. Each kind a check filters gets a typed reason from that check,
with the check's owner:

- `SelectRecipients`, with RS-2;
- `ManaAbilityWindow`, `ChooseXValue` and `GenericManaAllocation`, with item
  162's build;
- `ChooseSacrificeForCost`, with CP-2;
- `ChooseCopySource`, `ChooseAuxiliaryZoneChange` and
  `ChooseEnteringController`, with SU-8, since they read one filter apiece.

The kinds marked "no" need no reason: every candidate is offered. A why
asked about anything else says the question does not range over it.

## 10. Calibration

The bar: every family the registered cards use must classify as built,
since the engine plays them. The registered printings went through the same
classifiers by name:

<!-- cast-census: begin CALIBRATION -->
181 names registered in `mtgsim/src/cards/registry.rs`; Scryfall knows 180 of them, and the rest are fixtures: Loyalty Probe.

| table | family | status | registered cards that use it |
|---|---|---|---|
| symbol | `sym-hybrid` | owned | Mirrorweave |
| cost | `life` | built | Yawgmoth's Bargain |
| cost | `mana` | built | Aggravated Assault, Chainbreaker, Circle of Protection: Red, Deep Water, Doubling Cube, Elvish Warmaster and 4 more |
| cost | `sacrifice` | built | Altar's Reap, Bone Splinters, Kalitas, Traitor of Ghet, Krark-Clan Ironworks |
| cost | `sacrifice-self` | built | Dark Sphere, Mind Stone, Samite Censer-Bearer |
| cost | `tap-self` | built | Chainbreaker, Citanul Hierophants, Dark Sphere, Doubling Cube, Merfolk Thaumaturgist, Mind Stone and 5 more |
| keyword | `kw-affinity` | built | Frogmite, Myr Enforcer |
| keyword | `kw-equip` | built | Bonesplitter, Cobbled Wings |
| mana | `fed-mana` | built | Doubling Cube |
| mana | `fed-other` | built | Citanul Hierophants, Mind Stone, Nephalia Academy, Seat of the Synod, Sol Ring |
| mana | `fed-sacrifice` | built | Krark-Clan Ironworks |
| mana | `prod-double` | built | Doubling Cube |
| mana | `prod-one` | built | Citanul Hierophants, Mind Stone, Nephalia Academy, Seat of the Synod |
| mana | `prod-several` | built | Krark-Clan Ironworks, Sol Ring |
| when | `when-any` | built | Chainbreaker, Circle of Protection: Red, Citanul Hierophants, Dark Sphere, Deep Water, Doubling Cube and 13 more |
| when | `when-sorcery` | built | Aggravated Assault |
| where | `where-battlefield` | built | Aggravated Assault, Chainbreaker, Circle of Protection: Red, Citanul Hierophants, Dark Sphere, Deep Water and 14 more |
| who | `who-controller` | built | Aggravated Assault, Chainbreaker, Circle of Protection: Red, Citanul Hierophants, Dark Sphere, Deep Water and 14 more |
<!-- cast-census: end CALIBRATION -->

**It passes, with one exception on the record and one correction to the
census.**

- **Mirrorweave** prints `{2}{W/U}{W/U}` and is registered as `{2}{W}{U}`,
  because no payment path pays hybrid. Item 162 recorded that on 2026-09-30.
  The census finds it unprompted, which is the check working.
- **The first run put Ironworks and Sol Ring under "owned".** The mana table
  then had one status column, and it meant the affordability check rather
  than play. Splitting it into "plays" and "the affordability check" is the
  refinement `cr-coverage-audit.md` §3's calibration made for its own
  method: a surface can be built while a reader of it is not.

The calibration also shows how narrow the engine's corner of this area is.
Its registered cards use five cost actions, one activation limit, no zone but
the battlefield and no keyword family but affinity and equip.

## 11. SU-7's four findings — the shape at v1, the owner, the slot

1. **The permission-shaped reasons.**
   - **v1:** `CannotCast::NotInHand` becomes `NotPermitted { zone }`, CR
     601.3's "no rule or effect allows". `CannotPlayLand::NotInHand` (CR
     305.1) and `CannotActivate::NotOnBattlefield` (CR 113.6) become the same
     reason. The owner test leaves `can_begin_to_cast` and `can_play_land` and
     moves inside the permission: 903.8's commander is "a commander they
     own", Hostage Taker casts one its caster does not. RS-2's prohibition
     arrives at the same three checks as a reason naming its source, so both
     arms are designed together.
   - **Owner:** `permission-architecture.md`. **Slot:** its design beside
     RS-2; its first build before B2.
2. **Who may activate, per ability.**
   - **v1:** a field on `AbilityDef` naming who may activate: its controller
     (CR 602.2's default), any player (39 cards), or only an opponent (5).
     `can_begin_to_activate` takes the player and reads it, so the test is
     per ability. The enumeration walks every permanent in
     `battlefield_ids_ordered`, not only the player's, and
     `can_activate_its_abilities` folds into the per-ability check.
   - **Owner:** `permission-architecture.md`, graduating "Before card
     breadth" item 2. **Slot:** as item 1.
3. **The four costs neither checked nor paid.**
   - **v1:** `check_cost_resource` checks each one. Discard and exiling from
     a graveyard each need enough matching cards. Removing counters needs
     enough of the kind; putting counters on a permanent is payable unless a
     "can't" says otherwise (Solemnity, RS-4). Each is paid through the
     chokepoint, and its choice (which card, which counter) is asked before
     any payment is performed, as `Cost::Sacrifice`'s is. Counts are amounts
     that can be X, not a `u32`; 97 pieces in §3 need that already. The
     counter arms name their permanent, so "remove a counter from a creature
     you control" and blight are the same arms.
   - **Owner:** `cost-architecture.md` CP-2. **Slot:** before C, beside CP-1.
4. **The 24 question kinds the why does not explain.** Item 212 and #222's
   review reply both said "of 30"; `ChoiceKind` has 27, and the why answers 3.
   - **v1:** §9.
   - **Owner:** `setup-architecture.md` §7c, with each filtered kind's owner.
     **Slot:** SU-8 for the line every kind gets and for the two trace kinds;
     each typed reason with its owner's phase.

## 12. What the census settles for item 162's design

Item 162's design is the next PR. It "surveys every payment family the CR
and the cards have, says which fit which algorithm, and prices each on the
priority question". The census hands it the families with their counts, and
three scope changes.

- **Production it now owns**, per the owner's decision: one mana of any color
  (380 cards, plus 347 that make a Treasure); painland riders (129 abilities
  `resolve_mana_effect` cannot resolve, against `backlog.md` §2.19's claim
  that they are "already carriable"); "the chosen color" (32), whose record
  of the as-enters choice is `backlog.md` §2.2's; and "could produce" (14).
- **What the affordability check misreads today**, from §5's last column:
  - it counts one mana for several (156 abilities);
  - it counts each listed type as a source of its own (381);
  - it skips a variable amount (60) and the doubled pool (1);
  - it skips a source whose cost is mana unless the pool already holds that
    cost (175);
  - it counts a sacrifice as free, including the sacrificed source (143);
  - it counts restricted mana as unrestricted (153).

  Also unread: triggered mana (19 cards) and the replacements on it (10),
  from §8.
- **X, never offered:** 527 cards with `{X}` in their mana cost (§8). The
  bound the enumeration sets on X is `ChooseXValue`'s why (§9).
- **Payment by permanents or cards:** convoke 104, improvise 23, delve 28,
  assist 16, offering 6 and waterbend 28 (§7). Item 162's survey owned these
  already.
- **The window's offer:** the mana abilities at a priority question (item
  211) and `ManaAbilityWindow`'s why (§9).

Pricing each family on the priority question is that design's own work. The
census counts families; it does not measure them.

## 13. The brief's priors

<!-- cast-census: begin PRIORS -->
| the brief's prior | brief | reproduced | query | Commander-legal, `BASE` |
|---|---:|---:|---|---:|
| activated abilities (a colon in the text) | 8,612 | 8,612 | `o:":" legal:commander -t:token game:paper -is:funny` | 8,612 |
| an additional cost | 316 | 316 | `o:"additional cost" legal:commander -t:token game:paper -is:funny` | 316 |
| hybrid or Phyrexian mana | 668 | 668 | `(is:hybrid or is:phyrexian) legal:commander -t:token game:paper -is:funny` | 668 |
| convoke, improvise, delve, emerge or affinity | 243 | 243 | `(kw:convoke or kw:improvise or kw:delve or kw:emerge or kw:affinity) legal:commander -t:token game:paper -is:funny` | 243 |
| flashback, escape, unearth, retrace, jump-start, embalm, eternalize or disturb | 384 | 384 | `(kw:flashback or kw:escape or kw:unearth or kw:retrace or kw:jump-start or kw:embalm or kw:eternalize or kw:disturb) legal:commander -t:token game:paper -is:funny` | 384 |
| "spend this mana only" | 165 | 165 | `o:"spend this mana only" legal:commander -t:token game:paper -is:funny` | 165 |
| "any player may activate" (paper) | 40 | 40 | `o:"any player may activate" game:paper -is:funny` | 39 |
| "only your opponents may activate" (paper) | 5 | 5 | `o:"only your opponents may activate" game:paper -is:funny` | 5 |
| discard as a cost (paper) | 384 | 384 | `o:/discard [^:]*:/ game:paper -is:funny` | 384 |
| exile from a graveyard as a cost (paper) | 161 | 161 | `o:/exile [^:]*from your graveyard[^:]*:/ game:paper -is:funny` | 159 |
| removing counters as a cost (paper) | 334 | 334 | `o:/remove [^:]*counters? from [^:]*:/ game:paper -is:funny` | 334 |
| cycling (paper) | 399 | 399 | `kw:cycling game:paper -is:funny` | 394 |
<!-- cast-census: end PRIORS -->

All twelve reproduce on their first query. Three of the paper priors are
regexes that can run past a sentence, from an effect's "discard" to a later
ability's colon, so they are upper bounds. The last column puts them on the
census's corpus: compare it with §3, where each cost is cut apart, not with
the paper count. §3's numbers are the ones to cite.

## 14. What the census cannot see

- **A family's second half.** The census classifies the cost and the
  permission. A keyword's other half (evoke's sacrifice trigger, flashback's
  exile, buyback's return to hand) is named in the row's surface when one
  owner holds it, and is otherwise the card's.
- **Restrictions that arrive as keywords.** Split second is a prohibition on
  others (RS-2), and the "can't" census counts its printed half.
- **A count is not a cost.** Families are sized by cards waiting on them, not
  by the code they want. One `AmountExpr` in `Cost`'s count arms serves 97
  pieces.
- **Oracle text, not printings.** The corpus is `unique=cards`, so a reprint
  counts once.

## 15. The rule numbers, audited

The script resolves every rule number a row cites against the CR at startup,
and fails on one that does not resolve (`copy-census.py`'s label audit).

<!-- cast-census: begin LABELS -->
Every rule number the tables cite, resolved in `MTG-Rules/versions/tmnt.txt`: 106.1, 106.1b, 106.3, 106.4, 106.6, 106.7, 106.12b, 107.3, 107.3a, 107.4a, 107.4c, 107.4e, 107.4f, 107.4h, 107.5, 107.6, 107.14, 107.17a, 111.10a, 113.6, 113.6b, 113.6j, 113.6m, 116.2f, 117.1a, 118.1, 118.8, 118.9, 118.12, 119.4, 122.1, 305.1, 305.2, 406.3, 601.2, 601.2b, 601.2g, 601.3, 601.3b, 601.6, 602.1, 602.2, 602.5, 602.5a, 602.5b, 602.5d, 602.5e, 605.1a, 605.1b, 606.3, 606.4, 607.2d, 608.2g, 609.4b, 701.3d, 701.4a, 701.9a, 701.9b, 701.13, 701.17, 701.20a, 701.21a, 701.26a, 701.26b, 701.43, 701.59, 701.61, 701.67, 701.68, 702.6, 702.33, 702.41, 702.44, 702.51, 702.122, 702.172, 706.1, 708.4, 718.3, 903.8.
<!-- cast-census: end LABELS -->
