mtgsim/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   │
│   ├── types/                    # Pure data types, no logic, no dependencies on game
│   │   ├── mod.rs
│   │   ├── ids.rs                # ObjectId, PlayerId, AbilityId (newtypes)
│   │   ├── mana.rs               # ManaColor, ManaCost, ManaPool
│   │   ├── card.rs               # CardType, Supertype, Subtype enums (port from v1)
│   │   ├── zones.rs              # Zone enum
│   │   ├── keywords.rs           # KeywordAbility enum (Flying, Trample, etc.)
│   │   └── colors.rs             # Color enum
│   │
│   ├── objects/                  # Game object representation
│   │   ├── mod.rs
│   │   ├── card_data.rs          # CardData — the immutable "printed card" definition
│   │   ├── object.rs             # GameObject — runtime instance (ID + card_data_ref + state)
│   │   └── characteristics.rs    # Computed characteristics (after layer system applies)
│   │
│   ├── state/                    # The game state container
│   │   ├── mod.rs
│   │   ├── game_state.rs         # GameState — all zones, turn info, per-object state
│   │   ├── zones.rs              # Zone containers (Library, Hand, Battlefield, etc.)
│   │   ├── player.rs             # PlayerState (life, mana pool, counters)
│   │   └── battlefield.rs        # Battlefield-specific state (tapped, counters, attachments)
│   │
│   ├── engine/                   # The rules engine — reads and mutates GameState
│   │   ├── mod.rs
│   │   ├── actions.rs            # Player actions (cast spell, activate ability, play land)
│   │   ├── turns.rs              # Turn structure / phase progression
│   │   ├── priority.rs           # Priority system
│   │   ├── stack.rs              # Stack resolution
│   │   ├── combat.rs             # Combat system
│   │   ├── sba.rs                # State-based actions
│   │   ├── layers.rs             # Continuous effect layer system (rule 613)
│   │   └── zones.rs              # Zone transition logic (centralized, not per-object)
│   │
│   ├── effects/                  # Effect system — what cards DO
│   │   ├── mod.rs
│   │   ├── effect.rs             # Effect trait + one-shot effects
│   │   ├── continuous.rs         # Continuous effects (applied via layer system)
│   │   ├── replacement.rs        # Replacement effects
│   │   ├── triggered.rs          # Triggered ability definitions
│   │   └── costs.rs              # Cost definitions and payment
│   │
│   ├── targeting/                # Targeting system (port + refine from v1)
│   │   ├── mod.rs
│   │   ├── criteria.rs           # TargetCriteria (port the And/Or/Not composition)
│   │   └── resolution.rs         # Target validation and legality checks
│   │
│   ├── cards/                    # Card definitions — DATA ONLY
│   │   ├── mod.rs
│   │   ├── registry.rs           # Card registry (name → CardData)
│   │   ├── sets/                 # Organized by set for contributor clarity
│   │   │   ├── core.rs           # Basic lands, simple creatures
│   │   │   ├── alpha.rs          # Lightning Bolt, etc.
│   │   │   └── ...
│   │   └── helpers.rs            # Builder pattern for card definitions
│   │
│   ├── events/                   # Event bus for triggered abilities + logging
│   │   ├── mod.rs
│   │   ├── event.rs              # GameEvent enum
│   │   └── bus.rs                # Event dispatch and listener registration
│   │
│   └── ui/                       # Player interaction layer
│       ├── mod.rs
│       ├── decision.rs           # DecisionProvider trait (port from v1)
│       ├── cli.rs                # CLI implementation
│       └── display.rs            # Game state display/formatting
│
├── tests/
│   ├── integration/              # Integration tests by feature area
│   │   ├── combat.rs
│   │   ├── casting.rs
│   │   ├── mana.rs
│   │   └── ...
│   └── cards/                    # Per-card regression tests
│       ├── lightning_bolt.rs
│       └── ...
│
└── cards/                        # (future) External card data files (TOML/JSON)