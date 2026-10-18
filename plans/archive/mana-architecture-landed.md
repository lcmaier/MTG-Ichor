# mana-architecture.md — landed phases, evicted

**A record of finished work, not a plan.** Every section here sat under a ✅
heading in `plans/mana-architecture.md` and was moved out when its phase
shipped, leaving the heading, a stub and a pointer in the live doc. Nothing
here is owed and nothing here should be acted on; what each section is *for*
is the reasoning — the build as sized, what the building changed, the
measurement. `check_state_of_play.py` reads the ✅ headings that stay, and
fails when a landed section keeps more than 40 lines in the live doc
(`engineering-practices.md` §4). Later phases are appended by the PR that
lands them.

#### MA-1 — the inventory and the exact check — ✅ landed 2026-10-05

*Evicted 2026-10-05 from `plans/mana-architecture.md` §5 and §6's table, where §10 keeps the heading and a stub.*

### The build as sized (2026-10-05, at #224)

**§6's row.** §5's five commits: the rewind counter; `oracle::mana_supply`
(the inventory sorting only the player's permanents, the check, §3.3's
shapes for the registered cards, alternatives of different sizes and
Doubling Cube exact, §3.4's memo with its debug audit, §3.6's seam); the
gate in `can_cast` and `can_afford_ability_costs`, one inventory a priority
point; the window (§3.8); `ManaAbilityWindow`'s typed reason. A property
test: every "yes" on a random small board is a payment an exhaustive search
finds, and every "no" is not. Pools Krark-Clan Ironworks and Doubling Cube,
one card for each path it builds that no pooled card reaches, in its last
commit, after the arms read. **~600 engine, ~700 tests, ~100–250 fixtures:
1,400–1,550.** Consumer: the priority question on the budget's board, and
SU-7's review board (Grizzly Bears with one Everywhere: now short). Closes
item 162's oracle half, its offer customer.

