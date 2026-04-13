# Simulator Roadmap: Post-Phase 4.5 → Fully Functional

> Generated: 2026-04-01
> Informed by: comprehensive rules audit (sessions 6a/6b), implementation plan, design doc, effect system plan
> Status: Post-Phase 4.5, pre-Phase 5

---

## Where We Are

### Completed Phases

| Phase | Scope | Completed |
|-------|-------|-----------|
| **1** | Types, GameState, zones, turn structure, mana, priority, basic lands | 2026-03-28 |
| **2** | Stack, casting, spell resolution, one-shot effects (Bolt, Recall, Counterspell) | 2026-03-29 |
| **3** | Creatures, full combat system, SBAs for lethal damage / zero toughness / player loss | 2026-03-29 |
| **4** | 10 keyword abilities (flying, reach, defender, haste, vigilance, first/double strike, trample, lifelink, deathtouch) | 2026-03-29 |
| **4.5** | Oracle module, CLI + Random DecisionProviders, fuzz harness, CLI play binary | 2026-03-30 |

### By the Numbers

- **Tests:** 370 (312 unit + 48 integration + 1 doc-test + 9 pre-Phase3), zero warnings *(updated 2026-04-12)*
- **Fuzz:** 200/200 games pass (Random vs Random), zero errors/panics, ~32 spells/game, ~10 combats w/ attackers/game *(updated 2026-04-13)*
- **Cards:** 24 (5 basic lands, 5 spells, 4 vanilla creatures, 11 keyword creatures)
- **DecisionProvider methods:** 9 (all implemented for CLI, Random, Scripted, Passive, Dispatch) *(+choose_legend_to_keep from T14)*
- **Effect primitives implemented:** 9 of ~35 (DealDamage, DrawCards, GainLife, LoseLife, ProduceMana, CounterSpell, CounterAbility, Destroy, Untap)
- **Phase 5 Pre-Work tickets completed:** T14, T15, T15b, T16, plus TargetSpec→EffectRecipient refactor

### What Works End-to-End Today

A two-player game can be played from setup to completion via CLI or fuzz harness:
- Shuffle, draw opening hands (mulligan stubbed — always keep)
- Full turn structure (untap, upkeep, draw, main 1, combat, main 2, end, cleanup)
- Play lands, tap for mana, cast instants/sorceries/creatures
- Full priority system with SBA loop
- Declare attackers/blockers with keyword evasion, first/double strike damage steps, trample assignment
- Lifelink, deathtouch, damage-based creature death, player life loss → game over
- Stack with targeting, fizzle on illegal targets, counterspells
- Cleanup: discard to hand size, remove damage

### What Does NOT Work Yet

- No continuous effects (no +1/+1 buffs, no anthems, no control change)
- No triggered abilities (no ETB/death/upkeep triggers)
- No replacement effects (no "enters tapped", no damage prevention, no "instead" effects)
- No enchantments, planeswalkers, artifacts with static abilities
- No alternative/additional costs (kicker, flashback, etc.)
- No tokens
- No counters on permanents
- No hexproof/shroud/protection enforcement in targeting

---

## Definition of "Fully Functional"

### Tier 1: Core Rules Complete (v1.0 target)

A Standard-legal two-player game can be played correctly for any combination of implemented cards. All comprehensive rules that govern the interaction of those cards are implemented, not approximated. Specifically:

- All 7 layers of the continuous effects system (rule 613) with dependency detection
- Triggered abilities (rule 603) with correct APNAP ordering and delayed triggers
- Replacement effects (rules 614–616) with correct ordering and self-referential loop prevention
- All state-based actions (rule 704)
- Complete casting pipeline (rule 601.2a–i) with alternative/additional costs, cost modification
- Full targeting with hexproof, shroud, protection
- Token creation and cease-to-exist
- Counter manipulation (all types)
- Last Known Information
- All 10 currently-implemented keywords + at least 10 more (menace, infect, hexproof, protection, indestructible, etc.)

### Tier 2: Format Support

- **Standard/Pioneer/Modern:** Identical rules engine; format = card pool + deck constraints via `GameConfig`
- **Commander:** Command zone, commander tax, commander damage, color identity, 40 life, singleton. Requires `Format` trait migration from `GameConfig`.
- **Limited (Draft/Sealed):** Already supported by `GameConfig::limited()`. Needs draft/sealed card pool generation.

### Tier 3: Card Coverage

- **Core set:** ~50–100 cards spanning all card types and major keywords
- **Competitive staples:** Top 50 most-played Standard cards
- **Stress-test cards:** Humility, Opalescence, Blood Moon, Tarmogoyf (layer system validation)

### Tier 4: Polish

- TUI or web UI
- Network play
- AI beyond random (MCTS or similar)
- Performance optimization (layer caching, parallel fuzz)

---

## Revised Phase Plan

### Phase 5: Pre-Work Engine Fixes

**Goal:** Close all engine gaps identified by the comprehensive rules audit that do NOT require the layer system.

**Key deliverables:**
- Counters on permanents (`HashMap<CounterType, u32>`) + expanded `CounterType`
- Player counters (poison, commander damage)
- `is_token` / `is_copy` flags, attachment tracking (`attached_to` / `attached_by`)
- Summoning sickness rework (`controller_since_turn` replaces boolean)
- 9 new SBAs: counter annihilation, token cease-to-exist, legend rule, planeswalker loyalty, aura/equipment legality, poison, commander damage, indestructible guard, cleanup re-loop
- Alternative/additional cost framework + 601.2-compliant casting pipeline
- Activation restrictions, zone-activated abilities, linked abilities
- Infect/wither/toxic damage routing, planeswalker damage routing
- Duration variants + expiry hooks
- Hexproof, shroud, protection in targeting
- Evasion expansion (menace, shadow, fear, intimidate, skulk, landwalk)
- Combat requirements solver (508.1d / 509.1c)
- Mana spending restrictions design spike

**Ticket count:** 24 active (T01–T22, T15b, T21a–T21d). Scope: 10 Small, 10 Medium, 4 Large.

**Cards unblocked:** None directly (infrastructure phase), but unblocks all future card types: planeswalkers, auras, equipment, token producers, infect/toxic creatures, legendary permanents.

**Infrastructure added:**
- `CastInfo` carried from stack to permanent (enables "if kicked" / "if evoked")
- `enchant_filter: Option<SelectionFilter>` on `CardData` for aura attachment validation (unified with `validate_selection` — no separate `EnchantRestriction` type)
- `ActivationRestriction` + `activation_zone` on `AbilityDef`
- `CastingRestriction` on `CardData`
- `ProtectionQuality` enum
- Cost modification pipeline stub (passthrough, ready for Phase 5 layer)
- Duration expiry hook sites in `turns.rs`
- `get_effective_lands_per_turn` oracle function (passthrough, computed in Phase 5 layer)

**Risk/complexity:** Medium. Largest tickets are T18 (601.2 casting pipeline — consider splitting into T18a/b/c) and T21b (combat evasion + requirements). Most tickets are isolated data model additions with localized blast radius. The mana spending restrictions design spike (T12) produces a document, not code.

**Estimated test count after completion:** ~470+ (370 current as of 2026-04-12 + ~100 from remaining ~20 tickets)

