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
  keyword family) is already filed at Phase 8, correctly. See §4's second note.
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
- **Atoms** — 24, **not re-filed** — see §3's note on the `owed` collision.
  **14 of the 31 atoms `owed` currently reports are this section.**
- **Owner** — none yet.

### 2.6 CR 702 keyword abilities

- **Rules** — CR 702, the evasion/combat and static-ability half: menace
  (702.111), shroud (702.18), protection (702.16), improvise (702.126), flash
  (702.8), impending (702.176), warp (702.185), and first/double strike
  (702.4, 702.7)
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
- **Blocks** — nothing structural. Protection also needs §2.8's SBA legality
  re-check for Auras and Equipment.
- **Atoms** — 20, not re-filed.
- **Owner** — none yet.

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
  it reads a *partial* layer result, after L1–L3 but before L4–L7.
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
- **Blocks** — flash and every "activate only as a sorcery" ability; the
  once-per-turn restriction that must survive a controller change (602.5b);
  §2.3, in the sense that it is where those keywords will be expressed.
- **Atoms** — 14 across CR 602 (9) and CR 604 (5), not re-filed.
- **Owner** — none yet.

### 2.9 The information model — who can see what

- **Rules** — CR 400.2 (public vs hidden zones), 401.2/401.3 (library), 402.3
  (hand), 404.2 (graveyard)
- **Verdict** — **nothing in the tree answers "can player N see this object?"**
  A grep of `mtgsim/src` for a visibility predicate returns exactly one piece of
  hidden-information state: `BattlefieldEntity.face_down: bool`. Zones carry an
  identity but not a visibility, and no query takes a viewing player at all.
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
- **Atoms** — 4 here, plus **CR 400.2, which slice 1 filed under §3.2 in
  error**: "public zones are zones in which all players can see the cards" is
  this entry, not a zone guard. Corrected in §3.2's table.
- **Owner** — none yet.

### 2.10 Colour is a derived characteristic, and the engine stores it

- **Rules** — CR 202.2 (colour from the mana cost), 105.2, 105.3
- **Verdict** — `engine/layers/compute.rs:99` seeds Layer 5 with
  `colors: card.colors.clone()` — **the stored field, never the mana cost.**
  CR 202.2 makes colour *derived* from the mana symbols, so an authored card
  whose `colors` and `mana_cost` disagree is silently wrong and nothing can
  notice. `color_indicator` still has no reader (audit §5.2, Deferred
  Migrations), and `is_monocolored` / `is_multicolored` / `is_colorless` do not
  exist at all.
- **Size** — small-to-medium, and a **feature by the audit's test**, not a fact:
  `EffectiveCharacteristics.colors` is already the right type and Layer 5
  already applies to it, so the change is the seed plus a derivation function.
  It pairs naturally with §2.1 — reading a hybrid or Phyrexian symbol for its
  colours is the same symbol-decoding work as paying with one.
- **Blocks** — devotion (§2.7's CR 700.5a); protection from a colour (§2.6);
  every colour-matters card; the colour indicator's CV-5 landing.
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
- **Blocks** — every planeswalker card, which is a whole card type.
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
- **Owner** — none yet.

### 2.13 Deck-construction limits are configured and unenforced

- **Rules** — CR 100.2a, 100.2b, 100.4a
- **Verdict** — `DeckLimits` is **fully modelled and never consulted.**
  `GameConfig` carries `min_deck_size`, `max_deck_size`, `max_copies` and
  `sideboard_size`, and `standard()` and `limited()` both set them correctly
  (60/4/15 and 40/none/none). **No validator exists** — nothing in the tree
  reads them. Configuration with no consumer, which is the §2.7 `chosen_modes`
  shape again in a milder form.
- **Size** — tiny; one validation function against a `Decklist`.
- **Blocks** — nothing in play. It matters for a deckbuilding UI and for
  refusing a malformed decklist rather than starting a broken game.
- **Atoms** — 7. **This is a candidate for `codebase-state.md` Deferred
  Migrations instead of a backlog entry** — it is unconsumed scaffolding, not an
  unbuilt mechanic — and is filed here only because the whole cluster surfaced
  together. Move it if that reads better.
- **Owner** — none yet.

---

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
| 609 | 2 | `AffectedSet` defaults — audit §4 found no gap here |
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

### 3.3 The `owed` collision — a policy question, not a triage one

**None of §2.5–§2.8's atoms were re-filed to `Backlog`, deliberately.**
`owed` reports 31 atoms, and **27 of them sit in slice-1 sections — 14 in
CR 701 alone.** Re-filing this slice the way D3a re-filed the cost pipeline
would take the gate from 31 to 4.

D3a's re-file was justified because those atoms' shipped-phase filing was
*wrong* — hybrid mana payment was never shipped, so `owed` shrinking was a
consequence of a correction, not its purpose. Here the filing is not obviously
wrong in the same way: a `NEW` ticket under a shipped phase is precisely what
`owed` exists to surface, and an atom can honestly be both "a shipped phase
promised this" and "here is the mechanic that would deliver it".

**So this is a question about which register owns a `NEW`-ticketed atom, and it
moves a gate the project reads.** It is left open rather than settled in
passing — silencing a gate as a side effect of triage is the same error §1
refuses for `orphaned`.

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

**All 63 sections are triaged.** The query still reports 297 atoms across 63
sections and will keep doing so: **re-filing is not how a section leaves that
list, and §3's dispositions are prose the query cannot read.** Use it as the
source, not as a progress bar — and regenerate it before trusting any number
here, since the corpus moves.

Thirteen entries and 180 dispositioned atoms came out of it. Six things learned,
recorded so they are not re-derived:

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
  need it.
