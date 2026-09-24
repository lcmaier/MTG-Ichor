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

**Two checks per fact, added 2026-09-24** after item 30's shape was fixed twice
in review (§4a):

1. **Recorded when it exists, on the path production takes.** A type with a
   track that could hold the fact does not count if production never fills it,
   and a fact that exists only for a moment has to be captured in that moment.
2. **Its shape survives the rules that watch it.** At least a copy (CR 707.10:
   what a copy keeps and what it doesn't get), a zone change (CR 400.7), a
   control change, a replacement (CR 614: the modified event, not the proposed
   one), and per unit against aggregate.

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
| Multi-component permanent (CR 729) | `PermanentState` | ✅ one `object_id` per entity |
| Counters off the battlefield (CR 122.1a/b) | `GameObject` vs `PermanentState` | ✅ `counters` is on the battlefield sidecar only |
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

**Run again 2026-09-24, for §2's two checks**, against item 30 as it stood at
`7a7820c`, before PR #181 reshaped it. The run was blind: a subagent worked in a
worktree at that commit, was not told what #181 changed, and had the checks
without their examples. **It re-found both halves.** The copy leg found that
the cost decisions held inside `Option<CastFacts>` answer "was it kicked" or
"was it cast" wrongly for a copy of a kicked permanent spell, which is #181's
`CostChoices`. Check 1 with the per-unit leg found that a unit's source is gone
at `pool.add`, so capturing it at CR 601.2h is too late, which is item 33's
slot. The worktree had no CR, since `MTG-Rules/` is untracked, so the run cited
rules from quotes in the tree. Its other readings are in §4a.

---

## 4. The sweep

Per type: read it, read the CR sections describing what it models, answer the
question. Run 2026-08-31 over seventeen types; the re-sweep's rows (§4a) follow
the first table.

| Type | What the CR wants that it can't say | Verdict |
|---|---|---|
| `GameObject` | face-down state in a non-battlefield zone (foretell, CR 702.143) | feature — a flag, same shape as `is_token` |
| `PermanentState` | CR 729 components; counters elsewhere | **known facts**, already owned |
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
| `ContinuousEffect` | CR 611.2c's locked set; CR 613.8 dependency | **no gap** — `ObjectSet::Fixed` is exactly 611.2c; 613.8 is critical-path item 7 |
| `StackEntry` | **what was spent to pay the costs** | **FACT** — §5.1 |
| `ManaPool` | non-fungible mana — CR 106.6 restrictions, grants, persistence | **no gap in the type** (T12b built it); the *gatekeepers* are unwired — Deferred Migrations item 33. **Corrected 2026-09-24:** true of the restricted track only; the plain pool production uses drops a unit's source at `pool.add` (item 33) |
| `PlayerState` | continuous effects on player values and rules (CR 402.2, 613.10–613.11); counter *kinds* on players (CR 122.1) | features — `backlog.md` §2.15, §2.16; probed by Winter, Misanthropic Guide |
| `Zone` | per-viewer visibility; object identity across zones; CR 729 merging | **no gap** — every demand already owned (§2.9, item 6's LKI, CV-7). `is_public()` exists, unconsumed |

**The last three rows were not in the original fourteen — each arrived by
probe, all on 2026-08-31.** `ManaPool` from `o:"this mana"` (227 cards);
`PlayerState` from a single card — Winter, Misanthropic Guide, whose
maximum-hand-size clause is CR 613.11's own worked example; `Zone` swept
beside it to close the question. Each verdict is what §3's refined unit
exists for: a type can be complete while its gatekeepers are not, which a
field-level read cannot see.

**The list now states its criterion: every type `GameState` transitively
owns that stores rule-relevant state.** The remainder under it,
dispositioned rather than swept: `GameConfig` (already probed — item 32's
`DeckLimits` was its finding); `AttackingInfo`/`BlockingInfo` (RS-3's design
owns the combat-restriction surface); turn structure (one gap, extra turns —
`backlog.md` §2.17; skips are replacement effects, CR 614.10); the
`GameState`-level designations (monarch, initiative — session 9b atoms,
filed at Phase 7 as D15's source-less triggers); and the `GameAction`/event
vocabulary, which grows by contract (`CLAUDE.md`). A new type joins the
sweep the day it joins `GameState`. Card-population probes stay on as the
standing check that the criterion holds — three ran, three earned their
keep. *That promise lapsed between `3c322e5` and 2026-09-24, while `GameState`
went from 28 fields to 52; §4a is the catch-up. `check_type_surface.py` now
holds it: a pull request that adds a field this document does not name fails
(§4b).*

**Rows added by the re-sweep (2026-09-24, §4a): pass 1, replacement; pass 2, triggers; pass 3, cost; pass 4, the rest.**

| Type | What the CR wants that it can't say | Verdict |
|---|---|---|
| `PermanentState` *(re-read: `cast`, `cost_choices`)* | an "as it enters" choice (CR 614.12a, 707.6) | feature — `backlog.md` §2.2 |
| `ResolvingObject` *(for the entry)* | — | **no gap** — carries CR 400.7d's facts and the cost decisions to an "enters with" before the permanent exists |
| `ReplacementEffectRegistry` | a chosen source (CR 609.7a); targets held across a move (CR 400.7) | feature; main item 10 |
| the applied set (`rider_lineage`, the loop's `lineage`) | which replacement redirected an event, once it has been performed (CR 702.35a) | **FACT** — §5.4 |
| `EntrySelectionScope` | what an entry's zone change chose, after the batch (CR 614.14, 702.82b) | feature — `backlog.md` §2.2 |
| `PreventionAllocationScope` | — | **no gap** — CR 615.7, scoped to the batch |
| `EventLog` / `EventRecord` | the act of a zone change an "instead" redirected (CR 614.6, 701.9c) | **FACT** — §5.4 |
| `PendingTrigger` *(pass 2)* | — | **no gap** — the controller (CR 603.3a), the def and the source's card are all taken as it triggers |
| `TriggerBinding` *(pass 2)* | the object a move made (CR 400.7e, 603.6c), which the `ZoneChange` record does not carry | item 177 |
| `DepartureFrame`, `LookBackSnapshot` *(pass 2)* | a status (items 14, 15); a departed permanent's cost decisions (CR 603.4, 113.7a) | TR-4's frame; item 169, and `triggers-architecture.md` §3.11 amended |
| the window (`EventLog`) *(pass 2)* | the state just after a record's own batch, once a rider has run (CR 603.4, 603.6a) | item 175, sharpened |
| `StackEntry` *(pass 3: `trigger`, `mana_spent`)* | the objects that paid (CR 707.10, 400.7d); what a copy of it may keep (CR 707.10) | item 30's open half, sharpened; `copy-effects-architecture.md` §4.4 amended |
| `CastFacts`, `CostChoices`, `ManaSpent` *(pass 3)* | a count of payments (multikicker, replicate, squad) | feature: the record can repeat, and the CR 601.2b announcement cannot ask for a number |
| `ManaPool` *(pass 3, re-read)* | a unit's source; restrictions (CR 106.6, 107.4h) | item 33, as corrected above |
| `TargetInstance` *(pass 3)* | a target that left and came back (CR 400.7) | main item 10's `target_epochs` |
| `ResolvingObject` *(pass 3)* | — | **no gap** — CR 110.2b's default controller and CR 400.7d's facts, carried past the entry's removal |
| `GameObject` *(pass 4: `timestamp`)* | — | **no gap** — CR 613.7d–e built; 613.7m is "Before card breadth" item 4 |
| `PlayerState` *(pass 4: `counters`)* | a team's shared poison (CR 701.34b) | feature, Phase 9's Two-Headed Giant |
| `RestrictionRegistry` *(pass 4)* | a target that left and came back; the object a "for as long as" watches | main item 10; item 17 |
| `DurationRegistry` *(pass 4)* | the object a resolution's "for as long as" watches (CR 611.2b) | item 17, sharpened |
| `TurnPlan`, `turn_queue`, `turn_rotation` *(pass 4)* | a proposed turn being an extra one (CR 614.10) | `triggers-architecture.md` §3.9 amended |
| `GameResult`, `starting_life` *(pass 4)* | a team's win; a seat's own starting life (CR 103.4b, 103.4e) | features, Phase 9's variants |
| the departed frame *(pass 4, planned)* | a source or bound object leaving a hand or the stack (CR 113.7a, 608.2h) | item 169, sharpened |
| `CostChoices` *(pass 4, re-read)* | the player a payment chose (CR 702.174a) | item 30, sharpened |

---

## 4a. The re-sweep — facts that exist at one moment (2026-09-24)

**Why it ran.** Item 30 recorded one fact, and PR #181 had to fix its shape
twice. A unit's source looked like a missing field and was really the pool's
representation (item 33). "Kicked" was stored as part of how the spell was cast,
but CR 707.10 copies it to a copy that was never cast (`CostChoices`, beside
`CastFacts`). The 2026-08-31 sweep had found item 30 and still called
`ManaPool` "no gap in the type", which is true only of a track production never
fills. §2's two checks and §3's second run come from that.

**Sized before reading.**
- **Types.** Everything `GameState` owns through its fields' types (struct,
  enum and alias definitions under `mtgsim/src`, without card files, binaries
  or `#[cfg(test)]`) comes to **170** types, against 96 at `3c322e5`.
  - §4's criterion keeps the **43 that record something about this game**.
  - The rest are 96 vocabulary types (card text as data: `Effect`,
    `TriggerDef`, `ReplacementDef`, the filters), 18 ids and keys, 10
    instruments and caches, and 3 recomputed on every read
    (`EffectiveCharacteristics`, `CopiableValues`, `Resolved`).
  - The 43 make **23 units**. Types already swept and unchanged since
    (`ContinuousEffect` and its registry, `Phase`) are out, and a sub-part is
    read with its parent. `rider_lineage`, which holds CR 614.5's applied set,
    counts as a unit though it is a field rather than a type. The 23 are 5
    re-reads, 3 types that predate the sweep but were never in it, and 15 new.
- **CR lines.** Seven phrases read one moment's past, counted with
  `grep -E '^[0-9]{3}\.[0-9]+[a-z]?\.? ' tmnt.txt | grep -ciE '<phrase>'`:
  "chosen" 102, "this way" 59, "as .* enters" 47, "was paid" 21, "was cast" 20,
  "last known information" 14, "was spent" 3.
  - Together they hit **242** of 3,120 rule lines.
  - 77 of those are CR 702 keywords, 12 are CR 614–616, 4 are CR 603, 11 are
    casting, cost and mana, 5 are CR 707, and 133 belong to no one subsystem.

That is large, so the owner split it into four passes: **replacement, then
triggers, then cost, then the rest**. **All four ran on 2026-09-24.**

**Pass 1: replacement (CR 614–616).** Card counts are Scryfall's
`total_cards` for `game:paper -is:funny`, with the query beside each number.

| Fact | Where it lives | Recorded when it exists? | The rules that watch it | Verdict |
|---|---|---|---|---|
| How a permanent enters: tapped, its counters and who puts them (CR 614.1c–d) | `EnterMods` on the entry proposal, then `PermanentState` | yes | a copy entering applies the copied "enters with" (707.5, CV) | no gap; announcing the counters is "Before Triggered abilities" item 17 |
| An "as it enters" choice: a color, a creature type, a player, an anchor word (614.12a, 614.12c) | **nowhere**: no slot in `EnterMods` or `PermanentState` | — | a copy entering makes its own choice, and a permanent that becomes a copy later has none (707.6); it leaves with the permanent (400.7); a control change keeps it | feature with a constraint, `backlog.md` §2.2 (CR 607.2d). 208 cards, `o:/as [^.]*enters[^.]*, choose/`, of which 192 read "chosen" |
| What an entry's zone change chose: the cards Sutured Ghoul exiled (614.14), the creatures a devourer ate (702.82b) | `EntrySelectionScope.chosen`, for one batch. `AuxiliaryMove.per_chosen` turns it into counters | **no**: it is gone when the batch closes | a copy that gains the pair links anew (614.14); the exiled cards are new objects (400.7) | feature, `backlog.md` §2.2. Sutured Ghoul is item 59, already reachable and wrong. 8 readers: `o:"devoured"` 6, plus the entry pair "the exiled cards" 2 |
| The effects that have already applied to an event (614.5) | `lineage` per member in the loop; `rider_lineage` during a rider | yes, and a modified event or a rider inherits it | 903.9b's exemption is built; 717.6's is in §5.2 | no gap, but it is dropped once the event performs, and the next two rows need it |
| **What an "instead" left behind: the act of a redirected zone change** (614.6, 701.9c) | `GameEvent::ZoneChange.cause` | **no**: `Rewrite::Instead(ZoneChangeTo)` writes its own `cause` | the rulings on Rest in Peace, Leyline of the Void and Nephalia Academy say the card was still discarded; `triggers-architecture.md` §4 matches a sacrifice or discard on `cause` | **FACT**: item 176, reachable and wrong today |
| Which replacement redirected an event ("when this card is exiled this way", 702.35a) | nowhere, once the event has been performed | no | — | the same item's second half. 61 cards (`kw:madness`). CR 615.13's version already has a shape: `DamagePrevented.by` ("Before Triggered abilities" item 16) |
| The damage a prevention effect prevented (615.5, 615.13) | `Rider.prevented`, read by `AmountExpr::DamagePrevented`; `PreventionAllocationScope` for the batch | yes | CR 615.13's unit is one application, which the rider's number already is | no gap. 35 cards read it through a rider (`o:"prevented this way"`); the trigger is "Before Triggered abilities" item 16 (TR-5) |
| A chosen source, "a source of your choice" (609.7a) | nowhere: `SourceFilter` is `ControlledBy` only | — | 400.7c | feature: an additive `SourceFilter` arm that holds the source's identity. 65 cards (`o:"source of your choice"`) |
| A resolution-created row's targets (Divine Deflection) | `RegisteredReplacementEffect.targets` | yes | **400.7**: a bare `ObjectId` finds a target that left and came back | main item 10, sharpened with a fourth kind of reference |
| The entering spell's cost decisions, read by an "enters with" before the permanent exists (kicked, sunburst) | `ResolvingObject.cost_choices` and `.cast` | yes: `cost_choices()` checks the resolving object first | a copy of a kicked spell enters kicked (707.10) | no gap since #181. §3's run raised it, reading the tree from before #181 |

**One fact, and it is §5.1's shape again:** the event keeps the fact's
*outcome* and loses the fact itself. Item 30's mana arrived as counts per type.
Here the redirected move arrives as `Exiled`. A redirected discard is still a
discard, and trigger cards read the act: `trigger-survey.md` table two counts
555 behind "sacrifices" and 436 behind "discards".

**Pass 2: triggers (CR 603, 113.7a, 608.2h).** The four records, the window
they are dispatched from, and the shapes `triggers-architecture.md` has
already planned for them.

| Fact | Where it lives | Recorded when it exists? | The rules that watch it | Verdict |
|---|---|---|---|---|
| The trigger's controller (603.3a) and its ability as it triggered (113.7a) | `PendingTrigger.controller`; `TriggerBinding.def`, cloned out of the effective list | yes, at the dispatch | a later control change or Humility changes neither | no gap |
| The event's facts: who, and how much (603.2c, 608.2c) | the records, by `EventSeq`; nothing prunes the log within a game | yes | one trigger per occurrence or per window (TR-1); "that many" sums the records | no gap |
| **The object a move made** (400.7e, 603.6c) | `TriggerBinding.object`, the epoch **at dispatch**; the `ZoneChange` record has none | **no**: the move stamps it and the dispatch reads it later | a second move before the window's dispatch; a `OncePerEvent` "them" (59 cards) | item 177 |
| The appearance before a departure (603.10a) | `DepartureFrame`, then `ZoneChange.lki`; `LookBackSnapshot` for survivors | yes (TR-1, TR-1b) | status and the other two classes are items 14 and 15 (TR-4) | no gap beyond those |
| **A source's last known information after it triggered** (113.7a, 608.2h) | planned: §6.1's `departed` frame, typed as §3.11's `LastKnownInformation` | planned | **the cost decisions**: §3.11 carried `cast` only, and #181 moved kicked, bargained and evoked out of it. 72 "if it was kicked" triggers | item 169 sharpened; §3.11 amended |
| **The intervening "if" at the trigger** (603.4) | `settled_holds` when the window closes | **no**, for a window with a rider: the rider has run (615.5) | a rider that changes the condition; a rider's newcomer asked about an earlier record (603.6a) | item 175 sharpened |
| **"Do this only once each turn"** (603.2h) | planned: `action_taken_this_turn`, keyed by the ability | planned | **a control change**: the rule keys the gate on "its source's controller". 34 cards | §3.5, §6.4 and §7 amended |
| A state trigger's re-arm (603.8); "only once each turn" | planned: sets of `AbilityIdentity`, which carries the epoch | planned | a zone change resets both (400.7) | no gap |
| A delayed trigger's provenance and objects (603.7a–h) | planned: `DelayedTrigger` (§3.9) | planned: controller, source and refs as of the creating instant | refs by epoch, from the performed records | no gap, but the refs read item 177's missing field |
| A copy of a triggered ability (707.10b) | a clone of its `StackEntry`: identity, binding, frame | yes | counted as the same ability (§6.5) | no gap |
| Modes on a trigger (603.3c); a damage source's keywords from its LKI (702.2e, 702.15c, 702.80b, 702.90d) | `StackEntry.chosen_modes`, never written (item 31); `has_keyword`, live | — | — | owned: `backlog.md` §2.7 and §2.6 |

**Pass 2's findings are all one moment read at another.** Item 177 reads, at
the dispatch, the identity a move created. Item 175 reads, after the rider, a
condition the event met. The other two are planned shapes written before a
rule or a split reached them: CR 603.2h's "its source's controller", and #181's
cost decisions. None is reachable today.

**Pass 3: cost (CR 601.2, 118, 106–107, 707.10).** The facts a cast records,
where each lands, and what CR 707.10 lets a copy keep.

| Fact | Where it lives | Recorded when it exists? | The rules that watch it | Verdict |
|---|---|---|---|---|
| Who cast it, and from where (601.2a, 400.7d) | `StackEntry.cast_from` → `CastFacts.by`/`.from` | yes | a copy isn't cast (707.10); after it leaves, §3.11's frame | no gap |
| The mana spent, by type (601.2h, 400.7d) | `StackEntry.mana_spent` → `CastFacts.mana_spent` | yes, from `ManaPool::pay`'s return, on the plain pool production uses | a unit's source is item 33's; expend (700.14) is a `TurnSummary` field authored with its card, and the payment is on the entry when `SpellCast` dispatches | no gap |
| The cost decisions (601.2b, 118.8–9) | `StackEntry` → `CostChoices` | yes | a copy keeps them (707.10). Multikicker (19 cards), replicate (19) and squad (15) count payments: the `Vec` can repeat an entry, but the announcement cannot ask for a number. 702.152b and 702.157b's per-instance identity goes with the printed position | feature; no gap in the record |
| X (107.3m, 107.3h) | `x_value`, `StackEntry` → `PermanentState` | yes | a copy keeps it (707.10) | no gap |
| Targets, one entry per instance (601.2c, 115.3) | `TargetInstance` | yes | a target that left and came back (400.7) | owned: main item 10's `target_epochs` |
| Modes (601.2b, 700.2) | `chosen_modes`, never written | — (nothing is modal) | a copy keeps them | owned: item 31, `backlog.md` §2.7 |
| **The objects that paid** (601.2h, 400.7d) | `PaymentPlan.sacrifices`, until the cast completes | **no** | a copy uses the original's (707.10); Fling reads one object's power from its LKI; convoke's objects never move | item 30's open half, sharpened: beside the cost decisions, a list of identities. 34 + 6 readers |
| **What a copy of the spell gets** (707.10) | planned: CV-4's "`StackEntry` clone plus a new `ObjectId`" | planned | a clone keeps `cast_from`, `mana_spent` and `controller`, and 707.10 gives the copy none of the original's | `copy-effects-architecture.md` §4.4 amended |
| **Storm's count** (702.40a) | planned: `TurnSummary.spells_cast`, read live | planned | a spell cast in response is not "before it"; 33 cards | `triggers-architecture.md` §3.10 amended |
| A mana unit's source; restrictions (106.6, 107.4h) | dropped by the plain pool at `pool.add` | no | per unit | owned: item 33, slot B9 |

**Pass 3 files no new item.** Production records each cost fact at its own
moment. The one it does not record, the objects that paid, was already item
30's open half. The findings are planned shapes around those facts:
- CV-4's clone would keep three things a copy doesn't get (CR 707.10).
- Item 30's objects must sit with what a copy keeps, the same split #181 made
  for kicked (CR 707.10).
- Storm's count would be read at the wrong moment (CR 702.40a).

**The remainder, dispositioned.** 13 of the 25 fields added since `3c322e5`
hold no fact about the game:
- the five fast-path gates over printed abilities, which are indexes and fall
  under `CLAUDE.md`'s gate rule: `cost_modification_ability_sources`,
  `restriction_ability_sources`, `trigger_sources`,
  `zone_replacement_ability_sources`, `zone_trigger_sources`;
- the layer walk's cache, `layer_epoch` and `layer_memo`;
- three instruments nothing may branch on: `diagnostics`, `dispatch_audit`,
  `trace`;
- `nesting`, whose type `NestingGuards` guards the engine rather than a rule;
- two id allocators, `next_trigger_seq` and `next_object_id`.

**Carried to pass 3 from §3's run, and answered there.** CV-4's clone would
keep `mana_spent`, and also `cast_from` and `controller`, so
`copy-effects-architecture.md` §4.4 is amended. The multikicker count is a
feature: `CostChoices.additional` can repeat an entry, and the announcement
cannot yet ask for one.

**Fixed on sight: nothing.** The one wrong answer, item 176, needs a design line
and an A/B because it changes whether a CR 616.1 prompt appears, so it is an
item rather than an entry on the fix list.

**Pass 4: the rest.** The six types §4 listed, and the CR lines no pass had
read. `GameState`'s declaration had not changed since `c2dbf99`, so the list
stood.

| Fact | Where it lives | Recorded when it exists? | The rules that watch it | Verdict |
|---|---|---|---|---|
| An object's timestamp (613.7d, 613.7e) | `GameObject.timestamp` | yes: `add_object` and `move_object` stamp it, and `attach` stamps it again | simultaneous arrivals, in APNAP order and each player's own (613.7m); turning face up, transforming (613.7f–g) | no gap: 613.7m is "Before card breadth" item 4, 613.7f–g are `roadmap.md` D3, and 613.7n rides LH-2 |
| A player's counters (122.1) | `PlayerState.counters` | yes, through `perform_action`'s counter arms (RE-5) | proliferate (701.34a); a team's shared poison (701.34b) | no gap for v1: Two-Headed Giant is Phase 9's |
| A resolution's "can't", with its duration and its "you" (101.2, 611.2a, 611.2c) | `RegisteredRestriction` | yes | a target that left and came back (400.7); "for as long as this creature remains on the battlefield" (Suncleanser) | main item 10; the next row |
| **The object a "for as long as" watches** (611.2b) | nowhere: a resolution's row records `source: ctx.source`, the resolving stack object | **no** | a flicker ends it (400.7); leaving before the resolution means it never starts (Sower of Temptation's ruling); a change of control ends it for good (Dragonlord Silumgar's ruling, and suspend's haste, 702.62a) | **item 17, sharpened.** 78 cards, nearly all resolutions: `o:"for as long as ~ remains on the battlefield"` 27, `o:"for as long as you control ~"` 51 |
| When a row ends (514.2, 611.2a) | `DurationRegistry`'s two expiry hooks | yes, for the turn-scoped arms | a step- or phase-scoped duration (500.4, 500.5) | owned: `backlog.md` §2.12 |
| This turn's phases, and the natural rotation (500.1, 500.7, 500.8) | `TurnPlan`, `turn_rotation` | yes | a skipped turn advances the rotation, and an extra turn does not (614.10a, 500.7) | no gap; extra steps are `backlog.md` §2.17 |
| **A turn being an extra one** (500.7) | popped off `turn_queue` by `next_turn_taker`; not on `GameAction::BeginTurn` | **no**: gone before the proposal | "if a player would begin an extra turn" (614.10): Stranglehold, Ugin's Nexus, Gerrard's Hourglass Pendant, Trouble in Pairs (`o:"begin an extra turn"`) | **`triggers-architecture.md` §3.9 amended** |
| Which extra turn "that turn" is (500.7, 603.7) | planned: §3.9's `ExtraTurnId` | planned | a skipped turn fires nothing (614.10a; Alchemist's Gambit's ruling) | no gap. "During that turn" (Alchemist's Gambit, Kang the Conqueror) is a duration on the same id; Emrakul, the Promised End's "after that turn" waits for CR 722, controlling another player |
| The game's end (104) | `result`, beside `player_lost` | yes, by the performer or at the batch's settlement | 104.2a and 104.4a at settlement; 104.3f | no gap for v1. 104.3f is on `replacement-architecture.md`'s "Out of RE" list; a team's win and a draw for some players (104.2c–d, 104.3h, 104.4d–h) are Phase 9's |
| A player's starting life (103.4) | `starting_life` | yes | "your starting life total" (Exquisite Archangel) | feature: one number is right while every seat starts equal, as in every v1 format. Archenemy and Vanguard (103.4b, 103.4e) give a seat its own |

**Then the CR lines**, in two sets. Both start from §4a's base grep and drop
the sections passes 1–3 owned, `grep -vE '^(61[456]|603|10[67]|11[78]|60[12]|707)\.'`:
- **73 lines no pass had read:** "chosen", and none of the other six phrases.
- **137 lines passes 1–3 had read for their own subsystem only:** any of the
  other six.

**CR 702's 77 lines, 16 of the first set and 61 of the second, are keyword
breadth**, the way §6 retired `audit --dark`. The pass swept only a line that
names something an object or a player keeps, as 702.82b's "it devoured" and
702.138b's "escaped" do:
- gift's chosen opponent (702.174a): the last row of the next table;
- suspend's haste "until you lose control of the spell or the permanent it
  becomes" (702.62a): item 17's control leg;
- cipher's encoding, which survives a change of control (702.99c): a linked
  record, `backlog.md` §2.2;
- tribute's "if tribute wasn't paid" (702.104b): an entry's own fact, §2.2;
- soulbond's pair, which ends with either creature's control, type or zone
  (702.95a): §2.6;
- a foretold or a plotted card (702.143c, 702.170d): §4's first row's flag,
  plus the turn it happened, carried into the cast at 601.2a because the spell
  is a new object;
- morph's and disguise's X (702.37f, 702.168e): `PermanentState.x_value`,
  written by the action that turns the permanent face up (CV-6).

**Combat (508, 509) was re-read rather than swept**, since RS-3 owns its
restrictions. `AttackingInfo.target` records 508.1b's choice, and `is_blocked`
keeps 509.1h's "remains blocked". TR-5's item 11 puts each declared attacker's
defender on the `AttackersDeclared` record, and that record is what 508.7a's
"still considered to have attacked" and 802.2a's "before it was removed from
combat" read. A creature put onto the battlefield attacking (508.4) has no
record, and no card reads its target after it leaves combat (`o:"onto the
battlefield attacking" o:"defending player"` 0, `o:"removed from combat"
o:"defending player"` 0).

**The rest were owned already:** targets (115: `chosen_targets` holds the
announcement that 115.9a and 115.9c count; main item 10), modes (700.2: item
31), card names (201.4: `backlog.md` §2.4), who can see what (101.4a, 406.4:
§2.9), casting from other zones (400.7g–i, 715.3d: §2.3), a source of your
choice (609.7a: pass 1), split cards and prototype (709, 718: `CardData`'s
known fact), a departed player's last known information (800.4i: "Before
Commander" item 4), and setup and the casual variants (103.1, 103.2, 103.6b,
123, 717, 728, 800.5, 801, 805, 807, 810, 901). **One feature has no owner
yet:** 307.5a's "cast any time a sorcery couldn't have been cast" is a fact
about the cast, and `CastFacts` gains it with its first card (`o:"couldn't
have been cast"` 10, Necromancy among them).

