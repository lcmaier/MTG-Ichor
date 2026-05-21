# Mana Spending Restrictions — Design Document

**Ticket:** T12 (source: E15)  
**Status:** Design spike — no production code  
**Rule reference:** CR 106.6, 106.7, 106.8, 601.2g–h

---

## 1. Problem Statement

Several cards produce mana that carries spending restrictions ("spend this mana only to cast creature spells"), grants ("that spell can't be countered"), or persistence ("mana doesn't empty as steps and phases end"). The current `ManaPool` is a flat `HashMap<ManaType, u64>` — it has no infrastructure for tracking per-unit metadata.

The codebase already has standalone types for this (`ManaAtom`, `ManaRestriction`, `ManaGrant`, `ManaPersistence` in `types/mana.rs` lines 224–303), but they are not wired into `ManaPool` or any engine pipeline.

---

## 2. Open Questions — Resolved

### Q1: Where does restriction metadata live?

**Answer: On `ManaPool` itself, as a sidecar `Vec<ManaAtom>`.**

Rationale:
- Restrictions are properties of *mana units*, not of the player. A player may have 3 unrestricted green mana and 1 creature-only green mana simultaneously. The metadata must travel with the mana.
- A parallel structure on `PlayerState` would duplicate the "how much mana is available" question across two locations. Every consumer (`can_pay`, `pay`, `total`, `empty`, `available`) would need to reconcile them.
- The existing `ManaAtom` struct already models this cleanly. The `ManaPool` gains a `special: Vec<ManaAtom>` sidecar. The existing `pool: HashMap<ManaType, u64>` remains as the fast path for the 99% case (unrestricted mana).

This matches the plan already outlined in the `ManaPool` doc comments (lines 213–222 of `types/mana.rs`).

### Q2: How do restrictions compose with Trinisphere-like effects?

**Answer: Restricted mana counts toward the total paid, including Trinisphere's minimum — but only if the restriction is satisfied.**

Detailed analysis:

- **Trinisphere** (rule 601.2e cost modification): "If the total cost to cast a spell would be less than {3}, it costs {3} instead." This is a cost *modification* — it changes the total cost *before* payment. It does not care what kind of mana pays for it.
- After cost modification, the player has a final `ManaCost` to pay. At payment time (601.2h), any mana in the pool that satisfies its own restriction AND matches the cost's color requirements is eligible.
- **Example:** Player casts a creature spell. Trinisphere makes the cost {3}. Player has 1 creature-only {G} from Cavern of Souls + 2 unrestricted {R}. The creature-only {G} is eligible because the spell *is* a creature. The player pays {G}{R}{R}. Trinisphere is satisfied.
- **Example:** Player casts an instant. Trinisphere makes the cost {3}. Player has 1 creature-only {G} + 2 unrestricted {R}. The creature-only {G} is *not* eligible because the spell is not a creature. The player must pay {R}{R} + something else. If they only have 2R total, they can't pay.
- **Conclusion:** The cost modification pipeline (601.2e) does NOT need to know about restrictions. It produces a final `ManaCost`. The payment pipeline (601.2h) checks each mana unit's restriction against the spell being cast. This is a clean separation.

### Q3: Multi-restriction mana (Boseiju / OR semantics)?

**Answer: One restriction with OR semantics, modeled as a single `ManaRestriction` variant.**

Analysis of real cards:
- **Boseiju, Who Shelters All:** "Spend this mana only to cast instant or sorcery spells." → `OnlyForSpellTypes(vec![Instant, Sorcery])`
- **Cavern of Souls:** "Spend this mana only to cast a creature spell." → `OnlyForSpellTypes(vec![Creature])`
- **Boseiju, Who Endures (channel):** Not a mana restriction — it's an activated ability.
- **Mishra's Workshop:** "Spend this mana only to cast artifact spells." → `OnlyForSpellTypes(vec![Artifact])`
- **Eldrazi Temple:** "Spend this mana only to cast colorless Eldrazi spells or activate abilities of Eldrazi." → Needs a compound restriction.

The existing `ManaRestriction::OnlyForSpellTypes(Vec<CardType>)` already has OR semantics across the type list (the mana can be spent if the spell has *any* of the listed types). This correctly handles Boseiju: `OnlyForSpellTypes(vec![Instant, Sorcery])` matches a spell that is an Instant OR a Sorcery.

For cards like Eldrazi Temple that restrict to "cast spells OR activate abilities," we need additional variants:

```rust
pub enum ManaRestriction {
    /// Mana can only be spent to cast spells with at least one of these types.
    /// OR semantics: spell matches if it has ANY listed type.
    OnlyForSpellTypes(Vec<CardType>),

    /// Mana can only be spent to activate abilities of permanents with at
    /// least one of these types.
    OnlyForAbilityTypes(Vec<CardType>),

    /// Compound: satisfies if ANY inner restriction is met.
    /// Models "cast Eldrazi spells or activate abilities of Eldrazi."
    AnyOf(Vec<ManaRestriction>),

    /// Mana can only be spent to cast spells with a specific name.
    /// (Unlikely in paper Magic but exists in some Alchemy designs.)
    OnlyForSpellName(String),
}
```

**Edge case — Boseiju + "cards with channel":** Boseiju, Who Shelters All says "instant or sorcery spells." The channel ability on Boseiju, Who Endures is an activated ability, not a spell — so this is a separate mana source, not a multi-restriction on the same mana. No special modeling needed.

**Edge case — "only on creatures that share a creature type":** Cavern of Souls' second ability says "creature spells you cast of the chosen type can't be countered." This is a *grant* (modeled by `ManaGrant`), triggered on spend. The restriction itself is just `OnlyForSpellTypes(vec![Creature])`. The "chosen type" filtering would require extending the restriction:

