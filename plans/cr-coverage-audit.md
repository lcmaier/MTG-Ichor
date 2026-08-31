# CR Coverage Audit — is the plan complete against the frozen rules?

**Status: Pass A is complete** — A-0, A-1 and A-2 all landed 2026-08-31.
`audit --dark --families` prints an **in-scope surface of 0**: every `NNN.M`
family in the frozen CR has been triaged. **199 families, zero facts.** What it
found instead was its own generator (§8). What remains is **Pass B** — 406 dark
rules in 204 depth-gap families, 369 of them in CR 7 (§4.2, §5). This document
owns the method, the denominator and the session split. `codebase-state.md` → "Was the critical path complete?" is the parent —
this is the systematic version of the detector that section describes.

**Baseline:** `MTG-Rules/versions/tmnt.txt`, 3,120 rules, effective 2026-02-27.
Every number below is `python plans/specdb.py audit` against that tree, and is
reproducible by re-running it. **The freeze is
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
corpus atoms** — though not, as first claimed, unexamined; see §7).

**Read this section against §6's measurement before reusing its argument.**
All five were found by *reading rules*, and all five sit on rules that already
carried a verdict, an atom or a source cite — so the sweep this document
specifies would have flagged none of them. The detector's hit rate is real; it
just is not evidence for *this* method.

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
rules into §5's two triage sessions.

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

`python plans/specdb.py audit` prints exactly this. The figures are its output
on 2026-08-31, **after** A-1 fixed §8's two defects; the parenthesised numbers
are what it printed before, and are what A-0's exit criterion checked against.

| Column | Rules | |
|---|---:|---|
| CR rules in `tmnt.txt` | **3,120** | the frozen baseline |
| has a corpus atom | 1,260 | |
| has a classification verdict | **2,199** (1,035) | `TESTABLE`, `PURE-DEF`, `DEFERRED`, … |
| cited in Rust source or tests | 269 | the code already assumes something |
| cited in a plan doc | 268 | a design has considered it |
| **DARK** — none of the four | **514** (1,304) | the audit surface |

`specdb gaps` reports a narrower 153 "NEVER SEEN", which counts only rules no
session file ever *mentioned*. The 514 above is the stricter and more useful
figure: a rule can be name-dropped in a session and still never assessed.

**The pre-fix column 2 was under-reading the corpus by more than half**, and §8
has the mechanism. What that means for anyone reading an older draft of this
file: the audit surface was never 1,304 dark rules and 478 families. It was
514 and **68**, and most of what looked unexamined was examined in 2026-04 by a
session writing its verdict in a shape the parser did not know.

The scoping pass could have caught this and didn't. It recorded a
hand-measurement of 1,315 / 547 / 482 against the generator's 1,304 / 542 /
478, called the differences “the generator being right,” and moved on — but
§2.3's *other* hand-measurement, 180 of CR 702's 190 families already examined,
flatly contradicts the generator's 262 wholly-dark families for CR 7, and
nobody reconciled the two. **Where a hand count and the generator disagree by a
factor, the disagreement is the finding.**

### 2.2 Where the dark rules are

The 514 collapse into **273 dark families** (`NNN.M`), the triage unit. Of
those, **205 have an examined sibling** — a depth gap, and Pass B's — leaving
the **68 wholly dark families** that are Pass A's worklist.

```
CR 3  Card Types                 43      CR 1, 2, 4, 5, 6, 8, 9      0
CR 7  Additional Rules           25
```

**Six chapters come back 0**, and only two of those six were ever really dark:
CR 4 and CR 5 were accounted for before Pass A began, and CR 1, 2, 6, 8 and 9
were closed by A-1 — which found 124 of its 131 families already classified
and the remaining seven inside a range line the parser could not expand (§8).

§2.4's out-of-scope prune now removes **0**, because every rule it names
carries an explicit `OUT-OF-SCOPE` verdict in the corpus once the parser can
read one. **The prune is now a belt-and-braces guard, not a load-bearing
filter** — keep it, but stop quoting its 64 as part of the arithmetic.