---

### Phase 5: Continuous Effects & Layer System

**Goal:** Implement the full 7-layer continuous effects system (rule 613) so that characteristics of all game objects are computed correctly through layers, dependency detection, and timestamp ordering.

**Key deliverables:**
- `ContinuousEffect` struct, `Layer` enum, `Modification` enum, `AppliesTo` enum
- `EffectiveCharacteristics` struct computed on-demand via `compute_characteristics()`
- All 7 layers: Copy (stub), Control, Text (tree-walker), Type, Color, Ability, P/T (4 sublayers)
- Dependency detection with iterative 613.8b algorithm (all four 613.8a conditions)
- CDA handling in all zones (604.3)
- Static ability registration on ETB, removal on zone exit
- Duration tracking with cleanup hooks
- APNAP timestamp sub-ordering
- Locked-in target sets (613.6) and 611.2c spell lock-in
- Oracle routing migration (all `card_data` reads → `compute_characteristics`)
- Last Known Information system
- Post-layer pass: player action restrictions, cost modification scaffolding, `lands_per_turn`
- All-zone static ability field on `AbilityDef`

**Ticket count:** 21 (L01–L21). Scope: 4 Small, 11 Medium, 6 Large.

**Cards unblocked (11 new, 35 total):**
- **Tier 1:** Giant Growth, Glorious Anthem, Honor of the Pure, Tarmogoyf, Urborg, Blood Moon, Mind Snare
- **Tier 2:** Humility, Opalescence
- All anthem/lord effects, type-changing effects, control-changing effects, P/T setting/switching

**Infrastructure added:**
- `engine/layers.rs` — core layer computation engine
- `engine/dependency.rs` — dependency detection (Kahn’s topological sort) with cycle fallback to timestamp ordering
- `types/continuous.rs` — all continuous effect types, including:
  - `ContinuousEffectKind` enum: `CharacteristicModifying` (L1–L7), `GameRuleModifying` (613.11, post-layer), `CostModification { kind: CostModKind }` (601.2f pipeline)
  - `Applicability` enum: `SelfRef`, `ObjectFilter(ObjectFilter)`, `PlayerFilter(PlayerFilter)`, `EventFilter(GameActionPattern)` — cross-cutting type reused in Phases 6–7
  - `AbilityOrigin` enum: `Intrinsic`, `Granted { source_id: ObjectId, effect_timestamp: Timestamp }` — enables L3 text-changing to skip granted abilities
  - `Timestamp` struct: `{ global_seq: u64, sub_index: u16 }` — composite for APNAP ordering + intra-object sub-ordering
- Layer-aware oracle: every characteristic query routes through layers
- LKI snapshots wrapping `EffectiveCharacteristics`
- Player action restriction + cost modification scaffolding

**Risk/complexity:** **High.** This is the most architecturally complex phase. The dependency detection algorithm (613.8) requires iterative rebuild after each effect application. Blood Moon + Urborg interaction is the canonical stress test. Opalescence + Humility with both timestamp orders is the hardest correctness test.

**Estimated test count after completion:** ~500+ (420 + ~84 new)

**Verification gates:**
- **Gate 3:** Giant Growth on Bears = 5/5, reverts at cleanup
- **Gate 4:** Blood Moon + Urborg dependency correct, all 7 layers functional
- **Gate 5:** Opalescence + Humility correct for both timestamp orders, 500+ fuzz games pass

---

### Phase 6: Replacement Effects

**Goal:** "If X would happen, instead Y" and "prevent N damage" (rules 614–616). Also completes Layer 1 (copy effects) and expands Layer 3 (text-changing on permanents).

**Rationale for ordering (replacement before triggers):** The Phase 5 plan (and session 6b correction PC3) recommends swapping the original design_doc ordering (6=Triggers, 7=Replacement) so that replacement effects come first. **This roadmap adopts the swap.** Justification:

1. **ETB replacement effects are ubiquitous.** "Enters the battlefield tapped" (taplands), "enters with N +1/+1 counters" (hydras), "as this enters, choose a creature" (Clone) — all require replacement effects. Without them, even simple taplands are incorrect.
2. **Layer 1 (copy effects) is replacement-dominated.** Clone's "as this enters the battlefield, you may have it become a copy of" is a replacement effect. Completing L1 before triggers means triggered abilities see correct post-copy characteristics.
3. **Debugging is easier.** If triggers fire based on incorrect intermediate characteristics (because replacement effects haven't been applied yet), the bugs are extremely hard to trace. Replacement effects ensure state is correct *before* triggers observe it.

**Key deliverables:**
- `ReplacementEffect` struct with condition, replacement, source, duration, controller
- `apply_replacement_effects()` inserted into `execute_action()` (hook already designed)
- Rule 616 ordering (affected player/controller chooses when multiple replacements apply)
- Self-referential loop prevention (rule 614.5)
- Prevention effects as a subtype (rule 615)
- Layer 1 copy effects (Clone, Sakashima, "becomes a copy of")
- Copiable values definition (rule 707.2)
- "Enters the battlefield tapped" / "enters with counters" replacements
- "If this would die, instead..." (regeneration shields, totem armor)
- Damage prevention ("prevent the next N damage", Fog)
- "If you would draw a card, instead..." (replacement draw effects)

**`Effect::Simultaneous` vs `Effect::Sequence` distinction:** Today, `Effect::Sequence` is used for both "X and Y" (simultaneous instructions, e.g. Night's Whisper: "You draw two cards and lose 2 life") and "X. Then Y" (sequential instructions, e.g. "Draw two cards. Then each player loses two life"). The engine treats both identically — correct for now, since SBAs only fire after full spell resolution regardless. When replacement effects arrive, the distinction becomes observable: a replacement on "whenever you would draw" applies differently to a simultaneous event (whole event is one unit) vs sequential instructions (each instruction is a separate event). At that point, add `Effect::Simultaneous(Vec<Effect>)` and migrate cards accordingly. Test cards in test/definition files will likely be rewritten into proper set folders anyway, so the refactor cost is bounded.

**Estimated ticket count:** ~30–35

**Cards unblocked:**
- All taplands (Evolving Wilds, gain lands, shock lands)
- Clone, Sakashima, copy effects
- Fog, Holy Day (prevention)
- Hydras with "enters with X +1/+1 counters"
- Shield counters
- Regeneration
- "Dies" replacement effects (undying, persist, indestructible already handled in SBA)

**Infrastructure added:**
- Replacement effect registry on `GameState`
- `execute_action` middleware chain: `execute_action → apply_replacement_effects → perform_action`
- Unified replacement/prevention pipeline with priority ordering: self-replacements → control-changing → copy-becoming → others (see `session-6-audit-response.md`)
- Event decomposition pattern: outer events (e.g., `CreateTokens`) are processed first, then inner events (e.g., `EnterBattlefield`) are created and processed recursively — no nested event structures
- ETB look-ahead: "phantom" battlefield entry for evaluating a permanent’s characteristics before it actually enters, filtered by `Applicability` (from Phase 5)
- Copy effect infrastructure for Layer 1
- Prevention shield tracking

**Retrofit note — zone-destination hardcoding:** Multiple engine paths currently hardcode a destination zone that replacement effects could alter. When the `execute_action → apply_replacement_effects → perform_action` middleware lands, these must all route through it:
- **SBA creature death (704.5f/g):** `sba.rs` calls `move_object(id, Zone::Graveyard)` — Rest in Peace would redirect to exile.
- **Spell resolution (608.3):** `stack.rs` sends instants/sorceries to graveyard, permanents to battlefield — Flashback/Escape would redirect to exile.
- **Counterspelling (701.6a):** `resolve.rs` sends countered spells to graveyard — self-replacement effects per rule 614.15 (e.g. Dissipate "exile instead") or global replacements (Rest in Peace) would redirect.
- **Stack resolution pop-first pattern:** The pop-first design in `stack.rs` (removing the spell from the stack Vec before resolution effects execute) remains correct — the replacement layer intercepts the *destination* step, not the removal step. Similarly, counter effects that remove by position are fine; only the subsequent zone move needs replacement awareness.
- All three cases share the same architecture: a known default destination (graveyard/battlefield) that replacement effects modify. No special-casing needed per case — the middleware intercepts `move_object` uniformly.

**Risk/complexity:** **High.** Replacement effects interact with nearly every state mutation. The ordering rule (616) requires player choice when multiple replacements apply to the same event. Self-referential loop prevention adds bookkeeping. Copy effects are notoriously complex (copiable values, "as enters" timing, copy of copy).

---

### Phase 7: Triggered Abilities

**Goal:** "When/Whenever/At [event], [effect]" abilities that go on the stack (rule 603).

**Key deliverables:**
- `TriggerCondition` enum: `OnETB`, `OnLTB`, `OnDeath`, `OnDamageDealt`, `OnLifeGained`, `OnSpellCast`, `AtBeginningOf(StepType)`, `OnAttack`, `OnBlock`, etc.
- `TriggeredAbilityDef` on `AbilityDef`: trigger condition + effect
- Trigger checking: after each event batch / SBA cycle, scan for matching triggers
- Stack placement in APNAP order (rule 603.3b)
- Delayed triggered abilities (rule 603.7) — created by spells/abilities, fire once
- "Intervening if" conditions (rule 603.4)
- State triggers (rule 603.8) — "when you have 10+ poison counters" etc.
- Leaves-the-battlefield triggers using LKI
- "Whenever a creature enters the battlefield" (ETB triggers)
- "Whenever a creature dies" (death triggers)
- "At the beginning of your upkeep" triggers

**Estimated ticket count:** ~25–30

**Cards unblocked:**
- Soul Warden ("whenever a creature enters, gain 1 life")
- Blood Artist ("whenever a creature dies, drain 1")
- Mulldrifter (ETB draw, evoke)
- Oblivion Ring / Banisher Priest (exile-until-leaves, requires linked abilities from E30 + delayed triggers)
- Upkeep trigger creatures
- "Whenever you cast a spell" (prowess, magecraft)
- "Whenever this creature attacks" triggers
- Storm (spell copying + trigger)
- Sagas (chapter triggers + lore counters)

**Infrastructure added:**
- **Delta log** (`engine/delta_log.rs`): game-lifetime log of structured `GameDelta` entries emitted by all state-mutating methods (`move_object`, `perform_action`, etc.). Each entry records `(old, new)` pairs for the mutation. This is the canonical source of truth for trigger detection — NOT the diagnostic `EventLog`, which remains a UI/debug artifact (see design_doc.md §8 and §11 for rationale). The delta log also serves loop detection (Tier 1 forced-action counter, Tier 2 full-state hashing) and voluntary shortcut validation (D26).
- **Trigger scanner** drains delta log entries at well-defined checkpoints (after SBA cycles, after event batches). Matches deltas against registered `TriggerKind` patterns:
  - `EventDriven { watches_for: GameActionPattern }` — fires when a matching delta appears
  - `StateBased { condition: TriggerCondition }` — fires when deltas indicate a condition transitioned from false→true (solves the "momentarily empty hand" problem without per-mutation polling)
  - **Zone-agnostic constraint:** The trigger scanner must scan triggers on objects in ALL zones, not just battlefield permanents. Many paper MTG cards trigger from non-battlefield zones: graveyard (Bloodghast, Narcomoeba), command zone (emblems), exile (suspend). Alchemy boons also trigger from the command zone. The trigger registration index (`HashMap<TriggerKind, Vec<ObjectId>>`) must include objects regardless of zone. See `alchemy-mechanics-audit.md` Q7 for full analysis.
- Trigger queue with APNAP ordering
- Delayed trigger registry
- `perform_sba_and_triggers()` in priority loop (stub already exists)
- Per-turn tracker system (storm count, spells cast, etc.)
- `GameNumber` type stub (`type GameNumber = u64` initially) + trigger iteration cap (`MAX_TRIGGER_ITERATIONS`) for divergent loop safety. `DecisionProvider::declare_loop_count()` method returning `u64` for now. Full symbolic `GameNumber` enum deferred to Phase 9.

> **Architecture change (2026-04-06):** The original `pending_triggers: Vec<PendingTrigger>` push-based design has been replaced by the delta log approach from `state-tracking-architecture.md`. Rationale: push-based triggers require every mutation site to know about every trigger condition, making state-based triggers (rule 603.8) impractical without O(state_triggers × mutations) polling. The delta log centralizes detection: mutation sites emit generic deltas, and a single scanner runs pattern matching at checkpoints. This also unifies trigger detection with loop detection and shortcut validation, avoiding three separate observation mechanisms.

**Risk/complexity:** **High.** Trigger ordering (APNAP) is tricky in multiplayer. Interaction between triggers, SBAs, and replacement effects creates a complex event loop. Delayed triggers add lifecycle management. "Dies" triggers need LKI to see the creature's characteristics as it last existed.

---

### Phase 8: Remaining Primitives, Keywords & Card Breadth

**Goal:** Implement the ~20 remaining effect primitives, expand keyword coverage to 30+, and bring card count to 100+.

**Key deliverables:**
- Remaining primitives: Exile, ReturnToHand, ReturnToBattlefield, ShuffleIntoLibrary, Mill, Discard, Scry, Surveil, Search, Reveal, AddCounters, RemoveCounters, Proliferate, CreateToken, Fight, Sacrifice, Explore, Connive, Amass, Transform
- Remaining combinators: Optional ("you may"), Modal ("choose one"), ForEach, Repeat
- Token creation pipeline (CreateToken primitive + token factory)
- New keywords: menace (enforcement from T21b), hexproof (enforcement from T22), protection (full — blocking, damage prevention, attachment), ward, flash, equip, enchant, prowess, convoke, delve, escape, flashback, kicker (using alt/add cost framework from T17/T18)
- Card breadth: 50+ new cards across all types
- Pregame actions (rule 103.6): additive hook in `Game::setup()` after mulligans for "begin the game with on battlefield" effects (Leylines). Uses `controller_since_turn = 0` sentinel from T09 — no retrofit needed.

**Builder follow-up:** `CardDataBuilder::mana_ability_single` only handles "tap: add one mana of type X." Generalize to accept a `ManaOutput` directly when implementing Sol Ring, multi-mana lands, or choice-of-color producers (e.g. `mana_ability(costs, ManaOutput)`).

**Estimated ticket count:** ~40–50

**Cards unblocked:**
- Token generators (Raise the Alarm, Lingering Souls)
- Equipment (Sword of Fire and Ice, Bonesplitter)
- Auras (Pacifism, Rancor)
- Planeswalkers (basic loyalty abilities)
- Modal spells (Charm cycles)
- Flashback/Kicker cards
- Search effects (fetch lands, tutors)
- Mill cards, discard effects
- Ritual spells (Dark Ritual, Pyretic Ritual) — `ProduceMana` already works on spell abilities, just needs a card definition
- Leyline cycle (pregame actions via D27)

**Risk/complexity:** Medium individually (most primitives are straightforward zone movements or counter manipulation), but **volume is high**. The risk is in integration — each new primitive must interact correctly with replacement effects, triggered abilities, and the layer system.

---

### Phase 9: Format Support & Advanced Systems

**Goal:** Commander format, advanced game mechanics, and the remaining deferred items.

**Key deliverables:**
- `Format` trait: `config()`, `validate_decklist()`, `setup_game()`, `check_win_condition()`
- Commander: command zone, commander tax, commander damage tracking (data model from T02), color identity validation, 40 life, singleton rule, partner/companion
- Mulligan implementation (London mulligan — currently stubbed)
- Extra turns/phases/steps (mutable `TurnPlan` replacing fixed state machine)
- Phasing (502.1 — interacts with layers, combat, continuous effects)
- Face-down permanents (morph, manifest, disguise)
- Double-faced cards (transform, daybound/nightbound, MDFCs)
- Split cards, adventure, aftermath
- Day/Night global designation
- Monarch/Initiative designations
- Perpetual/Alchemy mechanics (see below)
- Prototype (702.160) — `PrototypeStats` on `CardData` + `cast_as_prototype: bool` via `CastInfo` (zone-sidecar scoped, stripped on zone change). See `implementation-plan-final.md` D20a.
- `GameNumber` full implementation: `Finite(u64)` / `Shortcut { id, iterations, per_iteration }` / `Relative { base, multiplier, offset }` enum with `PartialOrd`. Enables representing and comparing arbitrarily large quantities from shortcut loops (rule 727). `LoopDeclaration` type for `DecisionProvider::declare_loop_count()` supporting both concrete values and "match+N" relative declarations. Promotes the Phase 7 `u64` stub to the full compositional type.

**Estimated ticket count:** ~30–40

**Perpetual/Alchemy mechanics:** Alchemy-originated effects that modify characteristics permanently across zone changes. NOT in the Comprehensive Rules — will require ad-hoc testing. See `alchemy-mechanics-audit.md` for detailed per-mechanic analysis. Key patterns:
- **"Perpetually gets +N/+M"** — P/T delta that persists across zones.
- **"Power perpetually becomes 0"** — P/T set that persists across zones. Distinct from delta — order matters.
- **"Perpetually gains [ability]"** / **"Perpetually loses [ability]"** — ability grant/removal that persists across zones.
- **"Perpetually becomes [color/type]"** — color/type set that persists across zones.
- **Two distinct perpetual cost-change patterns:** (1) `SetManaCost` for "mana cost perpetually becomes {X}" (changes mana value), (2) `AddAbility` for "perpetually gains 'costs {X} more to cast'" (granted static ability feeding cost pipeline, does NOT change mana value). Incorporate uses pattern 2.
- **Conjure / Seek** — create cards from outside the game / random library search. Orthogonal to perpetual but part of the Alchemy scope. Requires `Arc<CardRegistry>` on `GameState` (scaffolded in Phase 5-Pre).
- **Draft / Spellbook** — present N random cards from a pool, player picks 1, conjure the pick. Two pool types: small inline `Vec<String>` on `CardData` (spellbooks, up to 15 cards), large pre-shuffled `DraftPool` on `GameState` (Booster Tutor / cube pools, 540+ cards with cursor for no-repeat sequential pulls). New DP method: `choose_draft_pick`.
- **Intensity** — standalone `Option<u32>` field on `GameObject` (NOT a PerpetualMod). Read as a value by abilities, incremented by "intensify" actions. Initialized from `CardData.starting_intensity`.
- **Boon** — reuses emblem infrastructure (command-zone `GameObject` with triggered ability). Extend emblem sidecar with `uses_remaining: Option<u32>`. No separate BoonState needed.
- **Specialize** — `PerpetualMod::ReplaceCardData(Arc<CardData>)`. Full card identity replacement that persists across zones. Independent of DFC face system.
- **New `DecisionProvider` methods:** `choose_draft_pick`, `choose_heist_card`, `choose_specialize_color`.

Perpetual modifications are **ordered and heterogeneous** — a card can accumulate multiple perpetual effects mixing set and modify operations. Order matters: "power becomes 0" then "+1/+2" ≠ "+1/+2" then "power becomes 0". Implementation: `perpetual_modifications: Vec<PerpetualMod>` on `GameObject`, applied in order by `compute_characteristics()` after CardData base but before the layer loop. `PerpetualMod` is an enum covering SetPower, ModifyPower, SetColors, AddAbility, RemoveAbility, RemoveAllAbilities, etc. — extended with ~3-4 new variants for Alchemy: `AddColor`, `ReplaceCardData`, and keyword-specific variants if keywords/abilities aren't unified. This is architecturally **separate** from Prototype (which uses a bool + static CardData field). See `implementation-plan-final.md` D20b and `session-8-audit-response.md` Round 2 for background.

**Risk/complexity:** Medium-High. Commander format is a significant behavioral surface area. DFCs require `back_face: Option<CardData>` restructuring. Face-down permanents need a default 2/2 characteristics system. Extra turns require replacing the fixed turn state machine. Perpetual mechanics require `compute_characteristics()` to replay the perpetual modification log before layer application — straightforward once the layer system exists. `GameNumber` is isolated and well-scoped — the comparison semantics are the only tricky part (incomparable values from unrelated loops, resolved by APNAP sequential declaration ordering).

---

### Phase 10: UI, AI & Performance

**Goal:** Move beyond CLI to a usable play experience.

**Key deliverables:**
- Web GUI (Wasm + framework TBD) — targeting a middle ground between XMage (functional but ugly) and Arena (beautiful but slow). TUI rejected: MtG's complexity makes graphical presentation significantly easier for humans.
- AI engine interacting via a well-defined API layer over `DecisionProvider` — the same trait that drives CLI/Random/Scripted, exposed as a structured API (REST/gRPC/direct Rust call) so AI implementations can be developed independently without engine coupling.
- Network play (stretch goal) — WebSocket for real-time, protocol TBD.
- Layer system caching and performance optimization (profile-driven — see Performance Analysis appendix).
- Parallel fuzz testing.
- `DraftEngine` for Limited format — separate system managing draft pod state, card pool selection, and pick/pass flow. Post-v1.

**Estimated ticket count:** Varies by scope — 20+ per deliverable

**Risk/complexity:** Medium for GUI (state rendering complexity), Medium for network (state synchronization), High for AI (game tree complexity with hidden information). `DraftEngine` is Medium — isolated system with clear interface to the game engine.

---

## Phase Dependencies

### Serial Chain (Critical Path)

```
Phase 5 Pre-Work ──→ Phase 5 Layers ──→ Phase 6 Replacement ──→ Phase 7 Triggers
```

These four phases are strictly serial. Each builds on infrastructure from the previous:
- Pre-work provides data model (counters, attachments, summoning sickness rework) needed by layers
- Layers provide `compute_characteristics` + `EffectiveCharacteristics` needed by replacement effects (LKI, copy)
- Replacement effects provide the `execute_action` middleware needed for correct trigger observation
- Triggers require correct state (post-replacement) to fire accurately

### Parallelizable Work

| Work | Can Parallel With | Notes |
|------|-------------------|-------|
| Phase 5 Pre-Work Tiers 3–5 (SBAs, casting, combat) | Phase 5 Pre-Work Tiers 1–2 | After data model is in place |
| Phase 8 keyword/primitive stubs | Phase 6–7 | Primitive *types* can be defined; *resolution* needs replacement/trigger infra |
| Phase 9 format support (Commander data model) | Phase 7 | Commander damage tracking (T02) already in pre-work; format trait is independent |
| Phase 10 TUI/Web UI | Phase 7+ | UI is presentation over `DecisionProvider`; can start once game is playable |

### Deferred Items → Phase Mapping

All deferred items from both audit documents are mapped below. None are orphaned.

#### From Session 6a (D1–D25)

| D# | Item | Target Phase | Notes |
|----|------|-------------|-------|
| D1 | Phasing (502.1) | Phase 9 | Deep layer interaction; needs `phased_out: bool` |
| D2 | Face-down permanents (708) | Phase 9 | Morph/manifest/disguise system |
| D3 | Double-faced cards (712) | Phase 9 | `back_face: Option<CardData>` |
| D4 | Split card / Adventure / CardLayout | Phase 9 | `CardLayout` restructuring |
| D5 | Copy system (707) | **Phase 6** | Layer 1 completion — replacement effect dominated |
| D6 | Extra turns/phases/steps | Phase 9 | Mutable `TurnPlan` |
| D7 | Multiplayer systems | Phase 9 | Commander format support |
| D8 | Replacement effects on zone transitions (400.6) | **Phase 6** | Core replacement effect work |
| D9 | "Can't" overrides "can" (101.2) | Embedded | Design pattern, not a system. Already followed. |
| D10 | "Can't have" ability prohibition (113.11) | Phase 8 | L6 concept, implement with relevant cards |
| D11 | Mandatory loop detection (104.4b) | Post-v1 / stretch | Extremely niche |
| D12 | Mulligan implementation (103.5) | Phase 9 | Game setup polish |
| D13 | Regeneration shield system | **Phase 6** | Replacement effect |
| D14 | Day/Night global designation (730) | Phase 9 | When daybound cards arrive |
| D15 | Monarch/Initiative (724/725) | **Phase 7** | Triggered abilities |
| D16 | Per-turn tracker system | **Phase 7** | Storm count, spells-cast-this-turn |
| D17 | Per-permanent designations | Phase 8 | Monstrous, exerted, goaded, etc. |
| D18 | Multi-name / "choose a card name" | Phase 8 | Pithing Needle, Meddling Mage |
| D19 | Spell copying (Storm, Replicate) | **Phase 7** | Requires triggered abilities + copy-on-stack |
| D20 | Companion — outside-the-game zone | Post-v1 / stretch | Niche mechanic |
| D21 | Exile zone metadata | Phase 8 | Face-down exile, exiled-by, play permissions |
| D22 | Excess damage redirection event metadata | **Phase 7** | Toralf-style triggers |
| D23 | "Can't gain life" / "Can't lose life" | Phase 8 | Prohibition effects |
| D24 | Player-leaves-game cleanup | Phase 9 | Multiplayer only |
| D25 | Land+other-type casting restriction (300.2a) | Phase 8 | One guard in `cast.rs` |
| D26 | Divergent loop shortcutting + `GameNumber` (rule 727) | **Phase 7** (stub) / **Phase 9** (full) | Trigger iteration cap in Phase 7; full symbolic `GameNumber` enum with `Relative` support in Phase 9. Needed for Astral Dragon + Parallel Lives class combos and infinity-vs-infinity races. |
| D27 | Pregame actions (103.6) | Phase 8 | "Begin the game with on battlefield" (Leylines). Additive hook in `Game::setup()` after mulligans: check opening hands, ask DP, place onto battlefield with `controller_since_turn = 0` (sentinel, see T09). No architectural retrofit — purely additive. Summoning sickness interaction correct by design: pregame permanents that later become creatures (e.g. via Opalescence) pass the `controller_since_turn >= turn_number` check since `0 >= 1` is `false`. |

#### From Session 6b (D1–D17)

| D# | Item | Target Phase | Notes |
|----|------|-------------|-------|
| D1 | Layer 1 (Copy Effects) | **Phase 6** | Same as 6a-D5 |
| D2 | Layer 3 (Text-Changing) — semantic tree-walker | **Phase 5 Layers (L12)** | Locked in — implemented in Phase 5 |
| D3 | Face-Down/Transform timestamps (613.7f–g) | Phase 9 | When face-down/transform cards arrive |
| D4 | Aura/Equipment re-timestamp on attach (613.7e) | Phase 8 | When Aura/Equipment cards arrive |
| D5 | Static ability timestamp = later of object vs. grant (613.7a) | Phase 8 | When grant-then-generate cards arrive |
| D6 | Counter timestamps within L7c (613.7c) | Post-v1 / stretch | Extremely rare relevance |
| D7 | "For as long as" duration failure (611.2b) | Phase 8 | Edge case |
| D8 | Deferred continuous effects — "next spell" (611.2f) | **Phase 7** | Requires triggered ability infra |
| D9 | "Until" zone-change effects — O-Ring pattern (610.3) | **Phase 7** | Delayed triggers + linked abilities |
| D10 | Keyword counters in Layer 6 | Phase 8 | Small L6 addition when cards arrive |
| D11 | Phasing interaction with layers | Phase 9 | Same as 6a-D1 |
| D12 | Bestow dual-nature permanent | Phase 8 | Requires Aura infra + L4 type-changing |
| D13 | Crew/Saddle type-changing abilities | Phase 8 | Vehicle/Mount cards |
| D14 | Devotion uses partial-layer result | Phase 8 | Theros gods + Nykthos |
| D15 | Exchange involving P/T | Phase 8 | L7b Set + L7d Switch |
| D16 | Continuous effects on stack (611.2a) | Post-v1 / stretch | Very rare |
| D17 | Cast legality look-ahead + flash grants (601.3) | Phase 8 | Oracle routing enables this naturally |

#### Cross-Cutting Items

These items span multiple phases:

| Item | Phases Involved | Notes |
|------|----------------|-------|
| **Copy system (707)** | Phase 5 (L1 stub), Phase 6 (full L1 + replacement), Phase 7 (spell copying/Storm) | Three-phase rollout |
| **Phasing (502.1)** | Phase 5 (layer interaction), Phase 9 (full implementation) | Layers must skip phased-out sources |
| **"Can't" overrides "can"** | All phases | Design principle, not a deliverable |
| **Perpetual/Alchemy** | Phase 5-Pre (scaffold: `Arc<CardRegistry>` on GameState), Phase 7 (zone-agnostic trigger scanner), Phase 9 (full implementation) | Prototype: `PrototypeStats` on CardData + bool on CastInfo (D20a). Perpetual: `Vec<PerpetualMod>` on GameObject (D20b). Intensity: standalone `Option<u32>` on GameObject. Boon: emblem + use counter. Draft pool: `DraftPool` on GameState. See `alchemy-mechanics-audit.md` for full per-mechanic analysis. |
| **Protection** | Phase 5 Pre-Work (targeting), Phase 6 (damage prevention), Phase 8 (blocking, attachment) | Three-aspect keyword |
| **Aura/Equipment** | Phase 5 Pre-Work (SBAs, attachment tracking, `enchant_filter` + `validate_selection`), Phase 6 (ETB attachment replacement), Phase 8 (equip/enchant abilities) | Progressive build-out. T15b completed: `enchant_filter: Option<SelectionFilter>` on CardData, unified validation via `validate_selection`, `has_any_legal_choice` pre-check, `attach_aura_on_etb` helper. |
| **Divergent loop shortcutting (727)** | Phase 7 (iteration cap + `GameNumber` stub), Phase 9 (full `GameNumber` enum + `LoopDeclaration` + `Relative` comparisons) | Safety cap first, expressive math later |

---

## Fuzz Harness Upgrade Roadmap

> Added 2026-04-13. Tracks planned improvements to `bin/fuzz_games.rs` and `ui/random.rs`.

### Current State

- **Deck generation:** 60-card color-coherent decks (1–2 colors, 24 lands / 36 nonlands from matching colors)
- **RandomDecisionProvider:** Phase-aware cast probability (80% main / 30% non-main), 100% land play, 50% attack chance per legal attacker
- **Target legality:** `castable_spells` checks `has_any_legal_choice` before allowing targeted spells; `choose_targets` respects `PermanentFilter` and excludes self from `Spell` targets
- **Reproducibility:** `--seed` flag derives per-game seeds from a master RNG
- **Observability:** `--dump-events` writes full event logs; per-game action stats (spells cast, lands played, combat w/ attackers, creatures died, damage events, total damage, life changes) with aggregate averages
- **Scale:** 200/200 games, 0 errors, ~32 spells/game, ~10 combats w/ attackers/game

### Planned Upgrades

#### Deck Generation Improvements
- **3-color decks:** Allow 3-color combinations (currently 1–2). Adjust land base to produce all three colors reliably (e.g., 8/8/8 split for 3-color). Gate on having enough cards in the pool to make 3-color decks viable.
- **Mana curve awareness:** Weight nonland selection toward a reasonable mana curve (more 2-drops than 5-drops) instead of uniform random. Prevents games where a player draws only expensive spells.
- **Singleton mode:** Option to generate singleton (highlander) decks for wider card coverage per game.
- **New-card weighting:** When new cards are added to the registry, bias deck generation toward including at least 1–2 copies of each new card. Ensures fuzz games exercise newly implemented cards rather than replaying games that could have been run before the addition. Could be driven by a `--new-cards` flag or a `CardRegistry` "recently added" tag.
- **Scaling with card pool:** As new cards are registered (Phase 5 Layers Tier 1/2, Phase 6+), auto-include them. Currently requires manual `registry.register()` calls.

#### Event Log Overhaul — Delta Log
- **Structured delta log:** Replace the current flat `GameEvent` text dump with a structured delta log that records the *diff* between game states at each event. Each entry captures zone changes, life total deltas, battlefield additions/removals, stack changes, and mana pool deltas as structured data (not display strings).
- **Machine-readable format:** Output as JSON or similar structured format so external tools (visualization, AI training, regression diffing) can consume it without parsing human-readable text.
- **Snapshot checkpoints:** Periodically emit full game state snapshots (e.g., every N events or at phase boundaries) so a reader can reconstruct state at any point without replaying from event 0.
- **Regression diffing:** Given two seeds with the same deck, diff their delta logs to pinpoint exactly where a code change altered game behavior. Useful for verifying that refactors are behavior-preserving.

#### RandomDecisionProvider Improvements
- **Smarter blocking:** Currently doesn't block. Add heuristic blocking (e.g., block when a favorable trade exists, or randomly with some probability).
- **Mulligan decisions:** Currently always keeps. Implement basic mulligan logic (e.g., mulligan hands with 0–1 or 6–7 lands).
- **Ability activation:** When activated abilities beyond mana abilities exist (Phase 5+), the DP needs to consider activating them during priority.
- **Enchantment/artifact awareness:** Phase-aware casting should also bias toward playing enchantments/artifacts during main phases when they're sorcery-speed.

#### Harness Infrastructure
- **Parallel fuzz:** Run games on multiple threads. `Game` is self-contained; needs only thread-safe seeding.
- **Regression test extraction:** Ability to "promote" a failing seed into a named integration test automatically.
- **Coverage tracking:** Track which card names were actually cast (not just in deck) across all games, to ensure every registered card gets exercised.
- **Error categorization:** When errors occur, categorize them (targeting error, mana error, SBA error, panic) and report counts by category.

---

## Milestones

### Milestone 1: Engine Audit Complete
**Trigger:** All 24 Phase 5 Pre-Work tickets merged
**Criteria:**
- ~470+ tests pass, 0 warnings
- 500/500 fuzz games pass
- All 48 E-items addressed (3 resolved in-place, 1 deferred to Phase 5 Layers)
- Counters, attachments, summoning sickness rework, hexproof/shroud/protection all functional

### Milestone 2: First Continuous Effect Resolves
**Trigger:** Phase 5 Layers Sub-Plan 5A complete (L01–L08)
**Criteria:**
- Giant Growth on Grizzly Bears = 5/5, reverts at cleanup
- Glorious Anthem: Bears = 3/3, two Anthems = 4/4, destroyed = reverts
- Oracle routes `get_effective_power` / `get_effective_toughness` through `compute_characteristics`

### Milestone 3: Full Layer System Operational
**Trigger:** Phase 5 Layers Sub-Plan 5B complete (L09–L16)
**Criteria:**
- Blood Moon makes nonbasics into Mountains (rule 305.7 baggage correct)
- Blood Moon + Urborg: dependency detection resolves correctly
- All oracle queries route through layers
- All direct `card_data` reads migrated

### Milestone 4: Layer System Stress-Tested
**Trigger:** Phase 5 Layers complete (all 21 tickets, L01–L21)
**Criteria:**
- Opalescence + Humility correct for both timestamp orders
- Tarmogoyf CDA correct in all zones
- Mind Snare control change + summoning sickness correct
- LKI snapshots capture effective characteristics
- 500+ fuzz games pass with Phase 5 cards in pool
- ~500+ total tests

### Milestone 5: First Replacement Effect Works
**Trigger:** Phase 6 core complete
**Criteria:**
- "Enters the battlefield tapped" works on a tapland
- Clone enters as a copy of a creature (Layer 1 functional)
- Fog prevents all combat damage for a turn
- `execute_action → apply_replacement_effects → perform_action` chain functional

### Milestone 6: First Triggered Ability Fires
**Trigger:** Phase 7 core complete
**Criteria:**
- Soul Warden gains 1 life when any creature enters
- Blood Artist drains on creature death
- "At the beginning of your upkeep" trigger fires correctly
- Triggers stack in APNAP order
- Delayed triggers fire once and are removed

### Milestone 7: Core Rules v1.0
**Trigger:** Phases 5–8 complete
**Criteria:**
- 100+ cards implemented
- All 7 layers, triggered abilities, replacement effects, all SBAs
- Full casting pipeline with alt/additional costs
- Token creation, counter manipulation, all common keywords
- All fuzz games pass with full card pool
- Any two implemented cards interact correctly per comprehensive rules

### Milestone 8: Commander Playable
**Trigger:** Phase 9 Commander support complete
**Criteria:**
- 4-player Commander game runs to completion
- Command zone, commander tax, commander damage, color identity all enforced
- `Format` trait correctly dispatches Commander vs Standard rules

---

## Open Questions & Decisions

### Format Support Scope — DECIDED
- **v1.0 target:** Standard two-player. Commander is Phase 9.
- **Limited (draft/sealed):** In scope, but post-v1. The rules engine doesn't change; Limited needs a separate `DraftEngine` to manage draft pod state (card pools, pick/pass flow, display). Sealed is simpler (random pool generation). Both slot into Phase 10.

### Card Coverage — DECIDED
- **v1.0 target:** ~100 cards spanning all types and major keywords. Cherry-pick competitive staples.
- **Manual card authoring enforced.** No Oracle JSON import. Two reasons:
  1. **Correctness.** Parsing raw text into structured `AbilityDef`/`Effect` trees is a hard NLP problem. Hand-authored definitions are provably correct and directly testable.
  2. **Community.** Manual card definitions are an ideal open-source contribution surface. MtG and CS have high player overlap — contributing card definitions is accessible, domain-enjoyable work that doesn't require deep engine knowledge. Lower barrier to entry than "fix a bug in the layer system."
- **Open question remaining:** Target a specific set for completeness, or cherry-pick across sets?

### Performance Targets — PARTIALLY DECIDED
- **Current:** 500 fuzz games complete quickly (no measured bottleneck)
- **AI self-play target:** Deferred — need more intuition on what AI approach to pursue before setting throughput goals. Interlinked with the performance analysis below.
- **Layer caching:** Profile-driven. See Performance Analysis appendix.

### UI/UX Scope — DECIDED
- **GUI, not TUI.** MtG's complexity (7 zones, stack, multiple card types, complex board states) makes graphical presentation the right call. TUI would be more engineering effort for a worse experience.
- **Target aesthetic:** Between XMage and Arena — functional AND pleasant.
- **AI interface:** API layer over `DecisionProvider`. The same trait that CLI/Random/Scripted use, exposed as a structured API so AI can be developed/swapped independently.
- **Network play:** Stretch goal. WebSocket for real-time.

---

## Performance Analysis

### Current Performance Profile

The engine today does very little per-object work:
- **Priority actions:** O(1) state lookups, O(n) SBA scan where n = battlefield size
- **Combat:** O(a × b) for attacker/blocker validation, O(a) for damage
- **Oracle queries:** Direct field reads from `card_data` / `BattlefieldEntity`

**Measured baseline (500 fuzz games):**
- **Debug build:** ~78 games/sec (~6.4s for 500 games). This is the default `cargo run` mode.
- **Release build:** ~848 games/sec (~0.59s for 500 games, 1.17ms/game). **~11x speedup over debug.** Avg 58.6 turns/game.

**All performance analysis below assumes release-mode measurements.** Debug-mode numbers are not actionable — they reflect compiler overhead, not engine architecture.

### Where Performance Will Degrade

#### 1. Layer System — `compute_characteristics()` (Phase 5)
**The biggest concern.** Every oracle query (`has_keyword`, `is_creature`, `get_effective_power`, etc.) will route through `compute_characteristics`, which must:
- Collect all active `ContinuousEffect`s per layer (7 layers + 4 P/T sublayers)
- For each layer: filter applicable effects, run dependency detection, topological sort, apply in order
- Dependency detection does hypothetical application for each effect pair

**Call frequency is high.** SBA checks call `is_creature` + `get_effective_toughness` for every permanent. Combat validation calls `has_keyword` for every attacker/blocker pair. Targeting calls `get_effective_types`. A single priority pass can trigger dozens of `compute_characteristics` calls.

**Scaling:** O(p × e × e) per full recompute, where p = permanents and e = active effects in the relevant layer. With 10 permanents and 5 effects, this is trivial. With 30 permanents and 20 effects (realistic mid-game with anthems, auras, control magic), it's noticeable.

#### 2. Trigger Checking (Phase 7)
After every event batch and SBA cycle, the engine must scan all permanents for matching triggers. With n permanents and m events per batch, this is O(n × m). Mostly linear, but trigger storms (e.g., Soul Warden + mass token creation) can cascade.

#### 3. Replacement Effect Checking (Phase 6)
Every state-mutating action runs through `apply_replacement_effects`, which scans all active replacement effects. This is O(r) per action where r = replacement effects. Usually small (r < 10), but "whenever damage would be dealt" prevention effects scan on every damage event.

#### 4. AI Game Tree Search
This is where it compounds. An AI exploring N game states per decision will call `compute_characteristics` thousands of times. If each call is 10μs, 100k evaluations per turn = 1 second. If each call is 100μs, that's 10 seconds.

### AI Feasibility — Is MtG Too Complex?

**No — but it requires learned evaluation, not brute-force search.** MtG combines all four difficulty axes: enormous state space (~10⁵⁰⁰+), high branching factor (10³–10⁶ legal action sequences per turn), hidden information (opponent's hand, libraries), and stochasticity (library order). No engine can brute-force this.

**What works for games of this complexity (from the literature):**

| Game | Key AI Insight | Relevance to MtG |
|------|---------------|-------------------|
| **AlphaStar** (SC2, DeepMind 2019) | Action space decomposition — break 10²⁶ possible actions into sub-decisions (action type → unit → target). Population-based training to avoid strategy collapse. | MtG priority actions decompose naturally: action type → card → targets → mana payment. Same pattern. |
| **Libratus/Pluribus** (Poker, CMU 2017/2019) | Information set abstraction + CFR. Hidden info means you can't search the exact tree — group similar states, solve over abstractions. | MtG hidden info (opponent's hand) requires either determinization (sample possible hands, solve each, average) or IS-MCTS (search over information sets directly). |
| **MtG-specific** (Ward & Cowling 2009, various 2018–2023) | Determinization + MCTS works for simplified MtG. Neural evaluators struggle with variable board state representation. | Feature engineering for neural nets is the hard problem — variable number of permanents with variable characteristics and interactions. |

**The practical AI architecture for our engine:**

```
┌─────────────────────────────────────────────┐
│  AI Decision Layer                          │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐ │
│  │ Neural   │  │ MCTS     │  │ Hand      │ │
│  │ Evaluator│──│ (guided) │──│ Sampling  │ │
│  └──────────┘  └──────────┘  └───────────┘ │
├─────────────────────────────────────────────┤
│  DecisionProvider API                       │
├─────────────────────────────────────────────┤
│  Engine (GameState)                         │
└─────────────────────────────────────────────┘
```

1. **Neural evaluator** — trained on game states, outputs position score + action policy. Replaces full simulation for leaf evaluation.
2. **Shallow MCTS** — 100–1000 rollouts per decision (not 100k). Neural eval at leaves instead of simulating to game end.
3. **Hand sampling** — for hidden info, sample N possible opponent hands, run MCTS on each, aggregate.

**Engine throughput needed:** ~100–1000 state evaluations per decision for real-time play. At even modest release-mode throughput, this is achievable *without* Tier 2 optimizations. Tier 2 (CoW, incremental) becomes important for **training data generation** (millions of self-play games), not for inference.

**Key advantage we have over AlphaStar's setup:** AlphaStar had to run the actual StarCraft II game client as its simulator — a massive bottleneck. We own our engine, it's pure Rust, and `DecisionProvider` is already the clean API boundary an AI needs.

### Mitigation Strategy (layered, profile-driven)

#### Tier 0: Free — Already Designed In
- **Oracle indirection.** All characteristic queries go through oracle functions, not direct field reads. This means we can swap the implementation (field read → layer computation → cached computation) without changing any call site. The indirection is already in place.
- **`GameAction` middleware.** Replacement effects slot into `execute_action` without changing any caller. One interception point.
- **Effect-free fast path.** If `game.continuous_effects.is_empty()`, `compute_characteristics` can return printed values directly. Early games (before enchantments/anthems) pay zero cost.

#### Tier 1: Easy — Implement When Profiling Shows Need
- **Per-object result caching with invalidation.** Cache `EffectiveCharacteristics` per `ObjectId`. Invalidate when:
  - A `ContinuousEffect` is added or removed
  - A permanent enters or leaves the battlefield (changes filter results)
  - A counter is added or removed (affects L7c)
  - A zone change occurs (affects CDAs like Tarmogoyf)
  
  This is a coarse-grained invalidation — any of the above events flushes the entire cache. But it means repeated queries within a single priority pass (common: SBA check calls `is_creature` + `get_effective_toughness` for 20 permanents, then combat does the same) hit the cache. **Expected 5–10x reduction in `compute_characteristics` calls.**

- **Layer-scoped invalidation (refinement).** Instead of flushing the entire cache, track which layers are "dirty." If only a L7c effect changed (Giant Growth), only P/T results are invalidated — type, color, ability results remain cached. More bookkeeping, but targeted.

- **Trigger condition indexing.** Instead of scanning all permanents for trigger matches, maintain an index: `HashMap<TriggerCondition discriminant, Vec<ObjectId>>`. When a `CreatureEnteredBattlefield` event fires, only check objects registered for that trigger type. O(1) lookup + O(k) where k = matching triggers, instead of O(n) full scan.

#### Tier 2: Moderate — Implement for AI Self-Play
- **Incremental `compute_characteristics`.** Instead of recomputing from scratch, maintain a "characteristics delta" when a single effect is added/removed. Most turns only change 1-2 effects — recomputing all 7 layers for all permanents is wasteful. Delta application: identify which objects are affected by the changed effect, recompute only those objects, only in the affected layer and above (layer changes cascade downward: a L4 type change can affect L6 ability checks, but an L7 P/T change affects nothing above).

- **`GameState` copy-on-write for AI.** AI tree search needs to fork game states. Current `GameState` is a monolithic struct with `HashMap`s — cloning is O(n) per object. CoW with `im` (immutable data structures) or `Rc`-based sharing would make forking O(1) for unchanged subtrees. This is a significant refactor but has massive payoff for AI throughput.

- **Parallel game evaluation.** The engine is single-threaded by design (mutable `GameState`). For AI, run independent game simulations on separate threads. The engine doesn't need internal parallelism — external parallelism (many games simultaneously) is simpler and scales linearly with cores.

#### Tier 3: Advanced — Only If Needed
- **Pre-compiled effect application.** For static abilities that don't change (Glorious Anthem: +1/+1 to your creatures), compile the effect into a fast-path closure at registration time instead of interpreting the `Modification` enum on every query. Saves dispatch overhead.

- **Dependency graph caching.** The dependency detection algorithm rebuilds the DAG iteratively after each effect application. If the set of active effects hasn't changed since the last computation, the ordering is stable and can be reused. Track effect set identity (hash of active effect IDs) and skip rebuilds on cache hit.

- **Compact game state for AI.** Instead of forking the full `GameState`, define a compressed state representation (bitboard-style) for AI evaluation. Creatures as `(power, toughness, keywords_bitfield)`, life totals as raw integers. Loses fidelity but enables millions of evaluations per second for MCTS rollouts.

### When to Invest

**All triggers measured in release mode** (`cargo run --release`). Debug numbers are not actionable.

| Trigger | Action |
|---------|--------|
| Phase 5 complete, release fuzz drops below 500 games/sec | Add Tier 1 caching |
| Phase 7 complete, release fuzz with trigger-heavy cards shows >5ms/game | Add trigger indexing |
| AI prototype shows <100 games/sec throughput (release) | Add Tier 2 (CoW, incremental) |
| AI training needs >10k games/sec | Evaluate Tier 3 |
| No slowdown observed in release mode | Do nothing — premature optimization is the root of all evil |

### Bottom Line

The architecture is **well-positioned** for performance work because of two design decisions made early:
1. **Oracle indirection** — every characteristic query goes through a function we control. Swapping from "read field" to "compute" to "cached compute" is a one-line change per function.
2. **`GameAction` middleware** — every state mutation goes through `execute_action`. Adding replacement effects, trigger checks, or cache invalidation is a single interception point.

The most likely bottleneck is `compute_characteristics` call volume during SBA/combat loops. Tier 1 caching (coarse invalidation) will handle this for human-speed play and fuzz testing. Tier 2 (CoW + incremental) is the investment needed for competitive AI. Tier 3 is exotic and may never be needed.

**Recommendation:** Build Phase 5 with the correctness-first recompute-on-query approach (already planned). Run the fuzz harness after completion. If games are still fast (likely — board states in fuzz are small), defer all caching. If not, add Tier 1. Measure again after each phase.

---

## Summary

| Phase | Name | Est. Tickets | Relative Complexity | Cards After |
|-------|------|-------------|--------------------:|------------:|
| 5-Pre | Engine Fixes (audit) | 24 | Medium | 24 |
| 5 | Continuous Effects & Layers | 21 | **Very High** | 35 |
| 6 | Replacement Effects | ~30–35 | **High** | ~55 |
| 7 | Triggered Abilities | ~25–30 | **High** | ~75 |
| 8 | Primitives, Keywords & Breadth | ~40–50 | Medium (volume) | ~125+ |
| 9 | Format Support & Advanced | ~30–40 | Medium-High | ~150+ |
| 10 | UI, AI & Performance | 20+ each | Varies | ~150+ |

The critical path runs through Phases 5-Pre → 5 → 6 → 7. Everything else can be parallelized or deferred. The simulator becomes "core rules complete" after Phase 8, "format-ready" after Phase 9, and "user-ready" after Phase 10.