```rust
/// Mana can only be spent to cast creature spells of the chosen type.
OnlyForCreatureType(CreatureType),
```

This is a future variant. For now, `OnlyForSpellTypes` covers the most common cards. We should add `OnlyForCreatureType` when Cavern of Souls is actually implemented, but the architecture supports it cleanly.

### Q4: Interaction with cost modification pipeline

**Answer: The cost modification pipeline does NOT need to know about restrictions. They are cleanly separated concerns. However, cost modification can *create* costs that restricted mana then pays — and when it does, grants flow through.**

The casting sequence per rule 601.2:
1. **601.2e — Determine total cost:** Apply cost increases (Thalia), reductions (Electromancer), Trinisphere floor. Output: a `ManaCost` (just symbols, no mana source information).
2. **601.2g — Mana ability window:** Player may activate mana abilities to generate mana.
3. **601.2h — Pay total cost:** Player pays the locked-in `ManaCost` from their pool. *This* is where restrictions are checked.

**Thalia + Boseiju example:** Consider a contrived 0-mana instant (or a real one like Ancestral Vision cast off suspend). Thalia, Guardian of Thraben adds {1} to noncreature spells. Boseiju, Who Shelters All produces {C} restricted to "only instants and sorceries" with the grant "that spell can't be countered."

- 601.2e: Thalia applies (it's a noncreature spell). Cost becomes {1}.
- 601.2h: Boseiju's mana is restricted to instants/sorceries — this IS an instant, so the restriction is satisfied. The player pays {C} from Boseiju. The grant (uncounterable) is applied to the spell.

**Result:** Thalia's tax created a cost where none existed before. The restricted mana satisfies it. The grant flows through. This is correct per the rules — 601.2e doesn't need to know that Boseiju mana will be used, and 601.2h doesn't care that the cost was inflated by Thalia. But the *consequence* is meaningful: the cost increase inadvertently gave the player an opportunity to spend restricted mana with a powerful grant.

**Counter-example — Thalia + Cavern:** Thalia adds {1} to noncreature spells. Cavern produces creature-only mana. If the player casts a noncreature instant:
- 601.2e: Thalia applies. Cost = base + {1}.
- 601.2h: Cavern's creature-only mana is NOT eligible (spell isn't a creature). The Thalia tax cannot be paid with Cavern mana.

The key insight: **cost modification changes the amount to pay; restrictions filter which mana units are eligible to pay it.** These are independent. The `pay_costs` / `ManaPool::pay` call site needs the spell's characteristics to evaluate restrictions, but the cost modification pipeline doesn't. The grant system is downstream of both — it fires based on which atoms were actually consumed during payment.

---

## 3. Data Model

### 3.1 ManaPool — Dual-Track Structure

```rust
pub struct ManaPool {
    /// Fast path: unrestricted mana (99% of all mana in typical games).
    /// O(1) lookups. No per-unit metadata.
    ///
    /// IMPORTANT: Mana in `simple` may still be subject to blanket persistence
    /// effects (see §3.6). Blanket persistence is NOT stored per-atom — it is
    /// resolved at empty-time by querying active continuous effects.
    simple: HashMap<ManaType, u64>,

    /// Slow path: mana with per-unit metadata (restrictions, grants, or
    /// time-gated persistence like Firebending's "until end of combat").
    /// Only populated when special lands/abilities are in play.
    ///
    /// Stored as counted groups: identical atoms are coalesced into a
    /// single `(ManaAtom, u64)` entry. This avoids Vec bloat if a loop
    /// or repeated effect produces many identical restricted mana units.
    special: Vec<(ManaAtom, u64)>,
}
```

**Invariant:** A unit of mana lives in exactly one of `simple` or `special`, never both. Unrestricted mana without per-unit grants goes into `simple` (even if a blanket persistence effect applies to it). Mana with restrictions, grants, or time-gated persistence goes into `special`.

### 3.2 ManaAtom (already exists — minor updates)

```rust
pub struct ManaAtom {
    pub mana_type: ManaType,
    pub source_id: Option<ObjectId>,
    pub restrictions: Vec<ManaRestriction>,
    pub grants: Vec<ManaGrant>,
    pub persistence: ManaPersistence,
}
```

No structural changes needed. The existing `ManaAtom` is already correct. The `restrictions` Vec acts as AND — all restrictions must be satisfied. (In practice, a single mana unit almost never has multiple independent restrictions. If it does, the `AnyOf` variant handles OR within a single restriction.)

### 3.3 ManaPersistence — Blanket vs. Time-Gated

Two fundamentally different persistence patterns exist in Magic:

**Time-gated persistence (per-atom, lives in `special`):**
- **Firebending N** — "add N {R}. Until end of combat, you don't lose this mana as steps and phases end." The mana has a *specific expiration* (end of combat) and was *produced with* the persistence metadata. It belongs in `special` as `ManaAtom` with a time-gated persistence value.
- **Birgi, God of Storytelling** — "Whenever you cast a spell, add {R}. Until end of turn, you don't lose this mana as steps and phases end."

**Important: time-gated mana survives its source dying.** Birgi's ability is a triggered ability — it produces mana with persistence metadata stamped at production time. If Birgi dies afterward, the mana atom in `special` still has `UntilEndOf(EndOfTurn)` and will persist through subsequent step/phase transitions until cleanup. This is fundamentally different from Omnath's static ability, where killing Omnath immediately removes the blanket protection on the next `empty_with_reason` call.

