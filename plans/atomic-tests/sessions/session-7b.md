# Session 7B — Keyword Abilities (702.1–702.80)

> **CR Source:** `MTG-Rules/LLM-Chapter-Splits/ch7-pt-2.txt`
> **Scope:** Rules 702.1 through 702.80 (Deathtouch through Wither)
> **Date:** 2026-04-07
> **Depends on:** design_doc.md, roadmap.md, implementation-plan-final.md

## 10 Already-Implemented Keywords (Phases 1–4)

Deathtouch (702.2), Defender (702.3), Double Strike (702.4), First Strike (702.7), Flying (702.9), Haste (702.10), Lifelink (702.15), Reach (702.17), Trample (702.19), Vigilance (702.20).

For these: ALREADY-IMPLEMENTED sub-rules go in the classification table only. Sub-rules with unimplemented behavior get ATOM tests tagged with the appropriate deferred phase.

## Chunk Plan

| Chunk | Rules | Top-Level Count |
|-------|-------|-----------------|
| 1 | 702.1–702.4 | 4 (General, Deathtouch, Defender, Double Strike) |
| 2 | 702.5–702.10 | 6 (Enchant, Equip, First Strike, Flash, Flying, Haste) |
| 3 | 702.11–702.16 | 6 (Hexproof, Indestructible, Intimidate, Landwalk, Lifelink, Protection) |
| 4 | 702.17–702.22 | 6 (Reach, Shroud, Trample, Vigilance, Ward, Banding) |
| 5 | 702.23–702.37 | 15 (Rampage through Morph) |
| 6 | 702.38–702.55 | 18 (Amplify through Haunt) |
| 7 | 702.56–702.72 | 17 (Replicate through Champion) |
| 8 | 702.73–702.80 + Summary | 8 + Classification Table + COMP + Gap Report |

---

## Chunk 1 — 702.1–702.4 (General Keyword Rules, Deathtouch, Defender, Double Strike)

### 702.1 — General Keyword Rules

**702.1a** — PURE-DEF. Defines "[keyword ability] cost" as referring to variable costs only. Prerequisite for understanding kicker costs, flashback costs, etc. No independent mechanical consequence.

**702.1b** — PURE-DEF. Variable values in granted keyword abilities are constantly reevaluated. Prerequisite for understanding cards like Volcano Hellion (echo with variable cost) and Past in Flames (flashback cost = mana cost). Mechanical consequence manifests through the specific keyword, not this meta-rule.

**702.1c** — PURE-DEF. "The same is true for" grants each variant/variable of listed keywords. Prerequisite for Concerted Effort-style cards. Mechanical consequence tested via specific keyword grant effects (Phase 5 layers / Phase 7 triggers).

**702.1d** — PURE-DEF. "With [keyword]" = "with a [keyword] ability." Textual shorthand with no mechanical consequence.

### 702.2 — Deathtouch

**702.2a** — PURE-DEF. "Deathtouch is a static ability." No independent mechanical consequence.

**702.2b** — ALREADY-IMPLEMENTED. Creature dealt damage by deathtouch source → destroyed as SBA. Implemented in `engine/sba.rs` via `damaged_by_deathtouch` flag.

**702.2c** — ALREADY-IMPLEMENTED. Any nonzero combat damage from deathtouch source = lethal for trample excess purposes. Implemented in `engine/combat/keywords.rs` `lethal_damage_for`.

**702.2d** — Deathtouch functions from any zone.

**ATOM-702.2d-001**
- **Rule:** 702.2d — The deathtouch rules function no matter what zone an object with deathtouch deals damage from.
- **Mechanism:** Deathtouch damage routing from non-battlefield zones (e.g., a source that deals damage from graveyard via LKI or an ability)
- **Minimal Board:** P0 controls a creature with deathtouch that has an ability dealing damage from a non-battlefield zone (e.g., an "exile this from graveyard: deal 1 damage to target creature" ability). P1 controls a 5/5 creature.
- **Action:** Activate the graveyard ability, dealing 1 damage to the 5/5.
- **Expected Result:** The 5/5 has `damaged_by_deathtouch = true` and is destroyed by SBAs despite only 1 damage being dealt.
- **Phase:** Phase 6 (replacement/any-zone ability infrastructure)
- **Ticket:** DEFERRED — Phase 6. Requires zone-activated abilities + deathtouch from non-battlefield zone.
- **Tags:** deathtouch, any-zone, DEFERRED

**702.2e** — LKI determines deathtouch after zone change.

**ATOM-702.2e-001**
- **Rule:** 702.2e — If an object changes zones before an effect causes it to deal damage, its last known information is used to determine whether it had deathtouch.
- **Mechanism:** LKI snapshot preserves deathtouch status for delayed/pending damage after zone change
- **Minimal Board:** P0 controls a creature with deathtouch that has a triggered ability "when this creature dies, it deals 2 damage to target creature." P1 controls a 5/5 creature.
- **Action:** P0's deathtouch creature dies. The death trigger deals 2 damage to the 5/5, using LKI to determine deathtouch.
- **Expected Result:** The 5/5 is destroyed by SBAs (damaged_by_deathtouch = true via LKI), despite the source no longer existing on the battlefield.
- **Phase:** Phase 5 (LKI system — L19/T20b)
- **Ticket:** T20b (LKI system, deferred to Part 2)
- **Tags:** deathtouch, LKI, DEFERRED

**702.2f** — ALREADY-IMPLEMENTED. Multiple instances of deathtouch are redundant. Engine uses `has_keyword` boolean check.

### 702.3 — Defender

**702.3a** — PURE-DEF. "Defender is a static ability." No independent mechanical consequence.

**702.3b** — ALREADY-IMPLEMENTED. Creature with defender can't attack. Implemented in `engine/combat/validation.rs` as `HasDefender` error.

**702.3c** — ALREADY-IMPLEMENTED. Multiple instances of defender are redundant. Engine uses `has_keyword` boolean check.

### 702.4 — Double Strike

**702.4a** — PURE-DEF. "Double strike is a static ability that modifies the rules for the combat damage step." Prerequisite for 702.4b.

**702.4b** — ALREADY-IMPLEMENTED. Double strike creatures deal damage in both first-strike and normal combat damage steps. Implemented in `engine/combat/keywords.rs` `should_deal_damage_this_step`.

**702.4c** — Removing double strike during first combat damage step stops second step damage.

**ATOM-702.4c-001**
- **Rule:** 702.4c — Removing double strike from a creature during the first combat damage step will stop it from assigning combat damage in the second combat damage step.
- **Mechanism:** Mid-combat keyword removal affects combat damage step participation
- **Minimal Board:** P0 controls a 3/3 creature with double strike, attacking. P1 controls a 1/1 blocker. A continuous effect removes double strike from the attacker after first-strike damage is dealt (e.g., "until end of turn" effect ending, or an instant removing the ability).
- **Action:** First combat damage step: 3/3 assigns 1 to blocker, 2 to defender. Double strike is then removed. Second combat damage step begins.
- **Expected Result:** The 3/3 does NOT assign combat damage in the second step. Only normal-damage creatures (and current double-strikers) assign damage in step 2.
- **Phase:** Phase 5 (continuous effects — mid-combat keyword changes)
- **Ticket:** DEFERRED — Phase 5 Layers. Requires continuous effect removal mid-combat.
- **Tags:** double-strike, mid-combat, continuous-effects, DEFERRED

**702.4d** — Giving double strike to a first-strike creature after first step allows second step damage.

**ATOM-702.4d-001**
- **Rule:** 702.4d — Giving double strike to a creature with first strike after it has already dealt combat damage in the first combat damage step will allow the creature to assign combat damage in the second combat damage step.
- **Mechanism:** Mid-combat keyword grant enables additional damage step participation
- **Minimal Board:** P0 controls a 2/2 with first strike, attacking unblocked. After first-strike damage is dealt, an effect gives it double strike.
- **Action:** First combat damage step: 2/2 deals 2 to defending player. Double strike is granted. Second combat damage step begins.
- **Expected Result:** The creature assigns 2 more combat damage in the second step (it now has double strike and is a "remaining attacker that currently has double strike").
- **Phase:** Phase 5 (continuous effects — mid-combat keyword grant)
- **Ticket:** DEFERRED — Phase 5 Layers. Requires continuous effect grant mid-combat.
- **Tags:** double-strike, first-strike, mid-combat, continuous-effects, DEFERRED

**702.4e** — ALREADY-IMPLEMENTED. Multiple instances of double strike are redundant. Engine uses `has_keyword` boolean check.

--- End of Chunk 1 ---

## Chunk 2 — 702.5–702.10 (Enchant, Equip, First Strike, Flash, Flying, Haste)

### 702.5 — Enchant

**702.5a** — Enchant restricts Aura targeting and attachment.

**ATOM-702.5a-001**
- **Rule:** 702.5a — Enchant is a static ability, written "Enchant [object or player]." The enchant ability restricts what an Aura spell can target and what an Aura can enchant.
- **Mechanism:** Aura spell targeting restricted by enchant ability
- **Minimal Board:** P0 has an Aura card "Enchant creature" in hand. P1 controls a creature and an artifact (non-creature).
- **Action:** P0 casts the Aura targeting the artifact.
- **Expected Result:** Targeting is illegal — the Aura can only target creatures. The cast fails (or the artifact is not a valid target choice).
- **Phase:** Phase 5-Pre (Aura attachment model)
- **Ticket:** T15b (Aura attachment logic, EnchantRestriction)
- **Tags:** enchant, aura, targeting

**702.5b** — PURE-DEF. Cross-reference to rule 303 (Enchantments). No independent mechanical consequence.

**702.5c** — Multiple enchant instances: Aura must satisfy ALL.

**ATOM-702.5c-001**
- **Rule:** 702.5c — If an Aura has multiple instances of enchant, all of them apply. The Aura's target must follow the restrictions from all the instances of enchant.
- **Mechanism:** Multiple enchant restrictions compose via AND logic
- **Minimal Board:** P0 has an Aura with "Enchant creature" and "Enchant green permanent" (granted by a text-changing or ability-granting effect). P1 controls a green creature and a red creature.
- **Action:** P0 casts the Aura targeting the red creature.
- **Expected Result:** Targeting is illegal — the Aura must enchant an object that is both a creature AND green. Only the green creature is a valid target.
- **Phase:** Phase 8 (multiple enchant instances are rare; requires ability-granting infrastructure)
- **Ticket:** DEFERRED — Phase 8. Edge case requiring granted enchant abilities.
- **Tags:** enchant, aura, multiple-instances, DEFERRED

**702.5d** — Auras that enchant players can target players, not permanents.

**ATOM-702.5d-001**
- **Rule:** 702.5d — Auras that can enchant a player can target and be attached to players. Such Auras can't target permanents and can't be attached to permanents.
- **Mechanism:** "Enchant player" Aura targeting restriction
- **Minimal Board:** P0 has an Aura with "Enchant player" in hand. P1 controls a creature.
- **Action:** P0 attempts to cast the Aura targeting P1's creature.
- **Expected Result:** Targeting is illegal — "Enchant player" Auras can only target players. Casting targeting P1 (the player) succeeds; targeting P1's creature fails.
- **Phase:** Phase 5-Pre (Aura targeting model)
- **Ticket:** T15b (Aura attachment logic)
- **Tags:** enchant, aura, enchant-player

### 702.6 — Equip

**702.6a** — Equip is an activated ability: "Attach to target creature you control. Sorcery speed."

**ATOM-702.6a-001**
- **Rule:** 702.6a — Equip [cost] means "[Cost]: Attach this permanent to target creature you control. Activate only as a sorcery."
- **Mechanism:** Equip ability activation, attachment, sorcery-speed restriction
- **Minimal Board:** P0 controls an Equipment with "Equip {2}" and a creature. It is P0's main phase, stack is empty.
- **Action:** P0 activates equip, paying {2}, targeting their creature.
- **Expected Result:** Equipment becomes attached to the creature (`attached_to` = creature, creature's `attached_by` includes Equipment). Activation at instant speed is illegal.
- **Phase:** Phase 5-Pre (attachment tracking + equip ability)
- **Ticket:** T15b (Aura/Equipment attachment logic)
- **Tags:** equip, attachment, sorcery-speed

**ATOM-702.6a-002**
- **Rule:** 702.6a — Equip targets only creatures the Equipment's controller controls.
- **Mechanism:** Equip targeting restriction: own creatures only
- **Minimal Board:** P0 controls an Equipment with "Equip {1}". P1 controls a creature. P0 controls no creatures.
- **Action:** P0 attempts to activate equip targeting P1's creature.
- **Expected Result:** Targeting is illegal — equip can only target creatures you control.
- **Phase:** Phase 5-Pre
- **Ticket:** T15b
- **Tags:** equip, targeting, controller

**ATOM-702.6a-003**
- **Rule:** 702.6a — Equip can only be activated as a sorcery (active player, main phase, empty stack).
- **Mechanism:** Sorcery-speed enforcement on equip activation
- **Minimal Board:** P0 controls an Equipment with "Equip {2}" and a creature. It is P1's turn (or P0's combat phase, or the stack is non-empty).
- **Action:** P0 attempts to activate equip.
- **Expected Result:** Activation is illegal — equip can only be activated at sorcery speed.
- **Phase:** Phase 5-Pre
- **Ticket:** T15b
- **Tags:** equip, sorcery-speed, negative-case

**702.6b** — PURE-DEF. Cross-reference to rule 301 (Artifacts). No independent mechanical consequence.

**702.6c** — Equip with quality restrictions (e.g., "Equip Knight").

**ATOM-702.6c-001**
- **Rule:** 702.6c — Equip abilities may further restrict what creatures may be chosen as legal targets. "Equip [quality]" targets only a creature controlled by the activating player that has the chosen quality.
- **Mechanism:** Equip quality restriction on targeting
- **Minimal Board:** P0 controls an Equipment with "Equip Knight {1}" and two creatures: a Knight and a non-Knight.
- **Action:** P0 activates equip targeting the non-Knight creature.
- **Expected Result:** Targeting is illegal. Equip targeting the Knight succeeds.
- **Phase:** Phase 8 (subtype-restricted equip is a card-breadth concern)
- **Ticket:** DEFERRED — Phase 8. Requires subtype checking in equip targeting.
- **Tags:** equip, quality-restriction, DEFERRED

**ATOM-702.6c-002**
- **Rule:** 702.6c — Additional restrictions for an equip ability don't restrict what the Equipment may be attached to (via non-equip means).
- **Mechanism:** Non-equip attachment bypasses equip quality restriction
- **Minimal Board:** P0 controls an Equipment with "Equip Knight {1}" already attached to a Knight. An effect moves the Equipment to a non-Knight creature (e.g., Magnetic Theft).
- **Action:** The effect attaches the Equipment to the non-Knight.
- **Expected Result:** The Equipment is legally attached to the non-Knight. The equip quality restriction only constrains the equip activated ability, not all attachment.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires non-equip attachment effects.
- **Tags:** equip, quality-restriction, non-equip-attachment, DEFERRED

**702.6d** — Multiple equip abilities on the same Equipment: any may be activated independently.

**ATOM-702.6d-001**
- **Rule:** 702.6d — If a permanent has multiple equip abilities, any may be activated. Generic equip targets any creature; type-restricted equip targets only matching creatures.
- **Mechanism:** Multiple equip abilities with different restrictions coexist independently
- **Minimal Board:** P0 controls an Equipment with "Equip {4}" and "Equip Knight {1}". P0 controls a Knight creature and a non-Knight creature.
- **Action:** P0 activates "Equip Knight {1}" targeting the non-Knight. Then P0 activates "Equip {4}" targeting the non-Knight.
- **Expected Result:** "Equip Knight" targeting non-Knight is illegal. "Equip {4}" targeting non-Knight is legal — the generic equip ability has no type restriction. The type-restricted ability doesn't infect the generic ability.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires multiple equip abilities + subtype checking.
- **Tags:** equip, multiple-abilities, quality-restriction, DEFERRED

**702.6e** — "Equip planeswalker" variant.

**ATOM-702.6e-001**
- **Rule:** 702.6e — "Equip planeswalker [cost]" means "[Cost]: Attach this permanent to target planeswalker you control as though that planeswalker were a creature. Activate only as a sorcery."
- **Mechanism:** Equip planeswalker variant attaches to planeswalkers
- **Minimal Board:** P0 controls an Equipment with "Equip planeswalker {2}" and a planeswalker.
- **Action:** P0 activates equip planeswalker targeting their planeswalker.
- **Expected Result:** Equipment becomes attached to the planeswalker. Normal equip on the same Equipment (if it has one) can't target the planeswalker.
- **Phase:** Phase 8 (planeswalker equipment is niche)
- **Ticket:** DEFERRED — Phase 8. Requires planeswalker cards + equip variant.
- **Tags:** equip, planeswalker, DEFERRED

### 702.7 — First Strike

**702.7a** — PURE-DEF. "First strike is a static ability that modifies the rules for the combat damage step." Prerequisite for 702.7b.

**702.7b** — ALREADY-IMPLEMENTED. First strike creatures assign damage only in the first combat damage step; a second step follows. Implemented in `engine/combat/keywords.rs` `should_deal_damage_this_step`.

**702.7c** — Gain/remove first strike mid-combat.

**ATOM-702.7c-001**
- **Rule:** 702.7c — Giving first strike to a creature without it after combat damage has already been dealt in the first combat damage step won't preclude that creature from assigning combat damage in the second combat damage step.
- **Mechanism:** Gaining first strike after first damage step doesn't prevent second step damage
- **Minimal Board:** P0 controls a 3/3 creature (no first strike) attacking unblocked. After first-strike damage step (in which it did not participate), an effect gives it first strike.
- **Action:** Second combat damage step begins.
- **Expected Result:** The 3/3 still assigns 3 damage in the second step. Gaining first strike after the first step doesn't retroactively exclude it.
- **Phase:** Phase 5 (continuous effects — mid-combat keyword changes)
- **Ticket:** DEFERRED — Phase 5 Layers.
- **Tags:** first-strike, mid-combat, continuous-effects, DEFERRED

**ATOM-702.7c-002**
- **Rule:** 702.7c — Removing first strike from a creature after it has already dealt combat damage in the first combat damage step won't allow it to also assign combat damage in the second combat damage step (unless the creature has double strike).
- **Mechanism:** Removing first strike after first step doesn't grant second step damage
- **Minimal Board:** P0 controls a 2/2 creature with first strike, attacking unblocked. After it deals 2 damage in the first step, an effect removes first strike.
- **Action:** Second combat damage step begins.
- **Expected Result:** The 2/2 does NOT assign damage in the second step. It already dealt damage in the first step, and losing first strike doesn't retroactively add it to the second step pool (it had first strike at the start of the first step, so it was excluded from the second step pool).
- **Phase:** Phase 5 (continuous effects — mid-combat keyword changes)
- **Ticket:** DEFERRED — Phase 5 Layers.
- **Tags:** first-strike, mid-combat, continuous-effects, DEFERRED

**702.7d** — ALREADY-IMPLEMENTED. Multiple instances of first strike are redundant. Engine uses `has_keyword` boolean check.

### 702.8 — Flash

**702.8a** — Flash allows casting at instant speed from any relevant zone.

**ATOM-702.8a-001**
- **Rule:** 702.8a — Flash is a static ability. "Flash" means "You may play this card any time you could cast an instant."
- **Mechanism:** Flash bypasses sorcery-speed timing restriction
- **Minimal Board:** P0 has a creature with flash in hand. It is P1's turn (P0 is not the active player), or it is P0's turn during the combat phase with an empty stack.
- **Action:** P0 casts the creature with flash.
- **Expected Result:** Cast is legal — flash allows casting at instant speed. Without flash, a creature could only be cast at sorcery speed (active player, main phase, stack empty).
- **Phase:** Phase 5-Pre (casting pipeline)
- **Ticket:** T18 (601.2-compliant casting pipeline — flash timing check)
- **Tags:** flash, timing, instant-speed

**ATOM-702.8a-002**
- **Rule:** 702.8a — Flash functions from any zone from which you could play the card.
- **Mechanism:** Flash works from non-hand zones (e.g., graveyard with a cast-from-graveyard permission)
- **Minimal Board:** P0 has a creature with flash in graveyard, plus an effect granting "you may cast creature cards from your graveyard." It is P1's turn.
- **Action:** P0 casts the flash creature from graveyard during P1's turn.
- **Expected Result:** Cast is legal — flash applies in the graveyard zone because P0 could play the card from there, and flash overrides sorcery timing.
- **Phase:** Phase 8 (zone-casting permissions)
- **Ticket:** DEFERRED — Phase 8. Requires cast-from-graveyard permission infrastructure.
- **Tags:** flash, any-zone, DEFERRED

