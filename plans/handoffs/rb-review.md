# PR #62 (Phase RB) — review findings ledger

**Delete this file when the last row is closed.** Per `CLAUDE.md`'s authority
table, a handoff is where to resume half-finished work and does not outlive it.

## Why this file exists

PR #62 is +5,475 / 33 files. A review of it produced ~40 findings, and the
failure mode is not the *number* — it is carrying all of them in one working
context and fixing a dozen unrelated things across nine commits in a single
pass. Quality degrades on the later ones, and it is easy to lose one silently.

**The file breaks that coupling.** Findings land here as they are made; fix
sessions start cold from this file and need nothing from the conversation that
produced them.

## How to use it

1. **Capture everything first, fix nothing.** Reviewing and fixing in the same
   pass is what produces the scatter.
2. **Triage into a bucket** before any code changes:
   - `fix` — a real defect in what RB shipped. Belongs in this PR.
   - `doc` — the code is right and a claim written about it is wrong. Cheap, no
     test, no re-measurement.
   - `defer` — real, but belongs to a later phase. Gets a `codebase-state.md`
     Deferred Migrations entry and **leaves this PR**.
   - `design` — not a defect; a modelling question that needs a decision before
     it becomes one. Answer it in a doc, not in a fix session.
3. **Close one *theme* per session, not one row.** The findings below are
   grouped A–G for exactly this reason: the rows inside a theme share a mental
   model, and a session that closes theme C is coherent where a session that
   closes C3, E1 and F4 is the thing this file exists to prevent.
4. **Close a row by naming the commit that did it.**
5. Re-run the gates once per session, not once per finding: `cargo build
   --all-targets` (zero warnings), `cargo test`, `fuzz_games --games 50 --seed
   12345` against the baseline, `specdb orphans` / `suspicious` if any `COVERS`
   line moved.

**Suggested session order.** G first (it is process, and it changes how
everything else gets written), then B and E (cheap, mechanical, no design
input), then C, then D and F. **A is not a fix session at all** — it is a design
pass that should probably become its own document before any of it is coded.

**The RB baseline:** P0 28 (56.0%) / P1 22 (44.0%), spells 20.8, lands 17.9,
combat 11.4, creatures died 5.6, damage events 24.4, total damage 53.6, life
changes 18.0. 744 tests, 0 warnings. Perf 13.04 ms/game at `--games 200 --seed
12345`.

---

## Already known and deliberately not fixed in RB

Seeded so a review does not spend findings re-discovering these. Each is
recorded in a doc already; disagreeing with the *decision* is a finding,
re-noticing the gap is not.

| # | Item | Where recorded | Why not in RB |
|---|---|---|---|
| K1 | CR 614.15 self-replacement has a `ReplacementClass` bucket and no producer; `ResolutionContext` still has three fields | §11 item 3 | Lands with the first card that needs it (**but see F5** — the review asks whether that is the right shape) |
| K2 | CR 614.17c's blocked path therefore always drops the event | §9's RB block | Correct today — the only class that could survive a block has nothing in it |
| K3 | §3.3 source 2 (static abilities in other zones) still unsized | §11 item 4 | **RB was asked to size this and did not** (see D5) |
| K4 | CR 704.5q counter annihilation still writes `BattlefieldEntity` directly | Deferred Migrations 6 | Removes two counter kinds at once; would have to join the SBA batch |
| K5 | CR 704.7's same-result collapse does not reach player loss | Deferred Migrations 6 | Needs `GameAction::PlayerLoses` (Phase RE) |
| K6 | CR 704.7's dedupe lives in the SBA sweep, not `execute_actions` | §9's RB block | The sweep is what knows CR order for naming the cause |
| K7 | Neither half of CR 903.9 is reachable in a real game | `codebase-state.md` CR 9 | Nothing outside tests sets `is_commander`; that is CR 903.7's hook |
| K8 | All of CR 615's prevention machinery is absent | `cards-unlocked-ledger.md` Part 3 | Phase RD |

---

## A. The "can't" model — one design gap wearing four costumes — ✅ **CLOSED 2026-08-27**

**Answered in `plans/cant-effects-architecture.md`**, which is now the authority
for CR 101.2 / 614.17 / 613.11. The evidence is
`plans/references/cant-census.py` (all 1,857 non-funny cards, 2,034 clauses,
classified by enforcement point). Nothing was coded; the four rows below are
closed as *designed*, not as *fixed*.