**Blanket persistence (continuous effect, does NOT stamp atoms):**
- **Omnath, Locus of Mana** — "Green mana doesn't empty from your mana pool as steps and phases end." This is a *static ability* that applies a continuous effect to the pool-emptying rule. It doesn't stamp individual mana units — ALL green mana in the pool is affected, including mana produced by basic Forests long before Omnath entered.
- **Upwelling** — "Mana pools don't empty as steps and phases end." Blanket effect on all mana.
- **Kruphix, God of Horizons** — "If you would lose unspent mana, that mana becomes colorless instead." Importantly this *preserves* any other restrictions--a colored Cavern of Souls mana that is converted to colorless this way can still only be spent on a creature spell.

**Why this distinction matters for performance:** In a mono-green Commander deck built around Omnath, the player may accumulate 20+ green mana in the pool. If each unit were a `ManaAtom` in `special`, we'd have 20+ atoms with no restrictions, no grants — just a persistence flag. Every `can_pay`, `pay`, and `total` call would iterate the sidecar for no benefit. The entire dual-track optimization collapses.

**Design:**

```rust
pub enum ManaPersistence {
    /// Normal mana — empties at every step/phase transition (rule 106.4).
    Normal,
    /// Persists until a specific game point, then empties.
    /// Produced by effects like Firebending ("until end of combat")
    /// or Birgi ("until end of turn").
    /// Lives in `special` sidecar.
    UntilEndOf(PersistenceExpiry),
}

// NOTE: No `Indefinite` variant. Every "mana doesn't empty" effect we can
// find is either:
//   - A blanket static ability (Omnath, Upwelling) → BlanketPersistenceSet
//   - A replacement effect (Kruphix, Horizon Stone) → Phase 6 replacement
// No card produces mana with inherent indefinite persistence as a per-atom
// property. If one surfaces, adding the variant back is trivial.

/// When time-gated persistent mana expires.
pub enum PersistenceExpiry {
    EndOfCombat,
    EndOfTurn,
    EndOfPhase,
    /// Custom: cleared when a specific event fires (rare).
    /// E.g., "until you cast your next spell."
    Custom(/* details TBD */),
}
```

**Blanket persistence** (Omnath, Upwelling) is modeled as a **continuous effect** — NOT as per-atom metadata. The engine queries active continuous effects during `empty_with_reason`:

```rust
impl ManaPool {
    pub fn empty_with_reason(
        &mut self,
        reason: ManaEmptyReason,
        blanket_persist: &BlanketPersistenceSet,
    ) {
        // Simple pool: retain mana types covered by blanket persistence
        for (mana_type, amount) in self.simple.iter_mut() {
            if !blanket_persist.persists(*mana_type) {
                *amount = 0;
            }
        }
        // Special pool: check per-atom persistence + blanket
        self.special.retain(|(atom, _count)| {
            match &atom.persistence {
                ManaPersistence::Normal => {
                    // Normal atom in special (has restrictions/grants but
                    // no persistence). Survives only if blanket protects it.
                    blanket_persist.persists(atom.mana_type)
                }
                ManaPersistence::UntilEndOf(expiry) => {
                    // Time-gated atom. Survives unless this empty reason
                    // matches its expiry. This is why ManaEmptyReason
                    // exists — a Birgi atom (EndOfTurn) survives
                    // StepOrPhase but not TurnEnd.
                    !expiry.matches(&reason)
                }
            }
        });
    }
}

/// Describes which mana types are protected from emptying by blanket
/// continuous effects (Omnath, Upwelling, etc.).
///
/// Built by the continuous effects layer each time the pool is emptied.
pub struct BlanketPersistenceSet {
    /// If true, ALL mana persists (Upwelling).
    pub all: bool,
    /// Specific mana types that persist (Omnath = {Green}).
    pub types: HashSet<ManaType>,
}

impl BlanketPersistenceSet {
    /// Whether this mana type is protected from emptying by a blanket effect.
    ///
    /// Does NOT branch on `ManaEmptyReason` — blanket effects like Omnath
    /// and Upwelling protect across all transitions (steps, phases, turns).
    /// Blanket protection only ends when the source permanent leaves play
    /// or loses its ability (at which point the continuous effects layer
    /// simply stops including it in the set).
    ///
    /// `ManaEmptyReason` IS still needed by `empty_with_reason` for
    /// time-gated atoms in `special` — e.g. Birgi mana (EndOfTurn)
    /// survives StepOrPhase but expires at TurnEnd. The reason just
    /// isn't relevant to *blanket* checks.
    pub fn persists(&self, mana_type: ManaType) -> bool {
        self.all || self.types.contains(&mana_type)
    }

    pub fn none() -> Self {
        BlanketPersistenceSet { all: false, types: HashSet::new() }
    }
}
```

**When the source of a blanket effect leaves play** (Omnath dies), the next `empty_with_reason` call will no longer include green in the `BlanketPersistenceSet`, and all that green mana empties normally. No per-atom cleanup needed.

**Key insight:** This design means Omnath's green mana stays in `simple` as plain `u64` counts. The persistence logic lives entirely in the emptying code path, not in the storage. The `special` sidecar is reserved for mana that genuinely has per-unit metadata.

**Performance note on `add_special` coalescing:** `add_special` does a linear scan of `special` to find a group with an identical `ManaAtom` (same type, restrictions, grants, persistence). With n ≤ ~10 groups in realistic games, this is a handful of struct comparisons — cheaper than a HashMap lookup (hash computation + potential collision). A HashMap would only win with 50+ distinct restriction profiles in one pool, which doesn't occur in real Magic. If the same source adds restricted mana multiple times in a phase, it coalesces into the same group (count incremented).

### 3.4 ManaRestriction (expand existing enum)

```rust
pub enum ManaRestriction {
    /// Spend only to cast spells with at least one of these types.
    OnlyForSpellTypes(Vec<CardType>),

    /// Spend only to activate abilities of permanents with at least one
    /// of these types.
    OnlyForAbilityTypes(Vec<CardType>),

    /// Compound OR: satisfies if ANY inner restriction matches.
    AnyOf(Vec<ManaRestriction>),

    /// Spend only to cast creature spells of a specific creature type.
    /// (Cavern of Souls' "chosen type" restriction.)
    OnlyForCreatureType(CreatureType),
}
```

