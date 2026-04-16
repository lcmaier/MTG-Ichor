# Implementation Plan: Pre-Phase 5 Engine Fixes + Phase 5 Continuous Effects

## Overview

- **Total ticket count:** 51 (30 active Part 1 tickets + 21 Part 2 tickets)
- **Scope breakdown:** 15 Small, 27 Medium, 9 Large (SPECIAL-5 Small, SPECIAL-6 Small–Medium added 2026-04-16)
- **Part 1:** Pre-Phase 5 engine fixes (T01–T22, T15b, T21a–T21d) covering E1–E48 + combat requirements solver (508.1d/509.1c)
- **Part 2:** Phase 5 continuous effects & layer system (L01–L21) covering W1–W15, PC1–PC12, §5a–§5m, plus E31 (LKI, deferred from Part 1)
- **Organization:** Part 1 tickets are grouped by tier (data model → state tracking → SBAs → casting → zone/combat/damage). Part 2 tickets follow Sub-Plan 5A/5B/5C structure. The Execution Order section linearizes across both parts with parallelism annotations.

---

## Execution Order (linearized)

### Legend
- `||` = can run in parallel
- `→` = sequential dependency
- `[P5-PREREQ]` = must complete before any Part 2 ticket
- `[GATE N]` = verification gate checkpoint

### Tier 1: Foundational Data Model (all parallel)
```
T01 [P5-PREREQ] || T02 || T03 || T04 || T05 [P5-PREREQ] || T06
```

### Tier 2: State Tracking
```
T09 [P5-PREREQ] → T10
T11 || T12                    (parallel with T09/T10)
T12 → T12b                   (T12b parallel with Tier 2)
```

### Tier 3: SBAs (after Tier 1 deps)
```
T13 (after T01,T03) || T14 (after T01) || T15 (after T04) || T15b (after T04) || T16 (after T01,T02)
```

### Tier 4: Casting & Activation (parallel with Tier 3)
```
T17 → T18a → SPECIAL-1a → SPECIAL-1b → SPECIAL-1c → {T18b, T18c, T18d}  (SPECIAL-1a/b/c = DP refactor split; T18b/c/d parallel after SPECIAL-1c)
SPECIAL-2 (priority retry on failed cast) — non-blocking QoL, pick up anytime after SPECIAL-1c test migration
SPECIAL-3 (shared test helpers for common DP script sequences) — non-blocking QoL, pick up anytime after SPECIAL-1c
SPECIAL-4 (CounterSpell/CounterAbility cleanup — use move_object) — non-blocking cleanup, pick up anytime
SPECIAL-5 (DP validation + contract property tests — Classes A/D) — non-blocking QoL, pick up anytime after SPECIAL-1c
SPECIAL-6 (ask_* option enumeration tests — Class C, living ticket) — non-blocking QoL, pick up anytime after SPECIAL-1c
T19 || T20                                    (parallel with T17/T18a–d, SPECIAL-1a/b/c)
T12b + T17 → T12c                            (after casting pipeline + sidecar)
```

### Tier 5: Zone, Combat, Damage, Targeting (parallel with Tier 4)
```
T21a (after T17) || T21b || T21c (after T01,T02) || T21d (after T21b) || T22
```

### Cross-Cutting: Mana Restrictions Cards
```
T12c → T12d                  (steps 1-5 after T12c; step 6 after L04)
```

### [GATE 1] — After Tier 1 data model tickets complete
### [GATE 2] — After ALL Part 1 tickets complete. All [P5-PREREQ] confirmed.

### Sub-Plan 5A: Foundation + P/T (L01–L08)
```
L01 → L02 (after T22 for hook sites)
L01 → L03
L01 + L03 → L04
L04 → L05
L04 + T01 → L06
L06 → L07
L07 + L02 → L08
```

### [GATE 3] — Giant Growth on Bears = 5/5, reverts. Oracle routes through layers.

### Sub-Plan 5B: Remaining Layers + Dependency (L09–L16)
```
L04 → L09 || L12 || L14 || L15    (all parallel after L04)
L04 + T05 → L10                    (parallel with above)
L04 + T09 → L11                    (parallel with above)
L06 + L09 + L10 + L11 → L13
L08 → L16
```

### [GATE 4] — All 7 layers functional. Blood Moon + Urborg correct.

### Sub-Plan 5C: Cards + Testing (L17–L21)
```
L08 + L10 + L11 + L14 → L17
L04 → L18
L05 + L09 + L10 + L14 → L19
L17 + L18 + L19 → L20
L17 + L19 + L20 → L21
```

### [GATE 5] — All tickets complete. Full regression pass.

---

## Testing Conventions

> Adopted 2026-04-16 following the DP refactor (SPECIAL-1a/b/c). Companion doc: `plans/test-strategy-post-dp-refactor.md`.

### Test class taxonomy
Every new test belongs to one of five classes. Ticket `Tests` bullets should tag tests by class (A/B/C/D/E) so reviewers can verify coverage at a glance.

| Class | Purpose | Typical location |
|-------|---------|------------------|
| **A** | Validation / negative: `validate_*` or `ask_*` rejects bad DP responses (`#[should_panic]` / `Result::Err` assertions) | `src/ui/ask.rs::tests`, `src/ui/random.rs::tests` |
| **B** | Decision sequence: assert the exact order and `ChoiceKind` of DP calls for an engine flow | integration tests (`tests/*`) |
| **C** | Option enumeration: assert the `ChoiceOption` list the engine presents to the DP for a given game state | `src/ui/ask.rs::tests` or `tests/ask_enumeration_test.rs` |
| **D** | Contract property: `RandomDecisionProvider` responses always satisfy the 4-primitive contract | `src/ui/random.rs::tests`, fuzz harness |
| **E** | Regression for scenarios previously masked by `PassiveDecisionProvider` (specific-choice semantics) | colocated with the engine module under test |

### Per-ticket coverage rules
When a ticket does any of the following, it **must** land the corresponding test(s) in the same PR:

- **Adds/modifies an `ask_*` function** → at least one Class C test.
- **Adds a `ChoiceKind` variant** → at least one Class B test exercising it end-to-end, plus a Class A test if the variant introduces new validation (e.g., new bounds or per-bucket constraints).
- **Adds a `SelectionFilter` or `PermanentFilter` variant** → at least one Class C test asserting the filter's inclusion/exclusion behavior.
- **Adds a new multi-step engine flow** (cast pipeline changes, new SBA, new trigger checkpoint, new combat step) → at least one Class B test asserting the DP call sequence.
- **Adds a new primitive input constraint** (new `per_bucket_mins`/`per_bucket_maxs` use site, new bounds) → at least one Class A test asserting the constraint rejects invalid responses.
- **Adds a `RandomDecisionProvider`-visible `ChoiceKind`** → fuzz harness smoke run (200 games) must pass in the ticket's acceptance criteria.
- **Replaces a former `PassiveDecisionProvider`-masked default** (legacy scaffolding only; should be rare post-SPECIAL-1c) → at least one Class E test.

### Standing rules
1. **Write the failing test first for DP/`ask_*` surface changes.** Retrofits are what SPECIAL-1c was; avoid repeating.
2. **Never weaken or delete an existing test without explicit direction.** Tests are contracts.
3. **`ScriptedDecisionProvider` requires explicit expectations.** No silent defaults. If the engine asks for a decision the test didn't anticipate, that's a bug (in the test or the engine) — diagnose, don't paper over.
4. **Keep `ChoiceKind` exhaustive and meaningful.** Resist catch-all variants. Every variant should carry enough semantic context that `ask_*` and validation can be written against it.
5. **Fuzz harness is the living Class D suite.** Debug-mode contract assertions inside `ask_*` wrappers (landing in SPECIAL-5) keep it honest.
6. **Report full test counts in ticket summaries:** `N new, X unit + Y integration + Z doc-test = T total`.

---

## Part 1: Pre-Phase 5 Engine Fixes

*[Merged verbatim from implementation-plan-part1.md]*

### Overview

- **Scope:** E-items only (E1–E48) from `plans/phase5-pre-work-engine-fixes.md`. No layer system, no W-items, no Phase 5 steps.
- **Codebase state:** 287 tests (238 unit + 48 integration + 1 doc-test), 0 warnings, 500/500 fuzz games pass.
- **Grouping:** Tickets are organized by tier (matching source doc). Tightly coupled E-items sharing the same struct/file are combined. Large items are split into 2–3 tickets.
- **Ticket count:** 24 active tickets (T07 resolved in-place, no ticket needed). T20b (LKI/E31) deferred to Part 2 — requires `EffectiveCharacteristics` from §5c. T21d (combat requirements solver) added during plan synthesis.

---

### Tier 1: Foundational Data Model (T01–T06) (DONE ✅)

#### T01: Add counters to BattlefieldEntity + expand CounterType [P5-PREREQ] (DONE ✅)
- **Scope:** Small
- **Source:** E1 + E2
- **Depends on:** none
- **Files:** `state/battlefield.rs` (modify), `types/effects.rs` (modify)
- **Steps:**
  1. In `types/effects.rs`, expand `CounterType` enum with the evergreen keyword counter types (rule 122.1b): `Flying`, `Deathtouch`, `Lifelink`, `Trample`, `FirstStrike`, `DoubleStrike`, `Hexproof`, `Indestructible`, `Menace`, `Reach`, `Vigilance`, `Haste`. Non-evergreen counter types (Shield, Stun, Lore, Level, etc.) will be added as relevant cards are implemented.
  2. In `state/battlefield.rs`, add `pub counters: HashMap<CounterType, u32>` to `BattlefieldEntity`. Import `HashMap` and `CounterType`.
  3. In `BattlefieldEntity::new()`, initialize `counters: HashMap::new()`.
  4. Add helper methods: `add_counters(&mut self, ct, n)`, `remove_counters(&mut self, ct, n) -> u32` (returns actual removed), `counter_count(&self, ct) -> u32`.
- **Codebase verification:** Confirmed `BattlefieldEntity` has comment `// Counters (future: HashMap<CounterType, u32>)` at line 38 — placeholder matches. `CounterType` in `types/effects.rs:173-180` currently has only `PlusOnePlusOne`, `MinusOneMinusOne`, `Loyalty`, `Charge`.
- **Tests:**
  - `test_add_counters` — add +1/+1 counters, verify count
  - `test_remove_counters` — remove counters, verify clamped at 0
  - `test_counter_count_default_zero` — unset counter type returns 0
  - `test_multiple_counter_types` — different types coexist
  - Note: Tests here verify the *data model* (add/remove/query). Testing that a keyword counter (e.g., Flying counter) actually grants the keyword ability is a Phase 5 integration concern (L7c counter reading) and belongs there, not here.
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: add counters field to BattlefieldEntity, expand CounterType (E1, E2)`

---

#### T02: Add player counters (poison, commander damage) (DONE ✅)
- **Scope:** Small
- **Source:** E3
- **Depends on:** none
- **Files:** `state/player.rs` (modify)
- **Steps:**
  1. Add `pub poison_counters: u32` to `PlayerState`.
  2. Add `pub commander_damage_taken: HashMap<ObjectId, u32>` to `PlayerState`. Import `ObjectId` (already imported) and `HashMap`.
  3. Initialize both in `PlayerState::new()`: `poison_counters: 0`, `commander_damage_taken: HashMap::new()`.
- **Codebase verification:** Confirmed `PlayerState` in `state/player.rs:9-28` has no counter fields. `ObjectId` import already present.
- **Tests:**
  - `test_player_poison_counters_default` — starts at 0
  - `test_player_commander_damage_default` — empty map
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `state: add poison_counters and commander_damage_taken to PlayerState (E3)`

---

#### T03: Add is_token and is_copy flags to GameObject (DONE ✅)
- **Scope:** Small
- **Source:** E4 + E5
- **Depends on:** none
- **Files:** `objects/object.rs` (modify)
- **Steps:**
  1. Add `pub is_token: bool` and `pub is_copy: bool` to `GameObject`.
  2. Initialize both to `false` in `GameObject::new()` and `GameObject::in_library()`.
  3. Note: The `is_copy` resolution branch in `stack.rs` (copy-of-permanent → create token) is deferred until spell copying is implemented. Only the field is added now.
- **Codebase verification:** Confirmed `GameObject` in `objects/object.rs:18-27` has no token/copy fields.
- **Tests:**
  - `test_game_object_default_not_token` — `is_token == false`
  - `test_game_object_default_not_copy` — `is_copy == false`
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `objects: add is_token and is_copy flags to GameObject (E4, E5)`

---

#### T04: Add attachment tracking to BattlefieldEntity (DONE ✅)
- **Scope:** Medium
- **Source:** E6
- **Depends on:** none
- **Files:** `state/battlefield.rs` (modify), `engine/zones.rs` (modify)
- **Steps:**
  1. Add `pub attached_to: Option<ObjectId>` and `pub attached_by: Vec<ObjectId>` to `BattlefieldEntity`.
  2. Initialize in `BattlefieldEntity::new()`: `attached_to: None`, `attached_by: Vec::new()`.
  3. In `engine/zones.rs` `cleanup_zone_state` (currently a stub hook), add detachment logic: when a permanent leaves the battlefield, if it has `attached_to`, remove its ID from the host's `attached_by`. If it has non-empty `attached_by`, clear `attached_to` on each attachment (Aura SBA will handle the resulting unattached auras).
  4. Add helper methods on `BattlefieldEntity`: `attach_to(&mut self, host: ObjectId)`, `detach(&mut self)`.
- **Codebase verification:** Confirmed `BattlefieldEntity` has no attachment fields. `cleanup_zone_state` is a stub (per Phase 3 audit memory).
- **Tests:**
  - `test_attachment_tracking_basic` — attach, verify both sides
  - `test_detach_clears_both_sides` — detach, verify cleanup
  - `test_zone_exit_detaches` — permanent leaves battlefield, attachments updated
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `state: add attachment tracking to BattlefieldEntity (E6)`

---

#### T05: Add color_indicator to CardData [P5-PREREQ] (DONE ✅)
- **Scope:** Small
- **Source:** E7
- **Depends on:** none
- **Files:** `objects/card_data.rs` (modify)
- **Steps:**
  1. Add `pub color_indicator: Option<Vec<Color>>` to `CardData`. Import `Color` (already imported).
  2. Initialize to `None` in `CardDataBuilder::new()`.
  3. Add builder method `color_indicator(mut self, colors: Vec<Color>) -> Self`.
  4. Note: `EffectiveCharacteristics` does not exist yet — the corresponding field will be added when that struct is created in Phase 5.
- **Codebase verification:** Confirmed `CardData` in `objects/card_data.rs:18-32` has no `color_indicator` field. `Color` is imported at line 6.
- **Tests:**
  - `test_card_data_color_indicator_none_default` — default is None
  - `test_card_data_color_indicator_set` — builder sets it correctly
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `objects: add color_indicator to CardData (E7)`

---

#### T06: Add x_value to BattlefieldEntity + carry from StackEntry (DONE ✅)
- **Scope:** Small
- **Source:** E9
- **Depends on:** none
- **Files:** `state/battlefield.rs` (modify), `engine/stack.rs` (modify)
- **Steps:**
  1. Add `pub x_value: Option<u32>` to `BattlefieldEntity`. Initialize to `None` in `new()`.
  2. In `engine/stack.rs` `resolve_top_of_stack`, in the permanent-type resolution branch (lines ~65-88), after `init_zone_state_with_controller`, set `x_value` on the newly created `BattlefieldEntity` from `entry.x_value`.
- **Codebase verification:** Confirmed `StackEntry` already has `x_value: Option<u32>` (seen in `cast.rs:73`). `BattlefieldEntity` has no `x_value` field. Stack resolution at `stack.rs:65-76` handles permanent spell placement.
- **Tests:**
  - `test_x_value_carried_to_permanent` — cast X-cost spell, verify BattlefieldEntity.x_value matches
  - `test_x_value_none_for_non_x_spell` — normal spell resolves with x_value = None
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: carry x_value from StackEntry to BattlefieldEntity (E9)`

---

> **T07 (E8 + E10 + E11) — RESOLVED, no ticket needed.**
> - E8 (P/T signedness): Verified — all fields are `i32`. No change needed.
> - E10 (Battle type in permanent check): **Fixed in code** — `stack.rs` now uses `obj.card_data.types.iter().any(|t| t.is_permanent())` instead of manual type list. Commit: `fix: use CardType::is_permanent() in stack resolution (E10)`.
> - E11 (Enum completeness): Verified — `ArtifactType` (19 variants) and `LandType` (17 variants) are complete for current needs.

---

### Tier 2: State Tracking (T09–T12, T12b) (DONE ✅)

#### T09: Summoning sickness — controller_since_turn [P5-PREREQ] (DONE ✅)
- **Scope:** Medium
- **Source:** E12
- **Depends on:** none
- **Files:** `state/battlefield.rs` (modify), `engine/zones.rs` (modify), `engine/turns.rs` (modify), `oracle/characteristics.rs` (modify), `engine/costs.rs` (modify)
- **Steps:**
  1. In `state/battlefield.rs`, replace `pub summoning_sick: bool` with:
     - `pub entered_battlefield_turn: u32`
     - `pub controller_since_turn: u32`
  2. Update `BattlefieldEntity::new()` to accept a `current_turn: u32` parameter and set both fields to it. Remove `summoning_sick: true`. **Convention:** `current_turn = 0` is a sentinel meaning "before the game began" (rule 103.6 pregame actions, e.g. Leylines). Normal gameplay callers pass `game.turn_number` (which starts at `1`). The query `controller_since_turn >= game.turn_number` correctly yields `false` (not sick) for pregame permanents since `0 >= 1` is `false`.
  3. In `engine/zones.rs`, update `init_zone_state` and `init_zone_state_with_controller` to pass the current turn number from `GameState.turn_number` (verified: initializes to `1`). Future pregame action code in `Game::setup()` will pass `0` explicitly.
  4. In `engine/turns.rs`, remove the untap-step logic that clears `summoning_sick` (if it exists).
  5. Create a query function (in `oracle/characteristics.rs` or inline): `has_summoning_sickness(game, object_id) -> bool` = `entry.controller_since_turn >= game.turn_number && !has_keyword(game, id, Haste)`.
  6. Update `engine/costs.rs` `check_cost_resource(Cost::Tap)` and `pay_single_cost(Cost::Tap)` to use the new query instead of `entry.summoning_sick`.
  7. Update `oracle/legality.rs` `can_attack` to use the new query.
  8. Search all references to `summoning_sick` and update them.
- **Codebase verification:** Confirmed `BattlefieldEntity.summoning_sick: bool` at `battlefield.rs:23`. `costs.rs:58-63` checks `entry.summoning_sick`. Tests in `costs.rs:222-224` set `entry.summoning_sick = false` — these need updating.
- **Tests:**
  - `test_new_permanent_has_summoning_sickness` — enters on turn 1, can't tap on turn 1
  - `test_summoning_sickness_clears_next_turn` — can tap on turn 2
  - `test_control_change_resets_summoning_sickness` — changing controller_since_turn re-sickens
  - `test_haste_bypasses_summoning_sickness` — haste creature can tap immediately
  - `test_haste_control_change` — haste creature changes controller, is still hasty
  - `test_pregame_permanent_no_summoning_sickness` — permanent with `controller_since_turn = 0` on turn 1 is NOT summoning sick (rule 103.6 pregame actions)
- **Acceptance:** All existing tests pass (after updating `summoning_sick` references) + new tests pass + 0 warnings
- **Commit:** `engine: replace summoning_sick bool with controller_since_turn tracking (E12)`

---

#### T10: {Q} (Untap symbol) summoning sickness check (DONE ✅)
- **Scope:** Small
- **Source:** E13
- **Depends on:** T09
- **Files:** `engine/costs.rs` (modify)
- **Steps:**
  1. In `check_cost_resource(Cost::Untap)`, add the same summoning sickness check that `Cost::Tap` has: if the permanent is a creature with summoning sickness and no haste, return Err.
  2. In `pay_single_cost(Cost::Untap)`, add the same guard.
- **Codebase verification:** Confirmed `Cost::Untap` in `costs.rs:67-73` only checks `!entry.tapped` — no summoning sickness check.
- **Tests:**
  - `test_untap_cost_blocked_by_summoning_sickness` — creature can't pay {Q} on ETB turn
  - `test_untap_cost_allowed_with_haste` — haste creature can pay {Q}
  - `test_untap_cost_allowed_on_noncreature` — artifact with {Q} cost is fine
  - `test_untap_cost_blocked_by_control_change` — creature can't pay {Q} on control change turn
  - `test_untap_cost_allowed_control_change_haste` — haste creature can pay {Q} on control change turn
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: add summoning sickness check to Cost::Untap (E13)`

---

#### T11: LifeChanged event — add source field (DONE ✅)
- **Scope:** Small
- **Source:** E14
- **Depends on:** none
- **Files:** `events/event.rs` (modify), `engine/actions.rs` (modify)
- **Steps:**
  1. Add `source: Option<ObjectId>` to `GameEvent::LifeChanged`. Use `Option` because some life changes (e.g., paying life as a cost) may not have a source object.
  2. Update all `LifeChanged` emission sites in `engine/actions.rs` (`perform_action` for `GainLife` and `LoseLife`) to include the source.
  3. Search for any other `LifeChanged` emission sites and update them.
- **Codebase verification:** Confirmed `LifeChanged` at `event.rs:56` has only `player_id`, `old`, `new` — no source field.
- **Tests:**
  - `test_life_changed_event_includes_source` — gain life from lifelink, verify source is the damage source
  - `test_simultaneous_lifelink` — two creatures with lifelink both deal damage, produce two LifeChanged events (to be handled during SBAs).
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `events: add source field to LifeChanged event (E14)`

---

#### T12: Mana spending restrictions — design spike (DONE ✅)
- **Scope:** Small (document output, no production code)
- **Source:** E15
- **Depends on:** none
- **Files:** `plans/mana-restrictions-design.md` (create)
- **Steps:**
  1. Write a design document (`plans/mana-restrictions-design.md`) that resolves the open questions below and recommends a concrete implementation approach.
  2. The document should include: data model, API surface, interaction with existing `ManaPool`/`pay_costs`/`mana_helpers`, test plan, and a migration strategy (how to adopt without breaking existing callers).
  3. Once the design doc is approved, follow-up implementation tickets (T12b, T12c, T12d) will be created.
- **Open questions resolved:**
  1. **Where does restriction metadata live?** On `ManaPool` as a `special: Vec<(ManaAtom, u64)>` counted-group sidecar.
  2. **How do restrictions compose?** Cost modification is independent. Restricted mana counts toward Trinisphere's minimum if the restriction is satisfied.
  3. **Multi-restriction mana:** Single `ManaRestriction` with OR within `OnlyForSpellTypes`. `AnyOf` for cross-category OR.
  4. **Interaction with cost modification:** No interaction needed. Clean separation at 601.2e (modify cost) vs 601.2h (pay cost). Grants flow downstream.
- **Acceptance:** Design document written, all 4 open questions resolved, approach approved ✅
- **Commit:** `docs: mana spending restrictions design spike (E15)`

---

#### T12b: ManaPool sidecar — types and methods (DONE ✅)
- **Scope:** Medium
- **Source:** E15 (implementation phase A from `plans/mana-restrictions-design.md` §10)
- **Depends on:** T12
- **Files:** `types/mana.rs` (modify), `types/effects.rs` (modify)
- **Steps:**
  1. Add `special: Vec<(ManaAtom, u64)>` and `last_spent_grants: Vec<ManaGrant>` fields to `ManaPool`.
  2. Implement `add_special` (coalesces identical atoms), `has_special`, `special_atoms`, `drain_spent_grants`.
  3. Implement `empty_with_reason` with `BlanketPersistenceSet` parameter. Keep `empty()` as test convenience (`BlanketPersistenceSet::none()`).
  4. Implement `can_pay_with_context` and `pay_with_plan`.
  5. Add new types: `SpendContext`, `SpendPurpose`, `ManaPaymentPlan`, `ManaEmptyReason`, `BlanketPersistenceSet`, `PersistenceExpiry`. `ManaPersistence` has two variants: `Normal` and `UntilEndOf` (no `Indefinite`).
  6. Expand `ManaRestriction` enum: `OnlyForSpellTypes`, `OnlyForAbilityTypes`, `AnyOf`, `OnlyForCreatureType`.
  7. Add `ManaRestriction::allows()` (private) and `ManaAtom::allows_spend()` (private).
- **Design reference:** `plans/mana-restrictions-design.md` §3 (Data Model), §4 (API Surface)
- **Tests:**
  - `test_add_special_atom` — add restricted atom, verify `has_special()` and `special_atoms()`
  - `test_special_coalesce_identical_atoms` — 5 identical atoms → one group with count 5
  - `test_amount_for_with_eligible_special` — creature-only {G} + creature context → counts
  - `test_can_pay_with_context_uses_special` — restricted mana makes otherwise-unpayable cost payable
  - `test_pay_with_plan_mixed` — some from simple, some from special
  - `test_pay_with_plan_collects_grants` — spent atoms' grants appear in `drain_spent_grants()`
  - `test_empty_with_reason_step_no_blanket` — normal empties, time-gated persists
  - `test_restriction_allows_spell_type_match` / `_mismatch` / `_any_of` / `_creature_type`
- **Acceptance:** All existing tests pass unchanged + new unit tests pass + 0 warnings
- **Commit:** `mana: add ManaPool sidecar for restricted/granted mana (T12b)`

---

### Tier 3: State-Based Actions (T13–T16, T15b) (DONE ✅)

#### T13: Counter annihilation + token cease-to-exist SBAs (DONE ✅)
- **Scope:** Small
- **Source:** E16 + E17
- **Depends on:** T01 (counters), T03 (is_token)
- **Files:** `engine/sba.rs` (modify)
- **Steps:**
  1. Add 704.5r (+1/+1 and -1/-1 counter annihilation): iterate battlefield, find permanents with both `PlusOnePlusOne` and `MinusOneMinusOne` counters, remove pairs (min of both counts).
  2. Add 704.5d (token cease-to-exist): iterate all objects, find objects with `is_token == true` in zones other than `Battlefield`. Remove directly from the objects store + zone collection (tokens cease to exist, not "die" — no death trigger, no zone-change event). Emit a dedicated `GameEvent::TokenCeasedToExist { object_id }` event for logging/debugging. Add the new variant to `events/event.rs`.
  3. Place both checks in `check_state_based_actions` after the existing checks.
- **Codebase verification:** Confirmed `sba.rs` has comment placeholders for future SBAs at lines 90-93. No counter annihilation or token SBA exists.
- **Tests:**
  - `test_sba_counter_annihilation` — permanent with 3 +1/+1 and 2 -1/-1 → ends with 1 +1/+1 and 0 -1/-1
  - `test_sba_counter_annihilation_equal` — equal counts → both zeroed
  - `test_sba_token_ceases_to_exist_in_graveyard` — token in graveyard is removed
  - `test_sba_token_on_battlefield_stays` — token on battlefield not removed
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: add counter annihilation and token cease-to-exist SBAs (E16, E17)`

