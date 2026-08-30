# PR #62 (Phase RB) — review findings ledger

**Delete this file when the last row is closed.** Per `CLAUDE.md`'s authority
table, a handoff is where to resume half-finished work and does not outlive it.

## Start here — the session queue as of 2026-08-30

Everything below this block is the *record*; this block is the *plan*. A cold
session takes the top undone line, reads only what that line names, closes it,
re-runs the gates once, and checks the line off. No session needs the whole
file and none needs any conversation.

1. ~~Commit this file's second audit onto `replacement/rb-pipeline`~~ ✅ 2026-08-30.
2. ~~Fix PR #64's findings and merge it into `replacement/rb-pipeline`~~
   ✅ 2026-08-30 — V1–V6 closed in `b64508f`, merged in `f142a74`.
3. ~~**The #62 pre-merge session**: H2 + I1–I9~~ ✅ 2026-08-30. All ten rows
   closed, plus **I10** (found while fixing I4 — see theme I). One row did not
   reproduce: **I7's third claim**. `specdb owed` prints **38** on this tree,
   before and after this session's changes and after a `specdb build`, so
   `codebase-state.md`'s "unchanged at 38" is right and was left alone.
   **#62 merged to `main` as `78d344c`.**
4. ~~Post-merge, on `main`: **themes C + E**~~ ✅ 2026-08-30, branch
   `replacement/rb-naming` (`cd83889`, `69a805e`). All eleven rows closed; no
   behaviour moved and both fuzz baselines are byte-identical. Two things came
   out of it that were not in the ledger: E4's answer found **seven cards that
   restrict the CR 514.2 cleanup damage wipe**, which has no enforcement point
   (`codebase-state.md` item 20), and the census cannot see restrictions
   phrased "isn't"/"doesn't" — 248 cards print "doesn't untap during …" — which
   is theme J's, logged as **J5**.
5. ~~**H1, H3–H6** — the latent code fixes~~ ✅ 2026-08-30, branch
   `replacement/rb-naming` (`068dd03`, `1fe1f8e`). All five closed. Two tests
   added, both shown failing against the pre-fix tree. Both fuzz baselines are
   byte-identical, 750 tests, zero warnings, `specdb owed` still 38. One row
   came out better than triaged: **H3 is closed by CR 400.3**, which makes the
   deleted guard dead forever rather than dead-today, so no field is owed on
   `ZoneChange` (`codebase-state.md` item 21). H1's fix also covers
   ATOM-614.7a-001 partially, which moves 614.7a out of the RD column of the
   slice measurement in `codebase-state.md`.
6. Then **themes D + F, plus H7–H9, J3–J5 and M1** (defer entries and design
   questions; F2's test rides along; J1/J2 are two `cant-census.py` regex
   fixes and can ride either session; **M1 is a 94-site mechanical rename and
   wants its own commit**, so it can ride any session but not be folded into
   one).
7. Then stop reviewing and **build**: RS-0, per `cant-effects-architecture.md`
   §7.1's queue. That list is the ordering authority from here on.

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
*(Superseded: A, B, C, E, G and I are closed; the live plan is "Start here" at
the top of this file. E was not "cheap, mechanical, no design input" — E4 was a
card-pool measurement, which is the one shape this ordering mis-predicted.)*

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

**What the owner's review of the design changed** (2026-08-27, same day):
CR 608.2c settles the "can't be regenerated" scope and makes it an *authoring*
concern rather than an inferred one; §4.2's combat solver is counted rather
than deferred (**1,249 of 1,262** Tier-1a clauses are a per-creature predicate)
and its algorithm is now an O(n log n) count-threshold sweep with brute force
demoted to the fallback, which is why RS-3 split into 3a/3b; CR 601.3a is not a
forward-looking search at all — Void Winnower's rulings make it
over-approximate-at-the-gate plus a CR 601.2 rewind; **§4.9 is new** — Sigarda's
rulings and CR 608.2d make "do not prompt for a choice a restriction forbids" a
rule rather than a UX nicety, and it lands in RS-1; and §9 finding 7's registry
work is **composition, not a split**, with a stated abort condition.

**One thing this design deliberately does not lean on:** `MAX_616_1F_ITERATIONS`
as precedent for a bounded-search guard. **E1 below is still open** and that
constant may genuinely be a test harness; §4.2 says so explicitly rather than
citing a decision nobody has made.

**The deadline is now specific.** "A must land before RC-4" resolves to **RS-1
must land before RC-4** — the Tier-2 spine only, which is a *deleting* PR
(three call sites replaced, one `GameState` field and one `turns.rs` clear
removed). RS-2/RS-3/RS-4 block nothing in the RC/RD/RE line.

---

## B. Doc and comment accuracy — the code is right, something written is wrong — ✅ **CLOSED 2026-08-29**

Cheapest theme. No tests move, no re-measurement.

