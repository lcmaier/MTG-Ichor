# Session 8 — Audit Response (Round 1)

**Date:** 2026-04-09
**Source:** User audit notes on session-8.md (702.81–702.190)

Legend:
- **AGREE** — Will fold the change back into session-8.md
- **DISCUSS** — Needs alignment before folding
- **NOTE** — Acknowledged, tagged for future reference

---

## General Notes

### DFC Deferral to Phase 9

> Does it broadly make sense to defer double-faced cards? Feels like there would be some architectural implications now given how unique they are?

**DISCUSS.** The current deferral rationale: DFCs need a `Face` abstraction on `GameObject` (front/back characteristics, transform state, zone-dependent face selection). That's a data model change that ripples through `card_data`, `object`, `characteristics.rs`, and the layer system.

**The risk of deferring:** If we design the layer system (Phase 5) and triggered ability system (Phase 7) without DFC awareness, we might paint ourselves into a corner. Specifically:
- **Phase 5 (layers):** `get_effective_*` needs to know which face is "up" to determine base characteristics. If we hardcode single-face assumptions now, retrofitting is painful.
- **Phase 7 (triggers):** Transform triggers ("When this creature transforms...") need the trigger system to recognize face changes.

**Proposed compromise:** Add a `Face` enum/struct to the data model in Phase 5 as a **stub** — single-face cards just have one face, DFC cards get two faces but the transform machinery is Phase 9. This way:
- `characteristics.rs` queries the "active face" from day one
- Phase 9 just needs to implement the transform action + triggers, not restructure the data model

Keywords affected: Daybound/Nightbound (702.146), Disturb (702.147), More Than Meets the Eye (702.162), Prototype (702.160), Craft (702.167).

**Action:** Create a design note in the Phase 5 plan documenting the Face stub requirement. No session-8.md change needed — the tests are correctly deferred.

---

## Rule-Specific Notes

### 702.82b — Devour "count devoured" trigger

> Should test this, consider the effect "When this creature enters, draw a card for each creature it devoured."

**AGREE.** The CR explicitly defines 702.82b as the "it devoured" linkage. This is a TESTABLE linked ability, not just a definition.

**New test:**

```
ATOM-702.82b-001
- Rule: 702.82b — "It devoured" = creatures sacrificed via its devour ability
- Mechanism: Linked ability referencing devour count
- Minimal Board: Player casts a creature with "Devour 2" and "When this creature enters, draw a card for each creature it devoured." Player sacrifices 3 creatures.
- Action: Creature enters with 6 +1/+1 counters (3 × 2). ETB trigger references devour count.
- Expected Result: Player draws 3 cards (number of creatures devoured, not counters).
- Phase: Phase 7 + Phase 8
- Ticket: (same as 702.82a)
- Dependencies: Phase 7 (triggered), Devour count tracking (linked ability)
```

**Tag: LINKED-ABILITY-PATTERN** — Devour count, Exploit target, Tribute paid/not-paid all share a pattern where a replacement/ETB mechanic stores a value that a linked triggered ability references. Engine needs a "linked ability context" for this.

---

### 702.83 — Exalted: "attacks alone" vs "enters attacking"

> Exalted requires the engine to distinguish between creatures declared as attacking and creatures that "enter attacking"

**AGREE.** This is a critical architectural note. CR 506.5 defines "attacks alone" as "only creature **declared** as an attacker." Tokens that enter tapped and attacking are NOT declared as attackers.

**New test:**

```
ATOM-702.83a-002
- Rule: 702.83a + 506.5 — Exalted triggers even when tokens enter attacking
- Mechanism: "Attacks alone" checks declaration, not total attacking creatures
- Minimal Board: Player controls creature A with exalted and creature B with "Whenever this creature attacks, create a 1/1 Warrior token tapped and attacking." Only B is declared as attacking.
- Action: Declare attackers with only B. B's trigger creates a Warrior token tapped and attacking.
- Expected Result: Exalted triggers (B attacked alone — it was the only creature *declared* as attacking). B gets +1/+1. The Warrior token does NOT prevent exalted.
- Phase: Phase 7 + Phase 8
- Ticket: (same as 702.83a)
- Dependencies: Phase 7 (triggered), "declared as attacking" vs "enters attacking" distinction
```

**Tag: DECLARED-VS-ENTERS-ATTACKING** — Affects Exalted, Melee (702.121), and any "attacks alone" / "whenever a creature attacks" trigger. Engine must track how a creature became attacking (declared vs entered).

---

### 702.84 — Unearth: "exile instead of going to any other zone"

> Should also test the "exile it instead of going to any other zone" logic

**AGREE.** This is a replacement effect baked into unearth's definition (702.84a). It's the most interesting part of unearth mechanically.

**New test:**

```
ATOM-702.84a-002
- Rule: 702.84a — Unearth: "If it would leave the battlefield, exile it instead"
- Mechanism: Replacement effect on zone change (any non-exile destination → exile)
- Minimal Board: Player unearthed a creature from graveyard. Opponent casts "Return target creature to its owner's hand."
- Action: Bounce spell resolves targeting the unearthed creature
- Expected Result: Creature is exiled instead of returned to hand. The replacement applies to ANY zone change off battlefield (bounce, destroy, shuffle into library, etc.).
- Phase: Phase 6 (replacement) + Phase 8
- Ticket: (same as 702.84a)
- Dependencies: Phase 6 (replacement effects), Unearth zone-change replacement
```

---

### 702.85a — Cascade: "doesn't cast" case + MV skip

> Should test the "doesn't cast the spell" case where everything remains in exile. 001 should also test that cascade skips spells with greater MV

**AGREE.** Two additions:

**Test amendment for ATOM-702.85a-001:** Add to Expected Result: "Cascade exiles cards one at a time until finding a nonland card with lesser MV. Cards with equal or greater MV remain exiled and are not castable." (This is implicit but should be explicit.)

**New test:**

```
ATOM-702.85a-003
- Rule: 702.85a — Cascade: player chooses not to cast the found spell
- Mechanism: Optional cast from exile; decline → all exiled cards go to bottom in random order
- Minimal Board: Player casts a spell with cascade (MV 4). Cascade exiles 3 cards, finds a nonland with MV 2.
- Action: Player declines to cast the found spell
- Expected Result: All exiled cards (including the found spell) are put on the bottom of library in a random order. No spell is cast.
- Phase: Phase 7 + Phase 8
- Ticket: (same as 702.85a)
```

---

### 702.87 — Level Up: cross-reference rule 711

> Note cross reference with rule 711

**AGREE.** Tag added:

**Tag: CROSS-REF rule 711** — Level Up (702.87) uses the leveler card layout defined in rule 711. Level symbols define ability bands based on level counter count. Session 9B should cover rule 711 and generate tests for the level-symbol-to-ability mapping. Session-8 test for 702.87a covers the counter-placing activation; rule 711 tests will cover the "which abilities are active at which level" logic.

---

### 702.89 — Umbra Armor: lethal damage destruction prevention

> Should also test lethal damage destruction prevention

**AGREE.** Current test only covers "would be destroyed" (e.g., Doom Blade). Need to also test the lethal-damage path where SBA would destroy the creature.

**New test:**

```
ATOM-702.89a-002
- Rule: 702.89a — Umbra Armor prevents destruction from lethal damage
- Mechanism: Replacement effect intercepts SBA destruction from lethal damage
- Minimal Board: 3/3 creature enchanted by Aura with umbra armor. Creature takes 3 damage.
- Action: SBAs check — creature has lethal damage, would be destroyed
- Expected Result: Instead, all damage is removed from the creature and the Aura with umbra armor is destroyed. Creature survives.
- Phase: Phase 6 (replacement) + Phase 8
- Ticket: (same as 702.89a)
```

---

### 702.90 — Infect: note Wither relationship

> Note this is Wither + an adjacent effect for dependency graph

**AGREE.** Tag:

**Tag: SHARED-BEHAVIOR wither** — Infect (702.90) = Wither (702.80, covered in Session 7B) + "damage to players = poison counters." For the dependency graph, infect's creature-damage behavior should reuse the wither implementation (deal damage as -1/-1 counters). The poison-counter-to-player logic is the only new piece.

---

### 702.93 — Undying: dual of Persist

> This is the dual of Persist

**AGREE.** Tag:

**Tag: SHARED-BEHAVIOR persist** — Undying (702.93) and Persist (702.79, Session 7B) are structural duals: Undying checks no +1/+1 counters → returns with +1/+1. Persist checks no -1/-1 counters → returns with -1/-1. Implementation should share a `dies_return_with_counter(counter_type, absence_check)` helper.

---

### 702.95a — Soulbond: pairing on later creature entry

> Should also test "if no other creatures when Soulbond creature enters, it pairs with the next creature that enters under controller's control"

**AGREE.** This is actually the *second* trigger of soulbond — 702.95a defines two triggers: one when the soulbond creature enters, and one when another creature enters under the same controller (the soulbond creature can choose to pair with it).

**New test:**

```
ATOM-702.95a-002
- Rule: 702.95a (second trigger) — Soulbond: another creature enters, unpaired soulbond creature may pair with it
- Mechanism: ETB trigger on OTHER creatures entering (soulbond creature's perspective)
- Minimal Board: Player controls unpaired creature A with soulbond (no other creatures). Player casts creature B.
- Action: Creature B enters; soulbond's second trigger fires on creature A
- Expected Result: Controller may pair A and B. If they do, both become paired.
- Phase: Phase 7 + Phase 8
- Ticket: (same as 702.95a)
```

---

### 702.95e — Soulbond: multi-clause unpairing conditions

> Should test each condition here--multiclause

**AGREE.** 702.95e lists several unpair conditions. Current test only covers "partner leaves battlefield." Need tests for:

```
ATOM-702.95e-002
- Rule: 702.95e — Soulbond: creature stops being a creature → unpaired
- Mechanism: Pairing dissolution on type loss
- Minimal Board: A and B are paired. Effect turns B into a noncreature artifact.
- Action: B loses creature type
- Expected Result: A and B become unpaired.
- Phase: Phase 5 + Phase 7 + Phase 8

ATOM-702.95e-003
- Rule: 702.95e — Soulbond: creature changes controller → unpaired
- Mechanism: Pairing dissolution on controller change
- Minimal Board: A and B are paired under player 1. Opponent gains control of B.
- Action: B's controller changes to player 2
- Expected Result: A and B become unpaired.
- Phase: Phase 5 + Phase 7 + Phase 8
```

---

### 702.96 — Overload: text-changing effect tag

> Should tag for when we deal with text-changing effect

**AGREE.** Tag:

**Tag: TEXT-CHANGING-EFFECT** — Overload (702.96), Splice (702.47 from Session 7B), and Cleave (702.148) all involve text-changing effects. While they map more naturally to other CR concepts (Overload → "each" targeting, Splice → ability granting, Cleave → text removal), the engine's text-changing infrastructure must handle all three. Consider a shared `TextModification` layer that these keywords feed into. Cross-ref rule 612 (text-changing effects).

---

### 702.98 — Unleash: counter from other source still prevents blocking

> Should also test the second static ability separately

**AGREE.** The CR says: "A creature with unleash can't block as long as it has a +1/+1 counter on it." This is a *separate* static ability from the optional ETB counter.

**New test:**

```
ATOM-702.98a-002
- Rule: 702.98a (second static) — Unleash: can't block with +1/+1 counter from ANY source
- Mechanism: Static blocking restriction conditional on counter presence
- Minimal Board: Player casts creature with unleash, chooses NOT to put the +1/+1 counter on it. Later, another effect puts a +1/+1 counter on it.
- Action: Creature now has a +1/+1 counter (from external source). Player tries to block with it.
- Expected Result: Block is illegal. The "can't block" restriction checks for ANY +1/+1 counter, not just the unleash one.
- Phase: Phase 5 + Phase 8
- Ticket: (same as 702.98a)
```

---

### 702.99 — Cipher: copies can't be ciphered

> Should test that copies of cipher spells can't be embedded

**AGREE.**

**New test:**

```
ATOM-702.99a-003
- Rule: 702.99a — Cipher: a copy of a cipher spell does not get encoded
- Mechanism: Copy exclusion from cipher encoding
- Minimal Board: Player casts a spell with cipher, it resolves and is encoded on creature A. Later, creature A deals combat damage, creating a copy of the encoded spell.
- Action: The copy resolves
- Expected Result: The copy is NOT encoded on any creature after resolving. Only the original card (already encoded) remains encoded. Copies cease to exist after resolving.
- Phase: Phase 8
- Ticket: (same as 702.99a)
```

---

### 702.100b — Evolve: "when a creature evolves" trigger

> Should test triggered abilities that happen when a permanent "evolves"

**AGREE.** 702.100b defines what "evolves" means for linked triggered abilities.

**New test:**

```
ATOM-702.100b-001
- Rule: 702.100b — "A creature evolves" = its evolve trigger resolves and a counter is placed
- Mechanism: Linked trigger definition (evolve event)
- Minimal Board: Creature A with evolve and "Whenever this creature evolves, draw a card." A is 1/1. Player casts a 3/3.
- Action: Evolve triggers (3/3 has greater power). Counter placed on A. "Evolves" event fires.
- Expected Result: Player draws a card (the "whenever evolves" trigger fires because the counter was actually placed).
- Phase: Phase 7 + Phase 8
- Ticket: (same as 702.100a)
- Dependencies: Phase 7 (triggered), LINKED-ABILITY-PATTERN

ATOM-702.100b-002
- Rule: 702.100b — Evolve trigger resolves but counter NOT placed (e.g., entering creature left) → doesn't "evolve"
- Minimal Board: Creature A with evolve and "Whenever evolves, draw a card." A is 1/1. A 3/3 enters, evolve triggers, but 3/3 is destroyed before trigger resolves.
- Action: Evolve trigger resolves but conditions no longer met (or if it still puts the counter: depends on re-check). Actually per CR, evolve re-checks on resolution — if entering creature left, the comparison can't be made, so no counter.
- Expected Result: No counter placed → creature did NOT "evolve" → no card drawn.
- Phase: Phase 7 + Phase 8
```

---

### 702.104b — Tribute: "if tribute wasn't paid" triggers

> The whole point of tribute is that they all have these abilities, need to test here.

**AGREE.** Tribute without its "wasn't paid" trigger is meaningless — the opponent would never pay.

**New test:**

```
ATOM-702.104b-001
- Rule: 702.104b — Tribute not paid → "if tribute wasn't paid" triggered ability fires
- Mechanism: Conditional triggered ability linked to tribute payment
- Minimal Board: Player casts a 2/2 creature with "Tribute 3" and "When this creature enters, if tribute wasn't paid, it deals 3 damage to target player."
- Action: Chosen opponent declines to pay tribute (creature enters as 2/2, no extra counters)
- Expected Result: "If tribute wasn't paid" trigger fires → 3 damage to target player.
- Phase: Phase 7 + Phase 8
- Ticket: (same as 702.104a)

ATOM-702.104b-002
- Rule: 702.104b — Tribute paid → creature enters with counters, "wasn't paid" trigger does NOT fire
- Minimal Board: Same setup as above.
- Action: Chosen opponent PAYS tribute → creature enters as 5/5 (2/2 + 3 counters)
- Expected Result: "If tribute wasn't paid" trigger does NOT fire. No damage dealt.
- Phase: Phase 7 + Phase 8
```

**Tag: LINKED-ABILITY-PATTERN** — Same as Devour/Exploit. Tribute stores a "was paid" boolean that linked abilities reference.

---

### 702.110b — Exploit: linked "exploited creature" abilities

> Similar to 702.104b, every creature with Exploit has an ability that references the creature it exploited

**AGREE.**

**New test:**

```
ATOM-702.110b-001
- Rule: 702.110b — Exploit: "the creature it exploited" linked ability
- Mechanism: Linked ability referencing the sacrificed creature
- Minimal Board: Player controls a 2/2 with exploit and "When this creature exploits a creature, draw cards equal to the exploited creature's toughness." Also controls a 1/4.
- Action: Exploit triggers, player sacrifices the 1/4.
- Expected Result: "Exploits a creature" trigger fires. Player draws 4 cards (exploited creature's toughness was 4).
- Phase: Phase 7 + Phase 8
- Ticket: (same as 702.110a)
- Dependencies: LINKED-ABILITY-PATTERN (exploit stores reference to sacrificed creature)
```

---

### 702.111 — Menace: audit current implementation

> Investigate current menace implementation and consider if anything should/must be altered given official rules. Nothing in current impl is sacred.

