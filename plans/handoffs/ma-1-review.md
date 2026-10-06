# MA-1 review — findings, triaged, and the order that closes them

The owner's review of [PR #225](https://github.com/lcmaier/MTG-Ichor/pull/225),
2026-10-06, captured before anything was fixed (`engineering-practices.md` §4).
Thirteen inline comments and four requests, numbered in the order they were
asked and indexed at the bottom. **Close one theme per session, starting cold
from this file**, and delete it in the PR that lands the last one.

Theme A lands on #225 before the merge, because a rename after it is a second
sweep over the same lines, and theme F is written against A's names. The rest
wait on the owner's calls below, on #225 or as follow-ups off `main`.

---

## The order, and why

| | Theme | Closes | Why in this position | Gate |
|---|---|---|---|---|
| 1 | **A — names** | #1 #2 #7 #15, and #6's and #9's names | Asked for, mechanical; everything after it is written against the final names | build with zero warnings, `clippy -D warnings`, `cargo test`, the ten checks; no game can move, so no sitting |
| 2 | **F — the walkthrough** | #8 #16 | The owner asked for it to review the module, and it is what theme C's decision is made from | `engineering-practices.md` §7's page, pinned to A's commit; every number from an instrumented run |
| 3 | **B — the inventory a priority point shares** | #3 #4 #5 | API shape only; the owner's call on the shape, below | as A, and `fuzz_ab.py` `IDENTICAL` (the laziness must not move) |
| 4 | **D — the caps** | #10 #11 #12 | The fold's worst case is a defect (below); the search's cap is a choice | the property test, and a fixture with six effects on one tap |
| 5 | **E — the audit's un-count** | #13 | The owner's call on the idiom, below | as A |
| 6 | **C — the cost reader** | #6 #9 #14 | A design change: decided first, sized against the band, then built | a design note reviewed first; then the property test widened, a fixture per cost kind, the arms re-run |

