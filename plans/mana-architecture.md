# Mana — what a player can pay, and what mana abilities make

> **Status:** design, 2026-10-05, for `codebase-state.md` item 162. No code
> written; reviewed before any build. Every number is from a measurement
> made 2026-10-05 on `f4df90a` (#223's merge), and §2 says how.
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
decision**, and one point of §3.1's budget is 6,255.

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
timestamp (3,600 instructions a sort).

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

The theorem needs each mana's type chosen on its own. Five shapes break
that, and each gets an exact answer below a cap. Above the cap the check
answers yes: **an over-offer costs a rewind, an under-offer hides a legal
play** (item 162, 2026-09-30), so the gate leans one way only.

- **One choice for several mana:** "two mana of any one color" (54
  abilities), or a multiplier over an entry of several types. Mana Reflection
  on Everywhere gives two of one type, not one W and one U. Each such entry
  is enumerated over the colored kinds the cost asks for, and its mana counts
  toward generic otherwise. Identical entries are enumerated as a multiset,
  capped at 4,096 leaves. **About 2,800 instructions a cost where present**:
  274 of the board's inventories.
- **Alternatives of different sizes**, such as a creature that taps for
  `{G}{G}` or convokes for one. The entry takes the largest amount over the
  union of types. That can over-offer; the probe met 4.
- **Mana-fed sources** (175: 128 that tap, once a turn; 47 repeatable). A
  single-use converter (a Signet, `{1}, {T}: Add {W}{U}`) adds its input to
  the demand and its output to the supply. Subsets of converters are
  enumerated (2^k, k ≤ 6), and a subset counts only if mana not from a
  converter can start the chain. Doubling Cube's output depends on the pool
  it resolves with, so its entry is taken as doubling the supply after its
  `{3}`, which may over-offer. A repeatable filter is extra supply of its
  output types, capped by what can pay it. **About 3,600 a cost where
  present.**
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
  `cost-architecture.md` §3.11's Mind Stone puzzle out of the offer: an ability whose cost sacrifices its
  source cannot also feed that source to Ironworks.

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
- A retype narrows an entry's types. Ignoring one can over-offer
  but never under-offer, so the first build may read the multiplier and the
  trigger and leave the narrowing for its first card.

**They are read once per layer epoch, for every player at once.** Each input
is a layer-walk input: effective abilities, attachments and controllers. So
the epoch argument the frame memo stands on (`layers-architecture.md` §12,
"7a") covers this cache too, and it is exact. On the board the scan costs
8,900 instructions and runs 5,099 times, against 17,906 times live: **3,600
a decision, against about 10,900 unmemoized**. A cache needs an audit, as the
frame memo's has: in debug builds, a live scan beside every hit, compared.

### 3.5 Order and determinism

The answer is a boolean from an exact algorithm, so the order the inventory
reads the battlefield cannot change it. The probe's sorted and unsorted
inventories agree to the mana on all 16,475 states it compared. **The
inventory still reads in timestamp order**, because the window reads the
same inventory and its option list is order-observable (`CLAUDE.md`'s
determinism rule). A second, unsorted path for the check alone would save
about 0.6 points; it is §9's lever, not this design's.

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
  argument. No card reaches them until T12c.
- **What item 33 builds:** the per-unit record, the payment that reads it,
  and the source axis on the generic split.

### 3.7 X (decision 6)

**A spell with X is offered when X = 0 is payable.** The X pips have no
demand at the gate. **The bound `ChooseXValue` offers is computed once, at
601.2b:** the largest X whose total the inventory covers. It is found by
bisection over [0, the inventory's mana], with the total previewed through
`cost_determination` at each X, because a reduction can absorb part of X: at
X = 0 a `{1}` reduction on `{X}{R}` is wasted, and at X = 1 it is not. The
prompt becomes `(0, bound)`. The random agent's land count goes, because the
engine bounds it. The why names the bound (§3.13). X in an activated
ability's cost is CP-1's, and X in an additional cost is CP-2's amount
shape. Both read this bound when they land.

### 3.8 The window reads the inventory

The window takes the inventory once. After each activation it drops the
activated permanent's entries, and it rescans only when the layer epoch moved
(a sacrifice) or a life payment changed what can be paid. **The option order
stays timestamp, then ability**, so the agent's stream does not move.
- **Saves:** the warm re-enumerations, about 4,500 instructions a decision
  (0.7 points). The cold first scan stays: the cast's move to the stack
  invalidated those frames for every reader.
- **Not this design's:** the random agent's preference (`mana_window_preference`,
  3,800 a decision) scans again for itself. Handing it the inventory through
  the prompt is a harness change.

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
  Sphere's draw) resolve through the general resolver inside the same 605.3b
  resolution, each its own proposal through the chokepoint. **About 22,500
  instructions an activation**, the measured cost of a production's own
  proposal (`resolve_mana_effect`, 4,417 calls).
- **A mana ability's own mana cost opens a window** (605.3a: "activating an
  ability that requires a mana payment"). TR-7 needs it for Chromatic
  Sphere's `{1}` (`triggers-architecture.md` §12), and Signets need it too.
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
  `SeatMode` is how a seat's standing policy reaches the engine, and building
  an offer the seat would always decline is cost without a decision
  (`backlog.md` §2.22's rule 2: a bot's seat takes its policy at
  construction). Built in MA-6, beside the
  person's-seat solver.

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
- **`ChooseXValue`:** the bound (§3.7). "X above N leaves the cost
  unpayable: N plus the cost's other pips is all the mana this board makes."
- **`GenericManaAllocation`:** the clamp. A bucket's maximum is the pool's
  mana of that type less the pips that need it.

The first lands with MA-1, the second and third with MA-2.

---

## 4. The families: algorithm, price, build

**Price** is callgrind instructions per decision on the budget's board,
12,560 decisions. Where a family is on the board it is measured. Where it is
not, its prototype ran on every checked cost of the board's real states
(33,550 of them), and the number is an upper bound: what the family would
cost if every cost the check read carried it.

| Family (`cast-census.md` §12) | Cards | Algorithm | Price | Build |
|---|---:|---|---|---|
| The plain check: several mana at once, listed types sharing a `{T}` | 155, 380 | §3.1–3.2 | **30.3K a decision with §3.4 (26.3K the inventory, 3.6K the scan, 0.4K the checks), against 45.6K today: −2.4 points** | MA-1 |
| Triggered mana and its replacements | 19, 10 | §3.4, memoized | 3.6K (in the line above); 10.9K unmemoized | MA-1 |
| One choice for several mana: "any one color", a multiplier on a multi-type entry | 54 | §3.3 enumeration | 2,800 a cost where present; bound 9.5K | MA-1 (the multiplier, on the board); MA-3 (the printed form) |
| Mana-fed: converters and filters | 175 | §3.3 subsets | 3,600 a cost where present; bound 9.6K | MA-1 (Doubling Cube, registered); MA-3 (Signets) |
| Sacrifice-fed, another permanent | 26 | §3.3 count less the cost's own | one candidate count an inventory where present | MA-1 (Ironworks, registered) |
| Sacrifice-fed, itself | 117 | an ordinary entry | 0 beyond the plain check | MA-1 |
| Variable amount; the doubled pool | 60; 1 | evaluated at the inventory (605.3b) | one `evaluate_amount` a source | MA-1 |
| One mana of any color; Treasure makers | 306 + 21, 347 | a five-type entry; the choice at resolution (§3.10) | 0 at the gate; one prompt an activation | MA-3 |
| Painland riders | 129 | gate unchanged; the rider through the chokepoint | 0 at the gate; about 22.5K an activation | MA-3 |
| The chosen color | 32 | the recorded bit; `backlog.md` §2.2's record, its mana half | 0 beyond a read | MA-4 |
| "Could produce" | 14 | the union, a fixpoint, memoized | one pass over the named set a memo miss | MA-4 |
| Restricted mana | 153 | §3.6, an edge refused | 2,100 a cost where present; bound 5.6K | the seam in MA-1, the cards in item 33's T12c |
| X | 527 | X = 0 at the gate; the bound by bisection at 601.2b | 159 a cost; bound 0.4K | MA-2 |
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

MA-1 changes what the priority question offers, so `new` against `main`
reads a different game, and §3.1's budget, at identical counters, cannot be
read off it (`cost-architecture.md` §6's CM-4 note, item 138). The PR's
commits are ordered so that each arm answers one question:

1. **The rewind counter, first.** A `Diagnostics` cell counted where
   `run_priority_round` takes an `Err` (`engine/priority.rs`), the row
   "Actions reversed" (CR 732.1's word) in `fuzz_games`, and its `ROWS` entry
   in `plans/fuzz_ab.py`. Arm *counter*: **IDENTICAL** on every gameplay row,
   and the new row reads 22.4 a game on the budget's board.
2. **The inventory and the check, unused by the gate.** Arm *inert*:
   **IDENTICAL**.
3. **The window reads the inventory** (§3.8). Arm *window*: **IDENTICAL**,
   about −0.7 points.
4. **The cost arm**, a throwaway patch on commit 3 in a `C:/w/arms`
   worktree, never a commit. The gate takes the inventory once per candidate
   list and answers with today's greedy, run over the inventory's per-ability
   list so its answers are today's. It also runs the exact check,
   black-boxed. Counters **IDENTICAL**. Instructions are the budget's
   reading, **predicted −2.3 points**: the new check's 30.3K plus the old
   greedy's few hundred a cost, against 45.6K. A cost arm that ran the old
   scans as well would read +5% and fail the budget for a reason that
   ships nowhere.
5. **The behavior commit**: the gate answers with the exact check, and
   `find_mana_sources` goes. Arm *shipped*: **differ**, predicted:
   "Actions reversed" falls from 22.4 to 0 on the budget's board, and about
   30 fewer decisions a game before the stream moves (317 prompts forced, 448
   re-asks gone and 134 points gained, over 20 games). Every other row moves
   with the games.

**Fixtures.** Scripted answers are positions (item 209). A test whose board
offered an over-offered card ahead of its answer renumbers. SU-7's test
`phase_su7_integration_test.rs:93` flips on purpose: "item 162's loose
offer; when it is fixed, this board says the mana is short". Count with
`cargo test --no-fail-fast` on commit 5, as A6j did. **The ratchet** re-bases
on commit 5 (item 138: a change that moves the stream re-bases it), and the
`fuzz-record.md` block says so.

---

## 6. The build, sized

Each PR carries a consumer and lands in or under the band
(`engineering-practices.md` §4). Sizes are code plus tests, and the
card-by-card hunt runs in each PR's brief.

| PR | Shape | Size | Consumer | Closes |
|---|---|---|---|---|
| **MA-1, the inventory and the exact check** | §5's five commits: the rewind counter; `oracle::mana_supply` (the inventory, the check, §3.3's shapes for the registered cards, §3.4's memo with its debug audit, §3.6's seam); the gate in `can_cast` and `can_afford_ability_costs`, one inventory a priority point; the window (§3.8); `ManaAbilityWindow`'s typed reason. A property test: every "yes" on a random small board is a payment an exhaustive search finds, and every "no" is not. Pools Krark-Clan Ironworks and Doubling Cube, one card for each path it builds that no pooled card reaches, in its last commit, after the arms read | ~600 engine, ~700 tests, ~100–250 fixtures: 1,400–1,550 | the priority question on the budget's board; SU-7's review board (Grizzly Bears with one Everywhere: now short) | item 162's oracle half, its offer customer |
| **MA-2, X** | §3.7: X = 0 at the gate, the bound by bisection, `(0, bound)`, the random agent's self-limit out; the why of `ChooseXValue` and `GenericManaAllocation`. Registers and pools an X spell (Blaze is the plain one) | ~250 engine, ~350 tests, a card: 650–800 | Blaze | `cast-census.md` §8's X row |
| **MA-3, any color, riders, the nested window** | §3.10's spec type and its sweep of `ManaOutput` sites; the choice at resolution, a `ChoiceKind`; riders through the resolver; a mana ability's mana cost opening a window. Cards: Birds of Paradise, a painland (Llanowar Wastes), Gilded Lotus ("three of any one color"), a Signet, a Treasure maker with `backlog.md` §2.27's Treasure | ~700 engine, ~800 tests: 1,500–2,000 | five cards; TR-7's three facilities | `backlog.md` §2.19 |
| **MA-4, the chosen color and "could produce"** | `backlog.md` §2.2's record, its mana half (the choice at entry, `EnterMods`, the permanent's field); "could produce" with its fixpoint. Cards: Thriving Grove, Coldsteel Heart, Exotic Orchard, Reflecting Pool | ~550 engine, ~650 tests: 1,200–1,500 | four cards | `backlog.md` §2.2's mana half |
| **MA-5, paying with permanents or cards** | §3.9: the entries; the payment prompt in `plan_payment`; the taps and exiles through the chokepoint. Cards: Stoke the Flames, Treasure Cruise, Reverse Engineer, a waterbend card. Assist and offering go here if the hunt keeps it in band, else to C with a card each | ~900 engine, ~1,000 tests: 1,900–2,400 | four cards | `cast-census.md` §7's payment row |
| **MA-6, a person's seat: the solver and item 211** | §3.12's solver and decorator, wired in `cli_play` and the dev GUI behind the toggle, with a `fuzz_games` flag off by default; §3.11's shape C (a `SeatMode` field, the "add mana" entry, CR 605.3a's window at priority). A GUI PR: `engineering-practices.md` §10's review | ~600 engine and client, ~600 tests: 1,100–1,400 | the CLI and dev GUI person's seat | `backlog.md` §2.18's solver (§2.22 rows 6 and 7), item 211 |

**Ordering.**
- **MA-1 before SU-8** is recommended (decision 7). It removes the wrong
  offer SU-7's click script met on the review board, and it hands SU-8 the
  inventory its `ManaAbilityWindow` line reads.
- **MA-2 to MA-6** go in `roadmap-v2.md` B, before C. **MA-3 before TR-7**,
  whose loop needs any-color mana, a draw inside a mana ability, and a window
  inside a mana ability's activation.
- **CP-1 after MA-1**: its gate half is §3.2's arms.
- **Item 33's design pass after MA-1**: its seam is §3.6.

---

## 7. The decisions

The brief named six. A seventh, the route, is the owner's call.

1. **Where the design lives.** This document (§0). `CLAUDE.md`'s row gains
   `mana` (106/605/601.2g–h's existence half, `MA-*`).
2. **Item 33's seam.** The check refuses an edge: an entry's restriction
   meets the cost's purpose, and `{S}` is a bit. The per-unit record, its
   payment and the source axis are item 33's own pass (§3.6).
3. **The algorithm.** As recommended: one entry per shared cost, amounts as
   capacities. Feasibility is read by Hall's condition over a 64-entry
   table, about 100 instructions a cost. §3.3's five shapes are enumerated,
   with a cap above which the check over-offers. On the budget's board the
   exact check is about 2.4 points cheaper than today's greedy, measured,
   because the inventory is taken once per priority point, not once per
   card.
4. **Item 211.** Shape C (§3.11): CR-complete for any seat that asks, and
   nothing asked of a seat that does not.
5. **The production shapes.** A spec with four forms; the choice at
   resolution; riders through the resolver; `backlog.md` §2.2's record,
   its mana half built in MA-4 in §2.2's shape; "could produce" with a fixpoint (§3.10).
6. **X.** As recommended, with one refinement: the bound bisects over the
   previewed total, because a reduction can absorb part of X (§3.7).
7. **The route.** MA-1 before SU-8; the rest in B before C, MA-3 before
   TR-7 (§6).

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
  lever 6. A check-only unsorted path would save about 0.6 points of it
  (§3.5); a maintained order would save most of the 9% everywhere.
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
