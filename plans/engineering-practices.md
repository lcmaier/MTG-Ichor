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
- **Status is not content** (added 2026-09-07). The file forbade "progress
  snapshots or counts" and then spent its largest section on one: the critical
  path carried ✅ marks, four dates and "RA, RB and all of RC are in", and it
  was the one section that went stale on every merge. It now carries ordering
  and dependencies only. **Landed status is derived** onto `state-of-play.md`
  from a `###`/`####` architecture-doc heading carrying ✅ — the convention
  three docs already followed. Making that true took two markers (RS-1, CV-1)
  and one defect: `ARCH_DOCS` was a hand-maintained list that had never been
  extended for `cost-architecture.md`, so CM-0, CM-1 and CM-2 shipped with the
  right heading and appeared on no board. The list is a glob now.
- **When the cap binds, choose the removal against the whole file.** The
  cheapest edit is always to compress whatever sits next to the insertion, and
  that optimises for shortenable rather than for least valuable. This section's
  own trim is the example: three lines were taken from the ✅ entries for 6b and
  7 to make room for `6a`, four hours before the ✅ entries were deleted
  outright as the wrong kind of content.

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

### 2.1 The comment audit — why the guard is a sweep and not a budget

**Asked in the RD-1 review (2026-09-08): should `CLAUDE.md` grow a rule against
over-commenting, or should the codebase get periodic sweeps?** The answer is
the sweep, and the reason is what the review actually found.

It found three comments in two card files asserting that `fuzz_games::random_deck`
"filters nonlands by color", with probabilities derived from it. Every one was
true when written; the filter was removed months later by an unrelated change
(the `Everywhere` land, 2026-09-03) that had no reason to touch a card file.
**A length budget would not have caught any of them** — they were short, they
were *why* rather than *what*, and each one justified a real decision. What was
missing was a re-read.

So the guard is an audit with an instrument, on the Deferred Migrations
triage's cadence — **every 15–20 PRs**, and at the close of any phase that
changed a measurable:

1. **Grep for claims that name a number**: `grep -rnE "(roughly|about|one in|
   [0-9]+%|[0-9]+ of [0-9]+|measured|counted)" mtgsim/src plans` over comments.
   A comment that carries a number is a comment that can go stale silently,
   because nothing recomputes it.
2. **Re-derive each one, or date it.** A claim that is still true gets a date;
   a claim that has become history gets rewritten as history — "this was true
   when X shipped, and Y changed it" — rather than deleted, because the
   decision it justified is still in the tree.
3. **Delete what the code now says.** A comment that restates a line it sits
   above is the *other* failure, and this is where it gets removed — but it is
   the cheaper of the two, because a redundant comment is noise while a stale
   one is a wrong answer a reader will act on.