### 3.5 SpendContext — What is the mana being spent on?

A new struct passed into the payment pipeline so restrictions can be evaluated:

```rust
/// Context describing what the mana is being spent on.
/// Used by the payment pipeline to evaluate ManaRestriction.
pub struct SpendContext<'a> {
    /// The spell or ability being paid for.
    pub purpose: SpendPurpose<'a>,
}

pub enum SpendPurpose<'a> {
    /// Casting a spell. Provides the spell's characteristics for restriction checks.
    CastSpell {
        card_types: &'a HashSet<CardType>,
        subtypes: &'a HashSet<Subtype>,
        name: &'a str,
    },
    /// Activating an ability on a permanent.
    ActivateAbility {
        source_card_types: &'a HashSet<CardType>,
        source_subtypes: &'a HashSet<Subtype>,
    },
    /// Paying a special action cost (e.g., morph). No restrictions typically apply.
    SpecialAction,
}
```

**Critical: `SpendContext` must use effective characteristics, not printed.**

The `card_types` and `subtypes` in `SpendPurpose::CastSpell` must reflect the spell's characteristics *after* continuous effects are applied — not the raw `card_data` fields. This matters for:

- **Changeling** — A creature card with changeling has every creature type. An "only for Elf spells" restriction must see all creature types in `subtypes`, which it will because changeling is a characteristic-defining ability that sets subtypes.
- **Maskwood Nexus** — "Creatures you control are every creature type. The same is true for creature spells you control and creature cards you own that aren't on the battlefield." With Maskwood Nexus active, a creature spell in hand has every creature type. An "only for Elf spells" restriction is satisfied.

The `cast_spell` call site builds `SpendContext` from `oracle::characteristics` (which routes through the Phase 5 layer system), not from raw `card_data`:

```rust
let effective_types = oracle::characteristics::get_types(self, card_id);
let effective_subtypes = oracle::characteristics::get_subtypes(self, card_id);
let spend_ctx = SpendContext {
    purpose: SpendPurpose::CastSpell {
        card_types: &effective_types,
        subtypes: &effective_subtypes,
        name: &card_data.name,
    },
};
```

The restriction evaluator itself is unchanged — it just sees the final characteristics. The complexity of Maskwood/changeling lives in the characteristics oracle, where it belongs.

### 3.6 Restriction Evaluation

`ManaRestriction::allows` and the internal `ManaAtom` helper are **private to `ManaPool`**. External callers use `can_pay_with_context` (query) and `pay_with_plan` (mutation) — they never evaluate individual atoms directly.

```rust
impl ManaRestriction {
    /// Check if this restriction allows spending in the given context.
    fn allows(&self, ctx: &SpendContext) -> bool {
        match self {
            ManaRestriction::OnlyForSpellTypes(types) => {
                match &ctx.purpose {
                    SpendPurpose::CastSpell { card_types, .. } => {
                        types.iter().any(|t| card_types.contains(t))
                    }
                    _ => false, // Spell-only restriction fails for abilities
                }
            }
            ManaRestriction::OnlyForAbilityTypes(types) => {
                match &ctx.purpose {
                    SpendPurpose::ActivateAbility { source_card_types, .. } => {
                        types.iter().any(|t| source_card_types.contains(t))
                    }
                    _ => false,
                }
            }
            ManaRestriction::AnyOf(inner) => {
                inner.iter().any(|r| r.allows(ctx))
            }
            ManaRestriction::OnlyForCreatureType(ct) => {
                match &ctx.purpose {
                    SpendPurpose::CastSpell { card_types, subtypes, .. } => {
                        card_types.contains(&CardType::Creature)
                            && subtypes.contains(&Subtype::Creature(*ct))
                    }
                    _ => false,
                }
            }
        }
    }
}

impl ManaAtom {
    /// Internal helper — checks all restrictions on this atom.
    /// An atom with no restrictions always allows spending.
    fn allows_spend(&self, ctx: &SpendContext) -> bool {
        self.restrictions.iter().all(|r| r.allows(ctx))
    }
}
```

The public validation API is `ManaPool::can_pay_with_context`, which orchestrates the constraint-satisfaction check across both `simple` and `special`. `pay_with_plan` validates the plan against the `SpendContext` before executing it — if any specified special group's atom fails `allows_spend`, the payment is rejected.

---

## 4. API Surface — ManaPool Changes

### 4.1 New Methods

```rust
impl ManaPool {
    /// Add special mana (restricted, granted, or persistent).
    pub fn add_special(&mut self, atom: ManaAtom) { ... }

    /// Total mana of a given type available for a specific spend context.
    /// Counts unrestricted + eligible special atoms.
    pub fn amount_for(&self, mana_type: ManaType, ctx: &SpendContext) -> u64 { ... }

    /// Total mana of any type available for a specific spend context.
    pub fn total_for(&self, ctx: &SpendContext) -> u64 { ... }

    /// Check if a ManaCost can be paid, considering restrictions.
    /// This is the restriction-aware version of `can_pay`.
    pub fn can_pay_with_context(&self, cost: &ManaCost, ctx: &SpendContext) -> bool { ... }

    /// Pay a ManaCost using a ManaPaymentPlan that specifies exactly which
    /// atoms/pool units to spend. See §4.3.
    pub fn pay_with_plan(&mut self, plan: &ManaPaymentPlan) -> Result<(), String> { ... }

    /// Empty the pool, respecting both per-atom persistence and blanket
    /// continuous effects. See §3.3 for the full signature and logic.
    pub fn empty_with_reason(
        &mut self,
        reason: ManaEmptyReason,
        blanket_persist: &BlanketPersistenceSet,
    ) { ... }

    /// Whether any special mana exists in the pool.
    /// Used to short-circuit: if false, all existing callers can use the
    /// fast path unchanged.
    pub fn has_special(&self) -> bool { !self.special.is_empty() }

    /// Iterate over special atoms (for UI display, DP queries).
    pub fn special_atoms(&self) -> &[ManaAtom] { &self.special }

    /// Collect all grants from atoms that were spent in the last payment.
    /// Called after pay_with_plan to apply grants (e.g., uncounterable).
    /// Returns the grants and clears the internal "last spent" buffer.
    pub fn drain_spent_grants(&mut self) -> Vec<ManaGrant> { ... }
}

pub enum ManaEmptyReason {
    StepOrPhase,
    TurnEnd,
}
```

