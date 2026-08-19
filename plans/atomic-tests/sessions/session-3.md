# Session 3: Chapter 3 — Card Types (Rules 300–315)

> Generated: 2026-04-03
> CR Source: `MTG-Rules/Chapter 3 - Card Types.txt`
> Cross-references: design_doc.md, roadmap.md, implementation-plan-final.md

## Scope

~120 sub-rules across 16 top-level rule groups (300–315). Covers card type definitions, attachment rules (Equipment, Aura, Fortification), zone behavior, type-specific mechanics (summoning sickness, loyalty, defense counters), and supplemental card types.

**Out-of-scope supplemental types (per roadmap.md):** Planes (311), Phenomena (312), Schemes (314), Conspiracies (315). These are supplemental format types with no relevance to Standard two-player.

**Deferred types:** Dungeons (309) → Phase 8–9 at earliest. Battles (310) → Phase 8–9. Vanguards (313) → stretch goal.

---

## Rules 300.x — General

### 300.1

**Classification: BOUNDARY-DEF**

Rule text: "The card types are artifact, battle, conspiracy, creature, dungeon, enchantment, instant, kindred, land, phenomenon, plane, planeswalker, scheme, sorcery, and vanguard."

**ATOM-300.1-001**
- **Rule:** 300.1 — The engine's CardType enum must contain exactly the 15 card types listed
- **Mechanism:** CardType enum completeness check
- **Minimal Board:** No board state needed — type system validation
- **Action:** Enumerate all CardType variants
- **Expected Result:** All 15 types exist in the enum: Artifact, Battle, Conspiracy, Creature, Dungeon, Enchantment, Instant, Kindred, Land, Phenomenon, Plane, Planeswalker, Scheme, Sorcery, Vanguard. No extra types exist.
- **Phase:** Phase 5 Pre-Work (T07 verified enum completeness)
- **Ticket:** T07 (resolved in-place — E11 enum completeness)

---

### 300.2

**Classification: META**

Rule text: "Some objects have more than one card type (for example, an artifact creature). Such objects combine the aspects of each of those card types, and are subject to spells and abilities that affect either or all of those card types."

This is a cross-cutting rule that affects multiple systems: targeting ("target artifact" must match artifact creatures), SBAs (lethal damage checks apply because it's a creature; equipment legality checks apply because it's an artifact), continuous effects (type-changing effects interact with multi-type objects), and zone guards (an artifact land is still subject to land-play-only restriction per 300.2a). Rather than a single atomic test, multi-type correctness is verified implicitly by the type-checking predicates (`is_creature`, `is_artifact`, `has_card_type`) across all systems that filter by type.

**Systems affected:** Targeting (type filters), SBAs (creature/artifact/enchantment/planeswalker-specific checks), Continuous Effects (L4 type changes on multi-type objects), Combat (creature-only attack/block), Zone Guards (land+other casting restriction).

---

### 300.2a

**Classification: TESTABLE**

Rule text: "An object that's both a land and another card type (for example, an artifact land) can only be played as a land. It can't be cast as a spell."

**ATOM-300.2a-001**
- **Rule:** 300.2a — A land that is also another type (e.g., artifact land) can only be played as a land, not cast as a spell
- **Mechanism:** Cast legality check rejects land+other type cards from being cast
- **Minimal Board:** Player has an artifact land card in hand. Stack is empty, main phase, active player.
- **Action:** Attempt to cast the artifact land as a spell
- **Expected Result:** Cast is rejected — the card can only be played as a land via the land-play action
- **Phase:** Phase 8 (D25 in roadmap — "Land+other-type casting restriction")
- **Ticket:** D25 — Land+other-type casting restriction (300.2a)

**ATOM-300.2a-002**
- **Rule:** 300.2a — A land+other-type card CAN be played as a land
- **Mechanism:** Land play action accepts land+other type cards
- **Minimal Board:** Player has an artifact land in hand. Main phase, active player, land drop available.
- **Action:** Play the artifact land as a land
- **Expected Result:** The artifact land enters the battlefield. Land play count incremented.
- **Phase:** Phase 8 (D25)
- **Ticket:** D25

---

### 300.2b

**Classification: PURE-DEF**

Rule text: "Each kindred card has another card type. Casting and resolving a kindred card follow the rules for casting and resolving a card of the other card type."

This rule is a cross-reference to the kindred (formerly tribal) type system. The casting/resolution behavior is defined by the other card type's rules, not independently. Tested via 308.1.

---

## Rules 301.x — Artifacts

### 301.1

**Classification: ALREADY-IMPLEMENTED**

Rule text: "A player who has priority may cast an artifact card from their hand during a main phase of their turn when the stack is empty."

The casting timing rules for permanent spells (main phase, stack empty, priority) are already implemented in `engine/cast.rs` `check_cast_legality`. Artifact is a permanent type handled by the existing permanent-cast path.

---

### 301.2

**Classification: ALREADY-IMPLEMENTED**

Rule text: "When an artifact spell resolves, its controller puts it onto the battlefield under their control."

Permanent spell resolution is implemented in `engine/stack.rs` — permanent spells resolve to battlefield under controller's control.

---

### 301.3

**Classification: PURE-DEF**

Rule text: "Artifact subtypes are always a single word and are listed after a long dash..."

Subtype format definition. No independent mechanical consequence — the engine's type system handles subtypes as data.

---

### 301.4

**Classification: PURE-DEF**

Rule text: "Artifacts have no characteristics specific to their card type. Most artifacts have no colored mana symbols in their mana costs, and are therefore colorless..."

Informational — no independent testable behavior. Color is determined by mana cost/color indicator, not card type.

---

### 301.5

**Classification: BOUNDARY-DEF**

Rule text: "Some artifacts have the subtype 'Equipment.' An Equipment can be attached to a creature. It can't legally be attached to anything that isn't a creature."

**ATOM-301.5-001**
- **Rule:** 301.5 — Equipment can be attached to a creature
- **Mechanism:** Attachment legality check — Equipment + creature = legal
- **Minimal Board:** One Equipment on battlefield, one creature on battlefield, same controller
- **Action:** Attach Equipment to creature (via equip ability resolution)
- **Expected Result:** Equipment's `attached_to` is set to the creature's ObjectId. Creature's `attached_by` includes the Equipment.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T04 (attachment tracking), T15 (attachment SBAs)

