# Engineering practices — how this codebase and its docs are written

**Authority:** this doc owns the *process* rules. The engine invariants live in
`layers-architecture.md`, `replacement-architecture.md` and
`cant-effects-architecture.md`; current state lives in `codebase-state.md`.
`CLAUDE.md` states each rule in a few lines and points here or there for the
reasoning — that split is itself one of the rules below.

Created 2026-08-29, closing theme G of `plans/handoffs/rb-review.md`. Three of
its five sections are rules that existed only as habit until PR #62 showed what
happens when a habit is the only enforcement.

---

## 1. `CLAUDE.md` has a line budget, and the budget is checked

**The rule: 200 lines, hard.**

```bash
python plans/check_claude_md.py     # exit 1 if over; prints per-section counts
```

**Why a script.** The previous rule was "keep it durable — no progress
snapshots or counts". It is a good rule and it did not work: the file crossed
300 lines twice, both times by accretion that each individual edit could
justify. A rule that can fail silently is not a mechanism. This one fails the
way a warning fails.

**Why 200 and not more.** `CLAUDE.md` loads into every session before the task
does. Every line competes with the work, and a line that is *nearly* current is
worse than a missing one, because it will be believed. 200 lines is roughly what
the file held when it was last unambiguously worth reading end to end.

**Three sub-rules, which are what make the cap reachable:**

- **Every invariant is at most three lines plus a pointer.** The *statement* of
  the rule lives in `CLAUDE.md`; the reasoning, the war story and the rule
  numbers live in the architecture doc. RB left the replacement-pipeline section
  at 48 lines re-arguing six decisions that `replacement-architecture.md` §4.1
  already argues at length.
- **Adding a section requires removing one.** The file describes the *current*
  shape of the project. A project does not accumulate invariants forever; it
  replaces them, and the discipline of naming what a new one displaces is how
  you find out whether it is really new.
- **A war story is not an invariant.** "This was the shape indestructible had
  before Phase RB" is history. It belongs in the commit message that changed it,
  or in the architecture doc that owns the decision.

**Raising the cap is allowed and has to be typed.** `--budget N` exists so that
a raise is a visible act with a reason attached, not a drift.

---

## 2. The comment rule

RB's comment density is far above the surrounding code — module essays, six-line
paragraphs re-narrating the four lines under them, and design archaeology in
source files. The rule that was missing:

- **Comment the *why*, and only where the why is not recoverable from the code
  plus one rule number.** `// CR 701.26a — only untapped permanents can be
  tapped` earns its place. Six lines restating what the next four lines do does
  not.
- **One rule cite beats a paragraph.** If a comment needs more than about four
  lines, that is the signal its reasoning belongs in `plans/` with a one-line
  pointer from the source.
- **A war story goes in the commit message or the architecture doc.** Source
  comments are read by someone changing the code now; the story of what the code
  used to be is read once, by someone doing archaeology, and `git log` and
  `plans/` are where they will look.
- **Doc comments on types and public functions are exempt from the brevity
  half** — they are the API's documentation and `cargo doc` renders them. The
  *why*-not-*what* half still applies.

This is a rule for new and edited code. It is not a licence for a sweep that
deletes comments nobody reread; themes C and E of the RB review are where the
existing density comes down, file by file, as those files are touched anyway.

---

## 3. Two card pools

`cards/registry.rs` builds two:

| Pool | Contents | For | Flag |
|---|---|---|---|
| **performance** | a *representative* board (`PERFORMANCE_POOL`) — grows deliberately, one card per new engine path | A/B-ing an engine change, **interleaved in one sitting** | `--pool performance` (default) |
| **stress** | every registered card (`default_registry`) | panics, errors, and effect interactions | `--pool stress` |

**Why two.** Before this split there was one pool, and it had to be both things
at once — so a card that would have exercised the new pipeline was kept *out of
the registry* on the grounds that it "would move the baseline". That is an
argument for separating the pools, not for keeping cards out: an unregistered
card is invisible to `fuzz_games`, to `card_pool_lowering_test`, and to
`cli_play` all at once, which is three losses to protect one number.

**And why the *freeze* went, 2026-09-01 — it is the same argument one level up.**
RS-1 added a gated subsystem that no card in the pool could open, so its A/B
measured only the closed path. Left alone, a frozen pool measures a shrinking
fraction of the engine: eventually "flat on the performance pool" means "flat on
the parts that existed in 2026-08". Keeping a card out of *this* pool to protect
a number costs one measurement where keeping it out of the registry cost three —
weaker, and the same shape. What the freeze was protecting no longer needs it:
an interleaved A/B uses one pool in both arms by construction, so stability
across months buys the timing measurement nothing.

**The rules that follow:**

- **Register the card.** Writing a card and registering it are the same act now.
  Registration is what puts it in front of every check the project has.
- **`PERFORMANCE_POOL` is representative, not frozen** (revised 2026-09-01; it
  was frozen for a year). **A phase that opens a new engine path adds one card
  to it, deliberately, and re-records the table below.** Adding is still not the
  same act as registering. `performance_pool()` panics on a name that is no
  longer registered, which is the guard against a rename shrinking the pool
  silently — the one failure a deliberate addition does not have.
- **Before making a word in a card's text *structural*, find two more cards in
  the same bucket and check they print it.** CV-1 made "each **other** creature"
  structural — a copy arm that excluded the donor with no way to say otherwise
  — on the reading that a class-scoped copy always says "other". Mirrorform
  prints the same shape without the word, and the arm could not express it at
  all. **Structure is the expensive direction to be wrong in:** data that is
  never varied costs a field, structure that has to vary costs an enum arm and
  every match on it. The census cannot catch this — it partitions by
  *mechanism*, and a one-word difference lives inside one bucket.
