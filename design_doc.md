# MTG Simulator — Design Document

> Last updated: 2026-03-29 (post-Phase 3, rev 3)
Project Goal: The ultimate goal for this project is a rules engine that is fast, 
correct, extensible, and managable, that a GUI could lay on top of for two humans 
to play over a network, or in a CLI/API where a bot is playing itself/another bot 
in dozens of parallel games.

This document is the single source of truth for the simulator's architecture,
current status, and upcoming work. Update it as decisions are made.

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Current Status (Post-Phase 2)](#2-current-status)
3. [Pre-Phase 3 Work Items](#3-pre-phase-3-work-items)
4. [Phase 3: Creatures & Combat](#4-phase-3)
5. [Phase 4: Keywords](#5-phase-4)
6. [Phase 5: Continuous Effects & Layers](#6-phase-5)
7. [Phase 6: Triggered & Replacement Effects](#7-phase-6)
8. [Excluded Cards](#8-excluded-cards)
9. [Design Decisions Log](#9-design-decisions-log)

---

## 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│  Game — lifecycle, setup, config, DecisionProvider dispatch  │
├─────────────────────────────────────────────────────────────┤
│  ui/decision.rs — DecisionProvider trait                     │
│  (CLI, AI, Scripted, Network implementations)               │
├─────────────────────────────────────────────────────────────┤
│  engine/ — Rules engine (reads + mutates GameState)         │
│  ┌──────────┬──────────┬────────────┬─────────────────────┐ │
│  │ cast.rs  │ stack.rs │priority.rs │ targeting.rs        │ │
│  │ turns.rs │ zones.rs │ resolve.rs │ costs.rs            │ │
│  │ sba.rs   │ mana.rs  │ combat/   │ (future: layers.rs) │ │
│  └──────────┴──────────┴────────────┴─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  state/ — Pure data (GameState, GameConfig, PlayerState)    │
│  objects/ — GameObject, CardData, BattlefieldEntity          │
│  types/ — Enums and value types (no logic)                  │
│  events/ — EventLog for game history                        │
│  cards/ — Card definitions (data only, via CardRegistry)    │
└─────────────────────────────────────────────────────────────┘
```

**Key architectural principles:**

- Central `HashMap<ObjectId, GameObject>` store; zones reference by ID
- Zone transitions go through `move_object()` (single chokepoint)
- Engine never does I/O; all player choices go through `DecisionProvider`
- `Game` struct owns lifecycle and `DecisionProvider` dispatch; engine methods are stateless transforms on `GameState`
- Card definitions are pure data (`CardData` + `AbilityDef` + `Effect` tree)
- `Effect` is a combinator tree: `Atom(Primitive, TargetSpec) | Sequence | Conditional | Modal | ...`

---

## 2. Current Status (Post-Phase 3)

**Test count:** 162 (128 unit + 33 integration + 1 doc-test), zero warnings.

### What's implemented

| Area               | Status      | Key files                                                                                                                  |
| ------------------ | ----------- | -------------------------------------------------------------------------------------------------------------------------- |
| Types & IDs        | ✅ Done      | `types/` (ids, mana, zones, colors, card_types, keywords, effects)                                                         |
| Game objects       | ✅ Done      | `objects/card_data.rs`, `objects/object.rs`                                                                                |
| Game state         | ✅ Done      | `state/game_state.rs`, `state/player.rs`, `state/battlefield.rs`                                                           |
| Game config        | ✅ Done      | `state/game_config.rs` — `GameConfig` (starting life, hand size, mulligan rule, deck limits) + `standard()`/`limited()`/`test()` presets |
| Game lifecycle     | ✅ Done      | `state/game.rs` — `Game` struct (owns `GameState` + `GameConfig` + `GameResult`), `setup()`, `run_turn()`, `run()`, `check_game_over()` |
| Zone transitions   | ✅ Done      | `engine/zones.rs`                                                                                                          |
| Turn structure     | ✅ Done      | `engine/turns.rs` (all phases/steps, untap, draw with first-player skip, cleanup damage removal)                           |
| Mana types         | ✅ Done      | `types/mana.rs` — `ManaSymbol` enum covers Colored, Generic, Colorless, Hybrid, MonoHybrid, Phyrexian, HybridPhyrexian, Snow, X |
| Mana payment       | ⚠️ Partial  | `types/mana.rs` (`can_pay`/`pay`) + `engine/mana.rs` — only Colored, Generic, Colorless symbols are payable; Hybrid/Phyrexian/X/Snow bail with errors. Full payment requires `DecisionProvider` choices (e.g. Phyrexian = color or 2 life?) |
| Cost payment       | ✅ Done      | `engine/costs.rs` — `can_pay_costs()` read-only pre-check + `pay_costs()`. Supports Tap, Untap, Mana, PayLife, SacrificeSelf. Future variants (Sacrifice, Discard, ExileFromGraveyard, RemoveCounters, AddCounters) return stub errors. `CostRestriction` framework designed for Phase 5. |
| Casting spells     | ✅ Done      | `engine/cast.rs` (rule 601.2, timing checks, sorcery/instant, `can_pay_costs` pre-check with rollback on failure)          |
| Stack & resolution | ✅ Done      | `engine/stack.rs` (rule 608, pop-first, fizzle handling)                                                                   |
| Priority system    | ✅ Done      | `engine/priority.rs` (rule 117, SBA loop, full priority round)                                                             |
| Targeting          | ✅ Done      | `engine/targeting.rs` (Creature, Player, Any, Permanent, Spell)                                                            |
| Effect resolver    | ⚠️ Partial  | `engine/resolve.rs` — DealDamage, DrawCards, GainLife, LoseLife, ProduceMana, CounterSpell, CounterAbility, Destroy, Untap. ~20 primitives still return stub errors. |
| SBAs               | ✅ Done      | `engine/sba.rs` — lethal damage, zero toughness, player loss flags (704.5a life ≤ 0, 704.5b empty library draw). Routes through EventLog, no println. |
| Game result        | ✅ Done      | `GameResult` enum (Winner/Draw). `Game::check_game_over()` reads `player_lost` flags set by SBAs.                          |
| Discard to hand    | ✅ Done      | `Game::run_turn()` handles cleanup step discard via `DecisionProvider::choose_discard`                                      |
| First-player skip  | ✅ Done      | `skip_next_draw` flag on `GameState`, set by `Game::new()` from `GameConfig::first_player_draws`, consumed in `process_draw_step` |
| Card registry      | ✅ Done      | `cards/registry.rs` + `cards/basic_lands.rs` + `cards/alpha.rs` + `cards/creatures.rs`                                     |
| Events             | ✅ Done      | `events/event.rs` (GameEvent enum, EventLog)                                                                               |
| DecisionProvider   | ✅ Done      | `ui/decision.rs` (trait + Passive + Scripted + auto_allocate_generic + choose_damage_order)                                |
| Combat validation   | ✅ Done      | `engine/combat/validation.rs` — validate_attackers, validate_blockers, AttackConstraints/BlockConstraints skeletons, effective characteristic helpers |
| Combat resolution   | ✅ Done      | `engine/combat/resolution.rs` — assign_combat_damage (read-only), apply_combat_damage (routes through GameAction::DealDamage) |
| Combat steps        | ✅ Done      | `engine/combat/steps.rs` — process_declare_attackers, process_declare_blockers, process_combat_damage (wired into Game::run_turn) |

### Cards implemented (13)

- **Basic lands:** Plains, Island, Swamp, Mountain, Forest
- **Alpha spells:** Lightning Bolt, Ancestral Recall, Counterspell
- **Other spells:** Burst of Energy (Urza's Destiny), Volcanic Upheaval (BFZ)
- **Vanilla creatures:** Grizzly Bears (2/2, {1}{G}), Hill Giant (3/3, {3}{R}), Savannah Lions (2/1, {W})

### Known gaps / TODOs in existing code

- `resolve.rs`: ~20 primitives still return stub errors
- `types/mana.rs`: `can_pay`/`pay` don't handle Hybrid, Phyrexian, MonoHybrid, Snow, X symbols
- `costs.rs`: Cost variants Sacrifice, Discard, ExileFromGraveyard, RemoveCounters, AddCounters return stub errors
- `game.rs`: Mulligan handling stubbed (players always keep opening hand)

---

## 3. Pre-Phase 3 Work Items ✅ COMPLETED

All items completed 2026-03-29. These fixes and features landed before Phase 3 (creatures & combat). They address correctness issues and lay groundwork.

### 3.1 Game + GameConfig Struct (HIGH) ✅

**Problem:** No concept of game lifecycle — setup, turn loop, termination, or format-specific configuration. Decision-requiring moments (discard, mulligans) have nowhere to live because the engine doesn't hold a `DecisionProvider`.

**Solution:** Introduce `GameConfig` (pure data) and `Game` (lifecycle wrapper that owns `DecisionProvider` dispatch).

**Design:**

```rust
// In state/game_config.rs

/// Configuration that varies by format. Pure data, no behavior.
pub struct GameConfig {
    pub starting_life: i64,
    pub starting_hand_size: usize,
    pub max_hand_size: i32,
    pub first_player_draws: bool,       // false in standard 2-player
    pub mulligan_rule: MulliganRule,
    pub deck_limits: DeckLimits,
}

pub enum MulliganRule {
    London,    // current official rule
    Paris,     // older rule
    None,      // no mulligans (for testing)
}

pub struct DeckLimits {
    pub min_deck_size: usize,          // 60 standard, 40 limited, 99 commander
    pub max_deck_size: Option<usize>,  // None = unlimited
    pub max_copies: u32,               // 4 for most formats
    pub sideboard_size: Option<usize>, // 15 for constructed
}

impl GameConfig {
    /// Standard/Modern/Pioneer defaults
    pub fn standard() -> Self { ... }
    /// Limited (draft/sealed) defaults
    pub fn limited() -> Self { ... }
}
```

```rust
// In state/game.rs

pub struct Game {
    pub state: GameState,
    pub config: GameConfig,
    pub result: Option<GameResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameResult {
    Winner(PlayerId),
    Draw,
    // Future: multiplayer eliminations
}

impl Game {
    /// Create a new game from config and decklists.
    pub fn new(config: GameConfig, decklists: Vec<Decklist>) -> Result<Self, String>;

    /// Perform game setup: validate decks, shuffle, draw opening hands,
    /// handle mulligans. This is where DecisionProvider is used for
    /// mulligan decisions.
    pub fn setup(&mut self, decisions: &dyn DecisionProvider) -> Result<(), String>;

    /// Run a single full turn. Handles:
    /// - Phase/step progression via advance_turn
    /// - Priority rounds (delegates to engine)
    /// - Cleanup step discard (calls DecisionProvider::choose_discard)
    /// - SBA checks and game-over detection
    pub fn run_turn(&mut self, decisions: &dyn DecisionProvider) -> Result<(), String>;

    /// Run the complete game until a result is determined.
    pub fn run(&mut self, decisions: &dyn DecisionProvider) -> Result<GameResult, String>;

    pub fn is_over(&self) -> bool { self.result.is_some() }
}
```

**`Game` is the owner of all decision-requiring interactions.** The engine methods on `GameState` remain pure state transforms. `Game::run_turn` is the only place that calls `choose_discard`, `choose_priority_action`, etc. This means `advance_turn` stays clean — no `DecisionProvider` threading.

**Format trait (future):** When we need Commander/Brawl, `Game` gains a `format: Box<dyn Format>` field. `Format` trait provides `config()`, `validate_decklist()`, `setup_game()`, `check_win_condition()`. Standard/Modern/Pioneer/Limited share a default impl. Commander overrides for command zone, commander damage, color identity, singleton rule. The `GameConfig` struct becomes a field of the `Format` implementor. `Game::setup` and `Game::run_turn` logic moves into default methods on the trait. The struct fields and their types don't change — only where the behavior lives.

**Files touched:** `state/game_config.rs` (new), `state/game.rs` (new), `state/mod.rs`
**Tests:** Unit tests for `GameConfig::standard()`, integration test for full game lifecycle

### 3.2 Game Result & Loss Handling (HIGH) ✅

**Problem:** SBAs detect loss conditions but just `println!`. No way to end the game.

**Solution:** Add `GameResult` to `Game` (not `GameState` — the game lifecycle owns win/loss state). SBAs set flags; `Game::run_turn` checks them and terminates.

**Design:**

`GameState` gains loss-condition flags (SBAs set these):

```rust
pub player_lost: Vec<bool>,  // indexed by PlayerId
```

In `sba.rs`, when a player loses:

```rust
self.player_lost[player_id] = true;
```

In `Game::run_turn`, after each SBA cycle:

```rust
if let Some(result) = self.check_game_over() {
    self.result = Some(result);
    return Ok(());
}
```

`Game::check_game_over` examines the `player_lost` flags and determines the result. This keeps `GameState` focused on state and `Game` on lifecycle logic.

**Files touched:** `state/game_state.rs` (loss flags), `state/game.rs` (`check_game_over`), `engine/sba.rs` (set flags instead of println)
**Tests:** Integration test: bolt a player to 0 life, verify game ends

### 3.3 Cost Validation & Rollback (HIGH) ✅

**Problem:** `cast_spell` moves the card to the stack, then pays costs. If payment fails, the card is stranded on the stack.

**Solution:** Add `can_pay_costs()` — a read-only pre-check that validates all costs can be paid before any mutation happens. On failure after stack placement, roll the card back to hand.

**Design:**

```rust
// In engine/costs.rs
impl GameState {
    /// Read-only check: can all costs be paid right now?
    /// Checks both resource availability AND cost restrictions.
    pub fn can_pay_costs(
        &self,
        costs: &[Cost],
        player_id: PlayerId,
        source_id: ObjectId,
    ) -> Result<(), String> {
        for cost in costs {
            self.check_cost_resource(cost, player_id, source_id)?;
            self.check_cost_restrictions(cost, player_id, source_id)?;
        }
        Ok(())
    }
}
```

**Two layers of validation:**

1. **Resource check** (`check_cost_resource`): Do you have the stuff?
   - `Tap` → `!entry.tapped && !(summoning_sick && creature)`
   - `Mana(mc)` → `mana_pool.can_pay(&mc)`
   - `PayLife(n)` → `life_total >= n`
   - `SacrificeSelf` → `battlefield.contains_key(source_id)`

2. **Restriction check** (`check_cost_restrictions`): Does the game state *allow* this cost?
   This queries a `Vec<CostRestriction>` on `GameState`, populated by continuous effects (Phase 5):

```rust
/// A restriction on what costs can be paid, applied by continuous effects.
pub enum CostRestriction {
    /// Cannot place counters on permanents matching filter (Solemnity)
    CannotPlaceCounters(PermanentFilter),
    /// Cannot pay life (Platinum Emperion, Angel of Jubilation for black costs)
    CannotPayLife,
    /// Cannot sacrifice permanents matching filter (Sigarda, Host of Herons)
    CannotSacrifice(PermanentFilter),
    /// Cannot activate abilities of permanents matching filter (Null Rod, Pithing Needle)
    CannotActivate(PermanentFilter),
    /// Cannot discard cards (Stabilizer — for cycling costs)
    CannotDiscard,
}
```

**We don't implement `CostRestriction` now.** We implement `can_pay_costs` with just the resource checks today. `check_cost_restrictions` starts as a no-op. When Phase 5 lands continuous effects, we populate the restriction list and add match arms. The function signature and call sites don't change.

**Example: Solemnity + Soul Immolation.** Soul Immolation has an additional cost of `AddCounters(MinusOneMinusOne, X)`. Solemnity creates a `CostRestriction::CannotPlaceCounters(PermanentFilter::All)`. When `can_pay_costs` checks the `AddCounters` cost, `check_cost_restrictions` finds the matching restriction and returns `Err`. The player can't even begin casting. This works *without* `can_pay_costs` needing to know about Solemnity specifically — it just queries the restriction list.

**Cast spell flow:**

1. `check_cast_legality` ✓
2. Move to stack ✓
3. Choose targets ✓
4. **`can_pay_costs` → if Err, move card back to hand, return Err**
5. `pay_costs` (guaranteed to succeed)

**Expand `Cost` enum** for future needs:

```rust
pub enum Cost {
    Tap,
    Untap,                                  // Devoted Druid
    Mana(ManaCost),
    PayLife(u64),
    SacrificeSelf,
    Sacrifice(PermanentFilter, u32),        // "Sacrifice a creature"
    Discard(CardFilter, u32),               // "Discard a card"
    ExileFromGraveyard(CardFilter, u32),    // "Exile a creature card from graveyard"
    RemoveCounters(CounterType, u32),       // "Remove a +1/+1 counter"
    AddCounters(CounterType, u32),          // Soul Immolation's blight cost
}
```

Only implement `can_pay` for variants we actually use right now; others return `Err("not yet implemented")`.

**Files touched:** `objects/card_data.rs` (Cost enum), `engine/costs.rs` (can_pay_costs, check_cost_resource, check_cost_restrictions), `engine/cast.rs` (rollback logic), `state/game_state.rs` (cost_restrictions: Vec)
**Tests:** Unit tests for each can_pay variant, integration test for failed-payment rollback

### 3.4 Discard to Hand Size (MEDIUM) ✅

**Problem:** Cleanup step (rule 514.1) requires discarding to max hand size. Currently stubbed.

**Solution:** `Game::run_turn` handles discard during the cleanup step by calling `DecisionProvider::choose_discard`. The engine method `on_step_begin(Cleanup)` still handles mechanical cleanup (damage removal, "until end of turn" expiry), but the discard interaction lives in `Game`.

**Design:**

```rust
// In Game::run_turn, when entering cleanup step:
let active = self.state.active_player;
let max = self.state.players[active].max_hand_size as usize;
while self.state.players[active].hand.len() > max {
    let card_id = decisions.choose_discard(&self.state, active)
        .ok_or("Player must choose a card to discard")?;
    self.state.move_object(card_id, Zone::Graveyard)?;
}
```

**Files touched:** `state/game.rs` (discard logic in `run_turn`)
**Tests:** Integration test: player with 8+ cards discards to 7

### 3.5 First-Player Draw Skip (LOW) ✅

**Problem:** Rule 103.8a — the starting player skips their first draw step.

**Solution:** A `skip_next_draw: bool` flag on `GameState`, set during `Game::setup` based on `GameConfig::first_player_draws`. In `process_draw_step`, check and clear the flag.

**Note on future "skip draw" effects:** This flag is *only* for the one-time game-setup rule 103.8a. In-game "skip your next draw" effects (e.g. Omen Machine, Maralen of the Mornsong) are **replacement effects** (Phase 6). They would use the replacement effect system, not additional boolean flags. The replacement effect framework naturally handles stacking multiple skip effects, "if you would draw, instead..." chains, etc.

**Files touched:** `state/game_state.rs` (flag), `engine/turns.rs` (check in `process_draw_step`), `state/game.rs` (set flag in `setup`)
**Tests:** Unit test: first turn draw is skipped, second turn draws normally

### 3.6 Minor Fixes (LOW) ✅

- **CounterSpell cleanup:** ✅ Remove the `StackEntry` for countered spells in `resolve_primitive`
- **Event consistency:** ✅ Emit `ZoneChange` events from `CounterSpell`/`CounterAbility`
- **SBA println removal:** ✅ Route SBA messages through `EventLog` instead of `println!`
- **Stack→Battlefield workaround:** Deferred — the temporary re-push hack in `stack.rs` works correctly and will be revisited if it causes issues in Phase 3

---

## 4. Phase 3: Creatures & Combat ✅ COMPLETED

All items completed 2026-03-29. Creatures resolve to the battlefield, full combat system (declare attackers, declare blockers, combat damage) wired into the turn loop, SBAs handle lethal damage.

### 3a. Permanent spell resolution fix ✅

Fixed the "re-push" hack in `stack.rs` where permanent spells were temporarily pushed back onto the stack for `move_object`. Now uses manual zone bookkeeping (same pattern as instant/sorcery resolution): set zone, call `init_zone_state`, emit `ZoneChange` and `PermanentEnteredBattlefield` events. `init_zone_state` made `pub(crate)` for this.

### 3b-3d. Combat validation (`engine/combat/validation.rs`) ✅

**Constraint skeletons:** `AttackConstraints` (restrictions + requirements) and `BlockConstraints` (restrictions + requirements + per-creature `blocking_limits` for multi-block). Both have `::none()` constructors for Phase 3. Phase 4/5 will populate from keywords and continuous effects.

**Effective characteristic helpers** on `GameState`: `is_creature()`, `can_attack()`, `get_effective_power()`, `get_effective_toughness()`. Phase 3 reads `card_data` directly; Phase 5 will swap in layer-system-aware lookups (single-point change).

**`validate_attackers`** (rule 508.1): per-creature checks (on battlefield, is creature, correct controller, untapped, not summoning-sick, valid attack target) + set-level constraint checks.

**`validate_blockers`** (rule 509.1): per-creature checks (on battlefield, is creature, correct controller, untapped, attacker is attacking this player) + per-creature block count vs `max_blocks_for()` + set-level constraint checks.

### 3e. Combat damage (`engine/combat/resolution.rs`) ✅

Two-phase design to avoid borrowing issues:
1. **`assign_combat_damage(&GameState, ...)`** — read-only free function. Computes `Vec<CombatDamageAssignment>` from battlefield state. Handles unblocked attackers (→ player), single blocker (→ all damage), multiple blockers (→ ordered by `damage_orders`, lethal-first), blocked-but-no-blockers (→ no damage), zero-power (→ no damage). `first_strike_only` parameter stubs for Phase 4.
2. **`GameState::apply_combat_damage(assignments)`** — routes each through `execute_action(GameAction::DealDamage { is_combat: true })` so Phase 6 replacement effects automatically intercept.

### 3f. Turn structure wiring ✅

`Game::run_turn` now calls combat turn-based actions before priority rounds:
- `DeclareAttackers`: `process_declare_attackers` (taps attackers, sets `AttackingInfo`)
- `DeclareBlockers`: `process_declare_blockers` (sets `BlockingInfo`, marks attackers as blocked, requests damage orders for multi-blocked attackers)
- `FirstStrikeDamage`: `process_combat_damage(first_strike_only=true)` — Phase 3 no-op
- `CombatDamage`: `process_combat_damage(first_strike_only=false)`

`GameState` gains `damage_orders: HashMap<ObjectId, Vec<ObjectId>>` for multi-blocker ordering. Cleared at end of combat phase alongside `attacks_declared`, `blockers_declared`, and per-permanent combat state.

### 3g. Cards implemented ✅

`cards/creatures.rs`: Grizzly Bears ({1}{G} 2/2), Hill Giant ({3}{R} 3/3), Savannah Lions ({W} 2/1). All registered in `CardRegistry`.

### 3h. Integration tests (8 tests) ✅

- Registry has Phase 3 creatures
- Unblocked attacker deals damage to defending player
- Blocked creatures trade (both die from lethal damage via SBAs)
- Bigger creature survives combat (Hill Giant vs Bears)
- No attackers = no combat damage
- Summoning-sick creature cannot attack (validation error)
- Combat damage kills player (game over via SBAs)
- Combat state cleared after combat phase

### Key files changed/created

| File | Change |
| ---- | ------ |
| `engine/combat/mod.rs` | New — module registration |
| `engine/combat/validation.rs` | New — CombatError, AttackConstraints, BlockConstraints, validate_attackers, validate_blockers, effective characteristic helpers (760 lines) |
| `engine/combat/resolution.rs` | New — CombatDamageAssignment, assign_combat_damage, apply_combat_damage (400 lines) |
| `engine/combat/steps.rs` | New — process_declare_attackers, process_declare_blockers, process_combat_damage (190 lines) |
| `engine/stack.rs` | Fixed permanent spell resolution (no re-push) |
| `engine/zones.rs` | `init_zone_state` made `pub(crate)` |
| `engine/turns.rs` | `damage_orders.clear()` in combat phase end |
| `state/game_state.rs` | Added `damage_orders` field |
| `state/game.rs` | Combat turn-based actions in `run_turn` |
| `ui/decision.rs` | Added `choose_damage_order` to trait + implementations |
| `cards/creatures.rs` | New — Grizzly Bears, Hill Giant, Savannah Lions |
| `cards/registry.rs` | Registered 3 creatures |
| `tests/phase3_integration_test.rs` | New — 8 integration tests |

---

## 5. Phase 4: Keywords

**Goal:** Keyword abilities that modify combat and other game behaviors.

### Keywords to implement

- **Flying** (rule 702.9): Can only be blocked by creatures with flying or reach
- **First Strike** (rule 702.7): Deals combat damage in first strike damage step
- **Haste** (rule 702.10): No summoning sickness
- **Trample** (rule 702.19): Excess combat damage dealt to defending player
- **Reach** (rule 702.17): Can block flyers
- **Vigilance** (rule 702.20): Doesn't tap to attack
- **Lifelink** (rule 702.15): Damage dealt also gains life for controller
- **Deathtouch** (rule 702.2): Any damage is lethal

### Implementation approach

Most keywords modify existing engine logic at specific hook points:

- **Flying/Reach**: Adds `BlockConstraint` entries in combat validation — flyers can only be blocked by creatures with flying or reach
- **First Strike / Double Strike**: Extra combat damage step in `engine/turns.rs` + `engine/combat/resolution.rs`. Creatures with first strike deal damage in the first strike damage step; creatures with double strike deal damage in both steps.
- **Haste**: Skip summoning sickness check in `engine/costs.rs` (tap cost) and `engine/combat/validation.rs` (attack legality)
- **Trample**: In damage assignment, excess over lethal damage to blockers tramples to defending player
- **Vigilance**: Attacking creature doesn't tap
- **Lifelink**: After damage, controller gains life equal to damage dealt. Implemented as a post-damage-step hook.
- **Deathtouch**: In SBA check, any damage from a deathtouch source counts as lethal regardless of amount

The `KeywordAbility` enum already exists in `types/keywords.rs` with all these variants.

### Cards to implement

- Serra Angel (4/4, Flying, Vigilance)
- Llanowar Elves (1/1, {T}: Add {G} — already have mana ability support)
- Goblin Guide (2/2, Haste)
- Typhoid Rats (1/1, Deathtouch)

---

## 6. Phase 5: Continuous Effects & Layer System

**Goal:** Effects that modify game state continuously (e.g. "all creatures get +1/+1", "+3/+3 until end of turn") and restrictions that prevent actions (Solemnity, Null Rod).

### The layer system (rule 613)

Effects are applied in a strict order:

1. Copy effects
2. Control-changing effects
3. Text-changing effects
4. Type-changing effects
5. Color-changing effects
6. Ability-adding/removing effects
7. Power/toughness effects (sublayers 7a-7e)

### Implementation plan

- `engine/layers.rs` — applies all continuous effects in layer order to produce "computed characteristics"
- `objects/characteristics.rs` — the computed output (effective P/T, types, abilities, colors)
- Duration tracking: `UntilEndOfTurn`, `WhileSourceOnBattlefield`, `Permanent`
- Effects registered on `GameState` as a `Vec<ContinuousEffect>` with metadata (source, duration, layer, timestamp)

### Cost restriction system

This phase also activates the `CostRestriction` framework designed in Section 3.3. Continuous effects that prevent costs (Solemnity, Null Rod, Sigarda, etc.) populate `GameState::cost_restrictions`, which `can_pay_costs` already queries.

Similarly, `AttackConstraint` and `BlockConstraint` (Section 4) are populated by continuous effects from this phase onward.

### Cards to implement

- Giant Growth (+3/+3 until end of turn — layer 7c)
- Glorious Anthem (Creatures you control get +1/+1 — layer 7a, static ability)
- Honor of the Pure (White creatures you control get +1/+1 — filtered static)

---

## 7. Phase 6: Triggered & Replacement Effects

**Goal:** "When X happens, do Y" and "If X would happen, instead Y".

### Triggered abilities

- Event-driven: subscribe to `GameEvent` types
- Placed on stack when triggered, controlled by source's controller
- `perform_sba_and_triggers` in `engine/priority.rs` is already stubbed for this

### Replacement effects

- Checked before the original event occurs
- Shield counters, damage prevention, "enters the battlefield tapped"
- **"Skip your draw step" effects** (Omen Machine, Maralen of the Mornsong) are replacement effects, not boolean flags. The framework handles stacking multiple replacements and "if you would draw, instead..." chains.

### Cards to implement

- Soul Warden ("Whenever a creature enters the battlefield, you gain 1 life")
- Thalia, Guardian of Thraben (cost increase — ties into cost modification pipeline from `engine/cast.rs`)
- Omen Machine (replacement effect for draw + triggered ability for reveal-and-cast)

---

## 8. Excluded Cards

These cards are **explicitly out of scope** due to rules ambiguities or engine-breaking interactions. They are non-competitive and not worth the architectural complexity they would require.

| Card | Reason |
| ---- | ------ |
| **Season of the Witch** | "Destroy each creature that couldn't attack this turn" — "couldn't attack" is still poorly defined in the comprehensive rules. Would require tracking hypothetical attack legality for every creature every turn. |
| **Panglacial Wurm** | "While you're searching your library, you may cast this card from your library." Casting from a library mid-search breaks stack assumptions, zone transition invariants, and mana ability resolution. Requires the ability to interrupt a search effect with a full cast sequence. |
| **Selvala, Explorer Returned** | Mana ability that reveals hidden information and produces an undefined amount of mana. You can begin casting a spell, activate Selvala to pay for it, discover you don't have enough mana, and have no way to cleanly rewind the game state (revealed cards, gained life, etc.). Breaks the assumption that mana abilities are deterministic and side-effect-free. |

Additional cards may be added to this list as development progresses. The general criteria: if a card requires architectural changes that benefit only that card and a handful of similar effects, it's not worth supporting until/unless the simulator's scope explicitly expands to include it.

---

## 9. Design Decisions Log

| Date       | Decision                                          | Rationale                                                                                          |
| ---------- | ------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| 2026-03-27 | Central `HashMap<ObjectId, GameObject>` store     | Zones reference by ID; one source of truth                                                         |
| 2026-03-27 | `DecisionProvider` trait for all player choices   | Engine stays pure, testable, UI-independent                                                        |
| 2026-03-28 | `Arc<CardData>` shared across instances           | Avoids cloning ~1KB card data per object                                                           |
| 2026-03-28 | `Vec<ManaSymbol>` for ManaCost                    | Supports hybrid/phyrexian/X/snow natively                                                          |
| 2026-03-28 | `Effect` combinator tree (Atom/Sequence/...)      | Composable, serializable, covers 95%+ of cards                                                     |
| 2026-03-29 | Stack pop-first during resolution                 | Resolving spell should NOT be visible on stack                                                     |
| 2026-03-29 | `auto_allocate_generic` lives in `ui/decision.rs` | Engine requires manual selection; convenience is layered on top                                    |
| 2026-03-29 | `can_pay_costs` = resource check + restriction check | Two-layer validation: "do you have it?" then "does the game allow it?" Restriction layer starts as no-op, activated by Phase 5 continuous effects |
| 2026-03-29 | `GameConfig` struct now, `Format` trait later     | Covers 90% of formats with pure data; `Format` trait only needed when Commander/Brawl require behavioral differences. `Game` struct already structured for easy migration. |
| 2026-03-29 | `Game` owns `DecisionProvider` dispatch           | Engine methods stay as pure state transforms. `Game::run_turn` is the only place that calls decision methods. No threading `DecisionProvider` through `advance_turn`. |
| 2026-03-29 | Combat split: validation + resolution             | Once attackers/blockers are locked in, damage is deterministic. Validation is the complex part (constraints, forced attacks, evasion). Extensible via `AttackConstraint` / `BlockConstraint` lists populated by continuous effects. |
| 2026-03-29 | `skip_next_draw` flag only for rule 103.8a        | In-game "skip draw" effects are replacement effects (Phase 6), not boolean flags. The flag is a one-time game-setup mechanism. |
| 2026-03-29 | Explicit excluded cards list                      | Season of the Witch, Panglacial Wurm, Selvala — non-competitive cards that require disproportionate architectural changes. |
| 2026-03-29 | Two-phase combat damage (compute then apply)      | `assign_combat_damage` is a read-only free function; `apply_combat_damage` mutates. Avoids borrow checker issues from reading battlefield while writing damage. |
| 2026-03-29 | Effective characteristic helpers on GameState     | `is_creature()`, `can_attack()`, `get_effective_power/toughness()` — combat code calls these instead of reading `card_data` directly, so Phase 5 layer-system swap is a single-point change. |
| 2026-03-29 | Permanent spell resolution: no re-push            | Fixed Phase 2 hack. Manual zone bookkeeping for permanent spells resolving to battlefield, consistent with instant/sorcery path. |

---

## Implementation Order

```
Pre-Phase 3 (current):
  3.1  Game + GameConfig struct (lifecycle, setup, config)
  3.2  GameResult + loss handling (flags in SBA, check in Game)
  3.3  can_pay_costs pre-check + Cost enum expansion + CostRestriction stub
  3.4  Discard to hand size (in Game::run_turn)
  3.5  First-player draw skip (flag + GameConfig)
  3.6  Minor fixes (CounterSpell cleanup, event consistency, SBA println)

Phase 3: Creatures & Combat ✅
  3a   Permanent spell resolution fix (no re-push) ✅
  3b   engine/combat/ module structure ✅
  3c   validate_attackers + AttackConstraints skeleton ✅
  3d   validate_blockers + BlockConstraints skeleton ✅
  3e   Combat damage assignment + resolution (two-phase) ✅
  3f   Wire combat into Game::run_turn + turn structure ✅
  3g   Cards: Grizzly Bears, Hill Giant, Savannah Lions ✅
  3h   Integration tests (8 tests) ✅
  3i   Design doc update ✅

Phase 4: Keywords
  4a   Flying / Reach (BlockConstraint entries)
  4b   First Strike / Double Strike (extra damage step)
  4c   Haste (summoning sickness bypass)
  4d   Trample (excess damage assignment)
  4e   Vigilance (no tap on attack)
  4f   Lifelink, Deathtouch (post-damage hook, SBA modification)
  4g   Cards: Serra Angel, Llanowar Elves, Goblin Guide, Typhoid Rats

Phase 5: Continuous Effects & Layers
  5a   ContinuousEffect struct + duration tracking
  5b   Layer application engine (rule 613)
  5c   Computed characteristics
  5d   "Until end of turn" effect cleanup
  5e   CostRestriction activation (populate from continuous effects)
  5f   AttackConstraint / BlockConstraint activation
  5g   Cards: Giant Growth, Glorious Anthem, Honor of the Pure

Phase 6: Triggered & Replacement Effects
  6a   Trigger registration + event matching
  6b   Triggered abilities on stack
  6c   Replacement effect framework (including "skip draw" effects)
  6d   Cards: Soul Warden, Thalia, Omen Machine
```
