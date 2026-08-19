# Session Handoff: Phase LD-1 — Layer 4 (Type-Changing Effects)

**Project:** MTG rules engine in Rust at `c:\Users\maier\Desktop\MTG Simulator\mtgsim_v2\mtgsim`.
**Test count:** 509 tests pass (415 unit + 93 integration + 1 doc-test), 0 warnings.

> **STATUS (2026-08-19): Part A is DONE and committed.** See commit
> "Phase LD: Added Layer 4 (Type-changing effects) functionality".
> `TypeChange` has `set_types`/`set_subtypes`/`set_supertypes`,
> `Primitive::ChangeType` resolves into sibling `ContinuousEffect` entries at
> `Layer4Type`, `register_static_effects` handles `ChangeType`,
> `PermanentFilter::BySupertype` and `EffectModification::SetSupertypes` exist,
> and the three oracle wrappers are in place. Test cards live in
> `src/cards/phase_ld_cards.rs`; 10 integration tests in
> `tests/phase_ld_integration_test.rs`.
>
> **Part B is NOT done.** `AbilityOrigin` does not exist anywhere in `src/`.
> Blood Moon currently replaces land subtypes correctly, but does **not**
> strip printed abilities or grant intrinsic mana abilities for the new basic
> land types — a Blood-Mooned dual land still taps for its original colors.
> **Resume at "Part 2: CR 305.7" below.**

**Current state (as originally written, pre-Part-A):** Phase LC is complete. Layer 5 (color-changing effects) is fully working — `ColorChange` enum (`Add`, `Set`, `RemoveAll`) maps to `EffectModification` variants, resolution registers `ContinuousEffect` at `Layer::Layer5Color`, static abilities register via `register_static_effects`, all effects expire properly. Test cards: Cerulean Wisps, Moonlace, Chromatic Ward.

---

## What Needs to Happen Next

Implement Layer 4 (type-changing effects). This is significantly more complex than Layer 5 because of **CR 305.7** (Blood Moon semantics) — setting a land's subtype to a basic land type has side effects on abilities.

---

## Part 1: Basic Type-Changing Operations (simpler, do first)

### Compute side (already done)
`apply_modification` in `compute.rs` already handles all Layer 4 `EffectModification` variants:
- `AddType(CardType)` / `RemoveType(CardType)` / `SetTypes(HashSet<CardType>)`
- `AddSubtype(Subtype)` / `RemoveSubtype(Subtype)` / `SetSubtypes(HashSet<Subtype>)`
- `AddSupertype(Supertype)` / `RemoveSupertype(Supertype)`

### Resolution side (needs work)
`Primitive::ChangeType(TypeChange, Duration)` is stubbed in `resolve.rs`. The current `TypeChange` struct:
```rust
pub struct TypeChange {
    pub add_types: Vec<CardType>,
    pub remove_types: Vec<CardType>,
    pub add_subtypes: Vec<Subtype>,
    pub remove_subtypes: Vec<Subtype>,
}
```

**Design decision needed:** `TypeChange` currently only supports add/remove operations, but the `EffectModification` enum also has `SetTypes` / `SetSubtypes` / `SetSubtypes` (overwrite) variants. Options:
1. Add `set_types: Option<HashSet<CardType>>`, `set_subtypes: Option<HashSet<Subtype>>`, etc. fields to the struct
2. Refactor to an enum like `ColorChange` was (but type changes are more complex — a single card effect can add types AND remove subtypes simultaneously)
3. Keep the struct but add set fields alongside add/remove fields, with set taking priority when present

Option 3 is probably cleanest — the struct already handles the composite nature of type changes. Add optional set fields.

Resolution follows the same pattern as `ChangeColor` — collect battlefield targets, allocate timestamp, register `ContinuousEffect` at `Layer::Layer4Type`. **However**, a single `TypeChange` may need to register **multiple** `ContinuousEffect` entries (one per EffectModification variant) since each variant belongs to one layer operation. Or, register a single effect with a composite modification. Check how multi-layer card text is handled (layers-architecture.md §3.3: split into sibling entries sharing timestamp+source per CR 613.1c).