- **A fixture may be invented. It must not wear a real card's name while
  behaving differently, and it must not be cited in the spec corpus or the
  plans as evidence about printed Magic.** Written down 2026-09-03; the rule
  being applied until then — "no invented cards" — was written nowhere, and
  this is the narrower one that actually earned the scars. Both scars are
  about the *name*, not the invention: `phase5_pre_cards::inside_out` stays
  unregistered because it simplifies a hybrid cost the engine cannot express
  while wearing Inside Out's name, and `codebase-state.md` item 15's "Mind
  Snare" was an invented name that reached the corpus and `roadmap.md` as a
  printed card. A fixture with its own name, or a token with its real text
  (Everywhere, `cards/dual_lands.rs::everywhere`, is the first admitted under this rule
  as written), misleads nobody; a doc that cites either as a fact about Magic
  does.
- **`PERFORMANCE_POOL` measures cost; `--require` measures coverage. Do not
  read either off the other.** Adding a card to the pool does *not* mean its path
  gets walked: CV-1 put Cytoshape in and it resolved **16 times in 200 games**,
  because any one card competes with the whole pool for 36 slots — and that
  ratio worsens with every card added, forever. **The unforced number is not a
  target**; `--require` is the answer to "was the path walked", and the pool
  measures cost. `fuzz_games --require "Card A,Card B"` forces a copy of each
  into every deck, then prints casts, resolutions, the share of games each
  reached, **copies per deck**, and **board diversity** — the share of games in
  which a permanent of a color no required card has entered the battlefield.
  **An empty `--require` changes nothing** — every RNG draw is guarded, so a
  reachability run and a timing run come from one binary without the first
  contaminating the second. **Run it once per phase that adds a card**, and put
  the resolution count in the phase's ledger entry **beside its copies per
  deck**: the flag adds one copy and the 36 draws add more, how many depends on
  the pool they came from, and a count is only comparable per copy.

  **It no longer seeds the deck's colors from the required card's
  (2026-09-03).** It had to, because nothing in the registry made more than two
  colors, and the price was the board: `--require Cytoshape` put every game on
  a G/U pair, so the card met a white, black or red permanent in **none** of
  them. Everywhere — the five-color land token, now the fill tier of every
  deck's mana base (`random_deck`) — is what let the seeding go. Same 200 games
  / seed 12345, `performance`:

  | `--require Cytoshape` | cast | resolved | games | copies/deck | non-G/U permanent seen |
  |---|---:|---:|---:|---:|---:|
  | with color seeding (`main`, 6dedaf8) | 400 | 390 | 177 (88%) | 2.68 | 0 — by construction |
  | without | 216 | 212 | 126 (63%) | 1.72 | **198 (99%)** |
  | without, and the agent taps for the pip it owes | 230 | 228 | 131 (66%) | 1.72 | **200 (100%)** |
  | the row above, plus `,Everywhere` forced | 222 | 219 | 125 (62%) | 1.72 | 200 (100%) |

  **Read the copies column first. This table was first written without it, and
  said "the resolution count halves, and that is the agent".** A deck seeded to
  G/U drew its 36 nonlands from 21 cards and held 1.7 *extra* Cytoshapes; the
  whole pool gives 0.7. A throwaway build that forced *exactly* one copy per
  deck — not shipped, since the extra copies are more stress and that is what
  the flag is for — read **149 / 141 / 133** resolutions for the first three
  rows: the seeding was worth about 5%, the agent's fix none of it, and the
  rest was copies. What the uniform tap *did* cost was real: with an any-color
  land a tap was a five-sided die, and land taps per spell cast went 3.86 →
  7.66. `RandomDecisionProvider` now taps for the pip it still owes, and the
  generic split comes back from the prompt already clamped to what the pips
  leave over (`ui/ask.rs`, 2026-09-03), which brings taps per cast to **3.18**
  — below `main` — and the pool's spells per game from 22.6 back to 25.0. It does not raise a forced card's count, because that count is
  bounded by drawing the copy and choosing it among everything else castable,
  not by paying for it; and forcing Everywhere on top changes nothing either,
  the deck already holding fourteen.
- **Say which pool a number came from.** `fuzz_games` prints it in the header
  and in the results block. The two pools are not comparable to each other, so a
  pasted stats block without its pool name is not evidence of anything.

**Re-recorded 2026-09-03 for RC-5** (CR 614.13's auxiliary zone changes and a
dynamic entry amount; `replacement-architecture.md` §9). Three new cards, two of
them in `PERFORMANCE_POOL` — Thunder-Thrash Elder and Master Biomancer, one per
engine path the phase opens — and Sutured Ghoul registered, which is what moves
`stress` from 72 cards to 75. **Every row here is the pool's and none is the
engine's, and that was measured rather than assumed** — see the A/B below.
`Layer walks` has been the epoch memo's *miss* count since 2026-09-03; walks
plus `Memo hits` is the number of questions asked, which is the figure the
entries before that one printed as a walk count.

| | performance (66 cards) | stress (75 cards) |
|---|---|---|
| P0 / P1 | 27 (54.0%) / 23 (46.0%) | 28 (56.0%) / 22 (44.0%) |
| Avg turns | 34.5 | 30.6 |
| Spells cast | 24.9 | 22.3 |
| Lands played | 20.0 | 18.2 |
| Combat w/ atk | 12.0 | 10.2 |
| Creatures died | 7.9 | 4.4 |
| Damage events | 24.3 | 24.0 |
| Total damage | 54.7 | 56.9 |
| Life changes | 16.3 | 17.0 |
| **Layer walks** | **2,550** | **2,214** |
| **Memo hits** | **102,077** | **87,597** |
| **Layer frames** | **3,555** | **3,116** |
| **Frames/walk** | **1.39** | **1.41** |
| **Replacement gathers** | **567** | **504** |
| **Restriction queries** | **570** | **506** |

