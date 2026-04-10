# Alchemy Mechanics — Architectural Impact Audit

> Produced against: `implementation-plan-final.md`, `roadmap.md`, `pass0-dependency-map.md`, engine source (Phase 4.5 codebase), and the 12 Alchemy-Derived-Rules files.

---

## Per-Mechanic Audit

### 1. Perpetually
- **Architectural Impact:** High
- **Affected Subsystems:** `GameObject` (new field), `compute_characteristics` (new application step), `move_object` / zone transitions (must NOT strip perpetual data), delta log (perpetual application = game-state mutation)
- **Key Design Question(s):**
  - The `PerpetualMod` enum (D20b) is already designed but needs additional variants for Alchemy mechanics beyond the base set/modify/ability operations. See Cross-Cutting Q1.
  - Should `PerpetualMod` application emit deltas for the trigger scanner? E.g., "perpetually gains flying" on a creature already on the battlefield should be observable by state-based triggered abilities that watch for keyword acquisition.
- **Recommendation:** D20b's design is sound. The `Vec<PerpetualMod>` on `GameObject` applied before the layer loop is correct. No Phase 5 scaffolding needed beyond the TODO comments already planned for L04. Extend `PerpetualMod` variants when implementing individual Alchemy mechanics in Phase 9.

---

### 2. Conjure
- **Architectural Impact:** High
- **Affected Subsystems:** `CardRegistry` (must support runtime card instantiation by name), `GameObject` creation pipeline (new creation path outside of decklist), `GameState` object store (must accept non-decklist objects), zone transitions (conjured cards go to specific zones — hand, library, graveyard, battlefield)
- **Key Design Question(s):**
  - The engine currently assumes all `GameObject`s originate from decklists (built at game start). Conjure creates cards from nothing at runtime. This requires a `CardRegistry` (or equivalent) accessible at runtime, not just during setup. See Cross-Cutting Q2.
  - Conjured cards are "real cards" — not tokens, not copies. They have their own `ObjectId` and behave normally in all zones. The `GameObject` struct handles this fine; the gap is the *creation pathway*.
  - Conjure to library requires inserting at a random position (or top/bottom per card text). Current `Vec<ObjectId>` library supports this.
- **Recommendation:** The `CardRegistry` already exists (`cards/registry.rs`) and supports `create(name) -> Arc<CardData>`. It needs to be available at resolution time (currently only used during game setup). Pass `&CardRegistry` into the resolution context or store a reference on `GameState`. This is a **medium-effort scaffold** that should be noted for Phase 9 but doesn't need Phase 5 changes. No new fields on core structs.

---

### 3. Seek
- **Architectural Impact:** Low
- **Affected Subsystems:** `DecisionProvider` (new method or engine-internal random selection), library zone access (filter + random pick), zone transitions (library → hand without shuffle)
- **Key Design Question(s):**
  - Seek is engine-controlled randomness, not a player choice — the player doesn't see rejected candidates. This means it should NOT go through `DecisionProvider`. It's an engine action using `rand`.
  - "Seek a [criterion] card" requires filtering the library by card characteristics. The oracle module can provide this, but filtering needs to work on objects in the library (currently oracle functions are battlefield-focused).
- **Recommendation:** Fits cleanly into existing infrastructure. Implement as a new `Primitive::Seek` variant in `types/effects.rs` + resolution logic. No architectural changes needed. The only new thing is accessing `oracle::characteristics` queries for non-battlefield objects, which already works (the functions take `&GameState` + `ObjectId` and read `card_data`).

---

### 4. Draft (from Spellbook / Draft Pool)
- **Architectural Impact:** Medium
- **Affected Subsystems:** `DecisionProvider` (new `choose_from_draft` method — present N options, player picks 1), draft pool storage (see Q3), `CardRegistry` (runtime card creation), `GameState` (conjured card creation)
- **Key Design Question(s):**
  - Draft = random sample N from a pool + player choice + conjure the chosen card. Combines pool lookup, randomness, player decision, and card creation.
  - The player choice (pick 1 of N) is a genuine decision that must go through `DecisionProvider`. New method: `choose_draft_pick(game, player_id, options: &[Arc<CardData>]) -> usize`.
  - **The draft primitive must be general enough to handle both small spellbook picks (3 from 15) and large cube/set pool picks (15 from 540+).** This is the Booster Tutor pattern — see Arena designer note below.
