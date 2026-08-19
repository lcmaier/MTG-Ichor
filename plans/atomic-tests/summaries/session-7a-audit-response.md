# Session 7A — Audit Response

## General Notes

### G1: Function signature style (`game` as parameter vs method)

**Agreed — signatures should reflect the architecture.** The existing codebase uses two patterns:
- **Mutation**: methods on `Game` or free functions taking `&mut GameState` (e.g., `move_object`, `execute_action`, `play_land`). These are in `engine/`.
- **Read-only queries**: free functions taking `&GameState` (e.g., `is_creature`, `has_keyword`, `compute_devotion`). These are in `oracle/`.

The pseudocode signatures like `attach(game, attachment_id, target_id)` should be understood as:
- If mutating: a method on `GameState` or `Game`, e.g., `game.attach(attachment_id, target_id)` or `attach(&mut game.state, attachment_id, target_id)`
- If querying: an oracle free function, e.g., `oracle::compute_devotion(&game.state, player_id, color)`

**Action:** Added a note to the session header clarifying this convention. Individual test specs left as-is since they're conceptual, not literal API signatures.

### G2: Multiplayer deferral reassessment

**Agreed — "simultaneous actions by multiple players" ≠ "multiplayer-only".** An effect like "Two target creatures controlled by different players each connive" is perfectly legal in 2-player and invokes APNAP ordering. The APNAP rules themselves are already implemented (turn structure uses active/non-active player ordering).

**Rules reclassified from DEFERRED(multiplayer) to DEFERRED(Phase 8):**
- 701.22c (simultaneous scry) — now Phase 8 with note about 2-player applicability
- 701.23i (simultaneous search) — now Phase 8
- 701.44d (simultaneous explore) — stays DEFERRED Phase 9 (genuinely can't have two different players' creatures explore simultaneously in 2-player from a single effect — the controller explores, not the creature's controller)
- 701.50d (simultaneous connive) — now Phase 8

**Rules that stay Phase 9:** 701.38 (Vote — inherently 3+ player mechanic, no 2-player voting card exists), 701.55d (villainous choice APNAP — trivial extension, fine to keep deferred).

## Rule-Specific Responses

### 700.2d — Modal execution ordering

**Agreed.** The test should verify that repeated modes execute in declaration order, not interleaved with other modes. Updated ATOM-700.2d-002 to explicitly test sequential same-mode execution ordering with different targets.

### 700.5a — Layers implication + example test

**Agreed on both counts.**
1. Added explicit note that devotion computation is a **layer system hook** — it must read characteristics after L1–L3 (copy/control/text) but before L4–L7 (type/color/ability/P-T). This means the layer system needs a `compute_at_layer(obj, layer_3)` query.
2. Added ATOM-700.5a-002 testing the Purphoros example directly (devotion exactly at threshold determines creature-hood, which then feeds back into anthems).

### 700.10 — "Activated this turn" tracking via delta log

**Yes, the delta log is the right mechanism.** "Activated this turn" (rule 700.10) tracks whether a permanent's activated ability was activated this turn. The delta log already records ability activations as `GameDelta` entries. A per-turn query `was_activated_this_turn(game, permanent_id) -> bool` would scan the current turn's deltas for `ActivateAbility { source: permanent_id }` matches.

Same pattern applies to 700.11 (Descended) — scan current turn's deltas for `ZoneChange { destination: Graveyard, was_permanent: true }`.

Added architectural note to both 700.10 and 700.11.

### 701.9c — Cross-reference to 603-2f-complexity.md

**Noted.** Added cross-reference annotation.

### 701.10f — Doubled mana carries no restrictions

**Good catch.** The CR explicitly states (in earlier chapters) that doubled mana is "new" mana with no restrictions from the original source. Added note to ATOM-701.10f-001 expected result clarifying this.

### 701.12b — Delta log event ordering for control exchange

**Good question.** For simultaneous control exchange, the delta log should emit both control changes as a single batch (same epoch/timestamp). The order within the batch shouldn't matter for any current trigger pattern, but for determinism, we emit in APNAP order (active player's permanent first). Added architectural note.

