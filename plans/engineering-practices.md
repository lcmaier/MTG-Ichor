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

**What stays out of `CLAUDE.md`:** a comment-length rule. That file is 200
lines and every section costs another; the rule it already has is the right
one, and a second rule that duplicates it in the negative would buy nothing
that this sweep does not.

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

**Re-recorded 2026-09-06 for LI-3** (conditional statics;
`layers-architecture.md` §13b). One new card in `performance` — Kird Ape,
69 → 70 — and none in `stress`, and no new row. **Every movement below is
the pool's**: with the registry and pools unchanged, LI-3's engine
reproduces LI-2's table byte for byte on both pools, at 50 games and at
200 (the middle arm of the three-arm table below), because no *registered*
card's static ability had a condition before Kird Ape and the new branch
in the existence check therefore never ran.

| | performance (70 cards) | stress (81 cards) |
|---|---|---|
| P0 / P1 | 28 (56.0%) / 22 (44.0%) | 27 (54.0%) / 23 (46.0%) |
| Avg turns | 29.3 | 30.3 |
| Spells cast | 22.9 | 23.2 |
| Lands played | 17.7 | 17.7 |
| Combat w/ atk | 10.0 | 9.8 |
| Creatures died | 6.8 | 5.1 |
| Damage events | 22.5 | 22.3 |
| Total damage | 57.2 | 57.8 |
| Life changes | 15.1 | 14.5 |
| **Layer walks** | **321** | **361** |
| **Board walks** | **232** | **269** |
| **Memo hits** | **87,860** | **102,836** |
| **Layer frames** | **4,074** | **4,975** |
| **Frames/walk** | **12.70** | **13.79** |
| **Dependency checks** | **21** | **44** |
| **Replacement gathers** | **476** | **522** |
| **Restriction queries** | **479** | **524** |

**Re-recorded 2026-09-07 for CM-1** (cost modification; `cost-architecture.md`).
One new card in `performance` — Thalia, Guardian of Thraben, 70 → 71 — and
three in `stress` (Thalia, Goblin Electromancer, Trinisphere; 83 → 86). **Every
movement below is the pool's, and this time that was checked by construction
rather than argued**: a third arm, the CM-1 engine with the three cards
registered but the *old* `PERFORMANCE_POOL`, reproduces `main`'s 200-game
`performance` run byte for byte outside the timing block (`fuzz_ab.py`:
`unpooled vs main outside Timing: IDENTICAL`), because a cost pipeline gated on
an empty source set is not there; on `stress` that arm and the pooled arm are
identical to each other, so pooling Thalia moved `performance` and nothing else.
Timing, 200 games, three interleaved rounds: CPU/game 15.67 → 15.70 ms
(+0.2%), ms per 1,000 walks 46.2 → 43.0, deterministic in all three arms.

