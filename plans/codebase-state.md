# Codebase State — CR Coverage Map

Ground-truth snapshot of CR coverage. Single source of truth — if another planning doc contradicts this, this wins. Last grounded-in-code audit: 2026-08-24.

---

## TL;DR

- **v1 is two use cases** (owner, 2026-08-24): peer-to-peer human games through a GUI, specifically **4-player Commander**, and **highly parallel AI games** over the CLI. Two-player Standard is a checkpoint, not the target. Ordering lives in `CLAUDE.md` → "Critical path to v1"; the consequence for this file is that CR 800/802 and CR 903 below are path items, not deferrals, and that new systems get written N-player-shaped.
- **Code size:** ~27,100 lines of Rust across 77 `.rs` files. 634 tests, 0 warnings, fuzz harness runs 250-game batches.
- **Well-covered:** CR 1 (game basics), CR 3 (card types), CR 4 (zones), CR 5 (turn structure), CR 7 (keyword abilities + SBAs).
- **Partially covered:** CR 6 (casting: pipeline skeleton + X/alt/additional-cost landed, mode choice + distribution + activation restrictions pending). CR 1 mulligan is a stub. Equip and Bestow (CR 702.6, 702.103) not started.
- **Not started:** **replacement effects (CR 614–616)** beyond a stub hook, **triggered abilities (CR 603)** beyond an enum variant, CR 800 multiplayer priority/turn rotation.
- **Layers (CR 613) — core landed, three layers live (Phases LA–LD, 2026-05 → 2026-08).** The system is real, not scaffolding: `Layer` enum with all 9 sublayer variants (`engine/layers/types.rs`), `EffectiveCharacteristics` struct (name, mana_cost, colors, types, subtypes, supertypes, keywords, abilities, P/T, controller), a `ContinuousEffect` registry with duration-based expiry (`state/continuous_effects.rs`, 304 lines), and `compute_characteristics` (`engine/layers/compute.rs`, 967 lines). Static abilities register through `GameState::register_static_effects`. `oracle/characteristics.rs` wrappers all route through `compute_characteristics`.
  - **Live layers:** 2 (control) — Layer 2 phase, 2026-08-23. 4 (types/subtypes/supertypes) — Phase LD Part A. 5 (color) — Phase LC. 6 (abilities) — Phase LF. 7a (CDA P/T) — Phase LE. 7b (set P/T), 7c (modify P/T), 7d (switch P/T) — Phase LB.
  - **Still stubbed:** Layer 3 (text) and Layer 1 (copy) are enum variants only. Nothing produces an effect in either.
  - **Dependency algorithm (CR 613.8) not implemented.** Ordering is timestamp-only, which is sufficient for the layers landed so far in isolation but will not survive Layer 6 + Layer 4 interaction (Humility/Opalescence).
  - **CR 305.7 / 305.6 — ✅ done (Phase LD Part B).** Blood Moon strips a nonbasic land's printed abilities and grants the intrinsic `{T}: Add {R}`; Urborg adds a basic land type and its mana ability without stripping. Lives in `engine/layers/land_types.rs`. `AbilityOrigin` was evaluated at Part B kickoff and **not built** — layer ordering makes it unnecessary; see `layers-architecture.md` §15.2 item 4.
