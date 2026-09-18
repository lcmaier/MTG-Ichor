# A4i review — findings, triaged

The owner's review of [PR #161](https://github.com/lcmaier/MTG-Ichor/pull/161),
2026-09-17, captured before anything was fixed (`engineering-practices.md` §4).
Sixteen comments, seven themes. **Close one theme per session, starting cold
from this file**; delete the file in the PR that lands the last one.

**Closed before merge, at the owner's direction (2026-09-17): A, B, C and F.**
The owner's call on C was explicit — *"I don't want to ship the double vec even
as a placeholder"* — so the representation landed in this PR rather than after
it. What is left open is **E** (choice versus target: the survey, and whether a
non-targeting `Choose` belongs in the announcement at all), which wants a
`backlog.md` entry rather than a fix, and **D**'s sibling questions that became
`codebase-state.md` items 157 and 158.

**A second audit after the merge (2026-09-17, theme I below) found two live
defects, both pre-existing and both inherited by A4i's new count arms, and
scheduled them as `roadmap-v2.md` rows A4o and A4p; its performance riders
went onto row A4n.**

Each theme below ends with what actually happened, including the one item that
was **withdrawn because building it showed it was wrong**.

---

## A — Naming and legibility · ✅ **closed cc1b4d9**

| # | Site | Finding |
|---|---|---|
| 1 | `backlog.md:1078` | "`EffectRecipient::SameInstanceAs(ix)`" — the prose never says what `ix` is. It is the clause's position in `effect_instances`' printed-order list. Say "by index" in words. |
| 3 | `phase_rf_integration_test.rs:133` | `ChosenTargets::NONE` reads as "could be full", which is meaningless for a targetless effect. **Rename to `NONE`.** The deeper problem the name exposes: one constant is doing two jobs — "this effect announced no instances" and "there are no *earlier* instances yet". Theme C item 11 removes the second. |
| 4 | `phase_re8_integration_test.rs:123` | `Instance(0)` is inscrutable at the call site. Rename the variant so the call site reads: **`SameInstanceAs(0)`**. Keeps CR 115.3's word, says what it does, and it is a variant this PR introduced — a handful of sites. |
| 12 | `targeting.rs:473` | `earlier` → **`earlier_targets`**, everywhere it appears as a parameter. |
| 5 | `engine/put_on_stack.rs` | The file holds `cast_spell`, `activate_ability`, `announce_targets` and `run_mana_ability_window` — CR 602.2b routes an activation through 601.2's steps, so the module is not about casting. Rename to **`engine/put_on_stack.rs`**: a cast and an activation both put an object on the stack by those steps, and "proposal" is taken by the action pipeline. Rename-only; ride it with `roadmap-v2.md` row A4m, which is already a rename PR. |

---

## B — The rule, stated correctly · ✅ **closed 615fd48** (prose + items 157, 158; the solver itself is item 157's, unscheduled)

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

### The optimum shape, surveyed across the whole CR (2026-09-17)

The owner's question, and it is the right one to be nervous about. Searching the
frozen CR for the phrase gives **exactly three sites**, and they are one rule
wearing three hats:

| Rule | What is maximized | Owner |
|---|---|---|
| **508.1d** | attack requirements obeyed | RS-3b |
| **509.1c** | block requirements obeyed | RS-3b |
| **601.2c** | targeting requirements obeyed | **unowned — this finding** |

There is no fourth. `cant-effects-architecture.md` already calls 508.1d/509.1c
the NP-hard one and gives it a bounded-exact search with a cap (§4.2), so the
shape is budgeted; what this finding adds is that **targeting is the third hat
and nobody has it**.

**And the targeting one is much smaller than combat's.** Combat searches over
every creature a player controls crossed with attack/block assignments.
Targeting searches over **the instances of one spell** — one to four in every
printed card — crossed with that instance's legal candidates. It is the same
algorithm on a search space two orders of magnitude smaller, and RS-3b's cap
covers it for free.

**Today the space is empty.** No printed card produces a targeting requirement
(survey below), so there is nothing to maximize and the greedy loop is exact.

