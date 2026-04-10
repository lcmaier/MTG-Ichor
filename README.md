# MTG-Ichor — Magic: The Gathering Rules Engine

A from-scratch Magic: The Gathering rules engine written in Rust, designed to be **fast**, **correct**, **extensible**, and **manageable**. The engine can power a GUI for two humans playing over a network, or run headless for bot-vs-bot self-play in dozens of parallel games.

> **Status:** Post-Phase 4.5 — 287 tests passing, 500/500 fuzz games complete with zero panics. Core gameplay loop (turns, mana, casting, combat, keywords) is fully functional. Currently preparing for Phase 5: Continuous Effects & Layer System.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Game — lifecycle, setup, config, DecisionProvider dispatch │
├─────────────────────────────────────────────────────────────┤
│  ui/decision.rs — DecisionProvider trait                    │
│  (CLI, Random, Scripted, Dispatch implementations)          │
├─────────────────────────────────────────────────────────────┤
│  engine/ — Rules engine (reads + mutates GameState)         │
│  ┌──────────┬──────────┬────────────┬─────────────────────┐ │
│  │ cast.rs  │ stack.rs │priority.rs │ targeting.rs        │ │
│  │ turns.rs │ zones.rs │ resolve.rs │ costs.rs            │ │
│  │ sba.rs   │ mana.rs  │ combat/    │ actions.rs          │ │
│  └──────────┴──────────┴────────────┴─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  oracle/ — Read-only game state queries                     │
│  (characteristics, legality, board, mana_helpers)           │
├─────────────────────────────────────────────────────────────┤
│  state/ — Pure data (GameState, GameConfig, PlayerState)    │
│  objects/ — GameObject, CardData, BattlefieldEntity         │
│  types/ — Enums and value types (no logic)                  │
│  events/ — EventLog for game history                        │
│  cards/ — Card definitions (data only, via CardRegistry)    │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Principles

- **Central object store:** All game objects live in a single `HashMap<ObjectId, GameObject>`. Zones reference objects by ID.
- **Single zone-transition chokepoint:** All zone moves go through `move_object()`.
- **Engine does no I/O:** Every player decision is routed through the `DecisionProvider` trait. The engine is pure state transforms.
- **Composable effect system:** Card effects are trees built from ~35 `Primitive` variants and ~7 `Effect` combinators (`Atom`, `Sequence`, `Conditional`, `Modal`, etc.).
- **Immutable card data:** `CardData` is `Arc`-shared across instances — the layer system computes effective characteristics on top of printed values.
- **Oracle module:** Read-only queries (`has_keyword`, `is_creature`, `get_effective_power/toughness`, `castable_spells`, etc.) are isolated in `oracle/`, making the Phase 5 layer-system swap a single-point change.
- **Action pipeline:** All observable mutations route through `execute_action(GameAction)`, designed for Phase 7 replacement-effect middleware.

---

## What's Implemented

| Area                                                                                                                 | Status     |
| -------------------------------------------------------------------------------------------------------------------- | ---------- |
| Turn structure (all phases/steps)                                                                                    | ✅          |
| Mana system (Colored, Generic, Colorless symbols)                                                                    | ✅          |
| Casting spells (rule 601.2)                                                                                          | ✅          |
| Stack & resolution (rule 608, pop-first)                                                                             | ✅          |
| Priority system (rule 117, SBA loop)                                                                                 | ✅          |
| Targeting (Creature, Player, Any, Permanent, Spell)                                                                  | ✅          |
| Effect resolver (DealDamage, DrawCards, GainLife, LoseLife, Destroy, etc.)                                           | ⚠️ Partial |
| State-based actions (704.5a life, 704.5b empty library, lethal damage, zero toughness)                               | ✅          |
| Combat (declare attackers/blockers, damage assignment, 2025 rules)                                                   | ✅          |
| 10 keyword abilities (flying, reach, defender, haste, vigilance, first/double strike, trample, lifelink, deathtouch) | ✅          |
| Oracle module (characteristics, legality, mana helpers, board queries)                                               | ✅          |
| CLI play (human vs random bot)                                                                                       | ✅          |
| Fuzz testing (random vs random, 500+ games)                                                                          | ✅          |

### Cards (24)