---

#### T14: Legend rule + planeswalker loyalty SBAs (DONE ✅)
- **Scope:** Medium
- **Source:** E18 + E20
- **Depends on:** T01 (counters for loyalty)
- **Files:** `engine/sba.rs` (modify), `oracle/characteristics.rs` (modify), `ui/decision.rs` (modify)
- **Steps:**
  1. Add `get_effective_name(game, object_id) -> String` to `oracle/characteristics.rs`. For now, returns `card_data.name`. Phase 5 will route through L1 copy effects.
  2. Add 704.5j (legend rule) to `check_state_based_actions`: group legendary permanents by (controller, effective_name). For each group with >1, the controller chooses one to keep; rest go to graveyard. This requires a new `DecisionProvider` method: `choose_legend_to_keep(game, player_id, legendaries: Vec<ObjectId>) -> ObjectId`.
  3. Add 704.5i (planeswalker 0 loyalty → graveyard): iterate battlefield for planeswalker-typed permanents, check `counters.counter_count(Loyalty) == 0`. If so, move to graveyard. **Note:** The SBA only checks "loyalty == 0." The question of what happens when a 6-power creature attacks a PW with 4 loyalty is handled by **damage routing** (T21 step 7), not this SBA. Per rule 119.3a, damage dealt to a PW removes that many loyalty counters. The removal should be `min(damage, current_loyalty)` — excess combat damage to a PW doesn't overflow anywhere; it's simply absorbed. After removal, loyalty == 0, and *this* SBA puts the PW into the graveyard.
  4. Planeswalker ETB should set initial loyalty counters from `card_data.loyalty` — add this to `init_zone_state` when the permanent is a planeswalker. Guard: only set counters if `card_data.loyalty` is `Some(n)` where `n > 0`. If `n <= 0` (no real card, but defensive), set 0 counters and let the SBA handle it immediately. No negative loyalty counters — `Loyalty` counter type uses `u32`.
- **Codebase verification:** Confirmed SBA comment at `sba.rs:90-91` mentions 704.5i and 704.5j as future. `DecisionProvider` currently has 8 methods.
- **Tests:**
  - `test_sba_legend_rule_two_same_name` — two legendary permanents with same name, one is kept
  - `test_sba_legend_rule_different_names_ok` — two different legendaries coexist
  - `test_sba_legend_rule_different_controllers_ok` — same name, different controllers coexist
  - `test_sba_planeswalker_zero_loyalty_dies` — planeswalker with 0 loyalty counters goes to graveyard
  - `test_sba_planeswalker_with_loyalty_stays` — planeswalker with >0 loyalty stays
  - `test_planeswalker_etb_sets_loyalty_counters` — PW enters battlefield, loyalty counters set from card_data
  - `test_planeswalker_zero_printed_loyalty_dies_immediately` — PW with printed loyalty 0 enters, SBA kills it
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: add legend rule and planeswalker 0-loyalty SBAs (E18, E20)`

---

#### T15: Aura/Equipment legality SBAs (DONE ✅)
- **Scope:** Medium
- **Source:** E19
- **Depends on:** T04 (attachment tracking)
- **Files:** `engine/sba.rs` (modify)
- **Steps:**
  1. Add 704.5m (Aura not attached → graveyard): find Aura-subtyped permanents with `attached_to == None`, move to graveyard.
  2. Add 704.5n (Aura attached to illegal object): **Phase 1 scope:** verify the host still exists on the battlefield. If not, move Aura to graveyard. **Deferred:** the full enchant restriction check (e.g., "Enchant creature" verifying host is still a creature) requires a structured representation of what an Aura can enchant. This depends on how Auras are modeled — currently no `AbilityDef` carries a structured enchant restriction, and Aura ETB attachment (rule 303.4f) is not implemented. Add a `// TODO: check enchant restriction once Aura targeting model is implemented` comment. The full check becomes a dependency when Aura cards are implemented.
  3. Add 704.5p (Equipment attached to non-creature → unattach): find Equipment with `attached_to` pointing to a non-creature. Set `attached_to = None` and remove from host's `attached_by`. Equipment stays on battlefield.
  4. Add 704.5q (non-Aura non-Equipment attached → unattach): catch-all for illegal attachments.
  5. **TODO comment (rule 303.4d):** Add `// TODO: 704.5r — An Aura that is also a creature can't enchant anything. If this occurs, the Aura becomes unattached and is put into its owner's graveyard. Relevant when L4 type-changing effects (e.g., a hypothetical non-Aura-excluding Opalescence variant) add Creature to an Aura. Bestow (702.103) avoids this by removing the Aura type when the enchanted permanent leaves, but the general SBA is still needed. Implement when L4 type-changing + Aura cards coexist.` in `sba.rs` near the other attachment SBAs.
- **Codebase verification:** No attachment SBAs exist. Depends on T04 attachment fields.
- **Tests:**
  - `test_sba_unattached_aura_dies` — Aura with no host goes to graveyard
  - `test_sba_aura_host_left_battlefield` — host removed, Aura goes to graveyard
  - `test_sba_equipment_on_noncreature_unattaches` — Equipment detaches but stays on battlefield
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: add Aura/Equipment legality SBAs (E19)`

---

#### T15b: Aura attachment logic (rule 303.4f) (DONE ✅)
- **Scope:** Small
- **Source:** Dependency surfaced by T15 (not an E-item — infrastructure for future Aura cards)
- **Depends on:** T04 (attachment tracking)
- **Note:** T15 and T15b are independent after T04 and can run in parallel.
- **Files:** `objects/card_data.rs` (modify), `types/effects.rs` (modify), `engine/stack.rs` (modify), `engine/resolve.rs` (modify), `engine/sba.rs` (modify), `engine/targeting.rs` (modify)
- **Steps:**
  1. **Enchant filter on CardData.** Added `pub enchant_filter: Option<SelectionFilter>` to `CardData`. Uses `SelectionFilter` directly — no intermediate `EnchantRestriction` enum. `PermanentFilter` extended with `BySubtype(Subtype)` and `PowerLE(i32)` variants to cover all enchant restriction cases. Builder method: `.enchant_filter(SelectionFilter::Creature)`.
  2. **Aura ETB from stack (rule 303.4f).** In `engine/stack.rs`, after permanent resolution for an Aura-subtyped spell, set `attached_to` on the `BattlefieldEntity` to the resolved target (from `entry.chosen_targets`). Aura's `object_id` added to host's `attached_by`.
  3. **Aura ETB without targeting (rule 303.4a).** Added `attach_aura_on_etb(aura_id, controller, dp)` to `engine/resolve.rs`. Reads `enchant_filter` directly, pre-checks via `has_any_legal_choice` (extracted to `targeting.rs` as `pub(crate)` helper), then asks DP to choose via `EffectRecipient::Choose(filter, TargetCount::Exactly(1))`. No `TargetContext` enum needed — the `Choose` vs `Target` distinction in `EffectRecipient` already conveys that targeting rules don't apply (hexproof/shroud don't apply per 303.4a). Returns `Ok(false)` if no legal host; SBA 704.5m handles cleanup.
  4. **SBA 704.5n updated.** Uses `validate_selection(filter, &ResolvedTarget::Object(host_id))` for the full legality check — single validation path for targeting, SBA, and Aura ETB. Deleted the intermediate `check_enchant_restriction()` function (~50 lines).
- **Design decision:** Initially created an `EnchantRestriction` enum with `to_selection_filter()` bridge method. Immediately recognized as an anti-pattern (parallel type hierarchy) and refactored to use `SelectionFilter` directly on `CardData`. The `EnchantRestriction` enum was deleted in the same MR.
- **Tests:**
  - `test_aura_attaches_to_target_on_resolve` — Aura spell resolves, attached_to set to target
  - `test_aura_host_in_attached_by` — host's attached_by includes the Aura
  - `test_aura_etb_no_legal_target_dies` — Aura enters without legal host, SBA removes it
  - `test_enchant_filter_creature_only` — Aura with SelectionFilter::Creature rejects non-creature
  - `test_aura_etb_non_stack_chooses_host` — rule 303.4a: Aura ETB not from stack, chooses host
  - `test_aura_etb_non_stack_no_legal_host` — no legal host, returns Ok(false), SBA kills
  - `test_aura_etb_non_aura_noop` — non-Aura permanent returns Ok(false), no-op
- **Acceptance:** 370 tests pass, 0 warnings
- **Commit:** `engine: Aura attachment logic, enchant_filter on CardData, unified validate_selection (303.4f)`

---

#### T16: Remaining SBAs — poison, commander, indestructible, cleanup re-loop (DONE ✅)
- **Scope:** Small
- **Source:** E21 + E22 + E23 + E24
- **Depends on:** T02 (player counters), T01 (for indestructible check context)
- **Files:** `engine/sba.rs` (modify), `engine/resolve.rs` (modify), `engine/turns.rs` or `state/game.rs` (modify)
- **Steps:**
  1. Add 704.5c (poison ≥ 10 → lose): check `player.poison_counters >= 10`. Add new `LossReason::PoisonCounters` variant.
  2. Add 704.5 commander damage (21+ from single commander → lose): iterate `player.commander_damage_taken`, check any value ≥ 21. Add `LossReason::CommanderDamage` variant.
  3. Add indestructible guard to 704.5g (lethal damage SBA): skip creatures with `has_keyword(game, id, Indestructible)` in the lethal damage check. The existing `// TODO: check for indestructible / regeneration` comment at `sba.rs:84` confirms this is needed.
  4. Add indestructible guard to `resolve_primitive(Destroy)` in `engine/resolve.rs`: if target has indestructible, the destroy effect fails (does nothing, not an error).
  5. Cleanup SBA re-loop (rule 514.3a): in the cleanup step handler (in `state/game.rs` `run_turn` or `engine/turns.rs`), after running SBAs, if any SBA fired, re-run cleanup (damage removal + discard) and give players priority. Implement as a loop.
- **Codebase verification:** Confirmed `sba.rs:84` has `// TODO: check for indestructible / regeneration`. `LossReason` at `event.rs:82-88` has only `LifeReachedZero` and `DrawnFromEmptyLibrary`.
- **Tests:**
  - `test_sba_poison_10_loses` — player with 10 poison counters loses
  - `test_sba_poison_9_survives` — player with 9 poison counters does not lose
  - `test_sba_commander_damage_21_loses` — 21 damage from one commander → lose
  - `test_sba_indestructible_survives_lethal_damage` — indestructible creature with lethal damage stays
  - `test_sba_indestructible_survives_destroy` — Destroy primitive does nothing to indestructible
  - `test_cleanup_sba_reloop` — SBA during cleanup triggers re-loop with priority
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: add poison/commander SBAs, indestructible guards, cleanup re-loop (E21–E24)`

---

### Tier 4: Casting & Activation (T17–T20b, T12c)

#### T17: Alternative/additional cost framework — type definitions (DONE ✅)
- **Scope:** Medium
- **Source:** E25 (part 1 of 2)
- **Depends on:** none
- **Files:** `types/effects.rs` (modify), `engine/stack.rs` (modify — StackEntry fields)
- **Steps:**
  1. Define cost type enums in `types/effects.rs`:
     ```rust
     pub enum AlternativeCost {
         Flashback(Vec<Cost>),
         Overload(Vec<Cost>),
         Dash(Vec<Cost>),
         Escape { mana: Vec<Cost>, exile_count: u32 },
         Evoke(Vec<Cost>),
         Bestow(Vec<Cost>),
         Custom(String, Vec<Cost>),
     }
     pub enum AdditionalCost {
         Kicker(Vec<Cost>),
         Buyback(Vec<Cost>),
         Entwine(Vec<Cost>),
         Casualty(u32),
         Bargain,
         Strive(Vec<Cost>), // per additional target
         Custom(String, Vec<Cost>),
     }
     ```
  2. Add to `StackEntry`:
     ```rust
     pub chosen_alternative_cost: Option<AlternativeCost>,
     pub additional_costs_paid: Vec<AdditionalCost>,
     ```
  3. Add to `CardData` (or `AbilityDef`):
     ```rust
     pub alternative_costs: Vec<AlternativeCost>,
     pub additional_costs: Vec<AdditionalCost>,
     ```
  4. Update `CardDataBuilder` with `alternative_cost()` and `additional_cost()` builder methods.
  5. Initialize the new `StackEntry` fields to `None` / `Vec::new()` in `cast.rs` where StackEntry is created.
- **Tests:**
  - `test_stack_entry_default_no_alt_cost` — default StackEntry has None/empty
  - `test_card_data_with_kicker` — builder creates card with kicker additional cost
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `types: add AlternativeCost, AdditionalCost enums and StackEntry fields (E25 part 1)`

---

#### T18: Casting pipeline — 601.2 compliance + casting restrictions (SPLIT into T18a–T18d)

> **Split rationale (2026-04-13):** The original T18 combined 4 E-items, restructured the most complex engine file, added 5+ DP methods, and covered ~35 ATOMs across orthogonal concerns. It has been split into four sub-tickets with a clear dependency graph: `T17 → T18a → {T18b, T18c, T18d}` (T18b/c/d are parallel after T18a).

##### 601.2 Comparison: Rules vs Current Code vs T18a–d

| Rule | Requires | Current `cast.rs` | Sub-ticket | Deferred? |
|------|----------|-------------------|------------|-----------|
| **601.2a** | Move to stack; apply continuous effects modifying spell characteristics on stack entry | ✅ `move_object(card_id, Zone::Stack)` | T18a (no change) | Continuous effects on stack entry → Phase 5 |
| **601.2b** | Choose modes; announce splice; announce alt/additional costs; announce X; announce hybrid/Phyrexian mana choices | ❌ No mode choice. X hardcoded to None. No alt/add cost. No hybrid/Phyrexian. | T18a (X value, alt/add cost choice), T18b (modes) | Hybrid mana, Phyrexian mana, splice → later tickets |
| **601.2c** | Choose targets (may vary based on modes/costs chosen in 601.2b) | ✅ `choose_targets` via DP, `validate_targets` | T18b (conditional targets, target uniqueness) | — |
| **601.2d** | Divide/distribute effects among targets | ❌ Not implemented | T18c (`choose_distribution` DP method) | — |
| **601.2e** | Legality check *after* proposal (601.2a–d) | ⚠️ Runs *before* proposal (before `move_object`) | T18a (split into pre- + post-proposal) | — |
| **601.2f** | Determine total cost: base/alt + additional + increases − reductions, then lock | ❌ Uses `card_data.mana_cost` directly | T18a (cost assembly + passthrough stub) | Full pipeline → L15 (Phase 5 Layers) |
| **601.2g** | Player may activate mana abilities before paying | ❌ No explicit mana ability window | T18a (TODO comment) | Explicit mana ability window → later |
| **601.2h** | Pay total cost (deterministic first, then random/library) | ✅ `pay_costs` | T18a (assembled cost), T18c (payment ordering) | — |
| **601.2i** | Apply cast-modification effects; spell becomes cast; triggers fire | ✅ `SpellCast` event emitted | T18a (no change) | Cast-trigger firing → Phase 7 |

##### Design note: Two-tier legality check (601.2e)

The pre-proposal check (timing, zone, ownership) is a **fast path** that catches ~99% of illegal casts cheaply before any state mutation. The post-proposal check (601.2e) is a **safety net** that runs after modes/targets/costs are chosen — it costs almost nothing when it passes. This two-tier pattern mirrors the mana system (pool check vs. full affordability). Both tiers are needed:
- **Correctness:** 601.2e is required by the CR. Continuous effects can make a proposed spell illegal after choices are made.
- **Speed:** The fast path prevents expensive work; the post-check is cheap since all info is already gathered.
- **Extensibility:** Once layers exist (Phase 5), continuous effects *will* create situations where post-proposal checking is load-bearing (e.g. Thalia making a spell unaffordable after cost assembly).

##### Follow-up note (EffectRecipient)

~~`TargetSpec` conflates targeting information with effect recipient.~~ **COMPLETED EARLY (T15b MR, 2026-04-12).** `TargetSpec` has been refactored into `EffectRecipient` with variants `Implicit`, `Controller`, `Target(SelectionFilter, TargetCount)`, `Choose(SelectionFilter, TargetCount)`. `Effect::Atom` is now `Atom(Primitive, EffectRecipient)`. `Target` = MTG targeting rules apply (hexproof/shroud/protection, fizzle). `Choose` = non-targeting selection (rule 303.4a Aura ETB, etc.). Variable renamed `target_spec` → `recipient` across 14 files. T18a–d can build directly on the existing `EffectRecipient` infrastructure.

---

#### T18a: Pipeline restructure + X value + cost assembly + rollback (DONE ✅)
- **Scope:** Medium
- **Source:** E25 (part 2 of 2, structural), E26
- **Depends on:** T17
- **Files:** `engine/cast.rs` (modify), `ui/decision.rs` (modify)
- **ATOMs:** 601.2-001, 601.2b-002, 601.2b-004, 601.2b-007, 601.2e-001, 601.2f-003, 601.2h-001, 601.5-001, 601.5-002, 107.3a-001, 107.3m-001, 602.2-002
- **Steps:**
  1. **Restructure `cast_spell` to follow 601.2a–i ordering.** The current function does legality → move → targets → pay. Reorder to:
     - **Pre-proposal check** (lightweight): timing + zone + ownership. Fail fast.
     - **601.2a:** Move to stack (existing `move_object`).
     - **601.2b — Alt/add costs:** Ask DP for alt cost choice and additional cost choices (methods from T17). Store on StackEntry. Enforce only-one-alt-cost rule (118.9a).
     - **601.2b — X value:** Add `choose_x_value(game, player_id, card_id) -> u64` to `DecisionProvider` (default: 0). Call when `card_data.mana_cost` contains an X symbol (or card has variable cost). Store in `StackEntry.x_value`. Handle `{X}{X}` costing twice the chosen X.
     - **601.2c — Targets:** Placeholder call (existing `choose_targets`). T18b will expand this.
     - **601.2d — Distribution:** Placeholder. T18c will implement.
     - **601.2e — Post-proposal legality:** Full legality check *after* 601.2a–d. On failure, roll back: move card from stack to hand, remove StackEntry, restore any state. Uses GameState snapshot (clone mutable portions before 601.2a; restore on failure — see Discrepancy §11).
     - **601.2f — Assemble total cost:** `assemble_total_cost(card_data, chosen_alt, chosen_additional, x_value) -> Vec<Cost>`. If alt cost chosen, use alt cost's `Vec<Cost>`; else use base mana cost. Concatenate additional cost mana components (from `AdditionalCost` payloads). Integrate X value into the mana cost's generic component. Apply `apply_cost_modifications()` (passthrough stub). Lock.
     - **601.2g — Mana ability window:** Add `// TODO: explicit mana ability activation window (rule 601.2g)` comment.
     - **601.2h — Pay total cost:** Call `pay_costs` with the assembled total. **Cross-ref T12:** Once mana spending restrictions exist, the cost payment step must know whether the spell being cast matches any restriction on pool mana. The `pay_costs` call will need access to the spell's characteristics (types, name, etc.) to determine which restricted mana is eligible. This is a key integration point between T12's design and this ticket — the design spike should specify the `pay_costs` API change needed.
     - **601.2i — Finalize:** Emit `SpellCast` event (existing).
  2. **Cost modification pipeline stub (E26):** Create `apply_cost_modifications(game, player_id, base_costs) -> Vec<Cost>` that currently returns `base_costs` unchanged. Document the pipeline as comments:
     ```
     // 1. Start with base mana cost (or alternative cost)
     // 2. Add additional costs
     // 3. Apply cost increases (e.g. Thalia)
     // 4. Apply cost reductions (e.g. Goblin Electromancer), player chooses order
     // 5. Apply Trinisphere-style minimum floors
     // 6. Lock final cost
     ```
     Full pipeline implementation → L15 (Phase 5 Layers).
  3. **New DP methods:** `choose_x_value`, `choose_alternative_cost`, `choose_additional_costs` — all with default implementations so existing DPs don't break.
- **Codebase verification:** Confirmed `cast.rs` has TODO at line 80 for cost modification pipeline. `check_cast_legality` at lines 222-260 runs before `move_object` (timing issue per 601.2e). No X value or alt/add cost wiring exists. `StackEntry` already has `chosen_alternative_cost`, `additional_costs_paid`, `x_value`, and `chosen_modes` fields (from T17).
- **Tests:**
  - `test_x_value_announced_via_dp` — X value flows from DP to StackEntry
  - `test_x_value_in_additional_cost` — X in kicker cost assembled correctly
  - `test_xx_cost_doubles_x` — `{X}{X}` spell with X=3 costs 6 generic
  - `test_legality_check_runs_post_proposal` — timing check passes, but post-proposal check rejects and rolls back
  - `test_rollback_restores_hand_and_stack` — failed cast returns card to hand, removes StackEntry
  - `test_cost_assembly_base_plus_additional` — kicker mana added to base cost
  - `test_cost_assembly_alt_cost_replaces_base` — alt cost substitutes for mana cost
  - `test_cost_lock_in` — assembled cost is used for payment, not raw mana cost
  - `test_cost_modification_passthrough` — stub returns base cost unchanged
  - `test_cost_phase_illegality_no_rewind` — changes during payment (601.2f–h) don't cause rollback (601.5-002)
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: restructure cast_spell to 601.2a-i, add X value, cost assembly, rollback (T18a)`

---

#### SPECIAL-1a: DP refactor — types, trait, ask functions, ScriptedDP
- **Scope:** Medium
- **Source:** DP scalability analysis (2026-04-13). Design doc: `plans/atomic-tests/supplemental-docs/decision-provider-refactor.md`
- **Depends on:** T18a (current DP methods for X/alt/additional costs exist and are tested)
- **Discovery:** Identified during T18a implementation — MtG's decision space is too large for typed DP methods. Cross-cutting refactor, not specific to the casting pipeline.
- **Rationale:** MtG requires 100+ distinct decision types when accounting for the full card pool. The current typed-methods approach would require 100+ trait methods, each propagated across 5 implementations. This refactor replaces all typed methods with 4 generic primitives (`pick_n`, `pick_number`, `allocate`, `choose_ordering`) plus a `ChoiceContext` enum for semantic labeling. Engine call sites use typed `ask_*` free functions that pack/unpack context. Adding a new decision type = 1 new `ChoiceKind` variant + 1 new `ask_*` function, zero changes to any DP implementation.
- **Compilability note:** During SPECIAL-1a the old trait methods remain on `DecisionProvider` (not deleted yet). The new 4-method trait is introduced as `GenericDecisionProvider` (or a separate trait). The old trait continues to compile. `ask_*` functions call the new trait. `ScriptedDecisionProvider` implements both traits. This keeps the codebase compiling and all existing tests passing without touching engine call sites or CLI/Random impls.
- **Files:** `ui/choice_types.rs` (create), `ui/ask.rs` (create), `ui/mod.rs` (modify), `ui/decision.rs` (add new trait + ScriptedDP new-trait impl)
- **Steps:**
  1. **Create `ui/choice_types.rs`:** Define `ChoiceKind` (enum with ~25 initial variants covering all current + planned decision types; exhaustive matching, no `#[non_exhaustive]` — see design_doc.md §11 2026-04-14 entry), `ChoiceContext` (kind only — no prompt string; display formatting belongs in DP impls), and `ChoiceOption` (enum: Object, Player, Action, AttackerTarget, BlockerAttacker, CostOption, Number, Color, CreatureType, CardName, CounterType, ManaType). `NameCard` variant on `ChoiceKind` uses `pick_number` with `CardRegistry` index for performance (see design doc §8.7).
  2. **Add new `DecisionProvider` trait (4 generic methods) in `ui/decision.rs`:** Temporarily coexists with the old trait (renamed `LegacyDecisionProvider` or kept as-is with a `// DEPRECATED` comment). New trait has: `pick_n`, `pick_number`, `allocate`, `choose_ordering`. Old trait is NOT deleted yet — that happens in SPECIAL-1c.
  3. **Create `ui/ask.rs`:** Typed free functions (`ask_choose_priority_action`, `ask_choose_attackers`, `ask_choose_blockers`, `ask_choose_targets`, `ask_choose_x_value`, `ask_choose_alternative_cost`, `ask_choose_additional_costs`, `ask_choose_generic_mana_allocation`, `ask_choose_discard`, `ask_choose_legend_to_keep`, `ask_choose_attacker_damage_assignment`, `ask_choose_trample_damage_assignment`). Each constructs `ChoiceContext` + `ChoiceOption` vec, calls the new DP trait, validates response, and returns typed result. Include validation: bounds checks, index range, allocation sum, permutation completeness.
  4. **Rewrite `ScriptedDecisionProvider`:** Replace ~12 `RefCell<Vec<_>>` queues with single `RefCell<VecDeque<ScriptedExpectation>>`. Each expectation pairs a mandatory `ChoiceKind` (discriminant-matched, fields ignored) with `ScriptedResponse` (PickN/Number/Allocation/Ordering). No `Any` fallback — every test must state what decision it expects. Add `expect_pick_n(kind, indices)`, `expect_number(kind, n)`, `expect_allocation(kind, alloc)`, `expect_ordering(kind, order)` helpers. `Drop` impl asserts queue is empty (unconsumed expectations = test bug). Panic messages include expected vs actual `ChoiceKind` for clear diagnostics. Old trait impl on ScriptedDP also kept (delegates to queues, preserving existing tests).