**ATOM-301.5-002**
- **Rule:** 301.5 — Equipment can't legally be attached to a non-creature
- **Mechanism:** Attachment legality check — Equipment + non-creature = illegal
- **Minimal Board:** One Equipment on battlefield, one non-creature artifact on battlefield
- **Action:** An effect attempts to attach the Equipment to the non-creature artifact
- **Expected Result:** The attachment fails (Equipment doesn't move). Per 301.5b: "If an effect attempts to attach an Equipment to an object that can't be equipped by it, the Equipment doesn't move."
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T15 (SBA 704.5p — Equipment attached to non-creature unattaches)

---

### 301.5a

**Classification: PURE-DEF**

Rule text: "The creature an Equipment is attached to is called the 'equipped creature.' The Equipment is attached to, or 'equips,' that creature."

Naming convention — no independent mechanical behavior.

---

### 301.5b

**Classification: TESTABLE (multi-clause)**

Rule text: "Equipment spells are cast like other artifact spells. Equipment enter the battlefield like other artifacts. They don't enter the battlefield attached to a creature. The equip keyword ability attaches the Equipment to a creature you control (see rule 702.6, 'Equip'). Control of the creature matters only when the equip ability is activated and when it resolves. Spells and other abilities may also attach an Equipment to a creature. If an effect attempts to attach an Equipment to an object that can't be equipped by it, the Equipment doesn't move."

**ATOM-301.5b-001**
- **Rule:** 301.5b — Equipment enters the battlefield unattached
- **Mechanism:** Permanent resolution for Equipment subtype
- **Minimal Board:** Equipment spell on the stack, a creature on the battlefield
- **Action:** Equipment spell resolves
- **Expected Result:** Equipment enters battlefield with `attached_to = None`. It is NOT automatically attached to any creature.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T04 (attachment tracking — `attached_to` initialized to None)

**ATOM-301.5b-002**
- **Rule:** 301.5b — Equip ability attaches Equipment to a creature you control
- **Mechanism:** Equip keyword ability resolution
- **Minimal Board:** Equipment on battlefield (controller = P0), creature on battlefield (controller = P0)
- **Action:** Activate equip ability targeting the creature; ability resolves
- **Expected Result:** Equipment's `attached_to` = creature's ObjectId. Creature's `attached_by` includes Equipment.
- **Phase:** Phase 8 (equip keyword ability)
- **Ticket:** NEW — Equip keyword ability implementation

**ATOM-301.5b-003**
- **Rule:** 301.5b — If an effect attempts to attach Equipment to an object it can't equip, the Equipment doesn't move
- **Mechanism:** Attachment legality guard in effect resolution
- **Minimal Board:** Equipment on battlefield attached to creature A. An effect says "attach Equipment to [non-creature permanent]."
- **Action:** Resolve the effect
- **Expected Result:** Equipment remains attached to creature A. No state change occurs.
- **Phase:** Phase 8
- **Ticket:** NEW — Equipment attachment legality guard in effect resolution

---

### 301.5c

**Classification: TESTABLE (multi-clause — 6 distinct behavioral clauses)**

Rule text: "An Equipment that's also a creature can't equip a creature unless that Equipment has reconfigure (see rule 702.151, 'Reconfigure'). An Equipment that loses the subtype 'Equipment' can't equip a creature. An Equipment can't equip itself. An Equipment that equips an illegal or nonexistent permanent becomes unattached from that permanent but remains on the battlefield. (This is a state-based action. See rule 704.) An Equipment can't equip more than one creature. If a spell or ability would cause an Equipment to equip more than one creature, the Equipment's controller chooses which creature it equips."

**ATOM-301.5c-001**
- **Rule:** 301.5c — An Equipment that's also a creature can't equip a creature (unless it has reconfigure)
- **Mechanism:** Equip legality check — Equipment+Creature type combination blocks equip
- **Minimal Board:** A permanent that is both an Equipment and a creature (e.g., animated Equipment via type-changing effect). Another target creature on the battlefield.
- **Action:** Attempt to activate equip ability targeting the other creature
- **Expected Result:** Equip activation is rejected — creature-Equipment can't equip unless it has reconfigure
- **Phase:** Phase 8 (requires L4 type-changing effects to animate Equipment)
- **Ticket:** NEW — Equipment-creature equip restriction

**ATOM-301.5c-002**
- **Rule:** 301.5c — An Equipment that loses the subtype "Equipment" can't equip a creature
- **Mechanism:** SBA or equip legality check — missing Equipment subtype blocks equip
- **Minimal Board:** A permanent that was an Equipment but had its Equipment subtype removed by a continuous effect. It is currently attached to a creature.
- **Action:** SBA check runs
- **Expected Result:** The permanent becomes unattached from the creature (SBA 704.5q — non-Equipment non-Aura attached → unattach). It remains on the battlefield.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T15 (SBA 704.5q)

**ATOM-301.5c-003**
- **Rule:** 301.5c — An Equipment can't equip itself
- **Mechanism:** Equip target legality — self-targeting blocked
- **Minimal Board:** An Equipment that is also a creature (e.g., via reconfigure or animation)
- **Action:** Attempt to target itself with its equip ability
- **Expected Result:** Targeting is rejected — Equipment can't equip itself
- **Phase:** Phase 8
- **Ticket:** NEW — Equipment self-equip prevention

**ATOM-301.5c-004**
- **Rule:** 301.5c — An Equipment that equips an illegal or nonexistent permanent becomes unattached but remains on the battlefield (SBA)
- **Mechanism:** SBA check — Equipment attached to illegal/nonexistent permanent
- **Minimal Board:** Equipment attached to a creature. The creature is destroyed (goes to graveyard).
- **Action:** SBA check runs after creature destruction
- **Expected Result:** Equipment's `attached_to` is set to None. Equipment remains on the battlefield.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T15 (SBA 704.5p — Equipment attached to non-creature unattaches) + T04 (zone-exit detachment in cleanup_zone_state)

**ATOM-301.5c-005**
- **Rule:** 301.5c — An Equipment can't equip more than one creature
- **Mechanism:** Equip resolution moves Equipment from old host to new host
- **Minimal Board:** Equipment attached to creature A. Player activates equip targeting creature B.
- **Action:** Equip ability resolves
- **Expected Result:** Equipment is now attached to creature B only. Creature A's `attached_by` no longer includes Equipment. Equipment's `attached_to` = creature B.
- **Phase:** Phase 8
- **Ticket:** NEW — Equip moves Equipment between creatures

**ATOM-301.5c-006** — REMOVED (audit: no known effect causes simultaneous multi-equip; can't compose from primitives; not worth explicit test)

---

### 301.5d

**Classification: TESTABLE (multi-clause)**

Rule text: "An Equipment's controller is separate from the equipped creature's controller; the two need not be the same. Changing control of the creature doesn't change control of the Equipment, and vice versa. Only the Equipment's controller can activate its abilities. However, if the Equipment grants an ability to the equipped creature (with 'gains' or 'has'), the equipped creature's controller is the only one who can activate that ability."

**ATOM-301.5d-001**
- **Rule:** 301.5d — Changing control of the equipped creature doesn't change control of the Equipment
- **Mechanism:** Control change independence — Equipment and creature have separate controllers
- **Minimal Board:** P0 controls Equipment attached to creature. P0 controls creature. An effect changes control of the creature to P1.
- **Action:** Resolve control change effect
- **Expected Result:** Creature's controller = P1. Equipment's controller = P0 (unchanged). Equipment remains attached.
- **Phase:** Phase 5 Layers (L11 — control-changing effects)
- **Ticket:** L11

**ATOM-301.5d-002**
- **Rule:** 301.5d — Only the Equipment's controller can activate its abilities
- **Mechanism:** Ability activation permission check
- **Minimal Board:** P0 controls Equipment attached to P1's creature. Equipment has an activated ability.
- **Action:** P1 attempts to activate the Equipment's ability
- **Expected Result:** Activation rejected — only P0 (Equipment's controller) can activate Equipment abilities
- **Phase:** Phase 8
- **Ticket:** NEW — Equipment ability activation controller check

**ATOM-301.5d-003**
- **Rule:** 301.5d — If Equipment grants ability to equipped creature, the creature's controller activates it
- **Mechanism:** Granted ability activation permission
- **Minimal Board:** P0 controls Equipment with "Equipped creature has '{T}: Deal 1 damage to any target'". Equipment attached to P1's creature.
- **Action:** P1 attempts to activate the granted ability on their creature
- **Expected Result:** Activation is legal — P1 (creature's controller) can activate the granted ability
- **Phase:** Phase 8 (granted abilities via Equipment)
- **Ticket:** NEW — Equipment-granted ability controller routing

---

### 301.5e

**Classification: TESTABLE**

Rule text: "If an effect attempts to put an Equipment that isn't also an Aura (see rule 303.4i) onto the battlefield attached to either an object it can't legally equip or an object that is undefined, the Equipment enters the battlefield unattached. If the Equipment is a token, it's created and enters the battlefield unattached."

**ATOM-301.5e-001**
- **Rule:** 301.5e — Equipment entering battlefield via effect attached to illegal target enters unattached
- **Mechanism:** ETB attachment guard for Equipment
- **Minimal Board:** An effect would put an Equipment onto the battlefield attached to a non-creature permanent
- **Action:** Resolve the effect
- **Expected Result:** Equipment enters the battlefield with `attached_to = None` (unattached). It is on the battlefield.
- **Phase:** Phase 8
- **Ticket:** NEW — Equipment ETB illegal-attachment guard

---

### 301.5f

**Classification: PURE-DEF**

Rule text: "An ability of a permanent that refers to the 'equipped creature' refers to whatever creature that permanent is attached to, even if the permanent with the ability isn't an Equipment."

Naming convention for effect resolution. The engine resolves "equipped creature" references by reading `attached_to` on the source permanent. No independent testable behavior beyond the attachment system.

---

### 301.6

**Classification: TESTABLE**

Rule text: "Some artifacts have the subtype 'Fortification.' A Fortification can be attached to a land. It can't legally be attached to an object that isn't a land. Fortification's analog to the equip keyword ability is the fortify keyword ability. Rules 301.5a–f apply to Fortifications in relation to lands just as they apply to Equipment in relation to creatures, with one clarification relating to rule 301.5c: a Fortification that's also a creature (not a land) can't fortify a land."

**ATOM-301.6-001**
- **Rule:** 301.6 — Fortification can be attached to a land, not to non-lands
- **Mechanism:** Attachment legality for Fortification subtype
- **Minimal Board:** One Fortification on battlefield, one land on battlefield, one creature on battlefield
- **Action:** Attempt to attach Fortification to the land (legal) and to the creature (illegal)
- **Expected Result:** Fortification attaches to land successfully. Attachment to creature is rejected.
- **Phase:** Phase 8–9 (Fortification is a very niche subtype)
- **Ticket:** NEW — Fortification attachment rules

---

### 301.7

**Classification: PURE-DEF**

Rule text: "Some artifacts have the subtype 'Vehicle.' Most Vehicles have a crew ability which allows them to become artifact creatures."

Introductory definition for Vehicles. The crew mechanic is defined in rule 702.122. No independent testable behavior here.

---

### 301.7a

**Classification: TESTABLE**

Rule text: "Each Vehicle has a printed power and toughness, but it has these characteristics only if it's also a creature. See rule 208.3."

**ATOM-301.7a-001**
- **Rule:** 301.7a — A Vehicle that is not a creature does not have power/toughness as characteristics
- **Mechanism:** Characteristic query for non-creature Vehicle
- **Minimal Board:** One Vehicle artifact on battlefield (not crewed, not a creature)
- **Action:** Query `get_effective_power` and `get_effective_toughness`
- **Expected Result:** The Vehicle has no P/T characteristics (returns None or is excluded from creature-only queries). It should NOT be treated as a creature for damage, combat, or SBA purposes.
- **Phase:** Phase 8 (Vehicles/crew)
- **Ticket:** NEW — Vehicle P/T only when creature

**ATOM-301.7a-002**
- **Rule:** 301.7a — A Vehicle that IS a creature has its printed power and toughness
- **Mechanism:** Characteristic query for crewed Vehicle
- **Minimal Board:** One Vehicle artifact creature on battlefield (crewed this turn)
- **Action:** Query `get_effective_power` and `get_effective_toughness`
- **Expected Result:** Returns the Vehicle's printed P/T values (possibly modified by continuous effects)
- **Phase:** Phase 8
- **Ticket:** NEW — Vehicle P/T when crewed

---

### 301.7b

**Classification: TESTABLE**

Rule text: "If a Vehicle becomes a creature, it immediately has its printed power and toughness. Other effects, including the effect that makes it a creature, may modify these values or set them to different values."

**ATOM-301.7b-001**
- **Rule:** 301.7b — Vehicle becoming a creature gets printed P/T, then modifications apply
- **Mechanism:** Layer system interaction — crew effect (L4 type change) + printed P/T + other P/T effects
- **Minimal Board:** Vehicle with printed 4/4 on battlefield. A +1/+1 anthem effect active ("Creatures you control get +1/+1").
- **Action:** Crew the Vehicle (it becomes an artifact creature)
- **Expected Result:** Vehicle is now 5/5 (4/4 base + 1/1 from anthem). The L4 type-changing effect (adding Creature type) happens before L7 P/T calculation, so the anthem applies.
- **Phase:** Phase 8 (requires crew + layer system)
- **Ticket:** NEW — Vehicle P/T interaction with continuous effects

---

## Rules 302.x — Creatures

### 302.1

**Classification: ALREADY-IMPLEMENTED**

Rule text: "A player who has priority may cast a creature card from their hand during a main phase of their turn when the stack is empty."

Creature casting timing is implemented in `engine/cast.rs` `check_cast_legality`. Creature is a permanent type handled by the existing permanent-cast path.

---

### 302.2

**Classification: ALREADY-IMPLEMENTED**

Rule text: "When a creature spell resolves, its controller puts it onto the battlefield under their control."

Permanent spell resolution implemented in `engine/stack.rs`. Controller is set from `StackEntry.controller` per Phase 3 audit fix.

---

### 302.3

**Classification: PURE-DEF**

Rule text: "Creature subtypes are usually a single word long and are listed after a long dash..."

Subtype format definition with an example. No independent mechanical consequence.

---

### 302.4

**Classification: BOUNDARY-DEF**

Rule text: "Power and toughness are characteristics only creatures have."

**ATOM-302.4-001**
- **Rule:** 302.4 — Non-creature objects do not have power/toughness
- **Mechanism:** Characteristic query — non-creature objects should not have P/T
- **Minimal Board:** One non-creature artifact on the battlefield (with no P/T printed).
- **Action:** Query effective P/T for the artifact
- **Expected Result:** Non-creature artifact returns None / is excluded from P/T queries. (Cross-ref 301.7a for Vehicles as a special case.)
- **Phase:** Phase 5 Layers (EffectiveCharacteristics struct)
- **Ticket:** L04 (layer engine — EffectiveCharacteristics)

**ATOM-302.4-002**
- **Rule:** 302.4 — Creatures have power and toughness as characteristics
- **Mechanism:** Characteristic query — creature returns valid P/T
- **Minimal Board:** Grizzly Bears (2/2) on the battlefield
- **Action:** Query effective P/T for Bears
- **Expected Result:** Power = 2, Toughness = 2. Creature has P/T as a characteristic.
- **Phase:** Phase 5 Layers (EffectiveCharacteristics struct)
- **Ticket:** L04

---

### 302.4a

**Classification: PURE-DEF**

Rule text: "A creature's power is the amount of damage it deals in combat."

This is defined by the combat damage system (rule 510). The combat damage resolution in `engine/combat/resolution.rs` already uses `get_effective_power` for damage amounts. Tested in combat sessions (Sessions 4–5).

---

### 302.4b

**Classification: PURE-DEF**

Rule text: "A creature's toughness is the amount of damage needed to destroy it."

Defined by SBA 704.5g (lethal damage). Already implemented in `engine/sba.rs`.

---

### 302.4c

**Classification: TESTABLE**

Rule text: "To determine a creature's power and toughness, start with the numbers printed in its lower right corner, then apply any applicable continuous effects. (See rule 613, 'Interaction of Continuous Effects.')"

**ATOM-302.4c-001**
- **Rule:** 302.4c — Creature P/T starts from printed values, then continuous effects apply
- **Mechanism:** Layer system P/T computation — base + modifications
- **Minimal Board:** Grizzly Bears (2/2) on battlefield. Giant Growth resolving targeting Bears (+3/+3 until end of turn).
- **Action:** Compute effective P/T after Giant Growth resolves
- **Expected Result:** Bears are 5/5 (2/2 base + 3/3 from L7c modification)
- **Phase:** Phase 5 Layers
- **Ticket:** L07 (P/T sublayers), L08 (Giant Growth card)

---

### 302.5

**Classification: ALREADY-IMPLEMENTED**

Rule text: "Creatures can attack and block."

Combat system is fully implemented (Phase 3). `engine/combat/validation.rs` validates attackers and blockers. Only creatures can attack and block per `validate_attackers` and `validate_blockers`.

---

### 302.6

**Classification: ALREADY-IMPLEMENTED**

Rule text: "A creature's activated ability with the tap symbol or the untap symbol in its activation cost can't be activated unless the creature has been under its controller's control continuously since their most recent turn began. A creature can't attack unless it has been under its controller's control continuously since their most recent turn began."

Summoning sickness is implemented. Currently as `summoning_sick: bool` flag (will be `controller_since_turn` per T09). Haste bypass implemented in Phase 4. `can_attack` in `oracle/legality.rs` and `Cost::Tap` / `Cost::Untap` in `engine/costs.rs` both check summoning sickness.

**Note:** T09 (controller_since_turn rework) refines the implementation but the behavior is already covered.

---

### 302.7

**Classification: ALREADY-IMPLEMENTED**

Rule text: "Damage dealt to a creature by a source with neither wither nor infect is marked on that creature... If the total damage marked on that creature is greater than or equal to its toughness, that creature has been dealt lethal damage and is destroyed as a state-based action... All damage marked on a creature is removed... during the cleanup step."

Damage marking, lethal damage SBA (704.5g), and cleanup damage removal are all implemented. `engine/sba.rs` checks damage vs toughness. `engine/turns.rs` cleanup step removes damage. Wither/infect routing is in T21c (Phase 5 Pre-Work).

---

## Rules 303.x — Enchantments

### 303.1

**Classification: ALREADY-IMPLEMENTED**

Rule text: "A player who has priority may cast an enchantment card from their hand during a main phase of their turn when the stack is empty."

Same permanent casting pipeline as artifacts/creatures. Already implemented.

---

### 303.2

**Classification: ALREADY-IMPLEMENTED**

Rule text: "When an enchantment spell resolves, its controller puts it onto the battlefield under their control."

Standard permanent resolution path in `engine/stack.rs`.

---

### 303.3

**Classification: PURE-DEF**

Rule text: "Enchantment subtypes are always a single word and are listed after a long dash..."

Subtype format definition. No independent mechanical consequence.

---

### 303.4

**Classification: TESTABLE**

Rule text: "Some enchantments have the subtype 'Aura.' An Aura enters the battlefield attached to an object or player. What an Aura can be attached to is defined by its enchant keyword ability (see rule 702.5, 'Enchant'). Other effects can limit what a permanent can be enchanted by."

**ATOM-303.4-001**
- **Rule:** 303.4 — An Aura enters the battlefield attached to an object or player
- **Mechanism:** Aura spell resolution attaches to target
- **Minimal Board:** Aura spell on stack with "Enchant creature," targeting a creature
- **Action:** Aura spell resolves
- **Expected Result:** Aura enters battlefield with `attached_to` = target creature's ObjectId. Creature's `attached_by` includes the Aura.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T15b (Aura attachment logic — rule 303.4f)

**ATOM-303.4-002**
- **Rule:** 303.4 — An Aura with "Enchant player" enters the battlefield attached to a player
- **Mechanism:** Aura spell resolution attaches to target player (not a game object)
- **Minimal Board:** Aura spell on stack with "Enchant player" (e.g., Curse of Misfortunes), targeting P1
- **Action:** Aura spell resolves
- **Expected Result:** Aura enters battlefield with `attached_to` referencing P1 (player, not a permanent). The attachment system must handle player targets, not just ObjectIds. This requires `AttachTarget` to support both `Object(ObjectId)` and `Player(PlayerId)`.
- **Phase:** Phase 8 (player-targeting Auras)
- **Ticket:** NEW — AttachTarget enum supporting Player variant for Enchant player Auras

---

### 303.4a

**Classification: TESTABLE**

Rule text: "An Aura spell requires a target, which is defined by its enchant ability."

**ATOM-303.4a-001**
- **Rule:** 303.4a — An Aura spell requires a target defined by its enchant ability
- **Mechanism:** Targeting validation during casting — Aura must have legal target
- **Minimal Board:** Aura with "Enchant creature" in hand. No creatures on battlefield.
- **Action:** Attempt to cast the Aura
- **Expected Result:** Cast fails — no legal targets available. (If on stack and all targets become illegal, it fizzles per rule 608.2b.)
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T15b (EnchantRestriction targeting model)

---

### 303.4b

**Classification: PURE-DEF**

Rule text: "The object or player an Aura is attached to is called enchanted. The Aura is attached to, or 'enchants,' that object or player."

Naming convention — no independent mechanical behavior. Same pattern as 301.5a for Equipment.

---

### 303.4c

**Classification: TESTABLE**

Rule text: "If an Aura is enchanting an illegal object or player as defined by its enchant ability and other applicable effects, the object it was attached to no longer exists, or the player it was attached to has left the game, the Aura is put into its owner's graveyard. (This is a state-based action. See rule 704.)"

**ATOM-303.4c-001**
- **Rule:** 303.4c — Aura enchanting an illegal object is put into graveyard (SBA)
- **Mechanism:** SBA check — Aura attached to object that no longer satisfies enchant restriction
- **Minimal Board:** Aura with "Enchant creature" attached to a permanent. A continuous effect removes the Creature type from the enchanted permanent (L4 type change).
- **Action:** SBA check runs
- **Expected Result:** Aura is put into its owner's graveyard. The permanent is no longer a legal host.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T15 (SBA 704.5m/704.5n — Aura legality)

**ATOM-303.4c-002**
- **Rule:** 303.4c — Aura whose host no longer exists is put into graveyard (SBA)
- **Mechanism:** SBA check — Aura attached to destroyed/exiled permanent
- **Minimal Board:** Aura attached to a creature. The creature is destroyed.
- **Action:** SBA check runs after creature goes to graveyard
- **Expected Result:** Aura's host no longer exists on battlefield. Aura is put into its owner's graveyard.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T15 (SBA 704.5m — unattached Aura) + T04 (zone-exit detachment)

**ATOM-303.4c-003**
- **Rule:** 303.4c — Aura attached to a player who has left the game is put into graveyard (SBA)
- **Mechanism:** SBA check — enchanted player has lost/left the game
- **Minimal Board:** Multiplayer game. P0 controls an Aura with "Enchant player" attached to P2. P2 loses the game and leaves.
- **Action:** SBA check runs after P2 leaves
- **Expected Result:** Aura is put into its owner's (P0's) graveyard. The attached-to player no longer exists in the game.
- **Phase:** Phase 9 (multiplayer support)
- **Ticket:** NEW — Aura falls off when enchanted player leaves game

---

### 303.4d

**Classification: TESTABLE (multi-clause — 3 distinct behavioral clauses)**

Rule text: "An Aura can't enchant itself. If this occurs somehow, the Aura is put into its owner's graveyard. An Aura that's also a creature can't enchant anything. If this occurs somehow, the Aura becomes unattached, then is put into its owner's graveyard. (These are state-based actions. See rule 704.) An Aura can't enchant more than one object or player. If a spell or ability would cause an Aura to become attached to more than one object or player, the Aura's controller chooses which object or player it becomes attached to."

**ATOM-303.4d-001**
- **Rule:** 303.4d — An Aura can't enchant itself; if so, it goes to graveyard (SBA)
- **Mechanism:** SBA check — self-enchantment
- **Minimal Board:** An Aura somehow has `attached_to` pointing to itself
- **Action:** SBA check runs
- **Expected Result:** Aura is put into its owner's graveyard
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T15 (Aura legality SBAs)

**ATOM-303.4d-002**
- **Rule:** 303.4d — An Aura that's also a creature can't enchant anything; if so, it becomes unattached then goes to graveyard (SBA)
- **Mechanism:** SBA check — Aura+Creature type combination
- **Minimal Board:** An Aura attached to a permanent. A continuous effect (e.g., Opalescence-like) adds the Creature type to the Aura.
- **Action:** SBA check runs
- **Expected Result:** Aura becomes unattached from its host, then is put into its owner's graveyard. (Two-step: unattach, then graveyard.)
- **Phase:** Phase 8 (requires L4 type-changing + Aura cards coexisting — see T15 TODO comment)
- **Ticket:** T15 (TODO comment documents this SBA: "704.5r — An Aura that is also a creature can't enchant anything")

**ATOM-303.4d-003** — REMOVED (audit: analogous to 301.5c-006; no known effect causes simultaneous multi-enchant; can't compose from primitives)

---

### 303.4e

**Classification: TESTABLE (multi-clause)**

Rule text: "An Aura's controller is separate from the enchanted object's controller or the enchanted player; the two need not be the same. If an Aura enchants an object, changing control of the object doesn't change control of the Aura, and vice versa. Only the Aura's controller can activate its abilities. However, if the Aura grants an ability to the enchanted object (with 'gains' or 'has'), the enchanted object's controller is the only one who can activate that ability."

**ATOM-303.4e-001**
- **Rule:** 303.4e — Changing control of enchanted object doesn't change control of Aura
- **Mechanism:** Control change independence — Aura and enchanted object have separate controllers
- **Minimal Board:** P0 controls Aura attached to P0's creature. An effect changes the creature's controller to P1.
- **Action:** Resolve control change
- **Expected Result:** Creature's controller = P1. Aura's controller = P0 (unchanged). Aura remains attached.
- **Phase:** Phase 5 Layers (L11 — control-changing effects)
- **Ticket:** L11

**ATOM-303.4e-002**
- **Rule:** 303.4e — Only the Aura's controller can activate its abilities; granted abilities are activated by enchanted object's controller
- **Mechanism:** Ability activation permission routing for Aura-granted abilities
- **Minimal Board:** P0 controls Aura with "Enchanted creature has '{T}: Draw a card'". Aura attached to P1's creature.
- **Action:** P1 attempts to activate the granted ability
- **Expected Result:** Legal — P1 (creature's controller) can activate the granted ability. P0 cannot activate the granted ability (it's on P1's creature).
- **Phase:** Phase 8 (granted abilities via Auras)
- **Ticket:** NEW — Aura-granted ability controller routing

**ATOM-303.4e-003**
- **Rule:** 303.4e — Aura cast targeting opponent's permanent: caster is controller of the Aura
- **Mechanism:** Aura controller assignment on ETB — the caster controls the Aura even though it enters attached to an opponent's permanent
- **Minimal Board:** P0 casts Pacifism (Aura with "Enchant creature") targeting P1's creature. Pacifism spell resolves.
- **Action:** Check controller of the Aura on the battlefield
- **Expected Result:** Pacifism's controller = P0 (the caster). Enchanted creature's controller = P1. The Aura enters already attached to P1's creature, but P0 controls it. This is the standard casting pipeline — `StackEntry.controller` is the caster — so this should be covered by the existing permanent ETB controller logic (rule 110.2, Phase 3 audit fix). **Note:** This is arguably already tested by the general permanent resolution controller test, but an Aura-specific regression test is worthwhile given the non-obvious interaction.
- **Phase:** Phase 5 Pre-Work (T15b)
- **Ticket:** T15b (Aura attachment logic)

---

### 303.4f

**Classification: TESTABLE**

Rule text: "If an Aura is entering the battlefield under a player's control by any means other than by resolving as an Aura spell, and the effect putting it onto the battlefield doesn't specify the object or player the Aura will enchant, that player chooses what it will enchant as the Aura enters the battlefield. The player must choose a legal object or player according to the Aura's enchant ability and any other applicable effects."

**ATOM-303.4f-001**
- **Rule:** 303.4f — Aura entering battlefield not from stack: controller chooses what to enchant
- **Mechanism:** Non-stack Aura ETB — controller selects legal enchant target (not targeting — hexproof/shroud don't apply)
- **Minimal Board:** Aura with "Enchant creature" in graveyard. An effect returns it to the battlefield. Opponent has a hexproof creature.
- **Action:** Resolve the effect. Controller chooses the hexproof creature.
- **Expected Result:** Aura enters attached to the hexproof creature. This is legal because 303.4f doesn't target — hexproof/shroud don't apply to non-targeting enchantment placement.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T15b (Aura ETB without targeting — rule 303.4a)

---

### 303.4g

**Classification: TESTABLE**

Rule text: "If an Aura is entering the battlefield and there is no legal object or player for it to enchant, the Aura remains in its current zone, unless that zone is the stack. In that case, the Aura is put into its owner's graveyard instead of entering the battlefield. If the Aura is a token, it isn't created."

**ATOM-303.4g-001**
- **Rule:** 303.4g — Aura entering from non-stack zone with no legal enchant target stays in current zone
- **Mechanism:** Aura ETB guard — no legal target from non-stack zone
- **Minimal Board:** Aura with "Enchant creature" in graveyard. No creatures on battlefield.
- **Action:** An effect attempts to return the Aura to the battlefield
- **Expected Result:** Aura remains in the graveyard (its current zone). It does not enter the battlefield.
- **Phase:** Phase 5 Pre-Work / Phase 8
- **Ticket:** T15b (Aura ETB without legal host)

**ATOM-303.4g-002**
- **Rule:** 303.4g — Aura resolving from stack with no legal target goes to graveyard
- **Mechanism:** Aura spell fizzle — no legal targets remain
- **Minimal Board:** Aura spell on stack targeting a creature. The creature is destroyed in response.
- **Action:** Aura spell resolves; target is no longer legal
- **Expected Result:** Aura spell fizzles (all targets illegal) and is put into its owner's graveyard. (Standard fizzle rule 608.2b applies.)
- **Phase:** Already implemented (fizzle handling in stack.rs)
- **Ticket:** ALREADY-IMPLEMENTED (spell fizzle)
- **Audit note:** This is covered by general fizzle logic (608.2b) already implemented in `stack.rs`. Keeping as an Aura-specific regression test since Auras are the most common single-target spell type and the fizzle→graveyard path is the same mechanism that 303.4g's stack-zone clause describes.

**ATOM-303.4g-003**
- **Rule:** 303.4g — Aura token with no legal target is not created
- **Mechanism:** Token creation guard — Aura token with no legal enchant target
- **Minimal Board:** An effect would create an Aura token with "Enchant creature." No creatures on battlefield.
- **Action:** Resolve the token creation effect
- **Expected Result:** The token is not created. No Aura enters the battlefield.
- **Phase:** Phase 8 (token creation)
- **Ticket:** NEW — Aura token creation guard

---

### 303.4h

**Classification: TESTABLE**

Rule text: "If an effect attempts to put a permanent that isn't an Aura, Equipment, or Fortification onto the battlefield attached to an object or player, it enters the battlefield unattached."

**ATOM-303.4h-001**
- **Rule:** 303.4h — Non-Aura/Equipment/Fortification permanent entering attached → enters unattached
- **Mechanism:** ETB attachment guard for non-attachment permanents
- **Minimal Board:** An effect attempts to put an enchantment (not an Aura) onto the battlefield "attached to" a creature
- **Action:** Resolve the effect
- **Expected Result:** The enchantment enters the battlefield unattached. `attached_to = None`.
- **Phase:** Phase 8
- **Ticket:** NEW — Non-attachment permanent ETB unattached guard

---

### 303.4i

**Classification: TESTABLE**

Rule text: "If an effect attempts to put an Aura onto the battlefield attached to either an object or player it can't legally enchant or an object or player that is undefined, the Aura remains in its current zone, unless that zone is the stack. In that case, the Aura is put into its owner's graveyard instead of entering the battlefield. If the Aura is a token, it isn't created."

**ATOM-303.4i-001**
- **Rule:** 303.4i — Aura entering battlefield from non-stack zone attached to illegal target: stays in current zone
- **Mechanism:** Aura ETB illegal-attachment guard (non-stack zone)
- **Minimal Board:** Aura with "Enchant creature" in graveyard. An effect puts it onto the battlefield attached to a non-creature permanent.
- **Action:** Resolve the effect
- **Expected Result:** Aura remains in graveyard (current zone) — does not enter battlefield
- **Phase:** Phase 8
- **Ticket:** NEW — Aura ETB illegal-attachment guard (from non-stack zone)

**ATOM-303.4i-002**
- **Rule:** 303.4i — Aura entering battlefield from stack attached to illegal target: goes to graveyard
- **Mechanism:** Aura ETB illegal-attachment guard (stack zone)
- **Minimal Board:** An effect on the stack would put an Aura with "Enchant creature" onto the battlefield attached to a non-creature permanent. (Note: this differs from normal fizzle — here the Aura isn't targeting; the effect specifies the attachment. Example: an effect says "return target Aura card from your graveyard to the battlefield attached to target permanent" where the permanent is illegal for the Aura.)
- **Action:** Resolve the effect
- **Expected Result:** Aura is put into its owner's graveyard instead of entering the battlefield
- **Phase:** Phase 8
- **Ticket:** NEW — Aura ETB illegal-attachment guard (from stack)

**ATOM-303.4i-003**
- **Rule:** 303.4i — Aura token entering battlefield attached to illegal target is not created
- **Mechanism:** Aura token creation guard with illegal host
- **Minimal Board:** An effect creates an Aura token attached to a non-creature permanent; Aura has "Enchant creature"
- **Action:** Resolve the effect
- **Expected Result:** Token is not created
- **Phase:** Phase 8
- **Ticket:** NEW — Aura token illegal-host guard

---

### 303.4j

**Classification: TESTABLE**

Rule text: "If an effect attempts to attach an Aura on the battlefield to an object or player it can't legally enchant, the Aura doesn't move."

**ATOM-303.4j-001**
- **Rule:** 303.4j — Effect attempting to move Aura to illegal enchant target: Aura doesn't move
- **Mechanism:** Re-attachment legality guard
- **Minimal Board:** Aura with "Enchant creature" attached to creature A. An effect says "attach this Aura to [non-creature permanent]."
- **Action:** Resolve the effect
- **Expected Result:** Aura remains attached to creature A. No state change.
- **Phase:** Phase 8
- **Ticket:** NEW — Aura re-attachment legality guard

---

### 303.4k

**Classification: DEFERRED**

Rule text: "If an effect allows an Aura that's being turned face up to become attached to an object or player..."

Face-down permanents are deferred to Phase 9 (D2 — morph/manifest/disguise). This rule only applies to face-down Auras being turned face up.

---

### 303.4m

**Classification: PURE-DEF**

Rule text: "An ability of a permanent that refers to the 'enchanted [object or player]' refers to whatever object or player that permanent is attached to, even if the permanent with the ability isn't an Aura."

Naming convention for effect resolution — same pattern as 301.5f. The engine resolves "enchanted creature" references by reading `attached_to` on the source permanent. No independent testable behavior beyond the attachment system.

**Audit note:** 301.5f says "equipped creature" and this rule says "enchanted [object or player]." In the engine, both resolve identically — they read `attached_to` on the source permanent. The semantic difference ("equipped" implies Equipment subtype, "enchanted" implies Aura subtype) doesn't matter mechanically because both rules explicitly say "even if the permanent with the ability isn't an Equipment/Aura." So the resolution is always just `source.attached_to`. No separate test needed for the semantic distinction.

---

### 303.5

**Classification: PURE-DEF**

Rule text: "Some enchantments have the subtype 'Saga.' See rule 714 for more information about Saga cards."

Cross-reference to rule 714. Saga mechanics will be tested when Saga rules are covered. No independent behavior here.

---

### 303.6

**Classification: PURE-DEF**

Rule text: "Some enchantments have the subtype 'Class.' See rule 716 for more information about Class cards."

Cross-reference to rule 716. No independent behavior here.

---

### 303.7

**Classification: PURE-DEF**

Rule text: "Some Aura enchantments also have the subtype 'Role.'"

Introductory definition for Roles. The mechanical behavior is in 303.7a.

---

### 303.7a

**Classification: TESTABLE**

Rule text: "If a permanent has more than one Role controlled by the same player attached to it, each of those Roles except the one with the most recent timestamp is put into its owner's graveyard. This is a state-based action. See rule 704."

**ATOM-303.7a-001**
- **Rule:** 303.7a — Multiple Roles from same controller on same permanent: keep newest, SBA removes rest
- **Mechanism:** SBA check — Role uniqueness per (permanent, controller) pair
- **Minimal Board:** A creature with two Role Auras attached, both controlled by P0. Role A has timestamp 5, Role B has timestamp 8.
- **Action:** SBA check runs
- **Expected Result:** Role A (older timestamp) is put into its owner's graveyard. Role B (newer timestamp) remains attached.
- **Phase:** Phase 8 (Roles are a specific Aura subtype)
- **Ticket:** NEW — SBA for Role uniqueness (704.x)

**ATOM-303.7a-002**
- **Rule:** 303.7a — Roles from different controllers on same permanent coexist
- **Mechanism:** SBA check — Role uniqueness is per-controller
- **Minimal Board:** A creature with two Role Auras attached: one controlled by P0, one controlled by P1
- **Action:** SBA check runs
- **Expected Result:** Both Roles remain attached. The uniqueness rule only applies within a single controller's Roles.
- **Phase:** Phase 8
- **Ticket:** NEW — SBA for Role uniqueness (different controllers OK)

---

## Rules 304.x — Instants

### 304.1

**Classification: ALREADY-IMPLEMENTED**

Rule text: "A player who has priority may cast an instant card from their hand. Casting an instant as a spell uses the stack. (See rule 601, 'Casting Spells.')"

Instant casting is implemented in `engine/cast.rs` — instants can be cast any time a player has priority (no main-phase/stack-empty restriction). This is the core timing distinction from sorcery-speed spells.

---

### 304.2

**Classification: ALREADY-IMPLEMENTED**

Rule text: "When an instant spell resolves, the actions stated in its rules text are followed. Then it's put into its owner's graveyard."

Instant/sorcery resolution is implemented in `engine/stack.rs` — non-permanent spells resolve their effects and go to graveyard.

---

### 304.3

**Classification: PURE-DEF**

Rule text: "Instant subtypes are always a single word and are listed after a long dash: 'Instant — Arcane.' Each word after the dash is a separate subtype. The set of instant subtypes is the same as the set of sorcery subtypes; these subtypes are called spell types."

Subtype format definition. No independent mechanical consequence.

---

### 304.4

**Classification: TESTABLE**

Rule text: "Instants can't enter the battlefield. If an instant would enter the battlefield, it remains in its previous zone instead."

**ATOM-304.4-001**
- **Rule:** 304.4 — An instant can't enter the battlefield; it remains in its previous zone
- **Mechanism:** Zone transition guard — reject instant-to-battlefield moves
- **Minimal Board:** An instant card in the graveyard. An effect attempts to return it to the battlefield.
- **Action:** Resolve the effect
- **Expected Result:** The instant remains in the graveyard. It does not enter the battlefield.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T21a (Instant/Sorcery battlefield guard — E34)

---

### 304.5

**Classification: TESTABLE**

Rule text: "If text states that a player may do something 'any time they could cast an instant' or 'only as an instant,' it means only that the player must have priority. The player doesn't need to have an instant card they could cast. Effects that would preclude that player from casting an instant spell don't affect the player's capability to perform that action (unless the action is actually casting an instant spell)."

**ATOM-304.5-001**
- **Rule:** 304.5 — "Any time you could cast an instant" means only "you have priority"
- **Mechanism:** Timing check for instant-speed actions — priority only, no card-in-hand requirement
- **Minimal Board:** Player has priority. An activated ability says "Activate only as an instant." Player has no instant cards in hand.
- **Action:** Attempt to activate the ability
- **Expected Result:** Activation is legal — the player has priority. Having no instant cards in hand is irrelevant.
- **Phase:** Phase 5 Pre-Work (T19 — activation restrictions)
- **Ticket:** T19 (ActivationRestriction framework — "instant speed" is the default for activated abilities)

**ATOM-304.5-002**
- **Rule:** 304.5 — Effect precluding instant casting doesn't affect "as an instant" actions
- **Mechanism:** Casting prohibition vs timing-permission independence
- **Minimal Board:** Player has priority. A continuous effect says "Players can't cast instant spells" (e.g., hypothetical Teferi-like effect). Player has an ability that says "Activate only as an instant."
- **Action:** Attempt to activate the ability
- **Expected Result:** Activation is legal — the "can't cast instants" prohibition doesn't affect abilities that use instant-timing, only actual instant spell casting.
- **Phase:** Phase 5 Layers / Phase 8 (restriction effects)
- **Ticket:** NEW — "As an instant" timing independence from casting prohibitions

**Audit note — Flash implementation:** Flash (702.8) is a static ability: "You may cast this spell any time you could cast an instant." Per rule 304.5, that means "you have priority" — it removes the sorcery-speed restriction.

**Current state** (`cast.rs:242-245`): `card_data.keywords.contains(&KeywordAbility::Flash)` is checked directly on the card data. If the card has flash or is an instant, it skips sorcery-speed timing checks. This is correct for printed flash on cards in hand (confirmed by Ch7 audit F7/C6).

**Phase 5 change (D17):** Replace `card_data.keywords.contains(Flash)` with `has_keyword(game, obj_id, Flash)` so the layer system is consulted. This enables continuous effects that grant flash (Leyline of Anticipation, Teferi +1, Vivien Champion of the Wilds) to work. Flash is NOT a `CastPermission` (unlike Flashback/Cascade which change the *zone* you cast from) — it's a timing override, and the existing `if !is_instant && !has_flash` branch in `check_cast_legality` already does the right thing. The only change is the keyword query path.

**Interaction with casting restrictions:**
1. "Can't cast instant spells" (e.g., Eidolon of Rhetoric's timing cousin) restricts by card type, not timing. A creature with flash is not an instant — so "can't cast instant spells" doesn't affect it. The timing check and the type-restriction check are independent.
2. "Each opponent can cast spells only any time they could cast a sorcery" (Teferi, Time Raveler) imposes a sorcery-speed restriction on the *player*, overriding flash on individual cards. This is 101.2 ("can't" beats "can"). Implementation: this will be a continuous effect that sets a player-level `CastingRestriction::SorcerySpeedOnly` flag, checked in `check_cast_legality` *after* the per-card timing check. Even if the card has flash, the player-level restriction blocks it.

---

## Rules 305.x — Lands

### 305.1

**Classification: ALREADY-IMPLEMENTED**

Rule text: "A player who has priority may play a land card from their hand during a main phase of their turn when the stack is empty. Playing a land is a special action; it doesn't use the stack... Rather, the player simply puts the land onto the battlefield."

Land play is implemented in `engine/zones.rs` `play_land` with timing guards (main phase, active player, stack empty). Land enters battlefield directly without using the stack.

---

### 305.2

**Classification: TESTABLE**

Rule text: "A player can normally play one land during their turn; however, continuous effects may increase this number."

**ATOM-305.2-001**
- **Rule:** 305.2 — Default land-per-turn limit is 1; can be increased by continuous effects
- **Mechanism:** Land play counter + effective lands-per-turn query
- **Minimal Board:** Player has played 0 lands this turn. Two lands in hand. `get_effective_lands_per_turn` returns 1 (default).
- **Action:** Play first land (succeeds). Attempt to play second land.
- **Expected Result:** First land play succeeds. Second land play is rejected (1 land played ≥ 1 allowed).
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T22 step 5 (lands_per_turn dynamic — E45)

**ATOM-305.2-002**
- **Rule:** 305.2 — Continuous effect increases land-per-turn limit
- **Mechanism:** Effective lands-per-turn oracle query increased by continuous effect
- **Minimal Board:** Player has played 1 land this turn. A continuous effect (e.g., Exploration) grants +1 land per turn. `get_effective_lands_per_turn` returns 2.
- **Action:** Attempt to play second land
- **Expected Result:** Second land play succeeds (1 land played < 2 allowed).
- **Phase:** Phase 5 Layers (L15 — post-layer pass computes effective lands_per_turn)
- **Ticket:** L15

**Audit note — Integration test needed:** Verify that land drop increases from continuous effects like Exploration interact correctly with per-turn land count resets. Scenario: Player controls Exploration (extra land per turn) and takes an extra turn. Turn 1: casts Exploration, plays 2 lands (valid: limit 2). Turn 2 (extra turn): `lands_played_this_turn` resets to 0 at turn start, but Exploration is still on the battlefield so the limit is still 2. Player plays 1 land (valid). Player attempts to play a 2nd land (valid — limit is still 2). Now suppose Exploration is destroyed between turns — on the next turn, the limit drops back to 1 and only 1 land can be played. This integration test belongs in the L15 / T22 test suite. Note that Exploration's effect is static (continuous) not "until end of turn," so it persists as long as Exploration is on the battlefield. The per-turn reset is just `lands_played_this_turn = 0` in `engine/turns.rs` at the start of each turn.

---

### 305.2a

**Classification: TESTABLE**

Rule text: "To determine whether a player can play a land, compare the number of lands the player can play this turn with the number of lands they have already played this turn (including lands played as special actions and lands played during the resolution of spells and abilities). If the number of lands the player can play is greater, the play is legal."

**ATOM-305.2a-001**
- **Rule:** 305.2a — Lands played during spell/ability resolution count toward the per-turn limit
- **Mechanism:** Land play counter incremented by effect-based land plays, not just special-action plays
- **Minimal Board:** Player has played 1 land this turn (via special action). An effect says "you may play an additional land this turn" (setting effective limit to 2). Another effect says "put a land from your hand onto the battlefield" (which counts as playing a land per some effects, but see 305.4 — "put" is NOT "play").
- **Action:** After the "put" effect resolves, attempt to play a land as a special action
- **Expected Result:** Per 305.4, "put onto the battlefield" ≠ "play a land." The put doesn't count. Player can still play their second land.
- **Phase:** Phase 5 Pre-Work / Phase 5 Layers
- **Ticket:** T22 step 5 (lands_per_turn dynamic)

---

### 305.2b

**Classification: TESTABLE**

Rule text: "A player can't play a land, for any reason, if the number of lands the player can play this turn is equal to or less than the number of lands they have already played this turn. Ignore any part of an effect that instructs a player to do so."

**ATOM-305.2b-001**
- **Rule:** 305.2b — Player can't play a land if at or over their limit, even if an effect says to
- **Mechanism:** Hard cap on land plays — effect instructions to play lands are ignored when at limit
- **Minimal Board:** Player has played 1 land this turn. Effective limit is 1. An effect says "You may play lands from among [exiled cards]" (e.g., impulse draw like Light Up the Stage).
- **Action:** Player attempts to play a land from the exiled cards
- **Expected Result:** Land play is rejected. The impulse draw effect says "you may play" but the hard cap (305.2b) says you can't — the instruction to play is ignored. The land remains in exile.
- **Phase:** Phase 5 Pre-Work / Phase 8
- **Ticket:** T22 step 5 + NEW — Effect land-play instruction ignored at limit

**Audit note:** This may be structurally satisfied by the architecture if all land-play paths go through the same `play_land` function which checks `lands_played_this_turn >= effective_lands_per_turn`. The impulse draw effect grants a *permission* to play from exile (a `CastPermission` zone override), but the land-play count check is upstream and universal. As long as no code path bypasses `play_land`'s count check, this rule is enforced structurally. Still worth an explicit test to prevent regression.

---

### 305.3

**Classification: TESTABLE**

Rule text: "A player can't play a land, for any reason, if it isn't their turn. Ignore any part of an effect that instructs a player to do so."

**ATOM-305.3-001**
- **Rule:** 305.3 — A player can't play a land if it isn't their turn
- **Mechanism:** Land play timing guard — active player only
- **Minimal Board:** It is P0's turn. P1 has a land in hand.
- **Action:** P1 attempts to play a land
- **Expected Result:** Land play is rejected — it's not P1's turn.
- **Phase:** Already implemented (play_land timing guards in engine/zones.rs)
- **Ticket:** ALREADY-IMPLEMENTED

---

### 305.4

**Classification: TESTABLE**

Rule text: "Effects may also allow players to 'put' lands onto the battlefield. This isn't the same as 'playing a land' and doesn't count as a land played during the current turn."

**ATOM-305.4-001**
- **Rule:** 305.4 — "Put a land onto the battlefield" does not count as a land played this turn
- **Mechanism:** Land play counter NOT incremented by "put" effects
- **Minimal Board:** Player has played 0 lands this turn. An effect "puts" a land from their hand onto the battlefield.
- **Action:** After the effect resolves, check `lands_played_this_turn`
- **Expected Result:** `lands_played_this_turn` is still 0. The "put" effect did not count. Player can still play their normal land for the turn.
- **Phase:** Phase 8 (when "put land onto battlefield" primitives exist)
- **Ticket:** NEW — "Put land" vs "play land" counter distinction

---

### 305.5

**Classification: PURE-DEF**

Rule text: "Land subtypes are always a single word and are listed after a long dash. Land subtypes are also called land types."

Subtype format definition. No independent mechanical consequence.

---

### 305.6

**Classification: TESTABLE**

Rule text: "The basic land types are Plains, Island, Swamp, Mountain, and Forest. If an object uses the words 'basic land type,' it's referring to one of these subtypes. An object with the land card type and a basic land type has the intrinsic ability '{T}: Add [mana symbol],' even if the text box doesn't actually contain that text or the object has no text box. For Plains, [mana symbol] is {W}; for Islands, {U}; for Swamps, {B}; for Mountains, {R}; and for Forests, {G}."

**ATOM-305.6-001**
- **Rule:** 305.6 — A land with a basic land type has the intrinsic mana ability for that type
- **Mechanism:** Intrinsic mana ability generation from basic land types
- **Minimal Board:** A Mountain on the battlefield (has basic land type Mountain)
- **Action:** Activate the Mountain's mana ability
- **Expected Result:** Produces {R}. The ability exists even if not printed in the card's text box.
- **Phase:** Already implemented (basic lands in cards/basic_lands.rs)
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-305.6-002**
- **Rule:** 305.6 — A non-basic land that gains a basic land type gains the corresponding mana ability
- **Mechanism:** Intrinsic mana ability addition when basic land type is added
- **Minimal Board:** A nonswamp land (e.g., a Mountain) on the battlefield. Urborg, Tomb of Yawgmoth is also on the battlefield with continuous effect "Each land is a Swamp in addition to its other land types."
- **Action:** Query the Mountain's types and abilities
- **Expected Result:** The Mountain is now a Basic Land - Mountain Swamp and now has both "{T}: Add {R}" (intrinsic from Mountain type, which it already had) and "{T}: Add {B}" (intrinsic from the newly-gained Swamp type). Urborg uses "in addition to" so old types/abilities are kept (305.7 additive clause), making this a cleaner test of 305.6 than Blood Moon which also strips abilities via the replacement clause of 305.7.
- **Phase:** Phase 5 Layers
- **Ticket:** L17

**ATOM-305.6-003**
- **Rule:** 305.6 — Boundary: "basic land type" means exactly Plains, Island, Swamp, Mountain, Forest
- **Mechanism:** Basic land type enumeration — other land subtypes (Desert, Gate, Lair, etc.) are NOT basic land types
- **Minimal Board:** A land with subtype "Desert" on the battlefield
- **Action:** Check if the land has an intrinsic mana ability from its subtype
- **Expected Result:** No intrinsic mana ability — Desert is not a basic land type. Only the five listed types grant intrinsic abilities.
- **Phase:** Phase 5 Layers (L17 — basic land type handling)
- **Ticket:** L17

---

### 305.7

**Classification: TESTABLE (multi-clause — critical for Blood Moon/L17)**

Rule text: "If an effect sets a land's subtype to one or more of the basic land types, the land no longer has its old land type. It loses all abilities generated from its rules text, its old land types, and any copiable effects affecting that land, and it gains the appropriate mana ability for each new basic land type. Note that this doesn't remove any abilities that were granted to the land by other effects. Setting a land's subtype doesn't add or remove any card types (such as creature) or supertypes (such as basic, legendary, and snow) the land may have. If a land gains one or more land types in addition to its own, it keeps its land types and rules text, and it gains the new land types and mana abilities."

**ATOM-305.7-001**
- **Rule:** 305.7 — Setting a land's subtype to a basic land type removes old land types
- **Mechanism:** L4 type-changing effect — "set" removes old subtypes
- **Minimal Board:** A nonbasic land with subtype "Lair" on the battlefield. A continuous effect sets its land subtypes to "Mountain."
- **Action:** Compute effective subtypes
- **Expected Result:** Land has subtype Mountain only. Lair is removed.
- **Phase:** Phase 5 Layers (L17)
- **Ticket:** L17

**ATOM-305.7-002**
- **Rule:** 305.7 — Setting to basic land type removes abilities from rules text and old land types
- **Mechanism:** L3/L4 interaction — lose rules-text abilities + gain intrinsic mana ability
- **Minimal Board:** A nonbasic land with "{T}: Add {U} or {B}" on the battlefield. Blood Moon (or equivalent effect) sets its subtypes to Mountain.
- **Action:** Compute effective abilities
- **Expected Result:** Land loses its printed "{T}: Add {U} or {B}" ability. Gains intrinsic "{T}: Add {R}" from Mountain type. Old mana abilities are gone.
- **Phase:** Phase 5 Layers (L17)
- **Ticket:** L17

**ATOM-305.7-003**
- **Rule:** 305.7 — Setting to basic land type does NOT remove abilities granted by other effects
- **Mechanism:** Granted ability persistence through type-setting
- **Minimal Board:** A nonbasic land on battlefield. A *separate* permanent (e.g., an enchantment like Training Grounds or a hypothetical enchantment with "Lands you control have '{T}: Draw a card'") grants the land "{T}: Draw a card." Blood Moon is also on the battlefield, setting the land's subtypes to Mountain.
- **Action:** Compute effective abilities
- **Expected Result:** Land has both the granted "{T}: Draw a card" ability AND the intrinsic "{T}: Add {R}" from Mountain. The granted ability was NOT generated from the land's rules text nor from its old land types — it was granted by an external source, so it survives Blood Moon's stripping. The land's own printed abilities are lost.
- **Phase:** Phase 5 Layers (L17)
- **Ticket:** L17

**ATOM-305.7-004**
- **Rule:** 305.7 — Setting subtypes doesn't change card types or supertypes
- **Mechanism:** L4 type-setting scope — only land subtypes change, not types/supertypes
- **Minimal Board:** A legendary nonbasic land on the battlefield. Blood Moon sets its subtypes to Mountain.
- **Action:** Compute effective types and supertypes
- **Expected Result:** Land retains its card type "Land" and supertype "Legendary." It does NOT gain supertype "Basic." It does NOT lose any card types.
- **Phase:** Phase 5 Layers (L17)
- **Ticket:** L17

**ATOM-305.7-005**
- **Rule:** 305.7 — "Gains in addition to" keeps old land types and rules text, adds new types + mana abilities
- **Mechanism:** L4 type-adding effect — additive, not replacing
- **Minimal Board:** A Forest on the battlefield. An effect says "this land is a Plains in addition to its other types."
- **Action:** Compute effective subtypes and abilities
- **Expected Result:** Land has both Forest and Plains subtypes. It has both "{T}: Add {G}" (from Forest) and "{T}: Add {W}" (from Plains). Original rules text is preserved.
- **Phase:** Phase 5 Layers
- **Ticket:** L17 / NEW — "in addition to" land type grant

---

### 305.8

**Classification: BOUNDARY-DEF**

Rule text: "Any land with the supertype 'basic' is a basic land. Any land that doesn't have this supertype is a nonbasic land, even if it has a basic land type."

**ATOM-305.8-001**
- **Rule:** 305.8 — A land with a basic land type but without the "basic" supertype is nonbasic
- **Mechanism:** Basic/nonbasic determination by supertype, not subtype
- **Minimal Board:** A nonbasic land with subtype Mountain (e.g., a dual land). It does NOT have the "Basic" supertype. A Blood Moon effect has set its subtypes to Mountain.
- **Action:** Query: is this land a basic land?
- **Expected Result:** No — it is a nonbasic land. Having the Mountain subtype does not make it basic. Only the "Basic" supertype makes a land basic. This matters for effects like "destroy all nonbasic lands."
- **Phase:** Phase 5 Layers (L17 — Blood Moon doesn't add Basic supertype)
- **Ticket:** L17

**ATOM-305.8-002**
- **Rule:** 305.8 — A land with the "Basic" supertype IS a basic land
- **Mechanism:** Basic/nonbasic determination — positive case
- **Minimal Board:** A basic Mountain on the battlefield (has both "Basic" supertype and "Mountain" subtype)
- **Action:** Query: is this land a basic land?
- **Expected Result:** Yes — it has the "Basic" supertype. This is what makes it basic, not the Mountain subtype.
- **Phase:** Already implemented (basic lands have Basic supertype in card data)
- **Ticket:** ALREADY-IMPLEMENTED

---

### 305.9

**Classification: DUPLICATE — see 300.2a**

Rule text: "If an object is both a land and another card type, it can be played only as a land. It can't be cast as a spell."

Identical to 300.2a. Tests ATOM-300.2a-001 and ATOM-300.2a-002 cover this rule. No separate test needed.

---

## Rules 306.x — Planeswalkers

### 306.1

**Classification: ALREADY-IMPLEMENTED**

Rule text: "A player who has priority may cast a planeswalker card from their hand during a main phase of their turn when the stack is empty. Casting a planeswalker as a spell uses the stack. (See rule 601, 'Casting Spells.')"

Planeswalker casting follows the standard permanent casting pipeline in `engine/cast.rs`. Same timing as creatures/artifacts/enchantments.

---

### 306.2

**Classification: ALREADY-IMPLEMENTED**

Rule text: "When a planeswalker spell resolves, its controller puts it onto the battlefield under their control."

Standard permanent resolution path in `engine/stack.rs`.

---

### 306.3

**Classification: PURE-DEF**

Rule text: "Planeswalker subtypes are always a single word and are listed after a long dash: 'Planeswalker — Jace.'"

Subtype format definition. No independent mechanical consequence.

---

### 306.4

**Classification: PURE-DEF**

Rule text: "Previously, planeswalkers were subject to a 'planeswalker uniqueness rule' that stopped a player from controlling two planeswalkers of the same planeswalker type. This rule has been removed and planeswalker cards printed before this change have received errata in the Oracle card reference to have the legendary supertype. Like other legendary permanents, they are subject to the 'legend rule' (see rule 704.5j)."

Historical note. The uniqueness rule is obsolete. Planeswalkers are now legendary and subject to the legend rule, which is covered by T14 (legend rule SBA). No independent testable behavior beyond 704.5j.

---

### 306.5

**Classification: BOUNDARY-DEF**

Rule text: "Loyalty is a characteristic only planeswalkers have."

**ATOM-306.5-001**
- **Rule:** 306.5 — Loyalty is a characteristic only planeswalkers have
- **Mechanism:** Characteristic query — non-planeswalker objects should not have loyalty
- **Minimal Board:** One creature and one planeswalker on the battlefield
- **Action:** Query loyalty for both objects
- **Expected Result:** Planeswalker has a loyalty value (equal to its loyalty counters per 306.5c). Creature does not have loyalty as a characteristic.
- **Phase:** Phase 5 Pre-Work (T14 — planeswalker loyalty SBA)
- **Ticket:** T14

---

### 306.5a

**Classification: TESTABLE**

Rule text: "The loyalty of a planeswalker card not on the battlefield is equal to the number printed in its lower right corner."

**ATOM-306.5a-001**
- **Rule:** 306.5a — Planeswalker card not on battlefield has loyalty equal to its printed number
- **Mechanism:** Loyalty characteristic in non-battlefield zones
- **Minimal Board:** A planeswalker card in hand with printed loyalty 4
- **Action:** Query the card's loyalty
- **Expected Result:** Loyalty = 4 (the printed number). This matters for effects that check loyalty of cards in graveyards, etc.
- **Phase:** Phase 5 Pre-Work / Phase 5 Layers (EffectiveCharacteristics)
- **Ticket:** T14 / L04

---

### 306.5b

**Classification: TESTABLE**

Rule text: "A planeswalker has the intrinsic ability 'This permanent enters with a number of loyalty counters on it equal to its printed loyalty number.' This ability creates a replacement effect (see rule 614.1c)."

**ATOM-306.5b-001**
- **Rule:** 306.5b — Planeswalker enters the battlefield with loyalty counters equal to its printed loyalty
- **Mechanism:** Planeswalker ETB sets initial loyalty counters
- **Minimal Board:** A planeswalker spell with printed loyalty 4 on the stack
- **Action:** Planeswalker spell resolves, enters battlefield
- **Expected Result:** Planeswalker has 4 loyalty counters on it (`counters[Loyalty] == 4`)
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T14 (step 4 — planeswalker ETB sets initial loyalty counters from card_data.loyalty)

---

### 306.5c

**Classification: TESTABLE**

Rule text: "The loyalty of a planeswalker on the battlefield is equal to the number of loyalty counters on it."

**ATOM-306.5c-001**
- **Rule:** 306.5c — On-battlefield planeswalker loyalty = number of loyalty counters
- **Mechanism:** Loyalty characteristic derived from counter count, not printed value
- **Minimal Board:** Planeswalker on battlefield with 3 loyalty counters (originally entered with 4, one removed by damage)
- **Action:** Query the planeswalker's loyalty
- **Expected Result:** Loyalty = 3 (from counters, not from printed value)
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T14
- **Audit note:** This test's setup is a sub-step of ATOM-306.8-001 (damage removes loyalty counters). ATOM-306.8-001 inherently verifies 306.5c because after dealing 3 damage to a PW with 5 loyalty, the expected result (2 loyalty) only works if loyalty = counter count. Keeping this as a separate test for atomicity — 306.8 tests the *damage routing*, while 306.5c tests the *characteristic derivation* from counters. In implementation, they can share setup code.

---

### 306.5d

**Classification: TESTABLE (multi-clause)**

Rule text: "Each planeswalker has a number of loyalty abilities, which are activated abilities with loyalty symbols in their costs. Loyalty abilities follow special rules: A player may activate a loyalty ability of a permanent they control any time they have priority and the stack is empty during a main phase of their turn, but only if none of that permanent's loyalty abilities have been activated that turn. See rule 606, 'Loyalty Abilities.'"

**ATOM-306.5d-001**
- **Rule:** 306.5d — Loyalty ability can be activated at sorcery speed (main phase, stack empty, priority)
- **Mechanism:** Loyalty ability activation timing restriction
- **Minimal Board:** Planeswalker on battlefield controlled by active player. Main phase, stack empty.
- **Action:** Activate a loyalty ability (e.g., +1 ability)
- **Expected Result:** Activation succeeds. Loyalty counters are adjusted per the cost.
- **Phase:** Phase 5 Pre-Work (T19 — activation restrictions: SorcerySpeed)
- **Ticket:** T19 (ActivationRestriction::SorcerySpeed)

**ATOM-306.5d-002**
- **Rule:** 306.5d — Only one loyalty ability per planeswalker per turn
- **Mechanism:** Per-turn activation tracking for loyalty abilities
- **Minimal Board:** Planeswalker on battlefield. Player has already activated one of its loyalty abilities this turn.
- **Action:** Attempt to activate another loyalty ability on the same planeswalker
- **Expected Result:** Activation is rejected — a loyalty ability of this permanent has already been activated this turn
- **Phase:** Phase 5 Pre-Work (T19 — ActivationRestriction::OncePerTurn)
- **Ticket:** T19

**ATOM-306.5d-003**
- **Rule:** 306.5d — Loyalty ability cannot be activated during opponent's turn or with non-empty stack
- **Mechanism:** Loyalty ability timing — sorcery speed only
- **Minimal Board:** Planeswalker on battlefield controlled by P0. It is P1's turn (or stack is non-empty).
- **Action:** P0 attempts to activate a loyalty ability
- **Expected Result:** Activation is rejected — not main phase of P0's turn and/or stack not empty
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T19

---

### 306.6

**Classification: TESTABLE**

Rule text: "Planeswalkers can be attacked. (See rule 508, 'Declare Attackers Step.')"

**ATOM-306.6-001**
- **Rule:** 306.6 — Planeswalkers can be attacked
- **Mechanism:** Attack target validation accepts planeswalkers
- **Minimal Board:** P0 controls a creature. P1 controls a planeswalker.
- **Action:** P0 declares their creature as attacking P1's planeswalker
- **Expected Result:** The attack declaration is legal. The creature is attacking the planeswalker.
- **Phase:** Phase 5 Pre-Work / Phase 8 (planeswalker attack targeting)
- **Ticket:** NEW — Planeswalker as attack target in validate_attackers

---

### 306.7

**Classification: PURE-DEF**

Rule text: "Previously, planeswalkers were subject to a redirection effect that allowed a player to have noncombat damage that would be dealt to an opponent be dealt to a planeswalker under that opponent's control instead. This rule has been removed..."

Historical note. The old "redirect to planeswalker" rule no longer exists. Cards that previously relied on this interaction have received Oracle errata to deal damage directly to planeswalkers. No testable behavior.

---

### 306.8

**Classification: TESTABLE**

Rule text: "Damage dealt to a planeswalker results in that many loyalty counters being removed from it."

**ATOM-306.8-001**
- **Rule:** 306.8 — Damage to planeswalker removes loyalty counters
- **Mechanism:** Damage routing — damage to PW removes loyalty counters instead of marking damage
- **Minimal Board:** Planeswalker with 5 loyalty counters on battlefield. A source deals 3 damage to it.
- **Action:** Resolve the damage
- **Expected Result:** Planeswalker now has 2 loyalty counters. Damage is not "marked" — counters are removed.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T21c (Planeswalker damage routing — E40)

**ATOM-306.8-002**
- **Rule:** 306.8 — Excess damage to planeswalker is absorbed (no overflow)
- **Mechanism:** Damage routing — damage exceeding loyalty doesn't overflow
- **Minimal Board:** Planeswalker with 3 loyalty counters. A source deals 6 damage to it.
- **Action:** Resolve the damage
- **Expected Result:** Planeswalker has 0 loyalty counters. The excess 3 damage is absorbed — it doesn't overflow to the controller or anywhere else. `min(damage, current_loyalty)` counters removed.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T21c (step 2 — PW damage overflow absorbed)

---

### 306.9

**Classification: TESTABLE**

Rule text: "If a planeswalker's loyalty is 0, it's put into its owner's graveyard. (This is a state-based action. See rule 704.)"

**ATOM-306.9-001**
- **Rule:** 306.9 — Planeswalker with 0 loyalty goes to graveyard (SBA)
- **Mechanism:** SBA check — planeswalker with 0 loyalty counters
- **Minimal Board:** Planeswalker on battlefield with 0 loyalty counters
- **Action:** SBA check runs
- **Expected Result:** Planeswalker is put into its owner's graveyard
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T14 (SBA 704.5i — planeswalker 0 loyalty)

**ATOM-306.9-002**
- **Rule:** 306.9 — Planeswalker with >0 loyalty stays on battlefield
- **Mechanism:** SBA check — planeswalker with positive loyalty is fine
- **Minimal Board:** Planeswalker on battlefield with 1 loyalty counter
- **Action:** SBA check runs
- **Expected Result:** Planeswalker remains on the battlefield. No SBA action taken.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T14

---

## Rules 307.x — Sorceries

### 307.1

**Classification: ALREADY-IMPLEMENTED**

Rule text: "A player who has priority may cast a sorcery card from their hand during a main phase of their turn when the stack is empty. Casting a sorcery as a spell uses the stack. (See rule 601, 'Casting Spells.')"

Sorcery casting timing (sorcery speed: main phase, stack empty, active player) is implemented in `engine/cast.rs` `check_cast_legality`.

---

### 307.2

**Classification: ALREADY-IMPLEMENTED**

Rule text: "When a sorcery spell resolves, the actions stated in its rules text are followed. Then it's put into its owner's graveyard."

Sorcery resolution is implemented in `engine/stack.rs` — non-permanent spells resolve effects then go to graveyard.

---

### 307.3

**Classification: PURE-DEF**

Rule text: "Sorcery subtypes are always a single word and are listed after a long dash: 'Sorcery — Arcane.' Each word after the dash is a separate subtype. The set of sorcery subtypes is the same as the set of instant subtypes; these subtypes are called spell types."

Subtype format definition. No independent mechanical consequence.

---

### 307.4

**Classification: TESTABLE**

Rule text: "Sorceries can't enter the battlefield. If a sorcery would enter the battlefield, it remains in its previous zone instead."

**ATOM-307.4-001**
- **Rule:** 307.4 — A sorcery can't enter the battlefield; it remains in its previous zone
- **Mechanism:** Zone transition guard — reject sorcery-to-battlefield moves
- **Minimal Board:** A sorcery card in the graveyard. An effect attempts to return it to the battlefield.
- **Action:** Resolve the effect
- **Expected Result:** The sorcery remains in the graveyard. It does not enter the battlefield.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T21a (Instant/Sorcery battlefield guard — E34)

---

### 307.5

**Classification: TESTABLE**

Rule text: "If a spell, ability, or effect states that a player can do something only 'any time they could cast a sorcery' or 'only as a sorcery,' it means only that the player must have priority, it must be during the main phase of their turn, and the stack must be empty. The player doesn't need to have a sorcery card they could cast. Effects that would preclude that player from casting a sorcery spell don't affect the player's capability to perform that action (unless the action is actually casting a sorcery spell)."

**ATOM-307.5-001**
- **Rule:** 307.5 — "Any time you could cast a sorcery" means priority + main phase + stack empty
- **Mechanism:** Sorcery-speed timing check for non-spell actions
- **Minimal Board:** Player has priority during their main phase, stack is empty. An activated ability says "Activate only as a sorcery." Player has no sorcery cards in hand.
- **Action:** Attempt to activate the ability
- **Expected Result:** Activation is legal — the three conditions (priority, main phase, stack empty) are met. Having no sorcery cards is irrelevant.
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T19 (ActivationRestriction::SorcerySpeed)

**ATOM-307.5-002**
- **Rule:** 307.5 — "Only as a sorcery" ability cannot be activated during opponent's turn
- **Mechanism:** Sorcery-speed timing rejection
- **Minimal Board:** It is P1's turn. P0 has an activated ability with "Activate only as a sorcery."
- **Action:** P0 attempts to activate the ability
- **Expected Result:** Activation is rejected — it's not P0's main phase
- **Phase:** Phase 5 Pre-Work
- **Ticket:** T19

**ATOM-307.5-003**
- **Rule:** 307.5 — Effect precluding sorcery casting doesn't affect "as a sorcery" actions
- **Mechanism:** Casting prohibition vs timing-permission independence
- **Minimal Board:** Player has priority during main phase, stack empty. A continuous effect says "Players can't cast sorcery spells." Player has an ability that says "Activate only as a sorcery."
- **Action:** Attempt to activate the ability
- **Expected Result:** Activation is legal — the "can't cast sorceries" prohibition only affects actual sorcery spell casting, not abilities using sorcery timing.
- **Phase:** Phase 5 Layers / Phase 8
- **Ticket:** NEW — "As a sorcery" timing independence from casting prohibitions

**ATOM-307.5-004**
- **Rule:** 307.5 — "Each opponent can cast spells only any time they could cast a sorcery" (Teferi, Time Raveler-style restriction)
- **Mechanism:** Casting restriction that overrides flash and instant-speed casting for opponents
- **Minimal Board:** P0 controls a permanent with "Each opponent can cast spells only any time they could cast a sorcery." It is P1's main phase, stack is empty. P1 has an instant in hand.
- **Action:** P1 attempts to cast the instant (at sorcery speed — main phase, stack empty, their turn)
- **Expected Result:** Cast is legal — P1 meets the sorcery-speed timing requirements. The restriction doesn't prevent casting entirely, it restricts *when* opponents can cast.
- **Phase:** Phase 5 Layers / Phase 8
- **Ticket:** NEW — Opponent sorcery-speed restriction (Teferi-style)

**ATOM-307.5-005**
- **Rule:** 307.5 — Teferi-style restriction prevents opponent from casting at instant speed
- **Mechanism:** Casting restriction rejects opponent casts during non-sorcery-speed windows
- **Minimal Board:** P0 controls "Each opponent can cast spells only any time they could cast a sorcery." It is P0's turn (or P1's turn but during combat). P1 has an instant in hand and has priority.
- **Action:** P1 attempts to cast the instant
- **Expected Result:** Cast is rejected — P1 can only cast spells at sorcery speed, and the current timing window is not sorcery speed for P1 (not P1's main phase, or stack is non-empty, etc.). Flash doesn't help — the restriction overrides the permission per 101.2 ("can't" beats "can").
- **Phase:** Phase 5 Layers / Phase 8
- **Ticket:** NEW — Opponent sorcery-speed restriction enforcement

---

### 307.5a

**Classification: TESTABLE**

Rule text: "Similarly, if an effect checks to see if a spell was cast 'any time a sorcery couldn't have been cast,' it's checking only whether the spell's controller cast it without having priority, during a phase other than their main phase, or while another object was on the stack."

**ATOM-307.5a-001**
- **Rule:** 307.5a — "Couldn't have been cast as a sorcery" checks only timing conditions
- **Mechanism:** Retroactive timing check — was the spell cast at non-sorcery-speed timing?
- **Minimal Board:** Player casts an instant spell during their opponent's turn (not their main phase, stack may be non-empty). An effect checks if the spell was cast "any time a sorcery couldn't have been cast."
- **Action:** Evaluate the check
- **Expected Result:** The check returns true — the spell was cast during a phase other than the caster's main phase. Effects that prevent sorcery casting (like "can't cast sorceries") are irrelevant to this check.
- **Phase:** Phase 8 (when cards with this check exist, e.g., Necromancy)
- **Ticket:** NEW — Retroactive sorcery-timing check

**Audit note — Scope of "any time a sorcery couldn't have been cast" cards:** The few cards that use this check (notably Necromancy) are all *self-referential* — they only care about their own cast timing, not an arbitrary spell's timing. Necromancy reads: "You may cast this spell as though it had flash. If you cast it any time a sorcery couldn't have been cast, the controller of the permanent it becomes sacrifices it at the beginning of the next cleanup step." This means we only need to store a `cast_at_non_sorcery_speed: bool` flag on the spell/permanent itself (set at cast time based on the three timing conditions), not a general system that tracks arbitrary spell cast timings retroactively. This significantly simplifies implementation. No card exists (or is likely to exist) that checks another spell's cast timing retroactively — the mechanic is too cumbersome with Miracle, Madness, Cascade, etc. modifying cast timing in complex ways.

**Implementation note:** Store `was_cast_at_non_sorcery_speed` in `CastInfo` (from T21a). Set it in `check_cast_legality` by checking: `!(is_active_player && is_main_phase && stack_is_empty)`. Effects on the permanent can then read this flag.

---

## Rules 308.x — Kindreds

### 308.1

**Classification: TESTABLE**

Rule text: "Each kindred card has another card type. Casting and resolving a kindred card follows the rules for casting and resolving a card of the other card type."

**ATOM-308.1-001**
- **Rule:** 308.1 — Kindred card follows the casting/resolution rules of its other card type
- **Mechanism:** Cast legality and resolution dispatch based on non-kindred card type
- **Minimal Board:** A kindred enchantment card in hand (e.g., "Kindred Enchantment — Merfolk"). Player has priority, main phase, stack empty.
- **Action:** Cast the kindred enchantment
- **Expected Result:** The card is cast as an enchantment (sorcery-speed timing). When it resolves, it enters the battlefield as a permanent (enchantment resolution rules). The kindred type itself doesn't change casting or resolution behavior.
- **Phase:** Phase 8 (when kindred cards are implemented)
- **Ticket:** NEW — Kindred card type casting dispatch

---

### 308.2

**Classification: PURE-DEF**

Rule text: "Kindred subtypes are usually a single word long and are listed after a long dash... The set of kindred subtypes is the same as the set of creature subtypes; these subtypes are called creature types."

Subtype format definition. The engine already handles creature types as a subtype category.

---

### 308.3

**Classification: PURE-DEF**

Rule text: "Some older kindred cards were printed with the 'tribal' card type. Cards printed with that type have received errata in the Oracle card reference."

Historical note about the tribal→kindred rename. No mechanical consequence.

**Audit note — Integration test: Kindred permanent loses its other permanent type.** If a Kindred Enchantment (e.g., Bitterblossom: "Kindred Enchantment — Faerie") has its Enchantment type removed by a continuous effect (L4 type change), it becomes just "Kindred — Faerie" on the battlefield.

Per CR 110.4c: "If a permanent somehow loses all its permanent types, it remains on the battlefield. It's still a permanent." The six permanent types are listed in CR 110.4: artifact, battle, creature, enchantment, land, and planeswalker. Kindred is NOT a permanent type (110.4 also notes: "Some kindred cards can enter the battlefield and some can't, depending on their other card types."). So removing Enchantment leaves the object with zero permanent types, but 110.4c says it stays on the battlefield.

The object still has the Kindred card type (L4 only removed Enchantment, not Kindred). Since Kindred shares its set of subtypes with creatures (308.2: "The set of kindred subtypes is the same as the set of creature subtypes"), the Faerie subtype is still valid — it belongs to Kindred, not to the now-removed Enchantment type. The permanent remains on the battlefield as "Kindred — Faerie" with no permanent types.

This integration test should be deferred to Phase 8 when Kindred cards are implemented.

---

## Rules 309.x — Dungeons

**Classification: DEFERRED (all sub-rules)**

Dungeons (309.1–309.7) are a specialized card type used with the "venture into the dungeon" keyword action. Per roadmap.md, dungeons are Phase 8–9 scope at earliest. All rules in this section are classified as DEFERRED.

| Rule | Summary |
|------|---------|
| 309.1 | Dungeon is a nontraditional card type |
| 309.2 | Dungeons begin outside the game |
| 309.2a | Venture chooses dungeon from outside game |
| 309.2b | Dungeon goes to command zone |
| 309.2c | Dungeons aren't permanents, can't be cast |
| 309.2d | Only venture brings dungeons into game |
| 309.3 | One dungeon per player at a time |
| 309.4 | Rooms connected with arrows, venture marker |
| 309.4a | Venture marker starts on topmost room |
| 309.4b | Room names are flavor text |
| 309.4c | Room abilities are triggered abilities |
| 309.5 | Venture moves marker down rooms |
| 309.5a | Venture from non-bottom room moves marker |
| 309.5b | Venture from bottom room removes dungeon, gets new one |
| 309.6 | SBA: completed dungeon removed from game |
| 309.7 | Completing a dungeon defined |

---

## Rules 310.x — Battles

Battles are a Phase 8–9 type per roadmap.md. However, several rules define testable behaviors that the engine infrastructure (counters, SBAs, attack targets) may need to support. I'll classify each rule but note the phase deferral.

### 310.1

**Classification: DEFERRED**

Rule text: "A player who has priority may cast a battle card from their hand during a main phase of their turn when the stack is empty."

Standard permanent casting — same pipeline as other permanent types. Deferred until battle cards are implemented.

---

### 310.2

**Classification: DEFERRED**

Standard permanent resolution. Deferred.

---

### 310.3

**Classification: PURE-DEF**

Subtype format definition.

---

### 310.4

**Classification: BOUNDARY-DEF (deferred)**

Rule text: "Defense is a characteristic that battles have."

Analogous to 306.5 (loyalty for planeswalkers). Deferred to Phase 8–9.

---

### 310.4a

**Classification: DEFERRED**

Defense of non-battlefield battle = printed number. Analogous to 306.5a. Deferred.

---

### 310.4b

**Classification: DEFERRED**

Battle ETB with defense counters (replacement effect). Analogous to 306.5b. Deferred.

---

### 310.4c

**Classification: DEFERRED**

On-battlefield defense = defense counter count. Analogous to 306.5c. Deferred.

---

### 310.5

**Classification: DEFERRED**

Battles can be attacked. Analogous to 306.6. Deferred.

---

### 310.6

**Classification: DEFERRED**

Damage to battle removes defense counters. Analogous to 306.8. Deferred.

---

### 310.7

**Classification: DEFERRED**

Battle with 0 defense → graveyard (SBA). Analogous to 306.9. Deferred.

---

### 310.8–310.8g

**Classification: DEFERRED (all)**

Protector designation system. All deferred to Phase 8–9.

| Rule | Summary |
|------|---------|
| 310.8 | Each battle has a protector |
| 310.8a | Controller chooses protector on ETB |
| 310.8b | Protector can't attack it; others can |
| 310.8c | Protector's creatures can block attackers of that battle |
| 310.8d | "Defending player" refers to protector |
| 310.8e | "Player who protects" means protector |
| 310.8f | One protector at a time |
| 310.8g | Protector doesn't change on type change/copy |

---

### 310.9

**Classification: DEFERRED**

Battle can't be attached. SBA unattaches. Deferred.

---

### 310.10

**Classification: DEFERRED**

Battle with no protector → controller chooses or SBA puts to graveyard. Deferred.

---

### 310.11, 310.11a, 310.11b

**Classification: DEFERRED**

Siege-specific rules: protector from opponents, intrinsic "exile when last defense counter removed" ability. Deferred.

---

## Rules 311.x — Planes

**Classification: OUT-OF-SCOPE (all sub-rules)**

Planes (311.1–311.7) are Planechase-only supplemental cards. Per roadmap.md, supplemental formats are not in scope.

| Rule | Classification |
|------|---------------|
| 311.1–311.7 | OUT-OF-SCOPE (Planechase only) |

---

## Rules 312.x — Phenomena

**Classification: OUT-OF-SCOPE (all sub-rules)**

Phenomena (312.1–312.7) are Planechase-only supplemental cards.

| Rule | Classification |
|------|---------------|
| 312.1–312.7 | OUT-OF-SCOPE (Planechase only) |

---

## Rules 313.x — Vanguards

**Classification: DEFERRED (all sub-rules — stretch goal)**

Vanguards (313.1–313.7) are Vanguard casual variant. Deferred to stretch goal phase.

| Rule | Classification |
|------|---------------|
| 313.1–313.7 | DEFERRED (Vanguard — stretch goal) |

---

## Rules 314.x — Schemes

**Classification: OUT-OF-SCOPE (all sub-rules)**

Schemes (314.1–314.7) are Archenemy casual variant only.

| Rule | Classification |
|------|---------------|
| 314.1–314.7 | OUT-OF-SCOPE (Archenemy only) |

---

## Rules 315.x — Conspiracies

**Classification: OUT-OF-SCOPE (all sub-rules)**

Conspiracies (315.1–315.7) are Conspiracy Draft variant only.

| Rule | Classification |
|------|---------------|
| 315.1–315.7 | OUT-OF-SCOPE (Conspiracy Draft only) |

---

## Composition Tests

These tests require 2+ atomic mechanisms working together. They are labeled COMP- and reference which ATOM- tests they compose.

**COMP-301.5c+303.4c-001**
- **Rule:** 301.5c (Equipment illegal host SBA) + 303.4c (Aura illegal host SBA)
- **Mechanism:** Creature with Equipment and Aura attached is destroyed; both attachment SBAs fire
- **Composes:** ATOM-301.5c-004, ATOM-303.4c-002
- **Minimal Board:** Creature with an Equipment and an Aura attached. Lightning Bolt resolving, dealing lethal damage.
- **Action:** Bolt resolves, creature takes lethal. SBA cycle: creature destruction → zone exit → detachment → Equipment SBA (unattach, stay on BF) + Aura SBA (unattached → graveyard)
- **Expected Result:** Creature in graveyard. Equipment on battlefield unattached. Aura in owner's graveyard.
- **Phase:** Phase 5 Pre-Work (T04 + T15)
- **Ticket:** T04 + T15

**COMP-305.7+305.6-001**
- **Rule:** 305.7 (Blood Moon type-setting) + 305.6 (intrinsic mana ability from basic land type)
- **Mechanism:** Blood Moon sets nonbasic land to Mountain; land gains {T}: Add {R} and loses old abilities
- **Composes:** ATOM-305.7-001, ATOM-305.7-002, ATOM-305.6-002
- **Minimal Board:** Nonbasic land with "{T}: Add {U} or {B}" on battlefield. Blood Moon enchantment on battlefield with continuous effect "Nonbasic lands are Mountains."
- **Action:** Activate the land's mana ability
- **Expected Result:** Land produces {R} (not {U} or {B}). Old mana ability is gone. New intrinsic Mountain ability applies.
- **Phase:** Phase 5 Layers (L17)
- **Ticket:** L17

**COMP-306.5b+306.8+306.9-001**
- **Rule:** 306.5b (PW ETB with loyalty) + 306.8 (damage removes loyalty) + 306.9 (0 loyalty SBA)
- **Mechanism:** Planeswalker enters, takes damage equal to its loyalty, SBA kills it
- **Composes:** ATOM-306.5b-001, ATOM-306.8-001, ATOM-306.9-001
- **Minimal Board:** Planeswalker spell with printed loyalty 3 on the stack. Opponent has a creature that will deal 3 damage (e.g., Lightning Bolt in hand).
- **Action:** PW resolves (enters with 3 loyalty). Opponent casts Bolt targeting PW. Bolt resolves, dealing 3 damage → 3 loyalty counters removed → 0 loyalty. SBA check runs.
- **Expected Result:** Planeswalker is put into its owner's graveyard via SBA.
- **Phase:** Phase 5 Pre-Work (T14 + T21c)
- **Ticket:** T14, T21c

**COMP-303.4f+303.4c-001**
- **Rule:** 303.4f (non-stack Aura ETB, controller chooses) + 303.4c (Aura on illegal host SBA)
- **Mechanism:** Aura returned from graveyard, attaches to creature, creature becomes non-creature via L4 type change, SBA removes Aura
- **Composes:** ATOM-303.4f-001, ATOM-303.4c-001
- **Minimal Board:** Aura with "Enchant creature" in graveyard. A creature on battlefield. An effect returns the Aura to battlefield (controller chooses the creature). Then a type-changing effect removes Creature from the host.
- **Action:** Aura enters attached to creature. Type-changing effect resolves. SBA check runs.
- **Expected Result:** Aura is put into graveyard — host is no longer a legal enchant target.
- **Phase:** Phase 5 Layers + Phase 5 Pre-Work
- **Ticket:** T15, T15b, L09

**COMP-301.5d+303.4e-001**
- **Rule:** 301.5d (Equipment controller ≠ creature controller) + 303.4e (Aura controller ≠ enchanted controller)
- **Mechanism:** Control change of a creature doesn't change control of its Equipment or Aura
- **Composes:** ATOM-301.5d-001, ATOM-303.4e-001
- **Minimal Board:** Creature controlled by P0 with Equipment (P0) and Aura (P0) attached. A control-changing effect gives the creature to P1.
- **Action:** Resolve control change
- **Expected Result:** Creature controller = P1. Equipment controller = P0. Aura controller = P0. All attachments remain.
- **Phase:** Phase 5 Layers (L11)
- **Ticket:** L11

**COMP-305.7+305.8-001**
- **Rule:** 305.7 (setting subtypes) + 305.8 (basic = supertype, not subtype)
- **Mechanism:** Blood Moon makes nonbasic land into Mountain, but it doesn't gain "basic" supertype
- **Composes:** ATOM-305.7-004, ATOM-305.8-001
- **Minimal Board:** Nonbasic legendary land. Blood Moon on battlefield.
- **Action:** Query: is this land a basic land? Does it have "Legendary" supertype?
- **Expected Result:** Land is still nonbasic (no Basic supertype). Still legendary. Has Mountain subtype and {T}: Add {R}.
- **Phase:** Phase 5 Layers (L17)
- **Ticket:** L17

**COMP-303.4c+303.4e+L11-001**
- **Rule:** 303.4c (Aura on illegal host → graveyard SBA) + 303.4e (Aura controller independence) + L11 (control change)
- **Mechanism:** Aura with "Enchant creature you control" falls off when enchanted creature changes controller
- **Composes:** ATOM-303.4c-001, ATOM-303.4e-001
- **Minimal Board:** P0 controls a creature. P0 controls an Aura with "Enchant creature you control" attached to that creature. A control-changing effect gives the creature to P1.
- **Action:** Control change resolves. SBA check runs.
- **Expected Result:** Creature controller = P1. Aura controller = P0 (unchanged per 303.4e). But now the enchant restriction "creature you control" is violated — the Aura's controller (P0) no longer controls the enchanted creature. SBA 303.4c fires: Aura is put into P0's graveyard.
- **Phase:** Phase 5 Layers (L11) + Phase 5 Pre-Work (T15)
- **Ticket:** L11, T15, T15b

**Audit note — Additional Blood Moon composition tests for future sessions:**
1. **Blood Moon + Urborg dependency (613.8):** When both Blood Moon and Urborg, Tomb of Yawgmoth are on the battlefield, dependency detection determines application order. Urborg makes all lands Swamps "in addition to"; Blood Moon makes nonbasic lands Mountains (replacing). If Blood Moon depends on Urborg (because Urborg changes what Blood Moon affects), Urborg applies first → all lands gain Swamp → then Blood Moon applies → nonbasic lands become Mountains (losing Swamp). Result: basics are Swamp+their type; nonbasics are just Mountain. If no dependency, timestamp order matters. This is a Gate 4 test per roadmap.md for the L17/613.8 session.
2. **Blood Moon + fetchlands:** A fetchland under Blood Moon loses its sacrifice ability (rules text stripped by 305.7). It can only tap for {R}. If Blood Moon leaves the battlefield, the fetchland regains its original abilities.
3. **Blood Moon + Dryad of the Ilysian Grove:** Dryad says "Lands you control are every basic land type in addition to their other types." This grants all 5 basic land types additively (305.7 additive clause). Blood Moon then tries to set nonbasics to Mountain. Dependency: Dryad affects what lands Blood Moon sees. Complex L4 interaction.
These belong in the Chapter 6 (613.x Layers interaction) session, not Chapter 3.

---

## Classification Summary

| Rule | Classification | Notes |
|------|---------------|-------|
| 300.1 | BOUNDARY-DEF | CardType enum completeness |
| 300.2 | META | Cross-cutting: targeting, SBAs, continuous effects, combat, zone guards |
| 300.2a | TESTABLE | Land+other type can only be played as land |
| 300.2b | PURE-DEF | Kindred casting cross-ref |
| 301.1 | ALREADY-IMPLEMENTED | Artifact casting timing |
| 301.2 | ALREADY-IMPLEMENTED | Artifact spell resolution |
| 301.3 | PURE-DEF | Artifact subtype format |
| 301.4 | PURE-DEF | Artifacts have no type-specific characteristics |
| 301.5 | BOUNDARY-DEF | Equipment attachment legality |
| 301.5a | PURE-DEF | "Equipped creature" naming |
| 301.5b | TESTABLE | Equipment ETB unattached; equip ability; illegal-attach guard |
| 301.5c | TESTABLE | 5 clauses (006 removed per audit): creature-Equipment, loses subtype, self-equip, illegal host SBA, one creature max |
| 301.5d | TESTABLE | Equipment/creature controller independence; ability activation routing |
| 301.5e | TESTABLE | Equipment ETB illegal-target enters unattached |
| 301.5f | PURE-DEF | "Equipped creature" reference resolution |
| 301.6 | TESTABLE | Fortification attachment to lands |
| 301.7 | PURE-DEF | Vehicle intro |
| 301.7a | TESTABLE | Vehicle P/T only when creature |
| 301.7b | TESTABLE | Vehicle P/T with continuous effects |
| 302.1 | ALREADY-IMPLEMENTED | Creature casting timing |
| 302.2 | ALREADY-IMPLEMENTED | Creature spell resolution |
| 302.3 | PURE-DEF | Creature subtype format |
| 302.4 | BOUNDARY-DEF | P/T only for creatures |
| 302.4a | PURE-DEF | Power = combat damage dealt |
| 302.4b | PURE-DEF | Toughness = damage to destroy |
| 302.4c | TESTABLE | P/T computed from printed + continuous effects |
| 302.5 | ALREADY-IMPLEMENTED | Creatures attack and block |
| 302.6 | ALREADY-IMPLEMENTED | Summoning sickness |
| 302.7 | ALREADY-IMPLEMENTED | Damage marking, lethal damage SBA, cleanup removal |
| 303.1 | ALREADY-IMPLEMENTED | Enchantment casting timing |
| 303.2 | ALREADY-IMPLEMENTED | Enchantment spell resolution |
| 303.3 | PURE-DEF | Enchantment subtype format |
| 303.4 | TESTABLE | Aura ETB attached to target |
| 303.4a | TESTABLE | Aura spell requires target from enchant ability |
| 303.4b | PURE-DEF | "Enchanted" naming |
| 303.4c | TESTABLE | Aura on illegal host → graveyard SBA; player-leaves-game SBA (003 added) |
| 303.4d | TESTABLE | 2 clauses (003 removed per audit): self-enchant SBA, creature-Aura SBA |
| 303.4e | TESTABLE | Aura/enchanted controller independence; granted ability routing; Pacifism caster=controller (003 added) |
| 303.4f | TESTABLE | Non-stack Aura ETB: controller chooses (not targeting) |
| 303.4g | TESTABLE | No legal enchant target: stays in zone / graveyard from stack / token not created |
| 303.4h | TESTABLE | Non-Aura/Equipment/Fortification entering attached → unattached |
| 303.4i | TESTABLE | Aura entering attached to illegal host: stays in zone (non-stack) / graveyard (stack, 002 added) / token not created |
| 303.4j | TESTABLE | Move Aura to illegal host: doesn't move |
| 303.4k | DEFERRED | Face-down Aura turning face up (Phase 9) |
| 303.4m | PURE-DEF | "Enchanted [X]" reference resolution |
| 303.5 | PURE-DEF | Saga cross-ref |
| 303.6 | PURE-DEF | Class cross-ref |
| 303.7 | PURE-DEF | Role intro |
| 303.7a | TESTABLE | Role uniqueness SBA per controller |
| 304.1 | ALREADY-IMPLEMENTED | Instant casting at any priority |
| 304.2 | ALREADY-IMPLEMENTED | Instant resolution → graveyard |
| 304.3 | PURE-DEF | Instant subtype format |
| 304.4 | TESTABLE | Instant can't enter battlefield |
| 304.5 | TESTABLE | "As an instant" = priority only; casting prohibition doesn't affect |
| 305.1 | ALREADY-IMPLEMENTED | Land play as special action |
| 305.2 | TESTABLE | One land per turn; continuous effects increase |
| 305.2a | TESTABLE | Land play count comparison |
| 305.2b | TESTABLE | Hard cap: can't play land at limit even if effect says to |
| 305.3 | ALREADY-IMPLEMENTED | Can't play land if not your turn |
| 305.4 | TESTABLE | "Put" ≠ "play"; doesn't count toward limit |
| 305.5 | PURE-DEF | Land subtype format |
| 305.6 | TESTABLE | Basic land type → intrinsic mana ability; boundary: only 5 types |
| 305.7 | TESTABLE | 5 clauses: set removes old types, loses rules-text abilities, gains mana ability, keeps granted abilities, "in addition to" is additive |
| 305.8 | BOUNDARY-DEF | Basic = supertype, not subtype |
| 305.9 | DUPLICATE | See 300.2a — test removed |
| 306.1 | ALREADY-IMPLEMENTED | Planeswalker casting timing |
| 306.2 | ALREADY-IMPLEMENTED | Planeswalker spell resolution |
| 306.3 | PURE-DEF | Planeswalker subtype format |
| 306.4 | PURE-DEF | Obsolete uniqueness rule → legend rule |
| 306.5 | BOUNDARY-DEF | Loyalty only for planeswalkers |
| 306.5a | TESTABLE | Non-BF PW loyalty = printed |
| 306.5b | TESTABLE | PW ETB with loyalty counters |
| 306.5c | TESTABLE | BF PW loyalty = loyalty counter count |
| 306.5d | TESTABLE | Loyalty ability timing: sorcery speed, once per PW per turn |
| 306.6 | TESTABLE | PW can be attacked |
| 306.7 | PURE-DEF | Obsolete damage redirection |
| 306.8 | TESTABLE | Damage to PW removes loyalty counters; excess absorbed |
| 306.9 | TESTABLE | 0 loyalty → graveyard SBA |
| 307.1 | ALREADY-IMPLEMENTED | Sorcery casting timing |
| 307.2 | ALREADY-IMPLEMENTED | Sorcery resolution → graveyard |
| 307.3 | PURE-DEF | Sorcery subtype format |
| 307.4 | TESTABLE | Sorcery can't enter battlefield |
| 307.5 | TESTABLE | "As a sorcery" = priority + main + empty stack; casting prohibition independent; Teferi-style restriction (004/005 added) |
| 307.5a | TESTABLE | "Couldn't cast as sorcery" retroactive timing check |
| 308.1 | TESTABLE | Kindred follows other type's casting rules |
| 308.2 | PURE-DEF | Kindred subtype = creature types |
| 308.3 | PURE-DEF | Tribal → kindred rename |
| 309.1 | DEFERRED | Dungeon nontraditional type |
| 309.2 | DEFERRED | Dungeons begin outside game |
| 309.2a | DEFERRED | Venture chooses dungeon |
| 309.2b | DEFERRED | Dungeon to command zone |
| 309.2c | DEFERRED | Not permanents, can't cast |
| 309.2d | DEFERRED | Only venture brings in |
| 309.3 | DEFERRED | One dungeon at a time |
| 309.4 | DEFERRED | Room structure |
| 309.4a | DEFERRED | Venture marker placement |
| 309.4b | DEFERRED | Room names are flavor |
| 309.4c | DEFERRED | Room triggered abilities |
| 309.5 | DEFERRED | Venture moves marker |
| 309.5a | DEFERRED | Move from non-bottom |
| 309.5b | DEFERRED | Move from bottom → new dungeon |
| 309.6 | DEFERRED | SBA: completed dungeon removed |
| 309.7 | DEFERRED | Completing defined |
| 310.1 | DEFERRED | Battle casting |
| 310.2 | DEFERRED | Battle resolution |
| 310.3 | PURE-DEF | Battle subtype format |
| 310.4 | DEFERRED | Defense characteristic |
| 310.4a | DEFERRED | Non-BF defense = printed |
| 310.4b | DEFERRED | Battle ETB with defense counters |
| 310.4c | DEFERRED | BF defense = counter count |
| 310.5 | DEFERRED | Battles can be attacked |
| 310.6 | DEFERRED | Damage removes defense counters |
| 310.7 | DEFERRED | 0 defense SBA |
| 310.8 | DEFERRED | Protector designation |
| 310.8a | DEFERRED | Choose protector on ETB |
| 310.8b | DEFERRED | Protector can't attack |
| 310.8c | DEFERRED | Protector blocks for battle |
| 310.8d | DEFERRED | Defending player = protector |
| 310.8e | DEFERRED | "Player who protects" |
| 310.8f | DEFERRED | One protector at a time |
| 310.8g | DEFERRED | Protector persists through copy |
| 310.9 | DEFERRED | Battle can't be attached |
| 310.10 | DEFERRED | No protector SBA |
| 310.11 | DEFERRED | Siege subtype |
| 310.11a | DEFERRED | Siege protector from opponents |
| 310.11b | DEFERRED | Siege exile on last counter |
| 311.1–311.7 | OUT-OF-SCOPE | Planes (Planechase) |
| 312.1–312.7 | OUT-OF-SCOPE | Phenomena (Planechase) |
| 313.1–313.7 | DEFERRED | Vanguards (stretch goal) |
| 314.1–314.7 | OUT-OF-SCOPE | Schemes (Archenemy) |
| 315.1–315.7 | OUT-OF-SCOPE | Conspiracies |

---

## Gap Report

### Mechanisms in roadmap.md / implementation-plan-final.md that SHOULD have tests from Chapter 3 but don't have a standalone CR sub-rule

1. **Equip keyword ability (702.6)** — Referenced by 301.5b but defined in Chapter 7 (keyword abilities). The ATOM-301.5b-002 test covers the *result* of equip, but the activation/resolution pipeline for equip is a Chapter 7 test. **Session covering 702.6 should generate equip-specific ATOM tests.**

2. **Enchant keyword ability (702.5)** — Referenced by 303.4 but defined in Chapter 7. The EnchantRestriction targeting model (T15b) is tested via 303.4a/303.4f tests, but the enchant ability itself is Chapter 7 scope.

3. **Crew keyword ability (702.122)** — Referenced by 301.7 but defined in Chapter 7. Vehicle P/T tests (ATOM-301.7a/b) depend on crew working, but crew itself is Chapter 7 scope.

4. **Fortify keyword ability (702.67)** — Referenced by 301.6 but defined in Chapter 7. Fortification attachment tests depend on fortify working.

5. **Loyalty abilities (606)** — Referenced by 306.5d. The activation timing tests (ATOM-306.5d) cover the timing restrictions, but the full loyalty cost payment pipeline (adding/removing loyalty counters as cost, rule 606.5) is Chapter 6 scope.

6. **Planeswalker as attack target** — 306.6 says planeswalkers can be attacked, but the attack declaration pipeline in Chapter 5 (508) defines how to declare attacks against planeswalkers (choosing which player or planeswalker to attack). ATOM-306.6-001 tests legality; the full attack-target-choice pipeline belongs in the Chapter 5 session. **T21c step 2 (PW damage routing) is the engine ticket.**

7. **"Put land onto battlefield" primitive** — 305.4 distinguishes "put" from "play," but no Primitive enum variant exists yet for "put a land onto the battlefield." This is a Phase 8 gap (when fetch lands, ramp effects, etc. are implemented). The land play counter must NOT be incremented for these effects.

8. **Blood Moon + Urborg dependency interaction** — L17 tests 305.7 extensively, but the dependency detection (613.8) interaction between Blood Moon and Urborg is a Phase 5 Layers concern (tested at Gate 4 per roadmap.md). The individual 305.7 ATOM tests verify Blood Moon's behavior in isolation; the dependency test is a COMP test for the Session covering 613.x.

9. **Role subtype SBA (303.7a)** — The Role uniqueness SBA is not in the current SBA list (T13–T16). This needs a new ticket when Role cards are implemented (Phase 8).

### Test Count Summary (post-audit)

- **ATOM tests generated:** 87 (was 64; +26 added, -3 removed per audit)
- **COMP tests generated:** 7 (was 6; +1 added per audit)
- **Total tests:** 94
- **Rules classified TESTABLE:** 36
- **Rules classified BOUNDARY-DEF:** 6
- **Rules classified PURE-DEF:** 28
- **Rules classified META:** 1 (300.2 reclassified from TESTABLE)
- **Rules classified ALREADY-IMPLEMENTED:** 18
- **Rules classified DEFERRED:** 40 (303.4k, 309.x dungeons, 310.x battles, 313.x vanguards)
- **Rules classified OUT-OF-SCOPE:** 27 (311.x planes, 312.x phenomena, 314.x schemes, 315.x conspiracies)
- **Rules classified DUPLICATE:** 1 (305.9 → see 300.2a)
- **Total sub-rules accounted for:** ~157

### Audit Changes Applied
- 300.2: TESTABLE → META
- 301.5c-006: REMOVED (no known multi-equip effect)
- 302.4-002: ADDED (positive test — creature has P/T)
- 303.4-002: ADDED (Aura attaching to player)
- 303.4c-003: ADDED (Aura falls off when enchanted player leaves game)
- 303.4d-003: REMOVED (no known multi-enchant effect)
- 303.4e-003: ADDED (Pacifism-style: caster = Aura controller even on opponent's permanent)
- 303.4g-002: Audit note added (covered by fizzle logic but kept as regression test)
- 303.4i-002: ADDED (from-stack illegal attachment → graveyard)
- 303.4m: Audit note added (no semantic difference with 301.5f in engine)
- 304.5: Audit note added (flash implementation implications — CastPermission modifier)
- 305.2: Audit note added (integration test for Exploration + extra turn interaction)
- 305.2b: Updated scenario to impulse draw; audit note about structural enforcement
- 305.6-002: Changed from Blood Moon to Urborg (cleaner test of same mechanism)
- 305.7-003: Clarified granting effect from separate permanent
- 305.8-002: ADDED (positive test — Basic supertype = basic land)
- 305.9: TESTABLE → DUPLICATE (cross-ref 300.2a, test removed)
- 306.5c: Audit note added (folds into 306.8-001 setup, kept for atomicity)
- 307.5-004, 307.5-005: ADDED (Teferi-style sorcery-speed restriction on opponents)
- 307.5a: Audit note added (self-referential timing checks only; Necromancy; CastInfo flag)
- 308: Audit note added (kindred losing permanent type — 300.4 SBA edge case)
- COMP-303.4c+303.4e+L11-001: ADDED (Enchant creature you control + control change)
- Blood Moon complex composition tests: Deferred to Chapter 6 (613.x) session