**The headline.** RB's `is_blocked` is correct and is **12% of the problem** —
236 of 2,034 clauses. The other 88% never reach `execute_action`, because the
player is never offered the choice that would propose the event. There are six
enforcement points, not one, and the design gives them one shared type
(`RestrictionDef`) discovered by one sweep and read by one predicate.

| # | Area | Finding | Verdict |
|---|---|---|---|
| A1 | design | **"Can't" effects are not scoped.** Motivating cards, deliberately spread across engine layers: **Aggressive Mining** / **Abyssal Persecutor** (stop a *state-based action* or a special action), **Conduit of Worlds** (a conditional restriction on *casting*), **Grafdigger's Cage** (categorical *zone-movement* prevention), **Rakdos, Lord of Riots** (a self-imposed casting restriction), **Yasharn, Implacable Earth** (prevents a *category of cost* from being paid). A `is_blocked(action)` predicate over `GameAction` reaches none of the casting, cost or special-action cases. | ✅ `cant-effects-architecture.md` §2 (measured), §3 (modelled), §6.1 (each motivating card placed) |
| A2 | `pipeline.rs:53` | `is_blocked`'s match will grow one arm per "can't" if the current shape is kept. Today one arm; the card pool says 1,888. | ✅ §6.2 — the arm becomes **data** (`Restriction::Event { pattern: EventPattern, .. }`), so it stops growing. Also supersedes the archive plan's `L15` enum, which was a variant per card |
| A3 | `resolve.rs:638` | `Primitive::CantBeRegenerated` — is there a primitive per "can't" stopper in the card base? | ✅ §6.3 — **no**: one `Primitive::Restrict(RestrictionDef, Duration)`, the exact analogue of `Primitive::Regenerate`. 563 of 2,034 clauses need it; the other 1,471 are discovered and need no primitive at all |
| A4 | `game_state.rs:269` | An entire `HashSet<ObjectId>` on `GameState` for one rule (`cant_be_regenerated`). Sound today; a bad precedent if every "can't" gets its own field. | ✅ §6.4 — replaced by a shared duration registry (§9 finding 7 — the third such registry, so the expiry half gets lifted first as **RS-0**). **Closes a real gap:** `turns.rs:138`'s cleanup clear is too *broad*; CR 608.2c scopes "It can't be regenerated" to that one destruction |

**What A did *not* answer, deliberately.** The 149 "can't … unless" clauses need
`Effect::Conditional` (Phase 6), and CR 613.10 still has no home doc.

**Three things the owner's review of the design changed** (2026-08-27, same
day): CR 608.2c settles the "can't be regenerated" scope and makes it an
*authoring* concern rather than an inferred one; §4.2's combat solver is now
counted rather than deferred (**1,249 of 1,262** Tier-1a clauses are a
per-creature predicate, and the solver exists for ~15 coupling cards plus ~150
requirement cards, which is why RS-3 split into 3a/3b); and CR 601.3a turned out
not to be a forward-looking search at all — Void Winnower's rulings make it
over-approximate-at-the-gate plus a CR 601.2 rewind.

**The deadline is now specific.** "A must land before RC-4" resolves to **RS-1
must land before RC-4** — the Tier-2 spine only, which is a *deleting* PR
(three call sites replaced, one `GameState` field and one `turns.rs` clear
removed). RS-2/RS-3/RS-4 block nothing in the RC/RD/RE line.

---

## B. Doc and comment accuracy — the code is right, something written is wrong

Cheapest theme. No tests move, no re-measurement.

| # | Area | Finding | Verdict |
|---|---|---|---|
| B1 | `actions.rs:162` | **"CR 701.8b makes 'destroyed' a strictly narrower fact than 'put into a graveyard from the battlefield'" is wrong.** Destroy is an *action*, not a description of a result: under Rest in Peace a destroyed creature never reaches a graveyard, and Karmic Justice still triggers. The two are overlapping, not nested. **The code is right** — `Destroy` being the outer event with `ZoneChange` inside it is exactly what makes the Rest in Peace case expressible — but the comment argues for it from a false premise. | `doc` |
| B2 | `gather.rs:228` | **Factually wrong: "the shield is still not spent, and is still there for a later turn."** CR 701.19a shields last *this turn* — `Duration::UntilEndOfTurn`, which the code sets correctly. Same sentence appears in the CR 701.19c test's comment. | `doc` |
| B3 | `actions.rs:282` | **The CR 101.4d justification is overstated.** The comment claims a restart is "unreachable in this shape" because `consume_use` only removes candidates. The real reason it has not come up is narrower: nothing in phase 1 can *create* a replacement effect, and each batch member's 616.1f loop runs to completion before the next begins — which is a simplification, not a proof. See F6 for the deferred part. | `doc` |
| B4 | `zones.rs:115` | `Fizzled`'s doc says "countered by game rules". Current CR 608.2b does not use that phrase — the spell "doesn't resolve" and is removed from the stack. Fizzling is not countering. | `doc` |
| B5 | `types/replacement.rs:333` | The "Why there is no `CounterBacked`" history is design archaeology and belongs in `replacement-architecture.md` §3.2, where it already is. The code needs the rule, not the story. | `doc` |
| B6 | `resolve.rs:83` | **Malformed string literal.** The `Effect::Replacement` error message is one long line with runs of ~18 literal spaces — a line-continuation that did not survive how the file was written. Cosmetic, but it is what a user sees. | `fix` |