Reachability, 200 games (`--require`, which since this phase re-joins a card
name that contains its own separator — the flag split "Thalia, Guardian of
Thraben" into two names nobody had registered): Thalia cast 210, resolved 210,
in 131 `performance` games (66%), 1.66 copies per deck; with Humility forced
beside her, Thalia in 138 games (69%) and Humility in 105 (52%), which is the
stripped-source path in a measured game. `stress`, Goblin Electromancer and
Trinisphere forced: 180 cast / 178 resolved in 126 games (63%) and 193 / 191 in
131 games (66%), ~1.5 copies per deck each.

| | performance (71 cards) | stress (86 cards) |
|---|---|---|
| P0 / P1 | 32 (64.0%) / 18 (36.0%) | 26 (52.0%) / 24 (48.0%) |
| Avg turns | 29.3 | 31.4 |
| Spells cast | 22.1 | 23.7 |
| Lands played | 17.3 | 18.4 |
| Combat w/ atk | 10.4 | 11.1 |
| Creatures died | 6.4 | 5.6 |
| Damage events | 21.4 | 25.3 |
| Total damage | 56.1 | 67.4 |
| Life changes | 14.7 | 15.8 |
| **Layer walks** | **363** | **467** |
| **Board walks** | **235** | **270** |
| **Memo hits** | **91,298** | **105,142** |
| **Layer frames** | **4,253** | **5,021** |
| **Frames/walk** | **11.70** | **10.76** |
| **Dependency checks** | **14** | **114** |
| **Replacement gathers** | **479** | **549** |
| **Restriction queries** | **482** | **552** |

**Re-recorded 2026-09-07 for CM-2** (the spell's own cost abilities;
`cost-architecture.md`). One new card in `performance` — Myr Enforcer, 71 → 72
— and two in `stress` (Myr Enforcer and Frogmite; 86 → 88). The middle arm —
CM-2's engine, both cards registered, the *old* `PERFORMANCE_POOL` — is
`IDENTICAL` to `main` on `performance` outside the timing block, so every
movement below is the pool's.

**It was not identical on the first run, and that is the whole reason the arm
exists.** +5 layer walks and +5 layer frames per 200 games, with every gameplay
counter unchanged — five non-member frames computed and thrown away. The cause:
source 2's gate asked "does this card print a cost ability", which is true of a
**Thalia in hand**, so her frame was computed at every castability preview and
then refused because her subject is other spells. The gate now asks the
subject. Nobody would have found five walks by reading the diff, and nobody
needed to.

Timing, 200 games, three interleaved rounds: CPU/game 16.63 ms `main` → 16.50
ms registered (−0.8%, inside the sitting's ~2–6% spread) → 15.01 ms pooled;
`deterministic` yes in all three arms, and three shell runs at one seed match
line for line on both pools. **The pooled column is a re-record, not a
speed-up**: never A/B a number across a pool change.

Reachability, 200 games: Myr Enforcer cast 167, resolved 166, in 113
`performance` games (56%), **1.74 copies per deck** — so the board where one
Enforcer counts the one already on the battlefield is routine rather than
contrived. `stress` with Trinisphere forced beside it: 158 / 157 in 108 games
(54%) and 193 / 192 in 136 games (68%), ~1.5 copies per deck each, which is
affinity's reduction and a direct-total effect meeting on one spell.

| | performance (72 cards) | stress (88 cards) |
|---|---|---|
| P0 / P1 | 27 (54.0%) / 23 (46.0%) | 24 (48.0%) / 26 (52.0%) |
| Avg turns | 28.4 | 30.0 |
| Spells cast | 21.5 | 22.9 |
| Lands played | 17.3 | 17.7 |
| Combat w/ atk | 9.3 | 10.5 |
| Creatures died | 6.3 | 4.2 |
| Damage events | 20.3 | 23.7 |
| Total damage | 47.6 | 59.6 |
| Life changes | 13.5 | 15.2 |
| **Layer walks** | **352** | **433** |
| **Board walks** | **220** | **254** |
| **Memo hits** | **80,262** | **99,787** |
| **Layer frames** | **3,602** | **4,734** |
| **Frames/walk** | **10.24** | **10.94** |
| **Dependency checks** | **11** | **73** |
| **Replacement gathers** | **449** | **527** |
| **Restriction queries** | **451** | **530** |

**Re-recorded 2026-09-08 for CM-3** (lock-in's payment side;
`cost-architecture.md`). One new card in `performance` — **Bone Splinters**,
72 → 73 — and six in `stress` (Bone Splinters, Altar's Reap, Thunderscape
Familiar, Krark-Clan Ironworks, Foundry Inspector, Mind Stone; 88 → 94).

**`cost-architecture.md` §6 said CM-3 "opens no new path a pooled card would
measure". The A/B says the first half and disproves the second.** The middle
arm — CM-3's engine with all five cards registered and the *old* pool — is
`IDENTICAL` to `main` on `performance` at 200 games, so the payment order,
the plan/pay split, the mandatory-cost announcement and the castability gate
cost the pool nothing and change no seeded stream. But `Cost::Sacrifice`
*is* a new engine path, and no card in the 72 could reach it: that is the
`PERFORMANCE_POOL` doc's own failure mode, "a gated subsystem no card in the
pool could open", and its rule is that a phase which opens a path adds one
card deliberately. So the pool gains one and §6's second clause was wrong.

The pooled card opens four things at once — the first non-mana cost paid
through `pay_costs`, the first mandatory additional cost (CR 118.8b), the
first castability answer that turns on something other than the mana cost,
and the first payment prompt that is not an allocation. The other five stay
registered and out: the Familiar and the Inspector open the path Thalia
already opens, and the Ironworks pair's window is CM-4's to measure once the
window stops closing early.

**Which card is a separate question from which path, and it was measured.**
Altar's Reap was pooled first, because CR 601.2h's example is written on it.
It costs **+20.2%** CPU/game, and it draws two cards, so the games it makes
are bigger. Bone Splinters opens the identical set of paths, sacrifices the
same way, and destroys a creature instead of drawing two: **+13.8%** in the
same sitting, +11.9% in a second. Altar's Reap stays registered — its tests
are the rule's — and the pool carries Bone Splinters.

### 3.1a What a pooled card costs is mostly the *slot*, measured 2026-09-08

The obvious reading of the paragraph above is that Altar's Reap's card draw is
the cost and a leaner card avoids it. **Half right, and the other half is the
more useful number.** A third arm settles it: `Cobbled Wings` — already
registered, opening *no new engine path at all*, since Bonesplitter is pooled
and equip is the same code — as the 73rd card instead.

| 73rd card | opens a new path | CPU/game vs `main` | ms / 1,000 queries |
|---|---|---:|---:|
| *(none — 72)* | — | +0.0% | 0.154 |
| Cobbled Wings (inert control) | no | **+14.0%** | 0.165 |
| Bone Splinters | yes | **+11.9%** | 0.158 |
| Altar's Reap | yes | +20.2% (prior sitting) | 0.161 |

**A card that does nothing new costs as much as Bone Splinters does.** So
~12–14% is what *a pool slot* costs, not what a mechanic costs: a 73rd
playable card changes deck composition, boards get bigger, and the layer walk
covers more objects per walk (frames 3,998 → 4,542 for the inert control,
which introduces no rows of its own). Only the surcharge above that line is
attributable to a card, and Altar's Reap's ~6 points is one; Bone Splinters is
indistinguishable from the control.

Three things follow, and they are what to quote the next time this comes up:

- **Choosing a leaner card is worth doing and worth about 6 points.** Between
  two cards that open the same path, take the cheaper board. Past that there is
  nothing to optimise: the slot is the cost.
- **This is not an engine regression and no engine work removes it.**
  ms/1,000 queries — the cost of a unit of work rather than of a game — moves
  2.8% for Bone Splinters and 7.2% for a card with no mechanic, both inside
  the sitting's 2–6% spread. The games got bigger; nothing got slower. Every
  CM phase's middle arm has been byte-identical to `main`.
- **The growth is the price of representativeness, and it is bounded by how
  often a phase opens a genuinely new path** — three times across CM-1, CM-2
  and CM-3. A pool that stopped growing would go back to measuring a shrinking
  fraction of the engine, which is the failure the freeze was lifted for.

**And the A/B sitting is not the development bottleneck it feels like**:
three arms, both pools, counters, fixture rows and three interleaved timing
rounds is **32 seconds** of wall clock (2026-09-08). What costs minutes is
building one release binary per arm and orchestrating them, which is
`--rounds`-independent. If a sitting ever does need to be cheaper, `--rounds 2`
or `--games 100` halves the timing block at the price of a wider spread; that
knob is there and has not been needed.

Reachability, 200 games with Bone Splinters forced into every `performance`
deck: cast 193, resolved 110, in 84 games (42%), **1.58 copies per deck**.
Casts and not resolutions is the number that matters here — the sacrifice is
paid at CR 601.2h whether or not the spell later resolves — and 193 against
Altar's Reap's 223 is 87% of the payment-path exercise for two-thirds of the
cost. Zero errors, zero panics and zero `Uncast resolved` in every arm on both
pools, which is the statement that matters most for a new payment arm.

| | performance (73 cards) | stress (94 cards) |
|---|---|---|
| P0 / P1 | 28 (56.0%) / 22 (44.0%) | 26 (52.0%) / 24 (48.0%) |
| Avg turns | 28.9 | 28.1 |
| Spells cast | 22.3 | 22.5 |
| Lands played | 17.5 | 16.9 |
| Combat w/ atk | 9.9 | 10.1 |
| Creatures died | 6.6 | 4.5 |
| Damage events | 21.7 | 22.7 |
| Total damage | 59.4 | 58.8 |
| Life changes | 14.3 | 14.8 |
| **Layer walks** | **371** | **431** |
| **Board walks** | **235** | **248** |
| **Memo hits** | **88,901** | **89,949** |
| **Layer frames** | **4,265** | **4,445** |
| **Frames/walk** | **11.49** | **10.32** |
| **Dependency checks** | **10** | **38** |
| **Replacement gathers** | **475** | **481** |
| **Restriction queries** | **477** | **483** |

**Re-recorded 2026-09-08 for the CR 704.5p fix** (`codebase-state.md` item 82).
No card and no pool change — a bug fix and, riding with it, the largest engine
speed-up the project has measured. **Every row that moved is the engine's.**

The bug is one sentence of CR 704.5p that was never implemented (an Equipment
that *becomes* a creature stayed attached). The speed-up is unrelated to it and
was found while fixing it: both attachment sweeps asked their `has_subtype`
questions *before* reading `attached_to`, so three characteristics frames were
computed for every permanent on the battlefield, every state-based-action
check, to answer a question about the handful that were attached. Reading the
field first is the whole change.

| | main | fixed |
|---|---:|---:|
| Memo hits, `performance` (200 games) | 99,530 | **62,215** (−37.5%) |
| Memo hits, `stress` | 97,783 | **61,424** (−37.2%) |
| CPU/game median (200 games, ×3) | 16.09 ms | **14.29 ms** (−11.2%) |
| ms / 1,000 walks | 42.57 | 37.70 (−11.4%) |

**Re-recorded 2026-09-08 for RD-1** — `PERFORMANCE_POOL` +1 (Furnace of Rath,
73 → 74) and the stress pool +5. Every row moved, and that is what a damage
doubler in every red deck does: games end sooner (avg turns 28.9 → 30.0 on
`performance` is the *other* direction and is deck-mix noise at 50 games; the
200-game run has 31.0 → 29.7), total damage per game rises 59.4 → 63.8, and
CPU/game falls **6.4%** because there is less game to play.

| | performance (74 cards) | stress (98 cards) |
|---|---|---|
| P0 / P1 | 27 (54.0%) / 23 (46.0%) | 20 (40.0%) / 30 (60.0%) |
| Avg turns | 30.0 | 27.2 |
| Spells cast | 23.5 | 21.0 |
| Lands played | 17.9 | 16.7 |
| Combat w/ atk | 9.7 | 9.2 |
| Creatures died | 7.0 | 4.1 |
| Damage events | 20.8 | 19.8 |
| Total damage | 63.8 | 56.3 |
| Life changes | 14.4 | 13.3 |
| **Layer walks** | **372** | **448** |
| **Board walks** | **245** | **245** |
| **Memo hits** | **60,807** | **56,268** |
| **Layer frames** | **4,602** | **4,374** |
| **Frames/walk** | **12.36** | **9.77** |
| **Dependency checks** | **22** | **18** |
| **Replacement gathers** | **518** | **480** |
| **Restriction queries** | **520** | **483** |

The `stress` column moved by a hair between the first recording and this one
(449 → 448 walks, 4,414 → 4,374 frames) — the review's `Primitive::Mill` fix,
which turns a mill of N into one batch instead of N and so bumps the layer
epoch once rather than N times. Nothing a game can see changes;
`performance` is untouched because Angel of Suffering is not in it.

Reachability, 200 `stress` games with Loyalty Probe forced into every deck:
cast 206, resolved 204, **in 133 games (66%)**, 1.49 copies per deck. It is
`{2}` and colorless, which is why the number is that high and why the fixture
is worth registering: **CR 704.5i fires 4 times in 400 unforced `stress`
games** — three Probes bolted to zero, and one Merfolk Thaumaturgist that
Cytoshape turned into a copy of a Probe and which died on the spot, because
CR 707.2 does not copy counters. That state-based action had measured 0 at
every game count since it was written.

**Re-recorded 2026-09-09 for RD-2** — `PERFORMANCE_POOL` +1 (Mending Hands,
74 → 75) and the stress pool +4 (98 → 102). The table gains a row,
**Prevention allocations**, which `plans/fuzz_ab.py` prints from here on: CR
615.7's allocation prompts per game — a *reachability* count and not a cost,
which is why it is not bold, and `?` on any binary older than RD-2. Every
other movement below is the pool's: with the registry and pools unchanged,
RD-2's engine (the `loop` arm) reproduces RD-1's 50-game `performance` table
to the digit — 372 walks, 4,602 frames, 518 gathers — and the item-29
suppression moves it by one game's worth (373 / 4,612 / 519); on `stress`
both differ by a hair in the two-Furnace games (448 → 446 → 448 walks). The
shipped columns are a different game set, because a one-mana instant in every
white deck reshuffles every deck; read them as RD-1's were read.

| | performance (75 cards) | stress (102 cards) |
|---|---|---|
| P0 / P1 | 26 (52.0%) / 24 (48.0%) | 26 (52.0%) / 24 (48.0%) |
| Avg turns | 32.3 | 26.8 |
| Spells cast | 23.9 | 21.2 |
| Lands played | 18.7 | 16.4 |
| Combat w/ atk | 10.8 | 8.4 |
| Creatures died | 7.8 | 4.9 |
| Damage events | 22.6 | 17.5 |
| Total damage | 65.4 | 52.4 |
| Life changes | 14.4 | 12.5 |
| **Layer walks** | **399** | **419** |
| **Board walks** | **258** | **237** |
| **Memo hits** | **67,128** | **50,792** |
| **Layer frames** | **4,717** | **4,023** |
| **Frames/walk** | **11.82** | **9.59** |
| **Dependency checks** | **16** | **18** |
| **Replacement gathers** | **561** | **445** |
| **Restriction queries** | **562** | **448** |
| Prevention allocations | 0.00 | 0.00 |

The engine's share, by the four-arm protocol (`replacement-architecture.md`
§9, RD-2 as landed): gathers and walks flat on both middle arms; the middle
arms differ from `main` outside the timing block in three and four of 200
`performance` games, every one with two Furnaces of Rath on the battlefield —
one CR 616.1 prompt where a two-member batch had two, then none where the
suppression removes it — and in nothing else; CPU/game −0.8% and −0.9% on the
middle arms, +2.1% shipped. Reachability is the `Prevention allocations` row
and the `--require` counts in that section: 0.02 per `stress` game unforced,
0.05 with the three unpooled cards forced.

**Re-recorded 2026-09-09 for RD-3** — `PERFORMANCE_POOL` +1 (Guardian Seraph,
75 → 76) and the stress pool +8 (102 → 110). **The engine's share is zero, and
this is the cleanest reading the protocol has produced:** the middle arm —
RD-3's engine with `registry.rs` and both pools unchanged — is *byte-identical*
to `main` outside `=== Timing ===` at 200 games on both pools, so every
movement below is the pool's. A widened `EventPattern` arm costs nothing until
a def writes the new fields (`replacement-architecture.md` §11 item 32).

| | performance (76 cards) | stress (110 cards) |
|---|---|---|
| P0 / P1 | 23 (46.0%) / 27 (54.0%) | 29 (58.0%) / 21 (42.0%) |
| Avg turns | 31.1 | 32.1 |
| Spells cast | 23.8 | 23.4 |
| Lands played | 18.4 | 18.8 |
| Combat w/ atk | 10.4 | 9.5 |
| Creatures died | 7.7 | 5.4 |
| Damage events | 21.3 | 19.9 |
| Total damage | 62.4 | 55.0 |
| Life changes | 14.4 | 12.5 |
| **Layer walks** | **374** | **514** |
| **Board walks** | **242** | **284** |
| **Memo hits** | **61,239** | **73,967** |
| **Layer frames** | **4,427** | **5,262** |
| **Frames/walk** | **11.84** | **10.23** |
| **Dependency checks** | **29** | **78** |
| **Replacement gathers** | **534** | **565** |
| **Restriction queries** | **536** | **568** |
| Prevention allocations | 0.02 | 0.00 |

At 200 games the shipped arm is `performance` walks 368 → 361 and gathers
506 → 508 — a source-side `ObjectFilter` on every damage event is free at this
board size — against `stress` walks 437 → 496 and gathers 513 → 563, which is
eight cards in every deck. CPU/game median 14.10 → 14.15 ms (+0.4%); zero
errors and zero panics on every arm and pool; three shell runs at one seed
identical outside the timing lines.

**One thing this instrument does not measure, found the hard way.** The
`--require` block counts a card's **casts**, and RD-3's pooled question was
about an *activated ability*: Circle of Protection: Red resolves in 130 of 200
forced `stress` games, and its `{1}` ability is activated **12,660 times** in
129 of them — about 98 per game it reaches the battlefield, which is the
opposite of the prediction that asked for the number. The count came from
`--dump-events` plus `grep "AbilityActivated: <name>"`, and until the report
grows a counter that is the recipe. **A phase whose consumer is an activated
ability should measure the activation, not the cast.**
| Layer walks / frames | 378 / 4,504 | 379 / 4,510 |

**Read `ms/1,000 queries` carefully here: it rises 41.8%, and that is the
speed-up rather than a regression.** Queries are walks plus memo hits, so a
37% fall in memo hits shrinks the denominator faster than the numerator falls.
This is the case §3.1 warns about — "fewer questions walked" is a different
finding from "the walk got slower" — and it is the first time the project has
produced it. CPU/game and memo hits are the rows to read.

One behavioural row moves, and it is the bug: total damage 56.6 → 56.5 per
`performance` game, which is an equipped Bonesplitter detaching under March of
the Machines and no longer granting +2/+0. Zero errors and zero panics on both
pools; three shell runs at one seed identical outside the timing lines.

| | performance (73 cards) | stress (94 cards) |
|---|---|---|
| P0 / P1 | 28 (56.0%) / 22 (44.0%) | 25 (50.0%) / 25 (50.0%) |
| Avg turns | 28.9 | 28.2 |
| Spells cast | 22.3 | 22.6 |
| Lands played | 17.5 | 17.0 |
| Combat w/ atk | 9.9 | 10.1 |
| Creatures died | 6.6 | 4.5 |
| Damage events | 21.7 | 22.7 |
| Total damage | 59.3 | 58.6 |
| Life changes | 14.3 | 14.8 |
| **Layer walks** | **371** | **434** |
| **Board walks** | **235** | **251** |
| **Memo hits** | **55,610** | **57,618** |
| **Layer frames** | **4,267** | **4,547** |
| **Frames/walk** | **11.50** | **10.48** |
| **Dependency checks** | **10** | **42** |
| **Replacement gathers** | **475** | **485** |
| **Restriction queries** | **477** | **487** |


**Three arms again (2026-09-06, LI-3).** `plans/fuzz_ab.py`, one sitting:
`main` at 9011d42 (A), LI-3's engine with the registry and both pools
unchanged (B), and LI-3 as shipped (C).

| | A: main | B: engine, pools unchanged | C: LI-3 |
|---|---|---|---|
| performance / stress, 200 games, outside `=== Timing ===` | — | **identical** | differs — pool |
| performance layer walks / board walks (50 games) | 334 / 241 | 334 / 241 | 321 / 232 |
| performance frames, frames/walk (50 games) | 4,296, 12.85 | 4,296, 12.85 | 4,074, 12.70 |
| performance dependency checks (50 games) | 23 | 23 | 21 |
| performance CPU/game median (200 games, ×3) | 15.72 ms | 15.89 ms (+1.1%) | 16.01 ms (+1.8%) |
| performance ms / 1,000 questions (walks + hits) | 0.163 | 0.164 (+1.1%) | 0.162 (−0.2%) |
| performance CPU/game p99 median | 45.36 ms | 45.68 ms | 70.31 ms |
| stress dependency checks (200 games) | 87 | 87 | 72 |

**B is identical to A this time, not "identical the new row aside".**
Every counter, every behavioural row and every cost row matches at 50 and
at 200 games on both pools; the three serial timing rounds are identical
line for line outside `=== Timing ===`; and on 40-game `--dump-events`
streams with the id masks applied, the whole stream is byte-identical on
both pools — 0 of 40 games differ. That is what a conditional-existence
clause predicts for a pool with no conditional card: `Effect::Conditional`
is an arm no registered ability's body reaches, so `condition::holds` is
never called and `condition_reads` adds no channel. The claim the A/B can
make is therefore narrow and exact — **the change is inert until a
conditional card is in the pool** — and every row that moves in C is Kird
Ape's.

Then C. 40 of 40 games differ on each pool, the first divergence in
`performance` being event 18, `Keldon Warlord ... Library -> Hand` against
`Kird Ape ... Library -> Hand`: the registry's sorted name list grew by
one, so `random_deck` draws different cards from the same seeded stream.
Nothing in those diffs is the engine's. The p99 column is the one to read
carefully — 45.36 → 70.31 ms is a *different set of games*, not a slower
engine, since B's p99 sits on A's; CPU/game median moves +1.8%, inside the
sitting's spread, and ms per 1,000 questions is flat to slightly down.

**Re-recorded 2026-09-06 for LI-2** (CR 613.8a/b/c, the dependency loop;
`layers-architecture.md` §13b). One new card in `performance` — Urborg,
Tomb of Yawgmoth, 68 → 69 — and three in `stress` — Urborg, Opalescence,
Ashaya, Soul of the Wild, 78 → 81 — and a new bold row, **`Dependency
checks`**: the CR 613.8a hypotheticals a game ran, which are the pairs the
static channel check could not settle. **Every movement below is the
pool's**: with the registry and pools unchanged, LI-2's engine reproduces
LI-1's table row for row on both pools (the middle arm of the three-arm
table), so what moved is Urborg dropping into `performance` decks and, in
`stress`, three new names in the list `random_deck` draws from — every
stress game's deck changed, and that column is not comparable to LI-1's
beyond the fact that the engine did not move it.