- **Booster Tutor pattern:** The card "Booster Tutor" ({B}, Instant — originally Un-set, now in Powered Cube on Arena) was implemented on Arena by treating the entire cube list as a draft pool. Arena's approach: at match creation, the cube list is pre-shuffled in collated order. Each cast of Booster Tutor pulls the next 15 cards off that pre-shuffled list (no repeats across casts). This is a generalization of the spellbook mechanic: instead of "3 random from 15", it's "15 sequential from a pre-shuffled 540+".
- **Note on flexibility:** Arena's approach is a useful reference, but we're not married to it. Arena had specific production constraints (bug-prone spellbook updates, right-click information leaks, artist/UX team integration) that drove their design. As a Rust hobby project without deadlines, we can explore alternatives — e.g., true random sampling from the full pool on each cast (simpler, no cursor state), or more sophisticated collation, or even letting the game setup provide a custom pool generation strategy. The generalized draft pool design below supports multiple approaches; the `DraftPool` struct is one option, not the only option.
- **Recommendation:** The engine needs a **generalized draft pool primitive** that supports:
  1. **Small inline pools** (spellbook-style): sample K cards randomly from a list of N card names, present to player, conjure the pick. Stateless — each draft is independent.
  2. **Large pre-shuffled pools** (cube/set-style): at game start, shuffle a large card list into a `DraftPool` with a cursor. Each draft pulls the next K cards sequentially (no repeats). Stateful — stored on `GameState`.
  
  Both funnel into the same DP method: `choose_draft_pick(game, player_id, options: &[Arc<CardData>]) -> usize`. The difference is how `options` are generated (random sample vs sequential pull). See Q3 for storage design.

---

### 5. Spellbook (small draft pool)
- **Architectural Impact:** Low (subsumed by the generalized draft pool design)
- **Affected Subsystems:** `CardData` (inline card name list), `CardRegistry` (must know all spellbook card names for runtime instantiation)
- **Key Design Question(s):**
  - Spellbooks are the small-pool case of the draft primitive described in Section 4. Where does spellbook data live? See Cross-Cutting Q3.
  - All cards named in spellbooks must be registered in `CardRegistry` for conjure/draft to work. This expands the required card pool significantly — a card's spellbook can reference up to 15 cards that aren't in any player's deck.
- **Recommendation:** Add `pub spellbook: Option<Vec<String>>` to `CardData` — an inline list of card names (up to 15). At resolution time, the draft primitive samples 3 randomly from this list, presents to player via `choose_draft_pick`, and conjures the chosen card. All spellbook cards must be pre-registered in `CardRegistry`. This is a data-loading concern, not an engine architecture concern. No Phase 5 changes needed. The larger "Booster Tutor" pool case uses a separate `DraftPool` on `GameState` (see Q3).

---

### 6. Boon
- **Architectural Impact:** Medium
- **Affected Subsystems:** `GameState` (new boon storage — similar to emblems), triggered ability system (Phase 7 — boons ARE triggered abilities), delta log (boon triggers must be scannable)
- **Key Design Question(s):**
  - Boons are sourceless triggered abilities with optional use counters. How do they interact with the Phase 7 trigger system? See Cross-Cutting Q4.
  - Boons need storage: they persist like emblems, are owned by a player, and have trigger conditions + effects + optional use counts.
  - Can boons reuse emblem infrastructure? Emblems are planned as command-zone objects with static abilities. Boons are similar but have triggered abilities and use counters.
- **Recommendation:** Boons should be `GameObject`s in the command zone with a `BoonState` sidecar (analogous to `BattlefieldEntity` for permanents). The sidecar holds `uses_remaining: Option<u32>` (None = unlimited). Their triggered abilities participate in the normal trigger scanner (Phase 7). This requires: (1) a `boons: HashMap<ObjectId, BoonState>` on `GameState` (or reuse the command zone), (2) the trigger scanner to check command-zone objects, not just battlefield. The trigger scanner design (delta log) naturally supports this — it scans registered trigger patterns against deltas, regardless of the trigger source's zone. **Scaffold note:** When implementing Phase 7's trigger scanner, ensure it doesn't hardcode battlefield-only sources. This is a design constraint to document now, not code to write now.

---

### 7. Intensity
- **Architectural Impact:** Medium
- **Affected Subsystems:** `GameObject` (tracked via `PerpetualMod` or dedicated field), `compute_characteristics` (intensity must be readable at resolution time), effect resolution (abilities reference intensity value)
- **Key Design Question(s):**
  - Is intensity a special case of `PerpetualMod`, or its own field? See Cross-Cutting Q5.
  - Intensity is *read* by abilities at resolution time ("deals damage equal to its intensity"), not just applied during `compute_characteristics`. This means it can't be purely a layer-system concept — it needs to be queryable as a raw value.
  - "Starting Intensity N" is a card property. "Intensify" increments it. Both persist across zones.
- **Recommendation:** Add a dedicated `pub intensity: Option<u32>` field on `GameObject` (not a `PerpetualMod` variant). Rationale: intensity is read as a *value* by effects (e.g., `AmountExpr::Intensity`), not just applied as a characteristic override. Making it a separate field gives clean query access (`obj.intensity.unwrap_or(0)`) without scanning the `PerpetualMod` vec. Initialize from `CardData.starting_intensity: Option<u32>` at object creation. This is a small addition to `GameObject` and `CardData`. No Phase 5 changes needed.

