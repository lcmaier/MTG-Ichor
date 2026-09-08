# Phase RD — damage (CR 615, 609.7, 614.9, 120.3)

**State (2026-09-08): designed and sized, no code written. Waiting on the
owner's review of the design check before RD-1 starts.**

## Where the work is

- `plans/replacement-architecture.md` §9, "Phase RD" — the design check
  (seven decisions), the four-PR table with measured call-site counts, each
  PR's consumers with their rulings passes, the atoms each claims, the
  measurement plan and the exit criteria. **Read this first.**
- `plans/replacement-architecture.md` §11 items 21–27 — the findings the
  sizing produced (player scoping is RD's, consume-after-apply, Harm's Way's
  split, item 15's answer, unpreventable as an event property, the
  "can't be prevented" family's missing halves, CR 120.3's eight results).
- `plans/cant-effects-architecture.md` §4.7 — the superseding note on
  `ReplacementKind`.
- `plans/roadmap-v2.md` row A4 — the size cell.

## The decisions that change the shape, for the review

0. `ReplacementDef.affected_players: PlayerSet` — a second field, not an
   `AffectedSet` variant; RD-1, not RE.
1. Partial prevention is `Rewrite::Amount(PreventUpTo | PreventShield)`;
   `Instead` gets no "N − k" template.
2. Shields are rows in the existing `ReplacementEffectRegistry`, made by
   `Primitive::CreateReplacement(Box<ReplacementDef>, Duration)`;
   `Uses::Shield(remaining)` decrements in place. `ApplyPrevention` is deleted.
3. Decisions are per `(batch, subject)`, rewrites per member; CR 615.7's
   `allocate` prompt ships in RD-2 with Mending Hands.
5. CR 120.3c ships in RD-1 with **Loyalty Probe**, a named fixture
   planeswalker, registered in the stress pool only.
7. `consume_use` moves after `apply_rewrite` and spends what the application
   did (CR 609.7b, 614.9, 615.12).

## Next step

On the owner's go-ahead: branch `replacement/rd-1-damage-subjects` off
`origin/main`, in the commit order
`register-a-card-only-once-the-engine-can-play-it` gives — scaffolding, cards
unregistered, red tests, fix, registration + `PERFORMANCE_POOL` + A/B, docs.

Delete this file in RD-4's landing PR.
