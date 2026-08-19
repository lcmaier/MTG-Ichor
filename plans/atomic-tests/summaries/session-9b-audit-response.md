# Session 9B — Audit Response

**Date:** 2026-04-09 (Round 1), 2026-04-09 (Round 2)
**Source:** User audit notes on session-9b.md (713–732)

Legend:
- **AGREE** — Will fold the change back into session-9b.md
- **DISCUSS** — Needs alignment before folding
- **NOTE** — Acknowledged, tagged for future reference
- **REJECTED** — User declined the proposed change (Round 2)

---

## Rule-Specific Notes

### 714.2b — Read Ahead rules exception

> Worth noting that Read Ahead has a rules exception to this

**AGREE.** The CR text for 714.2b says "When one or more lore counters are put onto this Saga, if the number of lore counters on it was less than N and became at least N, [effect]." Read Ahead (702.155) modifies how the initial lore counters are placed (714.3a), which means the ETB counter placement can trigger multiple chapter abilities simultaneously. But more importantly, Read Ahead Sagas have a specific exception: per 702.155, when a Read Ahead Saga enters with N lore counters, only the chapter N ability triggers — NOT chapters 1 through N. This is a departure from the general 714.2b behavior where jumping from 0 to N triggers ALL crossed thresholds.

**REJECTED (Round 2).** User removed ATOM-714.2b-004 from session-9b.md. Read Ahead interaction with 714.2b belongs in Session 8's coverage of 702.155, not here. The note about the exception stands as context but no test is added to this session.

---

### 714.2e — Not PURE-DEF, effects reference "final chapter ability"

> This is not pure def, there are effects that care about the final chapter ability of a saga you control resolving, see this ability from Narci, Fable Singer

**REJECTED (Round 2).** User reverted the reclassification and removed ATOM-714.2e-001/002 from session-9b.md. 714.2e remains PURE-DEF. The Narci interaction is a card-specific trigger pattern that will be handled when that card is implemented, not a rules-level testable behavior.

---

### 714.4 — Blood Moon + Urza's Saga interaction

> Notable consequence here is if a saga loses all its chapter abilities (say the Saga Land "Urza's Saga" becomes affected by a Blood Moon), it will NOT be sacrificed

**AGREE.** This is a critical edge case. Re-reading 714.4 literally: "If the number of lore counters on a Saga permanent **with one or more chapter abilities** is greater than or equal to its final chapter number..." — the "with one or more chapter abilities" clause means a Saga with zero chapter abilities is simply exempt from this SBA. And per 714.2d, a Saga with no chapter abilities has final chapter number 0.

The Blood Moon + Urza's Saga interaction is the poster child: Blood Moon removes Urza's Saga's abilities (it becomes a basic Mountain with no chapter abilities), so the sacrifice SBA doesn't apply even though it has lore counters ≥ 0.

Also worth noting: this is a recent rules update (Final Fantasy saga creatures), so the old behavior was different.

**REJECTED (Round 2).** User removed ATOM-714.4-003 from session-9b.md. The Blood Moon + Urza's Saga interaction is a layers + SBA composition scenario that belongs in Phase 5 composition tests, not in atomic tests for 714.4. The analysis about the "with one or more chapter abilities" gating clause stands as context.

---

### 715.2b — Clone copy on battlefield, not in hand

> The line "If cast as an Adventure later (from hand, after bouncing), the copy would use the copied Adventure characteristics." is completely wrong. Clone effects only apply on the battlefield, if bounced it would have the characteristics of the card that represents it

**REJECTED (Round 2), then ACCEPTED (Round 3).** User confirmed: Clone in hand should only have its printed characteristics. Blanket reversion was because changes were applied too fast, not because the fix was wrong.

**Applied fix:** Expected Result now reads: "The copy has the same Adventure alternative characteristics as the original as part of its copiable values while on the battlefield. If the copy is bounced to hand, the copy effect ceases (new game object per 400.7) — the card in hand has only its printed characteristics."

---

### 716.2b — Level designation persistence: engine management

> This is a weird result. How should we manage this in the engine?

**DISCUSS.** The rule says level is a designation that any permanent can have, persists if it stops being a Class, and is not copiable. This is structurally identical to how we handle other designations (solved, monstrous, renowned, etc.):

