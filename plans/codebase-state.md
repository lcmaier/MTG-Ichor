# Codebase State — CR Coverage Map

Ground-truth snapshot of CR coverage. Single source of truth — if another planning doc contradicts this, this wins. Last grounded-in-code audit: 2026-08-25.

---

## TL;DR

- **v1 is two use cases** (owner, 2026-08-24): peer-to-peer human games through a GUI, specifically **4-player Commander**, and **highly parallel AI games** over the CLI. Two-player Standard is a checkpoint, not the target. Ordering lives in `CLAUDE.md` → "Critical path to v1"; the consequence for this file is that CR 800/802 and CR 903 below are path items, not deferrals, and that new systems get written N-player-shaped. `plans/state-of-play.md` is the generated board; this list is the prose beside it, rewritten in place at the post-RE audit (2026-09-15).
- **Code size:** 62,241 lines of Rust across 118 `src/` files, plus 32,712 in `tests/`. 1,537 tests, 0 warnings. `fuzz_games` runs 200-game batches over two pools (`performance`, 89 cards; `stress`, every registered card, 159) at any seat count (`--players`), exits 1 on a panic or an unpaid resolution, and CI checks three runs at one seed line for line. The last readings: 13.98 ms CPU per two-seat game, 44.83 ms per four-seat (`fuzz-record.md`, RE-9's block).
- **Well-covered:** CR 1 (game basics), CR 3 (card types), CR 4 (zones), CR 5 (turn structure — every turn, phase and step is a proposed event since RE-1, and the turn's sequence is data since RE-10), CR 7 (keyword abilities + SBAs).
- **Partially covered:** CR 6 casting — the pipeline, X, alternative and additional costs, and **cost determination as its own pipeline** (`cost-architecture.md`, CM-0–CM-4, 2026-09-07/08: CR 601.2f's step, the spell's own cost abilities, sacrifice as a cost, the mana window and a payer split into two decorators); mode choice, distribution and target uniqueness pending; activation restrictions are one value (`ActivationRestriction::OnlyAsSorcery`, LH-2), the rest `backlog.md` §2.8. CR 1 mulligan is a stub. Equip ✅ (LH-2); Bestow not started.
- **Not started:** **triggered abilities (CR 603)** beyond an enum variant — the record they will match against is RA's performed stream, and `replacement-architecture.md` §14 lists what item 6 inherits; CR 802's defending player and CR 800.4f–h's choices by a departed player ("Before Commander" item 4); the information model (`backlog.md` §2.9).
- **Replacement effects (CR 614–616) — ✅ complete, Phases RA–RE, 2026-08-25 → 2026-09-15, twenty-four PRs; critical-path item 5 closed with RE-9 and was audited 2026-09-15.** Every observable mutation is a `GameAction` proposal (22 kinds) through one chokepoint; `apply_replacements` runs CR 616.1's loop between proposal and mutation; entering is one event through the CR 614.12 look-ahead frame; damage carries CR 120.3's results, CR 615.7's shields and CR 614.9's redirection; skips, draw, life, tokens, counters, the game's end and a player leaving it, discard, scry, mana and extra phases are all events. The CR 614–616 row below carries the "not yet" list; `replacement-architecture.md` §14 is the phase in hindsight.
- **"Can't" effects (CR 101.2/614.17/613.11) — the spine is live (RS-0, RS-1, 2026-08-31).** `plans/cant-effects-architecture.md` is authoritative; `RestrictionDef` / `Restriction`, the third `DurationRegistry` customer, and `engine::restriction::is_prohibited` — one predicate over *effective* ability lists, checked ahead of the replacement pipeline. Still ahead: RS-2 (casting/activating/targeting), RS-3a/b (combat), RS-4 (costs).
- **Copy effects (CR 707/712/708/729 + Layer 1) — the capture is live (CV-1, 2026-09-02).** `plans/copy-effects-architecture.md` is authoritative; `CopiableValues`, `EffectModification::CopyFrom` from `Primitive::Copy`, and the two gate legs a copied ability lights. Still ahead: CV-1b, CV-2 (enters as a copy — CR 616.1c's bucket has waited for it since RC-4), CV-3–CV-7; CV-7 (merging) back-stopped before Phase 8.
- **Layers (CR 613) — the system is complete except Layer 3 and Layer 1b (Phases LA–LK, 2026-05 → 2026-09-14).** `Layer` with all nine sublayer variants, `EffectiveCharacteristics`, a `ContinuousEffect` registry over the shared `DurationRegistry`, and `compute_characteristics` inside **one board-wide pass per board** (LI-1) with **the CR 613.8 dependency algorithm** (LI-2) and conditional statics (LI-3); attachment as a layers input and CR 613.7e's timestamp split (LH-1/LH-2); the zone-reaching `ObjectSet` (LJ) and **CR 113.6, which abilities function in which zone** (LK — the registration leg; RF, 2026-09-16 — the replacement sweep's zone leg, `replacement-architecture.md` §9; TR-1 review theme C, 2026-09-22 — the trigger dispatcher's, per ability rather than per object; the restriction sweep still visits the battlefield alone, main item 146). `oracle/characteristics.rs` wrappers all route through it. Layer 3 (text) is an enum variant; Layer 1b (face-down) waits on CV-6. CR 305.7/305.6 ✅ (`engine/layers/land_types.rs`).
- **Commander (CR 903) — the zone rules are in, the format is not.** Command zone ✅; commander damage ✅; **903.9a (CR 704.6d) and 903.9b ✅ (RB)**; games of three or more seats run, a lost player leaves (RE-6, RE-7: CR 104, 800.4a–e) and the rotation is N-player (RE-1's `turn_rotation`). Still missing: the tax (`cost-architecture.md` §3.8 — ~40 lines against the cost pipeline, waiting on designation), `GameConfig::commander()`, and a designation hook — nothing outside tests sets `is_commander`, so neither 903.9 half is reachable in a real game yet.
- **What is next on the spine:** the triggers architecture doc and critical-path item 6 — the gather's zone leg landed 2026-09-16 (RF, `replacement-architecture.md` §9), which closed critical-path 6a. Between phases, in the order pass 4 of the post-RE audit proposed and the owner decides (`roadmap-v2.md` §3a, rows A4e–A4k): item 138's counters and its two callgrind levers landed 2026-09-16 (A4e, A4f, A4g); item 139 with the fork test, A4b's rulings ledger and A4c's trace sink remain; RS-2 and CV-2 beside, pulled when a card family wants them. The audit's record is "Was critical-path item 5 done, and what sits before item 6? — audited 2026-09-15" below.
- **Before starting any of those systems:** see **[Deferred Migrations](#deferred-migrations)** for the debt owed by forward-looking scaffolding — 193 items as of 2026-09-15 (the audit's close), three of them reachable and wrong today (59, 60, 122), none unstated. Each target system (Triggers, Commander, Phase 8's breadth) has a subsection to read before its first ticket.
- **Five architecture docs own their subsystems:** `layers-architecture.md`, `replacement-architecture.md`, `cant-effects-architecture.md`, `copy-effects-architecture.md`, `cost-architecture.md` — each with its type shapes, phase codes and findings; `CLAUDE.md`'s authority table is the index. A subsequent session executes from those, never from this summary.
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
| 103.5 | Mulligan (London) | ⚠️ **stubbed** — "players always keep their first hand"; `backlog.md` §2.32 | `state/game.rs`, `Game::setup` |
| 103.6 | Starting hand size | ✅ | `state/game_config.rs`, `state/game.rs:98-104` |
| 107 | Mana values, X costs, hybrid/Phyrexian symbols (enum) | 🟡 enum defined; hybrid/Phyrexian/X payment = `NotImplemented` | `types/mana.rs`, `can_pay` returns false for hybrid |
| 108 | Tokens and cards | ✅ `is_token`, `is_copy` flags | `objects/object.rs` |
| 109 | Objects, characteristics | ✅ data model | `objects/card_data.rs`, `objects/object.rs` |
| 110 | Permanents | ✅ `PermanentState` + attachment | `state/battlefield.rs` |
| 111 | Tokens — cease-to-exist | ✅ SBA 704.5d | `engine/sba.rs:332+` |
| 117 | Timing + priority | ✅ priority rounds, mana-ability window (601.2g / 602.1b), bounded retry + pass fallback | `engine/priority.rs`, `engine/put_on_stack.rs` |
| 118 | Costs (types only) | ✅ alternative/additional cost enums; X + kicker + flashback + evoke scaffolding | `types/costs.rs` |
| 118.8–118.9 | Alternative / additional cost resolution | 🟡 determine_total_cost (`engine/cost_determination`) + rollback done (T18a); wiring per-cost-type semantics pending (T18b/c/d) | `engine/put_on_stack.rs`, `engine/costs.rs` |
| 119 | Life changes | ✅ with source attribution | `events/event.rs`, `engine/actions.rs` |
| 120 | Damage — combat damage routing, infect/wither/lifelink | 🟡 combat damage ✅, lifelink ✅, first/double strike ✅, trample ✅, deathtouch ✅. **CR 120.3's results are a list, decomposed off the target's effective types (RD-1, 2026-09-08)**: 120.3a proposes a contained `LoseLife { cause: Damage }` in the damage's batch, 120.3c proposes `RemoveCounters { Loyalty }` — so a planeswalker can die (CR 704.5i fires 4× in 400 stress games) — 120.3e is gated on the target being a creature, and 120.3f was already lifelink's. **The event carries `unpreventable` from RD-4 (2026-09-09)** — CR 615.12's per-event shape, set by the effect that proposes the damage and carried through a CR 614.9 redirect, because "the same damage" is what a redirect moves. **120.3b/d/g/h ❌** — poison, wither's counters, toxic, a battle's defense counters; each is one more arm on the same `DamageResults`, and each has an owner (`backlog.md` §2.6 and §2.23; Deferred Migrations items 86 and 87) | `engine/actions.rs` (`DamageResults`), `engine/combat/keywords.rs`, `engine/combat/resolution.rs` |
| 121 | Drawing | ✅ basic | `engine/actions.rs` |
| 122 | Counters | ✅ 21 counter kinds in one `CounterType` (12 evergreen keyword, +1/+1, -1/-1, loyalty, charge, shield, stun, finality, poison, energy); a permanent's in `PermanentState.counters` with CR 613.7c timestamps, a player's in `PlayerState.counters` (RE-5, 2026-09-13); both put on through `GameAction::AddCounters { subject: CounterSubject, by }`, and CR 122.6's entry counters watched through the entry's mods | `types/effects.rs`, `state/battlefield.rs`, `state/player.rs`, `engine/replacement/gather.rs` |
| 123 | Mana (pool, persistence, restrictions) | ✅ full `ManaPool` with restricted sidecar, persistence, grants, context-aware spending (T12b landed) | `types/mana.rs` (1370 lines) |

### CR 2 — Parts of a Card

| Section | Rule topic | Status | Where |
|---|---|---|---|
| 201–205 | Name/mana cost/color/color indicator/type line | ✅ data model | `objects/card_data.rs` |
| 205.4d | Supertypes (legendary) | ✅ enforced by legend rule SBA | `engine/sba.rs` (704.5j) |
| 206 | Expansion/rarity | not modeled — not needed for engine |
| 207 | Text box / rules text | 🟡 stored as `String`; not parsed into structured abilities (no NLP, hand-coded card defs) | `objects/card_data.rs` |
| 208 | P/T (`i32`) | ✅ signed, correct per E8 | `objects/card_data.rs` |
| 209 | Loyalty (for PW) | ✅ ETB counter init + 0-loyalty SBA, and CR 120.3c takes counters off from RD-1 on, so the SBA is reachable from a game (Loyalty Probe) | `engine/sba.rs` (704.5i), `state/game_state.rs` (init_etb_counters), `engine/actions.rs` |

### CR 3 — Card Types

| Section | Rule topic | Status | Where |
|---|---|---|---|
| 301 | Artifacts (incl. 301.5 Equipment — attachment + can't-attach-to-non-creature) | ✅ attachment tracking, `GameState::attach` / `detach` the one writer, **Equip ✅ (LH-2, 2026-09-05)** — 301.5b's "control matters when it resolves" is CR 608.2b's re-check; 301.5c's creature-Equipment clause and 702.6c qualities are card breadth | `state/game_state.rs`, `engine/resolve.rs`, `cards/phase_lh_cards.rs` |
| 302 | Creatures + summoning sickness | ✅ turn-based tracking (T09) | `oracle/characteristics.rs` `has_summoning_sickness` |
| 303 | Enchantments / Auras — an Aura spell targets its enchant ability (303.4a), enters attached (303.4 / 608.3c), fizzles against a gone target (608.3b), "enchanted creature" reaches the host (303.4m, LH-1 2026-09-04), control on resolve (303.4e); **non-stack ETB host choice (303.4f/g) ❌** — `attach_aura_on_etb` was dead code and was deleted with LH-1 | 🟡 | `engine/targeting.rs` `spell_recipient`, `engine/stack.rs` Aura branch, `state/game_state.rs` `attach`/`detach`, `objects/card_data.rs` `enchant_filter` |
| 304 | Instants | ✅ basic cast path | `engine/put_on_stack.rs` |
| 305 | Lands | ✅ basic lands + mana abilities | `cards/basic_lands.rs` |
| 306 | Planeswalkers | ✅ loyalty ETB, 0-loyalty SBA; loyalty-ability costs ❌ (T19 pending) | `engine/sba.rs` |
| 307 | Sorceries | ✅ basic cast path + sorcery-speed enforcement | `engine/put_on_stack.rs`, `oracle/legality.rs` |
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
| 408 | Stack | ✅ with rollback | `engine/stack.rs`, `engine/put_on_stack.rs` |

### CR 5 — Turn Structure

✅ Complete (Phase 1/2 work). `engine/turns.rs`, `engine/combat/steps.rs`, cleanup step with SBA re-loop (T16). 514.3a re-loop in `state/game.rs` `perform_cleanup_actions`.

### CR 6 — Spells, Abilities, and Effects (**THE BIG ONE**)

| Section | Rule topic | Status | Where |
|---|---|---|---|
| 601.2a | Announce spell / move to stack | ✅ | `engine/put_on_stack.rs` (780 lines) |
| 601.2b | Choose modes / X / alt+additional costs | 🟡 X **chosen and paid** ✅ (X-dependent *resolution* amounts ❌ — `engine/resolve.rs:786-804` returns `Err` for a resolving `Variable`/`TargetPower`/`CountOf` amount; loud, and unreachable with no such card registered), alt ✅, additional ✅ (T18a); **mode choice ❌** (T18b pending — `ChoiceKind::ChooseModes` not added yet) | `engine/put_on_stack.rs` |
| 601.2c | Choose targets + target uniqueness | ✅ multi-target with `TargetCount::Exactly(n)` / `UpTo(n)` min/max enforcement; `validate_targets` called post-selection; **uniqueness rules (115.3/4) ❌** (T18b) | `engine/put_on_stack.rs:130–152`, `ui/ask.rs` |
| 601.2d | Distribution (damage/counters among targets) | ❌ still unbuilt after A4i, and now the only half of `backlog.md` §2.20 left — `roadmap-v2.md` row A4l sizes it | `engine/put_on_stack.rs` |
| 601.2e | Post-proposal legality | ⚠️ **explicit no-op** with a comment: *"Currently a no-op (the pre-proposal check is sufficient for the cards we support). Future: validate that chosen targets are still legal after all proposal choices are made"* | `engine/put_on_stack.rs:175–182` |
| 601.2f | Determine total cost | ✅ | `engine/cost_determination/total.rs` `determine_total_cost` — the whole step since CM-1 (2026-09-07) |
| 601.2g | Mana ability activation window | ✅ (SPECIAL-2) | `engine/priority.rs` `run_mana_ability_window` |
| 601.2h | Pay costs (with rollback on failure) | ✅ for `Cost::SacrificeSelf`, `Cost::Tap`, `Cost::PayLife`, `Cost::Mana`; **`Cost::Sacrifice(filter, count)` = `NotImplemented`** (T18c) | `engine/costs.rs` |
| 601.2i | Spell becomes cast | ✅ | `engine/put_on_stack.rs` |
| 602 | Activated abilities (activate_ability + rollback) | ✅ structural; **activation restrictions** (sorcery-speed PW, graveyard-activated abilities) ❌ (T19) | `engine/actions.rs` activate_ability |
| **603** | **Triggered abilities** | ❌ `AbilityType::Triggered` enum variant exists (`objects/card_data.rs:49`), **no engine handling**. No trigger queue, no event→trigger mapping, no "puts X onto the stack" mechanism. | only in `ui/display.rs:164` for label printing |
| 604 | Static abilities | 🟡 keyword statics via `has_keyword`; continuous-effect statics (P/T, color, type) register via `GameState::register_static_effects` ✅; other non-keyword statics ❌ | `state/game_state.rs` |
| 605 | Mana abilities | ✅ detection + window + enumeration | `oracle/mana_helpers.rs`, `engine/priority.rs` |
| 606 | Loyalty abilities | ❌ (T19 pending) |
| 607 | Linked abilities | ❌ (T20 pending) |
| 608 | Resolution of spells and abilities — fizzle, Target vs Choose split | ✅ via T15b refactor (`TargetSpec` → `EffectRecipient`) | `engine/resolve.rs`, `engine/stack.rs` |
| 609–611 | Effects (one-shot, continuous) | ✅ one-shot via `Effect`/`Primitive`; continuous via the layer registry with duration-based expiry | `state/continuous_effects.rs` |
| 612 | Text-changing effects | ❌ |
| **613** | **Continuous effects — layer system** | 🟡 **core landed; layers 7b/7c/7d, 5, and 4 live.** `Layer` enum + `EffectiveCharacteristics` + `ContinuousEffect` registry + `compute_characteristics` all exist and are exercised by the Phase LB/LC/LD tests. **Missing:** Layer 3 (text), Layer 1b (face-down, CV-6). **Landed since this row was written:** the board-wide pass (LI-1) with the CR 613.8 dependency algorithm (LI-2) and conditional statics (LI-3), all 2026-09-06, `engine/layers/board.rs`; the zone-reaching `ObjectSet` (LJ) and CR 113.6's registration leg (LK), 2026-09-14. Layers 2 and 6 live since 2026-08-23; CR 305.7/305.6 land semantics landed in Phase LD Part B. | `engine/layers/{types,board,compute,cda,land_types}.rs`, `state/continuous_effects.rs`, `oracle/characteristics.rs` |
| **614–616** | **Replacement + prevention + interaction** | ✅ **Phases RA–RE complete (2026-08-25 → 2026-09-15, twenty-four PRs); critical-path item 5 closed with RE-9 and was audited 2026-09-15.** Every observable mutation is a `GameAction` proposal (22 kinds) through `execute_actions`, carrying `ZoneChangeCause`, the CR 603.10a LKI frame, a `BatchId` and its resolution (RA); `apply_replacements` runs CR 616.1a–g between proposal and mutation, with CR 614.4/5/6/7a/17, 616.2, CR 615.5 riders and CR 101.4 APNAP (RB); the CR 614.12 look-ahead frame (RC-4), entering as one event (RC-4b), CR 614.13's auxiliary moves (RC-5); damage with CR 120.3's results, CR 615.7's shields decided per `(batch, subject)`, CR 609.7's sources, CR 614.9's redirection and CR 615.12's unpreventable damage (RD-1–4); skips and the turn queue, draw with its lineage, life, tokens, counters on permanents and players, the game's end and a player leaving it, the discard and scry producers, mana, extra phases and the turn plan (RE-1–10). `Rewrite` is the closed algebra `replacement-architecture.md` §3.2b claims. **Not yet:** CR 614.15 self-replacement (bucket, no producer — §11 item 3); CR 614.1e turned face up (CV-6); CR 614.12b (main item 134); CR 614.12c and 614.14 (`backlog.md` §2.2); CR 614.9's *partial* redirection (`backlog.md` §2.25); CR 615.13 (critical-path item 6); dice (`backlog.md` §2.31) and search (`backlog.md` §2.5, with its producer) as events; CR 121.2c's draw order (main item 122); a substitution keeping the replaced event's `cause` (main item 131); CR 731 (`backlog.md` §2.28). §14 there is the phase in hindsight. |

### CR 7 — Additional Rules

| Section | Rule topic | Status | Where |
|---|---|---|---|
| 701.3 | Attach | ✅ `Primitive::Attach` → `GameAction::Attach` → `GameState::attach` (LH-2, 2026-09-05): 701.3b's same-host no-op and "doesn't move", 701.3c's new timestamp; Aura ETB attach ✅ (T15b) still calls the writer directly from `stack.rs` |
| 701.8 | Destroy (destroy keyword action, respects indestructible) | ✅ (T16) | `engine/resolve.rs` Primitive::Destroy |
| 701.21 | Sacrifice | 🟡 `Cost::SacrificeSelf` ✅; `Cost::Sacrifice(filter, count)` = `NotImplemented` (T18c) | `engine/costs.rs` |
| 702.2 | Deathtouch | ✅ (combat lethal-damage check, T09 fuzz run confirmed) | `engine/combat/keywords.rs` |
| 702.6 | **Equip** (activated ability "Equip {cost}") | ✅ (LH-2, 2026-09-05) `phase_lh_cards::equip`: `Primitive::Attach` behind `Target(Permanent(creature ∧ you control), Exactly(1))`, `ActivationRestriction::OnlyAsSorcery`; 702.6c/d/e variants are card breadth |
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
| **704.5a–w** | **State-based actions** | ✅ 704.5a (life ≤0), 704.5b (empty library draw), 704.5c (poison ≥10), 704.5d (tokens in non-BF zones), 704.5f (0 toughness), 704.5g (lethal damage with indestructible + deathtouch), 704.5h (deathtouch), 704.5i (PW 0 loyalty), 704.5j (legend rule), 704.5m (Aura illegal host), 704.5n (Equipment/Fort on illegal permanent), 704.5p 🟡 (the "attached to an illegal object" half; the *creature* clause is a TODO at `engine/sba.rs:323` — the sweep exempts every Aura/Equipment/Fortification without asking whether it is also a creature, and 704.5p's first sentence unattaches a creature regardless of what else it is. The illegal-host half fires — 15 per 200 stress games since LH-2 (2026-09-06; 7 on Bonesplitter, 8 on Cobbled Wings), Mirrorform copying a non-creature onto the equipped creature; the creature clause stays unreachable: no enchantment animator), 704.5q (+1/+1 / -1/-1 annihilation). 704.5s (Saga), 704.5t (dungeon), 704.5v/w/x (battle) ❌. Commander damage ✅. | `engine/sba.rs` (1015 lines) |
| 705 | Flipping coins, rolling dice | ❌ |

### CR 8 — Multiplayer Rules

**Commander is in scope. This section is a real gap, not an out-of-scope deferral.**

| Section | Rule topic | Status | Where |
|---|---|---|---|
| 800 | General multiplayer rules (active player turn order, multiple opponents) | 🟢 **800.4 is built** (RE-6, RE-7): the turn rotation and the priority loop pass over a departed seat (800.4j/k), their objects leave the game and what they controlled is exiled (800.4a, 800.4c), the four refusals hold (800.4b/d/e) and 800.4m expires at the turn that would have begun. 800.1's seat count is the gate — `GameState::is_multiplayer`. **Left: CR 800.4f–i**, the choices and the last known information a departed player owes, which are "Before Commander" item 4's | `engine/leaving.rs`, `engine/turns.rs`, `engine/priority.rs` |
| 801 | Limited range of influence | n/a for Commander (uses range = all) |
| 802 | Attack Multiple Players option | ❌ combat assumes single defender |
| 806 | **Free-for-All** — the default Commander game structure | 🟡 the rotation and the elimination are built (800.4, above) and measured at four seats by `fuzz_games --players 4`; what is missing is the *format* — `GameConfig::commander()` and the designation step, "Before Commander" items 2 and 3 |
| 810 | Two-Headed Giant | ❌ |
| others | Grand Melee, Team vs Team, Emperor, Alternating Teams | ❌ |

**Known multiplayer-shaped gaps in existing 2-player code** — under the v1 redefinition this list is the design checklist for CR 800, not a backlog. The cheap way to buy multiplayer is to write CR 614 and CR 603 N-player-shaped as they land (CR 616.1's affected-player ordering and CR 603's APNAP queue both take a player set in the CR); retrofitting is the expensive path. CR 800.4 elimination is the piece most likely to be underestimated — a leaving player's permanents, stack objects, and effects each need specific resolution:
- `engine/combat/validation.rs` assumes attacks go at "the defender" (single opponent). **Half closed 2026-09-12 (RE-6):** CR 506.2 filters the attack-target list to the active player's opponents, so a departed seat is not offered; CR 802's *choice* of defending player is still the harness's.
- ~~Priority passes loop player0 → player1 → back; no general N-player priority-pass loop.~~ ✅ **closed 2026-09-12 (RE-6)** — the loop rotates through `next_player_in_game` and 800.4j passes over a departed seat.
- Targeting prompts don't enumerate 3+ players as target candidates in most paths (SPECIAL-8 blocker pre-filter doesn't need to, but spell targeting does).
- ~~No player-elimination SBA (rule 800.4a)~~ ✅ **closed 2026-09-13 (RE-7)**, and it is **not** a state-based action — CR 800.4a says so in as many words. It runs inside the `PlayerLoses` performer (`engine/leaving.rs`), which is what the old bullet's "treated as though they don't exist" was reaching for.

### CR 9 — Casual Variants

**Commander (CR 903) — in scope, partially modeled.**

| Rule | Topic | Status | Where |
|---|---|---|---|
| 400.7 / 406 | **Command zone as a Zone variant** | ✅ `Zone::Command`, `GameState.command: Vec<ObjectId>`, wired into `move_object` / `remove_from_zone_collection` | `types/zones.rs:10`, `state/game_state.rs:70`, `engine/zones.rs:208–216,255–259` |
| 903.3 | **Starting life = 40** | ❌ no `GameConfig::commander()` constructor; `game_config.rs` header comment promises one via a future `Format` trait |
| 903.5a | **Mulligan (London, same as standard)** | ⚠️ mulligan itself is stubbed (`Game::setup`; `backlog.md` §2.32) regardless of format |
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

**What does not belong here (adopted 2026-09-12, RE-6's review).** A wrong
answer sized under about thirty lines, with a fixture that can prove it, is
**fixed in the PR that found it**, not recorded. This list is what a *later
system* has to carry — a facility that does not exist yet, an arm with no
customer, a wrong answer whose fix is a phase — and an entry longer than its
fix is the sign it should have been a commit. RE-6's first cut recorded six
entries and four were that shape (`replacement-architecture.md` §11 item 67).
**The backlog of small fixes as of 2026-09-12**, read off every open item's
own `**Sized:**` line, for a miscellaneous PR: items 12 (an explicit `None`
toughness arm, 10 lines), 49 (`is_prohibited`'s entering-permanent leg, 20),
84 (two dead helpers, 20), 103 (a `debug_assert!` on the swallowed filter
`Err`, 10), 117 (land drops reset at turn begin, 10), 118 (the repeated
cleanup step announced, 10), 32 (`DeckLimits` validated, 40) and "Before card
breadth" item 1 (CR 208.3's no-P/T gate, 10) — about 130 lines of engine and
their fixtures, no new arm among them; plus two perf items that want the A/B
beside them, 22 (the counter fast path's set, 15) and 50 (one enumeration per
CDA, 30), and one harness diagnostic, 104 (an activation counter, 20). The
other small-sized items are each one card or one system away and stay.

**Every open item is a wrong answer or a missing facility; reachability is
the axis the rows split on** (2026-09-13, RE-4's review). "Unreachable" says
no fuzz game can produce the wrong answer today, never that the answer is
right — an item that wrote "unreachable rather than wrong" meant "wrong, and
unreachable", and the phrase is retired.

**And an item is not closed by an empty Scryfall query** (2026-09-14, RE-5's
review). A facility the CR states is owed whether or not a card prints it —
a card can be printed next set, and custom card creation is a post-v1 goal.
The reachability line says "no printed producer", never "nothing owed", and
a fixture test is the customer until a card is. `engineering-practices.md`
§4; item 43 is the one that was closed that way and reopened.

**Item ids are section-scoped, not unique (decided 2026-09-03).** Four runs
share the numbers 1–65: the main run, which spans every dated "Found by …"
subsection, and one run each inside "Before Layers", "Before card breadth",
"Before Triggered abilities" and "Before Commander". Cite an item with its
section — "'Before card breadth' item 4", "main item 46" — and never bare;
`CLAUDE.md`'s critical path is a third namespace ("critical-path item 7") and
each architecture doc's §11-style findings a fourth
("`replacement-architecture.md` §11 item 5"). Renumbering into one space was
considered and not done here: it would rewrite ~30 item headings and the 477
"item N" citations counted across `plans/`, `CLAUDE.md` and `mtgsim/` for a diff
that would bury this triage, and it would still leave the critical-path
collision. The durable fix is a prefix (`DM-46`, `CP-7`, `RA§11-5`) swept
through every citation as its own mechanical PR; it is step 5 in the audit's
order below.

**Not done 2026-09-14, and this is where the next person should look.** It was
put to `refactor/object-set-rename` as a rider — same kind of sweep, same "its
own PR" argument — and declined, but the reason is not "later": it is that the
two sweeps are not the same shape. A type rename has a compiler and a
byte-identity check behind it; a citation prefix has neither, because the
citations are prose. Re-derived 2026-09-14 against the live tree (`plans/`,
`CLAUDE.md`, `mtgsim/src`, `mtgsim/tests`, archive excluded): 104 bare
"item 6"/"item 6's" against 62 already qualified, 166 in all. Deciding which
namespace each bare one meant is a *reading* of every site, not a
substitution — a `sed` that guesses wrong makes a citation confidently point
at the wrong item, which is worse than one that is merely ambiguous. So it is
owed as a PR with a human-checked table, not as a mechanical pass, and it
should be sized that way when it is taken.

**Closed items are evicted to `plans/archive/codebase-state-closed.md`
(2026-09-09).** An item leaves when its dated verdict says `closed` and leaves a
three-line stub behind — the heading, the verdict's closing sentence, and a
pointer — so every "item N" citation still resolves at its own number and
nothing is renumbered. The archive is a record of finished work, keyed by the
same section-scoped ids. **The eviction is the closing PR's job**, the same way
the verdict is: close it, stub it, append the body.

### Deferred Migrations, measured (2026-09-03) — and triaged the same day

**Asked on review after RC-5 added 13 items in one PR**, the largest single
addition this section has taken. Measured first, then triaged (steps 1 and 2
of the order below) in `docs/deferred-migrations-triage`. The live counts are
on `plans/state-of-play.md`, which derives them and is checked in CI; these are
the dated snapshots on either side of the triage. The parser changed between
the two columns and the change is itself a finding: the old one counted any
column-0 `N.` line, so it took this block's own four-step list and two lists in
prose for items, and it could not see the lettered sub-items (`7a`–`7h`,
`16b`–`16e`) at all. It now counts `Na.` too, the prose lists use the `)`
delimiter, and it reads the dated `**Reachability (YYYY-MM-DD):**` verdict every
open item now carries.

| | before, old parser | before, this parser | **after** |
|---|---:|---:|---:|
| Deferred Migrations, lines | 2,896 of 3,111 (93%) | — | **3,726 of 3,941 (94%)** |
| numbered items | 108 | 119 | **112** |
| closed / struck, still in the file | 20 | 23 | 21 |
| open — unreachable, and says why | 23 | 23 | 55 |
| open — reachable, **wrong today** | — | — | **5** |
| open — reachable, not wrong (perf, a name, a harness) | — | — | 9 |
| open — nothing to build; a record for a later phase | — | — | 22 |
| **open — reachability not stated** | **65** | **73** | **0** |
| open items carrying `**Sized:**` | 21 | 20 of 96 | **91 of 91** |

**What the triage found.** Every open item now names the card, stub or code
path that decides its reachability, dated, and carries a size — a range, a
pointer to where it is already sized, or "unknown until X", which is itself a
size. The verdicts that were not what the entry said:

- **Five items are reachable and wrong today — four distinct wrong answers,
  each a bug report filed as a deferral.** The CR 704.5d token sweep emits
  `TokenCeasedToExist` in `HashMap` order when two tokens leave in one sweep
  (main item 6; the `stress` pool reaches it through Kalitas's Zombies). Sutured
  Ghoul dies as a 0/0 after exiling a real creature — counted: 6 of 27 entries
  in 40 `--require` games, once per seven games (main item 59; waits on CR 607).
  Master Biomancer's Mutant type is missing, unobservably (main item 60). And
  **Humility before Citanul Hierophants leaves the Hierophants' mana grant on
  every creature Humility just stripped** ("Before Layers" items 7b and 8, one
  board): both cards are in `PERFORMANCE_POOL`, a creature under Humility taps
  for mana, and it is the CR 613.8 dependency case the entry had filed as three
  things away — pinned by
  `test_humility_before_hierophants_does_not_yet_retire_the_grant` so the
  board-wide pass has to flip it.
- **One "unreachable" claim had gone stale — 7b's — of the 23 the old parser
  counted**, and one had gone stale the other way: the CV-1 review's C6 called
  main item 10 a live wrong answer through Giant Growth, and it is not, because
  nothing returns an object. So the rate is about one stale verdict in twenty
  over the seventeen PRs since the oldest of them (RA-3, 2026-08-25). That is
  cheap enough to make a rule of: **re-derive the verdicts that name a thing a
  phase built** — a card, a stub, a primitive — **at that phase's close, and run
  the whole pass every 15–20 PRs.** The dates on the verdicts are what make the
  second half checkable.
- **One item was closed and never struck** — main item 5, CR 601.2a's
  announcement, closed by RC-4b under item 51's number. One was stale rather
  than wrong: "Before Commander" item 3 still said only CR 903.9b was missing,
  and both halves of command-zone redirection landed with RB.
- **`plans/handoffs/cv-1-review.md`'s six open items are absorbed** as main
  items 66–68 plus three dispositions, and both that file and the triage's own
  handoff are deleted, as their contract says.
- **Two stale local branches were deleted** — `replacement/rc-2-enter-battlefield`
  and `phase-ld-part-b`, one docs commit each, both already in `main` by
  another route (item 44's text; the CR 613.7a wording in
  `layers-architecture.md`).

**The order, re-recorded.** (1) reachability and (2) size are done above.
~~**(3) Collapse the 21 closed items to one line each**, pointing at the
commit~~ — **done 2026-09-09, at 37 items rather than 21**, and as an eviction
to `plans/archive/codebase-state-closed.md` rather than a deletion: the text is
the reasoning, and `git log` holding it is not the same as a reader finding it.
**(4) Splitting the file — asked and answered by (3) rather than taken.** The
objection stands exactly as written: this file wins over every other doc, and a
*Deferred Migrations* that lives elsewhere is one more doc to forget. But that
objection is about the **open** items, and nothing open moved — the live section
is still the whole of the live ledger, and only finished work left. So (4) is
closed as "no", not deferred. **(5) The id prefix sweep** — the namespace decision is in the section header: ids stay
section-scoped, cited with their section, and the durable fix is a prefix
(`DM-46`, `CP-7`) swept through the 477 "item N" citations across `plans/`,
`CLAUDE.md` and `mtgsim/` as its own mechanical PR.

**Should the next phase wait for any of it?** No. What gated correctness was
(1) and (2), and the four wrong answers above are the phase-independent output:
the token order is a five-line fix with a test, the Ghoul is CR 607's, the
Mutant type is a phase of its own, and Humility + Hierophants is critical-path
item 7's, already scheduled before Phase 8 breadth.

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
`layers-architecture.md` §12's 5.2—8.0→ figure is the CR 604.2 check
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
- **`replacement::EntryFrame`** — the frame per pipeline iteration (CR 614.12 clause 1), computed only when a filter-scoped `affected_objects` asks. `gather::set_affects` and `restriction::is_prohibited` both read it, for an `EnterBattlefield` — which since RC-4b is also the zone change onto the battlefield, so the frame has one basis.
- **CR 614.17d** in its two printed shapes. "Can't enter the battlefield" watches the zone change, because a refused entry would strand the card in the battlefield zone with no permanent (`propose_entry` is still loud about that); "can't have counters put on it" is asked as the `AddCounters` it is (CR 122.6) at the moment an `EnterWith` would add them (`replacement::strip_prohibited_counters`), refusing the counters while the entry goes on — Melira's ruling. `cant-effects-architecture.md` §5.3.
- **CR 616.1b** — `Rewrite::EnterUnderControlOf(PlayerRef)`, its class derived from the rewrite so a card cannot file it under `Other`. "An opponent of your choice" with several opponents is a prompt to the effect's controller (`ChoiceKind::ChooseEnteringController`), made before the permanent enters — CR 614.12a's timing, claimed partially.
- **`AmountExpr::CountOf`** — its first static-context evaluator, over `battlefield_ids_ordered` at the current `layer_index`, memoized within the walk. The entering object is invisible to it because it is not on that list: §5a's Thassa boundary, structural.
- **§11 item 19** — CR 616.1 no longer prompts when every member of the bucket is an `EnterWith` whose applicability no `EnterMods` field can move; item 47 below has the three expiry conditions and the debug-build check. Containment Priest landed first, so the multi-candidate branch is still reached by a registered board.
- **`GameAction::EnterBattlefield` carries the zone change that brought the object** (`None` for a token), because CR 601's "was it cast" is unrecoverable a moment later; `EventPattern::EnterBattlefield { cast }` projects it.
- **Three cards.** Containment Priest and Dryad Arbor (stress pool), Keldon Warlord (`PERFORMANCE_POOL` 61 → 62). Wall of Stone gains the Wall subtype it always printed. Two fixes rode along: CR 205.1a in `land_types::apply_set_subtypes` — Blood Moon made Dryad Arbor a Mountain with no creature type; shown failing first, own commit — and the pool's color check reading color indicators (CR 204.1).
- **`targeting::object_matches_filter` takes one layer walk per filter** instead of one per leaf, and only when a leaf reads a characteristic; the entry path matches against the frame through `object_matches_filter_in_frame`.

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

- **`compute.rs::effect_applies_to` gates on the battlefield *zone*, not on `game.battlefield` membership.** `move_object` writes `obj.zone` before the `EnterBattlefield` performer builds the `PermanentState` — RC-2's one-`emit`-wide window — so an entering permanent is in the zone with no entry, and the stricter question kept CR 614.12 clause (3)'s "continuous effects that already exist and would apply to the object" away from every entry. Hidden zones are untouched: a card in hand still has `zone == Hand`, so nothing new reaches a library or a graveyard.
- **`gather`'s source 1a admits `ObjectSet::SourceOnly` only** (`SelfScope::EnteringSelf`). CR 614.12's parenthesis — "if they affect only that permanent (as opposed to a general subset of permanents that includes it)" — is a membership rule, so it is here rather than in RC-4's overlay. Without it an entering Orb of Dreams finds its own "Permanents enter tapped" through `set_affects`, which matches a `Filter` against any object in any zone, and taps itself.
- **RC-2's two known-wrong answers are now right.** The first is the one RC-3 named: a tapland under Blood Moon enters **untapped** (CR 305.7 strips the ability first), reachable in a fuzz game from cards already in the pool. The second was *not* Humility + Chainbreaker — that pair is a further consequence of the same line, real and tested, but RC-2 never listed it. RC-2's second was `default_enter_mods` and a filter-scoped Layer 4 effect, and RC-3's ledger misstated which direction changed (corrected 2026-09-02, RC-4): `default_enter_mods` reads *printed* loyalty and gates on the *effective* type, so "a planeswalker made one by a Layer 4 effect" has `loyalty: None` and enters with no counters either way. What the line changed is the inverse — a planeswalker whose type a filter-scoped effect **removes** now enters with **no** loyalty counters, because CR 306.5b gives the ability to "a planeswalker" and on that battlefield it is not one. `phase_rc4_integration_test::test_a_planeswalker_whose_type_a_filter_effect_removes_enters_with_no_loyalty` is the test; RC-4 routed the read through `compute_as_entering` so it is the CR 614.12 frame that answers, not `has_type` on a printed card.
- **`base_controller` grew a `resolving` leg.** `resolve_top_of_stack` takes the `StackEntry` before it resolves anything, so the battlefield and stack probes both miss for the whole resolution and the owner fallback answered — right for a land drop, wrong for a spell cast by a non-owner (CR 110.2b). RC-3 owns it because RC-3 is what makes `ObjectFilter::ByController` askable of an entering permanent. **It fixes a wrong answer no registered card can produce**: `check_cast_legality` refuses "another player's spell", so the test builds the owner/controller disagreement after an ordinary cast. Confirmed rather than argued — event streams at 40 games are **40/40 identical on both pools** with and without the leg. The trap it removes belongs to whoever relaxes that check — the Commander track and Phase RE both want to.
- **One card: Root Maze** (`{G}`, "Artifacts and lands enter tapped"), `PERFORMANCE_POOL` 60 → 61, plus `ObjectFilter::Or`. **Not for RC-3's own claim** — that one needed no new card and the argument is in the card's doc comment: Blood Moon and Humility were already in the pool, both are filter-scoped layer effects, and RC-2 had already put two permanents that enter modified in beside them, so the gate change widens a path the pool walks rather than opening one. Root Maze is for the gap RC-2 left by mistake (see the correction under RC-2 above): **CR 616.1's multi-candidate branch, reachable since RB merged and registered by nobody**, which is the identical failure RB shipped with Kalitas.

**The measurement, and it says the gate is free.** Interleaved A/B in one sitting against a same-day `main`, 50 games / seed 12345 / `--threads 1`, with a third binary carrying the gate change and *not* Root Maze so the engine and the pool could be separated (RC-2's lesson, applied):

| | main | RC-3, card unregistered | RC-3 shipped |
|---|---|---|---|
| performance walks | 99,877 | 100,556 (+0.7%) | 108,632 |
| performance frames/walk | 1.24 | 1.24 | 1.25 |
| stress walks | 93,245 | 98,099 (+5.2%) | 98,843 |
| stress frames/walk | 1.16 | 1.17 | 1.20 |
| ms/game, performance (median of 7) | 80.6 | 81.2 | 92.3 |
| ms/game, stress (median of 7) | 63.3 | 68.1 | 72.4 |

**`Frames/walk` was the number to watch and it did not move.** The fear was that admitting non-battlefield objects to filter matching would make `object_matches_filter` request other objects' frames, at the 5.2×–8.0× the ungated CR 604.2 existence check measures (`layers-architecture.md` §12). It does not, because the gate admits objects in the battlefield *zone* — outside a one-`emit`-wide window per entry, the same set as before. Normalised, ms per 1,000 layer walks moves **0.807 → 0.808 on `performance` (+0.1%) and 0.679 → 0.694 on `stress` (+2.2%)**, seven interleaved runs each. So the gate costs essentially nothing per walk; the `stress` walk count rises 5.2% because the games get *longer* (29.6 → 30.6 turns), which is the behaviour change paying for itself, not the predicate. The shipped column is not comparable to either, because the pool changed — §3.1.

**Event-stream check, and the shape of the answer is the point.** 40 games / seed 12345, `--dump-events`, canonicalized and diffed against a same-day `main` with Root Maze unregistered. **36 of 40 `performance` games and 34 of 40 `stress` games are byte-identical**; every one of the 10 that diverge has a Humility or a Blood Moon on the battlefield before the divergence and a permanent that enters modified entering under it. Unlike RC-1's pure deletion, a real behaviour change cascades — the whole-stream diff is meaningless after the first divergence — so the checkable claim is the **first** divergence per game, and all ten are Chainbreaker not dying to CR 704.5f, Adaptive Shimmerer hitting for 3 less (its three +1/+1 counters), or an Idyllic Beachfront that never needed untapping.

**Determinism holds.** `--games 200 --seed 12345 --threads 1`, three runs per pool, byte-identical outside `=== Timing ===`. `--threads 1` vs `--threads 8` produce identical engine-work counters on both pools, which is the check §82's stored fixtures made real. 0 errors, 0 panics, 0 turn-limit hits. The `stress` pool's known `TokenCeasedToExist` reordering (Deferred Migrations item 6) did **not** appear in two same-binary `--dump-events` runs at 40 games — it is a `HashMap`-order flake, so absence is not a fix and item 6 stands.

**One methodology note worth keeping.** A raw `diff` of two `--dump-events` files always fails: `ObjectId` is a per-process v4 UUID, so every line carrying one differs between runs of the *same* binary. Both the 8-hex short form and the full UUID form appear in the dump, and canonicalizing only the short form leaves four games per pool looking falsely divergent. Canonicalize both, per game, before concluding anything.

**What RC-3 did not build.** No `compute_as_entering`, no accessor pair, no `GameState` clone — all RC-4's, per `replacement-architecture.md` §11 item 5. RC-3 removed a gate; RC-4 builds the frame. **ATOM-614.12-001 stays unclaimed in full** and is a type-surface gap rather than a behaviour one: its board is Yixlid Jailer ("cards in graveyards lose all abilities") and `ObjectFilter` has no zone leaf, so the scenario is inexpressible. The engine's answer is right for the right reason — `move_object` has already left the graveyard when `gather` asks — but nothing can demonstrate it, so the Blood Moon test claims it partially and the atom waits for a zone-scoped filter.

**Status 2026-09-01: Phase RC-2 ✅ — entering the battlefield is a proposed event.** `replacement-architecture.md` §9's RC-2 subsection carries the eight findings; this is the state ledger. What landed:

- **`GameAction::EnterBattlefield { object, controller, mods }`**, proposed by `perform_action`'s `ZoneChange` arm the statement after it emits, and performed by `place_on_battlefield`. `EventPattern::EnterBattlefield`, `Rewrite::EnterWith(EnterMods)`, and `EnterMods { tapped, counters }` with a `merge` that is CR 616.1f's accumulation — status is `|=`, counters are `+` per kind.
- **`init_zone_state` is deleted.** It created the `PermanentState` from inside `move_object`, below the chokepoint. CR 110.2b's default controller — `GameState::resolving`'s only reader since RC-1 — is now `GameState::default_enter_controller`, read at the proposal. The field still has exactly one reader.
- **`init_etb_counters` is deleted.** CR 306.5b's loyalty is `GameState::default_enter_mods`, which *seeds the proposal* — so Phase RE's counter doublers will replace it with no further work. It is the first thing in the engine to be modelled as "what the rules say this permanent enters with" rather than as a direct write.
- **One emitter.** `GameEvent::PermanentEnteredBattlefield` was emitted by `stack.rs` for a resolving permanent spell and by `resolve.rs` for a token, and **not at all for a land drop** — the most frequent entry in the game. The performer emits it now, once, with the *effective* controller (CR 400.7a's Layer 2 row has already moved a stolen permanent spell by then).
- **`gather` grew source 1a and a gate leg**: the entering permanent itself, read off its effective ability list, ahead of the fast-path gate. Without it every "this permanent enters tapped" is dead text, because `replacement_ability_sources` is written by `register_static_effects` *inside* the performer. `chooser_for_event` reads CR 616.1's chooser off the proposal, because an entering permanent has no controller for `controller_or_owner` to find.
- **Two cards, one per half of `EnterMods`**: Idyllic Beachfront (CR 110.5b) and Chainbreaker (CR 122.6a). `PERFORMANCE_POOL` 57 → 59; `engineering-practices.md` §3's table re-recorded. A third, **Adaptive Shimmerer**, is registered into the stress pool alone: a 0/0 is the only board state that dies if CR 122.6a's counters arrive after the entry is observable, and until it existed that claim rested on a fixture the registered pool could not build (§3.3).
- **Event-stream check**: at 40 games / seed 12345, canonicalized and diffed against a same-day `main` binary with the new cards unregistered, **the only difference is one added `ETB` line per land drop** — 708 on `performance`, 699 on `stress`, zero deletions, zero reorderings.

**One performance finding, and it is about the measurement rather than the code.** RC-2 is the first phase to put static replacement sources into `PERFORMANCE_POOL`, so `gather`'s board-level fast path — "is anything here a replacement source" — is true from the first tapland onward and the sweep walked *every* permanent with a full `compute_characteristics` per proposed action. The exit-criterion A/B ran with the new cards unregistered and so measured the gate closed. A **per-permanent** gate (the same predicate one object at a time, exact by the same argument, byte-identical event streams) is worth **10.3% of total game time on `performance` and 9.2% on `stress`**. The lesson generalizes past this phase: *an A/B that neutralises a PR's card additions measures the engine and not the game*, and both numbers are worth having.

**Two known-wrong answers it leaves, both RC-3's one line** (`compute.rs`'s `game.battlefield.contains_key` gate in `effect_applies_to`): no filter-scoped `ContinuousEffect` reaches an entering permanent, so Blood Moon does not strip an entering tapland's "enters tapped" (the real ruling says it does), and `default_enter_mods` would miss a planeswalker made one by a filter-scoped Layer 4 effect. `tests/phase_rc_integration_test.rs::test_blood_moon_does_not_yet_strip_an_entering_taplands_ability` asserts the wrong answer on purpose so RC-3 has to flip it.

**And one claim it got wrong, corrected 2026-09-02 before RC-3's line was written.** RC-2 recorded in four places — this block, `phase_rc_cards.rs` twice, and `replacement-architecture.md` §9 finding 7 — that *no* `ObjectSet::Filter` effect reaches an entering permanent, and concluded that CR 616.1's multi-candidate branch was unreachable until RC-3. **Both halves are false and neither was ever true.** `ObjectSet` is matched by two different functions on two different paths: `compute.rs::effect_applies_to` (`:629`) gates `ContinuousEffect` on battlefield membership, while `gather::set_affects` → `GameState::object_matches_filter` matches `ReplacementDef.affected_objects` and `RestrictionDef.affected_objects` with **no gate**. Probed on `main`: a Root-Maze-shaped `ReplacementDef` (`Filter { ByType(Land) }`, `EnterWith(tapped)`) already taps an entering Forest, and with Idyllic Beachfront entering under it the pipeline produces two candidates and `ask_choose_replacement` fires. So the branch has been reachable since RB, one registered card away, and Root Maze / Kismet / Loxodon Gatekeeper / Frozen Aether were never blocked on RC-3. **The generalizable error is naming a mechanism by its type rather than by its call path** — "an `ObjectSet::Filter` effect" reads like one thing and is two — and it cost the project the same unreachable-branch gap twice, after RB shipped Kalitas alone.

**Status 2026-08-26: Phase RB ✅ — the CR 616.1 pipeline is live and three consumers use it.** All nine of `replacement-architecture.md` §9's RB items shipped. What landed:

- **`apply_replacements`** — §4.1's loop, inside `execute_actions` upstream of `perform_action`. CR 616.1a–f, CR 614.5's applied set keyed on effect *instance*, CR 616.2's re-gather, CR 614.6's dropped event, CR 614.17/17c's blocked path, §4.1a's rider timing. `ActionContext.dp` has its first reader.
- **`execute_actions` is three phases, and the split is CR 704.3**: decide for every batch member against one board, then perform, then run riders. That is where §4.3's CR 101.4 APNAP ordering lives — choices in APNAP order of chooser, performance in batch order, riders last (CR 615.5). It returns `Result<Vec<GameAction>, String>` again, and the SBA sweep is the customer that earned it back.
- **New event vocabulary**: `GameAction::Destroy { source: DestructionSource }` (the outer event; its performer proposes the inner `ZoneChange`), `AddCounters`, `RemoveCounters`. `CounterType::{Shield, Stun, Finality}`; `GameEvent::CountersChanged`.
- **Indestructible is a CR 614.17 "can't"**, checked ahead of the pipeline in `engine::replacement::is_blocked` (RB's name; RS-1 replaced it with `engine::restriction::is_prohibited`), not filtered at the two call sites that each held their own copy of it.
- **Three consumers**: counters (CR 122.1c/d/h, no card text, 164 cards), regeneration (CR 701.19a/b/c, `Primitive::Regenerate` + the rider CR 701.19a spells out), and Kalitas, Traitor of Ghet — the only RB card, chosen for difficulty.
- **Commander's two halves**: CR 704.6d as a state-based action and CR 903.9b as the rules' only `exempt_from_614_5` replacement. Since 2026-08-30 (`rb-review.md` H4) 704.6d's accepted moves join the SBA batch instead of being performed one at a time as they are offered — 704.3's "single event" is all of 704, not just 704.5, and each offer being its own event let a later owner decide against a board an earlier owner's move had changed.
- Eight primitives: four stubs given implementations (`Tap`, `AddCounters`, `RemoveCounters`, `CreateToken`) and four new (`Regenerate`, `CantBeRegenerated`, `RemoveFromCombat`, `RemoveAllDamage`).

**Five findings where the plan and the tree disagreed, all recorded in `replacement-architecture.md` §9:** `Uses::CounterBacked` does not survive the CR text; CR 122.1c's replacement half is restricted to destruction *by an effect*; `EventPattern` ships six arms and `Rewrite` two, because an arm the pipeline cannot apply is a card that silently does nothing; `ObjectSet` and `ZoneChangeCause` had to move into `types/` to keep the crate's layering; and §4.1's loop **hangs** on a declined `exempt_from_614_5` optional without a second set.

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

1) **`play_land` was a fourth, undocumented chokepoint bypass** — and the most frequent zone change in the game (18.1 per fuzz game). It wrote straight to `move_object`, which is why `ZoneChangeCause::PlayedAsLand` had zero call sites and nobody noticed. `play_land` now takes an `ActionContext` and proposes like everything else.
2) **`rollback_cast_to_hand`'s doc comment was aspirational.** It claimed a CR 601.2 rewind is unobservable, but `move_object` emitted a `ZoneChange` for it anyway. Moving emission into `perform_action` made the claim true.
3) **`GameEvent::PermanentLeftBattlefield` had no emitter anywhere.** Dead since it was written. Deleted rather than wired up — see above.

**Still owed, and newly recorded (see the numbered items below):** the CR 601.2a forward-announcement gap (item 5), and the SBA mutations that are not zone changes and have no `GameAction` variant to propose through (item 6).

The replacement pipeline is designed to sit inside `execute_action` at `engine/actions.rs:86-89`. Every mutating action must flow through there for replacements to observe them. Status:

1. **Zone-change migration — ✅ done (2026-04-18).** — archived.
    **Reachability (2026-09-03):** closed — struck 2026-04-18; predates the PR
    record.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Replacement
    effects (CR 614–616)" item 1.

2. **Open-coded zone bookkeeping — ✅ CLOSED 2026-08-25 (RA-3 ticket 7).** —
    archived.
    **Reachability (2026-09-03):** closed — RA-3, PR #60 (825c602).
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Replacement
    effects (CR 614–616)" item 2.

3. **Event-stream refit — ✅ CLOSED 2026-08-25 (Phase RA, three PRs).** —
    archived.
    **Reachability (2026-09-03):** closed — Phase RA, PRs #58, #59, #60.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Replacement
    effects (CR 614–616)" item 3.

4. **~~Unresolved architecture fork~~ — ✅ RESOLVED 2026-08-24 (owner decision):
    trigger detection is the performed-action event stream; the delta log is
    rejected.** — archived.
    **Reachability (2026-09-03):** closed — owner decision 2026-08-24, PR #54
    (ccd6ac1).
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Replacement
    effects (CR 614–616)" item 4.

5. **CR 601.2a announces a move that CR 601.2 may un-happen (recorded
    2026-08-25, RA-3).** — ✅ closed, archived.
    **Reachability (2026-09-03):** closed — by RC-4b (PR #87, 6541d0b), and
    nobody struck it: `put_on_stack.rs` moves the card with the silent `CAST-ROLLBACK`
    mover in both directions and `announce_zone_change` records the 601.2a move
    at 601.2i beside `SpellCast` (`cast.rs:269`), so a rewound cast leaves no
    forward event. …
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Replacement
    effects (CR 614–616)" item 5.

6. **State-based actions that mutate outside the chokepoint — partly closed by RB (2026-08-26).** `GameAction::AddCounters`/`RemoveCounters` exist now, so counters have a proposal vocabulary; CR 704.5q's *annihilation* still writes directly, because it removes two kinds at once and would have to join the SBA batch to propose. **A card went in ahead of that routing, deliberately (2026-09-01).** `battlegrowth` makes the annihilation sweep run in a fuzz game — 0 → 10 occurrences per 200 stress games — against the direct-write code exactly as it stands. The argument is Darksteel Myr's: coverage *before* a move is worth more than after, because the move is what needs a witness. It also gives the proposal vocabulary its first production reader — `CountersChanged` goes 0 → 82 per 200 games, so `perform_action`'s `AddCounters` arm and `gather`'s `EventSubject::Object` leg for it are no longer reached only by tests. Player loss, the Equipment detach and the token cease-to-exist are untouched. Original entry (recorded 2026-08-25, RA-3; extended 2026-08-26): CR 704.5q's counter annihilation, CR 704.5p's Equipment detach, and CR 704.5q's attachment catch-all write `PermanentState` fields directly; CR 704.5d's token cease-to-exist removes from `objects` directly. **~~Player loss (704.5a/b/c and CR 903.10a) is the fourth and the most consequential~~ — ✅ closed 2026-09-12 (RE-6).** The four loops are `GameAction::PlayerLoses` members of the CR 704.3 batch, deduped per player by the same subject-keyed collapse that dedupes the zone changes (CR 704.7), so `ATOM-704.7-001`'s board is a test (partial — the Archangel stands in for Lich's Mirror). The `!player_lost[i]` guard is now the gate ahead of the proposal, for CR 800.4k's reason and not by accident. *Original entry:* it wrote `player_lost[i]` and emitted `PlayerLost` without proposing anything, so CR 704.7's own worked example — Lich's Mirror replacing a loss that two rules would cause at once — could not be expressed at all. They are outside RA's exit criterion by construction — the criterion is about mutations CR 614 can observe, and there is no proposal vocabulary for a counter or an attachment yet. **RB item 5 adds `CounterType::{Shield, Stun, Finality}` and their effects, which is when counters need an `AddCounters` / `RemoveCounters` action;** the attachment pair wants one when Equip lands (CR 702.6) — **the attach half landed with LH-2 (2026-09-05): `GameAction::Attach`, performed through `GameState::attach`, emitting `Attached` on the transition; the SBA detach (704.5n/p and the catch-all) still calls `detach` directly and stays here.** Until then they are correctly outside, not accidentally: `GameAction`'s own comment block lists them as the variants to add as primitives arrive. A token ceasing to exist is genuinely not a zone change (CR 704.5d removes it from the game) and `TokenCeasedToExist` is the right event for it. **The token sweep was also a live determinism leak, found 2026-09-01 while measuring RC-1 and pre-existing on `main`; closed 2026-09-04 (83333e9):** `engine/sba.rs`'s 704.5d gather iterated `self.objects` — a `HashMap` — straight into an ordered `Vec`, so two tokens ceasing to exist in one sweep emitted `TokenCeasedToExist` in per-process order. It was invisible to `fuzz_games`' summary, which counts no such ordering, and showed up only in a `--dump-events` diff on the `stress` pool (adjacent lines that swap between runs of the *same* binary). CLAUDE.md's rule covers it — "same rule for any collection reaching a choice" — and here the collection reaches the event log instead, which is why it went unnoticed. **The fix is a key, not the routing.** `GameObject.zone_change_epoch` orders the gather: `move_object` stamps it on every move, one tick per move, so tokens leaving in one batch carry distinct ticks in batch order, and a token is only ever created *in* the battlefield zone, so one reaching the sweep has moved. Routing was the obvious fix and is the wrong one — a token ceasing to exist is not a zone change (CR 704.5d removes it from the game), so the sweep has nothing to propose and needed an order, not a batch. **Measured.** At 200 `stress` games / seed 12345 the run holds exactly one multi-token sweep — game 108, three of Kalitas's Zombies sacrificed at once — and three same-binary `--dump-events` runs of the pre-fix tree order them **three different ways**, while four runs of the fixed tree give one order, the order the three tokens left the battlefield. The two streams are otherwise identical line for line (101,214 lines), so no event count moves and `engineering-practices.md` §3's fixture table is confirmed rather than re-recorded. The other three direct writes are untouched, and so is the routing they are recorded for.

   **Reachability (2026-09-04):** unreachable — the one wrong part is fixed and
   the three that remain are right today. The CR 704.5d sweep's `HashMap` order
   is closed above (83333e9), so what is left of this item is the *routing*, and
   re-deriving it a day later changes none of the 2026-09-03 verdicts:
   `!player_lost[i]` makes player loss's outcome right, and the CR 704.7 Lich's
   Mirror board needs a `PlayerLoses` card, of which none is registered; 704.5p
   has no Equipment to detach; 704.5q's annihilation is exercised by
   Battlegrowth + Chainbreaker and gives the right counts through a direct
   write. Each is right for a reason a card can take away — a registered
   Lich's Mirror, an Equipment, a counter-doubler — and none of those is in
   either pool, so the routing is owed and not yet observable.

   **Sized:** the leak alone was ~5 lines and is **done** (2026-09-04) — the
   sort by `GameObject.zone_change_epoch` and the unit test the size named, in
   their own commits, which is what the estimate said it would take.
   Annihilation joining the SBA batch is ~40 lines (two
   `RemoveCounters` members per object). Player loss is RE-6's
   `GameAction::PlayerLoses` (`replacement-architecture.md` §9, sized
   2026-09-11 — the four loops become CR 704.3 batch members); the detach
   pair lands with Equip.

   **Phase (2026-09-19):** the counter half — CR 704.5q's annihilation — is TR-5's: two `RemoveCounters` proposals in the state-based batch, `CountersAnnihilated` deleted (`triggers-architecture.md` §3.3, §12); Protean Hydra's rulings are why it is a removal event. The detach and the token halves stay here.

7. **The early stack pop — ✅ DELETED 2026-09-01 (phase RC-1).** — archived.
    **Reachability (2026-09-03):** closed — RC-1, PR #80 (093e12a).
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Replacement
    effects (CR 614–616)" item 7.

8. **CR 608.3b is unimplemented: a permanent spell with an illegal target does not fizzle (found 2026-08-26).** `resolve_popped`'s fizzle check reads `extract_recipient(&entry.effect)`, which for an Aura is the *spell ability's* recipient — and an Aura has no spell ability, so `has_targets` is false and the check never runs. The Aura's actual target lives in `entry.chosen_targets` and is read later, at the attach step. CR 608.3b says such a spell "doesn't resolve. It is removed from the stack and put into its owner's graveyard." Today it resolves and enters the battlefield attached to a target that may no longer be legal. Predates RA and is unreachable in the current pool (no registered Aura is castable from hand — `put_on_stack.rs` never reads `enchant_filter`), but it is the other half of the fizzle path RA-3 just routed, so it is recorded here rather than in the RA ledger. **Sized 2026-09-01, and it is one helper, not three:** `oracle/mana_helpers.rs::spell_recipient`, the inline block in `engine/put_on_stack.rs`, and `engine/stack.rs::extract_recipient` are three copies of the same fourteen lines, computing a spell's recipient from its *effect* — so none can see an `enchant_filter` and all three must learn the Aura rule together or disagree. **Scheduled 2026-09-01 as Phase LH-1** (`layers-architecture.md` §13a), and deliberately *not* as its own PR: **zero registered cards carry an `enchant_filter`**, so the shared helper returns exactly what the three copies return today for every card that exists. Shipping it alone would put a new arm in front of the performance pool that no card can open -- the failure `engineering-practices.md` §3 is written against -- so it ships with Holy Strength, which makes it live. **A second blocker was found the same day and it is the larger one: fixing 608.3b still would not make an Aura registerable.** No `ObjectSet` names an Aura's host — `static_object_set` has two productive arms (`FilteredPermanents` → `Filter`, `Implicit` → `SourceOnly`), `Duration::WhileEnchanted` has no consumer, and `register_static_effects` runs inside `place_on_battlefield`, *before* `resolve_taken` attaches the Aura, so even `Fixed` has nothing to capture. Every faithful Aura's text is about its host, so the Aura half is a phase with a layers change in it. `engine/resolve.rs::attach_aura_on_etb` is meanwhile dead code — zero production callers, three unit tests — implementing CR 303.4g's choose-on-entry for a path no card can take; the live path is `engine/stack.rs`'s Aura branch.

   **Reachability (2026-09-03):** unreachable — `put_on_stack.rs` still never reads
   `enchant_filter`, so no Aura is castable and none is registered; the board
   needs LH-1 (`layers-architecture.md` §13a, ~730 additions), which carries
   this fix.

   **Sized:** above (2026-09-01) — one shared helper, not three; ships inside
   LH-1 with Holy Strength.

   **Closed 2026-09-04 — LH-1 ✅ (`layers/lh-1-host-addressable`).** One
   helper, `engine/targeting.rs::spell_recipient(&CardData)`: an Aura's
   recipient is its enchant ability (CR 303.4a), anything else's is the spell
   ability's through `effect_recipient`, which is the fourteen lines kept once
   and shared with `activate_ability`. The resolution does not derive a fourth
   time — `StackEntry` records the recipient the targets were chosen against,
   so CR 608.2b re-checks the question 601.2c asked. `attach_aura_on_etb` was
   deleted rather than made the path. The four tests that pinned this
   (`tests/phase_lh_integration_test.rs`) were shown failing against the
   pre-fix tree at the first assertion each makes.

   **Reachability (2026-09-04):** closed — LH-1. One honest residue: the
   fizzle arm is covered and reachable in principle (a Bolt in response),
   but in 200 stress games at seed 12345 Holy Strength was countered 3 times
   and fizzled **0**, with or without `--require` — the random agent does not
   answer an Aura by killing its target. CR 704.5m/n, the other half of the
   Aura row, went 0 → 23 in the same run.

### Found by a judge-corpus pass (2026-08-26)

Five card interactions were put to the engine by the owner. Two moved
`replacement-architecture.md` §5 (see §5b/§5c there); three are general and land
here. None is blocking RB.

9. **CR 110.2b: a permanent spell's default controller is its *caster* — ✅ FIXED
   2026-08-26 (found and fixed the same day).** `stack.rs` hands
   `get_effective_controller(spell)` to `place_on_battlefield`, so
   `PermanentState.controller` — which `compute.rs::base_controller` treats as
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
   atom's own scenario is the characteristics clause (Deathlace recoloring a
   creature spell) and stays open.

   Still owed from this item: carrying the caster onto the *permanent*, which
   customer 3 below needs at ETB-trigger time and which is RC's to place.

   **Three unrelated customers for one fact — record it (updated 2026-08-26).**

   1. **CR 110.2b itself**, above: the default controller *is* the caster, so the
      rule cannot be implemented without the fact.
   2. **Uphill Battle** ("Creatures played by your opponents enter tapped") — a
      CR 614 replacement whose predicate is who *cast* the spell, not who
      controls it. Weak on its own: `o:"played by"` is **1 card in all of
      Magic**, `o:"cast by"` is 2. The "played by" `ObjectFilter` leaf stays
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

   **Landed 2026-09-19 (TR-1):** `PermanentState.cast: Option<CastFacts { by, from }>`
   — CR 400.7d's two facts, written once by `place_on_battlefield` off
   `ResolvingObject.cast_from` (carried beside `default_controller` for the
   same reason), `None` for a land drop, a token or an effect's entry.
   `TriggerEvent::EntersBattlefield { cast }` reads it;
   `a_permanent_remembers_whether_it_was_cast` is the test. "If you cast it"
   as an intervening "if" is a `Condition` leaf for the first card that
   prints it.

   **Reachability (2026-09-19):** closed — TR-1; the rule itself closed 2026-08-26.

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

    **Sized 2026-09-03, and scheduled with CV-1b as one PR: after CV-2,
    before RS-2.** The `ObjectId` stays — targets, attachments, events and
    the fuzz log all key on it, and re-keying is the whole engine — so what
    breaks is every *reference* made before the move. Three kinds hold one:

    - **`ObjectSet::Fixed` rows**, in all three registries. They share
      `DurationRegistry`, so one subject-keyed `retain` serves them: prune
      the mover from every `Fixed` set, drop a row whose set empties (a
      two-target pump keeps applying to the target that stayed), and keep
      `remove_by_source` beside it — `copy-effects-architecture.md` §5.3's
      "alongside" fact, which is why Clone must be on the board first.
      Called from `move_object` for every zone pair, through the continuous
      registry's `mutating` so the layer memo sees it.
    - **`StackEntry.chosen_targets`**, which CR 608.2b re-validates by id
      today, so a spell finds a target that died and came back. A parallel
      `target_epochs` on the entry, set where the targets are chosen and
      compared in `is_single_target_legal`; `ResolvedTarget::Object` has 38
      match sites, so the type does not change for this.
    - **Attachments**, already cleaned by `cleanup_zone_state`.

    The exceptions 400.7a–c are one carve-out: a move from the stack to the
    battlefield prunes nothing keyed on the mover, which is what keeps item
    9's stolen spell stolen and Xu-Ifit's rider on the permanent. 400.7d–k
    are trigger rules (e, f — item 6's, where the LKI frame finds the new
    object) and cast-permission rules (g–i — the cast-permission entry's);
    they stay there. **Content-neutral on both pools today**: no registered
    card returns an object (`ReturnToBattlefield` is a stub), so the fuzz
    rows should not move and the PR says so — the rule's tests are built by
    hand around `move_object`. ~500 additions for the rule, its tests and
    this entry's closure; CV-1b adds ~400 — `Duration::Indefinite`'s one
    consumer, Dimir Doppelganger, its tests and a `PERFORMANCE_POOL` seat,
    since an indefinite row is a new engine path. One PR under the band,
    A/B'd as two arms (the rule alone, then the card) the way the Everywhere
    PR was.

    **Reachability (2026-09-03):** unreachable — no registered card returns an
    object to any zone (`ReturnToHand`, `ReturnToBattlefield` and the rest of
    the zone-moving primitives are the stub arm at `resolve.rs:866`) and nothing
    recasts a countered spell, so no `Fixed` row and no `chosen_targets` entry
    ever meets its object after a zone change. The CV-1 review's C6 (absorbed
    below) called this "a live wrong answer" because Giant Growth is in
    `PERFORMANCE_POOL`; it is not — that reading needed a return path, and there
    is none. A reachability claim can go stale in either direction.

    **Sized:** above (2026-09-03) — ~500 for the rule and ~400 for CV-1b, one PR
    after CV-2.

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

    **Landed in part 2026-09-19 (TR-1):** CR 605.1b's half.
    `engine::triggers::is_mana_ability(def)` derives a *triggered* mana
    ability from the def — no instance of "target", every arm `ManaAdded`, a
    `ProduceMana` in the effect — and the dispatcher resolves it at once
    (CR 605.4a), inside the CR 601.2g window when the mana was made there;
    Wild Growth is the card and `wild_growth_adds_its_mana_at_once_without_the_stack`
    the test. CR 605.5a is the same derivation refusing: a target or another
    event queues it like any trigger.

    **Reachability (2026-09-19):** unreachable — the *activated* half: every
    registered activated ability's tag agrees with CR 605.1a (Sol Ring, the
    lands and Citanul's granted body are `Mana` with no target; Merfolk
    Thaumaturgist and Chainbreaker are `Activated` and add no mana), and the
    one Layer 6 grant in the pool carries a mana body.

    **Sized:** derive `is_mana_ability` for an activated def at the three
    dispatch sites (`mana.rs`, `priority.rs`, `activatable_abilities`)
    instead of reading the tag, ~60–100 lines plus the Toph-named test.

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

    **Reachability (2026-09-03):** unreachable — Layer 1 landed with CV-1 and
    Station has not (`station` appears nowhere in `src/`); no registered
    creature has `toughness == None` today (Sutured Ghoul is registered at a
    printed 0/0, item 59, and Keldon Warlord's CDA fills its box).

    **Sized:** the two `unwrap_or(0)` reads (`sba.rs:242`, `:254`)
    become an explicit `None` arm whose answer CR 721.2c decides, ~10 lines plus
    a test; lands with the first Station card (Phase 8).

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
    restriction. ~~It belongs with the cost-modification phase, the other
    CR 613.11 consumer (Before Layers item 3).~~ **Re-homed 2026-09-07:**
    `cost-architecture.md` §3.9 owns CR 613.11's *cost* half only and split
    this out — a player-scoped value applied in timestamp order is the rule's
    other sentence, and shares its surface with `max_hand_size` and player
    hexproof, which `backlog.md` §2.15 holds as one entry. That entry owns
    it. Two corpus atoms still carry the `L15` ticket: `ATOM-601.3-001` and
    `ATOM-613.10-001`; `ATOM-613.11-001/002` were re-filed to CM-1.

    **Reachability (2026-09-03):** nothing owed here — a record.
    `lands_per_turn` is still a raw field (`player.rs:23`) and no registered
    card changes it.

    **Sized:** the field becomes a computed player-scoped value inside
    `backlog.md` §2.15's surface, ~40 lines of its small-to-medium; the two
    remaining `L15` atoms move with it.

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

    **Reachability (2026-09-03):** unreachable — no registered card regenerates,
    forbids regeneration or makes a shield counter (`Regenerate`,
    `CantBeRegenerated` and `CounterType::Shield` appear in no card file), so
    the Wrath-plus-shield board cannot be built.

    **Sized:** a resolution-scoped `Duration` variant retired by a
    hook at the end of the top-level `resolve_effect` call, ~60–80 lines in
    `duration_registry.rs` and `resolve.rs`, plus a one-argument change per
    card; RS-2 owns it as the next phase touching `Primitive::Restrict`.

15. **Four `KeywordFlag` variants are constructible and enforced nowhere.**
    `Hexproof`, `Shroud`, `Menace` and `Intimidate`. The only `KeywordFlag::`
    references to them in `src/` outside the enum definition are `ui/display.rs`
    (two of the four) and `layers/land_types.rs`'s hexproof insertion. A card
    carrying one can be registered today and will quietly do nothing.
    `engine/targeting.rs:39` still carries the `T22` TODO that would fix two of
    them; `Menace` and `Intimidate` are `T21b`'s. All four are Tier 1a/1d in
    `cant-effects-architecture.md` §2.3 and land in RS-2 / RS-3.

    **Reachability (2026-09-03):** unreachable — no registered card sets any of
    the four flags (Sigarda's hexproof is deliberately left off,
    `phase_rs_cards.rs:81`).

    **Sized:** hexproof and shroud are ~40 lines at
    `targeting.rs:302` plus `enumerate_legal_selections`, RS-2 (Tier 1a/1d; the
    same change as "Before card breadth" item 7); menace and intimidate are ~60
    lines in combat validation, RS-3.

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
    `EffectOrigin::StaticAbility`, so CR 604.2 stops applying them as soon as the
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

    **Reachability (2026-09-03):** unreachable (the Layer 3 residual; both scans
    closed with CV-1, PR #89, 650a263) — nothing produces a Layer 3 effect, so
    the gates' missing leg has no route to it.

    **Sized:** one leg on each gate (`gather.rs`, `predicate.rs`),
    ~20 lines each, in the PR that gives Layer 3 a producer.

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

    **Reachability (2026-09-03):** unreachable — `Duration::Indefinite` has no
    consumer in `src/cards` (0 sites), so every re-copy's litter retires at the
    next cleanup; CV-1b is the trigger, as the entry says.

### Found by CV-1's reachability mode (2026-09-02)

16c. ~~**Spells resolve without being paid for, and `cast_spell` is one missing
    `rollback_cast_to_hand` away from correct**~~ — ✅ closed, archived.
    **Reachability (2026-09-03):** closed — PR #90 (6dedaf8), and the clamp in
    PR #94 (0ed6836).
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by CV-1's
    reachability mode (2026-09-02)" item 16c.

### Found by the CV-1 review (2026-09-02; absorbed 2026-09-03)

Six items sat in `plans/handoffs/cv-1-review.md` under "open, with an owner"
after CV-1 shipped, in a file whose contract says it is deleted when the work
lands. Absorbed here and the file deleted; the corrections (A1–A6) and answers
(B1–B5) it also held landed with PR #89 and are in the commit record. Three need
no item: **C2** (ATOM-702.131b-002, Ascend mid-resolution) found the mechanism
already right — nothing memoizes across a resolution — and its two missing
features have owners, the city's blessing designation at Phase 8 and conditional
statics at "Before Layers" item 7f (closed 2026-09-06); **C4** (reachability is thin) is done —
`--require` shipped with CV-1, Everywhere landed as PR #91, and the defect it
found is 16c; **C6** (promote CR 400.7?) was answered by PR #93 — main item 10
is CV-1b's first commit, not a critical-path row — and its claim that Giant
Growth already makes item 10 a live wrong answer is corrected under item 10: no
registered card returns an object.

66. **`--dump-events` cannot see a characteristic change, and one flag would
    (C1).** The event stream records performed game actions, which is CR 603.2's
    question and the right scope; a copy row is not one. A phase whose whole
    output is a characteristic change therefore has no differential instrument —
    CV-1 A/B'd on event streams and could only see the pool change size. Not a
    delta log and not a reopening of the 2026-08-24 fork: a **`--dump-state`**
    flag hashes every permanent's `EffectiveCharacteristics` in
    `battlefield_ids_ordered` order at each priority boundary and emits one line
    per boundary, recomputed from state, so it cannot drift.

    **Reachability (2026-09-03):** nothing owed to correctness — tooling. Its
    second customer is main item 10, whose failure is a state error no event
    diff sees.

    **Sized:** ~60 lines in `fuzz_games`, behind a flag, nothing on
    `GameState`; owner CV-2, the first phase with something to validate against
    it.

67. **~~`CopiableValues::apply_to` deep-clones a `Vec<AbilityDef>` into every
    frame of every copied object (C5)~~ ✅ CLOSED 2026-09-16 (A4f, PR #157) —
    the ability list is behind an `Arc`.** `CardData.abilities`,
    `EffectiveCharacteristics.abilities` and `CopiableValues.abilities` are
    `Arc<Vec<AbilityDef>>`, `get_effective_abilities` returns the memoized
    frame's own `Arc`, and the seven writers copy on write, so a frame no
    layer writes shares the card's allocation. Callgrind at four seats on
    `stress`, both arms on the same pool: the `to_vec` row (21.2% of `main`)
    is gone and the game fell 31.4% in instructions, 30.6% in native CPU
    (`layers-architecture.md` §12, the 2026-09-16 re-read).
    → `plans/archive/codebase-state-closed.md`.

    **Reachability (2026-09-16):** closed — landed, answer-preserving; the
    A/B read `IDENTICAL` on every counter, both pools, two seats and four.

68. **A `ChoiceKind` is named for the question, never for the card (C3).** Asked
    whether the vocabulary should be swept and designed ahead; no — appending a
    variant is O(1) (`ChooseCopySource` cost 14 lines across two files, both DPs
    have catch-alls, nothing is serialized), and a vocabulary derived from a
    word search over the corpus would be partitioned by English, which is the
    clause-is-the-wrong-unit mistake again. The one defect a late variant can
    carry that is expensive to fix is its name: `ChooseCopySource`, not
    `Cytoshape`; `ChooseEnteringController`, not `CR616_1b`. A DP heuristic
    keyed on the variant must be right for every card that ever produces it —
    and *reusing* a variant that fits mechanically and lies semantically
    (`SelectRecipients` for Cytoshape) is the mistake, not adding one.

    **Reachability (2026-09-03):** nothing owed — a naming rule, recorded;
    review enforces it.

    **Sized:** none.

69. **~~Every performance number this project owns is two-player, and v1's
    profile is four~~ ✅ CLOSED 2026-09-15 (the post-RE audit, pass 3) —
    measured.** `fuzz_games --players N` landed with RE-7 (2026-09-13) and
    every `fuzz-record.md` block since carries two- and four-seat columns; the
    Commander-scale board this item still owed — four 100-card decks at 40
    life — is measured in items 138 (decisions and cost) and 143 (the clone):
    83-turn games holding ~30 permanents at a priority prompt, 1.7–1.8× the
    CPU of a 60-card game, most of it length rather than board.
    → `plans/archive/codebase-state-closed.md`.

    **Reachability (2026-09-15):** closed — measured; the levers this item
    said to re-measure first are re-ranked against that board in item 138.

### Found by the Everywhere pool change (2026-09-03)

16d. **An any-color mana base turned the random agent's uniform tap into a
    five-sided die, and the agent now taps for the pip it owes (2026-09-03,
    `pool/everywhere-land`).** With fourteen Everywheres in every deck, a
    uniform pick among (land, ability) pairs paid `{1}{G}{U}` from three taps
    about one time in five; a failed payment rewinds with the lands still
    tapped (CR 732.1's "may not reverse" branch, `backlog.md` §2.18); land
    taps per spell cast went 3.86 → 7.66 (40-game `performance` dumps) and
    spells per game 27.9 → 22.6 with them. `RandomDecisionProvider` now
    prefers a source that makes a type an unpaid pip accepts, least flexible
    source first, declines when a pip is owed that nothing offered can make,
    and caps each type in the generic split at its pool amount less the pips
    of that type the same payment pays — policy, not payment law: the prompt
    still offers every legal option and the engine still validates the
    answer. Taps per cast read **3.18**, spells per game 25.0 / 24.8, and
    walks per game fall below `main` on `performance`, because every wasted
    tap was a window iteration that walked the whole board.
    `fuzz-record.md` has the three sittings.

    **What this entry first claimed, and why it was wrong.** The first draft
    read "`--require Cytoshape` resolves 212 times against 390 with seeding,
    and the halving is the agent". Most of it was copies: a deck seeded to
    G/U drew its 36 nonlands from 21 cards and held 2.7 Cytoshapes, the whole
    pool gives 1.7, and a throwaway build forcing exactly one copy per deck
    read 149 / 141 / 133 for seeded / unseeded / unseeded with the agent
    fixed. The seeding was worth about 5% per copy and the agent's fix none
    of it — a forced card's count is bounded by drawing the copy and choosing
    it among everything else castable, not by paying. `--require` now prints
    `copies/deck` beside every count so the next reader cannot make the same
    comparison.

    **Still owed in §2.18:** the CR 732.1 reversal prompt. The engine-side
    split clamp landed 2026-09-03 and took this entry's DP-side clamp with it
    — the prompt's maxima are now what they were computing, so the agent takes
    them at face value and the games are byte-identical (16c's closing entry).
    The tap preference above is untouched: choosing *which land to tap* is
    policy, and only the split was payment law.

    **Reachability (2026-09-03):** nothing owed here — a record of a harness
    change; the CR 732.1 reversal prompt it leaves is `backlog.md` §2.18's,
    sized there as small.

    **Sized:** none here.

16e. **96% of layer walks repeat an object nothing has touched, so item 7's
    memoization half is split out as 7a and moved ahead of triggers
    (2026-09-03, owner's call, on the Everywhere PR).** — ✅ closed, archived.
    **Reachability (2026-09-03):** closed — critical-path item 7a, PR #92
    (cbd7e59).
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by the
    Everywhere pool change (2026-09-03)" item 16e.

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

    **Reachability (2026-09-03):** unreachable — still exactly two producers of
    registry rows, `Primitive::Regenerate` (`UntilEndOfTurn`) and
    `Primitive::Restrict` (a duration the card writes), and no registered card
    uses either; static replacement and restriction abilities never register a
    row.

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

    **Reachability (2026-09-03):** nothing owed — a pointer for the combat
    phase.

    **Sized:** none.

19. **`zone_change_epoch` has one consumer and two counters behind it
    (`rb-review.md` E3).** `GameObject.zone_change_epoch` plus
    `GameState::next_zone_change_epoch` and the SBA-check tick exist for
    CR 704.6d alone — "was put into that zone since the last time state-based
    actions were checked", which nothing about an object answers a moment later.
    Recording an unrecoverable fact is the right call and item 10 wants the same
    tick for CR 400.7, so this is not debt to pay down; it is a **re-confirm
    when item 10 lands**. If 400.7 arrives and does *not* use the field, three
    pieces of plumbing serve one rule and the question becomes live again.

    **Reachability (2026-09-03):** nothing owed — re-confirm when item 10 lands.

    **Sized:** none.

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

    **Reachability (2026-09-03):** unreachable — RS-1 landed, so the
    prerequisite is met, and `turns.rs:137` still zeroes unconditionally; none
    of the seven cards is registered.

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

    **Reachability (2026-09-03):** nothing owed.

    **Sized:** none.

### Found by the theme D+F+H pass (2026-08-30)

22. **`gather`'s counter fast path is a full battlefield scan, and it stays one
    until CR 704.5q joins the chokepoint (`rb-review.md` D2).**
    `any_replacement_counter` walks every `PermanentState` on **every
    proposed action**, checking three counter kinds. Cheap at 15 permanents and
    it has never shown up in a profile, but it is the one part of the fast path
    that is not O(1). The review's correction is right and was recorded in the
    code: the old comment defended the scan by saying a cache would drift,
    which is true of a *count* and false of a **set** maintained the way
    `replacement_ability_sources` is. **What actually blocks the set is that
    counters have three mutation sites, not two** —
    `GameState::add_counters`, `perform_action`'s `RemoveCounters` arm, and CR
    704.5q's +1/+1 / -1/-1 annihilation, which writes `PermanentState`
    directly (item 6). A set maintained at a chokepoint that does not exist is
    exactly the drift, and it reads as a card silently doing nothing.
    **Sized:** a `HashSet<ObjectId>` beside `replacement_ability_sources`,
    inserted in `add_counters` and re-checked on removal — about 15 lines, and
    it becomes *sound* the day item 6 routes 704.5q through `execute_actions`.
    **Measure first** (`fuzz_games --games 200 --seed 12345` against the
    13.04 ms/game RB baseline); do not pay for it before the pool grows.

    **Reachability (2026-09-03):** reachable — not wrong; perf only.
    `any_replacement_counter` (`gather.rs:630`) runs on every proposed action of
    every game and has measured flat at pool size.

23. **Counters cannot exist on an object that is not on the battlefield, and
    three separate rules want them to (`rb-review.md` F1).** The map lives on
    `PermanentState` and `perform_action`'s `AddCounters` arm errors for
    anything else. CR 122.1a and 122.1b are both written for "a card in a zone
    other than the battlefield"; **71 suspend cards** (CR 702.62) put time
    counters on a card in exile; and CR 122.2's exception — "counters remain on
    this as it moves to any zone other than a player's hand or library" — is
    printed on two cards, Skullbriar, the Walking Grave and Me, the Immortal.
    **Sized:** move `counters: HashMap<CounterType, CounterStack>` from
    `PermanentState` to `GameObject`. 12 direct `.counters` sites outside
    `src/cards`, plus the `add_counters` / `remove_counters` / `counter_count`
    accessors — contained. The part that needs thought is
    `CounterStack.timestamp`: CR 613.7c timestamps a counter as it is put on,
    and a timestamp only means something where the layer system reads it, so
    the off-battlefield case needs an answer rather than a copy of the
    on-battlefield one. Skullbriar then costs a predicate at `move_object`'s
    clear point. **Trigger:** the first suspend card, or the first CR 122.1b
    keyword counter on a card outside the battlefield. Not Skullbriar alone —
    two cards do not earn the move. → `replacement-architecture.md` §11 item 10.

    **Reachability (2026-09-03):** unreachable — no registered card puts a
    counter on a card outside the battlefield; suspend, off-battlefield keyword
    counters and Skullbriar are all unregistered.

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

    **Reachability (2026-09-03):** unreachable — no registered card uses
    `Effect::Optional` (it is `Err("not yet implemented")` at `resolve.rs:119`)
    or an outcome conditional.

    **Sized:** `Effect::Optional` is a yes/no `DecisionProvider`
    prompt around the inner effect, ~60 lines in `resolve.rs`; the outcome
    conditional (`Effect::IfYouDo { did, didnt }`) ~80 more; both land with the
    first "if you do" card, which RD's prevention riders or Phase 8 will bring.

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

    **Reachability (2026-09-08, re-derived at RD-1's close):** still
    unreachable — `Rewrite::Retarget` is RD-4's and does not exist. RD-1 added
    `Rewrite::Amount`, which rewrites the amount and never the subject, so it
    cannot move a chooser either. Revisit at RD-4, not before.

    **Reachability (2026-09-09, re-derived at RD-2's close):** still
    unreachable. RD-2 changed the loop's *unit* — phase 1 now runs one whole
    CR 616.1f loop per subject group rather than per member, still in APNAP
    order of chooser — and not the order choosers are asked in; nothing in a
    group's loop can move another group's chooser, because no rewrite changes a
    subject. One thing to carry to RD-4: the group form makes 101.4d's restart
    *more* tractable, not less — the outer round-robin is already over
    self-contained groups, so interleaving is a change to that loop rather than
    to `apply_replacements`. Revisit at RD-4.

    **Reachability (2026-09-09, re-derived at RD-4's close):
    reachable at last, and still not reached.** `Rewrite::Retarget` is the
    rewrite every previous re-derivation said did not exist, and it *does*
    change a member's subject — so it can change who CR 616.1 asks. The board:
    an opponent's Pariah enchanting a creature **you** control moves damage
    aimed at them onto an object you control, and your own group may already
    have run to completion. That is CR 101.4d exactly.

    **No card in either pool builds it**, and the reason is structural rather
    than lucky: both printed redirects here are `PlayerSet::You`-scoped, so the
    crossing needs an Aura played on an opponent's creature, which a random
    agent does only by accident. The sizing is unchanged — phase 1 turned
    inside out, a per-member state machine under an outer APNAP round-robin.
    **Revisit at Phase 6**, where triggers make the other half of CR 101.4 live
    anyway; not at RD, which is finished.

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

    **Reachability (2026-09-03):** unreachable — re-checked: the SBA sweep
    dedupes per object, combat and untap batches are per-object unique, RC-5's
    auxiliary batch names each object once by CR 614.13b, and no registered
    spell proposes one object twice.

### Found by a rider read-through (2026-08-30)

Three items in the seam between `ReplacementDef.then` and `engine::resolve`.
None is reachable today and none is a defect in what RB shipped; each is a place
where the *next* phase will find a channel missing rather than wrong. §4.1a
settled *when* a rider runs — these are about *what it can reach*, which that
section never asked.

27. ~~**A rider cannot name the affected player, because `Rider` flattens the
    subject to an object.**~~ **Closed by RD-1 (2026-09-08)**, exactly as
    sized: `Rider.subject` is an `EventSubject` and `resolve_rider` emits
    `ResolvedTarget::Player`. Angel of Suffering's "mill twice that many cards"
    is the registered card that exercises it, and it is registered in the
    stress pool.

    **Refined by RD-4 (2026-09-09).** The subject a rider carries is now the
    subject of the first *member* the application touched, read before that
    member's own rewrite — not the group's key. CR 614.9 can move a member's
    subject mid-loop, so the two came apart; CR 615.5's "that much" is about
    the event the effect replaced, which is the pre-rewrite reading.
    `replacement-architecture.md` §11 item 35.

    The original entry follows.

    **A rider cannot name the affected player, because `Rider` flattens the
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

    **Reachability (2026-09-03):** unreachable — no registered card carries a
    rider on a player-subject event; Kalitas's rider rides a `ZoneChange`, and
    Notion Thief is a test fixture only.

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

    **Reachability (2026-09-03):** unreachable — no registered rider is phrased
    over a set, and `regeneration_rider`'s unread filter is inert rather than
    wrong.

    **Sized:** counted 2026-09-03: `resolve_primitive` has 41 arms
    now (29 when this was written). The shared route — one `recipients_for(ctx,
    recipient)` that resolves `EffectRecipient::Choose` and `FilteredPermanents`
    at resolution time, with the object-reaching arms reading `ctx.targets`
    through it — is ~150–250 lines; lands with the first set-phrased rider
    (RD/RE) or the first mass-removal spell, whichever comes first.

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

    **Reachability (2026-09-03):** unreachable — nothing decomposes:
    `GameAction::DrawCard { player }` still has no count field, so every nested
    call is containment.

    **Owner (2026-09-11):** RE-2 — `replacement-architecture.md` §9, RE
    decision 1; the outer `DrawCards` performer is the producer.

    **~~Closed 2026-09-11 by RE-2.~~** `GameAction::DrawCards`'s performer hands
    each of its `n` inner `DrawCard`s the applied set the outer's own CR 616.1
    loop accumulated, which took four signatures rather than the one call site
    this item sized: `apply_replacements` returns the group's applied set,
    `execute_batch_inner` carries it per member into phase 2, `perform_action`
    takes it, and `execute_actions_inheriting` hands it back down. The
    regression is `test_two_thought_reflections_draw_four_not_infinity` — Thought
    Reflection and not the Teferi §3.2d named, because Teferi's Ageless Insight
    is legendary. **The sizing's one thing worth keeping**: it called the failure
    a hang, and it is worse than that — the recursion overflows the stack, which
    aborts the whole test binary rather than one test. The bound is in the
    provider, not the engine.

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
    to `PermanentState` on resolution is a shape this codebase already has,
    so this rides it rather than re-threading anything — and the rail survives
    RC-2's ETB rewrite either way, which is why RC is not the gate. The part
    that needs thought is what a spent *object* is once it has left: Fling
    reads the sacrificed creature's power out of a graveyard, which is LKI, and
    LKI formalization belongs to item 6. **Trigger: capture is independent of
    every scheduled phase and cheap whenever; the design constraint lands at
    CV.** Recording it early is the cheap half — the mana atoms exist at
    payment time and nowhere afterward.
    → `cr-coverage-audit.md` §5.1.

    **Reachability (2026-09-03):** unreachable — nothing reads it: no sunburst,
    no spell copy (CV-4) and no CR 700.14 card is registered.

31. **`StackEntry.chosen_modes` is dead scaffolding — no writer, no reader
    (found 2026-08-31 by the D3b slice-1 triage).** Declared at
    `state/game_state.rs:33` as `Vec<usize>`, constructed `Vec::new()` at all
    twelve sites (`put_on_stack.rs` ×2, `stack.rs` ×5, `game_state.rs`, `ui/display.rs`
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

    **Reachability (2026-09-03):** unreachable — nothing is modal; the field is
    still `Vec::new()` at every construction site and read nowhere.

    **Sized:** owned by `backlog.md` §2.7, one phase; the field
    costs nothing until then.

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

    **Reachability (2026-09-03):** reachable — not wrong in play. Every fuzz
    deck starts unvalidated, and `random_deck` draws its 36 nonlands with
    replacement from ~60 names, so a few percent of decks exceed `max_copies`;
    CR 100.2a is deck construction, and no rule of play reads it.

    **Sized:** one `validate(&Decklist, &DeckLimits)` ~40 lines
    plus tests (`backlog.md` §2.13: tiny), called from `Game::new`, and a 4-of
    cap in `random_deck` if the harness is to be held to it; lands with the
    `GameConfig::commander()` constructor, whose 100-card singleton limit is the
    first one a fuzz deck would actually break.

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

    **Reachability (2026-09-03):** unreachable — dormant at both ends,
    re-verified: no production caller of `SpendContext`, `pay_with_plan` or
    `drain_spent_grants` outside `types/mana.rs`, and every `ProduceMana` still
    passes `special: vec![]`.

    **Sized:** T12c — `pay_single_cost`'s Mana arm builds a
    `SpendContext` and routes `pay_with_plan`, ~150–250 lines; T12d is the two
    cards; lands with item 30's capture PR or the first restricted-mana card,
    whichever first.

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

    **Reachability (2026-09-03):** nothing owed — item 17's hook is one `retain`
    on the generic; delayed triggers need a `DurationRow` impl, critical-path
    item 6's.

    **Sized:** none here.

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
    §3.3, which generalizes both halves into a rule.

    **Sized as two cards and zero engine change; the second half of that was
    wrong.** Both are non-legendary enchantments whose battlefield-side
    replacement is the shape Kalitas already proves — `EventPattern::ZoneChange` + `ObjectSet::Filter` +
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
    control, and `ObjectFilter` had only `ByController`. The two answers
    diverge whenever control has moved, which the registered pool can already
    reach: Act of Treason steals a creature, it dies, and it goes to the
    graveyard of the player who owns it. Shipping it as `ByController` would
    have been a *different card*, not a narrower one. `ObjectFilter::ByOwner`
    is new — one variant and two match arms (`compute.rs`, `targeting.rs`), with
    ownership read off the `GameObject` for the reason `Token` gives.

    **"From anywhere" shipped literal, after a correction in review.** Both cards
    were first written `from: Battlefield`, on the argument that
    `ObjectSet::Filter` carries an `ObjectFilter` and a card on the stack is
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

    **color is not derived from mana cost, and that is now guarded rather than
    fixed** (raised in review 2026-08-31). CR 202.2 makes a card's colors the
    colors of its mana cost, and `CardDataBuilder::color()` restates it by hand
    at **52** call sites. Measured: **0 of 58** registered cards disagree, so
    there is no bug today — only redundancy and drift risk.

    Deriving it is one small PR and *not* a deletion: CR 202.2e's color
    indicator is printed data no mana cost implies (Dryad Arbor is a green land
    with no mana cost, Ancestral Vision is blue with none), so the builder needs
    an override, and CR 702.114a's devoid is a **CDA** that belongs in Layer 5
    rather than in `CardData`. The minimal shape computes `colors` inside
    `build()`, so no reader changes.

    **Deadline: before Phase 8 breadth**, when 52 sites become 500+. The error
    class that will actually bite is **hybrid** — a `{W/U}` card is *both*
    colors (CR 202.2d) and that is what a human writing `.color()` gets wrong.
    The registry has no hybrid card yet. Until then
    `card_pool_lowering_test::test_every_registered_cards_color_matches_its_mana_cost`
    is the guard that makes deferring safe, and it was verified to fail on an
    injected miscolor rather than merely to pass.

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

    **Reachability (2026-09-03):** unreachable (three residuals; the branch
    itself closed 2026-08-31, PR #77, ac6e071) — (a) color derivation: 0 of 76
    registered cards disagree, guarded in CI by
    `test_every_registered_cards_color_matches_its_mana_cost`; (b)
    `PlayerRef::Owner` still resolves to the source's owner in `compute.rs:810`
    and to the tested object's in `targeting.rs:308`, and no registered filter
    spells `PlayerRef::Owner` (0 sites in `src/cards`); (c) Layer 2's second
    card is owed by Auras or triggers.

    **Sized:** (a) one small PR computing `colors` inside `build()`
    with a color-indicator override, before Phase 8; (b) pick one reading and
    thread `source` through `targeting::object_matches_filter`, ~20 lines,
    with the first card that reads `Owner`; (c) none — a card.

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

    **Reachability (2026-09-03):** nothing owed — a sizing lesson.

    **Sized:** none.

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

    **Reachability (2026-09-03):** nothing owed — RS-2 owns the cast-time and
    targeting sites.

    **Sized:** none.

38. **The keyword-derived restriction sweep is asked only of the event's
    subject, and the `debug_assert` is what keeps that sound.** Indestructible is
    `ObjectSet::SourceOnly`, so the only object whose synthesized restriction
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

    **Reachability (2026-09-03):** nothing owed — the `debug_assert` guards the
    shape.

    **Sized:** none.

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

    **Reachability (2026-09-03):** nothing owed — a measurement record.

    **Sized:** none.

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
    | `priority.rs` priority loop | `blacklist`, `retries` | **No for `retries`; yes for `blacklist`, measured 2026-09-15 and halved 2026-09-16.** `all_candidates` left the stack with item 139 — the list is enumerated per prompt now, so the prompt is a function of the board. What survives is the blacklist: a fresh enumeration can still offer the action that just failed, and a fork resuming as a new round is not filtering it. Item 140 has the size |
    | `put_on_stack.rs::run_mana_ability_window` | the `failed` set | **No** — same shape; the mana pool itself is on `GameState` |
    | `put_on_stack.rs` 601.2b–d | the in-flight `StackEntry`, pre-push | **Yes** — see below |
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

    **Violator 2 — `put_on_stack.rs`, the CR 601.2 announcement window.** Between the
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

    **Reachability (2026-09-03):** unreachable — nothing forks: no harness
    clones `GameState` at a decision point (`tests/` has no such test). RC-5's
    `GameState::entry_selection` (item 55) is the in-tree precedent for the
    fix's shape.

    **Sized:** violator 1 — a `PendingReplacement { applied,
    declined, exempt_applied }` on `GameState`, saved and restored by
    `execute_batch_inner` the way `entry_selection` is, ~80–120 lines; violator
    2 — a `PendingCast` holding the 601.2b–d proposal that the four rewind sites
    and the push read, ~200–300 lines; both land with the first fork-based
    harness, the AI track, not before.

    **RD-2 (2026-09-09):** violator 1's three sets are now per subject group
    rather than per member — the same frame, the same debt. One more piece of
    decision state arrived and *did* go on `GameState`: CR 615.7's allocation
    answers (`prevention_allocations`), because they are read across the
    prompts of groups decided later, which is exactly the fork-at-a-prompt
    test. Item 55's `entry_selection` was the first instance of the fix's
    shape; this is the second, and the table above gained no third violator
    then. **It has one now (2026-09-15, the post-RE audit's pass 3):** the
    priority loop's `all_candidates` and `blacklist`, found by running item
    41's test as a probe — item 139.

    **A4h (2026-09-16):** the row is honest now and one of its two halves is
    closed. `all_candidates` is gone from the frame; `blacklist` is not, and
    the fork test is what turned "wrong" into a number — with the enumeration
    fixed and the blacklist left alone, 52 of 15,646 branches still failed to
    replay, every one of them offered a list the original had filtered. The
    third thing the pass found was not decision state at all but an
    enumeration gap (item 150), and it is the reason the count is 0 rather
    than 52: the actions the blacklist still holds are ones no fresh
    enumeration offers back. **That is a property of today's pools, not of the
    design** — one card whose activation fails for a reason the oracle cannot
    read puts the count back above zero, and item 140's `blacklist` on
    `GameState` is what makes it structural.

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

    **Measured 2026-09-15 (the post-RE audit's pass 3), and promoted to a
    requirement.** The test was run as a throwaway probe in the shape above —
    record every provider answer; clone `GameState` at the first prompt of a
    priority round; replay the recorded answers from that prompt on; compare
    the rendered logs with ids masked — over 20 games at two seats
    (`performance`), 20 at four (`stress`) and 10 at Commander scale (four
    100-card decks, 40 life): **779 forks, 741 identical continuations, and
    every one of the 38 divergences is item 139's retry list** — a prompt
    mismatch at the fork itself, none silent. So nothing else on the stack
    at a round start is outcome-bearing, which is what this item predicted,
    and the one thing that is was not in item 40's table. Two things the
    probe could not fork: a prompt mid-round (item 140) and the cleanup
    step's CR 514.3a re-loop. **The requirement:** a fork at any priority
    prompt, resumed with the same answers, replays the original event
    stream identically; the test lands in item 139's PR, asserting every
    round-start fork, and gains the mid-round forks when item 140 lands.
    **The RNG question, decided rather than discovered:** a fork carries
    both streams — `GameState.rng` in the clone, and the provider's `StdRng`
    by cloning the provider (`RandomDecisionProvider` needs
    `#[derive(Clone)]`; `RefCell<StdRng>` is `Clone`) — so replay-exact is
    the default and the test's premise; a search that wants determinization
    reseeds the *branch's provider* explicitly, never the state's `rng`;
    and re-randomizing an unseen library is `backlog.md` §2.9's per-viewer
    question, since a clone carries every library in its shuffled order and a
    search over it is omniscient until that query exists.

    **Met for round starts 2026-09-16 (A4h), and the test is in the tree.**
    `tests/priority_fork_test.rs` clones `GameState` and the provider at every
    priority prompt where `priority_player == active_player` — the round start
    and the active player's own re-asks, which is what the state can tell apart
    — resumes through `Game::resume_turn_at_priority`, and requires the two
    rendered logs to be equal **verbatim, ids included**: A4g made ids
    process-stable, so the probe's mask is gone and the assertion is stronger
    than the probe's was. Three boards, one seed each, about six seconds in
    debug; the 64-seed sweep behind them is an `#[ignore]`d test, 192 games in
    about two seconds in release. What it took was **two** fixes, not one —
    item 139's fresh enumeration and item 150's target check — and the sweep
    reads 119 failing branches on `main`, 38 with item 139's fix alone, 92 with
    item 150's alone, **0 of 13,530 with both**. The RNG decision above is what
    the test implements: both streams cloned, and the guard on it is a fourth
    test that reseeds the branch's provider and requires a *different* game.

    **What is still owed** is item 140's half — a prompt anywhere else in a
    round, and CR 514.3a's cleanup re-loop — and with it the blacklist, which
    is the one thing on the frame that can still decide a prompt. The entry
    point item 140 extends rather than replaces is in the tree
    (`Game::resume_turn_at_priority`); what it adds is a round that can start
    at a seat other than the active player.

    **Reachability (2026-09-16):** closed for round starts — asserted every
    run; reachable for the rest, item 140.

    **Sized:** ~~the test, ~250 lines beside `tests/determinism_test.rs`~~ —
    **built 2026-09-16** (A4h), 440 lines: a fork recorder that doubles as the
    branch's prompt watcher, three boards, the reseed guard and the sweep. No
    id mask, and no replaying provider — cloning the random one is the replay.

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

    **Reachability (2026-09-15):** reachable — not wrong; every game's log grows
    without bound and is cloned whole with every `GameState` clone. Measured
    at Commander scale by the post-RE audit's pass 3 (item 143): the log is
    47 KB of a 92 KB clone at turn 40 and 700 KB of 786 KB when a 113-turn
    four-seat game ends, and it takes a 5 µs clone to 125 µs — so for a
    search that forks at every decision it is fifteen-twentieths of the
    fork, and for a batch of a thousand straight-line games it is most of a
    gigabyte of resident log.

    **Sized:** a bounded in-state window plus a sink, keeping
    `records_from` semantics, ~100–150 lines in `events/`; lands with the
    `TraceSink` ("Before Triggered abilities" item 5) or the first fork harness,
    whichever first — item 138 ranks it first among the levers for the fork
    use case and nowhere for straight-line throughput.

    **The stream design (the owner's review of PR #153, 2026-09-15).** What
    the rules need from the past is bounded, and the survey that settled it
    is worth keeping. This-turn counters: spells cast, life lost, cards
    drawn, permanents that left, damage dealt, lands played — every "this
    turn" condition the CR or a card states. Last-turn counters, and the
    turn is *the player's own*: the day/night rule (CR 726) reads the
    previous turn's spell count; Paladin of Atonement asks whether you lost
    life last turn; Arboria reads what a player did during *their* last
    turn; Concert Kaboomist counts your noncreature spells "since the
    beginning of your last turn", which at four seats spans a whole turn
    cycle — so the window is each player's current and previous own turn,
    not two turns of the table. A few "this game" counters, Approach of
    the Second Sun's cast count the printed one. Last-known information
    inside a single resolution (CR 603.10). Trigger matching, which reads
    the current batch (`records_from`). Loop detection (CR 731, `backlog.md`
    §2.28), which compares state hashes, not events. None reads the whole
    log. So the honest build materializes those summaries as **per-player
    counters on the turn**, two turns deep per player, bumped at the
    chokepoint — the lesson `PermanentState`'s materialized fields taught:
    a stored field means one thing, and a condition scanning even a short
    window of events is deriving CR state live — which leaves the trigger
    matcher's suffix as the window's only in-state consumer. Everything
    that wants the whole history is outside the engine and reads the sink:
    trace pages, `--dump-events`, the fork test's comparison, a GUI's game
    log, the fuzz harness's statistics. Sized: the per-player turn
    summaries, ~60 lines beside `last_turn_began`; the window plus the sink
    keeping `records_from` semantics, the ~100–150 above; the fork test
    compares the sink's output instead of the state's log.

    **What rode with A4c (2026-09-18, PR #170), and what did not.** The
    performed-event stream reaches the sink through one door,
    `GameState::emit_event`, which every emitter now calls and which writes
    the same text `--dump-events` prints — so the dump is a projection of a
    trace (`plans/trace_spine.py --events`), and the fork test's comparison
    can read a trace instead of the state's log when item 140 extends it.
    **The in-state window stays unbounded**: bounding it is a `GameState`
    representation change with its own clone-cost reading, and the
    per-player turn summaries wait for A6's doc by that row's own text. What
    is still sized here is the window and the summaries, ~60 plus ~100–150
    lines, minus the sink.

43. **~~CR 122.6a names a player and `EnterMods` does not carry one~~ ✅ CLOSED
    2026-09-14 (RE-5's review, theme A) — built.** `EntryCounters.by` and
    `EntryCountersTemplate.by`, the merge keyed on `(kind, putter)`, the entry
    door reading each row's putter ahead of the entry's controller, and a
    `by: Option<PlayerRef>` on `Primitive::AddCounters` and `GetCounters` for
    Bold Plagiarist's shape on a proposal. RE-5 had closed it on 2026-09-13
    on an empty Scryfall query, which the review's rule rejects — the CR is
    the customer, a printed card is the test (`engineering-practices.md` §4;
    `replacement-architecture.md` §11 item 83).
    → `plans/archive/codebase-state-closed.md`.

    **Reachability (2026-09-14):** closed — built; no printed producer names
    a putter at an entry, and the fixture tests do.

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
    (**The "one deck in sixteen" half is history from 2026-09-03 on**: the
    `Everywhere` land removed `random_deck`'s color filter, so every deck now
    draws uniformly from every nonland. Sigarda's five mana is the whole of the
    argument since then. Corrected 2026-09-08 by the RD-1 review, which found
    the same stale claim repeated in two card files.)

    **So this is deferred rather than done, and the trigger is a card, not a
    date.** The fix is five lines and exactly `gather`'s — skip the permanent
    unless `restriction_ability_sources.contains(&id)` or the registry summary
    reports a Layer 6 grant, which is the same predicate one object at a time,
    exact by the same argument and inheriting the same Layer 1 hole (item 16).
    **Do it with the first cheap or colorless restriction card**, which RS-2 and
    RS-3 will both want; until then it is a 10%-shaped cliff nobody is standing
    on. The general lesson is worth more than the item: **a sweep for this defect
    shape would have "fixed" this instance and reported a win it did not earn.**

    **Reachability (2026-09-03):** reachable — not wrong; perf only, measured
    flat because Sigarda is still the only restriction source and lands late.

    **Sized:** five lines at `predicate.rs:107` — `gather`'s
    per-permanent gate — with the first cheap or colorless restriction card,
    RS-2.

45. **Engine cost is now a fixture, and it found two determinism violations the
    old rule could not see (added 2026-09-01).** `state/diagnostics.rs` counts
    layer walks, computed frames, replacement gathers and restriction queries per
    game; `fuzz_games` prints them and `fuzz-record.md` stores them
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

    **Reachability (2026-09-03):** nothing owed — two open decisions (walk
    attribution; whether a clone inherits the counts), neither with a customer.

    **Sized:** none until a profiler question needs one.

### Found by the look-ahead frame (2026-09-02, RC-4)

46. **~~The frame is per entry, not per batch, and §5b says it should be per
    batch.~~ ✅ CLOSED 2026-09-13 (RE-4).** The frame half was RC-4b's; the
    producer is `GameAction::CreateTokens`, whose performer proposes every
    token's entry as one batch — Raise the Alarm in the pool is the first
    plural entry a measured game builds. → `plans/archive/codebase-state-closed.md`.

    **Reachability (2026-09-13):** closed — RE-4.

### Found by the RC-4 review's nesting audit (2026-09-02)

51. **~~A rewound cast leaves its CR 601.2a move in the log.~~ — ✅ CLOSED
    2026-09-02 (RC-4b).** — archived.
    **Reachability (2026-09-03):** closed — RC-4b, PR #87 (6541d0b).
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by the RC-4
    review's nesting audit (2026-09-02)" item 51.

### Found by RC-4b — entering is one event (2026-09-02)

52. **~~A token whose entry is exiled instead records `from: Battlefield`.~~
    ✅ CLOSED 2026-09-13 (RE-4).** The substitute is `GameAction::CreateTokenIn`
    — an appearance, announced as `TokenCreated { Exile }` — and the token has
    no `ZoneChange` at all; Hallowed Moonlight is registered and reaches it.
    → `plans/archive/codebase-state-closed.md`.

    **Reachability (2026-09-13):** closed — RE-4.

### Found by RC-5 — applying an entry can move the board (2026-09-03)

**Shipped:** CR 614.13/13a/13b as `Rewrite::EnterAfterMoving`, and
`EnterMods.counters` given an amount the board decides. `+2,239 / −121` across
18 files. **Trace page:**
[`plans/traces/rc-5-applying-an-entry-can-move-the-board.html`](traces/rc-5-applying-an-entry-can-move-the-board.html)
— four boards, every read labelled board / frame / scope / player.
`replacement-architecture.md` §9 has the design, the findings and the
measurement; what follows is what a later phase has to know.

53. **`apply_rewrite` takes `&mut GameState`, and exactly one arm needs it.**
    CR 614.13 is the rules' own statement that applying an entry replacement may
    change the board, so a `Rewrite` is no longer a pure function of the event.
    Every other arm still is. The mutation goes through
    `execute_actions_new_batch`, never a direct write — the chokepoint invariant
    has no exception here — and the loop's termination argument is untouched,
    because `apply_auxiliary_move` cannot create a replacement effect and the
    applied set still bounds the iterations.

    **Will the next arm need it? Asked on review, and the answer is mostly no.**
    RD's `Amount` (614.5 doublers, 615.7 partial prevention) is arithmetic on
    the event. RD's `Retarget` (614.9) re-checks its destination against the
    board, which is a read. CV-2's copy-on-enter (616.1c) *chooses* a donor,
    which is a prompt — `&GameState` plus `ctx.dp`, as
    `EnterUnderControlOf(Opponent)` already is. **The one that will is piece 4**,
    CR 614.12a's choice-carrying mods: "as this enters, choose a color" has to
    record the choice somewhere a linked ability can read it, and that write is
    on the object. So the count stands at one arm today and two when linked
    abilities land — which is the argument for leaving the signature `&mut`
    rather than threading a narrower capability that would be widened twice.

    **Answered by RE-3 (2026-09-12), and "mostly no" held.**
    `AmountRewrite::LifeFloor` is the second arm to consult the board — Ali from
    Cairo's clamp is against the affected player's life total *now* — and it is
    a **read**, one `get_player` in `apply_rewrite`'s `LoseLife` leg. So the
    count of arms needing `&mut` still stands at one, and the prediction's list
    of reasons an arm might want the board (arithmetic, a re-check, a prompt) is
    now four for four. Piece 4 remains the one that will write.

    **Reachability (2026-09-12):** nothing owed — a signature decision,
    recorded, and re-checked against a new arm.

    **Sized:** none.

54. **`execute_actions_new_batch` is §4.2's one exception, and the argument it
    needs is not "these are different".** A nested `execute_actions` joins the
    enclosing batch on CR 120.3f's grounds: lifelink's life gain is a *result
    of* the damage. CR 614.13's moves are performed in phase 1, while the entry
    is still being decided, so there is no entry event for them to be part of;
    and two devour creatures entering together apply their replacements one
    after the other, so joining would hand a CR 603.2c "whenever one or more
    creatures die" one event where the rules have two. **Unreadable today** —
    nothing consumes a `BatchId` until critical-path item 6 — which is why it
    is asserted in a test rather than left to be discovered there
    (`test_the_auxiliary_moves_are_their_own_batch`). A second caller needs the
    same argument made again, from the CR.

    **Reachability (2026-09-03):** nothing owed — asserted by a test; unreadable
    until critical-path item 6.

    **Sized:** none.

55. **`GameState::entry_selection` is batch-scoped state, and it is the third
    thing item 40 would have caught.** CR 614.13a's "not the entering object nor
    anything entering simultaneously" and 614.13b's "not the same object twice"
    are both read across the CR 616.1 prompt and both change the outcome if
    lost, so they are on `GameState` rather than on the pipeline's stack —
    **item 40's table gains no third violator.** Saved and restored by
    `execute_batch_inner` the way `open_batch`/`close_batch` handle the event
    stamp, and the chosen set is recorded *before* the moves, so the nested
    batch cannot lose it. Two mutations pin each half.

    **Reachability (2026-09-03):** nothing owed — done; recorded as item 40's
    precedent.

    **Sized:** none.

56. **CR 614.13b is redundant until two effects' zones chain, and that is worth
    knowing before the next rule like it.** A sacrificed creature stops matching
    the next battlefield filter on its own, so the CR's own example — one
    Runeclaw Bear, devour 3 and devour 5 — gives the right answer with the rule
    deleted. It bites when one effect *writes into* the zone the next one reads:
    devour into a graveyard, then Sutured Ghoul's exile. **Found by the mutation
    pass, not by the design**, and the lesson generalizes: `§10`'s
    "mutation-check every assertion" can report a weak *board* rather than a
    weak assertion.

    **Reachability (2026-09-03):** nothing owed — a lesson about boards.

    **Sized:** none.

57. **`AmountExpr::SourcePower` has exactly one evaluator, and the other two
    refuse it.** — ✅ closed, archived.
    **Reachability (2026-09-07):** closed — CM-2, and the answer is that there
    is no third evaluator. …
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by RC-5 —
    applying an entry can move the board (2026-09-03)" item 57.

58. **Item 47's predicate has a fourth expiry condition, and RC-5 fired it.**
    `ordering_cannot_change_outcome`'s theorem has two halves — every member still
    applies, and the applications commute — and the second was free while
    `EnterModsTemplate` held literals. It is not free now. The premise added is
    **exact rather than conservative**: an amount is order-invariant if it is
    `Fixed`, *or* its instance's source is not the entering object, since only
    then can `frame_of` return `Some`. Master Biomancer therefore keeps the
    suppressed prompt and the fuzz pool keeps its zero-prompt property. The rule
    for whoever adds a fifth is item 47's: revisit the predicate in the same
    commit.

    **Reachability (2026-09-03):** nothing owed — recorded so the fifth
    condition follows the rule.

    **Sized:** none.

59. **Sutured Ghoul's power and toughness are missing, not wrong.** Its `*/*`
    box is a CDA (CR 208.2) whose text reads "the exiled cards", which CR 614.14
    links to the exiling ability — CR 607, whose live home is `backlog.md` §2.2
    (the `T##` labels are the archived plan's vocabulary, not a queue), the same block Painter's
    Servant sits behind. The card is registered at its printed box's value
    without the CDA, **0/0**, which is also the right answer when nothing is
    exiled; a Ghoul that exiles something should live and dies to CR 704.5f
    instead. In the default registry, so `--pool stress` plays it; out of
    `PERFORMANCE_POOL`. **Closes with `backlog.md` §2.2**, and it is the second card in the
    pool whose printed P/T box the engine cannot fill (Keldon Warlord's is
    filled).

    **Reachability (2026-09-03):** reachable — wrong today, and counted
    (2026-09-03, `--pool stress --require "Sutured Ghoul"`, 40 games, seed
    12345): 27 entries; 11 exiled at least one creature card first (RC-5's
    auxiliary graveyard→exile moves, performed); 12 died at once to CR 704.5f as
    a 0/0; **6 did both** — the Ghoul exiled a Sigarda, a Knight of Meadowgrain,
    a Savannah Lions, a Thornweald Archer, and CR 208.2 with CR 604.3 make its
    power and toughness the exiled cards' totals, so each of those six should
    have lived. The ten that survived did so on Master Biomancer's counters or
    an anthem. Registered without the CDA on purpose and recorded as such, so
    this is a known wrong answer rather than a discovery — but it is the first
    item on this list that a fuzz game fails today, about once per seven games,
    and it stays that way until CR 607 (`backlog.md` §2.2).

    **Sized:** `backlog.md` §2.2 sizes linked abilities at one
    phase — per-pair state that survives a zone change; the Ghoul's CDA is then
    ~30 lines reading its linked exile set. Until then the choice is the pool's:
    keep the loud 0/0 (it dies at once, which the log shows) or take the Ghoul
    out of the default registry.

    **Scheduled (2026-09-15, post-RE audit):** owner `backlog.md` §2.2,
    unchanged, and the loud 0/0 stays in the stress pool on purpose — taking
    the Ghoul out is a pool move for a wrong answer the §2.2 phase fixes
    whole.

60. **Master Biomancer's Mutant clause is unimplemented, and it is not one
    field.** "…and as a Mutant in addition to its other types" wants a type on
    `EnterMods`, which fires item 47's expiry condition (a) directly:
    `ObjectFilter`'s `ByType` and `BySubtype` leaves would stop being
    mods-invariant, so **every** CR 616.1 entry bucket would start prompting.
    It also needs somewhere for the type to live *after* the entry — a Layer 4
    effect with no registry row and no duration, which is a shape the layer
    system does not have. **Sized:** the field is small and the two consequences
    are not; call it a phase of its own, and note that the printed population
    for "enters as a [type]" is thin enough that it is not urgent.

    **Reachability (2026-09-03):** reachable — wrong today, unobservably: Master
    Biomancer is registered (stress pool), every creature it pumps should also
    be a Mutant, and the engine adds the counters and not the type. No
    registered filter reads the Mutant subtype (the only subtype read in the
    pool is Keldon Warlord's non-Wall), so no game outcome moves.

    **Scheduled (2026-09-15, post-RE audit):** `backlog.md` §2.30 is the
    entry — one mechanic, "enters as an additional type", with its census
    (five printed cards). **Re-sized there the same day, and it is one PR of
    ~150–200 lines, not a phase:** both consequences above were answered by
    work that landed after this item was written — RE-5's `kinds_present`
    answers the prompting fear per kind rather than per bucket, and the
    board pass already reads rowless, durationless state off
    `PermanentState` at layers 6 and 7c (counters), which is the shape an
    entered-as type takes at Layer 4. The verdict stands: wrong today, and no
    outcome moves until a filter reads the type.

61. **Every auxiliary move of one entry event should be one batch, and RC-5
    ships one per application.** Thunder-Thrash Elder's own ruling
    (Gatherer, 2008-10-01): "If multiple creatures with devour are entering
    under your control at the same time, you may use each one's devour ability.
    A creature you already control can be devoured by only one of them, however.
    **All creatures devoured this way are sacrificed at the same time.**" The
    same ruling is where CR 614.13a and 614.13b came from, and RC-5 got those
    two right and the simultaneity wrong. `apply_auxiliary_move` performs its
    picks through `execute_actions_new_batch` immediately, so two applications
    are two batches at two moments; a CR 603.2c "whenever one or more creatures
    die" would fire twice where the rules fire once.

    **Unreachable today**, and it takes two things that do not exist to reach:
    a multi-entry batch (item 46) or a second devour ability on one entry
    (nothing grants devour; the plane in CR 614.13b's example is not a card
    type this engine has). The fixture in
    `phase_rc5_integration_test::grants_devour` is the only board that gets
    there.

    **Not a batch-id relabelling — a deferral, and it has a real cost.** The
    moves would be collected across phase 1 and performed once, before phase 2
    performs the entries. But the counters devour arrives with are computed
    *while applying*, and RC-5 counts what the nested batch actually performed
    (a dropped move is not a sacrifice, CR 701.21a). Deferred, there is nothing
    performed yet to count, so the count reverts to what was chosen and the
    prevented-move case (`test_a_prevented_sacrifice_is_not_counted`) inverts.
    **Whoever builds this owes an answer to that**, and the CR's own wording —
    "for each creature sacrificed this way" — is on the side of counting the
    performed moves, which argues for performing the batch *before* the mods
    are finalised rather than after. **Sized:** ~150 in `execute_batch_inner`
    and `apply_auxiliary_move`, plus the count question. Lands with item 46's
    producer, since neither is testable in a game without the other.

    **Reachability (2026-09-03):** unreachable — re-checked: no multi-entry
    batch (item 46) and nothing grants devour; `grants_devour` in
    `phase_rc5_integration_test.rs` is the only board.

62. **`AuxiliaryMove` has no chooser field, so "you" is the effect's
    controller — and the two printed shapes disagree.** CR 614.13a says "**you**
    may have to choose a number of objects", and who "you" is depends on where
    the ability came from: devour is the entering creature's own ability, so the
    choice is the *entering permanent's* controller's, while a filter-scoped
    effect ("each other creature **you** control enters …") means its own
    controller. RC-5 uses `ReplacementInstance::controller` for both, which is
    exact for every registered card — devour is `ObjectSet::SourceOnly`, and
    `gather` source 1a hands it the proposal's controller — and wrong for a
    *granted* devour, where the granting permanent's controller would choose and
    sacrifice their own creatures instead of the entering creature's controller
    doing it. **Sized:** one field on `AuxiliaryMove`
    (`EffectController | EnteringController`) read in one place, ~15 lines.
    Deliberately not built: no registered card takes the second road, and a
    field with one used value is the shape §3.2's growth contract warns about.
    The test fixture `grants_devour` takes it and says so.

    **Reachability (2026-09-03):** unreachable — no registered card takes the
    granted-devour road; the same fixture is the only board.

63. **`per_chosen` is a constant per object, and Thromok the Insatiable's is
    not.** "Devour X, where X is the number of creatures devoured this way" —
    X creatures give X² counters, so the multiplier *is* the count. One more
    shape in the payload (`per_chosen: PerChosen::Fixed(n) | PerChosen::Count`)
    and no new `Rewrite` arm, which is the growth contract working as intended.
    ~20 lines with the card. It is one of the 23 printed devour cards; the other
    22 are `Fixed`, and CR 702.82c's devour-[quality] variants are covered
    already by the payload's `filter`.

    **Reachability (2026-09-03):** unreachable — Thromok the Insatiable is not
    registered.

    **Sized:** ~20 lines with the card, as the entry says.

64. **~~`PermanentFilter` filters objects in zones where nothing is a
    permanent, and the name now lies.~~ ✅ Renamed to `ObjectFilter` 2026-09-07
    (CM-0, `cost-architecture.md` §3.2).** — archived.
    **Reachability (2026-09-07):** closed — renamed.
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by RC-5 —
    applying an entry can move the board (2026-09-03)" item 64.

65. **`order_invariant_entry_bucket` was named after its implementation, not its
    question.** The question is "does CR 616.1's ordering prompt have more than
    one outcome here" — §11 item 19's rule that the engine must not ask a
    player a question whose answer cannot matter. "Bucket" is CR 616.1a–e's
    forced-choice class, which a reader has to already know to parse the name.
    `entry_ordering_is_observable` (negated at the call site) says the question;
    the counter-argument is that "bucket" is the codebase's word for the thing
    the function takes, and renaming the predicate without renaming
    `forced_bucket` trades one mismatch for another. **Decide with the rename in
    item 64's PR or leave it**; recorded because the confusion was reported
    rather than guessed at.

    **Reachability (2026-09-03):** reachable — not wrong; a name, decided with
    item 64.

    **Sized:** none beyond item 64's PR.

    **Closed by RD-2 (2026-09-09), and the counter-argument was wrong on a
    fact.** The predicate is `ordering_cannot_change_outcome`, renamed when the
    second admissible shape arrived (item 47) because a name saying "entry" had
    become wrong as well as implementation-shaped. The counter-argument above —
    leave it, because "bucket" is the codebase's word for what the function
    takes — assumed the word was the CR's. **It is not**: CR 616.1a–e is a
    ladder of *steps*, each reading "if any … one of them must be chosen. If
    not, proceed to [the next]", and "bucket" appears nowhere in the rule. So
    the mismatch was real in both directions and `forced_bucket` was renamed
    with it, to `must_choose_among` — 616.1a's own sentence — with the local
    `bucket` becoming `choosable` and the ~25 doc uses of the word in the
    616.1 sense becoming "step". The word survives only in the
    `DecisionProvider::allocate` API, where it means a bucket to allocate a
    total across and is nobody's confusion.

    **The finding underneath, worth more than the rename:** this is item 89's
    shape again. The sentence "keeps its name — 'bucket' is CR 616.1a–e's own
    word there" was written on 2026-09-09 in the RD-2 docs commit, was false
    when written, and no test could fail on it. It was caught in review by a
    reader asking what a bucket *was* — which is the only instrument this
    project has for that class of claim, and the argument for
    "Before card breadth" item 11's glossary check.

### Found by CM-1 — cost modification (2026-09-07)

**Shipped:** CR 601.2f whole — assembly and modification — as
`engine/cost_determination/`, the cost effects discovered off
effective ability lists behind a three-leg gate; `Effect::CostModification`,
`Condition::SourceUntapped`, `ChoiceKind::OrderCostReductions`; the castability
preview reading the locked cost; Thalia (pooled), Goblin Electromancer,
Trinisphere. `plans/cost-architecture.md` has the design, the sizing of
CM-2–4 and CP-1, and the Krark-Clan Ironworks loop as the casting pipeline's
integration test; what follows is what a later phase has to know.

**Measured** (`plans/fuzz_ab.py`, 2026-09-07, three arms — `main`, the CM-1
engine with its three cards registered but not pooled, and CM-1 pooled): the
unpooled arm reproduces `main` byte for byte on `performance` outside the
timing block, so every counter the pooled arm moves is Thalia's doing and not
the engine's; CPU/game 15.67 → 15.70 ms (+0.2%), deterministic in all three
arms. Thalia forced into every `performance` deck resolves in 66% of 200
games; with Humility forced beside her, both are on the board in 52%.
`fuzz-record.md` has the re-recorded table.

70. **`run_mana_ability_window` closes the moment the pool covers the cost, and
    CR 605.3a has no such clause.** — ✅ closed, archived.
    **Reachability (2026-09-08):** closed — CM-4.
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by CM-1 — cost
    modification (2026-09-07)" item 70.

71. **The window's opening condition is right by accident.** — ✅ closed,
    archived.
    **Reachability (2026-09-08):** closed — CM-4.
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by CM-1 — cost
    modification (2026-09-07)" item 71.

72. **CR 732.1's reversal of mana abilities is the player's option, and the
    engine never offers it.** "Each player may also reverse any legal mana
    abilities that player activated while making the illegal play"; the
    rewind keeps them every time, which is *a* legal answer and not the
    player's. The Mind Stone puzzle (`cost-architecture.md` §3.11) is the
    board where it is observable; the one question left for a judge there
    is where the surviving trigger goes after the rewind, not whether the
    reversal is offered. Widens `backlog.md` §2.18's reversal entry, which
    already named the prompt.

    **Reachability (2026-09-07):** reachable — not wrong; a forced choice.

    **Sized:** a `ChoiceKind` at the two rewind sites and the mana undone
    silently, ~60 lines. **Placed 2026-09-08 (CM-4): with the trigger phase,
    critical-path item 6, and not with the payer.** Two reasons, and the first
    is this phase's own criterion. Which mana abilities to reverse is a
    strategic choice — the answers leave different permanents and different
    events, not different mana — so it fails `ui::AutoPayer`'s test for what a
    payer may answer and the payer is the wrong home for it. And the board
    where the reversal is observable is the Mind Stone puzzle, whose remaining
    question is where the surviving Scrap Trawler trigger lands; shipping the
    prompt before that is shipping it against a board the engine cannot
    finish.
    **Two options, and only two — decided 2026-09-18 (A4k, `backlog.md` §2.22
    row 9) and re-derived at its review.** *Reverse all* is a no-log clone
    taken at the window's first activation (item 42's measurement: 4–6 µs)
    and restored at the rewind with `events` truncated to its length — CR
    732.1's "no abilities trigger and no effects apply" by construction, the
    RNG rewound with it, the retry loop's locals and `Diagnostics` kept live.
    *Reverse some* is not a facility the chokepoint has: performed events with
    replacements applied have no per-event undo, and Arena offers undo-all
    only. Re-sized on that: ~40 lines, the clone, the restore and the
    truncation, in place of the ~60 below. **Answered by a middleware? Only
    under the same toggle as auto-yield.** On a human's seat it is a
    prompt-skipping automation — reverse all when the solver made the taps —
    inside the tap solver's decorator (main item 162); on a bot's seat a
    plain policy, `fuzz_games` keeping the taps as today's stream does; never
    silently on a human's. The placement here does not move.
    **The invariant it must keep:** a taken reversal undoes the ability's
    cost and its mana together — one without the other is infinite colorless
    mana from Ironworks and Mind Stone alone (`cost-architecture.md` §3.11).

73. **~~A conditional replacement or restriction static is inert.~~ — ✅
    closed 2026-09-12 (RE-6).** `register_static_effects` records a source
    through the `Effect::Conditional` wrapper for both kinds, and both sweeps
    — `replacement::gather`'s static leg and `is_prohibited`'s — peel it and
    ask `settled_holds` at the proposal, the evaluator CR 613.11's cost effects
    already use. Laboratory Maniac is the first such card on the replacement
    side; the restriction side is a fixture
    (`a_conditional_static_cant_is_honoured_while_its_condition_holds`).
    *Original entry:* a conditional one was never inserted as a source, and
    the sweeps matched only the bare body.

74. **The generic split and the mana window read only the first `Cost::Mana` —
    closed by CM-1's merge.** — ✅ closed, archived.
    **Reachability (2026-09-07):** closed — CM-1.
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by CM-1 — cost
    modification (2026-09-07)" item 74.

### Found by CM-2 — the spell's own cost abilities (2026-09-07)

**Shipped:** CR 113.6d and 702.41a — a spell's own cost ability as CR 601.2f's
second gather source (`CostSubject::Itself`), `CostChange::ReduceGeneric` with
`engine::layers::compute::settled_amount` behind it, `CardDataBuilder::
affinity_for`, and the `keyword` → `keyword_flag` rename that made room for it;
Myr Enforcer (pooled) and Frogmite. Three of `cost-architecture.md`'s claims
were corrected in place by the building, and §3.7's argument survived intact.
Main item 57 closes here.

**Measured** (`plans/fuzz_ab.py`, 2026-09-07, three arms — `main`, CM-2's
engine with both cards registered but not pooled, and CM-2 pooled): the middle
arm is `IDENTICAL` to `main` on `performance` outside the timing block, so
every counter the pooled arm moves is the pool's. CPU/game 16.63 → 16.50 ms
for the middle arm (−0.8%, inside the sitting's spread); deterministic in all
three arms, and three shell runs at one seed match on both pools. Myr Enforcer
forced into every `performance` deck: cast 167, resolved 166, in 56% of 200
games, 1.74 copies per deck. `fuzz-record.md` has the re-recorded table.

75. **A registry row cannot reach an object off the battlefield, so a granted
    or copied cost ability on a spell is unreachable.** `compute_non_member`
    applies CDAs and no rows at all, so a card in hand or a spell on the stack
    has exactly its printed ability list. Two consequences, and the first is
    load-bearing: source 2's printed gate leg is *exact* rather than an
    over-approximation, which is why reading `card_data.abilities` there is a
    gate and not an answer. The second is a gap — CR 113.6e's second sentence
    ("an object's ability that grants it another ability that restricts or
    modifies how that particular object can be played or cast functions only
    on the stack") has nothing to grant with, and neither has CR 707.10's copy
    of a spell.

    **Reachability (2026-09-07):** unreachable — no route exists to put a row
    on a non-member, and no registered card asks for one.

    **Re-derived (2026-09-14, LJ) — the route now exists, and the heading of
    this item is what went stale.** "A registry row cannot reach an object off
    the battlefield" is no longer true: a `Filter` row naming
    `ZoneSet::STACK` reaches a spell, and one naming `HAND` reaches a card in
    hand. The literal *reachability* sentence survives on a technicality — an
    object a row reaches is a **member**, so nothing puts a row on a
    non-member — and that technicality is the whole change, because
    `compute_non_member` is now the walk of an object **no row names** rather
    than of any object off the battlefield.

    What this costs the item's two consequences:

    - **The load-bearing one is still true, and is now true for a different
      reason.** Source 2's printed gate leg reads `card_data.abilities` for a
      spell, and that was *exact by construction* while no row could reach the
      stack. It is now exact only because no registered row names it. The
      backstop was already built and already OR'd into the gate —
      `any_granted_cost_modification` and `any_copied_cost_modification` — so
      the answer does not change today; what changed is that the gate rests on
      a flag rather than on a structural impossibility, which is a thing to
      know before writing the first stack-reaching cost ability.
    - **The gap half narrowed.** CR 113.6e's second sentence still has nothing
      to grant *with*, but no longer nothing to grant *to*: the zone half is
      built and it is CR 113.6 itself that is missing, which is roadmap row A5 (its third PR, LK).

    **Reachability (2026-09-14):** unreachable — still no registered card puts
    a cost ability on an object off the battlefield, and the two summary flags
    catch it when one does.

    **Re-derived (2026-09-14, LK) — the gap half now has an owner and a
    name.** CR 113.6e's second sentence still has nothing to grant *with*, and
    LK is the row that was going to change that and did not: §13d decision 4
    defers 113.6e, because its first sentence needs "any zone from which it
    could be played" and `check_cast_legality` still hard-codes `Zone::Hand`
    (`backlog.md` §2.3). So this item's second consequence is no longer waiting
    on an unbuilt facility — it is waiting on §2.3, which is a different
    queue. The load-bearing first consequence is untouched: source 2's printed
    gate leg still rests on a flag rather than on an impossibility, which is
    what LJ's re-derivation established and LK did nothing to move.

    **And the deleted method is worth naming here**, because this item cites
    it: `CostSubject::applies_from_battlefield` was CR 113.6d in a method and
    LK removed it. The zone answer is `engine::zone_function`'s now, derived
    from `applies_to_its_own_object`, which survives because it is the
    *identity* question rather than the zone one.

    **Reachability (2026-09-14, re-derived):** unreachable — unchanged, and now
    blocked on a named row rather than on a missing facility.

    **Sized:** none here; when a route exists, the gather's two summary flags
    (`any_granted_cost_modification`, `any_copied_cost_modification`) are
    already OR'd into source 2's gate and are what catches it, so the cost of
    forgetting is bounded to whatever builds the route. LJ built half the
    route and neither flag needed touching, which is that sentence holding.

76. **`Effect::as_…` says what an ability *is*, never where it applies from,
    and all three cost gates are about where.** — ✅ closed, archived.
    **Reachability (2026-09-07):** closed — CM-2. …
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by CM-2 — the
    spell's own cost abilities (2026-09-07)" item 76.

77. **The ordered battlefield sweeps cost ~5% of runtime, and it is the
    deriving, not the sorting.** `CLAUDE.md`'s determinism invariant routes
    every order-observable battlefield read through `battlefield_ids_ordered`
    / `battlefield_ordered`, and each call collects a `Vec<(u64, ObjectId)>`,
    sorts it, and maps into a second `Vec`. Measured 2026-09-07 on the
    `performance` pool: **5,744 calls per game at a mean n of 15.9**, and at
    n=16 one call is **127 ns** against **27 ns** to clone a Vec that was
    already in order and **0.2 ns** to hand out a slice of one. That is
    0.73 ms per game against a 14.2 ms game — **~5%**, of which ~4% is
    recoverable by cloning a maintained order and ~5% by lending it.

    **The sort is not the expensive part**; two heap allocations per call are.
    So the design that recovers it is not "sort less" but "derive less": keep
    the ordered vector on `GameState`, maintained at the three places the
    order can change — `place_on_battlefield` (append; entry timestamps are
    monotonic), removal, and `attach`, where CR 613.7e reassigns a timestamp
    and LH-2 already re-stamps rows — and hand out a slice. Debug-mode
    re-derivation and comparison is the guard, exactly as `audit_memo_hit` is
    for the layer memo.

    **Do not "fix" this by auditing which call sites observe order.** Some do
    not (`steps.rs`'s `any` over first strike, for one), but the invariant is
    blunt on purpose, and re-litigating observability at 52 call sites is how
    it gets decided wrong once. Maintaining the order makes the blunt rule
    free instead of making it negotiable.

    **"So determinism is what costs 5% — is it worth keeping?"** Asked at the
    LK review, and the answer is that determinism is not what costs anything.
    **The requirement is that an order exists and is the same in every run;
    what this entry measures is the cost of *deriving* it fresh 5,700 times a
    game.** Those are separable, and separating them is the whole of this item:
    a kept vector is exactly as deterministic as a sort and costs a slice.
    Compare the alternatives, which is where the requirement earns its place —
    dropping it means a `HashMap` iteration order that reseeds per **process**,
    so `fuzz_games --seed N` stops reproducing, a fork-and-search harness
    cannot compare two lines of play, and a bug found in one run cannot be
    replayed. That is not a performance trade; it is the difference between an
    engine you can debug and one you cannot. The 5% buys all of it, and this
    item is how to stop paying even that.

    **The "sort before accessing" paradigm is also not the only shape
    available**, and is the one this item replaces. Three were considered and
    the notes are here so they are not re-considered from scratch: *sort less*
    (audit which sites observe order — refused above); *sort cheaper* (the sort
    is not the cost, two heap allocations are, measured); *do not sort* (keep
    the order, which is this item). A fourth — a deterministic hasher, so the
    `HashMap` iterates reproducibly — is the one that sounds cheapest and is
    the worst: it makes the *order* an artifact of hashing, so inserting an
    unrelated permanent silently re-orders every decision list, and nothing
    about it corresponds to CR 613.7. Reproducible is not the same as correct.

    **Reachability (2026-09-07):** reachable — not wrong; a measured
    performance cost, and the numbers above are the measurement rather than an
    estimate.

    **LK raised the stake, and declined the fold (2026-09-14). Nothing is
    owed — read this as a bigger *prize*, not a deferred bill.** Asked at the
    LK review and worth stating plainly, because the entry can be read the
    other way: LK did **not** defer a cost. It hit one, paid it back inside the
    same PR, and ended level with `main`.

    What happened: LK moved CR 613.7's timestamp onto `GameObject` (613.7d, so
    a card in a graveyard has one) and that put it one `HashMap` hop from these
    two sweeps — **+16.5% of total game time on an arm whose counters are
    byte-identical**, which is this item's ~5% re-measured from the other side
    and is the sharpest number it has. LK then kept a **copy** on
    `PermanentState`, written by two doors that cannot disagree
    (`set_object_timestamp`, `insert_battlefield_entity`), and the same arm
    reads **−2.5%**. The +16.5% is gone; it never reached `main`.

    **What is still on the table is this item's original ~5%, unchanged** — a
    *win* nobody is obliged to collect. What LK added is a second reason to
    collect it: a maintained order vector needs no timestamp on the entry at
    all, so taking this item deletes the copy as well as the two allocations.
    That is why the copy is documented as temporary rather than as a design.

    **Why LK declined the fold**, given that the fold was in reach: this item
    sizes itself as medium-risk, a missed maintenance point silently corrupts
    every ordered sweep in the game — which is every decision list, log and
    count — and LK was a rules change whose reviewer would have had to check
    two unrelated arguments at once. Not because the fold is wrong.

    **Sized:** one field, three maintenance points, one debug audit, and a
    mechanical return-type change across 52 call sites (most become a borrow,
    the ones that mutate while iterating become the 27 ns clone). ~1 PR,
    medium risk — the risk is drift between the kept order and the truth,
    which is what the debug audit is for. Nothing depends on it; take it when
    ~5% is worth a PR.

### Found by CM-3 — lock-in's payment side (2026-09-07)

**Shipped:** CR 601.2h's payment as decide-then-perform — `plan_payment` takes
every choice against one board and `pay_costs` performs it asking nobody
(`engine::costs`); `payment_order_rank`, the engine's pick of CR 601.2h's "in
any order"; `Cost::Sacrifice(ObjectFilter, n)` paid through the chokepoint as
one `execute_actions` batch with `ChoiceKind::ChooseSacrificeForCost`;
`AdditionalCost::Mandatory` and `is_optional` (CR 118.8b/118.8c), with 601.2b
announcing the optional costs alone; and `castable_spells` refusing a spell
whose mandatory additional cost cannot be paid. Altar's Reap (pooled),
Thunderscape Familiar, Krark-Clan Ironworks, Foundry Inspector and Mind Stone.

**CR 732.1 is answered by construction, and this is where that is recorded.**
The rule's first sentence — "any payments already made are canceled" — has an
empty set to act on, and the reason is a property rather than a rewind:

- The order is chosen so nothing that can fail is paid after something that
  cannot be taken back. `Cost::Mana` is the only cost whose payment can fail
  on a player's choice and it is rank 0; rank-1 costs read state no rank-0
  payment changes; a rank-2 cost's own payment cannot fail, because
  `plan_payment` enumerated its candidates and `validate_pick_n` bounds the
  answer. A `debug_assert!` on `pay_costs`'s failure path enforces it.
- The Mind Stone puzzle (`cost-architecture.md` §3.11) reaches the check and
  not the payment: with the ability's own source sacrificed inside its 601.2g window,
  `can_pay_costs` refuses the `Cost::Tap` before any cost is paid, and
  `rollback_ability_activation` has nothing to cancel. The Ironworks
  activation stands with its mana and its cost, which is what both readings of
  the open judge question agree on.
- **No second chokepoint exemption.** `// CAST-ROLLBACK:` stays the only one.
  Un-sacrificing would mean retracting an emitted, already-replaced
  `ZoneChange` from the stream the trigger phase reads, and restoring a
  `PermanentState` the move destroyed — a system, not an arm. `CLAUDE.md`'s
  chokepoint section is unchanged by this phase.

**Measured** (`plans/fuzz_ab.py`, 2026-09-07, three arms — `main`, CM-3's
engine with all five cards registered and the old pool, and CM-3 pooled): the
middle arm is `IDENTICAL` to `main` on `performance` at 200 games, so the
engine change moves no counter and changes no seeded stream. `cost-
architecture.md` §6's claim that CM-3 "opens no new path a pooled card would
measure" was right about the engine and wrong about the pool: `Cost::Sacrifice`
is a path the 72 could not reach, so `PERFORMANCE_POOL` grows to 73 — carrying
**Bone Splinters**, +11.9% CPU/game, rather than Altar's Reap at +20.2% for the
same paths. An inert 73rd card costs +14.0%, so the tax is the pool slot and
not the mechanic (`fuzz-record.md` §3.1a).
Zero errors, zero panics, zero `Uncast resolved` in all three arms on both
pools. `fuzz-record.md` has the re-recorded table.

**A fix that came with it:** a mana ability whose cost has a generic component
now gets a real allocation prompt. `activate_mana_ability` passed an empty map,
so such an ability would have failed at payment; no registered mana ability has
one, so nothing was wrong in practice and nothing moved.

78. **`can_pay_costs` checks each cost against the same board, so two
    object-moving costs in one list can both pass and only one be payable.**
    "Sacrifice a creature" twice with one creature answers yes twice. The
    first is paid, the second fails, and the mana ahead of them is gone —
    which is the one board on which CR 732.1's cancellation would be needed
    after all, reached by a cost list rather than by anything a player did.

    **Reachability (2026-09-07):** unreachable — no registered card prints
    two object-moving costs in one list, and none of the five CM-3 added
    does. `payment_order_rank` does not help here: both costs are rank 2, and
    the failure is in the pre-check rather than in the order.

    **Sized:** the honest fix is a set-cover pre-check — filters can overlap
    without being equal ("an artifact" and "a creature" over one artifact
    creature), so summing counts per filter is wrong. ~80 lines and a
    matching `plan_payment` change, with the first card that prints two.

79. **CR 601.2h's payment order is the player's and the engine takes it.**
    "First, they pay all costs that don't involve random elements or moving
    objects from the library to a public zone, **in any order**" — the engine
    picks one order for everyone (`payment_order_rank`). `ATOM-601.2h-003`
    is the atom: Omnath, Locus of Mana plus Momentous Fall, where sacrificing
    before paying and paying before sacrificing give different draws, because
    Omnath's power is read off the pool.

    **Reachability (2026-09-07):** reachable — not wrong; a forced choice.
    No registered card makes the two orders differ, and Omnath is not
    registered.

    **Sized:** a `ChoiceKind` over the orderings of one rank, ~70 lines. **The
    constraint it must keep:** only orders that complete may be offered, which
    is CR 601.2h's own "unpayable costs can't be paid" applied to the order.
    Offering an order that bricks the payment is what would make 732.1's
    cancellation load-bearing, and building the cancellation is the
    alternative to that constraint rather than a companion to it.

80. **CR 601.2h's second payment group is not modelled, and nothing was
    scheduled to model it.** "First, they pay all costs that don't involve
    random elements or moving objects from the library to a public zone, in
    any order. **Then they pay all remaining costs in any order.**"
    `payment_order_rank` implements the first group's ordering and the second
    group is empty for every `Cost` arm — none is random and none moves a
    library card — so there is nothing to put in it and an arm the pipeline
    cannot apply would be worse than a missing one. What was missing is an
    *owner* for the day that changes; this item is it.

    **Reachability (2026-09-08):** unreachable — no `Cost` arm qualifies. The
    first one that will is **mill as a cost** (`ATOM-701.17b-002`, Phase 8,
    "can't pay a cost that requires milling more than library size"), which
    moves library cards to a public zone; a cost with "at random" in it is the
    other family and the corpus has no atom for one.

    **Sized:** a rank above `RANK_MOVES_AN_OBJECT` and a second sort key,
    ~15 lines, with whichever arm first qualifies. **The thing to notice when
    it lands:** a group-2 cost is by definition one whose payment cannot be
    predicted, so it is the first cost that can fail *after* a group-1
    sacrifice — item 79's constraint and §3.12's argument both meet it there.

81. **A keyworded additional cost can be mandatory, and `AdditionalCost`'s
    shape says otherwise.** Every named variant is a keyword whose keyword
    makes it optional, so `is_optional` reads as "named ⇒ optional" even
    though it is matched exhaustively. **Spree** (CR 702.172a) is the
    counterexample: "Choose one or more modes. As an additional cost to cast
    this spell, pay the costs associated with those modes" — choosing is not
    optional, and the costs follow the modes.

    Spree is not one variant away, though. Its costs are *per mode*, chosen at
    CR 601.2b along with the modes (CR 700.2), so it needs the modal machinery
    — `StackEntry.chosen_modes` is written and read by nothing — before it
    needs anything from this enum. When it lands, the additional cost it
    contributes is `Mandatory` with a payload assembled from the chosen modes,
    which is the variant behaving correctly rather than a new one.

    **Reachability (2026-09-08):** unreachable — no registered card is modal,
    and `chosen_modes` has no producer or consumer.

    **Sized:** with modal spells (`backlog.md` §2.3's neighbourhood), not
    before. Nothing here changes until then; the note exists so that "keyword
    means optional" is not inferred from the variant list.

82. **CR 704.5p's first sentence is not implemented: an Equipment that becomes
    a creature stays attached.** — ✅ closed, archived.
    **Reachability (2026-09-08):** closed — fixed the same day. …
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by CM-3 —
    lock-in's payment side (2026-09-07)" item 82.

### Found by CM-4 — the mana window and the payer (2026-09-08)

**Shipped:** CR 601.2g's opening condition and CR 605.3a's closing condition,
both explicit; `ui::ManaWindowStop` and `ui::AutoPayer`, two `DecisionProvider`
decorators that clients compose; `--no-auto-pay` on both binaries. Items 70 and
71 close here and item 72 is placed. No new card — §3.11's step 3 builds out of
CM-3's registered five — so `PERFORMANCE_POOL` does not move and
`engineering-practices.md` §3's table is not re-recorded.

**Why two decorators and not one payer with a scope.** The first design gave
`ui::AutoPayer<D>` a `PayerScope` enum so each client could take a subset of
the payment prompts. A scope enum is a closed, hand-rolled enumeration of the
subsets of something that already composes: the second automation — priority
passing, auto-block, an auto-tapper with lookahead — grows it an arm per subset
and makes the payer know about automations that are not its business. Clients
compose a stack instead, and the asymmetry lives in each binary's wiring.
**The stack invariant, written down before the third decorator arrives: one
decorator per `ChoiceKind`.** Disjoint kinds mean composition commutes and
stack order carries no meaning.

**What a payer may answer, as a criterion rather than a list.** **A prompt
belongs to a payer when it has exactly one legal answer**, so being asked
cannot change anything. `OrderCostReductions` always qualifies (§3.4's theorem:
every order gives the identical total); `GenericManaAllocation` only when the
caps admit one allocation; `ChooseSacrificeForCost` never.

The criterion shipped weaker and the owner's review corrected it the same day.
It read "every legal answer leaves the same game state except for mana",
justified by mana emptying at end of step (CR 500.4) — which ignores that
*within* the step the residue is playable resource. The board that settles it:
a `{2}{U}` three-drop cast off three blue sources, where which mana pays the
generic decides whether `{U}{U}` is still up for Counterspell, though the spell
being paid for never asked about blue. **The general shape, and it is the same
one item 83 has:** a local test on one payment cannot see a decision whose
consequences are a turn wide. The strict form survives because when there is
one answer there is nothing to see.

**Measured** (`plans/fuzz_ab.py` plus a fourth binary, 2026-09-08, 200 games
at seed 12345 on both pools). Zero errors, zero panics, zero `Uncast resolved`
and zero turn-limit hits everywhere; three shell runs at one seed identical
outside the timing lines on both pools.

| Arm | vs `main` |
|---|---|
| the engine changes alone, decorator dropped | game-identical: `stress` byte-identical, `performance` differs by **2 memo hits** and no game-state counter |
| as shipped (engine + `ManaWindowStop`) | **1 game of 200 differs on `performance`, 8 of 200 on `stress`** |
| `--no-auto-pay` | differs everywhere by design — the agent taps out on every cast |

**So `cost-architecture.md` §6's prediction was half right, and in the opposite
half from CM-3's.** CM-3 predicted "no new path a pooled card would measure"
and was wrong about the *pool*; CM-4 made the same prediction and is wrong
about the *engine* — the counters moved, and not for the reason the phase was
about. The divergence is item 83 and it is entirely the decorator's, which the
fourth binary is what proved: with the decorator dropped, both engine changes
together move no game.

**The cost of offering the window, which is real and is the phase's price.**
The engine no longer returns before enumerating, so every window that opens
pays one extra `enumerate_activatable_mana_abilities`, one `ChoiceContext` and
one options `Vec`. At 50 games that is memo hits 55,610 → 56,039 on
`performance` (+0.8%) and 57,618 → 58,163 on `stress` (+0.9%), with **layer
walks unchanged at 371** — the extra enumeration hits the memo rather than
walking, which is the good half. CPU/game median 13.42 → 13.53 ms (+0.8%),
inside the ~2.4% run-to-run spread but consistent in direction with the memo
count. There is no cheaper necessary condition for "is there a mana ability to
offer" than the enumeration itself, and the only way to skip it is to not
offer, which is item 70. If it ever matters, its owner is item 77.

83. **A source can tap itself for mana inside its own 601.2g window, and then
    its own `Cost::Tap` cannot be paid.** CR 605.3a lets a player activate any
    mana ability while paying, including one on the very permanent whose
    ability is being activated. `{3}, {T}: …` on a permanent that also has a
    mana ability is the board: tap it for mana in the window, and CR 601.2h
    cannot pay the `{T}`. The activation rewinds correctly under CR 732.1 with
    nothing paid (§3.12's ordering property), so the *outcome* is right in
    every arm.

    **What CM-4 changed is what happens before the rewind.** The engine's old
    stop was `can_pay_costs` over the whole cost list, so once the mana was
    covered and the `Cost::Tap` was not, the window kept enumerating and asking
    — and `RandomDecisionProvider`'s `AnyWillDo` arm tapped land after land
    until it ran out of sources or hit `WINDOW_ACTIVATION_CAP`, all of it spent
    on a payment that could never complete. `ui::ManaWindowStop` declines as
    soon as the mana component is covered, so the rewind happens having burned
    nothing extra. **That is the whole of CM-4's counter movement**, and it is
    strictly the better answer: no number of mana abilities can make a
    `Cost::Tap` payable.

    Traced with a debug build over 200 games at seed 12345: **Chainbreaker**
    once on `performance` (`{3}, {T}`, its mana ability granted by a Layer 6
    effect — the divergence needs a *granted* one there, since no pooled card
    prints both) and **Mind Stone** eleven times across eight `stress` games
    (`{1}, {T}, Sacrifice this artifact`, whose `{T}: Add {C}` is printed).
    `phase_cm_integration_test`'s CR 605.3a tests place the Forest before Mind
    Stone for exactly this reason, with a comment saying so.

    **Reachability (2026-09-08):** reachable — not wrong. Both arms reach the
    same board; only the mana wasted before the rewind differs. It is the mana
    twin of `cost-architecture.md` §3.11's Mind Stone puzzle, reached through
    a mana ability rather than through Krark-Clan Ironworks.

    **Sized:** nothing to build in the engine. An agent that wanted to avoid it
    would exclude the activation's own source from `mana_window_preference`,
    which is `RandomDecisionProvider`'s policy and its own measurement — ~15
    lines, and it would move every counter again, so it wants its own phase and
    its own A/B rather than a ride on this one.

84. **Five free helpers in `ui/decision.rs` had no callers; three are deleted
    and two are recorded.** `auto_allocate_generic` (which also iterated a
    `HashMap` to allocate, so its first caller would have been a determinism
    leak, and which re-subtracted the pips `ask_choose_generic_mana_allocation`
    already clamps — 16c/16d's bug), `queue_tap_and_cast` and
    `is_action_still_valid` (the old stateful RandomDP's pre-tap design) went
    with CM-4, since the payer is the caller each was waiting for and each
    would have been a trap for it. `default_damage_assignment` and
    `default_trample_assignment` are also callerless.

    **Reachability (2026-09-08):** unreachable for the two left — dead code,
    not wrong code. They belong to combat and a combat phase should decide
    whether a DP still wants a default damage assignment offered to it.

    **Sized:** ~~two deletions or two callers, ~20 lines either way~~ —
    **decided 2026-09-18 (A4k): two callers.** The helpers are the body of
    `backlog.md` §2.22 row 5, a `CombatDefaults<D>` decorator answering the
    two division prompts for a human under the full-control toggle, ~60 lines
    after main item 161. Before they become a body they owe one CR read: the
    trample helper's deathtouch branch treats a blocker with damage already
    marked as needing nothing, and CR 702.2c says any *nonzero* amount is
    lethal, so such a blocker still needs one.

    **And the design question the deletions answered**, recorded because it
    will be asked again when the enumeration cost is noticed: reviving
    `queue_tap_and_cast` — pre-tapping at priority so the window opens already
    covered — costs *more*, not less. `legality.rs` runs `castable_spells`
    (which previews every hand card's locked total since CM-1/CM-2) and
    `activatable_abilities` on every priority round, so three taps become three
    heavy rounds in place of three narrow mana-only sweeps. And it is wrong
    independently of cost: pre-tapping commits mana before CR 601.2f locks the
    total, so an agent that does it can never activate a mana ability *inside*
    the window — no Ironworks sacrifice mid-cast, no overpay play.

85. **Nothing bounds CR 601.2g's window against a `DecisionProvider` that
    never stops.** CM-4 removed the engine's stop (item 70) because the CR has
    none: a real Krark-Clan Ironworks loop activates as many mana abilities as
    the player likes, so any engine-side cap would be a rule Magic does not
    have. Termination is now entirely the client's, and every shipped client
    answers it — `ui::ManaWindowStop` declines once the component is covered,
    `RandomDecisionProvider::WINDOW_ACTIVATION_CAP` bounds the fuzz agent even
    without a stop, a human self-polices. A buggy or hostile provider has
    nothing: a filter ability (`{1}`: add one mana of any color) cycles forever
    and changes the pool every round, so even a no-progress check cannot see it.

    **Reachability (2026-09-08):** unreachable — every `DecisionProvider` in
    v1's two use cases is in-process code this repo owns. It becomes reachable
    the day a provider is a network seat, which is Phase 9's multiplayer or
    Phase 10's harness, whichever puts an untrusted party behind the trait.

    **Sized:** not an engine change, and that is the finding. The defense is a
    budget at the session layer — actions or wall-clock per player per priority
    window — because only the session layer can tell a legitimate 200-activation
    combo from a loop, and `run_mana_ability_window` cannot. Whoever adds the
    network seat owns it.

### Found by RD-1 — the damage event's two subjects and its results (2026-09-08)

**Shipped:** `ReplacementDef.affected_players: PlayerSet`; `Rewrite::Amount`
with `Multiplier`, `Halve` and `PreventHalf`, and `Rounding`; `Rider` carrying
an `EventSubject` and the replaced event's amount, with
`AmountExpr::ReplacedAmount` and `Multiply`; `GameAction::LoseLife.cause`;
`Primitive::Mill`; and CR 120.3's results decomposed off the target's effective
types. Five cards registered — Furnace of Rath (pooled), Ghosts of the
Innocent, Gisela, Blade of Goldnight, Angel of Suffering, and the Loyalty Probe
fixture. Item 27 closes here; items 86 and 87 are placed.

**Why 120.3e got a type gate nobody asked for.** Reworded 2026-09-08 after
review found it too compressed to follow.

*Before*, `perform_action`'s damage arm was a two-way `match` on the target and
each arm did one thing: an object had `damage_marked += amount` written on it,
a player had `life_total -= amount`. Two branches, mutually exclusive, and the
object branch asked nothing about what kind of object it was.

*After*, it asks the target which of CR 120.3's results it has. Three
consequences, and only the first was designed:

1) **A creature planeswalker gets two results.** CR 120.3 says damage "has
   **one or more** of the following results". Damage to a permanent that is
   both marks damage (120.3e) *and* removes loyalty (120.3c). Under a two-way
   `match` that needs a special case, because the arms are alternatives; under
   a struct of independent flags it is what the code already does.
2) **A non-creature, non-planeswalker permanent takes no result at all.** This
   is the part nobody asked for. Once each result names the type it belongs to,
   the old unconditional `damage_marked` on *any* battlefield object has no
   rule behind it — CR 120.3e is written about a creature — so it was
   bookkeeping the CR does not have, invisible while the arm was shaped as
   "object or player".
3) It is unreachable from the registered pool, because
   `SelectionFilter::Any` offers only creatures, planeswalkers and players, and
   combat cannot attack anything else. It is pinned by a test anyway
   (`damage_to_a_noncreature_nonplaneswalker_marks_nothing`), because the
   wither, infect and toxic arms land in this same struct and will be written
   by someone reading it.

86. **CR 120.3b, 120.3d and 120.3g are absent — poison from infect and toxic,
    and wither's and infect's −1/−1 counters.** RD-1's `DamageResults` is the
    seam: each is one more flag on that struct and one more block in
    `perform_action`'s `DealDamage` arm, read off the *source's* keywords
    rather than the target's types. The poison half proposes counters on a
    **player**, which `PermanentState`'s counter map cannot hold —
    `PlayerState.poison_counters` exists as a bare `u32` and no proposal
    reaches it, so CR 122.1's chokepoint has no player-side arm.

    **Reachability (2026-09-08):** unreachable — no card in the crate has
    infect, wither or toxic, and the three are keyword flags that do not
    exist. It becomes reachable with the first one registered.

    **Sized:** ~200–300 with the first infect card, per `backlog.md` §2.6,
    which names the three keywords and their CR 120.3 results. The player-side
    counter proposal is the part that is not mechanical; §2.16's player-counter
    map is the same work from the other side.

87. **CR 120.3h is absent — a battle's defense counters — and so is CR 310.**
    Damage dealt to a battle removes that many defense counters, which is the
    fourth missing result and the only one blocked on a *card type* rather than
    on a keyword. `CardType` has no `Battle` arm and nothing in the engine
    knows CR 310 exists.

    **Reachability (2026-09-08):** unreachable — the card type does not exist,
    so no object can be a battle and the arm can never be taken.

    **Sized:** unknown until CR 310 is scoped; `backlog.md` §2.23 owns it and
    was filed 2026-09-08 because no doc owned CR 310 at all. The 120.3h result
    itself is one flag on `DamageResults` and one block, the same shape as
    120.3c; everything else about battles is the size.

### Found by the RD-1 review (2026-09-08)

Fourteen review notes on the RD-1 branch. Most were answered in place; three
changed behavior or left a standing record, and one is a doc-hygiene finding
worth more than the comments it corrects.

88. **A mill of N was N batches, and it should have been one (fixed in the
    same review).** — ✅ closed, archived.
    **Reachability (2026-09-15):** closed — fixed in the RD-1 review that
    found it (2026-09-08), pinned by `a_mill_is_one_batch_of_many_moves`; the
    verdict had said "it was unreachable …", which the board read as unstated
    until the post-RE audit. → `plans/archive/codebase-state-closed.md`.

89. **A comment can state a measured fact and go stale without any code
    changing, and nothing in the process re-reads it.** The RD-1 review found
    three comments in two card files asserting that `fuzz_games::random_deck`
    "filters nonlands by color", with derived probabilities — "roughly one deck
    in sixteen", "about a third of decks". That filter was removed on
    2026-09-03 when the `Everywhere` land landed, and `registry.rs`'s own note
    records the removal. Nothing connected the two: the claims were true when
    written, no test could fail on them, and the card files they justify were
    selected on their basis.

    **Reachability (2026-09-08):** reachable and *actively misleading* — a
    later phase choosing cards on the stale rule would reject a gold card for a
    reason that no longer exists. Corrected in all three places.

    **Sized:** the fix is not a rule about comments, and this is the finding.
    `CLAUDE.md`'s comment rule already says the right thing ("comment the *why*,
    and only where it is not recoverable from the code plus one rule number"),
    and no rule about comment *length* would have caught a claim that was true
    when written. What is missing is a re-read, so it becomes an audit with an
    instrument, on the Deferred Migrations triage's own cadence —
    `engineering-practices.md` §2 now carries it.

### Found by RD-2 — CR 615.7 prevention shields, and the loop's unit (2026-09-09)

**Shipped:** `Primitive::CreateReplacement(def, Duration)`, filling its rows
from a target, a filter recipient (CR 615.11) or as authored;
`Uses::NextDamage`, `AmountRewrite::{PreventUpTo, PreventRemaining}`,
`ReplacementDef::is_prevention()`; `Rider.prevented` and
`AmountExpr::DamagePrevented`; `apply_replacements` in its group form —
decisions per `(batch, subject)`, rewrites per member — with `consume_use`
after `apply_rewrite`; `ChoiceKind::AllocateNextDamage`, `next_damage_shares`
and `GameState::prevention_allocations`; the all-multiplier suppression.
Four cards — Mending Hands (pooled), Samite Healer, Safe Passage, Samite
Censer-Bearer. Items 25, 40, 47 and 65 are updated above; `replacement-
architecture.md` §11 items 22, 24, 29 and 30 close. Trace page:
`plans/traces/rd-2-a-decision-is-per-subject.html`.

90. **A resolution-created row keeps its targets, and nothing reads them
    yet.** `RegisteredReplacementEffect.targets` is written by
    `Primitive::CreateReplacement` because the targets are unrecoverable a
    moment later and Divine Deflection's rider — "deals that much damage to
    any target", chosen at cast — needs them beside the event's subject
    (this item is the record; the handoff that first carried it is gone).
    Threading them onto `ReplacementInstance`
    and `Rider`, and giving `ReplacementDef::then` a recipient leaf that says
    "the thing this effect targeted at resolution", is that card's PR, which
    also needs `AmountExpr::Variable`.

    **The three rulings that pin the shape, carried here so they survive the
    handoff file** (Divine Deflection, verified on Scryfall 2026-09-08; the
    handoff is deleted when Phase RD lands and this item outlives it):

    > Divine Deflection's only target is the permanent or player it may deal
    > damage to. You choose that target as you cast Divine Deflection, not at
    > the time it prevents damage.

    — so a rider's `ResolutionContext` cannot be built from the event's subject
    alone: the damage goes where the *cast* pointed, while the prevented amount
    and the affected player come from the event. The damage is Divine
    Deflection's own and is not combat damage even when the prevented damage
    was, which is what makes this a rider and never a `Retarget`.

    > If Divine Deflection can't deal damage to the targeted permanent or
    > player … it will still prevent damage. It just won't deal any damage
    > itself.

    — the prevention is unconditional (CR 615.12) and the rider's own action
    drops, so the check belongs where the rider resolves and must **not** be
    `Destroy`-style loudness from `perform_action`, which would fail the batch.

    > Whether the targeted permanent or player is still a legal target is not
    > checked after Divine Deflection resolves.

    — so it is an *existence and type* check, not CR 608.2b's legality
    re-check. A rider that asked `validate_selection` would get shroud wrong.

    **Reachability (2026-09-09):** unreachable — no `then` can name a
    resolution target, and no registered rider wants one.

    **Sized:** ~80–120 — a field on the instance and the rider, one
    `EffectRecipient` arm, `resolve_rider` building a context that carries
    both, and the no-op check.

91. ~~**`AmountRewrite::PreventUpTo` has a performer and no printed
    producer.**~~ — ✅ closed, archived.
    **Reachability (2026-09-09):** closed — both printed producers are
    registered and Guardian Seraph is pooled, so the arm is reachable from a
    measured game (145 cast / 145 resolved in 102 of 200 forced `stress`
    games).
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by RD-2 — CR
    615.7 prevention shields, and the loop's unit (2026-09-09)" item 91.

92. **A CR 615.7 count spanning subjects with two choosers is refused, not
    answered.** `next_damage_shares` asserts one chooser across every bucket
    and errors otherwise, because every printed multi-subject count is scoped
    to one player and that player's permanents (Divine Deflection, Harm's
    Way). A hypothetical "prevent the next 3 damage that would be dealt to any
    number of target creatures" across two controllers would need CR 616.1's
    last sentence applied to *shares* — an APNAP round of allocations — which
    the rules do not describe for 615.7 and no card asks for.

    **Reachability (2026-09-09):** unreachable — every registered count's
    buckets share a chooser, and the error names the def.

    **Sized:** unknown until a card asks; the error is the right answer until
    then.

93. **A later group's doubling moves the member, not the allocation.**
    CR 615.7's allocation is taken at the members' then-current amounts, the
    first time the instance is chosen in any group; a later group whose
    CR 616.1 choice doubles a member ahead of the count applies the stored
    share to the doubled amount (`min(share, amount)`). §9's RD decision 3
    records this as CR 616.1's own per-subject ordering showing through —
    Divine Deflection's "you don't decide until the point at which the damage
    would be dealt" is satisfied, since the decision is made at that point for
    the first group and the later group's doubling is its own choice — and
    RD-2's review is where it is to be argued rather than smoothed over.

    **Reachability (2026-09-09):** unreachable from the pool — it needs a
    multi-subject count (item 90's card) beside a doubler on the later
    subject.

    **Sized:** none unless the review reverses the decision; then a re-ask
    when a bucket's amount changes after allocation, ~40 lines in
    `next_damage_shares`.

94. **Kitsune Palliator's "each creature and each player" has no recipient.**
    `EffectRecipient::FilteredPermanents` makes Samite Censer-Bearer's
    per-creature rows; an each-player recipient does not exist, and
    CR 615.11's per-player rows would be the same arm in `CreateReplacement`
    over the player list. One customer, so it waits.

    **Reachability (2026-09-09):** unreachable — no such recipient.

    **Sized:** ~30 — an each-player `EffectRecipient` arm (or a `PlayerSet`
    beside the filter) and one more branch in the primitive.

95. **The row a `Primitive::Regenerate` makes still names the ephemeral
    ability object; a `CreateReplacement` row names the permanent.**
    CR 113.7a makes an ability's source the object that has it, so
    `CreateReplacement` writes `ctx.ability_source.unwrap_or(ctx.source)` and
    Samite Healer's row names the Healer — which outlives the stack object
    CR 608.2n deletes and is what `ask_choose_replacement` offers a UI.
    `Regenerate` keeps `ctx.source`; no registered card regenerates from an
    activated ability, so its rows are never offered under a dead id.

    **Reachability (2026-09-09):** unreachable — `Primitive::Regenerate` has
    no registered producer at all; the row's `source` is read only by the
    CR 616.1 prompt and by `remove_by_source`, which nothing calls.

    **Sized:** one token, in the PR that registers the first regenerating
    card.

### Found by the RD-2 review (2026-09-09)

Fourteen comments on PR #120, captured in `plans/handoffs/rd-2-review.md`,
triaged, and absorbed here — the file is deleted in the review commit, as its
contract says. Eight were answered in place (a doc comment, a rename, a
restructure); three changed the **corpus** rather than the code, which is the
half worth recording; two are deferred with a home and a size (`backlog.md`
§2.24, "Before card breadth" item 11); and one closed item 65 while proving a
claim in it false, which is item 97.

96. **Two atoms moved in opposite directions, and the pair is the finding.**
    `specdb` came out unchanged — 118 full, 50 partial — because one atom was
    promoted and one demoted in the same pass, and the reasoning is the same
    reasoning read from both ends.

    **`COMP-614-616-DOUBLE-REPLACEMENT-001` was promoted to `COVERS`.** RD-2
    had marked it partial on the grounds that "Player A chooses order" is a
    half the engine deliberately does not build (§11 item 29's suppression).
    The review's objection: *if the choice is provably immaterial, that points
    at the atom rather than at the coverage*. Correct — and the atom now
    carries an audit note saying so. CR 616.1e does give the player the
    choice, and *because* multiplication commutes no rules-legal question
    distinguishes the two orders: not the damage, not its source, not the
    event log. An engine that asks and one that does not produce the same
    game, so a test proving 2 → 4 → 8 with each doubler applying once covers
    the atom. The clause that stays observable, and that a test here must not
    drop, is CR 614.5's "each applies once".

    **`BOUNDARY-DEF-615.1a-001` was demoted to `COVERS-PARTIAL`**, found while
    answering a different question ("isn't that test tautological?" — half of
    it was). The atom's out-of-set member is a *triggered ability*, and no type
    in this engine can express one until critical-path item 6, so the test
    substitutes the two nearest expressible non-members: a doubler, and
    regeneration. Regeneration is the load-bearing one — a `Prevent` that is
    **not** a prevention effect, because CR 615.1 is about damage and its
    pattern is a destruction — and the test is now named for it.

    **Reachability (2026-09-09):** nothing owed; `owed` is still 9 and neither
    atom is in a shipped phase.

    **Sized:** none. The out-of-set half of the boundary atom lands with
    item 6.

97. **A stale claim written the same day it was found, and only a reader caught
    it.** — ✅ closed, archived.
    **Reachability (2026-09-09):** closed — the claim is corrected in place and
    item 65 records what it got wrong.
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by the RD-2
    review (2026-09-09)" item 97.

98. **`next_damage_shares` re-checked its own guard, and the second check read
    as a mystery.** — ✅ closed, archived.
    **Reachability (2026-09-09):** closed.
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by the RD-2
    review (2026-09-09)" item 98.

### Found by RD-3 — sources (2026-09-09)

Five items from building CR 609.7's source predicate and the eight cards that
write it. Item 91 is closed here; item 47's condition (d) is re-derived because
this PR added the fields it names.

99. **`SelectionFilter::DamageSource` reaches two of CR 609.7a's four source
    categories, and the other two have named blockers.** Reachable: a permanent
    (`battlefield_ids_ordered`) and a spell on the stack (`StackEntry.is_spell`).
    Not reachable: (a) *an object referred to by an object on the stack, by a
    waiting replacement or prevention effect, or by a delayed triggered
    ability* — the engine has no referred-to relation. `StackEntry.
    chosen_targets` and `RegisteredReplacementEffect.targets` are **targets**
    (CR 115's word), which is a different fact; the atom's own example is an
    emblem naming a card in exile, and CR 603.7's delayed triggers are item 6's.
    (b) *a face-up object in the command zone* — `GameState::command` is never
    populated.

    There is one further gap inside the reachable half, and it is a
    consequence of `resolve_top_of_stack` rather than a decision:
    the spell or ability **currently resolving** has had its `StackEntry` taken,
    so it is on `game.stack` with no `is_spell` to read and is not offered. An
    effect choosing its own resolving spell as the source of future damage is
    the only thing that loses, and nothing printed does it.

    **Reachability (2026-09-09):** unreachable — neither category exists on any
    board the engine can build, and the resolving-object gap is asked for by no
    printed card. `ATOM-609.7a-001` and `BOUNDARY-DEF-609.7a-001` are
    `COVERS-PARTIAL` naming exactly the two categories.

    **Sized:** (a) is a `referred_to: Vec<ObjectId>` on `StackEntry` plus the
    same on a registry row plus CR 603.7 — not before item 6. (b) is one
    enumeration leg once the command zone is populated, ~10 lines, and belongs
    to whichever Commander PR fills it.

100. **`Primitive::DealDamage` proposes one batch, and that is now a property
     other primitives should be checked against.** It looped `execute_action`
     until RD-3, which opens a batch per target — invisible while every damage
     effect in the crate had one target, and wrong the moment Pyroclasm
     arrived: CR 704.3's simultaneity, CR 615.7's "two or more applicable
     sources at the same time" and CR 603.2c's "one or more" all read the
     batch. Fixed with the card that made it reachable.

     **The general form is `CLAUDE.md`'s own rule** — "a simultaneous rule needs
     `execute_actions`, not a loop" — and the audit it implies has not been
     run: `Primitive::DrawCards` loops on purpose (CR 121.2's "one at a time"),
     `Primitive::Mill` batches on purpose (701.17a), and the rest of the
     primitive table has never been asked. Nothing else in the registered pool
     acts on more than one object at a time, so there is no board to fail on
     today.

     **Reachability (2026-09-09):** unreachable — `DealDamage` is fixed, and no
     other primitive has a multi-object recipient, so there is no board on
     which the unaudited arms differ.

     **Sized:** one pass over `resolve_primitive`'s arms with the recipient in
     hand, ~30 primitives, an hour. Worth doing in the PR that gives a second
     primitive a `FilteredPermanents` recipient.

101. **`EffectRecipient::FilteredPermanents` now has two readers with the same
    semantics, and its doc said it had none.** — ✅ closed, archived.
    **Reachability (2026-09-09):** closed — two readers, both tested.
    Full entry: `plans/archive/codebase-state-closed.md`, "Found by RD-3 —
    sources (2026-09-09)" item 101.

102. **Item 47's condition (d) re-derived at RD-3, and the answer is that the
     suppression stands.** The multiplier bucket's premise says
     `ordering_cannot_change_outcome` goes false the day "an
     `EventPattern::DealDamage` field reads the *amount*". RD-3 added the first
     two fields that arm has ever had. Neither reads the amount: `source` is a
     predicate over the object dealing the damage (CR 609.7) and `combat` is
     CR 510.2's flag on the proposal. So no member of a multiplier bucket can
     fall out of applicability as another member changes the number, and the
     debug-build re-gather (`check_order_invariance`) keeps checking it per
     group member on every board a test or a debug fuzz run reaches. The rule
     item 47 states for whoever adds such a field — revisit the predicate in
     the same commit — was followed here; this is the record of it.

     **Reachability (2026-09-09):** nothing owed.

     **Sized:** none.

### Found by the RD-3 review (2026-09-09)

Seven comments on PR #121. Two found a doc claim that was simply wrong — the
`combat` field's "nothing printed asks for `Some(false)`" (nine cards do) and
§3.2a's Torbran paraphrase, which named the object half of the target predicate
and dropped the player half. Both are corrected in place. Three are answered
where they were asked (`replacement-architecture.md` §11 item 34, §3.2a's
battle paragraph, `backlog.md` §2.23). The three below are the ones that become
entries — one of them replacing a claim this review proved wrong — and the list
audit is at the end.

103. **`object_matches_filter`'s `Err` is swallowed wherever a filter is asked
     about an object, and the things that can raise it are card-authoring
     errors.** `set_affects` and three legs of `pattern_watches` — RD-3's
     source side and the two zone-change `object` filters — end in
     `.unwrap_or(false)`. **Four sites, not the three this item said until
     2026-09-09**: the original count named "both legs of `set_affects`" (the
     frame and no-frame branches, which RD-4's fix collapsed into one call) and
     missed the two zone-change legs entirely, which is its own small lesson
     about counting call sites by reading rather than by grepping. The causes
     were: an id with no object behind it, `ObjectFilter::NotSource`, and
     `PowerLE` against an object with no power. Every one of them reads as **a
     card that silently does nothing**, which is the failure mode this
     subsystem's own module doc names first.

     **`NotSource` is off the list from RD-4 (2026-09-09), and it never
     belonged on it.** Palisade Giant's "other permanents you control" made it
     live on its first board — the Giant redirected the damage aimed at *you*
     and none of the damage aimed at your other permanents, because a player
     subject never reaches the object filter. `set_affects` has carried the
     effect's `source` since RB and the layer walk has answered the same leaf
     off `FilterPlayers::source` since the layer system; the two simply were
     not connected. `object_matches_filter_of_source` connects them, and the
     two source-less entry points keep refusing the leaf, which is right for a
     *selection*.

     **The general lesson is about the instrument, not the leaf.** The 600-game
     zero below is evidence about the **pool**, and a reachability zero can
     only retire a concern the pool could have exercised. No card in either
     pool used `NotSource` in an affected set, so the measurement said nothing
     about it. It still holds for the two remaining causes, which are genuine
     authoring errors.

     **Measured before deciding: zero.** The three sites were instrumented and
     run over 600 fuzz games — 200 `stress` at seed 12345, 200 `stress` at seed
     999 with six RD-3 cards forced, 200 `performance` at seed 4242 — and the
     error path was not reached once. So this is a latent authoring trap, not a
     live bug, and the argument for leaving it is that a mid-game panic is
     worse than a card doing nothing.

     The one cause that is *not* an authoring error deserves separating: an id
     with no object behind it is a **source that has ceased to exist**, and
     CR 608.2h's last-known-information would have such a source still match
     its printed colour. Nothing reaches it today — a damage source is alive at
     every registered proposal — but a `Prevent`-on-a-dying-source board would
     answer "not red" where the CR says "red".

     **Reachability (2026-09-09):** unreachable — instrumented at zero across
     600 games on both pools, with the RD-3 cards forced. Re-derived at RD-4's
     close: unchanged for the two remaining causes, and the third is fixed
     rather than measured.

     **Sized:** one change at all four sites or none — a `debug_assert!` on
     the `Err` arm keeps release behaviour and makes a debug run and `cargo
     test` loud, ~10 lines. The LKI half is separate and larger, and belongs to
     whichever phase gives a damage source a way to die first.

104. **The expensive shape is a cheap *repeatable activation*, and the
     registry rows it makes are a fifth of the cost. The first version of this
     item blamed the rows; a control built at the RD-3 review says otherwise.**

     Circle of Protection: Red's `{1}` makes a `Uses::Once` row per activation.
     Forced into every `stress` deck at 200 games it is activated **12,660
     times in 129 games** — about 98 per game it reaches the battlefield,
     roughly 3 per turn of the game and ~5 per turn it is in play. The
     temptation is to read the cost as "rows accumulate and every damage event
     gathers them all", and that is not what is happening.

     **The isolating control is a card with the activation and no row:**
     "Activation Probe", `{1}: <empty effect>` — a throwaway fixture, built,
     measured and deleted, never registered on a branch that lands. Its effect
     is `Effect::Sequence(vec![])` rather than a life gain, because the first
     draft gained 1 life and stretched games from 30 to 44 turns, which is a
     confound rather than a measurement.

     All four forced the same way, 200 `stress` games at seed 12345, **on one
     throwaway build carrying the probe** — so these four rows compare to each
     other and not to the shipped table:

     | forced | Avg turns | Layer walks | Memo hits | Repl. gathers | CPU/game |
     |---|---:|---:|---:|---:|---:|
     | Dark Sphere — one activation, one row | 29.3 | 463 | 66,066 | 508 | 20.1 ms |
     | Guardian Seraph — static, no activation, no row | 29.9 | 463 | 68,802 | 528 | 20.7 ms |
     | **Activation Probe — repeatable, no row, no effect** | 30.2 | **518** | **89,850** | 540 | **26.6 ms** |
     | **Circle of Protection: Red — repeatable **and** a row each** | 31.1 | **533** | **95,351** | 564 | **28.3 ms** |

     **The attribution, and it is roughly 80/20.** Repeatable activation alone
     costs +55 walks (+12%), +21,048 memo hits (+31%) and +5.9 ms (+29%) over
     the static control. The rows on top of it cost +15 walks (+3%), +5,501
     memo hits (+6%) and +1.7 ms (+6%). The mechanism is unremarkable once
     named: an activation is a priority action, an object on the stack, and two
     more priority rounds before it resolves, and every priority round
     enumerates candidate actions — so the game does *more of everything it
     already does*. Queries rose 31% while walks rose 12%, which is the layer
     memo doing its job rather than a pathology.

     **So: no, this does not argue for coalescing the rows** (asked at review —
     run-length encoding, or one row with a count). It would target the 20%,
     and it is rules-wrong besides. Each row is its own CR 614.5 identity;
     `Uses::Once` × N is not `Uses::NextDamage(N)` (CR 615.8 is "the next
     instance regardless of amount", 615.7 counts damage points, and merging
     would change the answer); the rows almost never agree anyway, because each
     activation chooses its own CR 609.7a source; and CR 616.1 offers the
     player *instances*, so merging changes what is offered. The lever that
     would matter is on the other 80% — indexing watchers by event kind so a
     gather consults only the bucket that can match, which `replacement-
     architecture.md` §8c already budgets and no phase has needed yet.

     **Asked at review: should the fuzz harness cap repeated activations of one
     ability? No.** (1) It would make every counter a function of a policy
     knob, which is what `engineering-practices.md` §3 keeps out of the table —
     every historical row would become incomparable. (2) The random agent is
     not biased toward the ability: `candidate_priority_actions` lists it
     **once** however many times it could be paid for, so what the numbers show
     is leftover `{1}` having nothing else to buy. The probe confirms it — an
     ability that does *nothing* is activated just as freely, so this is
     general agent behaviour and not a fact about the Circle. (3) The observed
     rate is already ~3–5 per turn, the range a 3–5 cap would impose.

     **Reachability (2026-09-09):** reachable — not wrong; a cost observation
     on `--pool stress`, and the pooled table is unaffected today.

     **Sized:** none for the harness. A per-ability activation counter in the
     report is the wanted diagnostic (~20 lines) and is what would have
     answered §9's Circle prediction without a `--dump-events` grep. The §8c
     index is its own piece of work and has no consumer yet.

105. **The pool has never contained an ability that can be activated more than
     once in a turn, and that is an accident to correct rather than a policy to
     keep.** Audited at the RD-3 review, because item 104's first draft said
     "the pool boundary is what protects the cost instrument", which reads as a
     rule and would be a bad one.

     **The audit.** Seven registered cards have a non-mana activated ability;
     two are pooled (Chainbreaker, Merfolk Thaumaturgist) and both are
     `{T}`-gated, so at most once per turn each. Dark Sphere sacrifices itself.
     Circle of Protection: Red is the **only** card in the crate whose ability
     can be activated repeatedly within a turn, and it is not pooled.

     **Why it is not pooled, accurately.** `engineering-practices.md` §3's rule
     is one card per new engine path, chosen deliberately; RD-3's new path is a
     source-side filter evaluated per damage event, and Guardian Seraph is the
     cheapest card that opens it. The Circle was not excluded for cost — §9's
     sentence about its `{1}` competing for mana was a *reachability* guess
     (wrong, see item 104), never a pooling criterion.

     **Reading it as a policy would cascade, and the cascade is the wrong
     way.** Repeatable activated abilities are ordinary Magic — mana sinks,
     equip, pump, and most of what a commander does — and v1's target is
     4-player Commander, where they are most of the late game. A cost
     instrument that structurally excluded them would drift from the thing it
     exists to predict, which is the opposite of §3's purpose. The measurement
     is a *prior*, not a bar: the shape costs ~+29% CPU when forced into every
     deck, forced is the worst case by construction, and pooled normally it is
     one card among 76.

     **Reachability (2026-09-09):** reachable — not wrong; the pool is
     representative today because nothing has needed this shape, and it stops
     being representative the moment Phase 8 breadth arrives.

     **Sized:** none now. The action is a rule for later, and it is the rule
     §3 already has: the first phase whose engine path *is* an activated
     ability pools one deliberately and re-records the table, with item 104's
     numbers as the expected direction rather than as a reason to decline.

### Found by RD-4 — redirection and unpreventable damage (2026-09-09)

106. **`RetargetSpec::ToFixed(DamageTarget)` is the one arm §9 designed that
     RD-4 did not build, and it is a *feature* on the triage rather than a
     fact.** §9's RD-4 section names four arms; three shipped. A "fixed"
     destination is Harm's Way's "any target" and Divine Deflection's — both
     chosen **at cast**, which is the reason it is absent rather than an
     oversight: a card file cannot author a target it has not chosen, so the
     only thing that could fill the arm is
     `RegisteredReplacementEffect.targets` threaded onto the instance, which is
     item 90's work. Building the arm now would put an unreachable variant in a
     closed algebra, which §3.2's growth contract forbids for exactly the
     reason it would be dead.

     **Reachability (2026-09-09):** unreachable — no registered card, and no
     registrable one, can author a `ToFixed`.

     **Sized:** one enum arm, one match arm in `apply_rewrite`, and the
     resolution-side fill, which is item 90's — so it is item 90 plus ~20
     lines, and it lands with the first card that needs it. Not stubbed: the
     arm does not exist, so no card can silently do nothing.

107. **CR 614.9's re-check is an existence-and-type check, and the temptation
     to make it `validate_selection` is a real one with a printed answer.** A
     redirect's destination has to be on the battlefield and still a creature,
     planeswalker or battle, or a player still in the game — and that is
     *all*. It is not CR 608.2b's legality re-check and never becomes one,
     because **a redirect is not targeting** (CR 115.1): a hexproof or
     shrouded creature is a perfectly good destination for redirected damage.
     Divine Deflection's ruling draws the identical line from the rider's side
     (item 90), and a `validate_selection` at either site would get shroud
     wrong in the same way.

     Written as `pipeline::redirection_is_legal`, taking *both* ends because
     the rule names both — "redirected to **or from** a player who has left the
     game". The "from" leg is unreachable in a two-player game and is built
     anyway, because it is one `||` of a sentence the engine either implements
     or does not.

108. **~~A player who has left the game keeps their permanents, and CR 800.4a
     says they should not~~ — ✅ CLOSED 2026-09-13 (RE-7).** CR 800.4a's four
     clauses run inside the `PlayerLoses` performer, where the rule puts them,
     and the four-player run's "Departed-owned permanents" row is 0.0.
     Full entry: `plans/archive/codebase-state-closed.md`, "Found by RD-4 —
     redirection and unpreventable damage (2026-09-09)" item 108.

109. **The `NotSource` fix stopped one site short of the sites that have a
     source, and the biggest one is `Primitive`'s filter recipient.** RD-4 gave
     `set_affects` a source to answer `ObjectFilter::NotSource` against
     (item 103). Four callers of `targeting::object_matches_filter` remain
     source-less, and **only two of them are genuine selections** —
     `validate_permanent_target` and `costs.rs`'s cost-candidate filter, which
     have no effect source and are right to refuse the leaf. The other two do
     have one and do not pass it:

     - `resolve.rs`'s three `EffectRecipient::FilteredPermanents` sites
       (`DealDamage`, `CreateReplacement`, one more) hold `ctx.source`. This is
       the one with printed customers: **85 cards say "each other creature you
       control"** and 146 say "each other creature" (Scryfall, 2026-09-09).
       Today such a filter matches nothing and the card silently does nothing —
       Palisade Giant's bug at a different site.
     - `pipeline.rs`'s CR 614.13 auxiliary-move filter holds `chosen.source`.
       No printed customer: devour's payload is "creatures you control" and
       says nothing about "other".

     `pattern_watches`' three legs are a different question and stay refused:
     the function takes the effect's *controller* and not its source, and a
     source-side "other than me" predicate is Sokrates' one-customer shape that
     §8c already told RD-3 to record rather than build.

     **Reachability (2026-09-09):** unreachable — no registered card writes
     `NotSource` into a `FilteredPermanents` recipient, which is exactly the
     kind of zero item 103 has just been corrected for reading too broadly. It
     is evidence about the pool, and the pool has 85 printed candidates waiting
     outside it.

     **Sized:** three one-line changes in `resolve.rs` (and one in
     `pipeline.rs`) to `object_matches_filter_of_source`, plus one registered
     card that says "each other creature you control" to test it — so ~10 lines
     of engine and one card. It belongs with the first phase that writes that
     card, not with RD-4, whose scope is redirection. Not stubbed: the filter
     exists and answers `false`, which is the silent-card failure, so this line
     is the record that it does.

### Found by RE's sizing (2026-09-11)

No code. Four facts about the tree the census found while counting RE's sites
(`replacement-architecture.md` §9, Phase RE, "The census"), each owed to a
named RE PR.

111. **~~Mana production is a direct write with no event.~~ — ✅ closed
     2026-09-15 (RE-9).** — archived.
     **Reachability (2026-09-15):** closed — `GameAction::ProduceMana` is the one
     event, `perform_action`'s arm the one writer, and `GameEvent::ManaAdded` is
     emitted; `--dump-events` has mana lines from here on.
     Full entry: `plans/archive/codebase-state-closed.md`, main item 111.

112. **~~`has_drawn_from_empty_library` is set and never cleared.~~ — ✅ closed
     2026-09-12 (RE-6).** The check reads every player's flag into the batch
     and clears it, beside `last_sba_check_epoch`, because CR 704.5b's window
     is the same sentence as 704.6d's — "since the last time state-based
     actions were checked". **At the check and not in the performer**, which
     the sizing left open: a replaced loss (Exquisite Archangel) and a refused
     one (Platinum Angel) both perform nothing, and a flag cleared by the
     performer would have re-proposed the same loss at every later check. The
     Archangel's ruling is the test
     (`a_replaced_empty_library_loss_is_not_proposed_again_until_the_next_draw`).
     *Original entry:* `zones.rs:135` set it, `sba.rs:130` read it, no site
     cleared it; unobservable only because the one reader ended the game.

113. **A player who has lost stays in the priority rotation.** ~~The turn
     half closed 2026-09-11 (RE-1)~~: `GameState::next_turn_taker` reads
     `player_lost` and passes over a departed player, which is CR 800.4k ("if
     a player who has left the game would begin a turn, that turn doesn't
     begin") at the one site that can say it — ahead of the pipeline, because
     a turn that does not begin is not an event a replacement effect could
     have replaced. A queued extra turn for a lost player is popped and
     discarded there too. **What is left is CR 800.4j**: the priority loop
     still rotates `(priority_player + 1) % n` with no `player_lost` read.
     The rotation half of "Before Commander" item 4, separated because RE-6
     builds the `PlayerLoses` performer and a performer that leaves the player
     in the order is the two-player shape wearing an N-player event; 800.4a–e
     (their objects) is RE-7's, the PR after — item 108.

     **Reachability (2026-09-11):** unreachable — `fuzz_games` plays two
     (`fuzz_games.rs:827`); reachable from `test_support::setup_game(4)`, and
     the turn half is now covered there
     (`phase_re1_integration_test::a_lost_players_turn_does_not_begin`).

     **Sized:** ~20 lines at the one remaining site, RE-6, beside the
     `--players 4` fuzz mode item 4 sized at ~50.

     **✅ The priority half closed 2026-09-12 (RE-6).** `run_priority_round`
     starts from the active player if they are still in the game and from
     `next_player_in_game` after them otherwise, rotates through that
     function, and ends a round when everyone *still in the game* has passed
     — plus the three turn-based actions a departed active player has nobody
     to perform (attackers, the draw, the cleanup discard), so the turn
     "continues to its completion without an active player". CR 104.1 landed
     at the same loop: nobody receives priority in a game that has ended,
     where before a player who had just lost kept acting until the phase
     ended. `--players 4` exists and is measured (item 108). What is left of
     item 4's multiplayer list is CR 800.4a–e (RE-7) and CR 802.

114. **`Restriction::Event` has no player set.** `{ pattern, affected, by }` —
     the object set only — so "players can't gain life" (Skullcrack, Leyline
     of Punishment; 25 cards), "you can't lose the game" (Platinum Angel; 11)
     and "your opponents can't win" (9) cannot be written as rows.
     `ReplacementDef.affected_players` (RD-1) and
     `Restriction::ApplyReplacement.to_players` (RD-4) are the same field on
     the other two types.

     **Reachability (2026-09-11):** unreachable — no registered card prints a
     player-scoped "can't" over a proposed event; RD-4's fixture used
     `ApplyReplacement`.

     **Sized:** one field plus a `set_affects`-style union in
     `is_prohibited`'s `Event` arm, ~40 lines; RE-3, read by RE-6.

     ~~**Closed by RE-3 (2026-09-12)**~~ — `affected_players: PlayerSet`, unioned
     by the same `set_affects` call `ReplacementDef` and
     `Restriction::ApplyReplacement` use. Skullcrack is registered and lands the
     row in a game; Leyline of Punishment's static form is the extended RD-4
     fixture. The sizing's "~40 lines" was right for the engine and missed the
     constructions: 12 literal ones plus one exhaustive destructuring, because
     an enum variant cannot take a `..Default::default()`.

115. **`turn_rotation` is a second cursor beside `active_player`, and
     nothing enforces that they agree.** RE-1 added it because CR 500.7
     inserts an extra turn *after* a turn, so the natural rotation has to
     resume from the player whose natural turn it was; `active_player` is
     whose turn it is *now*, and an extra turn moves one without the other.
     Any code that writes `active_player` directly and then crosses a turn
     boundary gets that player's turn twice — which three test fixtures did,
     and `test_support::set_active_player` is the answer for fixtures. **No
     production writer exists outside `begin_turn`**, which is why this is a
     recorded hazard rather than a bug.

     **Reachability (2026-09-11):** unreachable in production — `begin_turn`
     is the only production writer of `active_player` and `next_turn_taker`
     the only writer of `turn_rotation`. Reachable from any new fixture.

     **Sized:** the honest fix is to make `active_player` private behind
     `begin_turn` and `turn_rotation` private behind `next_turn_taker`, ~15
     call sites in tests; a `debug_assert` in `advance_turn` that the two
     agree is wrong, because an extra turn is exactly when they do not. Do it
     when a second production writer wants to exist, not before.

116. **Extra phases and steps — CR 500.8, 500.9, 500.10 — and this is a
     pointer.** Filed here at RE-1's close and re-filed twice at its review.
     First to `backlog.md` §2.17, because `state-of-play.md` draws the line
     this got wrong: a Deferred Migration is *one code change* owed by
     scaffolding already in the tree, and a backlog entry is *one mechanic* the
     engine will need. Then **CR 500.8's half graduated to
     `replacement-architecture.md` §9, RE-10**, which is where its design,
     sizing and card now live. CR 500.9/500.10's half stays in §2.17 and is
     item 6's, because Obeka is a triggered ability.

     **Reachability (2026-09-15):** nothing owed — a record pointing at the
     two docs that own the halves: the phase half landed with RE-10 (below),
     and the step half is `backlog.md` §2.17's, waiting on item 6. (The
     2026-09-11 verdict said "nothing to build", which the board does not
     read as a class; re-worded at the post-RE audit.)

     **Sized:** RE-10 is ~1,100–1,300; the step half is one
     `Option<Vec<StepType>>` field and waits on item 6.
     `replacement-architecture.md` §11 item 49 is the finding that turned a
     deferral into a decision.

     **✅ The phase half landed 2026-09-14 (RE-10), at +1,066 / −171.**
     `GameState.turn_plan` is CR 500.1's sequence as data; `next_phase`'s chain
     is deleted. **The position is now two facts** — `phase`, which everything
     reads, and `turn_plan.cursor`, which the drainer reads — and
     `GameState::set_position` is the only seam that writes both. That is the
     hazard item 115 above describes, one level down and *with* the enforcement
     item 115 argues against for its own pair: a `debug_assert` in
     `advance_turn` is right here precisely because, unlike `active_player` and
     `turn_rotation`, these two never legitimately disagree at a drain
     boundary. It found all 40 affected fixtures in one run. **The step half
     stays in §2.17**, and `PlannedPhase` is the struct its one field goes on.

117. **An untap-step skip would not reset land drops.** `process_untap_step`
     calls `reset_lands_played` where CR 502 puts the untap step's turn-based
     actions, and RE-1 made the untap step skippable: eight printed cards say
     "skip your untap step", and under one of them a player's
     `lands_played_this_turn` never returns to zero, so they play no land for
     the rest of the game. "Until your next turn" moved to `on_turn_begin` for
     exactly this reason (CR 611.2b says the turn); the land drop did not,
     because no rule calls it a turn-start action and RE-1 registered no
     untap-step skip to make it reachable.

     **Reachability (2026-09-11):** unreachable — no registered card skips the
     untap step. Reachable the day one is registered, which is Phase 8's
     breadth or whichever PR wants Eon Hub's siblings.

     **Sized:** one line moved into `on_turn_begin` plus the CR citation that
     justifies it, ~10 lines; the care is that CR 505.5b counts land plays per
     *turn* and no rule places the reset, so the move needs an argument rather
     than a hunch.

118. **CR 514.3a's repeated cleanup step announces nothing.** — ✅ closed,
     archived.
     **Reachability (2026-09-15):** closed — fixed at the post-RE audit's
     close-out: `Game::run_turn`'s 514.3a loop proposes the second occurrence
     through `begin_step` ahead of the cleanup actions, so a refused one runs
     none; `state_based_actions_at_cleanup_begin_a_second_cleanup_step`
     failed against the pre-fix tree and pins it; the A/B at two and four
     seats was identical on every row. → `plans/archive/codebase-state-closed.md`.

119. **CR 103.6's "begin the game with this on the battlefield" has no
     implementation, and RE-1 made the seam explicit.** Leyline of the Void is
     registered with the clause recorded as dead text
     (`phase_rb_cards::leyline_of_the_void`), and `PermanentState`'s
     `control_since_turn = 0` is already documented as the pregame sentinel for
     it. The insertion point is now a named place: `Game::setup`, between
     CR 103.4's opening hands and `GameState::start_first_turn`. Gemstone
     Caverns' ruling gives the ordering — *"the starting player takes all such
     actions first in any order, followed by each other player in turn order.
     Then the first turn begins"* — and Gemstone Caverns is the harder shape,
     because it is conditional on not being the starting player and has a cost
     (exile a card from your hand).

     **Reachability (2026-09-11):** unreachable — the clause is a static
     ability functioning in the *hand*, which is
     `replacement-architecture.md` §3.3's source 2 and needs CR 113.6 (critical
     path item 6a). Leyline of the Void is castable for `{2}{B}{B}` and does
     nothing before it resolves, which is the whole of today's behaviour.

     **Re-derived (2026-09-14, LK) — the prerequisite is in and this is now
     ordinary unbuilt work.** LK landed CR 113.6, and the hand-zone lookup this
     item was waiting for exists: an ability may state
     `Condition::SourceInZone(ZoneSet::HAND)` and `zone_function::functions_in`
     answers for it. **The clause deliberately did not ride** (§13d decision
     5): what is missing is not a zone question but CR 103.6's *moment* — a
     step between `Game::setup`'s opening hands and `start_first_turn`, a
     `DecisionProvider` question, and Gemstone Caverns' ordering ruling. An
     ability that functions and then has nothing to happen is the unapplyable
     arm `CLAUDE.md` bans, so the registered Leyline keeps its second clause
     only and that is a correct card rather than a stub.

     **Reachability (2026-09-14):** unreachable — still no pre-game step, and
     no longer blocked on 6a.

     **Sized:** ~60 lines in `Game::setup` plus a `DecisionProvider` question
     per eligible card per player. **Unblocked as of LK.** The care is the
     ordering ruling above and CR 103.6's interaction with mulligans, which are
     themselves stubbed.

120. **`AbilityDef` has no named constructors, and five copies of two of them
     live in three card files.** `static_replacement` and `one_shot` are each
     written twice (`phase_rd_cards`, `phase_re_cards`) and `static_ability`
     once (`phase_li_cards`) — one shape, three spellings: build an
     `AbilityDef` with a fresh id, `is_characteristic_defining: false` and
     `ActivationRestriction::None`. Every card file a later phase adds writes
     it again.

     **Not `test_support`**, which is the obvious home and the wrong one: it is
     behind the `test-support` feature and release builds turn it off, while
     `src/cards/` ships. The **big test-file migration will not catch these
     either** — they are card files, not test files.

     **Reachability (2026-09-11):** reachable, not wrong — five correct copies
     of one constructor. It is a divergence risk rather than a defect: the day
     two of them disagree about `is_characteristic_defining`, one card file's
     abilities quietly stop being CDAs.

     **Sized:** named constructors beside the type in
     `objects/card_data.rs`, the way `ReplacementDef::new` sits beside
     `ReplacementDef` — three functions and ~30 call sites across three card
     files, ~120 lines net negative. **Its own PR**, so a mechanical sweep does
     not ride inside a rules change.

     **Asked and answered 2026-09-14** (`refactor/object-set-rename`, which is
     item 124's rename): this does **not** ride along, though both are
     mechanical and neither is a rules change. A rename can be reviewed by
     checking one claim — every hunk is the same substitution — and proved by
     byte-identical fuzz counters. Named constructors are new API, and the two
     questions they raise are what they are called and which `AbilityDef` field
     each bakes a default into, which is the very thing this item says turns a
     style choice into a rules bug. That diff has to be *read*, and putting it
     inside one that only has to be *scanned* costs the rename its review
     method while the rename's own proof says nothing about the constructors.

121. **~~Eon Hub's two trigger-shaped rulings have no test and cannot have one
     until item 6.~~ — ✅ CLOSED 2026-09-19 (TR-1).** — archived.
     `under_eon_hub_an_upkeep_trigger_never_triggers` and
     `under_eon_hub_an_untap_trigger_goes_on_the_stack_at_the_draw_step`
     in `tests/phase_tr1_integration_test.rs`, each carrying its `// RULING:`
     line; Verdant Force is the upkeep trigger and a fixture the untap one.
     **Reachability (2026-09-19):** closed — TR-1.
     Full entry: `plans/archive/codebase-state-closed.md`, "Item 121".

### Found by RE-2 — draw (2026-09-11)

**Shipped:** `GameAction::{DrawCards, DrawCard}` as CR 121.2a's instruction and
CR 121.1's draw, with `DrawCause`, the two `EventPattern` arms,
`GameActionTemplate::DrawCards { n, player }` in both its legs, and the outer
performer's decomposition handing each inner the applied set its own CR 616.1
loop accumulated — `execute_actions_inheriting` beside `execute_actions`, and
`apply_replacements` returning the group's applied set beside the members'
decided events. `GameState::draw_cards` deleted. Four cards — Thought
Reflection (pooled), Teferi's Ageless Insight, Alms Collector, Notion Thief.
Item 29 closes; `replacement-architecture.md` §11 items 18 and 42 close and
items 50–56 open, of which 53 is the one worth reading. CR 121.6c went to
`backlog.md` §2.26 rather than to this section: nothing was scaffolded for it,
so it is a mechanic the surface cannot express and not debt. Trace page:
[`plans/traces/re-2-a-draw-carries-its-lineage.html`](traces/re-2-a-draw-carries-its-lineage.html).

122. **CR 121.2c's two-player draw order is unexpressible, and RE-2 shipped its
     first customer.** *"If more than one player is instructed to draw cards,
     the active player performs all of their draws first, then each other
     player in turn order does the same."* Alms Collector's rider — "instead
     **you and that player** each draw a card" — is the first effect in the
     crate that instructs two players to draw, and it is an `Effect::Sequence`,
     which resolves in the order the card's text was written. When the affected
     opponent is the active player the two draws come out backwards.

     **Reachability (2026-09-11):** reachable, wrong today, and only in the
     event log. Alms Collector is registered and not pooled, so no fuzz game
     reaches it; a fixture does, and the order is asserted nowhere because
     asserting it would freeze the wrong answer. It becomes gameplay-visible
     the day item 6 lands "whenever you draw a card", where two players'
     triggers would go on the stack in the wrong order.

     **Sized: not one line.** The facility is APNAP ordering over *an effect's
     recipients*, and `Effect` has no arm that says "these atoms are one
     instruction to several players" — a `Sequence` is CR 608.2c's instruction
     sequencing, which is deliberately *not* reordered. The two candidate
     shapes are a recipient-plural draw primitive
     (`Primitive::DrawCards` with an `EffectRecipient::Filter`-style player set,
     ordered by `apnap_index` at resolution, ~40 lines and one new recipient
     reading) or a `Effect::Simultaneous` arm that sorts its atoms by chooser
     the way `apnap_batch_order` already sorts a batch (~60 lines, and a second
     ordering rule beside the batch's). CR 121.2d's shared-team-turns variant
     is a third leg on whichever lands. **One customer today**, which is why
     neither is built: §8c's "two customers before a leaf", applied to an
     ordering rule rather than a filter.

     **Scheduled (2026-09-15, post-RE audit):** critical-path item 6's
     architecture doc must carry CR 121.2c's recipient ordering —
     `roadmap-v2.md` A6's row says so now — because "whenever you draw a
     card" is the rule's first gameplay reader, and the choice between the
     two shapes below is that doc's to make with its trigger ordering.

     **Narrowed 2026-09-11, at RE-2's close.** Alms Collector's rider turned out
     to be one draw and not two — CR 614.5 forced the affected player's half
     into the rewrite (item 53 there) — so the order is no longer the card's
     text order but a structural one: the replaced event is performed, then the
     rider (§4.1a). That is still not CR 121.2c's, and it is now wrong in a
     narrower and more predictable way: the affected player always draws first,
     where the rule says the active player does. The facility is unchanged and
     so is the sizing.

     → `replacement-architecture.md` §11 item 52. ~~**Owner: RE-6**, which is
     where turn order stops being `(0..n)` because a lost player has left it.~~
     **Re-owned 2026-09-12, at RE-6's close.** RE-6 did make the rotation
     read `player_lost` (`GameState::next_player_in_game`), and that is not
     this item: the facility here is APNAP ordering over *an effect's
     recipients*, which §9's "Out of RE" declines on the same one-customer
     argument as before — Laboratory Maniac's second ruling is the second
     customer, and it is unexpressible for the same reason. **Owner: the
     first each-player draw producer**, wherever Phase 8 lands it; the
     rotation it will sort by exists now.

### Was critical-path item 5 done, and what sits before item 6? — audited 2026-09-15

Asked by the owner the day RE-9 merged (PR #140) and critical-path item 5
closed with it: twenty-four replacement PRs had landed between 2026-08-25 and
2026-09-15, interleaved with CM-0–CM-4, LH, LI, LJ, LK, CV-1 and RS-1, and the
next spine item was a doc nobody had sized. Planned as
`plans/handoffs/post-re-audit.md` (PR #141), run as five passes on
2026-09-15, and closed by the pass that deleted the handoff — this heading is
its record, pointers not prose. The handoff's last text is `git show
341ebf9:plans/handoffs/post-re-audit.md`; the practice it became is
`engineering-practices.md` §9, which names each pass's instrument and is
where the next run, at item 6's close, starts.

**Yes, with a "not yet" list, and the list is the CR 614–616 row's.** The
close-out read "done" off five places and collected it in one; nothing it
found was a wrong pipeline, and three things it found were wrong claims —
the TL;DR's line and test counts, `replacement-architecture.md` §13's "✅
through RB", and `owed` scoped to three older phases at every RE close.

- **Pass 1, the close-out (PR #142):** the board's classifier — "reachable
  but not wrong today" had been read as wrong, RD-1's prose list counted as
  three items, three verdicts worded so the board could not read them; the
  Phase 6 `owed` triage, `backlog.md` §3.3's second block, 21 `NEW` → 0 and
  Phase 6 in `SHIPPED_PHASES`; item 118 fixed, shown to fail first, A/B
  `IDENTICAL`; `replacement-architecture.md` §8a's four kinds dispositioned,
  §11 items 3, 4 and 14, §12 re-read, §13 current, §14 written; this file's
  "Found by the post-RE audit (2026-09-15)" — item 134 — and items 59, 60,
  122 and 131 scheduled, 88 and 118 closed, 116 and 121 re-worded.
- **Pass 1b, the eviction (PR #151):** `replacement-architecture.md` 6,725 →
  3,401 lines, `plans/archive/replacement-architecture-landed.md` 4,933 →
  8,647; every heading byte-identical, every §11 item still at its own
  number.
- **Pass 2, hygiene and CI (PRs #143, #144 re-landed by #145, then
  #146–#149):** clippy counted at 114 sites and adopted at `-D warnings`
  with five lints allowed; `check_glossary.py` in CI; the toolchain floor
  measured at 1.88 (`rust-version`) with the pin kept at 1.98; `cargo fmt
  --check` costed and left to the owner; `engineering-practices.md` §2.1's
  two recorded applications, 17,278 → 15,806 comment lines; twelve `TODO`s
  re-owned, `backlog.md` §2.32 filed for one.
- **Pass 3, readiness (PR #153):** items 138–143 below — the target as a
  ratchet with its first reading, the retry re-prompt's stale list, the
  round-resume entry point, the serialization boundary and the payload rule,
  the panic surface with `plans/panic_surface.py`, the clone at Commander
  scale; `engineering-practices.md` §3.1's per-PR budget; `layers-architecture.md`
  §12's callgrind subsection and the levers ranked by it; `backlog.md`
  §2.22's ask table; item 41 promoted to a requirement and its RNG question
  decided; item 69 closed.
- **Pass 4, scheduling (PR #154):** `roadmap-v2.md` §3a's A table, the one
  path-to-triggers table, extended in proposed order by the between-phases
  rows (the `A4x` family) with what each owes the doc, and the B rows with
  their atoms; §3b explains it; `engineering-practices.md` §9; the codebase map and the Rust notes
  homed at `roadmap-v2.md` row A4d; this heading, and the handoff deleted.

**The board at the close** (`state-of-play.md`, 2026-09-15): 193 items, three
reachable and wrong today — 59, 60 and 122, each with an owner — and none
with reachability unstated. The ratchet's first reading is item 138's table:
10,000 decisions per core-second on the 60-card `performance` board at four
seats and 7,950 at Commander scale, this machine, that day; the instruction
count that travels is `layers-architecture.md` §12's.

**What the audit changed in the rules, each where it lives:** §2.1's tighter
grep tier and its history-word block list; §3.1's 2.5-point budget and the
ratchet; §9 itself; clippy and the glossary gate in CI; `SHIPPED_PHASES`
gaining Phase 6; the board reading "not wrong today" as its own row. Nothing
changed in `CLAUDE.md`, by its own rule.

**What it left open, each with an owner and a size:** the engine items pass 3
sized — item 138's two levers and two counters, 139 with item 41's test, 140,
141's two methods, 42's summaries and window — ordered in `roadmap-v2.md`
§3b; the two after-the-passes artifacts at row A4d, neither started; the
three wrong-today items.

**Found on the PR's review (2026-09-16):** the UUID review, item 144 below —
both `Uuid` ids replaced by process-stable ones, decided by the owner, sized,
and `roadmap-v2.md` A4g's row.

**Next:** at critical-path item 6's close, per `engineering-practices.md` §9,
its first duty the ratchet's second reading against item 138's first.

### Deferred Migrations — is the list still working? Audited 2026-09-09

Asked at the RD-3 review, on passing 100 numbered entries and having gained a
dedicated `backlog.md` since the last check. Measured rather than felt.

**What is working.** Every entry added since the 2026-09-03 triage carries a
dated `**Reachability:**` verdict and an explicit `**Sized:**`, and
`check_state_of_play.py` derives the board from them rather than from prose —
153 items, 111 of 115 open ones sized, and only **3** without a stated
reachability, all of them pre-triage. The classification is doing its job: the
two bolded rows on `state-of-play.md` ("reachable, wrong today" = 2) are the
ones a reader acts on, and they have stayed small.

**What is not.** Two things, and neither is about classification.

1) **Nothing is ever evicted.** 37 items are closed and still carry their full
   text — some at 10 KB — so roughly **a third of the section is finished
   work**, and the section is now 5,169 of `codebase-state.md`'s 5,384 lines
   (96% of the file). The value of a closed item is its *reason*, which is one
   paragraph; the rest is the record of a fix that `git log` already holds.
2) **`backlog.md` arrived and nothing moved into it.** Its §1 draws the line —
   a *mechanic the type surface cannot express* is backlog, an *implementation
   debt in code that exists* is here — and several entries here are on the
   wrong side of it by that test.

**Done in this PR, on the owner's call.** 37 verdict-closed items evicted to
`plans/archive/codebase-state-closed.md`, each leaving a three-line stub at its
own number. **85,092 bytes out, 11,500 back in stubs**; the section goes
5,382 → 4,816 lines and the file 5,597 → 5,031.

Four things are worth keeping from doing it rather than proposing it:

1) **The dated verdict is the only safe selector, and the `✅` in a heading is
   not.** "Before Layers" item 8 reads "CR 613.8 dependency … ✅ done" and the
   2026-09-03 triage lists it among the five items that were *reachable and
   wrong*; LI-2 closed it three days later, and only the verdict says so. A
   sweep driven by headings would have archived a live bug on 2026-09-04.
2) **Two closed items still carry a `**Sized:**` that reads as owed** — main
   item 5's "one function, `cast_spell`, plus wherever the deferred event is
   flushed" and "Before Triggered abilities" item 4's "~300–500 additions".
   Both are pre-closure residue, hand-checked against their verdicts before
   moving. The rule that falls out: **a closure should strike the size, not
   just add a verdict.**
3) **The ids being section-scoped is what made this cheap.** Stubs stay at
   their own numbers inside their own sections, so none of the 477 "item N"
   citations moved and step (5)'s prefix sweep is neither helped nor hurt.
4) **The line projection above was wrong** — "under 3,500 lines" assumed the
   removed items were wrapped prose, and the oldest of them are single very
   long lines. The section is 96% → 95% of the file, because the section *is*
   the file; the win is 566 lines and 73 KB of finished work a reader no longer
   scrolls past, not a change in that ratio.

The audit's own conclusion stands and is now half-acted-on: the triage works,
and the eviction exists as of today — as a step every closing PR takes, per the
section header.

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

1) **"706"** is Rolling a Die in `tmnt.txt`. The intended section is **708,
   Face-Down Spells and Permanents** — CR 613.2b's Layer 1b, 10 uncovered atoms.
2) **"2,890 double-faced cards"** is `is:dfc`, and **2,243 of those are
   `layout:art_series`** — art cards with a signature on the back, not playable
   Magic and with no copiable values to model. With `double_faced_token` (80) and
   `reversible_card` (71) also removed, the rules-relevant population is **517**.
   The conclusion survives; the scoping argument does not survive a 5.6×
   overstatement, which is the difference between "schedule it" and "schedule it
   first". `copy-census.py --decompose` prints the breakdown.
3) **CR 729, Merging with Permanents, was missing** and CR 613.2a names it in the
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
| 704.5m/n Aura SBAs; 608.3c Aura ETB attach | `engine/put_on_stack.rs` never reads `enchant_filter` (0 references). A cast Aura reaches `resolve_popped` with no targets and the Aura branch errors. **Registering an Aura today would produce fuzz errors**, not coverage. Sibling of item 8 |
| 704.5d token cease-to-exist | `Primitive::CreateToken` is a stub |
| 704.5q counter annihilation | `Primitive::AddCounters` / `RemoveCounters` are stubs |
| 704.5p Equipment detach | ✅ LH-2 (2026-09-06) — 15 per 200 stress games |
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
Zombie tokens. Rules: `plans/engineering-practices.md` §3; both baselines:
`plans/fuzz-record.md`.

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
| 704.5p Equipment detach | `EquipmentDetached` on an Equipment subject | **0** → **15** (LH-2, 2026-09-06: 7 Bonesplitter, 8 Cobbled Wings) | **none** — Equip landed; Mirrorform copying a non-creature onto the equipped creature fires it. Count the subject: 5 more lines that sitting were the CR 704.5n catch-all on Mirrorform'd Holy Strengths |
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
CR 601.2c castability pre-check), the inline block in `engine/put_on_stack.rs`
(CR 601.2c target selection), and `engine/stack.rs::extract_recipient` (the
CR 608.2b fizzle). They are three copies of the same fourteen lines, so the fix
is one shared helper that takes the object rather than a fourth copy.

**But fixing it still would not make an Aura registerable, which is the new
finding.** No `ObjectSet` can name the permanent an Aura is attached to:
`static_object_set` has exactly two productive arms, `FilteredPermanents` →
`Filter` and `Implicit` → `SourceOnly`, and `register_static_effects` runs
inside `place_on_battlefield` — *before* `resolve_taken` attaches the Aura, so
even `Fixed` has nothing to capture. `Duration::WhileEnchanted` exists and has
no consumer. Every faithful Aura's text is about its host, so the Aura half
needs a layers change on top of item 8 and is a phase, not a card drop. Auras
were left out of this PR on that basis, and the phase is now written up and scheduled: **Phase LH**, `layers-architecture.md` §13a, before critical-path item 7. **The `ObjectSet` half is small** -- one arm in `effect_applies_to`, resolved during the walk exactly as `ByController` is, so registration running before the attach is not the problem it first looked like. The cost is CR 613.7e instead; see "Before card breadth" item 4.

**`attach_aura_on_etb` (`engine/resolve.rs`) is dead code**, found the same way:
zero production callers, reached only by its own three unit tests. It implements
CR 303.4g's *choose a host on entry*, which is the right shape for an Aura put
onto the battlefield without being cast — a path no card can take yet. Left in
place rather than deleted, and recorded here so the next reader does not mistake
it for the live path (`engine/stack.rs`'s Aura branch is). **Deleted with LH-1
(2026-09-04):** it was the second attach writer, which is what item 8 is about,
and whatever first returns an Aura to the battlefield brings CR 303.4f/g back
with its consumer.

**Also registered: `adaptive_shimmerer`.** Its own doc comment said "this one
grows the stress pool" and `registry.rs` never registered it, so the claim was
false from the day RC-2 landed — the same silent gap as an unregistered card,
one level down. Registering it makes the doc true and adds a colorless 0/0 that
lives only on its counters, which is the sharpest board CR 704.5f has.

**Gate, both pools, 2026-09-01:** 200 stress games at seed 12345 `--threads 1` —
0 errors, 0 panics, 0 turn-limit hits. Determinism re-checked three runs at one
seed on both pools, byte-identical outside `=== Timing ===`. Fixtures re-recorded
in `plans/fuzz-record.md`; **both columns moved, because both pools gained
Battlegrowth**, so nothing in that table is comparable across this commit.

**The Aura row closed 2026-09-04 (LH-1, `layers-architecture.md` §13a).** Same
instrument, same 200 stress games at seed 12345: `[AuraSba]` **0 → 23**, and
60 under `--require "Holy Strength"`. Both blockers the row named are gone —
`ObjectSet::Host` reaches the host, and `targeting::spell_recipient`
reads the enchant ability — but only one of the two paths they blocked is
*reached*: CR 704.5m/n fires because hosts die in combat, while the CR 608.3b
fizzle, covered by `tests/phase_lh_integration_test.rs`, went **0** in the same
200 games with and without `--require`. The random agent never answers an Aura
by killing its target; the closest it came was three Counterspells. That is a
statement about the agent, recorded here so the next reader does not count the
fizzle as fuzz-covered. The Equipment row closed with LH-2 (2026-09-06): 15 per 200 stress games on Equipment subjects (7 Bonesplitter, 8 Cobbled Wings), beside 5 catch-all detaches of Mirrorform'd Holy Strengths; the two Equipment re-equip 269 and 322 times in the same 200 games.

### Phasing (CR 702.26) — sized 2026-08-26, not started

Recorded because it reads as a large unknown and is not one. **CR 110.5 settles
the shape:** "A permanent's status is its physical state. There are four status
categories, each of which has two possible values: tapped/untapped,
flipped/unflipped, face up/face down, and **phased in/phased out**."

`PermanentState` already carries all four — `tapped`, `flipped`, `face_down`,
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

1. **Pre-layer P/T shim — ✅ done.** — archived.
    **Reachability (2026-09-03):** closed.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 1.

2. **Direct `CardData` reads — ✅ done (2026-08-19).** — archived.
    **Reachability (2026-09-03):** closed — 2026-08-19.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 2.

3. **~~Cost modification pipeline stub — ❌ still a passthrough.~~ ✅ CM-1
    (2026-09-07, `plans/cost-architecture.md`).** — archived.
    **Reachability (2026-09-07):** closed — CM-1. …
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 3.

4. **Mana-pool persistence stub — ❌ still stubbed.** `engine/turns.rs`'s `on_phase_end` and `on_step_end` still pass `BlanketPersistenceSet::none()`; since 2026-09-15 the comment there points here rather than at the archived `T12c`. The registry it needs now exists.

   **Reachability (2026-09-03):** unreachable — `BlanketPersistenceSet::none()`
   in `on_phase_end` and `on_step_end`; no registered card grants mana persistence.

   **Sized:** read the persistence set off the continuous-effects
   registry (a `ManaPersistence` row kind) at the two sites, ~40–60 lines, with
   T12c.

5. **Timestamps — ✅ live.** — archived.
    **Reachability (2026-09-03):** closed — timestamps are live; the 613.8 half
    is item 8 below.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 5.

6. **Direct `card_data.abilities` reads — ✅ done (2026-08-20, Phase LD Part
    B).** — archived.
    **Reachability (2026-09-03):** closed — 2026-08-20.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 6.

7. **Static-ability effect existence — ✅ strip half done (2026-08-21); grant half open.**

   `register_static_effects` still runs once at ETB off printed abilities, and still cannot use the layer system there (it runs inside `place_on_battlefield`, before the object's own effect is registered). That turned out not to be the thing to fix. **Registry membership is not effect existence.** The CR resolves existence *inside the layer walk* — `compute_characteristics` gathers the effects that apply at each layer and does not gather one whose generating ability is gone. Each effect applies at most once and the pending set only shrinks, so it terminates structurally; there is no fixpoint and nothing to cap.

   - **Stripped ability — fixed.** `ContinuousEffect.origin` (`EffectOrigin::StaticAbility { ability }` vs `Resolution`) makes existence askable, and `compute.rs::static_ability_still_exists` re-asks it at every layer against the source's frame *as of the end of the previous layer*. Blood Moon stripping a land now retires the effect that land registered at ETB. Gated on `RegistryScopeSummary::any_ability_changing` so it costs nothing on boards where no registered effect can change an ability set — `fuzz_games` at 200 games / seed 12345, measured back to back against `main` on the same machine: 27.59 → 28.62 ms/game, inside the documented ±3% run-to-run band. (Measure it that way — readings drift several ms across sessions, enough to invent a regression that isn't there.)
   - **Granted ability — ✅ half done (2026-08-23, Layer 6 phase); filter half open.** `GrantAbility(Box<AbilityDef>)` exists, and when a *resolution* grants a static-bodied ability, `resolve::register_granted_static_effects` registers the effects that ability generates then and there. Their `origin` is `StaticAbility { ability: <granted id> }`, so existence needs nothing new: the CR 604.2 check already re-asks at every layer whether the grantee still has the ability, and retires the derived effect when a later Layer 6 strip takes it away (`test_stripping_a_granted_ability_retires_the_effect_it_generated`).

     Eager registration works because a resolution's affected set is locked to its targets (CR 613.7b), so the grantee set is known at grant time. **What stays open is the case where it is not:** a *static* ability that grants a static ability over a filter — "enchanted creature has 'creatures you control get +1/+1'" — has a grantee set that changes with the board. That still needs the original plan here: the gather step in `apply_effects` unions the registry with effects derived from each permanent's frame-as-of-previous-layer, same loop, same frame cache, same CR 613.6 started-applying set.

   **Reachability (2026-09-03):** unreachable (the grant-over-a-filter half; the
   strip half and the resolution-grant half are closed) — no registered static
   ability grants a *static-bodied* ability over a filter: Citanul Hierophants'
   grant, the only `GrantAbility` in `src/cards`, has a mana body.

   **Sized:** the gather step unions the registry with rows derived
   from each permanent's frame-as-of-previous-layer, same loop and frame cache,
   ~150–250 lines in `apply_effects`, plus CR 613.7a's timestamp for a grantee
   set that moves with the board; lands with the first such card — an
   Aura-granted static after LH, or an Archetype-class lord.

7a. **Frame cache is live.** — ✅ closed, archived.
    **Reachability (2026-09-03):** closed — and the cross-call half is
    critical-path item 7a, PR #92.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 7a.

7b. **CR 613.7a clause 2 — ✅ implemented (2026-08-23).** — archived.
    **Reachability (2026-09-06):** closed — LI-1, for the layer 6 case. …
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 7b.

7c. **CR 613.6 "existence persists once started" — implemented, untested.** — ✅
    closed, archived.
    **Reachability (2026-09-06):** closed — LI-2, tested. …
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 7c.

7d. **`ContinuousEffect { id: 0 }` as "unassigned" — code smell, 16 sites (recounted 2026-08-24).** `ContinuousEffectRegistry::add` overwrites the field, so every construction site carries a meaningless value. The fix is a `ContinuousEffectDraft` that `add()` consumes, which changes `add`'s signature and every site — its own small refactor.

    **Reachability (2026-09-03):** reachable — not wrong; a smell, and it grew:
    26 sites now (16 at the 2026-08-24 recount).

    **Sized:** `ContinuousEffectDraft` consumed by `add`, the
    struct plus ~26 mechanical sites, ~100 lines; a quiet PR of its own, pairing
    with main item 64's rename.

7f. **Conditional static abilities — ✅ done (2026-09-06, LI-3).** `Effect::Conditional` lowers to its inner atom's rows, registered unconditionally, and the condition stays on the ability where CR 604.2's existence check already looks: `board::static_ability_still_exists` evaluates it against the pass's *live* board at the row's layer, so a condition reading types sees layer 4 applied. No field on `ContinuousEffect` and no second registry (`layers-architecture.md` §13b decision 5). `engine/layers/condition.rs` is the evaluator; `Condition` gained one leaf, `HostMatches(ObjectFilter)`, for "as long as enchanted permanent is …". **Kird Ape** is the consumer and is in `PERFORMANCE_POOL`. The historical note below is the reason the answer had to be an existence check rather than a gate, and it is kept.

    **Dog Umbra, the worked example this item carried, is still two systems away** — "As long as another player controls enchanted creature, it can't attack or block. Otherwise, this Aura has umbra armor." The condition and the `Host` recipient both exist now (this item and LH-1); what is left is `Effect::Restriction` under a conditional, which lowers to no rows on purpose (CR 101.2 reads the effective ability list instead, so the *condition* has to be read there rather than here — RS-2's), and umbra armor, a replacement effect. (Umbra armor is CR 702.89a; 702.89b renamed the older "totem armor" wording in Oracle, so use umbra armor.)

    **What the arm does *not* do, and why that is the right reading.** The condition is not consulted at registration: a card whose condition is false as it enters still registers its rows, and they apply to nothing until it becomes true. Consulting it at ETB would need a re-registration hook on every board change a condition can read, which is the reconciliation `CLAUDE.md` warns about — existence is decided in the layer walk or it is decided in an oscillating loop outside it.

    Historical note, because it cost a round trip: an `EffectModification::can_change_abilities()` gate briefly skipped the CR 604.2 existence check when nothing in the registry could change an ability set. It was worth 5-8x, and it was **removed** — it was valid only while no static ability is conditional, and it would have failed globally rather than arm by arm once they are. A rules engine has nothing to trade for a silently wrong answer. **The expiry fired on 2026-09-06**, when this item closed and Kird Ape entered the pool. The gate's premise was "if no registered modification can *change an ability set*, no ability can have gone away, so skip the check". Existence now has a second conjunct — the condition holds — whose truth is a function of the board (which permanents are where, who controls what), not of what the registry writes. Kird Ape alone on a battlefield registers nothing that touches abilities, so the gate would fire, the check would be skipped, and the Ape would keep +1/+2 with no Forest in play: a wrong answer on a pooled card, silently. **A repaired gate is possible and is not worth having** — add "and no registered row's generating ability is `Effect::Conditional`" and the premise holds again, computable at registration time. It would simply never fire on the boards that matter, since `PERFORMANCE_POOL` now contains a conditional card; and it would still be a shortcut whose validity is a property of the card pool, which is what §12's item 3 rules out on principle rather than on measurement. The measurements are `layers-architecture.md` **§12** (a whole section, not an item — the 2026-08-21 table, whose third column is the walk with the gate removed, 5.2x at N=10 rising to 8.0x at N=80), and the answer-preserving alternatives are that section's "The ordering that follows" list, whose item 3 is this shortcut's own post-mortem.

    **Built as sized**, and the size was right: ~340 lines for the evaluator
    with its tests, ~90 in the existence check and the channel table, ~25 in the
    lowering. The attachment-host `ObjectSet` LH brought is what
    `HostMatches` reads through.
7e. **Derivation silently drops non-`Fixed` amounts — ✅ done (2026-08-22).** —
    archived.
    **Reachability (2026-09-03):** closed — 2026-08-22.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 7e.

7g. **A static ability that grants a static ability registers no continuous effect (found 2026-09-06, LI-3).** `register_static_effects` lowers `Primitive::GrantAbility` to a layer-6 row and stops. The rows the *granted* ability itself generates are `resolve::register_granted_static_effects`' job, and that function has exactly one caller — `Primitive::GrantAbility` resolving. So an Aura reading "enchanted permanent has 'Equipped creature has flying'" puts the ability on the host's frame and nothing else happens: the equipped creature does not fly. Verified on the board before LI-3's fixture was written, which is why the fixture is Rune of Flight's *third* line rather than its fourth.

    **Not a missing call.** A resolution knows its grantees — `collect_battlefield_targets` names them once and they never change. A static ability's grantees are its `ObjectSet`, decided per pass: `Host` moves when the Aura is reattached, `Filter` gains and loses members every time the board does. So the derived rows would have to be re-derived per pass rather than registered once, which is a new kind of row (one whose source is another row) and a new question for CR 613.7a clause 2's timestamp. **Related to but not the same as** item 9's zone-reaching `ObjectSet`.

    **Reachability:** unreachable — no registered card is a static ability granting a static ability, and it cannot become one quietly for the *lowering*, which is loud; it becomes one quietly for the *behaviour*, which is exactly this item. **Cards it blocks:** Rune of Flight's Equipment clause, and the "enchanted/equipped permanent has '[static]'" shape generally.

    **Sized:** a per-pass derivation step in `board::applications_in_layer` that reads layer 6's granted abilities off the live frames and produces their rows in the same layer, plus the CR 613.7a clause-2 timestamp for a source that is itself a row, ~150–250 lines. Its own PR, and it wants a test board where the grant and the derived effect are ordered against a third layer-6 effect.

7h. **Two epoch bumps the 7a memo is owed (recorded 2026-09-03).** — ✅ closed,
    archived.
    **Reachability (2026-09-05):** closed — LH-2. …
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 7h.

8. **CR 613.8 dependency — two known-wrong cases, both Blood Moon — ✅ done
    (2026-09-06, LI-1 + LI-2).** — archived.
    **Reachability (2026-09-06):** closed — LI-2. …
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 8.

12. **The card → registry lowering is loud — ✅ done (2026-08-23).** — archived.
    **Reachability (2026-09-03):** closed — 2026-08-23. …
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 12.

9. **Abilities granted to cards outside the battlefield — ✅ done (the
    zone-reaching half 2026-09-14, LJ, `layers-architecture.md` §13c; the
    CR 613.7d half 2026-09-14, LK, §13d).** — archived.
    **Reachability (2026-09-14):** closed — both halves. LK put CR 613.7's
    timestamp on `GameObject`, so an object has one in every zone it enters
    (613.7d) and `static_effect_timestamp` reads it there; Wonder is the card
    that needed it and is registered and pooled. **Nothing of this item stays
    owed.** What LK *added* rather than closed is `PermanentState::timestamp`
    as a measured copy for the ordered sweeps — item 77 below owns deleting
    it, and §13d decision 2 has the +16.5% that put it back.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 9.

10. **The `Layer` enum is missing a sublayer split — doc/code drift.** (The CDA half of this item is ✅ done, 2026-08-22; see below.)

    **Layer 1a / 1b — and this entry had the two backwards until 2026-08-29.** `tmnt.txt` says **613.2a Layer 1a: Copiable effects** (copy effects, CR 707, and merging, CR 729) and **613.2b Layer 1b: Face-down**, i.e. copy *then* face-down. The enum has a single `Layer1Copy`. Not reachable today — nothing produces a layer 1 effect. `LAYER_ORDER` in `engine/layers/compute.rs` mirrors the enum, so splitting it later just lengthens that array; the frame-cache ceiling is an index into it, computed at runtime.

    **Both remaining sites corrected 2026-09-02 (CV-1): `compute.rs`'s `LAYER_ORDER` doc and `layers-architecture.md` §7 with its `Layer` code block.** `Layer1Copy` still collapses the two, and now has a producer in 1a; CV-6 splits it, and `layers::copy::END_OF_LAYER_1` is the one integer the split moves — pinned by a `debug_assert` that fires the moment `LAYER_ORDER[1]` stops being `Layer2Control`. The three sites had shared one wrong derivation: *"a Clone copying a face-down creature copies the 2/2 colorless characteristics, not the printed card (CR 707.2)"* is a **correct conclusion from a wrong premise**. The 2/2 does not come from face-down applying first; it is CR 708.2's own "Any listed characteristics are the copiable values of that object's characteristics", with CR 708.10 covering the copy-of-a-face-down case directly. Where it bites is exactly one integer: `copy-effects-architecture.md` §4.6 captures copiable values at the end of layer 1, and a reader who believes 1b is copy takes the ceiling one sublayer too early — a bug that appears only on boards with a face-down creature being copied.

    **~~Keywords are abilities, and we model some of them as markers.~~ ✅ resolved (2026-08-23, Layer 6 phase).** The old entry framed this as "`Primitive::GrantKeyword(Equip)` would set a flag and grant no ability", which understated it. CR 702 has 189 keyword abilities and they do not want one representation. The axis is **does the engine branch on the keyword, or execute it**, crossed with whether it takes a parameter: ① branch/no-param is a flag (flying, trample, vigilance); ② branch/param is a set of *values* (protection from [quality], [type]walk); ③ execute/no-param is a plain `AbilityDef` (storm, prowess, **devoid** — already modelled this way in `phase_le_cards`); ④ execute/param is an `AbilityDef` with args (equip [cost], ward [cost], cycling [cost]).

    `KeywordAbility` was renamed **`KeywordFlag`** and narrowed to quadrant ① — 16 variants, every one consumed by combat, SBA, casting or damage. `Enchant`, `Equip`, `Landwalk`, `Protection` and `Ward` were removed; none had a single construction anywhere in the crate, and there was no exhaustive `match` over the type, so it cost nothing. The full quadrant map lives in the type's doc comment, which is where the next person will need it.

    Why it mattered *here* rather than at Phase 8: the printed case is the common one. `equip {3}` on every Sword and `equip {1}` on Skullclamp can't put the cost in a fieldless variant, and CR 702.6d lets a permanent hold several equip abilities, which a `HashSet` of one variant structurally cannot express. Enchant was worse than unmodelled — it duplicated `CardData::enchant_filter`, which already works and is what the Aura targeting path reads. And leaving `Protection` in place would have made `GrantKeyword(Protection)` look like the way to write "target creature gains protection from the color of your choice" — common Magic, and inexpressible.

    Where the removed five go when their mechanics land: Equip → `AbilityType::Activated` with its cost; Protection and Landwalk → quadrant-② frame fields (`protections: HashSet<Quality>`, `landwalk`); Ward → `AbilityType::Triggered`, so it waits on CR 603; Enchant → nothing to build.

    **Residue, all small and all recorded rather than built:** quadrant ② has no frame representation (build it with the first card that needs one); naming an ability by its keyword wants a separate complete-CR-702 `KeywordName` enum, a different type doing a different job, wanted by UI display and keyword-matters cards; and `KeywordFlag::Hexproof` is CR 702.11's fieldless base form, while "hexproof from [quality]" (702.11d) is quadrant ② and unmodelled.

    **~~Keyword counters carry no timestamp.~~ ✅ fixed (2026-08-23), in the same PR that introduced it.** CR 122.1b keyword counters are layer 6 effects and CR 613.7c timestamps every counter, so they have to interleave with the layer's registry rows rather than follow them — Humility with a later timestamp than a flying counter really does strip that flying.

    `PermanentState::counters` is now `HashMap<CounterType, CounterStack>`, carrying a count and a timestamp. CR 613.7c's second sentence — "each counter of that kind receives a new timestamp identical to that of the new counter" — is what makes one timestamp *per kind* exact rather than a simplification, and it is applied on every add. `GameState::add_counters(id, kind, n)` is the entry point; `PermanentState::add_counters` still takes an explicit timestamp because it cannot allocate one.

    The sizing that made this look expensive was wrong and worth recording as a lesson: "17 `add_counters` call sites" counted 16 tests as if they were cost. There was **one** production caller (planeswalker loyalty at ETB) and **one** direct `.counters` reader outside `battlefield.rs`. Count production call sites, not grep hits.

    Layer 7c still applies its +1/+1 and -1/-1 counters after the registry slice rather than merging. That is now justified rather than approximated: every layer 7c effect is an addition, so the layer is order-independent. If a non-commutative 7c effect ever exists, it needs the same merge layer 6 has.

    **CDAs — ✅ done (2026-08-22),** and not the way §6 designed. `Layer::Layer7aCdaPT` is in the enum and in `LAYER_ORDER` (now 10 entries). Tarmogoyf and Culling Drone (Devoid) are in `cards/phase_le_cards.rs`.

    §6 planned `ContinuousEffect.is_cda` plus CDA-first partitioning of each layer's registry slice. **The registry holds no CDAs at all.** CR 604.3a(3) — a CDA "does not directly affect the characteristics of any other objects" — is a criterion, not an observation, so every CDA applies to exactly the object that has it. There is nothing for an `ObjectSet` to select. `engine/layers/cda.rs` applies them off the object's own effective ability list at Layers 4, 5 and 7a, ahead of that layer's registry slice, and `ContinuousEffectRegistry::add` asserts nothing registers into 7a. CR 613.3's ordering, CR 604.2's existence check, and CR 613.8a(c)'s first clause all fall out of that rather than being built.

    **This unblocked item 9 rather than depending on it.** The old claim here — "CR 604.3 makes CDAs function in all zones, which ties it to item 9 as well" — was wrong. Item 9 is about *filter-based* effects reaching other zones; a CDA has no filter, and `compute_characteristics` reads `game.objects`, so a Tarmogoyf in a graveyard has a power and toughness with none of item 9's work. `get_effective_power`/`get_effective_toughness` dropped their battlefield gate accordingly.

    **Provenance (CR 604.3a(2)) is not on the flag.** `AbilityDef.is_characteristic_defining` carries only the four criteria that are properties of the ability's text. Whether the ability was *printed on the object it affects* depends on how it got there, and the same `AbilityDef` can arrive by several routes — so it is maintained by whoever writes the ability onto the object. Copy (Layer 1) and text-changing (Layer 3) effects hand the def over whole, which is exactly what 604.3a(2) wants. **A future Layer 6 `GrantAbility` must clear the flag on the def it grants:** a granted ability is never a CDA however its text reads. That is the one debt this design leaves, and it belongs to the Layer 6 phase.

    Still open: **CDA↔CDA dependency** (613.8a(c)'s second clause) — see item 8.

    **Reachability (2026-09-03):** unreachable (the residuals) — `Layer1Copy` is
    still one variant and nothing produces a face-down permanent; quadrant ②
    keywords have no frame representation and no registered card; "hexproof
    from" is unmodelled.

    **Sized:** the 1a/1b split is CV-6's, ~30 lines
    (`END_OF_LAYER_1` moves and its `debug_assert` fires until it does);
    quadrant ② with its first card.

11. **Filter `PlayerRef` resolution — ✅ done (2026-08-23), ahead of Layer 2.**
    — archived.
    **Reachability (2026-09-03):** closed — 2026-08-23. …
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 11.

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

    - **`PermanentState.controller` was read directly at 20 sites; all 20 migrated** to `oracle::characteristics::get_effective_controller` / `controls`. Same shape and same silent-failure mode as Phase LD Part B's 21 `card_data` reads. **No `// PRE-LAYER ZONE:` exemptions were tagged** — that class is cast-zone and play-from-hand legality, which runs before the object is a permanent, and every site here asks about something already on the battlefield or the stack.

      Two needed more than a substitution. `stack.rs::resolve_top_of_stack` reads the controller *before* the pop, because a spell's controller lives on the `StackEntry` and the pop destroys it — one value now feeds both the `ResolutionContext` (CR 608.2) and the entering permanent's controller (CR 110.2b). `turns.rs`'s untap sweep needs two passes, since the predicate is a `&self` layer query and the untap is a `&mut self` write.

    - **CR 302.6 lives in the frame, not on the battlefield.** `PermanentState.controller_since_turn` could not be maintained: control from a continuous effect is derived, so an `UntilEndOfTurn` steal reverts at cleanup with no mutation to hang an update on and no event to hook. `EffectiveCharacteristics.control_since_turn` is computed beside the controller it describes, which is what makes reversion need nothing at all — the value stops being computed when the row leaves the registry. The battlefield field survives as the seed, owning every control change that is *not* a Layer 2 effect (entering the battlefield, today the only one).

      The Layer 2 arm advances it only when control actually moves, because CR 302.6 asks whether control was *continuous* and gaining control of your own creature is not a change. Act of Treason legally targets your own creature and would have hidden this behind its haste clause.

      **One honest gap, and it is unobservable.** Strictly, CR 302.6 makes a creature sick for its *original* controller the instant a steal expires, since control was interrupted during the turn. We report it as not sick. The only window between expiry and that player's next turn beginning is the cleanup step itself, where no player receives priority (CR 514.3) and no ability can be activated, so modelling it would mean recording an interruption nothing can read.

    - **Layer 2 reaches the stack.** `compute::base_controller` — now the single definition of the pre-Layer-2 seed, with three callers where there used to be three copies — has a `StackEntry` arm, which is CR 108.4's other half. `collect_controllable_targets` is the Layer-2-only sibling of `collect_battlefield_targets`: every other continuous effect describes a characteristic a permanent has, but control is the one thing a spell also has.

    - **Perf, interleaved, `fuzz_games --games 200 --seed 12345`.** `main` 79.7 → this branch 83.1 ms/game (9 rounds), of which ~1.8% is the migration measured with Act of Treason unregistered so the card pool matches, and the rest is the card being cast. **Under item 11's +7% ceiling.** The gate, however, is now worth **+28%** rather than 4% (83.7 → 107.2 with it forced off) because it went from protecting one call site to 21. The **sharper per-object gate item 11 proposed was built, measured and discarded**: 83.3 vs 82.6 ms/game over 5 rounds, inside the spread of either column and worse on the median, because `ObjectId` is a v4 UUID and the set probe costs a SipHash at every migrated call site on every board to save on the rare one. Do not rebuild it without a board that keeps the registry-wide flag true for a long time. Determinism holds: three runs at one seed byte-identical apart from wall-clock lines.

    - **Cross-call memoization deliberately did NOT land here** (`layers-architecture.md` §12 item 2, scheduled between this phase and 613.8). Three reasons: §12 requires the paranoid recompute-and-assert mode in the same commit, which is phase-sized on its own; +4% does not force it; and its invalidation key wants designing against item 8 step 4's board-wide sequential pass, which does not exist yet. Landing it before that pass risks building a cache for the wrong computation.

    **Reachability (2026-09-03):** unreachable (the residual; the layer closed
    2026-08-23) — the "an opponent gains control" lowering is still missing, no
    card of the nine is registered, and `fuzz_games` plays two players, so
    `PlayerRef::Opponent`'s above-two-players assert never fires.

    **Sized:** `Primitive::GainControl` takes a recipient and the
    resolution arm asks the `DecisionProvider` when more than one opponent
    exists, ~80–120 lines with the first of the nine cards.

14. **The targeting-side `ObjectFilter` could not resolve a `PlayerRef` — ✅
    done (2026-08-23).** — archived.
    **Reachability (2026-09-03):** closed — 2026-08-23.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 14.

15. **The corpus named a card that does not exist.** — ✅ closed, archived.
    **Reachability (2026-09-03):** closed — a record; the corpus was corrected.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Layers (CR
    613) — now DURING Layers" item 15.

16. **One `EffectGroup` per static ability, so one locked set and one
    CR 613.8 application — a static whose atoms have different recipients
    would share them.** `Board::started` (CR 613.6) and `board::Kind::Effect`
    (CR 613.8's unit of ordering, LI-2) both key on
    `EffectGroup::StaticAbility(source, ability)`, so an ability authored as
    one `Sequence` over two filters — "creatures you control get +1/+1 and
    creatures you don't control get -1/-1" as one `AbilityDef` — is one
    effect, one bundle and one locked set, and its second atom would apply to
    the first's set. The CR reads such text as two effects.

    **Reachability (2026-09-06):** unreachable — every registered static
    ability whose body is a `Sequence` (Humility, March of the Machines,
    Opalescence) has one recipient across its atoms; the other `Sequence`s in `src/cards` are
    resolutions, which lower through a different path. Authoring the shape
    as two `AbilityDef`s gives the CR's answer today, and the loud lowering
    (item 12) does not catch this one because both atoms lower fine.

    **Sized:** `EffectGroup::StaticAbility` gains the atom's index in the
    ability (~30 lines: `ContinuousEffect::group`, the two static
    registrations, `would_be_rows`, `register_copied_static_effects`), or
    the lowering splits a multi-recipient `Sequence` into one group per
    recipient; with the first card that needs it.

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
  static-effect registration fire) vs `place_bare` (inserts a `PermanentState` directly,
  firing neither). The combat tests need the second; collapsing them would put rows in the
  continuous-effects registry those tests do not expect.
- `put_on_battlefield` backdates entry to turn 0 (not summoning-sick) vs
  `put_on_battlefield_this_turn`, which does not. `GameState::new` starts at `turn_number:
  1`, so `layers::cda`'s two-argument version really was the second one, not a shorter
  spelling of the first.
- `registered` (`ObjectSet::Fixed(vec![id])`) vs `registered_source_only`
  (`ObjectSet::SourceOnly`). These agree in `effect_applies_to` when the source is the
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

   **Reachability (2026-09-03):** unreachable — no Vehicle is registered
   (`CardType::Vehicle` exists, no card uses it), and no registered effect
   removes the creature type from a permanent with printed P/T.

   **Sized:** a battlefield-only type gate in
   `get_effective_power`/`get_effective_toughness`, ~10 lines plus a test, with
   the first Vehicle.

2. **"Any player may activate this ability" is unmodeled (CR 602.1a).** `engine/put_on_stack.rs::activate_ability` rejects any activation by a player who does not control the permanent. That is CR 602.1a's *default* — "the controller of an activated ability is the player who activated it", and only that permanent's controller may do so — but the rule is overridable by the ability's own text, and **41 printed cards override it**: Aether Storm ("Pay 4 life: Destroy this enchantment... Any player may activate this ability"), Excavation, Feral Hydra, Deadly Designs, Fan Favorite, Endbringer's Revel, Casey Jones, and 34 more (Scryfall `o:"any player may activate"`, 2026-08-23).

   `AbilityDef` has nowhere to record the permission, so this is a missing field rather than a missing check: an `activatable_by` on `AbilityDef` (default: controller only), read by `put_on_stack.rs::activate_ability` and by `oracle::mana_helpers::activatable_abilities`, which currently enumerates only the asking player's permanents. Both halves are needed — a permission the action list never offers is invisible.

   Surfaced during the Layer 2 phase, whose migration rewrote the check but not its scope. The error message now names CR 602.1a and says the exception is unmodeled, rather than asserting the rule is universal.

   **Reachability (2026-09-03):** unreachable — none of the 41 cards is
   registered.

   **Sized:** `activatable_by` on `AbilityDef` plus the two read
   sites, ~60–80 lines, with the first such card.

3. **Named counters have no representation — `CounterType` is a closed enum.** CR 122.1 lets a counter be named anything, and "counters with the same name or description are interchangeable" makes the *name* the identity. Most named counters have no rules meaning at all: the card counts its own counters and nothing in the engine cares what they are called.

   **Breadth, measured 2026-08-23:** a ~1000-card Scryfall sample of `o:/counters? on/` yields **115 distinct counter-name words** — charge, time, oil, quest, age, storage, lore, doom, plan, flood, bounty, egg, energy, scream, page, delay, gold, fuse, mire, ice, verse, luck, ki, collection, spore, slumber, book, burden, filibuster, and on. One sample, not the whole set. A variant per name is not viable.

   **The split is the same one `KeywordFlag` uses.** CR 122 enumerates every counter the rules branch on, and it is a short closed list: 122.1a +X/+Y, 122.1b keyword, 122.1c shield, 122.1d stun, 122.1e loyalty, 122.1f poison, 122.1g defense, 122.1h finality, 122.1i rad, plus 122.3's +1/+1 ÷ -1/-1 annihilation. Those stay variants because engine code is keyed on them. Everything else is a name and a count.

   **`CounterType::Charge` is already on the wrong side of that line** — no CR entry, the single most common vanilla counter in Magic, a variant only because one card needed it. It moves with this work.

   **Shape:** `CounterType::Named(...)` carrying a `&'static str`. The constraint is that `CounterType` is `Copy` and a `HashMap` key with five by-value signatures, so `String` and `Arc<str>` are both out — they would ripple through all of them. `&'static str` is correct permanently: every card is a `CardDataBuilder` call compiled into the binary, there is no serde, no file I/O and no deserialization anywhere in `src/`, and Scryfall is a research tool for contributors, never a runtime dependency.

   The one open question is whether to wrap it in a `CounterName` newtype with `const` values per name. The argument for it is typo discipline, not future-proofing: with 100+ hand-authored names spread across `src/cards/*.rs`, `"charge"` misspelled once silently creates a second, unrelated counter kind that no test would catch. Decide when the first named counter lands.

   Nothing is lost to a catchall: CR 122.4 ("can't have more than N counters of a certain kind") and CR 122.7 ("when the Nth [kind] counter is put on") are both generic over *kind* and never need to know what a counter means.

   **Build it with the first card that needs a named counter, not before** — there is no consumer today, `CounterType::keyword_granted()` already returns `None` for anything unrecognized, and a representation with nothing to test against is what item 9 warns about.

   **Reachability (2026-09-03):** unreachable — every counter in `src/cards` is
   `+1/+1` or `-1/-1`.

   **Sized:** `CounterType::Named(&'static str)` and moving
   `Charge` across the line, ~80 lines, with the first card that needs a named
   counter.

4. **CR 613.7e re-timestamping on attachment is unimplemented — and it collides with the determinism doctrine (recorded 2026-08-24).** "An Aura, Equipment, or Fortification receives a new timestamp each time it becomes attached to an object or player." Nothing in the tree ever reassigns `PermanentState.timestamp`, and both CLAUDE.md and `battlefield_ordered`'s docs now state "allocated once per `place_on_battlefield`, never reassigned" as the *determinism* guarantee. `layers-architecture.md` §8 point 3 lists 613.7e as designed, so that doc currently claims more than the code does.

   Unreachable today — Equip is unimplemented and Auras attach only at ETB — but the day any reattachment path lands, every equip silently re-orders Layers 6 and 7. **Corrected and scheduled 2026-09-01 -- "unreachable today" was the wrong frame, and so was "restate the contract".** Reattachment is not exotic: Aura Finesse (`{U}` Instant, "Attach target Aura you control to target creature") and Equip both do it with no new subsystem behind them. And the field is doing **two jobs** -- `battlefield_ordered` and `battlefield_ids_ordered` read it as *determinism / decision order*, `static_effect_timestamp` reads it as CR 613.7a. Four production readers, counted. Reassigning it makes a reattached Aura jump to the end of every ordered sweep, so the work is a **field split** -- a stable entry timestamp and a CR 613.7 timestamp -- not a reassignment plus a doc edit. Bounded by an in-tree precedent: CR 613.7c already reassigns `CounterStack.timestamp` from the same monotonic counter (`state/battlefield.rs:144`). Now **Phase LH-2**, `layers-architecture.md` §13a, scheduled before critical-path item 7. Original entry: **Do these together:** reassign from the same monotonic counter (still deterministic — that is the point), restate the contract as "never reassigned *except by CR 613.7e*" in CLAUDE.md and in `battlefield_ordered`, and re-audit every site that reads `timestamp` as a proxy for ETB order. Caught by audit before it had a reproducer; the two before it were found by their reproducers.

   **CR 613.7m is the same rule family and is also unimplemented (recorded 2026-09-01, RC-2 review).** "If two or more objects would receive a timestamp simultaneously, such as by entering a zone simultaneously or becoming attached simultaneously, their relative timestamps are determined in APNAP order." `allocate_timestamp` hands out a strict sequence one call at a time, so the engine can only ever produce a total order in *allocation* order — which is the caller's loop order, not APNAP. It is exact today because every entry is its own singleton batch: `propose_entry` is called once per zone change, and nothing puts two permanents onto the battlefield as one event. **The one thing that changes that is `GameAction::CreateTokens` (Phase RE)**, plus the mass-return primitive item 46 now sizes. **Corrected 2026-09-03, by RC-5's re-size:** this row also named "CR 614.13's auxiliary zone changes (RC-4)", and they are not one of them. An object receives a timestamp in one production place — `place_on_battlefield` (`game_state.rs:671`; `:789` is CR 613.7c's per-counter-kind stack, not an object) — and 614.13's auxiliary moves are battlefield → graveyard and graveyard → exile. Neither destination allocates a timestamp and neither is an entry, so RC-5 produces no simultaneous entries and 613.7m stays with `CreateTokens` in RE, beside item 52's token residual. Note the shape of the fix is *not* "sort the batch": 613.7m gives the active player's objects the earlier timestamps **in the order of that player's choice**, so it is a decision point, not a sort. Related and separate: CR 613.7n's rule that an object's own static ability out-timestamps a resolving spell's effect on it is also unimplemented, and is the same batch of work.

   **Reachability (2026-09-03):** unreachable — no reattachment path exists
   (Equip is unimplemented, Aura Finesse is unregistered, no Aura is castable)
   and nothing enters simultaneously (613.7m; `CreateToken` still loops).

   **Re-dated 2026-09-13 (RE-4) — 613.7m's half now has simultaneous entries
   and is still not asked, on RC-4's rule.** `GameAction::CreateTokens`
   proposes every token's entry as one batch, so objects *do* receive
   timestamps simultaneously now, in every Raise the Alarm the pool casts.
   But the tokens one effect creates are identical and enter under one
   controller, so "in the order of that player's choice" is a choice with one
   outcome, and "never prompt for a choice with one outcome" applies
   (`replacement-architecture.md` §9, RE decision 3): the batch order is the
   creation's, and the log, the timestamps and `battlefield_ids_ordered`
   agree with it (`a_plural_creation_is_one_event_and_its_entries_join_it`).
   The prompt's customer is the first creation with *distinguishable*
   members — Academy Manufactor's "one of each", Bestial Menace — or the
   mass return item 46 used to size, and none is in reach. 613.7e's half is
   unchanged.

   **Sized:** 613.7e is LH-2, ~900 additions
   (`layers-architecture.md` §13a); 613.7m is a decision point (the active
   player orders their own), ~60 lines inside `CreateTokens`' performer, with
   the first distinguishable creation; 613.7n rides LH-2.

5. **Layer 7c is not order-independent in Magic, and 19 cards say so (recorded 2026-08-24).** `compute.rs` applies ±1/±1 counters after the 7c registry slice without a timestamp merge. That is correct while every 7c modification is an addition — but CR 701.10a makes "double [a creature's] power" a 7c continuous effect whose addend depends on what already applied, so two doublings, or a doubling and a pump, are order-dependent by timestamp. Scryfall: **19 cards** match the doubling shape (Bulk Up, Epic Fight, Exponential Growth, Unnatural Growth…), before looser wordings. Inexpressible today — `AmountExpr` has no affected-power leaf — so nothing is wrong now. The first doubling card needs that leaf **and** the timestamp merge Layer 6's keyword counters already use. The comment at the code site was corrected 2026-08-24; it used to claim order-independence as a property of the layer.

   **Reachability (2026-09-03):** unreachable — `AmountExpr` has no leaf reading
   the affected creature's power and none of the 19 doubling cards is
   registered.

   **Sized:** the leaf plus a 7c timestamp merge in the shape Layer
   6's keyword counters use, ~100–150 lines, with the first doubling card.

6. **Multi-attacker block damage is a silent stub.** `engine/combat/resolution.rs`'s multi-block arm: a blocker blocking 2+ attackers assigns *all* its damage to the first living attacker — no `DecisionProvider` choice, no error, CR 510.1d ignored (the comment there has pointed here since 2026-09-15). Unreachable (nothing in the pool grants "can block an additional creature"), and the plumbing to fix it already exists: `GameState.blocker_damage_divisions` is populated from `choose_blocker_damage_division`. Reachable with the first "blocks an additional creature" card. This is the silent-wrong-*choice* cousin of the silent-inertness class the loud-lowering work covered.

   **Reachability (2026-09-03):** unreachable — nothing populates
   `blocking_limits` (`validation.rs:135`; `max_blocks_for` always answers 1).

   **Sized:** read `blocker_damage_divisions` in
   that arm instead of the first living attacker, ~40–60 lines, with
   the first "can block an additional creature" card.

7. **Hexproof and shroud are unenforced in spell targeting.** `engine/targeting.rs`'s `Target` validation (its comments have pointed here rather than at the archived `T22` since 2026-09-15). `KeywordFlag::Hexproof` exists and combat honors it; spell targeting does not check either keyword. No registered card carries hexproof or shroud, so no game can reach it — reachable with the first such card, which is a Phase 8 event.

   **Reachability (2026-09-03):** unreachable — no registered card has hexproof
   or shroud.

   **Sized:** RS-2's Tier 1a/1d, ~40 lines in that validation
   plus `enumerate_legal_selections` — the same change as main item 15's first
   half.

8. **~~A token created in exile instead logs `from: Battlefield` — RC-4b's cheap token answer, item 52.~~ ✅ CLOSED 2026-09-13 (RE-4).** The line Dour Port-Mage and Aang would have read no longer exists: a token exiled instead is `TokenCreated { Exile }` with no zone change, asserted absent in `phase_re4_integration_test`. → `plans/archive/codebase-state-closed.md`, main item 52.

   **Reachability (2026-09-13):** closed — RE-4.

9. **`can_pay_costs` checks each `Cost::Mana` against the whole pool, and
   `pay_costs` is not atomic across them (found 2026-09-02, closing 16c).**
   `assemble_total_cost` (folded into `determine_total_cost` on review) appended an additional cost's mana as its *own*
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

   **Reachability (2026-09-03):** unreachable — still no registered card with an
   additional mana cost (`additional_cost` appears in no card file).

10. **Card files have no shared helper module, so every phase re-writes the
    same `AbilityDef` literal — and the only alternative on offer is
    `test_support` (raised in the RD-1 review, 2026-09-08).** The static-ability
    shape (`id: new_ability_id(), ability_type: Static, costs: vec![], effect,
    is_characteristic_defining: false, activation_restriction: None`) is written
    out **31** times across `src/cards/`, plus two private named helpers that
    wrap it — `phase_li_cards::static_ability` and
    `phase_rd_cards::static_replacement`, which cannot see each other. A card
    file must not depend on `test_support`, so the pull today is toward a third
    private copy rather than toward sharing.

    **Reachability (2026-09-08):** reachable and not wrong — it is duplication,
    not a defect, and nothing it produces is incorrect. What makes it a
    *deadline* rather than a nit is Phase 8: card breadth multiplies the
    per-file copies, and the moment a real card list arrives, the helpers have
    to be somewhere that is not a phase file and not the test crate, or they
    get cordoned off with the fixtures.

    **Sized:** ~150–250 lines, mechanical — a `cards::helpers` module beside
    the registry holding the ability constructors (static, activated, triggered
    when it exists) and the recipient shorthands, with the 31 sites rewritten
    to call them. It must not become a second `CardDataBuilder`: the builder
    owns the *card*, this owns the *ability*, and the line between them is that
    a helper here returns an `AbilityDef` and nothing else. Best done as its own
    mechanical PR before the first Phase 8 card file, not folded into one.

    **The module exists and is named (2026-09-20, the TR-1 review, theme
    B).** `src/cards/authoring/` — a directory rather than the single file
    this entry imagined, because the vocabulary splits by subsystem, and
    `authoring/triggers.rs` is the first tenant with CR 603's words. **The
    31 static-ability sites and the two private wrappers are untouched**:
    theme B was the trigger vocabulary, sized at ~120 lines. What is closed
    is the question this entry framed as open — there is now a place that
    is neither a phase file nor `test_support`, so the rest is a move into
    it and no longer a design.

    **What the transition to a real card list looks like — asked on review
    2026-09-08 and then measured, because both of us were arguing from
    impressions.** The measurement changed one of the answers.

    **Measured (Scryfall `/cards/collection`, 2026-09-08).** 98 registered
    names, **97 of them real** — the single exception is `Loyalty Probe`, which
    RD-1 added the day before. `Everywhere` looked like a second exception and
    is not: it is a real printed *token* (`tdsk`), which is exactly what
    `registry.rs` says it is. Separately, **29 card functions are defined and
    registered nowhere** — the fixtures: `generic_reducer`, `flight_clause`,
    `dual_land_ub`, the four `*_spell` stand-ins, and so on.

    **So the criterion is not quietly rotten; it has exactly one exception and
    it is deliberate.** But the framing was wrong, and this is the finding: the
    registry is **not** "the official card list". It is *the set of things a
    deck can be built from* — `fuzz_games` and `cli_play` can only play what is
    registered. Loyalty Probe is in it because CR 704.5i needed a planeswalker
    a random agent could reach, not because anyone thought it was a card. Three
    sets, not two:

    | | what it is | count today |
    |---|---|---|
    | **printings** | faithful to a real card or token | 97 |
    | **fixtures** | invented, exist to make an engine path testable | 30 |
    | **the playable pool** (`registry.rs`) | what a deck can contain — drawn from *both* | 98 |

    The third is a membership question ("can the engine play this, and does the
    harness need it"), which is why it will always be able to contain a fixture.

    **File by first printing — and the two paragraphs that stood here arguing
    for alphabetical shards were wrong, twice over (corrected on review,
    2026-09-08).**

    The wrong argument was: the pool measures 62 sets for 97 cards, most
    holding one, so set files would average 1.5 cards each and that is worse
    than what exists. **That optimizes a filing decision against a snapshot,
    which is exactly backwards.** A filing scheme is amortized over the whole
    life of the corpus and the cost that decides it is the **marginal cost of
    adding the next card**, not the tidiness of the first hundred:

    | | a new set arrives | two people add cards |
    |---|---|---|
    | **by first printing** | one new file, one `pub mod`, one block of `register` calls; **no existing file is touched** | different sets, different files — no conflict |
    | **alphabetical shards** | every shard is edited; shards grow unboundedly and eventually need a rebalance, which renames files and breaks every import | same shard, constant conflicts |

    **And the 62-sets figure was measured on the wrong key, which overstated
    it.** Scryfall's collection endpoint answers a name lookup with a printing
    of its own choosing, not the earliest; keyed on **first** printing the same
    97 cards give **46 sets, 33 of them holding one — and `lea` alone holds
    31**. What is left of the fragmentation is an artifact of how this pool was
    assembled: one card at a time, for engine reasons, across thirty years of
    Magic. It is close to the most set-fragmented sample the card base could
    produce, and a real import goes set by set into files holding hundreds.

    **The second error was inventing the constraint it optimized against.**
    "Phase 8 takes it to a Commander-viable few hundred" appears in no document
    — `roadmap-v2.md` says the opposite about the trajectory: after Phase 8
    "the bottleneck moves to card-authoring speed, which is when the deferred
    Scryfall import pipeline earns its slot". That pipeline emits **per set**,
    because that is how Scryfall's bulk data is shaped, so filing by set is
    also the scheme under which generated and hand-written cards land in the
    same place. Picking a layout that a few hundred cards would suit, and
    locking in against an import measured in tens of thousands, is the tail
    wagging the dog.

    **Findability was the other thing offered for alphabetical, and it is not
    filing's job.** A caller reaches a card by name through the registry
    (`registry.create("Furnace of Rath")`) or by function through the compiler;
    neither reads a directory listing. Filing has to serve *change*, and change
    arrives set-shaped.

    **First printing is the key, and it is the one that never moves.**
    `classify_cards.py --first-printings` resolves it — one request per card,
    because no bulk endpoint answers it. The existing 97 get filed by it on day
    one, small files and all; by type is already
    visibly failing (`creatures.rs`, `keyword_creatures.rs`,
    `utility_creatures.rs`) and one file per card stays available if a set file
    ever becomes unwieldy.

    **Fixtures move to `src/cards/fixtures/`, not to `tests/`** — and the
    reason is structural rather than aesthetic. `tests/` is a separate crate
    that `src/cards/registry.rs` cannot reference, and `test_support` is behind
    a feature flag that release builds turn off; a registered fixture has to be
    reachable from `src` either way. So it is a sibling module whose doc says
    "nothing here is a printing", and the split is visible at every import.

    **The plan, then:**

    1. Classify — done, above, and re-runnable: the registered names against
       Scryfall's collection endpoint, and defined-vs-registered off the tree.
    2. Move the 97 printings into per-set files keyed on **first** printing.
    3. Move the 30 fixtures into `cards::fixtures`. **The registered one stays
       registered** — its reason is the harness, and moving a file does not
       change it. **Tests need one changed `use` line each**: they call the
       function (`phase_li_cards::flight_clause`), never a registry string, so
       a rename stays a compile error.
    4. Delete the empty `phase_*_cards.rs` files. Their names were always
       archaeology — which engine phase first needed a card, not anything about
       the card.

    **Sized: ~1 PR, mechanical, and it must be its own** — nearly every line is
    a move, so a diff that also changes behavior would be unreviewable.
    Scheduled at Phase 8's gate (`roadmap-v2.md` §C), with the helper hoist
    above as the same PR's other half: both are "put the card layer in order
    before it triples", and neither is worth doing twice.

### Before card breadth (Phase 8) — added by the RD-2 review (2026-09-09)

11. **~~The codebase has enough invented vocabulary to need a glossary, and
    nothing defines the words in one place~~ — ✅ CLOSED 2026-09-11 (the
    glossary pass).** — archived.
    **Reachability (2026-09-11):** closed — `plans/glossary.md` and
    `plans/check_glossary.py`, PR #123.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before card breadth
    (Phase 8) — added by the RD-2 review" item 11.

### Before Triggered abilities (CR 603)

The dispatcher landed with TR-1 (2026-09-19): detection in `engine::triggers::dispatch`, placement in `place_pending_triggers` behind `perform_sba_and_triggers`. What is left here is what the later TR phases carry:

1. **~~Trigger dispatcher stub.~~ — ✅ CLOSED 2026-09-19 (TR-1).** — archived.
   The stub is `place_pending_triggers`, drained inside `perform_sba_and_triggers` in APNAP order over the seat list with CR 603.3b's two tiers; detection is `engine::triggers::dispatch`, at the close of the outermost batch and at an unbatched emission.
   **Reachability (2026-09-19):** closed — TR-1.
   Full entry: `plans/archive/codebase-state-closed.md`, "Before Triggered
   abilities (CR 603)" item 1.

2. **~~Event shape audit.~~ — ✅ CLOSED 2026-09-18 (A6 step 1, the trigger
   survey).** — archived. Run as `plans/references/trigger-survey.md`, with
   `plans/references/trigger-survey.py` regenerating every count: each event
   CR 603.1b–603.12a names against the corpus and the printed population, and
   the printed distribution against the performed event that would carry it.
   The three bullets held — per-permanent granularity with the batch as the
   one-or-more boundary, post-action timing with CR 603.2g's prevented events
   never reaching the stream, and context — except where a performer drops a
   field the proposal had, which is item 10's shape; the nine gaps are items
   10–18 below, and sixteen corner cases are kept as the doc's questions.
   **Reachability (2026-09-18):** closed — A6 step 1, the survey.
   Full entry: `plans/archive/codebase-state-closed.md`, "Before Triggered
   abilities (CR 603)" item 2.

4. **~~The entry hop: Containment Priest's substitute leaves a permanent's
    worth of zone changes in the log for a card the CR says never entered~~ — ✅
    CLOSED 2026-09-02 (RC-4b).** — archived.
    **Reachability (2026-09-03):** closed — RC-4b, PR #87 (6541d0b).
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Triggered
    abilities (CR 603)" item 4.

5. **~~Tier 2 of the trace plan — a `TraceSink` on `GameState`, owed before the
   dispatcher~~ — ✅ CLOSED 2026-09-18 (A4c, PR #170).** — archived. Built as
   `mtgsim/src/state/trace.rs`: five emit points behind one branch each, a
   handle whose `Clone` is the fork marker, JSON lines with no `serde`, and
   `plans/trace_spine.py` rendering a spine and never a page's argument. The
   dispatcher's own emit point is item 9 below.
   **Reachability (2026-09-18):** closed — A4c, PR #170.
   Full entry: `plans/archive/codebase-state-closed.md`, "Before Triggered
   abilities (CR 603)" item 5.

3. **~~LKI formalization.~~ — ✅ CLOSED 2026-09-19 (TR-1).** — archived.
   The reader is `engine::triggers::binding` — `bound_object`, `bound_player` and `bound_amount` over the record through the matched arm's projections, CR 603.6's "unable to be found" and CR 400.7 as one epoch comparison. The frame's *status* half is TR-4's `LastKnownInformation`; the two `sba.rs` probes were existence checks, not frames, and stay.
   **Reachability (2026-09-19):** closed — TR-1.
   Full entry: `plans/archive/codebase-state-closed.md`, "Before Triggered
   abilities (CR 603)" item 3.

6. **CR 603.6c's *phased-in* qualifier has no implementation, and the matcher
   will need it.** The rule names CR 800.4a's departure in as many words —
   "leaves-the-battlefield abilities trigger when a permanent moves from the
   battlefield to another zone, **or when a phased-in permanent leaves the game
   because its owner leaves the game**" — so a `GameEvent::LeftTheGame` from
   the battlefield fires them, and RE-7 puts the CR 603.10a frame on the event
   for the matcher to read. What it cannot express is the qualifier: a permanent
   that is **phased out** does not trigger, and phasing (CR 702.26) is not built
   (this file, "Phasing (CR 702.26) — sized 2026-08-26, not started"). Every
   permanent is phased in today, so the frame is unconditional and correct;
   the day phasing lands it needs a condition, and the day the matcher lands it
   needs to know that.

   **Reachability (2026-09-13):** unreachable twice over — no trigger matcher
   reads the event, and nothing can phase a permanent out. It stops being
   unreachable when *either* lands, which is why the line names both.

   **Sized:** one predicate at the frame's `if` in `owned_objects_leave`,
   ~5 lines, inside whichever of the two arrives second.

   **Phase (2026-09-18):** not this phase's — the matcher (TR-1) reads `LeftTheGame` unconditionally and the qualifier lands with phasing, which arrives second; `triggers-architecture.md` §3.3's `LeftTheGame` row and §16.

8. **"Whenever you create one or more tokens" has one key, and CR 111.13 is
   where it stops (recorded 2026-09-13, RE-4).** `GameEvent::TokenCreated {
   object_id, owner, zone }` is announced for every token an effect creates —
   ahead of `PermanentEnteredBattlefield` for one that enters, alone for one
   created elsewhere (Hallowed Moonlight's exile) — so a "create" trigger
   reads one event kind whatever the zone, and a plural creation is one batch
   of them, the way "one or more creatures die" is a batch of zone changes.
   The line it must not cross is CR 111.13: a copy of a permanent spell
   becomes a token as it resolves and "is not 'created' for the purposes of
   any … triggered abilities that refer to creating a token" — it enters from
   the stack with a `from`, takes the entry performer's card arm, and gets no
   `TokenCreated`. CV-4 is where that token first exists; the arm is already
   the right one.

   **Reachability (2026-09-13):** nothing owed to correctness — a note for
   item 6's event audit (item 2), so the dispatcher keys "create" on this
   event and not on `is_token` at entry.

   **Sized:** none; the event exists and is emitted.

7. **~~CR 800.4d's second sentence has no site until the dispatcher exists.~~ — ✅ CLOSED 2026-09-19 (TR-1).** — archived.
   One `in_game` read at the head of `place_pending_triggers`, with a `pending` trace record per refusal. The four-player fixture is a Blood Artist whose controller loses in the state-based check that kills another creature — its frame sees the death, the trigger queues under the departed player, placement refuses it (`a_trigger_a_departed_player_would_control_is_not_put_on_the_stack`); ATOM-800.4d-001 is `COVERS` there. Astral Slide's delayed shape is TR-3's.
   **Reachability (2026-09-19):** closed — TR-1.
   Full entry: `plans/archive/codebase-state-closed.md`, "Before Triggered
   abilities (CR 603)" item 7.

9. **~~The dispatcher is the trace sink's sixth emit point, and it does not
   exist yet (A4c, 2026-09-18).~~ — ✅ CLOSED 2026-09-19 (TR-1).** — archived.
   `trace_records::trigger` per matcher decision — the record, the identity with its instance, the zone, matched or the predicate that refused it, and whether it resolved as a mana ability — and `trace_records::pending` per placement or refusal, with the targets; `plans/traces/viewer.html` renders both kinds.
   **Reachability (2026-09-19):** closed — TR-1.
   Full entry: `plans/archive/codebase-state-closed.md`, "Before Triggered
   abilities (CR 603)" item 9.

10. **~~Three performers drop a proposal field the record needs (the trigger
    survey, 2026-09-18).~~ — ✅ CLOSED 2026-09-19 (TR-1).** — archived.
   `StepBegin.player`, `PhaseBegin.player`, `DamageDealt.is_combat` and `LifeChanged.cause: Option<LifeLossCause>` (`None` for a gain) off the three performers, every literal site patched, and the engine arm read `IDENTICAL` on every counter as the item predicted (`fuzz-record.md`, TR-1).
   **Reachability (2026-09-19):** closed — TR-1.
   Full entry: `plans/archive/codebase-state-closed.md`, "Before Triggered
   abilities (CR 603)" item 10.

11. **`AttackersDeclared` carries no defender (the trigger survey,
    2026-09-18).** CR 508.3a's "attacks [a player, planeswalker, or battle]",
    508.3b's "is attacked" and 508.3e's "attacks another player" read whom
    each creature was declared against; the record is the attacker list, and
    the defender sits on `AttackingInfo.target` — live at dispatch, and absent
    from the trace A4c built to answer "why did this fire". 1,695 cards carry
    an attack trigger.

    **Reachability (2026-09-18):** unreachable — no reader.

    **Sized:** `Vec<(ObjectId, AttackTarget)>` at the one emit site in
    `engine/combat/steps.rs` and its `format_event` arm, ~10 lines.

    **Phase (2026-09-18):** TR-5 — the defender on the record, and `BlockersDeclared` becomes one record per declaration step; `triggers-architecture.md` §3.12.

12. **No event announces a target being chosen (the trigger survey,
    2026-09-18).** CR 601.2c chooses targets, CR 601.2i says the abilities
    that trigger on the cast trigger then, and nothing between them emits:
    "becomes the target" is 117 cards and ward (CR 702.21a, "whenever this
    permanent becomes the target of a spell or ability an opponent controls")
    is 195 more — the largest printed population with no record at all. The
    stack entry knows (`chosen_targets`, A4i); the stream does not.

    **Reachability (2026-09-18):** unreachable — no reader.

    **Sized:** a variant carrying the targeting object, its controller and
    the target, emitted where `cast_spell` and `activate_ability` announce
    themselves and where CR 603.3d puts a trigger on the stack, ~30 lines.
    Whether one spell naming one permanent twice is one event or two
    (CR 115.9a counts instances) is the survey's question 11, the doc's.

    **Phase (2026-09-18):** TR-5 — `GameEvent::Targeted`, one record per (spell or ability, target) at three emit sites; question 11 decided as once per spell (Frost Titan's ruling); `triggers-architecture.md` §3.12, §14.

13. **No event announces a control change (the trigger survey,
    2026-09-18).** CR 603.10d's look-back triggers (126 cards) watch an event
    the engine performs as a Layer 2 registry row: `Primitive::GainControl`
    writes it and `get_effective_controller` answers differently from then
    on, and the row expiring at cleanup changes control back with no proposal
    anywhere. Not a field: control is a computed value, and "gains control"
    is a "becomes" on the layer walk's output, the shape CR 603.2e gives
    tapping.

    **Reachability (2026-09-18):** unreachable — no reader; the change itself
    happens in measured games, since Act of Treason is registered (LG).

    **Sized:** unknown until the doc says whether a change in a computed
    value is detected at the registry write and its expiry, by the walk, or
    as a state trigger's cousin; the doc's.

    **Phase (2026-09-18):** TR-4 — decided: a sweep at the state check, gated on `any_control_changing`, against a materialized `PermanentState.announced_controller`, emitting `ControlChanged`; `triggers-architecture.md` §3.12, §4.5.

14. **The LKI frame carries characteristics and no status (the trigger
    survey, 2026-09-18).** `EffectiveCharacteristics` is the CR 603.10a frame
    on `ZoneChange` and `LeftTheGame`, and it has no counters, no attachment
    link and no tapped bit. Persist (CR 702.79a) and undying (702.93a) are
    dies-triggers with an intervening-if on the counters the permanent
    *had*; an Aura's CR 603.6e trigger reads what it enchanted; the four
    "becomes unattached" Equipment (603.10c) read the host they left — 109
    cards on the survey's query.

    **Reachability (2026-09-18):** unreachable — no reader.

    **Sized:** three fields copied from `PermanentState` at the capture in
    `perform_zone_change` and in `owned_objects_leave`, ~15 lines; whether a
    frame typed as *characteristics* should carry status is the doc's, and
    CR 603.10's word is "appearance".

    **Phase (2026-09-18):** TR-4 — the frame becomes `LastKnownInformation { characteristics, status }`, CR 113.7a's phrase and the tree's `lki`; `triggers-architecture.md` §3.11.

15. **The frame is captured only for a battlefield departure (the trigger
    survey, 2026-09-18).** CR 603.10a names three look-back classes and the
    engine captures one: a card leaving a graveyard (38 cards) and a visible
    object put into a hand or library (8) get `lki: None`. A continuous effect's
    filter can reach a graveyard (`ZoneSet`, `layers-architecture.md` §13c;
    Yixlid Jailer is registered since LJ), so under it a "when this card
    leaves your graveyard" ability must not trigger, and only the frame from
    before the move can say.

    **Reachability (2026-09-18):** unreachable — no reader.

    **Sized:** the capture condition widened from `from == Battlefield` to
    the three classes, ~10 lines, once the doc says which zones the frame is
    computed for.

    **Phase (2026-09-18):** TR-4 — the capture widens to CR 603.10a's three classes; `triggers-architecture.md` §3.11.

16. **No event for a prevention effect applying (the trigger survey,
    2026-09-18).** CR 615.13: "such an ability triggers each time a
    prevention effect is applied to one or more simultaneous damage events"
    — 15 cards, Selfless Squire the plain one. The pipeline knows: A4c's
    `pipeline` record is written at that CR 616.1 iteration. The stream does
    not.

    **Reachability (2026-09-18):** unreachable — no reader.

    **Sized:** an event emitted by RD-2's shield path, one per prevention
    applied, ~10 lines; its fields are the doc's.

    **Phase (2026-09-18):** TR-5 — `GameEvent::DamagePrevented`, announced by the prevention leg once per instance per subject group; Selfless Squire is the card; `triggers-architecture.md` §3.12.

17. **Counters a permanent enters with announce nothing, and the entry
    record carries no `mods` (the trigger survey, 2026-09-18).** CR 122.6:
    counters "being put on an object … refers to putting counters on that
    object while it's on the battlefield and also to an object that's given
    counters as it enters the battlefield" — so "whenever one or more +1/+1
    counters are put on a creature you control" (72 cards on the survey's
    query) triggers for a creature entering with them. Today the entry
    announces no `CountersChanged` — by design on the replacement side, where
    CR 614.16's doublers replace the `EnterMods` rather than an event — and
    `PermanentEnteredBattlefield` carries no `mods`. Live-derivable at the
    entry's dispatch, since every counter on it then is an entry counter; not
    from the record.

    **Reachability (2026-09-18):** unreachable — no reader.

    **Sized:** the counter rows on the entry event, or a `CountersChanged`
    per row announced after the entry inside its batch, ~10 lines; which is
    the survey's question 4 — CR 122.7's "the Nth counter" and the
    replacement side read the two differently.

    **Phase (2026-09-18):** TR-5 — question 4 decided: one `CountersChanged` per entry row, announced after the entry inside its batch and never proposed, with `by` on the record; `triggers-architecture.md` §3.12.

18. **~~Three `GameEvent` variants are never emitted (the trigger survey,
    2026-09-18).~~ — ✅ CLOSED 2026-09-19 (TR-1).** — archived.
   `PhaseEnd`, `StepEnd` and `TurnEnd` deleted with their `format_event` arms; `CountersAnnihilated` is TR-5's, once CR 704.5q proposes.
   **Reachability (2026-09-19):** closed — TR-1.
   Full entry: `plans/archive/codebase-state-closed.md`, "Before Triggered
   abilities (CR 603)" item 18.

### Before Commander (CR 903)

1. **Commander damage increment — ✅ done (2026-04-18).** — archived.
    **Reachability (2026-09-03):** closed — 2026-04-18.
    Full entry: `plans/archive/codebase-state-closed.md`, "Before Commander (CR
    903)" item 1.

2. **Commander setup hook.** Nothing yet flips `is_commander = true` at deck construction or game setup. Needs a `GameConfig::commander()` constructor + a designation step (probably a field on `Decklist` or an analogous role entry). No tests exercise this yet — the flag is only set via direct field mutation in unit tests.

   **Reachability (2026-09-03):** unreachable — nothing sets `is_commander`
   outside two unit tests (`actions.rs:1256`, `:1339`); CR 903.9a/b and 704.6d
   are live and have no commander to act on.

   **Sized:** `GameConfig::commander()`, a designation on
   `Decklist`, and the flag set in `Game::new`, ~80–120 lines with tests; the
   first PR of the Commander interleave.

3. **`GameConfig::commander()` constructor.** Only the *hand/library* half of command-zone redirection waits on Replacement (903.9b). The graveyard/exile half is CR 704.6d, a state-based action, and can land with this constructor — so a "partial Commander" game here is life=40 + commander damage + 903.9a working, with only 903.9b missing.

   **Reachability (2026-09-03):** unreachable (as item 2) — and the entry is
   stale: both halves of command-zone redirection landed with RB (CR 704.6d as a
   state-based action, CR 903.9b as a replacement; PR #62, 78d344c), so nothing
   here waits on Replacement any more. The constructor is the whole residual.

   **Sized:** with item 2.

4. **Multiplayer priority rotation** (CR 800) — blocking for 3+ player Commander; not blocking for 2-player Commander.

   **Reachability (2026-09-03):** unreachable — `fuzz_games` plays two players
   (`fuzz_games.rs:827`). Part of this already exists untested at N>2:
   `Game::new` takes N decklists, and the priority loop, turn rotation and
   `apnap_index` are all modulo `num_players` (`priority.rs:187`, `turns.rs:30`,
   `game_state.rs:531`). What is missing is CR 800.4 — a player leaving, their
   objects and effects, and the 800.4c revert main item 9 is about — and CR
   802's choice of defending player.

   **Sized:** ~300–500 lines (800.4a–k; a defending-player choice
   on `AttackTarget`), plus a `--players 4` fuzz mode (~50 lines) so it is
   reachable at all; Commander interleave. **RE-6 carries the fuzz mode and
   800.4j/k, and RE-7 carries 800.4a–e and 800.4m (2026-09-11, re-cut on
   review; `replacement-architecture.md` §9 RE decision 5); CR 802's defending
   player and 800.4f–h's choices stay here.**

   **✅ The fuzz mode and 800.4j/k landed 2026-09-12 (RE-6).** `fuzz_games
   --players N` — one random deck per seat off the one stream, a
   two-player run byte identical to before, two rows only a wider table
   can move, and a "wins by effect" outcome key — and the four-player table
   is recorded in `fuzz-record.md` as RE-7's baseline. The mode
   was ~90 harness lines rather than ~50, because "a flag" was the optimistic
   reading: the deck loop, the copies count and the outcome key all had two
   players in them.

   **✅ CR 800.4a–e, 800.4c and 800.4m landed 2026-09-13 (RE-7)**, in
   `engine/leaving.rs` and at the four refusal sites, with CR 800.1's seat
   count as their gate (`GameState::is_multiplayer`). Item 108 closed with
   them. **What is left of this item, and it is the whole residual:** CR 802's
   choice of defending player, and CR 800.4f–i — a cost a departed player would
   pay (800.4f), a choice they would make (800.4g/h) and the last known
   information about them (800.4i). Each is a *choice-delegation* facility
   rather than an object sweep, which is why RE-7 did not absorb them: 800.4g's
   "the controller of the object chooses another player" has no shape in
   `DecisionProvider` and no printed customer in the pool, and 800.4i wants the
   LKI system this engine only has for zone changes.

   **Reachability (2026-09-13):** CR 802 is reachable in every four-player
   game — the defending player is chosen by the harness rather than by the
   attacking player, which is a missing prompt and not a wrong answer.
   800.4f–i are unreachable: the pool prints no cost or choice a player who is
   not the object's controller makes, so a departed seat is never asked for
   one.

   **Sized:** CR 802's defending-player choice ~60 lines at
   `candidate_priority_actions`' attack-target list plus a `ChoiceKind`;
   800.4f–h ~120 with the delegation rule and a fixture each; 800.4i waits on
   the LKI facility and is not sized here.

### Cross-cutting — keep this section honest

**~~The existence check is CR 604.2, not CR 613.7a (2026-09-06, LI-1 review).~~ — ✅ swept 2026-09-06.** `static_ability_still_exists` and every comment and doc line around it had called the "does the source still have the ability" question "CR 613.7a" since 2026-08-21. 613.7a is the timestamp rule; the question is CR 604.2 — "these effects are active as long as the permanent with the ability remains on the battlefield and has the ability" — and 611.3b says the same.

**57 lines re-cited** — 30 in the crate, 27 in these docs. 55 now read CR 604.2. **Two read CR 611.3a instead**, and they were a different wrong claim wearing the same number: `tests/filter_controller_test.rs` and item 16's note justified re-filtering an `ObjectSet::Filter` on every walk as "correct per CR 613.7a", which is 611.3a — "a continuous effect generated by a static ability isn't 'locked in'; it applies at any given moment to whatever its text indicates".

**The ~40 sites that stayed are the timestamp ones, and that is the whole point of the sweep.** Clause 2's `max` (`static_effect_timestamp`), the third sentence's re-stamp (`retime_static_rows`), the 613.7a/613.7b origin split on `EffectOrigin`, Rune of Flight as 613.7a's worked example, and the corpus id `ATOM-613.7a-001` all still cite 613.7a because they are all about *when* an effect applies relative to another, not *whether* it exists. `static_ability_still_exists` keeps its name: the function is named for the question, not the rule.

**~~`BattlefieldEntity` is misnamed (2026-09-06, LI-1 review).~~ — ✅ renamed to `PermanentState` 2026-09-06.** It holds what a permanent has that its card does not — controller, the CR 613.7 timestamp, CR 110.5's status (tapped, flipped, face-down, phased out), counters, attachments, damage — and the old name read as a second kind of object beside `GameObject`. 93 sites in 31 files, plus the `make_entity` helper in the struct's own test module.

**Three things kept their names, and each for a reason a rename should not override.** `GameState.battlefield` is the map, not the value. `state/battlefield.rs` is the module, and it holds `CounterStack`, `AttackingInfo`, `BlockingInfo` and `AttackTarget` besides — naming a five-type module after one of its types trades one wrong name for another. And `board.rs`/`lookahead.rs`'s `entity` accessors and `Board.entities` field are left alone on purpose: **LI-2 rewrites `apply_layer` and should start from this tree**, so this PR touches `board.rs` for the rename and the citation and nothing else.

**The docs the rename did not follow, and why.** `plans/archive/*` is superseded by CLAUDE.md's authority table, and `design_doc.md` / `plans/roadmap.md` are listed there as historical — all three record what was true when written. `plans/atomic-tests/**` keeps the old name too: `sessions/` is the authored spec corpus, and the index markdowns beside it are generated, so a prose edit there is a corpus edit plus a `specdb build` rather than a rename.

**Zero behaviour**, and checked as one: `fuzz_ab.py` byte-identical on both pools, three-run determinism on both, `specdb owed` unchanged.

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

**Spec-database annotation backfill — owed before the next phase.** `specdb stats` reads 0% on five finished phases because only the Phase 5-Layers tests were ever annotated (69 `COVERS` lines in the whole suite); it is measuring the annotation boundary, not coverage. A 2026-08-24 sample of ten Phase 5-Pre / ALREADY-IMPL atoms found no case where a 0% phase concealed a gap the chapter map claims is done — every real gap in the sample was already an honest ❌/🟡 here. The backfill is mechanical, roughly a day, mostly `// COVERS:` lines over existing sba/mana/cast/zone tests, and `plans/atomic-tests/phase-5-pre-audit.md` already reconciled the shipped 5-Pre tickets against the code with file:line citations — consume it rather than redo it. Use `COVERS-PARTIAL` honestly; a false link is worse than a blank. Reason to spend the day: Phase 6 has 124 atoms and Phase 7 has 133, and a tool that reads 0% on finished work has no credibility left for the phases that need it. **Still owed on 2026-09-03:** `specdb stats` reads Phase 5-Pre 2.6% full, ALREADY-IMPL 2.0%, Phase 7 0.0%; unchanged in kind since this was written.

**`fuzz_games::random_deck` land population — ✅ partly fixed (2026-08-23); artifacts still missing.**

Land slots used to be filled entirely from a color→basic table, so no deck could contain a nonbasic land. Blood Moon was in the card pool and inert, and CR 305.7 — the land-type carve-out in `engine/layers/land_types.rs`, the most intricate code in Layer 4 — had **zero** random-play coverage.

Fixed by registering the ten original dual lands (`cards/dual_lands.rs`) and giving `random_deck` a `NONBASIC_LANDS_PER_DECK` constant, currently 5. A land qualifies if it produces **at least one** of the deck's colors — deliberately not a subset test, because under a subset rule every dual is off by one color for a two-color deck and a mono-color deck gets none at all. An unusable second color on a land costs nothing in a fuzz deck.

Still crude, and knowingly so: a flat constant over a static pool is not a mana-base model. Replace it with a real picker when card breadth (Phase 8) gives it something to choose between.

**Inverted 2026-09-03 (`pool/everywhere-land`).** Every land in the registry made one or two colors, which is why a deck rolled one or two colors and filtered its nonlands to them, and why `--require` had to seed those colors from the required card's — a forced `{1}{G}{U}` in a deck that rolled red was included and never cast. Real "add one mana of any color" is not expressible (`backlog.md` §2.19), so the land is **Everywhere** (`cards/dual_lands.rs::everywhere`), the five-type token: it fills every land slot not taken by one basic of each type (`BASIC_LANDS_PER_DECK`, the contrast CR 305.7 needs) or by the `NONBASIC_LANDS_PER_DECK` draws. With that mana base the color roll and the nonland filter had nothing left to do and are gone: 36 nonlands come from the whole pool. What it bought and what it cost are under "Found by the Everywhere pool change" below; the short form is that `--require` now measures a card against a random board (200 of 200 games with a non-G/U permanent, from 0), and prints copies per deck beside the count because the first comparison across the two mana bases was counting copies — 16d has the correction.

**~~Still open — no artifact exists anywhere in `CardRegistry`~~ — ✅ fixed 2026-08-24, together with the Layer 7d hole.** March of the Machines was registered and inert for the same reason Blood Moon had been: nothing in the crate outside the `phase_l*` fixtures was an artifact, so Layer 7b had zero random-play coverage. Layer 7d had none either — no registered card switched P/T, and the one fixture that does (`phase5_pre_cards::inside_out`) simplifies a hybrid cost the engine cannot express, so it cannot be registered without misrepresenting the card.

Two real cards, authored verbatim: **Sol Ring** (`cards/artifacts.rs`) and **Merfolk Thaumaturgist** (`cards/utility_creatures.rs`). Sol Ring is colorless, so `random_deck` puts it in every deck rather than only the ones sharing its colors, and its mana value of 1 means it animates into a 1/1 that survives its own SBA check rather than a 0/0 that dies. Measured over 60 games at seed 7: a Sol Ring reached the battlefield in 53, a March in 23, **both in the same game in 20**, and Layer 7d resolved **98 times** (temporary probe, reverted). Both were zero before.

The Thaumaturgist is also the registry's **first `AbilityType::Activated` ability** — every other registered ability is a spell, a mana ability or a static — so `put_on_stack.rs::activate_ability`'s stack path, its target selection and its rollback arms now get random-play exposure too.

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
- **`GameState::battlefield_ordered` / `battlefield_ids_ordered`** — every sweep whose order is observable now goes through them: `oracle/{legality,mana_helpers,board}.rs`, all of `engine/sba.rs` (including the legend-rule grouping, now a `BTreeMap`), `engine/combat/{steps,resolution}.rs`, `ui/display.rs`. Sorting by `ObjectId` would *not* have worked — ids are v4 UUIDs, so the key is itself random per run. The deterministic key is `PermanentState::timestamp`, which `place_on_battlefield` allocates once from a monotonic counter and never reassigns, and which is CR 613.7's order anyway. Order-irrelevant sweeps (untap-all, clear-all-damage) still iterate the map directly.
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

123. **~~`Primitive::SetLifeTotal` does not exist, and three registered cards'
     rulings wait on it.~~ — ✅ closed 2026-09-12 (RE-6).** One arm that
     proposes the difference as a `GainLife` or a `LoseLife` and never writes
     the total, so the three rulings are three tests:
     `rhox_faithmender_makes_becomes_ten_from_three_seventeen`,
     `setting_a_life_total_higher_under_a_cant_gain_does_nothing` (Skullcrack's
     fourth, and Alhammarret's Archive's first is Rhox's board on a second
     card), with `ATOM-119.5-001`/`-002` claimed beside them.
     `AmountExpr::StartingLifeTotal` came with it, because Exquisite Archangel
     says "your starting life total" and v1's is 40. *Original entry:* CR 119.5
     had no producer, and Rhox Faithmender's, Alhammarret's Archive's and
     Skullcrack's rulings all named it.

124. **Three types carry an object set called `affected` — ✅ CLOSED
    2026-09-14 (`refactor/object-set-rename`).** — archived.
    **Reachability (2026-09-14):** closed — `refactor/object-set-rename`,
    PR #136; both fields are `affected_objects` and the type is `ObjectSet`.
    Full entry: `plans/archive/codebase-state-closed.md`, "Cross-cutting —
    keep this section honest" item 124.

### Found by LK — CR 113.6 (2026-09-14)

133. **Quadrant ① keyword abilities are frame characteristics, so CR 113.6
    never sees them — a card in a graveyard still reports its printed
    flying.** `seed_frame` seeds `keyword_flags` off the card in every zone,
    and `engine::zone_function` takes an `AbilityDef`; a `KeywordFlag` is not
    one (`plans/glossary.md`, “quadrant”, and `types::keywords`' own doc). So the
    default arm of CR 113.6 — "abilities of all other objects usually function
    only while that object is on the battlefield" — applies to every static
    ability on a card and to none of its keywords.

    **Found by writing Wonder**, whose plan claimed the card would exercise
    both arms of the predicate at once: flying on the battlefield, the grant
    from the graveyard. Half of that was wrong, and the card's doc comment and
    `layers-architecture.md` §13d record the correction.

    **Reachability (2026-09-14):** unreachable — nothing reads a
    non-battlefield object's keyword flags for a rules decision. Combat,
    SBA and the damage path all read the battlefield; `has_keyword` is public
    and will answer for a graveyard card, but no engine caller asks it about
    one. It becomes wrong with the first rule that does, and the two named
    candidates both arrive later: CR 702.35's madness (functions in hand,
    `backlog.md` §2.3) and critical-path item 6's trigger conditions
    (CR 113.6k).

    **Sized:** the honest fix is not a zone gate on `seed_frame` — that would
    strip a CDA-granted keyword too. It is to give the frame's keyword set the
    same treatment the ability list has: seeded from the card, then filtered by
    CR 113.6 at the one place a *rules* reader asks. ~60–80 lines and a decision
    about where that place is, which is why it waits for a reader rather than
    being guessed at now.

134. **`cleanup_zone_state`'s battlefield branch removes a source's rows
    whatever their origin, and CR 611.2a says a resolution's effect does not
    care where its source went.** `remove_by_source` is origin-blind. CR 611.3b
    is what the call is for — a static ability applies only while its source is
    on the battlefield — and CR 611.2a gives a resolution's effect "the duration
    stated by the spell or ability", which is not the source's lifetime.

    **Found by LK nearly writing the same call on the other branch.** The
    obvious way to generalize that function for "a static ability functioning
    in a graveyard leaves the graveyard" is `remove_by_source` again, and that
    would have deleted every pump spell's effect as the spell hit the
    graveyard — silently, and in every game. LK wrote the narrow
    `remove_static_by_source` instead, with
    `test_a_resolutions_effect_survives_its_spell_reaching_the_graveyard` as
    the regression. **The battlefield branch is untouched and is not LK's to
    fix**; this is the record that it is safe by accident.

    **Reachability (2026-09-14):** unreachable, and the bound is exact rather
    than a survey. A row is only at risk if its origin is `Resolution` *and*
    its source is a battlefield permanent. A resolution's `source` is
    `ResolutionContext::source`, the resolving **stack object** — ephemeral for
    an activated ability (CR 608.2n deletes it) and graveyard-bound for an
    instant or sorcery, so neither is ever on the battlefield. A permanent
    spell keeps its `ObjectId`, but CR 608.3 gives it no spell ability to
    resolve, so it registers nothing. Every other row on a permanent is
    `EffectOrigin::StaticAbility` — printed, granted
    (`register_granted_static_effects`) or copied
    (`register_copied_static_effects`) — and those are exactly the rows
    CR 611.3b wants removed.

    **Does this reorder the route? No** — asked at the LK review, and the
    answer is that RE-9 and RE-10 cannot produce the shape. RE-9 is mana and
    RE-10 the turn cursor; neither registers a continuous effect at all, let
    alone one sourced at a permanent. The first phase that can is critical-path
    **item 6**, because a triggered ability's source *is* the permanent rather
    than the ephemeral stack object an activated ability resolves through — so
    "whenever this creature deals damage, target creature gets +2/+2 until end
    of turn" is a row this branch would delete if the creature died first, and
    CR 611.2a says it should not. Item 6 is several phases out and this is ~10
    lines, so it can also just be taken between phases; what it must not do is
    land *after* item 6 builds tests against the wrong answer.

    **Sized:** swap the call for `remove_static_by_source`, which already
    exists, and decide what CR 611.3b means for a *granted* static ability
    whose grantee leaves — ~10 lines and one question.

### Found by the LJ review (2026-09-14)

132. **A crate-wide `.clone()` audit, owed at the end of replacement effects.**
    LJ's review found a `player.graveyard.clone()` inside
    `engine::layers::condition`'s `CardInGraveyard` arm that was never needed —
    both borrows are immutable and it compiles without. It had been added
    defensively rather than because the compiler asked, and it sat on a genuinely
    hot path: CR 604.2's existence check runs per application, per layer, per
    pass. **One unnecessary allocation found by eye is evidence of a class**, and
    the owner's call at the review was to schedule the sweep rather than widen
    this PR.

    **Sized against the tree (2026-09-14):** 192 `.clone()` sites in `src/`.
    121 are in `engine/`, of which **35 are in `engine/layers/`** — the
    per-layer-per-object path, and the ones worth reading first — and 17 in
    `engine/replacement/`. 9 are in `src/cards/`, which is card construction and
    cold by definition. The audit is a read of the 35 first, then the 17, and it
    is a *reading* pass with a fuzz A/B behind it, not a mechanical sweep: an
    `Arc::clone` is a refcount bump and belongs where it is, a `Vec` clone in a
    predicate is the shape this found, and telling them apart is the work.

    **Trigger: the end of replacement effects**, where the owner wants a
    housekeeping pass anyway — so it lands with RE closed and before Phase 6's
    triggers build on the same paths. Not urgent: nothing here is *wrong*, which
    is why it is scheduled rather than fixed.

    **Reachability (2026-09-14):** reachable — not wrong; a performance question
    on paths a measured game runs thousands of times a turn.

### Found by RE-6 — the game's end (2026-09-12)

**Shipped:** `GameAction::{PlayerLoses, PlayerWins}` with their two
`EventPattern` arms and `GameActionTemplate::PlayerWins`; the four
state-based loss loops as CR 704.3 batch members, deduped per player by
`subject_of`; CR 704.5b's window closed at the check; `GameResult` on
`GameState`, written by the `PlayerWins` performer and by the batch's
settlement (CR 104.2a/104.4a per batch, never per member); CR 104.1 at the
chokepoint, the state-based check and the priority loop; CR 104.4b's draw as
a cap on the state-based loop; CR 800.4j at the priority rotation and the
three turn-based actions a departed active player has nobody to perform;
CR 800.4a at the target rule and the attack-target list;
`Primitive::{LoseGame, WinGame, SetLifeTotal}`, `Primitive::Exile` for the
effect's own source and for a targeted card wherever it is,
`AmountExpr::StartingLifeTotal`, `Condition::LibraryEmpty` and the CR 604.2
leg in both static sweeps through `settled_holds`; `fuzz_games --players N`.
Four cards — Laboratory Maniac (pooled), Exquisite Archangel, Stunning
Reversal, Platinum Angel. Items 6 (the loss half), 73, 112, 113 (the
priority half) and 123 close; 108 is re-dated and measured; 122 is re-owned;
"Before Commander" item 4's fuzz mode is built. `replacement-architecture.md`
§11 items 61–67. **One line here, not six**: the review of this PR's first cut
found four small wrong answers recorded as debt with fixes shorter than their
entries, and they were fixed instead (§11 item 67 and the rule at the foot of
this section); a fifth was CR 104.3f, which is a catch-all and not a
migration.

125. **Exquisite Archangel dying in the check that would lose you the game
     takes the graveyard outcome, and the rider structure cannot offer the
     ruling's choice.** The card's first ruling: *"its effect applies ... You
     choose whether Exquisite Archangel is moved to exile or to your
     graveyard."* The loss and the death are two members of one batch decided
     against one board, so the replacement applies; but the card's "instead
     exile this creature and your life total becomes ..." is *two* events
     about two subjects, and a rewrite produces one (§3.2d), so the Archangel
     is encoded as `Prevent` plus a rider — and riders resolve after the batch
     performs (CR 615.5, §4.1a). The death has happened by then, the card in
     the graveyard is a new object (CR 400.7), and the rider's `Exile` finds
     nothing. Lich's Mirror's fifteenth ruling is the same board with a
     shuffle.

     **Reachability (2026-09-12):** reachable in `stress` — the Archangel is
     registered, and one combat with it blocking a 5-power attacker while
     another attacker is lethal to you is the board. **Wrong in the narrow
     sense**: the engine takes one of the ruling's two outcomes and never asks;
     the other is unreachable by construction. Tested as it behaves
     (`exquisite_archangel_replaces_the_loss_while_dying_in_the_same_check`).

     **Sized: it is `backlog.md` §2.25's facility, not a patch.** A
     replacement whose result is more than one event — the exile and the life
     total, simultaneous with the rest of the batch — needs a rewrite to yield
     *members* inserted into the batch in phase 1, which is exactly the
     one-event-becoming-two that RD-5 gated closed for Harm's Way (≈30
     mechanical sites in `apply_replacements`' return shape and
     `execute_batch_inner`'s phase-2 write, plus whatever the split itself
     needs); on top of it, two members moving one object to two zones must
     turn the CR 704.7 same-subject *collapse* into a *prompt* (~40 lines and
     a `ChoiceKind`). §2.25 now records the Archangel as its second customer,
     which is what §8c's "two customers before a leaf" asks for before that
     facility is built. Until then the engine's answer is the graveyard and
     this line is the record that the choice is missing.

### Found by RE-4 — tokens (2026-09-13)

**Shipped:** `GameAction::CreateTokens { defs, controller }` and
`EventPattern::CreateTokens { kind }` — a `TokenKind` matched against each
def, since the tokens are not objects when the pattern is asked; `Rewrite::
Amount(Multiplier)` over the `Vec`, repeating in place the defs the kind
matched; `GameActionTemplate::CreateTokens { def, count, mode }`, the
kind-changing substitution over a creation (`Replace`: Divine Visitation's
"that many Angels instead", one in each matched def's place and keeping how
the effect said it enters; `Append`: Chatterfang's "those tokens plus that
many Squirrels", Xorn's "plus an additional Treasure"); a performer that
creates the objects and proposes every entry as one contained batch,
un-creating a dropped member (CR 111.5); `GameAction::CreateTokenIn {
object, zone }` as the substitute for a token's entry, with
`GameState::put_token_into` and `GameEvent::TokenCreated` (two callers of one
emitter — the entry performer's token arm and `CreateTokenIn`'s); `TokenDef`
with `abilities`, `supertypes`, `rules_text`, `enchant_filter`,
`enters_tapped`, an `Option` name (CR 111.4's default) and `Option` power and
toughness (CR 208.3), lowered by `TokenDef::card_data`; Parallel Lives, Raise
the Alarm (both pooled), Hordeling Outburst, Hallowed Moonlight, Divine
Visitation, Bard, King of Dale. Items 46 and 52 and "Before card breadth"
item 8 closed. **The review's rule** (`engineering-practices.md` §4): an arm
the PR's own type opens, with a printed customer and sized under about eighty
lines, ships in that PR — which is why the kind, the template and
`enters_tapped` are above rather than below this line. **Left absent, each
with its customer named:**

126. **A kind-changing substitution from a *draw* to a creation has no leg.**
     Hullbreacher — "if an opponent would draw a card except the first one
     they draw in each of their draw steps, instead you create a Treasure
     token" — is `Instead(CreateTokens { .. })` applied to a `DrawCard`,
     which `substitute` refuses today (its `CreateTokens` leg pairs the
     template with a creation only). One leg, ~15 lines, `template_amount`
     already knowing a draw has no "that many"; the customer also needs a
     Treasure def, which is `backlog.md` §2.19's.

     **Reachability (2026-09-13):** unreachable — no registered def pairs
     the template with a draw.

     **Sized:** ~15 lines in `substitute`, with Hullbreacher, after §2.19.

127. **A creation template with a *choice* of def has no shape.** Jinnie Fay,
     Jetmir's Second — "you may instead create that many 2/2 green Cat
     creature tokens with haste or that many 3/1 green Dog creature tokens
     with vigilance" — is optional (the pipeline has that) and offers two
     defs, chosen as the replacement applies. `GameActionTemplate::CreateTokens`
     carries one def; the choice is a `ChoiceKind` asked in `apply_rewrite`
     the way CR 614.13's auxiliary move is, and the modal template is the
     field that holds the alternatives.

     **Reachability (2026-09-13):** unreachable — the only printed customer
     is unregistered.

     **Sized:** ~40 lines and a `ChoiceKind`, with Jinnie Fay.

128. **"Create … tapped and attacking" has the first half and not the
     second.** `TokenDef::enters_tapped` is CR 110.5b's word on the entry;
     "attacking" is CR 508.4's — a permanent put onto the battlefield
     attacking is attacking without having been declared, which is a
     combat-state write the entry performer does not have and combat's
     validation has never been asked about. 121 printed cards say it
     (Scryfall, `o:"tapped and attacking"`, 2026-09-13), every one a trigger.

     **Reachability (2026-09-13):** unreachable — no registered def says
     "attacking", and nothing could until CR 603.

     **Sized:** a field beside `enters_tapped` and a write into the combat
     state in `place_on_battlefield`'s wake, ~30 lines, with the first
     trigger that creates one — combat's phase to size, not this one's.

What is *not* a ledger line, and where each waits: Xorn's and Chatterfang's
Treasure and Hullbreacher's are `backlog.md` §2.19's (a Treasure def needs
any-color mana); Academy Manufactor's "one of each" is §2.27's library;
Ojer Taq's back face is CV-5's; Chatterfang's variable sacrifice cost is
`cost-architecture.md`'s.

### Found by RE-5 — counters, on permanents and players (2026-09-13)

**Shipped:** `CounterSubject { Object, Player }` on `GameAction::AddCounters`
and `RemoveCounters` and on `GameEvent::CountersChanged`; `AddCounters::by`,
the player putting them on — the resolving effect's controller, and at an
entry CR 122.6a's default; `EventPattern::AddCounters { counter, by:
Option<PlayerSet> }` and `RemoveCounters { counter }` — one arm per variant,
split at the review from a shared `CounterChange { adding }` — the first
watching an `AddCounters` and, CR 122.6's second door, an `EnterBattlefield`
whose mods carry a matching kind with one or more, asking "one or more" as
the rule does; `EntryCounters { counter, n, by }` rows on `EnterMods`, merged
on `(kind, putter)`, and `by: Option<PlayerRef>` on the two counter
primitives (the review's theme A); `Rewrite::Amount`'s two counter
legs (a proposal's count; each matched kind in an entry's mods, a kind at
zero leaving them); `PlayerState.counters: BTreeMap<CounterType, u32>` in
place of `poison_counters`, `CounterType::{Poison, Energy}`, CR 704.5c
reading the map; `Primitive::GetCounters`; the multiplier shape's entry
clause (item 47's condition (c), §11 item 84). Doubling Season, Hardened
Scales (pooled), Vorinclex, Monstrous Raider, Winding Constrictor, Live Fast,
Primal Vigor. Item 43 closed and evicted; `backlog.md` §2.16 graduated.
**Left absent, with its customer named:**

129. **A cost that puts counters has no fact on the event that says so.**
     Doubling Season's ruling — loyalty paid as a cost "isn't doubled …
     because those counters are put on as a cost, not as an effect" — and
     CR 614.16's own "the effect of a resolving spell or ability" both
     exclude a cost's counters from the counter doublers. `Cost::AddCounters`
     is unimplemented today (`engine/costs.rs` returns `Err` for validation
     and payment), so no proposal a cost makes exists to be wrongly matched.
     When it does, the payment's `AddCounters` needs a cause the pattern's
     `AddCounters` arm refuses — `LifeLossCause::Cost`'s shape — and
     `pattern_watches` one clause.

     **Reachability (2026-09-13):** unreachable — no cost puts counters;
     `backlog.md` §2.11's loyalty abilities are the producer.

     **Sized:** a `CounterCause { Effect, Cost }` on `AddCounters`, written
     by every producer, one clause in `pattern_watches`, ~20 lines, with
     §2.11's first loyalty ability.

What is *not* a ledger line, and where each waits: CR 122.6a's named putter
was closed on an empty Scryfall query and built at the review (§11 item
83); Doubling Season's battles are
`backlog.md` §2.23's; paying {E} is a cost, `cost-architecture.md`'s CP-1
slot with its first card; proliferate (CR 701.34a) is `backlog.md` §2.5's
and reads the map this phase built; the additive commutation shapes were
§2.29's and are built — the review's theme B made the predicate the
commutation table that entry designed (§11 item 85).

### Found by RE-8 — the producers: discard and scry (2026-09-14)

**Shipped:** `Primitive::Discard(AmountExpr, DiscardChooser)` with CR 701.9b's
default and "at random" choosers, moving N cards as **one batch of N members**
(`Primitive::Mill`'s argument, and CR 603.2c's unit); `GameState::
random_cards_from`, the first "at random" that is not a shuffle, drawn from the
game's own `rng`; `GameAction::Scry { player, n }` with `EventPattern::Scry`,
`GameEvent::Scried` and a performer that reorders the library in the arm and
proposes nothing (CR 701.22 moves no card between zones); CR 701.22b as
`never_happens`' fourth arm, where RE-2's `DrawCards { n: 0 }` still has none;
`ChoiceKind::{Discard, Scry, ScryOrder}` — `Discard` widened from
`DiscardToHandSize`, one kind for CR 514.1's turn-based action and a
resolution's instruction alike, with `source` the only difference;
`ReplacementDef::by: Option<SourceFilter>`, CR 101.2's "by" asked of a
replacement effect, with `SourceFilter::matches` moved onto the type so a
"can't" and a replacement ask one question with one evaluator, and
`pipeline::cause_of` naming the expression that answers it from a proposal's
context; `GameActionTemplate::DrawCards.n` as a `TemplateAmount`, and
`substitute`'s third draw leg turning a scry into a draw. Mind Rot, Hymn to
Tourach, Nephalia Academy (pooled: Mind Rot and Opt), Opt, Eligeth, Crossroads
Augur. `backlog.md` §2.5's RE-8 line struck; `--require` made repeatable.

**And CR 514.1's cleanup discard was reshaped rather than recorded**, on this
section's own thirty-line rule: the rule is one turn-based action over "enough
cards" and the engine asked one card at a time in a `while` loop, so a hand of
ten made three batches where a CR 603.2c trigger should see one
(`replacement-architecture.md` §11 item 88). **Left absent, with its customer
named:**

130. **A discard redirected into a hidden zone has undefined characteristics,
     and nothing models it.** CR 701.9c: a card discarded but put "into a
     hidden zone instead of into its owner's graveyard **without being
     revealed**" has every characteristic undefined, and a cost that named one
     of those characteristics becomes an illegal payment (CR 732). Nephalia
     Academy produces the board — hand to library, both hidden — and its "you
     may reveal that card" is the clause that avoids the rule, which the
     engine does not model at all: `Zone::is_public` exists with **zero
     callers** and no query takes a viewing player (`backlog.md` §2.9).

     **Reachability (2026-09-14):** unreachable — the rule's consequence is
     about a *cost*, and `Cost::Discard` returns `Err` at both its validation
     and its payment arms (`engine/costs.rs`), so nothing reads a
     characteristic of a card discarded this way. `ATOM-701.9c-001` is the
     corpus's board and is uncovered, filed Phase 8.

     **Sized:** a `revealed: bool` on the discard's zone change or, better,
     §2.9's per-viewer query answering it; plus the validation half of
     `Cost::Discard`. ~60 lines, and it arrives with §2.9 rather than before —
     a flag written by one card is the shape §2.9 exists to replace.

131. **A substituted zone change overwrites the replaced event's `cause`, and
     the cause is the only thing several facts are written on.** A discard is
     the instance that found it; a resolution is the one that bit.
     `GameActionTemplate::ZoneChangeTo` carries the substitute's
     `ZoneChangeCause` and overwrites the replaced event's, so under Leyline of
     the Void a discarded card's performed event is `ZoneChange { to: Exile,
     cause: Exiled }` and the `Discarded` cause is gone. The card *was*
     discarded: Dodecapod's and Wilt-Leaf Liege's rulings say "you've still
     discarded it. Abilities that trigger whenever you discard a card will
     trigger", and CR 701.9c calls a card put into a hidden zone this way
     "discarded" while describing the move. Found by reading a `--dump-events`
     log at RE-8's review (`replacement-architecture.md` §11 item 90).

     **The same erasure, on `Resolved`, is what made RE-8's own Hymn to
     Tourach row look wrong.** `events/event.rs` says a spell finishing
     resolution *is* "a `ZoneChange` out of the stack with
     `ZoneChangeCause::Resolved`" — the log's only signal for it — and under
     Leyline of the Void a resolved sorcery leaves as `Exiled` instead.
     `fuzz_games` read that cause and under-counted; it reads
     `EventStamp::resolution` now, which no rewrite can touch, because a
     substitution happens *inside* the resolution that proposed the event
     (`replacement-architecture.md` §11 item 91).

     **Reachability (2026-09-14):** reachable, and **not wrong today**, and the
     bound is worth stating: every *engine* reader of a cause reads the
     **proposal**, before any rewrite — `pattern_watches`' `ZoneChange
     { cause }` and `EnterBattlefield { cast }`, and `is_prohibited` through
     the same. The erasure is only on the **performed** event, whose one reader
     today is the harness, which is why it surfaced there and not in a test. It
     is wrong the day critical-path item 6 lands, and the board is unforced:
     **13 times in 200 `stress` games**.

     **Sized:** not thirty lines. The field is RB's with three deliberate
     customers (CR 122.1h's finality counter, CR 903.9b's commander, Kalitas),
     and the question is what a substitution *about the destination* should do
     to the reason the object is moving — `cause: Option<ZoneChangeCause>`
     meaning "keep the original", or that rule by default. ~80 lines plus a
     re-reading of every RB def, and it belongs to whoever builds
     critical-path item 6's zone-change matcher, who is the first reader that
     can tell it is wrong. **Where a stamp fits, it is the pattern to copy** —
     it answered the harness's question exactly and needs no field at all.

     **A third instance, sharper than both (RF's review, 2026-09-16).** Under
     Darksteel Colossus's clause a milled member's substitute is a zone
     change from the library *to the library*, and `perform_zone_change`
     performs and announces nothing for a same-zone move — so there is no
     record at all on which a kept cause could ride. The owner's wrinkle:
     "whenever an opponent mills one or more cards, you gain life equal to
     the number milled", on a mill of three with a Colossus in it. The
     performed stream shows two `Milled` records (and none under Rest in
     Peace, whose substitute writes `Exiled`); CR 701.17c's framing — "the
     zone it moved to from the library" — treats a milled card that went
     elsewhere as milled all the same, which reads 3, while the owner's first
     reading was 2. Which is right is item 6's to settle with 701.17c open;
     what RF adds is that a *stamp* is the only shape that can carry the
     fact, because a stamp can exist where no record does.

     **Scheduled (2026-09-15, post-RE audit):** critical-path item 6's
     zone-change matcher, as sized above; `replacement-architecture.md` §14
     lists it among what item 6 inherits. The board had counted this item as
     a wrong answer since 2026-09-14 because its verdict contains the words
     "wrong today"; the classifier learned the negation the same day.

What is *not* a ledger line, and where each waits: the to-battlefield entry
substitution and the five cards that print it are CR 113.6's, critical-path
item 6a (§11 item 87); `fuzz_games`' `resolved` counter read a cause where the
log's own signal for "this spell resolved" had been erased, which RE-8's
measurement caught and RE-8 fixed by reading `EventStamp::resolution` instead
(§11 item 91) — a harness bug, so no line here; CR 701.9b's third chooser — "another player chooses",
Coercion — is `backlog.md` §2.5's with its first card, and wants §2.9's reveal
beside it; CR 701.22c's simultaneous scry in APNAP order has no producer, no
effect making more than one player scry, and the corpus already defers it;
`Cost::Discard` itself is `backlog.md` §2.5's unimplemented half and was not
made reachable by this phase.

### Found by RE-9 — mana (2026-09-15)

**Shipped:** `GameAction::ProduceMana { player, source, mana, special,
tapped_for_mana }` — CR 106.6a's "the amount of mana produced by a spell or
ability", CR 106.12b's "the mana production event" — proposed by
`resolve_mana_effect` (a mana ability, now with a source and a context) and
`Primitive::ProduceMana` (a spell) and performed by one `perform_action` arm,
the only `mana_pool.add` and `add_special` outside tests; `GameEvent::ManaAdded`
emitted for the first time since the log was written, its `HashMap` a `Vec`
in proposal order (`replacement-architecture.md` §11 item 96) and
`tapped_for_mana` on it for CR 106.12a's triggers; `tapped_for_mana` read off
the activation cost, which is CR 106.12's definition (item 93);
`EventPattern::ProduceMana { tapped_for_mana, source }`; `Rewrite::Amount
(Multiplier)` scaling every plain unit and repeating every restricted atom,
CR 106.6a's "all mana produced" (item 95); `GameActionTemplate::ProduceMana
{ mana_type, amount }`, CR 106.12b's "specific type", retyping in place under
`ReplacedAmount` and refusing a mixed production under `Fixed`; the
`Mana productions` diagnostic row, and `fuzz_ab.py`'s compare taught to drop
a row the baseline never prints. Mana Reflection (pooled), Nyxbloom Ancient,
Deep Water; at review, Pale Moon (§11 item 98's census re-run) and Doubling
Cube — the corpus's own integration test for CR 106.6, whose dynamic amount
is `AmountExpr::UnspentMana` and the first a mana ability has carried, so
`resolve_mana_effect` evaluates through `evaluate_amount` against a
targetless resolution context rather than reading `Fixed` alone; item 111
closed. **The last of RE's ten PRs.**

**Left absent, with its customer named:**

132. **~~`GameState.counters` is the engine's diagnostics, and the two
     fields one struct over with the same name are CR 122's counters~~
     ✅ CLOSED 2026-09-18 (A4m, PR #165) — the type is `Diagnostics` and the
     field is `game.diagnostics`.** The owner's name rather than this
     entry's proposed `EngineMeters`: the module is already
     `state::diagnostics`, so the type stops fighting its path. **The sizing
     was low** — 120 lines across 21 files against this entry's 81 and row
     A4m's 72, because neither count included an argument site or a test
     file.
     → `plans/archive/codebase-state-closed.md`. **No `fuzz-record.md`
     block, because nothing moved:** both pools read `IDENTICAL` at two
     seats and at four, which is the whole claim a rename makes.

     **Reachability (2026-09-18):** closed — landed; every printed row label
     the recorded tables are keyed on is untouched.

133. **`EventPattern::ProduceMana` has a field for one of CR 106.12b's three
     axes.** The rule: a replacement applying "if a permanent 'is tapped for
     mana' or tapped for mana **of a specific type and/or amount**". The arm
     asks which permanent and nothing else; each of the other two has a
     printed card, and so does the *chosen* permanent the `source` filter
     cannot name (`replacement-architecture.md` §11 item 98, the review's
     census re-run).

     - **Type** — False Dawn ("would add *colored* mana"; its second
       sentence is a spend-as-any-color payment rule `ManaPool` lacks) and
       Quarum Trench Gnomes ("instead of *white* mana"). `mana_type:
       Option<ManaType>`, one `pattern_watches` clause, `reads_the_amount`
       still `false` — a type is not a count. ~15 lines.
     - **Amount** — Damping Sphere, "if a land is tapped for **two or more**
       mana, it produces {C} instead of any other type and amount".
       `at_least: Option<u64>`, Alms Collector's field, and **it reads the
       amount**: `reads_the_amount` becomes `at_least.is_some()`, so a
       doubler beside it keeps CR 616.1's question — Damping Sphere's first
       ruling is that board ("choose one to apply. After that, determine if
       any others are applicable"): a one-mana land under Sphere and
       Reflection is {C} with no choice (the Sphere is not applicable until
       the doubling makes it so), a two-mana land is 1 or 2 by the order.
       ~20 lines plus the ordering test; `engineering-practices.md` §4.1's
       question is owed at the `reads_the_amount` arm. Damping Sphere's
       second ability counts spells cast this turn per player, which nothing
       tracks, so its first line is a fixture until then.
     - **Chosen permanent** — Quarum Trench Gnomes, "{T}: If *target* Plains
       is tapped for mana, it produces colorless mana instead of white mana.
       (This effect lasts indefinitely.)" `source` becomes a `SourcePattern
       { object, filter }` as `DealDamage`'s is (~20 lines, the three defs
       wrapped), plus a `PatternFill` arm that writes the target into the
       row (Circle of Protection's `ChosenDamageSource` shape, ~20 lines)
       and an indefinite `Duration` for a row an activated ability makes.

     **Reachability (2026-09-15):** unreachable — no registered def names a
     type, an amount or a chosen permanent, and the pattern has no field to
     name any with. Pale Moon, the review's other find, needed none of the
     three and is registered.

     **Sized:** ~15 + ~20 + ~40 lines, each with its card: False Dawn after
     `backlog.md` §2.19's payment rule, Damping Sphere after a
     spells-cast-this-turn count (a `PlayerState` field the cast path
     increments and the turn resets, ~20 lines, CM's), the Gnomes after the
     fill arm. The amount field is the one that changes a proof and is owed
     the standing question when it lands.

134. **Three of the six printed type-changers want three facilities RE-9 did
     not build.** Hall of Gemstone ("that player chooses a color … lands
     tapped for mana produce mana of the chosen color") needs a chosen color
     stored on the permanent by an upkeep trigger — item 6's, plus a
     `ChosenColor` read in the template. Naked Singularity ("Plains produce
     {R}, Islands produce {G}, …") needs a template whose type is a function
     of the tapped permanent's subtypes rather than a constant. Harvest Mage
     ("one mana of a color of your choice") needs a choice *inside* the
     substitution, which `pipeline::substitute` is a pure function on purpose
     — RC-5's "an application that prompts" shape, on a different arm, and
     the one of the three that changes the pipeline's contract.

     **Reachability (2026-09-15):** unreachable — none is registered, and each
     of the three needs its facility before its def can be written.

     **Sized:** Hall of Gemstone ~40 lines after item 6; Naked Singularity a
     `TemplateManaType::BySubtype(Vec<(LandType, ManaType)>)` beside the
     constant, ~40 lines; Harvest Mage is `substitute` gaining a `ctx` and a
     prompt, ~80 lines and a design question about which prompts a
     substitution may ask. Contamination and Infernal Darkness need none of
     this — their lines are fixtures in `tests/phase_re9_integration_test.rs`
     and register the day item 6 owns their upkeep halves.

135. **A mixed mana production under a fixed retype is refused, not
     decided.** `GameActionTemplate::ProduceMana { amount: Fixed(n) }` on a
     production carrying both restricted and unrestricted units returns `Err`
     from `substitute`, because no rule says which restriction the `n` new
     units carry: CR 106.6a is about an ability's restrictions applying to all
     of its mana, and every printed mana ability produces mana that is
     uniformly restricted or uniformly free. Loud, over a silently dropped
     restriction; `a_mixed_production_under_a_fixed_retype_is_refused` asserts
     the refusal.

     **Reachability (2026-09-15):** unreachable — no registered ability mixes
     the two, and no printed one does (Scryfall, 2026-09-15).

     **Sized:** one arm in the `Fixed` leg, ~10 lines, the day a card brings
     the ruling that decides it. Recorded rather than guessed because a wrong
     answer here is a restriction that vanishes from a pool with nothing
     pointing at it.

136. **The chokepoint's fixed cost per event is the whole of what a proposal
     with nothing watching it costs, and the one lever §8 pre-approved cannot
     touch it.** RE-9's review probe: the engine arm with `gather` returning
     early for `ProduceMana` — what the event-kind bitmask would compute on a
     board with no mana watcher — reads +0.1% at two seats and −0.6% at four
     against the engine arm, rounds overlapping both times, where RE-1's
     same probe returned half of its cost. A production's sweep is three
     memo reads; what the +1.2% buys is `execute_batch_inner`'s per-batch
     work for a single member with no candidate: the `groups`, `decided`,
     `applied_to` and `riders` `Vec`s, the cloned `HashSet` per member,
     `apnap_batch_order`, the entry-selection and allocation saves,
     `is_prohibited`, the frame, and the emit — about 2 µs a proposal, 81
     proposals a game, on the commonest proposal after phases and steps. The
     same probe put a mana replacement *applying* on every tap, board held
     fixed, at **+2.7%** and **+2.2%** (`replacement-architecture.md`, the
     RE-9 archive's "Measured again at review").

     **Reachability (2026-09-15):** reachable, and not wrong — a cost. Every
     answer is right; the price is paid on every land tap of every game.

     **Sized:** an answer-preserving fast path, §8's own criterion: a batch
     of one member whose `gather` returns nothing, whose `is_prohibited` says
     no and whose action is not an entry performs directly, allocating none
     of the above, with the emit and the batch id unchanged so CR 603.2c
     sees the same event. ~40 lines in `execute_batch_inner`, a debug
     assertion that the fast path and the loop agree, and the A/B that says
     what it returns — measured before it is kept, like RE-1's gate. Not
     this PR's: RE's exit criteria name lever 2 as the pre-approved
     optimization, this is a different lever, and §8's ordering is measure
     first. The number to beat is +1.2% at two seats and +0.7% at four; the
     number that says whether it matters is a v1 CPU budget the plan has
     never set, and this item is where to write it when it is. **Set
     2026-09-15 as a ratchet (the post-RE audit's pass 3, item 138):** a PR
     may cost 2.5 points of CPU per decision on `performance`, and each
     spine close may not read worse per decision than the last; this
     lever's 1.2% is half of one PR's budget, which ranks it last of the
     levers item 138 lists.

137. **`ResolutionContext` carries CR 615.5's two rider numbers as two
     `Option<u64>` fields that every non-rider resolution sets to `None`.**
     Their own docs say "for a rider and for nothing else"; the type says
     two independent optionals, so every site that builds a context — seven
     in `src/`, forty-one in `tests/` — writes `replaced_amount: None,
     damage_prevented: None` about a mechanism it has never heard of, which
     the RE-9 review called out at the mana resolver. The honest shape is one
     optional thing: `rider: Option<RiderAmounts { replaced_amount:
     Option<u64>, damage_prevented: u64 }>` — `None` on every ordinary
     resolution, and inside a rider the prevented amount is a plain number
     (its `Some(0)` case is a rider that prevented nothing) while the
     replaced amount stays optional (a zone change has none).

     **Reachability (2026-09-15):** reachable, and not wrong — a shape.
     `AmountExpr::ReplacedAmount` and `DamagePrevented` refuse correctly
     outside a rider today; what is wrong is what forty-eight sites have to
     say to construct a context.

     **Sized:** the struct, the one writer (`resolve_rider` in
     `engine/actions.rs`), the two readers in `evaluate_amount`, and a
     mechanical sweep of the literals — ~48 sites, most of them
     `rider: None` — **its own PR**, on main item 124's precedent that a
     sweep does not ride inside a rules change. `ResolutionContext::
     untargeted(source, controller)` is the call-side half and shipped with
     RE-9; the mana resolver uses it.

What is *not* a ledger line, and where each waits: CR 605.1b's triggered mana
abilities — Wild Growth's "whenever enchanted land is tapped for mana, its
controller adds an additional {G}", eight cards — are critical-path item 6's,
and what they will propose is already fixed by the definition: a second
`ProduceMana` with `tapped_for_mana: false`, which is Mana Reflection's second
ruling ("that triggered mana ability won't be affected") before a trigger
exists; Virtue of Strength, the third multiplier, is an Adventure card and
waits for the second face (`backlog.md`'s Adventure entry when it is written);
`backlog.md` §2.19's any-color mana rewrites `resolve_mana_effect` next and is
ordered after this PR as `replacement-architecture.md` §9 said; and the
`--dump-events` A/B recipe that counted `Tapped:` land lines because the log
had no mana lines now has `ManaAdded:` lines to count, which is a note for
whoever next masks a dump and not a migration.

### Found by the post-RE audit (2026-09-15)

**The close-out of critical-path item 5** — pass 1 of
the post-RE audit ("Was critical-path item 5 done, and what sits before item 6? — audited 2026-09-15" above), the day after RE-9 merged. It read the
done-checklist off the tree and gave every entry a disposition where the
entry lives: `replacement-architecture.md` §8a (the four missing event
kinds), §11 items 3, 4 and 14, §12 (re-read), §13 (brought current), §14 (the
phase in hindsight, new); `backlog.md` §3.3 (the Phase 6 `owed` triage, 50
atoms, and `SHIPPED_PHASES` armed), §2.30 and §2.31 (two mechanics with no
surface); this file's items 59, 60, 122 and 131 (scheduled), 88 and 118
(closed, one of them fixed here), 116 and 121 (re-worded so the board reads
them); and the board's classifier, which had read "not wrong today" as
"wrong today". One migration was found that no doc owned:

134. **CR 614.12b — the combined costs of several entry choices, across
     permanents entering simultaneously.** "If multiple replacement effects
     that require choices from a player would modify how multiple permanents
     enter the battlefield simultaneously, that player may not make choices
     for those effects that would cause the combined costs of those effects
     to not be payable." `replacement-architecture.md` §12 parked it on cost
     modification; CM-1–CM-4 landed and it is still out, because the
     prerequisite was misnamed. It needs an entry replacement whose *choice
     has a cost* — the printed shape is a shockland's "As this enters, you
     may pay 2 life. If you don't, it enters tapped", and `EnterModsTemplate`
     carries `tapped` and `counters` only — and a plural entry of *cards*
     (RE-4's plural batch is tokens'; a Scapeshift, a mass reanimation), and
     the rule bites only with both at once and a player who cannot pay for
     all of them.

     **Reachability (2026-09-15):** unreachable — no registered entry
     replacement has a cost, and no producer enters two cards at once.
     Reachable the day both exist, which is Phase 8's second shockland
     beside its first mass entry.

     **Sized:** a `cost: Option<Cost>` on `EnterModsTemplate`, paid at
     application, ~60 lines with the first shockland; then the 614.12b check
     itself — the CR 616.1 loop's decide phase already sees the whole batch
     (CR 704.3's shape), so it is a payability sweep over the batch's chosen
     costs before any is performed, ~40 lines, with the second customer.
     Two customers before a leaf (`replacement-architecture.md` §8c).

What is *not* a ledger line, and where each waits: the four §8a event kinds
are dispositioned in `replacement-architecture.md` §8a itself (turned face
up → CV-6; dice → `backlog.md` §2.31; search → `backlog.md` §2.5's producer;
countering → closed, a zone change with a cause); Master Biomancer's type
(item 60) is `backlog.md` §2.30's, a mechanic with no surface; the
self-replacement producer is `replacement-architecture.md` §11 item 3's,
with its reachability line there; and the eviction of that document's
pre-build reasoning is planned section by section in the handoff and opens
as its own PR after this one merges.

### Found by the post-RE audit, pass 3 — parallel-play readiness (2026-09-15)

**Is the engine on track for the AI-harness use case?** Pass 3 of
the post-RE audit (its record: "Was critical-path item 5 done, and what sits before item 6? — audited 2026-09-15" above) measured rather than argued: a throwaway
counting provider around `fuzz_games`' own decks and streams (draw for draw,
so the games are the fixture tables'), a clone timer with a counting
allocator, and item 41's fork test run as a probe — record every answer,
clone at a round start, replay, compare. Nothing here is engine code; the
probe was thrown away, `plans/panic_surface.py` was kept. The answer is
yes with one number to set (138), one thing on the stack that should not be
(139), one entry point missing (140), and the serialization question closed
(141); the ask classification is `backlog.md` §2.22's table, and the
Commander-scale board closes item 69.

138. **The throughput target, proposed, with the instrument that reads it and
     the levers ranked against it.** The metric is the handoff's (§4, §6
     decision 3): decisions per core-second at four seats, random providers,
     `--threads 1`. **A decision is a `DecisionProvider` prompt with two or
     more options.** The engine already asks no inner question with fewer
     (`CLAUDE.md`'s rule), so the only forced prompts are priority prompts
     whose list is `[Pass]` — 91.5% of all priority prompts at four seats. A
     prompt count would measure the pass loop; a decision count measures
     what an agent is handed.

     **Measured 2026-09-15**, 200 games at 60 cards and 100 at Commander scale
     (four 100-card decks, 40 life), seed 12345, one thread:

     | | prompts / game | decisions / game (priority + inner) | CPU / game | per core-second | µs / decision |
     |---|---:|---:|---:|---:|---:|
     | `performance`, 2 seats | 872 | 282 (103 + 179) | 17.2 ms | 16,400 | 61 |
     | `stress`, 2 seats | 1,062 | 446 (167 + 279) | 25.0 ms | 17,900 | 56 |
     | **`performance`, 4 seats** | 2,544 | **513** (188 + 325) | **51.2 ms** | **10,000** | 100 |
     | `stress`, 4 seats | 3,024 | 833 (314 + 519) | 75.8 ms | 11,000 | 91 |
     | `performance`, 4 seats, Commander scale | 3,469 | 692 (248 + 445) | 87.1 ms | 7,950 | 126 |
     | `stress`, 4 seats, Commander scale | 4,278 | 1,253 (476 + 777) | 136.6 ms | 9,170 | 109 |

     The CPU column is this machine on this day (RE-9's 44.83 ms game reads
     51–52 ms here) and drifts like every timing number; every other column
     is a pure function of the seed. Between prompts the engine spends 23–40
     µs; between decisions, 56–126 µs.

     **The target — proposed as 20,000 decisions per core-second, set by
     the owner on review (2026-09-15) as a ratchet instead.** The objection
     that decided it: today's pool has few abilities per permanent and no
     triggers, so 100 µs a decision is a floor of what Commander will cost,
     and a number about a board that does not exist yet cannot be watched —
     but neither can "as little as possible", which never says when a
     regression matters. Three rules make minimization watchable:
     1) **a per-PR budget** — a phase's `performance` A/B at four seats may
     cost at most 2.5 points of CPU per game at identical counters, CPU per
     decision once the counters below land, or the PR says why (RE-9's gate,
     `replacement-architecture.md` §11 item 54, now the rule in
     `engineering-practices.md` §3.1); 2) **a dated reading at each spine
     close** — the recurring audit's readiness pass records decisions per
     core-second on both boards beside the previous close's, and it may not
     read worse per decision without a written reason; **the first reading
     is this table's: 10,000 on the 60-card `performance` board and 7,950
     at Commander scale, 2026-09-15, this machine**; 3) **the use-case
     check**, the only form that is about the harness rather than the
     machine — the engine is fast enough when the policy network, not the
     engine, bounds the loop, which is cores per GPU for a stated policy
     size, re-derived when the board changes; at today's rate a policy
     batching a thousand games per GPU-millisecond needs about a hundred
     cores per GPU (§4's arithmetic).

     **The instrument, built 2026-09-16 (A4e, PR #155).** Two cells on
     `Diagnostics`, `decisions` and `priority_decisions`, recorded in
     `ui::ask`'s four `validate_*` helpers rather than in the 24 bodies: each
     helper runs exactly once per prompt with the candidate list *and* the
     bounds in scope, so what counts is decided once per primitive instead of
     once per caller, at four sites instead of twenty-four. Two
     `=== Engine Work ===` rows, their `ROWS` entries in `plans/fuzz_ab.py`,
     and `µs / decision` beside `ms / 1,000 walks` in its timing table. No
     A/B: both pools at two and four seats read `IDENTICAL` to `main` on
     every other row, which is the proof a `Cell` increment changed no game.
     The rows are new, so no re-record — the fixture tables gain them at the
     next re-record. The reading that watches the target is
     `CPU/game ÷ decisions` in a sitting, never a stored millisecond
     (`engineering-practices.md` §3).

     **The definition the table above was measured with is not the one it
     states, and the instrument ships the stated one.** "The engine already
     asks no inner question with fewer" is false in this tree, so the probe's
     count — every prompt but a priority prompt offering only `Pass` — counted
     forced prompts as decisions. Measured at `performance`, four seats, 200
     games: `choose_generic_mana_allocation` is forced 4,738 times of 8,255
     (one bucket can take what is left), `select_recipients` 1,319 of 6,084
     (one legal recipient with `min == max == 1`), a trample split 11 of 34.
     **Thirty forced inner prompts a game, and nothing else** — item 145. The
     cell counts a prompt with **two or more legal answers**, per primitive:
     `pick_n` unless the count is fixed at none of the options or at all of
     them, `pick_number` when `max > min`, `allocate` when something is left
     over every bucket's minimum and two or more buckets can take it,
     `choose_ordering` at two items. That keeps the take-it-or-decline prompts
     a *candidate* count would drop — a mana window with one untapped source,
     13 a game — and drops the 30.

     **The instrument reproduces the census before anything else moved.** On
     the counters-only arm — the commit before item 145's fix, A/B `IDENTICAL`
     to `main` on every other row — `priority_decisions` reads 103 / 167 / 188 /
     314, which is the table above exactly, and the prompt counts reproduce
     `backlog.md` §2.22's. So the instrument sits on the census's prompt sites
     and the only thing that differs is the rule: 266 / 407 / 483 / 764 against
     the table's 282 / 446 / 513 / 833, the forced prompts and nothing else.

     **The six boards as shipped** (seed 12345, one thread, 200 games at 60
     cards and 100 at Commander scale, item 145's guard in). Every row moved
     from the paragraph above because the guard removes prompts and a prompt is
     an RNG draw, so these are different games — not a different engine cost:

     | | decisions / game (priority + inner) | CPU / game | per core-second | µs / decision |
     |---|---:|---:|---:|---:|
     | `performance`, 2 seats | 236 (96 + 140) | 14.9 ms | 15,800 | 63 |
     | `stress`, 2 seats | 376 (161 + 215) | 24.3 ms | 15,500 | 65 |
     | **`performance`, 4 seats** | **464** (188 + 276) | **52.8 ms** | **8,790** | 114 |
     | `stress`, 4 seats | 759 (322 + 437) | 77.5 ms | 9,800 | 102 |
     | `performance`, 4 seats, Commander scale | 692 (268 + 424) | 107.8 ms | 6,420 | 156 |
     | `stress`, 4 seats, Commander scale | 1,163 (487 + 676) | 161.6 ms | 7,200 | 139 |

     **So the ratchet's first reading is 8,790 decisions per core-second on the
     60-card `performance` board at four seats and 6,420 at Commander scale,
     2026-09-16, this machine.** The 2026-09-15 rows keep their date and their
     numbers; what they may no longer be read as is a decision count.

     **And the first thing the ratchet taught, on its first reading:** a change
     that moves the RNG stream re-bases it. Item 145's guard makes the engine do
     strictly less — 30 fewer prompts a game — and the reading *fell* 4%, because
     the games that follow a moved stream are different games. A reading is
     comparable across a change that leaves the stream alone and across nothing
     else; the arm that proves this one is the counters-only arm above, and the
     rule this makes concrete is `engineering-practices.md` §3's, "never A/B any
     number across a pool change", one level up.

     **What the count is mostly made of, and why it will fall.**
     `activate_mana_ability` is 194 of the 325 inner prompts a game at four
     seats — 60% of them, none forced under any rule — because CR 601.2g
     re-enters the window once per tap. `backlog.md` §2.18's tap solver, or an
     auto-payer answering the window, deletes most of them: the decision count
     drops sharply the day one lands, with the engine no slower and `CPU /
     decision` up by construction. **Read a fall in `Decisions` as a
     middleware landing until that is ruled out**, and read the two rows
     together — `priority_decisions` is the half no middleware may remove.

     **The Commander-scale board is a run now, not a source patch.**
     `--deck-size` and `--life` landed with the cells, so
     `--deck-size 100 --life 40 --players 4` is that board and
     `plans/fuzz_ab.py` passes both through. It had to be built because the
     two Commander rows above **could not be reproduced**: the probe's deck
     recipe is written down nowhere, and two plausible ones bracket them —
     76 nonlands of 100 reads 249 priority decisions on `performance` against
     the recorded 248 but 384 on `stress` against 476; 60 of 100 reads 472 and
     83.7 turns (item 69's "83-turn games") but 271 on `performance`. The flag
     scales the 36-of-60 nonland ratio, so **60 is the identity** — every
     recorded table's deck, card for card, RNG draw for RNG draw — and 100 is
     60 nonlands and 40 lands. The rows above are that board's, and they are
     the ones a later reading can be compared against.

     **The profile, taken** — after the review, callgrind under WSL over 200
     four-seat `stress` games (`layers-architecture.md` §12, "Measured at
     four seats on `stress`, 2026-09-15"): 865 M instructions a game, the
     counters identical to the native run's, and the engine's own logic a
     minor share — the instructions go to cloning, hashing and allocating
     the ability tree. The ranking below is that profile's.

     **The levers, sized and ranked by instruction share** — the profile's
     inclusive figures, which overlap, so they do not add:

     1. **The ability list behind an `Arc`** — 22.5% of all instructions
        clone `Vec<AbilityDef>`: 6.6% seeding a frame from `CardData` on a
        walk (item 67), and ~15% `get_effective_abilities` copying the list
        *out of a memo hit* for `activatable_abilities`,
        `available_mana_sources`, `is_prohibited` and `gather`.
        `CardData.abilities` and `EffectiveCharacteristics.abilities` as
        `Arc<Vec<AbilityDef>>`, the wrapper returning the `Arc`,
        `Arc::make_mut` in the Layer 4 and 6 arms — item 67's shape,
        ~100–150 lines plus 13 call sites, answer-preserving — and most of
        the 24% spent in `malloc` and `free` goes with it. **Rank 1.**
        **Landed 2026-09-16 (A4f, PR #157):** the `to_vec` row is gone and
        the game fell 31.4% in instructions on the same pool, 30.6% in
        native CPU at four seats; `hash_one` did not move by an instruction
        and is now 32.6%, so lever 2 is next. → `layers-architecture.md`
        §12, the 2026-09-16 re-read.
     2. **Process-stable ids in place of the v4 `Uuid` keys** — 22.1% hashes
        16-byte keys with SipHash for every memo, object and battlefield
        lookup (13.5 M `is_creature` lookups in 200 games alone). First
        sized as a `BuildHasher` over the v4 keys, ~30 lines plus a sweep of
        the map declarations; **decided by the owner on 2026-09-16 (item 144)
        as the larger change instead**: `ObjectId` a `u64` from the state's
        counter, `AbilityId` derived from the card, the hasher a one-line mix
        riding inside it, the `uuid` dependency gone — which also halves
        every id and retires the log masks. The one cost, and the fix that
        rides with it: iteration order becomes process-stable, so the CI
        determinism step seeds the hasher per run. **Rank 2.**
        **Landed 2026-09-16 (A4g, PR #158):** built as item 144 decided, in
        two arms — the type swap alone took the game from 622.4 M to 568.8 M
        instructions (−8.6%; SipHash over eight bytes instead of sixteen),
        the hasher on top to 380.7 M (**−38.8%** on A4f's column, −37.3% in
        native CPU per decision at four seats, −34.0% at two). The
        `hash_one` row is gone, `sip.rs` is down from 14.1% to 5.0% and what
        is left of it is the frame's `HashSet<CardType>` and
        `HashSet<Subtype>`; the memo probe fell 72%. The hasher arm is
        identical to the swap arm on every counter under a different seed
        per round; the swap moves six games in 800 against `main` through
        one board — item 149, a per-definition-id consequence, not an order
        leak. → `layers-architecture.md` §12, the second 2026-09-16 re-read;
        `fuzz-record.md`, the A4g block.
     3. **The SBA sweep's per-permanent questions** — `is_creature` 65,000
        times a game, 14.7% inclusive, one per permanent per check; one
        frame read per permanent (§12's `has_subtype` finding, now sized),
        ~40 lines. **Rank 3**, and mostly lever 2 in another place.
     4. **The candidate list per priority prompt** — 27.2% inclusive for
        2,462 prompts a game, 91.5% of them `[Pass]`; the enumeration is
        levers 1 and 2 at work, so its residual is measured after they
        land. A "nothing to do" pre-check or an epoch-keyed cached list is
        the shape, ~30 lines. Item 139 makes the list right; this makes it
        cheap. **Rank 4, re-measure first.**

        **Re-measured 2026-09-16 (A4h), which was this lever's own
        precondition, and two instruments agree.** Callgrind on the
        post-lever tree reads `candidate_priority_actions` at 14.62 G,
        **19.2%** (`layers-architecture.md` §12, the three-arm table);
        A4h's cost arm — one extra enumeration per priority window, wall
        clock — reads **+21.3%** of CPU a game. **Levers 1 and 2 worked on
        it and its share did not move**: 24.71 G → 14.62 G absolute (−41%),
        19.9% → 19.2% of a total that fell with it. So per-call work is not
        the lever left here — the next win **reduces calls or exits them
        early**, which is this row's own "91.5% of them `[Pass]`": the
        dominant cost is proving there is nothing to do.

        **Of the two shapes above the pre-check is the one the data picks,
        and the cache is wrong for the path A4h added.** A re-ask happens
        precisely *because* the board changed (CR 732.1's taps, item 139),
        so a list cached against a board epoch misses on exactly that prompt
        and helps only the pass case. What is cacheable is narrower: the
        **ability inventory** — which permanents a player controls that have
        an activated or mana ability at all — changes on a zone change or a
        Layer 6 grant, while **payability** changes on every tap, and
        `activatable_abilities` and `available_mana_sources` recompute both
        together per prompt today.

        **It may outrank 6 and 7, and that is the owner's call.** §12's rows
        are inclusive and overlap: both `activatable_abilities` and
        `available_mana_sources` iterate `battlefield_ordered()` (lever 6,
        16.8%, which §12 calls the largest single lever left) and allocate a
        `Vec` per call (lever 7, 18.2%). Cutting enumerations cuts the sorts
        and the allocations with them; caching the sort leaves the
        enumeration paying for everything else.
     5. **The mana window** — 15.0%: CR 601.2g's loop re-enumerates every
        mana ability per prompt, 394 times a game at ~220,000 instructions
        each. Levers 1 and 2 shrink it; `backlog.md` §2.18's solver removes
        it, one enumeration per cast. **Rank 5.**
     6. **The timestamp sort** — `battlefield_ordered` and
        `battlefield_ids_ordered` sort on every call, 5.1%; an order cached
        per epoch, ~30 lines, 42 call sites untouched. **Rank 6.**
     7. **Worker-thread scaling, and the allocator** — a worker is one of
        `fuzz_games --threads N`'s OS threads, each playing whole games one
        after another, and the harness multiplies everything. Measured the
        same day, 200 games, `performance` at four seats: 52.2 ms of CPU a
        game at one worker, 68.3 at eight (+31%), 99.1 at sixteen (+90%);
        wall-clock 52.9 → 8.79 → 6.46 ms a game, **6.0× on eight physical
        cores and 8.2× on sixteen threads**; `stress` reads +29% / +82% and
        6.1× / 7.7×; every counter identical across worker counts. The
        inflation is what a global allocator or less allocation could
        recover — up to a quarter of a full box's throughput at eight
        workers, a third at sixteen — and lever 1 removes most of the
        allocation before any allocator is chosen. Sized: a
        `#[global_allocator]` line and one dependency — the supply-chain
        cost the owner raised on 2026-09-15 is the whole price — measured
        with these three runs, after lever 1. **Rank 7 on one core, rank 1
        for a batch.** One doctrine line falls out now: `--threads` defaults
        to sixteen here and eight is the efficient count.
     8. **Item 42, the event-log window**: nothing for straight-line
        throughput; **rank 1 for the fork use case**, where the log is
        fifteen-twentieths of a clone (item 143).
     9. **Item 136's fast path**: +1.2% at two seats, +0.7% at four — half of
        one PR's budget; `gather`'s 14.3% is levers 1 and 2 per permanent,
        not the batch's fixed cost. Rank last.
     10. **Harness-side, not engine**: skipping forced prompts saves the
        provider round trip (~0.5 µs × 2,000 in-process, ~2%; out of process
        it is the difference between shipping 2,544 views and 513). A forced
        prompt consumes no RNG draw (`RandomDecisionProvider::pick_n`
        shuffles a one-element list), so the skip is stream-neutral and an
        engine-side version would be A/B-identical by construction.

     **Reachability (2026-09-16):** reachable — not wrong; a ratchet whose
     instrument now exists, so the next reading is the harness's rather than a
     probe's. What is still owed is the *target* half: a reading at each spine
     close, beside the one restated above.

     **Sized:** ~~the instrument, ~80 lines across `state/diagnostics.rs`,
     `ui/ask.rs`, `bin/fuzz_games.rs` and `plans/fuzz_ab.py`~~ — **built
     2026-09-16** (A4e), and wider than the size by the two harness flags the
     Commander board turned out to need; the next reading is the next spine
     close's.

139. **~~A retry re-prompt offers a list computed before the rejected action
     changed the board~~ ✅ CLOSED 2026-09-16 (A4h) — the enumeration moved
     inside the retry loop, so every priority prompt is built from the board it
     is asked about.** `run_priority_round` had built `all_candidates` once per
     priority window and re-offered it minus the `blacklist`; a rejected cast
     whose mana abilities stayed activated (CR 732.1, item 72) left the re-ask
     offering casts no enumeration of that board would. The blacklist filter
     stays — a fresh list can still hold the action that just failed — and the
     retry budget is taken from the first enumeration rather than re-derived,
     because a budget re-derived from a list the blacklist keeps shortening
     shrinks as it is spent. Item 41's test is what closes it, and it found a
     second mechanism of the same shape on the way (item 150).
     → `plans/archive/codebase-state-closed.md`; `fuzz-record.md`, the A4h block.

     **Reachability (2026-09-16):** closed — landed; 0 of 13,530 forked
     branches fail to replay across the test's 192-game sweep, against 119 on
     `main`.

140. **A prompt mid-round cannot be resumed: `consecutive_passes` and
     `current_priority` are loop locals, and `run_priority_round` starts
     every round at the active player.** So of a four-seat game's priority
     prompts only the round starts can be forked today — 972 of 2,799 on
     `stress`, 1,172 of 3,367 at Commander scale — and the 1,827 / 2,195
     mid-round prompts, each a seat's own decision point, cannot. Neither
     local is outcome-bearing by item 40's test: `priority_player` is already
     on `GameState`, and the passes so far are the turn-order distance from
     the round's first player to it (a player is asked at most once per
     round, in order). What is missing is an entry point, not state. The
     cleanup step's CR 514.3a re-loop is the other unforkable place — its
     priority loop is nested in `Game::run_turn`'s cleanup branch — at 0.05
     prompts a game.

     **Reachability (2026-09-15):** unreachable — nothing forks; the probe was
     a throwaway.

     **Sized:** `run_priority_round` taking the player to start from and
     deriving the pass count, ~30 lines; a `Game` entry that finishes the
     current step from its priority loop rather than re-running the step's
     turn-based actions (the probe did it by hand: `run_priority_loop`,
     `advance_turn`, then `run_turn`), ~30 lines; the `blacklist` either
     onto `GameState`, cleared per round (~20 lines), or made moot by the
     exact action space, which is Phase 10's. Lands with the first
     fork-based harness, after item 139.

     **A4h (2026-09-16) left it two things.** `Game::resume_turn_at_priority`
     exists — `run_turn`'s step drainer takes a `resuming` flag, so a caller
     re-enters at a step's priority round without re-running CR 703.4's
     turn-based actions — and what this item adds is the *other* half of the
     probe's hand-rolled resume: a round that starts at a seat other than the
     active player. And the `blacklist` is no longer "probably": with the
     enumeration fixed and the blacklist left on the frame, 52 of 15,646
     forked branches still failed to replay, each one offered an action the
     original had filtered. It reads 0 today only because item 150 took the
     last action the oracle was offering back — so the first card whose
     activation fails for a reason `activatable_abilities` cannot read turns
     `tests/priority_fork_test.rs` red, and that is the signal this item is
     due rather than a regression in the card.

141. **The `DecisionProvider` boundary, serialized: which fields, which
     crate, what it costs — and the `&GameState` parameter stays.** The wire
     surface is `ChoiceContext` (one field, `ChoiceKind`, 25 variants),
     `ChoiceOption` (12 variants) and `PriorityAction` (4), plus the four
     methods' scalars; its transitive closure over the crate's own types is
     **36 types, 788 definition lines**, and `ChoiceKind::SelectRecipients`'
     `EffectRecipient` is what drags most of it in — `SelectionFilter` →
     `ObjectFilter` → `Subtype` → `CreatureType` (303 lines),
     `PlaneswalkerType`, `PlanarType` — while ids are `Uuid` (`uuid`'s
     `serde` feature) and `usize`. The crate: `serde` with `derive` and
     `serde_json` or a binary codec; `serde` appears nowhere in the tree
     today. **What it costs on the straight-line path: nothing** — a derive
     generates code the in-process path never calls, and the compile-time
     price is the proc-macro crate once. **What it costs per prompt out of
     process is more than the engine**: 2,544 prompts a four-seat game ×
     (microseconds to encode a 100–500-byte prompt plus tens of microseconds
     of round trip) is 30–130 ms against a 51 ms game — which is why §4's
     batched boundary, forced-prompt suppression (item 138's lever 7) and an
     in-process binding are the harness's shape and per-prompt RPC is not.
     **The `&GameState` parameter.** §4 said the provider sees only a
     `ChoiceContext`; the trait hands every method the state, and two
     in-crate providers read it — `RandomDecisionProvider` for its tap
     preference and its X value, the CLI for display — while the decorators
     forward it. Out of process a provider cannot be handed it, so it is
     exactly the observation hook: the adapter at the boundary builds the
     observation from it through `backlog.md` §2.9's per-viewer query (§4
     consequence 3). Keeping it costs nothing, a reference; removing it is
     nine impls times four methods for no gain until that query exists.
     **Decided: keep.** One watch item rides with it — `roadmap-v2.md` §9's
     trait shape (a new method serializes too). The other was `backlog.md`
     §2.21's rule that every `ChoiceKind` carries its subject, and it is
     enforced since A4j (2026-09-18): `subject()` matches without a
     wildcard and `tests/prompt_subject_test.rs` walks games, so a
     wire format has nothing left to enforce there.

     **The payload rule (the owner's review of PR #153, 2026-09-15).** A
     `ChoiceKind` payload names things by id and by CR vocabulary — an
     `ObjectId`, a `PlayerId`, an `AbilityId`, a number, a `Zone`, a
     `CounterType`, a `ManaCost` — and never embeds an engine AST: not an
     `EffectRecipient`, an `ObjectFilter`, an `Effect` or a `Cost` tree.
     Arms are cheap (item 68: appending a variant is O(1)); payload depth is
     what costs, because everything in a payload must be serialized,
     versioned and understood by every client, and an AST value drags its
     whole vocabulary with it. The closure is 36 types only because
     `SelectRecipients` carries an `EffectRecipient`, which pulls in
     `SelectionFilter`, `ObjectFilter` and every subtype enum,
     `CreatureType`'s 303 lines included, and `ChoiceOption::AlternativeCost`
     and `AdditionalCost` carry `Cost` trees the same way. What a client
     needs is not the filter but what the engine already computed from it:
     the options *are* the legality, the subject id says which card is
     asking, and a rendering says why — `subject()` and `describe()` on
     `ChoiceKind`, `backlog.md` §2.21's shape, with the CR rule number as
     the stable handle. **Context is not dropped; it moves from
     engine-private structure to engine-rendered facts.** A GUI opens its
     dialog on the variant, highlights the subject, makes the options
     clickable and reads the text; an agent encodes the variant, the subject
     id, the option ids and the bounds, and learns the semantics from which
     ids get offered, as it learns a card from its id. Anything a client
     wants *structured* about the rule is an oracle query, never a prompt
     field. The events are untouched: a visual effect keys on the performed
     stream — `ZoneChange` with its catchall-free `ZoneChangeCause`,
     `PlayerLost`, `Tapped`, `ManaAdded` — whose own closure is enum- and
     id-shaped by construction, and whose per-viewer projection is
     `backlog.md` §2.9's. The shape: keep the structured payload in-process
     (the decorators read `ManaCost`), add the two rendering methods so the
     engine owns the text, and have the boundary adapter send ids plus that
     rendering; the wire closure then shrinks from 36 types to the ids and a
     handful of enums, and §2.21's gate becomes a mechanical check on the
     two methods.

     **Reachability (2026-09-15):** unreachable — nothing serializes.

     **Shipped 2026-09-18 (A4j), amended in its review:** `subject()`
     matched exhaustively over the 25 variants,
     `tests/prompt_subject_test.rs` walking games plus one fixture per
     variant. Three payloads gained their id and `LegendRule` did not —
     CR 704.5j names no member, `backlog.md` §2.21 has the finding.
     **`describe()` was built and taken out** (the owner, 2026-09-18): a
     rule number riding on a decision is superfluous when the variant is
     the handle, and an engine-rendered line is not the context — the
     variant, its typed fields, the options and the bounds are, and only a
     text client consumes English. So the shape above is amended: the
     boundary adapter sends the variant, the subject id, the option ids
     and the bounds, and each client renders; `ui/cli.rs`'s `prompt_line`
     is that client's, exhaustive. The structured payload stays
     in-process. **Sized:** what remains is `#[derive(Serialize,
     Deserialize)]` on the ids and the vocabulary enums in the wire type,
     ~30 lines, with the harness's adapter (Phase 10); nothing before.

142. **The panic surface, separated into engine and test — the separator is
     `plans/panic_surface.py`.** "341 `.unwrap()` in non-card `src/`"
     (handoff §2) counted the unit tests: `src` holds 59 `#[cfg(test)] mod
     tests` tails, 14,071 of its 60,631 lines. Split at the column-0 marker
     (plus one indented test-only fn in `costs.rs`), 2026-09-15:

     | | engine (44,342 lines) | harness (`src/bin`, 1,642) | unit-test tails (14,071) | `tests/` (32,907) |
     |---|---:|---:|---:|---:|
     | `panic!` | 7 | 0 | 10 | 12 |
     | `unreachable!` | 6 | 0 | 0 | 0 |
     | `.unwrap()` | 8 | 8 | 375 | 785 |
     | `.expect(` | 16 | 6 | 8 | 196 |
     | `assert!`, `assert_eq!`, `assert_ne!` | 23 | 0 | 1,596 | 2,794 |
     | `debug_assert*` (off in release) | 51 | 0 | 0 | 0 |

     **The engine's release-active surface is 60 sites, of four kinds.** 27
     are the provider contract — `ui/ask.rs`'s four `validate_*` helpers
     (13 asserts) and its eight "two or more candidates" caller assertions,
     plus `ScriptedDecisionProvider`'s expectation checks (5 `panic!`, 2
     asserts) — which a misbehaving provider trips and the engine cannot; 2
     are construction-time guards (`performance_pool` on a renamed card, a
     dual land's basic type); the 6 `unreachable!` and the 24 `unwrap` /
     `expect` each sit one line after the check that makes them so
     (`self.battlefield.get(&id).unwrap()` under a membership test,
     `stack.last()` under a non-empty check, the plan cursor under
     `phase_began`), and one `unwrap` is `ui/random.rs`'s own. None is a
     card-reachable panic by inspection, and 800 fuzz games on both pools at
     two and four seats reached none; `fuzz_games` already catches a panic
     per game (`catch_unwind`), so a training batch loses a game and not a
     batch, and `Game::run_turn`'s `Err` covers the engine's own refusals.
     The engine's real invariant surface is the 51 `debug_assert`s, and a
     debug fuzz run is the only thing that exercises them.

     **Reachability (2026-09-15):** nothing owed — a record and its
     instrument; re-run the script at each spine close, and a surface that
     grows faster than the engine is the finding.

     **Sized:** none.

143. **The clone at Commander scale, and memory per fork.** Handoff §4's
     table extended with the same method (2,000 clones a checkpoint, release,
     one thread) plus bytes and allocations per clone from a counting
     allocator; four 100-card decks at 40 life, `stress`, both seeds the
     original table used:

     | seed | turn | objects | on battlefield | events | µs / clone | KB / clone | allocs | µs, no log | KB, no log | allocs, no log |
     |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
     | 12345 | 1 | 400 | 0 | 59 | 4.1 | 52 | 15 | 3.5 | 46 | 14 |
     | 12345 | 40 | 402 | 41 | 1,379 | 21.4 | 227 | 258 | 5.3 | 69 | 36 |
     | 12345 | 80 | 400 | 64 | 3,070 | 62.9 | 440 | 544 | 6.1 | 92 | 36 |
     | 12345 | 113 (over) | 100 | 16 | 5,040 | 124.6 | 786 | 1,662 | 3.8 | 89 | 22 |
     | 777 | 40 | 400 | 28 | 1,259 | 26.9 | 205 | 224 | 4.8 | 61 | 29 |
     | 777 | 88 (over) | 100 | 19 | 3,523 | 87.8 | 563 | 1,114 | 3.3 | 68 | 20 |

     The 60-card rows re-taken the same way, for the join to §4's table:
     four seats at turn 40, 28.0 µs with the log and 4.5 without (67 KB, 35
     allocations); two seats at turn 30, 34.4 and 3.2 (45 KB). **Reading.**
     The board costs a fork about a microsecond — 400 objects and 64
     permanents clone in 6 µs and 92 KB, against 4.5 µs and 67 KB at 60
     cards — and the game's length costs it a hundred: the log is the whole
     of the growth, 700 of 786 KB and 1,600 of 1,662 allocations at the end
     of a 113-turn game, which is item 42 measured. `GameState` is 1,536
     bytes inline; a no-log clone is 14–37 allocations, most of them the
     per-player zone `Vec`s and the object and battlefield maps. For a batch
     of a thousand straight-line games that is 90 MB of state and up to 800
     MB of log; for a search forking at every decision it is the difference
     between 5 µs and 125 µs a branch.

     **Reachability (2026-09-15):** reachable — not wrong; a cost, item 42's.

     **Sized:** item 42's window; nothing else — the no-log clone is already
     at the floor `Arc<CardData>` sets.

What is *not* a ledger line: the 24 asks classified for the fork model are
`backlog.md` §2.22's table, beside the middleware census that will consume
them; item 69 is closed by the Commander-scale measurement and evicted; item
40's table row and item 41's status are corrected in place; and the
profile's recipe is `layers-architecture.md` §12's instrument paragraph and
`engineering-practices.md` §9's readiness pass, not here, because it is a
procedure and not a migration.

### Found by the post-RE audit's pass 4 review — the UUID review (2026-09-16)

**Asked by the owner on PR #154's review: how does `Uuid` get used, why are
v4 and v5 in one engine, and why do ids carry an outsized share of the
profile?** Read off the tree at 6e016d0. The answer is one item, and the
owner decided it the same day.

144. **~~`ObjectId` and `AbilityId` are v4 UUIDs, and the decision (the owner,
     2026-09-16) is to replace both with process-stable ids~~ ✅ CLOSED
     2026-09-16 (A4g, PR #158) — both are `u64` newtypes, the `uuid` crate is
     gone, and the id-keyed maps hash with a seeded one-line mix.** An
     `ObjectId` is stamped in `add_object` from the state's counter, beside
     CR 613.7d's timestamp; a printed `AbilityId` is derived in
     `CardDataBuilder::build` from the card name and the def's ordinal in one
     walk of the card, nested defs included, and the intrinsic site derives
     its own from the object and the land type over the integer. Both open
     questions decided: a granted ability keeps the id its def carries (the
     CR 113.10b test needs "printed + granted the same def = two instances of
     one ability"), and the two ids are newtypes. Callgrind at four seats on
     `stress`: 622.4 M → 380.7 M instructions a game (−38.8%), the `hash_one`
     row gone and `sip.rs` down to the frame's type sets; −37.3% native CPU
     per decision at four seats, −34.0% at two. The hasher is seeded per
     process from `MTGSIM_HASH_SEED`, CI's three runs use three seeds, and
     `fuzz_ab.py` seeds each round. The swap is not `IDENTICAL` to `main` on
     every board: six games in 800 diverge through one board — two Citanul
     Hierophants under one controller, whose two grants now share an id —
     which is item 149, the consequence of the per-definition id this item
     chose, and not the order leak the two-arm design was built to catch (the
     hasher arm is identical to the swap arm on every counter).
     → `plans/archive/codebase-state-closed.md`;
     `layers-architecture.md` §12, the second 2026-09-16 re-read;
     `fuzz-record.md`, the A4g block.

     **Reachability (2026-09-16):** closed — landed; the one answer-visible
     difference is item 149's, recorded with its games and its mechanism.

### Found by A4e — the decision counters (PR #155, 2026-09-16)

145. **~~Three asks still prompt when the answer is forced~~ ✅ FIXED
     2026-09-16 (PR #155, the same one), and CR 102.2 says a forced choice is
     not made.** Found by building item 138's counter,
     which had to decide per primitive what "two or more" means and so
     counted what the engine asks with one legal answer. At `performance`,
     four seats, 200 games: `choose_generic_mana_allocation` 4,738 forced
     prompts of 8,255 — the pool holds one type that can take what the pips
     leave, so the split is arithmetic, not a choice; `ask_select_recipients`
     1,319 of 6,084 — one legal recipient and `min == max == 1`;
     `ask_choose_trample_damage_assignment` 11 of 34. **Thirty a game**, and
     the measurement found no fourth.

     **The shape of the fix is already in the file.** `ask_discard` returns
     the hand unasked when the count takes all of it, `order_scry_group`
     returns below two cards, and five asks assert two or more candidates and
     leave the single case to the caller. These three want the same guard,
     either at the caller or at the top of the ask — about 30 lines.

     **Why it was not deferred** (the owner, on PR #155's review): with
     `decisions` the numerator of a throughput metric, a prompt that produces no
     decision is CPU and a round trip spent against the ratchet itself — so
     "reachable, not wrong" was the wrong classification, and the fix rode in
     the PR that built the instrument rather than waiting behind it.

     **What shipped.** `ui::ask::forced_allocation` answers a split with one
     legal allocation — nothing left over the minimums, nothing spare under the
     maxima, or one bucket free — and the four `allocate` sites take its answer
     instead of prompting; `ask_select_recipients` returns a fixed count that
     takes none of the legal recipients or all of them, in `legal_selections`
     order for `ask_discard`'s stated reason. `validate_allocation` counts a
     decision off the same predicate, so the count cannot drift from what the
     callers skip.

     **It moves the RNG stream, and that is most of its cost.** Item 138's
     lever 10 is right about `pick_n` — a one-option shuffle draws nothing — but
     not about `allocate`: `RandomDecisionProvider::allocate` walks the remainder
     bucket by bucket drawing as it goes, so a forced allocation consumes draws
     and skipping it changes every later game. The consequences, all measured:
     the A/B is a three-arm sitting (`main`, counters-only, the fix) where the
     counters arm is `IDENTICAL` and the fix differs by construction; **40
     scripted tests across 8 files** encoded prompts that no longer happen and
     were migrated with it; and one four-seat `stress` game reaches the 200-turn
     cap where none did before — at `--max-turns 600` the longest is 234 turns
     and nothing hits the cap, so it is a long game and not a hang, RE-9's
     finding in another place.

     **What the migration is worth reading for.** Three of those tests used the
     *prompt count* as their instrument — `phase_rs`'s edict controls, "one
     choice of two, not two choices of one" — and a forced board now reads zero
     either way, so each got one more creature and the claim became provable
     again rather than vacuous. Two more were `should_panic` tests about the
     generic split's clamp (`codebase-state.md` 16c's reproducer): one is now
     the statement that the only payable split is *taken* and never offered, and
     the other needed a third mana so that a choice still exists for the clamp
     to refuse. **A fixture that scripts an answer is a record of what the engine
     asks**, which is why 40 of them moved and why none was deleted.

     **One middleware lost one job to it — the smallest of three.**
     `auto_payer::split_is_forced` is the same predicate `forced_allocation`
     applies one layer down, so `AutoPayer::allocate`'s forced branch is not
     reached from a game. It keeps `OrderCostReductions`, and it keeps the
     thing §3.4 built it for: never taking a split while the pool has surplus,
     which is what leaves `{U}{U}` up for Counterspell. Whether the dead branch
     retired was the census's — **retired 2026-09-18 (A4k, `backlog.md` §2.22
     row 2)**: the branch, `split_is_forced` and the bucket walk are gone with
     six tests, one test in (a forced split reaches the wrapped provider), and
     `AutoPayer` answers `OrderCostReductions` alone — for now: the census's
     review moved that answer into the engine too, as an elision (item 165),
     and the module goes with it; `cost-architecture.md` §3.4's note records
     both. The line
     worth keeping: **a prompt with one legal answer belongs to the engine, not
     to a middleware** — a decorator can only spare a round trip the engine had
     already decided to spend.

     **Reachability (2026-09-16):** closed — fixed.

     **Sized:** ~~~30 lines plus the A/B and the re-read~~ — built: ~60 lines of
     engine, ~340 of test migration, one `fuzz-record.md` block.

### Found by RF — the gather's zone leg (2026-09-16)

146. **The restriction sweep visits the battlefield alone — the gather's zone
     leg has no twin in `engine::restriction::predicate`.**
     `is_prohibited` sweeps `battlefield_ids_ordered` gated on
     `restriction_ability_sources`, which `register_static_effects` fills only
     for `Zone::Battlefield`, so a "can't" printed on a card off the
     battlefield is invisible to it. `layers-architecture.md` §13d decision 4
     named the card: **Abrupt Decay**'s "This spell can't be countered"
     (CR 113.6g), whose *zone* answer is already free from
     `functioning_zones`'s default arm — an instant's abilities function on
     the stack — and whose gate leg is not built. RF built the replacement
     half deliberately alone (`replacement-architecture.md` §9, Phase RF
     decision 2): the two sweeps read different ability bodies, and the
     summary's restriction legs stayed bools for that reason.

     **Reachability (2026-09-16):** unreachable — no registered card prints
     a restriction that functions off the battlefield. RS-2
     (`cant-effects-architecture.md`), where casting and countering
     restrictions land, is the natural home.

     **Sized:** RF's shape exactly — a `zone_restriction_ability_sources` set
     filled at the same three doors and retired at `cleanup_zone_state`, a leg
     in `is_prohibited` after the battlefield loop reading the effective list
     with `functions_in`, and `set_affects` already asks the zone; ~80 lines
     plus Abrupt Decay and a counter-it test. One difference from the
     gather's leg to keep: a spell on the stack asks its *own* "can't be
     countered", so the restriction leg must not skip a stack source the way
     the gather's skips the entering object — being countered is not
     entering.

147. **`hollow_hands` owes the tests that show a strip in hand turning off
     what functions there, and none of that exists yet.** The fixture
     ("Cards in hands lose all abilities", `phase_rf_cards.rs`) proves today
     that the gather's zone leg reads the effective list — a Colossus in hand
     under it is discarded like any card. What it will *also* have to turn
     off is every ability that functions from a hand: cycling and channel
     (CR 113.6j, activated from hand — `layers-architecture.md` §13d decision
     4's 113.6j row), and the evoke and madness families once `backlog.md`
     §2.3 lets a card be cast from anywhere but a hand's ordinary door.
     Raised at RF's review (2026-09-16).

     **Reachability (2026-09-16):** nothing owed by the engine — no
     hand-functioning ability exists to be stripped. A record for the keyword
     that lands first.

     **Sized:** one test per keyword, ~30 lines each, in that keyword's own
     phase file, with this fixture as the negative: the ability is usable
     without Hollow Hands and not with it.

148. **Exile is the one zone the Colossus family's tests do not move
     *from*, and the reason is a missing cause.** The clause functions in
     exile (`ZoneSet::ALL`; `create_in_zone` and `arrive_in_zone` register
     there, and `test_the_candidate_set_follows_the_card_from_zone_to_zone`
     asserts the map holds an exiled Colossus), but no registered card moves
     a card from exile into a graveyard and no `ZoneChangeCause` names that
     move. Pull from Eternity — "put target face-up exiled card into its
     owner's graveyard" — is the card, and the no-catchall rule makes it
     bring its cause. Raised at RF's review (2026-09-16).

     **Reachability (2026-09-16):** unreachable — no producer.

     **Sized:** the card, its cause, and one test: a Colossus in exile under
     Pull from Eternity is shuffled into its owner's library instead. Phase 8.

### Found by A4g — process-stable ids (PR #158, 2026-09-16)

149. **~~Two instances of one ability on one object are indistinguishable by
     id, and the mana window lists them once.~~ — ✅ CLOSED 2026-09-19 (TR-1).** — archived.
     `AbilityIdentity` gained `zone_change_epoch` and `instance` — the ordinal
     among same-id defs in effective-list order — as `triggers-architecture.md`
     §3.6 decided; `activate_ability` and the dispatcher both fill them. The
     `(ObjectId, AbilityId)` sites stayed a pair: they are the mana window's
     keys, and §3.6 says why a mana ability's two instances need no telling
     apart (`archive/triggers-architecture-landed.md`, TR-1 note 8).
     **Reachability (2026-09-19):** closed — TR-1.
     Full entry: `plans/archive/codebase-state-closed.md`, "Item 149".

### Found by A4h — item 41's fork test (2026-09-16)

150. **~~`activatable_abilities` never checked whether the ability had a legal
     target~~ ✅ CLOSED 2026-09-16 (A4h, the same PR) — it does now, the way
     `castable_spells` always has.** The enumeration checked costs and CR
     602.5d's timing and stopped, so an Equipment's equip ability was offered
     on a creatureless board, rejected by `activate_ability`'s CR 601.2c check,
     blacklisted — and the next enumeration offered it right back. CR 602.2b
     routes an activation through 601.2b–i, so 601.2c applies to an ability
     exactly as it does to a spell; an empty legal-choice set is provably
     illegal from a static read, which is what the oracle may filter on
     (`dp-middleware-and-candidate-enumeration.md` §2). `UpTo` is left in the
     list, because choosing zero targets is legal.

     **How it was found, and why it is recorded rather than folded into 139.**
     It is not a rules bug either — the games played were legal, the engine
     rejected what it could not do. It is the *second* way the offered list
     stopped being a function of the board, and item 139's fix could not reach
     it: with the enumeration fresh and this gap open, 52 of 15,646 forked
     branches still failed to replay. The blacklist was deciding the prompt,
     which is item 40's violation with a different owner. Two cards in the
     pools reach it, Bonesplitter and Cobbled Wings, and both are Equipment;
     the cost of leaving it was a re-ask loop that could only end at the retry
     budget, which is the diagnostic `engine/priority.rs` has printed since it
     was written.

     **Reachability (2026-09-16):** closed — landed; and the reason the fork
     test reads 0, which item 140 inherits.

     **The budget it spent:** the target check runs after the cost check for a
     reason — before it, the shipped arm read +7.2% µs/decision at two seats
     and +5.7% after, because `has_any_legal_choice` walks the battlefield and
     most abilities are unaffordable anyway. `fuzz-record.md`, the A4h block.

### Found by A4b — the rulings ledger (2026-09-17)

151. **The rulings gate is scoped, so 308 of the ledger's 330 rulings are
     outside it by construction.** `plans/check_rulings.py --check` fails on a
     ruling that no test names and no disposition answers — but only for a card
     that carries a `read` stamp, or whose `first_seen` is later than the
     ledger's `created`. Everything else is the backlog, which is 308 rulings
     over 93 cards today, 103 of them on 38 `PERFORMANCE_POOL` cards.

     **Why it is scoped rather than armed.** A gate that failed on all 330 the
     day it landed is a gate nobody could pass, so it would have been turned
     off or worked around within a PR or two; `specdb owed` has the same shape
     for the same reason (`engineering-practices.md` §5.1). The forward half
     needs no scoping and has none: a card registered from here on is in scope
     the moment `--fetch` sees it, which is what makes §3.4 a rule rather than
     a habit.

     **Reachability (2026-09-17):** reachable but not wrong today — nothing in
     the engine is wrong because of this; what it means is that **the pool
     carries 103 unread claims about its own behaviour**, each one written by
     the people who adjudicate the game, and item 82 is the standing evidence
     that reading them finds live bugs. The debt is unread rulings, not
     unimplemented rules.

     **And the scope rule froze it.** A card registered from here on has a
     `first_seen` later than the ledger's `created`, so it is in scope from
     the moment `--fetch` sees it and its rulings are answered in the pull
     request that registers it. **Nothing new joins the backlog** — the 308 is
     exactly the rulings on the 93 cards that predate the ledger, and it only
     shrinks, apart from drift adding one to a card nobody has read yet. That
     weakens the urgency `engineering-practices.md` §3.4a was written with
     ("the last point at which the retroactive half is a sitting rather than a
     project, since the registry only grows"): the registry still only grows,
     but the backlog no longer grows with it.

     **Sized:** a sitting per tranche, not a phase. A4b's own head — three
     pooled cards, 22 rulings — came to ten new tests, five annotations on
     tests that already existed and nine dispositions. At 22 rulings a sitting
     the remaining **pooled 103 is about five** and the whole 308 is about
     fourteen, so the off-pool 205 is two thirds of the work on cards no
     measurement walks. **Scheduled by the owner 2026-09-17** (`roadmap-v2.md`
     row A4b): the pooled 103 takes a slot between phases whenever one is
     free, no deadline, and the off-pool 205 is not scheduled at all. Between
     phases rather than inside one because a bug this finds in a pooled card
     is an engine fix that moves the random agent's stream and owes its own
     A/B — the argument §3.4a used for A4b's own slot, and the reason the
     off-pool half can wait indefinitely: a card no measurement walks cannot
     make a measurement wrong. `python plans/check_rulings.py --queue` is the
     list; it is deliberately not copied into this file, because a list that
     is both generated and transcribed goes stale in the transcription.

### Found by A4i — several instances of "target" (2026-09-17)

Trace page: `plans/traces/a4i-a-target-belongs-to-an-instance.html` — three
boards read by read, written at the review because two of its questions (why a
resolution needs CR 601.2c's instance cursor, and how "another target" stacks)
are ones the diff cannot answer.

**The review's nine themes, and where each went.** The handoff file was deleted
with A4n (2026-09-18) under its own eviction contract, so this is the index.
**A** (names), **D** (the n−1 coverage boundary), **F** (`check_string_literals.py`)
and **G** (the trace page above) closed inside A4i's own PR — cc1b4d9, c731e55,
22a7d6f. **B** is items 157 and 158, with item 154 amended. **C** landed three of
four items (3ea4cb5; `fuzz-record.md`'s A4i block) and **withdrew the fourth after
building it** — `targeting.rs`'s `FilterIdentity` carries why, because that is
where someone would try it again. **E** is `backlog.md` §2.33. **H** and **I.2**
are `roadmap-v2.md` row A4n, which carries the measurement and the three riders —
closed 2026-09-18, with the third rider split out as row A4q.
**I.1** is items 159 and 160, closed by A4o and A4p. **I.3** amended item 155 in
place.

152. **~~Skullcrack's three damage never landed when the card was cast.~~**
     **Fixed by the instance walk, 2026-09-17.** *"Players can't gain life this
     turn. Damage can't be prevented this turn. Skullcrack deals 3 damage to
     target player or planeswalker."* Three atoms, the first two
     `EffectRecipient::Controller`, and the pre-A4i rule took a `Sequence`'s
     **first** atom's recipient as the whole spell's — so the spell announced
     no target, `chosen_targets` stayed empty, and the damage atom read an
     empty list. Cast from hand, Skullcrack did its two restrictions and
     nothing else.

     **Reachability (2026-09-18):** closed — fixed 2026-09-17 and covered by
     `phase_a4i_integration_test.rs`. Dated here because the strike-through
     alone sat too far into the item for `check_state_of_play.py` to read, so
     the board counted a closed item as a claim with no verdict.

     **How it survived RE-3 and three phases after it.** Every Skullcrack test
     stages a `ResolutionContext` with the target written in by hand, which
     proves the resolution reads a target and says nothing about whether
     CR 601.2c ever asked for one. A fixture that writes the answer cannot
     check the question. The regression
     (`skullcrack_cast_from_hand_deals_its_three_damage`) casts from hand for
     exactly that reason, and was shown to fail at `main` (aafb79a) first.

     **How it was found:** A4i's A/B. `performance` came out `IDENTICAL` and
     `stress` did not, on the same 161-card pool — so a registered, unpooled
     card was playing differently. The probe that named it compared the old
     first-atom derivation against `effect_instances` for every registered
     card; Skullcrack was the only pre-A4i card where they disagreed.

     **Reachability (2026-09-17):** was reachable and wrong; closed. The class
     is not: any card whose targeting atom is not the first in its sequence had
     the same defect, and the probe says Skullcrack was the only one registered.

153. **`effect_instances` walks `Atom` and `Sequence` and nothing else, so a
     modal spell would announce the targets of modes nobody chose.**
     CR 601.2b chooses modes *before* CR 601.2c announces targets, so a walk
     that descended into every branch of `Effect::Modal` would declare an
     instance per mode and ask for all of them. The walk therefore stops at the
     nodes it understands — the same scope the one-recipient rule it replaced
     had.

     **Reachability (2026-09-17):** unreachable. `Effect::Modal`,
     `Conditional`, `Optional`, `ForEach` and `Repeat` all resolve to an error
     today (`resolve_effect`), so no card can carry one and be played. It bites
     the moment modal spells land — `backlog.md` §2.7 — and that phase owns it.

     **Sized:** small, and it is a design choice rather than a sweep. The walk
     takes the chosen modes as an argument and descends only into those, which
     means `effect_instances` grows a second spelling for the post-601.2b call
     and `StackEntry.chosen_modes` becomes an input to it rather than a record.
     Under fifty lines; the cost is deciding where the two spellings live.

     **Amended 2026-09-18 (A4n):** the walk is `Effect::instances` now, run
     once per card by `CardDataBuilder::build` and stored on every def
     (`AbilityDef::instances`) and on the card (`CardData::spell_instances`);
     the engine reads the stored lists and never walks at cast time. That
     settles where the second spelling lives: a modal spell's post-601.2b list
     depends on the modes chosen at that cast, so it cannot be a stored
     constant, and the mode-aware walk runs at cast time from
     `StackEntry.chosen_modes` — once per cast, which is the rate the stored
     list was built to take castability *off*. The stored list stays the
     pre-601.2b one, and today a `Modal` node declares nothing in it.

154. **`ObjectFilter::OtherThanInstance` is only asked of a `Permanent`
     filter, so "another target" over `SelectionFilter::Any` or `Player` is
     not expressible.** The leaf lives inside an `ObjectFilter`, and
     `SelectionFilter::Any`, `Player`, `Spell` and `DamageSource` carry none;
     `enumerate_legal_selections` also pushes every player unconditionally for
     `Any` and `Player` without asking `validate_selection`, so a player
     exclusion would be skipped even if the leaf could be written.

     **Reachability (2026-09-17):** unreachable — no registered card writes
     "another target" over anything but a permanent filter. CR 115.4 puts the
     phrase over "any target" on real cards, so a printed one exists; it is
     Phase 8 breadth's, and the fix is a `SelectionFilter`-level exclusion
     rather than a wider `ObjectFilter`.

     **Amended at the A4i review (2026-09-17): it has eight named customers.**
     `o:"must target"` returns 8, and every one is *"Each mode must target a
     different player"* — a player-filter exclusion, which is exactly what this
     item says cannot be written. They arrive with modal spells; item 158 has
     the list and the two sibling axes.

     **Sized:** ~60 lines. The exclusion moves from the filter to the clause —
     the instance carries "not what instance k took" beside its filter — and
     the two enumeration arms that bypass `validate_selection` learn to apply
     it. That is the shape CR 601.2c actually describes, so it is a
     simplification as well as a widening; it was not done here because it
     touches `EffectRecipient`, which 146 sites construct.

155. **`every_instance_has_a_choice` feeds instances forward greedily, which
     is exact for the shapes that print and not in general.** An "another
     target" chain reuses one filter, so counting the candidates that pass it
     and subtracting the ones already taken is the answer a bipartite matching
     would give. A card whose instances carried *different* filters **and**
     excluded each other could be told it cannot be cast when a different
     assignment would work.

     **Reachability (2026-09-17):** unreachable — no printed card combines the
     two, and the failure mode is conservative (a cast refused, never an
     illegal one allowed). If one prints, the fix is Hopcroft–Karp over at most
     a handful of instances, which is small but is a different kind of code
     from what is there.

     **Sharpened at the audit (2026-09-17): the condition is monotone widening,
     not one filter.** Greedy is exact as long as no later clause is *narrower*
     than an earlier clause it excludes. The shape that fails is "target
     creature" followed by "another target creature you control" on a board
     where the caster's only creature is first in timestamp order: greedy takes
     it for the first clause and finds nothing for the second, when swapping
     would work. `o:/target creature[^.]*another target creature you control/`
     returns one card, Combine Guildmage, and it reuses one filter — so the
     reachability line above stands. This is the sentence a card author checks
     a new "another target" card against.

156. **The rulings gate has a same-day blind spot: a card registered on the
     day the ledger was created escapes `--check` unless someone stamps it
     `read`.** Scope is `read` or `first_seen != created`
     (`check_rulings.py::in_scope`), and A4i registered five cards on
     2026-09-17, the day A4b's ledger was created — so all five, and the three
     rulings between them, were out of scope until the PR hand-stamped them.
     The stamp is honest (the rulings *were* read), and the gate then refused
     all three until a test named each.

     **Reachability (2026-09-17):** reachable exactly once, and it already
     happened. `created` never moves again, so no future registration can
     collide with it. Recorded rather than fixed because `first_seen ==
     created` is genuinely ambiguous — every card present at creation carries
     it — and a finer stamp would be a ledger format change to close a hole
     that cannot recur.

157. **CR 601.2c's "must be chosen as a target" is unimplemented, and it is the
     third hat of the CR's only combinatorial-optimum rule.**

     > If any effects say that an object or player must be chosen as a target,
     > the player chooses targets so that they obey the **maximum possible
     > number** of such effects without violating any rules or effects that say
     > that an object or player can't be chosen as a target.

     **The whole CR has three sites for that phrase**, surveyed 2026-09-17:
     **508.1d** (attack requirements), **509.1c** (block requirements) and
     **601.2c** (targeting requirements). There is no fourth. The first two are
     RS-3b's, already sized as the NP-hard one with a bounded-exact search and a
     cap (`cant-effects-architecture.md` §4.2); this is the third and it is
     unowned.

     **It is much the smallest of the three.** Combat searches every creature a
     player controls crossed with attack/block assignments; targeting searches
     one spell's instances — one to four on every printed card — crossed with
     each instance's legal candidates. Same algorithm, and RS-3b's cap covers it.

     **Not to be confused with "as many as possible"** (CR 101.3, 701.17b,
     701.23d, the 601.2h discard example), which is a *clamp* rather than an
     optimization and is already implemented — `Primitive::Mill`,
     `sacrifice_of_choice`'s `count.min(candidates.len())`. Nothing in that
     family needs a search.

     **Reachability (2026-09-17): unreachable, and no printed card produces
     one.** `o:"must be chosen"` and `o:"chosen as a target"` return nothing;
     `o:/target.{0,20}if able/` returns four and all four are *combat*
     requirements (Dulcet Sirens, Hunt Down, Ravener, Rimehorn Aurochs), which
     are 508.1d/509.1c's. So this is a CR-stated facility with no printed
     producer, which by the RE-5 rule (`engineering-practices.md` §4) is owed
     with a fixture test and this line, never "nothing owed".

     **Sized:** small, and it belongs with RS-3b rather than alone — the search,
     the cap and the "requirements versus restrictions" split are the same code.
     What is A4i-specific is that the announcement loop has to hand the solver
     all the instances at once instead of deciding them one at a time, which is
     ~40 lines at the `announce_targets` seam. Until then the greedy loop is
     exact, because the set of requirements is empty.

158. **"Different from that one" has three axes and the engine expresses one.**
     A4i's `ObjectFilter::OtherThanInstance` is distinctness *across instances of
     one object* — Incremental Growth's "another target creature". Two more
     populations want the same idea on different axes and neither is reachable:

     - **Across modes of one object**, 8 cards: *"Each mode must target a
       different player"* — Balor, Chaos Balor, Casey & Raph, Donnie & April,
       Mikey & Mona and kin. This is item 154's gap (the exclusion is an
       `ObjectFilter` leaf and `SelectionFilter::Player` carries none) plus modal
       spells (`backlog.md` §2.7). **Item 154 said this gap had no named
       customer; it has these eight.**
     - **Across separate stack objects**, 13 cards: *"Each copy targets a
       different one of those creatures"* — Precursor Golem, Zada, Ink-Treader
       Nephilim, Mirrorwing Dragon, Radiate, Agrus Kos, Beamsplitter Mage,
       Exterminator Magmarch, Feather, Frontline Heroism, Ivy, Radiant Performer,
       Zevlor. The constraint is written by the card as the copies are created,
       not by CR 601.2c as one spell is announced, so it is
       `copy-effects-architecture.md`'s and not this one's.

     **Reachability (2026-09-17):** unreachable — none of the 21 is registered,
     and both axes need a facility that does not exist yet (modes, spell
     copying). Recorded together because the three axes are one idea, and sizing
     any of them alone would miss that the expression they want is shared.

     **Sized:** the modal axis is item 154's ~60 lines (move the exclusion from
     the `ObjectFilter` leaf to the clause, so a `Player` or `Any` filter can
     carry it) plus §2.7's mode work. The copy axis is CV's and is sized there.

159. **~~`SelectionFilter::Spell` accepts an activated ability on the stack,
     and `Primitive::CounterSpell` then puts the ephemeral ability object into
     a graveyard as a card~~ ✅ CLOSED 2026-09-18 (A4o, PR #163) — the filter
     asks `is_spell_on_stack`, at all three sites.** CR 112.1's "a spell is a
     card on the stack", asked of the `StackEntry` rather than of stack
     membership, by the validator, the count arm and the enumeration arm; the
     sibling `DamageSource` arms, which already carried three inline copies of
     the same predicate, call the one function now. **It was live in the
     measured games:** `main` countered 26 ability objects into graveyards
     across the four A/B arms — Chainbreaker, Bonesplitter, Merfolk
     Thaumaturgist, Samite Healer, Mind Stone, Words of Worship, Deep Water,
     Circle of Protection: Red, Aggravated Assault — and the fixed arm none.
     **A6 inherits the fix and should keep the shape it rests on:** a
     triggered ability on the stack will be the same ephemeral object with
     `is_spell: false`, so "counter target spell" cannot name one as long as
     that stays true, and the complement filter Stifle's class wants
     (`Primitive::CounterAbility`, built, no registered card) negates exactly
     this predicate.
     → `plans/archive/codebase-state-closed.md`;
     `fuzz-record.md`, the A4o block.

     **Reachability (2026-09-18):** closed — landed; the stream moved, and
     every diverging game in all four A/B arms is one where this filter's
     answer changed.

160. **~~The `Player` and `Any` selection arms count and offer seats that
     have left the game~~ ✅ CLOSED 2026-09-18 (A4p, PR #164) — the player
     iterator and both count arms ask `in_game`, and `validate_any_target`
     asks it too.** CR 800.4a, at the three sites the item named: the
     enumeration's `players()` closure, which feeds the `Player` and `Any`
     arms alike; `has_legal_choices`' two arms, which now count seats rather
     than the vector; and the "any target" validator, which had no CR 800.4a
     check at all while its `Player` sibling had one — so that sibling's
     comment, "not offered at CR 601.2c", is true now rather than aspirational.
     **It was live in the measured games:** at four seats `main` resolved 11
     Lightning Bolts against a player who had left, over 11 games of the 400
     the A/B dumped, and the fixed arm none.
     → `plans/archive/codebase-state-closed.md`;
     `fuzz-record.md`, the A4p block.

     **Reachability (2026-09-18):** closed — landed; the stream moved at four
     seats on both pools and at neither pool at two, and every diverging game
     is one where the offer list changed.

### Found by A4k — the middleware census (2026-09-18)

**The census is `backlog.md` §2.22**, rewritten this day: three rules, nine
rows sized and sequenced, five things named as not rows. What is recorded here
is the debt — one line per middleware the census names and defers, so the
target system's prerequisites are where this section's readers look for them.
Items 72 (CR 732.1's reversal), 84 (the two combat helpers) and 145 (the
payer's dead branch) are updated in place; the one code change, the payer's
forced branch retired, is recorded on item 145. **Amended the same day at the
owner's review of PR #169**, and the items below carry the amended shape: an
answer that cannot change the game is the engine's to elide, so the payer's
remaining prompt moves into the engine (item 165) and no decorator is
"answer-preserving"; full control is a switch above the stack, not a handle in
each decorator (item 161); the solver's preference is the client's in the
client's own shape, and the solver owns both payment prompts (item 162); the
reversal is two options, keep or reverse all, by a clone (items 72, 163). One
thing the census read on the way and did not fix, because the ticket touched
no engine file but the payer: `ui/ask.rs`'s module doc, `forced_allocation`'s
doc and item 145 cite "CR 102.2" for "a forced choice is not made", and in
`tmnt.txt` 102.2 is the two-player-opponent rule — the CR states the general
form nowhere, and the anchors the tree actually rests on are CR 616.1's "two
or more" and 601.2f's "if multiple". A comment fix, next time a hand is in
the file.

161. **Full control and auto-yield — sized, sequenced, not built.** Full
     control is the raw provider entered and left mid-game: a
     `FullControl<D, R>` at the top of the seat holding the decorated stack and
     the raw provider, forwarding each of the four methods to one or the other
     on a `Cell<bool>`, plus a `CliDecisionProvider` command intercepted before
     an index is parsed. The decorators stay stateless and know nothing of it.
     Auto-yield is `AutoYield<D>`, answering `PriorityAction` with `Pass` while
     one of three yield conditions holds, read off the `&GameState` every
     prompt carries. **Neither ships without the other** (§2.22 rows 3 and 4):
     auto-yield makes the tell — a fast-forwarded turn says the player holds
     nothing at instant speed — and the switch is what puts the prompts back.
     Human seats only; a bot's non-forced pass is its agent's decision. The
     switch's position and the yield command must ride in the recorded input
     stream or a CLI game stops replaying. A GUI provider with state wants
     `Rc<P>` and a forwarding impl so the raw side and the decorated side are
     one provider.

     **Reachability (2026-09-18):** unreachable — a facility that does not
     exist; nothing wrong today, since nothing auto-passes.

     **Sized:** one PR, ~250–350 lines with tests — the switch ~100 with the
     command and wiring (2026-09-08's ~200 was the handle shape, re-derived at
     review), auto-yield ~100–150. A/B `IDENTICAL` by construction:
     `fuzz_games` stacks neither. Any time, before the GUI; §2.22's sequence
     step 3.

162. **The tap solver's two halves — the matching and its two customers.**
     §2.18's oracle half is a bipartite matching from the pips
     `remaining_cost_after_pool` still owes to the mana abilities
     `available_mana_sources` offers, in `oracle/`; the decorator picks in CR
     601.2g's window while the component is uncovered, disjoint from
     `ManaWindowStop`'s predicate, **and answers `GenericManaAllocation` when
     the pool has surplus**, since one preference decides both. The second
     customer is `castable_spells`' affordability, a heuristic
     overapproximation today (`find_mana_sources`), whose over-offers are the
     CR 732.1 rewinds A4h made the retry loop state-dependent for (item 139)
     and the enumeration lever 4 prices at 19.2% (item 138). **The Arena
     problem is the preference, not the matching, and the preference is the
     client's in the client's own shape**: the framework owes possibility —
     any client-side intent maps onto the matching's (source, type) edges, so
     a per-type spend-or-keep setting a GUI might draw does — and sets no
     default from an example; a seat that supplies none gets the random
     agent's measured least-flexible-first policy.

     **Reachability (2026-09-18):** reachable — not wrong; a cost. 194 window
     prompts a game at four seats, 60% of inner prompts, and the rewinds the
     harness does not count.

     **Sized:** the matching ~150–250 lines with tests, its own oracle PR any
     time — **and that number is the plain bipartite case**, the one row with
     algorithmic legwork (§2.22, rows 6 and 7): amounts as capacities, since
     `available_mana_sources` drops them today and `find_mana_sources`
     refuses hybrid, so a flow rather than a matching; mana-spending filter
     chains out of scope and left to the window; a property test that every
     covering set is one `ManaPool::pay` accepts (16c's class); re-derive
     against a written algorithm before scheduling. The affordability customer moves the random agent's stream and owes
     an A/B with `differ` predicted on both pools, and a rewind counter first
     since `fuzz_games` prints none; the decorator ~60 lines after item 161,
     human under the toggle, a harness flag off by default, read in its A/B as
     `Decisions` falling with the engine no faster. §2.22 rows 6 and 7.

163. **CR 603.3b's ordering prompt, classified before it exists — and the
     reversal's shape settled beside it.** In §2.22's fork-model table the
     ordering is a **C** row and part of the residual: asked of each trigger's
     controller in APNAP order, mid-step, of a seat that did not act. Two
     halves. The engine's, by §2.22's rule 1: two triggers that are copies of
     one ability under one controller with no targets give the same game in
     either order, so the engine declines to ask — measured first, then elided
     with expiry conditions, item 47's precedent for CR 616.1's prompt. The
     decorator's, for the rest: timestamp order for a human under the toggle,
     the agent's own for a bot. **Item 72's reversal (CR 732.1), decided
     2026-09-18 and re-derived at review: two options, keep or reverse all.**
     *Reverse all* is a no-log clone taken at the window's first activation
     and restored at the rewind with `events` truncated to its length — CR
     732.1's "no abilities trigger and no effects apply" by construction, the
     RNG rewound with it, the retry loop's locals and `Diagnostics` kept live.
     *Reverse some* is not a facility the chokepoint has: performed events
     with replacements applied have no per-event undo, and Arena offers
     undo-all only. A decorator may answer the offer only under the same
     toggle as auto-yield — reverse all when the solver made the taps, on a
     human's seat; a plain policy on a bot's (`fuzz_games`: keep, today's
     stream); never silently on a human's. It lives inside item 162's
     decorator, since the taps a solver made are the ones it should unmake.

     **Built in part 2026-09-19 (TR-1):** the ordering half. `ChoiceKind::OrderTriggers
     { player, tier }` is asked through `choose_ordering` with two or more
     entries, and `trigger_order_cannot_change_outcome` elides it with the expiry
     conditions written beside the predicate — a binding that differs, an
     instance of "target", a tier-2 entry — each of which has a test
     (`two_identical_triggers_are_placed_without_an_ordering_prompt` and the two
     that reopen it). The reversal half is untouched.

     **Reachability (2026-09-19):** reachable — not wrong: the ordering prompt exists
     and is asked or elided per the predicate; the reversal prompt does not exist,
     and CR 732.1's option is simply never offered.

     **Sized:** the ordering decorator ~40; the reversal prompt ~40 in the engine
     (the clone, the restore, the truncation) and its decorator arm ~15; §2.22
     rows 8 and 9.

164. **The `[Pass]`-only priority prompt is still asked of the provider.**
     `candidate_priority_actions` always offers `Pass`, and 91.5% of priority
     prompts at four seats offer nothing else (item 138) — over two thousand
     round trips a game for an answer the engine has, item 145's class exactly:
     a prompt with one legal answer belongs to the engine (§2.22's rule 1),
     and out of process it is CPU and a round trip spent against the ratchet's
     numerator without a decision to count. Not middleware: a decorator can
     only spare a round trip the engine had already decided to spend.

     **Reachability (2026-09-18):** reachable — not wrong; a cost. Every game,
     every seat.

     **Sized:** ~10 lines at `run_priority_round`, taking `Pass` without
     asking when the list is `[Pass]` alone. No counter moves —
     `Priority decisions` already excludes the one-option prompt — and the
     random agent's stream does not either, since a one-option `pick_n` draws
     nothing (item 145, lever 10); so the A/B prediction is `IDENTICAL` and
     the whole cost is the fixture migration: **148 scripted
     `ChoiceKind::PriorityAction` expectations, 134 in ten test files and 14
     in `src` unit tests**, an upper bound because some answer a longer list.
     Size the migration by running it before scheduling; if most of the 148
     are `[Pass]` answers this is a stream-preserving PR of the kind item 145's
     was not, and cheap.

165. **CR 601.2f's ordering prompt is answered by a decorator where the engine
     should not ask.** `AutoPayer` answers `OrderCostReductions` with gather
     order because, by `cost-architecture.md` §3.4's theorem, every order gives
     the identical total — which makes the prompt §2.22's rule 1, second case:
     an answer that cannot change the game is the engine's to elide, as item
     47's `pipeline::ordering_cannot_change_outcome` already does for CR
     616.1's. §3.4 kept the prompt "until it shows in a profile"; the owner's
     review of the census (2026-09-18) read that as a timing call and struck
     the "answer-preserving" decorator kind, so the elision is owed and the
     module goes with it.

     **Reachability (2026-09-18):** reachable — not wrong; a cost, and a
     second home for one fact. 0 / 0.07 / 0.02 prompts a game.

     **Sized:** ~20 lines — one guard at the call site in
     `cost_determination/total.rs`, false the day either expiry condition
     lands (a reduction whose amount is a hybrid symbol, CR 118.7e; a
     `not_below` reduction), the two conditions §3.4 already names;
     ATOM-601.2f-004's test restated from "the prompt is asked" to "both
     reductions apply and every order gives the CR's answer"; `ui/auto_payer.rs`
     deleted and `cli_play`'s human stack `ManaWindowStop(Cli)`. It moves the
     random agent's stream by the prompts it removes, so a `differ` A/B and its
     own PR — §2.22's sequence step 2, first after this one.

- Every new forward-looking stub, TODO, or half-wired abstraction gets a line here at commit time — unless its fix is under about thirty lines with a fixture, in which case it is fixed instead; the rule is at the head of this section, "What does not belong here".
- When a migration is completed, strike the line (keep it visible in history for a few revisions, then remove).
- Migrations that are substantial enough to warrant ticketing get a link from here to their ticket; tiny migrations are just done inline.

### Found by A4c — the trace sink (2026-09-18)

**The sink is `mtgsim/src/state/trace.rs`; the record is its module doc and
`tests/trace_sink_test.rs`; what the row did not predict is on `roadmap-v2.md`
row A4c.** One thing is owed, by decision 6:

166. **The two-version trace diff — the artifact item 5 called higher-value,
     shaped for and not built.** One board through two engine builds, compared
     record by record: what `plans/fuzz_ab.py` does for counters and a human
     cannot do by hand. Out of A4c because nothing it would compare exists
     twice yet; the format is ready for it — one record per line,
     process-stable ids (A4g), `seq` and `branch` as the join keys, and a
     `game` header carrying the seed and the build's commit (`stamp_commit.rs`, so a
     binary copied aside for a sitting says what it was built from). What the
     format does not settle is the alignment: once one arm writes a record the
     other does not (a `layer_walk` the memo answered on one side), `seq`
     drifts, and the diff has to pair `batch` records by their proposals the
     way `diff` pairs lines — which is the whole of the tool.

     **Reachability (2026-09-18):** nothing owed to correctness — tooling.

     **Sized:** ~150 lines of Python beside `plans/trace_spine.py`, aligning on
     `batch` records and reporting the first `pipeline` or `decision` that
     differs; its first customer is the next stream-moving PR's A/B, which
     today attributes a divergence by hand (`fuzz-record.md`, A4h's block).

### Found by TR-1 — the trigger spine (2026-09-19)

**Shipped:** `types/triggers.rs`, `engine/triggers/{dispatch,placement,binding}.rs`,
the five cards, fifty tests, and the record in `fuzz-record.md`. What follows
is what the building left behind, one item each; `archive/triggers-architecture-landed.md`
has the notes that are not items.

**The matcher was reviewed and rebuilt (2026-09-22, the TR-1 review, theme
C).** Four things, and only one of them changed an answer. **F1**: the
dispatcher applied CR 113.6 per *object* — `register_static_effects` files
only the ability ids that function in the object's zone, and `find_matches`
read the map's keys and then asked the whole effective list, so a card in a
graveyard for its "from anywhere" half was also asked its battlefield-only
half. `functions_in` is now asked per def, on every leg, as
`replacement::gather` has always asked it. Unreachable on the pools and
wrong the day TR-4 registers Bloodghast or Ichorid. **#16**: the records x
candidates x abilities loop is two loops over a pre-pass that answers CR
113.6, the instance ordinal and the identity once per def. **§11's lever**:
`trigger_sources` is an `IdMap<ObjectId, EventKindMask>` and the gate asks
whether any source reads a kind this window carries, which took candidate
visits per game from 297 to 30 on the shipped pool and from 10,131 to 707
on a board with eight copies each of the three; `TriggerEvent::reads` is
written from the mask's table rather than beside it, because two tables
that had to agree is what that lever would otherwise have shipped with.
**#38**: `fuzz_games` gained `--copies N` so a mechanic-heavy board is
re-takable without a throwaway build, and the pooled-cards game asserts CR
117.5 at every priority prompt on a deck that can actually cast the three.

167. **A look-back arm on a *surviving* permanent reads its post-event
     ability list.** CR 603.10 looks back "using the existence of those
     abilities ... immediately prior to the event", and the dispatcher does so
     off the CR 603.10a frame for a permanent that *left* (§4.2 leg 2). A
     permanent that stays reads the list it has after the window closed: a
     Blood Artist surviving the wipe that took Humility triggers on the
     deaths, where before the event it had no abilities and should not. The
     frames a record carries cannot answer this — nothing about a survivor is
     recorded — and the memo's stale entry is a cache, not a record
     (`triggers-architecture.md` §15 item 4).

     **Reachability (2026-09-19):** reachable — wrong today: Humility and Blood
     Artist are both pooled, and one state-based check that kills Humility
     and a creature while Blood Artist lives is the board. Rare, and the
     answer is one extra trigger.

     **Sized:** ~60 lines: when the outermost batch is a battlefield departure
     and `trigger_sources` is non-empty, snapshot each source's look-back defs
     (an `Arc` clone per source) at the batch's open and match those; or TR-4's
     `LastKnownInformation` grows a per-window frame for every source the
     batch touched. Which is the review's, with its cost measured.

168. **A frame candidate's identity carries the post-move epoch, or 0 for an
     object that left the game.** `AbilityIdentity.zone_change_epoch` is read
     off the store at dispatch; a departed object's *old* existence is what
     triggered, and the frame does not carry its epoch.

     **Reachability (2026-09-19):** unreachable — nothing reads the identity's
     epoch yet; CR 603.7h's counter (TR-2) and the gates (TR-2) are its
     readers, and neither keys a dies-trigger's source.

     **Sized:** the frame gains the epoch when it becomes
     `LastKnownInformation` (TR-4), ~5 lines at the two captures.

169. **The intervening "if" reads CR 109.5's "you" off the source's frame and
     answers false for a source that has left.** `settled_holds(condition,
     game, source)` is the evaluator at both instants (`match_def`,
     `resolve_taken`), and `holds` returns false for an object not in the
     store or with no frame to read "you" from — right for Felidar Sovereign,
     wrong for a dies-trigger with a clause (persist's shape, TR-4).

     **Reachability (2026-09-19):** unreachable — no registered trigger with an
     intervening "if" leaves the battlefield before its check.

     **Sized:** TR-2's `TriggerContext` evaluator (§6.1): the trigger's
     locked controller as "you" and the binding's frame for a clause about
     the bound object, ~40 lines.

170. **`OrderTriggers` offers the entries' sources, so two entries of one
     source are indistinguishable to a human client.** `ask_order_triggers`
     builds `ChoiceOption::Object(source)` per entry in trigger order; the
     engine's answer is a permutation either way, and the elision keeps the
     identical-binding case from being asked at all.

     **Reachability (2026-09-19):** reachable — not wrong: a presentation, and
     the CLI prints the same name twice.

     **Sized:** a `ChoiceOption::Trigger { seq, source }` variant and its
     arms, ~20 lines, when a client wants it (Phase 10).

171. **Two `TriggerEvent` arms shipped early and narrow.** `Attacks {
     attacker, occurrence }` (one attacker is one occurrence; no defender,
     no shape) because §13 owed ATOM-508.1m-001 here, and `GainsLife {
     player, occurrence }` because it owed ATOM-119.9-001/-002. TR-5 widens
     the first to the five shapes with item 11's defender; TR-2 adds
     `LosesLife`, the other half of the sign split.

     **Reachability (2026-09-19):** nothing owed — a record, so the later phases
     widen rather than add a second arm.

172. **The three bound-fact leaves are `TriggeringObject`, `TriggeringPlayer`
     and `TriggeringAmount`; `TriggeringPower` waits.** §3.4 named four; the
     fourth reads the live object or the frame's power (CR 608.2h), and the
     frame that carries a status is TR-4's. Nothing prints it before Paladin
     of Atonement's toughness read (TR-2) and Heart-Piercer Manticore's power
     (TR-3).

     **Reachability (2026-09-19):** nothing owed — a record for TR-2, whose
     `TriggeringToughness` is the same leaf with the other box.