**The engine's share is zero, and `performance` is where that is provable
(2026-09-03, RC-5).** Three binaries in one sitting through `plans/fuzz_ab.py`:
`main` (A), RC-5's engine with `PERFORMANCE_POOL` exactly as `main` had it (B),
and RC-5 shipped (C).

| | A: main | B: engine, pool unchanged | C: shipped |
|---|---|---|---|
| performance, 200 games, outside `=== Timing ===` | — | **byte-identical to A** | differs |
| performance walks (50 games) | 2,663 | 2,663 | 2,550 |
| performance frames/walk | 1.43 | 1.43 | 1.39 |
| performance CPU/game median (200 games, ×3) | 12.73 ms | 12.82 ms (+0.7%) | 12.86 ms (+1.0%) |
| performance ms / 1,000 walks | 5.074 | 5.110 (+0.7%) | 5.124 (+1.0%) |
| stress, 200 games | — | **identical to C** | — |

**Read the first and last rows together.** B against A is byte-identical on
`performance` — same games, same counters, same event stream — so the new arm,
the template evaluation and the two exclusion sets cost the pool nothing when
no card reaches them. B against C on `stress` is *also* identical, and for the
opposite reason: `stress` is the whole registry, so a registered card is in it
whether or not it joined the measured pool. Between them the two columns
partition the change exactly — everything that moved on `performance` is the two
cards joining that pool, and everything that moved on `stress` is the three
cards being registered at all. Time is +0.7% for the engine alone and +1.0%
shipped, both inside the ~2–6% spread a sitting shows, and both were left
un-chased for the reason RC-4b left its `stress` delta: a number inside the
spread is not a finding.

**What the cards do to the game is worth naming, because it is large.**
Thunder-Thrash Elder is cast in 149 of 200 `performance` games and sacrifices
its controller's own creatures to enter; `stress` loses four turns a game
(34.4 → 30.6) and gains a death a game (3.5 → 4.4). Games that end sooner do
fewer of everything, which is why every behavioural row on `stress` fell while
`Creatures died` rose. `Frames/walk` moved the other way on the two pools —
1.43 → 1.39 on `performance`, 1.33 → 1.41 on `stress` — and neither is a
per-walk cost change: a Master Biomancer on the board puts a filter-scoped
sub-frame under entries that did not have one, and a shorter game has fewer of
the cheap walks that dilute the average.

*Previous values, 2026-09-03 (performance 64, stress 72 — before RC-5's three
cards; reproduced to the digit by this sitting's `main` arm, which is the check
that the re-record is the cards and not the machine): performance 28/22, 34.6
turns, 26.0 spells, 20.3 lands, 13.1 combats, 8.2 deaths, 26.7 damage events,
58.4 damage, 18.4 life changes, 2,663 walks / 105,963 memo hits / 3,818 frames /
1.43 per walk / 584 gathers / 585 queries; stress 24/26, 34.4, 25.3, 19.7, 11.6,
3.5, 25.6, 54.4, 17.5, 2,719 / 108,699 / 3,612 / 1.33 / 599 / 600. Before that,
2026-09-03 (performance 64, stress 72 — the Everywhere pool,
mana base and agent, before the epoch memo; every row but the three cost rows
is unchanged by it, and `Memo hits` did not exist): performance 108,626 walks
/ 161,827 frames / 1.49 per walk / 584 gathers / 585 queries; stress 111,418 /
152,428 / 1.37 / 599 / 600 — reproduced to the digit on `main` at 90692f7 in
the same sitting. Before that, 2026-09-03 (performance 63 → 64, stress 71 → 72
— the same PR, before the random agent learned to tap for the pip it owes):
performance
30/20, 36.0 turns, 22.2 spells, 20.6 lands, 11.2 combats, 5.2 deaths, 22.3
damage events, 50.8 damage, 16.5 life changes, 108,423 walks / 158,203 frames /
1.46 per walk / 650 gathers / 651 queries; stress 31/19, 36.6, 21.7, 20.7,
11.0, 3.2, 22.2, 48.2, 17.9, 114,839 / 148,874 / 1.30 / 694 / 696. Before
that, 2026-09-02 (pools unchanged at 63 / 71; a cast whose payment fails now
rewinds instead of resolving unpaid, 16c): performance 25/25, 30.6
turns, 26.4 spells, 19.2 lands, 11.8 combats, 6.7 deaths, 24.9 damage events,
50.9 damage, 16.8 life changes, 104,622 walks / 146,052 frames / 1.40 per walk /
588 gathers / 590 queries; stress 25/25, 30.8, 25.1, 18.8, 11.8, 5.6, 25.2,
58.9, 16.9, 100,674 / 137,318 / 1.36 / 578 / 582 — reproduced to the digit on
`main` at 6dedaf8 before the Everywhere re-record. Before that, 2026-09-02
(performance 62 → 63, stress 68 → 71; Cytoshape,
Mirrorweave and Mirrorform, CV-1): performance 23/27, 28.8 turns, 21.0 spells,
18.4 lands, 11.0 combats, 6.7 deaths, 24.2 damage events, 49.8 damage, 16.8 life
changes, 93,914 walks / 136,338 frames / 1.45 per walk / 531 gathers / 533
queries; stress 27/23, 28.3, 19.6, 17.6, 10.5, 4.4, 23.4, 55.3, 15.7, 88,252 /
117,612 / 1.33 / 514 / 517 — reproduced to the digit on `main` at 650a263
before the 16c re-record. Before that, 2026-09-02 (performance 60 → 61, Root
Maze, RC-3): performance
23/27, 31.7 turns, 23.0 spells, 19.8 lands, 12.4 combats, 7.8 deaths, 25.7 damage
events, 52.2 damage, 17.0 life changes, 108,632 walks / 135,449 frames / 1.25
per walk / 669 gathers / 670 queries; stress 30/20, 29.8, 21.8, 18.1, 11.6, 4.5,
25.8, 58.8, 16.5, 98,843 / 118,256 / 1.20 / 618 / 618. Before that, 2026-09-01
(performance 59 → 60, Battlegrowth and Adaptive
Shimmerer): performance 25/25, 30.4 turns, 22.1 spells, 19.1 lands, 11.9
combats, 7.6 deaths, 25.0 damage events, 52.3 damage, 17.0 life changes, 99,877
walks / 123,802 frames / 1.24 per walk / 626 gathers / 627 queries; stress 26/24,
29.6, 21.4, 18.1, 11.1, 4.7, 24.7, 57.4, 15.5, 93,245 / 108,422 / 1.16 / 600 /
601. Before that (performance 57 → 59, RC-2): performance 25/25,
31.8 turns, 22.8 spells, 19.7 lands, 12.7 combats, 8.6 deaths, 26.7 damage
events, 53.3 damage, 17.6 life changes, 108,902 walks / 135,893 frames / 1.25
per walk / 669 gathers / 670 queries; stress 27/23, 30.6, 21.9, 18.6, 12.1, 4.8,
27.5, 60.5, 17.3, 101,929 / 127,209 / 1.25 / 648 / 649. Before that (performance
55 → 57, RS-1): performance 29/21, 28.4 turns, 21.0 spells, 18.1 lands, 11.2
combats, 5.9 deaths, 22.8 damage events, 49.1 damage, 16.6 life changes; stress
28/22, 28.8, 20.9, 17.9, 12.1, 3.8, 25.5, 57.0, 19.2.*