### Not to be confused with: "as many as possible"

A different phrase, a different shape, and the engine already does it. CR 101.3
("only the possible portion is performed"), 701.17b (mill a short library),
701.23d (search for more than the zone holds) and the 601.2h discard example are
**clamps**, not optimizations: take `min(asked, available)` and move on. They
cost nothing and are implemented — `Primitive::Mill`, `sacrifice_of_choice`'s
`count.min(candidates.len())`. Nothing in that family needs a search.

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

**And a third population the owner supplied, which my survey missed.**
`o:"the copy targets" or o:"the copies target" or o:"each copy targets"` returns
**13**: Precursor Golem, Zada, Ink-Treader Nephilim, Mirrorwing Dragon, Radiate,
Agrus Kos, Beamsplitter Mage, Exterminator Magmarch, Feather, Frontline Heroism,
Ivy, Radiant Performer, Zevlor. Most carry *"Each copy targets a different one of
those creatures."*

**It is the same rule shape on a different axis, and it is not A4i's.** A4i's
`OtherThanInstance` is distinctness *across instances of one object*; the modal
eight are distinctness *across modes of one object*; these thirteen are
distinctness *across separate stack objects* — the copies — and the constraint is
written by the card as the copies are created, not by CR 601.2c as one spell is
announced. So it belongs to `copy-effects-architecture.md`, not here. Recorded
because the three together are the real size of "the engine cannot say *different
from that one*": **8 + 13 + the A4i family**, three axes, one missing expression.

**Action:** fix the backlog sentence now; open a `codebase-state.md` item for the
"must be chosen" half with the survey and a size; amend item 154 with the eight.

---

## C — Representation and cost · ✅ **closed 3ea4cb5**, A/B in `fuzz-record.md` (three of four items; the fourth withdrawn, below)

**#9 — the nested `Vec` is still there.** The last round moved the nesting behind
a name and made exactly one site index it, which fixed the *indexing* smell and
not the *representation*. The owner is right that the justification is thin.

**Fix: a flat buffer plus offsets.**

```rust
pub struct ChosenTargets {
    /// Every instance's targets, concatenated in instance order.
    flat: Vec<ResolvedTarget>,
    /// Where each instance starts in `flat`, plus a trailing sentinel.
    /// `bounds.len() == instances + 1` and `bounds[instances] == flat.len()`.
    bounds: Vec<u32>,
}
```

`instance(i)` is `&flat[bounds[i] as usize .. bounds[i + 1] as usize]`.

**Why `bounds` is a `Vec` and not an array or a field.** The number of instances
is a property of the *card*, known only at run time and unbounded in principle —
one for Lightning Bolt, three for Seeds of Strength, four for Decimate, and
nothing in the CR caps it. So the index structure has to grow, which makes it a
`Vec`. It is a **prefix-sum**, not a list of lengths: storing starts rather than
sizes makes `instance(i)` two reads instead of a running total, and the trailing
sentinel is what lets the last instance use the same expression as every other
one instead of a special case.

**Why not `Vec<Range<u32>>`.** Same length as the instance list, no sentinel
needed — but twice the memory, and it lets the ranges be non-contiguous or
out of order, which is a state this type should not be able to represent. The
prefix-sum form makes contiguity structural rather than an invariant someone has
to maintain.

**What it buys.** `Vec<Vec<T>>` is `1 + N` allocations for N instances; this is
**2, always**. `all()` stops being a `flatten` and becomes `&flat`. `push` is
`flat.extend(targets); bounds.push(flat.len() as u32)`, starting from
`bounds: vec![0]`. The public surface — `instance`, `all`, `len`, `is_empty`,
`push`, `one`, the `NONE` constant — does not change, which is what makes this a
follow-up rather than a re-design.

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
`enumerate_legal_selections`' `exclude_id` is the *same kind of fact*: an
identity the filter must exclude, unanswerable by any layer, sitting outside the
struct as a third positional argument. **Fold it in.**