| | performance (69 cards) | stress (81 cards) |
|---|---|---|
| P0 / P1 | 26 (52.0%) / 24 (48.0%) | 30 (60.0%) / 20 (40.0%) |
| Avg turns | 31.2 | 28.5 |
| Spells cast | 23.9 | 21.7 |
| Lands played | 18.4 | 17.4 |
| Combat w/ atk | 11.0 | 9.6 |
| Creatures died | 7.2 | 4.1 |
| Damage events | 23.7 | 20.8 |
| Total damage | 58.0 | 54.6 |
| Life changes | 16.6 | 14.1 |
| **Layer walks** | **334** | **334** |
| **Board walks** | **241** | **247** |
| **Memo hits** | **95,297** | **93,311** |
| **Layer frames** | **4,296** | **4,452** |
| **Frames/walk** | **12.85** | **13.34** |
| **Dependency checks** | **23** | **63** |
| **Replacement gathers** | **513** | **483** |
| **Restriction queries** | **516** | **486** |

**Three arms, and the middle one is `main` (2026-09-06, LI-2).**
`plans/fuzz_ab.py`, one sitting: `main` at a6f2ed8 (A), LI-2's engine with
the registry and pools unchanged (B), and LI-2 as shipped (C).

| | A: main | B: engine, pools unchanged | C: LI-2 |
|---|---|---|---|
| performance, 200 games, outside `=== Timing ===` | — | identical (the new row aside) | differs — pool |
| performance layer walks / board walks (50 games) | 328 / 236 | 328 / 236 | 334 / 241 |
| performance frames, frames/walk (50 games) | 4,289, 13.08 | 4,289, 13.08 | 4,296, 12.85 |
| performance dependency checks (50 games) | — | 13 | 23 |
| performance CPU/game median (200 games, ×3) | 15.42 ms | 15.89 ms (+3.0%) | 15.72 ms (+1.9%) |
| performance ms / 1,000 questions (walks + hits) | 0.155 | 0.160 (+3.0%) | 0.163 (+4.8%) |
| performance CPU/game p99 median | 48.12 ms | 49.25 ms | 45.45 ms |
| stress, 200 games | — | identical (the new row aside) | differs — pool |
| stress dependency checks (200 games) | — | 7 | 87 |