**These are fixtures, not benchmarks, and the distinction is the point.** Every
row is seed-deterministic, so it is comparable across machines and across months
and a change to it means the *engine's behaviour* moved. **`ms/game` is
deliberately absent** — it was in this table until 2026-09-01 and never belonged:
commit `a926627` established that a stored timing number is machine drift, and
`CLAUDE.md` accordingly mandates an interleaved A/B in one sitting. A number you
cannot compare is worse than no number, because someone will compare it.

### The five bold rows are engine *cost*, and they are here for one reason

The rows above say what the engine **did**; these say what it **spent doing it**
(`state/diagnostics.rs`, added 2026-09-01). They are fixtures by the same
argument — a pure function of the seed and the card pool, verified identical
across three runs and across `--threads 1` and `--threads 8` on both pools — so
a change to one means the engine's **cost model** moved, exactly as a change to
"creatures died" means its behaviour did. Their own overhead was A/B'd and is
below the noise floor.

**They exist because an A/B has to be run by someone who already suspects
something.** RC-2 shipped a 10.3% regression and its 10.3% fix in the same PR,
and neither was visible in any stored number: every behavioural row was
unchanged while `gather` went from walking one permanent to walking all of them.
Layer walks per game would have roughly tripled and said so on sight. That is
the gap this closes — not "make it fast", but "notice".

**How to read them.**

- **Layer walks** is the headline. Almost every cost question in this engine
  reduces to how many full CR 613 walks a game does. Since the epoch memo
  (item 7a, 2026-09-03) it is the *miss* count: a walk is paid once per object
  per epoch rather than once per question.
- **Memo hits** is the rest of the questions. Walks plus hits is what the
  oracle was asked, and it did not move when the memo landed. A hit share that
  falls means some writer started bumping the epoch more often; one that rises
  means the games got more repetitive, not the engine cheaper.
- **Frames/walk** is what CR 613.7a's existence re-check costs — a walk needing
  no sub-frame is 1.00, and `layers-architecture.md` §5.2's descending ceiling is
  what bounds this number instead of letting it iterate.
- **Replacement gathers** and **restriction queries** are the two sweeps that
  *multiply* into layer walks: one gather can be one walk per permanent. Reading
  them beside the walk count is how you tell "a sweep got greedy" from "the game
  got longer".

**The first thing they say is not about the sweeps.** ~109,000 walks against
**669** gathers means the CR 614 pipeline is a low single-digit percentage of
engine cost even when it sweeps the whole board. The overwhelming majority is
ordinary oracle traffic — `has_type`, `get_effective_power`, summoning sickness,
mana-ability discovery — each a full walk with no memo between calls. **That is
`CLAUDE.md`'s critical-path item 7**, which already schedules cross-call
memoization alongside the CR 613.8 dependency algorithm and already carries a
hard back-stop before Phase 8. The measurement did not find new work; it found
that the work already on the plan is the lever, and that a general optimization
sweep would be aimed away from it. **Item 7a landed that memo on 2026-09-03**:
walks per game 108,626 → 2,663 with every other row in the table unchanged,
and the residual is in `layers-architecture.md` §12.

**Adding them cost two determinism fixes, and that is the sharpest thing here.**
`combat/steps.rs`'s first-strike scan and `targeting.rs`'s `has_any_legal_choice`
both short-circuited an `any` over `battlefield`'s `HashMap` with a layer query
inside. Both were **correct** under `CLAUDE.md`'s rule as it was written: `any`
over a set is order-independent, so the *answer* never varied, and neither site
ever showed up in `determinism_test` or in a `--dump-events` diff. What varied
was how much work the engine did getting there — ~14 walks per 50 games. **Making
cost a fixture is what turned two benign `HashMap` walks into determinism
violations**, and `CLAUDE.md`'s determinism line now says "a choice, log **or
count**" for that reason.

