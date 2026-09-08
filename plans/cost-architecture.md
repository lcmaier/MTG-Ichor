# Costs — CR 601.2f–h, 118.7–118.9, 613.11's cost half, 903.8

> **Status:** design, authored 2026-09-07 and revised the same day against the
> owner's review (seven notes and a judge's walkthrough of the Ironworks loop;
> each is answered where it lands and listed in §11). CM-0 and CM-1 built.
> **Authority:** the cost pipeline — what a cost modification *is* in this
> engine, which objects can have one, where one is discovered, the CR 601.2f
> order as built, the lock-in, and the phase sequencing for CR 601.2f–h /
> 118.7–118.9 / 613.11's cost half / 903.8. Where this contradicts
> `codebase-state.md`, that file wins on *what exists*; this file wins on
> *what is being built*. `CLAUDE.md` → "Critical path to v1" still owns the
> ordering against the other phases (this is the "cost modification" the
> Commander interleave names).
> **Companions:** `cant-effects-architecture.md` owns CR 613.11's *other*
> half — "all other such effects are applied in timestamp order" — and
> `backlog.md` §2.15 owns the player-scoped values that half applies to
> (`max_hand_size`, `lands_per_turn`). `replacement-architecture.md` §3.3 and
> `cant-effects-architecture.md` §3.4 are the discovery pattern §3 here
> reuses; read them before §4. `layers-architecture.md` §11.2 said "Layers
> don't touch it" and was right — this document is the one that does.
> **Graduates:** `backlog.md` §2.1 (cost modification and the cost pipeline),
> both halves: modification is CM-1–CM-4, payment is CP-1 (§6), a sized slot
> in this document rather than a design.
> **Supersedes:** the cost half of ticket **L15**, whose `TODO(L15)` sat on
> `engine/costs.rs::apply_cost_modifications` from 2026-04 to 2026-09-07.

---

## 0. Where the design lives, and why it is a document

The prompt for this phase asked for one decision before any other: a section
in an existing document, or a document of its own. **The sizing decides it
(§6): five PRs and a slot, so a document.** The reasoning, so it does not get
re-litigated:

- **The two documents that could host a section each disclaim the rule.**
  `layers-architecture.md` §11.2: "Cost modification (CR 601.2f) isn't part of
  the layer system … Layers don't touch it." `cant-effects-architecture.md`
  claims CR 613.11 on the strength of its *timestamp* half and says of the
  cost half, in its §3.6: "'this effect can't reduce the cost below one mana'
  is CR 601.2f's pipeline and belongs to the cost-modification phase". A
  section inside a document that says "not mine" is a section nobody finds.
- **CR 613.11 delegates, so the two halves have different owners.** "Continuous
  effects that affect the costs of spells or abilities are applied according
  to the order specified in rule 601.2f. All other such effects are applied in
  timestamp order." The first sentence is this document. The second is the
  restriction model's for prohibitions and `backlog.md` §2.15's for
  player-scoped values. `CLAUDE.md`'s authority row lists both owners against
  613.11 and that is correct, not a conflict.
- **`backlog.md` §1 makes graduation the normal path** — "design claims the
  rule; listing it does not" — and §2.1 has been the largest un-owned entry
  since the backlog was written. It said "not small, wants splitting" and
  named the split: representation-and-payment versus modification. This
  document takes both, sizes each, and sequences them (§6), which is what
  lets §2.1 graduate whole rather than half.
- **`CLAUDE.md`'s authority table gains no row.** The `plans/*-architecture.md`
  row's list is extended in place — `cost` (601.2f–h/118.7–9/613.11 cost
  half/903.8, `CM-*`/`CP-*`) — which is what keeps that table fixed-size
  (commit ca3a685). The file was at 200/200 lines when this was written; the
  edit adds none.

Phase codes: `CM-0` (a rename the first consumer forces), `CM-1`–`CM-4`
(**c**ost **m**odification), `CP-1` (cost **p**ayment). Branches are
`cost/cm-<n>-…`.

---

## 1. The rule, verbatim

> **601.2f** The player determines the total cost of the spell. Usually this
> is just the mana cost. Some spells have additional or alternative costs.
> Some effects may increase or reduce the cost to pay, or may provide other
> alternative costs. Costs may include paying mana, tapping permanents,
> sacrificing permanents, discarding cards, and so on. The total cost is the
> mana cost or alternative cost (as determined in rule 601.2b), plus all
> additional costs and cost increases, and minus all cost reductions. If
> multiple cost reductions apply, the player may apply them in any order. If
> the mana component of the total cost is reduced to nothing by cost reduction
> effects, it is considered to be {0}. It can't be reduced to less than {0}.
> Once the total cost is determined, any effects that directly affect the
> total cost are applied. Then the resulting total cost becomes "locked in."
> If effects would change the total cost after this time, they have no effect.

> **613.11** Some continuous effects affect game rules rather than objects.
> For example, effects may modify a player's maximum hand size, or say that a
> creature must attack this turn if able. These effects are applied after all
> other continuous effects have been applied. Continuous effects that affect
> the costs of spells or abilities are applied according to the order
> specified in rule 601.2f. All other such effects are applied in timestamp
> order. See also the rules for timestamp order and dependency (rules 613.7
> and 613.8).

> **113.6d** An object's ability that allows a player to pay an alternative
> cost rather than its mana cost or otherwise modifies what that particular
> object costs to cast functions on the stack.
> **113.6e** An object's ability that restricts or modifies how that
> particular object can be played or cast functions in any zone from which it
> could be played or cast and also on the stack. …
> **702.41a** Affinity is a static ability that functions while the spell with
> affinity is on the stack. "Affinity for [text]" means "This spell costs {1}
> less to cast for each [text] you control."

> **601.2h** … *Example:* You cast Altar's Reap, which costs {1}{B} and has an
> additional cost of sacrificing a creature. You sacrifice Thunderscape
> Familiar, whose effect makes your black spells cost {1} less to cast.
> Because a spell's total cost is "locked in" before payments are actually
> made, you pay {B}, not {1}{B}, even though you're sacrificing the Familiar.

Four consequences that shape everything below:

1. **The order is the rule's, and it is total.** Base, plus additional costs
   and increases, minus reductions (player-ordered), floored at {0}, then the
   effects that "directly affect the total cost", then locked. 613.11 adds
   *when*: after every object's characteristics are final. No timestamp, no
   dependency, no layer — the frame a cost effect reads is a finished one.
2. **The "directly affect the total cost" clause is one printed card's.**
   Trinisphere (Darksteel, 2004) is the only card whose text is "costs three
   mana to cast" (Scryfall, 2026-09-07: one hit), and the sentence exists for
   it. Its own rulings restate 601.2f's order in full and add the lock-in
   ruling §3.3 tests: "If Trinisphere leaves the battlefield or becomes
   tapped or untapped as a cost to cast a spell, this cost is paid after
   you've locked in the total cost." An arm for one card is still an arm — it
   is the third position in the order, and a custom card can take it — but it
   is not a family, and nothing about it generalizes.
3. **Lock-in is a property of the pipeline's shape, not a flag.** The total is
   computed once at 601.2f and paid at 601.2h; nothing between them re-reads
   the board. The example makes it observable: a reducer that leaves the
   battlefield *as a cost is paid* has already been counted — and so has one
   that leaves during 601.2g's mana window, which is the Ironworks board
   (§3.11).
4. **Cost abilities do not all live on the battlefield.** CR 113.6 makes the
   battlefield the default, and 113.6d/e make a spell's *own* "this spell
   costs … less" and affinity function on the stack and from any zone it can
   be cast from. So the objects a cost determination consults are not "the
   permanents" but "the objects whose cost abilities function now" — today
   the battlefield plus the spell itself (§3.1, §4); emblems and graveyard
   cost abilities arrive with `roadmap-v2.md` A5's zone-function predicate.

---

## 2. What the engine had (2026-09-07, before CM-1)

- `engine/costs.rs::assemble_total_cost` builds base-or-alternative, expands X
  into generic, appends chosen additional costs, and calls
  `apply_cost_modifications`, **a passthrough with a test asserting it is
  one**. One production caller (`cast.rs`, at 601.2f), ten mentions in
  `costs.rs`'s own tests.
- Nothing registered modified a cost. No `Effect` arm, no `ChoiceKind`, no
  row kind, no gate. `castable_spells` (`oracle/mana_helpers.rs`) read the
  printed mana cost to decide what a player may cast.
- Two latent defects found by reading the pipeline for this design, both
  fixed by CM-1's merge step (§3.3): a kicked spell's kicker mana was a
  *second* `Cost::Mana` in the total, and `choose_generic_allocation` and
  `run_mana_ability_window` both took the *first* `Cost::Mana` only — so a
  kicker with a generic component was allocated against the base cost's
  split, and the mana window stopped at the base cost.
- `run_mana_ability_window` returns the moment the pool covers the cost —
  engine policy, not the CR's (§3.11, §8 item 3, CM-4).