### Static ability registration
`register_static_effects` in `game_state.rs` needs a new match arm for `ChangeType`, same pattern as `ChangeColor`.

### Oracle wrappers needed
- `get_effective_types(game, id) -> HashSet<CardType>`
- `get_effective_subtypes(game, id) -> HashSet<Subtype>`
- `get_effective_supertypes(game, id) -> HashSet<Supertype>`

These may already be partially covered by existing code — check before adding.

---

## Part 2: CR 305.7 — Blood Moon Semantics (complex, do second)

**Rule 305.7:** "If an effect sets a land's subtype to one or more basic land types, the land no longer has its old land type. It loses all abilities generated from its rules text and its old land types, and it gains the appropriate intrinsic mana ability for each new basic land type."

### What this means in practice
When `EffectModification::SetSubtypes(new_subtypes)` is applied to a Land in Layer 4, and `new_subtypes` contains any basic land type (Plains/Island/Swamp/Mountain/Forest):

1. **Replace subtypes** — old land subtypes are gone, replaced by `new_subtypes`
2. **Strip rules-text abilities** — abilities printed on the card are removed
3. **Strip old intrinsic mana abilities** — mana abilities granted by old basic land subtypes
4. **Grant new intrinsic mana abilities** — for each new basic land subtype: Plains→{W}, Island→{U}, Swamp→{B}, Mountain→{R}, Forest→{G}
5. **Keep Layer 6 granted abilities** — abilities from other effects survive
6. **Don't touch card types** — still a Land
7. **Don't touch supertypes** — Legendary/Basic/Snow remain. **Blood Moon does NOT add Basic supertype** (ATOM-305.8-001, PC12)

### Prerequisite: AbilityOrigin tracking

**This does not exist yet.** `AbilityDef` currently has no `origin` field. Per `layers-architecture.md` §15.2 item 4, the needed variants are:
```rust
pub enum AbilityOrigin {
    /// Printed in the card's rules text
    PrintedRulesText,
    /// Intrinsic mana ability from a basic land type (rule 305.6)
    IntrinsicLandType(Subtype),
    /// Granted by a continuous effect (Layer 6)
    LayerGranted(EffectId),
}
```

Add `origin: AbilityOrigin` to `AbilityDef` (default `PrintedRulesText` for all existing cards). The `EffectiveCharacteristics.abilities` list also carries this field, so `apply_modification` for `SetSubtypes` can selectively strip `PrintedRulesText` and `IntrinsicLandType` abilities while preserving `LayerGranted`.

### Where the 305.7 logic lives
Inside `apply_modification` in `compute.rs`, special-cased on `SetSubtypes` when the target is a Land. Per layers-architecture.md §3.5a: "This is application-time logic, local to Layer 4's `apply()`."

The algorithm (from §3.5a):
```
if new_subtypes contains any basic land subtype AND object has type Land:
    1. Set effective subtypes to new_subtypes
    2. Strip abilities where origin == PrintedRulesText
    3. Strip abilities where origin == IntrinsicLandType(_)
    4. Grant intrinsic mana ability for each basic land subtype in new_subtypes
    5. Don't touch types or supertypes
else:
    // non-basic subtypes: set subtypes normally, no stripping
```

### "In addition to" variant (rule 305.7, ATOM-305.7-005)
Effects like Urborg ("each land is a Swamp **in addition to** its other land types") use `AddSubtype` (not `SetSubtypes`). When adding a basic land subtype:
- Old types are **kept**
- Old abilities are **kept**
- The intrinsic mana ability for the new type is **added**
- Per ATOM-305.6-002, this is handled in the `AddSubtype` arm, not `SetSubtypes`

---

## Implementation Order

Suggested two-part approach:

