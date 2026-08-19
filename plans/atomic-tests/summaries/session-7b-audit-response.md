# Session 7B — Audit Response

> **Date:** 2026-04-08
> **Auditor notes from:** User (project lead)
> **Responder:** LLM (Senior Rust Engineer / CR Expert)

This document addresses each audit note, proposes concrete actions, and flags items that need further discussion before session-7b.md edits are applied.

---

## General Notes

### G1: Scrutinize Already-Implemented Keywords Against the Four Pillars

**Auditor:** Nothing currently implemented is sacred. If we find something that should be torn down and rewritten, we do it now.

**Response:** Agreed. After reviewing the ten implemented keywords against their full CR rules, here's the pillar assessment:

| Keyword | Correctness | Speed | Maintainability | Extensibility | Action |
|---|---|---|---|---|---|
| Deathtouch | ✅ Core correct. `damaged_by_deathtouch` flag + SBA is the right model. | ✅ | ✅ | ⚠️ LKI + any-zone gaps are well-scoped for Phase 5/6. | No rewrite needed. |
| Defender | ✅ | ✅ | ✅ | ✅ | No action. |
| Double Strike | ✅ `should_deal_damage_this_step` is clean. | ✅ | ✅ | ⚠️ Mid-combat grant/loss needs layers (Phase 5). | No rewrite; layers will compose. |
| First Strike | Same as Double Strike. | | | | No action. |
| Flying | ✅ Per-pair check in validation.rs. | ✅ | ⚠️ See G2 (unified evasion). | ⚠️ See G2. | Potential refactor — see G2. |
| Haste | ✅ | ✅ | ✅ | ✅ | No action. |
| Lifelink | ✅ Per-source application in `apply_lifelink`. | ✅ | ✅ | ⚠️ LKI + any-zone same as deathtouch. | No rewrite. |
| Reach | ✅ | ✅ | ⚠️ See G2. | ⚠️ See G2. | Part of unified evasion discussion. |
| Trample | ✅ Core assignment correct. | ✅ | ⚠️ DP interface concern — see R-702.19c. | ⚠️ See R-702.19c. | Potential DP refactor. |
| Vigilance | ✅ | ✅ | ✅ | ✅ | No action. |

**Bottom line:** No keyword needs a full rewrite. Two architectural concerns warrant design discussion: unified evasion (G2) and trample DP unification (R-702.19c). Both are forward-looking extensibility improvements, not correctness bugs.

### G2: Unified Evasion Ability

**Auditor:** Flying, Shadow, Fear, Intimidate, Horsemanship are all mechanically similar. Should we have a unified `EvasionAbility` that varies only in the blocking filter?

**Response:** This is an excellent extensibility insight. Current state:

The flying check in `validation.rs` is a hardcoded `if has_keyword(Flying) && !(has_keyword(Flying) || has_keyword(Reach))` block. Every new evasion keyword would need another `if` branch with similar structure but different filter logic.

**Proposed model:**

```rust
/// An evasion restriction: "this creature can't be blocked except by
/// creatures matching `can_block_filter`."
struct EvasionRestriction {
    /// The keyword that grants this evasion (for display/query purposes)
    keyword: KeywordAbility,
    /// Filter that a potential blocker must pass to legally block
    can_block_filter: BlockerFilter,
}

enum BlockerFilter {
    /// Flying: blocker must have Flying or Reach
    HasAnyKeyword(Vec<KeywordAbility>),
    /// Shadow: blocker must have Shadow (bidirectional — attacker without
    /// shadow also can't be blocked BY shadow creatures)
    Bidirectional(KeywordAbility),
    /// Fear: blocker must be artifact creature OR share a color
    ArtifactOrSharesColor,
    /// Intimidate: blocker must be artifact creature OR share a color
    /// (same filter as Fear — could deduplicate)
    ArtifactOrSharesColorWithAttacker,
    /// Horsemanship: blocker must have Horsemanship
    HasKeyword(KeywordAbility),
    /// Landwalk: can't be blocked at all if defending player controls
    /// a land with the specified type (different — conditional unblockable)
    ConditionalUnblockable(LandwalkCondition),
}
```

The validate_blockers loop would become:
```rust
for evasion in get_evasion_restrictions(game, attacker_id) {
    if !evasion.can_block_filter.allows(game, blocker_id, attacker_id) {
        return Err(CombatError::EvasionBlock(blocker_id, attacker_id, evasion.keyword));
    }
}
```