---

### 8. Specialize
- **Architectural Impact:** High
- **Affected Subsystems:** `GameObject` (perpetual transformation to a different card identity), `CardData` / `CardRegistry` (specialized variants are distinct cards), `PerpetualMod` (new variant or specialized mechanism), `compute_characteristics` (specialized card replaces base characteristics)
- **Key Design Question(s):**
  - How does specialize interact with the Face abstraction planned for DFCs (Phase 9)? See Cross-Cutting Q6.
  - Specialize permanently replaces the card's identity (name, mana cost, P/T, abilities) with a color-variant. This is *not* like Prototype (which swaps a subset of characteristics) — it's a full card replacement that persists across zones.
  - "Unspecialize" exists — some cards can revert.
- **Recommendation:** Specialize is closest to a perpetual full-card replacement. Two approaches:
  1. **`PerpetualMod::Specialize(Arc<CardData>)`** — a perpetual mod that replaces the entire `card_data` base. `compute_characteristics` checks for this variant first and uses the specialized `CardData` instead of `obj.card_data`. Unspecialize = remove this mod. Simple, clean, fits the existing `PerpetualMod` framework.
  2. **Face system (DFC-style)** — treat specialized variants as alternate faces. This requires the Face abstraction (D3, Phase 9). If DFC infrastructure exists, specialize becomes "perpetually set active face to variant X."

  **Recommended:** Option 1 for initial implementation (it's self-contained and doesn't depend on DFC infrastructure). When DFCs land, evaluate whether to migrate specialize to the face system. Add `PerpetualMod::ReplaceCardData(Arc<CardData>)` variant. The specialized variant `CardData`s must be in `CardRegistry`.

---

### 9. Double Team
- **Architectural Impact:** Low
- **Affected Subsystems:** Triggered ability system (Phase 7 — triggers on attack), conjure pipeline (creates a duplicate), `PerpetualMod` (both original and duplicate perpetually lose Double Team)
- **Key Design Question(s):**
  - "Conjure a duplicate" is a conjure that creates a card with the same `card_data` as the original — but explicitly does NOT carry over perpetual modifications. This means conjure must use the *base* `card_data`, not snapshot the current object.
  - Both original and duplicate "perpetually lose double team" — this is `PerpetualMod::RemoveAbility(KeywordAbility::DoubleTeam)` applied to both.
- **Recommendation:** Fits cleanly into planned infrastructure. Triggered ability (Phase 7) + conjure + perpetual mod. No new architectural patterns. `KeywordAbility::DoubleTeam` variant needed. The "conjure duplicate" action should create a new `GameObject` from `obj.card_data` (the `Arc<CardData>`, ignoring perpetual mods on the source), which is the natural behavior.

---

### 10. Heist
- **Architectural Impact:** Medium
- **Affected Subsystems:** Library access (look at 3 random nonland cards from opponent), exile zone metadata (face-down exile, "exiled by" tracking — D21), casting permission system (`CastPermission` — cast from exile, spend mana as any type), `DecisionProvider` (choose 1 of 3)
- **Key Design Question(s):**
  - Heist requires exile zone metadata: the exiled card is face-down, and the heisting player (not the owner) can cast it. This is exactly D21 (exile zone metadata — face-down, exiled-by), already a deferred item.
  - "Spend mana as though it were mana of any type" is a mana spending permission that modifies cost payment. This interacts with T12's mana spending restrictions design (Phase 5-Pre).
  - Casting from exile requires `CastPermission` (documented in `cast.rs` TODO).
- **Recommendation:** Heist depends on three deferred systems: D21 (exile metadata), `CastPermission` (zone-casting), and mana spending permissions. All three are already identified as future work. No new architectural patterns beyond what's planned. When implementing D21, ensure the exile metadata supports "this player may cast this card" and "spend mana as any type" flags.

---

