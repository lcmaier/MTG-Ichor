# MTG Simulator — Design Document

> Last updated: 2026-03-30 (post-Phase 4.5, rev 5)
> Project Goal: The ultimate goal for this project is a rules engine that is fast, 
> correct, extensible, and managable, that a GUI could lay on top of for two humans 
> to play over a network, or in a CLI/API where a bot is playing itself/another bot 
> in dozens of parallel games.

This document is the single source of truth for the simulator's architecture,
current status, and upcoming work. Update it as decisions are made.

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Current Status (Post-Phase 4.5)](#2-current-status)
3. [Pre-Phase 3 Work Items](#3-pre-phase-3-work-items)
4. [Phase 3: Creatures & Combat](#4-phase-3)
5. [Phase 4: Keywords](#5-phase-4)
6. [Phase 4.5: Oracle Helpers & Decision Providers](#6-phase-45)
7. [Phase 5: Continuous Effects & Layers](#7-phase-5)
8. [Phase 6: Triggered Abilities](#8-phase-6)
9. [Phase 7: Replacement & Prevention Effects](#9-phase-7)
10. [Excluded Cards](#10-excluded-cards)
11. [Design Decisions Log](#11-design-decisions-log)

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

## 2. Current Status (Post-Phase 4.5)

**Test count:** 287 (238 unit + 48 integration + 1 doc-test), zero warnings. 500/500 fuzz games pass with zero errors/panics.

### What's implemented

| Area               | Status     | Key files                                                                                                                                                                                                                                                                                 |
| ------------------ | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Types & IDs        | ✅ Done     | `types/` (ids, mana, zones, colors, card_types, keywords, effects)                                                                                                                                                                                                                        |
| Game objects       | ✅ Done     | `objects/card_data.rs`, `objects/object.rs`                                                                                                                                                                                                                                               |
| Game state         | ✅ Done     | `state/game_state.rs`, `state/player.rs`, `state/battlefield.rs`                                                                                                                                                                                                                          |
| Game config        | ✅ Done     | `state/game_config.rs` — `GameConfig` (starting life, hand size, mulligan rule, deck limits) + `standard()`/`limited()`/`test()` presets                                                                                                                                                  |
| Game lifecycle     | ✅ Done     | `state/game.rs` — `Game` struct (owns `GameState` + `GameConfig` + `GameResult`), `setup()`, `run_turn()`, `run()`, `check_game_over()`                                                                                                                                                   |
| Zone transitions   | ✅ Done     | `engine/zones.rs`                                                                                                                                                                                                                                                                         |
| Turn structure     | ✅ Done     | `engine/turns.rs` (all phases/steps, untap, draw with first-player skip, cleanup damage removal)                                                                                                                                                                                          |
| Mana types         | ✅ Done     | `types/mana.rs` — `ManaSymbol` enum covers Colored, Generic, Colorless, Hybrid, MonoHybrid, Phyrexian, HybridPhyrexian, Snow, X                                                                                                                                                           |
| Mana payment       | ⚠️ Partial | `types/mana.rs` (`can_pay`/`pay`) + `engine/mana.rs` — only Colored, Generic, Colorless symbols are payable; Hybrid/Phyrexian/X/Snow bail with errors. Full payment requires `DecisionProvider` choices (e.g. Phyrexian = color or 2 life?)                                               |
| Cost payment       | ✅ Done     | `engine/costs.rs` — `can_pay_costs()` read-only pre-check + `pay_costs()`. Supports Tap, Untap, Mana, PayLife, SacrificeSelf. Future variants (Sacrifice, Discard, ExileFromGraveyard, RemoveCounters, AddCounters) return stub errors. `CostRestriction` framework designed for Phase 5. |
| Casting spells     | ✅ Done     | `engine/cast.rs` (rule 601.2, timing checks, sorcery/instant, `can_pay_costs` pre-check with rollback on failure)                                                                                                                                                                         |
| Stack & resolution | ✅ Done     | `engine/stack.rs` (rule 608, pop-first, fizzle handling)                                                                                                                                                                                                                                  |
| Priority system    | ✅ Done     | `engine/priority.rs` (rule 117, SBA loop, full priority round)                                                                                                                                                                                                                            |
| Targeting          | ✅ Done     | `engine/targeting.rs` (Creature, Player, Any, Permanent, Spell)                                                                                                                                                                                                                           |
| Effect resolver    | ⚠️ Partial | `engine/resolve.rs` — DealDamage, DrawCards, GainLife, LoseLife, ProduceMana, CounterSpell, CounterAbility, Destroy, Untap. ~20 primitives still return stub errors.                                                                                                                      |
| SBAs               | ✅ Done     | `engine/sba.rs` — lethal damage, zero toughness, player loss flags (704.5a life ≤ 0, 704.5b empty library draw). Routes through EventLog, no println.                                                                                                                                     |
| Game result        | ✅ Done     | `GameResult` enum (Winner/Draw). `Game::check_game_over()` reads `player_lost` flags set by SBAs.                                                                                                                                                                                         |
| Discard to hand    | ✅ Done     | `Game::run_turn()` handles cleanup step discard via `DecisionProvider::choose_discard`                                                                                                                                                                                                    |
| First-player skip  | ✅ Done     | `skip_first_draw` flag on `GameState`, set by `Game::new()` from `GameConfig::first_player_draws`, consumed in `process_draw_step`                                                                                                                                                         |
| Card registry      | ✅ Done     | `cards/registry.rs` + `cards/basic_lands.rs` + `cards/alpha.rs` + `cards/creatures.rs` + `cards/keyword_creatures.rs`                                                                                                                                                                     |
| Events             | ✅ Done     | `events/event.rs` (GameEvent enum, EventLog)                                                                                                                                                                                                                                              |
| DecisionProvider   | ✅ Done     | `ui/decision.rs` (trait + Passive + Scripted + Dispatch + auto_allocate_generic + shared helpers)                                                                                                                                                                                         |
| Oracle module      | ✅ Done     | `oracle/characteristics.rs` (has_keyword, is_creature, effective P/T), `oracle/legality.rs` (playable_lands, legal_attackers, legal_blockers), `oracle/board.rs`, `oracle/mana_helpers.rs` (find_mana_sources, castable_spells, activatable_abilities)                                     |
| CLI play           | ✅ Done     | `ui/cli.rs` (CliDecisionProvider — all 8 methods via stdin/stdout), `ui/display.rs` (text formatting for CLI/logs)                                                                                                                                                                       |
| Fuzz testing       | ✅ Done     | `ui/random.rs` (RandomDecisionProvider — all 8 methods, internal action queue), `bin/fuzz_games.rs` (N games of Random vs Random, --dump-events), `bin/cli_play.rs` (Human vs Random bot via DispatchDecisionProvider)                                                                   |
| Combat validation  | ✅ Done     | `engine/combat/validation.rs` — validate_attackers, validate_blockers, AttackConstraints/BlockConstraints skeletons, effective characteristic helpers                                                                                                                                     |
| Combat resolution  | ✅ Done     | `engine/combat/resolution.rs` — assign_combat_damage (read-only), apply_combat_damage (routes through GameAction::DealDamage)                                                                                                                                                             |
| Combat steps       | ✅ Done     | `engine/combat/steps.rs` — process_declare_attackers, process_declare_blockers, process_combat_damage (wired into Game::run_turn)                                                                                                                                                         |

### Cards implemented (24, unchanged from Phase 4)

- **Basic lands:** Plains, Island, Swamp, Mountain, Forest
- **Alpha spells:** Lightning Bolt, Ancestral Recall, Counterspell
- **Other spells:** Burst of Energy (Urza's Destiny), Volcanic Upheaval (BFZ)
- **Vanilla creatures:** Grizzly Bears (2/2, {1}{G}), Hill Giant (3/3, {3}{R}), Savannah Lions (2/1, {W}), Earth Elemental (4/5, {3}{R}{R})
- **Keyword creatures (Phase 4):** Serra Angel (4/4 flying, vigilance), Thornweald Archer (2/1 reach, deathtouch), Raging Cougar (2/2 haste), Wall of Stone (0/8 defender), Elvish Archers (2/1 first strike), Ridgetop Raptor (2/1 double strike), War Mammoth (3/3 trample), Knight of Meadowgrain (2/2 first strike, lifelink), Rhox War Monk (3/4 lifelink), Giant Spider (2/4 reach), Vampire Nighthawk (2/3 flying, lifelink, deathtouch)

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

**Solution:** A `skip_first_draw: bool` flag on `GameState`, set during `Game::setup` based on `GameConfig::first_player_draws`. In `process_draw_step`, check and clear the flag.

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

| File                               | Change                                                                                                                                      |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `engine/combat/mod.rs`             | New — module registration                                                                                                                   |
| `engine/combat/validation.rs`      | New — CombatError, AttackConstraints, BlockConstraints, validate_attackers, validate_blockers, effective characteristic helpers (760 lines) |
| `engine/combat/resolution.rs`      | New — CombatDamageAssignment, assign_combat_damage, apply_combat_damage (400 lines)                                                         |
| `engine/combat/steps.rs`           | New — process_declare_attackers, process_declare_blockers, process_combat_damage (190 lines)                                                |
| `engine/stack.rs`                  | Fixed permanent spell resolution (no re-push)                                                                                               |
| `engine/zones.rs`                  | `init_zone_state` made `pub(crate)`                                                                                                         |
| `engine/turns.rs`                  | `damage_orders.clear()` in combat phase end                                                                                                 |
| `state/game_state.rs`              | Added `damage_orders` field                                                                                                                 |
| `state/game.rs`                    | Combat turn-based actions in `run_turn`                                                                                                     |
| `ui/decision.rs`                   | Added `choose_damage_order` to trait + implementations                                                                                      |
| `cards/creatures.rs`               | New — Grizzly Bears, Hill Giant, Savannah Lions                                                                                             |
| `cards/registry.rs`                | Registered 3 creatures                                                                                                                      |
| `tests/phase3_integration_test.rs` | New — 8 integration tests                                                                                                                   |

---

## 5. Phase 4: Keywords ✅ COMPLETED

**Goal:** Keyword abilities that modify combat and other game behaviors.

Completed 2026-03-29. 10 keyword abilities implemented, 11 keyword-bearing creature cards, 51 new tests.

### Keywords implemented

| Keyword | Rule | Hook point | Implementation |
|---------|------|-----------|----------------|
| Flying | 702.9b | `validation.rs` validate_blockers | Per-pair check: flyer can only be blocked by flying/reach |
| Reach | 702.17b | `validation.rs` validate_blockers | Allows blocking flyers |
| Defender | 702.3b | `validation.rs` validate_attackers | `HasDefender` error; can still block |
| Haste | 702.10b/c | `validation.rs` can_attack, `costs.rs` Cost::Tap | Bypasses summoning sickness for attacking and tap abilities |
| Vigilance | 702.20b | `steps.rs` process_declare_attackers | Pre-collected `HashSet<ObjectId>` vigilance set; skips tapping |
| First Strike | 702.7 | `resolution.rs`, `steps.rs` | Filters `assign_combat_damage` by first_strike_only flag |
| Double Strike | 702.4 | `resolution.rs`, `steps.rs` | Deals damage in both first-strike and normal steps |
| Trample | 702.19b/d | `resolution.rs`, `decision.rs` | `choose_trample_damage_assignment` on DecisionProvider; excess to defender |
| Lifelink | 702.15b/f | `actions.rs` perform_action(DealDamage) | Controller gains life equal to damage dealt; does not stack |
| Deathtouch | 702.2b | `actions.rs`, `sba.rs`, `battlefield.rs` | `damaged_by_deathtouch: bool` flag; any nonzero damage is lethal in SBA |

### Key design decisions

1. **`has_keyword(id, KeywordAbility) -> bool`** is the single query point for all keyword checks. Phase 5 will swap this to query the layer system (single-point change).
2. **Flying check is a per-pair validation check**, not a constraint-list entry — simpler, matches Forge/XMage.
3. **Trample delegates to `DecisionProvider`** via `choose_trample_damage_assignment` (8th method on trait). Engine validates but does not choose. Default implementations use `default_trample_assignment` helper.
4. **Deathtouch uses `damaged_by_deathtouch: bool`** on `BattlefieldEntity` — O(1) SBA check, cleared in cleanup.
5. **Lifelink hooks into `perform_action(DealDamage)`** — catches both combat and noncombat damage. Boolean check (does not stack per 702.15f).
6. **First/double strike uses `dealt_first_strike_damage: HashSet<ObjectId>`** on `GameState` — cleared with combat state at end of combat phase.
7. **Vigilance uses pre-collected set** to avoid borrow-checker conflict between `has_keyword` (reads objects) and battlefield mutation.

### New `CombatError` variants

- `HasDefender(ObjectId)` — creature with defender can't attack
- `CantBlockFlyer(ObjectId, ObjectId)` — ground creature can't block flyer

### New `GameState` fields

- `dealt_first_strike_damage: HashSet<ObjectId>` — tracks who dealt first-strike damage

### New `BattlefieldEntity` fields

- `damaged_by_deathtouch: bool` — set by deathtouch damage, checked in SBA, cleared in cleanup

### New `DecisionProvider` method

- `choose_trample_damage_assignment(game, player_id, attacker_id, blockers, defending_target, power, has_deathtouch) -> (Vec<(ObjectId, u64)>, u64)` — returns (blocker assignments, overflow to defender)

---

## 6. Phase 4.5: Oracle Helpers & Decision Providers ✅ COMPLETED

**Goal:** Bridge Phase 4 and Phase 5 by building the oracle query module, mana helpers, display formatting, CLI and Random decision providers, fuzz testing harness, and CLI play binary. Infrastructure phase — no new cards.

Completed 2026-03-30. 287 tests pass (238 unit + 48 integration + 1 doc-test). 500/500 fuzz games pass with zero errors/panics.

### Design: Mana strategy for Decision Providers

The engine doesn't auto-tap lands; tapping is a player decision (mana ability activation via `ActivateAbility`). Since `choose_priority_action` returns a single `PriorityAction` per call, multi-step "tap lands then cast" requires an **action-plan queue**.

**Solution:** Both CLI and Random DPs use a two-part approach:

1. **`oracle/mana_helpers.rs`** — Shared read-only query module:
   - `find_mana_sources(game, player_id, mana_cost)` — Greedy algorithm: reserve colored sources first, then assign remaining to generic. Returns `None` if insufficient.
   - `available_mana_sources(game, player_id)` — All mana sources whose costs can currently be paid. Checks per-ability costs (rule 605.1a/605.1b), not blanket tapped-state.
   - `castable_spells(game, player_id)` — Spells in hand that pass timing + affordability (pool + tap combined via `remaining_cost_after_pool`).
   - `activatable_abilities(game, player_id)` — Non-mana activated abilities affordable with pool + available sources.
   - `passes_timing_check(game, player_id, card_id)` — Read-only mirror of `check_cast_legality`.

2. **Internal action queue** — `RefCell<VecDeque<PriorityAction>>` on each DP. When casting: queue N `ActivateAbility` actions (land taps) followed by one `CastSpell`. Each activation is a separate priority action flowing through the normal engine loop.

**Why:** Engine stays clean (no auto-tap), reusable (CLI, Random, future AI all use `oracle/mana_helpers`), extensible (cost modification queries the cost pipeline), correct (each mana ability activation is a real priority action).

### Shared helpers in `ui/decision.rs`

- `queue_tap_and_cast(queue, sources, card_id)` — Queue mana ability activations followed by `CastSpell`. Used by both CLI and Random DPs.
- `is_action_still_valid(game, player_id, action)` — Best-effort staleness check for queued actions. If one action is stale, the entire plan is discarded (later actions assumed earlier ones would succeed).
- `DispatchDecisionProvider` — Routes decisions to different providers per player. Enables any combination of human/bot/network players.

### 4.5a. `oracle/mana_helpers.rs` + `oracle/legality.rs` expansion ✅

- `oracle/mana_helpers.rs`: `find_mana_sources`, `available_mana_sources`, `castable_spells`, `activatable_abilities`, `passes_timing_check`, `remaining_cost_after_pool`, `can_afford_ability_costs`. 14 unit tests.
- `oracle/legality.rs` expanded: `playable_lands`, `legal_attackers`, `legal_blockers`. 12 unit tests.

### 4.5b. `CliDecisionProvider` (`ui/cli.rs`) ✅

- All 8 `DecisionProvider` methods via stdin/stdout
- Uses `oracle/mana_helpers` to show affordable spells and suggest land taps
- Internal action queue for tap-and-cast sequences
- Multiplayer-ready `choose_attackers` (auto-selects in 2-player, prompts in multiplayer)

### 4.5c. `RandomDecisionProvider` (`ui/random.rs`) ✅

- All 8 methods by making random **legal** choices
- Internal `RefCell<VecDeque<PriorityAction>>` plan queue
- ~85% land play probability, ~40% cast probability
- Stale queue validation via shared `is_action_still_valid`
- 5 unit tests

### 4.5d. Fuzz harness (`bin/fuzz_games.rs`) ✅

- Runs N games of Random vs Random with `std::panic::catch_unwind` per game
- CLI args: `--games`, `--max-turns`, `--verbose`, `--dump-events <path>`
- Reports: completed, errors, panics, hit-turn-limit, avg turns, max turns, time/game
- Random deck generation from `CardRegistry` (~17 lands, ~23 nonlands)

### 4.5e. CLI play binary (`bin/cli_play.rs`) ✅

- Human (CLI, player 0) vs Random bot (player 1)
- Uses `DispatchDecisionProvider` from library
- Fixed test deck: Mountains, Forests, Grizzly Bears, Hill Giants, Lightning Bolts

### 4.5f. Bug fixes from fuzz testing ✅

1. `priority.rs`: `ActivateAbility` now checks ability type — routes mana abilities to `activate_mana_ability()` (rule 605, resolves immediately) instead of `activate_ability()` which rejected them.
2. `oracle/mana_helpers.rs` `passes_timing_check`: Added ownership check (`obj.owner == player_id`) and zone check (`obj.zone == Hand`) to prevent suggesting opponent's spells.
3. `ui/random.rs`: Added `is_action_still_valid()` to validate queued actions before returning them. Clears stale queue if a permanent was tapped or a card left hand between plan creation and execution.

### Display formatting (`ui/display.rs`) ✅

Moved from `oracle/display.rs` to `ui/display.rs` — oracle module is for read-only game state queries, display formatting is presentation logic that belongs in `ui/`.

- `card_label`, `card_name`, `format_permanent` (with P/T, damage, keywords, non-keyword abilities, status flags)
- `format_hand`, `format_battlefield` (grouped by type: Creatures, Lands, Other)
- `format_stack` (with `<- top (resolves next)` and `<- bottom` markers)
- `format_phase`, `format_player_summary`, `format_mana_pool`
- `format_event`, `format_event_log` (for fuzz harness event dumps)
- 10 unit tests

---

## 7. Phase 5: Continuous Effects & Layer System

**Goal:** Effects that modify game state continuously (e.g. "all creatures get +1/+1", "+3/+3 until end of turn") and restrictions that prevent actions (Solemnity, Null Rod). All 7 layers implemented. Correctness-first, recompute-on-query (no caching).

### Core types

```rust
struct ContinuousEffect {
    id: EffectId,
    source: ObjectId,              // permanent or spell that created it
    timestamp: u64,                // from GameState::allocate_timestamp()
    duration: Duration,
    layer: Layer,
    modification: Modification,    // what it does
    applies_to: AppliesTo,         // which objects it affects
    controller: PlayerId,
    is_cda: bool,                  // characteristic-defining ability (rule 604.3)
}

enum Layer {
    Copy,                          // Layer 1 — rule 613.1a
    Control,                       // Layer 2 — rule 613.1b
    Text,                          // Layer 3 — rule 613.1c
    Type,                          // Layer 4 — rule 613.1d
    Color,                         // Layer 5 — rule 613.1e
    Ability,                       // Layer 6 — rule 613.1f
    PowerToughness(PTSublayer),    // Layer 7 — rule 613.1g
}

enum PTSublayer {
    CDA,       // 7a — CDAs that define P/T (e.g. Tarmogoyf)
    SetBase,   // 7b — effects that set P/T to specific values
    Modify,    // 7c — effects that modify P/T (+N/+N, -N/-N)
    Switch,    // 7d — effects that switch P/T
}

enum Modification {
    CopyOf(ObjectId),                                       // Layer 1
    ChangeController(PlayerId),                             // Layer 2
    ChangeText { from: String, to: String },                // Layer 3
    AddTypes(Vec<CardType>), RemoveTypes(Vec<CardType>),    // Layer 4
    AddSubtypes(Vec<Subtype>), RemoveSubtypes(Vec<Subtype>),
    SetCreatureType(Vec<Subtype>),
    SetColors(Vec<Color>), AddColors(Vec<Color>), RemoveColors(Vec<Color>),  // Layer 5
    AddAbility(KeywordAbility), RemoveAbility(KeywordAbility),               // Layer 6
    RemoveAllAbilities,
    SetPT(i32, i32), ModifyPT(i32, i32), SwitchPT,         // Layer 7
}

enum AppliesTo {
    Single(ObjectId),                      // "target creature" or "enchanted creature"
    Filter(PermanentFilter, PlayerId),     // "creatures you control" / "white creatures"
    All,                                   // "all creatures"
    Self_,                                 // the source permanent itself
}
```

### The layer system (rule 613)

Effects are applied in strict layer order: 1 (copy) → 2 (control) → 3 (text) → 4 (type) → 5 (color) → 6 (ability) → 7a-7d (P/T sublayers).

Within each layer/sublayer, effects are ordered by:
1. Dependency detection (rule 613.8) — structural analysis + hypothetical fallback
2. Timestamp (rule 613.7) — ties broken by `BattlefieldEntity.timestamp`

### Dependency detection (rule 613.8)

**Hybrid algorithm:** structural analysis eliminates most pairs cheaply, hypothetical check runs only on candidates.

1. **Collect** all active effects in this layer/sublayer
2. **Static check** — Does B's `ModifiesCategory` overlap A's `FilterDependency`? If no overlap → independent.
3. **CDA guard** (613.8a(c)) — If one is CDA and the other isn't → independent.
4. **Hypothetical check** — Temporarily apply B, recompute A's `applies_to`, compare. If different → A depends on B.
5. **Build DAG** — Edges: B → A (apply B before A).
6. **Topological sort** — Ties by timestamp. Cycles (613.8b) → fall back to timestamp order.

Implementation files: `engine/dependency.rs`, `types/continuous.rs`

### Cost restriction system

This phase also activates the `CostRestriction` framework designed in Section 3.3. Continuous effects that prevent costs (Solemnity, Null Rod, Sigarda, etc.) populate `GameState::cost_restrictions`, which `can_pay_costs` already queries.

Similarly, `AttackConstraint` and `BlockConstraint` (Section 4) are populated by continuous effects from this phase onward.

### Rule 613.11: Game-rule-modifying effects + cost pipeline

Static abilities like Thalia ("noncreature spells cost {1} more") create continuous effects that modify game rules rather than object characteristics. Wire into the `cast.rs` cost-modification pipeline (rule 601.2e):

1. Start with base mana cost
2. Apply cost increases (from 613.11 effects)
3. Apply cost reductions
4. Apply Trinisphere-style floors
5. Lock final cost

### Implementation sub-steps

- **5a** — `types/continuous.rs` + `GameState` field: `ContinuousEffect`, `Layer`, `Modification`, `AppliesTo` types. `continuous_effects: Vec<ContinuousEffect>` on `GameState`. `register_continuous_effect()`, `remove_effects_from_source()`, `effects_in_layer()`.
- **5b** — Duration tracking + cleanup: `UntilEndOfTurn` removed in cleanup step. `WhileSourceOnBattlefield` removed on zone-change events. `Indefinite` persists until explicit removal.
- **5c** — Layer engine core (`engine/layers.rs`): `compute_characteristics(game, object_id) -> EffectiveCharacteristics`. Processes layers 1-7 in order.
- **5d** — Layer 7: P/T sublayers 7a-7d. **Update `oracle/characteristics::get_effective_power/toughness`** to route through layer engine. Remove `power_modifier`/`toughness_modifier` from `BattlefieldEntity`.
- **5e** — Layer 6: Abilities. `compute_effective_keywords`. **Update `oracle/characteristics::has_keyword`** to route through layer engine.
- **5f** — Layers 4-5: Type + color change. **Update `oracle/characteristics::is_creature`** to route through layer engine.
- **5g** — Layer 2: Control-changing effects.
- **5h** — Layers 1+3: Copy + text change (minimal scaffolding, expanded later).
- **5i** — Dependency detection (`engine/dependency.rs`): `FilterDependency`, `ModifiesCategory` enums. Static + hypothetical hybrid algorithm. Topological sort with cycle fallback.
- **5j** — Hook `resolve_primitive` → register effects: `ModifyPowerToughness` → layer 7c, `AddAbility`/`RemoveAbility` → layer 6, `SetPowerToughness` → layer 7b, `ChangeColor` → layer 5, `ChangeType` → layer 4, `GainControl` → layer 2.
- **5k** — Rule 613.11 + cost modification pipeline in `cast.rs`.
- **5l** — Cards: Giant Growth ({G}, instant, +3/+3 until end of turn), Glorious Anthem ({1}{W}{W}, enchantment, creatures you control get +1/+1), Honor of the Pure ({1}{W}, enchantment, white creatures you control get +1/+1), Clone ({3}{U}, creature, copy effect scaffold).
- **5m** — Tests: unit tests in `layers.rs` + `dependency.rs`, integration tests for Giant Growth / Anthem / Honor / Clone, fuzz regressions with new cards.

---

## 8. Phase 6: Triggered Abilities

**Goal:** "When/Whenever/At [event], [effect]" abilities that go on the stack (rule 603).

### Key components

- **TriggerCondition** enum: `OnETB(filter)`, `OnLTB(filter)`, `OnDeath(filter)`, `OnDamageDealt(filter)`, `OnLifeGained`, `OnSpellCast(filter)`, `AtBeginningOf(StepType)`, etc.
- **TriggeredAbilityDef** on `CardData`: `trigger: TriggerCondition, effect: Effect`
- **Trigger checking**: after each event batch (or SBA cycle), scan all permanents for matching triggers
- **Stack placement**: triggered abilities go on stack in APNAP order (rule 603.3b)
- **Delayed triggers**: created by spells/abilities, fire once (rule 603.7)

### Integration point

- `priority.rs` `perform_sba_and_triggers()` — already stubbed; triggers slot in here
- After SBAs, the trigger scanner walks new entries in the **delta log** (`engine/delta_log.rs`), matches them against registered `TriggerKind` patterns, and places matching triggered abilities on the stack in APNAP order

**IMPORTANT — Do NOT scan `EventLog` for trigger matching.** The event log is a diagnostic/UI artifact, not a source of truth for game mechanics. The correct approach: all state-mutating methods (`move_object`, `perform_action`, etc.) emit structured `GameDelta` entries into a dedicated **delta log** on `GameState`. The trigger scanner reads this log at well-defined checkpoints. This keeps the rules engine self-contained and the event log purely observational.

**Architecture change (2026-04-06):** The original `pending_triggers: Vec<PendingTrigger>` push-based design has been replaced by the delta log approach (see `state-tracking-architecture.md`). Rationale: push-based triggers require every mutation site to explicitly know about every trigger condition, making state-based triggers (rule 603.8 — e.g., "whenever you have no cards in hand") impractical without O(state_triggers × mutations) polling at each substep. The delta log centralizes detection: mutation sites emit generic `GameDelta` entries with `(old, new)` pairs, and a single scanner runs pattern matching after event batches. This also unifies trigger detection with loop detection and voluntary shortcut validation (D26), avoiding three separate observation mechanisms.

### EventLog relationship

The `EventLog` (`events/event.rs`) will become a **projection** of the delta log once Phase 6 is built. Currently, mutation sites manually call `self.events.emit(...)` — this is scaffolding. In the final design:

1. Mutation methods emit **deltas only** (no manual `events.emit()`).
2. At checkpoints (post-SBA, post-resolution), a **projection function** converts new deltas into human-readable `GameEvent` entries for the EventLog.
3. The trigger scanner reads the same deltas independently.

This guarantees single-source-of-truth semantics. Existing type-specific events like `CreatureDied`, `PlaneswalkerDied`, and `LegendRuleSacrificed` are display conveniences — trigger detection uses the zone-transition delta plus **last-known characteristics** (rule 603.10) to match triggers, not event type. This correctly handles multi-type permanents (e.g. Gideon dying triggers both "when a creature dies" and "when a planeswalker is put into a graveyard" from a single delta).

### Cards to implement

- **Soul Warden** ("Whenever a creature enters, you gain 1 life") — ETB trigger
- **Blood Artist** ("Whenever a creature dies, target player loses 1 life and you gain 1 life") — death trigger
- One "at beginning of upkeep" trigger TBD

---

## 9. Phase 7: Replacement & Prevention Effects

**Goal:** "If X would happen, instead Y" and "prevent N damage" (rules 614-616).

### Key components

- **ReplacementEffect** struct: condition, replacement, source, duration, controller
- **Prevention effect** is a subtype of replacement (rule 615)
- **`apply_replacement_effects()`** inserted into `execute_action()` — the hook is already designed (`execute_action` → `apply_replacement_effects` → `perform_action`)
- **Rule 616 ordering**: when multiple replacements apply, affected player/controller chooses
- **Self-referential loop prevention** (rule 614.5): each replacement applies at most once per event
- **"Skip your draw step" effects** (Omen Machine, Maralen) are replacement effects, not boolean flags. The framework handles stacking multiple replacements and "if you would draw, instead..." chains.

### Cards to implement

- **Fog** ("Prevent all combat damage this turn") — prevention shield
- **Enters-tapped** effects (taplands) — ETB replacement
- Additional TBD

---

## 10. Excluded Cards

These cards are **explicitly out of scope** due to rules ambiguities or engine-breaking interactions. They are non-competitive and not worth the architectural complexity they would require.

| Card                           | Reason                                                                                                                                                                                                                                                                                                                                                             |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Season of the Witch**        | "Destroy each creature that couldn't attack this turn" — "couldn't attack" is still poorly defined in the comprehensive rules. Would require tracking hypothetical attack legality for every creature every turn.                                                                                                                                                  |
| **Panglacial Wurm**            | "While you're searching your library, you may cast this card from your library." Casting from a library mid-search breaks stack assumptions, zone transition invariants, and mana ability resolution. Requires the ability to interrupt a search effect with a full cast sequence.                                                                                 |
| **Selvala, Explorer Returned** | Mana ability that reveals hidden information and produces an undefined amount of mana. You can begin casting a spell, activate Selvala to pay for it, discover you don't have enough mana, and have no way to cleanly rewind the game state (revealed cards, gained life, etc.). Breaks the assumption that mana abilities are deterministic and side-effect-free. |

Additional cards may be added to this list as development progresses. The general criteria: if a card requires architectural changes that benefit only that card and a handful of similar effects, it's not worth supporting until/unless the simulator's scope explicitly expands to include it.

---

## 11. Design Decisions Log

| Date       | Decision                                             | Rationale                                                                                                                                                                                                                           |
| ---------- | ---------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 2026-03-27 | Central `HashMap<ObjectId, GameObject>` store        | Zones reference by ID; one source of truth                                                                                                                                                                                          |
| 2026-03-27 | `DecisionProvider` trait for all player choices      | Engine stays pure, testable, UI-independent                                                                                                                                                                                         |
| 2026-03-28 | `Arc<CardData>` shared across instances              | Avoids cloning ~1KB card data per object                                                                                                                                                                                            |
| 2026-03-28 | `Vec<ManaSymbol>` for ManaCost                       | Supports hybrid/phyrexian/X/snow natively                                                                                                                                                                                           |
| 2026-03-28 | `Effect` combinator tree (Atom/Sequence/...)         | Composable, serializable, covers 95%+ of cards                                                                                                                                                                                      |
| 2026-03-29 | Stack pop-first during resolution                    | Resolving spell should NOT be visible on stack                                                                                                                                                                                      |
| 2026-03-29 | `auto_allocate_generic` lives in `ui/decision.rs`    | Engine requires manual selection; convenience is layered on top                                                                                                                                                                     |
| 2026-03-29 | `can_pay_costs` = resource check + restriction check | Two-layer validation: "do you have it?" then "does the game allow it?" Restriction layer starts as no-op, activated by Phase 5 continuous effects                                                                                   |
| 2026-03-29 | `GameConfig` struct now, `Format` trait later        | Covers 90% of formats with pure data; `Format` trait only needed when Commander/Brawl require behavioral differences. `Game` struct already structured for easy migration.                                                          |
| 2026-03-29 | `Game` owns `DecisionProvider` dispatch              | Engine methods stay as pure state transforms. `Game::run_turn` is the only place that calls decision methods. No threading `DecisionProvider` through `advance_turn`.                                                               |
| 2026-03-29 | Combat split: validation + resolution                | Once attackers/blockers are locked in, damage is deterministic. Validation is the complex part (constraints, forced attacks, evasion). Extensible via `AttackConstraint` / `BlockConstraint` lists populated by continuous effects. |
| 2026-03-29 | `skip_first_draw` flag only for rule 103.8a           | In-game "skip draw" effects are replacement effects (Phase 7), not boolean flags. The flag is a one-time game-setup mechanism.                                                                                                      |
| 2026-03-29 | Explicit excluded cards list                         | Season of the Witch, Panglacial Wurm, Selvala — non-competitive cards that require disproportionate architectural changes.                                                                                                          |
| 2026-03-29 | Two-phase combat damage (compute then apply)         | `assign_combat_damage` is a read-only free function; `apply_combat_damage` mutates. Avoids borrow checker issues from reading battlefield while writing damage.                                                                     |
| 2026-03-29 | Effective characteristic helpers on GameState        | `is_creature()`, `can_attack()`, `get_effective_power/toughness()` — combat code calls these instead of reading `card_data` directly, so Phase 5 layer-system swap is a single-point change.                                        |
| 2026-03-29 | Permanent spell resolution: no re-push               | Fixed Phase 2 hack. Manual zone bookkeeping for permanent spells resolving to battlefield, consistent with instant/sorcery path.                                                                                                    |
| 2026-03-30 | Oracle module separates read-only queries from engine | `oracle/` contains characteristics, legality, board, mana_helpers — all pure `&GameState` queries. Engine stays mutation-only. Single-point change for Phase 5 layer integration.                                                   |
| 2026-03-30 | `display.rs` in `ui/`, not `oracle/`                 | Display formatting is presentation logic, not game-state queries. Oracle = "what is true", UI = "how to present it".                                                                                                                |
| 2026-03-30 | Action-plan queue for mana ability + cast sequences   | DPs hold `RefCell<VecDeque<PriorityAction>>`. Each mana tap is a real priority action. Engine stays clean; no auto-tap.                                                                                                             |
| 2026-03-30 | `DispatchDecisionProvider` routes by player ID       | Enables any combination of human/bot/network players in a single game. Replaces per-binary boilerplate with a reusable library type.                                                                                                |
| 2026-03-30 | Phase restructuring: old Phase 6 split into 6+7      | Triggered abilities (stack-based, rule 603) and replacement effects (middleware-based, rules 614-616) are architecturally distinct. Separate phases reduce per-phase complexity.                                                     |
| 2026-03-30 | Correctness-first layer system, no caching           | Recompute-on-query for all characteristics. Caching added later only if parallel AI self-play proves too slow.                                                                                                                      |
| 2026-04-01 | Trigger system uses delta log, not EventLog | EventLog is a diagnostic/UI artifact. Trigger matching is driven by a structured **delta log** (`Vec<GameDelta>`) on GameState, populated by all state-mutating methods. Never scan EventLog for game-mechanical purposes. *(Updated 2026-04-06: replaced push-based `PendingTrigger` queue with delta log — see `state-tracking-architecture.md` for rationale.)* |
| 2026-04-03 | `was_cast_at_non_sorcery_speed: bool` stored in `CastInfo` | Cast-time timing metadata belongs in `CastInfo` (general struct on `BattlefieldEntity`), not per-card storage. Computed at cast time: `!(is_active_player && is_main_phase && stack_is_empty)`. Used by Necromancy-style cards (307.5a). 1 bool per permanent, avoids polymorphism/bespoke storage. Same pattern as `was_kicked`. |
| 2026-04-11 | EventLog is derived from delta log, not independently emitted | Currently the EventLog is populated by manual `events.emit()` calls at each mutation site. In Phase 6, when the delta log is built, the EventLog will become a **projection** of the delta log: mutation methods emit deltas only, and a projection function at well-defined checkpoints converts new deltas into `GameEvent` entries for display/logging. This eliminates dual-emit fragility and guarantees EventLog can never disagree with the trigger scanner about what happened. The manual `events.emit()` calls are scaffolding until Phase 6. |
| 2026-04-11 | No type-specific death events for trigger detection | `CreatureDied`, `PlaneswalkerDied`, `LegendRuleSacrificed` are display/logging conveniences only. They must NOT be used for trigger matching. The correct model: the delta log records one zone-transition delta (`Battlefield → Graveyard`), and the trigger scanner inspects the object's **last-known characteristics** (rule 603.10) to determine which triggers fire. For multi-type permanents (e.g. Gideon as both creature and planeswalker), both "when a creature dies" and "when a planeswalker is put into a graveyard" triggers match from the same delta — no need for multiple events. |
| 2026-04-13 | `DecisionProvider` refactored to 4 generic primitives | MtG requires 100+ distinct decision types across the full card pool. Typed methods don't scale. Trait now has 4 methods: `pick_n`, `pick_number`, `allocate`, `choose_ordering`. Semantic context passed via `ChoiceContext` (containing `ChoiceKind` enum — no prompt string; display formatting belongs in DP impls; exhaustive matching, no `#[non_exhaustive]`). Engine call sites use typed `ask_*` free functions that pack/unpack context and validate responses. `ScriptedDecisionProvider` uses `ChoiceKind`-aware queue with assertion matching (`expect_pick_n(kind, response)`) — no `Any` fallback, every expectation requires a `ChoiceKind`. Card name selection uses `pick_number` with `CardRegistry` index for O(1) calls (see design doc §8.7). Design doc: `plans/atomic-tests/supplemental-docs/decision-provider-refactor.md`. Tickets: SPECIAL-1a/b/c (3-way split: types+trait+ask+ScriptedDP → CLI/Random/Dispatch impls → engine migration+old trait deletion). |
| 2026-04-14 | No `#[non_exhaustive]` on any enum | Single-crate project — `#[non_exhaustive]` trades away the compiler's exhaustive-match warnings (which flag missed cases when a variant is added) for downstream compatibility (irrelevant here). **Policy:** No enum gets `#[non_exhaustive]`. This includes `ChoiceKind`: when a new variant is added, the compiler flags every match site that doesn't handle it — this is strictly better than a `_ =>` catch-all that silently hides unhandled cases. Applies to all enums: `KeywordAbility`, `Primitive`, `CreatureType`, `GameEvent`, etc. If the project later becomes multi-crate (engine lib + UI bins), revisit for public-API enums at that boundary. |

---

## Implementation Order

```
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

Phase 4: Keywords ✅
  4a   has_keyword helper ✅
  4b   Flying / Reach (per-pair evasion check) ✅
  4c   Defender (can't attack) ✅
  4d   Haste (summoning sickness bypass) ✅
  4e   Vigilance (no tap on attack) ✅
  4f   First Strike / Double Strike (two damage steps) ✅
  4g   Trample (excess to defender via DecisionProvider) ✅
  4h   Lifelink (damage → life gain in perform_action) ✅
  4i   Deathtouch (damaged_by_deathtouch flag + SBA) ✅
  4j   Cards: 11 keyword creatures ✅
  4k   Integration tests (12 tests) ✅
  4l   Design doc update ✅

Phase 4.5: Oracle Helpers & Decision Providers ✅
  4.5a oracle/mana_helpers.rs + oracle/legality.rs expansion ✅
  4.5b CliDecisionProvider (ui/cli.rs) ✅
  4.5c RandomDecisionProvider (ui/random.rs) ✅
  4.5d Fuzz harness binary (bin/fuzz_games.rs) ✅
  4.5e CLI play binary (bin/cli_play.rs) ✅
  4.5f Bug fixes from fuzz/CLI testing ✅
  ---  Design doc restructure ✅

Phase 5: Continuous Effects & Layer System
  5a   types/continuous.rs + GameState field
  5b   Duration tracking + cleanup hooks
  5c   Layer engine core (engine/layers.rs)
  5d   Layer 7: P/T sublayers (update oracle/characteristics)
  5e   Layer 6: abilities (update oracle/characteristics)
  5f   Layers 4-5: types + colors (update oracle/characteristics)
  5g   Layer 2: control
  5h   Layers 1+3: copy + text (scaffolding)
  5i   Dependency detection (engine/dependency.rs)
  5j   Hook resolve_primitive → register effects
  5k   Rule 613.11 + cost modification pipeline
  5l   Cards: Giant Growth, Glorious Anthem, Honor of the Pure, Clone
  5m   Tests + fuzz regression

Phase 6: Triggered Abilities
  6a   TriggerCondition enum + TriggeredAbilityDef
  6b   Trigger checking + event matching
  6c   Stack placement (APNAP order)
  6d   Delayed triggers
  6e   Cards: Soul Warden, Blood Artist

Phase 7: Replacement & Prevention Effects
  7a   ReplacementEffect struct + apply_replacement_effects()
  7b   Rule 616 ordering + loop prevention (614.5)
  7c   Prevention effects (rule 615)
  7d   Cards: Fog, enters-tapped lands
```