---

## C. Naming and semantics

Rows here share one cause: RB introduced a lot of vocabulary quickly and some of
it collides with vocabulary that already existed.

| # | Area | Finding | Verdict |
|---|---|---|---|
| C1 | `gather.rs:57` | **`Affected` and `AffectedSet` are two different ideas one letter apart.** `Affected` is *what a proposed event is about*; `AffectedSet` is *the boundary of an effect's shield*. Reading them side by side implies a relationship that does not exist. Rename one. | `fix` |
| C2 | `types/replacement.rs:45` | Reusing `AffectedSet` for `ReplacementDef.affected` reads as too narrow for how broad the name is — the doc calls it "which objects it shields", which is regeneration/shield-counter language applied to every replacement effect. Kalitas does not shield anything. | `fix` (naming/doc) |
| C3 | `replacement_effects.rs:44` | `ReplacementRowId` / `RegisteredReplacement` / `ReplacementRegistry` read as "replacing a row in some other registry". Tack on `Effect`: `ReplacementEffectId`, `RegisteredReplacementEffect`. | `fix` |
| C4 | `actions.rs:350` | `resolve_rider` is documented as "one CR 615.5 rider", but 615.5 is about *prevention* effects only. Kalitas's token is a rider on a plain replacement. Find the right citation (CR 614.1a's "instead" clause plus the effect's own text) or drop the rule number. | `doc` |
| C5 | `pipeline.rs:134` | `candidates` is used for two different things a line apart — the gathered-and-filtered list, and then `bucket` after `forced_bucket`. Name the first `applicable` and the second `bucket`, or fold the filter into `gather`. | `fix` |
| C6 | `gather.rs:246` | **`applies_to` checks "does the pattern match" AND "is the affected object inside the shield", and the second half is named `shield_contains` on effects that shield nothing.** The function is correct — an effect applies only if it watches this *kind* of event *and* this event is about something in its scope — but the naming makes it read as two unrelated checks. Rename to something like `scope_contains` / `watches`. | `fix` |
| C7 | `replacement/mod.rs:1` | Every other `mod.rs` in this project is a re-exporter. This one carries type definitions and a module-level essay. Move `ReplacementInstance` / `ReplacementInstanceId` into a file. | `fix` |

---

## D. Scaling — shapes that work at three cards and may not at three thousand

Not defects. Each is a question about whether the current shape is the one to
grow.

| # | Area | Finding | Verdict |
|---|---|---|---|
| D1 | `pipeline.rs:272` | `apply_rewrite`'s match grows per `(template, event)` pair. Is there a shape that does not, or is the hit the right trade? | `design` |
| D2 | `gather.rs:344` | `any_replacement_counter` scans every battlefield entity's `counters` map on **every proposed action**. Cheap at 15 permanents and 3 counter kinds; measure before the pool grows. The comment claims a cached count would drift — that is true of a *count*, but not of a set maintained at the same two chokepoints `replacement_ability_sources` uses. | `defer` (measure first) |
| D3 | `gather.rs:330` | **`REPLACEMENT_COUNTERS` may be short.** CR 122.1b keyword counters grant keywords, and some keywords are replacement-shaped. Audit all fifteen (122.1b) against CR 614 before trusting the list at three. *(First pass: none of the twelve modelled keyword counters is a CR 614 replacement — indestructible is a "can't", lifelink is CR 120.3f — but that reasoning is nowhere in the code.)* | `fix` (document the audit) |
| D4 | `gather.rs:396` | **Why are shield counters two `CounterEffectKind`s when one counter does both?** Because CR 122.1c literally creates two effects with two CR 614.5 identities — but that is not obvious, and the split looks like it defeats the purpose of one counter. Needs the rule quoted at the enum. | `doc` |
| D5 | `gather.rs:11` | **§3.3 source 2 (static abilities functioning in other zones) is deferred past RE — should the phase be able to close without it?** Wonder, Bridge from Below, every "from your graveyard" replacement. A phase called "replacement effects" that cannot see a graveyard source is arguably incomplete. RB was also asked to *size* this and did not (K3). | `design` |