Read B first, because it is the finding. **The engine change alone changes
no game in either pool**: every counter and every behavioural row matches
A at 50 and at 200 games, the three serial timing rounds are identical
line for line outside `=== Timing ===`, and on 40-game `--dump-events`
streams with the id masks applied, 0 of 40 games differ on `performance`
and 0 of 40 on `stress`. That is CR 613.8's prediction for this pool: its
only dependency-shaped pairs are Humility beside a creature's static
ability, and the dependency's answer there was already timestamp order's
in both directions (LI-1's flipped pin). B's 8 hypotheticals per game at
200 games are exactly those pairs, each confirming a dependency that
changes nothing. (`fuzz_ab.py` prints "differ" for B against A because the
new counter row is a new line; the raw outputs under `--out` minus that
line are what "identical" means here.)

Then C. 29 of 40 `performance` games and 40 of 40 `stress` games differ
from A, every one first at a `Library -> Hand` draw event: the registry's
sorted name list changed, so `random_deck` draws different cards from the
same seeded stream. Nothing in those diffs is the engine's. The cost rows
move with the pool too — `Dependency checks` 13 → 23 on `performance` is
Blood Moon meeting Urborg — and CPU/game is flat inside the sitting's
spread: B +3.0% and C +1.9% against A, with round 3 reading B *faster*
than A. The loop's fast path is why: a layer whose applications are
pairwise independent under the channel check costs N² bit-ands and no
hypothetical, which is every layer of every board that has no
dependency-shaped card.

**How to read `Dependency checks` from now on.** It is the CR 613.8 loop's
slow path — pairs the static check could not settle, each a frame clone
per member the other application reaches, applied and taken back. Zero
means every pair in every layer was settled statically. Read it beside
`Board walks`: checks per board walk is how many pairs per pass reached
the expensive half, and a rise against unchanged board walks means a
dependency-shaped card started meeting another more often.

**Re-recorded 2026-09-06 for LI-1** (the board-wide sequential pass;
`layers-architecture.md` §13b). No new card and no pool change — the
consumer, Humility beside Citanul Hierophants, was already in both pools —
so for once **every row is the engine's**, and the table gains a row,
`Board walks`: the layer walks that computed the whole working set at once.
Since LI-1 a miss for a permanent walks the whole board and fills the memo
for every member, so `Layer walks` is the number of *boards* computed plus
the walks of objects no row can reach, and `Frames/walk` reads near the
board's size.

| | performance (68 cards) | stress (78 cards) |
|---|---|---|
| P0 / P1 | 29 (58.0%) / 21 (42.0%) | 31 (62.0%) / 19 (38.0%) |
| Avg turns | 30.7 | 33.1 |
| Spells cast | 23.4 | 24.3 |
| Lands played | 18.3 | 18.9 |
| Combat w/ atk | 10.1 | 11.6 |
| Creatures died | 6.6 | 4.0 |
| Damage events | 21.8 | 25.6 |
| Total damage | 53.6 | 60.1 |
| Life changes | 15.7 | 17.4 |
| **Layer walks** | **328** | **382** |
| **Board walks** | **236** | **286** |
| **Memo hits** | **95,005** | **122,108** |
| **Layer frames** | **4,289** | **5,682** |
| **Frames/walk** | **13.08** | **14.88** |
| **Replacement gathers** | **505** | **602** |
| **Restriction queries** | **507** | **604** |

**Two arms, and the second legitimately differs from `main` (2026-09-06,
LI-1).** `plans/fuzz_ab.py`, one sitting: `main` at 650633f (A) and LI-1
(B). There is no "engine, pool unchanged" arm distinct from the shipped one,
because the pool did not change; and B is not A's stream, because the pooled
board's answer did — a creature under Humility no longer taps for the
Hierophants' {G}.

| | A: main | B: LI-1 |
|---|---|---|
| performance, 200 games, outside `=== Timing ===` | — | differs |
| performance layer walks / board walks (50 games) | 2,425 / — | 328 / 236 |
| performance frames, frames/walk (50 games) | 3,755, 1.55 | 4,289, 13.08 |
| performance CPU/game median (200 games, ×3) | 14.56 ms | 14.81 ms (+1.7%) |
| performance ms / 1,000 questions (walks + hits) | 0.146 | 0.149 (+2.1%) |
| performance CPU/game p99 median | 48.28 ms | 46.58 ms |
| stress, 200 games | — | differs |

Read the divergence first, because it is the finding. On 40-game
`--dump-events` streams with the id masks applied, **one game in forty
differs on each pool** — `performance` game 38, `stress` game 20 — and in
both, Humility and Citanul Hierophants had entered the battlefield before
the first divergent event, which is a choice made by index (a mana source
tapped for a payment; a blocker) from a list in which a creature under
Humility no longer offers the granted ability. Every other game is
byte-identical to `main`'s. The behavioural rows move by that one game per
pool: `performance` by a tenth of a turn, `stress` by more because its game
20 diverged early (event 521 of 814) and ran to 1,297 events.

Then the cost rows, which are the pass's shape rather than its price. Layer
walks fell 7× because one board walk fills the memo for every member where
each member used to miss on its own; frames rose 14% because a pass builds one frame
per member where a walk built 1.55; the two together are the +1.7% of
CPU/game and the +2.1% per question, both inside the sitting's spread
(round 1 read +0.5%, round 2 +4.7%, round 3 +1.7%). The look-ahead's cost —
a pass per entry where it was one walk — is in there and did not show;
`layers-architecture.md` §13b names the two answer-preserving levers if a
later board makes it show.

**The `main` arm has to be built at the commit the worktree is synced to.**
The first sitting compared against a `fuzz_games.exe` built the day before
#102's last commits, and every game differed from its first draw because the
decks did. `git worktree list` says where the source is; only a
`cargo build --release --bin fuzz_games` in that worktree says where the
binary is.

**How to read `Board walks` from now on.** A layer walk is a board walk or
the walk of an object no row can reach (`board::membership`); board walks ×
the working set is the bulk of `Layer frames`. A board-walk count that rises
against unchanged layer walks means the engine started asking about
permanents at more epochs; frames rising against unchanged board walks means
the boards got bigger.

**Re-recorded 2026-09-06 for LH-2** (CR 613.7e and Equip;
`layers-architecture.md` §13a). Two new cards: Bonesplitter in both pools —
the first Equipment, the first `Primitive::Attach` / `GameAction::Attach`,
and the first activation restriction — and Cobbled Wings in `stress` only,
since it opens no path Bonesplitter does not. That moves `performance` from
67 cards to 68 and `stress` from 76 to 78. **The rows are the pool's and a
bugfix's, not the walk's**: the four arms below say which.

| | performance (68 cards) | stress (78 cards) |
|---|---|---|
| P0 / P1 | 29 (58.0%) / 21 (42.0%) | 31 (62.0%) / 19 (38.0%) |
| Avg turns | 30.8 | 32.3 |
| Spells cast | 23.5 | 23.7 |
| Lands played | 18.3 | 18.6 |
| Combat w/ atk | 10.1 | 11.4 |
| Creatures died | 6.6 | 4.0 |
| Damage events | 21.8 | 25.1 |
| Total damage | 53.6 | 59.1 |
| Life changes | 15.7 | 17.2 |
| **Layer walks** | **2,425** | **3,384** |
| **Memo hits** | **93,215** | **113,452** |
| **Layer frames** | **3,755** | **4,629** |
| **Frames/walk** | **1.55** | **1.37** |
| **Replacement gathers** | **506** | **579** |
| **Restriction queries** | **508** | **581** |

**Four arms this time, because the consumer found a bug (2026-09-06,
LH-2).** `plans/fuzz_ab.py`, one sitting: `main` (A); LH-2's engine with the
`activate_ability` allocation fix reverted and both new cards unregistered
(B, "clean"); the engine as shipped, both registered and Bonesplitter not
pooled (B′); and LH-2 shipped (C). B is the "engine, pool unchanged" arm the
protocol asks for. B′ exists because the fix — an activated ability with a
generic pip had never been payable, so it was blacklisted at every
activation — changes what Chainbreaker, a pooled card, does in a game.

| | A: main | B: clean engine | B′: engine + fix | C: shipped |
|---|---|---|---|---|
| performance, 200 games, outside `=== Timing ===` | — | **A's, but `Memo hits` +4** | differs | differs |
| performance walks (50 games) | 2,421 | 2,421 | 2,543 | 2,425 |
| performance frames/walk | 1.44 | 1.44 | 1.43 | 1.55 |
| performance CPU/game median (200 games, ×3) | 13.16 ms | 13.23 ms (+0.5%) | 13.65 ms (+3.7%) | 14.92 ms (+13.4%) |
| performance ms / 1,000 walks | 5.454 | 5.483 (+0.5%) | 5.695 (+4.4%) | 5.637 (+3.4%) |
| stress, 200 games | — | A's, but `Memo hits` +3 | **identical to C** | — |

Read B against A first. Its event stream *is* `main`'s — 40-game dumps
identical after the id masks, and the sweeps keying on a timestamp CR 613.7e
now reassigns moved no pick in 200 games — and the extra memo hits are the
new `can_pay_costs` pre-check in `activate_ability` asking cached questions.
The walk itself is `main`'s code: a re-stamp at attach time
(`retime_static_rows`) costs one registry pass per equip, not a per-frame
sort, and B's +0.5% is inside the sitting's own noise (round 2 had B faster
than A). B′ against B is Chainbreaker's ability resolving at all (85 times
per 40 games, from 0): more stack, more resolutions. C against B′ is the
card: an Equipment on the battlefield in 140 of 200 games, whose `Host` row
walks its source for the existence check. That is the frames/walk
1.43 → 1.55 and the +13.4% of CPU/game, of which +3.4% is per walk — the
shape LH-1 measured for Holy Strength at 1.39 → 1.44, on a card cast about
three times as often because it is a colorless {1}. The first version of
this PR read the rows' timestamps live in the walk instead, and the same
board then cost +9.5% per 1,000 walks; §13a records why that shape was
replaced.