The lines gave two facts. The first began as a lead pass 3 left: God-Eternal
Kefnet copies a revealed card, and if the card leaves the hand first, its
ruling says "you'll copy it using its last known information".

| Fact | Where it lives | Recorded when it exists? | The rules that watch it | Verdict |
|---|---|---|---|---|
| **The last known information of a source or a bound object that left a zone other than the battlefield** (113.7a, 608.2h) | planned: §6.1's `departed` frame, written by `capture_departure_frames` | **no**: that frames what leaves the battlefield, and TR-4 widens it to 603.10a's three classes only | a card revealed in a hand leaves before the trigger resolves (God-Eternal Kefnet's ruling); a spell is countered before its copy trigger resolves, and the copy is still made (Double Vision's and Galvanic Iteration's rulings); 603.10e's `IsCountered` look-back | **item 169, sharpened; `triggers-architecture.md` §6.1 and §3.11 and `copy-effects-architecture.md` §4.4, amended.** `(o:"when " or o:"whenever ") o:"copy that spell"` 70; with `"copy that card"`, 3 |
| **The player a payment chose** (702.174a) | nowhere: `CostChoices.additional` holds the cost definitions paid | **no** | a copy of the spell keeps the opponent, and a permanent that enters as a copy does not (Into the Flood Maw's ruling), which is CR 707.10's side | **item 30, sharpened.** `kw:gift` 25. Behold's objects (701.4, `o:"behold"` 24) join the list |

**Three of pass 4's four findings are the re-sweep's shape again**: a fact
that exists at one moment and is read later from a place that does not hold
it. The object a "for as long as" watches is known at the resolution and never
written down. A turn's being an extra one is known when the queue is popped,
and gone by the proposal. The player a payment chose is gone when the cast
completes. The fourth is pass 2's shape: a planned frame, written before the
rule that reads it from every zone reached it. None is reachable today.

**The follow-up list**: three wrong citations, small enough to fix on sight.
All but one sit in `src/` comments, so they wait for a code PR, and the one in
a plan (main item 18) goes with its twin in the source.
- `PlayerState.counters`' doc says CR 613.7c timestamps counters on objects.
  The rule says "an object or player". Its conclusion, that no layer reads a
  player's, stands.
- Six CR 103 citations in `src/` are one off against `tmnt.txt`. The starting
  life total is 103.4, not 103.3 (`game_state.rs:373`, `types/effects.rs:119`,
  `resolve.rs:2020`). Opening hands are 103.5, not 103.4 (`turns.rs:51`). The
  starting player is 103.1 and their first turn 103.8, not 103.7
  (`game_state.rs:329`, `game.rs:118`).
- `remove_from_combat`'s doc (`game_state.rs:1467`) and main item 18 cite CR
  506.4b for "remains blocked", which `tmnt.txt` puts in 509.1h. The quoted
  wording is an older CR's.

---

## 4b. `GameState`'s fields, and where each was read

`plans/check_type_surface.py` fails a pull request whose `GameState` holds a
field this document does not name: §4's promise, made a gate on 2026-09-24 at
the owner's call. A field is named here once §2's question has been asked of
it, beside the pass that asked or the reason it holds nothing. The list itself
is derivable; where each name was read is not, and the gate needs the names.

- **The 27 fields `GameState` had at `3c322e5` and still has**, under §4's
  first table and the remainder it dispositioned: `objects`, `players`,
  `stack`, `stack_entries`, `resolving`, `battlefield`, `exile`, `command`,
  `turn_number`, `last_turn_began`, `active_player`, `priority_player`,
  `phase`, `attacks_declared`, `blockers_declared`,
  `blocker_damage_divisions`, `dealt_first_strike_damage`, `next_timestamp`,
  `player_lost`, `skip_first_draw`, `continuous_effects`,
  `replacement_effects`, `replacement_ability_sources`,
  `next_zone_change_epoch`, `last_sba_check_epoch`, `events`, `rng`. The
  re-sweep read several again; §4's second table says which.
- **The 25 added since**, by the pass of §4a that read them:
  - pass 1: `entry_selection`, `prevention_allocations`, `rider_lineage`;
  - pass 2: `pending_triggers`, `look_back_snapshots`, `departure_frames`;
  - pass 4: `restrictions`, `turn_plan`, `turn_queue`, `turn_rotation`,
    `result`, `starting_life`;
  - no fact about the game (pass 3's remainder): `cost_modification_ability_sources`,
    `restriction_ability_sources`, `trigger_sources`,
    `zone_replacement_ability_sources`, `zone_trigger_sources`, `layer_epoch`,
    `layer_memo`, `diagnostics`, `dispatch_audit`, `trace`, `nesting`,
    `next_trigger_seq`, `next_object_id`.

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
carried `StackEntry` → `PermanentState` on resolution. This rides it, and
the rail survives RC-2's ETB rewrite either way.

→ **Owner: `codebase-state.md` Deferred Migrations item 30. Capture is
  independent of every scheduled phase and cheap whenever. The design
  constraint lands at CV — the 707.10 split above — which is where the
  back-stop belongs.** The mana half was captured by type 2026-09-23; a
  unit's source needs a per-unit pool and moved to item 33.

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

### 5.4 The re-sweep's findings (2026-09-24, §4a)

- **The act of a redirected zone change.** `Rewrite::Instead(ZoneChangeTo)`
  writes its own `cause`, so a discard or sacrifice that Rest in Peace or
  Leyline of the Void redirects is performed as `Exiled`. The record also
  cannot name the replacement that redirected it, which madness's trigger
  needs. → `codebase-state.md` item 176, reachable and wrong today.
- **References held across a move: a fourth kind.**
  `RegisteredReplacementEffect.targets`, which item 90's rider reads. →
  main item 10, sharpened.
- **The linked-ability records an entry makes**: an "as it enters" choice, and
  what the entry's own zone change chose. Both exist only at the entry. →
  `backlog.md` §2.2, which gains the constraints.
- **The object a move made** (pass 2). The `ZoneChange` record lacks the epoch
  the move stamped, so a binding reads it at dispatch, and "them" cannot
  read it at all. → `codebase-state.md` item 177.
- **A departed permanent's cost decisions** (pass 2). §3.11's frame kept
  `cast` only. → item 169, sharpened; `triggers-architecture.md` §3.11,
  amended.
- **The intervening "if" and a newcomer, read after a rider** (pass 2).
  → item 175, sharpened.
- **CR 603.2h's gate keyed without its controller** (pass 2). →
  `triggers-architecture.md` §3.5, §6.4, §7, amended.
- **What a copy of a spell gets** (pass 3). CV-4's planned `StackEntry` clone
  would keep `cast_from`, `mana_spent` and `controller`. →
  `copy-effects-architecture.md` §4.4, amended.
- **Storm's count, read at resolution** (pass 3). →
  `triggers-architecture.md` §3.10, amended.
- **The objects that paid** (pass 3). The list belongs with what a copy
  keeps, and holds identities rather than a count. → item 30, sharpened.
- **Last known information off the battlefield** (pass 4). §6.1's
  `departed` frame is written only for what leaves the battlefield, so a
  revealed card that leaves a hand, or a spell countered before its copy
  trigger resolves, has none. A spell's frame also lacks the decisions CR
  707.10 copies. →
  item 169, sharpened; `triggers-architecture.md` §6.1 and §3.11 and
  `copy-effects-architecture.md` §4.4, amended.
- **The object a resolution's "for as long as" watches** (pass 4). The row's
  `source` is the resolving stack object, so item 17's fix, keyed on it, never
  finds the permanent. → item 17, sharpened.
- **A proposed turn does not say it is an extra one** (pass 4), which four
  "would begin an extra turn" replacements read. → `triggers-architecture.md`
  §3.9, amended.
- **The player a payment chose** (pass 4): gift's opponent, which a copy of
  the spell keeps. → item 30, sharpened.

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
- **Settled 2026-09-24: the re-sweep.** Pass 4 (§4a) closed it, and
  `check_type_surface.py` holds §4's promise from here on (§4b).
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