**The owner's question corrected the file and two source comments.** I wrote, and
the pre-existing doc comments on `has_any_legal_choice` and
`enumerate_legal_selections` said, that `exclude_id` is "the Aura, which can't
enchant itself" — and the owner's objection is exactly right: *every* Aura can't
enchant itself, so that cannot be what a per-call parameter is for.

**What it actually implements is CR 115.5** — *"A spell or ability on the stack
is an illegal target for itself."* Every caller inside CR 601.2c's loop passes
the object being cast or activated, not an Aura. The Aura case it was named for
**cannot arise at all**: an Aura spell is on the *stack* when its target is
chosen, and an enchant filter is a `Permanent` filter, so `require_on_battlefield`
rejects it before `exclude_id` is consulted. Where the parameter actually bites
is the stack-reading filters — `Spell` (a Counterspell offered itself) and
`DamageSource`. Both comments are corrected; doc only, no behaviour.

**But the fold itself was wrong, and building it is what showed why.**
`FilterIdentity` is read by `object_matches_filter_with` and by nothing else —
and the arms where `exclude_id` actually bites, `Spell` and `DamageSource`, never
call it. They have no `ObjectFilter` to walk at all; they answer membership
questions directly. Folding `exclude_id` into `FilterIdentity` would move CR
115.5 into a leaf those two arms cannot reach, which is a regression wearing a
tidier signature.

So `exclude_id` stays where it is — applied in the enumeration, uniformly,
whatever the filter's shape — and what it needed was a name for its rule, which
it now has. **`FilterIdentity` is left holding one fact plus the announcement
view**, and it keeps its shape for the reason it was written: a filter leaf that
asks about identity rather than a characteristic has one place to read from.
**Resolved as "will not do", with the reason recorded.**

---

## D — Coverage · ✅ **closed c731e55**

**#16 — the n−1 boundary is untested.** Incremental Growth's tests cover **1 of
3** illegal (`..._still_counters_the_creatures_that_are_left`) and **3 of 3**
(`..._does_not_resolve_with_every_creature_gone`). **2 of 3 is not covered**, and
that is the boundary an off-by-one in `surviving_targets`' `any` would hide —
`survived` reading `all` instead of `any` passes both existing tests and fails
only here. **Done** — `incremental_growth_resolves_on_its_last_legal_creature`.

