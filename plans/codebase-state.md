# Codebase State — CR Coverage Map

Ground-truth snapshot of CR coverage. Single source of truth — if another planning doc contradicts this, this wins. Last grounded-in-code audit: 2026-08-19.

---

## TL;DR

- **Code size:** ~22,500 lines of Rust across 68 `.rs` files. 509 tests (415 unit + 93 integration + 1 doc-test), 0 warnings, fuzz harness runs 250-game batches.
- **Well-covered:** CR 1 (game basics), CR 3 (card types), CR 4 (zones), CR 5 (turn structure), CR 7 (keyword abilities + SBAs).
- **Partially covered:** CR 6 (casting: pipeline skeleton + X/alt/additional-cost landed, mode choice + distribution + activation restrictions pending). CR 1 mulligan is a stub. Equip and Bestow (CR 702.6, 702.103) not started.
- **Not started:** **replacement effects (CR 614–616)** beyond a stub hook, **triggered abilities (CR 603)** beyond an enum variant, CR 800 multiplayer priority/turn rotation.
- **Layers (CR 613) — core landed, three layers live (Phases LA–LD, 2026-05 → 2026-08).** The system is real, not scaffolding: `Layer` enum with all 9 sublayer variants (`engine/layers/types.rs`), `EffectiveCharacteristics` struct (name, mana_cost, colors, types, subtypes, supertypes, keywords, abilities, P/T, controller), a `ContinuousEffect` registry with duration-based expiry (`state/continuous_effects.rs`, 304 lines), and `compute_characteristics` (`engine/layers/compute.rs`, 967 lines). Static abilities register through `GameState::register_static_effects`. `oracle/characteristics.rs` wrappers all route through `compute_characteristics`.
  - **Live layers:** 7b (set P/T), 7c (modify P/T), 7d (switch P/T) — Phase LB. 5 (color) — Phase LC. 4 (types/subtypes/supertypes) — Phase LD Part A.
  - **Still stubbed:** Layer 6 (abilities) — `Primitive::GrantKeyword` / `RemoveAbility` return `NotImplemented` at `engine/resolve.rs:435-438`. Layer 2 (control) — `Primitive::GainControl` likewise. Layer 3 (text) and Layer 1 (copy) are enum variants only.
  - **Dependency algorithm (CR 613.8) not implemented.** Ordering is timestamp-only, which is sufficient for the layers landed so far in isolation but will not survive Layer 6 + Layer 4 interaction (Humility/Opalescence).
  - **CR 305.7 / 305.6 — ✅ done (Phase LD Part B).** Blood Moon strips a nonbasic land's printed abilities and grants the intrinsic `{T}: Add {R}`; Urborg adds a basic land type and its mana ability without stripping. Lives in `engine/layers/land_types.rs`. `AbilityOrigin` was evaluated at Part B kickoff and **not built** — layer ordering makes it unnecessary; see `layers-architecture.md` §15.2 item 4.
