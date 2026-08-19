# Session 5 Audit Response — Planned Changes to `session-5.md`

> **Audit source:** `plans/atomic-tests/session-5-audit.md`
> **Referenced files:** `plans/atomic-tests/603-2f-complexity.md`, `plans/atomic-tests/607-2-examples.md`
> **Generated:** 2026-04-05

---

## General Notes Response

Agree on both points:
1. **Difficulty spike acknowledged.** Chapter 6 rules are deeply interconnected. Will add a top-level design note to session-5.md about architectural implications.
2. **Gatherer rulings as supplementary test sources.** Will add a META note that card-specific Gatherer rulings serve as acceptance criteria when implementing individual cards.

---

## Rule-by-Rule Change Plan

### 601.2a — Add test for non-hand casting zones

**Action: ADD ATOM-601.2a-003**
- Test casting from graveyard (flashback pattern)
- Test casting from exile (Light Up the Stage pattern)
- Verifies `move_to_stack` works from zones other than Hand

### 601.2b — Add X-in-additional-cost test + last-line note

**Action: ADD ATOM-601.2b-007**
- Card: Devastating Summons ({R}, additional cost: sacrifice X lands, create two X/X tokens)
- Tests that X is chosen during 601.2b even when X appears only in additional cost, not mana cost
- X value stored on StackEntry, used later in 601.2f/h

**Action: ADD implementation note to 601.2b section**
- Note about last line: "Previously made choices may restrict options." Strategy: build engine so constraints compose naturally (e.g., flashback sets zone=graveyard, morph sets face-down=true, these restrict later choices automatically). Not a separate test — emergent from correct pipeline implementation.

### 601.2c — Add tests for conditional targets + alternative targets

**Action: ADD ATOM-601.2c-006**
- Card: Probe ({2}{U}, Kicker {1}{B}). Unkicked: no target. Kicked: target player discards 2.
- Tests: "spell cast as though it did not require those targets" when kicker not paid

**Action: ADD ATOM-601.2c-007**
- Card: Bloodchief's Thirst ({B}, Kicker {2}{B}). Unkicked: destroy target CMC≤2. Kicked: destroy target creature or planeswalker (no CMC restriction).
- Tests: alternative targets based on additional cost — different target criteria, not just presence/absence

**Action: ADD implementation note**
- Architectural question: distinguish "no targets" vs "untargeted" at StackEntry level. Answer: Yes — StackEntry should have `targets: Vec<TargetSlot>` where slots can be `Required`, `Conditional(CostChoice)`, or `Alternative(CostChoice)`. Empty targets vec = untargeted; present-but-inactive slots = conditional targets not activated.

**Action: RE-EVALUATE existing 601.2c tests against each clause (second pass)**
- Verify each distinct clause maps to an ATOM test. Document which clause each tests.

### 601.2d — Add DecisionProvider note

**Action: ADD implementation note to 601.2d section**
- Division/distribution choices must route through `DecisionProvider::choose_distribution()`. Note this for future DP interface expansion.

### 601.2e — Add rollback/snapshot implementation note

**Action: ADD META-GAMESTATE-SNAPSHOT entry (deferred — no architectural risk)**
- Casting rollback requires GameState snapshot before 601.2a. On failure at any step, restore snapshot.
- Note overlap with loop detection (rule 731) — both need state snapshot/comparison infrastructure.
- Potential implementation: clone mutable portions of GameState before 601.2a; restore on failure. Simple and correct.
- **Deferred:** No architectural decisions needed now. The `GameState` struct is already a complete, concise representation of game state, which is a sufficient starting point for both rollback and future loop detection. Implementation can happen when casting pipeline (T18) is built.

### 601.2f — Add cost-reduction ordering test + defer 003

**Action: ADD ATOM-601.2f-004**
- Multiple cost reduction effects: player chooses application order via DecisionProvider
- Minimal board: two different reduction effects that could apply in either order. Test that DP is prompted.

**Action: MODIFY ATOM-601.2f-003**
- Add note: "See also ATOM-601.2h-001 which tests cost lock-in more thoroughly. This test focuses specifically on the lock-in *preventing* later modifications."

### 601.2h — Add cost-payment ordering test (Omnath + Momentous Fall)

