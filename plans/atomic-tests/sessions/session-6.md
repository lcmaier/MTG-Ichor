# Session 6: Atomic Test Specifications — CR 609–616

> **Scope:** Effects, One-Shot Effects, Continuous Effects, Text-Changing Effects, Layer System (613), Replacement Effects (614), Prevention Effects (615), Interaction of Replacement/Prevention (616)
> **CR file:** `MTG-Rules/LLM-Chapter-Splits/ch6-pt-2.txt`
> **Session 5 covered:** 600–608 (spells, abilities, resolving)
> **Date:** 2026-04-06

---

## 609. Effects

### 609.1 — PURE-DEF
An effect is something that happens as a result of a spell or ability. Defines the concept of "effect" — one-shot vs continuous, static vs resolving. No independent mechanical consequence; other rules reference this definition.

### 609.2 — TESTABLE

Effects apply only to permanents unless the instruction's text states otherwise or they clearly can apply only to objects in other zones.

**ATOM-609.2-001**
- **Rule:** 609.2 — Effects default to applying to permanents only
- **Mechanism:** A continuous effect that says "all lands become creatures" should not affect land cards in graveyards
- **Minimal Board:** Player A controls an effect "All lands are 2/2 creatures." Player A has a Forest on the battlefield and a Forest card in their graveyard.
- **Action:** Query effective characteristics of both the battlefield Forest and the graveyard Forest.
- **Expected Result:** Battlefield Forest is a 2/2 creature land. Graveyard Forest is NOT a creature — it retains only its printed characteristics.
- **Phase:** Phase 5 Layers (L10 — type-changing effects)
- **Ticket:** L10

**ATOM-609.2-002**
- **Rule:** 609.2 — Effects that clearly apply to non-permanent zones do so
- **Mechanism:** An effect that says "spells cost {1} more to cast" applies only to spells on the stack
- **Minimal Board:** Player A controls Thalia, Guardian of Thraben (noncreature spells cost {1} more). Player A has Lightning Bolt in hand (cost {R}).
- **Action:** Player A casts Lightning Bolt.
- **Expected Result:** Total cost to cast is {1}{R}, not {R}. The cost increase applies to the spell on the stack, even though Thalia's effect targets "noncreature spells" (objects in the stack zone).
- **Phase:** Phase 5 Layers (L15 — cost modification scaffolding)
- **Ticket:** L15

### 609.3 — TESTABLE (META-adjacent)

> **Audit note:** This rule is quasi-meta — nearly all effects are subject to it. We test the two most common impossible-instruction patterns (discard and mill). Exhaustive coverage would mean testing every primitive against an impossible precondition, which is over-exhaustive. These two tests establish the engine's partial-execution contract; further primitives should inherit the same behavior.

If an effect attempts to do something impossible, it does only as much as possible.

**ATOM-609.3-001**
- **Rule:** 609.3 — Impossible partial effect: discard more than hand size
- **Mechanism:** Engine must perform as much as possible of an impossible instruction
- **Minimal Board:** Player A holds 1 card. A resolving spell says "Discard two cards."
- **Action:** Resolve the discard effect.
- **Expected Result:** Player A discards the 1 card they hold. Hand is now empty. No error raised — the "second discard" is silently ignored.
- **Phase:** Phase 8 (Discard primitive)
- **Ticket:** NEW — Partial effect execution for impossible instructions

**ATOM-609.3-002**
- **Rule:** 609.3 — Impossible partial effect: move cards from library
- **Mechanism:** Engine must move as many cards as possible when library has fewer than requested
- **Minimal Board:** Player A's library has 2 cards. A resolving effect says "Mill four cards."
- **Action:** Resolve the mill effect.
- **Expected Result:** 2 cards are milled (moved from library to graveyard). The other 2 are silently ignored. No error.
- **Phase:** Phase 8 (Mill primitive)
- **Ticket:** NEW — Mill handles insufficient library size gracefully

### 609.4 — TESTABLE

"As though" effects apply only to the stated effect.

**ATOM-609.4-001**
- **Rule:** 609.4 — "As though" is limited to the stated effect
- **Mechanism:** An "as though it had flash" effect allows instant-speed casting but does NOT grant the flash keyword for other purposes
- **Minimal Board:** Player A controls Vedalken Orrery ("You may cast spells as though they had flash"). Player A has a sorcery in hand.
- **Action:** (1) Player A casts the sorcery during opponent's turn (instant timing). (2) Query: does the sorcery on the stack have the keyword Flash?
- **Expected Result:** (1) Cast succeeds — timing check passes due to "as though flash." (2) The spell does NOT have keyword Flash — an effect that triggers "whenever a player casts a spell with flash" would NOT trigger.
- **Phase:** Phase 8 (keyword grant vs "as though")
- **Ticket:** NEW — "As though" effects scoped to stated effect only

### 609.4a — TESTABLE

If two "as though" effects apply to the same action, both conditions can apply simultaneously. The first sentence means: when two "as though" effects grant different permissions for the same action and neither contradicts the other, both permissions are available to the player.

**ATOM-609.4a-001**
- **Rule:** 609.4a — Two "as though" effects combine
- **Mechanism:** Multiple "as though" conditions stack for the same action
- **Minimal Board:** Player A controls Vedalken Orrery ("cast spells as though they had flash") and has resolved Shaman's Trance ("play lands and cast spells from other players' graveyards this turn as though those cards were in your graveyard"). Opponent's graveyard contains a sorcery with flashback.
- **Action:** Player A attempts to cast the sorcery from opponent's graveyard via flashback during opponent's turn.
- **Expected Result:** Cast succeeds. Both "as though" conditions apply: treated as though in your graveyard (for flashback zone check) AND as though it had flash (for timing).
- **Phase:** Phase 8
- **Ticket:** NEW — Multiple "as though" conditions compose

### 609.4b — TESTABLE

"As though it were mana of any type" affects only how mana pays costs, not what mana was spent.

**ATOM-609.4b-001**
- **Rule:** 609.4b — "Mana of any type" doesn't change actual mana identity
- **Mechanism:** Spending mana "as though it were any color" pays the cost but the mana's actual color is unchanged for effects that care
- **Minimal Board:** Player A has {G} in mana pool and an effect allowing spending mana as though it were any color. Player A casts a spell with cost {U}.
- **Action:** Pay {U} cost using {G} mana with the "any type" permission.
- **Expected Result:** Cost is paid successfully. If any effect checks "what color of mana was spent to cast this spell," the answer is Green, not Blue.
- **Phase:** Phase 8
- **Ticket:** NEW — Mana spending "as though" preserves actual mana identity

**ATOM-609.4b-002**
- **Rule:** 609.4b — "Mana of any color" vs "mana of any type" distinction
- **Mechanism:** "Any color" covers {W}{U}{B}{R}{G}; "any type" also covers {C}. Engine must distinguish these.
- **Minimal Board:** Player A has {C} in mana pool and an effect allowing spending mana as though it were mana of any color. Player A attempts to cast a spell with cost {U}.
- **Action:** Pay {U} cost using {C} mana with "any color" permission.
- **Expected Result:** Payment FAILS. {C} is colorless, not a color. "Any color" doesn't make colorless mana act as colored. (With "any type" permission, it would succeed.)
- **Phase:** Phase 8
- **Ticket:** NEW — "Any color" vs "any type" mana distinction

**ATOM-609.4b-003**
- **Rule:** 609.4b — Mana restrictions persist through "as though" spending
- **Mechanism:** Mana with a spending restriction ("spend only to cast creature spells") cannot fulfill a cost for a non-creature spell even with an "as though any color" permission
- **Minimal Board:** Player A has {C} with restriction "spend only to cast creature spells." Player A controls an effect: "You may spend mana as though it were mana of any color to cast planeswalker spells." Player A casts a planeswalker spell costing {2}{W}.
- **Action:** Attempt to use the restricted {C} for the {2} generic portion of the planeswalker cost.
- **Expected Result:** Payment FAILS for that mana. The restriction "only creature spells" is not overridden by the "as though any color" permission — restrictions are separate from color identity. Player A must use unrestricted mana.
- **Phase:** Phase 8
- **Ticket:** NEW — Mana spending restrictions persist through "as though"

### 609.5 — PURE-DEF
Ties are resolved by the spell or ability text. No default tie resolution in the game. This is a meta-rule about how card text works — no independent mechanical test.

### 609.6 — PURE-DEF
Some continuous effects are replacement or prevention effects. Cross-reference to 614/615 — definitional pointer.

### 609.7 — PURE-DEF
Some effects apply to damage from a source. Introduces the concept of damage-source effects. Mechanical details are in 609.7a–c.

### 609.7a — BOUNDARY-DEF

Defines what constitutes a valid "source of damage" choice.

**BOUNDARY-DEF-609.7a-001**
- **Rule:** 609.7a — Valid damage source choices
- **Mechanism:** Engine must validate source-of-damage choices; permanents, stack spells, and objects referenced by stack/replacement/delayed triggers are valid; random objects in hand are not
- **In-set member:** A creature permanent on the battlefield — valid damage source choice
- **Out-of-set member:** A card in a player's hand that is not referenced by anything on the stack — NOT a valid damage source choice
- **Phase:** Phase 6 (Replacement/Prevention effects with source choice)
- **Ticket:** NEW — Damage source choice validation

**ATOM-609.7a-001**
- **Rule:** 609.7a — All valid source categories are selectable
- **Mechanism:** Per the CR, a source of damage is any permanent, any spell on the stack, or any object referred to by a stack object, replacement/prevention, or delayed trigger. Test each category.
- **Minimal Board:** (a) Creature permanent on battlefield. (b) Lightning Bolt on the stack. (c) An emblem that references a card in exile ("exile a card... that card deals damage").
- **Action:** An effect asks the player to choose a source of damage. Player selects each of (a), (b), (c).
- **Expected Result:** All three are legal choices. The engine accepts permanents, stack objects, and objects referred to by stack/replacement/delayed effects.
- **Phase:** Phase 6 (Replacement/Prevention effects)
- **Ticket:** NEW — Full source-of-damage category validation

### 609.7b — TESTABLE

Shield from resolved spell/ability rechecks source properties; if no longer matching, shield is not used up.

**ATOM-609.7b-001**
- **Rule:** 609.7b — Prevention shield rechecks source properties
- **Mechanism:** A "prevent damage from a red source" shield must recheck color at damage time
- **Minimal Board:** Player A has a prevention shield: "Prevent the next 2 damage a red source would deal to you." Opponent controls a Mountain-Walking creature that is currently red. An effect changes the creature to blue.
- **Action:** The (now blue) creature deals combat damage to Player A.
- **Expected Result:** Damage is NOT prevented — the source is no longer red. The shield is NOT used up (it remains for the next applicable red source).
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Prevention shield source property rechecking

### 609.7c — TESTABLE

Prevention/replacement from static abilities apply to permanents with the property AND non-battlefield sources with the property.

