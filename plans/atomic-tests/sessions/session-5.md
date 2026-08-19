# Session 5: Rules 600–608 — Spells, Abilities, and Effects (Casting/Activation/Resolution Pipeline)

> **CR Source:** `MTG-Rules/LLM-Chapter-Splits/ch6-pt-1.txt` (rules 600–608)
> **Generated:** 2026-04-05
> **Scope:** Casting pipeline (601), activation (602), triggered abilities (603), static abilities (604), mana abilities (605), loyalty abilities (606), linked abilities (607), resolution (608)

---

## Rule 600 — General

### 600.1 — Chapter header

**Classification: PURE-DEF.** "Chapter 6 covers spells, abilities, and effects." No mechanical consequence.

---

## Rule 601 — Casting Spells

### 601.1 — Errata: "playing" → "casting"

**Classification: PURE-DEF.** Oracle text errata renaming only. No mechanical consequence for the engine — we use "cast" terminology throughout.

### 601.1a — "Playing a card" means playing as land or casting as spell

**Classification: BOUNDARY-DEF.** The engine must distinguish between "play a card" (which allows either land play or spell cast) and "cast a spell" (which is spell-only). Effects that say "play" must permit both paths.

**ATOM-601.1a-001**

- **Rule:** 601.1a — "Playing a card" means playing that card as a land or casting that card as a spell, whichever is appropriate.
- **Mechanism:** "Play a card" effect must allow land play for land cards
- **Minimal Board:** Player has a land card exiled with "you may play this card." It is their main phase, stack empty, they haven't played a land this turn.
- **Action:** Player uses the "play" permission on the exiled land.
- **Expected Result:** The land enters the battlefield (land play path). It is NOT cast as a spell and does NOT go on the stack.
- **Phase:** Phase 8 (play-from-exile permissions)
- **Ticket:** NEW — "play a card" permission must route to land play or cast based on card type

**ATOM-601.1a-002**

- **Rule:** 601.1a — "Playing a card" means playing that card as a land or casting that card as a spell, whichever is appropriate.
- **Mechanism:** "Play a card" effect must allow spell cast for non-land cards
- **Minimal Board:** Player has an instant exiled with "you may play this card." They have priority.
- **Action:** Player uses the "play" permission on the exiled instant.
- **Expected Result:** The instant is cast (goes on the stack as a spell).
- **Phase:** Phase 8 (play-from-exile permissions)
- **Ticket:** Same as ATOM-601.1a-001

---

### 601.2 — Casting a spell: overview of the full procedure

**Classification: TESTABLE.** The rule defines that casting is a sequential procedure (601.2a–i) and that if a player can't comply with a step, the entire casting is illegal and the game rewinds. The rewind behavior is independently testable.

**ATOM-601.2-001**

- **Rule:** 601.2 — If a player is unable to comply with the requirements of a step, the casting is illegal; the game returns to the moment before casting was proposed.
- **Mechanism:** Rollback on failed casting step
- **Minimal Board:** Player has a spell in hand that requires a target creature. No creatures on the battlefield.
- **Action:** Player attempts to cast the spell.
- **Expected Result:** Casting fails at 601.2c (no legal targets). The card is returned to hand. Mana pool is unchanged. Stack is unchanged.
- **Phase:** Phase 5 Pre-Work (T18 — 601.2 pipeline)
- **Ticket:** T18

---

### 601.2a — Move spell to stack, apply continuous effects

**Classification: TESTABLE.** The spell must move to the stack and continuous effects that modify spell characteristics begin immediately.

**ATOM-601.2a-001**

- **Rule:** 601.2a — Player moves the card to the stack. It has all the characteristics of the card and that player becomes its controller.
- **Mechanism:** Card moves from hand to stack; controller is set
- **Minimal Board:** Player 0 has Lightning Bolt in hand.
- **Action:** Player 0 begins casting Lightning Bolt.
- **Expected Result:** Lightning Bolt is on the stack. Its controller is Player 0. It has all printed characteristics (instant type, {R} mana cost, etc.).
- **Phase:** ALREADY-IMPLEMENTED (cast.rs move_object to stack)
- **Ticket:** N/A

**ATOM-601.2a-002**

- **Rule:** 601.2a — Continuous effects that modify the spell's characteristics as you start casting it begin as it is put on the stack.
- **Mechanism:** Spell-modifying continuous effects apply on stack entry
- **Minimal Board:** Player controls a permanent with "Instant and sorcery spells you cast cost {1} less." Player has a {1}{R} instant in hand.
- **Action:** Player begins casting the instant; it is placed on the stack.
- **Expected Result:** The spell on the stack reflects modified characteristics (the cost reduction continuous effect is considered when determining total cost in 601.2f).
- **Phase:** Phase 5 Layers (L15 — cost modification scaffolding)
- **Ticket:** L15 / T18

**ATOM-601.2a-003**

- **Rule:** 601.2a — Card moves from current zone to stack when casting begins.
- **Mechanism:** `move_to_stack` works from zones other than Hand
- **Minimal Board:** Player has a card in graveyard with flashback. Player has sufficient mana.
- **Action:** Player begins casting the spell from graveyard via flashback.
- **Expected Result:** The card moves from graveyard to the stack. Controller is set. All printed characteristics present.
- **Phase:** Phase 8 (play-from-exile/graveyard permissions)
- **Ticket:** T18, Phase 8

---

### 601.2b — Choose modes, announce splice, alt/add costs, X, hybrid/Phyrexian choices

**Classification: TESTABLE.** This is a dense multi-clause rule. Each clause is independently testable.

**ATOM-601.2b-001**

- **Rule:** 601.2b — If the spell is modal, the player announces the mode choice.
- **Mechanism:** Mode choice stored on StackEntry
- **Minimal Board:** Player has a modal spell (e.g., a Charm with 3 modes) in hand and sufficient mana.
- **Action:** Player casts the modal spell, choosing mode 2.
- **Expected Result:** `StackEntry.chosen_modes` contains `[2]`. Resolution executes only mode 2's effect.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

**ATOM-601.2b-002**

- **Rule:** 601.2b — The player announces intentions to pay alternative or additional costs.
- **Mechanism:** Alt/add cost choice stored on StackEntry
- **Minimal Board:** Player has a spell with kicker {2} in hand. Player has enough mana for base + kicker.
- **Action:** Player casts the spell, announcing they will pay the kicker cost.
- **Expected Result:** `StackEntry.additional_costs_paid` contains `Kicker(...)`. The total cost in 601.2f includes the kicker mana.
- **Phase:** Phase 5 Pre-Work (T17/T18)
- **Ticket:** T17, T18

**ATOM-601.2b-003**

- **Rule:** 601.2b — A player can't apply two alternative methods of casting or two alternative costs to a single spell.
- **Mechanism:** Only one alternative cost allowed per cast
- **Minimal Board:** Player has a spell with both flashback and retrace. Spell is in graveyard.
- **Action:** Player attempts to cast with flashback AND retrace cost simultaneously.
- **Expected Result:** Illegal — the engine rejects the attempt. Only one alternative cost may be chosen.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

**ATOM-601.2b-004**

- **Rule:** 601.2b — If the spell has a variable cost (X), the player announces the value of X.
- **Mechanism:** X value announced and stored on StackEntry
- **Minimal Board:** Player has a spell with {X}{R} cost in hand.
- **Action:** Player casts the spell, announcing X=3.
- **Expected Result:** `StackEntry.x_value` is `Some(3)`. Total cost in 601.2f is {3}{R}.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

**ATOM-601.2b-005**

- **Rule:** 601.2b — If a cost includes hybrid mana symbols, the player announces the nonhybrid equivalent.
- **Mechanism:** Hybrid mana choice recorded
- **Minimal Board:** Player has a spell with {W/U} cost. Player has only {U} available.
- **Action:** Player casts the spell, choosing to pay {U} for the hybrid symbol.
- **Expected Result:** The total cost locked in 601.2f treats the hybrid as {U}. Payment succeeds.
- **Phase:** DEFERRED — Hybrid mana payment (types/mana.rs currently stubs hybrid)
- **Ticket:** NEW — "Hybrid mana choice during casting"

**ATOM-601.2b-006**

- **Rule:** 601.2b — If a cost includes Phyrexian mana symbols, the player announces whether to pay 2 life or colored mana.
- **Mechanism:** Phyrexian mana choice recorded
- **Minimal Board:** Player has a spell with {G/P} cost. Player has 2+ life but no green mana.
- **Action:** Player casts the spell, choosing to pay 2 life for the Phyrexian symbol.
- **Expected Result:** Total cost includes PayLife(2) instead of mana. Player's life decreases by 2 during payment (601.2h).
- **Phase:** DEFERRED — Phyrexian mana payment (types/mana.rs currently stubs Phyrexian)
- **Ticket:** NEW — "Phyrexian mana choice during casting"

**ATOM-601.2b-007**

- **Rule:** 601.2b — X value announced during 601.2b even when X appears only in additional cost, not mana cost.
- **Mechanism:** X choice for additional-cost-only X
- **Minimal Board:** Player has Devastating Summons ({R}, additional cost: sacrifice X lands, create two X/X tokens) in hand. Player controls 3 lands.
- **Action:** Player casts Devastating Summons, choosing X=2.
- **Expected Result:** X value stored on StackEntry as 2. Two lands sacrificed as additional cost. On resolution, two 2/2 tokens created.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

> **Implementation note (601.2b last line):** "Previously made choices may restrict options." Strategy: build engine so constraints compose naturally (e.g., flashback sets zone=graveyard, morph sets face-down=true, these restrict later choices automatically). Not a separate test — emergent from correct pipeline implementation.

---

### 601.2c — Choose targets

**Classification: TESTABLE.** Multiple independent clauses.

**ATOM-601.2c-001**

- **Rule:** 601.2c — The player announces their choice of an appropriate object or player for each target the spell requires.
- **Mechanism:** Target selection during casting
- **Minimal Board:** Player has Lightning Bolt in hand. Opponent is at 20 life.
- **Action:** Player casts Bolt targeting the opponent.
- **Expected Result:** `StackEntry.chosen_targets` contains the opponent's player ID.
- **Phase:** ALREADY-IMPLEMENTED (engine/cast.rs + engine/targeting.rs)
- **Ticket:** N/A

**ATOM-601.2c-002**

- **Rule:** 601.2c — A spell may require some targets only if an alternative or additional cost was chosen; otherwise cast as though it did not require those targets.
- **Mechanism:** Conditional targets based on kicker/mode
- **Minimal Board:** Player has a spell: "Destroy target creature. If this spell was kicked, destroy target enchantment as well." Kicker not paid.
- **Action:** Player casts the spell without kicker.
- **Expected Result:** Only one target is required (the creature). The enchantment target slot is skipped.
- **Phase:** Phase 5 Pre-Work (T18 — conditional targets based on additional costs)
- **Ticket:** T18

**ATOM-601.2c-003**

- **Rule:** 601.2c — The same target can't be chosen multiple times for any one instance of "target." However, the same object CAN be chosen once for each separate instance of "target."
- **Mechanism:** Per-instance target uniqueness
- **Minimal Board:** Player has a spell: "Tap two target creatures." Two creatures on the battlefield (A and B).
- **Action:** Player attempts to choose creature A for both target slots.
- **Expected Result:** Illegal — same creature can't be chosen twice for one instance of "target" that requires two targets.
- **Phase:** Phase 5 Pre-Work (T18 — target validation)
- **Ticket:** T18

**ATOM-601.2c-004**

- **Rule:** 601.2c — The same object CAN be chosen for separate instances of the word "target."
- **Mechanism:** Different target instances can share a target (Example in CR: "Destroy target artifact and target land" — same artifact land can be both)
- **Minimal Board:** Player has a spell: "Destroy target artifact and target land." An artifact land is on the battlefield.
- **Action:** Player chooses the artifact land for both the "target artifact" and "target land" slots.
- **Expected Result:** Legal. The artifact land is targeted by both instances. On resolution, it is destroyed.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

**ATOM-601.2c-005**

- **Rule:** 601.2c — If effects say an object must be chosen as a target, the player chooses targets to obey the maximum possible number of such effects without violating "can't be targeted" rules.
- **Mechanism:** Target-forcing effects maximized
- **Minimal Board:** Two effects say "creature A must be chosen as a target" and "creature B must be chosen as a target." Spell has only one target slot.
- **Action:** Player casts the spell.
- **Expected Result:** Player must choose one of A or B (maximizing: 1 out of 2 requirements met). The spell cannot have zero targets if a legal forced target exists.
- **Phase:** Phase 8 (target-forcing effects are rare and complex)
- **Ticket:** NEW — "Target-forcing effect maximization during 601.2c"

**ATOM-601.2c-006**

- **Rule:** 601.2c — Spell cast as though it did not require conditional targets when kicker not paid.
- **Mechanism:** Conditional targets absent when additional cost not paid
- **Minimal Board:** Player has Probe ({2}{U}, Kicker {1}{B}). Unkicked: no target. Kicked: target player discards 2. Player casts without kicker.
- **Action:** Player casts Probe without paying kicker.
- **Expected Result:** No target required. Spell resolves: controller draws 3 cards. No discard effect.
- **Phase:** Phase 5 Pre-Work (T18 — conditional targets)
- **Ticket:** T18

**ATOM-601.2c-007**

- **Rule:** 601.2c — Alternative target criteria based on additional cost paid.
- **Mechanism:** Kicker changes target legality criteria, not just presence/absence
- **Minimal Board:** Player has Bloodchief's Thirst ({B}, Kicker {2}{B}). Unkicked: destroy target CMC≤2 creature or planeswalker. Kicked: destroy target creature or planeswalker (no CMC restriction). Opponent controls a 5-CMC creature.
- **Action:** Player casts Bloodchief's Thirst with kicker, targeting the 5-CMC creature.
- **Expected Result:** Legal — kicked version has no CMC restriction. Without kicker, targeting a 5-CMC creature would be illegal.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

> **Implementation note (601.2c architecture):** StackEntry should have `targets: Vec<TargetSlot>` where slots can be `Required`, `Conditional(CostChoice)`, or `Alternative(CostChoice)`. Empty targets vec = untargeted; present-but-inactive slots = conditional targets not activated. This distinguishes "no targets" vs "untargeted" at the StackEntry level.

> **Re-evaluation note:** Each distinct clause in 601.2c should map to an ATOM test. Verify coverage on second pass when implementing T18.

---

### 601.2d — Divide or distribute effects among targets

**Classification: TESTABLE.**

**ATOM-601.2d-001**

- **Rule:** 601.2d — If the spell requires the player to divide or distribute an effect among one or more targets, the player announces the division. Each target must receive at least one.
- **Mechanism:** Damage/counter distribution at cast time
- **Minimal Board:** Player has Arc Lightning ("Deal 3 damage divided as you choose among one, two, or three targets"). Three creatures on battlefield.
- **Action:** Player casts Arc Lightning, distributing 2 to creature A and 1 to creature B.
- **Expected Result:** Distribution is stored on the StackEntry: [(A, 2), (B, 1)]. Each target receives ≥1. Resolution deals exactly those amounts.
- **Phase:** Phase 5 Pre-Work (T18 — choose_distribution DP method)
- **Ticket:** T18

**ATOM-601.2d-002**

- **Rule:** 601.2d — Each target must receive at least one of whatever is being divided.
- **Mechanism:** Zero-allocation rejected
- **Minimal Board:** Same Arc Lightning scenario with 3 creatures.
- **Action:** Player attempts to distribute 3 to A, 0 to B, 0 to C.
- **Expected Result:** Illegal distribution — B and C receive 0. Must redistribute so each chosen target gets ≥1.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

> **Implementation note (601.2d):** Division/distribution choices must route through `DecisionProvider::choose_distribution()`. Note this for future DP interface expansion.

---

### 601.2e — Post-proposal legality check

**Classification: TESTABLE.**

**ATOM-601.2e-001**

- **Rule:** 601.2e — The game checks to see if the proposed spell can legally be cast. If illegal, the game rewinds.
- **Mechanism:** Post-proposal legality check with rollback
- **Minimal Board:** Player proposes casting a spell (moves to stack, chooses targets), but a continuous effect makes it illegal (e.g., "players can't cast instant spells" and the spell is an instant).
- **Action:** 601.2e check runs after 601.2a–d.
- **Expected Result:** The spell is illegal. The game rewinds: card returns to hand, stack restored, no costs paid.
- **Phase:** Phase 5 Pre-Work (T18 — split legality check into pre-proposal + post-proposal)
- **Ticket:** T18

> **META-GAMESTATE-SNAPSHOT (deferred — no architectural risk):** Casting rollback requires GameState snapshot before 601.2a. On failure at any step, restore snapshot. Overlap with loop detection (rule 731) — both need state snapshot/comparison infrastructure. Potential implementation: clone mutable portions of GameState before 601.2a; restore on failure. Simple and correct. **Deferred:** No architectural decisions needed now. The `GameState` struct is already a complete, concise representation of game state, which is a sufficient starting point for both rollback and future loop detection. Implementation can happen when casting pipeline (T18) is built. See also [`state-tracking-architecture.md`](state-tracking-architecture.md).

---

### 601.2f — Determine total cost and lock it

**Classification: TESTABLE.** Multi-clause rule with cost assembly pipeline.

**ATOM-601.2f-001**

- **Rule:** 601.2f — Total cost = base mana cost (or alt cost) + additional costs + cost increases − cost reductions. Cost can't be reduced below {0}. Then total cost is locked in.
- **Mechanism:** Cost assembly pipeline
- **Minimal Board:** Player has a {3}{R} spell. A continuous effect says "spells cost {1} more." Player is paying kicker {2}.
- **Action:** Player casts the spell with kicker.
- **Expected Result:** Total cost = {3}{R} (base) + {2} (kicker) + {1} (increase) = {6}{R}. This is locked in.
- **Phase:** Phase 5 Pre-Work (T18) + Phase 5 Layers (L15 for cost modification effects)
- **Ticket:** T18, L15

**ATOM-601.2f-002**

- **Rule:** 601.2f — If the mana component is reduced to nothing, it is considered {0}. It can't be reduced to less than {0}.
- **Mechanism:** Cost floor at {0}
- **Minimal Board:** Player has a {1}{G} spell. Two effects each reduce cost by {1}. A third effect reduces cost by {G}. Total reduction = {2}{G}, but mana component is only {1} generic + {G}.
- **Action:** Player casts the spell.
- **Expected Result:** After reducing generic by {1} (first reduction) the generic becomes {0}. The second reduction can't reduce below {0}. After reducing by {G}, total mana cost is {0} (even though reduction is greater than the total cost). Final cost is at least {0}.
- **Phase:** Phase 5 Layers (L15)
- **Ticket:** L15

**ATOM-601.2f-003**

- **Rule:** 601.2f — Once the total cost is determined, effects that directly affect the total cost are applied. Then the resulting total cost becomes "locked in."
- **Mechanism:** Cost lock-in prevents later modifications
- **Minimal Board:** Player casts a spell. Total cost is locked at {2}{R}. After lock-in, a new cost-increase effect enters (e.g., from a trigger resolving).
- **Action:** Player proceeds to pay costs.
- **Expected Result:** The new cost increase does NOT apply — the cost was already locked in. Player pays {2}{R}.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

> **Note (601.2f-003):** See also ATOM-601.2h-001 which tests cost lock-in more thoroughly. This test focuses specifically on the lock-in *preventing* later modifications.

**ATOM-601.2f-004**

- **Rule:** 601.2f — Multiple cost reduction effects: player chooses application order via DecisionProvider.
- **Mechanism:** Cost reduction ordering choice
- **Minimal Board:** Player controls two different cost reduction permanents (e.g., "Instant spells cost {1} less" and "Red spells cost {R} less"). Player casts a {1}{R}{R} red instant.
- **Action:** Player chooses which reduction to apply first via DP.
- **Expected Result:** DP is prompted for application order. Both reductions apply; final cost depends on order if reductions interact (e.g., reducing generic first vs. colored first).
- **Phase:** Phase 5 Layers (L15 — cost modification)
- **Ticket:** L15

---

### 601.2g — Mana ability window before payment

**Classification: TESTABLE.**

**ATOM-601.2g-001**

- **Rule:** 601.2g — If the total cost includes a mana payment, the player then has a chance to activate mana abilities. Mana abilities must be activated before costs are paid.
- **Mechanism:** Mana ability activation window during casting
- **Minimal Board:** Player has a {2}{R} spell on the stack (costs locked). Player controls 3 Mountains (untapped).
- **Action:** Player activates mana abilities of the 3 Mountains within the 601.2g window.
- **Expected Result:** 3 red mana is added to pool. Mana abilities resolve immediately (no stack). Player can then pay {2}{R} in 601.2h.
- **Phase:** Phase 5 Pre-Work (T18 — noted as TODO for explicit mana ability window)
- **Ticket:** T18

---

### 601.2h — Pay total cost

**Classification: TESTABLE.** The Example in the CR text makes this clearly testable.

**ATOM-601.2h-001**

