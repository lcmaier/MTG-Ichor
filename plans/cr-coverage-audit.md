# CR Coverage Audit — what the plan cannot express

**Status: the instrument changed on 2026-08-31.** Pass A swept the frozen CR
for *dark* rules — ones nobody had examined — and came back with **zero
facts** across 199 families. It was not useless; it found its own generator
(§1). But it searched the wrong space, and this document is now the method
that replaces it: a **type-surface audit**. For each fact-bearing type, ask

> **What can the CR require in this area that this type cannot represent?**

`codebase-state.md` → "Was the critical path complete?" is the parent; this is
that detector made systematic. **Baseline:** `MTG-Rules/versions/tmnt.txt`,
frozen, effective 2026-02-27. The freeze is what makes a result durable.

---

## 0. The budget

This document owns the **method**, the **calibration bar**, and the **register
of what the audit has found**. It is not a findings dump: a finding that needs
action goes to `codebase-state.md`'s Deferred Migrations or to an owning
architecture doc, and §5 keeps one line and a pointer.

Anything a query can derive does not belong here. That now includes every
denominator the old draft argued from — `specdb.py audit` and `orphaned` print
them on demand, and a number quoted in prose is a number that rots.

---

## 1. Why the darkness sweep failed — and the distinction that survives it

Six gaps motivated this audit. Measured against the corpus afterward:

| Gap | Corpus atoms | Dark? |
|---|---:|---|
| Cost modification (CR 601.2f, 118) | 20+ | no |
| Casting from a non-hand zone (CR 601, 607) | 20+ | no |
| "Can't" effects (CR 101.2, 614.17, 613.11) | 20+ | no |
| Copy effects (CR 707, 712, 708, 729) | 20+ | no |
| CR 601/607 linked abilities | 20+ | no |
| **Voting (CR 701.38)** | **0** | **yes** |

Five of the six were examined in 2026-04 and then **orphaned** — classified
correctly, scoped too narrowly, and never given an owner. A darkness filter
removes them *before the sweep starts*. Only voting was dark, and it is the
only one with no atom at all.

**Darkness and ownership are different questions, and the corpus fails at
both, in opposite directions.**

```
darkness    "has anyone *looked* at this rule?"   ->  audit --dark
ownership   "does anyone *own* it?"               ->  orphaned
```

Verified in the tree: `audit --dark` catches voting and misses the other five;
`orphaned` catches the five and misses voting, which has no atom to orphan.
**Neither query over the corpus catches both** — which is why neither is the
instrument. Only 1 of the 6 was found by a query at all; the rest came from a
person asking a concrete question.

**This is the transferable lesson**, and it generalizes past this project: a
coverage metric measures the corpus, not the engine, and a corpus can be wrong
by omission *or* by staleness. The derived worklist both queries emit is
gitignored — regenerate it, never commit it.

**What the failed pass did land**, and it was worth the sessions:

- **Two generator defects.** `parse_rule_mentions` read one of the three shapes
  the corpus writes verdicts in, and the family-collapse sibling test was a
  string prefix that no lettered subrule can satisfy (`613.4a` does not start
  with `613.4.`). Together they had `audit` counting *unread* verdicts as
  unexamined rules. Fixed; the parser learned the shapes rather than the corpus
  being rewritten to match a regex.
- **A third, in `normalize_phase`** — a literal backspace where `\b` was meant,
  so two flags were permanently false. Same failure mode: a regex that silently
  matches nothing and reports a confident number anyway.
- **Corpus verdicts** ratifying judgments the sessions already held, in a shape
  the parser can see (CR 103.6, 305.9, 309–315, 713, 717).
- **`specdb.py orphaned`**, the ownership half.