**Re-recorded 2026-09-04 for LH-1** (the Aura host becomes addressable;
`layers-architecture.md` §13a). One new card, Holy Strength, in both pools —
the first `AffectedSet::Host` row, and the first spell whose target
is its enchant ability rather than a spell ability — which moves `performance`
from 66 cards to 67 and `stress` from 75 to 76. **Every row here is the pool's
and none is the engine's**, measured the way RC-5 measured it, below.

| | performance (67 cards) | stress (76 cards) |
|---|---|---|
| P0 / P1 | 30 (60.0%) / 20 (40.0%) | 32 (64.0%) / 18 (36.0%) |
| Avg turns | 31.9 | 28.3 |
| Spells cast | 24.6 | 21.1 |
| Lands played | 19.0 | 17.1 |
| Combat w/ atk | 11.7 | 10.1 |
| Creatures died | 7.7 | 3.4 |
| Damage events | 26.0 | 22.3 |
| Total damage | 66.2 | 56.1 |
| Life changes | 18.0 | 15.0 |
| **Layer walks** | **2,421** | **1,962** |
| **Memo hits** | **94,202** | **75,607** |
| **Layer frames** | **3,496** | **2,697** |
| **Frames/walk** | **1.44** | **1.37** |
| **Replacement gathers** | **527** | **452** |
| **Restriction queries** | **530** | **454** |