- **Rule:** 601.2h — The player pays the total cost. First, costs that don't involve random elements or moving objects from library to public zone. Then remaining costs.
- **Mechanism:** Cost payment ordering
- **Minimal Board:** Player has a spell with costs: {1}{B} mana + sacrifice a creature. Player controls a creature that reduces black spell costs by {1}.
- **Action:** Player sacrifices the creature (non-random, non-library cost), then pays mana.
- **Expected Result:** Per the Example (Altar's Reap + Thunderscape Familiar): cost was locked in at {B} (not {1}{B}) because cost locking happened BEFORE payment. Even though the familiar is sacrificed during payment, the locked cost doesn't change.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

**ATOM-601.2h-002**

- **Rule:** 601.2h — Partial payments are not allowed. Unpayable costs can't be paid.
- **Mechanism:** Atomic cost payment — no partial
- **Minimal Board:** Player has a spell with total cost {3}{R}. Player only has {2}{R} in pool.
- **Action:** Player attempts to pay.
- **Expected Result:** Payment fails. The spell casting is rewound (per 601.2 overall rollback). No mana is spent.
- **Phase:** ALREADY-IMPLEMENTED (can_pay_costs pre-check in costs.rs)
- **Ticket:** N/A

**ATOM-601.2h-003**

- **Rule:** 601.2h — Cost payment ordering matters and is player-controlled.
- **Mechanism:** Player chooses order of cost components via DP
- **Minimal Board:** Player controls Omnath, Locus of Mana (1/1, "Green mana doesn't empty from your mana pool" + "Omnath gets +1/+1 for each green mana in your mana pool"). Pool has 4 green mana → Omnath is 5/5. Player casts Momentous Fall ({2}{G}{G}, additional cost: sacrifice a creature, draw cards equal to sacrificed creature's power, gain life equal to its toughness).
- **Action A:** Sacrifice Omnath first → LKI P/T = 5/5 → draw 5, gain 5. Then pay {2}{G}{G} from pool.
- **Action B:** Pay {2}{G}{G} first → pool has 0 green → Omnath is 1/1 → sacrifice → LKI P/T = 1/1 → draw 1, gain 1.
- **Expected Result:** DP chooses payment order. Both orderings are legal but produce different outcomes. Tests that cost payment ordering is player-controlled.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

---

### 601.2i — Spell becomes cast; triggers fire

**Classification: TESTABLE.**

**ATOM-601.2i-001**

- **Rule:** 601.2i — Once 601.2a–h are completed, effects that modify the spell as it's cast are applied, then the spell becomes cast. Abilities that trigger when a spell is cast trigger at this time.
- **Mechanism:** SpellCast event emitted after all steps complete
- **Minimal Board:** Player casts Lightning Bolt. A permanent has "Whenever a player casts an instant spell, draw a card."
- **Action:** Player completes all casting steps for Bolt.
- **Expected Result:** `SpellCast` event is emitted. The triggered ability ("draw a card") triggers. (Trigger goes on stack per 603.3.)
- **Phase:** Phase 7 (triggered abilities). SpellCast event: ALREADY-IMPLEMENTED.
- **Ticket:** Phase 7 triggers

**ATOM-601.2i-002**

- **Rule:** 601.2i — If the spell's controller had priority before casting it, they get priority.
- **Mechanism:** Priority returns to caster after casting
- **Minimal Board:** Player 0 has priority. Player 0 casts a spell.
- **Action:** Casting completes.
- **Expected Result:** Player 0 receives priority again (they can cast another instant or pass).
- **Phase:** ALREADY-IMPLEMENTED (priority.rs)
- **Ticket:** N/A

**ATOM-601.2i-003**

- **Rule:** 601.2i — Effects modifying spell characteristics apply at the "spell becomes cast" step.
- **Mechanism:** Spell characteristics modified by continuous effects on completion of casting
- **Minimal Board:** Mycosynth Lattice ("All spells are colorless") on battlefield. Player casts a red spell.
- **Action:** Casting completes (601.2i).
- **Expected Result:** After 601.2i, spell on stack is colorless (Lattice's continuous effect applies). Triggers checking color see the modified color.
- **Phase:** Phase 5 Layers (L10 — color modification)
- **Ticket:** L10

---

### 601.3 — Permission and prohibition to cast

**Classification: TESTABLE.**

> **META-CAST-PERMISSION-LAYERS:** 601.3 is a meta-rule. Multiple rule subsystems feed into cast permission: timing (505.6a/b), prohibitions (L15 CantCastSpells), flash grants (601.3b–d), zone permissions (601.3f). Implementation should have a single `can_begin_casting()` function that queries all subsystems.

**ATOM-601.3-001**

- **Rule:** 601.3 — A player can begin to cast a spell only if a rule or effect allows it and no rule or effect prohibits it.
- **Mechanism:** Cast permission check
- **Minimal Board:** Player has an instant in hand. An effect says "Players can't cast spells." Player has priority.
- **Action:** Player attempts to cast the instant.
- **Expected Result:** Casting is prohibited. The attempt fails before 601.2a.
- **Phase:** Phase 5 Layers (L15 — PlayerActionRestriction::CantCastSpells)
- **Ticket:** L15

### 601.3a — Prohibition look-ahead for choices that change qualities

**Classification: TESTABLE.** Has an Example in the CR text.

> **Implementation note:** This requires careful architectural thought. The look-ahead for mutable-during-proposal characteristics is a constraint-satisfaction problem. Implementation needs a "tentative proposal" phase where the engine explores possible choices before committing.

**ATOM-601.3a-001**

- **Rule:** 601.3a — If a prohibition prevents casting spells with certain qualities, and choices during proposal could change those qualities, the player may begin casting (ignoring the prohibition).
- **Mechanism:** Cast-time look-ahead past prohibition
- **Minimal Board:** Player controls Void Winnower ("Opponents can't cast spells with even mana values"). Opponent has Rolling Thunder ({X}{R}{R}) in hand.
- **Action:** Opponent attempts to cast Rolling Thunder (choosing X to make mana value odd).
- **Expected Result:** Legal — because choosing X=1 makes mana value 3 (odd), the opponent may begin casting. The prohibition is checked against all possible choices.
- **Phase:** Phase 8 (look-ahead casting — D17 in implementation plan)
- **Ticket:** D17 — "Cast legality look-ahead + game-rule flash grants (601.3)"

### 601.3b — Flash look-ahead for choices that change qualities

**Classification: TESTABLE.** Has an Example.

**ATOM-601.3b-001**

- **Rule:** 601.3b — If an effect allows casting spells with certain qualities as though they had flash, choices during proposal that cause the spell to gain those qualities allow the player to begin casting as though it had flash.
- **Mechanism:** Flash grant look-ahead
- **Minimal Board:** An effect says "You may cast Aura spells as though they had flash." Player has a creature card with bestow in hand. It is opponent's turn.
- **Action:** Player begins casting with bestow (making it an Aura spell).
- **Expected Result:** Legal — choosing bestow makes it an Aura, which matches the flash-granting effect.
- **Phase:** Phase 8 (D17)
- **Ticket:** D17

### 601.3c — Flash if alternative/additional cost is paid

**Classification: TESTABLE.**

**ATOM-601.3c-001**

- **Rule:** 601.3c — If an effect allows casting as though it had flash only if an alternative or additional cost is paid, the player may begin casting as though it had flash.
- **Mechanism:** Conditional flash grant from alt/add cost
- **Minimal Board:** A spell has "You may cast this as though it had flash if you pay an additional {2}." It is opponent's turn.
- **Action:** Player begins casting, intending to pay the additional {2}.
- **Expected Result:** Legal — the flash condition is met by the intended additional cost payment.
- **Phase:** Phase 8 (D17)
- **Ticket:** D17

**ATOM-601.3c-002**

- **Rule:** 601.3c — Alternative cost (not just additional cost) can grant flash.
- **Mechanism:** Alternative cost flash grant
- **Minimal Board:** Player has a creature with "You may cast this by paying {E}. If you cast this way, you may cast it as though it had flash." (Primal Prayers pattern). It is opponent's turn.
- **Action:** Player begins casting, intending to pay the alternative {E} cost.
- **Expected Result:** Legal — the alternative cost grants flash.
- **Phase:** Phase 8 (D17)
- **Ticket:** D17

### 601.3d — Flash from conditions being met

**Classification: TESTABLE.**

**ATOM-601.3d-001**

- **Rule:** 601.3d — If a spell would have flash only if certain conditions are met, its controller may begin to cast it as though it had flash if those conditions are met.
- **Mechanism:** Conditional flash
- **Minimal Board:** A creature spell has "This spell has flash as long as you control an artifact." Player controls an artifact. It is opponent's turn.
- **Action:** Player begins casting the creature spell.
- **Expected Result:** Legal — condition is met, spell has flash.
- **Phase:** Phase 8 (D17)
- **Ticket:** D17

### 601.3e — Alternative characteristics for cast legality

**Classification: TESTABLE.** Has two Examples.

> **Implementation note:** Adventure cards (e.g., Bonecrusher Giant) are an excellent integration test for 601.3e + flash interactions. Each half of an Adventure card should be testable for flash independently when an effect grants flash to one characteristic set. Defer actual test spec to Phase 8/9 when Adventure is in scope.

**ATOM-601.3e-001**

- **Rule:** 601.3e — Some rules state that alternative characteristics are considered for cast legality. These replace the object's characteristics for this determination.
- **Mechanism:** Alternative characteristic check for cast permissions
- **Minimal Board:** Player controls Garruk's Horde ("You may cast creature spells from the top of your library"). Top card is a noncreature card with morph. Morph allows casting face-down as a creature.
- **Action:** Player attempts to cast the morph card from library top.
- **Expected Result:** Legal — the alternative characteristics (face-down 2/2 creature) satisfy Garruk's Horde's permission.
- **Phase:** Phase 9 (morph/face-down)
- **Ticket:** DEFERRED — Phase 9: Face-down permanents

### 601.3f — Casting from face-down exile

**Classification: TESTABLE.**

**ATOM-601.3f-001**

- **Rule:** 601.3f — A player may begin to cast a spell from face-down exile only if they can look at the face-down card.
- **Mechanism:** Face-down exile cast permission requires visibility
- **Minimal Board:** A card is face-down in exile. An effect says "You may cast cards exiled with [this]." Player is the one who exiled it (can look at it).
- **Action:** Player attempts to cast the face-down exiled card.
- **Expected Result:** Legal — player can look at it.
- **Phase:** Phase 8 (D21 — exile zone metadata)
- **Ticket:** D21

**ATOM-601.3f-002**

- **Rule:** 601.3f — Player cannot cast a face-down exiled card they can't look at.
- **Mechanism:** Face-down exile cast permission denied + information leak prevention
- **Minimal Board:** Card A is face-down in exile, exiled by Player 0's effect (Player 0 can look at it, can cast it). Card B is face-down in exile, exiled by Player 1's effect (Player 0 cannot look at it).
- **Action:** Player 0 attempts to cast Card B.
- **Expected Result:** Illegal — Player 0 cannot look at Card B. Player 0 should not receive any information about Card B's characteristics.
- **Phase:** Phase 8 (D21 — exile zone metadata)
- **Ticket:** D21

---

### 601.4 — Mode/cost look-ahead within 601.2b

**Classification: TESTABLE.** Has an Example.

**ATOM-601.4-001**

- **Rule:** 601.4 — While announcing choices in 601.2b, if an option is available only if other choices are made later in that step, the player may consider those later choices.
- **Mechanism:** Intra-step look-ahead for mode/cost choices
- **Minimal Board:** Player has Inscription of Abundance (modal with kicker: "Choose one. If kicked, choose any number."). Player intends to pay kicker.
- **Action:** Player announces multiple modes (allowed because kicker will be paid in the same step).
- **Expected Result:** Legal — the player considers the kicker choice when choosing modes, even though kicker announcement normally comes after mode choice within 601.2b.
- **Phase:** Phase 5 Pre-Work (T18 — modal + additional cost interaction)
- **Ticket:** T18

> **Implementation note:** This rule is vaguely worded to permit a class of intra-step look-ahead interactions. Current test (Inscription of Abundance) covers the kicker→mode case. Other cards may exercise this rule differently. Flag for re-evaluation when new cards are added.

---

### 601.5 — Post-proposal illegality causes rewind

**Classification: TESTABLE.**

**ATOM-601.5-001**

- **Rule:** 601.5 — If a player is no longer allowed to cast a spell after completing its proposal (601.2a–d), the casting is illegal and the game rewinds.
- **Mechanism:** Post-proposal re-check
- **Minimal Board:** Player begins casting a spell. Between 601.2a and 601.2e, a triggered ability resolves that creates a "can't cast" prohibition matching this spell.
- **Action:** 601.2e checks legality.
- **Expected Result:** The spell is now illegal. Game rewinds to before casting was proposed.
- **Phase:** Phase 5 Pre-Work (T18 — post-proposal legality check)
- **Ticket:** T18

**ATOM-601.5-002**

- **Rule:** 601.5 — It doesn't matter if a rule makes casting illegal during cost determination/payment (601.2f–h) or after the spell has been cast.
- **Mechanism:** Cost-phase illegality does NOT cause rewind
- **Minimal Board:** Player is in the middle of paying costs (601.2h). A continuous effect that would make the spell illegal appears (e.g., from sacrificing a permanent as a cost).
- **Action:** Player continues paying costs.
- **Expected Result:** The spell is still cast. Post-proposal check (601.2e) already passed. Changes during payment don't rewind.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

### 601.5a — Flash conditions met at start persist

**Classification: TESTABLE.**

**ATOM-601.5a-001**

- **Rule:** 601.5a — Once a player has begun casting a spell that had flash because conditions were met, they may continue even if those conditions stop being met.
- **Mechanism:** Flash condition persistence after casting begins
- **Minimal Board:** A creature has "This spell has flash as long as you control an artifact." Player controls an artifact. Player begins casting (at instant speed).
- **Action:** During cost payment, the artifact is sacrificed (as a cost of another ability). The flash condition is no longer met.
- **Expected Result:** Casting continues — flash was valid when casting began.
- **Phase:** Phase 8 (D17)
- **Ticket:** D17

---

### 601.6 — Opponent makes choices for controller

**Classification: TESTABLE.**

**ATOM-601.6-001**

- **Rule:** 601.6 — Some spells specify that an opponent does something the controller normally does (e.g., choose mode, choose targets).
- **Mechanism:** Opponent-directed choice during casting
- **Minimal Board:** Player casts a spell: "Target opponent chooses a mode." One opponent.
- **Action:** The opponent chooses the mode.
- **Expected Result:** The chosen mode is the opponent's choice, not the caster's.
- **Phase:** Phase 8 (rare mechanic — Fact or Fiction, etc.)
- **Ticket:** NEW — "Opponent-directed choices during casting (601.6)"

### 601.6a — Controller picks which opponent decides

**Classification: TESTABLE.**

**ATOM-601.6a-001**

- **Rule:** 601.6a — If more than one opponent could make such a choice, the spell's controller decides which opponent will make the choice.
- **Mechanism:** Controller selects which opponent decides
- **Minimal Board:** 3-player game. Player 0 casts a spell where "an opponent" chooses. Players 1 and 2 are opponents.
- **Action:** Player 0 selects Player 1 to make the choice.
- **Expected Result:** Player 1 makes the choice.
- **Phase:** Phase 9 (multiplayer)
- **Ticket:** DEFERRED — Phase 9: Multiplayer

### 601.6b — Simultaneous actions: controller goes first

**Classification: TESTABLE.**

**ATOM-601.6b-001**

- **Rule:** 601.6b — If the spell instructs controller and another player to do something simultaneously during casting, controller goes first.
- **Mechanism:** Ordering exception for simultaneous cast-time actions
- **Minimal Board:** A spell says "You and target opponent each discard a card as you cast this spell."
- **Action:** Casting reaches the step where both discard.
- **Expected Result:** Controller discards first, then opponent discards. (Exception to APNAP rule 101.4.)
- **Phase:** Phase 8
- **Ticket:** NEW — "Controller-first simultaneous actions during casting (601.6b)"

---

### 601.7 — Casting a cost-altering spell doesn't affect existing stack

**Classification: TESTABLE.**

**ATOM-601.7-001**

- **Rule:** 601.7 — Casting a spell that alters costs won't affect spells and abilities that are already on the stack.
- **Mechanism:** Cost alteration doesn't retroactively modify stack entries
- **Minimal Board:** Player A's spell ({3}{R}) is on the stack. Player B casts a creature that says "Spells cost {1} more to cast."
- **Action:** Player B's creature resolves (or is cast). Player A's spell is still on the stack.
- **Expected Result:** Player A's spell's cost is unchanged (was locked at {3}{R}). The new cost increase only affects future spells.
- **Phase:** Phase 5 Layers (L15) — but the lock-in mechanism is T18
- **Ticket:** T18

---

## Rule 602 — Activating Activated Abilities

### 602.1 — Activated abilities have a cost and an effect

**Classification: PURE-DEF.** Defines the format "[Cost]: [Effect.]" — naming convention only.

### 602.1a — Activation cost is everything before the colon

**Classification: PURE-DEF with TESTABLE Example.** The Example is a gloss on cost parsing; the engine already structures costs as data. However, the principle that the activation cost must be paid by the activating player is testable.

**ATOM-602.1a-001**

- **Rule:** 602.1a — An ability's activation cost must be paid by the player who is activating it.
- **Mechanism:** Activation cost charged to activating player
- **Minimal Board:** Player 0 controls a permanent with "{2}, {T}: You gain 1 life." Player 0 has 2 mana in pool.
- **Action:** Player 0 activates the ability.
- **Expected Result:** 2 mana is deducted from Player 0's pool (not any other player). The permanent is tapped.
- **Phase:** ALREADY-IMPLEMENTED (engine/costs.rs pay_costs)
- **Ticket:** N/A

### 602.1b — Activation instructions after the effect

**Classification: PURE-DEF.** Defines where activation instructions appear in ability text. No independent mechanical consequence beyond 602.2/602.5 which test the actual restrictions.

### 602.1c — Only activated abilities can be "activated"

**Classification: PURE-DEF.** Naming clarification.

### 602.1d — Errata: "playing" → "activating"

**Classification: PURE-DEF.** Oracle text errata only.

### 602.1e — "Activation cost" modifications apply to total cost

**Classification: TESTABLE.**

**ATOM-602.1e-001**

- **Rule:** 602.1e — If a spell or ability modifies how a player may pay an "activation cost," that modification applies to the total cost, even if increased/decreased by other effects.
- **Mechanism:** Activation cost modifications apply to total, not base
- **Minimal Board:** Player controls a permanent with activation cost "{2}, {T}:" A continuous effect says "Activated abilities cost {1} more to activate." Another says "Activated abilities cost {1} less to activate."
- **Action:** Player activates the ability.
- **Expected Result:** Total cost = {2} (base) + {1} (increase) − {1} (reduction) = {2}, {T}. Modifications apply to total.
- **Phase:** Phase 5 Layers (L15 — cost modification scaffolding, applied to abilities)
- **Ticket:** L15

**ATOM-602.1e-002**

- **Rule:** 602.1e — Single cost modifier applies to total activation cost.
- **Mechanism:** Single cost increase on activation cost
- **Minimal Board:** Player controls a permanent with activation cost "{2}, {T}:" A continuous effect says "Activated abilities cost {1} more to activate." No reduction effects.
- **Action:** Player activates the ability.
- **Expected Result:** Total cost = {2} (base) + {1} (increase) = {3}, {T}. Prevents false pass where neither effect applies.
- **Phase:** Phase 5 Layers (L15 — cost modification scaffolding)
- **Ticket:** L15

---

### 602.2 — Activation procedure (parallel to 601.2)

**Classification: TESTABLE.** Establishes that only the controller (or owner if no controller) can activate, and that the procedure follows 601.2 steps.

**ATOM-602.2-001**

- **Rule:** 602.2 — Only an object's controller can activate its activated ability unless the object specifically says otherwise.
- **Mechanism:** Activation restricted to controller
- **Minimal Board:** Player 0 controls a permanent with an activated ability. Player 1 has priority.
- **Action:** Player 1 attempts to activate Player 0's permanent's ability.
- **Expected Result:** Illegal — Player 1 is not the controller.
- **Phase:** ALREADY-IMPLEMENTED (engine/cast.rs activate_ability checks controller)
- **Ticket:** N/A

**ATOM-602.2-002**

- **Rule:** 602.2 — If at any point during activation a player can't comply, the activation is illegal and the game rewinds.
- **Mechanism:** Activation rollback on failure
- **Minimal Board:** Player controls a permanent with "{T}, Sacrifice a creature: Draw a card." Player controls no other creatures.
- **Action:** Player begins activating (taps the permanent), then tries to sacrifice — no legal creature to sacrifice.
- **Expected Result:** Activation is illegal. The permanent is untapped (rollback). No card drawn.
- **Phase:** Phase 5 Pre-Work (T18 covers activation rollback by reference through 602.2b)
- **Ticket:** T18

### 602.2a — Announce activation, create ability on stack

**Classification: TESTABLE.**

**ATOM-602.2a-001**

- **Rule:** 602.2a — The player announces they are activating the ability. The ability is created on the stack as an object that's not a card. Its controller is the player who activated it.
- **Mechanism:** Ability object created on stack
- **Minimal Board:** Player 0 controls a permanent with an activated ability (non-mana).
- **Action:** Player 0 activates the ability.
- **Expected Result:** A non-card ability object appears on the stack. It has the text of the ability. Its controller is Player 0.
- **Phase:** ALREADY-IMPLEMENTED (engine/cast.rs activate_ability)
- **Ticket:** N/A

**ATOM-602.2a-002**

- **Rule:** 602.2a — If an activated ability is being activated from a hidden zone, the card that has that ability is revealed.
- **Mechanism:** Reveal on activation from hidden zone
- **Minimal Board:** Player has a card in hand with an activated ability that can be activated from hand (e.g., cycling). Hand is a hidden zone.
- **Action:** Player activates the ability.
- **Expected Result:** The card is revealed to all players as part of activation.
- **Phase:** Phase 8 (zone-activated abilities — T19 activation_zone)
- **Ticket:** T19

**ATOM-602.2a-003**

- **Rule:** 602.2a — Ability object on stack has no card name, no mana cost, no color, no card types.
- **Mechanism:** Ability object characteristic profile distinct from spell
- **Minimal Board:** Player 0 controls a permanent with an activated ability (non-mana). Player 0 activates the ability.
- **Action:** Inspect the ability object on the stack.
- **Expected Result:** The ability object has no card name, no mana cost, no color, no card types. It has the text of the ability and a controller. It is NOT a card.
- **Phase:** Phase 5 Pre-Work (T19)
- **Ticket:** T19

### 602.2b — Remainder of activation procedure = 601.2b–i

**Classification: TESTABLE.** This is a critical structural rule — it says activation follows the same pipeline as casting.

**ATOM-602.2b-001**

- **Rule:** 602.2b — The remainder of the activation process is identical to 601.2b–i. An activated ability's analog to a spell's mana cost is its activation cost.
- **Mechanism:** Activation follows casting pipeline steps
- **Minimal Board:** Player controls a permanent with an activated ability that has targets and a mana cost.
- **Action:** Player activates: chooses targets (601.2c analog), determines total cost (601.2f analog), activates mana abilities (601.2g analog), pays costs (601.2h analog).
- **Expected Result:** All steps execute in order. Target is stored, costs are locked and paid, ability is on the stack.
- **Phase:** Phase 5 Pre-Work (T18/T19)
- **Ticket:** T18, T19

---

### 602.3 — Opponent makes choices for controller (abilities)

**Classification: PURE-DEF (parallel to 601.6).** Same rule as 601.6 but for abilities. Tested via 601.6 tests — no separate ATOM needed.

### 602.3a — Controller picks which opponent decides (abilities)

**Classification: PURE-DEF (parallel to 601.6a).** Subsumed by 601.6a tests.

### 602.3b — Simultaneous actions: controller goes first (abilities)

**Classification: PURE-DEF (parallel to 601.6b).** Subsumed by 601.6b tests.

---

### 602.4 — Activating a cost-altering ability doesn't affect existing stack

**Classification: TESTABLE (parallel to 601.7).**

**ATOM-602.4-001**

- **Rule:** 602.4 — Activating an ability that alters costs won't affect spells and abilities that are already on the stack.
- **Mechanism:** Cost alteration from abilities doesn't retroactively affect stack
- **Minimal Board:** Player A's spell is on the stack with locked cost {3}{R}. Player B activates an ability that creates a continuous effect "Spells cost {1} more."
- **Action:** The ability resolves.
- **Expected Result:** Player A's spell cost is unchanged (was locked). New cost increase only affects future spells/abilities.
- **Phase:** Phase 5 Layers (L15)
- **Ticket:** L15

> **Implementation note (602.4):** Costs are locked at payment time, so re-activating a "spells cost less" ability doesn't retroactively discount already-cast spells. This is implicitly handled by the lock-in architecture. Urianger Augurelt scenario from audit is a good integration test candidate.

---

### 602.5 — Can't begin activating a prohibited ability

**Classification: TESTABLE.**

**ATOM-602.5-001**

- **Rule:** 602.5 — A player can't begin to activate an ability that's prohibited from being activated.
- **Mechanism:** Activation prohibition check
- **Minimal Board:** Player controls a permanent with an activated ability. An effect says "Activated abilities of artifacts can't be activated." The permanent is an artifact.
- **Action:** Player attempts to activate the ability.
- **Expected Result:** Activation is prohibited. Fails before any steps.
- **Phase:** Phase 5 Layers (L15 — CantActivateAbilities)
- **Ticket:** L15, T19

### 602.5a — Tap/untap cost + summoning sickness

**Classification: TESTABLE.**

**ATOM-602.5a-001**

- **Rule:** 602.5a — A creature's activated ability with {T} or {Q} in its cost can't be activated unless the creature has been under its controller's control since the start of their most recent turn. Ignore for haste.
- **Mechanism:** Summoning sickness blocks {T}/{Q} activation
- **Minimal Board:** Player plays a creature with "{T}: Add {G}." It entered this turn (summoning sick).
- **Action:** Player attempts to activate the tap ability.
- **Expected Result:** Illegal — creature is summoning sick.
- **Phase:** ALREADY-IMPLEMENTED (engine/costs.rs check for Cost::Tap + summoning sickness). T10 adds {Q} check.
- **Ticket:** T09 (summoning sickness rework), T10 ({Q} check)

**ATOM-602.5a-002**

- **Rule:** 602.5a — Ignore summoning sickness for creatures with haste.
- **Mechanism:** Haste bypasses summoning sickness for tap/untap costs
- **Minimal Board:** Player plays a creature with haste and "{T}: Add {R}." It entered this turn.
- **Action:** Player activates the tap ability.
- **Expected Result:** Legal — haste bypasses summoning sickness.
- **Phase:** ALREADY-IMPLEMENTED (engine/costs.rs haste check)
- **Ticket:** N/A

### 602.5b — Activation restriction persists through controller change

**Classification: TESTABLE.**

**ATOM-602.5b-001**

- **Rule:** 602.5b — If an activated ability has a restriction on its use (e.g., "Activate only once each turn"), the restriction continues to apply even if its controller changes.
- **Mechanism:** Once-per-turn restriction persists across controller change
- **Minimal Board:** Player 0 controls a permanent with "Activate only once each turn." Player 0 activates it. Then Player 1 gains control of it.
- **Action:** Player 1 attempts to activate the same ability this turn.
- **Expected Result:** Illegal — the ability was already activated this turn, regardless of controller change.
- **Phase:** Phase 5 Pre-Work (T19 — activation restrictions, once-per-turn tracking)
- **Ticket:** T19

### 602.5c — Acquired restriction applies only to that instance

**Classification: TESTABLE.**

**ATOM-602.5c-001**

- **Rule:** 602.5c — If an object acquires an activated ability with a restriction from another object, that restriction applies only to that ability as acquired from that object. Not to other identically worded abilities.
- **Mechanism:** Per-acquisition restriction scoping
- **Minimal Board:** Object A has "{T}: Draw a card. Activate only once each turn." Object B copies that ability from A. Object B also has its own "{T}: Draw a card. Activate only once each turn."
- **Action:** Object B activates the copied ability (once restriction from A). Then B attempts to activate its own identically-worded ability.
- **Expected Result:** Legal — B's own ability is separate from the copied ability. Each has its own once-per-turn restriction.
- **Phase:** Phase 8 (ability copying)
- **Ticket:** DEFERRED — Phase 8: Ability copying

> **Integration test note:** Necrotic Ooze + 2× Skinshifter in graveyard: Ooze gains two separate instances of the activated ability, each with independent "once each turn" restriction. Good Phase 8 integration test.

### 602.5d — "Activate only as a sorcery" means sorcery timing

**Classification: TESTABLE.**

> **Design note:** Sorcery-speed timing check should be a shared utility function reused across: spell casting (505.6a), land play (505.6b), loyalty abilities (606.3), "activate only as a sorcery" (602.5d). Single `passes_sorcery_timing()` function.

**ATOM-602.5d-001**

- **Rule:** 602.5d — "Activate only as a sorcery" means the player must follow sorcery timing rules (active player, main phase, empty stack).
- **Mechanism:** Sorcery-speed activation restriction
- **Minimal Board:** Player controls a permanent with an ability: "{1}: Effect. Activate only as a sorcery." It is the opponent's turn.
- **Action:** Player attempts to activate the ability.
- **Expected Result:** Illegal — not the active player.
- **Phase:** Phase 5 Pre-Work (T19 — ActivationRestriction::SorcerySpeed)
- **Ticket:** T19

**ATOM-602.5d-002**

- **Rule:** 602.5d — "Activate only as a sorcery" — sorcery-speed means stack must be empty.
- **Mechanism:** Sorcery-speed requires empty stack
- **Minimal Board:** Player is active player in main phase. A spell is on the stack.
- **Action:** Player attempts to activate a sorcery-speed ability.
- **Expected Result:** Illegal — stack is not empty.
- **Phase:** Phase 5 Pre-Work (T19)
- **Ticket:** T19

### 602.5e — "Activate only as an instant" means instant timing

**Classification: TESTABLE.**

**ATOM-602.5e-001**

- **Rule:** 602.5e — "Activate only as an instant" means the player must follow instant timing rules.
- **Mechanism:** Instant-speed activation restriction
- **Minimal Board:** Player controls a permanent with "Activate only as an instant." Player has priority.
- **Action:** Player activates the ability.
- **Expected Result:** Legal — instant timing means "whenever you have priority," which is the default for activated abilities. However, this restriction exists primarily for cards like Lion's Eye Diamond, preventing mana abilities from being activated during casting to circumvent cost-payment rules. "Activate only as an instant" means the ability can be activated any time you have priority, but NOT during the casting/activation process (mana ability windows, etc.).
- **Phase:** Phase 5 Pre-Work (T19)
- **Ticket:** T19

---

## Rule 603 — Handling Triggered Abilities

### 603.1 — Triggered abilities: "When/Whenever/At" format

**Classification: PURE-DEF.** Defines the textual format. No mechanical consequence beyond 603.2+.

### 603.1a — Triggered ability may include post-effect instructions

**Classification: PURE-DEF.** Structural note about where targeting/counter-prevention text lives.

### 603.1b — Multiple trigger conditions with "all"

**Classification: TESTABLE.**

> **META-MULTI-CONDITION-TRIGGERS:** Multi-condition triggers ("whenever you cast a creature AND an artifact in the same turn") require per-turn event tracking. Architecture: `TurnEventLog` on GameState tracking event categories per player per turn. Trigger matcher checks log for all conditions being met. This is an architectural decision needed before Phase 7 implementation.

**ATOM-603.1b-001**

- **Rule:** 603.1b — A triggered ability may have more than one trigger condition with an instruction referring to "all" of those conditions happening in a period.
- **Mechanism:** Multi-condition trigger tracking
- **Minimal Board:** A permanent has "Whenever you cast a creature spell and an artifact spell in the same turn, draw a card." Player has cast a creature this turn but no artifact.
- **Action:** Player casts an artifact spell.
- **Expected Result:** Both conditions met this turn — the ability triggers.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

---

### 603.2 — Trigger event matching

**Classification: TESTABLE.**

**ATOM-603.2-001**

- **Rule:** 603.2 — Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers. The ability doesn't do anything at this point.
- **Mechanism:** Trigger detection (no immediate effect)
- **Minimal Board:** Player controls Soul Warden ("Whenever a creature enters, you gain 1 life"). Opponent casts a creature spell.
- **Action:** The creature spell resolves and enters the battlefield.
- **Expected Result:** Soul Warden's ability triggers (is added to pending_triggers queue). No life gain yet — the ability hasn't resolved.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.2a — Triggers fire even when casting/activating is illegal

**Classification: TESTABLE.**

**ATOM-603.2a-001**

- **Rule:** 603.2a — Triggered abilities can trigger even when it isn't legal to cast spells and activate abilities. Effects that preclude abilities from being activated don't affect them.
- **Mechanism:** Triggers bypass activation prohibitions
- **Minimal Board:** An effect says "Activated abilities can't be activated." A permanent has a triggered ability "Whenever a creature enters, draw a card." A creature enters.
- **Action:** Creature enters the battlefield.
- **Expected Result:** The triggered ability triggers normally. The activation prohibition does not affect triggered abilities.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.2b — "At the beginning of" phase/step triggers

**Classification: TESTABLE.**

**ATOM-603.2b-001**

- **Rule:** 603.2b — When a phase or step begins, all abilities that trigger "at the beginning of" that phase or step trigger.
- **Mechanism:** Phase/step-begin trigger
- **Minimal Board:** Player controls a permanent: "At the beginning of your upkeep, you gain 1 life."
- **Action:** Player's upkeep step begins.
- **Expected Result:** The ability triggers.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

> **Composition test note:** Integration test: multiple "at beginning of upkeep" triggers from both players. Test APNAP ordering + player-chosen ordering within same player's triggers.

### 603.2c — Triggers once per event, but multiple times for multiple occurrences

**Classification: TESTABLE.** Has an Example.

**ATOM-603.2c-001**

- **Rule:** 603.2c — An ability triggers only once each time its trigger event occurs. However, it can trigger repeatedly if one event contains multiple occurrences.
- **Mechanism:** Per-occurrence triggering
- **Minimal Board:** A permanent has "Whenever a land is put into a graveyard from the battlefield, draw a card." A spell destroys 3 lands simultaneously.
- **Action:** The spell resolves, sending 3 lands to graveyard.
- **Expected Result:** The ability triggers 3 times (once per land), not once for the batch.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.2d — Additional trigger counts

**Classification: TESTABLE.**

> **Implementation note:** Trigger multipliers (Panharmonicon) could be implemented as a game-engine level effect that modifies trigger count during the `check_triggers()` pass. Not a replacement effect — it's a count modifier. Needs design decision before Phase 7.

**ATOM-603.2d-001**

- **Rule:** 603.2d — An ability may state that a triggered ability triggers additional times. The ability triggers that many times rather than just once.
- **Mechanism:** Trigger multiplier
- **Minimal Board:** Player controls Panharmonicon ("If an artifact or creature entering the battlefield causes a triggered ability to trigger, that ability triggers an additional time"). Player controls Soul Warden. A creature enters.
- **Action:** Creature enters the battlefield.
- **Expected Result:** Soul Warden triggers twice (once normal + one additional from Panharmonicon).
- **Phase:** Phase 7 (triggered abilities — trigger doubling)
- **Ticket:** Phase 7 triggers

### 603.2e — "Becomes" triggers only on state change

**Classification: TESTABLE.** Has an Example.

**ATOM-603.2e-001**

- **Rule:** 603.2e — "Becomes" triggers only at the time the named event happens — not if the state already exists, and not if a permanent enters the battlefield already in that state.
- **Mechanism:** "Becomes tapped" doesn't trigger on ETB tapped
- **Minimal Board:** A permanent has "Whenever a permanent becomes tapped, draw a card." A creature enters the battlefield tapped (via replacement effect).
- **Action:** The creature enters tapped.
- **Expected Result:** The "becomes tapped" ability does NOT trigger. The creature was never on the battlefield in an untapped state.
- **Phase:** Phase 7 (triggered abilities) + Phase 6 (replacement effects for "enters tapped")
- **Ticket:** Phase 7 triggers

**ATOM-603.2e-002**

- **Rule:** 603.2e — "Becomes attached" doesn't trigger when re-equipping same creature.
- **Mechanism:** "Becomes" requires state change — no re-trigger on same state
- **Minimal Board:** An Equipment with "Whenever this becomes attached to a creature, draw a card" is attached to creature A. Player activates Equip targeting creature A (the same creature).
- **Action:** Equip resolves targeting creature A.
- **Expected Result:** Equipment doesn't "become attached" (it already was). Trigger does NOT fire.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.2f — Hidden objects don't trigger

**Classification: TESTABLE.**

> **META-HIDDEN-ZONE-TRIGGER-COMPLEXITY:** Reference `plans/atomic-tests/603-2f-complexity.md` for the Library of Leng + Guerrilla Tactics + Future Sight scenario. This exercises: hidden zone trigger suppression (603.2f), "look back in time" (603.10), replacement effects on discard (Library of Leng), and top-of-library reveal (Future Sight). Architectural takeaway: trigger checking must happen AFTER all replacement effects resolve and zone changes finalize, using the final public/hidden zone status of each object. "MtG is arbitrarily complex. Trust composition + unit tests where correct behavior is well-defined."

**ATOM-603.2f-001**

- **Rule:** 603.2f — If a triggered ability's trigger condition is met, but the object with the ability is at no time visible to all players, the ability does not trigger.
- **Mechanism:** Hidden-zone trigger suppression
- **Minimal Board:** A card in a player's library has "Whenever a creature enters the battlefield, you gain 1 life." A creature enters.
- **Action:** Creature enters the battlefield.
- **Expected Result:** The library card's ability does NOT trigger — it was never visible to all players.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.2g — Prevented/replaced events don't trigger

**Classification: TESTABLE.** Has an Example.

**ATOM-603.2g-001**

- **Rule:** 603.2g — An ability triggers only if its trigger event actually occurs. An event that's prevented or replaced won't trigger anything.
- **Mechanism:** Prevented events don't cause triggers
- **Minimal Board:** A permanent has "Whenever damage is dealt to you, draw a card." Fog is active ("Prevent all combat damage this turn"). An attacker deals combat damage.
- **Action:** Combat damage is prevented.
- **Expected Result:** The "whenever damage is dealt" ability does NOT trigger — the damage was prevented.
- **Phase:** Phase 7 (triggered abilities) + Phase 6 (prevention effects)
- **Ticket:** Phase 7 triggers

### 603.2h — "Do this only once each turn" triggered ability limit

**Classification: TESTABLE.**

**ATOM-603.2h-001**

- **Rule:** 603.2h — A triggered ability with "Do this only once each turn" triggers only if the action hasn't been taken that turn.
- **Mechanism:** Once-per-turn trigger restriction
- **Minimal Board:** A permanent has "Whenever a creature enters, draw a card. Do this only once each turn." Two creatures enter sequentially.
- **Action:** First creature enters (triggers, resolves, player draws). Second creature enters.
- **Expected Result:** Second entry does NOT trigger the ability — already used this turn.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

**ATOM-603.2h-002**

- **Rule:** 603.2h — "Do this only once each turn" checked at resolution, not trigger time.
- **Mechanism:** Once-per-turn restriction on triggered ability checked at resolution
- **Minimal Board:** Player controls Nykthos Paragon ("Whenever you gain life, you may put that many +1/+1 counters on each creature you control. Do this only once each turn."). Two lifelink creatures deal combat damage simultaneously → two Paragon triggers on stack.
- **Action:** First trigger resolves: player puts counters. Second trigger resolves.
- **Expected Result:** First trigger: DP prompted, counters placed. Second trigger: "do this only once" prevents the action — DP is NOT prompted, no counters placed.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

---

### 603.3 — Triggered ability goes on stack at next priority

**Classification: TESTABLE.**

> **Implementation note:** Similar to 602.2a: triggered ability objects on stack should be checked for characteristic profile (no card name, no mana cost, etc.).

**ATOM-603.3-001**

- **Rule:** 603.3 — Once triggered, the controller puts it on the stack the next time a player would receive priority.
- **Mechanism:** Trigger → stack placement timing
- **Minimal Board:** A creature's ETB trigger fires during resolution of a spell. The spell is still resolving.
- **Action:** Spell finishes resolving.
- **Expected Result:** Before the next player gets priority, the triggered ability is placed on the stack.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.3a — Triggered ability controller = source controller at trigger time

**Classification: TESTABLE.**

> **Implementation note:** This is hard to trigger in practice. Multiplayer concession during trigger stacking may be the primary scenario. Defer concrete test until Phase 9 multiplayer.

**ATOM-603.3a-001**

- **Rule:** 603.3a — A triggered ability is controlled by the player who controlled its source at the time it triggered (except delayed triggers).
- **Mechanism:** Trigger controller = source controller at trigger time
- **Minimal Board:** Player 0 controls a permanent with "Whenever a creature enters, draw a card." Player 0's creature enters. Before the trigger goes on the stack, Player 1 gains control of the permanent.
- **Action:** The trigger is placed on the stack.
- **Expected Result:** Player 0 controls the trigger (they controlled the source when it triggered), even though Player 1 now controls the permanent.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.3b — APNAP ordering for multiple triggers

**Classification: TESTABLE.**

**ATOM-603.3b-001**

- **Rule:** 603.3b — If multiple abilities have triggered, they are placed in APNAP order. First: triggers whose condition isn't "another ability triggering." Second: remaining triggers.
- **Mechanism:** APNAP trigger stacking order
- **Minimal Board:** Player 0 (active) and Player 1 each control a "Whenever a creature enters" trigger. A creature enters.
- **Action:** Both triggers fire. They are placed on the stack.
- **Expected Result:** Player 0's trigger goes on the stack first (APNAP: active player first), then Player 1's. Player 1's resolves first (top of stack).
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

**ATOM-603.3b-002**

- **Rule:** 603.3b — Two-tier trigger stacking: first tier = triggers whose condition isn't "another ability triggering." Second tier = triggers that trigger from other triggers.
- **Mechanism:** Two-tier trigger stacking order
- **Minimal Board:** Strict Proctor ("Whenever a permanent entering causes a triggered ability to trigger, counter that ability unless its controller pays {2}") + a creature with ETB trigger (e.g., "When this enters, draw a card").
- **Action:** Creature enters the battlefield.
- **Expected Result:** Creature ETB trigger goes on stack first (tier 1). Strict Proctor's trigger goes on stack second (tier 2, above the ETB trigger). Proctor resolves first, potentially countering the ETB.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

> **META-TWO-TIER-TRIGGER-STACKING:** Engine needs a way to classify triggers as "triggers-on-trigger" vs "triggers-on-event." During `stack_pending_triggers()`, process in two passes.

### 603.3c — Modal triggered ability: mode chosen on stack placement

**Classification: TESTABLE.**

**ATOM-603.3c-001**

- **Rule:** 603.3c — If a triggered ability is modal, its controller announces mode choice when putting it on the stack. If no mode can be legally chosen, the ability is removed.
- **Mechanism:** Modal trigger mode selection
- **Minimal Board:** A permanent triggers with "Choose one — deal 2 damage to target creature; or gain 3 life." No creatures on the battlefield.
- **Action:** Trigger goes on the stack. Mode 1 requires a target creature (none exists). Mode 2 (gain life) is legal.
- **Expected Result:** Mode 2 is chosen. If neither mode were legal, the trigger would be removed from the stack.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

**ATOM-603.3c-002**

- **Rule:** 603.3c — Modal triggered ability with no legal mode is removed from the stack.
- **Mechanism:** Modal trigger fizzle when no mode is legal
- **Minimal Board:** A permanent triggers with "Choose one — destroy target artifact; or destroy target enchantment." No artifacts or enchantments on the battlefield.
- **Action:** Trigger fires.
- **Expected Result:** No legal mode can be chosen → ability removed from stack without resolving.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.3d — Trigger stack procedure = 601.2c–d

**Classification: TESTABLE.**

> **Implementation note:** The rule itself is a structural reference ("follow 601.2c–d"). The existing ATOM test covers the key behavior (no legal targets → removed from stack). Don't split further — the 601.2c/d tests cover the individual steps.

**ATOM-603.3d-001**

- **Rule:** 603.3d — The process for putting a triggered ability on the stack follows 601.2c–d (choose targets, divide effects). If no legal choices exist, the ability is removed from the stack.
- **Mechanism:** Trigger target selection follows casting rules
- **Minimal Board:** A triggered ability requires "target creature." No creatures on the battlefield.
- **Action:** Trigger tries to go on the stack.
- **Expected Result:** No legal target → ability is removed from the stack (never resolves).
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

---

### 603.4 — Intervening "if" clause

**Classification: TESTABLE.** Has an Example.

**ATOM-603.4-001**

- **Rule:** 603.4 — "When/Whenever/At [event], if [condition], [effect]" — condition checked on trigger AND on resolution. If false at either time, ability does nothing.
- **Mechanism:** Intervening-if double check — condition true at both trigger and resolution
- **Minimal Board:** Felidar Sovereign: "At the beginning of your upkeep, if you have 40 or more life, you win the game." Player has 42 life.
- **Action:** Upkeep begins. Condition is true (42 ≥ 40). Ability triggers. Life remains ≥ 40 through resolution.
- **Expected Result:** Ability resolves normally — player wins the game.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

**ATOM-603.4-002**

- **Rule:** 603.4 — Intervening-if: condition false at trigger time → ability doesn't trigger at all.
- **Mechanism:** Intervening-if blocks trigger
- **Minimal Board:** Felidar Sovereign. Player has 38 life at beginning of upkeep.
- **Action:** Upkeep begins.
- **Expected Result:** Condition is false (38 < 40). Ability does NOT trigger.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

**ATOM-603.4-003**

- **Rule:** 603.4 — Intervening-if: condition true at trigger time, false at resolution → ability does nothing.
- **Mechanism:** Intervening-if fails at resolution
- **Minimal Board:** Felidar Sovereign. Player has 42 life at beginning of upkeep. Trigger goes on stack.
- **Action:** In response, opponent casts a spell dealing 5 damage (player drops to 37 life). Trigger resolves.
- **Expected Result:** Condition is false at resolution (37 < 40). Ability does nothing (removed from stack).
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

---

### 603.5 — Optional triggered abilities ("may")

**Classification: TESTABLE.**

**ATOM-603.5-001**

- **Rule:** 603.5 — Triggered abilities with "may" go on the stack regardless; the "may" choice is made on resolution.
- **Mechanism:** Optional trigger still stacks
- **Minimal Board:** "At the beginning of your upkeep, you may draw a card." Player's upkeep begins.
- **Action:** Upkeep begins.
- **Expected Result:** The ability triggers and goes on the stack. On resolution, the player chooses whether to draw. The trigger is NOT skipped at trigger time.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

---

### 603.6 — Zone-change triggers

**Classification: TESTABLE.**

> **Implementation note:** Cross-reference META on LKI system (L18). Zone-change trigger resolution depends on LKI infrastructure.

**ATOM-603.6-001**

- **Rule:** 603.6 — Zone-change triggers look for the object in the zone it moved to. If the object can't be found there, that part of the ability fails.
- **Mechanism:** Zone-change trigger resolution finds object in new zone
- **Minimal Board:** "When this creature enters, put a +1/+1 counter on it." Creature enters the battlefield. Before trigger resolves, creature is bounced to hand.
- **Action:** Trigger resolves.
- **Expected Result:** The creature is no longer on the battlefield → the "put a counter on it" part fails. No counter is placed.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.6a — Enters-the-battlefield triggers

**Classification: TESTABLE.** Has an implicit Example via description.

**ATOM-603.6a-001**

- **Rule:** 603.6a — ETB abilities trigger when a permanent enters the battlefield. All permanents (including newcomers) are checked.
- **Mechanism:** ETB trigger detection including newcomer self-trigger
- **Minimal Board:** Soul Warden ("Whenever a creature enters, you gain 1 life") on battlefield. A creature with its own ETB trigger ("When this enters, draw a card") resolves.
- **Action:** Creature enters the battlefield.
- **Expected Result:** Both Soul Warden's trigger AND the entering creature's own ETB trigger fire. Tests the "all permanents including newcomers are checked" clause.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.6b — Continuous effects modify permanent on entry (not before)

**Classification: TESTABLE.** Has an Example.

> **Implementation note:** This is NOT a replacement effect. Continuous effects from the layer system are applied before trigger checking. The permanent never exists on the battlefield without modifications. Implementation: layer recalculation happens synchronously during `move_to_battlefield()`, before `check_triggers()`.

**ATOM-603.6b-001**

- **Rule:** 603.6b — Continuous effects modify a permanent the moment it is on the battlefield. The permanent is never on the battlefield with unmodified characteristics.
- **Mechanism:** Continuous effects apply at the moment of ETB
- **Minimal Board:** "All lands are creatures" effect active. A land card is played.
- **Action:** Land enters the battlefield.
- **Expected Result:** The land is a creature the moment it enters. "Whenever a creature enters" triggers fire.
- **Phase:** Phase 5 Layers + Phase 7 (triggers see post-layer characteristics)
- **Ticket:** Phase 7 triggers, L13 (oracle routing)

**ATOM-603.6b-002**

- **Rule:** 603.6b — Converse: "All creatures lose all abilities" strips ETB abilities at the moment of entry.
- **Mechanism:** Ability removal prevents ETB trigger
- **Minimal Board:** Humility ("All creatures are 1/1 and have no abilities") on battlefield. A creature with an ETB trigger enters.
- **Action:** Creature enters the battlefield.
- **Expected Result:** The creature has no abilities the moment it enters → the ETB ability doesn't trigger.
- **Phase:** Phase 5 Layers (L09 — Layer 6 RemoveAllAbilities) + Phase 7
- **Ticket:** L09, Phase 7 triggers

### 603.6c — Leaves-the-battlefield triggers

**Classification: TESTABLE.**

> **Implementation note:** Multiplayer edge case: player leaves game → all their permanents leave battlefield simultaneously → LTB triggers for each. Extractor Demon example: 10 creatures leave → 10 separate triggers, each requiring target choice via DP.

**ATOM-603.6c-001**

- **Rule:** 603.6c — LTB triggers fire when a permanent moves from battlefield to another zone. The ability checks for the card only in the first zone it went to.
- **Mechanism:** LTB trigger zone tracking
- **Minimal Board:** A permanent has "When this leaves the battlefield, return it to its owner's hand." The permanent is destroyed (goes to graveyard). Before the trigger resolves, the card is exiled from the graveyard.
- **Action:** Trigger resolves.
- **Expected Result:** The trigger looks for the card in the graveyard (first zone it went to). Card is no longer there → the "return to hand" part fails.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

**ATOM-603.6c-002**

- **Rule:** 603.6c — LTB trigger checks for card only in the first zone it went to.
- **Mechanism:** First-zone-only tracking on LTB trigger
- **Minimal Board:** Enduring Renewal ("When a creature is put into your graveyard from the battlefield, return it to your hand"). A creature dies → goes to graveyard. Before trigger resolves, opponent exiles it from graveyard with instant-speed effect.
- **Action:** Trigger resolves → looks in graveyard → card not found.
- **Expected Result:** Trigger does nothing — card is no longer in the first zone it went to.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.6d — "Enters with/as" text is a static ability, not a trigger

**Classification: BOUNDARY-DEF.**

**ATOM-603.6d-001**

- **Rule:** 603.6d — "[This permanent] enters with..." / "As [this permanent] enters..." / "[This permanent] enters tapped" — these are static abilities (not triggered abilities) whose effects are part of the ETB event.
- **Mechanism:** "Enters tapped" is NOT a triggered ability
- **Minimal Board:** A land with "This land enters tapped." Player plays the land.
- **Action:** Land enters the battlefield.
- **Expected Result:** The land enters tapped as part of the ETB event. This is NOT a triggered ability — no ability goes on the stack. The land is tapped immediately.
- **Phase:** Phase 6 (replacement effects — "enters tapped" is a replacement)
- **Ticket:** Phase 6 replacement effects

### 603.6e — Aura LTB triggers can find new objects

**Classification: TESTABLE.**

**ATOM-603.6e-001**

- **Rule:** 603.6e — Some Auras have triggered abilities that trigger on the enchanted permanent leaving. These can find the new object the permanent became AND the new object the Aura became.
- **Mechanism:** Aura LTB trigger cross-zone resolution
- **Minimal Board:** Aura with "When enchanted creature leaves the battlefield, return this Aura to its owner's hand." The enchanted creature is destroyed. The Aura goes to graveyard (SBA for unattached Aura).
- **Action:** Trigger resolves.
- **Expected Result:** The trigger can find the Aura in the graveyard and return it to hand.
- **Phase:** Phase 7 (triggered abilities) + Phase 5 Pre-Work (T15 — Aura SBAs)
- **Ticket:** Phase 7 triggers

**ATOM-603.6e-002**

- **Rule:** 603.6e — Aura's LTB-adjacent trigger can find the object the enchanted creature became.
- **Mechanism:** Aura trigger tracks enchanted creature across zones
- **Minimal Board:** Abduction (Aura, "When enchanted creature dies, return that card to the battlefield under its owner's control."). Enchanted creature dies.
- **Action:** Abduction's trigger resolves.
- **Expected Result:** Trigger finds the creature card in the graveyard (the object the enchanted creature became) and returns it to the battlefield under its owner's control.
- **Phase:** Phase 7 (triggered abilities) + Phase 5 Pre-Work (T15 — Aura SBAs)
- **Ticket:** Phase 7 triggers

---

### 603.7 — Delayed triggered abilities

**Classification: TESTABLE.**

**ATOM-603.7-001**

- **Rule:** 603.7 — An effect may create a delayed triggered ability that fires at a later time. Contains "when," "whenever," or "at."
- **Mechanism:** Delayed trigger creation and storage
- **Minimal Board:** A spell resolves with effect "At the beginning of the next end step, destroy target creature."
- **Action:** Spell resolves, creating a delayed trigger.
- **Expected Result:** A delayed triggered ability is registered. It will trigger at the beginning of the next end step.
- **Phase:** Phase 7 (triggered abilities — delayed triggers)
- **Ticket:** Phase 7 triggers

### 603.7a — Delayed triggers created during resolution, not retroactive

**Classification: TESTABLE.** Has Examples.

> **Implementation note:** Delayed triggers referencing objects that can never satisfy the condition are orphaned data. Potential optimization: lazy cleanup of delayed triggers whose tracked ObjectId has an expired epoch. Not critical for correctness — just memory hygiene. Defer to Phase 8 optimization pass.

**ATOM-603.7a-001**

- **Rule:** 603.7a — A delayed trigger won't trigger until it has actually been created, even if its trigger event occurred just beforehand.
- **Mechanism:** No retroactive delayed triggers
- **Minimal Board:** An effect creates a delayed trigger "When target creature leaves the battlefield, exile it." The creature already left the battlefield before the effect resolved.
- **Action:** The effect resolves and creates the delayed trigger.
- **Expected Result:** The delayed trigger does NOT retroactively fire. The creature is already gone; the trigger waits for the next time the event occurs.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.7b — Delayed triggers fire once (unless duration stated)

**Classification: TESTABLE.**

**ATOM-603.7b-001**

- **Rule:** 603.7b — A delayed trigger triggers only once (the next time its event occurs), unless it has a stated duration like "this turn."
- **Mechanism:** One-shot delayed trigger
- **Minimal Board:** A delayed trigger: "The next time a creature enters the battlefield, draw a card." A creature enters (trigger fires). Then another creature enters.
- **Action:** Second creature enters.
- **Expected Result:** The delayed trigger does NOT fire again — it already fired once and is consumed.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

**ATOM-603.7b-002**

- **Rule:** 603.7b — If trigger event occurs more than once simultaneously, controller chooses which event causes the ability to trigger.
- **Mechanism:** Simultaneous-event delayed trigger choice
- **Minimal Board:** Tatsumasa, the Dragon's Fang (exile self, create token, "Return Tatsumasa when that token dies") + Anointed Procession (token doubler → two tokens created). Both tokens die simultaneously.
- **Action:** Controller chooses which death triggers the delayed ability.
- **Expected Result:** Tatsumasa returns once (one delayed trigger, controller chose which token death caused it).
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.7c — Delayed triggers track specific objects despite characteristic changes

**Classification: TESTABLE.** Has an Example.

> **Implementation note:** This differs from targeting rules. Delayed triggers track by ObjectId, not by characteristics. If ObjectId still exists in expected zone, effect applies regardless of characteristic changes. If ObjectId left its zone (400.7 new object), effect can't find it. This is a simpler system than target legality rechecking.

**ATOM-603.7c-001**

- **Rule:** 603.7c — A delayed trigger referring to a particular object still affects it even if characteristics change. But if the object left its expected zone, the ability won't affect it (rule 400.7 — new object).
- **Mechanism:** Delayed trigger tracks object identity, not characteristics
- **Minimal Board:** An ability: "Exile this creature at the beginning of the next end step." The permanent stops being a creature (type-changing effect) before end step.
- **Action:** End step begins.
- **Expected Result:** The delayed trigger fires and exiles the permanent, even though it's no longer a creature.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.7d — Spell-created delayed trigger: source = spell, controller = spell's controller

**Classification: TESTABLE.**

**ATOM-603.7d-001**

- **Rule:** 603.7d — If a spell creates a delayed trigger, the source is that spell and the controller is whoever controlled the spell as it resolved.
- **Mechanism:** Delayed trigger source/controller from spell
- **Minimal Board:** Player 0 casts a spell that creates a delayed trigger "At the beginning of the next upkeep, each player draws a card."
- **Action:** Delayed trigger fires.
- **Expected Result:** The trigger's controller is Player 0 (who controlled the spell). The trigger's source is the spell.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.7e — Ability-created delayed trigger: source = ability's source

**Classification: TESTABLE.**

**ATOM-603.7e-001**

- **Rule:** 603.7e — If an activated/triggered ability creates a delayed trigger, the source is the same as the source of the creating ability.
- **Mechanism:** Delayed trigger source inheritance from ability
- **Minimal Board:** A permanent's activated ability creates a delayed trigger.
- **Action:** Delayed trigger fires.
- **Expected Result:** The trigger's source is the permanent (same as the activated ability's source).
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.7f — Static ability replacement → delayed trigger: source = object with static

**Classification: TESTABLE.**

**ATOM-603.7f-001**

- **Rule:** 603.7f — If a static ability's replacement effect creates a delayed trigger, the source is the object with the static ability.
- **Mechanism:** Delayed trigger from replacement effect inherits static ability source
- **Minimal Board:** An object has a replacement effect that creates a delayed trigger.
- **Action:** Replacement effect applies, creating the delayed trigger.
- **Expected Result:** Source of the delayed trigger = the object with the static ability. Controller = controller of that object at the time the replacement was applied.
- **Phase:** Phase 7 + Phase 6 (replacement effects)
- **Ticket:** Phase 7 triggers

> **Note:** This test is intentionally abstract. Concrete card examples for this pattern are rare. Revisit and specify more concretely when implementing Phase 6 replacement effects.

### 603.7g — Static ability action → delayed trigger

**Classification: TESTABLE.**

**ATOM-603.7g-001**

- **Rule:** 603.7g — If a static ability allows a player to take an action and creates a delayed trigger if they do, the source is the object with the static ability.
- **Mechanism:** Static-action delayed trigger source/controller
- **Minimal Board:** A permanent has "You may exile a creature card from your graveyard. When you do, create a 2/2 token at the beginning of the next end step."
- **Action:** Player exiles a creature card.
- **Expected Result:** Delayed trigger is created. Source = the permanent. Controller = controller of the permanent when the action was taken.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.7h — Delayed trigger from N-th resolution

**Classification: TESTABLE.**

**ATOM-603.7h-001**

- **Rule:** 603.7h — An ability may create a delayed trigger that fires "when this ability has resolved a certain number of times." The delayed trigger is created only once, at the appropriate resolution.
- **Mechanism:** Resolution-count delayed trigger
- **Minimal Board:** An ability: "Put a +1/+1 counter on this creature. When this ability has resolved three times this turn, sacrifice this creature." First and second resolutions this turn.
- **Action:** Third resolution this turn.
- **Expected Result:** Delayed trigger "sacrifice this creature" is created. It triggers immediately (condition already met).
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

---

### 603.8 — State triggers

**Classification: TESTABLE.** Has an Example.

**ATOM-603.8-001**

- **Rule:** 603.8 — State triggers fire when a game state matches the condition, not on an event. They fire once and don't re-trigger until the ability has left the stack.
- **Mechanism:** State-trigger one-shot until resolution
- **Minimal Board:** A permanent: "Whenever you have no cards in hand, draw a card." Player empties their hand.
- **Action:** State-trigger fires (no cards in hand).
- **Expected Result:** The ability triggers once and goes on the stack. It does NOT trigger again while on the stack, even though the condition is still true. After it resolves (player draws), if hand is still empty, it triggers again.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

> **CORRECTION (from audit):** The CR Example for 603.8 is explicit: state triggers check continuously, NOT only at priority-granting time. If the game state momentarily matches the trigger condition during resolution, the trigger fires. **Architectural implication:** The engine's state-trigger checking cannot be limited to priority checkpoints. It must also run during effect resolution steps — specifically after each discrete game action within a resolving effect (e.g., after the discard step but before the draw step of "discard hand, draw that many"). This is a significant architectural constraint for Phase 7. The `resolve_effect()` pipeline must interleave state-trigger checks between sequential sub-effects.

**ATOM-603.8-002**

- **Rule:** 603.8 — State triggers fire mid-resolution when condition momentarily becomes true.
- **Mechanism:** State trigger fires during spell resolution
- **Minimal Board:** A permanent has "Whenever you have no cards in hand, draw a card." Player has 3 cards in hand.
- **Action:** Player casts "Discard your hand, then draw that many cards" (3 cards discarded, then draw 3).
- **Expected Result:** After the discard step, hand is momentarily empty → state trigger fires and goes on pending triggers queue. Then the draw step draws 3 cards. After resolution, the state trigger is placed on the stack. When it resolves, player draws 1 more card.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

---

### 603.9 — "Loses the game" triggers

**Classification: TESTABLE.**

**ATOM-603.9-001**

- **Rule:** 603.9 — Triggered abilities that trigger when a player loses the game trigger regardless of reason, unless the player leaves due to a draw.
- **Mechanism:** Player-loss trigger
- **Minimal Board:** A permanent: "When a player loses the game, you gain 10 life." Opponent loses (life ≤ 0 SBA).
- **Action:** Opponent loses.
- **Expected Result:** The trigger fires.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

---

### 603.10 — "Look back in time" trigger exceptions

**Classification: PURE-DEF.** Defines the concept of "look back in time." Testable behavior lives in 603.10a–g sub-rules.

### 603.10a — LTB triggers look back in time

**Classification: TESTABLE.**

**ATOM-603.10a-001**

- **Rule:** 603.10a — Leaves-the-battlefield abilities, abilities triggering on cards leaving graveyards, and abilities triggering on visible objects going to hand/library all look back in time.
- **Mechanism:** LTB trigger look-back
- **Minimal Board:** Two creatures and an artifact with "Whenever a creature dies, you gain 1 life" are on the battlefield. A spell destroys all artifacts, creatures, and enchantments.
- **Action:** All three permanents go to the graveyard simultaneously.
- **Expected Result:** The artifact's "whenever a creature dies" ability triggers twice (once per creature), even though the artifact itself is destroyed at the same time. The game looks back to before the event to see the artifact had the ability.

**ATOM-603.10a-002**

- **Rule:** 603.10a — LTB self-trigger look-back.
- **Mechanism:** Death trigger on dying creature
- **Minimal Board:** A creature has "When this creature dies, draw a card." The creature is destroyed.
- **Action:** Creature goes to graveyard.
- **Expected Result:** The death trigger fires even though the creature is no longer on the battlefield — the game looks back to see it had the ability.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.10b — Phase-out triggers look back

**Classification: DEFERRED — Phase 9: Phasing (D1).**

### 603.10c — Unattach triggers look back

**Classification: TESTABLE.**

**ATOM-603.10c-002**

- **Rule:** 603.10c — Equipment "becomes unattached" trigger fires when equipment moves from creature A to creature B.
- **Mechanism:** Re-attach triggers unattach from original
- **Minimal Board:** Equipment with "becomes unattached" trigger. Move equipment from creature A to creature B (via Equip targeting B).
- **Action:** Equip resolves.
- **Expected Result:** Equipment becomes unattached from A → trigger fires. Then attaches to B.
- **Phase:** Phase 7 (triggered abilities) + Phase 5 Pre-Work (T04 — attachment tracking)
- **Ticket:** Phase 7 triggers

**ATOM-603.10c-003**

- **Rule:** 603.10c — "Becomes attached" trigger fires when equipment moves from creature A to a *different* creature B.
- **Mechanism:** Re-attach to different creature triggers "becomes attached" (distinct from 603.2e "becomes" state change — 603.2e tests whether equipping the *same* creature triggers; this tests the cross-creature case)
- **Minimal Board:** Equipment with "Whenever this becomes attached to a creature, put a +1/+1 counter on that creature." Currently attached to creature A.
- **Action:** Equip targeting creature B resolves. Equipment detaches from A, attaches to B.
- **Expected Result:** "Becomes attached" trigger fires for creature B (new attachment). Also, "becomes unattached" trigger from 603.10c-002 fires for creature A. Two separate triggers.
- **Phase:** Phase 7 (triggered abilities) + Phase 5 Pre-Work (T04 — attachment tracking)
- **Ticket:** Phase 7 triggers

**ATOM-603.10c-001**

- **Rule:** 603.10c — Abilities that trigger when an object becomes unattached look back in time.
- **Mechanism:** Unattach trigger look-back
- **Minimal Board:** An Equipment has "Whenever this becomes unattached from a creature, draw a card." The equipped creature is destroyed.
- **Action:** Equipment becomes unattached (creature gone).
- **Expected Result:** The trigger fires, looking back to when the equipment was attached.
- **Phase:** Phase 7 (triggered abilities) + Phase 5 Pre-Work (T04 — attachment tracking)
- **Ticket:** Phase 7 triggers

### 603.10d — Lose-control / opponent-gain-control triggers look back

**Classification: TESTABLE.**

**ATOM-603.10d-001**

- **Rule:** 603.10d — Abilities that trigger when a player loses control or opponent gains control look back in time.
- **Mechanism:** Control-change trigger look-back
- **Minimal Board:** A permanent has "Whenever an opponent gains control of a permanent you own, draw a card." Opponent steals one of your creatures (control change effect).
- **Action:** Control changes.
- **Expected Result:** Trigger fires, looking back to see you had the ability before the control change.
- **Phase:** Phase 7 (triggered abilities) + Phase 5 Layers (L11 — control change)
- **Ticket:** Phase 7 triggers

### 603.10e — "Spell is countered" triggers look back

**Classification: TESTABLE.**

**ATOM-603.10e-001**

- **Rule:** 603.10e — Abilities that trigger when a spell is countered look back in time.
- **Mechanism:** Counter-trigger look-back
- **Minimal Board:** A permanent: "Whenever a spell is countered, you gain 2 life." A Counterspell counters a spell.
- **Action:** Spell is countered.
- **Expected Result:** Trigger fires.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 603.10f — "Player loses the game" triggers look back

**Classification: TESTABLE.** Already covered by 603.9 — the look-back aspect ensures the trigger fires even if the source also disappears.

### 603.10g — "Planeswalks away from a plane" triggers look back

**Classification: OUT-OF-SCOPE.** Planechase is permanently excluded.

---

### 603.11 — Static ability linked to triggered ability

**Classification: PURE-DEF (reference to rule 607).** No independent mechanical consequence; tested via 607 linked abilities.

### 603.12 — Reflexive triggered abilities

**Classification: TESTABLE.** Has an Example.

**ATOM-603.12-001**

- **Rule:** 603.12 — A resolving ability may create a reflexive triggered ability ("When you do, [effect]") that triggers immediately based on actions taken during that resolution.
- **Mechanism:** Reflexive trigger created and checked during resolution
- **Minimal Board:** Heart-Piercer Manticore: "When this enters, you may sacrifice another creature. When you do, this deals damage equal to that creature's power to any target."
- **Action:** Manticore enters. Player sacrifices a 3-power creature.
- **Expected Result:** Reflexive trigger fires ("When you do"). Player chooses a target. 3 damage dealt.
- **Phase:** Phase 7 (triggered abilities — reflexive triggers)
- **Ticket:** Phase 7 triggers

### 603.12a — Reflexive trigger for "pay cost any number of times"

**Classification: TESTABLE.**

**ATOM-603.12a-001**

- **Rule:** 603.12a — If a reflexive trigger is tied to paying a cost "any number of times" and triggers "when you pay one or more times," it triggers only once regardless of how many times the cost was paid.
- **Mechanism:** Reflexive trigger coalesces multiple payments
- **Minimal Board:** An ability: "You may pay {1} any number of times. When you pay {1} one or more times this way, draw that many cards."
- **Action:** Player pays {1} three times.
- **Expected Result:** The reflexive trigger fires once (not three times). The effect draws 3 cards (referencing the total).
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

---

## Rule 604 — Handling Static Abilities

### 604.1 — Static abilities do something all the time

**Classification: PURE-DEF.** Defines what static abilities are — they are "simply true." No independent testable consequence; the layer system (Phase 5) implements this.

### 604.2 — Static abilities create continuous effects

**Classification: BOUNDARY-DEF.** Links static abilities to continuous effects, including prevention and replacement effects.

**ATOM-604.2-001**

- **Rule:** 604.2 — Static abilities create continuous effects. These effects are active as long as the permanent with the ability remains on the battlefield and has the ability.
- **Mechanism:** Static ability effect lifetime = source lifetime on battlefield
- **Minimal Board:** Glorious Anthem ("Creatures you control get +1/+1") on battlefield. Player controls a 2/2 creature.
- **Action:** Creature has 3/3 effective stats. Anthem is destroyed.
- **Expected Result:** After Anthem leaves, creature reverts to 2/2. The continuous effect ends when the source leaves.
- **Phase:** Phase 5 Layers (L08 — static ability registration + removal)
- **Ticket:** L08

### 604.3 — Characteristic-defining abilities (CDAs)

**Classification: BOUNDARY-DEF.** Defines CDAs and establishes that they function in all zones.

**ATOM-604.3-001**

- **Rule:** 604.3 — A characteristic-defining ability conveys information about an object's characteristics. CDAs function in all zones and outside the game.
- **Mechanism:** CDA applies in all zones (not just battlefield)
- **Minimal Board:** Tarmogoyf in graveyard. Graveyard contains creature and instant card types.
- **Action:** Query Tarmogoyf's power/toughness while in graveyard.
- **Expected Result:** Tarmogoyf's power is 2 and toughness is 3 (2 card types + 1). The CDA applies even in the graveyard.
- **Phase:** Phase 5 Layers (L04 — compute_characteristics CDA all-zone handling, PC3/W6)
- **Ticket:** L04

### 604.3a — CDA criteria (5 conditions)

**Classification: BOUNDARY-DEF.** Defines the five criteria for a static ability to be a CDA. This is a classification rule used at card-data authoring time (`is_cda: bool` on AbilityDef).

**ATOM-604.3a-001**

- **Rule:** 604.3a — A static ability is a CDA if: (1) defines colors/subtypes/P/T; (2) printed on the card it affects; (3) doesn't affect other objects; (4) not a self-grant; (5) not conditional.
- **Mechanism:** CDA classification validation
- **Minimal Board:** Tarmogoyf has `is_cda: true` on its P/T ability. Glorious Anthem has `is_cda: false` (affects other objects, violating criterion 3).
- **Action:** Verify CDA flag on card data.
- **Expected Result:** Tarmogoyf's P/T ability is CDA. Anthem's +1/+1 is NOT CDA.
- **Phase:** Phase 5 Layers (L01 — `is_cda` on AbilityDef, PC1)
- **Ticket:** L01

### 604.4 — Aura/Equipment/Fortification static abilities modify attached object

**Classification: BOUNDARY-DEF.**

**ATOM-604.4-001**

- **Rule:** 604.4 — Many Auras, Equipment, and Fortifications have static abilities that modify the attached object. These don't target. If moved to a different object, the ability stops affecting the old and starts affecting the new.
- **Mechanism:** Attachment-based static effect re-targeting
- **Minimal Board:** An Equipment with "Equipped creature gets +2/+0" is attached to creature A. Equipment is moved to creature B.
- **Action:** Equipment moves from A to B.
- **Expected Result:** A loses +2/+0. B gains +2/+0.
- **Phase:** Phase 5 Pre-Work (T04 — attachment tracking) + Phase 5 Layers (L08 — static registration)
- **Ticket:** T04, L08

### 604.5 — Static abilities that apply on the stack

**Classification: BOUNDARY-DEF.**

**ATOM-604.5-001**

- **Rule:** 604.5 — Some static abilities apply while a spell is on the stack: abilities that counter, "As an additional cost to cast," "You may pay [cost] rather than pay mana cost," and "You may cast without paying mana cost."
- **Mechanism:** Stack-zone static abilities
- **Minimal Board:** A spell has "As an additional cost to cast this spell, sacrifice a creature." The spell is on the stack.
- **Action:** During casting (601.2b), the additional cost is announced.
- **Expected Result:** The static ability operates while the spell is on the stack — the additional cost is part of the total cost.
- **Phase:** Phase 5 Pre-Work (T17 — alt/add cost types, T18 — 601.2 pipeline)
- **Ticket:** T17, T18

**ATOM-604.5-002**

- **Rule:** 604.5 — "You may cast without paying mana cost" as a stack-zone static ability.
- **Mechanism:** Stack-zone static ability for alternative cost
- **Minimal Board:** A card has "You may cast this spell without paying its mana cost if a creature died this turn." A creature died this turn. The spell is on the stack.
- **Action:** During casting (601.2b), the alternative cost is chosen.
- **Expected Result:** The static ability applies while the spell is on the stack — the alternative cost (free cast) is available.
- **Phase:** Phase 5 Pre-Work (T17 — alt cost types, T18 — 601.2 pipeline)
- **Ticket:** T17, T18

### 604.6 — Static abilities that apply from castable zones

**Classification: BOUNDARY-DEF.**

**ATOM-604.6-001**

- **Rule:** 604.6 — Some static abilities apply while a card is in a zone you could cast/play it from (usually hand): "You may cast [this card]...," "You can't cast [this card]...," "Cast [this card] only..."
- **Mechanism:** Hand-zone static abilities for cast permissions
- **Minimal Board:** A card in hand has "Cast this spell only during combat." It is the main phase.
- **Action:** Player attempts to cast the spell.
- **Expected Result:** Illegal — the static ability restricts casting to combat only, and it applies from hand.
- **Phase:** Phase 5 Pre-Work (T18 — casting restrictions from card text)
- **Ticket:** T18

**ATOM-604.6-002**

- **Rule:** 604.6 — "Cast only during combat" restriction applies from hand zone.
- **Mechanism:** Hand-zone static ability restricts casting timing
- **Minimal Board:** A card in hand has "Cast this spell only during combat." It is the combat phase.
- **Action:** Player casts the spell.
- **Expected Result:** Legal — the static ability's condition (combat) is met.
- **Phase:** Phase 5 Pre-Work (T18 — casting restrictions from card text)
- **Ticket:** T18

### 604.7 — Static abilities can't use last known information

**Classification: BOUNDARY-DEF.**

**ATOM-604.7-001**

- **Rule:** 604.7 — Unlike spells and other abilities, static abilities can't use an object's last known information for determining how their effects are applied.
- **Mechanism:** Static ability CDA fails when source is gone (no LKI)
- **Minimal Board:** Saproling Burst (fading 7, activated ability creates token whose P/T = fade counters on Burst). Activate Saproling Burst's ability. Before it resolves, Burst is destroyed.
- **Action:** Token enters the battlefield. Token's CDA references "number of fade counters on Saproling Burst."
- **Expected Result:** Burst is gone. Static ability can't use LKI → token is 0/0 → dies to SBA (toughness ≤ 0).
- **Phase:** Phase 5 Layers (L18 — LKI system, C19 exclusion)
- **Ticket:** L18

> **Cross-reference:** Saproling Burst is also a good example for 607.1d (linked abilities across multiple objects) — the token's CDA references data from a different object (the Burst permanent). When Burst is gone, the cross-object link breaks.

---

## Rule 605 — Mana Abilities

### 605.1 — Mana ability criteria

**Classification: BOUNDARY-DEF.** Defines the two sets of criteria for mana abilities. Critical for engine routing (mana abilities bypass the stack).

### 605.1a — Activated mana ability criteria

**Classification: TESTABLE.**

**ATOM-605.1a-001**

- **Rule:** 605.1a — An activated ability is a mana ability if: (1) doesn't require a target, (2) could add mana to a player's mana pool when it resolves, and (3) is not a loyalty ability.
- **Mechanism:** Mana ability classification for activated abilities
- **Minimal Board:** A permanent has "{T}: Add {G}." — no target, adds mana, not loyalty.
- **Action:** Classify the ability.
- **Expected Result:** It is a mana ability. It resolves immediately (doesn't use the stack).
- **Phase:** ALREADY-IMPLEMENTED (priority.rs routes mana abilities to activate_mana_ability)
- **Ticket:** N/A

**ATOM-605.1a-002**

- **Rule:** 605.1a — An activated ability that requires a target is NOT a mana ability, even if it could produce mana.
- **Mechanism:** Target disqualifies mana ability classification
- **Minimal Board:** A permanent has "{T}: Target player adds {G}." — has a target.
- **Action:** Classify the ability.
- **Expected Result:** It is NOT a mana ability. It uses the stack.
- **Phase:** Phase 5 Pre-Work (ability classification refinement)
- **Ticket:** NEW — "Mana ability classification: target check"

**ATOM-605.1a-003**

- **Rule:** 605.1a — A loyalty ability that adds mana is NOT a mana ability.
- **Mechanism:** Loyalty ability disqualifies mana ability classification
- **Minimal Board:** Chandra, Torch of Defiance (+1: "Add {R}{R}"). Satisfies (1) no target and (2) adds mana, but is a loyalty ability.
- **Action:** Classify the ability.
- **Expected Result:** NOT a mana ability. Uses the stack.
- **Phase:** Phase 8 (Planeswalkers)
- **Ticket:** Phase 8 planeswalkers

**ATOM-605.1a-004**

- **Rule:** 605.1a — A spell/ability with targets is NOT a mana ability despite adding mana.
- **Mechanism:** Target disqualifies mana ability even with mana production
- **Minimal Board:** Explosive Welcome ({7}{R}, "Deals 5 to target, 3 to another target. Add {R}{R}{R}."). Has targets.
- **Action:** Classify the ability.
- **Expected Result:** NOT a mana ability. Uses the stack.
- **Phase:** Phase 5 Pre-Work (ability classification refinement)
- **Ticket:** NEW — "Mana ability classification: target check"

> **Implementation note (605.1a/1b):** Triggered vs activated mana ability handling: classification is the same system (check 3 criteria). Resolution differs (activated = immediate, triggered = immediate after triggering mana ability). One set of classification tests covers both; resolution tests are separate.

### 605.1b — Triggered mana ability criteria

**Classification: TESTABLE.**

**ATOM-605.1b-001**

- **Rule:** 605.1b — A triggered ability is a mana ability if: (1) doesn't require a target, (2) triggers from activation/resolution of an activated mana ability or from mana being added, and (3) could add mana when it resolves.
- **Mechanism:** Triggered mana ability classification
- **Minimal Board:** An enchantment has "Whenever a player taps a land for mana, that player adds one mana of any type that land produced." — no target, triggers from mana ability, adds mana.
- **Action:** A player taps a Forest for {G}.
- **Expected Result:** The triggered ability is a mana ability. It resolves immediately after the Forest's ability, without using the stack.
- **Phase:** Phase 7 (triggered abilities — mana trigger special case)
- **Ticket:** Phase 7 triggers

### 605.2 — Mana ability remains a mana ability regardless of game state

**Classification: TESTABLE.** Has an Example.

**ATOM-605.2-001**

- **Rule:** 605.2 — A mana ability remains a mana ability even if the game state doesn't allow it to produce mana.
- **Mechanism:** Mana ability classification is static, not state-dependent
- **Minimal Board:** A permanent has "{T}: Add {G} for each creature you control." Player controls no creatures.
- **Action:** Player attempts to activate the ability.
- **Expected Result:** The ability IS still a mana ability (resolves immediately, no stack). It resolves and adds {G} × 0 = no mana. The classification doesn't change based on expected output.
- **Phase:** ALREADY-IMPLEMENTED (ability classification is based on card data, not game state)
- **Ticket:** N/A

### 605.3 — Activated mana ability special rules

**Classification: PURE-DEF (header for 605.3a–c).** No independent mechanical consequence.

### 605.3a — Mana ability timing: any time a mana payment is needed

**Classification: TESTABLE.**

**ATOM-605.3a-001**

- **Rule:** 605.3a — A player may activate an activated mana ability whenever they have priority, whenever they are casting a spell or activating an ability that requires mana, or whenever a rule or effect asks for a mana payment.
- **Mechanism:** Mana ability activation window during casting
- **Minimal Board:** Player is in the middle of casting a spell (601.2g mana ability window). Player controls untapped lands.
- **Action:** Player taps a land for mana during the mana ability window.
- **Expected Result:** Legal — mana abilities can be activated during the casting process, not just during priority.
- **Phase:** ALREADY-IMPLEMENTED (priority.rs + mana ability window in cast flow)
- **Ticket:** N/A

**ATOM-605.3a-002**

- **Rule:** 605.3a — Mana ability activation when a rule asks for mana payment (not just casting/activating).
- **Mechanism:** Mana ability window during rule-required mana payment
- **Minimal Board:** Rhystic Study ("Whenever an opponent casts a spell, you may draw a card unless that player pays {1}"). Opponent casts a spell. Opponent has no mana in pool but controls an untapped land.
- **Action:** Opponent activates a mana ability during the "unless they pay" window.
- **Expected Result:** Legal — mana abilities can be activated when a rule or effect asks for mana payment.
- **Phase:** Phase 7 (triggered abilities — mana payment windows)
- **Ticket:** Phase 7 triggers

**ATOM-605.3a-003**

- **Rule:** 605.3a — Mana ability activation during ability activation that requires mana.
- **Mechanism:** Mana ability window during ability activation cost payment
- **Minimal Board:** Player controls a permanent with "{2}, {T}: Draw a card." Player has no mana in pool but controls untapped lands.
- **Action:** Player begins activating the ability, then activates mana abilities to pay the {2} cost.
- **Expected Result:** Legal — mana abilities can be activated during ability activation (not just spell casting).
- **Phase:** ALREADY-IMPLEMENTED (same window as spell casting)
- **Ticket:** N/A

### 605.3b — Mana abilities don't use the stack

**Classification: TESTABLE.**

**ATOM-605.3b-001**

- **Rule:** 605.3b — An activated mana ability doesn't go on the stack. It can't be targeted, countered, or responded to. It resolves immediately.
- **Mechanism:** Mana ability immediate resolution
- **Minimal Board:** Player taps a Forest for {G}. Opponent has a Stifle in hand.
- **Action:** Player activates Forest's mana ability.
- **Expected Result:** The mana ability resolves immediately. {G} is added to pool. Opponent cannot counter or respond — the ability never goes on the stack.
- **Phase:** ALREADY-IMPLEMENTED (engine/mana.rs activate_mana_ability resolves immediately)
- **Ticket:** N/A

### 605.3c — No re-activation until resolved

**Classification: TESTABLE.**

**ATOM-605.3c-001**

- **Rule:** 605.3c — Once a player begins to activate a mana ability, that ability can't be activated again until it has resolved.
- **Mechanism:** Mana ability re-activation lock (non-tap cost)
- **Minimal Board:** Player controls a permanent with "{1}: Add one mana of any color." (mana-filtering ability, no tap cost). Player begins activating it.
- **Action:** Player attempts to activate the same ability again before the first activation resolves.
- **Expected Result:** Illegal — the ability is mid-resolution. Engine prevents re-activation until the first activation resolves. (Note: tap-cost mana abilities inherently prevent re-activation since the permanent is tapped. This test uses a non-tap cost to exercise the rule independently.)
- **Phase:** ALREADY-IMPLEMENTED (mana abilities resolve synchronously — no window for re-activation)
- **Ticket:** N/A

> **Structural guard note:** The engine resolves mana abilities synchronously (no stack, immediate resolution). This means there is no window during which a player could attempt to re-activate the same ability before it resolves — the rule is enforced structurally by the architecture, not by an explicit guard. This test may be **untestable in practice** since the violation scenario cannot arise. Keep spec for documentation but consider marking as STRUCTURAL-GUARD rather than a runtime test.

### 605.4 — Triggered mana abilities: follow triggered ability rules with exception

**Classification: PURE-DEF (header for 605.4a).** No independent mechanical consequence.

### 605.4a — Triggered mana abilities don't use the stack

**Classification: TESTABLE.** Has an Example.

**ATOM-605.4a-001**

- **Rule:** 605.4a — A triggered mana ability doesn't go on the stack. It resolves immediately after the mana ability that triggered it, without waiting for priority.
- **Mechanism:** Triggered mana ability immediate resolution
- **Minimal Board:** An enchantment: "Whenever a player taps a land for mana, that player adds one additional mana of any type that land produced." Player taps an Island for {U}.
- **Action:** Island's mana ability resolves ({U} added). The triggered mana ability fires.
- **Expected Result:** The triggered ability resolves immediately — player gets an additional {U} (or another type the Island could produce). No stack involvement.
- **Phase:** Phase 7 (triggered abilities — triggered mana ability special case)
- **Ticket:** Phase 7 triggers

### 605.5 — Non-mana abilities that look like mana abilities

**Classification: BOUNDARY-DEF (header for 605.5a–b).** Defines what is NOT a mana ability.

### 605.5a — Targeted "mana" abilities are not mana abilities

**Classification: TESTABLE.**

**ATOM-605.5a-001**

- **Rule:** 605.5a — An ability with a target is not a mana ability, even if it could put mana into a pool. Same for triggered abilities that could produce mana but trigger from non-mana events.
- **Mechanism:** Target disqualifies mana ability; wrong trigger source disqualifies
- **Minimal Board:** A permanent has "Whenever you cast a spell, add {G}." — triggers from casting (not from mana ability activation). It could add mana.
- **Action:** Classify this triggered ability.
- **Expected Result:** NOT a mana ability — it triggers from a non-mana event (casting a spell, not activating a mana ability or adding mana). It uses the stack.
- **Phase:** Phase 7 (triggered abilities — ability classification)
- **Ticket:** Phase 7 triggers

### 605.5b — Spells are never mana abilities

**Classification: BOUNDARY-DEF.**

**ATOM-605.5b-001**

- **Rule:** 605.5b — A spell can never be a mana ability, even if it could add mana. Old "mana source" cards are now instants.
- **Mechanism:** Spells always use the stack regardless of mana output
- **Minimal Board:** Player casts Dark Ritual (instant: "Add {B}{B}{B}").
- **Action:** Dark Ritual is cast.
- **Expected Result:** Dark Ritual goes on the stack as a spell. It can be countered. It is NOT a mana ability.
- **Phase:** ALREADY-IMPLEMENTED (all spells go on stack)
- **Ticket:** N/A

---

## Rule 606 — Loyalty Abilities

### 606.1 — Some activated abilities are loyalty abilities

**Classification: PURE-DEF.** Names the concept.

### 606.2 — Loyalty symbol in cost = loyalty ability

**Classification: BOUNDARY-DEF.**

**ATOM-606.2-001**

- **Rule:** 606.2 — An activated ability with a loyalty symbol in its cost is a loyalty ability. Normally, only planeswalkers have loyalty abilities.
- **Mechanism:** Loyalty ability classification
- **Minimal Board:** A planeswalker has abilities with +1, -2, -7 loyalty costs.
- **Action:** Classify the abilities.
- **Expected Result:** All three are loyalty abilities.
- **Phase:** DEFERRED — Phase 8 (Planeswalkers)
- **Ticket:** DEFERRED — Phase 8: Planeswalkers

### 606.3 — Loyalty ability timing: sorcery speed + once per turn

**Classification: TESTABLE.**

**ATOM-606.3-001**

- **Rule:** 606.3 — A loyalty ability can only be activated at sorcery speed (main phase, stack empty, controller has priority) AND only if no loyalty ability of that permanent has been activated that turn.
- **Mechanism:** Loyalty ability timing + once-per-turn restriction
- **Minimal Board:** Player controls a planeswalker with two loyalty abilities. It is their main phase, stack empty.
- **Action:** Player activates the first loyalty ability. Then attempts to activate the second.
- **Expected Result:** First activation: legal. Second activation: illegal — a loyalty ability of that permanent was already activated this turn.
- **Phase:** DEFERRED — Phase 8 (Planeswalkers)
- **Ticket:** DEFERRED — Phase 8: Planeswalkers

**ATOM-606.3-002**

- **Rule:** 606.3 — Loyalty ability at instant speed is illegal.
- **Mechanism:** Loyalty ability sorcery-speed enforcement
- **Minimal Board:** Player controls a planeswalker. An opponent's spell is on the stack.
- **Action:** Player attempts to activate a loyalty ability.
- **Expected Result:** Illegal — stack is not empty.
- **Phase:** DEFERRED — Phase 8 (Planeswalkers)
- **Ticket:** DEFERRED — Phase 8: Planeswalkers

### 606.4 — Loyalty counter payment as cost

**Classification: TESTABLE.**

**ATOM-606.4-001**

- **Rule:** 606.4 — The cost to activate a loyalty ability is to put on or remove loyalty counters as shown by the loyalty symbol.
- **Mechanism:** Loyalty counter cost payment
- **Minimal Board:** A planeswalker with 4 loyalty counters. It has a -2 ability.
- **Action:** Player activates the -2 ability.
- **Expected Result:** 2 loyalty counters are removed. Planeswalker now has 2 loyalty counters.
- **Phase:** DEFERRED — Phase 8 (Planeswalkers)
- **Ticket:** DEFERRED — Phase 8: Planeswalkers

> **Composition test note:** Counter doubling (Doubling Season) + planeswalker +1 ability → +2 loyalty. Good integration test for Phase 8 planeswalker implementation.

### 606.5 — Multiple loyalty costs combined

**Classification: TESTABLE.** Has an Example.

**ATOM-606.5-001**

- **Rule:** 606.5 — If the total cost contains multiple loyalty counter add/remove costs, they are combined into a single net cost.
- **Mechanism:** Loyalty cost combination
- **Minimal Board:** Player controls Carth the Lion ("Planeswalkers' loyalty abilities you control cost an additional [+1] to activate.") and a planeswalker with a -4 ability. Planeswalker has 3 loyalty counters.
- **Action:** Player activates the -4 ability with Carth's +1 modifier.
- **Expected Result:** Net cost = -4 + 1 = -3. Remove 3 loyalty counters. Planeswalker has 0 loyalty counters. (Without Carth, the -4 ability couldn't be activated with only 3 counters.)
- **Phase:** DEFERRED — Phase 8 (Planeswalkers)
- **Ticket:** DEFERRED — Phase 8: Planeswalkers

### 606.6 — Negative loyalty cost requires sufficient counters

**Classification: TESTABLE.**

> **Implementation note:** Edge case: planeswalker with 3 loyalty, -4 ability, but a Carth the Lion effect adds +1 to loyalty costs → net cost is -3. Now affordable. Tests 606.5 + 606.6 interaction.

**ATOM-606.6-001**

- **Rule:** 606.6 — A loyalty ability with a negative loyalty cost (after modifications) can't be activated unless the permanent has at least that many loyalty counters.
- **Mechanism:** Loyalty counter floor check
- **Minimal Board:** A planeswalker with 2 loyalty counters. It has a -3 ability.
- **Action:** Player attempts to activate the -3 ability.
- **Expected Result:** Illegal — planeswalker has only 2 counters, needs at least 3.
- **Phase:** DEFERRED — Phase 8 (Planeswalkers)
- **Ticket:** DEFERRED — Phase 8: Planeswalkers

---

## Rule 607 — Linked Abilities

### 607.1 — Definition of linked abilities

**Classification: BOUNDARY-DEF.** Defines that two printed abilities can be linked: one causes actions/affects objects, the other refers to those actions/objects. The second refers ONLY to the first, not to any other ability.

> **META-LINKED-ABILITY-STORAGE:** Engine needs per-permanent storage for linked ability data: `linked_data: HashMap<AbilityId, LinkedAbilityData>` on BattlefieldEntity or GameObject. LinkedAbilityData stores exiled ObjectIds, chosen values, noted information, paid costs, etc. Each ability that writes data tags it with its AbilityId; the reading ability looks up only its linked AbilityId's data.

**ATOM-607.1-001**

- **Rule:** 607.1 — If two abilities are linked, the second refers only to actions taken or objects affected by the first, and not by any other ability.
- **Mechanism:** Linked ability scoping — second ability reads only first ability's data
- **Minimal Board:** A permanent has "When this enters, exile target creature" (ability A) and "When this leaves the battlefield, return the exiled card to the battlefield" (ability B). A and B are linked.
- **Action:** Ability A exiles creature X. Separately, another effect also exiles creature Y referencing the same permanent. Permanent leaves → ability B triggers.
- **Expected Result:** Ability B returns ONLY creature X (exiled by ability A). Creature Y (exiled by a different source) is NOT returned.
- **Phase:** Phase 5 Pre-Work (T20 — linked abilities)
- **Ticket:** T20

### 607.1a — Granted ability is "printed on"

**Classification: PURE-DEF.** Clarifies that an ability printed within a grant clause counts as "printed on" the object. No independent test needed — relevant to 607.2 subtypes.

### 607.1b — Double-faced card abilities are "printed on"

**Classification: PURE-DEF.** Both faces' abilities count. Relevant to Phase 9 (DFCs).

### 607.1c — Self-linked ability

**Classification: TESTABLE.**

**ATOM-607.1c-001**

- **Rule:** 607.1c — An ability that fulfills both criteria (causes actions AND refers to those actions) is linked to itself.
- **Mechanism:** Self-linked ability (single ability causes action AND references result)
- **Minimal Board:** Tyrant's Choice ({1}{B} Sorcery, "Will of the council — Starting with you, each player votes for death or torture. If death gets more votes, each opponent sacrifices a creature. If torture gets more votes or tied, each opponent loses 4 life."). Single ability causes the vote (action) and references the vote result (self-linked).
- **Action:** Cast Tyrant's Choice. Both players vote.
- **Expected Result:** If death wins: each opponent sacrifices. If torture wins/ties: each opponent loses 4 life. The vote-causing and vote-reading are within a single ability — self-linked.
- **Phase:** Phase 5 Pre-Work (T20 — linked abilities)
- **Ticket:** T20

### 607.1d — Cross-object linked abilities (token/emblem sources)

**Classification: TESTABLE.**

**ATOM-607.1d-001**

- **Rule:** 607.1d — Abilities on two objects can be linked if one is a token/emblem/permanent and the second was the source that created/put it onto the battlefield.
- **Mechanism:** Cross-object linked abilities
- **Minimal Board:** A spell creates a token with "This token has 'When this leaves the battlefield, return the exiled card.'" The spell also exiled a card as part of creating the token.
- **Action:** Token leaves the battlefield.
- **Expected Result:** The token's LTB ability refers to the card exiled by the spell that created it. The link spans two objects.
- **Phase:** DEFERRED — Phase 8 (token creation + linked abilities)
- **Ticket:** DEFERRED — Phase 8

> **Deferral note:** Audit recommends deferring concrete implementation. No clear card examples found that strictly require cross-object linking in early phases. The architectural change is an exception (cross-object linking), not a fundamental engine change. Low retrofit cost. Defer to Phase 8 when concrete cards exercise this rule.

---

### 607.2 — Kinds of linked abilities

**Classification: PURE-DEF (header).** Enumerates the different linking patterns. Each sub-rule below is classified independently.

### 607.2a — Exile + "exiled cards" link

**Classification: TESTABLE.** See also: Isochron Scepter (imprint + cast copy).

**ATOM-607.2a-001**

- **Rule:** 607.2a — If an object has an ability that exiles cards AND an ability that refers to "the exiled cards" or cards "exiled with [this object]," they are linked.
- **Mechanism:** Exile-reference linking (O-Ring pattern)
- **Minimal Board:** Banishing Light: "When this enters, exile target nonland permanent an opponent controls until this leaves the battlefield."
- **Action:** ETB exiles creature A. Banishing Light leaves → LTB trigger.
- **Expected Result:** LTB returns creature A specifically — it was exiled by the linked ETB ability.
- **Phase:** Phase 5 Pre-Work (T20 — linked abilities)
- **Ticket:** T20

**ATOM-607.2a-002**

- **Rule:** 607.2a — Object with two different exile abilities, each linked to a separate "exiled cards" reference.
- **Mechanism:** Per-ability exile tracking (two independent linked pairs)
- **Minimal Board:** A permanent has ability A ("When this enters, exile target creature") and ability C ("When this attacks, exile top card of defending player's library"). It also has ability B ("When this leaves, return all creatures exiled with this") linked to A, and ability D ("You may play cards exiled with this") linked to C.
- **Action:** Ability A exiles creature X. Ability C exiles card Y. Permanent leaves.
- **Expected Result:** Ability B returns creature X only. Ability D allows playing card Y only. The two exile pools are independent.
- **Phase:** Phase 5 Pre-Work (T20 — linked abilities)
- **Ticket:** T20

### 607.2b — Replacement exile + "exiled cards" link

**Classification: TESTABLE.** See also: Shared Fate (replacement exile + play from exile).

**ATOM-607.2b-001**

- **Rule:** 607.2b — If an object has a replacement effect that exiles cards AND an ability referring to "the exiled cards," they are linked.
- **Mechanism:** Replacement-exile linking
- **Minimal Board:** A permanent has "If a creature would die, exile it instead" and "You may cast cards exiled with this."
- **Action:** A creature dies → exiled by replacement. Player casts the exiled card.
- **Expected Result:** The second ability sees only cards exiled by the replacement effect, not cards exiled by other means.
- **Phase:** Phase 6 (replacement effects) + T20 (linked abilities)
- **Ticket:** T20, Phase 6

### 607.2c — ETB + "put onto the battlefield with" link

**Classification: TESTABLE.** See also: Diabolic Servitude (ETB + "put onto battlefield with" + LTB).

**ATOM-607.2c-001**

- **Rule:** 607.2c — Ability that puts objects onto the battlefield + ability that refers to objects "put onto the battlefield with [this object]" are linked.
- **Mechanism:** ETB-creation linking
- **Minimal Board:** A permanent has "When this enters, create two 1/1 tokens" and "Tokens put onto the battlefield with this have flying."
- **Action:** ETB triggers, creates tokens.
- **Expected Result:** The two tokens created by the ETB get flying. Tokens created by other means do not.
- **Phase:** Phase 5 Pre-Work (T20) + Phase 7 (triggered abilities)
- **Ticket:** T20

### 607.2d — "Choose a value" + "the chosen value" link

**Classification: TESTABLE.** See also: Haktos the Unscarred (random choice + protection from non-chosen), True-Name Nemesis (choose a player + protection from that player).

**ATOM-607.2d-001**

- **Rule:** 607.2d — Ability that causes "choose a [value]" + ability referring to "the chosen [value]" are linked.
- **Mechanism:** Choice-value linking
- **Minimal Board:** Voice of All: "As this enters, choose a color." + "This creature has protection from the chosen color."
- **Action:** Player chooses blue on ETB.
- **Expected Result:** The creature has protection from blue. The second ability reads the choice made by the first.
- **Phase:** Phase 5 Pre-Work (T20 — linked abilities, choice storage)
- **Ticket:** T20

**ATOM-607.2d-002**

- **Rule:** 607.2d — Choice persists across zone changes if ability specifies.
- **Mechanism:** Choice-value persistence through zone change
- **Minimal Board:** Cavern of Souls: "As this enters, choose a creature type." + "{T}: Add {C}. Spend this mana only to cast a creature spell of the chosen type, and it can't be countered." Player chooses "Elf."
- **Action:** Player taps Cavern for mana, casts an Elf creature spell.
- **Expected Result:** The spell can't be countered. The chosen type "Elf" persists on the permanent.
- **Phase:** Phase 5 Pre-Work (T20 — linked abilities, choice storage)
- **Ticket:** T20

### 607.2e — "Note" + "noted information" link

**Classification: TESTABLE.** See also: Meddling Mage (note card name + named card can't be cast).

**ATOM-607.2e-001**

- **Rule:** 607.2e — Ability that notes information + ability that refers to noted information are linked.
- **Mechanism:** Noted-information linking
- **Minimal Board:** A permanent: "As this enters, note target creature's power." + "This creature's power is equal to the noted value."
- **Action:** Target creature has power 5. Permanent enters, noting 5.
- **Expected Result:** The permanent's power is 5 (the noted value).
- **Phase:** Phase 8 (rare mechanic)
- **Ticket:** DEFERRED — Phase 8

### 607.2f — "Choose a word" + "the chosen word" link

**Classification: TESTABLE.** See also: Archangel of Strife (war/peace word choice).

**ATOM-607.2f-001**

- **Rule:** 607.2f — Ability that causes choosing from words with no rules meaning + ability referring to that choice are linked.
- **Mechanism:** Word-choice linking
- **Minimal Board:** A permanent: "As this enters, choose 'fire' or 'ice'." + "If the chosen word is 'fire,' this has first strike."
- **Action:** Player chooses "fire" on ETB.
- **Expected Result:** Permanent has first strike.
- **Phase:** Phase 8 (rare mechanic)
- **Ticket:** DEFERRED — Phase 8

### 607.2g — "Pay cost as it enters" + "cost paid as entered" link

**Classification: TESTABLE.** See also: Phyrexian Processor (pay life as enters + create X/X token).

**ATOM-607.2g-001**

- **Rule:** 607.2g — Ability that causes paying a cost as it enters + ability referring to "the cost paid as [this] entered" are linked.
- **Mechanism:** ETB-cost linking
- **Minimal Board:** A permanent: "As this enters, you may pay {2}." + "If the cost was paid, this enters with two +1/+1 counters."
- **Action:** Player pays {2} as the permanent enters.
- **Expected Result:** Permanent enters with two +1/+1 counters.
- **Phase:** Phase 6 (replacement effects — ETB cost/counter interaction)
- **Ticket:** Phase 6

### 607.2h — Static + triggered in same paragraph

**Classification: TESTABLE.** See also: Keranos, God of Storms (static reveal + triggered abilities in same paragraph).

**ATOM-607.2h-001**

- **Rule:** 607.2h — A static ability and triggered abilities in the same paragraph are linked. Triggered abilities refer only to actions from the static ability. See 603.11.
- **Mechanism:** Same-paragraph static+triggered linking
- **Minimal Board:** A permanent: "Enchanted creature has 'Whenever this creature deals damage, you gain that much life.'" — the triggered ability is linked to the static ability that grants it.
- **Action:** The enchanted creature deals 3 damage.
- **Expected Result:** Controller gains 3 life. The triggered ability refers to the static ability's context.
- **Phase:** Phase 5 Pre-Work (T20) + Phase 7
- **Ticket:** T20

### 607.2i — Additional cost + "was this cost paid" link

**Classification: TESTABLE.** Has an Example (kicker).

**ATOM-607.2i-001**

- **Rule:** 607.2i — Ability allowing an additional cost + ability referring to whether that cost was paid are linked. The second refers only to whether the intent to pay was declared during casting.
- **Mechanism:** Kicker-style additional cost linking
- **Minimal Board:** A creature with "Kicker {W}" and "When this enters, if it was kicked, destroy target enchantment."
- **Action:** Player casts the creature, paying the kicker cost. It enters.
- **Expected Result:** The ETB trigger checks "was it kicked?" — yes, linked to the kicker cost payment. Destroys target enchantment.
- **Phase:** Phase 5 Pre-Work (T17 — alt/add cost types, T20 — linked abilities)
- **Ticket:** T17, T20

**ATOM-607.2i-002**

- **Rule:** 607.2i — Multiple kicker costs can each have their own linked ability.
- **Mechanism:** Per-kicker-cost linking (Stormscape Battlemage Example)
- **Minimal Board:** Stormscape Battlemage: "Kicker {W} and/or {2}{B}." ETB #1 triggers if {W} was paid. ETB #2 triggers if {2}{B} was paid.
- **Action:** Player casts with only {W} kicker paid.
- **Expected Result:** ETB #1 fires. ETB #2 does NOT fire — it's linked to the {2}{B} kicker, which wasn't paid.
- **Phase:** Phase 5 Pre-Work (T17, T20)
- **Ticket:** T17, T20

### 607.2j — Variable additional cost + "cost paid as cast" link

**Classification: TESTABLE.** Note: no known cards currently exercise this exact pattern (per 607-2-examples.md). Test spec kept for completeness.

**ATOM-607.2j-001**

- **Rule:** 607.2j — Ability causing a variable additional cost + ability referring to "the cost paid as [this] was cast" are linked. The second refers to the value of X chosen during casting.
- **Mechanism:** Variable cost value linking
- **Minimal Board:** A creature: "As an additional cost to cast this, pay X life." + "This enters with X +1/+1 counters."
- **Action:** Player pays 3 life as the additional cost (X=3).
- **Expected Result:** Creature enters with 3 +1/+1 counters.
- **Phase:** Phase 5 Pre-Work (T17, T18, T20)
- **Ticket:** T17, T18, T20

### 607.2k — Champion linked abilities

**Classification: DEFERRED — Phase 9 (Champion keyword, rule 702.72).**

### 607.2m — Anchor word linked abilities

**Classification: DEFERRED — Phase 9 (Anchor words, rule 614.12b).** See also: Monastery Siege (Khans or Dragons anchor word choice).

### 607.2n — Pre-game exile + "exiled with cards named" link

**Classification: OUT-OF-SCOPE.** This rule applies exclusively to Conspiracy-type cards (draft matters cards with pre-game exile zones). Companion and Leyline of the Void do NOT use this rule — those mechanics have their own rules. Conspiracy draft is permanently out of scope.

### 607.2p — Pre-game CDA choice link

**Classification: OUT-OF-SCOPE.** Same rationale as 607.2n — Conspiracy-type cards only.

### 607.2q — Permanent spell exile-as-cost + "exiled with" link

**Classification: TESTABLE.** See also: Champion of the Path (behold + exile + LTB return).

**ATOM-607.2q-001**

- **Rule:** 607.2q — If a permanent spell has an ability that exiles cards while paying a cost to cast it, and the permanent refers to "exiled with [this object]," those are linked.
- **Mechanism:** Cast-cost-exile linking
- **Minimal Board:** A permanent spell: "As an additional cost to cast this, exile a creature card from your graveyard." + "This creature's power is equal to the exiled card's power."
- **Action:** Player exiles a 4-power creature from graveyard as cost. Permanent enters.
- **Expected Result:** The permanent's power is 4 — linked to the specific card exiled as a casting cost.
- **Phase:** Phase 5 Pre-Work (T17, T20)
- **Ticket:** T17, T20

---

### 607.3 — Multiple exiled cards from copied ability

**Classification: TESTABLE.**

**ATOM-607.3-001**

- **Rule:** 607.3 — If one ability of a linked pair refers to "the exiled card" but the other exiled multiple cards (usually from being copied), the ability refers to each exiled card.
- **Mechanism:** Multi-exile linked ability resolution
- **Minimal Board:** An ability that exiles one card is copied (so it exiles two cards total). The linked ability says "return the exiled card to its owner's hand."
- **Action:** Linked return ability resolves.
- **Expected Result:** Both exiled cards are returned. The singular "the exiled card" is treated as each card individually.
- **Phase:** Phase 8 (copy effects + linked abilities)
- **Ticket:** DEFERRED — Phase 8

### 607.4 — Ability in multiple linked pairs

**Classification: PURE-DEF.** Has an Example (Paradise Plume). Notes that one ability can be part of more than one linked pair. No independent test — confirmed by testing individual link types.

### 607.5 — Acquired linked abilities

**Classification: TESTABLE.** Has an Example.

**ATOM-607.5-001**

- **Rule:** 607.5 — If an object acquires a pair of linked abilities as part of the same effect, the abilities are linked on that object. They can't be linked to any other ability on the object.
- **Mechanism:** Acquired-pair isolation
- **Minimal Board:** Quicksilver Elemental gains Arc-Slogger's "{R}, Exile top 10: Deal 2 damage" ability. Quicksilver also gains Sisters of Stone Death's exile and return abilities (a separate pair).
- **Action:** Quicksilver uses Arc-Slogger's exile, then Sisters' return.
- **Expected Result:** Sisters' return ability can only return cards exiled by Sisters' exile ability (its linked pair). Cards exiled by Arc-Slogger's ability are NOT returnable.
- **Phase:** Phase 8 (ability copying + linked abilities)
- **Ticket:** DEFERRED — Phase 8

### 607.5a — Undefined choice from unlinked grant

**Classification: TESTABLE.** Has two Examples.

**ATOM-607.5a-001**

- **Rule:** 607.5a — If an object gains an ability referring to a choice but doesn't copy the linked choice-making ability (or no choice was made), the choice is "undefined" and that part does nothing.
- **Mechanism:** Undefined choice from broken link
- **Minimal Board:** Unstable Shapeshifter copies Voice of All ("As this enters, choose a color. Protection from chosen color."). Shapeshifter didn't enter as Voice of All → no color was chosen.
- **Action:** Check Shapeshifter's abilities.
- **Expected Result:** The "protection from the chosen color" part does nothing — the choice is undefined.
- **Phase:** Phase 8 (copy effects + linked abilities)
- **Ticket:** DEFERRED — Phase 8

---

## Rule 608 — Resolving Spells and Abilities

### 608.1 — Resolution trigger: all players pass in succession

**Classification: TESTABLE.**

**ATOM-608.1-001**

- **Rule:** 608.1 — Each time all players pass in succession, the spell or ability on top of the stack resolves.
- **Mechanism:** Stack resolution trigger on all-pass
- **Minimal Board:** A Lightning Bolt is on top of the stack targeting a creature. Both players pass priority.
- **Action:** All players pass in succession.
- **Expected Result:** Lightning Bolt resolves (deals 3 damage to the creature).
- **Phase:** ALREADY-IMPLEMENTED (priority.rs consecutive pass detection → stack.rs resolve)
- **Ticket:** N/A

---

### 608.2 — Instant/sorcery/ability resolution procedure

**Classification: BOUNDARY-DEF (header).** Establishes the resolution order: 608.2a–b first, then 608.2c–m in order, then 608.2n and 608.2p last.

**ATOM-608.2-001**

- **Rule:** 608.2 — Resolution procedure order: 608.2a–b checks happen before 608.2c–m instruction execution, and 608.2n/p happen last.
- **Mechanism:** Full resolution procedure order verification
- **Minimal Board:** A targeted triggered ability with an intervening-if clause. Target is legal. Condition is true.
- **Action:** Ability resolves. Verify order: (1) intervening-if checked (608.2a), (2) target legality checked (608.2b), (3) effect executed (608.2c), (4) ability removed from stack (608.2n), (5) post-resolution triggers fire (608.2p).
- **Expected Result:** All steps execute in correct order. This is a meta-test verifying the resolution pipeline.
- **Phase:** Phase 7 (triggered abilities) + Phase 5 Pre-Work (T18)
- **Ticket:** T18

### 608.2a — Intervening "if" check at resolution

**Classification: TESTABLE.** Cross-references 603.4.

**ATOM-608.2a-001**

- **Rule:** 608.2a — If a triggered ability has an intervening "if" clause, it checks the condition at resolution. If false, the ability is removed and does nothing.
- **Mechanism:** Intervening-if resolution check
- **Minimal Board:** A triggered ability: "At the beginning of your upkeep, if you have 40+ life, you win." Player had 42 life at trigger time; now has 38 (damaged in response).
- **Action:** Ability resolves.
- **Expected Result:** Condition is false at resolution (38 < 40). Ability is removed from stack, does nothing.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 608.2b — Target legality recheck at resolution

**Classification: TESTABLE.** Has two Examples. Multi-clause rule.

**ATOM-608.2b-001**

- **Rule:** 608.2b — On resolution, check if targets are still legal. If ALL targets are illegal, the spell/ability doesn't resolve (fizzle). If it's a spell, it goes to graveyard.
- **Mechanism:** All-targets-illegal fizzle
- **Minimal Board:** Lightning Bolt targeting a creature. Before resolution, the creature gains protection from red (making it an illegal target).
- **Action:** Bolt tries to resolve.
- **Expected Result:** Target is illegal. Bolt fizzles — removed from stack, put into owner's graveyard. No damage dealt.
- **Phase:** ALREADY-IMPLEMENTED (engine/stack.rs target recheck before resolution)
- **Ticket:** N/A

**ATOM-608.2b-002**

- **Rule:** 608.2b — If SOME targets are legal and others are illegal, the spell resolves but illegal targets aren't affected.
- **Mechanism:** Partial-target resolution (Plague Spores Example)
- **Minimal Board:** "Destroy target nonblack creature and target land." Same artifact creature land chosen for both. It becomes black before resolution.
- **Action:** Spell resolves. "Target nonblack creature" is now illegal (it's black). "Target land" is still legal.
- **Expected Result:** The "destroy target nonblack creature" part doesn't affect it. The "destroy target land" part DOES destroy it. The spell resolves because at least one target is legal.
- **Phase:** Phase 5 Pre-Work (T18 — multi-target partial resolution)
- **Ticket:** T18

**ATOM-608.2b-003**

- **Rule:** 608.2b — A target that's no longer in the zone it was in when targeted is illegal.
- **Mechanism:** Zone-change makes target illegal
- **Minimal Board:** A spell targets a creature on the battlefield. Before resolution, the creature is bounced to hand.
- **Action:** Spell tries to resolve.
- **Expected Result:** Target is illegal (no longer on the battlefield). If it was the only target, the spell fizzles.
- **Phase:** ALREADY-IMPLEMENTED (targeting.rs zone check)
- **Ticket:** N/A

**ATOM-608.2b-004**

- **Rule:** 608.2b — If the source of an ability has left its zone, last known information is used for the target legality check.
- **Mechanism:** LKI for source during target recheck
- **Minimal Board:** A permanent's triggered ability targets a creature. Before resolution, the permanent is destroyed.
- **Action:** The triggered ability resolves.
- **Expected Result:** The ability uses the permanent's last known information (LKI) to check the target. If the target is still legal, the ability resolves.
- **Phase:** Phase 5 Layers (L18 — LKI system)
- **Ticket:** L18

**ATOM-608.2b-005**

- **Rule:** 608.2b — Partial-target resolution with simpler card.
- **Mechanism:** Partial-target resolution (Jagged Lightning)
- **Minimal Board:** Jagged Lightning ({3}{R}{R}, "Deals 3 damage to each of two target creatures."). Target creature A and creature B. Before resolution, creature A gains protection from red.
- **Action:** Spell resolves.
- **Expected Result:** Creature A is illegal target, but creature B is still legal. Spell resolves. Creature B takes 3 damage. Creature A is unaffected.
- **Phase:** Phase 5 Pre-Work (T18 — multi-target partial resolution)
- **Ticket:** T18

---

### 608.2c — Follow instructions in order written

**Classification: TESTABLE.**

> **Implementation note:** This is implicitly verified by all resolution tests since `resolve_effect(Effect::Sequence(...))` inherently processes instructions in order. The "can't be regenerated" modifier test (below) is a good standalone verification of instruction ordering semantics. Keep test but note it's also covered by composition.

**ATOM-608.2c-001**

- **Rule:** 608.2c — The controller follows instructions in the order written. Replacement effects may modify actions. Later text may modify earlier text's meaning.
- **Mechanism:** Sequential instruction execution
- **Minimal Board:** A spell: "Destroy target creature. It can't be regenerated."
- **Action:** Spell resolves.
- **Expected Result:** The "It can't be regenerated" clause modifies the "Destroy" clause. The destruction happens without allowing regeneration. Instructions are read holistically, not step-by-step in isolation.
- **Phase:** ALREADY-IMPLEMENTED (engine/resolve.rs sequential effect resolution)
- **Ticket:** N/A

### 608.2d — Resolution-time choices

**Classification: TESTABLE.** Has an Example.

**ATOM-608.2d-001**

- **Rule:** 608.2d — Choices not made during casting are announced during resolution. Can't choose illegal/impossible options (exception: drawing from empty library).
- **Mechanism:** Resolution-time choice announcement
- **Minimal Board:** A spell: "You may sacrifice a creature. If you don't, you lose 4 life." Player controls no creatures.
- **Action:** Spell resolves. Player must choose.
- **Expected Result:** The sacrifice option is impossible (no creatures). Player must choose "lose 4 life."
- **Phase:** Phase 7 (triggered abilities — resolution choices via DecisionProvider)
- **Ticket:** Phase 7 / T18

**ATOM-608.2d-002**

- **Rule:** 608.2d — If an effect divides among untargeted objects, each chosen object must receive at least one.
- **Mechanism:** Resolution-time untargeted distribution
- **Minimal Board:** An effect: "Distribute 3 +1/+1 counters among creatures you control." Player controls 2 creatures.
- **Action:** Player distributes: 2 to creature A, 1 to creature B.
- **Expected Result:** Legal — each receives at least 1. Distribution of 3/0 would be illegal.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

**ATOM-608.2d-003**

- **Rule:** 608.2d — Player can choose fewer objects than the maximum for distribution, as long as each chosen object gets at least 1.
- **Mechanism:** Flexible distribution with minimum-per-object
- **Minimal Board:** An effect: "Distribute 3 +1/+1 counters among creatures you control." Player controls 3 creatures but chooses only 2.
- **Action:** Player distributes: 2 to creature A, 1 to creature B. Creature C gets nothing.
- **Expected Result:** Legal — each chosen creature receives at least 1. Third creature wasn't chosen.
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

### 608.2e — Multi-player multi-step: APNAP ordering

**Classification: TESTABLE.**

**ATOM-608.2e-001**

- **Rule:** 608.2e — Multi-step effects involving multiple players: choices for each step in APNAP order, then that step processes simultaneously.
- **Mechanism:** APNAP-ordered resolution steps
- **Minimal Board:** A spell: "Each player discards a card, then each player draws a card." Two players.
- **Action:** Spell resolves. Step 1: Active player chooses discard, then nonactive. Both discard simultaneously. Step 2: Same for draw.
- **Expected Result:** Discard choices made in APNAP order, executed simultaneously. Draw choices (trivial) in APNAP order, executed simultaneously.
- **Phase:** Phase 9 (multiplayer APNAP) — basic 2-player case may be Phase 5 Pre-Work
- **Ticket:** T18 (2-player), Phase 9 (multiplayer)

### 608.2f — Simultaneous multi-player/object actions

**Classification: TESTABLE.** Has two Examples.

**ATOM-608.2f-001**

- **Rule:** 608.2f — Actions on multiple players/objects are processed simultaneously. If can't be simultaneous, APNAP order used.
- **Mechanism:** Simultaneous action processing
- **Minimal Board:** Blatant Thievery: "For each opponent, gain control of target permanent that player controls." Two opponents.
- **Action:** Spell resolves.
- **Expected Result:** Controller gains control of both permanents simultaneously.
- **Phase:** Phase 9 (multiplayer)
- **Ticket:** Phase 9

**ATOM-608.2f-002**

- **Rule:** 608.2f — If simultaneous actions can't truly be simultaneous, use APNAP.
- **Mechanism:** APNAP-ordered simultaneous sacrifice
- **Minimal Board:** Effect says "Each player sacrifices a creature." Two players. Active player controls a creature with a death trigger.
- **Action:** APNAP order: active player chooses sacrifice first, then nonactive. Both sacrifices happen simultaneously.
- **Expected Result:** Both sacrifices processed simultaneously. Death triggers from both go on stack in APNAP order.
- **Phase:** Phase 9 (multiplayer APNAP) — basic 2-player case may be Phase 5 Pre-Work
- **Ticket:** T18 (2-player), Phase 9 (multiplayer)

### 608.2g — Mana ability window during resolution + cast-during-resolution

**Classification: TESTABLE.**

**ATOM-608.2g-001**

- **Rule:** 608.2g — If an effect gives the option to pay mana, player may activate mana abilities first. If an effect allows casting during resolution, follow 601.2a–i; no player gets priority after; spell goes on top of stack.
- **Mechanism:** Cast-during-resolution (cascade-like)
- **Minimal Board:** A spell resolving: "You may cast a spell from among the exiled cards without paying its mana cost."
- **Action:** Player casts the free spell during resolution.
- **Expected Result:** The free spell goes on top of the stack. No priority is granted to any player after it's cast. The original spell continues resolving.
- **Phase:** Phase 8 (cast-during-resolution effects)
- **Ticket:** DEFERRED — Phase 8

> **Implementation note:** Casting during resolution does NOT make the cast spell uncounterable. After the resolving spell/ability finishes, players get priority with the newly cast spell on the stack. Opponents can respond to it normally.

### 608.2h — Information determined once; current info for public zone, LKI otherwise

**Classification: TESTABLE.**

**ATOM-608.2h-001**

- **Rule:** 608.2h — Information from the game is determined once when the effect is applied. If the source is in a public zone, use current info; if it left or moved to hidden zone, use LKI.
- **Mechanism:** Single-determination + LKI fallback
- **Minimal Board:** An ability says "This creature deals damage equal to its power to target player." The creature has 3 power when the ability was put on the stack. Before resolution, a Giant Growth makes it 6 power.
- **Action:** Ability resolves.
- **Expected Result:** Damage = 6 (current power, since the creature is still on the battlefield in a public zone). NOT 3 (value when triggered).
- **Phase:** Phase 5 Layers (L18 — LKI) + Phase 7
- **Ticket:** L18

**ATOM-608.2h-002**

- **Rule:** 608.2h — If the source left its public zone, use LKI.
- **Mechanism:** LKI for departed source
- **Minimal Board:** A creature's ability: "Deal damage equal to this creature's power." Creature has 4 power. Before resolution, creature dies.
- **Action:** Ability resolves.
- **Expected Result:** Damage = 4 (LKI of power when it was last on the battlefield).
- **Phase:** Phase 5 Layers (L18)
- **Ticket:** L18

### 608.2i — Look-back-in-time for historical information

**Classification: TESTABLE.** Has an Example.

**ATOM-608.2i-001**

- **Rule:** 608.2i — Effects that look back at previous game states don't require the objects to currently be in the relevant zone or meet the criteria, as long as they did at the specified time.
- **Mechanism:** Historical look-back exception to 608.2h
- **Minimal Board:** Search Party Captain: "This spell costs {1} less for each creature you attacked with this turn." Player attacked with Bear Cub, which later became a noncreature.
- **Action:** Player casts Search Party Captain.
- **Expected Result:** Cost is reduced by {1} — the player attacked with a creature this turn, even though Bear Cub is no longer a creature.
- **Phase:** Phase 5 Pre-Work (T18 — cost reduction based on historical game state)
- **Ticket:** T18

> **Implementation note:** Fight (Chapter 7) is an exception to the historical look-back pattern. When reaching Fight rules, cross-reference 608.2i.

### 608.2j — Characteristic checks are value-only

**Classification: TESTABLE.** Has an Example.

**ATOM-608.2j-001**

- **Rule:** 608.2j — An effect checking characteristics checks only the specified value, regardless of related characteristics.
- **Mechanism:** Strict characteristic matching
- **Minimal Board:** An effect: "Destroy all black creatures." A white-and-black creature is on the battlefield.
- **Action:** Effect resolves.
- **Expected Result:** The white-and-black creature IS destroyed (it is black). An effect saying "Destroy all nonblack creatures" would NOT destroy it (it is black).
- **Phase:** Phase 5 Layers (L10 — color in compute_characteristics) + Phase 7
- **Ticket:** L10

### 608.2k — Untargeted object reference persists through characteristic changes

**Classification: TESTABLE.** Has an Example.

**ATOM-608.2k-001**

- **Rule:** 608.2k — If an ability's effect refers to a specific untargeted object previously referenced by cost or trigger condition, it still affects that object even if characteristics changed.
- **Mechanism:** Untargeted-reference persistence
- **Minimal Board:** Wall of Tears: "Whenever this blocks a creature, return that creature to its owner's hand at end of combat." Wall blocks a creature. Before end of combat, the blocked permanent stops being a creature.
- **Action:** End of combat — delayed trigger resolves.
- **Expected Result:** The permanent is returned to hand, even though it's no longer a creature. The reference was established by the trigger condition.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

### 608.2m — Spell/ability leaving stack during resolution continues

**Classification: TESTABLE.**

**ATOM-608.2m-001**

- **Rule:** 608.2m — If an instant, sorcery, or ability that can legally resolve leaves the stack once it starts resolving, it continues to resolve fully.
- **Mechanism:** Resolution continuation after stack departure
- **Minimal Board:** A sorcery resolves. During resolution, an effect exiles the sorcery from the stack (unusual corner case).
- **Action:** The sorcery was mid-resolution.
- **Expected Result:** The sorcery continues to resolve fully, even though it's no longer on the stack.
- **Phase:** Phase 8 (rare corner case)
- **Ticket:** DEFERRED — Phase 8

> **Note:** Audit questions if this is truly testable or is a catchall. The scenario (spell leaving stack during its own resolution) is extremely rare. Keep test spec for completeness but mark as LOW PRIORITY. May be untestable in practice.

### 608.2n — Final step: instant/sorcery → graveyard, ability → ceases to exist

**Classification: TESTABLE.**

**ATOM-608.2n-001**

- **Rule:** 608.2n — As the final part of resolution: instant/sorcery → owner's graveyard. Ability → removed from stack, ceases to exist.
- **Mechanism:** Post-resolution zone transition
- **Minimal Board:** Lightning Bolt (instant) on the stack. It resolves.
- **Action:** Bolt finishes resolving.
- **Expected Result:** Bolt is moved from stack to owner's graveyard.
- **Phase:** ALREADY-IMPLEMENTED (engine/stack.rs pop-first approach — spell → graveyard)
- **Ticket:** N/A

**ATOM-608.2n-002**

- **Rule:** 608.2n — Ability ceases to exist after resolution.
- **Mechanism:** Ability removal from stack post-resolution
- **Minimal Board:** An activated ability on the stack resolves.
- **Action:** Ability finishes resolving.
- **Expected Result:** The ability object is removed from the stack. It no longer exists.
- **Phase:** ALREADY-IMPLEMENTED (engine/stack.rs removes ability after resolution)
- **Ticket:** N/A

### 608.2p — Post-resolution triggers

**Classification: TESTABLE.**

**ATOM-608.2p-001**

- **Rule:** 608.2p — Abilities that trigger/track when a spell or ability resolves.
- **Mechanism:** Resolution-count tracking on triggered ability
- **Minimal Board:** Player controls Ashling, Flame Dancer ({2}{R}{R}, Legendary Creature — Elemental Shaman). Ashling has: "Magecraft — Whenever you cast or copy an instant or sorcery spell, discard a card, then draw a card. If this is the second time this ability has resolved this turn, Ashling deals 2 damage to each opponent and each creature they control. If it's the third time, add {R}{R}{R}{R}." Player has 3 cards in hand. Opponent controls a 1/1 creature.
- **Action:** Player casts 3 instant/sorcery spells sequentially, letting each Magecraft trigger resolve before casting the next.
- **Expected Result:** 1st resolution: discard 1, draw 1 (no bonus). 2nd resolution: discard 1, draw 1, then Ashling deals 2 damage to each opponent and each creature they control (1/1 dies). 3rd resolution: discard 1, draw 1, then add {R}{R}{R}{R} to player's mana pool.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7 triggers

> **Note:** Ashling also tests the Magecraft keyword and resolution-count delayed triggers (603.7h), making it an excellent cross-rule integration card.

---

### 608.3 — Permanent spell resolution

**Classification: BOUNDARY-DEF (header).** Establishes that permanent spell resolution follows 608.3a–e.

### 608.3a — No-target permanent spell → ETB

**Classification: TESTABLE.**

> **Implementation note:** Rule 400.4a (instants/sorceries can't enter the battlefield) ensures that only permanent spells reach 608.3a. This makes 608.3a "safe" — it will never be asked to resolve a non-permanent spell.

**ATOM-608.3a-001**

- **Rule:** 608.3a — If the resolving permanent spell has no targets, it becomes a permanent and enters the battlefield under the controller's control.
- **Mechanism:** Untargeted permanent spell → ETB
- **Minimal Board:** Grizzly Bears (creature spell, no targets) on the stack. Player 0 is the controller.
- **Action:** Bears resolves.
- **Expected Result:** Bears enters the battlefield as a permanent. Controller is Player 0.
- **Phase:** ALREADY-IMPLEMENTED (engine/stack.rs permanent spell resolution → init_zone_state)
- **Ticket:** N/A

### 608.3b — Targeted permanent spell → target legality recheck

**Classification: TESTABLE.**

**ATOM-608.3b-001**

- **Rule:** 608.3b — If the resolving permanent spell has a target, check target legality (per 608.2b). If illegal and it's a bestowed Aura, it becomes a creature instead. Otherwise, fizzle → graveyard.
- **Mechanism:** Targeted permanent fizzle or bestow fallback
- **Minimal Board:** An Aura spell targets a creature. The creature is removed before resolution.
- **Action:** Aura tries to resolve.
- **Expected Result:** Target is illegal. Non-bestow Aura fizzles → graveyard.
- **Phase:** Phase 5 Pre-Work (T15b — Aura attachment on resolution)
- **Ticket:** T15b

**ATOM-608.3b-002**

- **Rule:** 608.3b — Bestowed Aura with illegal target becomes a creature spell and resolves per 608.3a.
- **Mechanism:** Bestow fallback
- **Minimal Board:** A bestowed Aura spell targets a creature. The creature is removed. The Aura has bestow.
- **Action:** Aura tries to resolve. Target is illegal.
- **Expected Result:** The Aura becomes a creature spell and enters the battlefield as a creature (not an Aura). Per 608.3a.
- **Phase:** Phase 9 (Bestow keyword — rule 702.103e)
- **Ticket:** DEFERRED — Phase 9

> **Implementation note:** Mutate (rule 729) should also be tested here when brought into scope. Currently deferred in 608.3d. If mutate is implemented, add test for "permanent spell resolves, tries to mutate onto target creature" — target legality recheck applies.

### 608.3c — Aura spell → ETB attached to target

**Classification: TESTABLE.**

**ATOM-608.3c-001**

- **Rule:** 608.3c — Aura spell resolves → becomes permanent, enters attached to the target.
- **Mechanism:** Aura ETB attachment
- **Minimal Board:** An Aura targeting creature A is on the stack. Creature A is legal.
- **Action:** Aura resolves.
- **Expected Result:** Aura enters the battlefield attached to creature A.
- **Phase:** Phase 5 Pre-Work (T15b — Aura attachment)
- **Ticket:** T15b

### 608.3d — Mutating creature spell → merge

**Classification: DEFERRED — Phase 9.** Mutate (rule 729) appears in Pioneer and Modern — must be supported eventually. Changed from OUT-OF-SCOPE to DEFERRED to signal intent to implement while not blocking current work.

### 608.3e — Permanent can't enter → graveyard

**Classification: TESTABLE.** Has an Example.

**ATOM-608.3e-001**

- **Rule:** 608.3e — If a permanent spell resolves but its controller can't put it onto the battlefield, that player puts it into its owner's graveyard.
- **Mechanism:** ETB prohibition fallback
- **Minimal Board:** Worms of the Earth ("Lands can't enter the battlefield") on the battlefield. Player casts Clone copying Dryad Arbor (a land creature).
- **Action:** Clone resolves, tries to enter as a land creature.
- **Expected Result:** Can't enter (it's a land, and lands can't enter). Goes to owner's graveyard instead.
- **Phase:** Phase 6 (replacement effects — ETB prohibition)
- **Ticket:** Phase 6

### 608.3f — Copy of permanent spell → token

**Classification: TESTABLE.**

**ATOM-608.3f-001**

- **Rule:** 608.3f — If the resolving object is a copy of a permanent spell, it becomes a token permanent. It is NOT "created" for purposes of token-creation triggers.
- **Mechanism:** Spell-copy → token (not "created")
- **Minimal Board:** A copy of a creature spell is on the stack (e.g., from a spell-copying effect).
- **Action:** The copy resolves.
- **Expected Result:** A token enters the battlefield. It is NOT considered "created" — abilities that trigger "when a token is created" do NOT trigger.
- **Phase:** Phase 8 (spell copying)
- **Ticket:** DEFERRED — Phase 8

### 608.3g — Stack static ability → delayed trigger on ETB

**Classification: TESTABLE.**

**ATOM-608.3g-001**

- **Rule:** 608.3g — If a permanent spell has a static ability that functions on the stack and creates a delayed triggered ability, that delayed trigger is created as the permanent enters.
- **Mechanism:** Stack-zone static → delayed trigger on ETB (Dash, Blitz, Warp)
- **Minimal Board:** A creature with Blitz: "When this enters, it gains haste and 'When this creature dies, draw a card.' At the beginning of the next end step, sacrifice it."
- **Action:** Creature resolves (cast with Blitz).
- **Expected Result:** On ETB, the delayed trigger is created: "At beginning of next end step, sacrifice."
- **Phase:** Phase 8 (Dash/Blitz keywords)
- **Ticket:** DEFERRED — Phase 8

---

## Composition Tests

These tests exercise interactions between multiple rules from this session, validating that the casting/activation/resolution pipeline works end-to-end.

### COMP-601+608-001 — Full casting pipeline → resolution → graveyard

- **Rules:** 601.2a–i, 608.2c, 608.2n
- **Mechanism:** Cast a targeted instant through the full pipeline, resolve it, verify post-resolution zone
- **Board:** Player 0 has Lightning Bolt in hand, 1 Mountain untapped. Opponent controls a 2/2 creature.
- **Action:** Player 0 casts Bolt: 601.2a (move to stack) → 601.2c (target creature) → 601.2f (total cost = {R}) → 601.2g (tap Mountain) → 601.2h (pay {R}). Both players pass. Bolt resolves.
- **Expected:** Creature takes 3 damage (dies to SBA). Bolt moves to graveyard (608.2n). Stack is empty.
- **Phase:** ALREADY-IMPLEMENTED (integration test exists)

### COMP-601+608-002 — Fizzle on target removal

- **Rules:** 601.2c, 608.2b
- **Mechanism:** Cast a spell, remove the target in response, verify fizzle
- **Board:** Player 0 has Lightning Bolt. Opponent controls creature A. Player 1 has an Unsummon.
- **Action:** Player 0 casts Bolt targeting creature A. Player 1 casts Unsummon on creature A (bouncing it). Both players pass on Unsummon → resolves. Both pass on Bolt → 608.2b target check fails.
- **Expected:** Bolt fizzles → graveyard. No damage dealt. Creature A is in Player 1's hand.
- **Phase:** ALREADY-IMPLEMENTED

### COMP-601+602+605-001 — Mana ability during casting (605.3a + 601.2g)

- **Rules:** 605.3a, 601.2g, 605.3b
- **Mechanism:** Activate mana abilities during casting's mana window
- **Board:** Player has Hill Giant ({3}{R}) in hand. 4 Mountains untapped.
- **Action:** Player begins casting Hill Giant. At 601.2g, activates 4 Mountains' mana abilities. Each resolves immediately (605.3b). Pays {3}{R} at 601.2h.
- **Expected:** Hill Giant is on the stack. 4 Mountains tapped. Mana pool at 0 (all spent).
- **Phase:** ALREADY-IMPLEMENTED

### COMP-602+605-001 — Summoning sickness blocks tap ability but not sacrifice ability

- **Rules:** 602.5a, 605.1a
- **Mechanism:** Summoning-sick creature with {T} mana ability vs. sacrifice-cost ability
- **Board:** Player plays a creature this turn with "{T}: Add {G}" and "{B}, Sacrifice this creature: Draw a card."
- **Action:** Player attempts to activate the tap ability → blocked (602.5a, summoning sick). Player activates the sacrifice ability → allowed (no {T} in cost).
- **Expected:** Tap ability fails. Sacrifice ability succeeds — creature is sacrificed, player draws a card.
- **Phase:** ALREADY-IMPLEMENTED (engine/costs.rs summoning sickness only checks Cost::Tap)

### COMP-603+608-001 — ETB trigger → stack placement → resolution

- **Rules:** 603.2, 603.3, 603.6a, 608.2c
- **Mechanism:** Permanent enters, ETB triggers, trigger resolves
- **Board:** Player controls Soul Warden ("Whenever a creature enters, gain 1 life"). Player casts Grizzly Bears.
- **Action:** Bears resolves → enters battlefield. Soul Warden triggers (603.2). Trigger placed on stack (603.3). Both players pass → trigger resolves (608.2c).
- **Expected:** Player gains 1 life.
- **Phase:** Phase 7 (triggered abilities)

### COMP-601+604+605-001 — Static cost reduction + mana ability + casting

- **Rules:** 604.2, 601.2f, 605.3a
- **Mechanism:** Cost reduction from static ability modifies total cost during casting
- **Board:** Player controls Goblin Electromancer ("Instant and sorcery spells you cast cost {1} less"). Player has Lightning Bolt in hand. Player controls 1 Mountain.
- **Action:** Player casts Bolt. 601.2f: base {R} − {0} generic reduction (no generic to reduce) = {R}. But for a {1}{R} spell: base {1}{R} − {1} = {R}. Taps Mountain at 601.2g. Pays {R} at 601.2h.
- **Expected:** Spell is cast with reduced cost. (Bolt specifically has no generic, so Electromancer doesn't help — this test is better with a {1}{R} spell.)
- **Phase:** Phase 5 Layers (L15 — cost modification)

### COMP-607+608-001 — Linked exile + return (O-Ring pattern)

- **Rules:** 607.1, 607.2a, 608.3a, 603.6c
- **Mechanism:** O-Ring enters → exiles target → O-Ring leaves → returns exiled card
- **Board:** Player casts Banishing Light targeting opponent's creature A. Banishing Light resolves.
- **Action:** ETB: exile creature A (linked ability A). Later, Banishing Light is destroyed → LTB trigger (linked ability B) fires.
- **Expected:** Creature A returns to the battlefield. Only creature A (exiled by linked ability A) is returned.
- **Phase:** Phase 5 Pre-Work (T20) + Phase 7

---

## Classification Summary Table

### PURE-DEF Rules (no test needed — definitions/naming only)

| Rule   | Description                                    |
| ------ | ---------------------------------------------- |
| 600.1  | Chapter header                                 |
| 601.1  | Errata: "playing" → "casting"                  |
| 602.1  | Activated abilities format definition          |
| 602.1b | Activation instructions after effect           |
| 602.1c | Only activated abilities can be "activated"    |
| 602.1d | Errata: "playing" → "activating"               |
| 602.3  | Opponent choice (parallel to 601.6)            |
| 602.3a | Controller picks opponent (parallel to 601.6a) |
| 602.3b | Simultaneous actions (parallel to 601.6b)      |
| 603.1  | Triggered ability format ("When/Whenever/At")  |
| 603.1a | Post-effect instructions                       |
| 603.11 | Static+triggered link (reference to 607)       |
| 604.1  | Static abilities "simply true"                 |
| 605.1  | Mana ability criteria header                   |
| 605.3  | Activated mana ability header                  |
| 605.4  | Triggered mana ability header                  |
| 606.1  | Loyalty abilities naming                       |
| 607.1a | Granted ability is "printed on"                |
| 607.1b | DFC abilities are "printed on"                 |
| 607.2  | Kinds of linked abilities header               |
| 603.10 | "Look back in time" trigger concept definition |
| 607.4  | Ability in multiple linked pairs               |

### ALREADY-IMPLEMENTED Rules

| Rule             | Description                                   | Verified by            |
| ---------------- | --------------------------------------------- | ---------------------- |
| 601.2a (partial) | Move spell to stack, set controller           | cast.rs                |
| 601.2h (partial) | Atomic cost payment (can_pay pre-check)       | costs.rs               |
| 601.2i (partial) | Priority returns to caster                    | priority.rs            |
| 602.1a           | Activation cost charged to controller         | costs.rs               |
| 602.2 (partial)  | Activation restricted to controller           | cast.rs                |
| 602.2a (partial) | Ability on stack, controller set              | cast.rs                |
| 602.5a           | Summoning sickness + haste bypass             | costs.rs               |
| 605.1a (partial) | Mana ability classification                   | priority.rs            |
| 605.2            | Classification is static, not state-dependent | card data              |
| 605.3a           | Mana ability window during casting            | priority.rs            |
| 605.3b           | Mana abilities don't use the stack            | mana.rs                |
| 605.3c           | No re-activation until resolved               | synchronous resolution |
| 605.5b           | Spells are never mana abilities               | stack architecture     |
| 608.1            | All-pass → resolve top of stack               | priority.rs            |
| 608.2b (partial) | All-targets-illegal fizzle                    | stack.rs               |
| 608.2b (partial) | Zone-change makes target illegal              | targeting.rs           |
| 608.2c           | Follow instructions in order                  | resolve.rs             |
| 608.2n           | Instant/sorcery → graveyard after resolution  | stack.rs               |
| 608.3a           | Untargeted permanent → ETB                    | stack.rs               |

### DEFERRED Rules

| Rule               | Description                                   | Deferred To               |
| ------------------ | --------------------------------------------- | ------------------------- |
| 601.2b (hybrid)    | Hybrid mana choice during casting             | Future — hybrid mana      |
| 601.2b (Phyrexian) | Phyrexian mana choice during casting          | Future — Phyrexian mana   |
| 601.3a             | Cast legality look-ahead                      | D17                       |
| 601.3b             | Flash look-ahead for bestow/morph             | D17                       |
| 601.3c             | Conditional flash from alt/add cost           | D17                       |
| 601.3d             | Conditional flash                             | D17                       |
| 601.3e             | Alternative characteristics for cast legality | Phase 9 (morph)           |
| 601.3f             | Face-down exile cast permission               | Phase 8 (D21)             |
| 601.5a             | Flash condition persistence                   | D17                       |
| 601.6a             | Controller picks opponent (multiplayer)       | Phase 9                   |
| 602.5c             | Per-acquisition restriction scoping           | Phase 8 (ability copying) |
| 603.10b            | Phase-out triggers look back                  | Phase 9 (Phasing)         |
| 606.2–606.6        | All loyalty ability rules                     | Phase 8 (Planeswalkers)   |
| 607.1d             | Cross-object linked abilities                 | Phase 8                   |
| 607.2e             | Noted-information linking                     | Phase 8                   |
| 607.2f             | Word-choice linking                           | Phase 8                   |
| 607.2g             | ETB-cost linking                              | Phase 6                   |
| 607.2k             | Champion linked abilities                     | Phase 9                   |
| 607.2m             | Anchor word linked abilities                  | Phase 9                   |
| 607.3              | Multi-exile from copied ability               | Phase 8                   |
| 607.5              | Acquired linked ability isolation             | Phase 8                   |
| 607.5a             | Undefined choice from broken link             | Phase 8                   |
| 608.2g             | Cast-during-resolution                        | Phase 8                   |
| 608.2m             | Resolution continuation after stack departure | Phase 8                   |
| 608.3b (bestow)    | Bestow fallback                               | Phase 9                   |
| 608.3e             | ETB prohibition fallback                      | Phase 6                   |
| 608.3d             | Mutating creature spell → merge               | Phase 9                   |
| 608.3f             | Spell-copy → token                            | Phase 8                   |
| 608.3g             | Stack static → delayed trigger on ETB         | Phase 8                   |

### OUT-OF-SCOPE Rules

| Rule    | Description                 | Reason                       |
| ------- | --------------------------- | ---------------------------- |
| 603.10g | Planeswalks away triggers   | Planechase excluded          |
| 607.2n  | Pre-game exile linking      | Conspiracy cards only        |
| 607.2p  | Pre-game CDA choice linking | Conspiracy cards only        |

---

## Gap Report

### Missing Engine Capabilities Identified

1. **Casting pipeline completeness (T18)** — The engine has basic casting but lacks:
   
   - Mode choice storage on StackEntry (601.2b)
   - Additional/alternative cost declaration (601.2b, T17)
   - X value storage on StackEntry (601.2b, T06)
   - Conditional targets based on kicker/mode (601.2c)
   - Per-instance target uniqueness enforcement (601.2c)
   - Distribution storage on StackEntry (601.2d)
   - Post-proposal legality recheck with rollback (601.2e)
   - Full cost assembly pipeline: base + increases − reductions + lock-in (601.2f, L15)
   - Explicit mana ability window (601.2g — currently implicit)
   - Partial-target resolution (608.2b — currently all-or-nothing)

2. **Activation restrictions (T19)** — Missing:
   
   - Once-per-turn tracking per ability (602.5b)
   - Sorcery-speed activation restriction (602.5d)
   - `CantActivateAbilities` enforcement (602.5, L15)
   - Reveal-on-activation from hidden zone (602.2a)

3. **Linked abilities (T20)** — No infrastructure for:
   
   - Per-ability exile tracking ("exiled with [this]")
   - Choice storage on permanents (607.2d)
   - Kicker-paid flags on StackEntry/permanent (607.2i)
   - Self-linked ability detection (607.1c)

4. **Triggered ability infrastructure (Phase 7)** — Not yet implemented:
   
   - pending_triggers queue (architectural decision documented)
   - APNAP ordering for trigger stacking (603.3b)
   - Intervening-if double check (603.4)
   - State triggers (603.8)
   - Delayed triggered abilities (603.7)
   - Reflexive triggered abilities (603.12)
   - Look-back-in-time for LTB triggers (603.10a)
   - "Becomes" event filtering (603.2e)
   - Trigger doubling (603.2d)
   - Once-per-turn trigger restriction (603.2h)

5. **Multi-condition trigger tracking (603.1b)** — Not yet implemented:
   
   - Structured multi-condition trigger representation (e.g., "whenever you cast a creature spell that costs 4+ mana")
   - Condition evaluation pipeline that checks all conditions before queuing trigger
   - META-MULTI-CONDITION-TRIGGERS architecture note documents the approach

6. **Two-tier trigger stacking (603.3b)** — Not yet implemented:
   
   - APNAP ordering for trigger placement on stack
   - Within-player ordering via DecisionProvider choice
   - Two-tier system: inter-player (APNAP, mandatory) then intra-player (choice)

7. **GameState snapshot infrastructure (601.2e)** — Not yet implemented:
   
   - Full GameState snapshot before casting proposal begins
   - Rollback capability if proposal becomes illegal at 601.2e recheck
   - See state-tracking-architecture.md for approach comparison

8. **LKI system (L18)** — Not yet implemented:
   
   - LKISnapshot wrapping EffectiveCharacteristics
   - Snapshot on zone change
   - Source LKI for target recheck (608.2b)
   - Current-info vs. LKI determination (608.2h)

9. **Cost modification (L15)** — Scaffolding planned but not implemented:
   
   - CostModification enum (IncreaseCost, ReduceCost, SetMinimumCost)
   - PlayerActionRestriction for CantCastSpells, CantActivateAbilities
   - Post-layer pass computation

### New Tickets Identified

| Ticket | Description                                                   | Rules  |
| ------ | ------------------------------------------------------------- | ------ |
| NEW-1  | "Play a card" permission routing (land play vs. cast)         | 601.1a |
| NEW-2  | Mana ability classification: target check                     | 605.1a |
| NEW-3  | Target-forcing effect maximization during 601.2c              | 601.2c |
| NEW-4  | Opponent-directed choices during casting (601.6)              | 601.6  |
| NEW-5  | Controller-first simultaneous actions during casting (601.6b) | 601.6b |

---

## Session Statistics

- **Total sub-rules processed:** ~156
- **TESTABLE atoms generated:** ~113 (was 95; +18 new ATOMs from audit)
- **BOUNDARY-DEF atoms generated:** 14
- **PURE-DEF rules (no test):** 22 (was 21; +603.10 reclassified)
- **ALREADY-IMPLEMENTED rules:** 19
- **DEFERRED rules:** 31 (was 30; +608.3d moved from OUT-OF-SCOPE)
- **OUT-OF-SCOPE rules:** 3 (was 4; −608.3d moved to DEFERRED)
- **Composition tests:** 7
- **New tickets identified:** 5
- **META entries added (audit):** 6
- **Implementation notes added (audit):** ~20
- **ATOMs modified (audit):** ~4
- **ATOMs replaced (audit):** ~2 (607.1c-001, 608.2p-001)