> **The long-form record is [PR #69](https://github.com/lcmaier/MTG-Ichor/pull/69),
> deliberately closed rather than merged.** It is the argument for the method
> this document just dropped, kept as remote documentation of a wrong turn.
> Do not reopen it.

---

## 2. The method — the type surface

All six motivating gaps share one shape: **a type or a function could not
express what the CR requires.**

| Gap | The surface that couldn't say it |
|---|---|
| "Can't" effects | `is_blocked` was a predicate over one enum |
| Copy effects | nothing produced a Layer 1 effect |
| Cost modification | `apply_cost_modifications` is a passthrough |
| Non-hand casting | `check_cast_legality` hardcodes `Zone::Hand` |
| Voting | `DecisionProvider` is four index-shaped methods |

That is the definition of a **fact** in `codebase-state.md`'s fact/feature
triage — and **facts live in types, not in rules.** Auditing 3,120 rules
searched a space facts do not occupy.

**The triage question, unchanged from the old §3.3:**

> If this rule is true, does an existing type need a new field, or does an
> existing assumption need to become false?

- **No** → a feature. One line, move on. Do not size it, do not count cards.
- **Yes** → a **fact**. Name the type, name the phases that would encode its
  absence, and give it an owner and a back-stop.

**A "yes" is rare by construction** — six in the project's history. A sweep
that escalates twenty rows has misread the question. The test is not "is this
unimplemented", it is "would implementing this later require *unbuilding*
something".

**The unit is the type *and the functions that gate on it*.** §3 is why: one of
the six is invisible at the field level.

---

## 3. Calibration — run the question against what you already know

**A method that cannot rediscover the facts you already have is the wrong
method, and you stop rather than sweep with it.** This check is exactly what
the failed pass skipped: it never asked whether darkness would have caught the
gaps that motivated it, and the answer was no for all six.

Run 2026-08-31, before the sweep:

| Known fact | Surface | Rediscovered? |
|---|---|---|
| Provenance of an event | `ActionContext` | ✅ `new()` means "no resolution"; nothing names a source |
| Multi-component permanent (CR 729) | `BattlefieldEntity` | ✅ one `object_id` per entity |
| Counters off the battlefield (CR 122.1a/b) | `GameObject` vs `BattlefieldEntity` | ✅ `counters` is on the battlefield sidecar only |
| A second card face (CR 712) | `CardData` | ✅ flat, single-face struct |
| N-player from day one | `GameState.players`, APNAP | ✅ correctly reports **no gap** — already a `Vec` with `apnap_index` |
| **Casting from a non-hand zone** | `check_cast_legality` | ⚠️ **not at the field level** |

**Five clean, one refinement.** The last is the useful result:
`StackEntry.cast_from` already represents the fact *correctly* — the type is
fine and the **function** gating on it is not. Asking the question of fields
alone would have missed it. So the unit is the type plus its gatekeepers, which
is how §2's table was framed in the first place.

The method passes. It is also honest about its own reach: **it finds what a
type cannot say, not what a type says wrongly.** Reviews own the other half.

---

## 4. The sweep — fifteen types

Per type: read it, read the CR sections describing what it models, answer the
question. Run 2026-08-31.

| Type | What the CR wants that it can't say | Verdict |
|---|---|---|
| `GameObject` | face-down state in a non-battlefield zone (foretell, CR 702.143) | feature — a flag, same shape as `is_token` |
| `BattlefieldEntity` | CR 729 components; counters elsewhere | **known facts**, already owned |
| `CardData` | a back face (CR 712) | **known fact**, phase CV-5 |
| `AbilityDef` | a trigger condition; activation restrictions (CR 602.5d); functioning zone (CR 113.6) | trigger condition is **critical-path item 6**; the rest additive |
| `GameAction` | `Sacrifice`, `Exile`, `LoseGame`, … | feature **by contract** — one arm per variant, `CLAUDE.md` |
| `ActionContext` | who caused a non-resolution mutation | **known fact**, threaded at RA |
| `ResolutionContext` | divided/distributed amounts per target (CR 601.2d) | feature — same shape as `x_value`, already precedent |
| `EffectiveCharacteristics` | several names (CR 201.2a) | near-miss, §5.2 |
| `Effect` | durational replacement (CR 614.3) | feature — named, phase RD |
| `Primitive` | the unimplemented half of CR 701 | feature by contract |
| `Cost` | tap/sacrifice a permanent *other than* the source | feature — additive variant |
| `DecisionProvider` | a vote (CR 701.38); a card name (CR 201.4) | near-miss, §5.2 |
| `ContinuousEffect` | CR 611.2c's locked set; CR 613.8 dependency | **no gap** — `AffectedSet::Fixed` is exactly 611.2c; 613.8 is critical-path item 7 |
| `StackEntry` | **what was spent to pay the costs** | **FACT** — §5.1 |
| `ManaPool` | non-fungible mana — CR 106.6 restrictions, grants, persistence | **no gap in the type** (T12b built it); the *gatekeepers* are unwired — Deferred Migrations item 33 |

**`ManaPool` was not in the original fourteen.** It was swept 2026-08-31 after
a card-population probe (`o:"this mana"`, 227 cards) asked why the audit had
nothing to say about it — and its verdict is what §3's refined unit exists
for: the type is complete and its gatekeepers are not, which a field-level
read cannot see. **The sweep list still has no enumeration criterion**; until
it states one, population probes of exactly that shape are the check on it.

---

## 5. Findings register

One line per finding, with a pointer to where it actually lives.

### 5.1 The fact — cost-payment provenance

**`StackEntry` records *that* a cost was paid, never *what paid it*.**
`chosen_alternative_cost` and `additional_costs_paid` hold the cost
*definitions*; both are written at cast and read by no production code. No
field anywhere records which mana was spent or which objects were spent.

Both are **destroyed by payment and unrecoverable afterward** — the provenance
shape exactly. The CR asks for them in at least five places:

| Rule | Needs |
|---|---|
| CR 400.7d | a permanent referencing the costs paid for the spell it was |
| CR 702.44a/b | Sunburst — counters per *color of mana spent* |
| CR 707.10 | a copy uses **the original's** paid objects (Fling) |
| CR 107.4h | snow `{S}` — mana *from a snow source* spent on a cost |
| CR 700.14 | expend N — mana spent to cast spells *this turn* |

**Why it is a fact and not a feature — and where it actually lands.** The
corpus already scheduled the readers, and it named this dependency first:
ATOM-702.44a-001 (sunburst) is ticketed *"DEFERRED — Phase 8. Requires
mana-color-spent tracking."* So **RC does not read this**, and an earlier draft
of this section back-stopped it there wrongly.

The earliest reader is **CV's spell-copy work** (CR 707.10, atoms under D5,
superseded by `copy-effects-architecture.md`), then **item 6** for CR 700.14's
expend, then **Phase 8** for sunburst itself.