**The engine's share is zero again, by the same three-arm protocol
(2026-09-04, LH-1).** `plans/fuzz_ab.py`, one sitting: `main` (A), LH-1's
engine with `PERFORMANCE_POOL` exactly as `main` had it (B), and LH-1 shipped
(C).

| | A: main | B: engine, pool unchanged | C: shipped |
|---|---|---|---|
| performance, 200 games, outside `=== Timing ===` | — | **byte-identical to A** | differs |
| performance walks (50 games) | 2,550 | 2,550 | 2,421 |
| performance frames/walk | 1.39 | 1.39 | 1.44 |
| performance CPU/game median (200 games, ×3) | 12.84 ms | 12.78 ms (−0.5%) | 12.15 ms (−5.4%) |
| performance ms / 1,000 walks | 5.116 | 5.092 (−0.5%) | 5.035 (−1.6%) |
| stress, 200 games | — | **identical to C** | — |

Read as RC-5's was read. B against A is byte-identical on `performance`, so
the new `effect_applies_to` arm, the `StackEntry` field and the recipient
helper cost the pool nothing when no card reaches them; B against C is
identical on `stress`, because a registered card is in that pool whether or
not it joined the measured one. **The shipped column's −5.4% is the game, not
the walk**: a +1/+2 Aura in roughly a third of decks ends games sooner (33.3 →
31.8 turns over 200 games), so fewer walks happen, while `ms / 1,000 walks`
moves −1.6% — inside the sitting's spread, and not a finding. `Frames/walk`
rises on `performance` (1.39 → 1.44) for the reason RC-5 gave for Master
Biomancer: a row scoped to one host puts a sub-frame under its source. And
`stress` loses a death a game (4.4 → 3.4) because a creature wearing +1/+2
survives combats it used to lose — the one behavioural row LH-1 moves on its
own.

