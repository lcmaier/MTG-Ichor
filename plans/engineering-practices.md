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
- **Say which pool a number came from.** `fuzz_games` prints it in the header
  and in the results block. The two pools are not comparable to each other, so a
  pasted stats block without its pool name is not evidence of anything.

**Gameplay fixtures, 50 games / seed 12345 / `--threads 1`, re-recorded
2026-09-01 (performance pool 57 → 59, RC-2):**

| | performance (59 cards) | stress (63 cards) |
|---|---|---|
| P0 / P1 | 25 (50.0%) / 25 (50.0%) | 27 (54.0%) / 23 (46.0%) |
| Avg turns | 31.8 | 30.6 |
| Spells cast | 22.8 | 21.9 |
| Lands played | 19.7 | 18.6 |
| Combat w/ atk | 12.7 | 12.1 |
| Creatures died | 8.6 | 4.8 |
| Damage events | 26.7 | 27.5 |
| Total damage | 53.3 | 60.5 |
| Life changes | 17.6 | 17.3 |

*Previous values, 2026-09-01 morning (performance 55 → 57, RS-1): performance
29/21, 28.4 turns, 21.0 spells, 18.1 lands, 11.2 combats, 5.9 deaths, 22.8
damage events, 49.1 damage, 16.6 life changes; stress 28/22, 28.8, 20.9, 17.9,
12.1, 3.8, 25.5, 57.0, 19.2.*

**These are fixtures, not benchmarks, and the distinction is the point.** Every
row is seed-deterministic, so it is comparable across machines and across months
and a change to it means the *engine's behaviour* moved. **`ms/game` is
deliberately absent** — it was in this table until 2026-09-01 and never belonged:
commit `a926627` established that a stored timing number is machine drift, and
`CLAUDE.md` accordingly mandates an interleaved A/B in one sitting. A number you
cannot compare is worse than no number, because someone will compare it.

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