**Action: ADD ATOM-601.2h-003**
- Cards: Omnath, Locus of Mana + Momentous Fall
- Scenario: Omnath has 4 green mana in pool → power/toughness = 5/5 (1/1 base + 4 green mana). Cast Momentous Fall (sacrifice Omnath as additional cost + pay {2}{G}{G}).
- Order A: Sacrifice Omnath first → LKI P/T = 5/5 → draw 5, gain 5. Then pay {2}{G}{G} from pool.
- Order B: Pay {2}{G}{G} first → pool has 0 green → Omnath is 1/1 → sacrifice → LKI P/T = 1/1 → draw 1, gain 1.
- Tests that cost payment ordering matters and is player-controlled via DP.

### 601.2i — Add characteristic-modifying effect test

**Action: ADD ATOM-601.2i-003**
- Mycosynth Lattice ("All spells are colorless") on battlefield.
- Cast a red spell. After 601.2i completes, spell on stack should be colorless.
- Tests that effects modifying spell characteristics apply at the "spell becomes cast" step.

### 601.3 — Add META note

**Action: ADD META-CAST-PERMISSION-LAYERS**
- Note: 601.3 is a meta-rule. Multiple rule subsystems feed into cast permission: timing (505.6a/b), prohibitions (L15 CantCastSpells), flash grants (601.3b–d), zone permissions (601.3f). Implementation should have a single `can_begin_casting()` function that queries all subsystems.

### 601.3a — Add implementation note

**Action: ADD implementation note**
- Flag this as requiring careful architectural thought. The look-ahead for mutable-during-proposal characteristics is a constraint-satisfaction problem. Implementation needs a "tentative proposal" phase.

### 601.3c — Add alternative cost flash test

**Action: ADD ATOM-601.3c-002**
- Card pattern: Primal Prayers ("You may cast creature spells with MV 3 or less by paying {E}. If you cast this way, you may cast it as though it had flash.")
- Tests: alternative cost grants flash, not just additional cost

### 601.3e — Add adventure integration test note

**Action: ADD implementation note**
- Adventure cards (e.g., Bonecrusher Giant) are an excellent integration test for 601.3e + flash interactions.
- Note: "Each half of an Adventure card should be testable for flash independently when an effect grants flash to one characteristic set."
- Defer actual test spec to Phase 8/9 when Adventure is in scope.

### 601.3f — Add negative case test

**Action: ADD ATOM-601.3f-002**
- Card A is face-down in exile, exiled by Player 0's effect (Player 0 can look at it, can cast it).
- Card B is face-down in exile, exiled by Player 1's effect (Player 0 cannot look at it).
- Player 0 attempts to cast Card B → Illegal. Player 0 should not receive any information about Card B's characteristics.
- Tests both the permission check and the information leak prevention.

### 601.4 — Add META note on scope

**Action: ADD implementation note**
- This rule is vaguely worded to permit a class of intra-step look-ahead interactions. Current test (Inscription of Abundance) covers the kicker→mode case. Note that other cards may exercise this rule differently. Flag for re-evaluation when new cards are added.

### 602.1e — Add single-condition test

**Action: ADD ATOM-602.1e-002**
- Only one cost modifier (increase by {1}). Activation cost {2},{T} → total {3},{T}.
- Prevents false pass where neither effect applies.

### 602.2a — Add characteristic absence test

**Action: ADD ATOM-602.2a-003**
- Ability object on stack: verify it has no card name, no mana cost, no color, no card types.
- Tests that ability objects are distinct from spell objects in their characteristic profile.

### 602.4 — Add implementation note with Urianger example

**Action: ADD implementation note**
- Reference Urianger Augurelt scenario from audit as integration test candidate.
- Note: costs are locked at payment time, so re-activating a "spells cost less" ability doesn't retroactively discount already-cast spells. This is implicitly handled by the lock-in architecture.

### 602.5c — Add Necrotic Ooze integration test note

**Action: ADD COMP note or implementation note**
- Necrotic Ooze + 2× Skinshifter in graveyard: Ooze gains two separate instances of the activated ability, each with independent "once each turn" restriction.
- Good Phase 8 integration test. Note for future.

### 602.5d — Add design note

**Action: ADD implementation note**
- Sorcery-speed timing check should be a shared utility function reused across: spell casting (505.6a), land play (505.6b), loyalty abilities (606.3), "activate only as a sorcery" (602.5d). Single `passes_sorcery_timing()` function.