### Part A: Basic type operations (no 305.7)
1. Expand `TypeChange` struct or refactor (design decision)
2. Implement `ChangeType` resolution in `resolve.rs`
3. Add `register_static_effects` match arm for `ChangeType`
4. Add oracle wrappers (`get_effective_types`, `get_effective_subtypes`)
5. Test cards: simple type-adding effect (e.g., made-up "Ensoul Artifact" style: "target artifact becomes an artifact creature"), or Spreading Seas ({1}{U} enchant land, "enchanted land is an Island")
6. Unit tests in `compute.rs` for basic L4 operations
7. Integration tests for type-changing spells

### Part B: CR 305.7 + AbilityOrigin
1. Add `AbilityOrigin` enum to `card_data.rs`
2. Add `origin: AbilityOrigin` field to `AbilityDef` (default `PrintedRulesText`)
3. Implement 305.7 special case in `apply_modification` for `SetSubtypes`
4. Implement intrinsic mana ability granting for `AddSubtype` (basic land types)
5. Test cards: Blood Moon ({2}{R} Enchantment: "Nonbasic lands are Mountains"), Urborg-style effect
6. Tests: ATOM-305.7-001 through 305.7-005, ATOM-305.8-001, COMP-305.7+305.6-001

---

## Key Files

- `src/engine/layers/compute.rs` — `apply_modification` already handles L4 variants; 305.7 logic goes here
- `src/engine/layers/types.rs` — `EffectModification` L4 variants already defined; `EffectiveCharacteristics` already has types/subtypes/supertypes fields
- `src/engine/resolve.rs` — `Primitive::ChangeType` stub needs real implementation
- `src/state/game_state.rs` — `register_static_effects` needs L4 match arm
- `src/types/effects.rs` — `TypeChange` struct may need expansion; `Primitive::ChangeType` definition
- `src/types/card_types.rs` — `LandType::is_basic_land_type()` already exists (returns true for Plains/Island/Swamp/Mountain/Forest)
- `src/objects/card_data.rs` — `AbilityDef` needs `origin: AbilityOrigin` field (Part B)
- `src/oracle/characteristics.rs` — new oracle wrappers
- `src/cards/phase_lc_cards.rs` or new `phase_ld_cards.rs` — test card definitions
- `tests/phase_ld_integration_test.rs` — integration tests

## Reference Documents

- `plans/layers-architecture.md` §3.5a — CR 305.7 algorithm
- `plans/layers-architecture.md` §5 — compute_characteristics data flow (per-layer filter re-evaluation)
- `plans/layers-architecture.md` §15.2 item 4 — AbilityOrigin enum shape
- `plans/archive/implementation-plan-final.md` L10 ticket — detailed steps for Layers 4-5
- `plans/atomic-tests/sessions/session-3.md` — ATOM-305.7-001 through 305.7-005, ATOM-305.8-001
- `plans/atomic-tests/phase-index-phase-5-layers.md` — L10/L17 atom index

## Constraints

- Don't implement the dependency algorithm — timestamp ordering is sufficient for Layer 4 in isolation.
- Don't implement Layer 3 (text-changing) or Layer 1 (copy/face-down) in this session.
- Don't implement Layer 6 (abilities) resolution — GrantKeyword is still stubbed; that's a separate session.
- Part A can be done without AbilityOrigin; Part B requires it.
- Blood Moon's filter ("nonbasic lands") requires checking supertypes — a land is nonbasic if it does NOT have `Supertype::Basic`. The filter `PermanentFilter::Not(Box::new(PermanentFilter::And(Box::new(PermanentFilter::ByType(CardType::Land)), Box::new(PermanentFilter::BySupertype(Supertype::Basic)))))` may need a new `PermanentFilter::BySupertype` variant (check if it exists).
- Small commits. Don't refactor speculatively.

## Planned layer ordering after this session
L4 (this session) → L6 (abilities, Humility) → L2 (control) → dependency algorithm → L3 (scaffold) → L1 (scaffold, deferred to Phase 6/9)