| # | Area | Finding | Verdict |
|---|---|---|---|
| B1 | `actions.rs:162` | **"CR 701.8b makes 'destroyed' a strictly narrower fact than 'put into a graveyard from the battlefield'" is wrong.** Destroy is an *action*, not a description of a result: under Rest in Peace a destroyed creature never reaches a graveyard, and Karmic Justice still triggers. The two are overlapping, not nested. **The code is right** — `Destroy` being the outer event with `ZoneChange` inside it is exactly what makes the Rest in Peace case expressible — but the comment argues for it from a false premise. | ✅ `doc` — rewritten to argue from **CR 122.1c vs 122.1h**, which is the CR drawing the same line itself (shield replaces "would be destroyed", finality replaces "would be put into a graveyard from the battlefield"). Overlap stated, nesting dropped. Deliberately *not* sourced on Rest in Peace: whether a destroyed-then-exiled permanent counts as destroyed is a live question the rulings do not settle, and the finality counter makes the point from a mechanic RB already ships |
| B2 | `gather.rs:228` | **Factually wrong: "the shield is still not spent, and is still there for a later turn."** CR 701.19a shields last *this turn* — `Duration::UntilEndOfTurn`, which the code sets correctly. Same sentence appears in the CR 701.19c test's comment. | ✅ `doc` — now "there for a later destruction **this turn**", with `Duration::UntilEndOfTurn` named. **The second half of the finding was wrong:** the CR 701.19c test's comment says "still in the registry afterwards, unspent" and never claims a later turn. Nothing to fix there |
| B3 | `actions.rs:282` | **The CR 101.4d justification is overstated.** The comment claims a restart is "unreachable in this shape" because `consume_use` only removes candidates. The real reason it has not come up is narrower: nothing in phase 1 can *create* a replacement effect, and each batch member's 616.1f loop runs to completion before the next begins — which is a simplification, not a proof. See F6 for the deferred part. | ✅ `doc` — downgraded from "unreachable" to "not implemented, and here is the narrower reason it has not come up", pointing at F6 |
| B4 | `zones.rs:115` | `Fizzled`'s doc says "countered by game rules". Current CR 608.2b does not use that phrase — the spell "doesn't resolve" and is removed from the stack. Fizzling is not countering. | ✅ `doc` — CR 608.2b's own wording, and the same stale phrase fixed in `replacement-architecture.md`'s `ZoneChangeCause` listing |
| B5 | `types/replacement.rs:333` | The "Why there is no `CounterBacked`" history is design archaeology and belongs in `replacement-architecture.md` §3.2, where it already is. The code needs the rule, not the story. | ✅ `doc` — 24 lines down to 7, ending in a pointer. Verified the full history is at `replacement-architecture.md`:338 before cutting |
| B6 | `resolve.rs:83` | **Malformed string literal.** The `Effect::Replacement` error message is one long line with runs of ~18 literal spaces — a line-continuation that did not survive how the file was written. Cosmetic, but it is what a user sees. | ✅ `fix` — real `\` continuations |
| **B7** | 6 `.rs` + 3 `.md` sites | **Found while fixing B1: indestructible is cited as CR 701.8a throughout, and CR 701.8a is "to destroy a permanent, move it from the battlefield to its owner's graveyard".** The rule that makes indestructible a "can't" is **CR 702.12b**. Nine sites said 701.8a where they meant 702.12b — `actions.rs`, `pipeline.rs`, `resolve.rs`, `sba.rs`, `artifacts.rs`, `phase_rb_integration_test.rs`, `cant-effects-architecture.md`'s census table, `replacement-architecture.md` ×2. The three sites that cite 701.8a *for the destroy action* are correct and were left alone | ✅ `doc` — all nine corrected |
| **B8** | `CLAUDE.md` | **Found while writing §2a: "a sixth arm" is wrong for `Rewrite`, which ships two** (`Prevent`, `Instead`). §3.2b designed five; the growth contract was written against the design, not the code | ✅ `doc` — "a new arm", plus a pointer to §2a |

**The as-built map is in** — `replacement-architecture.md` §2a. Every shipped
type on one page with a Growth column, plus one action traced from
`execute_actions` to a performed `GameEvent`. `engine/replacement/mod.rs`'s
module doc now points at it; C7's code move (types out of `mod.rs`) followed
on 2026-08-30 — `engine/replacement/instance.rs`.

---

## C. Naming and semantics — ✅ **CLOSED 2026-08-30** (`cd83889`)

Rows here share one cause: RB introduced a lot of vocabulary quickly and some of
it collides with vocabulary that already existed.

**All seven closed in one commit, no behaviour.** The theme resolved into one
sentence the code now reads as: an *event* has a **subject**, an *effect*
**watches** a pattern and **affects** a set. Two rows landed differently from
what the ledger proposed, both recorded here rather than silently:

- **C2 kept `ReplacementDef.affected`.** The row read as "rename and/or fix the
  doc"; the name is the same question `ContinuousEffect.affected` asks, and
  renaming one of that pair would have re-created C1's collision on a different
  axis. The doc is what was wrong, and it is fixed: CR 614.1's shield is a
  metaphor for *every* replacement effect, but in this codebase "shield" is
  taken by CR 701.19a and CR 122.1c, which is the actual defect.
- **C6 used `affects`, not `scope_contains`.** The row said "something like
  `scope_contains` / `watches`"; `affects` is the CR's own word and the one
  `AffectedSet` is already named for, so `applies_to` = `watches(…) &&
  affects(…)` needs no third vocabulary.

| # | Area | Finding | Verdict |
|---|---|---|---|
| C1 | `gather.rs:57` | **`Affected` and `AffectedSet` are two different ideas one letter apart.** `Affected` is *what a proposed event is about*; `AffectedSet` is *the boundary of an effect's shield*. Reading them side by side implies a relationship that does not exist. Rename one. | `fix` ✅ — `Affected` → `EventSubject`, `affected_of` → `subject_of`, `Rider.affected` → `Rider.subject` |
| C2 | `types/replacement.rs:45` | Reusing `AffectedSet` for `ReplacementDef.affected` reads as too narrow for how broad the name is — the doc calls it "which objects it shields", which is regeneration/shield-counter language applied to every replacement effect. Kalitas does not shield anything. | `fix` ✅ — the doc, not the name; see the note above |
| C3 | `replacement_effects.rs:44` | `ReplacementRowId` / `RegisteredReplacement` / `ReplacementRegistry` read as "replacing a row in some other registry". Tack on `Effect`: `ReplacementEffectId`, `RegisteredReplacementEffect`. | `fix` ✅ — `ReplacementEffectId` / `RegisteredReplacementEffect` / `ReplacementEffectRegistry`, the last now parallel to `ContinuousEffectRegistry` |
| C4 | `actions.rs:350` | `resolve_rider` is documented as "one CR 615.5 rider", but 615.5 is about *prevention* effects only. Kalitas's token is a rider on a plain replacement. Find the right citation (CR 614.1a's "instead" clause plus the effect's own text) or drop the rule number. | `doc` ✅ — CR 614.1a + 614.6 for a rider on a plain replacement; 615.5 kept where the shield counter's *prevention* half queues one |
| C5 | `pipeline.rs:134` | `candidates` is used for two different things a line apart — the gathered-and-filtered list, and then `bucket` after `forced_bucket`. Name the first `applicable` and the second `bucket`, or fold the filter into `gather`. | `fix` ✅ — `applicable`, then `bucket` |
| C6 | `gather.rs:246` | **`applies_to` checks "does the pattern match" AND "is the affected object inside the shield", and the second half is named `shield_contains` on effects that shield nothing.** The function is correct — an effect applies only if it watches this *kind* of event *and* this event is about something in its scope — but the naming makes it read as two unrelated checks. Rename to something like `scope_contains` / `watches`. | `fix` ✅ — `watches` / `affects`; see the note above |
| C7 | `replacement/mod.rs:1` | Every other `mod.rs` in this project is a re-exporter. This one carries type definitions and a module-level essay. Move `ReplacementInstance` / `ReplacementInstanceId` into a file. | `fix` ✅ — `engine/replacement/instance.rs`; `mod.rs` is a re-exporter pointing at §2a |

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

## E. Engine hygiene — ✅ **CLOSED 2026-08-30** (`69a805e`)

**Four rows, and E1 took three passes to land.** It got a decision first (the
cap is a backstop, not a harness), then — on the owner's reading of that answer,
that the *doc* was long because the *design* was weak — a check for the property
CR 903.9b actually has, and then, on the same objection repeated, the deletion
the argument had earned all along. E2 and E3 are `codebase-state.md` items 18
and 19.
E4 asked whether "no card replaces this" is safe futureproofing and got **two
different answers** — sound for removal from combat, false for the analogy the
damage comment leaned on — plus, on the same review, the missing half of the
argument: *what it costs to be wrong*, which is what makes the deferral sound
rather than lucky. §8a carries both.

**The lesson worth keeping from E1.** A long defensive comment is evidence about
the code under it. The first answer was accurate and the reviewer was still
right to reject it, because "bounded by nothing else" is a fact about the design,
not about the prose — and the second answer was better and *still* left a
number in the tree, which the same objection removed. The finished shape is one
check per way the argument can fail, each with its own error, and no budget.

| # | Area | Finding | Verdict |
|---|---|---|---|
| E1 | `pipeline.rs:72` | `MAX_616_1F_ITERATIONS` is documented as "a test harness" and lives in engine code. Either it is a real invariant (then say so without calling it a harness) or it belongs behind `#[cfg(test)]` / a debug assertion. | `fix` ✅ — **the constant is deleted and the loop is uncapped.** Three answers, each rejected for the right reason. (1) "It is a bug backstop, not a test harness" — true, and the length of the comment defending it was the tell that the design under it was weak. (2) `check_exempt_terminates` enforcing the property CR 903.9b actually has (its rewrite leaves its own pattern), which left the cap covering two-exempt ping-pong. (3) That case is a check too — at most one exemption applies to one event — so every leg of the argument is now enforced where it can fail and the budget has nothing left to do. `MAX_616_1F_ITERATIONS` is gone; `apply_replacements` is a plain `loop`. Two tests pin the two failure modes, both shown failing against the capped tree. `cant-effects-architecture.md` §4.2 no longer cites it as precedent — it cites how it died |
| E2 | `game_state.rs:589` | `remove_from_combat` (CR 506.4) is combat code that arrived in a replacement-effects PR. It is genuinely needed by CR 701.19a's rider, but it is the kind of thing a split would have kept out. | `defer` ✅ — `codebase-state.md` item 18. Recorded so the combat phase *uses* it: it handles CR 506.4 in both directions, and the one-line `entry.attacking = None` version is the one that looks right and is not |
| E3 | `object.rs:44` | `zone_change_epoch` + two `GameState` counters for one consumer (CR 704.6d). Justified as a *fact* rather than a feature, and `codebase-state.md` item 10 wants the same field for CR 400.7 — but with one live use it is worth re-confirming the plumbing earns its place. | `defer` ✅ — `codebase-state.md` item 19. A re-confirm when item 10's CR 400.7 lands, not debt: recording an unrecoverable fact is right, and if 400.7 does not use the field the question becomes live |
| E4 | `resolve.rs:659` | **`RemoveFromCombat` / `RemoveAllDamage` write `BattlefieldEntity` directly, justified as "no card replaces this".** Is that safe futureproofing? A card saying "creatures you control can't be removed from combat" is plausible, and the design's own premise is that any game event should be replaceable. | `design` ✅ — **combat: sound.** CR 506.4 is a *consequence* of seven causes, six already proposed, so a "can't" attaches to a cause; all 25 printed "from combat" cards cause removal and nothing watches it. **Damage: the analogy was false.** Seven cards restrict the CR 514.2 cleanup wipe the comment cited as precedent — item 20, one filter once RS-1's sweep exists. The tripwire for both is a *trigger*, not a replacement |

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