**CR 707.10 splits the fact in half, and that is the part that must be designed
rather than bolted on.** A copy inherits the *objects* used to pay the
original's costs — the Fling case the CR spells out by name — but **not** the
mana, because "mana isn't an object" (the Dawnglow Infusion example). Both
halves are already atoms: ATOM-707.10-002 and ATOM-707.10-003. A copy spine
that treats cost-payment provenance as one undifferentiated blob gets this
wrong in one direction or the other.

`ManaPool.last_spent_grants` is already a partial, transient version of the
mana half — the engine knows it needs *something* here and drains it after one
`pay_with_plan` call.

**Not a rewrite.** `x_value` is the precedent and the rail: captured at cast,
carried `StackEntry` → `BattlefieldEntity` on resolution. This rides it, and
the rail survives RC-2's ETB rewrite either way.

→ **Owner: `codebase-state.md` Deferred Migrations item 30. Capture is
  independent of every scheduled phase and cheap whenever. The design
  constraint lands at CV — the 707.10 split above — which is where the
  back-stop belongs.**

### 5.2 Near-misses

Recorded because §2 says a "yes" is rare, and a register is only credible if it
says what it looked at hardest. Each answered **no** — each is additive, and
none makes an existing assumption false.

- **CR 201.4, choose a card name.** `DecisionProvider`'s four methods are all an
  index into a supplied list or a number in a range; "the name of a card in the
  Oracle reference" has no list. A fifth method is additive across five impls.
  **This is voting's gap again** — whether the trait carries the CR's *choice
  shapes* — and 201.4 is a second witness for one still unowned.
- **CR 204.2, color indicator.** `CardData.color_indicator` has **zero readers
  and zero writers**; `compute_characteristics` seeds Layer 5 from `card.colors`
  alone. Dead forward-looking scaffolding — Deferred Migrations, not a fact —
  and its own doc comment names the phase that makes it live (CV-5).