---

## E. Engine hygiene

| # | Area | Finding | Verdict |
|---|---|---|---|
| E1 | `pipeline.rs:72` | `MAX_616_1F_ITERATIONS` is documented as "a test harness" and lives in engine code. Either it is a real invariant (then say so without calling it a harness) or it belongs behind `#[cfg(test)]` / a debug assertion. | `fix` |
| E2 | `game_state.rs:589` | `remove_from_combat` (CR 506.4) is combat code that arrived in a replacement-effects PR. It is genuinely needed by CR 701.19a's rider, but it is the kind of thing a split would have kept out. | `defer` (note only) |
| E3 | `object.rs:44` | `zone_change_epoch` + two `GameState` counters for one consumer (CR 704.6d). Justified as a *fact* rather than a feature, and `codebase-state.md` item 10 wants the same field for CR 400.7 — but with one live use it is worth re-confirming the plumbing earns its place. | `defer` |
| E4 | `resolve.rs:659` | **`RemoveFromCombat` / `RemoveAllDamage` write `BattlefieldEntity` directly, justified as "no card replaces this".** Is that safe futureproofing? A card saying "creatures you control can't be removed from combat" is plausible, and the design's own premise is that any game event should be replaceable. | `design` |

---

## F. Rules questions that need research before code

| # | Area | Finding | Verdict |
|---|---|---|---|
| F1 | `actions.rs:607` | **Skullbriar, the Walking Grave** — counters stay on it as it changes zones, against CR 122.2. The `AddCounters` performer errors for a non-battlefield object. How much does honouring this change the counter model, and is it worth it? | `design` |
| F2 | `ui/ask.rs:586` | **Partner commanders dying to one board wipe.** The CR 704.6d sweep collects every eligible commander and asks per commander, so it should fire twice — but there is no test, and the UX question (one prompt at a time vs. one combined) is undecided. | `fix` (add the test) |
| F3 | `pipeline.rs:202` | **"If you do" clauses vs. unconditional riders.** RB queues riders unconditionally, citing CR 615.12. That rule is about *prevention*. Academy Rector's "you may exile it. **If you do**, search…" is conditional on its first half actually happening. Is the unconditional rule right for non-prevention replacements? | `design` |
| F4 | `types/replacement.rs:74` | **Is `then: Option<Effect>` rich enough?** A rider like "you may put a +1/+1 counter on each of them. If you don't, draw a card" needs `Effect::Optional` (unimplemented) plus a conditional on the optional's *outcome* (no vocabulary at all). | `defer` |
| F5 | `gather.rs:100` | **Should CR 614.15 self-replacement effects be authored or derived?** The plan assumes authored (a `ReplacementClass` a card sets). The analogy to CDAs and CR 605.1 mana abilities suggests *derived* — 614.15's "self-replacement" is a property of where the effect comes from, not a label an author picks. Decide before the first card. | `design` |
| F6 | `actions.rs:282` | **CR 101.4d's restart is not implemented**, and B3 downgrades the claim that it cannot arise. The Notion-Thief-vs-Notion-Thief case (the chooser *changes* as the event is rewritten) **is** handled — `chooser_for` is recomputed every iteration. What is not is interleaving several batch members' choices rather than running each member's loop to completion. | `defer` |
| F7 | `zones.rs:67` | `ZoneChangeCause::Returned` groups "return to hand" and "return to the battlefield". Mechanically distinct despite the shared word; check whether any card distinguishes them as a *cause* before keeping the merge. | `design` |

---

## G. Process — how the codebase and its docs are written

**Do this theme first.** It changes how every other fix gets written.