**Proposed approach:**
- `BattlefieldEntity` gets a `designations: HashSet<Designation>` or `designations: DesignationSet` field
- `Designation::Level(u8)` stores the current level
- When a permanent leaves the battlefield, designations are stripped (standard zone-change new-object behavior)
- Copy effects don't copy designations (explicitly non-copiable per 716.2b, 719.3b, etc.)
- Queries like `get_level(game, id) -> u8` check the designation, defaulting to 1 per 716.2d

This is the same pattern we need for:
- **Solved** (719.3b) — Case designation
- **Monstrous** (702.98) — monstrosity
- **Renowned** (702.111) — renown
- **Saddled** (702.171) — Mount designation
- **Harnessed** (701.64) — harness designation

**Action:** Create a unified `Designation` enum on `BattlefieldEntity`. Level is one variant. All share: persists on battlefield, stripped on zone change, not copiable, not affected by type changes.

```rust
enum Designation {
    Level(u8),     // 716.2b — Class level
    Solved,        // 719.3b — Case solved
    Monstrous,     // 702.97 — Monstrosity
    Renowned,      // 702.112 — Renown
    Saddled,       // 702.171 — Saddle
    Harnessed,     // 701.64 — Harness
}
```

Player-level designations (City's Blessing, Monarch, Initiative) are separate — they live on `PlayerState`, not `BattlefieldEntity`, and have different lifetime rules (persist across zone changes, game-level scope). They'll use their own fields/enum.

**Tag: ARCHITECTURE-CONCERN unified-designation-system** — Consolidate all designation tracking into a unified system with clear lifetime rules.

---

### 716.2c — "Gain a Class level" is referenced by effects

> Potentially testable, currently one ability that cares about it

**AGREE.** The ability "Spend this mana only to cast an instant or sorcery spell or to **gain a Class level**" directly references 716.2c's definition. The engine needs to recognize "gain a Class level" as a valid mana spending category.

**Reclassification:** 716.2c → **BOUNDARY-DEF** (defines a term that mana spending restrictions reference).

**New test:**

```
ATOM-716.2c-001
- Rule: 716.2c — Mana spending restriction recognizes "gain a Class level" as a valid category
- Mechanism: Mana spending restriction validation against Class level activation
- Minimal Board: Player A controls a creature with "{T}: Add {U} or {R}. Spend this mana only to cast an instant or sorcery spell or to gain a Class level." Player A also controls a Class at level 1 with level 2 cost "{1}{U}."
- Action: Player A taps the creature for {U}, then uses it to activate the Class level 2 ability.
- Expected Result: Legal. "Gain a Class level" = "activate a class level bar ability" per 716.2c, and the mana spending restriction permits this use.
- Phase: Phase 5-Pre (T17 mana spending restrictions) + Phase 8
- Ticket: T17 + NEW — Class level activation as mana spending category (716.2c)
```

---

### 718 — Cross-reference with Prototype discussion in session-8 audit response

> Cross reference with the Prototype discussion in the session-8 audit response doc, does the new info here alter anything we talked through there?

**NOTE.** I've reviewed both the session-9b Prototype tests (718.1–718.5) and the session-8 audit response's Prototype architecture discussion (Proposal C: `CharacteristicOverrides`). The session-9b tests are **fully consistent** with the Proposal C design:

1. **718.2 (alternative characteristics on stack/battlefield)** — Maps directly to `CharacteristicOverrides` on `BattlefieldEntity`/`StackEntry` zone sidecars.
2. **718.2a (copiable values)** — Proposal C handles this: `base_overrides` lives on the zone sidecar, so copies inherit it on the battlefield.
3. **718.3b (color derivation from prototype mana cost)** — The `colors: Option<Vec<Color>>` field in `CharacteristicOverrides` covers this. Color is derived from the alternative mana cost and stored as an override.
4. **718.4 (normal characteristics in non-stack/battlefield zones)** — Proposal C handles this naturally: overrides only exist on zone sidecars, so they're stripped on zone change.
5. **718.5 (non-overridden characteristics unchanged)** — Proposal C's `Option` fields handle this: `types`, `abilities`, `name` are all `None` in the override struct (fall through to CardData).

**One new insight from 718.3d:** "If a permanent that was a prototyped spell is copied, the copy has the alternative power, toughness, and mana cost." This confirms that `base_overrides` must be part of the copiable values when on the battlefield — a copy of a prototyped permanent should inherit the overrides. Proposal C already handles this (overrides on `BattlefieldEntity` are copied when a Clone enters), but it's worth an explicit test note.

**No changes needed** to the Proposal C architecture. The session-9b tests validate it.

---

### 720 — Omen: mechanically similar to Adventure

> Mechanically very similar to Adventure, just shuffles away instead of exiling (so it's arguably even simpler)

**AGREE.** Omen is essentially Adventure with a different resolution destination:
- **Adventure:** resolves to exile, grants play-from-exile permission
- **Omen:** resolves by shuffling into library, no re-cast permission

Omen is indeed simpler — no exile permission tracking needed. The D4 CardLayout restructuring should handle Adventure and Omen with a shared `AlternativeCharacteristics` system. The resolution behaviors (Adventure → exile with play permission, Omen → shuffle into library) are different enough that hard-coding each is fine — there are only two cases and the behavior is well-defined. No enum abstraction needed at this stage. Other mechanics with distinct post-resolution behaviors (Flashback, Aftermath) are keyword-driven rather than card-layout-driven, so they'd use a different code path anyway.

**Tag: SHARED-BEHAVIOR adventure-omen** — Adventure and Omen share: two-part card frame, alternative characteristics on stack, cast-mode choice, zone-dependent characteristic masking, copiable values, spell copy behavior. Only differ in resolution destination. Hard-code each resolution path.

---

### 721.2b — Summoning sickness correction

> Incorrect on summoning sickness unless the permanent entered that turn

**REJECTED (Round 2), then CLARIFIED (Round 3).** User agreed a clarifying note makes sense. Rather than changing the Expected Result, the Minimal Board now specifies "It entered the battlefield this turn" and the Expected Result says "It has summoning sickness (entered this turn)." This disambiguates the test without changing its intent.

---

### 727.1a — Not PURE-DEF, effects directly reference it

> Not pure-def, a card directly references this with the ability "You gain life rather than lose life from radiation."

**AGREE.** The ability "You gain life rather than lose life from radiation" directly references 727.1a's definition of "life loss from radiation." The engine needs to tag life loss from the rad counter triggered ability distinctly so that replacement effects can identify it.

**Reclassification:** 727.1a → **BOUNDARY-DEF**

**New test:**

```
ATOM-727.1a-001
- Rule: 727.1a — "Life loss from radiation" is identifiable for replacement effects
- Mechanism: Life loss source tagging + replacement effect matching
- Minimal Board: Player A has 3 rad counters and controls a permanent with "You gain life rather than lose life from radiation." Top 3 cards: 2 nonland, 1 land.
- Action: Precombat main phase begins, rad counter ability triggers and resolves. Player A mills 3, finds 2 nonland.
- Expected Result: Instead of losing 2 life, Player A gains 2 life (replacement effect applies specifically to "life loss from radiation"). 2 rad counters are still removed (that part isn't replaced).
- Phase: Phase 6 (replacement) + Phase 8
- Ticket: NEW — Radiation life loss tagging for replacement effects (727.1a)
- Dependencies: Phase 6 (replacement effects), life loss source tracking
```

**Tag: LIFE-LOSS-SOURCE-TRACKING** — The engine needs to tag life loss events with their source/cause so replacement effects can match on specific causes (radiation, combat damage, etc.). This is the same infrastructure needed for "If you would lose life from a source an opponent controls" effects.

---

### 730.1a — Not PURE-DEF, triggers on day/night transition

> Not pure def, several abilities that care about "when day becomes night or night becomes day, [triggered effect]"

**AGREE.** Multiple cards trigger on "when day becomes night or night becomes day" — this is a game event that triggered abilities watch for. The engine needs to emit a delta/event when the day/night designation changes so triggers can match it.

**Reclassification:** 730.1a → **TESTABLE**

**New tests:**

```
ATOM-730.1a-001
- Rule: 730.1a — "When day becomes night" triggers fire on day→night transition
- Mechanism: Game designation change event → triggered ability matching
- Minimal Board: It is day. Player A controls a permanent with "When day becomes night, create a 1/1 Wolf token." Previous turn's active player cast 0 spells.
- Action: Untap step day/night check transitions from day to night.
- Expected Result: The trigger fires. Player A creates a 1/1 Wolf token.
- Phase: Phase 7 (triggered) + Phase 9 (D14)
- Ticket: D14 + NEW — Day/night transition trigger event (730.1a)

ATOM-730.1a-002
- Rule: 730.1a — "When night becomes day" triggers fire on night→day transition
- Mechanism: Game designation change event → triggered ability matching
- Minimal Board: It is night. Player A controls a permanent with "When night becomes day, draw a card." Previous turn's active player cast 2 spells.
- Action: Untap step day/night check transitions from night to day.
- Expected Result: The trigger fires. Player A draws a card.
- Phase: Phase 7 (triggered) + Phase 9 (D14)
- Ticket: D14
```

**Tag: DELTA-LOG-EVENT day-night-change** — The delta log must emit a `DayNightChanged { old: Day, new: Night }` (or vice versa) entry for trigger matching.

---

### 731 — Cross-reference with state-tracking-architecture.md

> Cross reference with the voluntary loop design discussion in state-tracking-architecture.md, does the new info here alter anything we talked through there? In particular, how does 731.3 and 731.5/6 fit in?

**DISCUSS.** I've reviewed `state-tracking-architecture.md` against the full text of 731.1b–731.6. Here's the analysis:

#### What the architecture doc already covers well

- **731.1b (loop detection):** Covered by Tier 1 forced-action counter + Tier 2 full-state hash. ✅
- **731.4 (mandatory loop = draw):** Covered explicitly in the "Three categories of loops" table (Category A). ✅
- **D26 voluntary shortcuts (731.2a–c):** Covered by the "execute first, then declare" pattern. ✅

#### What needs additions

**731.3 — Fragmented loops.** This is a gap in the current architecture. The doc's Tier 1 counter tracks consecutive forced actions *by the engine*. But 731.3 describes a fragmented loop where each player independently takes an optional action (e.g., "{0}: gains flying" vs "{0}: loses flying") and the game returns to the same state. Both actions involve meaningful player decisions (DP called with 2+ options), so Tier 1 never fires (counter keeps resetting).

**How Tier 2 catches this:** The full-state hash at quiescent points (empty stack + priority) WILL detect this — after one cycle of "gain flying → opponent removes flying," the game state matches a previously seen state. So Tier 2 is the correct tier for fragmented loops.

**But the *resolution* is different from mandatory loops:** Per 731.3, the active player (or first involved player in turn order) must make a different choice. This is NOT a draw — it's a forced choice change. The architecture doc's resolution for Tier 2 hash matches just says "forced draw," which is wrong for fragmented loops.

**Proposed addition to state-tracking-architecture.md:**

Add a new loop category:

| Category | Example | Detection | Resolution |
|---|---|---|---|
| **D — Fragmented, voluntary** | "{0}: gains flying" vs "{0}: loses flying" | Tier 2 hash match | Active player must choose differently (731.3) |

And add a resolution handler:
```rust
enum LoopResolution {
    ForcedDraw,                    // 731.4 — all mandatory
    ActivePlayerMustChooseDifferently(PlayerIndex), // 731.3 — fragmented
    NoForceToBreak,               // 731.5 — outside objects can't be forced
}
```

**731.5 — No forced break by uninvolved objects.** This constrains the engine's loop resolution: when a mandatory loop is detected (Category A), the engine cannot prompt a player to use objects not involved in the loop (like Seal of Cleansing) to break it. Only objects *in the loop* can be used for forced actions.

**Implication for the architecture:** The loop detector needs to track which objects are involved in the loop (the set of `ObjectId`s that appear in the delta log transcripts for the repeating segment). When prompting for a loop-breaking action, the DP should only be offered actions involving those objects.

**731.6 — Unless-clause loops.** "[A] unless [B]" where no player is forced to choose [B]. This is an important constraint on the DP routing: when the loop detector identifies an "unless" clause, it must NOT auto-select [B] as the loop-breaking action. If no player voluntarily chooses [B], [A] becomes mandatory and the loop continues (potentially becoming a Category A mandatory loop → draw).

**Proposed additions:**
1. Add Category D (fragmented voluntary) to the three-category table
2. Add a note to Tier 2 resolution: hash match → check if fragmented (voluntary actions from multiple players) vs mandatory → different resolution paths
3. Add a note to D26 validation: shortcut system must track involved objects (for 731.5) and respect unless-clause constraints (for 731.6)

**No fundamental architectural change needed** — Tier 2 already detects these. The gap is in the *resolution* logic, not detection. The resolution handler needs to distinguish mandatory-only (draw) from fragmented (active player changes choice) from unless-clause (can't force [B]).

**RESOLVED (Round 2).** User agrees: draft the additions to `state-tracking-architecture.md` now to spatially couple this info with the existing loop discussion.

---

### 732 — Potentially PURE-DEF for our engine

> This might be pure def, our engine shouldn't allow a player to take an illegal action

**DISCUSS.** You raise a good point. In our engine, the legality checks should prevent illegal actions from ever being attempted. The `choose_priority_action` → `validate_action` → `execute_action` pipeline should reject illegal moves before state mutation occurs. In theory, 732.1's rollback mechanism is never needed because:

1. Casting spells: 601.2 pipeline validates before committing
2. Activating abilities: validation before execution
3. Declaring attackers/blockers: `validate_attackers`/`validate_blockers` before committing
4. All DP choices are validated against legal options

**However,** there's one scenario the engine might hit: **partial execution of a complex action that fails partway through.** For example:
- Player begins casting a spell (601.2a–d succeed), but mana payment (601.2f) fails because the player doesn't have the right mana after all cost modifications are computed
- The casting rollback mechanism (T18, rule 601.2e referenced in state-tracking-architecture.md §Casting rollback) needs to undo the partial state changes

So 732.1 isn't fully PURE-DEF — it maps to the **T18 casting rollback** infrastructure which is already planned. But it's narrower than the general-purpose "rollback any illegal action" that the CR describes.

**RESOLVED (Round 2).** User agrees: reclassify to ALREADY-HANDLED-BY-DESIGN.

732.1 → **ALREADY-HANDLED-BY-DESIGN** for the general case (engine prevents illegal actions), with **T18** covering the casting-rollback edge case. 732.2 → **ALREADY-HANDLED-BY-DESIGN** (priority retention after failed validation is natural — the DP just gets re-queried).

Selvala, Explorer Returned is explicitly excluded. Nondeterministic illegal actions are not a concern. If a contrived scenario arises in the future, we handle it then.

**Action:** Reclassify 732.1 and 732.2 in the summary table. Remove the "Illegal action rollback + state snapshot" gap report entry (it's covered by T18). Remove ATOM-732.1-001/002/003 and ATOM-732.2-001.

---

## Summary of Actions (Post-Round 2)

### Accepted Reclassifications
| Rule | Old | New | Reason |
|------|-----|-----|--------|
| 716.2c | PURE-DEF | BOUNDARY-DEF | Mana spending restriction references "gain a Class level" |
| 727.1a | PURE-DEF | BOUNDARY-DEF | "Life loss from radiation" referenced by replacement effects |
| 730.1a | PURE-DEF | TESTABLE | Triggered abilities fire on day/night transitions |
| 732.1 | TESTABLE | ALREADY-HANDLED-BY-DESIGN | Engine prevents illegal actions; T18 covers casting rollback |
| 732.2 | TESTABLE | ALREADY-HANDLED-BY-DESIGN | Priority retention is natural after failed validation |

### Rejected Reclassifications
| Rule | Proposed | Kept | Reason |
|------|----------|------|--------|
| 714.2e | BOUNDARY-DEF | PURE-DEF | Card-specific trigger, not rules-level testable |

### New Tests to Add (5)
- ATOM-716.2c-001 (mana spending restriction for Class level)
- ATOM-727.1a-001 (radiation life loss tagging)
- ATOM-730.1a-001/002 (day/night transition triggers)

### Rejected Tests (not added to session-9b.md)
- ~~ATOM-714.2b-004~~ (Read Ahead — belongs in Session 8 / 702.155)
- ~~ATOM-714.2e-001/002~~ (Narci — card-specific, not rules-level)
- ~~ATOM-714.4-003~~ (Blood Moon — layers composition, not atomic 714.4)

### Test Fixes — All Rejected
- ~~ATOM-715.2b-001~~ — original text stands
- ~~ATOM-721.2b-001~~ — original text stands

### Tests to Remove from session-9b.md
- ATOM-732.1-001/002/003 and ATOM-732.2-001 (reclassified to ALREADY-HANDLED-BY-DESIGN)

### Architecture Notes (all accepted)
- **Unified Designation system** on `BattlefieldEntity` — Level, Solved, Monstrous, Renowned, Saddled, Harnessed. Player-level designations (City's Blessing, Monarch, Initiative) are separate on `PlayerState`.
- **Prototype cross-ref confirmed** — session-9b tests fully consistent with Proposal C (`CharacteristicOverrides`)
- **Omen shared behavior** with Adventure — D4 handles both, hard-code resolution paths (no enum)

### state-tracking-architecture.md Updates (to draft now)
1. Add Category D (fragmented voluntary loops) to the loop category table
2. Add resolution logic distinction: mandatory (draw) vs fragmented (active player changes choice) vs unless-clause (731.6)
3. Add 731.5 constraint: loop-breaking actions limited to involved objects
4. No fundamental architectural change — Tier 2 detection already covers these; gap is in resolution logic