- **CR 607.4, an ability in more than one linked pair.** Constrains the
  representation before it exists: a link cannot be one `Option<AbilityId>` on
  `AbilityDef`. A free constraint for whoever takes linked abilities.
- **CR 201.2a / 612.7, an object with several names.** `Characteristics.name` is
  one `String`; 201.2a's "at least one name in common" wants a set. No on
  population, not on principle: exactly one comparison site exists (`sba.rs`'s
  CR 704.5j legend grouping), and both drivers — Spy Kit, name stickers — need
  Layer 3, which is unbuilt.
- **CR 717.6, an explicit exception to CR 614.5.** Attractions are out of scope,
  but 717.6's replacement "may apply more than once to the same event. This is
  an exception to rule 614.5." **The CR carves exceptions out of 614.5**, so
  whatever identity key RD settles on must express "exempt from the applied
  set" — the same shape CR 903.9b already needs.
- **CR 305.9, a land that is also another type.** Checked rather than assumed:
  `castable_spells` skips any card whose printed types include Land, so a Land
  Creature is never offered as a spell; `play_land` accepts it because it tests
  *for* the Land type rather than for Land alone. Both halves hold. Enforcement
  is at the enumeration boundary, which is "performers are loud; callers check
  legality" working as designed — worth stating because the obvious place to
  look for the guard is the wrong one.

### 5.3 Inherited, and now inventoried

Not findings of this audit — findings it inherited. All three have a home as of
2026-08-31: **`plans/backlog.md`**, which is where they are maintained. Kept here
as the register's record of what it handed over.

- **Cost modification** (CR 601.2f, 118). `apply_cost_modifications` is a
  passthrough stub with a test asserting so; `replacement-architecture.md` §9
  says it "needs a phase marker of its own … and it is not small". → backlog §2.1
- **Casting from a non-hand zone** (CR 601, 607). `check_cast_legality`
  hardcodes `Zone::Hand`. → backlog §2.3
- **Voting** (CR 701.38), and CR 201.4 with it — `DecisionProvider`'s choice
  shapes. Zero atoms, and the only one of the six that was genuinely dark.
  → backlog §2.4

**One correction, made while filing them: the "CR 601, 607" pairing above names
the verdict, not the atoms.** It reads as one mechanic and is two. No CR 601 or
607 atom under a shipped phase concerns a non-hand zone at all — those are filed
at Phase 8, correctly — and the 43 that are there decompose into casting-procedure
depth (25), **linked abilities (10, which no plan doc mentioned)**, cost pipeline
(7) and one already covered. Linked abilities was invisible precisely because it
was travelling under another mechanic's section number. → backlog §2.2, §3.

---

## 6. Confirmations, not instruments

The queries stay. They stopped being the method and became the check.

```bash
python plans/specdb.py orphaned    # shipped-phase behavior no test and no doc owns
python plans/specdb.py owed        # the phase-exit gate — must be clean to close
python plans/specdb.py audit --dark --families   # darkness; in-scope surface is 0
```