| # | Area | Finding | Verdict |
|---|---|---|---|
| G1 | `CLAUDE.md` | **Over 300 lines, twice now.** "Never prompt a `DecisionProvider` with fewer than two candidates" does not need a paragraph in the top-level instruction file. The file needs a *budget with a mechanism*, not another reminder to keep it short — see the proposal below. | `fix` |
| G2 | everywhere | **Too many comments.** Comment density across RB is far above the surrounding code. Needs a stated rule, not a vibe — see the proposal below. | `fix` |
| G3 | `phase_rb_cards.rs:79` | **Kalitas should probably be registered, and the stated reason for not registering it is wrong.** The fuzz pool exists to stress effect interactions and find panics; "it would move the baseline" is an argument for *separating the pools*, not for keeping cards out. Proposal: a frozen **performance pool** (today's 55 cards, for A/B-ing engine changes) and a growing **stress pool** (everything, for panics and interaction coverage). `fuzz_games` takes a flag. | `fix` |

### G1 proposal — a budget with a mechanism

The rule that has failed twice is "keep it durable". A rule that can fail
silently is not a mechanism. Suggested replacement, to be decided in the fix
session:

- **Hard cap: 200 lines**, checked by a script the `Commands` block names, so
  exceeding it fails the way a warning fails.
- **Every invariant is at most three lines plus a pointer.** The statement of
  the rule lives in `CLAUDE.md`; the reasoning, the war story and the rule
  numbers live in the architecture doc. RB's "replacement pipeline" section is
  33 lines and should be about 6.
- **Adding a section requires removing one.** The file describes the *current*
  shape of the project, and a project does not accumulate invariants forever —
  it replaces them.

### G2 proposal — a comment rule

Suggested, to be decided in the same session:

- Comment the **why**, and only where the why is not recoverable from the code
  plus one rule number. `// CR 701.26a — only untapped permanents can be tapped`
  earns its place; six lines re-narrating what the next four lines do does not.
- **A war story goes in the commit message or the architecture doc, not the
  source.** "This was the shape indestructible had before Phase RB" is history;
  the source needs the rule.
- **One rule cite beats a paragraph.** If a comment needs more than ~4 lines,
  that is a signal the reasoning belongs in `plans/` with a one-line pointer.

---

## Answered inline, not logged as findings

These came up in review, have answers, and need no code change. Recorded so the
same question is not asked twice.

- **`.map(|_| ())`** — `execute_actions` returns `Result<Vec<GameAction>, _>` and
  these two callers want `Result<(), _>`. `map` transforms the `Ok` payload and
  passes `Err` through untouched; `|_| ()` discards the vector. It is "same
  error, no value".
- **`battlefield_ids_ordered`** — the battlefield is conceptually unordered, but
  a `DecisionProvider` picks from a list *by index*, so any sweep that reaches a
  decision has an observable order, and `HashMap` iteration order differs per
  process. The ordering key is `BattlefieldEntity::timestamp`, which is CR
  613.7's order anyway.
- **`retain_effects` is called** — three times, by `remove_by_source`,
  `remove_expired_at_cleanup` and `remove_expired_at_turn_start`. It keeps
  everything matching a predicate and returns the rest; the doc comment leads
  with an `O(n²)` aside, which is why it reads as "drops all but one". Fold into
  C-theme clarity work.
- **The decline path does not double-insert.** `applied.insert` then
  `declined.insert` then `continue` — the second `applied.insert` further down
  is on the path where the effect was *accepted*. `HashSet::insert` is
  idempotent regardless.
- **"Inheritance" in `pipeline.rs:84`** is CR 614.5's applied-set being passed
  down to a decomposed event, not OO inheritance. The parameter is
  `inherited: &HashSet<...>`. Rust is not being fought; the word is overloaded.
  Rename to `carried` or `parent_applied` under theme C.
- **`unwrap_or(true)` in `pattern_matches`** is safe and is the "no constraint"
  case: each field of `EventPattern::ZoneChange` is an `Option`, `None` means
  "any", so `None.map(..).unwrap_or(true)` reads "unconstrained fields match".
  The nested one is different and worth a comment: `permanent_matches_filter`
  returns `Result`, and its `unwrap_or(false)` means "an object we cannot
  evaluate does not match" — a *failure* defaulting closed, not a "no
  constraint" defaulting open. Two `unwrap_or`s with opposite meanings three
  lines apart.
- **`forced_bucket`** implements CR 616.1a–e. `ReplacementClass` derives `Ord`
  in the rule's own order (`SelfReplacement` < `ControlChanging` < `CopyOnEnter`
  < `BackFaceUp` < `Other`), so `.min()` over the candidates' classes finds the
  highest-priority class that *anything is in* — which is precisely what
  616.1a–e's "if any … one of them must be chosen; if not, proceed to" ladder
  says. The filter then keeps only that class. `Other` is 616.1e's fallthrough
  and needs no special case because it sorts last. Worth rewriting as an
  explicit ladder for readability even though the `min()` is correct.