*Previous values, 2026-09-03 (performance 66, stress 75 — RC-5; reproduced to
the digit by this sitting's `main` arm, which is the check that the re-record
is the cards and not the machine): performance 27/23, 34.5 turns, 24.9 spells,
20.0 lands, 12.0 combats, 7.9 deaths, 24.3 damage events, 54.7 damage, 16.3
life changes, 2,550 walks / 102,077 memo hits / 3,555 frames / 1.39 per walk /
567 gathers / 570 queries; stress 28/22, 30.6, 22.3, 18.2, 10.2, 4.4, 24.0,
56.9, 17.0, 2,214 / 87,597 / 3,116 / 1.41 / 504 / 506. RC-5's own three-arm
A/B read engine +0.7% and shipped +1.0% CPU/game against its `main`, B
byte-identical to A on `performance` and to C on `stress` — the same partition
this sitting reproduces. Before that, 2026-09-03 (performance 64, stress 72 —
before RC-5's three cards; reproduced to the digit by RC-5's `main` arm): performance 28/22, 34.6
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

**Census, 2026-09-08, all 92 registered cards** (`/cards/collection` for
identity, then one rulings fetch each):

| | |
|---|---:|
| cards carrying at least one ruling | 43 of 92 (46%) |
| total rulings | 145 |
| …on the 73 `PERFORMANCE_POOL` cards | 87 |
| median rulings per card | 0 |
| most on one card | 8 (Cytoshape, pooled) |

