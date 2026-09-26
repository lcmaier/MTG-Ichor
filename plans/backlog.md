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

### 2.1 Cost modification and the cost pipeline — ✅ graduated 2026-09-07 to `cost-architecture.md`

*Both halves are owned there: modification is CM-1 (shipped) to CM-4, payment
is CP-1, a sized slot. The entry is kept as written for the record.*

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
- **Owner** — `plans/cost-architecture.md`.

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
- **Two records an entry makes, and nothing keeps either**
  (`cr-coverage-audit.md` §4a, 2026-09-24).
  - **An "as it enters" choice:** a color, a creature type, a player or an
    anchor word (CR 614.12a, 614.12c). 208 cards,
    `o:/as [^.]*enters[^.]*, choose/`.
  - **What the entry's own zone change chose:** CR 614.14's "the exiled cards"
    (Sutured Ghoul, item 59) and CR 702.82b's "it devoured" (6 cards). Today
    it exists in `EntrySelectionScope.chosen` for one batch, and
    `AuxiliaryMove.per_chosen` turns it into counters.

  Four constraints on their shape:
  - **Made before the permanent enters (CR 614.12a),** so the choice travels
    in `EnterMods`, and the look-ahead frame reads it. A chosen creature type
    changes which other entry replacements apply (CR 614.12's "as it would
    exist").
  - **Kept on the permanent, so it leaves with it (CR 400.7).** It never goes
    into `CopiableValues`: a copy entering makes its own choice, and a
    permanent that becomes a copy later has none (CR 707.6).
  - **Keyed by the ability pair (CR 614.14, 607.4).**
  - **The auxiliary move's record holds the moved objects as they are after
    the move.**

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
  keyword family) is already filed at Phase 8, correctly. See §4's second note.
  **One test is owed here by RE-4** (its review, R16): Hallowed Moonlight's
  ruling that it "won't affect any creature that was cast, no matter which
  zone it was cast from" is tested from the hand only, and the first PR
  that opens the gate adds the board where a creature is cast from a
  graveyard or from exile under the Moonlight and enters.
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

### 2.5 CR 701 keyword actions — the unimplemented half of `Primitive`

- **Rules** — CR 701.3 (attach/unattach), 701.10–701.11 (doubling/tripling
  P/T), 701.21 (sacrifice), 701.40 (manifest), 701.43 (exert), 701.58 (cloak),
  701.62 (manifest dread)
- **Verdict** — `Primitive` is the type, and audit §4 already ruled it a
  **feature by contract**: one arm per keyword action, so nothing here makes an
  existing assumption false. Confirmed against the enum — `Destroy`, `Exile`,
  `Sacrifice`, `Mill`, `Discard`, `Scry`, `Surveil`, `Regenerate`,
  `CreateToken` exist; **`Manifest`, `Cloak`, `ManifestDread`, `Exert` and
  `Unattach` do not**, and neither does a P/T-doubling arm. `Attach` exists
  without its inverse.
- **Size** — per arm, batched a few to a PR. **One exception that is not
  additive:** the doubling/tripling family, which `codebase-state.md` Deferred
  Migrations item 5 already owns — CR 701.10a makes doubling a Layer 7c effect
  whose addend depends on what already applied, needing an `AmountExpr`
  affected-power leaf *and* a timestamp merge. 6 of the 24 atoms are that, and
  they are the only ones with a design question.
- **Blocks** — manifest and cloak need the face-down subsystem, shared with
  foretell in §2.3; exert needs skip-untap tracking; unattach is the missing
  half of an `Attach` that already works.
- **Atoms** — 14 re-filed to `Backlog` by §3.3's policy, this entry having
  captured their tickets; the remainder stay on their shipped phases —
  `orphaned --bucket unbuilt` lists them under CR 701 (11 at the 2026-08-31
  re-count). Nothing of this section remains in `owed`.
