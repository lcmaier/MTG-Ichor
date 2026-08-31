# CR Coverage Audit — is the plan complete against the frozen rules?

**Status:** planned, not started. This document is the target a cold audit
session executes from; it owns the method, the denominator and the session
split. `codebase-state.md` → "Was the critical path complete?" is the parent —
this is the systematic version of the detector that section describes.

**Baseline:** `MTG-Rules/versions/tmnt.txt`, 3,120 rules, effective 2026-02-27.
Every number below was measured against that tree on 2026-08-31 and is
reproducible from `plans/atomic-tests/spec.sqlite` plus a grep. **The freeze is
what makes the result durable** — against a moving CR it would rot.

---

## 0. The budget — why this stays tight

Same rule the other planning docs carry. This document exists to make audit
sessions *cheap and resumable*, so anything a session can derive from the
generator does not belong here. What belongs here is the method, the
disposition vocabulary, the session split, and the register of what the audit
has already found. Numbers live here only where they are the *argument*.

**It must not become a findings dump.** A finding that needs action goes to
`codebase-state.md`'s Deferred Migrations or gets an owning architecture doc;
§8 is a one-line register with pointers, not a second home for the content.

---

## 1. Verdict — worth doing, and how much

### 1.1 Do more than the critical path, but not a full coverage sweep

**Yes**, and the evidence is that the detector has now paid out five times in
four days: "can't" effects (1,857 cards, no owner), copy effects (1,628 cards,
no owner), cost modification (~903 cards, named but homeless), casting from a
zone other than hand (~764 cards, no owner), and voting (CR 701.38, **zero
corpus atoms** — never examined at all).

**But not by classifying all 3,120 rules.** That is the audit you get by
default and it is the wrong shape: it spends most of its effort on rules whose
disposition is `PURE-DEF`, and it treats a definitional gap and a structural
one as the same row. Scope it instead to the question that actually carries
risk.

### 1.2 What the audit is for: facts, not features

`codebase-state.md` already states the discriminator and it is the whole basis
for this document:

> Both misses are **features** on this file's own fact/feature triage, not
> facts. Nothing had to be unbuilt … **Keep auditing for facts; let features be
> found late.**

- A **feature** is a mechanic the engine can grow into. `is_blocked` was
  correct-but-narrow and the restriction model extends it; `apply_cost_modifications`
  is a passthrough stub that becomes a real function. Late discovery costs a
  phase, not a rewrite.