**ATOM-609.7c-001**
- **Rule:** 609.7c — Static prevention applies to non-battlefield sources too
- **Mechanism:** A static ability "prevent all damage from red sources" applies to a red instant on the stack dealing damage, not just red permanents
- **Minimal Board:** Player A controls a permanent with "If a red source would deal damage to you, prevent that damage." Opponent casts Lightning Bolt (a red instant) targeting Player A.
- **Action:** Lightning Bolt resolves, dealing 3 damage to Player A.
- **Expected Result:** Damage is prevented. The static prevention applies because Lightning Bolt is a red source (it's on the stack/resolving, which is a non-battlefield zone, but the static ability covers non-battlefield sources with the matching property).
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Static prevention covers non-battlefield sources

---

## 610. One-Shot Effects

### 610.1 — PURE-DEF
A one-shot effect does something once with no duration. Examples: dealing damage, destroying, creating tokens, zone changes. Definitional — no independent test.

### 610.2 — PURE-DEF
Some one-shot effects create delayed triggered abilities. Cross-reference to rule 603.7. Definitional pointer.

### 610.3 — TESTABLE

Zone change "until" effects create a second one-shot effect that returns the object when the condition is met.

**ATOM-610.3-001**
- **Rule:** 610.3 — "Until" zone change creates return effect
- **Mechanism:** O-Ring pattern: exile a creature "until [source] leaves the battlefield" — when source leaves, creature returns
- **Minimal Board:** Player A controls Banisher Priest (exile target creature an opponent controls until Banisher Priest leaves the battlefield). Opponent's creature has been exiled by the Priest.
- **Action:** Banisher Priest is destroyed (leaves the battlefield).
- **Expected Result:** The exiled creature returns to the battlefield under its owner's control.
- **Phase:** Phase 7 (Triggered abilities — delayed triggers + linked abilities)
- **Ticket:** NEW — "Until leaves" zone-change return effect (D9 in roadmap)

### 610.3a — TESTABLE

If the "until" event already occurred before the initial one-shot resolves (but after the spell was put on the stack), the object doesn't move.

**ATOM-610.3a-001**
- **Rule:** 610.3a — "Until" event already occurred before effect resolves (spell/ability)
- **Mechanism:** If the source that would define the "until" condition has already left before the exile effect resolves, the target doesn't move
- **Minimal Board:** Player A casts a spell: "Exile target creature until target enchantment leaves the battlefield." In response, opponent destroys the targeted enchantment. The spell resolves.
- **Action:** The spell's exile effect attempts to resolve.
- **Expected Result:** The target creature is NOT exiled. The "until" event (enchantment leaving) has already occurred, so the initial one-shot effect does nothing.
- **Phase:** Phase 7 (Triggered abilities)
- **Ticket:** NEW — "Until" pre-event check for spells/abilities

### 610.3b — TESTABLE

Same as 610.3a but for triggered abilities: if the "until" event occurred after the trigger but before resolution, object doesn't move.

**ATOM-610.3b-001**
- **Rule:** 610.3b — "Until" event already occurred before triggered ability resolves
- **Mechanism:** Same as 610.3a but trigger-sourced
- **Minimal Board:** A triggered ability says "When this enters, exile target creature until this leaves." After the trigger goes on the stack but before it resolves, the source is bounced to hand.
- **Action:** The triggered ability resolves.
- **Expected Result:** Target creature is NOT exiled — the "until" event occurred between trigger and resolution.
- **Phase:** Phase 7 (Triggered abilities)
- **Ticket:** NEW — "Until" pre-event check for triggered abilities

### 610.3c — TESTABLE

Object returned to battlefield by "until" effect returns under its OWNER's control unless otherwise specified.

**ATOM-610.3c-001**
- **Rule:** 610.3c — "Until" return goes to owner's control
- **Mechanism:** When an "until" effect returns an object, it enters under the owner's control, NOT the controller of the effect
- **Minimal Board:** Player A controls Banisher Priest, which exiled a creature owned by Player B (but previously controlled by Player A via a control-changing effect). Banisher Priest leaves.
- **Action:** The exiled creature returns to the battlefield.
- **Expected Result:** Creature enters under Player B's control (the owner), not Player A's.
- **Phase:** Phase 7 (Triggered abilities)
- **Ticket:** NEW — "Until" return defaults to owner's control

**ATOM-610.3c-002**
- **Rule:** 610.3c — "Until" return under specific controller when stated
- **Mechanism:** Some effects explicitly say "return under your control" — this overrides the owner-default
- **Minimal Board:** Player A controls a permanent with ability: "Exile target creature until this permanent leaves the battlefield. Return it under your control." Player A exiles Player B's creature.
- **Action:** The source permanent leaves the battlefield.
- **Expected Result:** Player B's creature returns to the battlefield under Player A's control (not Player B's), because the effect explicitly specifies "your control."
- **Phase:** Phase 7 (Triggered abilities)
- **Ticket:** NEW — "Until" return under specific controller override

### 610.3d — TESTABLE

Multiple simultaneous "until" returns happen simultaneously.

**ATOM-610.3d-001**
- **Rule:** 610.3d — Simultaneous "until" returns are simultaneous
- **Mechanism:** Two Banisher Priests each exiling a creature; both die to Day of Judgment — both exiled creatures return simultaneously
- **Minimal Board:** Player A controls two Banisher Priests, each exiling a different creature. Day of Judgment destroys all creatures.
- **Action:** Both Banisher Priests leave the battlefield simultaneously.
- **Expected Result:** Both exiled creatures return to the battlefield simultaneously (not sequentially). ETB triggers from the returned creatures all see each other entering.
- **Phase:** Phase 7 (Triggered abilities)
- **Ticket:** NEW — Simultaneous "until" returns

### 610.4 — DEFERRED — Phase 9: Phasing
"Until" effects that cause phasing out. Full phasing system deferred.

### 610.4a — DEFERRED — Phase 9: Phasing
Phased-out permanent doesn't phase in via turn-based action; other effects may cause it.

### 610.4b — DEFERRED — Phase 9: Phasing
Spell/ability "until" phase-out with pre-event check.

### 610.4c — DEFERRED — Phase 9: Phasing
Triggered ability "until" phase-out with pre-event check.

### 610.4d — DEFERRED — Phase 9: Phasing
Simultaneous "until" phase-in effects.

### 610.5 — TESTABLE

Static abilities that grant abilities to spells as they are cast apply at the time the spell is put on the stack (601.2a).

> **Audit note:** Real-world cards this rule applies to primarily grant the *spell itself* a cost-reduction or casting mechanic (Convoke, Delve, Affinity). The original test (ETB grant to permanent) is noted below as a deferred composition test — it's correct in principle but involves the separate question of whether permanent spells carry abilities from their spell object into their permanent form. The primary tests use Convoke/Delve/Affinity-style grants.

**ATOM-610.5-001**
- **Rule:** 610.5 — Static ability grants Convoke to creature spells at cast time
- **Mechanism:** A permanent with "Creature spells you cast have convoke" grants convoke at the moment the spell is put on the stack (601.2a), allowing the player to tap creatures to pay for it during the cost-payment step
- **Minimal Board:** Player A controls a permanent with "Creature spells you cast have convoke." Player A casts a creature with cost {3}{G}. Player A controls three untapped creatures.
- **Action:** Player A casts the creature spell, tapping three creatures to pay {3} of the cost.
- **Expected Result:** Cast succeeds. The spell gains convoke at stack-placement time. Three creatures are tapped to pay 3 generic mana. Total mana paid: {G}.
- **Phase:** Phase 7 (Static grants to spells)
- **Ticket:** NEW — Static ability grants casting mechanic at cast time (610.5 + 601.2a)

**ATOM-610.5-002**
- **Rule:** 610.5 — Granting permanent destroyed before resolution: spell retains granted ability
- **Mechanism:** If the permanent granting convoke/delve is destroyed after the spell is on the stack but before resolution, the spell still has the granted ability (it was granted at cast time)
- **Minimal Board:** Player A controls a permanent with "Creature spells you cast have convoke." Player A casts a creature, tapping creatures for convoke. In response, opponent destroys the granting permanent.
- **Action:** The creature spell resolves.
- **Expected Result:** The spell still had convoke (granted at cast time). The cost was already paid. The spell resolves normally. The grant persists per 611.2a (no stated duration → indefinite).
- **Phase:** Phase 7 (Static grants to spells)
- **Ticket:** NEW — Cast-time grant persists after source destruction

> **Deferred composition test (noted):** A permanent with "Creature spells you cast have 'When this creature enters, draw a card'" — whether the ETB ability transfers from spell object to permanent object is a composition question involving 610.5 + 611.3d + object continuity. Worth testing when triggered abilities are implemented in Phase 7.

---

## 611. Continuous Effects

### 611.1 — PURE-DEF
A continuous effect modifies characteristics, control, or affects players/rules for a fixed or indefinite period. Names the concept — no independent mechanical consequence.

### 611.2 — PURE-DEF
A continuous effect may be generated by the resolution of a spell or ability. Introduces the category; sub-rules define the behaviors.

### 611.2a — TESTABLE

A continuous effect from a resolving spell/ability lasts as stated; if no duration, lasts until end of game.

**ATOM-611.2a-001**
- **Rule:** 611.2a — Spell-generated continuous effect with stated duration expires
- **Mechanism:** "Until end of turn" duration causes effect removal at cleanup
- **Minimal Board:** Player A controls Grizzly Bears (2/2). Giant Growth resolves targeting Bears: +3/+3 until end of turn.
- **Action:** Move to cleanup step.
- **Expected Result:** The +3/+3 effect expires. Bears return to 2/2.
- **Phase:** Phase 5 Layers (L07 — Giant Growth)
- **Ticket:** L07

**ATOM-611.2a-002**
- **Rule:** 611.2a — Spell-generated continuous effect with no stated duration lasts indefinitely
- **Mechanism:** A resolving ability that creates a continuous effect with no duration persists until end of game
- **Minimal Board:** A resolving ability says "Target creature becomes blue" (no duration stated).
- **Action:** Multiple turns pass.
- **Expected Result:** The creature remains blue indefinitely — the effect never expires.
- **Phase:** Phase 5 Layers (L02 — duration tracking)
- **Ticket:** L02

### 611.2b — TESTABLE

"For as long as" duration: if the duration condition was never true or ended before the effect would first apply, the effect does nothing.

**ATOM-611.2b-001**
- **Rule:** 611.2b — "For as long as" duration never starts → effect does nothing
- **Mechanism:** If the controlling condition is false before the effect begins, the effect never applies
- **Minimal Board:** Master Thief ("When this creature enters, gain control of target artifact for as long as you control this creature"). Player A loses control of Master Thief before the triggered ability resolves.
- **Action:** The triggered ability resolves.
- **Expected Result:** No control change occurs. The duration "as long as you control Master Thief" was over before the effect began.
- **Phase:** Phase 8 (D7 — "for as long as" duration failure)
- **Ticket:** NEW — "For as long as" duration pre-check (D7 in roadmap)

### 611.2c — TESTABLE (multi-clause)

Spell/ability continuous effects that modify characteristics or change controller lock in their affected set at creation. Effects that modify rules of the game do NOT lock in.

**ATOM-611.2c-001**
- **Rule:** 611.2c — Characteristic-modifying effect locks in affected set
- **Mechanism:** "All white creatures get +1/+1 until end of turn" locks in which creatures are white at resolution; creatures that become white later are NOT affected
- **Minimal Board:** Player A controls a white Bear (2/2) and a green Elf (1/1). A spell resolves: "All white creatures get +1/+1 until end of turn."
- **Action:** After the spell resolves, an effect turns the Elf white.
- **Expected Result:** Bear is 3/3 (was white at resolution, locked in). Elf is still 1/1 — even though it's now white, it wasn't white when the effect's set was locked in.
- **Phase:** Phase 5 Layers (L07 — 611.2c lock-in)
- **Ticket:** L07

**ATOM-611.2c-002**
- **Rule:** 611.2c — Game-rule-modifying effect does NOT lock in
- **Mechanism:** "Prevent all damage creatures would deal this turn" applies to all creatures, including those entering after the effect was created
- **Minimal Board:** A spell resolves: "Prevent all damage creatures would deal this turn." After resolution, a new creature enters the battlefield.
- **Action:** The new creature attempts to deal damage.
- **Expected Result:** Damage from the new creature IS prevented. The prevention effect modifies game rules, so it applies dynamically to all creatures, not just those present at resolution.
- **Phase:** Phase 6 (Prevention effects — game-rule modification)
- **Ticket:** NEW — Game-rule prevention effects apply dynamically (611.2c)

**ATOM-611.2c-003**
- **Rule:** 611.2c — Mixed effect: parts that modify characteristics lock in; parts that modify rules do not
- **Mechanism:** A single effect with both characteristic-modifying and rule-modifying parts determines affected sets independently
- **Minimal Board:** A single resolving effect says "White creatures get +1/+1 and can't be blocked this turn." Player A controls a white creature at resolution. After resolution, another creature becomes white.
- **Action:** Query both creatures.
- **Expected Result:** Original creature: gets +1/+1 (locked in) AND can't be blocked (rule-mod). New creature: does NOT get +1/+1 (not in locked set) but CAN'T be blocked (rule-mod applies dynamically).
- **Phase:** Phase 5 Layers + Phase 6
- **Ticket:** NEW — Mixed characteristic/rule effect independent set determination

### 611.2d — TESTABLE

Variable X in a continuous effect is determined once, on resolution.

**ATOM-611.2d-001**
- **Rule:** 611.2d — X value locked at resolution
- **Mechanism:** If a spell with X creates a continuous effect referencing X, the value is fixed at resolution
- **Minimal Board:** A spell resolves with X=3: "Target creature gets +X/+X until end of turn." The creature gets +3/+3.
- **Action:** After resolution, circumstances that would affect X change (irrelevant — X is already locked).
- **Expected Result:** The creature has exactly +3/+3. The value of X does not retroactively change.
- **Phase:** Phase 5 Layers (L07)
- **Ticket:** L07

### 611.2e — TESTABLE

A resolving spell/ability that both puts a permanent onto the battlefield AND creates a continuous effect stating the permanent "is [characteristic]" or "has [characteristic]" applies simultaneously with the permanent entering.

**ATOM-611.2e-001**
- **Rule:** 611.2e — "Is [characteristic]" applies simultaneously with entering
- **Mechanism:** Arbiter of the Ideal puts a card onto the battlefield and says "That permanent is an enchantment in addition to its other types." An ETB trigger for enchantments DOES trigger.
- **Minimal Board:** Player A controls a permanent that triggers "whenever an enchantment enters the battlefield." A spell resolves putting a creature card onto the battlefield with "That permanent is an enchantment in addition to its other types."
- **Action:** The creature enters the battlefield.
- **Expected Result:** The creature is an enchantment from the moment it enters. The "whenever an enchantment enters" trigger DOES fire. The creature does not enter as non-enchantment then become one.
- **Phase:** Phase 7 (Triggered abilities) + Phase 5 Layers
- **Ticket:** NEW — Simultaneous "is [type]" ETB characteristic (611.2e)

### 611.2f — DEFERRED — Phase 7
"Next spell" continuous effects don't begin immediately; they apply when the next appropriate spell is put on the stack. Requires triggered ability infrastructure (D8 in roadmap).

### 611.3 — PURE-DEF
A continuous effect may be generated by a static ability. Introduces the category.

### 611.3a — TESTABLE

Static ability continuous effects are NOT locked in — they apply dynamically.

**ATOM-611.3a-001**
- **Rule:** 611.3a — Static ability effect applies dynamically
- **Mechanism:** "All white creatures get +1/+1" from a static ability applies/unapplies as creatures change color
- **Minimal Board:** Player A controls Glorious Anthem variant "White creatures you control get +1/+1" and a 2/2 white creature.
- **Action:** An effect changes the creature's color to red.
- **Expected Result:** The creature loses the +1/+1 bonus and becomes 2/2 again. The static effect is not locked in — it continuously re-evaluates.
- **Phase:** Phase 5 Layers (L08 — static ability registration)
- **Ticket:** L08

### 611.3b — TESTABLE

The static ability effect applies at all times the generating permanent is on the battlefield (or the object is in the appropriate zone).

**ATOM-611.3b-001**
- **Rule:** 611.3b — Static effect ceases when source leaves battlefield
- **Mechanism:** When the permanent generating a static effect leaves the battlefield, the effect stops
- **Minimal Board:** Player A controls Glorious Anthem (creatures you control get +1/+1) and Grizzly Bears (effectively 3/3 due to Anthem).
- **Action:** Glorious Anthem is destroyed.
- **Expected Result:** Bears immediately revert to 2/2.
- **Phase:** Phase 5 Layers (L08)
- **Ticket:** L08

### 611.3c — TESTABLE

Continuous effects from static abilities that modify characteristics apply simultaneously with the permanent entering the battlefield (not after).

**ATOM-611.3c-001**
- **Rule:** 611.3c — Static effect applies as permanent enters, not after
- **Mechanism:** A 1/1 white creature enters the battlefield while "White creatures get +1/+1" is active. It enters as 2/2, not as 1/1 then becomes 2/2.
- **Minimal Board:** Player A controls Honor of the Pure ("White creatures you control get +1/+1"). Player A casts a 1/1 white creature.
- **Action:** Creature spell resolves and enters the battlefield.
- **Expected Result:** Creature is 2/2 from the moment it enters. A trigger that checks P/T "when a creature enters" sees it as 2/2. It does NOT enter as 1/1 and then change.
- **Phase:** Phase 5 Layers (L08)
- **Ticket:** L08

### 611.3d — TESTABLE

Static abilities that allow casting/playing a permanent spell or grant it an ability — the granted ability lasts as stated, or until end of game if no duration stated. Exception to 611.3a–b.

**ATOM-611.3d-001**
- **Rule:** 611.3d — Static grant of ability to permanent spell persists after source leaves
- **Mechanism:** If a static ability grants a permanent spell an ability and the granting source is later removed, the granted ability persists (per its stated duration or indefinitely)
- **Minimal Board:** Player A controls a permanent with "Creature spells you cast have lifelink." Player A casts a creature. While the creature spell is on the stack, the granting permanent is destroyed.
- **Action:** The creature spell resolves and enters the battlefield.
- **Expected Result:** The creature has lifelink. The grant was made when the spell was cast (610.5 / 601.2a) and persists indefinitely (no duration stated). The source leaving doesn't end the effect — this is an exception to 611.3a–b.
- **Phase:** Phase 7 (Triggered abilities) + Phase 5 Layers
- **Ticket:** NEW — Static ability grant persists per 611.3d

**ATOM-611.3d-002**
- **Rule:** 611.3d — Dream Devourer: foretold cards castable after source leaves
- **Mechanism:** Dream Devourer grants foretell to nonland cards in hand. Even if Dream Devourer leaves the battlefield, cards already foretold using its granted ability can still be cast from exile on later turns.
- **Minimal Board:** Player A controls Dream Devourer ({1}{B}, 0/3, "Each nonland card in your hand without foretell has foretell. Its foretell cost is equal to its mana cost reduced by {2}."). Player A foretells a card using the granted foretell ability. Dream Devourer is then destroyed.
- **Action:** On a later turn, Player A attempts to cast the foretold card from exile.
- **Expected Result:** The cast succeeds. The foretell was granted by Dream Devourer's static ability and used to exile the card. Per 611.3d, the granted foretell's effects (the card being in exile with permission to be cast) persist after Dream Devourer leaves. The card remains castable for its foretell cost.
- **Phase:** Phase 7 + Phase 5 Layers
- **Ticket:** NEW — Dream Devourer foretell grant persists (611.3d real-card example)

---

## 612. Text-Changing Effects

### 612.1 — PURE-DEF
Text-changing effects change an object's text. Generally affects rules text and/or type line. Defines the concept.

### 612.2 — TESTABLE

Text-changing effects only change words used correctly (color words as color, land types as land types, creature types as creature types). Cannot change card names even if they contain matching words.

**ATOM-612.2-001**
- **Rule:** 612.2 — Text change respects word context: color word as color
- **Mechanism:** A text-changing effect that changes "red" to "blue" should change color references in abilities but NOT the card name even if it contains "red"
- **Minimal Board:** A permanent named "Red Elemental Blast Engine" with ability "Counter target blue spell." A text-changing effect says "Change all instances of 'blue' to 'red' in target permanent's text."
- **Action:** Apply the text-changing effect.
- **Expected Result:** The ability becomes "Counter target red spell." The NAME remains "Red Elemental Blast Engine" — text-changing doesn't affect names, even if names contain color words.
- **Phase:** Phase 5 Layers (L12 — Layer 3 text-changing)
- **Ticket:** L12

### 612.2a — TESTABLE

Token names derived from creature types CAN be changed by text-changing effects.

**ATOM-612.2a-001**
- **Rule:** 612.2a — Text change affects creature-type-derived token names
- **Mechanism:** A spell that creates "a 1/1 white Soldier creature token" uses "Soldier" as both name and creature type. A text-changing effect changing "Soldier" to "Warrior" affects both.
- **Minimal Board:** A permanent with ability "Create a 1/1 white Soldier creature token." A text-changing effect changes "Soldier" to "Warrior" on that permanent.
- **Action:** Activate the token-creating ability.
- **Expected Result:** The token is created as a 1/1 white Warrior creature token named "Warrior."
- **Phase:** Phase 5 Layers (L12) + Phase 8 (tokens)
- **Ticket:** L12

### 612.3 — TESTABLE

Abilities added/removed by effects are NOT affected by text-changing effects on the same object.

**ATOM-612.3-001**
- **Rule:** 612.3 — Granted abilities immune to text change on object
- **Mechanism:** If a creature has been granted flying by an external effect, a text-changing effect on that creature cannot remove or alter the granted flying
- **Minimal Board:** Player A controls a creature that was granted "flying" by an Aura. A text-changing effect on the creature changes "flying" to "reach" in the creature's text.
- **Action:** Query the creature's abilities.
- **Expected Result:** The creature still has flying (granted by the Aura — the grant is not part of the creature's own text). The text-changing effect only modified the creature's own printed text, not externally granted abilities.
- **Phase:** Phase 5 Layers (L12)
- **Ticket:** L12

### 612.4 — PURE-DEF
A token's subtypes and rules text are defined by the creating spell/ability. Text-changing effects can change these. Definitional — no independent test beyond 612.2a.

### 612.5 — DEFERRED — Post-v1
Exchange of Words: exchange text boxes of two objects. Single-card mechanic, extremely niche.

### 612.6 — DEFERRED — Post-v1
Volrath's Shapeshifter: "full text" replacement. Single-card mechanic.

### 612.7 — DEFERRED — Post-v1
Spy Kit: "all names of nonlegendary creature cards." Single-card mechanic.

### 612.8 — TESTABLE

Some effects set the name of an object. The object loses all previous names and has only the specified name.

**ATOM-612.8-001**
- **Rule:** 612.8 — Effect that sets name replaces all existing names
- **Mechanism:** An effect that says "that permanent's name becomes [X]" causes the object to lose its original name
- **Minimal Board:** Player A controls a creature named "Grizzly Bears." An effect resolves: "Target permanent's name becomes 'Runeclaw Bear.'"
- **Action:** Query the creature's name.
- **Expected Result:** Name is "Runeclaw Bear." The original name "Grizzly Bears" is completely gone. Legend rule checks would use the new name.
- **Phase:** Phase 5 Layers (L12 — Layer 3)
- **Ticket:** L12

> **Audit note — name restoration:** If the name-setting effect expires (e.g., "until end of turn") or is generated by a static ability that leaves the battlefield, the engine must restore the original printed name. This is handled by the layer system’s re-evaluation (613.5): when the continuous effect ceases, the name reverts to the copiable/printed value because the effect is no longer applied. No special "undo" mechanism is needed — the layer system recalculates from scratch each time. **DESIGN: see session-6-audit-response.md for discussion.**

### 612.9 — OUT-OF-SCOPE
Name stickers. Un-set/sticker mechanic — permanently excluded.

### 612.10 — DEFERRED — Phase 8
Splice changes spell text by adding rules text. Requires splice keyword implementation.

---

## 613. Interaction of Continuous Effects (Layer System)

> **THIS IS THE MOST CRITICAL SECTION.** Every layer and sublayer defines a specific computation order. Getting the order wrong produces different observable results. Nearly every sub-rule here is TESTABLE.

### 613.1 — TESTABLE

Object characteristics start from printed values (or token/copy definition), then all continuous effects are applied in layer order 1–7.

**ATOM-613.1-001**
- **Rule:** 613.1 — Layer system applies effects in order 1 through 7
- **Mechanism:** A type-changing effect (L4) must be applied before a P/T-setting effect (L7b) that references the new type
- **Minimal Board:** Player A controls a noncreature artifact. An effect says "All artifacts are 2/2 creatures" (L4 type change + L7b P/T set).
- **Action:** Query effective characteristics of the artifact.
- **Expected Result:** The artifact is a 2/2 artifact creature. L4 adds creature type first, then L7b sets P/T to 2/2. If the order were reversed, the P/T set would apply to a non-creature (no effect or error).
- **Phase:** Phase 5 Layers (L04, L10)
- **Ticket:** L04, L10

### 613.1a — TESTABLE

Layer 1: Copy effects are applied.

**ATOM-613.1a-001**
- **Rule:** 613.1a — Layer 1 copy must apply before P/T modification; verified by observable outcome
- **Mechanism:** A Clone copies a creature. The critical test is that the copy establishes the **base P/T** in L1 before any L7 effects modify it. If copy happened after L7, the base would be wrong.
- **Minimal Board:** Player A controls a 5/5 creature with no abilities. A Clone enters as a copy of the 5/5. An effect says "Target creature gets -3/-3 until end of turn" targeting the Clone.
- **Action:** Query Clone's effective P/T.
- **Expected Result:** Clone is 2/2 (base 5/5 from L1 copy, then -3/-3 in L7c). If the copy had NOT applied in L1 (e.g., Clone stayed 0/0), the -3/-3 would result in -3/-3 (dead) or the base would be undefined. The 2/2 result proves L1 copy applied first.
- **Phase:** Phase 6 (Copy effects require replacement effect infrastructure — D1)
- **Ticket:** NEW — Layer 1 copy effect ordering (D1 in roadmap)

### 613.1b — TESTABLE

Layer 2: Control-changing effects are applied.

**ATOM-613.1b-001**
- **Rule:** 613.1b — Layer 2 control change before type/color/ability/PT
- **Mechanism:** A control-changing effect (Act of Treason) changes controller; subsequent layers use the new controller for "you control" filters
- **Minimal Board:** Player A controls Glorious Anthem ("Creatures you control get +1/+1"). Player B controls a 2/2 Bear. Player A casts Act of Treason on the Bear (gain control).
- **Action:** Query the Bear's effective P/T.
- **Expected Result:** Bear is 3/3. L2 changes controller to Player A first. Then L7c applies Anthem's +1/+1 because Player A now controls the Bear.
- **Phase:** Phase 5 Layers (L11 — Layer 2 control)
- **Ticket:** L11
- **Corpus correction (2026-08-23, Layer 2 phase):** this atom named **"Mind Snare"**, which is **not a Magic card** — Scryfall 404s on both `cards/named?fuzzy=` and an exact-name search. The name was invented in `plans/archive/implementation-plan-final.md` (§L17 describes it as "{3}{U}{U} Instant, GainControl with WhileTargetOnBattlefield", i.e. a re-costed Control Magic) and propagated into the corpus from there. Substituted **Act of Treason**, verbatim: "Gain control of target creature until end of turn. Untap that creature. It gains haste until end of turn." Its untap and haste clauses are inert for a P/T query, so the atom's claim is unchanged. Act of Treason is also what ATOM-613.6-002 names, which is fine — one card, two atoms, different assertions.

### 613.1c — TESTABLE

Layer 3: Text-changing effects are applied.

**ATOM-613.1c-001**
- **Rule:** 613.1c — Layer 3 text change before type/color/ability/PT
- **Mechanism:** A text-changing effect changes "Forest" to "Island" in a permanent's text before L4 type-changing effects process the modified text
- **Minimal Board:** Player A controls a permanent with "Forests you control have '{T}: Add {R}'." A text-changing effect changes "Forest" to "Island" in that permanent's text.
- **Action:** Query what the permanent's ability now references.
- **Expected Result:** After L3, the ability reads "Islands you control have '{T}: Add {R}'." L4/L6 then apply based on the modified text.
- **Phase:** Phase 5 Layers (L12 — Layer 3)
- **Ticket:** L12

### 613.1d — TESTABLE

Layer 4: Type-changing effects are applied.

**ATOM-613.1d-001**
- **Rule:** 613.1d — Layer 4 type change before color/ability/PT
- **Mechanism:** An effect that makes an artifact into a creature (L4) must happen before ability-granting (L6) and P/T-setting (L7b)
- **Minimal Board:** Player A controls Opalescence ("Each other non-Aura enchantment is a creature with P/T equal to its mana value"). Player A controls a 3-MV enchantment.
- **Action:** Query the enchantment's effective characteristics.
- **Expected Result:** L4 adds creature type. L7b sets P/T to 3/3 (MV=3). The enchantment is a 3/3 enchantment creature.
- **Phase:** Phase 5 Layers (L10, L19)
- **Ticket:** L10, L19

### 613.1e — TESTABLE

Layer 5: Color-changing effects are applied.

**ATOM-613.1e-001**
- **Rule:** 613.1e — Layer 5 color change before ability/PT
- **Mechanism:** A color-changing effect (L5) makes a creature white before an ability-conditional effect (L6/L7) that grants abilities/P/T to white creatures
- **Minimal Board:** Player A controls Honor of the Pure ("White creatures you control get +1/+1") and a 2/2 green creature. An effect changes the creature's color to white.
- **Action:** Query the creature's effective P/T.
- **Expected Result:** L5 changes creature to white. L7c then applies Honor's +1/+1. Creature is 3/3.
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

### 613.1f — TESTABLE

Layer 6: Ability-adding/removing effects and keyword counters are applied.

**ATOM-613.1f-001**
- **Rule:** 613.1f — Layer 6 ability changes before P/T
- **Mechanism:** Humility removes all abilities (L6) before P/T effects are applied (L7). A CDA ability removed in L6 doesn't contribute to L7a.
- **Minimal Board:** Player A controls Humility ("All creatures lose all abilities and have base P/T 1/1") and Tarmogoyf (CDA: P/T = graveyard card types).
- **Action:** Query Tarmogoyf's effective characteristics.
- **Expected Result:** L6 removes all abilities (including the CDA). L7a has no CDA to apply. L7b applies Humility's "base 1/1." Tarmogoyf is 1/1 with no abilities. (Note: timestamp does *not* arbitrate between L6 and L7b — CR 613.1 fixes layer order, and CR 613.7 orders effects only *within* a layer. On this board there is exactly one L7b effect, so nothing is timestamp-dependent at all. Timestamp order *within 7b* matters only once a second 7b effect exists — Opalescence, per the Humility rulings — and that needs CR 613.8.)
- **Phase:** Phase 5 Layers (L09, L19)
- **Ticket:** L09, L19

**ATOM-613.1f-002**
- **Rule:** 613.1f — Layer 6 keyword counters grant abilities
- **Mechanism:** A flying counter on a creature grants flying in L6, before P/T evaluation
- **Minimal Board:** Player A controls a creature with a flying keyword counter.
- **Action:** Query the creature's abilities.
- **Expected Result:** Creature has flying (from the keyword counter applied in L6).
- **Phase:** Phase 5 Layers (L09 — D10 keyword counters)
- **Ticket:** NEW — Keyword counters in Layer 6 (D10 in roadmap)

### 613.1g — TESTABLE

Layer 7: Power/toughness-changing effects are applied.

**ATOM-613.1g-001**
- **Rule:** 613.1g — Layer 7 is final layer for characteristic computation
- **Mechanism:** All P/T modifications happen in L7 after all other layers
- **Minimal Board:** Player A controls a 2/2 creature with a +1/+1 counter and Giant Growth (+3/+3 until EOT).
- **Action:** Query effective P/T.
- **Expected Result:** 2/2 base + 1/1 counter (L7c) + 3/3 Giant Growth (L7c) = 6/6.
- **Phase:** Phase 5 Layers (L06, L07)
- **Ticket:** L06, L07

### 613.2 — TESTABLE

Within Layer 1, apply effects in sublayer order (1a then 1b), and within each sublayer use timestamp order. Dependency may alter order.

**ATOM-613.2-001**
- **Rule:** 613.2 — Layer 1 sublayer ordering: 1a before 1b
- **Mechanism:** Copy effects (1a) apply before face-down modifications (1b)
- **Minimal Board:** A face-down creature (2/2 per 708.2) is also subject to a copy effect. The copy effect (1a) sets characteristics first, then the face-down overlay (1b) overrides them.
- **Action:** Query effective characteristics of the face-down creature.
- **Expected Result:** The creature has face-down characteristics (2/2, no name, no abilities) per 1b, applied after 1a.
- **Phase:** Phase 8 (Morph/face-down — D3)
- **Ticket:** NEW — Layer 1 sublayer ordering (D3 in roadmap)

### 613.2a — TESTABLE

Layer 1a: Copiable effects applied. Includes copy effects and "as enters" abilities that set P/T.

**ATOM-613.2a-001**
- **Rule:** 613.2a — Layer 1a copiable effects establish base characteristics
- **Mechanism:** A Clone entering as a copy of a 5/5 creature has base 5/5 after L1a
- **Minimal Board:** Player A controls a 5/5 creature. A Clone enters the battlefield as a copy of it.
- **Action:** Query Clone's base characteristics after L1a.
- **Expected Result:** Clone has the 5/5 creature's printed characteristics (name, types, abilities, P/T = 5/5) as its copiable values.
- **Phase:** Phase 6 (Copy effects — D1)
- **Ticket:** NEW — Layer 1a copiable effects (D1)

### 613.2b — DEFERRED — Phase 8: Morph
Layer 1b: Face-down spells/permanents get 708.2 characteristics. Requires face-down infrastructure (D3).

### 613.2c — TESTABLE

After Layer 1, the object's characteristics are its copiable values.

**ATOM-613.2c-001**
- **Rule:** 613.2c — Post-Layer 1 characteristics are copiable values
- **Mechanism:** A subsequent copy of a Clone (which is copying a Bear) should copy the Bear's characteristics, not the Clone's printed text
- **Minimal Board:** Clone A is a copy of Grizzly Bears (2/2). Clone B enters as a copy of Clone A.
- **Action:** Query Clone B's characteristics.
- **Expected Result:** Clone B is a 2/2 Grizzly Bears — it copied the copiable values established after L1 for Clone A, which are Bear's characteristics.
- **Phase:** Phase 6 (Copy effects — D1)
- **Ticket:** NEW — Copiable values are post-Layer-1 state (D1)

### 613.3 — TESTABLE

> **Audit note:** CDAs can appear in Layers 4 (type), 5 (color), 6 (ability), and 7a (P/T). Testing CDA-before-non-CDA in every layer would be over-exhaustive. We test the L5 (color) case here as the representative example. The L7a (P/T) case is tested under 613.4a. Other layers are covered implicitly—the engine should use the same CDA-first ordering path regardless of layer.

Within Layers 2–6, CDAs apply first, then all other effects in timestamp order. Dependency may override.

**ATOM-613.3-001**
- **Rule:** 613.3 — CDAs apply before non-CDAs in Layers 2–6
- **Mechanism:** In Layer 5, a CDA that defines a creature's color applies before a non-CDA color-changing effect
- **Minimal Board:** A creature has a CDA "This creature's color is the color of the most recent spell cast" (currently red) and a non-CDA effect "This creature is blue until end of turn."
- **Action:** Query the creature's color.
- **Expected Result:** CDA applies first (red), then non-CDA overrides (blue). Final color: blue. If the order were reversed, the CDA would override the spell effect — wrong result.
- **Phase:** Phase 5 Layers (L04, L10)
- **Ticket:** L04

### 613.4 — PURE-DEF
Within Layer 7, effects are applied in sublayer order (7a–7d), timestamp within sublayers. Introduces sublayer structure — specific sublayers are testable individually.

### 613.4a — TESTABLE

Layer 7a: CDA P/T effects apply first.

**ATOM-613.4a-001**
- **Rule:** 613.4a — Layer 7a CDAs set P/T before other effects
- **Mechanism:** Tarmogoyf's CDA (P/T = graveyard card types) applies in 7a, before any L7b/L7c/L7d effects
- **Minimal Board:** Player A controls Tarmogoyf. Graveyard has 3 card types. Giant Growth is active on Tarmogoyf.
- **Action:** Query Tarmogoyf's effective P/T.
- **Expected Result:** L7a: CDA sets P/T to 3/4 (3 types → P=3, T=3+1). L7c: Giant Growth adds +3/+3. Final: 6/7.
- **Phase:** Phase 5 Layers (L04, L17)
- **Ticket:** L04, L17

### 613.4b — TESTABLE

Layer 7b: Effects that SET P/T to specific values (overriding base).

**ATOM-613.4b-001**
- **Rule:** 613.4b — Layer 7b P/T setting overrides L7a CDA
- **Mechanism:** "Target creature becomes 0/1 until end of turn" (L7b) overrides the CDA base from L7a
- **Minimal Board:** Player A controls Tarmogoyf (CDA: 3/4 from 3 card types). An effect resolves: "Target creature's base P/T becomes 0/1 until end of turn."
- **Action:** Query Tarmogoyf's effective P/T.
- **Expected Result:** L7a: CDA → 3/4. L7b: Set to 0/1. L7c: (no modifiers). Final: 0/1. The L7b override completely replaces the L7a base.
- **Phase:** Phase 5 Layers (L04, L07)
- **Ticket:** L04, L07

**ATOM-613.4b-002**
- **Rule:** 613.4b — Layer 7b with subsequent L7c modifiers
- **Mechanism:** After L7b sets base, L7c modifiers still apply on top
- **Minimal Board:** Player A controls a Gray Ogre (2/2). An effect sets it to 0/1 (L7b). A +1/+1 counter is on it. Giant Growth (+3/+3) is active.
- **Action:** Query effective P/T.
- **Expected Result:** L7b: 0/1. L7c: +1/+1 (counter) + +3/+3 (Growth) = 4/5. This matches the CR 613.5 example.
- **Phase:** Phase 5 Layers (L04, L06, L07)
- **Ticket:** L04, L06, L07

### 613.4c — TESTABLE

Layer 7c: Effects and counters that MODIFY P/T (add/subtract, not set).

**ATOM-613.4c-001**
- **Rule:** 613.4c — Layer 7c counters and effects stack additively
- **Mechanism:** +1/+1 counter + Glorious Anthem (+1/+1) + Giant Growth (+3/+3) all apply in 7c
- **Minimal Board:** Player A controls a 2/2 creature with one +1/+1 counter, Glorious Anthem active, and Giant Growth active.
- **Action:** Query effective P/T.
- **Expected Result:** 2/2 base + 1/1 (counter) + 1/1 (Anthem) + 3/3 (Growth) = 7/7.
- **Phase:** Phase 5 Layers (L06, L07, L08)
- **Ticket:** L06, L07, L08

### 613.4d — TESTABLE (multi-clause, with Examples)

Layer 7d: P/T switch effects. Switch takes current P and T values and swaps them. Subsequent switches cancel. New modifiers after a switch apply to the "unswitched" side then get re-switched.

**ATOM-613.4d-001**
- **Rule:** 613.4d — Basic P/T switch
- **Mechanism:** A 1/3 creature given +0/+1 (to 1/4), then switched → 4/1
- **Minimal Board:** Player A controls a 1/3 creature. An effect gives it +0/+1 (L7c). Then a switch effect applies (L7d).
- **Action:** Query effective P/T.
- **Expected Result:** After L7c: 1/4. After L7d switch: 4/1.
- **Phase:** Phase 5 Layers (L04 — SwitchPT)
- **Ticket:** L04

**ATOM-613.4d-002**
- **Rule:** 613.4d — Switch with subsequent modifier removal
- **Mechanism:** Per CR example: 1/3 + 0/+1 → 1/4, switched → 4/1. If the +0/+1 ends, becomes 3/1 (not 1/3)
- **Minimal Board:** Player A controls a 1/3 creature. An effect gives +0/+1 until EOT (L7c). A switch effect applies (L7d). Then the +0/+1 expires.
- **Action:** Query effective P/T after +0/+1 expires but switch persists.
- **Expected Result:** Base 1/3, no +0/+1, switch active → 3/1. The switch swaps the current values (1/3 → 3/1), not the previously-switched values.
- **Phase:** Phase 5 Layers (L04)
- **Ticket:** L04

**ATOM-613.4d-003**
- **Rule:** 613.4d — Double switch cancels out
- **Mechanism:** Per CR example: two switches cancel each other
- **Minimal Board:** Player A controls a 1/3 creature given +0/+1 (1/4). Two switch effects apply.
- **Action:** Query effective P/T.
- **Expected Result:** 1/4 → switch → 4/1 → switch → 1/4. Two switches cancel.
- **Phase:** Phase 5 Layers (L04)
- **Ticket:** L04

**ATOM-613.4d-004**
- **Rule:** 613.4d — Modifier added AFTER switch applies to unswitched side then re-switches
- **Mechanism:** Per CR example: 1/3 +0/+1 = 1/4, switched = 4/1, then +5/+0 → unswitched would be 6/4, so actual = 4/6
- **Minimal Board:** Player A controls a 1/3 creature. Effect 1: +0/+1 (L7c). Effect 2: switch (L7d). Effect 3: +5/+0 (L7c, but with later timestamp than switch).
- **Action:** Query effective P/T.
- **Expected Result:** The +5/+0 is applied in L7c (before L7d switch). Unswitched: 1+5/3+1 = 6/4. After switch: 4/6.
- **Phase:** Phase 5 Layers (L04)
- **Ticket:** L04

### 613.5 — TESTABLE (with Examples)

The layer system is continually and automatically re-evaluated. Changes are instantaneous.

**ATOM-613.5-001**
- **Rule:** 613.5 — Color change in L5 immediately triggers re-evaluation of L7 effects
- **Mechanism:** Per CR example: Honor of the Pure gives +1/+1 to white creatures. A 2/2 black creature becomes white (L5) → immediately gets +1/+1 (L7c) → 3/3. If changed to red (L5) → loses +1/+1 → back to 2/2.
- **Minimal Board:** Player A controls Honor of the Pure and a 2/2 black creature.
- **Action:** (1) An effect turns the creature white. (2) Later, an effect turns it red.
- **Expected Result:** (1) Creature becomes 3/3. (2) Creature becomes 2/2. Changes are instantaneous — no "pending" state.
- **Phase:** Phase 5 Layers (L08, L10)
- **Ticket:** L08, L10

**ATOM-613.5-002**
- **Rule:** 613.5 — Complex multi-layer interaction (CR example: Gray Ogre)
- **Mechanism:** Per CR example: 2/2 creature, +1/+1 counter (L7c) → 3/3, +4/+4 spell (L7c) → 7/7, +0/+2 enchantment (L7c) → 7/9, then "becomes 0/1" (L7b) → 5/8 (0/1 base from L7b, then all L7c: +4/+4 + +0/+2 + +1/+1 = +5/+4 → 5/5... wait, 0+5/1+4 = 5/5? No: 0+4+0+1 / 1+4+2+1 = 5/8). Yes: 0+4+0+1=5 / 1+4+2+1=8.
- **Minimal Board:** Gray Ogre (2/2), one +1/+1 counter, "+4/+4 until EOT" effect, enchantment "+0/+2" on battlefield, "becomes 0/1 until EOT" effect.
- **Action:** Query effective P/T.
- **Expected Result:** L7b: base becomes 0/1 (overrides printed 2/2). L7c: +1/+1 counter + +4/+4 spell + +0/+2 enchantment = +5/+7. Total: 5/8.
- **Phase:** Phase 5 Layers (L04, L06, L07, L08)
- **Ticket:** L04, L06, L07, L08

### 613.6 — TESTABLE (with Examples)

Multi-layer effects: each part applies in its appropriate layer. Once an effect starts applying to a set of objects, it continues in later layers even if the generating ability is removed. "Locked set across layers."

**ATOM-613.6-001**
- **Rule:** 613.6 — Effect applies in multiple layers to same set
- **Mechanism:** Per CR example: "All noncreature artifacts become 2/2 artifact creatures until end of turn." L4 adds creature type to noncreature artifacts. L7b sets P/T to 2/2 on the SAME set — even though they are now creatures (no longer "noncreature artifacts").
- **Minimal Board:** Player A controls three artifacts: two noncreature artifacts and one artifact creature.
- **Action:** An effect resolves: "All noncreature artifacts become 2/2 artifact creatures until end of turn."
- **Expected Result:** The two noncreature artifacts become 2/2 artifact creatures. The artifact creature is unaffected (wasn't a noncreature artifact when L4 determined the set). In L7b, the set is the same two permanents even though they're now creatures.
- **Phase:** Phase 5 Layers (L05 — locked-in target sets)
- **Ticket:** L05

**ATOM-613.6-002**
- **Rule:** 613.6 — Act of Treason: control in L2, haste in L6
- **Mechanism:** Per CR example: "Gain control... Untap... It gains haste until end of turn." Control applied in L2, haste applied in L6.
- **Minimal Board:** Player B controls a creature. Player A casts Act of Treason targeting it.
- **Action:** Act of Treason resolves.
- **Expected Result:** L2: Player A gains control. L6: The creature gains haste. Both parts apply to the same creature.
- **Phase:** Phase 5 Layers (L09, L11)
- **Ticket:** L09, L11

**ATOM-613.6-003**
- **Rule:** 613.6 — Multi-layer effect persists even if generating ability removed
- **Mechanism:** If an effect applies in L4 and L7b, and the generating ability is removed between those layers (e.g., by another L6 effect), the L7b part still applies
- **Minimal Board:** Effect A: "Enchantments are 1/1 creatures" (L4 + L7b). Effect B: "Remove all abilities from target permanent" applied to the source of Effect A between L4 and L7b computation.
- **Action:** Query an enchantment's characteristics.
- **Expected Result:** The enchantment is a 1/1 creature. Even though the source's ability was removed, the effect that already started applying in L4 continues to apply in L7b per 613.6.
- **Phase:** Phase 5 Layers (L05)
- **Ticket:** L05

> **Audit note — Svogthos composition test:** Svogthos, the Restless Tomb ("{3}{B}{G}: Until end of turn, Svogthos becomes a black and green Plant Zombie creature with 'This creature’s power and toughness are each equal to the number of creature cards in your graveyard.' It's still a land.") is a good integration test for 613.6 because it creates a multi-layer effect (L4 type change, L5 color set, L7a CDA). See COMP-613-SVOGTHOS-001 in Composition Tests.

### 613.7 — TESTABLE

Timestamp system: earlier timestamp applies first within a layer/sublayer.

**ATOM-613.7-001**
- **Rule:** 613.7 — Timestamp ordering observable via conflicting set-PT effects in L7b
- **Mechanism:** Two "base P/T becomes X/Y" effects in L7b — the later timestamp wins because it overwrites the earlier. This directly verifies timestamp ordering.
- **Minimal Board:** Effect A (timestamp T1): "Target creature's base P/T becomes 5/5 until end of turn." Effect B (timestamp T2, T2 > T1): "Target creature's base P/T becomes 0/2 until end of turn." Both target the same 3/3 creature.
- **Action:** Query the creature's effective P/T.
- **Expected Result:** L7b: T1 sets 5/5, then T2 sets 0/2. Final base: 0/2. If timestamps were reversed, result would be 5/5. This test verifies the later timestamp wins.
- **Phase:** Phase 5 Layers (L03)
- **Ticket:** L03

> **Audit note:** Previous ATOM-613.7-002 was subsumed into -001 — both tested the same mechanism (conflicting L7b set-PT, later timestamp wins) with trivially different numbers.

### 613.7a — TESTABLE

Static ability timestamp = object's timestamp or the timestamp of the effect that created the ability, whichever is later. If the ability-creating effect has the later timestamp and the object gets a new timestamp, all static effects get new timestamps but relative order is preserved.

**ATOM-613.7a-001**
- **Rule:** 613.7a — Static ability uses later of object vs. granting effect timestamp
- **Mechanism:** Per CR example: Rune of Flight (Aura, timestamp T_rune) grants "Equipped creature has flying" to Colossus Hammer (timestamp T_hammer). If T_rune > T_hammer, the granted flying uses T_rune's timestamp.
- **Minimal Board:** Colossus Hammer (T1) has "Equipped creature gets +10/+10 and loses flying." Rune of Flight (T2, T2 > T1) grants "Equipped creature has flying" to the Hammer. Both are attached to a creature.
- **Action:** Query: does the equipped creature have flying?
- **Expected Result:** "Loses flying" (T1) applies before "has flying" (T2) in L6. Creature HAS flying — the later timestamp wins.
- **Phase:** Phase 5 Layers (D5 — deferred timestamp)
- **Ticket:** NEW — Static ability timestamp = later of object vs grant (D5)

### 613.7b — TESTABLE

Spell/ability continuous effects get timestamp at creation time.

**ATOM-613.7b-001**
- **Rule:** 613.7b — Spell effect timestamp set at creation
- **Mechanism:** Giant Growth (cast first) has earlier timestamp than a later pump spell. Both apply in L7c by timestamp order.
- **Minimal Board:** Player A casts Giant Growth (+3/+3) on a 2/2 creature. Later, Player A casts another spell giving -2/-2 on the same creature.
- **Action:** Query effective P/T.
- **Expected Result:** L7c: +3/+3 (earlier timestamp) then -2/-2 (later timestamp). Net: +1/+1. Creature is 3/3. (Additive, so order doesn't matter for final result, but timestamp is tracked.)
- **Phase:** Phase 5 Layers (L07)
- **Ticket:** L07

### 613.7c — TESTABLE

Counters get timestamps. If a counter of the same kind already exists, all counters of that kind get the new timestamp.

**ATOM-613.7c-001**
- **Rule:** 613.7c — Counter timestamp updates when new counter of same kind added
- **Mechanism:** Adding a second +1/+1 counter updates the timestamp of all +1/+1 counters on that object
- **Minimal Board:** A creature has one +1/+1 counter (timestamp T1). A second +1/+1 counter is added (timestamp T2).
- **Action:** Query the timestamp of the +1/+1 counters.
- **Expected Result:** All +1/+1 counters now have timestamp T2. This affects ordering relative to other L7c effects.
- **Phase:** Backlog — layers timestamp debt (was Phase 5 Layers)
- **Ticket:** NEW — Counter timestamps within L7c (D6)

### 613.7d — TESTABLE

Objects get timestamp when entering a zone.

**ATOM-613.7d-001**
- **Rule:** 613.7d — Object timestamp set on zone entry
- **Mechanism:** A permanent that entered the battlefield earlier has an earlier timestamp than one that entered later
- **Minimal Board:** Anthem A enters the battlefield at time T1. Anthem B enters at time T2 (T2 > T1). Both have the same static effect in L7b: "Creatures are 1/1."
- **Action:** Query a creature's base P/T.
- **Expected Result:** L7b: Anthem A applies (1/1) then Anthem B applies (1/1). Anthem B's timestamp is later, so it "wins" — but since they set the same value, the result is 1/1 either way. The test verifies timestamps are distinct.
- **Phase:** Phase 5 Layers (L03)
- **Ticket:** L03

### 613.7e — TESTABLE

Aura/Equipment/Fortification gets new timestamp on each attach.

**ATOM-613.7e-001**
- **Rule:** 613.7e — Equipment re-timestamp on attach
- **Mechanism:** Re-attaching an Equipment gives it a new timestamp, which may change ordering of its effects relative to others in the same layer
- **Minimal Board:** Equipment A (timestamp T1) gives equipped creature "loses flying." Aura B (timestamp T2, T2 > T1) gives enchanted creature flying. Both on creature X. Equipment A is moved to creature Y, getting new timestamp T3 (T3 > T2), then moved back to creature X.
- **Action:** Query: does creature X have flying?
- **Expected Result:** After re-attach, Equipment A has timestamp T3 > T2. In L6: Aura B (T2, "has flying") applies first, then Equipment A (T3, "loses flying"). Creature does NOT have flying — the re-attach changed the outcome.
- **Phase:** Backlog — layers timestamp debt (was Phase 5 Layers)
- **Ticket:** NEW — Aura/Equipment re-timestamp on attach (D4)

### 613.7f — DEFERRED — Phase 8: Morph/Transform
Permanent receives new timestamp on face-up/face-down.

### 613.7g — DEFERRED — Phase 8: Transform
Double-faced permanent receives new timestamp on transform/convert.

### 613.7h — OUT-OF-SCOPE
Plane/phenomenon/scheme cards. Planechase/Archenemy mechanics.

### 613.7i — DEFERRED — Post-v1 (Vanguard stretch goal)
Vanguard card timestamp.

### 613.7j — OUT-OF-SCOPE
Conspiracy card timestamp. Conspiracy draft mechanic.

### 613.7k — OUT-OF-SCOPE
Sticker timestamp. Un-set mechanic.

### 613.7m — TESTABLE

Simultaneous timestamp assignment uses APNAP order.

**ATOM-613.7m-001**
- **Rule:** 613.7m — APNAP ordering for simultaneous timestamps
- **Mechanism:** Two permanents entering simultaneously get relative timestamps in APNAP order (active player's permanents first)
- **Minimal Board:** Active player's creature and non-active player's creature enter the battlefield simultaneously (e.g., from a mass zone change). Both have conflicting L7b effects.
- **Action:** Query the relative timestamps.
- **Expected Result:** Active player's permanent has earlier timestamp. Non-active player's permanent has later timestamp. In a layer conflict, the non-active player's effect (later timestamp) "wins."
- **Phase:** Phase 5 Layers (L03 — APNAP sub-ordering)
- **Ticket:** L03

### 613.7n — TESTABLE

Simultaneous static ability + resolving effect: static ability gets earlier timestamp.

**ATOM-613.7n-001**
- **Rule:** 613.7n — Static ability timestamp < resolving effect timestamp when simultaneous
- **Mechanism:** Per 611.2e: a spell puts a permanent onto the battlefield and sets it to be a color. The permanent's own static ability gets earlier timestamp than the spell's continuous effect.
- **Minimal Board:** A spell puts a creature onto the battlefield and says "That creature is blue." The creature has a static ability "This creature is green."
- **Action:** Query the creature's color.
- **Expected Result:** Static ability (green, earlier timestamp) applies first in L5. Then the spell's effect (blue, later timestamp) applies. Final color: blue. The spell's effect wins because it has the later timestamp, but the static ability was correctly ordered first.
- **Phase:** Phase 5 Layers (L08 — PC10)
- **Ticket:** L08

### 613.8 — TESTABLE

Dependency system overrides timestamp within a layer/sublayer.

**ATOM-613.8-001**
- **Rule:** 613.8 — Dependency overrides timestamp order
- **Mechanism:** If effect A depends on effect B, B is applied first regardless of timestamp
- **Minimal Board:** Two continuous effects in L6 on the same creature: Effect A (timestamp T1, earlier): "Target creature gains flying." Effect B (timestamp T2, later): "Target creature loses all abilities." B does not depend on A (removing abilities doesn't care about which abilities exist). A depends on B: applying B changes A's effect (if the creature has no abilities, granting flying adds one; vs if it already has abilities, it adds one more — actually, this doesn't change A's text/existence/what-it-applies-to). Better example: Effect A: "Enchanted creature has all activated abilities of other creatures you control" (granting abilities). Effect B: "Enchanted creature loses all abilities." A depends on B because applying B first (remove all) changes the outcome of A (A re-grants abilities on a blank slate). B does not depend on A.
- **Action:** Determine application order.
- **Expected Result:** A depends on B → B applied first (loses all abilities), then A (gains abilities from other creatures). Final: creature has the copied activated abilities. Without dependency (pure timestamp), A (T1) would apply first (gain abilities), then B (T2) would remove them all. Dependency overrides timestamp to produce the correct result.
- **Phase:** Phase 5 Layers (L14)
- **Ticket:** L14

### 613.8a — TESTABLE

Definition of dependency: (a) same layer/sublayer; (b) applying the other would change text/existence/applies-to/effect of the first; (c) neither is CDA or both are CDAs.

**ATOM-613.8a-001**
- **Rule:** 613.8a — Blood Moon + Urborg: dependency analysis and correct outcome
- **Mechanism:** Blood Moon: "Nonbasic lands are Mountains" (L4, SetLandType). Urborg: "Each land is a Swamp in addition to its other types" (L4, AddSubtypes). **Urborg is itself a nonbasic land.** Applying Blood Moon changes Urborg into a Mountain, removing its printed ability — this changes the **existence** of Urborg's effect (dependency condition b). Therefore Urborg's effect depends on Blood Moon's effect. Blood Moon does NOT depend on Urborg (Urborg adding Swamp doesn't change what Blood Moon applies to — "nonbasic" is a supertype check, not a subtype check). Since Urborg depends on Blood Moon: Blood Moon applies first regardless of timestamps.
- **Minimal Board:** Blood Moon and Urborg, Tomb of Yawgmoth on the battlefield. Player A controls a nonbasic land (e.g., Breeding Pool). Timestamps irrelevant due to dependency.
- **Action:** Query Breeding Pool's subtypes and mana abilities.
- **Expected Result:** Blood Moon applies first: Breeding Pool becomes a Mountain (loses all subtypes, printed abilities, gains Mountain + {T}: Add {R}). Urborg's effect is then applied — BUT Blood Moon also makes Urborg itself into a Mountain, removing Urborg's static ability. Urborg's effect ceases to exist. Therefore Urborg's "add Swamp" never applies. **Breeding Pool can only tap for {R}.** All nonbasic lands are Mountains. Urborg is a Mountain (not a Swamp).
- **Phase:** Phase 5 Layers (L14, L17)
- **Ticket:** L14, L17

> **Audit note — Blood Moon / rule 305.7:** Blood Moon's interaction with rule 305.7 (basic land types grant intrinsic mana abilities and remove other abilities) has been flagged by top MTG judges as a potentially unintuitive carveout. The current test follows today's rules. If 305.7 is ever revised to align more intuitively with the layer system, this test would need updating.

**ATOM-613.8a-002**
- **Rule:** 613.8a — Dependency condition (b): applying B changes existence of A's effect
- **Mechanism:** A nonbasic land has a static ability generating a continuous effect in L4. Blood Moon makes it a Mountain, removing its printed abilities, which means the land's continuous effect ceases to exist. This is dependency condition (b).
- **Minimal Board:** Player A controls a nonbasic land with ability "Other lands you control are Forests in addition to their other types" and Blood Moon.
- **Action:** Query: does the land's static ability effect apply?
- **Expected Result:** Blood Moon makes the land a Mountain, removing its printed abilities. The "other lands are Forests" effect ceases to exist. Dependency condition (b) is satisfied: applying Blood Moon changed the existence of the land's effect.
- **Phase:** Phase 5 Layers (L14)
- **Ticket:** L14

**ATOM-613.8a-003**
- **Rule:** 613.8a — CDA guard: one CDA + one non-CDA → independent
- **Mechanism:** A CDA and a non-CDA in the same layer are always independent (condition c fails)
- **Minimal Board:** A creature with a CDA "This creature's power is equal to cards in your hand" and a non-CDA effect "This creature gets +2/+2."
- **Action:** Determine dependency between the two effects.
- **Expected Result:** They are independent. The CDA applies first (per 613.3/613.4a rule), not because of dependency but because CDAs always go first. The dependency system doesn't alter this.
- **Phase:** Phase 5 Layers (L14)
- **Ticket:** L14

### 613.8b — TESTABLE

Dependent effect waits for dependencies. Multiple dependent effects applied in timestamp order. Circular dependency → ignore dependency, use timestamp.

**ATOM-613.8b-001**
- **Rule:** 613.8b — Circular dependency falls back to timestamp
- **Mechanism:** If A depends on B and B depends on A (circular), dependency is ignored and timestamp order is used
- **Minimal Board:** Two effects in the same sublayer that are mutually dependent (each changes what the other applies to). Effect A (timestamp T1), Effect B (timestamp T2, T2 > T1).
- **Action:** Determine application order.
- **Expected Result:** Circular dependency detected → ignored. Effects applied in timestamp order: A (T1) first, then B (T2).
- **Phase:** Phase 5 Layers (L14)
- **Ticket:** L14

> **Audit note — dependency loop detection:** The engine needs a graph-based algorithm to detect circular dependencies. Topological sort on the dependency DAG naturally handles this: if the sort detects a cycle (remaining nodes with no in-edges missing while nodes remain), those nodes fall back to timestamp order. We previously discussed Kahn’s algorithm for this. See session-6-audit-response.md for design discussion.

### 613.8c — TESTABLE

After each effect is applied, remaining effects' dependency ordering is re-evaluated.

**ATOM-613.8c-001**
- **Rule:** 613.8c — Dependency re-evaluation after each application
- **Mechanism:** After applying effect X, effect Y may become dependent on or independent of effect Z (which hasn't been applied yet)
- **Minimal Board:** Three effects in L4: A (T1), B (T2), C (T3). Initially, B depends on A (so A goes first). After A is applied, C becomes dependent on B (which wasn't true before A was applied). So order is: A → B → C.
- **Action:** Apply effects with iterative dependency re-evaluation.
- **Expected Result:** A applies first (B depends on it). After A, re-evaluate: C now depends on B. B applies next. Then C. Without re-evaluation, C might have applied before B.
- **Phase:** Phase 5 Layers (L14)
- **Ticket:** L14

### 613.9 — TESTABLE (with Examples)

One continuous effect can override another. "Has flying" vs "loses flying" — timestamp order determines winner.

**ATOM-613.9-001**
- **Rule:** 613.9 — Conflicting ability effects: later timestamp wins
- **Mechanism:** Per CR example: "Enchanted creature has flying" (Aura A, T1) vs "Enchanted creature loses flying" (Aura B, T2). Later timestamp wins.
- **Minimal Board:** A creature has two Auras: Aura A (T1) giving flying, Aura B (T2, T2 > T1) removing flying.
- **Action:** Query: does the creature have flying?
- **Expected Result:** L6: T1 grants flying, then T2 removes flying. Creature does NOT have flying.
- **Phase:** Phase 5 Layers (L09)
- **Ticket:** L09

**ATOM-613.9-002**
- **Rule:** 613.9 — Color change causes downstream effect to apply
- **Mechanism:** Per CR example: "White creatures get +1/+1" and "Enchanted creature is white." The enchanted creature gets +1/+1 regardless of previous color.
- **Minimal Board:** Player A controls Honor of the Pure ("White creatures you control get +1/+1") and a red creature. An Aura makes the creature white.
- **Action:** Query the creature's P/T.
- **Expected Result:** L5: creature becomes white. L7c: Honor's +1/+1 applies (creature is now white). This shows one effect (color) feeding into another (P/T).
- **Phase:** Phase 5 Layers (L08, L10)
- **Ticket:** L08, L10

### 613.10 — TESTABLE

Continuous effects on players (e.g., protection from red) applied in timestamp order after object characteristics are determined.

**ATOM-613.10-001**
- **Rule:** 613.10 — Player-affecting continuous effects applied after object characteristics
- **Mechanism:** "You have protection from red" applies after all object layers are computed
- **Minimal Board:** Player A has a continuous effect granting "protection from red." A red spell targets Player A.
- **Action:** Check targeting legality.
- **Expected Result:** The protection effect is computed after all object characteristics are determined. The spell is red (determined in layers). Player A has protection from red (applied post-layers). Targeting is illegal.
- **Phase:** Phase 5 Layers (L15 — player action restrictions)
- **Ticket:** L15

### 613.11 — TESTABLE

Game-rule-modifying continuous effects applied after all other continuous effects. Cost-modification effects follow 601.2f order.

> **Audit note:** This rule is closely linked to the 611.2c design question about how the engine discriminates between characteristic-modifying effects and game-rule-modifying effects. See session-6-audit-response.md for design discussion.

**ATOM-613.11-001**
- **Rule:** 613.11 — Game-rule-modifying effects apply AFTER all L1–L7 characteristic effects
- **Mechanism:** An effect that modifies rules of the game (e.g., "creatures can't attack") must be applied after all layer computations are complete, because it needs to know the final characteristics of objects
- **Minimal Board:** Player A controls a permanent with "Red creatures can't attack." Player A controls a creature that is green (printed) but has a L5 color-changing effect making it red.
- **Action:** Player A attempts to attack with the creature.
- **Expected Result:** Attack is illegal. The creature's color is determined in L5 (red). Then the game-rule-modifying effect ("red creatures can't attack") is applied per 613.11 (post-layer). It sees the creature is red and prevents the attack.
- **Phase:** Phase 5 Layers (L15 — game-rule modification)
- **Ticket:** L15

**ATOM-613.11-002**
- **Rule:** 613.11 — Cost modification effects follow 601.2f pipeline
- **Mechanism:** Increases before reductions before Trinisphere floor
- **Minimal Board:** Player A controls Thalia (noncreature spells cost {1} more) and Electromancer (instant/sorcery spells cost {1} less). Player A casts Lightning Bolt (cost {R}).
- **Action:** Determine total cost.
- **Expected Result:** Base: {R}. Increase (Thalia): {1}{R}. Reduction (Electromancer): {R}. Net: {R}. Per 601.2f, increases apply before reductions.
- **Phase:** Phase 5 Layers (L15 — cost modification)
- **Ticket:** L15

---

## 614. Replacement Effects

### 614.1 — PURE-DEF
Replacement effects watch for events and replace them. Introduces the concept. Sub-rules define specific patterns.

### 614.1a — BOUNDARY-DEF

"Instead" keyword identifies replacement effects.

**BOUNDARY-DEF-614.1a-001**
- **Rule:** 614.1a — "Instead" identifies a replacement effect
- **Mechanism:** Engine must classify effects using "instead" as replacement effects (not triggered abilities, not one-shots)
- **In-set member:** "If this creature would die, instead exile it" — contains "instead," IS a replacement effect
- **Out-of-set member:** "When this creature dies, exile it" — contains "when," is a triggered ability, NOT a replacement effect
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Replacement effect classification by "instead" keyword

### 614.1b — BOUNDARY-DEF

"Skip" keyword identifies replacement effects.

**BOUNDARY-DEF-614.1b-001**
- **Rule:** 614.1b — "Skip" identifies a replacement effect
- **Mechanism:** "Skip your draw step" is a replacement effect that replaces the draw step with nothing
- **In-set member:** "Skip your next draw step" — IS a replacement effect
- **Out-of-set member:** "At the beginning of your draw step, draw an additional card" — is a triggered ability, NOT a replacement effect
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — "Skip" as replacement effect keyword

### 614.1c — BOUNDARY-DEF

"Enters with," "As [this] enters," "enters as" are replacement effects.

**BOUNDARY-DEF-614.1c-001**
- **Rule:** 614.1c — ETB modification patterns are replacement effects
- **Mechanism:** Engine must treat "enters with" / "as enters" / "enters as" as replacement effects that modify the ETB event
- **In-set member:** "This creature enters with two +1/+1 counters" — IS a replacement effect
- **Out-of-set member:** "When this creature enters, put two +1/+1 counters on it" — is a triggered ability
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — ETB replacement effect classification

> **Audit note:** All ETB "clone" permanents (Clone, Clever Impersonator, etc.) use "as enters" and are therefore replacement effects per this rule. This is important for copy effect ordering in 616.1c — copy replacements have third priority in the replacement ordering system.

### 614.1d — BOUNDARY-DEF

"[This permanent] enters..." and "[Objects] enter [the battlefield]..." continuous effects are replacement effects.

**BOUNDARY-DEF-614.1d-001**
- **Rule:** 614.1d — Continuous ETB modification is a replacement effect
- **Mechanism:** "Creatures enter the battlefield tapped" is a replacement effect, not a triggered ability
- **In-set member:** "Creatures enter the battlefield tapped" — replacement effect
- **Out-of-set member:** "Whenever a creature enters the battlefield, tap it" — triggered ability
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Continuous ETB replacement classification

### 614.1e — DEFERRED — Phase 8: Morph
"As [this permanent] is turned face up" replacement effects. Requires face-down infrastructure.

### 614.2 — PURE-DEF
Some replacement effects apply to damage from a source. Cross-reference to 609.7.

### 614.3 — PURE-DEF
No restrictions on casting/activating spells/abilities that generate replacement effects. They last until used up or duration expires. Definitional.

### 614.4 — TESTABLE

Replacement effects must exist BEFORE the event — can't retroactively change something that already happened.

**ATOM-614.4-001**
- **Rule:** 614.4 — Replacement effect must exist before the event
- **Mechanism:** A regeneration shield created after a creature is destroyed cannot save it
- **Minimal Board:** Player A controls a creature. Opponent casts Doom Blade (destroy target nonblack creature). Doom Blade resolves, destroying the creature. THEN Player A tries to activate a regeneration ability.
- **Action:** Player A attempts to regenerate after destruction has already resolved.
- **Expected Result:** Regeneration fails — the creature is already destroyed. Replacement effects cannot "go back in time."
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Replacement effect timing enforcement (pre-event only)

> **Audit note:** Given our implementation approach (replacement effects intercept events as they happen via the execute_action middleware), retroactive application should be impossible by construction. This test may not be meaningfully testable in our engine since the architecture prevents it, but it serves as a contract verification.

### 614.5 — TESTABLE (with Example)

A replacement effect gets only ONE opportunity to affect an event. It doesn't invoke itself repeatedly.

**ATOM-614.5-001**
- **Rule:** 614.5 — Replacement effect doesn't self-repeat
- **Mechanism:** Per CR example: two "double damage" effects → 4x, not infinite. Each replacement applies once.
- **Minimal Board:** Player A controls two permanents, each with "If a creature you control would deal damage, it deals double that damage instead." A creature would deal 2 damage.
- **Action:** The creature deals damage.
- **Expected Result:** 2 → doubled by effect A → 4 → doubled by effect B → 8. NOT infinite. Each replacement applied exactly once to the event.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Replacement effect single-application rule

### 614.6 — TESTABLE

If an event is replaced, the original never happens. The modified event may trigger abilities. Impossible instructions in the modified event are ignored.

**ATOM-614.6-001**
- **Rule:** 614.6 — Replaced event never happens; modified event triggers abilities
- **Mechanism:** "If this creature would die, exile it instead." Death triggers don't fire; exile triggers do.
- **Minimal Board:** Player A controls a creature with "If this creature would die, exile it instead." An ability triggers "when a creature dies" and another triggers "when a creature is exiled."
- **Action:** The creature would be destroyed (lethal damage or destroy effect).
- **Expected Result:** The creature is exiled, not put into graveyard. "When a creature dies" does NOT trigger. "When a creature is exiled" DOES trigger. The original event (dying) never happened.
- **Phase:** Phase 6 (Replacement effects) + Phase 7 (Triggered abilities)
- **Ticket:** NEW — Replaced event suppression + modified event triggers

### 614.7 — TESTABLE

If a replacement effect would replace an event that never happens, the replacement does nothing.

**ATOM-614.7-001**
- **Rule:** 614.7 — Replacement of non-event is a no-op
- **Mechanism:** "If this creature would die, exile it instead." If the creature is already exiled by another effect, the replacement has nothing to replace.
- **Minimal Board:** Player A controls a creature with "If this creature would die, exile it instead." A different effect exiles the creature directly (not death).
- **Action:** The creature is exiled.
- **Expected Result:** The replacement effect doesn't apply — there's no "would die" event to replace. The creature is simply exiled normally.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Replacement effect no-op when event doesn't occur

### 614.7a — TESTABLE

Zero damage is not damage. Replacement effects that would modify zero damage have no event to replace.

**ATOM-614.7a-001**
- **Rule:** 614.7a — Zero damage source has no damage event to replace; additive replacements don't upgrade zero
- **Mechanism:** A source that would deal 0 damage doesn't deal damage at all. Even an additive replacement ("deals that much damage plus 1") should NOT upgrade 0 to 1, because 0 damage is a non-event.
- **Minimal Board:** Player A controls a permanent with "If a source you control would deal damage to a creature, it deals that much damage plus 1 instead." A 0-power creature attacks and is unblocked.
- **Action:** Combat damage step — the 0-power creature would assign 0 damage.
- **Expected Result:** No damage is dealt. The "+1" replacement has no event to replace (0 damage = no damage event). The creature does NOT deal 1 damage. No "damage dealt" triggers fire.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Zero damage is no-event for replacement (already partially implemented: test_execute_zero_damage_is_noop in actions.rs)
- **Tags:** ALREADY-IMPLEMENTED (partial — zero damage no-op exists, but replacement effect layer doesn't)

### 614.8 — TESTABLE

Regeneration is a destruction-replacement effect. "Regenerate [permanent]" means "instead remove all damage, tap it, remove from combat."

**ATOM-614.8-001**
- **Rule:** 614.8 — Regeneration replaces destruction
- **Mechanism:** When a creature with a regeneration shield would be destroyed, instead: remove all damage, tap it, remove from combat
- **Minimal Board:** Player A controls a creature with a regeneration shield active. Opponent casts Doom Blade targeting it.
- **Action:** Doom Blade resolves, attempting to destroy the creature.
- **Expected Result:** Creature is NOT destroyed. All damage is removed. Creature becomes tapped. If it was attacking or blocking, it is removed from combat. The regeneration shield is consumed.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Regeneration as destruction-replacement

**ATOM-614.8-002**
- **Rule:** 614.8 — Regeneration: damage triggers still fire
- **Mechanism:** Per CR: "Abilities that trigger from damage being dealt still trigger even if the permanent regenerates."
- **Minimal Board:** A creature with a regeneration shield takes lethal damage. An ability triggers "whenever damage is dealt to a creature."
- **Action:** Lethal damage is dealt to the creature. SBA would destroy it, but regeneration replaces destruction.
- **Expected Result:** The damage-dealt trigger DOES fire (damage happened). The destruction is replaced by regeneration. The creature survives with damage removed.
- **Phase:** Phase 6 (Replacement effects) + Phase 7 (Triggered abilities)
- **Ticket:** NEW — Regeneration damage triggers still fire

### 614.9 — TESTABLE

Redirection effects: replace damage dealt to one target with same damage to another. If the destination is gone, the effect does nothing.

**ATOM-614.9-001**
- **Rule:** 614.9 — Damage redirection: destination gone → effect does nothing
- **Mechanism:** "The next time a source would deal damage to you, that damage is dealt to target creature instead." If the creature is no longer on the battlefield when the damage would occur, the redirection fails and the original damage is dealt.
- **Minimal Board:** Player A has a redirection shield: "redirect next damage to target creature." The target creature leaves the battlefield.
- **Action:** A source deals damage to Player A.
- **Expected Result:** The creature is gone. The redirection fails. Damage is dealt to Player A normally. The shield is NOT used up.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Damage redirection with invalid destination

### 614.10 — TESTABLE

"Skip" effects replace events with nothing. Once a step/phase/turn has started, skip effects wait for the next occurrence.

**ATOM-614.10-001**
- **Rule:** 614.10 — Skip replaces step/phase/turn with nothing
- **Mechanism:** "Skip your next draw step" means the draw step doesn't happen at all
- **Minimal Board:** Player A has an effect: "Skip your next draw step."
- **Action:** Player A's turn begins, reaching the draw step.
- **Expected Result:** The draw step is entirely skipped — no draw, no turn-based actions, no priority in that step. The turn moves directly from untap step to the first main phase.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Skip step/phase replacement

**ATOM-614.10-002**
- **Rule:** 614.10 — "Skip" beginning mid-step doesn't end current step
- **Mechanism:** If "skip your draw step" begins applying during an already-started draw step, the current draw step completes normally; the skip applies to the NEXT draw step
- **Minimal Board:** Player A is in their draw step. During the draw step (after the draw), an effect creates "Skip your next draw step."
- **Action:** Player A's current draw step continues. Player A's next turn's draw step arrives.
- **Expected Result:** Current draw step completes normally (player already drew). The NEXT draw step is skipped. The skip effect doesn't retroactively end or cancel the current step.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Skip effect doesn't end current step

### 614.10a — TESTABLE

Skipped step/phase: scheduled events don't happen. "Next" effects wait for the first non-skipped occurrence. Two skip effects = skip the next two occurrences.

**ATOM-614.10a-001**
- **Rule:** 614.10a — Two skip effects consume two occurrences
- **Mechanism:** Two "skip your next draw step" effects → skip two consecutive draw steps
- **Minimal Board:** Player A has two effects: "Skip your next draw step" (from two separate sources).
- **Action:** Player A's next two draw steps arrive.
- **Expected Result:** First draw step: skipped (consumes one skip effect). Second draw step: skipped (consumes the other). Third draw step: normal.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Multiple skip effects consumed sequentially

### 614.10b — TESTABLE (META-adjacent)

> **Audit note:** This rule may be quasi-meta — no current cards use the "skip then do X" pattern (Scryfall regex `o:/skip.*then/` returns empty). However, the rule exists in the CR and the engine should handle it correctly if such a card is printed. We include the test for completeness but mark it low-priority.

Skip + follow-up action: the follow-up is the first thing that happens in the next actual occurrence.

**ATOM-614.10b-001**
- **Rule:** 614.10b — Skip with follow-up action defers to next occurrence
- **Mechanism:** "Skip your next draw step. At the beginning of your next draw step, draw two cards." The draw-two happens at the NEXT draw step that actually occurs (the one after the skipped one).
- **Minimal Board:** Player A resolves an effect that skips their next draw step and says "draw two cards at the beginning of your next draw step."
- **Action:** Player A's next two draw steps.
- **Expected Result:** First draw step: skipped entirely. Second draw step: the "draw two" action is the first thing that happens.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Skip follow-up action deferred to next real occurrence

### 614.11 — TESTABLE

Draw-replacement effects apply even if library is empty.

**ATOM-614.11-001**
- **Rule:** 614.11 — Draw replacement applies even with empty library
- **Mechanism:** "If you would draw a card, instead put the top card of target opponent's library into your hand" applies even if your own library is empty
- **Minimal Board:** Player A's library is empty. Player A has a replacement: "If you would draw a card, [do X] instead."
- **Action:** Player A would draw a card.
- **Expected Result:** The replacement applies. The draw is replaced by [X]. The "empty library" SBA (drawing from empty = lose) does NOT trigger because no actual draw occurred.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Draw replacement on empty library

**ATOM-614.11-002**
- **Rule:** 614.11 — Laboratory Maniac: draw replacement with empty library win condition
- **Mechanism:** "If you would draw a card while your library has no cards in it, you win the game instead." This is a draw-replacement that replaces the draw event with a game-winning event.
- **Minimal Board:** Player A controls Laboratory Maniac ("If you would draw a card while your library has no cards in it, you win the game instead."). Player A's library is empty.
- **Action:** Player A would draw a card (e.g., during draw step).
- **Expected Result:** The draw is replaced by "you win the game." Player A wins. The empty-library SBA ("lose for attempting to draw from empty library") never fires because no actual draw occurred — it was replaced.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Draw replacement win condition (Lab Maniac style)

### 614.11a — TESTABLE

Draw replacement within a sequence: complete all replacement actions before resuming the draw sequence.

**ATOM-614.11a-001**
- **Rule:** 614.11a — Draw replacement completes before sequence resumes
- **Mechanism:** "Draw three cards" with a replacement on draws: each draw is replaced individually, and the replacement completes before the next draw begins
- **Minimal Board:** Player A has "If you would draw a card, mill two cards then draw a card instead." Player A resolves "Draw three cards."
- **Action:** Resolve the draw-three effect.
- **Expected Result:** Draw 1: replaced → mill 2, draw 1 (this replacement draw is a new event, but the replacement doesn't re-apply to its own replacement per 614.5). Draw 2: replaced → mill 2, draw 1. Draw 3: replaced → mill 2, draw 1. Total: mill 6, draw 3.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Draw replacement within draw sequence

### 614.11b — TESTABLE

If a draw is replaced, additional actions on "that card" don't apply to replacement-drawn cards.

**ATOM-614.11b-001**
- **Rule:** 614.11b — "Additional action on drawn card" lost if draw replaced
- **Mechanism:** "Draw a card, then discard that card" — if the draw is replaced by "put a card from graveyard into your hand," the discard doesn't apply to the graveyard card
- **Minimal Board:** Player A resolves an effect: "Draw a card, then discard that card." Player A has a replacement: "If you would draw a card, instead return a card from your graveyard to your hand."
- **Action:** The draw is replaced.
- **Expected Result:** A card returns from graveyard to hand (replacement). The "discard that card" additional action is NOT performed on the returned card — the original draw was replaced, so "that card" no longer refers to anything.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Additional action lost on replaced draw

### 614.12 — TESTABLE (with Examples)

ETB replacement effects: check the permanent's characteristics AS IT WOULD EXIST on the battlefield, taking into account already-applied replacements, the permanent's own static abilities, and existing continuous effects.

**ATOM-614.12-001**
- **Rule:** 614.12 — ETB replacement uses look-ahead characteristics
- **Mechanism:** Per CR example: Scarwood Treefolk ("This creature enters tapped") enters from graveyard while Yixlid Jailer ("Cards in graveyards lose all abilities") is on the battlefield. The Treefolk's ability is checked as it WOULD exist on the battlefield (where Jailer doesn't apply), so it enters tapped.
- **Minimal Board:** Yixlid Jailer on the battlefield ("Cards in graveyards lose all abilities"). Scarwood Treefolk in graveyard ("This creature enters tapped").
- **Action:** Scarwood Treefolk is put onto the battlefield from the graveyard.
- **Expected Result:** Treefolk enters tapped. Even though it had no abilities in the graveyard (due to Jailer), the look-ahead checks what it WOULD be on the battlefield — and on the battlefield, Jailer doesn't suppress its abilities.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — ETB look-ahead characteristic evaluation

**ATOM-614.12-002**
- **Rule:** 614.12 — ETB replacement: permanent's own static ability applies
- **Mechanism:** Per CR example: Voice of All ("As this creature enters, choose a color. This creature has protection from the chosen color.") A token copy of Voice of All must make the color choice.
- **Minimal Board:** An effect creates a token that's a copy of Voice of All.
- **Action:** The token is created and would enter the battlefield.
- **Expected Result:** The token's controller chooses a color as it enters. The token has protection from the chosen color. The "as enters" replacement is the token's own ability.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — ETB look-ahead: own static abilities apply

**ATOM-614.12-003**
- **Rule:** 614.12 — ETB replacement: permanent doesn't affect itself
- **Mechanism:** Per CR example: Orb of Dreams ("Permanents enter tapped") doesn't affect itself entering.
- **Minimal Board:** Orb of Dreams is cast and resolves.
- **Action:** Orb of Dreams enters the battlefield.
- **Expected Result:** Orb of Dreams enters UNTAPPED. Its own ability isn't on the battlefield yet when it checks "would this permanent enter tapped?" — the look-ahead considers existing continuous effects, but the Orb itself isn't "existing" yet.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — ETB look-ahead: self doesn't affect self

### 614.12a — TESTABLE

ETB replacement choices are made BEFORE entering.

**ATOM-614.12a-001**
- **Rule:** 614.12a — ETB replacement choices made before entering
- **Mechanism:** "As this enters, choose a color" — the choice is made before the permanent is on the battlefield
- **Minimal Board:** A creature with "As this creature enters, choose a color. This creature has protection from the chosen color" is entering the battlefield.
- **Action:** The creature enters.
- **Expected Result:** The choice is made before the creature is on the battlefield. Effects that trigger "when a creature enters" see the creature already having protection from the chosen color — the choice was made pre-entry.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — ETB choice timing (pre-entry)

> **Audit note:** Cross-reference with steps of casting a spell (601.2a–601.2h) to determine at exactly which step this choice happens during the casting process. For ETB replacements, the choice happens as the permanent would enter (not during the casting steps), but for spells that put things onto the battlefield, the timing of when the replacement is evaluated matters.

### 614.12b — DEFERRED — Phase 8
Multiple simultaneous ETB replacements with choices — combined costs must be payable. Complex corner case.

### 614.12c — DEFERRED — Phase 8
Anchor word ETB choices ("Ward" / "Backup" style). Linked abilities with anchor words.

### 614.13 — TESTABLE

ETB modification may cause OTHER objects to change zones.

**ATOM-614.13-001**
- **Rule:** 614.13 — ETB replacement causes other zone changes
- **Mechanism:** Devour: "As this creature enters, you may sacrifice any number of creatures." The sacrificed creatures change zones as part of the ETB replacement.
- **Minimal Board:** Player A casts a creature with Devour 3. Player A controls two other creatures.
- **Action:** As the Devour creature enters, Player A sacrifices both creatures.
- **Expected Result:** Both sacrificed creatures go to graveyard. The Devour creature enters with +1/+1 counters (3 per sacrificed creature = 6 counters). The sacrifices happen as part of the ETB replacement, not as a triggered ability.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — ETB replacement with auxiliary zone changes

### 614.13a — TESTABLE (with Example)

Can't choose the entering object itself or other simultaneously-entering objects for auxiliary zone changes.

**ATOM-614.13a-001**
- **Rule:** 614.13a — Can't choose entering object or simultaneous-entry objects
- **Mechanism:** Per CR example: Sutured Ghoul and Runeclaw Bear enter from graveyard simultaneously. Can't exile either for Sutured Ghoul's "exile creature cards from graveyard" ETB.
- **Minimal Board:** Sutured Ghoul ("As this enters, exile any number of creature cards from your graveyard") and Runeclaw Bear both in graveyard. An effect puts both onto the battlefield simultaneously.
- **Action:** Sutured Ghoul's ETB replacement asks to choose creatures to exile from graveyard.
- **Expected Result:** Neither Sutured Ghoul itself nor Runeclaw Bear can be chosen — both are entering the battlefield simultaneously. Only other creature cards in the graveyard are legal choices.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — ETB auxiliary zone change exclusion for simultaneous entry

### 614.13b — TESTABLE (with Example)

Same object can't be chosen for multiple ETB auxiliary zone changes.

**ATOM-614.13b-001**
- **Rule:** 614.13b — Object can't be chosen for multiple ETB replacements
- **Mechanism:** Per CR example: Thunder-Thrash Elder has devour 3, and Jund plane grants devour 5. A single Runeclaw Bear can be sacrificed to ONE of the devour effects, not both.
- **Minimal Board:** Player A controls Runeclaw Bear. Player A casts Thunder-Thrash Elder (devour 3) while a "devour 5" effect also applies.
- **Action:** As Thunder-Thrash Elder enters, Player A chooses to sacrifice Runeclaw Bear.
- **Expected Result:** Runeclaw Bear can be sacrificed to devour 3 OR devour 5, but not both. Thunder-Thrash Elder enters with either 0, 3, or 5 +1/+1 counters — never 8.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Single object for single ETB replacement

### 614.13c — DEFERRED — Phase 8
ETB replacement that mills/exiles from library: entering card from that library is excluded. Corner case with Ashiok + shock lands.

### 614.14 — PURE-DEF
Linked abilities between exile-replacement and "exiled with this" reference. Cross-reference to rule 607. Definitional — tested via the linked abilities that use it.

### 614.15 — TESTABLE

Self-replacement effects: a resolving spell/ability replaces part of its own effect. Self-replacements apply BEFORE other replacement effects.

**ATOM-614.15-001**
- **Rule:** 614.15 — Self-replacement applies before other replacements
- **Mechanism:** A spell that says "Deal 3 damage to any target. If a creature would be dealt damage this way, it deals double that damage instead" — the "double" is a self-replacement and applies first
- **Minimal Board:** Player A controls a permanent with "If a source would deal damage to a creature, prevent 1 of that damage." Player A casts a spell: "Deal 3 damage to target creature. If a creature would be dealt damage this way, deal double that damage instead."
- **Action:** The spell resolves targeting a creature.
- **Expected Result:** Self-replacement first: 3 → 6. Then prevention: 6 - 1 = 5 damage dealt. Self-replacement applied before the external prevention.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Self-replacement effect priority

**ATOM-614.15-002**
- **Rule:** 614.15 — Self-replacement: Aang's Journey (real card, kicked search replacement)
- **Mechanism:** Aang's Journey ({2} Sorcery — Lesson, Kicker {2}): "Search your library for a basic land card. If this spell was kicked, instead search your library for a basic land card and a Shrine card. Reveal those cards, put them into your hand, then shuffle." The kicked clause is a self-replacement that modifies the spell's own search effect.
- **Minimal Board:** Player A casts Aang's Journey with kicker paid ({4} total). Player A's library contains basic lands and Shrine cards.
- **Action:** Aang's Journey resolves.
- **Expected Result:** Self-replacement applies: instead of searching for just a basic land, Player A searches for a basic land AND a Shrine card. Both are revealed, put into hand, then shuffle. The self-replacement modifies the spell's own effect before any external replacements could apply.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Self-replacement real card example (Aang's Journey)

### 614.16 — TESTABLE

Token/counter replacement effects apply to tokens/counters created by other replacement effects, not just original spell effects.

**ATOM-614.16-001**
- **Rule:** 614.16 — Token replacement applies to tokens from other replacements
- **Mechanism:** Doubling Season ("If an effect would create tokens, create twice that many") applies even if the tokens are created by a replacement effect
- **Minimal Board:** Player A controls Doubling Season. A replacement effect says "If a creature would die, instead create a 1/1 Spirit token."
- **Action:** A creature would die.
- **Expected Result:** Death replaced → create 1/1 Spirit. Doubling Season applies to the token creation → create two 1/1 Spirits instead of one. 614.16 ensures the doubling applies to replacement-created tokens.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Token/counter replacement chains

### 614.17 — TESTABLE

"Can't" effects are NOT replacement effects but follow similar rules. Tags: META-101.2

**ATOM-614.17-001**
- **Rule:** 614.17 — "Can't" effects follow replacement-like rules but aren't replacement effects
- **Mechanism:** "Damage can't be prevented" is not a replacement effect — it doesn't replace the prevention, it stops it from functioning
- **Minimal Board:** Player A controls a permanent with "Damage dealt by sources you control can't be prevented." Player B has a prevention shield.
- **Action:** Player A's creature deals damage to Player B.
- **Expected Result:** Prevention shield doesn't prevent the damage. The "can't" effect isn't a replacement — it overrides the prevention. Per META-101.2, "can't" beats "can."
- **Phase:** Phase 6 (Replacement/Prevention effects)
- **Ticket:** NEW — "Can't" effect overrides prevention
- **Tags:** META-101.2

### 614.17a — TESTABLE

"Can't" effects must exist before the event — same timing rule as replacement effects.

**ATOM-614.17a-001**
- **Rule:** 614.17a — "Can't" must pre-exist the event
- **Mechanism:** "Creatures can't attack" must be active before the declare attackers step; creating it after attackers are declared doesn't undo the attack
- **Minimal Board:** Player B's creature is declared as an attacker. After attackers are declared, an effect creates "Creatures can't attack."
- **Action:** Combat continues.
- **Expected Result:** The creature remains attacking. The "can't attack" effect can't retroactively undo a declaration that already happened.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — "Can't" effect timing (pre-event only)

### 614.17b — TESTABLE

If an event can't happen, a player can't choose to pay a cost that includes that event.

**ATOM-614.17b-001**
- **Rule:** 614.17b — Can't pay costs involving impossible events
- **Mechanism:** If "creatures can't be sacrificed" is active, a player can't pay a sacrifice cost
- **Minimal Board:** Player A controls a permanent with "Sacrifice a creature: Draw a card." An effect says "Creatures can't be sacrificed."
- **Action:** Player A attempts to activate the sacrifice ability.
- **Expected Result:** Activation is illegal — the cost (sacrifice a creature) involves an event that can't happen. The ability can't be activated.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Impossible cost events block activation
- **Tags:** META-101.2

### 614.17c — TESTABLE

If an event can't happen, it can only be replaced by a self-replacement effect. Other replacement/prevention effects can't modify it.

**ATOM-614.17c-001**
- **Rule:** 614.17c — "Can't" event only replaceable by self-replacement that changes the event type
- **Mechanism:** If damage "can't be prevented," a prevention effect has no effect. A self-replacement can still modify the event, but only if it changes the event to something other than what "can't" prohibits. A self-replacement that still results in damage doesn't bypass the "can't prevent" — the key is that the self-replacement must change the event type entirely (e.g., replace damage with life loss or exile).
- **Minimal Board:** A source would deal 3 damage that "can't be prevented." The source spell has a self-replacement: "If this spell would deal damage, instead exile the top 3 cards of that player's library." A prevention shield also exists.
- **Action:** The source's effect resolves.
- **Expected Result:** Self-replacement applies: damage is replaced with exile (event type changed entirely). Prevention shield is irrelevant — there's no longer a damage event to prevent. The "can't be prevented" is also irrelevant because no damage is dealt. The self-replacement successfully changed the event because it transformed it into a non-damage event.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Self-replacement for "can't" events must change event type
- **Tags:** META-101.2

### 614.17d — META (TESTABLE per-ETB-type)

"Can't" effects on ETB use the same look-ahead as 614.12. This rule is **META**: it applies to every ETB replacement type (can't enter tapped, can't gain counters, can't be sacrificed as ETB cost, etc.). Rather than a single atomic test, each ETB replacement type the engine supports should have a corresponding 614.17d "can't" test. We include the most common case (enters tapped) as the representative test.

**ATOM-614.17d-001**
- **Rule:** 614.17d — ETB "can't" uses look-ahead characteristics (representative: enters tapped)
- **Mechanism:** "Creatures can't enter the battlefield tapped" — checks the permanent's characteristics as it would exist on the battlefield
- **Minimal Board:** Player A controls a permanent with "Creatures can't enter the battlefield tapped." A creature with "This creature enters tapped" enters.
- **Action:** The creature enters the battlefield.
- **Expected Result:** The creature enters UNTAPPED. The "can't enter tapped" effect overrides the "enters tapped" replacement. The look-ahead sees the creature as it would exist on the battlefield to check if the "can't" applies.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — ETB "can't" effect with look-ahead
- **Tags:** META-101.2, META-614.17d

---

## 615. Prevention Effects

### 615.1 — PURE-DEF
Prevention effects are continuous effects that watch for damage events and prevent damage. Introduces the concept — "shields."

### 615.1a — BOUNDARY-DEF

"Prevent" keyword identifies prevention effects.

**BOUNDARY-DEF-615.1a-001**
- **Rule:** 615.1a — "Prevent" identifies a prevention effect
- **Mechanism:** Engine must classify effects using "prevent" as prevention effects
- **In-set member:** "Prevent the next 3 damage that would be dealt to target creature" — IS a prevention effect
- **Out-of-set member:** "Whenever damage is dealt to target creature, you gain that much life" — triggered ability, NOT a prevention effect
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Prevention effect classification by "prevent" keyword

### 615.2 — PURE-DEF
Many prevention effects apply to damage from a source. Cross-reference to 609.7. Definitional.

### 615.3 — PURE-DEF
No restrictions on casting/activating prevention-generating spells/abilities. They last until used up or duration expires. Definitional.

### 615.4 — TESTABLE (with Example)

Prevention effects must exist before the damage event. Same timing rule as replacement effects (614.4).

**ATOM-615.4-001**
- **Rule:** 615.4 — Prevention must exist before damage event
- **Mechanism:** Per CR example: a prevention ability activated in response to a damage spell works; activating it after the spell resolves is too late
- **Minimal Board:** Player A controls a creature. Opponent casts Lightning Bolt targeting it. Player A has an ability "Prevent the next 3 damage to target creature."
- **Action:** (1) Player A activates the prevention ability in response to Bolt. Bolt resolves. (2) Alternatively: Bolt resolves first, THEN Player A tries to prevent.
- **Expected Result:** (1) Prevention succeeds — shield was active before damage. (2) Prevention fails — damage already happened.
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Prevention timing enforcement

### 615.5 — TESTABLE

Prevention effects may include additional effects referencing the amount prevented. The prevention happens at damage-time; the additional effect happens immediately after.

**ATOM-615.5-001**
- **Rule:** 615.5 — Prevention with additional effect fires after prevention
- **Mechanism:** "Prevent the next 3 damage that would be dealt to you. If damage is prevented this way, you gain that much life." Prevention first, then life gain.
- **Minimal Board:** Player A has a shield: "Prevent the next 3 damage. If damage is prevented, gain that much life." A source would deal 5 damage to Player A.
- **Action:** The damage event occurs.
- **Expected Result:** 3 damage prevented (shield consumed). 2 damage dealt to Player A. Player A gains 3 life (the additional effect, referencing the 3 prevented damage).
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Prevention additional effect with prevented amount

### 615.6 — TESTABLE

Prevented damage never happens. The modified event (with reduced or zero damage) may trigger abilities. Impossible instructions in the modified event are ignored.

**ATOM-615.6-001**
- **Rule:** 615.6 — Prevented damage never happens; modified event triggers
- **Mechanism:** "Prevent all damage that would be dealt to target creature this turn." Damage-dealt triggers don't fire for prevented damage.
- **Minimal Board:** Player A controls a creature with full prevention. An ability triggers "whenever damage is dealt to a creature."
- **Action:** A source would deal 3 damage to the creature. All 3 is prevented.
- **Expected Result:** No damage is dealt. The "whenever damage is dealt" trigger does NOT fire. The prevention completely negated the event.
- **Phase:** Phase 6 (Prevention effects) + Phase 7 (Triggered abilities)
- **Ticket:** NEW — Prevented damage suppresses damage triggers

### 615.7 — TESTABLE (with Example)

Prevention shields with specific amounts ("Prevent the next N damage"). Each 1 damage prevented reduces the shield by 1. Multiple simultaneous sources: controller chooses which damage is prevented. Once shield = 0, remaining damage dealt normally.

**ATOM-615.7-001**
- **Rule:** 615.7 — Prevention shield depletes per damage point
- **Mechanism:** "Prevent the next 3 damage" shield — absorbs up to 3 damage
- **Minimal Board:** Player A has a "Prevent the next 3 damage to you this turn" shield. A source deals 5 damage to Player A.
- **Action:** Damage event occurs.
- **Expected Result:** 3 damage prevented (shield consumed). 2 damage dealt to Player A. Shield is now at 0.
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Prevention shield depletion

**ATOM-615.7-002**
- **Rule:** 615.7 — Multiple simultaneous damage sources: controller chooses allocation
- **Mechanism:** Two creatures deal damage simultaneously to Player A who has a 3-damage shield. Player A chooses how to allocate the prevention.
- **Minimal Board:** Player A has a 3-damage prevention shield. Two creatures deal combat damage simultaneously: creature A (2 damage) and creature B (4 damage).
- **Action:** Combat damage is dealt simultaneously.
- **Expected Result:** Player A chooses which damage to prevent. E.g., prevent all 2 from creature A and 1 from creature B → take 3 from B. Or prevent 3 from creature B → take 2 from A and 1 from B. Total damage taken is always 3 (6 total - 3 prevented).
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Prevention shield allocation choice for simultaneous damage

### 615.8 — TESTABLE

"Prevent the next instance of damage from [source]" — prevents the entire next damage event from that source, regardless of amount. Subsequent instances from the same source deal damage normally.

**ATOM-615.8-001**
- **Rule:** 615.8 — Prevent next instance from source: amount-independent
- **Mechanism:** "Prevent the next time [source] would deal damage" prevents all damage from the next instance, even if it's 100 damage
- **Minimal Board:** Player A has a shield: "Prevent the next time target creature would deal damage this turn." The creature would deal 7 combat damage.
- **Action:** Combat damage from the creature.
- **Expected Result:** All 7 damage prevented (the entire instance). The shield is consumed. If the creature deals damage again this turn (e.g., via an ability), that damage is NOT prevented.
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Prevent next instance (amount-independent)

### 615.9 — TESTABLE

Prevention from sources with certain properties: recheck source properties when damage would be dealt. If properties no longer match, shield is not used up.

**ATOM-615.9-001**
- **Rule:** 615.9 — Prevention shield rechecks source properties
- **Mechanism:** "Prevent damage from a red source of your choice." If the chosen source changes color, the shield doesn't apply and isn't consumed.
- **Minimal Board:** Player A has a prevention shield targeting a red creature as the damage source. An effect changes the creature to blue.
- **Action:** The (now blue) creature deals damage.
- **Expected Result:** Prevention doesn't apply (source is no longer red). Shield is NOT used up. Damage is dealt normally.
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Prevention shield source property recheck (also tested in 609.7b)

### 615.10 — TESTABLE (with Example)

Static prevention with specific amount: "If a source would deal damage to you, prevent 1 of that damage." Applies to EACH applicable damage event independently, not a total pool.

**ATOM-615.10-001**
- **Rule:** 615.10 — Static prevention applies per-event independently
- **Mechanism:** Per CR example: Daunting Defender prevents 1 damage to each Cleric from each source. Pyroclasm deals 2 to each creature → each Cleric takes 1 (2-1).
- **Minimal Board:** Daunting Defender ("If a source would deal damage to a Cleric you control, prevent 1 of that damage"). Player A controls two Clerics and one non-Cleric. Opponent casts Pyroclasm (2 damage to each creature).
- **Action:** Pyroclasm resolves.
- **Expected Result:** Each Cleric takes 1 damage (2 - 1 prevented). The non-Cleric takes 2 damage. The prevention applies independently to each damage event to each Cleric.
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Static prevention per-event application

### 615.11 — TESTABLE (with Example)

Prevention shields for each of a number of untargeted creatures: one shield per applicable creature at resolution time. Changing creature colors after resolution doesn't add/remove shields.

**ATOM-615.11-001**
- **Rule:** 615.11 — Prevention shields created per-creature at resolution
- **Mechanism:** Per CR example: "Prevent next 1 damage to target creature and each creature sharing a color." Shields assigned at resolution; new creatures later don't get shields.
- **Minimal Board:** Player A controls three white creatures. An ability resolves: "Prevent the next 1 damage that would be dealt to target creature and each creature that shares a color with it this turn." A new white creature enters after resolution.
- **Action:** All creatures take damage.
- **Expected Result:** The three original white creatures each have a 1-damage shield. The new creature (entered after resolution) has NO shield. Each original creature takes 1 less damage from the first source.
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Per-creature prevention shield assignment at resolution

### 615.12 — TESTABLE

> **Audit note — META-101.2 cross-reference:** This rule is a concrete instance of the META "can't beats can" principle from Chapter 1 (rule 101.2). The engine's implementation of 615.12 should use the same "can't" override mechanism as 614.17. Both are tracked under META-101.2.

"Damage can't be prevented" — prevention effects still apply but don't prevent anything. Additional effects from prevention still happen. Existing shields are NOT reduced.

**ATOM-615.12-001**
- **Rule:** 615.12 — Unpreventable damage: prevention applies but doesn't prevent; shields preserved
- **Mechanism:** "Damage can't be prevented" means prevention effects are applied but prevent zero damage. Shields are not reduced.
- **Minimal Board:** Player A has a 3-damage prevention shield. An effect says "Damage can't be prevented this turn." A source deals 4 damage to Player A.
- **Action:** Damage event occurs.
- **Expected Result:** Prevention effect is applied but prevents 0 damage (unpreventable). Player A takes 4 damage. The shield remains at 3 — it was not consumed because no damage was actually prevented.
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Unpreventable damage preserves prevention shields
- **Tags:** META-101.2

**ATOM-615.12-002**
- **Rule:** 615.12 — Unpreventable damage: additional prevention effects still fire
- **Mechanism:** "Prevent the next 3 damage. If damage is prevented this way, [additional effect]." With unpreventable damage, the prevention prevents 0, so the "if damage is prevented" additional effect does NOT fire.
- **Minimal Board:** Player A has "Prevent the next 3 damage to you. If damage is prevented, gain that much life." Damage can't be prevented this turn.
- **Action:** A source deals 3 damage.
- **Expected Result:** Prevention is applied but prevents 0 (unpreventable). Player A takes 3 damage. No life is gained — "if damage is prevented this way" saw 0 prevented damage.
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Unpreventable damage: additional effect conditional on amount prevented

### 615.12a — TESTABLE

Prevention effect applied to unpreventable damage event only once — doesn't invoke itself repeatedly.

**ATOM-615.12a-001**
- **Rule:** 615.12a — Prevention on unpreventable damage: single application
- **Mechanism:** A prevention effect doesn't loop trying to prevent unpreventable damage
- **Minimal Board:** Player A has a prevention effect. Damage is unpreventable.
- **Action:** Damage occurs.
- **Expected Result:** Prevention effect is applied once, prevents nothing, done. No infinite loop.
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW — Prevention single-application on unpreventable damage

### 615.13 — DEFERRED — Phase 7
Triggered abilities that trigger when damage is prevented. Requires triggered ability infrastructure.

---

## 616. Interaction of Replacement and/or Prevention Effects

### 616.1 — TESTABLE (with Examples)

When multiple replacement/prevention effects apply to the same event, the affected object's controller (or owner if no controller) or the affected player chooses which to apply. APNAP for simultaneous multi-player choices.

**ATOM-616.1-001**
- **Rule:** 616.1 — Player chooses which replacement/prevention to apply
- **Mechanism:** Per CR example: "If a card would go to graveyard, exile it instead" and "If this creature would die, shuffle into library instead." Controller chooses which applies first.
- **Minimal Board:** Player A controls a creature. Two replacement effects apply to it dying: Effect A ("exile instead") and Effect B ("shuffle into library instead").
- **Action:** The creature would die.
- **Expected Result:** Player A (controller) chooses which replacement to apply first. If they choose A → creature exiled (B no longer applies, creature didn't die). If they choose B → creature shuffled into library (A no longer applies).
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Player choice for multiple applicable replacements

### 616.1a — TESTABLE

Self-replacement effects have priority: if any applicable, one MUST be chosen before non-self-replacements.

**ATOM-616.1a-001**
- **Rule:** 616.1a — Self-replacement must be chosen first
- **Mechanism:** A spell's self-replacement effect must be applied before external replacement effects
- **Minimal Board:** A spell says "Deal 3 damage to target creature. If this damage would be prevented, deal double that damage instead" (self-replacement). Player B has a prevention shield.
- **Action:** Spell resolves.
- **Expected Result:** Self-replacement (616.1a) must be chosen first. After self-replacement, the remaining prevention may apply to the modified event.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Self-replacement priority in 616.1 ordering

### 616.1b — TESTABLE

Control-changing replacements have second priority (after self-replacements).

**ATOM-616.1b-001**
- **Rule:** 616.1b — Control-changing replacement chosen before other ETB replacements
- **Mechanism:** If a permanent entering the battlefield has both a control-changing replacement and other replacements, the control-changing one must be chosen first (after any self-replacements)
- **Minimal Board:** A creature enters with two applicable replacement effects: "This creature enters under target opponent's control" and "This creature enters with two +1/+1 counters."
- **Action:** The creature enters the battlefield.
- **Expected Result:** Per 616.1b, the control-changing replacement must be applied first. Then the +1/+1 counter replacement applies. The controller is determined before counters are placed.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Control-changing ETB replacement priority

> **Audit note:** Need to find a confirmed-correct example for this sub-rule — the CR doesn't provide examples for 616.1b specifically. The test above is inferred from the rule text. Verify with a judge or CR errata source before implementing.

### 616.1c — TESTABLE

Copy-becoming replacements have third priority (after control-changing).

**ATOM-616.1c-001**
- **Rule:** 616.1c — Copy replacement chosen before generic ETB replacements
- **Mechanism:** Per CR example (Essence of the Wild): a creature entering that would be copied by Essence of the Wild AND enter tapped — the copy effect must be applied first. If the copy removes the "enter tapped" ability, it won't enter tapped.
- **Minimal Board:** Essence of the Wild ("Creatures you control enter as a copy of this creature") on the battlefield. Player A casts Rusted Sentinel ("This creature enters tapped").
- **Action:** Rusted Sentinel enters the battlefield.
- **Expected Result:** Per 616.1c, copy replacement applied first: Rusted Sentinel becomes a copy of Essence of the Wild. The copy no longer has "enters tapped." Rusted Sentinel enters UNTAPPED as a copy of Essence of the Wild.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Copy ETB replacement priority (616.1c)

### 616.1d — DEFERRED — Phase 8: Transform
Back-face-up replacements have fourth priority. Requires transform/DFC infrastructure.

### 616.1e — PURE-DEF
After self-replacement, control, copy, and back-face priorities, any remaining replacement/prevention may be chosen. Defines the fallthrough — tested implicitly by 616.1-001.

### 616.1f — TESTABLE

After applying one replacement/prevention, repeat the process with remaining applicable effects until none remain.

**ATOM-616.1f-001**
- **Rule:** 616.1f — Iterative application until no more replacements apply
- **Mechanism:** After one replacement is applied, re-check for applicable replacements on the modified event. Continue until none remain.
- **Minimal Board:** Three replacement effects apply to a dying creature: A ("exile instead"), B ("shuffle instead"), C ("return to hand instead").
- **Action:** Creature would die. Player chooses A (exile).
- **Expected Result:** After A applies, the event is now "exile." B and C watch for "die" events, not "exile." Neither applies. Process stops. Creature is exiled.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Iterative replacement application loop

### 616.1g — TESTABLE (with Example)

Nested events: a replacement that applies to an event contained within another event can't be chosen until the outer event's replacement is applied first.

**ATOM-616.1g-001**
- **Rule:** 616.1g — Outer event replacement before inner event replacement
- **Mechanism:** Per CR example: Doubling Season ("If an effect would create tokens, create twice that many") contains the event of tokens entering the battlefield. Voice of All's "As this enters, choose a color" applies to entering. Doubling Season must be applied first (outer event), then Voice of All choices for each token (inner event).
- **Minimal Board:** Doubling Season on the battlefield. An effect creates a token copy of Voice of All.
- **Action:** Token creation event occurs.
- **Expected Result:** Doubling Season applies first (outer: token creation doubled → two tokens). Then each Voice of All token makes its "choose a color" choice independently (inner: entering the battlefield). Two tokens, each with potentially different color choices.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Outer/inner event replacement ordering

> **Audit note — 616.1g engine design:** How does the engine distinguish "outer" vs "inner" events? The replacement system needs to recognize containment: a token-creation event contains individual ETB events. Design discussion in session-6-audit-response.md.

### 616.2 — TESTABLE (with Example)

A replacement/prevention can become applicable as a result of another replacement modifying the event.

**ATOM-616.2-001**
- **Rule:** 616.2 — Replacement chains: new replacement applies to modified event
- **Mechanism:** Per CR example: "If you would gain life, draw that many cards instead" + "If you would draw a card, return a card from graveyard instead." Gaining 1 life → draw 1 card → return 1 card from graveyard.
- **Minimal Board:** Player A has Effect A: "If you would gain life, draw that many cards instead." Player A has Effect B: "If you would draw a card, return a card from your graveyard to your hand instead."
- **Action:** Player A would gain 1 life.
- **Expected Result:** Effect A replaces life gain with card draw. Effect B then becomes applicable and replaces the card draw with graveyard-to-hand. Final result: return a card from graveyard to hand. The chain works regardless of which effect was created first.
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Replacement effect chaining across event modifications

---

## Composition Tests

Tests that require 2+ atomic mechanisms working together.

**COMP-613-HUMILITY-OPALESCENCE-001**
- **Rule:** 613.1f + 613.1g + 613.4b + 613.7 — Humility + Opalescence timestamp interaction
- **Mechanism:** Both Humility and Opalescence affect creatures in L6 and L7. Timestamp order determines the result.
- **Minimal Board:** Player A controls Humility (T1) and Opalescence (T2, T2 > T1). Both are 4-MV enchantments.
- **Action:** Query Opalescence's characteristics (as animated by itself? No — Opalescence says "each OTHER non-Aura enchantment"). Query Humility's characteristics.
- **Expected Result:** Humility is animated by Opalescence (L4: creature, L7b: P/T = 4/4 from MV). Humility is then affected by its own "lose all abilities, base 1/1" (L6: removes abilities, L7b: sets 1/1). BUT: Opalescence L7b (T2) and Humility L7b (T1) — T2 > T1 → Opalescence's "P/T = MV" applies last → Humility is 4/4. If timestamps reversed: Humility is 1/1.
- **Composes:** ATOM-613.1f-001, ATOM-613.4b-001, ATOM-613.7-002
- **Phase:** Phase 5 Layers (L19, L20)
- **Ticket:** L19, L20

**COMP-613-BLOOD-MOON-URBORG-001**
- **Rule:** 613.8a + 613.8 — Blood Moon + Urborg dependency analysis
- **Mechanism:** Both in L4. Urborg depends on Blood Moon (Blood Moon makes Urborg a Mountain, removing its static ability → changes existence of Urborg's effect, dependency condition b). Blood Moon does NOT depend on Urborg. Therefore Blood Moon applies first regardless of timestamps.
- **Minimal Board:** Blood Moon and Urborg, Tomb of Yawgmoth on the battlefield. A nonbasic land (e.g., Tropical Island). Timestamps irrelevant due to dependency.
- **Action:** Query Tropical Island's subtypes and mana abilities. Query Urborg's subtypes and mana abilities.
- **Expected Result:** Blood Moon applies first: all nonbasic lands become Mountains (losing all subtypes and printed abilities, gaining Mountain + "{T}: Add {R}"). This includes Urborg itself — Urborg becomes a Mountain, losing its "Each land is a Swamp" ability. Urborg's effect ceases to exist and never applies. **Tropical Island can only tap for {R}. Urborg can only tap for {R}.** No lands gain the Swamp subtype.
- **Composes:** ATOM-613.8a-001, ATOM-613.8-001
- **Phase:** Phase 5 Layers (L14, L17, L20)
- **Ticket:** L14, L17, L20

**COMP-613-TARMOGOYF-HUMILITY-001**
- **Rule:** 613.1f + 613.4a + 613.4b — Tarmogoyf under Humility
- **Mechanism:** Humility removes Tarmogoyf's CDA in L6, then sets base 1/1 in L7b. CDA never applies.
- **Minimal Board:** Player A controls Humility and Tarmogoyf. 5 card types in graveyard.
- **Action:** Query Tarmogoyf's P/T.
- **Expected Result:** Tarmogoyf is 1/1. L6: Humility removes all abilities (CDA gone). L7a: no CDA to apply. L7b: Humility sets 1/1.
- **Composes:** ATOM-613.1f-001, ATOM-613.4a-001, ATOM-613.4b-001
- **Phase:** Phase 5 Layers (L19, L20)
- **Ticket:** L19, L20

**COMP-614-616-DOUBLE-REPLACEMENT-001**
- **Rule:** 614.5 + 616.1 — Two doublers with player choice
- **Mechanism:** Two "double damage" replacements — player chooses order per 616.1, each applies once per 614.5.
- **Minimal Board:** Player A controls two "damage is doubled" effects. A creature deals 2 damage.
- **Action:** Damage event.
- **Expected Result:** Player A chooses order. Either way: 2 → 4 → 8. Total: 8 damage (not infinite).
- **Composes:** ATOM-614.5-001, ATOM-616.1-001
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW

**COMP-614-DAMAGE-ORDERING-001**
- **Rule:** 614.5 + 616.1 — Two damage-modification replacements: order matters for non-commutative operations
- **Mechanism:** Two replacement effects: Effect A ("If a source would deal damage, it deals that much damage plus 1 instead") and Effect B ("If a source would deal damage, it deals double that damage instead"). These are NOT commutative: A then B = (N+1)*2, B then A = (N*2)+1. Player chooses order per 616.1.
- **Minimal Board:** Player A controls both Effect A and Effect B. A creature would deal 3 damage.
- **Action:** Player A chooses which replacement to apply first.
- **Expected Result:** If A first then B: 3 → 4 → 8 damage. If B first then A: 3 → 6 → 7 damage. Player A chooses the order. This test verifies that (a) player choice is presented, (b) both orderings produce different but correct results, and (c) each replacement applies exactly once per 614.5.
- **Composes:** ATOM-614.5-001, ATOM-616.1-001
- **Phase:** Phase 6 (Replacement effects)
- **Ticket:** NEW — Non-commutative replacement ordering with player choice

**COMP-615-UNPREVENTABLE-SHIELD-001**
- **Rule:** 615.12 + 615.7 — Unpreventable damage with shield
- **Mechanism:** Prevention shield applied to unpreventable damage: prevents 0, shield not consumed.
- **Minimal Board:** Player A has a 3-damage shield. Damage is unpreventable.
- **Action:** 5 unpreventable damage dealt to Player A.
- **Expected Result:** Shield applies, prevents 0. Player A takes 5 damage. Shield remains at 3.
- **Composes:** ATOM-615.12-001, ATOM-615.7-001
- **Phase:** Phase 6 (Prevention effects)
- **Ticket:** NEW

**COMP-613-SVOGTHOS-001**
- **Rule:** 613.6 + 613.4a — Svogthos multi-layer activation
- **Mechanism:** Svogthos's activated ability creates a continuous effect spanning L4 (type change to creature), L5 (color set to black/green), and L7a (CDA for P/T). Tests that a single effect correctly applies its parts in multiple layers per 613.6.
- **Minimal Board:** Player A controls Svogthos, the Restless Tomb. Player A's graveyard contains 4 creature cards. Player A activates Svogthos's ability: "{3}{B}{G}: Until end of turn, Svogthos becomes a black and green Plant Zombie creature with 'This creature’s power and toughness are each equal to the number of creature cards in your graveyard.' It's still a land."
- **Action:** Query Svogthos's characteristics after activation.
- **Expected Result:** L4: Svogthos gains creature type (Plant Zombie), retains land type. L5: color set to black and green. L7a: CDA sets P/T to 4/4 (4 creature cards in graveyard). If a creature card is added to graveyard, P/T updates dynamically (CDA re-evaluates). Svogthos is a 4/4 black/green Plant Zombie Land creature.
- **Composes:** ATOM-613.6-001, ATOM-613.4a-001, ATOM-613.3-001
- **Phase:** Phase 5 Layers (L04, L05, L10)
- **Ticket:** L04, L05, L10

**COMP-613-LAYERS-FULL-STACK-001**
- **Rule:** 613.1a–g — Full layer stack: copy + control + text + type + color + ability + P/T
- **Mechanism:** A single permanent affected by effects in all 7 layers, verifying correct ordering
- **Minimal Board:** A Clone (L1: copies a Bear), stolen (L2: control change), text-changed (L3: color word swap), type-added (L4: becomes enchantment too), color-changed (L5: becomes blue), ability-granted (L6: gains flying), P/T-modified (L7c: +2/+2).
- **Action:** Query final effective characteristics.
- **Expected Result:** All layers applied in order 1–7. Clone is a 4/4 blue enchantment creature Bear with flying, controlled by the stealing player.
- **Composes:** ATOM-613.1a-001 through ATOM-613.1g-001
- **Phase:** Phase 5 Layers + Phase 6
- **Ticket:** L04 through L12

---

## Gap Report

### Mechanisms in roadmap/implementation-plan that SHOULD have tests in this CR section but don't have a single CR sub-rule

| Gap | Description | Recommended Action |
|-----|-------------|-------------------|
| G1 | **Oracle routing migration (L13)** — No CR rule specifically governs "read effective characteristics instead of printed." This is an engineering concern, not a CR rule. | Covered by L13 ticket's own tests (test_sba_uses_effective_toughness, etc.). No ATOM test needed. |
| G2 | **Fuzz regression (L21)** — No CR rule for fuzz testing. | L21 ticket's acceptance criteria. Not a CR test. |
| G3 | **LKI system (L18)** — Rule 608.2h (last known information) is in Session 5's scope (600–608). L18 tests are specified there. | Cross-reference Session 5 for LKI tests. |
| G4 | **Phase 6 execute_action middleware insertion** — The `execute_action` passthrough in `actions.rs` needs replacement effect interception. No single CR rule governs this architectural hook. | Engineering task covered by Phase 6 design, not a CR-specific test. |
| G5 | **Replacement effect registration and matching** — CR 614 defines behavior but not the mechanism of registering/matching. | Engineering infrastructure. Test via behavioral ATOM tests (614.x series). |

### META-101.2 Concrete Tests in This Session

Per the META tracking protocol, the following ATOM tests in this session implement META-101.2 ("can't overrides can"):

| Test ID | Rule | System |
|---------|------|--------|
| ATOM-614.17-001 | 614.17 | Prevention override |
| ATOM-614.17b-001 | 614.17b | Cost payment |
| ATOM-614.17c-001 | 614.17c | Self-replacement exception |
| ATOM-614.17d-001 | 614.17d | ETB "can't" |
| ATOM-615.12-001 | 615.12 | Unpreventable damage |

---

## Classification Summary Table

### PURE-DEF (no test needed)

| Rule | Summary |
|------|---------|
| 609.1 | Definition of "effect" |
| 609.5 | Ties resolved by card text |
| 609.6 | Cross-reference to 614/615 |
| 609.7 | Damage-source effects intro |
| 610.1 | Definition of one-shot effect |
| 610.2 | Delayed triggered abilities intro |
| 611.1 | Definition of continuous effect |
| 611.2 | Continuous effect from spell/ability intro |
| 611.3 | Continuous effect from static ability intro |
| 612.1 | Definition of text-changing effect |
| 612.4 | Token text defined by creator |
| 613.4 | Layer 7 sublayer structure intro |
| 614.1 | Definition of replacement effect |
| 614.2 | Damage-source replacement cross-reference |
| 614.3 | No casting restrictions for replacement generators |
| 614.14 | Linked exile/reference abilities cross-reference |
| 615.1 | Definition of prevention effect |
| 615.2 | Damage-source prevention cross-reference |
| 615.3 | No casting restrictions for prevention generators |
| 616.1e | Fallthrough after priority categories |

### OUT-OF-SCOPE

| Rule | Reason |
|------|--------|
| 612.9 | Name stickers — Un-set/sticker mechanic |
| 613.7h | Plane/phenomenon/scheme cards — Planechase/Archenemy |
| 613.7j | Conspiracy card timestamp — Conspiracy draft |
| 613.7k | Sticker timestamp — Un-set mechanic |

### DEFERRED

| Rule | Target Phase | Reason |
|------|-------------|--------|
| 610.4 | Phase 9 | Phasing |
| 610.4a | Phase 9 | Phasing |
| 610.4b | Phase 9 | Phasing |
| 610.4c | Phase 9 | Phasing |
| 610.4d | Phase 9 | Phasing |
| 611.2f | Phase 7 | "Next spell" continuous effects (D8) |
| 612.5 | Post-v1 | Exchange of Words — single card |
| 612.6 | Post-v1 | Volrath's Shapeshifter — single card |
| 612.7 | Post-v1 | Spy Kit — single card |
| 612.10 | Phase 8 | Splice keyword |
| 613.2b | Phase 8 | Face-down Layer 1b (D3/Morph) |
| 613.7f | Phase 8 | Face-up/face-down timestamp (Morph/Transform) |
| 613.7g | Phase 8 | Transform/convert timestamp |
| 613.7i | Post-v1 | Vanguard stretch goal |
| 614.1e | Phase 8 | "As turned face up" replacement (Morph) |
| 614.12b | Phase 8 | Multiple simultaneous ETB choices |
| 614.12c | Phase 8 | Anchor word ETB choices |
| 614.13c | Phase 8 | ETB mill/exile from library exclusion |
| 615.13 | Phase 7 | Triggered abilities on prevention |
| 616.1d | Phase 8 | Back-face-up ETB replacement priority (Transform) |

### NEW Tickets Identified

| Ticket | Rule(s) | Phase | Description |
|--------|---------|-------|-------------|
| NEW-609.3 | 609.3 | Phase 8 | Partial effect execution for impossible instructions |
| NEW-609.4 | 609.4, 609.4a, 609.4b | Phase 8 | "As though" effect scoping and composition |
| NEW-609.7a | 609.7a | Phase 6 | Damage source choice validation |
| NEW-609.7b | 609.7b | Phase 6 | Prevention shield source property rechecking |
| NEW-609.7c | 609.7c | Phase 6 | Static prevention covers non-battlefield sources |
| NEW-610.3 | 610.3–610.3d | Phase 7 | "Until leaves" zone-change return effects (D9) |
| NEW-610.5 | 610.5 | Phase 7 | Static ability grants at cast time |
| NEW-611.2b | 611.2b | Phase 8 | "For as long as" duration pre-check (D7) |
| NEW-611.2c-mix | 611.2c | Phase 5+6 | Mixed characteristic/rule effect independent sets |
| NEW-611.2e | 611.2e | Phase 7+5 | Simultaneous "is [type]" ETB characteristic |
| NEW-611.3d | 611.3d | Phase 7+5 | Static grant persists after source leaves |
| NEW-613.1a | 613.1a | Phase 6 | Layer 1 copy effect ordering (D1) |
| NEW-613.2 | 613.2 | Phase 8 | Layer 1 sublayer ordering (D3) |
| NEW-613.2a | 613.2a | Phase 6 | Layer 1a copiable effects (D1) |
| NEW-613.2c | 613.2c | Phase 6 | Copiable values post-Layer-1 (D1) |
| NEW-613.7a | 613.7a | Phase 5 | Static ability timestamp = later of object vs grant (D5) |
| NEW-613.7c | 613.7c | Phase 5 | Counter timestamps within L7c (D6) |
| NEW-613.7e | 613.7e | Phase 5 | Aura/Equipment re-timestamp on attach (D4) |
| NEW-613.1f-kw | 613.1f | Phase 5 | Keyword counters in Layer 6 (D10) |
| NEW-614.1a-d | 614.1a–d | Phase 6 | Replacement/prevention effect classification |
| NEW-614.4 | 614.4 | Phase 6 | Replacement timing enforcement |
| NEW-614.5 | 614.5 | Phase 6 | Single-application rule |
| NEW-614.6 | 614.6 | Phase 6+7 | Replaced event suppression + triggers |
| NEW-614.7 | 614.7, 614.7a | Phase 6 | Non-event replacement no-op |
| NEW-614.8 | 614.8 | Phase 6 | Regeneration as destruction-replacement |
| NEW-614.9 | 614.9 | Phase 6 | Damage redirection with invalid destination |
| NEW-614.10 | 614.10, 614.10a, 614.10b | Phase 6 | Skip replacement effects |
| NEW-614.11 | 614.11, 614.11a, 614.11b | Phase 6 | Draw replacement effects |
| NEW-614.12 | 614.12, 614.12a | Phase 6 | ETB look-ahead + choice timing |
| NEW-614.13 | 614.13, 614.13a, 614.13b | Phase 6 | ETB auxiliary zone changes |
| NEW-614.15 | 614.15 | Phase 6 | Self-replacement priority |
| NEW-614.16 | 614.16 | Phase 6 | Token/counter replacement chains |
| NEW-614.17 | 614.17, 614.17a–d | Phase 6 | "Can't" effects (META-101.2) |
| NEW-615.4 | 615.4 | Phase 6 | Prevention timing enforcement |
| NEW-615.5 | 615.5 | Phase 6 | Prevention additional effects |
| NEW-615.6 | 615.6 | Phase 6+7 | Prevented damage trigger suppression |
| NEW-615.7 | 615.7 | Phase 6 | Prevention shield depletion + allocation choice |
| NEW-615.8 | 615.8 | Phase 6 | Instance-based prevention |
| NEW-615.9 | 615.9 | Phase 6 | Prevention source property recheck |
| NEW-615.10 | 615.10 | Phase 6 | Static per-event prevention |
| NEW-615.11 | 615.11 | Phase 6 | Per-creature shield assignment at resolution |
| NEW-615.12 | 615.12, 615.12a | Phase 6 | Unpreventable damage shield preservation |
| NEW-616.1 | 616.1, 616.1a–c, 616.1f–g | Phase 6 | Multiple replacement ordering + priority + iteration |
| NEW-616.2 | 616.2 | Phase 6 | Replacement chaining across event modifications |

---

## Statistics (post-audit)

| Category | Count |
|----------|-------|
| **ATOM tests** | 94 |
| **BOUNDARY-DEF tests** | 7 |
| **COMP tests** | 9 |
| **PURE-DEF** | 20 |
| **OUT-OF-SCOPE** | 4 |
| **DEFERRED** | 20 |
| **META-101.2 concrete tests** | 5 |
| **NEW tickets** | 42 |
| **Total sub-rules processed** | 138 |
| **Audit notes added** | 18 |

> Statistics updated after session-6 audit. +13 ATOM tests, +1 BOUNDARY-DEF test, +3 COMP tests, +4 NEW tickets, +18 audit notes/design references.

