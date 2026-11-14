# Mana — what a player can pay, and what mana abilities make

> **Status:** design, 2026-10-05, for `codebase-state.md` item 162, decided
> at #224's review the same day: the owner took every recommendation in
> §7, shape C for item 211 among them. MA-1 landed 2026-10-05 (§10); §3.3,
> §3.4 and §3.8 say where it built otherwise than designed. Every number in
> §2 to §6 is from a measurement made 2026-10-05 on `f4df90a` (#223's
> merge), and §2 says how. MA-2 landed 2026-10-18 (§10): §3.7 holds the
> bound on X as built, and its plan is in the archive. **Mana provenance**
> (`codebase-state.md` item 33) is designed in
> `#11-mana-provenance-what-a-unit-of-mana-keeps`, 2026-11-13, at its design
> stop: what a unit of mana keeps, which unit pays, restricted mana, `{S}`,
> and mana kept or lost as a step ends (CR 106.3, 106.4, 106.6, 107.4h,
> 400.7d, 500.5).
> **Authority:** whether a player can pay a cost at this moment, the
> existence half of CR 601.2g–h and 602.2b that decides what the priority
> question offers. What a mana ability makes and how it resolves (CR 106.1b,
> 106.7, 605.1–605.4). The inventory CR 601.2g's window reads, and CR
> 605.3a's window at a priority question. The bound `ChooseXValue` offers
> (CR 107.3a). Paying with permanents or cards: convoke (702.51), delve
> (702.66), improvise (702.126), waterbend (701.67), assist (702.132) and
> offering (702.48). On what exists, `codebase-state.md` wins; on what is
> being built, this file does; `CLAUDE.md`'s critical path owns the ordering.
> **Graduates:** item 162 (the matching and both its customers), item 211
> (mana abilities at a priority question), `backlog.md` §2.18's solver
> (§2.22 rows 6 and 7), §2.19 (one mana of any color), and the mana half of
> §2.2's as-enters record.
> **Companions:**
> - `cost-architecture.md`. `hybrid-and-phyrexian-payment` pays the symbol alphabet, and its check
>   half is §3.2 here. `remaining-costs` owns the cost actions.
> - `codebase-state.md` item 33 (mana provenance, `roadmap-v2.md` `mana-provenance`), whose
>   seam is §3.6.
> - `setup-architecture.md` §7c, the why of the three mana prompts (§3.13).
> - `replacement-architecture.md` (RE-9's mana replacements) and
>   `triggers-architecture.md` (TR-1's triggered mana abilities): §3.4 reads
>   both and changes neither.
>
> Phase codes are `MA-*`; branches are `mana/ma-<n>-…`.

---

## 0. Where this sits, and why a document

**On the event path, it runs before anything is proposed.** The priority
question builds its candidate list (`candidate_priority_actions`), and for
each card in hand and each ability asks `can_cast` or `can_activate`. Their
last step asks whether the mana can be paid. When the player picks a cast,
`cast_spell` walks CR 601.2. At 601.2g the window asks one mana ability at a
time (`ManaAbilityWindow`). Each activation resolves at once
(`resolve_mana_effect`) and proposes `GameAction::ProduceMana` through the
chokepoint, where CR 106.12b's replacements apply and CR 605.1b's triggered
mana abilities fire. At 601.2h `plan_payment` decides and `pay_costs`
performs. A cast the check offered and the pool cannot cover rewinds
(CR 732.1). This design changes the first step, reads the replacement and
trigger subsystems' outputs without changing them, and adds forms to what a
mana ability makes. The payment's performing side stays `hybrid-and-phyrexian-payment`'s and item 33's.

**The owner's rule** (2026-09-30, item 162): the check at every priority
point asks only whether a payment exists. Comparing payments runs once per
payment, on a person's seat. §3 is the first; §3.12 the second.

**A document of its own (decision 1).** The build is six PRs (§6). It owns
rules no document owns: CR 106's production and 605's resolution, 601.2g's
window, and 702.51's family of payments. It graduates three backlog entries
and two items. `cost-architecture.md` §0 made the same call on the same
count. `CLAUDE.md`'s architecture row gains a name, not a line.

---

## 1. The rules this reads

Paraphrased, with the rule each comes from; `MTG-Rules/versions/tmnt.txt`
has the text.

| Rule | What it says, for this design |
|---|---|
| 601.2g | The window opens only when the total cost includes mana, and mana abilities are activated before costs are paid |
| 601.2h | The total cost is paid all at once: no partial payment, and an unpayable cost cannot be paid. So the offer must agree with the payment (`cost-architecture.md` §3.6) |
| 605.1a, 605.1b | What makes an ability a mana ability: activated (no target, could add mana, not loyalty) or triggered (from a mana ability or from mana being added) |
| 605.3a | A mana ability may be activated whenever its controller has priority, while paying, or when a rule or effect asks for mana. Item 211 is the first clause |
| 605.3b, 605.4a | It resolves at once, so the board it reads is the board it was activated on, and so is a triggered mana ability's |
| 106.1b | Six types of mana; "any color" is a choice among five |
| 106.7 | "Could produce": the types a permanent's mana abilities would make now, replacement effects applied in any order, costs ignored |
| 106.12b | A replacement on a permanent "tapped for mana" changes that production (Mana Reflection, Deep Water) |
| 107.3a | X is announced while casting, as part of 601.2b |
| 702.51a, 702.66a, 702.126a, 701.67a | Convoke, delve, improvise and waterbend pay a pip with a tapped permanent or an exiled card instead of mana. 702.51b and its siblings: they apply after the total cost is set, so at 601.2h, after the window |

**The rules version.** The engine targets the frozen CR, `tmnt.txt`. The
Hobbit update (August 2026) adds a criterion to CR 605.1a: an activated
ability whose cost or effect moves a card to or from a library is not a mana
ability. Ten cards stop being mana abilities and gain "Activate only as an
instant": Chromatic Sphere, Selvala, Explorer Returned, Millikin, Deranged
Assistant, Charmed Pendant, and the five Eggs (Darkwater, Mossfire,
Shadowblood, Skycloud, Sungrass). Triggered mana abilities (605.1b) are
unchanged. What it changes here:
- **Nothing in the algorithm.** The inventory reads the abilities a card
  marks as mana abilities, the same ones the window offers. A rules version
  is one predicate where that classification lives (`cost-architecture.md`
  §3.11), and the inventory follows it.
- **`any-color-plus-painlands-plus-signets`'s draw inside a mana ability is a frozen-text facility.** The Eggs,
  Chromatic Sphere and Selvala draw, and under the new text those abilities
  use the stack. `ironworks-loop`'s Ironworks loop (TR-7's until 2026-10-13) tests the
  frozen loop and needs it. A painland's
  damage moves no card, so it stays a rider under both texts, and so does
  the window inside a Signet's activation.
- **Millikin's and Deranged Assistant's "Mill a card"** is `remaining-costs`'s single-arm
  action under either text (in C); the text decides only whether the ability
  it pays for is a mana ability.

The census's walk finds eight of the ten among its mana abilities. Charmed
Pendant and Selvala make their mana with "for each", which its production
pattern does not read as a mana ability, a note for the census's owner.

---

## 2. What the engine has, and what it costs

### 2.1 The tree, at `f4df90a`

- `can_cast` (`oracle/mana_helpers.rs:212`) and `can_afford_ability_costs`
  (`:616`) call `find_mana_sources` (`:40`) after `remaining_cost_after_pool`
  (`:317`). It is a greedy count over `available_mana_sources` (`:91`), which
  lists one source per (permanent, ability, type) with a `Fixed` amount, and
  it returns `None` for any symbol but colored, `{C}` and generic.
- The window, `run_mana_ability_window` (`engine/put_on_stack.rs:533`),
  re-enumerates `enumerate_activatable_mana_abilities` (`oracle/mana_helpers.rs:295`)
  once per prompt. `ManaWindowStop` declines once the cost is covered.
- `ManaPool::can_pay` and `pay` (`types/mana.rs:793`, `:829`) read the simple
  pool. `resolve_mana_effect` (`engine/mana.rs:94`) resolves `ProduceMana`
  atoms and sequences of them, and refuses anything else (`:127`).
- `ask_choose_x_value` offers `(0, u64::MAX)` and leaves the bound to each
  provider (`ui/ask.rs:604`). The random agent counts untapped lands.

### 2.2 What it costs on `close_out.py`'s board

Callgrind on `main`, the budget's board (`--games 20 --seed 12345 --pool
performance --players 4 --deck-size 100 --life 40`, `MTGSIM_HASH_SEED=1`):
7,856,427,581 instructions over 628 decisions a game, so **625,512 a
decision**. **A point is one percent of that, 6,255 instructions a
decision**: `engineering-practices.md#the-budget` lets a PR add 2.5
points at identical games, and every "points" figure below is this unit.

| What | Share | Calls | Per call | Per decision |
|---|---:|---:|---:|---:|
| `candidate_priority_actions` | 18.73% | 58,921 | | 117,131 |
| `available_mana_sources`, from both callers | 12.50% | 45,962 | 21,374 | 78,214 |
| `find_mana_sources` (the check) | 6.72% | 39,617 | 13,331 | 42,048 |
| `remaining_cost_after_pool` | 0.57% | 39,617 | 1,132 | 3,571 |
| `run_mana_ability_window` | 9.74% | 1,955 | 391,231 | 60,896 |

Item 162 recorded 18.6% and 13% on SU-6's engine arm; this reproduces them on
the tree after SU-7. **The check, with its subtraction, is 45,620
instructions a decision, 7.3 points.** Almost all of it is the scan: one
`available_mana_sources` for every card that reaches the mana check (33,550
cards and 6,067 abilities in 20 games). A scan reads about 8.8 permanents the
player controls and 16 mana abilities, after sorting the whole battlefield by
timestamp (3,600 instructions a sort). **The sort is the determinism rule**
(`CLAUDE.md`): the battlefield is a `HashMap` whose hasher is seeded per
process, and the scan's order reaches a choice twice. The greedy picks
sources by position, and the window's option list is built from the same
scan, and a provider answers by index. It sorts every permanent of every
player to keep the handful the player controls, on every call: item 138's
lever 6, the timestamp sort behind 42 call sites. §3.5 says what the
inventory does instead.

The window's scans are a different cost. 6,345 enumerations serve 1,955
windows. The first in each window costs about 214,000, because the cast's
CR 601.2a move bumped the layer epoch and every frame is walked again. The
rest cost about 13,000. A tap bumps nothing (`bump_layer_epoch`'s list).

### 2.3 What an exact check changes: the probe

**The probe** is a throwaway crate outside the tree (`engineering-practices.md`'s
rule for a measurement), never merged. It depends on the crate at `f4df90a` and replays the budget's board
game for game as `fuzz_games` plays it. A decorator outermost in the stack
stops at every priority point. It answers a `[Pass]`-only prompt itself, so
the agent draws nothing it did not draw before, and it runs the prototype
check beside the engine's own, black-boxed. It reproduces the board: 58,921
priority points (`castable_spells`' call count), 33,550 cards and 6,067
abilities reaching the mana check (`find_mana_sources`' two caller counts),
81.5 turns, and 628 decisions a game. Every prototype piece is its own
function, so one callgrind run prices them all (§4).

What the exact check would change on that board, in 20 games:

| | Cards | Abilities |
|---|---:|---:|
| Offered today, refused by the exact check | 1,963 | 44 |
| Refused today, offered by the exact check | 16 | 136 |

- **The over-offers** are mostly Everywhere's five abilities counted as five
  sources. One untapped Everywhere offers Rhox Faithmender's `{3}{W}`.
- **The under-offers are the census's families, live on this board.** Wild
  Growth's extra `{G}` on a Badlands pays Thornweald Archer's `{1}{G}`. Sol
  Ring's `{C}{C}` pays Chainbreaker's `{3}` beside a Plains. Mana Reflection
  doubles an Everywhere into `{W}{W}` and a dual into `{U}{U}` for Humility's
  `{2}{W}{W}`. Each is a castable spell the engine does not offer, so item
  162 is **reachable — wrong today**, not only a cost.
- **Rewinds:** 448, 22.4 a game (428 casts, 20 activations), the count
  `fuzz_games` does not print. **All 448 were offers the exact check
  refuses**, and none came from the agent's window policy failing where a
  payment existed.
- **Soundness on real play:** of 1,397 casts and activations that reached
  the mana check and then completed, the exact check said yes to all 1,397.
- **The prompts:** 317 priority prompts become `[Pass]`-only, and 134
  `[Pass]`-only points gain a cast.

---

## 3. The model

### 3.1 The inventory

**What a player can pay with, at one moment.** It is taken once per player
per priority point, at the first card or ability that reaches the mana
check, not once per card. That makes 17,906 inventories in 20 games against
today's 39,617 scans. Each **entry** is one use of a shared cost and what it
gives:
- **How much:** Sol Ring's entry gives two.
- **Which pips that mana can pay:** a set of the six types. Everywhere's
  five abilities share one `{T}`, so they are one entry of one mana, any of
  W, U, B, R or G. A dual is one entry, W or U.
- **Whether the amount is one choice:** "two mana of any one color", or a
  multiplier over an entry of several types (§3.3).
- **Its permanent**, so an ability's own `{T}` can take it out.

The entries come from the effective ability list, as `available_mana_sources`
reads it (`CLAUDE.md`'s layer invariant), grouped by the cost they share.
Abilities sharing `{T}` are alternatives, and one entry holds them. An ability
whose cost is a sacrifice or nothing is an entry of its own. The pool is
entries too, one per type of mana in it.

**The mana check runs last, for abilities as it already does for cards.**
`can_cast` asks timing, targets and the other costs before mana; before
MA-1, `can_afford_ability_costs` walked the printed cost list, mana first on
most cards, and asked targets after, and MA-1's `can_activate_as_its_controller`
asks mana last. Asking the cheap refusals first is what takes
the inventory count to 17,906 from 19,675. It changes one thing a person can
see: the why of a tapped Chainbreaker with too little mana says "tapped", not
"not enough mana" (SU-7's rule: the first reason the check refuses).

### 3.2 The check: Hall's condition

**A cost is payable exactly when, for every set of its pip kinds, the pips
of those kinds number at most the mana of the entries that can pay any of
them.** Entries supply mana, pips demand it, and an entry can pay a pip when
their type sets meet. That is a transportation problem, and Gale's
supply–demand theorem (Hall's, for b-matchings) says it is feasible exactly
when the condition holds. It is the flow item 162 and `backlog.md` §2.22
named, with its feasibility read in closed form instead of by augmenting
paths.

**The alternatives, and why not** (the owner asked at #225's review):
- **Flow by augmenting paths** gives the same answer with more work. It is
  kept for `mana-payment-solver`'s solver, which needs the payment, not only its existence.
- **A linear program, or any convex relaxation**, is exact where the
  closed form already is: the transportation polytope's vertices are
  integral. It is wrong where the closed form needs help. A relaxation
  splits a one-choice entry's mana across types, half a `{W}{W}` and half
  a `{U}{U}`, and so pays `{W}{U}` from Mana Reflection on an Everywhere,
  which no tap makes.
- **Integer programming** is exact, but it is a solver call per check
  against about 100 instructions a cost, and floating point at a decision
  site is a determinism risk.
- **Restricted mana** needs none of them. A restriction names what the
  mana may pay for, a spell's types or an ability's source, which is the
  whole cost, so the purpose filters the supply before Hall's condition and
  the check stays exact (§3.6). A grant (Cavern of Souls' "can't be
  countered") changes nothing about whether a payment exists. Hall's
  theorem holds for any bipartite graph, so a restriction naming part of a
  cost would only narrow what that mana reaches.

**How it runs.** The inventory keeps its free mana as a list of (type set,
amount) (`SplitSupply::free`; this design had a table over every type set,
built once by a subset-sum transform, and MA-1 built the list). For each
nonempty subset of the cost's colored pip kinds, `feasible` adds up the
mana whose type set meets that subset and compares it with the pips asked.
A generic pip accepts every type, so it enters only the total. A cost with
k distinct colored pip kinds needs at most 2^k − 1 comparisons; k is 1–3
on nearly every printed card. **About 100 instructions a cost**,
measured on the board.

**Every symbol has a demand:**

| Symbol | Its pip kind | Exact? |
|---|---|---|
| `{W}` … `{G}`, `{C}` | one type | yes |
| generic | the total only | yes |
| hybrid `{W/U}`, and `{C/W}` | the two types | yes, by the same theorem |
| `{2/W}`, Phyrexian `{W/P}` and `{W/U/P}` | a choice per symbol, paid colored or another way (two generic, or 2 life): enumerate how many of each kind go each way, n+1 choices for n such symbols of one type | yes |
| `{S}` | a seventh bit, set on a snow permanent's entries | yes for permanents; the pool's snow mana waits for item 33 |
| `{X}` | a spell's is expanded at its least legal X before the check: 0, or 1 generic under "X can't be 0" (§3.7). An unexpanded `{X}`, an activated ability's, is refused until `hybrid-and-phyrexian-payment` announces it | yes |

**Until a payment path pays a symbol, the check refuses it.** Enumeration
must agree with enforcement (`cost-architecture.md` §3.6): the gate's hybrid
arm switches on in `hybrid-and-phyrexian-payment`'s PR, beside `ManaPool::pay`'s, and `{S}` with item
33's. That is today's behavior for those symbols, and nothing is offered
that the payment would refuse.

### 3.3 Where mana is not independent

The theorem needs each mana's type chosen on its own. The shapes below
break that, and each gets an exact answer. Only an enumeration past its
stated cost cap answers yes without finishing: **an over-offer costs a
rewind, an under-offer hides a legal play** (item 162, 2026-09-30), so the
gate leans one way only.

- **One choice for several mana:** "two mana of any one color" (54
  abilities), or a multiplier over an entry of several types. Mana Reflection
  on Everywhere gives two of one type, not one W and one U. Each such entry
  is enumerated over the colored kinds the cost asks for, and its mana counts
  toward generic otherwise. Identical entries are enumerated as a multiset,
  capped at 4,096 leaves. **About 2,800 instructions a cost where present**:
  274 of the board's inventories.
- **Alternatives of different sizes.** On the board this is Sol Ring made a
  creature by March of the Machines and granted Citanul Hierophants'
  `{T}: Add {G}`: one tap makes `{C}{C}` *or* `{G}` (4 of the probe's
  inventories). Merged into one entry, "two mana, C or G", it would pay
  `{G}{G}`, which no tap of Sol Ring makes. So the entry keeps its
  alternatives, and the check tries each, as it tries a one-choice entry's
  types. An alternative another one dominates (no fewer mana, and every type
  it makes) is dropped first: a green creature that taps for `{G}{G}` or
  convokes for one green or generic only ever needs the `{G}{G}`. These are
  rare enough that the enumeration costs nothing measurable on the board.
- **Mana-fed sources** (175: 128 that tap, once a turn; 47 repeatable). A
  converter's input is extra demand and its output extra supply. The order
  they are activated in matters, because a converter cannot be paid with its
  own output. Three rules keep the check exact without enumerating orders
  where it can:
  - **A Signet-shaped converter**, one generic in and its mana out (Signets,
    Odyssey's filter lands), needs no order: a set of them can be activated
    exactly when the static problem can be solved with at least one of
    their inputs paid by mana that is not a converter's. Activate first the
    converters whose outputs pay the others' inputs, and each step finds a
    mana to spend. That is one more pip kind in Hall's condition, a
    converter's input only base mana can pay. A Plains, Golgari Signet
    (`{B}{G}`) and Izzet Signet (`{U}{R}`) against `{W}{B}{G}`: without the
    rule, each Signet's `{1}` is paid with the other's mana and the check
    says yes. But the Plains' `{W}` is the only mana that can start either
    Signet, and the cost needs it, so the board cannot pay; with the rule,
    the check agrees.
  - **Identical converters are counted, not subsetted**, and one whose output
    meets none of the cost's colored pips only adds to the total, so it is
    used all or not at all.
  - **Other shapes** (an input of a color, `{W/U}`'s filter lands; an input
    of two or more; a net loss, Celestial Prism) have their orders
    enumerated for up to three on a board. Beyond that the check over-offers.

  **The cap is a cost.** Each combination is one Hall check, about 100
  instructions. The prototype capped subsets at 2^6 = 64, which is 6,400
  instructions a cost: if every cost on the budget's board met it, 2.9
  points. That cap was the prototype's choice, not a measurement of decks.
  With counting and collapsing it binds only past six *distinct* converters
  whose outputs a cost's colored pips can use, on one player's board. `any-color-plus-painlands-plus-signets`
  states it as a cost (64 combinations a cost) and counts how often a fuzz
  game reaches it. **About 3,600 instructions a cost where present**, from
  the prototype's subsets.
- **Doubling Cube** (1 card) doubles the pool it resolves with, so a player
  taps everything else first, pays its `{3}`, and doubles the rest; mana
  made after it is not doubled. Treating that as "every mana counts twice,
  less six" over-offers, because each mana left doubles into two of *its
  own* type. Four Plains, three Islands and the Cube against
  `{W}{W}{W}{W}{W}{U}{U}{U}`: counted twice less six, the board is eight
  mana and seems to pay. But five white needs three white left after the
  `{3}` and three blue two blue, five mana kept of seven, leaving two for a
  `{3}`.
  **It is exact as a transform of the demand** (as built in MA-1): a pip of
  a type needs `ceil(n / f)` of that type left after the input, the whole
  cost `ceil(total / f)` left of any, with `f` two, or three under Mana
  Reflection, and the input is paid first; one Hall check (§3.2) on the
  mana made before the Cube answers. What Krark-Clan Ironworks sacrifices
  after the doubling, the Cube itself and then the last Ironworks, pays as
  made. It runs only on a board that holds the Cube.
- **A repeatable filter** (47) has no `{T}` and converts at a loss: Prismite
  turns `{2}` into one mana of any color, as often as it is paid. It is a
  converter activated k times, with k enumerated up to half the mana, and
  its inputs payable only by mana that is not its own output, the base-mana
  pip again. Counting its output without the mana it eats would over-offer.
- **Sacrifice-fed sources** (143). The 117 that sacrifice themselves
  (Treasure, Lotus Petal) are ordinary entries. The 26 that sacrifice
  another permanent (Ironworks, Ashnod's Altar) give their output once per
  permanent they may sacrifice. The cost's own sacrifices come out of that
  count first: the window comes before the payment (601.2g, then 601.2h), so
  a permanent sacrificed for mana cannot also pay the cost. Before MA-1 the
  greedy counted the sacrifice free (143); MA-1's check takes the cost's own
  sacrifices out of an outlet's fodder, and still reads loosely a source that
  sacrifices itself for mana and is what the cost's own sacrifice must take
  (`codebase-state.md` item 213).
- **The cost's own consumption.** An activated ability whose cost includes
  `{T}` takes its own source's entry out. Chainbreaker with Citanul
  Hierophants' grant cannot tap itself to pay its own `{3}`; before MA-1 the
  greedy counted it, and it was among the probe's 44 (§2.3). MA-1's check
  leaves it out. The same rule keeps
  `cost-architecture.md` §3.11's Mind Stone puzzle out of the offer: an
  ability whose cost sacrifices its source cannot also feed that source to
  Ironworks.

**Variable amounts** (60) are evaluated when the inventory is taken. A mana
ability resolves at once (605.3b), so the board it reads is this one.

### 3.4 What changes what a tap makes

CR 106.12b's replacements (Mana Reflection doubles, Nyxbloom Ancient
triples, Deep Water and Pale Moon retype) and CR 605.1b's triggered
mana (Wild Growth's `{G}`) change an entry. **Both are on the budget's
board**, and they matter in 918 of the 16,475 inventories the cards' checks
took.
- A multiplier multiplies an entry's amount, and makes an entry of several
  types a one-choice entry.
- Triggered mana is an entry of its own, tied to its host's tap and not
  multiplied (Mana Reflection's ruling).
- A retype changes an entry's types. Ignoring one errs both ways: a Forest
  under Deep Water would offer the `{G}` it no longer makes and hide the
  `{U}` it now does. Two retypes on one tap are its player's order (CR
  616.1), so the tap makes either type. Deep Water and Pale Moon are
  registered, so MA-1 reads them.

**The board's are read once per layer epoch, for every player at once.**
Each input is a layer-walk input: effective abilities, attachments and
controllers. So the epoch argument the frame memo stands on
(`layers-architecture.md` §12, "7a") covers this cache too, and it is exact.
A resolution's, in the replacement registry (Deep Water's, Pale Moon's),
are read at each inventory, since making one moves no layer input, and so
is a static ability's "as long as". On the board the scan costs
8,900 instructions and runs 5,099 times, against 17,906 times live: **3,600
a decision, against about 10,900 unmemoized**. A cache needs an audit, as the
frame memo's has: in debug builds, a live scan beside every hit, compared.

### 3.5 Order and determinism

The answer is a boolean from an exact algorithm, so the order the inventory
reads the battlefield cannot change it. The probe's sorted and unsorted
inventories agree to the mana on all 16,475 states it compared. **The
entries still come out in timestamp order**, because the window reads the
same inventory and its option list is order-observable (`CLAUDE.md`'s
determinism rule).

**What changes is what gets sorted.** Today's scan sorts every permanent on
the battlefield, then drops the ones the player does not control. The
inventory keeps the player's permanents first and sorts only those, about 9
of the board's 40–60. Timestamps come from one monotonic counter and never
tie (`battlefield_ordered`'s doc), so the order is exactly the full sort's.
The probe measured the whole sort at about 2,700 instructions an inventory
(the sorted and unsorted inventories' difference); sorting nine leaves a
few hundred of it. That is about 0.5 points saved, an estimate MA-1 measures.

### 3.6 Item 33's seam: an edge refused (decision 2)

**Restricted mana** (153 abilities). An entry carries its restriction, and
the check takes the purpose: the spell's types, or the activated ability's
source's (`SpendPurpose`, which exists). Entries the purpose refuses leave
the table before the check, and a second table is built when the inventory
holds any. **About 2,100 instructions a cost where present.** MA-1 built
neither the entry's restriction nor the purpose argument: the inventory
does not count restricted mana (`ManaOutput::special`) and reads only the
pool's unrestricted mana, so it never offers a payment that needs it. Both
come with item 33's build.
- **Snow** is §3.2's seventh bit.
- **The pool** is read through one function, today
  `ManaPool::unrestricted_by_type()`, which leaves restricted mana out (the
  inventory's pool entry and `remaining_cost_after_pool` call it; the
  payment's generic split still reads `available()`). Item 33's per-unit
  record replaces it with the units the purpose may
  spend, which `amount_for` already computes for the special atoms.
- **What this design builds:** the entry's restriction and the purpose
  argument, with item 33's build (`roadmap-v2.md` `mana-provenance`), which makes
  restricted mana spendable; until then MA-1 leaves it uncounted.
- **What item 33 builds:** the per-unit record, the payment that reads it,
  and the source axis on the generic split. Designed 2026-11-13 in
  `#11-mana-provenance-what-a-unit-of-mana-keeps`: the entry's restriction
  and the purpose argument are its second row's, and snow's bit its first's.
  There the purpose is the payment's `EventCause`, which the engine already
  carries, and `SpendPurpose` goes with the rest of the dormant types.

### 3.7 X (decision 6)

**Two questions, answered at two times.**
- **Is it offered at all?** Asked at every priority point: is some legal X
  payable? For a plain X spell the smallest legal X is the cheapest, so the
  check tests that one. That is X = 0, or X = 1 for the 17 cards that print
  "X can't be 0" (`o:"X can't be 0"` with the census's corpus filter). The X
  pips have no demand at the gate.
- **Which X?** Asked once, at 601.2b: `ChooseXValue` offers every legal X,
  and the player or agent picks. The X = 0 the check tested is not the X
  that gets cast unless it is the one picked. A learning agent sees Blaze
  with every X from 0 to its bound, and how it values X = 0 is its policy,
  as it values any legal but weak play (`backlog.md` §2.22's rule 2).

**The legal set is an intersection, and it need not be an interval.**
- **Payability gives an interval**, from the minimum to the bound. The bound
  is the largest X whose total the inventory covers. It is read in closed
  form, not searched (amended at #259's review: a search cannot survive a
  loop's mana, CR 731.2a; as built, below).
  X adds only generic mana, and no cost change's size depends on the
  spell's X, so the total is the same pips beside `a − r + kX` generic,
  read off 601.2f's own arithmetic in `cost_determination`, and the
  inventory says once how much generic it can pay beside those pips. The
  `r` is why the arithmetic is 601.2f's and not the printed cost's: a
  reduction can absorb part of X, so at X = 0 a `{1}` reduction on
  `{X}{R}` is wasted, and at X = 1 it is not. A larger X never costs less
  while no cost change's *applying* depends on X either, so this part is an
  interval. One whose filter reads the spell's mana value breaks both
  (below): on the stack the mana value counts X (CR 202.3e), and Stonecoil
  Serpent under Krosan Drover costs `{5}` at X = 5 and `{4}` at X = 6. No
  filter can read a mana value today. **The owner's condition** (2026-10-17,
  at #259): if the bound ever costs materially, the engine falls back to
  no bound, the payment deciding, while the algorithm improves (below).
- **Other rules cut it.** A loyalty ability's −X can be no more than the
  loyalty (CR 606.6). "X can't be 0" sets a minimum. 64 cards name a card
  with mana value exactly X (`o:"mana value X" -o:"mana value X or less"
  -o:"mana value X or greater"`). Where that card is a target or a choice
  (Detonate, Disembowel, Ashiok's −X), X is cut to the values the candidates
  have; where it is "each" (Dauntless Dismantler), nothing is cut. Tamiyo,
  Compleated Sage's −X targets a nonland permanent card with
  mana value X in its controller's graveyard. With cards of mana value 0, 1,
  4, 6 and 7 there and seven loyalty, its legal set is {0, 1, 4, 6, 7}.
  Detonate's `{X}{R}` is both: the artifacts' mana values, cut by the bound.

**So `ChooseXValue` gains a set form.** It is a `pick_n` over the legal
values when the set is not an interval, and today's `(min, max)`
`pick_number` when it is. The gate is "the set is not empty". The why names
each cut (§3.13). The random agent's land count goes, because the engine
bounds it.

**Owners.** MA-2 builds the set shape, the payability interval and the
printed minimum, with a fixture whose set has a gap. Each other cut lands
with its surface and reads MA-2's shape:
- the loyalty −X, with `remaining-costs`'s counter arms (an amount that can be X);
- a target's mana value, with the first card that prints it, in C;
- X in an activated ability's mana cost, `hybrid-and-phyrexian-payment`'s;
- X in an additional cost, `remaining-costs`'s amount shape;
- a "can't cast" that names a quality X changes (CR 601.3a: Void Winnower's
  even mana values), with `cant-cast-play-or-activate`'s cast prohibitions. `cant-effects-architecture.md`
  §4.3 plans a reversal after the announcement; the set form lets the same
  check cut instead (§9 says why both are the CR's);
- a minimum that depends on how the spell is cast (Light Up the Night's
  flashback: "If you cast this spell this way, X can't be 0"), with its card.

The base 601.2b chose decides whether X is announced at all (amended at
#259): an alternative cost with no X leaves 0 the only legal value (CR
601.2b announces "a variable cost that will be paid"; 107.3b says the same of
an effect's free cast), and an alternative cost's own X is expanded as the
mana cost's is (107.3a).

**The most, as built (MA-2, 2026-10-18).**
- **The total in X.** X adds only generic mana, and no cost change's size
  depends on the spell's X: `representable` refuses an X in an increase or a
  reduction, and a "where X is" amount such as Golden-Tail Trainer's is its
  own X (CR 107.3c), read once at 601.2f. So 601.2f's arithmetic, run on
  `cost_determination::ManaComponent`, gives the same pips P at every X
  beside `max(F, a − r + kX)` generic: a the generic besides X, r what the
  reductions take off, k the X symbols, F a Trinisphere-like floor. The cast
  evaluates the same component at the announced X.
- **The budget.** `ManaSupply::max_generic_payable_alongside` answers once
  how much generic mana the inventory pays alongside P, G: the existence
  check's own walk with a maximum at its leaves, since generic mana enters
  Gale's condition only through the total; past a pool multiplier, each
  leaf's total is carried forward through its transform, inverted. The most is
  `floor((G + r − a) / k)`, when F ≤ G; "X can't be 0" moves only the least.
- **Checked twice.** A property test runs the budget against MA-1's
  exhaustive search on 3,000 random boards, and in debug builds
  `oracle::x_values` asks the gate's own predicate at every X up to a small
  bound and eight past it.
- **One value is not asked**, and none reverses the cast at 601.2b, before a
  window opens for a payment that cannot happen.
- **The owner's condition, as measured:** the bound's inclusive share in
  callgrind past half a point (0.5% of instructions per decision, §2.2's
  unit) on `close_out.py`'s boards falls back to no bound. Every close-out
  that changes the inventory or registers an X card reads it. MA-2's whole
  change read +0.07% on the Commander board.
- **Owed.** A cost change that applies by a spell's mana value (Krosan
  Drover, Thryx) makes the total piecewise in X: a larger X can cost less
  (Stonecoil Serpent under Krosan Drover costs `{5}` at X = 5 and `{4}` at
  X = 6), the payable values gain gaps, and the least need not be the
  cheapest. No filter reads a mana value, and `gather`'s `applies_to`
  asserts in debug that none applying to an X spell does
  (`codebase-state.md` item 274). With `GameNumber` the bound stays the
  inequality `kX + a − r ≤ G`; for k ≥ 2 the most is a floor quotient the
  planned type cannot hold (`backlog.md` §2.28). The set form has no
  producer until the first cut (item 273).

### 3.8 The window reads the inventory

The window reads its offer once (`ManaAbilityWindowOffer`, as built in MA-1): the
player's mana abilities in timestamp order, one per definition. Each prompt
re-asks only each ability's costs, a `{T}`-only one by its tapped flag, since
its summoning sickness moves only with the epoch; the offer is read again
when the epoch moves (a sacrifice). **The option order stays timestamp, then
ability**, so the agent's stream does not move. It offers what it always has
and whatever else the check counts, so a payment the check found is one the
window can make: Doubling Cube, which no window had offered.
- **Saves:** the warm re-enumerations, about 4,500 instructions a decision
  (0.7 points). The cold first scan stays: the cast's move to the stack
  invalidated those frames for every reader.
- **Not this design's:** the random agent's preference (`mana_window_preference`,
  3,800 a decision) scans again for itself. Handing it the inventory through
  the prompt is a harness change. Its scan reads each ability's production on
  the board, as the check does (`available_mana_sources`), or it declines a
  window a payment covers: a land Wild Growth enchants makes the `{G}` it owes.

### 3.9 Paying with permanents or cards

These are payments, not mana (702.51b and its siblings). The check counts
them as entries for the one cost they serve:

| Keyword (cards) | Entry | Notes |
|---|---|---|
| convoke (104) | each untapped creature: one, of its colors, or generic | a mana creature keeps its one entry, with its types widened |
| improvise (23) | each untapped artifact not already an entry: generic only | an artifact that taps for mana is no worse as mana |
| delve (28) | each card in the graveyard: generic only | `remaining-costs`'s exile-from-graveyard cost shares the same cards |
| waterbend (28) | artifacts and creatures, for the waterbend cost's own generic only (701.67b) | an eighth bit its generic pips carry |
| assist (16) | another player's inventory, generic only | over-offers, since that player may refuse (702.132a) |
| offering (6) | one check per creature it may sacrifice, its mana cost as a reduction | an additional cost that reduces, 601.2f's pipeline |

**The payment** (which creatures tap, which cards are exiled, for which pips)
is asked in `plan_payment`, before anything is performed
(`cost-architecture.md` §3.12). A creature tapped for mana in the window
cannot then convoke. The inventory's check covers both uses, and the
payment's own check runs after the window. **Injected prices:** convoke
about 2,600 instructions a cost and delve or improvise about 2,350, each
including a scan that the build folds into the inventory once per priority
point.

### 3.10 What a mana ability makes (decision 5)

- **A production spec.** `ManaOutput`'s type becomes one of four forms: a
  printed type; a choice among types (made once for the whole amount, as in
  "two mana of any one color" or "{R} or {G}", or once per mana, as in "in
  any combination of colors"); the type recorded as the source entered; or
  a type the named permanents could produce. **The choice is asked as the
  ability resolves (`backlog.md` §2.19), never in the window**: the window offers the
  ability. A new `ChoiceKind` carries the window's remaining cost when it is
  asked inside one, so an agent can pick the color a pip needs. Everywhere
  stays five abilities: it has every basic land type, and CR 305.6 gives it
  five.
- **Riders.** A mana ability's effect is a sequence. Its `ProduceMana` atoms
  resolve as today, and its other atoms (a painland's damage, Chromatic
  Sphere's draw under the frozen text, §1) resolve through the general
  resolver inside the same 605.3b
  resolution, each its own proposal through the chokepoint. **About 22,500
  instructions an activation**, the measured cost of a production's own
  proposal (`resolve_mana_effect`, 4,417 calls).
- **A mana ability's own mana cost opens a window** (605.3a: "activating an
  ability that requires a mana payment"). `ironworks-loop`'s Ironworks loop needs it for
  Chromatic Sphere's `{1}` (`roadmap-v2.md` `ironworks-loop`), and Signets need it too.
- **The as-enters record's mana half** (`backlog.md` §2.2). A color chosen as the
  permanent enters (CR 614.12a) travels in `EnterMods`, so the look-ahead
  frame reads it. It is kept on the permanent, never in `CopiableValues`
  (707.6), and it leaves with it (400.7). The four constraints §2.2 sets
  shape it.
  Its other kinds (a creature type, which Cavern of Souls and item 33 both
  need; a player; an anchor word) arrive with their first card.
- **"Could produce"** (106.7) is the union of types the named permanents'
  mana abilities would make now. Two Reflecting Pools need a fixpoint: types
  only grow, so at most six rounds. It is memoized with §3.4's scan.

At the gate each form is a type set: a printed type is one bit, a choice is
its set (once for the whole amount means a one-choice entry), "chosen" is
the recorded bit, and "could produce" is the union.

### 3.11 Item 211: mana abilities at a priority question (decision 4)

CR 605.3a lets a player activate a mana ability whenever they have
priority. The engine offers one only in a cost's window. Four shapes, priced
on the board:

| Shape | CR-complete | New decisions a game | Options added | Engine cost | Fixtures and stream |
|---|---|---:|---|---|---|
| **A.** Every mana ability an option, every seat | yes | **1,749** (+278% on 628) | ~19.7 where offered | an inventory at all 58,921 points, 41,015 more than the check takes: about +60K a decision (+9.6%) | prompts come back to every scripted test with an untapped land at a priority point, A6j's migration in reverse; the stream moves everywhere |
| **B.** One entry, "add mana", opening 605.3a's window with the existing prompt, every seat | yes | 1,749 | 1 | as A, or less with a scan that stops at the first ability (not priced) | as A |
| **C.** B, offered to a seat that asks for it (`SeatMode`, A6j's mechanism). A person's client asks; `fuzz_games` and the tests do not | yes, for any seat that asks | 0 by default | 1, to seats that ask | 0 by default; A's price for a seat that asks | none by default |
| **D.** Offered only where the ability does more than add mana (a cost beyond `{T}`, a rider, a watched tap) | **no** | 138 | a few | a cheap scan | moves the stream |

- **The cases behind the counts.** 34,982 of the board's 54,412 forced
  priority points have an activatable mana ability. Shape A's options average
  19.7 because Everywhere is five.
- **D is not the CR.** In response to Stone Rain, a player floats mana from
  the doomed land and casts a sorcery with it once Stone Rain resolves. That
  needs a plain land offered at priority.
- **Recommended: C.** It keeps the engine CR-complete for every seat that
  asks, and costs nothing where no one does. It follows A6j's precedent:
  `SeatMode` is how a seat's standing policy reaches the engine.
- **What "declining" means here.** The offer is not a cost the check might
  refuse: it is the action "add mana now", and any untapped land makes it
  available, so it is there at nearly every priority point. A seat that
  never floats mana answers it with `Pass` every time. Under A or B,
  `fuzz_games`' agent would be asked 1,749 more times a game, and a policy
  that always answers `Pass` makes each of those an answer the engine
  could have predicted. Building the offer is the inventory at every
  point, and asking it is a round trip. Under C that seat never asked, so
  neither cost is paid (`backlog.md` §2.22's rule 2: a bot's seat takes its
  policy at construction). Built in `mana-payment-solver`, beside the person's-seat solver.

### 3.12 The solver: a payment, not existence

Item 162's second customer is `backlog.md` §2.22 rows 6 and 7. From the
inventory it finds an assignment of entries to pips by augmenting paths
(dozens of entries at most), ordered by the owner's preference
(2026-09-30):
1. toggled colors first;
2. then the payment that leaves the most of the hand castable: each
   candidate payment's leftover inventory asks the existence check about
   every other card in hand, which is why that check has to be cheap;
3. then the least flexible source first.

**It runs once per payment, on a person's seat**, never at a priority
question. About 20 candidate payments times a hand of 7 times 100 is about
14,000 instructions a cast. The decorator answers `ManaAbilityWindow` while
the cost is uncovered, which is disjoint from `ManaWindowStop`'s predicate
(§2.22's rule 3), and `GenericManaAllocation` when the pool has surplus. A person
gets it under the toggle; `fuzz_games` gets a flag, off by default
(`backlog.md` §2.18's floor 1 note).

**Two inputs for the build** (2026-11-04). A mode that will not auto-pay with
sacrificial mana, so the solver never spends a permanent the player would not
have chosen to lose. And a provider decorator that remembers a replacement
answer, so a repeated choice is asked once with no engine change.

### 3.13 The why of the three mana prompts

`setup-architecture.md` §7c and `cast-census.md` §9 give these kinds typed
reasons "with item 162's build":
- **`ManaAbilityWindow`:** the refusals are the window's: for each of the
  source's mana abilities the window does not offer, the `CannotPay` its
  costs return when they cannot be paid, saying why. A permanent whose
  abilities can't be activated at all gets that one refusal instead
  (`ui/why.rs`' `window_refusals`).
- **`ChooseXValue`:** the bound and each cut (§3.7). "X above N leaves the
  cost unpayable: N plus the cost's other pips is all the mana this board
  makes." "X = 2 is not offered: no artifact has mana value 2."
- **`GenericManaAllocation`:** the clamp. A bucket's maximum is the pool's
  mana of that type less the pips that need it.

The first landed with MA-1, the second and third with MA-2 (`ui/why.rs`' `x_refusals` and `generic_split_refusals`).
MA-2's dev GUI notes PR added the window's sources it does not offer *yet*:
a mana ability whose other costs can be paid while the pool lacks its mana
(`ui/why.rs`' `not_offered_yet`), which a client draws beside the options so
that a Doubling Cube waiting for its {3} does not look unusable.

---

## 4. The families: algorithm, price, build

**Price** is callgrind instructions per decision on the budget's board,
12,560 decisions, and a point is §2.2's unit, one percent of `main`'s. Where a family is on the board it is measured. Where it is
not, its prototype ran on every checked cost of the board's real states
(33,550 of them), and the number is an upper bound: what the family would
cost if every cost the check read carried it.

| Family (`cast-census.md` §12) | Cards | Algorithm | Price | Build |
|---|---:|---|---|---|
| The plain check: several mana at once, listed types sharing a `{T}` | 155, 380 | §3.1–3.2 | **30.3K a decision with §3.4 (26.3K the inventory, 3.6K the scan, 0.4K the checks), against 45.6K today: −2.4 points** | MA-1 |
| Triggered mana and its replacements | 19, 10 | §3.4, memoized | 3.6K (in the line above); 10.9K unmemoized | MA-1 |
| One choice for several mana: "any one color", a multiplier on a multi-type entry | 54 | §3.3 enumeration | 2,800 a cost where present; bound 9.5K | MA-1 (the multiplier, on the board); `any-color-plus-painlands-plus-signets` (the printed form) |
| Mana-fed: converters and filters | 175 | §3.3: Signet-shaped exact by a base-mana pip; identical ones counted; other shapes ordered, up to three | 3,600 a cost where present; bound 9.6K | MA-1 (Doubling Cube, registered, exact); `any-color-plus-painlands-plus-signets` (Signets) |
| Alternatives of different sizes | — | §3.3: each alternative tried, dominated ones dropped | not measurable on the board (4 inventories) | MA-1 |
| Sacrifice-fed, another permanent | 26 | §3.3 count less the cost's own | one candidate count an inventory where present | MA-1 (Ironworks, registered) |
| Sacrifice-fed, itself | 117 | an ordinary entry | 0 beyond the plain check | MA-1 |
| Variable amount; the doubled pool | 60; 1 | evaluated at the inventory (605.3b) | one `evaluate_amount` a source | MA-1 |
| One mana of any color; Treasure makers | 306 + 21, 347 | a five-type entry; the choice at resolution (§3.10) | 0 at the gate; one prompt an activation | `any-color-plus-painlands-plus-signets` |
| Painland riders | 129 | gate unchanged; the rider through the chokepoint | 0 at the gate; about 22.5K an activation | `any-color-plus-painlands-plus-signets` |
| The chosen color | 32 | the recorded bit; `backlog.md` §2.2's record, its mana half | 0 beyond a read | `chosen-color-plus-could-produce` |
| "Could produce" | 14 | the union, a fixpoint, memoized | one pass over the named set a memo miss | `chosen-color-plus-could-produce` |
| Restricted mana | 153 | §3.6, an edge refused | 2,100 a cost where present; bound 5.6K | item 33's build (`roadmap-v2.md` `mana-provenance`); MA-1 leaves restricted mana uncounted |
| X | 527 | the smallest legal X at the gate; the legal set at 601.2b, the bound in closed form, cut by the other rules (§3.7) | 159 a cost; bound 0.4K | MA-2; each cut with its surface |
| Hybrid, `{C/W}` | 582, 1 | a two-type pip | 0 beyond the plain check | `hybrid-and-phyrexian-payment`, against §3.2 |
| Mono-hybrid, Phyrexian | 20, 36 | counts enumerated | 1,300 a cost where present; bound 3.6K | `hybrid-and-phyrexian-payment`, against §3.2 |
| Convoke | 104 | §3.9 | 2,600 a cost where present; bound 6.9K | `convoke-delve-improvise` |
| Improvise, delve | 23, 28 | §3.9, generic only | 2,350 a cost where present; bound 6.3K | `convoke-delve-improvise` |
| Waterbend | 28 | §3.9, an eighth bit | as convoke | `convoke-delve-improvise` |
| Assist, offering | 16, 6 | §3.9 | another player's inventory; one check a candidate | `convoke-delve-improvise`, or C with the card (§6) |
| `{S}` | 2 | §3.2's seventh bit | 0 beyond the plain check | item 33 (`mana-provenance`) |
| The window's inventory | — | §3.8 | −4.5K | MA-1 |
| Mana abilities at priority (item 211) | — | §3.11, shape C | 0 by default | `mana-payment-solver` |
| The solver and its decorator | — | §3.12 | ~14K a cast, a person's seat only | `mana-payment-solver` |

The counts are the census's (`plans/references/cast-census.md` §5, §7, §8),
reproduced from its cache this session (`cast-census.py --no-fetch`). Three
rows are split here, by the census's own walk over its cached corpus, the
production text from its first "add", first pattern wins:

| Census row | Split | Pattern |
|---|---|---|
| one mana of any color or type (391) | 54 one choice for several | `any one color` |
| | 21 a choice per mana | `any combination of colors\|in any combination` |
| | 306 one mana | `one mana of any color` |
| | 10 the colors of something else | `of that color\|of either of\|any of (the\|its\|~'s)\|any colors` |
| costs mana (175) | 128 tap, once a turn | `{T}` in the cost |
| | 47 repeatable | the rest |
| costs a sacrifice (143) | 117 sacrifice themselves | `sacrifice (~\|this)` in the cost |
| | 26 sacrifice another permanent | the rest |

---

## 5. Measuring MA-1: a change that moves the agent's stream

Moved to `plans/archive/mana-architecture-landed.md`, "MA-1", when it
landed: five commits, each an arm that answers one question, read in
`fuzz-record.md`'s MA-1 block. What carries to MA-2 to `mana-payment-solver`: a change that
moves what the priority question offers reads the budget off a cost arm
(today's answers, the new machinery run black-boxed), and "Actions
reversed" (CR 732.1's word) prices an offer no payment covers, each one a
re-ask that moves the agent's stream.

---

## 6. The build, sized

Each PR carries a consumer and lands in or under the band
(`engineering-practices.md#sizing-a-phase`). Sizes are code plus tests, and the
card-by-card hunt runs in each PR's brief.

| PR | Shape | Size | Consumer | Closes |
|---|---|---|---|---|
| **MA-1, the inventory and the exact check** | Landed 2026-10-05 (#225): §10, and the archive's "MA-1" for the shape as sized | | | item 162's offer customer |
| **`mana-ability-costs`, every cost a mana ability can have, in the check** (MA-1's review, 2026-10-06) | Every cost a mana ability can have, read per `Cost` arm with no wildcard into what one use spends: its tap (once a payment, since only `{Q}` untaps inside one), itself, counters (their count over N uses), life (one budget with the action's own life payment, CR 119.4), others' sacrifice (one fodder pool), mana in (a pool multiplier's input; a converter's is `any-color-plus-painlands-plus-signets`'s rule). Sorcery timing read, and CR 602.5's "activate only if" conditions once `permission-architecture.md` gives them a surface, in the order one window can change them (Mox Opal beside Krark-Clan Ironworks). The fold of replacement effects by state rather than order: 2^n·n! leaves at six effects become at most 2^n states. The choice search as a dynamic program over what the cost's pips still need, exact with no cap. **Its first commit is the stress board**: Nyxbloom Ancient registered beside Mana Reflection, fifteen distinct dual lands, a pool multiplier and three- to five-color costs, read in callgrind before the dynamic program and after | ~350–450 engine, ~300–400 tests: 650–850 | the stress board; the 116 printed cards whose mana costs life, counters, a tap with another sacrifice, or `{Q}` | item 213; MA-1's review, themes C and D |
| **MA-2, the legal values of X** | Landed 2026-10-18 (#259): §10, and the archive's "MA-2" for the plan as sized | | | `cast-census.md` §8's X row |
| **`any-color-plus-painlands-plus-signets`, a color chosen at resolution, a mana ability that does more, a mana ability with a mana cost** | §3.10's spec type and its sweep of `ManaOutput` sites; the choice at resolution, a `ChoiceKind`; riders through the resolver; a mana ability's mana cost opening a window; §3.3's converter rules, with the combination cap stated as a cost and counted in fuzz. Cards: Birds of Paradise, a painland (Llanowar Wastes), Gilded Lotus ("three of any one color"), a Signet, a Treasure maker with `backlog.md` §2.27's Treasure | ~700 engine, ~800 tests: 1,500–2,000 | five cards; the Ironworks loop's three facilities (`roadmap-v2.md` `ironworks-loop`) | `backlog.md` §2.19 |
| **`chosen-color-plus-could-produce`, the chosen color and "could produce"** | `backlog.md` §2.2's record, its mana half (the choice at entry, `EnterMods`, the permanent's field); "could produce" with its fixpoint. Cards: Thriving Grove, Coldsteel Heart, Exotic Orchard, Reflecting Pool | ~550 engine, ~650 tests: 1,200–1,500 | four cards | `backlog.md` §2.2's mana half |
| **`convoke-delve-improvise`, paying with permanents or cards** | §3.9: the entries; the payment prompt in `plan_payment`; the taps and exiles through the chokepoint. Cards: Stoke the Flames, Treasure Cruise, Reverse Engineer, a waterbend card. Assist and offering go here if the hunt keeps it in band, else to C with a card each | ~900 engine, ~1,000 tests: 1,900–2,400 | four cards | `cast-census.md` §7's payment row |
| **`mana-payment-solver`, a person's seat: the solver and item 211** | §3.12's solver and decorator, wired in the dev GUI behind the toggle, with a `fuzz_games` flag off by default; §3.11's shape C (a `SeatMode` field, the "add mana" entry, CR 605.3a's window at priority). A GUI PR: `engineering-practices.md#the-dev-gui`'s review | ~600 engine and client, ~600 tests: 1,100–1,400 | the CLI and dev GUI person's seat | `backlog.md` §2.18's solver (§2.22 rows 6 and 7), item 211 |

**Ordering.**
- **MA-1 landed before SU-8** (decision 7). It removed the wrong offer
  SU-7's click script met on the review board, and wrote the why's
  `ManaAbilityWindow` line SU-8 would have read off its inventory.
- **MA-2 moved up to right after TR-5b** (the owner, 2026-10-17, at #258):
  TR-5b's Protean Hydra is an X spell the gate cannot offer. Its inputs are
  built, MA-1's check and the cost determination's X, and `mana-ability-costs`'s mana
  ability costs are no part of its X arm.
- **`mana-ability-costs` first** (the owner, 2026-10-06, at #225's review): it widens the
  check that the mana phases still to come extend. It is not urgent for today's games: the
  choice search never passed 6 leaves in about 150,000 checks over the four
  fuzz boards (2026-10-06). It is the guarantee for v1's Commander boards,
  where Nyxbloom Ancient and many lands do meet. It lands before
  `permission-design`'s builds since #314, so its CR 602.5 read rides
  `command-zone-casting`, the first build of that surface.
- **MA-2 to `mana-payment-solver`** go in `roadmap-v2.md` B, before C. **`any-color-plus-painlands-plus-signets` before `ironworks-loop`**,
  the Ironworks loop (TR-7's until 2026-10-13), which needs any-color mana, a
  draw inside a mana ability, and a window inside a mana ability's activation.
- **`hybrid-and-phyrexian-payment` after MA-1**: its gate half is §3.2's arms.
- **Item 33's design pass after MA-1**: its seam is §3.6. **Its build right after
  `mana-ability-costs`** (the owner, 2026-11-05, #312): the source axis reshapes the
  generic split, which every payment row after it would otherwise build first.
  **The design comes before `costs-and-choices-at-resolution`, and those three rows
  right after `resolution-recipients-and-amounts`** (the owner, 2026-11-06, #314): a
  cost paid in mana as an effect resolves, Rhystic Study's, goes through the cast's
  `plan_payment`, so the design says whether `Primitive::Pay` needs its build first.
  **The design's answer is no** (`#decision-9-whether-the-build-comes-before-costs-and-choices-at-resolution`,
  2026-11-13, for the owner's review), and its three rows keep this place.

---

## 7. The decisions

The brief named six, and a seventh, the route, was the owner's call. **All
seven were taken as recommended at #224's review** (the owner,
2026-10-05).

1. **Where the design lives.** This document (§0). `CLAUDE.md`'s row gains
   `mana` (106/605/601.2g–h's existence half, `MA-*`).
2. **Item 33's seam.** The check refuses an edge: an entry's restriction
   meets the cost's purpose, and `{S}` is a bit. The per-unit record, its
   payment and the source axis are item 33's own pass (§3.6).
3. **The algorithm.** As recommended: one entry per shared cost, amounts as
   capacities. Feasibility is read by Hall's condition over a 64-entry
   table (MA-1 built it over a list of (type set, amount), §3.2), about
   100 instructions a cost. §3.3's shapes are exact: one-choice
   entries and alternatives tried, Signet-shaped converters by a base-mana
   pip, Doubling Cube by its input's split. Only enumerations past a stated
   cost cap over-offer. On the budget's board the
   exact check is about 2.4 points cheaper than today's greedy, measured,
   because the inventory is taken once per priority point, not once per
   card.
4. **Item 211.** Shape C (§3.11): CR-complete for any seat that asks, and
   nothing asked of a seat that does not.
5. **The production shapes.** A spec with four forms; the choice at
   resolution; riders through the resolver; `backlog.md` §2.2's record,
   its mana half built in `chosen-color-plus-could-produce` in §2.2's shape; "could produce" with a fixpoint (§3.10).
6. **X.** As recommended, with three refinements (§3.7). The gate tests
   the smallest legal X, which is 1 on 17 cards. The bound bisects over the
   previewed total, because a reduction can absorb part of X (replaced at
   #259's review by the closed form, §3.7). And the legal
   X is a set that other rules cut, so `ChooseXValue` gains a set form for
   Tamiyo's −X and Detonate.
7. **The route.** MA-1 before SU-8; the rest in B before C, `any-color-plus-painlands-plus-signets` before
   `ironworks-loop` (§6).

---

## 8. Out of scope

- **CR 732.1's reversal** (item 72). It is `backlog.md` §2.22 row 9's,
  slotted to `ironworks-loop` (`codebase-state.md` item 72). The
  solver's "reverse all when the solver made the taps" is row 9's, not `mana-payment-solver`'s.
- **The payment of hybrid, Phyrexian and `{C/W}`** is `hybrid-and-phyrexian-payment`'s. Of `{S}` and
  restricted mana, item 33's.
- **CR 609.4b's "spend as though it were mana of any color"** (49 cards) is
  `backlog.md` §2.24's, with `hybrid-and-phyrexian-payment`. At the gate it widens every entry's types,
  one line in §3.1, when it lands.
- **Mana that does not empty** (24) and mana that does something when spent
  (8) are item 33's.
- **A spell or non-mana ability that adds mana** (19, Dark Ritual) is
  resolution, not the check, and already built.

---

## 9. Found while designing

- **The timestamp sort is still the largest single lever.**
  `battlefield_ordered`'s sort is 8.97% of the budget's board, item 138's
  lever 6. The inventory sorts only the player's own permanents (§3.5),
  about 0.5 points; a maintained order would save most of the 9% everywhere.
- **Item 211's verdict did not count as wrong.** "Reachable — wrong:" lacks
  the word `check_state_of_play.py` reads ("wrong today"), so the board
  filed it as not wrong. Corrected in this PR, beside item 162's new
  verdict. The board's wrong-today count goes from 3 to 5.
- **The window's first scan per cast is about 214,000 instructions**
  (derived from §2.2's call counts), a frame
  re-walk after CR 601.2a's move. It is not this design's to remove: every
  reader after the move pays it once. It is why the window's saving (§3.8)
  is 0.7 points, not 6.
- **The window's prompt has no subject at a priority question.**
  `ChoiceKind::ManaAbilityWindow` names the spell or ability being paid for.
  Shape C's window at priority pays for nothing, so `mana-payment-solver` makes that field an
  `Option` or adds a kind, and decides which with `ChoiceKind::subject()`'s
  match (A4j).

Found by MA-2's plan (2026-10-17, at #259):
- **The census files an activated ability's `{X}` as unowned.**
  `cast-census.py`'s cost row "mana with {X}" says "an ability's X is never
  asked", owner none, while `cost-architecture.md`'s `hybrid-and-phyrexian-payment` row claims it
  (`activate_ability` never asks, `x_value: None`; 145 cards). The owner
  column is corrected at MA-2's close.
- **Two plans for Void Winnower's X.** `cant-effects-architecture.md` §4.3
  reads CR 601.3a as "begin the cast, then reverse an announced X the
  prohibition forbids" (Endless One with X = 5, not 6, from the card's
  ruling). With the set form, the same check can cut the even mana values
  out of what is offered instead. Both are the CR's: 601.3a lets the cast
  begin, and 601.2e reverses an announcement that makes it illegal. Offering
  only what can be completed is `cost-architecture.md` §3.6's preference, so
  §3.7's owners list the cut with `cant-cast-play-or-activate`, and the choice is `cant-cast-play-or-activate`'s.
- **Mana value with X on the stack** (CR 202.3e) stays `ManaSymbol::X => 0`
  everywhere. `hybrid-and-phyrexian-payment`'s row owns it; no registered card reads a spell's mana
  value on the stack, so it is not reachable when X spells become castable.
- **"X can't be 0" is a fixture.** Three paper cards print it on a mana
  cost's X (`o:"X can't be 0" m:{X} -is:digital`, 3, Scryfall 2026-10-17):
  Ertai's Meddling needs delay counters, Mind Grind revealing until a count,
  and Light Up the Night's applies only when it is flashed back (§3.7's
  owners). §3.7's 17 is the census's corpus filter; Scryfall without it
  gives 21, most of them suspend's, a kicker's or an activated ability's X.
- **`ManaCost` holds generic mana as one symbol per mana** (found at #259's
  review). 601.2f's step 1 expands X with `repeat_n(Generic, x_value *
  x_count)` (`total.rs`), and `apply_reduction` rebuilds the generic part
  the same way, so a cast's cost allocates as many symbols as its mana. On
  today's boards that is tens. `GameNumber` (`backlog.md` §2.28) needs the
  generic part held as a count, beside the colored symbols, whichever way
  decision 2 goes; a line for §2.28 at MA-2's close. Pool multipliers make
  it far more: on the dev GUI's board-1, four Doubling Cubes beside Nyxbloom
  Ancient, an X of 11,318 was announced and paid, and beside Nyxbloom each
  Cube multiplies what the pool can reach by about four, so a dozen reach
  about 10^9 (#260, which fixed the byte counts an X past 255 overflowed).
  *Annotated at the item 6 audit (2026-10-22):* MA-2 then held the
  arithmetic's generic part as a count (`ManaComponent`, §3.7), but the
  `ManaCost` it hands on (`ManaComponent::at`, `ManaCost::with_x_as`) still
  writes a `Generic` symbol per mana, so the point stands.
- **`GameNumber`'s plan has no division** (found at #259's review). The
  most X for a cost with two or more X symbols is a floor quotient of the
  mana the board makes (§3.7), and `decision-provider-refactor.md` §9's
  planned `Finite`, `Shortcut` and `Relative` hold products and offsets
  only, so a player cannot announce "half my pool". It needs a
  floor-quotient node compared through `k·c ≤ E`; a second line for
  §2.28 at MA-2's close.
- **A cost change can apply by the spell's mana value** (found at #259's
  review, from Golden-Tail Trainer). On the stack the mana value counts X
  (CR 202.3e), so Krosan Drover and five other printed cards make the
  total piecewise in X, and §3.7's "a larger X never costs less" fails.
  No filter can read a mana value yet; §3.7 and `codebase-state.md` item
  274 say what the first one must bring and how `filter_reads` keeps it from
  passing silently.

---

## 10. Landed

### MA-1 — the inventory and the exact check — ✅ landed 2026-10-05

**What shipped.** #225, §6's first PR. `oracle::mana_supply` takes the
inventory once a priority point, through a lazily filled cell
`candidate_priority_actions` hands down, and answers by Hall's condition over
the six types (§3.2). One-choice mana and ways of different sizes are tried a
choice at a time; Krark-Clan Ironworks counts once per artifact it can
sacrifice; Doubling Cube is a transform of the demand; a cost's own `{T}` and
sacrifices come out of what it can make; and each tap's production is read
after the board's replacement effects and triggered mana (§3.4), whose
watchers are memoized per layer epoch with a debug audit. `can_cast` and
`can_activate` ask it where `find_mana_sources` stood, which is gone, and
`can_cast` returns `Ok(())`. CR 601.2g's window reads its offer once a window
(`ManaAbilityWindowOffer`) and offers what the check counts, the Cube included, and the
why names the cost that keeps a mana ability out of it. "Actions reversed"
counts what the engine rewinds. Ironworks and the Cube are pooled.

**What moved on the way in.** Four things the design did not have, each
amended where it stands: retypes are read (§3.4: ignoring one errs both
ways, since a Forest under Deep Water makes `{U}`); the Cube is a transform
of the demand (§3.3); what Ironworks sacrifices after the Cube's doubling
pays undoubled (§3.3); and the random agent's window view reads each
ability's production on the board (§3.8), without which it declined windows
a payment covered. `codebase-state.md` item 213 lists what the inventory
still reads one way. It landed at 1,583 code and 903 tests, against ~600 and
~800–950 sized.

**Measured.** `close_out.py` with six arms against #224's merge. Counter,
inert, window and cost `IDENTICAL` on every gameplay row, both pools, two
seats and four. The budget's reading, the cost arm, −1.96% instructions a
decision against −3.0 predicted; the window −1.22% against −0.6. Shipped:
"Actions reversed" 22.4 a game → 0.0 on the budget board, `Decisions` 628 →
547. Pooled, the new baseline: 2.3 a game, every one read a payment the
random agent did not make, none an over-offer (item 214).

→ `plans/archive/mana-architecture-landed.md`, "MA-1" (as sized, sized
against built, what the build changed); `fuzz-record.md`, MA-1's block;
`plans/traces/ma-1-a-payment-is-a-matching-not-a-count.html`, six boards.

### MA-2 — the legal values of X — ✅ landed 2026-10-18

**What shipped.** #259, §6's MA-2 row. The gate offers an X spell when its
least legal X is payable, 0 or 1 under "X can't be 0"
(`CardData::x_cant_be_zero`). CR 601.2b's `ChooseXValue` offers the values
`oracle::x_values` reads in closed form (§3.7): an interval by `pick_number`,
a set with a gap by `pick_n`, one value unasked, none a reversal at the
announcement. An alternative cost's own X is the announced X, and one with
no X holds it at 0. The random agent's land-count cap is gone, and it pays
through Doubling Cube (item 214). The why names X's bound and least and the
generic split's clamp; the scenario word `| x N` answers the prompt. Blaze is
registered and pooled, and Protean Hydra is castable.

**What moved on the way in.** The bound was planned as a search and is read
in closed form, decision 2 at the owner's review, with a fallback to no
bound if it ever costs half a point. A cost change that applies by mana
value (CR 202.3e) was found to make the total piecewise, and is guarded and
owed (item 274). `ChooseXValue`'s values are boxed, so `ChoiceKind` stays
48 bytes. The census had two question kinds without rows, now added. 994
code and 943 tests added, against ~600–640 and ~720 sized.

**Measured.** The Hydra-free pair, item 214's arm and the engine arm, read
IDENTICAL on every gameplay row, both pools, two seats and four;
instructions per decision +0.07% on the Commander board, against ±0.1%
predicted. "Actions reversed" on the pooled Commander board: 0.4 → 0.0 with
item 214, 0.1 shipped, none from an X cast. The reachability pass cast
Protean Hydra 169 and 112 times and Blaze 182 and 102, two seats and four,
200 games each.

→ `plans/archive/mana-architecture-landed.md`, "MA-2" (the plan as sized,
sized against built, what the build changed); `fuzz-record.md`, MA-2's
block.

---

## 11. Mana provenance: what a unit of mana keeps

> **Status:** design, 2026-11-13, under the owner's review on #323: nothing
> here is built. Round 1 (2026-11-13) settled decisions 6 and 7 and the
> reading inside decision 1, renamed the types of decisions 1, 3 and 4, and
> opened one question inside decision 1 and one inside decision 8. Decisions
> 2, 3 and 9 are still being discussed. It is `roadmap-v2.md`'s row
> `mana-provenance`, the design, for `codebase-state.md` item 33 (the tested
> restricted-mana subsystem that no payment calls).

### 11.1 Where it sits

**In a game:** a player taps a Snow-Covered Forest and a Forest and has two
green mana that are not alike. One can pay a `{S}`, the other cannot. Cavern
of Souls' mana casts only one creature type, and the spell it pays for can't
be countered. Hired Hexblade asks whether mana from a Treasure paid for it.
Leyline Tyrant keeps red mana from step to step. Each needs the pool to
remember more about a unit of mana than its type, and none works today.

**On the event path**, four places, in the order a unit meets them:
1. **Made.** A mana ability or a spell proposes `GameAction::ProduceMana`, and
   its arm in `perform_action` is the pool's one writer
   (`engine/actions.rs`). Today it adds a count by type and drops the source.
2. **Offered.** At each priority point the existence check asks whether the
   pool and the untapped sources cover a cost (`oracle::mana_supply`, landed
   as MA-1, the exact check). It reads only unrestricted mana.
3. **Spent.** At CR 601.2h `plan_payment` asks how the generic part is split
   across the pool's types, and `pay_costs` removes the mana and hands back
   what was spent, by type (`engine/costs.rs`). The spell's stack entry keeps
   that, and the permanent it becomes keeps it after (CR 400.7d).
4. **Lost.** As each step and phase ends, `empty_mana_pools` clears every pool
   by writing it directly (`engine/turns.rs`).

This design changes all four, and the three payment rows after it
(`hybrid-and-phyrexian-payment`, `convoke-delve-improvise`,
`remaining-costs`) build on what it leaves at the third.

### 11.2 What the engine has

T12b (the archived plan's label for the restricted-mana types) built
`ManaRestriction`, `ManaGrant`, `ManaPersistence`, a `special` list of
`ManaAtom` groups beside the plain pool, `SpendContext` and `pay_with_plan`,
with thirty unit tests, all in `types/mana.rs`. Production reaches none of
it: every `ProduceMana` passes an empty `special`, nothing builds a
`SpendContext`, and `pay_single_cost` pays from the plain pool. The plain
pool is a map from type to count, so a unit's source is gone the moment it
is added.

### 11.3 The intake

`python plans/check_state_of_play.py --slotted mana-provenance`, 2026-11-13.
"Sized" is each item's own line, re-counted against the tree in "The build,
sized".

| Item | What it is | Taken as |
|---|---|---|
| main item 33 | the dormant subsystem; the per-unit record; restricted mana; `{S}`; readers of a mana's source | decisions 1 to 4 and 6; rows **`snow-mana`** and **`restricted-mana-plus-spend-riders`** |
| "Before Layers" item 4 | mana kept past a step's end; Yurlok of Scorch Thrash's life loss | decision 5; row **`mana-kept-and-lost`** |
| main item 403, its second half | CR 500.5's emptying has no event, so Leyline Tyrant cannot keep its mana. Slotted to `tarkir-dragonstorm-decks`, not to this row | found here: the same build as the item above. Proposed to move to `mana-kept-and-lost` with decision 5, the gap census's record with it. Its first half, Seedborn Muse's untap, stays where it is |
| main item 135 | one ability making restricted and unrestricted mana at once, under a replacement that fixes the amount | decision 7, settled (the owner, 2026-11-13): it stays refused, re-slotted to `breadth-features` with its first card |
| main item 162 | the tap solver, which names this row | not taken: its solver is `mana-payment-solver`'s. Decision 2 says what that solver will answer |

**The rules' atoms** (`python plans/specdb.py next --rule`, for 106.4, 106.6,
107.4h, 400.7d, 500.5 and 601.2h, and for the rules that watch them):

| Atom | Scenario | Planned test |
|---|---|---|
| ATOM-107.4h-001 | `{S}` paid with mana from a snow source | `snow-mana`: Icehide Golem cast with a Snow-Covered Forest, and refused with a Forest |
| ATOM-107.4h-002, -003 | a generic reduction leaves `{S}` alone | `snow-mana`: a `{1}{S}` and a `{S}` cost under a "costs {1} less" effect |
| ATOM-106.3-001 | mana keeps its source in the pool | `snow-mana`, for what it observes: the snow `{R}` pays `{S}` and the other does not. Its "expected" line also names today's two-track storage, which decision 1 replaces, so the session file's line is reworded with the build if decision 1 is taken |
| ATOM-106.4-001 | the pool empties at each step and phase | `mana-kept-and-lost`, at the event it adds |
| ATOM-500.5-001 | effects expire, then mana empties | `mana-kept-and-lost`, the emptying half, as a partial: the expiry half is a duration's |
| ATOM-500.5a-001 | "until end of combat" ends with the phase | not this row's: a continuous effect's duration. Its mana cousin, firebending's mana kept until end of combat, is tested in `mana-kept-and-lost` |
| ATOM-601.2h-002 (partly covered) | no partial payment | `snow-mana`: a split the cost cannot be assigned to pays nothing |
| ATOM-609.4b-003 | a restriction survives "as though any color" | `hybrid-and-phyrexian-payment`, which builds CR 609.4b's spending, against decision 3's edge test |

CR 106.6 and 400.7d have no uncovered atom. Their covering tests are
integration tests, so deleting `types/mana.rs`'s thirty unit tests uncovers
none.

### 11.4 Counted

**The rules.** `MTG-Rules/versions/tmnt.txt`, paraphrased.

| Rule | What it says, for this design |
|---|---|
| 106.3 | Mana's source is the spell that made it, or the source of the ability that did |
| 106.4, 500.5, 703.4q | Each pool empties as each step and phase ends, one turn-based action, and the player "loses" that mana |
| 106.6 | Mana can carry three things: a restriction on how it is spent, an additional effect on the spell or ability it is spent on, and a delayed triggered ability that triggers when it is spent. None changes its type |
| 106.6a | When a replacement makes more mana, all of it carries them, with a separate delayed trigger or effect for each mana |
| 106.13 | Drain Power moves mana to another player with its source, restrictions and effects unchanged |
| 107.4h | `{S}` is paid with one mana of any type from a snow source. A generic reduction does not reduce it. "`{S}` spent" means mana from a snow source that was spent |
| 111.10h | A Powerstone's mana "can't be spent to cast a nonartifact spell": a restriction stated as what it cannot pay |
| 118.12, 605.3a, 608.2g | A cost paid as an effect resolves is paid in mana the same way, after a window for mana abilities |
| 400.7d | A permanent's abilities may read what mana was spent on the spell it was |
| 601.2h | The player pays the total cost; no partial payment |
| 603.7a, 603.7d–f | A delayed trigger exists from the moment it is created, and its source is the source of what created it |
| 608.2h, 113.7a | An effect reads a source that has left its zone as it last existed: a sacrificed Treasure |
| 609.4b, 118.14 | "Spend as though any type" changes how a cost may be paid, not what mana was spent. Not this design's: `hybrid-and-phyrexian-payment` builds it on decision 3's edge test |
| 700.14 | "Expend N" counts mana spent on spells in a turn. Not this design's: it reads the event decision 4 adds |

**The printed cards**, by what the shape must do. Counted by clause from the
cached corpus (`python plans/references/mana-provenance-census.py`, over
`cast-census.py`'s cache of `legal:commander -t:token game:paper -is:funny`,
32,115 cards, fetched 2026-10-05). `--names` lists each family's cards with
the clause that put them there. Every family under 30 was read card by card,
and all 121 restriction sentences.

| What a unit of mana must carry | Cards | Example | Held today |
|---|---:|---|---|
| a restriction on what it may pay | 175 | Mishra's Workshop, Cavern of Souls | `ManaRestriction`, four arms, dormant |
| a rider on what it is spent on | 22 | Boseiju, Who Shelters All; Path of Ancestry | `ManaGrant`, one arm, dormant |
| that it is kept until end of turn or combat | 50 | Savage Ventmaw; firebending (26 by keyword) | `ManaPersistence`, dormant |
| that a snow source made it, a permanent or a spell (CR 106.3, 109.2c) | 34 sources, one of them a sorcery (Tundra Fumarole); 39 cards with `{S}` in a cost; 7 reading `{S}` spent | Icehide Golem, Graven Lore | nothing |
| that a source with another part of a type line made it | 23: a Treasure 16, a creature 2, an artifact 2, a basic land 1, a Cave 1, a Desert 1 | Hired Hexblade, Imperiosaur | nothing |
| …and whether that source was a permanent | of the 23, all but one ask a permanent (CR 109.2: "a Treasure", "creatures"); Coin of Mastery asks "an artifact source", any zone | Myr Superion's ruling (2011-06-01): Elvish Spirit Guide's mana, made from a hand, is not "produced by creatures" | nothing |
| which object made it | 3 | Tecutlan, the Searing Rift: "Whenever you cast a permanent spell using mana produced by Tecutlan" | nothing |
| nothing new: the pool is read for how much unspent mana a player has | 5 | Omnath, Locus of Mana; Doubling Cube, whose ruling (2004-12-01) says the mana it adds carries no restriction | `AmountExpr::UnspentMana`, a count by type |

**The brief's counts, re-counted on Scryfall, 2026-11-13.** `o:"this mana"`
is 229 with no filter (the 227 of 2026-08-31) and 213 under `game:paper
-is:funny`. `o:"{S}"` is 43 and `o:"was spent to cast"` is 102, as carried.
`o:"lose unspent"` is 12, not 31: the 31 does not reproduce, and counted by
clause the family of kept and lost mana is 67 cards, in decision 5's table
(two of them in two of its rows).

**A wider net, at the owner's review (2026-11-13).** The owner asked whether
a card that says "that mana" (Domri, Chaos Bringer) could be missed. Domri
is counted: the table above is the clause classifier's, whose gates match
"this" and "that" mana, and the Scryfall phrases only re-count the brief's
numbers. The question still found something. Every sentence that names mana
beside "spend", "spent", "unspent", "produced by" or "mana from" was listed,
and the ones no gate had claimed were read. Five things had been missed and
are now counted by the script:
- three lands that read **which object** made the mana (the row above),
  which no type line answers;
- five cards that read the unspent mana in a pool (the row above);
- two more costs that name their mana (Soul Burn, Throne of Eldraine) and
  one that caps it (Emblazoned Golem), in decision 3's second table;
- Soul Burn's "the amount of `{B}` spent on X", the one printed reader of
  which part of a cost a unit paid, which decision 2 had said no card reads;
- Illusionary Mask's reader of the mana an activation spent.

What the net still leaves unclaimed is sentences about mana value and about
losing abilities.

**The rulings** read for timing and for which object is which (Scryfall,
2026-11-13): Imperiosaur, Myr Superion, Boreal Outrider, Cavern of Souls,
Boseiju, Jegantha, Rosheen Meanderer, Omnath, Locus of Mana, Kruphix,
Yurlok, Path of Ancestry, Pyromancer's Goggles, Hall of the Bandit Lord,
Leyline Tyrant, Upwelling, Doubling Cube and Tecutlan, the Searing Rift.
What they settle is cited where it is used.

**The binding table**, one row for each piece of rules state the design adds.

| State | Bound | Live or latched | Stored | Read by | Ends |
|---|---|---|---|---|---|
| What a unit's source was: which of the watched kinds of source | as `ProduceMana` is performed, from the source's characteristics then, or as it last existed if its own cost moved it (a Treasure, CR 608.2h) | latched: read once, as the mana is made (the owner, 2026-11-13, with the judges' answer) | on the unit, in the pool | a `{S}` pip; a cost that names its mana (decision 3); the spent record | when the unit is spent or lost. Drain Power moves it intact (CR 106.13) |
| Which object a unit's source was | at the same moment, and only where an ability then reads that object's own mana (decision 1, settled: not on every unit) | latched | on the unit | a trigger that asks for mana its own source produced (Tecutlan, the Searing Rift) | with the unit, then with the record of what was spent |
| A unit's restriction and its riders | at the same moment, from the ability's text. A type or color the source chose as it entered is read then | latched. A restriction's or rider's own condition reads the spell as the mana is spent (Path of Ancestry's ruling, 2023-07-28) | on the unit | the payment's edge test; the existence check | with the unit. A change of type keeps them (Kruphix's ruling, 2014-04-26) |
| A delayed-trigger rider's source and controller | at the same moment: the source of the ability that made the mana, as an existence, and the player who controlled that ability (CR 603.7d–e) | latched | on that rider alone, so mana with only a restriction keeps no object | the trigger, as it is put on the stack | with the unit |
| How long a unit is kept | at the same moment | latched | on the unit | the emptying | **ahead of the unit**: when its turn or combat ends, the effect expires before the mana empties (CR 500.5). If something else then keeps the unit, it stays without this term and regroups with mana like it |
| What a payment spent | as the payment is performed | latched | on the stack entry, then on the permanent it becomes (`CastFacts`), then in that permanent's last known information, which `snow-mana` adds beside the first reader | conditions and amounts on the spell, on the permanent, and on other objects' triggers | with that existence. A copy of a spell has none (Boseiju's ruling, 2004-12-01) |
| Which unspent mana an ability or effect stops from emptying. No player chooses: Upwelling stops all of it, Omnath a player's green | not stored: asked as the mana would be lost | live | the "can't" registry and the effective ability lists it already reads | the emptying | when its ability or effect does. Once Upwelling leaves, the pools empty as the current step or phase ends (its ruling, 2009-10-01) |

**Last known information holds a spell's spent mana today and a permanent's
never.** A spell that left the stack keeps its stack entry there, with the
record on it. A permanent that left the battlefield keeps `PermanentFacts`,
which has no cast facts. Item 33 says the first reader adds them, and
`snow-mana` has one: a creature whose enters trigger asks what was spent, and
is asked again after the creature has left.

### 11.5 The decisions

Numbered from 1. Each leads with what changing the answer after the build
would cost.

#### Decision 1: what a unit keeps, and how the pool holds it

**Retrofit cost: the highest here.** The record is read by the payment, the
check, the emptying and every reader of spent mana, and written by one arm.
Changing it after the build re-opens all three build rows.

**The problem.** The pool must tell units apart by what the rules can read
of them, and no further: every difference the pool keeps is a difference the
payment may have to ask about (decision 2). The table above says what can be
read: a type, parts of the source's type line, whether the source was a
permanent, for three cards which object the source was, and the terms.

- **A. One list of counted units; a unit is its type, what its source was,
  and its terms behind a pointer.**
  ```rust
  struct ManaUnit { mana_type: ManaType, source: ManaSourceFacts, terms: Option<Arc<ManaTerms>> }
  struct ManaUnits { unit: ManaUnit, count: u64 }      // one class of unit, counted
  struct ManaPool { units: Vec<ManaUnits> }            // alike units are one entry
  /// What was true of a unit's source as the mana was made (CR 106.3).
  struct ManaSourceFacts {
      kinds: ManaSourceKinds,       // which of the watched kinds of source it was
      object: Option<ObjectRef>,    // which object it was, where an ability reads that
  }
  struct ManaSourceKinds(u32);      // a bit for each kind on the watch list
  /// One kind of source a rule or a card asks about: "a snow source", "a Treasure".
  struct ManaSourceKind { has_all_of: &'static [ManaSourceCharacteristic], permanents_only: bool }
  enum ManaSourceCharacteristic { Supertype(Supertype), CardType(CardType), Subtype(Subtype) }
  struct ManaTerms { restriction: Option<ManaSpendRestriction>, riders: Vec<ManaRider>, lifetime: Option<ManaLifetime> }
  enum ManaLifetime { UntilEndOfTurn, UntilEndOfCombat }   // `None`: lost as the step or phase ends (CR 106.4)
  ```
  **The watch list.** One table holds each kind of source a rule or a
  registered card asks about, as a `ManaSourceKind`: the characteristics a
  source of that kind has, and whether only a permanent counts. CR 109.2
  makes "a Treasure" a permanent and "a snow source" an object in any zone
  (109.2c), so a snow sorcery's mana pays `{S}`, and a creature card's mana
  made from a hand is not "produced by creatures". A unit holds a bit for
  each kind: whether its source was one as the mana was made. The list
  starts with "a snow source", CR 107.4h's. The printed readers name eight
  kinds: a snow source, a Treasure, a Cave, a Desert, a creature, an
  artifact, an artifact source, a basic land. A reader names a kind on the
  list and nothing else, and a thirty-third kind widens the `u32`.
  **Which object.** Three lands ask for mana their own permanent produced
  (Tecutlan, the Searing Rift), which no kind answers. `object` holds it,
  and is `None` on every unit whose source nothing asks about, so two
  Forests' mana is one entry and two Tecutlans' is two. When it is set is
  settled below.
  **Two units with equal records are ones no rule can tell apart**, because
  the record holds the answer to every question the engine can ask. The
  converse holds but for two restrictions written differently that permit
  the same spends, which costs a question with one outcome and loses
  nothing. `ManaTerms` holds no object: only a delayed-trigger rider keeps
  its source's existence and its controller (decision 4), so two Mishra's
  Workshops' mana is one entry. `terms` is `None` exactly when a unit has no
  restriction, rider or lifetime, and a `ManaTerms` with all three empty is
  never built. `ManaUnits` is the one spelling of a counted class, in the
  pool, in the action that loses mana, in the event of spending and in the
  record of what was spent. The two-track pool, `ManaAtom`,
  `ManaPaymentPlan`, `pay_with_plan`, `SpendContext` and their unit tests go.
- **B. Keep today's two tracks and key the plain one by type and answers.**
  Less of `types/mana.rs` is rewritten. The payment, the check and the
  emptying each keep two paths for one concept, which is the shape the
  dormant code already has and nothing exercised.
- **C. A unit keeps which object made it, and readers ask that object.** No
  watch list: "mana from a Cave" needs no entry. But two Forests' mana are
  then two records, so entries never merge, the pool grows with the lands
  tapped, and decision 2 has no way to know which differences matter. Each
  production would also capture its source as it last existed, where today
  only a source its own cost moved is captured.

**What a clone pays** (floor 2 is allocations per clone, at most 64;
measured in a scratch probe outside the tree, 2026-11-13, a counting
allocator around one `clone()`):

| Pool | Never written | Holding mana | After it emptied |
|---|---:|---:|---:|
| Today: a `HashMap` by type, zeroed entries kept | 0 | 1 allocation, 84 bytes | **1 allocation, 84 bytes** |
| A: a `Vec` of counted units | 0 | 1 allocation, 72 bytes for three entries of 24 | **0** |
| B | as today | as today, plus 1 while restricted mana floats | as today |
| C | 0 | 1, larger: an entry per source | 0 |

Today a pool costs every later clone one allocation from the first mana its
player makes, four at four seats, since the map keeps its table when its
counts are zero. A's list is empty at every decision where no mana floats,
which is most of them, so **A lowers floor 2's count by up to one a seat**
and holds it while mana floats. Terms are behind an `Arc`, so a clone copies
a pointer.

The probe's unit held a 16-bit set and no object. With `object` an entry is
about 48 bytes where the probe's was 24: counted from the fields, not
measured. The allocation count is the same.

**A pool's size is its kinds of mana, never its amount** (the owner asked
about Nyxbloom Ancient and loops that make mana without limit, 2026-11-13).
A million `{G}` from Forests is one entry whose count is a million, and a
clone copies the same 48 bytes for it as for one `{G}`, in the same one
allocation. Entries number the classes floating: a type, with the kinds of
source and the terms that are actually in the pool. That is a handful on a
busy board, a few hundred bytes against floor 3's 128 KB. A tripled
production adds to a count, since CR 106.6a gives the new mana the same
terms. `count` is a `u64` because every count of mana in the engine is one
today. It is the field `GameNumber` takes when `backlog.md`'s loops capture
gives the engine a number for "arbitrarily large", and nothing else in the
record changes then.

**How A widens.** The eight printed kinds all read the type line, so
`ManaSourceCharacteristic` starts with three arms. That is where it starts,
not what it can hold: a custom card may ask more (the owner, 2026-11-13),
and each wider question has one place to go.

| A card asks about | It needs | Where |
|---|---|---|
| another kind over the type line ("a Gate") | a row | the watch list |
| a kind over another characteristic of the source ("a red source", "a source with mana value 3 or less") | an arm | `ManaSourceCharacteristic`, read by the one function that fills the bits |
| which object the source was | nothing new | `ManaSourceFacts::object` |
| a comparison with the reader, which a yes or no fixed at production cannot hold ("a source that shares a color with this spell") | a field holding the value, here the source's colors | `ManaSourceFacts`, so that equal facts still mean alike units |

What it cannot hold is a question about the source after the mana was made,
which the rules do not ask (the reading below). The engine's general
`ObjectFilter` was the other candidate for a kind's description, and is
wider on day one. It was not taken because several of its leaves ask about
a live object or about the filter's own source (`ThisTurn`, `NotSource`,
`ByController`), which a source read once cannot answer, and because a kind
built from it is not a constant. `ManaSource` is already the inventory's row
for an ability that can make mana, the same CR 106.3 noun, so these names
extend it.

**The reading, settled: what the source was is read once, as the mana is
made** (the owner, 2026-11-13, with the judges' answer). A Snow-Covered
Forest makes `{G}`, Arcum's Weathervane makes the land no longer snow, and
the `{G}` in the pool still pays `{S}`. The other direction follows: mana a
land made before it became snow does not. `snow-mana` tests both, with
Arcum's Weathervane or a fixture with its two abilities. CR 106.3 names the
source at that moment, and a Treasure is already in the graveyard by then,
so its facts are last known information (CR 608.2h).

**Found in round 1 and settled in round 2: when a unit keeps which object
made it.** Retrofit cost: low. The field is there under either answer. What
moves is one function at the production and the payment's grouping of
entries, about a hundred lines, and no fixture.

Three printed lands read it, each by a trigger on the land itself:
"Whenever you cast a permanent spell using mana produced by Tecutlan". Two
Forests' units must be one class, or every payment with spare mana would
ask which Forest's `{G}` to spend. A Tecutlan's units must not merge with a
Mountain's, because spending one triggers and spending the other does not.

- **1a. Keep the object where an ability reads it as the mana is made.**
  The production asks whether its source then has an ability that reads its
  own mana, and sets `object` if so. Equal records still mean alike units,
  and a pool has as many entries as it has kinds of mana. One board reads
  differently under 1b: a Thespian's Stage makes `{C}`, Upwelling keeps it,
  the Stage becomes a copy of Tecutlan, and the old `{C}` pays for a
  permanent spell. Under 1a the trigger does not fire. A card's name in its
  text means that object (CR 201.5) and a mana's source is the object that
  made it (CR 106.3), which argues that it should. No ruling covers it.
- **1b. Keep the object on every unit made by a permanent that is still on
  the battlefield, and group the entries as the payment asks.** No miss. A
  pool then has an entry for each land tapped, and "alike" stops being
  equality of records: the payment groups entries by what an ability on the
  battlefield can tell apart at that moment, a second definition to keep in
  step with the readers. A source that has left, a sacrificed Treasure,
  keeps no object under either answer, since nothing can come to read it.

**Settled: 1a** (the owner, 2026-11-13), built with the first of the three
lands. That board is the snow reading again: what a unit's source was is
read as the mana is made, and as that `{C}` was made nothing read which
object made it. With no ruling against it the reading stands, so the board
is not filed as a wrong answer. 1a keeps the one definition of alike that
decision 2 rests on, and 1b is the change above if a ruling ever says
otherwise.

**Recommended: A.** It is the only option where "which units differ" has an
answer the engine can compute, and it is cheaper than today for a clone.

#### Decision 2: which unit pays, and what the engine asks

**Retrofit cost: high.** It fixes what a provider is asked at every payment,
so a change after the build is a fixture migration and a moved random
stream: the 10 test sites and 11 source sites that name
`GenericManaAllocation` today, and whatever the three rows add.

**The problem.** CR 601.2h has the player pay, so once two `{G}` differ,
which one pays is the player's choice. Today one question is asked: how the
generic part is split across the pool's types, each type capped at what its
own pips leave, and skipped when only one split is legal. Colored pips are
never asked, since one type pays each.

**What the game can read of a payment** is which units were spent and which
are left, and with one printed exception never which pip a unit paid.
Cavern of Souls' and Boseiju's rulings say "any cost of the spell", Rosheen
Meanderer's "any part of a cost", and Boreal Outrider's (2021-02-05) "it
doesn't matter which part of the total cost the snow mana was spent on". So
a payment is a choice of units, legal when some assignment of them to the
pips exists, and the engine finds the assignment.

**The exception is Soul Burn**, `{X}{2}{B}`: its life gain is capped by "the
amount of `{B}` spent on X", so the same four units spent with one `{B}` on
X or with none are two games. Round 1's wider net found it, and a search of
the corpus for "spent on" found no second card that reads a part of a cost.
Where a reader names a part, the split is asked for that part first, over
the units the part accepts, then for the rest, and the record of what was
spent keeps that part's units apart. Both wait for the card, in "What
waits".

- **A. One split, over classes of unit.** The buckets are the pool's entries
  that this payment may spend (decision 3), and the answer is how many of
  each to spend. Units every legal payment must spend are taken first and
  are not asked, so the question covers only what is free, with each
  bucket's cap exact. **Where every type in the pool is one class, this is
  today's question with today's buckets and caps.** It is skipped when one
  answer is legal, as today. Caps cannot say "at most three of these two
  buckets together", which needs three classes of one type sharing a scarce
  property. There the whole answer is checked before anything is paid, and
  one the cost cannot be assigned to is refused, naming the pip left unpaid,
  and asked again, as an order of payment that leaves a part unpaid is today
  (`ask_payment_order`'s `Rejection`).
- **B. A class at a time, as numbers.** "How many snow `{G}`? 0 to 2", then
  the next class, each interval exact given the answers before it. No
  refusal is ever needed. But today's one question becomes one per class, so
  every fixture that scripts a split is re-scripted, the random agent draws
  differently in every game with a generic cost, and decisions per game
  rise.
- **C. A unit for each pip.** The literal reading. Outside Soul Burn, two
  answers that differ only in which pip a unit paid give one game, which `choices-architecture.md`'s
  decision 6 ruled out for an order: a question whose answers all give one
  game is not asked.

**The names.** The kind covers colored pips once a type has two classes, so
`GenericManaAllocation` no longer says what it asks. Proposed:
`ChoiceKind::WhichManaPays`, its options `ChoiceOption::Mana(ManaUnit)`, one
option kind for a plain unit and a snow one. That is a counted sweep by
`replace_exact.py`, 21 kind sites and 13 option sites in the crate and one
in the dev GUI, with no answer re-scripted. A client gets each bucket as a
`ManaUnit` and draws its type, what its source was and its terms.

**How A is kept right.** Decision 1's record makes the elision exact rather
than argued: two units are one bucket only when their records are equal, and
equal records answer every question alike. What can go wrong is a reader
added without its kind, a card that asks "mana from a Cave" by some
other route. So a reader can name only a kind on the watch list, and
there is no second route to add.

**Recommended: A.** It asks only where the answers differ, changes no
question a game asks today, and is what makes decision 9 a "no".

#### Decision 3: what a restriction is, and what it is asked about

**Retrofit cost: medium.** A restriction is card data, so its shape is in
every card that makes restricted mana, none registered yet, and in the one
function that reads it.

**The table.** The 175 cards' 121 distinct sentences, by everything each
predicate reads; a card is in every row it reads.

| The predicate reads | Cards | Example |
|---|---:|---|
| that the payment casts a spell | 155 | "only to cast creature spells" |
| …the spell's type line | 124 | Mishra's Workshop; Haven of the Spirit Dragon (`tdc`, the five decks' set) |
| …its color, mana value, cost symbols, keywords or abilities | 17 | "with mana value 4 or greater", "with devoid", "with no abilities" |
| …how it is cast: its zone, kicked, face down, its owner, one named card, an alternative cost by name | 17 | Lord of the Forsaken (`tdc`): "from your graveyard"; "a disturb cost" |
| …a type its source chose as it entered | 6 | Cavern of Souls |
| that the payment activates an ability | 53 | "only to activate abilities" |
| …the ability's source, by type line or color | 38 | "abilities of artifacts" |
| …the ability's kind | 4 | equip, power-up, a Class level |
| that the payment is a special action's (CR 116.2b, 116.2h, 116.2m) | 7 | turning a permanent face up; foretelling a card; unlocking a door |
| the cost itself, by a symbol in it, whatever it is for | 5 | Rosheen Meanderer: "costs that contain {X}"; Cultivator Drone: "a cost that contains {C}" |
| that the payment is a keyword's cost, paid as its trigger resolves | 2 | "only to pay cumulative upkeep costs" (CR 702.24a) |
| one part of a cost | 1 | Jegantha: "can't be spent to pay generic mana costs" |
| stated as what it cannot pay | 10 | a Powerstone |

Today's `ManaRestriction` holds a spell's card types, an ability source's
card types, one creature type, and "any of". It was drawn from the cards the
old plan named: it can say "creature spells", "abilities of artifacts" and
Cavern's chosen type, and nothing from the color row down.

- **A. A restriction is a list of payments it permits, or a list it
  forbids. There is an arm for each thing a payment of mana can be for, and
  the filters inside an arm say which.**
  ```rust
  enum ManaSpendRestriction { Only(Vec<ManaSpendFilter>), Never(Vec<ManaSpendFilter>) }
  enum ManaSpendFilter {
      /// Casting a spell (CR 601.2h): which spell, and how it is cast.
      Spell { is: ObjectFilter, cast: Option<CastFilter> },
      /// Activating an ability (CR 602.2b): of which object, and which keyword's, where one is named.
      Activation { of: ObjectFilter, keyword: Option<AbilityKeyword> },
      /// A cost a resolving spell or ability asks for (CR 118.12): of which object, and which keyword's.
      EffectCost { of: ObjectFilter, keyword: Option<AbilityKeyword> },
      /// A special action (CR 116), by the engine's own enum.
      SpecialAction(SpecialAction),
      /// Any of those whose cost holds this symbol: "costs that contain {X}".
      CostContains(ManaSymbol),
  }
  struct ManaPayment<'a> { cause: EventCause, ability: Option<AbilityRef>, cost: &'a ManaCost }
  impl ManaUnit { fn may_be_spent_on(&self, payment: &ManaPayment) -> bool }  // its restriction, once for each class of unit
  impl ManaPip { fn accepts(&self, unit: &ManaUnit) -> bool }                 // two mask tests, once for each kind of pip
  ```
  **What a payment is for is its cause, which the engine already has.**
  Every proposal names an `EventCause`, and a payment's is `Cost { player,
  paying_for }` for a spell or an ability, `Effect` for a cost a resolution
  asks for (CR 118.12), and `SpecialAction`. A restriction reads that value,
  so `SpendPurpose` goes with `SpendContext` and nothing re-spells it.
  `ManaPayment` adds the two things the cause does not hold: which ability
  the payment is for, an activation's or the one resolving, named as the
  engine names an ability (`AbilityRef`, never a position); and the mana
  cost being paid, which Rosheen's "costs that contain {X}" reads. A spell
  is read through the one `ObjectFilter` leaf table every other filter uses.
  `CastFilter` reads what the stack entry already records of a cast: its
  zone, its additional costs and its alternative cost. The struct is not
  `ManaSpend`, which is one letter from decision 4's event, `ManaSpent`.
  **The arms are the four things the CR lets mana pay for, and the cost
  itself**, so a custom card's restriction is a new combination of filters
  inside an arm, not a new arm (the owner, 2026-11-13). The first draft had
  `KeywordCost(CostKeyword)` for "only to pay cumulative upkeep costs", an
  arm shaped by its two cards. Cumulative upkeep's cost is one a resolving
  ability asks for, so it is `EffectCost { of: All, keyword:
  Some(CumulativeUpkeep) }`, and the same arm says "only to pay ward costs"
  or "only to pay costs your opponents' spells and abilities ask of you"
  with no engine change. `AbilityKeyword` is the one tag for which keyword
  an ability is, read off the ability through its `AbilityRef`: equip and a
  Class level for an activation, cumulative upkeep and ward for a cost at
  resolution. Six printed restrictions name one, the table's rows of four
  and two.
  **A cost paid as an effect resolves has `Effect` as its cause**, so mana
  "only to cast creature spells" cannot pay Rhystic Study's `{1}`, and
  Rosheen Meanderer's can pay Condescend's `{X}` (its ruling, 2017-11-17):
  `Primitive::Pay` passes the cause it resolves under and the ability that
  is resolving.
- **B. An arm per printed phrasing on today's enum.** No new filter type.
  121 sentences is where that ends, each arm restating a predicate
  `ObjectFilter` already has.

**At the offer the spell is still a card.** The existence check asks before
anything is proposed, so it builds the cause the payment would have, and
that cause names a card in a hand or a graveyard. The card's type line,
color and mana value are read as they are, and its zone is known, since the
offer is for a cast from that zone. A choice CR 601.2b has not made yet,
kicked or face down or an alternative cost, is read as met: mana "only to
cast kicked spells" counts toward a kickable spell's offer. That over-offers,
which costs a rewind, where the other reading would hide a legal play (the
owner's rule for the check, 2026-09-30). The payment at CR 601.2h reads the
choices as made. `restricted-mana-plus-spend-riders` tests one of each at
the offer and at the payment.

**The other side of the edge: a cost that names its mana.** Twelve cards
restrict from the cost's side.

| The cost says | Cards | Example |
|---|---:|---|
| only mana of a color pays it, or pays its X | 8 | Consume Spirit: "Spend only black mana on X"; Soul Burn: "black and/or red"; Throne of Eldraine: "only mana of the chosen color to activate this ability" |
| …and at most one mana of each color | 1 of the 8 | Emblazoned Golem |
| only mana from a kind of source pays it | 3 | Imperiosaur, Myr Superion, Security Rhox |
| no mana pays it | 1 | Hogaak, Arisen Necropolis |

A pip says what it accepts, as a `ManaPip`: which types, and which kinds of
source. `{S}` is the first such pip (decision 6): any type, from a snow
source. Consume Spirit's X is pips that accept black alone; Rosheen's ruling
says the limit is on that part of the cost and no other. Imperiosaur's whole
cost accepts units whose source was a basic land on the battlefield.
Hogaak's accepts none, so convoke and delve pay all of it. Emblazoned
Golem's cap is not a pip's: it bounds how much of one type a part of the
cost takes, a capacity in the cover test, and waits for the card. Jegantha's
mana, "can't be spent to pay generic mana costs", is the one printed
restriction that reads the pip from the unit's side. Its arm comes with the
card and is read where the pip is.

**What the edge test costs** (the owner asked whether it needs a fast path,
2026-11-13). It is two questions, and neither is asked of each unit for
each pip.
- *Does this unit's restriction permit this payment?* `may_be_spent_on`,
  asked once for each class of unit in a payment, and only of a class that
  has terms. A filter reads the spell or the ability's source, which is the
  same for every pip.
- *Does this pip accept this unit?* `accepts`: a pip is two small sets, the
  types it takes and the kinds of source it asks for, and a unit holds one
  of each, so it is two mask tests. For a plain pip and a plain unit it is
  today's test of one bit.

The existence check already works on classes. It groups the supply into
entries by type set and the cost into kinds of pip, and visits neither a
unit nor a pip (`#32-the-check-halls-condition`, about 100 instructions a
cost). Restricted entries are filtered once ahead of it, the 2,100
instructions `#36-item-33s-seam-an-edge-refused-decision-2` priced, paid
only where a pool or a source has terms. So the plain path is the fast
path: a game with no restricted mana and no cost that names its mana runs
today's instructions and one test of `terms.is_none()` for each class.

**Recommended: A.** Every row of the table is a filter inside one of five
arms, the purpose is a value the engine already carries, and the filter is
the one it already reads spells and permanents with.

#### Decision 4: riders, and the event of spending

**Retrofit cost: medium.** It adds an event every payment emits, which
triggers and the history then read.

**The table.**

| A rider does | Cards | Example | What it creates |
|---|---:|---|---|
| a delayed trigger when the mana is spent | 10 | Path of Ancestry (`tdc`), Pyromancer's Goggles | a triggered ability on the stack, one for each unit spent (Goggles' ruling, 2024-11-08) |
| the spell it paid for can't be countered | 3 | Cavern of Souls, Boseiju | a "can't" on that spell (CR 101.2) |
| the permanent the spell becomes gains an ability | 5 | Hall of the Bandit Lord | a continuous effect on the spell that follows it to the battlefield |
| that permanent enters with counters | 4 | Opal Palace | a replacement of that spell's entry |

**The problem.** CR 106.6 says the delayed trigger is created with the mana
and "triggers when that mana is spent", and 106.6a says there is one for
each unit. The engine detects triggers from performed events, and spending
mana emits none: `pay_single_cost` writes the pool directly.

- **A. The rider stays on the unit, and the payment announces what it
  spent.**
  ```rust
  struct ManaRider { when: ManaSpendFilter, then: ManaRiderEffect }
  enum ManaRiderEffect {
      Trigger { effect: Effect, source: ObjectRef, controller: PlayerId },
      SpellCantBeCountered,
      PermanentGains { keyword: KeywordFlag, until: Duration },
      EntersWithCounters { kind: CounterType, amount: AmountExpr },
  }
  ```
  One arm for each row of the table, and `when` is decision 3's filter, so
  "if that mana is spent on a creature spell" is written once. The names
  carry `Mana` because `Rider` alone is the replacement pipeline's word, for
  an effect queued behind a replaced event. `pay_costs` performs the removal
  and emits one `GameEvent::ManaSpent { player, units }` **for each payment
  that spends mana, not for each unit**, whose stamp already carries the
  cause, what it was spent on. Each rider
  applies **once for each unit spent** whose `when` the spend meets (CR
  106.6a's "a separate effect … for each mana"): two of Path of Ancestry's
  mana on one creature spell are two triggers (its ruling, 2020-11-10). The
  pool holds the rider once and the count beside it, so a million mana from
  one loop is one entry. A trigger's source and controller are the ability's
  that made the mana, fixed then (CR 603.7e), which differs from the spender
  only once Drain Power has moved the mana. The other three kinds are handed
  to the subsystem that owns each: the "can't" registry; a continuous effect
  on the spell, which the engine already carries onto the permanent it
  becomes (CR 400.7a, `engine/zones.rs`); and a replacement of that spell's
  entry. Spending is not replaceable (no effect says "if you would spend
  mana"), so the write stays where it is, takes the `// NOT REPLACEABLE:`
  tag, and gains the event the rule needs. **The event takes the name**: it
  is `ManaSpent` beside `ManaAdded` and `ManaLost`, and the record of what a
  payment spent, `ManaSpent` today, becomes `SpentMana` (decision 6).
- **B. Register a delayed trigger for each unit as it is made.** The CR's
  own wording, and the trigger registry's existing shape. A registration per
  unit is what counting alike units exists to avoid, and each emptying would
  have to retire the triggers of the mana it lost.

**What the event adds to a game** (the owner asked whether every spend is
now logged, 2026-11-13). Yes: one record for each payment that spends mana,
about 70 a game on the budget's board. Each production already announces a
`ManaAdded`, and a payment is usually several productions, so the stream
gains fewer records than it holds for mana today. A record is what the
engine finds triggers from, so it is the only way a rule can react to
spending: the ten delayed triggers above, and expend (CR 700.14, twelve
cards), which counts the mana spent on spells in a turn. The record holds a
pointer to the slice the spent record keeps (decision 6) and copies
nothing. It is not a proposal. Nothing can replace or stop spending, so it
does not pass the replacement pipeline, and where no unit has a rider and
no trigger watches spending its cost is the append and the matcher's look
for a trigger on that kind of event.

**Recommended: A.** The unit is the one place the rider lives, and the event
is one a second rule already needs.

#### Decision 5: mana kept past a step's end, and mana lost

**Retrofit cost: low to medium.** It turns two direct writes into an event,
which nothing reads until this row's cards.

**The table.**

| The shape must express | Cards | Example | Held today |
|---|---:|---|---|
| a unit kept until end of turn or combat | 50 | Savage Ventmaw; firebending | `ManaPersistence::UntilEndOf`, dormant |
| mana kept by a static ability, all of it or one color | 6 | Upwelling; Omnath, Locus of Mana; Leyline Tyrant (`tdc`) | `BlanketPersistenceSet`, always empty |
| …kept by a resolved effect with a duration | 1 | The Last Agni Kai | nothing |
| the loss replaced: the mana changes type and stays | 5 | Kruphix, God of Horizons | nothing |
| the loss has a result | 2 | Yurlok of Scorch Thrash (life lost); Drain Power (the mana moves) | nothing |
| an effect makes a player lose unspent mana, at no step's end | 5 | Mana Short, Power Sink, Worldpurge | nothing |

**The problem.** `empty_mana_pools` writes every pool directly, so no
replacement can watch it (Kruphix), no "can't" can stop it (Omnath), and
nothing can follow it (Yurlok). `CLAUDE.md`'s chokepoint rule already says a
direct write is invisible to CR 614. And the last row loses mana by an
effect, which Kruphix replaces and Yurlok follows as they do the step's, and
which "you don't lose unspent red mana as steps and phases end" does not
stop.

- **A. Losing mana is a proposal, and each family is the subsystem that
  already owns its kind of statement.** As a step or phase ends, the engine
  proposes one batch through `execute_actions`, a member for each player and
  class of unit: `GameAction::LoseUnspentMana(player, ManaUnits)`, under the
  turn-based action's cause (CR 703.4q). Mana Short proposes the same action
  under its own effect's cause.
  - **A unit's own lifetime.** A unit still inside it is not proposed at a
    step's end. Once its turn or combat has ended it is proposed like any
    other, and if something else keeps it, the term is dropped and the unit
    regroups with mana like it (CR 500.5: the effect expires, then mana
    empties). A firebender's `{R}` under Leyline Tyrant is plain `{R}` next
    turn.
  - **Kept by a static ability or an effect.** "You don't lose unspent red
    mana as steps and phases end" is a `Restriction::Event` on that action
    for that player and type, in the "can't" registry, which already holds
    static abilities and effects with durations and re-checks that a static
    one still exists. Its pattern names the turn-based action's cause, which
    every proposal carries, so it does not stop Mana Short.
  - **Replaced.** Kruphix is an `EventPattern::LoseUnspentMana` arm that
    names no cause, so it matches both. Its rewrite is `Rewrite::Instead`,
    CR 614.1a's arm, of a second new action that changes those units' type
    and leaves their terms (its ruling, 2014-04-26). The closed algebra
    gains no arm.
  - **Followed.** Yurlok's loss is a result of the batch, as lifelink's gain
    is of a damage batch (CR 120.3f, `engine/actions.rs`): the batch performs
    after its replacements are decided, so it knows what each player lost,
    and proposes one life loss for each player for that total (its ruling,
    2020-11-10: at the same time, no stack, one loss however many Yurloks).
  A pool with nothing to lose proposes nothing.
- **B. As item 4 was sized: read a kept set off a registry row at the two
  sites, and keep the direct write.** About 50 lines. It holds the 7 cards
  that keep mana by a static ability or an effect, and none of the 12 that
  replace, follow or cause a loss.

**What A costs, measured** (the owner asked whether a probe should answer
this, 2026-11-13). A counter where the pools empty, in a scratch copy of the
engine at `256972fa`, over the standard 200 games at seed 12345:

| | Emptyings a game | Emptyings that lose any mana | Proposals A would make, a game |
|---|---:|---:|---:|
| Two seats | 377 | 1.2 a game, one in 310 | 1.3 |
| Four seats | 900 | 2.6 a game, one in 347 | 2.7 |

A proposal is one player's mana of one type. At the 22,500 instructions
`#310-what-a-mana-ability-makes-decision-5` measured for a production's
proposal, A adds about 30,000 instructions a game at two seats and 60,000
at four, in games of 191 and 478 decisions. The record's last readings of
one decision are 0.65 to 0.85 million instructions, on the Commander board,
so a game's whole cost of A is under a tenth of one decision. An emptying
with nothing to lose proposes nothing and costs what it does today, and B
costs what today does. The probe does not separate them, so the choice is
on what each can express. The random agent rarely leaves mana floating; a
deck built around Omnath or Upwelling would, and there the proposal is the
card working.

**Recommended: A.** One action serves all six families, and each family
lands in the registry that already knows how to ask whether its source still
has the ability.

#### Decision 6: `{S}`, and the record of what was spent

**Retrofit cost: medium.** The spent record sits on every stack entry and
every permanent that was cast.

**The table.** 212 cards read the mana a payment spent.

| They read | Cards | Example |
|---|---:|---|
| its colors or types | 113 | converge and sunburst (44 by keyword); adamant; "if `{U}` was spent" |
| how much, of any kind | 55 | "if no mana was spent to cast it" |
| what kind of source made it | 27 | "if mana from a Treasure was spent" |
| which object made it | 3 | Tecutlan, the Searing Rift (decision 1) |
| its type and its source together | 1 | Boreal Outrider: "`{S}` of any of that spell's colors" |
| how much of a type paid one part of the cost | 1 | Soul Burn: "the amount of `{B}` spent on X" (decision 2) |
| how much this turn | 12 | expend; decision 4's event |

Where, by the sentence's shape, so approximate: 99 on the permanent the
spell became, 67 by a trigger or effect while the spell is on the stack, 56
by the spell as it resolves, 9 of an ability's activation (Forsworn
Paladin, Illusionary Mask). The engine records a cast's spent mana today
and an activation's never.

**The problem.** The record is six counts by type, held inline. It must now
answer by what the source was, and by type and source together.

- **A. The units as they were spent, shared.** `SpentMana(Option<Arc<[ManaUnits]>>)`:
  the same slice the `ManaSpent` event carries, written once by the payment
  and never edited. It answers every row of the table but Soul Burn's, whose
  card adds which of the units paid X, and whatever a later kind on the
  watch list asks. The units keep their terms, which nothing reads once
  they are spent; dropping them would make a second type for one value.
  `None` is the only empty, today's `ManaSpent::NONE`: a payment that spends
  no mana writes `None`, and an empty slice is never built. An activation
  records one too.
- **B. Inline counts, by type and by answer.** `{ by_type: [u64; 6],
  answering: [[u64; 6]; N] }`, 48 bytes for each watched kind.

| | Allocations a clone | Bytes inline, each stack entry and cast permanent | Grows with |
|---|---:|---:|---|
| Today | 0 | 48 | nothing |
| A | 0: a pointer is copied | 8 | nothing |
| B | 0 | 96 with snow alone, 432 with the eight printed kinds | each new kind |

A costs one allocation when a payment spends mana, beside the payment's
existing list of what it used.

**`{S}` itself** is a pip any type pays and only a snow unit may:
`#32-the-check-halls-condition`'s seventh bit, set on a snow source's
entries and on the pool's snow units. A generic reduction never touches it,
since the reduction removes `Generic` symbols and `{S}` is its own. It is
paid by decision 2's split.

**What B has over A** (the owner asked, 2026-11-13): a payment allocates
nothing, where A allocates one slice for each payment that spends mana,
about 70 a game; and the record stays `Copy`. Neither is on a path that is
counted: a clone copies a pointer under A, and a reader walks a slice of a
few entries. B cannot answer the row for which object.

**Settled: A** (the owner, 2026-11-13). It is smaller than today on the
state and does not grow with the vocabulary.

#### Decision 7: item 135, a mixed production under a fixed amount

**Retrofit cost: none.** A replacement that sets a production to a fixed
amount of one type (Damping Sphere) meets an ability making restricted and
unrestricted mana at once, and the engine refuses, loudly. Item 135 asked
this design for the reading that decides it.

The per-unit record does not decide it. CR 106.6a covers a production made
larger, where every unit carries what the ability's units carried; a
production made smaller out of two kinds of unit has no rule saying which
kind the new mana is. No printed mana ability makes such a mix (Scryfall,
2026-11-13, three phrasings: `o:/spend (one|the|that) [^.]*mana only/`,
`o:/one of (this|that) mana/` and `o:/add [^.]*\. spend (only )?(the|one)/`,
each 0).

- **A. Keep the refusal, and re-slot the item to `breadth-features`, with
  its first card**, whose ruling decides it.
- **B. Pick a reading now**, all restricted or all free. Either is a guess a
  wrong answer hides: a restriction that vanishes from a pool, or one that
  appears.

**Settled: A** (the owner, 2026-11-13): the reading is ill-defined without
a card that makes the mix, which is likely why none is printed. Item 135's
slot moved with this answer.

#### Decision 8: the build, in three rows

**Not a choice between options** (the owner asked what there was to decide,
2026-11-13). It is the sizing the brief asked for, in "The build, sized"
below, numbered here so its answer has a place: a yes puts three named rows
on the route where one was carried at 1–2 PRs. The record under `{S}`
first, then restrictions and riders, then kept and lost mana, each with
printed cards that need no later row. The first two sit at the band's top,
so each names where it splits.

**One real question inside it, found in round 1: does `{S}` wait until
after v0.5?** Retrofit cost: none. The record is designed with its source
facts under either answer, and what moves is which PR builds them. None of
the five decks holds a snow card, a `{S}` or a reader of a mana's source:
`mana-provenance-census.py --names` against `tdc-names.txt` finds the decks
in restricted mana (Haven of the Spirit Dragon, Lord of the Forsaken), one
rider (Path of Ancestry), kept mana (Leyline Tyrant) and the amount spent
(Manaform Hellkite). The route's rule for work no deck needs is
`breadth-features`, after v0.5.
- **As drawn: `snow-mana` first.** `{S}` is the smallest proof that the
  record's source facts work, built while the record is new. Its `{S}` half
  is about half a PR ahead of v0.5 that no deck needs.
- **Snow after v0.5.** The first row becomes the record, the pool and the
  spent record alone, with every game `IDENTICAL`. The split over classes
  moves to the restricted row, whose first card is its first consumer. The
  watch list, `{S}` and the snow cards become a fourth row after v0.5.

**Recommended: snow after v0.5**, by the route's own rule. The table below
is as drawn until the owner answers, and is re-cut and re-sized on a yes.

#### Decision 9: whether the build comes before `costs-and-choices-at-resolution`

**Retrofit cost of a wrong "no":** the fixtures that row writes for a
payment at resolution, re-scripted. Of a wrong "yes": two to three PRs moved
ahead of step 5's last three rows for nothing.

**The question.** `choices-architecture.md`'s decision 4 builds
`Primitive::Pay`, a cost paid as an effect resolves (Rhystic Study, Mana
Leak, ward), through the cast's own payment. This design reshapes that
payment. Would building `Pay` first unbuild something?

**No, under decisions 2 and 3 as recommended**, for three reasons:
1. **`Pay` touches the payment through three calls**, the ones the cast and
   the activation make (`put_on_stack.rs`, CR 601.2g to 601.2h): the mana
   window, the check that the cost is payable, then `plan_payment` and
   `pay_costs`. The build changes what those do inside and gives each one
   more argument, what the payment is for (decision 3's `ManaPayment`). A third caller gains it in the same edit the
   other two do.
2. **A plain pool is asked what it is asked today** (decision 2's A). No
   snow or restricted mana can exist before the build, so every fixture
   `costs-and-choices-at-resolution` writes meets a plain pool, and none is
   re-scripted. The rename is a counted sweep over all of them.
3. **Nothing can be misspent in between**: no production makes a unit the
   plain pool does not hold.

**It would be "yes" under decision 2's B**, a class at a time: that changes
the question a plain pool is asked, and every split fixture written before
it is re-scripted.

**So `mana-ability-costs` does not move.** Had the answer been yes, it would
have moved with the build: `snow-mana` extends the existence check's entries
and its cover test, which `mana-ability-costs` reshapes first (the owner,
2026-10-06, at #225's review).

**One line for that row's brief.** Leyline Tyrant is one of its consumers,
for "pay any amount of `{R}`". Its other ability keeps red mana, decision
5's, so the card is registered by `mana-kept-and-lost`, and
`costs-and-choices-at-resolution` tests the payment on a fixture.

**Round 1** (the owner, 2026-11-13): yes to the recommendation, once
decisions 2 and 3 are settled, and they are still being discussed.

### 11.6 The build, sized

Three PRs, each with printed cards, after `mana-ability-costs` and before
`any-color-plus-painlands-plus-signets`, in this order. Code and tests
apart. "Raw" is counted from the sites below; "at the doc's ratio" applies
this document's own record, MA-1 at 1.6 times its sizing and MA-2 at 1.4.

| PR | What it builds | Consumer | Code, raw | Tests, raw | At the doc's ratio |
|---|---|---|---:|---:|---:|
| **`snow-mana` — a unit keeps what its source was, and `{S}` is paid** | Decision 1's record and pool, with the dormant types and their tests removed; `ProduceMana` carrying units, their source facts read by its arm; decision 2's split and its sweep; decision 6's spent record with its rename, on activations and in a permanent's last known information; `{S}` in the demand, the check and the window | The five Snow-Covered basics; Icehide Golem (`{S}` in a mana cost); one of the 35 cards with `{S}` in an activation cost; Graven Lore (`{S}` spent); Steamcore Weird (a creature asking what was spent, asked again once it has left); Arcum's Weathervane or its fixture (the source read once). Each checked by the row's card hunt | 650–850 | 500–650 | 1,600–2,400 |
| **`restricted-mana-plus-spend-riders` — mana that pays only some things, and mana that does something when spent** | Decision 3's restriction, `ManaPayment` and edge test, through the payment, the window and the check (the entry's restriction and the purpose argument of `#36-item-33s-seam-an-edge-refused-decision-2`); the cost's side, a pip that says what it accepts, with X's bound for such a pip; decision 4's event and three of its four riders | Mishra's Workshop; Soldevi Machinist (abilities of artifacts); a Powerstone maker; Consume Spirit (black mana on X); Scaled Nurturer (a delayed trigger); Boseiju, Who Shelters All (can't be countered); Hall of the Bandit Lord (haste). Each checked by the row's hunt | 600–800 | 550–700 | 1,600–2,400 |
| **`mana-kept-and-lost` — losing mana as an event** | Decision 5: the action and its batch at the two expiry sites and from an effect, the kept "can't" scoped to the step's end, the unit's own lifetime and its regrouping, the replacement and the action it rewrites to, Yurlok's loss | Upwelling; Omnath, Locus of Mana; Leyline Tyrant; Savage Ventmaw; Horizon Stone; Yurlok of Scorch Thrash; Mana Short | 350–500 | 400–500 | 1,050–1,600 |

**Where the first two split**, if a brief's re-count puts one past the band
(1,500 to 2,500 lines). `snow-mana`: the record, the pool and the spent
record first, with Steamcore Weird and every game `IDENTICAL`; then the
watch list, `{S}`, the split over classes and the snow cards.
`restricted-mana-plus-spend-riders`: the restriction and the cost's side
first; then the event and the riders.

**What the raw counts rest on** (the tree at `8c96e41a`): `types/mana.rs`
holds about 660 lines of pool and restriction code and 675 of tests, of
which the dormant subsystem is about half of each. `ProduceMana` is built
or matched at 34 sites and `special:` written at 38. The crate reads
`.mana_pool` at 42 sites; the 240 `mana_pool.add(type, n)` calls, nearly all
in tests, keep working as the way to add plain units. The payment's three calls have two
callers.

**Against what was carried.** `roadmap-v2.md`'s sizing table had the build
at 1–2 PRs: T12c (the archived plan's label for routing restricted mana into
the cast) at 150–250 lines, three items summing 200–320, and an unsized
source axis and migration. Re-counted, T12c's routing is the second row's
purpose plumbing, at seven call sites and the check; item 4's 40–60 lines is
decision 5's B, which holds 7 of the 19 cards that keep mana by a static
ability or an effect, or replace, follow or cause its loss; and the axis is
the first row. **Three PRs, about 4,250 to 6,400 lines.**

**What waits, and the row it waits in.** Each is designed above and has no
card in a row of this build.

| What | Cards | Waits for | Row |
|---|---:|---|---|
| Haven of the Spirit Dragon, Path of Ancestry, Lord of the Forsaken (`tdc`) | 3 | any-color mana; the commander's designation; casting from a graveyard | the row that frees each: `any-color-plus-painlands-plus-signets`, `game-formats`, `casting-from-other-zones` |
| "a Treasure" on the watch list, and its readers | 16 | Treasure tokens | `any-color-plus-painlands-plus-signets`, which makes the token, with Hired Hexblade as its reader |
| a type the source chose as it entered | 6 | the as-enters record's creature type (`#310-what-a-mana-ability-makes-decision-5`) | `chosen-color-plus-could-produce`, with Cavern of Souls |
| the kinds for a creature, an artifact, an artifact source, a basic land, a Cave and a Desert | 7 | nothing built: each widens what every pool tells apart, measured by its first card | `breadth-features`, with its first card |
| filter leaves a restriction reads: mana value, a keyword, "no abilities", a count of colors | 17 | the shared `ObjectFilter` table's leaf | `breadth-features`, with its first card |
| the special actions a restriction names | 7 | each action: turning face up, foretelling, unlocking a door | `face-down-and-morph` for turning face up; `breadth-features` for the other two |
| a rider that puts counters on the permanent | 4 | any-color mana (3 cards), a commander (Opal Palace) | `breadth-features`, with its first card |
| expend (CR 700.14) | 12 | a count of mana spent on spells this turn, read off decision 4's event | `breadth-features`, with its first card |
| Jegantha's mana, which pays no generic cost | 1 | companion | `breadth-features`, with its card |
| Drain Power's move between pools (CR 106.13) | 1 | forcing a player's mana abilities | `breadth-features`, with its card |
| Hogaak's cost, which no mana pays | 1 | convoke and delve | `convoke-delve-improvise` |
| which object made a unit, and the three lands that read it | 3 | transforming double-faced cards, which all three are | `breadth-features`, with its first card |
| a restriction that names a keyword: equip, power-up, a Class level, cumulative upkeep | 6 | `AbilityKeyword`, the tag on an ability's definition | `breadth-features`, with its first card |
| Soul Burn's `{B}` spent on X: the split asked for a part of the cost, and the record keeping that part's units apart | 1 | nothing built | `breadth-features`, with its card |
| Emblazoned Golem's cap of one mana of each color on X | 1 | a capacity in the cover test | `breadth-features`, with its card |
| Throne of Eldraine's cost, which takes only mana of a chosen color | 1 | the as-enters record's color | `chosen-color-plus-could-produce` |

### 11.7 The fuzz games, predicted

- **`snow-mana`, with its cards unregistered:** every gameplay row
  `IDENTICAL` to `main`. A plain pool is asked the same question with the
  same buckets in the same order (decision 2), and with no snow source in a
  game every unit's source is alike on the one watched kind.
  Instructions per decision within ±0.3%: derived, not measured. A payment
  reads a short list where it read a map, and there are about 70 payments a
  game on the budget's board (`#23-what-an-exact-check-changes-the-probe`'s
  1,397 over 20 games). Allocations per clone fall by up to one a seat
  wherever no mana floats (decision 1's probe).
- **With its cards:** the stream moves with them, read through `fuzz_ab
  --require`. A new question appears only where a pool holds snow and other
  mana of one type and need not spend both.
- **`restricted-mana-plus-spend-riders`, unregistered:** `IDENTICAL`, since
  no unit has terms. The check's purpose argument is the price
  `#36-item-33s-seam-an-edge-refused-decision-2` carried: about 2,100
  instructions a cost where restricted mana is present, and a branch where
  it is not.
- **`mana-kept-and-lost`, unregistered:** `IDENTICAL` on gameplay rows. A
  step that ends with mana in a pool now proposes its loss, about 22,500
  instructions a proposal (`#310-what-a-mana-ability-makes-decision-5`'s
  measured price for a production's). Decision 5's probe counted it: 1.3
  proposals a game at two seats and 2.7 at four, under a tenth of one
  decision's instructions a game, so the budget's reading should not move
  past its noise.

### 11.8 The fixtures

- **Re-scripted: none**, under decision 2's A. Under its B: the 10 test
  sites that script a split, the scenario format's `split` word, and the
  random agent's answer.
- **Renamed by a counted sweep:** `GenericManaAllocation`, 11 sites in
  `src` and 10 in `tests`; `ChoiceOption::ManaType`, 7 and 6, and one in the
  dev GUI; the record `ManaSpent`, which gives its name to the event, 43
  sites.
- **Removed:** `types/mana.rs`'s thirty unit tests of the dormant subsystem.
  The atoms they could have covered are covered by integration tests today.
- **Moved:** `mana_supply`'s two tests that build a `ManaAtom`, onto the new
  unit.

### 11.9 Found while designing

- **Item 403's second half and "Before Layers" item 4 are one build**, filed
  twice: once from the layers audit and once from the decks' gap census.
  Each item carries a line saying so. Their Slotted lines, and item 135's,
  move at the owner's answers, not before.
- **The emptying is a direct write no gate sees.** `check_invariants.py`
  gates a list of fields, and a mana pool is not on it, so `turns.rs` and
  `costs.rs` write pools outside `perform_action` untagged. Decision 5 routes
  the first. The second, spending, keeps its write and gains a tag and an
  event (decision 4). Adding the pool to the gate's list is a process change,
  put to the owner in `plans/handoffs/process-pr.md`, not made here.
- **`costs-and-choices-at-resolution` names Leyline Tyrant as a consumer**,
  and its static ability is this design's. Annotated in
  `choices-architecture.md`'s build table.
- **ATOM-106.3-001's expected line prescribes storage** ("in the `ManaAtom`
  Vec, not the fast-path HashMap"). The observable half is tested; the line
  is the owner's to reword or keep.
- **Myr Superion's ruling (2011-06-01) narrows "produced by creatures"** to
  creatures on the battlefield: Elvish Spirit Guide's mana, made from a
  hand, does not count. CR 109.2 is why, and 109.2c is why "a snow source"
  is wider: Tundra Fumarole, a snow sorcery, makes mana that pays `{S}`. So
  each kind on the watch list says whether only a permanent counts.
- **Imperiosaur's ruling (2021-03-19)** confirms CR 106.3 for triggered
  mana: Wild Growth's `{G}` is Wild Growth's, not the Forest's. The
  production's source is already the trigger's, so nothing changes.
- **A card's kind of source widens what every game asks.** Registering Myr
  Superion adds "a creature" to the watch list, and from then a pool holding
  an Elf's `{G}` beside a Forest's asks which pays, in games with no Myr Superion.
  That is the CR's choice once such a card exists, and it is asked only on a
  surplus. Six printed cards read a creature, an artifact or a basic land
  this way. The first to register measures the questions it adds in its own
  A/B.