- **Design decisions:**
  - `ChoiceKind` uses exhaustive matching (no `#[non_exhaustive]`). Single-crate project — compiler flags every match site when a variant is added. Revisit if project becomes multi-crate.
  - Validation in `ask_*` functions: invalid DP responses panic in debug builds, preventing silent illegal states.
  - Serde derives on `ChoiceContext`/`ChoiceOption`/response types for future network serialization.
- **Tests:**
  - `test_pick_n_returns_correct_indices` — scripted pick_n returns expected selections
  - `test_pick_number_returns_value` — scripted pick_number returns expected value
  - `test_allocate_returns_distribution` — scripted allocate returns expected allocation
  - `test_choose_ordering_returns_permutation` — scripted ordering returns expected order
  - `test_ask_choose_attackers_roundtrip` — ask function packs options, DP picks, ask unpacks typed result
  - `test_ask_choose_x_value_roundtrip` — X value flows through ask→DP→result correctly
  - `test_validation_rejects_out_of_bounds` — ask function panics on invalid index
  - `test_validation_rejects_wrong_count` — ask function panics on wrong selection count
  - `test_validation_rejects_bad_allocation_sum` — ask function panics on mismatched total
  - `test_scripted_wrong_kind_panics` — expect_pick_n(DeclareAttackers) panics when engine asks ChooseTargets
  - `test_scripted_unconsumed_panics` — leftover expectations in queue panic on Drop
  - `test_scripted_empty_queue_panics` — DP call with empty queue panics with descriptive message
  - All existing tests pass unchanged (old trait still works)
- **Acceptance:** All existing tests pass + new unit tests pass + 0 warnings
- **Commit:** `ui: add 4-primitive DecisionProvider trait, ChoiceKind, ask_* functions, ScriptedDP (SPECIAL-1a)`

---

#### SPECIAL-1b: DP refactor — CLI, Random, Dispatch implementations
- **Scope:** Small
- **Depends on:** SPECIAL-1a (new trait + choice types exist)
- **Files:** `ui/cli.rs` (rewrite impl), `ui/random.rs` (rewrite impl), `ui/decision.rs` (DispatchDP rewrite)
- **Steps:**
  1. **Rewrite `CliDecisionProvider`:** Implement new 4-method trait. `pick_n` matches on `ChoiceKind` for prompt formatting, shows options + bounds, reads indices. `pick_number` shows range, reads number. `allocate` shows buckets, reads values. `choose_ordering` shows items, reads permutation. Old trait impl kept temporarily (delegates to new methods where possible).
  2. **Rewrite `RandomDecisionProvider`:** Implement new 4-method trait. `pick_n` shuffles + truncates. `pick_number` picks in range. `allocate` distributes randomly. `choose_ordering` shuffles. Old trait impl kept temporarily.
  3. **Rewrite `DispatchDecisionProvider`:** Forward all 4 new-trait methods by player_id index. Old trait forwarding kept temporarily.
- **Design decisions:**
  - `choose_priority_action` becomes `ask_choose_priority_action` which enumerates legal actions via oracle helpers and calls `pick_n(bounds: (1,1))`. The DP no longer needs its own oracle calls for priority. CLI/Random `choose_priority_action` old-trait impls remain until SPECIAL-1c migrates engine call sites.
  - **RandomDP `pick_number` for `ChooseXValue`:** `ask_choose_x_value` passes `(0, u64::MAX)` — affordability is enforced by casting pipeline rollback, not by the ask function. The Random DP must NOT blindly pick in that range. Instead, it should inspect `GameState` (mana pool + known mana sources) to self-compute a reasonable upper bound for X, then pick within `[0, reasonable_max]`. This keeps the `ask_*` API clean (no `affordable_hint` parameter) while avoiding degenerate rollback loops in fuzz testing.
- **Tests:**
  - `test_dispatch_routes_by_player` — DispatchDP forwards to correct inner DP via new trait
  - Fuzz harness 200/200 — RandomDP via new trait produces legal games
  - All existing tests still pass (old trait impls still present)
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings + fuzz harness 200/200
- **Commit:** `ui: implement 4-primitive trait for CLI, Random, Dispatch DPs (SPECIAL-1b)`

---

#### SPECIAL-1c: DP refactor — engine migration + old trait deletion
- **Scope:** Medium (mechanical but wide-reaching)
- **Depends on:** SPECIAL-1b (all DP impls have new trait)
- **Files:** `ui/decision.rs` (delete old trait), `engine/cast.rs` (modify call sites), `engine/priority.rs` (modify), `engine/combat/steps.rs` (modify), `engine/sba.rs` (modify), `engine/costs.rs` (modify), `engine/resolve.rs` (modify), all test files (update scripted DP usage)
- **Steps:**
  1. **Update all engine call sites:** Replace `decisions.choose_X(...)` with `ask_choose_X(decisions, ...)`. Touch: `cast.rs`, `priority.rs`, `combat/steps.rs`, `sba.rs`, `costs.rs`, `resolve.rs`. Engine functions change parameter type from `&dyn DecisionProvider` (old) to `&dyn DecisionProvider` (new — same name, different trait after deletion).
  2. **Update all tests:** Mechanical replacement of scripted DP setup from old queue-per-method style to `expect_pick_n(kind, response)` style. Every test now uses explicit `ChoiceKind` expectations.
  3. **Delete old trait:** Remove all old trait method signatures and their impls from Scripted/CLI/Random/Dispatch. The old trait ceases to exist. Only the 4-method trait remains, named `DecisionProvider`.
  4. **Clean up:** Remove any shim/adapter code from SPECIAL-1a/1b. Ensure `ui/decision.rs` exports only the final clean trait.
- **Tests:**
  - All existing integration tests pass with updated scripted DP calls
  - All existing unit tests pass with updated scripted DP calls
  - Game logic completely unaffected — only the DP calling convention changed
- **Acceptance:** All existing tests pass (with new-style scripted DP calls) + 0 warnings + fuzz harness 200/200 + no references to old trait remain in codebase
- **Commit:** `ui: migrate engine to 4-primitive DP, delete legacy trait (SPECIAL-1c)`

---

#### SPECIAL-3: Shared test helpers for common DP script sequences
- **Scope:** Small
- **Source:** SPECIAL-1c audit (2026-04-15)
- **Depends on:** SPECIAL-1c (test migration complete)
- **Files:** `src/ui/scripted.rs` (create — or `tests/common/scripted.rs`), `ui/decision.rs` (modify), test files (modify imports)
- **Steps:**
  1. Extract `ScriptedDecisionProvider` and helpers like `queue_empty_turn_passes` from `ui/decision.rs` into a dedicated `src/ui/scripted.rs` module. `decision.rs` stays clean as the trait definition + production helpers only.
  2. Add common sequence helpers alongside `queue_empty_turn_passes`: `queue_turn_passes_with_no_attacks` (16 passes + 1 empty DeclareAttackers), `queue_cast_and_resolve(spell_index)` (pick spell + 2 passes to resolve), and any other recurring patterns identified during migration.
  3. Update all test imports to use the new module location.
- **Tests:** All existing tests pass unchanged after extraction.
- **Acceptance:** All tests pass + 0 warnings + `decision.rs` contains no test-only infrastructure
- **Commit:** `ui: extract ScriptedDecisionProvider and test helpers into scripted.rs (SPECIAL-3)`

---

#### SPECIAL-4: CounterSpell/CounterAbility cleanup — use move_object
- **Scope:** Small
- **Source:** SPECIAL-1c audit (2026-04-15), phase2_integration_test.rs review
- **Depends on:** none
- **Files:** `engine/resolve.rs` (modify)
- **Steps:**
  1. **`Primitive::CounterSpell`:** Replace the manual `stack.remove()` + `stack_entries.remove()` + `graveyard.push()` + `obj.zone = Graveyard` + manual `ZoneChange` event emission with a single `move_object(countered_id, Zone::Graveyard)` call. `move_object` already handles stack cleanup, zone field update, zone-change events, and `cleanup_zone_state`. Keep the `SpellCountered` event emission after the move.
  2. **`Primitive::CounterAbility`:** Replace the manual `stack.remove()` + `stack_entries.remove()` + `objects.remove()` with `move_object` to a sink zone (or direct removal if abilities have no destination zone). Review whether `move_object` handles ability objects correctly — abilities on the stack are not cards and should cease to exist, not go to graveyard. If `move_object` doesn't support this, add a `remove_from_stack(id)` helper that consolidates the cleanup. Keep `AbilityCountered` event emission.
  3. **Test:** Add `test_counter_creature_spell_mid_resolution` — counter a creature spell, verify it goes to graveyard (not battlefield), stack is clean, no dangling `stack_entries`.
- **Tests:**
  - All existing counter-related tests pass unchanged
  - `test_counter_creature_spell_mid_resolution` — new regression test
- **Acceptance:** All tests pass + 0 warnings + no manual stack/zone manipulation in CounterSpell/CounterAbility
- **Commit:** `engine: CounterSpell/CounterAbility use move_object instead of manual stack manipulation (SPECIAL-4)`

---

#### SPECIAL-5: DP validation + contract property tests (Classes A + D)
- **Scope:** Small
- **Source:** `plans/test-strategy-post-dp-refactor.md` §3 (Classes A, D), SPECIAL-1c follow-up
- **Depends on:** SPECIAL-1c (4-primitive trait is the canonical DP surface)
- **Discovery:** The DP refactor made invalid-response testing tractable for the first time. Without a dedicated validation/contract suite, bugs in `validate_*` or `RandomDecisionProvider` will surface as confusing integration failures downstream. Cheap to write, high leverage.
- **Files:** `src/ui/ask.rs` (extend tests module), `src/ui/random.rs` (extend tests module), `tests/dp_contract_test.rs` (create — optional, or keep inline)
- **Steps:**
  1. **Class A — `validate_allocation` negatives (`#[should_panic]` or `Result::Err` assertions):** sum != total; bucket below `per_bucket_mins`; bucket above `per_bucket_maxs`; wrong bucket count. Some already exist (trample tests) — backfill the rest and keep them colocated in `ask.rs::tests`.
  2. **Class A — `pick_n` bound checks:** index out of `options.len()` range → panic via `ask_*` wrapper; count < min or > max → panic; repeated index when uniqueness required. Assert via a minimal `ScriptedDecisionProvider` returning a crafted bad response.
  3. **Class A — `pick_number` bound checks:** result < min or > max → panic.
  4. **Class A — `choose_ordering`:** non-permutation response (duplicate or missing index) → panic.
  5. **Class D — `RandomDecisionProvider` contract property tests:** run 200 iterations each of `pick_n`, `pick_number`, `allocate`, `choose_ordering` with randomized valid inputs (random option counts, random bounds, random per-bucket mins/maxs). Assert every response satisfies the primitive's contract. Use a seeded RNG for determinism.
  6. **Fuzz harness assertion:** add a debug-mode contract check inside `ask_*` wrappers (or a post-call `validate_*` call) so the existing 200-game fuzz harness also exercises the contracts on every DP response in realistic game states.
- **Tests added:** ~12–15 total (5 Class A + 4 Class A for pick_n/number/ordering + 3 Class D property + 2 regression from fuzz if any surface).
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings + fuzz harness 200/200.
- **Commit:** `ui: add DP validation and contract property tests (SPECIAL-5)`

---

#### SPECIAL-6: `ask_*` option enumeration tests (Class C, living ticket)
- **Scope:** Small–Medium (initial landing), living thereafter
- **Source:** `plans/test-strategy-post-dp-refactor.md` §3 Class C, SPECIAL-1c follow-up
- **Depends on:** SPECIAL-1c
- **Discovery:** The `ask_*` layer now builds `ChoiceOption` lists from game state before any DP implementation sees them. This is the correct place to assert "the engine presents the right options." These tests catch a category of bug (wrong options enumerated) that DP-level sequence tests fundamentally cannot see. Highest long-term leverage as `SelectionFilter`/keyword/type surface grows (Phases 5–8).
- **Files:** `src/ui/ask.rs` (extend tests module), or `tests/ask_enumeration_test.rs` (create if test module gets too large)
- **Guiding principle:** one enumeration test per `ask_*` function per non-trivial enumeration branch. Set up game state, invoke the `ask_*` wrapper with a `ScriptedDecisionProvider` returning index 0 (content-agnostic), and **assert the `options` the engine offered** — either by intercepting via a capturing DP or by indirect verification (e.g., assert the DP was not asked when no legal option exists).
- **Initial scope (landing PR):**
  1. `ask_choose_attackers` — only untapped, non-summoning-sick, non-defender creatures appear; vigilance creatures still listed; haste creatures appear even when newly ETB'd; creatures controlled by non-active players excluded.
  2. `ask_choose_blockers` — only untapped creatures of defending player appear; tapped creatures excluded; flying attackers filter blocker candidates per-pair (post-SPECIAL-1c §15c fix if applied).
  3. `ask_choose_priority_action` — Pass always first; PlayLand appears only in main phase with land drop remaining and empty stack; CastSpell appears only when affordable; ActivateAbility appears only for abilities whose costs are payable.
  4. `ask_select_objects` — `SelectionFilter::Creature` excludes non-creatures; `SelectionFilter::Permanent(PermanentFilter::BySubtype(...))` filters correctly; `SelectionFilter::Player` enumerates all players including controller.
  5. `ask_choose_legend_to_keep` — only duplicates listed; singleton legends never trigger the ask.
- **Living extension rule:** every future ticket that adds or modifies an `ask_*` function, a `ChoiceKind` variant, a `SelectionFilter` variant, or a `PermanentFilter` variant must add at least one Class C test in the same PR. This is added to Testing Conventions (see §Testing Conventions).
- **Tests added:** ~8–12 initially. Expected to grow with each phase.
- **Acceptance:** All existing tests pass + new enumeration tests pass + 0 warnings + Testing Conventions updated to require Class C coverage for future `ask_*` changes.
- **Commit:** `ui: add ask_* option enumeration tests (SPECIAL-6)`

---

