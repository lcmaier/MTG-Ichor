# Codebase State — CR Coverage Map

Ground-truth snapshot of CR coverage. Single source of truth — if another planning doc contradicts this, this wins. Last grounded-in-code audit: 2026-08-25.

---

## TL;DR

- **v1 is two use cases** (owner, 2026-08-24): peer-to-peer human games through a GUI, specifically **4-player Commander**, and **highly parallel AI games** over the CLI. Two-player Standard is a checkpoint, not the target. Ordering lives in `CLAUDE.md` → "Critical path to v1"; the consequence for this file is that CR 800/802 and CR 903 below are path items, not deferrals, and that new systems get written N-player-shaped.
- **Code size:** ~34,100 lines of Rust across 89 `.rs` source files (~45,200 with the integration tests). 888 tests, 0 warnings, fuzz harness runs 200-game batches and fails a run in which any spell resolves unpaid (16c).
- **Well-covered:** CR 1 (game basics), CR 3 (card types), CR 4 (zones), CR 5 (turn structure), CR 7 (keyword abilities + SBAs).
- **Partially covered:** CR 6 (casting: pipeline skeleton + X/alt/additional-cost landed, mode choice + distribution + activation restrictions pending). CR 1 mulligan is a stub. Equip and Bestow (CR 702.6, 702.103) not started.
- **Not started:** **triggered abilities (CR 603)** beyond an enum variant, though RA built the record they will match against; CR 800 multiplayer priority/turn rotation.
- **Replacement effects (CR 614–616) — the pipeline is live (Phases RA–RB, 2026-08-25 → 2026-08-26).** RA made every observable mutation a proposal; RB put CR 616.1's loop between the proposal and the mutation, with counters (CR 122.1c/d/h), regeneration (CR 701.19) and Kalitas as its three consumers, and Commander's CR 704.6d / 903.9b pair alongside. **RC is under way (2026-09-02): RC-1 deleted the early stack pop; RC-2 made entering the battlefield a proposed event (`GameAction::EnterBattlefield`, `Rewrite::EnterWith`, `EnterMods`), with `place_on_battlefield` as its performer and CR 110.5b / 122.6a as its two consumers; RC-3 settled CR 614.12's membership rule in both directions — a filter-scoped layer effect now reaches an entering permanent, and an entering permanent's own filter-scoped replacement no longer reaches itself; RC-4 built the frame those effects are evaluated against (`layers::compute_as_entering`, a read-side overlay through `FrameCache`'s accessor pair — never a `GameState` clone), put CR 614.17d and CR 616.1b on it, and stopped CR 616.1 prompting for a choice with one outcome.** **RC-4b (2026-09-02) made entering one event: `GameAction::EnterBattlefield` carries `from` and its performer moves the card, so no `ZoneChange` onto the battlefield is ever proposed, a substituted entry is one move with no LKI walk, a refused entry leaves the card where it was (CR 608.3e for a resolved spell), and `cast_spell`'s 601.2a move is silent in both directions until 601.2i.** Still ahead: **RC-5** (CR 614.13's auxiliary zone changes and the batch-scoped frame), RD (damage and prevention, CR 615), RE (the remaining event kinds).
- **"Can't" effects (CR 101.2/614.17/613.11) — the spine is live (Phases RS-0, RS-1, 2026-08-31).** `plans/cant-effects-architecture.md` is authoritative for phases RS-0–RS-4 and §7.1 carries the interleaved Track R / Track S order. **RS-0**: `state/duration_registry.rs` is the `DurationRegistry<T>` both effect registries own and delegate to, so the CR 514.2 expiry rules exist once instead of twice. **RS-1**: `RestrictionDef` / `Restriction` (`types/restriction.rs`), the third `DurationRegistry` customer (`state/restrictions.rs`), and `engine::restriction::is_prohibited` — one predicate, a battlefield sweep off *effective* ability lists, and §4.9's candidate filter. Indestructible, "can't be regenerated" and Sigarda all reach it. **The hard join is satisfied: RC-4 is unblocked.** Still ahead on this track: RS-2 (casting/activating/targeting), RS-3a/b (combat), RS-4 (costs).
- **Layers (CR 613) — core landed, three layers live (Phases LA–LD, 2026-05 → 2026-08).** The system is real, not scaffolding: `Layer` enum with all 9 sublayer variants (`engine/layers/types.rs`), `EffectiveCharacteristics` struct (name, mana_cost, colors, types, subtypes, supertypes, keyword_flags, abilities, P/T, controller), a `ContinuousEffect` registry whose row storage and duration-based expiry live in the shared `state/duration_registry.rs` it owns (RS-0, 2026-08-31), and `compute_characteristics` (`engine/layers/compute.rs`, 967 lines). Static abilities register through `GameState::register_static_effects`. `oracle/characteristics.rs` wrappers all route through `compute_characteristics`.
  - **Live layers:** 2 (control) — Layer 2 phase, 2026-08-23. 4 (types/subtypes/supertypes) — Phase LD Part A. 5 (color) — Phase LC. 6 (abilities) — Phase LF. 7a (CDA P/T) — Phase LE. 7b (set P/T), 7c (modify P/T), 7d (switch P/T) — Phase LB.
  - **Still stubbed:** Layer 3 (text) is an enum variant only. Layer 1 has a producer as of CV-1 (2026-09-02) — `EffectModification::CopyFrom`, from `Primitive::Copy`; its face-down sublayer (CR 613.2b) waits on CV-6.
  - **Dependency algorithm (CR 613.8) not implemented.** Ordering is timestamp-only, which is sufficient for the layers landed so far in isolation but will not survive Layer 6 + Layer 4 interaction (Humility/Opalescence).
  - **CR 305.7 / 305.6 — ✅ done (Phase LD Part B).** Blood Moon strips a nonbasic land's printed abilities and grants the intrinsic `{T}: Add {R}`; Urborg adds a basic land type and its mana ability without stripping. Lives in `engine/layers/land_types.rs`. `AbilityOrigin` was evaluated at Part B kickoff and **not built** — layer ordering makes it unnecessary; see `layers-architecture.md` §15.2 item 4.
- **Commander (CR 903) — in scope, skeleton only:** command zone ✅ as a `Zone` variant + `GameState.command` field; commander damage loss SBA ✅; commander damage **increment on combat damage now wired** (2026-04-18) via `GameObject.is_commander` flag + per-source accumulation in `execute_action(DealDamage)`. **903.9a (CR 704.6d) and 903.9b both landed with Phase RB, 2026-08-26.** Still missing: commander tax, `GameConfig::commander()`, and a commander designation/setup hook — nothing outside tests sets `is_commander = true`, so neither 903.9 half is reachable in a real game yet.
- **Biggest single block of work remaining before the engine can run real Magic:** the CR 613.8 dependency algorithm + triggered abilities + replacement effects. These are tangled — CR 613.1c says abilities themselves can be layer-modified, replacement effects depend on effective characteristics, triggers often fire on events that must be observed post-replacement. **Commander's zone-redirection dependency is discharged** — both halves of 903.9 shipped with Phase RB — so what Commander still needs is cost modification (tax) and multiplayer (800 priority).
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
| **614–616** | **Replacement + prevention + interaction** | 🟡 **The pipeline is live. Phases RA (2026-08-25) and RB (2026-08-26) complete.** RA made every observable mutation a `GameAction` proposal carrying `ZoneChangeCause`, the CR 603.10a LKI frame, a `BatchId` and its resolution. RB put `apply_replacements` between proposal and mutation: CR 616.1a–g, 614.4/5/6/17, 616.2, CR 615.5 riders, CR 101.4 APNAP. Consumers: CR 122.1c/d/h counters, CR 701.19 regeneration, Kalitas, CR 903.9b. **Not yet:** CR 614.15 self-replacement (bucket, no producer), CR 614.10/11/16 (RE), CR 614.12/13 ETB (RC), **all of CR 615's prevention detail (RD)** — RB has `Rewrite::Prevent` and CR 122.1c's prevention half, not shields or amounts. See `plans/replacement-architecture.md` §9. |

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
| 903.7 | **Commander designation + command zone start** | 🟡 `GameObject.is_commander: bool` flag exists (2026-04-18); no deck-construction / setup hook yet flips it, and no "commander starts in command zone" routing. **This is now the gate on 903.9** — both halves of the zone redirection work and neither is reachable in a real game, because nothing outside tests sets the flag |
| 903.8 | **Commander tax (+{2} per prior cast from command zone)** | ❌ no cast counter, no cost modification |
| 903.9a | **Commander in graveyard or exile → command zone** | ✅ 2026-08-26 (Phase RB item 8). A *state-based action* (**CR 704.6d**), not a replacement effect — the 2026-08-24 correction, now in code. "Since the last time state-based actions were checked" is answered by `GameObject.zone_change_epoch`, with the window read at the **top** of the check: a commander CR 704.5g kills moves *during* a check, so an end-of-check boundary would never offer it | `engine/sba.rs` |
| 903.9b | **Commander to hand or library → command zone instead** | ✅ 2026-08-26 (Phase RB item 9). Synthesized per event as a *game rule*, not card text, and the rules' only `exempt_from_614_5` effect. Its chooser needs no special case: CR 616.1's "or its owner if it has no controller" already answers for a card in a graveyard, library or hand. **It is also what exposed the hang in §4.1's loop** — exempt *and* optional means a decline is re-offered forever unless declines are tracked separately | `engine/replacement/gather.rs` |
| 903.10a / 704.6c | **Commander damage loss (≥21 combat damage from one commander)** | ✅ SBA (T16); `commander_damage_taken: HashMap<ObjectId, u32>` on `PlayerState` (T02). The loss itself is the state-based action at **CR 704.6c**; 903.10a is the Commander-variant rule it implements | `state/player.rs`, `engine/sba.rs` |
| 903.10a | Attacking with commander + accumulating commander damage | ✅ (2026-04-18) `GameObject.is_commander` flag + `execute_action(DealDamage)` accumulates per-source `commander_damage_taken` when `is_combat && target == Player && source.is_commander`. 5 unit tests cover basic accumulation, 21-damage threshold, non-combat exclusion, non-commander exclusion, and per-source isolation. Still requires a Commander-format setup hook to actually flip the flag at deck construction — no gameplay wiring yet sets `is_commander = true` outside tests. |
| 903.11 | Cards from outside the game (wishes) | ❌ Phase 9 — `atomic-tests/sessions/session-10.md` defers it, with the Companion cross-ref |
| 702.124 | Partner, incl. partner with, choose a Background, Doctor's companion | ❌ (`EnchantmentType::Background` exists as a data-type, no mechanics). CR 702.124a collects all five partner abilities under one keyword — they were 903.12/903.13 in an older CR |

**Other variants in CR 9 (Brawl, Planechase, Archenemy, Vanguard, etc.) — ❌ not started.**

---

## Deferred Migrations

**Purpose:** track technical debt incurred by forward-looking scaffolding. Each entry is a migration owed to a future system before that system can safely land. These don't show up as test failures today because the dependent system doesn't exist yet — which is exactly why they're easy to forget. Any time a new forward-looking stub is added, it should be recorded here.

**How to use this section:** before opening the first ticket of a listed target system, re-read that system's subsection and treat the items as prerequisites to schedule before or alongside the system's core work.

### Before Replacement effects (CR 614–616)

**The phase has an architecture doc as of 2026-08-24: `plans/replacement-architecture.md`.** It is authoritative for the type shapes, the CR 616.1 pipeline, the ETB look-ahead frame, and the RA–RE sequencing; item 3 below *is* its Phase RA. This section stays the status ledger.

**RA ships as three PRs (sized 2026-08-25, `replacement-architecture.md` §9).** RA-1 = the `ActionContext` sweep + `ZoneChangeCause`; RA-2 = the six routing tickets; RA-3 = batch form, LKI/cause/batch-id payloads, the three bypass closures, and the death-event demotion. Ticket numbers below are stable and cited by §9.

**Status 2026-09-02: Phase CV-1 ✅ — the copy spine (CR 707, layer 1a).**
`copy-effects-architecture.md` §7a carries the five findings and the
measurement; this is the state ledger. What landed:

- **`layers::copy`** — `CopiableValues`, and `copiable_values(game, id)` as the
  engine's one capture point. CR 613.2c makes copiable values the *output* of
  layer 1, so the capture is `compute_to_ceiling` at `END_OF_LAYER_1`: an
  existing entry point at a ceiling the existing array already indexes, with a
  `debug_assert` pinning the constant to `LAYER_ORDER` so CV-6's sublayer split
  is a test failure rather than a silent read one sublayer early. Deliberately
  **not** `EffectiveCharacteristics` reused — that type carries `controller`,
  and `compute.rs`'s `any_control_changing` fast path proves itself on the claim
  that Layer 2 is the only channel that writes one.
- **`EffectModification::CopyFrom(Box<CopiableValues>)`**, applied at
  `LAYER_ORDER[0]`, replacing every characteristic channel at once. Layer 1 had
  been a slot the walk always visited and nothing ever occupied.
- **`Primitive::Copy(CopyRoles, Duration)`.** `CopyRoles` has two arms because
  the two cards bind the atom's target to opposite roles: Cytoshape targets what
  *becomes* a copy and chooses its donor; Mirrorweave targets the donor. The
  `{ from, to }` product was rejected — two of its four combinations are
  unreachable states.
- **`ChoiceKind::ChooseCopySource`**, its own decision site rather than
  `SelectRecipients`, which `sacrifice_of_choice` reuses. There the chosen object
  is what the primitive acts on; here it is the opposite, and a DP heuristic
  keyed on the recipient kind would read the donor as the victim. Item 40 holds
  by construction: one prompt, and the chosen id is a local that never spans a
  second decision. Asked only with two or more candidates (CR 102.2's shape), so
  no existing scripted test gained a prompt.
- **Both legs of item 16**, and there were three ETB scans rather than the two
  that item recorded — see it, above.
- **Three cards.** Cytoshape (`PERFORMANCE_POOL` 62 → 63); **Mirrorform**,
  added in review as the card that turned the donor exclusion from structure
  into a field — it prints Mirrorweave's shape without the word "other", so its
  affected set includes the target, and `CopyRoles::OthersCopyRecipient` could
  not express it (`plans/handoffs/cv-1-review.md` A1); and Mirrorweave, which
  is the crate's **first registered card with a simplified hybrid cost** —
  audited, not assumed: of the 70 registered cards, checked against Scryfall,
  Mirrorweave is the only one whose printed cost carries a hybrid or Phyrexian
  symbol. **`inside_out` is a fixture and has never been registered**, which is
  the whole of the 2026-08-24 precedent: it prints `{1}{U/R}`, the crate spells
  it `{1}{U}`, and it stayed out of `registry.rs` rather than misrepresent the
  card. Mirrorweave is registered as `{2}{W}{U}` — a strict *subset* of the real
  card's legal payments — because `ManaSymbol::Hybrid` exists and no payment
  path handles it, so a verbatim cost would make the card *silently uncastable*.
  The precedent was right there and is wrong here for a reason about the pool
  rather than the card: Inside Out's engine path (Layer 7d) had a registered
  substitute in Merfolk Thaumaturgist, and Mirrorweave and Mirrorform are the
  only consumers of `CopyRoles::FilteredCopyRecipient`. **The first real hybrid payment path should delete
  the approximation and this paragraph together.**
- **The CR 613.2 sublayer inversion, fixed in the last two places it lived** —
  `compute.rs`'s `LAYER_ORDER` doc and `layers-architecture.md` §7 with its
  `Layer` code block and its algorithm sketch. 1a is copy, 1b is face-down.
- **A trace page**, `plans/traces/cv-1-a-copy-is-a-snapshot.html`: a Cytoshape
  resolution from the choice to the row, the row read back, the copied anthem and
  its leg-2 row, and CR 707.4's re-copy tearing that row down through the
  existence check rather than through a removal path.

**The measurement says the engine is exactly free.** Three binaries, interleaved,
200 games / seed 12345 / `--threads 1`, medians of five: **B (CV-1's engine,
pool unchanged) matches A (`main` at 103acf1) to the digit on every counter** —
100,496 walks, 136,402 frames, 1.36 frames/walk, 567 gathers — and their event
streams are **40/40 byte-identical**. What moved is the pool: `Frames/walk`
1.36 → 1.37 — **and that is game content, not a mechanism.** The first draft
of this entry attributed it to Citanul Hierophants being copiable, which was a
guess stated as a fact and does not survive arithmetic: Cytoshape resolves **16
times in 200 games** (counted, `--dump-events`), each row lives for the rest of
one turn, so copy rows exist during ~16 of ~6,160 turns and could account for at
most ~0.2 of the 0.7 percentage points even if every one had copied a static
ability. C also plays *different games* — a 63-card pool, not 62 — and its
walk count moved —0.5% and its turns +0.3% in the same run. The ratio shift is
inside that. **The mechanism that would raise `Frames/walk` is real** (a copied
static ability is an `EffectOrigin::StaticAbility` row and pays what a printed
anthem pays); this sample cannot see it. **The timing column separates
nothing** and the phase says so: run-to-run spread for one binary at 200 games
was —4% to +6% here, and B read *faster* than A while playing identical games.
`layers-architecture.md` §12's 5.2—8.0→ figure is the CR 613.7a check
*without its gate*, and a copy row never pays it — `EffectOrigin::Resolution`
short-circuits before any frame is computed. The phase was sized expecting its
risk in the copy row; the copy row is the cheap half.

**`--dump-events` is a weak instrument for this track**, which is worth knowing
before CV-2. Registering a row is not an observable action, so the event stream
sees a Cytoshape reach the graveyard and nothing else, in both arms. Every
divergence found (2 on `performance` A vs C, 7 on `stress` A vs B) is the first
differing *draw*, i.e. the pool changed size. **Reachability, counted:**
Cytoshape resolved once in 40 `performance` games, Mirrorweave twice in 40
`stress` games — thin, and named as thin. Determinism holds: three
`--threads 1` runs per pool, byte-identical outside `=== Timing ===`, 0 errors,
0 panics, 0 turn-limit hits.

**What CV-1 did not build**, deliberately: `Duration::Indefinite` (CV-1b, blocked
on item 10); the ETB path, CR 708 and CR 729, untouched; `is_copy`, still with no
writer; and `CopiableValues.back_face`, which
`copy-effects-architecture.md` §3.2 specifies and which was **omitted** — a
field no producer writes and no reader reads is a Deferred Migrations line bought
for nothing, and CV-5 adds it in the commit that populates it.

**Status 2026-09-02: Phase RC-4 ✅ — CR 614.12's look-ahead frame.** `replacement-architecture.md` §9's RC-4 subsection carries the six findings; this is the state ledger. What landed:

- **`layers::compute_as_entering`** — the frame. A `Lookahead` (`engine/layers/lookahead.rs`: the object, its proposed controller, the pending `EnterMods`, and the rows its own static abilities would generate) threaded through `FrameCache`, read by the two accessors every concrete-state read in `compute.rs` now goes through — `entity` for the battlefield-entity facts and `rows_in_layer` for the registry slice. Both answer for the would-be permanent when the object being computed is the entering one and off the real board for everything else, which is how §5b's asymmetry falls out of the structure. A read-side overlay, **no `GameState` clone anywhere** (§11 item 5). It lives on the stack: item 40's test is "drop it and re-derive", and the frame is a pure function of `GameState` and the proposal, so it is bookkeeping.
- **`replacement::EntryFrame`** — the frame per pipeline iteration (CR 614.12 clause 1), computed only when a filter-scoped `affected` asks. `gather::set_affects` and `restriction::is_prohibited` both read it, for an `EnterBattlefield` — which since RC-4b is also the zone change onto the battlefield, so the frame has one basis.
- **CR 614.17d** in its two printed shapes. "Can't enter the battlefield" watches the zone change, because a refused entry would strand the card in the battlefield zone with no permanent (`propose_entry` is still loud about that); "can't have counters put on it" is asked as the `AddCounters` it is (CR 122.6) at the moment an `EnterWith` would add them (`replacement::strip_prohibited_counters`), refusing the counters while the entry goes on — Melira's ruling. `cant-effects-architecture.md` §5.3.
- **CR 616.1b** — `Rewrite::EnterUnderControlOf(PlayerRef)`, its class derived from the rewrite so a card cannot file it under `Other`. "An opponent of your choice" with several opponents is a prompt to the effect's controller (`ChoiceKind::ChooseEnteringController`), made before the permanent enters — CR 614.12a's timing, claimed partially.
- **`AmountExpr::CountOf`** — its first static-context evaluator, over `battlefield_ids_ordered` at the current `layer_index`, memoized within the walk. The entering object is invisible to it because it is not on that list: §5a's Thassa boundary, structural.
- **§11 item 19** — CR 616.1 no longer prompts when every member of the bucket is an `EnterWith` whose applicability no `EnterMods` field can move; item 47 below has the three expiry conditions and the debug-build check. Containment Priest landed first, so the multi-candidate branch is still reached by a registered board.
- **`GameAction::EnterBattlefield` carries the zone change that brought the object** (`None` for a token), because CR 601's "was it cast" is unrecoverable a moment later; `EventPattern::EnterBattlefield { cast }` projects it.
- **Three cards.** Containment Priest and Dryad Arbor (stress pool), Keldon Warlord (`PERFORMANCE_POOL` 61 → 62). Wall of Stone gains the Wall subtype it always printed. Two fixes rode along: CR 205.1a in `land_types::apply_set_subtypes` — Blood Moon made Dryad Arbor a Mountain with no creature type; shown failing first, own commit — and the pool's colour check reading colour indicators (CR 204.1).
- **`targeting::permanent_matches_filter` takes one layer walk per filter** instead of one per leaf, and only when a leaf reads a characteristic; the entry path matches against the frame through `permanent_matches_filter_in_frame`.

**The measurement, three binaries.** Interleaved in one sitting, 50 games / seed 12345 / `--threads 1`, medians of seven rounds: `main` at 1045b70 (A), RC-4's engine with the pools exactly as `main` had them (B — the three cards unregistered, Keldon Warlord out of `PERFORMANCE_POOL`), and RC-4 shipped (C).

| | A: main | B: engine, pools unchanged | C: shipped |
|---|---|---|---|
| performance walks | 108,632 | 108,709 | 93,854 |
| performance frames/walk | 1.25 | 1.25 | **1.45** |
| performance ms/game | 99.1 | 102.2 (+3.2%) | 122.2 |
| performance ms / 1,000 walks | 0.912 | 0.940 (+3.1%) | 1.302 |
| stress walks | 98,843 | 98,889 | 85,738 |
| stress frames/walk | 1.20 | 1.20 | **1.31** |
| stress ms/game | 76.6 | 77.9 (+1.6%) | 80.9 |
| stress ms / 1,000 walks | 0.775 | 0.787 (+1.6%) | 0.944 |

**The frame is close to free; the count is the quadratic it was said to be.** B against A is the overlay's cost with nothing new on the board. Walks and frames are within 0.1% and 0.6%, and those deltas are game content rather than per-walk cost — the suppressed prompts shift the random agent's stream (below), so B plays slightly different games. On time B read slower at 50 games — +3.2% / +1.6%, slower in 5 of 7 `performance` rounds and 4 of 7 `stress` rounds — which §11 item 5 says to treat as the indirection leaking into the hot walk. Re-run at **200 games, three interleaved rounds**, it reversed on `performance` (B 84.1 ms against A's 86.0, **−2.2%**; per 1,000 walks 0.799 against 0.811) and held at **+1.1%** on `stress` (72.1 against 71.3; per 1,000 walks 0.769 against 0.761). Both are inside the run-to-run spread, so the 50-game number was noise, and inspection of the hot walk finds only what the overlay has to add to every frame — one `Option` check on the look-ahead at the seed, the zone gate and the row iterator, and CR 122.1a's two counts read once per frame rather than only at layer 7c. No leak found by measurement or by reading. What moved in C is **`Frames/walk`** — 1.25 → 1.45 on `performance`, 1.20 → 1.31 on `stress` — and that is Keldon Warlord: a walk of the Warlord asks every permanent's frame at layer 7a's ceiling, memoized within that walk, so it costs 1 + N frames where an ordinary walk costs about 1.25. Per 1,000 walks C is +38% over B on `performance`, which is the pool (the Warlord entered 18 times in 40 games) and not the engine: the walk that got expensive is the one `layers-architecture.md` §12 says gets expensive, and critical-path item 7's cross-call memoization is the lever, exactly as before this phase. Games also got shorter (31.7 → 28.7 turns on `performance`; a 4-mana */* that is usually 3/3 or 4/4 ends games), so `Layer walks` per game fell while each walk cost more.

**Event streams: every divergence is a suppressed prompt.** 40 games / seed 12345, `--dump-events`, canonicalized — both ObjectId spellings, per game — and diffed A against B, with the first divergence per game attributed rather than the whole stream (a behaviour change cascades: `RandomDecisionProvider` draws once per prompt, so one skipped prompt moves every later choice). **33 of 40 `performance` games and 32 of 40 `stress` games are byte-identical**, and every one of the 15 that diverge has, ahead of its first differing event, a CR 616.1 prompt RC-4 no longer asks: a Chainbreaker or Idyllic Beachfront entering under one Root Maze (11 games), or a land entering under two or three Root Mazes (4 games — two of one player's, or one of each player's, both apply to any entering land). No identical game contains a suppressed prompt. A against C differs from the first deck, which is the pool and not evidence of anything.

**Determinism holds.** 200 games / seed 12345 per pool, three `--threads 1` runs each, byte-identical outside `=== Timing ===`; `--threads 8` differs from `--threads 1` only by the header line that names the thread count, with the results and engine-work blocks identical. 0 errors, 0 panics, 0 turn-limit hits on both pools. The known `TokenCeasedToExist` reordering (item 6) did not appear.

**Reachability, counted rather than assumed** (40 `stress` games, C): Containment Priest entered 13 times, Dryad Arbor 7, Keldon Warlord 18, and the Priest exiled a Dryad Arbor twice — the non-commuting bucket, prompting in a fuzz game.

**What RC-4 did not build.** CR 614.13/13a/13b, the batch-scoped frame and a dynamic `EnterWith` amount are **RC-5** (`replacement-architecture.md` §9, sized before code); CR 616.1c is CV-2's. Grist and the Theros gods stay blocked on item 7f, not on the frame. Tests 823 → 853 (25 integration, 5 unit for the overlay, 1 unit for CR 205.1a; one RC-3 prompt test folded into its neighbour). `specdb owed` unchanged; ATOM-616.1b-001 claimed in full, ATOM-614.12a-001 and ATOM-614.17d-001 partially with the reason on the test.

**Status 2026-09-02: Phase RC-3 ✅ — CR 614.12's membership rule, both directions.** `replacement-architecture.md` §9's RC-3 subsection carries the findings; this is the state ledger. What landed:

- **`compute.rs::effect_applies_to` gates on the battlefield *zone*, not on `game.battlefield` membership.** `move_object` writes `obj.zone` before the `EnterBattlefield` performer builds the `BattlefieldEntity` — RC-2's one-`emit`-wide window — so an entering permanent is in the zone with no entry, and the stricter question kept CR 614.12 clause (3)'s "continuous effects that already exist and would apply to the object" away from every entry. Hidden zones are untouched: a card in hand still has `zone == Hand`, so nothing new reaches a library or a graveyard.
- **`gather`'s source 1a admits `AffectedSet::SourceOnly` only** (`SelfScope::EnteringSelf`). CR 614.12's parenthesis — "if they affect only that permanent (as opposed to a general subset of permanents that includes it)" — is a membership rule, so it is here rather than in RC-4's overlay. Without it an entering Orb of Dreams finds its own "Permanents enter tapped" through `set_affects`, which matches a `Filter` against any object in any zone, and taps itself.
- **RC-2's two known-wrong answers are now right.** The first is the one RC-3 named: a tapland under Blood Moon enters **untapped** (CR 305.7 strips the ability first), reachable in a fuzz game from cards already in the pool. The second was *not* Humility + Chainbreaker — that pair is a further consequence of the same line, real and tested, but RC-2 never listed it. RC-2's second was `default_enter_mods` and a filter-scoped Layer 4 effect, and RC-3's ledger misstated which direction changed (corrected 2026-09-02, RC-4): `default_enter_mods` reads *printed* loyalty and gates on the *effective* type, so "a planeswalker made one by a Layer 4 effect" has `loyalty: None` and enters with no counters either way. What the line changed is the inverse — a planeswalker whose type a filter-scoped effect **removes** now enters with **no** loyalty counters, because CR 306.5b gives the ability to "a planeswalker" and on that battlefield it is not one. `phase_rc4_integration_test::test_a_planeswalker_whose_type_a_filter_effect_removes_enters_with_no_loyalty` is the test; RC-4 routed the read through `compute_as_entering` so it is the CR 614.12 frame that answers, not `has_type` on a printed card.
- **`base_controller` grew a `resolving` leg.** `resolve_top_of_stack` takes the `StackEntry` before it resolves anything, so the battlefield and stack probes both miss for the whole resolution and the owner fallback answered — right for a land drop, wrong for a spell cast by a non-owner (CR 110.2b). RC-3 owns it because RC-3 is what makes `PermanentFilter::ByController` askable of an entering permanent. **It fixes a wrong answer no registered card can produce**: `check_cast_legality` refuses "another player's spell", so the test builds the owner/controller disagreement after an ordinary cast. Confirmed rather than argued — event streams at 40 games are **40/40 identical on both pools** with and without the leg. The trap it removes belongs to whoever relaxes that check — the Commander track and Phase RE both want to.
- **One card: Root Maze** (`{G}`, "Artifacts and lands enter tapped"), `PERFORMANCE_POOL` 60 → 61, plus `PermanentFilter::Or`. **Not for RC-3's own claim** — that one needed no new card and the argument is in the card's doc comment: Blood Moon and Humility were already in the pool, both are filter-scoped layer effects, and RC-2 had already put two permanents that enter modified in beside them, so the gate change widens a path the pool walks rather than opening one. Root Maze is for the gap RC-2 left by mistake (see the correction under RC-2 above): **CR 616.1's multi-candidate branch, reachable since RB merged and registered by nobody**, which is the identical failure RB shipped with Kalitas.

**The measurement, and it says the gate is free.** Interleaved A/B in one sitting against a same-day `main`, 50 games / seed 12345 / `--threads 1`, with a third binary carrying the gate change and *not* Root Maze so the engine and the pool could be separated (RC-2's lesson, applied):

| | main | RC-3, card unregistered | RC-3 shipped |
|---|---|---|---|
| performance walks | 99,877 | 100,556 (+0.7%) | 108,632 |
| performance frames/walk | 1.24 | 1.24 | 1.25 |
| stress walks | 93,245 | 98,099 (+5.2%) | 98,843 |
| stress frames/walk | 1.16 | 1.17 | 1.20 |
| ms/game, performance (median of 7) | 80.6 | 81.2 | 92.3 |
| ms/game, stress (median of 7) | 63.3 | 68.1 | 72.4 |

**`Frames/walk` was the number to watch and it did not move.** The fear was that admitting non-battlefield objects to filter matching would make `permanent_matches_filter` request other objects' frames, at the 5.2×–8.0× the ungated CR 613.7a existence check measures (`layers-architecture.md` §12). It does not, because the gate admits objects in the battlefield *zone* — outside a one-`emit`-wide window per entry, the same set as before. Normalised, ms per 1,000 layer walks moves **0.807 → 0.808 on `performance` (+0.1%) and 0.679 → 0.694 on `stress` (+2.2%)**, seven interleaved runs each. So the gate costs essentially nothing per walk; the `stress` walk count rises 5.2% because the games get *longer* (29.6 → 30.6 turns), which is the behaviour change paying for itself, not the predicate. The shipped column is not comparable to either, because the pool changed — §3.1.

**Event-stream check, and the shape of the answer is the point.** 40 games / seed 12345, `--dump-events`, canonicalized and diffed against a same-day `main` with Root Maze unregistered. **36 of 40 `performance` games and 34 of 40 `stress` games are byte-identical**; every one of the 10 that diverge has a Humility or a Blood Moon on the battlefield before the divergence and a permanent that enters modified entering under it. Unlike RC-1's pure deletion, a real behaviour change cascades — the whole-stream diff is meaningless after the first divergence — so the checkable claim is the **first** divergence per game, and all ten are Chainbreaker not dying to CR 704.5f, Adaptive Shimmerer hitting for 3 less (its three +1/+1 counters), or an Idyllic Beachfront that never needed untapping.

**Determinism holds.** `--games 200 --seed 12345 --threads 1`, three runs per pool, byte-identical outside `=== Timing ===`. `--threads 1` vs `--threads 8` produce identical engine-work counters on both pools, which is the check §82's stored fixtures made real. 0 errors, 0 panics, 0 turn-limit hits. The `stress` pool's known `TokenCeasedToExist` reordering (Deferred Migrations item 6) did **not** appear in two same-binary `--dump-events` runs at 40 games — it is a `HashMap`-order flake, so absence is not a fix and item 6 stands.

**One methodology note worth keeping.** A raw `diff` of two `--dump-events` files always fails: `ObjectId` is a per-process v4 UUID, so every line carrying one differs between runs of the *same* binary. Both the 8-hex short form and the full UUID form appear in the dump, and canonicalizing only the short form leaves four games per pool looking falsely divergent. Canonicalize both, per game, before concluding anything.

**What RC-3 did not build.** No `compute_as_entering`, no accessor pair, no `GameState` clone — all RC-4's, per `replacement-architecture.md` §11 item 5. RC-3 removed a gate; RC-4 builds the frame. **ATOM-614.12-001 stays unclaimed in full** and is a type-surface gap rather than a behaviour one: its board is Yixlid Jailer ("cards in graveyards lose all abilities") and `PermanentFilter` has no zone leaf, so the scenario is inexpressible. The engine's answer is right for the right reason — `move_object` has already left the graveyard when `gather` asks — but nothing can demonstrate it, so the Blood Moon test claims it partially and the atom waits for a zone-scoped filter.

**Status 2026-09-01: Phase RC-2 ✅ — entering the battlefield is a proposed event.** `replacement-architecture.md` §9's RC-2 subsection carries the eight findings; this is the state ledger. What landed:

- **`GameAction::EnterBattlefield { object, controller, mods }`**, proposed by `perform_action`'s `ZoneChange` arm the statement after it emits, and performed by `place_on_battlefield`. `EventPattern::EnterBattlefield`, `Rewrite::EnterWith(EnterMods)`, and `EnterMods { tapped, counters }` with a `merge` that is CR 616.1f's accumulation — status is `|=`, counters are `+` per kind.
- **`init_zone_state` is deleted.** It created the `BattlefieldEntity` from inside `move_object`, below the chokepoint. CR 110.2b's default controller — `GameState::resolving`'s only reader since RC-1 — is now `GameState::default_enter_controller`, read at the proposal. The field still has exactly one reader.
- **`init_etb_counters` is deleted.** CR 306.5b's loyalty is `GameState::default_enter_mods`, which *seeds the proposal* — so Phase RE's counter doublers will replace it with no further work. It is the first thing in the engine to be modelled as "what the rules say this permanent enters with" rather than as a direct write.
- **One emitter.** `GameEvent::PermanentEnteredBattlefield` was emitted by `stack.rs` for a resolving permanent spell and by `resolve.rs` for a token, and **not at all for a land drop** — the most frequent entry in the game. The performer emits it now, once, with the *effective* controller (CR 400.7a's Layer 2 row has already moved a stolen permanent spell by then).
- **`gather` grew source 1a and a gate leg**: the entering permanent itself, read off its effective ability list, ahead of the fast-path gate. Without it every "this permanent enters tapped" is dead text, because `replacement_ability_sources` is written by `register_static_effects` *inside* the performer. `chooser_for_event` reads CR 616.1's chooser off the proposal, because an entering permanent has no controller for `controller_or_owner` to find.
- **Two cards, one per half of `EnterMods`**: Idyllic Beachfront (CR 110.5b) and Chainbreaker (CR 122.6a). `PERFORMANCE_POOL` 57 → 59; `engineering-practices.md` §3's table re-recorded. A third, **Adaptive Shimmerer**, is registered into the stress pool alone: a 0/0 is the only board state that dies if CR 122.6a's counters arrive after the entry is observable, and until it existed that claim rested on a fixture the registered pool could not build (§3.3).
- **Event-stream check**: at 40 games / seed 12345, canonicalized and diffed against a same-day `main` binary with the new cards unregistered, **the only difference is one added `ETB` line per land drop** — 708 on `performance`, 699 on `stress`, zero deletions, zero reorderings.

**One performance finding, and it is about the measurement rather than the code.** RC-2 is the first phase to put static replacement sources into `PERFORMANCE_POOL`, so `gather`'s board-level fast path — "is anything here a replacement source" — is true from the first tapland onward and the sweep walked *every* permanent with a full `compute_characteristics` per proposed action. The exit-criterion A/B ran with the new cards unregistered and so measured the gate closed. A **per-permanent** gate (the same predicate one object at a time, exact by the same argument, byte-identical event streams) is worth **10.3% of total game time on `performance` and 9.2% on `stress`**. The lesson generalises past this phase: *an A/B that neutralises a PR's card additions measures the engine and not the game*, and both numbers are worth having.

**Two known-wrong answers it leaves, both RC-3's one line** (`compute.rs`'s `game.battlefield.contains_key` gate in `effect_applies_to`): no filter-scoped `ContinuousEffect` reaches an entering permanent, so Blood Moon does not strip an entering tapland's "enters tapped" (the real ruling says it does), and `default_enter_mods` would miss a planeswalker made one by a filter-scoped Layer 4 effect. `tests/phase_rc_integration_test.rs::test_blood_moon_does_not_yet_strip_an_entering_taplands_ability` asserts the wrong answer on purpose so RC-3 has to flip it.

**And one claim it got wrong, corrected 2026-09-02 before RC-3's line was written.** RC-2 recorded in four places — this block, `phase_rc_cards.rs` twice, and `replacement-architecture.md` §9 finding 7 — that *no* `AffectedSet::Filter` effect reaches an entering permanent, and concluded that CR 616.1's multi-candidate branch was unreachable until RC-3. **Both halves are false and neither was ever true.** `AffectedSet` is matched by two different functions on two different paths: `compute.rs::effect_applies_to` (`:629`) gates `ContinuousEffect` on battlefield membership, while `gather::set_affects` → `GameState::permanent_matches_filter` matches `ReplacementDef.affected` and `RestrictionDef.affected` with **no gate**. Probed on `main`: a Root-Maze-shaped `ReplacementDef` (`Filter { ByType(Land) }`, `EnterWith(tapped)`) already taps an entering Forest, and with Idyllic Beachfront entering under it the pipeline produces two candidates and `ask_choose_replacement` fires. So the branch has been reachable since RB, one registered card away, and Root Maze / Kismet / Loxodon Gatekeeper / Frozen Aether were never blocked on RC-3. **The generalisable error is naming a mechanism by its type rather than by its call path** — "an `AffectedSet::Filter` effect" reads like one thing and is two — and it cost the project the same unreachable-branch gap twice, after RB shipped Kalitas alone.

**Status 2026-08-26: Phase RB ✅ — the CR 616.1 pipeline is live and three consumers use it.** All nine of `replacement-architecture.md` §9's RB items shipped. What landed:

- **`apply_replacements`** — §4.1's loop, inside `execute_actions` upstream of `perform_action`. CR 616.1a–f, CR 614.5's applied set keyed on effect *instance*, CR 616.2's re-gather, CR 614.6's dropped event, CR 614.17/17c's blocked path, §4.1a's rider timing. `ActionContext.dp` has its first reader.
- **`execute_actions` is three phases, and the split is CR 704.3**: decide for every batch member against one board, then perform, then run riders. That is where §4.3's CR 101.4 APNAP ordering lives — choices in APNAP order of chooser, performance in batch order, riders last (CR 615.5). It returns `Result<Vec<GameAction>, String>` again, and the SBA sweep is the customer that earned it back.
- **New event vocabulary**: `GameAction::Destroy { source: DestructionSource }` (the outer event; its performer proposes the inner `ZoneChange`), `AddCounters`, `RemoveCounters`. `CounterType::{Shield, Stun, Finality}`; `GameEvent::CountersChanged`.
- **Indestructible is a CR 614.17 "can't"**, checked ahead of the pipeline in `engine::replacement::is_blocked`, not filtered at the two call sites that each held their own copy of it.
- **Three consumers**: counters (CR 122.1c/d/h, no card text, 164 cards), regeneration (CR 701.19a/b/c, `Primitive::Regenerate` + the rider CR 701.19a spells out), and Kalitas, Traitor of Ghet — the only RB card, chosen for difficulty.
- **Commander's two halves**: CR 704.6d as a state-based action and CR 903.9b as the rules' only `exempt_from_614_5` replacement. Since 2026-08-30 (`rb-review.md` H4) 704.6d's accepted moves join the SBA batch instead of being performed one at a time as they are offered — 704.3's "single event" is all of 704, not just 704.5, and each offer being its own event let a later owner decide against a board an earlier owner's move had changed.
- Eight primitives: four stubs given implementations (`Tap`, `AddCounters`, `RemoveCounters`, `CreateToken`) and four new (`Regenerate`, `CantBeRegenerated`, `RemoveFromCombat`, `RemoveAllDamage`).

**Five findings where the plan and the tree disagreed, all recorded in `replacement-architecture.md` §9:** `Uses::CounterBacked` does not survive the CR text; CR 122.1c's replacement half is restricted to destruction *by an effect*; `EventPattern` ships six arms and `Rewrite` two, because an arm the pipeline cannot apply is a card that silently does nothing; `AffectedSet` and `ZoneChangeCause` had to move into `types/` to keep the crate's layering; and §4.1's loop **hangs** on a declined `exempt_from_614_5` optional without a second set.

**The blast-radius watch held.** §11 item 7 predicted that every existing test would start traversing the pipeline; **zero new `DecisionProvider` prompts** appeared, §4.1's two-candidate rule was never relaxed, and `fuzz_games --games 50 --seed 12345` is identical to the pre-RB baseline on every line. Perf: 13.01 → 13.04 ms/game at `--games 200`, medians of three interleaved runs in one worktree.

**Still owed by the replacement track**, and none of it blocks RC:

- **CR 614.15 self-replacement has a bucket and no producer.** `ResolutionContext` still has three fields (§11 item 3). Consequence worth knowing: CR 614.17c's blocked-event path always drops the event, because the only class that could survive it has nothing in it.
- **§3.3 source 2 — static abilities functioning in other zones — is still unsized.** §11 item 4 asked RB to count the cards and it did not. The *shape* question is now answerable, though: `gather`'s sweep is written, so it is a zone parameter on one loop plus a timestamp on `GameObject` (item 9's, already owed), not a separate registry.
- **CR 704.7's same-result collapse** is still per-object inside the SBA sweep and does not reach player loss; it needs `GameAction::PlayerLoses` (item 6, RE).
- **`specdb owed` is unchanged at 38** and cannot be the gate for RB, because RB is part of Phase 6 and Phase 6 also covers triggered abilities — adding it to `SHIPPED_PHASES` would arm a gate against work that has not started. The honest measure is the CR 614/615/616/122.1/701.8/701.19 slice of Phase 6: **14 atoms fully covered, 3 partial, 49 uncovered**, and all but a handful of the 49 are RC's (614.12/13, 616.1b/c), RD's (615.\*, 614.9) or RE's (614.10/11/16, 704.7). **614.7a left that list on 2026-08-30**: it was parked as RD's because it reads like prevention, but it is a proposal-side rule about an event that never happens, and moving the check off `perform_action` (`rb-review.md` H1) made ATOM-614.7a-001 partially covered. The RB-shaped remainder is ATOM-614.17a-001, ATOM-614.17b-001, ATOM-614.17c-001, ATOM-616.1a-001 and the four `BOUNDARY-DEF-614.1*` markers.

---

**Status 2026-08-25: RA-1 ✅ (PR #58 + two follow-up commits), RA-2 ✅, RA-3 ✅. Phase RA is complete.** What landed:

- `ActionContext { dp, resolution }` threaded through every mutation chokepoint. `dp` is still unread — RB is where `apply_replacements` consults it. `resolution` is read: RA-3 stamps it onto every emitted event.
- `ZoneChangeCause` on `GameAction::ZoneChange`, **11** production movers labelled. Not 10, and not 9: §11's derivation counted `change_zone` callers and missed both `resolve.rs`'s direct `execute_action(ZoneChange)` for `Primitive::Destroy` **and** `play_land`, which wrote straight to `move_object` (see the RA-3 findings below).
- **Cast rollbacks left the chokepoint.** Four `cast_spell` failure paths had been routing CR 601.2 rewinds through `change_zone` since the Phase 6 migration. A rewind is not a zone change and no replacement may see one — under RB, CR 903.9b would have redirected a commander whose cast merely failed. Now `rollback_cast_to_hand`, tagged `// CAST-ROLLBACK:`.
- New events: `CardDrawn` (CR 121.5), `Tapped`/`Untapped` (transition-only per CR 603.2e), `AbilityActivated`/`AbilityResolved` (durable `(source, ability)` identity per CR 603.7h).
- `StackEntry.cast_from: Option<Zone>` and `StackEntry.ability_identity: Option<AbilityIdentity>`, with complementary invariants against `is_spell`.
- `perform_action`'s `Tap`/`Untap` arms are loud; `Primitive::Untap` gained the CR 608.2b guard that makes them safe.
- The untap sweep moved to `battlefield_ids_ordered` — routing it made its order observable.
- **`execute_actions`, the batch form (CR 704.3 / 510.2 / 502.1).** `execute_action` is `execute_actions(vec![action])`; the batch opens a `BatchId` that every event it emits carries, and a *nested* call joins the enclosing batch rather than opening its own (CR 120.3f makes lifelink's gain a *result of* the damage, and CR 120.4c/d process the results and then let the one damage event occur). Three callers batch: combat damage, the untap step, and the SBA sweep.
- **The SBA sweep is one event.** It used to gather 704.5f's victims, move them, and only then ask 704.5g's question. Now every condition is read against one game state and the moves are performed as a single batch, deduped per object — CR 704.7's same-result collapse, with the first condition in CR order naming the cause. `StateBasedActionPerformed` collapsed with it: one emission per check, where it had been one per action and none at all for the two creature-death sweeps.
- **`perform_action`'s `ZoneChange` arm is the only production emitter of `GameEvent::ZoneChange`.** `move_object` performs and says nothing. That is not tidiness: the arm is the only place that knows the `cause`, and the only place that can capture the CR 603.10a LKI frame *before* the object stops being a permanent. The frame is a `compute_characteristics` call taken ahead of the mutation — no overlay and no `GameState` clone; the entering-object hypothetical that needs one is RC-4's problem.
- **The three `// REPLACEMENT-BYPASS:` sites are closed** (item 2 below). `GameState::resolving` names the popped-but-not-yet-anywhere state instead of routing around it, which lets `remove_from_zone_collection(Stack)` treat a missing entry as expected for exactly that object and lets `init_zone_state` read CR 110.2b's controller. `init_zone_state_with_controller` is gone.
- **The type-specific death events are deleted, not demoted** (revised 2026-08-26 in review — a doc-comment policy was weak enforcement for something the type system can enforce). `CreatureDied`, `PlaneswalkerDied`, `LegendRuleSacrificed`, `AuraDied`, `SpellResolved` and `PermanentLeftBattlefield` are all gone. `ui/display.rs` builds its line from the `ZoneChange`'s `cause` and `lki`; `fuzz_games` counts deaths the same way. Three reasons, and the third is measured: they partition one event by type and permanent types are not a partition (a Gideon is both); they name a subset without naming its boundary, which ATOM-603.6c-001 needs; and **the redundancy was hiding a bug** — `CreatureDied` was emitted only by the SBA sweep, so a creature killed by a spell never counted, and `fuzz_games` undercounted at 5.3 where the zone changes say **6.2**. That is the one fuzz number RA-3 moves, and it moves because it was wrong.
- **RA-2's exit criterion holds and is grep-provable:** the only production writes to `life_total` or `entry.tapped` are `perform_action`'s own arms. RA-3 adds the same for `GameEvent::ZoneChange` emission.

**Three findings from RA-3, where the plan and the tree disagreed:**

1. **`play_land` was a fourth, undocumented chokepoint bypass** — and the most frequent zone change in the game (18.1 per fuzz game). It wrote straight to `move_object`, which is why `ZoneChangeCause::PlayedAsLand` had zero call sites and nobody noticed. `play_land` now takes an `ActionContext` and proposes like everything else.
2. **`rollback_cast_to_hand`'s doc comment was aspirational.** It claimed a CR 601.2 rewind is unobservable, but `move_object` emitted a `ZoneChange` for it anyway. Moving emission into `perform_action` made the claim true.
3. **`GameEvent::PermanentLeftBattlefield` had no emitter anywhere.** Dead since it was written. Deleted rather than wired up — see above.

**Still owed, and newly recorded (see the numbered items below):** the CR 601.2a forward-announcement gap (item 5), and the SBA mutations that are not zone changes and have no `GameAction` variant to propose through (item 6).

The replacement pipeline is designed to sit inside `execute_action` at `engine/actions.rs:86-89`. Every mutating action must flow through there for replacements to observe them. Status:

1. **Zone-change migration — ✅ done (2026-04-18).** `move_object` is now `pub(crate)` with documentation directing external callers to `change_zone` / `execute_action(GameAction::ZoneChange)`. All 12 previously-direct callers (5 SBA sites in `engine/sba.rs`, `Cost::SacrificeSelf` in `engine/costs.rs`, push-to-stack + 4 rollbacks in `engine/cast.rs`, cleanup discard in `state/game.rs`) now route through the chokepoint. `engine/actions.rs::change_zone(id, to)` is the new convenience wrapper. Internal helpers (`draw_card`, `play_land`, and the `GameAction::ZoneChange` arm itself) continue to call `move_object` directly from inside `engine/zones.rs`.

2. **Open-coded zone bookkeeping — ✅ CLOSED 2026-08-25 (RA-3 ticket 7).**
   - `engine/resolve.rs` `Primitive::CounterSpell` calls `change_zone(id, Graveyard)`, which tears down the `StackEntry` via `remove_from_zone_collection(Stack)`, then emits `SpellCountered` (2026-04-18).
   - The three `engine/stack.rs` sites (permanent-spell ETB, instant/sorcery → graveyard, `handle_fizzle`) are closed. They bypassed because the stack-pop-first pattern removes the object from the stack `Vec` before resolution begins, so `move_object` would have double-removed. **The fix names the in-between state rather than routing around it:** `GameState::resolving` records the popped object and the CR 110.2b controller the destroyed `StackEntry` was carrying, and two readers consult it — `remove_from_zone_collection(Stack)` (a missing entry is expected for exactly that object, and a bug for anything else) and `init_zone_state` (an entering permanent takes the resolving spell's controller). Cleared on every path out of `resolve_top_of_stack`, including the error ones, which is what `resolve_popped` exists to make single-sited.
   - All three now carry a cause, and `Resolved` vs `Fizzled` finally separates CR 608.2n/608.3 from CR 608.2b's counter-by-game-rules — previously indistinguishable Stack→Graveyard moves. **`// REPLACEMENT-BYPASS:` no longer names anything;** `move_object`'s doc comment is down to one exception, `// CAST-ROLLBACK:`, which is permanent.
   - Note for later: `resolving` also makes the pop-first pattern replaceable. CR 608.2 keeps a resolving spell *on* the stack; the engine pops it early so in-flight effects cannot see it. Turning that into a mark-resolving flag is now a change to `CounterSpell` and targeting alone, not to the zone code.

3. **Event-stream refit — ✅ CLOSED 2026-08-25 (Phase RA, three PRs).** Specified 2026-08-24 as the CR 614 phase's opening ticket block, from a census of all 42 production `events.emit` sites plus the known bypasses. Every item shipped:

   - **Draw-step draw through the chokepoint**, with `CardDrawn` — CR 121.5 makes a library→hand move without the word "draw" a different, trigger-visible fact (RA-2). Opening-hand draws stay direct: pregame, nothing can observe them.
   - **Tap/untap through the chokepoint**, with transition-only `Tapped`/`Untapped` per CR 603.2e (RA-2). The untap sweep is ordered, and since RA-3 it is one batch — CR 502.1 untaps simultaneously.
   - **Life mutations inside the chokepoint** — lifelink and `Cost::PayLife` propose `GainLife`/`LoseLife` instead of writing `life_total` (RA-2). Both *emitted* `LifeChanged` while writing by hand, which is exactly how the 42-site census missed them: the pipeline reads the proposal, not the event. One member of the class remains undecomposed and is *inside* the chokepoint — `perform_action(DealDamage)` performs a player's life loss inline, so a `LoseLife` watcher would miss combat damage. That is CR 120.3's results-of-damage decomposition, scheduled with Phase RD (which also owes CR 120.3c — see the CR 1 map's 120 row).
   - **`AbilityActivated` plus an identity-bearing `AbilityResolved`** (RA-2), for CR 603.7h counting.
   - **Payload upgrades** (RA-3): the CR 603.10a LKI frame on battlefield-leaving zone changes, the `cause`, a `BatchId`, and the resolution that proposed the event. The last two ride an `EventRecord` envelope rather than per-variant fields — they are the same two facts for every kind of event, and a variant that forgets to carry them fails silently.
   - **The three `// REPLACEMENT-BYPASS:` sites closed** (RA-3; item 2 above).
   - **Type-specific death events demoted** to display sugar (RA-3), with `PermanentLeftBattlefield` deleted and `SpellResolved` renamed `StackObjectResolved`.

   **Exit criterion, met:** every state mutation observable by CR 614 or CR 603 is emitted from exactly one place, and an event-log replay can distinguish drawn from tutored, destroyed from sacrificed, and countered from resolved.

4. **~~Unresolved architecture fork~~ — ✅ RESOLVED 2026-08-24 (owner decision): trigger detection is the performed-action event stream; the delta log is rejected.** No `engine/delta_log.rs` will exist. The shape: `GameAction` (proposed) → CR 614 replacement pipeline → perform → `GameEvent` (performed record carrying LKI frame, cause, batch id, resolution context) → **synchronous dispatch at the mutation instant** (turn trackers update; event triggers match against *effective* ability lists across all zones; state triggers and designations evaluate against live state) → pending-trigger queue → APNAP placement at the CR 603.3 moment (the `engine/priority.rs:240` stub). Detection happens per atomic event; only *placement* defers — CR 702.131d, the city's-blessing-before-SBA ruling, and CR 603.8's momentary-condition example all require exactly that split.

   `state-tracking-architecture.md` remains the statement of the four problems; its **Resolution postscript** records how each is answered and why the delta representation fails them (semantic identity is not recoverable from `(old,new)` state pairs — CR 121.5, 603.10e; evaluating past instants requires replaying the layer engine — CR 603.10, 603.6b, 702.131d). What the delta doc got right is adopted: central detection with no per-site knowledge of conditions, single-funnel emission discipline (item 3 above is that ticket list), LKI-over-type-specific-events, and resolution-context stamping. Loop detection Tiers 1–3 and D26 survive, with D26 transcripts re-based on performed-action sequences. Struck in writing: `roadmap.md` delta block + 2026-04-06 blockquote, `design_doc.md` §8 subsections + decision rows 2026-04-01 / 2026-04-11×2, and the CLAUDE.md authority-table carve-out.

   **N-player from day one:** trackers are per-`PlayerId` vectors plus global; the pending queue takes the player set and orders per CR 603.3b APNAP; CR 616.1 ordering hangs off the affected object's controller / affected player, with APNAP among simultaneous choosers.

5. **CR 601.2a announces a move that CR 601.2 may un-happen (recorded 2026-08-25, RA-3).** `cast_spell` proposes the hand→stack move at CR 601.2a — correctly, since the object really is on the stack while costs are paid — but the `ZoneChange` is emitted *then*, before it is knowable whether the cast rewinds. A rewind leaves the forward event in the log, so a replay sees a move CR 601.2 says never happened. RA-3 fixed the other half (the rollback itself is silent, which is what `// CAST-ROLLBACK:` had always claimed and never delivered); this half needs the announcement deferred to CR 601.2i without deferring the move, which is a two-phase cast rather than a payload change. **Sized:** one function, `cast_spell`, plus wherever the deferred event is flushed. `tests/phase_ra_integration_test.rs::test_a_failed_cast_announces_nothing` documents the gap where it lives. Not blocking RB — no replacement effect applies to a rewind — but it is a wrong entry in a log the trigger matcher will read, so it should land before Phase 6.

6. **State-based actions that mutate outside the chokepoint — partly closed by RB (2026-08-26).** `GameAction::AddCounters`/`RemoveCounters` exist now, so counters have a proposal vocabulary; CR 704.5q's *annihilation* still writes directly, because it removes two kinds at once and would have to join the SBA batch to propose. **A card went in ahead of that routing, deliberately (2026-09-01).** `battlegrowth` makes the annihilation sweep run in a fuzz game — 0 → 10 occurrences per 200 stress games — against the direct-write code exactly as it stands. The argument is Darksteel Myr's: coverage *before* a move is worth more than after, because the move is what needs a witness. It also gives the proposal vocabulary its first production reader — `CountersChanged` goes 0 → 82 per 200 games, so `perform_action`'s `AddCounters` arm and `gather`'s `EventSubject::Object` leg for it are no longer reached only by tests. Player loss, the Equipment detach and the token cease-to-exist are untouched. Original entry (recorded 2026-08-25, RA-3; extended 2026-08-26): CR 704.5q's counter annihilation, CR 704.5p's Equipment detach, and CR 704.5q's attachment catch-all write `BattlefieldEntity` fields directly; CR 704.5d's token cease-to-exist removes from `objects` directly. **Player loss (704.5a/b/c and CR 903.10a) is the fourth and the most consequential**: it writes `player_lost[i]` and emits `PlayerLost` without proposing anything, so CR 704.7's own worked example — Lich's Mirror replacing a loss that two rules would cause at once, ATOM-704.7-001 — cannot be expressed at all. RA-3's dedupe covers same-object zone changes and not this. The `!player_lost[i]` guard makes the outcome right by accident. Needs `GameAction::PlayerLoses` (CR 104; `replacement-architecture.md` §8a schedules it for Phase RE, where the 6 printed cards live). They are outside RA's exit criterion by construction — the criterion is about mutations CR 614 can observe, and there is no proposal vocabulary for a counter or an attachment yet. **RB item 5 adds `CounterType::{Shield, Stun, Finality}` and their effects, which is when counters need an `AddCounters` / `RemoveCounters` action;** the attachment pair wants one when Equip lands (CR 702.6). Until then they are correctly outside, not accidentally: `GameAction`'s own comment block lists them as the variants to add as primitives arrive. A token ceasing to exist is genuinely not a zone change (CR 704.5d removes it from the game) and `TokenCeasedToExist` is the right event for it. **The token sweep is also a live determinism leak, found 2026-09-01 while measuring RC-1 and pre-existing on `main`:** `engine/sba.rs`'s 704.5d gather iterates `self.objects` — a `HashMap` — straight into an ordered `Vec`, so two tokens ceasing to exist in one sweep emit `TokenCeasedToExist` in per-process order. It is invisible to `fuzz_games`' summary, which counts no such ordering, and shows up only in a `--dump-events` diff on the `stress` pool (two adjacent lines that swap between runs of the *same* binary). CLAUDE.md's rule covers it — "same rule for any collection reaching a choice" — and here the collection reaches the event log instead, which is why it went unnoticed. Routing the sweep fixes it, since `execute_actions` orders a batch.

7. **The early stack pop — ✅ DELETED 2026-09-01 (phase RC-1).** `resolve_top_of_stack` used to remove the object from the `stack` `Vec` before resolving, documented as keeping an in-flight Counterspell from seeing the resolving object. Nothing could see it: CR 608.2g forbids casting a spell or activating an ability during a resolution, so no effect can *acquire* it as a target mid-resolution, and a spell cannot choose itself at CR 601.2c because `enumerate_legal_selections` (`oracle/legality.rs`) and `has_any_legal_choice` (`engine/targeting.rs`) already exclude it by `exclude_id`. The CR meanwhile keeps a resolving spell **on** the stack (CR 608.2; 608.2n/608.3a move it at the end), so the pop was an engine artifact the rules do not have.

    **What shipped.** The pop is gone; the `StackEntry` is still taken (the body owns it). `move_object`'s `remove_from_zone_collection(Stack)` now does the removal it was always asked to do and finds the object where CR 608.2 says it is, which deleted the leniency branch that had licensed a stack removal finding nothing — a branch that could mask a genuinely missing stack object. The two ability paths (`resolve_taken`'s completed arm and `handle_fizzle`'s) remove from `stack` explicitly, because an ability ceasing to exist is not a zone change. `resolve_popped` was renamed `resolve_taken`: there is no pop for it to be named after. **`GameState::resolving` now has exactly one reader** — `init_zone_state` at `engine/zones.rs`, CR 110.2b's default controller — and it is a rules question rather than an engine artifact, so the field's doc no longer describes a window.

    **The audit, re-run 2026-09-01 and corrected.** `replacement-architecture.md` §9 claimed five `stack.is_empty()` readers; there are **six**. `zones.rs:169` had drifted to `:177`, and the unlisted one is **`ui/display.rs:287`, `format_stack`** — which has **no production caller at all** (it is `pub`, and every use is a test in its own file), so the deletion changed no rendering. The CR is on the deletion's side even where it does render: CR 608.2 puts the resolving object on the stack, so showing it is right, and the CLI would only ever reach it through a mid-resolution `DecisionProvider` prompt. `engine/stack.rs:27`'s guard is a seventh occurrence and correctly excluded from both counts — it runs before the resolution, not during. The `GameState::resolving` count of six was exact. **None of the six is reachable during a resolution today**, which is what made the deletion safe; CR 608.2g's "unless an effect instructs" case makes `engine/cast.rs`'s reachable once RC-era cards arrive, and that site now carries a comment saying so and naming the choice it will have to make. Not fixed here.

    **Exit met, mechanically.** Whole suite green, zero warnings, and `fuzz_games --games 200 --seed 12345` byte-identical to a same-day `main` binary on **both** pools outside the `=== Timing ===` block; three runs each, identical. A `--dump-events` diff at 40 games was added on top of the summary and is identical too, after canonicalizing the per-process v4 `ObjectId`s — 18,097 event lines on `performance`, 18,738 on `stress`, same kinds and same counts. No new `GameEvent`, no new behavior.

8. **CR 608.3b is unimplemented: a permanent spell with an illegal target does not fizzle (found 2026-08-26).** `resolve_popped`'s fizzle check reads `extract_recipient(&entry.effect)`, which for an Aura is the *spell ability's* recipient — and an Aura has no spell ability, so `has_targets` is false and the check never runs. The Aura's actual target lives in `entry.chosen_targets` and is read later, at the attach step. CR 608.3b says such a spell "doesn't resolve. It is removed from the stack and put into its owner's graveyard." Today it resolves and enters the battlefield attached to a target that may no longer be legal. Predates RA and is unreachable in the current pool (no registered Aura is castable from hand — `cast.rs` never reads `enchant_filter`), but it is the other half of the fizzle path RA-3 just routed, so it is recorded here rather than in the RA ledger. **Sized 2026-09-01, and it is one helper, not three:** `oracle/mana_helpers.rs::spell_recipient`, the inline block in `engine/cast.rs`, and `engine/stack.rs::extract_recipient` are three copies of the same fourteen lines, computing a spell's recipient from its *effect* — so none can see an `enchant_filter` and all three must learn the Aura rule together or disagree. **Scheduled 2026-09-01 as Phase LH-1** (`layers-architecture.md` §13a), and deliberately *not* as its own PR: **zero registered cards carry an `enchant_filter`**, so the shared helper returns exactly what the three copies return today for every card that exists. Shipping it alone would put a new arm in front of the performance pool that no card can open -- the failure `engineering-practices.md` §3 is written against -- so it ships with Holy Strength, which makes it live. **A second blocker was found the same day and it is the larger one: fixing 608.3b still would not make an Aura registerable.** No `AffectedSet` names an Aura's host — `static_affected_set` has two productive arms (`FilteredPermanents` → `Filter`, `Implicit` → `SourceOnly`), `Duration::WhileEnchanted` has no consumer, and `register_static_effects` runs inside `place_on_battlefield`, *before* `resolve_taken` attaches the Aura, so even `Fixed` has nothing to capture. Every faithful Aura's text is about its host, so the Aura half is a phase with a layers change in it. `engine/resolve.rs::attach_aura_on_etb` is meanwhile dead code — zero production callers, three unit tests — implementing CR 303.4g's choose-on-entry for a path no card can take; the live path is `engine/stack.rs`'s Aura branch.

### Found by a judge-corpus pass (2026-08-26)

Five card interactions were put to the engine by the owner. Two moved
`replacement-architecture.md` §5 (see §5b/§5c there); three are general and land
here. None is blocking RB.

9. **CR 110.2b: a permanent spell's default controller is its *caster* — ✅ FIXED
   2026-08-26 (found and fixed the same day).** `stack.rs` hands
   `get_effective_controller(spell)` to `place_on_battlefield`, so
   `BattlefieldEntity.controller` — which `compute.rs::base_controller` treats as
   the *default*, the value Layer 2 modifies — becomes the player who stole the
   spell. CR 110.2b says the opposite: "the first player controls the permanent
   that spell becomes, **but the permanent's controller by default is the player
   who put that spell onto the stack**." CR 400.7a is what keeps the thief in
   control — the steal's Layer 2 row continues to apply to the permanent — so the
   *effective* answer is right today and the base is wrong underneath it.

   **Where it bites: CR 800.4c**, which the rule's own parenthetical points at.
   When the thief leaves a multiplayer game and the control effect ends, the
   permanent should revert to the caster or be exiled; with the base wrong it
   stays with a player who is gone. That is 4-player Commander, i.e. v1.
   `tests/phase_lg_integration_test.rs::test_gaining_control_of_a_permanent_spell_moves_the_permanent`
   currently asserts the wrong value and its comment argues for it.

   **The fix.** `resolve_top_of_stack` now reads two controllers instead of one:
   `effective_controller` (who follows the spell's instructions, CR 608.2c, and
   controls the permanent) and `default_controller = entry.controller` (CR
   110.2b's "player who put that spell onto the stack"). `ResolvingObject` carries
   the second, `init_zone_state` writes it, and the steal's Layer 2 row layers the
   thief back on top per CR 400.7a.

   `tests/phase_lg_integration_test.rs::test_gaining_control_of_a_permanent_spell_moves_the_permanent`
   is the distinguishing observation and now asserts **both** halves — base 0,
   effective 1. It previously asserted base 1, which is the wrong branch of an
   underdetermined pair: `get_effective_controller` answers 1 whether the permanent
   entered under P0 with a row on top or under P1 with the row double-counting, so
   an effective-only assertion lets either model pass. It also now carries
   `COVERS-PARTIAL: ATOM-400.7a-001` for that rule's *controller* clause; the
   atom's own scenario is the characteristics clause (Deathlace recolouring a
   creature spell) and stays open.

   Still owed from this item: carrying the caster onto the *permanent*, which
   customer 3 below needs at ETB-trigger time and which is RC's to place.

   **Three unrelated customers for one fact — record it (updated 2026-08-26).**

   1. **CR 110.2b itself**, above: the default controller *is* the caster, so the
      rule cannot be implemented without the fact.
   2. **Uphill Battle** ("Creatures played by your opponents enter tapped") — a
      CR 614 replacement whose predicate is who *cast* the spell, not who
      controls it. Weak on its own: `o:"played by"` is **1 card in all of
      Magic**, `o:"cast by"` is 2. The "played by" `PermanentFilter` leaf stays
      deferred on that count; the fact underneath it does not.
   3. **Bringer of the Last Gift** and its whole template — "When this creature
      enters, **if you cast it**, …". That is a CR 603.4 intervening-if whose
      "you" is the *ability's* controller, so a copy of the trigger controlled by
      someone who did not cast the permanent fails the check and does nothing.
      **`o:"if you cast it"` is 82 cards.**

   Note what RA already supplies for customer 3, and what it does not. "Was it
   cast at all, or put onto the battlefield by an effect" is answered by RA-3's
   `ZoneChangeCause` on the entering zone change — a resolved permanent spell
   arrives with `Resolved`, an effect putting one onto the battlefield will
   arrive with its own cause. What is missing is only *by whom*, which is the
   `entry.controller` value this item is already about.

   **The rule this settles, and it is worth stating generally: count cards to
   decide when to build a *feature*; never to decide whether to record a *fact*.**
   A feature — a filter leaf, a `Rewrite` arm — is a normal diff whenever it is
   added, so §8c's two-customers guard applies and deferring is free. A fact —
   who cast this, is this the same object, what were its characteristics an
   instant ago — is unrecoverable if it is not captured at the moment it exists,
   and adding it later means re-threading every system built in between. Phase RA
   was, in its entirety, a facts phase; that is what "the event spine" meant.

10. **CR 400.7 is unimplemented: an object keeps its identity across zones (found
    2026-08-26; the *field* landed with RB, the rule did not).** `GameObject.zone_change_epoch` now exists — stamped by `move_object`, read by CR 704.6d — so the tick this item wanted is recorded and does not need re-threading later. What is still missing is the rule itself and, more importantly, its exception list. `move_object` preserves the `ObjectId`, and
    `cleanup_zone_state` removes only effects *sourced by* the leaving object,
    never effects *targeting* it. So a `Duration::UntilEndOfTurn` pump on a
    creature that dies and returns the same turn still applies to it, against
    "an object that moves from one zone to another becomes a new object with no
    memory of, or relation to, its previous existence."

    Two consequences worth separating. The **default** is wrong as above. The
    **exceptions** (400.7a–c: effects that changed a permanent spell's
    characteristics or controller, and prevention effects, continue to apply to
    the permanent it becomes) currently work *by accident*, because we never
    break the relation in the first place — so implementing 400.7 without its
    exception list would regress item 9's control case and Xu-Ifit's
    "has no abilities" rider.

    `plans/alchemy-mechanics-audit.md` already designed a
    `last_zone_change_epoch` on `GameObject` for this and calls it "already
    designed"; that document is **not** in `CLAUDE.md`'s authority table and
    nothing implements it. Either promote the design or restate it here before
    the first card needs it. This is the general form of the Xu-Ifit case in
    `replacement-architecture.md` §5c, and it is a replacement × continuous
    interaction, so RC is the natural forcing function.

11. **`AbilityType::Mana` is a printed tag; CR 605.1 defines mana abilities
    dynamically (found 2026-08-26, via Toph + Caged Sun).** `engine/mana.rs` and
    `engine/priority.rs` dispatch on the tag a card author wrote. The rule does
    not care what the author thought:

    - **CR 605.1a** — an *activated* ability is a mana ability if it doesn't
      require a target, could add mana, and isn't a loyalty ability, "regardless
      of what other effects they may generate". An ability that adds mana and
      also draws a card is a mana ability. A mis-tagged one is dispatched to the
      stack and gets priority it should never have. **A Layer 6 `GrantAbility`
      carries its author's tag onto a new object**, which is the same failure
      with no card author in the loop.
    - **CR 605.1b** — a *triggered* ability is a mana ability if it triggers from
      a mana ability's activation or resolution (or from mana being added) and
      could add mana. Triggered mana abilities **resolve immediately and never
      use the stack**, and that is not optional polish: CR 601.2g's mana window
      during casting depends on it. Entirely unmodelled. Caged Sun's "Whenever a
      land's ability causes you to add one or more mana of the chosen color, add
      an additional one" is exactly this, and Toph, the First Metalbender
      ("Nontoken artifacts you control are lands in addition to their other
      types") makes Caged Sun a land, so Caged Sun's own ability becomes a land's
      ability and the loop question the CR is imprecise about becomes reachable.
      Do not build a loop guard for it — CR 731 detection is out of scope
      (`replacement-architecture.md` §12) and the ambiguity is the rules', not
      ours. Do model 605.1 properly, so that when the case arrives the engine is
      wrong for the same reason a judge would be, not for a modelling shortcut.

    **What is already right:** `engine/layers/land_types.rs` grants the intrinsic
    `{T}: Add` on a *basic land subtype* (CR 305.6), not on `CardType::Land`, so
    Toph's reminder text — "(They don't gain the ability to {T} for mana.)" — is
    honored without doing anything. It has no test naming Toph; it should.

12. **`EffectiveCharacteristics.toughness: Option<i32>` is load-bearing, and
    `unwrap_or(0)` in the SBA sweep is doing rules work (found 2026-08-26, via
    Taskmaster + The Seriema).** Taskmaster, Mercenary Mimic copying a stationed
    Spacecraft dies immediately, and the chain is three rules deep: CR 721.2b
    makes a station symbol's `[P/T]` part of a *static ability* ("as long as this
    permanent has N or more charge counters… is a creature with base power and
    toughness [P/T]"), CR 721.2c gives station cards no P/T off the battlefield,
    and CR 707.2 excludes counters from copiable values. So the copy has the
    station ability, zero charge counters, and **no power or toughness at all** —
    and 704.5f puts it into the graveyard. `get_effective_toughness(...)
    .unwrap_or(0) <= 0` produces exactly that. The 2026-08-26 review audit
    classified that `unwrap_or(0)` as "0 is a real value here"; it is more than
    that, and the `Option` must never be flattened to an `i32` with a default at
    the type level. Unreachable until Layer 1 (copy) and Station land; recorded so
    neither phase quietly removes it.

### Found by the "can't" design pass (2026-08-27)

Three facts recorded by `plans/cant-effects-architecture.md`, which is now the
authority for CR 101.2 / 614.17 / 613.11. The design is written; none of it is
built, and none of it blocks RC-1 through RC-3.

13. **Ticket `L15` is superseded and was never built.** `plans/archive/
    implementation-plan-final.md`'s "Post-layer pass" specified a
    `PlayerActionRestriction` enum with `CantCastSpells(PlayerId)`,
    `CantGainLife(PlayerId)`, `CantAttack(PlayerId)`,
    `CantActivateAbilities(PlayerId, Option<String>)` and
    `CantDrawExtraCards(PlayerId)` as sibling variants. Grep confirms none of it
    exists in `src/`. It is a variant per card wearing a rule's name —
    `CantGainLife` and `CantDrawExtraCards` are the same restriction with
    different `EventPattern`s — and `cant-effects-architecture.md` §6.2 replaces
    it. **What L15 owned that the restriction model does not:** `lands_per_turn`
    is still a raw field (`state/player.rs:23`, read directly by
    `PlayerState::can_play_land`) and is a *computed player-scoped value*, not a
    restriction. It belongs with the cost-modification phase, the other CR 613.11
    consumer (Before Layers item 3). Four corpus atoms still carry the `L15`
    ticket: `ATOM-601.3-001`, `ATOM-613.10-001`, `ATOM-613.11-001/002`.

14. **A duration CR 608.2c does not give it — the scope is too *broad*.**
    ⚠️ **RELOCATED, NOT FIXED, by RS-1 (2026-08-31).** `turns.rs`'s
    `cant_be_regenerated.clear()` and the `GameState` `HashSet` are both gone;
    the restriction is now a `RestrictionRegistry` row whose duration is a
    `Primitive::Restrict` argument the *card* writes. **The scope is still
    `Duration::UntilEndOfTurn` and still wrong** — what changed is that it is
    now authored and greppable instead of an engine assumption, so correcting it
    is a one-argument change at each card rather than an engine change. The fix
    remains a resolution-scoped `Duration` variant, and RS-1 did not add one:
    §7's "must not become one 5,000-line PR" won, and the variant needs a hook
    at the end of a resolution that `resolve_effect`'s recursion (Sequence,
    riders) makes a design question rather than a line. **The next phase that
    touches `Primitive::Restrict` owns it.** Original finding follows.

    The CR 514.2 cleanup used to clear
    `GameState::cant_be_regenerated` under a comment asserting that "can't be
    regenerated" is a this-turn fact, with no rule cited. The governing rule is
    **CR 608.2c**, which names this exact card text as an example of later text
    modifying the meaning of earlier text: "Destroy target creature. It can't be
    regenerated" is one instruction, so the restriction is scoped to *that
    destruction* and is not a continuous effect at all. (CR 611.2's "until end
    of game" was an earlier misreading of this and is wrong — 611.2 never
    engages.) Reachable divergence: Wrath of God destroys a creature carrying a
    CR 122.1c shield counter, the shield replaces the destruction, the creature
    survives, and the engine withholds every regeneration shield from it until
    cleanup where the CR allows one immediately. Fixed by a resolution-scoped
    `Duration` variant, which does not exist yet — `Duration` today has no way
    to say "for the event this resolution is about to perform".

    **The general rule is the part worth keeping** (`cant-effects-architecture.md`
    §9 finding 1): CR 608.2c instructs the reader to "apply the rules of English
    to the text", so a restriction's scope **cannot be derived by the engine**
    and must be authored per card. Two cards with identical restriction text can
    have different scopes because of the sentence before them.

15. **Four `KeywordFlag` variants are constructible and enforced nowhere.**
    `Hexproof`, `Shroud`, `Menace` and `Intimidate`. The only `KeywordFlag::`
    references to them in `src/` outside the enum definition are `ui/display.rs`
    (two of the four) and `layers/land_types.rs`'s hexproof insertion. A card
    carrying one can be registered today and will quietly do nothing.
    `engine/targeting.rs:39` still carries the `T22` TODO that would fix two of
    them; `Menace` and `Intimidate` are `T21b`'s. All four are Tier 1a/1d in
    `cant-effects-architecture.md` §2.3 and land in RS-2 / RS-3.

16. **Two ETB-time scans read *printed* abilities, and a Layer 1 copy defeats
    both (found 2026-08-29, writing `copy-effects-architecture.md` §4.7).**
    `place_on_battlefield` runs exactly two ability scans, and both take
    `card_data.abilities`:

    - `register_static_effects` (`state/game_state.rs:737`), whose doc says it
      "Reads printed abilities on purpose" — correctly, to avoid circularity
      inside `place_on_battlefield`. Consequence: **a copy of a permanent with a
      static continuous ability registers no row for it.** A Clone of Glorious
      Anthem would pump nothing.
    - the `replacement_ability_sources` insert (`game_state.rs:760`), which is
      `rb-review.md` I9: a copied replacement ability is invisible to `gather`'s
      fast-path gate (`gather.rs:143`) and silently never applies.

    **→ Both fixed, CV-1, 2026-09-02** — and there were **three** scans, not
    two. `restriction::is_prohibited`'s gate (`restriction/predicate.rs:87`) is a
    third instance of the same shape, built by RS-1 after this item was written,
    under a comment that already named CV-1 as the owner of its third leg. What
    shipped: `RegistryScopeSummary::any_copied_replacement` and
    `any_copied_restriction`, each recomputed from the rows on every mutation and
    each scanning the *captured ability bodies* rather than counting copy rows, so
    a copy of a vanilla creature turns neither on; and
    `GameState::register_copied_static_effects`, beside `register_static_effects`
    rather than changing its read, since that read is correct for the case it was
    written for. Removing the derived rows needed no path: they are
    `EffectOrigin::StaticAbility`, so CR 613.7a stops applying them as soon as the
    copy expires or a CR 707.4 re-copy supersedes it, and matching their
    `Duration` to the copy row's retires them in one CR 514.2 sweep. Recorded
    here because the general rule outlives the instance and Phase 6 will meet it
    next: **a fast-path gate must be derived from the same place its sweep
    reads.** `gather` reads the *effective* ability list; the gate reads two
    *sources* of ability. Layer 1 and Layer 3 are the two routes to that list
    that do not exist yet, and each needs a leg on every such gate. Triggered-
    ability registration (critical-path item 6) will want the same scan.

    `game_state.rs:249`'s "between them the gate is sound" is now scoped to
    "sound only until Layer 1 or Layer 3 exists", with the §4.7 pointer
    (2026-08-30, `rb-review.md` I9). **Layer 3 is the only route left**, on both
    gates; the comments at `gather.rs` and `predicate.rs` say so.

### Found by CV-1 (2026-09-02)

16b. **A re-copy inside one turn leaves its superseded derived rows in the
    registry, inert, until CR 514.2 (found 2026-09-02, CV-1).**
    `register_copied_static_effects` registers a CR 613.7a row per static ability
    in a `CopyFrom` capture. When a CR 707.4 re-copy replaces the copy row, the
    old derived rows stop *applying* immediately — the existence check reads the
    subject's frame, which now carries the new capture — but they stay
    registered until their `Duration` retires them with the copy row they came
    from. **Harmless for everything CV-1 ships**, which is turn-bounded only: one
    cleanup step retires the lot, and an inert row costs a `HashSet` lookup in
    `effects_in_layer`'s slice.

    **It stops being harmless at `Duration::Indefinite`**, which is CV-1b: a
    permanent re-copying itself every turn would accumulate derived rows without
    bound, and nothing would remove them. **So this is CV-1b's prerequisite, not
    a bug in CV-1.**

    **Three things this is not, because an earlier draft of this entry ran them
    together** (corrected 2026-09-02 in review):

    - **It is not item 10's problem.** Item 10 is CR 400.7: `move_object`
      preserves `ObjectId`, so a row keyed on a *leaving* object's id
      re-attaches to whatever comes back. A derived row's `source` is the
      copying permanent, so `remove_by_source` already reaches it on a zone
      change. What is unreachable is "the ability that justified this row is
      gone", which is a different question with a different fix.
    - **It is not solved by updating the row in place.** A re-copy must
      **add** a `CopyFrom` row, never overwrite the existing one's payload:
      CR 613.2a orders layer 1 by timestamp and each row carries its own
      `Duration`, so an `UntilEndOfTurn` copy laid over an `Indefinite` one has
      to expire *back* to the indefinite values. Overwriting would delete an
      effect that is still running. Two rows is the correct model and the
      litter is the price of it.
    - **It is not the pump-spell shape either.** A pump row is inert-but-present
      for a different reason (its subject left and returned); this one's subject
      never moved.

    **Sized:** at the moment `apply_copy` registers a `CopyFrom` for a subject
    that already has one, `retain` out the derived rows whose
    `EffectGroup::StaticAbility(subject, ability)` names an ability id that the
    *new* capture does not carry — and only those, because the subject may also
    **print** the same ability, which no copy row justifies or removes. One
    `retain` and one set difference; the care is entirely in the second clause.

### Found by CV-1's reachability mode (2026-09-02)

16c. ~~**Spells resolve without being paid for, and `cast_spell` is one missing
    `rollback_cast_to_hand` away from correct**~~ ✅ **closed 2026-09-02,
    `fix/cast-rollback-on-payment-failure`** (found 2026-09-02 by CV-1's
    `--require`; diagnosed the same session; fixed the next).

    **The defect, as diagnosed.** `cast.rs` had five fallible steps between CR
    601.2a's silent move to the stack and CR 601.2i's announcement. Four called
    `rollback_cast_to_hand` before returning `Err`; the fifth — `pay_costs`,
    the last statement before 601.2i — was a bare `?`. When it failed the card
    stayed on the stack, was never announced as cast, and resolved on the next
    pass with the mana still in the pool. `activate_ability` had handled the
    identical failure correctly the whole time, and `priority.rs`'s "leave game
    state clean" contract named the one that did not.

    **The mechanism, confirmed by the failing test rather than inferred.**
    `can_pay_costs` passes; then `ask_choose_generic_mana_allocation` lets the
    player split the generic part across every type in the pool, capped only by
    each type's amount, so a split that spends a colour a pip still needs
    passes the prompt's own validation and `ManaPool::pay` refuses it. Grizzly
    Bears `{1}{G}` against `{R}{G}` with the generic put on Green is the whole
    reproducer, and on the pre-fix tree it fails at "back in hand: left
    `Stack`". The prompt only runs when `generic_count() > 0`, which is why
    every ghost had a generic component and no zero-generic card ever was one.
    Census of the **199** in 40 `performance` games on `main` at 650a263:
    Merfolk Thaumaturgist 17, Blood Moon 15, Volcanic Upheaval 13, Serra Angel
    10, March of the Machines 10, … and never Lightning Bolt, Counterspell,
    Dark Ritual or Giant Growth. (The 206 the first draft cited was at 103acf1,
    before CV-1 grew the pool.)

    **What landed.** (1) The rollback, in the shape of its four siblings.
    (2) `test_a_cast_whose_payment_fails_rewinds_and_keeps_the_mana`
    (`phase_ra_integration_test.rs`), shown failing with `mtgsim/src` stashed:
    back in hand, both mana still in the pool, no `SpellCast`, stack and
    `stack_entries` empty, event log unchanged. (3) **The durable half:
    `fuzz_games` checks, in every game and every mode, that every object
    leaving the stack with `ZoneChangeCause::Resolved` has a prior `SpellCast`
    for that object** — counted per object, so a recast owes a second one; an
    ability ceases to exist and emits no zone change, so it never trips it. A
    violation prints beside errors and panics with the card and the event
    index, sums as `Uncast resolved:` in the results block, and fails the run.
    By the same rule `main`'s dumps read **199 / 188** (`performance` /
    `stress`, 40 games) and the fixed tree reads **0** in every run — three
    rounds of 200 games per pool, plus the 40- and 50-game runs. It would have
    caught this on the first fuzz run after the cast pipeline landed.

    **What the free spells were worth — 200 games / seed 12345 / `--threads
    1`, medians of three interleaved rounds, both binaries in one sitting.**
    `main` resolved **5.5** spells per `performance` game and **5.0** per
    `stress` game that it never announced — 1,104 and 991 in 200 games — on
    top of the 21.7 / 20.4 it did. The fixed tree announces **27.9 / 24.8**, so
    *about the same number of spells reach the battlefield*; they are now paid
    for, and the mana they cost is not spent on something else. That is what
    moved everything else: turns 30.8 → 33.3 and 30.0 → 30.9, and the cost
    rows with them — walks 99,952 → 116,233 and 95,855 → 100,619, of which
    walks *per turn* are +7.5% / +2%. Per-walk time is flat-to-down, 1.125 →
    1.081 and 1.056 → 0.988 ms per 1,000 walks (−4% / −6%, interleaved), and
    `Frames/walk` fell on both pools (1.37 → 1.33, 1.33 → 1.29): the ghosts
    were disproportionately three- and four-mana statics — Humility, Blood
    Moon, March of the Machines and Glorious Anthem are 37 of the 199 — which
    are exactly the permanents that put a sub-frame under every walk. Creatures
    died is flat (7.9 → 7.8, 4.9 → 4.6). `engineering-practices.md` §3 carries
    the 50-game table, re-recorded: the first re-record where the pool did not
    move, so every row in it is the engine's.

    **Attributed, not assumed:** 40 `--dump-events` games per pool, ids
    masked, first divergence per game. `performance`: all 40 diverge, and
    every first divergence lies inside its game's first ghost's window — after
    the card was drawn, at or before the event where `main` resolved it unpaid
    — none outside, none in a game without a ghost. `stress`: 39 of 40 the
    same way, and the one game with no ghost on `main` is byte-identical after
    masking. In 30 of the 40 `performance` games the logs are identical *up to*
    the unpaid resolution itself: the failed cast is silent on both trees, the
    random agent's next decisions do not depend on where the card went, and the
    trees part at the pass-pass that resolves a spell on one and nothing on the
    other.

    **Should an illegal split be refused at the prompt instead? Yes; it is
    small; and it is its own PR, deliberately.** Every other `ask_*` offers a
    `DecisionProvider` only legal choices — targets are enumerated legal,
    priority actions are, CR 616.1 asks only among applicable effects — and a
    DP should not need payment law to answer "which mana". The clamp is exact
    and about ten lines: each bucket's max becomes `available − pips of that
    type`, and whenever `can_pay` passed the clamped maxima still sum to at
    least the generic count, so a feasible answer always exists. Two reasons it
    is not here, both consequences of the fix landing first: it moves game
    content a second time — every one of those ~5 rewinds per game becomes a
    cast — and the measurement above is readable only with one change per
    binary (RC-4's A/B/C table is the shape for the follow-up); and this PR's
    regression test drives the rollback *through* the bad split, which the
    clamp turns into a `validate_allocation` panic before payment. The
    follow-up replaces that test with a prompt-shape one and leaves the
    rollback as the CR 601.2 backstop the fuzz guard watches. `backlog.md`
    §2.18 owns it beside the CR 732.1 reversal; note that after this PR the
    random agent's rewind rate is §2.18's ~7.5 per game *plus* these ~5.

    **A second route to the same hole, unreachable today:** "Before card
    breadth" item 9 — `can_pay_costs` checks each `Cost::Mana` entry against
    the whole pool, so a kicker's mana is double-counted and `pay_costs` fails
    with the base already spent. The rollback returns the card; it does not
    return the mana.

### Found by the Everywhere pool change (2026-09-03)

16d. **The random agent cannot use an any-colour mana base, and the harness
    now measures that in every game.** `--require Cytoshape` without colour
    seeding resolves **212** times in 200 `performance` games (126 games,
    63%) against **390** with it — and every deck can pay `{1}{G}{U}`, since
    fourteen or more of its 24 lands are Everywhere. The agent picks a (land,
    ability) pair uniformly in the 601.2g window, so an Everywhere tap is a
    five-sided die: three taps hold a G and a U about one time in five, five
    taps two in five. A failed payment rewinds silently with the lands still
    tapped (CR 732.1's "may not reverse" branch, `backlog.md` §2.18), so the
    turn's mana is gone. Land taps per spell cast: 3.86 on `main`, 7.66
    shipped (40-game `performance` dumps). Spells cast per game fell with it
    in the plain run, 27.9 → 22.6 on `performance`, and games run longer
    (33.3 → 37.4 turns). **This is §2.18's auto-payment oracle, measured from the deck's
    side rather than the agent's** — the mana base a Commander deck has, the
    v1 target, and an agent that turns most of it into nothing. §2.18 already
    sizes the oracle; this entry is the number that says what it is worth.
    Not chased here: the fix is agent-side and moves game content a third
    time, and this PR already moves it once.

    **What the pool change did to the fixtures** is in
    `engineering-practices.md` §3, separated into the registration (nothing:
    the middle arm reproduces `main` byte for byte on `performance`) and the
    deck construction (everything else). The counters: games four to seven turns longer, spells per game 27.9 → 22.6 / 24.9 → 22.6, creatures died 7.8 → 5.6 / 4.6 → 3.8, walks +0.7% / +24%. Per-walk time is +34% / +39%, and a fourth binary with the Everywhere fill tier swapped for basics reads *below* `main` (0.977 against 1.060 ms per 1,000 walks) — so the cost is the land itself: a five-ability, five-subtype object is heavier to seed a frame from, and the mana window walks lands more than anything else. Not an engine change; item 7's memoization in another guise.

### Found by the #62 pre-merge pass (2026-08-30)

17. **Nothing expires a registry row with a source-scoped duration (found
    2026-08-30, closing `rb-review.md` H2).** `cleanup_zone_state` used to call
    `replacement_effects.remove_by_source` when a permanent left the
    battlefield, citing CR 611.2a for the opposite of what it says: every row in
    that registry was made by a *resolution*, and 611.2a gives those the
    duration the spell or ability stated. The call is gone. What goes with it:
    `RegisteredReplacementEffect.duration` can hold `WhileSourceOnBattlefield`,
    `WhileEnchanted` and `WhileEquipped`, and no hook removes any of them —
    `remove_expired_at_cleanup` handles `UntilEndOfTurn` and
    `remove_expired_at_turn_start` handles `UntilYourNextTurn`. Unreachable
    today: `Primitive::Regenerate` is the only producer and it makes
    `UntilEndOfTurn` rows. **Sized:** one `retain_effects` closure keyed on
    duration *and* source, called from `cleanup_zone_state` — the right cite is
    CR 611.2b ("for as long as . . ."), not 611.2a. Owed by the first
    resolution-created replacement whose text is "for as long as [this
    permanent] is on the battlefield".

### Found by the theme C+E pass (2026-08-30)

18. **Combat already has a CR 506.4 implementation, and it arrived in a
    replacement PR (`rb-review.md` E2).** `GameState::remove_from_combat` is
    combat code that RB needed for CR 701.19a's regeneration rider — it untaps
    and removes the regenerating creature from combat — and it does the job
    properly in both directions: an attacker leaving combat also stops being
    *blocked by* its blockers, and a blocker leaving stops appearing in the
    attackers' `blocked_by` lists, with CR 506.4b's "remains blocked even if all
    creatures blocking it are removed" deliberately preserved. **Recorded so the
    combat phase uses it rather than writing a second one**; a one-line
    `entry.attacking = None` is the version that looks right and is not. No
    action owed before then. (It is also the sizing lesson: a phase split on
    subsystem lines would have kept it out, which is `engineering-practices.md`
    §4's rule and why RB ran to +5,475.)

19. **`zone_change_epoch` has one consumer and two counters behind it
    (`rb-review.md` E3).** `GameObject.zone_change_epoch` plus
    `GameState::next_zone_change_epoch` and the SBA-check tick exist for
    CR 704.6d alone — "was put into that zone since the last time state-based
    actions were checked", which nothing about an object answers a moment later.
    Recording an unrecoverable fact is the right call and item 10 wants the same
    tick for CR 400.7, so this is not debt to pay down; it is a **re-confirm
    when item 10 lands**. If 400.7 arrives and does *not* use the field, three
    pieces of plumbing serve one rule and the question becomes live again.

20. **The CR 514.2 cleanup damage wipe has no enforcement point, and seven cards
    want one (found 2026-08-30, answering `rb-review.md` E4).** `turns.rs:133`
    zeroes `damage_marked` on every battlefield entry unconditionally. Ancient
    Adamantoise, Case of the Market Melee, Melt Through, Patient Zero,
    Switchgrass Grazer, Uthgardt Fury and Victory of the Pyrohammer all say
    damage isn't removed during cleanup steps (Scryfall
    `o:/damage isn.t removed/ -is:funny`, 2026-08-30). **Not this pipeline's
    problem:** CR 514.2 is a turn-based action that "doesn't use the stack", and
    the restrictions are per-permanent and filtered ("from creatures your
    opponents control"), so the wipe needs to consult a predicate — the sixth
    enforcement point in `cant-effects-architecture.md` §3 — rather than propose
    a `GameAction`. **Sized:** one filter on one loop, once `RestrictionDef` and
    its sweep exist (RS-1). Until then the seven cards are unauthorable, which
    is the right state; what was wrong was `Primitive::RemoveAllDamage`'s
    comment citing this wipe as an *unrestricted* precedent for its own direct
    write. → `replacement-architecture.md` §8a.

    Also from that check, and owed to the census rather than to this section:
    `cant-census.py` queries `o:"can't"`, so restrictions phrased "isn't" or
    "doesn't" are outside its 2,034 clauses — including **248 cards printing
    "doesn't untap during ..."**. `cant-effects-architecture.md` §2.2 is where
    that belongs.

### Found by the theme H pass (2026-08-30)

21. **CR 903.9b's "its owner's hand or library" needs no check, and the one it
    had was a tautology (found 2026-08-30, closing `rb-review.md` H3).**
    `commander_zone_replacement` guarded on
    `obj.owner != owner_of_destination(game, object)`, and that helper ignored
    the action and returned `game.objects[object].owner` — the same field, so
    the comparison was always false. Its comment promised it would "become the
    check that stops 903.9b firing" once an effect put a card into a different
    player's library. **It cannot, and no such effect can exist:** CR 400.3 —
    "if an object would go to any library, graveyard, or hand other than its
    owner's, it goes to its owner's corresponding zone". A hand or library
    destination *is* the owner's, by rule, which is also why
    `add_to_zone_collection` files by `obj.owner` and why
    `GameAction::ZoneChange` carries no destination player. Guard and helper are
    deleted and the CR 400.3 argument is in the function — **recorded here so
    nobody re-adds the guard**, which is the same job H2's inverted test does.
    Nothing owed.

### Found by the theme D+F+H pass (2026-08-30)

22. **`gather`'s counter fast path is a full battlefield scan, and it stays one
    until CR 704.5q joins the chokepoint (`rb-review.md` D2).**
    `any_replacement_counter` walks every `BattlefieldEntity` on **every
    proposed action**, checking three counter kinds. Cheap at 15 permanents and
    it has never shown up in a profile, but it is the one part of the fast path
    that is not O(1). The review's correction is right and was recorded in the
    code: the old comment defended the scan by saying a cache would drift,
    which is true of a *count* and false of a **set** maintained the way
    `replacement_ability_sources` is. **What actually blocks the set is that
    counters have three mutation sites, not two** —
    `GameState::add_counters`, `perform_action`'s `RemoveCounters` arm, and CR
    704.5q's +1/+1 / -1/-1 annihilation, which writes `BattlefieldEntity`
    directly (item 6). A set maintained at a chokepoint that does not exist is
    exactly the drift, and it reads as a card silently doing nothing.
    **Sized:** a `HashSet<ObjectId>` beside `replacement_ability_sources`,
    inserted in `add_counters` and re-checked on removal — about 15 lines, and
    it becomes *sound* the day item 6 routes 704.5q through `execute_actions`.
    **Measure first** (`fuzz_games --games 200 --seed 12345` against the
    13.04 ms/game RB baseline); do not pay for it before the pool grows.

23. **Counters cannot exist on an object that is not on the battlefield, and
    three separate rules want them to (`rb-review.md` F1).** The map lives on
    `BattlefieldEntity` and `perform_action`'s `AddCounters` arm errors for
    anything else. CR 122.1a and 122.1b are both written for "a card in a zone
    other than the battlefield"; **71 suspend cards** (CR 702.62) put time
    counters on a card in exile; and CR 122.2's exception — "counters remain on
    this as it moves to any zone other than a player's hand or library" — is
    printed on two cards, Skullbriar, the Walking Grave and Me, the Immortal.
    **Sized:** move `counters: HashMap<CounterType, CounterStack>` from
    `BattlefieldEntity` to `GameObject`. 12 direct `.counters` sites outside
    `src/cards`, plus the `add_counters` / `remove_counters` / `counter_count`
    accessors — contained. The part that needs thought is
    `CounterStack.timestamp`: CR 613.7c timestamps a counter as it is put on,
    and a timestamp only means something where the layer system reads it, so
    the off-battlefield case needs an answer rather than a copy of the
    on-battlefield one. Skullbriar then costs a predicate at `move_object`'s
    clear point. **Trigger:** the first suspend card, or the first CR 122.1b
    keyword counter on a card outside the battlefield. Not Skullbriar alone —
    two cards do not earn the move. → `replacement-architecture.md` §11 item 10.

24. **`ReplacementDef.then` is rich enough; the `Effect` tree is not
    (`rb-review.md` F4).** A rider like "you may put a +1/+1 counter on each of
    them. If you don't, draw a card" needs `Effect::Optional`, which is
    unimplemented, plus a conditional on that optional's *outcome*, for which
    there is no vocabulary at all. **Recorded here so it is not mistaken for a
    replacement-vocabulary gap:** `then` is an `Effect`, which is §3.2's whole
    point — per-mechanic variety goes in the existing tree — so nothing in
    `types/replacement.rs` changes when this lands. The demand is not
    replacement-shaped either: **1,249 cards print "if you do" and 165 print
    "if you don't"**, against 35 that print both "if you do" and "instead"
    (Scryfall 2026-08-30). Owed by the `Effect` tree, on behalf of the whole
    card pool, and `replacement-architecture.md` §11 item 11 records the
    matching authoring rule — an "if you do" written as a bare unconditional
    `then` is a card bug, not a modelling choice.

25. **Phase 1 runs each batch member's CR 616.1f loop to completion, so
    CR 101.4d's restart cannot happen (`rb-review.md` F6).**
    `execute_batch_inner` decides members in APNAP order, one whole loop at a
    time. CR 101.4d's case — a later player's choice forcing an earlier player
    to choose again — needs the members *interleaved*, one 616.1 choice each in
    turn. Not the same as the Notion-Thief-vs-Notion-Thief case, which **is**
    handled: `chooser_for` is recomputed every iteration, so a rewrite that
    changes who chooses is picked up. **Sized:** turning phase 1 inside out —
    a per-member loop state machine driven by an outer APNAP round-robin,
    which is a rewrite of `apply_replacements`'s signature rather than a patch
    to it. Unreachable until two members of one batch can affect each other's
    choosers, which needs a `Retarget` rewrite (Phase RD) at minimum. Revisit
    with RD, not before.

26. **Batch phase 2 does not re-check member legality, and CR 608.2b says it
    should (`rb-review.md` H7).** If a batch carries two members naming one
    object — two `ZoneChange`s, or a `Destroy` and a `ZoneChange` — the first
    perform moves it and the second returns `Err` ("proposed Battlefield→X,
    which is in Graveyard"), which fails the whole batch where CR 608.2b's
    do-as-much-as-possible says to skip that member. Unreachable today: the SBA
    sweep dedupes per object, combat and untap batches are per-object unique,
    and no registered spell proposes one object twice. It becomes reachable
    with the first multi-clause spell ("Destroy target creature. Destroy target
    creature.") or the first replacement that redirects member 1 into member
    2's precondition. **Sized:** a `still_legal(&GameAction) -> bool` predicate,
    one arm per `GameAction` variant (10 today), asked in phase 2's loop before
    each perform, skipping rather than erroring. It is the *caller's* job by
    `CLAUDE.md`'s own rule — performers are loud, callers check legality — and
    the batch is the caller. What it must not become is a `match` on error
    strings, which would make a real bug indistinguishable from a stale member.

### Found by a rider read-through (2026-08-30)

Three items in the seam between `ReplacementDef.then` and `engine::resolve`.
None is reachable today and none is a defect in what RB shipped; each is a place
where the *next* phase will find a channel missing rather than wrong. §4.1a
settled *when* a rider runs — these are about *what it can reach*, which that
section never asked.

27. **A rider cannot name the affected player, because `Rider` flattens the
    subject to an object.** `subject_object` (`engine/replacement/pipeline.rs`)
    maps `EventSubject::Player(_)` to `None`, and `resolve_rider`
    (`engine/actions.rs`) then builds a `ResolutionContext` with empty
    `targets`. So for a rider on `DrawCard`, `GainLife`, `LoseLife` or damage
    dealt to a **player**, the only recipient that resolves is
    `EffectRecipient::Controller` — the *effect's* controller, which is in
    general a different player from the one the event was about.

    **Accidentally correct on the one card that exercises it.** Notion Thief's
    rider is "you draw a card", and "you" *is* the effect's controller (CR
    109.5). A rider phrased "… instead that player loses 1 life" has no channel
    at all, and would quietly act for the wrong player rather than erroring —
    which is the shape that reads as a card doing something subtly wrong.

    **Sized: two call sites.** Carry `EventSubject` verbatim on `Rider` instead
    of `Option<ObjectId>`, and emit `ResolvedTarget::Player(pid)` for the player
    case in `resolve_rider`. `ResolvedTarget` already has the variant and
    `resolve_player_for_self` already reads it, so nothing new is invented.
    **Trigger:** RD — prevention riders are the first that ride routinely on
    player-subject events. → `replacement-architecture.md` §11 item 16.

28. **A rider reaches exactly one object — its subject — because
    `resolve_primitive` ignores the `EffectRecipient` for everything that
    affects an object.** Of its 29 arms, **24 reach objects only through
    `ctx.targets`** (directly, via `collect_battlefield_targets` /
    `collect_permanent_or_spell_targets`, or via
    `register_resolution_ability_effect`), **4 are player-directed** through
    `resolve_player_for_self`, and one — `ProduceMana` — is neither. The
    recipient's `SelectionFilter` and `TargetCount` are read at *cast* time and
    nowhere else: `EffectRecipient::Choose` has no generic resolution-time
    enumeration path (the only such call in the tree is the Aura-host special
    case in `resolve.rs`), and `FilteredPermanents`'s own doc scopes it to ETB
    registration.

    For a rider that means an `Effect::Sequence` can do several *things*, but
    only ever to the one object the event was about. A rider phrased over a set
    — item 24's "put a +1/+1 counter on each of them" — cannot be written at
    all. **The visible symptom is inert data:** `regeneration_rider`'s
    `Target(Permanent(All), Exactly(1))` filter and count satisfy the type and
    are read by nothing.

    **Not replacement-shaped, and recorded here so it is not mistaken for one.**
    This is owed by the `Effect` tree on behalf of the whole card pool, exactly
    as item 24's "if you do" is, and nothing in `types/replacement.rs` changes
    when it lands. **Deliberately unsized:** `ask_select_recipients` and
    `enumerate_legal_selections` both exist already, so the number that decides
    the phase is how many of the 24 arms want per-arm treatment rather than a
    shared helper. Count that before committing to one.
    → `replacement-architecture.md` §11 item 17.

29. **`apply_replacements`'s `inherited` applied-set is empty at its only call
    site, so §3.2d's lineage rule ships with no producer.**
    `execute_batch_inner` builds `let inherited = HashSet::new()` and hands the
    same empty set to every batch member. The parameter is CR 614.5's
    *termination* argument, not a nicety: a **decomposed** event continues its
    parent's applied set, a **contained** event of a different kind starts a
    fresh one, and without inheritance on decomposition Teferi's Ageless Insight
    re-applies to its own output and the game **hangs** rather than answering
    wrongly.

    **Correct today because nothing decomposes.** `Rewrite` is 1→1 by §3.2d, and
    every nested call RB makes is containment, which wants the fresh set it
    already gets. Decomposition lives in a *performer*, never in a rewrite:
    CR 121.2 carries out "draw N" as N individual draws, and
    `GameAction::DrawCard { player }` has no count field yet.

    **Trigger: RE**, the draw replacement — *not* RD, whose CR 120.3
    results-of-damage split is containment and is already right. **Sized: one
    call site**, in the first performer that decomposes; the parameter, the
    clone and the doc comment all exist, so nothing is re-threaded. §3.2d
    already names the regression this owes —
    `test_two_teferis_draw_four_not_infinity` — and notes it needs a bounded
    iteration guard, because it hangs rather than fails if the rule is wrong.
    → `replacement-architecture.md` §11 item 18.

30. **Nothing records what was spent to pay a cost, and five rules want it
    (found 2026-08-31 by the type-surface audit; `cr-coverage-audit.md` §5.1).**
    `StackEntry.chosen_alternative_cost` and `additional_costs_paid` hold the
    cost *definitions* — which alternative was chosen, which additional costs
    were paid — and both are written at cast and read by no production code.
    Neither says **which mana** or **which objects** were spent, and payment
    destroys both: the mana leaves the pool, and a sacrificed permanent is in a
    graveyard by the time the spell resolves.

    **A fact rather than a feature, because separate future phases each encode
    its absence — but not the phase a first draft of this entry named.** The
    corpus scheduled the readers years ago and named this dependency while
    doing it: ATOM-702.44a-001 (sunburst, CR 702.44) is ticketed *"DEFERRED —
    Phase 8. Requires mana-color-spent tracking."* **RC does not read this.**
    The order is **CV** (CR 707.10, atoms under D5) → **item 6** (CR 700.14's
    "expend N", mana spent to cast spells this turn) → **Phase 8** (sunburst).
    CR 400.7d is the general form — "an ability of a permanent can reference …
    what costs were paid to cast that spell or what mana was spent to pay those
    costs" — and CR 107.4h's snow `{S}` is a fourth reader.

    **CR 707.10 splits the fact in half, and that half is the design
    constraint.** A copy of a spell inherits the *objects* used to pay the
    original's costs — the Fling case the CR names — but **not** the mana,
    "because mana isn't an object" (the Dawnglow Infusion example). Both halves
    are already atoms: ATOM-707.10-002 and ATOM-707.10-003. A copy spine that
    treats cost-payment provenance as one undifferentiated blob gets one of the
    two wrong. `ManaPool.last_spent_grants` is a partial, transient version of
    the mana half already — grants from atoms spent in the *last*
    `pay_with_plan` call, drained after one read.

    **Sized: the `x_value` rail.** A cast-time value carried from `StackEntry`
    to `BattlefieldEntity` on resolution is a shape this codebase already has,
    so this rides it rather than re-threading anything — and the rail survives
    RC-2's ETB rewrite either way, which is why RC is not the gate. The part
    that needs thought is what a spent *object* is once it has left: Fling
    reads the sacrificed creature's power out of a graveyard, which is LKI, and
    LKI formalization belongs to item 6. **Trigger: capture is independent of
    every scheduled phase and cheap whenever; the design constraint lands at
    CV.** Recording it early is the cheap half — the mana atoms exist at
    payment time and nowhere afterward.
    → `cr-coverage-audit.md` §5.1.

31. **`StackEntry.chosen_modes` is dead scaffolding — no writer, no reader
    (found 2026-08-31 by the D3b slice-1 triage).** Declared at
    `state/game_state.rs:33` as `Vec<usize>`, constructed `Vec::new()` at all
    twelve sites (`cast.rs` ×2, `stack.rs` ×5, `game_state.rs`, `ui/display.rs`
    ×3, one test), and **read nowhere in the tree.**

    Exactly the shape of `CardData.color_indicator` in `cr-coverage-audit.md`
    §5.2: a field that represents its fact correctly, with nothing on either
    side of it. So this is **debt, not a fact** — the type is already right, and
    whoever builds modal spells inherits it rather than designing it.

    **Nothing chooses a mode, so nothing can be modal**, which is why CR 700.2's
    seven atoms sit under a shipped phase with no test. The reader arrives with
    that work; CR 700.2h's per-mode additional costs and 700.2c's
    mode-conditional targeting both reach into the cost pipeline, so it wants
    doing near it. → `backlog.md` §2.7.

32. **`DeckLimits` is configured and never consulted (found 2026-08-31 by the
    D3b slice-2 triage).** `GameConfig` carries `min_deck_size`,
    `max_deck_size`, `max_copies` and `sideboard_size`, and both `standard()`
    and `limited()` set them correctly — 60/4/15 and 40/none/none. **Nothing
    in the tree reads them.** One validation function against a `Decklist`
    closes it; until then a malformed decklist starts a game.

    Third instance of the same shape in one triage, after item 31 and
    `color_indicator`: **configuration or a field that looks like progress in a
    grep and has no consumer.** Worth naming as a class — it is what a
    type-surface audit finds easily and a test suite never does.
    → `backlog.md` §2.13.

33. **The CR 106.6 subsystem has no production consumer on the payment side
    (found 2026-08-31 by a card-population probe — `o:"this mana"`, 227 cards —
    during the roadmap review).** T12b built non-fungible mana whole:
    `ManaRestriction` (Cavern of Souls' chosen-type variant included),
    `ManaGrant`, `ManaPersistence`, the `special` sidecar, `SpendContext` and
    `pay_with_plan`, unit-tested in `types/mana.rs`. Production never touches
    it: `pay_single_cost`'s Mana arm routes `ManaPool::pay` /
    `pay_specific_only`, which read the simple pool only; nothing builds a
    `SpendContext`; `drain_spent_grants` has no caller; and every
    `Primitive::ProduceMana` site passes `special: vec![]`.

    **Safely dormant, verified at both ends**: no restricted atom can enter a
    real game's pool, and one could not be spent — not misspent — if it did.
    The emptying half *is* wired (`empty_with_reason` at step transitions),
    minus the blanket-persistence `TODO(T12c)` the detector section below
    already tracks.

    Fourth instance of the item-31/32 class, and much the largest — a whole
    unit-tested subsystem that looks finished in a grep. The wiring is
    ticketed in the ledger: **T12c**, restricted mana in the casting pipeline;
    **T12d**, the cards and the grants (Cavern of Souls, Boseiju). Grant
    delivery to the cast spell rides item 30's `StackEntry` rail, so the two
    want doing together. **Trigger: T12d's cards, or item 30's capture PR,
    whichever comes first.**
    → `plans/cards-unlocked-ledger.md` T12 rows; `roadmap-v2.md` §4;
    `cr-coverage-audit.md` §4's `ManaPool` row.

### Found by the RS-0 refactor (2026-08-31)

34. **§9 finding 7's abort condition did not trigger, and the reason is worth
    keeping.** `cant-effects-architecture.md` §9 finding 7 said to stop and keep
    three registries if composing them meant the wrapper leaking the generic's
    internals, naming "`effects_in_layer`'s layer-indexed cache" as the thing
    that might. **There is no cache.** `effects_in_layer` is two
    `partition_point` calls over a `Vec` that `add` *maintains* in
    `(layer, timestamp, id)` order — an ordering invariant, not a memo, and it
    has exactly one production caller (`compute.rs:242`).

    A second correction, found in review: the shared rules content is **CR 514.2
    alone**, reaching both registries through CR 611.2a. An earlier draft of the
    module comment said "CR 514.2 and CR 613.7" — but 613.7 is timestamp
    ordering, the one axis the registries do *not* share, and `SortKey` exists
    precisely to let them differ on it. The durable measure of what the generic
    bought is greppable: the engine now dispatches on a `Duration` variant in
    **two** places, both in `duration_registry.rs`, and `Duration` is a closed
    enum item 14 is scheduled to grow.

    That distinction is what let composition work. A memo would have had to live
    on one side of the boundary and be invalidated from the other; an ordering
    invariant can simply *move inside* the generic. `DurationRow::SortKey` is
    where it went: the row type declares its storage key — `(Layer, Timestamp)`
    for CR 613.7, `()` for CR 616.1's registration order — `DurationRegistry::add`
    places by `(sort_key, id)`, and the ascending never-reused id makes the
    unkeyed case degenerate to a push. One `add` and one `is_sorted` serve both
    registries, and the wrapper only ever reads `as_slice()` to binary-search
    what the generic already ordered.

    **The premise checked before building on it, since the finding rested on
    it:** `remove_expired_at_turn_start` diffs to *nothing* between the two
    registries once the row type is renamed, and `remove_expired_at_cleanup`
    differs only in the wording of the comment that says it matches its twin.

    **What did not move into the generic, and should not.** The CR 613.6 summary
    flags and the layer slice, both `ContinuousEffectRegistry`'s. Every
    `ContinuousEffectRegistry` mutation funnels through a private `mutating()`
    so a method added later cannot skip the summary rebuild.

    **`ReplacementEffectRegistry` is a type alias, not a wrapper** (revised in
    review, 2026-08-31). Finding 7 predicted a struct keeping "`Uses::Once`
    removal, gather-order iteration"; both turned out to *be* generic methods —
    `remove(id)` and `iter()` — so all nine of its methods were one-line
    delegations. A wrapper that adds nothing costs a hop at every call site and
    makes the reader ask what it is for, which is exactly what happened in
    review. It can grow a method later without becoming a struct again: an
    inherent `impl DurationRegistry<RegisteredReplacementEffect>` is legal
    because the generic is crate-local. **The general rule: compose where the
    wrapper has its own surface, alias where it does not.**

    **Owed by the next customer, not by RS-0.** Item 17's source-scoped expiry
    hook is now a one-line `retain` closure on the generic that both registries
    inherit the day it is written — it was two closures before. And the third
    customer the finding predicted, delayed triggers (CR 603.7), needs a
    `DurationRow` impl and nothing else.

### Found by the fuzz-tail pass (2026-08-31)

35. **CR 616.1's multi-candidate branch had never been reachable in a fuzz game
    — ✅ CLOSED 2026-08-31, two cards later.** Measured: exactly one registered card
    produces a replacement effect — Kalitas, in `phase_rb_cards.rs`. It is
    Legendary and CR 704.5j is enforced (`sba.rs:287`), so no player controls
    two; and two opposing copies each apply only to the *other* player's
    creatures, so any single death still has exactly one candidate. CR 616.1
    engages only among "two or more", so the ordering choice, the applied set
    *across instances*, and the APNAP ordering among simultaneous choosers are
    all structurally unreachable *from the pool*. RB's status line confirms it
    from the other side: "zero new `DecisionProvider` prompts appeared."

    **The atom was not uncovered, which is the sharp part.** `ATOM-616.1-001`
    had a passing test throughout, built on `graveyard_probe` — a fixture
    defined in `phase_rb_integration_test.rs` and registered in no pool. A
    bespoke fixture can cover an atom while the registered pool cannot build the
    same scenario, so **`specdb` coverage and fuzz reachability are different
    measurements and neither implies the other.** → `engineering-practices.md`
    §3.3, which generalises both halves into a rule.

    **Sized as two cards and zero engine change; the second half of that was
    wrong.** Both are non-legendary enchantments whose battlefield-side
    replacement is the shape Kalitas already proves — `EventPattern::ZoneChange` + `AffectedSet::Filter` +
    `Rewrite::Instead(ZoneChangeTo { Exile })`. Text verified on Scryfall
    2026-08-31:

    - **Rest in Peace** `{1}{W}` — "If a card or token would be put into a
      graveyard from anywhere, exile it instead." Global, so it competes with
      both of the others. Its ETB "exile all graveyards" is a triggered ability
      and is deferred, the way Blood Moon shipped while Aura casting did not.
    - **Leyline of the Void** `{2}{B}{B}` — "If a card would be put into an
      opponent's graveyard from anywhere, exile it instead." Opponent-scoped, so
      the pair disagrees about *which player chooses* — which is the half of
      CR 616.1 a same-scope pair would not exercise. Its opening-hand clause is a
      static ability functioning in another zone and is deferred on the same
      terms as `replacement-architecture.md` §3.3's source 2.

    With Kalitas that is **three** sources that can apply to one event (an
    opponent's nontoken creature dying), reaching CR 616.1 at both two and three
    candidates. Non-legendary, so even one card in a deck reaches ≥2. Cost
    anchor: `phase_rb_cards.rs` is 195 lines for one card including its doc
    block, so this is a small PR, not a phase.

    **What shipped, 2026-08-31.** Both cards, registered, plus five integration
    tests. CR 616.1 now prompts with two *printed* cards, and the choice is
    observable rather than notional: both effects exile, but Kalitas carries a
    CR 615.5 rider, so picking it makes a Zombie and picking Rest in Peace does
    not. Whichever applies first, the other stops matching.

    **The sizing was wrong about "zero engine change", and the reason is worth
    keeping.** Leyline says "an opponent's **graveyard**", and CR 400.3 sends a
    card to its *owner's* graveyard — so the clause is about ownership, not
    control, and `PermanentFilter` had only `ByController`. The two answers
    diverge whenever control has moved, which the registered pool can already
    reach: Act of Treason steals a creature, it dies, and it goes to the
    graveyard of the player who owns it. Shipping it as `ByController` would
    have been a *different card*, not a narrower one. `PermanentFilter::ByOwner`
    is new — one variant and two match arms (`compute.rs`, `targeting.rs`), with
    ownership read off the `GameObject` for the reason `Token` gives.

    **"From anywhere" shipped literal, after a correction in review.** Both cards
    were first written `from: Battlefield`, on the argument that
    `AffectedSet::Filter` carries a `PermanentFilter` and a card on the stack is
    not a permanent. **That argument was wrong about these two cards.** Rest in
    Peace's filter is `All`, which reads nothing; Leyline's is `Not(Token)` and
    `ByOwner`, which read `GameObject.is_token` and `GameObject.owner` — present
    in every zone and not characteristics the layer system computes. So neither
    needs the object to be a permanent, and `from: None` costs nothing.

    It is reachable and load-bearing rather than theoretical: stack→graveyard
    happens on every resolved instant or sorcery (CR 608.2n) and every fizzle
    (CR 608.3), and hand→graveyard on the CR 514.1 cleanup discard. Milling is
    the third and `Primitive::Mill` is unimplemented, so the library case is out
    of reach rather than out of scope. The narrowed version shipped a card that
    did not do what it says — a resolving Lightning Bolt went to the graveyard.

    **Kalitas stays `from: Battlefield`, and that is a different call**: CR 700.4
    *defines* "dies" as "put into a graveyard from the battlefield", so the
    clause is its text rather than a limit. What the filter language still cannot
    do is describe a card by a **characteristic** off the battlefield — "if a red
    card would be put into a graveyard" needs the hidden-zone work — and no card
    here asks it to.

    **The general lesson.** "Wait for a card that needs it" was the right
    instinct and the wrong diagnosis: the trigger was not a missing card, it was
    a missing *event* already in the tree. Before deferring a widening, check
    which events can reach it — not only which cards.

    **Colour is not derived from mana cost, and that is now guarded rather than
    fixed** (raised in review 2026-08-31). CR 202.2 makes a card's colours the
    colours of its mana cost, and `CardDataBuilder::color()` restates it by hand
    at **52** call sites. Measured: **0 of 58** registered cards disagree, so
    there is no bug today — only redundancy and drift risk.

    Deriving it is one small PR and *not* a deletion: CR 202.2e's colour
    indicator is printed data no mana cost implies (Dryad Arbor is a green land
    with no mana cost, Ancestral Vision is blue with none), so the builder needs
    an override, and CR 702.114a's devoid is a **CDA** that belongs in Layer 5
    rather than in `CardData`. The minimal shape computes `colors` inside
    `build()`, so no reader changes.

    **Deadline: before Phase 8 breadth**, when 52 sites become 500+. The error
    class that will actually bite is **hybrid** — a `{W/U}` card is *both*
    colours (CR 202.2d) and that is what a human writing `.color()` gets wrong.
    The registry has no hybrid card yet. Until then
    `card_pool_lowering_test::test_every_registered_cards_color_matches_its_mana_cost`
    is the guard that makes deferring safe, and it was verified to fail on an
    injected miscolour rather than merely to pass.

    **Two findings from writing them.** A Rest in Peace that is itself dying
    still has its static ability at the instant the event is proposed, so it
    applies to its own death — correct rules, and it cost a test fixture that
    had two applicable effects where it wanted one. And `PlayerRef::Owner`
    resolves differently at the two filter sites: `compute.rs` reads it as the
    *source's* owner (a `FilterPlayers` has the source), `targeting.rs` as the
    tested object's (its signature has only `you`). Pre-existing — `ByController`
    already diverges the same way — and not fixed here, because no card reads
    either spelling and changing it would move `ByController` too.

    **Layer 2 has the same shape of gap and is genuinely blocked.** One producer
    (`SetController`, Act of Treason). A second *shape* would be a static or
    ETB control effect, and every candidate is an Aura (blocked by item 8's
    `enchant_filter` gap) or a triggered ability (CR 603, not started), or wants
    a `WhileSourceTapped` duration that does not exist. Recorded so it is not
    re-derived: **Layer 2's second card is owed by Auras or triggers, not by
    card-writing effort.**

### Found by the RS-1 spine (2026-08-31)

36. **RS-1 is net-*adding*, and `cant-effects-architecture.md` §7 never said
    otherwise — the "net-deleting" reading was a paraphrase drift.** Counted
    before building, because the sizing row was going to be built on:

    | | lines |
    |---|---|
    | The five folded-in mechanisms, together | **73** |
    | §7's own new-code anchors (`replacement_effects.rs` 259 + `gather`'s sweep/gate ~120) | **~379** |
    | What actually shipped in `src/` | **+1,104 / −86 = net +1,018** |

    §7's risk cell reads "it *deletes* two bespoke mechanisms and adds no new
    call site", which is a claim about **mechanisms** and is true: the
    `GameState::cant_be_regenerated` `HashSet`, its hand-rolled `turns.rs` clear,
    and `is_blocked`'s hardcoded indestructible arm are all gone, and no
    enforcement site was added — `pipeline.rs` and `gather.rs` ask a different
    function at the same two places they already asked one. **A phase that
    introduces a type surface, a registry, a sweep, a predicate, a primitive and
    a candidate filter cannot come out negative on lines**, and the doc's own
    anchors already predicted +300. The remaining ~640 over that estimate is
    §4.9's candidate filter (which needed `Primitive::Sacrifice` to have any
    prompt to suppress), two cards, and this project's doc-comment density.

    **The rule worth keeping: size a phase in mechanisms *or* in lines, and say
    which.** "Net-deleting" is unfalsifiable when the unit is left implicit, and
    it survived two document revisions unchallenged because of it.

37. **`Primitive::Sacrifice` had to ship for RS-1's headline to be observable,
    and that was not in the plan.** §4.9 makes "Sigarda produces no prompt" a
    rules requirement (CR 608.2d), and `Primitive::Sacrifice` was a stub — so
    "no prompt" and "no code path" were the same board and no test could tell
    them apart. It shipped narrow: `Primitive::Sacrifice(SelectionFilter)` plus
    `resolve.rs::sacrifice_one_of_choice`, which is Diabolic Edict and nothing
    else.

    **Two filters, because they are two questions.** The `EffectRecipient` names
    who sacrifices (CR 115.1's target — Diabolic Edict's "*target player*"); the
    primitive's own filter names what is sacrificed (CR 701.21a's "its controller
    moves **it**"). A first draft read both off the recipient and would have made
    every edict either sacrifice a player or target a creature.

    **The choosing player is the target, not the caster**, which is why this is
    §4.9's "resolution-time selection path" and not a cast-time one. RS-2 owns
    the cast-time and targeting sites and RS-1 did not touch them.

38. **The keyword-derived restriction sweep is asked only of the event's
    subject, and the `debug_assert` is what keeps that sound.** Indestructible is
    `AffectedSet::SourceOnly`, so the only object whose synthesized restriction
    can match an event about X is X itself — sweeping the battlefield would cost
    one full `compute_characteristics` walk *per permanent per proposed action*,
    where asking the subject costs the one walk `is_blocked` already paid. §3.5's
    commitment 2 says keyword restrictions do not need the gate, and this is why
    they can afford not to.

    **The next keyword restriction that is not `SourceOnly` breaks it silently**,
    which is why `keyword_prohibits` asserts the shape rather than assuming it.
    Hexproof, shroud, menace and intimidate (item 15) are all axis-2 and none of
    them lands here, so the assertion has no near-term customer — it has a
    near-term *reader*, which is the point.

39. **Perf did not move, measured interleaved in one sitting.** Six alternating
    200-game runs at `--seed 12345` on the frozen `PERFORMANCE_POOL`, first pair
    discarded as warm-up: **main 207.1 ms CPU/game, RS-1 205.0** — a 1.0%
    *improvement*, i.e. indistinguishable from zero. Both binaries built from
    the same toolchain minutes apart, main from a throwaway `git worktree`.

    **RB's recorded 13.04 ms/game was not used and could not have been.** This
    machine now produces ~14.6 ms/game on *unmodified main*, which is the same
    machine drift commit `a926627` documented. A stored baseline from another
    day would have reported a 12% regression that does not exist.

    **What this did *not* measure — and the policy change that came out of it.**
    `PERFORMANCE_POOL` contained no restriction source, so every `is_prohibited`
    call in the A/B returned on the gate's **closed** path. That is the case
    that matters for "did RS-1 slow down every board that existed before it",
    and the answer is no. The **open** path — a board that *does* have a
    restriction source, where each proposed action costs a
    `battlefield_ids_ordered` sweep of `get_effective_abilities` — was
    unmeasured, and could not be measured by an A/B against `main`, because
    `main` cannot play the cards that open it.

    **So the pool stopped being frozen** (owner call, 2026-09-01). The freeze
    existed to keep stored numbers comparable, and `CLAUDE.md` already forbids
    comparing a stored *timing* number — an interleaved A/B uses one pool in
    both arms by construction, so stability across months bought it nothing.
    What the freeze cost instead was representativeness: left alone, the pool
    measures a shrinking fraction of the engine, and "flat on the performance
    pool" decays into "flat on the parts that existed in 2026-08". That is the
    same argument the two-pool split already won one level down, where a card
    was kept *out of the registry* to protect a number.

    Sigarda and Diabolic Edict were added, 55 → 57, and
    `engineering-practices.md` §3's table was re-recorded — with `ms/game`
    **removed** from it, since a stored timing figure was never comparable
    across days and someone will always compare a number that is present. Its
    seed-deterministic rows stay, because those are fixtures rather than
    benchmarks and an addition genuinely does invalidate them.

    **And the open path finally has a number: ~1%.** Interleaved 55-pool vs
    57-pool at `--seed 12345`, **same engine binary source**, four rounds after
    a discarded warm-up pair: **187.6 vs 189.6 ms CPU/game**. That is a *card*
    cost rather than an engine one — both arms run identical code, and what
    differs is how often a board has a restriction source at all. It is inside
    the ~2% within-sitting spread §3.2 measures, so the honest statement is
    "the gate holds up on boards that open it", not "it costs 1%".

    Worth noticing on its own: this sitting read 187–193 ms where the sitting
    two hours earlier read 203–210 on the same tree. Third independent
    confirmation, after `a926627` and the RS-1 A/B, that a stored ms figure is
    a measurement of the machine.

### Found by the fork-and-search question (2026-09-01)

40. **The decision-site invariant, and the two things that break it today.**

    > **No decision site may hold state that changes the game's *outcome* and is
    > not in `GameState`.**

    **Why this shape and not "everything in `GameState`".** The absolutist form
    was the framing this arrived in, and it is too strong: it condemns state
    whose loss costs nothing. The test that separates them is *drop it and
    re-derive* — if the game reaches the same outcome, the state is bookkeeping
    and may live on the stack.

    **What the invariant protects.** Forking. To search, a harness clones
    `GameState` at a decision point, plays forward, and comes back to try the
    other option. `GameState` already derives `Clone`; a fork is sound exactly
    when the clone is a complete description of the game at that instant. It is
    also what makes a pending decision *serializable*, which is the same
    property a self-hosted server needs to survive a dropped connection — so
    this is not AI-only spend.

    **Measured, not assumed. What is on the stack at a decision point:**

    | Site | Stack-resident | Outcome-bearing? |
    |---|---|---|
    | `priority.rs` priority loop | `blacklist`, `retries`, `all_candidates` | **No** — drop them and the fork re-offers a cast that fails again. Slower, same game |
    | `cast.rs::run_mana_ability_window` | the `failed` set | **No** — same shape; the mana pool itself is on `GameState` |
    | `cast.rs` 601.2b–d | the in-flight `StackEntry`, pre-push | **Yes** — see below |
    | `apply_replacements` | `applied` / `declined` / `exempt_applied` | **Yes** — see below |

    `priority_player` is already a `GameState` field, which is the fact that
    makes the first row cheap.

    **Violator 1 — `pipeline.rs:101/115/119`, the CR 614.5 sets.** `applied`,
    `declined` and `exempt_applied` live in `apply_replacements`' frame and are
    consulted across the `ask_choose_replacement` / `ask_apply_optional_replacement`
    prompts. Fork at a CR 616.1 prompt and the branches disagree about which
    effects have already applied — so a *different replacement* applies, which
    is a different game. Worse than a wrong answer in one respect: the `declined`
    set is what stops the loop re-offering a declined optional forever, so a
    resumed frame that lost it **hangs**.

    **Violator 2 — `cast.rs`, the CR 601.2 announcement window.** Between the
    card leaving hand and the `StackEntry` being pushed, the proposal (modes,
    targets, X, chosen costs) is a local. The four `rollback_cast_to_hand` call
    sites are the evidence: a rewind is possible precisely because the state is
    not yet committed anywhere a clone would see. This one is a *fact* on the
    triage split — the announcement is CR 601.2's own sequence, and a later
    phase cannot reconstruct what a player chose at 601.2b.

    **Neither is a bug today**, and that is the point of recording them: nothing
    forks. They are the debt a fork-based harness inherits, and the cost grows
    per phase — RS-1 added one decision site (`sacrifice_of_choice`), RS-2 adds
    six by its own sizing row. **The invariant is cheap as a review rule and
    expensive as a migration**, which is why it is written down before there is
    a customer.

41. **A fork at a *priority boundary* is probably sound today, and one test
    would settle it.** Every entry in the table above is unwound at a priority
    pass: the two non-outcome-bearing sets are loop locals that do not survive
    the iteration, and neither cast nor `apply_replacements` is on the stack.
    That matters more than it sounds, because the priority boundary is where
    essentially all the strategic signal in Magic lives — what to cast, what to
    attack with. Search over *those* with a fast policy answering the micro
    choices inside a rollout is how strong engines in adjacent games are built,
    and it needs no resumable engine at all.

    **The test, ~150 lines:** clone `GameState` at every priority pass, run both
    copies to completion under a scripted `DecisionProvider`, assert identical
    event logs. If it passes, forkability exists today and item 40 is purely
    forward-looking. If it fails, the failures *are* the enumerated list of what
    is on the stack that should not be — useful either way, which is what makes
    it worth a day before any refactor is priced.

    **One thing that test will surface, and it should be a decision rather than
    a discovery:** `RandomDecisionProvider` owns its `StdRng`, outside
    `GameState` (deliberately — `CLAUDE.md` names it as one of two opt-outs).
    For search that is arguably correct, since determinization *wants* to
    re-randomize the unseen library per branch. For "replay this game exactly"
    it is wrong. The two forks share nothing and diverge, and which of those is
    intended has never been written down.

42. **`EventLog` is on `GameState` and grows monotonically.** `clear()` is
    documented as between-games only, so a ~33-turn game carries every
    `EventRecord` it ever emitted — and every clone carries them again. The
    trigger matcher already reads a *suffix* (`records_from(index)`), so the
    whole history is not what any consumer wants.

    Bounding the in-state window and streaming the rest to an external sink pays
    three different phases at once: clone cost for search, `serde` size for a
    self-hosted server's wire format, and the per-viewer projection (backlog
    §2.9) that both the GUI and an AI observation need. **Not urgent and not
    hard**; recorded because it is the cheapest of the three, and because "the
    log is unbounded" is the kind of fact that is obvious once and invisible
    afterwards.

43. **CR 122.6a names a player and `EnterMods` does not carry one (recorded
    2026-09-01, RC-2).** "If an object enters the battlefield with counters on
    it, the effect causing the object to be given counters **may specify which
    player puts those counters on it**. If the effect doesn't specify a player,
    the object's controller puts them on." `EnterMods.counters` is
    `Vec<(CounterType, u32)>`, so only the default half exists.

    **Nothing in reach needs the named half** — no registered card specifies a
    player, and ATOM-122.6a-001 is covered by the default. What needs it is
    Phase **RE**: the atom's own expected result says so, because Doubling
    Season doubles counters *you* put on, so a doubler has to know who put them
    on before it can decide whether it applies. The field is one `Option<PlayerId>`
    per entry and the merge already coalesces by kind, which is the thing that
    would have to change — two effects giving counters of the same kind on
    behalf of different players cannot share a row. **Size it before RE writes
    its first doubler, not after.**

44. **`is_prohibited`'s battlefield sweep has the gate defect `gather` just had,
    and it measures flat — which is the finding (recorded 2026-09-01, RC-2
    review).** `engine/restriction/predicate.rs` gates its sweep only on
    `has_static_source`, "is *anything* on this board a static restriction
    source", and then walks `get_effective_abilities` for every permanent. It is
    structurally identical to the sweep in `replacement::gather` whose
    per-permanent gate was worth **10.3%** of total game time — and
    `is_prohibited` is called more often, on every iteration of the CR 616.1 loop
    for every proposed action.

    **Measured anyway, because structure is not exposure: −1.0% on
    `performance` and −1.6% on `stress`**, four interleaved rounds each at 200
    games, both inside the run-to-run spread. Event streams byte-identical. The
    reason is the card, not the code: `restriction_ability_sources` is non-empty
    only while a printed restriction is on the battlefield, and the only one in
    the pool is **Sigarda at {2}{G}{W}{W}** — gold and five mana, so she is in
    one deck in sixteen and lands late when she lands at all. `gather`'s gate
    opened on turn two of most games because RC-2 put a *tapland* in the pool.

    **So this is deferred rather than done, and the trigger is a card, not a
    date.** The fix is five lines and exactly `gather`'s — skip the permanent
    unless `restriction_ability_sources.contains(&id)` or the registry summary
    reports a Layer 6 grant, which is the same predicate one object at a time,
    exact by the same argument and inheriting the same Layer 1 hole (item 16).
    **Do it with the first cheap or colorless restriction card**, which RS-2 and
    RS-3 will both want; until then it is a 10%-shaped cliff nobody is standing
    on. The general lesson is worth more than the item: **a sweep for this defect
    shape would have "fixed" this instance and reported a win it did not earn.**

45. **Engine cost is now a fixture, and it found two determinism violations the
    old rule could not see (added 2026-09-01).** `state/diagnostics.rs` counts
    layer walks, computed frames, replacement gathers and restriction queries per
    game; `fuzz_games` prints them and `engineering-practices.md` §3 stores them
    beside turns-per-game. Overhead A/B'd below the noise floor.

    **Two sites had to be ordered before the numbers held still**, and both were
    correct under CLAUDE.md's determinism rule as written — `combat/steps.rs`'s
    first-strike scan and `targeting.rs`'s `has_any_legal_choice`, each an `any`
    over `battlefield`'s `HashMap` with a layer query inside. `any` over a set is
    order-independent, so the *answer* never varied and neither site ever
    appeared in `determinism_test` or a `--dump-events` diff; what varied was how
    many walks the short circuit took to get there (~14 per 50 games). The rule
    now reads "a choice, log **or count**".

    **The number itself points at work already scheduled.** ~109,000 layer walks
    per game against **669** gathers: the CR 614 pipeline is low single digits of
    engine cost even sweeping the whole board, and the bulk is ordinary oracle
    traffic with no memo between calls. That is `CLAUDE.md` critical-path item
    7's cross-call memoization, which already has a hard back-stop before Phase
    8 — so the instrument's first finding is that **no new performance work is
    warranted**, which is exactly what an instrument is for.

    Two follow-ups it makes cheap rather than urgent: attributing the walks to
    call sites (a profiler's job, deliberately not built here), and deciding
    whether a `GameState` clone for search should inherit the counts or reset
    them — today it inherits, so a branch's cost is a subtraction.

### Found by the look-ahead frame (2026-09-02, RC-4)

46. **The frame is per entry, not per batch, and §5b says it should be per
    batch.** Two Master Biomancers entering as one event should give each other
    nothing — every member's look-ahead reads the pre-batch board. Today an
    entry is proposed *inside* its zone change's performer (RC-2's nested
    `propose_entry`), so the second member of a `[ZoneChange, ZoneChange]`
    batch is decided after the first was performed and sees it on the
    battlefield. **Unreachable rather than wrong**: no caller produces a
    multi-entry batch (`Primitive::ReturnToBattlefield` is a stub;
    `CreateToken` loops `propose_entry`). The fix is structural — decide the
    entry in phase 1 beside its zone change, perform it in phase 2 — and it is
    the same restructuring CR 613.7m needs ("Before card breadth" item 4), so
    the two are scheduled together as RC-5 part 2 (`replacement-architecture.md`
    §9). **Sized:** ~400 additions in `execute_batch_inner` and
    `perform_action`'s `ZoneChange` arm. The entry hop ("Before Triggered
    abilities" item 4) was the same restructuring seen from the log's side,
    and **RC-4b closed it (2026-09-02)**: an entry is a phase-1 proposal that
    carries `from`, so a multi-entry batch would now decide every member
    against the pre-batch board. What is left here is producing one —
    `CreateToken` still loops `propose_entry` one token per batch — and
    CR 613.7m's APNAP timestamps.

47. **`pipeline::order_invariant_entry_bucket` is a semantics-assuming shortcut,
    and these are its expiry conditions.** It skips CR 616.1's prompt when every
    member of the bucket is an `EnterWith` whose applicability no `EnterMods`
    field can move, which is true today because the only characteristic the
    mods feed is power (through `+1/+1` and `-1/-1` counters, CR 122.1a) and
    the only leaf that reads power is `PermanentFilter::PowerLE`, which the
    predicate excludes. It goes false, silently, the day any of these lands:
    (a) `EnterMods` gains a field that feeds a characteristic — **face-down**,
    which is Layer 1 and changes everything (Phase CV); (b) `PermanentFilter`
    gains a leaf that reads power, toughness, keywords or counters
    (`ToughnessLE`, `HasKeyword`, `HasCounter` — RS-2/RS-3 candidates); (c)
    `EventPattern::EnterBattlefield` gains a field that reads `mods`.
    `filter_is_mods_invariant` is matched exhaustively, so (b) is a compile
    error rather than a silent default; (a) and (c) are not, and
    `check_order_invariance` is the debug-build check that computes the theorem
    the other way — re-gather after the suppressed choice and assert the rest
    still apply — which catches either on any board a test or a debug fuzz run
    reaches. **The rule for whoever adds (a) or (c): revisit the predicate in
    the same commit.**

48. **`default_enter_controller` has three roads and answers a fourth wrongly
    — and RC-4 stopped standing on it.** The three that exist are exact: a
    resolving permanent spell (`GameState::resolving`, CR 110.2b's default),
    a land drop (owner), a token (owner, CR 111.2). The fourth is an effect
    putting a card onto the battlefield *under a player's control* who is not
    its owner — Reanimate's "put target creature card from a graveyard onto
    the battlefield under your control" — where owner is wrong and nothing in
    the proposal says otherwise. No registered effect takes that road
    (`Primitive::ReturnToBattlefield` is a stub). **Sized:** the mover has to
    say under whose control — a `controller: Option<PlayerId>` on the
    `ZoneChange` proposal, or a `propose_entry` argument the `Returned` arm
    threads — one field, read in one place. The `base_controller` `resolving`
    leg RC-3 added was re-checked for this phase as the brief asked: it is
    consulted for an entering object only by the finished-board
    `permanent_matches_filter`, which RC-4 no longer uses for an entry (the
    frame seeds its controller from the proposal), so the frame does not stand
    on it and it stays as RC-3 left it — right for the three roads, inert in a
    game.

49. **`is_prohibited` has no source-1a leg.** `gather` asks the entering
    permanent itself for its `SourceOnly` replacement abilities ahead of the
    battlefield sweep, because `replacement_ability_sources` is written by the
    performer; the restriction sweep has no twin, so an entering Tatterkite's
    "this creature can't have counters put on it" (Melira's Keepers has the same
    sentence) is invisible to an entry that would give it counters. CR 614.17d's
    parenthesis licenses the leg. No registered effect gives an entering
    permanent counters *from outside* — Master Biomancer is RC-5's — so there is
    no board to fail on yet; add the leg with the first such card, mirroring
    `gather`'s `SelfScope::EnteringSelf`, ~20 lines.

50. **A count enumerates the battlefield twice per CDA.** `SetPowerToughness`
    evaluates its two amounts separately, so Keldon Warlord's `CountOf` sorts
    `battlefield_ids_ordered` and matches every permanent twice per layer-7a
    application — the second pass hits the frame cache for every frame but
    repeats the sort and the filter walk. Not measured to matter (see the RC-4
    block above); recorded so that whoever sees `Frames/walk` climb on a
    CDA-heavy board knows the factor of two is here and not in the cache.

### Found by the RC-4 review's nesting audit (2026-09-02)

51. **~~A rewound cast leaves its CR 601.2a move in the log.~~ — ✅ CLOSED 2026-09-02 (RC-4b).** `cast_spell`
    moves the card to the stack through `change_zone` with cause `Cast`, which
    emits, and the four rewind sites — 601.2b's cost choices, 601.2c's
    targets, 601.2h's can-pay check — move it back with the silent
    `CAST-ROLLBACK` mover. So a cast that fails leaves
    `ZoneChange { Hand → Stack, Cast }` in the log with no counterpart.
    `SpellCast` is emitted only at 601.2i, so a cast trigger keyed on it is
    safe; a matcher on the zone change is not. Same class as the entry hop
    (item 4 under "Before Triggered abilities"), different fix: nothing in the
    CR replaces a card being put onto the stack, so the 601.2a move is not a
    replaceable event and should use the silent mover in both directions, with
    the `ZoneChange` recorded at 601.2i beside `SpellCast` — the moment CR
    601.2i says the spell becomes cast. Mana abilities activated in 601.2g stay
    performed and stay in the log, which is CR 732.1. No rewind site exists
    after cost payment begins, so nothing paid is ever un-paid. **Bundled
    into RC-4b** (`replacement-architecture.md` §9, design item 7): ~30
    lines in `cast.rs`, and RS-2's Tier 1b/1e exits are more rewind sites,
    so it gets more reachable with time, not less. The rule both fixes
    follow is `replacement-architecture.md` §11 item 20. **Closed as
    designed:** the 601.2a move is `move_object` in both directions, tagged
    `CAST-ROLLBACK`, and `announce_zone_change` records it at 601.2i beside
    `SpellCast`; a rewound cast leaves no `ZoneChange` and keeps its 601.2g
    taps (`phase_rc4b_integration_test`).

### Found by RC-4b — entering is one event (2026-09-02)

52. **A token whose entry is exiled instead records `from: Battlefield`.** A
    token is created in `Zone::Battlefield` with no entity and in no
    collection until its entry is decided (`Primitive::CreateToken`), and its
    `EnterBattlefield` carries `from: None`. `Instead(ZoneChangeTo)` on it is
    performed as `ZoneChange { from: Battlefield, to }` with no LKI, so the
    log says the token left the battlefield where CR 111 says it was created
    in exile (Hallowed Moonlight's ruling); CR 704.5d then removes it. A
    *dropped* token entry un-creates the object (CR 111.5). **Unreachable
    today**: Containment Priest excludes tokens and no registered card is
    Hallowed Moonlight. The honest fix is Phase RE's `CreateTokens` proposal
    (CR 614.16's doublers need it anyway), where the creation is the event and
    the entry's decision sets its destination —
    `replacement-architecture.md` §9, RC-4b's token decision. **Sized:** the
    `CreateTokens` arm of `GameAction` and `EventPattern`, ~150, inside RE.
    **Not optional before Phase 8 (owner, RC-4b review):** Dour Port-Mage
    ("Whenever one or more other creatures you control leave the battlefield
    without dying, draw a card.") and Aang, Airbending Master ("... you get an
    experience counter.") are the matcher that reads this line — a
    leaves-the-battlefield trigger keyed on `ZoneChange { from: Battlefield }`
    — and both would fire for a token that was created in exile and never left
    anything. Cross-listed as "Before card breadth" item 8.

### Was the critical path complete? — audited 2026-08-27

Asked by the owner after the "can't" model turned out to be a whole subsystem
nobody had scheduled. The useful form of the question is not "are we confident"
but **"what shape of thing did we miss, and does anything else have that
shape?"** Both halves are answerable.

**The signature of the miss, in three parts.** "Can't" effects have no CR
section of their own — they are CR 101.2, 613.11, 614.17, 508.1c, 509.1b,
601.3, 602.5, 115.6 and 701.19c, one or two subrules each. The planning docs
mirror CR *sections* (`layers-architecture.md` = 613, `replacement-architecture.md`
= 614–616), so a mechanic spread across nine of them has no natural home and
lands in nobody's doc. Meanwhile **the corpus knew**: `ATOM-614.17a/b/c/d`,
`ATOM-601.3-001`, `ATOM-509.1b-002` all existed. What failed was the *query* —
`specdb owed` filters to `ticket LIKE 'NEW%'`, and those atoms carry `L15` and
`T21b`, so they sat in `owed --all`'s 550 and never in the default 38.

> **The detector:** a CR section with many uncovered atoms that **no
> architecture doc mentions**, or whose atoms are tagged to a phase the plan
> does not schedule. Run it as `owed --all` grouped by rule prefix, cross-checked
> against which of `layers-`/`replacement-`/`cant-effects-`/`copy-effects-architecture.md`
> mentions that section. **Two refinements from the 2026-08-29 run:** `owed --all`
> is not enough on its own — 26 of the copy cluster's atoms carry ticket `D5` and
> the rest are tagged `Phase 9`, so the grouping has to be by *rule prefix*, never
> by ticket. And a headline Scryfall count is a hypothesis: check what it
> includes before it becomes a scoping argument.

**Run against the tree today it finds one more, and it is the same shape.**

| | "Can't" effects | **Copy effects (CR 707 + 712 + 708 + 729 + Layer 1)** |
|---|---|---|
| Spread across | 9 CR subrules | CR 707 (copying), 712 (DFC/meld), 708 (face-down), 729 (merging), 613.2 (Layer 1) |
| Owning doc | none, until 2026-08-27 | none, until **2026-08-29** — `plans/copy-effects-architecture.md` |
| Corpus atoms, uncovered | ~21 | **101**, all uncovered (707: 30, 712: 36, 729: 19, 708: 10, 613.2: 3, 710: 3) |
| Corpus's own phase tag | Phase 6 / 5-Pre | **CR 707: 23 atoms tagged "Phase 6"** — i.e. the corpus files copy effects *with replacement effects*, and RC–RE does not schedule them |
| A shipped enum already waiting for it | `ReplacementClass::SelfReplacement` | **`ReplacementClass::CopyOnEnter` and `BackFaceUp`** — two of RB's five buckets, with no producer |
| On `CLAUDE.md`'s critical path | no (now item 5b) | no — **now item 5c**, phases CV-1–CV-7 |
| Card population | 1,857 printed + keywords | **517 double-faced** (396 transform + 100 modal + 21 meld), 752 cards printing "copy", 304 face-down producers, 34 mutate |

**Three cells of the original table were wrong, and they are corrected above.**
Recorded rather than silently edited, because the detector's *method* is the
thing this section is arguing for and its failure modes are worth knowing.

1. **"706"** is Rolling a Die in `tmnt.txt`. The intended section is **708,
   Face-Down Spells and Permanents** — CR 613.2b's Layer 1b, 10 uncovered atoms.
2. **"2,890 double-faced cards"** is `is:dfc`, and **2,243 of those are
   `layout:art_series`** — art cards with a signature on the back, not playable
   Magic and with no copiable values to model. With `double_faced_token` (80) and
   `reversible_card` (71) also removed, the rules-relevant population is **517**.
   The conclusion survives; the scoping argument does not survive a 5.6×
   overstatement, which is the difference between "schedule it" and "schedule it
   first". `copy-census.py --decompose` prints the breakdown.
3. **CR 729, Merging with Permanents, was missing** and CR 613.2a names it in the
   same breath as CR 707: layer 1a is "copy effects (see rule 707) *and changes
   … determined by merging an object with a permanent (see rule 729)*". 19
   uncovered atoms; 34 mutate cards. **In v1 and scheduled** — phase CV-7, with a
   back-stop before Phase 8 card breadth, because a multi-component permanent is
   a *fact*: every phase built meanwhile writes code against the
   single-component assumption.

**A headline Scryfall count is a hypothesis, not a measurement.** That is the
one lesson to carry to the next detector run: `is:dfc` answers a question about
card *faces*, not about rules the engine must implement, and nothing in the
number says so.

`Layer1Copy` is documented in `engine/layers/types.rs` as "still a stub: the
variant exists, nothing produces an effect in it", and CR 616.1c's bucket ships
in `ReplacementClass` for the same reason CR 614.15's does. **This is not a
finding that anything is broken** — it is a finding that a system with **1,628
cards** behind it had no doc and no critical-path slot, exactly as "can't" did
on 2026-08-26. (752 printing a copy clause, 901 carrying a face or state that is
one, overlapping by 25 — measured, not summed by hand.)

**✅ Closed 2026-08-29 by `plans/copy-effects-architecture.md`** (`CLAUDE.md`
item 5c). The seam question is answered by CR 613.2c rather than chosen: copiable
values are the *output* of layer 1, so CR 707 owns the payload and Layer 1a owns
its application, and the payload is a snapshot (CR 707.2b/2c) stored in one
`EffectModification::CopyFrom` row. Three consequences worth carrying here:

- **RC-4 cannot produce a `CopyOnEnter`** and should not try — it keeps CR
  616.1b (Layer 2 is shipped) and gives 616.1c up to phase CV-2. That makes RC-4
  smaller, not later, and it is why the doc landed before RC-4 rather than after.
- **Copy work is *not* blocked on critical-path item 7**, provided a copy row
  stores values rather than an `ObjectId`. A reference-carrying row would be
  dependent under CR 613.8a(b) *and* would break `layers-architecture.md` §5.2's
  strictly-descending termination argument, since two permanents copying each
  other is a legal board.
- **`register_static_effects` has the same hole `gather`'s gate has** (item 16
  below), found while writing the doc.

**Why late discovery has been cheap so far, and what would make it expensive.**
Both misses are **features** on this file's own fact/feature triage, not facts.
Nothing had to be unbuilt: RB's `is_blocked` was correct-but-narrow and the
restriction model extends it. The expensive kind is a missing *fact*, and the
evidence that those have had deliberate attention is Sigarda — "spells and
abilities your opponents control can't cause you to sacrifice permanents" needs
to know **who caused an event**, and it costs one `Option<SourceFilter>` field
only because Phase RA threaded provenance through `ActionContext` first. Had it
not, that one card would have been a re-thread through every system built since.
**Keep auditing for facts; let features be found late.**

**What this does not claim.** The corpus is ~1,760 atoms and the critical path is
seven items; the plan will always be coarser than the rules, and a third
`can't`-shaped gap is likely rather than unlikely. The claim is narrower and
checkable: the detector above is cheap, it has found two, and running it at the
start of each phase is a better instrument than confidence.

**Update 2026-08-31 — it has now found six, and the method has its own
document.** Asked whether the critical path covers everything in the CR, a run
of the detector turned up three in one sitting: **cost modification** (~903
cards; `replacement-architecture.md` §9 already called it homeless and "not
small", and `apply_cost_modifications` is a passthrough stub with a test
asserting so), **casting from a zone other than hand** (~764 cards;
`check_cast_legality` hard-codes `Zone::Hand`; **CR 607 linked abilities was
bundled in here and is a separate mechanic** — no CR 601 atom concerns a non-hand
zone, see `backlog.md` §2.2/§2.3), and **voting** (CR
701.38 — zero atoms in a 1,753-atom corpus, though ~~never examined~~ DEFERRED
in session 7A since 2026-04-07; zero *atoms* is the half that was true).

`plans/cr-coverage-audit.md` owns the method. **It changed on 2026-08-31, and
the reason belongs here, because this section is where the detector lives.**
The first plan swept the frozen CR for *dark* rules — ones nobody had examined
— and came back with **zero facts** across 199 families. Measured afterward,
five of the six gaps carry 20+ atoms each: they were examined in 2026-04 and
then **orphaned**, so a darkness filter removes them before the sweep starts.
Only voting was dark.

**Darkness is not ownership**, and the corpus fails at both, in opposite
directions — `audit --dark` catches voting and misses the other five;
`orphaned` catches the five and misses voting, which has no atom to orphan.
Neither query is the instrument. What all six *do* share is that **a type or a
function could not express what the CR requires**, which is this section's own
fact/feature triage — and facts live in types, not in rules. The audit is now a
**type-surface** sweep: for each fact-bearing type, what can the CR require
here that this type cannot represent? It found one fact, Deferred Migrations
item 30 (cost-payment provenance), and it is calibrated against the six above
before it is trusted.

The lesson the failed pass paid for applies to this section's own tooling: **a
number nobody can reproduce by hand is a number nobody has checked.** `audit`
had been counting *unread* verdicts as unexamined rules for as long as it
existed, because `parse_rule_mentions` read one of the three shapes the corpus
writes a verdict in.

### Fuzz-pool coverage — audited 2026-08-26

**The pool is thin exactly where the engine is thin, and card selection cannot
outrun that.** Audited before Phase RB, because RA-3 wrote SBA-batch code whose
branches the harness never reaches. 55 registered cards; every one targets
`TargetCount::Exactly(1)`.

State-based actions and paths with **zero** `fuzz_games` coverage, each with its
actual blocker — none of them is a card-selection problem:

| Never exercised | Blocked by |
|---|---|
| 704.5m/n Aura SBAs; 608.3c Aura ETB attach | `engine/cast.rs` never reads `enchant_filter` (0 references). A cast Aura reaches `resolve_popped` with no targets and the Aura branch errors. **Registering an Aura today would produce fuzz errors**, not coverage. Sibling of item 8 |
| 704.5d token cease-to-exist | `Primitive::CreateToken` is a stub |
| 704.5q counter annihilation | `Primitive::AddCounters` / `RemoveCounters` are stubs |
| 704.5p Equipment detach | needs Equip (CR 702.6) |
| 704.5i planeswalker zero loyalty | loyalty abilities need the special-action path, and CR 120.3c (damage → loyalty) is unimplemented, so a planeswalker can never die |
| Mass removal; multi-member `Destroy` batches | `EffectRecipient::FilteredPermanents` is read only by `register_static_effects` — it is a *static-ability* recipient. At resolution `ctx.targets` is empty, so `Primitive::Destroy` over a filter destroys nothing. Wrath of God cannot be written. The CR 608.2f batch added 2026-08-26 is correct and reachable only from a multi-target spell, of which the pool has none |

**One gap was card-fixable and was closed: indestructible.** `KeywordFlag::Indestructible`
is read in two places — SBA 704.5g and `Primitive::Destroy` — and no card in any
card file carried it, so neither branch was ever taken. **Darksteel Myr** ({3}
Artifact Creature — Myr, 0/1, Indestructible) is registered as of 2026-08-26 and
the SBA branch now fires ~15,000 times per 50-game run. Chosen over other
indestructible creatures for three reasons: RB item 4 *moves* that check to the
CR 614.17 "can't" path and coverage before a move is worth more than after;
Humility strips the keyword, so the pool now carries a Layer 6 effect that
changes an SBA outcome on every run; and it is the registry's second artifact,
which gives March of the Machines a second subject.

**Baseline moved once, deliberately, before RB** (`--games 50 --seed 12345`):
P0 28 (56.0%) / P1 22 (44.0%), spells 20.8, lands 17.9, combat 11.4, creatures
died 5.6, damage events 24.4, total damage 53.6, life changes 18.0. Creature
deaths fell (6.2 → 5.6) and combats rose (9.7 → 11.4), which is what an
unkillable 0/1 blocker does.

**The rule this establishes:** when a mechanic has no fuzz coverage, name the
blocker before reaching for a card. If the blocker is a stubbed primitive or a
missing subsystem, adding a card buys errors rather than coverage.

**Superseded in part 2026-08-29 (RB review, theme G3): there are now two pools.**
The audit above was run against one pool that had to be both the panic hunter and
the A/B baseline, which is why Kalitas was written and left out of the registry —
"it would move the baseline". That is an argument for splitting, not for keeping
cards out. `cards/registry.rs` now builds `default_registry()` (every registered
card, `fuzz_games --pool stress`) and `performance_pool()` (`PERFORMANCE_POOL`,
the default, and what every baseline in this file was measured on — *frozen*
until 2026-09-01, and since then growing one card per new engine path). The paragraph above stays exactly true of the performance pool. Kalitas is
registered into the stress pool only; 50 stress games at seed 12345 give P0 30 /
P1 20, turns 26.8, spells 19.5, lands 17.2, combat 10.7, creatures died 5.2,
damage events 25.0, total damage 54.2, life changes 17.7, zero panics and 202
Zombie tokens. Rules and both baselines: `plans/engineering-practices.md` §3.

### Fuzz-pool coverage — re-audited 2026-09-01

**Two of the six paths the audit above calls unreachable had already closed, and
nobody had re-measured.** That audit is prose, and prose does not expire loudly:
it was written when `Primitive::CreateToken` was a stub and before combat could
produce a batch. The instrument this time is 200 stress games at seed 12345 with
`--dump-events`, counting each path's own event signature in the log, so every
row below is a number someone else can reproduce.

| SBA / path | Signature counted in the dump | Occurrences | Blocker today |
|---|---|---|---|
| 704.5d token cease-to-exist | `TokenCeasedToExist` | **43**, across 23 of 200 games | **none** — RB's Kalitas closed it |
| multi-member `Destroy` batch | ≥2 `[DestroyedBySba]` between two `StateBasedActionPerformed` | **155 batches**, largest 6 members | **none** — ordinary combat produces them |
| 704.5q counter annihilation | `CountersAnnihilated` | **0** → **10** | **none** — a real card gap, closed below |
| 704.5m/n Aura | `[AuraSba]` | **0** | *two* items deep — see below |
| 704.5p Equipment detach | `EquipmentDetached` | **0** | Equip (CR 702.6). `attach_to` has exactly one production caller and it is the Aura path |
| 704.5i planeswalker death | `[ZeroLoyalty]` | **0** | loyalty abilities have no `AbilityType`, and CR 120.3c is unimplemented |

**The two closed rows are the finding, not the two cards.** The old audit's
entry for multi-member `Destroy` was written about *mass removal from a spell* —
Wrath of God, still unwritable, `EffectRecipient::FilteredPermanents` still a
static-ability recipient. But the CR 608.2f batch it worried about is reached
155 times per 200 games from the **SBA sweep**, because two creatures trading in
combat is one `execute_actions` call with two `GameAction::Destroy` members. A
gap named by its cause outlived the cause.

**CR 704.5q closed with one card.** The rule is defined over one permanent
holding two counter kinds, and `chainbreaker` (RC-2, colorless, in every deck)
was already the -1/-1 half; nothing anywhere produced a +1/+1 counter.
`cards/phase_sba_cards.rs::battlegrowth` — `{G}` Instant, "Put a +1/+1 counter
on target creature", the only one of Scryfall's 72 cards with that clause whose
oracle text is *just* that sentence — is the other half. It is also the first
thing in the crate to propose a `GameAction::AddCounters`: Chainbreaker's
counters arrive through `EnterMods` and `GameState::add_counters`, a direct
write inside the entry performer, so `perform_action`'s arm and `gather`'s
`EventSubject::Object` leg for it had no production reach at all. `CountersChanged`
goes **0 → 82** per 200 games with it, which is what says the arm is live rather
than merely correct. New engine path, so it takes the deliberate
`PERFORMANCE_POOL` addition §3 asks for.

**The Aura row is the one worth reading, because item 8 is only half of it.**
Item 8 (CR 608.3b) is real and still owed, and this pass sized it: three
functions compute a spell's recipient from its effect and *none* of them can see
an Aura's `enchant_filter` — `oracle/mana_helpers.rs::spell_recipient` (the
CR 601.2c castability pre-check), the inline block in `engine/cast.rs`
(CR 601.2c target selection), and `engine/stack.rs::extract_recipient` (the
CR 608.2b fizzle). They are three copies of the same fourteen lines, so the fix
is one shared helper that takes the object rather than a fourth copy.

**But fixing it still would not make an Aura registerable, which is the new
finding.** No `AffectedSet` can name the permanent an Aura is attached to:
`static_affected_set` has exactly two productive arms, `FilteredPermanents` →
`Filter` and `Implicit` → `SourceOnly`, and `register_static_effects` runs
inside `place_on_battlefield` — *before* `resolve_taken` attaches the Aura, so
even `Fixed` has nothing to capture. `Duration::WhileEnchanted` exists and has
no consumer. Every faithful Aura's text is about its host, so the Aura half
needs a layers change on top of item 8 and is a phase, not a card drop. Auras
were left out of this PR on that basis, and the phase is now written up and scheduled: **Phase LH**, `layers-architecture.md` §13a, before critical-path item 7. **The `AffectedSet` half is small** -- one arm in `effect_applies_to`, resolved during the walk exactly as `ByController` is, so registration running before the attach is not the problem it first looked like. The cost is CR 613.7e instead; see "Before card breadth" item 4.

**`attach_aura_on_etb` (`engine/resolve.rs`) is dead code**, found the same way:
zero production callers, reached only by its own three unit tests. It implements
CR 303.4g's *choose a host on entry*, which is the right shape for an Aura put
onto the battlefield without being cast — a path no card can take yet. Left in
place rather than deleted, and recorded here so the next reader does not mistake
it for the live path (`engine/stack.rs`'s Aura branch is).

**Also registered: `adaptive_shimmerer`.** Its own doc comment said "this one
grows the stress pool" and `registry.rs` never registered it, so the claim was
false from the day RC-2 landed — the same silent gap as an unregistered card,
one level down. Registering it makes the doc true and adds a colorless 0/0 that
lives only on its counters, which is the sharpest board CR 704.5f has.

**Gate, both pools, 2026-09-01:** 200 stress games at seed 12345 `--threads 1` —
0 errors, 0 panics, 0 turn-limit hits. Determinism re-checked three runs at one
seed on both pools, byte-identical outside `=== Timing ===`. Fixtures re-recorded
in `plans/engineering-practices.md` §3; **both columns moved, because both pools
gained Battlegrowth**, so nothing in that table is comparable across this commit.

### Phasing (CR 702.26) — sized 2026-08-26, not started

Recorded because it reads as a large unknown and is not one. **CR 110.5 settles
the shape:** "A permanent's status is its physical state. There are four status
categories, each of which has two possible values: tapped/untapped,
flipped/unflipped, face up/face down, and **phased in/phased out**."

`BattlefieldEntity` already carries all four — `tapped`, `flipped`, `face_down`,
`phased_out`. The state is modelled; the behavior is not. Three consequences
worth having written down before someone re-derives them anxiously:

- **Phasing is not a zone change.** CR 110.5d: only permanents have status, and a
  phased-out permanent is still on the battlefield. So the whole RA event spine
  is untouched — no `ZoneChangeCause`, no LKI frame, no batch. CR 603.10b gives
  phase-out triggers their own look-back, which is the same shape as 603.10a's.
- **"Treated as though it doesn't exist" is an enumeration boundary**, and this
  engine has exactly one: `battlefield_ordered` / `battlefield_ids_ordered`,
  which `CLAUDE.md` already makes a hard invariant for determinism. That is the
  same boundary `replacement-architecture.md` §5a draws for the look-ahead frame
  — visible to applicability, invisible to enumeration — so the two want the same
  predicate, not two.
- **The turn-based action is CR 502.1**, phasing in and out *before* untapping,
  which `process_untap_step` already owns and which RA-3 already batched.

Size: 20 rules under 702.26; 13 cards with the phasing keyword, 47 touching it at
all. One status field (exists), one predicate at two enumeration functions, one
turn-based action, and the CR 603.10b look-back. It is a Phase 8-class mechanic
by card count and a small one by shape, and it does not interact with the
replacement pipeline beyond the boundary §5a already needs.

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

    **Layer 1a / 1b — and this entry had the two backwards until 2026-08-29.** `tmnt.txt` says **613.2a Layer 1a: Copiable effects** (copy effects, CR 707, and merging, CR 729) and **613.2b Layer 1b: Face-down**, i.e. copy *then* face-down. The enum has a single `Layer1Copy`. Not reachable today — nothing produces a layer 1 effect. `LAYER_ORDER` in `engine/layers/compute.rs` mirrors the enum, so splitting it later just lengthens that array; the frame-cache ceiling is an index into it, computed at runtime.

    **Both remaining sites corrected 2026-09-02 (CV-1): `compute.rs`'s `LAYER_ORDER` doc and `layers-architecture.md` §7 with its `Layer` code block.** `Layer1Copy` still collapses the two, and now has a producer in 1a; CV-6 splits it, and `layers::copy::END_OF_LAYER_1` is the one integer the split moves — pinned by a `debug_assert` that fires the moment `LAYER_ORDER[1]` stops being `Layer2Control`. The three sites had shared one wrong derivation: *"a Clone copying a face-down creature copies the 2/2 colorless characteristics, not the printed card (CR 707.2)"* is a **correct conclusion from a wrong premise**. The 2/2 does not come from face-down applying first; it is CR 708.2's own "Any listed characteristics are the copiable values of that object's characteristics", with CR 708.10 covering the copy-of-a-face-down case directly. Where it bites is exactly one integer: `copy-effects-architecture.md` §4.6 captures copiable values at the end of layer 1, and a reader who believes 1b is copy takes the ceiling one sublayer too early — a bug that appears only on boards with a face-down creature being copied.

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

   Unreachable today — Equip is unimplemented and Auras attach only at ETB — but the day any reattachment path lands, every equip silently re-orders Layers 6 and 7. **Corrected and scheduled 2026-09-01 -- "unreachable today" was the wrong frame, and so was "restate the contract".** Reattachment is not exotic: Aura Finesse (`{U}` Instant, "Attach target Aura you control to target creature") and Equip both do it with no new subsystem behind them. And the field is doing **two jobs** -- `battlefield_ordered` and `battlefield_ids_ordered` read it as *determinism / decision order*, `static_effect_timestamp` reads it as CR 613.7a. Four production readers, counted. Reassigning it makes a reattached Aura jump to the end of every ordered sweep, so the work is a **field split** -- a stable entry timestamp and a CR 613.7 timestamp -- not a reassignment plus a doc edit. Bounded by an in-tree precedent: CR 613.7c already reassigns `CounterStack.timestamp` from the same monotonic counter (`state/battlefield.rs:144`). Now **Phase LH-2**, `layers-architecture.md` §13a, scheduled before critical-path item 7. Original entry: **Do these together:** reassign from the same monotonic counter (still deterministic — that is the point), restate the contract as "never reassigned *except by CR 613.7e*" in CLAUDE.md and in `battlefield_ordered`, and re-audit every site that reads `timestamp` as a proxy for ETB order. Caught by audit before it had a reproducer; the two before it were found by their reproducers.

   **CR 613.7m is the same rule family and is also unimplemented (recorded 2026-09-01, RC-2 review).** "If two or more objects would receive a timestamp simultaneously, such as by entering a zone simultaneously or becoming attached simultaneously, their relative timestamps are determined in APNAP order." `allocate_timestamp` hands out a strict sequence one call at a time, so the engine can only ever produce a total order in *allocation* order — which is the caller's loop order, not APNAP. It is exact today because every entry is its own singleton batch: `propose_entry` is called once per zone change, and nothing puts two permanents onto the battlefield as one event. **The two things that change that are `GameAction::CreateTokens` (Phase RE) and CR 614.13's auxiliary zone changes (RC-4)** — either one makes the order APNAP has to fix reachable, and neither is far away. Note the shape of the fix is *not* "sort the batch": 613.7m gives the active player's objects the earlier timestamps **in the order of that player's choice**, so it is a decision point, not a sort. Related and separate: CR 613.7n's rule that an object's own static ability out-timestamps a resolving spell's effect on it is also unimplemented, and is the same batch of work.

5. **Layer 7c is not order-independent in Magic, and 19 cards say so (recorded 2026-08-24).** `compute.rs` applies ±1/±1 counters after the 7c registry slice without a timestamp merge. That is correct while every 7c modification is an addition — but CR 701.10a makes "double [a creature's] power" a 7c continuous effect whose addend depends on what already applied, so two doublings, or a doubling and a pump, are order-dependent by timestamp. Scryfall: **19 cards** match the doubling shape (Bulk Up, Epic Fight, Exponential Growth, Unnatural Growth…), before looser wordings. Inexpressible today — `AmountExpr` has no affected-power leaf — so nothing is wrong now. The first doubling card needs that leaf **and** the timestamp merge Layer 6's keyword counters already use. The comment at the code site was corrected 2026-08-24; it used to claim order-independence as a property of the layer.

6. **Multi-attacker block damage is a silent stub.** `engine/combat/resolution.rs:146`: a blocker blocking 2+ attackers assigns *all* its damage to the first living attacker — no `DecisionProvider` choice, no error, CR 510.1c ignored. Unreachable (nothing in the pool grants "can block an additional creature"), and the plumbing to fix it already exists: `GameState.blocker_damage_divisions` is populated from `choose_blocker_damage_division`. Reachable with the first "blocks an additional creature" card. This is the silent-wrong-*choice* cousin of the silent-inertness class the loud-lowering work covered.

7. **Hexproof and shroud are unenforced in spell targeting.** `engine/targeting.rs:302`, `TODO` tagged T22 (with matching notes at :39 and :293). `KeywordFlag::Hexproof` exists and combat honors it; spell targeting does not check either keyword. No registered card carries hexproof or shroud, so no game can reach it — reachable with the first such card, which is a Phase 8 event.

8. **A token created in exile instead logs `from: Battlefield` — RC-4b's cheap token answer, item 52 (recorded 2026-09-02).** Dour Port-Mage and Aang, Airbending Master — "leave the battlefield without dying" — read exactly that line and would draw a card or grant an experience counter for a token Hallowed Moonlight created in exile. The fix is Phase RE's `CreateTokens` proposal, whose destination the entry's decision sets, and it lands before any pool pairs a token-exiling replacement with a leaves-without-dying trigger. A hard back-stop, not an RE nicety.

9. **`can_pay_costs` checks each `Cost::Mana` against the whole pool, and
   `pay_costs` is not atomic across them (found 2026-09-02, closing 16c).**
   `assemble_total_cost` appends an additional cost's mana as its *own*
   `Cost::Mana` entry, and `check_cost_resource` asks "can the pool pay this
   one" per entry — so `{1}{R}` with kicker `{R}` passes against a pool of
   `{R}{R}`, the 601.2g window stops tapping the moment it passes, `pay_costs`
   pays the base and fails on the kicker, and CR 601.2's rewind returns the
   card with the base **already spent**. It is the second route to the hole
   16c closed, and the rollback closes only the stranding half of it.
   Unreachable today: no registered card carries an additional mana cost
   (`grep additional_cost src/cards` finds one comment), so no pool can build
   the board. Reachable with the first kicker card, which `backlog.md` §2.1
   owns. **Sized:** sum the `Cost::Mana` entries before checking and pay the
   sum once — that is also what CR 601.2h's "total cost" means — rather than
   snapshotting the pool around each entry.

### Before Triggered abilities (CR 603)

The trigger dispatcher's designated insertion point is `engine/priority.rs:234-240`. Today's gaps:

1. **Trigger dispatcher stub.** `let triggers_placed = false; // Phase 7 stub` at `engine/priority.rs:235`. This is the single-point insertion — **for placement only** (2026-08-24): detection runs synchronously at event dispatch per the resolved Replacement item 4; this stub is where the pending queue drains onto the stack in APNAP order (CR 603.3b, over the full player set).

2. **Event shape audit.** Every `events.emit(...)` call site is a potential trigger source. Before wiring triggers, audit that:

   **Known missing already (found 2026-08-24, registering the first activated ability):** `GameEvent` has no variant for an activated ability being put on the stack or resolving. `cast.rs::activate_ability` pushes the ability object onto the stack without `move_object`, so not even a `ZoneChange` is emitted, and the resolution emits nothing either — an activation is completely invisible in the event log. `AbilityCountered` exists, which is the whole of the vocabulary. Triggers that watch activations ("Whenever a player activates an ability…") have nothing to watch, and the event log cannot be used to audit activation behavior at all — measuring how often Merfolk Thaumaturgist's ability resolved needed a temporary probe in `resolve.rs`. Fix as part of the event-stream refit (Replacement item 3): the fork was resolved 2026-08-24 — `AbilityActivated` plus an identity-bearing `AbilityResolved` (source + ability, for CR 603.7h counting), emitted from the chokepoint.

   - Events are emitted at the correct granularity (e.g., `PermanentEnteredBattlefield` fires per-permanent, not per-batch).
   - Event timing is post-action, not pre-action, so triggers observe the completed state change.
   - Events carry enough context for trigger predicates (controller, source, type filters).

4. **~~The entry hop: Containment Priest's substitute leaves a permanent's worth of zone changes in the log for a card the CR says never entered~~ — ✅ CLOSED 2026-09-02 (RC-4b).** Entering is one proposal: `GameAction::EnterBattlefield` carries `from`, `change_zone` routes a battlefield destination to it, and its performer moves the card, announces the zone change, builds the entity and announces the entry. The Priest's substitute is one `ZoneChange { Graveyard → Exile }` with no LKI and one epoch, a dropped entry leaves the card where it was, and a "can't enter" may watch the entry (`phase_rc4b_integration_test`; `replacement-architecture.md` §9, RC-4b). The token residual is item 52. The record as found (2026-09-02, RC-4; sharpened in review): The `ZoneChange` performer moves the card into the battlefield zone and *then* proposes the `EnterBattlefield` (RC-2's one-`emit`-wide window), so "exile it instead" is performed as a `ZoneChange { from: Battlefield, to: Exile }`. Three things observe that: (a) the log holds a `ZoneChange` *into* the battlefield, so an ETB matcher on the zone change would fire — it must key on `PermanentEnteredBattlefield`, the performer's event, which a permanent that never entered does not have; (b) the log holds a `ZoneChange` *out of* it, `from: Battlefield` with a CR 603.10a LKI frame, so a leaves-the-battlefield or "exiled from the battlefield" matcher would fire, and a "leaves your graveyard" matcher would not, because the recorded `from` is wrong; (c) `zone_change_epoch` advances twice, so CR 400.7 sees two new objects. None is reachable today — no trigger matcher exists — but (b) and (c) have no keying rule that fixes them, so this is a bug-in-waiting for item 6, not a convention. **The fix is to reverse the nesting**, and it is the same restructuring as Deferred Migrations item 46: `GameAction::EnterBattlefield` carries `from`, its performer does the move, the placement and both emissions, and the `ZoneChange { to: Battlefield }` arm forwards to it *before* moving anything. Then the Priest's substitute is one `ZoneChange { from: <source zone>, to: Exile }`, the window is gone, `propose_entry`'s "replaced away" error is gone (a dropped entry leaves the card where it was, which is CR 614.6), a CR 614.17d "can't enter" may watch the entry, and a multi-entry batch is decided in phase 1 like any other proposal. The Priest stays in Root Maze's CR 616.1 bucket, which is what §11 item 19 needs reachable — moving the Priest to the zone change instead would split that bucket and force Priest-first. **Sized:** ~300–500 additions in `actions.rs` (two arms, `propose_entry`), `pipeline.rs` (the `Instead` arm), the token path in `resolve.rs` (`from: None`), and the RC-4 Priest tests' log assertions; CLAUDE.md's "one emitter" line is restated to name the entry performer. **Planned as RC-4b** — `replacement-architecture.md` §9 has the design, the token and CR 608.3e decisions, and the sizing — as its own PR ahead of RC-5, which needs entries to be batch members anyway.

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

**Inverted 2026-09-03 (`pool/everywhere-land`).** Every land in the registry made one or two colours, which is why a deck rolled one or two colours and filtered its nonlands to them, and why `--require` had to seed those colours from the required card's — a forced `{1}{G}{U}` in a deck that rolled red was included and never cast. Real "add one mana of any color" is not expressible (`backlog.md` §2.19), so the land is **Everywhere** (`cards/token_lands.rs`), the five-type token: it fills every land slot not taken by one basic of each type (`BASIC_LANDS_PER_DECK`, the contrast CR 305.7 needs) or by the `NONBASIC_LANDS_PER_DECK` draws. With that mana base the colour roll and the nonland filter had nothing left to do and are gone: 36 nonlands come from the whole pool. What it bought and what it cost are under "Found by the Everywhere pool change" below; the short form is that `--require` now measures a card against a random board (198 of 200 games with a non-G/U permanent, from 0) at half the resolutions, and the half is the random agent's tapping, not the deck.

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
