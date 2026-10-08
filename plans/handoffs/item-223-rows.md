# Item 223's row half — the rows a resolution makes, and the sweep they need

**Resumes in TR-3b**, as a row of its pieces table (`triggers-architecture.md`,
"TR-3b"). The owner split `codebase-state.md` item 223 at PR #233's design
review (2026-10-08): #233 landed the half whose effects are events (damage,
life, mana, destruction, "its owner"), and this file holds the half whose
effects are registry rows, so it cannot dangle. **Delete this file in TR-3b's
landing PR**, with items 223 and 95 closed there.

## What is left

- **The ten sites** in `engine/resolve.rs` that write a registry row's
  `source`: the rows of `ModifyPowerToughness`, `SetPowerToughness`,
  `SwitchPowerToughness`, `ChangeColor`, `ChangeType` and `GainControl`; the
  Layer 6 helper `register_resolution_ability_effect`; `apply_copy`;
  `Regenerate` (item 95); and `Restrict`. Each reads
  `ResolutionContext::effect_source`, which #233 added, in place of
  `ctx.source`, the stack object CR 608.2n removes.
- **The battlefield sweep.** When a permanent leaves the battlefield,
  `cleanup_zone_state` (`engine/zones.rs`) calls the broad `remove_by_source`,
  which drops every continuous-effect row naming it, a resolution's included.
  Once a resolution's row names the permanent, that would end "target creature
  gains flying until end of turn" the moment the ability's source died, against
  CR 611.2a. Narrow it to the rows that end with their source: a static
  ability's (CR 611.3b, what `remove_static_by_source` already selects) and a
  resolution's whose duration is "for as long as" its source (CR 611.2b:
  `WhileSourceOnBattlefield`, `WhileEnchanted`, `WhileEquipped`).
- **The look-back gate.** `RegistryScopeSummary::ability_list_sources` takes the
  same predicate, or a pooled pump's source dying takes look-back snapshots
  nobody needs.

Replacement and restriction rows are never swept by source (`zones.rs`'s
`cleanup_zone_state` says why), so a regeneration shield or a "can't" naming
the permanent already outlives it.

## Tests

- A pump an activated ability made outlives its source's death. Passes on
  `main`, where the row names the stack object, and fails on a tree that moves
  the sites without the sweep: the regression this file exists to prevent.
- A "for as long as" row a resolution made ends as its source leaves (Sower of
  Temptation's shape). Fails on `main`, where the row names the stack object
  and never ends.
- A regeneration shield from an activated ability names the permanent at CR
  616.1's prompt (item 95).

## Size and prediction

About 170 lines with the tests (#233's design note). Gameplay `IDENTICAL` on
both pools: once the sweep is narrowed, nothing that decides play keys on a
resolution row's source.