### 602.5e — Fix explanation

**Action: MODIFY ATOM-602.5e-001**
- Current explanation is generic. Replace with: "This restriction exists primarily for cards like Lion's Eye Diamond, preventing mana abilities from being activated during casting to circumvent cost-payment rules. 'Activate only as an instant' means the ability can be activated any time you have priority, but NOT during the casting/activation process (mana ability windows, etc.)."

### 603.1b — Add architectural note

**Action: ADD META-MULTI-CONDITION-TRIGGERS**
- Multi-condition triggers ("whenever you cast a creature AND an artifact in the same turn") require per-turn event tracking. Architecture: `TurnEventLog` on GameState tracking event categories per player per turn. Trigger matcher checks log for all conditions being met.
- Flag: This is an architectural decision needed before Phase 7 implementation.

### 603.2b — Add composition test note

**Action: ADD to COMP section or implementation note**
- Integration test: multiple "at beginning of upkeep" triggers from both players. Test APNAP ordering + player-chosen ordering within same player's triggers.

### 603.2d — Add architectural note

**Action: ADD implementation note**
- Trigger multipliers (Panharmonicon) could be implemented as a game-engine level effect that modifies trigger count during the `check_triggers()` pass. Not a replacement effect — it's a count modifier. Needs design decision before Phase 7.

### 603.2e — Add equip-to-same-creature test

**Action: ADD ATOM-603.2e-002**
- Equipment with "becomes attached" trigger. Equip targeting the creature it's already attached to.
- Expected: Equipment doesn't "become attached" (it already was). Trigger does NOT fire.

### 603.2f — Add stress test reference + 603.10 overlap note

**Action: ADD META-HIDDEN-ZONE-TRIGGER-COMPLEXITY**
- Reference `plans/atomic-tests/603-2f-complexity.md` for the Library of Leng + Guerrilla Tactics + Future Sight scenario.
- This scenario exercises: hidden zone trigger suppression (603.2f), "look back in time" (603.10), replacement effects on discard (Library of Leng), and top-of-library reveal (Future Sight).
- Architectural takeaway: trigger checking must happen AFTER all replacement effects resolve and zone changes finalize, using the final public/hidden zone status of each object.
- Note the broader design concern from the audit: "MtG is arbitrarily complex. Trust composition + unit tests where correct behavior is well-defined."

### 603.2h — Add multiple-triggers-on-stack test

**Action: ADD ATOM-603.2h-002**
- Card: Nykthos Paragon ("Whenever you gain life, you may put counters. Do this only once each turn.")
- Scenario: Two lifelink creatures deal combat damage simultaneously → two Paragon triggers on stack.
- First trigger resolves: player takes the action (puts counters). Second trigger resolves: "do this only once" prevents the action; DP is NOT prompted.
- Tests that the once-per-turn check happens at resolution, not at trigger time.

### 603.3 — Add characteristic test note

**Action: ADD implementation note**
- Similar to 602.2a: triggered ability objects on stack should be checked for characteristic profile (no card name, no mana cost, etc.).

### 603.3a — Add implementation note on controller-change timing

**Action: ADD implementation note**
- Audit notes this is hard to trigger in practice. Note for future: multiplayer concession during trigger stacking may be the primary scenario. Defer concrete test until Phase 9 multiplayer.

### 603.3b — Add two-tier system test + examples

**Action: ADD ATOM-603.3b-002**
- Test the two-tier trigger stacking: first tier = triggers whose condition isn't "another ability triggering." Second tier = triggers that trigger from other triggers (Strict Proctor, Aboleth Spawn pattern).
- Cards: Strict Proctor ("Whenever a permanent entering causes a triggered ability to trigger, counter that ability unless its controller pays {2}") + a creature with ETB trigger.
- Expected: creature ETB trigger goes on stack first (tier 1). Strict Proctor's trigger goes on stack second (tier 2, above the ETB trigger). Proctor resolves first, potentially countering the ETB.

**Action: ADD META-TWO-TIER-TRIGGER-STACKING**
- Architectural note: engine needs a way to classify triggers as "triggers-on-trigger" vs "triggers-on-event." During `stack_pending_triggers()`, process in two passes.

