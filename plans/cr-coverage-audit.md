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

## 4. The sweep — fourteen types

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

**Why it is a fact and not a feature.** Sunburst is an ETB replacement, so
**RC** would encode its absence; CR 707.10 is a copy rule, so **CV-2** would
encode it again; CR 700.14 is a trigger, so **item 6** would make it three.
`ManaPool.last_spent_grants` is already a partial, transient version of this —
the engine knows it needs *something* here and drains it after one call.

**Not a rewrite.** `x_value` is the precedent and the rail: captured at cast,
carried `StackEntry` → `BattlefieldEntity` on resolution. This rides it.

→ **Owner: `codebase-state.md` Deferred Migrations item 30. Back-stop: the
  first reader — sunburst at RC, or CR 707.10 at CV-2, whichever lands first.**

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

### 5.3 Still unowned

Not findings of this audit — findings it inherited, and none has an owner yet.

- **Cost modification** (CR 601.2f, 118). `replacement-architecture.md` §9 says
  it "needs a phase marker of its own … and it is not small";
  `apply_cost_modifications` is a passthrough stub with a test asserting so.
- **Casting from a non-hand zone** (CR 601, 607). `check_cast_legality`
  hardcodes `Zone::Hand`; linked abilities is 20 atoms, all uncovered.
- **Voting** (CR 701.38), and CR 201.4 with it — `DecisionProvider`'s choice
  shapes.

---

## 6. Confirmations, not instruments

The queries stay. They stopped being the method and became the check.

```bash
python plans/specdb.py orphaned    # shipped-phase behavior no test and no doc owns
python plans/specdb.py owed        # the phase-exit gate — must be clean to close
python plans/specdb.py audit --dark --families   # darkness; in-scope surface is 0
```

- **`owed` is a gate**, not a report: a phase does not close until it is clean.
- **`orphaned` triages by cluster, never by atom.** It cannot separate a missing
  `// COVERS:` on code that exists from missing behavior — only reading the code
  does that.
- **`audit --dark` is retired as an instrument.** Its in-scope surface is 0 and
  the remaining darkness is a *depth* gap, overwhelmingly CR 702 keyword
  subrules. That is corpus authoring, it belongs beside the phases that need it,
  and **it is not scheduled here.** Recording that is what stops a type sweep
  sliding into it.

Citations in both queries match per **rule**, never per section: `plans/*.md`
say "CR 702" constantly, and a section-level test hands every keyword ability
to five documents at once — which is how an ownership query silently turns back
into a darkness one.

---

## 7. Documents this owes

- **`codebase-state.md`** gets §5.1 as a Deferred Migrations entry, back-stopped
  before RC.
- **Open — the second vocabulary.** Four rules carry a disposition outside the
  eight `specdb` knows: `DUPLICATE` (305.9), `PARTIALLY DEFERRED` (108.5),
  `ALREADY-HANDLED-BY-DESIGN` (732.1, 732.2). Either normalise them or admit a
  ninth — `DUPLICATE` says something the eight cannot. The parser keeps all four
  visible via its legacy fallback, so this is not urgent.
- **Open — ~100 rules whose only verdict is a session's summary table.** A
  summary table is *derived*, and six rules already disagree with their own body
  verdict (305.3, 310.4, 400.12, 402.3, 700.4, 701.12d). Teaching the parser to
  read tables would make `audit` depend on a restatement that is out of date in
  six places. **The body is the record; the table is a view** — so this is
  corpus work, and the disagreements resolve in the body's favour.
