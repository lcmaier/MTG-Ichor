# Layers Architecture — CR 613 Continuous Effects

**Status:** design doc. Frozen decisions + types + module layout + work-phase plan for the Layer System (CR 613).

**Authoritative for:** type shapes, module boundaries, sublayer enumeration, dependency-algorithm sketch, work-phase sequencing.

**Not authoritative for:** per-card mechanics (see `atomic-tests/phase-index-phase-5-layers.md`), every `EffectModification` variant we'll eventually need (the list is bounded by the CR; we enumerate what's foreseeable).

**Companion docs:**
- `codebase-state.md` — ground-truth of current coverage + deferred migrations.
- `design_doc.md:636-664` — prior hybrid dependency algorithm sketch (adopted verbatim below).
- `atomic-tests/phase-index-phase-5-layers.md` — 178-row test index (scope grounding per slice).
- `atomic-tests/pass0-dependency-map.md` §8 — cross-cutting architecture decisions.

Last updated: 2026-04-18.

---

## 1. Purpose & Non-Goals

### Purpose

Produce a fully specified design for the engine's implementation of **CR 613 (Interaction of Continuous Effects)**, the "layer system". When a subsequent Cascade session picks this up, it should be able to execute phases LA → LD directly from this document without reopening design questions.

### Non-Goals

- **Not a tutorial on CR 613.** The reader is assumed to have the CR open (or quoted inline where decisions hinge on specific subrule text).
- **Not a card-by-card implementation plan.** Cards are tracked via atomic-test indexes and `cards-unlocked-ledger.md`.
- **Not final commitments on per-variant enum shapes.** We enumerate what's foreseeable; new variants land as cards demand them.
- **Not a schedule.** Work-phase ordering is fixed (LA → LD must happen in sequence). Calendar time is whatever it is.

---

## 2. Terminology

We use these terms consistently:

- **Layer** — one of CR 613's seven top-level layers (1: copy, 2: control, 3: text, 4: type, 5: color, 6: ability, 7: P/T). Sometimes we say "layer N" for shorthand.
- **Sublayer** — Layer 1 and Layer 7 have sublayers (1a, 1b; 7a, 7b, 7c, 7d).
- **Continuous effect** — an effect that modifies characteristics of one or more objects for some duration. CR 611.
- **CDA** — characteristic-defining ability (CR 604.3). Its effect applies in all zones, and within a layer it orders *before* non-CDA effects (CR 613.6).
- **Timestamp** — a monotonic integer assigned to a continuous effect when it becomes active, used to break ties within a layer. CR 613.7.
- **Dependency** — Effect A depends on Effect B if applying B changes what A affects, modifies, or produces. CR 613.8. Within a layer, dependency-ordering trumps timestamp-ordering.
- **Affected set** — the set of objects a continuous effect applies to, determined by evaluating its predicate against the current game state.
- **Base characteristics** — the object's printed values (`CardData`) plus zone-specific overrides that don't live in the layer system (e.g., morph face-down values are layer-system content, but `BattlefieldEntity.counters` feed into layer 7d directly).
- **Effective characteristics** — the output of walking all applicable continuous effects in layer order. What the rest of the engine queries.
- **Phase L[A–D]** — work phases for landing this system incrementally. Distinct from CR "layers".

---

## 3. Type Surface

All definitions are Rust-like pseudocode. Final syntax may differ in trivial ways (lifetimes, derives); semantics are frozen.

### 3.1 `Layer` / `Sublayer`

```rust
/// CR 613 layers, including sublayers. Ordered by the sort key
/// `Ord::cmp` so a sorted `Vec<ContinuousEffect>` applies in rules order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layer {
    /// Layer 1a — face-down effects (CR 613.2a).
    /// Turns objects face-down (morph, manifest, cloak, disguise).
    Layer1aFaceDown,
    /// Layer 1b — copy effects (CR 613.2b).
    Layer1bCopy,
    /// Layer 2 — control-changing effects (CR 613.3).
    Layer2Control,
    /// Layer 3 — text-changing effects.
    Layer3Text,
    /// Layer 4 — type-changing effects (types, subtypes, supertypes).
    Layer4Type,
    /// Layer 5 — color-changing effects.
    Layer5Color,
    /// Layer 6 — ability-adding and ability-removing effects.
    Layer6Ability,
    /// Layer 7a — CDA P/T (CR 613.4a). E.g., Tarmogoyf.
    Layer7aCdaPT,
    /// Layer 7b — effects that set P/T to specific values (CR 613.4b).
    Layer7bSetPT,
    /// Layer 7c — P/T modifications (CR 613.4c).
    /// Anthems, +N/+N pumps, and counters (+1/+1, -1/-1, NOT keyword counters though).
    Layer7cModifyPT,
    /// Layer 7d — switch P/T (CR 613.4d).
    Layer7dSwitchPT,
}
```