### 2.3 Two collapses that make this tractable

**Most rows are dispositions, not work.** Of the 2,199 rules already carrying a
verdict, only **793 came back `TESTABLE`** — 36%. The rest are `PURE-DEF` (516),
`DEFERRED` (408), `OUT-OF-SCOPE` (314), `BOUNDARY-DEF` (96),
`ALREADY-IMPLEMENTED` (60), `META` (7), `LKI` (3). Applying that rate to 68
projects **~25 rows that produce anything at all**.

Three rules carry an off-vocabulary verdict — `108.5` (`PARTIALLY DEFERRED`)
and `732.1`/`732.2` (`ALREADY-HANDLED-BY-DESIGN`). They are corpus defects to
normalise into §3.2's eight, not new dispositions; `parse_rule_mentions` keeps
them only so that widening the parser could not narrow it.

**CR 7's 262 is shallower than it reads.** Measured directly: of CR 702's **190
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

**A-1 settled which of the two CR 7 numbers to believe: this hand count.** 180
of 190 examined cannot coexist with 262 wholly-dark families, and §8 D2 is why
the generator said 262. Corrected, CR 7's in-scope surface is **24 families**.

### 2.4 What is out of scope, and why it is a prune rather than a judgment

**64 dark families**, removed by rule prefix rather than by opinion. The list
lives in `specdb.py`'s `OUT_OF_SCOPE`, as data, so extending it is a one-line
diff with its reason beside it:

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
| 1 | has a corpus atom? | `spec.sqlite` → `atoms.rule_num` | 1,260 rules |
| 2 | has a classification verdict? | `spec.sqlite` → `rule_mentions.verdict` | 2,199 rules |
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
python plans/specdb.py audit --dark --families --out plans/handoffs/cr-audit-worklist.md
```

**The worklist is generated, never committed.** `--out` writes the Pass A
worklist wherever the session wants it; `plans/handoffs/` is the natural home
because a handoff is exactly what it is. It is *derived*, so it follows
`spec.sqlite`'s rule rather than the corpus's: regenerate it, do not hand-edit
it, and do not check it in. Verdicts a triage session reaches go into the
**corpus** (`plans/atomic-tests/sessions/*.md`), which is where `rule_mentions`
comes from — so the next `audit` run reflects the session's work and the
worklist shrinks on its own.

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

In-scope dark families, triaged on §3.3's single question. Output is a
disposition per family plus, for the rare `fact`, an escalation.

**Exit criterion:** every in-scope dark family carries a disposition, and every
`fact` has either an owning doc or a Deferred Migrations entry with a
back-stop. **Not** "every rule has an atom" — that is Pass B.

✅ **Met 2026-08-31.** The surface was **68**, not the 478 this section was
written against; §8's two defects account for the difference. Zero facts
escalated across A-1's 131 families and A-2's 68.

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
| ~~**A-0 — the generator**~~ ✅ **2026-08-31** | `specdb.py audit` per §3.2. No triage, no findings | +177 lines in one file, reusing the DB | landed — exit criteria below |
| ~~**A-1 — CR 8, 6, 2, 1, 9**~~ ✅ **2026-08-31** | 131 families triaged. **Zero facts.** All 131 were already classified — the generator could not read the shapes the corpus wrote them in. Fixing that (§8 D1–D2) is what A-1 produced | +59 / −11 in `specdb.py`; a note in `session-1.md` | landed — exit criteria below |
| ~~**A-2 — CR 7 and CR 3**~~ ✅ **2026-08-31** | ~~347~~ 68 families triaged — CR 3 **43**, CR 7 **25**. **Zero facts**, and the same story as A-1: all 68 were already classified, in four more shapes the parser could not read | +1 char in `specdb.py`; verdicts in `session-3.md`, `session-9b.md` | landed — **Pass A's in-scope surface is now 0** |

**Three sessions, not five to seven.** The earlier estimate was for a full
coverage classification; scoping to §3.3's question is what removes the rest.

**A-0's exit criteria, met on 2026-08-31.** `audit` reproduces §2.1's six
column totals and §2.2's family collapse; `specdb build`, `gaps`, `orphans` and
`owed` are all unchanged (`owed` still prints **38**, `orphans` clean, `gaps`
still 1,260 / 496 / 155). A-0 is Python, so `cargo test` is not its gate and no
Rust changed.

**A-1's exit criteria, met on 2026-08-31.** Every one of the 131 families
carries a disposition, and no family escalated to a fact. With §8's defects
fixed, `audit` reports **0 in-scope dark families across all five of A-1's
chapters**, which is the machine-checkable form of that claim. `orphans` is
clean, `owed` still **38**, `stats` unchanged at 1,753 / 56 / 25, and
`build`, `next` and `suspicious` all run; `gaps` moves to 1,260 / **1,320** /
153, the middle column being the fix showing up. No Rust changed.

**Regression bar for the parser widening:** every rule the old regex
classified still carries the same verdict — 0 lost, 0 changed. That is checked
directly rather than argued, because a parser that reads *more* is only safe if
it cannot read *differently*.

**A-2's exit criteria, met on 2026-08-31.** All 68 families carry a
disposition, none escalated, and `audit --dark --families` now prints an
**in-scope surface of 0**. Same gate results as A-1: `orphans` clean, `owed`
**38**, `stats` unchanged, 0 verdicts lost or changed. No Rust changed.

**Pass A is complete, and here is exactly what that does and does not claim.**
Zero *wholly dark families* — every `NNN.M` in the CR has been looked at by
somebody. It does **not** mean the CR is covered: **406 rules are still dark**,
in **204 families that each have an examined sibling**. That is a depth gap,
which §2.2 defines as Pass B's, and 369 of the 406 are in CR 7 — overwhelmingly
CR 702 keyword subrules, exactly where §2.3 predicted they would be.

| chapter | rules | dark | depth-gap families |
|---|---:|---:|---:|
| 1 · 2 · 4 · 6 · 8 · 9 | 1,322 | **8** | 8 |
| 3 | 163 | 9 | 2 |
| 5 | 148 | 24 | 4 |
| **7** | 1,487 | **369** | 194 |

**Two sessions, 199 families, zero facts.** §3.3 predicted a “yes” would be
rare; it was never once the answer. The honest reading is not that the method
found nothing — it is that the method found the *generator*, and the corpus was
in better shape than any number in this document had claimed.

**The ordering argument did not survive contact.** CR 8 led A-1 because it was
“the only chapter where the target use case and the dark-family count point at
the same place.” They did not: **all 47** of CR 8's dark families were already
classified OUT-OF-SCOPE in `session-10.md` on 2026-04-09, in a bold span the
parser could not read (§8 D1). The chapter with the most dark families was the
chapter with the tidiest prose — a count of *unread* verdicts, not of unmade
ones. A dark-family count is a proxy for attention only while the thing
counting it can see attention when it is there.

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

  > **This bullet turned out to be the whole story, and it was written before
  > the sweep ran.** Measured on 2026-08-31 after Pass A completed: of the five
  > gaps in §7 that motivated this document, and the six facts in §1.2 that
  > define what it hunts, **every one sits on rules the corpus had already
  > examined.** Darkness would have flagged none of them.
  >
  > | motivating gap | CR rules | dark before Pass A? |
  > |---|---|---|
  > | "can't" effects | 101.2, 614.17, 613.11 | no — atoms, verdicts, source cites |
  > | copy effects | 707, 708, 712, 729 | no — verdicts on all |
  > | cost modification | 601.2f, 118.7, 118.9 | no — atoms and source cites |
  > | casting from a non-hand zone | 601.2, 601.3 | no — atoms and source cites |
  > | voting | 701.38 | no — DEFERRED in session 7A, 2026-04-07 |
  >
  > Same for all six facts: 614.1/603.2 (provenance), 729.x (multi-component),
  > 122.1a/b (counters off the battlefield), 712.x (a second face).
  >
  > **So the method is calibrated for a failure mode this project does not
  > have.** Its gaps are not unexamined rules; they are rules examined once, in
  > 2026-04, against an engine that did not yet exist — classified correctly and
  > *scoped* too narrowly. Finding those needs re-reading rules that already
  > carry a verdict, which is the opposite of what `audit --dark` selects for.
  > Pass A's real yield was §8: the generator, not the rules.
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
| **Voting** (CR 701.38) | 2026-08-31 | 42 cards | **unowned.** ~~never examined~~ — that half was wrong, and wrong because of §8's D1: 701.38 has been DEFERRED in session 7A since 2026-04-07. Zero *atoms* is still true. The real question is whether `DecisionProvider` carries the CR's *choice shapes* (simultaneous, secret, APNAP tally), not the card count |

The last three came out of one conversation, which is the argument for running
the sweep deliberately rather than when something feels off.

**But note what that conversation was: reading rules, not counting dark ones.**
§6 measures the consequence — none of these five would have been caught by
`audit --dark`, because all five carry verdicts, atoms or source cites. The
table is the method's hit rate; it is not this document's hit rate.

---

## 8. Findings register

One line per fact the audit escalates, with a pointer to where it actually
lives. Features and dispositions do not appear here — they go to the corpus and
to `codebase-state.md`.

| # | Rule | The fact | Type it touches | Owner |
|---|---|---|---|---|
| — | — | *(none. Pass A triaged 199 families across A-1 and A-2; zero escalations)* | — | — |

### 8.1 The near-misses

Recorded because §3.3 says a “yes” is rare, and a register with nothing in it
is only credible if it says what it looked at hardest. Each answered **no** —
each is additive, and none makes an existing assumption false.

**From A-2 (2026-08-31), two worth carrying:**

- **CR 305.9 — a land that is also another type.** Checked against the engine
  rather than assumed: `castable_spells` skips any card whose printed types
  include Land, citing CR 305.1, so a Land Creature is never offered as a
  spell; `play_land` accepts it because it tests for the Land type rather than
  for Land alone. Both halves hold. Enforcement is at the *enumeration*
  boundary and not inside `cast_spell`, which is `CLAUDE.md`'s “performers are
  loud; callers check legality” working as designed — worth stating because the
  obvious place to look for the guard is the wrong one.
- **CR 717.6 — an explicit exception to CR 614.5.** Attractions are
  OUT-OF-SCOPE, but 717.6 says its replacement “may apply more than once to the
  same event. This is an exception to rule 614.5.” The engine builds 614.5's
  applied set and `replacement-architecture.md` §11 item 15 still owes it an
  identity key. **The CR carves exceptions out of 614.5**, so whatever key RD
  settles on needs to express “exempt from the applied set” — the same shape
  CR 903.9b already needs. Out of scope as a mechanic, in scope as a constraint.

**From A-1:**

- **CR 201.4, choose a card name.** `DecisionProvider` is four methods and every
  one is an index into a supplied list or a number in a range; “the name of a
  card in the Oracle card reference” has no list. But a fifth method is additive
  across five impls, so it costs a phase, not a rewrite. **This is §7's voting
  row again** — “whether `DecisionProvider` carries the CR's *choice shapes*” —
  and 201.4 is a second witness for a gap already named and still unowned.
- **CR 204.2, color indicator.** `CardData.color_indicator` exists and has
  **zero readers and zero writers**; `compute_characteristics` seeds Layer 5
  from `card.colors` alone. Dead forward-looking scaffolding, which is
  `codebase-state.md`'s Deferred Migrations, not a fact — and its own doc
  comment already names the phase that makes it live (CV-5, DFC back faces).
- **CR 607.4, an ability in more than one linked pair.** Constrains the
  representation before it exists: a link cannot be one `Option<AbilityId>` on
  `AbilityDef`. Worth writing down because linked abilities is §7's
  cast-from-a-non-hand-zone row, still unowned, and this is a free constraint
  for whoever takes it.
- **CR 201.2a/612.7, an object with several names.** `Characteristics.name` is
  one `String`, and CR 201.2a's “at least one name in common” wants a set. It
  answers no on population, not on principle: exactly one comparison site
  exists (`sba.rs`'s CR 704.5j legend grouping, keyed on `get_effective_name`),
  and both drivers — Spy Kit and name stickers — need Layer 3, which is unbuilt.

### 8.2 Generator defects — what A-1 actually found ✅ fixed 2026-08-31

Both were in `specdb.py`, both were one-line reads, and together they are why
§2.1's table was off by a factor.

| # | Where | Defect | Cost |
|---|---|---|---|
| **D1** | `parse_rule_mentions` | `CLASSIFY_RE` matched `**NNN.M** — VERDICT` and nothing else. The corpus writes verdicts in **three** shapes: that one (sessions 1, 4, 7a, 7b, 9a, 9b, 10), `### NNN.M — Title` + `**Classification: VERDICT.**` (2, 3, 5, 8), and `### NNN.M — VERDICT` (6). It also lost every multi-rule or range span — `**102.3–102.4** — OUT-OF-SCOPE`, `**805.1–805.10f** (all sub-rules) — OUT-OF-SCOPE` | column 2 read **1,035**; the corpus holds **2,199**. DARK 1,304 → 571 |
| **D2** | `audit`'s family collapse | the sibling test was `k.startswith(f + ".")`, which can never match a lettered subrule: `613.4a` does not start with `613.4.`, so a family whose subrules were all examined still reported wholly dark | “examined sibling” read **55**; it is **205**. In-scope families 478 → 251 |

Applied together: **DARK 514, in-scope wholly-dark families 68**, all in CR 3
and CR 7. That is what Pass A's surface always was — not 478. Re-runnable, as
§3.2 requires: `audit --dark --families`.

**Three decisions inside the D1 fix**, each of which could have gone the other
way:

- **Teach the parser, don't rewrite the corpus.** Three shapes is what twelve
  session files actually did over five months, and `CLAUDE.md` calls the corpus
  authored, never generated. Rewriting nine files to satisfy a regex is the
  wrong direction.
- **The verdict must come first on the line, not merely appear on it.** Three
  corpus lines discuss a verdict mid-sentence (“They are all TESTABLE but
  belong to Session 5”); reading those as classifications would claim rules no
  session classified. Precision over recall — over-claiming is the failure the
  table exists to prevent, so those three stay unclassified.
- **Rule numbers come from the span, never the prose.** `**107.8–107.8b** —
  DEFERRED. … behavior lives with keyword 702.87` must not classify 702.87.
  Ranges expand against the CR's own numbering, capped at the section, so
  `104.3g–k` skips the letters the CR does not use and `805.1–805.10f` stops
  before 806.

**Neither defect was A-0's alone.** D1 predates `audit` — `parse_rule_mentions`
had under-read the corpus since `specdb` was built, and `gaps`'s
“classified, no atom” column (496, now **1,320**) had been low the whole time.
A-0 inherited it and `audit` made it load-bearing.

~~**A third, unrelated bug is still open.**~~ ✅ fixed 2026-08-31.
`normalize_phase`'s `has_l_ticket` / `has_t_ticket` meant to search for
`r"\bL\d+"` and `r"\bT\d+"`, but the source held a literal backspace, `\x08`,
where the `\b` should be, so both were permanently false. Same failure mode as
D1 and D2 — a regex that silently matches nothing and reports a confident
number anyway.

**Fixing it changed no output, and the reason is the more interesting half.**
The two flags are read only inside a branch guarded by "the phase string
contains no digits at all" — and any string carrying an `L##`/`T##` contains
digits by definition, so the branch was unreachable for a second, independent
reason. Every phase string in the corpus that names a ticket also names its
phase (`Phase 5 Layers (L10)`), so nothing was ever mislabelled. Left as the
fallback it was written to be: promoting it above the digit path would relabel
atoms and move what `owed` gates on, and nothing is asking for that.

### 8.3 What A-2 found: four more shapes, and a second vocabulary

A-2's 68 families told the same story as A-1's 131 — **all 68 were already
classified**, in shapes D1's fix still did not reach:

| Shape | Where | Instances | Resolution |
|---|---|---:|---|
| `- **726.1** — DEFERRED` (a bullet before the known shape) | sessions 4, 7a, 9b | 75 rules | **parser** — `(?:[-*]\s+)?`. The same shape with a list marker earns a character, not a branch; 35 dark rules resolved, 0 disagreements |
| `## Rules 311.x — Planes` + `**Classification: …**` | session 3 | 6 | **corpus** — A-2 wrote per-family range lines |
| `### 713. Substitute Cards — OUT-OF-SCOPE` | session 9b | 2 | **corpus** — same |
| A session's final Classification Summary Table | sessions 3, 4, 7a, 9b, 10 | 178 rules, 100 of them dark | **corpus, deliberately not the parser** |

**The table row is where the line got drawn, and why matters.** A summary table
is *derived* from the body, and derived data drifts: six rules already disagree
between their body verdict and their table row (305.3, 310.4, 400.12, 402.3,
700.4, 701.12d). Teaching the parser to read tables would make `audit` depend on
a restatement that is demonstrably out of date in six places. The body is the
record; the table is a view. **D1's lesson is "read the shapes the corpus
writes", not "read everything that looks like a verdict."**

**A second vocabulary exists, and §3.2's eight do not cover it.** Four rules
carry a disposition outside the closed set: `DUPLICATE` (305.9), `PARTIALLY
DEFERRED` (108.5), and `ALREADY-HANDLED-BY-DESIGN` (732.1, 732.2). All four are
meaningful — `DUPLICATE` in particular says something the eight cannot, namely
"this rule restates another and is covered there". Normalising them is listed
in §9; adding a ninth verdict would be the alternative and is a bigger call.

**Neither defect is A-0's fault alone.** D1 predates `audit` — `parse_rule_mentions`
has under-read the corpus since `specdb` was built, and `gaps`'s
“classified, no atom” column has been low the whole time. A-0 inherited it and
`audit` made it load-bearing.

---

## 9. Documents this owes

- ~~**`CLAUDE.md`'s authority table** needs a row for this file.~~ ✅ done
  2026-08-31, and it **freed** two lines rather than costing one. The four
  per-architecture-doc rows collapsed into a single `plans/*-architecture.md`
  row carrying the phase-code mapping, which is the load-bearing part; the
  table was never an index (it omitted two files in `plans/`) and framing it as
  a precedence list is what stops it growing per subsystem. 199 → 197.
- ~~**`codebase-state.md`** → "Was the critical path complete?" gets a pointer
  here, since this document is that section's method made systematic.~~ ✅ done
  2026-08-31, and re-baselined to 514 / 68 when A-1 fixed §8's defects.
- ~~**`plans/specdb.py`'s module docstring** gains the `audit` subcommand when
  A-0 lands.~~ ✅ done 2026-08-31.
- ~~**`plans/specdb.py` owes §8's D1 and D2.**~~ ✅ done 2026-08-31, A-1.
- ~~`normalize_phase`'s backspace bug.~~ ✅ fixed 2026-08-31 (§8.2).
- **Open — the second vocabulary.** Four rules carry a disposition outside
  §3.2's eight: `DUPLICATE` (305.9), `PARTIALLY DEFERRED` (108.5),
  `ALREADY-HANDLED-BY-DESIGN` (732.1, 732.2). Either normalise them or admit a
  ninth verdict — `DUPLICATE` says something the eight cannot. Not urgent; the
  parser keeps all four visible via its legacy fallback.
- **Open — 100 rules whose only verdict is a summary-table row** (§8.3). The
  body should carry them, and six body/table disagreements should be resolved
  in the body's favour. This is corpus work, not parser work, and it is the
  natural first slice of Pass B.
- **`engineering-practices.md` §5** quotes `gaps` figures that D1 moved:
  “classified, no atom” is now **1,428** and NEVER SEEN **150**.