- **`owed` is a gate**, not a report: a phase does not close until it is clean.
- **`orphaned` triages by cluster, never by atom**, and it **pre-sorts the read
  that used to be its whole cost.** Separating "a missing `// COVERS:` on code
  that exists" from "genuinely unbuilt" is the expensive half, and a *source*
  citation is the proxy: code citing a rule has encoded some assumption about
  it. Today, of 406 orphaned atoms across 66 sections — **74 cited in `src/`**
  and **332 cited nowhere**, the latter being the backlog's upper bound.
  `--bucket cited|unbuilt` lists one; sections rank by the unbuilt count,
  because a section that is mostly cited is not mostly work.

  **A pre-sort, never a verdict**, erring in both directions: a comment can cite
  a rule the code contradicts — `check_cast_legality` cites CR 117.1a and still
  hardcodes `Zone::Hand` — and behavior can exist uncited. Calibrated on the
  hardest available case, CR 613 under the shipped layer phases: it withheld
  613.6 (cited by two card files) and flagged 613.8c, the dependency algorithm,
  which is unbuilt and is critical-path item 7. Confirm a cluster before acting
  on its bucket.

  **How weak the `cited` half is, measured.** Of the 18 whose citation sits in
  `mtgsim/tests/`, exactly **three** turned out to be real: the rest cite a rule
  in an assertion message or an explanatory comment while testing something
  else. `test_a_fizzling_spell_moves_through_the_chokepoint` discusses CR 608.3a
  to explain what it is *not* doing. So `cited` splits again — **18 sit in
  `tests/`** (a possible annotation) and **54 only in `src/`** (behavior with no
  test at all, which needs a test *written*, not annotated). Neither half
  reduces the 332.

  *Measured before D3a's re-file, which moved four cited atoms; the split reads
  15/55 today. `backlog.md` §5 carries the live figure — this one is the record
  of the run that produced the finding.*

  **This doc is excluded from its own ownership set**, and the reason is a bug
  it caused: writing §5.1 made `orphaned` treat CR 117.1a, 601.2f and 17 others
  as *owned*, because the third filter reads any plan-doc mention as a design
  claiming the rule. An instrument must not satisfy its own filter — the query
  was deflating by the act of being documented. `audit` still counts this file,
  since for *darkness* a mention really is somebody looking.

  **`backlog.md` is excluded too, for a different reason**, and it is the more
  important of the two because that file's job is to name every rule this query
  finds. Its citations are genuine claims, not examples — but it is `orphaned`'s
  *output*, so leaving it in makes the query converge to zero by being written.
  A gate any prose can satisfy is not a gate; contrast `owed`, which needs a
  test. The burn-down still happens and is driven by the right thing: a mechanic
  graduating to an architecture doc. **Design claims the rule; listing it does
  not.** → `backlog.md` §1, which also measures the trap — section-level prose
  claims nothing, and of the 332 only 2 atoms cite more than one rule.
- **`audit --dark` is retired as an instrument.** Its in-scope surface is 0 and
  the remaining darkness is a *depth* gap, overwhelmingly CR 702 keyword
  subrules. That is corpus authoring — writing atoms for rules that carry a
  verdict but have none — it belongs beside the phases that need it, and **it is
  not scheduled here.** Recording that is what stops a type sweep sliding into
  it. (Older drafts and PR #69 call this **"Pass B"**, against a **"Pass A"**
  fact sweep. Both names are retired with the A/B split; the activity is not.)

Citations in both queries match per **rule**, never per section: `plans/*.md`
say "CR 702" constantly, and a section-level test hands every keyword ability
to five documents at once — which is how an ownership query silently turns back
into a darkness one.

---

## 7. Documents this owes

- **Settled.** §5.1 is `codebase-state.md` Deferred Migrations item 30 — its
  back-stop is **CV**, not RC (`77bda5e`). §5.3's three are `backlog.md` §2.
- **Open — the second vocabulary, now three rules.** `DUPLICATE` (305.9) was
  the fourth and is **settled**: `1f2c8da` restated it as `ALREADY-IMPLEMENTED`
  with the duplication explained in prose, which is the worked example for the
  rest. Left: `PARTIALLY DEFERRED` (108.5) and `ALREADY-HANDLED-BY-DESIGN`
  (732.1, 732.2). Either normalise them the same way or admit a ninth verdict.

  **108.5 is the one with a concrete defect**, not just an off-vocabulary label:
  the legacy fallback's `([A-Z][A-Z-]+)` stops at the space, so `rule_mentions`
  holds the verdict `PARTIALLY` — a truncation that is in no vocabulary at all
  and reads as a word rather than a disposition. Not urgent: the fallback keeps
  all three rules *visible*, so none is dark, and `audit` never gates on the
  label's value.
- **Open — ~100 rules whose only verdict is a session's summary table.** A
  summary table is *derived*, and six rules already disagree with their own body
  verdict (305.3, 310.4, 400.12, 402.3, 700.4, 701.12d). Teaching the parser to
  read tables would make `audit` depend on a restatement that is out of date in
  six places. **The body is the record; the table is a view** — so this is
  corpus work, and the disagreements resolve in the body's favour.