- `StackEntry.cast_from` is recorded before 601.2f (CR 903.8's fact, §3.8).
- `Condition` has a static evaluator (`engine/layers/condition.rs`, LI-3)
  and a test-only settled-board wrapper; `AmountExpr::SourcePower` has one
  evaluator and two refusals (`codebase-state.md` main item 57).
- `PermanentFilter` is the engine's only characteristic filter, and every
  matcher of it is written for a permanent (`targeting.rs`, `compute.rs`).

---

## 3. The model — the decisions, in the order they were asked

### 3.1 A cost modification is discovered, not registered — and only sources are consulted

**A static ability whose body is `Effect::CostModification` generates no
layer row.** It is discovered at 601.2f by reading its object's *effective*
ability list — the third static shape to use the pattern `Effect::Replacement`
(RB) and `Effect::Restriction` (RS-1) established, and for the same reasons:

- **The layer-system invariant.** A Thalia under Humility stops taxing, and a
  Layer 6 grant of "spells cost {1} more" taxes. Reading the effective list
  *is* CR 604.2's existence check; nothing has to be reconciled.
- **A registry row is a layer-walk input.** Every `ContinuousEffect` names a
  `Layer` and an `AffectedSet` of *objects*, and `compute_characteristics`
  applies it to a frame. A cost effect has no layer — 613.11 puts it after
  all of them — and applies to no object; it applies to a *cost being
  determined*. A row nothing in the walk applies is a list wearing a
  registry's name, and the CDA lesson (`CLAUDE.md`) is that one ability
  reaching an answer through two channels is how it applies twice.
- **`codebase-state.md` "Before Layers" item 3 said "a `CostModification` row
  kind wired to the continuous-effects registry"** on 2026-08-24, before RB
  built the discovery pattern (2026-08-25) and before RS-1 reused it
  (2026-08-31). That sentence is superseded by this section; item 3 says so.

**Which objects are consulted, and what a consultation costs.** Not every
permanent — the *sources*. `GameState::cost_modification_ability_sources` is
the set of battlefield objects that printed a cost ability (recorded at ETB by
`register_static_effects`, removed by `cleanup_zone_state`), and the sweep is
that set sorted by `PermanentState::timestamp` — `O(k log k)` in the number of
sources, and on the common board `k = 0` and the gather is a
`HashSet::is_empty`. Each source costs one `get_effective_abilities`, which is
the memoized top-level frame (7a), not a walk.

The set can only *under*-approximate through two routes it cannot see, and
each has a registry-wide flag: `RegistryScopeSummary::any_granted_cost_
modification` (a Layer 6 `GrantAbility` whose body is one) and
`any_copied_cost_modification` (CR 707.2a, a `CopyFrom` whose captured list
has one). **When either flag is on, the sweep widens to every permanent**,
because the flag is not attributed to an object — attributing it would mean
resolving the grant's or copy's affected filter per permanent per cast, which
is what the flag exists to avoid, and `gather.rs` makes the same trade for the
same reason. Both are the rare board. So the reading "a sweep over every
permanent per cast" was the widened case described as the default, and this
paragraph replaces it. **A new gather source, or a new route to the effective
list, needs a leg on every gate** — `CLAUDE.md`'s rule, now three gates wide.

**The spell itself is a source too** (CR 113.6d, 702.41a) — **CM-2, built**.
"This spell costs {1} less to cast for each …" is 289 printed cards and
affinity is 75 more (Scryfall, 2026-09-07), against 249 battlefield reducers
and 9 increases with "costs … more". It is discovered off the spell's own
effective ability list with `CostSubject::Itself`, an identity test no
permanent can satisfy, which is what partitions the two sources with no flag
passed anywhere.

**Which zone, and the correction CM-2 made.** CR 113.6d says an ability that
"otherwise modifies what that particular object costs to cast" **functions on
the stack**, full stop — no zone list — and CR 702.41a says the same of affinity
by name ("functions while the spell with affinity is on the stack"). 113.6e's
"any zone from which it could be played or cast" is the neighbouring rule, for
abilities that restrict or modify *how* an object is cast, and this document
cited it here and in §3.6 for a cost ability, which was wrong. Nothing in the
code turned on it: the castability preview reads the hand frame because
enumeration has to agree with enforcement (§3.6), which is an engine
requirement and not a rules permission. The CR never has to answer "what would
this cost if I cast it"; the engine does.

**And it is gated, which this section also had wrong.** "It needs no gate: one
frame per cast, already computed for the filter match" assumed a frame that is
not there — the gather returns *before* computing one when no cost source is on
the board, which is the common board. Ungated, source 2 puts one
`compute_characteristics` on every card in hand at every castability preview,
forever. So the third gate gets a third leg: the spell's own printed abilities,
OR'd with the two summary flags. It is exact today, because `compute_non_member`
applies no registry rows — a non-member's effective ability list *is* its
printed list, so neither a Layer 6 grant nor a Layer 1 copy can reach a spell on
the stack or a card in hand (§8 item 7). **The leg asks the subject, not the
body**: Thalia prints a cost ability and a Thalia in hand modifies nothing of
her own, and asking the body alone cost five non-member walks per 200 measured
games — found by the A/B arm, not by argument (§8 item 8).

**Other zones** — an emblem's "spells you cast cost {1} less", Convergence of
Dominion's graveyard abilities — are `roadmap-v2.md` A5's zone-function
predicate, which is what answers "which abilities function from where" for
every static shape at once; this document adds a source when that exists
rather than a graveyard sweep now.

**One thing the other two gates get wrong and this one does not.**
`register_static_effects` inserts a source when `ability.effect` *is* an
`Effect::Replacement` or `Effect::Restriction`; a conditional one —
`Effect::Conditional(cond, Replacement)` — is never inserted and the gather
never sees it, so a conditional replacement or restriction static is inert
today (no registered card has one; §8). Trinisphere prints exactly that shape
— "as long as this artifact is untapped" — so this gate sees through the
wrapper: `Effect::as_cost_modification` peels `Conditional` and returns the
condition beside the definition, and all three legs use it.

### 3.2 The type surface

`types/cost_modification.rs`, data only, like `types::restriction`:

```rust
pub struct CostModificationDef {
    /// Which spells or abilities the effect applies to.
    pub applies_to: CostSubject,
    /// What it does to the total cost, and so where in CR 601.2f's order it applies.
    pub change: CostChange,
}

pub enum CostSubject {
    /// Spells whose characteristics match `filter`, evaluated against the
    /// spell's frame with "you" resolved to the source's *current* controller
    /// (CR 109.5); the spell's own controller is its caster (CR 601.2a).
    Spells(ObjectFilter),
    // Itself — the spell this ability is on (CR 113.6d/e, 702.41a): CM-2.
    // ActivatedAbilities(ObjectFilter) — §3.10, with its first consumer.
}

pub enum CostChange {
    /// "cost {N} more to cast" — a cost increase. Symbols are added to the mana
    /// component as printed: {1} adds generic, {W} adds a white pip.
    Increase(ManaCost),
    /// "cost {N} less to cast" — a cost reduction, applied under CR 118.7a–d.
    Reduce(ManaCost),
    // ReduceGeneric(AmountExpr) — "cost {X} less, where X is …": CM-2, with
    //   the evaluator §3.7 argues for.
    // Reduce { not_below } — "can't reduce the mana in that cost to less than
    //   one mana", a clamp on one reduction's application, printed on 8 cards,
    //   all of them activated-ability reducers: with that subject (§3.10).
    /// CR 601.2f's "effects that directly affect the total cost" — Trinisphere,
    /// and only Trinisphere (§1). Raises the mana component's mana value to N
    /// with generic mana, after every increase and reduction, never lowers it.
    TotalAtLeast(u8),
}
```

**`ObjectFilter`, and the rename that has to come first (CM-0).** The leaves
are characteristic predicates — type, subtype, supertype, color, controller —
and a spell on the stack has all of them (CR 601.2a). But the type is named
`PermanentFilter`, every matcher of it documents a permanent, and spells and
permanents are disjoint sets: `Spells(PermanentFilter)` does not make nominal
sense, and this phase is the first consumer to apply the filter to something
that is not a permanent. `roadmap-v2.md` A5 already scheduled the rename
("preceded by the `PermanentFilter → ObjectFilter` rename as its own
zero-behaviour PR", layers item 9). **It moves ahead of CM-1 as CM-0**:
counted 2026-09-07, 275 occurrences in 25 `src/` files, 119 in 18 test files,
56 in live plan documents — a mechanical sweep with `cargo build` as the
check and no behaviour to test. The matcher that answers it,
`targeting::permanent_matches_filter_in_frame`, keeps its body and becomes
`object_matches_filter_in_frame`; `EachOther` and `PowerLE` answer for a spell as for any object;
`Token` and `ByOwner` read the `GameObject`, which a spell also is. The
**zone leaf** item 9 wants on the same type is not part of CM-0 — a rename
with a semantic change in it is two PRs wearing one name.

**Why three arms and not a `Vec<ManaSymbol>` delta.** They are the three
positions in 601.2f's order, and the order is the whole rule: an increase is
added before any reduction is subtracted, and the direct-total effect is
applied after the floor. A signed delta could not say which of the three it
was. **The commented arms are not in the enum until their phase** —
`CLAUDE.md`'s rule that an arm the pipeline cannot apply is worse than a
missing one applies to a field the same way — and they are written here so
the phase that adds each knows what §3.4's theorem says it expires.

### 3.3 The pipeline as built — `engine/cost_determination/`

`mod.rs` declares; `gather.rs` finds the effects that apply to this spell
(`cost_modifications_for`); `total.rs` owns the whole step as
`determine_total_cost` — the assembly of CR 601.2b's choices and the
arithmetic in one function, the one call `cast_spell` makes at 601.2f.
`engine::costs` keeps *payment* (601.2h). The steps:

1. **Merge.** Every `Cost::Mana` in the assembled list — the base or
   alternative cost, an X expansion, a kicker's mana — becomes **one** mana
   component. CR 601.2f speaks of "the mana component of the total cost" in
   the singular, and CR 118.8 says additional costs are paid "at the same
   time". The non-mana costs keep their order behind it; **the component goes
   first**, because CR 601.2h lets the first group's costs be paid "in any
   order", the engine picks one, and the mana payment is the one that can
   still fail after `can_pay_costs` (an illegal generic split) — a failure
   mid-list leaves what came before it paid. This is also what fixes the two
   latent defects in §2.
2. **Gather** (§4): every applying `CostModificationInstance` — source,
   definition, controller — sources in battlefield timestamp order, the spell
   itself last.
3. **Increases**, in that order. Addition commutes, so the order is for the
   log and nothing else.
4. **Reductions**, in the order the player chooses (§3.4), each under
   CR 118.7a–d: a generic reduction touches only generic (118.7a); a colored
   or colorless reduction removes its own pips first and its excess, or the
   whole amount when the cost has no such pip, comes off generic (118.7b–d);
   snow reduces generic (118.7g). Generic saturates at zero at every step,
   which is 601.2f's "can't be reduced to less than {0}" and "considered to be
   {0}". A `not_below` reduction additionally stops at its own floor — the
   mana value may not drop below N *by this reduction* — which is a clamp on
   the step and not on the total: a later, unfloored reduction may still take
   it lower (§3.4's worked example).
5. **Direct-total effects.** `TotalAtLeast(n)` raises the component's mana
   value to `n` with generic. Two Trinispheres agree, so the order is
   immaterial and none is chosen.
6. **Locked in.** The returned `Vec<Cost>` is what 601.2g's mana window is
   sized against and what 601.2h pays. Nothing after this point consults the
   board about the cost; the lock is that there is no second call.

Increase and reduction symbols that the payment half cannot represent yet —
hybrid, Phyrexian — are refused in debug (`debug_assert!`) and ignored in
release, never guessed at. §3.4 says why the reduction case is the one that
matters.

### 3.4 "In any order they choose" — the prompt, the theorem, and the middleware

**Two or more reductions apply → `ask_order_cost_reductions` asks the caster
to order them** (`ChoiceKind::OrderCostReductions`), and the pipeline applies
them in the order returned. One reduction has no order to choose and asks
nothing; that is `CLAUDE.md`'s "never prompt with fewer than two candidates",
which CR 601.2f states itself with "if multiple cost reductions apply". The
candidates are listed in battlefield timestamp order (the spell's own last),
so the index a `DecisionProvider` returns means the same thing in every
process (determinism), and the option shown is the *source*, since the
reduction is its text.

**The theorem: with the symbols the engine pays today and no floored
reduction, the order never changes the answer.** At 601.2b the player
announces the nonhybrid equivalent of any hybrid symbol in the cost, so by
601.2f the mana component holds only generic, colored, colorless and snow
symbols. Under 118.7a–d a reduction of a color takes that color's pips first
and spills its excess to generic; so for each color the pips removed are
`min(pips_c, Σ reductions_c)` and the generic removed is `Σ generic reductions
+ Σ_c max(0, Σ reductions_c − pips_c)`, floored at zero — every term a sum
over the set of reductions, not a function of their order, and sequential
flooring of subtractions equals one floor of their sum.

**Two expiry conditions, named here so the phases that admit them cannot
miss them:**

- **A reduction whose own amount is a hybrid symbol** (118.7e: "the player
  paying that cost chooses one half of that symbol at the time the cost
  reduction is applied"). No registered card has one; §3.3's refusal is where
  it would enter.
- **A `not_below` reduction.** Cost {3}; R1 is "{2} less, not below one mana",
  R2 is "{2} less". R1 first: {1} (clamped), then R2: {0}. R2 first: {1}, then
  R1 may not reduce below one and it is already there: {1}. Two outcomes. All
  eight printed carriers reduce *activated-ability* costs (§3.10), so the
  condition cannot fire in CM-1, and the arm that admits it lands with the
  card that needs it.

Three things follow, and all three are used:

- **The preview does not prompt and is exact** (§3.6): `castable_spells` applies
  the reductions in gather order and, by the theorem, reads the same total the
  cast will lock in whatever the player later answers. When a floored
  reduction is in play the preview applies floored reductions first, highest
  floor first, which is the order that minimizes the total for one floored
  reduction — a clamp binds against the largest remaining cost — and a stated
  heuristic for two, where a preview may only cost a rollback (over) or an
  offer (under), never an answer.
- **The cast path asks anyway**, because the CR makes it the player's choice
  and this phase measured nothing that says the prompt is noise: it needs two
  reducers and a matching spell on one board, and a `RandomDecisionProvider`
  answers it in one call. `pipeline::order_invariant_entry_bucket`'s elision
  (`codebase-state.md` item 47) was taken *after* a measurement — CR 616.1's
  prompt on every land drop under Root Maze — and carries expiry conditions.
  If this prompt ever shows up in a profile, the theorem above is the elision
  and the two conditions are its expiry; take it then, not now.
- **The engine keeps asking; a payer answers** — **built, CM-4, as two
  decorators rather than one.** The right home for "answer the payment prompts
  without bothering anyone" is a `DecisionProvider` decorator, not an engine
  shortcut. `ui::AutoPayer<D>` answers `GenericManaAllocation` (CR 601.2h) and
  `OrderCostReductions` (CR 601.2f); `ui::ManaWindowStop<D>` declines
  `ManaAbilityWindow` once the locked mana component is covered, which is where
  `run_mana_ability_window`'s stop condition went (§3.11, §8 item 3). That is
  `backlog.md` §2.18's "auto-payment oracle", and it is what a GUI's auto-pay
  button and an AI harness both want.

  **The sacrifice prompt is not in it, and this paragraph is where that was
  decided.** The sentence above listed it, written before CM-3 built the
  prompt. The criterion CM-4 settled on: **a payer answers a prompt when every
  legal answer leaves the same game state except for mana.** Mana is spent by
  the payment or emptied at end of step (CR 500.4) — it is not durable, so
  choosing among those answers is paying. Which creature to sacrifice for
  Altar's Reap leaves a different permanent on the battlefield and a different
  `ZoneChange` in the stream the trigger phase reads; a payer that answers it
  is playing the game rather than paying for it. The criterion is matched
  exhaustively over `ChoiceKind`, so CP-1's announcement prompt and item 72's
  reversal each have to pick a side. A client that wants an auto-sacrifice
  policy stacks its own decorator for that kind, in the AI harness or the GUI
  where strategy lives.

  **Two decorators, not one with a scope, and clients compose.** The first
  design gave one payer a `PayerScope` enum so each client could take a subset.
  A scope enum is a closed, hand-rolled enumeration of the subsets of something
  that already composes, and the second automation — priority passing,
  auto-block, an auto-tapper with lookahead — grows it an arm per subset. The
  stack invariant, written down before the third decorator arrives: **one
  decorator per `ChoiceKind`**, so composition commutes and stack order carries
  no meaning. `cli_play`'s two seats stack differently (the human takes both,
  the bot takes the stop alone) and that asymmetry lives in the binary rather
  than in `ui::`.

`ATOM-601.2f-004`'s expected result says "final cost depends on order if
reductions interact (e.g., reducing generic first vs. colored first)". The
example is wrong by the theorem — generic-first and colored-first commute —
and the atom's session file carries a note saying so; what the test proves is
that the prompt is asked, that both reductions apply, and that every order
gives the CR's answer.

### 3.5 Which frame a cost effect reads, and the condition on it

**A finished one.** CR 613.11 applies cost effects after all other continuous
effects, so `gather` reads each source through `get_effective_abilities` —
the memoized top-level frame, every layer applied — and the spell through
`compute_characteristics`, whose stack-zone frame is seeded from the
`StackEntry` (`compute::base_controller`), which `cast_spell` writes before it
reaches 601.2f. No dependency question arises and no layer ceiling is chosen:
the read is the same one `is_prohibited` makes, at the same moment-after-
everything, and that is stated here rather than left implied.

**A conditional cost effect** ("as long as this artifact is untapped") is
`Effect::Conditional(cond, CostModification)` on the ability, exactly LI-3's
shape, and `gather` evaluates `cond` through
`engine::layers::condition::settled_holds` — the settled-board reader LI-3
wrote as a test helper and this phase makes public, because a post-layer
consumer of `Condition` is what `CLAUDE.md`'s critical-path item 6 says the
trigger phase will add ("a reader, not a language"). It is a reader over
`Board::settled()` at the full layer ceiling, so every leaf answers as the
live pass would have at its last layer.

`Condition` gains one leaf, **`SourceUntapped`** — Trinisphere's clause, and
the same words open Winter Orb and Static Orb (Scryfall, 2026-09-07). It
reads `PermanentState.tapped`, a status no layer writes (CR 110.5), so
`board::condition_reads` declares nothing for it and it can never be a
CR 613.8 dependency.

### 3.6 Enumeration must agree with enforcement

`castable_spells` decides what the priority loop offers; `cast_spell` decides
what is paid. Before CM-1 the first read the printed cost and the second will
now lock a modified one, so a Thalia on the board would have offered spells
the cast then rolled back, and an Electromancer would have withheld spells the
player could afford — the disagreement `cant-effects-architecture.md` §4.3
names for RS-2, one phase early. `castable_spells` now previews the total
through the same `total.rs` (no prompt, §3.4's theorem) against the card's
in-hand frame, whose controller is its owner (CR 108.4a) and so the
prospective caster. **Nothing licenses that read but this section.** A cost
ability functions on the stack (CR 113.6d, 702.41a; §3.1), so the preview is
not claiming the card's affinity works in hand — it is predicting the total the
cast will lock, using the one arithmetic that will lock it. Enumeration and
enforcement agreeing is the requirement; the CR has no rule about it. `find_mana_sources` then reasons about the previewed
component as it did about the printed one.

### 3.7 The dynamic-amount evaluator, and `SourcePower`'s third reader — the argument, made once (built, CM-2)

`codebase-state.md` main item 57 warns that a cost-modification evaluator for
`SourcePower` would be its third, and that "the answer depends on which board
the caller is entitled to". The entry path's asymmetry (§5b of
`replacement-architecture.md`) exists because the *source* of an entry
replacement can be the entering object itself, whose frame is hypothetical.
**At 601.2f no such asymmetry is possible**: the source of a cost effect is
either a permanent on the battlefield or the spell on the stack, and in
neither case is there a hypothetical frame in play — 613.11 says the board is
finished, and the spell's stack frame is its real one. The evaluator reads
the source's memoized frame for `SourcePower` and the real battlefield for
`CountOf`, with "you" the source's controller — the caster, for the spell's
own ability — and nothing else is defensible. That argument is the whole of
the design.

**Built as a reader, not a third leaf table** (CM-2).
`engine::layers::compute::settled_amount` is `evaluate_amount` over
`Board::settled()` at the full ceiling — the one line
`condition::settled_holds` already was, and for the same reason. That board
*is* §3.7's two entitlements with nothing added: `battlefield_ids` returns the
real battlefield when the board is not live, and `frame_of` answers a member
from the memo. `CountOf` needed no new arm at all; with `origin: None` it
already resolves "you" to the frame's controller.

**So item 57 closes with "there is no third evaluator".** There is a third
*caller* of one leaf table. The sharper form of the item's warning — the answer
depends on which board the caller is entitled to — resolves into a statement
about which *object* the caller means by "source": the cost pipeline always
hands the evaluator **the ability's own source** (the permanent for a `Spells`
subject, the spell for `Itself`), so `object == source`, and `SourcePower`
would read `chars.power` with no cross-object read at all. The walk cannot say
that — there `object_id` is the *affected* object and `origin.source` is
somewhere else, which is the CR 613.8 dependency it refuses on purpose.

**`SourcePower` is answered in the reader and still refused by the walk**, and
the pair of tests is the argument above stated twice. The first draft of this
section shipped no arm at all, on the grounds that Golden-Tail Trainer waits for
item 6 — but that was an argument about the *card*, and
`engineering-practices.md` §3 lets a fixture with its own name be a consumer.
`phase_cm_cards::power_reducer` is that fixture, and its board is the
entitlement claim made observable: an anthem on the reducer makes it reduce
more, because the amount is read off the source's **effective** frame.

**What the arm is actually worth, counted before it was written** (Scryfall,
2026-09-07). "This spell costs {X} less to cast, where X is …" is **33** printed
cards. Of those, **2** read a creature's power, and one of the two — Maelstrom
Muse — is a trigger-created effect with a duration, so it is §3.10's
`Primitive::ModifyCost` and not this arm. `SourcePower` therefore serves
**one** printed static: Golden-Tail Trainer. The population is not the reason to
build it; the argument is.

**And the 33 are blocked on leaves, not on this one.** Only two of them are a
plain `CountOf` ("the number of …"); the rest want "your devotion to black",
"the total mana value of noncreature artifacts you control", "the greatest power
among creatures you control", "the number of differently named lands you
control". `ReduceGeneric` is built and general; what Phase 8 owes it is
`AmountExpr` leaves, one per shape, each with a static evaluator. Affinity's 75
cards are the `CountOf` population and they are covered.

**It landed in CM-2 with affinity**, not with Golden-Tail Trainer. Affinity is
`Itself` + `ReduceGeneric(CountOf(PermanentsMatching(…you control)))`
(702.41a), Myr Enforcer and Frogmite are vanilla bodies with nothing else on
them, and 75 + 289 cards is the population. Golden-Tail Trainer's arm
(`SourcePower` on a battlefield source) is the same evaluator with one more
match arm, and its card still waits: its second ability is an attack trigger
(critical-path item 6), and registering it with half its text would wear its
name while behaving differently (`engineering-practices.md` §3).

### 3.8 Commander tax — waits for designation, and what is ready for it

CR 903.8 is an **additional cost** in 601.2f's vocabulary ("costs an
additional {2} for each previous time …"), so it enters the pipeline in step 1
beside kicker, not as an increase — the distinction is invisible to the
arithmetic (additions commute and precede reductions) and visible to a reader
of the log. It needs two facts and one designation:

- *cast from the command zone* — `StackEntry.cast_from`, recorded at 601.2a
  and readable at 601.2f (nothing to thread);
- *how many times before* — a per-player counter keyed on the commander's
  `ObjectId`, which survives zone changes here (`move_object` keeps the id and
  bumps `zone_change_epoch`), incremented at 601.2i when the spell becomes
  cast and `cast_from == Command && is_commander`;
- *a commander* — `is_commander` is set nowhere in production ("Before
  Commander" item 2), and `check_cast_legality` admits only the hand.

The counter is a fact, and facts are recorded on their first customer
(`engineering-practices.md` §5). The first customer is the tax, and the tax
has no payer until `GameConfig::commander()` designates one and casting from
the command zone exists (`backlog.md` §2.3). **So it ships with `roadmap-v2.md`
B2**, as one arm in `total.rs` step 1 plus the counter, ~40 lines against this
pipeline. What this phase leaves ready: the pipeline takes the game, the
caster and the spell, so the arm has everything it reads at the call.

### 3.9 `lands_per_turn` is CR 613.11's other half, and stays in §2.15

`codebase-state.md` main item 13 assigned `lands_per_turn` to "the
cost-modification phase, the other CR 613.11 consumer". It is the other
*sentence* of 613.11 — a player-scoped value applied in timestamp order — and
it shares its surface with `max_hand_size` and player hexproof, which
`backlog.md` §2.15 already holds as one entry: an effective-value query beside
the oracle layer plus one post-layer, timestamp-ordered application step.
Building one of the three here would open a second engine path with its own
pool obligation inside a phase whose discovery reads *spells*. **Split out**:
it stays in §2.15, item 13 now points there, and this document claims nothing
about it.

### 3.10 Activated abilities, and resolution-created cost effects — out, with their shapes

CR 602.2b applies 601.2f–h to activated abilities, so Training Grounds and
Heartstone are `CostSubject::ActivatedAbilities(ObjectFilter)` — a second
subject with a second call site (`cast.rs::activate_ability` pays
`ability.costs` directly and does not pass through `assemble_total_cost`).
**This is where the one-mana floor lives**: every one of the eight cards that
print "this effect can't reduce the mana in that cost to less than one mana"
— Agatha of the Vile Cauldron, Biomancer's Familiar, Convergence of Dominion,
Forensic Gadgeteer, Heartstone, Power Artifact, Training Grounds, Zirda, the
Dawnwaker (Scryfall, 2026-09-07) — reduces ability costs, and 12 cards reduce
them at all. §3.4's theorem names the floor as an expiry condition; the field
that carries it lands with the subject. ~80 lines with its first consumer,
after CM-3.

"Spells cost {1} more to cast this turn" is a cost effect *created by a
resolution* and needs a duration (CR 611.2a). It is `Primitive::ModifyCost(def,
Duration)` into a `DurationRegistry<…>` — RS-1's `Primitive::Restrict` and
`state/restrictions.rs`, shape for shape — and `gather` gains a registry
source beside the sweep. `Effect::CostModification` refuses to resolve
(`resolve.rs`), as `Replacement` and `Restriction` do, so an author reaching
for it on a spell is told where to go.

### 3.11 The Ironworks board — the judged loop, step by step, and what this pipeline must leave possible

Krark-Clan Ironworks ("Sacrifice an artifact: Add {C}{C}") was banned in
Modern on 2019-01-21 with the rules interactions named as a factor: "games
with Krark-Clan Ironworks can often involve excessively arcane rules
interactions using mana ability timing windows, the understanding of which
are necessary for players to agree on the game state" (the announcement,
§12). It is the integration test for the casting pipeline as a whole, because
every one of those windows is a step of 601.2 that this document owns or
borders. The loop below is the one Wizards published and a judge walked on
video (Dave Alden, *Judging for the Win*; transcript supplied by the owner,
§12); every card's text is Scryfall's (2026-09-07). Five cards on the
battlefield: Ironworks, Scrap Trawler ("Whenever this creature dies or
another artifact you control is put into a graveyard from the battlefield,
return to your hand target artifact card in your graveyard with lesser mana
value"), Myr Retriever ("When this creature dies, return another target
artifact card from your graveyard to your hand"), Mox Opal ("Metalcraft —
{T}: Add one mana of any color. Activate only if you control three or more
artifacts"), Chromatic Sphere ("{1}, {T}, Sacrifice this artifact: Add one
mana of any color. Draw a card. (Activate only as an instant.)").

**The rules moved under this board one month ago, and the engine's frozen
CR did not.** The Hobbit update (August 2026; §12) added a criterion to
CR 605.1a — a mana ability's "cost and effect don't move any card to or
from a library" — and ten cards stopped being mana abilities, Chromatic
Sphere among them; its "(Activate only as an instant.)" is that errata's
reminder text, and it now uses the stack and cannot be activated while
casting or paying. The judge's walkthrough below is eight years older than
the change. Two things follow, and the owner checked the first with judges
on 2026-09-07:

- **The core of the loop is unchanged.** Casting the Sphere, or activating
  its ability, still opens the mana window in which Ironworks is activated
  before the cost is paid — the Sphere's *own* ability being a mana
  ability or not has no bearing on which mana abilities may be activated
  while paying for it. Steps 2–4 read the same under both rules texts.
- **Steps 5–6 reorder.** Under the frozen CR (`tmnt.txt`, and the
  walkthrough) the Sphere's ability resolves at once and the sacrifice
  triggers go on the stack after it; under the current CR the ability goes
  on the stack, the triggers go on top of it, and the returns resolve
  *before* the draw and its mana. The loop nets the same either way.

**The engine targets the frozen CR**, so `phase` cards classify the Sphere's
ability as `AbilityType::Mana` and the loop below is the one to test; the
reordered variant is the second board, written down so that whoever moves
the baseline knows what flips. Nothing here builds it: the only piece it
needs that the frozen loop does not is the CR 605.1a criterion itself,
which is a card *classification* today — an author marks an ability
`Mana` or `Activated` by applying 605.1a — and would become a rule the
engine applies ("could add mana, no target, not loyalty, moves no library
card") on the day a rules version is a knob. That is the whole cost of
"play as the rules were": one predicate per rules version where the
classification lives, not a second engine. It is worth having for
exactly this kind of board and for nothing before the modern CR's
vocabulary (the owner's limit), and it is not v1's.

| # | The play | The rule | Who owns the step |
|---|---|---|---|
| 1 | With priority, tap Mox Opal for one mana of any color, then sacrifice it to Ironworks: three mana | 605.3a (a mana ability whenever you have priority); metalcraft is an activation restriction with a count condition | `backlog.md` §2.8 (the restriction), §2.19 (any-color mana); CM-3 (the sacrifice cost) |
| 2 | Announce Chromatic Sphere's ability. 602.2b runs 601.2f: the total cost, {1}, is locked; 601.2g opens the mana window because the cost includes a mana payment | 601.2f, 601.2g | **this document** (the lock); CM-4 (the window) |
| 3 | In the window, sacrifice Myr Retriever to Ironworks, then Ironworks to itself: seven mana. "Obviously you don't need any more mana to activate Chromatic Sphere, but there's nothing saying you can't take advantage of that rule here to make some more" | 605.3a — "whenever they are … activating an ability that requires a mana payment", with no "until it is paid". Each activation is its own event, in order: Trawler sees both leave; Retriever's own trigger fires | CM-3 (Ironworks sacrificing itself is `Cost::Sacrifice(Artifact, 1)` choosing the source); **CM-4** — today the window closes the moment the pool covers {1}, so step 3 is impossible (§8) |
| 4 | Pay {1}, tap and sacrifice the Sphere: six mana. Trawler sees the Sphere die | 601.2h, after the window | CM-3 |
| 5 | The mana ability resolves at once: one mana of any color, draw a card. Seven mana | 605.3b — no stack, no priority in between. **Frozen CR only**: under the current 605.1a the Sphere's ability uses the stack, and step 6's triggers go on top of it | §2.19; a draw inside a mana ability's effect, which `mana.rs::resolve_mana_effect` must carry (noted for §2.19's owner) |
| 6 | "All these triggered abilities don't go on the stack right away; they wait until a player is about to get priority, which in this case means right after we finish resolving Chromatic Sphere's ability." Targets are chosen as each goes on the stack, so Retriever, Ironworks and the Sphere are all in the graveyard: Trawler's trigger from Ironworks (mana value 4) returns Retriever (2), its trigger from Retriever returns the Sphere (1), its trigger from the Sphere returns Mox Opal (0), and Retriever's own trigger returns Ironworks | 603.3, 603.3b (the controller orders them), 603.3d (targets chosen as the ability is put on the stack) | critical-path item 6. **A vocabulary gap for it:** "with lesser mana value" is a graveyard target compared against the *trigger's source*, a selection leaf relative to the ability's origin, which no filter says today |
| 7 | Recast Ironworks, Retriever and the Sphere with the seven mana: the starting board, one card up | ordinary casts; the pool persists within the phase | nothing new |

**Where it breaks, and the rule at each break** — the judge's three warnings,
each of which the engine must reproduce:

- **No window without a mana payment.** "You can't sacrifice Ironworks and
  Myr Retriever while you're, say, casting Mox Opal or activating Mox Opal's
  ability. These do not require a mana payment, so the game never gives you a
  chance to activate mana abilities." 601.2g's "if the total cost includes a
  mana payment" and 605.3a's "that requires a mana payment". **Decision:**
  the window opens iff the locked mana component is non-empty. A component
  reduced to nothing — 601.2f's "considered to be {0}" — is read the same
  way — **decided in CM-4, and the residual question §3.11 left for a judge
  closes here.** By 601.2g the total is locked: a `Vec<Cost>` with no record of
  how it got there, because that is exactly what CR 601.2f's lock-in discards.
  A rule that distinguished "printed {0}" from "reduced to {0}" would need the
  pipeline to carry that history, so a free Myr Enforcer behind seven artifacts
  and a Mox Opal are one board. Tested both ways, and the gate reads the
  component's *symbols* — `determine_total_cost` always emits a `Cost::Mana`,
  empty when the total is {0}.
- **Sacrifices inside one window are sequential, not simultaneous.** "Even
  though it seems like the artifacts all hit the graveyard at the same time,
  there's actually an order to it, defined by the order they get
  sacrificed." Trawler sacrificed first does not see Ironworks die; Ironworks
  cannot be sacrificed before Trawler, because nothing is left to sacrifice
  Trawler to. The engine already has this shape — each mana-ability
  activation pays its cost through its own `execute_action`, in activation
  order — and the trigger phase must read the stream that way.
- **The window is before payment.** Trawler sacrificed for mana while
  activating the Sphere does not see the Sphere sacrificed as the cost is
  paid: "the step where you can activate mana abilities happens before the
  step where you pay costs". 601.2g then 601.2h, which is the engine's order
  already.

**Mind Stone as the loop's engine, and the puzzle it makes.** Mind Stone's "{1},
{T}, Sacrifice this artifact: Draw a card" adds no mana, so it was never a
mana ability under either rules text: activating it uses the stack, and the
window at 602.2b is the same window the Sphere opens. Used as the loop's
engine instead of the Sphere, it gives the *reordered* loop under the frozen
CR — the sacrifice triggers go on the stack above its draw and resolve first,
which is exactly what the current CR makes the Sphere do. So both orderings
are reachable under `tmnt.txt`, one per engine, and the rules-version knob
above decides only which engine gives which.

The puzzle is the case where Ironworks eats that engine itself: announce Mind
Stone's ability, sacrifice Mind Stone to Ironworks in the window, and the
activation cannot pay its own sacrifice at 601.2h. Two actions are in play,
and 732.1 treats them differently. **Action one** is the activation of Mind
Stone's ability — the illegal one, whose cost of {1}, {T} and the sacrifice
was never completed; 732.1 reverses it and "any payments already made are
canceled". **Action two** is the activation of Ironworks' mana ability, legal
when it happened, whose cost was Mind Stone; 732.1 lets the player "reverse
any legal mana abilities that player activated while making the illegal
play", and a reversal not taken leaves it standing — {C}{C} in the pool, Mind
Stone in the graveyard, Scrap Trawler triggered, none of it "a result of an
undone action". Taken, it undoes action two whole: the mana leaves the pool,
Mind Stone is back, and the trigger never happened; the rule's "unless mana
from those abilities … was spent on another mana ability that wasn't
reversed" cannot bind here, because nothing was spent on a mana ability
between the two activations. **Why that reading is not optional** (the
owner's reductio, 2026-09-07): were the sacrifice instead a payment of
action one, canceled by 732.1's first sentence while Ironworks' mana stood,
the puzzle would be infinite colorless mana from two cards — sacrifice,
rewind, Mind Stone returns, {C}{C} stays, repeat. A reversal is of an
action whole: cost and mana undone together, or kept together. The engine
keeps both today, and the offer item 72 adds must undo both.
**This gains the player nothing** — sacrificing
Mind Stone to Ironworks with priority reaches the same board legally — unless
the one question below has its second answer. The engine's rollback already
keeps mana abilities (`CLAUDE.md`, `// CAST-ROLLBACK:`) and never offers the
reversal (§8). The Hobbit update touched 605.1a and nothing in 601.2, 605.3
or 732 (§12), so the puzzle reads the same under both texts.

**The one question left for a judge**, with the board on which its two
answers differ. Asked in a judge forum 2026-09-07; the answer — "if you
aren't reversing the Ironworks activation, how is Mind Stone returning?" —
confirms the reversal's shape above, and the owner closed the matter on it
and on the card's history (a mana loop this cheap would still be played).
Placement itself stays item 6's, under either reading. After the reversal, 732.2 says the player "retains" priority
— no opponent acts first. CR 603.3 puts a trigger on the stack "the next time
a player would receive priority", and 117.3c grants priority after an ability
is activated; whether a *reversed* activation counts is the question. Board:
Ironworks, Scrap Trawler, Mind Stone (mana value 2) and Chromatic Star (mana
value 1) on the battlefield, an empty graveyard. The player attempts the
puzzle and keeps the Ironworks activation. *Reading A* — retaining priority
is receiving it: Trawler's trigger for Mind Stone goes on the stack at once,
finds no artifact card of lesser mana value in the graveyard, and is removed
(603.3d). *Reading B* — it is not: the trigger waits; the player casts a spell
and sacrifices the Star to Ironworks in its window; after the cast, both
triggers go on the stack together and Mind Stone's returns the Star. Under A
the puzzle is worthless; under B it banks a trigger, which is the overpay play
through a side door. This document builds nothing on either answer. **The
judged loop needs none of this** — every step of it is a legal action — and
CM-3's test of the puzzle asserts what both readings share: the activation
rewinds, the Ironworks sacrifice and its mana stand, the trigger fires. Where
it goes on the stack is item 6's, under whichever reading the judges give.

**What this document guarantees the board, and what it leaves.** The
pipeline reads the board exactly once, before the window opens, and returns
a value nothing re-reads — that is the whole of the cost half, and it holds
by construction from CM-1 on. Of the loop's seven steps this document owns
step 2 and, through CM-3 and CM-4, steps 3 and 4; step 1 and step 5 are
`backlog.md` §2.8's and §2.19's; step 6 is item 6's, with the mana-value
target leaf named above. The loop is written down once, here, so that each
owner can see its step, and the integration test that walks all seven is
item 6's exit criterion rather than any phase's here.

---

### 3.12 Paying is deciding, then performing — and CR 732.1 has nothing to cancel (built, CM-3)

> **732.1** If a player takes an illegal action or starts to take an action but
> can't legally complete it, the entire action is reversed and any payments
> already made are canceled. No abilities trigger and no effects apply as a
> result of an undone action. …

The seam §6 named (`backlog.md` §2.1's payment half) reaches this document at
exactly one point: a sacrifice is the first payment that is a *choice* and a
*move*, and a move that is later un-made is a chokepoint question. `CLAUDE.md`
names one permanent exemption, `// CAST-ROLLBACK:`, CR 601.2a's silent move.
The question this phase had to settle is whether 732.1 is a second exemption,
a reuse of the first, or neither. **It is neither, and CM-3 builds the
property that makes it so rather than the cancellation it would otherwise
need.** `CLAUDE.md`'s chokepoint section is unchanged.

**Why not an exemption.** `// CAST-ROLLBACK:` is silent in *both* directions
because the 601.2a move was never an event — nothing observed it, so nothing
observes its undoing. A sacrifice is the opposite: it is a proposal, it goes
through `apply_replacements`, a replacement may have changed where it went,
and it emits a `ZoneChange` into the stream the trigger phase reads as its
source of truth. Undoing it means retracting an emitted event and restoring a
`PermanentState` — timestamp, counters, attachments — that the move destroyed.
That is a system with a new concept in it (event retraction), not an arm, and
it would be built for a rule the engine should never reach.

**Why it is never reached — the order, and the gate on it.** CR 601.2h hands
the order of a total cost's payment to the player, "in any order", and the
engine picks one. It picks **the order in which nothing fallible is paid after
something irreversible**, which is §3.3's mana-first argument generalised:

| rank | costs | why nothing after it can fail |
|---|---|---|
| 0 | `Mana` | the only cost whose payment can fail on a choice the player made — an illegal generic split — so it is paid while nothing else has been |
| 1 | `Tap`, `Untap`, `PayLife`, counters | reads the source's tapped state, its summoning sickness, or a life total; no rank-0 payment touches any of them |
| 2 | `Sacrifice`, `SacrificeSelf`, `Discard`, `ExileFromGraveyard` | its own payment cannot fail — `plan_payment` enumerated the candidates and `validate_pick_n` bounds the answer — and nothing follows it that a removal could invalidate |

`payment_order_rank` is matched exhaustively, so a new `Cost` arm decides where
it sits, and a `debug_assert!` on `pay_costs`'s failure path says the claim out
loud. **§3.3 was not enough on its own**: its merge lives in
`determine_total_cost`, which only spells go through, while `activate_ability`
and `activate_mana_ability` pay an ability's *printed* cost list. Every printed
card happens to put its mana and tap first; a card printing "Sacrifice a
creature, {T}:" would eat its own source and fail the tap with a payment made.
`phase_cm_cards::self_eating_engine` is that card, and it is the only board on
which the rank is visible.

**601.2h's own two groups are not modelled**, deliberately: no `Cost` arm is
random and none moves a card out of a library, so the second group is empty for
every cost that exists, and an arm the pipeline cannot apply is worse than a
missing one (`CLAUDE.md`).

**And deciding is separated from performing.** `plan_payment` takes every
choice a payment needs — the generic split, and which permanents pay each
`Cost::Sacrifice` — against the board as it stands; `pay_costs` performs the
plan and asks nobody anything. The invariant is **no payment prompt is asked
after a payment has been performed**, and it buys three things. It is the
replacement pipeline's own shape (CR 704.3: decide the batch against one
board, then perform). It makes the rank-2 argument above true rather than
nearly true, since a mid-payment prompt would read a board a previous payment
had already changed. And it is what lets a client stage a payment the way
Arena does — delve exiles, convoke taps, a sacrifice, all take-back-able until
the player confirms — because the engine never holds a half-performed one:
prompting mid-payment would have made 732.1's cancellation *mandatory* for the
GUI, which is v1's first use case. Building the staged UI is Phase 10's;
`backlog.md` §2.18 carries it beside the payer.

**The Mind Stone puzzle reaches the check, not the payment.** §3.11's board —
an activation's own source sacrificed to Ironworks inside that activation's
own 601.2g window — fails at `can_pay_costs`, which runs after the window and
before anything is paid. The activation rewinds with no payment to cancel; the
Ironworks activation, legal when it happened, stands with its mana and its
cost. That is what CM-3's test asserts, and it is what both readings of the
open judge question share.

**What is left, and it is written down rather than argued away.** Two boards
would need the cancellation, and both are `codebase-state.md` items with their
reachability stated: **item 78**, two object-moving costs in one list, which
`can_pay_costs` clears independently and only one of which is payable
(unreachable — no card prints two; the fix is a set-cover pre-check); and
**item 79**, the day the order becomes the player's (`ATOM-601.2h-003`), which
must offer only orders that complete — CR 601.2h's "unpayable costs can't be
paid" applied to the order — or build the cancellation instead.

---

## 4. Where the modifications come from — `gather.rs`

Mirror of `replacement::gather` and `restriction::is_prohibited`, narrower.

| # | Source | What | Order | Phase |
|---|---|---|---|---|
| 1 | **the sources** | every object in `cost_modification_ability_sources` — or every permanent while a summary flag is on (§3.1) — its effective ability list read once, each static ability whose body `as_cost_modification` accepts, its condition (if any) checked through `settled_holds`, its `applies_to` matched against the spell's frame with "you" = the source's current controller | `PermanentState::timestamp` — CR 613.7's, process-independent | CM-1 |
| 2 | **the spell itself** | its own effective ability list (stack frame at 601.2f, hand frame for the preview), `CostSubject::Itself` only — a `Spells` subject printed on the object being cast is *not* consulted, since CR 113.6d licenses only "that particular object" | last | CM-2 ✅ |
| 4 | a registry | §3.10's resolution-created cost effects | registration order | later |
| — | other zones | emblems, graveyard cost abilities | — | with A5 |

The fast-path gate: the source set non-empty, or either summary flag. When
all three are false the sweep is skipped — the common board, and the reason
the pipeline costs a `HashSet::is_empty` on every cast that no cost effect
touches.

**Source 2 is gated too** (§3.1's correction): the spell's own printed
abilities are the third leg, OR'd with the same two flags, and the leg asks
`CostSubject::applies_to_its_own_object` rather than "prints a cost ability".
Only when some leg opens is a frame computed at all — and when source 1's
sweep is running, that frame is the one it was already going to compute.

**Two predicates, one per gate, and they are not each other's negation.**
`applies_from_battlefield` decides whether `register_static_effects` records a
permanent as a source at all — an affinity creature is a source of nothing,
because its subject is an identity test a permanent can never satisfy, and
recording it widened the sweep on every cast for a match that could not
succeed. `applies_to_its_own_object` is source 2's leg. CR 602.2b's activated
abilities (§3.10) will answer `true` to the first and `false` to the second,
which is why one predicate could not do both jobs; both are matched
exhaustively, so a new subject has to answer each.

**What the spell-side match reads.** `object_matches_filter_in_frame(spell,
filter, you, &frame)` — `permanent_matches_filter_in_frame` until CM-0 — with
`frame = compute_characteristics(game, spell)`. For a spell on the stack that frame's
controller is the caster; for the preview (§3.6) it is the owner, which is
the same player.

---

## 5. Engine interaction points — counted against the tree, 2026-09-07

| Site | Change |
|---|---|
| `engine/costs.rs::assemble_total_cost` and `apply_cost_modifications` | both deleted; `cost_determination::determine_total_cost` is the whole of CR 601.2f, assembly included, and `costs.rs` keeps payment. 1 production caller, 8 tests moved |
| `engine/cast.rs` 601.2f | passes the new arguments |
| `oracle/mana_helpers.rs::castable_spells` | previews the total (§3.6) |
| `state/game_state.rs` | `cost_modification_ability_sources`; `register_static_effects` inserts through `as_cost_modification`; `atoms_of_static_body` gains the no-rows arm |
| `state/continuous_effects.rs::RegistryScopeSummary` | `any_granted_cost_modification`, `any_copied_cost_modification` |
| `engine/zones.rs::cleanup_zone_state` | removes the source |
| `engine/resolve.rs` | the refusal arm |
| `engine/layers/condition.rs`, `board.rs` | `SourceUntapped`; `settled_holds` public |
| `types/effects.rs` | `Effect::CostModification(Box<_>)`, `Effect::as_cost_modification`, `Condition::SourceUntapped` |
| `ui/choice_types.rs`, `ui/ask.rs`, `ui/cli.rs` | `OrderCostReductions`, `ask_order_cost_reductions`, the label |

**CM-2 adds five, and changes one of CM-1's** (2026-09-07):

| Site | Change |
|---|---|
| `engine/layers/compute.rs` | `settled_amount` — `evaluate_amount` over `Board::settled()`, `pub`, one line and a paragraph of why (§3.7) |
| `engine/cost_determination/gather.rs` | source 2, its gate leg, and `collect`/`applies_to` shared by both sources |
| `engine/cost_determination/total.rs` | `is_reduction` and `apply_one_reduction` — `ReduceGeneric` joins the ordering prompt and reads its amount at application |
| `types/cost_modification.rs` | `CostSubject::Itself`, `CostChange::ReduceGeneric`, `CostModificationDef::itself`, and the two gate predicates |
| `objects/card_data.rs` | `CardDataBuilder::affinity_for`, and `keyword` → `keyword_flag` (45 sites) so the two can coexist |
| `state/game_state.rs` | `register_static_effects` records a source only when its subject can apply from the battlefield |

---

## 6. Sizing and the phase plan

**Sized before writing, and split in the doc** (`engineering-practices.md`
§4). Every PR carries a consumer.

| PR | Shape | Measured size | Risk |
|---|---|---|---|
| **CM-0 — `PermanentFilter → ObjectFilter`** | the rename `roadmap-v2.md` A5 scheduled, pulled forward because CM-1 is its first non-permanent consumer (§3.2). No zone leaf, no behaviour | 275 occurrences / 25 `src/` files, 119 / 18 test files, 56 plan lines; `cargo build --all-targets` and a green suite are the whole check | low — pure rename; the one hazard is a doc line left saying the old name, and grep is the test |
| **CM-1 — the pipeline** | §3.1–3.6: the type, the gate, the sweep over sources, 601.2f's order, the prompt, the preview, `SourceUntapped`; no `Itself`, no `ReduceGeneric`, no `not_below` (their phases'). **Consumers:** Thalia, Guardian of Thraben (increase, **pooled**), Goblin Electromancer (reduction), Trinisphere (direct-total, conditional). Fixtures: a self-tapping sphere for lock-in across 601.2g; a kicked spell; an alternative-cost spell; three small reducers for the floor | §5's 11 sites; ~400 new engine lines in `cost_modification/`, ~250 across the sites, ~350 of cards, ~600 of tests | **medium** — the first cast-time sweep; the preview is the site that can disagree with the engine, and the merge step touches every cast |
| **CM-2 — the spell's own cost abilities** ✅ | `CostSubject::Itself` (source 2), `CostChange::ReduceGeneric(AmountExpr)`, the evaluator §3.7 argues for (`CountOf` and `SourcePower` over the finished board), affinity lowered to it, the preview reading the hand frame's cost abilities (113.6e). **And the `CardDataBuilder::keyword` → `keyword_flag` rename** (42 sites in 16 `src/` files, 3 in tests; zero behaviour, its own commit), because affinity is the first keyword a builder writes that is not a flag. **Consumers:** Myr Enforcer, Frogmite (affinity for artifacts; one pooled — the pool's first self-reduction and the first `CountOf` at cast time) | 1 source, 1 arm, 1 evaluator (~120), a builder helper, 2 cards, ~250 of tests: ~600 | low-medium — the evaluator is a third reader of `AmountExpr` and item 57's warning is answered in §3.7 |
| **CM-3 — lock-in's payment side** ✅ | `Cost::Sacrifice(filter, n)` paid through the chokepoint with a `ChoiceKind` for which permanent, as a spell's additional cost and as a mana ability's cost; a mandatory additional cost (`AdditionalCost` today is all optional, CR 118.8b); the mana component is already paid *first* (§3.3), so a split that fails has paid nothing else — and CR 732.1's "any payments already made are canceled" turned out to need nothing added — §3.12 is the answer and the gate. **Consumers:** Altar's Reap + Thunderscape Familiar (CR 601.2h's own example, a named board); Krark-Clan Ironworks + Foundry Inspector (the lock-in through the window, §3.11); Mind Stone (the 732.1 board, its trigger half left for item 6) | 2 payment arms + 1 check arm in `costs.rs`, 1 prompt, 1 `ask_choose_additional_costs` change, 5 cards, ~350 of tests: ~800 | low — payment machinery with the CR's own board and the banned deck's as the tests |
| **CM-4 — the mana window and the payer** ✅ | `run_mana_ability_window` opens only when the locked mana component is non-empty (601.2g) and then runs until the player declines or no ability is left (605.3a) — it also stopped the moment the pool covered the cost, which was a payer's policy in the engine's loop (§3.11, §8). The policy moved to **two** decorators, not one: `ui::ManaWindowStop<D>` takes the stop, `ui::AutoPayer<D>` takes `GenericManaAllocation` and `OrderCostReductions`, and CM-3's sacrifice choice stays the wrapped provider's (§3.4's criterion). Clients compose; `--no-auto-pay` drops the stack. **Consumers:** the loop's step 3 with CM-3's cards, no new card needed; the fuzz harness | ~35 in the window, ~290 decorators, ~60 wiring, ~330 tests: ~715 | medium — every cast's prompt sequence passes through it; the A/B is the check, and it found the one board where the payer's stop is narrower than the engine's was |
| **CP-1 — payment (a sized slot, not a design)** | §2.1's other half. 601.2b's announcement of a nonhybrid equivalent and of Phyrexian halves (a `ChoiceKind`, before 601.2f); `ManaPool::pay`/`can_pay` branches for `Hybrid`, `MonoHybrid`, `Phyrexian`, `HybridPhyrexian` (`pay_life` for the latter, through the chokepoint); `find_mana_sources` and `remaining_cost_after_pool` for them (the AI cannot cast a hybrid card today); `ask_choose_generic_mana_allocation`'s tally; mana value with X on the stack (202.3e — a characteristic, read off the `StackEntry`); `{Q}` exists as `Cost::Untap` and its atoms want annotations. **A note for the Scryfall parser that CP-1 or Phase 8 writes:** a printed cost's symbol order is not WUBRG — two-colour costs follow the colour wheel's shorter arc ({G}{U}, {R}{W}), shards and wedges have their own — and `ManaCost` equality is *sequence* equality, so a parser must keep the printed order for display and comparisons must be by multiset. `ATOM-107.4e/f-*` (7 `NEW`), `ATOM-202.3*` (7) | 6 sites; ~1 PR | medium — `ManaPool::pay` is on every cast |

**Why the modification/payment seam is where it is** (the owner's question,
§11 note 7). CP-1 is *sequenced*, not deferred: it sits at 601.2b and 601.2h,
on either side of this pipeline, and touches `ManaPool`, the auto-tap and the
allocation prompt — a disjoint set of sites from §5's. 601.2b's announcement
runs *before* 601.2f, so the pipeline sees only nonhybrid symbols whether or
not CP-1 exists, and the one clause of CR 118.7 that CP-1 changes for the
pipeline — 118.7e, a hybrid *reduction* symbol — is already an expiry
condition in §3.4. It blocks breadth that modification does not: every
hybrid and Phyrexian card is uncastable by the AI today. So it goes right
after CM-1 if a card family wants it before CM-2, and nothing in either
depends on the other.

**Why CM-2, CM-3 and CM-4 are not folded into CM-1.** The lock-in is proven in CM-1
across the 601.2g window (a fixture taps itself for mana and the cost stays
at three); CM-2 adds a source and an evaluator with its own argument; CM-3
adds payment machinery — the seam §2.1 named; CM-4 changes every cast's
prompt sequence and belongs in a PR whose whole review is that change.
Folded, CM-1 passes 3,000 additions with docs. Five PRs, each in band, each
with a consumer.

**Ordering.** CM-0, CM-1 before RD (`cant-effects-architecture.md` §7.1 row
10; `replacement-architecture.md` §9 "Interleaved — Commander"). CM-2, CM-3,
CM-4 and CP-1 any time after CM-1, in any order, except that CM-4's overpay
test wants CM-3's cards; RS-4 "reads better after" the modification phases.
**CM-3 and CM-4 before critical-path item 6**, because §3.11's loop is item
6's integration test and steps 3–4 are theirs. Nothing here depends on
RC/RD/RE and nothing in them depends on this.

**Every PR carries** the card registered where there is one, `PERFORMANCE_
POOL` +1 per new engine path (CM-1: Thalia; CM-2: Myr Enforcer; **CM-3:
Bone Splinters** — this row first said CM-3 opened no path a pooled card would
measure, and the A/B disproved it: the engine change is `IDENTICAL` to `main`
on the old pool, but `Cost::Sacrifice` is a path no card in the 72 could
reach, which is the pool doc's own failure mode. **CM-4 opens none and the pool
does not move — and its prediction was wrong in the other half:** the engine
changes are game-identical to `main` with the decorator dropped, but the
shipped stack moves 1 game in 200 on `performance` and 8 in 200 on `stress`,
because the payer's stop is narrower than the engine's was
(`codebase-state.md` item 83). No card would have measured that; a fourth
binary did), `plans/
fuzz_ab.py` against a same-day `main` worktree, three-run determinism on both
pools, `specdb owed` clean for `Phase 5-Layers`, and `// COVERS:` on exactly
what each test builds. None qualifies for a trace page: each adds a read, none
changes how an existing read is answered (`engineering-practices.md` §7).

---

## 7. Testing — the atoms this owes

Re-filed into `Phase 5 Layers (CM-<n> …)` so `owed` gates them, from
`Backlog — cost pipeline`; `ATOM-613.11-001/002` were already there under
`L15`. Ticket `NEW — cost-architecture.md CM-<n>` until covered.

| Atom | Claim | Test | Mark |
|---|---|---|---|
| `ATOM-601.2f-001` | base + kicker + increase, one component | CM-1: a kicked {3}{R} fixture under Thalia pays {6}{R} | COVERS |
| `ATOM-601.2f-002` | floor at {0} | CM-1: {1}{G} under {1}, {1} and {G} reducers pays {0} | COVERS |
| `ATOM-601.2f-003` | locked before payment | CM-1: the self-tapping sphere — locked at three, tapped in 601.2g, three paid | COVERS-PARTIAL in CM-1 (the mutation is a tap, not an entering increase); **completed in CM-3** — Foundry Inspector settles Mind Stone's total at {1} and Ironworks eats the Inspector inside the window; the leftover colorless is the assertion |
| `ATOM-601.2f-004` | two reductions, the player orders them | CM-1: two reducers on one instant — the prompt is asked, both apply, every order agrees | COVERS, with the §3.4 note |
| `ATOM-613.11-002` | increases before reductions | CM-1: Thalia + Electromancer on Lightning Bolt: {R} | COVERS |
| `ATOM-613.11-001` | game-rule effects read final characteristics | CM-1: Thalia under Humility stops taxing — the cost half of the claim | COVERS-PARTIAL — the atom's board is an attack restriction over an effective color, RS-3a's |
| `ATOM-118.7-001`, `-002` | reduced to nothing is {0}; free to cast | CM-1: {1}{R} reduced by {2} is {R}; a spell reduced to {0} casts from an empty pool | COVERS |
| `ATOM-118.7a/b/c/d-001` | the arithmetic | CM-1: unit tests in `total.rs` | COVERS |
| `ATOM-118.9d-001` | modifications apply to an alternative cost | CM-1: an alternative cost of {R} under Thalia pays {1}{R} | COVERS |
| `ATOM-702.41a-001` | affinity reduces generic by the count it names | CM-2: Myr Enforcer cast from an exact pool at four artifact counts, an opponent's not counted, and seven artifacts casting it free | COVERS; the `{6}`/4 arithmetic is `total.rs`'s COVERS-PARTIAL, since it computes and does not cast |
| `ATOM-601.2h-001` | the Altar's Reap example | CM-3: Altar's Reap under Thunderscape Familiar, from an exact {B} pool, the Familiar paying | COVERS; re-filed into `Phase 5 Layers (CM-3 …)` |
| `ATOM-118.8d-001` | an additional cost does not change the mana cost | CM-3: Altar's Reap on the stack is mana value 2 | COVERS; re-filed, with a session note that the board is {1}{B} and not the atom's {2}{R} |
| `ATOM-601.2g-001` | the window opens for a mana payment | CM-4: a `{G}` artifact with a Forest untapped — one window | COVERS; **not re-filed** — see below |
| `ATOM-605.3a-001` | a mana ability may be activated while casting | CM-4: the same cast; and COVERS-PARTIAL from the overpay board | COVERS |
| `ATOM-605.3a-003` | …and while activating an ability (CR 602.2b) | CM-4: Mind Stone's `{1}, {T}, Sacrifice` from an empty pool | COVERS |
| `ATOM-605.3b-001` | a mana ability resolves at once, never on the stack | CM-4: a Forest tapped with the stack empty before and after | COVERS |
| `ATOM-107.4e/f-*`, `ATOM-202.3*` | payment | **CP-1** | stay `Backlog`, labelled CP-1 |

`ATOM-118.7e-*`, `-f`, `-g` (hybrid, Phyrexian, snow reductions) and
`ATOM-601.7-001` stay in `Backlog`: the first three are CP-1's symbols, the
last is structural and observes nothing.

**CM-4's four atoms were not re-filed, and checking that was the point.** CM-1
through CM-3 re-filed theirs out of `Backlog` so `owed` would gate them. CM-4's
were never in `Backlog`: `ATOM-601.2g-001` sits in `Phase 5-Pre` under ticket
`T18` and the three CR 605.3 atoms in `ALREADY-IMPL` under `N/A`, all four
uncovered since those phases closed. `owed` selects on `ticket LIKE 'NEW%'`, so
covering them shrinks `owed --all` and cannot grow `owed`; re-filing them into
`Phase 5 Layers (CM-4 …)` with a `NEW` ticket would be a lateral move between
two shipped phases that changes no gate, and doing the same to
`ATOM-605.3a-002` — Rhystic Study's "unless that player pays", which is item
6's — would have grown `owed` from 9 to 10 for an atom this phase cannot cover.
**`owed` stands at 9, unchanged, and none of the nine is CM's.**

**Two claims of this phase have no atom**, and both say so in the test rather
than borrowing one. CR 601.2g's *negative* — a `{0}` cost opens no window — has
none; the corpus has `ATOM-601.2g-001` for the window opening and nothing for
it staying shut. Nor does the overpay play itself: `ATOM-605.3a-001` says a
mana ability may be activated during a cast, not that a *second* one may be
activated after the cost is covered, so the overpay test claims it only as
COVERS-PARTIAL. The session file carries a note on each.

**CR 113.6d and 113.6e have no atoms**, and CM-2 checked rather than assumed:
session-1 records both as PURE-DEF ("Framework"), so there was nothing to
re-file for either and this document is the only place the zone question is
written down. **CR 702.41b has no atom either** — the corpus has it as a
deferred one-liner — and the engine reached it anyway: affinity lowers to one
static ability per instance and the gather returns one instance per ability, so
"each of them applies" is the gather's shape rather than a case.
`total.rs::two_instances_of_affinity_each_apply` covers it and claims no atom;
the session file carries a note saying so.

**Reachability, not just coverage** (`engineering-practices.md` §3.3). Thalia
is pooled, so the sweep and the increase run in every measured game; Humility
is pooled, so the strip path does. Electromancer and Trinisphere are in the
stress pool, and two Electromancers in one deck is what makes the ordering
prompt reachable from a fuzz game at all.

Myr Enforcer is pooled for CM-2, so source 2, the hand-frame preview and a
cast-time `CountOf` run in measured games: cast 167 / resolved 166 in 113 of
200 (56%), **1.74 copies per deck**, which is what makes "one Enforcer counts
the one already on the battlefield" routine rather than contrived. On `stress`
with Trinisphere forced beside it — 158 / 157 in 108 games and 193 / 192 in 136
— affinity's reduction and a direct-total effect meet on one spell.

---

## 8. Findings

1. **A conditional replacement or restriction static is inert** (§3.1):
   `register_static_effects` matches the body without peeling
   `Effect::Conditional`, and neither gather does either. No registered card
   has one; the fix is the same `as_…` peel this phase wrote for cost effects.
   `codebase-state.md` "Before card breadth", new item.
2. **Two `Cost::Mana` entries in one total were paid against one allocation**
   (§2). Fixed by the merge; a kicked spell with a generic kicker had never
   been cast in a fuzz game (no registered card kicks).
3. **`run_mana_ability_window` stops when the pool covers the cost** (§3.11).
   CR 605.3a has no such clause and the Ironworks loop's step 3 is exactly
   the play it forbids — confirmed by the judged walkthrough ("there's
   nothing saying you can't take advantage of that rule here to make some
   more"). The stop belongs in a payer `DecisionProvider` (§3.4); the engine
   should offer the window until the player declines. **CM-4, ✅ built.** The
   stop is `ui::ManaWindowStop`. It is narrower than the engine's was — the
   engine asked until `can_pay_costs` cleared the *whole* cost list, the
   decorator declines once the *mana component* is covered — and that
   difference is reachable and is the whole of CM-4's counter movement
   (`codebase-state.md` item 83).
3a. **The window's opening condition is right by accident.** 601.2g opens it
   only "if the total cost includes a mana payment", and the judge's warning
   is the board: casting Mox Opal offers no window. `run_mana_ability_window`
   is called unconditionally and returns at once because a zero cost is
   already payable — the right answer, produced by the stop condition CM-4
   removes. **✅ built, CM-4**, in the commit before the one that removes the
   cover, with the reduced-to-nothing reading decided the same way (§3.11).
4. **CR 732.1's reversal of mana abilities is the player's option and the
   engine never offers it** (§3.11). It always keeps them, which is *a* legal
   answer, not the player's. A `ChoiceKind` on rewind. **Placed by CM-4: with
   the trigger phase, not the payer.** Which mana abilities to reverse fails
   §3.4's criterion — its answers differ in permanents and events, not in mana
   — so the payer is the wrong home; and the board where it is observable is
   the Mind Stone puzzle, whose open question is where the surviving trigger
   lands.
5. **`ATOM-601.2f-004`'s worked example is wrong** (§3.4). Noted in the
   session file, not rewritten — the corpus is authored.
6a. **A gate that reads a body and not its subject is a gate for the wrong
   question** (CM-2, §3.1/§4). Two of them, found on the same day and from
   opposite directions. `register_static_effects` recorded any static whose
   body is a cost modification, so an affinity permanent became a "source"
   that widened CR 601.2f's sweep on every cast for a match no permanent can
   satisfy. Source 2's gate asked whether the card prints a cost ability,
   which is true of a Thalia *in hand*, so her frame was computed at every
   castability preview and refused — five non-member walks per 200 measured
   games, found by `fuzz_ab.py`'s middle arm and by nothing else. Both fixed
   by asking `CostSubject` a question (`applies_from_battlefield`,
   `applies_to_its_own_object`), matched exhaustively so a new subject decides
   both. **The general shape**: `Effect::as_…` tells you what kind of thing an
   ability is, never where it can apply from, and every one of the three gates
   is about *where*.
7. **A registry row cannot reach a non-member, so CR 113.6e's second sentence
   has nothing to build on** (CM-2, §3.1). `compute_non_member` applies CDAs
   and no rows at all, so a card in hand or a spell on the stack has exactly
   its printed ability list. That is what makes source 2's printed gate leg
   *exact* rather than an over-approximation — and it means "an ability that
   grants it another ability that restricts or modifies how that particular
   object can be played or cast functions only on the stack" is unreachable
   today. Recorded as a Deferred Migrations item with the gate's flag legs
   named as what catches it when it stops being unreachable.
8. **Affinity is the first mechanic that reduces a component to nothing on a
   printed card**, which makes item 71's board (CR 601.2g's "if the total cost
   includes a mana payment", read against a component reduced to nothing)
   reachable from a measured game for the first time. The window's opening
   condition is still right today and still CM-4's to make explicit; only its
   reachability moved.
9. **"Is a mana ability" is an author's classification, and the rule it
   applies moved after the freeze** (§3.11). `AbilityType::Mana` is CR
   605.1a applied by whoever writes the card, and the Hobbit update added
   a criterion to that rule. A rules-version knob would make the
   classification the engine's — one predicate over the ability's cost
   and effect — and re-read every registered card through it. Not v1's;
   recorded so the CR sync that moves the baseline knows the Sphere flips.

---

## 9. Explicitly out

- **Payment** — hybrid, Phyrexian, X mana value, `{Q}`'s annotations: CP-1,
  sequenced in §6 with its rationale.
- **Activated abilities' costs** (12 reducers, the 8 floored ones among them)
  and **resolution-created cost effects**: §3.10, with their shapes.
- **Commander tax**: §3.8, with B2.
- **`lands_per_turn`, `max_hand_size`, player hexproof**: `backlog.md` §2.15.
- **Golden-Tail Trainer**: the *card*, with item 6 — its second ability is an
  attack trigger, and a card wearing its name with half its text is what
  `engineering-practices.md` §3 forbids. Its *mechanism* is in:
  `settled_amount` answers `SourcePower` and `phase_cm_cards::power_reducer`
  is the fixture that consumes it (§3.7). Registering the printed card is then
  a card change with no engine change behind it.
- **Cost abilities in other zones** (emblems, Convergence of Dominion): §3.1,
  with A5.
- **The Ironworks board's trigger half and the 732.1 reversal choice**:
  §3.11 and §8; the reversal is placed with critical-path item 6 rather than
  with the payer (§8 item 4). The window itself is CM-4, built.
- **An agent that declines to tap the activation's own source for mana**:
  `codebase-state.md` item 83. Legal under CR 605.3a and always a mistake, but
  it is `RandomDecisionProvider`'s policy and it would move every counter, so
  it wants its own phase and its own A/B.
- **Alternative costs *provided by* effects** ("you may cast it without paying
  its mana cost"): `backlog.md` §2.3.

---

## 10. Documents this phase owes

`CLAUDE.md` (the row, in place); `codebase-state.md` ("Before Layers" item 3,
main item 13, the CM-1 entry, §8's findings as Deferred Migrations items);
`backlog.md` §2.1 (graduated), §2.15 (owns `lands_per_turn` alone now), §2.18
(the payer middleware named, and from CM-3 the staged payment beside it);
`cant-effects-architecture.md` §7.1 row 10;
`replacement-architecture.md` §9's "needs a phase marker of its own";
`roadmap-v2.md` A5 (the rename is CM-0) and B1; `layers-architecture.md`
§11.2's pointer; the session files for the re-filed atoms and the notes under
`ATOM-601.2f-004` and `ATOM-118.8d-001`; `engineering-practices.md` §3
re-recorded whenever the pool moves (CM-1, CM-2, CM-3); and
`plans/state-of-play.md` regenerated. **`CLAUDE.md`'s chokepoint section is
not on this list and CM-3 confirmed it stays off** (§3.12).

---

## 11. The review notes, and where each landed (2026-09-07)

| Note | Where |
|---|---|
| Krark-Clan Ironworks as the integration test | §3.11, §8 items 3–4, one judge question with the board its answers split on; CM-3's consumers |
| The last 601.2f clause is Trinisphere's alone | §1 consequence 2, §3.2 |
| Why sweep every permanent, and should it be objects | §3.1: the sweep is over sources; the widening is the unattributed flags; the candidates are 113.6's objects, so the spell itself is source 2 (CM-2) and other zones are A5's |
| `Spells(PermanentFilter)` is nominally wrong | §3.2: CM-0 renames it first |
| Agatha's one-mana floor | §3.2 `not_below`, §3.4 second expiry condition with the worked example, §3.10: all eight carriers are ability reducers |
| An auto-payer middleware pre-v1 | §3.4's third consequence, §8 item 3 |
| Why defer payment | §6 "Why the seam is where it is": sequenced, sized, disjoint sites |
| The judged Ironworks loop (transcript) | §3.11 rewritten around it: seven steps with owners, the three warnings as engine decisions, the Mind Stone case reduced to a puzzle with one open question; CM-4 added |

---

## 12. Sources consulted for §3.11 (2026-09-07)

- `MTG-Rules/versions/tmnt.txt`: 601.2e–h, 603.3, 605.3a, 732.1–2.
- Wizards of the Coast, *January 21, 2019, Banned and Restricted Announcement*
  (magic.wizards.com/en/news/announcements/january-21-2019-banned-and-restricted-announcement):
  "Games with Krark-Clan Ironworks can often involve excessively arcane rules
  interactions using mana ability timing windows, the understanding of which
  are necessary for players to agree on the game state."
- Dave Alden, *Judging for the Win*, the Krark-Clan Ironworks combo video —
  an AI transcript lightly punctuated by the owner and supplied 2026-09-07;
  the loop, the trigger timing and the three warnings in §3.11 are its.
- Card Kingdom, *A Beginner's Guide to Krark-Clan Ironworks*
  (blog.cardkingdom.com/a-beginners-guide-to-krark-clan-ironworks/): the
  "overpay" play and the simultaneous-graveyard targeting it produces.
- Magic Judges forum, *When are you assumed to have used your mana
  abilities?* (apps.magicjudges.org/forum/topic/45414/, 2018-08-28): the
  Inventor's Fair board and "after 601.2e, we don't check again".
- Scryfall, 2026-09-07: oracle text of every card named above; the
  population counts in §3.1 and §3.10.
- **The second pass, narrowed to August 2026** (the owner's request after
  the Sphere's reminder text was noticed): Wizards of the Coast, *The
  Hobbit Update Bulletin*
  (magic.wizards.com/en/news/announcements/the-hobbit-update-bulletin) —
  CR 605.1a gains "its cost and effect don't move any card to or from a
  library. Do not take into account replacement effects that may apply,
  other than self-replacement effects"; ten cards lose mana-ability status
  (Chromatic Sphere, Selvala, Explorer Returned, Millikin, Deranged
  Assistant, Charmed Pendant and the five Eggs) and gain "(Activate only as
  an instant.)"; the stated reason is that "it can be disruptive for cards
  to be moving in and out of hidden zones" during casting and resolution.
  Draftsim, *3 New MTG Rules Changes You Might've Missed*
  (draftsim.com/mtg-hob-3-rule-changes/) lists the release's rules
  changes; none touches 601.2, 605.3 or 732. The owner's judges,
  2026-09-07: "Sphere is unaffected by the change" as far as activating
  mana abilities while casting it or paying for its ability goes.

#### CM-0 — `PermanentFilter` → `ObjectFilter` — ✅ 2026-09-07

The sweep §3.2 sized, and nothing else: 275 sites in 25 `src/` files, 119 in
18 test files, the live plan docs, and the three `permanent_matches_filter*`
matchers renamed with it. `EffectRecipient::FilteredPermanents` and
`SelectionFilter::Permanent` keep their names — both still name permanents.
Zero warnings, the suite green, and a same-seed `fuzz_games` diff against
`main` identical outside the timing block. No zone leaf (layers item 9's).

#### CM-1 — the pipeline — ✅ 2026-09-07

Built as §3 says. What the building changed is in `codebase-state.md`'s CM-1
entry (main items 70–74): the merge step closed a latent kicker defect (item
74), and the recording test provider now records an ordering prompt's *kind*.
Thalia is pooled; the §3 table is re-recorded in `engineering-practices.md`.

#### CM-2 — the spell's own cost abilities — ✅ 2026-09-07

Built as §3.7 argued, with three corrections to this document that the
building forced, each marked in place: **113.6d and not 113.6e** is affinity's
zone rule and the preview is licensed by §3.6's engine requirement rather than
by either (§3.1); **source 2 is gated**, because the frame §3.1 called "already
computed" is not computed on the common board (§3.1, §4); and the evaluator is
a **reader over one leaf table**, so `codebase-state.md` item 57 closes with
"there is no third evaluator" rather than with a third one (§3.7).

`SourcePower` is answered in the reader and refused by the walk, with a test
each way and a fixture — `power_reducer` — consuming it, since the objection
to Golden-Tail Trainer was always about the card's *name* and never about the
arm. Myr Enforcer is pooled and Frogmite registered; the `keyword` →
`keyword_flag` rename went first, alone. The A/B's middle arm is `IDENTICAL` to
`main` on `performance` — on the second run: the first found two gates asking
about a body where the question was about a subject (§8 items 6a and 7).

#### CM-3 — lock-in's payment side — ✅ 2026-09-07

Built as §3.12 argues, and §3.12 is what the building added to this document:
the 732.1 question the phase opened with is closed by an ordering property and
a gate on it, not by a rollback and not by a second chokepoint exemption. Two
residues are `codebase-state.md` items 78 and 79 with their reachability
stated, and the reversal *offer* stays item 72's.

`Cost::Sacrifice` is paid as one `execute_actions` batch with "you control"
supplied by CR 701.21a rather than by the card's filter; `AdditionalCost::
Mandatory` gives CR 118.8b its other half, and 601.2b announces the optional
costs alone. `castable_spells` gained the mandatory-cost question, which §3.6
required and no mana cost could answer.

Five printed cards in three pairs of a board, two fixtures for the two claims
no printed card can carry — and the first **rulings pass**
(`engineering-practices.md` §3.4), which turned eleven Scryfall rulings into
six tests, two already-covered notes and three named gaps. One of the six is a
board no atom in the corpus asks for: the Familiar reduces a black-*and*-green
spell once, which an author writing two colour abilities instead of one `Or`
would get wrong with every existing test still green. **§6's pool prediction was wrong and the A/B is what
said so**: the engine arm is `IDENTICAL` to `main`, and the pool grows to 73.
Which card is a second question, also measured — Altar's Reap costs +20.2%
CPU/game and Bone Splinters +11.9% for the identical set of paths, so the pool
carries Bone Splinters and Altar's Reap stays registered as CR 601.2h's own
example. An inert 73rd card costs +14.0%, which is the finding underneath
both numbers: the tax is the slot, not the mechanic
(`engineering-practices.md` §3.1a).

#### CM-4 — the mana window and the payer — ✅ 2026-09-08

Built as §3.4 and §6 say, with two corrections the building forced.

**One payer became two decorators.** §3.4 named a single `ui::AutoPayer<D>`
answering four prompts, and §6 sized it with a flag. Both are wrong for the
same reason: the four prompts do not belong to one client. `ManaWindowStop`
and `AutoPayer` toggle independently — a human turning off auto-pay wants the
window to keep offering, an agent without a stop has only
`WINDOW_ACTIVATION_CAP` — and a scope enum inside one payer would have been a
closed enumeration of the subsets of something that already composes. Clients
compose a stack; the invariant is one decorator per `ChoiceKind`.

**And the sacrifice prompt left the payer**, on a criterion rather than a
judgement call: a payer answers a prompt when every legal answer leaves the
same game state except for mana (§3.4). That criterion also placed §8 item 4 —
CR 732.1's reversal offer goes with critical-path item 6, not with the payer.

The residual 601.2g reading is closed: a component reduced to nothing opens no
window, because the lock-in discards the history that would distinguish it from
a printed `{0}`. Both directions are tested and both tests fail with the gate
removed.

**No new card.** §6's claim that §3.11's step 3 builds out of CM-3's registered
five held — Ironworks, Mind Stone and two spare artifacts — so
`PERFORMANCE_POOL` stays at 73 and `engineering-practices.md` §3's table is not
re-recorded. Its `Memo hits` row is the one number the tree now disagrees with
(+0.8% on both pools, from the extra enumeration the window costs); recorded in
`codebase-state.md`'s CM-4 block rather than re-recorded here, since the pool
did not move.

**§6's own prediction was wrong, in the opposite half from CM-3's.** CM-3
predicted no new pooled path and was wrong about the pool; CM-4 predicted the
same and is wrong about the engine. The counters moved — 1 game in 200 on
`performance`, 8 in 200 on `stress` — and a fourth binary attributed all of it
to the decorator rather than to either engine change: with the stack dropped,
the 601.2g gate and the removed early return together leave `stress`
byte-identical and `performance` different by two memo hits and no game-state
counter. The board is `codebase-state.md` item 83, a source tapping itself for
mana inside its own window; the payer stops asking a question no mana ability
could answer, which `main` did not. Traced to Chainbreaker once and Mind Stone
eleven times, by name, in a debug build.

**No trace page** (`engineering-practices.md` §7): no read is answered
differently, only asked by a different party. And no seam for recording why the
payer chose what it chose — its answers are a pure function of
`(ChoiceKind, remaining_cost)`, already in the prompt stream, and a decorator
buffering anything to explain itself would be a decision site holding
outcome-bearing state off `GameState`. A4c's four emit points stay four.
