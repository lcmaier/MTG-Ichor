# A4i review — findings, triaged

The owner's review of [PR #161](https://github.com/lcmaier/MTG-Ichor/pull/161),
2026-09-17, captured before anything was fixed (`engineering-practices.md` §4).
Sixteen comments, seven themes. **Close one theme per session, starting cold
from this file**; delete the file in the PR that lands the last one.

Nothing here is merged into the PR yet. Two themes (A, F) are pre-merge in my
reading; the rest are follow-ups, and the owner decides.

---

## A — Naming and legibility · *pre-merge, mechanical*

| # | Site | Finding |
|---|---|---|
| 1 | `backlog.md:1078` | "`EffectRecipient::Instance(ix)`" — the prose never says what `ix` is. It is the clause's position in `effect_instances`' printed-order list. Say "by index" in words. |
| 3 | `phase_rf_integration_test.rs:133` | `ChosenTargets::EMPTY` reads as "could be full", which is meaningless for a targetless effect. **Rename to `NONE`.** The deeper problem the name exposes: one constant is doing two jobs — "this effect announced no instances" and "there are no *earlier* instances yet". Theme C item 11 removes the second. |
| 4 | `phase_re8_integration_test.rs:123` | `Instance(0)` is inscrutable at the call site. Rename the variant so the call site reads: **`SameInstanceAs(0)`**. Keeps CR 115.3's word, says what it does, and it is a variant this PR introduced — a handful of sites. |
| 12 | `targeting.rs:473` | `earlier` → **`earlier_targets`**, everywhere it appears as a parameter. |
| 5 | `engine/cast.rs` | The file holds `cast_spell`, `activate_ability`, `announce_targets` and `run_mana_ability_window` — CR 602.2b routes an activation through 601.2's steps, so the module is not about casting. Rename to **`engine/put_on_stack.rs`**: a cast and an activation both put an object on the stack by those steps, and "proposal" is taken by the action pipeline. Rename-only; ride it with `roadmap-v2.md` row A4m, which is already a rename PR. |

---

## B — The rule, stated correctly · *pre-merge for the prose, design for the rest*