### 701.12c-002 — Explicit "can't gain life" test

**Agreed.** Rewrote the test to be more explicit: P0 at 5, P1 at 20, P0 has "can't gain life." Exchange attempts: P0 would go 5→20 (gain 15, blocked), P1 would go 20→5 (loss 15, allowed). Since P0's half fails → entire exchange fails per 701.12a. Clear, testable.

### 701.12d-f — Mass zone exchange test

**Agreed.** Added ATOM-701.12d-001 for "exchange your hand and graveyard" — a real effect (Magus of the Jar type). Tests that all cards in hand go to graveyard and all cards in graveyard go to hand simultaneously.

### 701.14d — Switch to combat damage trigger, drop lifelink

**Agreed.** The point of this test is "fight damage is NOT combat damage." The clearest way to test this is: creature has "whenever this deals combat damage" trigger, creature fights, trigger should NOT fire. Lifelink muddied the test. Rewritten.

### 701.18b — "Play" permission union (lands + spells)

**Yes, this needs architectural consideration.** Effects like "Exile top 3, you may play them this turn" grant a `PlayPermission` that covers both land plays and spell casts from exile. The engine needs:
```rust
enum PlayPermission {
    CastSpell { from_zone: Zone, ... },
    PlayLand { from_zone: Zone, ... },
    PlayCard { from_zone: Zone, ... },  // union: either cast or play land
}
```
Added architectural note to 701.18b upgrading it from PURE-DEF to DEFERRED with design sketch.

### 701.19b — Static regeneration example fix

**Agreed.** Replaced the example with Mossbridge Troll ("If this creature would be destroyed, regenerate it.") which is a proper static replacement effect, not an activated ability creating a shield.

### 701.20 — Momentary vs persistent reveal

**Good architectural point.** Two distinct patterns:
1. **Momentary reveal**: "Reveal the top card" — revealed during effect resolution, then stops being revealed.
2. **Persistent reveal**: "Play with the top card of your library revealed" — static ability, continuous reveal while condition holds.

These are different mechanisms: momentary reveal is an event (fire-and-forget), persistent reveal is a continuous effect (layer system, checked every time visibility is queried). Added architectural note distinguishing the two and noting that persistent reveal is Phase 5-Layers territory.

### 701.22c — Not necessarily multiplayer-specific

**Agreed.** Reclassified to Phase 8. See G2 above. "Each player scries 2" is a perfectly valid 2-player effect.

### 701.23i — Same as 701.22c

**Agreed.** Reclassified to Phase 8.

### 701.24b — Effect text ordering fix

**Agreed — the test description had the order wrong.** "Search your library for a card, put it on top, then shuffle your library" would shuffle the card away. The correct card text is "Search your library for a card. Shuffle your library, then put that card on top." Fixed.

### 701.24c — Why PURE-DEF?

701.24c says: "If an effect would cause a player to shuffle one or more specific objects into a library, that library is shuffled even if none of those objects are in the zone they're expected to be in."

This is PURE-DEF because the engine already handles this naturally: `shuffle()` is called on the library regardless of whether the objects were successfully moved. The "edge case" is: a card says "shuffle Target Card into its owner's library" but the target already left the graveyard. The engine: (1) attempts zone change — fails/no-ops because object isn't there, (2) shuffles library anyway. Step (2) always happens because shuffle is an independent instruction in the effect sequence, not conditional on the zone change succeeding.

No *additional* code is needed — the existing effect resolution pipeline already handles this because shuffle is a separate instruction. That's why it's PURE-DEF rather than TESTABLE. Added explanatory note.

### 701.29 — Fateseal should use scry infrastructure

**Agreed.** Fateseal is literally "scry N but on an opponent's library." If scry is implemented, fateseal is a one-line wrapper: `scry(game, opponent_id, n, decisions)` instead of `scry(game, self_id, n, decisions)`. Reclassified from DEFERRED to ATOM with test. Added ATOM-701.29a-001.

### 701.34a — Counter annihilation in test

