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
  (Everywhere, `cards/token_lands.rs`, is the first admitted under this rule
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
  reached, and **board diversity** — the share of games in which a permanent of
  a colour no required card has entered the battlefield. **An empty `--require`
  changes nothing** — every RNG draw is guarded, so a reachability run and a
  timing run come from one binary without the first contaminating the second.
  **Run it once per phase that adds a card**, and put the resolution count in
  the phase's ledger entry.

  **It no longer seeds the deck's colours from the required card's
  (2026-09-03).** It had to, because nothing in the registry made more than two
  colours, and the price was the board: `--require Cytoshape` put every game on
  a G/U pair, so the card resolved **390** times in 200 games and met a white,
  black or red permanent in **none** of them. Everywhere — the five-colour land
  token, now the fill tier of every deck's mana base (`random_deck`) — is what
  let the seeding go. Same 200 games / seed 12345 / `--threads 1`, `performance`:

  | `--require Cytoshape` | cast | resolved | games | games with a non-G/U permanent |
  |---|---:|---:|---:|---:|
  | with colour seeding (`main`, 6dedaf8) | 400 | 390 | 177 (88%) | 0 — by construction |
  | without | 216 | 212 | 126 (63%) | **198 (99%)** |
  | without, plus `,Everywhere` forced | 217 | 214 | 126 (63%) | 198 (99%) |

  **The resolution count halves, and that is the agent, not the deck.** Every
  deck can pay `{1}{G}{U}` — fourteen or more of its 24 lands tap for any
  colour — but the random agent picks a (land, ability) pair uniformly in the
  601.2g window, so each Everywhere tap is a five-sided die. Three uniform taps
  hold both a G and a U about one time in five (1 − 2·(4/5)³ + (3/5)³ ≈ 0.19;
  four taps ≈ 0.31, five ≈ 0.42), where a seeded deck's Forests and Islands
  could not tap wrong. A failed payment rewinds silently and leaves the lands
  tapped (`backlog.md` §2.18), so the turn's mana is spent on nothing: land
  taps per spell cast go 3.86 → 7.66 on `performance` (40-game
  dumps). That is §2.18's auto-payment oracle measured from the other side —
  the harness now has the mana base a Commander deck has, and an agent that
  cannot use it. Forcing Everywhere on top changes nothing, because the deck
  already holds a dozen.
- **Say which pool a number came from.** `fuzz_games` prints it in the header
  and in the results block. The two pools are not comparable to each other, so a
  pasted stats block without its pool name is not evidence of anything.

**Gameplay fixtures, 50 games / seed 12345 / `--threads 1`, re-recorded
2026-09-03 (performance 63 → 64, stress 71 → 72; Everywhere, and with it a new
mana base and no colour filter in `random_deck` — a pool *and* deck-construction
change, so nothing here is comparable across it):**

| | performance (64 cards) | stress (72 cards) |
|---|---|---|
| P0 / P1 | 30 (60.0%) / 20 (40.0%) | 31 (62.0%) / 19 (38.0%) |
| Avg turns | 36.0 | 36.6 |
| Spells cast | 22.2 | 21.7 |
| Lands played | 20.6 | 20.7 |
| Combat w/ atk | 11.2 | 11.0 |
| Creatures died | 5.2 | 3.2 |
| Damage events | 22.3 | 22.2 |
| Total damage | 50.8 | 48.2 |
| Life changes | 16.5 | 17.9 |
| **Layer walks** | **108,423** | **114,839** |
| **Layer frames** | **158,203** | **148,874** |
| **Frames/walk** | **1.46** | **1.30** |
| **Replacement gathers** | **650** | **694** |
| **Restriction queries** | **651** | **696** |

**Both columns moved, and the engine's share was separated from the pool's
before this table was recorded (2026-09-03).** Three binaries, one sitting,
200 games / seed 12345 / `--threads 1`, three interleaved rounds: `main`
(6dedaf8), Everywhere registered but not pooled and `random_deck` untouched
(1ad5797), and shipped (eb9efb8). **The middle arm reproduces `main` byte for
byte outside `=== Timing ===` on `performance`** — registering a card is still
not the same act as adding one — and on `stress` it differs only by the
registered card being drawn.

| | A: `main` | B: registered, not pooled | C: shipped |
|---|---|---|---|
| performance turns / spells / deaths | 33.3 / 27.9 / 7.8 | 33.3 / 27.9 / 7.8 | 37.4 / 22.6 / 5.6 |
| performance walks / frames per walk | 116,233 / 1.33 | 116,233 / 1.33 | 117,031 / 1.41 |
| performance ms / 1,000 walks (median of 3) | 1.064 | 1.121 | 1.476 |
| stress turns / spells / deaths | 30.9 / 24.8 / 4.6 | 31.2 / 24.9 / 5.1 | 38.7 / 22.6 / 3.8 |
| stress walks / frames per walk | 100,619 / 1.29 | 99,938 / 1.28 | 123,868 / 1.32 |
| stress ms / 1,000 walks (median of 3) | 0.992 | 0.961 | 1.328 |

**Read it as two changes, and they pull in different directions.** A against B is the registration: identical counters on `performance`, and per-walk time inside the spread (+5% / −3%). B against C is the deck construction, and every behavioural row moved the way an any-colour mana base under a random agent should move it: games run four to seven turns longer (33.3 → 37.4 and 31.2 → 38.7), spells cast per game fall (27.9 → 22.6 and 24.9 → 22.6) because the agent taps Everywhere for random colours and rewinds (`codebase-state.md` 16d), and fewer creatures die because fewer are cast. Walks per game are flat on `performance` (+0.7%) and +24% on `stress`, where the longer games and the whole-pool statics show up. **The row to read is the cost row: +34% / +39% per 1,000 walks, far outside the ±4–6% spread — and it is the objects, not the engine.** A fourth binary settled that in the same sitting: **D** is C with the any-colour fill tier replaced by basics (`BASIC_LANDS_PER_DECK` 5 → 19, nothing else), and it reads **0.977** ms per 1,000 walks against 1.060 for `main` and 1.421 for C, both re-run beside it in its own three interleaved rounds (`performance`; the table's 1.064 / 1.476 are the earlier rounds, and the gap between the two sittings is the usual ~4%). So drawing nonlands from the whole pool costs nothing per walk, and the fill tier costs a third: a five-ability, five-subtype land is the object the 601.2g window walks most, and every walk of one clones five abilities and five subtypes into its frame where a dual clones two. That is a fact about `compute_characteristics` seeding each frame from `CardData`, and critical-path item 7's cross-call memoization is still the lever — no engine line changed between A and C, so it is a heavier board, not a regression. The number the timing arm carries from here is C's, and an A/B is still a delta within one sitting on one pool. (Two things seen and not chased: D casts more spells than C, 24.5 against 22.6, and Blood Moon is the likely reason — it turns an Everywhere mana base entirely red and leaves basics alone; and the first player's share of wins rises from 104/96 to 122/78 at 200 games, a tempo effect of a mana base that always has the colour.)

**The first re-record where the pool did not move, so every row is the
engine's (2026-09-02, 16c).** The row to read is `Spells cast`, and it went
*up* — 21.0 → 26.4 and 19.6 → 25.1 — without five more spells per game being
cast: they are the same spells, now announced. The old tree resolved ~5.5
spells per game without a `SpellCast`, so the row never counted them, and the
200-game dumps put `main` at 27.2 resolutions per `performance` game against
21.7 announced. Every other row moved because those spells are now paid for:
games run about two turns longer, and the cost rows follow the length (walks
+11% / +14%, walks per turn +5% / +5%). Per-walk *time* did not move — −4% /
−6% at 200 games, interleaved, inside a sitting — and the ghosts were
disproportionately the statics that put a sub-frame under every walk, which is
where the small `Frames/walk` movement comes from. **The check the fix added
is deliberately not a row here.** `fuzz_games` prints `Uncast resolved:` beside
`Errors:` and `Panics:` and fails the run on any value but 0; that is a
threshold, like the stress column's, and a threshold in a fixture table would
be read as a number that can drift. Both runs above read 0.

**Most of the movement in the CV-1 re-record (the first entry under *Previous
values*) was not CV-1's, and the version before it was already stale when CV-1
read it.** `main` at 103acf1 answers 93,717 walks / 530
gathers on `performance`, against the 93,854 / 594 the table printed — so the
gather column had fallen ~11% before this phase touched anything. The cause is
**RC-4b**, which made entering the battlefield one proposal instead of two and
merged without re-recording here. CV-1's own contribution is the small half:
+197 walks and +1 gather on `performance`, +249 walks and +3 on `stress`. **The
rule the miss argues for is the one already written above** — re-record the
table in the PR that moves it — and the reason it is cheap to forget is that
nothing fails when you don't. Worth one line in a phase's exit checklist rather
than a check: the numbers are seed-deterministic, so a stale row is silently
wrong rather than noisily so.

**Both columns moved, and the engine's share was separated from the pool's
before this table was recorded** — three binaries in one sitting, medians of
seven interleaved rounds: `main` (A), RC-4's engine with the pools exactly as
`main` had them (B), and RC-4 shipped (C).

| | A: main | B: engine, pools unchanged | C: shipped |
|---|---|---|---|
| performance walks | 108,632 | 108,709 | 93,854 |
| performance frames/walk | 1.25 | 1.25 | **1.45** |
| performance ms / 1,000 walks | 0.912 | 0.940 (+3.1%) | 1.302 |
| stress walks | 98,843 | 98,889 | 85,738 |
| stress frames/walk | 1.20 | 1.20 | **1.31** |
| stress ms / 1,000 walks | 0.775 | 0.787 (+1.6%) | 0.944 |

**`Frames/walk` is the number RC-4 was told to watch, and it moved for the
reason the plan gave.** A against B — the frame, with nothing new on the board
— is flat on every fixture row and inside the spread on time (−2.2% / +1.1% at
200 games over three interleaved rounds). What moved in C
is Keldon Warlord: a walk of the Warlord asks every permanent's frame at layer
7a's ceiling, so it costs 1 + N frames where an ordinary walk costs about 1.25,
and `Frames/walk` carries it. Games also got shorter (31.7 → 28.7 turns on
`performance`), so walks per game *fell* while each walk cost more. That is
`layers-architecture.md` §12's quadratic by design, measured rather than
assumed, and critical-path item 7's cross-call memoization is still the lever.

*Previous values, 2026-09-02 (pools unchanged at 63 / 71; a cast whose payment
fails now rewinds instead of resolving unpaid, 16c): performance 25/25, 30.6
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
  reduces to how many full CR 613 walks a game does.
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
sweep would be aimed away from it.

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
byte-identical across runs at one seed, so the three-run check in `CLAUDE.md` is
a diff of two regions rather than a hunt for scattered lines. Strip the block and
the runs must match exactly:

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