- **Basic lands:** Plains, Island, Swamp, Mountain, Forest
- **Instants/Sorceries:** Lightning Bolt, Ancestral Recall, Counterspell, Burst of Energy, Volcanic Upheaval
- **Vanilla creatures:** Grizzly Bears, Hill Giant, Savannah Lions, Earth Elemental
- **Keyword creatures:** Serra Angel, Thornweald Archer, Raging Cougar, Wall of Stone, Elvish Archers, Ridgetop Raptor, War Mammoth, Knight of Meadowgrain, Rhox War Monk, Giant Spider, Vampire Nighthawk

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)

### Build & Test

```bash
cd mtgsim
cargo build
cargo test
```

### Run the CLI (Human vs Random Bot)

```bash
cargo run --bin cli_play
```

You play as Player 0 (CLI) against a random-decision bot (Player 1). The game uses a fixed test deck of Mountains, Forests, creatures, and Lightning Bolts.

### Run the Fuzz Harness

```bash
# Run 500 random-vs-random games
cargo run --bin fuzz_games -- --games 500

# Verbose mode with event log dump
cargo run --bin fuzz_games -- --games 100 --verbose --dump-events events.log
```

CLI flags: `--games N`, `--max-turns N`, `--verbose`, `--dump-events <path>`

---

## Project Structure

```
mtgsim/
├── src/
│   ├── main.rs                  # Entry point stub
│   ├── lib.rs                   # Crate root (module declarations)
│   ├── bin/
│   │   ├── cli_play.rs          # Human vs bot binary
│   │   └── fuzz_games.rs        # Random-vs-random fuzz harness
│   ├── cards/
│   │   ├── registry.rs          # CardRegistry (name → CardData factory)
│   │   ├── basic_lands.rs       # Plains, Island, Swamp, Mountain, Forest
│   │   ├── alpha.rs             # Lightning Bolt, Ancestral Recall, Counterspell, etc.
│   │   ├── creatures.rs         # Vanilla creatures (Bears, Giant, Lions, Elemental)
│   │   └── keyword_creatures.rs # Keyword-bearing creatures (11 cards)
│   ├── engine/
│   │   ├── actions.rs           # GameAction enum + execute_action chokepoint
│   │   ├── cast.rs              # Spell casting (rule 601.2)
│   │   ├── costs.rs             # Cost validation + payment
│   │   ├── combat/              # Combat subsystem
│   │   │   ├── validation.rs    # Attack/block validation + constraints
│   │   │   ├── resolution.rs    # Damage assignment + application
│   │   │   ├── steps.rs         # Combat step processors
│   │   │   └── keywords.rs      # Combat keyword helpers (first strike, trample)
│   │   ├── keywords.rs          # Non-combat keyword hooks (deathtouch, lifelink)
│   │   ├── mana.rs              # Mana ability resolution
│   │   ├── priority.rs          # Priority loop (rule 117)
│   │   ├── resolve.rs           # Effect tree resolver
│   │   ├── sba.rs               # State-based actions (rule 704)
│   │   ├── stack.rs             # Stack management + spell resolution
│   │   ├── targeting.rs         # Target validation
│   │   ├── turns.rs             # Turn/phase/step progression
│   │   └── zones.rs             # Zone transitions (move_object)
│   ├── events/
│   │   └── event.rs             # GameEvent enum + EventLog
│   ├── objects/
│   │   ├── card_data.rs         # CardData, AbilityDef, Cost, CardDataBuilder
│   │   └── object.rs            # GameObject (central object)
│   ├── oracle/
│   │   ├── characteristics.rs   # has_keyword, is_creature, effective P/T
│   │   ├── legality.rs          # playable_lands, legal_attackers/blockers
│   │   ├── board.rs             # permanents_controlled_by
│   │   └── mana_helpers.rs      # castable_spells, find_mana_sources, etc.
│   ├── state/
│   │   ├── game_state.rs        # GameState, Phase, StackEntry
│   │   ├── game.rs              # Game lifecycle wrapper
│   │   ├── game_config.rs       # GameConfig (format presets)
│   │   ├── player.rs            # PlayerState (life, hand, library, mana pool)
│   │   └── battlefield.rs       # BattlefieldEntity (tapped, damage, combat state)
│   ├── types/
│   │   ├── ids.rs               # ObjectId, PlayerId, AbilityId
│   │   ├── mana.rs              # ManaSymbol, ManaCost, ManaPool
│   │   ├── effects.rs           # Primitive, Effect, TargetSpec, Duration, etc.
│   │   ├── card_types.rs        # CardType, Supertype, Subtype (full enums)
│   │   ├── colors.rs            # Color enum
│   │   ├── keywords.rs          # KeywordAbility enum
│   │   └── zones.rs             # Zone enum
│   └── ui/
│       ├── decision.rs          # DecisionProvider trait + Dispatch + shared helpers
│       ├── cli.rs               # CliDecisionProvider (stdin/stdout)
│       ├── random.rs            # RandomDecisionProvider (fuzz/bot)
│       └── display.rs           # Text formatting for CLI/logs
├── tests/
│   ├── integration_test.rs
│   ├── phase2_integration_test.rs
│   ├── phase3_integration_test.rs
│   ├── phase4_integration_test.rs
│   └── pre_phase3_integration_test.rs
├── Cargo.toml
└── Cargo.lock
```