#### T18b: Mode choice + conditional targets + target rules
- **Scope:** Medium
- **Source:** E25 (part 2 of 2, mode/target features)
- **Depends on:** SPECIAL-1c (DP refactor complete; mode choice adds `ChoiceKind::ChooseModes` + `ask_choose_modes`)
- **Files:** `engine/cast.rs` (modify), `ui/ask.rs` (modify), `ui/choice_types.rs` (modify), `engine/targeting.rs` (modify)
- **ATOMs:** 601.2b-001, 601.2b-003, 601.2c-002, 601.2c-003, 601.2c-004, 601.2c-006, 601.2c-007, 601.4-001, 115.3-001, 115.3-002, 115.3/4-001, 115.3/4-002, 115.6-001, 604.5-001, 604.5-002, 604.6-001, 604.6-002
- **Steps:**
  1. **Mode choice via DP:** Add `ChoiceKind::ChooseModes { min, max }` variant to `ChoiceKind` and `ask_choose_modes` free function in `ui/ask.rs`. Call when spell's effect is `Effect::Modal`. Store in `StackEntry.chosen_modes`. Validate count matches `ModalCount` (exactly N, up to N, any number if kicked — 601.4 look-ahead).
  2. **Conditional targets:** Update target-choosing step to pass `chosen_modes` and `additional_costs_paid` so conditional targets can be included/excluded. A kicker spell with "if kicked, target creature" should not require a target when unkicked, and should require one when kicked. A modal spell only requires targets for chosen modes.
  3. **Target uniqueness per instance (rule 115.3):** Enforce that a single "target [type]" instance can't choose the same object twice (sad path). But *different* target instances can share a target (Decimate targeting an artifact creature for both "target artifact" and "target creature").
  4. **"Up to N" targets (rule 115.6):** When `TargetCount::UpTo(n)`, allow 0 targets chosen. Spell still resolves (it just does nothing to the absent targets).
  5. **All-targets-legal precondition:** For spells with multiple required target slots (like Decimate), verify in the pre-proposal check that at least one legal choice exists for each slot. If not, the spell can't be cast. This is the rule behind Decimate's reminder text: *(You can't cast this spell unless you have legal choices for all its targets.)*
  6. **601.4 intra-step look-ahead:** When choosing modes in 601.2b, the DP may consider kicker intent (additional cost choice happens in the same step). Implementation: pass available additional costs to `choose_modes` so the DP can factor them in.
  7. **Per-atom targeting for multi-recipient sequences (replaces first-atom-only extraction):** The current `cast_spell` extracts `recipient` from the first `Effect::Atom` in a `Sequence`. This is wrong: a spell like "You draw two cards. Target opponent discards two cards" has `Controller` on atom 0 and `Target` on atom 1, causing the target prompt to be skipped entirely (L127 bug, see Discrepancy §15). **Fix:** Scan ALL atoms in an `Effect::Sequence`, collect a `Vec<(atom_index, EffectRecipient)>` of targeting requirements. For each `Target`/`Choose` recipient, enumerate legal selections and prompt the DP. Store per-atom target sets on `StackEntry` (new field: `pub targets_per_atom: Vec<Vec<ResolvedTarget>>`). Resolution reads from the appropriate slot. Atoms with `Implicit`/`Controller` get empty target lists.
  8. **Optional-cost-dependent targeting:** Alternative costs and additional costs (kicker, entwine) can add or change targeting requirements on a spell. A spell with "if kicked, target creature gets +2/+2" has no target when unkicked but gains one when kicked. **This means the target-collection step (601.2c) must run AFTER cost choices (601.2b) and must consult `chosen_modes`, `chosen_alt_cost`, and `chosen_additional_costs` to determine which atoms require targets.** Similarly, an alt cost that changes the spell's effect (e.g. overload replaces "target" with "each") can remove targeting requirements entirely. The per-atom scan from step 7 must be filtered through the cost/mode selection.
- **Tests:**
  - `test_mode_choice_via_dp` — modal spell stores chosen modes on StackEntry
  - `test_modal_single_mode_resolves_correctly` — chosen mode's effect runs, others don't
  - `test_only_one_alt_cost_per_spell` — attempting two alt costs rejected (118.9a)
  - `test_conditional_target_present_when_kicked` — kicker spell requires target when kicked
  - `test_conditional_target_absent_when_not_kicked` — unkicked spell has no target requirement
  - `test_kicker_changes_target_legality` — Bloodchief's Thirst: kicked removes CMC restriction
  - `test_target_uniqueness_same_instance` — same "target creature" can't pick same creature twice
  - `test_target_shared_across_instances` — artifact creature can be "target artifact" and "target creature"
  - `test_up_to_zero_targets_resolves` — "up to 2 targets" with 0 chosen → spell resolves (no-op)
  - `test_modal_plus_kicker_look_ahead` — Inscription of Abundance: multiple modes allowed when kicker intended
  - `test_all_targets_must_be_legal_to_cast` — Decimate can't be cast without legal targets for all 4 slots
  - `test_per_atom_targeting_sequence` — "You draw 2. Target opponent discards 2" — Controller atom has no target, Target atom prompts for opponent
  - `test_first_atom_controller_second_atom_target` — L127 regression: first atom is Controller, second is Target — target selection not skipped
  - `test_kicker_adds_target_requirement` — unkicked spell has 0 targets, kicked spell prompts for 1
  - `test_alt_cost_changes_targeting` — overload alt cost removes targeting (targets all instead)
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: mode choice, conditional targets, target uniqueness rules (T18b)`

---

#### T18c: Distribution + Cost::Sacrifice + payment ordering + partial resolution
- **Scope:** Medium
- **Source:** E25 (part 2 of 2, distribution), rules 601.2d, 601.2h, 608.2b/d/i
- **Depends on:** SPECIAL-1c (DP refactor complete). Parallel with T18b and T18d.
- **Files:** `engine/cast.rs` (modify), `engine/costs.rs` (modify), `engine/resolve.rs` (modify), `ui/ask.rs` (modify), `ui/choice_types.rs` (modify)
- **ATOMs:** 601.2d-001, 601.2d-002, 601.2h-003, 608.2b-002, 608.2b-005, 608.2d-002, 608.2d-003, 608.2i-001, 118.6-001
- **Steps:**
  1. **Implement `Cost::Sacrifice(PermanentFilter, u32)` in `costs.rs`:** Currently returns `"not yet implemented"` in both `check_cost_resource` and `pay_single_cost`. Implement:
     - `check_cost_resource`: verify player controls ≥ N permanents matching the filter.
     - `pay_single_cost`: call `choose_sacrifice` on DP, validate choices match filter + count, then `move_object` each to graveyard.
     - Add `ChoiceKind::ChooseSacrifice` variant and `ask_choose_sacrifice` free function in `ui/ask.rs`. This makes `AdditionalCost::Kicker(vec![Cost::Sacrifice(Creature, 1)])` fully functional.
  2. **Distribution via DP:** Add `ChoiceKind::ChooseDistribution` variant and `ask_choose_distribution` free function in `ui/ask.rs`. Uses `allocate` DP method. Call when spell has a divisible effect (e.g. `Primitive::DealDamage` with `AmountExpr::Divide`). Store distribution on StackEntry (new field: `pub distribution: Vec<(ResolvedTarget, u32)>`).
  3. **Distribution validation:** Each target must receive ≥1 of the divided amount. Total must equal the full amount. Reject and re-prompt (or error) on violation.
  4. **Cost payment ordering via DP:** Add `ChoiceKind::CostPaymentOrder` variant and `ask_choose_cost_payment_order` free function in `ui/ask.rs`. Uses `choose_ordering` DP method. The order matters when sacrifice-as-cost interacts with mana costs (ATOM-601.2h-003: Omnath + Momentous Fall). Sort deterministic costs (sacrifice, tap, life) before random/library costs per 601.2h, then let DP choose order within each category.
  5. **Partial-target resolution (608.2b):** When resolving a multi-target spell where some targets have become illegal, resolve the remaining targets normally. Do not fizzle the whole spell unless *all* targets are illegal. Update `resolve_effect` to skip effects whose target is gone but continue with effects for legal targets. Test with Decimate pattern (4 independent targets, one becomes illegal → other 3 still destroy) and Jagged Lightning / Plague Spores patterns.
  6. **Resolution-time untargeted distribution (608.2d/608.2i):** For effects that distribute at resolution rather than cast time (e.g., "choose how to distribute N damage among any number of targets"), call DP at resolution. Historical look-back (608.2i): effects referencing "the amount of damage dealt" use the cast-time locked value, not a recalculated one.
- **Test cards:** Decimate ({2}{R}{G} Sorcery — "Destroy target artifact, target creature, target enchantment, and target land"), Arc Lightning ({2}{R} Sorcery — "Deal 3 damage divided as you choose among one, two, or three targets"), Altar's Reap ({1}{B} Instant — "As an additional cost, sacrifice a creature. Draw two cards"), Plague Spores ({4}{B}{R} Sorcery — "Destroy target nonblack creature and target land. They can't be regenerated").
- **Tests:**
  - `test_distribution_choice_via_dp` — Arc Lightning distributes 2+1 among two targets
  - `test_distribution_zero_rejected` — distributing 3+0+0 among 3 targets rejected
  - `test_distribution_sum_matches_total` — 2+2 for a "deal 3" effect rejected
  - `test_sacrifice_as_cost_implemented` — Altar's Reap sacrifices creature, draws 2
  - `test_sacrifice_as_cost_no_legal_target` — can't cast Altar's Reap with no creatures
  - `test_cost_payment_ordering` — Momentous Fall: sacrifice-first vs mana-first produces different outcomes
  - `test_partial_target_resolution` — Decimate: one target illegal → other 3 resolve
  - `test_resolution_time_distribution` — distribution chosen at resolution for untargeted divide
  - `test_historical_look_back` — 608.2i: damage reference uses locked-in value
  - `test_unpayable_mana_cost_fails` — spell with no way to pay mana cost is rejected (118.6)
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: distribution, Cost::Sacrifice, payment ordering, partial resolution (T18c)`

---

#### T18d: Casting restrictions + no-mana-cost guard + legendary sorcery
- **Scope:** Small
- **Source:** E27 + E28
- **Depends on:** T18a (post-proposal legality check must exist), SPECIAL-1c (if any DP calls needed). Parallel with T18b and T18c.
- **Files:** `engine/cast.rs` (modify), `objects/card_data.rs` (modify)
- **ATOMs:** 202.1b-001, 205.4e-001, 205.4e-002, 118.9a-001
- **Steps:**
  1. **No-mana-cost guard (E27):** In the pre-proposal legality check, if `card_data.mana_cost.is_none()` and no alternative cost is being offered, return Err. Note: per rule 601.3a, if an alt cost *exists* on the card, the player may begin casting even if the base cost is None.
  2. **Legendary sorcery restriction (E28):** In `check_cast_legality`, if card has `Supertype::Legendary` and any type in `{Instant, Sorcery}`, check that the caster controls a legendary creature or planeswalker. If not, return Err.
  3. **Casting restrictions (E28 extended):** Add `CastingRestriction` enum to `objects/card_data.rs`:
     ```rust
     pub enum CastingRestriction {
         OnlyDuringCombat,
         OnlyBeforeCombatDamageStep,
         OnlyDuringYourTurn,
         OnlyIfCondition(CastCondition),
     }
     pub enum CastCondition {
         YouCastAnotherSpellThisTurn,
         YouControlCreatureWithPowerGE(i32),
         YouControlPermanentWithType(CardType),
         Custom(String), // fallback for complex conditions
     }
     ```
     Add `pub casting_restrictions: Vec<CastingRestriction>` to `CardData`. These are self-imposed restrictions like Berserk's "Cast only before the combat damage step" or Illusory Angel's "Cast only if you've cast another spell this turn." Enforce in the post-proposal legality check. **Note:** These are distinct from T19's `ActivationRestriction` (which applies to activated abilities, not spells).
  4. **Mana ability window TODO (601.2g):** If not already added by T18a, ensure the `// TODO: explicit mana ability activation window (rule 601.2g)` comment exists in the pipeline.
- **Tests:**
  - `test_no_mana_cost_card_rejected` — card with mana_cost: None can't be cast normally
  - `test_no_mana_cost_with_alt_cost_allowed` — card with alt cost can be cast despite no base mana cost
  - `test_legendary_sorcery_needs_legendary_permanent` — rejected without legendary creature/PW
  - `test_legendary_sorcery_allowed_with_legendary` — allowed with legendary creature
  - `test_casting_restriction_only_during_combat` — Berserk-style restriction enforced
  - `test_casting_restriction_condition` — Illusory Angel-style "cast another spell" condition
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: casting restrictions, no-mana-cost guard, legendary sorcery (T18d)`

---

#### T19: Activation restrictions + zone-activated abilities
- **Scope:** Medium
- **Source:** E29 + E33
- **Depends on:** none
- **Files:** `objects/card_data.rs` (modify), `engine/cast.rs` (modify), `engine/priority.rs` (modify), `oracle/mana_helpers.rs` (modify)
- **Note:** This ticket covers restrictions on **activated abilities** (rule 602), NOT self-imposed spell casting restrictions like Berserk's "Cast only before the combat damage step." Spell casting restrictions are handled by `CastingRestriction` on `CardData` in T18d. The two concepts are parallel:
  - `ActivationRestriction` → on `AbilityDef` → checked in `activate_ability`
  - `CastingRestriction` → on `CardData` → checked in `check_cast_legality`
- **Steps:**
  1. Add `pub activation_restrictions: Vec<ActivationRestriction>` to `AbilityDef`. Define enum:
     ```rust
     pub enum ActivationRestriction {
         OncePerTurn,
         SorcerySpeed,
         OnlyDuringYourTurn,
         OnlyDuringCombat,
     }
     ```
  2. Add `pub activation_zone: Zone` to `AbilityDef` (default: `Zone::Battlefield`). Import `Zone`.
  3. Update `CardDataBuilder` or `AbilityDef` construction sites to default `activation_restrictions: Vec::new()` and `activation_zone: Zone::Battlefield`.
  4. In `engine/cast.rs` `activate_ability`, enforce `activation_restrictions`: check `OncePerTurn` (needs per-turn activation tracking on `GameState` — `HashMap<(ObjectId, AbilityId), u32>` tracking activations this turn, cleared in cleanup step), check `SorcerySpeed` (active player, main phase, empty stack), etc.
  5. In `engine/cast.rs` `activate_ability` and `engine/priority.rs`, check `activation_zone` instead of assuming battlefield. If `activation_zone == Zone::Hand`, look for the object in hand; if `Zone::Graveyard`, look in graveyard.
  6. Update `oracle/mana_helpers.rs` `activatable_abilities` to check abilities across all zones, not just battlefield.
- **Codebase verification:** Confirmed `AbilityDef` at `card_data.rs:53-59` has no `activation_restrictions` or `activation_zone` field.
- **Tests:**
  - `test_sorcery_speed_restriction_blocks_on_opponent_turn` — ability with SorcerySpeed can't activate at instant speed
  - `test_once_per_turn_restriction` — second activation in same turn fails
  - `test_once_per_turn_resets_on_new_turn` — counter resets at cleanup
  - `test_activation_from_graveyard` — ability with activation_zone: Graveyard activates from GY
  - `test_activation_from_hand` — ability with activation_zone: Hand activates from hand
  - `test_activation_from_wrong_zone_fails` — battlefield ability can't activate from graveyard
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: add activation restrictions and zone-activated abilities (E29, E33)`

---

#### T20: Linked abilities + mana ability debug assertion
- **Scope:** Small
- **Source:** E30 + E32
- **Depends on:** none
- **Files:** `objects/card_data.rs` (modify), `state/game_state.rs` (modify), `engine/resolve.rs` (modify)
- **Steps:**
  1. **Linked abilities (E30):** Add `pub linked_group: Option<u32>` to `AbilityDef`. Add `pub linked_ability_data: HashMap<(ObjectId, u32), Vec<ObjectId>>` to `GameState`. When an ability with a linked group resolves and affects objects, record them in the map. The paired ability reads from the map.
  2. **Mana ability debug assertion (E32):** In `CardDataBuilder::build()`, add `#[cfg(debug_assertions)]` check: if any ability has `ability_type == Mana` and its effect uses `EffectRecipient::Target(_, _)` (mana abilities cannot target per rule 605.1a), emit a warning via `eprintln!` or `debug_assert!`. *(Note: `TargetSpec` was refactored to `EffectRecipient` in the T15b MR — `Implicit` and `Controller` are non-targeting, `Target` is targeting.)*
- **Codebase verification:** Confirmed `AbilityDef` has no `linked_group`. `CardDataBuilder::build()` at `card_data.rs:223-225` does no validation.
- **Tests:**
  - `test_linked_ability_data_stored` — resolve linked ability, data recorded
  - `test_linked_ability_data_read` — paired ability reads stored data
  - `test_mana_ability_debug_assertion` — (debug only) mana ability with targets triggers assertion
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: add linked abilities and mana ability debug check (E30, E32)`

---

#### T20b: Last Known Information (LKI) system — DEFERRED TO PART 2
- **Source:** E31
- **Rationale:** LKI snapshots must capture effective (post-layer) characteristics per rule 608.2h. Defining an interim `LKISnapshot` with inlined fields here would create throwaway work — the struct is nearly identical to `EffectiveCharacteristics` (which Part 2 creates in §5c). Implementing LKI alongside `EffectiveCharacteristics` in Part 2 avoids the build-then-replace anti-pattern.
- **What Part 2 will do:** Define `LKISnapshot` as a wrapper around `EffectiveCharacteristics` + `owner`, `zone`, `counters`, `card_data`. Hook `move_object` to snapshot before zone change. Provide `query_lki()`. Wire into `resolve_primitive` for dead-source lookups.
- **Design analysis preserved:** The full LKI deep dive (when LKI is used, snapshot lifetime decision, phased approach, complexity concerns) is documented in this ticket's git history and should be carried forward into the Part 2 ticket.
- **No Part 1 ticket depends on T20b.**

---

#### T12c: Mana restrictions — engine integration
- **Scope:** Medium
- **Source:** E15 (implementation phase B from `plans/mana-restrictions-design.md` §10)
- **Depends on:** T12b, T17
- **Files:** `engine/costs.rs` (modify), `engine/cast.rs` (modify), `engine/mana.rs` (modify), `ui/decision.rs` (modify), `oracle/mana_helpers.rs` (modify), `types/effects.rs` (modify)
- **Steps:**
  1. Add `ManaProducedMeta` to `ManaOutput`. Default `special: None`.
  2. Update `resolve_mana_effect` to route special mana to `add_special`.
  3. Add `ManaPaymentPlan::simple_only(HashMap<ManaType, u64>)` constructor.
  4. Change `pay_costs` signature to take `Option<&ManaPaymentPlan>` instead of `&HashMap`. Update all call sites to use `Some(&ManaPaymentPlan::simple_only(...))`. Mana-free costs pass `None`.
  5. Add `build_default_payment_plan` to `ui/decision.rs`.
  6. Replace `choose_generic_mana_allocation` with `choose_mana_payment` on `DecisionProvider`. All existing DPs delegate to `build_default_payment_plan`. **Trait method count stays at 8** (replace, not add).
  7. Update `castable_spells` to use `can_pay_with_context`. Build `SpendContext` from `oracle::characteristics` (effective types, not raw `card_data`).
  8. Deprecate `can_pay()` and `pay()` with `#[deprecated]`.
- **Design reference:** `plans/mana-restrictions-design.md` §5 (Integration Points)
- **Tests:**
  - `test_pay_costs_with_mana_plan` — `pay_costs` with `ManaPaymentPlan` end-to-end
  - `test_pay_costs_backward_compat` — `simple_only` works like old `generic_allocation`
  - `test_build_default_payment_plan_no_special` — identical to `auto_allocate_generic`
  - `test_build_default_payment_plan_prefers_special` — eligible restricted mana used first
- **Acceptance:** All existing tests pass (with updated call sites) + new tests pass + 0 warnings + `#[deprecated]` on old methods
- **Commit:** `mana: integrate restriction-aware payment into engine (T12c)`

---

### Tier 5: Zone, Combat, Damage, Targeting (T21a–T22)

#### T21a: Zone guards + CastInfo carried to permanent
- **Scope:** Small
- **Source:** E34 + E35
- **Depends on:** T17 (CastInfo uses AdditionalCost)
- **Files:** `engine/zones.rs` (modify), `state/battlefield.rs` (modify), `engine/stack.rs` (modify)
- **Steps:**
  1. **Instant/Sorcery battlefield guard (E34):** In `engine/zones.rs` `move_object` (or `init_zone_state`), if the destination is `Battlefield` and the object's types include `Instant` or `Sorcery` (and no permanent type), skip the move and return Ok (the card stays in its current zone). Add a comment citing rules 304.4 and 307.4.
  2. **CastInfo carried to permanent (E35):** Define `CastInfo` struct in `state/battlefield.rs`:
     ```rust
     pub struct CastInfo {
         pub mana_spent: Option<ManaCost>,
         pub x_value: Option<u32>,
         pub additional_costs_paid: Vec<AdditionalCost>,
         pub was_cast_at_non_sorcery_speed: bool, // 307.5a — set at cast time: !(active_player && main_phase && stack_empty). Used by Necromancy-style cards.
     }
     ```
     Add `pub cast_info: Option<CastInfo>` to `BattlefieldEntity` (init `None`). In `engine/stack.rs` permanent resolution, populate from `StackEntry`.
- **Codebase verification:** Confirmed no instant/sorcery battlefield guard exists. No `CastInfo` struct.
- **Tests:**
  - `test_instant_cannot_enter_battlefield` — instant returned from GY to battlefield stays in GY
  - `test_sorcery_cannot_enter_battlefield` — sorcery returned from GY to battlefield stays in GY
  - `test_cast_info_carried_to_permanent` — kicked creature has cast_info.additional_costs_paid
  - `test_cast_info_none_for_noncasted` — permanent entering without casting has cast_info = None
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: instant/sorcery battlefield guard, CastInfo on permanent (E34, E35)`

---

#### T21b: Combat removal + evasion framework + trample co-assigned damage
- **Scope:** Medium
- **Source:** E36 + E37 + E38
- **Depends on:** none
- **Files:** `engine/combat/validation.rs` (modify), `engine/combat/keywords.rs` (modify), `engine/combat/resolution.rs` (modify), `engine/combat/steps.rs` (modify), `types/keywords.rs` (modify)
- **Steps:**
  1. **Combat removal on control/type change (E36):** Add a helper `remove_from_combat(game, object_id)` that clears `attacking`/`blocking` on the entity and removes it from any attacker's `blocked_by` list. Call it when control changes, when the permanent stops being a creature, or when it phases out. For now, add the helper and call it from the control-change site (which doesn't exist yet — stub with a TODO). The combat removal itself is the deliverable.
  2. **Evasion framework expansion (E37):** Per rule 509.1b, blocking restrictions are evasion abilities on the attacker that constrain what can block it. They fall into three validation categories:

     **Category A — Per-pair checks** (in `can_block(attacker, blocker)`, checked for each attacker–blocker pair):
     - `Flying` (702.9b): blocker must have flying or reach *(already implemented)*
     - `Shadow` (702.28b): blocker must have shadow; also, non-shadow attackers can't be blocked by shadow creatures (bidirectional)
     - `Fear` (702.36b): blocker must be artifact creature or black
     - `Intimidate` (702.13b): blocker must be artifact creature or share a color with attacker
     - `Skulk` (702.118b): blocker must have power ≤ attacker's power
     - `Horsemanship` (702.31b): blocker must have horsemanship
     - `Protection` (702.16f): blocker can't have the protected-from quality *(targeting part in T22, blocking part here)*

     **Category B — Per-pair contextual checks** (still per-pair, but check the defending player's board state, not just the blocker):
     - `Landwalk` (702.14c): attacker with e.g. islandwalk can't be blocked at all *as long as the defending player controls an Island*. This is a per-attacker unblockability check, not a per-blocker filter. If the defending player controls the relevant land type, no creature can block the attacker. Landwalk variants are parameterized: `KeywordAbility::Landwalk(LandwalkType)` where `LandwalkType` enumerates basic land types, supertypes (nonbasic), etc.

     **Category C — Count-based checks** (in `validate_full_block_assignment`, after all blockers are declared, checking the aggregate assignment):
     - `Menace` (702.111b): attacker must be blocked by ≥2 creatures or be unblocked. This is validated on the final blocking assignment, not per-pair.
     - Future "super-menace" effects (e.g., "can't be blocked except by three or more creatures") fit this same category — parameterize as a minimum blocker count.

     **Note:** Blocking requirements (509.1c, "must block if able") are a distinct concern — see Discrepancy §6 for the architectural note.

     Add `Shadow`, `Fear`, `Skulk`, `Horsemanship`, `Intimidate` variants to `KeywordAbility` enum. Add `Landwalk(LandwalkType)` parameterized variant. Define `LandwalkType` enum (Plains, Island, Swamp, Mountain, Forest, Nonbasic, Snow, etc.).
  3. **Trample co-assigned damage (E38):** In `engine/combat/keywords.rs` `assign_trample_damage`, accept a `pre_assigned_damage: u64` parameter representing damage already assigned to the blocker by other attackers. Subtract from the blocker's remaining toughness before calculating excess. Update `engine/combat/resolution.rs` call site to pass accumulated damage.
- **Codebase verification:** Confirmed combat validation only checks Flying/Reach evasion. `KeywordAbility` at `keywords.rs:7-30` is missing Shadow, Fear, Skulk, Horsemanship.
- **Tests:**
  - `test_remove_from_combat_helper` — creature removed from combat has no attacking/blocking
  - `test_menace_requires_two_blockers` — single blocker can't block menace creature
  - `test_menace_unblocked_ok` — menace creature with zero blockers is legal
  - `test_shadow_evasion` — non-shadow can't block shadow; shadow can't block non-shadow
  - `test_fear_evasion` — non-black non-artifact can't block fear creature
  - `test_intimidate_evasion` — must be artifact or share color
  - `test_skulk_evasion` — higher-power blocker can't block skulk creature
  - `test_landwalk_unblockable` — islandwalk creature can't be blocked when defender controls Island
  - `test_landwalk_blockable_without_land` — islandwalk creature can be blocked when defender has no Islands
  - `test_trample_with_coassigned_damage` — pre-assigned damage reduces lethal threshold
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: combat removal helper, evasion framework, trample co-assigned damage (E36–E38)`

---

#### T21c: Infect/Wither + Planeswalker damage routing + Toxic
- **Scope:** Medium
- **Source:** E39 + E40 + E41
- **Depends on:** T01 (counters for -1/-1), T02 (player counters for poison)
- **Files:** `engine/actions.rs` (modify), `engine/keywords.rs` (modify), `types/keywords.rs` (modify)
- **Steps:**
  1. **Infect/Wither damage routing (E39):** In `engine/actions.rs` `perform_action(DealDamage)`, check source for `Infect`/`Wither` keywords. If infect to player → add poison counters instead of life loss. If infect/wither to creature → add -1/-1 counters instead of marking damage. Add `Infect`, `Wither` variants to `KeywordAbility`.
  2. **Planeswalker/Battle damage routing (E40):** In `perform_action(DealDamage)`, when target is a planeswalker (check `DamageTarget::Object` where object has planeswalker type), remove that many loyalty counters instead of marking damage. Remove `min(damage, current_loyalty)` counters — excess damage doesn't overflow (a 6/6 attacking a 4-loyalty PW removes 4 counters; the extra 2 damage is simply absorbed). Same for battles with defense counters.
  3. **Toxic (E41):** In `engine/keywords.rs` (or `actions.rs`), after combat damage is dealt to a player by a creature with Toxic, add N poison counters to that player. Use `KeywordAbility::Toxic(u32)` as a parameterized enum variant — this is more conceptually correct than a dedicated `CardData` field, since Toxic is a keyword ability per the rules (702.162), and the parameter can theoretically be modified by continuous effects. Note: `Toxic(3)` and `Toxic(2)` are distinct values in a `HashSet<KeywordAbility>`, which is correct per 702.162c (multiple instances trigger independently). If a creature has both `Toxic(2)` and `Toxic(3)`, both apply when it deals combat damage, resulting in 5 total poison counters. The `KeywordAbility` enum is already `Copy` — `u32` is Copy, so this doesn't break the derive.
- **Codebase verification:** Confirmed `KeywordAbility` at `keywords.rs:7-30` is missing Infect, Wither, Toxic. Damage pipeline in `actions.rs` has no infect/PW routing.
- **Tests:**
  - `test_infect_to_player_gives_poison` — infect damage → poison counters, no life loss
  - `test_infect_to_creature_gives_counters` — infect damage → -1/-1 counters, no damage marked
  - `test_wither_to_creature_gives_counters` — wither damage → -1/-1 counters, no damage marked
  - `test_wither_to_player_is_normal` — wither to player deals normal life loss (wither only affects creatures)
  - `test_planeswalker_damage_removes_loyalty` — damage to PW removes loyalty counters
  - `test_planeswalker_damage_overflow_absorbed` — 6 damage to 4-loyalty PW removes 4 counters, no overflow
  - `test_toxic_adds_poison_on_combat_damage` — toxic creature deals combat damage, player gets poison
  - `test_toxic_multiple_instances_stack` — creature with Toxic(2) + Toxic(3) gives 5 poison
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: infect/wither routing, PW damage, toxic poison (E39–E41)`

---

#### T22: Duration + turn structure + targeting fixes
- **Scope:** Medium
- **Source:** E42 + E43 + E44 + E45 + E46 + E47 + E48
- **Depends on:** none
- **Coordination note:** T22 step 4 places stub `remove_expired_effects` hooks in `turns.rs` and `game_state.rs`. Part 2's L02 (Duration tracking) implements the real `continuous_effects` Vec and replaces these stubs. T22 should land before L02 so L02 can build on the hook sites rather than duplicating them. If L02 lands first, T22's stubs become no-ops — no correctness issue, but watch for merge conflicts in `turns.rs`.
- **Files:** `types/effects.rs` (modify), `engine/turns.rs` (modify), `state/game_state.rs` (modify), `state/player.rs` (modify), `oracle/legality.rs` (modify), `engine/targeting.rs` (modify), `types/keywords.rs` (modify)
- **Steps:**
  1. **Duration::UntilEndOfCombat (E43):** Add `UntilEndOfCombat` variant to `Duration` enum in `types/effects.rs`.
  2. **Duration::ThisTurn (E44):** Add `ThisTurn` variant to `Duration` enum. Document that both `UntilEndOfTurn` and `ThisTurn` expire during the cleanup step (rule 514.2).
  3. **Duration::UntilEndOfYourNextTurn (bonus):** Add `UntilEndOfYourNextTurn` variant. Not in the E-list but common enough to warrant adding now. Reference card: Wrenn's Resolve ({1}{R} Sorcery — "Exile the top two cards of your library. Until the end of your next turn, you may play those cards."). Expires at the cleanup step of the specified player's next turn.
  4. **Duration expiry hooks (E42):** In `engine/turns.rs`, add calls to `remove_expired_effects(duration)` at phase/step boundaries. Create `GameState::remove_expired_effects(&mut self, expired_duration: Duration)` that removes continuous effects (from the future `continuous_effects` Vec) matching the duration. For now, this is a stub — the Vec doesn't exist until Phase 5. But the hooks in `turns.rs` should be placed: `UntilEndOfCombat` at end of combat phase, `UntilEndOfTurn`/`ThisTurn` at cleanup step, `UntilYourNextTurn` at the start of that player's next turn, `UntilEndOfYourNextTurn` at the cleanup step of that player's next turn.
  5. **lands_per_turn dynamic (E45):** The field `lands_per_turn: u32` already exists on `PlayerState` (confirmed at `player.rs:21`), initialized to 1. However, per F18 (chapter 3 audit, rule 305.2), continuous effects like Exploration and Azusa can increase this number, and those effects have durations, can stack, and can be removed mid-turn. A simple mutable field that effects write to is insufficient — `lands_per_turn` needs to be *computed* like other effective characteristics. **This ticket's scope:** (a) Ensure `oracle/legality.rs` `playable_lands` reads from a query function (not a hardcoded 1 or a raw field read). (b) Create `oracle::get_effective_lands_per_turn(game, player_id) -> u32` that currently returns `player.lands_per_turn` as a passthrough. (c) Document that Phase 5's L15 (post-layer pass) should compute this value from base (1) + active continuous effects that grant additional land plays, analogous to how player action restrictions are recomputed after each layer pass. The raw `lands_per_turn` field on `PlayerState` becomes the *base* value; the oracle function returns the *effective* value. This matches the pattern established by `get_effective_power`/`get_effective_toughness` routing through `compute_characteristics`.
  6. **Hexproof (E46):** In `engine/targeting.rs`, update `validate_creature_target`, `validate_any_target`, and `validate_permanent_target` to check for `KeywordAbility::Hexproof`. If the target has hexproof and the spell/ability controller is an opponent, return Err. This requires passing `controller: PlayerId` into the validation methods (or into `validate_targets`). **Important (post-T15b):** The `EffectRecipient::Target` vs `Choose` distinction already exists. Hexproof/shroud/protection checks must ONLY apply to `Target` effects, NOT `Choose` effects (rule 303.4a — Aura ETB choosing is not targeting). `validate_selection` dispatches both; the hexproof/shroud/protection logic should be gated inside the `Target`-specific validation path. `has_any_legal_choice` in `targeting.rs` will also need updating to respect hexproof/shroud for `Target`-originated queries vs `Choose`-originated queries.
  7. **Shroud (E47):** Same as hexproof but blocks ALL targeting (including own controller). Add check for `KeywordAbility::Shroud` — if target has shroud, always return Err. Same `Target`-only gating applies.
  8. **Protection targeting restriction (E48):** Add a `Protection` representation. Currently `KeywordAbility::Protection` exists but is not parameterized. For the targeting check, we need to know "protection from what." Options: (a) add `ProtectionQuality` enum and store on the card data, or (b) use a separate field. Recommend: add `pub protection_from: Vec<ProtectionQuality>` to `CardData` where:
     ```rust
     pub enum ProtectionQuality {
         Color(Color),
         CardType(CardType),
         Subtype(Subtype),
         All, // "protection from everything"
     }
     ```
     In targeting validation, if the target has protection from a quality matching the source spell/ability, targeting is illegal. This requires knowing the source's characteristics — pass source_id to validation.
- **Codebase verification:** Confirmed `Duration` enum at `effects.rs:79-93` is missing `UntilEndOfCombat` and `ThisTurn`. `targeting.rs` has no hexproof/shroud/protection checks. `validate_targets` does not receive controller or source info.
- **Tests:**
  - `test_duration_until_end_of_combat_variant` — enum variant exists and is distinct
  - `test_duration_this_turn_variant` — enum variant exists
  - `test_duration_until_end_of_your_next_turn_variant` — enum variant exists
  - `test_duration_expiry_hook_called` — stub method exists and is called at correct time
  - `test_lands_per_turn_dynamic` — set to 2, player can play 2 lands
  - `test_hexproof_blocks_opponent_targeting` — opponent can't target hexproof creature
  - `test_hexproof_allows_own_targeting` — controller can target own hexproof creature
  - `test_shroud_blocks_all_targeting` — nobody can target shroud creature
  - `test_protection_blocks_matching_source` — pro-red creature can't be targeted by red spell
  - `test_protection_allows_nonmatching_source` — pro-red creature targetable by green spell
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: duration variants, expiry hooks, dynamic lands, hexproof/shroud/protection targeting (E42–E48)`

---

#### T21d: Combat requirements solver — attack and block requirement maximization
- **Scope:** Medium
- **Source:** Rules 508.1d (attack requirements), 509.1c (block requirements)
- **Depends on:** T21b (evasion framework provides restriction structs and validation functions)
- **Files:** `engine/combat/validation.rs` (modify), `engine/combat/steps.rs` (modify)
- **Steps:**
  1. **Expand `BlockRequirement` enum.** The current `MustBlockIfAble(ObjectId)` only models the blocker-side case. Add:
     ```rust
     pub enum BlockRequirement {
         /// Blocker-side: this creature must block something if able
         MustBlockIfAble(ObjectId),
         /// Blocker-side: this creature must block a specific attacker if able
         MustBlockSpecific { blocker: ObjectId, attacker: ObjectId },
         /// Attacker-side (lure): all legal blockers must block this attacker
         AllMustBlock(ObjectId),
     }
     ```
     `AllMustBlock` is an attacker-side property (Lure) — it fans out into per-blocker constraints at solve time. `MustBlockSpecific` is blocker-side with a designated target (Provoke). These are structurally different: `AllMustBlock` affects the defender's entire block assignment, while the blocker-side variants constrain a single creature.
  2. **Expand `AttackRequirement` enum.** Add goad support:
     ```rust
     pub enum AttackRequirement {
         /// This creature attacks each combat if able
         MustAttackIfAble(ObjectId),
         /// Goad: must attack if able, and must attack a player other than the
         /// goading player if able (combined requirement + restriction)
         Goaded { creature: ObjectId, goading_player: PlayerId },
     }
     ```
  3. **Implement the requirement-maximizing solver.** Per rules 508.1d and 509.1c, the game must find an assignment that satisfies the *maximum number* of requirements without violating any restrictions. Algorithm:
     - Collect all applicable requirements and restrictions for the current combat.
     - For each requirement, determine if the creature is *able* (not tapped, not summoning-sick, no `CantAttack`/`CantBlock` restriction that applies). A creature that "must attack if able" but is tapped is not able — the requirement is vacuously satisfied.
     - For `AllMustBlock(attacker_id)`, expand into per-blocker `MustBlockSpecific` constraints for each blocker where `can_block(attacker, blocker)` is true.
     - Given the expanded requirements and the proposed assignment, count how many requirements are satisfied. Verify this count equals the maximum achievable count. If the player's proposal satisfies fewer requirements than the maximum, reject with an error indicating which requirements could additionally be met.
     - **Note:** Finding the true maximum is a matching problem. For typical board states (≤20 creatures), brute-force or greedy approaches are acceptable. If performance becomes a concern, model as a maximum bipartite matching problem.
  4. **Replace naive checks in `validate_attack_constraints` and `validate_block_constraints`.** The current loops just check if required creatures appear in the proposed set. Replace with calls to the solver from step 3.
  5. **Population hooks (scaffolding only).** The solver is ready to consume requirements, but nothing populates the vecs yet. Add `// TODO: populate from continuous effects and designations (goad, Lure, etc.)` comments at the sites in `steps.rs` where `AttackConstraints::none()` and `BlockConstraints::none()` are constructed. Actual population requires triggered abilities (Phase 6) and per-permanent designations (D17).
- **Tests:**
  - `test_must_attack_if_able_enforced` — required creature not in proposed set → error
  - `test_must_attack_tapped_vacuous` — tapped creature with requirement → vacuously satisfied, no error
  - `test_must_attack_sick_vacuous` — summoning-sick creature with requirement → vacuously satisfied
  - `test_goad_must_attack_other_player` — goaded creature must attack non-goading player
  - `test_goad_only_one_opponent_can_attack_goader` — if only one opponent exists, goad restriction is impossible → creature still must attack
  - `test_must_block_if_able_enforced` — required blocker not in proposed set → error
  - `test_must_block_specific_attacker` — MustBlockSpecific not paired with correct attacker → error
  - `test_all_must_block_lure` — AllMustBlock expands: all legal blockers must block the lured attacker
  - `test_all_must_block_some_cant` — blocker with flying restriction can't block non-flying lured attacker → not required
  - `test_requirement_maximization` — two requirements, only one satisfiable → one satisfied is maximum, proposal accepted
  - `test_requirement_not_maximized_rejected` — two requirements both satisfiable, only one satisfied → rejected
- **Acceptance:** All existing tests pass + new tests pass + 0 warnings
- **Commit:** `engine: combat requirements solver — attack/block requirement maximization (508.1d, 509.1c)`

---

### Cross-Cutting: Mana Restrictions Cards (T12d)

#### T12d: First restricted-mana cards + persistence
- **Scope:** Large
- **Source:** E15 (implementation phase C from `plans/mana-restrictions-design.md` §10)
- **Depends on:** T12c; blanket persistence (step 6) depends on L04
- **Files:** `cards/` (new card files), `engine/cast.rs` (modify), `engine/turns.rs` (modify), `engine/mana.rs` (modify)
- **Steps:**
  1. Implement Cavern of Souls (or simpler test card) with `ManaOutput.special`.
  2. Wire grant application in `cast_spell`: call `drain_spent_grants()` after `pay_costs`, apply grants (e.g., uncounterable) to stack entry.
  3. Update `empty()` call sites in `turns.rs` to `empty_with_reason()` with `BlanketPersistenceSet::none()` (no blanket effects yet).
  4. Implement Birgi or test persistent-mana card (time-gated: `UntilEndOf(EndOfTurn)`).
  5. Remove deprecated `can_pay()` and `pay()` methods.
  6. *(After L04)* Implement `build_blanket_persistence_set()` querying continuous effects. Wire Omnath/Upwelling as static abilities producing blanket persistence.
- **Design reference:** `plans/mana-restrictions-design.md` §7 (Grants), §3.3 (Persistence)
- **Note:** Steps 1–5 can execute after T12c without waiting for Phase 5. Step 6 is gated on L04 and can be a follow-up commit within the same ticket.
- **Tests:**
  - `test_cavern_mana_pays_creature` / `_rejected_for_instant` / `_grant_uncounterable`
  - `test_thalia_boseiju_grant_flow` — Thalia taxes 0-mana instant, Boseiju mana pays + grants uncounterable
  - `test_doubling_cube_does_not_inherit_restrictions` — doubled mana is unrestricted
  - `test_persistence_birgi` — `UntilEndOf(EndOfTurn)` survives phase transition
  - `test_birgi_mana_survives_birgi_dying` — atom metadata persists after source dies
  - `test_omnath_blanket_persistence` / `_omnath_dies_mana_empties` — blanket via continuous effects
  - `test_fuzz_games_with_restricted_mana` — fuzz harness with restricted-mana lands
- **Acceptance:** All tests pass + fuzz harness 500/500 with restricted-mana decks + 0 warnings
- **Commit:** `mana: first restricted-mana cards + persistence (T12d)`

---

### Part 1 Deferred Items (no tickets — reference only)

| # | Summary |
|---|---------|
| D1 | Phasing (502.1) — needs layer system interaction |
| D2 | Face-down permanents (708) — needs morph/disguise |
| D3 | Double-faced card system (712) — needs back_face CardData |
| D4 | Split card / Adventure / CardLayout (709/715/718/720) — needs CardLayout restructuring |
| D5 | Copy system (707) — deferred to Phase 5 L1 |
| D6 | Extra turns/phases/steps (500.7–11) — needs mutable TurnPlan |
| D7 | Multiplayer systems — commander designation, color identity, etc. |
| D8 | Replacement effects on zone transitions (400.6) — Phase 7 |
| D9 | "Can't" overrides "can" (101.2) — already embedded as design pattern |
| D10 | "Can't have" ability prohibition (113.11) — Layer 6 concept |
| D11 | Mandatory loop detection (104.4b) — extremely niche |
| D12 | Mulligan implementation (103.5) — game setup polish |
| D13 | Regeneration shield system — Phase 7 replacement effect |
| D14 | Day/Night global designation (730) — when daybound cards arrive |
| D15 | Monarch/Initiative (724/725) — Phase 6 triggered abilities |
| D16 | Per-turn tracker system — start with dedicated fields when needed |
| D17 | Per-permanent designations (monstrous, exerted, goaded, etc.) — when cards arrive |
| D18 | Multi-name / "choose a card name" (201.2a/201.4) — when Pithing Needle arrives |
| D19 | Spell copying (Storm, Replicate) — Phase 6+ |
| D20 | Companion — outside-the-game zone |
| D21 | Exile zone metadata (face-down, exiled-by) — when Gonti/Prosper arrive |
| D22 | Excess damage redirection event metadata (120.4a) — Phase 6 |
| D23 | "Can't gain life" / "Can't lose life" — prohibition effects |
| D24 | Player-leaves-game cleanup — multiplayer only |
| D25 | Land+other-type casting restriction (300.2a) — when dual-type lands arrive |

---

## Part 2: Phase 5 Continuous Effects & Layer System

*[Merged verbatim from implementation-plan-part2.md]*

### Overview

**Scope:** Layer system (rule 613), continuous effect types, dependency detection, all seven layers (L1/L3 stubbed), Tier 1 + Tier 2 test cards, LKI system deferred from Part 1.

**Prerequisites from Part 1 ([P5-PREREQ] tickets):**
- **T01** — Counters field on `BattlefieldEntity` + expanded `CounterType` (needed for L7c)
- **T05** — `color_indicator` on `CardData` (needed for L5)
- **T09** — `controller_since_turn` summoning sickness rework (needed for L2)

**Sub-plan mapping:**
- **5A (Foundation + P/T):** L01–L08 — types, duration, timestamps, layer engine, P/T, Giant Growth, Anthem
- **5B (Remaining Layers + Dependency):** L09–L16 — abilities, type/color, control, stubs, migration, dependency, scaffolding
- **5C (Cards + Testing):** L17–L21 — Tier 1/2 cards, LKI, integration tests, fuzz

**Ticket count:** 21

---

### Sub-Plan 5A: Foundation + P/T Layers (L01–L08)

#### L01: Core types — `types/continuous.rs` + `is_cda` on `AbilityDef`
- **Scope:** Medium
- **Source:** §5a, W1, W4 (+ PC1)
- **Depends on:** none
- **Files:** `types/continuous.rs` (create), `types/mod.rs` (modify), `objects/card_data.rs` (modify), `state/game_state.rs` (modify)
- **Steps:**
  1. Create `types/continuous.rs`: `EffectId` (Uuid alias — matches `ObjectId`/`AbilityId` convention), `ContinuousEffect` struct (id, source, timestamp: Timestamp, duration, layer, modification, applies_to, controller, is_cda, **kind: ContinuousEffectKind**), `Layer` enum (Copy/Control/Text/Type/Color/Ability/PowerToughness(PTSublayer) — derives PartialOrd/Ord), `PTSublayer` (CDA/SetBase/Modify/Switch), `Modification` enum (all variants from §5a plan), `PTComputation` (GraveyardCardTypes/ManaValue), `AppliesTo` (Single/Filter/All/Self_). No L1/L3 variants yet. Add `new_effect_id()` to `types/ids.rs`.
     > **Session 6 audit design decisions (see `session-6-audit-response.md`):**
     > - **`ContinuousEffectKind` enum:** `CharacteristicModifying` (L1–L7), `GameRuleModifying` (613.11, post-layer), `CostModification { kind: CostModKind }` (601.2f pipeline). Discriminates effect routing — characteristic effects go through layers, game-rule effects go through the post-layer pass (L15), cost modifications go through the 601.2f pipeline.
     > - **`Applicability` enum:** Replaces the simpler `AppliesTo`. Four variants: `SelfRef` ("~ gets +1/+1"), `ObjectFilter(ObjectFilter)` ("creatures you control"), `PlayerFilter(PlayerFilter)` ("each opponent"), `EventFilter(GameActionPattern)` (for replacement/trigger matching in Phases 6–7). This is a cross-cutting type used by continuous effects, replacement effects, and triggered abilities.
     > - **`AbilityOrigin` tag:** `Intrinsic` vs `Granted { source_id, effect_timestamp }`. Added to ability instances so Layer 3 text-changing only modifies intrinsic text, not externally granted abilities (see L12).
  2. Add `pub mod continuous` to `types/mod.rs`.
  3. **PC1:** Add `pub is_cda: bool` to `AbilityDef` (default false). Source of truth for CDA status. Update builder.
  4. Add to `GameState`: `continuous_effects: Vec<ContinuousEffect>`, `register_continuous_effect()`, `remove_effects_from_source()`, `effects_in_layer()`. **Audit correction:** No `next_effect_id` counter — `EffectId` is UUID-based (consistent with `ObjectId`/`AbilityId`). Each `ContinuousEffect` gets its ID via `new_effect_id()` at creation time.
  5. W4b confirmed: no `mana_cost` on EffectiveCharacteristics. W4c/W4d: player-affecting → L15, game-rule → W14 (no code).
- **Tests:**
  - `test_effect_registration` — register, verify present
  - `test_effect_removal_by_source` — remove one source, other remains
  - `test_effects_in_layer_filtering` — correct subset returned
  - `test_layer_ordering` — Copy < Control < ... < PT(Switch)
  - `test_is_cda_on_ability_def` — field works
- **Acceptance:** All existing + new tests pass, 0 warnings
- **Commit:** `types: ContinuousEffect, Layer, Modification; is_cda on AbilityDef (§5a, W1, W4, PC1)`

---

#### L02: Duration tracking + cleanup hooks
- **Scope:** Small
- **Source:** §5b
- **Depends on:** L01
- **Coordination note:** Part 1's T22 (Duration + turn structure fixes) places stub `remove_expired_effects` hooks in `turns.rs` and `game_state.rs`. L02 replaces these stubs with real implementations backed by the `continuous_effects` Vec. Recommended order: T22 first (places hook sites), then L02 (fills them in). Watch for merge conflicts in `turns.rs` if ordering differs.
- **Files:** `types/effects.rs` (modify), `engine/turns.rs` (modify), `engine/zones.rs` (modify)
- **Steps:**
  1. Add `WhileTargetOnBattlefield` and `UntilEndOfYourNextTurn` to `Duration` enum. `WhileTargetOnBattlefield` is for Mind Snare L2. `UntilEndOfYourNextTurn` expires at the *end* of your next turn's cleanup step (distinct from `UntilYourNextTurn` which expires at the *start* of your next beginning phase).
  2. Cleanup step: `retain` to remove `UntilEndOfTurn` effects (514.2). Also remove `UntilEndOfYourNextTurn` effects whose controller is the active player (expires at end of that player's turn).
  3. Beginning phase: remove `UntilYourNextTurn` for active player (500.4).
  4. `cleanup_zone_state`: call `remove_effects_from_source`. On target leaving: remove `WhileTargetOnBattlefield` effects targeting that permanent.
- **Tests:**
  - `test_eot_removed_at_cleanup` / `test_uyn_persists_opponent` / `test_uyn_expires_own` / `test_source_exit_removes` / `test_target_exit_removes` / `test_until_end_of_your_next_turn_persists_then_expires`
- **Acceptance:** All existing + new tests pass, 0 warnings
- **Commit:** `engine: duration tracking and cleanup hooks (§5b)`

---

#### L03: Timestamp struct — APNAP sub-ordering
- **Scope:** Small
- **Source:** §5c (partial), W10 (+ PC4)
- **Depends on:** L01
- **Files:** `state/battlefield.rs` (modify), `types/continuous.rs` (modify), `state/game_state.rs` (modify)
- **Steps:**
  1. **Decision: two-part timestamp (W10 option 1).** `Timestamp { global_seq: u64, sub_index: u16 }` with derived `Ord` (lexicographic).
     > **Session 6 audit refinement (see `session-6-audit-response.md`):** Field names refined to `global_seq` (APNAP-ordered monotonic counter) and `sub_index` (intra-object relative ordering for multiple effects on the same object at the same global timestamp). `sub_index` is `u16` (not `u8`) for future-proofing. `global_seq` assignment follows APNAP: active player's objects get earlier sequence numbers.
  2. Change `BattlefieldEntity.timestamp` from `u64` to `Timestamp`.
  3. Change `ContinuousEffect.timestamp` likewise.
  4. `allocate_timestamp()` returns `Timestamp { base: counter, sub_order: 0 }`. Add `allocate_timestamp_batch(count)` for simultaneous ETBs.
  5. Update all comparison sites. C3/C6 confirm field exists — type change only.
- **Tests:**
  - `test_timestamp_ordering` / `test_apnap_ordering` / `test_batch_allocation`
- **Acceptance:** All existing + new tests pass, 0 warnings
- **Commit:** `state: Timestamp struct with APNAP sub-ordering (§5c, W10, PC4)`

---

#### L04: `EffectiveCharacteristics` + `compute_characteristics` skeleton
- **Scope:** Medium
- **Source:** §5c, W4, W6 (+ PC3)
- **Depends on:** L01, L03
- **Files:** `engine/layers.rs` (create), `engine/mod.rs` (modify)
- **Steps:**
  1. Create `engine/layers.rs`. Define `EffectiveCharacteristics`: name, colors, color_indicator, types, supertypes, subtypes, keywords, abilities, power, toughness, controller. **Rule 302.4:** `power` and `toughness` are `Option<i32>` — set to `None` if the effective types do NOT include `Creature` (even if `card_data.power` is `Some`). This gates P/T behind creature status, which is required for Vehicles (301.7a), Opalescence (adds Creature → gains P/T), and Humility (keeps Creature → keeps P/T). Init: if `card_data.types.contains(Creature)`, copy `card_data.power`/`toughness`; else set `None`.
  2. Implement `compute_characteristics(game, object_id) -> Option<EffectiveCharacteristics>`:
     - Init from `card_data`.
     - **PC3/W6 (CDA all-zone, option 2):** Before registered effects, scan `card_data.abilities` for `is_cda == true`, synthesize and apply in appropriate layer. CDAs are intrinsic, not registered.
     - For each layer: collect effects, CDAs first (W5), then non-CDAs by dependency+timestamp (stub dependency → L14). Apply modifications. Re-evaluate remaining (613.8c).
  3. Implement `apply_modification` — dispatch on variant. Initially: PT variants (ModifyPT, SetPT, SetPTDynamic, SwitchPT). Others stub.
  4. Implement `evaluate_applies_to`.
  5. Add `pub mod layers` to `engine/mod.rs`.
  6. **Design note (CDA portability):** `is_cda` travels with the `AbilityDef`. Clone effects (L1, Phase 6) copy the source's `card_data` including CDA-flagged abilities. Text exchange (L3, deferred) swaps ability definitions — the `is_cda: true` flag moves with the ability. This means a vanilla creature that receives Tarmogoyf's text box via exchange would correctly gain a CDA. No special handling needed beyond the existing `is_cda` field on `AbilityDef`.
- **Tests:**
  - `test_no_effects_returns_printed` / `test_single_modify_pt` / `test_set_then_modify` / `test_cda_before_non_cda` / `test_cda_in_graveyard` / `test_switch_pt`
- **Acceptance:** All existing + new tests pass, 0 warnings
- **Commit:** `engine: EffectiveCharacteristics and compute_characteristics (§5c, W4, W6, PC3)`

---

#### L05: Locked-in target sets (613.6)
- **Scope:** Small
- **Source:** §5c (partial), W8
- **Depends on:** L04
- **Files:** `engine/layers.rs` (modify)
- **Steps:**
  1. Per-computation `HashMap<EffectId, Vec<ObjectId>>` for locked sets.
  2. First evaluation stores result; later layers reuse stored set.
  3. Exercised by Opalescence (L4 + L7b).
- **Tests:**
  - `test_locked_set_persists_across_layers` / `test_first_eval_stored`
- **Acceptance:** All existing + new tests pass, 0 warnings
- **Commit:** `engine: locked-in target sets for 613.6 (§5c, W8)`

---

#### L06: Layer 7 P/T — counters in 7c, deprecate modifiers
- **Scope:** Medium
- **Source:** §5d
- **Depends on:** L04, T01
- **Files:** `engine/layers.rs` (modify), `state/battlefield.rs` (modify), `oracle/characteristics.rs` (modify)
- **Steps:**
  1. L7c reads `BattlefieldEntity.counters` for +1/+1 and -1/-1 (rule 613.4c).
  2. Route `get_effective_power`/`get_effective_toughness` through `compute_characteristics`.
  3. Remove `power_modifier`/`toughness_modifier` from `BattlefieldEntity`. Migrate all references.
- **Tests:**
  - `test_7c_counters` / `test_7c_mixed_counters` / `test_7c_counters_and_effect` / `test_oracle_power_routes` / `test_oracle_toughness_routes`
- **Acceptance:** All existing tests pass (after migration) + new, 0 warnings
- **Commit:** `engine: Layer 7 P/T, counters in 7c, deprecate modifiers (§5d)`

---

#### L07: Hook resolve_primitive — P/T effects + Giant Growth + 611.2c lock-in
- **Scope:** Medium
- **Source:** §5j (partial), §5l (Giant Growth), Ch6 audit F5 (rule 611.2c)
- **Depends on:** L06
- **Files:** `engine/resolve.rs` (modify), `engine/layers.rs` (modify), `cards/phase5_t1.rs` (create), `cards/mod.rs` (modify), `cards/registry.rs` (modify)
- **Steps:**
  1. Implement `ModifyPowerToughness` and `SetPowerToughness` primitive arms → create L7c/L7b ContinuousEffect.
  2. **Rule 611.2c — Spell/ability effect lock-in.** When `resolve_primitive` creates a ContinuousEffect from a resolving spell or ability that uses `AppliesTo::Filter(...)` or `AppliesTo::All(...)`, immediately evaluate the filter against the current game state and store the result as `AppliesTo::Specific(Vec<ObjectId>)`. This locks in the affected set at resolution time — objects that later match the filter do NOT gain the effect. Static ability effects (registered via `register_static_abilities`) are NOT locked in and continue using dynamic `Filter`/`All`. Add `AppliesTo::Specific(Vec<ObjectId>)` variant to the `AppliesTo` enum. In `evaluate_applies_to`, `Specific` returns the stored vec directly without re-evaluating. **Note:** `AppliesTo::Single(ObjectId)` (used by Giant Growth) is already locked-in by nature — this step handles the general filter case. **Also:** Per 611.2c, effects that don't modify characteristics (game-rule effects per §5k) are NOT locked in, even from spells — they re-evaluate dynamically.
  3. Giant Growth: {G} Instant, +3/+3 UntilEndOfTurn. Register in CardRegistry.
- **Tests:**
  - `test_giant_growth_creates_effect` / `test_bears_5_5` / `test_reverts_at_cleanup` / `test_lethal_after_revert`
  - `test_spell_filter_effect_locks_in_set` — spell creating filter-based ContinuousEffect stores `Specific(...)`, new matching objects don't gain the effect
  - `test_static_filter_effect_stays_dynamic` — static ability effect keeps `Filter(...)`, new matching objects do gain the effect
- **Acceptance:** **5A Gate:** Giant Growth on Bears = 5/5, reverts. Lock-in semantics verified. All tests pass.
- **Commit:** `engine: hook ModifyPT/SetPT, Giant Growth card, 611.2c lock-in (§5j, §5l, Ch6-F5)`

---

#### L08: Static ability registration + Glorious Anthem
- **Scope:** Medium
- **Source:** §5j (partial), W11 (+ PC10)
- **Depends on:** L07, L02
- **Dependency rationale:** L02 implements `remove_effects_from_source` and duration-based cleanup in `zones.rs`. Without L02, static abilities registered with `WhileSourceOnBattlefield` duration will leak when the source leaves the battlefield. `test_static_removed_on_exit` requires L02's cleanup path.
- **Files:** `engine/layers.rs` (modify), `engine/zones.rs` (modify), `cards/phase5_t1.rs` (modify)
- **Steps:**
  1. `register_static_abilities(game, object_id)`: scan `card_data.abilities` for Static type, create ContinuousEffect with `WhileSourceOnBattlefield`, `is_cda` from AbilityDef, timestamp = permanent's timestamp.
  2. **PC10/W11:** Call in `init_zone_state`/`init_zone_state_with_controller` BEFORE `PermanentEnteredBattlefield` event. Static ability timestamp T1 < spell effect timestamp T2.
  3. Glorious Anthem: {1}{W}{W} Enchantment. Static → L7c ModifyPT(1,1) on creatures you control.
- **Tests:**
  - `test_static_registered_on_etb` / `test_static_removed_on_exit` / `test_anthem_bears_3_3` / `test_anthem_destroyed_reverts` / `test_two_anthems_4_4` / `test_registration_before_etb_event`
- **Acceptance:** All existing + new tests pass, 0 warnings
- **Commit:** `engine: static ability registration, Glorious Anthem (§5j, W11, PC10)`

---

### Sub-Plan 5B: Remaining Layers + Dependency (L09–L16)

#### L09: Layer 6 — Abilities
- **Scope:** Small
- **Source:** §5e, W5
- **Depends on:** L04
- **Files:** `engine/layers.rs` (modify), `oracle/characteristics.rs` (modify), `engine/resolve.rs` (modify)
- **Steps:**
  1. `apply_modification` for AddAbility/RemoveAbility/RemoveAllAbilities.
  2. Route `has_keyword` through `compute_characteristics`.
  3. Hook `AddAbility`/`RemoveAbility` primitives in resolve.rs.
  4. **TODO comment (D10):** Add `// TODO: L6 must also read keyword counters from BattlefieldEntity.counters (rule 613.1f) — implement when keyword counter cards arrive. See D10.` in the L6 processing block.
     > **Session 6 audit note (see `session-6-audit-response.md`):** Granted abilities must carry an `AbilityOrigin::Granted { source_id, effect_timestamp }` tag so that Layer 3 text-changing (L12) can distinguish intrinsic vs. granted abilities and only modify intrinsic text. The `AbilityOrigin` type is defined in L01.
- **Tests:**
  - `test_grant_flying` / `test_remove_flying` / `test_remove_all` / `test_oracle_has_keyword_routes`
- **Commit:** `engine: Layer 6 abilities, oracle routing (§5e, W5)`

---

#### L10: Layers 4–5 — Type + Color + SetLandType 305.7
- **Scope:** Large
- **Source:** §5f, W12 (+ PC2, PC6, PC7)
- **Depends on:** L04, T05
- **Note:** L09 (Layer 6) was previously listed as a dependency but is not required. L10's `SetLandType` clears printed abilities inline in L4 (which runs before L6 in the layer loop) and does not call any L09 code. L09 and L10 can run in parallel after L04.
- **Files:** `engine/layers.rs` (modify), `oracle/characteristics.rs` (modify)
- **Steps:**
  1. L4 basic operations: `AddTypes`/`RemoveTypes` — set add/remove on `EffectiveCharacteristics.types`. `RemoveSubtypes` — set remove on `.subtypes`.
  2. **`AddSubtypes` (PC7, rule 305.6):** When adding a basic land subtype (Plains/Island/Swamp/Mountain/Forest), also grant the corresponding intrinsic mana ability (e.g., Swamp → `{T}: Add {B}`). This is the mechanism Urborg uses. Non-land subtypes (e.g., AddSubtypes(Goblin)) are a plain set-add with no side effects.
  3. **`SetLandType` (W12, PC2, PC6, rule 305.7):** (a) remove all existing land subtypes, (b) add new land subtype, (c) clear printed abilities (L4 before L6 — safe), (d) grant intrinsic mana for new type. **PC12:** does NOT add Basic supertype. This is distinct from `AddSubtypes` — it *replaces* rather than *adds*. Blood Moon uses this.
  4. L5: SetColors/AddColors/RemoveColors. Apply color_indicator from card_data.
  5. New oracle: `get_effective_colors`, `get_effective_types`, `get_effective_subtypes`, `get_mana_value`. Route `is_creature` through layers.
- **Tests:**
  - `test_add_types` / `test_add_subtypes_grants_mana` / `test_set_land_type_clears` / `test_set_land_type_no_basic` / `test_set_colors` / `test_color_indicator` / `test_get_mana_value` / `test_is_creature_routes`
- **Commit:** `engine: Layers 4-5, SetLandType 305.7 (§5f, W12, PC2, PC6, PC7)`

---

#### L11: Layer 2 — Control
- **Scope:** Medium
- **Source:** §5g (+ PC8)
- **Depends on:** L04, T09
- **Files:** `engine/layers.rs` (modify), `oracle/characteristics.rs` (modify), `engine/resolve.rs` (modify)
- **Steps:**
  1. `apply_modification` for ChangeController.
  2. **PC8:** Update `controller_since_turn` on control change → re-sickens creature.
  3. `get_effective_controller` oracle function. Hook `GainControl` primitive.
- **Tests:**
  - `test_change_controller` / `test_summoning_sickness_on_steal` / `test_haste_bypass_steal`
- **Commit:** `engine: Layer 2 control (§5g, PC8)`

---

#### L12: Layer 1 stub + Layer 3 text-changing implementation
- **Scope:** Medium *(upgraded from Small — D2 locked in per phase5-pre-work-final.md audit)*
- **Source:** §5h, D2 (locked in)
- **Depends on:** L04
- **Files:** `types/continuous.rs` (modify), `engine/layers.rs` (modify), `types/effects.rs` (modify)
- **Steps:**
  1. **Layer 1 — stub.** Verify loop iterates L1 as a no-op (zero effects collected). Doc comment noting L1 → Phase 6 (copy effects require replacement effects).
  2. **Layer 3 — `TextChange` enum.** Define in `types/continuous.rs`:
     > **Session 6 audit constraint (see `session-6-audit-response.md`):** The tree-walker must check `AbilityOrigin` on each ability — only abilities with `AbilityOrigin::Intrinsic` are subject to text-changing. Abilities with `AbilityOrigin::Granted { .. }` are skipped by the walker. This prevents text-changing effects from modifying externally granted abilities (e.g., an Urborg-granted "{T}: Add {B}" should not be affected by a text-changing effect on the land).
     ```rust
     pub enum TextChange {
         ColorWord { from: Color, to: Color },
         BasicLandType { from: LandType, to: LandType },
         CreatureType { from: CreatureType, to: CreatureType },
     }
     ```
     Add `TextChange` as a `Modification` variant (or a dedicated `Layer::Text` modification type). Rule 612.2 constrains text-changing effects to only modify color words, basic land type words, and creature type words — no unstructured string manipulation.
  3. **`apply_text_change_to_effect` tree-walker.** Implement a recursive function on `Effect` that walks the tree and replaces matching enum values:
     - `Color` values in `Selector`, `PermanentFilter`, `CardFilter`, `ProtectionQuality`, etc.
     - `LandType` values in `LandwalkType`, `SetLandType` arguments, etc.
     - `CreatureType` values in subtype references.
     This is a semantic tree-walker, not string manipulation — it operates on typed enum values in the `Effect` AST.
  4. **Wire into L3 slot in `compute_characteristics`.** In the layer loop, when processing `Layer::Text`, collect text-changing effects, apply them via the tree-walker to affected objects' ability definitions. Text changes modify what abilities *do*, not whether they exist — `is_cda` flags are unaffected.
  5. **Splice/Overload/Cleave note:** These are casting-time structural transforms on the `Effect` tree (not L3 continuous effects). Subsequent L3 effects can modify the transformed text using the same walker. The walker implementation here enables that future interaction. No splice cards in Phase 5.
  6. **Deferred:** Permanent-targeting text-changing cards (Mind Bend, Magical Hack, Sleight of Mind) are rare and not in Phase 5 card pool. The infrastructure supports them but no test cards exercise L3 on permanents yet.
- **Tests:**
  - `test_text_change_color_word` — walker replaces Color::Red with Color::Blue in effect tree
  - `test_text_change_land_type` — walker replaces LandType::Mountain with LandType::Island
  - `test_text_change_creature_type` — walker replaces CreatureType in subtype filter
  - `test_text_change_no_match_noop` — walker on effect with no matching values is identity
  - `test_l1_stub_noop` — L1 slot collects zero effects, no modification
  - `test_l3_slot_applies_text_changes` — registered text-changing effect modifies ability in L3
- **Acceptance:** All existing + new tests pass, 0 warnings
- **Commit:** `engine: Layer 1 stub, Layer 3 text-changing tree-walker (§5h, D2)`

---

#### L13: Oracle routing + card_data read migration
- **Scope:** Medium
- **Source:** W2, W3, W15
- **Depends on:** L06, L09, L10, L11
- **Files:** `oracle/characteristics.rs` (modify), `engine/sba.rs` (modify), `engine/combat/validation.rs` (modify), `engine/costs.rs` (modify), `engine/targeting.rs` (modify), `ui/display.rs` (modify)
- **Steps:**
  1. W2 completed by L06/L09/L10/L11. New oracle funcs added.
  2. **W3:** Grep `card_data.types`, `.keywords`, `.colors`, `.power`, `.toughness`, `entry.controller`. Migrate to oracle. C14 exception: `stack.rs` reads printed types (correct).
  3. **W15:** `targeting.rs` uses `is_creature`/`get_effective_types` for target validation.
  4. `ui/display.rs` shows effective values.
- **Tests:**
  - `test_sba_uses_effective_toughness` / `test_targeting_effective_types` / `test_stack_uses_printed`
- **Commit:** `engine: oracle routing, migrate card_data reads (W2, W3, W15)`

---

#### L14: Dependency detection — `engine/dependency.rs`
- **Scope:** Large
- **Source:** §5i, W9 (+ PC9)
- **Depends on:** L04
- **Files:** `engine/dependency.rs` (create), `engine/mod.rs` (modify), `engine/layers.rs` (modify)
- > **Session 6 audit note (see `session-6-audit-response.md`):** The dependency system should be treated as a first-class subsystem in its own module (`engine/layers/dependency.rs` or `engine/dependency.rs`). Uses Kahn's algorithm for topological sort with cycle detection. Cycles are broken by falling back to timestamp ordering per 613.8b.
- **Steps:**
  1. Implement `depends_on(a, b, intermediate, game) -> bool` with **PC9 — all four 613.8a conditions:**
     - (1) Changes applies_to: hypothetical apply B to scratch copy, compare A's target set before/after
     - (2) Changes what A does: B modifies a characteristic that A's modification reads
     - (3) Changes text: defer (L3/D2), return false
     - (4) Changes existence: B removes A's generating ability (Blood Moon/Urborg)
  2. CDA guard: one CDA + one non-CDA → independent (return false immediately).
  3. **Iterative dependency resolution algorithm (613.8b).** The algorithm from the comprehensive rules is iterative — after each application, the dependency graph must be rebuilt because applying one effect may change dependencies. Functionally equivalent to:
     ```
     (a) Build a directed graph: one vertex per effect, edge A→B means "A depends on B" (B must be applied first).
     (b) Remove all edges that participate in any cycle (613.8b: cyclic dependencies are ignored).
     (c) Find all vertices with no outgoing edges (no remaining dependencies).
     (d) Among those, select the one with the earliest timestamp.
     (e) Apply that effect.
     (f) Remove it from the graph and go back to step (a) with the remaining effects.
     ```
     The rebuild in step (f)→(a) is critical: applying an effect may change what other effects do or apply to, potentially creating or breaking dependencies that didn't exist before. A one-shot topological sort is NOT sufficient.
  4. `order_effects_in_layer()` → CDAs first (sorted by timestamp), then non-CDAs via the iterative algorithm above. Returns effects in application order.
  5. Integrate into `compute_characteristics`, replacing the stub ordering.
- **Tests:**
  - `test_independent_timestamp` / `test_color_change_dependency` / `test_blood_moon_urborg` / `test_cycle_fallback` / `test_cda_guard` / `test_changes_what_a_does` / `test_dependency_rebuild_after_application`
- **Commit:** `engine: dependency detection, iterative 613.8b algorithm (§5i, W9, PC9)`

---

#### L15: Post-layer pass (player action restrictions, cost modification scaffolding, lands_per_turn)
- **Scope:** Medium *(expanded from Small — now includes lands_per_turn computation per F18/§7)*
- **Source:** §5k, W13, W14 (+ PC5), F18 (chapter 3 audit, rule 305.2)
- **Depends on:** L04
- > **Session 6 audit note (see `session-6-audit-response.md`):** Game-rule-modifying effects (`ContinuousEffectKind::GameRuleModifying`) and cost modifications (`ContinuousEffectKind::CostModification`) are routed here, NOT through the L1–L7 layer loop. The `ContinuousEffectKind` discriminant (defined in L01) drives this routing.
- **Files:** `engine/layers.rs` (modify), `state/game_state.rs` (modify), `engine/costs.rs` (modify), `engine/cast.rs` (modify), `engine/combat/validation.rs` (modify), `oracle/characteristics.rs` (modify), `oracle/legality.rs` (modify)
- **Steps:**
  1. **PC5/W13 — Player action restrictions (613.10).** These restrict *what players can do*, not object characteristics. Distinct from cost modifications. Define `PlayerActionRestriction` enum with variants: `CantCastSpells(PlayerId)` (Silence), `CantGainLife(PlayerId)` (Erebos), `CantAttack(PlayerId)`, `CantActivateAbilities(PlayerId, Option<String>)` (Pithing Needle — optional card name filter), `CantDrawExtraCards(PlayerId)` (Narset). Store as `player_action_restrictions: Vec<PlayerActionRestriction>` on GameState, recomputed after each full layer pass. Wire checks into action gates: `cast.rs` check_cast_legality (CantCastSpells), `combat/validation.rs` validate_attackers (CantAttack), `actions.rs` execute_action for DrawCard/GainLife. Scaffolding — no Phase 5 cards populate these yet.
  2. **§5k — Cost modification scaffolding (separate concern).** These modify *how much* an action costs, not whether it's allowed. `CostModification` enum: `IncreaseCost(ManaCost)` (Thalia), `ReduceCost(ManaCost)` (Electromancer), `SetMinimumCost(u32)` (Trinisphere). Store as `cost_modifications: Vec<CostModification>` on GameState. Hook into the 601.2e cost pipeline (documented TODO in cast.rs). No Phase 5 cards use this either — scaffolding only.
  3. **W14:** Game-rule effects (613.11) confirmed as engine-level. No code change.
  4. **F18 — `lands_per_turn` computed query (rule 305.2).** `lands_per_turn` is a player-scoped value modified by continuous effects (Exploration grants +1, Azusa grants +2, etc.). Like player action restrictions, it's not an object characteristic — it's computed in the post-layer pass from base (1) + active continuous effects that grant additional land plays. Steps:
     - Add `effective_lands_per_turn: HashMap<PlayerId, u32>` to GameState, recomputed alongside `player_action_restrictions` after each full layer pass.
     - Define `LandPlayGrant { player: PlayerId, additional: u32, source: ObjectId }` as a post-layer output (or fold into an existing post-layer results struct).
     - Computation: start with base=1, scan `continuous_effects` for effects that grant additional land plays (new `Modification::GrantAdditionalLandPlays(u32)` variant or a `PlayerActionGrant` approach), sum them.
     - Implement `oracle::get_effective_lands_per_turn(game, player_id) -> u32` in `oracle/characteristics.rs`. T22 step 5 creates this as a passthrough; L15 replaces the passthrough with the real computation.
     - Update `oracle/legality.rs` `playable_lands` to call `get_effective_lands_per_turn` instead of reading `player.lands_per_turn` directly.
     - The raw `PlayerState.lands_per_turn` field becomes the *base* value (always 1 unless a replacement effect changes it). The oracle function returns the *effective* value. This matches the pattern of `get_effective_power`/`get_effective_toughness` routing through `compute_characteristics`.
     - No Phase 5 cards exercise this — scaffolding. Exploration/Azusa arrive when their cards are implemented.
- **Tests:**
  - `test_cant_cast_spells_blocks_casting` / `test_cant_attack_blocks_attackers` / `test_no_restrictions_default` / `test_cost_modification_scaffolding`
  - `test_effective_lands_per_turn_default_1` — no effects → returns 1
  - `test_effective_lands_per_turn_with_grant` — one GrantAdditionalLandPlays(1) effect → returns 2
  - `test_effective_lands_per_turn_stacks` — two grants → returns 3
  - `test_playable_lands_uses_effective` — `playable_lands` respects effective value, not raw field
- **Commit:** `engine: player action restrictions, cost modification scaffolding, lands_per_turn computation (§5k, W13, W14, PC5, F18)`

---

#### L16: All-zone static ability field (W7)
- **Scope:** Small
- **Source:** W7
- **Depends on:** L08
- **Files:** `objects/card_data.rs` (modify), `engine/layers.rs` (modify)
- **Steps:**
  1. Add `pub active_zones: Vec<Zone>` to `AbilityDef` (default: `vec![Battlefield]`).
  2. `register_static_abilities` checks zone match. CDAs handled separately (W6).
  3. No Phase 5 cards exercise this — forward-compatible architecture.
- **Tests:**
  - `test_active_zones_default` / `test_wrong_zone_not_registered`
- **Commit:** `objects: active_zones on AbilityDef (W7)`

---

### Sub-Plan 5C: Cards + Testing (L17–L21)

#### L17: Tier 1 cards — Honor, Tarmogoyf, Urborg, Blood Moon, Mind Snare
- **Scope:** Large
- **Source:** §5l
- **Depends on:** L08, L10, L11, L14
- **Files:** `cards/phase5_t1.rs` (modify)
- **Steps:**
  0. **CardDataBuilder color auto-derivation (from Phase 5-Pre session):** In `CardDataBuilder::build()`, auto-derive `colors` from `mana_cost` symbols per rules 202.2/105.3, eliminating redundant manual `.color()` calls. Add a `colors_explicitly_set: bool` flag to the builder; if false at `build()` time, derive colors from colored `ManaSymbol`s in the cost. Keep `.color()` for overrides (Devoid, colorless artifacts with colored activation costs, etc.). Remove `.color()` from all existing card definitions that don't need an override. Add tests: `test_build_derives_color_from_cost`, `test_build_explicit_color_overrides`, `test_build_colorless_no_cost`.
  1. **Honor of the Pure** — {1}{W} Enchantment. Static: L7c ModifyPT(1,1) on white creatures you control.
  2. **Tarmogoyf** — {1}{G} Creature—Lhurgoyf. **PC11:** `is_cda: true` on P/T ability. L7a SetPTDynamic(GraveyardCardTypes). Implement computation: count distinct CardType values across all graveyards; power=count, toughness=count+1.
  3. **Urborg** — Legendary Land. Static: L4 AddSubtypes(Swamp) on all lands. PC7: grants {T}: Add {B}.
  4. **Blood Moon** — {2}{R} Enchantment. Static: L4 SetLandType(Mountain) on nonbasic lands. Need nonbasic PermanentFilter. PC12: no Basic supertype.
  5. **Mind Snare** — {3}{U}{U} Instant. GainControl with WhileTargetOnBattlefield. **Audit change:** Instant (was Sorcery) to enable testing control-change during combat and at instant speed.
- **Tests:** Card data unit tests. Integration in L20.
- **Commit:** `cards: Tier 1 — Honor, Tarmogoyf, Urborg, Blood Moon, Mind Snare (§5l)`

---

#### L18: Last Known Information (LKI)
- **Scope:** Medium
- **Source:** E31 (deferred from T20b)
- **Depends on:** L04
- **Files:** `state/game_state.rs` (modify), `engine/zones.rs` (modify), `engine/resolve.rs` (modify)
- **Steps:**
  1. `LKISnapshot` wraps `EffectiveCharacteristics` + owner, zone, counters, card_data.
  2. `lki_cache: HashMap<ObjectId, LKISnapshot>` on GameState.
  3. In `move_object`, snapshot before zone transition via `compute_characteristics`.
  4. `query_lki(game, id) -> Option<&LKISnapshot>`.
  5. Wire into resolve_primitive for dead-target lookups (608.2h).
  6. **ObjectId semantics note:** Per MTG rules (400.7), an object that changes zones becomes a "new object" with no memory of its previous existence. However, our implementation reuses the same `ObjectId` (UUID) across zone transitions — `move_object` updates `obj.zone` in place rather than creating a new `GameObject`. The "new object" rule is enforced *behaviorally*: continuous effects expire via duration cleanup (L02), and re-entering the battlefield creates a new `BattlefieldEntity` with a fresh timestamp. The stable `ObjectId` is an implementation convenience that lets us key the LKI cache naturally — when an object moves zones, its old LKI is overwritten with the snapshot from its departing zone. If the same card later moves again (e.g., graveyard → exile), the LKI is overwritten again. This is correct: LKI is "last known," not "all previously known."
  7. C19: static abilities use current information, not LKI.
- **Tests:**
  - `test_lki_stored_on_zone_change` / `test_lki_has_effective_chars` / `test_lki_overwritten` / `test_query_unknown_returns_none`
- **Commit:** `state: LKI system wrapping EffectiveCharacteristics (E31)`

---

#### L19: Tier 2 cards — Humility + Opalescence
- **Scope:** Medium
- **Source:** §5l (Tier 2)
- **Depends on:** L05, L09, L10, L14
- **Files:** `cards/phase5_t2.rs` (create), `cards/mod.rs` (modify), `cards/registry.rs` (modify)
- **Steps:**
  1. **Humility** — {2}{W}{W} Enchantment. L6 RemoveAllAbilities + L7b SetPT(1,1) on all creatures.
  2. **Opalescence** — {2}{W}{W} Enchantment. L4 AddTypes(Creature) + L7b SetPTDynamic(ManaValue) on each other non-Aura enchantment. "Each other" excludes self. 613.6 locked targets tested.
  3. Humility+Opalescence: timestamp order determines winner in L7b (same sublayer).
- **Tests:** Card data unit tests. Integration in L20.
- **Commit:** `cards: Tier 2 — Humility, Opalescence (§5l)`

---

#### L20: Integration tests
- **Scope:** Large
- **Source:** §5m
- **Depends on:** L17, L18, L19
- **Files:** `tests/phase5_integration_test.rs` (create)
- **Steps:**
  1. **Tier 1 tests (21):** Giant Growth (buff/revert/SBA), Anthem (buff/destroy/stack), Honor (color filter), Tarmogoyf (empty/count/dynamic/+growth), Urborg (swamp/mana), Blood Moon (nonbasic/basic/no-Basic), Blood Moon+Urborg (dependency), Mind Snare (controller/sickness/haste/bounce).
  2. **Tier 2 tests (6):** Humility (1/1 no abilities/restore), Opalescence (animate/not-self), Opalescence+Humility (both timestamp orders), Tarmogoyf+Humility.
- **Acceptance:** All 27+ tests pass, 0 warnings. **5B Gate:** Blood Moon+Urborg correct. **5C Gate:** Opalescence+Humility correct.
- **Commit:** `tests: Phase 5 integration tests (§5m)`

---

#### L21: Fuzz regression
- **Scope:** Small
- **Source:** §5m
- **Depends on:** L17, L19, L20
- **Files:** `ui/random.rs` (modify), `bin/fuzz_games.rs` (modify)
- **Steps:**
  1. Add all Tier 1+2 cards to fuzz pool.
  2. Bump cast probability (~40% → ~60%) to exercise enchantments.
  3. Update `is_action_still_valid` for new card types if needed.
  4. Run 500+ games. Zero errors/panics.
- **Acceptance:** 500+ fuzz games pass.
- **Commit:** `test: fuzz regression with Phase 5 cards (§5m)`

> **Current state (2026-04-13):** Cast probability is already phase-aware (80% main / 30% non-main). Land play is 100%. Deck gen produces 60-card color-coherent 1–2 color decks. Seed control (`--seed`), event dump (`--dump-events`), and per-game action stats are all implemented. Target legality checks in `castable_spells` and `choose_targets` prevent illegal casts (Doom Blade on black creatures, Counterspell on empty stack, self-targeting, etc.). See **Fuzz Harness Upgrade Roadmap** in `roadmap.md` for planned future improvements.

---

### Plan Corrections Applied (Part 2)

| PC# | Correction | Applied in |
|-----|-----------|-----------|
| PC1 | `is_cda` source of truth on `AbilityDef`, not just `ContinuousEffect` | L01 |
| PC2 | `SetLandType` must grant intrinsic mana ability (inline in apply logic) | L10 |
| PC3 | CDAs handled as special case in `compute_characteristics` for all zones | L04 |
| PC4 | Timestamp changed from u64 to two-part struct for APNAP | L03 |
| PC5 | Post-layer pass for player action restrictions (613.10), separated from cost modifications | L15 |
| PC6 | `SetLandType` full 305.7 mechanism specified | L10 |
| PC7 | `AddSubtypes` for basic land type also grants intrinsic mana ability | L10 |
| PC8 | `controller_since_turn` (T09) is prerequisite for §5g | L11 |
| PC9 | `depends_on()` checks all four 613.8a conditions | L14 |
| PC10 | Static ability timestamp < spell continuous effect timestamp | L08 |
| PC11 | Tarmogoyf `is_cda: true` on ability definition | L17 |
| PC12 | Blood Moon test verifies no Basic supertype added | L10, L17 |

---

### Part 2 Deferred Items (no tickets — reference only)

| D# | Summary |
|----|---------|
| D1 | Layer 1 (Copy Effects) — Phase 6 with replacement effects |
| D2 | Layer 3 (Text-Changing) — semantic tree-walker, locked in for splice |
| D3 | Face-Down/Transform timestamps (613.7f–g) |
| D4 | Aura/Equipment re-timestamp on attach (613.7e) |
| D5 | Static ability timestamp = later of object vs. grant (613.7a) |
| D6 | Counter timestamps within L7c (613.7c) |
| D7 | "For as long as" duration failure (611.2b) |
| D8 | Deferred continuous effects — "next spell" (611.2f) |
| D9 | "Until" zone-change effects — O-Ring pattern (610.3) |
| D10 | Keyword counters in Layer 6 |
| D11 | Phasing interaction with layers |
| D12 | Bestow dual-nature permanent |
| D13 | Crew/Saddle type-changing abilities |
| D14 | Devotion uses partial-layer result |
| D15 | Exchange involving P/T |
| D16 | Continuous effects on stack (611.2a special case) |
| D17 | Cast legality look-ahead + game-rule flash grants (601.3) |
| D18 | Aura-creature SBA (303.4d) — Aura that is also a creature can't enchant; relevant when L4 adds Creature to Auras. Bestow (702.103) has its own escape valve. See T15 step 5 TODO. |
| D19 | Role SBA (303.7a) — if a permanent has >1 Role from the same controller, all but the newest go to graveyard. Requires attachment tracking (T04) + EnchantmentType::Role subtype. |
| D20a | Prototype base characteristics — `PrototypeStats` on `CardData` + `cast_as_prototype: bool` via `CastInfo`. Zone-sidecar scoped, stripped on zone change. Phase 9. See Discrepancy §13. |
| D20b | Perpetual modifications — `perpetual_modifications: Vec<PerpetualMod>` on `GameObject`. Ordered set/modify/ability ops that persist across zones. Phase 9 (Alchemy). See Discrepancy §13. |

---

## Cross-Part Ticket Dependency Graph

### Legend
- `→` = "depends on" (right must complete before left begins)
- `~~>` = coordination (recommended ordering, not blocking)
- Tickets with no incoming edges can start immediately
- **Bold** = critical path ticket

### Mermaid Diagram

```mermaid
graph LR
    subgraph "Tier 1 — Data Model (all parallel)"
        T01["T01 counters [P5-PREREQ]"]
        T02["T02 player counters"]
        T03["T03 is_token/is_copy"]
        T04["T04 attachment"]
        T05["T05 color_indicator [P5-PREREQ]"]
        T06["T06 x_value"]
    end

    subgraph "Tier 2 — State Tracking"
        T09["T09 controller_since_turn [P5-PREREQ]"]
        T10["T10 untap sickness"]
        T11["T11 LifeChanged source"]
        T12["T12 mana restrictions spike ✅"]
        T12b["T12b ManaPool sidecar"]
    end

    subgraph "Tier 3 — SBAs"
        T13["T13 counter annihilation + token SBA"]
        T14["T14 legend rule + PW loyalty"]
        T15["T15 Aura/Equip SBAs"]
        T15b["T15b Aura attachment"]
        T16["T16 poison/cmdr/indestructible"]
    end

    subgraph "Tier 4 — Casting"
        T17["T17 alt/add cost types"]
        T18a["T18a pipeline restructure"]
        T18b["T18b modes + targets"]
        T18c["T18c distribution + sacrifice"]
        T18d["T18d casting restrictions"]
        T19["T19 activation restrictions"]
        T20["T20 linked abilities"]
        T12c["T12c engine integration"]
    end

    subgraph "Tier 5 — Zone/Combat/Damage"
        T21a["T21a zone guards + CastInfo"]
        T21b["T21b combat/evasion"]
        T21c["T21c infect/PW damage/toxic"]
        T21d["T21d combat requirements solver"]
        T22["T22 duration + targeting"]
    end

    subgraph "Cross-Cutting — Mana Restrictions"
        T12d["T12d restricted cards + persistence"]
    end

    T09 --> T10
    T12 --> T12b
    T12b --> T12c
    T17 --> T12c
    T12c --> T12d
    L04 -.->|blanket persistence| T12d
    T01 --> T13
    T03 --> T13
    T01 --> T14
    T04 --> T15
    T04 --> T15b
    T02 --> T16
    T01 --> T16
    T17 --> T18a
    T18a --> T18b
    T18a --> T18c
    T18a --> T18d
    T17 --> T21a
    T01 --> T21c
    T02 --> T21c
    T21b --> T21d

    subgraph "Sub-Plan 5A"
        L01["L01 core types"]
        L02["L02 duration tracking"]
        L03["L03 timestamp struct"]
        L04["**L04** EffectiveChars"]
        L05["L05 locked sets"]
        L06["**L06** Layer 7 P/T"]
        L07["**L07** resolve hooks + Giant Growth"]
        L08["**L08** static reg + Anthem"]
    end

    L01 --> L02
    L01 --> L03
    L01 --> L04
    L03 --> L04
    L04 --> L05
    L04 --> L06
    T01 --> L06
    L06 --> L07
    L07 --> L08
    L02 --> L08
    T22 ~~> L02

    subgraph "Sub-Plan 5B"
        L09["L09 Layer 6 abilities"]
        L10["L10 Layers 4-5 type/color"]
        L11["L11 Layer 2 control"]
        L12["L12 L1/L3 stubs"]
        L13["L13 oracle migration"]
        L14["L14 dependency detection"]
        L15["L15 action restrictions"]
        L16["L16 all-zone static"]
    end

    L04 --> L09
    L04 --> L10
    T05 --> L10
    L04 --> L11
    T09 --> L11
    L04 --> L12
    L06 --> L13
    L09 --> L13
    L10 --> L13
    L11 --> L13
    L04 --> L14
    L04 --> L15
    L08 --> L16

    subgraph "Sub-Plan 5C"
        L17["**L17** Tier 1 cards"]
        L18["L18 LKI"]
        L19["L19 Tier 2 cards"]
        L20["**L20** integration tests"]
        L21["L21 fuzz regression"]
    end

    L08 --> L17
    L10 --> L17
    L11 --> L17
    L14 --> L17
    L04 --> L18
    L05 --> L19
    L09 --> L19
    L10 --> L19
    L14 --> L19
    L17 --> L20
    L18 --> L20
    L19 --> L20
    L17 --> L21
    L19 --> L21
    L20 --> L21
```

### Critical Path (longest sequential chain)

```
T01 → L06 → L07 → L08 → L17 → L20 → L21
```

With L01 → L03 → L04 as the prerequisite chain feeding into L06.

**Full critical path with prerequisites:**
```
L01 → L03 → L04 → L06 (+ T01) → L07 → L08 (+ L02) → L17 (+ L10, L11, L14) → L20 (+ L18, L19) → L21
```

### Parallel Branches (off critical path)

| Branch | Tickets | Joins at |
|--------|---------|----------|
| **Data model** | T02, T03, T04, T05, T06 (all parallel with T01) | Various Tier 3/5 deps |
| **State tracking** | T09 → T10, T11, T12 → T12b (parallel with Tier 1) | T09 joins at L11 |
| **Mana restrictions** | T12b + T17 → T12c (Tier 4) → T12d (cross-cutting; step 6 after L04) | T12d step 6 joins at L04 |
| **SBAs** | T13, T14, T15, T15b, T16 (after Tier 1 deps) | No Phase 5 deps |
| **Casting** | T17 → T18a → {T18b, T18c, T18d}, T19, T20 (parallel with SBAs) | T17 joins at T21a |
| **Combat/damage** | T21a, T21b → T21d, T21c (parallel) | No Phase 5 deps |
| **Duration stubs** | T22 (parallel) | Coordinates with L02 |
| **Layer 6** | L09 (after L04) | Joins at L13, L19 |
| **Layers 4-5** | L10 (after L04 + T05) | Joins at L13, L17, L19 |
| **Layer 2** | L11 (after L04 + T09) | Joins at L13, L17 |
| **L1 stub/L3 text-changing** | L12 (after L04) | No downstream |
| **Dependency** | L14 (after L04) | Joins at L17, L19 |
| **Scaffolding** | L15 (after L04) | No downstream |
| **All-zone static** | L16 (after L08) | No downstream |
| **LKI** | L18 (after L04) | Joins at L20 |
| **Locked sets** | L05 (after L04) | Joins at L19 |

### Cross-Part Dependencies (Part 1 → Part 2)

| Part 1 Ticket | Part 2 Ticket | Relationship |
|---------------|---------------|-------------|
| **T01** [P5-PREREQ] | L06 | L7c reads counters from BattlefieldEntity |
| **T05** [P5-PREREQ] | L10 | L5 applies color_indicator from CardData |
| **T09** [P5-PREREQ] | L11 | L2 control change updates controller_since_turn |
| T22 | L02 | L02 fills in T22's duration expiry stubs (coordination, not blocking) |

### Circular Dependency Check

**No circular dependencies found.** All edges are acyclic. The mermaid graph renders as a valid DAG.

---

## Verification Gates

### Gate 1: Tier 1 Data Model Complete
- **Trigger:** T01, T02, T03, T04, T05, T06 all merged
- **Criteria:**
  - All existing 287 tests pass
  - New data model unit tests pass (est. ~15 new)
  - 0 compiler warnings
  - `cargo test` clean
- **Significance:** Unblocks Tier 3 SBAs and Tier 5 combat/damage tickets

### Gate 2: All Part 1 Tickets Complete — Phase 5 Ready
- **Trigger:** All 24 Part 1 tickets (T01–T22, T15b, T21a–T21d) merged
- **Criteria:**
  - All tests pass (est. 287 existing + ~120 new = ~407 total)
  - 0 compiler warnings
  - 500/500 fuzz games pass (re-run with same harness)
  - **[P5-PREREQ] verification:** T01, T05, T09 confirmed merged
  - Duration expiry hooks in `turns.rs` placed (T22 step 4)
  - Cost modification pipeline stub exists (T18a step 2)
  - `DecisionProvider` trait has new methods with defaults (T14, T18a–c)
- **Significance:** Green light for Part 2. No Part 2 ticket may begin before Gate 2 passes.

### Gate 3: Sub-Plan 5A Complete — Giant Growth + Anthem Working
- **Trigger:** L01–L08 all merged
- **Criteria:**
  - Giant Growth on Grizzly Bears = 5/5 (2+3, 2+3)
  - Effect reverts at cleanup step (Bears return to 2/2)
  - Glorious Anthem on Bears = 3/3
  - Two Anthems stack: Bears = 4/4
  - Anthem destroyed: Bears revert to 2/2
  - Static ability timestamp < spell effect timestamp (PC10)
  - All existing + new tests pass
  - 0 warnings
- **Significance:** Layer engine foundation proven. Oracle routes through `compute_characteristics`.

### Gate 4: Sub-Plan 5B Complete — All 7 Layers Functional
- **Trigger:** L09–L16 all merged
- **Criteria:**
  - Blood Moon makes nonbasic lands into Mountains (only {T}: Add {R})
  - Blood Moon does NOT add Basic supertype (PC12)
  - Urborg makes all lands also Swamps (grants {T}: Add {B})
  - Blood Moon + Urborg: dependency system resolves correctly (Urborg depends on Blood Moon → Blood Moon applies first → Urborg sees Mountains → adds Swamp subtype → land has both Mountain and Swamp mana abilities)
  - `has_keyword` routes through layers (L09)
  - `is_creature` routes through layers (L10)
  - Control change re-sickens creature (L11/PC8)
  - All card_data reads migrated to oracle (L13/W3)
  - All existing + new tests pass
  - 0 warnings
- **Significance:** Full layer system operational. Ready for complex test cards.

### Gate 5: All Tickets Complete — Full Regression
- **Trigger:** L17–L21 all merged (all 45 tickets done)
- **Criteria:**
  - All Tier 1 integration tests pass (21 tests)
  - All Tier 2 integration tests pass (6 tests)
  - Opalescence + Humility resolves correctly for both timestamp orders
  - Tarmogoyf + Humility: Humility wins in L7b (later timestamp), Tarmogoyf is 1/1
  - LKI snapshots capture effective characteristics
  - **500+ fuzz games pass with zero errors/panics** (Phase 5 cards in pool)
  - Total test count: est. ~407 (Part 1) + ~80 (Part 2) = ~487+
  - 0 warnings
- **Significance:** Phase 5 complete. Ready for Phase 6 (triggered abilities + replacement effects).

---

## Comprehensive Coverage Matrix

### E-Items (E1–E48) → Part 1 Tickets

| E# | Description | Ticket | Status |
|----|-------------|--------|--------|
| E1 | Counters field on BattlefieldEntity | T01 | ✅ |
| E2 | CounterType enum expansion (evergreen keyword counters) | T01 | ✅ |
| E3 | Player counters (poison, commander damage) | T02 | ✅ |
| E4 | is_token flag | T03 | ✅ |
| E5 | is_copy flag | T03 | ✅ |
| E6 | Attachment tracking | T04 | ✅ |
| E7 | color_indicator on CardData | T05 | ✅ |
| E8 | P/T i32 signedness | *(resolved)* | ✅ verified |
| E9 | x_value on BattlefieldEntity | T06 | ✅ |
| E10 | Battle type in permanent check | *(resolved)* | ✅ fixed |
| E11 | Minor enum completeness | *(resolved)* | ✅ verified |
| E12 | controller_since_turn summoning sickness | T09 | ✅ |
| E13 | {Q} summoning sickness check | T10 | ✅ |
| E14 | LifeChanged event source field | T11 | ✅ |
| E15 | Mana spending restrictions (design spike) | T12 | ✅ |
| E15 | Mana spending restrictions (sidecar impl) | T12b | |
| E15 | Mana spending restrictions (engine integration) | T12c | |
| E15 | Mana spending restrictions (cards + persistence) | T12d | |
| E16 | Counter annihilation SBA | T13 | ✅ |
| E17 | Token cease-to-exist SBA | T13 | ✅ |
| E18 | Legend rule SBA | T14 | ✅ |
| E19 | Aura/Equipment legality SBAs | T15 | ✅ |
| E20 | Planeswalker loyalty SBA + ETB loyalty counters | T14 | ✅ |
| E21 | Poison counter loss SBA | T16 | ✅ |
| E22 | Commander damage loss SBA | T16 | ✅ |
| E23 | Indestructible SBA + Destroy guard | T16 | ✅ |
| E24 | Cleanup SBA re-loop | T16 | ✅ |
| E25 | Alternative/additional cost framework | T17+T18a | ✅ |
| E26 | Cost modification pipeline stub | T18a | ✅ |
| E27 | No-mana-cost casting guard | T18d | ✅ |
| E28 | Legendary spell restriction + CastingRestriction | T18d | ✅ |
| E29 | Activation restrictions on AbilityDef | T19 | ✅ |
| E30 | Linked abilities infrastructure | T20 | ✅ |
| E31 | Last Known Information (LKI) system | L18 *(deferred)* | ✅ |
| E32 | Mana ability debug assertion | T20 | ✅ |
| E33 | Zone-activated abilities | T19 | ✅ |
| E34 | Instant/Sorcery battlefield guard | T21a | ✅ |
| E35 | Spell cost info carried to permanent | T21a | ✅ |
| E36 | Combat removal on control/type change | T21b | ✅ |
| E37 | Evasion framework expansion | T21b | ✅ |
| E38 | Trample co-assigned damage | T21b | ✅ |
| E39 | Infect/Wither damage routing | T21c | ✅ |
| E40 | Planeswalker/Battle damage routing | T21c | ✅ |
| E41 | Toxic poison counter addition | T21c | ✅ |
| E42 | Duration expiry hooks | T22 | ✅ |
| E43 | Duration::UntilEndOfCombat | T22 | ✅ |
| E44 | "This turn" effect tracking | T22 | ✅ |
| E45 | lands_per_turn dynamic | T22 | ✅ |
| E46 | Hexproof in targeting | T22 | ✅ |
| E47 | Shroud in targeting | T22 | ✅ |
| E48 | Protection targeting restriction | T22 | ✅ |

**48/48 E-items covered. 0 omissions.** 3 resolved in-place (E8/E10/E11). 1 deferred to Part 2 (E31 → L18).

---

### §5 Steps (§5a–§5m) → Part 2 Tickets

| Step | Description | Ticket | Status |
|------|-------------|--------|--------|
| §5a | Core types (ContinuousEffect, Layer, Modification) | L01 | ✅ |
| §5b | Duration tracking + cleanup hooks | L02 | ✅ |
| §5c | Timestamp struct, EffectiveCharacteristics, locked sets | L03, L04, L05 | ✅ |
| §5d | Layer 7 P/T + counters in 7c | L06 | ✅ |
| §5e | Layer 6 abilities | L09 | ✅ |
| §5f | Layers 4-5 type + color + SetLandType 305.7 | L10 | ✅ |
| §5g | Layer 2 control | L11 | ✅ |
| §5h | Layers 1+3 stubs | L12 | ✅ |
| §5i | Dependency detection (613.8) | L14 | ✅ |
| §5j | Hook resolve_primitive + static registration | L07, L08 | ✅ |
| §5k | Post-layer pass + cost scaffolding | L15 | ✅ |
| §5l | Test cards (Giant Growth, Anthem, Tier 1, Tier 2) | L07, L08, L17, L19 | ✅ |
| §5m | Integration tests + fuzz regression | L20, L21 | ✅ |

**13/13 §5 steps covered. 0 omissions.**

---

### W-Items (W1–W15) → Part 2 Tickets

| W# | Description | Ticket | Status |
|----|-------------|--------|--------|
| W1 | ContinuousEffect struct definition | L01 | ✅ |
| W2 | Oracle query functions for effective values | L13 (via L06/L09/L10/L11) | ✅ |
| W3 | Migrate card_data reads to oracle | L13 | ✅ |
| W4 | EffectiveCharacteristics fields + scope | L01, L04 | ✅ |
| W5 | CDA ordering (CDAs first, then non-CDAs) | L04, L09, L10 | ✅ |
| W6 | CDA all-zone evaluation | L04 | ✅ |
| W7 | All-zone static ability field | L16 | ✅ |
| W8 | Locked-in target sets (613.6) | L05 | ✅ |
| W9 | Dependency detection implementation | L14 | ✅ |
| W10 | Timestamp APNAP sub-ordering | L03 | ✅ |
| W11 | Static ability registration timing | L08 | ✅ |
| W12 | SetLandType 305.7 mechanism | L10 | ✅ |
| W13 | Player action restrictions (613.10) | L15 | ✅ |
| W14 | Game-rule effects (613.11) — design confirmation | L15 | ✅ |
| W15 | Targeting uses effective types | L13 | ✅ |

**15/15 W-items covered. 0 omissions.**

---

### PC-Items (PC1–PC12) → Part 2 Tickets

| PC# | Correction | Ticket | Status |
|-----|-----------|--------|--------|
| PC1 | `is_cda` on AbilityDef | L01 | ✅ |
| PC2 | SetLandType grants intrinsic mana | L10 | ✅ |
| PC3 | CDAs as special case in compute_characteristics | L04 | ✅ |
| PC4 | Timestamp two-part struct | L03 | ✅ |
| PC5 | Player action restrictions separated from cost mods | L15 | ✅ |
| PC6 | SetLandType full 305.7 | L10 | ✅ |
| PC7 | AddSubtypes grants intrinsic mana for basic land types | L10 | ✅ |
| PC8 | controller_since_turn prerequisite for L2 | L11 | ✅ |
| PC9 | depends_on checks all four 613.8a conditions | L14 | ✅ |
| PC10 | Static ability timestamp < spell effect timestamp | L08 | ✅ |
| PC11 | Tarmogoyf is_cda: true | L17 | ✅ |
| PC12 | Blood Moon no Basic supertype | L10, L17 | ✅ |

**12/12 PC-items covered. 0 omissions.**

---

## Discrepancies & Notes

### 1. T22/L02 Duration Variant Overlap
T22 adds `UntilEndOfYourNextTurn` to the `Duration` enum. L02 also adds `UntilEndOfYourNextTurn` and `WhileTargetOnBattlefield`. If T22 lands first (recommended), L02 should not re-add `UntilEndOfYourNextTurn` — verify it already exists and only add `WhileTargetOnBattlefield`. If L02 lands first, T22's addition becomes a no-op for that variant. **No correctness issue — watch for merge conflicts only.**

### 2. L12 Layer 3 Text-Changing — RESOLVED
~~The `phase5-pre-work-final.md` audit upgraded D2 (Layer 3 text-changing) from "deferred" to "locked in for Phase 5." L12 only verified the L3 stub.~~ **Resolved:** L12 has been upgraded in-place from Small (stub verification) to Medium (full L3 implementation). It now defines the `TextChange` enum, implements `apply_text_change_to_effect` tree-walker, and wires it into the L3 slot in `compute_characteristics`. Layer 1 remains a stub (Phase 6). See L12 ticket for details.

### 3. T15b Not an E-Item
T15b (Aura attachment logic) is infrastructure surfaced by T15, not sourced from an E-item. It is correctly listed in Part 1 and counted in the 24-ticket total. No coverage gap.

### 4. No Conflicts Between Parts
- No ticket in Part 1 modifies the same file section as a ticket in Part 2 (except T22/L02 coordination noted above).
- No E-item is claimed by both a Part 1 and Part 2 ticket (E31 is explicitly deferred from T20b to L18).
- All [P5-PREREQ] tickets (T01, T05, T09) are in Part 1 and appear in Gate 2 verification.

### 5. DecisionProvider Trait Growth
Part 1 adds: `choose_legend_to_keep`, `choose_modes`, `choose_x_value`, `choose_distribution`, `choose_alternative_cost`, `choose_additional_costs` (~6 new methods). All have default implementations. `ScriptedDecisionProvider` and `RandomDecisionProvider` will need updates for any non-default behavior. No blocking issue — the default impls prevent compilation failures. **Note (updated 2026-04-12):** ~~Aura ETB without targeting (T15b step 3) reuses `choose_targets` with a `TargetContext::EnchantmentPlacement` discriminant~~ **Actual implementation:** No `TargetContext` enum was created. Instead, the `EffectRecipient::Choose(SelectionFilter, TargetCount)` variant conveys that targeting rules don't apply. `attach_aura_on_etb` in `resolve.rs` passes a `Choose` variant to the DP's `choose_targets` method — the DP sees the same target selection UI, and the engine knows not to apply hexproof/shroud/protection rules because it's a `Choose`, not a `Target`.

### 6. Combat Requirements Architecture — Attack (508.1d) and Block (509.1c) — RESOLVED

**Resolved:** New ticket **T21d** (combat requirements solver) has been created in Tier 5, depending on T21b. It implements the requirement-maximizing solver, expands `BlockRequirement` and `AttackRequirement` enums (including `AllMustBlock` for attacker-side lure effects vs. `MustBlockSpecific` for blocker-side forced blocks, and `Goaded` for goad's combined requirement+restriction), and adds population hook scaffolding. See T21d ticket for full details.

**Key design decision preserved:** Blocking requirements are split into two structurally distinct categories:
- **Attacker-side lure** (`AllMustBlock`): fans out into per-blocker constraints at solve time
- **Blocker-side forced block** (`MustBlockIfAble`, `MustBlockSpecific`): single-creature constraints

Actual population from continuous effects/designations (goad, Lure, etc.) still requires triggered abilities (Phase 6) and per-permanent designations (D17).

### 7. T22 Step 5 — `lands_per_turn` Needs Computed Query — RESOLVED
~~The chapter 3 audit (F18, rule 305.2) identifies that `lands_per_turn` must be modifiable by continuous effects. T22 creates a passthrough, L15 should compute the real value.~~ **Resolved:** L15 has been expanded from Small to Medium. It now includes step 4 (F18) which adds `effective_lands_per_turn: HashMap<PlayerId, u32>` to GameState, recomputed in the post-layer pass from base (1) + active continuous effects. `oracle::get_effective_lands_per_turn` (created by T22 as a passthrough) is replaced with the real computation. `playable_lands` in `oracle/legality.rs` reads the effective value. See L15 ticket step 4 for details.

### 9. T18 Scope — SPLIT DONE (2026-04-13)

~~T18 (casting pipeline 601.2 compliance) combines 4 E-items (E25 part 2, E26, E27, E28), restructures the most complex file in the engine (`cast.rs`), adds 5 new DP methods, and includes 11 tests. This is a lot of orthogonal concerns for one ticket.~~ **Split completed.** T18 has been split into four sub-tickets:
- **T18a:** Pipeline restructure + X value + cost assembly + rollback (E25p2 structural, E26)
- **T18b:** Mode choice + conditional targets + target uniqueness rules (E25p2 modes/targets)
- **T18c:** Distribution + Cost::Sacrifice + payment ordering + partial resolution (E25p2 distribution, 601.2d/h, 608.2b/d/i)
- **T18d:** Casting restrictions + no-mana-cost guard + legendary sorcery (E27, E28)

Dependency graph: `T17 → T18a → {T18b, T18c, T18d}`. T18b/c/d are parallel after T18a.

### 10. Test Count Estimates
- **Pre-plan baseline:** 287 tests (238 unit + 48 integration + 1 doc-test)
- **Current actual (2026-04-12):** 370 tests (312 unit + 48 integration + 1 doc-test + 9 pre-Phase3). T14, T15b, T16, and EffectRecipient refactor complete.
- **Post-Part 1 estimate:** ~470+ tests (370 current + ~100 from remaining ~20 tickets)
- **Post-Part 2 estimate:** ~554+ tests (+84 from 21 tickets, includes L12 and L15 expansions)
- **Fuzz:** 500+ games at Gate 2 and Gate 5

---

**All 48 E-items, 13 §5 steps, 15 W-items, and 12 PC-items are covered by the 45 active tickets. 0 omissions. 0 circular dependencies. 0 conflicts. Additionally: combat requirements solver (508.1d/509.1c) incorporated via T21d, Layer 3 text-changing incorporated via L12 upgrade, lands_per_turn computed query incorporated via L15 expansion, 611.2c spell/ability lock-in incorporated via L07 expansion, creature-type P/T gate incorporated via L04 clarification, Aura-creature SBA (303.4d) and Role SBA (303.7a) added to deferred items (D18/D19).**

---

### Pass0 Architectural Decisions — Cross-Reference

> Source: `plans/atomic-tests/pass0-dependency-map.md` Section 8.
> Updated: 2026-04-10, after atomic test catalog triage and merge pipeline abandonment.

### Coverage Matrix

| Decision | Source META | Needed By | Coverage | Notes |
|----------|-----------|-----------|----------|-------|
| `LinkedAbilityData` storage per permanent | META-LINKED-ABILITY-STORAGE | Phase 5-Pre (T20) | **T20 ✅** | `linked_group: Option<u32>` on `AbilityDef` + `linked_ability_data: HashMap<(ObjectId, u32), Vec<ObjectId>>` on `GameState`. |
| `ProtectionQuality` enum + `matches_quality()` | META-7B-02 | Phase 5-Pre (T22) | **T22 step 8 ✅** | Four variants: `Color`, `CardType`, `Subtype`, `All`. Targeting validation uses source characteristics. |
| `EvasionRestriction` + `BlockerFilter` enum | META-7B-01 | Phase 5-Pre (T21b) / Phase 8 | **T21b ✅** | Three-category framework (per-pair, per-pair contextual, count-based). Shadow/Fear/Intimidate/Skulk/Horsemanship/Landwalk/Menace all covered. Phase 8 cards populate the framework. |
| Menace enforcement in `validate_blockers` | *(existing code bug)* | Immediate | **T21b step 2 (Category C) ✅** | Min-blockers count check in `validate_full_block_assignment`. |
| Casting rollback via GameState snapshot | META-GAMESTATE-SNAPSHOT | Phase 5-Pre (T18a) | **T18a step 1 ✅ (implicit)** | See Discrepancy §11 below for the explicit rollback mechanism note added to T18a. |
| `ObjectRef` with epoch-stamp for stale references | META-EPOCH-STAMP | Phase 5-Pre (T22) | **Superseded** | See Discrepancy §12 below. Rule 400.7 "new object" semantics are handled behaviorally by stable `ObjectId` + LKI (L18), not by an explicit epoch-stamp type. |
| Prototype + Perpetual base characteristic overrides | *(roadmap line 109)* | Phase 9 | **Deferred items D20a/D20b added** | See Discrepancy §13 below. Prototype and Perpetual are architecturally distinct — split into separate mechanisms. No Phase 5 scaffolding needed. |
| `TurnEventLog` for multi-condition triggers | META-MULTI-CONDITION-TRIGGERS | Phase 7 | **Roadmap Phase 7 ✅** | Superseded by delta log architecture (`engine/delta_log.rs`). Trigger scanner reads structured `GameDelta` entries, not a raw event log. Documented in `design_doc.md §8`, `roadmap.md` Phase 7 infrastructure. |
| Two-pass trigger stacking | META-TWO-TIER-TRIGGER-STACKING | Phase 7 | **Roadmap Phase 7 ✅** | APNAP ordering of triggered abilities on stack placement (rule 603.3b). Beyond impl plan scope (Phase 5 only). |
| Trigger checking after all replacements finalize | META-HIDDEN-ZONE-TRIGGER-COMPLEXITY | Phase 7 | **Roadmap Phase 7 ✅** | Delta log checkpoints run after SBA cycles and event batches, ensuring triggers see post-replacement state. |
| `choose_ordering` DP consolidation | META-DP-ORDERING-CONSOLIDATION | Phase 7 | **Roadmap Phase 7 ✅** | Trigger ordering via APNAP. Specific DP method design deferred to Phase 7 ticketing. |
| Copy-spell vs copy-card distinction | META-7B-03 | Phase 7 (D19) | **Deferred D19 ✅** | Listed in Part 1 deferred items. Phase 7 ticket will distinguish copy-on-stack (Storm) from copy-as-permanent (Clone). |
| `TrampleContext` for unified trample DP | META-7B-04 | Phase 8 | **Roadmap Phase 8 ✅** | T21b step 3 adds `pre_assigned_damage` parameter as the immediate fix. Full `TrampleContext` struct deferred to Phase 8 when more trample variants arrive. |
| Splice as temporary resolution extension | META-7B-07 | Phase 8 | **Roadmap Phase 8 ✅** | Cluster L in pass0. Text-modification infrastructure (L12 tree-walker) supports splice. Actual splice implementation is Phase 8. |
| `can_begin_casting()` unified permission check | META-CAST-PERMISSION-LAYERS | Phase 8 (D17) | **Deferred D17 ✅** | Listed in Part 1 deferred items. Oracle routing (L13) enables this naturally. |

### Result: **14/14 decisions covered.** 12 directly by tickets or roadmap phases. 2 superseded by better designs (epoch-stamp → LKI; TurnEventLog → delta log). 2 new deferred items added (D20a: Prototype, D20b: Perpetual — split from original shared `CharacteristicOverrides` proposal).

---

### 11. T18a — Casting Rollback Mechanism (META-GAMESTATE-SNAPSHOT)

T18a step 1 restructures `cast_spell` to follow 601.2a–i ordering. Step 601.2a moves the spell to the stack *before* the full legality check (601.2e). If the post-proposal legality check fails, the engine must undo the stack placement. **Rollback mechanism:** Use `GameState::clone()` before 601.2a, restore on failure. This is a coarse-grained snapshot — acceptable because casting failures are rare and the clone is shallow (only the state that matters is the object's zone + stack contents). A finer-grained `UndoEntry` approach (recording only the zone change) is an optimization for later if profiling shows clone cost is material. The `GameState::clone()` approach is consistent with how `fuzz_games.rs` already clones state for parallel evaluation.

### 12. ObjectRef Epoch-Stamp — SUPERSEDED

The pass0 `META-EPOCH-STAMP` decision proposed an `ObjectRef { id: ObjectId, epoch: u64 }` type to detect stale references after zone changes (rule 400.7: "an object that moves from one zone to another becomes a new object"). **This is superseded by the current design:**

- `ObjectId` is stable across zone transitions (`move_object` updates `obj.zone` in place).
- The "new object" rule is enforced **behaviorally**: continuous effects expire via duration cleanup (L02), re-entering the battlefield creates a new `BattlefieldEntity` with a fresh timestamp, and LKI (L18) snapshots the departing characteristics.
- Stale-reference detection for targeting is handled by `validate_targets` checking that the target still exists in the expected zone — no epoch needed.
- If a future edge case requires epoch-based staleness detection (e.g., "exile target creature, then return it" — the returned creature is a new object that shouldn't be found by effects targeting the old one), the `BattlefieldEntity.timestamp` already serves as an effective epoch within the battlefield zone. For cross-zone staleness, a `zone_change_counter: u64` on `GameObject` could be added as a targeted fix without introducing `ObjectRef` everywhere.

**No ticket needed.** The existing infrastructure is sufficient.

### 13. Prototype + Perpetual — Revised Design (D20a / D20b)

The roadmap (line 109) originally listed a shared `CharacteristicOverrides` struct for both Prototype and Perpetual, defined in Phase 5 so `compute_characteristics` could reference it early. After deeper analysis (session-8-audit-response.md Proposal C), **this shared struct is architecturally unsound.** Prototype and Perpetual differ on every axis: storage location, application semantics, stripping behavior, and stacking model. They are split into two independent deferred items.

#### D20a: Prototype (Phase 9 — alongside DFC infrastructure)

**What it is:** A single-faced card with alternative base characteristics (P/T, color, mana cost) used when cast for the prototype cost. Zone-dependent: overrides apply on stack/battlefield only, stripped on zone change. A copiable value per 702.160b.

**Design:** Prototype is a **cast-mode memory** — the entire override set is derivable from `card_data` + one boolean.

```rust
// On CardData:
pub prototype_stats: Option<PrototypeStats>,

pub struct PrototypeStats {
    pub power: i32,
    pub toughness: i32,
    pub mana_cost: ManaCost,
    pub colors: Vec<Color>,
    // Note: Prototype does NOT change abilities or types (702.160a)
}

// CastInfo (already carried from stack to permanent via T21a):
pub cast_as_prototype: bool,
```

`compute_characteristics` checks `cast_info.cast_as_prototype` — if true and object is on stack/battlefield, use `card_data.prototype_stats` for P/T/color/mana_cost, fall through to `card_data` for everything else. Zone change strips `BattlefieldEntity` (and its `CastInfo`), so the prototype override disappears naturally.

**Why not a shared struct:** Prototype values are printed on the card (static, known at compile time). They don't stack. They affect a fixed set of characteristics (no abilities). A single bool + a CardData field is all that's needed.

#### D20b: Perpetual Modifications (Phase 9 — Alchemy/digital-only)

**What it is:** Effects that permanently modify an object's characteristics in ways that survive zone changes. NOT in the Comprehensive Rules — Alchemy-originated, will require ad-hoc testing.

**Key insight: Perpetual operations are ordered and heterogeneous.** A card can accumulate multiple perpetual effects over time, mixing **set** and **modify** operations:

| Alchemy text | Operation type | Example |
|---|---|---|
| "Power perpetually becomes 0" | **Set** P/T | Overwrite base power |
| "Perpetually gets +1/+2" | **Modify** P/T | Delta on top of current base |
| "Perpetually becomes black" | **Set** color | Replace color identity |
| "Perpetually gains flying" | **Add** ability | Grant keyword |
| "Perpetually loses all abilities" | **Remove** abilities | Strip all |

Order matters: "power becomes 0" then "+1/+2" = power 2. "+1/+2" then "power becomes 0" = power 0. This means perpetual modifications are a **log of operations**, not a bag of Option fields.

**Design:**

```rust
// On GameObject (persists across zones):
pub perpetual_modifications: Vec<PerpetualMod>,

pub enum PerpetualMod {
    SetPower(i32),
    SetToughness(i32),
    ModifyPower(i32),          // delta
    ModifyToughness(i32),      // delta
    SetColors(Vec<Color>),
    AddAbility(AbilityDef),
    RemoveAbility(KeywordAbility),
    RemoveAllAbilities,
    SetManaValue(ManaCost),
    // Extend as Alchemy mechanics require — see alchemy-mechanics-audit.md Q1 for
    // planned variants: AddColor, ReplaceCardData, keyword-specific variants
}
```

`compute_characteristics` applies the vec in order after CardData base but before the layer loop:

```rust
// In compute_characteristics:
let mut chars = EffectiveCharacteristics::from_card_data(card_data);

// D20a: Prototype override (zone-sidecar, replaces base)
if cast_info.cast_as_prototype {
    if let Some(proto) = &card_data.prototype_stats {
        chars.power = Some(proto.power);
        chars.toughness = Some(proto.toughness);
        chars.colors = proto.colors.clone();
        chars.mana_cost = proto.mana_cost.clone();
    }
}

// D20b: Perpetual modifications (object-level, applied in order)
for m in &obj.perpetual_modifications {
    match m {
        PerpetualMod::SetPower(p) => chars.power = Some(*p),
        PerpetualMod::ModifyPower(d) => {
            chars.power = chars.power.map(|p| p + d);
        }
        PerpetualMod::AddAbility(a) => chars.abilities.push(a.clone()),
        PerpetualMod::RemoveAllAbilities => chars.abilities.clear(),
        // ... etc
    }
}

// Then: layer loop (L1–L7) applies continuous effects on top
```

**Interaction: Prototype + Perpetual.** A Prototype creature with perpetual buffs: Prototype sets base P/T (step 1), perpetual mods apply on top (step 2), layer effects apply on top of that (step 3). If the creature bounces to hand, Prototype override disappears (CastInfo stripped), but perpetual mods persist on the `GameObject`. In hand, the creature's characteristics are: CardData base → perpetual mods. Correct.

**Why a Vec, not a struct of Options:** Because "power becomes 0" followed by "+1/+2" ≠ "+1/+2" followed by "power becomes 0". Order-dependent operations require an ordered log. A flat struct with `power: Option<i32>` can only represent the *last* set/modify — it loses history and can't correctly resolve mixed set+modify sequences.

#### L04 TODO Comment

L04's `compute_characteristics` should include:
```rust
// TODO D20a: Check cast_info.cast_as_prototype → apply card_data.prototype_stats
// TODO D20b: Apply obj.perpetual_modifications in order before layer loop
```

No struct definitions needed in Phase 5. Both mechanisms are Phase 9, and no Phase 5 ticket depends on them. The TODO comments mark the insertion points.

Added to Part 2 Deferred Items table as D20a and D20b.

---

### 14. TargetSpec → EffectRecipient Refactor — COMPLETED EARLY (2026-04-12)

The original T18 follow-up note proposed splitting `TargetSpec` into `EffectRecipient` during the T18 casting pipeline rewrite. This refactor was completed early during the T15b MR, before T18 began. **Impact across the plan:**

- **T18a–d:** No longer need to perform the split. `EffectRecipient` with `Target`/`Choose`/`Implicit`/`Controller` variants is already in place. T18a–d can build directly on this for mode choice, X value, and distribution.
- **T20:** Mana ability debug assertion should check for `EffectRecipient::Target(_, _)` instead of the deleted `TargetSpec::None`.
- **T22:** Hexproof/shroud/protection checks must only apply to `EffectRecipient::Target`, not `Choose`. The validation path already dispatches both — the keyword checks need to be gated inside the `Target`-specific path.
- **Discrepancy §5:** No `TargetContext` enum was created. `EffectRecipient::Choose` serves the same purpose.
- **14 files, 109 match sites** updated. Variable renamed `target_spec` → `recipient` everywhere. `validate_selection` dispatcher added. Fizzle fix: `Choose` effects no longer participate in 608.2b fizzle check.

**No tickets were skipped or reduced in scope.** The refactor was additive infrastructure that happened to align with T15b's needs (Aura ETB choosing).

---

### 15. SPECIAL-1c Post-Migration Audit — Findings (2026-04-14)

After completing the engine-side migration from the old typed `DecisionProvider` methods to the new `ask_*` / 4-primitive trait, an audit revealed three issues:

#### 15a. L127 Bug — First-Atom-Only Recipient Extraction

`cast_spell` at `engine/cast.rs:127` extracts `recipient` from the first `Effect::Atom` in a `Sequence` and skips target selection if it's `Implicit` or `Controller`. This is wrong for multi-atom sequences where a later atom has a `Target`/`Choose` recipient (e.g. "You draw two cards. Target opponent discards two cards" — first atom is `Controller`, second is `Target`). The target prompt is silently skipped.

**Fix:** T18b step 7 (per-atom targeting) replaces the first-atom-only extraction with a scan of ALL atoms. Until T18b, no existing cards exercise this pattern (all current spell abilities have a single atom or a sequence where the first atom carries the targeting). **Severity:** latent bug, no gameplay impact yet.

#### 15b. Priority Retry Gap — Failed Cast/Activate Errors Out

When `ask_choose_priority_action` returns `CastSpell(id)` or `ActivateAbility(id, aid)` and the subsequent execution fails (can't pay costs, targets become illegal, etc.), `priority.rs` propagates the `Err` upward instead of re-prompting the player. Per rule 601.2e/601.5, a failed cast attempt should roll back game state and the player should receive priority again to make a different choice.

**Current behavior:** `cast_spell` partially rolls back (moves card from stack back to hand on target validation failure), but `run_priority_round` treats any `Err` from `cast_spell` as terminal and returns it to the caller. A human player at the CLI who selects "Cast Spell" and then can't pay has no way to back out — the game errors.

**Severity:** Real usability bug. Affects all DPs (CLI, Random, Scripted). Random DP works around it via `is_action_still_valid` pre-check, but this is a heuristic that doesn't catch all failure modes.

**Design for fix (new ticket SPECIAL-2):**

The priority loop must catch execution failures and retry:

```rust
// In run_priority_round, CastSpell arm:
PriorityAction::CastSpell(card_id) => {
    match self.cast_spell(current_priority, card_id, decisions) {
        Ok(()) => {
            self.perform_sba_and_triggers(decisions)?;
            return Ok(PriorityResult::ActionTaken);
        }
        Err(_e) => {
            // Cast failed — player gets priority again.
            // Game state was rolled back by cast_spell.
            // Do NOT reset consecutive_passes — a failed cast is not
            // a game action; if the player subsequently passes, that's
            // still a consecutive pass.
            continue;
        }
    }
}
```

Same pattern applies to `ActivateAbility`. The `PlayLand` arm can use the same pattern but land plays rarely fail (static legality is exact).

**Deeper walkback (long-term UX):** Allowing the DP to cancel *during* the casting pipeline (e.g., a CLI player who starts casting, sees the target list, and decides not to cast) requires either a sentinel "cancel" return from `pick_n`/`pick_number` or a cancellation-aware middleware wrapper. Both are non-trivial. This is a long-term UX priority (D27), not a near-term concern.

**Where to schedule:** SPECIAL-2 is a non-blocking quality-of-life fix — small scope (~20 lines of engine code + 3-4 tests), no ticket depends on it. Can be picked up opportunistically whenever convenient. Not gated on any phase.

#### 15c. Blocker Candidate Overapproximation — Flying/Reach Not Pre-Filtered

`legal_blockers()` in `oracle/legality.rs` returns all untapped creatures controlled by the defender, without checking flying/reach compatibility against specific attackers. The cross-product `(blocker, attacker)` in `process_declare_blockers` includes illegal pairs like "ground creature blocks flyer." These are caught by `validate_blockers()` post-selection, so correctness is maintained, but the DP sees options it can't legally pick.

**Severity:** Low — validation catches it. But becomes a UX issue for CLI (confusing options) and an efficiency issue for Random DP (wasted retries). The fix is straightforward: filter the `legal_block_pairs` cross-product through the flying/reach per-pair check before passing to `ask_choose_blockers`. This is a refinement that can be done anytime — no ticket dependency.

---

### 16. Deferred Items — Part 1 Additions (2026-04-14)

| D# | Summary |
|----|---------|
| D26 | Priority action retry on failed cast/activate — `run_priority_round` should catch `Err` from `cast_spell`/`activate_ability` and re-prompt instead of propagating. See Discrepancy §15b. Ticket: SPECIAL-2. Non-blocking QoL, pick up anytime. |
| D27 | Mid-pipeline cast cancellation — allow DP to cancel during 601.2b–c (target selection, cost choices). Long-term UX priority. Requires sentinel return value or cancellation-aware middleware wrapper. See Discrepancy §15b. |
| D28 | Shared test helpers — extract `ScriptedDecisionProvider` from `decision.rs` into dedicated module, add common DP script sequence helpers (`queue_turn_passes_with_no_attacks`, `queue_cast_and_resolve`, etc.). Ticket: SPECIAL-3. Non-blocking QoL, pick up anytime after SPECIAL-1c. |
| D29 | CounterSpell/CounterAbility cleanup — replace manual stack/zone manipulation in `resolve.rs` with `move_object`. Ticket: SPECIAL-4. Non-blocking cleanup, pick up anytime. |
| D31 | DP validation + contract property tests (Classes A/D) — centralized `#[should_panic]` coverage for `validate_*` and property tests for `RandomDecisionProvider` against the 4-primitive contract. Ticket: SPECIAL-5. Non-blocking QoL, pick up anytime after SPECIAL-1c. |
| D32 | `ask_*` option enumeration tests (Class C) — assert the engine presents correct `ChoiceOption` lists for each `ask_*` function. Living ticket; every future `ask_*`/`ChoiceKind`/`SelectionFilter`/`PermanentFilter` change must add at least one Class C test in the same PR. Ticket: SPECIAL-6. Non-blocking QoL, pick up anytime after SPECIAL-1c. |
| D30 | Declare attackers Cartesian product refactor — `process_declare_attackers` builds `O(creatures × targets)` pairs. With planeswalkers and battles as attack targets (T21b+), even 2-player games can grow large. Refactor to two-step approach: (1) pick which creatures attack, (2) assign each a target. Reduces options from `O(creatures × targets)` to `O(creatures + creatures)`. Relevant after T21b adds PW/battle attack targets. See TODO in `engine/combat/steps.rs:34-37`. |

---

## Atomic Test Catalog — Authoritative Source Reference

> Updated: 2026-04-10, after abandoning the lossy LLM merge pipeline.

### Pipeline Status

The original multi-stage merge pipeline (Pass 1 → Pass 2 → Stage 2B catalogs) has been **abandoned** due to:
1. Pass 1 merge dropped ~70% of testable ATOMs per half (output token limits)
2. Half B misclassified 84 ATOM-702.81+ keyword tests under Phase 7 instead of Phase 8
3. Cascading fixes proved unworkable

### Current Authoritative Sources

| File | Contents | Entry Count |
|------|----------|-------------|
| `atomic-tests/global-test-index.md` | All ATOMs/BOUNDARYs/COMPs with phase assignment | 1780 |
| `atomic-tests/phase-index-phase-5-pre.md` | Phase 5-Pre test index | 280 |
| `atomic-tests/phase-index-phase-5-layers.md` | Phase 5-Layers test index | 178 |
| `atomic-tests/phase-index-phase-6.md` | Phase 6 test index | 138 |
| `atomic-tests/phase-index-phase-7.md` | Phase 7 test index | 202 |
| `atomic-tests/phase-index-phase-8.md` | Phase 8 test index | 547 |
| `atomic-tests/phase-index-phase-9.md` | Phase 9 test index | 226 |
| `atomic-tests/phase-index-deferred.md` | Deferred (unassigned phase) | 3 |

### Workflow: Finding the Full Spec for an ATOM

1. **Identify which ATOMs** to implement from the relevant `phase-index-phase-*.md` file
2. **Find the session** from the `Session` column (e.g., `S5` → `sessions/session-5.md`)
3. **Grep the session file** for the ATOM ID to get the full enriched test spec
4. **Cross-reference** `pass0-dependency-map.md` for shared-mechanism clusters and architectural dependencies

### Source Files (retained)

- `atomic-tests/extract-phase-index.py` — script that generated the indexes from summaries
- `atomic-tests/sessions/` — 12 full session files (authoritative test specs)
- `atomic-tests/summaries/` — 12 condensed summaries + audit responses
- `atomic-tests/supplemental-docs/` — supporting research documents
- `atomic-tests/pass0-dependency-map.md` — cross-session dedup, shared clusters, arch decisions