Both columns moved when RS-1 added Sigarda and Diabolic Edict, which is what a
pool addition is expected to do. The stress column's creatures-died halving is
the edict and Kalitas together — the edict kills a creature the combat maths was
counting on, and Kalitas exiles rather than letting things die.

**Both columns moved again when RC-2 added Idyllic Beachfront and Chainbreaker,
and the direction is worth reading.** Every row is *up*: three more turns per
game, and more of everything that happens in a turn. The tapland is why — a land
that enters tapped is a land that produces no mana the turn it lands, so a deck
holding several of them curves out later and games run longer. The
creatures-died jump on `performance` (5.9 → 8.6) is Chainbreaker: a 3/3 body for
`{2}` that arrives as a 1/1 both blocks more often and dies more often than
anything else in that pool. **This is a pool change, not an engine regression** —
the engine A/B for the same PR ran on identical pools and is recorded in the PR.

The stress column carries a third RC-2 card, Adaptive Shimmerer, which is
registered but **not** in `PERFORMANCE_POOL` — which is why only that column
moved a second time and why the two columns now differ by more than the pool
sizes suggest.

### 3.1 The gate: run both pools, and read them differently

Each pool answers a question the other cannot, so a PR runs both. **They are
different instruments, not a cheap and an expensive version of one.**

```bash
python plans/fuzz_ab.py --arm main=../mtgsim_v2_main/mtgsim/target/release/fuzz_games.exe --arm new=mtgsim/target/release/fuzz_games.exe
```

**One sitting, sized to what each number needs (2026-09-03).** The script runs
three kinds of number at the cheapest setting that is still the same number:
the counters and the §3 fixture rows threaded on both pools, because they are
identical at every thread count; the timing rounds serial and interleaved on
`performance` only, because `stress` milliseconds are a threshold that is never
compared; and still 200 games and medians of three, because that is where the
run-to-run spread sits at ~2.4% and cutting either is what raises it. A
three-arm sitting is about four minutes where the hand-run version had grown
past twenty. `--require NAMES` adds the reachability rows; `--arm` is
repeatable, and the first arm is the baseline every other is diffed against —
which is how "registered but not pooled reproduces `main`" is checked by
construction. By hand, the two runs it replaces are:

```bash
cd mtgsim && cargo run --release --bin fuzz_games -- --games 200 --seed 12345 --threads 1
cd mtgsim && cargo run --release --bin fuzz_games -- --games 200 --seed 12345 --threads 1 --pool stress
```

| Pool | Question | Read | Grows with the card list? |
|---|---|---|---|
| **performance** | *Did my change make the engine slower?* | A **delta**, measured as an interleaved A/B in one sitting — never against a stored number | Deliberately: one card per new engine path, with a re-record |
| **stress** | *Is there a card shape that makes the engine fall over?* | An **absolute threshold**: 0 errors, 0 panics, 0 turn-limit hits, and no tail number off its scale | Yes, by design |

**Never A/B any number across a pool change.** It moves for two reasons at once
— the change and the new cards — and that conflation is the thing the pool split
exists to prevent. This is why the performance pool grows *deliberately*: an
addition is the one moment a delta is not readable, so it is a decision with a
re-record attached rather than a side effect of registering a card. A stress run
is pass/fail against a ceiling; only the performance pool measures a delta.

**Determinism check.** Everything outside `fuzz_games`' `=== Timing ===` block is
byte-identical across runs at one seed and at any `--threads`, so `fuzz_ab.py`
gets the three-run check in `CLAUDE.md` for free — every timing round must
reproduce the threaded counter run outside that block, and it prints `NO` under
`deterministic` when one does not. By hand it is a diff of two regions rather
than a hunt for scattered lines. Strip the block and the runs must match
exactly:

```bash
cd mtgsim && for i in 1 2 3; do cargo run --release --bin fuzz_games -- --games 50 --seed 12345 | sed '/^=== Timing ===$/,/^$/d' > run$i.txt; done && diff run1.txt run2.txt && diff run1.txt run3.txt
```

### 3.2 Reading the tail, and why the mean cannot do this job

`CPU/game` is a mean, and **a mean is the one statistic guaranteed to hide a
performance cliff**: a card shape that makes the layer walk fall over moves the
slowest game by orders of magnitude and a 50-game mean by about two percent. The
`tail` lines report p50 / p99 / max instead, and `Slowest game` prints the seed,
because every game is a pure function of its own seed and the outlier replays
alone with `--seed N --games 1`.

**Two tails, and the pair is the point.** `CPU/game` conflates *long* games with
*slow* ones; `CPU/turn` divides that out. Compare them:

**Absolute ms are comparable only *within one measurement sitting*, and that is
not a caveat — it is the method.** Measured 2026-08-31: the performance pool read
~63 ms p50 early in a session and ~72 ms p50 hours later, **13%** apart with
byte-identical game content.

**That gap was A/B'd against the obvious suspect and the suspect was cleared.**
The tree had gained `PermanentFilter::ByOwner`, a new match arm in
`permanent_matches_filter` — which `compute.rs` calls inside the layer walk, the
hottest path the engine has. Two release binaries were built from the two
commits into separate target directories and run **alternately** in one sitting,
four runs each, on the pool whose cards are identical on both sides:

| | pre-`ByOwner` | with `ByOwner` |
|---|---|---|
| CPU/game, 4 interleaved runs | 96.37 / 96.70 / 96.39 / 98.25 ms | 96.87 / 96.52 / 96.69 / 97.59 ms |

