# Phase RD — damage (CR 615, 609.7, 614.9, 120.3)

**State (2026-09-08): designed and sized, no code written. Two review rounds
applied (eight notes, then three); waiting on the go-ahead before RD-1
starts.**

## Where the work is

- `plans/replacement-architecture.md` §9, "Phase RD" — the design check
  (seven decisions plus the shield glossary), the four-PR table with measured
  call-site counts, each PR's consumers with their rulings passes, the atoms
  each claims, RD-5's gate, the measurement plan and the exit criteria.
  **Read this first.**
- `plans/replacement-architecture.md` §11 items 21–28 — the findings the
  sizing produced (player scoping is RD's, consume-after-apply, Harm's Way's
  split and its gate, item 15's answer, unpreventable as an event property,
  the "can't be prevented" family's missing halves, CR 120.3's eight results
  and their owners, rounding is the card's).
- `plans/backlog.md` §2.6 (now names infect/wither/toxic and RD-1's seam) and
  §2.23 (battles, new).
- `plans/cant-effects-architecture.md` §4.7 — the superseding note on
  `ReplacementKind`. `plans/roadmap-v2.md` row A4 — the size cell.

## The decisions that change the shape

0. `ReplacementDef.affected_players: PlayerSet` — a second field, not an
   `AffectedSet` variant; RD-1, not RE.
1. `AmountRewrite` is six arms: `Multiplier`, `Halve(Rounding)`, `Plus`,
   `PreventUpTo`, `PreventHalf(Rounding)`, `PreventRemaining`. `Rounding` has
   no default (CR 107.1a). `Instead` gets no "N − k" template.
2. CR 615.7 shields are rows in the existing `ReplacementEffectRegistry`, made
   by `Primitive::CreateReplacement(Box<ReplacementDef>, Duration)`, with
   `Uses::NextDamage(remaining)` — the rule's own phrase, a count of damage
   and never of uses — decremented in place. The word "shield" is not reused
   in code; the glossary in §9 says why. `ApplyPrevention` is deleted.
3. Decisions are per `(batch, subject)`, rewrites per member. The CR 615.7
   `allocate` prompt is per *instance* (one row, whatever subjects its members
   have), asked the first time the row is chosen in the batch, and ships in
   RD-2 with Mending Hands.
5. CR 120.3c ships in RD-1 with **Loyalty Probe**, a named fixture
   planeswalker, registered in the stress pool only.
7. `consume_use` moves after `apply_rewrite` and spends what the application
   did (CR 609.7b, 614.9, 615.12).

## Consumers, by PR

- **RD-1** — Furnace of Rath, Ghosts of the Innocent, Gisela, Blade of
  Goldnight, Angel of Suffering, Loyalty Probe.
- **RD-2** — Mending Hands, Samite Healer, Safe Passage, Samite Censer-Bearer.
- **RD-3** — Circle of Protection: Red, Reverse Damage, Guardian Seraph,
  Daunting Defender + Pyroclasm, Fog, Torbran, Thane of Red Fell, Dark Sphere.
- **RD-4** — Pariah, Palisade Giant, Pinpoint Avalanche.
- **RD-5 (candidate)** — Harm's Way, gated on the split staying off the
  per-member path.

## Next step

On the owner's go-ahead: branch `replacement/rd-1-damage-subjects` off
`origin/main`, in the commit order
`register-a-card-only-once-the-engine-can-play-it` gives — scaffolding, cards
unregistered, red tests, fix, registration + `PERFORMANCE_POOL` + A/B, docs.

Delete this file in the last RD PR to land.