### 4.2 Existing Methods — Migration Path

During Phase B, all callers migrate from old to new methods:

- `can_pay()` → `can_pay_with_context()`. The old `can_pay` is **deprecated in Phase B and removed in Phase C**. Keeping dead code around invites bugs where a caller forgets to use the context-aware version and silently ignores restricted mana. During Phase B the compiler can enforce migration via `#[deprecated]`.
- `pay()` → `pay_with_plan()`. Same deprecation schedule.
- `empty()` → `empty_with_reason()`. `empty()` is retained as a test convenience only (calls `empty_with_reason(StepOrPhase, &BlanketPersistenceSet::none())` then clears Indefinite atoms too).
- `add()`, `amount()`, `total()` remain — they operate on `simple` and are still useful for the unrestricted-mana fast path.

This is a tight migration: Phase B changes all call sites (using `simple_only()` for backward compat), Phase C removes the deprecated methods.

### 4.3 ManaPaymentPlan

Instead of the current `generic_allocation: HashMap<ManaType, u64>`, a richer payment plan is needed when special mana is involved:

```rust
/// A fully specified plan for paying a mana cost.
///
/// Built by the DecisionProvider (or auto-builder), validated and executed
/// by ManaPool::pay_with_plan.
pub struct ManaPaymentPlan {
    /// Mana to draw from the unrestricted simple pool.
    /// Maps ManaType → amount.
    pub from_simple: HashMap<ManaType, u64>,

    /// Mana to draw from specific special groups.
    /// Each entry is (group_index, amount_to_spend).
    /// `pay_with_plan` decrements the group's count and removes it if
    /// the count reaches zero.
    pub from_special: Vec<(usize, u64)>,
}
```

**Why counted groups instead of per-atom indices?** The `special` sidecar uses `Vec<(ManaAtom, u64)>` — identical atoms are coalesced. The payment plan references groups by index and specifies how many to spend from each. This eliminates the reindexing hazard of multi-delete (decrementing a count is safe regardless of order), and handles infinite-mana loops gracefully (1000 identical restricted mana = one group with count 1000, not 1000 Vec entries). `pay_with_plan` processes `from_special` in a single pass: decrement counts, collect grants proportionally, then `retain` to drop zero-count groups.

---

## 5. Integration Points

### 5.1 `engine/costs.rs` — `pay_costs` / `pay_single_cost`

Current signature:
```rust
pub fn pay_costs(&mut self, costs: &[Cost], player_id: PlayerId,
    source_id: ObjectId, generic_allocation: &HashMap<ManaType, u64>)
    -> Result<(), String>
```

New signature:
```rust
pub fn pay_costs(&mut self, costs: &[Cost], player_id: PlayerId,
    source_id: ObjectId, mana_plan: Option<&ManaPaymentPlan>)
    -> Result<(), String>
```

`mana_plan` is `Option` because some costs have no mana component at all (e.g., Dread Return flashback: "Sacrifice three creatures"). Passing `None` means "no mana payment needed." If a `Cost::Mana` arm is encountered with `mana_plan: None`, `pay_single_cost` returns an error (this is a bug in the caller — cost assembly should have ensured a plan exists when mana is owed).

For callers that don't involve special mana, `ManaPaymentPlan::simple_only(generic_allocation)` wraps a `HashMap<ManaType, u64>` into `Some(ManaPaymentPlan { from_simple: ..., from_special: vec![] })`.

The `Cost::Mana` arm in `pay_single_cost` changes from:
```rust
player.mana_pool.pay(mana_cost, generic_allocation)
```
to:
```rust
player.mana_pool.pay_with_plan(mana_plan)
```

After payment, if the spell context has grants, call `drain_spent_grants()` and apply them to the spell on the stack.

### 5.2 `engine/cast.rs` — `cast_spell`

Currently calls `decisions.choose_generic_mana_allocation(...)`. This becomes:

```rust
// Use effective characteristics (handles Changeling, Maskwood Nexus, etc.)
let effective_types = oracle::characteristics::get_types(self, card_id);
let effective_subtypes = oracle::characteristics::get_subtypes(self, card_id);
let spend_ctx = SpendContext {
    purpose: SpendPurpose::CastSpell {
        card_types: &effective_types,
        subtypes: &effective_subtypes,
        name: &card_data.name,
    },
};
let mana_plan = decisions.choose_mana_payment(
    self, player_id, &total_cost, &spend_ctx,
);
self.pay_costs(&costs, player_id, card_id, Some(&mana_plan))?;

// Apply grants from special mana that was spent
let grants = self.get_player_mut(player_id)?.mana_pool.drain_spent_grants();
for grant in grants {
    match grant {
        ManaGrant::GrantKeyword(kw) => {
            // Add keyword to the stack entry (e.g., uncounterable)
            // Implementation details depend on Phase 6 stack metadata
        }
    }
}
```

### 5.3 `engine/mana.rs` — `resolve_mana_effect`