### 11. Incorporate
- **Architectural Impact:** Low
- **Affected Subsystems:** `PerpetualMod` (new variants for ability granting and color addition), `compute_characteristics` (incorporated colors become part of card's colors), cost modification pipeline (additional cost via granted static ability)
- **Key Design Question(s):**
  - Incorporate perpetually grants abilities, changes colors, and adds a mandatory additional casting cost. But **how** the cost addition works matters — see the two perpetual cost patterns below.
  - "Mana value does not change" despite the additional cost.
- **Two distinct perpetual cost-change patterns in Alchemy:**

  | Pattern | Template | Example | Mechanism |
  |---------|----------|---------|-----------|
  | **Direct mana cost replacement** | "its mana cost perpetually becomes {X}" | Thought Partition: "its mana cost perpetually becomes {5}" | `PerpetualMod::SetManaCost(ManaCost)` — directly overwrites the card's mana cost characteristic. Mana value DOES change. |
  | **Granted cost-modifying ability** | "perpetually gains 'This spell costs {X} more/less to cast'" | Nightclub Bouncer: "It perpetually gains 'This spell costs {2} more to cast.'" | `PerpetualMod::AddAbility(AbilityDef)` — grants a static ability that feeds into the **cost modification pipeline** (T18 step 601.2e). Mana value does NOT change. The card's printed mana cost is unchanged; the extra cost is applied during casting. |

  **Incorporate uses the second pattern** — it perpetually grants a "costs {X} more" static ability (plus other abilities and colors). The additional cost is NOT a modification to the mana cost characteristic; it's a granted ability that the cost modification pipeline reads at cast time.

  This distinction is important: `SetManaCost` changes what `mana_value()` returns. A granted "costs more" ability does not. They are architecturally different paths — one modifies `compute_characteristics` output, the other feeds the cost pipeline.

- **Recommendation:** Decompose incorporate into individual `PerpetualMod` entries:
  - `AddAbility(AbilityDef)` — for the "costs {X} more" static ability AND any other granted abilities
  - `AddColor(Color)` — for the color addition
  
  No new `AddCastingCost` variant needed. The additional cost is expressed as a granted static ability, which is how Arena templates it. The cost modification pipeline (T18, rule 601.2e: base cost → increases → reductions → Trinisphere floor) reads these granted abilities at cast time. `compute_characteristics` applies the color addition normally.

---

### 12. Starting Player
- **Architectural Impact:** None
- **Affected Subsystems:** `GameState` (already tracks `active_player` and turn order; needs a `starting_player: PlayerId` field or derive from turn 1 active player)
- **Key Design Question(s):** None significant. "If you're not the starting player" is a simple boolean check against a game-level field.
- **Recommendation:** Add `pub starting_player: PlayerId` to `GameState` (set during game setup). Cards check `game.starting_player != controller`. Trivial. Can be added at any time with zero architectural impact.

---

## Cross-Cutting Questions

### Q1: Does the `PerpetualMod` enum need additional variants?

**Yes.** The D20b design lists these variants:

```
SetPower, SetToughness, ModifyPower, ModifyToughness,
SetColors, AddAbility, RemoveAbility, RemoveAllAbilities, SetManaValue
```

**Additional variants needed for Alchemy mechanics:**

| Variant | Required by | Notes |
|---------|------------|-------|
| `AddColor(Color)` | Incorporate | Adds a color without replacing existing colors |
| `ReplaceCardData(Arc<CardData>)` | Specialize | Full card identity replacement |
| `RemoveKeyword(KeywordAbility)` | Double Team ("perpetually lose double team") | Alias of existing `RemoveAbility` if abilities and keywords are unified; otherwise new variant |
| `AddKeyword(KeywordAbility)` | Various perpetual "gains [keyword]" | May overlap with `AddAbility` depending on keyword/ability unification |

**Already covered by the base D20b set (no new variants):**
- `SetManaCost` — covers Thought Partition's "mana cost perpetually becomes {5}" pattern (direct mana cost replacement)
- `AddAbility` — covers Nightclub Bouncer / Incorporate's "perpetually gains 'costs {2} more to cast'" pattern (granted static ability that feeds cost pipeline, does NOT change mana value)

**Not needed as PerpetualMod variants:**
- **Intensity** — separate `Option<u32>` field on `GameObject` (read as value, not characteristic override)
- **Double Team** — uses existing `RemoveAbility`/`RemoveKeyword` + conjure
- **Specialize "unspecialize"** — remove the `ReplaceCardData` mod from the vec
- **~~AddCastingCost~~** — incorporate's additional cost is a perpetually granted static ability (`AddAbility`), not a direct cost modification. See Section 11 for the two distinct perpetual cost-change patterns.

**Summary:** ~3-4 new variants beyond the base set. The `PerpetualMod` enum is designed to be extended (`// Extend as Alchemy mechanics require`), so this is expected.

---

### Q2: Does the engine need a CardFactory/CardRegistry for runtime card creation?

**Yes, but it already exists.** `CardRegistry` in `cards/registry.rs` maps card names to factory functions (`fn() -> Arc<CardData>`). It supports `create(name) -> Result<Arc<CardData>>`.

**Gap:** `CardRegistry` is currently only used at game startup (deck building). Conjure/Draft/Heist need it at resolution time. Two options:

1. **Store `Arc<CardRegistry>` on `GameState`** — simple, direct access during resolution. Slightly increases `GameState` size but `CardRegistry` is immutable after construction.
2. **Pass `&CardRegistry` into resolution context** — cleaner separation but requires threading it through `resolve_effect`, `resolve_primitive`, and any action that might conjure.

**Recommendation:** Option 1 (store on `GameState`). The registry is read-only and lightweight. The runtime creation path is:

```rust
let card_data = game.card_registry.create("Goblin Token")?;
let obj = GameObject::new(card_data, owner, target_zone);
let id = game.add_object(obj);
game.add_to_zone_collection(id, target_zone)?;
```

This reuses the existing `GameObject` creation pipeline. No new subsystem needed — just making the existing registry accessible at resolution time.

**Performance analysis (thousands of cards):** The `CardRegistry` is a `HashMap<String, fn() -> Arc<CardData>>`. Each entry is a string key + a function pointer (8 bytes). For 10,000 cards, that's ~10K entries × ~(avg 20-byte string + 8-byte fn ptr + HashMap overhead) ≈ **~500 KB**. This is negligible. The registry is wrapped in `Arc`, so `GameState` only holds a single pointer (8 bytes) — no cloning cost. The `HashMap::get` lookup is O(1) amortized regardless of size. The factory functions are not called until a card is conjured, and each call produces an `Arc<CardData>` (one allocation). **No performance concern at any realistic registry size.**

The only scaling consideration is memory for `Arc<CardData>` instances *created* at runtime (conjured cards). Each conjured card creates one `Arc<CardData>` + one `GameObject`. In practice, a game might conjure tens of cards — trivial. If a hypothetical effect conjured thousands (not realistic in MTG), the bottleneck would be `GameState.objects` HashMap growth, not the registry.

**Important constraint:** All cards that can be conjured (spellbook cards, heist targets, double team duplicates) must be pre-registered in `CardRegistry`. For spellbook cards, this is deterministic (known at game start from decklists). For heist, the opponent's entire library is already registered. For double team, the conjured duplicate uses the same `Arc<CardData>` as the original.

---

### Q3: Where should draft pool / spellbook data live?

There are two cases to support, and they have different storage needs.

#### Case A: Small inline pools (spellbooks — up to 15 cards)

**On `CardData`, as an inline `Vec<String>`.**

```rust
// On CardData:
pub spellbook: Option<Vec<String>>,  // card names
```

- Spellbooks are per-card static data (printed on the card). `CardData` is the natural home.
- Up to 15 entries — trivial memory cost.
- At resolution time: `card_data.spellbook.as_ref()` → sample K randomly → look up each in `CardRegistry` → present to player via `choose_draft_pick`.
- Stateless: each draft from a spellbook is independent (random sample with replacement from the full list).

#### Case B: Large pre-shuffled pools (Booster Tutor / cube pools — 100s of cards)

**On `GameState`, as a `DraftPool` with a cursor.**

```rust
// On GameState:
pub draft_pools: HashMap<DraftPoolId, DraftPool>,

pub type DraftPoolId = u32;  // or String key like "cube"

pub struct DraftPool {
    /// Pre-shuffled card name list (collated at game start)
    cards: Vec<String>,
    /// Cursor: next index to pull from. Advances by K on each draft.
    cursor: usize,
}

impl DraftPool {
    /// Pull the next `count` cards from the pool. Returns fewer if pool is exhausted.
    pub fn pull(&mut self, count: usize) -> Vec<&str> {
        let end = (self.cursor + count).min(self.cards.len());
        let slice = &self.cards[self.cursor..end];
        self.cursor = end;
        slice.iter().map(|s| s.as_str()).collect()
    }
}
```

- Initialized at game/match start: the cube/set list is shuffled (with optional collation for color balance) into a `DraftPool`.
- Each cast of a "draft from pool" effect (e.g., Booster Tutor) calls `pool.pull(15)` to get the next 15 cards, looks them up in `CardRegistry`, presents to the player, and conjures the pick.
- **Stateful:** the cursor ensures no repeats across casts within the same game. If the pool is exhausted, the effect fails gracefully (present fewer options, or fizzle — TBD per card text).
- The pool is per-game, not per-card — a card like Booster Tutor references a pool ID, not an inline list.

#### How cards reference their pool

```rust
// On CardData (for spellbook cards):
pub spellbook: Option<Vec<String>>,

// On CardData (for large-pool cards like Booster Tutor):
pub draft_pool_id: Option<DraftPoolId>,
pub draft_pull_count: Option<usize>,  // how many to show (15 for Booster Tutor, 3 for typical draft)
```

At resolution time, the engine checks `card_data.draft_pool_id` first (large pool path), then falls back to `card_data.spellbook` (small pool path). Both produce a `Vec<Arc<CardData>>` of options that go to `choose_draft_pick`.

#### Collation

Arena's Booster Tutor implementation uses collated ordering (color-balanced packs, not pure random). This is a shuffling concern at pool creation time, not an engine concern. The `DraftPool` just stores the pre-shuffled list; the shuffling/collation algorithm lives in the game setup code. The engine doesn't need to know about collation — it just pulls sequentially.

---

### Q4: How do boons interact with the triggered ability system?

**Boons are emblems with use counters.** There's nothing a boon does that an emblem with a triggered ability and a `uses_remaining: Option<u32>` counter doesn't cover.

**Implementation:** Reuse emblem infrastructure (D7/Phase 9). Emblems are command-zone `GameObject`s with abilities. Extend the emblem sidecar with:

```rust
pub uses_remaining: Option<u32>,  // None = unlimited (normal emblem), Some(n) = boon with n uses
```

When a boon's triggered ability resolves, decrement the counter. If it hits 0, remove the emblem/boon from the command zone.

**Key constraint for Phase 7:** The trigger scanner must scan command-zone objects (emblems, boons), not just battlefield permanents. This is a **design constraint to document now** in the Phase 7 trigger scanner design. The delta log architecture naturally supports this (it matches patterns against deltas, source-zone-agnostic), but the trigger *registration* step must include command-zone objects.

---

### Q5: Is intensity a PerpetualMod or its own field?

**Its own field on `GameObject`.** Intensity is sufficiently different from "regular" perpetual effects — it's a mutable numeric value read by abilities, not an ordered operation log applied to characteristics.

```rust
// On GameObject:
pub intensity: Option<u32>,

// On CardData (printed starting value):
pub starting_intensity: Option<u32>,
```

**Rationale:**
- Intensity is **read as a value** by card abilities at resolution time: "deals damage equal to its intensity" maps to `AmountExpr::Intensity` in the effect system, which needs to read a raw `u32`, not scan a `PerpetualMod` vec.
- Intensity is **incremented** by "intensify" actions, which need a mutable numeric target — not a "append to log" operation.
- Intensity persists across zones (like perpetual mods), but it's a single numeric value, not an ordered operation log.
- Making it a `PerpetualMod` variant would require scanning the vec to find the current intensity value every time an ability references it — O(n) vs O(1).

**Interaction with PerpetualMod:** Intensity is orthogonal to `PerpetualMod`. A card can have both perpetual modifications AND intensity. They don't interact — intensity is read by effects, perpetual mods modify characteristics. `compute_characteristics` doesn't need to know about intensity (it's not a characteristic).

**New types needed:**
- `AmountExpr::Intensity` variant in `types/effects.rs`
- `Primitive::Intensify(AmountExpr)` variant for incrementing intensity
- `obj.intensity` field initialized from `card_data.starting_intensity` at `GameObject::new()`

---

### Q6: How does specialize interact with the Face abstraction (DFCs)?

**Specialize is NOT a DFC transform.** It's closer to a perpetual full-card replacement.

| Dimension | DFC Transform | Specialize |
|-----------|--------------|------------|
| **Persistence** | Zone-dependent (back face on battlefield only) | Perpetual (survives zone changes) |
| **Trigger** | "Transform" keyword action | Activated ability (pay cost + discard) |
| **Reversibility** | Transform back | "Unspecialize" (some cards only) |
| **Data source** | Back face on same CardData | Separate CardData per color variant |
| **Characteristics** | Entirely different card face | Entirely different card identity |
| **Layer interaction** | Active face determines base for layer loop | Specialized CardData determines base before layer loop |

**Specialize is architecturally closer to Prototype than to DFC:**
- Prototype: alternative base characteristics from `CardData.prototype_stats`, gated by `CastInfo.cast_as_prototype`
- Specialize: alternative base characteristics from a *different* `CardData`, gated by `PerpetualMod::ReplaceCardData`
- DFC: alternative face from `CardData.back_face`, gated by permanent status (transformed/not)

**Recommendation:** Implement specialize via `PerpetualMod::ReplaceCardData(Arc<CardData>)` independently of the DFC face system. When DFCs land (D3, Phase 9), evaluate whether specialize should migrate to the face abstraction. The perpetual approach is simpler, self-contained, and doesn't create a dependency on DFC infrastructure.

**If DFC faces DO handle specialize later:** The face system would need to support "perpetually active face" (not just "currently transformed"), which is a meaningful extension. The migration path would be: remove `ReplaceCardData` mod → set active face to specialized variant with perpetual duration.

---

### Q7: Are there Alchemy mechanics that would be painful to retrofit after Phases 5–8?

**Two items warrant attention now. The rest fit cleanly.**

#### 1. Phase 7 trigger scanner must not be battlefield-only (affects: many mechanics, not just Alchemy)

The trigger scanner (Phase 7) must scan triggers on objects in ALL zones, not just battlefield permanents. **This is not just an Alchemy concern** — plenty of paper MTG cards trigger from non-battlefield zones:

- **Graveyard:** Bloodghast ("Whenever a land enters the battlefield under your control..."), Narcomoeba ("When this card is put into your graveyard from your library...")
- **Command zone:** Emblems from paper planeswalkers have triggered abilities; boons (Alchemy) are triggered abilities with use counters
- **Exile:** Suspend triggers ("When the last time counter is removed..."), cards with "when this card is exiled" triggers
- **Hand:** Theoretically possible — an effect could trigger from hand (e.g., Madness-adjacent mechanics)

If the scanner hardcodes `game.battlefield.keys()` as its trigger source set, ALL of the above require refactoring — not just Alchemy boons.

**Action:** When designing the Phase 7 trigger scanner, document and implement it as scanning all objects with registered triggers, using an index like `HashMap<TriggerKind, Vec<ObjectId>>` that includes objects in any zone. This is architecturally trivial if done from the start, painful if the scanner is battlefield-scoped and must be widened later.

#### 2. CardRegistry must be available at resolution time (affects: Conjure, Draft, Heist, Double Team)

Four Alchemy mechanics create cards at runtime. The `CardRegistry` is currently game-setup-only. Making it available during resolution is a small change (`Arc<CardRegistry>` on `GameState` or in `ResolutionContext`), but if resolution is built without this, every conjure-like effect requires a workaround.

**Action:** When implementing Phase 5-Pre or Phase 5, add `pub card_registry: Arc<CardRegistry>` to `GameState` (or to `ResolutionContext`). This is a one-line field addition with no behavioral impact on existing code. It scaffolds the runtime card creation path for Phase 9.

#### Everything else fits:

- **Perpetual** — D20b is well-designed, `Vec<PerpetualMod>` on `GameObject` applied in `compute_characteristics`. No Phase 5 changes beyond TODO comments (already planned).
- **Seek** — Engine-internal randomness + library filter. No architectural impact.
- **Spellbook / Draft Pool** — Small pools on `CardData`, large pools via `DraftPool` on `GameState`. Self-contained.
- **Intensity** — New field on `GameObject` + `CardData`. Orthogonal to everything.
- **Specialize** — `PerpetualMod::ReplaceCardData`. Self-contained.
- **Double Team** — Triggered ability + conjure + perpetual mod. All planned.
- **Incorporate** — Perpetually granted abilities + color addition. Cost increase is a granted static ability feeding the cost pipeline, not a direct mana cost mod. Fits existing `AddAbility` + `AddColor` variants.
- **Starting Player** — Trivial boolean check. Zero impact.
- **Heist** — Depends on D21 (exile metadata) + CastPermission + mana spending permissions, all already deferred.

---

## Summary of Recommended Changes to Implementation Plan

### Immediate (add to existing tickets/docs)

1. **Add `starting_player: PlayerId` to `GameState`** — trivial, can go into any Phase 5-Pre ticket or standalone. Zero risk.

2. **Add `card_registry: Arc<CardRegistry>` to `GameState`** — add as a field in the `GameState::new()` constructor (or a `GameState::with_registry()` variant). One-line scaffold. Recommend adding in Phase 5-Pre alongside other `GameState` field additions. No performance concern even at 10K+ cards (see Q2).

3. **Document Phase 7 trigger scanner constraint** — in `roadmap.md` Phase 7 section, add a note: "Trigger scanner must support triggers on objects in any zone (not just battlefield). Required for: emblems, boons (Alchemy), command-zone triggered abilities."

### Phase 9 Alchemy Implementation Notes (add to roadmap)

4. **Expand `PerpetualMod` enum** — add ~3-4 new variants: `AddColor`, `ReplaceCardData`, and keyword-specific variants if keywords and abilities aren't unified by then. Note: `AddCastingCost` is NOT needed — incorporate's additional cost is a perpetually granted static ability (`AddAbility`), not a direct cost modification. The existing `SetManaCost` covers Thought Partition-style direct mana cost replacement. See Section 11 for the two distinct patterns.

5. **Add `intensity: Option<u32>` to `GameObject`** and `starting_intensity: Option<u32>` to `CardData`. Add `AmountExpr::Intensity` variant. Standalone field, not a `PerpetualMod`.

6. **Add `spellbook: Option<Vec<String>>` to `CardData`** — for small inline draft pools (up to 15 card names).

7. **Add `draft_pools: HashMap<DraftPoolId, DraftPool>` to `GameState`** — for large pre-shuffled draft pools (Booster Tutor / cube pools). `DraftPool` holds a pre-shuffled `Vec<String>` + cursor. Initialized at game/match start. See Q3 for full design.

8. **Extend emblem infrastructure with `uses_remaining: Option<u32>`** — this is all boons need. No separate `BoonState` required.

9. **New `DecisionProvider` methods** — `choose_draft_pick`, `choose_heist_card`, `choose_specialize_color` (3 new methods). These are Phase 9 additions to the trait.

### No Changes Needed

- **Phase 5 (Layers):** No changes. D20b TODO comments in L04 are sufficient.
- **Phase 6 (Replacement Effects):** No Alchemy-specific changes.
- **Phase 7 (Triggered Abilities):** Only the "scan all zones" constraint noted above.
- **Phase 8 (Keywords/Breadth):** No Alchemy-specific changes.

### Risk Assessment

| Item | Risk if not scaffolded now | Effort to scaffold now |
|------|--------------------------|----------------------|
| Trigger scanner zone-agnostic | Medium — refactor to widen scan scope | Zero — just a design constraint in docs |
| CardRegistry on GameState | Low — easy to add later, but every conjure workaround is ugly | 1 line of code + constructor update |
| starting_player field | None — trivial retrofit | 1 line of code |
| DraftPool on GameState | None — self-contained addition | Small struct + HashMap field |
| Everything else | None — fits planned infrastructure | N/A |

**Bottom line:** The engine architecture is well-positioned for Alchemy. The `Vec<PerpetualMod>` design (D20b) handles ~80% of Alchemy's complexity. The two scaffolding items (registry access + trigger scanner constraint) are minimal effort now and prevent the only two potential pain points. The generalized draft pool design (Q3) enables both standard spellbook effects and the Booster Tutor pattern with the same `choose_draft_pick` DP method.

---

## Appendix: GameObject Field Additions — Performance Analysis

### Current `GameObject` layout

```rust
pub struct GameObject {       // current
    pub id: ObjectId,         // Uuid = 16 bytes
    pub owner: PlayerId,      // usize = 8 bytes
    pub card_data: Arc<CardData>,  // 8 bytes (pointer)
    pub zone: Zone,           // enum, 1 byte + padding → 8 bytes (alignment)
}
// Total: ~40 bytes (with alignment padding)
```

### Planned additions (cumulative across all phases)

| Field | Type | Size | Added by | Notes |
|-------|------|------|----------|-------|
| `last_zone_change_epoch` | `u64` | 8 bytes | Phase 5-Pre (rule 400.7 epoch model) | Already designed |
| `perpetual_modifications` | `Vec<PerpetualMod>` | 24 bytes (empty Vec = ptr+len+cap) | Phase 9 (D20b) | 0 heap allocation if empty; most cards never get perpetual mods |
| `intensity` | `Option<u32>` | 8 bytes (4 + discriminant + padding) | Phase 9 (Alchemy) | `None` for ~99% of cards |
| `cast_as_prototype` | `bool` (via CastInfo) | 1 byte (in CastInfo sidecar) | Phase 9 (D20a) | NOT on GameObject — lives on StackEntry/sidecar |

**New total: ~80 bytes per `GameObject`** (up from ~40). This is the steady-state size including all planned additions.

### Is this a problem?

**No.** Here's why:

1. **Object count is small.** A typical MTG game has ~120-180 `GameObject`s (two 60-card decks + tokens/conjured cards). At 80 bytes each, that's **~14 KB** for the entire objects HashMap values. Negligible.

2. **The Vec is zero-cost when empty.** An empty `Vec<PerpetualMod>` is 24 bytes on the stack (pointer, length, capacity) with zero heap allocation. In a non-Alchemy game, every card has an empty vec — no heap overhead at all. Even in Alchemy, only the cards that have been perpetually modified allocate, and typical counts are 1-5 entries per affected card.

3. **Option<u32> is zero-cost for None.** The `None` variant is a single discriminant byte (+ alignment padding). No heap allocation. Only Alchemy intensity cards use `Some(n)`.

4. **Clone cost.** `GameObject` derives `Clone`. The current struct is trivially cheap to clone (Uuid copies, Arc bumps refcount). The additions:
   - `u64` — trivial copy
   - `Option<u32>` — trivial copy
   - `Vec<PerpetualMod>` — this is the only one that matters. Cloning a Vec allocates + copies. But: (a) empty Vecs clone to empty Vecs (no allocation), (b) typical perpetual mod vecs are 1-5 entries of small enums, (c) `GameState` cloning is already dominated by the `HashMap<ObjectId, GameObject>` iteration cost, not individual field costs.

5. **AI game-tree search.** The roadmap mentions copy-on-write for AI. The `Vec<PerpetualMod>` could use `Arc<Vec<PerpetualMod>>` (or `im::Vector`) if clone cost becomes measurable in profiling. This is a targeted optimization for later, not an architectural concern now.

### Comparison to BattlefieldEntity

For reference, `BattlefieldEntity` is already ~100+ bytes (ObjectId, PlayerId, u64 timestamp, 5 bools, u32 damage, bool deathtouch, 2× i32 modifiers, 2× Option<AttackingInfo/BlockingInfo>). `GameObject` at ~80 bytes is smaller, and there are fewer of them on average (not all objects are on the battlefield).

### Verdict

**No performance concern.** The planned additions add ~40 bytes per object with zero heap overhead for the common case (no perpetual mods, no intensity). The total memory for all `GameObject`s in a game is under 20 KB. Clone cost is dominated by HashMap iteration, not per-object field copying.