**Decision — counter effects are derived, not stored.** Counters remain written to `BattlefieldEntity.counters`. When `compute_characteristics` walks layer 7c, it synthesizes virtual `ContinuousEffect`s from the counter map and orders them *after* any non-counter 7c modifiers (per CR 613.4c's "however, effects that modify power or toughness (with the exception of counters) are applied before counters"). This avoids double-booking state between the counter map and the effect registry.

**Decision — face-down is also derived.** `BattlefieldEntity.face_down: bool` already exists. When set, `compute_characteristics` synthesizes a cluster of Layer-1a effects at compute time (set colorless, lose all abilities, become 2/2 creature with no name). No effect-registry entries needed for vanilla face-down; effects that *turn something face-down* (the action) are registered in Layer 1a, and their application side-effect is flipping the flag.

### 3.2 `EffectiveCharacteristics`

```rust
/// Output of `compute_characteristics(game, id)`. Mirrors `CardData`'s
/// user-queryable fields plus zone-derived characteristics.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveCharacteristics {
    // --- Identity ---
    pub name: String,
    pub mana_cost: Option<ManaCost>,
    pub color_indicator: Option<Vec<Color>>,

    // --- Types (layer 4) ---
    pub types: HashSet<CardType>,
    pub subtypes: HashSet<Subtype>,
    pub supertypes: HashSet<Supertype>,

    // --- Colors (layer 5) ---
    pub colors: HashSet<Color>,

    // --- Abilities (layer 6) ---
    /// All abilities (keyword and non-keyword). Keywords are represented
    /// as `AbilityDef::Keyword(KeywordAbility)` variants; there is no
    /// separate `keywords` set — keywords are abilities per CR 702.
    /// `has_keyword(id, kw)` is derived by scanning this vec.
    pub abilities: Vec<AbilityDef>,

    // --- P/T (layer 7a-d) ---
    pub power: Option<i32>,
    pub toughness: Option<i32>,

    // --- Control (layer 2) ---
    pub controller: PlayerId,

    // --- Planeswalker/Battle ---
    pub loyalty: Option<u32>,
    pub defense: Option<u32>,

    // --- Text box (layer 3) ---
    pub text: Option<String>,

    // --- Derived flag (layer 1a) ---
    /// Whether the object is currently face-down. Derived from
    /// `BattlefieldEntity.face_down`. No back-pointer to a copy source
    /// is stored: copy effects (layer 1b) lock characteristics in at
    /// resolution time (CR 707.2 — the copy's copiable values are the
    /// copied object's copiable values as they exist at the moment
    /// the copy is made).
    pub face_down: bool,
}
```

### 3.3 `ContinuousEffect`

```rust
/// A single active continuous effect. Lives in `ContinuousEffectRegistry`.
#[derive(Debug, Clone)]
pub struct ContinuousEffect {
    /// Stable ID for this effect (distinct from source ObjectId —
    /// one source may produce many effects).
    pub id: EffectId,

    /// The object that generates this effect. May be on the
    /// battlefield, in the command zone (for emblems/commanders), etc.
    /// When the source leaves its functional zone, `Duration::WhileSourceActive`
    /// effects are removed.
    pub source: ObjectId,

    /// Which layer/sublayer this effect applies in.
    pub layer: Layer,

    /// True if this effect comes from a CDA (CR 604.3). CDAs sort
    /// before non-CDAs within the same layer and are handled specially
    /// by dependency detection (CR 613.8a(c)).
    pub is_cda: bool,

    /// When the effect becomes inactive.
    pub duration: Duration,

    /// Timestamp assigned when the effect became active (CR 613.7).
    /// Unique across the whole game — `GameState.next_timestamp` is a
    /// monotonic counter incremented for each assignment. APNAP tie-
    /// breaking for simultaneously-created effects is resolved at
    /// assignment time (active-player effects get smaller timestamps
    /// than non-active-player effects in the same batch); see §8.
    pub timestamp: Timestamp,

    /// Predicate selecting which objects this effect applies to.
    /// Evaluated against the frame state at the start of this effect's
    /// layer (§5). Not re-entered for `compute_characteristics` of any
    /// object during evaluation — predicates read the frame directly.
    pub affected: AffectedSet,

    /// What the effect does to each affected object.
    pub modification: EffectModification,

    /// What characteristic categories the `affected` predicate reads.
    /// Pre-computed at effect creation for the dependency algorithm's
    /// cheap static-check step (§9). Distinct from the effect's own
    /// output category, which is derived from `layer` (see `Layer::category`
    /// in §3.5) — no need to store it twice.
    ///
    /// Example: a Layer 6 effect "Creatures you control gain flying"
    /// has `layer = Layer6Ability` (output = Ability) and
    /// `filter_reads = {Type, Control}` (the filter inspects types and
    /// controller). Dependency step 2 fires if some other effect's
    /// output category is in this set.
    pub filter_reads: HashSet<CharacteristicCategory>,
}

pub type EffectId = u64;
pub type Timestamp = u64;
```

**Why `modifies` is not a stored field:** each **registry entry** operates in exactly one layer, so its output category is `layer.category()` — no separate field needed. What *is* stored is `filter_reads`, because the filter's reads are distinct from the layer's output.

**Multi-layer card text** is split at registration time per **CR 613.1c** ("If an effect should be applied in different layers or sublayers, the parts of the effect each apply in their appropriate ones."). So a spell whose resolving effect text is "target creature gets +1/+1 and becomes the color of your choice until end of turn" registers **two sibling registry entries** at resolution:

- one Layer 5 `AddColor(chosen)` with `AffectedSet::Fixed(vec![target])`, `Duration::UntilEndOfTurn`;
- one Layer 7c `ModifyPowerToughness { +1, +1 }` with the same `affected` and `duration`.

The two share `source` and `timestamp` (they are created simultaneously; the timestamp counter increments once per batch, not per registered entry) so their ordering within their respective layers is consistent. The resolution code (a helper like `register_multi_layer(...)`) does this splitting. Card authors never construct a single effect touching multiple layers.

### 3.4 `AffectedSet`

```rust
/// Selects objects for a continuous effect to apply to.
pub enum AffectedSet {
    /// "This permanent" (the source itself). Layer 6 static abilities
    /// like "this creature has flying" use this.
    SourceOnly,
    /// "Creatures you control" etc. Data-driven filter — reuse the
    /// existing `SelectionFilter` infrastructure from targeting.
    Filter(SelectionFilter),
    /// A concrete set captured at effect creation time. Pump spells
    /// ("target creature gets +2/+2 UEOT") use this — the target is
    /// fixed at resolution, not re-queried.
    Fixed(Vec<ObjectId>),
}
```

**Decision:** reuse `SelectionFilter` (data-driven) rather than function pointers. Rationale: serialization, debugging, and the dependency-algorithm static check can inspect filter fields. Function pointers would force the dependency algorithm into the "hypothetical check" path unnecessarily often.

### 3.5 `EffectModification` + `CharacteristicCategory`

```rust
/// The mutation applied to each affected object by a single registry
/// entry. Each variant belongs to exactly one layer. Multi-layer card
/// text ("becomes a 0/0 Golem artifact creature that loses all abilities")
/// is split at registration into sibling entries sharing timestamp and
/// source (see CR 613.1c note in §3.3): a Layer 4 SetTypes, a Layer 6
/// LoseAllAbilities, and a Layer 7b SetPowerToughness.
pub enum EffectModification {
    // --- Layer 1a ---
    /// Turns the affected object face-down. Registered by morph/manifest/
    /// cloak/disguise actions. Its side-effect is flipping
    /// `BattlefieldEntity.face_down`; `compute_characteristics` then
    /// synthesizes the 2/2 no-name no-colors no-abilities cluster.
    TurnFaceDown,

    // --- Layer 1b ---
    /// Copy effect. `copiable` is a snapshot of copiable values at the
    /// moment the copy is made (CR 707.2). Opaque to later layers — once
    /// the copy is applied, it *is* the base for subsequent layers.
    CopyFrom { copiable: CopiableValues },

    // --- Layer 2 ---
    SetController(PlayerId),

    // --- Layer 3 ---
    SetText(String),

    // --- Layer 4 ---
    AddType(CardType),
    RemoveType(CardType),
    SetTypes(HashSet<CardType>),
    AddSubtype(Subtype),
    RemoveSubtype(Subtype),
    /// "[Land] is a [basic land subtype]" — see the CR 305.7 handling
    /// note below. Blood Moon and Urborg use this variant.
    SetSubtypes(HashSet<Subtype>),
    AddSupertype(Supertype),
    RemoveSupertype(Supertype),

    // --- Layer 5 ---
    AddColor(Color),
    SetColors(HashSet<Color>),
    /// "Becomes colorless". Used by both non-CDA effects ("target
    /// creature becomes colorless UEOT") and CDA effects (Devoid).
    /// The `is_cda` flag on the containing `ContinuousEffect`
    /// differentiates; the `EffectModification` variant is the same.
    RemoveAllColors,

    // --- Layer 6 ---
    GrantAbility(AbilityDef),
    LoseAbility(AbilityId),
    LoseAllAbilities,

    // --- Layer 7a (CDA) ---
    /// Set base P/T via CDA. Tarmogoyf, Mortivore, etc.
    /// `is_cda` on the effect must be true.
    CdaSetPT { power: i32, toughness: i32 },

    // --- Layer 7b ---
    SetPowerToughness { power: i32, toughness: i32 },

    // --- Layer 7c ---
    /// +N/+N or -N/-N. Includes counter-derived modifiers synthesized
    /// from `BattlefieldEntity.counters`.
    ModifyPowerToughness { power: i32, toughness: i32 },

    // --- Layer 7d ---
    SwitchPowerToughness,
}

/// Top-level characteristic categories — what an effect's output
/// affects. Used by the dependency algorithm's cheap static-check step.
/// Mapped 1:1 from the top-level of `Layer` (sublayer granularity
/// collapsed: all Layer 7 sublayers → PowerToughness).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacteristicCategory {
    Copy,            // layer 1a + 1b
    Control,         // layer 2
    Text,            // layer 3
    Type,            // layer 4 (includes subtypes + supertypes)
    Color,           // layer 5
    Ability,         // layer 6
    PowerToughness,  // layer 7 (all sublayers)
}

impl Layer {
    pub fn category(self) -> CharacteristicCategory { ... }
}
```

### 3.5a Handling CR 305.7 (Blood Moon semantics)

CR 305.7 is a carve-out of Layer 4 application: setting a land's subtype to one or more basic land types *strips abilities-from-rules-text and old land types and copy effects*, but leaves card types and supertypes alone. This is **not** an architectural concern — it's a special case inside the `apply()` function for `EffectModification::SetSubtypes`.

Algorithm for Layer 4 `apply()` when the modification is `SetSubtypes(new_subtypes)` and the affected object currently has type `Land`:

1. If `new_subtypes` contains any basic land subtype (Plains, Island, Swamp, Mountain, Forest, Wastes):
   - Set effective subtypes to `new_subtypes` (replacing old land subtypes).
   - Strip abilities that came from *rules text* (tracked by ability origin).
   - Strip abilities that came from old land subtypes (intrinsic mana abilities).
   - Grant the intrinsic mana ability for each new basic land subtype.
   - Do **not** touch card types (still Land) or supertypes (Legendary, Basic, Snow remain).
   - Do **not** touch abilities granted by *other effects* (Layer 6 grants survive).
2. Else (non-basic subtypes only): union with existing, no stripping.

This requires `AbilityDef` to track its origin (printed rules text vs. intrinsic land-type vs. layer-granted). Phase LA adds an `AbilityOrigin` enum field on `AbilityDef`. Phase LD (when Layer 4 lands) uses it for 305.7.

**Why this is not an architecture concern:** the type surface above is expressive enough — `SetSubtypes` carries what the effect says (the target subtypes). The CR 305.7 special-case is application-time logic, local to Layer 4's `apply()`, and documented where it lives.

### 3.6 `Duration`

```rust
pub enum Duration {
    /// Effect is active while its source is in its functional zone
    /// (battlefield for most permanents; command zone for emblems and
    /// commander auras that function there; all zones for CDAs).
    /// Replaces the earlier proposed `Static` / `UntilSourceLeaves` pair.
    WhileSourceActive,

    /// Classic "until end of turn" — ends at cleanup step.
    UntilEndOfTurn,

    /// "Until your next turn" — ends at the beginning of this player's
    /// next untap step.
    UntilYourNextTurn,

    /// "Until end of combat" — ends at end of combat step.
    UntilEndOfCombat,

    /// "For as long as X controls Y" — a predicate re-evaluated by SBA.
    /// See `ExpirationPredicate` below.
    WhileCondition(ExpirationPredicate),

    /// Permanent effect generated by a resolving spell that doesn't
    /// specify a duration (rare). Lives until the object receiving it
    /// leaves the battlefield.
    UntilTargetLeavesBattlefield(ObjectId),
}

pub struct ExpirationPredicate {
    /// Data-driven predicate evaluated during SBA. Examples:
    /// "Source controls Target", "Target is tapped", etc.
    pub kind: ExpirationKind,
}
```

### 3.7 `ContinuousEffectRegistry`

```rust
/// Lives on GameState. Owns all active continuous effects.
pub struct ContinuousEffectRegistry {
    effects: Vec<ContinuousEffect>,
    /// Monotonic counter for assigning unique `EffectId`s. Needed because
    /// (a) a single source can register multiple effects (a card with two
    /// static abilities generates two effects with the same `source`);
    /// and (b) callers need a stable handle to remove an individual effect
    /// later — e.g., an UntilEndOfTurn effect must be removed at cleanup
    /// without affecting sibling effects from the same source. Removing
    /// by `source` only would be too coarse.
    next_effect_id: EffectId,
}

impl ContinuousEffectRegistry {
    pub fn add(&mut self, effect: ContinuousEffect) -> EffectId { ... }
    pub fn remove(&mut self, id: EffectId) -> Option<ContinuousEffect> { ... }
    pub fn remove_by_source(&mut self, source: ObjectId) -> Vec<ContinuousEffect> { ... }
    pub fn iter(&self) -> impl Iterator<Item = &ContinuousEffect> { ... }
}
```

---

## 4. Module Layout

```
mtgsim/src/
  state/
    continuous_effects.rs      # ContinuousEffectRegistry struct + EffectId.
                               # Data owner. Lives on GameState.
    game_state.rs              # + field: continuous_effects: ContinuousEffectRegistry
                               # + field: next_timestamp: Timestamp
  engine/
    layers/
      mod.rs                   # Public API. Re-exports types, exposes
                               # compute_characteristics, helpers to
                               # register/remove effects from spell resolution.
      types.rs                 # Layer, Duration, ContinuousEffect, EffectModification,
                               # CharacteristicCategory, AffectedSet, Timestamp,
                               # EffectiveCharacteristics, CopiableValues, AbilityOrigin.
      compute.rs               # compute_characteristics(game, id).
                               # Pure function of (registry, base characteristics).
      dependency.rs            # Hybrid dependency algorithm. Standalone,
                               # unit-testable with synthetic effects.
      lifecycle.rs             # register_effect / deregister_effect helpers
                               # called from resolve.rs, zones.rs, turn cleanup.
  oracle/
    characteristics.rs         # Wrappers (has_keyword, is_creature,
                               # get_effective_power, get_effective_toughness,
                               # get_effective_name, ...) route through
                               # compute_characteristics. No other code reads
                               # CardData directly for battlefield objects.
```

**Rationale for `state/continuous_effects.rs` vs `engine/layers/`:**
Existing codebase split — `state/` holds data structs stored on `GameState` (battlefield, player, mana pool), `engine/` holds behavior. Registry is data, compute is behavior.

---

## 5. Data Flow: `compute_characteristics`

The single entry point all oracle queries route through.

**Critical correctness requirement:** applicable effects are gathered **per layer**, not once upfront. This is why the layer system exists — Layer 4 can change what "is a creature", which determines whether a Layer 6 effect targeting creatures applies. Gathering all applicable effects before layer 1 would miss this.

```
compute_characteristics(game, object_id) -> EffectiveCharacteristics:

  1. frame = EffectiveCharacteristics::from_card_data(game.objects[id].card_data)
     // Printed values. For zones where the pipeline is a no-op (hand,
     // library, etc.), we return frame directly. See §5.1.

  2. Apply Layer 1a (face-down synthesis) derived from
     BattlefieldEntity.face_down, plus any registered TurnFaceDown
     effects (for objects not on battlefield).

  3. For layer in [Layer1bCopy, Layer2Control, Layer3Text,
                   Layer4Type, Layer5Color, Layer6Ability,
                   Layer7aCdaPT, Layer7bSetPT, Layer7cModifyPT,
                   Layer7dSwitchPT]:

       // Predicates are evaluated against the frame *as of the end of
       // the previous layer*. That's why we re-filter each iteration
       // rather than pre-gathering.
       applicable = registry.iter().filter(|e|
           e.layer == layer
           && affected_under_frame(e.affected, object_id, &frame)
       )

       ordered = resolve_order_within_layer(applicable, &frame)

       for effect in ordered:
           apply(&mut frame, effect, object_id)

       // Special case: Layer 7c orders non-counter modifiers before
       // counter-derived modifiers, per CR 613.4c. Implemented inside
       // resolve_order_within_layer by partitioning then concatenating.

  4. return frame
```

The `apply()` function for each `EffectModification` variant mutates `frame` in place.

### 5.1 Zone scope — hidden-zone fast path

Continuous effects can and do touch hidden zones. Mycosynth Lattice ("All cards that aren't on the battlefield, spells, and permanents are colorless") and Painter's Servant ("all cards that aren't on the battlefield ... are the chosen color") are the canonical examples. CDAs apply in all zones per CR 604.3. So there is no rule-level shortcut that lets us skip the pipeline for hidden zones wholesale.

What we *can* do is a **runtime fast path**: in the vast majority of games, no continuous effect targets objects outside the battlefield/stack. Detect that condition cheaply and skip the pipeline when it holds.

**Mechanism:** the `ContinuousEffectRegistry` maintains a summary flag set computed on every `add`/`remove`:

```rust
pub struct RegistryScopeSummary {
    /// True iff any active effect's `affected` could match objects in
    /// hand / library / graveyard / exile. Set on register; cleared on
    /// the last applicable effect's removal.
    pub touches_hidden_zones: bool,
    /// True iff any active effect could match stack objects.
    pub touches_stack: bool,
    /// True iff any active CDA exists at all.
    pub has_active_cdas: bool,
}
```

Then `compute_characteristics(game, id, zone)` dispatches:

| Caller zone | Fast path when... | Slow path otherwise |
|---|---|---|
| Battlefield | never — always slow path | full pipeline |
| Hand / Library / Graveyard / Exile | `!summary.touches_hidden_zones && !summary.has_active_cdas` (or CDA summary rules out relevance) → return printed characteristics | full pipeline, but with a filter that only considers effects whose `affected` can match this zone |
| Stack | `!summary.touches_stack && !summary.has_active_cdas` → return printed characteristics | full pipeline |
| Command | same summary check | full pipeline |

In the common case (no Mycosynth Lattice / Painter's Servant / Leyline of the Void / etc. on the battlefield), hidden-zone queries short-circuit to printed characteristics in O(1). When a hidden-zone-touching effect is active, the pipeline runs over those objects but the effect-filter already narrows the candidate set.

**Consequence for SBA.** SBA reads battlefield permanents only (CR 704 checks are battlefield- or player-scoped). 4-player Commander = ~40 battlefield objects per SBA pass, not ~1600 including libraries.

**Consequence for search-library effects.** If no effect touches the library, characteristic queries during a search are O(1) printed-lookups. If Mycosynth Lattice is active, the queries pay one pipeline walk per library card — still tractable.

**Longer-term optimization (out of scope here, noted as future work):** a pre-game analysis pass over every card in the starting libraries + command zones computes a "universal effect filter" — the set of continuous-effect *patterns* that could ever fire in this game. Effects not in the filter are pre-pruned from dependency checks regardless of runtime state. Tracked as a Phase LE+ possibility, not planned now.

### 5.2 Acyclicity argument

Predicates inside `compute_characteristics(id)` sometimes need to know characteristics of *other* objects. Example: "enchantments you control" needs to know what's an Enchantment. That could naively recurse into `compute_characteristics(other_id)` and loop.

Resolution:

- CR 613 applies effects in layer order against the game state *as modified by prior layers*. So predicates in layer N read characteristics-as-of-end-of-layer-(N-1).
- Predicates never call back into `compute_characteristics`. Instead, they read from a **frame cache** built during this one computation.
- The frame cache is a lazy map `ObjectId -> EffectiveCharacteristics` that accumulates during the current `compute_characteristics` call. When a predicate needs another object's layer-(N-1) characteristics, it requests them from the cache, which runs the pipeline up to layer N-1 for that object (re-entering the same function but with a lower layer-ceiling; bounded recursion).
- Cycles within a layer are broken by CR 613.8b's timestamp fallback (§9).

The frame cache is the *only* cache we maintain, and only within one top-level `compute_characteristics` call. It's discarded on return.

---

## 6. CDA Handling

CR 613.6: CDAs are applied before non-CDA effects in each layer.

Implementation:

1. `ContinuousEffect.is_cda: bool` is set at effect construction.
2. Within each layer, `resolve_order_within_layer` partitions effects into CDAs and non-CDAs.
3. Within each partition, dependency ordering (or timestamp fallback) is applied.
4. CDAs apply first, then non-CDAs.

CDAs are never dependent on non-CDAs (CR 613.8a(c)) — the dependency algorithm's step 3 enforces this by short-circuiting CDA↔non-CDA pairs as independent regardless of static-check results.

CDAs come from printed abilities marked as such on the card (a new field on `AbilityDef`, e.g. `is_characteristic_defining: bool`). Phase LA introduces this field; Phase LB populates it for cards that need 7c interactions (none yet); Phase LC populates it for Tarmogoyf-family cards (Layer 7a CDA P/T).

---

## 7. Layer 1 Sublayers

CR 613.2 splits layer 1 into:

- **1a — face-down effects.** Morph, manifest, cloak, disguise. The *action* that turns an object face-down registers an effect here; the resulting face-down state overrides most characteristics (2/2 colorless creature, no name, no abilities, no mana cost) per CR 708.
- **1b — copy effects.** Clone, Phantasmal Image, etc. Copy effects apply after face-down is resolved, so a Clone entering as a copy of a face-down creature copies the 2/2 colorless characteristics, not the printed card (CR 707.2).

Phase LA ships both variants in the `Layer` enum. Phase LD implements them.

**Implementation note (face-down):** `BattlefieldEntity.face_down: bool` is the canonical state. In Layer 1a, `compute_characteristics` reads the flag and synthesizes the vanilla-face-down characteristics (empty name, colorless, no abilities, P/T 2/2, types={Creature}). Actual `TurnFaceDown` effects (from morph etc.) flip the flag when they apply; they don't store separate per-object state. This mirrors the 7c counter approach — derive from state already owned, don't duplicate it in the registry.

---

## 8. Timestamps

Assignment (CR 613.7c–d):

1. **On ETB** — `BattlefieldEntity.timestamp = game.next_timestamp(); game.next_timestamp += 1;`. Existing infrastructure: the field is already populated (`state/battlefield.rs:80-84`); Phase LA starts *reading* it.
2. **On effect creation** — `ContinuousEffect.timestamp = game.next_timestamp();` at registration.
3. **Re-timestamping on aura/equipment attachment** (CR 613.7e) — when an Aura moves from one creature to another (e.g., via Sun Titan returning it), the Aura's effect timestamp updates. Similarly when a permanent becomes an Aura/Equipment.

Storage: `GameState.next_timestamp: Timestamp` — monotonic counter, never rewound. Saturation not a practical concern (u64).

**APNAP tie-breaking:** CR 613.7d covers the case of *simultaneously-created* effects. We resolve this at assignment time rather than at sort time: when a batch of effects enters together (e.g., two triggered abilities that trigger from the same event), timestamps are assigned in APNAP order during the batch, so the active player's effects get strictly smaller timestamps than the non-active player's. After assignment, timestamps are unique integers and sorting is plain integer comparison. Subsequent controller changes do **not** re-break the tie — timestamps are frozen at assignment.

---

## 9. Dependency Resolution (CR 613.8)

Adopted from `design_doc.md:636-664`, adjusted for `CharacteristicCategory`:

> **Hybrid algorithm:** structural analysis eliminates most pairs cheaply; hypothetical check runs only on candidates.
>
> 1. **Collect** all active effects in this layer/sublayer.
> 2. **CDA guard** (CR 613.8a(c)) — if one is a CDA and the other isn't, they're independent. Cheap bool check; do this before the static check to prune the candidate pool.
> 3. **Static check** — does `B.layer.category()` appear in `A.filter_reads`? If not → independent.
> 4. **Hypothetical check** — temporarily apply B to a frame snapshot, recompute A's `affected`, compare. If different → A depends on B.
> 5. **Build DAG** — edges: B → A (apply B before A).
> 6. **Topological sort** — ties broken by timestamp. Cycles (CR 613.8b) → fall back to timestamp order.

### Interface (Phase LA — signature only; Phase LC — body)

```rust
/// Resolve dependency + timestamp ordering within a single layer partition
/// (CDAs separate from non-CDAs, each called once).
///
/// # Arguments
/// - `effects`: all effects in this (layer, cda-ness) bucket
/// - `frame`: the game state as of end-of-previous-layer, used for
///            hypothetical checks (§5.2)
///
/// # Returns
/// Effects in the order they must be applied.
pub fn resolve_order_within_layer(
    effects: &[&ContinuousEffect],
    frame: &LayerFrame,
) -> Vec<EffectId>;
```

### Hypothetical check implementation

For each candidate pair (A, B) surviving steps 2+3:

1. Snapshot the frame.
2. Apply B to the snapshot.
3. Evaluate A's `affected` against the snapshot.
4. Compare to A's `affected` against the original frame.
5. Different → edge B→A.
6. Discard snapshot.

For performance, the snapshot is a thin overlay (CoW) over the frame, not a deep clone. Phase LC decides the snapshot shape. Phase LA just reserves the interface.

---

## 10. Registry Lifecycle

### Where effects get added

| Source | When | Who calls `register_effect` |
|---|---|---|
| Static ability on a permanent | ETB | `engine/zones.rs::init_zone_state` (new hook: scan static abilities, register their effects) |
| Resolving spell ("gets +2/+2 UEOT") | `resolve.rs` `ModifyPowerToughness` primitive | `engine/resolve.rs` |
| Resolving activated ability with continuous effect | Same as above | Same |
| Commander in command zone (emblem-like) | When put into command zone | `engine/zones.rs::move_object` (when destination is command zone + source is commander) |
| Emblem | Emblem creation | Future Phase 7 hook |
| Alchemy perpetual mod | When applied | Phase 9 |

### Where effects get removed

| Trigger | Who calls `deregister_effect` |
|---|---|
| Source leaves battlefield | `engine/zones.rs::cleanup_zone_state` (new hook) |
| Cleanup step (UntilEndOfTurn) | `engine/turns.rs::cleanup_step` |
| Start of controller's untap (UntilYourNextTurn) | `engine/turns.rs::begin_untap` |
| End of combat (UntilEndOfCombat) | `engine/combat/steps.rs::end_of_combat` |
| Condition predicate goes false (WhileCondition) | SBA loop checks these and removes |
| Target leaves battlefield (UntilTargetLeavesBattlefield) | `engine/zones.rs::cleanup_zone_state` on target |

**Invariant:** `ContinuousEffectRegistry` should never contain effects whose source has left its functional zone. The `remove_by_source` helper runs on every zone change out of battlefield.

---

## 11. Engine Interaction Points

### 11.1 `oracle/characteristics.rs`

This is the single read-side chokepoint. All wrappers route through `compute_characteristics`. Phase LA rewires the existing wrappers (`has_keyword`, `is_creature`, `get_effective_name`, `get_effective_power`, `get_effective_toughness`). New wrappers added as needed:

- `get_effective_colors(game, id) -> HashSet<Color>`
- `get_effective_types(game, id) -> HashSet<CardType>`
- `get_effective_controller(game, id) -> PlayerId`
- `get_effective_abilities(game, id) -> Vec<AbilityDef>`

### 11.2 Cast pipeline

Cost modification (CR 601.2f) isn't part of the layer system, but some cost-modifying effects are continuous effects (e.g., "spells cost 1 more to cast"). Phase 6 (replacement effects) covers this; `CostRestriction` registry exists already. Layers don't touch it.

**Exception:** effects that change whether something is a creature (Layer 4) affect what can be targeted by a cast spell. The cast pipeline already calls `is_creature` → wrappers → layers. Works transparently after Phase LA.

### 11.3 SBA

SBA reads effective characteristics via `oracle/characteristics.rs`. Zero changes to SBA code for Phase LA through LD beyond the wrapper rewiring. `WhileCondition` duration predicate evaluation happens in a dedicated SBA pass added in Phase LC (adjacent to 704.5 checks).

### 11.4 Combat

Combat uses `get_effective_power` / `get_effective_toughness` for damage assignment. After Phase LB (Layer 7c migration), anthem effects Just Work in combat.

### 11.5 Targeting / Legality

Targeting legality (`oracle/legality.rs`) reads effective characteristics via wrappers. Works transparently.

---

## 12. Memoization / Performance

**Phase LA through LD ship with no cache.** Each `compute_characteristics` call re-walks the registry.

Worst-case cost: `O(effects × objects × layers)` per frame if every effect's predicate inspects every object. In practice effects are few and filters are cheap. No evidence we need a cache yet.

If profiling later shows this is hot:

1. **Per-frame cache** (inside a single `compute_characteristics` call) — already required for acyclicity (§5.2). Implicit.
2. **Registry-invalidated cache on `GameState`** — keyed by `(object_id, registry_version)`, cleared when the registry or any battlefield state changes. Adds a dirty-flag bit.
3. **Dirty-object tracking** — only recompute for objects whose inputs changed. Complex; defer.

This is the explicit correctness-first stance discussed in design review.

---

## 13. Work-Phase Plan

Each phase is a single bounded deliverable. Tests green at the end of each phase.

### Phase LA — Scaffolding (no behavior change)

**Scope:**

1. Create module tree (§4).
2. Define all types from §3 with full variant enumeration. Bodies on helper functions may be `todo!()` for future-phase concerns.
3. Add `ContinuousEffectRegistry` + `next_timestamp` to `GameState`. Constructor defaults.
4. Implement `compute_characteristics` such that output is identical to current `oracle/characteristics.rs` behavior:
   - Read base from `CardData`.
   - Apply `BattlefieldEntity.power_modifier` / `toughness_modifier` shim as Layer 7c synthesized effects (the pipeline's one 7c source for now).
   - Synthesize counter-derived 7c effects from `BattlefieldEntity.counters`, ordered after non-counter 7c per CR 613.4c.
   - Zone-scope fast-path for hidden zones (§5.1).
5. Rewire existing `oracle/characteristics.rs` wrappers to call `compute_characteristics`.
6. **Direct-`card_data` read audit** (deferred-migration item): grep for `obj.card_data.{keywords,colors,types,subtypes,power,toughness,name}` outside `oracle/characteristics.rs` and `engine/cast.rs` (cast-zone legality is pre-stack). Migrate each direct read to a wrapper call OR document why the direct read is correct. Expected output: a list commit + migration edits.
7. Modify `AbilityDef` to carry a `Keyword(KeywordAbility)` variant; `CardData.keywords` is retained as a write-time convenience but readers go through `EffectiveCharacteristics.abilities`. Add `is_characteristic_defining: bool` and `origin: AbilityOrigin` fields to `AbilityDef` (defaults: false, `PrintedRulesText`). No non-default producers yet.
8. Stub `ExpirationPredicate` AST (§15.1) with a minimal leaf set. No evaluator yet — just the types so `Duration::WhileCondition` compiles.

**Exit criteria:**

- All 433 existing tests pass unchanged.
- New unit tests in `compute.rs` verifying identity behavior on base + shim-7c + counters-7c paths + zone-scope fast-path.
- Grep audit documented in the PR.

**Estimated size:** 700–1000 lines added; ~30–50 lines migrated in the audit pass.

### Phase LB — Layer 7 real

**Scope:**

1. Replace the `power_modifier` / `toughness_modifier` shim with real Layer 7c `ContinuousEffect`s:
   - Pump spells (`ModifyPowerToughness` primitive) register effects with `Duration::UntilEndOfTurn`.
   - Static anthems on the battlefield (e.g., Glorious Anthem) register `Duration::WhileSourceActive` effects with `AffectedSet::Filter(...)`.
2. Delete the shim fields from `BattlefieldEntity`.
3. Implement Layers 7b (set P/T) and 7d (switch P/T). 7a (CDA P/T) scaffolded but no consumers yet.
4. Cleanup-step effect deregistration wired.
5. Intra-7c ordering verified: non-counter modifiers before counter-derived modifiers (CR 613.4c).

**Atomic-test coverage target:** ATOM-613.4c-001/002, ATOM-613.4b-001, ATOM-613.7-001 (timestamp within a layer), ATOM-122.1a-001 (counters + pump interaction).

**Exit criteria:**

- Pump spells go through the registry (no direct field mutation in tests).
- First anthem-family card unlocked (e.g., Glorious Anthem).
- Timestamp ordering verified in a test (two pumps, later wins on overlap).
- Counter-vs-pump ordering verified (a creature with +1/+1 counter + "gets -X/-X" checks correct final P/T).

### Phase LC — Layers 2, 5, 6 + dependency algorithm

**Scope:**

1. Implement Layer 2 (Control). `SetController` effect. Updates `BattlefieldEntity.controller` via compute + a sync step (or reads through compute directly — decide in PR).
2. Implement Layer 5 (Color). `AddColor`, `SetColors`, `RemoveAllColors`.
3. Implement Layer 6 (Ability add/remove). `GrantKeyword` / `GrantAbility` / `LoseAllAbilities`.
4. Implement the hybrid dependency algorithm (§9). Standalone `dependency.rs` module with unit tests on synthetic effects.
5. Wire `WhileCondition` duration evaluation into SBA pass.

**Atomic-test coverage target:** ATOM-613.1f, ATOM-613.3, ATOM-613.5, ATOM-613.6, ATOM-613.8-001 through 613.8c.

### Phase LD — Layers 1, 3, 4

**Scope:**

1. Layer 4 (Type-changing). Includes CR 205.1a/b subtype replacement semantics and **CR 305.7 (Blood Moon)** special-case handling in `apply()` (§3.5a). Uses `AbilityOrigin` field added in Phase LA.
2. Layer 3 (Text-changing). Rare in practice; e.g., [[Artificial Evolution]].
3. Layer 1a (Face-down). Morph, manifest. Uses the synthesized-from-flag approach (§7).
4. Layer 1b (Copy). Clone, Phantasmal Image, etc. Resolution produces a `CopyFrom { copiable: CopiableValues }` Layer 1b effect. Concrete `CopiableValues` struct defined at kickoff.

**Atomic-test coverage target:** 613.2 series, 613.3 series, CR 707/708 face-down interactions, CR 305.7 Blood-Moon-style tests.

### Post-Phase LD

- Add specific cards per `cards-unlocked-ledger.md`.
- Post-mortem: did the architecture hold? File-by-file review of what we'd change with hindsight. Document in `codebase-state.md`.

---

## 14. Testing Strategy

Per phase:

- **Unit tests** in `compute.rs` and `dependency.rs` for algorithm correctness on synthetic effects (no real cards).
- **Integration tests** in `tests/phaseL{A,B,C,D}_integration_test.rs` for real-card scenarios grounded in the atomic-test index.
- **Regression suite** — every phase must pass all prior tests unchanged. Especially load-bearing: phase LA must pass all existing 433 tests; phase LB must not regress pump-spell or counter tests.

Not in scope here (too detailed): the exact test list. Driven by the atomic-test phase index at implementation time.

---

## 15. Resolved & Open Decisions

### 15.1 Resolved during review

**→ Ability-granted effects on tokens (resolved).** If the token-creating effect's wording says the token "has [ability]", the ability is inherent to the token and baked into its `CardData` at creation. If the effect is "creature tokens you control get +2/+2" (a separate ongoing effect), that's a Layer 6 or 7c continuous effect registered on the source. No architecture change needed; card implementations follow this rule.

**→ Emblem / commander-source effects (resolved).** Continuous effects whose source is a commander are *removed* when the commander leaves its functional zone (including to the command zone); the `remove_by_source` helper handles this. Exceptions: **Eminence abilities** are static abilities functioning in the command zone, so they register with `source = commander, zone_of_function = {Command, Battlefield}`; they *re-register* automatically on each zone entry. **Alchemy perpetual mods** are a separate system that edits `CardData` directly and is orthogonal to the continuous-effect registry (see `alchemy-mechanics-audit.md`). Emblems themselves live in the command zone and their effects register with `source = emblem_id, zone_of_function = Command` — removed only when the emblem itself is removed (rare).

**→ APNAP tie-breaking on simultaneous timestamps (resolved).** Assigned at timestamp creation: when a batch of effects enters together, the active player's get strictly smaller timestamps. No runtime tie-breaking. See §8.

**→ `WhileCondition` predicate AST (tentative — see risk below).** Unified with the existing intervening-if / conditional-effect AST, not a parallel system. The `Effect::Conditional(Condition, Box<Effect>)` variant in `types/effects.rs` already has a `Condition` type for "if [condition]" text; that type grows into the shared predicate AST used for both intervening-ifs and duration predicates. Unifying avoids two vocabularies drifting apart and means new leaves added for one use case benefit the other.

Shape sketch:

```rust
/// Reused for intervening-ifs, expiration predicates, and anywhere
/// else static game-state predicates are needed.
pub enum Condition {
    // Object predicates (accept an ObjectRef so "this", "equipped creature",
    // and concrete ids all go through the same leaf).
    ObjectInZone { object: ObjectRef, zone: Zone },
    ObjectHasType(ObjectRef, CardType),
    ObjectHasSubtype(ObjectRef, Subtype),
    ObjectIsTapped(ObjectRef),
    ObjectIsAttacking(ObjectRef),
    ObjectIsBlocking(ObjectRef),
    ObjectHasCounterCount { object: ObjectRef, counter: CounterType, cmp: Ordering, n: u32 },
    // Player predicates.
    PlayerControls { player: PlayerRef, filter: SelectionFilter },
    PlayerLifeCompare { player: PlayerRef, cmp: Ordering, n: i32 },
    // Zone predicates.
    ZoneContainsCard { owner: PlayerRef, zone: Zone, filter: SelectionFilter },
    // Combinators.
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Not(Box<Condition>),
}

pub enum ObjectRef { Source, AttachedTo, Fixed(ObjectId) }

pub type ExpirationPredicate = Condition;   // alias — same AST, different use.
```

Samples from the reviewer's Scryfall query (~420 relevant cards after the given filters):
- *"has vigilance as long as there's a Lesson card in your graveyard"* → `ZoneContainsCard { Graveyard, Subtype(Lesson), ... }`
- *"As long as equipped creature is attacking"* → `ObjectIsAttacking(AttachedTo)`
- *"As long as this has 8+ +1/+1 counters"* → `ObjectHasCounterCount { Source, PlusOne, Ge, 8 }`

**Risk — the AST could spaghettify.** The reviewer flagged (correctly) that 1040 "as long as" vintage-legal cards is a lot of surface, weird leaves land every set, and refactoring an AST once cards depend on it is painful. Mitigations:

1. **Don't claim closure.** The AST is open: we add leaves as cards demand them. Leaves are *data*, not trait dispatch, so adding one is a variant + evaluator-match arm, not a refactor.
2. **Shared with intervening-ifs.** Amortizes leaf additions across two systems. Each new leaf is justified by at least one card in each or in both.
3. **Escape hatch.** If a card's condition resists decomposition (weird verbs like "as long as you control your opponents while they're searching their libraries"), a `Custom(CardId)` leaf or `OpaquePredicate(Arc<dyn Fn(&GameState) -> bool>)` leaf is acceptable. It sacrifices introspectability (dependency algorithm can't analyze it) but is always correct. Use sparingly; each use is a tech-debt marker.
4. **Deferred decision point.** If by Phase LC the AST growth rate suggests it *is* ossifying, we revisit with a different model (e.g., a tiny embedded script language or a trait-based predicate). Phase LC gate: count leaf variants needed for the atomic-test phase-index cards; if it exceeds ~25 or the list looks uncategorizable, escalate.

Phase LA ships a **minimum** AST (3–4 leaves) to unblock the type surface. No evaluator yet. Phase LC builds the evaluator and adds leaves for that phase's cards only.

### 15.2 Still open

1. **`compute_characteristics` caching at call-site.** No caching for Phase LA. Revisit after profiling Phase LC when the dependency algorithm runs. Frame cache (§5.2) is required for correctness and is the only cache.

2. **Copy-effect snapshot shape (Layer 1b).** `CopiableValues` is referenced as a type in §3.5 but not defined. Needs a concrete struct: likely a trimmed `CardData` containing only copiable characteristics (name, mana cost, color indicator, card types, subtypes, supertypes, rules text, power, toughness, loyalty, defense — per CR 707.2). Resolve at Phase LD kickoff.

3. **Dependency hypothetical-check snapshot performance.** Clone vs. CoW overlay for step-4 frame snapshots. Resolve in Phase LC.

4. **`AbilityOrigin` enum variants.** Needed for CR 305.7. Leaves needed: `PrintedRulesText`, `IntrinsicLandType(Subtype)`, `LayerGranted(EffectId)`. Defer exact shape to Phase LD kickoff, but the variant set above is close to final.

---

## 16. Explicitly Deferred

These are CR 613 features we will *not* implement in Phase LA–LD. Each is acceptable-for-now debt:

- **Dungeon / Venture** (CR 309, 701.52) — interacts with emblem-like command-zone tracking but is a keyword-action concern, not a layers concern.
- **Day/Night designation** (CR 726) — similar; not a layers concern even though it influences characteristics.
- **Energy counters, mana symbols in text** — Layer 3 (text) can theoretically touch these; practical coverage deferred to when a card needs it.
- **Dependency cycles among 3+ effects.** CR 613.8b says cycles → timestamp order. Simple to implement; we do it in Phase LC.
- **Perpetual modifiers (Alchemy).** Distinct system from continuous effects — perpetual mods edit the `CardData` of the object directly. See `alchemy-mechanics-audit.md`. Layer system cooperates with it via the usual wrapper reads.

---

## 17. Changes to `codebase-state.md` After Each Phase

Each phase's PR must update:

- **Phase LA** — remove "Before Layers" item 1 (pre-layer P/T shim) from Deferred Migrations → promote to "Layer 7 shim scheduled for removal in Phase LB". Remove Before-Layers item 2 (direct `CardData` reads) if audit clears all sites. Update CR 613 rows in the "Chapter-by-chapter map".
- **Phase LB** — mark CR 613.4b/c/d as ✅. Remove any lingering shim mentions. Update "What's missing in the layer system" TL;DR bullet.
- **Phase LC** — mark CR 613.8, 613.3 (layer 2), 613.5, 613.6, 613.1f as ✅.
- **Phase LD** — mark CR 613.2 (all), 613.3 (layer 3), 613.4 (all sublayers now including 7a CDA), CR 305.7 handling as ✅. Update the "Not started" TL;DR entry for Layers.

---

## 18. Pointers

Essential reading before Phase LA starts:

- CR 613 full text.
- `mtgsim/src/oracle/characteristics.rs` (current single-point wrapper module; receive rewiring).
- `mtgsim/src/state/battlefield.rs:12-100` (existing `BattlefieldEntity` with `timestamp`, `power_modifier`, `toughness_modifier`, `counters`).
- `mtgsim/src/types/effects.rs:290-322` (layer-annotated `Primitive` variants already defined).
- `design_doc.md:636-664` (hybrid dependency algorithm — adopted verbatim in §9).
- `atomic-tests/phase-index-phase-5-layers.md` (test scope grounding).
- `atomic-tests/pass0-dependency-map.md §8` (cross-cutting architecture decisions).

Done.
