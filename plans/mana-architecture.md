# Mana — what a player can pay, and what mana abilities make

> **Status:** design, 2026-10-05, for `codebase-state.md` item 162, decided
> at #224's review the same day: the owner took every recommendation in
> §7, shape C for item 211 among them. MA-1 landed 2026-10-05 (§10); §3.3,
> §3.4 and §3.8 say where it built otherwise than designed. Every number in
> §2 to §6 is from a measurement made 2026-10-05 on `f4df90a` (#223's
> merge), and §2 says how. MA-2's build plan is §6.1, written 2026-10-17
> against `baf55ae` (#258's merge) and under review; §3.7 is amended where
> the plan refines it.
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
> - `cost-architecture.md`. CP-1 pays the symbol alphabet, and its check
>   half is §3.2 here. CP-2 owns the cost actions.
> - `codebase-state.md` item 33 (mana provenance, `roadmap-v2.md` B9), whose
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
mana ability makes. The payment's performing side stays CP-1's and item 33's.

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
- **MA-3's draw inside a mana ability is a frozen-text facility.** The Eggs,
  Chromatic Sphere and Selvala draw, and under the new text those abilities
  use the stack. B13's Ironworks loop (TR-7's until 2026-10-13) tests the
  frozen loop and needs it. A painland's
  damage moves no card, so it stays a rider under both texts, and so does
  the window inside a Signet's activation.
- **Millikin's and Deranged Assistant's "Mill a card"** is CP-2's single-arm
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
decision**: `engineering-practices.md` §3.1's budget lets a PR add 2.5
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
`can_cast` asks timing, targets and the other costs before mana;
`can_afford_ability_costs` walks the printed cost list, mana first on most
cards, and asks targets after. Asking the cheap refusals first is what takes
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
  kept for MA-6's solver, which needs the payment, not only its existence.
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

**How it runs.** The inventory keeps one table: for each of the 64 type
sets (128 once `{S}` needs a seventh bit), how much mana reaches it, built once with a subset-sum transform (384
additions). A generic pip accepts every type, so it enters only the total. A
cost with k distinct colored pip kinds needs at most 2^k − 1 comparisons;
k is 1–3 on nearly every printed card. **About 100 instructions a cost**,
measured on the board.

**Every symbol has a demand:**

| Symbol | Its pip kind | Exact? |
|---|---|---|
| `{W}` … `{G}`, `{C}` | one type | yes |
| generic | the total only | yes |
| hybrid `{W/U}`, and `{C/W}` | the two types | yes, by the same theorem |
| `{2/W}`, Phyrexian `{W/P}` and `{W/U/P}` | a choice per symbol, paid colored or another way (two generic, or 2 life): enumerate how many of each kind go each way, n+1 choices for n such symbols of one type | yes |
| `{S}` | a seventh bit, set on a snow permanent's entries | yes for permanents; the pool's snow mana waits for item 33 |
| `{X}` | none at the gate (§3.7) | yes |

**Until a payment path pays a symbol, the check refuses it.** Enumeration
must agree with enforcement (`cost-architecture.md` §3.6): the gate's hybrid
arm switches on in CP-1's PR, beside `ManaPool::pay`'s, and `{S}` with item
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
  whose outputs a cost's colored pips can use, on one player's board. MA-3
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
  a permanent sacrificed for mana cannot also pay the cost. Today the check
  counts the sacrifice free (143).
- **The cost's own consumption.** An activated ability whose cost includes
  `{T}` takes its own source's entry out. Chainbreaker with Citanul
  Hierophants' grant cannot tap itself to pay its own `{3}`; today's greedy
  counts it, and it is among the probe's 44. The same rule keeps
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
holds any. **About 2,100 instructions a cost where present.**
- **Snow** is §3.2's seventh bit.
- **The pool** is read through one function, today `ManaPool::available()`.
  Item 33's per-unit record replaces it with the units the purpose may
  spend, which `amount_for` already computes for the special atoms.
- **What this design builds:** the entry's restriction and the purpose
  argument. No card reaches them until item 33's build (`roadmap-v2.md` B9)
  makes restricted mana spendable.
