# Phase RD — damage (CR 615, 609.7, 614.9, 120.3)

**State (2026-09-08): RD-1 landed. RD-2 is next and is the phase's highest-risk
PR — the only one that changes the CR 616.1 loop's unit, and the one whose
defect shape is a silent wrong choice rather than an error.**

## Where the work is

- `plans/replacement-architecture.md` §9, "Phase RD" — the design check
  (seven decisions plus the shield glossary), the four-PR table with measured
  call-site counts, each PR's consumers with their rulings passes, the atoms
  each claims, RD-5's gate, the measurement plan and the exit criteria.
  **Read this first.** RD-1's section carries an "As landed" block with what
  the build changed and what the A/B measured.
- `plans/replacement-architecture.md` §11 items 21–28 — the findings the
  sizing produced. 21 and 27 are closed by RD-1; 22, 24 and 25 carry a note
  saying why RD-1 left them alone.
- `plans/backlog.md` §2.6 and §2.23 — the CR 120.3 results RD does not ship,
  and the seam RD-1 cut for them (`engine::actions::DamageResults`).
  `codebase-state.md` items 86 and 87 are the dated lines.
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

## The decisions that change the shape

0. ~~`ReplacementDef.affected_players: PlayerSet`~~ — **landed in RD-1**.
1. `AmountRewrite` is six arms: three landed in RD-1; `PreventUpTo` and
   `PreventRemaining` are RD-2's, `Plus` is RD-3's. `Rounding` has no default
   (CR 107.1a). `Instead` gets no "N − k" template.
2. CR 615.7 shields are rows in the existing `ReplacementEffectRegistry`, made
   by `Primitive::CreateReplacement(Box<ReplacementDef>, Duration)`, with
   `Uses::NextDamage(remaining)` — the rule's own phrase, a count of damage
   and never of uses — decremented in place. The word "shield" is not reused
   in code; the glossary in §9 says why. `ApplyPrevention` is deleted.
3. Decisions are per `(batch, subject)`, rewrites per member. The CR 615.7
   `allocate` prompt is per *instance* (one row, whatever subjects its members
   have), asked the first time the row is chosen in the batch, and ships in
   RD-2 with Mending Hands.
5. ~~CR 120.3c ships in RD-1 with **Loyalty Probe**~~ — **landed**, and the
   fixture is a genuine consumer: CR 704.5i fires 4× in 400 stress games.
7. `consume_use` moves after `apply_rewrite` and spends what the application
   did (CR 609.7b, 614.9, 615.12). RD-2's, and RD-1 deliberately left the
   order alone — nothing it ships can be chosen and then do nothing.

## Consumers, by PR

- ~~**RD-1**~~ — landed: Furnace of Rath, Ghosts of the Innocent, Gisela,
  Blade of Goldnight, Angel of Suffering, Loyalty Probe.
- **RD-2** — Mending Hands, Samite Healer, Safe Passage, Samite Censer-Bearer.
- **RD-3** — Circle of Protection: Red, Reverse Damage, Guardian Seraph,
  Daunting Defender + Pyroclasm, Fog, Torbran, Thane of Red Fell, Dark Sphere.
- **RD-4** — Pariah, Palisade Giant, Pinpoint Avalanche.
- **RD-5 (candidate)** — Harm's Way, gated on the split staying off the
  per-member path.

## Two notes for RD-2, from Divine Deflection (verified on Scryfall 2026-09-08)

Divine Deflection is *not* RD-2's consumer — it needs `AmountExpr::Variable`
— but it is the card that decides two things about the shape RD-2 builds, so
they are recorded before the row type is written rather than after.

> {X}{W} Instant. Prevent the next X damage that would be dealt to you and/or
> permanents you control this turn. If damage is prevented this way, Divine
> Deflection deals that much damage to any target.

1. **A resolution-created row must keep its targets, and the rider context
   needs both them and the event's subject.** The ruling: *"Divine Deflection's
   only target is the permanent or player it may deal damage to. You choose
   that target as you cast Divine Deflection, not at the time it prevents
   damage."* So the rider's `ResolutionContext` cannot be built from the
   event's subject alone — the way `resolve_rider` builds it today — because
   the damage goes to a target chosen at cast, while the *prevented amount* and
   the affected player come from the event. **A recipient leaf for each**: the
   `then` needs one way to say "the thing this effect targeted at resolution"
   and another to say "the thing the event was about", and today
   `EffectRecipient::Target` means the second. `Primitive::CreateReplacement`
   should therefore carry the resolution's targets onto the row, and
   `Rider` should carry them beside `subject`.

   Two more rulings pin the same seam: the damage is Divine Deflection's own,
   not the original source's (*"not a redirection effect … the characteristics
   of the original source … don't affect this damage"*), and it is not combat
   damage even when the prevented damage was — so this is a rider, never a
   `Retarget`.

2. **Target legality is the rider runner's question, and the answer is a
   no-op — not the performer's.** The ruling: *"If Divine Deflection can't deal
   damage to the targeted permanent or player (because the creature is no
   longer on the battlefield, or is no longer a creature, or the player is no
   longer in the game, for example), it will still prevent damage. It just
   won't deal any damage itself."* The prevention is unconditional (CR 615.12,
   the rule RD-1 already relies on); the rider's own action drops. So the check
   belongs where the rider resolves, and it must **not** be `Destroy`-style
   loudness from `perform_action`, which would fail the batch.

   And it is an *existence and type* check, not CR 608.2b's legality re-check:
   *"Whether the targeted permanent or player is still a legal target is not
   checked after Divine Deflection resolves. For example, if a creature
   targeted by Divine Deflection gains shroud … Divine Deflection can still
   deal damage to that creature."* A rider that asked `validate_selection`
   would get shroud wrong.

## Next step

RD-2: branch `replacement/rd-2-prevention-shields` off `origin/main`, in the
commit order `register-a-card-only-once-the-engine-can-play-it` gives —
scaffolding, cards unregistered, red tests, fix, registration +
`PERFORMANCE_POOL` (Mending Hands) + A/B, docs. The trace-page decision is
taken at RD-2's close (`engineering-practices.md` §7).

Delete this file in the last RD PR to land.