**Finding:** Menace is in the `KeywordAbility` enum (`types/keywords.rs:22`) but **NOT enforced in blocker validation**. `validate_blockers()` in `engine/combat/validation.rs` checks flying/reach and block count limits, but has no menace logic. The `BlockConstraints` doc comment (line 104) mentions "menace adds min-blockers" for Phase 4, but this is unimplemented.

**What the CR says (702.111a):** "A creature with menace can't be blocked except by two or more creatures."

**What needs to change:** `validate_blockers()` needs a post-assignment check: for each attacker with menace, count how many blockers are assigned to it. If blocked at all, must be blocked by ≥ 2 creatures. This is a **set-level constraint** (you can't check it per-blocker).

**Proposed approach:**
1. After all (blocker, attacker) pairs are validated individually, add a menace check pass.
2. For each attacker with `KeywordAbility::Menace`, count `proposed.iter().filter(|(_, a)| a == attacker_id).count()`.
3. If count == 1, return `CombatError::MenaceRequiresTwoBlockers(attacker_id)`.
4. Count == 0 is fine (not blocked).

**Status:** This is a Phase 4 item that should be implemented now since the keyword enum variant already exists and the validation infrastructure supports it. Recommend creating a ticket.

**Action:** Add a note to session-8.md 702.111 entry. Create a separate implementation ticket.

---

### 702.113 — Awaken: original spell effects still apply

> Should also test that the spell's original effects work when you cast with the awaken mode

**AGREE.** Awaken is an additional cost — the spell still does its normal thing plus animates a land.

**New test / note:**

```
ATOM-702.113a-002
- Rule: 702.113a — Awaken: spell's original effects still apply when awaken cost is paid
- Mechanism: Additional cost does not replace spell effects
- Minimal Board: Player casts "Sheer Drop" ({2}{W} destroy target tapped creature, Awaken 3 — {5}{W}) for awaken cost, targeting tapped creature and own land.
- Action: Spell resolves
- Expected Result: Tapped creature is destroyed (original effect). Land gets 3 +1/+1 counters and becomes a 0/0 Elemental creature (awaken effect). Both effects apply.
- Phase: Phase 8
- Ticket: (same as 702.113a)
- Note: Awaken is an additional cost + additional effect, not a replacement. Handled by T17 additional cost infrastructure.
```

---

### 702.114 — Devoid: colorless card findable by "search for colorless"

> Should have a test like "Search your library for a colorless card" being able to find a devoid card with colored mana symbols in its cost

**AGREE.** This is the key architectural consequence of devoid.

**New test:**

```
ATOM-702.114a-002
- Rule: 702.114a — Devoid: card is colorless despite colored mana symbols
- Mechanism: Characteristic-defining effect (color) vs mana cost color indicators
- Minimal Board: Player's library contains a card with "Devoid" and mana cost {1}{R}{G}. An effect says "Search your library for a colorless card."
- Action: Player searches library
- Expected Result: The devoid card is a legal find (it's colorless). Despite having {R}{G} in its mana cost, devoid overrides the color-from-mana-cost rule.
- Phase: Phase 5 (layer 5 — color CDA)
- Ticket: (same as 702.114a)
```

---

### 702.119 — Improvise: only reduces generic cost

> Should also test that this *only* reduces colorless cost, no colored mana reduction

**AGREE.**

**New test:**

```
ATOM-702.119a-002
- Rule: 702.119a — Improvise: tapping artifacts only pays generic mana, not colored
- Mechanism: Cost reduction restriction (generic only)
- Minimal Board: Player casts a spell with "Improvise" costing {2}{U}{U}. Controls 4 untapped artifacts.
- Action: Player taps all 4 artifacts
- Expected Result: Only {2} generic is paid by tapping artifacts (maximum generic portion). Player must still pay {U}{U} from mana pool. Tapping extra artifacts beyond the generic portion has no effect.
- Phase: Phase 5-Pre T17 + Phase 8
- Ticket: (same as 702.119a)
- Tag: SHARED-BEHAVIOR convoke — Same "tap to pay generic" pattern.
```

---

### 702.122 — Crew: non-Vehicle permanents with crew ability

> We should test that a crew ability can be used even by non-Vehicle permanents (e.g. through ability copying effects like Myr Welder)

**AGREE.** Crew's definition doesn't restrict itself to Vehicles — it just says "this permanent becomes an artifact creature."

**New test:**

```
ATOM-702.122-003
- Rule: 702.122a — Crew on a non-Vehicle permanent (via ability copying)
- Mechanism: Crew ability functions on any permanent, not just Vehicles
- Minimal Board: A non-Vehicle artifact has gained "Crew 3" through an ability-copying effect. Player controls a 3/3 creature.
- Action: Tap the 3/3 to crew the artifact
- Expected Result: The artifact becomes an artifact creature until end of turn (even though it's not a Vehicle).
- Phase: Phase 8
- Ticket: (same as 702.122a)
```

---

### 702.122b — Crew: "whenever this creature crews a Vehicle" triggers

> Some effects relate to creatures crewing vehicles

**AGREE.** This is a significant interaction space.

**New test:**

```
ATOM-702.122b-001
- Rule: 702.122b — "Whenever this creature crews a Vehicle" trigger
- Mechanism: Crew event triggers linked abilities
- Minimal Board: Player controls creature A with "This creature crews Vehicles as though its power were 2 greater" (power 1 → effective 3 for crewing) and creature B with "Whenever this creature crews a Vehicle, that Vehicle gains flying until end of turn." Vehicle with Crew 3.
- Action: Tap creature A and creature B to crew the Vehicle (effective total power: 3+1 = 4 ≥ 3)
- Expected Result: Vehicle becomes artifact creature. B's trigger fires → Vehicle gains flying until EOT. A's power-boost is accounted for in the crew cost check.
- Phase: Phase 7 + Phase 8
- Ticket: (same as 702.122a)
- Tag: CREW-EVENT — Engine needs a "creature crewed a Vehicle" event for triggers.
```

---

### 702.126 — Improvise: shared code with Convoke

> Likely shared code with Convoke (similar abilities, just varies on what you can tap down and Convoke lets you pay colored costs)

**AGREE.** Tag:

**Tag: SHARED-BEHAVIOR convoke-improvise** — Convoke (702.50, Session 7B) and Improvise (702.119) share a "tap permanents to reduce cost" pattern. Key differences:
- **Convoke:** tap creatures → pay {1} OR one mana of creature's color per creature
- **Improvise:** tap artifacts → pay {1} per artifact (generic only)

Implementation should share a `tap_to_reduce_cost(permanent_filter, reduction_type)` helper. `reduction_type` is either `GenericOnly` (Improvise) or `GenericOrColor` (Convoke).

---

### 702.131 — Ascend: static ability architecture concern + rulings

> Should test that this is a state ability [...] Consider an effect that says "Each player creates a 1/1 white Human creature token. Then destroy all creatures."

**AGREE — this is a significant architectural concern.** The key insight from the rulings:

1. **Ascend on a permanent is a static ability, not triggered.** It doesn't use the stack. You get city's blessing the instant you control 10+ permanents.
2. **Mid-resolution acquisition:** If you gain your 10th permanent during spell resolution (e.g., token creation), you get city's blessing immediately — before the spell finishes resolving.
3. **Permanence:** Once acquired, city's blessing can never be lost, even if you drop below 10 permanents.
4. **SBA-adjacent timing:** 702.131d says continuous effects reapply after acquiring city's blessing, before trigger checks.

**Architecture implication:** The engine needs a "check designations" pass that runs:
- After each game action that changes permanent count
- During spell resolution (between sequential effects within a single spell)
- This is similar to how SBAs work but must run mid-resolution

**New tests:**

```
ATOM-702.131a-001 (instant/sorcery version - already exists, keep as-is)

ATOM-702.131b-001
- Rule: 702.131b — Ascend on permanent: get city's blessing immediately when controlling 10+ permanents
- Mechanism: Static ability, no stack, immediate designation
- Minimal Board: Player controls 9 permanents including one with ascend. Player plays a land (10th permanent).
- Action: Land enters the battlefield
- Expected Result: Player gets city's blessing immediately. No trigger, no stack. Opponents cannot respond between land entry and blessing acquisition.
- Phase: Phase 8
- Ticket: NEW — Ascend keyword (static designation, permanence)
- Dependencies: D17 (designations), SBA-adjacent timing

ATOM-702.131b-002
- Rule: 702.131b — Ascend: mid-resolution acquisition
- Mechanism: Static ability checked during spell resolution
- Minimal Board: Player controls 9 permanents (one with ascend). Player casts a spell that says "Each player creates a 1/1 white Human creature token. Then destroy all creatures." Player also has a creature with "As long as you have the city's blessing, this creature has indestructible."
- Action: Spell resolves: token created (player now controls 10 permanents) → city's blessing acquired → "destroy all creatures" effect happens
- Expected Result: Player's creature with indestructible-if-blessed survives the destroy effect because city's blessing was acquired mid-resolution before the destroy clause.
- Phase: Phase 8
- ARCHITECTURAL NOTE: Requires mid-resolution static ability checks.

ATOM-702.131c-001
- Rule: 702.131c — City's blessing persists even below 10 permanents
- Minimal Board: Player has city's blessing, controls 10 permanents. 5 permanents are destroyed.
- Expected Result: Player still has city's blessing (5 permanents remaining).
```

**Tag: ARCHITECTURE-CONCERN mid-resolution-static-checks** — Ascend requires the engine to check static abilities (specifically ascend) between sequential effects within a single resolving spell. This is NOT the same as SBAs (which run between spells/abilities resolving). Needs design discussion.

---

### 702.132 — Assist: "any other player," not just allies

> Doesn't need to choose an "ally", can choose any other player.

**AGREE.** Will fix the test description. CR 702.132a says: "another player may pay up to X of that cost." In multiplayer, that's ANY player, not just opponents or allies.

---

### 702.133 — Jump-Start: not exiled if cast from hand

> Should test that it doesn't get exiled if it's cast from hand.

**AGREE.** The exile-instead replacement only applies to the jump-start casting.

**New test:**

```
ATOM-702.133a-002
- Rule: 702.133a — Jump-Start: cast from hand → goes to graveyard normally (no exile)
- Mechanism: Exile replacement only applies when cast via jump-start
- Minimal Board: Player casts a spell with jump-start from hand normally (not from GY)
- Action: Spell resolves
- Expected Result: Spell goes to graveyard as normal. NOT exiled (the exile clause only applies to jump-start casts).
- Phase: Phase 8
- Ticket: (same as 702.133a)
```

---

### 702.134 — Mentor: no applicable target → trigger behavior

> Should we test that if there's no applicable target the ability doesn't even trigger? Or is that handled with triggered abilities?

**NOTE.** This is handled by the general triggered ability infrastructure — rule 603.3c says triggered abilities with targets that can't be fulfilled don't go on the stack. This is NOT mentor-specific. Session 9A (triggered ability rules) will cover this. Mentor's test should note this dependency but doesn't need a mentor-specific test for it.

**Tag: TRIGGERED-ABILITY-INFRA 603.3c** — "No legal targets → trigger doesn't go on stack" is a general rule, not keyword-specific. Applies to Mentor, Backup, Fabricate, etc.

---

### 702.137 — Spectacle: negative case (no life lost)

> Should we also test the negative case (no life lost, can't cast for Spectacle)?

**AGREE.**

**New test:**

```
ATOM-702.137a-002
- Rule: 702.137a — Spectacle: can't use if no opponent lost life this turn
- Mechanism: Condition check failure
- Minimal Board: No opponent has lost life this turn. Player has a spell with "Spectacle {B}" (normal cost {2}{B}).
- Action: Player attempts to cast for spectacle cost
- Expected Result: Cast is illegal. Spectacle requires an opponent to have lost life this turn.
- Phase: Phase 8
- Ticket: (same as 702.137a)
- Dependencies: D16 (per-turn life-loss tracker)
```

---

### 702.143c — Foretell: cost reduction / timing modification

> How are we handling an ability like "Foretelling cards from your hand costs {1} less and can be done on any player's turn."

**DISCUSS.** This is an effect that modifies the foretell special action itself (rule 702.143a defines foretell as paying {2} to exile face-down). An effect that reduces the {2} cost or removes the "only on your turn" restriction modifies the parameters of a special action.

**Engine approach options:**
1. **Parameterized special action:** Foretell's cost and timing are stored as modifiable fields. Continuous effects can alter them (like cost reduction and timing windows).
2. **Continuous effect hooks on special actions:** The special action checks for active continuous effects that modify its parameters before executing.

Option 2 is more general-purpose and aligns with how we handle cost modifications for spells (T17). Recommend extending the cost modification pipeline to cover special actions.

**Tag: ARCHITECTURE-CONCERN special-action-modification** — Foretell cost/timing modification requires the cost modification pipeline (T17) to extend to special actions, not just spell casting. Also affects Plot (702.170) which is also a special action.

---

### 702.148 — Cleave: similar to Overload

> Very similar to Overload (text-changing effect that functions as a distinct but similar spell)

**AGREE.** Tag:

**Tag: TEXT-CHANGING-EFFECT cleave** — Cleave (702.148) removes bracketed text when the cleave cost is paid. Same text-changing-effect infrastructure as Overload (702.96) and Splice (702.47). Already tagged under 702.96 note above. Cross-ref rule 612.

---

### 702.155 — Read Ahead: counter doubler interaction

> Good test is counter doubler with read ahead saga

**AGREE.** This is a very precise and important interaction.

**New test:**

```
ATOM-702.155-002
- Rule: 702.155 + ruling — Read Ahead: counter doubler modifies actual entry counters
- Mechanism: Replacement effect modifies lore counter count after chapter choice
- Minimal Board: Player controls a permanent with "If a permanent would enter with one or more counters, it enters with twice that many instead." Player casts a Read Ahead Saga, choosing chapter I (1 lore counter).
- Action: Saga enters
- Expected Result: Player chose 1 lore counter, but replacement doubles it to 2. Saga enters with 2 lore counters. ONLY the chapter II ability triggers (matching actual counter count), NOT chapters I and II. The "all chapters up to chosen" Read Ahead rule is bypassed because the replacement changed the actual count.
- Phase: Phase 6 (replacement) + Phase 8
- Ticket: (same as 702.155a)
- Note: This differs from normal saga ETB (which would trigger chapters I and II for 2 counters). Read Ahead specifically triggers only the chapter matching the actual counter count.
```

---

### 702.160 — Prototype: additional scrutiny

> Exotic ability, we should think about the plan for handling it with additional scrutiny

**DISCUSS.** Prototype is unique because it's NOT a DFC but has zone-dependent characteristics:
- On stack and battlefield (if cast for prototype cost): uses prototype P/T, color, mana cost
- Everywhere else (hand, GY, library, exile): uses normal characteristics

**Architecture concern:** This is the ONLY non-DFC keyword that changes base characteristics based on how it was cast. The layer system needs to handle "this permanent's base characteristics were modified at cast time" as a copiable value (702.160b).

**Proposed handling:**
- `GameObject` gets a `cast_mode: Option<CastMode>` field
- `CastMode::Prototype { power, toughness, mana_cost, color }` stores the overrides
- `characteristics.rs::get_effective_*` checks cast_mode when the object is on stack/battlefield
- For zones other than stack/battlefield, cast_mode is ignored

This is simpler than DFCs (no face-flipping, no transform triggers) but shares the "base characteristics depend on context" pattern. Could be a stepping stone toward DFC support.

**Action:** Add a design note to Phase 5 plan. Prototype could be implemented in Phase 5 alongside the layer system rather than waiting for Phase 9.

---

### 702.165 — Backup: "subsequent abilities only" + Death-Greeter's Champion

> A weird note with this ability is that only the *subsequent* abilities on the card.

**DISCUSS.** The CR is explicit: 702.165a says "non-backup abilities of this creature **printed below this one**" and 702.165c says "Only abilities **printed** on the object with backup are granted."

For Death-Greeter's Champion:
```
Dash {3}{R}
Backup 1
Double strike
```
Backup grants **only** double strike (printed after backup), NOT dash (printed before backup).

**Engine consideration:** The engine needs to know the **print order** of abilities on a card. Currently `CardData` likely stores abilities as a `Vec<Ability>` — the order in that vec IS the print order as long as we maintain it during card construction.

**Implementation:**
1. `backup_grants_from_index: Option<usize>` on the ability — index into the abilities vec marking where backup's "below this" starts.
2. When backup resolves, grant abilities from `[backup_index+1..]`.

**Re: "will they change this rule?"** — Unlikely soon. The rule is clunky but precise. MaRo has said backup was designed so that the last ability on the card is the "shared" one, and pre-backup abilities are intentionally not shared. I'd implement as-written.

---

### 702.165c, d — Backup: caveats

> These are important caveats we should test

**AGREE.**

**New tests:**

```
ATOM-702.165c-001
- Rule: 702.165c — Backup: only printed abilities are granted, not gained ones
- Mechanism: Granted abilities vs printed abilities distinction
- Minimal Board: Creature A with "Backup 1" and printed "Flying". A also has been granted "Trample" by a continuous effect. Creature B enters.
- Action: Backup triggers, targeting B
- Expected Result: B gets +1/+1 counter AND flying (printed after backup) until EOT. B does NOT get trample (granted, not printed).
- Phase: Phase 7 + Phase 8

ATOM-702.165d-001
- Rule: 702.165d — Backup: granted abilities determined when trigger goes on stack, not on resolution
- Mechanism: Lock-in at trigger time
- Minimal Board: Creature A has "Backup 1" with printed "Flying" and "Deathtouch". A enters, backup triggers, goes on stack. In response, A loses deathtouch.
- Action: Backup trigger resolves targeting B
- Expected Result: B gets +1/+1 counter AND both flying and deathtouch until EOT. Deathtouch was locked in when the trigger went on the stack.
- Phase: Phase 7 + Phase 8
```

---

### 702.167c — Craft: testing plan

> Should probably test this, or at least have a plan to

**AGREE.** 702.167c likely covers "what happens to the crafted card" — it transforms. Given that Craft is Phase 9 (DFC), the test is deferred but should be explicitly listed.

**Tag: DEFERRED-TEST craft-transform** — 702.167c test deferred to Phase 9 DFC implementation. Will be testable once transform infrastructure exists.

---

### 702.168 — Disguise: shared code with Morph

> Almost a Morph clone, lots of shared code dna between these two abilities

**AGREE.** Tag:

**Tag: SHARED-BEHAVIOR morph-disguise** — Disguise (702.168) and Morph (702.37, Session 7B) share:
- Face-down casting as 2/2 for {3}
- Turn face-up as special action
- Rule 708 (face-down spells) infrastructure

Differences:
- Disguise grants ward {2} while face-down (Morph doesn't)
- Different keyword names (matters for "cards with disguise" effects)

Implementation should share a `FaceDownCasting` module. Disguise adds a ward hook.

---

### 702.170d-f — Plot: testable sub-rules

> All feel testable

**AGREE.** Will add tests for these. Let me check what they say.

**Tag: ADD-TESTS 702.170d-f** — Will generate these during fold-back pass. Need to read CR text for 702.170d-f specifically.

---

### 702.171 — Saddle: functionally similar to Crew for creatures

> Functionally identical to Vehicles but applies to creatures instead

**AGREE.** Tag:

**Tag: SHARED-BEHAVIOR crew-saddle** — Saddle (702.171) and Crew (702.122) share:
- "Tap creatures with total power N+" as a cost
- Designation result (crewed / saddled)

Differences:
- Crew → permanent becomes artifact creature until EOT
- Saddle → creature becomes "saddled" (designation only, no type change)
- Saddle is on creatures (Mounts), not artifacts (Vehicles)

Implementation should share a `TapCreaturesForPower` cost type.

---

### 702.172 — Spree: must pick at least one mode

> Should test that you're required to pick at least one cost

**AGREE.**

**New test:**

```
ATOM-702.172a-002
- Rule: 702.172 — Spree: must choose at least one mode (and pay its cost)
- Mechanism: Modal casting constraint (minimum 1 mode)
- Minimal Board: Player casts a spell with spree modes.
- Action: Player attempts to cast without choosing any modes
- Expected Result: Illegal. At least one mode must be chosen.
- Phase: Phase 8
- Ticket: (same as 702.172a)
```

---

### 702.173 — Freerunning: Commander component

> Has a Commander component that should at least be noted

**AGREE.** 702.173 likely has a clause about commanders dealing damage enabling freerunning. Tag:

**Tag: COMMANDER-INTERACTION freerunning** — Freerunning's condition includes "a commander you control dealt combat damage to a player this turn." This is relevant for Phase 9 (Commander). Note in session-8.md entry.

---

### 702.174 — Gift: needs another pass

> Needs another pass, doesn't seem right

**AGREE.** The current entry oversimplifies Gift. Looking at the actual CR text:

702.174a defines Gift as TWO abilities:
1. Static on stack: "As an additional cost, you may **choose an opponent**"
2. Either a static (instants/sorceries) or triggered (permanents) ability that gives the chosen opponent a specific benefit

The current test says "Gift a card (opponent draws a card as additional cost)" — but the additional cost is choosing an opponent, not the gift effect itself. The gift effect happens on resolution (for instants/sorceries) or on ETB (for permanents).

Also missing: 702.174c (triggers when a player "gives a gift"), 702.174d-i (specific gift definitions), 702.174j (gift effect happens before other spell effects on instants/sorceries), 702.174k (gift "promised" definition), 702.174m (target selection conditional on gift promise).

**Full rewrite needed.** Will include in fold-back pass.

---

### 702.177 — Exhaust: reset on zone change

> Need to check this gets reset if the object leaves and re-enters the battlefield

**AGREE.** This is standard game object identity — when a permanent leaves and re-enters, it's a new game object. The "activate only once" counter resets.

**New test:**

```
ATOM-702.177a-002
- Rule: 702.177a — Exhaust: resets on zone change (new game object)
- Mechanism: Game object identity reset
- Minimal Board: Permanent with exhaust ability has been activated once (exhausted). It's bounced to hand, then recast.
- Action: Player attempts to activate the exhaust ability on the new permanent
- Expected Result: Activation is legal. The new permanent is a different game object — "activate only once" starts fresh.
- Phase: Phase 8
- Ticket: (same as 702.177a)
```

---

### 702.182 — Job Select: shared code with For Mirrodin!

> Can share almost all its code with For Mirrodin

**AGREE.** Tag:

**Tag: SHARED-BEHAVIOR for-mirrodin-job-select** — For Mirrodin! (702.163) and Job Select (702.182) both:
- ETB trigger creates a creature token
- Auto-attaches the Equipment to it

Differences:
- For Mirrodin!: 2/2 red Rebel token
- Job Select: 1/1 colorless Hero token

Implementation: shared `etb_create_token_and_attach(token_spec)` helper.

---

### 702.186 — ∞: test harness acquisition

> Should test an ability like "{5}{B}: Harness this artifact" so we can test the designation-acquisition process

**AGREE.** The ∞ test should include the harness action, not just assume the permanent is already harnessed.

**New test:**

```
ATOM-702.186-002
- Rule: 702.186b + 701.64 — ∞: harness activation → gains ∞ ability
- Mechanism: Activated ability (harness) → designation change → static ability activation
- Minimal Board: Player controls an Infinity artifact with "{5}{B}: Harness this artifact" and "∞ — This artifact is a 5/5 creature." Artifact is NOT harnessed.
- Action: Player activates harness ability, paying {5}{B}
- Expected Result: Artifact becomes harnessed (designation). ∞ ability activates → artifact is now a 5/5 creature.
- Phase: Phase 8
- Ticket: (same as 702.186b)
- Dependencies: Harness (701.64), D17 (designations)
```

---

### 702.187 — Mayhem: new game object after GY re-entry

> We should test that once you cast a spell from the graveyard with mayhem, even if it ends back up in the graveyard that same turn, if it wasn't discarded to get back to the graveyard you can't use the mayhem ability again

**AGREE.** This is a new-game-object + discard-tracker interaction.

**New test:**

```
ATOM-702.187b-003
- Rule: 702.187b — Mayhem: cast from GY, dies, returns to GY → can't use mayhem again (new object, not discarded)
- Mechanism: Game object identity + discard designation
- Minimal Board: Player discards a creature with "Mayhem {1}{R}" this turn. Casts it from GY via mayhem. Creature enters, then is destroyed (goes back to GY).
- Action: Player attempts to cast from GY via mayhem again
- Expected Result: Illegal. The card in the graveyard is a new game object. It was not *discarded* to the graveyard this time — it was destroyed. Mayhem condition not met.
- Phase: Phase 8
- Ticket: (same as 702.187b)
- Note: The delta log must track "was discarded" per game-object identity, not per card. When the card re-enters GY via destruction, it's a new object without the "discarded" tag.
```

---

### 702.190 — Sneak: shared logic with Ninjutsu

> This is essentially a "fixed" ninjutsu. Lots of shared logic between them

**AGREE.** Tag:

**Tag: SHARED-BEHAVIOR ninjutsu-sneak** — Ninjutsu (702.49, Session 7B) and Sneak (702.190) share:
- Timing during combat (declare blockers step)
- Return an attacking creature to hand as part of cost
- New creature enters tapped and attacking the same target

Differences:
- Ninjutsu: return an **unblocked** attacking creature (hand activation)
- Sneak: return an **unblocked** creature you control (stack alt cost)
- Ninjutsu is an activated ability from hand; Sneak is an alternative cost for casting

Implementation: shared `combat_swap_creature(source_creature, new_creature, attack_target)` helper for the "enters tapped and attacking the same target" pattern.

---

## Summary of Actions

### Tags Created (for dependency graph)
| Tag | Keywords | Description |
|-----|----------|-------------|
| LINKED-ABILITY-PATTERN | Devour, Tribute, Exploit, Evolve | ETB/replacement stores value → linked trigger references it |
| DECLARED-VS-ENTERS-ATTACKING | Exalted, Melee | Engine must track how a creature became attacking |
| SHARED-BEHAVIOR wither | Infect, Wither | Infect reuses wither for creature damage |
| SHARED-BEHAVIOR persist | Undying, Persist | Structural duals, shared helper |
| SHARED-BEHAVIOR convoke-improvise | Convoke, Improvise | Tap permanents to reduce cost |
| SHARED-BEHAVIOR crew-saddle | Crew, Saddle | Tap creatures for power |
| SHARED-BEHAVIOR morph-disguise | Morph, Disguise | Face-down casting + turn face-up |
| SHARED-BEHAVIOR ninjutsu-sneak | Ninjutsu, Sneak | Combat creature swap |
| SHARED-BEHAVIOR for-mirrodin-job-select | For Mirrodin!, Job Select | ETB token + auto-attach |
| TEXT-CHANGING-EFFECT | Overload, Splice, Cleave | Shared text modification infrastructure |
| CREW-EVENT | Crew | Engine needs crew event for triggers |
| ARCHITECTURE-CONCERN mid-resolution-static | Ascend | Static abilities checked mid-spell-resolution |
| ARCHITECTURE-CONCERN special-action-mod | Foretell, Plot | Cost mod pipeline extends to special actions |
| TRIGGERED-ABILITY-INFRA 603.3c | Mentor, Backup, etc. | No legal targets → trigger doesn't go on stack |
| COMMANDER-INTERACTION | Freerunning | Commander damage enables freerunning |

### New Tests to Add (~30)
702.82b-001, 702.83a-002, 702.84a-002, 702.85a-003, 702.89a-002, 702.95a-002, 702.95e-002/003, 702.98a-002, 702.99a-003, 702.100b-001/002, 702.104b-001/002, 702.110b-001, 702.113a-002, 702.114a-002, 702.119a-002, 702.122-003, 702.122b-001, 702.131b-001/002, 702.131c-001, 702.133a-002, 702.137a-002, 702.155-002, 702.165c-001, 702.165d-001, 702.172a-002, 702.177a-002, 702.186-002, 702.187b-003, 702.190 (already covered)

### Items Needing Discussion Before Fold-Back
1. **DFC stub in Phase 5** — Face abstraction on data model
2. **Prototype in Phase 5 vs Phase 9** — Could be a stepping stone
3. **702.174 Gift full rewrite** — Current entry is oversimplified
4. **702.165 Backup print-order** — Implement as-written?
5. **702.131 Ascend mid-resolution checks** — Architecture concern
6. **702.143c Foretell special-action modification** — T17 extension scope
7. **702.160 Prototype architecture** — cast_mode approach

### Implementation Tickets Identified
1. **Menace enforcement in validate_blockers** — Phase 4, should do now
2. **Crew event system** — For "whenever this creature crews" triggers

---

## Addendum: Deep-Dive Discussions (Round 1.5)

### 702.111 — Refined Evasion Framework (META-7B-01 + Menace + Session 8 keywords)

Session 7B proposed META-7B-01: a unified `EvasionRestriction` + `BlockerFilter` enum. With the full keyword context from Session 8, here's a refined version that incorporates **menace** (702.111) and **skulk** (702.116) alongside the Session 7B keywords.

#### The Problem

Current `validate_blockers()` hardcodes flying/reach as a per-pair check (line 323–329 of `validation.rs`). Menace is a **set-level** constraint (count of blockers per attacker), not a per-pair check. The original META-7B-01 proposal only covered per-pair evasion filters, not minimum-blocker constraints.

#### Full Evasion Taxonomy

Evasion abilities fall into two categories:

**Category A: Per-Pair Blocker Filters** — "Can't be blocked except by [filter]"
| Keyword | Filter | CR |
|---------|--------|-----|
| Flying | HasKeyword(Flying) OR HasKeyword(Reach) | 702.9b |
| Shadow | HasKeyword(Shadow) — bidirectional | 702.28b |
| Fear | IsArtifact OR SharesColor | 702.36b |
| Intimidate | IsArtifact OR SharesColor | 702.13b |
| Horsemanship | HasKeyword(Horsemanship) | 702.31b |
| Landwalk(type) | ConditionalUnblockable(DefenderControlsLandType) | 702.14c |
| Skulk | PowerLessOrEqual(this.power) | 702.116b |
| Protection(Q) | NOT MatchesQuality(Q) | 702.16f |

**Category B: Minimum-Blocker Constraints** — "Can't be blocked except by N+ creatures"
| Keyword | Constraint | CR |
|---------|-----------|-----|
| Menace | MinBlockers(2) | 702.111a |

These are fundamentally different: Category A filters apply to each (blocker, attacker) pair independently. Category B constrains the *count* of blockers assigned to a single attacker.

#### Proposed Design

```rust
/// Category A: per-pair filter on individual blockers.
/// Applied during per-pair validation: "can this blocker block this attacker?"
enum BlockerFilter {
    /// Blocker must have at least one of these keywords (Flying: [Flying, Reach])
    HasAnyKeyword(Vec<KeywordAbility>),
    /// Bidirectional: blocker must have the keyword, AND non-keyword creatures
    /// can't block keyword creatures (Shadow)
    Bidirectional(KeywordAbility),
    /// Blocker must be artifact creature or share a color (Fear, Intimidate)
    ArtifactOrSharesColor,
    /// Attacker can't be blocked at all if condition on defending player is met (Landwalk)
    ConditionalUnblockable(LandwalkCondition),
    /// Blocker's power must be ≤ attacker's power (Skulk)
    PowerAtMost(i32),
    /// Blocker must NOT match quality (Protection)
    NotMatchesQuality(ProtectionQuality),
}

/// Category B: set-level constraint on blocker count per attacker.
/// Applied AFTER all pairs are validated: "are enough blockers assigned?"
enum MinBlockerConstraint {
    /// Must be blocked by N or more creatures, or not at all (Menace)
    MinBlockers(usize),
}

/// Combined evasion profile for an attacker.
struct EvasionProfile {
    filters: Vec<BlockerFilter>,       // Category A
    min_blockers: Option<usize>,       // Category B (None = 1, i.e. normal)
}

/// Build evasion profile from an attacker's keywords + continuous effects.
fn get_evasion_profile(game: &GameState, attacker_id: ObjectId) -> EvasionProfile {
    let mut profile = EvasionProfile { filters: vec![], min_blockers: None };

    if has_keyword(game, attacker_id, KeywordAbility::Flying) {
        profile.filters.push(BlockerFilter::HasAnyKeyword(
            vec![KeywordAbility::Flying, KeywordAbility::Reach]
        ));
    }
    if has_keyword(game, attacker_id, KeywordAbility::Menace) {
        profile.min_blockers = Some(
            profile.min_blockers.map_or(2, |n| n.max(2))
        );
    }
    // ... Shadow, Fear, Intimidate, Horsemanship, Skulk, Protection, Landwalk
    // Phase 5 continuous effects would also inject filters here

    profile
}
```

#### Validation Flow

```
validate_blockers():
  1. Per-pair checks (existing):
     - On battlefield, is creature, correct controller, untapped, attacker targeting this player
     - NEW: for each (blocker, attacker) pair, get attacker's EvasionProfile,
       check all BlockerFilters against blocker → CantBlockDueToEvasion error

  2. Per-attacker count check (NEW):
     - For each attacked creature, count assigned blockers
     - If attacker has min_blockers = Some(N) and 0 < count < N → MenaceRequiresNBlockers error
     - (0 is fine — not blocked. N+ is fine — legally blocked.)

  3. Set-level constraints (existing):
     - BlockRestriction::CantBlock, CantBlockUnless
     - BlockRequirement::MustBlockIfAble
```

#### Why Menace Doesn't Fit in BlockerFilter

It's tempting to model menace as a BlockerFilter, but it fundamentally can't be — a single blocker IS capable of being assigned to a menace creature (it passes all per-pair checks). The illegality only manifests when you look at the total assignment: "you assigned only 1 blocker to this menace creature, that's illegal." This is a set-level constraint, not a pair-level filter.

#### Interaction: Menace + Flying

A creature with both menace and flying requires:
- Each blocker passes flying filter (has flying or reach) — Category A
- At least 2 such blockers are assigned — Category B

These compose naturally: the per-pair filter eliminates ineligible blockers, then the count check ensures enough eligible blockers were assigned.

#### When to Build This

The current hardcoded flying check can be migrated to this framework as the **first step** when implementing menace. The migration is:
1. Replace the flying check in `validate_blockers` with `get_evasion_profile()` + filter loop
2. Add menace to `get_evasion_profile()`
3. Add the post-assignment count check

This is a natural Phase 4.5/5-Pre ticket since `KeywordAbility::Menace` already exists in the enum. Each subsequent evasion keyword (Shadow, Fear, Intimidate, etc.) becomes a one-liner addition to `get_evasion_profile()`.

---

### 702.131 — Ascend: Initial Analysis (Superseded by Round 2)

*The initial proposal for `check_state_based_designations()` identified the right problem but left open the question of how the engine detects mid-resolution state. See Round 2 deep-dive below for the full 4-pillar analysis leading to the `execute_action` hook solution.*

**Key stress test preserved:** Skymarcher Aspirant (ascend + "has flying if city's blessing") must survive a boardwipe that first creates tokens (pushing to 10 permanents mid-resolution) then destroys all non-flyers.

---

## Discussion Item Resolutions

### 1. DFC Face Stub in Phase 5

**RESOLVED: Add Face abstraction now.**

Rationale: Cheap to add (enum with single variant for now), prevents hardcoded single-face assumptions in the layer system. Even if the design changes later, a stub is low-risk.

Action: Add `Face` struct/enum to `CardData` in Phase 5. Single-face cards get `faces: vec![FaceData { ... }]`. DFC cards will get two entries. `characteristics.rs` queries `active_face()` → returns `&faces[0]` for now.

### 2. Prototype in Phase 9 (alongside DFCs)

**RESOLVED: Keep Prototype deferred to Phase 9.**

Rationale: Even though the proposed `cast_mode` approach adds a field to `GameObject` itself (not `CardData`), the zone-dependent-characteristics behavior is sufficiently novel that it benefits from having the Face abstraction in place. Prototype's "which characteristics are active depends on zone + cast mode" is a natural extension of "which face is active depends on zone + transform state."

### 3. 702.174 Gift Full Rewrite

**RESOLVED: Proceed with rewrite during fold-back.** No user input needed — the CR text is clear and the current entry is provably wrong.

### 4. 702.165 Backup Print-Order: Vec gives us this for free

**RESOLVED: Implement as-written using Vec index.**

Granted abilities question: Backup 702.165c explicitly says "Only abilities **printed** on the object with backup are granted." So granted abilities (from continuous effects, copy effects, etc.) are excluded by definition. Implementation:
- `CardData.abilities: Vec<Ability>` maintains print order
- `backup_start_index: usize` marks where "below backup" begins
- When backup resolves, grant `abilities[backup_start_index..]` to target
- Abilities added by continuous effects are NOT in `CardData.abilities` — they're in the layer system's effect list, so they're naturally excluded

### 5. 702.131 Ascend Mid-Resolution Checks

**RESOLVED: Proposal A (`execute_action` hook).** Add `self.check_immediate_designations()` call after `self.perform_action()` in `execute_action()`. No resolution flag needed — runs after every mutation, which is exactly what the CR requires. See Round 2 deep-dive for full 4-pillar analysis.

### 6. 702.143c Foretell Special-Action Modification: Option 2

**RESOLVED: Extend T17 cost modification pipeline to special actions.**

The cost modification pipeline already handles spell costs. Special actions (Foretell exile for {2}, Plot exile) have simpler cost structures but should route through the same pipeline for consistency. Continuous effects that modify special action costs/timing will register as cost modifiers with a `CostModifierTarget::SpecialAction(SpecialActionType::Foretell)` discriminant.

### 7. Prototype Architecture

**RESOLVED: Proposal C (`CharacteristicOverrides`).** Perpetual/Alchemy mechanics are confirmed IN-SCOPE, and they require the same "base characteristic overrides" pattern as Prototype. This tips the decision from A (simplest for Prototype alone) to C (handles both Prototype and Perpetual). See Round 2 deep-dive for full analysis and the Perpetual note below.

---

## Addendum: Deep-Dive Discussions (Round 2)

### 702.131 — How Does the Engine Know a Spell Is Resolving?

#### The core tension

The previous proposal for `check_state_based_designations()` said "call it after each atomic game action, including mid-resolution." But the current engine has no concept of "mid-resolution." Let's examine why, and what the options are.

#### Current resolution architecture

```
priority.rs::run_priority_loop()
  → all pass → resolve_top_of_stack()
    → pop spell off stack
    → resolve_effect(effect, ctx)
      → Effect::Sequence([sub1, sub2, sub3])
        → resolve_effect(sub1, ctx)  // e.g. "each player creates a token"
        → resolve_effect(sub2, ctx)  // e.g. "destroy all creatures without flying"
        → resolve_effect(sub3, ctx)
    → post-resolution zone change (spell → GY / permanent → BF)
  → perform_sba_and_triggers()
```

Key observations:
1. `resolve_effect` is a synchronous recursive walk of the `Effect` tree
2. `Effect::Sequence` is a simple `for sub in effects { self.resolve_effect(sub, ctx)?; }` loop
3. There is no flag, counter, or state on `GameState` that indicates "we are mid-resolution"
4. SBAs and trigger checks ONLY run after `resolve_top_of_stack` returns to the priority loop

Ascend breaks assumption #4: it's a static ability, not a trigger, and the CR says it applies immediately — even between clauses of a resolving spell.

#### Why the delta log doesn't solve this

`state-tracking-architecture.md` §603.8 explicitly says: "Single post-resolution pass, not per-sub-action." That works for 603.8 state triggers because triggers use the stack — they're detected after resolution, then placed on the stack for the next round of priority. The momentary state change IS recorded in the delta log, and the trigger IS matched.

But Ascend doesn't use the stack. City's blessing must be **acquired** mid-resolution so that continuous effects reapply and change the game state that subsequent clauses of the same spell observe. A post-resolution scan is too late.

#### The Skymarcher Aspirant stress test, mechanically

```
Resolving: "Each player creates a 1/1 token. Then destroy all creatures without flying."

  1. resolve_effect(CreateToken for each player)
     → execute_action(CreateToken) for Player A
     → Player A now controls 10 permanents
     → ??? Ascend check needed HERE ???
     → City's blessing acquired
     → Continuous effects reapply: Aspirant gains flying

  2. resolve_effect(DestroyAll { filter: !flying })
     → Aspirant has flying → survives
```

If the check runs after step 2 instead of between 1 and 2, Aspirant is destroyed before acquiring flying.

#### Four proposals, evaluated against the 4 pillars

##### Proposal A: `execute_action` hook (recommended)

**Mechanism:** `execute_action()` is already the single chokepoint for all state mutations (designed for Phase 6 replacement effects). Add a post-mutation call:

```rust
pub fn execute_action(&mut self, action: GameAction) -> Result<(), String> {
    // Phase 6: let action = self.apply_replacement_effects(action)?;
    self.perform_action(action)?;
    self.check_immediate_designations(); // NEW
    Ok(())
}
```

`check_immediate_designations()` only handles designations that the CR says apply immediately without the stack. Currently that's just Ascend. The function is a no-op once city's blessing is acquired for all players.

| Pillar | Rating | Notes |
|--------|--------|-------|
| **Speed** | ✅ Good | Early-exit: `if all_players_have_blessing { return; }`. Once acquired, zero cost forever. Before acquisition: O(battlefield_size) count, but only fires after mutations — not on reads. The existing `execute_action` call overhead is already paid. |
| **Correctness** | ✅ Correct | Designation is checked after every state mutation, exactly per CR. No timing gap. Works mid-resolution because `resolve_effect` → `resolve_primitive` → `execute_action` → `check_immediate_designations` is a natural call chain. |
| **Maintainability** | ✅ Good | One function, one call site. Ascend is the only current occupant. New designations add to the same function. No new state flags, no resolution-tracking machinery. |
| **Extensibility** | ✅ Good | Any future "immediate static designation" drops into `check_immediate_designations`. The `execute_action` hook is also where Phase 6 replacement effects will live — they need the same "run between mutation and next clause" timing. |

**The key insight:** We don't need to *know* we're mid-resolution. We just need to run the check after every mutation. `execute_action` already exists as exactly that hook point. The engine doesn't need a `resolving` flag because it doesn't need to behave differently mid-resolution vs between-resolutions for this check — it always runs.

**Continuous effect reapplication:** After granting city's blessing, the function calls `reapply_continuous_effects()`. This is the same call that Phase 5's layer system will make whenever a continuous effect changes. The timing is correct: it runs before `resolve_effect` returns to process the next sub-effect in the `Sequence`.

##### Rejected alternatives (brief)

- **B: Resolution context flag** (`resolving: Option<ObjectId>` on GameState) — Strictly worse. Ascend applies outside resolution too (play land as 10th permanent), so the flag doesn't eliminate the need for the check elsewhere. Adds a mutable flag maintenance hazard.
- **C: `Effect::Sequence` interleave** (checkpoint between sub-effects) — Subsumed by A. `execute_action` is deeper in the call chain, catching all mutations regardless of Effect tree shape. C would miss mutations inside ForEach/Conditional.
- **D: Delta log scan** — Over-engineered. Delta log is for Phase 7 triggers (stack-based). Ascend is a static designation (non-stack). Conceptual mismatch + dependency on unbuilt infrastructure.

#### Recommendation

**Proposal A: `execute_action` hook.** It scores well on all 4 pillars and requires minimal new code — one function, one call site. It doesn't need to know about resolution state because it runs after every mutation regardless of context, which is exactly what the CR requires for immediate static designations.

The only open question is the `reapply_continuous_effects()` call inside `check_immediate_designations()`. This function doesn't exist yet (it's Phase 5). For now, the designation check can be implemented without it — the continuous effect reapplication is only needed once the layer system exists. Pre-Phase-5, Ascend can store the designation and individual card implementations can query it directly (like Skymarcher Aspirant checking `game.has_designation(player, CityBlessing)` in its own ability definition).

#### Implications for `state-tracking-architecture.md`

The doc's claim "603.8: No interleaving needed — single post-resolution pass" remains correct for 603.8 state *triggers*. But it needs a note: **immediate static designations (702.131 Ascend) ARE different from state triggers and DO require mid-resolution checks, handled via the `execute_action` hook, not the delta log.**

---

### Prototype — Architecture Proposals

The question: how does the engine handle a permanent whose base characteristics (P/T, color, mana cost) depend on how it was cast, and only while it's on the stack or battlefield?

#### What makes Prototype unique

1. **Zone-dependent characteristics:** On stack/battlefield → prototype values. In hand/GY/library/exile → normal values.
2. **Cast-mode memory:** The permanent must remember "I was cast for my prototype cost" — this is a copiable value per 702.160b.
3. **Not a DFC:** Single face, no transform. But zone-dependent characteristic overrides.
4. **Affects base characteristics:** This isn't a +1/+1 counter or a continuous effect. It changes the *base* P/T, color, and mana cost — layer 1 / copiable values.

#### Rejected alternatives (brief)

- **A: `CastMode` enum on zone sidecars** — Correct and simple for Prototype alone, but cast-mode-only scope. Doesn't extend to Perpetual/Alchemy (which aren't cast-mode-based). Would have been the winner without Perpetual in scope.
- **B: `FaceData` vector + `active_face_index`** — Rejected due to the **partial-override problem**: Prototype changes P/T, color, and mana cost but NOT abilities or types. DFC faces have entirely different abilities. Shoehorning both into `faces[active_face]` requires "use face 1 for stats but face 0 for abilities" — breaks the clean face abstraction. Face abstraction should focus on DFCs (genuinely multi-faced).

#### Proposal C: Characteristic override layer (most general)

**Mechanism:** Instead of cast-mode-specific storage, add a general "base characteristic override" mechanism that sits between CardData and the layer system:

```rust
struct CharacteristicOverrides {
    power: Option<i32>,
    toughness: Option<i32>,
    mana_cost: Option<ManaCost>,
    colors: Option<Vec<Color>>,
    // None = use CardData default
}

// On BattlefieldEntity:
base_overrides: CharacteristicOverrides,  // applied before layer system
```

`characteristics.rs` checks `base_overrides` first, falls through to `card_data` for any `None` fields. Prototype sets the overrides at cast time. DFCs could also use this (override everything for back face). Any future "this permanent has different base characteristics" effect uses the same mechanism.

| Pillar | Rating | Notes |
|--------|--------|-------|
| **Speed** | ✅ Good | Option chain per characteristic. Same as Proposal A. |
| **Correctness** | ✅ Correct | Partial overrides work naturally — Prototype sets P/T/color/cost, leaves abilities as None (fall through to CardData). DFCs would set everything. Zone-dependent: overrides only exist on zone sidecars. |
| **Maintainability** | ⚠️ Moderate | More general than needed for just Prototype. The struct has fields that are always None for most permanents. But it's simple (no trait objects, no dynamic dispatch). |
| **Extensibility** | ✅ Good | Any effect that changes base characteristics (Prototype, DFC, hypothetical future "this card has different stats when cast from graveyard") uses the same pattern. More general than CastMode. |

##### Copiable value handling for C

Same as Proposal A: `base_overrides` lives on the zone sidecar, so copies inherit it on the battlefield, and zone changes strip it naturally.

#### Decision: Proposal C wins (updated with Perpetual/Alchemy)

Perpetual/Alchemy mechanics are confirmed IN-SCOPE. This changes the calculus — Proposal A (simplest for Prototype alone) loses to C (handles both Prototype and Perpetual).

**Perpetual** effects (e.g., "perpetually gains +1/+2", "perpetually becomes blue") modify base characteristics in a way that **persists across zone changes**. This is different from Prototype (zone-sidecar-scoped) but uses the same struct shape — `CharacteristicOverrides` with `Option` fields for each characteristic.

The key difference is **where the overrides live:**
- **Prototype:** `CharacteristicOverrides` on `BattlefieldEntity` / `StackEntry` (zone sidecars). Stripped on zone change — correct per CR.
- **Perpetual:** `CharacteristicOverrides` on `GameObject` itself. Survives zone changes — correct per Alchemy rules.

Both use the same struct, same `characteristics.rs` query logic (check overrides → fall through to CardData), just stored at different levels. The query order becomes:

```rust
pub fn get_base_power(game: &GameState, id: ObjectId) -> Option<i32> {
    // 1. Zone-sidecar overrides (Prototype — zone-dependent)
    if let Some(entry) = game.battlefield.get(&id) {
        if let Some(power) = entry.base_overrides.power {
            return Some(power);
        }
    }
    // 2. Object-level overrides (Perpetual — zone-independent)
    let obj = game.objects.get(&id)?;
    if let Some(power) = obj.perpetual_overrides.power {
        return Some(power);
    }
    // 3. CardData defaults
    obj.card_data.power
}
```

This naturally handles the interaction: a Prototype creature with a perpetual +1/+2 buff uses the prototype P/T as the base (step 1), and perpetual as a separate layer. If the Prototype creature moves to hand (sidecar stripped), the perpetual override (step 2) still applies on top of CardData defaults (step 3).

**Decision: Proposal C is the winner.** The `CharacteristicOverrides` struct serves both Prototype and Perpetual with the same code, differentiated only by storage location. Face abstraction (Discussion Item #1) remains separate — it handles DFCs, which have genuinely different faces with different abilities, not partial overrides.