**702.8b** — PURE-DEF. Multiple instances of flash are redundant. No independent mechanical consequence (boolean check).

### 702.9 — Flying

**702.9a** — PURE-DEF. "Flying is an evasion ability." No independent mechanical consequence.

**702.9b** — ALREADY-IMPLEMENTED. Creature with flying can't be blocked except by creatures with flying and/or reach. Implemented in `engine/combat/validation.rs` per-pair check.

**702.9c** — ALREADY-IMPLEMENTED. Multiple instances of flying are redundant. Engine uses `has_keyword` boolean check.

### 702.10 — Haste

**702.10a** — PURE-DEF. "Haste is a static ability." No independent mechanical consequence.

**702.10b** — ALREADY-IMPLEMENTED. Haste allows attacking without continuous control since turn start. Implemented in `oracle/legality.rs` `can_attack`.

**702.10c** — ALREADY-IMPLEMENTED. Haste allows tap/untap symbol ability activation despite summoning sickness. Implemented in `engine/costs.rs` Cost::Tap and Cost::Untap checks.

**702.10d** — ALREADY-IMPLEMENTED. Multiple instances of haste are redundant. Engine uses `has_keyword` boolean check.

--- End of Chunk 2 ---

## Chunk 3 — 702.11–702.16 (Hexproof, Indestructible, Intimidate, Landwalk, Lifelink, Protection)

### 702.11 — Hexproof

**702.11a** — PURE-DEF. "Hexproof is a static ability." No independent mechanical consequence.

**702.11b** — Hexproof on a permanent prevents targeting by opponents' spells/abilities.

**ATOM-702.11b-001**
- **Rule:** 702.11b — "Hexproof" on a permanent means "This permanent can't be the target of spells or abilities your opponents control."
- **Mechanism:** Hexproof targeting restriction on permanents
- **Minimal Board:** P0 controls a creature with hexproof. P1 has a spell "Destroy target creature" in hand.
- **Action:** P1 casts the spell targeting P0's hexproof creature.
- **Expected Result:** Targeting is illegal — hexproof prevents opponents from targeting. P0 casting a spell targeting their own hexproof creature IS legal.
- **Phase:** Phase 5-Pre
- **Ticket:** T22 (hexproof/shroud/protection targeting restrictions)
- **Tags:** hexproof, targeting

**ATOM-702.11b-002**
- **Rule:** 702.11b — Hexproof allows the controller to target their own hexproof permanent.
- **Mechanism:** Hexproof only blocks opponent targeting, not own
- **Minimal Board:** P0 controls a creature with hexproof. P0 has a spell "Target creature gets +3/+3 until end of turn" in hand.
- **Action:** P0 casts the spell targeting their own hexproof creature.
- **Expected Result:** Targeting is legal — hexproof only restricts opponents.
- **Phase:** Phase 5-Pre
- **Ticket:** T22
- **Tags:** hexproof, self-targeting

**702.11c** — Hexproof on a player.

**ATOM-702.11c-001**
- **Rule:** 702.11c — "Hexproof" on a player means "You can't be the target of spells or abilities your opponents control."
- **Mechanism:** Hexproof on player prevents opponent targeting
- **Minimal Board:** P0 has hexproof (e.g., from Leyline of Sanctity). P1 has a spell "Target player discards two cards."
- **Action:** P1 casts the spell targeting P0.
- **Expected Result:** Targeting is illegal — P0 has hexproof and can't be targeted by P1's spells.
- **Phase:** Phase 8 (player hexproof requires player-targeting framework)
- **Ticket:** DEFERRED — Phase 8. Requires player-as-target infrastructure.
- **Tags:** hexproof, player, DEFERRED

**702.11d** — "Hexproof from [quality]" variant.

**ATOM-702.11d-001**
- **Rule:** 702.11d — "Hexproof from [quality]" on a permanent means "This permanent can't be the target of [quality] spells your opponents control or abilities your opponents control from [quality] sources."
- **Mechanism:** Conditional hexproof restricts targeting by quality
- **Minimal Board:** P0 controls a creature with "hexproof from black." P1 has a black spell (e.g., Doom Blade "Destroy target nonblack creature") and a red spell (e.g., Lightning Bolt).
- **Action:** P1 casts Doom Blade targeting P0's creature. Then P1 casts Lightning Bolt targeting P0's creature.
- **Expected Result:** Doom Blade targeting is illegal (black spell, blocked by hexproof from black). Lightning Bolt targeting is legal (red spell, not blocked).
- **Phase:** Phase 8 (hexproof-from-quality requires color-checking in targeting)
- **Ticket:** DEFERRED — Phase 8. Requires quality-filtered targeting restrictions.
- **Tags:** hexproof, hexproof-from, quality, DEFERRED

**702.11e** — Losing hexproof also loses all "hexproof from [quality]" abilities; effects that bypass hexproof also bypass hexproof-from.

**ATOM-702.11e-001**
- **Rule:** 702.11e — Any effect that causes an object to lose hexproof will cause an object to lose all "hexproof from [quality]" abilities.
- **Mechanism:** Losing hexproof strips all hexproof-from variants
- **Minimal Board:** P0 controls a creature with "hexproof from black" and "hexproof from red." An effect says "target creature loses hexproof."
- **Action:** The effect resolves, removing hexproof.
- **Expected Result:** The creature loses both "hexproof from black" and "hexproof from red." It can now be targeted by black and red spells from opponents.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires hexproof-from implementation + ability removal.
- **Tags:** hexproof, hexproof-from, ability-removal, DEFERRED

**702.11f** — "Hexproof from [A] and from [B]" = two separate hexproof abilities. PURE-DEF (shorthand expansion). Mechanical consequence tested via 702.11d and 702.11e.

**702.11g** — "Hexproof from each [characteristic]" = multiple separate hexproof abilities. PURE-DEF (shorthand expansion). Same as 702.11f.

**702.11h** — PURE-DEF. Multiple instances of the same hexproof ability are redundant. Boolean check.

### 702.12 — Indestructible

**702.12a** — PURE-DEF. "Indestructible is a static ability." No independent mechanical consequence.

**702.12b** — Indestructible prevents destruction and ignores lethal-damage SBA.

**ATOM-702.12b-001**
- **Rule:** 702.12b — A permanent with indestructible can't be destroyed. Such permanents aren't destroyed by lethal damage, and they ignore the state-based action that checks for lethal damage (see rule 704.5g).
- **Mechanism:** Indestructible prevents destruction by lethal damage
- **Minimal Board:** P0 controls a 2/2 creature with indestructible. P1 casts a spell dealing 5 damage to it.
- **Action:** 5 damage is dealt to the 2/2 indestructible creature. SBAs are checked.
- **Expected Result:** The creature is NOT destroyed. It has 5 damage marked but SBA 704.5g is skipped for indestructible permanents. The creature remains on the battlefield.
- **Phase:** Phase 5-Pre
- **Ticket:** T09 (indestructible implementation)
- **Tags:** indestructible, lethal-damage, SBA

**ATOM-702.12b-002**
- **Rule:** 702.12b — A permanent with indestructible can't be destroyed by "destroy" effects.
- **Mechanism:** Indestructible prevents destruction by destroy effects
- **Minimal Board:** P0 controls a creature with indestructible. P1 casts "Destroy target creature" targeting it.
- **Action:** The destroy spell resolves.
- **Expected Result:** The creature is NOT destroyed. It remains on the battlefield. The destroy effect is simply not applied.
- **Phase:** Phase 5-Pre
- **Ticket:** T09
- **Tags:** indestructible, destroy-effect

**702.12c** — PURE-DEF. Multiple instances of indestructible are redundant. Boolean check.

### 702.13 — Intimidate

**702.13a** — PURE-DEF. "Intimidate is an evasion ability." No independent mechanical consequence.

**702.13b** — Creature with intimidate can't be blocked except by artifact creatures and/or creatures sharing a color.

