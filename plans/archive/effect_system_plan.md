# Effect System Design: Primitives & Phasing

## Proposed Architecture

Based on a comprehensive reading of rules 609–616 (effects, continuous effects, replacement/prevention), rule 701 (keyword actions), and your implementation phases, here's the design.

### Core Types

```rust
/// What an effect does when it resolves (one-shot effects, rule 610)
enum Primitive {
    // === Zone movement (rule 701) ===
    Destroy,                    // 701.8 — move to graveyard (respects indestructible/regenerate)
    Exile,                      // 701.13
    Sacrifice,                  // 701.21 — controller moves to graveyard, no regeneration
    ReturnToHand,               // "bounce"
    ReturnToBattlefield,        // from exile/graveyard
    PutOnTopOfLibrary,
    PutOnBottomOfLibrary,
    ShuffleIntoLibrary,
    Mill(CountExpr),            // 701.17
    Discard(CountExpr),         // 701.9

    // === Damage & life ===
    DealDamage(AmountExpr),     // rule 120
    GainLife(AmountExpr),
    LoseLife(AmountExpr),
    PayLife(AmountExpr),        // cost-like, used in Phyrexian mana etc.

    // === Card flow ===
    DrawCards(CountExpr),
    Scry(CountExpr),            // 701.22
    Surveil(CountExpr),         // 701.25
    Reveal,                     // 701.20
    Search(ZoneFilter),         // 701.23

    // === Mana ===
    ProduceMana(ManaOutput),

    // === Counters ===
    AddCounters(CounterType, CountExpr),
    RemoveCounters(CounterType, CountExpr),
    Proliferate,                // 701.34

    // === Tokens ===
    CreateToken(TokenDef, CountExpr),   // 701.7

    // === Combat ===
    Fight,                      // 701.14 — each deals power to the other
    Tap,                        // 701.26
    Untap,

    // === Control & characteristics ===
    GainControl(Duration),      // layer 2
    SetPowerToughness(AmountExpr, AmountExpr),  // layer 7b
    ModifyPowerToughness(AmountExpr, AmountExpr), // layer 7c
    AddAbility(KeywordAbility, Duration),        // layer 6
    RemoveAbility(KeywordAbility, Duration),     // layer 6
    ChangeColor(Color, Duration),                // layer 5
    ChangeType(TypeChange, Duration),            // layer 4

    // === Counter spells/abilities ===
    CounterSpell,               // 701.6

    // === Misc keyword actions ===
    Explore,                    // 701.44
    Connive(CountExpr),         // 701.50
    Amass(Subtype, CountExpr),  // 701.47
    Transform,                  // 701.27
    Detain,                     // 701.35
}
```

### Combinators (compose primitives into real card effects)

```rust
enum Effect {
    /// Apply a primitive to resolved targets
    Atom(Primitive, TargetSpec),

    /// Execute effects in order: [DealDamage, DrawCards]
    Sequence(Vec<Effect>),

    /// "If [condition], [effect]" — intervening if (rule 603.4)
    Conditional(Condition, Box<Effect>),

    /// Optional: "you may [effect]" (rule 603.5)
    Optional(Box<Effect>),

    /// Modal: "Choose one/two —" (rule 700.2)
    Modal { count: ModalCount, modes: Vec<Effect> },

    /// "For each [selector], [effect]"
    ForEach(Selector, Box<Effect>),

    /// "Do this N times" or "repeat this process"
    Repeat(CountExpr, Box<Effect>),

    /// Register a continuous effect (rule 611) with duration
    ApplyContinuous(ContinuousEffect),

    /// Register a replacement effect (rule 614)
    ApplyReplacement(ReplacementEffect),

    /// Register a prevention shield (rule 615)
    ApplyPrevention(PreventionEffect),

    /// Create a delayed triggered ability (rule 603.7)
    CreateDelayedTrigger(TriggerCondition, Box<Effect>, Duration),

    /// Escape hatch: looked up in a registry of custom handlers
    Custom(CardId),
}
```

### Supporting Types

```rust
/// How amounts are determined
enum AmountExpr {
    Fixed(u64),
    Variable,               // X, chosen on cast
    CountOf(Selector),      // "equal to the number of creatures you control"
    TargetPower,            // "equal to that creature's power"
    TargetToughness,
    DamageDealt,            // "equal to the damage dealt this way"
}

/// How counts are determined (similar but specifically for "how many times")
enum CountExpr {
    Fixed(u64),
    Variable,
    CountOf(Selector),
}

/// What objects an effect looks at or iterates over
enum Selector {
    ControlledCreatures,
    CreaturesInGraveyard(PlayerRef),
    PermanentsMatching(PermanentFilter),
    CardsInHand(PlayerRef),
    CardsInGraveyard(PlayerRef),
    // extensible with filters
}

/// Conditions for Conditional effects
enum Condition {
    ControlPermanent(PermanentFilter),
    LifeAtLeast(AmountExpr),
    LifeAtMost(AmountExpr),
    OpponentControlsPermanent(PermanentFilter),
    CardInGraveyard(CardFilter),
    SpellWasKicked,
    ModeChosen(usize),
    SourceOnBattlefield,
    // extensible
}

/// Duration for continuous effects
enum Duration {
    UntilEndOfTurn,
    UntilYourNextTurn,
    WhileSourceOnBattlefield,  // static ability duration
    WhileEnchanted,
    WhileEquipped,
    Indefinite,
}
```