**Finding (#2, `backlog.md:1080`).** "CR 601.2c announces one clause at a time"
is a claim about the engine, not about the rule. The rule says only *"The player
announces their choice of an appropriate object or player for each target the
spell requires"* — it fixes no order and does not say whether the choices are
simultaneous. The sequential loop is an **implementation** order, forced by
`ObjectFilter::OtherThanInstance` needing the earlier choices. Fix the sentence.

**It is outcome-equivalent today, and stops being so at one named rule.** Every
assignment a simultaneous announcement allows is reachable by announcing in
index order, and the loop cannot produce one it forbids. That equivalence breaks
exactly when 601.2c's last sentence lands:

> If any effects say that an object or player **must be chosen as a target**, the
> player chooses targets so that they obey the **maximum possible number** of
> such effects without violating any rules or effects that say that an object or
> player can't be chosen as a target.

"Maximum possible number" is a **global optimum over all instances**. A greedy
per-instance loop can take a first target that makes a later requirement
unsatisfiable, and no amount of per-instance checking sees it. So the announcement
order stops being free the day requirements exist.

**Neither half is handled, and they have different owners.**

- The **"can't be chosen"** half is RS-2's, already scheduled
  (`cant-effects-architecture.md`, `targeting.rs::validate_targets` is one of its
  six enforcement sites).
- The **"must be chosen"** half is **unowned**. It is the targeting analogue of
  combat's `Requirement` (CR 508.1d, RS-3b) and wants the same bounded search.

**Survey, 2026-09-17.** No printed card says an object "must be chosen as a
target": `o:"must be chosen"` and `o:"chosen as a target"` return nothing, and
`o:/target.{0,20}if able/` returns 4 cards that are all *combat* requirements
(Dulcet Sirens, Hunt Down, Ravener, Rimehorn Aurochs) — RS-3b's, not this. So
the sentence is **CR-stated with no printed producer**, which by the RE-5 rule
(`engineering-practices.md` §4) is owed with a fixture test and a "no printed
producer" reachability line, never "nothing owed".

**A related survey that did find cards, and it sharpens item 154.**
`o:"must target"` returns **8**, and every one is *"Each mode must target a
different player"* — Balor, Chaos Balor, Casey & Raph, Donnie & April, Mikey &
Mona and kin. That is not a requirement; it is **distinctness across instances
over a player filter**, which is `OtherThanInstance` applied to something
`SelectionFilter::Player` cannot carry. `codebase-state.md` item 154 says that
gap has no named customer — it has eight, and they arrive with modal spells
(`backlog.md` §2.7). Update the item.

**Action:** fix the backlog sentence now; open a `codebase-state.md` item for the
"must be chosen" half with the survey and a size; amend item 154 with the eight.

---

## C — Representation and cost · *follow-up, one PR*

**#9 — the nested `Vec` is still there.** The last round moved the nesting behind
a name and made exactly one site index it, which fixed the *indexing* smell and
not the *representation*. The owner is right that the justification is thin.

**Fix: a flat buffer plus offsets.** `ChosenTargets { flat: Vec<ResolvedTarget>,
bounds: Vec<u32> }`, instance `i` being `flat[bounds[i]..bounds[i+1]]`. One
allocation instead of one per instance, `instance(ix)` still returns a slice, and
`all()` becomes the buffer itself. The public surface does not change, which is
what makes it a follow-up rather than a re-design.

**#13 — `surviving_targets` copies the world twice.** It clones every instance's
`Vec` (including untargeted ones, which are never filtered), collects into a new
`ChosenTargets`, and then builds `announced_targets` as a *second* full clone so
the CR 608.2b re-check can read the announcement. Two full copies per resolution,
where the announcement is already sitting on the `StackEntry` and could be
borrowed. The layer walk per target is required by 608.2b and stays.

**#10 — why the feed-forward rebuilds rather than subtracts.** Because a clause's
filter is its own: Decimate's four clauses are artifact / creature / enchantment /
land, so there is no single legal list to subtract the taken object from. That is
the general case. **But it is not the case the feed-forward exists for** — the
feed-forward only runs when a later clause reads an earlier instance, and that is
the "another target" family, where the clauses share a filter. So the owner's
instinct is right for exactly the code path that matters:

> When the clauses that read each other share a filter, enumerate **once** and
> subtract. Fall back to per-clause enumeration only when the filters differ.

That turns theme C's hot loop from O(I) enumerations into one, and it is what
makes #14's complexity defensible rather than merely measured.

**#11 — `FilterIdentity` is bespoke, and it is already missing a member.**
`enumerate_legal_selections`' `exclude_id` parameter — the Aura that cannot
enchant itself — is the *same kind of fact*: an identity the filter must exclude,
unanswerable by any layer. It sits outside the struct as a third positional
argument. CR 115.5 ("a spell or ability on the stack is an illegal target for
itself") is a fourth of the same kind and is unmodeled. **Fold `exclude_id` in**,
and the struct stops being bespoke and starts being the answer to "what identity
facts does a selection filter need".

---

## D — Coverage · *pre-merge, small*

**#16 — the n−1 boundary is untested.** Incremental Growth's tests cover **1 of
3** illegal (`..._still_counters_the_creatures_that_are_left`) and **3 of 3**
(`..._does_not_resolve_with_every_creature_gone`). **2 of 3 is not covered**, and
that is the boundary an off-by-one in `surviving_targets`' `any` would hide —
`survived` reading `all` instead of `any` passes both existing tests and fails
only here. Add it.

---

## E — Choice versus target · *design, then a backlog entry*

**#8.** `spell_instances` returns whatever the card declares. For a **cast** Aura
that is `Target` (CR 303.4c — an Aura spell targets); `EffectRecipient::Choose`
is for the non-targeting selections, CR 303.4a's Aura put onto the battlefield
without being cast among them.

**Two things the review surfaced, and neither is A4i's doing.**

1. **No registered card uses `Choose`.** Its only constructor is
   `resolve.rs::sacrifice_of_choice`, built at resolution. So the `Choose` leg of
   the instance model — collected by `effect_instances`, announced by
   `announce_targets`, re-checked (and exempted) by `surviving_targets` — is
   exercised by no card at all.
2. **The timing may be wrong, and it was wrong before this PR.** `announce_targets`
   asks a `Choose` clause **at cast time**, because the code it replaced did
   (`cast.rs` matched `Target(..) | Choose(..)` in one arm). But a plain "choose"
   in an effect's text is made **on resolution** (CR 608.2c), not as the spell is
   cast; only a clause that says "as you cast" is announced early (CR 601.2b).
   The Black Gate — *"Choose a player with the most life or tied for most life.
   Target creature can't be blocked by creatures that player controls this turn"* —
   is one ability carrying both, and they happen at different times.

**There is no survey of "choose" across the CR or the pool, and there should
be.** `o:"choose a player"` alone is 17 cards. The entry wants: which CR rules
put a choice at announcement versus at resolution; which of those the engine can
express; and whether `Choose` belongs in the instance list at all or is a
resolution-time selection that never had an instance. **Open it as a
`backlog.md` entry**; it is the kind of question §2.9's information-model entry
is shaped like.

---

## F — Enforce the broken string categorically · *pre-merge; it is a gate*

**#7.** Two strings in this PR are corrupted the same way — a `\`-newline
continuation that became a run of literal spaces, because the script that wrote
the file ate the backslash (`resolve.rs:143`, `:2042`). There is a memory note
about it and a note is not a gate.

**Measured, 2026-09-17.** A run of **≥10 spaces inside a Rust string literal**
finds 33 lines. Two are `fuzz_games`' column-aligned output (`"Errors:
{}"`-shaped, a run between a `:` and a `{`) and are exempt by that shape. The
rest are genuine, and **five predate this PR**: `phase_rb_cards.rs:208`, `:287`,
`cast.rs:358`, `resolve.rs:1917`, and `resolve.rs:143`. This defect has landed at
least four times before anyone noticed it.

**Action: `plans/check_string_literals.py`**, joining the check family with a CI
step (the rule in
`codebase-state.md` — a gate that must fail a PR is a `check_*.py`, offline, and
costs `CLAUDE.md` zero lines). Fail on a ≥10-space run inside a string literal,
exempting the label-to-placeholder shape. Fix the seven. ~60 lines.

---

## G — Comprehension, answered · *the trace page is the artifact*

**#6 — why `resolve_effect` needs a cursor.** An atom's recipient says what
*kind* of clause it is, never *which one*: Seeds of Strength's three atoms carry
three structurally equal `Target(Creature, Exactly(1))` values and must map to
instances 0, 1, 2. The announcement numbered them by pre-order position; the
resolution has to reproduce that numbering to look them up, and the cursor is
that reproduction — walk the same tree in the same order, and every atom that
*declares* takes the next number. An `Instance(ix)` atom declares nothing,
consumes no number, and reads `ix` directly, which is why the cursor counts
declarations rather than atoms.

**The alternative, and why not.** Number the tree once and store the index on the
atom. `Effect` is card data behind an `Arc` and shared by every copy of the card,
so the numbering would have to live on a lowered copy on the `StackEntry` —
a second tree to keep in step with the first. The cursor is the cheap form of the
same thing. **The invariant it rests on is worth stating in the source and is
not:** the effect the resolution walks is the one the announcement walked
(`StackEntry.effect` is cloned at cast and never mutated), so the two numberings
cannot disagree.

**#15 — does `OtherThanInstance` stack.** Yes, by nesting. Incremental Growth's
third clause is `And(And(ByType(Creature), OtherThanInstance(0)),
OtherThanInstance(1))` — each leaf is its own predicate and `And` conjoins them,
so the third clause excludes both earlier choices. Walked board by board on the
trace page.

**Artifact:** `plans/traces/a4i-a-target-belongs-to-an-instance.html`, written at
this review. A4i qualifies under §7's own test — it changes *how* a target read
is answered rather than what the answer is — and both questions above are ones
the diff cannot answer.