**§5, measuring MA-1: a change that moves the agent's stream.** MA-1
changes what the priority question offers, so `new` against `main` reads a
different game, and §3.1's budget, at identical counters, cannot be read
off it (`cost-architecture.md` §6's CM-4 note, item 138). The PR's commits
are ordered so that each arm answers one question:

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

### Sized against built

Lines added, a `src` file's `#[cfg(test)]` module counted as tests, read off
the PR's last code commit (`2e16467`) against #224's merge (`a1815e3`).

| Part | Where | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---|---:|---:|---:|---:|
| The rewind counter, and the diagnostics' rewind the watchers' debug audit needs | `diagnostics.rs`, `priority.rs`, `fuzz_games.rs`, `fuzz_ab.py` | ~600, together | 68 | | |
| The inventory, the check, §3.3's shapes, §3.4's watchers with their memo and audit, the window's offer, the agent's source list | `oracle/mana_supply.rs`, `replacement/gather.rs` and `mod.rs`, `layer_memo.rs` | (above) | 1,328 | ~700, together | 645, the module's own with the property test |
| The gate, one inventory a priority point; `find_mana_sources` and its helpers out (−325) | `mana_helpers.rs`, `legality.rs`, `costs.rs` | (above) | 101 | | 1 |
| The window, its why, the agent's view | `put_on_stack.rs`, `ui/why.rs`, `ui/random.rs` | (above) | 70 | | 2 |
| The pool | `registry.rs`, `phase_re9_cards.rs` | (above) | 16 | | |
| The under-offer boards, the window's refusal, a decorator | `phase_ma1`, `phase_su7`, `phase_su2`, `trace_sink_test.rs` | | | (above) | 240 |
| Fixtures: `castable_spells` returns ids | eight test files | | | 100–250 | 15 |
| **MA-1, the whole diff** | | **~600** | **1,583** | **~800–950** | **903** |

Code ran at 2.6 times its sizing, 2.0 net of the 400 lines it took out, and
tests at 1.0. Of `mana_supply.rs`'s 1,265 lines of code, 214 are comments.
Three things the sizing left out:
- **What a tap makes, read off the board**, about 440 lines: the watchers'
  scan with its memo and audit, the fold of the replacement effects over
  every order, triggered mana, and the retypes §3.4 had left for a later
  card. The sizing priced the check; the production it checks against was
  most of the module.
- **The window's offer and the agent's view**, about 140. §3.8 had the
  agent's scan as "not this design's"; it had to read the production too
  (item 4 below).
- **The cost's own consumption**, about 70: a `{T}` or a sacrifice in the
  cost being paid takes its permanent's mana out of the supply, and the
  outlets' fodder with it.

The band stayed under 2,500 at every commit: 2,486 in all.

### What the build changed in the design

1. **Retypes are read** (§3.4, amended). The design held that ignoring one
   can only over-offer; a Forest under Deep Water makes `{U}` and not `{G}`,
   so it errs both ways, and Deep Water and Pale Moon are registered. A
   resolution's replacement effects are read at each inventory, not memoized
   by epoch: making one moves no layer input.
2. **The Cube is a transform of the demand** (§3.3, amended), not an
   enumeration of the entries that pay its `{3}`: a pip needs `ceil(n / f)`
   of its type left after the input, the cost `ceil(total / f)` of any. The
   design's example board paid `{W}{U}` without the Cube, so the amended
   section has one that does not.
3. **What Ironworks sacrifices after the Cube's doubling pays undoubled**
   (`c07ef44`, found writing item 213's list against the registered cards):
   the Cube itself, then the last Ironworks, go after the doubling or not at
   all. The first build doubled them: with eight Plains, Ironworks and the
   Cube it offered `{15}` where the board makes fourteen.
4. **The window offers what the check counts** (§3.8, amended), so a payment
   the check found is one the window can make: Doubling Cube, which no window
   offered before. The random agent's view reads each ability's production
   on the board as the check does; without it, it declined windows a payment
   covered, a Wild Growth land's `{G}` among them.
5. **The window re-asks only costs** (§3.8, as built): it takes its offer
   once, one entry per ability definition, re-asks each ability's costs at
   each prompt (a `{T}`-only one by its tapped flag), and reads the offer
   again when the epoch moves.
6. **Copies of a way are capped, and only when every way is the same
   size** (`c07ef44`). Twenty Everywheres under Mana Reflection against
   three colored kinds were 1,771 leaves a check; past what one type's pips
   need, a copy pays only generic. A first draft capped ways of different
   sizes too, and a fixed test holds that case.
7. **Decisions 4 and 5 as taken.** `can_cast` returns `Ok(())`, and
   `ManaSource` lives in `mana_supply` as the agent's per-ability list. An
   ability's mana is checked last, after its other costs in printed order
   and its targets; SU-2's Mind Stone line now reads "the engine does not
   offer this", since an over-offer was the only natural board for its
   "reversed" refusal.
8. **The fixtures were three tests, not 100–250 lines**: SU-7's flip (on
   purpose), SU-2's line (item 7), and a trace test that waited for a random
   over-offer to reach a rejection, whose seat now declines its first
   window, the one rejection an exact offer cannot see coming. The 15 lines
   in eight files are `castable_spells`' return type.
9. **The cost arm read twice.** Before any arm, `d495df4` kept the
   supply's split once an inventory: the check had rebuilt it at each of
   about 40,000 checks, nearly a point the design's price did not have. On
   `d495df4` the arm read +0.72%, against −3.0 predicted: the inventory cost about 29K instructions a take, where the
   prototype priced 18.4K, and the patch's greedy walked the per-ability
   list, not the design's counts. `eabdc9e` takes the inventory without
   allocating for what is not there, the patch moved to counts per type,
   and the arm read −1.96% (`fuzz-record.md`, MA-1's block).

#### MA-2 — the legal values of X — ✅ landed 2026-10-18

*Evicted 2026-10-18 from `plans/mana-architecture.md` §6.1, where §10 keeps the heading and a stub. Its live design, the bound as built, moved to §3.7 first.*

### The build as sized (2026-10-17, at #259)

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
| 3 | "X can't be 0" | the card's text | `CardData::x_cant_be_zero`, read as printed: the gate reads the card in hand and the cast its snapshot from before the CR 601.2a move, as both read the mana cost |
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
    the parts (`XBound` carries G, a − r and k), so that node can be built
    from them (§9).
- **Empty.** If the least is not payable, the cast is reversed at 601.2b,
  with the reason 601.2h would give, before a window opens for a payment
  that cannot happen. Only an announcement can take what the gate counted: a
  kicker on an X spell, or 107.3b's 0 under "X can't be 0", which leaves no
  legal value at all (CR 601.2e).
- **One value** is not asked: `backlog.md` §2.22's rule 1, A4e's. Blaze with
  only `{R}` casts with X = 0 and no prompt.

**The shape (pieces 6 and 8).** As built, after #259's code review cut the
sketch's two reason enums: a least of 1 is "X can't be 0", and no bound is
the cost being paid with no X.
```rust
pub struct LegalXValues { pub offered: XOffer, pub x_cant_be_zero: bool, pub bound: Option<XBound> }
pub enum XOffer { Interval { at_least: u64, at_most: u64 }, Set(Vec<u64>) }
pub struct XBound { pub pips: ManaCost, pub most_generic_payable: u64, pub generic_besides_x: i64, pub x_symbol_count: u64 } // 601.2h: P, G, a − r, k
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
so no game reaches `XOffer::Set` until a cut lands. Under decision 1's
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

### Sized against built

| | Sized (§6.1, after decision 2) | Built |
|---|---:|---:|
| Code added | ~600–640 | 994 (272 removed, net 722) |
| Tests added | ~720 | 943 (131 removed, net 812) |
| Total added | 1,320–1,360, about 1,650 at MA-1's 1.5 on code | 1,937 |

Code ran 1.6 times its sizing, tests 1.3. The overage is the restructure of
601.2f's arithmetic onto `ManaComponent` (a move, counted once added and once
removed), the scenario word's driver and refusals, and the property test's
board generator, pulled out to share.

### What the build changed in the design

- **The bound is read in closed form**, decision 2, taken at the owner's
  review on a condition: past half a point, the engine falls back to no
  bound. The plan's first draft searched by doubling.
- **`ChooseXValue`'s values are boxed.** Inline, the 80-byte `LegalXValues`
  would have doubled `ChoiceKind`'s 48 bytes and every prompt's context.
- **An unasked `x N` is checked** at the line's confirmation, against the
  stack entry, so a line naming the wrong X for a one-value spell stops.
- **The generic split's buckets and caps** became one function each
  (`GameState::generic_split_buckets`, `ui::ask::generic_split_caps`), read by
  the ask and the why alike.
- **The census** lacked rows for two question kinds added by earlier trigger
  phases (`ChooseDelayedTriggerEvent`, `ChooseEnteringDefender`) and could not
  regenerate; their rows were added with MA-2's, and the registered-cards
  cache was refreshed.
- **A 200-turn game in the four-seat `stress` smoke** (seed 810 of 777's
  1,000) is a three-player stall after a departure; its reversals are Rancor
  and Yixlid Jailer, the agent's, and Blaze was never cast in it.