**0.2% apart, and the game content byte-identical.** So the arm costs nothing and
the drift is machine state — thermal or background load over a long session, not
diagnosed further because it does not need to be. Note what the table also
shows: *both* binaries read ~96 ms where the same pool read ~83–87 ms earlier
that day. Within a sitting the spread is ~2%; across sittings it is ~15%.

**So the performance pool is the control, and what you compare is the two pools
measured together**, never stress-today against stress-last-week — and a
suspicious cross-sitting delta gets an interleaved A/B before it gets a
diagnosis. The numbers below are medians of three interleaved runs in one
sitting, which is what makes the two columns mean anything beside each other.

**Baselines, 200 games / seed 12345 / `--threads 1`, medians of three
interleaved runs, 2026-08-31:**

| | performance (55 — the pool as of 2026-08-31) | stress (58) |
|---|---|---|
| CPU/game p50 / p99 / max | 69.12 / 418.18 / 514.26 ms | 66.28 / 497.11 / 568.28 ms |
| CPU/turn p50 / p99 / max | 2.59 / 7.04 / 7.58 ms | 2.48 / 7.51 / 8.59 ms |
| game tail ratio (max ÷ p50) | 7.4x | 8.6x |
| turn tail ratio (max ÷ p50) | 2.9x | 3.5x |

**The ratios are the durable numbers**, and that is measured rather than
asserted: across the 13% drift above, the performance pool's game-tail ratio held at
**7.1–7.4x** and its turn-tail ratio at **2.8–2.9x**. Deterministic game content travels too, and it
is what moved when Rest in Peace and Leyline of the Void landed and were widened
to "from anywhere": stress avg turns 30.1 → **29.6**, max turns 98 → **75**.
Games end sooner because graveyards stop filling — the cards doing their job,
not a cost appearing. The performance column did not move at all, because
registering a card was not the same act as adding one to that pool — which is
still true, and is the half of the freeze that survived it (2026-09-01). What
did not survive is the ms figures in this table: they are a *sitting*, not a
baseline, and are kept only as the evidence for the ratios below them.

**The two tails, and the gap between them is the finding.** The game tail runs
7–9x the median and the turn tail about 3x. Most of the game tail is games being
*longer* (73 and 87 turns against a ~30 average); the residual ~3x is genuine
per-turn growth as the board fills — more permanents, more expensive layer
walks. Superlinear but modest, and expected.

**What a regression looks like, then.** A turn tail that climbs while the median
holds is the signal to chase; a game tail that climbs with it is probably just a
longer game. **The p99 equals the max below 100 games** — nearest-rank on 50
samples puts rank 50 at the last element — so run 200 when the tail is the thing
you are reading.

**The mean is still the benchmarking number.** `CPU/game` on the performance
pool at `--threads 1` is what an A/B compares — *within one sitting, against a
binary built from the other tree*, never against a figure in this file. The tail
says whether a *new* cost appeared, not whether an existing one grew.

### 3.3 How many cards a mechanic owes — ask the rule, not a quota

**The question is "is this rule defined over one object, or over several?"** If
several, one card leaves the multi-object branch unreachable — not rarely hit,
*unreachable*, at any game count — and the tests pass because nothing can build
the scenario. Count is the wrong metric; the axis the rule is defined over is
the right one.

**Tier 1 — the rule itself requires two, and one card is dead code.** CR 616.1
applies only among "two or more"; CR 613.7 orders effects *within* a layer, so it
needs two in one layer on one object; CR 614.5's applied set is keyed on effect
*instance*; CR 704.7 collapses two actions with the same result. **Worked
example, measured 2026-08-31 and closed the same day:** exactly one registered
card produced a replacement effect (Kalitas), it is Legendary, and CR 704.5j is
enforced — so no player could control two, and two opposing copies each apply
only to the *other* player's creatures. CR 616.1's entire multi-candidate branch
had never been reachable in a fuzz game. Rest in Peace and Leyline of the Void
are the second and third sources; `phase_rb_integration_test.rs` now reaches
CR 616.1 with two *printed* cards.

**The sharpest part of that finding is what it says about coverage
measurements.** The atom was not uncovered — `ATOM-616.1-001` had a passing test
the whole time, built on `graveyard_probe`, a fixture defined in the test file
and registered in no pool. **A bespoke fixture can cover an atom while the
registered pool cannot build the same scenario**, so `specdb` coverage and fuzz
reachability are different measurements and neither implies the other. When a
rule needs two objects, check that two *registered* cards can produce them.

**Tier 2 — two are valuable, and they must differ in shape.** RB's discovered
hang was "a declined `exempt_from_614_5` optional without a second set" — a bug
at the *intersection* of two attributes. A card with the ordinary shape never
reaches it. The axes worth varying are the ones the CR itself names:
optional/mandatory, self/other (CR 614.15), exempt/not, and which player the
effect is scoped to (that is who CR 616.1 asks).

**Tier 3 — one is plenty.** A keyword flag, a vanilla body, a one-shot with no
interaction surface.

**And the counter-pressure, which is equally real.** A second card of the *same
shape* buys nothing, cards cost authoring plus registration plus a test, and PRs
are sized 1,500–2,500. This must not become "N cards per phase". The current
distribution is the argument for shape over count: **11** keyword creatures for a
boolean flag, **1** card for the whole CR 616.1 pipeline, **1** for Layer 2.

**What makes this cheap now.** Registering a card is not the same act as adding
one to `PERFORMANCE_POOL`, so it cannot move a recorded fixture — it only grows
the stress pool, which is read as a threshold. That is what §3 bought, and it
survived the pool un-freezing (2026-09-01): what changed is that a phase opening
a new engine path now *also* makes the second, deliberate move.