**ATOM-702.13b-001**
- **Rule:** 702.13b — A creature with intimidate can't be blocked except by artifact creatures and/or creatures that share a color with it.
- **Mechanism:** Intimidate evasion: blocker must be artifact creature or share color
- **Minimal Board:** P0 controls a red creature with intimidate, attacking. P1 controls: a red creature (shares color), a green creature (doesn't share), and an artifact creature.
- **Action:** P1 declares blockers.
- **Expected Result:** The red creature and artifact creature can legally block. The green creature cannot block (doesn't share a color and isn't an artifact creature).
- **Phase:** Phase 8 (intimidate is a deprecated keyword but still in CR)
- **Ticket:** DEFERRED — Phase 8. Requires color-sharing check in blocker validation.
- **Tags:** intimidate, evasion, blocking, DEFERRED

**702.13c** — PURE-DEF. Multiple instances of intimidate are redundant. Boolean check.

### 702.14 — Landwalk

**702.14a** — PURE-DEF. Defines landwalk as a generic term "[type]walk." Prerequisite for 702.14c.

**702.14b** — PURE-DEF. "Landwalk is an evasion ability." No independent mechanical consequence.

**702.14c** — Creature with landwalk can't be blocked if defending player controls matching land.

**ATOM-702.14c-001**
- **Rule:** 702.14c — A creature with landwalk can't be blocked as long as the defending player controls at least one land with the specified land type.
- **Mechanism:** Landwalk evasion: unblockable if defender controls matching land
- **Minimal Board:** P0 controls a creature with islandwalk, attacking. P1 controls an Island and a creature.
- **Action:** P1 declares blockers, attempting to block the islandwalker.
- **Expected Result:** Blocking is illegal — P1 controls an Island, so the creature with islandwalk can't be blocked.
- **Phase:** Phase 8 (landwalk)
- **Ticket:** DEFERRED — Phase 8. Requires land-type checking in blocker validation.
- **Tags:** landwalk, evasion, blocking, DEFERRED

**ATOM-702.14c-002**
- **Rule:** 702.14c — Landwalk doesn't apply if the defending player doesn't control a matching land.
- **Mechanism:** Landwalk evasion inapplicable when no matching land
- **Minimal Board:** P0 controls a creature with islandwalk, attacking. P1 controls only Mountains and a creature.
- **Action:** P1 declares blockers, blocking the islandwalker.
- **Expected Result:** Blocking is legal — P1 doesn't control an Island, so islandwalk doesn't prevent blocking.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** landwalk, evasion, blocking, DEFERRED

**702.14d** — Landwalk abilities don't cancel one another.

**ATOM-702.14d-001**
- **Rule:** 702.14d — Landwalk abilities don't "cancel" one another.
- **Mechanism:** Landwalk on both attacker and blocker doesn't negate evasion
- **Minimal Board:** P0 controls a creature with forestwalk, attacking. P1 controls a snow Forest and a creature with forestwalk.
- **Action:** P1 attempts to block P0's forestwalk creature with their forestwalk creature.
- **Expected Result:** Blocking is illegal — P1 controls a Forest, so P0's creature with forestwalk can't be blocked. P1's creature having forestwalk doesn't cancel P0's forestwalk.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** landwalk, evasion, no-cancel, DEFERRED

**702.14e** — PURE-DEF. Multiple instances of the same kind of landwalk are redundant. Boolean check per landwalk type.

### 702.15 — Lifelink

**702.15a** — PURE-DEF. "Lifelink is a static ability." No independent mechanical consequence.

**702.15b** — ALREADY-IMPLEMENTED. Damage dealt by a source with lifelink → controller gains that much life. Implemented in `engine/keywords.rs` `apply_lifelink`.

**702.15c** — LKI determines lifelink after zone change.

**ATOM-702.15c-001**
- **Rule:** 702.15c — If an object changes zones before an effect causes it to deal damage, its last known information is used to determine whether it had lifelink.
- **Mechanism:** LKI snapshot preserves lifelink status for pending damage after zone change
- **Minimal Board:** P0 controls a creature with lifelink that has a triggered ability "when this creature dies, it deals 3 damage to target player." P0 is at 10 life.
- **Action:** P0's lifelink creature dies. The death trigger deals 3 damage to P1. LKI determines the source had lifelink.
- **Expected Result:** P0 gains 3 life (now at 13). The damage source's LKI indicates lifelink, so the lifelink rules apply despite the source being in the graveyard.
- **Phase:** Phase 5 (LKI system — L19/T20b)
- **Ticket:** T20b (LKI system)
- **Tags:** lifelink, LKI, DEFERRED

**702.15d** — Lifelink functions from any zone.

**ATOM-702.15d-001**
- **Rule:** 702.15d — The lifelink rules function no matter what zone an object with lifelink deals damage from.
- **Mechanism:** Lifelink works when damage source is in a non-battlefield zone
- **Minimal Board:** P0 controls a source with lifelink in graveyard that deals damage via an activated ability (e.g., "exile from graveyard: deal 2 damage to any target"). P0 is at 10 life.
- **Action:** Activate the graveyard ability, dealing 2 damage to P1.
- **Expected Result:** P0 gains 2 life (now at 12). Lifelink applies regardless of the source's zone.
- **Phase:** Phase 6 (any-zone ability infrastructure)
- **Ticket:** DEFERRED — Phase 6. Requires zone-activated abilities.
- **Tags:** lifelink, any-zone, DEFERRED

**702.15e** — ALREADY-IMPLEMENTED. Multiple lifelink sources dealing damage simultaneously cause separate life-gain events. Implemented via per-source damage application in `engine/keywords.rs`.

**702.15f** — ALREADY-IMPLEMENTED. Multiple instances of lifelink on the same object are redundant. Engine uses `has_keyword` boolean check.

### 702.16 — Protection

**702.16a** — Protection definition and quality resolution.

**ATOM-702.16a-001**
- **Rule:** 702.16a — Protection is a static ability, written "Protection from [quality]."
- **Mechanism:** Protection ability exists and is queryable
- **Minimal Board:** P0 controls a creature with "protection from red."
- **Action:** Query the creature's abilities.
- **Expected Result:** The creature has a protection ability with quality = red. This is a prerequisite for all protection sub-rules.
- **Phase:** Phase 5-Pre
- **Ticket:** T22 (protection implementation)
- **Tags:** protection, ability-query

**ATOM-702.16a-002**
- **Rule:** 702.16a — Protection from [cardname] blocks targeting/damage from permanents with that specific name.
- **Mechanism:** Protection quality matching by card name
- **Minimal Board:** P0 controls a creature with "protection from Grizzly Bears." P1 controls a Grizzly Bears and another creature (e.g., Hill Giant).
- **Action:** Both of P1's creatures attempt to deal combat damage to P0's protected creature (as blockers or attackers in relevant scenarios).
- **Expected Result:** Damage from Grizzly Bears is prevented; damage from Hill Giant is dealt normally. Abilities from the source named "Grizzly Bears" also cannot target the protected creature.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires name-based quality matching in protection.
- **Tags:** protection, protection-from-cardname, DEFERRED

**ATOM-702.16a-003**
- **Rule:** 702.16a — Protection from snow blocks targeting/damage from all snow sources: snow permanents, snow instants, snow sorceries, etc.
- **Mechanism:** Protection quality matching by supertype (applies to all object types, not just permanents)
- **Minimal Board:** P0 controls a creature with "protection from snow." P1 controls a Snow creature and a non-Snow creature. P1 also has a Snow instant (e.g., Skred — "deal damage to target creature") and a non-Snow instant (e.g., Lightning Bolt).
- **Action:** (1) P1's Snow creature and non-Snow creature attempt to deal combat damage to P0's protected creature. (2) P1 casts Skred targeting P0's protected creature. (3) P1 casts Lightning Bolt targeting P0's protected creature.
- **Expected Result:** (1) Damage from the Snow creature is prevented; damage from the non-Snow creature is dealt normally. (2) Skred cannot target the protected creature — it is a Snow spell, blocked by protection. (3) Lightning Bolt CAN target the protected creature — it is not Snow. Protection from snow applies to any source with the Snow supertype, regardless of whether that source is a permanent, spell, or ability.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires supertype-based quality matching in protection across all source types.
- **Tags:** protection, protection-from-snow, supertype, snow-spell, DEFERRED

**702.16b** — Protection prevents targeting by spells/abilities with the stated quality.

**ATOM-702.16b-001**
- **Rule:** 702.16b — A permanent or player with protection can't be targeted by spells with the stated quality and can't be targeted by abilities from a source with the stated quality.
- **Mechanism:** Protection targeting restriction
- **Minimal Board:** P0 controls a creature with "protection from red." P1 has Lightning Bolt (red instant, "deal 3 damage to any target").
- **Action:** P1 casts Lightning Bolt targeting P0's creature.
- **Expected Result:** Targeting is illegal — the creature has protection from red, and Lightning Bolt is a red spell.
- **Phase:** Phase 5-Pre
- **Ticket:** T22
- **Tags:** protection, targeting

**ATOM-702.16b-002**
- **Rule:** 702.16b — Protection doesn't prevent targeting by spells/abilities without the stated quality.
- **Mechanism:** Protection targeting restriction only applies to matching quality
- **Minimal Board:** P0 controls a creature with "protection from red." P1 has a blue spell "Unsummon: return target creature to its owner's hand."
- **Action:** P1 casts Unsummon targeting P0's creature.
- **Expected Result:** Targeting is legal — Unsummon is blue, not red. Protection from red doesn't block blue spells.
- **Phase:** Phase 5-Pre
- **Ticket:** T22
- **Tags:** protection, targeting, non-matching

**702.16c** — Protection prevents enchantment by Auras with the stated quality; such Auras fall off as SBA.

**ATOM-702.16c-001**
- **Rule:** 702.16c — A permanent with protection can't be enchanted by Auras that have the stated quality. Such Auras are put into their owners' graveyards as a state-based action.
- **Mechanism:** Protection causes illegal Auras to fall off as SBA
- **Minimal Board:** P0 controls a creature enchanted by a red Aura. P0's creature gains "protection from red" (e.g., via a spell resolving).
- **Action:** SBAs are checked.
- **Expected Result:** The red Aura is put into its owner's graveyard. The creature can't be enchanted by a red Aura while it has protection from red.
- **Phase:** Phase 5-Pre
- **Ticket:** T22
- **Tags:** protection, aura, SBA

**702.16d** — Protection causes Equipment/Fortifications with the stated quality to become unattached (SBA), but remain on the battlefield.

**ATOM-702.16d-001**
- **Rule:** 702.16d — A permanent with protection can't be equipped by Equipment that have the stated quality. Such Equipment become unattached as a state-based action, but remain on the battlefield.
- **Mechanism:** Protection causes illegal Equipment to detach as SBA
- **Minimal Board:** P0 controls a creature equipped with a red Equipment. P0's creature gains "protection from red."
- **Action:** SBAs are checked.
- **Expected Result:** The red Equipment becomes unattached from the creature but remains on the battlefield (not destroyed, not sent to graveyard).
- **Phase:** Phase 5-Pre
- **Ticket:** T22
- **Tags:** protection, equipment, SBA

**702.16e** — Protection prevents all damage from sources with the stated quality.

**ATOM-702.16e-001**
- **Rule:** 702.16e — Any damage that would be dealt by sources that have the stated quality to a permanent or player with protection is prevented.
- **Mechanism:** Protection damage prevention in combat
- **Minimal Board:** P0 controls a 2/2 creature with "protection from red," blocking a 3/3 red attacking creature controlled by P1.
- **Action:** Combat damage step: the 3/3 red creature assigns 3 damage to the protected blocker.
- **Expected Result:** All 3 damage from the red source is prevented. The protected creature takes 0 damage and survives. The protected creature still deals its 2 damage to the red attacker normally (protection prevents incoming damage, not outgoing).
- **Phase:** Phase 5-Pre
- **Ticket:** T22
- **Tags:** protection, damage-prevention, combat

**702.16f** — Protection prevents blocking by creatures with the stated quality.

**ATOM-702.16f-001**
- **Rule:** 702.16f — Attacking creatures with protection can't be blocked by creatures that have the stated quality.
- **Mechanism:** Protection evasion: can't be blocked by matching quality
- **Minimal Board:** P0 controls an attacking creature with "protection from green." P1 controls a green creature and a red creature.
- **Action:** P1 declares blockers.
- **Expected Result:** The green creature can't block (matches protection quality). The red creature can legally block.
- **Phase:** Phase 5-Pre
- **Ticket:** T22
- **Tags:** protection, blocking, evasion

**702.16g** — "Protection from [A] and from [B]" = two separate protection abilities. PURE-DEF (shorthand expansion). Mechanical consequence tested via 702.16b–f.

**702.16h** — "Protection from each [characteristic]" = multiple separate protection abilities. PURE-DEF (shorthand expansion).

**702.16i** — "Protection from each [set]" = multiple separate protection abilities. PURE-DEF (shorthand expansion).

**702.16j** — "Protection from everything" variant.

**ATOM-702.16j-001**
- **Rule:** 702.16j — A permanent or player with protection from everything has protection from each object regardless of characteristic values. Can't be targeted, enchanted, equipped, fortified, or blocked. All damage prevented.
- **Mechanism:** Protection from everything applies to all objects
- **Minimal Board:** P0 controls a creature with "protection from everything," attacking. P1 controls creatures and has spells.
- **Action:** P1 attempts to: (1) target the creature with a spell, (2) block the creature.
- **Expected Result:** (1) Targeting is illegal. (2) Blocking is illegal — no creature can block a creature with protection from everything. All damage to it is prevented.
- **Phase:** Phase 8 (protection from everything is a rare variant)
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** protection, protection-from-everything, DEFERRED

**702.16k** — "Protection from [a player]" variant.

**ATOM-702.16k-001**
- **Rule:** 702.16k — A permanent with protection from a specific player has protection from each object that player controls and from each object that player owns not controlled by another player.
- **Mechanism:** Protection from a player blocks that player's objects
- **Minimal Board:** P0 controls a creature with "protection from P1." P1 controls a creature and has a spell.
- **Action:** P1 attempts to (1) target P0's creature with a spell, (2) block with their creature, (3) deal combat damage.
- **Expected Result:** All three are prevented — spells P1 controls can't target it, creatures P1 controls can't block it, damage from sources P1 controls is prevented.
- **Phase:** Phase 9 (multiplayer/Commander)
- **Ticket:** DEFERRED — Phase 9.
- **Tags:** protection, protection-from-player, multiplayer, DEFERRED

**ATOM-702.16k-002**
- **Rule:** 702.16k — Protection from [player] applies to objects that player controls, not objects that player owns but another player controls.
- **Mechanism:** Protection from player checks controller, not owner
- **Minimal Board:** 3-player game. P0 controls a creature with "protection from P1." P2 controls a permanent owned by P1 (stolen via a control-change effect).
- **Action:** P2's stolen permanent (owned by P1, controlled by P2) attempts to deal damage to P0's protected creature.
- **Expected Result:** Damage is NOT prevented — P2 controls the permanent, not P1. Protection from P1 only applies to objects P1 controls, not objects P1 merely owns.
- **Phase:** Phase 9 (multiplayer/Commander)
- **Ticket:** DEFERRED — Phase 9.
- **Tags:** protection, protection-from-player, multiplayer, control-vs-ownership, DEFERRED

**702.16m** — PURE-DEF. Multiple instances of protection from the same quality are redundant.

**702.16n** — Auras that grant protection and say "this effect doesn't remove" themselves.

**ATOM-702.16n-001**
- **Rule:** 702.16n — Some Auras give protection and say "this effect doesn't remove" that Aura. The specified Auras aren't put into their owners' graveyards as SBA.
- **Mechanism:** Self-exception on protection Auras prevents self-detachment SBA
- **Minimal Board:** P0 controls a creature enchanted by an Aura that grants "protection from white. This effect doesn't remove Auras." The Aura itself is white.
- **Action:** SBAs are checked.
- **Expected Result:** The Aura remains attached despite being white and the creature having protection from white. Other white Auras without the exception would fall off.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires per-Aura exception tracking on protection SBA.
- **Tags:** protection, aura, self-exception, DEFERRED

**702.16p** — Benevolent Blessing "already attached" variant. DEFERRED — Phase 8 (extremely niche, single card). Similar to 702.16n but with "already attached" timing. No separate ATOM test; covered by 702.16n's framework.

--- End of Chunk 3 ---

## Chunk 4 — 702.17–702.22 (Reach, Shroud, Trample, Vigilance, Ward, Banding)

### 702.17 — Reach

**702.17a** — PURE-DEF. "Reach is a static ability." No independent mechanical consequence.

**702.17b** — ALREADY-IMPLEMENTED. Creature with flying can't be blocked except by creatures with flying and/or reach. Implemented in `engine/combat/validation.rs` per-pair check. (Note: 702.17b restates flying's blocking rule from reach's perspective.)

**702.17c** — ALREADY-IMPLEMENTED. Multiple instances of reach are redundant. Engine uses `has_keyword` boolean check.

### 702.18 — Shroud

**702.18a** — Shroud prevents targeting by ALL spells/abilities (including controller's own).

**ATOM-702.18a-001**
- **Rule:** 702.18a — Shroud means "This permanent or player can't be the target of spells or abilities."
- **Mechanism:** Shroud targeting restriction — blocks all targeting, including own controller
- **Minimal Board:** P0 controls a creature with shroud. P0 has a spell "Target creature gets +3/+3 until end of turn."
- **Action:** P0 casts the spell targeting their own shroud creature.
- **Expected Result:** Targeting is illegal — shroud prevents ALL targeting, including by the creature's own controller. This differs from hexproof (which allows own controller's targeting).
- **Phase:** Phase 5-Pre
- **Ticket:** T22 (hexproof/shroud/protection targeting restrictions)
- **Tags:** shroud, targeting

**ATOM-702.18a-002**
- **Rule:** 702.18a — Shroud on a player prevents targeting by all spells/abilities.
- **Mechanism:** Shroud on player prevents all targeting
- **Minimal Board:** P0 has shroud (e.g., from Imperial Mask). P0 has a spell "Target player draws 3 cards."
- **Action:** P0 casts the spell targeting themselves.
- **Expected Result:** Targeting is illegal — shroud on a player prevents ALL targeting, even self-targeting.
- **Phase:** Phase 8 (player shroud requires player-targeting framework)
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** shroud, player, DEFERRED

**ATOM-702.18a-003**
- **Rule:** 702.18a — Shroud also prevents opponents from targeting the permanent.
- **Mechanism:** Shroud blocks opponent targeting (complements ATOM-702.18a-001 which tests controller targeting)
- **Minimal Board:** P0 controls a creature with shroud. P1 has a spell "Destroy target creature."
- **Action:** P1 casts the spell targeting P0's shroud creature.
- **Expected Result:** Targeting is illegal — shroud prevents ALL targeting, including by opponents. (Unlike hexproof, which only blocks opponents, shroud blocks everyone including the controller.)
- **Phase:** Phase 5-Pre
- **Ticket:** T22
- **Tags:** shroud, targeting, opponent

**702.18b** — PURE-DEF. Multiple instances of shroud are redundant. Boolean check.

### 702.19 — Trample

**702.19a** — PURE-DEF/ALREADY-IMPLEMENTED. "Trample is a static ability that modifies the rules for assigning an attacking creature's combat damage. The ability has no effect when a creature with trample is blocking or is dealing noncombat damage." The no-effect-while-blocking clause is implicitly handled — `assign_trample_damage` is only called for attackers.

**702.19b** — ALREADY-IMPLEMENTED. Trample damage assignment: assign lethal to blockers, excess to defending player/PW/battle. Implemented in `engine/combat/keywords.rs` `assign_trample_damage`.

**702.19c** — Trample over planeswalkers variant.

**ATOM-702.19c-001**
- **Rule:** 702.19c — Trample over planeswalkers: after assigning lethal damage to blockers and damage equal to the PW's loyalty, excess may be assigned to the PW's controller.
- **Mechanism:** Trample over planeswalkers allows excess damage through to the PW's controller
- **Minimal Board:** P0 controls a 7/7 with trample over planeswalkers, attacking P1's planeswalker (3 loyalty). P1 controls a 2/2 blocker.
- **Action:** Combat damage assignment: P0 assigns 2 to blocker (lethal), 3 to planeswalker (= loyalty), 2 to P1 (excess).
- **Expected Result:** Blocker takes 2, planeswalker takes 3 (destroyed by SBA — loyalty reaches 0), P1 takes 2 damage. Without trample-over-PW, excess beyond blocker lethal would go only to the PW, not to the player.
- **Phase:** Phase 8 (planeswalker combat)
- **Ticket:** DEFERRED — Phase 8. Requires planeswalker attacking + trample-over-PW variant.
- **Tags:** trample, trample-over-planeswalkers, DEFERRED

**702.19d** — All blockers removed before damage → all damage to defending player.

**ATOM-702.19d-001**
- **Rule:** 702.19d — If an attacking creature with trample is blocked, but there are no creatures blocking it when damage is assigned, its damage is assigned to the defending player and/or planeswalker as though all blocking creatures have been assigned lethal damage.
- **Mechanism:** Trample with no surviving blockers at damage assignment → all damage to defender
- **Minimal Board:** P0 controls a 5/5 with trample, attacking. P1 declares a 2/2 as blocker. Before combat damage, the 2/2 is destroyed (e.g., by a spell).
- **Action:** Combat damage step: no blockers remain. The 5/5 is still "blocked" but has no blocking creatures.
- **Expected Result:** All 5 damage is assigned to the defending player. The trample creature was blocked (so it doesn't deal damage as though unblocked per normal rules), but with 0 blockers to assign lethal to, all damage overflows to the defender.
- **Phase:** Phase 4 (engine likely handles this already — empty `alive_blockers` → all overflow)
- **Ticket:** NEW — Explicit regression test for trample with all-blockers-removed. Verify `assign_trample_damage` handles empty blocker list.
- **Tags:** trample, all-blockers-removed, regression

**702.19e** — Trample over planeswalkers: PW removed from combat → damage to defending player after lethal to blockers.

**ATOM-702.19e-001**
- **Rule:** 702.19e — If a creature with trample over planeswalkers is attacking a planeswalker and that planeswalker is removed from combat, the creature's excess damage may be assigned to the defending player once all blocking creatures have been dealt lethal damage.
- **Mechanism:** Trample-over-PW with PW removed: excess goes to player instead of PW
- **Minimal Board:** P0 controls a 6/6 with trample over planeswalkers, attacking P1's planeswalker. P1 blocks with a 2/2. Before damage, the planeswalker is removed from combat.
- **Action:** Combat damage assignment.
- **Expected Result:** 2 damage to blocker (lethal), 4 damage to P1 (defending player). The planeswalker is no longer a valid damage target, so excess flows to the player. This is an exception to 506.4c.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires planeswalker combat + removal-from-combat.
- **Tags:** trample, trample-over-planeswalkers, PW-removed, DEFERRED

**702.19f** — Non-trample-over-PW creature attacking PW: no damage to defending player even if PW removed or damage exceeds loyalty.

**ATOM-702.19f-001**
- **Rule:** 702.19f — If a creature without trample over planeswalkers is attacking a planeswalker, none of its combat damage can be assigned to the defending player, even if that planeswalker has been removed from combat or the damage exceeds PW loyalty.
- **Mechanism:** Normal trample (not trample-over-PW) can't overflow to player when attacking PW
- **Minimal Board:** P0 controls a 10/10 with regular trample (not trample-over-PW), attacking P1's planeswalker (3 loyalty). No blockers.
- **Action:** Combat damage assignment.
- **Expected Result:** All 10 damage is assigned to the planeswalker. None can be assigned to P1 (the defending player). Even though 10 > 3 loyalty, regular trample doesn't allow overflow to the player when attacking a planeswalker.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** trample, planeswalker, no-overflow-to-player, DEFERRED

**702.19g** — ALREADY-IMPLEMENTED. Multiple instances of trample are redundant. Multiple instances of trample over planeswalkers are redundant. Engine uses `has_keyword` boolean check.

### 702.20 — Vigilance

**702.20a** — PURE-DEF. "Vigilance is a static ability that modifies the rules for the declare attackers step." Prerequisite for 702.20b.

**702.20b** — ALREADY-IMPLEMENTED. Attacking doesn't cause creatures with vigilance to tap. Implemented via pre-collected `HashSet` in `engine/combat/steps.rs` `process_declare_attackers`.

**702.20c** — ALREADY-IMPLEMENTED. Multiple instances of vigilance are redundant. Engine uses `has_keyword` boolean check.

### 702.21 — Ward

**702.21a** — Ward is a triggered ability: "When targeted by opponent's spell/ability, counter unless they pay [cost]."

**ATOM-702.21a-001**
- **Rule:** 702.21a — Ward [cost] means "Whenever this permanent becomes the target of a spell or ability an opponent controls, counter that spell or ability unless that player pays [cost]."
- **Mechanism:** Ward trigger on being targeted; counter unless cost paid
- **Minimal Board:** P0 controls a creature with "Ward {2}." P1 has Lightning Bolt in hand.
- **Action:** P1 casts Lightning Bolt targeting P0's ward creature. Ward triggers.
- **Expected Result:** Ward trigger goes on the stack above Lightning Bolt. When ward resolves, P1 must pay {2} or Lightning Bolt is countered. If P1 pays {2}, Lightning Bolt resolves normally. If P1 doesn't pay, Lightning Bolt is countered and put into P1's graveyard.
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** NEW — Phase 7: Ward implementation. Triggered ability: on-target trigger + counter-unless-pay resolution.
- **Tags:** ward, triggered-ability, counter-unless-pay

**ATOM-702.21a-002**
- **Rule:** 702.21a — Ward only triggers for opponent's spells/abilities, not controller's own.
- **Mechanism:** Ward does not trigger when controller targets their own ward creature
- **Minimal Board:** P0 controls a creature with "Ward {2}." P0 has a spell "Target creature gets +2/+2."
- **Action:** P0 casts the spell targeting their own ward creature.
- **Expected Result:** Ward does NOT trigger — ward only triggers for spells/abilities an opponent controls. P0's spell resolves normally without any additional cost.
- **Phase:** Phase 7
- **Ticket:** NEW — Phase 7: Ward (same ticket as 702.21a-001).
- **Tags:** ward, triggered-ability, self-targeting

**702.21b** — Ward with X: value determined at resolution, not trigger.

**ATOM-702.21b-001**
- **Rule:** 702.21b — Some ward abilities include an X in their cost and state what X is equal to. This value is determined at the time the ability resolves, not locked in as the ability triggers.
- **Mechanism:** Ward X cost is evaluated at resolution time
- **Minimal Board:** P0 controls a creature with "Ward {X}, where X is the number of creatures you control." P0 controls 2 creatures (including the ward creature). P1 targets the ward creature. Before ward resolves, P0 gains a third creature (e.g., a token enters).
- **Action:** Ward trigger resolves.
- **Expected Result:** P1 must pay {3} (the current count at resolution time, not {2} from trigger time). X is reevaluated at resolution.
- **Phase:** Phase 7 / Phase 8 (ward X is a rare variant)
- **Ticket:** DEFERRED — Phase 8. Requires variable-cost ward + on-resolution evaluation.
- **Tags:** ward, variable-cost, resolution-time, DEFERRED

### 702.22 — Banding

**702.22a** — PURE-DEF. "Banding is a static ability that modifies the rules for combat." Prerequisite for 702.22c–k.

**702.22b** — "Bands with other" is a special form. Losing banding loses all "bands with other." OUT-OF-SCOPE.

**702.22c** — Declaring attacking bands: creatures with banding + up to one without. OUT-OF-SCOPE.

**702.22d** — All creatures in an attacking band must attack the same target. OUT-OF-SCOPE.

**702.22e** — Band persists for rest of combat even if banding is removed. OUT-OF-SCOPE.

**702.22f** — Creature removed from combat is removed from its band. OUT-OF-SCOPE.

**702.22g** — Banding doesn't share abilities between band members. OUT-OF-SCOPE.

**702.22h** — If one creature in a band is blocked, all become blocked by that blocker. OUT-OF-SCOPE.

**702.22i** — If one band member would become blocked by an effect, the entire band becomes blocked. OUT-OF-SCOPE.

**702.22j** — Defending player chooses damage assignment for attacker blocked by a creature with banding. OUT-OF-SCOPE.

**702.22k** — Active player chooses damage assignment for blocker blocking a creature with banding. OUT-OF-SCOPE.

**702.22m** — PURE-DEF. Multiple instances of banding are redundant.

> **Note on Banding:** All sub-rules 702.22b–k are classified as **OUT-OF-SCOPE**. Banding requires three invasive changes to the combat subsystem: (1) band-as-group tracking through declare attackers, declare blockers, and damage assignment — the only mechanic in the game that treats multiple creatures as a single blocking/attacking unit; (2) damage assignment authority rerouting (702.22j/k) that inverts the normal attacker-chooses / defender-chooses flow, interacting with every damage-assignment keyword (trample, deathtouch, wither, etc.); (3) group-blocking semantics where blocking one band member blocks all. The implementation cost is disproportionate: ~15 cards across all of Magic have banding, none are competitively relevant, and every future combat keyword must be banding-aware. Correctness is non-negotiable, and banding's interactions with trample/deathtouch/wither/etc. create a combinatorial testing surface that threatens maintainability. Could be reconsidered as a stretch goal if demand emerges, but not worth the architectural tax for the foreseeable roadmap. Skeleton ATOMs removed.

--- End of Chunk 4 ---

## Chunk 5 — 702.23–702.37 (Rampage, Cumulative Upkeep, Flanking, Phasing, Buyback, Shadow, Cycling, Echo, Horsemanship, Fading, Kicker, Flashback, Madness, Fear, Morph)

### 702.23 — Rampage

**702.23a** — Rampage is a triggered ability: "Whenever this creature becomes blocked, it gets +N/+N until end of turn for each creature blocking it beyond the first."

**ATOM-702.23a-001**
- **Rule:** 702.23a — "Rampage N" means "Whenever this creature becomes blocked, it gets +N/+N until end of turn for each creature blocking it beyond the first."
- **Mechanism:** Rampage triggered ability grants P/T bonus based on blocker count
- **Minimal Board:** P0 controls a 2/2 with "Rampage 2," attacking. P1 blocks with three creatures.
- **Action:** Declare blockers step: rampage triggers (creature became blocked by 3 creatures, beyond the first = 2).
- **Expected Result:** The 2/2 gets +4/+4 (2 × Rampage 2) until end of turn, becoming 6/6.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Niche keyword, triggered ability.
- **Tags:** rampage, triggered-ability, DEFERRED

**702.23b** — Rampage bonus calculated once at resolution; later blocker changes don't affect it. DEFERRED — Phase 8. Covered by ATOM-702.23a-001 (bonus calculated when trigger resolves).

**702.23c** — Multiple instances of rampage each trigger separately. DEFERRED — Phase 8. One-liner.

### 702.24 — Cumulative Upkeep

**702.24a** — Cumulative upkeep: triggered ability adding age counters and requiring increasing payment.

**ATOM-702.24a-001**
- **Rule:** 702.24a — "Cumulative upkeep [cost]" means "At the beginning of your upkeep, put an age counter on this permanent. Then you may pay [cost] for each age counter. If you don't, sacrifice it."
- **Mechanism:** Cumulative upkeep trigger: age counter + escalating cost + sacrifice on non-payment
- **Minimal Board:** P0 controls a creature with "Cumulative upkeep {2}" and 1 age counter already on it. It is P0's upkeep.
- **Action:** Cumulative upkeep triggers. A second age counter is added. P0 must pay {2} × 2 = {4} or sacrifice.
- **Expected Result:** If P0 pays {4}, the creature stays with 2 age counters. If P0 doesn't pay, the creature is sacrificed. Partial payment is not allowed.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Niche keyword, triggered ability with counter management.
- **Tags:** cumulative-upkeep, triggered-ability, age-counters, DEFERRED

**702.24b** — Multiple instances of cumulative upkeep trigger separately; age counters are shared across all instances. DEFERRED — Phase 8. One-liner.

### 702.25 — Flanking

**702.25a** — Flanking: "Whenever this creature becomes blocked by a creature without flanking, the blocking creature gets -1/-1 until end of turn."

**ATOM-702.25a-001**
- **Rule:** 702.25a — "Flanking" means "Whenever this creature becomes blocked by a creature without flanking, the blocking creature gets -1/-1 until end of turn."
- **Mechanism:** Flanking trigger debuffs non-flanking blockers
- **Minimal Board:** P0 controls a 2/2 with flanking, attacking. P1 blocks with a 2/2 without flanking.
- **Action:** Declare blockers: flanking triggers targeting the blocker.
- **Expected Result:** The blocker gets -1/-1 until end of turn, becoming 1/1. If toughness reaches 0, SBAs destroy it before damage.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Niche keyword, triggered ability.
- **Tags:** flanking, triggered-ability, DEFERRED

**702.25b** — Multiple instances of flanking trigger separately. DEFERRED — Phase 8. One-liner.

### 702.26 — Phasing

**702.26a** — Phasing: during untap step, phased-in permanents with phasing phase out; phased-out permanents phase in.

**ATOM-702.26a-001**
- **Rule:** 702.26a — During each player's untap step, before untapping, all phased-in permanents with phasing that player controls phase out. Simultaneously, all phased-out permanents that had phased out under that player's control phase in.
- **Mechanism:** Phasing event during untap step — phase out and phase in simultaneously
- **Minimal Board:** P0 controls a creature with phasing (currently phased in). It is P0's untap step.
- **Action:** Phasing event occurs before untapping.
- **Expected Result:** The creature phases out (status = phased out). While phased out: it can't be targeted, doesn't count as "a creature you control," and is removed from combat if it was in combat. It is treated as not existing until it phases back in on P0's next untap step.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires phased-out status tracking + untap step modification.
- **Tags:** phasing, untap-step, phase-out

**702.26b** — Phased-out permanent is treated as though it doesn't exist; removed from combat.

**ATOM-702.26b-001**
- **Rule:** 702.26b — A phased-out permanent is treated as though it does not exist. It can't affect or be affected by anything. A permanent that phases out is removed from combat.
- **Mechanism:** Phased-out permanents are invisible to game rules
- **Minimal Board:** P0 controls 3 creatures, one of which is phased out. P0 casts "Draw a card for each creature you control."
- **Action:** The spell resolves, counting creatures P0 controls.
- **Expected Result:** P0 draws 2 cards (not 3). The phased-out creature doesn't count.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** phasing, phased-out, invisible

**702.26c** — PURE-DEF. "If a permanent phases in, its status changes to 'phased in.' The game once again treats it as though it exists." Prerequisite for 702.26a.

**702.26d** — Phasing doesn't cause zone changes; no zone-change triggers; tokens persist; counters remain.

**ATOM-702.26d-001**
- **Rule:** 702.26d — The phasing event doesn't cause a permanent to change zones or control. Zone-change triggers don't trigger. Tokens continue to exist. Counters and stickers remain.
- **Mechanism:** Phasing is NOT a zone change — no ETB/LTB triggers, tokens survive, counters persist
- **Minimal Board:** P0 controls a token creature with 2 +1/+1 counters and phasing. It phases out, then phases back in.
- **Action:** Phase out on P0's untap step. Next untap step: phase in.
- **Expected Result:** The token still exists with 2 +1/+1 counters. No "enters the battlefield" or "leaves the battlefield" triggers fired during either phasing event.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** phasing, no-zone-change, tokens, counters

**702.26e** — Continuous effects from spell/ability resolution don't include phased-out permanents in their affected set. DEFERRED — Phase 8. One-liner (requires continuous effects + phasing interaction).

**702.26f** — Continuous effects may expire while a permanent is phased out. "For as long as" durations end when the permanent phases out. DEFERRED — Phase 8. Blocked by Phase 5 layers: "for as long as" duration tracking requires the continuous effect infrastructure to be in place before this interaction can be tested.

**702.26g** — Auras, Equipment, Fortifications attached to a phasing permanent phase out indirectly with it; phase in with it.

**ATOM-702.26g-001**
- **Rule:** 702.26g — When a permanent phases out, any Auras/Equipment/Fortifications attached to it phase out at the same time (indirectly). They phase in along with the permanent they're attached to.
- **Mechanism:** Indirect phasing of attached permanents
- **Minimal Board:** P0 controls a creature with phasing, equipped with an Equipment and enchanted by an Aura. The creature phases out.
- **Action:** Phasing event occurs.
- **Expected Result:** The creature, the Equipment, and the Aura all phase out simultaneously. When the creature phases back in, the Equipment and Aura phase in still attached to it.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** phasing, indirect-phasing, attachment

**702.26h** — If an object would phase out both directly and indirectly, it phases out indirectly. DEFERRED — Phase 8. One-liner.

**702.26i** — Aura/Equipment that phased out directly phases in attached to the same object/player if still valid; otherwise unattached. DEFERRED — Phase 8. One-liner.

**702.26j** — Attach/unattach triggers don't fire on phasing. DEFERRED — Phase 8. One-liner.

**702.26k** — Phased-out permanents owned by a leaving player also leave the game. DEFERRED — Phase 9 (multiplayer). One-liner.

**702.26m** — If untap step is skipped, phasing event doesn't occur. DEFERRED — Phase 8. Blocked by Phase 8 phasing implementation; the test itself is a simple guard (`if untap_step_skipped { skip_phasing_event(); }`) but requires phasing to exist first.

**702.26n** — Multiplayer phasing rules for player who leaves. DEFERRED — Phase 9. One-liner.

**702.26p** — PURE-DEF. Multiple instances of phasing are redundant.

### 702.27 — Buyback

**702.27a** — Buyback: pay additional cost → spell returns to hand instead of graveyard on resolution.

**ATOM-702.27a-001**
- **Rule:** 702.27a — "Buyback [cost]" means "You may pay an additional [cost] as you cast this spell" and "If the buyback cost was paid, put this spell into its owner's hand instead of into that player's graveyard as it resolves."
- **Mechanism:** Buyback additional cost; spell returns to hand on resolution
- **Minimal Board:** P0 has a sorcery with "Buyback {3}" in hand. P0 has enough mana for the spell's cost + {3}.
- **Action:** P0 casts the spell, paying the buyback cost. The spell resolves.
- **Expected Result:** The spell resolves its effect, then goes to P0's hand instead of P0's graveyard.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires additional cost framework (cross-ref T17) + alternative destination on resolution.
- **Tags:** buyback, additional-cost, hand-return, DEFERRED, T17

**ATOM-702.27a-002**
- **Rule:** 702.27a — If buyback cost is NOT paid, spell goes to graveyard normally.
- **Mechanism:** Buyback not paid → normal graveyard destination
- **Minimal Board:** P0 has a sorcery with "Buyback {3}" in hand. P0 casts it without paying buyback.
- **Action:** The spell resolves.
- **Expected Result:** The spell goes to P0's graveyard as normal.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** buyback, no-buyback, DEFERRED

### 702.28 — Shadow

**702.28a** — PURE-DEF. "Shadow is an evasion ability."

**702.28b** — Shadow evasion: shadow can't be blocked by non-shadow; non-shadow can't be blocked by shadow.

**ATOM-702.28b-001**
- **Rule:** 702.28b — A creature with shadow can't be blocked by creatures without shadow, and a creature without shadow can't be blocked by creatures with shadow.
- **Mechanism:** Shadow evasion is bidirectional — shadow and non-shadow can't block each other
- **Minimal Board:** P0 controls a creature with shadow, attacking. P1 controls a creature without shadow and a creature with shadow.
- **Action:** P1 declares blockers.
- **Expected Result:** The creature with shadow can block. The creature without shadow cannot block (shadow can't be blocked by non-shadow). Conversely, if P0 had a non-shadow attacker, P1's shadow creature couldn't block it.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires shadow check in blocker validation.
- **Tags:** shadow, evasion, blocking, DEFERRED

**702.28c** — PURE-DEF. Multiple instances of shadow are redundant.

### 702.29 — Cycling

**702.29a** — Cycling: activated ability from hand — pay cost, discard, draw a card.

**ATOM-702.29a-001**
- **Rule:** 702.29a — "Cycling [cost]" means "[Cost], Discard this card: Draw a card." Functions only from hand.
- **Mechanism:** Cycling activated ability — discard from hand to draw
- **Minimal Board:** P0 has a card with "Cycling {2}" in hand.
- **Action:** P0 activates cycling, paying {2} and discarding the card.
- **Expected Result:** P0 draws a card. The cycled card goes to P0's graveyard.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires zone-specific activated ability framework.
- **Tags:** cycling, activated-ability, discard-draw

**702.29b** — Cycling ability continues to exist in all zones (affects "has an activated ability" checks). PURE-DEF. No independent mechanical consequence beyond ability existence.

> **Note:** Cycling's instant-speed activation is validated by the base activated ability timing framework (activated abilities can be activated any time you have priority unless restricted). No cycling-specific speed test is needed.

**702.29c** — "When you cycle this card" triggers. DEFERRED — Phase 8. One-liner (requires cycling + triggered abilities).

**702.29d** — "Whenever a player cycles or discards a card" triggers only once when cycled. DEFERRED — Phase 8. One-liner.

**702.29e** — Typecycling variant: "[Type]cycling [cost]" → discard, search library for [type] card.

**ATOM-702.29e-001**
- **Rule:** 702.29e — "[Type]cycling [cost]" means "[Cost], Discard this card: Search your library for a [type] card, reveal it, put it into your hand. Then shuffle."
- **Mechanism:** Typecycling searches for a specific type instead of drawing
- **Minimal Board:** P0 has a card with "Mountaincycling {2}" in hand. P0's library contains Mountains.
- **Action:** P0 activates mountaincycling, paying {2} and discarding.
- **Expected Result:** P0 searches library for a Mountain card, reveals it, puts it into hand, then shuffles.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires library search + typecycling variant.
- **Tags:** cycling, typecycling, library-search, DEFERRED

**702.29f** — Typecycling IS cycling for all trigger/cost/effect purposes. PURE-DEF (relationship definition). Mechanical consequence tested via cycling triggers.

### 702.30 — Echo

**702.30a** — Echo: triggered ability — "At the beginning of your upkeep, if this permanent came under your control since your last upkeep, sacrifice it unless you pay [cost]."

**ATOM-702.30a-001**
- **Rule:** 702.30a — "Echo [cost]" means "At the beginning of your upkeep, if this permanent came under your control since the beginning of your last upkeep, sacrifice it unless you pay [cost]."
- **Mechanism:** Echo trigger on first upkeep after gaining control; sacrifice or pay
- **Minimal Board:** P0 cast a creature with "Echo {3}" last turn. It is now P0's upkeep.
- **Action:** Echo triggers. P0 chooses whether to pay {3}.
- **Expected Result:** If P0 pays {3}, the creature stays. If P0 doesn't pay, the creature is sacrificed. On subsequent upkeeps (if P0 has controlled it continuously), echo doesn't trigger again.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires upkeep trigger + "since last upkeep" tracking.
- **Tags:** echo, triggered-ability, sacrifice, DEFERRED

**ATOM-702.30a-002**
- **Rule:** 702.30a — Echo re-triggers after control change: "came under your control since the beginning of your last upkeep" includes control-change effects.
- **Mechanism:** Stealing an echo creature causes echo to trigger on the new controller's next upkeep
- **Minimal Board:** P0 controls a creature with "Echo {3}" that has already survived its echo trigger (P0 paid last upkeep). P1 gains control of the creature (e.g., via a control-change effect).
- **Action:** P1's next upkeep arrives. Echo triggers because the creature came under P1's control since P1's last upkeep.
- **Expected Result:** P1 must pay {3} or sacrifice the creature. The echo cost must be paid again because a new player gained control.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires "since last upkeep" tracking per controller.
- **Tags:** echo, control-change, triggered-ability, DEFERRED

**702.30b** — Errata note: old echo cards now have echo cost = mana cost. PURE-DEF (Oracle errata note). No mechanical consequence for the simulator (uses Oracle text).

### 702.31 — Horsemanship

**702.31a** — PURE-DEF. "Horsemanship is an evasion ability."

**702.31b** — Horsemanship: can't be blocked by creatures without horsemanship; can block creatures with or without horsemanship.

**ATOM-702.31b-001**
- **Rule:** 702.31b — A creature with horsemanship can't be blocked by creatures without horsemanship.
- **Mechanism:** Horsemanship evasion (functions like flying but separate keyword)
- **Minimal Board:** P0 controls a creature with horsemanship, attacking. P1 controls a creature without horsemanship and a creature with horsemanship.
- **Action:** P1 declares blockers.
- **Expected Result:** Only the creature with horsemanship can block. The creature without horsemanship cannot block the horsemanship attacker.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** horsemanship, evasion, blocking, DEFERRED

**702.31c** — PURE-DEF. Multiple instances of horsemanship are redundant.

### 702.32 — Fading

**702.32a** — Fading: enters with N fade counters; each upkeep remove one; if can't, sacrifice.

**ATOM-702.32a-001**
- **Rule:** 702.32a — "Fading N" means "This permanent enters with N fade counters on it" and "At the beginning of your upkeep, remove a fade counter. If you can't, sacrifice the permanent."
- **Mechanism:** Fading ETB counters + upkeep removal + sacrifice when empty
- **Minimal Board:** P0 casts a creature with "Fading 2."
- **Action:** Creature enters with 2 fade counters. P0's upkeep: remove 1 (now 1). Next upkeep: remove 1 (now 0). Next upkeep: can't remove → sacrifice.
- **Expected Result:** The creature survives 2 upkeep cycles then is sacrificed on the 3rd upkeep (when it has 0 counters and can't remove one).
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires ETB counter placement + upkeep trigger.
- **Tags:** fading, counters, sacrifice, DEFERRED

### 702.33 — Kicker

**702.33a** — Kicker: static ability — "You may pay an additional [cost] as you cast this spell."

**ATOM-702.33a-001**
- **Rule:** 702.33a — "Kicker [cost]" means "You may pay an additional [cost] as you cast this spell."
- **Mechanism:** Kicker additional cost payment during casting
- **Minimal Board:** P0 has a spell with "Kicker {2}" in hand (base cost {R}, effect: deal 2 damage; if kicked, deal 4 instead).
- **Action:** P0 casts the spell and pays the kicker cost ({R} + {2}).
- **Expected Result:** The spell is "kicked." On resolution, the kicked effect applies (4 damage instead of 2).
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires additional cost framework (cross-ref T17) + "kicked" status tracking.
- **Tags:** kicker, additional-cost, T17

**702.33b** — "Kicker [cost 1] and/or [cost 2]" = two separate kicker costs. PURE-DEF (shorthand). Mechanical consequence via 702.33d.

**702.33c** — Multikicker: "You may pay an additional [cost] any number of times."

**ATOM-702.33c-001**
- **Rule:** 702.33c — "Multikicker [cost]" means "You may pay an additional [cost] any number of times as you cast this spell."
- **Mechanism:** Multikicker allows repeated additional cost payments
- **Minimal Board:** P0 has a spell with "Multikicker {1}" (effect: create a 1/1 token for each time it was kicked).
- **Action:** P0 casts the spell, paying multikicker 3 times ({3} additional).
- **Expected Result:** The spell was kicked 3 times. On resolution, 3 tokens are created.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires multikicker count tracking.
- **Tags:** kicker, multikicker, DEFERRED

**702.33d** — A spell is "kicked" if any kicker cost was paid; can be kicked multiple times. PURE-DEF (defines "kicked" status). Prerequisite for 702.33e–g.

**702.33e** — Kicked abilities are linked to their kicker ability. PURE-DEF (linked abilities reference, rule 607).

**702.33f** — Multiple kicker costs with specific "if kicked with its [A]/[B] kicker" abilities. DEFERRED — Phase 8. One-liner (requires multi-kicker tracking per cost).

**702.33g** — If a kicked-only portion includes targets, targets chosen only if kicked.

**ATOM-702.33g-001**
- **Rule:** 702.33g — If part of a spell's ability has its effect only if kicked, and that part includes targets, the targets are chosen only if the spell was kicked.
- **Mechanism:** Conditional target selection based on kicked status
- **Minimal Board:** P0 has a spell with "Kicker {R}. Deal 2 damage to target creature. If this spell was kicked, deal 3 damage to target player." P1 controls a creature.
- **Action:** P0 casts without kicker — chooses target creature only. P0 casts with kicker — chooses target creature AND target player.
- **Expected Result:** Without kicker: only creature target is chosen. With kicker: both targets are chosen. If kicked and the player target becomes illegal, the creature portion still resolves.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires conditional target selection.
- **Tags:** kicker, conditional-targets, DEFERRED

**702.33h** — Sticker kicker. OUT-OF-SCOPE (Un-set mechanics / stickers).

### 702.34 — Flashback

**702.34a** — Flashback: cast from graveyard by paying flashback cost; exile instead of going anywhere else when leaving stack.

**ATOM-702.34a-001**
- **Rule:** 702.34a — "Flashback [cost]" means "You may cast this card from your graveyard by paying [cost] rather than its mana cost" and "If the flashback cost was paid, exile this card instead of putting it anywhere else any time it would leave the stack."
- **Mechanism:** Flashback alternative cost from graveyard + exile on resolution/counter
- **Minimal Board:** P0 has an instant with "Flashback {2}{R}" in their graveyard (normally costs {R}, deals 3 damage).
- **Action:** P0 casts the instant from graveyard, paying {2}{R}.
- **Expected Result:** The spell resolves, dealing 3 damage. Then it is exiled (not returned to graveyard). If the spell were countered, it would also be exiled (not go to graveyard).
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires alternative cost framework (cross-ref T17) + CastPermission from graveyard + exile-on-leave-stack replacement.
- **Tags:** flashback, alternative-cost, graveyard-cast, exile, T17

### 702.35 — Madness

**702.35a** — Madness: if discarded, exile instead of graveyard; then may cast for madness cost; if not, put into graveyard.

**ATOM-702.35a-001**
- **Rule:** 702.35a — "Madness [cost]" means "If a player would discard this card, exile it instead" and "When exiled this way, its owner may cast it by paying [cost]. If they don't, put it into their graveyard."
- **Mechanism:** Madness replacement effect on discard + triggered cast from exile
- **Minimal Board:** P0 has a creature with "Madness {B}" in hand (normal cost {2}{B}).
- **Action:** P0 discards the card (e.g., to a discard spell). Madness replacement exiles it. Madness trigger: P0 may cast it for {B}.
- **Expected Result:** If P0 pays {B}, the creature is cast from exile at the reduced cost. If P0 doesn't pay, the card goes to P0's graveyard.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires replacement effect on discard + triggered ability from exile + alternative cost.
- **Tags:** madness, replacement-effect, discard, exile-cast, DEFERRED

**702.35b** — Casting with madness follows alternative cost rules. PURE-DEF (cross-reference to 601.2b/f-h).

**702.35c** — After madness trigger resolves, if card wasn't cast and moved to a public zone, discard-referencing effects can find it. DEFERRED — Phase 8. This requires the replacement effect system (Phase 6) to reroute the discard to exile, the triggered ability system (Phase 7) to fire the "may cast" trigger, and the event tracking system to maintain the "discarded card" reference through the hand → exile → graveyard path so that effects referencing the original discard event can locate the card in its final resting place. Non-trivial dependency chain.

> **Implementation note:** The delta log architecture and invariant ObjectIds may significantly simplify this. Because ObjectId persists across zone changes and the delta log records each `(old_zone, new_zone)` transition, an effect that says "the discarded card" can trace the original discard event's ObjectId through hand → exile → graveyard without needing a separate tracking mechanism. The delta log entry for the original discard captures the ObjectId; subsequent zone changes for that same ObjectId are findable by scanning later deltas. This is a natural fit for the existing architecture.

### 702.36 — Fear

**702.36a** — PURE-DEF. "Fear is an evasion ability."

**702.36b** — Fear: can't be blocked except by artifact creatures and/or black creatures.

**ATOM-702.36b-001**
- **Rule:** 702.36b — A creature with fear can't be blocked except by artifact creatures and/or black creatures.
- **Mechanism:** Fear evasion: blocker must be artifact creature or black
- **Minimal Board:** P0 controls a creature with fear, attacking. P1 controls: a black creature, a green creature, and an artifact creature.
- **Action:** P1 declares blockers.
- **Expected Result:** The black creature and artifact creature can legally block. The green creature cannot.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires color/type check in blocker validation.
- **Tags:** fear, evasion, blocking, DEFERRED

**702.36c** — PURE-DEF. Multiple instances of fear are redundant.

### 702.37 — Morph

**702.37a** — Morph: cast face-down as 2/2 with no text/name/subtypes/mana cost by paying {3}.

**ATOM-702.37a-001**
- **Rule:** 702.37a — "Morph [cost]" means "You may cast this card as a 2/2 face-down creature with no text, no name, no subtypes, and no mana cost by paying {3}."
- **Mechanism:** Morph casting: face-down 2/2 for {3}
- **Minimal Board:** P0 has a creature with "Morph {1}{G}" in hand (normally a 4/4 for {3}{G}{G}).
- **Action:** P0 casts the card face-down, paying {3}.
- **Expected Result:** A 2/2 face-down creature spell is on the stack with no text, no name, no subtypes, no mana cost. It resolves and enters the battlefield as a face-down 2/2 permanent.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires face-down spell/permanent infrastructure (rule 708).
- **Tags:** morph, face-down, casting

**702.37b** — Megamorph variant: morph + put +1/+1 counter when turned face up if megamorph cost was paid. DEFERRED — Phase 8. One-liner.

**702.37c** — Detailed morph casting procedure (copiable values, {3} alternative cost, enters with same characteristics). PURE-DEF / procedural detail. Mechanical consequence covered by ATOM-702.37a-001.

**702.37d** — You can't normally cast a card face down; morph allows it. PURE-DEF. Prerequisite for 702.37a.

**702.37e** — Turn face-down permanent face up as special action: show morph cost, pay it, turn face up.

**ATOM-702.37e-001**
- **Rule:** 702.37e — Any time you have priority, you may turn a face-down permanent you control with a morph ability face up. This is a special action (doesn't use the stack). Pay the morph cost, then turn it face up. ETB abilities don't trigger.
- **Mechanism:** Morph face-up special action: pay cost, reveal, no ETB triggers
- **Minimal Board:** P0 controls a face-down 2/2 (which is actually a 4/4 with "Morph {1}{G}" face down).
- **Action:** P0 pays {1}{G} and turns the permanent face up.
- **Expected Result:** The permanent becomes a 4/4 with its normal characteristics. No "enters the battlefield" triggers fire. This is a special action — it doesn't use the stack and can't be responded to.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires special action framework + face-down permanent tracking.
- **Tags:** morph, face-up, special-action

**702.37f** — If morph cost includes X, other abilities may reference that X value. DEFERRED — Phase 8. One-liner.

**702.37g** — PURE-DEF. Cross-reference to rule 708 (Face-Down Spells and Permanents).

> **Cross-cutting dependency:** Rule 702.37 (Morph) requires the face-down infrastructure defined in rule 708 (face-down spells and permanents: 2/2 colorless with no abilities, turning face up as a special action). Tag: `face-down-infra` applies only to morph (and future megamorph/manifest/disguise). The dependency pass should group 702.37 + 708 together.

--- End of Chunk 5 ---

## Chunk 6 — 702.38–702.55 (Amplify, Provoke, Storm, Affinity, Entwine, Modular, Sunburst, Bushido, Soulshift, Splice, Offering, Ninjutsu, Epic, Convoke, Dredge, Transmute, Bloodthirst, Haunt)

### 702.38 — Amplify

**702.38a** — Amplify: "As this object enters, reveal cards from your hand that share a creature type. This permanent enters with N +1/+1 counters for each card revealed."

**ATOM-702.38a-001**
- **Rule:** 702.38a — "Amplify N" means "As this object enters, reveal any number of cards from your hand that share a creature type with it. This permanent enters with N +1/+1 counters on it for each card revealed."
- **Mechanism:** Amplify ETB counter placement based on revealed hand cards
- **Minimal Board:** P0 casts a Dragon creature with "Amplify 2." P0's hand contains 2 Dragon cards and 1 non-Dragon.
- **Action:** P0 reveals the 2 Dragon cards as the creature enters.
- **Expected Result:** The creature enters with 4 +1/+1 counters (2 revealed × Amplify 2). The non-Dragon could not be revealed.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires ETB replacement + hand reveal + creature type sharing.
- **Tags:** amplify, counters, ETB, DEFERRED

**702.38b** — Multiple instances of amplify work separately. DEFERRED — Phase 8. One-liner.

### 702.39 — Provoke

**702.39a** — Provoke: "Whenever this creature attacks, you may have target creature defending player controls block this creature if able. If you do, untap that creature."

**ATOM-702.39a-001**
- **Rule:** 702.39a — "Provoke" means "Whenever this creature attacks, you may choose to have target creature defending player controls block this creature this combat if able. If you do, untap that creature."
- **Mechanism:** Provoke trigger forces a block and untaps the forced blocker
- **Minimal Board:** P0 controls a creature with provoke, attacking. P1 controls a tapped creature.
- **Action:** Provoke triggers. P0 targets P1's tapped creature. The trigger resolves, untapping it and forcing it to block the provoke creature if able.
- **Expected Result:** P1's creature is untapped and must block the provoke creature (if it's able to block).
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires forced-block constraint + untap on trigger.
- **Tags:** provoke, triggered-ability, forced-block, DEFERRED

**702.39b** — Multiple instances of provoke trigger separately. DEFERRED — Phase 8. One-liner.

### 702.40 — Storm

**702.40a** — Storm: "When you cast this spell, copy it for each other spell cast before it this turn."

**ATOM-702.40a-001**
- **Rule:** 702.40a — "Storm" means "When you cast this spell, copy it for each other spell that was cast before it this turn. If the spell has any targets, you may choose new targets for any of the copies."
- **Mechanism:** Storm trigger creates copies based on storm count
- **Minimal Board:** P0 has cast 3 spells this turn (storm count = 3). P0 casts a spell with storm.
- **Action:** Storm triggers. The spell is copied 3 times.
- **Expected Result:** 3 copies of the spell are put on the stack. Each copy may have new targets chosen. The original spell is also on the stack (4 total instances).
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires spell-cast-this-turn counter + spell copying.
- **Tags:** storm, triggered-ability, copy, spell-count

**702.40b** — Multiple instances of storm trigger separately. DEFERRED — Phase 8. One-liner.

### 702.41 — Affinity

**702.41a** — Affinity: "This spell costs {1} less to cast for each [text] you control."

**ATOM-702.41a-001**
- **Rule:** 702.41a — "Affinity for [text]" means "This spell costs {1} less to cast for each [text] you control."
- **Mechanism:** Affinity cost reduction based on permanent count
- **Minimal Board:** P0 has a spell with "Affinity for artifacts" (base cost {6}). P0 controls 4 artifacts.
- **Action:** P0 casts the spell. Total cost is {6} - {4} = {2}.
- **Expected Result:** The spell costs {2} to cast. The cost reduction applies to generic mana in the total cost.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires cost modification framework (cross-ref T17).
- **Tags:** affinity, cost-reduction, T17

**702.41b** — Multiple instances of affinity each apply. DEFERRED — Phase 8. One-liner.

### 702.42 — Entwine

**702.42a** — Entwine: pay additional cost to choose all modes of a modal spell.

**ATOM-702.42a-001**
- **Rule:** 702.42a — "Entwine [cost]" means "You may choose all modes of this spell instead of just the number specified. If you do, you pay an additional [cost]."
- **Mechanism:** Entwine additional cost enables all modes
- **Minimal Board:** P0 has a modal spell with "Choose one — (A) or (B). Entwine {2}."
- **Action:** P0 casts and pays entwine. Both modes A and B are chosen.
- **Expected Result:** Both modes resolve in order (A then B per 702.42b). Without entwine, only one mode could be chosen.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires modal spell framework + additional cost.
- **Tags:** entwine, modal, additional-cost, DEFERRED

**702.42b** — Entwined modes resolve in printed order. PURE-DEF (procedural detail, covered by ATOM-702.42a-001).

### 702.43 — Modular

**702.43a** — Modular: enters with N +1/+1 counters; on death, move counters to target artifact creature.

**ATOM-702.43a-001**
- **Rule:** 702.43a — "Modular N" means "This permanent enters with N +1/+1 counters on it" and "When this permanent is put into a graveyard from the battlefield, you may put a +1/+1 counter on target artifact creature for each +1/+1 counter on this permanent."
- **Mechanism:** Modular ETB counters + death trigger transferring counters
- **Minimal Board:** P0 controls a creature with "Modular 3" (has 3 +1/+1 counters) and an artifact creature. The modular creature dies.
- **Action:** Death trigger: P0 targets the artifact creature.
- **Expected Result:** The artifact creature gets 3 +1/+1 counters (matching the number on the modular creature when it died, using LKI).
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires ETB counters + death trigger + LKI counter count.
- **Tags:** modular, counters, death-trigger, DEFERRED

**ATOM-702.43a-002**
- **Rule:** 702.43a — "Modular N" means "This permanent enters with N +1/+1 counters on it."
- **Mechanism:** Modular ETB counter placement
- **Minimal Board:** P0 casts a creature with "Modular 3."
- **Action:** The creature enters the battlefield.
- **Expected Result:** The creature enters with exactly 3 +1/+1 counters.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires ETB counter placement.
- **Tags:** modular, counters, ETB, DEFERRED

**702.43b** — Multiple instances of modular work separately. DEFERRED — Phase 8. One-liner.

### 702.44 — Sunburst

**702.44a** — Sunburst: enters with +1/+1 counters (creature) or charge counters (non-creature) equal to number of colors of mana spent.

**ATOM-702.44a-001**
- **Rule:** 702.44a — "Sunburst" means "If entering as a creature, enter with a +1/+1 counter for each color of mana spent to cast it. Otherwise, enter with a charge counter for each color."
- **Mechanism:** Sunburst ETB counters based on colors of mana spent
- **Minimal Board:** P0 casts a creature with sunburst (cost {5}) using {W}{U}{B}{R}{G} (5 colors).
- **Action:** Creature enters the battlefield.
- **Expected Result:** Creature enters with 5 +1/+1 counters. If it were a non-creature, it would enter with 5 charge counters instead.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires mana-color-spent tracking.
- **Tags:** sunburst, counters, mana-colors, DEFERRED

**702.44b** — Sunburst only works from stack as resolving spell; only counts colored mana. PURE-DEF (restriction on 702.44a).

**702.44c** — Sunburst can set a variable for another ability (e.g., "Modular—Sunburst"). DEFERRED — Phase 8. One-liner.

**702.44d** — Multiple instances of sunburst work separately. DEFERRED — Phase 8. One-liner.

### 702.45 — Bushido

**702.45a** — Bushido: "Whenever this creature blocks or becomes blocked, it gets +N/+N until end of turn."

**ATOM-702.45a-001**
- **Rule:** 702.45a — "Bushido N" means "Whenever this creature blocks or becomes blocked, it gets +N/+N until end of turn."
- **Mechanism:** Bushido trigger on blocking/being blocked
- **Minimal Board:** P0 controls a 2/2 with "Bushido 2," attacking. P1 blocks with a creature.
- **Action:** Declare blockers: bushido triggers (creature became blocked).
- **Expected Result:** The 2/2 gets +2/+2 until end of turn, becoming 4/4.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Triggered ability.
- **Tags:** bushido, triggered-ability, DEFERRED

**702.45b** — Multiple instances of bushido trigger separately. DEFERRED — Phase 8. One-liner.

### 702.46 — Soulshift

**702.46a** — Soulshift: "When this permanent is put into a graveyard from the battlefield, you may return target Spirit card with mana value N or less from your graveyard to your hand."

**ATOM-702.46a-001**
- **Rule:** 702.46a — "Soulshift N" means "When this permanent is put into a graveyard from the battlefield, you may return target Spirit card with mana value N or less from your graveyard to your hand."
- **Mechanism:** Soulshift death trigger recovers Spirit card from graveyard
- **Minimal Board:** P0 controls a creature with "Soulshift 4." P0's graveyard contains a Spirit card with mana value 3.
- **Action:** The soulshift creature dies. Death trigger targets the Spirit card.
- **Expected Result:** The Spirit card (MV ≤ 4) is returned to P0's hand.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires death trigger + graveyard targeting by type/MV.
- **Tags:** soulshift, death-trigger, spirit, DEFERRED

**ATOM-702.46a-002**
- **Rule:** 702.46a — Soulshift N cannot return a Spirit card with mana value greater than N.
- **Mechanism:** Soulshift MV cap enforcement (negative case)
- **Minimal Board:** P0 controls a creature with "Soulshift 4." P0's graveyard contains a Spirit card with mana value 5.
- **Action:** The soulshift creature dies. Death trigger looks for valid targets.
- **Expected Result:** The Spirit with MV 5 is NOT a legal target for Soulshift 4 (5 > 4). If no other valid Spirit exists, the trigger has no legal targets.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** soulshift, MV-cap, negative-case, DEFERRED

**702.46b** — Multiple instances of soulshift trigger separately. DEFERRED — Phase 8. One-liner.

### 702.47 — Splice

**702.47a** — Splice onto [quality]: reveal from hand during casting of a [quality] spell to add rules text and pay splice cost.

**ATOM-702.47a-001**
- **Rule:** 702.47a — "Splice onto [quality] [cost]" means "You may reveal this card from your hand as you cast a [quality] spell. If you do, that spell gains the text of this card's rules text and you pay [cost] as an additional cost."
- **Mechanism:** Splice adds rules text from hand to a spell being cast
- **Minimal Board:** P0 has an Arcane spell in hand and a card with "Splice onto Arcane {1}{R}" (rules text: "deal 2 damage to any target") in hand.
- **Action:** P0 casts the Arcane spell, revealing the splice card and paying {1}{R} additional. The splice card stays in hand.
- **Expected Result:** The Arcane spell gains "deal 2 damage to any target" as additional text. The splice card remains in P0's hand (not cast, not discarded).
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires text-addition during casting + splice cost as additional cost.
- **Tags:** splice, text-changing, additional-cost, DEFERRED

**702.47b** — Can't splice same card twice onto one spell; effects happen after main spell. DEFERRED — Phase 8. One-liner.

**702.47c** — Spliced spell gains only rules text, not other characteristics (name, color, etc.). DEFERRED — Phase 8. One-liner (text-changing effect per rule 612).

> **Splice deferral justification:** Splice is blocked by three independent infrastructure requirements: (1) T17 additional cost framework, (2) text-changing effects (rule 612 — distinct from ability-granting; no other keyword in 702.1–80 modifies a spell's rules text), (3) dynamic target addition (targets added during casting based on spliced text). All three are Phase 8 work. No risk of splice falling through the cracks.

**702.47d** — Targets for spliced text chosen normally. PURE-DEF (target selection procedure).

**702.47e** — Splice changes are lost when spell leaves the stack. PURE-DEF (transient modification).

### 702.48 — Offering

**702.48a** — Offering: sacrifice a [quality] permanent to reduce cost by its mana cost and cast at instant speed.

**ATOM-702.48a-001**
- **Rule:** 702.48a — "[Quality] offering" means "As an additional cost, you may sacrifice a [quality] permanent. If you do, this spell's total cost is reduced by that permanent's mana cost, and you may cast at instant speed."
- **Mechanism:** Offering: sacrifice for cost reduction + instant-speed permission
- **Minimal Board:** P0 has a creature with "Goblin offering" (cost {5}{R}) in hand. P0 controls a Goblin (mana cost {1}{R}).
- **Action:** P0 sacrifices the Goblin as an additional cost. The spell's cost is reduced by {1}{R}, becoming {4}.
- **Expected Result:** P0 pays {4} total. The spell may be cast at instant speed (because offering was chosen). The Goblin is sacrificed.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires sacrifice-as-cost + mana cost reduction + instant-speed override.
- **Tags:** offering, sacrifice, cost-reduction, DEFERRED

**702.48b** — Permanent chosen during 601.2b, sacrificed during 601.2h. PURE-DEF (procedural).

**702.48c** — Cost reduction: colored mana reduces same color, excess reduces generic. PURE-DEF (cross-ref rule 118.7, covered by cost modification framework).

### 702.49 — Ninjutsu

**702.49a** — Ninjutsu: activated ability from hand — pay cost, return unblocked attacker, put this card onto battlefield tapped and attacking.

**ATOM-702.49a-001**
- **Rule:** 702.49a — "Ninjutsu [cost]" means "[Cost], Reveal this card, Return an unblocked attacking creature you control to its owner's hand: Put this card onto the battlefield tapped and attacking."
- **Mechanism:** Ninjutsu: swap unblocked attacker for creature from hand
- **Minimal Board:** P0 has a creature with "Ninjutsu {U}{B}" in hand. P0 controls an unblocked attacking creature.
- **Action:** P0 activates ninjutsu, paying {U}{B}, revealing the ninjutsu card, and returning the unblocked attacker to hand.
- **Expected Result:** The ninjutsu creature enters the battlefield tapped and attacking the same target as the returned creature. The returned creature goes to P0's hand.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires hand-activated ability + ETB attacking + swap mechanic.
- **Tags:** ninjutsu, activated-ability, swap, ETB-attacking, DEFERRED

**ATOM-702.49a-002**
- **Rule:** 702.49a — Ninjutsu can be activated any time after blockers are declared (any priority window in declare blockers step, first-strike damage step, combat damage step, or end-of-combat step).
- **Mechanism:** Ninjutsu after first-strike damage for pseudo-double-strike
- **Minimal Board:** P0 has a creature with "Ninjutsu {U}{B}" and first strike in hand. P0 controls an unblocked attacking creature (no first strike). First-strike damage step occurs (P0's attacker doesn't participate). Regular damage step hasn't happened yet.
- **Action:** After first-strike damage, P0 activates ninjutsu: returns the unblocked attacker to hand, puts the ninjutsu creature (with first strike) onto the battlefield tapped and attacking.
- **Expected Result:** The ninjutsu creature enters tapped and attacking. It deals combat damage in the regular damage step. Combined with the original attacker's absence from the first-strike step, this creates a pseudo-double-strike effect (original creature could have dealt first-strike damage, then ninjutsu creature deals regular damage).
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** ninjutsu, first-strike, timing, DEFERRED

**ATOM-702.49a-003**
- **Rule:** 702.49a — Ninjutsu activated in end-of-combat step: creature enters attacking but deals no combat damage (damage steps already passed).
- **Mechanism:** Ninjutsu in end-of-combat for no-damage entry
- **Minimal Board:** P0 has a creature with "Ninjutsu {U}{B}" in hand. P0 controls an unblocked attacking creature. Combat damage has been dealt. It is the end-of-combat step.
- **Action:** P0 activates ninjutsu during end-of-combat step, returning the attacker and putting the ninjutsu creature onto the battlefield tapped and attacking.
- **Expected Result:** The ninjutsu creature enters tapped and attacking, but deals no combat damage this turn (the combat damage step has already passed). It will be removed from combat at end of the end-of-combat step.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** ninjutsu, end-of-combat, timing, DEFERRED

**702.49b** — Ninjutsu card remains revealed until ability leaves stack. PURE-DEF (procedural).

**702.49c** — Ninjutsu creature enters attacking the same target as the returned creature.

Covered by ATOM-702.49a-001 (expected result specifies same attack target).

**702.49d** — Commander ninjutsu: also works from command zone. DEFERRED — Phase 9 (Commander). One-liner.

### 702.50 — Epic

**702.50a** — Epic: "For the rest of the game, you can't cast spells" + "At the beginning of each of your upkeeps, copy this spell."

**ATOM-702.50a-001**
- **Rule:** 702.50a — "Epic" means "For the rest of the game, you can't cast spells" and "At the beginning of each of your upkeeps for the rest of the game, copy this spell except for its epic ability."
- **Mechanism:** Epic locks out casting + creates recurring copies
- **Minimal Board:** P0 casts a spell with epic.
- **Action:** The spell resolves. P0 can no longer cast spells for the rest of the game. On each subsequent upkeep, a copy of the spell is created.
- **Expected Result:** P0's subsequent spell cast attempts are illegal. Each upkeep, a copy (without epic) is put on the stack.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires permanent casting restriction + delayed trigger + spell copying.
- **Tags:** epic, casting-restriction, delayed-trigger, DEFERRED

**702.50b** — Epic only prevents casting; effects can still put spell copies on the stack. PURE-DEF (clarification of 702.50a).

### 702.51 — Convoke

**702.51a** — Convoke: tap creatures to pay for mana in the spell's cost.

**ATOM-702.51a-001**
- **Rule:** 702.51a — "Convoke" means "For each colored mana in this spell's total cost, you may tap an untapped creature of that color you control rather than pay that mana. For each generic mana, you may tap an untapped creature you control rather than pay that mana."
- **Mechanism:** Convoke: tap creatures as mana payment
- **Minimal Board:** P0 has a spell with convoke (cost {3}{G}{G}). P0 controls 2 green creatures (untapped) and 1 red creature (untapped). P0 has {2} in mana pool.
- **Action:** P0 taps both green creatures (paying {G}{G}) and the red creature (paying {1} generic). P0 pays {2} from pool for the remaining {2} generic. Total: {2} mana + 3 tapped creatures.
- **Expected Result:** The spell is successfully cast. 3 creatures are tapped. {1} mana spent from pool.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires cost modification framework (cross-ref T17) + creature-tap-as-mana.
- **Tags:** convoke, cost-modification, tap-creatures, T17

**702.51b** — Convoke isn't additional/alternative cost; applies after total cost determined. PURE-DEF (procedural, important for cost modification ordering with T17).

**702.51c** — PURE-DEF. A creature tapped for convoke is said to have "convoked" that spell.

**702.51d** — PURE-DEF. Multiple instances of convoke are redundant.

### 702.52 — Dredge

**702.52a** — Dredge: replace a draw with self-mill N to return this card from graveyard to hand.

**ATOM-702.52a-001**
- **Rule:** 702.52a — "Dredge N" means "As long as you have at least N cards in your library, if you would draw a card, you may instead mill N cards and return this card from your graveyard to your hand."
- **Mechanism:** Dredge replacement effect: replace draw with mill + graveyard-to-hand
- **Minimal Board:** P0 has a card with "Dredge 3" in graveyard. P0 has 5 cards in library. P0 would draw a card.
- **Action:** P0 chooses to dredge instead of drawing. P0 mills 3 cards and returns the dredge card to hand.
- **Expected Result:** 3 cards moved from library to graveyard. Dredge card moved from graveyard to hand. No card was drawn.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires draw replacement + self-mill + graveyard-to-hand.
- **Tags:** dredge, replacement-effect, mill, graveyard

**702.52b** — Can't dredge if library has fewer than N cards.

**ATOM-702.52b-001**
- **Rule:** 702.52b — Dredge N is not available if the player's library has fewer than N cards.
- **Mechanism:** Dredge availability check: library size ≥ N required
- **Minimal Board:** P0 has a card with "Dredge 3" in graveyard. P0 has only 2 cards in library. P0 would draw a card.
- **Action:** P0 attempts to choose dredge instead of drawing.
- **Expected Result:** Dredge option is not available — library has fewer than 3 cards. P0 must draw normally.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** dredge, library-size, boundary, negative-case, DEFERRED

### 702.53 — Transmute

**702.53a** — Transmute: activated ability from hand — pay cost, discard, search library for card with same mana value.

**ATOM-702.53a-001**
- **Rule:** 702.53a — "Transmute [cost]" means "[Cost], Discard this card: Search your library for a card with the same mana value as the discarded card, reveal that card, put it into your hand. Then shuffle. Sorcery speed."
- **Mechanism:** Transmute: hand-based tutor by mana value
- **Minimal Board:** P0 has a card with "Transmute {1}{U}{B}" (mana value 3) in hand.
- **Action:** P0 activates transmute, paying {1}{U}{B} and discarding the card. P0 searches for a card with MV 3.
- **Expected Result:** P0 finds a card with mana value 3 from library, reveals it, puts it into hand, and shuffles.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires hand-activated ability + library search by MV.
- **Tags:** transmute, activated-ability, tutor, DEFERRED

**702.53b** — Transmute ability exists in all zones (affects "has activated ability" checks). PURE-DEF.

> **Cross-reference:** Transmute (702.53) and Cycling (702.29) both follow the "discard from hand + activated ability" pattern. Implementation likely shares the hand-zone activation framework. Tag: `cross-ref-cycling`.

### 702.54 — Bloodthirst

**702.54a** — Bloodthirst: enters with +1/+1 counters if an opponent was dealt damage this turn.

**ATOM-702.54a-001**
- **Rule:** 702.54a — "Bloodthirst N" means "If an opponent was dealt damage this turn, this permanent enters with N +1/+1 counters on it."
- **Mechanism:** Bloodthirst conditional ETB counters
- **Minimal Board:** P0 dealt damage to P1 this turn (e.g., via combat or a spell). P0 casts a creature with "Bloodthirst 2."
- **Action:** Creature enters the battlefield.
- **Expected Result:** The creature enters with 2 +1/+1 counters (opponent was dealt damage this turn).
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires "opponent damaged this turn" tracking.
- **Tags:** bloodthirst, counters, ETB, DEFERRED

**702.54b** — "Bloodthirst X" = counters equal to total damage dealt to opponents this turn. DEFERRED — Phase 8. One-liner.

**702.54c** — Multiple instances of bloodthirst apply separately. DEFERRED — Phase 8. One-liner.

### 702.55 — Haunt

**702.55a** — Haunt: on death (permanent) or on resolution going to graveyard (spell), exile haunting target creature.

**ATOM-702.55a-001**
- **Rule:** 702.55a — "Haunt" on a permanent means "When this permanent is put into a graveyard from the battlefield, exile it haunting target creature." On a spell: "When this spell is put into a graveyard during its resolution, exile it haunting target creature."
- **Mechanism:** Haunt trigger on death/resolution exiles card haunting a creature
- **Minimal Board:** P0 controls a creature with haunt. P1 controls a creature. P0's haunt creature dies.
- **Action:** Haunt trigger targets P1's creature. The dead creature is exiled "haunting" the target.
- **Expected Result:** The haunt card is in exile, associated with ("haunting") P1's creature. When the haunted creature dies, the haunt card's ability triggers again.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires exile-with-association + haunting tracking.
- **Tags:** haunt, exile, haunting, DEFERRED

**702.55b** — PURE-DEF. Defines "haunt" and "creature it haunts" terminology.

> **Note:** "Haunting" / "is haunted by" is a **designation** (game-state relationship), not a **characteristic** (rule 109.3). The engine should track this as an association between the exiled card and the haunted permanent (e.g., `haunted_by: Option<ObjectId>`), not as a characteristic on the object.

**702.55c** — Haunting cards' triggered abilities can trigger from exile. DEFERRED — Phase 8. One-liner (requires exile-zone triggers).

--- End of Chunk 6 ---

## Chunk 7 — 702.56–702.72 (Replicate, Forecast, Graft, Recover, Ripple, Split Second, Suspend, Vanishing, Absorb, Aura Swap, Delve, Fortify, Frenzy, Gravestorm, Poisonous, Transfigure, Champion)

### 702.56 — Replicate

**702.56a** — Replicate: pay additional cost any number of times; on cast, copy the spell for each time replicate was paid.

**ATOM-702.56a-001**
- **Rule:** 702.56a — "Replicate [cost]" means "As an additional cost to cast this spell, you may pay [cost] any number of times" and "When you cast this spell, if a replicate cost was paid, copy it for each time its replicate cost was paid."
- **Mechanism:** Replicate additional cost + triggered copy creation
- **Minimal Board:** P0 has a spell with "Replicate {R}" (deals 2 damage to target creature). P0 pays replicate twice.
- **Action:** Spell is cast with replicate paid 2 times. Trigger creates 2 copies.
- **Expected Result:** 3 total instances on the stack (original + 2 copies). Each copy may have new targets.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires additional cost counting + spell copying.
- **Tags:** replicate, additional-cost, copy, DEFERRED

**702.56b** — Multiple instances of replicate are paid and trigger separately. DEFERRED — Phase 8. One-liner.

### 702.57 — Forecast

**702.57a** — Forecast: special activated ability from hand, activated only during owner's upkeep, once per turn.

**ATOM-702.57a-001**
- **Rule:** 702.57a/b — A forecast ability is activated only from a player's hand, only during the upkeep step of the card's owner, and only once each turn. The card is revealed until it leaves hand or a non-upkeep step begins.
- **Mechanism:** Forecast: hand-based ability with upkeep + once-per-turn restriction
- **Minimal Board:** P0 has a card with "Forecast — {1}{W}, Reveal this card from your hand: You gain 2 life." in hand. It is P0's upkeep.
- **Action:** P0 activates forecast, paying {1}{W} and revealing the card.
- **Expected Result:** P0 gains 2 life. The card remains in hand (revealed until end of upkeep). P0 cannot activate it again this turn.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires hand-based ability + upkeep-only + once-per-turn tracking.
- **Tags:** forecast, activated-ability, upkeep, DEFERRED

### 702.58 — Graft

**702.58a** — Graft: enters with N +1/+1 counters; whenever another creature enters, may move a +1/+1 counter to it.

**ATOM-702.58a-001**
- **Rule:** 702.58a — "Graft N" means "This permanent enters with N +1/+1 counters on it" and "Whenever another creature enters, if this permanent has a +1/+1 counter on it, you may move a +1/+1 counter from this permanent onto that creature."
- **Mechanism:** Graft ETB counters + trigger to share counters
- **Minimal Board:** P0 controls a creature with "Graft 3" (3 +1/+1 counters). P0 casts another creature.
- **Action:** The new creature enters. Graft triggers. P0 chooses to move a counter.
- **Expected Result:** Graft creature goes from 3 to 2 counters. New creature gets 1 +1/+1 counter. If the graft creature had 0 counters, the trigger wouldn't happen (condition: "if this permanent has a +1/+1 counter").
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires ETB counters + ETB trigger + counter movement.
- **Tags:** graft, counters, triggered-ability, DEFERRED

**702.58b** — Multiple instances of graft work separately. DEFERRED — Phase 8. One-liner.

### 702.59 — Recover

**702.59a** — Recover: "When a creature is put into your graveyard from the battlefield, you may pay [cost]. If you do, return this card from your graveyard to your hand. Otherwise, exile this card."

**ATOM-702.59a-001**
- **Rule:** 702.59a — "Recover [cost]" means "When a creature is put into your graveyard from the battlefield, you may pay [cost]. If you do, return this card to your hand. Otherwise, exile it."
- **Mechanism:** Recover trigger from graveyard: pay to recover or exile
- **Minimal Board:** P0 has a card with "Recover {2}{B}" in graveyard. P0's creature dies (goes to graveyard).
- **Action:** Recover triggers. P0 may pay {2}{B}.
- **Expected Result:** If P0 pays: card returns to hand. If P0 doesn't pay: card is exiled. Either way, the trigger resolves.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires graveyard trigger + pay-or-exile choice.
- **Tags:** recover, graveyard-trigger, DEFERRED

### 702.60 — Ripple

**702.60a** — Ripple: "When you cast this spell, you may reveal the top N cards of your library. Cast any with the same name for free, put the rest on the bottom."

**ATOM-702.60a-001**
- **Rule:** 702.60a — "Ripple N" means "When you cast this spell, you may reveal the top N cards. You may cast any with the same name without paying their mana costs. Put the rest on the bottom in any order."
- **Mechanism:** Ripple trigger: reveal, free-cast same-name cards, bottom the rest
- **Minimal Board:** P0 casts a spell with "Ripple 4" named "Surging Flame." P0's library has a copy of "Surging Flame" in the top 4 cards.
- **Action:** Ripple triggers. P0 reveals top 4. One is "Surging Flame."
- **Expected Result:** P0 may cast the revealed "Surging Flame" without paying its mana cost (which triggers its own ripple). The other 3 cards go to the bottom of P0's library.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires library reveal + name matching + free cast.
- **Tags:** ripple, triggered-ability, free-cast, DEFERRED

**702.60b** — Multiple instances of ripple trigger separately. DEFERRED — Phase 8. One-liner.

### 702.61 — Split Second

**702.61a** — Split second: while this spell is on the stack, players can't cast spells or activate non-mana abilities.

**ATOM-702.61a-001**
- **Rule:** 702.61a — "Split second" means "As long as this spell is on the stack, players can't cast other spells or activate abilities that aren't mana abilities."
- **Mechanism:** Split second restricts actions while on stack
- **Minimal Board:** P0 casts a spell with split second. P1 has a counterspell in hand.
- **Action:** P1 attempts to cast the counterspell while the split second spell is on the stack.
- **Expected Result:** P1 cannot cast the counterspell — split second prevents casting other spells. P1 can still activate mana abilities and take special actions (702.61b).
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires stack-presence check gating priority actions.
- **Tags:** split-second, stack, casting-restriction

**702.61b** — Mana abilities, special actions, and triggered abilities still work with split second on stack.

**ATOM-702.61b-001**
- **Rule:** 702.61b — Players may activate mana abilities and take special actions while a spell with split second is on the stack. Triggered abilities trigger and are put on the stack as normal.
- **Mechanism:** Split second exceptions: mana abilities, special actions, triggers still function
- **Minimal Board:** P0 casts a spell with split second. P1 controls a land (mana ability) and a permanent with a triggered ability that triggers in response to the split second spell being cast.
- **Action:** P1 taps land for mana. The triggered ability goes on the stack.
- **Expected Result:** Mana ability activation succeeds. Triggered ability is placed on the stack normally. Only casting spells and activating non-mana abilities are prevented.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** split-second, mana-abilities, triggers, DEFERRED

**702.61c** — PURE-DEF. Multiple instances of split second are redundant.

### 702.62 — Suspend

**702.62a** — Suspend: exile with time counters from hand; remove counters each upkeep; cast for free when last counter removed.

**ATOM-702.62a-001**
- **Rule:** 702.62a — Suspend exile: pay cost from hand, exile with N time counters. This is a special action (doesn't use the stack).
- **Mechanism:** Suspend initial exile from hand
- **Minimal Board:** P0 has a creature with "Suspend 3—{R}" in hand.
- **Action:** P0 pays {R} and exiles the card with 3 time counters.
- **Expected Result:** The card is in exile with 3 time counters. This is a special action — it doesn't go on the stack, can't be responded to. The card is now "suspended" (in exile + has suspend + has time counters).
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires exile-with-counters + special action from hand.
- **Tags:** suspend, exile, special-action

**ATOM-702.62a-002**
- **Rule:** 702.62a — Suspend upkeep trigger: each upkeep removes one time counter from the suspended card.
- **Mechanism:** Suspend upkeep counter removal
- **Minimal Board:** P0 has a suspended card in exile with 3 time counters. It is P0's upkeep.
- **Action:** Upkeep trigger removes a time counter.
- **Expected Result:** The card now has 2 time counters. Next upkeep: 1 counter. The removal is a triggered ability that uses the stack.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires upkeep trigger + counter removal.
- **Tags:** suspend, upkeep-trigger, time-counters

**ATOM-702.62a-003**
- **Rule:** 702.62a — Suspend last-counter trigger: when the last time counter is removed, cast the spell without paying its mana cost. Creature spells gain haste.
- **Mechanism:** Suspend free cast on last counter removal
- **Minimal Board:** P0 has a suspended creature card in exile with 1 time counter. It is P0's upkeep.
- **Action:** Upkeep trigger removes the last time counter. The "last counter removed" trigger fires, casting the spell for free.
- **Expected Result:** The creature spell is cast without paying its mana cost. Because it's a creature spell cast via suspend, it gains haste (until P0 loses control of it). If the spell can't be cast (e.g., due to a prohibition effect), it remains in exile.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires free cast + haste grant + casting prohibition check.
- **Tags:** suspend, free-cast, haste, last-counter

**702.62b** — PURE-DEF. Defines "suspended" = in exile + has suspend + has time counter.

**702.62c** — Suspend casting check considers effects that prohibit casting. DEFERRED — Phase 8. One-liner.

**702.62d** — Casting via suspend follows alternative cost rules. PURE-DEF (cross-ref 601.2b/f-h).

### 702.63 — Vanishing

**702.63a** — Vanishing: enters with N time counters; each upkeep remove one; sacrifice when last is removed.

**ATOM-702.63a-001**
- **Rule:** 702.63a — "Vanishing N" means "This permanent enters with N time counters," "At the beginning of your upkeep, if it has a time counter, remove one," and "When the last time counter is removed, sacrifice it."
- **Mechanism:** Vanishing ETB counters + upkeep removal + sacrifice trigger
- **Minimal Board:** P0 casts a creature with "Vanishing 3."
- **Action:** Enters with 3 time counters. Each upkeep removes one. When last is removed → sacrifice trigger.
- **Expected Result:** The creature survives 3 upkeep cycles. On the 3rd upkeep, the last counter is removed and the sacrifice trigger fires.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires ETB counters + upkeep trigger + last-counter trigger.
- **Tags:** vanishing, time-counters, sacrifice, DEFERRED

**702.63b** — Vanishing without a number: no ETB counters, but still has upkeep removal and last-counter sacrifice triggers.

**ATOM-702.63b-001**
- **Rule:** 702.63b — A permanent with vanishing that enters with 0 time counters (e.g., due to Solemnity preventing counter placement) does not trigger the sacrifice ability.
- **Mechanism:** Vanishing with 0 counters: no counter to be "last removed" → no sacrifice
- **Minimal Board:** P0 controls Solemnity ("Counters can't be placed on permanents"). P0 casts a creature with "Vanishing 3."
- **Action:** The creature enters. Solemnity prevents the 3 time counters from being placed. Upkeep arrives.
- **Expected Result:** The creature enters with 0 time counters. The upkeep trigger "remove a time counter" does nothing (no counter to remove). The sacrifice trigger "when the last time counter is removed" never fires because no counter was removed. The permanent persists indefinitely.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires counter-prevention interaction.
- **Tags:** vanishing, solemnity, zero-counters, DEFERRED

**ATOM-702.63b-002**
- **Rule:** 702.63b — If a time counter is externally added to a vanishing permanent that has 0 counters, the vanishing loop resumes.
- **Mechanism:** External counter restarts vanishing countdown
- **Minimal Board:** P0 controls a creature with vanishing that currently has 0 time counters (persisting due to Solemnity or similar). An effect places 1 time counter on it (e.g., Clockspinning).
- **Action:** P0's next upkeep: the upkeep trigger removes the time counter (now 0). The sacrifice trigger fires (last counter removed).
- **Expected Result:** The creature is sacrificed. Adding a time counter restarted the vanishing mechanic — when that counter was removed, it became the "last" counter removed, triggering the sacrifice.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** vanishing, external-counter, sacrifice, DEFERRED

**702.63c** — Multiple instances of vanishing work separately. DEFERRED — Phase 8. One-liner.

### 702.64 — Absorb

**702.64a** — Absorb: "If a source would deal damage to this creature, prevent N of that damage."

**ATOM-702.64a-001**
- **Rule:** 702.64a — "Absorb N" means "If a source would deal damage to this creature, prevent N of that damage."
- **Mechanism:** Absorb prevents N damage from each source
- **Minimal Board:** P0 controls a 2/4 creature with "Absorb 2." P1 deals 5 damage to it from one source.
- **Action:** Damage is dealt. Absorb prevents 2.
- **Expected Result:** The creature takes 3 damage (5 - 2 prevented). It has 3 damage marked on a 4-toughness creature, so it survives.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires damage prevention per-source.
- **Tags:** absorb, damage-prevention, DEFERRED

**ATOM-702.64a-002**
- **Rule:** 702.64a/b — Absorb N applies separately to each damage source.
- **Mechanism:** Absorb per-source prevention with multiple simultaneous sources
- **Minimal Board:** P0 controls a 2/6 creature with "Absorb 2." Two different sources each deal 3 damage to it simultaneously (e.g., two creatures in combat).
- **Action:** Damage from source A (3) and source B (3) are dealt.
- **Expected Result:** Absorb prevents 2 from each source: source A deals 1 (3-2), source B deals 1 (3-2). Total damage: 2. The creature survives (2 damage on 6 toughness). Without per-source application, absorb would only prevent 2 total, leaving 4 damage.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** absorb, per-source, multiple-sources, DEFERRED

**702.64b** — Absorb applies separately to each source at each time. PURE-DEF (clarification of per-source application).

**702.64c** — Multiple instances of absorb apply separately. DEFERRED — Phase 8. One-liner.

### 702.65 — Aura Swap

**702.65a** — Aura swap: activated ability to exchange an Aura on the battlefield with an Aura card in hand.

**ATOM-702.65a-001**
- **Rule:** 702.65a — "Aura swap [cost]" means "[Cost]: You may exchange this permanent with an Aura card in your hand."
- **Mechanism:** Aura swap: exchange attached Aura with one from hand
- **Minimal Board:** P0 controls an Aura with "Aura swap {2}{U}" attached to a creature. P0 has another Aura card in hand that can enchant the same creature.
- **Action:** P0 activates aura swap, paying {2}{U}.
- **Expected Result:** The battlefield Aura goes to P0's hand. The hand Aura is put onto the battlefield attached to the same creature (no targeting, no casting).
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires exchange mechanic + Aura attachment validation.
- **Tags:** aura-swap, exchange, DEFERRED

**702.65b** — If either half of the exchange can't complete, no exchange occurs. DEFERRED — Phase 8. One-liner.

### 702.66 — Delve

**702.66a** — Delve: exile cards from graveyard to pay generic mana.

**ATOM-702.66a-001**
- **Rule:** 702.66a — "Delve" means "For each generic mana in this spell's total cost, you may exile a card from your graveyard rather than pay that mana."
- **Mechanism:** Delve: exile graveyard cards to reduce generic cost
- **Minimal Board:** P0 has a spell with delve (cost {6}{U}{U}). P0's graveyard has 5 cards. P0 has {1}{U}{U} in mana pool.
- **Action:** P0 exiles 5 graveyard cards (paying {5} of the {6} generic). P0 pays {1}{U}{U} from pool.
- **Expected Result:** The spell is cast for {1}{U}{U} mana + 5 exiled graveyard cards. Total cost of {6}{U}{U} is fully paid.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires cost modification framework (cross-ref T17) + graveyard exile as payment.
- **Tags:** delve, cost-modification, graveyard-exile, T17

**702.66b** — Delve isn't additional/alternative cost; applies after total cost determined. PURE-DEF (same structure as convoke 702.51b).

**702.66c** — PURE-DEF. Multiple instances of delve are redundant.

### 702.67 — Fortify

**702.67a** — Fortify: activated ability of Fortifications. "Attach this Fortification to target land you control. Sorcery speed."

**ATOM-702.67a-001**
- **Rule:** 702.67a — "Fortify [cost]" means "[Cost]: Attach this Fortification to target land you control. Activate only as a sorcery."
- **Mechanism:** Fortify attaches Fortification to a land
- **Minimal Board:** P0 controls a Fortification with "Fortify {3}" and a land.
- **Action:** P0 activates fortify, paying {3}, targeting their land.
- **Expected Result:** The Fortification becomes attached to the land.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires Fortification subtype + attachment to lands.
- **Tags:** fortify, fortification, attachment, DEFERRED

**702.67b** — PURE-DEF. Cross-reference to rule 301 (Artifacts).

**702.67c** — Multiple fortify abilities: any may be activated. PURE-DEF (same as equip 702.6d).

### 702.68 — Frenzy

**702.68a** — Frenzy: "Whenever this creature attacks and isn't blocked, it gets +N/+0 until end of turn."

**ATOM-702.68a-001**
- **Rule:** 702.68a — "Frenzy N" means "Whenever this creature attacks and isn't blocked, it gets +N/+0 until end of turn."
- **Mechanism:** Frenzy trigger on unblocked attack
- **Minimal Board:** P0 controls a 2/2 with "Frenzy 3," attacking. P1 doesn't block.
- **Action:** After blockers declared (none), frenzy triggers.
- **Expected Result:** The creature gets +3/+0 until end of turn, becoming 5/2.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Triggered ability.
- **Tags:** frenzy, triggered-ability, DEFERRED

**702.68b** — Multiple instances of frenzy trigger separately. DEFERRED — Phase 8. One-liner.

### 702.69 — Gravestorm

**702.69a** — Gravestorm: "When you cast this spell, copy it for each permanent put into a graveyard from the battlefield this turn."

**ATOM-702.69a-001**
- **Rule:** 702.69a — "Gravestorm" means "When you cast this spell, copy it for each permanent that was put into a graveyard from the battlefield this turn."
- **Mechanism:** Gravestorm trigger: copies based on permanents-died-this-turn count
- **Minimal Board:** 3 permanents were put into graveyards from the battlefield this turn. P0 casts a spell with gravestorm.
- **Action:** Gravestorm triggers. 3 copies created.
- **Expected Result:** 3 copies on the stack plus the original (4 total).
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires permanents-died-this-turn counter + spell copying.
- **Tags:** gravestorm, triggered-ability, copy, DEFERRED

**702.69b** — Multiple instances of gravestorm trigger separately. DEFERRED — Phase 8. One-liner.

> **Cross-reference:** Gravestorm (702.69) and Storm (702.40) share the "copy for each [count]" triggered pattern. Implementation should use the same spell-copying infrastructure. Tag: `cross-ref-storm`.

### 702.70 — Poisonous

**702.70a** — Poisonous: "Whenever this creature deals combat damage to a player, that player gets N poison counters."

**ATOM-702.70a-001**
- **Rule:** 702.70a — "Poisonous N" means "Whenever this creature deals combat damage to a player, that player gets N poison counters."
- **Mechanism:** Poisonous trigger gives poison counters on combat damage to player
- **Minimal Board:** P0 controls a 1/1 with "Poisonous 2," attacking. P1 doesn't block.
- **Action:** Combat damage: 1 damage to P1. Poisonous triggers.
- **Expected Result:** P1 gets 2 poison counters (from poisonous, not from the damage itself). The 1 damage is dealt normally. If P1 reaches 10 poison counters, they lose (rule 104.3d).
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires poison counter tracking + combat damage trigger.
- **Tags:** poisonous, poison-counters, triggered-ability, DEFERRED

**702.70b** — Multiple instances of poisonous trigger separately. DEFERRED — Phase 8. One-liner.

### 702.71 — Transfigure

**702.71a** — Transfigure: activated ability — sacrifice, search library for creature with same mana value, put onto battlefield.

**ATOM-702.71a-001**
- **Rule:** 702.71a — "Transfigure [cost]" means "[Cost], Sacrifice this permanent: Search your library for a creature card with the same mana value and put it onto the battlefield. Then shuffle. Sorcery speed."
- **Mechanism:** Transfigure: sacrifice-tutor for same MV creature
- **Minimal Board:** P0 controls a creature with "Transfigure {1}{B}{B}" (mana value 3).
- **Action:** P0 activates transfigure, paying {1}{B}{B} and sacrificing the creature. P0 searches for a creature with MV 3.
- **Expected Result:** The sacrificed creature is gone. A creature with mana value 3 from P0's library enters the battlefield. Library shuffled.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires sacrifice-as-cost + library search by MV + ETB.
- **Tags:** transfigure, sacrifice, tutor, DEFERRED

### 702.72 — Champion

**702.72a** — Champion: "When this permanent enters, sacrifice it unless you exile another [object] you control" + "When this permanent leaves the battlefield, return the exiled card."

**ATOM-702.72a-001**
- **Rule:** 702.72a — "Champion an [object]" means "When this permanent enters, sacrifice it unless you exile another [object] you control" and "When this permanent leaves the battlefield, return the exiled card to the battlefield under its owner's control."
- **Mechanism:** Champion ETB exile + LTB return
- **Minimal Board:** P0 casts a creature with "Champion a creature." P0 controls another creature.
- **Action:** Champion ETB trigger: P0 exiles the other creature. Later, the champion creature dies.
- **Expected Result:** On ETB: the other creature is exiled. On LTB: the exiled creature returns to the battlefield under its owner's control. If P0 had no valid creature to exile, the champion creature is sacrificed.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires ETB trigger + exile association + LTB trigger + linked abilities (rule 607).
- **Tags:** champion, ETB, LTB, exile, linked-abilities, DEFERRED

**ATOM-702.72a-002**
- **Rule:** 702.72a — Champion with no valid target: if no other [object] exists to exile, the champion creature is sacrificed.
- **Mechanism:** Champion ETB sacrifice when no valid exile target exists
- **Minimal Board:** P0 casts a creature with "Champion a creature." P0 controls no other creatures.
- **Action:** Champion ETB trigger fires. P0 has no valid creature to exile.
- **Expected Result:** P0 must sacrifice the champion creature (the "unless" clause fails because no valid exile target exists).
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** champion, no-valid-target, sacrifice, DEFERRED

**ATOM-702.72a-003**
- **Rule:** 702.72a — Champion: player chooses not to exile despite having a valid target → sacrifice.
- **Mechanism:** Champion ETB sacrifice when player declines to exile
- **Minimal Board:** P0 casts a creature with "Champion a creature." P0 controls another creature.
- **Action:** Champion ETB trigger fires. P0 chooses NOT to exile the other creature.
- **Expected Result:** P0 must sacrifice the champion creature. The "unless" clause means choosing not to exile is equivalent to failing the condition.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** champion, decline-exile, sacrifice, DEFERRED

**702.72b** — The two champion abilities are linked (rule 607). PURE-DEF (linked abilities reference).

**702.72c** — PURE-DEF. Defines "championed" terminology.

--- End of Chunk 7 ---

## Chunk 8 — 702.73–702.80 (Changeling, Evoke, Hideaway, Prowl, Reinforce, Conspire, Persist, Wither) + Classification Summary + COMP + Gap Report

### 702.73 — Changeling

**702.73a** — Changeling is a characteristic-defining ability: "This object is every creature type." Works everywhere, even outside the game.

**ATOM-702.73a-001**
- **Rule:** 702.73a — "Changeling" means "This object is every creature type." This ability works everywhere, even outside the game. See rule 604.3.
- **Mechanism:** Changeling grants all creature types as a CDA
- **Minimal Board:** P0 has a creature with changeling on the battlefield. An effect checks "if you control a Goblin."
- **Action:** The check is performed.
- **Expected Result:** The changeling creature counts as a Goblin (and every other creature type). This applies on the battlefield, in hand, in graveyard, in exile, and even outside the game.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires CDA implementation for creature types.
- **Tags:** changeling, CDA, creature-types

**ATOM-702.73a-002**
- **Rule:** 702.73a — Changeling works in all zones, including outside the game.
- **Mechanism:** Changeling type-granting persists across all zones
- **Minimal Board:** P0 has a changeling card in graveyard. An effect says "Return target Merfolk card from your graveyard to your hand."
- **Action:** P0 targets the changeling card.
- **Expected Result:** Targeting is legal — the changeling card is a Merfolk (and every other creature type) even in the graveyard.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** changeling, CDA, all-zones, DEFERRED

### 702.74 — Evoke

**702.74a** — Evoke: alternative cost from any castable zone + ETB sacrifice trigger if evoke cost was paid.

**ATOM-702.74a-001**
- **Rule:** 702.74a — "Evoke [cost]" means "You may cast this card by paying [cost] rather than paying its mana cost" and "When this permanent enters, if its evoke cost was paid, its controller sacrifices it."
- **Mechanism:** Evoke alternative cost + conditional sacrifice on ETB
- **Minimal Board:** P0 has a creature with "Evoke {1}{B}" (normal cost {3}{B}{B}, has an ETB ability: "destroy target nonblack creature") in hand.
- **Action:** P0 casts for evoke cost {1}{B}. The creature enters the battlefield. ETB ability triggers (destroy target). Evoke sacrifice trigger also triggers.
- **Expected Result:** Both triggers go on the stack (P0 chooses order). The ETB ability resolves (destroying the target). The evoke sacrifice trigger resolves (P0 sacrifices the evoked creature). Net result: target destroyed, evoke creature sacrificed.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires alternative cost framework (cross-ref T17) + conditional ETB sacrifice trigger.
- **Tags:** evoke, alternative-cost, ETB, sacrifice, T17

### 702.75 — Hideaway

**702.75a** — Hideaway: ETB trigger — look at top N cards, exile one face down, rest on bottom.

**ATOM-702.75a-001**
- **Rule:** 702.75a — "Hideaway N" means "When this permanent enters, look at the top N cards of your library. Exile one face down and put the rest on the bottom in a random order."
- **Mechanism:** Hideaway ETB: library peek + face-down exile + bottom remainder
- **Minimal Board:** P0 casts a land with "Hideaway 4."
- **Action:** The land enters. Hideaway triggers: P0 looks at top 4 cards, exiles one face down, puts 3 on the bottom.
- **Expected Result:** 1 card is in exile face down (P0 can look at it). 3 cards are on the bottom of P0's library in random order.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires face-down exile + library manipulation.
- **Tags:** hideaway, ETB, face-down-exile, DEFERRED

**702.75b** — Old hideaway cards had "enters tapped" and looked at 4 cards (now errata'd). PURE-DEF (Oracle errata note).

### 702.76 — Prowl

**702.76a** — Prowl: alternative cost if a source you controlled with a matching creature type dealt combat damage to a player this turn.

**ATOM-702.76a-001**
- **Rule:** 702.76a — "Prowl [cost]" means "You may pay [cost] rather than this spell's mana cost if a player was dealt combat damage this turn by a source that was under your control and had any of this spell's creature types."
- **Mechanism:** Prowl alternative cost conditioned on combat damage from shared creature type
- **Minimal Board:** P0 controls a Rogue that dealt combat damage to P1 this turn. P0 has a Rogue spell with "Prowl {1}{B}" (normal cost {3}{B}{B}).
- **Action:** P0 casts the Rogue spell for its prowl cost {1}{B}.
- **Expected Result:** Cast is legal at prowl cost because a Rogue P0 controlled dealt combat damage this turn. The spell resolves normally.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires combat-damage-by-type tracking + alternative cost.
- **Tags:** prowl, alternative-cost, creature-type, DEFERRED

### 702.77 — Reinforce

**702.77a** — Reinforce: activated ability from hand — pay cost, discard, put +1/+1 counters on target creature.

**ATOM-702.77a-001**
- **Rule:** 702.77a — "Reinforce N—[cost]" means "[Cost], Discard this card: Put N +1/+1 counters on target creature."
- **Mechanism:** Reinforce: hand-based ability to add +1/+1 counters
- **Minimal Board:** P0 has a card with "Reinforce 2—{1}{G}" in hand. P0 controls a creature.
- **Action:** P0 activates reinforce, paying {1}{G}, discarding the card, targeting the creature.
- **Expected Result:** The creature gets 2 +1/+1 counters.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires hand-activated ability + counter placement.
- **Tags:** reinforce, activated-ability, counters, DEFERRED

**702.77b** — Reinforce ability exists in all zones. PURE-DEF (same as cycling 702.29b).

### 702.78 — Conspire

**702.78a** — Conspire: tap two creatures sharing a color as additional cost; on cast, copy the spell if conspire was paid.

**ATOM-702.78a-001**
- **Rule:** 702.78a — "Conspire" means "As an additional cost, you may tap two untapped creatures you control that each share a color with it" and "When you cast this spell, if its conspire cost was paid, copy it."
- **Mechanism:** Conspire: tap creatures as additional cost + triggered copy
- **Minimal Board:** P0 has a red spell with conspire. P0 controls two untapped red creatures.
- **Action:** P0 casts the spell, tapping 2 red creatures as conspire cost. Conspire trigger copies the spell.
- **Expected Result:** 2 instances on the stack (original + 1 copy). The copy may have new targets.
- **Phase:** Phase 8 (niche)
- **Ticket:** DEFERRED — Phase 8. Requires tap-creatures-as-cost + color-sharing + spell copying.
- **Tags:** conspire, additional-cost, copy, DEFERRED

**702.78b** — Multiple instances of conspire are paid and trigger separately. DEFERRED — Phase 8. One-liner.

### 702.79 — Persist

**702.79a** — Persist: "When this permanent is put into a graveyard from the battlefield, if it had no -1/-1 counters, return it with a -1/-1 counter."

**ATOM-702.79a-001**
- **Rule:** 702.79a — "Persist" means "When this permanent is put into a graveyard from the battlefield, if it had no -1/-1 counters on it, return it to the battlefield under its owner's control with a -1/-1 counter on it."
- **Mechanism:** Persist death trigger: conditional return with -1/-1 counter. The "had no -1/-1 counters" check uses **LKI** (the creature's state at the moment it left the battlefield, not its state in the graveyard).
- **Minimal Board:** P0 controls a 3/3 creature with persist and no -1/-1 counters. The creature dies.
- **Action:** Persist trigger checks via LKI: creature had no -1/-1 counters when it died. It returns.
- **Expected Result:** The creature returns to the battlefield with a -1/-1 counter (now 2/2). If it dies again, persist doesn't trigger (LKI shows it had a -1/-1 counter this time).
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8 + Phase 5 LKI dependency (T20b). Requires death trigger + LKI counter check + return with counter.
- **Tags:** persist, death-trigger, minus-counters, LKI

**ATOM-702.79a-002**
- **Rule:** 702.79a — Persist does not trigger if the creature had -1/-1 counters when it died (checked via LKI).
- **Mechanism:** Persist condition: no -1/-1 counters at time of death (LKI check)
- **Minimal Board:** P0 controls a 3/3 with persist that has 1 -1/-1 counter on it (effectively 2/2). The creature dies.
- **Action:** Persist checks via LKI: creature had -1/-1 counters → persist does NOT trigger.
- **Expected Result:** The creature goes to the graveyard and stays there. Persist's LKI shows it had -1/-1 counters, so the condition is not met.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8 + Phase 5 LKI dependency (T20b).
- **Tags:** persist, no-trigger, minus-counters, LKI, DEFERRED

### 702.80 — Wither

**702.80a** — Wither: damage dealt to a creature by a source with wither causes -1/-1 counters instead of marked damage.

**ATOM-702.80a-001**
- **Rule:** 702.80a — Wither is a static ability. Damage dealt to a creature by a source with wither isn't marked on that creature. Rather, it causes that source's controller to put that many -1/-1 counters on that creature.
- **Mechanism:** Wither replaces damage marking with -1/-1 counters on creatures
- **Minimal Board:** P0 controls a 3/3 with wither, attacking. P1 controls a 4/4 creature blocking.
- **Action:** Combat damage: 3 damage from wither source to the 4/4.
- **Expected Result:** The 4/4 gets 3 -1/-1 counters instead of 3 damage marked. It becomes a 1/1 with no damage marked. It does NOT die from "lethal damage" SBA (toughness is still > 0 at 1). The 3/3 wither creature also takes 4 regular damage from the 4/4 blocker (the blocker doesn't have wither) and is destroyed by SBA (4 damage ≥ 3 toughness). Wither only changes how the wither source's damage is applied — the blocker's damage to the wither creature is normal marked damage.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8. Requires damage routing to place counters instead of marking damage.
- **Tags:** wither, minus-counters, damage-replacement

**ATOM-702.80a-002**
- **Rule:** 702.80a — Wither only affects damage to creatures, not to players.
- **Mechanism:** Wither doesn't change how damage is dealt to players
- **Minimal Board:** P0 controls a 2/2 with wither, attacking unblocked.
- **Action:** Combat damage: 2 damage to P1 (a player).
- **Expected Result:** P1 loses 2 life normally. No -1/-1 counters are involved — wither only applies to damage dealt to creatures.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** wither, player-damage, DEFERRED

**702.80b** — LKI determines wither after zone change.

**ATOM-702.80b-001**
- **Rule:** 702.80b — If an object changes zones before an effect causes it to deal damage, its last known information is used to determine whether it had wither.
- **Mechanism:** LKI preserves wither status for pending damage after zone change
- **Minimal Board:** P0 controls a creature with wither and a triggered ability "when this creature dies, it deals 2 damage to target creature." P1 controls a 4/4.
- **Action:** P0's wither creature dies. Death trigger deals 2 damage to the 4/4. LKI determines the source had wither.
- **Expected Result:** The 4/4 gets 2 -1/-1 counters (becoming 2/2) instead of 2 marked damage. Wither applies via LKI.
- **Phase:** Phase 8 (requires LKI system)
- **Ticket:** DEFERRED — Phase 8 + Phase 5 LKI dependency.
- **Tags:** wither, LKI, DEFERRED

**702.80c** — Wither functions from any zone. DEFERRED — Phase 8 (same pattern as deathtouch 702.2d and lifelink 702.15d). One-liner.

**702.80d** — PURE-DEF. Multiple instances of wither are redundant.

--- End of Chunk 8 (Rules) ---

---

## Classification Summary Table

| Rule Range | Keyword | Classification | Phase | Ticket | Notes |
|---|---|---|---|---|---|
| 702.1 | (General) | PURE-DEF | — | — | Defines keyword ability concept |
| 702.2 | Deathtouch | ALREADY-IMPL + DEFERRED(c,d,e) | P4 done / P5-P6 | T20b | Core done; LKI + any-zone deferred |
| 702.3 | Defender | ALREADY-IMPL | P4 done | — | Fully implemented |
| 702.4 | Double Strike | ALREADY-IMPL + DEFERRED(c,d) | P4 done / P5 | — | Core done; mid-combat grant/remove deferred |
| 702.5 | Enchant | TESTABLE | P5-Pre | T15b | Aura targeting restrictions |
| 702.6 | Equip | TESTABLE + DEFERRED(c,d,e) | P5-Pre / P8 | T15b | Core equip testable; quality/PW/multi-equip variants deferred. 702.6d reclassified from PURE-DEF to TESTABLE. |
| 702.7 | First Strike | ALREADY-IMPL + DEFERRED(c) | P4 done / P5 | — | Core done; mid-combat grant/remove deferred |
| 702.8 | Flash | TESTABLE + DEFERRED(a-zone) | P5-Pre / P8 | T18 | Basic flash testable; any-zone deferred |
| 702.9 | Flying | ALREADY-IMPL | P4 done | — | Fully implemented |
| 702.10 | Haste | ALREADY-IMPL | P4 done | — | Fully implemented |
| 702.11 | Hexproof | TESTABLE + DEFERRED(c,d,e) | P5-Pre / P8 | T22 | Permanent hexproof testable; player/from-quality deferred |
| 702.12 | Indestructible | TESTABLE | P5-Pre | T09 | Both lethal-damage and destroy-effect tests |
| 702.13 | Intimidate | DEFERRED | P8 | — | Deprecated evasion keyword |
| 702.14 | Landwalk | DEFERRED | P8 | — | Niche evasion |
| 702.15 | Lifelink | ALREADY-IMPL + DEFERRED(c,d) | P4 done / P5-P6 | T20b | Core done; LKI + any-zone deferred |
| 702.16 | Protection | TESTABLE + DEFERRED(j,k,n,p) | P5-Pre / P8-P9 | T22 | Core DEBT tests; everything/player/self-exception deferred |
| 702.17 | Reach | ALREADY-IMPL | P4 done | — | Fully implemented |
| 702.18 | Shroud | TESTABLE + DEFERRED(player) | P5-Pre / P8 | T22 | Permanent shroud testable; player shroud deferred |
| 702.19 | Trample | ALREADY-IMPL + TESTABLE(d) + DEFERRED(c,e,f) | P4 done / P8 | — | Core done; all-blockers-removed regression NEW; PW variants deferred |
| 702.20 | Vigilance | ALREADY-IMPL | P4 done | — | Fully implemented |
| 702.21 | Ward | TESTABLE + DEFERRED(b) | P7 | NEW | Triggered ability; ward-X deferred |
| 702.22 | Banding | OUT-OF-SCOPE (all) | — | — | Disproportionate implementation cost (~15 cards), invasive combat subsystem changes, combinatorial interaction surface with trample/deathtouch/wither. Stretch goal only. |
| 702.23 | Rampage | DEFERRED | P8 | — | Niche triggered ability |
| 702.24 | Cumulative Upkeep | DEFERRED | P8 | — | Niche triggered ability |
| 702.25 | Flanking | DEFERRED | P8 | — | Niche triggered ability |
| 702.26 | Phasing | DEFERRED | P8 | — | Complex status change mechanic |
| 702.27 | Buyback | DEFERRED | P8 | T17 | Additional cost + hand return |
| 702.28 | Shadow | DEFERRED | P8 | — | Niche bidirectional evasion |
| 702.29 | Cycling | DEFERRED | P8 | — | Hand-based activated ability |
| 702.30 | Echo | DEFERRED | P8 | — | Niche upkeep trigger |
| 702.31 | Horsemanship | DEFERRED | P8 | — | Niche evasion (flying clone) |
| 702.32 | Fading | DEFERRED | P8 | — | Niche counter mechanic |
| 702.33 | Kicker | DEFERRED | P8 | T17 | Additional cost + kicked status |
| 702.34 | Flashback | DEFERRED | P8 | T17 | Alternative cost from graveyard |
| 702.35 | Madness | DEFERRED | P8 | — | Replacement effect on discard |
| 702.36 | Fear | DEFERRED | P8 | — | Deprecated evasion |
| 702.37 | Morph | DEFERRED | P8 | — | Face-down infrastructure |
| 702.38 | Amplify | DEFERRED | P8 | — | Niche ETB counters |
| 702.39 | Provoke | DEFERRED | P8 | — | Niche forced-block |
| 702.40 | Storm | DEFERRED | P8 | — | Spell copying |
| 702.41 | Affinity | DEFERRED | P8 | T17 | Cost reduction |
| 702.42 | Entwine | DEFERRED | P8 | — | Modal + additional cost |
| 702.43 | Modular | DEFERRED | P8 | — | ETB counters + death trigger |
| 702.44 | Sunburst | DEFERRED | P8 | — | Mana-color tracking |
| 702.45 | Bushido | DEFERRED | P8 | — | Niche triggered ability |
| 702.46 | Soulshift | DEFERRED | P8 | — | Niche death trigger |
| 702.47 | Splice | DEFERRED | P8 | — | Text-changing during cast |
| 702.48 | Offering | DEFERRED | P8 | — | Sacrifice + cost reduction |
| 702.49 | Ninjutsu | DEFERRED | P8 | — | Hand-based swap mechanic |
| 702.50 | Epic | DEFERRED | P8 | — | Casting restriction + delayed trigger |
| 702.51 | Convoke | DEFERRED | P8 | T17 | Creature-tap-as-mana |
| 702.52 | Dredge | DEFERRED + TESTABLE(b) | P8 | — | Draw replacement. 702.52b reclassified from PURE-DEF to TESTABLE (boundary). |
| 702.53 | Transmute | DEFERRED | P8 | — | Hand-based tutor |
| 702.54 | Bloodthirst | DEFERRED | P8 | — | Conditional ETB counters |
| 702.55 | Haunt | DEFERRED | P8 | — | Exile association mechanic |
| 702.56 | Replicate | DEFERRED | P8 | — | Additional cost + spell copying |
| 702.57 | Forecast | DEFERRED | P8 | — | Hand-based upkeep ability |
| 702.58 | Graft | DEFERRED | P8 | — | ETB counters + triggered counter sharing |
| 702.59 | Recover | DEFERRED | P8 | — | Graveyard trigger |
| 702.60 | Ripple | DEFERRED | P8 | — | Library reveal + free cast |
| 702.61 | Split Second | DEFERRED | P8 | — | Stack restriction |
| 702.62 | Suspend | DEFERRED | P8 | — | Exile + time counters + free cast |
| 702.63 | Vanishing | DEFERRED + TESTABLE(b) | P8 | — | Time counters + sacrifice. 702.63b expanded from one-liner to 2 ATOMs (0-counter + external counter). |
| 702.64 | Absorb | DEFERRED | P8 | — | Damage prevention per source |
| 702.65 | Aura Swap | DEFERRED | P8 | — | Exchange mechanic |
| 702.66 | Delve | DEFERRED | P8 | T17 | Graveyard exile as payment |
| 702.67 | Fortify | DEFERRED | P8 | — | Land attachment |
| 702.68 | Frenzy | DEFERRED | P8 | — | Niche triggered ability |
| 702.69 | Gravestorm | DEFERRED | P8 | — | Died-this-turn counting + copies |
| 702.70 | Poisonous | DEFERRED | P8 | — | Poison counters |
| 702.71 | Transfigure | DEFERRED | P8 | — | Sacrifice-tutor |
| 702.72 | Champion | DEFERRED | P8 | — | ETB exile + LTB return |
| 702.73 | Changeling | DEFERRED | P8 | — | CDA for creature types |
| 702.74 | Evoke | DEFERRED | P8 | T17 | Alternative cost + ETB sacrifice |
| 702.75 | Hideaway | DEFERRED | P8 | — | Face-down exile (hidden information only; does NOT require rule 708 face-down infrastructure) |
| 702.76 | Prowl | DEFERRED | P8 | — | Conditional alternative cost |
| 702.77 | Reinforce | DEFERRED | P8 | — | Hand-based counter ability |
| 702.78 | Conspire | DEFERRED | P8 | — | Tap-as-cost + spell copying |
| 702.79 | Persist | DEFERRED | P8 | T20b | Death trigger + conditional return. LKI dependency (Phase 5 T20b). |
| 702.80 | Wither | DEFERRED | P8 | — | Damage → -1/-1 counters |

## Composition (COMP) Tests

These cross-keyword tests exercise interactions between multiple keyword abilities on one or more objects.

**COMP-702-001: Deathtouch + First Strike**
- **Rule Pair:** 702.2b + 702.7b
- **Mechanism:** A first-striking deathtouch creature kills any toughness blocker in the first step before the blocker can deal damage back.
- **Minimal Board:** P0 controls a 1/1 with deathtouch + first strike, attacking. P1 blocks with a 5/5.
- **Expected Result:** First combat damage step: 1 deathtouch damage to 5/5 → destroyed by SBA (deathtouch makes any amount lethal). Second step: 5/5 is dead, deals no damage. P0's 1/1 survives.
- **Status:** ALREADY-IMPLEMENTED (both keywords present, engine handles step ordering).
- **Tags:** deathtouch, first-strike, composition

**COMP-702-002: Deathtouch + Trample**
- **Rule Pair:** 702.2b + 702.19b
- **Mechanism:** When assigning trample damage with deathtouch, 1 damage to each blocker is "lethal" (per rule 702.2b), so excess overflows.
- **Minimal Board:** P0 controls a 6/6 with deathtouch + trample, attacking. P1 blocks with a 5/5 and a 3/3.
- **Expected Result:** P0 assigns 1 to the 5/5 (lethal with deathtouch) and 1 to the 3/3 (lethal with deathtouch), and 4 tramples to P1. Both blockers die.
- **Status:** ALREADY-IMPLEMENTED (deathtouch+trample interaction in `assign_trample_damage`).
- **Tags:** deathtouch, trample, composition

**COMP-702-003: Double Strike + Lifelink**
- **Rule Pair:** 702.4b + 702.15b
- **Mechanism:** A double-striking creature with lifelink gains life in both combat damage steps.
- **Minimal Board:** P0 controls a 3/3 with double strike + lifelink, attacking unblocked. P0 is at 10 life.
- **Expected Result:** First step: 3 damage to P1, P0 gains 3 (→13). Second step: 3 more damage to P1, P0 gains 3 (→16). Total: 6 damage dealt, 6 life gained.
- **Status:** ALREADY-IMPLEMENTED.
- **Tags:** double-strike, lifelink, composition

**COMP-702-004: Flying + Reach (the classic)**
- **Rule Pair:** 702.9b + 702.17b
- **Mechanism:** A creature with reach can block a creature with flying.
- **Minimal Board:** P0 controls a 3/3 with flying, attacking. P1 controls a 2/4 with reach.
- **Expected Result:** The 2/4 with reach can legally block the 3/3 with flying.
- **Status:** ALREADY-IMPLEMENTED.
- **Tags:** flying, reach, composition

**COMP-702-005: Protection + Trample**
- **Rule Pair:** 702.16e + 702.19b
- **Mechanism:** Protection prevents ALL damage from matching sources, but trample still requires assigning "lethal" damage to blockers before overflow. The damage assigned to a blocker with protection is prevented, but the attacker still calculates trample overflow based on lethal damage assignment.
- **Minimal Board:** P0 controls a 7/7 red creature with trample, attacking. P1 blocks with a 2/2 with protection from red.
- **Expected Result:** P0 must assign at least 2 to the blocker (lethal = toughness). Then 5 tramples to P1. The 2 damage assigned to the protection creature is prevented, so the blocker survives. P1 takes 5.
- **Phase:** Phase 5-Pre
- **Ticket:** T22 + existing trample
- **Tags:** protection, trample, composition, damage-prevention

**COMP-702-006: Wither + Persist**
- **Rule Pair:** 702.80a + 702.79a
- **Mechanism:** A persist creature that takes wither damage gets -1/-1 counters. If it dies with -1/-1 counters, persist doesn't trigger.
- **Minimal Board:** P0 controls a 3/3 with persist (no counters). P1's wither creature deals 2 damage to it (2 -1/-1 counters → now 1/1). Then another source deals 1 damage.
- **Expected Result:** The creature dies (1 damage on a 1-toughness creature). Persist checks: creature had -1/-1 counters → persist does NOT trigger. The creature stays in graveyard.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** wither, persist, composition, DEFERRED

**COMP-702-007: Hexproof Granted Between Cast and Resolution → Fizzle**
- **Rule Pair:** 702.11b + 608.2b
- **Mechanism:** Granting hexproof to a permanent after an opponent's spell is cast but before it resolves causes the spell to fizzle (all targets illegal at resolution).
- **Minimal Board:** P1 casts "Destroy target creature" targeting P0's creature. Before the spell resolves, P0 grants hexproof to the creature (e.g., via an instant).
- **Expected Result:** When the destroy spell tries to resolve, the target is illegal (hexproof from opponents). All targets are illegal → spell fizzles per rule 608.2b.
- **Phase:** Phase 5-Pre
- **Ticket:** T22 + targeting legality at resolution
- **Tags:** hexproof, targeting, fizzle, composition

**COMP-702-008: Cumulative Upkeep + Solemnity (Counters Can't Be Placed)**
- **Rule Pair:** 702.24a + Solemnity
- **Mechanism:** If age counters can't be placed on a permanent with cumulative upkeep (e.g., Solemnity), the CU trigger adds 0 age counters, and the cost is [cost] × 0 = nothing. The permanent persists indefinitely.
- **Minimal Board:** P0 controls Solemnity and a creature with "Cumulative upkeep {2}." It is P0's upkeep.
- **Expected Result:** CU trigger fires. It tries to add an age counter → prevented by Solemnity. P0 pays {2} × 0 (age counters) = {0}. The creature is not sacrificed. This repeats every upkeep with the same result.
- **Phase:** Phase 8
- **Ticket:** DEFERRED — Phase 8.
- **Tags:** cumulative-upkeep, solemnity, counters, composition, DEFERRED

## META Notes (Audit Round 1)

**META-7B-01: Unified Evasion Framework (NEW-3)**
Flying, Shadow, Fear, Intimidate, Horsemanship, and Landwalk all share the pattern "can't be blocked except by [filter]." Current engine hardcodes flying check in `validate_blockers`. Proposed: `EvasionRestriction` struct with `BlockerFilter` enum (`HasAnyKeyword`, `Bidirectional`, `ArtifactOrSharesColor`, `HasKeyword`, `ConditionalUnblockable`). Validation loop becomes `for evasion in get_evasion_restrictions(game, attacker_id)`. Implement when Phase 8 lands the first non-Flying evasion keyword.

**META-7B-02: ProtectionQuality Enum**
The CR uses "quality" informally for hexproof-from and protection-from. Proposed enum: `Color(Color)`, `CardType(CardType)`, `Subtype(SubtypeId)`, `CardName(String)`, `Everything`, `Player(PlayerId)`, `ManaValueAtMost(u32)`. A centralized `matches_quality()` function serves both hexproof-from and protection.

**META-7B-03: Copy-Spell vs Copy-Card**
Pattern A (Storm/Replicate/Conspire): copy spell directly on stack, not "cast," doesn't trigger cast triggers. Pattern B (rare exile effects): copy a card in a zone, grant cast permission, IS casting. Pattern C (Fork/Reverberate): copy target existing spell on stack. Engine needs `copy_spell_on_stack()` (Pattern A/C) and `create_card_copy()` (Pattern B).

**META-7B-04: Unified Trample DP (NEW-4)**
Proposed `TrampleContext` struct with `blockers`, `intermediates` (planeswalker loyalty / battle defense thresholds), and `final_target` (player). Normal trample: empty intermediates. Trample-over-PW: one intermediate with loyalty threshold. One DP method, one validation path.

**META-7B-05: Protection-from-Everything Is an Enum Variant, Not a Base Case**
`ProtectionQuality::Everything` where `matches_quality()` returns true unconditionally. Additive matching ("protection FROM [quality]") is the correct mental model, not subtractive filtering ("block everything except").

**META-7B-06: Haunting Is a Designation**
"Haunting" / "is haunted by" is a game-state relationship (like "paired" or "renowned"), not a characteristic (rule 109.3). Track as an association between exiled card and haunted permanent.

**META-7B-07: Splice Implementation Sketch**
Splice could be modeled as a temporary continuous effect that adds rules text to a spell during casting (step 601.2b). Conceptually, this operates in the text-changing layer (layer 3, rule 613.1c) — distinct from ability-granting (layer 6). The engine would: (1) during casting, if player chooses to splice, create a `SpliceEffect { source_card_id: ObjectId, rules_text: Vec<Effect> }` attached to the spell on the stack; (2) during resolution, the spell executes its own effects followed by the spliced effects; (3) when the spell leaves the stack, the splice effect is stripped. The splice card stays in hand (not cast, not discarded). Key difference from normal ability-granting: splice adds *rules text* wholesale (including targets), not individual keyword abilities. This is closer to "the spell has additional resolution steps" than "the permanent gains an ability." The `SpliceEffect` struct would carry the parsed effect tree from the splice source, avoiding actual text manipulation. This approach means splice doesn't need a true text-changing engine — it just needs the effect resolver to iterate over `[spell_effects] ++ [splice_effects]` at resolution time.

## Gap Report

### Infrastructure Dependencies (blocking multiple keywords)

1. **Cost Modification Framework (T17)** — Blocks: Kicker, Buyback, Flashback, Affinity, Convoke, Delve, Evoke, Offering, Prowl. The `601.2e` cost pipeline (base → increases → reductions → Trinisphere floor) is documented in `cast.rs` but not implemented.

2. **LKI System (T20b)** — Blocks: Deathtouch-LKI (702.2e), Lifelink-LKI (702.15c), Wither-LKI (702.80b). Required for any "zone change before pending damage" scenario.

3. **Continuous Effects & Layers (Phase 5)** — Blocks: Mid-combat keyword grant/removal tests (702.4c/d, 702.7c). Required for dynamic keyword changes.

4. **Triggered Abilities (Phase 7)** — Blocks: Ward (702.21), plus all Phase 8 triggered-ability keywords (Rampage, Flanking, Bushido, Storm, etc.).

5. **Spell Copying** — Blocks: Storm (702.40), Replicate (702.56), Conspire (702.78), Gravestorm (702.69).

6. **Face-Down Infrastructure (rule 708)** — Blocks: Morph (702.37), Hideaway (702.75).

7. **Aura/Equipment Attachment Model (T15b)** — Blocks: Enchant (702.5), Equip (702.6).

### NEW Tickets Identified

| ID | Keyword | Description | Phase |
|---|---|---|---|
| NEW-1 | Ward | Ward triggered ability: on-target trigger + counter-unless-pay | Phase 7 |
| NEW-2 | Trample | Regression test: `assign_trample_damage` with empty blocker list (all blockers removed before damage) | Phase 4 (immediate) |
| NEW-3 | Evasion | Unified Evasion Framework: `EvasionRestriction` + `BlockerFilter` enum in `validate_blockers` | Phase 8 |
| NEW-4 | Trample | Unified Trample DP: `TrampleContext` struct for trample / trample-over-PW / trample-over-battle | Phase 8 |

### Statistics

- **Total top-level rules audited:** 80 (702.1–702.80)
- **Total sub-rules classified:** ~260
- **ATOM tests generated:** 106 (was 87; +22 from audit round 1, −3 banding skeletons removed in round 2)
- **COMP tests generated:** 8 (was 6; +2 from audit round 1)
- **META notes:** 7 (6 from audit round 1, +1 from round 2: splice implementation sketch)
- **ALREADY-IMPLEMENTED keywords:** 10 (Deathtouch*, Defender, Double Strike*, First Strike*, Flying, Haste, Lifelink*, Reach, Trample*, Vigilance) — * = has deferred sub-rules
- **TESTABLE (immediate/near-term) keywords:** 7 (Enchant, Equip, Flash, Hexproof, Indestructible, Protection, Shroud)
- **DEFERRED keywords:** 57 (702.13–702.14, 702.23–702.80 minus already-implemented; was 58, −1 for banding OUT-OF-SCOPE)
- **Reclassifications:** 5 (702.6d PURE-DEF→TESTABLE, 702.22 DEFERRED→OUT-OF-SCOPE, 702.52b PURE-DEF→TESTABLE, 702.63b one-liner→2 ATOMs, 702.37g hideaway dependency corrected)
- **OUT-OF-SCOPE:** 702.22 (banding, all sub-rules) + 702.33h (sticker kicker, Un-set)
- **NEW tickets flagged:** 4 (was 2; +NEW-3, +NEW-4)

--- End of Session 7b ---