- **Commander (CR 903) — in scope, skeleton only:** command zone ✅ as a `Zone` variant + `GameState.command` field; commander damage loss SBA ✅; commander damage **increment on combat damage now wired** (2026-04-18) via `GameObject.is_commander` flag + per-source accumulation in `execute_action(DealDamage)`. Still missing: commander tax, command-zone replacement (depends on CR 614), `GameConfig::commander()`, commander designation/setup hook.
- **Biggest single block of work remaining before the engine can run real Magic:** the rest of Layers (6, 2, dependency algorithm) + triggered abilities + replacement effects. These are tangled — CR 613.1c says abilities themselves can be layer-modified, replacement effects depend on effective characteristics, triggers often fire on events that must be observed post-replacement. **Commander specifically depends on replacement effects (903.9 command-zone redirection) and multiplayer (800 priority).**
- **Before starting any of those systems:** see **[Deferred Migrations](#deferred-migrations)** for prerequisite cleanups owed by forward-looking scaffolding. Each target system (Replacement, Layers, Triggers, Commander) has a short list of pending migrations that don't surface as test failures until that system lands.
- **Layers has a formalized architecture doc:** `plans/layers-architecture.md` (2026-04-18). Authoritative for type shapes, module layout, sublayer enumeration, dependency algorithm, and Phase LA→LD work sequencing. A subsequent session should execute from that doc.

---

## Spec database

`plans/specdb.py` joins the atomic-test corpus (`plans/atomic-tests/sessions/*.md`, ~1,753 entries) against the Rust test suite and answers "what is actually covered" as a query instead of a judgment call.

```
python plans/specdb.py build     # rebuild plans/atomic-tests/spec.sqlite
python plans/specdb.py stats     # coverage by phase
python plans/specdb.py next --phase "Phase 5-Layers" --rule 613
python plans/specdb.py show ATOM-305.7-002
python plans/specdb.py orphans   # COVERS ids that match no atom
python plans/specdb.py gaps --chapter 6   # CR rules the corpus never examined
```

The CR itself is in the database as ground truth: `MTG-Rules/versions/tmnt.txt`
(3,120 rules, effective 2026-02-27) is the baseline the engine targets. `gaps`
reports CR rules no session ever mentioned — 155 total, of which 65 are
out-of-scope variants and ~90 are card-breadth or genuinely unexamined. **CR 6
has zero blind spots.** Known real gaps: CR 115.7b–f (changing targets),
115.9 (targeting-aware objects), 508.7b–d (reselecting attack targets).

A test declares coverage with a comment directly above `#[test]`:

```rust
// COVERS: ATOM-305.7-001, ATOM-305.7-004
#[test]
fn test_blood_moon_makes_nonbasic_lands_mountains() { ... }
```

The database is **derived** and gitignored — never hand-edit it. The two authored inputs are the session files (spec) and the `COVERS:` annotations (status). If a number looks wrong, fix one of those and rebuild.

---

## Chapter-by-chapter map

Legend: ✅ done (with test coverage) · 🟡 partial · ⚠️ stub or sketch · ❌ not started

### CR 1 — Game Concepts

| Section | Rule topic | Status | Where |
|---|---|---|---|
| 100 | Formats / deck legality (size, copy limits) | 🟡 config present, enforcement not wired | `state/game_config.rs` |
| 103.2 | Starting life | ✅ | `state/game_config.rs` |
| 103.4 | Mulligan (London) | ⚠️ **stubbed** — "players always keep their first hand" | `state/game.rs:88-90` |
| 103.6 | Starting hand size | ✅ | `state/game_config.rs`, `state/game.rs:98-104` |
| 107 | Mana values, X costs, hybrid/Phyrexian symbols (enum) | 🟡 enum defined; hybrid/Phyrexian/X payment = `NotImplemented` | `types/mana.rs`, `can_pay` returns false for hybrid |
| 108 | Tokens and cards | ✅ `is_token`, `is_copy` flags | `objects/object.rs` |
| 109 | Objects, characteristics | ✅ data model | `objects/card_data.rs`, `objects/object.rs` |
| 110 | Permanents | ✅ `BattlefieldEntity` + attachment | `state/battlefield.rs` |
| 111 | Tokens — cease-to-exist | ✅ SBA 704.5d | `engine/sba.rs:332+` |
| 117 | Timing + priority | ✅ priority rounds, mana-ability window (601.2g / 602.1b), bounded retry + pass fallback | `engine/priority.rs`, `engine/cast.rs` |
| 118 | Costs (types only) | ✅ alternative/additional cost enums; X + kicker + flashback + evoke scaffolding | `types/costs.rs` |
| 118.8–118.9 | Alternative / additional cost resolution | 🟡 assemble_total_cost + rollback done (T18a); wiring per-cost-type semantics pending (T18b/c/d) | `engine/cast.rs`, `engine/costs.rs` |
| 119 | Life changes | ✅ with source attribution | `events/event.rs`, `engine/actions.rs` |
| 120 | Damage — combat damage routing, infect/wither/lifelink | 🟡 combat damage ✅, lifelink ✅, first/double strike ✅, trample ✅, deathtouch ✅; infect/wither/toxic ❌ (T21c pending) | `engine/combat/keywords.rs`, `engine/combat/resolution.rs` |
| 121 | Drawing | ✅ basic | `engine/actions.rs` |
| 122 | Counters | ✅ 19 counter types (12 evergreen keyword + +1/+1, -1/-1, loyalty, charge, poison, commander damage), per-entity HashMap | `types/effects.rs`, `state/battlefield.rs`, `state/player.rs` |
| 123 | Mana (pool, persistence, restrictions) | ✅ full `ManaPool` with restricted sidecar, persistence, grants, context-aware spending (T12b landed) | `types/mana.rs` (1370 lines) |

### CR 2 — Parts of a Card

| Section | Rule topic | Status | Where |
|---|---|---|---|
| 201–205 | Name/mana cost/color/color indicator/type line | ✅ data model | `objects/card_data.rs` |
| 205.4d | Supertypes (legendary) | ✅ enforced by legend rule SBA | `engine/sba.rs` (704.5j) |
| 206 | Expansion/rarity | not modeled — not needed for engine |
| 207 | Text box / rules text | 🟡 stored as `String`; not parsed into structured abilities (no NLP, hand-coded card defs) | `objects/card_data.rs` |
| 208 | P/T (`i32`) | ✅ signed, correct per E8 | `objects/card_data.rs` |
| 209 | Loyalty (for PW) | ✅ ETB counter init + 0-loyalty SBA | `engine/sba.rs` (704.5i), `state/game_state.rs` (init_etb_counters) |

### CR 3 — Card Types

| Section | Rule topic | Status | Where |
|---|---|---|---|
| 301 | Artifacts (incl. 301.5 Equipment — attachment + can't-attach-to-non-creature) | 🟡 attachment tracking ✅, `attach_to`/`detach` primitives ✅; **Equip activated ability ❌** | `state/battlefield.rs`, `engine/zones.rs` |
| 302 | Creatures + summoning sickness | ✅ turn-based tracking (T09) | `oracle/characteristics.rs` `has_summoning_sickness` |
| 303 | Enchantments / Auras — ETB attach, enchant filter, control on resolve, non-stack ETB host choice | ✅ all via T15b | `engine/resolve.rs` `attach_aura_on_etb`, `objects/card_data.rs` `enchant_filter` |
| 304 | Instants | ✅ basic cast path | `engine/cast.rs` |
| 305 | Lands | ✅ basic lands + mana abilities | `cards/basic_lands.rs` |
| 306 | Planeswalkers | ✅ loyalty ETB, 0-loyalty SBA; loyalty-ability costs ❌ (T19 pending) | `engine/sba.rs` |
| 307 | Sorceries | ✅ basic cast path + sorcery-speed enforcement | `engine/cast.rs`, `oracle/legality.rs` |
| 308 | Kindred (formerly Tribal) | ✅ data model only |
| 309 | Dungeons | ❌ |
| 310 | Battles | 🟡 enum exists; battle-specific mechanics ❌ |
| (Sagas) | Saga enchantments (subtype of 303 + chapter mechanics in CR 7xx) | ❌ |

### CR 4 — Zones

| Section | Rule topic | Status | Where |
|---|---|---|---|
| 400–405 | Zones + move_object + cleanup_zone_state (with attachment cleanup) | ✅ | `engine/zones.rs` (450 lines) |
| 406 | Library | ✅ |
| 407 | Graveyard | ✅ |
| 408 | Stack | ✅ with rollback | `engine/stack.rs`, `engine/cast.rs` |

### CR 5 — Turn Structure

✅ Complete (Phase 1/2 work). `engine/turns.rs`, `engine/combat/steps.rs`, cleanup step with SBA re-loop (T16). 514.3a re-loop in `state/game.rs` `perform_cleanup_actions`.

### CR 6 — Spells, Abilities, and Effects (**THE BIG ONE**)

| Section | Rule topic | Status | Where |
|---|---|---|---|
| 601.2a | Announce spell / move to stack | ✅ | `engine/cast.rs` (780 lines) |
| 601.2b | Choose modes / X / alt+additional costs | 🟡 X ✅, alt ✅, additional ✅ (T18a); **mode choice ❌** (T18b pending — `ChoiceKind::ChooseModes` not added yet) | `engine/cast.rs` |
| 601.2c | Choose targets + target uniqueness | ✅ multi-target with `TargetCount::Exactly(n)` / `UpTo(n)` min/max enforcement; `validate_targets` called post-selection; **uniqueness rules (115.3/4) ❌** (T18b) | `engine/cast.rs:130–152`, `ui/ask.rs` |
| 601.2d | Distribution (damage/counters among targets) | ❌ literal placeholder at `engine/cast.rs:154` (single-line comment, no code) | `engine/cast.rs` |
| 601.2e | Post-proposal legality | ⚠️ **explicit no-op** with a comment: *"Currently a no-op (the pre-proposal check is sufficient for the cards we support). Future: validate that chosen targets are still legal after all proposal choices are made"* | `engine/cast.rs:175–182` |
| 601.2f | Determine total cost | ✅ | `engine/costs.rs` `assemble_total_cost` |
| 601.2g | Mana ability activation window | ✅ (SPECIAL-2) | `engine/priority.rs` `run_mana_ability_window` |
| 601.2h | Pay costs (with rollback on failure) | ✅ for `Cost::SacrificeSelf`, `Cost::Tap`, `Cost::PayLife`, `Cost::Mana`; **`Cost::Sacrifice(filter, count)` = `NotImplemented`** (T18c) | `engine/costs.rs` |
| 601.2i | Spell becomes cast | ✅ | `engine/cast.rs` |
| 602 | Activated abilities (activate_ability + rollback) | ✅ structural; **activation restrictions** (sorcery-speed PW, graveyard-activated abilities) ❌ (T19) | `engine/actions.rs` activate_ability |
| **603** | **Triggered abilities** | ❌ `AbilityType::Triggered` enum variant exists (`objects/card_data.rs:49`), **no engine handling**. No trigger queue, no event→trigger mapping, no "puts X onto the stack" mechanism. | only in `ui/display.rs:164` for label printing |
| 604 | Static abilities | 🟡 keyword statics via `has_keyword`; continuous-effect statics (P/T, color, type) register via `GameState::register_static_effects` ✅; other non-keyword statics ❌ | `state/game_state.rs` |
| 605 | Mana abilities | ✅ detection + window + enumeration | `oracle/mana_helpers.rs`, `engine/priority.rs` |
| 606 | Loyalty abilities | ❌ (T19 pending) |
| 607 | Linked abilities | ❌ (T20 pending) |
| 608 | Resolution of spells and abilities — fizzle, Target vs Choose split | ✅ via T15b refactor (`TargetSpec` → `EffectRecipient`) | `engine/resolve.rs`, `engine/stack.rs` |
| 609–611 | Effects (one-shot, continuous) | ✅ one-shot via `Effect`/`Primitive`; continuous via the layer registry with duration-based expiry | `state/continuous_effects.rs` |
| 612 | Text-changing effects | ❌ |
| **613** | **Continuous effects — layer system** | 🟡 **core landed; layers 7b/7c/7d, 5, and 4 live.** `Layer` enum + `EffectiveCharacteristics` + `ContinuousEffect` registry + `compute_characteristics` all exist and are exercised by the Phase LB/LC/LD tests. **Missing:** Layer 6 (abilities), Layer 2 (control), Layer 3 (text), Layer 1 (copy); the CR 613.8 dependency algorithm (timestamp ordering only). CR 305.7/305.6 land semantics landed in Phase LD Part B. | `engine/layers/{types,compute,land_types}.rs`, `state/continuous_effects.rs`, `oracle/characteristics.rs` |
| **614–616** | **Replacement + prevention + interaction** | ⚠️ Stub only. `engine/actions.rs:86-89` `execute_action` is a pass-through to `perform_action` with a comment: *"Phase 6: A `apply_replacement_effects(action)` call will be inserted here"*. No `ReplacementEffect` struct. |

### CR 7 — Additional Rules

| Section | Rule topic | Status | Where |
|---|---|---|---|
| 701.3 | Attach | 🟡 Aura ETB attach ✅ (T15b); **general `attach(attachment, target)` primitive ❌** — no path to reattach Equipment outside Aura ETB, because Equip activation (702.6a) isn't implemented |
| 701.8 | Destroy (destroy keyword action, respects indestructible) | ✅ (T16) | `engine/resolve.rs` Primitive::Destroy |
| 701.21 | Sacrifice | 🟡 `Cost::SacrificeSelf` ✅; `Cost::Sacrifice(filter, count)` = `NotImplemented` (T18c) | `engine/costs.rs` |
| 702.2 | Deathtouch | ✅ (combat lethal-damage check, T09 fuzz run confirmed) | `engine/combat/keywords.rs` |
| 702.6 | **Equip** (activated ability "Equip {cost}") | ❌ not implemented as an activated ability type |
| 702.10c | Untap symbol {Q} — summoning-sickness check | ✅ (T10) | `engine/costs.rs` |
| 702.12 | Indestructible | ✅ (T16) | `engine/sba.rs`, `engine/resolve.rs` |
| 702.15 | Flash | ✅ flash casting window honored |
| 702.19 | Trample (co-assigned lethal damage) | ✅ with per-blocker maxes (SPECIAL-1c) | `engine/combat/keywords.rs` `assign_trample_damage` |
| 702.27 | Haste | ✅ |
| 702.11 | First/double strike | ✅ (damage steps split) |
| 702.16 | Lifelink (per-source LifeChanged) | ✅ (T11) |
| 702.14 | Landwalk, 702.7 Flying, 702.9 Reach, 702.23 Vigilance, 702.18 Menace, 702.24 Shroud, 702.11 Hexproof | ✅ blocker-legality pre-filter (SPECIAL-8) covers flying/reach. Others validate in combat. |
| 702.103 | **Bestow** | ❌ |
| 702.X | Numerous keyword abilities (Bestow, Overload, Awaken, Emerge, etc.) | ❌ (these are the ~45 `NEW-*` atomic-tests) |
| 703 | Turn-based actions | ✅ |
| **704.5a–w** | **State-based actions** | ✅ 704.5a (life ≤0), 704.5b (empty library draw), 704.5c (poison ≥10), 704.5d (tokens in non-BF zones), 704.5f (0 toughness), 704.5g (lethal damage with indestructible + deathtouch), 704.5h (deathtouch), 704.5i (PW 0 loyalty), 704.5j (legend rule), 704.5m (Aura illegal host), 704.5n (Equipment/Fort on illegal permanent), 704.5p (creature/other attached catch-all), 704.5q (+1/+1 / -1/-1 annihilation). 704.5s (Saga), 704.5t (dungeon), 704.5v/w/x (battle) ❌. Commander damage ✅. | `engine/sba.rs` (1015 lines) |
| 705 | Flipping coins, rolling dice | ❌ |

### CR 8 — Multiplayer Rules

**Commander is in scope. This section is a real gap, not an out-of-scope deferral.**

| Section | Rule topic | Status | Where |
|---|---|---|---|
| 800 | General multiplayer rules (active player turn order, multiple opponents) | 🟡 `GameState.players: Vec<PlayerState>`, `active_player: usize`, `priority_player: usize` support N players architecturally; priority round logic and targeting assume 2-player semantics in several places | `state/game_state.rs`, `engine/priority.rs` |
| 801 | Limited range of influence | n/a for Commander (uses range = all) |
| 802 | Attack Multiple Players option | ❌ combat assumes single defender |
| 806 | **Free-for-All** — the default Commander game structure | ❌ not implemented. No turn-order rotation past 2 players; no player-elimination handling (when a player loses in a 3+ player game, their permanents, stack entries, and triggers need specific resolution per 800.4) |
| 810 | Two-Headed Giant | ❌ |
| others | Grand Melee, Team vs Team, Emperor, Alternating Teams | ❌ |

**Known multiplayer-shaped gaps in existing 2-player code:**
- `engine/combat/validation.rs` assumes attacks go at "the defender" (single opponent).
- Priority passes loop player0 → player1 → back; no general N-player priority-pass loop.
- Targeting prompts don't enumerate 3+ players as target candidates in most paths (SPECIAL-8 blocker pre-filter doesn't need to, but spell targeting does).
- No player-elimination SBA (rule 800.4a — "a player who has left the game is treated as though they don't exist").

### CR 9 — Casual Variants

**Commander (CR 903) — in scope, partially modeled.**

| Rule | Topic | Status | Where |
|---|---|---|---|
| 400.7 / 406 | **Command zone as a Zone variant** | ✅ `Zone::Command`, `GameState.command: Vec<ObjectId>`, wired into `move_object` / `remove_from_zone_collection` | `types/zones.rs:10`, `state/game_state.rs:70`, `engine/zones.rs:208–216,255–259` |
| 903.3 | **Starting life = 40** | ❌ no `GameConfig::commander()` constructor; `game_config.rs` header comment promises one via a future `Format` trait |
| 903.5a | **Mulligan (London, same as standard)** | ⚠️ mulligan itself is stubbed (`state/game.rs:88-90`) regardless of format |
| 903.5b | Deck construction (100 cards singleton + color identity) | 🟡 `DeckLimits { min_deck_size: 99, max_copies: 1 }` fields exist but no commander-config factory wires them; **color identity enforcement not implemented** |
| 903.7 | **Commander designation + command zone start** | 🟡 `GameObject.is_commander: bool` flag exists (2026-04-18); no deck-construction / setup hook yet flips it, and no "commander starts in command zone" routing |
| 903.8 | **Commander tax (+{2} per prior cast from command zone)** | ❌ no cast counter, no cost modification |
| 903.9 | **Commander zone change replacement** (graveyard/exile/hand/library → "instead in the command zone") | ❌ requires the replacement effect pipeline (CR 614), which is a stub hook at `engine/actions.rs:86-89` |
| 903.10 | **Commander damage loss (≥21 combat damage from one commander)** | ✅ SBA (T16); `commander_damage_taken: HashMap<ObjectId, u32>` on `PlayerState` (T02) | `state/player.rs`, `engine/sba.rs` |
| 903.11 | Attacking with commander + accumulating commander damage | ✅ (2026-04-18) `GameObject.is_commander` flag + `execute_action(DealDamage)` accumulates per-source `commander_damage_taken` when `is_combat && target == Player && source.is_commander`. 5 unit tests cover basic accumulation, 21-damage threshold, non-combat exclusion, non-commander exclusion, and per-source isolation. Still requires a Commander-format setup hook to actually flip the flag at deck construction — no gameplay wiring yet sets `is_commander = true` outside tests. |
| 903.12 | Partner | ❌ (`EnchantmentType::Background` exists as a data-type, no mechanics) |
| 903.13 | Friends Forever / Choose a Background / Doctor's companion | ❌ |

**Other variants in CR 9 (Brawl, Planechase, Archenemy, Vanguard, etc.) — ❌ not started.**

---

## Deferred Migrations

**Purpose:** track technical debt incurred by forward-looking scaffolding. Each entry is a migration owed to a future system before that system can safely land. These don't show up as test failures today because the dependent system doesn't exist yet — which is exactly why they're easy to forget. Any time a new forward-looking stub is added, it should be recorded here.

**How to use this section:** before opening the first ticket of a listed target system, re-read that system's subsection and treat the items as prerequisites to schedule before or alongside the system's core work.

### Before Replacement effects (CR 614–616)

The replacement pipeline is designed to sit inside `execute_action` at `engine/actions.rs:86-89`. Every mutating action must flow through there for replacements to observe them. Status:

1. **Zone-change migration — ✅ done (2026-04-18).** `move_object` is now `pub(crate)` with documentation directing external callers to `change_zone` / `execute_action(GameAction::ZoneChange)`. All 12 previously-direct callers (5 SBA sites in `engine/sba.rs`, `Cost::SacrificeSelf` in `engine/costs.rs`, push-to-stack + 4 rollbacks in `engine/cast.rs`, cleanup discard in `state/game.rs`) now route through the chokepoint. `engine/actions.rs::change_zone(id, to)` is the new convenience wrapper. Internal helpers (`draw_card`, `play_land`, and the `GameAction::ZoneChange` arm itself) continue to call `move_object` directly from inside `engine/zones.rs`.

2. **Open-coded zone bookkeeping — ✅ CounterSpell migrated; 3 stack.rs sites tagged as structural bypasses (2026-04-18).**
   - `engine/resolve.rs` `Primitive::CounterSpell` now calls `change_zone(id, Graveyard)` which tears down the `StackEntry` via `remove_from_zone_collection(Stack)`, then emits `SpellCountered`. No more manual bookkeeping.
   - `engine/stack.rs` three sites (permanent-spell ETB, instant/sorcery → graveyard, `handle_fizzle`) are tagged `// REPLACEMENT-BYPASS:` because the stack-pop-first pattern (see doc comment at `engine/stack.rs:27-32`) removes the object from the stack `Vec` before resolution begins, so `move_object` would double-remove. When the Phase 6 replacement pipeline lands, these sites need their own ZoneChange dispatch variant that skips the stack-Vec removal step — or the pattern needs to change. Flagging now so the decision isn't re-discovered.

3. **Event-emission audit for trigger observability.** Many actions emit `GameEvent`s directly via `self.events.emit(...)` bypassing `execute_action`. Before Replacement lands (and especially before Triggers), sanity-check that event emission happens *after* the replacement pipeline runs and reflects the final action taken, not the originally-proposed action.

### Before Layers (CR 613) — now DURING Layers

The layer system's designated single-point change site is `oracle/characteristics.rs`. Status as of 2026-08-19, after Phases LA–LD:

1. **Pre-layer P/T shim — ✅ done.** `BattlefieldEntity.power_modifier` / `toughness_modifier` no longer exist anywhere in `src/`. Layer 7c output replaced them.

2. **Direct `CardData` reads — ✅ done (2026-08-19).** 21 battlefield/stack call sites now route through `oracle/characteristics.rs`. New predicate helpers `has_type`, `has_subtype`, `has_supertype`, `has_permanent_type` join the existing `is_creature` / `get_effective_*` wrappers.
   - Migrated: `engine/sba.rs` (8 — planeswalker loyalty, legend rule, Aura/Equipment/Fortification attachment SBAs), `engine/targeting.rs` (7 — creature target, creature-or-planeswalker target, the whole `PermanentFilter` match), `engine/resolve.rs` (Aura ETB), `engine/stack.rs` (2 — permanent-spell routing, Aura spell), `state/game_state.rs` (ETB loyalty counters), `ui/display.rs`, `ui/random.rs`.
   - **Deliberately NOT migrated (6 sites):** `engine/zones.rs:144` (play a land from hand), `oracle/legality.rs:59` (playable lands in hand), `oracle/mana_helpers.rs` (×4 — castable spells in hand, instant/flash timing). These are cast-zone / play-from-hand legality, evaluated before the object is a permanent, so the layer system has nothing to contribute. Same exemption as `engine/cast.rs`. Each is tagged `// PRE-LAYER ZONE:` in source so a future grep audit doesn't re-flag it.
   - Regression coverage: `mtgsim/tests/layer_aware_queries_test.rs`, 5 tests. Verified to fail against the pre-fix tree and pass after.

3. **Cost modification pipeline stub — ❌ still a passthrough.** `engine/costs.rs:255` `apply_cost_modifications` with `TODO(L15)`. Wires to the continuous-effects registry for Thalia/Electromancer/Trinisphere.

   **Vocabulary gap this also owns.** Golden-Tail Trainer — "Aura and Equipment spells you cast cost {X} less to cast, where X is this creature's power" — is a static ability whose amount is read live. `AmountExpr` cannot say "this creature's power": `TargetPower` means the target of a resolving spell, and `Variable` is CR 107.3's X, chosen as a spell is cast. A `SourcePower`-style variant is needed, and the card is blocked on this item too, since cost modification is CR 613.11 / 601.2f rather than a characteristic change.

4. **Mana-pool persistence stub — ❌ still stubbed.** `engine/turns.rs:65,142` still pass `BlanketPersistenceSet::none()` with `TODO(T12c)`. The registry it needs now exists.

5. **Timestamps — ✅ live.** `BattlefieldEntity.timestamp` is now read by the layer system for 613.7 ordering (4 read sites). The CR 613.8 *dependency* algorithm is still unimplemented; ordering is timestamp-only.

6. **Direct `card_data.abilities` reads — ✅ done (2026-08-20, Phase LD Part B).** 7 battlefield sites route through the new `oracle::characteristics::get_effective_abilities`, because CR 305.7 makes printed abilities wrong for a Blood-Mooned land.
   - Migrated: `oracle/mana_helpers.rs` (×2 — `available_mana_sources`, `activatable_abilities`), `engine/mana.rs` (`activate_mana_ability`), `engine/priority.rs` (×2 — mana dispatch, id→index), `engine/cast.rs` (`activate_ability`), `ui/display.rs`.
   - **Index coupling:** `activatable_abilities` produces an ability index, `priority.rs` re-derives it by id, `cast.rs::activate_ability` consumes it. All three index the *effective* list. Migrating one alone silently activates the wrong ability — they move together or not at all.
   - **Deliberately NOT migrated:** `state/game_state.rs::register_static_effects` — see item 7. Plus `engine/cast.rs:56`, `oracle/mana_helpers.rs:174`, `engine/stack.rs:259` (spell abilities read pre-battlefield, same class as the `// PRE-LAYER ZONE:` sites).
   - Cost: `fuzz_games` went 25.9 → 29.2 ms/game (+12%, ±3% run-to-run) at 200 games / seed 12345, because mana-source enumeration is now a `compute_characteristics` walk per permanent instead of a field read. Accepted for now — `layers-architecture.md` §15.2 item 1 defers caching until after profiling, and this is the profiling. Revisit if it compounds when Layer 6 lands.

7. **Static-ability effect existence — ✅ strip half done (2026-08-21); grant half open.**

   `register_static_effects` still runs once at ETB off printed abilities, and still cannot use the layer system there (it runs inside `place_on_battlefield`, before the object's own effect is registered). That turned out not to be the thing to fix. **Registry membership is not effect existence.** The CR resolves existence *inside the layer walk* — `compute_characteristics` gathers the effects that apply at each layer and does not gather one whose generating ability is gone. Each effect applies at most once and the pending set only shrinks, so it terminates structurally; there is no fixpoint and nothing to cap.

   - **Stripped ability — fixed.** `ContinuousEffect.origin` (`EffectOrigin::StaticAbility { ability }` vs `Resolution`) makes existence askable, and `compute.rs::static_ability_still_exists` re-asks it at every layer against the source's frame *as of the end of the previous layer*. Blood Moon stripping a land now retires the effect that land registered at ETB. Gated on `RegistryScopeSummary::any_ability_changing` so it costs nothing on boards where no registered effect can change an ability set — `fuzz_games` at 200 games / seed 12345, measured back to back against `main` on the same machine: 27.59 → 28.62 ms/game, inside the documented ±3% run-to-run band. (Measure it that way — readings drift several ms across sessions, enough to invent a regression that isn't there.)
   - **Granted ability — ✅ half done (2026-08-23, Layer 6 phase); filter half open.** `GrantAbility(Box<AbilityDef>)` exists, and when a *resolution* grants a static-bodied ability, `resolve::register_granted_static_effects` registers the effects that ability generates then and there. Their `origin` is `StaticAbility { ability: <granted id> }`, so existence needs nothing new: the CR 613.7a check already re-asks at every layer whether the grantee still has the ability, and retires the derived effect when a later Layer 6 strip takes it away (`test_stripping_a_granted_ability_retires_the_effect_it_generated`).

     Eager registration works because a resolution's affected set is locked to its targets (CR 613.7b), so the grantee set is known at grant time. **What stays open is the case where it is not:** a *static* ability that grants a static ability over a filter — "enchanted creature has 'creatures you control get +1/+1'" — has a grantee set that changes with the board. That still needs the original plan here: the gather step in `apply_effects` unions the registry with effects derived from each permanent's frame-as-of-previous-layer, same loop, same frame cache, same CR 613.6 started-applying set.

7a. **Frame cache is live.** `layers-architecture.md` §5.2's per-call `(ObjectId, layer_ceiling)` memo now exists, because the existence check needs another object's characteristics mid-walk. The strictly-descending ceiling **is** the termination argument, and it is load-bearing: `test_self_stripping_land_terminates_and_is_stable` overflows the stack if the check asks at the full ceiling instead of `layer_index`. Only sub-computations are memoized; the top-level frame is requested once per call, so caching it would be a pure clone.

7b. **CR 613.7a clause 2 — ✅ implemented (2026-08-23).** "…or the timestamp of the effect that created the ability, whichever is later." It is the `max()` in `GameState::static_effect_timestamp`, which now takes `granted_at: Option<Timestamp>`; `None` means printed, and `register_static_effects` — running at ETB off printed text — is the caller that always passes it. `resolve::register_granted_static_effects` is the clause-2 caller.

    Tested so the two clauses disagree: grantee entered long ago, a Layer 7b `SetPowerToughness(9,9)` sits at timestamp 50, and the granting spell resolves after 60. Clause 1 alone gives 9/9; clause 2 gives 3/3. Deleting the `max()` fails that test and only that test.

    **One shape does not work, and it asserts rather than failing silently.** A granted static ability whose own effect lands in **layers 1–6** cannot apply: the grant applies *at* layer 6, so at any layer ≤ 6 the frame the CR 613.7a existence check reads is the pre-grant frame, and the derived effect finds no ability to justify itself. This is not a corner — it is CR 613.7a's own worked example, Rune of Flight granting "Equipped creature has flying", which is a layer 6 effect. That card needs Equip as well, so it is out of reach twice over. `register_granted_static_effects` carries a `debug_assert!` at the registration site so a card author is stopped instead of shipping a card that quietly does nothing.

    **The two halves of that limitation are not one problem, and only one is waiting on anything.**

    - **Layer 6 exactly** — Rune of Flight, "As long as enchanted permanent is an Equipment, it has 'Equipped creature has flying.'" The CR resolves this purely by timestamp within layer 6, and our *ordering* is already correct for it: clause 2 makes the derived timestamp `max(grantee, grant)`, which is ≥ the grant's, and on a tie the grant row is registered first so it wins the `EffectId` tiebreak in `effects_in_layer`. The grant therefore always sorts at-or-before its own derived effect. The single missing piece is that `static_ability_still_exists` asks `compute_to_ceiling(source, layer_index)` — the frame as of the *end of the previous layer* — and so cannot see a partially-applied current layer. Item 8 step 4's board-wide sequential pass is exactly that frame: apply a layer over its ordered applications, mutating a per-object map, and the check at position *k* sees everything applied earlier in the same layer. That is the same change 613.8b's loop rule needs to become exact, which is why they belong in one phase. Nothing about the Layer 6 work needs redoing — only the frame the check reads.
      (Rune of Flight additionally needs Equip and item 7f's conditional statics, so it is three things away, not one.)

    - **Layers 1–5** — a layer 6 grant whose ability generates a layer 4 or 5 effect. This is *not* scheduled, and not because it is hard. CR 613.8a(a) confines dependency to a single layer, so the CR itself supplies no mechanism for a later layer to reach back into an earlier one; any ordering we picked would be invented rather than implemented. Searched Scryfall for granted statics that define a type, color or subtype — every hit is a false positive (quoted text inside a granted *activated* ability, plus Animate Dead's enchant clause). Real grants are of triggered abilities, activated abilities, keywords, or layer 7 statics. Revisit if a card ever appears; there is nothing to build against today.

7c. **CR 613.6 "existence persists once started" — implemented, untested.** The `started` set in `apply_effects` keys on `EffectGroup`, so an effect that has begun applying keeps applying even if a later layer removes its ability. No test: every construction available today puts the strip in the *same* layer as the effect's first part, so the correct answer depends on 613.8 dependency ordering, and a test now would pin the timestamp-only answer that 613.8 must change. See item 8.

7d. **`ContinuousEffect { id: 0 }` as "unassigned" — code smell, ~20 sites.** `ContinuousEffectRegistry::add` overwrites the field, so every construction site carries a meaningless value. The fix is a `ContinuousEffectDraft` that `add()` consumes, which changes `add`'s signature and every site — its own small refactor.

7f. **Conditional static abilities are unmodeled.** `register_static_effects` handles `Effect::Atom` and `Effect::Sequence` and `continue`s on everything else, so `Effect::Conditional` — "as long as [X], this has [Y]" — registers nothing. Wanted by a large class of real cards, and by Layer 2 in particular. **Dog Umbra** is the worked example: "As long as another player controls enchanted creature, it can't attack or block. Otherwise, this Aura has umbra armor." A conditional static whose condition is *control*, so applying a Layer 2 effect changes which abilities the Aura has. (Umbra armor is CR 702.89a; 702.89b renamed the older "totem armor" wording in Oracle, so use umbra armor.)

    Historical note, because it cost a round trip: an `EffectModification::can_change_abilities()` gate briefly skipped the CR 613.7a existence check when nothing in the registry could change an ability set. It was worth 5-8x, and it was **removed** — it was valid only while no static ability is conditional, and it would have failed globally rather than arm by arm once they are. A rules engine has nothing to trade for a silently wrong answer. `layers-architecture.md` §12 records the measurements and the answer-preserving alternatives.

7e. **Derivation silently drops non-`Fixed` amounts — ✅ done (2026-08-22).** `EffectModification::{SetPowerToughness, ModifyPowerToughness}` now carry a `PtValue`: `Fixed(i32)` for a signed literal, `Dynamic(AmountExpr)` for an expression re-evaluated at every layer by `compute::evaluate_pt_value`. Two variants rather than one because `AmountExpr::Fixed` is `u64` and `ModifyPowerToughness { power: -1 }` needs a sign. Resolution-time effects stay `Fixed` (CR 608.2h locks a resolving spell's value in); it is static abilities that must stay live (CR 604.7). March of the Machines is back to its printed "equal to its mana value" via `AmountExpr::AffectedManaValue`.

    Residue, now loud instead of silent: the evaluator returns `Option<i32>` and `debug_assert!`s on an amount with no static-context meaning. `Variable` genuinely cannot appear on a static ability (CR 107.3's X is chosen as a spell is cast), but the `Target*` family points at a real vocabulary gap — see item 3.

8. **CR 613.8 dependency — two known-wrong cases, both Blood Moon.** Under timestamp-only ordering the engine gets both of these wrong. They are the concrete motivating cases for the 613.8 phase, and together they show why 305.7 is applied per-effect: dependency detection needs effect identity to hang a relation on.

   - **Rootpath Purifier** ("Lands you control and land cards in your library are basic") changes the set of permanents Blood Moon affects, so Blood Moon *depends* on it and applies second regardless of timestamp — Blood Moon never touches that player's lands. We get this wrong whenever Blood Moon has the earlier timestamp.

   - **Intra-layer re-evaluation is part of 613.8, and the written design omits it.** `layers-architecture.md` §5 orders each layer once via `resolve_order_within_layer`, then applies in that order. The CR re-evaluates dependencies **after each effect is applied** — the judge walkthrough of Blood Moon + Ashaya + Opalescence + Urborg applies one independent effect, recomputes every remaining pair, and repeats, which is how "Urborg no longer has an effect so we're done in layer 4" falls out. §9's hybrid algorithm needs to run inside that loop, not once per layer.

   - **CDAs are not in the DAG. Here is what to do if that ever costs us.** `engine/layers/cda.rs` applies characteristic-defining abilities intrinsically, so no CDA is ever a registry row. 613.8a(c)'s *first* clause — "neither effect is from a characteristic-defining ability" — therefore holds structurally, for free. Its *second* clause, both-CDA dependency, is currently unreachable, and the reason is worth stating precisely rather than filed as "can't do".

     **What would trigger it.** A CDA that reads a characteristic another CDA sets **in the same layer**: the hypothetical "this creature's power is equal to the greatest power among other creatures on the battlefield". 604.3a(3) bars a CDA from *affecting* another object but not from *reading* one, and 613.8a(a) requires the same layer, so this is the only shape that qualifies. Every printed CDA reads non-layer information (graveyards, hands, life) or strictly lower-layer information — Nightmare and Master of Etherium read Layer 4 type counts at Layer 7a — and those are independent under 613.8a(a). Searched Scryfall for both same-layer shapes: zero cards.

     **What we would get wrong.** Two copies of that card depend on each other, which is a *loop*, and 613.8b resolves loops by ignoring dependency and applying in timestamp order — so the symmetric case needs nothing. The asymmetric case is the live one: one power-reader plus any other Layer 7a CDA (a Tarmogoyf). The power-reader depends on Tarmogoyf and Tarmogoyf does not depend back, so 613.8b makes Tarmogoyf apply first and the reader must see its **post-7a** power. `evaluate_amount` resolves other objects at `compute_to_ceiling(other, layer_index)` — the frame as of the *end of the previous layer* — so it would read Tarmogoyf's pre-7a value.

     **The fix, and why it is not extra work.** The root cause is not that CDAs are intrinsic; it is that `compute_characteristics` walks **one object at a time** while CR 613 describes a board-wide pass per layer. The per-object walk with a descending ceiling (`layers-architecture.md` §5.2) is an optimization that is exact exactly while no two objects have a same-layer dependency — which is also the condition 613.8 exists to handle for registry effects. So:

     1. Make the unit of ordering in a layer an *application* rather than a registry row: either a `ContinuousEffect` row or one object's intrinsic CDA application. The intrinsic pass already produces `EffectModification`s, so this is a wrapper type, not a redesign — it is the whole of what CDAs need to join the DAG.
     2. `resolve_order_within_layer` then sees both kinds. 613.8a(c) prunes CDA↔non-CDA pairs; 613.3 keeps CDAs ahead of the independents.
     3. Termination stops being "the ceiling descends" and becomes "the dependency graph is acyclic", with 613.8b's loop rule supplying the acyclicity: a loop means no edges, so members apply in timestamp order.
     4. Mechanically that wants a per-layer memo keyed `(ObjectId, layer_index)` holding the **post-layer** value, filled in dependency order. Note the honest limit of the cheap version: marking a key in-progress and falling back to its pre-layer value on re-entry *approximates* 613.8b rather than implementing it — 613.8b has loop members apply in timestamp order relative to each other, so the second one does see the first one's result. Getting that exact means applying the layer board-wide in one sequential pass, i.e. a `compute_all(game)` with today's single-object entry point as a projection of it.

     Step 4 is the expensive one and it interacts with §12's cross-call memoization, which is already scheduled between Layer 2 and 613.8 — they should land together. Steps 1–3 are cheap and are the ones the 613.8 phase would do anyway.

   - **A test is waiting on this.** CR 613.6's "an effect that started applying keeps applying even if its ability is removed" is implemented (item 7c) but untested, because every construction available today puts the strip in the same layer as the effect's first part. Once 613.8 lands, that test can assert a stable answer.

   - **Urborg, Tomb of Yawgmoth.** Urborg is itself a Legendary — therefore nonbasic — Land, so Blood Moon turns Urborg into a Mountain and CR 305.7 strips the ability generating Urborg's effect. Applying Blood Moon changes the *existence* of Urborg's effect (613.8a(b)), so Urborg is dependent and applied last, by which point it does nothing. **Blood Moon wins in both orders.** There is no reverse dependency: Urborg grants the Swamp subtype, never the `Basic` supertype, and CR 305.8 makes a land nonbasic on the supertype alone. We currently produce order-dependent results here. Fixing it needs 613.8 *and* item 7 (a stripped static ability must retire the effect it registered at ETB) — 613.8 alone is not sufficient. `phase_ld_cards::urborg_effect()` is deliberately an Enchantment so the 305.6 tests don't depend on any of this.

9. **Abilities granted to cards outside the battlefield — ❌ inexpressible.** The layer system can only apply filter-based effects to permanents: `effect_applies_to` returns `false` for any object not in `game.battlefield` (`engine/layers/compute.rs`), and the filter type is `PermanentFilter`. So a whole class of real cards has no representation — Yawgmoth's Will and Underworld Breach (flashback on graveyard cards), Aminatou, Veil Piercer ("Each enchantment card in your hand has miracle"), Future Sight and Bolas's Citadel (playing off the library), foretell-style grants on face-down exile.

   Two pieces are needed, in this order:
   - **A card filter and a zone-aware `AffectedSet`,** so the effect can say which zone it reaches. This is the actual blocker; it is a type change, not a tuning problem.
   - **Timestamps must move off `BattlefieldEntity` and onto the object.** CR 613.7d gives an object a timestamp when it enters *any* zone; we store one only on `BattlefieldEntity`. Wonder ("as long as this card is in your graveyard and you control an Island, creatures you control have flying" — a static ability functioning from the graveyard, CR 113.6b) has nowhere to read one from, so `GameState::static_effect_timestamp` has no answer for it. Its `None` arm is unreachable today only because `register_static_effects` is called from `place_on_battlefield`.

   - **A `reachable_zones` bitmask on `ContinuousEffectRegistry`,** maintained on add/remove. **This now has a home:** `RegistryScopeSummary` exists (`state/continuous_effects.rs`), recomputed on every `add`/`remove`, carrying the one field the CR 613.7a existence check needed. `layers-architecture.md` §5.1 already specifies `touches_hidden_zones` / `touches_stack` / `has_active_cdas` on that same struct — extend it rather than adding a parallel counter. `compute_characteristics` checks the object's zone against it and returns base characteristics on a miss. This keeps the cost at zero until someone actually plays a zone-reaching card, and even then confines it to the one zone that card reaches — queried on demand at castability-check time, never as an eager sweep over every card in the game.

   **Narrowed by the CDA phase (2026-08-22).** This item once carried CR 604.3's "CDAs function in all zones" as well. It doesn't: a CDA has no filter (CR 604.3a(3)), so it never needed a zone-aware `AffectedSet`, and it now works in every zone via the intrinsic pass. What remains here is the original thing — *filter-based* effects reaching other zones. Note for whoever builds the `reachable_zones` fast path: it must not early-out an object that has a CDA of its own, which is why `apply_effects`' existing fast path already has a third term.

   The mask also generalizes the existing fast path, which today early-outs only when the registry is *entirely* empty: with it, a card in hand early-outs even with many battlefield effects registered. Worth building **with** the first zone-reaching card, not before — there is nothing to test against otherwise. Note that Aminatou additionally needs item 3 (the cost-modification pipeline) for "its miracle cost is equal to its mana cost reduced by {4}".

10. **The `Layer` enum is missing a sublayer split — doc/code drift.** (The CDA half of this item is ✅ done, 2026-08-22; see below.)

    **Layer 1a / 1b.** CR 613.2 splits layer 1 into face-down effects (1a) and copy effects (1b), and `layers-architecture.md` §7 specifies both variants and says Phase LA ships them. The enum has a single `Layer1Copy`. Order matters: a Clone copying a face-down creature copies the 2/2 colorless characteristics, not the printed card (CR 707.2). Not reachable today — nothing produces a layer 1 effect. `LAYER_ORDER` in `engine/layers/compute.rs` mirrors the enum, so splitting it later just lengthens that array; the frame-cache ceiling is an index into it, computed at runtime.

    **~~Keywords are abilities, and we model some of them as markers.~~ ✅ resolved (2026-08-23, Layer 6 phase).** The old entry framed this as "`Primitive::GrantKeyword(Equip)` would set a flag and grant no ability", which understated it. CR 702 has 189 keyword abilities and they do not want one representation. The axis is **does the engine branch on the keyword, or execute it**, crossed with whether it takes a parameter: ① branch/no-param is a flag (flying, trample, vigilance); ② branch/param is a set of *values* (protection from [quality], [type]walk); ③ execute/no-param is a plain `AbilityDef` (storm, prowess, **devoid** — already modelled this way in `phase_le_cards`); ④ execute/param is an `AbilityDef` with args (equip [cost], ward [cost], cycling [cost]).

    `KeywordAbility` was renamed **`KeywordFlag`** and narrowed to quadrant ① — 16 variants, every one consumed by combat, SBA, casting or damage. `Enchant`, `Equip`, `Landwalk`, `Protection` and `Ward` were removed; none had a single construction anywhere in the crate, and there was no exhaustive `match` over the type, so it cost nothing. The full quadrant map lives in the type's doc comment, which is where the next person will need it.

    Why it mattered *here* rather than at Phase 8: the printed case is the common one. `equip {3}` on every Sword and `equip {1}` on Skullclamp can't put the cost in a fieldless variant, and CR 702.6d lets a permanent hold several equip abilities, which a `HashSet` of one variant structurally cannot express. Enchant was worse than unmodelled — it duplicated `CardData::enchant_filter`, which already works and is what the Aura targeting path reads. And leaving `Protection` in place would have made `GrantKeyword(Protection)` look like the way to write "target creature gains protection from the color of your choice" — common Magic, and inexpressible.

    Where the removed five go when their mechanics land: Equip → `AbilityType::Activated` with its cost; Protection and Landwalk → quadrant-② frame fields (`protections: HashSet<Quality>`, `landwalk`); Ward → `AbilityType::Triggered`, so it waits on CR 603; Enchant → nothing to build.

    **Residue, all small and all recorded rather than built:** quadrant ② has no frame representation (build it with the first card that needs one); naming an ability by its keyword wants a separate complete-CR-702 `KeywordName` enum, a different type doing a different job, wanted by UI display and keyword-matters cards; and `KeywordFlag::Hexproof` is CR 702.11's fieldless base form, while "hexproof from [quality]" (702.11d) is quadrant ② and unmodelled.

    **~~Keyword counters carry no timestamp.~~ ✅ fixed (2026-08-23), in the same PR that introduced it.** CR 122.1b keyword counters are layer 6 effects and CR 613.7c timestamps every counter, so they have to interleave with the layer's registry rows rather than follow them — Humility with a later timestamp than a flying counter really does strip that flying.

    `BattlefieldEntity::counters` is now `HashMap<CounterType, CounterStack>`, carrying a count and a timestamp. CR 613.7c's second sentence — "each counter of that kind receives a new timestamp identical to that of the new counter" — is what makes one timestamp *per kind* exact rather than a simplification, and it is applied on every add. `GameState::add_counters(id, kind, n)` is the entry point; `BattlefieldEntity::add_counters` still takes an explicit timestamp because it cannot allocate one.

    The sizing that made this look expensive was wrong and worth recording as a lesson: "17 `add_counters` call sites" counted 16 tests as if they were cost. There was **one** production caller (planeswalker loyalty at ETB) and **one** direct `.counters` reader outside `battlefield.rs`. Count production call sites, not grep hits.

    Layer 7c still applies its +1/+1 and -1/-1 counters after the registry slice rather than merging. That is now justified rather than approximated: every layer 7c effect is an addition, so the layer is order-independent. If a non-commutative 7c effect ever exists, it needs the same merge layer 6 has.

    **CDAs — ✅ done (2026-08-22),** and not the way §6 designed. `Layer::Layer7aCdaPT` is in the enum and in `LAYER_ORDER` (now 10 entries). Tarmogoyf and Culling Drone (Devoid) are in `cards/phase_le_cards.rs`.

    §6 planned `ContinuousEffect.is_cda` plus CDA-first partitioning of each layer's registry slice. **The registry holds no CDAs at all.** CR 604.3a(3) — a CDA "does not directly affect the characteristics of any other objects" — is a criterion, not an observation, so every CDA applies to exactly the object that has it. There is nothing for an `AffectedSet` to select. `engine/layers/cda.rs` applies them off the object's own effective ability list at Layers 4, 5 and 7a, ahead of that layer's registry slice, and `ContinuousEffectRegistry::add` asserts nothing registers into 7a. CR 613.3's ordering, CR 613.7a's existence check, and CR 613.8a(c)'s first clause all fall out of that rather than being built.

    **This unblocked item 9 rather than depending on it.** The old claim here — "CR 604.3 makes CDAs function in all zones, which ties it to item 9 as well" — was wrong. Item 9 is about *filter-based* effects reaching other zones; a CDA has no filter, and `compute_characteristics` reads `game.objects`, so a Tarmogoyf in a graveyard has a power and toughness with none of item 9's work. `get_effective_power`/`get_effective_toughness` dropped their battlefield gate accordingly.

    **Provenance (CR 604.3a(2)) is not on the flag.** `AbilityDef.is_characteristic_defining` carries only the four criteria that are properties of the ability's text. Whether the ability was *printed on the object it affects* depends on how it got there, and the same `AbilityDef` can arrive by several routes — so it is maintained by whoever writes the ability onto the object. Copy (Layer 1) and text-changing (Layer 3) effects hand the def over whole, which is exactly what 604.3a(2) wants. **A future Layer 6 `GrantAbility` must clear the flag on the def it grants:** a granted ability is never a CDA however its text reads. That is the one debt this design leaves, and it belongs to the Layer 6 phase.

    Still open: **CDA↔CDA dependency** (613.8a(c)'s second clause) — see item 8.

### ~~Test-support duplication — cross-cutting~~ ✅ done (2026-08-22)

**`tests/common/` could not reach unit tests, and the helpers had forked.** This was
structural, not laziness: integration tests link the crate as an external dependency, so
they cannot see `#[cfg(test)]` items, and unit tests inside `src/` cannot see
`tests/common/`. Nothing shared could live in either place.

Fixed with `pub mod test_support` in the library (`src/test_support.rs`), behind a
`test-support` cargo feature the crate enables for itself via a dev-dependency on itself.
`cargo test` and `cargo build --all-targets` turn it on; a plain `cargo build`/`--release`
does not build dev-dependencies at all, so the module is excluded from release artifacts —
verified by grepping the release rlib. Cargo resolves the self-dependency to a *single*
compilation of the crate, so it costs no extra build time. `tests/common/mod.rs` is
deleted; its callers import from `mtgsim::test_support`.

**The count was worse than this entry recorded.** It named three copies of
`registered`/`make_effect` and warned about a fourth. There were eighteen: `layers::compute`'s
test module alone carried 15 inline `ContinuousEffect` literals of exactly that shape,
because each test wrote the struct out rather than reaching for a helper it could not see.
The same held for card factories — the Forest builder appeared four times, Lightning Bolt
twice, Pacifism twice, `set_attacking` three times.

**Four things are deliberately NOT unified. Each looks like leftover duplication and will
invite a "cleanup" that quietly changes what a test runs against:**

- `put_on_battlefield` (routes through `place_on_battlefield`, so ETB counters and
  static-effect registration fire) vs `place_bare` (inserts a `BattlefieldEntity` directly,
  firing neither). The combat tests need the second; collapsing them would put rows in the
  continuous-effects registry those tests do not expect.
- `put_on_battlefield` backdates entry to turn 0 (not summoning-sick) vs
  `put_on_battlefield_this_turn`, which does not. `GameState::new` starts at `turn_number:
  1`, so `layers::cda`'s two-argument version really was the second one, not a shorter
  spelling of the first.
- `registered` (`AffectedSet::Fixed(vec![id])`) vs `registered_source_only`
  (`AffectedSet::SourceOnly`). These agree in `effect_applies_to` when the source is the
  only member, so they are interchangeable *today* — but they are different variants, and
  `state::continuous_effects`' tests were written against `SourceOnly`.
- `setup_game_with_creature` in `engine/actions.rs` vs `engine/resolve.rs` — same name,
  different bodies (the first uses `place_on_battlefield`, the second inserts with
  timestamp 0 / turn 1).

Likewise `combat/validation.rs::place_creature_with_keywords` keeps its own body despite
the shared name: it takes keywords before P/T and builds a creature with no color and no
mana cost. Where a local signature differed from the shared one, the local name survives as
a one-line wrapper, so no test body changed anywhere in the migration.

The argument for doing this before the next phase was right, and the CDA merge demonstrated
it mid-refactor: `AbilityDef` gained `is_characteristic_defining` and broke the same inline
Lightning Bolt literal in two files at once.

### Before card breadth (Phase 8)

1. **CR 208.3 — a noncreature permanent has no P/T.** "A noncreature permanent has no power or toughness, even if it's a card with a power and toughness printed on it (such as a Vehicle)." `get_effective_power`/`get_effective_toughness` return the printed numbers for an unanimated Vehicle. Pre-existing and unreachable today — no Vehicle is implemented — but visible now that those accessors are no longer gated on the battlefield (CDA phase, 2026-08-22). Fix belongs with the first Vehicle: gate on `chars.types.contains(Creature)` for battlefield objects only, since CR 208.3's *other* half deliberately keeps P/T on a card outside the battlefield.

2. **Named counters have no representation — `CounterType` is a closed enum.** CR 122.1 lets a counter be named anything, and "counters with the same name or description are interchangeable" makes the *name* the identity. Most named counters have no rules meaning at all: the card counts its own counters and nothing in the engine cares what they are called.

   **Breadth, measured 2026-08-23:** a ~1000-card Scryfall sample of `o:/counters? on/` yields **115 distinct counter-name words** — charge, time, oil, quest, age, storage, lore, doom, plan, flood, bounty, egg, energy, scream, page, delay, gold, fuse, mire, ice, verse, luck, ki, collection, spore, slumber, book, burden, filibuster, and on. One sample, not the whole set. A variant per name is not viable.

   **The split is the same one `KeywordFlag` uses.** CR 122 enumerates every counter the rules branch on, and it is a short closed list: 122.1a +X/+Y, 122.1b keyword, 122.1c shield, 122.1d stun, 122.1e loyalty, 122.1f poison, 122.1g defense, 122.1h finality, 122.1i rad, plus 122.3's +1/+1 ÷ -1/-1 annihilation. Those stay variants because engine code is keyed on them. Everything else is a name and a count.

   **`CounterType::Charge` is already on the wrong side of that line** — no CR entry, the single most common vanilla counter in Magic, a variant only because one card needed it. It moves with this work.

   **Shape:** `CounterType::Named(...)` carrying a `&'static str`. The constraint is that `CounterType` is `Copy` and a `HashMap` key with five by-value signatures, so `String` and `Arc<str>` are both out — they would ripple through all of them. `&'static str` is correct permanently: every card is a `CardDataBuilder` call compiled into the binary, there is no serde, no file I/O and no deserialization anywhere in `src/`, and Scryfall is a research tool for contributors, never a runtime dependency.

   The one open question is whether to wrap it in a `CounterName` newtype with `const` values per name. The argument for it is typo discipline, not future-proofing: with 100+ hand-authored names spread across `src/cards/*.rs`, `"charge"` misspelled once silently creates a second, unrelated counter kind that no test would catch. Decide when the first named counter lands.

   Nothing is lost to a catchall: CR 122.4 ("can't have more than N counters of a certain kind") and CR 122.7 ("when the Nth [kind] counter is put on") are both generic over *kind* and never need to know what a counter means.

   **Build it with the first card that needs a named counter, not before** — there is no consumer today, `CounterType::keyword_granted()` already returns `None` for anything unrecognized, and a representation with nothing to test against is what item 9 warns about.

### Before Triggered abilities (CR 603)

The trigger dispatcher's designated insertion point is `engine/priority.rs:234-240`. Today's gaps:

1. **Trigger dispatcher stub.** `let triggers_placed = false; // Phase 7 stub` at `engine/priority.rs:235`. This is the single-point insertion.

2. **Event shape audit.** Every `events.emit(...)` call site is a potential trigger source. Before wiring triggers, audit that:
   - Events are emitted at the correct granularity (e.g., `PermanentEnteredBattlefield` fires per-permanent, not per-batch).
   - Event timing is post-action, not pre-action, so triggers observe the completed state change.
   - Events carry enough context for trigger predicates (controller, source, type filters).

3. **LKI formalization.** Several dies-handling sites already read `self.objects.get(&id)` *before* `move_object` to capture pre-move state (see `engine/sba.rs` dies handlers). This is ad-hoc LKI. Triggered abilities that reference "the creature that died" need a formalized `LastKnownInformation` snapshot mechanism, especially after layers land (LKI needs *post-layer* characteristics at moment-of-death, per rule 603.10 / 608.2h).

### Before Commander (CR 903)

1. **Commander damage increment — ✅ done (2026-04-18).** `GameObject.is_commander: bool` added; `execute_action(DealDamage)` accumulates `commander_damage_taken[source]` when `is_combat && target == Player && source.is_commander`. 5 unit tests. The loss SBA (`engine/sba.rs:73`) now has a live writer.

2. **Commander setup hook.** Nothing yet flips `is_commander = true` at deck construction or game setup. Needs a `GameConfig::commander()` constructor + a designation step (probably a field on `Decklist` or an analogous role entry). No tests exercise this yet — the flag is only set via direct field mutation in unit tests.

3. **`GameConfig::commander()` constructor.** Must wait on Replacement for 903.9 command-zone redirection to actually function; can be implemented earlier as a stub that produces a "partial Commander" game with life=40 + commander damage working but no zone redirection.

4. **Multiplayer priority rotation** (CR 800) — blocking for 3+ player Commander; not blocking for 2-player Commander.

### Cross-cutting — keep this section honest

**`fuzz_games --seed N` does not reproduce a run, and the perf protocol assumed it did.** Measured 2026-08-23 on `main` at 200 games / seed 12345: three consecutive runs of the same binary gave 25.9, 27.6 and 28.0 average turns per game. The seeding itself is fine (`game_seed = master_seed + game_num`, `StdRng::seed_from_u64`). The divergence is `HashMap` iteration order — `game.battlefield` is a `HashMap`, `oracle/legality.rs` iterates its keys to build the legal-action list, and Rust's default hasher is seeded per *process*, so the random AI sees a different action order every run and the games branch apart.

This is exactly the failure `engine/layers/land_types.rs::basic_land_types_sorted` has a comment warning about; that function sorts for this reason. The bug is live one level up.

Two consequences. Reproducing a reported panic from a seed does not work. And the documented perf protocol — "200 games / seed 12345, back to back, ±3% band" — is comparing *different amounts of work*, so a real regression can hide inside the spread and a phantom one can appear. Until it is fixed, normalize to **milliseconds per turn** and take a median of five, which is what the Layer 6 phase did: it turned an apparent +6.7% into a measured -4.0%.

The fix is an ordered iteration at the decision boundary — either a `BTreeMap`/sorted-key sweep in `oracle/legality.rs` and `oracle/mana_helpers.rs`, or a deterministic hasher on `game.battlefield`. Not attempted here; it is its own change with its own measurement.

- Every new forward-looking stub, TODO, or half-wired abstraction gets a line here at commit time.
- When a migration is completed, strike the line (keep it visible in history for a few revisions, then remove).
- Migrations that are substantial enough to warrant ticketing get a link from here to their ticket; tiny migrations are just done inline.