**Good catch.** +1/+1 and -1/-1 counters annihilate each other per SBA 704.5q. The test has both on the same creature, which means after SBA check: 2 +1/+1 and 1 -1/-1 → 1 +1/+1 and 0 -1/-1 → proliferate would only add +1/+1. Moved -1/-1 counters to a separate permanent to avoid this issue.

### 701.34b — Reclassify header to OUT-OF-SCOPE

**Agreed.** Header was inconsistent with the description. Fixed to `OUT-OF-SCOPE`.

### 701.40 — Layer 1b specification

**Agreed.** Added explicit note: face-down characteristics override is layer 1b ("face-down status"), which occurs after layer 1a (copy effects). This ordering is critical because it means a face-down morph creature retains its copy-layer-applied morph ability (from the front face) which lets it be turned face up.

### 701.40f — Manifest ordering

**Agreed — premature optimization.** The manifest function should: (1) check if zone change is legal, (2) if yes: set face-down characteristics + move to battlefield, (3) if no: do nothing. No need to "prepare" face-down characteristics before checking. Simplified the description.

### 701.41 — Nonpermanent Support test

**Agreed.** Added ATOM-701.41a-002 testing Support on an instant/sorcery spell, where the "other" restriction is absent — the spell can target any creatures including those the caster controls.

### 701.43 — Interaction with untap restriction effects

**Good question.** Effects like "Creatures with flying don't untap during their controllers' untap steps" and "Players can't untap more than one land during their untap steps" are **continuous effects that modify the untap step**, not exert-related. They would be implemented as:
1. **Can't-untap restrictions**: static abilities creating restriction effects (layer 6 or restriction layer). The untap step queries: "for each permanent, is untapping legal?" checking all active restrictions.
2. **Untap-count limits**: "untap no more than N" effects require the untap step to be a player-choice step (DP chooses which N to untap) rather than automatic.

Exert's `skip_next_untap` is a separate mechanism — it's a per-permanent flag, not a restriction effect. The untap step checks: (1) is this permanent restricted from untapping by a continuous effect? (2) does it have skip_next_untap > 0? Either → don't untap.

Added architectural note to 701.43 about untap step needing to support both mechanisms.

### 701.44c — Counter placement on destroyed permanent

**The test already covers this implicitly** — the LKI test says "No counter placed (C doesn't exist)." But added explicit assertion: "Verify no +1/+1 counter is placed on any object (counter placement targets the permanent which no longer exists, so it's impossible)."

### 701.47a-003 — Army choice only with token doubler

**Agreed — the test is wrong.** You can only have a choice of which Army to place counters on if you control multiple Army creatures, which normally can't happen from amass alone (it creates one only if you have none). You'd need a token doubler (Anointed Procession) or another source of Army tokens. Removed 003 as a standalone test and added a note that multi-Army choice is a composition test requiring token doublers.

### 701.54 — Designation registry for players vs permanents

**The designation registry on `BattlefieldEntity` handles permanent designations (monstrous, suspected, harnessed, renowned, ring-bearer). The Ring emblem and temptation counter are per-player state, not designations on permanents.** So:
- `Designation::RingBearer` goes in the permanent's `designations: HashSet<Designation>`
- `ring_temptation_count: u32` goes on `PlayerState`
- The Ring emblem is a `GameObject` in the command zone

No union struct needed. Player-level booleans/counters are cheap and the right abstraction. The Ring-bearer designation on permanents uses the same `HashSet<Designation>` as monstrous/suspected/etc.

### 701.57 — Cascade first, then Discover

**Agreed.** Cascade (702.84) is the older, more widely-played mechanic and Discover is essentially "Cascade with a cap." Implementation order should be: (1) Cascade in Session 7B/8, (2) Discover as a thin wrapper. Added note to 701.57 that implementation should follow Cascade, and the tests here define the *shared* infrastructure that both need.

### 701.65 — Airbend vs Cascade/Discover distinction

**Agreed.** Removed the cross-reference to Discover/Cascade from the Airbend section. While they share a `CastPermission` concept, the mechanics are distinct enough (persistent permission vs one-shot, fixed {2} vs free, batch exile vs sequential) that linking them is misleading. Each should be designed on its own merits.
