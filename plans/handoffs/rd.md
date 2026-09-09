# Phase RD — damage (CR 615, 609.7, 614.9, 120.3)

**State (2026-09-09): RD-1 and RD-2 landed. RD-3 is next — sources — and it
inherits a working count, a working prevented-amount channel and a working
allocation, so its risk is the source-side predicate and nothing in the
loop.**

## Where the work is

- `plans/replacement-architecture.md` §9, "Phase RD" — the design check
  (seven decisions plus the shield glossary), the four-PR table with measured
  call-site counts, each PR's consumers with their rulings passes, the atoms
  each claims, RD-5's gate, the measurement plan and the exit criteria.
  **Read this first.** RD-1's and RD-2's sections each carry an "As landed"
  block with what the build changed and what the A/B measured.
- `plans/replacement-architecture.md` §11 items 21–30 — the findings the
  sizing and the reviews produced. 21, 22, 24, 27, 29 and 30 are closed;
  23 (Harm's Way, RD-5's gate), 25 (CR 615.12, RD-4's), 26 and 28 stand.
- `plans/traces/rd-2-a-decision-is-per-subject.html` — the loop's new unit,
  walked read by read on the boards the design check argued about.
- `plans/backlog.md` §2.6 and §2.23 — the CR 120.3 results RD does not ship,
  and the seam RD-1 cut for them (`engine::actions::DamageResults`).
  `codebase-state.md` items 86 and 87 are the dated lines; items 90–95 are
  RD-2's.
- `plans/cant-effects-architecture.md` §4.7 — the superseding note on
  `ReplacementKind`. `plans/roadmap-v2.md` row A4 — the size cell.

## What RD-1 shipped

`ReplacementDef.affected_players: PlayerSet` (decision 0); `Rewrite::Amount`
with `Multiplier`, `Halve` and `PreventHalf`, and `Rounding` with no `Default`
(decision 1); `Rider` carrying an `EventSubject` and the replaced event's
amount, plus `AmountExpr::ReplacedAmount` and `Multiply`;
`GameAction::LoseLife.cause: LifeLossCause` (decision 4); CR 120.3's results
decomposed off the target's effective types, including 120.3c (decision 5);
`Primitive::Mill`. Registered: Furnace of Rath (also `PERFORMANCE_POOL`),
Ghosts of the Innocent, Gisela, Blade of Goldnight, Angel of Suffering, Loyalty
Probe.

## What RD-2 shipped

`Primitive::CreateReplacement(Box<ReplacementDef>, Duration)` — the durational
form `Effect::Replacement` refused to be — filling its rows from a `Target`
(one per target), a `FilteredPermanents` recipient (one per permanent,
CR 615.11) or as authored (`Implicit`; a `Filter`/`PlayerSet` asked at each
event); `Uses::NextDamage(u64)` paired both ways with
`AmountRewrite::PreventRemaining`, and `PreventUpTo(n)` as the shape a count is
cut down to (`capped`); `ReplacementDef::is_prevention()`, derived (CR 615.1a);
`Rider.prevented` read by `AmountExpr::DamagePrevented` (decision 2 and the
rider's other number). **The loop's unit:** `execute_batch_inner` groups the
APNAP-ordered members by `subject_of` and `apply_replacements` runs one CR
616.1 loop per group — one applied set, one chooser, the chosen instance
applied to every member it applies to, its rider queued once with the amounts
summed, its use spent once (decision 3). `consume_use` runs after
`apply_rewrite` and spends `Applied { took_effect, prevented }` (decision 7).
CR 615.7's allocation is `ChoiceKind::AllocateNextDamage`, asked once per
instance over every member it applies to — later groups' members included —
and kept on `GameState::prevention_allocations` (item 40's shape). The
all-multiplier bucket asks nobody (§11 item 29). Registered: Mending Hands
(also `PERFORMANCE_POOL`), Samite Healer, Safe Passage, Samite Censer-Bearer.

## The decisions that change the shape

0. ~~`ReplacementDef.affected_players: PlayerSet`~~ — **landed in RD-1**.
1. `AmountRewrite` is six arms: three landed in RD-1, `PreventUpTo` and
   `PreventRemaining` in RD-2; `Plus` is RD-3's. `Rounding` has no default
   (CR 107.1a). `Instead` gets no "N − k" template.
2. ~~CR 615.7 shields are rows in the existing `ReplacementEffectRegistry`~~ —
   **landed in RD-2**, as designed: `Primitive::CreateReplacement`,
   `Uses::NextDamage(remaining)` decremented in place through
   `spend_next_damage`, the word "shield" nowhere in code, `ApplyPrevention`
   deleted.
3. ~~Decisions are per `(batch, subject)`, rewrites per member; the CR 615.7
   `allocate` prompt is per *instance*~~ — **landed in RD-2** with Mending
   Hands. The one corner §9 recorded for review — a later group's doubling
   ahead of the count moves the member, not the allocation — is
   `codebase-state.md` item 93.
5. ~~CR 120.3c ships in RD-1 with **Loyalty Probe**~~ — **landed**.
7. ~~`consume_use` moves after `apply_rewrite`~~ — **landed in RD-2**.

## Consumers, by PR

- ~~**RD-1**~~ — landed: Furnace of Rath, Ghosts of the Innocent, Gisela,
  Blade of Goldnight, Angel of Suffering, Loyalty Probe.
- ~~**RD-2**~~ — landed: Mending Hands, Samite Healer, Safe Passage, Samite
  Censer-Bearer.
- **RD-3** — Circle of Protection: Red, Reverse Damage, Guardian Seraph,
  Daunting Defender + Pyroclasm, Fog, Torbran, Thane of Red Fell, Dark Sphere.
- **RD-4** — Pariah, Palisade Giant, Pinpoint Avalanche.
- **RD-5 (candidate)** — Harm's Way, gated on the split staying off the
  per-member path.

## Two notes for RD-3, from RD-2's build (2026-09-09)

1. **What the source-side predicate needs from the row shape, and what it
   does not.** RD-3 adds `EventPattern::DealDamage { source: Option<SourcePattern>,
   combat: Option<bool> }`. Nothing on the *row* has to change for it: a
   resolution-created row already carries its `def` whole, so Circle of
   Protection: Red's chosen source is `SourcePattern { object: Some(chosen),
   .. }` written into the def by `Primitive::CreateReplacement` the way the
   resolution fills the affected set today — and that is the one thing the
   primitive must grow: a way to fill a **pattern** field from a resolution
   choice, beside filling a set from a target. Two designs: (a) a
   `SelectionFilter::DamageSource` choice made at resolution and written into
   `def.pattern` before the row is added (the `Restrict` shape — the card
   authors an empty `object: None` and the resolution overwrites it, with the
   same `debug_assert` that it was empty); (b) a `Choose` recipient whose
   resolved choice the primitive routes into the pattern rather than the set.
   (a) keeps the recipient for what the row is *about* and is the shape the
   design check assumed (`ChoiceKind` for the source, `enumerate_legal_selections`
   for the candidates). What RD-2 settled that RD-3 should not reopen: the
   group form evaluates `applies_to` **per member**, so a source predicate
   that matches one attacker and not the other already yields a candidate
   with a one-member list and the count allocates over exactly the members it
   applies to (`next_damage_shares` reads `Candidate.members`; the later-group
   buckets are filtered by `applies_to` too). `PreventUpTo`'s performer is in
   and tested member-uniform through a fixture; Guardian Seraph and Daunting
   Defender are the printed producers (`codebase-state.md` item 91). And
   `ordering_cannot_change_outcome`'s multiplier clause is written against
   `matches!(pattern, EventPattern::DealDamage)` without fields — adding
   fields keeps it true as long as none reads the *amount*
   (`codebase-state.md` item 47's condition (d)).

2. **Does `Uses::NextDamage`'s decrement survive a source that stops matching
   mid-turn? Yes, and by construction — CR 609.7b's recheck costs nothing
   here.** CR 615.9/609.7b: a "next 3 damage from a red source" shield whose
   chosen creature loses red before it deals damage does not apply and is not
   spent. In the group form the row is gathered per member per iteration, and
   `pattern_watches` is asked *then*, off the source's effective
   characteristics at the moment of the proposal; a source that no longer
   matches produces no candidate, so the instance is never chosen, never
   applied, and `consume_use` never runs for it — the count is untouched
   because nothing was there to spend, exactly as §9's decision 7 predicted
   ("609.7b's property recheck needs nothing here"). Two things RD-3 should
   still pin with tests rather than inherit: (i) a count that has already
   been partially spent on a matching source, then meets the same source
   after it stopped matching, keeps its remainder (`spend_next_damage` is
   only reached through an application, and the remainder is in the row, not
   in the instance snapshot); (ii) a multi-source batch where the source
   predicate admits one attacker and rejects the other allocates over one
   bucket — no prompt (CR 615.7's "two or more *applicable* sources"), and the
   rejected member's damage is dealt in full. Both fall out of the code as it
   stands; the tests are what stop a later refactor from folding the predicate
   into the row.

## Two notes for a later PR, from Divine Deflection (verified on Scryfall 2026-09-08)

Divine Deflection is *not* RD-2's or RD-3's consumer — it needs
`AmountExpr::Variable` — but it is the card that decided two things about the
row RD-2 built, and they were recorded before the row type was written.

> {X}{W} Instant. Prevent the next X damage that would be dealt to you and/or
> permanents you control this turn. If damage is prevented this way, Divine
> Deflection deals that much damage to any target.

1. **A resolution-created row must keep its targets, and the rider context
   needs both them and the event's subject.** The ruling: *"Divine
   Deflection's only target is the permanent or player it may deal damage to.
   You choose that target as you cast Divine Deflection, not at the time it
   prevents damage."* So the rider's `ResolutionContext` cannot be built from
   the event's subject alone — the damage goes to a target chosen at cast,
   while the *prevented amount* and the affected player come from the event.
   **RD-2 did the half that is a fact**: `RegisteredReplacementEffect.targets`
   is written by `Primitive::CreateReplacement`. The half that is a feature —
   a recipient leaf for "the thing this effect targeted at resolution",
   threaded row → `ReplacementInstance` → `Rider` → the rider's context — is
   `codebase-state.md` item 90 and lands with the card. Two more rulings pin
   the same seam: the damage is Divine Deflection's own, not the original
   source's, and it is not combat damage even when the prevented damage was —
   so this is a rider, never a `Retarget`.

2. **Target legality is the rider runner's question, and the answer is a
   no-op — not the performer's.** The ruling: *"If Divine Deflection can't deal
   damage to the targeted permanent or player … it will still prevent damage.
   It just won't deal any damage itself."* The prevention is unconditional
   (CR 615.12); the rider's own action drops. So the check belongs where the
   rider resolves, and it must **not** be `Destroy`-style loudness from
   `perform_action`, which would fail the batch. And it is an *existence and
   type* check, not CR 608.2b's legality re-check: *"Whether the targeted
   permanent or player is still a legal target is not checked after Divine
   Deflection resolves."* A rider that asked `validate_selection` would get
   shroud wrong.

## Next step

RD-3: branch `replacement/rd-3-sources` off `origin/main`, in the commit order
`register-a-card-only-once-the-engine-can-play-it` gives — scaffolding, cards
unregistered, red tests, fix, registration + `PERFORMANCE_POOL` (Guardian
Seraph) + A/B, docs. Read note 1 above before designing the source choice.

Delete this file in the last RD PR to land.