**And the rule this section did not have, added 2026-09-01: a gap register
needs a date on it too.** `codebase-state.md`'s fuzz-pool audit named six
unreachable SBA paths, each with a blocker. Re-measured against 200 stress games
with `--dump-events`, **two of the six had closed on their own** — a token
ceasing to exist reached 43 times per 200 games once RB's Kalitas made tokens,
and multi-member `GameAction::Destroy` batches 155 times, because two creatures
trading in combat is one `execute_actions` call. The multi-member entry had been
written about *mass removal from a spell*, which is still unwritable; the branch
it worried about was being taken constantly by a route nobody had connected to
it. **A gap named by its cause outlives the cause**, so the register is a
measurement and gets re-run, not inherited — and counting an event signature in
a `--dump-events` log is cheap enough that there is no excuse for inheriting.

---

## 4. Sizing a phase, and splitting it

**Size a phase before writing it, and split it in the doc, not in the moment.**
Implementation PRs here run 1,500–2,500 additions; the ones that went badly went
past that. Phase RB shipped at +5,475 across 33 files — 2.2× the largest before
it. The cause was not the decision to keep it whole, it was that nobody counted
first: RA was split into three because someone counted call sites and wrote a
*Measured size* column, and RB got nine bullets and no measurement, so it ran
until it was done.

Sub-phases are numbered (`RA-1`, `RC-2`), not lettered.

- **Every PR in a split carries at least one consumer of what it builds.** The
  tempting seam is "engine first, consumers after", and it is wrong: RB's
  pipeline commit was 1,306 lines with zero integration tests, because the
  consumers are what make a pipeline testable — and its one real defect was
  reachable only from the *last* consumer. Splitting relocates that risk rather
  than removing it, so plan for a later PR fixing an earlier one.
- **Review findings go to `plans/handoffs/<phase>-review.md`, not into a
  session.** Capture everything before fixing anything, triage into
  fix / doc / defer / design, then close one *theme* per session starting cold
  from the file. A dozen unrelated fixes carried in one context is where quality
  degrades; the themes exist because the rows inside one share a mental model.
- **`git log main..HEAD` lies after a squash-merge** (same content, new SHA).
  Check content: `git diff --stat origin/main HEAD`.

---

**A shipped phase marks its own heading.** `#### <code> — <what it was> — ✅
landed <date>` in the owning architecture doc, in the commit that ships it.
This was already the habit for six of RC's phases and missing from four;
`check_state_of_play.py` now reads those markers and fails when `CLAUDE.md`'s
critical path still calls a landed phase "next", which is the drift that made
`roadmap-v2.md` §2 unusable. **It only works on the track that has headings** —
the "can't" and copy docs record phases in sizing tables with no status marker,
so normalising those is the next cheap thing anyone touching them can do.

## 5. The spec database as a gate

`plans/specdb.py` joins the atomic-test corpus to the test suite and the CR, so
coverage is a query rather than prose. Its module docstring is the command
reference; these are the rules around it.

**Annotate at write time**, directly above `#[test]`: `// COVERS:` when the test
builds the atom's whole scenario, `// COVERS-PARTIAL:` otherwise. **Never claim
an atom a test doesn't prove** — a false link is worse than a blank. `suspicious`
is a smell test: a hit means read it, silence proves nothing. Tests with no atom
are normal; this measures rules coverage, not completeness.

**A phase does not close until `owed` is clean for it.** Every atom in the phase
is covered, or explicitly deferred with a reason written down. Add the phase to
`SHIPPED_PHASES` when it lands — that is what arms the gate.

**Why it is a gate and not a report.** Phase 5-Pre shipped carrying 223 atoms
and zero coverage. One of them specified the CR 400.7 `zone_change_epoch` field
by name, and nothing asked — so the design was lost for two years and
rediscovered by hand.

**Triage what `owed` reports as a fact or a feature**, because they have
opposite economics:

- A **fact** — object identity, who cast this, an object's characteristics an
  instant ago — is unrecoverable if not captured at the moment it exists, and
  adding it later means re-threading every system built in between. Record it on
  the first customer. Phase RA was, in its entirety, a facts phase.
- A **feature** — a filter leaf, an enum arm — is a normal diff whenever it
  lands, so defer it freely and apply the two-customers guard
  (`replacement-architecture.md` §8c).

Count cards to decide *when* to build a feature; never to decide *whether* to
record a fact.

---

## 6. Module layout — a `mod.rs` declares and re-exports; it does not define

**Checked, not trusted:** `python plans/check_module_layout.py`, in CI beside the
`CLAUDE.md` budget. It fails on `fn`, `struct`, `enum`, `trait`, `impl`, `type`,
`const`, `static` or `macro_rules!` at the top of a line in any `mod.rs` under
`mtgsim/src/`. `lib.rs` and `main.rs` are exempt — they are crate roots, and a
crate-level re-export belongs in one.

A `mod.rs` may carry module docs, `mod` declarations, `use`/`pub use`, and
attributes. The implementation goes in a sibling **named for what it does**.
`engine/replacement/` is the pattern to copy: `gather.rs` finds things,
`pipeline.rs` decides, `instance.rs` names one, and `mod.rs` is the page you
read to learn that.

**Why this needed a mechanism.** Every `mod.rs` in the crate was a pure
re-exporter for two years, by unwritten convention — and then two phases in a
row put a few hundred lines of working code in one. Nobody argued for it; "this
module is small enough to be one file" is a locally reasonable thought that
produces a globally inconsistent tree. A rule that can fail silently is not a
mechanism, which is §1's argument for the `CLAUDE.md` budget and the same
argument here.