- A **fact** is a shape every later phase encodes. Provenance ("who caused this
  event") cost one `Option<SourceFilter>` field **only because** Phase RA
  threaded it through `ActionContext` first; had it not, Sigarda alone would
  have been a re-thread through every system built since.

**So the audit's job is to find the remaining facts**, and a feature it happens
to surface is a bonus rather than the point. That reframing is what turns 3,120
rules into §5's two sessions.

**Facts in this codebase have a recognizable shape.** Every one found so far is
either a *type* many phases read or a *context* threaded through many calls:

| Fact | Where it lives | Found |
|---|---|---|
| Provenance of an event | `ActionContext` | early (RA) — cheap |
| N-player from day one | every new system | early (owner decision) — cheap |
| Multi-component permanent (CR 729) | `BattlefieldEntity` | 2026-08-29, back-stopped before Phase 8 |
| Counters off the battlefield (CR 122.1a/b) | `GameObject` vs `BattlefieldEntity` | 2026-08-30, Deferred Migrations 23 |
| Casting from a non-hand zone | `check_cast_legality`, every cast path | 2026-08-31, **unowned** |
| A second card face (CR 712) | `CardData` | 2026-08-29, phase CV-5 |

### 1.3 Why now rather than at the Phase 8 back-stop

The obvious slot is beside CR 613.8 and CV-7, which already back-stop Phase 8
card breadth. **Pass A should run earlier than that**, for a specific reason:
three of the type shapes a fact-sweep would examine are about to be modified by
already-scheduled phases.

- **CV-5** adds a back face to `CardData`.
- **CV-7** makes `BattlefieldEntity` multi-component.
- **RD** extends the `GameAction` vocabulary and settles CR 614.5's identity
  key (`replacement-architecture.md` §11 item 15).

A fact discovered *after* those land is a fact discovered after three phases
encoded its absence. A fact discovered before costs a field. **Pass A is two
sessions and it buys the right to run CV-5, CV-7 and RD without wondering.**

Pass A does **not** block RS-0, RS-1, RC-1 or RC-2 — all four are small,
net-deleting or narrow, and none introduces a type shape.

---

## 2. Scope, measured

### 2.1 The denominator

| | Rules | |
|---|---:|---|
| CR rules in `tmnt.txt` | **3,120** | the frozen baseline |
| Examined by the corpus — has an atom **or** a classification verdict | 1,747 | 56% |
| Unexamined | 1,373 | 44% |
| …and cited nowhere at all: no atom, no verdict, no source comment, no plan doc | **1,315** | the dark zone |

`specdb gaps` reports a narrower 155 "NEVER SEEN", which counts only rules no
session file ever *mentioned*. The 1,315 above is the stricter and more useful
figure: a rule can be name-dropped in a session and still never assessed.

### 2.2 Where the dark rules are

The 1,315 collapse into **547 wholly-dark rule families** (`NNN.M`), which is
the triage unit. After pruning out-of-scope variants (§2.4): **482 in-scope**.

```
CR 7  Additional Rules          266      CR 6  Spells/Abilities/Effects   33
CR 3  Card Types                 85      CR 2  Parts of a Card            31
CR 8  Multiplayer                47      CR 1  Game Concepts              17
                                          CR 9  Casual Variants (903 only)  3
```

### 2.3 Two collapses that make this tractable

**Most rows are dispositions, not work.** Of the 1,035 rules already carrying a
verdict, only **492 came back `TESTABLE`** — 48%. The rest are `PURE-DEF` (336),
`DEFERRED` (92), `OUT-OF-SCOPE` (37), `BOUNDARY-DEF` (35), `ALREADY-IMPLEMENTED`
(32), `META` (5), `LKI` (3). Applying that rate to 482 projects **~229 rows that
produce anything at all**, and those cluster into far fewer mechanics.

**CR 7's 266 is shallower than it reads.** Measured directly: of CR 702's **190
keyword-ability families, 180 already have an examined subrule**. The darkness
there is at *depth*, not at the family level, and a depth gap is a coverage
question rather than a planning one. Only **10 families are wholly untouched**,
and four of them matter for a 4-player Commander target:

> **Partner** (702.124, 14 subrules) · **Daybound/Nightbound** (702.145, 8) ·
> **Disturb** (702.146, 3) · **Warp** (702.185, 4)
>
> The other six — Forecast, Hidden Agenda, Space Sculptor, Visit, More Than
> Meets the Eye, Harmonize — are niche or variant-bound and are Pass B's.

CR 7's dark families are therefore dominated by **CR 700** (general) and **CR
701** (keyword *actions* — proliferate, populate, monstrosity, vote), not by
keyword abilities.

### 2.4 What is out of scope, and why it is a prune rather than a judgment

65 dark families, removed by rule prefix rather than by opinion:

- **CR 901–910** — Planechase, Vanguard, Archenemy, Conspiracy and the rest.
  Commander (903) stays; `CLAUDE.md` names it as a v1 target.
- **CR 407** — ante.
- **CR 100.6, 100.7** — tournament and casual-play framing.
- **CR 801** — limited range of influence. An *optional* multiplayer rule; CR
  800 and 802, which the critical path does name, are not pruned.

---

## 3. The method

### 3.1 The five-way join

Every row is one CR rule (or family) and five mechanical columns. All five were
prototyped on 2026-08-31 and work:

| # | Column | Source | Coverage today |
|---|---|---|---|
| 1 | has a corpus atom? | `spec.sqlite` → `atoms.rule_num` | 1,240 rules |
| 2 | has a classification verdict? | `spec.sqlite` → `rule_mentions.verdict` | 1,035 rules |
| 3 | cited in Rust source or tests? | grep `\b(CR\|rule)\s+NNN` over `mtgsim/` | 269 rules |
| 4 | cited in a plan doc? | same grep over `plans/*.md` | 261 rules |
| 5 | card population | Scryfall, per mechanic (CR 701/702) | on demand |

Columns 3 and 4 are the ones the existing tooling lacks, and they are what
separate "nobody looked" from "the code already assumes something."

### 3.2 `specdb.py audit` — the generator's contract

**Session 0 builds this and nothing else.** It is what makes every later session
cheap, resumable and re-runnable, exactly as `specdb` did for coverage: the
audit stops being prose and becomes a query.

```
python plans/specdb.py audit                 # the full five-column table
python plans/specdb.py audit --dark          # only rules with none of columns 1-4
python plans/specdb.py audit --chapter 8     # restrict
python plans/specdb.py audit --families      # collapse to NNN.M, the triage unit
```

Requirements:

- **Derived, never hand-edited** — same rule the rest of `spec.sqlite` carries.
- **Deterministic output order** (rule number), so two runs diff cleanly.
- **The out-of-scope prune is data, not code**: a literal prefix list in the
  script with §2.4's four entries, so extending it is a one-line diff with a
  reason beside it.
- **Emits the disposition vocabulary** already in `rule_mentions` (`TESTABLE`,
  `PURE-DEF`, `DEFERRED`, `OUT-OF-SCOPE`, `BOUNDARY-DEF`, `ALREADY-IMPLEMENTED`,
  `META`, `LKI`) so a triage session writes verdicts the corpus already
  understands, rather than inventing a second vocabulary.
- **Re-runnable as a gate.** Once it exists, `audit --dark --families | wc -l`
  is a number that should only ever go down, and it belongs beside
  `specdb owed` in the phase-close checklist.

Sized against its siblings: `specdb.py` already owns the DB, the CR ingest and
the grep-adjacent `orphans`/`suspicious` commands, so this is a subcommand of
roughly 150 lines, not a new tool.

### 3.3 The one question each row is triaged on

Pass A asks exactly one thing, and the discipline is that **most rows answer it
in seconds**:

> **If this rule is true, does an existing type need a new field, or does an
> existing assumption need to become false?**

- **No** → it is a feature or a definition. Write the disposition, move on.
  Do not size it, do not count cards, do not look for an owner.
- **Yes** → it is a **fact**. Stop and do the full treatment: name the type,
  name the phases that would encode its absence, and give it either a
  Deferred Migrations entry with a back-stop or an owning doc.

**A "yes" is rare and that is the point.** Six facts in the project's whole
history (§1.2's table). A session that escalates twenty rows has misread the
question — the test is not "is this unimplemented", it is "would implementing
this later require unbuilding something".

---

## 4. The two passes

### 4.1 Pass A — the fact sweep (do this)

482 in-scope dark families, triaged on §3.3's single question. Output is a
disposition per family plus, for the rare `fact`, an escalation.

**Exit criterion:** every in-scope dark family carries a disposition, and every
`fact` has either an owning doc or a Deferred Migrations entry with a
back-stop. **Not** "every rule has an atom" — that is Pass B.

### 4.2 Pass B — the completeness sweep (optional, incremental, later)

Turning dispositions into atoms, and closing the depth gaps §2.3 identifies —
CR 702's 180 examined-at-the-headline families. This is corpus authoring, it is
the thing the atomic-test sessions already do, and it should happen
incrementally beside the phases that need it rather than as a project.

**Pass B is explicitly not scheduled here.** Recording that it exists, and that
Pass A does not do it, is what stops a Pass A session sliding into it — which
is the failure mode that would turn two sessions into fifteen.

---

## 5. Session plan

**Sized before writing, split in the doc** — `engineering-practices.md` §4.

| Session | Shape | Size | Risk |
|---|---|---|---|
| **A-0 — the generator** | `specdb.py audit` per §3.2, and nothing else. No triage, no findings | ~150 lines, one file, reuses the DB | low — its check is that column totals reproduce §2.1 |
| **A-1 — CR 8, CR 6, CR 1, CR 2** | 128 families. CR 8 **first**: v1 is 4-player Commander and multiplayer is the least-examined *relevant* chapter | 128 rows, mostly definitional | medium — CR 8 is where an N-player fact would hide |
| **A-2 — CR 7 and CR 3** | 351 families. CR 7 collapses hard (§2.3); CR 3 is card-type definitions and is fast | 351 rows, high collapse rate | medium |

**Three sessions, not five to seven.** The earlier estimate was for a full
coverage classification; scoping to §3.3's question is what removes the rest.

**Ordering argument.** A-0 first because its output *is* the handoff — A-1 and
A-2 start cold from a generated table rather than a blank page, which is the
same property `rb-review.md` was built for. CR 8 leads A-1 because it is the
only chapter where the target use case (4-player Commander) and the dark-family
count point at the same place.

**What each session must not do.** A-0 must not triage. A-1 and A-2 must not
author atoms (that is Pass B), must not size features, and must not fix
anything — a fact-sweep that starts fixing is the scatter
`plans/handoffs/rb-review.md` exists to prevent.

---

## 6. What this audit cannot tell you

Stated up front because the value claim depends on it.

- **It bounds unknown-unknowns; it does not validate known-knowns.** A rule with
  an atom *and* a verdict can still be mis-modelled. This finds what nobody has
  looked at, not what is wrong. Reviews and close-read sessions own the other
  half.
- **A disposition is a judgment, and judgments drift.** `PURE-DEF` today can
  become `TESTABLE` when a phase gives the rule a consumer. The generator being
  re-runnable is the mitigation; the register in §8 is not a closed list.
- **It is only as good as the freeze.** Against a newer CR the right move is to
  diff rule text by `rules.text_sha` — which `spec.sqlite` already stores — and
  re-audit only what changed, rather than re-running the whole sweep.
- **Column 5 is a hypothesis.** A headline Scryfall count answers a question
  about card *faces* or *text*, not about rules the engine must implement.
  `is:dfc` was a 5.6× overstatement and `keyword:adventure` silently matches
  every card in the database. Check what a query includes before it becomes a
  scoping argument.

---

## 7. Prior art — the five gaps that motivated this

Kept because the method's credibility rests on its hit rate, and because two of
these are still unowned.

| Gap | Found | Population | Status |
|---|---|---|---|
| "Can't" effects (CR 101.2, 614.17, 613.11, +6) | 2026-08-26 | 1,857 cards / 2,034 clauses | owned — `cant-effects-architecture.md`, item 5b |
| Copy effects (CR 707, 712, 708, 729, 613.2) | 2026-08-29 | 1,628 cards, 101 atoms | owned — `copy-effects-architecture.md`, item 5c |
| **Cost modification** (CR 601.2f, 613.11, 118) | 2026-08-31 | ~903 cards; CR 107+202+118 = 53 uncovered atoms | **named, homeless.** `replacement-architecture.md` §9 says it "needs a phase marker of its own … and it is not small"; `apply_cost_modifications` is a passthrough stub with a test asserting so |
| **Casting from a non-hand zone** (CR 601, 607) | 2026-08-31 | ~764 cards | **unowned.** `check_cast_legality` hard-codes `Zone::Hand`; CR 607 linked abilities is 20 atoms, all uncovered, no doc mentions it |
| **Voting** (CR 701.38) | 2026-08-31 | 42 cards | **unowned, and never examined** — zero atoms in a 1,753-atom corpus. The real question is whether `DecisionProvider` carries the CR's *choice shapes* (simultaneous, secret, APNAP tally), not the card count |

The last three came out of one conversation, which is the argument for running
the sweep deliberately rather than when something feels off.

---

## 8. Findings register

One line per fact the audit escalates, with a pointer to where it actually
lives. **Empty until Pass A runs.** Features and dispositions do not appear
here — they go to the corpus and to `codebase-state.md`.

| # | Rule | The fact | Type it touches | Owner |
|---|---|---|---|---|
| — | — | *(none yet)* | — | — |

---

## 9. Documents this owes

- **`CLAUDE.md`'s authority table** needs a row for this file. The file is at
  199/200 lines, and its own rule is that **adding a section requires removing
  one** — so this is a deliberate trade for the owner to make, not a silent
  edit. Candidate: fold the `plans/handoffs/*.md` row into the
  `engineering-practices.md` row, which already owns process.
- **`codebase-state.md`** → "Was the critical path complete?" gets a pointer
  here, since this document is that section's method made systematic.
- **`plans/specdb.py`'s module docstring** gains the `audit` subcommand when
  A-0 lands.