145 is a bounded job, not an open-ended one, and at CM-3's observed rate — 11
rulings into 6 tests, 2 already-covered, 3 named gaps — it is roughly 80 tests
across the whole registry, or 50 if the pool goes first.

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

Sizing the ledger alone: ~250 lines of Python, one data file, one line in the
check command. About `check_state_of_play.py`.

**Scheduled (the owner, 2026-09-08): after replacements, between phases —
`roadmap-v2.md` row A4b.** It gates nothing and nothing gates it, which is the
argument for giving it a slot rather than a "whenever": a task that is nobody's
blocker is deferred indefinitely by default, and this one has already produced
a live bug in the measured pool on its first afternoon (item 82). A4 is also
the last point at which the retroactive half is a sitting rather than a
project, since the registry only grows.

The two halves stay separable and only one of them is the tool. **The reading**
needs no tool and is pool-first: 87 rulings across the 73 pooled cards. **The
tool** is the drift detector and the parser's acceptance test, and it must be
in place before the first machine-ingested card is registered.

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
atoms as of 2026-09-03, nearly all damage and prevention).

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
| `rd-2-a-decision-is-per-subject.html` | RD-2 | the CR 616.1 loop's new unit: two shield counters under two blockers through the per-member loop and the per-subject one, and the first-strike twin that shows the key is the batch; Furnace beside Mending Hands in both orders; a `NextDamage(3)` under sources of 2 and 4 with the allocation asked once; the two boards where nothing is consumed — Safe Passage beside Mending Hands, and a `Once` half chosen against 1 — and the consume-after-apply order that makes them right |

**When to write one: at phase close, for a phase that changes *how* a read is
answered rather than what the answer is.** That is the property the two above
share, and it is why a phase that adds a card, an enum arm or a pool entry does
not get one. The phases that qualify were listed when the practice started:
RC-4 ✓, RC-4b ✓, CV-1 ✓, RC-5 ✓, item 7 ✓ (twice — LI-1 mid-phase, because the
pass changed every read at once, and the close), RD-2 ✓ (the loop's unit; the
one RD phase that qualifies, decided at its close as §9 scheduled), **RS-2,
critical-path item 6**. Budget
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
  **Scheduled: its own PR, after CM-4 and before item 6** — `roadmap-v2.md`
  row A4c, moved there 2026-09-08 out of the trigger phase's first PR. The first
  question anyone asks a trigger dispatcher is "why did this fire, or not",
  which is a trace question, and it wants answering before that phase starts
  rather than with it. Sized at `codebase-state.md`, "Before Triggered
  abilities" item 5 — **with two corrections recorded there**: a sink generates
  a page's *spine*, never its argument, so tier 1 is not subsumed by tier 2;
  and item 5's "gated the way `EngineCounters` is" describes no mechanism,
  since those counters are always on.
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