Currently does:
```rust
player.mana_pool.add(*mana_type, *amount);
```

For special mana, `ManaOutput` needs extension:

```rust
pub struct ManaOutput {
    pub mana: HashMap<ManaType, u64>,
    /// If present, produced mana carries these restrictions/grants.
    /// None = unrestricted (use simple pool path).
    pub special: Option<ManaProducedMeta>,
}

pub struct ManaProducedMeta {
    pub restrictions: Vec<ManaRestriction>,
    pub grants: Vec<ManaGrant>,
    pub persistence: ManaPersistence,
}
```

The `resolve_mana_effect` becomes:
```rust
match &output.special {
    None => {
        // Fast path — unrestricted
        for (&mana_type, &amount) in &output.mana {
            player.mana_pool.add(mana_type, amount);
        }
    }
    Some(meta) => {
        for (&mana_type, &amount) in &output.mana {
            for _ in 0..amount {
                player.mana_pool.add_special(ManaAtom {
                    mana_type,
                    source_id: Some(permanent_id),
                    restrictions: meta.restrictions.clone(),
                    grants: meta.grants.clone(),
                    persistence: meta.persistence,
                });
            }
        }
    }
}
```

### 5.4 `ui/decision.rs` — DecisionProvider Trait

Replace `choose_generic_mana_allocation` with a more general method:

```rust
/// Choose how to pay a mana cost from the pool.
///
/// When the pool has no special mana, this reduces to generic allocation
/// (same as the old choose_generic_mana_allocation). When special mana
/// exists, the player must decide which special atoms to spend.
fn choose_mana_payment(
    &self,
    game: &GameState,
    player_id: PlayerId,
    mana_cost: &ManaCost,
    spend_context: &SpendContext,
) -> ManaPaymentPlan;
```

**Backward compatibility for DPs:** A `build_default_payment_plan()` free function replaces `auto_allocate_generic()`. It:
1. Prefers spending eligible special mana first (for restrictions — "use it or lose it").
2. Falls back to unrestricted mana for remaining symbols.
3. For generic symbols, greedily assigns from surplus.

`PassiveDecisionProvider`, `ScriptedDecisionProvider`, and `RandomDecisionProvider` all call this default. `CliDecisionProvider` can eventually show special mana options to the player.

### 5.5 `oracle/mana_helpers.rs` — Affordability Queries

`castable_spells` and `can_pay` checks need restriction awareness:

- `ManaPool::can_pay` (no context) remains for backward compat — it ignores special mana entirely (conservative: pretends special mana doesn't exist).
- `ManaPool::can_pay_with_context` is the new restriction-aware check.
- `castable_spells` constructs a `SpendContext` for each spell in hand and calls `can_pay_with_context`.
- `available_mana_sources` doesn't change — it reports what sources *can be activated*, not what the produced mana can pay for.
- `remaining_cost_after_pool` gains an optional `SpendContext` parameter to account for eligible special atoms when computing remaining cost.

### 5.6 `engine/turns.rs` — Mana Pool Emptying

Currently calls `player.mana_pool.empty()` at step/phase transitions. Change to:
```rust
let blanket = build_blanket_persistence_set(game, player_id);
player.mana_pool.empty_with_reason(ManaEmptyReason::StepOrPhase, &blanket);
```

At end of turn (cleanup step, after other cleanup):
```rust
let blanket = build_blanket_persistence_set(game, player_id);
player.mana_pool.empty_with_reason(ManaEmptyReason::TurnEnd, &blanket);
```

`build_blanket_persistence_set` queries active continuous effects (Phase 5 layer system) for static abilities like Omnath and Upwelling. Before the layer system exists, it returns `BlanketPersistenceSet::none()` — identical to current behavior.

---

## 6. Performance Considerations

- **Fast path preservation:** When `special.is_empty()` (the vast majority of games, especially fuzz/AI training), all operations are identical to today. The `has_special()` check is a single Vec length test.
- **Blanket persistence stays in fast path:** Omnath-style effects keep mana in `simple`. No per-atom overhead for the common "lots of green mana" Commander scenario. The only cost is building `BlanketPersistenceSet` once per empty call (queries continuous effects — cheap).
- **Special path cost:** `pay_with_plan` iterates the `special` Vec (typically 1-5 atoms). `can_pay_with_context` does a constraint-satisfaction check over special atoms. For realistic game states this is negligible.
- **No allocation in hot path:** `ManaPaymentPlan` is stack-allocated (HashMap + Vec of group refs). No heap allocation beyond what HashMap already does.
- **Counted groups avoid bloat:** Infinite-mana loops producing identical restricted atoms = one group entry with a large count, not thousands of Vec elements. `pay_with_plan` decrements counts in-place (O(1) per group), then a single `retain` pass drops zero-count groups.

---

## 7. Grant Application Flow

When special mana with grants is spent to cast a spell:

1. `pay_with_plan()` moves spent `ManaAtom`s out of `special`, collecting their `grants` into an internal `last_spent_grants: Vec<ManaGrant>` buffer on `ManaPool`.
2. After `pay_costs` returns, `cast_spell` calls `drain_spent_grants()`.
3. Each grant is applied to the spell's stack entry:
   - `GrantKeyword(Uncounterable)` → set a flag on `StackEntry` (or add to a keywords set). The `resolve_spell` / counter-spell logic checks this flag.
   - `GrantKeyword(Haste)` → for creatures, applied when the permanent enters the battlefield. Store on the stack entry and transfer to `BattlefieldEntity` in `resolve_spell`.

**Edge case — mana spent on abilities:** If `ManaGrant::GrantKeyword` is granted from mana spent to activate an ability, does it apply? This is card-specific. Cavern of Souls says "creature spells you cast ... can't be countered" — it's the restriction + grant on the mana-for-spells path. Arena of Glory's haste grant is also spell-specific. For now, grants only apply during `cast_spell`. If a card grants something on ability activation, it would need a different mechanism (likely a triggered ability, not a mana grant).

---

## 8. Uncertain Edge Cases

These need card-specific verification before implementation:

1. **Mana spent across multiple cost components:** If a spell has both a mana cost and an additional cost (e.g., kicker with a mana component), and restricted mana is spent, does the restriction check against the spell or against the specific cost component? **Answer: the restriction checks the spell being cast, not the individual cost component.** Additional costs (kicker, buyback) and alternative costs (flashback, overload) are all part of "casting the spell" — CR 601.2b–f assembles the total cost, which is then paid as a unit in 601.2h. Cavern says "to cast creature spells" — the entire act of casting is the scope. Confirmed.

2. **Selvala / Metalworker / mana abilities that produce variable amounts:** These produce mana with no predetermined type. The restriction/grant metadata would need to be attached at resolution time based on the ability's definition, not at production time. The current design handles this — `resolve_mana_effect` attaches metadata from `ManaOutput.special` regardless of how the amount was determined.

3. **Mana doubling effects (Doubling Cube, Mana Reflection, Nyxbloom Ancient):** Does doubled/tripled mana inherit restrictions from the original? **Answer: No.** Per CR 701.10f and CR 106.6 example:

   > *"A player's mana pool contains {R}{G} which can be spent only to cast creature spells. That player activates Doubling Cube's ability [...] The player's mana pool now has {R}{R}{G}{G} in it, {R}{G} of which can be spent on anything."*

   The new mana is freshly produced — it does NOT inherit restrictions, grants, or persistence from the original. `Doubling Cube` adds unrestricted mana to `simple`. Mana Reflection and Nyxbloom Ancient are replacement effects ("if you would add mana, add that much plus..." / "if a land you control would add mana, it adds three times that much instead") — the replacement produces additional mana of the same type. Per the CR example, the additional mana is unrestricted. Implementation: the replacement effect in Phase 6 produces mana via `add()` (simple path), not `add_special()`.

4. **Snow mana:** Snow mana (`ManaSymbol::Snow`) must be paid with mana from a snow source. This is a cost-side constraint, not a spending restriction — it's the same mechanism as colored symbols ({R} must be paid with red mana). The existing `ManaSymbol::Snow` handling checks `source_id` against snow permanents at payment time. This is orthogonal to `ManaRestriction` and requires no special handling from T12.

5. **Convoke / Delve / Improvise:** These alternative payment methods bypass the mana pool entirely (tapping creatures, exiling cards from graveyard). They don't interact with `ManaRestriction` — they're not mana. No special handling needed.

---

## 9. Test Plan

### Unit Tests (types/mana.rs)

| Test | Description |
|------|-------------|
| `test_add_special_atom` | Add a restricted atom, verify `has_special()` and `special_atoms()` |
| `test_amount_for_unrestricted` | `amount_for` with no context returns simple count |
| `test_amount_for_with_eligible_special` | Creature-only {G} + creature context → counts |
| `test_amount_for_with_ineligible_special` | Creature-only {G} + instant context → doesn't count |
| `test_can_pay_with_context_simple_only` | No special mana, identical to `can_pay` |
| `test_can_pay_with_context_uses_special` | Restricted mana makes an otherwise-unpayable cost payable |
| `test_can_pay_with_context_rejects_ineligible` | Restricted mana for wrong type doesn't help |
| `test_pay_with_plan_simple` | Pay from simple pool only |
| `test_pay_with_plan_special` | Pay using specific special groups by index + count |
| `test_pay_with_plan_mixed` | Some from simple, some from special |
| `test_pay_with_plan_collects_grants` | Spent atoms' grants appear in `drain_spent_grants()` |
| `test_empty_with_reason_step_no_blanket` | Normal mana empties, time-gated atoms (EndOfTurn, EndOfCombat) persist |
| `test_empty_with_reason_turn_no_blanket` | Normal + EndOfTurn atoms empty at TurnEnd. EndOfCombat atoms also empty. |
| `test_empty_with_blanket_green` | With blanket green persistence, green mana in `simple` survives step/phase empty |
| `test_empty_with_blanket_all` | With blanket all persistence (Upwelling), all simple mana survives |
| `test_blanket_removed_then_empties` | Green mana persisted by blanket; on next empty without blanket, it empties |
| `test_firebending_until_end_of_combat` | Time-gated atom with `UntilEndOf(EndOfCombat)` persists through declare blockers, empties at end of combat |
| `test_restriction_allows_spell_type_match` | `OnlyForSpellTypes([Creature])` matches creature spell |
| `test_restriction_rejects_spell_type_mismatch` | `OnlyForSpellTypes([Creature])` rejects instant |
| `test_restriction_any_of` | `AnyOf` with mixed restrictions evaluates correctly |
| `test_restriction_creature_type` | `OnlyForCreatureType(Elf)` matches Elf creature spell |
| `test_restriction_changeling_matches_any_type` | Changeling creature (all types) matches `OnlyForCreatureType(Elf)` |
| `test_special_coalesce_identical_atoms` | Adding 5 identical restricted atoms → one group with count 5 |
| `test_special_no_coalesce_different_grants` | Atoms with different grants stay separate groups |

### Unit Tests (engine/costs.rs)

| Test | Description |
|------|-------------|
| `test_pay_costs_with_mana_plan` | `pay_costs` with `ManaPaymentPlan` works end-to-end |
| `test_pay_costs_backward_compat` | `ManaPaymentPlan::simple_only` works like old `generic_allocation` |

### Unit Tests (ui/decision.rs)

| Test | Description |
|------|-------------|
| `test_build_default_payment_plan_no_special` | Identical to `auto_allocate_generic` |
| `test_build_default_payment_plan_prefers_special` | Eligible restricted mana used first |
| `test_build_default_payment_plan_skips_ineligible` | Ineligible restricted mana not used |

### Integration Tests

| Test | Description |
|------|-------------|
| `test_cavern_mana_pays_creature` | Cavern-restricted mana successfully pays for creature spell |
| `test_cavern_mana_rejected_for_instant` | Cavern-restricted mana cannot pay for instant |
| `test_cavern_grant_uncounterable` | Creature cast with Cavern mana gains uncounterable |
| `test_mixed_pool_creature_then_instant` | Restricted for creature + unrestricted. Cast creature (uses restricted), then instant (uses unrestricted). Both succeed. |
| `test_trinisphere_plus_restricted` | Trinisphere floor + restricted mana. Restricted mana counts toward the {3} if eligible. |
| `test_thalia_boseiju_grant_flow` | Thalia taxes a 0-mana instant to {1}. Boseiju's instant-only mana pays it and imparts uncounterable grant. |
| `test_doubling_cube_does_not_inherit_restrictions` | Pool has creature-only {G}. Doubling Cube adds unrestricted {G}. Only original is restricted. |
| `test_maskwood_nexus_enables_elf_restriction` | With Maskwood Nexus, any creature spell satisfies `OnlyForCreatureType(Elf)` |
| `test_persistence_birgi` | Red mana with `UntilEndOf(EndOfTurn)` persistence survives phase transition |
| `test_persistence_empties_at_turn_end` | `UntilEndOf(EndOfTurn)` mana empties during cleanup |
| `test_birgi_mana_survives_birgi_dying` | Birgi produces mana, Birgi dies, mana persists through next step (atom metadata, not blanket) |
| `test_omnath_blanket_persistence` | Omnath on battlefield → green mana persists across phases via blanket set, not atom metadata |
| `test_omnath_dies_mana_empties` | Omnath leaves battlefield → next phase transition empties green mana |
| `test_firebending_combat_persistence` | Firebending mana persists through combat steps, empties at end of combat |
| `test_fuzz_games_with_restricted_mana` | Fuzz harness with Cavern-like lands doesn't panic |

---

## 10. Migration Strategy

### Phase A: Wire `ManaPool` sidecar (T12b — implementation ticket)

1. Add `special: Vec<(ManaAtom, u64)>` and `last_spent_grants: Vec<ManaGrant>` fields to `ManaPool`. `add_special` coalesces identical atoms into existing groups.
2. Implement `add_special`, `has_special`, `special_atoms`, `drain_spent_grants`.
3. Implement `empty_with_reason` with `BlanketPersistenceSet` parameter (keep `empty()` as convenience with `BlanketPersistenceSet::none()`).
4. Implement `can_pay_with_context` and `pay_with_plan`.
5. Add `SpendContext`, `SpendPurpose`, `ManaPaymentPlan`, `ManaEmptyReason`, `BlanketPersistenceSet`, `PersistenceExpiry` types. (`ManaPersistence` has two variants: `Normal` and `UntilEndOf`; no `Indefinite`.)
6. Add `ManaRestriction::allows()` (private) and `ManaAtom::allows_spend()` (private).
7. **All existing methods unchanged.** All existing tests pass without modification.

### Phase B: Integrate into engine (T12c or part of T12b)

1. Add `ManaProducedMeta` to `ManaOutput`. Default `special: None`.
2. Update `resolve_mana_effect` to route special mana to `add_special`.
3. Add `ManaPaymentPlan::simple_only(HashMap<ManaType, u64>)` constructor.
4. Change `pay_costs` signature to take `Option<&ManaPaymentPlan>` instead of `&HashMap`. Update all call sites to use `Some(&ManaPaymentPlan::simple_only(...))` — zero behavior change. Mana-free costs pass `None`.
5. Add `build_default_payment_plan` to `ui/decision.rs`.
6. Replace `choose_generic_mana_allocation` with `choose_mana_payment` on `DecisionProvider`. All existing DPs delegate to `build_default_payment_plan`. **Trait method count stays at 8** (replace, not add).
7. Update `castable_spells` to use `can_pay_with_context`.

### Phase C: First restricted-mana cards

1. Implement Cavern of Souls (or a simpler test card) with `ManaOutput.special`.
2. Wire grant application in `cast_spell`.
3. Update `empty()` call sites in `turns.rs` to `empty_with_reason()` with `BlanketPersistenceSet::none()` (no blanket effects yet).
4. Implement Birgi or a test persistent-mana card (time-gated: `UntilEndOf(EndOfTurn)`).
5. When Phase 5 layer system is available: implement `build_blanket_persistence_set()` querying continuous effects. Wire Omnath/Upwelling as static abilities producing blanket persistence.

### Rollout Safety

- Phase A and B are **pure additive** — no existing behavior changes. All 287+ existing tests continue to pass.
- Phase C introduces the first behavior change (special mana actually appearing in the pool). Integration tests cover the new paths.
- Fuzz harness gains decks with restricted-mana lands to exercise the new code paths.

---

## 11. Summary of Decisions

| Question | Decision |
|----------|----------|
| Q1: Where does metadata live? | On `ManaPool` as `special: Vec<(ManaAtom, u64)>` counted-group sidecar |
| Q2: Trinisphere interaction? | Cost modification is independent. Restricted mana counts toward total if restriction satisfied. |
| Q3: Multi-restriction / OR? | Single `ManaRestriction` with OR within `OnlyForSpellTypes`. `AnyOf` for cross-category OR. |
| Q4: Cost modification interaction? | No interaction needed. Clean separation at 601.2e (modify cost) vs 601.2h (pay cost). |