---

## Roadmap

| Phase   | Scope                                                   | Status     |
| ------- | ------------------------------------------------------- | ---------- |
| **1**   | Types, GameState, zones, turn structure, mana, priority | ✅ Complete |
| **2**   | Stack, casting, spell resolution, one-shot effects      | ✅ Complete |
| **3**   | Creatures, combat (full), SBAs                          | ✅ Complete |
| **4**   | Keywords (10 abilities)                                 | ✅ Complete |
| **4.5** | Oracle helpers, CLI + Random DPs, fuzz harness          | ✅ Complete |
| **5**   | **Continuous effects & layer system (rule 613)**        | 🔜 Next    |
| **6**   | Triggered abilities (rule 603)                          | Planned    |
| **7**   | Replacement & prevention effects (rules 614–616)        | Planned    |

### Phase 5: Continuous Effects & Layers

The next major milestone. Implements all 7 layers of the continuous effect system (rule 613), dependency detection (rule 613.8), duration tracking, cost restriction activation, and the cost modification pipeline (rule 601.2e).

**New cards:** Giant Growth, Glorious Anthem, Honor of the Pure, Clone

### Phase 6: Triggered Abilities

"When/whenever/at" abilities that go on the stack (rule 603). ETB triggers, death triggers, upkeep triggers. APNAP ordering and delayed triggers.

**Target cards:** Soul Warden, Blood Artist

### Phase 7: Replacement & Prevention Effects

"If X would happen, instead Y" (rule 614) and damage prevention (rule 615). Hooks into the `execute_action` pipeline.

**Target cards:** Fog, enters-tapped lands

---

## DecisionProvider

The engine is completely UI-agnostic. All player choices route through the `DecisionProvider` trait (8 methods):

| Method                              | Purpose                                                  |
| ----------------------------------- | -------------------------------------------------------- |
| `choose_priority_action`            | What to do when you have priority (cast, activate, pass) |
| `choose_targets`                    | Target selection for spells/abilities                    |
| `choose_attackers`                  | Declare attackers                                        |
| `choose_blockers`                   | Declare blockers                                         |
| `choose_discard`                    | Choose cards to discard (cleanup step)                   |
| `choose_attacker_damage_assignment` | Divide combat damage among blockers                      |
| `choose_trample_damage_assignment`  | Divide trample damage (blockers + defender)              |
| `choose_generic_mana_allocation`    | Which mana types to spend for generic costs              |

**Built-in implementations:**

- `CliDecisionProvider` — interactive stdin/stdout for human play
- `RandomDecisionProvider` — makes random legal choices (for fuzz testing and bot opponents)
- `ScriptedDecisionProvider` — pre-programmed decisions (for deterministic integration tests)
- `PassiveDecisionProvider` — always passes priority (for unit tests)
- `DispatchDecisionProvider` — routes decisions to different providers per player ID

---

## Design Documents

- [`design_doc.md`](design_doc.md) — Single source of truth for architecture, status, and upcoming work
- [`effect_system_plan.md`](effect_system_plan.md) — Detailed design for the Primitive/Effect combinator system
- [`implementation_phases.md`](implementation_phases.md) — Phase scope summary

---

## License

See [LICENSE](LICENSE) for details.