**First recorded application — 2026-09-15, the post-RE audit's pass 2 (PR
#143 for CI, PR #144 for the sweep), 80 PRs after the last sweep at #62.**
The grep as written found **456 hits in `mtgsim/src`** (133 in `cards/`),
**140 in `tests/`** and **1,049 in live `plans/`** (archive and the corpus
excluded). Most of the `src/` count is the instrument's noise: 265 of the 323
non-card hits were "about" as a preposition or "one in" inside "zone in" and
"one instead". Read with a second, tighter tier — `about`, `roughly` and `one
in` only before a quantity; `counted` and `measured` as verbs; any `N%` or
`N of M` — the non-card `src/` list is **58**, of which 39 sat in a paragraph
carrying no date, phase code or PR number, and **13 were measurements or
censuses with no date**. Each was dated from `git blame`, or re-derived where
the claim was about the tree (`pipeline.rs::substitute`'s counted block, 7 → 8
templates and 235 → 310 lines; `card_pool_lowering_test`'s 52 → ~150 call
sites); one estimate nobody had derived (`ManaPool`'s "99% case") became "the
common case", and one future-tense doc (`GameAction`'s "in Phase 6 a pipeline
will…") became history. Every `cards/` hit was a Scryfall census carrying its
date already. The twelve `TODO`s citing the archived plan's vocabulary got a
live owner in place of the label or were deleted where the code had moved on
(one new owner had to be filed: `backlog.md` §2.32). `plans/` is dated by
convention almost everywhere — the `Found by …` headings, `fuzz-record.md`'s
`**Re-recorded YYYY-MM-DD**` blocks, the census line under each "Scope,
measured" — and the six undated measurements left were dated in place. The
sweep also caught what the instrument does not look for: one American-spelling
gap at twelve sites (the forms are on `check_glossary.py`'s list now) and
three stale line-number pointers in `codebase-state.md`. **After the sweep the
tighter tier reads 19 in non-card `src/`, 6 in `cards/`, 12 in `tests/`, all of
them `counted` or `measured` describing what the code does rather than a
number** — that residue is the instrument's floor, not work. **Step 3 is a
separate read, and the first cut skipped it**: dating and re-deriving replaced
comment lines one for one (97 out, 96 in), and the owner read the net-positive
PR as a sweep that cut nothing. Applied in full afterwards — every inline
comment block in the 55 files the two PRs touch, read against the line beneath
it — 144 lines came out of 20 files, nearly all from the pre-RB files
(`mana_helpers`, `turns`, `game`, `zones`, `cast`, `types/mana`, `steps`) and
almost none from the RB-era ones; 50 went in where an archived label needed a
live owner. Read the whole file's comments, not the grep's hits. **Next time,
run both tiers**: the tighter one is the list to read, the wider one is the count
to record here, and a hit under the tighter tier with no date in its paragraph
is the work.

**Second application — 2026-09-15, the same day, after the owner read the
first as still too small (PRs #146, #147, #148, #149).** "Restates
directly below it" catches a comment that repeats the code and misses a war
story or an over-explanation, which is where the length was. Sized against the
tree before starting: non-card, non-test `src/` carried **15,619 comment
lines**, 12,675 of them in blocks over four lines (9,066 doc, 2,882 inline, 727
module) and **4,358 in 249 blocks that narrate history** — phase codes, "used
to", "until YYYY-MM-DD", "this PR", "the old …". Two rules, applied whole file
by whole file from the largest candidate down and split at §4's diff band:
history narration comes out of every comment kind, a doc comment keeping its
summary and contract; and an inline `//` block over four lines is compressed
to the decision, the rule cite and a pointer to the architecture doc that
carries the reasoning. Card files were left alone — their comments are Scryfall
censuses, dated already. Across the four PRs (3, 4, 49 and 1 files) the
whole-file comment counts went **3,235 → 2,798, 3,358 → 2,794, 10,164 → 9,722
and 521 → 492: 17,278 → 15,806, 1,472 lines out** and the source diffs 1,191,
2,006, 1,932 and 463 lines. **The instrument for this pass is not the grep above.**
It is a block list — every comment block over four lines, per file, with the
history words flagged — read in full, then a second grep over the *whole* file
for phase codes and the history words, because the block list misses a
one-line "since RC-1" and a "Phase RB" in a section header; the second grep
found about forty more sites the first pass had walked past. What it turned up
that neither instrument looks for: **twelve comments the code had outgrown**,
each corrected rather than shortened — `execute_action`'s doc still describing
a "direct passthrough" with the pipeline as a future step, `ActionContext`'s
"nothing reads either field yet" on fields with nine readers, `resolve_effect`'s
"Phase 2 scope", a `replacement_ability_sources` doc waiting on a copy leg
that existed, two "one reader" claims where there were three, a `CR 701.?`
placeholder, a Destroy citing Create's rule number, a `first_strike_only` that
"always false", a CR 704.7 test premise the losses batch had falsified, a
"nothing reads `ctx.dp`", and an `ObjectFilter`-is-the-wrong-name paragraph
that outlived the rename it asked for. And one gate interaction: deleting the
only "this used to be `forced_bucket`" note in the crate turned
`check_glossary.py` red, because the glossary's **step** entry anchored on the
old name; the entry now says the rename without it. **Next time, run the
history grep over every touched file before opening the PR**, and read the
doc comments for stale claims as a step of its own — a shorter comment that is
wrong is not an improvement.

**What stays out of `CLAUDE.md`:** a comment-length rule. That file is 200
lines and every section costs another; the rule it already has is the right
one, and a second rule that duplicates it in the negative would buy nothing
that this sweep does not.

## 2a. When a one-line wrapper earns its place

Added 2026-09-14, at the LK review, because the question recurs: *is this
one-line wrapper justified?* It kept being answered per-site, and the answer
kept being "it reads better", which is not an answer — a name that only
re-spells an expression is a second name for one thing, and the reader now has
to learn both.

**"It reads better at N call sites" is not a reason when N is small, and stops
being a reason at all when N is 1.** Three things are:

1. **It names a question the expression only answers implicitly**, *and* that
   name is already used in prose elsewhere. The test is whether the docs
   already say "does X function in zone Z" — if the phrase is in the
   architecture doc, the function should have it too. A name nobody writes in
   prose fails this.
2. **It pins a comparison or a borrow the callers would otherwise get wrong.**
   The strongest form: the hand-written version compiles and is subtly wrong.
   `zone_function::functions_in` is the worked example — it wraps
   `functioning_zones(a, t).contains(z)`, and the shape a caller reaches for
   instead is `functioning_zones(a, t) == ZoneSet::of(z)`, which is right for
   every one-zone statement and silently false for Squee, the Immortal's "from
   your graveyard **or** from exile". One caller, and the wrapper is still
   worth it.
3. **It is the narrow half of a pair whose wide half must stay reachable.**
   The wide one is what tests assert against and what a future caller needs;
   the narrow one is what callers should reach for. Both public, and the
   narrow one's doc says which is which.

**What to do when none of the three applies:** inline it. And when one does,
**write the reason in the doc comment** — the reason is the whole justification,
so a wrapper whose doc says only what the body says has not been justified. The
review finding this section came from was a doc comment claiming "every caller
but registration asks this shape" about a function with exactly one caller; the
claim was checkable and wrong, and the real reason (2, above) was better.

Related and different: a wrapper that exists to be *the one door* through which
a mutation happens is not this section's subject. `GameState::
insert_battlefield_entity` and `set_object_timestamp` are chokepoints, and a
chokepoint is justified by what it makes impossible, not by what it reads like.

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
  to it, deliberately, and re-records both tables in `fuzz-record.md`.** Adding
  is still not the same act as registering. `performance_pool()` panics on a name
  that is no
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
  reached, **copies per deck**, and **board diversity**. **The flag repeats,
  and that is how a card whose name contains a comma is named at all**
  (RE-8, 2026-09-14): `-r "Opt" -r "Eligeth, Crossroads Augur"` is the union
  of both, deduplicated, in the order given. Before it, Vorinclex, Monstrous
  Raider (RE-5) and Eligeth were read through their tests instead — two
  phases running, which is what made the ten-line fix worth taking — the share of games in
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

**The tables these rules produce are in `plans/fuzz-record.md`, newest first**
— a phase appends its block there whenever it moves a pool; the rules stay here.

**Those tables are fixtures, not benchmarks, and the distinction is the point.**
Every row is seed-deterministic, so it is comparable across machines and across
months and a change to it means the *engine's behaviour* moved. **`ms/game` is
deliberately absent** — it was in them until 2026-09-01 and never belonged:
commit `a926627` established that a stored timing number is machine drift, and
`CLAUDE.md` accordingly mandates an interleaved A/B in one sitting. A number you
cannot compare is worse than no number, because someone will compare it.

### The five bold rows are engine *cost*, and they are here for one reason

A record entry's behavioural rows say what the engine **did**; the five bold ones
say what it **spent doing it** (`state/diagnostics.rs`, added 2026-09-01). They
are fixtures by the same argument — a pure function of the seed and the card
pool, verified identical across three runs and across `--threads 1` and
`--threads 8` on both pools — so
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
- **Dependency checks** is the CR 613.8 loop's slow path (LI-2): the
  hypotheticals a game ran, each a frame clone per member reached. Zero
  means the static channel check settled every pair in every layer; with
  Urborg pooled it reads ~23 per game, nearly all Blood Moon beside Urborg
  and Humility beside a creature static.
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

**One thing this instrument does not measure, found the hard way.** The
`--require` block counts a card's **casts**, and RD-3's pooled question was
about an *activated ability*: Circle of Protection: Red resolves in 130 of 200
forced `stress` games, and its `{1}` ability is activated **12,660 times** in
129 of them — about 98 per game it reaches the battlefield, which is the
opposite of the prediction that asked for the number. The count came from
`--dump-events` plus `grep "AbilityActivated: <name>"`, and until the report
grows a counter that is the recipe. **A phase whose consumer is an activated
ability should measure the activation, not the cast.**

**Asked at the RD-3 review: cap how often the random agent may re-activate one
ability? No** — and the reasons are this section's own doctrine. A cap makes
every counter a function of a policy knob, which is exactly what keeps the
seed-deterministic rows comparable across phases; the agent is not biased
toward the ability in the first place, because `candidate_priority_actions`
lists it **once** however many times it could be paid for, so what the number
shows is leftover `{1}` having nothing else to buy; and the observed rate is
already about 3–5 activations per turn, so a 3–5 cap would bind almost never
while making the instrument dishonest — and a control that does *nothing* when
activated is activated just as freely, so this is general agent behaviour and
not a fact about one card.

`codebase-state.md` item 104 carries the isolating control table (LK,
2026-09-13). The headline
matters for how this section reads a future phase: **the cost is the repeatable
activation, not the registry rows it happens to make** — ~+29% CPU for the
activation traffic and ~+6% for the rows on top — because an activation is a
priority action plus a stack object plus two more priority rounds, so the game
does more of everything it already does. The wanted fix is a per-ability
activation **counter**, a diagnostic, never a behavioural cap.

**And the corollary this section owes, because the first draft got it
backwards** (item 105): "keep repeatable activations out of `PERFORMANCE_POOL`"
is *not* the rule and must not become one. The pool has never held an ability
activatable more than once a turn — seven registered non-mana activated
abilities, two pooled, both `{T}`-gated — and that is an accident of what the
phases have needed, not a boundary being enforced. Repeatable activations are
ordinary Magic and are most of what a commander does; a cost instrument that
excluded them would drift from the 4-player Commander game it exists to
predict. §3's rule is unchanged — the first phase whose engine path *is* an
activated ability pools one deliberately and re-records the table — and item
104's numbers are the expected direction, not a reason to decline.

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
construction. `--deck-size 100 --life 40 --players 4` is the Commander-scale
board `codebase-state.md` item 138 reads the ratchet on; both flags pass
through to `fuzz_games`, and at their defaults — 60 and 20 — they change no RNG
draw, so every number recorded before they existed is the same run. By hand, the two runs it replaces are:

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

**The budget (adopted 2026-09-15, on the review of the post-RE audit's pass 3).**
A phase's `performance` delta at four seats may cost at most **2.5 points of
CPU per game at identical counters** — CPU per *decision* since item 138's
counters landed (A4e, 2026-09-16), a decision count being a fixture row like
the rest — or the PR says why in its `fuzz-record.md` block and the reviewer
decides. RE-9 used the number as its gate
(`replacement-architecture.md` §11 item 54); this makes it the rule. It is the
per-PR half of a ratchet: the other half is the readiness pass of each
spine-phase audit, which records decisions per core-second on both boards as a
dated reading beside the previous close's and may not read worse per decision
without a written reason. The owner chose a ratchet over an absolute rate
because today's pool has few abilities per permanent and no triggers, so a
number about this board is a floor of Commander's cost rather than a target;
"as little as possible" is the goal, and this is the form of it that can be
watched. **The reading that travels across machines is an instruction
count**, not a millisecond: `valgrind --tool=callgrind` over `fuzz_games` at a
fixed seed in the WSL Ubuntu distro (a four-seat `stress` game is ~2.8 s under
it, so 200 games take ten minutes, and the counters outside `=== Timing ===`
come out identical to the native run's), first taken by item 138 on
2026-09-15 and read in `layers-architecture.md` §12; milliseconds stay in the
A/B sitting.

**Determinism check.** Everything outside `fuzz_games`' `=== Timing ===` block is
byte-identical across runs at one seed and at any `--threads`, so `fuzz_ab.py`
gets the three-run check in `CLAUDE.md` for free — every timing round must
reproduce the threaded counter run outside that block, and it prints `NO` under
`deterministic` when one does not. By hand it is a diff of two regions rather
than a hunt for scattered lines. Strip the block and the runs must match
exactly. **Under three hasher seeds, since A4g (2026-09-16):** ids are
process-stable and the id-keyed maps hash with `types::ids::IdHash`, seeded
once per process from `MTGSIM_HASH_SEED`, so three runs under one seed would
iterate every map in the same order and agree with each other whether or not a
sweep is ordered. A different seed per run restores the property the check had
under `RandomState`; `fuzz_ab.py` sets one per timing round and CI's step sets
one per run.

```bash
cd mtgsim && for i in 1 2 3; do MTGSIM_HASH_SEED=$i cargo run --release --bin fuzz_games -- --games 50 --seed 12345 | sed '/^=== Timing ===$/,/^$/d' > run$i.txt; done && diff run1.txt run2.txt && diff run1.txt run3.txt
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
The tree had gained `ObjectFilter::ByOwner`, a new match arm in
`object_matches_filter` — which `compute.rs` calls inside the layer walk, the
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

### 3.4 The rulings pass — read a card's rulings before you register it

**Adopted 2026-09-08 (the owner, reviewing CM-3).** Oracle text has been
verified against Scryfall since the corpus started. Rulings had not been, and
they are the cheaper half: a card's rulings exist *because players got those
cases wrong*, so they are a free list of the boards a naive implementation
misses — written by the people who adjudicate them, and already minimal.

**The rule: every card a phase registers gets a rulings pass, and the pass is
written down.** For each ruling, one of three answers, and the third is as
useful as the first:

1. **Testable now** — it becomes a test, named after the ruling's claim.
2. **Already tested** — say which test, so the next reader does not redo it.
3. **Not yet expressible** — say which facility is missing and who owns it.
   This is the valuable one: a ruling the engine cannot state is a gap
   discovered from the outside, by someone who was not looking at the code.

Fetch them with `curl` and a UA header, like the oracle text (`CLAUDE.md`'s
references row): `/cards/named?exact=…` carries a `rulings_uri`.

**Where it goes: in the card's own doc comment, under `# The rulings, and where
each is tested`** — RD-1's shape, and the one to copy. Written down after RE-3's
review asked the same question it had asked at RE-2's: RE-1, RE-2 and RE-3 each
appended a module-level "rulings pass" block, and by the third the header of
`phase_re_cards.rs` was **412 lines** of prose a reader had to scroll past to
reach a card. Moved onto the cards it costs nothing and reads better — the
rulings for Ali from Cairo are the paragraph above Ali from Cairo — and the
module doc keeps what is genuinely about the *phase*: the axes, the design
claims the cards were chosen to press on, and what a random deck can draw.

**Format-variant rulings are dropped, not answered.** A Two-Headed Giant ruling
gets no line: CR 810 is not implemented, "n/a" is not one of the three answers,
and the list's whole value is that every entry names a board. The same goes for
any ruling about a format §9 has not built.

**What it caught on its first outing (CM-3, five cards, eleven rulings).** Six
became tests, and none of them was a board the corpus had:

- *"If a spell is both black and green, you pay {1} less, not {2} less"*
  (Thunderscape Familiar). The card is one ability with an `Or` over two colour
  leaves, so the gather returns one instance. Written as two abilities — the
  obvious alternative, and the one a card author reaches for first — it would
  reduce twice. **No atom in the corpus asks this**, and no test in CM-1 or
  CM-2 could have caught it.
- *"You must sacrifice exactly one creature … you cannot sacrifice additional
  creatures"* (Altar's Reap). The prompt's bound is the claim, and
  `picking_all` is the provider that falsifies it.
- *"Players can only respond once this spell has been cast and all its costs
  have been paid"* (Altar's Reap) and *"no player may take actions to try to
  remove Foundry Inspector before that spell's cost is locked in"* — two cards'
  rulings, one engine claim: nothing yields priority between CR 601.2a and
  601.2i. Asserted as the absence of a priority prompt.
- *"Choose the value for X first, and then reduce the cost by {1}"* (Foundry
  Inspector), with the ruling's own numbers as the test's.

Two were already tested (the Familiar's floor at zero; "if this card is
sacrificed to pay part of a spell's cost, the cost reduction still applies",
which is `ATOM-601.2h-001`). Three were not expressible and are recorded as
such rather than skipped — chiefly *"the generic X cost is still considered
generic even if there is a requirement that a specific color be used"*, which
needs the spend-restriction model (`backlog.md` §2.19's neighbourhood).

**And a leaf the pass does not reach by itself.** Writing the Familiar's
"both black and green" test made the card's `Or` look covered when only its
*left* leaf was: every board in the file casts a black spell, so
`Or(Black, Black)` would have passed all of them. A ruling names a case; it does
not name the ways an implementation can accidentally satisfy it. **After the
rulings pass, ask what the card's filter has that no test varies** — here the
right leaf and the "you cast" clause, three assertions, all of them green.

### 3.4a How big is the retroactive half, and is the tool worth building?

**Deferred once as "not worth the time", and the deferral was measured on the
wrong thing.** The idea was a harness that forces every card in the official
pool to pass all its Scryfall rulings. That tool cannot exist: a ruling is
prose, and nothing compiles prose into an assertion. What *can* exist is a
**ledger with a gate**, the shape `specdb` already proved — the corpus is
authored, the join is generated, and the check fails when a claim has no
disposition.

**Census, re-counted 2026-09-17 at A4b, and this is the live one.** The
2026-09-08 figures it replaces — 92 registered cards, 43 carrying a ruling
(46%), 145 rulings, 87 of them on the then-73 pooled cards — are kept in the
line below because the *rate* moved and the rate is what sizes the rest.

| | |
|---|---:|
| registered names | 161 |
| …real printings (the rest is one deliberate fixture, Loyalty Probe) | 160 |
| cards carrying at least one ruling | 96 of 160 (60%) |
| total rulings | 330 |
| …on the 90 `PERFORMANCE_POOL` cards | 125 |
| median rulings per card | 2 |
| most on one card | 9 (Stunning Reversal, Live Fast, Bard, King of Dale — none pooled) |
| most on a *pooled* card | 8 (Cytoshape) |

`python plans/check_rulings.py` prints it from the ledger; nothing here is
typed twice.

**The registry grew 1.7× and the rulings grew 2.3×**, which is the number a
re-count exists to find. Per card it is 1.58 → 2.06 and the carrying rate 46%
→ 60%, so the phases since CM-3 registered rulings-heavier cards than the ones
before them — unsurprising in hindsight, since a replacement or copy card is
exactly the kind players ask about. It also means **a projection from the old
ratio would have been wrong by a third**: 145 scaled by card count is ~250, and
the count is 330.

330 is still a bounded job. At CM-3's observed rate — 11 rulings into 6 tests,
2 already-covered, 3 named gaps — it is roughly 180 tests across the whole
registry, or 68 if the pool goes first. A4b's own sitting came in cheaper than
that rate, for a reason worth writing down: see "what the queue's head is
made of", below.

**What the census cost, and what it immediately bought.** Reading the rulings of
three pooled cards nobody had checked found `codebase-state.md` **item 82**:
CR 704.5p's first sentence is not implemented, so an Equipment that becomes a
creature stays attached — Bonesplitter under March of the Machines, both of them
in `PERFORMANCE_POOL`, leaving a 2/2 reading 4/2 in any measured game where they
meet. Nothing in the corpus, the suite or the fuzz harness had said so, and
March's ruling says it in one sentence. **One afternoon of reading found a live
bug in the measured pool**, which is the number the original deferral did not
have.

**So: build the ledger, not the verifier — and the unit of the gate is a
linked test, not a disposition** (the owner, 2026-09-08, sharpening this
section): *"card author needs to make a test and link it to the ruling for
someone to review in PR review."* That is a stronger obligation than "record an
answer" and a cheaper one to review, because it puts the ruling and the test
side by side in the diff where a reviewer already is. The mechanism is
`specdb`'s, one level over: a `// RULING:` annotation naming the card and the
ruling's date carries what `// COVERS:` carries for an atom, the ledger holds
the rulings themselves, and `--check` fails on a registered card with a ruling
no test names. "Not expressible" stays a legal answer — it names the missing
facility and its owner — because a gate with no honest escape gets satisfied
dishonestly.

Two things fall out that no amount of care at card-add time gives you: **drift**
— Scryfall adds rulings, so a card correct when registered can acquire one later
that the engine violates, and nothing else in the project would ever notice —
and a **pool-first work queue** for the 87.

**And the reason to build it before the parser, not after.** The other half of
the owner's plan is a static parser, so that a new set gives most of its cards
for free and only the complex or genuinely new effects are hand-authored
(`cost-architecture.md` §6's CP-1 row already leaves that parser a note about
printed symbol order; Phase 8 is where it lands). A parser without this ledger
is a liability rather than a shortcut: it turns card authoring from a
deliberate act, where somebody read the card, into a bulk import where nobody
did — and the *only* per-card evidence Scryfall ships that a machine cannot
fabricate is the rulings. **The ledger is what makes machine-ingested cards
safe to admit**, which reverses the dependency: it is not a nice-to-have
alongside the parser, it is the parser's acceptance test. It also changes its
own economics, because at that point the 145 rulings here are a pilot for
thousands.

**Built 2026-09-17 (A4b).** `plans/check_rulings.py`, `plans/rulings-ledger.json`,
and the check line's fifth script. **The two halves of the sizing above
disagreed with each other and the comparison was the right one**: 640 lines of
Python, not ~250, but `check_state_of_play.py` is itself 487 — so "about
`check_state_of_play.py`" was accurate and the number beside it was not. A
fifth of the file is its docstring, which is where the four decisions below are
spelled out for whoever runs the gate. The data file is 1,082 lines, one line
per ruling so that an added ruling is an added line in `git diff`. Four
decisions the section had left open, and what each was decided on:

- **A `check_*.py` script, not a `specdb` subcommand.** The `CLAUDE.md` line
  budget argued the other way and lost to one fact: **CI deliberately does not
  run `specdb`** (the owner, 2026-08-31 — `plans/` is largely generated and its
  labels are not trusted enough to block a merge), so a subcommand would have
  been a report rather than a gate. Joining the `check_*` family is what makes
  an unlinked ruling fail a pull request, which is what `check_state_of_play.py`
  says in as many words. The budget cost turned out to be zero: the check line
  is one line and takes a fifth `&&`.
- **A ruling is keyed `<Card Name> #<n>`, with `n` stored and never reused.**
  Card plus date was the obvious key and the census refuses it: **298 of the 330
  — 90% — share a date with another ruling on the same card**, and 70 of the 84
  cards carrying more than one carry all of them on a single date. A date
  therefore needs an ordinal, and an ordinal derived
  from position renumbers its neighbours when a ruling is inserted — silently
  re-pointing every annotation below it. `specdb`'s rule for atom ids, for
  `specdb`'s reason.
- **Drift fails rather than reports, and the two are not in tension here.**
  `--check` is offline, so it cannot see a ruling the ledger does not hold;
  drift can only surface in the pull request that runs `--fetch`. The failing
  form is the one that catches it *and* the one that cannot land on somebody
  whose branch touched nothing.
- **The escape is five kinds, not one free-text field**, because each becomes
  untrue a different way and that is the only thing a disposition is for:
  `not-expressible` (names a facility and a `plans/` doc that must exist),
  `no-registered-card` (the engine can state it; nothing reaches the board),
  `defect` (names a `codebase-state.md` item, which must exist),
  `format-variant`, and `no-board` — a ruling about card frames names none and
  never will. A `format-variant` is a disposition rather than a fetch-time
  regex for the same reason: a filter re-decides it on every run, where a
  judgement is made once.

**The gate is scoped, or it is a report.** A card is in scope when someone has
`read` it, or when its `first_seen` stamp is later than the ledger's `created`
— a card registered *after* the ledger existed is one §3.4 already obliged.
Everything else is the backlog: counted, queued, never failed. Same shape as
`owed`'s `SHIPPED_PHASES` and for the same reason (§5.1).

**Scheduled (the owner, 2026-09-08): after replacements, between phases —
`roadmap-v2.md` row A4b.** It gates nothing and nothing gates it, which is the
argument for giving it a slot rather than a "whenever": a task that is nobody's
blocker is deferred indefinitely by default, and this one has already produced
a live bug in the measured pool on its first afternoon (item 82). A4 was also
believed to be the last point at which the retroactive half is a sitting
rather than a project, since the registry only grows — **and the gate's scope
rule turned out to defuse that** (`codebase-state.md` item 151). A card
registered after the ledger is in scope immediately, so it never joins the
backlog; the backlog is frozen at the 308 rulings on the 93 cards that predate
the ledger and only shrinks. The reading can therefore be spread out without
the job getting larger, which the slot's original argument assumed it could
not be.

The two halves stay separable and only one of them is the tool. **The tool** is
the drift detector and the parser's acceptance test, and it is in place. **The
reading** needs no tool and is pool-first — the pool is what every measurement
walks, which is where item 82 came from — and within the pool it goes by
descending ruling count, then by name, which is the order `--queue` prints.

**A4b's sitting read the queue's head**: every pooled card carrying more than
four rulings, which is Cytoshape (8), Culling Drone (7) and Yixlid Jailer (7).
A rule rather than a count, so the next sitting does not have to ask where the
last one stopped. Twenty-two rulings: thirteen answered by a test — ten new in
`tests/rulings_pass_test.rs`, five annotations on tests that already existed —
and nine by a disposition. **The remainder is 308 rulings over 93 cards, 103 of
them on 38 pooled cards**, and it is the ledger's own backlog rather than a
list in this file.

### What the queue's head is made of, and what that predicts

**No defect.** The engine answered all thirteen testable rulings correctly,
which is a different result from item 82's afternoon and the reason is
structural rather than luck: **a card with many rulings is a complex card, and
in this crate a complex card is one some phase was built around.** Culling
Drone is LE's Layer 5 CDA, Yixlid Jailer is LJ's whole point, Cytoshape is what
CV-1 was sized against. Three spot-checks outside the head say the same thing
from the other end — Badlands and Keldon Warlord are pool filler and were
right; Furnace of Rath's tests *quote its rulings verbatim*, because RD was
written from them.

So the head of a count-ordered queue is the best-covered part of it, and the
honest prediction is that **the gap rate rises as the queue drains**. That is
not an argument to reorder it: an "already tested" answer is one annotation and
is exactly what the gate wants, so the cheap cards genuinely are the ones to do
first. It is an argument against reading the first sitting's zero as the
registry's rate.

### 3.5 The input we have none of — a human playing the game

**Noted 2026-09-08, deliberately unscheduled.** Every verification method in
this file is one a machine runs or a person reads: two card pools and a fuzz
harness (§3), the atom corpus and its coverage gate (§5), the rulings pass
(§3.4), trace pages (§7). **None of them is a person playing a game and
noticing something is off**, and that is a distinct source of cases rather than
a convenience — its whole value is that the boards come from someone's Magic
knowledge instead of from a list somebody already wrote.

The evidence that it would pay is this week's. `codebase-state.md` item 82 —
an Equipment that becomes a creature staying attached, in the *measured* pool —
survived 1,063 tests, months of fuzz games and a coverage gate, and was found
by a human reading a card's rulings. A person playing would plausibly have
equipped something, cast March of the Machines, and seen a 2/2 render as 4/2.
Same mechanism, different door: outside knowledge generating a case nobody had
thought to write down.

**What it needs is not obviously a GUI**, and that is the part to establish
before anyone builds one. `cli_play.rs` is 86 lines over a 756-line
`ui/display.rs` that already groups the battlefield into creatures, lands and
other, and formats the hand, the stack, the phase, the mana pool and the event
log. "Hard to visualise" may be a display problem in one small file rather than
a missing application — and a GUI drags in `backlog.md` §2.9's per-viewer
information model, which is a v1 blocker in its own right, plus §2.21. The
cheap first step is to sit with `cli_play` and write down what is actually
unreadable: if the board prints fine but *why anything happened* is invisible,
that is a different fix, and a much smaller one, than "we need a GUI".

Not on the route. `roadmap-v2.md` row E owns the GUI as a v1 deliverable; this
section owns the *reason* to want one early, so that when it is picked up the
motivation on record is verification and not only delivery.

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
- **An arm the PR's own type opens, with a printed customer and sized under
  about eighty lines, ships in that PR** — adopted at RE-4's review
  (2026-09-13, `plans/handoffs/re-4-review.md`, theme B). RE-4 opened
  `TokenDef` and `EventPattern::CreateTokens` and recorded four arms on them
  as ledger lines, each with its customers named and each under the size;
  the review's question was why they were plates to juggle rather than
  commits, and there was no answer. The ledger is for *facilities* that do
  not exist yet, an arm whose type is open in the PR is a normal diff, and
  deferring it costs a second pass over the same code. The band still holds
  — RE-4 landed at the top of it with the arms in — and the exception is an
  arm whose customer needs a facility the PR does not have (Xorn's template
  exists; its Treasure does not), which is a ledger line pointing at that
  facility.
- **The CR is the customer; a printed card is the test** — adopted at RE-5's
  review (2026-09-14, `plans/handoffs/re-5-review.md`, theme A). RE-5 sized
  CR 122.6a's named putter, found no printed effect that specifies one, and
  closed `codebase-state.md` item 43 on that. The owner's rule: a facility
  the CR states is owed whether or not a card prints it — a card can be
  printed next set, and custom card creation is a post-v1 goal, so any
  author can write the rule's first sentence. A rule-stated facility with no
  printed card gets a fixture test and a reachability line that says "no
  printed producer", never "nothing owed"; the band above still decides
  *when* it lands, not *whether*. Bold Plagiarist turned out to print the
  shape anyway, on a proposal rather than an entry.

---

**A shipped phase marks its own heading.** `#### <code> — <what it was> — ✅
landed <date>` in the owning architecture doc, in the commit that ships it.
This was already the habit for six of RC's phases and missing from four;
`check_state_of_play.py` now reads those markers and fails when `CLAUDE.md`'s
critical path still calls a landed phase "next", which is the drift that made
`roadmap-v2.md` §2 unusable. **It only works on the track that has headings** —
the "can't" and copy docs record phases in sizing tables with no status marker,
so normalising those is the next cheap thing anyone touching them can do.

**A landed phase's section is a stub, and the body is in the archive.** Added
2026-09-11, when the replacement doc reached 7,100 lines with 28% of it under ✅
headings and RE's sizing found three answers it needed sitting unread in the
same file (skips said twice, the discard row stale for sixteen days, RD-1's
`Mill` precedent 2,000 lines above the paragraph it contradicted). The design
sections have a budget (`replacement-architecture.md` §0) and it held; the
record had none. The rule is `codebase-state.md`'s eviction rule applied to
the architecture docs: **in the PR that lands a phase, its section's body —
the plan as sized, "as landed", "measured", reachability — moves to
`plans/archive/<doc>-landed.md` under the same heading, and the live doc keeps
the heading, a stub of what shipped and a pointer.** `check_state_of_play.py
--check` fails when a ✅ section keeps more than 40 lines. What stays live is
what the *next* phase needs: the design sections, the sizing of unlanded
phases, and §11-style findings still open. Rulings passes live in the *card's* own doc
comment at registration (§3.4, corrected at RE-3's review); a sizing names the
card and the rulings that become tests, one line each.

### 4.1 The standing review question

**"What does this actually check, and what happens if it runs twice?"**

Ask it of any claim the engine makes about itself. It is written down because
`pipeline::ordering_cannot_change_outcome` — one predicate, whose whole job is
to prove that a CR 616.1 prompt has one outcome — has been **corrected in three
consecutive PRs**, and because of *how* the three were found.

**The three** (`replacement-architecture.md` §11 items 55, 58, 60; item 19 is the
founding argument, not a defect). Two of them mis-stated the premise as a **list
of shapes**: RD-1 wrote the multiplier shape as "the pattern is
`EventPattern::DealDamage`", RE-2 found the draw shape needed a clause that list
could not express, and RE-3 found a `Multiplier` over `EventPattern::GainLife`
falling straight through the damage gate into a prompt the rules do not require.
Each time the axis that moved was `EventPattern`, which grows one arm per
`GameAction` variant every replacement phase — so every new arm landed outside a
premise written before it existed. The third mis-stated it about **one
application**: it argued the event after one rewrite is the same either way and
inferred that the same members are still applicable, when applicability resolves
against each instance's own controller. What was holding that theorem up was
idempotence, which nothing had said.

**How they were found is the argument for a standing question.** Two came out of
review rather than from a test. The third came from a test that asserted *the
absence of a prompt* — and that kind of test only exists because the review
before it had established that it should. None of the three is a shape CI finds
on its own: a suppression that is wrong produces a question nobody needed, or an
answer that is right for a reason that will stop being true.

So the question has two halves, and both earn their place:

1. **What does this actually check?** Name the values being compared and their
   types, in a sentence. *"`Rewrite: PartialEq` compares def data a card file
   wrote; the release predicate reads no board at all; the debug check compares
   one `GameAction` to one `GameAction`"* is the sentence that exposed item 60 —
   the hole was invisible until someone had to write down what was being
   compared. If the answer seems obvious, write it anyway: the writing is the
   check.
2. **What happens if it runs twice?** Any predicate that suppresses, memoizes or
   short-circuits inside a loop has an iteration count it is not controlling. A
   premise that argues one application has *assumed* something about repeating
   it; make it say which.

**When to ask it — a list, so it is a gate and not a mood.** Any change to
`ordering_cannot_change_outcome` or its leaf tables; any new `EventPattern`,
`Rewrite` or `GameActionTemplate` arm; any `debug_assert` standing in for a
proof; and any "this is safe because …" in a doc comment where the *because* is
a list of cases rather than a property.

**The cheap tell, if you only remember one thing:** a premise that enumerates
shapes will be wrong when the shape list grows, and the shape list always grows.
Prefer a property on the type whose arms grow — `EventPattern::reads_the_amount`
rather than `matches!(pattern, DealDamage { .. })` — and put it beside the enum,
so the question is in front of whoever writes the next arm rather than buried in
a predicate they have no reason to open. The contrast is
`pipeline::filter_is_mods_invariant`, which stays at its caller because it
relates *two* types and so is a fact about neither.

**A reviewer does not need to know the subsystem to ask this.** All three were
caught by asking about the *form* of the claim, not about Magic — which is why
it belongs here rather than in a rules doc, and why it is worth asking even when
the answer turns out to be "checked, and it holds".

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


### 5.1 What `owed` does not cover, and what closed RC instead

**`owed`'s default scope is `SHIPPED_PHASES`** — `ALREADY-IMPL`, `Phase 5-Pre`,
`Phase 5-Layers` — and the constant's own comment says why: an atom parked in a
phase that has *not* landed is being waited on, so counting it would make the
query a report rather than a gate. Correct, and it has a consequence nobody had
written down: **Phase 6 is not in that list, so no replacement phase has ever
been gated by `owed`.** Every RC exit line that says "`specdb owed` unchanged"
is a true statement about three *other* phases. What actually held RC's spec
coverage to account was the `// COVERS:` discipline above, plus `orphans` and
`suspicious`, and those are per-PR habits rather than a gate.

**Two things follow.** Add `Phase 6` to `SHIPPED_PHASES` when RE lands, which is
what turns the habit into a gate for the whole replacement track — and until
then, a replacement phase that wants the real number runs
`specdb owed --phase "Phase 6"` and reads it as a to-do list for RD and RE (54
atoms as of 2026-09-03, nearly all damage and prevention). **Done 2026-09-15,
at the post-RE audit's close-out**: the query was run once by hand against
the finished track (127 atoms, 50 uncovered, 21 of them ticketed `NEW`), every
uncovered atom was given a line — annotated, tested, or re-filed with its
owner — and Phase 6 joined the constant. The triage is `backlog.md` §3.3's
second application block; from here the gate is armed for the track, and a
phase that adds Phase 6 atoms closes against it like any other.

**And the seam neither instrument covers:** a defect in shipped behaviour that
has no atom is invisible to both `owed` and Deferred Migrations. RC-5's item 61
is one — the card ruling it violates was never written into the corpus, so no
query could have asked for it. That is what the review pass is for, and it is
the argument for `plans/state-of-play.md` rendering the contrast rather than
letting the two words blur.

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

Three existed when the practice was written down, and they are the template:

| Page | Phase | What it proves |
|---|---|---|
| `rc-4b-entering-is-one-event.html` | RC-4b | four entries through the CR 614.12 look-ahead frame, each read labelled board or frame, and what RC-4b changed trace by trace |
| `cv-1-a-copy-is-a-snapshot.html` | CV-1 | a Cytoshape resolution from the choice to the copy row and back, and CR 707.4's re-copy tearing that row down through the existence check |
| `rc-5-applying-an-entry-can-move-the-board.html` | RC-5 | devour's selection and its nested batch, the zone chain that makes CR 614.13b bite, `frame_of(source)` and §5b's asymmetry, and two entries decided against one board |
| `item-7-an-effect-waits-for-what-it-reads.html` | LI-2 + LI-3, closing item 7 | the CR 613.8 loop's fast and slow paths; the judge answer's four-card layer 4 step by step through `next_ready` and the journal; a condition that *is* a dependency (Simian Clause under Blood Moon, with the sabotage step that shows what `condition_reads` buys) and one that is not (Kird Ape, two layers apart); the read-by-read table |
| `li-1-one-pass-per-board.html` | LI-1 | the `Board` struct field by field, the entry's three routes, and Humility + Citanul Hierophants through the old walk and the pass — the one read that produced the wrong answer, and where it reads from now; a look-ahead entry; a graveyard Keldon Warlord |
| `re-2-a-draw-carries-its-lineage.html` | RE-2 | the first decomposed event: two Thought Reflections through the applied set that travels with it, and the same board without it — a stack overflow at depth two rather than a wrong number; Teferi's exception living in the shape of the event tree instead of a counter; **Alms Collector in both encodings**, the shipped one and the one §9 sized, which is a CR 104.4b loop; three Notion Thieves moving the event's subject and CR 616.1's chooser with it; the read-by-read table |
| `rd-2-a-decision-is-per-subject.html` | RD-2 | the CR 616.1 loop's new unit: two shield counters under two blockers through the per-member loop and the per-subject one, and the first-strike twin that shows the key is the batch; Furnace beside Mending Hands in both orders; a `NextDamage(3)` under sources of 2 and 4 with the allocation asked once; the two boards where nothing is consumed — Safe Passage beside Mending Hands, and a `Once` half chosen against 1 — and the consume-after-apply order that makes them right |
| `rf-a-source-off-the-battlefield.html` | RF | the gather's zone leg read by read: a Colossus in a library while a Bolt resolves, which is why neither library is ever walked (the map, the printed-def precheck, and the two things that *would* walk a zone); the same Colossus second of three in a mill — one batch, one member replaced, the same-zone no-op, the rider after the batch; a Colossus commander sacrificed, which is CR 616.1 twice on one card and CR 701.24c's shuffle of a library the card never reached; the read-by-read table |

**When to write one: at phase close, for a phase that changes *how* a read is
answered rather than what the answer is.** That is the property the two above
share, and it is why a phase that adds a card, an enum arm or a pool entry does
not get one. The phases that qualify were listed when the practice started:
RC-4 ✓, RC-4b ✓, CV-1 ✓, RC-5 ✓, item 7 ✓ (twice — LI-1 mid-phase, because the
pass changed every read at once, and the close), RD-2 ✓ (the loop's unit; the
one RD phase that qualifies, decided at its close as §9 scheduled), RE-2 ✓ (the
applied set answered for a decomposed event; one of the two RE phases §9 named,
decided at its close), RE-4 ✗ (decided *no* at its close, 2026-09-13: the
read it was named for was RC-5's page's already, and what it changed is what
is proposed — the archive's "Trace-page decisions" has the argument), RF ✓
(decided *no* at its close on the same test and reversed at its review the
same day, 2026-09-16: the owner could not see why the gather's zone leg walks
no library the moment a Colossus is in one, which is this section's own
trigger — a question the diff could not answer),
**RS-2, critical-path item 6**. Budget
two to three hours; that is the right cost for a phase's close and the wrong
cost for a question asked mid-debugging, which is what tier 2 below is for.

**Its examples are the phase's findings, not its feature list.** A page that
walks the happy path explains the feature; a page that walks the board the
review argued about explains the phase.

**And a page may walk a design the phase rejected** — RE-2's does, side by side
with the one that shipped. That is not a second feature list: the two Alms
Collector encodings differ by a printed ruling and by whether the game
terminates, and the *test* for it can only assert that the loop does not happen.
Where a phase's finding is "this other reading is wrong", the page is the only
artifact that can show why.

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
  default and gated the way `Diagnostics` is, recording what tier 1 records
  by hand: each proposal entering a batch, each pipeline iteration, each
  top-level layer walk, and the performed events. JSON lines, plus a script
  that turns one into a page in this format, so tier 1 becomes generated.
  **Landed 2026-09-18 (row A4c, PR #170)** as `mtgsim/src/state/trace.rs`: five
  emit points — the batch, the CR 616.1 iteration, the layer walk, the
  decision boundary (A4h's addition: the prompt, and the rejection between a
  prompt and its re-ask) and the performed event — off at one branch each,
  reachable as `fuzz_games --trace DIR`, `cli_play --trace PATH` and
  `test_support::install_trace`. `plans/trace_spine.py` renders one game's
  lines through `plans/traces/viewer.html`, which also reads a file dropped
  on it. **Tier 1 is not subsumed.** A sink generates a page's *spine* — the
  step rows, each read labelled by what it consulted — and never its
  argument: which boards, which question each answers, the "where the reads
  differ" table and the closing section are authored, and a generated page
  marks their places. What it is for is the question asked mid-debugging,
  and the regeneration of a page's rows after a refactor: rd-2's Trace A
  regenerated from its test found two rows the pinned page states and
  today's engine does not (row A4c has them), which is the diff a spine
  exists to show and not a reason to touch the page. The two corrections
  item 5 recorded before the code held: "gated the way `Diagnostics` is"
  became one `Option` branch per emit point with the payload built behind
  it, and the check was `IDENTICAL` on every counter with the sink compiled
  in and off, and on (`fuzz-record.md`, the A4c block).
- **Tier 3 — the codebase map.** One structural page: the modules and what each
  owns, the chokepoint's arms, the three gate legs a new replacement source
  must extend, the two `object_matches_filter`s, the accessor pair, and the
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

## 8. The rules pass — read the rules that name the *rule*, not just the site

**Before deferring a rules question, search the CR for the rule that watches
the one you are implementing.** The census habit this project runs — read the
tree, then read the rule that names each call site — finds what an event *does*
and is blind to what *observes* it, because the observer is written somewhere
else entirely.

RE-7 is the scar. CR 800.4a says what becomes of a departing player's
permanents, and it says nothing about triggers; so "does a leaves-the-
battlefield ability fire when a permanent leaves the *game*?" was recorded as
an open question, with `GameEvent::LeftTheGame` shipped without the CR 603.10a
frame a matcher would need. **CR 603.6c answers it in the sentence that names
the event** — "leaves-the-battlefield abilities trigger when a permanent moves
from the battlefield to another zone, **or when a phased-in permanent leaves
the game because its owner leaves the game**" — and the search that would have
found it is one `grep` for the rule number of the *other* side.

**The habit, three greps and about two minutes:**

1. `grep` the chapter that owns the thing you are changing (800.4 for a
   departure) — this is the census's own step, and it is the one that is never
   skipped.
2. `grep` the chapter that owns the thing that *watches* it. For an event
   that is CR 603 (triggers) and CR 614 (replacement); for a characteristic,
   CR 613; for a zone change, CR 400.7 and CR 603.10.
3. `grep` the *name* of what you are building, in the CR's own words, across
   the whole file. "leaves the game" appears in 603.6c, and 603.6c is nowhere
   near 800.4.

**What makes this worth a section rather than a comment**: the failure is
silent and it is *shaped like progress*. A deferral with a reachability line
and a size reads exactly like a deferral that was researched, and the ledger
cannot tell them apart — `codebase-state.md`'s rule catches an entry longer
than its fix, not an entry whose premise was never checked. The counter-example
is `replacement-architecture.md` §11 item 64, where the same search *was* run
and CR 604.2 plus CR 614.4 closed a design question two rules away from the one
being implemented; the difference between the two was a few minutes of reading,
not a difference in difficulty.

**And it applies to the corpus, not only to code.** `ATOM-800.4c-001`'s board
could not reach its own rule — it had the creature's owner leave, which
CR 800.4a's first clause forbids — and that is the same failure from the
authoring side: a scenario written against one rule without the rule that
overrides it. → `replacement-architecture.md` §11 items 70 and 73.

---

## 9. The spine-phase audit — a close is audited, not declared

**When a critical-path item closes, audit it before the next one starts.**
Decided by the owner at the post-RE audit (2026-09-15): the audit recurs at
each spine-phase close — the next at critical-path item 6's — as a set of
passes with named instruments, each pass its own PR, planned in a
`plans/handoffs/` file that the last pass's PR deletes. Findings land where the
project already records them: a numbered item under a dated `### Found by …`
heading in `codebase-state.md`'s Deferred Migrations, a `backlog.md` §2 entry
for a mechanic with no surface, a stale claim rewritten in place, a rule here.
The run's own record is a `### … — audited YYYY-MM-DD` heading in
`codebase-state.md`, pointers not prose, written when the handoff goes.

**Why a practice and not a checklist in the closing PR.** The first run found
that "done" was defined in five places and collected in none — the doc's
findings list, its known-missing table, its out-of-scope list, the CR row and
the `owed` query — and that `specdb owed` had never been run for the phase at
its close, because `SHIPPED_PHASES` did not contain it. It found the §2.1
sweep overdue by eighty PRs against a fifteen-to-twenty cadence, no throughput
target stated anywhere in `plans/`, and the performance levers ranked by
argument rather than by a profile. None of those is a bug a test catches;
each is a claim about the tree that had gone stale without anything re-reading
it, which is §8's failure shaped like progress. A closing PR is the wrong
place to look for them, because the person closing the phase is the one who
believes the claims. Reading, measuring and dispositioning is its own sitting
with its own instruments, and it runs at the one moment the material is fresh
and the registry is still small enough for the retroactive halves to be
sittings rather than projects.

**The passes, each with its instrument.** In this order: the close-out gates
the rest, the eviction opens second, hygiene and readiness commute, scheduling
needs the close-out and readiness.

1. **The close-out.** The phase's done-checklist read off the tree and every
   entry dispositioned — closed, scheduled with an owner, or deferred with a
   dated reachability line and a size. Instruments: **the board**
   (`check_state_of_play.py --write` then `--check`; its two bolded rows are
   the work) and **`specdb owed --phase "<the phase>"`** once `SHIPPED_PHASES`
   gains it — a gate, so the phase is not closed until it is clean (§5); the
   doc's own findings list, its known-missing table and its out-of-scope list
   re-read against what landed since; the summaries — `codebase-state.md`'s
   TL;DR and the CR row — rewritten in place, never appended to. Produces the
   retrospective in `replacement-architecture.md` §14's shape: pointers, under
   150 lines, written once and not maintained. A fix found here is shown to
   fail first; one that moves a pool is its own PR with a `fuzz-record.md`
   block.
2. **The eviction.** The phase doc's pre-build reasoning and closed findings
   move to `plans/archive/<doc>-landed.md` under **the ≤40-line stub rule**,
   planned section by section with line counts before anything moves: every
   heading byte-identical so `grep` still lands, every closed item leaving its
   `N. **title**` line so every outside citation resolves, every outside
   citation of a moved section read against its stub, `check_glossary.py` on
   the result. Opens second, after the close-out merges, so the stubs point at
   a retrospective that exists on `main`.
3. **Hygiene and CI.** **§2.1's two grep tiers** — the wide one is the count
   to record, the tight one the list to read — plus the history-word block
   list over every touched file (§2.1's second record, which is the
   instrument that actually cut lines); every `TODO` given a live owner or
   deleted; **clippy counted before it is decided** (`cargo clippy
   --all-targets`, the count deciding which lints go to `-D` and which are
   allowed with a reason each); the toolchain pin measured; the four checks
   in `ci.yml`, each with a comment giving its reason.
4. **Readiness.** Is the engine on track for the harness use case? **Its first
   duty is the ratchet's re-reading** (§3.1): decisions per core-second on
   both boards at four seats and one thread, recorded as a dated reading
   beside the previous close's, and it may not read worse per decision without
   a written reason. Instruments: **the census provider** (item 138's two
   counters once they land; a throwaway counting provider around
   `fuzz_games`' own decks and streams until then), **the clone timer** with a
   counting allocator (item 143's table, extended at each close), **the fork
   record-and-replay** (item 41's test — record every answer, clone at a
   round start, replay, compare the logs verbatim; the id mask retired with
   A4g on 2026-09-16, since a fork's ids are its parent's), **`plans/panic_surface.py`**
   (the engine's release-active surface, separated from the unit-test tails;
   a surface growing faster than the engine is the finding), **the three-run
   contention read** (`--threads 1`, 8 and 16 at 200 games, item 138's lever
   7) and **callgrind under WSL** (`layers-architecture.md` §12's instrument
   paragraph: `valgrind --tool=callgrind` over `fuzz_games` at a fixed seed,
   `callgrind_annotate --inclusive=yes`; the instruction count is the reading
   that travels between machines, the milliseconds stay in the A/B sitting).
   Produces the lever list re-ranked by instruction share, each lever sized in
   its item.
5. **Scheduling.** **One table**: each open track phase and lattice entry, what
   it unlocks in cards (the ledger and the docs' consumer lists — no Scryfall
   count re-derived) and in atoms (`spec.sqlite` by rule prefix, the prefixes
   named), what it needs, and which of the next spine phase's design questions
   needs it landed first; then a proposed order for the between-phases slot.
   The pass proposes; the owner decides. The first instance is `roadmap-v2.md` §3a's A table — the between-phases rows it gained on 2026-09-15, which §3b explains.

**Binding rules across passes.** Docs and small fixes only — anything larger
becomes a scheduled item with an owner. No count is re-derived that a doc
already carries with a date, and a number in a heading is a number that will
be wrong. A pass re-reads what it touches: every number in a pass's brief is
a starting point, and two of the first run's were wrong (§11 held 99
findings, not 108; clippy read 114 sites, not "hundreds"), which is the reason
the brief says so about itself.

**The first instance — the post-RE audit, 2026-09-15**, at critical-path item
5's close, the day RE-9 merged. Planned as `plans/handoffs/post-re-audit.md`
(PR #141), which pass 4's PR #154 deleted; its last text is `git show
341ebf9:plans/handoffs/post-re-audit.md`, and the record is
`codebase-state.md`, "Was critical-path item 5 done, and what sits before item
6? — audited 2026-09-15". Where each pass's output lives:

- **Close-out (PR #142):** the board's classifier corrected and Phase 6 armed
  in `SHIPPED_PHASES`; the Phase 6 `owed` triage, `backlog.md` §3.3's second
  block; `replacement-architecture.md` §8a, §11 items 3, 4 and 14, §12, §13
  and §14; `codebase-state.md`'s TL;DR, its CR 614–616 row, and "Found by the
  post-RE audit (2026-09-15)" — item 134, and items 59, 60, 122 and 131
  scheduled; item 118 fixed in it.
- **Eviction (PR #151):** `replacement-architecture.md` 6,725 → 3,401 lines,
  the archive 4,933 → 8,647, planned section by section in the handoff and
  re-counted against the tree in the same block.
- **Hygiene and CI (PRs #143, #144 re-landed by #145, then #146–#149):**
  clippy adopted at `-D warnings` with five lints allowed, `check_glossary.py`
  in CI, the floor measured at 1.88 and recorded as `rust-version`; §2.1's
  two recorded applications, the second over every non-card source file;
  twelve `TODO`s re-owned, one new owner filed (`backlog.md` §2.32).
- **Readiness (PR #153):** `codebase-state.md` items 138–143; §3.1's budget
  and ratchet; `layers-architecture.md` §12's callgrind subsection;
  `backlog.md` §2.22's ask table; `plans/panic_surface.py`; item 41 promoted
  to a requirement and item 69 closed.
- **Scheduling (PR #154):** `roadmap-v2.md` §3a's A table (the `A4x` rows) and §3b, this section, and the audited heading
  above; the two after-the-passes artifacts — the codebase map (§7.1's tier 3)
  and the Rust notes — homed at `roadmap-v2.md` row A4d.

**What the next run adds, at critical-path item 6's close.** The ratchet's
second reading against item 138's first, read through the counters rather
than a probe; `plans/panic_surface.py` re-run; the trace-page decision
recorded on the phase heading (§7); `check_glossary.py --suggest` triaged,
which is the glossary's own obligation at a phase close; `roadmap-v2.md` §3b's
table re-derived, since every count in it carries the date it was read; and
A4d's two artifacts, if still open, because the trigger phase is the largest
new subsystem the map would have to absorb.
