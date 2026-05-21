# Codebase State — CR Coverage Map

Ground-truth snapshot of CR coverage. Single source of truth — if another planning doc contradicts this, this wins. Last grounded-in-code audit: 2026-04-18.

---

## TL;DR

- **Code size:** ~15,000 lines of Rust across 58 `.rs` files. 433 tests (384 unit + 48 integration + 1 doc-test), 0 warnings, fuzz harness runs 250-game batches.
- **Well-covered:** CR 1 (game basics), CR 3 (card types), CR 4 (zones), CR 5 (turn structure), CR 7 (keyword abilities + SBAs).
- **Partially covered:** CR 6 (casting: pipeline skeleton + X/alt/additional-cost landed, mode choice + distribution + activation restrictions pending). CR 1 mulligan is a stub. Equip and Bestow (CR 702.6, 702.103) not started.
- **Not started:** **Layers (CR 613) core** (scaffolding *is* in place — see below), **replacement effects (CR 614–616)** beyond a stub hook, **triggered abilities (CR 603)** beyond an enum variant, CR 800 multiplayer priority/turn rotation.
- **Layer system nuance:** the framing "layers not started" undersells it. Layer-annotated `Primitive` variants exist, `BattlefieldEntity.timestamp` exists for 613.7 dependency ordering, a pre-layer `power_modifier`/`toughness_modifier` scalar shim powers `get_effective_power`/`get_effective_toughness`, and `oracle/characteristics.rs` exposes Phase-5-aware wrappers (`has_keyword`, `is_creature`, `get_effective_power`, `get_effective_name`) that all engine consumers already call through. What's missing is the core: `Layer` enum, `EffectiveCharacteristics` struct, continuous-effect registry, `compute_characteristics`. Resolution of layer primitives returns `NotImplemented` (`engine/resolve.rs:270–278`).
- **Commander (CR 903) — in scope, skeleton only:** command zone ✅ as a `Zone` variant + `GameState.command` field; commander damage loss SBA ✅; commander damage **increment on combat damage now wired** (2026-04-18) via `GameObject.is_commander` flag + per-source accumulation in `execute_action(DealDamage)`. Still missing: commander tax, command-zone replacement (depends on CR 614), `GameConfig::commander()`, commander designation/setup hook.
- **Biggest single block of work remaining before the engine can run real Magic:** Layers + triggered abilities + replacement effects. These are tangled — CR 613.1c says abilities themselves can be layer-modified, replacement effects depend on effective characteristics, triggers often fire on events that must be observed post-replacement. **Commander specifically depends on replacement effects (903.9 command-zone redirection) and multiplayer (800 priority).**
- **Before starting any of those systems:** see **[Deferred Migrations](#deferred-migrations)** for prerequisite cleanups owed by forward-looking scaffolding. Each target system (Replacement, Layers, Triggers, Commander) has a short list of pending migrations that don't surface as test failures until that system lands.
- **Layers has a formalized architecture doc:** `plans/layers-architecture.md` (2026-04-18). Authoritative for type shapes, module layout, sublayer enumeration, dependency algorithm, and Phase LA→LD work sequencing. A subsequent session should execute from that doc.

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
| 604 | Static abilities | 🟡 keyword statics via `has_keyword`; non-keyword static abilities ❌ |
| 605 | Mana abilities | ✅ detection + window + enumeration | `oracle/mana_helpers.rs`, `engine/priority.rs` |
| 606 | Loyalty abilities | ❌ (T19 pending) |
| 607 | Linked abilities | ❌ (T20 pending) |
| 608 | Resolution of spells and abilities — fizzle, Target vs Choose split | ✅ via T15b refactor (`TargetSpec` → `EffectRecipient`) | `engine/resolve.rs`, `engine/stack.rs` |
| 609–611 | Effects (one-shot, continuous) — one-shot only | 🟡 one-shot ✅ via `Effect`/`Primitive`; continuous ❌ |
| 612 | Text-changing effects | ❌ |
| **613** | **Continuous effects — layer system** | ⚠️ **pre-wired at boundaries, core missing.** Scaffolding in place: (a) layer-annotated `Primitive` variants exist — `SetPowerToughness` (7b), `ModifyPowerToughness` (7c), `AddAbility`/`RemoveAbility` (6), `ChangeColor` (5), `ChangeType` (4), `GainControl` (2) at `types/effects.rs:298–313`; (b) `BattlefieldEntity.timestamp: u64` exists with doc-comment referencing rule 613.7; (c) pre-layer scalar shim `power_modifier`/`toughness_modifier` on `BattlefieldEntity` powers `get_effective_power`/`get_effective_toughness`; (d) Phase-5-aware interface functions `has_keyword`, `is_creature`, `get_effective_power`, `get_effective_name` in `oracle/characteristics.rs` are the single-point change site documented in comments. **What's missing:** no `Layer` enum, no `EffectiveCharacteristics` struct, no continuous-effect registry on `GameState`, no `compute_characteristics`. Resolution of layer primitives returns `NotImplemented` at `engine/resolve.rs:270–278`. The commented stub at `types/effects.rs:365` is for the `ApplyContinuous` high-level combinator; lower-level continuous primitives are further along. |
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

### Before Layers (CR 613)

The layer system's designated single-point change site is `oracle/characteristics.rs`. All characteristic queries already route through that module. Today's gaps:

1. **Pre-layer P/T shim.** `BattlefieldEntity.power_modifier` and `toughness_modifier` are scalar fields that `get_effective_power`/`get_effective_toughness` read directly. When the layer system lands, layer 7c output replaces this shim. Every mutation site that writes these fields must become a layer-registered continuous effect. Grep-audit needed for current write sites.

2. **Direct `CardData` reads that should route through Phase-5 wrappers.** The layer-aware wrappers (`has_keyword`, `is_creature`, `get_effective_power`, `get_effective_name`, etc.) exist, but some call sites still read `obj.card_data.keywords` / `obj.card_data.colors` / `obj.card_data.types` directly. Direct reads skip the layer system entirely. Audit grep: `card_data\.(keywords|colors|types|subtypes)` outside `oracle/characteristics.rs` and `engine/cast.rs` (cast-zone legality is pre-stack, not layer-affected).

3. **Cost modification pipeline stub.** `engine/costs.rs:255-263` `apply_cost_modifications` is a passthrough with a TODO. When Layers lands (specifically L15), this wires to the continuous-effects registry for Thalia/Electromancer/Trinisphere-style modifications.

4. **Mana-pool persistence stub.** `engine/turns.rs:65, 132` pass `BlanketPersistenceSet::none()` to `empty_with_reason`. The TODO `TODO(T12c): build BlanketPersistenceSet from continuous effects layer` marks the layer-dependent wiring.

5. **Timestamps are set but never read.** `BattlefieldEntity.timestamp: u64` is populated on ETB (referencing rule 613.7 in its doc comment) but has no current reader. When layers implement dependency ordering, this is the primary input. No migration needed — just flag that the field exists and is expected to become live.

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

- Every new forward-looking stub, TODO, or half-wired abstraction gets a line here at commit time.
- When a migration is completed, strike the line (keep it visible in history for a few revisions, then remove).
- Migrations that are substantial enough to warrant ticketing get a link from here to their ticket; tiny migrations are just done inline.