## G. Process — how the codebase and its docs are written — ✅ **CLOSED 2026-08-29**

**Do this theme first.** It changes how every other fix gets written.

All three proposals were adopted as written. The reasoning that left `CLAUDE.md`
landed in a new doc, **`plans/engineering-practices.md`**, which is now the
authority for process rules and has a row in `CLAUDE.md`'s authority table.

| # | Area | Finding | Verdict |
|---|---|---|---|
| G1 | `CLAUDE.md` | **Over 300 lines, twice now.** "Never prompt a `DecisionProvider` with fewer than two candidates" does not need a paragraph in the top-level instruction file. The file needs a *budget with a mechanism*, not another reminder to keep it short — see the proposal below. | ✅ `fix` — 312 → 195 lines, no section added or dropped. `plans/check_claude_md.py` enforces the 200-line cap (exit 1, per-section table, `--budget N` to raise it deliberately) and is named in the `Commands` block. Rules in `engineering-practices.md` §1 |
| G2 | everywhere | **Too many comments.** Comment density across RB is far above the surrounding code. Needs a stated rule, not a vibe — see the proposal below. | ✅ `fix` — stated in `engineering-practices.md` §2, with a one-bullet statement under `CLAUDE.md`'s Conventions. **Deliberately not applied retroactively:** themes C and E bring the existing density down file by file, as those files are touched anyway |
| G3 | `phase_rb_cards.rs:79` | **Kalitas should probably be registered, and the stated reason for not registering it is wrong.** The fuzz pool exists to stress effect interactions and find panics; "it would move the baseline" is an argument for *separating the pools*, not for keeping cards out. Proposal: a frozen **performance pool** (today's 55 cards, for A/B-ing engine changes) and a growing **stress pool** (everything, for panics and interaction coverage). `fuzz_games` takes a flag. | ✅ `fix` — Kalitas registered; `PERFORMANCE_POOL` (frozen 55) and `default_registry` (everything); `fuzz_games --pool performance\|stress`, default `performance`, pool named in the header and the results block. Both baselines recorded in `engineering-practices.md` §3 |

**What the split bought immediately.** Kalitas now runs under
`card_pool_lowering_test` (the check that a static ability lowers at all, which
an unregistered card escapes) and under `cli_play`. 50 stress games at seed
12345: zero panics, zero errors, 202 Zombie tokens, and creature deaths down
5.6 → 5.2 because opponents' creatures are exiled instead. The performance
column is byte-identical to the RB baseline, which is the acceptance test for
the split.

**One decision the proposal left open, made here: `performance` is the
default.** Every baseline recorded in `plans/` was measured on it, and the
review's own re-run gate (`fuzz_games --games 50 --seed 12345`) is a regression
check, so the bare command has to keep reproducing the recorded numbers.
`--pool stress` is the deliberate "play everything" mode, and `CLAUDE.md`'s
`Commands` block names it so it does not rot.

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

1. ~~**Theme G — process.**~~ ✅ **done 2026-08-29.** G1's budget script, G2's
   comment rule and G3's pool split all landed, plus
   `plans/engineering-practices.md` as their home. It stayed one session and did
   not need its own PR. Gates re-run: 746 tests (744 + the two new registry
   tests), zero warnings, performance-pool fuzz byte-identical to the baseline.
2. ~~**Theme B — doc and comment accuracy.**~~ ✅ **done 2026-08-29.** Six rows
   closed, two new ones found and closed (B7's nine wrong rule cites, B8), one
   half of a finding refuted (B2's second sentence). The as-built map landed as
   `replacement-architecture.md` §2a. 746 tests, zero warnings, no behaviour
   touched.

Then **merge #62** — both blocking themes are closed. *(Superseded 2026-08-30:
the second audit below added pre-merge rows. The live plan is "Start here" at
the top of this file.)*

### After the merge, as follow-up PRs on `main`

3. ~~**Themes C + E — naming and hygiene.**~~ ✅ **done 2026-08-30**, branch
   `replacement/rb-naming`, two commits (`cd83889`, `69a805e`). One PR as
   planned, and theme G's comment rule did shrink what it touched. The one
   surprise: E4's "is this safe futureproofing" turned out to be two questions
   with two answers, and the damage half was wrong.
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

---

# Second audit (2026-08-29) — the surfaces the first review did not open

Scope: the 2,002-line integration test file, the doc changes
(`replacement-architecture.md` §2a, `codebase-state.md`,
`engineering-practices.md`, the `CLAUDE.md` trim), the "can't" design doc
against its census script, the three post-review commits, and a fresh
correctness pass over the pipeline aimed at the paths with the least test
pressure. Themes C, D, E and F were left alone per instructions; nothing found
here suggests any of them is triaged wrong. Rows continue the lettering at H.

## What re-verified clean — recorded so nobody re-audits it

- **All gates pass on this tree.** 746 tests (not 744 — see I7), zero warnings,
  `check_claude_md` 197/200, `specdb orphans` clean, `suspicious` one
  pre-existing heuristic false-positive (ATOM-611.2a-001's link predates RB and
  the atom matches the test). Both fuzz baselines reproduce **byte-identically**
  against `engineering-practices.md` §3: performance P0 28/P1 22, spells 20.8,
  deaths 5.6; stress P0 30/P1 20, spells 19.5, deaths 5.2.
- **`PERFORMANCE_POOL` is the pre-split registry, exactly** — the 55 names match
  `56d2072^`'s `register()` calls one for one, and `performance_pool()` panics
  on a missing name. What the freeze does *not* cover: the pools share factory
  functions, so an edit to a card's definition moves the frozen pool's behavior
  silently — membership is guarded by code, behavior only by the fuzz re-run
  gate. That is the design, but say it when citing the freeze.
- **The census's headline number reproduces.** `cant-census.py` prints Tier 2 =
  236 of 2,034 exactly as §2.1 records, tiers sum to the clause total, and the
  §2.6 Sigarda reclassification is real in the buckets. The 1,249/1,262 claim is
  a dated *hand count* recorded in §4.2's decomposition table (1,200 + 49), not
  a script output — see J3.
- **RS-1's deleting-PR shape is accurate against this tree.** All three call
  sites exist (`pipeline.rs:53`, `gather.rs:229`, `resolve.rs:643` — the doc
  says 638, five lines of drift), plus `game_state.rs:269`'s field, its init,
  and `turns.rs:138`'s clear. Q3's answer is: no contradiction between the
  "can't" design and what RB shipped.
- **The `CLAUDE.md` trim dropped no invariant.** Every deleted rule survives
  either compressed in place or in the doc its pointer names (PRE-LAYER's file
  list now lives in `codebase-state.md` and the source tags themselves); the
  deleted text was war stories and examples, which is what the comment rule
  says to cut. `engineering-practices.md`'s two baselines both reproduce.
- **The integration test file is the strongest part of the PR, not the
  weakest.** Both branches of every choice are tested, absence is asserted by
  unscripted-provider panic, timing is asserted on the event log where end
  states tie, and the "atom scenarios built in full" section builds
  ATOM-616.1-001's printed scenario exactly. One test pins wrong behavior (H2);
  none is vacuous.
- **Kalitas's Oracle text matches the card file verbatim** (Scryfall,
  2026-08-29), and the shield-counter rulings confirm the engine's CR 122.1c
  reading on every point except the two H9 raises.

## H. Pipeline and SBA correctness — second pass

**H2 closed 2026-08-30** (the pre-merge session). **H1 and H3–H6 closed
2026-08-30** in `068dd03` / `1fe1f8e`, their own session as planned. H7–H9
remain, and are `defer`/`design`. Note the line numbers in the rows below
predate 2026-08-30's renames and the H1/H4 fixes; they have drifted.

The first review found no correctness defect. This pass found one reachable
family and one test-pinned wrong behavior, both currently latent — nothing in
the registered card pool can trigger either, which is why 746 tests and two
byte-identical fuzz baselines coexist with them.

| # | Area | Finding | Verdict |
|---|---|---|---|
| H1 ✅ | `actions.rs:415`, `resolve.rs:132` | **The 0-damage check sits on the wrong side of the pipeline.** `perform_action`'s `DealDamage` arm returns early on `amount == 0`, correctly citing CR 614.7a — but that arm runs *after* `apply_replacements`, so a 0-damage proposal traverses the pipeline first. `EventPattern::DealDamage` has no amount constraint, so a shield counter's prevention half applies to it, and its rider removes a shield counter for an event CR 614.7a/120.8 says "never happens" and "replacement effects … have no event to replace". Combat can't produce it (CR 510.1a filter at `resolution.rs:55`), but `Primitive::DealDamage` does no filtering, so any X=0 or computed-0 damage effect triggers it the day one is registered alongside a shield-counter producer. Same shape, lower stakes: `GainLife`/`LoseLife`/`AddCounters` 0-amounts also traverse (no patterns match them today). The proposal side, not the perform side, is where CR 614.7a lives. **Closed 2026-08-30 (`068dd03`):** `replacement::never_happens` gates the CR 616.1 loop, asked per iteration like `is_blocked` because 616.1f can rewrite an event into a 0. Life gain joined it on CR 119.10's own words ("no life gain event would occur, and these effects won't apply"); life loss and 0-count counter changes have no such rule and keep their no-op guards in `perform_action`, which the helper's doc says. `replacement-architecture.md` §4.1's "already short-circuited in `perform_action`" — the sentence that made this invisible — is corrected, and §2a's trace gained the step. | `fix` ✅ |
| H2 ✅ | `zones.rs:369`, `replacement_effects.rs:108`, `phase_rb_integration_test.rs:1266` | **`replacement_effects.remove_by_source` at battlefield-leave implements a rule that does not exist, cites CR 611.2a for the opposite of what it says, and a test pins it.** The registry holds *only* resolution-created rows (its own module doc says so), and CR 611.2a says exactly those "last as long as stated" — a shield does not die with the permanent whose ability made it (611.3b is the static-ability rule, and no registry row is one). Production never reaches the call: `ResolutionContext.source` for anything on the stack is the stack object (a spell, or `activate_ability`'s ephemeral object), which is never on the battlefield — so the code is dead except from tests that hand-build a battlefield `ResolutionContext`, which is what `test_a_shield_dies_with_the_permanent_that_made_it` does, asserting the CR-wrong outcome under a comment citing 611.2a. Delete the test and the call (unspent shields already expire via `remove_expired_at_cleanup`), or re-scope the call to a future `WhileSourceOnBattlefield` duration with the right cite. Note the *adjacent, pre-existing* miscite it was copied from: `cleanup_zone_state`'s continuous-effects line also cites 611.2a where it means 611.3b. **Closed 2026-08-30:** the call is deleted and the adjacent 611.2a→611.3b miscite fixed; `remove_by_source` keeps its unit test and gains a doc saying it has no production caller and why. The test was **inverted rather than deleted** — `test_a_shield_outlives_the_permanent_that_made_it` now pins CR 611.2a's actual outcome, which is what stops the next session re-adding the call. `codebase-state.md` Deferred Migrations **item 17** records what the deletion leaves owed: no hook expires a row whose duration is source-scoped. | `fix` ✅ |
| H3 ✅ | `gather.rs:518,554` | **`commander_zone_replacement`'s owner guard is a tautology.** `owner_of_destination` ignores the proposed action entirely and returns `game.objects[object].owner` — so the check is `obj.owner != obj.owner`, always false. The comment claims that "the day an effect puts a card into a *different* player's library this becomes the check that stops 903.9b firing" — it cannot: `ZoneChange` carries no destination player, and this function has no input from which to learn one. Correct today only because `add_to_zone_collection` files by owner unconditionally; the guard is dead code documented as live insurance. Make it honest: delete the guard and state the by-construction argument, or give `ZoneChange` the field the day it needs one. **Closed 2026-08-30 (`068dd03`), and the CR closes it harder than the row asks:** CR 400.3 — "if an object would go to any library, graveyard, or hand other than its owner's, it goes to its owner's corresponding zone" — so the day the comment promised cannot arrive and `ZoneChange` is never owed the field. Guard and helper deleted, 400.3 stated in their place, recorded as `codebase-state.md` item 21 so nobody re-adds it. | `fix` ✅ |
| H4 ✅ | `sba.rs:152–190` | **The CR 704.6d moves are performed one `change_zone` at a time, mid-check, outside the batch.** The deaths in the same function are gathered and performed as one `execute_actions` batch under a comment citing CR 704.3's "simultaneously as a single event"; the commander offers are asked *and performed* one by one during the gathering phase, so each is its own batch and a later owner's decision is made against a board an earlier move already changed — the exact decide/perform interleaving the file's own doctrine forbids. Mechanical fix: collect the accepted moves, then one `execute_actions` batch alongside (or with) the deaths. Pairs naturally with F2's partner-commander test. **Closed 2026-08-30 (`068dd03`):** *with* the deaths, not alongside — CR 704.3's "single event" is all of 704, not just 704.5 — and last in batch order, which is 704.6's place in CR order. `deaths` is now `batch`. The regression is `test_two_commanders_offered_in_one_check_move_as_one_event`, asserting on the batch id because the end state is identical either way, which is what made this latent; pre-fix it reports `BatchId(2)` and `BatchId(3)`. | `fix` ✅ |
| H5 ✅ | `gather.rs:473` | **`forced_bucket`'s `debug_assert` is a tautology.** `top` is `min()` over the candidates' classes, so `candidates.iter().any(\|c\| c.def.class == top)` holds by construction and the assert can never fire (the `Other` short-circuit doubly so). Delete it or assert something real (e.g. the returned bucket is non-empty). **Closed 2026-08-30 (`068dd03`):** deleted. The suggested replacement is the same tautology restated — `top` came out of the candidates, so the filter always keeps at least the one that produced it — and the comment above now says so instead. | `fix` ✅ |
| H6 ✅ | `sba.rs:69` | **`ordered_objects` sorts on `(zone_change_epoch, ObjectId)` — the tiebreak is the key `CLAUDE.md`'s determinism section bans** ("never `ObjectId` (v4 UUIDs); same rule for any collection reaching a choice"). Latent, not live: every object passing the 704.6d filter has moved at least once and epochs are unique per move, so ties exist only among never-moved objects that the filter then discards. But the function's contract says "every object in the game, in a deterministic order", which is false as stated, and the first future consumer that reads epoch-0 objects inherits a per-process order. Filter before sorting, or drop the tiebreak and assert uniqueness among survivors. **Closed 2026-08-30 (`068dd03`):** both. `ordered_objects` is `moved_since(game, since)`, which takes the CR 704.6d epoch filter inside the function, sorts on the epoch alone, and `debug_assert`s that no two survivors share one — unlike H5's, that assert can fail, because the uniqueness it checks is maintained in `move_object` rather than by this function. | `fix` ✅ |
| H7 | `actions.rs:302–312` | **Phase 2 performs do not re-check member legality, and two members about the same object error loudly instead of skipping.** If a batch carries two `ZoneChange`/`Destroy` members naming one object, the first perform moves it and the second errors ("proposed Battlefield→X, which is in Graveyard") — a game-killing `Err` where CR 608.2b's partial resolution says do-as-much-as-possible. Unreachable today: the SBA sweep dedupes per object, combat/untap batches are per-object unique, and no registered spell proposes the same object twice. Becomes reachable with the first multi-clause spell ("Destroy target creature. Destroy target creature.") or any replacement that redirects member 1 into member 2's precondition. Related to F6's interleaving question but distinct: this is perform-phase staleness, not choice-phase restarts. | `defer` |
| H8 | `pipeline.rs:108,193` | **Declining CR 903.9b is recorded as final for the whole event, but the rule's 614.5 exemption contemplates re-application after the event changes.** `declined` is keyed on the same instance id regardless of destination, so an owner who declines the command zone for a hand-bound event is never re-asked if a later replacement redirects the event to the library — where 903.9b becomes newly applicable and "may apply more than once to the same event". No second hand/library-redirecting effect exists and nothing sets `is_commander` in real games (K7), so this is unreachable; it is also genuinely unclear in the rules whether a *decline* exhausts the exemption. Research question, not a bug: settle it when RD/RE gives the event a second redirector. | `design` |
| H9 | `pipeline.rs:94`, §4.2 | **CR 614.5's identity may need to be per `(event, affected object)`, not per batch member — the shield counter's two-source ruling is the first crack in §4.2's per-member doctrine.** Simultaneous combat damage is one event (CR 510.4); CR 122.1c makes any number of shield counters "a single prevention effect"; the printed ruling removes **one** counter when multiple sources damage the creature at once. The engine models each source's damage as a separate batch member with a fresh applied set, so a creature blocking two attackers with two shield counters loses **two** — each member's prevention applies, each queues a rider. (With one counter the outcome is accidentally right: both prevented, the second rider removes zero and is a no-op.) Kalitas's per-death ruling and this ruling are both real, and they reconcile exactly if the applied set is keyed per affected object within the batch: Kalitas's deaths are different objects → N applications ✓; two damages to one creature are one object → one application ✓. Unreachable today (no shield-counter producer, and it needs a multi-blocked creature), but **RD must settle this before building CR 615.7's shield allocation on the per-member shape** — 615.7's "one shield, several simultaneous sources, controller chooses" is the same modeling question asked officially. | `design` |

## I. Doc and comment accuracy, second pass — what B could not have seen — ✅ **CLOSED 2026-08-30**

B closed against the code that existed then; these are in the surfaces B did
not open (§2a, the trim's neighbors, the unreviewed commits) plus two rule
numbers from an older CR that B7's sweep did not cover. Same character as B:
the code is right, something written about it is wrong.

**Closed in the #62 pre-merge session, 2026-08-30.** Every row fixed except one
sub-claim of I7, which did not reproduce (see the row). Three sites outside the
rows were swept along, each the same defect as a row that named it:
`objects/object.rs:34,36` carried I6's two stale rule numbers, the CR 903 table's
`903.12`/`903.13` labels are Brawl and Commander Draft in `tmnt.txt` (Partner is
CR 702.124), and **I10** below was found while fixing I4.

| # | Area | Finding | Verdict |
|---|---|---|---|
| I1 ✅ | `replacement-architecture.md:221` | **§2a's `ReplacementDef` row lists eight fields; the struct has nine — `optional` is missing.** The absent one is load-bearing (it is the field the decline-tracking rule exists for). The as-built map's whole warrant is field-for-field fidelity. | `doc` |
| I2 ✅ | `replacement-architecture.md:225`, `types/replacement.rs:292` | **`GameActionTemplate::ZoneChangeTo` has three RB customers, not two** — the finality counter, CR 903.9b, *and Kalitas's exile rewrite*. Both the §2a table ("two customers each") and the type's own doc ("Two customers in RB: … finality … and 903.9b") undercount; Kalitas landed after the doc sentence and nobody re-counted. `RemoveCountersFromAffected`'s two is right. | `doc` |
| I3 ✅ | `replacement-architecture.md:255` | **§2a's trace says "1 → apply it, never prompt" and calls CR 616.1 "the only prompt" — both false for the optional path.** A single optional candidate *does* prompt (`ask_apply_optional_replacement`; CR 903.9b is exactly a one-candidate prompt, and two tests script it). The never-prompt rule governs the CR 616.1 *ordering* choice only. While editing: property 1's "before anything is written" overstates phase 1 — `consume_use` deliberately writes during deciding (a spent shield must be gone for the next member; the code documents this), so say "against one board" and name the exception. | `doc` |
| I4 ✅ | `types/replacement.rs:25` | **B8 stopped one file short: the module doc still says "a sixth arm"** 200 lines before the five-arm table that makes the phrase parseable, over an enum that ships two. B8's own reasoning ("the growth contract was written against the design, not the code") applies verbatim; `CLAUDE.md` got "a new arm", this doc should too. | `doc` |
| I5 ✅ | `actions.rs:384,481` | **Two comments still describe the pre-RA emitter topology.** "Internal helpers like `draw_card` and `play_land` still call `move_object` directly" and "draw_card already emits ZoneChange events via move_object" — both false: `draw_card` routes through `change_zone` → `execute_action`, which is why the one-emitter invariant holds and why the empty-library draw correctly reaches the pipeline (CR 121.6a). As written they contradict the invariant three paragraphs above them. | `doc` |
| I6 ✅ | `actions.rs:442,1022,1078`, `codebase-state.md` §903 | **Commander-damage rule numbers are from an older CR — B7's disease, different rules.** In `tmnt.txt`: the 21-damage loss SBA is **CR 704.6c** (704.5u is space sculptor) and the commander-damage rule is **CR 903.10/903.10a** (903.11a is "bring a card from outside the game"). Three sites in `actions.rs` and `codebase-state.md`'s "903.11 — Attacking with commander + accumulating commander damage" row label are stale. Pre-date RB, but the PR's B7 sweep established the standard and these sit in files it touched. | `doc` |
| I7 ⚠️ | `codebase-state.md:10,235,246` | **Three stale counts in the top-authority doc, all moved by this branch's own later commits.** "744 tests" — the tree runs **746** (G3 added two registry tests and the TL;DR was not re-touched). "Six stubbed primitives implemented: [four names], plus new [four names]" — four stubs got implementations and four primitives are new; no reading yields six. "`specdb owed` is unchanged at 38" — it prints **37** on this tree (the post-review COVERS corrections moved it after the sentence was written). **2026-08-30: the first two are fixed; the third did not reproduce.** `specdb owed` prints **38** — before this session's changes, after them, and after a `specdb build` — and no post-review commit touches a `COVERS` line, so the doc was right and stands. | `doc` ✅ (two of three) |
| I8 ✅ | `ui/ask.rs:639` | **B6's malformed-literal disease has a second RB instance**: `ask_choose_replacement`'s assert message carries a run of ~10 literal spaces from a line-continuation that never happened ("two or more          applicable effects"). Same fix as B6 (real `\` continuations). Noting in passing: `cast.rs:298` has the same defect and predates the PR. | `fix` |
| I9 ✅ | `game_state.rs:249` | **"Between them the gate is sound" needs its expiry date.** True today: printed abilities are recorded at ETB and Layer-6 grants are summarized. But a **Layer 1 copy** of a permanent with a printed replacement ability (or a Layer 3 text-change granting one) changes the *effective* ability list through neither half — the copy's replacement would be silently dead, which is this project's named worst failure mode. `codebase-state.md`'s own 2026-08-27 audit already demands a copy-effects design doc before RC-4 produces a `CopyOnEnter`; that doc must inherit this gate obligation explicitly, and the soundness comment should scope its claim ("sound until Layer 1/Layer 3 exist") so the copy phase cannot miss it. **Closed 2026-08-30:** `copy-effects-architecture.md` §4.7 already owns the gate obligation (and found its twin in `register_static_effects`); the `game_state.rs` comment is now scoped and points at §4.7. | `doc` ✅ |
| I10 ✅ | `types/replacement.rs:132`, `codebase-state.md:237`, `replacement-architecture.md:2100` | **`EventPattern`'s own growth-contract heading undercounts it, in exactly I2's shape.** "Why five arms and not eight" sits over an enum that ships **six** — `DealDamage`, `ZoneChange`, `Untap`, `Tap`, `Destroy`, `CounterChange` — against a ten-variant `GameAction`. §2a's table has 6 and is right; the enum's own doc, `codebase-state.md`'s five-findings paragraph and §9's finding all say five. The body's substance holds (the three player-affecting variants really have no arm); the arithmetic drifted, and the collapse it hides is worth stating — `CounterChange` covers `AddCounters` *and* `RemoveCounters` through `adding`, the one place "one arm per `GameAction` variant" is not 1:1. Found 2026-08-30 while fixing I4, and fixed with it. | `doc` ✅ |

## J. The census and the "can't" doc — Q2's answer in rows

The verdict first: **the numbers the design leans on hold.** 236/2,034
reproduces exactly; 1,249/1,262 is internally consistent as §4.2's hand-counted
1,200 + 49. The rows are the places where the apparatus is weaker than the doc
implies — none moves any RS phase's sizing by more than a rounding error.

| # | Area | Finding | Verdict |
|---|---|---|---|
| J1 | `cant-census.py:145` | **"can't search (701.19)" cites the regeneration rule.** Search is **CR 701.23** in `tmnt.txt`; 701.19 is Regenerate — the same number the engine cites correctly a hundred times, so the collision is maximally confusing. Label only; the count is fine. | `fix` |
| J2 | `cant-census.py:150` | **"can't be attacked" is bucketed under *attachment* (Tier 2) and is a combat restriction (Tier 1a).** Two clauses hit it — Island Sanctuary and The Aetherspark, verified against the cache — both restrictions on attack declaration. True Tier 2 is 234, Tier 1a 1,264; nothing resizes, but the classifier has a provable mis-bucket and RS-1's spine count inherits it. Nearby, lower: "can't become suspected/night/monarch" (3 clauses) land under "transform / turn face up / phase", which is the right tier with the wrong label. | `fix` |
| J3 | `cant-effects-architecture.md:142` | **"Every number below is `cant-census.py`" overclaims — §2.4's and §4.2's derived numbers have no script behind them.** The 62 distinct tails, 77 counting clauses, 1,200/49/13 decomposition, the 35%/28%/11% duration shares and the 149/138 conditional counts are all dated hand counts. Two loose threads a reader cannot resolve from the page: §2.4 says 77 counting clauses where §4.2's table has 49 per-attacker counts (where are the other 28?), and the "2 cards" global-cap row only sums to 1,262 if those cards' clauses live inside the 13. Either extend the script to print the decomposition or mark those numbers as hand counts with their method, and reconcile 77/49/13. | `doc` |
| J4 | `cant-effects-architecture.md` §2.1 | The doc's tier table faithfully mirrors the script — which means it inherits J2's ±2. When J2's script fix lands, re-run and update the table (236 → 234, 1,262 → 1,264) in the same commit, or the doc and its evidence disagree by exactly the amount that looks like drift. | `doc` |
| J5 | `cant-census.py:54` | **The census's query is `o:"can't" -is:funny`, so restrictions phrased "isn't" or "doesn't" are outside all 2,034 clauses** — and that is a *scope*, correctly stated by the script, not a regex bug. It is worth stating in the doc because one such family is not small: **248 cards print "doesn't untap during …"** (`o:/doesn.t untap during/ -is:funny`, 2026-08-30), and 7 print "damage isn't removed …" (`codebase-state.md` item 20). §2.2 "What the census cannot see" already exists for exactly this and currently lists only the keyword-borne restrictions; these are the phrasing-borne ones. Found while answering E4, which is why it is here rather than in E. | `doc` |

## M. Owner review of the C+E branch (2026-08-30)

Five points raised on `replacement/rb-naming` before merge. **Four closed on
the branch** and are recorded where they belong: E1's answer was rejected for
the right reason and replaced with `check_exempt_terminates` (see theme E);
`replacement-architecture.md` §8a lost an overreach about CR 506.4 and gained
the "what does it cost to be wrong" pricing it was missing; the vestigial
`impl Default for ReplacementEffectRegistry` is deleted. **One is new and
stays open.**

| # | Area | Finding | Verdict |
|---|---|---|---|
| M1 | `card_data.rs:31`, `layers/types.rs:260` | **The field `keywords` reads as "this object's keywords" and is really "the presence-only ones".** `KeywordFlag`'s own doc is exemplary — CR 702's 189 keyword abilities split by a four-quadrant table, and this enum is quadrant ① — but the *field* drops the qualifier the *type* carries, on both `CardData` and `EffectiveCharacteristics`. A card with ward or cycling has those in `abilities`, so `card.keywords` is **not** the card's keyword list, and `CLAUDE.md`'s layer invariant lists `keywords` beside `abilities` as though they partitioned. Nothing is wrong today — every reader passes a `KeywordFlag`, so the type enforces the intent — which is exactly why this is legibility debt rather than a bug. Note the *other* half of the word is already clean: CR 701's keyword **actions** have their own type in `types/keyword_actions.rs`, so the ambiguity is inside CR 702 only. **Sized:** rename the field to `keyword_flags` to match its type — 94 sites across 33 files, every one compiler-enforced, no behaviour. Do it in its own commit, not folded into anything. | `fix` (naming) |

---

## The merge question, re-answered

**The first review's headline survives contact with the unexamined surfaces,
with one asterisk.** There is still no correctness defect reachable from the
registered card pool — H1, H2 and H9 are all latent, which the byte-identical
fuzz baselines corroborate. The test file is not the liability the closing note
feared; it is the best-tested part of the PR, and it caught nothing because
there was nothing reachable to catch.

The asterisk is **H2**: the branch ships a test that *enforces* a rules
violation. That is worse than an untested path — the next session that touches
the registry will keep the wrong behavior green. It is also a five-minute
close: delete one call, one test, and fix two comments; production behavior
cannot move (the call is dead there), so the fuzz gate is a formality.

**Recommended before merge, one short session:** H2, plus the authority-doc
rows I1–I4, I6, I7 (all `doc`, all in `plans/` or one comment block — the same
class theme B blocked the merge for) and I8 (one string). H1 and H4–H6 are real
but latent code changes; they re-run gates and belong in the post-merge C+E
session, where H5/H6 are one-liners and H1/H4 are small. H7–H9 and J are
`defer`/`design`/script work and block nothing.

**That session ran 2026-08-30 and closed H2 and all of I (I5 and I9 rode
along, I10 was found inside it).** Gates after: 746 tests, 0 warnings,
`check_claude_md` 197/200, `specdb orphans` clean and `owed` 38 — and both fuzz
baselines byte-identical, as expected for a branch whose only behavior change
deletes a call that production could not reach. **#62 is ready to merge.**

**What this audit changes on the critical path: nothing re-orders, two
obligations attach.** RC and its ~1,350-card unlock are unaffected; RS-1's
shape is verified accurate; the census stands. The attachments: (1) **RD's
design must open with H9** — CR 615.7's shield allocation sits exactly on the
per-member-vs-per-affected-object applied-set question, and building it on the
unexamined per-member shape risks a redesign; H1 also wants closing before RD's
`Amount` rewrites multiply damage numbers through more paths — **it closed
2026-08-30, ahead of RD, so that obligation is discharged**. (2) **The
copy-effects design doc `codebase-state.md` already demands must own the gather
gate's third leg** (I9), or Layer 1's first copied replacement ability is
silently dead. Both are notes to designs already scheduled, not new phases.