**G (#17), the performance impact, is an answer, not a theme**: given in the
review reply, and recorded in `fuzz-record.md`'s MA-1 block.

---

## A — names · *closed 2026-10-06 on #225, `d535d2e`*

Rename sweeps anchored on each item's own definition and call sites, never on
a bare word (`sizing-and-doing-a-rename-sweep`). The rule (`CLAUDE.md`, name
for the call site): a name used outside `mana_supply.rs` says *mana*, and a
private one says it where "production", "supply", "entry" or "way" would
otherwise read as something else. `production` alone reads as the deployed
build (#7).

| # | Now | Becomes | Why |
|---|---|---|---|
| 1 | `replacement::applies_to_production` | `applies_to_mana_production` | asked |
| 7 | `production_watchers`, `ProductionWatchers`, `scan_watchers`, `audit_watchers`, the memo's `production_watchers` / `insert_production_watchers` | `mana_production_watchers`, `ManaProductionWatchers`, `scan_mana_production_watchers`, `audit_mana_production_watchers`, and the memo's two with `mana_` | asked |
| 2 | `WindowOffer`, `WindowOffer::take` | `ManaAbilityWindowOffer`, `::read` | `ChoiceKind::ManaAbilityWindow` is the question it offers for |
| 2 | `ManaSupply::take` | `ManaSupply::read` | `take` reads as moving a value out (`Option::take`); this reads the board |
| 2 | `Payment` | `NonManaCosts` | it holds the costs beside the mana, and the permanent "this" names in them |
| 2 | `Bag`, `Types`, `ALL_TYPES` | `ManaBag`, `ManaTypes`, `ALL_MANA_TYPES` | |
| 2 | `Entry`, `Way` | `SupplyEntry`, `EntryWay` | `entry` is also the battlefield's, read beside it in two functions |
| 2 | `Taken` (`takes`, `takes_no_fodder`) | `ReservedByCosts` (`reserves`, `reserves_no_fodder`) | what the payment's other costs reserve for themselves |
| 6 | `Doubler`, `doublers`, `made_after_doubling`, `before_doubling`, `doubles_the_pool` | `PoolMultiplier`, `pool_multipliers`, `made_after_multiplying`, `before_multiplying`, `multiplies_the_pool` | the factor was never two: three under Mana Reflection, four under Nyxbloom Ancient |
| 2 | `Outlet` | `SacrificeOutlet` | |
| 9 | `Activation` and its arms `Once`, `PerSacrifice`, `Doubling`, `Unread` | `ManaAbilityCost`: `SpendsItsTapOrItself`, `SacrificesOthers`, `MultipliesThePool`, `NotCounted` | "once" claimed more than it means (#9); theme C replaces the type |
| 2 | `production_of`, `produce`, `made_by`, `rewritten`, `trigger_adds`, `types_made` | `mana_production_of`, `add_mana_produced`, `mana_made_by`, `mana_after_replacements`, `mana_trigger_adds`, `mana_types_made` | |
| 2 | `ProductionRewrite`, `RewriteKind`, `fold`, `apply_in_every_order`, `most_of_every_type`, `undominated_bags`, `MAX_ORDERED_REWRITES` | `ManaRewrite`, `ManaRewriteKind`, `fold_mana_rewrites`, `apply_mana_rewrites_in_every_order`, `most_mana_of_every_type`, `undominated_mana_bags`, `MAX_ORDERED_MANA_REWRITES` | |
| 2 | `Demand`, `Pieces` (`pieces`, `plain`, `pay`), `Search`, `MAX_LEAVES`, `Located`, `OfferedAbility`, `Payable` | `ManaDemand`, `SplitSupply` (`split`, `plain_split`, `pays`), `ChoiceSearch`, `MAX_CHOICE_LEAVES`, `AbilityAt`, `OfferedManaAbility`, `CostRecheck` | |

The live docs that name them (`mana-architecture.md`, `codebase-state.md`,
`engine-map.md`, `backlog.md`) follow; the archive and `fuzz-record.md` are
records and keep the names they were written with.

---

## F — the walkthrough · *closed 2026-10-06 on #225*

**A trace page, `engineering-practices.md` §7's format**: MA-1 changes how a
read is answered, "can this player pay?", and the owner could not follow it
from the diff, which is the section's own trigger (RF's and RG's pages were
added at review the same way). Pinned to theme A's commit. The boards are the
phase's findings: Everywhere, which the greedy count read as five sources;
Mana Reflection on Everywhere, one choice of type; Sol Ring granted a green
tap, ways of different sizes; Wild Growth's trigger; Chainbreaker paying with
its own tap; Ironworks beside Doubling Cube; and the Blood Moon board where
the random agent misses the Cube's route (item 214). **Every number on it is
the engine's**, from an instrumented run of each board. The owner's second
question, how to know the module is understood, gets the page's last section:
boards to predict before revealing the answer, each answer from the same runs,
and what each piece of the module is for, by what breaks without it.

**And the module says its own shape**: the review's walkthrough, condensed,
becomes `mana_supply.rs`'s module doc, its sections in reading order (TR-1's
review precedent, its #13 and #17).

---

## B — the inventory a priority point shares (#3, #4, #5)

**Why the cell exists.** `candidate_priority_actions` asks every card in hand
and every ability whether it can be paid for. An inventory costs about 29,000
instructions, and the cheap refusals come first (timing, targets, other
costs), so most priority points never reach the mana: the design counted
17,906 inventories against 58,921 priority points (`mana-architecture.md`
§3.1, §3.11). The cell is shared by the spells and the abilities, and filled
only by the first check that needs it. Taken inside each function, it would
be two inventories where there is at most one; taken eagerly, about 41,000
more of them.

**What is wrong with it is the shape**: a bare `&OnceCell<ManaSupply>` in
three signatures says *how* it is cached, not *what* is shared, and each
`_with` function has a one-line twin that hands it an empty cell. The two
shapes for the owner:

| | Code | Call sites | Says |
|---|---|---|---|
| **B1. A named lazy inventory** | `let mana = LazyManaSupply::new(game, player_id); castable_spells_with(&mana)` | the three pairs keep their twins | what is shared, in the type |
| **B2. The checks as methods on one view** (recommended) | `let checks = PlayerChecks::new(game, player_id); checks.castable_spells(); checks.can_cast(card)` | 33 call sites, most in tests; every twin goes | one value per question time, its inventory inside it |

B2 is the Rust idiom for "a borrowed view with lazily computed state"; it
removes three function pairs, and a caller asking one question writes
`PlayerChecks::new(game, p).can_cast(card)`. Behavior cannot move; the gate
is `fuzz_ab.py` `IDENTICAL`, which reads the laziness through the cost rows.

---

## D — the caps (#10, #11, #12)

**`fold_mana_rewrites` (#10, #11): a defect at its own cap.** CR 616.1 lets
the player order the replacement effects on a production, and the order
matters: Deep Water and Pale Moon on one tap make blue or colorless as
the player orders them, and "set to" before a multiplier is multiplied, after
it is not. The fold tries every order, with each optional effect declinable,
which is **2^n · n! leaves for n effects: 8 for two, 48 for three, 46,080 for
six**, the cap. On the pool no tap meets more than two, so the cost is small
(`mana_made_by` was about 1% of instructions on the budget board), but at the
cap one tap is about 46,000 bags, for every tap of every inventory.

**The fix is exact and small:** fold by state, not by order. Two orders
that have applied the same effects and made the same mana are the same from
there on, so memoize on (effects applied, mana made): at most 2^n states
times the few distinct bags, 64 at six effects. Multipliers commute with each
other and with retypes, so in practice far fewer. The cap then guards
nothing a board can reach, and goes or rises. ~30 lines; the property test
widened to random effect lists against the order-by-order fold.

**`MAX_CHOICE_LEAVES` (#12): a choice.** It bounds the search over entries
whose mana is one choice of type (Mana Reflection on an Everywhere: two of
one type) or ways of different sizes: each leaf is one Hall check, about 100
instructions, so 4,096 leaves is about 400,000 instructions a check, and past
it the check answers yes. Leaning yes is the owner's rule from item 162
(2026-09-30): an over-offer costs a rewind, an under-offer hides a legal
play. No board in the fuzz is near it (copies of one entry are counted as a
group, `useful_copies`). Two options:
- **D1. Keep it, and count it**: a `Diagnostics` row for a cap that binds, so
  the fuzz record says 0 rather than nobody looking. ~15 lines.
- **D2. Replace the search with a dynamic program** over what the cost's pips
  still need: each choice entry moves the state, and one Hall check of the
  plain mana per final state. Exact with no cap, pseudo-polynomial in the
  cost's pips (a few dozen states for a printed cost). ~80 lines, and the
  property test is its gate.

Recommended: the fold now, and D1; D2 if C makes the choices more common.

---

## E — the audit's un-count (#13)

**Why it exists.** In debug builds the watcher memo is audited: every hit is
checked against a fresh scan, as the frame memo's are (`compute.rs`'s
`audit_memo_hit`). The fresh scan reads characteristics, which counts
`Memo hits` and layer walks, so without an un-count a debug build's rows
would not be a release build's. The frame memo's audit does the same thing
for four counters (`rewind_layer_work`); `rewind_to` does it for all of them.

**Is it fighting the language?** The `Cell`s are not: counting from code that
holds `&GameState` is what interior mutability is for. The field-by-field
copy is boilerplate, and the full destructure is what keeps it exhaustive
(a new counter that is not restored fails to compile). Three ways out:

| | Shape | Cost |
|---|---|---|
| **E1. Audit on a copy** (recommended) | `let fresh = scan(&game.clone())`: the copy's counters move, the game's do not; `rewind_to` goes, and `rewind_layer_work` can follow | a clone per audited hit, debug builds only; needs the trace sink not to follow the clone (to check) |
| **E2. One `Cell<Counts>`** | the counters a `Copy` struct; save and restore are one `get` and one `set` | every increment copies the struct unless the compiler narrows it: measure first |
| **E3. Counters indexed by an enum** | `[Cell<u64>; N]` by `Counter`; save and restore are a map and a zip | every `record_*` call site, about 40, or one-line wrappers |

---

## C — the cost reader (#6, #9, #14)

**What is general and what is not.** The check is general: Hall's condition
and the choice search work on entries, whatever card made them, and every
production goes through the board's replacement effects and triggered mana
by their effects' shapes (Nyxbloom Ancient's tripling is `Multiply(3)`, the
same arm as Mana Reflection's doubling, #6). **The reader in front of it is
not.** `activation_of` matches whole cost lists against four shapes, and two
of them have one printed card each: `PerSacrifice` takes a sacrifice with no
`{T}` (Krark-Clan Ironworks; Ashnod's Altar), and `Doubling` is Doubling
Cube's exact cost and effect. Everything else falls to `Unread` and is not
counted, an under-offer. That is the catch-all `CLAUDE.md`'s exhaustive-match
rule and the "build for the CR" rule exist to stop, and item 213's "with each
shape's first card" is not a slot.

**What the catch-all hides, counted** (Scryfall, 2026-10-06, unique cards):

| Shape | Query | Cards |
|---|---|---:|
| `{T}`, pay life | `o:/\{t\}, pay \d+ life: add/` | 36 |
| remove counters | `o:/remove (a\|an\|one\|two\|x\|\w+) [^:]*counters? from [^:]*: add/` | 61 |
| `{T}` and a sacrifice of another permanent | `o:/\{t\}, sacrifice (a\|an\|another) [^:]*: add/` | 18 |
| `{Q}` | `o:/\{q\}[^:]*: add/` | 1 |
| mana in (Signets, filters) | the census (`cast-census.md` §5) | 170 |

The engine has `Cost` arms for all of them (`PayLife`, `RemoveCounters`,
`UntapSelf`, `Mana`), so a registered card of any of these shapes is played
correctly and offered wrongly. **Every such arm the PR's own type opens, with
a printed customer, is under about 80 lines** (`engineering-practices.md` §4's
rule), so by that rule they were owed in MA-1.

**The shape proposed.** A per-cost reader: each `Cost` arm, matched with no
wildcard, says what one activation spends. Its tap: once a payment, since
only a mana ability can untap it mid-payment, `{Q}`, which is the one place
that "once" is not exact (#9). Itself: once. Counters: count ÷ N uses. Life:
a budget shared by every life-costing entry. Others' sacrifice: a fodder pool
shared by every outlet and the cost's own sacrifices. Mana: a converter's
input, MA-3's rule, or a pool multiplier's. Each kind is handled once in the
check, so Phyrexian Tower (`{T}` and a sacrifice) composes from two kinds,
where today it needs a fifth whole-list shape. `PoolMultiplier` keeps the
demand transform, which is general in its factor; the Cube is its one printed
card, and the arm is named for the mechanism.

**For the owner:** on #225 before the merge (past the 2,500 band, which the
PR body would then report), or as its own PR off `main` before MA-2. Size: about
200–300 lines of code and 200 of tests, with one fixture card per cost kind
the pool can register.

---

## The comments, indexed

| # | Where | Asked | Theme |
|---|---|---|---|
| 1 | `gather.rs:584` | `applies_to_mana_production`? | A |
| 2 | `put_on_stack.rs:568` | mark the new types and functions with `Mana` | A |
| 3 | `legality.rs:163` | why pass an empty `OnceCell` | B |
| 4 | `mana_helpers.rs:109` | is `can_cast`'s wrapper needed | B |
| 5 | `mana_helpers.rs:379` | is `activatable_abilities`' wrapper needed | B |
| 6 | `mana_supply.rs:177` | triplers like Nyxbloom Ancient; `doublers` reads bespoke | A, C |
| 7 | `mana_supply.rs:199` | `mana_production_watchers` | A |
| 8 | `mana_supply.rs:217` | a deeper walkthrough of the file | F |
| 9 | `mana_supply.rs:587` | carve-outs for Ironworks and the Cube; "once" and untapping | A, C |
| 10 | `mana_supply.rs:977` | why `MAX_ORDERED_REWRITES` | D |
| 11 | `mana_supply.rs:981` | combinatorial explosion | D |
| 12 | `mana_supply.rs:1132` | `MAX_LEAVES` | D |
| 13 | `diagnostics.rs:285` | is the field-by-field rewind fighting Rust | E |
| 14 | the review | did MA-1 overindex on the Cube and Ironworks | C |
| 15 | the review | names too vague for mana production | A |
| 16 | the review | an explainer, and how to know it is understood | F |
| 17 | the review | the performance impact | G |