---

## What to Implement Per Phase

### Phase 2 — Stack, Casting, Spell Resolution
**Cards unblocked:** Lightning Bolt, Giant Growth

**Primitives needed:**
- `DealDamage(Fixed)` with targeting
- `ModifyPowerToughness(Fixed, Fixed)` with `Duration::UntilEndOfTurn`
- `Atom`, `Sequence` combinators
- `TargetSpec` basics (target creature, target player, "any target")
- `ApplyContinuous` for Giant Growth's +3/+3 until end of turn
- `CounterSpell` (for Counterspell itself)

**Infrastructure:**
- Effect resolver that walks the `Effect` tree and dispatches on `Primitive`
- Continuous effect registry in [GameState](cci:2://file:///c:/Users/maier/Desktop/MTG%20Simulator/mtgsim_v2/mtgsim/src/state/game_state.rs:17:0-52:1) with timestamp + duration
- Basic targeting validation

**Scope: ~8 primitives, 3 combinators**

### Phase 3 — Creatures, Combat, SBAs
**Cards unblocked:** Grizzly Bears, vanilla creatures

**Primitives needed:**
- `Destroy`, `Sacrifice`
- `GainLife`, `LoseLife`
- `Fight`
- `CreateToken(TokenDef, Fixed)` — for simple token producers

**Infrastructure:**
- Combat damage pipeline (uses `DealDamage` internally)
- SBA integration with the effect system
- Death triggers framework (zone-change triggers)

**Scope: +5 primitives**

### Phase 4 — Keywords
**Cards unblocked:** Most common creatures

**No new primitives** — keywords are mostly static abilities that modify combat rules, not effects. But:
- `AddAbility` / `RemoveAbility` needed for cards that grant keywords
- `Tap` / `Untap` as effect primitives (for cards like "tap target creature")
- `Conditional` combinator for "if this creature has flying..." type checks

**Scope: +3 primitives, +1 combinator**

### Phase 5 — Continuous Effects + Layer System
**Cards unblocked:** Anthem effects, lord creatures

**Primitives needed:**
- `SetPowerToughness` (layer 7b — e.g., "becomes a 0/1")
- `ChangeColor`, `ChangeType` (layers 4–5)
- `GainControl` (layer 2)

**Infrastructure (heavy):**
- Full 7-layer system (rule 613) using timestamps from [BattlefieldEntity](cci:2://file:///c:/Users/maier/Desktop/MTG%20Simulator/mtgsim_v2/mtgsim/src/state/battlefield.rs:8:0-34:1)
- Dependency detection (rule 613.8)
- Characteristic recalculation engine

**Scope: +4 primitives, but the layer engine is the real work**

### Phase 6 — Triggered Abilities, Replacement Effects
**Cards unblocked:** Most remaining cards

**Primitives needed:**
- `Exile`, `ReturnToHand`, `ReturnToBattlefield`, `ShuffleIntoLibrary`
- `Discard`, `Mill`, `Scry`, `Surveil`, `Search`, `Reveal`
- `AddCounters`, `RemoveCounters`, `Proliferate`
- `Explore`, `Connive`, `Amass`, `Transform`

**Combinators needed:**
- `Optional` ("you may...")
- `Modal` ("choose one")
- `ForEach`, `Repeat`
- `CreateDelayedTrigger`
- `ApplyReplacement`, `ApplyPrevention`
- `Custom` escape hatch

**Infrastructure:**
- Triggered ability engine (rule 603)
- Replacement effect engine (rule 614)
- Prevention effect engine (rule 615)
- Delayed trigger tracking

**Scope: +15 primitives, +6 combinators — this is the "big phase"**

---

## Summary Table

| Phase | Primitives | Combinators | Key Infrastructure |
|-------|-----------|-------------|-------------------|
| 2 | ~8 | 3 | Effect resolver, targeting, continuous registry |
| 3 | +5 | — | Combat pipeline, death triggers |
| 4 | +3 | +1 | Keyword interaction with effects |
| 5 | +4 | — | Layer system (613), dependency |
| 6 | +15 | +6 | Triggers (603), replacement (614), prevention (615) |
| **Total** | **~35** | **~10** | |

The `Custom` escape hatch grows organically — you won't need it much until you're implementing specific weird cards beyond these phases.

---

The key takeaway: **you don't need to build all ~35 primitives upfront.** Phase 2 needs only ~8 primitives and 3 combinators to be functional. Each subsequent phase adds incrementally. The types can be defined now (as an enum with variants), but the resolver match arms only need to exist when their phase arrives.

Want me to start implementing the Phase 2 foundation — the `Primitive`, `Effect`, `TargetSpec` types and a skeleton resolver — or would you like to refine the design further first?