- **~~`Discard` and `Scry` are RE-8's (2026-09-11)~~ — ✅ built 2026-09-14.**
  `Primitive::Discard(n, DiscardChooser)` with CR 701.9b's default and random
  choosers, `Primitive::Scry` with `GameAction::Scry`, `GameEvent::Scried` and
  CR 701.22a's three choices; the cause predicate landed as
  `ReplacementDef::by: Option<SourceFilter>` rather than a `caused_by` on the
  zone-change pattern, with Nephalia Academy as its printed customer. Mind Rot,
  Hymn to Tourach, Opt and Eligeth, Crossroads Augur beside it; Mind Rot and
  Opt pooled. **Two halves stayed behind, each with its facility named**: the
  to-battlefield leg and the five cards that print it (Dodecapod, Wilt-Leaf
  Liege, Loxodon Smiter, Nullhide Ferox, Obstinate Baloth), whose clause is on
  a card in *hand* and so wants CR 113.6 — critical-path item 6a,
  `replacement-architecture.md` §11 item 87 — and 701.9b's third chooser,
  below. *Original entry:* — on RD-1's precedent
  (`Primitive::Mill` landed inside a replacement PR because a rider needed it).
  The replacement arm for discards has existed since RB (`EventPattern::
  ZoneChange { cause: Some(Discarded) }`); RE-8 builds the producer with
  CR 701.9b's default and random choosers, a `caused_by` on the zone-change
  pattern (sixteen of the seventeen "causes you to discard" cards say "an
  opponent controls"), the to-battlefield leg on `Instead(ZoneChangeTo)`
  (Dodecapod, Wilt-Leaf Liege), and `GameAction::Scry` with Opt and Eligeth,
  Crossroads Augur. The sizing's first cut sent both here on §8a's sentence;
  the review overturned it on cost of delay — item 6's 1,045 discard-watchers
  would otherwise test against fixtures. "Another player chooses" (701.9b's
  third shape) stays here with its first card.
- **`Shuffle` (CR 701.24a) — ✅ built 2026-09-16 (RF)**, on RD-1's precedent
  again: `Primitive::ShuffleLibrary` and `GameAction::ShuffleLibrary` landed
  inside a replacement PR because Darksteel Colossus's rider needed them
  (`replacement-architecture.md` §9, Phase RF, decision 6). Whose library is
  the recipient's: `Controller` is "shuffle your library", `ThisObject` the
  source's owner's, a target a player or an object standing for its owner.
  **`ShuffleIntoLibrary` stays here** — a spell's "shuffle target card into
  your library" must *move* the card first (CR 701.24c), which the rider must
  not — and is now one `change_zone` plus one `ShuffleLibrary` proposal per
  target; Lich's Mirror's three-zone recipient is the other half it waits on.
- **Owner** — none yet.

### 2.6 CR 702 keyword abilities

- **Rules** — CR 702, the evasion/combat and static-ability half: menace
  (702.111), shroud (702.18), protection (702.16), improvise (702.126), flash
  (702.8), impending (702.176), warp (702.185), first/double strike
  (702.4, 702.7), and **infect (702.90), wither (702.80) and toxic (702.164)
  with their CR 120.3b/d/g results of damage** — named here 2026-09-08 because
  the ledger's `T21c` pointed at this entry and this entry did not mention them;
  and **suspend (702.62)**, named 2026-09-14, for the store it needs below
- **Verdict** — mostly card breadth rather than a missing surface, which is why
  audit §6 retired `audit --dark` over exactly this material: it is *depth*, and
  it belongs beside the phases that need it. Two exceptions worth naming
  separately, because they are engine behavior and not a card:
  **mid-combat keyword change** (702.4c/d, 702.7c — gaining or losing
  first/double strike between damage steps re-decides who participates) and
  **LKI for a damage source that changed zones** (702.2e, 702.15c, 702.90d),
  which is item 6's LKI formalization.
- **Size** — the keywords are Phase 8 breadth. The mid-combat re-check is one
  focused change to the combat damage step; the LKI half rides item 6.
  **Infect/wither/toxic: ~200–300 with the first infect card** — three
  `KeywordFlag`s, three result arms in `perform_action(DealDamage)`, and a
  counter proposal whose subject is a *player* (§2.16's map; the same
  player-subject shape RD-1 gives `ReplacementDef`). **The seam is cut, as of
  2026-09-08**: `engine::actions::DamageResults` is the struct RD-1 built for
  CR 120.3's "one or more of the following results", so each keyword adds one
  flag there and one block in `perform_action`'s `DealDamage` arm — with the
  difference that these three read the *source's* keywords where 120.3c/e read
  the target's types. `codebase-state.md` item 86 carries the dated line.
- **Counters on an object off the battlefield** (RE-5's review, 2026-09-14,
  R3) — the store, not the subject. `CounterSubject::Object(ObjectId)` names
  any object and CR 122.1a/b already speak of "a creature card in a zone
  other than the battlefield", so the event vocabulary is done; what is not
  is that counters live on `PermanentState`, `perform_action`'s counter arms
  refuse an object off the battlefield, and `move_object` drops the entity,
  which is CR 122.2 by construction. Suspend's time counters on an exiled
  card, Darigaaz Reincarnated's egg counters, and Skullbriar, the Walking
  Grave's "counters remain on Skullbriar as it moves to any zone other than
  a player's hand or library" all want `counters` on the `GameObject`, with
  `PermanentState` keeping only CR 613.7c's timestamps — and Skullbriar's
  ruling that the retained counters "aren't 'placed'" means no `AddCounters`
  is proposed for them and Doubling Season does not see them: a zone-change
  performer question, not a pipeline one. ~60 lines with the first
  exiled-with-counters card; nothing owed until then.
- **Blocks** — nothing structural. Protection also needs §2.8's SBA legality
  re-check for Auras and Equipment.
- **Atoms** — 20, not re-filed.
- **Owner** — none yet.

### 2.24 "As though" effects (CR 609.4) — a rule fiction, scoped to one question

**The surface that cannot express it.** Nothing in the engine can say "answer
*this one* legality or cost question as if the board were different, and
answer every other question normally". Every continuous effect the engine has
changes what an object *is* (CR 613's layers) or forbids an action outright
(CR 101.2's restrictions, `cant-effects-architecture.md`). An "as though"
effect does neither: the object is unchanged for all purposes but one.

**The CR names the category and scopes it**, which is the thing that makes
this tractable rather than frightening:

> **609.4.** Some effects state that a player may do something "as though" some
> condition were true or a creature can do something "as though" some condition
> were true. **This applies only to the stated effect. For purposes of that
> effect, treat the game exactly as if the stated condition were true. For all
> other purposes, treat the game normally.**

So Masako the Humorless — "tapped creatures you control may block as though
they were untapped" — does **not** make a creature untapped. It is still
tapped for `Cost::Tap`, for "untap all creatures", for a "whenever a tapped
creature" trigger, and for its own controller's next attack. Exactly one
question, block legality, is answered against the fiction. That is the
invariant to write down before anything is built, because the tempting
implementation — a Layer-shaped "counts as untapped" row — gets every one of
those other questions wrong at once.

Two more rules matter and both make life easier:

- **609.4a — they compose, and one can satisfy another's premise.** The CR's
  own example is Vedalken Orrery ("cast spells as though they had flash") plus
  Shaman's Trance ("play lands and cast spells from other players' graveyards
  as though those cards were in your graveyard"): a sorcery with flashback in
  an opponent's graveyard becomes castable. So the consult is a *union* over
  applicable fictions, re-asked until it stops changing — the same shape
  `is_prohibited` already unions restrictions in, not a precedence ladder.
- **609.4b — spending mana "as though it were mana of any type" affects only
  how a cost is paid.** "It doesn't change that cost, and it doesn't change
  what mana was actually spent" — so this one is `cost-architecture.md`'s
  payment side (CP-1), not cost *determination*, and it must not touch the
  locked total. CR 118.14's "mana of any type can be spent" is the same rule.

**The population, counted 2026-09-09.** Scryfall `o:"as though" -is:funny`:
**287** cards. Not an edge case, and not one mechanic either — the shapes
split by *which question* they re-answer:

| Shape | Question re-answered | Examples | Rough count |
|---|---|---|---|
| Timing | may this be cast now (CR 601.3b–d, which the CR spells out for flash specifically) | Vedalken Orrery, Leyline of Anticipation | part of the **117** that pair "as though" with "you may cast"/"may play" |
| Zone | may this be played from here | Shaman's Trance, Yawgmoth's Will, Wildfire Devils | the rest of that 117 |
| Payment | may this mana pay that pip (609.4b) | Chromatic Lantern's cousins, Cascading Cataracts | **62** on `o:"spend mana as though"` |
| Attack / block legality | may this attack or block | Masako the Humorless, **49** on `o:"as though it didn't have defender"` | ~50+ |
| A value inside a check | what number does this rule read | "crew as though its power were 2 greater", "adapt as though it had no +1/+1 counters" (Biomancer's Familiar), Elvish Refueler's exhaust | **41** on `o:"as though" o:"power"`, plus the counter-fiction ones |

**Why the first four are one mechanism and the fifth is the interesting one.**
Waiving a check ("as though it didn't have defender", "as though it were
untapped", "as though it had flash") is a *permission* consulted where a
legality predicate is asked — the exact mirror of `Restriction`, which is
already built, swept off effective ability lists, and asked through
`is_prohibited(Query)` with one arm per decision site. A `Permission` def and
an `is_permitted(Query)` beside it is a known shape done twice already
(restrictions, cost modification), and CR 609.4a's union is `is_prohibited`'s
union with the sign flipped.

The fifth shape substitutes a **value** rather than waiving a check, and it is
where the design question actually lives. The engine has the machinery: RC-4's
`EntryFrame` already answers "what would this object's characteristics be
under a hypothesis" for one bounded question, with `frame_of` returning the
hypothetical for the object under test and the real board for everything else
— which is CR 609.4's "for that effect / for all other purposes" split,
built. So the likely shape is a **question-scoped frame**, not a new layer:
the decision site names the question, the sweep supplies the fictions that
apply to it, and the check runs against the overlay. What must be written
down first, in the doc: **the fiction never reaches `compute_characteristics`
and never reaches the layer walk** — the day it does, Masako's creature is
untapped for the untap step and the whole rule inverts.

**What it blocks.** Nothing on the critical path, and it is not a Phase 8
breadth item either: it is a mechanism, and 287 cards is enough that Phase 8
will hit it immediately. The three sub-shapes are separable and only the
first is cheap.

**Rough size.** The permission half (timing, zone, attack/block legality):
~600–900, a `PermissionDef` mirroring `RestrictionDef`, `is_permitted` with
one `Query` arm per site, the sweep, and 3–4 cards. The payment half
(609.4b): ~200–300 on top of CP-1's payment, and it belongs with it. The
value-substitution half: **unknown until scoped** — it needs the frame
generalized past entries, which is RC-4's machinery pointed at a new
question, and the sizing has to count the decision sites first. Graduates to
`as-though-architecture.md` when scheduled; it is a sibling of
`cant-effects-architecture.md` and should be written against it, because the
two share the sweep, the union and the decision-site vocabulary and differ
only in sign.

**Prior art in this file:** §2.8 (where an ability functions) is the closest
neighbour and overlaps on the zone shape.

**What is scheduled, and none of it is here — because this file is an
inventory and §0 means it.** Two things now point outward, and the second was
added 2026-09-09 after the observation that an entry with an owner and a size
and no slot is not scheduled, it is merely recorded.

1. **The back-stop lives on the route**: `roadmap-v2.md` §3a row **B8**, which
   puts the permission half after B5 and before Phase 8, gates Phase 8's start
   on it beside CV-7 and B4, rides the payment half with CP-1, and says why the
   value-substitution half deliberately has no date. A back-stop is what turns
   "recorded" into "scheduled", and it could not go in this file: §0 allows six
   fields and no seventh, and "nothing here has a date" is the sentence that
   makes the inventory trustworthy.
2. **The seam is a constraint on two other phases.**
   `cant-effects-architecture.md` §3.7 carries a constraint on **RS-3a and
   RS-4**, written 2026-09-09 and deliberately ahead of this entry's own
   scheduling. Those two phases rewrite the combat validators and the payment
   sites, which are the sites an "as though" consult needs, and they are the
   only phases between now and this entry that touch them. §3.7 asks two things
   of that rewrite — that a consult can say *which* restriction forbade, and
   that a base-rule clause reads through an accessor rather than a field — so
   that this entry, whenever it is scheduled, is a normal diff rather than a
   second rewrite of the same four functions.

Nothing else here is owed early.

### 2.26 "That card", after it was drawn (CR 121.6c)

**The surface that cannot express it.** An `Effect::Sequence` is CR 608.2c's
instruction sequencing and threads nothing between its atoms;
`Primitive::DrawCards` returns no object ids to a later one. So *"draw a card
and reveal it. If it isn't a land card, discard it"* cannot be written at all,
and CR 121.6c — the rule that says the additional action is **not** performed on
a card that arrived by replacement instead — has nothing to govern.

**Why it is here rather than in `codebase-state.md`'s Deferred Migrations
(2026-09-12, at RE-2's review).** That section is debt owed by scaffolding, and
RE-2 scaffolded nothing for this: with no producer of an additional action,
there is no site at which the rule could be got wrong. It is a mechanic the
types cannot say, which is this file's subject.

| Field | |
|---|---|
| **Rules** | CR 121.6c, and CR 121.7's ordering beside it |
| **Verdict** | `Effect::Sequence` carries no result between atoms; `Primitive` returns nothing to the tree |
| **Size** | one field on `ResolutionContext`, written by the draw performer through the resolution that proposed it, plus a `SelectionFilter` leaf that reads it — the shape `replaced_amount` and `damage_prevented` already have. ~80 lines, and the same field is what CR 701's "the cards milled this way" and "the token created this way" will want |
| **Blocks** | the "draw and reveal" four — Fa'adiyah Seer, Sindbad, Pact Weapon, Breathstealer's Crypt (Scryfall, 2026-09-12) — each of which *also* needs `Primitive::Discard` (RE-8) or the information model (§2.9), so none is unblocked by this alone |
| **Atoms** | `ATOM-614.11b-001`, uncovered with this reason in `tests/phase_re2_integration_test.rs`'s module doc |
| **Owner** | — |

**Note what the four cards have in common**: every one of them is "draw and
reveal, then act on what it was", and none is "draw, then act on it blind". The
information model (§2.9) is therefore a co-requisite rather than a neighbour,
which is the fact that makes building the field alone produce zero playable
cards — §8c's "two customers before a leaf", failed on the first count.

### 2.25 Partial damage redirection (CR 614.9) — one event becoming two

**The surface that cannot express it.** A `Rewrite` returns *one* proposal
(`Option<GameAction>`), and §3.2d of `replacement-architecture.md` removed the
`Split` variant that could return more. Harm's Way — "The next 2 damage that a
source of your choice would deal to you and/or permanents you control this turn
is dealt to any target instead" — redirects *part* of one event: 2 of a
3-damage Lightning Bolt goes to the chosen target and 1 stays on you. That is
one `DealDamage` becoming two with different targets, and neither the rewrite's
return type nor the pipeline's member list can hold it.

**Everything else it needs already exists.** RD-4 built `Rewrite::Retarget` and
CR 614.9's destination re-check; RD-2 built `Uses::NextDamage` spent by the
amount moved, which is Harm's Way's own ruling (*"if the chosen source would
deal just 1 damage … Harm's Way's effect will redirect that damage and still
have a 'shield' left for another 1 damage from that source later in the turn"*);
RD-3 built CR 609.7a's chosen source. The card is one mechanism short.

| Field | |
|---|---|
| **Rules** | CR 614.9 (partial redirection), 615.7 (the count allocated across members) |
| **Verdict** | `Rewrite` returns one proposal; `apply_replacements` returns one entry per *batch index*, and a split-off event has none |
| **Size** | ≈ 30 mechanical sites plus new logic in `next_damage_shares` — above `replacement-architecture.md` §9's ~300–400 estimate, which did not contain the allocation half |
| **Blocks** | Harm's Way; and, since RE-6, the choice Exquisite Archangel's first ruling offers — see the second customer below. Divine Deflection is a *pooled* amount, not a split, and waits on `AmountExpr::X` and `codebase-state.md` item 90 |
| **Atoms** | none filed under `Backlog`; CR 614.9's atoms are covered by RD-4's whole-event redirects |
| **Owner** | `replacement-architecture.md` §9's RD-5 section, which carries the shape, the site table and the gate that closed |

**Why it is here rather than on the route** — and it is a *number*, which is
the only reason §9 allows for excluding one card. RD-5's gate said the split
ships if the member insertion lands "without touching the code every combat step
runs … no change to `apply_replacements`' signature". Read against the tree at
RD-4's close it does touch both: `apply_replacements` returns
`Vec<(usize, Option<GameAction>)>` keyed by batch position, and
`execute_batch_inner`'s `decided[i] = action` is phase 2's write. The full table
is in §9.

**Two things a later PR inherits that the gate did not know about.** RD-4 made
the CR 616.1 chooser, the prompt and `Instead(RemoveCountersFromAffected)` read
the *event* rather than the group key (`replacement-architecture.md` §11 item
35), which is the split's awkward case already solved. And the genuinely new
work is `next_damage_shares`: CR 615.7's count and the split are **one** choice,
because the ruling lets the 2 go "1 damage … to each of two different
recipients".

**Re-checked at RE's sizing (2026-09-11): RE does not reopen this.** The one
place RE turns one event into several — CR 121.2's draw decomposition — is a
*performer's* loop over `execute_action`, one inner draw per batch, never a
second member inserted into the batch being decided; `apply_replacements`'
`Vec<(usize, Option<GameAction>)>` and `execute_batch_inner`'s `decided[i]`
are touched by none of RE's nine PRs. The site table above stands.

**A second customer, found at RE-6 (2026-09-12), and it is not a split.**
Exquisite Archangel's "if you would lose the game, instead exile this creature
and your life total becomes equal to your starting life total" replaces one
event with *two*, about two subjects. A rewrite yields one event, so RE-6
encoded it as `Prevent` plus a rider — and a rider resolves after the batch
performs (CR 615.5, `replacement-architecture.md` §4.1a). The card's first
ruling is the board where that timing is observable: dealt lethal damage in
the same check that would lose you the game, *"its effect applies ... You
choose whether Exquisite Archangel is moved to exile or to your graveyard"*.
The exile has to be a member of the batch beside the death for that choice to
exist, which is this entry's facility — a rewrite yielding members inserted in
phase 1 — plus one thing Harm's Way does not need: two members moving one
object to two zones turn the CR 704.7 same-subject *collapse* into a *prompt*
(~40 lines and a `ChoiceKind`). Lich's Mirror's fifteenth ruling is the same
board with a shuffle. The engine takes the graveyard outcome today and never
asks (`codebase-state.md` item 125; `replacement-architecture.md` §11 item
63). **Two customers is the bar, and it is met — what is open is the slot,
not the case.** §8c's "two customers before a leaf" was never what closed
RD-5; its gate closed on *shape* — the ≈30 sites above, two of them on the
path every combat step runs — and that table is unchanged. What the second
customer changes is the cost of delay: the first was one unregistered card,
and this one is a registered card answering a printed ruling without the
choice the ruling names, reachable in `stress`. Nothing in the project's
rules asks for a third. Recommended (2026-09-12, RE-6's review): its own PR
under the RD-5 heading, sized there at ~300–400 plus the collapse-to-prompt,
**after RE-7** — the two commute, RE-7 is the cheaper half of the four-player
work RE-6 opened, and a change to `apply_replacements`' return shape should
not ride inside a rules phase. Ordering authority stays with `CLAUDE.md`'s
critical path, which lists neither; that is the owner's line to add.

### 2.23 Battles (CR 310), and CR 120.3h

- **Rules** — CR 310 whole: casting a battle, its protector, defense counters
  (122.1g), being attacked (506.1, 508.1), CR 120.3h's damage result, and
  the siege's flip on defeat (310.11–310.12)
- **Verdict** — `CardType::Battle` and `AttackTarget::Battle` exist and
  nothing reads them: `validation.rs` refuses a battle as an attack target,
  `perform_action(DealDamage)` marks damage on any object, and no protector
  is chosen anywhere. A whole subtype's rule set, not a missing leaf.
- **The protector is a second control-like relation, and `ObjectFilter` has a
  leaf for only one of them** (noted 2026-09-09, RD-3 review). CR 310.8a has
  the *controller* choose a protector and 310.8b lets a Siege be attacked by
  its own controller, so "a permanent an opponent controls" and "a battle an
  opponent defends" are different questions about the same permanent. Nothing
  registered reads the second — Torbran, Thane of Red Fell says "controls" and
  is therefore already right about a Siege you control — so this is a leaf this
  entry owes, not one `SourcePattern` or `ObjectSet` is missing today.
- **Size** — one PR in the band, after item 6 (the flip is a trigger) and
  the planeswalker attack path it shares (`replacement-architecture.md` §9,
  RD-1 leaves both out of combat on purpose).
- **Blocks** — every battle card (March of the Machine's 36 and later
  printings); CR 120.3h's result arm, which is one flag on RD-1's
  `engine::actions::DamageResults` and one block beside CR 120.3c's, and is
  blocked on the card type rather than on the arm (`codebase-state.md`
  item 87).
- **Atoms** — none filed under `Backlog` yet; CR 310's are in Phase 8.
- **Owner** — none yet. Entered 2026-09-08 by RD's design check, because no
  document owned CR 310 and the results-of-damage decomposition needed to
  point somewhere real rather than at a `T##`.

### 2.7 Modal spells and abilities (CR 700.2), and devotion (700.5)

- **Verdict** — **`StackEntry.chosen_modes` is dead scaffolding.** Declared at
  `state/game_state.rs:33` as `Vec<usize>`, written `Vec::new()` at all twelve
  construction sites, and **read nowhere**. That is the same shape as
  `CardData.color_indicator` in audit §5.2 — a field that represents the fact
  correctly with no writer and no reader — so it is Deferred Migrations debt,
  not a fact. Nothing chooses a mode, so nothing can be modal.
- **Size** — one phase, and it wants doing near §2.1: CR 700.2h's per-mode
  additional costs and 700.2c's mode-conditional targeting both reach into the
  cost pipeline and into CR 601.2b/601.2c, which §4 leaves for later triage.
  700.2e hands the choice to an opponent, so it also needs a `DecisionProvider`
  method — §2.4's question again, in a different costume.
- **Blocks** — every charm and command; escalate and entwine; the
  "choose one or both" cycle. Devotion (700.5a) is separate and layer-shaped:
  it reads a *partial* layer result, after L1–L3 but before L4–L7 — and it is
  itself modifiable, so the count needs a hook rather than being a pure query
  (Altar of the Pantheon, "your devotion to each color is increased by one").
- **Atoms** — 7, not re-filed.
- **Owner** — none yet.

### 2.8 Where an ability functions, and when it can be activated

- **Rules** — CR 602.5 (activation restrictions), 604.5/604.6 (static abilities
  that function on the stack or in hand), 113.6 (functioning zones), 608.3g
- **Verdict** — audit §4 already ruled both halves on `AbilityDef`:
  **activation restrictions (CR 602.5d) and functioning zone (CR 113.6) are
  additive** — the trigger-condition field in the same row is critical-path
  item 6, these two are not. Today an `AbilityDef` says what an ability does
  and not where it works or when it may be used, so "activate only as a
  sorcery" and "you may cast this from your hand for its flash cost" have no
  representation.
- **Size** — one small phase for both fields, but it is a **prerequisite that
  looks optional**: §2.3's cast-from-elsewhere keywords are written as
  hand-zone or graveyard-zone static abilities (604.6), so this is the surface
  that admits them. 608.3g's stack-zone static → ETB delayed trigger (Dash,
  Blitz, Warp) also needs item 6.
- **Landed in part (2026-09-05, LH-2)** — `AbilityDef::activation_restriction:
  ActivationRestriction`, with the one value Equip needed, `OnlyAsSorcery`
  (CR 602.5d), honoured at the three ability-index sites CLAUDE.md names.
  The surface this section owns *grows that enum* — once-per-turn, "only
  during combat", the functioning zone — rather than starting a second one.
  **Two look-alikes are not this section's** (review, 2026-09-06): Teferi,
  Time Raveler's "Each opponent can cast spells only any time they could cast
  a sorcery" is a continuous restriction on *players*, the "can't" track
  (`cant-effects-architecture.md`, a `RestrictionDef` the cast-timing check
  consults), and Grave Servitude's "If you cast it any time a sorcery
  couldn't have been cast" is a *condition* about how the spell was cast,
  read later by a delayed trigger — item 7's `Condition` AST plus item 6,
  with the cast-time fact recorded on the object. All three ask one question,
  and `GameState::check_sorcery_timing` is the one place that answers it.
- **Blocks** — flash; the once-per-turn restriction that must survive a
  controller change (602.5b); §2.3, in the sense that it is where those
  keywords will be expressed. No longer "activate only as a sorcery".
- **Atoms** — 14 across CR 602 (9) and CR 604 (5), not re-filed.
- **Owner** — none yet.

### 2.9 The information model — who can see what

- **Rules** — CR 400.2 (public vs hidden zones), 401.2/401.3 (library), 402.3
  (hand), 404.2 (graveyard)
- **Verdict** — **nothing in the tree answers "can player N see this object?"**
  A grep of `mtgsim/src` for a visibility predicate returns exactly one piece of
  hidden-information state: `PermanentState.face_down: bool`.
  `Zone::is_public()` exists as the coarse, viewer-independent classification —
  **and nothing calls it** (the fifth no-consumer find, 2026-08-31) — while no
  query takes a viewing player at all. The per-viewer query is the gap, and it
  should consume the classifier rather than duplicate it.
- **Size** — a real phase, and the one entry in this file that is arguably a
  **v1 blocker rather than card breadth**. `CLAUDE.md` names v1 as 4-player
  Commander through a GUI and highly parallel AI games over the CLI: the first
  must render only what one player may see, and the second must not leak a
  hidden zone into an agent's observation. The current CLI is omniscient, which
  is why nothing has needed this yet.
- **Blocks** — any non-omniscient UI; face-down permanents beyond the single
  flag (§2.5's manifest and cloak, §2.3's foretell); "look at the top card of
  your library"; revealing, and every effect whose text distinguishes *reveal*
  from *look at*.
- **Where the corpus already sited it** — two retired `NEW` tickets named the
  shape before this entry existed: 400.2 wanted a *zone visibility
  classification query*, and 402.3 put *hand visibility enforcement in the
  oracle layer*. Both point the same way — a per-viewer query beside
  `oracle/characteristics.rs`, not a flag on the zone.
- **Atoms** — 4 here, plus **CR 400.2, which slice 1 filed under §3.2 in
  error**: "public zones are zones in which all players can see the cards" is
  this entry, not a zone guard. Corrected in §3.2's table.
- **Asked at RE-8's close (2026-09-14): should this move up beside CR 113.6?
  No — but it becomes a named *seam* in critical-path item 6's doc, which is a different
  thing.** CR 113.6 moved because it **gates** the next item on the spine and
  because RE-8 had five printed cards physically unbuildable without it. This
  entry gates no spine item before Phase 8, and everything RE-8 touched is a
  no-op rather than a wrong answer: a "reveal" and a "look at" change nothing
  observable in an engine whose every decision provider already sees the whole
  board, so Nephalia Academy's clause and Opt's reminder text cost nothing
  being absent. `CLAUDE.md`'s interleave already puts it before Phase 8's
  reveal cards and before Phase 10, which is where the GUI and the parallel AI
  harness — the two halves of v1 — actually need it.

  **What RE-8 did change is that critical-path item 6 must not answer this question by
  accident.** CR 603.10a is written about *visibility*, not zones —
  "abilities that trigger when an object that **all players can see** is put
  into a hand or library" — and
  `atomic-tests/supplemental-docs/603-2f-complexity.md`'s board is the proof:
  Future Sight and Telepathy flip the answer without moving a card. So the
  trigger phase's doc names the seam and consumes `Zone::is_public()` — the
  coarse, viewer-independent classifier that still has **zero callers**, the
  fifth no-consumer find above — rather than folding "hidden zone" into
  CR 113.6's zone predicate, where this entry would later arrive to find its
  answer already written in the wrong place.

  **RE-8's own customers, for whoever takes this:** Nephalia Academy's "you
  may reveal that card" and Opt's "look at the top card", both no-ops today;
  CR 701.9c's undefined characteristics for a discard put into a hidden zone
  *without being revealed* (`codebase-state.md`, "Found by RE-8", item 130);
  Kenessos, Priest of Thassa's second ability, which is why that card is a
  fixture here and not a registration; and §2.24's "draw and reveal" four.
- **The trigger survey's customer (2026-09-18, `plans/references/trigger-survey.md`, table one's 603.2f row):** the dispatcher's visibility gate. CR 603.2f is "visible to all players" at the instant after the event — a *global* bit, a subset of this entry's per-viewer query — and until a reveal exists no hidden-zone object is visible, so `Zone::is_public()` is exact and the triggers doc names the predicate per object for this entry to fill. The survey read it as no reason to move this entry up, the RE-8 answer above standing; the owner decides whether the GUI's nearness does.
- **Slot (2026-09-25)** — **the design, merged with §2.34, sits right after item
  6's close audit**, ahead of Phase 8, and the owner reviews it before any build
  (`roadmap-v2.md` A6f). The AI floors' survey
  (`plans/references/ai-performance-floors.md`) found the query alone
  insufficient for honest search (§2.34), and the knowledge record it adds is
  forked state, which floor 3 bounds by the board. The build keeps its
  back-stop, before Phase 8's reveal cards and Phase 10.
- **Its cost section (the owner, 2026-09-25)** — before review, the design
  sets a ceiling on each cost against a floor (`engineering-practices.md` §3.1),
  and its build PRs are held to those ceilings:
  - the observation cost k per decision: floor 1 scales by 1 + k, so at
    today's 15,600 a k above ~0.56 fails it before triggers add anything;
  - the knowledge record's bytes and allocations per clone (floors 2 and 3):
    bits on each object stay under 1 KB with no allocation, and a per-viewer
    event history breaks both floors;
  - the redeal's µs per fork, and whether the layer memo stays warm through
    it. The memo keys on one global epoch (`state/layer_memo.rs`), so a redeal
    that writes a walk input starts every fork cold;
  - the visibility query's instructions per decision.
  The bounded-state PR's probe measured k and a naive redeal first
  (`roadmap-v2.md` A6a; `fuzz-record.md`, its block), so the ceilings start
  from numbers:
  - **k = 0.106** on floor 1's board: a naive observation of every public
    object, the decider's hand and the hidden-zone counts, written into a
    reused buffer, is 7.7 µs against 66.3 µs of engine per decision, and
    0.139 on three `stress` games. Floor 1's 15,600 becomes about 14,100.
  - **The redeal costs 1.7–6.1 µs per fork**, about one clone again, so a
    determinized fork is about twice a plain one and passes floor 2's 10 µs
    early in a game, when the libraries are full.
  - **The ceiling assumes a cold memo.** The memo stays warm only while no
    registry row reaches a hidden zone the redeal moves cards between, which
    is a one-compare check (`RegistryScopeSummary::reachable_zones`). No pooled
    card has such a row, but Mycosynth Lattice, Painter's Servant and Arcane
    Adaptation do, and they are played. On our boards a cold first decision
    spent one board walk more, 10–40 µs from mid-game on. On a board with such
    a card that walk covers every card in the reached zones, about 400
    objects. It costs 76–224 µs over a warm first decision at every stage of
    the game, against 3–39 µs without the row (`codebase-state.md` item 181,
    measured 2026-09-25). Item 181's lever, walking a card in a library or a
    hand only when something reads it, landed as LL (2026-09-25). Most of
    what a cold first decision still costs is not the row's: every one
    re-walks the battlefield, 2.9–35.3 µs by turn without the row. The
    row's own part fell from about 70–190 µs to 2–32, by turn from turn 10:
    5.5–63.8 µs over a warm decision on those boards, the top a two-game
    median at turn 50. Wall-clock on the native Windows machine, medians of
    seven per fork; LL's gain proper is floor 1, every decision rather
    than a fork's first (`fuzz-record.md`, LL).
  - **These are the naive model's numbers.** With no knowledge record, every
    hidden card is unknown, so the redeal shuffles all of them, and the
    observation makes no visibility query. The build pays a lookup per card
    for the first and the per-viewer query for the second.
- **Owner** — none yet.

### 2.10 color is a derived characteristic, and the engine stores it

- **Rules** — CR 202.2 (color from the mana cost), 105.2, 105.3
- **Verdict** — `engine/layers/compute.rs:99` seeds Layer 5 with
  `colors: card.colors.clone()` — **the stored field, never the mana cost.**
  CR 202.2 makes color *derived* from the mana symbols, so an authored card
  whose `colors` and `mana_cost` disagree is silently wrong and nothing can
  notice. `color_indicator` still has no reader (audit §5.2, Deferred
  Migrations), and `is_monocolored` / `is_multicolored` / `is_colorless` do not
  exist at all.
- **Size** — small-to-medium, and a **feature by the audit's test**, not a fact:
  `EffectiveCharacteristics.colors` is already the right type and Layer 5
  already applies to it, so the change is the seed plus a derivation function.
  It pairs naturally with §2.1 — reading a hybrid or Phyrexian symbol for its
  colors is the same symbol-decoding work as paying with one.
- **Blocks** — devotion (§2.7's CR 700.5a); protection from a color (§2.6);
  every color-matters card; the color indicator's CV-5 landing.
- **Atoms** — 20, across CR 202 (11) and CR 105 (9).
- **Owner** — none yet.

### 2.11 Loyalty abilities

- **Rules** — CR 306.5 (loyalty as a characteristic), 306.5d (activation),
  306.8 (damage), 209.2
- **Verdict** — **the counter half is built and the ability half is not.**
  `CounterType::Loyalty` exists, CR 704.5i's zero-loyalty SBA is implemented and
  tested (`engine/sba.rs:277`, with `ZoneChangeCause::ZeroLoyalty`), and combat
  already targets planeswalkers (`AttackTarget::Planeswalker`). What is absent
  is any notion of a *loyalty ability*: nothing in the tree names one, so
  neither 306.5d's sorcery-speed restriction nor its one-per-permanent-per-turn
  limit can be expressed. 306.5a/c — loyalty as a characteristic, printed off
  the battlefield and counter-derived on it — has no query either.
- **Size** — small, but it is **downstream of §2.8**: "activate only as a
  sorcery" is exactly the activation restriction that entry adds to
  `AbilityDef`. The per-turn limit needs per-permanent turn-scoped state.
- **Blocks** — every planeswalker card, which is a whole card type. Among
  them Grist, the Hunger Tide, whose first ability LL tests through a clause
  fixture (`phase_ll_cards::grist_insect_clause`); the card waits on these
  abilities, and its −2 on TR-3's reflexive trigger.
- **Atoms** — 12, across CR 306 (10) and CR 209 (2).
- **Owner** — none yet.

### 2.12 Step- and phase-scoped durations

- **Rules** — CR 500.4, 500.5, 500.5a, 511.2, 511.3, 513.2, 703.4p, 703.4q
- **Verdict** — **`Duration` has six variants and not one of them is a step or
  a phase**: `UntilEndOfTurn`, `UntilYourNextTurn`, `WhileSourceOnBattlefield`,
  `WhileEnchanted`, `WhileEquipped`, `Indefinite`. "Until end of combat" — the
  common case, and the one CR 500.5a names — is **inexpressible today**.
- **Size** — an additive variant plus expiry hooks, so a feature; the care is in
  *when* they fire. CR 500.4 expires effects at the **beginning** of a step or
  phase and 500.5 at the **end**, they are different hooks, and CR 513.2 carves
  out an explicit exception for effects created during the step they name.
  `remove_expired_at_cleanup` is today's only expiry point, and it is
  turn-scoped.
- **Blocks** — every "until end of combat" pump; 511.3's combat cleanup of
  `AttackingInfo`/`BlockingInfo`; 703.4q's mana-pool emptying per step.
- **Atoms** — 8.
- **Owner** — none yet, and the hooks now exist: RE-1 landed
  `GameEvent::{TurnBegin, PhaseBegin, StepBegin}` and
  `GameState::{begin_phase, begin_step}` (2026-09-11), so a step- or
  phase-scoped `Duration` has a place to expire from. CR 500.4's "as a step or
  phase **begins**" is `begin_step`/`begin_phase` after the proposal survives,
  beside `on_turn_begin`, which RE-1 added for CR 611.2b's turn half; CR 500.5's
  "as it **ends**" is `on_step_end`/`on_phase_end`, which RE-1 also made run
  only for a unit that happened. This is the PR after RE-1 once an "until end
  of combat" consumer appears.

### 2.13 Deck-construction limits are configured and unenforced

- **Rules** — CR 100.2a, 100.2b, 100.4a
- **Verdict** — `DeckLimits` is **fully modelled and never consulted.**
  `GameConfig` carries `min_deck_size`, `max_deck_size`, `max_copies` and
  `sideboard_size`, and `standard()` and `limited()` both set them correctly
  (60/4/15 and 40/none/none). **No validator exists** — nothing in the tree
  reads them. Configuration with no consumer, which is the §2.7 `chosen_modes`
  shape again in a milder form.
- **Size** — tiny; one validation function against a `Decklist`. CR 100.4a is
  the only clause with a wrinkle: the copy limit counts **main deck and
  sideboard combined**, so the check is not per-list.
- **Blocks** — nothing in play. It matters for a deckbuilding UI and for
  refusing a malformed decklist rather than starting a broken game.
- **Atoms** — 7. **This is a candidate for `codebase-state.md` Deferred
  Migrations instead of a backlog entry** — it is unconsumed scaffolding, not an
  unbuilt mechanic — and is filed here only because the whole cluster surfaced
  together. Move it if that reads better.
- **Owner** — none yet.

### 2.14 Untap restrictions — "doesn't untap"

*Added 2026-08-31 from `owed`'s kept pair, during the roadmap review — the
triage's thirteen entries plus one.*

- **Rules** — CR 502.3, 703.4c
- **Verdict** — nothing can say "this permanent doesn't untap during its
  controller's untap step." `Duration` has no such shape, and the untap step's
  sweep consults nothing before proposing untaps. The *seam* exists —
  `Primitive::Untap`'s contract is already "performers are loud; callers check
  legality," and RB's stun counters already intercept the untap **event** as a
  replacement — what is missing is the static-restriction flavour and the
  query the sweep would ask.
- **Size** — small: a continuous-effect query consulted by the untap TBA. The
  design call is the home — a "can't" (`is_blocked`, RS's model) or an
  untap-step filter — and it is taken at RS scoping, since
  `cant-effects-architecture.md` owns that boundary. One-shot riders
  ("doesn't untap during its controller's next untap step", Frost Titan)
  additionally want item 6's delayed-trigger machinery.
- **Blocks** — Colossus-class "doesn't untap" statics; exert (§2.5's
  skip-untap tracking is this plus a turn marker); tap-and-lock cards.
- **Atoms** — ATOM-502.3-002 and ATOM-703.4c-002, both in `owed`'s kept nine.
  **This entry captures their tickets, so §3.3's policy re-files them at the
  next `owed` pass** — left in place for now so this addition moves no gate
  number.
- **Owner** — none yet; RS scoping decides.

### 2.15 Player-scoped continuous effects (CR 613.10–613.11)

*Added 2026-08-31 by the `PlayerState` type sweep — probed by Winter,
Misanthropic Guide, whose hand-size clause is CR 613.11's own worked example.*

- **Rules** — CR 613.10, 613.11 (less the cost half, which is CR 601.2f's and
  travels with §2.1, and the prohibition half, which is
  `cant-effects-architecture.md`'s); CR 402.2 as modified
- **Verdict** — **`PlayerState` stores rule values and no effect can reach
  them.** `max_hand_size` is seeded from config and read raw by
  `handle_cleanup_discard`; nothing can say "you have hexproof" for a player,
  and target legality consults nothing player-scoped. CR 613.11 applies these
  after all object layers, in timestamp order — that application point does
  not exist. `lands_per_turn` is the same shape and already dispositioned to a
  layers-owned query (§3.1's CR 305 row); this entry is the general surface
  both should share — **and owns `lands_per_turn` outright since
  2026-09-07**: `cost-architecture.md` §3.9 took CR 613.11's cost half only
  and sent the timestamp half's values here (`codebase-state.md` main item
  13).
- **Size** — small-to-medium: an effective-value/ability query beside the
  oracle layer plus one post-layer, timestamp-ordered application step; one
  production reader to migrate today. Conditional gating ("as long as…")
  arrives with critical-path item 6.
- **Blocks** — Winter's clause and the Reliquary Tower class (62 cards touch
  maximum hand size, 43 remove it — Commander staples); player hexproof and
  shroud (Leyline of Sanctity's class, 14 cards); effective lands-per-turn's
  general form.
- **Atoms** — the two CR 402.2 atoms cover the base rule only; the
  modification half has none. Corpus-thin — see §5.
- **Owner** — none yet.

### 2.16 Counters on players (CR 122.1) — ✅ graduated 2026-09-13 (RE-5)

*Built as this entry designed it: `PlayerState.counters` is the kind → count
map (a `BTreeMap`, so a walk is process-independent) sharing `CounterType`
with a permanent's, `CounterType::{Poison, Energy}` are its first kinds,
CR 704.5c reads it, `GameAction::AddCounters { subject: CounterSubject::Player
(..) }` puts them on and `Primitive::GetCounters` is Oracle's "you get". Live
Fast is the producer; Vorinclex's and Winding Constrictor's player halves the
watchers. Costs paid in energy wait for their first card
(`cost-architecture.md`'s CP-1 slot). The entry is kept as written for the
record.*

- **Rules** — CR 122.1's player half; proliferate reads it (CR 701.34a)
- **Verdict** — `PlayerState.poison_counters: u32` hardcodes one kind where
  the CR wants kinds-on-players: energy (145 cards), experience (16,
  Commander-native), rad. A kind → count map plus a handful of reader
  migrations (the poison SBA today; proliferate later).
- **Size** — small; the one design decision is whether player counters share
  `CounterType` with object counters — they should, because CR 701.34a
  iterates permanents and players in a single sweep.
- **Blocks** — every energy card; the experience-counter commanders;
  proliferate reaching players; the "or player" halves of Vorinclex,
  Monstrous Raider and Winding Constrictor.
- **Atoms** — thin under the obvious phrasings; see §5.
- **Owner** — **RE-5** (2026-09-11, re-cut on review): `AddCounters` gains a
  `CounterSubject { Object, Player }` beside its putter field while the type
  is on the table, `PlayerState.poison_counters` becomes the kind → count map
  this entry designed, CR 704.5c reads the map, Live Fast is the producer and
  Vorinclex's and Winding Constrictor's player halves the watchers —
  `replacement-architecture.md` §9, RE decision 4. Costs paid in energy wait
  for their first card. **Struck as graduated when RE-5 lands.**

### 2.17 Extra phases and steps (CR 500.8, 500.9, 500.10)

> **The phase half graduated 2026-09-14 to `replacement-architecture.md`
> (RE-10) and is struck below.** `GameState.turn_plan` is CR 500.1's
> sequence as a `Vec` the drainer indexes, rebuilt per turn;
> `Primitive::ExtraPhases` splices at `cursor + 1`; `next_phase`'s chain is
> deleted; Aggravated Assault is the producer. CR 500.8's *"most recently
> created phase will occur first"* needs no comparator — a later splice at
> the same index pushes the earlier one back. **This entry stays open for
> the *step* half (CR 500.9, 500.10) only**, whose one field is named on
> `PlannedPhase` and whose only producer is a triggered ability.

**The turn half graduated 2026-09-11 to `replacement-architecture.md` (RE-1):**
`GameState::turn_queue` is CR 500.7's stack, pushed by `Primitive::ExtraTurn`,
drained by `advance_turn`'s `next_turn_taker`, with `turn_rotation` beside it
so an extra turn does not move the natural rotation. CR 500.7's APNAP sentence
has no producer — no `EffectRecipient` that primitive accepts resolves to more
than one player. **This entry is the half that did not graduate**, and it is a
mechanic rather than a migration, which is why it is here and not in
`codebase-state.md` (item 116 is a pointer).

- **Rules** — CR 500.8 (extra phases), 500.9 (extra steps), 500.10 + 500.10a
  (a step added after a *phase* creates the containing phase, and that phase's
  other steps are **skipped** — CR 500.11, which makes this rule a skip
  *producer* and the reason it reads as replacement-adjacent).
- ~~**Verdict** — **the drainer's cursor cannot hold two of the same phase.**~~ *(graduated; what shipped is above)*
  RE-1 wrote it as `(Option<PhaseType>, Option<StepType>, phase_began)` and
  `next_turn_unit` answers "what follows" from the phase *type*, so a turn with
  two combat phases cannot say which one the cursor is at. The shape that can
  is the `TurnPlan` `state::game_state::next_phase`'s pre-RE-1 TODO described
  and RE-1 re-pointed at rather than built: a per-turn `Vec` of phases the
  cursor indexes, spliced by a new producer. **So `advance_turn` is rewritten a
  second time** — see `replacement-architecture.md` §11 item 49, which is the
  finding this entry exists to carry.
- ~~**Size** — the plan plus its index cursor, one `Primitive`, and the per-turn
  clear: ~250–350 additions, plus a card and its tests. Well inside one PR.~~
  *(shipped at +1,066 / −171; the additions this line did not price are the
  46 sites that write `GameState.phase` by hand — see RE-10's findings)*
  **Aggravated Assault is the cheapest whole card** ({2}{R} enchantment,
  `{3}{R}{R}` activated, `ActivationRestriction::OnlyAsSorcery`, which exists):
  its only other need is `Primitive::Untap` accepting an
  `EffectRecipient::FilteredPermanents`, the arm `DealDamage` and
  `CreateReplacement` already have. Seize the Day needs flashback and World at
  War needs rebound and "creatures that attacked this turn"; Obeka needs item
  6's triggers, so **CR 500.10 cannot land before item 6** whatever happens to
  500.8.
- ~~**Blocks**~~ *(the phase half's; the step half blocks Obeka alone)* — `o:"additional combat phase"` is **46 cards** (Scryfall,
  2026-09-11), `o:"additional main phase"` 9, `o:"additional upkeep step"` 3.
  And one card that is *already registered*: Moment of Silence's first ruling —
  "if they manage to have two combat phases, then only their next one combat
  phase is skipped" — has no engine-produced board, only the cursor-moving
  fixture in `phase_re1_integration_test`.
- **Atoms** — **none.** `session-4.md` marks 500.8, 500.9 and 500.10 DEFERRED
  with no atom ids and assigns them to *Phase 9*, whose stated content is
  formats and multiplayer; that assignment reads like the era's `TurnPlan` TODO
  rather than a judgement, and is flagged here rather than edited, because the
  corpus is authored and corrections land in the session file. Either way
  `specdb owed` cannot ask for these and no phase's exit criteria move.
- **Owner** — **`replacement-architecture.md` §9, RE-10** (2026-09-11): the
  owner's call at RE-1's review, on the argument that the "written once"
  sentence is unkept and that 46 cards plus a registered card's fixture-only
  ruling outweigh "RE is event kinds". **The phase half of this entry graduates
  when RE-10 lands**, leaving the step half below.

- **What RE-10 does not take: CR 500.9 and 500.10's extra *steps*.** They
  cannot come into RE at all — their only producer is Obeka, Splitter of
  Seconds, whose ability is **triggered**, so they are critical-path item 6's
  whatever RE does. RE-10 decision 4 names the one field they want, an
  `Option<Vec<StepType>>` on `PlannedPhase` that overrides a phase's natural
  step list, which is precisely 500.10's *"any other steps that phase would
  normally have are skipped"*. **This entry stays open for that half** after
  RE-10 lands.

### 2.18 Mana payment — CR 732.1's reversal, and an auto-payment oracle

- **Rules** — CR 732.1 (reversing mana abilities activated during an illegal
  action); CR 601.2g–h.
- **Verdict** — the rewind site in `cast_spell` takes 732.1's "may not
  reverse" branch unasked: no `DecisionProvider` question exists for "reverse
  the mana abilities you activated while casting", which is the undo Arena
  offers and the GUI will be expected to. Separately, nothing helps a player
  or an agent *choose* the taps — the 601.2g window asks one ability at a
  time, and the agent either assembles a covering set or rewinds. Measured
  (RC-4b, 40 `performance` games, seed 12345): the random agent rewound 303
  casts, ~7.5 per game, each a decision round-trip spent on nothing. **A third
  gap, and this one is engine-side (2026-09-02):** the generic-split prompt
  offers every type in the pool as a bucket with the pool's amount as its max,
  so a split that spends a color a pip still needs passes the prompt's own
  validation and fails at `ManaPool::pay`. About five casts per game did that,
  and until `codebase-state.md` 16c closed they *resolved unpaid*; now they
  rewind, on top of the 7.5 above. **Closed 2026-09-03, in two halves.** The
  DP-side one (`ui/random.rs`): the random agent taps for the pip it still
  owes, which took its land taps per cast from 7.66 to 3.18 on an any-color
  mana base. Then the engine-side clamp (`mana/generic-split-clamp`):
  `ask_choose_generic_mana_allocation` caps each bucket at `available − pips
  of that type`, so no DP can name an unpayable split and none needs payment
  law to avoid one. The agent's own copy of the clamp came out with it —
  keeping it would have subtracted the pips a second time — and the fuzz A/B
  is identical on every counter and byte-identical in the event streams, which
  is what proves the prompt now computes what the agent was computing. **What
  is left of this entry is the reversal prompt and the oracle.** **The
  oracle is built (2026-09-08, CM-4), and it is two decorators rather than
  one.** `ui::ManaWindowStop<D>` declines `ManaAbilityWindow` once the locked
  mana component is covered — the stop that used to sit in the engine's loop
  (`codebase-state.md` main item 70) — and `ui::AutoPayer<D>` answers
  `GenericManaAllocation` (CR 601.2h) and `OrderCostReductions` (CR 601.2f).
  Clients compose the stack they want; `cli_play`'s human seat takes both, its
  bot seat and `fuzz_games` take the stop alone, and `--no-auto-pay` drops it.
  **The stack invariant is one decorator per `ChoiceKind`**, so composition
  commutes and stack order carries no meaning — that is what a third
  automation (priority passing, auto-block) extends rather than a scope enum
  inside the payer.
  **The criterion is strict: a payer answers a prompt only when it has exactly
  one legal answer.** So the sacrifice choice is out — which creature dies is
  strategy, and belongs to whatever stacks a decorator for it. So is the generic
  split whenever the pool has anything spare, since which mana pays the generic
  decides what is left up for the rest of the step; the payer takes it only when
  the caps admit one allocation. `OrderCostReductions` is the one prompt it
  always answers, because §3.4's theorem says every order gives the same total.
  §3.4 has the argument, matched exhaustively so a new payment prompt has to
  pick a side.
  **What is left of the oracle here is the solver half**, which CM-4 did not
  build: the bipartite matching between pips and the colours each ability can
  make, so a client can be told *which* sources to tap rather than answering
  one window prompt at a time. `ManaWindowStop` only ever declines; it never
  picks, which is what keeps a human's taps the human's. **§2.22 sizes and
  schedules it** beside every other middleware v1 wants, because the solver's
  hard half is the Arena auto-tapper problem and that is a policy question
  rather than a payment one.
  The reversal prompt is main item 72, and CM-4 placed it with critical-path
  item 6 rather than here: reversing a mana ability fails the payer's own
  criterion, and the board it is observable on is waiting for the trigger
  phase. **And a third thing lives here now (2026-09-07, CM-3):**
  the *staged* payment Arena offers — delve exiles, convoke taps, a sacrifice,
  all shown and take-back-able until the player confirms the cast. CM-3 made
  that possible without any engine facility by separating deciding from
  performing: every payment prompt is asked against one board before anything
  moves (`cost-architecture.md` §3.12), so a client can buffer the answers and
  let the player revise them, and the engine never holds a half-performed
  payment to undo. It is Phase 10 GUI work, not engine work, and the thing to
  protect is the invariant rather than the UI — a phase that asks a payment
  prompt *after* a payment would take the staging away and make CR 732.1's
  cancellation mandatory.
- **Size** — small for the reversal: one prompt at the rewind site, and the
  taps undone silently, the way the cast's own rewind is — a 732.1 reversal is
  not an untap event and nothing may observe it. The oracle is harness-side:
  for one cost it is a bipartite matching between pips and the colors each
  ability can make, polynomial over `enumerate_activatable_mana_abilities` and
  `remaining_cost_after_pool`; the lookahead across the rest of the hand — the
  part Arena's auto-tapper gets wrong — is a policy for the AI harness and the
  GUI assistant, not an engine rule.
- **Blocks** — nothing rules-wise; both v1 use cases ergonomically. The AI
  harness pays the rewind rate above in every game; a human pays it in clicks.
  The split's ~5 per game are gone; the window's ~7.5 are not.
- **Atoms** — ATOM-601.2h-002 and ATOM-601.2-001 are claimed partial by
  RC-4b's rewind test; 732.1 has none.
- **Floor 1 and this entry** — the solver moves floor 1 only through the stack
  a seat runs. Its decorator (§2.22 row 7) is off by default on a bot's seat,
  so the reference stack's reading does not move when it lands. Turned on, it
  answers the window's picks and the splits with surplus, 194 and 41 a game at
  60 cards (§2.22's counts), about half the default's decisions: the agent and
  the GPU get half the work per game, and the engine, no slower, becomes the
  tighter constraint. That is another operating point, not a re-base
  (`engineering-practices.md` §3.1). The engine's own lever applies under
  either stack: the window re-enumerates every mana ability once per tap,
  13.5% of instructions (`layers-architecture.md` §12), where an inventory
  taken once per cast and a payability check per tap would do.
- **Owner** — none yet.

### 2.19 Any-color mana — "Add one mana of any color"

- **Rules** — CR 106.1b (the six types of mana; "any color" is a choice among
  five of them), 605.1a; CR 111.10a and 111.10c (Treasure and Gold print the
  text).
- **Verdict** — `ManaType` has no any-color variant and `ManaOutput.mana` is
  `Vec<(ManaType, AmountExpr)>`, so the ability cannot be written; and
  `engine/mana.rs::resolve_mana_effect` accepts only `ProduceMana` atoms with
  `Fixed` amounts and never asks a `DecisionProvider` anything, so a new atom
  would still resolve without the color being chosen. The choice belongs at
  resolution, not at activation — `ChoiceKind::ManaAbilityWindow`'s docs
  already anticipate a Cavern of Souls whose second mode is "add one mana of
  any color", and today that mode would have to be five abilities. **The
  fuzz pool sidestepped it on 2026-09-03** with Everywhere
  (`cards/dual_lands.rs::everywhere`): five one-color abilities, so the color *is*
  the ability picked in the 601.2g window. That is faithful for a five-type
  land and wrong for everything below, which prints one ability and a choice.
- **Size** — small-to-medium. A `ManaOutput` arm (or a `ManaAtom`) for "one
  mana of any color", one `ChoiceKind` asked in `resolve_mana_effect`, and
  `fuzz_games::land_mana_colors` learning to read it. Riders are separate
  and already carriable: City of Brass's damage and Mana Confluence's life
  loss are a mana ability with a non-mana effect, which the `Sequence` arm
  can hold once the primitive exists. Command Tower needs color identity on
  top, which is the Commander track's, not this one's.
- **Blocks** — Command Tower, Birds of Paradise, Chromatic Lantern, City of
  Brass, Mana Confluence, Gemstone Mine, Exotic Orchard; every Treasure and
  Gold token; Cavern of Souls' second mode; every mana filter ("{1}: Add one
  mana of any color"). Any Commander-viable mana base. **And every effect
  that makes a Treasure** (RE-4's review, 2026-09-13): the Treasure def in
  §2.27's library, and with it Xorn, Chatterfang and Hullbreacher, whose
  replacement shapes `GameActionTemplate::CreateTokens` already carries.
- **Atoms** — ATOM-111.10-001's expected result prints the text; ATOM-605.3c-001's
  board is a mana filter that adds "one mana of any color". Neither is about
  this mechanic and neither is claimed. The spending-side rules — CR 609.4b's
  "as though it were mana of any color", ATOM-609.4b-001..003 — are a
  different surface (`ManaPool::pay`) and stay where they are.
- **Owner** — none yet. (Recorded here on 2026-09-03 when
  `plans/handoffs/pool-five-color-land.md` was closed; its City of Brass
  recommendation is this entry.)

---

### 2.21 Can a client render what it is being asked? — **shipped 2026-09-18 (A4j)**

- **What shipped.** `ChoiceKind::subject() -> Option<ObjectId>`, matched
  without a wildcard, so a new variant cannot compile without deciding which
  object it is about. `tests/prompt_subject_test.rs` walks whole games at two
  seats and four, each deck a different window of the registry, and checks
  that every prompt raised carries a subject or is a declared `None`; a
  second test builds one of each variant for the prompts no registered card
  raises. `ui/cli.rs` renders its own strings behind one exhaustive
  `prompt_line` with no `_ =>` arm, so a new variant must get a line there
  too.
- **Built and taken out in review (the owner, 2026-09-18).** A
  `describe() -> PromptText { rule: &'static str, text: String }` on the
  engine, per item 141's "the engine owns the text". Two objections, both
  right: a rule citation riding on a decision is superfluous, since the
  variant is already the stable handle and the number belongs in its doc;
  and the strings are not the context — the context is the variant, its
  typed fields, the options and the bounds, which serialized *is* the
  literal-named enum a client keys on. Only a text client would ever consume
  engine English; a GUI will not display it and an agent will not parse it.
  The rendering went back to the CLI, and item 141's shape is amended: what
  a boundary adapter sends is the variant, the subject id, the option ids
  and the bounds, and each client renders.
- **The gap, counted against the tree.** 18 of 25 variants carried an
  `ObjectId` under five field names, which is why the contract is a method
  and not a name. Three carried none and gained it: `ChooseAlternativeCost`
  and `ChooseAdditionalCosts` were bare unit variants, so a client could not
  tell which spell was asking, and `GenericManaAllocation` carried a
  `ManaCost` and no id. **`LegendRule` was counted as the fourth and is not
  one.** CR 704.5j singles out no member of the group; the options are the
  whole subject, and a highlighted member would misstate the rule — it is
  the fourth legitimate `None` beside `PriorityAction`, `DeclareAttackers`
  and `DeclareBlockers`. Two more are runtime rather than shape:
  `Discard { source: None }` is CR 514.1's cleanup discard, which the test
  allows in that step only, and `ChooseReplacementEffect { affected_object:
  None }` is an event about the choosing player. `Option<ObjectId>` held;
  nothing argued for a richer `PromptSubject`.
- **Found on the way.** `ChooseCopySource`'s doc cited CR 707.4, which in
  the baseline is a copying permanent changing what it copies; Cytoshape's
  choice is CR 608.2d's, announced while applying the effect. The doc says
  so now. The check that found it read `MTG-Rules/`, which is gitignored,
  so it failed CI; it went with `describe()`.
- **Not started here, on purpose.** `SelectRecipients` still carries an
  `EffectRecipient`; item 141's payload rule is the incoming contract for
  new arms, and retiring the AST there is its own piece.
- **The check.** Every gameplay counter `IDENTICAL` on both pools at two
  seats and four, `Memo hits` included — the row adds one method and fills
  three payloads, and decides nothing.
- **The rest of this entry is the record as it was sized.** One name in it
  had gone stale by the time the row ran and is corrected in place:
  `DiscardToHandSize` is `Discard { source: None }`.

#### 2.21 as sized (2026-09-08)

- **Rules** — none. This is an engine-interface question, not a CR one, which
  is why it needs writing down: nothing in the CR will fail if we get it wrong.
- **Verdict** — a `ChoiceContext` is supposed to carry enough that a UI can say
  *why* a player is being asked and highlight *what* they may pick, and today
  it does: the options are `ChoiceOption`s naming real objects and players, and
  each `ChoiceKind` carries its source (`ChooseSacrificeForCost`'s
  `spell_or_ability_id`, `ApplyOptionalReplacement`'s and
  `ChooseAuxiliaryZoneChange`'s `source`, whose doc argues the point outright:
  "why am I being asked this is answered by the source and by nothing else on
  this prompt"). **But nothing enforces it.** The discipline lives in prose on
  individual variants, there is no test that a new `ChoiceKind` carries a
  source, and the only consumer that would notice is `ui/cli.rs`, which is
  omniscient and formats a line of text. The next variant added under time
  pressure can drop the source and every test will pass.
- **Not §2.9.** That entry asks *may* this player see the object; this one asks
  *can the client draw the question*. They fail differently and are fixed
  differently: §2.9 is a per-viewer query over zones, this is a shape
  obligation on one enum. A UI can be perfectly legal about hidden information
  and still be unable to tell the player what it wants from them.
- **Size** — small if taken as a gate rather than a redesign. The shape:
  a `fn subject(&self) -> Option<ObjectId>` (or a richer `PromptSubject`) that
  every `ChoiceKind` answers, matched exhaustively so a new variant must
  decide, plus a test that walks a game and asserts every prompt raised carries
  one. The variants that legitimately have no subject — `PriorityAction`,
  `Discard { source: None }` — say `None` and say why, which is the same discipline
  `ZoneChangeCause`'s no-catchall rule uses. ~1 small PR.
- **Blocks** — the GUI half of v1, quietly. Not a rules bug and not something
  the fuzz harness can find, because `RandomDecisionProvider` picks by index
  and never asks what the prompt means. The cost of skipping it is discovered
  during GUI work, one prompt at a time, by which point the variants are
  numerous and the fixes are individually cheap and collectively not.
- **Atoms** — none, and there will be none: the corpus is derived from the CR
  and the CR has nothing to say about interfaces.
- **Where it came from** — the owner, reviewing CM-3's sacrifice prompt
  (2026-09-08): "I'm generally getting paranoid we're not actually giving the
  UI enough info to display the gamestate properly." That prompt turned out to
  be fine; the absence of anything that would have told us is the entry.

### 2.20 Several target clauses on one spell — **shipped 2026-09-17 (A4i), except CR 601.2d**

- **What shipped.** CR 115.3's instance is the unit: `StackEntry.chosen_targets`
  is a `Vec<TargetInstance>`, `Effect::instances` walks an effect's
  clauses in printed order, `EffectRecipient::SameInstanceAs` is how a later atom
  refers back to one — by the clause's position in that list — and
  `resolve_effect` hands each atom its own instance as a flat slice.
  `ObjectFilter::OtherThanInstance` is "another target"; CR 608.2b is asked per
  instance *and* per target, so a spell resolves unless every target is illegal
  and an illegal one is simply not affected.
- **The announcement order is the engine's, not the rule's.** CR 601.2c fixes no
  order and does not say whether the choices are simultaneous; the loop asks one
  clause at a time because `OtherThanInstance` needs the earlier answers. The two
  are outcome-equivalent — every assignment a simultaneous announcement allows is
  reachable in index order, and the loop cannot produce one it forbids — until
  601.2c's *"must be chosen as a target … the maximum possible number"* lands,
  which is a global optimum a greedy loop cannot see. `codebase-state.md` item
  157. Six of the eight atoms are covered;
  the consumers are Seeds of Strength (pooled), Incremental Growth, Jagged
  Lightning, Plague Spores and Seat of the Synod. The A/B was `IDENTICAL` on
  `performance` and found a live bug on `stress` (`codebase-state.md` item 152).
- **What is left: CR 601.2d, and it is scheduled.** ATOM-601.2d-001 and -002 —
  "deal 3 damage divided as you choose among one, two, or three targets". It is
  a second mechanism rather than a second clause: `TargetCount` gains a divided
  form, `TargetInstance` a parallel allocation, `DealDamage` a per-target
  amount, and the `DecisionProvider` a prompt that returns numbers rather than
  indices. ~500–700 lines with Arc Lightning as its consumer (the atoms are
  written around its numbers) and Forked Bolt as the cheap pooled option at one
  mana. `roadmap-v2.md` row **A4l**, back-stopped before Phase 8's breadth,
  where the 48 "divided as you choose" cards live; it blocks nothing on the
  spine.
- **The rest of this entry is the record of the design as it was sized**, kept
  because the atoms and the card counts are still what A4j is sized against.

#### 2.20 as sized (2026-09-04)

- **Rules** — CR 115.1 ("one or more objects or players as targets"), 115.3
  (one object may be chosen once per *instance* of "target", and for several
  instances), 601.2c (targets are announced per instance), 601.2d (a divided
  effect is announced per target), 608.2b (a spell with *some* legal targets
  resolves, and the illegal ones are not affected).
- **Verdict** — a spell has one recipient. `StackEntry` carries one
  `EffectRecipient` and one flat `chosen_targets`; `targeting::effect_recipient`
  takes a `Sequence`'s *first* atom's recipient and every later atom resolves
  against the same targets; `cast_spell` asks one `SelectRecipients`; and
  `any_targets_still_legal` answers for the whole list. "Target creature
  deals damage equal to its power to another target creature" has no slot for
  its second choice, no prompt for it, and no rule that keeps the spell
  resolving when only one of the two is gone. Wants a recipient *per atom* (or
  per instance of "target"), targets keyed the same way, a CR 601.2c loop over
  the instances, and CR 608.2b's per-target legality at resolution.
- **Size** — medium, one PR in the band: per-atom target slots on `StackEntry`,
  `cast_spell` / `activate_ability` looping the instances, `resolve_effect`
  reading its own atom's slot, `any_targets_still_legal` becoming "any instance
  still legal" with the illegal ones skipped. No new `DecisionProvider` shape —
  one `SelectRecipients` per instance. CR 601.2d's division is a second step on
  top and wants `DecisionProvider::allocate`.
- **Blocks** — by Scryfall (2026-09-04): 228 instants and sorceries whose text
  names "target" twice in one sentence, and 165 permanents with such an
  ability; among them 36 bite spells ("another target creature"), 32 of
  Decimate's "target X and target Y" shape (Decimate itself is four), and the
  48 "divided as you choose" spells (Electrolyze, Fiery Justice) that also
  want 601.2d. Aura Finesse, LH-2's alternative consumer, is here too.
- **Atoms** — ATOM-601.2c-003/-004 (one object for several instances of
  "target", never twice for one), ATOM-608.2b-002 and -005 (some targets
  legal → resolve, the illegal ones unaffected), ATOM-601.2d-001/-002
  (division), ATOM-115.3-001/-002. None claimed; every one of them needs two
  target clauses to build.
- **Owner** — none yet; **scheduled** in `roadmap-v2.md` §3a beside A,
  back-stopped before RS-2 and before A6's first PR (2026-09-04), because
  both would otherwise build on the one-recipient shape. (Recorded 2026-09-04
  in LH-1's review: the `effect_recipient` doc had described the first-atom
  rule as "the convention of every card written so far", and the reviewer
  asked for the whole pool.)

### 2.22 Which `DecisionProvider` middleware v1 ships with — the census (A4k, 2026-09-18)

- **Rules** — none. An engine-interface question like §2.21, and written down
  for the same reason: nothing in the CR fails if we get it wrong. The CR does
  reach in from the other side, though, and it is what makes the rows below a
  policy question rather than a payment one: CR 601.2g's window, 601.2h's
  split, 601.2f's order, 510.1c's division, 603.3b's order and 732.1's
  reversal are each *the player's* choice, so a decorator that answers one is
  a client choosing on the player's behalf — fine on a bot's seat, a toggle on
  a human's, never the engine's.
- **Verdict** — CM-4 built two decorators (`ui::ManaWindowStop`,
  `ui::AutoPayer`) one at a time against one phase's need, and settled the
  composition rule; what was missing was the census — every row v1's two use
  cases want, sized against the tree and sequenced against each other. **Three
  rules come out of it, and every row below is read against them.** They were
  amended at the owner's review of PR #169 the same day, and the amendments
  are marked.
  1. **A prompt whose answer cannot change the game belongs to the engine, not
     to a middleware.** Two cases, and the engine already holds one of each. A
     prompt with one legal answer, which `ui::ask` declines before any provider
     is asked (`ask_discard`, `order_scry_group`, `ask_select_recipients`,
     `forced_allocation`, and five asks that assert two or more candidates —
     A4e, `codebase-state.md` item 145). And a prompt whose every legal answer
     leaves the same game, which the engine elides with stated expiry
     conditions — item 47's `pipeline::ordering_cannot_change_outcome` for CR
     616.1's prompt. A decorator can only spare a round trip the engine had
     already decided to spend, which is the ceiling on what any row here saves
     — and the reason the payer's forced branch retires in this PR and the
     rest of the payer follows it (row 2). A note on the number the tree cites
     for the first case, since the census read it: `ui/ask.rs` and item 145
     say "CR 102.2", and in `tmnt.txt` 102.2 is the two-player-opponent rule;
     the CR states the general form nowhere, and the anchors are CR 616.1's
     "two or more" and 601.2f's "if multiple". One comment fix, next time a
     hand is in the file.
  2. **Every decorator is a policy, and a policy's seat decides where it
     lives.** The answers differ and a rule picks one — declining the window
     once the cost is covered, passing priority, dividing combat damage, which
     sources to tap. It lives in `ui::` only when both use cases want the same
     rule; a **human's seat takes it under a toggle**, because the automation
     is a choice made for the player, and a **bot's seat takes it at
     construction**, because for an agent it is the harness's policy and
     nothing else. **The census's first draft had a second kind**,
     *answer-preserving* — every legal answer leaves the same game,
     `OrderCostReductions` by `cost-architecture.md` §3.4's theorem — living
     in `ui::` with no toggle and stacked by every client. **Struck at review
     (the owner, 2026-09-18):** an answer that cannot change the game is rule
     1's, the engine's to elide, and a decorator for it is a second home for
     one fact. CM-4's "exactly one legal answer" was the right bar for what a
     payer might answer; it is now the bar for what the engine declines to
     ask, and the supplemental doc's `AutoYieldDP` and auto-tapper are what
     decorators are for.
  3. **At most one decorator answers any one prompt**, so composition
     commutes and stack order carries no meaning (`ui/mana_window_stop.rs`,
     CM-4's review). Two predicates on one `ChoiceKind` coexist when they are
     disjoint — the stop answers the window once covered, a solver while it
     is not. **Full control is therefore a switch above the stack, never a
     wrapper in it and never a handle inside each decorator** (amended at
     review; row 3): on, the seat's prompts go to the raw provider; off, to
     the decorated stack. The supplemental doc's ordering question (§6) is
     closed by construction rather than by convention.
- **The tree, counted 2026-09-18.** `ui/ask.rs` has **24 `ask_*` functions**,
  the whole decision surface, at **26 engine call sites in 12 files** — every
  ask once, `ask_select_recipients` and `ask_discard` twice. The
  classification below recorded 27 on 2026-09-15 and was right for its tree:
  `cast.rs` asked recipients from two sites, casting and activating, and A4i's
  `put_on_stack.rs` asks from one. A loose grep reads 28 today because two
  test-function names in `cards/phase_rd_cards.rs` match `ask_…(`; count calls.
  `ChoiceKind` has **25 variants**, every one answering `subject()` since A4j
  (PR #168). **Two decorators, both CM-4's, both stateless.** The wiring:
  `cli_play` runs `AutoPayer(ManaWindowStop(Cli))` for the human and
  `ManaWindowStop(Random)` for the bot, `fuzz_games` runs
  `ManaWindowStop(Random)` behind a `MiddlewareConfig` with one field, and
  `--no-auto-pay` drops the stack on both. **`AutoPayer` has one client**,
  `cli_play`'s human seat — no test stacks it and the harness never has — so
  nothing this entry does to it can move a fuzz counter. The engine answers
  every forced prompt itself since A4e; the four `validate_*` helpers count
  the rest as `Decisions` and `Priority decisions` (item 138's instrument, the
  rows `fuzz_ab.py` diffs). The engine holds one `&dyn DecisionProvider` for
  the whole game — `Game::setup`, `run_turn` and the signatures between — which
  is the fact row 3 is shaped around.
- **The supplemental doc's §4, re-derived against what CM-4 built.** Four
  claims stood, three moved, one was wrong:
  - **The pattern stands**, as a generic parameter rather than a
    `Box<dyn DecisionProvider>`: `ManaWindowStop<D>`, `AutoPayer<D>`, `inner()`
    reaching the wrapped provider. `DispatchDecisionProvider` is the same
    pattern applied to seat routing, as §4 said. **Its `AutoPayDP` is three
    things today**: the window's stop (`ManaWindowStop`, built), the forced
    split (the engine's, A4e) and the tap solver (not built, §2.18 — the row
    below). "Solves the generic split outright" was the wrong claim: §3.4
    restricted the payer to the forced case because which mana pays the
    generic decides what is left up for the step, and A4e then moved even the
    forced case into the engine.
  - **`RawDP` is the undecorated provider**, not a mode: `--no-auto-pay` is
    it, and rule 1 is what makes "raw" mean the same thing for a human and an
    agent — neither is handed a prompt the engine could answer. Full control
    (row 3) is `RawDP` entered and left mid-game.
  - **"Full control generalized" per wrapper moved**: one toggle, not one per
    decorator, because the per-wrapper form is the scope enum CM-4's review
    took out in another shape, and because two of the rows are counterweights
    (auto-yield makes the tell that full control answers) — a player who could
    switch one off and not the other would have the leak without the remedy.
    The *mechanism* §4 sketched — each decorator consults a switch — is not
    how it is built either: the switch sits above the stack and the
    decorators know nothing of it.
  - **The Arena problem is correctly diagnosed and belongs to a client**: it is
    the *preference* half of the tap-solver row, and this census frames it as
    policy (that row).
  - §6's five open questions: **wrapper ordering** — closed (rule 3); **mana
    solver design** — the oracle half is a matching, sized below, and the
    lookahead is the client's; **preference system** — the client's, in
    whatever shape the client wants, mapped onto the matching's edges (rows 6
    and 7); **training mode** — the harness builds its own stack at
    construction, so it always knows what is on; **serialization** — both
    built decorators are stateless, the switch's position and auto-yield's
    yield-until condition are client state that must ride in the recorded
    input stream (the replay caveat below), and nothing here is
    outcome-bearing in item 40's sense, since dropping a decorator and
    re-asking the human reaches the same game.
- **The candidate rows, sized and sequenced.** Kind is rule 2's; "human" means
  under the toggle; "bot" means the harness's choice at construction. Per-game
  counts are the classification table's (four seats, `performance` / `stress`
  / Commander-scale `stress`, the 2026-09-15 stream).

  | # | Middleware | Answers | Kind | Human / bot | Size | Status |
  |---|---|---|---|---|---:|---|
  | 1 | `ManaWindowStop` | `ManaAbilityWindow`, decline once covered | policy — declining forecloses CR 605.3a's float | toggle (`--no-auto-pay` today) / on by default | 0 | built, CM-4 |
  | 2 | `AutoPayer` | `OrderCostReductions` (0 / 0.07 / 0.02 a game) | an elision the engine owes (rule 1, item 47's shape) | both, no toggle | −50 this PR; the decorator goes next | built, CM-4; **forced split retired here, the rest moves into the engine in the follow-up PR** |
  | 3 | full control | nothing — a switch above the stack: raw or decorated | the toggle | human only | ~100 + tests | re-derived at review, below |
  | 4 | auto-yield | `PriorityAction` → `Pass` while a yield holds | policy | human only; never a bot's | ~100–150 + tests | **one PR with row 3** |
  | 5 | combat defaults | `AssignCombatDamage` (2.1 / 2.3 / 3.7), `AssignTrampleDamage` (0.17 / 0.28 / 0.40) | policy | human under the toggle / the agent's own | ~60 + a CR read | item 84's helpers are its body; after 3 |
  | 6 | tap solver, oracle half | nothing — a query: a covering set for `remaining_cost` | an oracle, not a decorator | both, as a query | ~150–250, plain case | §2.18; two customers; **the one row with algorithmic legwork**, below |
  | 7 | tap solver, decorator | `ManaAbilityWindow`, *pick* while uncovered (194 / 332 / 476); `GenericManaAllocation` with surplus (41 / 88 / 136) | policy | human under the toggle / a flag, off by default | ~60 | after 3 and 6 |
  | 8 | auto-order triggers | CR 603.3b's order, once it exists | engine elision for identical triggers; policy for the rest | human under the toggle / the agent's | ~40 + ~30 engine | with critical-path item 6; classified at birth below |
  | 9 | reversal policy | CR 732.1's offer, keep or reverse all (item 72) | policy | human under the toggle / a bot policy — never silent on a human's | ~15 in row 7; the prompt ~40 in the engine | with item 6 |

  **Row 2, decided: the payer keeps no answer the engine stopped asking for.**
  `AutoPayer::allocate`'s forced branch and `auto_payer::split_is_forced`
  duplicated `ui::ask::forced_allocation` one layer up and were unreachable
  from a game since A4e; keeping them would have covered a client that drives
  the provider by another route — the raw-action-space harness is such a
  client — and rule 1 says that client should get the engine's answer too.
  Retired: the branch, its predicate, its bucket walk and six tests out, one
  test in (a forced split reaches the wrapped provider), about fifty lines
  net. What the payer keeps beyond that is the *absence* of an answer to a
  split with surplus, which is the `{2}{U}` board's `{U}{U}` still up for
  Counterspell. **And at review (the owner, 2026-09-18) the rest goes too.**
  The ordering answer is rule 1's second case: by §3.4's theorem every order
  gives one total, so the engine elides the prompt the way item 47 elides CR
  616.1's — one guard at the call site in `cost_determination/total.rs`,
  false the day either expiry condition lands (a reduction whose amount is a
  hybrid symbol, CR 118.7e; a `not_below` reduction), ~20 lines;
  ATOM-601.2f-004's test restated from "the prompt is asked" to "both
  reductions apply and every order gives the CR's answer"; `AutoPayer` deleted
  and `cli_play`'s human stack becomes `ManaWindowStop(Cli)`. It moves the
  random agent's stream by the 0.07 prompts a game the stress pool asks, so it
  is a `differ` A/B and its own PR, first in the sequence below. §3.4's "take
  it then, not now" was a timing call, and this is the time: the prompt is
  kept alive today by a module whose one job is to answer it.

  **Row 3, full control, re-derived at review (2026-09-18; sized 2026-09-08 at
  ~200 lines as a handle, now ~100 as a switch).** The reason it is small is
  worth keeping: **the engine asks the provider at every priority point** —
  `candidate_priority_actions` always offers `Pass`, so
  `ask_choose_priority_action` is always reached (`engine/priority.rs`). The
  toggle needs no keystroke listener beside the engine; it is one more command
  at a prompt the player already sits at, Arena's granularity. **What full
  control is:** the raw provider, for as long as it is on — the supplemental
  doc's `RawDP`, entered and left mid-game. The engine holds one
  `&dyn DecisionProvider` for the whole game, so the stack cannot be rebuilt
  mid-game without an indirection, and the census's first draft put that
  indirection inside every decorator as a handle each one checked. **The
  owner's shape is better:** one `FullControl<D, R>` at the top of the seat
  holding the decorated stack `D` and the raw provider `R`, forwarding each of
  the four methods to one or the other on a `Cell<bool>` (~40);
  `CliDecisionProvider` intercepting the command before it parses an index —
  `pick_n`'s `read_usize` loop is the one place (~45); wiring in `cli_play`
  (~10). The decorators stay stateless and know nothing of the toggle, so
  rule 3 needs no toggle arm and a new decorator adds nothing here. One
  wrinkle: `R` and the bottom of `D` must be the same human provider. The
  CLI's is a unit struct, so two instances are fine; a GUI provider holding a
  channel wants `Rc<P>` and a forwarding `impl DecisionProvider for Rc<P>`
  (~10 lines), which is the framework's answer for any client whose provider
  has state. It is client state rather than `GameState` — item 40's test is
  "drop it and re-derive", and a game replayed with the switch off asks the
  human what the decorators answered, reaching the same game from the same
  answers.

  **Row 4, auto-yield, and why it ships with row 3 and never before it.**
  `AutoYield<D>` answers `PriorityAction` with `Pass` while a yield condition
  holds — Arena's three: until end of turn, until the stack changes (a
  response is wanted), until the player's next turn — reading the step, the
  active player, the turn and the stack's top from the `&GameState` every
  prompt already carries. ~100–150 lines with the condition type, plus the
  CLI command through row 3's intercept. **Human only**: an agent's non-forced
  pass is a decision, and a rule that makes it is the agent, not middleware.
  **The counterweight** (the owner, review, 2026-09-08): a human whose turn
  fast-forwards has told the table they hold nothing at instant speed. That
  tell does not exist today — nothing auto-passes, so every human turn looks
  the same — and it arrives with this row, which is why the toggle that puts
  the prompts back lands in the same PR and why tying full control to the GUI
  instead would have let the harness ship auto-yield first. **What auto-yield
  does not do** is skip the `[Pass]`-only prompt; that is the engine's, and
  (a) below sizes it. **Replay caveat, designed for rather than retrofitted:**
  a yield set mid-game makes a CLI game non-replayable unless the command is
  in the recorded input stream — cheap now, expensive once a replay format
  exists; it binds rows 3 and 4 alike, and any client setting a decorator
  reads.

  **Row 5, combat defaults.** Item 84's two callerless helpers,
  `default_damage_assignment` and `default_trample_assignment`, are the body
  of a `CombatDefaults<D>` that answers the two division prompts the way
  Arena's default does — lethal to each blocker in order, the remainder to the
  last blocker or, with trample, to the player. Policy, because CR 510.1c and
  702.19b make the division the controller's; human under the toggle; a bot's
  own agent divides. ~60 lines with the forwarding, **after one CR read the
  helpers owe before they become a body**: the trample helper's deathtouch
  branch counts a blocker with damage already marked as needing nothing,
  and CR 702.2c says any *nonzero* amount is lethal — such a blocker still
  needs one. Two prompts a game; after row 3, and it may ride in row 3's PR
  if the read is clean. Item 84 is sized here now.

  **Rows 6 and 7, the tap solver, framed.** §2.18's oracle half is a
  **matching**: pips on one side, the mana abilities `available_mana_sources`
  offers on the other, an edge where the ability makes a type the pip accepts
  — hybrid pips take either half, an any-color source (§2.19) reaches every
  pip, and Phyrexian or mono-hybrid pips are payable without their color, so
  which half to pay is a preference and not an edge. `remaining_cost_after_pool`
  is the left side already; ~150–250 lines in `oracle/` with tests, and it
  answers "is there a covering set, and here is one". **It has two customers,
  and the second is the larger.** The decorator (row 7) picks in CR 601.2g's
  window while the component is uncovered — disjoint from the stop's
  predicate, so rule 3 holds — and removes most of the 194 window prompts a
  game at four seats, 60% of all inner prompts (item 138); ~60 lines, human
  under the toggle, and on a bot's seat **a `fuzz_games` flag that is off by
  default**, because `RandomDecisionProvider`'s tap preference is a measured
  policy and a solver answering for it moves every counter for reasons that
  are not the engine's (`build_stack`'s own comment). The other customer is
  **`castable_spells`' affordability**, which is a heuristic overapproximation
  today (the supplemental doc's §3, `find_mana_sources`): the engine offers a
  cast the pool cannot cover and rewinds it under CR 732.1, the path A4h had
  to make state-dependent (item 139) and the enumeration item 138's lever 4
  prices at 19.2% of instructions. The matching makes the offer exact. That
  customer is engine-side, moves the random agent's stream — fewer offered
  casts are different games — and owes its own A/B with `differ` predicted on
  both pools; the harness prints no rewind count today, so the row's first
  job is the counter that reads it. **The Arena problem is the preference,
  not the matching**: when several covering sets exist, which lands stay
  untapped is a question about the rest of the hand and the opponents'
  boards, so the solver takes a preference from its client and breaks ties by
  it. **The census fixes the interface and not the preference's shape.** Part
  of the engine's purpose is that people write their own GUIs and hook them
  in (`CLAUDE.md`'s v1), so what the framework owes a client is *possibility*:
  any client-side expression of intent must map onto the matching's edges,
  and every edge is a (source, type) pair, so a preference stated over types
  does — the owner's example is a color wheel, a per-type spend-or-keep
  setting a GUI could draw, which maps as an order over edges by the type they
  produce. The example sets no default. A seat that supplies no preference
  gets the one policy that is measured, the random agent's least flexible
  source first, which took its land taps per cast from 7.66 to 3.18 (§2.18);
  a harness's learned preference is another client's expression of the same
  interface. **And the solver owns both payment prompts a client stacks it
  for**: the window's picks, and `GenericManaAllocation` when the pool has
  surplus (41 / 88 / 136 a game), because the same preference answers both
  and the second reaches a human as typed numbers today. Rule 3 holds — one
  owner, two prompts.

  **This is the one row with algorithmic legwork, and the tree's own
  heuristic shows where** (the owner's question at review, 2026-09-18).
  Every other decorator is a `matches!` and a rule; this one is a graph
  problem, and the size above is for its plainest shape.
  `available_mana_sources` records one `ManaSource` per (permanent, ability,
  type) and drops the amount, so Sol Ring's {C}{C} counts as one mana, and
  `find_mana_sources` returns `None` on any hybrid, Phyrexian or X symbol —
  the heuristic the matching replaces is a one-unit-per-source greedy that
  gives up on half the symbol alphabet. What the matching must carry: a
  producer of *k* units is a vertex of capacity *k*, which makes it a flow
  rather than a matching; a hybrid pip is a vertex with edges to two types;
  an any-color ability (§2.19, unbuilt) is a per-activation choice that
  reaches every pip; and a producer of "*k* mana of any one color" is *k*
  units that must agree, which no single flow expresses — enumerate its
  color when such a source is on the board, since there are few. **Out of
  the solver's scope and left to the window**: filter abilities that spend
  mana to make mana, which are a search over activation sequences rather
  than a graph — the chains `WINDOW_ACTIVATION_CAP` exists for. The
  preference makes it a weighted problem, and the pragmatic first cut is the
  shape the random agent already plays in ~60 lines of
  `mana_window_preference`: a greedy over edges in preference order with a
  feasibility check by augmenting path after each commitment, which is O(E)
  matchings over at most a few dozen sources. **Two things the row owes
  before its size is trusted.** A property test that every covering set the
  solver returns, under the split it chooses, is one `ManaPool::pay` accepts
  — a solver that says "covered" and a payment that refuses is
  `codebase-state.md` 16c's rewind in a new place, and owning both prompts
  (above) is what makes the two computations one. And a re-derivation of
  the ~150–250 against a written algorithm, since that number is the plain
  bipartite case with chains excluded. Nobody writes a solver here; this is
  its frame. Row 6 may land any time as an oracle PR; row 7 after rows 3
  and 6.

  **Row 8, classified at birth, since the prompt does not exist.** CR 603.3b's
  ordering is asked of each trigger's controller in APNAP order as the
  abilities go on the stack; in the classification below it is a **C** row
  and part of the **residual** — asked mid-step of a seat that did not act.
  Two halves, and the first is the engine's by rule 1: two triggers that are
  copies of one ability under one controller with no targets give the same
  game in either order, and the engine declines to ask — *measured first*,
  then elided with expiry conditions, which is item 47's precedent, ~30
  lines. The rest is policy — a human under the toggle gets timestamp order,
  a bot's agent orders — ~40 lines of decorator, sized inside item 6's doc.

  **Row 9, the reversal (item 72), decided 2026-09-18 and re-derived at review
  — two options, and only two.** Is a reversal even possible in an engine
  that performs every mutation through the chokepoint with replacements
  applied? *Reverse all* is, and cheaply: `GameState` is `Clone` and a no-log
  clone is 4–6 µs (item 42's measurement, `codebase-state.md`), so the engine
  takes one at the window's first activation and, if the player takes CR
  732.1's offer at the rewind, restores it, truncating `events` to the
  snapshot's length. That is the rule's "no abilities trigger and no effects
  apply as a result of an undone action" by construction — the clone predates
  the tap, the mana, any replacement that applied to it and any trigger it
  queued, which item 40 keeps on `GameState` — and the RNG rewinds with it,
  which is the right reading. Two things stay live across the restore: the
  retry loop's locals, which are about the offer list and sit outside
  `GameState` on purpose (item 140), and the `Diagnostics` cells, which
  counted work that happened. *Reverse some* is not possible: the tap, the
  mana and a sacrificed Ironworks artifact all went through the chokepoint as
  performed events, and the engine has no per-event undo — selective
  reversal is an undo log or a re-simulation of the window's activations from
  the snapshot minus the reversed ones, a facility and not thirty lines, and
  Arena offers undo-all only. So the prompt is *keep* or *reverse all*, ~40
  lines in the engine, with item 6 as CM-4 placed it. **A decorator may
  answer it only under the same toggle as auto-yield.** On a human's seat it
  is a prompt-skipping automation — *reverse all* when the solver made the
  taps, since the taps a solver made are the ones it should unmake — and it
  lives inside row 7; on a bot's seat it is a plain policy (`fuzz_games`:
  keep, which is today's stream); **never silently on a human's**. Item 72's
  placement with item 6 does not move, and the invariant it states — the
  ability's cost and its mana undone together — is what the restore gives for
  free.

- **Not rows, named here so nobody files them as one.**
  - **(a) The `[Pass]`-only priority prompt is the engine's**, by rule 1, and
    it is the largest forced prompt left: 91.5% of priority prompts at four
    seats offer `Pass` alone (item 138), over two thousand round trips a game
    for an answer the engine has. `Priority decisions` already excludes them
    and the random agent draws nothing on a one-option `pick_n` (item 145,
    lever 10), so skipping the prompt would move no counter and no stream;
    what it costs is the fixture migration — **148 scripted
    `ChoiceKind::PriorityAction` expectations, 134 in ten test files and 14
    in `src` unit tests**, an upper bound since some answer a longer list —
    against ~10 lines at `run_priority_round`. Item 145's class, owed for
    item 145's reason; `codebase-state.md` item 164.
  - **(b) "Why can't I?"** — `engine::restriction::predicate::is_prohibited`
    is `pub(crate)` and returns a `bool`, so a GUI cannot glow the permanent
    that forbids an attack and an observation cannot name it. An oracle
    question, filed as **§2.35** from A4j's review; a decorator answers
    prompts, and this is a query.
  - **(c) Determinization** — a search over a cloned `GameState` sees its next
    draw; **§2.34**, the harness's, filed from the same review.
  - **(d) Auto-sacrifice** — struck from the stack. No v1 client wants
    `ChooseSacrificeForCost` answered by a rule: a human chooses, an agent
    decides, and a harness that wants a rule writes it into its own provider.
  - **(e) Staged payment** — a client buffering its answers until the player
    confirms, which CM-3's deciding-before-performing already allows; not a
    decorator. §2.18 owns it.
- **Sequence.** (1) **This PR**: row 2's forced branch. (2) **Row 2's
  remainder, alone**: the ordering elided in the engine and `AutoPayer`
  deleted, ~20 lines and a restated test, its own `differ` A/B — the one
  engine change the census owes, small and stream-moving, so it travels by
  itself (`codebase-state.md` item 165). (3) **Rows 3 and 4 in one PR**,
  ~250–350 lines with tests — the toggle's first customer is auto-yield and
  auto-yield's counterweight is the toggle, so neither ships alone
  (`engineering-practices.md` §4's consumer rule, applied to a toggle). Any
  time; before the GUI; commutes with A6, and its A/B is `IDENTICAL` by
  construction since `fuzz_games` stacks neither. (4) **Row 5**, after or
  inside (3). (5) **Row 6** as its own oracle PR any time, its affordability
  customer with its own A/B; **row 7** after (3) and (5), read in the A/B as
  `Decisions` falling with the engine no faster — item 138's own warning.
  (6) **Rows 8 and 9 with critical-path item 6**, the engine's elision
  measured first. Every PR is one middleware and none is near §4's band.
- **Size** — the census was this sitting and one code change. The rows: 0,
  −50 then the module, ~100, ~100–150, ~60, ~150–250, ~60, ~70, ~40 + ~15.
- **Blocks** — the GUI's ergonomics (rows 3–5, 7); the harness's raw mode,
  which rule 1 defines (an undecorated provider is handed nothing the engine
  could answer); §2.18's solver, framed here as policy so nobody writes the
  matching to solve the preference; any client's own GUI, which the rows are
  shaped to make possible and never to prescribe.
- **Atoms** — none, and there will be none: the corpus is derived from the CR
  and the CR has nothing to say about interfaces (§2.21 says the same).
- **Owner** — this entry, until each row graduates to its PR; **sequenced
  ahead of full control** by the owner (2026-09-08) so that full control is
  scheduled against the whole stack rather than against the one decorator
  that happened to need it. **Reviewed by the owner on PR #169 (2026-09-18)**,
  with four amendments folded in above: rule 2's second kind struck and the
  payer scheduled into the engine, full control a switch above the stack, the
  solver's preference the client's in the client's own shape and both payment
  prompts its, the reversal two options by a clone. Row A4k in
  `roadmap-v2.md` §3a.
- **The 24 asks, classified for the fork model** (pass 3 of
  the post-RE audit, 2026-09-15 — the handoff's §4 consequence 2 and §6
  decision 4; the handoff is deleted, its last text `git show
  341ebf9:plans/handoffs/post-re-audit.md`, its record `codebase-state.md`'s
  "Was critical-path item 5 done, and what sits before item 6? — audited 2026-09-15"). The owner's rule: an inner ask is either a *parameter of
  the action chosen at priority* or *answered by a policy the harness
  supplies*, never a separate observation. Read against `ui/ask.rs`'s 24
  functions — the whole decision surface, 27 engine call sites and none in
  `tests/` on 2026-09-15, 26 on 2026-09-18 once A4i asked recipients from one
  site (the count above) — three classes come out, **B** (a boundary of its own: the seat
  is asked to act), **P** (a parameter of the action chosen at priority) and
  **C** (answered as an effect resolves, by a policy or the agent), plus the
  residual §4 flags. The last column is prompts per game at four seats,
  `performance` / `stress` / Commander-scale `stress` (four 100-card decks,
  40 life), 200 / 200 / 100 games at seed 12345, from a throwaway counting
  provider — a pure function of the seed, like every fixture row:

  | `ask_*` | `ChoiceKind` | asked of | class | per game |
  |---|---|---|---|---|
  | `choose_priority_action` | `PriorityAction` | the seat with priority | **B**, the boundary — but **188 / 314 / 476** of these offer more than `Pass`; the rest are forced and a harness skips them unobserved | 2,219 / 2,505 / 3,502 |
  | `choose_attackers` | `DeclareAttackers` | the active player | **B** — CR 508.1's turn-based action, a decision of its own and never a parameter | 34 / 32 / 45 |
  | `choose_blockers` | `DeclareBlockers` | the defending player | **B** — and it lands on a seat other than the active player by construction | 15 / 14 / 19 |
  | `choose_attacker_damage_assignment` | `AssignCombatDamage` | the attacker's controller | **B** or **C** — strategy at the margin; `default_damage_assignment` is the policy | 2.1 / 2.3 / 3.7 |
  | `choose_trample_damage_assignment` | `AssignTrampleDamage` | same | **B** or **C** — `default_trample_assignment` | 0.17 / 0.28 / 0.40 |
  | `choose_x_value` | `ChooseXValue` | the caster | **P** | 0 / 0 / 0 — no pooled X spell |
  | `choose_alternative_cost` | `ChooseAlternativeCost` | the caster | **P** | 0 / 0 / 0 — none registered |
  | `choose_additional_costs` | `ChooseAdditionalCosts` | the caster | **P** | 0 / 0 / 0 — none registered |
  | `select_recipients`, at CR 601.2c | `SelectRecipients` | the caster | **P** — the one parameter that must reach the agent: targets are strategy | 30 / 26 / 36, both sites |
  | `select_recipients`, at resolution | `SelectRecipients` | the controller | **C** or the agent's — a non-targeting "choose" as the spell resolves | in the row above |
  | `activate_mana_ability` | `ManaAbilityWindow` | the payer | **P** by policy — the tap solver §2.18 still owes; `ManaWindowStop` is the half built | 194 / 332 / 476 |
  | `choose_generic_mana_allocation` | `GenericManaAllocation` | the payer | **P** — the engine when forced (A4e, item 145); otherwise the agent's, or §2.22 row 7's solver as policy | 41 / 88 / 136 |
  | `order_cost_reductions` | `OrderCostReductions` | the payer | **P** by policy — `AutoPayer`, always (`cost-architecture.md` §3.4) | 0 / 0.07 / 0.02 |
  | `choose_sacrifice_for_cost` | `ChooseSacrificeForCost` | the payer | **P** — strategy; the agent's, or the auto-sacrifice row above | 0.22 / 0.38 / 0.62 |
  | `commander_to_command_zone` | `CommanderToCommandZoneSba` | the commander's owner | **C**, and **residual** — an SBA's question to whichever seat owns the commander, mid-batch | 0 / 0 / 0 — `fuzz_games` seats no commander |
  | `discard` | `Discard` | the affected player | **C** for CR 514.1's cleanup (the active player); **residual** for "target player discards" — Mind Rot's target chooses | 3.6 / 4.0 / 8.6 |
  | `scry` | `Scry` | the controller | **C** or the agent's | 0.84 / 0.44 / 0.36 |
  | `scry`, the ordering | `ScryOrder` | the controller | **C** | 0 / 0 / 0 — every registered scry is scry 1 |
  | `choose_replacement` | `ChooseReplacementEffect` | the affected object's controller, or the affected player (CR 616.1) | **C**, and the **residual** §4 names — it lands on whoever is affected, mid-resolution, and often not on the actor | 1.8 / 4.0 / 12.0 |
  | `apply_optional_replacement` | `ApplyOptionalReplacement` | same (CR 614.1a's "you may") | **C**, residual | 0 / 0.01 / 0.09 |
  | `allocate_next_damage` | `AllocateNextDamage` | the affected player (CR 615.7) | **C**, residual | 0.01 / 0.04 / 0.04 |
  | `choose_entering_controller` | `ChooseEnteringController` | the caster (CR 616.1b) | **P**-shaped but asked at entry, after the action — a policy, or the agent | 0 / 0 / 0 — Xantcha unpooled |
  | `choose_auxiliary_zone_change` | `ChooseAuxiliaryZoneChange` | the entering permanent's controller (CR 614.13a) | **C** or the agent's — devour's count is strategy | 0.49 / 0.33 / 0.48 |
  | `choose_copy_source` | `ChooseCopySource` | the controller (CR 707.4) | **C** or the agent's | 0.49 / 0.22 / 0.32 |
  | `choose_damage_source` | `ChooseDamageSource` | the controller (CR 609.7a) | **C** or the agent's | 0 / 16 / 37 |
  | `choose_legend_to_keep` | `LegendRule` | the controller of the duplicates | **C**, **residual** when the duplicates are an opponent's — an SBA's question, mid-batch | 0.09 / 0.13 / 0.06 |

  **Every number in the last column is the stream as it was on 2026-09-15, and
  A4e moved it** (`codebase-state.md` items 138 and 145, 2026-09-16). The
  counters reproduced this table exactly on the commit before the guard, which
  is what says the census counted the right sites; then the guard stopped three
  asks prompting when the answer is forced — `select_recipients`,
  `choose_generic_mana_allocation` and the trample split — and a skipped prompt
  is a skipped RNG draw, so *all* of these counts are now lower and the games
  behind them are different games. The classification is unaffected: a forced
  prompt was never a decision for the fork model either. The Commander column's
  board was a source patch when this was taken and is `--deck-size 100 --life
  40` now.

  **Three things to read off it.** (1) **Six of the 24 are never reached by
  the fuzz harness** — X, the two cost prompts, the entering controller, the
  commander SBA and the scry ordering — because no pooled card produces
  them; their rows come from the rules, not the census. (2) **The action
  space is not "complete legal actions, sub-choices resolved" today**:
  `PriorityAction` is four variants with no sub-choice, and every **P** row
  is a separate ask made *after* the action is chosen, inside `cast_spell`
  (CR 601.2b–h), where a rejected choice rewinds the cast
  (`codebase-state.md` item 40's second violator). A harness that wants the
  sub-choices in the action either enumerates the product — cast ×
  alternative × additional × X × targets × payment, which only the tap
  solver bounds — or presents the P rows as a hierarchical action, one
  prompt at a time from the same fork; the second is what the record-and-
  replay shape of the fork test (main item 41) gives for free, and it is
  what every published Magic agent does. (3) **The residual is small and it
  is named**: the rows marked *residual* land on a seat other than the actor
  mid-resolution or mid-SBA — CR 616.1's affected player, CR 615.7's, a
  targeted discard, a legend rule or a commander SBA on somebody else's
  permanent — and the census counts them at **about 2 / 3 / 7 per game** at
  four seats, under 1% of the prompts with a choice. What each way costs: as
  a *policy*, the other seat's provider answers inside the acting seat's
  engine step, which `DispatchDecisionProvider` already does in-process and
  which an out-of-process harness pays as a second round trip mid-step; as an
  *observation*, the batched boundary's return type becomes "the next prompt
  with a choice, whichever seat it is for" — which subsumes the **B** rows
  too, and is the shape Phase 10's harness doc starts from. CR 603.3b's
  ordering joins the residual the day triggers exist, asked of each trigger's
  controller in APNAP order — classified **C**, residual, at birth (row 8 of
  the census above), before the `ChoiceKind` exists.

  **Per class, the cost of each way.** A **B** row is an observation and
  costs one prompt's encoding. A **P** row answered by policy costs nothing
  outside the engine — two of the decorators exist — and a **P** row
  surfaced to the agent costs one more observation per parameter, from the
  same fork. A **C** row answered by policy is free in-process and a
  mid-step round trip out of it; surfaced, it is an observation for a seat
  that did not act.

### 2.27 The token vocabulary — a token cannot have an ability — type half ✅ landed 2026-09-13 (RE-4)

- **Rules** — CR 111.10a–v (the twenty predefined token types), CR 111.11
  (a token created by name), CR 111.4 (a token has the characteristics the
  effect that created it says), CR 701.16a (investigate = create a Clue).
- **Verdict** — `TokenDef` (`types/effects.rs`) has **seven** fields — name,
  colors, types, subtypes, power, toughness, keyword flags — and
  `resolve.rs::token_card_data` lowers it into a `CardData` that has
  **seventeen**. Six of the ten it cannot fill are right to be absent:
  `mana_cost` (CR 111.6 gives a token none), `alternative_costs` and
  `additional_costs` (cast-time, and a token is never cast), `color_indicator`
  (a token's colors are stated outright, so nothing is derived), `loyalty` (no
  printed card creates a planeswalker token — measured, zero) and `defense`
  (battles, `backlog.md` §2.23). **Four are the gap: `abilities`,
  `supertypes`, `rules_text` and `enchant_filter`.**

  **So every one of CR 111.10's twenty predefined tokens is inexpressible, and
  the rule's own text is why**: each of the twenty is *defined by an ability*.
  Treasure, Food, Gold, Clue, Blood, Powerstone, Map, Junk, Lander, Mutagen and
  Shard are an activated ability apiece; the six Roles are Auras — `enchant
  creature` plus a static grant, and Wicked Role's is a *triggered* ability;
  Incubator is a double-faced token with `{2}: Transform this token`; only
  Walker (a 2/2 black Zombie named Walker) is expressible, and it is the one
  of the twenty with no ability at all.

  `token_card_data` does not drop abilities by oversight — there is no field to
  read. This is the same failure shape as `backlog.md` §2.19's: **a `Vec` whose
  element type cannot say the thing**, found by counting what the lowering
  writes against what the target type holds.
- **Size** — small for the vocabulary, a phase for the library. The type change
  is ~6 fields plus ~20 lines of `token_card_data`, and a `cards::tokens` module
  holding CR 111.10's twenty is one constructor each — but eleven of the twenty
  need `Primitive::Sacrifice` as a *cost* (CM-3 shipped it), Treasure and Gold
  need **§2.19's any-color mana** and are blocked on it, Wicked Role needs
  CR 603 and is blocked on critical-path item 6, and the Roles need an Aura
  token to attach on creation. So: **the type in RE-4's PR, the library in a
  phase of its own**, and the library graduates entry by entry rather than all
  at once. CR 111.11's by-name lookup is a separate ~40 lines against
  `CardRegistry` and wants the information model (§2.9) before it can reveal
  what it made.
- **Blocks** — measured on Scryfall 2026-09-13, `unique=cards`: **3,582** cards
  create a token at all, and **1,085 of them create one this vocabulary cannot
  express** — 749 that name a CR 111.10 type (Treasure 375, Food 155,
  Powerstone 44, Blood 43, Role 39, Clue 28, Map 13, Incubator 7, and the
  rest), 211 that quote an ability inline (`create … token with "…"`), and 138
  more that say **investigate** without ever printing the word Clue. A further
  **55** create a *legendary* token, which is the missing `supertypes` field
  rather than the missing abilities, and **80** printed tokens are double-faced
  (CV-5's `back_face`, not this entry's). Copy-shaped tokens — "create a token
  that's a copy of", 270, plus amass — are **CV-3's** and not here.

  **The retrofit argument, which is why this is not simply Phase 8 breadth.**
  A token with no abilities is not a token that is missing something; it is a
  token the engine believes has none, and nothing fails. Every card written
  against the current vocabulary is written *around* it — Academy Manufactor is
  already recorded as uncastable for exactly this reason
  (`replacement-architecture.md`, "Out of RE") — and Phase 8 is 643 atoms of
  writing cards. The same back-stop argument CV-7 won on applies with a larger
  population: **before Phase 8 card breadth**.
- **Atoms** — `ATOM-111.10-001` (Treasure) and `-002` (Food), both tagged
  Phase 8 and uncovered; `ATOM-111.11-001` (token by name). CR 111.4's
  characteristics atoms are covered where they are. The twenty types do not
  each owe an atom — CR 111.10 is one rule with twenty rows, and the corpus
  files it as `BOUNDARY-DEF` with two examples, which is the right granularity.
- **Owner** — **the type half landed with RE-4 (2026-09-13)**: `TokenDef`
  carries `abilities`, `supertypes`, `rules_text` and `enchant_filter`, and
  two things the field count above had not seen — the name is an `Option`,
  since CR 111.4 names an unnamed token "[subtypes] Token" (Kalitas's Zombie
  is "Zombie Token"), and power and toughness are `Option`s, since CR 208.3
  gives a noncreature none and the lowering had been writing `Some(0)` onto
  one. `resolve.rs::token_card_data` is `TokenDef::card_data`, and every one
  of the twenty predefined tokens is now *expressible* as a def: a token that
  carries Master Biomancer's static ability is a test, Boo's supertype meets
  the legend rule, and a Role-shaped Aura def lowers with its enchant filter.
  **What is left is the library and the lookup**, both unowned: `cards::tokens`
  with CR 111.10's twenty as constructors — eleven need `Primitive::Sacrifice`
  as a cost (CM-3 shipped it), Treasure and Gold need §2.19's any-color mana,
  Wicked Role needs CR 603, and the Roles need an Aura token to attach on
  creation; and CR 111.11's by-name lookup, ~40 lines against `CardRegistry`,
  which wants the information model (§2.9) before it can reveal what it made.
  The library graduates entry by entry, Walker first, since it is the one that
  needs nothing.
- **Plan** (2026-09-13, the RE-4 review's question about Clue and Treasure) —
  **three steps, in this order.** (1) `cards::tokens`, one small PR: Walker,
  Clue, Food, Blood, Map, Junk, Lander, Mutagen, Shard and Powerstone as
  `TokenDef` constructors — every one needs only `Primitive::Sacrifice` as a
  cost, which CM-3 shipped — with `Primitive::Investigate` (138 printed
  "investigate" say Clue without the word) and one card per token type that
  makes it. (2) **§2.19, next on the mana side**: a Treasure def needs "add one
  mana of any color", and so does every Commander mana base; Treasure and
  Gold land with it, and so do Xorn, Chatterfang and Hullbreacher (RE-4's
  template has their shape and waits on the def). (3) The Roles with
  attach-on-creation, Wicked Role with CR 603, Incubator with CV-5's back
  face. The vocabulary for all twenty exists since RE-4; nothing in the
  library needs a type change.

---

### 2.28 Loops (CR 104.4b, 731) — a capture, not a design

- **Rules** — CR 104.4b (a loop of mandatory actions is a draw), CR 731.1–731.2
  (shortcuts: a player proposes a sequence of choices, the others accept or
  shorten it). The older design called it rule 727, its number before `tmnt`.
- **Verdict** — nothing detects a loop. What exists is two bounds that stop
  the *engine*: `check_state_based_actions_loop`'s cap on a state-based check
  that keeps performing, and `engine::actions::BATCH_NESTING_LIMIT`, a guard
  against a lost lineage (`replacement-architecture.md` §11 item 77). After
  RE-4 no replacement-only chain can loop — CR 614.5 bounds it once every
  nested batch carries its lineage — so the mandatory half is a **trigger**
  question and waits for critical-path item 6. Two halves, designed before
  under `roadmap.md` D11 and D26 (`GameNumber` with `Finite`/`Shortcut`/
  `Relative`, `LoopDeclaration`, `ask_declare_loop_count` through
  `pick_number`; "loop detection Tiers 1–3 survive, re-based on
  performed-action transcripts" — `archive/codebase-state-closed.md`, item 3):
  (1) a watcher over the performed stream keeping a buffer of the last N
  batches with a state hash each, flagging a repeated state with no player
  choice between occurrences — CR 104.4b's draw; (2) a player *declaring* a
  loop as a choice sequence plus an expected per-iteration delta, the engine
  running one iteration to verify the delta and applying it N times as one
  batch — CR 731.2's procedure, the piece no simulator has, and the AI
  harness's infinite-mana question.
- **Size** — unsized; (1) needs item 6 and a state hash that is a pure
  function of `GameState` (item 40's discipline); (2) needs a
  `DecisionProvider` surface (§2.22) and `GameNumber`.
- **Blocks** — every combo deck's win; `fuzz_games`' 200-turn limit stands in
  for both halves until then.
- **Owner** — none; raised at RE-4's review (R21).

### 2.29 The suppression predicate as a commutation table — ✅ graduated 2026-09-14 (RE-5's review, theme B)

*Built as this entry designed it, in `pipeline::ordering_cannot_change_outcome`:
a [`Commuting`] class per member — multiplicative, additive, mods-adding,
draw-doubling, idempotent substitute, absorbing exit — carrying the counter
kinds it touches, the shared clauses factored into `shared_clauses_hold`,
and a pairwise `commutes` table with the one board read the kind axis needs,
which kinds the entry's mods hold now. `classify` is exhaustive over
`Rewrite` and `AmountRewrite`, so a new arm is a compile error at the table.
The sixth shape — Divine Visitation beside Parallel Lives — is a cell, and so
are RE-5's additive pairs; a plus beside an `EnterWith` writing a new kind is
a real order and stays asked. `check_order_invariance` dispatches on the
chosen member's cell. Measured in the pool, the cells are rare — `Replacement
prompts` 2.14 → 2.12 per game at two seats — and what the pool still asks is
two Guardian Seraphs (`PreventUpTo` beside `PreventUpTo`, one outcome, the
table's next cell and RD's arm) and devour beside Master Biomancer, opaque by
design. The entry is kept as written for the record.*

- **Rules** — CR 616.1's choice among applicable replacement effects, and
  §11 item 19's rule that a choice with one outcome is not put to a player.
- **Verdict** — `pipeline::ordering_cannot_change_outcome` proves that rule
  five shapes at a time — all `EnterWith`, all multipliers, all draw doublers,
  one shared `Instead`, one exit beside `EnterWith`s — one shape per phase
  since RC-4, each a proof over a whole bucket with its own debug check. The
  organization they want is pairwise: a commutation class per `Rewrite` on an
  event kind (multiplicative, additive, absorbing exit, mods-adding,
  idempotent substitute) and a table of which classes commute, with the
  common clauses (static, rider-less, not optional, not a counter instance)
  factored out and one debug check per class. **The sixth shape is already on
  the board and is the trigger**: Divine Visitation beside Parallel Lives is
  asked today and has one outcome either way (a multiplier and a
  replace-by-"that many" commute; `phase_re4_integration_test` asks both
  ways), and RE-5's Hardened Scales beside Doubling Season is the pair the
  table states as *not* commuting.
- **Size** — ~150 lines, a refactor of a predicate the standing review
  question (`engineering-practices.md` §4.1) has corrected three times; wants
  a session of its own, at the sixth shape.
- **Blocks** — nothing today; a needless prompt per uncovered pair.
- **Owner** — none; raised at RE-4's review (R22). **RE-5 added no shape
  (2026-09-13) and put the pairs it makes reachable on this list instead.**
  Season beside Season is the multiplier bucket and asks nothing; Season
  beside Scales is a multiplier beside a plus, does not commute, and the
  prompt is Scales' own ruling. What is asked and has one outcome: Scales
  beside Scales (additive, `two_hardened_scales_add_two` asserts the
  prompt), and a plus beside an `EnterWith` of the **same** kind at an
  entry. **A plus beside an `EnterWith` of a *different* kind is a real
  order** — CR 614.5 gives the plus one opportunity, so a kind the
  `EnterWith` adds afterwards is not raised — which means the table's
  classes need the kind axis, not the rewrite alone; the sixth shape's
  session should build it that way. RE-5 also fired item 47's condition (c)
  from the multiplier side: a pattern arm that reads an entry's mods, so the
  multiplier clause asks `object_set_is_mods_invariant` of an entry's members
  now, the clause the `EnterWith` shape always asked.

---

### 2.30 Enters as an additional type (CR 614.1c, and a Layer 4 effect with no row)

**The surface that cannot express it.** Master Biomancer: "Each other
creature you control enters with a number of additional +1/+1 counters on it
equal to Master Biomancer's power **and as a Mutant in addition to its other
types**." `EnterMods` carries `tapped` and `counters` and nothing a type could
go in; and once the permanent is on the battlefield the type has to live
somewhere.

**Sized against the tree at the post-RE audit (2026-09-15), and it is one
PR, not the phase `codebase-state.md` main item 60 called it on 2026-09-03.**
Item 60 was written before RE-5, and its two fears are each answered by
something built since:

- *"A type on `EnterMods` breaks the mods-invariance that lets
  `ObjectFilter::ByType` be a frame-free check, so every CR 616.1 entry bucket
  would start prompting."* That is exactly what +1/+1 counters did to
  `ObjectFilter::PowerLE` at RE-5, and the answer was not to prompt every
  bucket: `pipeline::kinds_present` reads which counter kinds *this* entry's
  mods hold, and `commutes` asks per pair whether the kinds a member writes
  meet the kinds another reads. A type is the same shape one axis over —
  `filter_is_mods_invariant`'s `ByType`/`BySubtype`/`BySupertype` arms stop
  being unconditionally `true` and become "true unless this entry's mods add
  that type", read off the event the way `kinds_present` is. Containment
  Priest's `ByType(Creature)` beside Master Biomancer's Mutant does not ask;
  a `BySubtype(Mutant)` filter beside it does, and should. ~25 lines, and the
  match is exhaustive so the compiler names the arm.
- *"A Layer 4 effect that has no registry row, no source and no duration is a
  shape the layer system does not have."* It has it twice already: counters
  are state on `PermanentState` that the board pass reads at layers 6 and 7c
  with the entity's timestamp (`board.rs`, "the entering object's are the
  counters it would enter with"), and `Lookahead::new` builds the would-be
  entity from the pending mods the same way. An entered-as type is a third
  field of that kind — written once by `place_on_battlefield` from the mods,
  read at Layer 4 at the entity's timestamp so a later `SetSubtypes` with a
  later timestamp applies over it (CR 613.7), and carried into the look-ahead
  frame by the same constructor. No row, because it is not an effect with a
  source; the object *entered as* that type.

The rest is mechanical, and the tripwires are already built: `is_fixed`
destructures `EnterModsTemplate` in full, `merge` gains a union beside its
`|=`, and there are 8 `EnterMods` struct literals in `src/` (0 in tests) and
6 `EnterModsTemplate` literals. Scryfall lists one ruling for the card and it
is about the counters, so the type's duration is the CR's: the modification
is part of how the object entered (CR 614.1c), has no duration of its own,
and lasts while the object stays on the battlefield — the reading this entry
adopts, and the one fixture the PR should pin.

| Field | |
|---|---|
| **Rules** | CR 614.1c ("enters the battlefield as"), CR 613.1d's layer 4 and CR 613.7's timestamp for where the type lives afterwards |
| **Verdict** | `EnterModsTemplate` / `EnterMods` have no type field; `PermanentState` has no entered-as field for the board pass to read at Layer 4 |
| **Size** | ~150–200 lines, one PR: the two fields and their merge, one write in `place_on_battlefield`, one read in `board.rs`'s Layer 4 seeding, one in `Lookahead::new`, the three filter arms made mods-aware, Master Biomancer's clause registered, and three tests (the type is there, a `BySubtype` filter beside it asks, a later Layer 4 row wins) |
| **Blocks** | five printed cards say "enters … as a [type] in addition to its other types" (Scryfall, 2026-09-15: Master Biomancer, Eluge, the Shoreless Sea, Minas Morgul, Dark Fortress, Tarrian's Journal, Xolatoyac, the Smiling Flood); Master Biomancer is registered without the clause, in the stress pool, and wrong unobservably. Not to be confused with CR 707.9d's "in addition to its other types" on a *copy*, which is CV-2's exception path |
| **Atoms** | none filed; the corpus has no CR 614.1c atom for the type half — the PR files one |
| **Owner** | — (any sitting; nothing on the spine waits on it, and nothing stored today would need migrating, since no permanent carries a type it should not) |

### 2.31 Dice and coins (CR 705, 706)

**The surface that cannot express it.** Nothing rolls: there is no
`Primitive` for a die or a coin, no `GameAction` for the roll (so nothing a
"would roll … instead" replacement could watch — `replacement-architecture.md`
§8a's fourth missing kind), and no reader of `GameState.rng` outside
shuffling and the random provider. CR 706.2's re-roll replacements and
CR 706.3's "roll again" are the replacement side; CR 705's coins are the same
shape with two outcomes.

| Field | |
|---|---|
| **Rules** | CR 705.1–705.5 (coins), CR 706.1–706.6 (dice), CR 706.2's replacement ("would roll … instead") |
| **Verdict** | no primitive, no event, no result carried to the effect that rolled; `AmountExpr` has no "the result" reading |
| **Size** | one RE-shaped kind — a `GameAction::RollDice { player, sides, count }` family, its `EventPattern` arm, a performer drawing from `GameState.rng` (never ambient, `CLAUDE.md`), an `AmountExpr` for the result — ~300–400 lines with two cards, on RE's per-kind measure; coins fold in as `sides: 2` |
| **Blocks** | ~84 printed cards roll a die outside Un-sets (Scryfall, 2026-09-15: the AFR and CLB Dragons, Barbarian Class, Pixie Guide, Wyll, Blade of Frontiers, …); seven of them replace the roll |
| **Atoms** | the CR 705/706 atoms are Phase 8's and Phase 9's in the corpus |
| **Owner** | — |

### 2.32 Mulligans (CR 103.5)

**The surface that cannot express it.** `Game::setup` deals every opening
hand and asks nobody. `GameConfig::mulligan_rule` exists — `London`, `Paris`,
`None` — and nothing reads it, so every player keeps their first seven in
every game, the four-seat Commander games v1 is for included. Nothing else
is missing: the ask is two `pick_n`s per player, keep-or-mulligan and then
which N cards go to the bottom. Filed 2026-09-15, when the post-RE audit's
comment sweep found the `TODO` in `Game::setup` with no owner.

| Field | |
|---|---|
| **Rules** | CR 103.5 (declarations in turn order from the starting player, then each mulligan taken, then N cards to the bottom); 103.5c (in a multiplayer game the first mulligan is free); 103.5b ("any time [that player] could mulligan"); 903.5a — Commander is the same rule |
| **Verdict** | a stub whose configuration is already in place; the engine answers the game's first decision for the player |
| **Size** | ~100 lines in `Game::setup`: two `ChoiceKind`s, the loop CR 103.5 states, 103.5c keyed on the player count, and the random provider's policy — keep, always, so no fuzz counter moves; the pool moves only if an agent ever mulligans |
| **Blocks** | every game's opening: the RL harness's first decision (the post-RE audit's fork model, `codebase-state.md` main item 41) and `cli_play`'s human seat |
| **Atoms** | `ATOM-103.5-001` |
| **Owner** | — |

### 2.33 Choice versus target — when a non-targeting selection is made (CR 601.2b, 608.2c)

**The surface that cannot express it.** `EffectRecipient::Choose` is the
non-targeting selection — CR 303.4a's Aura put onto the battlefield without being
cast, "sacrifice a creature of your choice", the player or creature type a spell
names without targeting. `announce_targets` asks it **at cast time**, in the same
match arm as `Target` (`put_on_stack.rs:302`), and `Effect::instances` collects it
as an instance of "target". The CR makes a plain "choose"
at **resolution** (CR 608.2c); only a clause that says so is announced early
(CR 601.2b). One enum arm is doing two rules' work and nothing on it says which.

**Neither half is A4i's doing, and one of them is invisible.** The code A4i
replaced already matched `Target(..) | Choose(..)` in one arm, so the timing
predates the instance model. And **no registered card constructs a `Choose`** —
its one construction site is `resolve.rs:1807`, inside the sacrifice-of-choice
path, built at resolution and never announced — so the `Choose` leg of the
instance model, collected by `Effect::instances`, announced by `announce_targets`
and exempted from CR 608.2b's re-check by `surviving_targets`, is exercised by no
card at all. The wrong timing and the dead leg are the same fact.

**The Black Gate is the card that separates them** — *"Choose a player with the
most life or tied for most life. Target creature can't be blocked by creatures
that player controls this turn"* — one ability carrying both, at two different
times.

**What the survey has to produce**, and it is a read of the CR before it is a
design: which rules put a choice at announcement and which at resolution; which of
those the engine can express today; and whether `Choose` belongs in the instance
list at all, or is a resolution-time selection that never had an instance and
should leave it. Filed 2026-09-18 out of A4i's review, theme E (2026-09-17), which
found both halves and fixed neither: there is no survey of "choose" across the CR
or the pool, and there should be.

| Field | |
|---|---|
| **Rules** | CR 601.2b (modes, and the other choices made "as you cast"), 601.2c (targets, announced per instance), 608.2c (a resolving spell's instructions, in the order written — where a plain "choose" is made), 303.4a/303.4c (an Aura *spell* targets; an Aura put onto the battlefield without being cast does not), 700.2 (modal, which 601.2b sends first) |
| **Verdict** | `EffectRecipient::Choose` carries a filter and a count and no timing, and `announce_targets` reads it in `Target`'s arm; `Effect::instances` makes it an instance. A card whose "choose" is a resolution choice cannot say so |
| **Size** | the survey is a sitting. The change after it is small if `Choose` leaves the instance list — one arm and its readers — and medium if it gains a timing axis: a second recipient kind, `announce_targets` filtering on it, and `resolve_effect` asking the rest |
| **Blocks** | nothing today — zero cards. The first card with a non-targeting choice, and the Black Gate class that carries both timings in one ability. **And A6**: CR 603.3d hands `announce_targets` its clauses, so a triggered ability with a "choose" clause inherits whatever this decides. Pool scale (Scryfall, `game:paper`, 2026-09-18): `o:"choose a player"` 17, `o:"choose a creature type"` 91, `o:"as you cast this spell"` 37 |
| **Atoms** | none filed under `Backlog`. CR 601.2b's seven atoms are cost and mode announcement and `ATOM-608.2c-001` is instruction order, so the question this entry asks has no atom — the survey's first output is whether it needs one |
| **Owner** | — |
### 2.34 A search over a cloned `GameState` sees its next draw — determinization, and what §2.9 does not do

**The surface that cannot express it.** Phase 10's AI harness is search: clone
the game at a priority prompt, try a line, resume, compare. The fork exists —
`Game::resume_turn_at_priority`, `tests/priority_fork_test.rs`,
`codebase-state.md` items 41 and 140 — and it clones everything: every library
in its shuffled order, every hand, every face-down card. A search that reads
the clone knows what it will draw next turn and what its opponents are
holding, and a look-ahead that simulates to its next draw is not looking
ahead, it is looking at the answer.

**§2.9 cannot prevent it, as planned.** §2.9 is a *query* — "may player N see
this object?" — consumed by a renderer or an observation builder that asks
before showing. A search does not ask; it holds the state. Item 41 says the
re-randomization is "§2.9's per-viewer question", and that is half of it: the
query decides *which* cards are unknown, but nothing in §2.9 builds a state in
which they are. That is a second facility — determinization, in the
information-set-search sense: before each branch, rebuild the clone from the
searcher's information set — shuffle the unknown portion of every library,
redeal each opponent's unknown hand cards from that opponent's unknown library
cards, leave every card the searcher knows where it is — and search over
several such samples rather than the true state. Both halves are needed. With
the query and no redeal the search is omniscient; with a redeal and no query
the redeal cannot tell a scried top card from an unknown one.

**Knowledge is more than zone visibility.** CR 400.2's public/hidden split is
per zone; what a player *knows* is per card and per player: the top card after
a scry (CR 701.22a), a card revealed (CR 701.20), a card looked at, an
opponent's hand seen through a discard spell, a face-down permanent's identity
known to its controller alone. The redeal must keep every known card in place
and the query must answer "known", not "in a public zone". So the information
model needs a per-viewer *knowledge* record that events write — a reveal, a
look, a scry — rather than a predicate over zones alone; §2.9's "reveal versus
look at" line is this in one sentence.

**And the opponents' providers in a rollout.** A branch plays the other seats
too, with some provider. Handed the true clone, the opponent model plays with
the searcher's knowledge of the searcher's hand; handed the determinized
clone, it plays with the searcher's uncertainty about its own hand, which is
also wrong — an opponent knows what it holds. The honest shape is one
determinized state per branch, every seat's hidden information sampled from
the searcher's point of view, and the searcher accepting an imperfect
opponent model; per-seat redeals inside one rollout are not a single game and
the engine should refuse to represent them. Recorded so that Phase 10 does not
discover this one training run at a time.

| Field | |
|---|---|
| **Rules** | CR 400.2 (public and hidden zones), 401.2 (a library's order is hidden), 402.3 (a player can't look at another's hand), 103.3 (the shuffle), 701.20 (reveal), 701.22a (scry: looked at). No rule stops an engine from reading its own state; the obligation is the harness's |
| **Verdict** | `GameState::clone` is the fork and is complete by design (item 41's premise). No facility rebuilds a clone from one player's information set, and nothing records what a player knows beyond `PermanentState.face_down` (§2.9's verdict) |
| **Size** | medium, after §2.9: a per-viewer knowledge record written by the reveal, look and scry paths; a redeal from a viewer that shuffles each library's unknown portion and swaps unknown hand cards with unknown library cards, drawing on `GameState.rng` so a determinized branch is itself seeded and replayable; and a test that a searcher over many samples predicts its next draw no better than chance |
| **Blocks** | Phase 10's search harness — any look-ahead past what the searcher knows, and any training signal from self-play, which otherwise learns to play against known draws. Nothing on the spine |
| **Atoms** | none; the CR does not speak to a harness's honesty, and none should be written |
| **Owner** | — ; filed 2026-09-18 by the owner, from A4j's review. Designed together with §2.9, right after item 6's close (`roadmap-v2.md` A6f, 2026-09-25) |

### 2.35 "Why can't I?" — the permanent that prohibits an action, as an oracle question

**The surface that cannot express it.** `engine::restriction::predicate::
is_prohibited(game, &Query) -> bool` is the one reader of every restriction
(`cant-effects-architecture.md` §3.5), it is `pub(crate)`, and it is a
disjunction: it returns `true` at the first source that forbids and names
none. Candidate enumeration already applies it per candidate, so a forbidden
action is *absent* from the prompt rather than offered and refused (§4.9:
never prompt for a choice a restriction forbids). That is right for the rules
and blind for a client: the GUI that greys a card in hand or an attack has
nothing to glow — the Sigarda that stops the sacrifice, the static that keeps
the creature home — and an agent's observation cannot carry "prohibited by
#N", so a search that sees an attack missing cannot plan to remove what
forbids it.

**It is the predicate's other question, not a second predicate.** §4.8 records
CR 101.3's "if they can't" as `is_prohibited` with the sign flipped; this is
the same sweep asked to *collect* instead of short-circuit. One entry point
over the same five sources in the same order (§3.5's reason: a second sweep is
a second place for the answer to drift), returning the sources —
`(ObjectId, AbilityId)` pairs, or the first — beside the `bool`, in `oracle/`
where a client may call it. The `Query` type is already the one thing both
would take.

**Not a middleware row.** §2.22 names it as adjacent and leaves it here: a
decorator answers prompts, and this is a query a renderer or an observation
builder asks between them.

| Field | |
|---|---|
| **Rules** | CR 101.2 (a "can't" wins), 614.17 and 613.11's cost half — the rules the predicate already enforces; none obliges the game to say *which* effect forbade, so the obligation is the client's |
| **Verdict** | `is_prohibited` answers whether, never what; nothing in the tree returns the forbidding source, and `Query` is crate-private |
| **Size** | small: a collecting twin of the sweep, ~60–100 lines plus a fixture with two forbidding sources on one board so the answer is a list and not a flag; after §2.9's per-viewer query if the answer must respect hidden information (a face-down source forbids too, and the client may not be told what it is) |
| **Blocks** | the GUI half of v1's explanation of a greyed action; an observation that names why an action is absent. Nothing on the spine |
| **Atoms** | none; the CR does not speak to interfaces, and none should be written |
| **Owner** | — ; filed 2026-09-18 by A4k, from A4j's review |


### 2.36 How an object or a player is named in text — `#17` was a mistake

**The surface that cannot express it.** `ObjectId`'s `Display` is `#17`
(A4g, 2026-09-16, chosen so an id "reads as one wherever it lands in a log
line"), and everything that renders text follows it: `ui::display`'s event
lines (`Everywhere (#39)`), the trace sink's `render_debug`, which rewrites
`{:?}`'s `ObjectId(17)` to `#17` inside a rendered action, `render_option`'s
`#17` for an object and `P0` for a player, and `plans/traces/viewer.html`,
which finds `#N` by regex to attach a name. The owner's review of A4c
(2026-09-18): `ObjectId(17)` carried more than `#17` does, and the prefix was
a mistake from the start that nobody caught. Read alone, `#17` does not say
whether it is an object, a player, a counter or an ordinal, and `P0` is a
second convention for the other id.

**What the decision is.** One rule for how an object and a player are named
in every human-facing line — event log, trace, prompt, the CLI's board, the
GUI's tooltip — and whether a machine-facing rendering (a record, a wire
payload) should say the type at all or leave it to the field's schema.
Candidates: the type's own `Debug` (`ObjectId(17)`, `PlayerId(0)`), a typed
short form (`obj 17`, `player 0`), or a name-first form with the id behind
it (`Grizzly Bears [17]`, `Alice [0]`), which is what a client wants and what
`backlog.md` §2.9's per-viewer query would supply. Not a rendering the engine
decides per site: one `Display` and one place the trace and the prompt line
take it from.

| Field | |
|---|---|
| **Rules** | none; the CR does not speak to how a game names its objects, and CR 400.7's "new object" identity is what an id already is |
| **Verdict** | `ObjectId::fmt` (`types/ids.rs`) is one convention and `render_option` is a second; four renderers repeat the choice, and a trace record's `#17` is only readable through the names table |
| **Size** | small: one `Display` decision, the four renderers, the viewer's regex, and the `format_event` fixtures that assert a line's text; a docs pass over the trace pages' examples is not owed (they are pinned) |
| **Blocks** | nothing on the spine; the readability of every log, trace and prompt line, and main item 141's wire shape, which should not inherit a spelling by accident |
| **Atoms** | none |
| **Owner** | — ; filed 2026-09-18 from the owner's review of A4c (PR #170) |

### 2.37 Which clause fits which slot — parity at TR-6's close, the matrix with custom-card design

**Revised 2026-09-24, the day it was filed.** The first version planned one
docs-only matrix: every clause slot against every vocabulary, run once TR-2 to
TR-6 had landed, to decide the owner's "universal currency" refactor. On
review, the refactor is not expected to pay, for three reasons:
- **The currency already exists.** `Effect` is the universal "do X" and
  `GameAction` the universal event. The most visible asymmetry, that "instead"
  takes only event templates, is the CR's own rule, which no refactor removes.
- **Filling the matrix's holes ahead of cards would break a rule.**
  `CLAUDE.md` says an arm the pipeline cannot apply is worse than a missing one.
- **Which holes matter is undecided.** That depends on how custom cards will be
  authored.

So the entry splits in two. The piece with v1 value is kept and moved earlier;
the matrix waits for the design that needs it.

**The surface.** Each slot takes its own vocabulary:
- **"Do X"** takes `Effect` (50 `Primitive` arms): a resolution, a trigger,
  TR-3's delayed and reflexive triggers, and a replacement's rider.
- **"Instead, X"** takes `GameActionTemplate` (8 arms). This is CR-shaped: CR
  614.1a replaces events with other events, and CR 616.1f applies replacements
  again to the result. "Instead, do any instruction" is `Rewrite::Prevent` plus
  a rider.
- **"If X would" and "when X"** are `EventPattern` (20 arms, over proposals,
  CR 614) and `TriggerEvent` (12, over records, CR 603), with no parity rule
  between them.
- `Condition` leaves, `TemplateAmount` arms and `TurnSummary` fields grow with
  the card that needs them, on purpose.

**Two pieces, two times.**
- **Parity, at TR-6's close.** A table of `EventPattern` against
  `TriggerEvent`, event kind by event kind. Each kind has both arms, or a
  recorded reason it doesn't (a CR rule, or no card yet). Commander's pool
  exercises both sides of most events, and TR-6 adds the trigger vocabulary's
  last arms.
- **The matrix, as the first step of custom-card design (post-v1).** Every
  clause slot against every vocabulary, with each hole marked either CR-forced
  or incidental. It is measured against whatever format custom cards are
  authored in, since that format's grammar decides which holes matter. No hole
  is filled ahead of a card that needs it.

| Field | |
|---|---|
| **Rules** | CR 614.1a and 614.6 (a replacement's substitute is an event); 616.1f (the loop re-gathers on it); 603.2 (any game event can be a trigger event); 608.2c (a resolution follows its instructions) |
| **Verdict** | `Rewrite::Instead` takes `GameActionTemplate` (8 arms), while every other "do X" slot takes `Effect`. `EventPattern` and `TriggerEvent` describe the same events for two readers, and no rule keeps them in parity |
| **Size** | parity is a table, about an hour. The matrix is a docs pass, about a session. The refactor the entry was first filed to decide is not expected to pay |
| **Blocks** | the matrix blocks post-v1 custom card support. Nothing on the spine |
| **Atoms** | none; the CR states the slots, not their implementation |
| **Owner** | — ; parity goes to `triggers-architecture.md` at TR-6's close, the matrix to the custom-card design. Filed and revised 2026-09-24, from the owner's questions while closing PR #182 |

### 2.38 The v1 GUI (Arena-lite) — what the engine owes it, and two open questions

**The surface that cannot serve it.** v1's first use case is four-player
Commander through a GUI, and `roadmap-v2.md` §6 puts the target between
XMage's function and Arena's polish: a board that resizes with what is on it,
and moves that animate. Four things stand between the engine and that client:

- **Nothing answers "what may this seat see".** That is §2.9, built as B4, and
  a GUI for one human renders one player's view.
- **The wire surface is not serializable.** `serde` is not a dependency, and
  `codebase-state.md` main item 141's payload rule is unmet in two places:
  `SelectRecipients` carries an `EffectRecipient` tree, and `ChoiceOption`'s
  cost options carry `Cost` trees.
- **The decision loop needs a thread that can block.** `Game::run(&dp)` calls
  the provider and waits for the answer. Forge and XMage share the design:
  Forge's game thread waits on a `CountDownLatch`, and XMage's sleeps in
  `HumanPlayer.waitForResponse` until the client answers. Natively that costs
  a thread and a channel; a browser tab's main thread cannot block, which is
  open question 1.
- **A Commander client needs state the engine does not have yet:** mulligans
  (§2.32), deck validation (§2.13), and Phase 9's Commander state.

**What a client already gets**, so Phase 10's design starts from it: prompts
that name their subject (`ChoiceKind::subject()`, matched exhaustively); a
post-replacement stream of performed events to animate from; an `ObjectId`
that survives a zone change, which main item 10's plan keeps, so a card moving
from hand to battlefield animates as one object; effective characteristics
through `oracle/characteristics.rs`; and a measured board to lay out, about 30
permanents at a priority prompt on a Commander board and up to about 80
(`roadmap-v2.md` §E, the post-RE audit's pass 3).

**Open question 1: where the engine runs.** `roadmap-v2.md` §E and §6 named
Wasm; they point here now and leave it open. In a browser tab the blocking
provider needs one of four things:

- a Web Worker waiting on `Atomics.wait`, which needs cross-origin isolation,
  and isolation also restricts cross-origin card images;
- JSPI, which lets synchronous Wasm suspend on a JavaScript promise with no
  engine change; its browser support is checked at Phase 10;
- an engine that suspends and resumes by replay. Most of it exists: a fork at
  a priority prompt is proven sound at round starts (`tests/priority_fork_test.rs`,
  main item 41) and resumes through `Game::resume_turn_at_priority`, and main
  item 140 extends it to any priority prompt. What a suspend adds is the
  unwinding: the chokepoints already return `Result` up to `Game::run`, so it
  rides the existing `?` chains, and the 27 `ask_*` call sites and the four
  trait methods change, with either a sentinel error or a typed one in place
  of the 135 `Result<_, String>` signatures. One or two PRs, mostly mechanical;
- an async rewrite, the textbook answer and the largest: 82 engine signatures
  in 20 files carry the provider (38 directly, 44 inside `ActionContext`),
  recursion needs boxing, and the cost lands on the AI harness's in-process
  path.

None of the four is needed if the engine runs natively on a host, with the UI
in a browser or a Tauri window. That is the recommendation, with Wasm as an
optional later target such as offline single-player.

**Open question 2: are v1's four seats four machines?** `roadmap-v2.md` §1
says "peer-to-peer 4-player Commander through a GUI" and §6 says "Network play
is a stretch goal". If they are, the GUI gains host and join, reconnection,
and a way through home routers. Reconnection does not need main item 40: the
host's engine thread is still blocked on the pending prompt and sends it again.

**One client, not the interface.** Nothing in the engine is shaped for this
GUI beyond what any client is owed: item 141's payload rule and §2.22's census.
The stack is a recommendation for Phase 10's design, not a decision: a
TypeScript frontend (Svelte 5, or React with Motion), a Rust host behind a
WebSocket with `serde` and `ts-rs` generating the shared types, and Tauri 2 for
the desktop app.

| Field | |
|---|---|
| **Rules** | none of its own; CR 400.2's public and hidden zones are §2.9's |
| **Verdict** | nothing answers a per-viewer query (§2.9); `serde` is absent and item 141's payload rule is unmet at `SelectRecipients` and the cost options; `Game::run`'s provider needs a thread that can block |
| **Size** | a judgment, ±50%: the frontend ~12,000–20,000 lines, the protocol and host ~2,500–4,000; about 5–8 weeks at the rate measured 2026-08-19 → 09-25 (151 PRs, ~2,200 net Rust lines a calendar day), about a third of it writing and the rest tuning; plus 2–3 weeks if the seats are four machines. Re-derive from A6g's measured rate before scheduling |
| **Blocks** | v1's first use case. Nothing on the spine |
| **Atoms** | none; the CR does not speak to interfaces |
| **Owner** | — ; Phase 10 (`roadmap-v2.md` §E), with A6g's dev GUI as its first slice. Filed 2026-09-25 from the owner's question about a GUI |

### 2.39 The grain of the CR 613.8 pre-check — a performance lever

**What the pre-check is.** `layers/board.rs` decides CR 613.8a(b) in two
steps. `Channels` compares what one application reads with what another
writes, one bit per characteristic. Any pair that comparison cannot rule out
goes to the exact test: apply the other under a journal, observe, restore. The
exact test is always right, and the grain only decides how often it runs. LL
(`layers-architecture.md` §13e) reuses the same comparison as its guard, so
the grain also decides which boards keep their hidden zones in the pass.

**What the grain costs, measured 2026-09-25** with a throwaway probe on LL's
tree that timed and classified every exact test. It ran 20 Commander games at
four seats per pool (`--games 20 --seed 12345 --players 4 --deck-size 100
--life 40 --threads 1`):

| | `performance` | `stress` |
|---|---:|---:|
| exact tests, answered yes / no | 3,283 / 1,337 | 2,336 / 1,844 |
| time in them, yes / no | 19.4 / 11.9 ms | 29.2 / 7.8 ms |
| engine time, 20 games | ~880 ms | ~1,070 ms |

Two finer grains would have ruled out most of the "no"s without applying
anything:
- **Values of card types, supertypes and colors.** An added type changes only
  a filter that tests that type. That would catch Blood Moon's "nonbasic
  lands" and Urborg's "each land" asked of March of the Machines' or
  Opalescence's added Creature, and Opalescence's "non-Aura enchantments" of
  another Opalescence. 476 of `performance`'s no's (2.2 ms) and 1,736 of
  `stress`'s (7.4 ms).
- **Direction.** An application that only adds an ability cannot end
  another's existence, which CR 604.2 reads off the source's abilities. That
  would catch a second Urborg's Swamp mana ability asked of the first, and
  Ashaya's and Citanul Hierophants' grants asked of their own copies. 643 of
  `performance`'s (7.8 ms) and 23 of `stress`'s (0.2 ms).

The rest, 218 and 85, answer "no" on this board and "yes" on another
(Urborg beside Blood Moon), which no static grain can decide.

**So about 1% of engine time on `performance` and 0.7% on `stress`**, and
more on a board where LL's guard holds a hidden zone: there the exact test
reads every card in it, about 50 µs a test on the tripped board LL measured.
Card-type values would also shrink the guard's printed trips from seven cards
beside Biotransference or Encroaching Mycosynth to Biotransference beside
Encroaching.

**For custom cards (post-v1)**, asked at PR #191's review. A trip never
changes an answer, only what a decision costs: on a tripped board, `main`'s
price before LL, about 0.25 ms a decision, which a player at a GUI cannot
notice. It matters to a research pool built for throughput, and two shapes
trip it: a row reaching a library or a hand that reads the board to resolve
(a power by an amount, a control change), and two effects of one layer
reaching one where the first writes a characteristic the second's filter
reads. The custom-card design lists both; this grain narrows the second.

| Field | |
|---|---|
| **Rules** | CR 613.8a(b) (what makes one effect depend on another), 604.2 (existence read off the source), 613.6 (a locked set) |
| **Verdict** | `Channels` is one bit per characteristic. `AddType(Artifact)` and `ByType(Creature)` share `TYPES`, and a grant and an existence check share `ABILITIES`, so the pre-check sends pairs that cannot depend to the exact test |
| **Size** | ~150 lines and ~60 of tests in `board.rs` for card-type, supertype and color values, with a debug audit that runs the exact test wherever the finer grain says "independent" and the coarse one would have looked. Direction is ~40 more. Subtypes (a large enum) and seats ("you own" rows under different players, which share no card) are further grains with no measured waste yet |
| **Blocks** | nothing; a performance lever. `Dependency checks` falls on the pools, and every gameplay row stays identical |
| **Atoms** | none; the rule is implemented, and this is its cost |
| **Owner** | — ; `layers-architecture.md` when taken. Filed 2026-09-25 at the owner's request at LL's approval, conditional on the data, which supports it |

## 3. Dispositioned — sections that need no entry of their own

The triage ran in two passes over `orphaned --bucket unbuilt`'s 63 sections.

**Pass 1 — the 38 sections a plan doc already discusses (197 atoms).** They are
in the worklist only because **citations match per rule, never per section**:
`layers-architecture.md` saying "CR 613" never claims `613.1e`. That is the
filter working as specified, not a miss. Five became §2.5–§2.8 (65 atoms) and
CR 400.2 moved to §2.9; the other 33 sections are §3.1 and §3.2 (131 atoms).

**Pass 2 — the 25 sections no plan doc mentions (100 atoms).** Five became
§2.9–§2.13 (51 atoms); the other 17 sections are §3.4 (49 atoms).

Across both, **180 of 297 atoms needed no entry.** Every table below is a
**pre-sort, not a verdict**, in exactly the sense audit §6 means: confirm a
cluster before acting on its row.

### 3.1 Owned by a doc or a critical-path item (17 sections, 76 atoms)

| CR | Atoms | Belongs to |
|---|---|---|
| 205 | 15 | Layer 4 type-changing — `layers-architecture.md` |
| 613 | 8 | 613.8c is **critical-path item 7**; 613.5/613.9/613.7m–n are `layers-architecture.md` |
| 107 | 8 | §2.1 — these are the 28 that stayed put, D3a |
| 118 | 7 | §2.1, same |
| 601 | 5 | the casting-procedure cluster — §4 |
| 305 | 4 | effective lands-per-turn, a continuous-effect query — layers |
| 704 | 4 | SBA; 704.5m Aura legality pairs with §2.6's protection |
| 608 | 4 | resolution; 608.3g is §2.8 |
| 612 | 3 | Layer 3 text-changing — unbuilt, and `layers-architecture.md`'s |
| 208 | 3 | Layer 7 P/T, incl. 208.3a's dormant effect on a type change |
| 122 | 3 | SBA (704.5i/704.5c/704.5q) |
| 110 | 3 | `is_permanent()` and characteristics; 110.4c is a layers invariant |
| 611 | 2 | continuous-effect start; 611.2d's X lock is §2.1-adjacent |
| 609 | 2 | `ObjectSet` defaults — audit §4 found no gap here |
| 109 | 2 | `EffectiveCharacteristics` — characteristics |
| 302 | 2 | P/T as a characteristic — layers |
| 111 | 1 | token cease-to-exist SBA — `copy-effects-architecture.md` |

### 3.2 Behavior that exists, missing only a test (16 sections, 55 atoms)

The `orphaned` doc's own caveat — *"behavior can exist uncited"* — is not a
footnote here; it is **the larger half of this slice.** Each of these names a
function the tree defines. This is D2b-shaped work: a test written or a
`// COVERS:` added, never a backlog entry.

| CR | Atoms | Confirmed present |
|---|---|---|
| 120 | 8 | `assign_combat_damage`, `damage_marked`, `lethal_damage_for`, `perform_cleanup_actions` |
| 104 | 6 | `check_game_over`, `draw_card`, poison SBA |
| 113 | 6 | `activate_mana_ability`; ability-type enum |
| 115 | 6 | target legality checking |
| 506 | 5 | `AttackingInfo` / `BlockingInfo`, combat step structure |
| 605 | 5 | `activate_mana_ability` and its casting-time window |
| 400 | 3 | zone guards in `move_object`; ordered zone collections. **400.2 was wrong here — it is §2.9** |
| 119 | 4 | `GameConfig.starting_life`, life gain/loss, life SBA |
| 106 | 2 | `ManaSymbol::Colored`, colorless distinct from generic |
| 121 | 2 | draw-from-empty SBA flag |
| 509 | 2 | `BlockingInfo` on declaration |
| 510 | 2 | combat damage assignment and validation |
| 103 | 1 | `Game::setup()` |
| 108 | 1 | `is_token` |
| 116 | 1 | priority after a special action |
| 508 | 1 | `AttackingInfo` on declaration |

**Two of these rows are weaker than the rest and were not confirmed to the
function**: CR 113 and CR 115 each mix implemented behavior with one atom that
is not (113.6j's zone-agnostic activation is §2.8; 115.4's "any target" names a
`TargetSpec` type the tree does not define). Confirm before annotating.

### 3.3 The `owed` policy — decided

`owed` selects atoms in a shipped phase, ticketed `NEW…`, with no test. Its
docstring says what it is for: `ATOM-400.7-001`'s ticket specified a
`zone_change_epoch` field on `GameObject` **down to the name**, nobody asked at
phase close, and the design was lost for two years. **`owed` exists to stop a
design dying inside a ticket.**

That makes the rule about capture, not about bookkeeping:

> **Re-file when a backlog entry or a Deferred Migrations item has captured the
> design. Keep when the ticket is the only place the design exists.**

Re-filing a captured atom loses nothing — the ticket has been read, and the
entry says more than it did. Re-filing an uncaptured one is exactly the failure
`owed` was built after. **This is also why "re-file everything" and "re-file
nothing" were both wrong**: the first guts the gate, the second leaves it
permanently red at 31, which erodes a gate into a report nobody reads.

Applied 2026-08-31: **22 re-filed, 9 kept, `owed` 31 → 9.** Three entries were
strengthened first, so that no retired ticket said more than the entry
inheriting it — §2.13 gained CR 100.4a's combined main-plus-sideboard copy
count, §2.9 gained the two tickets' shared siting of the query in the oracle
layer, and §2.7 gained devotion's modifiability.

**The nine that stayed, and why each is uncaptured:**

| Atom | Why it stays |
|---|---|
| `ATOM-208.4b-001` | names `get_base_power`/`get_base_toughness` down to the name — the `zone_change_epoch` case exactly |
| `ATOM-400.7a-002` | text-change persistence across stack→battlefield; Layer 3, and no entry covers it |
| `ATOM-613.1f-002` | Layer 6 keyword counters (roadmap D10) — §3.1 files CR 613 as owned, but not this |
| `ATOM-605.1a-002/004` | a mana ability may not require a target. **Contradicts §3.2's pre-sort**, which read CR 605 as implemented — confirm before annotating |
| `ATOM-502.3-002`, `ATOM-703.4c-002` | "doesn't untap" restriction effects — **one mechanic wearing two section numbers**; §2.14 names it now (2026-08-31) — re-file at the next pass |
| `COMP-ZONE-TRANSITION-001` | a cross-rule composition test to write; no design to capture |
| `ATOM-702.19d-001` | a trample regression test — D2b-shaped, not backlog |

Two of the nine were worth acting on rather than filing: the **605.1a pair**
disagrees with a disposition in this file (still open — confirm before
annotating), and the **502.3/703.4c pair** got its entry, §2.14 (2026-08-31).

**Applied 2026-09-15 to Phase 6, at the post-RE audit's close-out** (pass 1 of
the post-RE audit — `codebase-state.md`, "Was critical-path item 5 done, and what sits before item 6? — audited 2026-09-15"). The replacement track had closed against a
`SHIPPED_PHASES` that did not contain it, so this was the query's first run
over the phase: **127 atoms, 50 uncovered, 21 of them ticketed `NEW`; `owed`
21 → 0**, and Phase 6 joined the constant the same day. Four atoms were proven
by tests that lacked their line (two with one assertion added so the claim is
asserted and not implied), one got a test written the same day, four are
partial, and twenty-two were re-filed with the owner written into the ticket
— every re-filed ticket keeps its original words after `was:`, so nothing the
corpus author wrote is lost. The nineteen CR 707 atoms carry `D5` and stay as
they are: `copy-effects-architecture.md` §8 owns them by name and argues
against re-tagging them.

| Atom | Disposition (2026-09-15) |
|---|---|
| `ATOM-400.6-001` | **covered** — `test_tapland_enters_tapped` (RC-2) |
| `ATOM-603.6d-001` | **covered** — the same test, with the stack asserted empty |
| `BOUNDARY-DEF-614.1d-001` | **covered** — `test_root_maze_taps_an_entering_land` (RC-3), stack asserted empty |
| `COMP-7A-004` | **covered** — `test_cant_be_regenerated_withholds_the_shield_without_destroying_it` (RB) |
| `ATOM-400.7c-001` | **tested** — `a_shield_chosen_on_a_spell_follows_it_onto_the_battlefield` (RD-3, new); the engine gets CR 400.7c from object identity, `move_object` keeping the id across the stack→battlefield move |
| `ATOM-400.6-002` | **partial** — RB's two `ChooseReplacementEffect` tests: a sacrifice with exile against stay, not the atom's destruction with exile against hand |
| `COMP-7A-001` | **partial** — `test_sacrifice_is_not_destruction_so_indestructible_does_not_save_it` (RS-1); the regeneration-shield half is not on the board |
| `ATOM-613.1a-001` | **partial** — CV-1's `test_later_layers_still_apply_to_the_copy` proves the layer-1 base under a layer-7c add with a "becomes a copy" row; the entering Clone is CV-2's |
| `ATOM-611.2c-002` | **partial** — `fog_prevents_damage_from_a_creature_that_entered_after_it_resolved` (RD-3, new); Fog is combat damage where the atom's effect is all damage |
| `ATOM-400.7c-002` | **re-filed** — Phase 8's first token-making planeswalker; the identity half is `ATOM-615.9-001`'s, covered |
| `ATOM-611.2c-003` | **re-filed** — RS-3 for "can't be blocked"; the lock-in half is Phase 5-Layers' |
| `ATOM-614.10b-001` | **re-filed** — critical-path item 6; no printed card says "skip … then" (RE-1's census) and the follow-up is a trigger |
| `ATOM-614.11b-001` | **re-filed** — §2.26 |
| `ATOM-614.15-001`, `ATOM-614.15-002`, `ATOM-616.1a-001`, `ATOM-614.17c-001` | **re-filed** — `replacement-architecture.md` §11 item 3, the self-replacement producer, fixture-first |
| `ATOM-614.17a-001` | **re-filed** — RS-3 (combat) |
| `ATOM-614.17b-001` | **re-filed** — RS-4 (costs) |
| `ATOM-616.1c-001` | **re-filed** — CV-2; the CR 616.1c bucket has existed since RC-4 |
| `ATOM-701.40f-001` | **re-filed** — CV-6; manifest needs face-down |
| `ATOM-702.176a-001` | **re-filed** — §2.6, after item 6 for the end-step trigger |
| `ATOM-704.5e-001`, `ATOM-704.5e-002` | **re-filed** — CV-4, CR 707.10a's SBA |
| `ATOM-107.3m-001` | **re-filed** — Phase 8's first "enters with X counters" card; X is unreadable at resolution at all today (`codebase-state.md`, the CR 601.2b row), which comes first |
| `ATOM-607.2b-001`, `ATOM-607.2g-001` | **re-filed** — §2.2 |
| `COMP-613-LAYERS-FULL-STACK-001` | **re-filed** — Layer 3 is unbuilt and the Clone half is CV-2's; the per-layer atoms it composes are covered one by one |
| `ATOM-702.15d-001`, `ATOM-702.2d-001` | **kept deferred**, the owner added — §2.8, on CR 113.6 (LK) |
| `ATOM-702.180a-002` | **re-filed** — §2.6 (harmonize); its exile-instead half is a stack-exit rewrite the pipeline expresses today |
| nineteen `ATOM-707.*` and `BOUNDARY-707.2c-001` | **owned as they stand** — `copy-effects-architecture.md` §8: CV-2, CV-3, CV-4 |

Three things the run learned, for the next close: **a `COVERS-PARTIAL` is enough
to silence `owed`** — the gate reads the coverage table, which holds partials —
so the partial column of `specdb stats` is where a phase's remainder lives once
the gate is green, and a close should read both; **a ticket that is not `NEW`
is invisible to the gate whatever it says** (`D5`, `T20`, `Phase 6`, "(same as
above)"), which is the CR 707 pattern §8 already names, and the reason this
pass read `--all` rather than the default; and **the two atoms that arrived
already `DEFERRED`** (702.15d, 702.2d) were the only recorded deferrals in a
127-atom phase, which is what the `SHIPPED_PHASES` omission cost.

### 3.4 Pass 2's remainder (17 sections, 49 atoms)

Unmentioned by any plan doc, and still no entry — the same two shapes as §3.1
and §3.2, which is the useful result: **being undiscussed did not predict being
unbuilt.** Pass 2's hit rate (5 entries from 25 sections) is barely better than
pass 1's (5 from 38), so "no doc mentions it" turned out to be a weak signal.

| CR | Atoms | Disposition |
|---|---|---|
| 117 | 8 | priority — implemented: `pass_priority`, `resolve_top_of_stack`, untap-step skip |
| 405 | 8 | the stack — implemented: LIFO, `StackEntry.controller`, mana abilities bypassing it |
| 403 | 4 | 403.2's battlefield-default scope is targeting; 403.4's new-object-on-ETB is CR 400.7 object identity |
| 301 | 3 | Equipment — 301.5b ETB unattached; 301.5c's two are SBA legality, beside §2.6's protection and §2.5's `Unattach` |
| 307 | 3 | 307.4 is the zone guard already in §3.2's CR 400 row; 307.5's two are `check_cast_legality` sorcery timing |
| 500 | 3 | 500.1–500.3 phase iteration and priority — implemented (500.4/500.5 are §2.12) |
| 505 | 3 | two main phases; sorcery-speed casting — implemented |
| 703 | 3 | 703.3 TBA ordering and 703.4d draw step implemented; **703.4c untap restrictions is `owed` with a `NEW` ticket** |
| 112 | 2 | spell controller — implemented |
| 303 | 2 | Aura SBAs — beside 704.5m, already §3.1 |
| 304 | 2 | instant zone guard and instant-speed timing — implemented |
| 402 | 2 | maximum hand size — `perform_cleanup_actions` / `handle_cleanup_discard` (402.3 is §2.9) |
| 404 | 2 | graveyard ordering and empty start — implemented (404.2 is §2.9) |
| 201 | 1 | 201.2a is audit §5.2's "an object with several names" near-miss — already recorded, needs Layer 3 |
| 300 | 1 | `CardType` enum completeness — implemented |
| 503 | 1 | upkeep step has no TBA — implemented |
| 504 | 1 | draw-step TBA — implemented |

---

## 4. What the triage learned

```bash
python plans/specdb.py orphaned --bucket unbuilt --all
```

**All 63 sections are triaged.** The query reports **277 atoms across 63
sections** after §3.3's re-file — down from the 297 the triage read, because 20
of the 22 re-filed atoms were in this bucket. It will not fall much further on
its own: **re-filing is not how a section leaves that list, and §3's
dispositions are prose the query cannot read.** Use it as the source, not as a
progress bar, and regenerate it before trusting any number here.

Thirteen entries and 180 dispositioned atoms came out of it (§2.14 arrived
later from `owed`'s kept pair; §2.15–§2.17 from the 2026-08-31
`PlayerState`/`Zone` type sweep). Six things learned,
recorded so they are not re-derived:

1. **A CR section is a loose proxy for a mechanic, and it over-collects.** The
   CR 107/118/202 shipped-phase bucket reads as one cluster and is at least four:
   cost modification, exotic mana-symbol payment, mana-value computation, and
   Layer 5 color derivation — plus a fifth group that is genuinely implemented
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

3. **The `Mechanism`-field method does not automate.** Matching backticked
   identifiers against what the Rust tree defines was tried and abandoned: 224
   of the 297 name no identifier at all, and the "absent" bucket is mostly false
   negatives — `sba.rs` is a file, `TargetSpec::AnyTarget` a variant, `i32` a
   primitive. **It is a human read**, and that is the sizing constraint: D3a read
   ~97 atoms by eye, slice 1 read ~197.

4. **Being undiscussed did not predict being unbuilt.** The obvious hypothesis
   going into pass 2 was that the sections no plan doc mentions would be the
   real work. They were not: pass 2 yielded 5 entries from 25 sections, pass 1
   yielded 5 from 38. **"No doc mentions it" is a weak signal** — weaker than
   the `Mechanism` field, and roughly as weak as the source-citation pre-sort
   audit §6 already measured.

5. **Scaffolding with no consumer is the recurring shape.** Three found in one
   triage: `StackEntry.chosen_modes` (§2.7, twelve empty writers, no reader),
   `CardData.color_indicator` (§2.10, audit §5.2's near-miss), and `DeckLimits`
   (§2.13, fully configured, never validated against). Each looks like progress
   in a grep and is debt. **When an entry's verdict is "the type is right and
   nothing uses it", it belongs in `codebase-state.md` Deferred Migrations, and
   the entry here is a pointer.**

6. **The one v1 blocker in this file is §2.9.** Everything else is card breadth
   or a mechanic that can wait for the cards that need it. **No per-player
   visibility query exists at all**, and both of `CLAUDE.md`'s v1 use cases —
   a GUI and parallel AI games — need one. It is unbuilt because the CLI is
   omniscient, so nothing has ever asked.

The 25 remaining CR 601 casting-procedure atoms — modal announcement,
kicker-conditional targets, divide-or-distribute (CR 601.2d, which audit §4
already sizes as "same shape as `x_value`"), the 601.2g mana window, 601.4's
look-ahead — are **the one cluster deliberately left without an entry.** Their
verdicts are not established, and §3.1 files them as owned by that pending
judgment rather than pretending otherwise.

---

## 5. What this file does not cover

- **The critical path** — `CLAUDE.md` items 1–7, and the Commander/multiplayer
  track interleaved after item 5. Cited by number elsewhere; do not restate them
  here.
- **Deferred migrations** — debt owed by scaffolding already in the tree lives in
  `codebase-state.md`, which wins over every other doc on current state. A stub
  with a `TODO` is that file's; a mechanic with no code is this one's.
- **Missing tests for behavior that exists** — needs a test written or a
  `// COVERS:` added, never a backlog entry. This is the workstream's largest
  outstanding item and it grew during the triage: **55 atoms** in `orphaned`'s
  `cited` half (behavior encoded in `src/` that no test mentions), plus the
  **104 in §3.2 and §3.4** that this file confirmed present and untested.
  ~159 tests to write, and none of it buys new capability — it is regression
  protection for behavior that already works, so it wants doing in
  high-value subsets beside other work rather than as a block.
  Audit §6 measures how weak the `cited` signal is; §3.2's rows name the
  functions.
- **Corpus authoring** — rules carrying a verdict but no atom, overwhelmingly
  CR 702 keyword subrules. Unscheduled, and it belongs beside the phases that
  need it. The type-sweep probes named three more spots, thin rather than
  empty: CR 402.2's modification half, CR 122.1's player-counter kinds, and
  CR 500.7's extra turns.