---

---

## The plan — what happens in what order, and what blocks the merge

**Read this before starting a session.** It exists so that a cold session knows
whether its theme is on the branch or after it, and so nobody re-derives the
ordering.

### What the review actually found

**No confirmed correctness defect.** 744 tests green, zero warnings, `fuzz_games`
identical to the pre-RB baseline, and the three RA invariants still grep-provable.
Of 36 numbered findings: **one cosmetic defect** (B6, a malformed error string)
and **one untested path that might be one** (F2, Partner commanders). Everything
else is naming, comment accuracy, process, or a design question about work that
has not started.

That distinction is the whole plan. **Legibility debt is recoverable; a wrong
pipeline would not have been.** The reason it does not *feel* recoverable is
specific and fixable: twelve new types landed at once with no map (see "the
as-built map" below).

### On the branch, before #62 merges

Two sessions. Both are small, neither touches behaviour, and both are things it
would be actively bad to ship into `main` as-is — this project's docs are
load-bearing, so a wrong comment is a wrong doc.

1. **Theme G — process.** First, because it changes how every later fix is
   written: G2's comment rule is what themes C and E apply as they go. G3
   (register Kalitas, split the fuzz pool in two) is a code change with a
   baseline consequence and can move to its own PR if it grows.
2. **Theme B — doc and comment accuracy.** Seven rows, all local, zero test
   impact. B6 is the one line of code.

Then **merge #62**.

### After the merge, as follow-up PRs on `main`

3. **Themes C + E — naming and hygiene.** One PR. Mechanical renames plus
   `MAX_616_1F_ITERATIONS`'s home and `mod.rs`'s shape. Do it with theme G's
   comment rule in hand, and it will shrink the file it touches.
4. **Themes D + F — scaling and rules questions.** Mostly `defer` entries in
   `codebase-state.md` rather than code. F2's test is the exception and is
   worth pulling forward into step 3.

### Theme A runs in parallel, and it has a deadline — ✅ the design half is done

The "can't" model is a design document, not a fix session, and it does **not**
block #62 or RC-1 through RC-3. It **does** block **RC-4**, which carries
CR 614.17d ("can't" effects that modify how a permanent enters). So:

> **A can start any time and must land before RC-4.**

That is the only hard ordering constraint the review created.

**Landed 2026-08-27, docs only, on its own branch off #62's base.**
`plans/cant-effects-architecture.md` + `plans/references/cant-census.py`. The
deadline is now specific: **RS-1** (the Tier-2 spine — a net-deleting PR) is
what RC-4 needs, and RS-2/RS-3/RS-4 are independent of RC/RD/RE entirely.
RS-3 (combat, 62% of the corpus) additionally wants the CR 613.8 dependency
cluster first, because CR 509.1b's evasion is cumulative and a solver reading
effective characteristics under timestamp-only ordering will change answers
when 613.8 lands.

### The as-built map — the antidote to "I lost the mental model"

Twelve types landed in one PR with no summary of what they are, which is a real
cost of the size and not a personal failing. The fix is one short **as-built**
section in `replacement-architecture.md` — the doc that is already the authority
— giving each shipped type one line, marking which are **closed** (`Rewrite`,
`ReplacementClass`, `Uses`) and which are **growing** (`EventPattern`,
`GameActionTemplate`, `ZoneChangeCause`), and tracing one proposed action end to
end from `execute_actions` to a performed `GameEvent`.

It resolves C7 at the same time: `engine/replacement/mod.rs` currently carries
that essay, and it should carry a pointer.

**Write it as part of theme B**, since it is the same kind of work and the same
kind of session.

---

## Notes for whoever fixes these

- **`git checkout --` on a file with uncommitted work loses it.** This bit
  during RB's mutation checks. Copy the file aside instead.
- **Mutation-check any assertion a fix adds or changes.** RB shipped one vacuous
  test that a mutation caught.
- **A `COVERS` link is a claim.** If a fix changes what a test proves, re-read
  the atom with `specdb show`. RB's own review pass found ten overstated links
  and two wrong ones.
- **The integration tests and doc changes in #62 were not reviewed**, on the
  expectation that this ledger will move some of them. Re-review them after
  themes B, C and G land.