- **Commander (CR 903) — in scope, skeleton only:** command zone ✅ as a `Zone` variant + `GameState.command` field; commander damage loss SBA ✅; commander damage **increment on combat damage now wired** (2026-04-18) via `GameObject.is_commander` flag + per-source accumulation in `execute_action(DealDamage)`. Still missing: commander tax, 903.9b hand/library redirection (depends on CR 614), 903.9a graveyard/exile redirection (an SBA, CR 704.6d — *not* blocked on CR 614), `GameConfig::commander()`, commander designation/setup hook.
- **Biggest single block of work remaining before the engine can run real Magic:** the CR 613.8 dependency algorithm + triggered abilities + replacement effects. These are tangled — CR 613.1c says abilities themselves can be layer-modified, replacement effects depend on effective characteristics, triggers often fire on events that must be observed post-replacement. **Commander specifically depends on replacement effects (903.9b hand/library redirection) and multiplayer (800 priority)** — though 903.9a, the graveyard/exile half, is an SBA (CR 704.6d) and needs neither.
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
| 120 | Damage — combat damage routing, infect/wither/lifelink | 🟡 combat damage ✅, lifelink ✅, first/double strike ✅, trample ✅, deathtouch ✅; infect/wither/toxic ❌ (T21c pending); **120.3c ❌ — damage to a planeswalker does not remove loyalty counters** (audit 2026-08-25: `perform_action(DealDamage)` marks damage on any battlefield object and nothing reads it off a planeswalker, while SBA 704.5i reads only the counter count, so a planeswalker can never die to damage. Unreachable — no planeswalker registered, combat can't attack one — but Lightning Bolt's "any target" already validates them, so the first registered planeswalker makes it live. Fix scheduled with Phase RD's CR 120.3 decomposition, `replacement-architecture.md` §9) | `engine/combat/keywords.rs`, `engine/combat/resolution.rs` |
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
| 601.2b | Choose modes / X / alt+additional costs | 🟡 X **chosen and paid** ✅ (X-dependent *resolution* amounts ❌ — `engine/resolve.rs:786-804` returns `Err` for a resolving `Variable`/`TargetPower`/`CountOf` amount; loud, and unreachable with no such card registered), alt ✅, additional ✅ (T18a); **mode choice ❌** (T18b pending — `ChoiceKind::ChooseModes` not added yet) | `engine/cast.rs` |
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
| **614–616** | **Replacement + prevention + interaction** | ⚠️ Stub only. `engine/actions.rs:86-89` `execute_action` is a pass-through to `perform_action` with a comment: *"Phase 6: A `apply_replacement_effects(action)` call will be inserted here"*. No `ReplacementEffect` struct. **Designed 2026-08-24 — execute from `plans/replacement-architecture.md` (phases RA–RE).** |

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
| **704.5a–w** | **State-based actions** | ✅ 704.5a (life ≤0), 704.5b (empty library draw), 704.5c (poison ≥10), 704.5d (tokens in non-BF zones), 704.5f (0 toughness), 704.5g (lethal damage with indestructible + deathtouch), 704.5h (deathtouch), 704.5i (PW 0 loyalty), 704.5j (legend rule), 704.5m (Aura illegal host), 704.5n (Equipment/Fort on illegal permanent), 704.5p 🟡 (the "attached to an illegal object" half; the *creature* clause is a TODO at `engine/sba.rs:323` — the sweep exempts every Aura/Equipment/Fortification without asking whether it is also a creature, and 704.5p's first sentence unattaches a creature regardless of what else it is. Unreachable: no enchantment animator, no attachable Equipment in the pool), 704.5q (+1/+1 / -1/-1 annihilation). 704.5s (Saga), 704.5t (dungeon), 704.5v/w/x (battle) ❌. Commander damage ✅. | `engine/sba.rs` (1015 lines) |
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

**Known multiplayer-shaped gaps in existing 2-player code** — under the v1 redefinition this list is the design checklist for CR 800, not a backlog. The cheap way to buy multiplayer is to write CR 614 and CR 603 N-player-shaped as they land (CR 616.1's affected-player ordering and CR 603's APNAP queue both take a player set in the CR); retrofitting is the expensive path. CR 800.4 elimination is the piece most likely to be underestimated — a leaving player's permanents, stack objects, and effects each need specific resolution:
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
| 903.9a | **Commander in graveyard or exile → command zone** | ❌ — but **not blocked on CR 614**. Corrected 2026-08-24: this half is a *state-based action* (**CR 704.6d**), not a replacement effect. `check_state_based_actions` already takes a `&dyn DecisionProvider`, which is all the rule's "its owner may" needs. Buildable today | `engine/sba.rs` |
| 903.9b | **Commander to hand or library → command zone instead** | ❌ requires the replacement effect pipeline (CR 614), a stub hook at `engine/actions.rs:86-89`. Carries the rules' only explicit exception to CR 614.5 ("may apply more than once to the same event") | `plans/replacement-architecture.md` § Phase RB |
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

**The phase has an architecture doc as of 2026-08-24: `plans/replacement-architecture.md`.** It is authoritative for the type shapes, the CR 616.1 pipeline, the ETB look-ahead frame, and the RA–RE sequencing; item 3 below *is* its Phase RA. This section stays the status ledger.

**RA ships as three PRs (sized 2026-08-25, `replacement-architecture.md` §9).** RA-1 = the `ActionContext` sweep + `ZoneChangeCause` (pure mechanical, ~165 call sites, zero behavior change); RA-2 = the six routing tickets (draw, tap/untap, ability events, `cast_from`, lifelink, `Cost::PayLife`); RA-3 = batch form, LKI/cause/batch-id payloads, the three bypass closures, and the death-event demotion. Ticket numbers below are stable and cited by §9.

The replacement pipeline is designed to sit inside `execute_action` at `engine/actions.rs:86-89`. Every mutating action must flow through there for replacements to observe them. Status:

1. **Zone-change migration — ✅ done (2026-04-18).** `move_object` is now `pub(crate)` with documentation directing external callers to `change_zone` / `execute_action(GameAction::ZoneChange)`. All 12 previously-direct callers (5 SBA sites in `engine/sba.rs`, `Cost::SacrificeSelf` in `engine/costs.rs`, push-to-stack + 4 rollbacks in `engine/cast.rs`, cleanup discard in `state/game.rs`) now route through the chokepoint. `engine/actions.rs::change_zone(id, to)` is the new convenience wrapper. Internal helpers (`draw_card`, `play_land`, and the `GameAction::ZoneChange` arm itself) continue to call `move_object` directly from inside `engine/zones.rs`.

2. **Open-coded zone bookkeeping — ✅ CounterSpell migrated; 3 stack.rs sites tagged as structural bypasses (2026-04-18).**
   - `engine/resolve.rs` `Primitive::CounterSpell` now calls `change_zone(id, Graveyard)` which tears down the `StackEntry` via `remove_from_zone_collection(Stack)`, then emits `SpellCountered`. No more manual bookkeeping.
   - `engine/stack.rs` three sites (permanent-spell ETB, instant/sorcery → graveyard, `handle_fizzle`) are tagged `// REPLACEMENT-BYPASS:` because the stack-pop-first pattern (see doc comment at `engine/stack.rs:27-32`) removes the object from the stack `Vec` before resolution begins, so `move_object` would double-remove. When the Phase 6 replacement pipeline lands, these sites need their own ZoneChange dispatch variant that skips the stack-Vec removal step — or the pattern needs to change. Flagging now so the decision isn't re-discovered.

3. **Event-stream refit — the CR 614 phase's opening ticket block (specified 2026-08-24, when item 4 was resolved).** What was an open-ended "audit" is now an inventory. Emission moves inside the chokepoints (`execute_action` / `move_object` / the SBA handlers) and always records the *performed* action, post-replacement. From a census of all 42 production `events.emit` sites plus the known bypasses:

   - **Route the draw-step draw through the chokepoint.** `engine/turns.rs:203` calls `draw_card` directly, bypassing `execute_action(DrawCard)`; CR 614 ("if you would draw", "skip your draw step") and every draw tracker must see it. Add a `CardDrawn` event — today a draw is a bare `ZoneChange`, and CR 121.5 makes a library→hand move without the word "draw" a *different, trigger-visible* fact (106 cards say "whenever you draw"; 54 say "draw your second card"; Tamiyo, Inquisitive Student needs the third). (`state/game.rs:117` opening-hand draws may stay direct — pregame, nothing can observe them.)
   - **Tap/untap through the chokepoint, with `Tapped`/`Untapped` events.** Four silent production sites: `engine/costs.rs:150` (`Cost::Tap`), `costs.rs:165` ({Q}), `engine/turns.rs:184` (untap sweep — may batch), `engine/combat/steps.rs:70` (attackers tap). "Becomes tapped/untapped" triggers (CR 603.2e) currently have nothing to watch.
   - **Life mutations outside the chokepoint (audit 2026-08-25).** `engine/keywords.rs:58` (lifelink) and `engine/costs.rs:184` (`Cost::PayLife`) write `life_total` directly instead of proposing `GainLife`/`LoseLife`. Both *emit* `LifeChanged`, which is exactly how the 42-site emissions census above missed them — they emit without proposing, so CR 614 life watchers (Tainted Remedy must see lifelink; CR 119.4 makes paying life a loss Bloodletter doubles) would silently never fire. RA items 11–12 in `replacement-architecture.md` §9. A third member of the class sits *inside* the chokepoint but undecomposed: `perform_action(DealDamage)` performs a player's life loss inline, so a `LoseLife`-watching replacement would miss combat damage entirely; that is CR 120.3's results-of-damage decomposition, scheduled with Phase RD (which also owes CR 120.3c — see the CR 1 map's 120 row).
   - **`AbilityActivated` plus an identity-bearing `AbilityResolved`.** `cast.rs::activate_ability` pushes to the stack without `move_object` and emits nothing; resolution emits only a generic `SpellResolved` carrying the just-deleted ephemeral's id. CR 603.7h counting (Ashling the Pilgrim; Ashling, Flame Dancer) needs `(source, ability)` identity.
   - **Payload upgrades:** a layer-computed LKI frame captured at emission instant on battlefield-leaving `ZoneChange`s (CR 603.10a); a `cause` field (destroyed / sacrificed / SBA / discarded — the sacrifice-vs-destroy semantic carrier); a batch id for simultaneity (CR 603.2c / 603.6a "one or more" triggers); resolution context on every event (which resolution emitted it).
   - **Close the three `// REPLACEMENT-BYPASS:` sites** (item 2 above) with the planned pop-aware ZoneChange dispatch variant — now also the place where the proposed move meets the replacement pipeline *and* the performed event + LKI is emitted. Not optional for v1 Commander: a commander creature spell that fizzles goes stack→graveyard through `stack.rs:172`, and CR 903.9b must be able to offer the command zone instead.
   - **Demote type-specific death events from trigger matching.** `CreatureDied` / `PlaneswalkerDied` / `LegendRuleSacrificed` become display sugar at most; matching keys on the zone-change event plus its LKI frame, so a multi-type permanent (the Gideon case) matches every applicable trigger from one event. (This is the surviving half of design_doc's 2026-04-11 decision row.)

4. **~~Unresolved architecture fork~~ — ✅ RESOLVED 2026-08-24 (owner decision): trigger detection is the performed-action event stream; the delta log is rejected.** No `engine/delta_log.rs` will exist. The shape: `GameAction` (proposed) → CR 614 replacement pipeline → perform → `GameEvent` (performed record carrying LKI frame, cause, batch id, resolution context) → **synchronous dispatch at the mutation instant** (turn trackers update; event triggers match against *effective* ability lists across all zones; state triggers and designations evaluate against live state) → pending-trigger queue → APNAP placement at the CR 603.3 moment (the `engine/priority.rs:240` stub). Detection happens per atomic event; only *placement* defers — CR 702.131d, the city's-blessing-before-SBA ruling, and CR 603.8's momentary-condition example all require exactly that split.

   `state-tracking-architecture.md` remains the statement of the four problems; its **Resolution postscript** records how each is answered and why the delta representation fails them (semantic identity is not recoverable from `(old,new)` state pairs — CR 121.5, 603.10e; evaluating past instants requires replaying the layer engine — CR 603.10, 603.6b, 702.131d). What the delta doc got right is adopted: central detection with no per-site knowledge of conditions, single-funnel emission discipline (item 3 above is that ticket list), LKI-over-type-specific-events, and resolution-context stamping. Loop detection Tiers 1–3 and D26 survive, with D26 transcripts re-based on performed-action sequences. Struck in writing: `roadmap.md` delta block + 2026-04-06 blockquote, `design_doc.md` §8 subsections + decision rows 2026-04-01 / 2026-04-11×2, and the CLAUDE.md authority-table carve-out.

   **N-player from day one:** trackers are per-`PlayerId` vectors plus global; the pending queue takes the player set and orders per CR 603.3b APNAP; CR 616.1 ordering hangs off the affected object's controller / affected player, with APNAP among simultaneous choosers.

### Before Layers (CR 613) — now DURING Layers

The layer system's designated single-point change site is `oracle/characteristics.rs`. Status as of 2026-08-19, after Phases LA–LD:

1. **Pre-layer P/T shim — ✅ done.** `BattlefieldEntity.power_modifier` / `toughness_modifier` no longer exist anywhere in `src/`. Layer 7c output replaced them.

2. **Direct `CardData` reads — ✅ done (2026-08-19).** 21 battlefield/stack call sites now route through `oracle/characteristics.rs`. New predicate helpers `has_type`, `has_subtype`, `has_supertype`, `has_permanent_type` join the existing `is_creature` / `get_effective_*` wrappers.
   - Migrated: `engine/sba.rs` (8 — planeswalker loyalty, legend rule, Aura/Equipment/Fortification attachment SBAs), `engine/targeting.rs` (7 — creature target, creature-or-planeswalker target, the whole `PermanentFilter` match), `engine/resolve.rs` (Aura ETB), `engine/stack.rs` (2 — permanent-spell routing, Aura spell), `state/game_state.rs` (ETB loyalty counters), `ui/display.rs`, `ui/random.rs`.
   - **Deliberately NOT migrated (6 sites):** `engine/zones.rs:144` (play a land from hand), `oracle/legality.rs:59` (playable lands in hand), `oracle/mana_helpers.rs` (×4 — castable spells in hand, instant/flash timing). These are cast-zone / play-from-hand legality, evaluated before the object is a permanent, so the layer system has nothing to contribute. Same exemption as `engine/cast.rs`. Each is tagged `// PRE-LAYER ZONE:` in source so a future grep audit doesn't re-flag it.
   - Regression coverage: `mtgsim/tests/layer_aware_queries_test.rs`, 5 tests. Verified to fail against the pre-fix tree and pass after.

3. **Cost modification pipeline stub — ❌ still a passthrough. Promoted 2026-08-24: this is Commander-critical, not background debt.** `engine/costs.rs:255` `apply_cost_modifications` with `TODO(L15)`. Wires to the continuous-effects registry for Thalia/Electromancer/Trinisphere — and **commander tax is a cost modification** (CR 903.8 / 601.2f / 613.11), so under the new v1 the Commander track runs through this stub.

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

7d. **`ContinuousEffect { id: 0 }` as "unassigned" — code smell, 16 sites (recounted 2026-08-24).** `ContinuousEffectRegistry::add` overwrites the field, so every construction site carries a meaningless value. The fix is a `ContinuousEffectDraft` that `add()` consumes, which changes `add`'s signature and every site — its own small refactor.

7f. **Conditional static abilities are unmodeled.** `register_static_effects` handles `Effect::Atom` and `Effect::Sequence` and asserts on everything else, so `Effect::Conditional` — "as long as [X], this has [Y]" — registers nothing and now says so loudly. Wanted by a large class of real cards.

    **Layer 2 shipped without it (2026-08-23), and it was not in the way after all.** The worked example below, Dog Umbra, is three systems away rather than one: it needs this item, *and* an "enchanted permanent" `AffectedSet` (which does not exist — `EffectRecipient` has `FilteredPermanents` and `Implicit`, neither of which names an attachment's host), *and* umbra armor, which is a replacement effect and therefore CR 614 work. No card in Layer 2's reach is conditional: Act of Treason and Threaten are unconditional, and Mind Control's "You control enchanted creature" is blocked on the attachment recipient, not on this. Still the largest item standing between the engine and card breadth; no longer coupled to any specific layer. **Dog Umbra** is the worked example: "As long as another player controls enchanted creature, it can't attack or block. Otherwise, this Aura has umbra armor." A conditional static whose condition is *control*, so applying a Layer 2 effect changes which abilities the Aura has. (Umbra armor is CR 702.89a; 702.89b renamed the older "totem armor" wording in Oracle, so use umbra armor.)

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

12. **The card → registry lowering is loud — ✅ done (2026-08-23).** `register_static_effects` had five arms that declined to lower something and `continue`d, registering nothing and saying nothing. Every one now `debug_assert!`s first.

    **Why this class is worth its own item.** A dropped atom produces a card that is *inert* — it panics nothing, computes nothing wrong, and stays perfectly deterministic. `fuzz_games` structurally cannot see it: it catches crashes and non-determinism, and a card that does nothing exhibits neither. This codebase has already paid for the pattern twice (item 7e was a `continue` on a non-`Fixed` amount that "silently dropped the whole atom and failed no test"; item 7f is the same shape still open). Refusing to be quiet at the door is the only check that catches it.

    - **The two lowering steps are now shared,** as `GameState::static_ability_atoms` and `GameState::static_affected_set`, and `resolve::register_granted_static_effects` routes through both. They had been two hand-copied matches, which is the drift `static_primitive_rows`' doc comment already warns about: identical card text must behave identically whether printed or granted.
    - **`debug_assert!` rather than a hard error,** matching the existing asserts in `register_granted_static_effects` and `compute::evaluate_pt_value`. A card author running the suite is stopped; release keeps the old skip-and-carry-on rather than panicking mid-game.
    - **Nine unit tests in `state/game_state.rs`** (`mod static_lowering`) — three positive controls plus one `#[should_panic]` per declining arm. An assertion nothing exercises is indistinguishable from one that never fires.
    - **`tests/card_pool_lowering_test.rs` puts every registered card onto the battlefield** under live assertions, for both controllers. Neither existing check reaches this: `fuzz_games` is a release build so `debug_assert!` is compiled out, and the per-phase suites each place only their own handful of cards. Verified non-vacuous by temporarily registering a conditional static — it fails by card name with an actionable message.

    **Findings from the audit, none of them bugs, all of them scope.** The existing 63-card pool lowers completely clean. But three `static_primitive_rows` arms had *no card reaching them at all*, and the reasons differ:

    - **`GrantAbility` — now covered.** `cards/phase_lf_cards::citanul_hierophants` ("Creatures you control have '{T}: Add {G}'", real card, verbatim). Every other `GrantAbility` in the pool arrives through a *resolution*, which builds `AffectedSet::Fixed` against the spell's targets — a different path from a live `Filter`. It works because the granted body is a *mana* ability and so generates no continuous effect of its own; swap it for a static body and it lands in item 7's open filter half immediately. It also crosses into mana enumeration, which is the `activatable_abilities` / `priority` / `cast::activate_ability` index coupling CLAUDE.md warns about, and which had no card able to test it end to end.
    - **`RemoveKeywordFlag` — still uncovered, and the reason is a modeling gap, not laziness.** Searched Scryfall for static keyword removal (`oracle:/creatures your opponents control lose/`, excluding instants and sorceries): 9 cards, and 8 of them also say "**can't have or gain** [keyword]". That prohibition is a CR 613.1f continuous effect the engine cannot express at all, and without it the card is not merely incomplete but *wrong* — a later grant would put the keyword back. The one clean exception is Melira, Sylvok Outcast ("Creatures your opponents control lose infect"), and `infect` is not in `KeywordFlag`'s 16 variants, while Melira's other two clauses (poison counters, `-1/-1` counter prohibition) are also unmodeled. **So: no honest card exists for this arm yet, and prohibition effects are what gate the whole class.** Worth its own item when someone reaches for an Archetype.
    - **`LoseAbility(AbilityId)` — no natural card shape.** Real cards say "loses all abilities" (Humility, covered) rather than naming one. The arm exists for effects that already hold an id. Not a gap; recorded so the next audit does not re-flag it.

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

11. **Filter `PlayerRef` resolution — ✅ done (2026-08-23), ahead of Layer 2.** `AffectedSet::Filter` carried a `controller: Option<PlayerId>` that `register_static_effects` resolved from `PermanentFilter::ByController(PlayerRef::You)` at ETB. That is a snapshot of who controlled the source when it entered, and CR 109.5 says the opposite — "for a static ability, [you] is the *current* controller of the object it's on". Glorious Anthem kept buffing the team of whoever controlled it at ETB.

    Demonstrable before Layer 2 exists, which is why it shipped as a bugfix rather than as scaffolding: CR 110.2 makes `BattlefieldEntity.controller` the default controller, `compute_to_ceiling` seeds `chars.controller` from it, and writing that field is the pre-Layer-2 half of gaining control. `tests/filter_controller_test.rs` was shown failing against the pre-fix tree.

    - **The field is gone; `permanent_matches_filter` owns the whole question.** `ByController` used to return `true` unconditionally and defer to the `AffectedSet` field, so one question lived in two functions and only one half was re-asked during the walk. That split is what let the snapshot hide, and it had a second victim: `extract_controller_from_filter` walked only `And` nodes, so `Not(ByController(You))` silently dropped its constraint and matched nothing.
    - **"You" is origin-dependent, and both arms are CR text.** `EffectOrigin::StaticAbility` → the source's *effective* controller, via `compute_to_ceiling(effect.source, layer_index)` (CR 109.5). `EffectOrigin::Resolution` → `effect.controller`, fixed when the effect began (CR 611.2c). Same `layer_index` ceiling `static_ability_still_exists` uses — never the full ceiling, per `layers-architecture.md` §5.2.
    - **All four `PlayerRef` variants resolve; none asserts.** `Opponent` is matched as a *predicate* (`controller != you`) rather than resolved to an id, because CR 102.2 makes it one player in a two-player game but CR 102.3 makes "your opponents" a set in multiplayer — the predicate is the same answer in both, and an `Option<PlayerId>` would have been the wrong shape for half the CR. `Owner` resolves to the source object's owner (CR 108.3 / 110.2); no card says it, but it is exactly determined, so asserting would be inventing a restriction.
    - **What Layer 2 hit — ✅ confirmed 2026-08-23, nothing needed redoing.** A Layer 2 effect whose own filter says "you control" asks at `layer_index == 1`, i.e. the frame *before* Layer 2 applied — `BattlefieldEntity.controller`. That is exact whenever the source is not itself under a control-changing effect, and it is the CR's own fallback when it is: two same-layer effects where applying one changes what the other applies to are dependent under CR 613.8a, a mutual pair is a dependency *loop*, and 613.8b resolves loops in timestamp order. The exact version needs the frame the check reads to be a partially-applied layer — item 8 step 4's board-wide sequential pass, the same missing piece a granted Layer 6 static ability already waits on. Nothing here needs redoing when it arrives; only the ceiling it asks at.
    - **Two fixes for the cost, both exact, and the attribution matters for Layer 2.** `effect_applies_to` runs *before* the CR 613.7a existence check, so it fires for objects the filter goes on to reject — unlike the existence check, which only fires for matches. Resolving "you" unconditionally therefore added a source-frame walk per non-matching permanent per layer. The fixes: resolve **lazily**, so a filter with no `ByController` node and an `And` that short-circuits on type both cost what they cost before; and **gate** on `RegistryScopeSummary::any_control_changing`, reading `BattlefieldEntity.controller` directly while no `SetController` row is registered, because Layer 2 is the only channel that writes `chars.controller` (no CDA lives there, no counter touches it) so the walk's seed *is* its answer.

      All four combinations, interleaved, 200 games / seed 12345, against a 73.0 ms/game pre-refactor baseline:

      | | gate on | gate off |
      |---|---|---|
      | **lazy** (shipped) | 74.8 | 78.1 |
      | **eager** | 82.2 | **749** |

      The 749 belongs to *eager and ungated together*. An earlier revision of this item pinned it on the gate, which would have told the Layer 2 phase to expect a 10x when the flag starts coming on. **It should expect the 78.1 cell — +7% over baseline, and an upper bound at that,** since the measurement forces the walk on every board while the real flag is only true while a `SetController` row exists. Laziness is the half that has to survive future refactoring here; the gate is a further ~4%.

      **How the prediction held, measured 2026-08-23 with Act of Treason in the card pool** — see item 13 for the full numbers. The **+7% ceiling was right and generous: the phase cost +4%**. The **~4% attributed to the gate was wrong by an order of magnitude in the other direction: it is now +28%,** because the phase put 20 more call sites behind it, several inside per-permanent sweeps. And the **sharper gate was built, measured and discarded** — it is not faster, because `ObjectId` is a v4 UUID and the set probe costs a SipHash at every migrated call site on every board to save on the rare one. The bigger lever for this whole class remains `layers-architecture.md` §12's cross-call memoization, already scheduled between Layer 2 and 613.8: the frame cache is discarded per top-level call today, so a board-wide sweep recomputes each source's frame once per object it looks at.
    - **One thing this did NOT fix, recorded as a note rather than an item.** CR 611.2c also says a resolution effect's affected *set* locks in when it begins; an `EffectOrigin::Resolution` row over an `AffectedSet::Filter` still re-filters on every walk, so ATOM-611.2c-001 stays uncovered. No card produces that combination — all three production `Filter` construction sites are static abilities, where re-filtering is what CR 613.7a wants — so there are zero call sites to migrate. It becomes real the first time a resolving spell wants a filter instead of a target list.
    - **Cost: none measurable.** Pre- and post-fix binaries built side by side and run **interleaved**, 200 games / seed 12345: pre 82.0 / 81.0 / 82.0 / 80.6, post 83.7 / 82.5 / 83.0 / 76.5 ms/game — same mean, and the spread inside each set is wider than the gap between them. Measuring the two in separate batches first showed a phantom +7%; interleave, or the drift the perf protocol already warns about invents a regression. Game outcomes are byte-identical to the pre-fix tree at seed 12345, and three runs at one seed still agree byte for byte.

13. **Layer 2 — control-changing effects (CR 613.1b) — ✅ done (2026-08-23).** The layer is live end to end: `Primitive::GainControl` lowers through both channels, Act of Treason is registry-eligible, and all seven of the corpus's Layer 2 atoms are covered.

    - **`EffectModification::SetController` carries a `PlayerRef`, not a `PlayerId`.** A resolved id in a registry row is a snapshot of who controlled the source at registration, which is item 11's bug in a new variant: CR 109.5 makes a static ability's "you" the *current* controller, so Mind Control's "You control enchanted creature" follows the Aura when the Aura changes hands. `compute::resolve_set_controller` resolves it during the walk through the same `FilterPlayers` a filter's `ByController` uses, so both halves of CR 109.5 have one implementation. It also keeps `static_primitive_rows` a pure map from primitive to rows — that table has no game and no source, so a `PlayerId` would have left `GainControl` in the `_ => Vec::new()` arm item 12 exists to empty.

      `PlayerRef::Owner` deliberately means something different here than in a filter: in `ByController` it describes the *source*, here it describes the object being moved (Homeward Path hands each creature to its own owner). The `Opponent` arm and what it does not cover are below.

      **Which player identities may stay symbolic, and the gap that follows.** The layer walk is a pure read — it cannot prompt, and it runs many times per game state — so a `PlayerRef` survives into a registry row only when it has to be *re-derived* every walk: `You` (CR 109.5 makes a static ability's "you" the source's current controller) and `Owner` (fixed by CR 108.3, free to recompute). Every other identity is settled when the effect is created and stored as `Player(pid)`.

      That is not a restriction on what cards can say, and the cards make the point:

      | Card | New controller is | Settled |
      |---|---|---|
      | Akroan Horse, Fateful Handoff, Rainbow Vale (9 cards, `o:"opponent gains control" -o:"target opponent"`) | "an opponent", **not** targeted | at resolution — the Akroan Horse ruling is explicit that "in a multiplayer game, you choose the opponent as the ability resolves" |
      | Risky Move | "that player", from a per-player trigger | at trigger resolution |
      | Scrambleverse | a player chosen at random | at resolution |
      | Illicit Auction | whoever bid the most life | at resolution |
      | Donate, Harmless Offering | a targeted player | at cast (CR 601.2c) |

      **What is missing is the lowering, not the type.** `Primitive::GainControl(Duration)` carries no player and lowers to `PlayerRef::You`, so "an opponent gains control" has no representation today — the 9-card group above needs `GainControl` to name a recipient and the resolution arm to make the choice through the `DecisionProvider` when more than one opponent exists. `PlayerRef::Opponent` is exact and free in a two-player game (CR 102.2 leaves nothing to choose), which is what `compute::resolve_set_controller` resolves; above two players it asserts, because reaching the walk means that choice was skipped.

      A new `PlayerRef` variant is **not** the fix for the computed cases and would be the wrong shape for them: "the player with the highest life total", an auction winner, or a random player are computations over game state at one instant, and re-running them on every layer walk would let the answer drift between walks of an unchanged registry.


    - **`BattlefieldEntity.controller` was read directly at 20 sites; all 20 migrated** to `oracle::characteristics::get_effective_controller` / `controls`. Same shape and same silent-failure mode as Phase LD Part B's 21 `card_data` reads. **No `// PRE-LAYER ZONE:` exemptions were tagged** — that class is cast-zone and play-from-hand legality, which runs before the object is a permanent, and every site here asks about something already on the battlefield or the stack.

      Two needed more than a substitution. `stack.rs::resolve_top_of_stack` reads the controller *before* the pop, because a spell's controller lives on the `StackEntry` and the pop destroys it — one value now feeds both the `ResolutionContext` (CR 608.2) and the entering permanent's controller (CR 110.2b). `turns.rs`'s untap sweep needs two passes, since the predicate is a `&self` layer query and the untap is a `&mut self` write.

    - **CR 302.6 lives in the frame, not on the battlefield.** `BattlefieldEntity.controller_since_turn` could not be maintained: control from a continuous effect is derived, so an `UntilEndOfTurn` steal reverts at cleanup with no mutation to hang an update on and no event to hook. `EffectiveCharacteristics.control_since_turn` is computed beside the controller it describes, which is what makes reversion need nothing at all — the value stops being computed when the row leaves the registry. The battlefield field survives as the seed, owning every control change that is *not* a Layer 2 effect (entering the battlefield, today the only one).

      The Layer 2 arm advances it only when control actually moves, because CR 302.6 asks whether control was *continuous* and gaining control of your own creature is not a change. Act of Treason legally targets your own creature and would have hidden this behind its haste clause.

      **One honest gap, and it is unobservable.** Strictly, CR 302.6 makes a creature sick for its *original* controller the instant a steal expires, since control was interrupted during the turn. We report it as not sick. The only window between expiry and that player's next turn beginning is the cleanup step itself, where no player receives priority (CR 514.3) and no ability can be activated, so modelling it would mean recording an interruption nothing can read.

    - **Layer 2 reaches the stack.** `compute::base_controller` — now the single definition of the pre-Layer-2 seed, with three callers where there used to be three copies — has a `StackEntry` arm, which is CR 108.4's other half. `collect_controllable_targets` is the Layer-2-only sibling of `collect_battlefield_targets`: every other continuous effect describes a characteristic a permanent has, but control is the one thing a spell also has.

    - **Perf, interleaved, `fuzz_games --games 200 --seed 12345`.** `main` 79.7 → this branch 83.1 ms/game (9 rounds), of which ~1.8% is the migration measured with Act of Treason unregistered so the card pool matches, and the rest is the card being cast. **Under item 11's +7% ceiling.** The gate, however, is now worth **+28%** rather than 4% (83.7 → 107.2 with it forced off) because it went from protecting one call site to 21. The **sharper per-object gate item 11 proposed was built, measured and discarded**: 83.3 vs 82.6 ms/game over 5 rounds, inside the spread of either column and worse on the median, because `ObjectId` is a v4 UUID and the set probe costs a SipHash at every migrated call site on every board to save on the rare one. Do not rebuild it without a board that keeps the registry-wide flag true for a long time. Determinism holds: three runs at one seed byte-identical apart from wall-clock lines.

    - **Cross-call memoization deliberately did NOT land here** (`layers-architecture.md` §12 item 2, scheduled between this phase and 613.8). Three reasons: §12 requires the paranoid recompute-and-assert mode in the same commit, which is phase-sized on its own; +4% does not force it; and its invalidation key wants designing against item 8 step 4's board-wide sequential pass, which does not exist yet. Landing it before that pass risks building a cache for the wrong computation.

14. **The targeting-side `PermanentFilter` could not resolve a `PlayerRef` — ✅ done (2026-08-23).** There are two functions called `permanent_matches_filter`, and item 11 rewrote only one. `compute::permanent_matches_filter` asks whether a continuous effect applies to a permanent mid-layer-walk and reads an `EffectiveCharacteristics` frame; `targeting::permanent_matches_filter` asks whether a permanent is a legal *selection* and reads the finished board. The second still had `_ => Err("PlayerRef {:?} not supported")` for every variant but `Player(_)`, untouched since Phase LD.

    The consequence was silent and total: SBA 704.5n calls `validate_selection` on an Aura's host, got `Err`, and treated the Aura as validly attached. **Every "Enchant creature you control" Aura had an unenforceable restriction** — Ethereal Armor, Gryff's Boon, Angelic Destiny, the whole cycle.

    Fixed by threading `you: PlayerId` through `validate_targets`, `validate_selection`, `validate_permanent_target`, `permanent_matches_filter`, `has_any_legal_choice`, `is_single_target_legal`, `any_targets_still_legal` and `enumerate_legal_selections`. Every caller had the value already: the caster in `cast.rs`, the spell's controller in `stack.rs`, the Aura's controller in `resolve::try_attach_aura_on_etb` and SBA 704.5n, the enumerating player in `mana_helpers`.

    Two tests fail if either half is reverted — restoring the `Err` arm, or reading `BattlefieldEntity.controller` instead of the effective one.

15. **The corpus named a card that does not exist.** ATOM-613.1b-001's board said "Mind Snare". Scryfall 404s on both `cards/named?fuzzy=` and an exact-name search. The name was invented in `plans/archive/implementation-plan-final.md` §L17 ("{3}{U}{U} Instant, GainControl with WhileTargetOnBattlefield" — a re-costed Control Magic) and propagated into the corpus and into `roadmap.md` from there. Substituted Act of Treason, verbatim; the atom's claim is unchanged because its untap and haste clauses are inert for a P/T query. `plans/archive/*` is historical per CLAUDE.md and was left alone; `cards-unlocked-ledger.md` was corrected, and the two live `roadmap.md` sites (the Tier 1 card list and Milestone 4's criterion) followed on 2026-08-24 — they sat inside the slice the staleness banner tells readers to trust, which is the one place a fake card can still mislead.

    **The general lesson is worth more than the fix.** The corpus is authored from a close read of the CR, but its *boards* were written against a plan document rather than against Scryfall, so a card name in an atom is not evidence the card exists. Verify before building to one.

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

**Standing constraint until the CR 613.8 cluster lands (2026-08-24): author no
dependency-ordering-sensitive cards.** Ordering inside a layer is timestamp-only, and
item 8 lists the two known-wrong boards. Under the new v1 this is a real limit on the
ledger rather than a formality — Phase 8 is where a Commander-viable pool arrives, and
that pool is dense in interacting statics, which is why 613.8 is back-stopped to land
first.

1. **CR 208.3 — a noncreature permanent has no P/T.** "A noncreature permanent has no power or toughness, even if it's a card with a power and toughness printed on it (such as a Vehicle)." `get_effective_power`/`get_effective_toughness` return the printed numbers for an unanimated Vehicle. Pre-existing and unreachable today — no Vehicle is implemented — but visible now that those accessors are no longer gated on the battlefield (CDA phase, 2026-08-22). Fix belongs with the first Vehicle: gate on `chars.types.contains(Creature)` for battlefield objects only, since CR 208.3's *other* half deliberately keeps P/T on a card outside the battlefield.

2. **"Any player may activate this ability" is unmodeled (CR 602.1a).** `engine/cast.rs::activate_ability` rejects any activation by a player who does not control the permanent. That is CR 602.1a's *default* — "the controller of an activated ability is the player who activated it", and only that permanent's controller may do so — but the rule is overridable by the ability's own text, and **41 printed cards override it**: Aether Storm ("Pay 4 life: Destroy this enchantment... Any player may activate this ability"), Excavation, Feral Hydra, Deadly Designs, Fan Favorite, Endbringer's Revel, Casey Jones, and 34 more (Scryfall `o:"any player may activate"`, 2026-08-23).

   `AbilityDef` has nowhere to record the permission, so this is a missing field rather than a missing check: an `activatable_by` on `AbilityDef` (default: controller only), read by `cast.rs::activate_ability` and by `oracle::mana_helpers::activatable_abilities`, which currently enumerates only the asking player's permanents. Both halves are needed — a permission the action list never offers is invisible.

   Surfaced during the Layer 2 phase, whose migration rewrote the check but not its scope. The error message now names CR 602.1a and says the exception is unmodeled, rather than asserting the rule is universal.

3. **Named counters have no representation — `CounterType` is a closed enum.** CR 122.1 lets a counter be named anything, and "counters with the same name or description are interchangeable" makes the *name* the identity. Most named counters have no rules meaning at all: the card counts its own counters and nothing in the engine cares what they are called.

   **Breadth, measured 2026-08-23:** a ~1000-card Scryfall sample of `o:/counters? on/` yields **115 distinct counter-name words** — charge, time, oil, quest, age, storage, lore, doom, plan, flood, bounty, egg, energy, scream, page, delay, gold, fuse, mire, ice, verse, luck, ki, collection, spore, slumber, book, burden, filibuster, and on. One sample, not the whole set. A variant per name is not viable.

   **The split is the same one `KeywordFlag` uses.** CR 122 enumerates every counter the rules branch on, and it is a short closed list: 122.1a +X/+Y, 122.1b keyword, 122.1c shield, 122.1d stun, 122.1e loyalty, 122.1f poison, 122.1g defense, 122.1h finality, 122.1i rad, plus 122.3's +1/+1 ÷ -1/-1 annihilation. Those stay variants because engine code is keyed on them. Everything else is a name and a count.

   **`CounterType::Charge` is already on the wrong side of that line** — no CR entry, the single most common vanilla counter in Magic, a variant only because one card needed it. It moves with this work.

   **Shape:** `CounterType::Named(...)` carrying a `&'static str`. The constraint is that `CounterType` is `Copy` and a `HashMap` key with five by-value signatures, so `String` and `Arc<str>` are both out — they would ripple through all of them. `&'static str` is correct permanently: every card is a `CardDataBuilder` call compiled into the binary, there is no serde, no file I/O and no deserialization anywhere in `src/`, and Scryfall is a research tool for contributors, never a runtime dependency.

   The one open question is whether to wrap it in a `CounterName` newtype with `const` values per name. The argument for it is typo discipline, not future-proofing: with 100+ hand-authored names spread across `src/cards/*.rs`, `"charge"` misspelled once silently creates a second, unrelated counter kind that no test would catch. Decide when the first named counter lands.

   Nothing is lost to a catchall: CR 122.4 ("can't have more than N counters of a certain kind") and CR 122.7 ("when the Nth [kind] counter is put on") are both generic over *kind* and never need to know what a counter means.

   **Build it with the first card that needs a named counter, not before** — there is no consumer today, `CounterType::keyword_granted()` already returns `None` for anything unrecognized, and a representation with nothing to test against is what item 9 warns about.

4. **CR 613.7e re-timestamping on attachment is unimplemented — and it collides with the determinism doctrine (recorded 2026-08-24).** "An Aura, Equipment, or Fortification receives a new timestamp each time it becomes attached to an object or player." Nothing in the tree ever reassigns `BattlefieldEntity.timestamp`, and both CLAUDE.md and `battlefield_ordered`'s docs now state "allocated once per `place_on_battlefield`, never reassigned" as the *determinism* guarantee. `layers-architecture.md` §8 point 3 lists 613.7e as designed, so that doc currently claims more than the code does.

   Unreachable today — Equip is unimplemented and Auras attach only at ETB — but the day any reattachment path lands, every equip silently re-orders Layers 6 and 7. **Do these together:** reassign from the same monotonic counter (still deterministic — that is the point), restate the contract as "never reassigned *except by CR 613.7e*" in CLAUDE.md and in `battlefield_ordered`, and re-audit every site that reads `timestamp` as a proxy for ETB order. Caught by audit before it had a reproducer; the two before it were found by their reproducers.

5. **Layer 7c is not order-independent in Magic, and 19 cards say so (recorded 2026-08-24).** `compute.rs` applies ±1/±1 counters after the 7c registry slice without a timestamp merge. That is correct while every 7c modification is an addition — but CR 701.10a makes "double [a creature's] power" a 7c continuous effect whose addend depends on what already applied, so two doublings, or a doubling and a pump, are order-dependent by timestamp. Scryfall: **19 cards** match the doubling shape (Bulk Up, Epic Fight, Exponential Growth, Unnatural Growth…), before looser wordings. Inexpressible today — `AmountExpr` has no affected-power leaf — so nothing is wrong now. The first doubling card needs that leaf **and** the timestamp merge Layer 6's keyword counters already use. The comment at the code site was corrected 2026-08-24; it used to claim order-independence as a property of the layer.

6. **Multi-attacker block damage is a silent stub.** `engine/combat/resolution.rs:146`: a blocker blocking 2+ attackers assigns *all* its damage to the first living attacker — no `DecisionProvider` choice, no error, CR 510.1c ignored. Unreachable (nothing in the pool grants "can block an additional creature"), and the plumbing to fix it already exists: `GameState.blocker_damage_divisions` is populated from `choose_blocker_damage_division`. Reachable with the first "blocks an additional creature" card. This is the silent-wrong-*choice* cousin of the silent-inertness class the loud-lowering work covered.

7. **Hexproof and shroud are unenforced in spell targeting.** `engine/targeting.rs:302`, `TODO` tagged T22 (with matching notes at :39 and :293). `KeywordFlag::Hexproof` exists and combat honors it; spell targeting does not check either keyword. No registered card carries hexproof or shroud, so no game can reach it — reachable with the first such card, which is a Phase 8 event.

### Before Triggered abilities (CR 603)

The trigger dispatcher's designated insertion point is `engine/priority.rs:234-240`. Today's gaps:

1. **Trigger dispatcher stub.** `let triggers_placed = false; // Phase 7 stub` at `engine/priority.rs:235`. This is the single-point insertion — **for placement only** (2026-08-24): detection runs synchronously at event dispatch per the resolved Replacement item 4; this stub is where the pending queue drains onto the stack in APNAP order (CR 603.3b, over the full player set).

2. **Event shape audit.** Every `events.emit(...)` call site is a potential trigger source. Before wiring triggers, audit that:

   **Known missing already (found 2026-08-24, registering the first activated ability):** `GameEvent` has no variant for an activated ability being put on the stack or resolving. `cast.rs::activate_ability` pushes the ability object onto the stack without `move_object`, so not even a `ZoneChange` is emitted, and the resolution emits nothing either — an activation is completely invisible in the event log. `AbilityCountered` exists, which is the whole of the vocabulary. Triggers that watch activations ("Whenever a player activates an ability…") have nothing to watch, and the event log cannot be used to audit activation behavior at all — measuring how often Merfolk Thaumaturgist's ability resolved needed a temporary probe in `resolve.rs`. Fix as part of the event-stream refit (Replacement item 3): the fork was resolved 2026-08-24 — `AbilityActivated` plus an identity-bearing `AbilityResolved` (source + ability, for CR 603.7h counting), emitted from the chokepoint.

   - Events are emitted at the correct granularity (e.g., `PermanentEnteredBattlefield` fires per-permanent, not per-batch).
   - Event timing is post-action, not pre-action, so triggers observe the completed state change.
   - Events carry enough context for trigger predicates (controller, source, type filters).

3. **LKI formalization.** Several dies-handling sites already read `self.objects.get(&id)` *before* `move_object` to capture pre-move state (see `engine/sba.rs` dies handlers). This is ad-hoc LKI. Triggered abilities that reference "the creature that died" need a formalized `LastKnownInformation` snapshot mechanism, especially after layers land (LKI needs *post-layer* characteristics at moment-of-death, per rule 603.10 / 608.2h).

### Before Commander (CR 903)

1. **Commander damage increment — ✅ done (2026-04-18).** `GameObject.is_commander: bool` added; `execute_action(DealDamage)` accumulates `commander_damage_taken[source]` when `is_combat && target == Player && source.is_commander`. 5 unit tests. The loss SBA (`engine/sba.rs:73`) now has a live writer.

2. **Commander setup hook.** Nothing yet flips `is_commander = true` at deck construction or game setup. Needs a `GameConfig::commander()` constructor + a designation step (probably a field on `Decklist` or an analogous role entry). No tests exercise this yet — the flag is only set via direct field mutation in unit tests.

3. **`GameConfig::commander()` constructor.** Only the *hand/library* half of command-zone redirection waits on Replacement (903.9b). The graveyard/exile half is CR 704.6d, a state-based action, and can land with this constructor — so a "partial Commander" game here is life=40 + commander damage + 903.9a working, with only 903.9b missing.

4. **Multiplayer priority rotation** (CR 800) — blocking for 3+ player Commander; not blocking for 2-player Commander.

### Cross-cutting — keep this section honest

**~~Summoning sickness ended one turn early (CR 302.6).~~ — ✅ fixed 2026-08-24.** `has_summoning_sickness` compared `control_since_turn >= game.turn_number`, which asks "was control gained during the turn now being played" — the same answer as the CR only on the controller's own turn. A creature you cast on your turn went unsick as soon as the turn passed, one full turn early, and at four players three turns early. Reachable in the current pool: Citanul Hierophants grants "{T}: Add {G}", and instant-speed mana activation on an opponent's turn is an ordinary play.

The comparison now runs against `GameState.last_turn_began[controller]` — a per-player record of when that player's most recent turn began, written only by the new `GameState::begin_turn`, because it cannot be derived from `turn_number` at more than two players or once extra turns exist. Two boundaries are carried by the "no turn yet" arm: the CR 103.6 pregame sentinel (`control_since_turn = 0`) stays unsick, and a permanent that arrives before its controller's first turn is sick until that turn begins. Five tests in `tests/phase_lg_integration_test.rs`, three of which fail against the pre-fix tree; `tests/determinism_test.rs` and the three-runs-one-seed fuzz check are unaffected.

The stale doc comment was the audit's fourth overclaim of the sprint: it justified the old comparison as "the same answer as the CR's wording whenever turns alternate", and the divergent window *is* the alternating case.

**Follow-up, 2026-08-24: `has_summoning_sickness` answered the wrong question for noncreatures.** It reported `true` for any permanent whose controller gained it this turn, creature or not, and every caller happened to gate on `is_creature` first — so nothing was wrong, but the predicate's name asked a question CR 302.6 only poses about creatures, and the next caller to forget the gate would have been told a fresh Sol Ring cannot tap for mana. The type check now lives inside, read off the frame the function already computes so the two halves cannot disagree; the four `engine/costs.rs` sites and `ui/display.rs` dropped their own `is_creature` call, which was a second full layer walk each. `oracle::legality::can_attack` gained the check instead of losing it — it had been relying on sickness to keep noncreatures out of the attacker list (CR 508.1a), and would otherwise have started reporting that an untapped mana rock can attack.

Surfaced by writing the March-of-the-Machines animation tests, whose interaction is the interesting one: a Sol Ring taps for mana the turn it lands, and stops the moment March animates it, because CR 302.6 asks how long its *controller* has had it and not how long it has been a creature. Both Scryfall rulings say so verbatim and are quoted in `tests/phase_ld_integration_test.rs`. Haste (CR 702.10c) buys the ability back.

**~~The corpus had two parsers over two tiers.~~ — fixed 2026-08-24.** `extract-phase-index.py` generated the markdown indexes from `summaries/` (frozen 2026-08-19) while `specdb.py` built the database from `sessions/` (the authored tier, corrected twice since). The channel was not merely stale: the two disagreed by 27 entries and on the size of three phases — the Phase 7 index claimed 202 entries against the database's 133 — so a session grounding its scope in an index was reading a different corpus than one querying the database.

`specdb.py build` now writes `global-test-index.md` and `phase-index-*.md` from the same parse that builds the sqlite, `normalize_phase` moved in-house, and the extraction script is **deleted** — it lived at `plans/atomic-tests/extract-phase-index.py` and is recoverable with `git log --diff-filter=D -- plans/atomic-tests/extract-phase-index.py`. It was briefly moved to `plans/archive/` instead, which was wrong: that directory holds superseded *documents*, which are read, and this was a *script*, which can be run — and running it regenerates the indexes from `summaries/` and puts the drift straight back. A comment saying "do not run" is not a guard. `summaries/` carries a README saying it generates nothing.

Two bugs fell out of the same parse, both silent:

- **Entry ids swallowed their titles.** Sessions write a COMP heading as `COMP-702-001: Deathtouch + First Strike`, and 21 entries were stored with the title inside the id — which is why `specdb show COMP-7A-005` answered "no such atom", and why the ids never lined up with the indexes. Ids are bare now; the title becomes the summary when there is no `Rule` line to derive one from.
- **The `COVERS` scanner looked 8 lines ahead for the test name.** Annotations here carry their reasoning — the block above `test_tarmogoyf_pt_is_layer_7a_and_an_ability_strip_removes_it` runs twelve lines — so two atoms were recorded as covered by a test with no name. It now walks past comments and attributes to the `fn`, however far that is.

**`specdb suspicious` — the check `orphans` could not do (2026-08-24).** `orphans` catches a `COVERS` id that does not exist. Nothing caught one that exists and is *wrong*, which is the failure this project actually fears: a blank reads as work remaining, a false link reads as done. The new command compares the vocabulary of the atom's board/action/expected against the annotated test's source and flags links that share almost nothing. It is a smell detector — a hit means read it, and silence is not a proof.

It found a real over-claim on its first run over the existing 76 links. `ATOM-613.4d-004` ("modifier added *after* a switch applies to the unswitched side, then re-switches") was claimed as a full `COVERS` by `test_layer_ordering_7b_7c_7d_cast_in_layer_order` — the one arrangement that cannot demonstrate it, because casting in layer order means nothing is ever added after the switch. The atom's mechanism is exercised by the *reverse-order* test next to it, and only partially, since the atom's board is a 1/3 under two modifiers and the test is a 2/2 under one. Moved and downgraded; full coverage went 35 → 34, which is the number becoming true rather than getting worse.

**Spec-database annotation backfill — owed before the next phase.** `specdb stats` reads 0% on five finished phases because only the Phase 5-Layers tests were ever annotated (69 `COVERS` lines in the whole suite); it is measuring the annotation boundary, not coverage. A 2026-08-24 sample of ten Phase 5-Pre / ALREADY-IMPL atoms found no case where a 0% phase concealed a gap the chapter map claims is done — every real gap in the sample was already an honest ❌/🟡 here. The backfill is mechanical, roughly a day, mostly `// COVERS:` lines over existing sba/mana/cast/zone tests, and `plans/atomic-tests/phase-5-pre-audit.md` already reconciled the shipped 5-Pre tickets against the code with file:line citations — consume it rather than redo it. Use `COVERS-PARTIAL` honestly; a false link is worse than a blank. Reason to spend the day: Phase 6 has 124 atoms and Phase 7 has 133, and a tool that reads 0% on finished work has no credibility left for the phases that need it.

**`fuzz_games::random_deck` land population — ✅ partly fixed (2026-08-23); artifacts still missing.**

Land slots used to be filled entirely from a colour→basic table, so no deck could contain a nonbasic land. Blood Moon was in the card pool and inert, and CR 305.7 — the land-type carve-out in `engine/layers/land_types.rs`, the most intricate code in Layer 4 — had **zero** random-play coverage.

Fixed by registering the ten original dual lands (`cards/dual_lands.rs`) and giving `random_deck` a `NONBASIC_LANDS_PER_DECK` constant, currently 5. A land qualifies if it produces **at least one** of the deck's colours — deliberately not a subset test, because under a subset rule every dual is off by one colour for a two-colour deck and a mono-colour deck gets none at all. An unusable second colour on a land costs nothing in a fuzz deck.

Still crude, and knowingly so: a flat constant over a static pool is not a mana-base model. Replace it with a real picker when card breadth (Phase 8) gives it something to choose between.

**~~Still open — no artifact exists anywhere in `CardRegistry`~~ — ✅ fixed 2026-08-24, together with the Layer 7d hole.** March of the Machines was registered and inert for the same reason Blood Moon had been: nothing in the crate outside the `phase_l*` fixtures was an artifact, so Layer 7b had zero random-play coverage. Layer 7d had none either — no registered card switched P/T, and the one fixture that does (`phase5_pre_cards::inside_out`) simplifies a hybrid cost the engine cannot express, so it cannot be registered without misrepresenting the card.

Two real cards, authored verbatim: **Sol Ring** (`cards/artifacts.rs`) and **Merfolk Thaumaturgist** (`cards/utility_creatures.rs`). Sol Ring is colorless, so `random_deck` puts it in every deck rather than only the ones sharing its colors, and its mana value of 1 means it animates into a 1/1 that survives its own SBA check rather than a 0/0 that dies. Measured over 60 games at seed 7: a Sol Ring reached the battlefield in 53, a March in 23, **both in the same game in 20**, and Layer 7d resolved **98 times** (temporary probe, reverted). Both were zero before.

The Thaumaturgist is also the registry's **first `AbilityType::Activated` ability** — every other registered ability is a spell, a mana ability or a static — so `cast.rs::activate_ability`'s stack path, its target selection and its rollback arms now get random-play exposure too.

**Perf: the code costs nothing, the pool costs ~9%.** Four binaries built side by side and run **interleaved**, 200 games / seed 12345, median of five, `--release`:

| Binary | ms/game | ms/turn | vs main | turns/game |
|---|---|---|---|---|
| `main` | 79.56 | 2.6257 | — | 30.3 |
| this branch, **both cards unregistered** | 79.26 | 2.6158 | **−0.4%** | 30.3 |
| this branch as shipped | 86.64 | 2.8880 | +10.0% | 30.0 |

The second row is the only equal-work comparison in the table, and it is exact: with the two `registry.register` lines removed the binary plays games **byte-identical** to `main`, so −0.4% is the whole cost of the summoning-sickness type gate, the two dropped `is_creature` calls in `costs.rs`/`display.rs`, and the one added in `can_attack`. Inside noise, and it nets slightly favorable because a tap-cost check now does one layer walk where it did two.

Every other row plays *different games* — a new card changes deck composition and therefore every random choice downstream — so those percentages measure the pool, not shared code. Attribution, second interleaved batch against its own baseline (medians of five; note the baseline itself moved 2.6257 → 2.5297 ms/turn between batches, which is exactly the session drift that makes interleaving mandatory):

| Pool | ms/game | ms/turn | vs main |
|---|---|---|---|
| `main` | 76.65 | 2.5297 | — |
| + Sol Ring only | 74.16 | 2.6391 | +4.3% |
| + Merfolk Thaumaturgist only | 80.39 | 2.6977 | +6.6% |
| + both | 83.13 | 2.7710 | +9.5% |

Sol Ring is the interesting row: it makes each turn 4.3% more expensive and each *game* 3% cheaper, because acceleration ends games sooner (28.1 turns vs 30.3). ms/turn is the honest statistic for exactly this reason.

**Accepted.** The extra work is layers 7b and 7d actually running, which is what the cards were registered to cause — March of the Machines was free before because it applied to nothing. If fuzz throughput ever binds on this, the knob is deck composition (Sol Ring is colorless, so it is in *every* deck) rather than the engine, and the thread-parallel harness on the audit's list would dwarf it either way.

**Also recorded, from `cards/dual_lands.rs`:** CR 305.6 makes a land's mana abilities *intrinsic to its basic land types* — the parenthesised reminder text on a printed dual is not rules text. We model them as two explicit printed `AbilityType::Mana` abilities, because base characteristics are read straight from `CardData` and nothing derives abilities from printed subtypes. The two models agree everywhere currently observable (305.7 clears `chars.abilities` wholesale before granting the new intrinsic; Humility removes mana abilities from an animated land either way). Revisit if an effect ever needs to tell an intrinsic ability from a printed one.

**~~`fuzz_games --seed N` does not reproduce a run, and the perf protocol assumed it did.~~ — ✅ fixed 2026-08-23 (`fuzz/deterministic-seeding`).** `--seed N` now replays a run exactly: three consecutive 200-game runs at seed 12345 produce byte-identical output, and game *k* of a batch reproduces standalone from the per-game seed the harness prints, which is what makes "reproduce the panic from the seed" work.

The original entry named `HashMap` iteration order as the cause. That was real but second: **the seed never reached the AI or the shuffle at all.** `ui/random.rs` called `rand::rng()` — an OS-seeded `ThreadRng` — fresh in each of the four `DecisionProvider` methods, and `state/game.rs::shuffle_library` did the same, so `--seed N` controlled deck *composition* and nothing else. `cards/registry.rs::card_names()` was a third: it returned `HashMap` keys, so the seeded deck builder drew from a differently-ordered list each process and built a different deck from the same seed. Three separate leaks, each sufficient on its own; the recorded measurement (25.9 / 27.6 / 28.0 avg turns) was the sum.

What landed:

- **Randomness is owned, not ambient.** `GameState.rng: StdRng` (seeded to `DEFAULT_RNG_SEED` — a fixed value, so an unseeded game is still reproducible) with `reseed` / `reseed_from_entropy`, and `GameState::shuffle_library` as the one shuffle entry point. `RandomDecisionProvider` holds its own `StdRng`; `::new()` is entropy-seeded, `::seeded(u64)` is not. `fuzz_games` derives three independent streams per game from `master_seed + game_num`; `cli_play` reseeds from entropy, since an identical opening hand every session would be the bug.
- **`GameState::battlefield_ordered` / `battlefield_ids_ordered`** — every sweep whose order is observable now goes through them: `oracle/{legality,mana_helpers,board}.rs`, all of `engine/sba.rs` (including the legend-rule grouping, now a `BTreeMap`), `engine/combat/{steps,resolution}.rs`, `ui/display.rs`. Sorting by `ObjectId` would *not* have worked — ids are v4 UUIDs, so the key is itself random per run. The deterministic key is `BattlefieldEntity::timestamp`, which `place_on_battlefield` allocates once from a monotonic counter and never reassigns, and which is CR 613.7's order anyway. Order-irrelevant sweeps (untap-all, clear-all-damage) still iterate the map directly.
- `tests/determinism_test.rs` holds the regression. Both halves were shown failing against the pre-fix tree.

**Cost: none measurable.** 200 games / seed 12345, median of five, ms/turn: 1.124 → 1.131 (+0.7%), inside run-to-run noise — the per-sweep `Vec` + sort is nothing next to the `compute_characteristics` walk it wraps. A `BTreeMap` swap was considered and not needed. Wall-clock spread over those five runs collapsed from 5.64–6.43 s to 6.20–6.29 s, because the runs now do identical work.

**`fuzz_games` runs its games on a worker pool — ✅ 2026-08-24.** Games were already independent (every input is a pure function of `master_seed + game_num`), so this was a worker pool and nothing else: a shared atomic index, per-worker result vectors, and a sort back into game order before anything is printed or aggregated. `GameState`, `Game`, `RandomDecisionProvider` and `CardRegistry` were already `Send`, so no type changed. **Output is byte-identical at any `--threads` value** — that is the acceptance test, and `--threads 1` is kept as the serial reference. The formatted event-log snapshot is now built only when `--dump-events` asks for it; the serial harness formatted one per game and dropped it.

**Which mode to use, and why the obvious answer is wrong.** 200 games / seed 12345, ten runs each:

| Mode | wall/run | speedup | run-to-run CV |
|---|---|---|---|
| `--threads 1` | 17.8s | 1.0× | **2.4%** |
| `--threads 8` | 3.5s | 5.0× | **6.1%** |
| `--threads 16` | 2.7s | 6.7× | 4.8% (n=5) |

- **Coverage — hunting panics and errors — wants threads.** 6.7×, and a pass/fail sweep has no precision requirement. This is where the harness's wall-clock time actually goes.
- **Benchmarking wants `--threads 1`.** Threading inflates the CV from 2.4% to 6.1%, and matching a serial median-of-five's standard error would take ~32 threaded runs — *more* wall time than the five serial runs, not less. Contention noise is not a fixed offset that cancels in an A/B. (An n=5 sample said 8 threads was *tighter* than serial; ten runs said otherwise. Five samples is enough for a median, not for a variance.)

**Do not cut the game count to save time.** Measured the same way: N=200 has a 2.1% run-to-run spread, N=100 has 4.3% and N=50 has 4.2% — halving the batch doubles the noise and buys 9 seconds, and the ±3% band stops being achievable at all. Absolute ms/turn is also not comparable across N (2.41 / 2.47 / 2.68 at 50 / 100 / 200), because a longer batch contains more of the long games whose boards are expensive. If a benchmark is taking too long, the lever is the *matrix* — fewer variants and rounds, and only benchmarking changes that touch the layer walk or a per-permanent sweep — not N.

**A data point for the parallel-AI use case — and a correction.** The first version of this note said the engine "is contending for memory bandwidth, not cores." That was an inference from two data points, and a finer sweep does not support stating it as fact. Measured on a Ryzen 7 7700X (**8 physical cores, 16 logical**; ~15% background load from other applications at the time), 100 games / seed 12345, median of three:

| workers | wall | speedup | efficiency | CPU/game | inflation |
|---|---|---|---|---|---|
| 1 | 7.47s | 1.00× | 100% | 74.5ms | 1.00× |
| 2 | 4.15s | 1.80× | 90% | 82.4ms | 1.11× |
| 4 | 2.16s | 3.46× | 86% | 85.2ms | 1.14× |
| 6 | 1.67s | 4.47× | 75% | 94.7ms | 1.27× |
| 8 | 1.42s | 5.26× | 66% | 102.7ms | 1.38× |
| 12 | 1.24s | 6.02× | 50% | 123.9ms | 1.66× |
| 16 | 1.12s | 6.67× | 42% | 146.6ms | 1.97× |

What the *shape* says, as opposed to what one endpoint suggested:

- **Per-game cost inflates from two workers onward** (1.11× at 2, where core contention cannot be the explanation) and climbs smoothly. That is a shared-resource signature, not a core-count one.
- **It steepens past 8**, which is where logical processors stop being physical ones. "16 cores" is 8 cores plus SMT, and SMT siblings share execution units.
- **Four candidate causes are not separated here:** all-core boost-clock reduction, shared L3 / memory bandwidth, allocator contention (the layer walk allocates a `HashSet`/`Vec` per object per call, and this is Windows' system allocator), and the background load of whatever else the machine is running. The cheap discriminator for the allocator is a `mimalloc`/`jemalloc` swap and a re-run of this table; it would cost the project its third dependency, so it is a decision rather than a task.

**What to design against:** ~6.7× is the ceiling on this machine, **8 workers buys 79% of it**, and past the physical core count each doubling of workers returns ~25%. Default to physical cores or a little under, not logical, and treat worker count as a tuning knob rather than a constant. Determinism is unaffected by any of it — outcomes are identical at every worker count, verified.

**The perf protocol is trustworthy again.** "200 games / seed 12345, back to back, ±3% band" now compares equal work, so avg-turns is a *check* rather than a variable: if two runs at the same seed report different turn counts, something reintroduced process state into a decision, and the perf reading is meaningless until it is found. Median-of-five ms/turn remains the better statistic, but for machine noise now, not for divergence.

- Every new forward-looking stub, TODO, or half-wired abstraction gets a line here at commit time.
- When a migration is completed, strike the line (keep it visible in history for a few revisions, then remove).
- Migrations that are substantial enough to warrant ticketing get a link from here to their ticket; tiny migrations are just done inline.