**#7b, from the same round — a doubler per instruction.** The owner asked whether
Incremental Growth's three counter instructions interact correctly with
Vorinclex, Monstrous Raider, and **both cards are registered**, so a `stress`
game can build the board. Tested and correct: each clause is its own
`AddCounters` proposal, so CR 614.5 asks Vorinclex three times and the answer is
**2 / 4 / 6** — not twelve on one creature and not six doubled once.
`vorinclex_doubles_each_of_incremental_growths_three_instructions`. RE-5 had
asserted the shape against a two-atom fixture on *one* instance (Winding
Constrictor's ruling); this is the same claim where the instructions also land on
different subjects, which is what A4i made reachable.

---

## E — Choice versus target · ✅ **closed 2026-09-18** — `backlog.md` §2.33

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
   (`put_on_stack.rs` matched `Target(..) | Choose(..)` in one arm). But a plain "choose"
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

**Filed as `backlog.md` §2.33 (2026-09-18, A4n's PR).** All three claims hold
against the tree: the one construction site is `resolve.rs:1807`, no card file or
test constructs a `Choose`, and `announce_targets` still reads it in `Target`'s
arm at `put_on_stack.rs:302`. The entry carries the pool scale the theme asked
for and says what the survey must produce.

---

## F — Enforce the broken string categorically · ✅ **closed 22a7d6f** — and it was thirty scars, not seven

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

## G — Comprehension, answered · ✅ **trace page written**

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

---

## H — Found at round four (2026-09-17), the owner's performance question

**The storage and the walks are tight; the *derivation* is not, and it is
recomputed on a hot path.**

`spell_instances` / `effect_instances` allocate a `Vec` and clone each clause,
every call. `castable_spells` calls it once per card in hand per priority check.
Measured with a thread-local probe, 50 games at seed 12345, single-threaded,
`performance` pool:

| | per game |
|---|---:|
| `effect_instances` calls | **366** |
| clauses cloned | **325** |
| (for scale) layer walks | 340 |
| (for scale) decisions | 226 |

**It is derived more often than the layer system walks.** Each call is a `Vec`
allocation plus a deep clone per clause — and for a filter like Doom Blade's
`And(ByType(Creature), Not(ByColor(Black)))` the clone is a small `Box` tree,
not a memcpy.

**Not an A4i regression**, which is why every arm read `IDENTICAL`: the code it
replaced called `spell_recipient`, which cloned one recipient per call. A4i
turned one clone into a `Vec` plus *n* clones, and *n* is 1 for every card but
Incremental Growth. At ~366 allocations per ~6,470 µs game it is a few tenths of
a percent — under the ~2.4% run-to-run spread, and invisible to the sitting.

**The fix is not a faster derivation; it is not deriving.** The instance list is
a pure function of the card, and `CardData` is built once and shared behind an
`Arc` — so the list is a constant being recomputed. Compute it in
`CardDataBuilder::build()`, store it on `CardData` (and per `AbilityDef`), and
`spell_instances` returns `&[EffectRecipient]`: **zero allocations, zero clones,
per call**.

It also retires an invariant rather than restating one. `resolve_effect`
currently re-derives the list and relies on "the effect the resolution walks is
the one the announcement walked"; with the list on the card there is one list,
and nothing to keep in step.

**Sized:** a field on `CardData` and on `AbilityDef`, filled at `build()`; the
two derivation functions become accessors; ~40 call sites take a slice instead
of a `Vec`. Small, mechanical, and it owes an A/B — where `IDENTICAL` counters
with a *lower* CPU reading is the prediction, since nothing about what the
engine decides changes.

**Not done in this PR**: it changes `CardData`, which is a core type and was not
part of the four themes the owner scheduled. Proposed as its own row.

---

## I — Second audit, fresh eyes after the merge (2026-09-17)

A read of the merged PR against two questions the owner set: is it
performance-tight, and does it anticipate the card pool. The tree it read is
`d0102bd`: 1,594 tests green, zero warnings. **The model holds.** The instance
walk, the CR 608.2b split, the refusal of `OtherThanInstance` wherever there is
no announcement to read, and `announce_targets`' `(chooser, source, clauses)`
signature — which is what CR 603.3d will hand it — are all right and are left
alone. What follows is what the read found, ranked, with where each went.

### I.1 — Two live defects, both pre-existing, both inherited by A4i's new arms · ✅ **both closed**

Neither is A4i's doing. Both sit in the selection arms A4i rewrote to count `n`
candidates, and the rewrite carried the gap into the new logic rather than
closing it. Both were **reproduced with a throwaway fixture** before being
scheduled, so the rows below start from a known board rather than a suspicion.

**`SelectionFilter::Spell` accepts an activated ability on the stack, and the
pool has both halves.** `validate_spell_target` checks stack membership only;
`enumerate_legal_selections_upto`'s `stack()` closure yields every stack id;
A4i's `has_legal_choices` `Spell` arm counts every stack id. Only the sibling
`DamageSource` arm filters on `is_spell`. Counterspell and Merfolk
Thaumaturgist's activated ability are both in `PERFORMANCE_POOL`, so a random
game can cast Counterspell at the ability — and with the ability the only other
object on the stack, CR 102.2 makes the choice forced and nothing prompts.
`Primitive::CounterSpell` then moves the ephemeral ability object to its
owner's graveyard through `change_zone`.

The fixture: Thaumaturgist on the battlefield under player 1, summoning
sickness cleared; Counterspell in player 0's hand with {U}{U} in the pool;
player 1 activates, player 0 casts, the stack resolves. **`castable_spells`
offers Counterspell, the entry's one instance holds the ability object, and
the game ends with two Merfolk Thaumaturgist objects — one on the battlefield
and one in player 1's graveyard.** A phantom card, in every measured game that
lines the two up. `codebase-state.md` item 159; **row A4o**, first in the
slot because it is live. ✅ **Landed 2026-09-18, PR #163** — the filter asks
`is_spell_on_stack` at all three sites, and `main` had been manufacturing 26
phantom cards over the four A/B arms.

**The `Player` and `Any` arms count and offer seats that have left the game.**
`num_players()` is the player vector's length, which CR 800.4a never shrinks,
and neither the enumeration's `players()` closure nor `has_legal_choices`'
`Player` and `Any` arms ask `in_game`. Reproduced at four seats with seat 3
departed: both filters offer `Player(3)`. `validate_targets` refuses it for
`Player` — a cast the oracle offered and the engine rewinds, item 139's class,
and the validator's own comment claims the seat is "not offered at CR 601.2c",
which the enumeration contradicts — and **accepts it for `Any`**, because
`validate_any_target` never asks `in_game`. "Any target" damage resolves
against a player who is not in the game. Two-player streams cannot move,
since a two-player departure ends the game (CR 104.2a); the four-seat `stress`
arm is where the fix will differ. `codebase-state.md` item 160; **row A4p**. ✅ **Landed
2026-09-18, PR #164** — and it was not only `stress`: both four-seat pools
diverge, `main` resolved 11 Lightning Bolts against a seat that had left, and
the CR 608.2b window the fixture builds turned out not to be the one the
measured games used. The enumeration was already offering the departed seat at
announcement.

### I.2 — Performance: one allocation the redesign missed, and three riders for A4n

**The resolution walk allocates per atom, which defeats the flat buffer.**
Theme C replaced the nested `Vec` so that reading an instance is a slice
borrow, and the one site that indexes it — `resolve_effect_at` — then calls
`to_vec()` on the slice before handing it to `resolve_primitive`. Every
targeting atom of every resolution still heap-allocates. The copy is not
needed for the borrow checker: `ctx` is a `&ResolutionContext` independent of
`&mut self`, and the slice passes straight through — removed, compiled with
`cargo check`, clean, reverted. Two lines. Small today; it is the path A6
multiplies by every trigger it resolves. **Rides with A4n**, whose A/B it
shares.

**A4n is the right call, and its row now says three more things.** The list is
recomputed on three paths, not one — the castability check per card in hand per
priority pass, the activatable-ability check per ability per permanent per
priority pass, and the resolution walk once per resolution — and the row's fix
covers all three. But the precompute cannot live only in
`CardDataBuilder::build()`: card files construct `AbilityDef` as struct
literals, and a Layer 6 `GrantAbility` carries one inside a `Primitive` that
never meets the builder, so the field is filled where every definition is born
or computed lazily. And identical clauses that read no earlier instance are
checked once: Seeds of Strength, pooled, runs three `has_legal_choices`
battlefield scans per priority pass per copy in hand for one answer. Both are
on the row.

### I.3 — Item 155's condition is too narrow, and its reachability line holds

The item says the greedy feed-forward is exact when an "another target" chain
"reuses one filter". The exact condition is **monotone widening**: greedy is
right as long as no later clause is narrower than an earlier clause it
excludes. The shape that fails is "target creature" followed by "another
target creature you control", on a board where the caster's only creature is
first in timestamp order — greedy takes it for the first clause and finds
nothing for the second, when swapping would work. A Scryfall regex for that
narrowing shape (`o:/target creature[^.]*another target creature you control/`)
returns one card, Combine Guildmage, and it uses the same filter twice. So the
item is safe and the failure stays conservative; the item now states the rule a
card author can check against. Amended in place.

### What was not found

No A4i regression. No path where the new count logic is asked more often than
the single check it replaced, except the identical-clause case above. No
determinism hazard: the bounded enumeration takes the first `n` in timestamp
order. No hole in the refusals: an undeclared back-reference is refused at
registration and at resolution, and a filter leaf asked outside the loop errors
rather than matching everything.
