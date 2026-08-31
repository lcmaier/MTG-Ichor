# Backlog — everything off the critical path

`CLAUDE.md`'s **Critical path to v1** owns the ordered spine: seven numbered
items, cited by number from four other documents, and deliberately short. This
file owns **the rest** — every mechanic the engine will need that the spine does
not schedule.

It is an **inventory, not a schedule.** Nothing here has a date, and the order of
the entries carries no priority. What each entry does carry is the one thing that
was expensive to recover: *the surface that cannot express the rule*, which is
the finding `cr-coverage-audit.md` §2 was built to produce.

---

## 0. What an entry is

Six fields, and no seventh:

| Field | What it says |
|---|---|
| **Rules** | the CR sections, at the granularity actually claimed — see §1 |
| **Verdict** | the type or function that can't say what the CR requires |
| **Size** | rough, in phases or in the project's 1,500–2,500-addition PR band |
| **Blocks** | what stays unwritable until it lands |
| **Atoms** | corpus atoms filed under `Backlog`, and where the rest of them are |
| **Owner** | the doc that will take it — blank until one does |

**An entry is not a design.** One mechanic, a few lines. When a mechanic is
actually designed it *graduates*: an architecture doc takes it (extending
`CLAUDE.md`'s authority-table row, never adding one), that doc's rule citations
claim the rules, and the entry here shrinks to a pointer. That graduation is the
only thing that should shrink `orphaned` — see §1.

**Sizing precedes writing.** Two estimates in this workstream ran low (D1 called
~20 lines, landed +58/−17; D2a called 18 annotations, produced 3). A size here is
a starting guess, not a commitment, and it is re-derived from a live query before
anyone schedules it.

---

## 1. Why this file is invisible to `orphaned`

`specdb.py orphaned`'s third filter is *"no plan doc cites its rule"*, and it
reads any mention in `plans/*.md` as ownership. This file is `orphaned`'s output
and its job is to eventually name every rule the query found — so left in the
scan, the query would converge to zero **by being written**, not by anything
being owned. `backlog.md` is therefore in `_scan_citations`'s `exclude_docs`,
beside `cr-coverage-audit.md`.

**The reason is not the audit doc's**, and the difference is worth stating
because the arguments look identical. The audit doc is excluded because its
citations are *examples* — CR 117.1a appears there only to show where the
source-citation proxy is weak — so its mentions are a false ownership signal. A
backlog entry's citations are genuine claims. It is excluded anyway, for a
structural reason:

- **A gate any prose can satisfy is not a gate.** Contrast `owed`, which needs a
  `// COVERS:` and therefore a test. If listing a rule satisfied `orphaned`, its
  completion signal would be guaranteed to fire regardless of whether the
  engineering happened.
- **The 332 must stay re-derivable after the inventory exists.** It is the only
  way this file can be checked against its own source. An inventory that erases
  its evidence cannot be audited for what it left out.
- **The burn-down still happens, driven by the right thing.** When a mechanic
  graduates to an architecture doc, that doc claims the rules and `orphaned`
  shrinks legitimately. *Design claims the rule; listing it does not.*

**Measured, so the size of the trap is known rather than feared.** Citations
match per rule (`\d{3}\.\d+[a-z]?`), never per section, so section-level prose —
"CR 107", "CR 118" — claims **zero** atoms either way; only an exact rule number
bites. And collateral is near-nil: of the 332 unbuilt atoms, **2** cite more than
one rule and **0** cite more than one section. The exclusion matters for the
rule-level citations this file will accumulate as it fills, not for today's text.

---

## 2. Entries

### 2.1 Cost modification and the cost pipeline

- **Rules** — CR 107.3, 107.4, 107.6; 118.6–118.9; 202.3; 601.2f–601.2h, 601.7
- **Verdict** — `apply_cost_modifications` is a passthrough stub with a test
  asserting so. `ManaPool::pay` has no hybrid and no Phyrexian branch; seven
  atoms carry a `NEW` ticket naming exactly that. `StackEntry`'s
  `chosen_alternative_cost` and `additional_costs_paid` are written at cast and
  read by no production code.
- **Size** — not small, and it wants splitting: cost *representation and
  payment* (hybrid, Phyrexian, `{Q}`, mana value with X / hybrid / Phyrexian) is
  separable from cost *modification* (reduction, increase, the CR 601.2f lock-in,
  and the reduction-ordering choice 601.2f-004 hands to a `DecisionProvider`).
  Two phases is the honest guess. `replacement-architecture.md` §9 already says
  it "needs a phase marker of its own … and it is not small".
- **Blocks** — kicker and every additional-cost keyword; alternative costs, and
  so the whole cast-from-elsewhere family in §2.3; the Commander cost-modification
  track `CLAUDE.md` interleaves after critical-path item 5.
- **Atoms** — 33 re-filed to `Backlog`, from sessions S1, S2, S5. 28 further
  atoms in CR 107/118/202 stay on their shipped phase: their `Mechanism` names a
  function that exists and does the thing (`ManaPool::spend()`, `pay_life()`,
  `check_cost_resource`), so they are missing a test, not missing behavior.
- **Owner** — none yet.

### 2.2 Linked abilities (CR 607)

- **Rules** — CR 607 entire
- **Verdict** — `AbilityDef` carries no link, and **nothing designs one**. CR 607
  is *named* in three plan docs — `codebase-state.md`, `replacement-architecture.md`
  twice (a Phase 6 component, and a CR 614.14 dependency already ticketed `T20`),
  and audit §5.2 — but all four mentions are dependency notes. **None carries a
  rule-level citation**, which is why the atoms survived `orphaned`'s ownership
  filter and is the filter behaving exactly as specified: citations match per
  rule, never per section, so "CR 607" claims no atom of CR 607.2a. The shape is
  already constrained: §5.2's CR 607.4 near-miss establishes that a link cannot
  be one `Option<AbilityId>`, because an ability may be in more than one pair.
- **Size** — one phase, and it is a *data*-lifetime problem rather than a
  resolution one: per-ability state that survives a zone change (607.2d-002) and
  is per-pair rather than per-object (607.2a-002's two independent exile sets).
- **Blocks** — the O-Ring / Banisher Priest exile-and-return pattern; abilities
  that read whether an additional cost was paid (kicker's second half, 607.2i);
  "the chosen [value]" cards (607.2d). Overlaps §2.1 at 607.2i/607.2j, which
  read a cost the cost pipeline does not yet record.
- **Atoms** — 10 re-filed to `Backlog`, all session S5; 20 in the corpus, the
  remainder correctly filed at Phase 8 and later.
- **Owner** — none yet.

### 2.3 Casting from a non-hand zone

- **Rules** — CR 601.3, 601.3f, 117.1a; the CR 702 cast-from-elsewhere keywords
- **Verdict** — `check_cast_legality` hard-codes `Zone::Hand`, and cites
  CR 117.1a while doing it. The *type* is already right: `StackEntry.cast_from`
  represents the fact correctly, which is why audit §3's calibration flagged this
  one at the function level and not the field level. The gate is the gap.
- **Size** — small at the gate, large in what the gate admits; the keywords
  behind it are Phase 8 card breadth, not one phase.
- **Blocks** — flashback, escape, jump-start, aftermath, foretell, plot, warp,
  discover, airbend. Most also need §2.1, because they are alternative costs.
- **Atoms** — **none re-filed, and that is the finding.** Every atom about
  casting from a non-hand zone (601.2a-003, 601.3f-001/002, and the CR 702
  keyword family) is already filed at Phase 8, correctly. See §3's second note.
- **Owner** — none yet.

### 2.4 Voting, and the `DecisionProvider` choice shapes

- **Rules** — CR 701.38 (voting); CR 201.4 (choose a card name)
- **Verdict** — `DecisionProvider` is four index-shaped methods: an index into a
  supplied list, or a number in a range. A vote is neither, and "the name of a
  card in the Oracle reference" has no list to index. The real question is
  whether the trait carries the CR's *choice shapes* at all; 201.4 is a second
  witness for the same gap (audit §5.2).
- **Size** — small per method, multiplied by five implementations, plus one
  design decision about the trait's shape that should be taken once rather than
  per-method.
- **Blocks** — Council's Judgment and the will-of-the-council / council's-dilemma
  cycle; Pithing Needle, Meddling Mage, Runed Halo for 201.4.
- **Atoms** — **zero, in the whole corpus.** This entry is invisible to `owed`,
  `orphaned` and `audit` alike, and stays that way: it was the only one of the
  six motivating gaps that was genuinely *dark*. Authoring atoms for it is corpus
  work — writing atoms for a rule that carries a verdict but has none — and that
  is explicitly unscheduled (audit §6).
- **Owner** — none yet.

---

## 3. Not triaged yet

```bash
python plans/specdb.py orphaned --bucket unbuilt --all
```

**297 atoms across 63 sections** as of 2026-08-31, down from 332/64 by this
file's re-file. Regenerate rather than trusting that number — it is a query, and
a number in prose rots. The largest remaining clusters are CR 701 (24), CR 702
(20), CR 205 type-changing effects (15), CR 306 loyalty (10), CR 602 activating
abilities (9). Each needs one judgment: real backlog item, rough size, what it
blocks.

Two things learned while seeding §2, recorded so they are not re-derived:

1. **A CR section is a loose proxy for a mechanic, and it over-collects.** The
   CR 107/118/202 shipped-phase bucket reads as one cluster and is at least four:
   cost modification, exotic mana-symbol payment, mana-value computation, and
   Layer 5 colour derivation — plus a fifth group that is genuinely implemented
   and merely untested. 28 of its 54 atoms stayed put for that reason. **Triage
   from the atom's `Mechanism` field**, which names the function, rather than
   from its rule number.

2. **CR 601 is not "casting from a non-hand zone".** Not one of the 43
   shipped-phase CR 601/607 atoms concerns a non-hand zone; they are
   casting-procedure depth (25, still shipped, and D3b's largest single cluster),
   linked abilities (10, §2.2), cost pipeline (7, §2.1) and one already covered.
   The pairing "CR 601/607" traces to `codebase-state.md`'s detector write-up,
   which bundles the `Zone::Hand` hard-code and linked abilities into one bullet.
   They are two mechanics that share no rule, no type and no phase.

The 25 remaining CR 601 casting-procedure atoms — modal announcement,
kicker-conditional targets, divide-or-distribute (CR 601.2d, which audit §4
already sizes as "same shape as `x_value`"), the 601.2g mana window, 601.4's
look-ahead — are the obvious next entry and were left out of §2 deliberately:
their verdicts are not yet established, and establishing them is triage, not
seeding.

---

## 4. What this file does not cover

- **The critical path** — `CLAUDE.md` items 1–7, and the Commander/multiplayer
  track interleaved after item 5. Cited by number elsewhere; do not restate them
  here.
- **Deferred migrations** — debt owed by scaffolding already in the tree lives in
  `codebase-state.md`, which wins over every other doc on current state. A stub
  with a `TODO` is that file's; a mechanic with no code is this one's.
- **Missing tests for behavior that exists** — the `cited` half of `orphaned`
  (`--bucket cited`), which needs a test written or a `// COVERS:` added, not a
  backlog entry. Audit §6 measures how weak that signal is.
- **Corpus authoring** — rules carrying a verdict but no atom, overwhelmingly
  CR 702 keyword subrules. Unscheduled, and it belongs beside the phases that
  need it.