**The cost is not aesthetic.** `mod.rs` is the file you open to find out what a
module *contains*. Once it also contains the implementation, that question takes
a second read — and the next file added to the module has to relitigate where it
goes, because the module no longer has a shape to match.

**Splitting is not the same as adding files.** A one-file module is fine:
`engine/restriction/` is `mod.rs` plus `predicate.rs`, and `predicate.rs` is
allowed to be the whole module. What is not fine is that file being called
`mod.rs`.

---

## 7. Trace pages

A **trace page** is a hand-authored HTML file under `plans/traces/` that walks
two or three real board states through the engine call by call, with every read
labelled by what it consulted — the board, a hypothetical frame, a registry, a
player. It ends in a table of where those reads differ.

It is not a design doc and not a test. A design doc says what the engine should
do; a test asserts one outcome; the page shows the **path between them**, which
is the thing a diff cannot show and a reviewer cannot reconstruct. Both existing
pages were written because a review asked a question the diff could not answer.

Two exist, and they are the template:

| Page | Phase | What it proves |
|---|---|---|
| `rc-4b-entering-is-one-event.html` | RC-4b | four entries through the CR 614.12 look-ahead frame, each read labelled board or frame, and what RC-4b changed trace by trace |
| `cv-1-a-copy-is-a-snapshot.html` | CV-1 | a Cytoshape resolution from the choice to the copy row and back, and CR 707.4's re-copy tearing that row down through the existence check |
| `rc-5-applying-an-entry-can-move-the-board.html` | RC-5 | devour's selection and its nested batch, the zone chain that makes CR 614.13b bite, `frame_of(source)` and §5b's asymmetry, and two entries decided against one board |

**When to write one: at phase close, for a phase that changes *how* a read is
answered rather than what the answer is.** That is the property the two above
share, and it is why a phase that adds a card, an enum arm or a pool entry does
not get one. The phases that qualify were listed when the practice started:
RC-4 ✓, RC-4b ✓, CV-1 ✓, RC-5 ✓, **RS-2, critical-path item 6, item 7**. Budget
two to three hours; that is the right cost for a phase's close and the wrong
cost for a question asked mid-debugging, which is what tier 2 below is for.

**Its examples are the phase's findings, not its feature list.** A page that
walks the happy path explains the feature; a page that walks the board the
review argued about explains the phase.

**Naming: `<phase>-<claim>.html`**, kebab-case, where the claim is the sentence
the page proves — `rc-4b-entering-is-one-event`, not `rc-4b-traces`. The file
name is the first thing the next reader sees, and a phase code alone tells them
nothing.

**Structure**, in this order:

1. **The map** — the boards, the cards, and which trace answers which question.
   A mermaid flowchart of the path, with the phase's new nodes coloured.
2. **Lettered traces** — A, B, C…, each a numbered walk. Every step names the
   function and what it read.
3. **Where the reads differ** — the comparison table. This is the payload.
4. **A closing section that ties it to the phase** — what changed, trace by
   trace, or what is still open.

**Pinned, never maintained.** The first line of the file is an HTML comment
naming the commit and the frozen CR version. A page records the engine *at that
commit*; when a later phase changes a path, it gets **its own page**, and the
old one is left alone. That is what makes it safe to write in this much detail
— nothing has to be kept true — and it is why the pin is not optional: a stale
page has to say so on its own first line.

**Self-contained**, and only the CDNs the artifact sandbox allows: fonts from
Google Fonts, mermaid from jsDelivr, nothing else. **Linked from two places** —
the owning architecture doc's phase entry and the phase's `codebase-state.md`
entry — because the `plans/` markdown stays the authority and a page nothing
links is a page nobody finds.

**What not to do.** Do not generate a page from every test: the point is the two
examples that carry the phase's idea, and eight hundred traces are a log, not an
explanation. Do not diagram code that is about to change — the codebase map
below waited for the entry-hop fix for exactly that reason.

### 7.1 The two tiers this does not cover

Tier 1 is the practice above. Two more were planned with it, and both are
schedulable rather than done:

- **Tier 2 — engine-emitted traces.** A `TraceSink` on `GameState`, off by
  default and gated the way `EngineCounters` is, recording what tier 1 records
  by hand: each proposal entering a batch, each pipeline iteration, each
  top-level layer walk, and the performed events. JSON lines, plus a script
  that turns one into a page in this format, so tier 1 becomes generated.
  **Owed before critical-path item 6** — the first question anyone asks a
  trigger dispatcher is "why did this fire, or not", which is a trace question.
  Sized and scheduled at `codebase-state.md`, "Before Triggered abilities"
  item 5.
- **Tier 3 — the codebase map.** One structural page: the modules and what each
  owns, the chokepoint's arms, the three gate legs a new replacement source
  must extend, the two `permanent_matches_filter`s, the accessor pair, and the
  decision sites item 40 tracks — everything `CLAUDE.md` states as an
  invariant, drawn once. Unblocked since the entry-hop fix landed
  (2026-09-02); a day to draw, then minutes per refresh. It wants a ten-line
  check that its list of `perform_action` arms matches the enum, so it cannot
  rot silently.

**Where this was written down before, and why that was wrong.** All of the
above lived in a section of `rc-4b-entering-is-one-event.html` itself — a
plan for the practice, inside one instance of the practice. It was invisible
to `grep`, unreachable from `CLAUDE.md`'s authority table, and pinned to a
commit like the trace around it, so the convention aged like a snapshot when
it is the one part that must not. Lifted here 2026-09-03. The page keeps its
section as the historical record; **this is the authority.**