- **What item 33 builds:** the per-unit record, the payment that reads it,
  and the source axis on the generic split.

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
  form, not searched (amended at §6.1, decision 2, which the owner asked
  for at #259's review: a search cannot survive a loop's mana, CR 731.2a).
  X adds only generic mana, and no cost change's size depends on the
  spell's X, so the total is the same pips beside `a − r + kX` generic,
  read off 601.2f's own arithmetic in `cost_determination`, and the
  inventory says once how much generic it can pay beside those pips. The
  `r` is why the arithmetic is 601.2f's and not the printed cost's: a
  reduction can absorb part of X, so at X = 0 a `{1}` reduction on
  `{X}{R}` is wasted, and at X = 1 it is not. A larger X never costs less
  while no cost change's *applying* depends on X either, so this part is an
  interval. One whose filter reads the spell's mana value breaks both
  (§6.1): on the stack the mana value counts X (CR 202.3e), and Stonecoil
  Serpent under Krosan Drover costs `{5}` at X = 5 and `{4}` at X = 6. No
  filter can read a mana value today. **The owner's condition** (2026-10-17,
  at #259): if the bound ever costs materially, the engine falls back to
  §6.1's option B, no bound, while the algorithm improves.
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
- the loyalty −X, with CP-2's counter arms (an amount that can be X);
- a target's mana value, with the first card that prints it, in C;
- X in an activated ability's mana cost, CP-1's;
- X in an additional cost, CP-2's amount shape;
- a "can't cast" that names a quality X changes (CR 601.3a: Void Winnower's
  even mana values), with RS-2's cast prohibitions. `cant-effects-architecture.md`
  §4.3 plans a reversal after the announcement; the set form lets the same
  check cut instead (amended at §6.1, which says why both are the CR's);
- a minimum that depends on how the spell is cast (Light Up the Night's
  flashback: "If you cast this spell this way, X can't be 0"), with its card.

The base 601.2b chose decides whether X is announced at all (amended at
§6.1): an alternative cost with no X leaves 0 the only legal value (CR
601.2b announces "a variable cost that will be paid"; 107.3b says the same of
an effect's free cast), and an alternative cost's own X is expanded as the
mana cost's is (107.3a).

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
| delve (28) | each card in the graveyard: generic only | CP-2's exile-from-graveyard cost shares the same cards |
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
  ability that requires a mana payment"). B13's Ironworks loop needs it for
  Chromatic Sphere's `{1}` (`roadmap-v2.md` B13), and Signets need it too.
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
  policy at construction). Built in MA-6, beside the person's-seat solver.

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

### 3.13 The why of the three mana prompts

`setup-architecture.md` §7c and `cast-census.md` §9 give these kinds typed
reasons "with item 162's build":
- **`ManaAbilityWindow`:** the refusals are the inventory's, one
  `CannotPay` per mana ability it did not make an entry of, saying why.
- **`ChooseXValue`:** the bound and each cut (§3.7). "X above N leaves the
  cost unpayable: N plus the cost's other pips is all the mana this board
  makes." "X = 2 is not offered: no artifact has mana value 2."
- **`GenericManaAllocation`:** the clamp. A bucket's maximum is the pool's
  mana of that type less the pips that need it.

The first lands with MA-1, the second and third with MA-2.

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
| One choice for several mana: "any one color", a multiplier on a multi-type entry | 54 | §3.3 enumeration | 2,800 a cost where present; bound 9.5K | MA-1 (the multiplier, on the board); MA-3 (the printed form) |
| Mana-fed: converters and filters | 175 | §3.3: Signet-shaped exact by a base-mana pip; identical ones counted; other shapes ordered, up to three | 3,600 a cost where present; bound 9.6K | MA-1 (Doubling Cube, registered, exact); MA-3 (Signets) |
| Alternatives of different sizes | — | §3.3: each alternative tried, dominated ones dropped | not measurable on the board (4 inventories) | MA-1 |
| Sacrifice-fed, another permanent | 26 | §3.3 count less the cost's own | one candidate count an inventory where present | MA-1 (Ironworks, registered) |
| Sacrifice-fed, itself | 117 | an ordinary entry | 0 beyond the plain check | MA-1 |
| Variable amount; the doubled pool | 60; 1 | evaluated at the inventory (605.3b) | one `evaluate_amount` a source | MA-1 |
| One mana of any color; Treasure makers | 306 + 21, 347 | a five-type entry; the choice at resolution (§3.10) | 0 at the gate; one prompt an activation | MA-3 |
| Painland riders | 129 | gate unchanged; the rider through the chokepoint | 0 at the gate; about 22.5K an activation | MA-3 |
| The chosen color | 32 | the recorded bit; `backlog.md` §2.2's record, its mana half | 0 beyond a read | MA-4 |
| "Could produce" | 14 | the union, a fixpoint, memoized | one pass over the named set a memo miss | MA-4 |
| Restricted mana | 153 | §3.6, an edge refused | 2,100 a cost where present; bound 5.6K | the seam in MA-1, the cards in item 33's build (`roadmap-v2.md` B9) |
| X | 527 | the smallest legal X at the gate; the legal set at 601.2b, the bound in closed form (§6.1, decision 2), cut by the other rules (§3.7) | 159 a cost; bound 0.4K | MA-2; each cut with its surface |
| Hybrid, `{C/W}` | 582, 1 | a two-type pip | 0 beyond the plain check | CP-1, against §3.2 |
| Mono-hybrid, Phyrexian | 20, 36 | counts enumerated | 1,300 a cost where present; bound 3.6K | CP-1, against §3.2 |
| Convoke | 104 | §3.9 | 2,600 a cost where present; bound 6.9K | MA-5 |
| Improvise, delve | 23, 28 | §3.9, generic only | 2,350 a cost where present; bound 6.3K | MA-5 |
| Waterbend | 28 | §3.9, an eighth bit | as convoke | MA-5 |
| Assist, offering | 16, 6 | §3.9 | another player's inventory; one check a candidate | MA-5, or C with the card (§6) |
| `{S}` | 2 | §3.2's seventh bit | 0 beyond the plain check | item 33 (B9) |
| The window's inventory | — | §3.8 | −4.5K | MA-1 |
| Mana abilities at priority (item 211) | — | §3.11, shape C | 0 by default | MA-6 |
| The solver and its decorator | — | §3.12 | ~14K a cast, a person's seat only | MA-6 |

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
`fuzz-record.md`'s MA-1 block. What carries to MA-2 to MA-6: a change that
moves what the priority question offers reads the budget off a cost arm
(today's answers, the new machinery run black-boxed), and "Actions
reversed" (CR 732.1's word) prices an offer no payment covers, each one a
re-ask that moves the agent's stream.

---

## 6. The build, sized

Each PR carries a consumer and lands in or under the band
(`engineering-practices.md` §4). Sizes are code plus tests, and the
card-by-card hunt runs in each PR's brief.

| PR | Shape | Size | Consumer | Closes |
|---|---|---|---|---|
| **MA-1, the inventory and the exact check** | Landed 2026-10-05 (#225): §10, and the archive's "MA-1" for the shape as sized | | | item 162's offer customer |
| **MA-7, the check widened** (MA-1's review, 2026-10-06) | Every cost a mana ability can have, read per `Cost` arm with no wildcard into what one use spends: its tap (once a payment, since only `{Q}` untaps inside one), itself, counters (their count over N uses), life (one budget with the action's own life payment, CR 119.4), others' sacrifice (one fodder pool), mana in (a pool multiplier's input; a converter's is MA-3's rule). Sorcery timing read, and CR 602.5's "activate only if" conditions once `permission-architecture.md` gives them a surface, in the order one window can change them (Mox Opal beside Krark-Clan Ironworks). The fold of replacement effects by state rather than order: 2^n·n! leaves at six effects become at most 2^n states. The choice search as a dynamic program over what the cost's pips still need, exact with no cap. **Its first commit is the stress board**: Nyxbloom Ancient registered beside Mana Reflection, fifteen distinct dual lands, a pool multiplier and three- to five-color costs, read in callgrind before the dynamic program and after | ~350–450 engine, ~300–400 tests: 650–850 | the stress board; the 116 printed cards whose mana costs life, counters, a tap with another sacrifice, or `{Q}` | item 213; MA-1's review, themes C and D |
| **MA-2, the legal values of X** | §3.7: the smallest legal X at the gate; the legal set at 601.2b (the bound in closed form, §6.1's decision 2; the printed minimum) and `ChooseXValue`'s set form, with a fixture whose set has a gap; the random agent's self-limit out; the why of `ChooseXValue` and `GenericManaAllocation`. Registers and pools an X spell (Blaze is the plain one). Makes TR-5b's Protean Hydra, registered and pooled, castable. **The plan is §6.1** | ~350 engine, ~450 tests, a card: 800–950 as sized here; §6.1's plan, with item 214 and what the rules pass added, ~600–640 code and ~720 tests: 1,320–1,360 | Blaze, Protean Hydra | `cast-census.md` §8's X row |
| **MA-3, any color, riders, the nested window** | §3.10's spec type and its sweep of `ManaOutput` sites; the choice at resolution, a `ChoiceKind`; riders through the resolver; a mana ability's mana cost opening a window; §3.3's converter rules, with the combination cap stated as a cost and counted in fuzz. Cards: Birds of Paradise, a painland (Llanowar Wastes), Gilded Lotus ("three of any one color"), a Signet, a Treasure maker with `backlog.md` §2.27's Treasure | ~700 engine, ~800 tests: 1,500–2,000 | five cards; the Ironworks loop's three facilities (`roadmap-v2.md` B13) | `backlog.md` §2.19 |
| **MA-4, the chosen color and "could produce"** | `backlog.md` §2.2's record, its mana half (the choice at entry, `EnterMods`, the permanent's field); "could produce" with its fixpoint. Cards: Thriving Grove, Coldsteel Heart, Exotic Orchard, Reflecting Pool | ~550 engine, ~650 tests: 1,200–1,500 | four cards | `backlog.md` §2.2's mana half |
| **MA-5, paying with permanents or cards** | §3.9: the entries; the payment prompt in `plan_payment`; the taps and exiles through the chokepoint. Cards: Stoke the Flames, Treasure Cruise, Reverse Engineer, a waterbend card. Assist and offering go here if the hunt keeps it in band, else to C with a card each | ~900 engine, ~1,000 tests: 1,900–2,400 | four cards | `cast-census.md` §7's payment row |
| **MA-6, a person's seat: the solver and item 211** | §3.12's solver and decorator, wired in `cli_play` and the dev GUI behind the toggle, with a `fuzz_games` flag off by default; §3.11's shape C (a `SeatMode` field, the "add mana" entry, CR 605.3a's window at priority). A GUI PR: `engineering-practices.md` §10's review | ~600 engine and client, ~600 tests: 1,100–1,400 | the CLI and dev GUI person's seat | `backlog.md` §2.18's solver (§2.22 rows 6 and 7), item 211 |

**Ordering.**
- **MA-1 landed before SU-8** (decision 7). It removed the wrong offer
  SU-7's click script met on the review board, and wrote the why's
  `ManaAbilityWindow` line SU-8 would have read off its inventory.
- **MA-2 moved up to right after TR-5b** (the owner, 2026-10-17, at #258):
  TR-5b's Protean Hydra is an X spell the gate cannot offer. Its inputs are
  built, MA-1's check and the cost determination's X, and MA-7's mana
  ability costs are no part of its X arm.
- **MA-7 first** (the owner, 2026-10-06, at #225's review): it widens the
  check that MA-3 to MA-6 extend. It is not urgent for today's games: the
  choice search never passed 6 leaves in about 150,000 checks over the four
  fuzz boards (2026-10-06). It is the guarantee for v1's Commander boards,
  where Nyxbloom Ancient and many lands do meet.
- **MA-2 to MA-6** go in `roadmap-v2.md` B, before C. **MA-3 before B13**,
  the Ironworks loop (TR-7's until 2026-10-13), which needs any-color mana, a
  draw inside a mana ability, and a window inside a mana ability's activation.
- **CP-1 after MA-1**: its gate half is §3.2's arms.
- **Item 33's design pass after MA-1**: its seam is §3.6.

### 6.1 MA-2's build plan (2026-10-17, for review)

**Where it sits.** On the casting path, at four points (`engine-map.md`'s
CR 601.2 row). Lines are `baf55ae`'s.
1. **The offer.** At each priority question `can_cast_with`
   (`oracle/mana_helpers.rs:110`) previews the card's cost and asks the
   inventory whether it is payable. A cost with `{X}` has no demand
   (`ManaCost::demand`, `types/mana.rs:166`), so `ManaSupply::covers`
   (`oracle/mana_supply.rs:312`) refuses it, and no X spell is offered.
2. **CR 601.2b, the announcement.** `cast_spell` asks `ask_choose_x_value`
   (`ui/ask.rs:603`, from `engine/put_on_stack.rs:170`) after the
   alternative and additional costs, over `(0, u64::MAX)`.
3. **CR 601.2f, the total.** `determine_total_cost`
   (`engine/cost_determination/total.rs:39`) turns each X into that much
   generic mana before cost changes apply. It does so for the mana cost
   only: an alternative cost's X stays a symbol nothing pays.
4. **CR 601.2h, the payment.** An X the player cannot pay is found here, and
   the cast is reversed (CR 732.1).

MA-2 changes the first two, and the third's alternative-cost half. The
payment does not change: every X the prompt offers is one it can pay.

**The pieces.**

| # | What | CR | Where |
|---|---|---|---|
| 1 | X expanded by one function | 107.3a, 601.2f | `ManaCost::with_x_as(x)` in `types/mana.rs`. `determine_total_cost`'s step 1 calls it on whichever base 601.2b chose, so an alternative cost's X (miracle's `{X}{R}`) is expanded too |
| 2 | The total as a function of X | 601.2f; `cost-architecture.md` §3.6 | beside `preview_mana_cost` (`total.rs:135`): step 1's assembly and steps 3–6's arithmetic run once on the 601.2b choices with the generic part kept as a count, giving the pips and the a, r, k and F of the most's closed form (below). `determine_total_cost` reads the same run at the announced X (`modify`, `:77`, takes the reduction order as an argument), the reductions in gather order for the preview (`cost-architecture.md` §3.4's theorem), no prompt |
| 3 | "X can't be 0" | the card's text | `CardData::x_cannot_be_zero`, read as printed: the gate reads the card in hand and the cast its snapshot from before the CR 601.2a move, as both read the mana cost |
| 4 | The gate tests the smallest legal X | 107.3a | `can_cast_with` previews the printed cost at X = 0, or 1 under "X can't be 0". A cost with no X skips the expansion and allocates nothing |
| 5 | The legal values at 601.2b | 107.3a, 107.3b, 601.2b | a new function in `oracle`, `legal_x_values`, over the base, the announced additional costs and one inventory read (below); the inventory's most generic beside a cost's pips is a mode of `ManaSupply`'s existing walk (`oracle/mana_supply.rs:331`), never a second matcher |
| 6 | `ChooseXValue` carries them | 601.2b | the kind (`ui/choice_types.rs:50`) gains the legal values and their reasons. Asked by `pick_number(min, max)` when they are an interval, by `pick_n` over `ChoiceOption::Number` when they are not, and not asked when there is one |
| 7 | The random agent's cap goes | — | `ui/random.rs:335`. It caps X at the pool plus the untapped lands, counting the land that pays `{R}` toward X as well: with four Mountains it picks X = 4 one time in five, and that cast is reversed. It also misses Sol Ring's second mana. The engine's bound replaces it |
| 8 | The why of `ChooseXValue` and `GenericManaAllocation` | 601.2b, 601.2h | `ui/why.rs:362`, `:372`: the X lines below, and the generic split's clamp from one function the ask (`ui/ask.rs:829`) and the why share (§3.13) |
| 9 | The scenario word `x N` | — | `scenario/text.rs`, `board.rs`, `build.rs`, `setup.rs:156`: a `then: … casts` line's `x N` segment answers the X prompt, and a value the engine does not offer stops the line. `setup-architecture.md` §5.1 gave it to "the PR that offers one" |
| 10 | Item 214, the Doubling Cube preference | — | `mana_window_preference` (`ui/random.rs:63`), the first commit, with item 214's test on the Humility board |
| 11 | Blaze | 107.3a | `cards/phase_ma2_cards.rs`, registered and pooled. It replaces `put_on_stack.rs`' `make_x_spell` fixture (`:926`), which is Blaze unregistered. Scryfall, 2026-10-17: `{X}{R}` Sorcery, "Blaze deals X damage to any target.", no rulings |

**The legal values (piece 5)**, decided in this order:
- **The base.** If the cost 601.2b chose has no X (an alternative cost
  without one), 0 is the only legal value: CR 601.2b announces "a variable
  cost that will be paid", and 107.3b says the same of an effect's free
  cast. Today the engine asks X there, pays the alternative cost, and keeps
  the announced X: Blaze cast for an X-less alternative cost would deal any
  X. No registered card has both, so it is not reachable.
- **The least.** 0, or 1 under "X can't be 0".
- **The most, in closed form (decision 2).** Two facts make it one
  division instead of a search. X adds only generic mana. And no cost
  change's size depends on the spell's X: `representable` (`total.rs`)
  already refuses an X in an increase or a reduction, and a `ReduceGeneric`
  amount is read off the board. Golden-Tail Trainer's "cost {X} less, where
  X is this creature's power" is that kind: its X is its own, defined by its
  text (CR 107.3c) and read once at 601.2f, so it is a constant in r. (Its
  ruling: a change to the power after the lock changes nothing.) Whether a
  change *applies* is the next bullet. So at every X the total is the same
  pips P (after reductions)
  beside `max(F, a − r + kX)` generic, where:
  - a is the cost's generic mana besides its X, increases included;
  - r is what the reductions take off generic, a colored reduction that
    finds no pip of its type included (CR 118.7b–d);
  - k is the number of X symbols;
  - F is what a Trinisphere-like floor adds (CR 601.2f's direct effects).

  All four come from one run of 601.2f's own arithmetic, kept as numbers
  instead of symbols (piece 2), so the preview and the cast still share one
  arithmetic (`cost-architecture.md` §3.6). The inventory answers once how
  much generic mana it can pay beside P, with the announced costs'
  reservations: call it G. That is the existence check's own walk with a
  maximum at its leaves instead of a yes, since generic mana enters Hall's
  condition only through the total (§3.2), and the choice search's pruning
  reads only the pips:
  - on the plain split, all the supply less P, when P can be paid;
  - where one tap makes several mana of one chosen type, the best leaf;
  - past a pool multiplier, its transform run forward: what is made before
    it, less its input, times its factor, plus what is made after (§3.3).

  The bound is `floor((G + r − a) / k)`, when F ≤ G. The cost is one
  inventory read, one pass of the arithmetic and a division, the same at
  X = 3 as at X = 10^34. The four numbers describe the arithmetic, not a
  cast at X = 0, so "X can't be 0" changes only the least: the legal values
  are the least to the most, and none when the most is below the least. A
  `{1}` reduction on an X-can't-be-0 `{X}{R}` has a = 0, r = 1, k = 1, so
  X = 1 costs `{R}`: the reduction absorbs the one X.

  **Checked twice.** A test runs the closed form against the exhaustive
  search MA-1's property test already has
  (`the_check_agrees_with_an_exhaustive_search`, `oracle/mana_supply.rs`),
  extended from "does it pay" to "the most generic it pays", on the same
  random boards. And in debug builds an audit asks `covers` at the bound
  and one above it, so a closed form that drifts from the check fails a
  test or a fuzz run, not a game. Past the choice search's cap (§3.3) the
  maximum leans high, as the check leans yes.

  **What it rests on.** The bound is as exact as the existence check, and no
  more; `codebase-state.md` item 213 lists where the check is not, each
  shape one no registered card has. The check's evidence is three things.
  The property test, 3,000 random boards of §3.3's shapes with copies and
  pool multipliers, each answer more than 500 times, against a search that
  tries every payment. A test that forks the game, taps each registered
  mana ability with 16 seeds, and compares what it made with what the
  inventory predicted (`what_the_inventory_predicts_a_tap_makes_is_what_the_engine_makes`).
  And fuzz: each of the 64 reversals MA-1's close-out read in traces was
  the random agent's misplay, none an over-offer (`fuzz-record.md`, MA-1's
  block). The property test leaves out sacrifice outlets, a cost's own
  reservations and restricted mana, which are tested board by board.
- **A cost change that applies by mana value** (found at #259's review).
  On the stack a spell's mana value counts X (CR 202.3e), so a change whose
  filter reads it applies at some X and not others. Six paper cards print
  one, read by hand from 16 matches on Scryfall, 2026-10-17:
  `o:/mana value (\d+|x) or (greater|less)/ o:/cost(s)? \{[0-9X]\} (less|more) to cast/ -is:digital`.
  Krosan Drover, Thryx, Spectacle Mage, The Scarlet Witch and It
  That Heralds the End reduce, and Reidane increases. With one, the total
  is affine only between thresholds: "mana value m or greater" is
  `X ≥ ceil((m − the rest's mana value) / k)`. Three things then fail.
  - A larger X can cost less. Stonecoil Serpent (`{X}`) under Krosan Drover
    costs `{5}` at X = 5 and `{4}` at X = 6.
  - The payable values can have a gap: from four mana, X is 0 to 4, or 6.
    The set form carries it (decision 1).
  - The least need not be the cheapest. A creature spell costing
    `{X}{3}{G}{G}` under Krosan Drover costs five at X = 0 and four at
    X = 1, so the gate must test each threshold's first X, not only the
    least.

  **Not built in MA-2, and not reachable**: no `ObjectFilter` arm reads a
  mana value, so no cost change can apply by one, and the total is one
  piece. **Guarded, so the first such arm cannot pass silently.**
  `filter_reads` (`engine/layers/board.rs:744`) matches every
  `ObjectFilter` arm with no wildcard, and its `MANA_COST` channel is set
  by no filter today. A mana-value arm cannot compile without an arm there,
  which is where it says it reads `MANA_COST`.
  `legal_x_values` asks it of each gathered cost change's filter, and a
  filter that reads `MANA_COST` fails a debug assertion naming this
  bullet. The thresholds, the per-piece division and the gate's per-piece
  check land with the first mana-value filter, beside CR 202.3e's value on
  the stack (`cost-architecture.md`'s CP-1 row). `codebase-state.md`'s
  Deferred Migrations gets a line naming it. A second guard, debug-only:
  when the bound is under 64, every X from 0 to the bound plus 8 is checked
  with `covers` against the closed form's set, which catches any other
  shape the pieces miss.
- **With `GameNumber`** (`backlog.md` §2.28; `roadmap.md` D26;
  `decision-provider-refactor.md` §9's planned shape: `Finite(u64)`;
  `Shortcut`, a loop's iterations times what each makes, either of them
  itself a `GameNumber`; and `Relative`, a number times a whole-number
  multiplier plus an offset, which is "your number plus one"), G is a
  number a loop made: CR 731.2a's "a loop that repeats a specified number of
  times". The bound stays the inequality `kX + a − r ≤ G`.
  - **k = 1** (Stroke of Genius, Awaken the Woods): the most is
    `G + r − a`, a `Relative` of G. The prompt can offer it as a number.
  - **k ≥ 2** (Gelatinous Genesis, `{X}{X}{G}`): the most is
    floor((G + r − a) / k). The rule is plain: divide, round down, and the
    remainder, fewer than k mana, stays unspent in the pool. Which types
    stay is the payment's generic split (CR 601.2h), the player's or the
    agent's choice, not the bound's. What is missing is a way for the
    planned type to *hold* that number, since no planned variant divides.
    Sometimes the engine could rewrite it. A nested loop of 10^18 iterations
    making 10^18 green each leaves 10^36 green, more than a `u64`; the `{G}`
    pip takes one, so G = 10^36 − 1, and the most is 5 × 10^35 − 1, a
    `Shortcut` of 10^18 times 5 × 10^17, less one. With an odd count of
    iterations, or a pool that is "your opponent's number plus one" where
    that number is itself symbolic, no rewrite exists: the value is
    well-defined, and the type cannot name it. Checking an announced X
    always works, since `2X ≤ G` is a comparison the planned `PartialOrd`
    makes. But a player cannot announce the most: the planned declaration
    is a `u64` or "theirs plus N" (§9.6 there), so there is no "half my
    pool". `GameNumber` needs a floor-quotient node, floor(E / k) for a
    constant k, kept unevaluated and compared through `k·c ≤ E`. MA-2 keeps
    the parts (`MostX` carries G, a − r and k), so that node can be built
    from them (§9).
- **Empty.** If the least is not payable, the cast is reversed at 601.2b,
  with the reason 601.2h would give, before a window opens for a payment
  that cannot happen. Only an announcement can take what the gate counted: a
  kicker on an X spell, or 107.3b's 0 under "X can't be 0", which leaves no
  legal value at all (CR 601.2e).
- **One value** is not asked: `backlog.md` §2.22's rule 1, A4e's. Blaze with
  only `{R}` casts with X = 0 and no prompt.

**The shape (pieces 6 and 8).** A sketch; the names are the build's, and the
PR body lists them.
```rust
pub struct LegalXValues { pub offered: XOffer, pub least: LeastX, pub most: MostX }
pub enum XOffer { Range { min: u64, max: u64 }, Values(Vec<u64>) }
pub enum LeastX { Zero, CannotBeZero, NotInTheCostPaid } // 107.3a; the text; 601.2b and 107.3b
pub enum MostX { Payable { generic: u64, fixed: i64, per_x: u64, beside: ManaCost }, TheLeast } // 601.2h: G, a − r, k, P
```
The kind carries them, because the announcement's other choices are not on
the stack entry yet when X is asked, so the why cannot work them out again.
Its lines, about the spell, at the question:
- "X can be 0 to 4."
- "X above 4 leaves the cost unpayable: each X adds {1}, and beside {R}
  Player 0 can pay at most 4 generic mana." (CR 601.2h)
- "X can't be 0: the card says so."
- "X is 0: the cost being paid has no X." (CR 601.2b, 107.3b)
- the generic split's: "At most 1 red toward the generic part: the pool's 2
  red, less the {R} that needs it." (CR 601.2h)

A gap's reason arrives with the cut that makes it (§3.7's owners).

**The set form with no cut (decision 1).** No rule MA-2 builds cuts a gap,
so no game reaches `XOffer::Values` until a cut lands. Under decision 1's
recommendation it is driven directly: a test hands the ask Tamiyo's set
from §3.7, {0, 1, 4, 6, 7}, and checks that the random agent answers only
from it over many seeds, that a scripted `Number(4)` answers it by value,
that a decision log's line replays it, and that the setup word refuses
`x 2`. `codebase-state.md`'s Deferred Migrations gains a line: a shape no
producer reaches, CP-2's loyalty −X the first.

**The atoms.** One Blaze test on `ATOM-107.3a-001`'s and
`ATOM-601.2b-004`'s board, `// COVERS:` both: five mana, X = 3,
`x_value == Some(3)`, `{3}{R}` paid, one mana left, and the prompt offered 0
to 4. `ATOM-107.3b-001`, `// COVERS-PARTIAL:` by the alternative-cost
fixture: its free-cast effect lands in Phase 8. `ATOM-601.2b-007`, X only in
an additional cost, stays CP-2's. **A staged fixture cannot check the
question**, so one test casts Blaze from hand at a real priority question
from four untapped Mountains under `ManaWindowStop`: offered, X from 0 to
3, cast with 3, every Mountain tapped.

**Commits, each an arm** (`engineering-practices.md` §3.1; §5's carry-over:
a budget is read only off identical games).

| # | Commit | Arm | Predicted |
|---|---|---|---|
| 1 | Item 214's preference (piece 10) | **cube** | differs from `main` where Doubling Cube is dealt, as it is pooled. "Actions reversed" on the performance pool from 2.3 a game toward 0: 42 of item 214's 46 reversals in 20 games were the agent declining a window the Cube covered |
| 2 | X's arithmetic (pieces 1, 2) | — | no game moves |
| 3 | The legal values, the printed minimum (3, 5) | — | no game moves |
| 4 | The gate, the announcement, the prompt's forms, the agent's cap out, the fixtures moved (4, 6, 7) | — | changes only games that cast an X spell |
| 5 | The why (8) | — | no game moves |
| 6 | The scenario word (9) | **engine** | with Protean Hydra unregistered (below), **IDENTICAL** to *cube* unregistered the same way |
| 7 | Blaze registered and pooled; the Hydra's cast path tested (11) | **shipped** | differs wherever Protean Hydra or Blaze is dealt; reported against *cube* |
| 8 | The close-out's docs | — | — |

**The pair that must read IDENTICAL.** Making Protean Hydra castable changes
every game that deals it, so *engine* cannot read IDENTICAL against anything
with the Hydra registered. The pair is *cube* and *engine*, each with a
throwaway commit in a detached `C:/w/arms` worktree that takes Protean
Hydra out of the registry and the pool, never on the branch. Then no
registered card has an X, and the X work may change no game: every gameplay
row IDENTICAL, both pools, two seats and four. **Its base is *cube*, not
`main`**: commit 1 changes the games that deal the Cube, so `main` and
*engine* differ for a reason that is not the X work. The pair is also the
budget's reading: callgrind instructions per decision on the Commander
board, predicted within ±0.1%, since a card with no X pays one scan of its
symbols at the gate.

**The rest of the close-out.** Two `close_out.py` sittings, since it
compares every arm with its first: the patched pair, *cube* first, which
reads the IDENTICAL table and callgrind; then *main*, *cube* and *shipped*
with `--no-cpu`, *shipped* last so that its `--require` reachability pass
runs there (it reads only the last arm, and `main` has no Blaze). Each arm's
games run once. The pass must cast Protean Hydra and Blaze, and reports how
often, with which X, and "Actions reversed" from X casts, predicted 0. Then
Blaze's effect on how `stress` games end, as for the last cards (a burn
spell, so some games end by damage to a player); the 1,000-game audited
smoke; and a `fuzz-record.md` block. The ratchet re-bases on *shipped*,
which moves the stream.

**The fixtures that move.** Eight sites answer `ChooseXValue` today:
- `put_on_stack.rs`' three. X = 3 from four red keeps its answer (the bound
  is 3). X = 0 from `{R}` loses its answer: one value, not asked.
  `test_cast_x_spell_insufficient_mana_rollback` asks X = 3 from two red,
  which is never offered; it becomes the bound's test (X from 0 to 1), and
  its rollback goes to the empty set's.
- `tests/`' five. `phase_cm_integration_test.rs:1166` (Foundry Inspector, X
  = 4 from `{3}` under a `{1}` reduction) is §3.7's case and its bound;
  `phase_tr3a_integration_test.rs:908` (X = 3 from exactly `{3}{G}`) and
  `phase_tr5b_integration_test.rs:824` (X = x from x + 1 green; no call uses
  x = 0) answer their bounds; `phase_su8_integration_test.rs:208` and
  `prompt_subject_test.rs:222` build the kind and gain its field.
- Unit tests that build the kind: `ui/ask.rs` (3), `ui/random.rs` (1).

**The size.**

| Piece | Code | Tests |
|---|---:|---:|
| Item 214 (10) | 40–80 | 40 |
| X's arithmetic, the alternative cost's X (1, 2) | 50 | 60 |
| The printed minimum, the gate (3, 4) | 30 | 60 |
| The legal values, the announcement, the prompt, the agent (5–7) | 170 | 220 |
| The closed form: the total in X, the most generic, the audits and the mana-value guard (decision 2) | 140 | 90 |
| The why, both kinds (8) | 80 | 80 |
| The scenario word (9) | 60 | 50 |
| Blaze and the Hydra's cast path (11) | 30 | 80 |
| The fixtures that move | | ~40 changed |
| **Total** | **~600–640** | **~720** |

1,320–1,360, against the row's 800–950 plus item 214's 40–80. The
difference is four things the row did not price: the generic split's why
(§3.13 gave it to MA-2), the scenario word (setup-architecture's promise),
the alternative cost's X (the rules pass), and the closed form, tested
against the exhaustive search and audited against `covers`, with its
mana-value guard. MA-1 ran about 1.5 times its code sizing; at that rate this is about
1,650, inside the band.

**What the close-out owes the docs.** `cast-census.py`'s `{X}` row built,
its activated-ability X row's owner corrected (§9), §8's prose and §10's
line, the census regenerated with `--no-fetch`. `setup-architecture.md`
§5.1's X row becomes the word, and §7c's two kinds are built.
`codebase-state.md`: item 214 closed, the CR 107 row, the Deferred
Migrations line. Blaze's line in the rulings ledger (no rulings), the
`PERFORMANCE_POOL` entry, `cards-unlocked-ledger.md`. §3.13 here landed;
§10's ✅ heading with a stub of 40 lines or fewer, this section to the
archive; `state-of-play.md` regenerated.

**Decisions.** Both taken.

1. **The set form with no cut. Taken: A** (the owner, 2026-10-17, at
   #259's review). The problem: MA-2 builds the set form (§3.7), and nothing
   MA-2 builds produces one.

   | | **A. Built, tested directly** (recommended) | **B. An early cut** | **C. Wait for the first cut** |
   |---|---|---|---|
   | In code | the ask's `Values` branch, the scripted provider's `Number` answer to a `pick_n`, the setup word's refusal: ~35 lines and ~60 of tests | a cut written now on a fixture, the loyalty −X on a fixture planeswalker | nothing here; CP-2 writes the branch with the loyalty −X |
   | Performance | none: no game builds a set | none | none |
   | Upkeep | one shape with no producer, in Deferred Migrations. The clients move once, in the PR that already changes the prompt (its range goes from `(0, u64::MAX)` to the engine's bound), and every cut after only produces | a second copy of CP-2's counter-amount work, which CP-2 replaces | the prompt changes shape again in CP-2: the CLI, the dev GUI, the scripted provider, replay and the setup driver move a second time, and the dev GUI job reruns. Reopens §3.7's "MA-2 builds the set shape" (decision 6) |

2. **Whether the engine bounds X by what the board can pay, and how.
   Taken: A, on a condition** (the owner, 2026-10-17, at #259's review):
   if the bound ever costs materially, the engine falls back to B while the
   algorithm improves. **Material, as measured:** the bound's inclusive
   share in callgrind (`legal_x_values` and the walk under it) past half a
   point, 0.5% of instructions per decision (§2.2's unit), on
   `close_out.py`'s boards. Every close-out that changes the inventory or
   registers an X card reads it, and its `fuzz-record.md` block says the
   number. The fallback is local: `legal_x_values` stops computing the
   most, the interval's top opens, and the agents take back a cheap cap
   of their own.

   The problem: a search for the bound cannot
   survive a loop's mana. CR 731.2a's shortcut repeats a loop "a specified
   number of times", so a pool can hold more mana than a `u64` once
   `GameNumber` lands (`backlog.md` §2.28), and Stroke of Genius or Awaken
   the Woods is then cast for that much. The plan's first draft searched by
   doubling, about 113 steps to bracket 10^34 and as many to bisect (a step
   count that is fine), but every step expanded X into one generic symbol
   per mana, which is not. Arena does not compute the bound, beyond rule
   cuts such as Tamiyo's (the owner's account). Both options keep the gate
   (the smallest X) and every rule cut.

   | | **A. The bound in closed form** (recommended) | **B. No bound: the payment decides, as Arena does** |
   |---|---|---|
   | In code | the total as `a − r + kX` beside fixed pips, from 601.2f's arithmetic; the inventory's most generic beside the pips, a maximum at the existence walk's leaves; a debug audit at the bound and one above; ~100 lines, ~60 of tests | `ChooseXValue` offers the least and up, cut only by rules; ~60 lines fewer than A |
   | Performance | once per X cast, never at a priority question: one inventory read, about 18,000 instructions (§2.2's 26.3K a decision over 17,906 reads), plus one pass of the arithmetic; the same at any X | nothing at 601.2b |
   | Agents | pick from what they can pay: the random agent draws in [least, most], and a learning agent sees each payable X (§3.7) | need their own cap. Today's reverses one cast in five with four Mountains; an exact one is A's computation, done in the agent, a second matcher |
   | `GameNumber` | the bound is an inequality, `kX + a − r ≤ G`, one comparison per answer; affine in the planned shape when k = 1 | the prompt is "the least or more"; 601.2h's payment makes the same comparison |
   | 601.2f's expansion, one symbol per mana (§9) | reached only up to what the board makes, so no X a person types can exhaust memory today; with `GameNumber`, generic must become a count under either option | a person can type any X into the CLI or the dev GUI, and X = 10^12 asks 601.2f for 10^12 symbols today |
   | Upkeep | a second reader of the existence walk, held to it by the audit. It holds while no cost change depends on X, which `representable` enforces. Exact wherever the gate is, since it asks the gate's question for a number instead of a yes: a board too complex for it is one the gate already misjudges, and MA-7's dynamic program fixes both | `cost-architecture.md` §3.6's "enumeration agrees with enforcement" gains an exception for X; the why says the payment decides; "Actions reversed" counts each X the agent overbids |

**Taken as settled** (named so the review can strike one):
- **Item 214 first**, measured as its own arm: `codebase-state.md` item 214
  slots it there, and the X work is measured against it.
- **Blaze pooled**, §6's row, its effect on how games end reported.
- **The alternative cost's X and 107.3b's alternative-cost half** (pieces 1
  and 5): about 70 lines with tests. They are CR 107.3a and 601.2b, and the
  latent wrong above.
- **The scenario word** (piece 9): `setup-architecture.md` §5.1's promise.
- **One value is not asked**: rule 1.

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
   table, about 100 instructions a cost. §3.3's shapes are exact: one-choice
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
   its mana half built in MA-4 in §2.2's shape; "could produce" with a fixpoint (§3.10).
6. **X.** As recommended, with three refinements (§3.7). The gate tests
   the smallest legal X, which is 1 on 17 cards. The bound bisects over the
   previewed total, because a reduction can absorb part of X (the search
   is reopened at #259's review, §6.1's decision 2). And the legal
   X is a set that other rules cut, so `ChooseXValue` gains a set form for
   Tamiyo's −X and Detonate.
7. **The route.** MA-1 before SU-8; the rest in B before C, MA-3 before
   B13, the Ironworks loop (§6).

---

## 8. Out of scope

- **CR 732.1's reversal** (item 72). It stays with critical-path item 6, as
  `backlog.md` §2.22 row 9 placed it. The solver's "reverse all when the
  solver made the taps" is row 9's, not MA-6's.
- **The payment of hybrid, Phyrexian and `{C/W}`** is CP-1's. Of `{S}` and
  restricted mana, item 33's.
- **CR 609.4b's "spend as though it were mana of any color"** (49 cards) is
  `backlog.md` §2.24's, with CP-1. At the gate it widens every entry's types,
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
  Shape C's window at priority pays for nothing, so MA-6 makes that field an
  `Option` or adds a kind, and decides which with `ChoiceKind::subject()`'s
  match (A4j).

Found by MA-2's plan (2026-10-17, §6.1):
- **The census files an activated ability's `{X}` as unowned.**
  `cast-census.py`'s cost row "mana with {X}" says "an ability's X is never
  asked", owner none, while `cost-architecture.md`'s CP-1 row claims it
  (`activate_ability` never asks, `x_value: None`; 145 cards). The owner
  column is corrected at MA-2's close.
- **Two plans for Void Winnower's X.** `cant-effects-architecture.md` §4.3
  reads CR 601.3a as "begin the cast, then reverse an announced X the
  prohibition forbids" (Endless One with X = 5, not 6, from the card's
  ruling). With the set form, the same check can cut the even mana values
  out of what is offered instead. Both are the CR's: 601.3a lets the cast
  begin, and 601.2e reverses an announcement that makes it illegal. Offering
  only what can be completed is `cost-architecture.md` §3.6's preference, so
  §3.7's owners list the cut with RS-2, and the choice is RS-2's.
- **Mana value with X on the stack** (CR 202.3e) stays `ManaSymbol::X => 0`
  everywhere. CP-1's row owns it; no registered card reads a spell's mana
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
  decision 2 goes; a line for §2.28 at MA-2's close.
- **`GameNumber`'s plan has no division** (found at #259's review). The
  most X for a cost with two or more X symbols is a floor quotient of the
  mana the board makes (§6.1), and `decision-provider-refactor.md` §9's
  planned `Finite`, `Shortcut` and `Relative` hold products and offsets
  only, so a player cannot announce "half my pool". It needs a
  floor-quotient node compared through `k·c ≤ E`; a second line for
  §2.28 at MA-2's close.
- **A cost change can apply by the spell's mana value** (found at #259's
  review, from Golden-Tail Trainer). On the stack the mana value counts X
  (CR 202.3e), so Krosan Drover and five other printed cards make the
  total piecewise in X, and §3.7's "a larger X never costs less" fails.
  No filter can read a mana value yet; §6.1 says what the first one must
  bring and how `filter_reads` keeps it from passing silently.

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