### 603.3c — Add fizzle test

**Action: ADD ATOM-603.3c-002**
- Modal triggered ability: "Choose one: destroy target artifact or destroy target enchantment." Neither permanent type on battlefield.
- Expected: No legal mode can be chosen → ability removed from stack without resolving.

### 603.3d — Add implementation note

**Action: ADD implementation note**
- Audit asks if this is independently testable or subsumed by 601.2 tests. Answer: The rule itself is a structural reference ("follow 601.2c–d"). The existing ATOM test covers the key behavior (no legal targets → removed from stack). Don't split further — the 601.2c/d tests cover the individual steps.

### 603.4 — Split 001 into two tests

**Action: MODIFY ATOM-603.4-001**
- Narrow to: condition true at trigger time AND true at resolution → ability resolves normally.

**Action: ADD ATOM-603.4-003**
- Condition true at trigger time, false at resolution → ability does nothing (removed).
- (Current 002 covers condition false at trigger time → doesn't trigger at all. This new test covers the other failure mode.)

### 603.6 — Add LKI system overlap note

**Action: ADD implementation note**
- Cross-reference META on LKI system (L18). Zone-change trigger resolution depends on LKI infrastructure.

### 603.6a — Modify test to include newcomer self-trigger

**Action: MODIFY ATOM-603.6a-001**
- Add to board: entering creature ALSO has an ETB trigger (e.g., "When this enters, draw a card").
- Expected: Both Soul Warden's trigger AND the entering creature's own trigger fire.
- Tests the "all permanents including newcomers are checked" clause.

### 603.6b — Add implementation note

**Action: ADD implementation note**
- This is NOT a replacement effect. Continuous effects from the layer system are applied before trigger checking. The permanent never exists on the battlefield without modifications. Implementation: layer recalculation happens synchronously during `move_to_battlefield()`, before `check_triggers()`.

### 603.6c — Add multiplayer note + first-zone-only test

**Action: ADD implementation note**
- Multiplayer edge case: player leaves game → all their permanents leave battlefield simultaneously → LTB triggers for each.
- Extractor Demon example: 10 creatures leave → 10 separate triggers, each requiring target choice via DP.

**Action: MODIFY ATOM-603.6c-001 or ADD ATOM-603.6c-002**
- Strengthen the "first zone only" test: Enduring Renewal ("When a creature is put into your graveyard from the battlefield, return it to your hand"). Creature dies → goes to graveyard. Before trigger resolves, opponent exiles it from graveyard with instant-speed effect. Trigger resolves → looks in graveyard → card not found → trigger does nothing.

### 603.6e — Add attached-creature tracking test

**Action: ADD ATOM-603.6e-002**
- Card: Abduction (Aura, "When enchanted creature dies, return that card to the battlefield under its owner's control.")
- Tests that the Aura's LTB-adjacent trigger can find the object the enchanted creature became (in graveyard), not just the Aura itself.

### 603.7a — Add implementation note on orphaned triggers

**Action: ADD implementation note**
- Audit raises valid efficiency concern. Delayed triggers referencing objects that can never satisfy the condition are orphaned data. Potential optimization: lazy cleanup of delayed triggers whose tracked ObjectId has an expired epoch. Not critical for correctness — just memory hygiene. Defer to Phase 8 optimization pass.

### 603.7b — Add simultaneous-event choice test

**Action: ADD ATOM-603.7b-002**
- Card: Tatsumasa, the Dragon's Fang (exile self, create token, "Return Tatsumasa when that token dies") + token doubler (Anointed Procession → two tokens created).
- Both tokens die simultaneously. Controller chooses which death triggers the delayed ability.
- Tests: "If trigger event occurs more than once simultaneously, controller chooses which event causes the ability to trigger."

### 603.7c — Add implementation note on targeting vs delayed-trigger tracking

**Action: ADD implementation note**
- Audit correctly notes this differs from targeting rules. Delayed triggers track by ObjectId, not by characteristics. If ObjectId still exists in expected zone, effect applies regardless of characteristic changes. If ObjectId left its zone (400.7 new object), effect can't find it. This is a simpler system than target legality rechecking.

### 603.7f — Add note to revisit

**Action: MODIFY ATOM-603.7f-001**
- Add note: "This test is intentionally abstract. Concrete card examples for this pattern are rare. Revisit and specify more concretely when implementing Phase 6 replacement effects."

### 603.8 — CORRECTION: State triggers fire mid-resolution

**Action: ADD implementation note + ADD ATOM-603.8-002**
- **CORRECTION:** My original note was wrong. The CR Example for 603.8 is explicit: "If its controller casts a spell that reads 'Discard your hand, then draw that many cards,' the ability will trigger during the spell's resolution because the player's hand was momentarily empty."
- State triggers check continuously, NOT only at priority-granting time. If the game state momentarily matches the trigger condition during resolution, the trigger fires.
- **Architectural implication:** The engine's state-trigger checking cannot be limited to priority checkpoints. It must also run during effect resolution steps — specifically after each discrete game action within a resolving effect (e.g., after the discard step but before the draw step of "discard hand, draw that many").
- This is a significant architectural constraint for Phase 7. The `resolve_effect()` pipeline must interleave state-trigger checks between sequential sub-effects.

**ATOM-603.8-002:**
- **Rule:** 603.8 — State triggers fire mid-resolution when condition momentarily becomes true.
- **Mechanism:** State trigger fires during spell resolution
- **Minimal Board:** A permanent has "Whenever you have no cards in hand, draw a card." Player has 3 cards in hand.
- **Action:** Player casts "Discard your hand, then draw that many cards" (3 cards discarded, then draw 3).
- **Expected Result:** After the discard step, hand is momentarily empty → state trigger fires and goes on pending triggers queue. Then the draw step draws 3 cards. After resolution, the state trigger is placed on the stack. When it resolves, player draws 1 more card.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.10 — Fix document structure

**Action: MODIFY 603.10 section**
- Move the Example (artifact + creatures destroyed simultaneously) from 603.10 to 603.10a where it belongs.
- 603.10 top-level: reclassify as PURE-DEF (defines the concept of "look back in time"). Keep the TESTABLE classification on 603.10a.

### 603.10c — Add re-attach trigger test

**Action: ADD ATOM-603.10c-002**
- Equipment with "becomes unattached" trigger. Move equipment from creature A to creature B (via Equip targeting B).
- Expected: Equipment becomes unattached from A → trigger fires. Then attaches to B.

### 604.5, 604.6 — Add per-condition tests

**Action: ADD ATOM-604.5-002**
- Test "As an additional cost to cast" static ability specifically (separate from the existing test).
- Test "You may cast without paying mana cost" as its own test case.

**Action: ADD ATOM-604.6-002**
- Test "Cast only during combat" restriction specifically.
- Test "You can't cast this spell" prohibition specifically.

### 604.7 — Fix example

**Action: REPLACE ATOM-604.7-001 board/action/expected**
- Current example (Glorious Anthem + dead creature) is wrong per audit.
- New example: Saproling Burst (fading 7, activated ability creates token whose P/T = fade counters on Burst).
- Scenario: Activate Saproling Burst's ability. Before it resolves, Burst is destroyed. Token enters. Token's CDA references "number of fade counters on Saproling Burst" — but Burst is gone. Static ability can't use LKI → token is 0/0 → dies to SBA.
- This correctly demonstrates the rule's actual meaning.

### 605.1a, 605.1b — Add per-qualifier failure tests

**Action: ADD ATOM-605.1a-003**
- Loyalty ability that adds mana (e.g., Chandra, Torch of Defiance's +1: "Add {R}{R}"). Satisfies (1) no target and (2) adds mana, but is a loyalty ability → NOT a mana ability. Uses the stack.

**Action: ADD ATOM-605.1a-004**
- Card: Explosive Welcome ({7}{R}, "Deals 5 to target, 3 to another target. Add {R}{R}{R}."). Has targets → NOT a mana ability despite adding mana. Uses the stack.

**Action: ADD implementation note**
- Triggered vs activated mana ability handling: classification is the same system (check 3 criteria). Resolution differs (activated = immediate, triggered = immediate after triggering mana ability). One set of classification tests covers both; resolution tests are separate.

### 605.3a — Add multi-clause tests

**Action: ADD ATOM-605.3a-002**
- Test: activate mana ability when a rule asks for mana payment (e.g., during a replacement effect that requires mana, or Rhystic Study's "unless they pay {1}").

**Action: ADD ATOM-605.3a-003**
- Test: activate mana ability during ability activation that requires mana (not just spell casting).

### 605.3c — Add non-tap mana ability re-activation test

**Action: MODIFY ATOM-605.3c-001**
- Audit correctly notes tap-cost inherently prevents re-activation. Change example to a mana-filtering ability: "{1}: Add one mana of any color." This ability could theoretically be activated again before resolving if this rule didn't exist.
- Expected: Engine prevents re-activation until the first activation resolves.

### 606.4 — Add counter-doubling composition test note

**Action: ADD to COMP section or implementation note**
- Counter doubling (Doubling Season) + planeswalker +1 ability → +2 loyalty.
- Good integration test for Phase 8 planeswalker implementation.

### 606.6 — Add 606.5 interaction test note

**Action: ADD implementation note**
- Edge case: planeswalker with 3 loyalty, -4 ability, but a Carth the Lion effect adds +1 to loyalty costs → net cost is -3. Now affordable. Tests 606.5 + 606.6 interaction.

### 607.1c — Fix example

**Action: REPLACE ATOM-607.1c-001 board/action/expected**
- Current example incorrectly describes two abilities in one sentence. 607.1c requires a SINGLE ability that both causes actions AND refers to those actions.
- New example: Tyrant's Choice ({1}{B} Sorcery, "Will of the council — Starting with you, each player votes for death or torture. If death gets more votes, each opponent sacrifices a creature. If torture gets more votes or tied, each opponent loses 4 life.")
- Single ability: causes the vote (action) and refers to the vote result (reference). Self-linked.

### 607.1d — Add deferral note

**Action: MODIFY ATOM-607.1d-001**
- Add note: "Audit recommends deferring concrete implementation. No clear card examples found. The architectural change is an exception (cross-object linking), not a fundamental engine change. Low retrofit cost. Defer to Phase 8 when concrete cards exercise this rule."
- Keep test spec for reference but mark as DEFERRED.

### 607.2 — Integrate card examples from `607-2-examples.md`

**Action: UPDATE tests for subrules a through q**
- 607.2a: Reference Isochron Scepter (imprint + cast copy)
- 607.2b: Reference Shared Fate (replacement exile + play from exile)
- 607.2c: Reference Diabolic Servitude (ETB + "put onto battlefield with" + LTB)
- 607.2d: Reference Haktos the Unscarred (random choice + protection from non-chosen), add Voice of All and True-Name Nemesis as alternatives
- 607.2e: Reference Meddling Mage (note card name), keep Voice of All
- 607.2f: Reference Archangel of Strife (war/peace word choice)
- 607.2g: Reference Phyrexian Processor (pay life as enters + X/X token)
- 607.2h: Reference Keranos, God of Storms (static reveal + triggered abilities in same paragraph)
- 607.2i: Already has CR example, keep as-is
- 607.2j: Note "no known cards" per examples file
- 607.2k: Note Champion keyword, keep DEFERRED
- 607.2m: Reference Monastery Siege (anchor word / Khans-or-Dragons)
- 607.2q: Reference Champion of the Path (behold + exile + LTB return)

### 608.2 — Consider adding atomic test

**Action: ADD ATOM-608.2-001**
- Audit asks if this needs testing. It's currently BOUNDARY-DEF (header).
- Add a test that verifies the resolution procedure order: 608.2a–b checks happen before 608.2c–m instruction execution, and 608.2n/p happen last.
- Minimal test: a targeted triggered ability with intervening-if. Verify: (1) intervening-if checked (608.2a), (2) target legality checked (608.2b), (3) effect executed (608.2c), (4) ability removed from stack (608.2n), (5) post-resolution triggers fire (608.2p). All in order.

### 608.2b — Add partial-target resolution test

**Action: ADD ATOM-608.2b-005**
- Card: Jagged Lightning ({3}{R}{R}, "Deals 3 damage to each of two target creatures.")
- Target creature A and creature B. Before resolution, creature A gains protection from red.
- Expected: Creature A is illegal target, but creature B is still legal. Spell resolves. Creature B takes 3 damage. Creature A is unaffected.
- Tests partial-target resolution with a simpler card than Plague Spores.

### 608.2c — Reclassify or add implementation note

**Action: ADD implementation note**
- Audit says this may not be testable in our engine since we use pre-tokenized effect structures. Agree partially: the "follow instructions in order" is inherently how `resolve_effect(Effect::Sequence(...))` works. Keep test but add note that this is implicitly verified by all resolution tests. The "can't be regenerated" modifier is a good standalone test of instruction ordering.

### 608.2d — Add flexible distribution test

**Action: ADD ATOM-608.2d-003**
- "Distribute 3 +1/+1 counters among creatures you control." Player controls 3 creatures but chooses only 2.
- Expected: Legal. Player puts at least 1 counter on each chosen creature (e.g., 2 and 1). Third creature gets nothing (wasn't chosen).
- Tests that player can choose fewer objects than the maximum, as long as each chosen object gets at least 1.

### 608.2f — Add second example test

**Action: ADD ATOM-608.2f-002**
- Test the CR's second example pattern: if simultaneous actions can't truly be simultaneous, use APNAP.
- Scenario: Effect says "Each player sacrifices a creature." APNAP order determines who sacrifices first (relevant if sacrifice triggers matter).

### 608.2g — Add implementation note on non-uncounterability

**Action: ADD implementation note**
- Clarify: casting during resolution does NOT make the cast spell uncounterable. After the resolving spell/ability finishes, players get priority with the newly cast spell on the stack. Opponents can respond to it normally.

### 608.2i — Add Fight mechanic note

**Action: ADD implementation note**
- Fight (Chapter 7) is an exception to the historical look-back pattern. Note for future: when reaching Fight rules, cross-reference 608.2i.

### 608.2m — Add uncertainty note

**Action: MODIFY ATOM-608.2m-001**
- Add note: "Audit questions if this is truly testable or is a catchall. The scenario (spell leaving stack during its own resolution) is extremely rare. Keep test spec for completeness but mark as LOW PRIORITY. May be untestable in practice."

### 608.2p — Fix test (resolution triggers, not cast triggers)

**Action: REPLACE ATOM-608.2p-001**
- Current test incorrectly uses a cast trigger ("whenever a player casts an instant or sorcery"). This is a 601.2i trigger, not a 608.2p trigger.
- New example: Maelstrom Muse ("Whenever this creature attacks, the next instant or sorcery spell you cast this turn costs {X} less, where X is this creature's power as this ability resolves.")
- Actually, Muse is an attack trigger. Better example: any "whenever a spell or ability resolves" effect. These are rare. Consider using a delayed trigger pattern or a "storm count" analog.
- If no clean single-card example exists, use: "A permanent has 'Whenever an instant or sorcery spell resolves, put a +1/+1 counter on this creature.'" (hypothetical but mechanically clear).

### 608.3a — Add interaction note

**Action: ADD implementation note**
- Note: rules 400.4a (instants/sorceries can't enter battlefield) ensures that only permanent spells reach 608.3a. This makes 608.3a "safe" — it will never be asked to resolve a non-permanent spell.

### 608.3b — Add mutate note

**Action: ADD implementation note**
- Audit says mutate should be tested here. Currently marked OUT-OF-SCOPE in 608.3d. Revisit scope decision — mutate appears in Pioneer/Modern. If brought into scope, add test for "permanent spell resolves, tries to mutate onto target creature."

### 608.3d — Change scope: OUT-OF-SCOPE → DEFERRED

**Action: RECLASSIFY 608.3d**
- Mutate appears in Pioneer and Modern — must be supported eventually.
- Change classification from OUT-OF-SCOPE to **DEFERRED — Phase 9**.
- This signals intent to implement while not blocking current work.
- Update OUT-OF-SCOPE summary table (remove 608.3d) and DEFERRED summary table (add 608.3d).

---

## Summary Table Updates

After all changes above are applied, update the summary tables:

1. **ATOM count:** ~18 new ATOMs added, ~4 modified, ~1 replaced
2. **META entries:** ~6 new META entries
3. **Implementation notes:** ~20 new implementation notes scattered throughout
4. **COMP tests:** ~2 new composition test references
5. **Classification changes:**
   - 603.10: TESTABLE → PURE-DEF (move test content to 603.10a)
   - 607.1d: TESTABLE → DEFERRED (keep spec for reference)
   - 608.3d: OUT-OF-SCOPE → DEFERRED — Phase 9
6. **New tickets:** Review whether any new tickets are needed from the added tests
7. **Gap report:** Add entries for multi-condition trigger tracking (603.1b), two-tier trigger stacking (603.3b), and GameState snapshot infrastructure (601.2e)

---

## Referenced Files Integration

### From `603-2f-complexity.md`
- Integrated as META-HIDDEN-ZONE-TRIGGER-COMPLEXITY entry under 603.2f
- Key architectural takeaway documented: trigger checking must happen after replacement effects and zone changes finalize

### From `607-2-examples.md`
- Card examples integrated into 607.2a–q tests as concrete card references
- Noted subrules l and o don't exist in CR (formatting artifact)
- Subrule p flagged for further investigation
- Subrule n (Conspiracy) confirmed OUT-OF-SCOPE

---

## Open Questions — RESOLVED

### Q1: 608.3d / Mutate scope
**Answer: Yes.** Mutate is DEFERRED — Phase 9 (not OUT-OF-SCOPE). Updated above.

### Q2: 607.1d deferral
**Answer: Yes.** Cross-object linked abilities test deferred. Keep spec for reference.

### Q3 + Q4 (merged): State tracking, trigger detection, and loop detection architecture

> **Full analysis:** See [`state-tracking-architecture.md`](state-tracking-architecture.md) for the complete discussion including approach comparisons, existing simulator research, implementation sketches, and the D26 voluntary shortcut system design.

**Summary of decisions:**

- **Trigger tracking (603.8, 603.1b, cross-turn, resolution counting):** Delta log — structured `GameDelta` entries emitted by state-mutating methods. Zero steady-state overhead. Compatible with `pending_triggers` queue architecture.
- **Loop detection (731):** Tiered — forced-action counter (Tier 1) + full-state hash at quiescent points (Tier 2) + deep compare on hash collision (Tier 3).
- **Voluntary shortcuts (729):** D26 "execute first, then declare" pattern — player executes 2 iterations normally, declares shortcut, engine validates via delta-log transcript comparison and bulk-applies. Shares `Hash`/`PartialEq` infrastructure with Tier 2/3.
- **Zobrist hashing:** Rejected (MTG state space not finitely enumerable).
- **`im` crate:** Rejected (pervasive refactor + constant 2-5x performance tax not justified).
- **Pregame sweep:** Good future optimization for delta emission filtering. Deferrable.
- **All deferred to Phase 7.** No current code depends on these decisions. Compatible with existing architecture.

### Q5: 608.2p — Concrete card for resolution triggers
**Answer: Ashling, Flame Dancer.**

**Action: REPLACE ATOM-608.2p-001**
- Card: Ashling, Flame Dancer ({2}{R}{R}, Legendary Creature — Elemental Shaman)
- Ability: "Magecraft — Whenever you cast or copy an instant or sorcery spell, discard a card, then draw a card. If this is the second time this ability has resolved this turn, Ashling deals 2 damage to each opponent and each creature they control. If it's the third time, add {R}{R}{R}{R}."
- The "second/third time this ability has resolved this turn" clause is a direct test of resolution-count tracking (608.2p + 603.7h).
- **ATOM-608.2p-001 (revised):**
  - **Rule:** 608.2p — Abilities that trigger/track when a spell or ability resolves.
  - **Mechanism:** Resolution-count tracking on triggered ability
  - **Minimal Board:** Player controls Ashling, Flame Dancer. Player has 3 cards in hand. Opponent controls a 1/1 creature.
  - **Action:** Player casts 3 instant/sorcery spells sequentially, letting each Magecraft trigger resolve before casting the next.
  - **Expected Result:** 1st resolution: discard 1, draw 1 (no bonus). 2nd resolution: discard 1, draw 1, then Ashling deals 2 damage to each opponent and each creature they control (1/1 dies). 3rd resolution: discard 1, draw 1, then add {R}{R}{R}{R} to player's mana pool.
  - **Phase:** Phase 7 (triggered abilities)
  - **Ticket:** Phase 7 triggers

Note: Ashling also tests the Magecraft keyword and resolution-count delayed triggers (603.7h), making it an excellent cross-rule integration card.

---

*End of audit response — planned changes to `session-5.md`*
*All open questions resolved. Ready to apply changes to session-5.md on user approval.*