**Benefits:**
- Adding a new evasion keyword = adding a `BlockerFilter` variant + registering it. Zero changes to validation loop.
- Flying/Horsemanship unification: `HasKeyword(Horsemanship)` vs `HasAnyKeyword(vec![Flying, Reach])`.
- Landwalk is slightly different (conditional on defender's lands, not blocker traits), but `ConditionalUnblockable` handles it.
- Shadow's bidirectionality (shadow can't be blocked by non-shadow AND non-shadow can't be blocked by shadow) is the one wrinkle — needs a second filter applied to the "does the attacker evade this blocker" direction. The `Bidirectional` variant handles this.

**Cost:** Medium refactor of `validate_blockers` + introducing the `EvasionRestriction` type. Should be done in Phase 8 when we implement the first non-Flying evasion keyword.

**Recommendation:** Add a META note to session-7b. Create a NEW ticket (NEW-3: Unified Evasion Framework). Don't refactor now — the current flying-only code is correct and we'd be refactoring without a second evasion to test against. But when Phase 8 lands Shadow/Fear/Intimidate/Horsemanship, the first one implemented should introduce this framework.

**Action:** Add META note + NEW-3 ticket.

### G3: "Quality" and "Text" Are Not Explicitly Defined in the CR

**Auditor:** How do we specify the set of qualities for hexproof-from, and what are the limits on affinity's `[text]`?

**Response:** This is a real ambiguity that we need to resolve at implementation time. Here's my analysis:

**For "quality" (hexproof, protection):**
The CR uses "quality" informally to mean "a characteristic or other property." In practice, qualities appearing on printed cards are:
- Colors: "protection from red," "hexproof from black"
- Card types/subtypes: "protection from Goblins," "protection from artifacts"
- Specific card names: "protection from Emrakul"
- "Everything": "protection from everything"
- Mana values: "protection from mana value 3 or less" (rare)
- Players: "protection from [player]" (Commander)

**Proposed `ProtectionQuality` enum:**
```rust
enum ProtectionQuality {
    Color(Color),
    CardType(CardType),
    Subtype(SubtypeId),
    CardName(String),
    Everything,
    Player(PlayerId),
    ManaValueAtMost(u32),
    // Extensible — new variants as needed
}
```

A `matches_quality(game: &GameState, source_id: ObjectId, quality: &ProtectionQuality) -> bool` function centralizes all quality matching. Hexproof-from and protection both use the same quality system.

**For "text" (affinity):**
Affinity's `[text]` in practice is always a permanent type or characteristic: "affinity for artifacts," "affinity for Plains," "affinity for Forests." The most exotic printed affinity is "affinity for Daleks" (Doctor Who set). The definition "costs {1} less for each [text] you control" means `[text]` is always a filter on controlled permanents.

**Proposed:** Affinity takes a `PermanentFilter` (same type used elsewhere in the effect system) rather than trying to parse arbitrary text. For the cards that actually exist, this covers everything.

**For truly abstract affinity** (e.g., hypothetical "Affinity for creatures with power 4 or greater"): `PermanentFilter` already supports compound predicates. We don't need to solve the general case — we need to support the filters that appear on actual printed cards.

**Action:** Add META note to session-7b documenting quality representation design. No test changes needed — this is an implementation concern.

### G4: "Copy a spell" vs "Copy a card, you may cast the copy"

**Auditor:** Subtle difference between Storm-style "copy this spell" (directly on stack) and "copy a card, you may cast the copy" (card copy in a zone, cast permission). Rule 707.10a is relevant.

**Response:** This is a critical distinction for correctness. Let me break it down:

**Pattern A — "Copy this spell" (Storm, Replicate, Conspire):**
- Creates a spell copy directly on the stack (rule 707.10)
- Not "cast" — doesn't trigger "when you cast" abilities
- Targets can be changed for the copies (rule 707.10d)
- The copy is not a card — it ceases to exist when it leaves the stack

**Pattern B — "Copy a card and you may cast the copy" (some exile effects, Panharmonicon-adjacent):**
- Rule 707.10a: creates a copy of a card (not a spell) in a specified zone (often exile)
- The controller gets permission to cast the copy (usually "without paying its mana cost" or "until end of turn")
- Casting the copy IS casting a spell — triggers "when you cast" abilities
- If not cast, the copy ceases to exist per SBA (rule 707.10a last sentence)

**Pattern C — "Copy target spell" (e.g., Reverberate, Fork):**
- Similar to Pattern A but targets an existing spell on the stack
- Creates a copy on the stack; "you may choose new targets"

**Engine implications:**
- We need a `copy_spell_on_stack(spell_id) -> ObjectId` that clones a StackEntry, assigns a new ObjectId, and places it on the stack
- We need a `create_card_copy(card_id, destination_zone) -> ObjectId` that creates a temporary GameObject that ceases to exist when SBAs clean up unclaimed copies
- Pattern A/C are simpler (stack manipulation); Pattern B requires the "castable copy" permission system

**Recommendation:** Document this as a META note. The actual implementation splits naturally into Phase 8 work. Storm/Replicate/Conspire all use Pattern A. Pattern B is rarer in the 702.1-80 range.

**Action:** Add META note.

---

## Rule-Specific Notes

### R-702.6a: Negative Case for Sorcery Speed

**Auditor:** Should we test that you can't equip at instant speed?

**Response:** Yes, absolutely. The current ATOM-702.6a-001 mentions it in the expected result ("Activation at instant speed is illegal") but it's a clause in the same test, not a standalone negative test. Per our atomicity principle, this should be its own ATOM.

**Action:** Add ATOM-702.6a-003 — "Equip activation is illegal during an opponent's turn / during combat / with a non-empty stack (sorcery-speed enforcement)."

### R-702.6d: Multiple Equip Abilities

**Auditor:** 702.6d was classified PURE-DEF, but Equipment with both generic equip and "Equip Knight" should test that the generic equip works on non-Knights.

**Response:** Good catch. 702.6d isn't purely definitional — it has a testable mechanical consequence: if Equipment has "Equip {4}" and "Equip Knight {1}", the {4} ability can target non-Knights. This is worth an ATOM because it validates that the engine doesn't accidentally merge equip restrictions.

**Action:** Reclassify 702.6d from PURE-DEF to TESTABLE. Add ATOM-702.6d-001 — "Equipment with generic equip and type-restricted equip: generic equip targets any creature, type-restricted equip only targets matching creatures."

### R-702.11: Hexproof Granted Between Cast and Resolution → Fizzle

**Auditor:** Should we test that granting hexproof after a spell is cast but before resolution causes the spell to fizzle?

**Response:** This is a targeting legality check at resolution time (rule 608.2b), not a hexproof-specific test. When targets are checked on resolution:
1. If the target has gained hexproof from the opponent who cast the spell → target is illegal
2. Spell with all targets illegal → fizzles (rule 608.2b)

This is partly covered by the general targeting/fizzle tests in session 4 (rule 400.7 epoch/fizzle tests). However, a hexproof-specific composition test is worth adding because it validates that hexproof's targeting restriction is checked at resolution, not just at cast time.

**Action:** Add COMP-702-007 — "Hexproof granted between cast and resolution causes fizzle." Tags: hexproof, targeting, fizzle, composition.

### R-702.16a: Multiclause — Protection from Cardname / Snow

**Auditor:** Should test protection from non-color qualities like card names or "snow."

**Response:** Agreed. The current ATOM-702.16a-001 only tests that a protection ability with quality = red exists. We should add tests for the breadth of the quality system.

**Action:** Add ATOM-702.16a-002 — "Protection from [cardname]: blocks targeting/damage from a permanent with that name." Add ATOM-702.16a-003 — "Protection from snow: blocks targeting/damage from snow permanents."

### R-702.16e: Clean Up Reasoning Text, Better Test

**Auditor:** Remove the Bolt part, use combat damage prevention from a protected creature blocking a [quality] attacker.

**Response:** Agreed, the current test body is muddled. A clean combat-focused test is better for 702.16e because it isolates protection's damage prevention in its natural habitat (combat).

**Action:** Rewrite ATOM-702.16e-001 with a clean combat board: creature with "protection from red" blocking a red attacker. Expected: combat damage from the red creature to the protected blocker is prevented. No Bolt, no "P0 the player" confusion.

### R-702.16j: Protection from Everything as Base Case

**Auditor:** Should "protection from everything" be the implementation base case, with specific protections filtering?

**Response:** Interesting design idea. Let me think through it:

**Option A (current implicit model):** Protection stores a quality. `matches_quality` checks if a source matches. "Protection from everything" is `ProtectionQuality::Everything`, which always returns true.

**Option B (auditor's suggestion):** Protection base case is "everything." Specific protections are filters that narrow the base.

Option A is simpler and more correct. "Protection from everything" IS the degenerate case where the quality filter matches all objects. Making it the base case would invert the mental model — you'd think of protection as "block everything, except..." which is backwards. The CR defines protection as "protection FROM [quality]" — additive matching, not subtractive filtering.

**Recommendation:** Keep Option A. `ProtectionQuality::Everything` as one variant in the enum, where `matches_quality` returns `true` unconditionally. Simple, correct, matches the CR's framing.

**Action:** Add design note to session-7b META, but no test changes.

### R-702.16k: Multiplayer Stolen Permanent Edge Case

**Auditor:** Third player controls a permanent owned by the player you have protection from → you do NOT have protection from that permanent (because the player you have protection from doesn't *control* it).

**Response:** Excellent edge case. The current test only covers the 2-player case. We need a 3-player ATOM.

**Action:** Add ATOM-702.16k-002 — "Protection from [player]: a permanent owned by that player but controlled by a third player is NOT subject to the protection (control, not ownership, matters). Phase 9 (multiplayer)."

### R-702.18: Both Players Can't Target Shroud

**Auditor:** Test both the controller AND the opponent being unable to target.

**Response:** The current ATOM-702.18a-001 tests that the controller can't target their own shroud creature. ATOM-702.18a-002 was about player shroud. We're missing the explicit opponent-can't-target case (which hexproof also blocks, but shroud blocks the controller too — that's the distinguishing test).

Looking more carefully: ATOM-702.18a-001 tests the controller case (which is the unique differentiator from hexproof). For completeness, we should add the opponent case as well.

**Action:** Add ATOM-702.18a-003 — "Opponent can't target a permanent with shroud either." Simple and completes the pair.

### R-702.19c: Trample Over Planeswalkers — DP Unification

**Auditor:** Can we avoid a separate DP primitive for trample-over-PWs? Unified "assign damage" interface for trample / trample-over-PW / future trample-over-battles?

**Response:** Currently, `choose_trample_damage_assignment` in the DP takes `(alive_blockers, defending_target, damage, has_deathtouch)`. The signature is:

```rust
fn choose_trample_damage_assignment(
    &self, game: &GameState, player: PlayerId,
    attacker: ObjectId, alive_blockers: &[ObjectId],
    defending_target: DamageTarget, damage: u64,
    has_deathtouch: bool,
) -> (Vec<(ObjectId, u64)>, u64);
```

For trample-over-PW, the difference is:
1. Damage flows: blocker(s) → planeswalker → player (three levels, not two)
2. The PW has a "lethal" threshold = loyalty, not toughness

**Proposed unified signature:**

```rust
struct TrampleContext {
    attacker: ObjectId,
    /// Blockers that must receive at least lethal damage
    blockers: Vec<ObjectId>,
    /// Intermediate targets (planeswalker/battle) that can absorb damage
    /// after blockers, before the final target. Empty for normal trample.
    intermediates: Vec<TrampleIntermediate>,
    /// Final overflow target (defending player)
    final_target: DamageTarget,
    total_damage: u64,
    has_deathtouch: bool,
}

struct TrampleIntermediate {
    target: DamageTarget,
    /// Minimum damage to "satisfy" this target (loyalty for PW, defense for battle)
    threshold: u64,
}
```

Normal trample: `intermediates = []`, overflow goes to `final_target` (player).
Trample over PW: `intermediates = [TrampleIntermediate { target: PW, threshold: loyalty }]`, overflow goes to player.
Trample over battle (hypothetical): same pattern with defense as threshold.

This keeps ONE DP method, ONE validation path, and the engine decides what intermediates exist based on the attacker's keywords and attack target.

**Action:** Add META note proposing `TrampleContext` unification. Add to NEW tickets (NEW-4: Unified Trample DP). No immediate code changes — current trample is correct for the player-only case.

### R-702.22: Banding — Reconsider Deferral

**Auditor:** Don't defer out of intimidation. Banding is vintage-legal, we want broad support. Unless there's a real architectural blocker, plan for inclusion.

**Response:** Fair challenge. Let me re-evaluate:

**Architectural complexity of banding:**
1. **Band formation (702.22c-d):** Creatures with banding + up to one without can form a band. All must attack the same target. This requires a new `Band` concept in the attack declaration — a group of attackers treated as one unit for blocking purposes.
2. **Block sharing (702.22h-i):** If any creature in a band is blocked, ALL creatures in the band are blocked by that blocker. This changes how `validate_blockers` works — blocking one member blocks the band.
3. **Damage redirection (702.22j-k):** The *defending* player chooses how the attacking banded creatures' damage is divided among blockers (and vice versa). This is a DP change — damage assignment authority shifts.

**Architectural blockers:**
- (1) requires a `Vec<Band>` or similar in the combat state, which is a structural addition to `AttackingInfo` but not a rewrite.
- (2) requires changing `validate_blockers` from per-pair to band-aware, which touches the core of blocker validation. With the unified evasion proposal (G2), this is manageable — the evasion check runs per-attacker, and bands would wrap that.
- (3) is the real change: `choose_attacker_damage_assignment` currently assumes the attacking player assigns damage. Banding reverses this for specific pairs. The DP needs a `choose_banding_damage_assignment` or the existing method needs a `damage_assigner: PlayerId` parameter.

**Verdict:** No single piece is a showstopper. The complexity is in the number of interacting changes, not in any one being architecturally impossible. It touches combat state, blocker validation, and damage assignment — three core combat systems — but each change is additive, not a rewrite.

**Recommendation:** Reclassify from "DEFERRED (niche, extremely low priority)" to "DEFERRED Phase 8 (vintage-legal, medium priority)." Remove the "extremely complex" language. Add a dependency note: "Recommended after unified evasion framework (NEW-3) since band blocking interacts with evasion." Generate skeleton ATOM tests for the core rules (702.22c, 702.22h, 702.22j) so they're ready when implementation begins.

**Action:** Update 702.22 note in session-7b. Add 3 skeleton ATOMs (band formation, block sharing, damage redirection). Update classification table to remove "extremely low priority."

### R-702.24: Cumulative Upkeep + Solemnity Integration Test

**Auditor:** CU + "counters can't be placed" = dodge the cost entirely.

**Response:** Great integration test. If age counters can't be placed (e.g., Solemnity: "Counters can't be placed on permanents"), then the CU trigger tries to add an age counter, fails, and then you pay cost × (number of age counters) = cost × 0 = nothing. The permanent persists indefinitely.

**Action:** Add COMP-702-008 — "Cumulative Upkeep + Solemnity: age counter can't be placed → CU cost is always 0 → permanent never sacrificed." Phase 8. Tags: cumulative-upkeep, counters, composition.

### R-702.26a: Test Phasing Out, Not Just the Cycle

**Auditor:** Should also test the phase-out itself here.

**Response:** The current ATOM-702.26a-001 describes the full cycle (phase out → phase in) but doesn't separately test the phase-out mechanics. Splitting:

**Action:** Update ATOM-702.26a-001 to focus on the phase-out event: creature phases out, verify it's treated as not existing (can't be targeted, doesn't count for "creatures you control" effects, removed from combat). The phase-in is covered by 702.26c/d tests. This makes the test more atomic.

### R-702.26f, 702.26m: Are We Sure These Can Be Deferred?

**Auditor:** Tricky cases, want to think them through.

**Response:**

**702.26f** — "For as long as" durations end when the permanent phases out. This means if a continuous effect says "for as long as [this permanent] is on the battlefield" and the permanent phases out, the effect ends. When it phases back in, the effect does NOT resume (it's a new "as long as" check).

This is Phase 5 (continuous effects) dependent. Without the layer system, we can't even represent "for as long as" durations. So yes, this genuinely can't be tested until Phase 5 lands. Correctly deferred.

**702.26m** — "If a player's untap step is skipped, the phasing event for that player doesn't happen." This is a simple guard: `if untap_step_skipped { skip_phasing_event(); }`. It requires phasing to be implemented first (Phase 8), but the test itself is trivial.

This can be deferred because phasing itself is Phase 8, and this is a minor modifier on the phasing event.

**Verdict:** Both are correctly deferred. 702.26f is blocked by Phase 5 layers. 702.26m is blocked by Phase 8 phasing.

**Action:** Add clarifying notes to both one-liners explaining WHY they're deferred (not just "one-liner").

### R-702.29: Cycling Speed

**Auditor:** Should we test that cycling is at instant speed? Or is that implicit in activated ability logic?

**Response:** The activated ability framework should enforce timing rules by default. Cycling doesn't say "activate only as a sorcery," so it follows default activated ability timing (any time you have priority = instant speed). If the framework correctly defaults to instant-speed timing for activated abilities, no separate cycling speed test is needed.

However, it's worth a brief note: "Cycling's instant-speed activation is validated by the base activated ability timing framework, not by a cycling-specific test."

**Action:** Add a note to the cycling section confirming this is covered by the base framework. No new ATOM.

### R-702.30: Control Change Re-Triggers Echo

**Auditor:** Should test control exchange effects re-triggering the echo cost.

**Response:** The echo condition is "if this permanent came under your control since the beginning of your last upkeep." If Player B steals Player A's echo creature, Player B now "gained control since their last upkeep" → echo triggers on B's next upkeep.

**Action:** Add ATOM-702.30a-002 — "Echo re-triggers after control change: stealing an echo creature causes echo to trigger on the new controller's next upkeep." Phase 8.

### R-702.35c: Madness Tracking After Trigger Resolves

**Auditor:** Reexamine if this can be deferred. Could get naturally handled by base engine systems.

**Response:** 702.35c states: "After the madness trigger resolves, if the card wasn't cast and was moved to a public zone rather than a hidden zone, effects that refer to characteristics of the discarded card can find that card."

This is about maintaining the "discarded card" reference tracking through the madness exile→graveyard path. If the card goes from hand → exile (madness replacement) → graveyard (not cast), effects that triggered on the original discard event need to be able to find it in the graveyard.

The base engine's zone-change tracking (epoch system) would handle this IF we ensure that "discarded card" tracking follows the card through zones rather than being bound to the original zone-change event. This is non-trivial — it requires the discard event to track the card's final resting place, not just the zone it was in when discarded.

**Verdict:** This is indeed complex and interacts with both the replacement effect system (Phase 6) and the triggered ability system (Phase 7). It can't be "naturally handled" without deliberate design. Correct to defer, but worth a note about the dependency chain.

**Action:** Expand the one-liner for 702.35c to note the replacement effect + trigger tracking dependency.

### R-702.37: Cross-Reference with Rule 708

**Auditor:** Note the dependency so the grouping is clear for the dependency pass.

**Response:** Agreed.

**Action:** Add a cross-reference note to 702.37 and all face-down keywords (702.75 Hideaway) pointing to rule 708 as a hard dependency. Tag with `face-down-infra`.

### R-702.43: Test ETB Counters

**Auditor:** Should also test that a Modular 3 creature enters with 3 counters.

**Response:** The current ATOM-702.43a-001 starts from a state where the creature already has 3 counters (testing the death trigger). The ETB counter placement is assumed but not explicitly tested.

**Action:** Add ATOM-702.43a-002 — "Modular N creature enters the battlefield with exactly N +1/+1 counters." Simple ETB verification.

### R-702.46: Soulshift MV Cap

**Auditor:** Should test we can't get back a 5 CMC Spirit with Soulshift 4.

**Response:** The current test only shows the positive case (MV ≤ N works). The negative case (MV > N fails) is important for correctness.

**Action:** Add ATOM-702.46a-002 — "Soulshift N cannot return a Spirit card with mana value greater than N. A Spirit with MV 5 is not a legal target for Soulshift 4."

### R-702.47: Splice — Confirm Safe to Defer

**Auditor:** Another one we should be absolutely sure can be safely deferred.

**Response:** Splice is genuinely unusual:
1. It modifies a spell's text during casting (rule 612 text-changing effects)
2. The spliced card stays in hand (not cast, not exiled)
3. Spliced text is lost when the spell leaves the stack
4. Targets for spliced text are chosen during casting

Splice requires:
- Text-changing effect infrastructure (not just P/T or abilities — actual rules text modification)
- Additional cost framework (T17)
- The ability to dynamically add targets to a spell after modes are chosen

The text-changing aspect is the real blocker. No other keyword in 702.1-80 modifies a spell's rules text. This is genuinely different from granting abilities (which we handle in Phase 5 layers) — it's splicing rules text onto a spell on the stack.

**Verdict:** Safe to defer. Splice is blocked by:
1. T17 (additional cost framework)
2. Text-changing effects (no infrastructure for this yet, distinct from ability-granting)
3. Dynamic target addition (targets added during casting based on spliced text)

All three are Phase 8 work. No risk of splice "falling through the cracks" — it's clearly scoped.

**Action:** Expand the 702.47 note to enumerate the three blockers explicitly.

### R-702.49: Ninjutsu Timing Subtleties

**Auditor:** Ninjutsu can be activated any time after blockers are declared — not just during the declare blockers step. This enables: (a) ninjutsu in a first-striker after first-strike damage for pseudo-double-strike, (b) ninjutsu in end-of-combat so the creature doesn't deal damage.

**Response:** Excellent timing observations. The CR says ninjutsu can be activated "whenever an attacking creature you control is unblocked" — which is true from declare blockers through end of combat. This is indeed any time you have priority in those steps.

Both scenarios are important edge cases:
- (a) Ninjutsu after first-strike damage: the ninjutsu creature enters attacking and deals damage in the regular damage step.
- (b) Ninjutsu in end-of-combat: the creature enters attacking but combat damage has already been dealt; it deals no combat damage this turn.

**Action:** Add ATOM-702.49a-002 — "Ninjutsu after first-strike damage: creature enters attacking and deals combat damage in the regular damage step." Add ATOM-702.49a-003 — "Ninjutsu in end-of-combat step: creature enters attacking but deals no combat damage (damage steps already passed)." Both DEFERRED Phase 8.

### R-702.51: Convoke Numbers Off

**Auditor:** Either need {2} in the mana pool or the spell needs to cost 1 less.

**Response:** Current test: spell costs {3}{G}{G}, P0 taps 2 green creatures ({G}{G}), 1 red creature ({1} generic), has {1} in pool. That's {G}{G} + {1} + {1} = {3}{G}{G}. Math checks out — 3 creatures tap for {G}{G} + {1} generic, plus {1} from pool covers the remaining {1} generic and the {1} is {3} total generic ({1} from red creature + {1} from pool + {1} more... wait.

Actually: cost is {3}{G}{G}. The {G}{G} is covered by 2 green creature taps. The {3} generic needs 3 generic mana. 1 red creature tap covers {1} generic. {1} from pool covers another {1} generic. That's only {2} generic, but we need {3}.

**The auditor is right.** The math is off by 1. Either the pool needs {2} or the spell should cost {2}{G}{G}.

**Action:** Fix ATOM-702.51a-001 — change "P0 has {1} in mana pool" to "P0 has {2} in mana pool" to make the math work ({2}{G}{G} creature taps + {2} pool - {1} for the red creature = wait, let me redo this cleanly.

Spell cost: {3}{G}{G} = 3 generic + GG.
Payment: tap 2 green creatures → covers {G}{G}. Tap 1 red creature → covers {1} generic. Pool has {2} → covers remaining {2} generic. Total generic covered: 1 + 2 = 3. ✅

**Fix:** Change pool from {1} to {2} in the test.

### R-702.52b: Test Even Though Dredge Restriction Is Obvious

**Auditor:** Should we test the "can't dredge if library has fewer than N cards" case?

**Response:** Yes. While the "as long as" clause in 702.52a makes this implicit, an explicit negative test is good practice for a mechanic that interacts with library size. It's a cheap, high-value boundary test.

**Action:** Reclassify 702.52b from PURE-DEF to TESTABLE (boundary). Add ATOM-702.52b-001 — "Dredge N with fewer than N cards in library: dredge option is not available; player must draw normally."

### R-702.53: Cross-Reference with Cycling

**Auditor:** Transmute and cycling share the "discard from hand + activated ability" pattern.

**Response:** Agreed. Both are hand-based activated abilities with discard as cost. The implementation will likely share the zone-specific activation framework.

**Action:** Add `cross-ref-cycling` tag to 702.53 tests. Add a note: "Implementation likely shares hand-zone activation framework with Cycling (702.29)."

### R-702.55: Haunting Is a Designation, Not a Characteristic

**Auditor:** The CR doesn't say this explicitly, but "haunting" / "is haunted by" is a designation.

**Response:** Correct. "Haunting" is a game-state relationship (like "paired" from Soulbond, or "renowned"), not a characteristic (rule 109.3 lists characteristics: name, mana cost, color, etc.). This matters because characteristics are checked by continuous effects, but designations are checked by specific abilities.

In the engine, this means haunt's association is tracked as a relationship (`haunted_by: Option<ObjectId>` on a BattlefieldEntity or an exile-zone association), not as a characteristic on the object.

**Action:** Add note to 702.55 section: "Haunting is a designation (game-state relationship), not a characteristic. Engine tracks as an association between exiled card and haunted permanent."

### R-702.62: Test Suspend Sub-Abilities Separately

**Auditor:** Since suspend is complex, test the three sub-abilities separately: (1) the in-hand static ability that exiles with counters (doesn't use the stack), (2) the upkeep counter-removal trigger, (3) the last-counter-removed cast trigger.

**Response:** Agreed. The current ATOM-702.62a-001 tests the whole cycle as one test. Breaking it into three is more atomic and better isolates failures.

**Action:** Split ATOM-702.62a-001 into three:
- ATOM-702.62a-001: "Suspend exile: pay cost from hand, exile with N time counters. This is a special action (not a spell, doesn't use the stack)."
- ATOM-702.62a-002: "Suspend upkeep trigger: each upkeep removes one time counter."
- ATOM-702.62a-003: "Suspend last-counter trigger: when last time counter is removed, cast the spell without paying its mana cost. Creature spells gain haste."

### R-702.63b: Vanishing Without a Number / Solemnity Interaction

**Auditor:** Important subtleties: (1) Vanishing with no time counters (e.g., Solemnity) doesn't trigger the sacrifice because the rule says "when the LAST time counter is removed" — no counter was ever there to be "last." (2) If a time counter is moved onto such a permanent, the vanishing loop resumes.

**Response:** Excellent nuance. This is a real correctness trap:

1. "Vanishing N" + Solemnity = enters with 0 counters. The "remove a time counter" upkeep trigger does nothing (no counter to remove). The "when the last time counter is removed" trigger never fires (no counter was removed). Permanent lives forever.
2. If someone later puts a time counter on it (e.g., Clockspinning), the upkeep trigger will start removing counters, and when the last is removed, the sacrifice trigger fires.

**Action:** Expand 702.63b from one-liner to two ATOMs:
- ATOM-702.63b-001: "Vanishing with 0 time counters: neither the upkeep removal trigger nor the sacrifice trigger fires. Permanent persists indefinitely."
- ATOM-702.63b-002: "Vanishing with 0 time counters + external time counter added: upkeep removal resumes; when last counter is removed, sacrifice trigger fires."

### R-702.64: Absorb Per-Source

**Auditor:** Absorb applies to each damage source independently. Should test multiple sources.

**Response:** The current ATOM-702.64a-001 only tests one source. An ATOM with two sources dealing damage in the same step validates the per-source nature.

**Action:** Add ATOM-702.64a-002 — "Absorb N applies per source: two sources each dealing 3 damage to an Absorb 2 creature → 1 + 1 = 2 total damage dealt (each source prevented by 2)."

### R-702.69: Gravestorm — Storm Variant

**Auditor:** Similar logic to Storm.

**Response:** Correct. Gravestorm and Storm share the "copy for each [count]" pattern. The implementation should use the same spell-copying infrastructure.

**Action:** Add `cross-ref-storm` tag to 702.69. Add note: "Shares spell-copying infrastructure with Storm (702.40)."

### R-702.72: Champion — Can't/Won't Pay Case

**Auditor:** Test when the player can't (or chooses not to) exile a creature, and the champion is sacrificed.

**Response:** The current ATOM-702.72a-001 mentions this in passing ("If P0 had no valid creature to exile, the champion creature is sacrificed") but it's a clause, not a standalone test.

**Action:** Add ATOM-702.72a-002 — "Champion with no valid target: if no other [object] exists to exile, the champion creature is sacrificed." Add ATOM-702.72a-003 — "Champion with valid target but player chooses not to exile: champion is sacrificed (the 'unless' clause means choosing not to exile = sacrifice)."

### R-702.79: Persist Must Use LKI

**Auditor:** Must use the LKI system.

**Response:** Correct. Persist checks "if it had no -1/-1 counters on it" — this is a check on the creature's state at the time it died, which is LKI. The creature is already in the graveyard when persist's trigger resolves.

The current ATOM-702.79a-001 doesn't explicitly mention LKI. And the ticket says "DEFERRED — Phase 8" but should note the LKI dependency.

**Action:** Update ATOM-702.79a-001 and 702.79a-002 to explicitly reference LKI. Update ticket to "DEFERRED — Phase 8 + Phase 5 LKI dependency."

### R-702.80a-001: Wither Creature Is Still Destroyed

**Auditor:** The test should also verify that the wither creature takes lethal combat damage from the blocker and is destroyed normally.

**Response:** Good catch. The current test focuses on the 4/4 receiving counters but doesn't verify the wither creature's fate. In the combat scenario (3/3 wither vs 4/4 blocker), the 4/4 deals 4 damage to the 3/3, which is normal damage (the 4/4 doesn't have wither), so the 3/3 takes 4 marked damage on 3 toughness → destroyed by SBA.

**Action:** Update ATOM-702.80a-001 expected result to include: "The 3/3 wither creature also takes 4 regular damage from the 4/4 blocker and is destroyed by SBA (lethal damage). Wither only changes how the wither source's damage is applied — the blocker's damage to the wither creature is normal marked damage."

---

## Summary of Actions

### New ATOMs to Add
| ID | Rule | Summary |
|---|---|---|
| ATOM-702.6a-003 | 702.6a | Equip is illegal at instant speed (negative case) |
| ATOM-702.6d-001 | 702.6d | Multiple equip abilities: generic works on all creatures |
| ATOM-702.16a-002 | 702.16a | Protection from [cardname] |
| ATOM-702.16a-003 | 702.16a | Protection from snow |
| ATOM-702.16k-002 | 702.16k | Stolen permanent not subject to protection (multiplayer) |
| ATOM-702.18a-003 | 702.18a | Opponent can't target shroud creature either |
| ATOM-702.30a-002 | 702.30a | Echo re-triggers after control change |
| ATOM-702.43a-002 | 702.43a | Modular N enters with N counters |
| ATOM-702.46a-002 | 702.46a | Soulshift can't return Spirit with MV > N |
| ATOM-702.49a-002 | 702.49a | Ninjutsu after first-strike damage |
| ATOM-702.49a-003 | 702.49a | Ninjutsu in end-of-combat (no damage) |
| ATOM-702.52b-001 | 702.52b | Dredge with insufficient library (negative case) |
| ATOM-702.62a-001/002/003 | 702.62a | Suspend split into 3 separate ATOMs |
| ATOM-702.63b-001 | 702.63b | Vanishing 0 counters = permanent persists |
| ATOM-702.63b-002 | 702.63b | Vanishing 0 + external counter = loop resumes |
| ATOM-702.64a-002 | 702.64a | Absorb per-source with multiple sources |
| ATOM-702.72a-002 | 702.72a | Champion with no valid target → sacrifice |
| ATOM-702.72a-003 | 702.72a | Champion with valid target, player declines → sacrifice |

### New COMPs to Add
| ID | Rules | Summary |
|---|---|---|
| COMP-702-007 | 702.11b + 608.2b | Hexproof granted between cast and resolution → fizzle |
| COMP-702-008 | 702.24a + Solemnity | Cumulative Upkeep + "counters can't be placed" |

### New Tickets
| ID | Description | Phase |
|---|---|---|
| NEW-3 | Unified Evasion Framework (BlockerFilter) | Phase 8 |
| NEW-4 | Unified Trample DP (TrampleContext) | Phase 8 |

### Reclassifications
| Rule | From | To |
|---|---|---|
| 702.6d | PURE-DEF | TESTABLE |
| 702.22 | DEFERRED (niche, extremely low priority) | DEFERRED Phase 8 (vintage-legal, medium priority) |
| 702.52b | PURE-DEF | TESTABLE (boundary) |
| 702.63b | One-liner DEFERRED | TESTABLE (2 ATOMs) |

### Edits to Existing Tests
| ID | Change |
|---|---|
| ATOM-702.16e-001 | Rewrite with clean combat board (remove Bolt confusion) |
| ATOM-702.26a-001 | Focus on phase-out event specifically |
| ATOM-702.51a-001 | Fix math: pool {1} → {2} |
| ATOM-702.62a-001 | Split into 3 separate ATOMs |
| ATOM-702.79a-001/002 | Add LKI reference, update ticket to note LKI dependency |
| ATOM-702.80a-001 | Add expected result for wither creature being destroyed |

### META Notes to Add
- G2: Unified Evasion Framework design sketch
- G3: Quality/Text representation (ProtectionQuality enum)
- G4: Copy-spell vs copy-card distinction
- R-702.16j: Protection-from-everything as enum variant, not base case
- R-702.19c: TrampleContext DP unification proposal
- R-702.47: Three explicit blockers for splice deferral
- R-702.55: Haunting is a designation, not a characteristic
- R-702.29: Cycling instant-speed covered by base framework
- R-702.53: Transmute cross-ref with cycling
- R-702.69: Gravestorm cross-ref with storm

### Tags/Notes to Add
- 702.26f: Why deferred (Phase 5 layers dependency)
- 702.26m: Why deferred (Phase 8 phasing dependency)
- 702.35c: Replacement effect + trigger tracking dependency chain
- 702.37: Cross-ref to rule 708 face-down infrastructure
- 702.75: Cross-ref to rule 708 face-down infrastructure
