# Session 9B — Atomic Test Generation
## CR Rules 713–732 (ch7-pt-5.txt)

**Scope:** Substitute cards, Saga cards, Adventurer cards, Class cards, Attraction cards, Prototype cards, Case cards, Omen cards, Station cards, controlling another player, ending turns/phases, restarting the game, Monarch, Initiative, rad counters, subgames, merging with permanents, day/night, shortcuts, handling illegal actions.

**Source file:** `MTG-Rules/LLM-Chapter-Splits/ch7-pt-5.txt`

**Chunk plan:**
- Chunk 1: Rules 713–721
- Chunk 2: Rules 722–727
- Chunk 3: Rules 728–732
- Chunk 4: Classification Summary, Composition Tests, Gap Report

---

## Chunk 1: Rules 713–721

### 713. Substitute Cards — OUT-OF-SCOPE

All sub-rules (713.1, 713.2, 713.2a, 713.2b, 713.2c, 713.3, 713.4, 713.5) are **OUT-OF-SCOPE**. Substitute cards are a physical play aid with no digital equivalent. The simulator represents all cards directly.

---

### 714. Saga Cards

**714.1** — PURE-DEF. Describes card frame layout. No mechanical consequence.

**714.1a** — PURE-DEF. Describes Saga creature printing layout. The statement "abilities in that text box are independent of its chapter symbols" is a prerequisite for understanding how Saga creatures work, but is enforced by card data authoring, not the engine.

**714.2** — PURE-DEF. Defines "chapter symbol" as a keyword ability representing a triggered ability. Prerequisite for 714.2b.

**714.2a** — PURE-DEF. Defines Roman numeral notation for chapter symbols. No independent mechanical consequence.

**714.2b** — TESTABLE. Defines the trigger condition for chapter abilities. This is the core Saga trigger rule: "When one or more lore counters are put onto this Saga, if the number of lore counters on it was less than N and became at least N, [effect]."

**ATOM-714.2b-001**
- **Rule:** 714.2b — Chapter ability triggers when lore counter count crosses the chapter threshold
- **Mechanism:** Triggered ability firing based on counter threshold crossing
- **Minimal Board:** Player A controls a Saga with chapters I, II, III. It currently has 0 lore counters.
- **Action:** A lore counter is placed on the Saga (e.g., via the turn-based action from 714.3b), bringing the count from 0 to 1.
- **Expected Result:** Chapter I ability triggers (count was less than 1, became at least 1). Chapter II and III do NOT trigger.
- **Phase:** Phase 7 (triggered abilities) + Phase 8 (card breadth — Saga card type)
- **Ticket:** NEW — Saga chapter ability trigger condition (714.2b)

**ATOM-714.2b-002**
- **Rule:** 714.2b — Multiple chapters trigger simultaneously when counter count jumps past multiple thresholds
- **Mechanism:** Triggered ability firing for all crossed thresholds at once
- **Minimal Board:** Player A controls a Saga with chapters I, II, III. It currently has 0 lore counters.
- **Action:** An effect puts 2 lore counters on the Saga at once, bringing the count from 0 to 2.
- **Expected Result:** Both chapter I and chapter II abilities trigger (count crossed both thresholds). Chapter III does NOT trigger.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Saga multi-chapter trigger on bulk counter add (714.2b)

**ATOM-714.2b-003**
- **Rule:** 714.2b — Chapter ability does NOT re-trigger if counter count was already at or above threshold
- **Mechanism:** Trigger suppression when threshold was already met
- **Minimal Board:** Player A controls a Saga with chapters I, II, III. It currently has 2 lore counters.
- **Action:** A lore counter is placed on the Saga, bringing the count from 2 to 3.
- **Expected Result:** Only chapter III triggers (count was less than 3, became at least 3). Chapters I and II do NOT trigger (count was already ≥ 1 and ≥ 2 before the counter was added).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Saga chapter threshold already-met suppression (714.2b)

**714.2c** — TESTABLE. Multi-chapter symbols share an effect.

**ATOM-714.2c-001**
- **Rule:** 714.2c — A chapter symbol listing multiple numerals (e.g., "{rI}, {rII}") triggers on each threshold independently
- **Mechanism:** Shared chapter ability fires at each listed chapter number
- **Minimal Board:** Player A controls a Saga with "{rI}, {rII} — Draw a card" and "{rIII} — [other effect]". It has 0 lore counters.
- **Action:** A lore counter is placed on the Saga (0 → 1).
- **Expected Result:** The shared chapter ability triggers (for chapter I). Later, when a second counter is added (1 → 2), it triggers again (for chapter II).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Saga multi-numeral chapter symbol (714.2c)

**714.2d** — TESTABLE. Defines final chapter number computation.

**ATOM-714.2d-001**
- **Rule:** 714.2d — A Saga's final chapter number is the greatest chapter numeral among its chapter abilities
- **Mechanism:** Final chapter number computation for SBA sacrifice check (714.4)
- **Minimal Board:** Player A controls a Saga with chapters I, II, III. It has 3 lore counters.
- **Action:** SBA check runs.
- **Expected Result:** Final chapter number = 3 (greatest chapter numeral). Since lore counters (3) ≥ final chapter number (3), and no chapter ability is on the stack, the Saga is sacrificed.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Saga final chapter number computation (714.2d)

**714.2e** — PURE-DEF. Defines "final chapter ability" as the chapter ability with the final chapter number. Prerequisite for understanding 714.4 but no independent mechanical consequence beyond 714.2d.

**714.3** — PURE-DEF. States Sagas use lore counters. Prerequisite for 714.3a/b.

**714.3a** — TESTABLE. Defines ETB lore counter behavior (with and without read ahead).

**ATOM-714.3a-001**
- **Rule:** 714.3a — A Saga without read ahead enters the battlefield with one lore counter
- **Mechanism:** ETB replacement/TBA placing initial lore counter
- **Minimal Board:** Player A has a Saga card (no read ahead) on the stack, resolving.
- **Action:** The Saga spell resolves and enters the battlefield.
- **Expected Result:** The Saga enters with exactly 1 lore counter. Chapter I ability triggers.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Saga ETB initial lore counter (714.3a)

**ATOM-714.3a-002**
- **Rule:** 714.3a — A Saga with read ahead enters with a chosen number of lore counters
- **Mechanism:** DecisionProvider choice for read-ahead Saga initial counter count
- **Minimal Board:** Player A has a Saga card with read ahead (chapters I, II, III) on the stack, resolving.
- **Action:** The Saga spell resolves. Player A chooses 2 via DecisionProvider.
- **Expected Result:** The Saga enters with 2 lore counters. Chapters I and II both trigger (thresholds crossed from 0).
- **Phase:** Phase 8 (read ahead keyword 702.155) + Phase 7
- **Ticket:** NEW — Saga read ahead ETB choice (714.3a)

**714.3b** — TESTABLE. Turn-based action: lore counter added at precombat main phase.

**ATOM-714.3b-001**
- **Rule:** 714.3b — At the beginning of each player's precombat main phase, a lore counter is put on each Saga they control
- **Mechanism:** Turn-based action adding lore counters
- **Minimal Board:** Player A controls a Saga with 1 lore counter (chapters I, II, III).
- **Action:** Player A's precombat main phase begins.
- **Expected Result:** A lore counter is added (1 → 2). This doesn't use the stack. Chapter II triggers (threshold crossed).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Saga precombat main phase lore counter TBA (714.3b)

**714.4** — TESTABLE. SBA: sacrifice Saga when lore counters ≥ final chapter number and no chapter ability is on the stack from this Saga.

**ATOM-714.4-001**
- **Rule:** 714.4 — Saga is sacrificed when lore counters ≥ final chapter number and no chapter ability on stack
- **Mechanism:** State-based action for Saga sacrifice
- **Minimal Board:** Player A controls a Saga with chapters I, II, III. It has 3 lore counters. No chapter abilities from this Saga are on the stack.
- **Action:** SBAs are checked.
- **Expected Result:** The Saga is sacrificed (moved to graveyard). This SBA doesn't use the stack.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Saga sacrifice SBA (714.4)

**ATOM-714.4-002**
- **Rule:** 714.4 — Saga is NOT sacrificed while a chapter ability from it is on the stack
- **Mechanism:** SBA suppression while source chapter ability is pending
- **Minimal Board:** Player A controls a Saga with chapters I, II, III. It has 3 lore counters. Chapter III's triggered ability is on the stack (has triggered but not yet resolved).
- **Action:** SBAs are checked.
- **Expected Result:** The Saga is NOT sacrificed yet. After Chapter III's ability resolves and leaves the stack, the next SBA check will sacrifice it.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Saga sacrifice SBA deferred while chapter on stack (714.4)

---

### 715. Adventurer Cards

**715.1** — PURE-DEF. Describes card frame layout.

**715.2** — TESTABLE (zone-dependent characteristics). The alternative characteristics exist while the object is a spell (on the stack cast as Adventure). Otherwise normal characteristics apply.

**ATOM-715.2-001**
- **Rule:** 715.2 — Adventurer card's alternative characteristics define what the object has while it's an Adventure spell
- **Mechanism:** Zone-dependent characteristic selection for Adventure spells
- **Minimal Board:** Player A has an adventurer card in hand. The card's normal characteristics are Creature 2/2 {1}{G}; its Adventure characteristics are Instant {G} "deal 2 damage."
- **Action:** Player A casts the card as an Adventure.
- **Expected Result:** On the stack, the spell has the Adventure characteristics (Instant, {G} mana cost, "deal 2 damage" effect). It does NOT have the creature characteristics.
- **Phase:** Phase 9 (D4 — CardLayout restructuring)
- **Ticket:** D4 — Split card / Adventure / CardLayout

**715.2a** — TESTABLE. "Has an Adventure" reference works even when not using alternative characteristics.

**ATOM-715.2a-001**
- **Rule:** 715.2a — An effect referring to a card that "has an Adventure" finds the adventurer card even when in hand/graveyard (not using alternative characteristics)
- **Mechanism:** Object query for "has an Adventure" flag
- **Minimal Board:** Player A has an adventurer card in their graveyard. An effect says "return a card that has an Adventure from your graveyard to your hand."
- **Action:** The effect resolves, searching for cards "that have an Adventure."
- **Expected Result:** The adventurer card in the graveyard qualifies (it "has an Adventure" even though it's using normal characteristics in the graveyard).
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**715.2b** — TESTABLE. Alternative characteristics are copiable values.

**ATOM-715.2b-001**
- **Rule:** 715.2b — The alternative characteristics of an adventurer card are part of its copiable values
- **Mechanism:** Copy effect on adventurer card preserves Adventure characteristics
- **Minimal Board:** Player A controls a Clone-type creature. An adventurer creature is on the battlefield.
- **Action:** Clone copies the adventurer creature.
- **Expected Result:** The copy has the same Adventure alternative characteristics as the original as part of its copiable values while on the battlefield. If the copy is bounced to hand, the copy effect ceases (new game object per 400.7) — the card in hand has only its printed characteristics.
- **Phase:** Phase 9 (D4) + Phase 6 (copy effects)
- **Ticket:** D4

**715.2c** — PURE-DEF. An adventurer card is one card, not two. No independent mechanical consequence for the engine (the engine already models it as one GameObject).

**715.3** — TESTABLE. Mode choice at cast time.

**ATOM-715.3-001**
- **Rule:** 715.3 — Player chooses to play adventurer card normally or as an Adventure
- **Mechanism:** Cast-mode choice via DecisionProvider
- **Minimal Board:** Player A has an adventurer card in hand with sufficient mana for either mode.
- **Action:** Player A casts the card.
- **Expected Result:** DecisionProvider is asked to choose between normal cast and Adventure cast. The chosen mode determines which characteristics are used on the stack.
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**715.3a** — TESTABLE. Only alternative characteristics evaluated for Adventure cast legality.

**ATOM-715.3a-001**
- **Rule:** 715.3a — When casting as an Adventure, only the alternative characteristics determine cast legality
- **Mechanism:** Cast legality check uses Adventure characteristics (mana cost, timing) not normal characteristics
- **Minimal Board:** Player A has an adventurer card in hand. Normal: Creature {3}{G}{G}. Adventure: Instant {G}. Player A has only {G} available.
- **Action:** Player A attempts to cast the card as an Adventure.
- **Expected Result:** The cast is legal — the Adventure's mana cost ({G}) is evaluated, not the creature's ({3}{G}{G}).
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**715.3b** — TESTABLE. On stack as Adventure, spell has ONLY alternative characteristics.

**ATOM-715.3b-001**
- **Rule:** 715.3b — While on the stack as an Adventure, the spell has only its alternative characteristics
- **Mechanism:** Characteristic query on Adventure spell returns only Adventure characteristics
- **Minimal Board:** An adventurer card is on the stack, cast as an Adventure. Normal: Creature. Adventure: Instant.
- **Action:** An effect checks whether there is an "instant spell" on the stack.
- **Expected Result:** The Adventure spell IS found as an instant. It is NOT found as a creature.
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**715.3c** — TESTABLE. Copy of Adventure spell is also an Adventure.

**ATOM-715.3c-001**
- **Rule:** 715.3c — If an Adventure spell is copied, the copy is also an Adventure with alternative characteristics
- **Mechanism:** Spell copy preserves Adventure mode
- **Minimal Board:** An adventurer card is on the stack, cast as an Adventure (Instant {G} "deal 2 damage"). An effect copies this spell.
- **Action:** The copy is created on the stack.
- **Expected Result:** The copy is also an Adventure. It has the alternative characteristics, not the normal creature characteristics.
- **Phase:** Phase 9 (D4) + Phase 7 (spell copying D19)
- **Ticket:** D4 + D19

**715.3d** — TESTABLE. Resolution destination: exile instead of graveyard, with re-cast permission.

**ATOM-715.3d-001**
- **Rule:** 715.3d — An Adventure spell resolves to exile (not graveyard), and may be played from exile (not as Adventure)
- **Mechanism:** Resolution zone override + exile play permission
- **Minimal Board:** An adventurer card is on the stack, cast as an Adventure.
- **Action:** The Adventure spell resolves.
- **Expected Result:** The card is exiled (not put into graveyard). The controller gains permission to play the card from exile. The card CANNOT be cast as an Adventure from exile via this permission.
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**715.4** — TESTABLE. In all zones except the stack (as Adventure), only normal characteristics apply.

**ATOM-715.4-001**
- **Rule:** 715.4 — In zones other than the stack (or on the stack not as Adventure), adventurer card has only normal characteristics
- **Mechanism:** Zone-dependent characteristic masking
- **Minimal Board:** Player A has an adventurer card in hand. Normal: Creature 2/2 {1}{G}. Adventure: Instant {G}.
- **Action:** An effect checks the characteristics of the card in hand.
- **Expected Result:** The card has only its normal characteristics (Creature, 2/2, {1}{G}). The Adventure characteristics are not visible for game-mechanical queries (e.g., searching for "an instant card in your hand" does not find it).
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**715.5** — TESTABLE. Player may choose an adventurer card's alternative name when choosing a card name.

**ATOM-715.5-001**
- **Rule:** 715.5 — A player may choose an adventurer card's alternative name when instructed to choose a card name
- **Mechanism:** Card name selection allows Adventure names
- **Minimal Board:** An effect instructs Player A to choose a card name (e.g., Pithing Needle).
- **Action:** Player A chooses the Adventure name of an adventurer card.
- **Expected Result:** The choice is legal. The named Adventure name functions for any effects that reference the chosen name.
- **Phase:** Phase 9 (D4) + Phase 8 (D18 — card name choice)
- **Ticket:** D4 + D18

---

### 716. Class Cards

**716.1** — PURE-DEF. Describes card frame layout.

**716.2** — TESTABLE. Class level bar defines both an activated ability and a static ability.

**ATOM-716.2-001**
- **Rule:** 716.2 — A class level bar represents an activated ability (to gain the level) and a static ability (granting abilities at that level)
- **Mechanism:** Class level-up activation + conditional static ability
- **Minimal Board:** Player A controls a Class enchantment at level 1. It has a level bar: "{1}{W}: Level 2 — Creatures you control get +1/+1." Player A has {1}{W} available.
- **Action:** Player A activates the level 2 ability.
- **Expected Result:** The Class becomes level 2. Its static ability is now active: creatures Player A controls get +1/+1.
- **Phase:** Phase 8 (card breadth — Class card type)
- **Ticket:** NEW — Class level-up activation and static ability (716.2)

**716.2a** — TESTABLE. Defines the activation restriction (only from level N-1, sorcery speed).

**ATOM-716.2a-001**
- **Rule:** 716.2a — Class level activation is restricted to level N-1 and sorcery speed
- **Mechanism:** Activation restriction enforcement
- **Minimal Board:** Player A controls a Class enchantment at level 1. It has level 2 and level 3 bars.
- **Action:** Player A attempts to activate the level 3 ability while the Class is at level 1.
- **Expected Result:** Activation is illegal — must be at level 2 to activate level 3. Similarly, activating during opponent's turn or with a non-empty stack is illegal (sorcery speed).
- **Phase:** Phase 8
- **Ticket:** NEW — Class level activation restriction (716.2a)

**716.2b** — TESTABLE. Level is a designation, persists even if permanent stops being a Class. Not copiable.

**ATOM-716.2b-001**
- **Rule:** 716.2b — A Class retains its level even if it stops being a Class; level is not copiable
- **Mechanism:** Designation persistence across type changes; copy does not copy level
- **Minimal Board:** Player A controls a Class enchantment at level 3. An effect removes the Class type from it.
- **Action:** Check the permanent's level designation.
- **Expected Result:** The permanent still has level 3, even though it's no longer a Class. If another effect copies this permanent, the copy does NOT inherit the level designation.
- **Phase:** Phase 8 + Phase 5 (layers — type change)
- **Ticket:** NEW — Class level designation persistence and non-copiability (716.2b)

**716.2c** — BOUNDARY-DEF. Defines "to gain a Class level" = "to activate a class level bar ability." Referenced by mana spending restrictions (e.g., "Spend this mana only to cast an instant or sorcery spell or to gain a Class level.").

**ATOM-716.2c-001**
- **Rule:** 716.2c — Mana spending restriction recognizes "gain a Class level" as a valid category
- **Mechanism:** Mana spending restriction validation against Class level activation
- **Minimal Board:** Player A controls a creature with "{T}: Add {U} or {R}. Spend this mana only to cast an instant or sorcery spell or to gain a Class level." Player A also controls a Class at level 1 with level 2 cost "{1}{U}."
- **Action:** Player A taps the creature for {U}, then uses it to activate the Class level 2 ability.
- **Expected Result:** Legal. "Gain a Class level" = "activate a class level bar ability" per 716.2c, and the mana spending restriction permits this use.
- **Phase:** Phase 5-Pre (T17 mana spending restrictions) + Phase 8
- **Ticket:** T17 + NEW — Class level activation as mana spending category (716.2c)

**716.2d** — TESTABLE. A permanent without a level designation is treated as level 1.

**ATOM-716.2d-001**
- **Rule:** 716.2d — A permanent without a level designation is treated as though its level is 1
- **Mechanism:** Default level value for permanents that don't have one
- **Minimal Board:** Player A controls a Class enchantment that just entered the battlefield (no level-up activations yet).
- **Action:** An effect references "this permanent's level."
- **Expected Result:** The permanent is treated as level 1.
- **Phase:** Phase 8
- **Ticket:** NEW — Default level = 1 for permanents without level (716.2d)

**716.3** — TESTABLE. Non-class-level-bar abilities on a Class are always active.

**ATOM-716.3-001**
- **Rule:** 716.3 — Abilities on a Class card not preceded by a class level bar are active at all times
- **Mechanism:** Unconditional ability presence on Class cards
- **Minimal Board:** Player A controls a Class enchantment at level 1. The top text box has "When this enchantment enters the battlefield, draw a card." and a static ability "You gain 1 life whenever you cast a spell."
- **Action:** Check which abilities the Class has at level 1.
- **Expected Result:** The top-section abilities are active regardless of level. The level 2/3 abilities are NOT active.
- **Phase:** Phase 8
- **Ticket:** NEW — Class top-section abilities always active (716.3)

**716.4** — PURE-DEF. Distinguishes Class cards from leveler cards (711). No testable interaction — they use completely separate systems (level counters vs level designation).

---

### 717. Attraction Cards — OUT-OF-SCOPE

All sub-rules (717.1, 717.2, 717.2a, 717.2b, 717.3, 717.4, 717.5, 717.6, 717.6a) are **OUT-OF-SCOPE**. Attractions are an Un-set mechanic from Unfinity with non-standard card backs and dice rolling.

---

### 718. Prototype Cards

**718.1** — PURE-DEF. Describes card frame layout.

**718.2** — TESTABLE (zone-dependent characteristics). Alternative mana cost, power, and toughness apply while spell or permanent on battlefield.

**ATOM-718.2-001**
- **Rule:** 718.2 — Prototype's alternative characteristics (mana cost, P/T) apply while on stack as prototyped spell or on battlefield as prototyped permanent
- **Mechanism:** CharacteristicOverrides for prototyped spells/permanents
- **Minimal Board:** Player A has a prototype card in hand. Normal: 7/7 {7}. Prototype: 3/3 {1}{R}{R}.
- **Action:** Player A casts it as a prototyped spell.
- **Expected Result:** On the stack, the spell has mana cost {1}{R}{R}, and will have P/T 3/3. On the battlefield after resolution, the permanent has P/T 3/3 and mana cost {1}{R}{R}.
- **Phase:** Phase 9 (Prototype — CharacteristicOverrides on zone sidecars)
- **Ticket:** NEW — Prototype alternative characteristics on stack/battlefield (718.2)

**718.2a** — TESTABLE. Alternative characteristics are copiable values.

**ATOM-718.2a-001**
- **Rule:** 718.2a — The existence and values of prototype alternative characteristics are part of the object's copiable values
- **Mechanism:** Copy of prototype card includes alternative characteristic data
- **Minimal Board:** Player A controls a prototyped permanent (3/3 {1}{R}{R}). A Clone effect copies it.
- **Action:** Clone enters as a copy.
- **Expected Result:** The copy has the prototype alternative characteristics as copiable values. The copy is 3/3 with mana cost {1}{R}{R} (copies the prototyped state).
- **Phase:** Phase 9 + Phase 6 (copy)
- **Ticket:** NEW — Prototype copiable values (718.2a)

**718.3** — TESTABLE. Cast-time mode choice.

**ATOM-718.3-001**
- **Rule:** 718.3 — Player chooses to cast a prototype card normally or as a prototyped spell
- **Mechanism:** Cast-mode choice via DecisionProvider
- **Minimal Board:** Player A has a prototype card in hand. Normal: {7}. Prototype: {1}{R}{R}. Player A has {1}{R}{R} but not {7}.
- **Action:** Player A casts the card.
- **Expected Result:** DecisionProvider is asked to choose between normal and prototype cast. Only prototype is affordable, so choosing normal would fail the cost check.
- **Phase:** Phase 9
- **Ticket:** NEW — Prototype cast mode choice (718.3)

**718.3a** — TESTABLE. Only alternative P/T and mana cost are used for cast legality of prototyped spell.

**ATOM-718.3a-001**
- **Rule:** 718.3a — While casting a prototyped spell, only alternative P/T and mana cost are evaluated for cast legality
- **Mechanism:** Cast legality uses prototype characteristics
- **Minimal Board:** Player A has a prototype card in hand. Normal: {7}. Prototype: {1}{R}{R}. Player A has {1}{R}{R}.
- **Action:** Player A casts it as a prototyped spell.
- **Expected Result:** Cast is legal — the prototype mana cost ({1}{R}{R}) is used, not the normal cost ({7}).
- **Phase:** Phase 9
- **Ticket:** NEW — Prototype cast legality uses alternative cost (718.3a)

**718.3b** — TESTABLE. Prototyped spell and permanent have only alternative P/T, mana cost, and derived color.

**ATOM-718.3b-001**
- **Rule:** 718.3b — A prototyped permanent has only the alternative mana cost, and its color is derived from that mana cost
- **Mechanism:** Color derivation from prototype mana cost
- **Minimal Board:** Player A controls a prototyped permanent. Normal: {7} (colorless artifact creature). Prototype: {1}{R}{R}.
- **Action:** Check the permanent's color.
- **Expected Result:** The permanent is red (derived from {R}{R} in prototype mana cost), not colorless.
- **Phase:** Phase 9
- **Ticket:** NEW — Prototype color derivation from alternative mana cost (718.3b)

**718.3c** — TESTABLE. Copy of prototyped spell is also prototyped.

**ATOM-718.3c-001**
- **Rule:** 718.3c — If a prototyped spell is copied, the copy is also a prototyped spell with alternative characteristics
- **Mechanism:** Spell copy preserves prototype mode
- **Minimal Board:** A prototyped spell is on the stack (3/3 {1}{R}{R}). An effect copies it.
- **Action:** Copy is created.
- **Expected Result:** The copy is also prototyped — it has 3/3 P/T and {1}{R}{R} mana cost, not the normal 7/7 and {7}.
- **Phase:** Phase 9 + Phase 7 (spell copying D19)
- **Ticket:** NEW — Prototype spell copy (718.3c) + D19

**718.3d** — TESTABLE. Copy of prototyped permanent has alternative characteristics.

**ATOM-718.3d-001**
- **Rule:** 718.3d — If a permanent that was a prototyped spell is copied, the copy has the alternative P/T and mana cost
- **Mechanism:** Permanent copy respects prototype state
- **Minimal Board:** Player A controls a prototyped permanent (3/3 {1}{R}{R}). A Clone enters copying it.
- **Action:** Clone becomes a copy.
- **Expected Result:** Clone is 3/3 with mana cost {1}{R}{R}, not 7/7 with {7}.
- **Phase:** Phase 9 + Phase 6 (copy)
- **Ticket:** NEW — Prototype permanent copy (718.3d)

**718.4** — TESTABLE. In zones other than stack/battlefield (or on stack/battlefield when not prototyped), only normal characteristics.

**ATOM-718.4-001**
- **Rule:** 718.4 — A prototype card in hand/graveyard/library has only normal characteristics
- **Mechanism:** Zone-dependent characteristic masking
- **Minimal Board:** Player A has a prototype card in their graveyard. Normal: 7/7 {7}. Prototype: 3/3 {1}{R}{R}.
- **Action:** An effect searches for "a creature card with mana value 7 or greater in your graveyard."
- **Expected Result:** The prototype card is found — in the graveyard, it has its normal characteristics ({7}, mana value 7, P/T 7/7).
- **Phase:** Phase 9
- **Ticket:** NEW — Prototype normal characteristics in non-stack/battlefield zones (718.4)

**ATOM-718.4-002**
- **Rule:** 718.4 — A prototype card on the battlefield NOT cast as prototyped has only normal characteristics
- **Mechanism:** Non-prototyped permanent uses normal characteristics
- **Minimal Board:** Player A casts a prototype card normally (paying {7}). It resolves to the battlefield.
- **Action:** Check the permanent's P/T and mana cost.
- **Expected Result:** The permanent is 7/7 with mana cost {7} (normal characteristics, not prototype).
- **Phase:** Phase 9
- **Ticket:** NEW — Non-prototyped permanent uses normal characteristics (718.4)

**718.5** — TESTABLE. Non-P/T/mana-cost/color characteristics are unchanged by prototype mode.

**ATOM-718.5-001**
- **Rule:** 718.5 — A prototype card's types, abilities, name, etc. remain the same regardless of cast mode
- **Mechanism:** Prototype only overrides P/T, mana cost, and color; other characteristics unchanged
- **Minimal Board:** Player A controls a prototyped permanent. The card has type "Artifact Creature — Construct" and ability "Trample." Normal: 7/7 {7}. Prototype: 3/3 {1}{R}{R}.
- **Action:** Check the permanent's types and abilities.
- **Expected Result:** The permanent is still "Artifact Creature — Construct" with Trample. Only P/T (3/3), mana cost ({1}{R}{R}), and color (red) differ from normal.
- **Phase:** Phase 9
- **Ticket:** NEW — Prototype non-overridden characteristics unchanged (718.5)

---

### 719. Case Cards

**719.1** — PURE-DEF. Describes card frame layout.

**719.2** — PURE-DEF. "The Case frame has no additional rules meaning."

**719.3** — PURE-DEF. Defines the two keyword abilities on Case cards. Prerequisite for 719.3a–c.

**719.3a** — TESTABLE. Defines the "To solve" triggered ability.

**ATOM-719.3a-001**
- **Rule:** 719.3a — "To solve — [Condition]" triggers at the beginning of your end step to solve the Case if condition is met
- **Mechanism:** End-step triggered ability checking solve condition
- **Minimal Board:** Player A controls a Case enchantment (not solved). The Case's solve condition is "You control three or more creatures." Player A controls 3 creatures.
- **Action:** Player A's end step begins.
- **Expected Result:** The "To solve" ability triggers and resolves. The Case becomes solved (gains the solved designation).
- **Phase:** Phase 7 (triggered abilities) + Phase 8 (card breadth)
- **Ticket:** NEW — Case "To solve" triggered ability (719.3a)

**ATOM-719.3a-002**
- **Rule:** 719.3a — "To solve" does NOT trigger if the Case is already solved
- **Mechanism:** Trigger suppression when already solved
- **Minimal Board:** Player A controls a Case enchantment that is already solved.
- **Action:** Player A's end step begins.
- **Expected Result:** The "To solve" ability does NOT trigger (Case is already solved).
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Case solve trigger suppression when already solved (719.3a)

**719.3b** — TESTABLE. Solved is a designation: persists until leaves battlefield, not copiable.

**ATOM-719.3b-001**
- **Rule:** 719.3b — Solved designation persists until the permanent leaves the battlefield and is not copiable
- **Mechanism:** Designation persistence and non-copiability
- **Minimal Board:** Player A controls a solved Case enchantment. A Clone-type effect copies the Case.
- **Action:** The copy enters the battlefield.
- **Expected Result:** The copy is NOT solved (designation is not a copiable value). The original remains solved.
- **Phase:** Phase 8 + Phase 6 (copy)
- **Ticket:** NEW — Solved designation persistence and non-copiability (719.3b)

**719.3c** — TESTABLE. "Solved — [Ability]" is active only when the Case has the solved designation.

**ATOM-719.3c-001**
- **Rule:** 719.3c — A Case's "Solved —" ability is active only when the Case has the solved designation
- **Mechanism:** Conditional ability activation based on solved designation
- **Minimal Board:** Player A controls a Case enchantment (NOT solved). The Case has "Solved — Creatures you control get +2/+2."
- **Action:** Check if creatures get the +2/+2 bonus.
- **Expected Result:** Creatures do NOT get +2/+2 (Case is not solved). After the Case becomes solved, the static ability becomes active and creatures get +2/+2.
- **Phase:** Phase 8
- **Ticket:** NEW — Case solved ability conditional activation (719.3c)

---

### 720. Omen Cards

**720.1** — PURE-DEF. Describes card frame layout.

**720.2** — TESTABLE (zone-dependent characteristics, parallel to 715.2 for Adventure).

**ATOM-720.2-001**
- **Rule:** 720.2 — Omen card's alternative characteristics define what the object has while it's an Omen spell
- **Mechanism:** Zone-dependent characteristic selection for Omen spells
- **Minimal Board:** Player A has an omen card in hand. Normal: Creature. Omen: Sorcery.
- **Action:** Player A casts the card as an Omen.
- **Expected Result:** On the stack, the spell has the Omen characteristics (Sorcery type, Omen mana cost, Omen effect).
- **Phase:** Phase 9 (D4 — CardLayout restructuring)
- **Ticket:** D4

**720.2a** — TESTABLE. "Has an Omen" reference works even when not using alternative characteristics (parallel to 715.2a).

**ATOM-720.2a-001**
- **Rule:** 720.2a — An effect referring to an object that "has an Omen" finds the omen card even when not using alternative characteristics
- **Mechanism:** Object query for "has an Omen" flag
- **Minimal Board:** Player A has an omen card in their graveyard.
- **Action:** An effect searches for "a card that has an Omen."
- **Expected Result:** The omen card qualifies (it "has an Omen" even though it's using normal characteristics in the graveyard).
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**720.2b** — TESTABLE. Omen alternative characteristics are copiable values (parallel to 715.2b).

**ATOM-720.2b-001**
- **Rule:** 720.2b — The alternative characteristics of an omen card are part of its copiable values
- **Mechanism:** Copy effect preserves Omen characteristics
- **Minimal Board:** A Clone-type effect copies an omen card.
- **Action:** Copy is made.
- **Expected Result:** The copy has the same Omen alternative characteristics as the original.
- **Phase:** Phase 9 (D4) + Phase 6 (copy)
- **Ticket:** D4

**720.2c** — PURE-DEF. An omen card is one card, not two. No independent engine consequence.

**720.3** — TESTABLE. Mode choice at cast time (parallel to 715.3).

**ATOM-720.3-001**
- **Rule:** 720.3 — Player chooses to cast omen card normally or as an Omen
- **Mechanism:** Cast-mode choice via DecisionProvider
- **Minimal Board:** Player A has an omen card in hand.
- **Action:** Player A casts the card.
- **Expected Result:** DecisionProvider is asked to choose between normal and Omen cast.
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**720.3a** — TESTABLE. Only alternative characteristics evaluated for Omen cast legality (parallel to 715.3a).

**ATOM-720.3a-001**
- **Rule:** 720.3a — When casting as an Omen, only the alternative characteristics determine cast legality
- **Mechanism:** Cast legality check uses Omen characteristics
- **Minimal Board:** Player A has an omen card in hand. Normal: Creature {4}{G}{G}. Omen: Sorcery {1}{G}.
- **Action:** Player A casts as an Omen with {1}{G} available.
- **Expected Result:** Cast is legal — the Omen's mana cost is evaluated.
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**720.3b** — TESTABLE. On stack as Omen, spell has only alternative characteristics (parallel to 715.3b).

**ATOM-720.3b-001**
- **Rule:** 720.3b — While on the stack as an Omen, the spell has only its alternative characteristics
- **Mechanism:** Characteristic masking on stack
- **Minimal Board:** An omen card is on the stack as an Omen spell. Normal: Creature. Omen: Sorcery.
- **Action:** An effect checks for "a sorcery spell on the stack."
- **Expected Result:** The Omen spell qualifies as a sorcery, not a creature.
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**720.3c** — TESTABLE. Copy of Omen spell is also an Omen (parallel to 715.3c).

**ATOM-720.3c-001**
- **Rule:** 720.3c — If an Omen spell is copied, the copy is also an Omen
- **Mechanism:** Spell copy preserves Omen mode
- **Minimal Board:** An Omen spell is on the stack. An effect copies it.
- **Action:** Copy is created.
- **Expected Result:** The copy is also an Omen with alternative characteristics.
- **Phase:** Phase 9 (D4) + Phase 7 (D19)
- **Ticket:** D4 + D19

**720.3d** — TESTABLE. Resolution destination: shuffle into library instead of graveyard.

**ATOM-720.3d-001**
- **Rule:** 720.3d — An Omen spell resolves by being shuffled into its owner's library instead of going to graveyard
- **Mechanism:** Resolution zone override — library shuffle instead of graveyard
- **Minimal Board:** An omen card is on the stack as an Omen spell.
- **Action:** The Omen spell resolves.
- **Expected Result:** The card is shuffled into its owner's library (not put into graveyard).
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**720.4** — TESTABLE. In all zones except the stack (as Omen), only normal characteristics (parallel to 715.4).

**ATOM-720.4-001**
- **Rule:** 720.4 — In zones other than the stack (as Omen), omen card has only normal characteristics
- **Mechanism:** Zone-dependent characteristic masking
- **Minimal Board:** Player A has an omen card in hand. Normal: Creature. Omen: Sorcery.
- **Action:** An effect searches for "a creature card in your hand."
- **Expected Result:** The omen card is found as a creature (normal characteristics in hand).
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**720.5** — TESTABLE. Player may choose omen card's alternative name (parallel to 715.5).

**ATOM-720.5-001**
- **Rule:** 720.5 — A player may choose an omen card's alternative name when instructed to choose a card name
- **Mechanism:** Card name selection allows Omen names
- **Minimal Board:** An effect instructs Player A to choose a card name.
- **Action:** Player A chooses the Omen name.
- **Expected Result:** The choice is legal.
- **Phase:** Phase 9 (D4) + Phase 8 (D18)
- **Ticket:** D4 + D18

---

### 721. Station Cards

**721.1** — PURE-DEF. Describes card frame layout and station symbols.

**721.2** — PURE-DEF. Defines station symbol as a static ability. Prerequisite for 721.2a–c.

**721.2a** — TESTABLE. Station threshold ability: "As long as this permanent has N or more charge counters, it has [abilities]."

**ATOM-721.2a-001**
- **Rule:** 721.2a — "{N+}[abilities]" means the permanent has [abilities] when it has N or more charge counters
- **Mechanism:** Counter-threshold conditional static ability
- **Minimal Board:** Player A controls a Station permanent with "{3+} — This permanent has flying." It has 2 charge counters.
- **Action:** Check if the permanent has flying.
- **Expected Result:** No flying (only 2 charge counters, needs 3). After adding a third charge counter, the permanent gains flying.
- **Phase:** Phase 8 (card breadth — Station card type)
- **Ticket:** NEW — Station counter-threshold ability grant (721.2a)

**721.2b** — TESTABLE. Station threshold with creature transformation: gains abilities AND becomes a creature with base P/T.

**ATOM-721.2b-001**
- **Rule:** 721.2b — "{N+}[abilities][P/T]" means the permanent gains [abilities] and becomes a creature with base P/T when at N+ charge counters
- **Mechanism:** Counter-threshold type addition + P/T setting
- **Minimal Board:** Player A controls a Station artifact with "{5+} — Trample [4/4]". It entered the battlefield this turn. It has 4 charge counters (not a creature).
- **Action:** A charge counter is added (4 → 5).
- **Expected Result:** The permanent is now an artifact creature with base P/T 4/4 and Trample, in addition to its other types. It has summoning sickness (entered this turn).
- **Phase:** Phase 8
- **Ticket:** NEW — Station counter-threshold creature transformation (721.2b)

**721.2c** — TESTABLE. Station cards have no P/T in zones other than battlefield.

**ATOM-721.2c-001**
- **Rule:** 721.2c — Station cards do not have power or toughness while in any zone other than the battlefield
- **Mechanism:** Zone-dependent P/T absence
- **Minimal Board:** A Station card is in Player A's graveyard. The card has a station symbol with [4/4].
- **Action:** An effect searches for "a creature card with power 4 in your graveyard."
- **Expected Result:** The Station card is NOT found — it has no P/T in the graveyard.
- **Phase:** Phase 8
- **Ticket:** NEW — Station no P/T in non-battlefield zones (721.2c)

**721.3** — PURE-DEF. Text box striations have no game significance.

**721.4** — TESTABLE. Non-station-symbol abilities are always active.

**ATOM-721.4-001**
- **Rule:** 721.4 — Abilities not preceded by a station symbol are active at all times, including the station keyword ability
- **Mechanism:** Unconditional ability presence on Station cards
- **Minimal Board:** Player A controls a Station artifact with 0 charge counters. The card has the station keyword ability (702.184) and a "{3+}" ability.
- **Action:** Player A attempts to activate the station ability.
- **Expected Result:** The station ability can be activated regardless of charge counter count. The "{3+}" abilities are NOT active (0 < 3).
- **Phase:** Phase 8
- **Ticket:** NEW — Station base abilities always active (721.4)

--- End of Chunk 1 ---

## Chunk 2: Rules 722–727

### 722. Controlling Another Player

**722.1** — TESTABLE. Player-controlling effect applies to the next turn actually taken, lasts entire turn.

**ATOM-722.1-001**
- **Rule:** 722.1 — A player-controlling effect applies to the controlled player's next actual turn and lasts until the beginning of the next turn after that
- **Mechanism:** Player control designation tracking with turn-scoped duration
- **Minimal Board:** Player A resolves an effect that says "You control target player during that player's next turn." targeting Player B.
- **Action:** Player B's next turn begins.
- **Expected Result:** Player A controls Player B for the entire turn. The effect ends at the beginning of the turn after Player B's controlled turn.
- **Phase:** Phase 8 (controlling another player)
- **Ticket:** NEW — Player control effect duration (722.1)

**722.1a** — TESTABLE. Multiple player-controlling effects on the same player: last one wins.

**ATOM-722.1a-001**
- **Rule:** 722.1a — Multiple player-controlling effects on the same player overwrite each other; last created wins
- **Mechanism:** Player control overwrite tracking
- **Minimal Board:** Player A resolves an effect to control Player C next turn. Then Player B resolves an effect to control Player C next turn.
- **Action:** Player C's next turn begins.
- **Expected Result:** Player B controls Player C (last effect created wins). Player A's control effect is overwritten.
- **Phase:** Phase 8
- **Ticket:** NEW — Player control overwrite (722.1a)

**722.1b** — TESTABLE. Skipped turns don't consume the control effect.

**ATOM-722.1b-001**
- **Rule:** 722.1b — If a turn is skipped, pending player-controlling effects wait until the player actually takes a turn
- **Mechanism:** Control effect persists through skipped turns
- **Minimal Board:** Player A has a pending control effect on Player B. An effect causes Player B to skip their next turn.
- **Action:** Player B's turn is skipped.
- **Expected Result:** The control effect is NOT consumed. It waits and applies to the next turn Player B actually takes.
- **Phase:** Phase 8
- **Ticket:** NEW — Player control persists through skipped turns (722.1b)

**722.2** — PURE-DEF. Names two specific cards that allow limited-duration player control. No independent mechanical consequence beyond 722.1.

**722.3** — TESTABLE. Objects remain controlled by their normal controllers; controlled player is still the active player.

**ATOM-722.3-001**
- **Rule:** 722.3 — Only control of the player changes; objects keep their normal controllers; controlled player is still active player
- **Mechanism:** Player control does not transfer object control; active player unchanged
- **Minimal Board:** Player A controls Player B during Player B's turn. Player B controls a creature.
- **Action:** Check who controls the creature and who is the active player.
- **Expected Result:** Player B still controls the creature (not Player A). Player B is still the active player. Player A makes decisions FOR Player B but doesn't become the controller of Player B's permanents.
- **Phase:** Phase 8
- **Ticket:** NEW — Player control vs object control distinction (722.3)

**722.4** — TESTABLE. Information visibility: in-game objects visible to both, outside-game info visible only to controlled player. Has Example.

**ATOM-722.4-001**
- **Rule:** 722.4 — The controller of a player can see that player's hand and face-down creatures they control
- **Mechanism:** Information visibility extension for player controller
- **Minimal Board:** Player A controls Player B. Player B has cards in hand and a face-down creature.
- **Action:** Player A queries hidden information about Player B's game objects.
- **Expected Result:** Player A can see Player B's hand and the face of Player B's face-down creatures. Cards outside the game that Player B could see remain hidden from Player A.
- **Phase:** Phase 8
- **Ticket:** NEW — Player control information visibility (722.4)

**722.5** — TESTABLE. Controller makes ALL choices and decisions for the controlled player. Has Examples.

**ATOM-722.5-001**
- **Rule:** 722.5 — The controller of another player makes all game choices and decisions for the controlled player
- **Mechanism:** DecisionProvider routing — all decision calls for the controlled player go to the controlling player's DecisionProvider
- **Minimal Board:** Player A controls Player B. Player B has a creature and a spell in hand.
- **Action:** During Player B's main phase, Player A decides Player B casts the spell and chooses targets.
- **Expected Result:** All decision points (priority actions, target choices, attacker declarations, damage assignment) for Player B are routed to Player A's DecisionProvider during this turn.
- **Phase:** Phase 8
- **Ticket:** NEW — Player control decision routing (722.5)

**722.5a** — TESTABLE. Controller can only use the controlled player's resources. Has Example.

**ATOM-722.5a-001**
- **Rule:** 722.5a — The controller of another player can only use that player's resources to pay costs
- **Mechanism:** Resource isolation for controlled player
- **Minimal Board:** Player A controls Player B. Player A has 5 mana in their pool. Player B has 2 mana in their pool. A spell costs {3}.
- **Action:** Player A decides Player B casts the {3} spell.
- **Expected Result:** The cast is illegal — Player B only has 2 mana. Player A's mana cannot be used to pay Player B's costs.
- **Phase:** Phase 8
- **Ticket:** NEW — Player control resource isolation (722.5a)

**722.5b** — PURE-DEF. Controller can't make non-game choices (restroom, trading, etc.). Tournament-only concern, not engine-testable.

**722.6** — TESTABLE. Controller can't force concession.

**ATOM-722.6-001**
- **Rule:** 722.6 — The controller of another player can't make that player concede
- **Mechanism:** Concession immunity for controlled players
- **Minimal Board:** Player A controls Player B.
- **Action:** Player A attempts to make Player B concede.
- **Expected Result:** The action is illegal. Only Player B themselves can choose to concede.
- **Phase:** Phase 8
- **Ticket:** NEW — Player control cannot force concession (722.6)

**722.7** — TESTABLE. The control effect may restrict or mandate specific actions.

**ATOM-722.7-001**
- **Rule:** 722.7 — The player-controlling effect may restrict or specify actions the controlled player must take
- **Mechanism:** Effect-specific action restrictions/mandates during player control
- **Minimal Board:** An effect says "Control target player during their next turn. That player can't cast spells this turn." Player A controls Player B under this effect.
- **Action:** Player A attempts to have Player B cast a spell.
- **Expected Result:** The cast is illegal — the effect restricts spell casting.
- **Phase:** Phase 8
- **Ticket:** NEW — Player control effect action restrictions (722.7)

**722.8** — PURE-DEF. The controlling player continues to make their own decisions. No additional mechanical consequence — the engine naturally handles this via separate DecisionProvider routing.

**722.9** — TESTABLE. A player can be given control of themselves (no-op).

**ATOM-722.9-001**
- **Rule:** 722.9 — An effect may give a player control of themselves; they make their own decisions as normal
- **Mechanism:** Self-control is a valid no-op state
- **Minimal Board:** An effect gives Player A control of themselves.
- **Action:** Player A's turn proceeds.
- **Expected Result:** Player A makes their own decisions as normal. No behavioral change.
- **Phase:** Phase 8
- **Ticket:** NEW — Player self-control no-op (722.9)

---

### 723. Ending Turns and Phases

**723.1** — TESTABLE. Defines the "end the turn" procedure as a multi-step sequence.

**ATOM-723.1-001**
- **Rule:** 723.1 — When an effect ends the turn, a specific multi-step procedure is followed
- **Mechanism:** End-the-turn procedure entry point
- **Minimal Board:** Player A is in their main phase. A spell on the stack says "End the turn."
- **Action:** The spell resolves.
- **Expected Result:** The end-the-turn procedure (723.1a–f) is followed in order.
- **Phase:** Phase 8 (end-the-turn)
- **Ticket:** NEW — End-the-turn procedure (723.1)

**723.1a** — TESTABLE. Pending (not-yet-stacked) triggered abilities cease to exist.

**ATOM-723.1a-001**
- **Rule:** 723.1a — Triggered abilities that triggered before the end-the-turn process but haven't been stacked yet cease to exist
- **Mechanism:** Pending trigger purge during end-the-turn
- **Minimal Board:** A creature died this turn, causing a "whenever a creature dies" trigger. Before that trigger is put on the stack, an effect ends the turn.
- **Action:** The end-the-turn process begins.
- **Expected Result:** The pending "dies" trigger ceases to exist and is never put on the stack.
- **Phase:** Phase 8
- **Ticket:** NEW — End-the-turn pending trigger purge (723.1a)

**723.1b** — TESTABLE. Exile every object on the stack.

**ATOM-723.1b-001**
- **Rule:** 723.1b — Exile every object on the stack, including the resolving object
- **Mechanism:** Stack exile during end-the-turn
- **Minimal Board:** The stack contains: the end-the-turn spell (resolving), plus two other spells below it.
- **Action:** The end-the-turn process executes step 723.1b.
- **Expected Result:** All three objects on the stack (including the end-the-turn spell itself) are exiled. Non-card objects (copies) will cease to exist at next SBA check.
- **Phase:** Phase 8
- **Ticket:** NEW — End-the-turn stack exile (723.1b)

**723.1c** — TESTABLE. SBAs are checked, but no priority and no triggers stacked.

**ATOM-723.1c-001**
- **Rule:** 723.1c — State-based actions are checked during end-the-turn; no player gets priority and no triggered abilities are stacked
- **Mechanism:** SBA check with trigger/priority suppression
- **Minimal Board:** A creature has lethal damage marked on it. The end-the-turn process reaches step 723.1c.
- **Action:** SBAs are checked.
- **Expected Result:** The creature is destroyed (SBA). But any triggers that would result (e.g., "dies" triggers) are NOT put on the stack. No player gets priority.
- **Phase:** Phase 8
- **Ticket:** NEW — End-the-turn SBA check with suppression (723.1c)

**723.1d** — TESTABLE. Current phase/step ends, skip to cleanup. Combat creatures removed.

**ATOM-723.1d-001**
- **Rule:** 723.1d — The current phase ends and the game skips to the cleanup step; if during combat, creatures are removed from combat
- **Mechanism:** Phase skip to cleanup + combat removal
- **Minimal Board:** It is the combat damage step. Two creatures are in combat. An effect ended the turn.
- **Action:** Step 723.1d executes.
- **Expected Result:** All creatures and planeswalkers are removed from combat. The game skips directly to the cleanup step — the postcombat main phase, end step, etc. are all skipped.
- **Phase:** Phase 8
- **Ticket:** NEW — End-the-turn phase skip to cleanup (723.1d)

**ATOM-723.1d-002**
- **Rule:** 723.1d — If end-the-turn happens during cleanup, a new cleanup step begins
- **Mechanism:** Recursive cleanup on end-the-turn during cleanup
- **Minimal Board:** During the cleanup step, a triggered ability causes an effect that ends the turn.
- **Action:** The end-the-turn process executes during cleanup.
- **Expected Result:** A new cleanup step begins (does not loop infinitely — the new cleanup proceeds normally unless further triggers occur).
- **Phase:** Phase 8
- **Ticket:** NEW — End-the-turn during cleanup creates new cleanup (723.1d)

**723.1e** — TESTABLE. "At the beginning of the end step" triggers DON'T trigger because the end step is skipped.

**ATOM-723.1e-001**
- **Rule:** 723.1e — "At the beginning of the end step" triggers don't fire because the end step is skipped
- **Mechanism:** End-step trigger suppression when turn is ended early
- **Minimal Board:** Player A controls a permanent with "At the beginning of your end step, draw a card." An effect ends the turn during the main phase.
- **Action:** The end-the-turn process completes, skipping to cleanup.
- **Expected Result:** The "at the beginning of your end step" ability does NOT trigger (the end step was skipped).
- **Phase:** Phase 8
- **Ticket:** NEW — End-the-turn skips end step triggers (723.1e)

**723.1f** — TESTABLE. Triggers that fire DURING the end-the-turn process go on the stack during cleanup, granting priority.

**ATOM-723.1f-001**
- **Rule:** 723.1f — Abilities that trigger during the end-the-turn process are put on the stack during cleanup; active player gets priority
- **Mechanism:** Cleanup-deferred triggers from end-the-turn process
- **Minimal Board:** During step 723.1b, exiling an object triggers "Whenever a card is exiled, draw a card." The end-the-turn process completes.
- **Action:** The cleanup step begins.
- **Expected Result:** The "exiled" trigger is put on the stack during cleanup. Active player gets priority. After all triggers resolve, a new cleanup step begins.
- **Phase:** Phase 8
- **Ticket:** NEW — End-the-turn cleanup triggers and priority (723.1f)

**ATOM-723.1f-002**
- **Rule:** 723.1f — If NO triggers fired during the end-the-turn process, no player gets priority during cleanup
- **Mechanism:** Cleanup priority suppression when no triggers occurred
- **Minimal Board:** An effect ends the turn. No abilities trigger during the end-the-turn process.
- **Action:** The cleanup step begins.
- **Expected Result:** No player gets priority during cleanup. The turn ends immediately after the cleanup step's normal actions (discard to hand size, remove "until end of turn" effects).
- **Phase:** Phase 8
- **Ticket:** NEW — End-the-turn cleanup no priority when no triggers (723.1f)

---

**723.2** — TESTABLE. Defines the "end the combat phase" procedure.

**ATOM-723.2-001**
- **Rule:** 723.2 — When an effect ends the combat phase, a specific multi-step procedure is followed
- **Mechanism:** End-combat-phase procedure entry point
- **Minimal Board:** It is the declare attackers step. A spell says "End the combat phase."
- **Action:** The spell resolves.
- **Expected Result:** The end-combat-phase procedure (723.2a–f) is followed in order.
- **Phase:** Phase 8
- **Ticket:** NEW — End-combat-phase procedure (723.2)

**723.2a** — TESTABLE. Pending triggers cease to exist (parallel to 723.1a).

**ATOM-723.2a-001**
- **Rule:** 723.2a — Triggered abilities that triggered before end-combat-phase but haven't been stacked yet cease to exist
- **Mechanism:** Pending trigger purge during end-combat-phase
- **Minimal Board:** A "whenever a creature attacks" trigger has fired but not yet been put on the stack. An effect ends the combat phase.
- **Action:** End-combat-phase process begins.
- **Expected Result:** The pending "attacks" trigger ceases to exist.
- **Phase:** Phase 8
- **Ticket:** NEW — End-combat-phase pending trigger purge (723.2a)

**723.2b** — TESTABLE. Exile every object on the stack (parallel to 723.1b).

**ATOM-723.2b-001**
- **Rule:** 723.2b — Exile every object on the stack during end-combat-phase
- **Mechanism:** Stack exile during end-combat-phase
- **Minimal Board:** The stack contains the end-combat-phase spell and a combat trick below it.
- **Action:** Step 723.2b executes.
- **Expected Result:** Both the end-combat-phase spell and the combat trick are exiled.
- **Phase:** Phase 8
- **Ticket:** NEW — End-combat-phase stack exile (723.2b)

**723.2c** — TESTABLE. SBAs checked with no priority/triggers (parallel to 723.1c).

**ATOM-723.2c-001**
- **Rule:** 723.2c — SBAs checked during end-combat-phase; no priority, no triggers stacked
- **Mechanism:** SBA check with trigger/priority suppression
- **Minimal Board:** A creature has lethal damage. End-combat-phase reaches step 723.2c.
- **Action:** SBAs are checked.
- **Expected Result:** Creature destroyed. "Dies" triggers NOT stacked. No priority.
- **Phase:** Phase 8
- **Ticket:** NEW — End-combat-phase SBA check with suppression (723.2c)

**723.2d** — TESTABLE. Combat phase ends, creatures removed from combat, "until end of combat" effects expire, skip to next phase.

**ATOM-723.2d-001**
- **Rule:** 723.2d — Combat phase ends; creatures removed from combat; "until end of combat" effects expire; skip to postcombat main phase
- **Mechanism:** Combat phase termination + duration expiry + phase skip
- **Minimal Board:** Two creatures are in combat. One has "gets +3/+3 until end of combat." An effect ends the combat phase.
- **Action:** Step 723.2d executes.
- **Expected Result:** Both creatures removed from combat. The +3/+3 bonus expires. The game proceeds to the postcombat main phase (skipping end of combat step, etc.).
- **Phase:** Phase 8
- **Ticket:** NEW — End-combat-phase termination and duration expiry (723.2d)

**723.2e** — TESTABLE. "At end of combat" triggers don't fire because end of combat step is skipped.

**ATOM-723.2e-001**
- **Rule:** 723.2e — "At end of combat" triggers don't fire because the end of combat step is skipped
- **Mechanism:** End-of-combat trigger suppression
- **Minimal Board:** Player A controls a creature with "At end of combat, sacrifice this creature." An effect ends the combat phase during declare blockers.
- **Action:** End-combat-phase process completes.
- **Expected Result:** The "at end of combat" ability does NOT trigger. The creature is NOT sacrificed by that ability.
- **Phase:** Phase 8
- **Ticket:** NEW — End-combat-phase skips end-of-combat triggers (723.2e)

**723.2f** — TESTABLE. Triggers during the process go on stack in the following phase (parallel to 723.1f).

**ATOM-723.2f-001**
- **Rule:** 723.2f — Abilities that trigger during end-combat-phase are put on the stack during the following phase
- **Mechanism:** Post-combat-phase deferred triggers
- **Minimal Board:** During step 723.2b, exiling an object triggers an ability. End-combat-phase completes.
- **Action:** The postcombat main phase begins.
- **Expected Result:** The triggered ability is put on the stack. Active player gets priority.
- **Phase:** Phase 8
- **Ticket:** NEW — End-combat-phase deferred triggers in following phase (723.2f)

**723.2g** — TESTABLE. Ending combat phase outside combat does nothing.

**ATOM-723.2g-001**
- **Rule:** 723.2g — If an effect attempts to end the combat phase outside of combat, nothing happens
- **Mechanism:** No-op for end-combat-phase outside combat
- **Minimal Board:** It is Player A's main phase (not combat). An effect says "End the combat phase."
- **Action:** The effect resolves.
- **Expected Result:** Nothing happens. The game continues normally in the current phase.
- **Phase:** Phase 8
- **Ticket:** NEW — End-combat-phase no-op outside combat (723.2g)

---

### 724. The Monarch

**724.1** — TESTABLE. Monarch is a player designation; doesn't exist until an effect creates it.

**ATOM-724.1-001**
- **Rule:** 724.1 — The monarch designation does not exist until an effect instructs a player to become the monarch
- **Mechanism:** Monarch designation initialization
- **Minimal Board:** A new game starts. No monarch-granting effects have resolved.
- **Action:** Check if any player is the monarch.
- **Expected Result:** No player is the monarch. The game has no monarch designation.
- **Phase:** Phase 7 (D15 — triggered abilities with no source)
- **Ticket:** D15

**724.2** — TESTABLE. Two inherent triggered abilities: end-step card draw and combat damage monarch transfer. No source, controlled by monarch.

**ATOM-724.2-001**
- **Rule:** 724.2 — "At the beginning of the monarch's end step, that player draws a card"
- **Mechanism:** Source-less triggered ability at end step for monarch
- **Minimal Board:** Player A is the monarch. It is the beginning of Player A's end step.
- **Action:** End step begins.
- **Expected Result:** The monarch's end-step trigger fires. Player A draws a card. The trigger has no source and is controlled by Player A (the monarch at time of trigger).
- **Phase:** Phase 7 (D15)
- **Ticket:** D15

**ATOM-724.2-002**
- **Rule:** 724.2 — "Whenever a creature deals combat damage to the monarch, its controller becomes the monarch"
- **Mechanism:** Source-less triggered ability on combat damage to monarch
- **Minimal Board:** Player A is the monarch. Player B controls a creature that deals combat damage to Player A.
- **Action:** Combat damage is dealt.
- **Expected Result:** The combat-damage trigger fires. Player B becomes the monarch. Player A ceases to be the monarch.
- **Phase:** Phase 7 (D15)
- **Ticket:** D15

**724.3** — TESTABLE. Only one monarch at a time; becoming monarch displaces the current monarch.

**ATOM-724.3-001**
- **Rule:** 724.3 — As a player becomes the monarch, the current monarch ceases to be the monarch
- **Mechanism:** Monarch designation exclusivity
- **Minimal Board:** Player A is the monarch. An effect causes Player B to become the monarch.
- **Action:** The effect resolves.
- **Expected Result:** Player B is now the monarch. Player A is no longer the monarch. At no point are both players simultaneously the monarch.
- **Phase:** Phase 7 (D15)
- **Ticket:** D15

**724.4** — TESTABLE. Monarch leaves the game: active player becomes monarch.

**ATOM-724.4-001**
- **Rule:** 724.4 — If the monarch leaves the game, the active player becomes the monarch
- **Mechanism:** Monarch transfer on player exit
- **Minimal Board:** Three-player game. Player B is the monarch. It is Player A's turn (active player).
- **Action:** Player B loses and leaves the game.
- **Expected Result:** Player A (active player) becomes the monarch simultaneously as Player B leaves.
- **Phase:** Phase 7 (D15) + Phase 9 (multiplayer)
- **Ticket:** D15

**ATOM-724.4-002**
- **Rule:** 724.4 — If the active player is the one leaving and is the monarch, the next player in turn order becomes the monarch
- **Mechanism:** Monarch fallback to next player in turn order
- **Minimal Board:** Three-player game (A, B, C in turn order). Player A is both the active player and the monarch.
- **Action:** Player A loses and leaves the game.
- **Expected Result:** Player B (next in turn order) becomes the monarch.
- **Phase:** Phase 7 (D15) + Phase 9 (multiplayer)
- **Ticket:** D15

**724.5** — TESTABLE. Continuous effect based on monarch does nothing if no monarch exists.

**ATOM-724.5-001**
- **Rule:** 724.5 — A continuous effect that depends on who is the monarch does nothing if there is no monarch
- **Mechanism:** Monarch-dependent continuous effect null-check
- **Minimal Board:** A static ability says "The monarch has hexproof." No player is currently the monarch.
- **Action:** Check if any player has hexproof from this effect.
- **Expected Result:** No player has hexproof. The effect does nothing because there is no monarch.
- **Phase:** Phase 7 (D15) + Phase 5 (continuous effects)
- **Ticket:** D15

---

### 725. The Initiative

**725.1** — TESTABLE. Initiative is a player designation; doesn't exist until an effect creates it.

**ATOM-725.1-001**
- **Rule:** 725.1 — The initiative designation does not exist until an effect instructs a player to take the initiative
- **Mechanism:** Initiative designation initialization
- **Minimal Board:** A new game starts. No initiative-granting effects have resolved.
- **Action:** Check if any player has the initiative.
- **Expected Result:** No player has the initiative.
- **Phase:** Phase 7 (D15)
- **Ticket:** D15

**725.2** — TESTABLE. Three inherent triggered abilities: upkeep venture, combat damage transfer, and take-initiative venture. No source.

**ATOM-725.2-001**
- **Rule:** 725.2 — "At the beginning of the upkeep of the player who has the initiative, that player ventures into Undercity"
- **Mechanism:** Source-less upkeep trigger for initiative holder
- **Minimal Board:** Player A has the initiative. It is the beginning of Player A's upkeep.
- **Action:** Upkeep begins.
- **Expected Result:** The upkeep trigger fires. Player A ventures into Undercity.
- **Phase:** Phase 7 (D15) + Phase 9 (Dungeon/Venture)
- **Ticket:** D15

**ATOM-725.2-002**
- **Rule:** 725.2 — "Whenever one or more creatures a player controls deal combat damage to the player who has the initiative, the controller of those creatures takes the initiative"
- **Mechanism:** Source-less combat damage trigger for initiative transfer
- **Minimal Board:** Player A has the initiative. Player B's creature deals combat damage to Player A.
- **Action:** Combat damage is dealt.
- **Expected Result:** Player B takes the initiative. Player A loses the initiative.
- **Phase:** Phase 7 (D15)
- **Ticket:** D15

**ATOM-725.2-003**
- **Rule:** 725.2 — "Whenever a player takes the initiative, that player ventures into Undercity"
- **Mechanism:** Source-less trigger on taking the initiative
- **Minimal Board:** An effect causes Player A to take the initiative.
- **Action:** The effect resolves.
- **Expected Result:** Player A takes the initiative and the "ventures into Undercity" trigger fires.
- **Phase:** Phase 7 (D15) + Phase 9 (Dungeon/Venture)
- **Ticket:** D15

**725.3** — TESTABLE. Only one player can have the initiative at a time (parallel to 724.3).

**ATOM-725.3-001**
- **Rule:** 725.3 — As a player takes the initiative, the current initiative holder ceases to have it
- **Mechanism:** Initiative designation exclusivity
- **Minimal Board:** Player A has the initiative. An effect causes Player B to take the initiative.
- **Action:** The effect resolves.
- **Expected Result:** Player B has the initiative. Player A no longer has it.
- **Phase:** Phase 7 (D15)
- **Ticket:** D15

**725.4** — TESTABLE. Initiative holder leaves the game: active player takes it (parallel to 724.4).

**ATOM-725.4-001**
- **Rule:** 725.4 — If the player who has the initiative leaves the game, the active player takes the initiative
- **Mechanism:** Initiative transfer on player exit
- **Minimal Board:** Three-player game. Player B has the initiative. It is Player A's turn.
- **Action:** Player B loses and leaves the game.
- **Expected Result:** Player A (active player) takes the initiative simultaneously as Player B leaves. This also triggers "Whenever a player takes the initiative" (725.2).
- **Phase:** Phase 7 (D15) + Phase 9 (multiplayer)
- **Ticket:** D15

**725.5** — TESTABLE. Initiative holder re-taking the initiative still triggers the "takes the initiative" ability.

**ATOM-725.5-001**
- **Rule:** 725.5 — If the player who already has the initiative is instructed to take it again, the "takes the initiative" trigger fires but no second designation is created
- **Mechanism:** Re-take trigger without duplicate designation
- **Minimal Board:** Player A has the initiative. An effect says "Player A takes the initiative."
- **Action:** The effect resolves.
- **Expected Result:** Player A still has the initiative (no second designation). The "Whenever a player takes the initiative, that player ventures into Undercity" trigger DOES fire (Player A ventures again).
- **Phase:** Phase 7 (D15) + Phase 9 (Dungeon/Venture)
- **Ticket:** D15

---

### 726. Restarting the Game — DEFERRED (Post-v1)

Per session guidance: extremely niche (Karn Liberated only). One-liner entries.

- **726.1** — DEFERRED — Post-v1. Restart procedure: game ends, no one wins/loses/draws, new game starts. TESTABLE if implemented.
- **726.1a** — DEFERRED — Post-v1. Starting player in restarted game is the controller of the restart effect. TESTABLE.
- **726.2** — DEFERRED — Post-v1. All cards involved in the ended game are part of the new game. TESTABLE (has Example).
- **726.3** — DEFERRED — Post-v1. Players with <7 cards lose to SBA in the new game. TESTABLE.
- **726.4** — DEFERRED — Post-v1. Restart effect finishes resolving before the first untap step. TESTABLE.
- **726.5** — DEFERRED — Post-v1. Effects may exempt cards from restart. TESTABLE.
- **726.5a** — DEFERRED — Phase 9 (Commander). Commander exemption from restart.
- **726.6** — DEFERRED — Post-v1. Subgame restart doesn't affect main game.
- **726.7** — DEFERRED — Post-v1. Multiplayer limited-range restart involves all players. OUT-OF-SCOPE (limited range of influence).

---

### 727. Rad Counters

**727.1** — TESTABLE. Defines the inherent triggered ability for rad counters: at precombat main, mill N cards (N = rad counters), lose 1 life and remove 1 rad counter per nonland card milled.

**ATOM-727.1-001**
- **Rule:** 727.1 — Rad counter triggered ability: at precombat main, mill cards equal to rad counters, lose life and remove counters for nonland cards milled
- **Mechanism:** Source-less triggered ability for rad counters on players
- **Minimal Board:** Player A has 3 rad counters. Top 3 cards of Player A's library are: Mountain (land), Lightning Bolt (nonland), Forest (land).
- **Action:** Player A's precombat main phase begins.
- **Expected Result:** Player A mills 3 cards. 1 nonland card was milled (Lightning Bolt), so Player A loses 1 life and removes 1 rad counter (3 → 2 rad counters remaining).
- **Phase:** Phase 8 (card breadth — Fallout set mechanics)
- **Ticket:** NEW — Rad counter triggered ability (727.1)

**ATOM-727.1-002**
- **Rule:** 727.1 — Rad counter ability doesn't trigger if player has 0 rad counters
- **Mechanism:** Trigger condition check (one or more rad counters)
- **Minimal Board:** Player A has 0 rad counters.
- **Action:** Player A's precombat main phase begins.
- **Expected Result:** The rad counter ability does NOT trigger (condition "if that player has one or more rad counters" is not met).
- **Phase:** Phase 8
- **Ticket:** NEW — Rad counter zero-counter no-trigger (727.1)

**ATOM-727.1-003**
- **Rule:** 727.1 — All milled cards are nonland: lose life equal to rad counters and remove all rad counters
- **Mechanism:** Full nonland mill scenario
- **Minimal Board:** Player A has 2 rad counters. Top 2 cards are both nonland spells.
- **Action:** Player A's precombat main phase begins and the ability resolves.
- **Expected Result:** Player A mills 2 cards (both nonland). Loses 2 life and removes 2 rad counters (2 → 0).
- **Phase:** Phase 8
- **Ticket:** NEW — Rad counter full nonland mill (727.1)

**727.1a** — BOUNDARY-DEF. Defines "life loss from radiation" as referring to life lost from the rad counter trigger. Referenced by replacement effects (e.g., "You gain life rather than lose life from radiation.").

**ATOM-727.1a-001**
- **Rule:** 727.1a — "Life loss from radiation" is identifiable for replacement effects
- **Mechanism:** Life loss source tagging + replacement effect matching
- **Minimal Board:** Player A has 3 rad counters and controls a permanent with "You gain life rather than lose life from radiation." Top 3 cards: 2 nonland, 1 land.
- **Action:** Precombat main phase begins, rad counter ability triggers and resolves. Player A mills 3, finds 2 nonland.
- **Expected Result:** Instead of losing 2 life, Player A gains 2 life (replacement effect applies specifically to "life loss from radiation"). 2 rad counters are still removed (that part isn't replaced).
- **Phase:** Phase 6 (replacement) + Phase 8
- **Ticket:** NEW — Radiation life loss tagging for replacement effects (727.1a)
- **Dependencies:** Phase 6 (replacement effects), life loss source tracking

--- End of Chunk 2 ---

## Chunk 3: Rules 728–732

### 728. Subgames — DEFERRED (Post-v1)

Per session guidance: extremely niche (Shahrazad banned everywhere). One-liner entries.

- **728.1** — DEFERRED — Post-v1. One card (Shahrazad) creates subgames. PURE-DEF (names the card).
- **728.1a** — DEFERRED — Post-v1. Defines subgame as a separate game within a game. PURE-DEF.
- **728.1b** — DEFERRED — Post-v1. Effects/definitions don't cross subgame boundary. TESTABLE if implemented.
- **728.2** — DEFERRED — Post-v1. New zones created; libraries moved and shuffled; random first player. TESTABLE if implemented.
- **728.2a** — DEFERRED — Post-v1. Supplementary decks moved to subgame command zone. TESTABLE if implemented.
- **728.2b** — DEFERRED — Post-v1. Vanguard card moves to subgame. DEFERRED (Vanguard).
- **728.2c** — DEFERRED — Phase 9 (Commander). Commander moves to subgame command zone.
- **728.3** — DEFERRED — Post-v1. Players with <7 cards in deck lose subgame to SBA. TESTABLE if implemented.
- **728.4** — DEFERRED — Post-v1. Main-game objects are outside the subgame. TESTABLE if implemented.
- **728.4a** — DEFERRED — Post-v1. Cards brought into subgame trigger main-game leave-zone abilities (deferred to main game resume). TESTABLE if implemented.
- **728.4b** — DEFERRED — Post-v1. Player counters don't cross subgame boundary. TESTABLE if implemented.
- **728.5** — DEFERRED — Post-v1. Subgame end: cards return to main-game library, zones cease to exist, main game resumes. TESTABLE if implemented (has Example).
- **728.5a** — DEFERRED — Post-v1. Nontraditional cards returned to supplementary decks.
- **728.5b** — DEFERRED — Post-v1. Vanguard card returns. DEFERRED (Vanguard).
- **728.5c** — DEFERRED — Phase 9 (Commander). Commander returns to main-game command zone.
- **728.6** — DEFERRED — Post-v1. Subgame within a subgame is valid (recursive). TESTABLE if implemented.

---

### 729. Merging with Permanents (Mutate)

All rules in 729 are **DEFERRED — Phase 9** (Mutate system). Full ATOM specs below since each sub-rule defines independently testable behavior.

**729.1** — PURE-DEF. References rule 702.140 "Mutate." Prerequisite for 729.2+.

**729.2** — TESTABLE. Defines the merge action: place object on top of or under a permanent.

**ATOM-729.2-001**
- **Rule:** 729.2 — To merge, place the object on top of or under the target permanent; the permanent becomes a merged permanent represented by all components
- **Mechanism:** Merge action — component stacking and merged permanent creation
- **Minimal Board:** Player A controls Creature X on the battlefield. Player A casts Creature Y with mutate targeting Creature X. Player A chooses "on top."
- **Action:** Creature Y resolves and merges with Creature X.
- **Expected Result:** The permanent is now a merged permanent with Y on top and X underneath. It has Y's characteristics (topmost component per 729.2a).
- **Phase:** Phase 9 (Mutate)
- **Ticket:** NEW — Merge action and component stacking (729.2)

**729.2a** — TESTABLE. Merged permanent has characteristics of topmost component. Copiable effect with timestamp.

**ATOM-729.2a-001**
- **Rule:** 729.2a — A merged permanent has only the characteristics of its topmost component (copiable effect, timestamped at merge time)
- **Mechanism:** Topmost-component characteristic selection
- **Minimal Board:** Merged permanent with Creature Y (3/3 flying) on top and Creature X (2/2 trample) underneath.
- **Action:** Check the merged permanent's characteristics.
- **Expected Result:** The permanent has Y's characteristics: 3/3, flying. It does NOT have X's characteristics (2/2, trample) as base characteristics. The topmost-component rule is a copiable effect.
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent topmost component characteristics (729.2a)

**ATOM-729.2a-002**
- **Rule:** 729.2a — All abilities from ALL components apply to the merged permanent
- **Mechanism:** Ability aggregation from all components (not just top)
- **Minimal Board:** Merged permanent with Creature Y (3/3 flying) on top and Creature X (2/2, "Whenever this creature attacks, draw a card") underneath.
- **Action:** The merged permanent attacks.
- **Expected Result:** The "Whenever this creature attacks, draw a card" ability from the bottom component X triggers. The merged permanent has abilities from ALL components, but base characteristics (name, P/T, mana cost, types) only from the topmost.
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent ability aggregation from all components (729.2a)

**729.2b** — TESTABLE. Merging object leaves its previous zone; resulting permanent is NOT considered to have just entered.

**ATOM-729.2b-001**
- **Rule:** 729.2b — The merging object leaves its previous zone but the resulting permanent is not considered to have just entered the battlefield
- **Mechanism:** Zone transition without ETB
- **Minimal Board:** Creature Y is on the stack (cast with mutate). Creature X is on the battlefield with "Whenever a creature enters the battlefield, gain 1 life."
- **Action:** Creature Y resolves and merges with Creature X.
- **Expected Result:** Creature Y leaves the stack. The merged permanent does NOT trigger "enters the battlefield" abilities (it was already on the battlefield). No life is gained from X's ETB trigger.
- **Phase:** Phase 9
- **Ticket:** NEW — Merge does not trigger ETB (729.2b)

**729.2c** — TESTABLE. Merged permanent retains continuous effects and controller status.

**ATOM-729.2c-001**
- **Rule:** 729.2c — The merged permanent is the same object; continuous effects continue to apply; no "new controller" triggers
- **Mechanism:** Object identity continuity through merge
- **Minimal Board:** Creature X has a +1/+1 counter and is enchanted by an Aura granting +2/+2. Creature Y merges on top.
- **Action:** After merge, check the permanent's state.
- **Expected Result:** The +1/+1 counter and the Aura's +2/+2 still apply to the merged permanent. It hasn't "just come under a player's control."
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent identity continuity (729.2c)

**729.2d** — TESTABLE. Token status determined by topmost component.

**ATOM-729.2d-001**
- **Rule:** 729.2d — If a merged permanent contains a token, it is a token only if the topmost component is a token
- **Mechanism:** Token status from topmost component
- **Minimal Board:** Token creature T is on top, card creature C is underneath in a merged permanent.
- **Action:** Check if the merged permanent is a token.
- **Expected Result:** Yes, it is a token (topmost component T is a token). If the order were reversed (C on top, T underneath), it would NOT be a token.
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent token status (729.2d)

**729.2e** — TESTABLE. Face-up/face-down status determined by topmost component.

**ATOM-729.2e-001**
- **Rule:** 729.2e — A merged permanent's face-up/face-down status is determined by its topmost component
- **Mechanism:** Topmost component determines face status
- **Minimal Board:** A face-down permanent (morph) is on the battlefield. A face-up creature merges on top of it.
- **Action:** Check the merged permanent's status.
- **Expected Result:** The merged permanent is face up (topmost component is face up). This does NOT count as "being turned face up" for trigger purposes.
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent face status from topmost (729.2e)

**729.2f** — TESTABLE. Turning a merged permanent face down/up affects all components.

**ATOM-729.2f-001**
- **Rule:** 729.2f — If a merged permanent is turned face down, all face-up components turn face down; if turned face up, all face-down components turn face up
- **Mechanism:** Bulk face status change for all components
- **Minimal Board:** A merged permanent (face up) has two face-up card components.
- **Action:** An effect turns the merged permanent face down.
- **Expected Result:** Both components are turned face down. The merged permanent is now face down.
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent bulk face-down/up (729.2f)

**729.2g** — TESTABLE. Face-down merged permanent with instant/sorcery card can't turn face up.

**ATOM-729.2g-001**
- **Rule:** 729.2g — A face-down merged permanent containing an instant or sorcery card can't be turned face up
- **Mechanism:** Face-up prevention for merged permanents with instant/sorcery components
- **Minimal Board:** A face-down merged permanent contains a creature card and an instant card as components.
- **Action:** An effect would turn the merged permanent face up.
- **Expected Result:** The permanent stays face down. Its controller reveals it (showing it contains an instant). "Turned face up" triggers do NOT trigger.
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent instant/sorcery face-up lock (729.2g)

**729.2h** — TESTABLE. Flip card component uses alternative characteristics if merged permanent is flipped.

**ATOM-729.2h-001**
- **Rule:** 729.2h — If a merged permanent contains a flip card and the permanent is flipped, that component uses its alternative characteristics
- **Mechanism:** Flip card interaction within merged permanent
- **Minimal Board:** A merged permanent contains a flip card (with alternative characteristics) as a component.
- **Action:** The merged permanent is flipped.
- **Expected Result:** The flip card component uses its alternative characteristics instead of its normal characteristics.
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent flip card component (729.2h)

**729.2i** — TESTABLE. Merged permanent is NOT a DFC even with DFC components; transform affects only DFC components.

**ATOM-729.2i-001**
- **Rule:** 729.2i — A merged permanent is not a double-faced permanent; transforming it turns only its DFC components to their other face
- **Mechanism:** Transform scoping within merged permanent
- **Minimal Board:** A merged permanent has a DFC component (front face up) and a non-DFC card component.
- **Action:** An effect transforms the merged permanent.
- **Expected Result:** Only the DFC component flips to its other face. The non-DFC component is unaffected. The merged permanent is NOT considered a double-faced permanent.
- **Phase:** Phase 9 (D14 — DFC system)
- **Ticket:** D14

**729.2j** — TESTABLE. Face-up merged permanent with a DFC component can't be turned face down.

**ATOM-729.2j-001**
- **Rule:** 729.2j — A face-up merged permanent containing a double-faced component can't be turned face down
- **Mechanism:** Face-down prevention for merged permanents with DFC components
- **Minimal Board:** A face-up merged permanent contains a DFC card.
- **Action:** An effect would turn the permanent face down.
- **Expected Result:** The permanent can't be turned face down. It remains face up.
- **Phase:** Phase 9 (D14)
- **Ticket:** D14

**729.3** — TESTABLE. When a merged permanent leaves the battlefield, components are separated to the appropriate zone.

**ATOM-729.3-001**
- **Rule:** 729.3 — When a merged permanent leaves the battlefield, one permanent leaves and each component goes to the appropriate zone
- **Mechanism:** Component separation on zone change
- **Minimal Board:** A merged permanent (two card components + one token component) is on the battlefield.
- **Action:** The merged permanent is destroyed.
- **Expected Result:** One permanent leaves the battlefield. The two card components go to their owners' graveyards. The token component ceases to exist (SBA).
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent component separation on leave (729.3)

**729.3a** — TESTABLE. Player may arrange component order when going to graveyard or library.

**ATOM-729.3a-001**
- **Rule:** 729.3a — If a merged permanent goes to graveyard or library, the owner may arrange the components in any order
- **Mechanism:** Component ordering choice on zone change
- **Minimal Board:** A merged permanent (two cards owned by the same player) is destroyed.
- **Action:** Components are put into the graveyard.
- **Expected Result:** The owner chooses the order of the two cards in their graveyard. If going to library, the order is not revealed.
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent component ordering (729.3a)

**729.3b** — TESTABLE. Exiling a merged permanent: controller determines timestamp order.

**ATOM-729.3b-001**
- **Rule:** 729.3b — When exiling a merged permanent, the exiling player determines the relative timestamp order of the cards
- **Mechanism:** Timestamp ordering for exiled merged components
- **Minimal Board:** A merged permanent (two card components) is exiled by an effect.
- **Action:** The components are exiled.
- **Expected Result:** The player who exiled it determines the relative timestamp order of the exiled cards.
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent exile timestamp ordering (729.3b)

**729.3c** — TESTABLE. Effects that track "the new object" find ALL components.

**ATOM-729.3c-001**
- **Rule:** 729.3c — An effect that finds the new object a merged permanent becomes finds all components
- **Mechanism:** Multi-object tracking for merged permanent zone change
- **Minimal Board:** A merged permanent (cards A and B) is exiled by an effect that says "exile it, then return it to the battlefield."
- **Action:** The merged permanent is exiled.
- **Expected Result:** Both cards A and B are exiled. The "return it" effect applies to BOTH cards — both are returned to the battlefield (as separate permanents).
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent multi-object tracking (729.3c)

**729.3d** — TESTABLE. Replacement effects on a merged permanent leaving apply to all components.

**ATOM-729.3d-001**
- **Rule:** 729.3d — Applying a replacement effect to a merged permanent leaving the battlefield applies to all components
- **Mechanism:** Replacement effect bulk application to merged components
- **Minimal Board:** A merged permanent would be put into the graveyard. A replacement effect says "If this would be put into a graveyard, exile it instead."
- **Action:** The merged permanent is destroyed.
- **Expected Result:** ALL components are exiled instead of going to the graveyard (the replacement effect applies to the whole merged permanent).
- **Phase:** Phase 9 + Phase 6 (replacement effects)
- **Ticket:** NEW — Merged permanent replacement effect on all components (729.3d)

**729.3e** — TESTABLE. Replacement effects on "cards" (not tokens) interact with mixed token/card merged permanents.

**ATOM-729.3e-001**
- **Rule:** 729.3e — A replacement effect on "a card" being put into a zone applies to all components of a non-token merged permanent, including token components
- **Mechanism:** Card-vs-token replacement effect scoping for merged permanents
- **Minimal Board:** A merged permanent (topmost: card, so it's not a token per 729.2d) contains a card component and a token component. A replacement effect says "If a card would be put into a graveyard, exile it instead."
- **Action:** The merged permanent is destroyed.
- **Expected Result:** All components (including the token) are exiled because the merged permanent as a whole is not a token. The replacement effect applies to the whole object.
- **Phase:** Phase 9 + Phase 6 (replacement effects)
- **Ticket:** NEW — Merged permanent card replacement includes token components (729.3e)

---

### 730. Day and Night

**730.1** — TESTABLE. Day/night is a game-level designation. Starts as neither. Once set, always has exactly one.

**ATOM-730.1-001**
- **Rule:** 730.1 — The game starts with neither day nor night; once set, it always has exactly one designation
- **Mechanism:** Game-level day/night designation tracking
- **Minimal Board:** Game start. No daybound/nightbound cards played.
- **Action:** Check the game's day/night designation.
- **Expected Result:** The game has neither day nor night designation.
- **Phase:** Phase 9 (D14 — DFC system, day/night)
- **Ticket:** D14

**ATOM-730.1-002**
- **Rule:** 730.1 — Once it becomes day or night, the game always has exactly one of those designations
- **Mechanism:** Day/night permanence after initialization
- **Minimal Board:** An effect causes it to become day.
- **Action:** Check if the designation can be removed.
- **Expected Result:** The game now has the "day" designation. It cannot return to "neither" — from this point forward, it is always either day or night.
- **Phase:** Phase 9 (D14)
- **Ticket:** D14

**730.1a** — TESTABLE. Defines "day becomes night" / "night becomes day" as losing one designation and gaining the other. Multiple cards trigger on "when day becomes night or night becomes day."

**ATOM-730.1a-001**
- **Rule:** 730.1a — "When day becomes night" triggers fire on day→night transition
- **Mechanism:** Game designation change event → triggered ability matching
- **Minimal Board:** It is day. Player A controls a permanent with "When day becomes night, create a 1/1 Wolf token." Previous turn's active player cast 0 spells.
- **Action:** Untap step day/night check transitions from day to night.
- **Expected Result:** The trigger fires. Player A creates a 1/1 Wolf token.
- **Phase:** Phase 7 (triggered) + Phase 9 (D14)
- **Ticket:** D14 + NEW — Day/night transition trigger event (730.1a)

**ATOM-730.1a-002**
- **Rule:** 730.1a — "When night becomes day" triggers fire on night→day transition
- **Mechanism:** Game designation change event → triggered ability matching
- **Minimal Board:** It is night. Player A controls a permanent with "When night becomes day, draw a card." Previous turn's active player cast 2 spells.
- **Action:** Untap step day/night check transitions from night to day.
- **Expected Result:** The trigger fires. Player A draws a card.
- **Phase:** Phase 7 (triggered) + Phase 9 (D14)
- **Ticket:** D14

**730.2** — TESTABLE. Untap step check for day/night transition.

**ATOM-730.2-001**
- **Rule:** 730.2 — As part of the untap step, the game checks the previous turn to determine if day/night should change
- **Mechanism:** Untap step day/night transition check
- **Minimal Board:** It is currently day. The previous turn's active player cast 0 spells during that turn.
- **Action:** The untap step of the current turn executes its day/night check.
- **Expected Result:** It becomes night (day → night because previous turn's active player cast 0 spells).
- **Phase:** Phase 9 (D14)
- **Ticket:** D14

**730.2a** — TESTABLE. Day → night transition: previous turn's active player cast no spells.

**ATOM-730.2a-001**
- **Rule:** 730.2a — If it's day and the previous turn's active player didn't cast any spells, it becomes night
- **Mechanism:** Day-to-night transition condition
- **Minimal Board:** It is day. Previous turn's active player cast 0 spells.
- **Action:** Day/night check occurs during untap step.
- **Expected Result:** It becomes night.
- **Phase:** Phase 9 (D14)
- **Ticket:** D14

**ATOM-730.2a-002**
- **Rule:** 730.2a — If it's day and the previous turn's active player DID cast a spell, it stays day
- **Mechanism:** Day stays day when spells were cast
- **Minimal Board:** It is day. Previous turn's active player cast 1 spell.
- **Action:** Day/night check occurs.
- **Expected Result:** It remains day (only transitions to night if 0 spells were cast).
- **Phase:** Phase 9 (D14)
- **Ticket:** D14

**730.2b** — TESTABLE. Night → day transition: previous turn's active player cast 2+ spells.

**ATOM-730.2b-001**
- **Rule:** 730.2b — If it's night and the previous turn's active player cast 2+ spells, it becomes day
- **Mechanism:** Night-to-day transition condition
- **Minimal Board:** It is night. Previous turn's active player cast 2 spells.
- **Action:** Day/night check occurs during untap step.
- **Expected Result:** It becomes day.
- **Phase:** Phase 9 (D14)
- **Ticket:** D14

**ATOM-730.2b-002**
- **Rule:** 730.2b — If it's night and the previous turn's active player cast only 1 spell, it stays night
- **Mechanism:** Night stays night when fewer than 2 spells were cast
- **Minimal Board:** It is night. Previous turn's active player cast 1 spell.
- **Action:** Day/night check occurs.
- **Expected Result:** It remains night (needs 2+ spells to transition to day).
- **Phase:** Phase 9 (D14)
- **Ticket:** D14

**730.2c** — TESTABLE. If neither day nor night, the check doesn't happen.

**ATOM-730.2c-001**
- **Rule:** 730.2c — If it's neither day nor night, the day/night check doesn't happen; it remains neither
- **Mechanism:** Day/night check skip when neither designation exists
- **Minimal Board:** The game has neither day nor night designation. Previous turn's active player cast 0 spells.
- **Action:** Untap step day/night check would occur.
- **Expected Result:** No transition happens. The game remains without a day/night designation (does NOT become night just because 0 spells were cast — the check is skipped entirely).
- **Phase:** Phase 9 (D14)
- **Ticket:** D14

---

### 731. Taking Shortcuts

**731.1** — PURE-DEF. Players use mutually understood shortcuts. No independent engine mechanical consequence — this describes human play patterns.

**731.1a** — PURE-DEF. Shortcut rules are largely informal. No engine consequence.

**731.1b** — TESTABLE. Loop detection: when actions could repeat indefinitely, shortcut rules determine iteration count.

**ATOM-731.1b-001**
- **Rule:** 731.1b — When a set of actions creates a loop, the shortcut rules determine how many times it repeats
- **Mechanism:** Loop detection and iteration shortcutting
- **Minimal Board:** Player A controls a creature with "{T}: Create a 1/1 token" and an enchantment with "Whenever a creature enters, untap all creatures." Player A has priority.
- **Action:** Player A proposes to repeat the sequence 1,000,000 times.
- **Expected Result:** The engine detects the loop pattern and allows the shortcut, creating 1,000,000 tokens without executing each iteration individually.
- **Phase:** Phase 7 (D26 — GameNumber stub) / Phase 9 (full GameNumber)
- **Ticket:** D26

**731.1c** — OUT-OF-SCOPE. Tournament rules override. Not relevant to engine implementation.

**731.2** — PURE-DEF. Describes the shortcut proposal procedure. The engine equivalent is DecisionProvider choosing to pass priority through a sequence.

**731.2a** — TESTABLE. Shortcut proposal: player describes a legal sequence that may include loops.

**ATOM-731.2a-001**
- **Rule:** 731.2a — A shortcut can describe a loop that repeats N times; it can't include conditional actions
- **Mechanism:** Shortcut proposal validation — no conditional branches allowed
- **Minimal Board:** Player A proposes a shortcut: "I'll activate this ability 100 times."
- **Action:** The engine validates the shortcut proposal.
- **Expected Result:** The shortcut is accepted (it's a fixed-count loop with no conditionals). A conditional shortcut like "I'll keep doing this until I draw a land" would be rejected.
- **Phase:** Phase 9 (full GameNumber / shortcut system)
- **Ticket:** D26

**731.2b** — TESTABLE. Other players can shorten the proposed sequence.

**ATOM-731.2b-001**
- **Rule:** 731.2b — Each other player may accept or shorten the shortcut by naming a point where they'll make a different choice
- **Mechanism:** Shortcut interruption / shortening
- **Minimal Board:** Player A proposes "pass until end of turn." Player B wants to cast a spell during the beginning of combat step.
- **Action:** Player B shortens the shortcut to end at beginning of combat.
- **Expected Result:** The game advances to the beginning of combat step. Player B gets priority and must make a different choice than "pass."
- **Phase:** Phase 9
- **Ticket:** D26

**731.2c** — TESTABLE. Once all players accept/shorten, the shortcut is taken.

**ATOM-731.2c-001**
- **Rule:** 731.2c — Once the last player accepts or shortens, the game advances to the final endpoint
- **Mechanism:** Shortcut execution
- **Minimal Board:** Player A proposes a shortcut. All players accept.
- **Action:** The shortcut is taken.
- **Expected Result:** The game advances to the ending point of the shortcut with all proposed choices having been made.
- **Phase:** Phase 9
- **Ticket:** D26

**731.3** — TESTABLE. Fragmented loops: active player (or first involved player) must break the loop. Has Example.

**ATOM-731.3-001**
- **Rule:** 731.3 — In a fragmented loop, the active player (or first involved player in turn order) must make a different choice to break the loop
- **Mechanism:** Fragmented loop detection and mandatory break by active player
- **Minimal Board:** Active player has "{0}: Target creature gains flying." Nonactive player has "{0}: Target creature loses flying." They alternate activating, returning to the same game state.
- **Action:** The game detects the fragmented loop.
- **Expected Result:** The active player must make a different choice (stop activating the flying ability). The nonactive player has the final say — the creature ends up without flying (per the Example).
- **Phase:** Phase 7 (D26) / Phase 9
- **Ticket:** D26

**731.4** — TESTABLE. Mandatory-only loop = game is a draw.

**ATOM-731.4-001**
- **Rule:** 731.4 — If a loop contains only mandatory actions, the game is a draw
- **Mechanism:** Mandatory loop draw detection
- **Minimal Board:** Three permanents create a loop of mandatory triggered abilities that cycle endlessly with no optional actions available.
- **Action:** The engine detects the loop consists entirely of mandatory actions.
- **Expected Result:** The game ends in a draw (per rules 104.4b and 104.4f).
- **Phase:** Phase 7 (D26) / Phase 9
- **Ticket:** D26
- **Tags:** META-104.4b, META-104.4f

**731.5** — TESTABLE. No player can be forced to break a loop with actions not called for by loop objects. Has Example.

**ATOM-731.5-001**
- **Rule:** 731.5 — No player can be forced to perform an action that would end a loop unless that action is called for by objects involved in the loop
- **Mechanism:** Loop-break action constraint
- **Minimal Board:** A mandatory loop involving an artifact exists. Player A controls Seal of Cleansing ("Sacrifice ~: Destroy target artifact or enchantment").
- **Action:** The engine evaluates if Player A must sacrifice Seal of Cleansing to end the loop.
- **Expected Result:** Player A is NOT forced to sacrifice Seal of Cleansing. The Seal is not involved in the loop. Only actions called for by the loop's objects can be forced.
- **Phase:** Phase 7 (D26) / Phase 9
- **Ticket:** D26

**731.6** — TESTABLE. "[A] unless [B]" loop: no player forced to perform [B].

**ATOM-731.6-001**
- **Rule:** 731.6 — In a loop with "[A] unless [B]," no player can be forced to perform [B]; if no one chooses [B], [A] becomes mandatory
- **Mechanism:** Unless-clause loop handling
- **Minimal Board:** A loop contains an effect "Each player loses 1 life unless they pay {1}." No player chooses to pay {1}.
- **Action:** The engine evaluates the loop.
- **Expected Result:** No player is forced to pay {1} to break the loop. Since no one pays, "each player loses 1 life" is treated as mandatory. If this creates an all-mandatory loop, the game is a draw.
- **Phase:** Phase 7 (D26) / Phase 9
- **Ticket:** D26

---

### 732. Handling Illegal Actions — ALREADY-HANDLED-BY-DESIGN

The engine prevents illegal actions via validation before state mutation (`choose_priority_action` → `validate_action` → `execute_action`). The general-purpose rollback mechanism described in 732.1 is not needed because:
1. Casting spells: 601.2 pipeline validates before committing (T18 covers casting rollback for partial failures)
2. Activating abilities: validation before execution
3. Declaring attackers/blockers: `validate_attackers`/`validate_blockers` before committing
4. All DP choices are validated against legal options

Cards that can create nondeterministic illegal actions (e.g., Selvala, Explorer Returned) are explicitly excluded from scope.

**732.1** — ALREADY-HANDLED-BY-DESIGN. Engine prevents illegal actions; T18 covers casting rollback edge case.

**732.2** — ALREADY-HANDLED-BY-DESIGN. Priority retention after failed validation is natural — the DP is simply re-queried.

--- End of Chunk 3 ---

## Chunk 4: Classification Summary, Composition Tests, Gap Report

### Classification Summary Table

| Rule | Classification | Notes |
|------|---------------|-------|
| 713.1 | OUT-OF-SCOPE | Physical play aid — substitute cards |
| 713.2 | OUT-OF-SCOPE | Substitute card markings |
| 713.2a | OUT-OF-SCOPE | Substitute card style (2011–2018) |
| 713.2b | OUT-OF-SCOPE | Substitute card style (Core 2019) |
| 713.2c | OUT-OF-SCOPE | Substitute card style (ZNR) |
| 713.3 | OUT-OF-SCOPE | Substitute card deck rules |
| 713.4 | OUT-OF-SCOPE | Substitute card identity |
| 713.5 | OUT-OF-SCOPE | Substitute card public zone handling |
| 714.1 | PURE-DEF | Card frame layout |
| 714.1a | PURE-DEF | Saga creature printing layout |
| 714.2 | PURE-DEF | Chapter symbol definition |
| 714.2a | PURE-DEF | Roman numeral notation |
| 714.2b | TESTABLE | Chapter trigger condition — 3 ATOMs |
| 714.2c | TESTABLE | Multi-numeral chapter symbol — 1 ATOM |
| 714.2d | TESTABLE | Final chapter number computation — 1 ATOM |
| 714.2e | PURE-DEF | Final chapter ability definition |
| 714.3 | PURE-DEF | Lore counter usage |
| 714.3a | TESTABLE | ETB lore counter (with/without read ahead) — 2 ATOMs |
| 714.3b | TESTABLE | Precombat main phase lore counter TBA — 1 ATOM |
| 714.4 | TESTABLE | Saga sacrifice SBA — 2 ATOMs |
| 715.1 | PURE-DEF | Card frame layout |
| 715.2 | TESTABLE | Adventure zone-dependent characteristics — 1 ATOM |
| 715.2a | TESTABLE | "Has an Adventure" flag query — 1 ATOM |
| 715.2b | TESTABLE | Adventure copiable values — 1 ATOM |
| 715.2c | PURE-DEF | Adventurer card is one card |
| 715.3 | TESTABLE | Adventure cast mode choice — 1 ATOM |
| 715.3a | TESTABLE | Adventure cast legality — 1 ATOM |
| 715.3b | TESTABLE | Adventure on-stack characteristics — 1 ATOM |
| 715.3c | TESTABLE | Adventure spell copy — 1 ATOM |
| 715.3d | TESTABLE | Adventure resolution to exile — 1 ATOM |
| 715.4 | TESTABLE | Adventure normal characteristics in non-stack zones — 1 ATOM |
| 715.5 | TESTABLE | Adventure alternative name choice — 1 ATOM |
| 716.1 | PURE-DEF | Card frame layout |
| 716.2 | TESTABLE | Class level bar activation + static — 1 ATOM |
| 716.2a | TESTABLE | Class level activation restriction — 1 ATOM |
| 716.2b | TESTABLE | Class level designation persistence/non-copiability — 1 ATOM |
| 716.2c | BOUNDARY-DEF | "Gain a Class level" definition — referenced by mana spending restrictions — 1 ATOM |
| 716.2d | TESTABLE | Default level = 1 — 1 ATOM |
| 716.3 | TESTABLE | Class top-section abilities always active — 1 ATOM |
| 716.4 | PURE-DEF | Class vs leveler card distinction |
| 717.1 | OUT-OF-SCOPE | Attraction card definition (Un-set) |
| 717.2 | OUT-OF-SCOPE | Attraction deck rules |
| 717.2a | OUT-OF-SCOPE | Attraction constructed rules |
| 717.2b | OUT-OF-SCOPE | Attraction limited rules |
| 717.3 | OUT-OF-SCOPE | Attraction ETB from command zone |
| 717.4 | OUT-OF-SCOPE | Attraction roll-to-visit TBA |
| 717.5 | OUT-OF-SCOPE | Attraction visit ability |
| 717.6 | OUT-OF-SCOPE | Attraction zone replacement |
| 717.6a | OUT-OF-SCOPE | Attraction junkyard pile |
| 718.1 | PURE-DEF | Prototype card frame layout |
| 718.2 | TESTABLE | Prototype alternative characteristics — 1 ATOM |
| 718.2a | TESTABLE | Prototype copiable values — 1 ATOM |
| 718.3 | TESTABLE | Prototype cast mode choice — 1 ATOM |
| 718.3a | TESTABLE | Prototype cast legality — 1 ATOM |
| 718.3b | TESTABLE | Prototype color derivation — 1 ATOM |
| 718.3c | TESTABLE | Prototype spell copy — 1 ATOM |
| 718.3d | TESTABLE | Prototype permanent copy — 1 ATOM |
| 718.4 | TESTABLE | Prototype zone-dependent characteristics — 2 ATOMs |
| 718.5 | TESTABLE | Prototype non-overridden characteristics — 1 ATOM |
| 719.1 | PURE-DEF | Case card frame layout |
| 719.2 | PURE-DEF | Case frame no rules meaning |
| 719.3 | PURE-DEF | Case keyword abilities |
| 719.3a | TESTABLE | Case "To solve" triggered ability — 2 ATOMs |
| 719.3b | TESTABLE | Solved designation persistence/non-copiability — 1 ATOM |
| 719.3c | TESTABLE | Solved ability conditional activation — 1 ATOM |
| 720.1 | PURE-DEF | Omen card frame layout |
| 720.2 | TESTABLE | Omen zone-dependent characteristics — 1 ATOM |
| 720.2a | TESTABLE | "Has an Omen" flag query — 1 ATOM |
| 720.2b | TESTABLE | Omen copiable values — 1 ATOM |
| 720.2c | PURE-DEF | Omen card is one card |
| 720.3 | TESTABLE | Omen cast mode choice — 1 ATOM |
| 720.3a | TESTABLE | Omen cast legality — 1 ATOM |
| 720.3b | TESTABLE | Omen on-stack characteristics — 1 ATOM |
| 720.3c | TESTABLE | Omen spell copy — 1 ATOM |
| 720.3d | TESTABLE | Omen resolution to library shuffle — 1 ATOM |
| 720.4 | TESTABLE | Omen normal characteristics in non-stack zones — 1 ATOM |
| 720.5 | TESTABLE | Omen alternative name choice — 1 ATOM |
| 721.1 | PURE-DEF | Station card frame layout |
| 721.2 | PURE-DEF | Station symbol definition |
| 721.2a | TESTABLE | Station counter-threshold ability — 1 ATOM |
| 721.2b | TESTABLE | Station counter-threshold creature transformation — 1 ATOM |
| 721.2c | TESTABLE | Station no P/T in non-battlefield zones — 1 ATOM |
| 721.3 | PURE-DEF | Text box striations no significance |
| 721.4 | TESTABLE | Station base abilities always active — 1 ATOM |
| 722.1 | TESTABLE | Player control duration — 1 ATOM |
| 722.1a | TESTABLE | Player control overwrite — 1 ATOM |
| 722.1b | TESTABLE | Player control persists through skipped turns — 1 ATOM |
| 722.2 | PURE-DEF | Names two limited-duration control cards |
| 722.3 | TESTABLE | Player control vs object control — 1 ATOM |
| 722.4 | TESTABLE | Player control information visibility — 1 ATOM |
| 722.5 | TESTABLE | Player control decision routing — 1 ATOM |
| 722.5a | TESTABLE | Player control resource isolation — 1 ATOM |
| 722.5b | PURE-DEF | Non-game choices (tournament only) |
| 722.6 | TESTABLE | Player control cannot force concession — 1 ATOM |
| 722.7 | TESTABLE | Player control action restrictions — 1 ATOM |
| 722.8 | PURE-DEF | Controller makes own decisions too |
| 722.9 | TESTABLE | Player self-control no-op — 1 ATOM |
| 723.1 | TESTABLE | End-the-turn procedure — 1 ATOM |
| 723.1a | TESTABLE | End-the-turn pending trigger purge — 1 ATOM |
| 723.1b | TESTABLE | End-the-turn stack exile — 1 ATOM |
| 723.1c | TESTABLE | End-the-turn SBA with suppression — 1 ATOM |
| 723.1d | TESTABLE | End-the-turn phase skip to cleanup — 2 ATOMs |
| 723.1e | TESTABLE | End-the-turn end-step trigger suppression — 1 ATOM |
| 723.1f | TESTABLE | End-the-turn cleanup triggers — 2 ATOMs |
| 723.2 | TESTABLE | End-combat-phase procedure — 1 ATOM |
| 723.2a | TESTABLE | End-combat-phase pending trigger purge — 1 ATOM |
| 723.2b | TESTABLE | End-combat-phase stack exile — 1 ATOM |
| 723.2c | TESTABLE | End-combat-phase SBA with suppression — 1 ATOM |
| 723.2d | TESTABLE | End-combat-phase termination + duration expiry — 1 ATOM |
| 723.2e | TESTABLE | End-combat-phase end-of-combat trigger suppression — 1 ATOM |
| 723.2f | TESTABLE | End-combat-phase deferred triggers — 1 ATOM |
| 723.2g | TESTABLE | End-combat-phase no-op outside combat — 1 ATOM |
| 724.1 | TESTABLE | Monarch designation initialization — 1 ATOM |
| 724.2 | TESTABLE | Monarch inherent triggered abilities — 2 ATOMs |
| 724.3 | TESTABLE | Monarch exclusivity — 1 ATOM |
| 724.4 | TESTABLE | Monarch transfer on player exit — 2 ATOMs |
| 724.5 | TESTABLE | Monarch-dependent effect null-check — 1 ATOM |
| 725.1 | TESTABLE | Initiative designation initialization — 1 ATOM |
| 725.2 | TESTABLE | Initiative inherent triggered abilities — 3 ATOMs |
| 725.3 | TESTABLE | Initiative exclusivity — 1 ATOM |
| 725.4 | TESTABLE | Initiative transfer on player exit — 1 ATOM |
| 725.5 | TESTABLE | Initiative re-take trigger — 1 ATOM |
| 726.1 | DEFERRED — Post-v1 | Restart procedure |
| 726.1a | DEFERRED — Post-v1 | Restart starting player |
| 726.2 | DEFERRED — Post-v1 | Restart card inclusion |
| 726.3 | DEFERRED — Post-v1 | Restart <7 card loss |
| 726.4 | DEFERRED — Post-v1 | Restart resolves before untap |
| 726.5 | DEFERRED — Post-v1 | Restart card exemption |
| 726.5a | DEFERRED — Phase 9 | Commander restart exemption |
| 726.6 | DEFERRED — Post-v1 | Subgame restart isolation |
| 726.7 | OUT-OF-SCOPE | Limited range of influence restart |
| 727.1 | TESTABLE | Rad counter triggered ability — 3 ATOMs |
| 727.1a | BOUNDARY-DEF | "Life loss from radiation" definition — referenced by replacement effects — 1 ATOM |
| 728.1 | DEFERRED — Post-v1 | Subgame card (Shahrazad) |
| 728.1a | DEFERRED — Post-v1 | Subgame definition |
| 728.1b | DEFERRED — Post-v1 | Subgame effect isolation |
| 728.2 | DEFERRED — Post-v1 | Subgame zone creation |
| 728.2a | DEFERRED — Post-v1 | Supplementary decks in subgame |
| 728.2b | DEFERRED — Post-v1 | Vanguard in subgame |
| 728.2c | DEFERRED — Phase 9 | Commander in subgame |
| 728.3 | DEFERRED — Post-v1 | Subgame <7 card loss |
| 728.4 | DEFERRED — Post-v1 | Main-game objects outside subgame |
| 728.4a | DEFERRED — Post-v1 | Cards brought into subgame |
| 728.4b | DEFERRED — Post-v1 | Player counters don't cross |
| 728.5 | DEFERRED — Post-v1 | Subgame end procedure |
| 728.5a | DEFERRED — Post-v1 | Nontraditional cards return |
| 728.5b | DEFERRED — Post-v1 | Vanguard return |
| 728.5c | DEFERRED — Phase 9 | Commander return |
| 728.6 | DEFERRED — Post-v1 | Nested subgames |
| 729.1 | PURE-DEF | Mutate keyword reference |
| 729.2 | TESTABLE (DEFERRED Phase 9) | Merge action — 1 ATOM |
| 729.2a | TESTABLE (DEFERRED Phase 9) | Topmost component characteristics — 2 ATOMs |
| 729.2b | TESTABLE (DEFERRED Phase 9) | Merge zone transition, no ETB — 1 ATOM |
| 729.2c | TESTABLE (DEFERRED Phase 9) | Merged permanent identity continuity — 1 ATOM |
| 729.2d | TESTABLE (DEFERRED Phase 9) | Merged permanent token status — 1 ATOM |
| 729.2e | TESTABLE (DEFERRED Phase 9) | Merged permanent face status — 1 ATOM |
| 729.2f | TESTABLE (DEFERRED Phase 9) | Bulk face-down/up — 1 ATOM |
| 729.2g | TESTABLE (DEFERRED Phase 9) | Instant/sorcery face-up lock — 1 ATOM |
| 729.2h | TESTABLE (DEFERRED Phase 9) | Flip card in merged permanent — 1 ATOM |
| 729.2i | TESTABLE (DEFERRED Phase 9) | DFC in merged permanent — 1 ATOM |
| 729.2j | TESTABLE (DEFERRED Phase 9) | DFC face-down prevention — 1 ATOM |
| 729.3 | TESTABLE (DEFERRED Phase 9) | Component separation on leave — 1 ATOM |
| 729.3a | TESTABLE (DEFERRED Phase 9) | Component ordering on zone change — 1 ATOM |
| 729.3b | TESTABLE (DEFERRED Phase 9) | Exile timestamp ordering — 1 ATOM |
| 729.3c | TESTABLE (DEFERRED Phase 9) | Multi-object tracking — 1 ATOM |
| 729.3d | TESTABLE (DEFERRED Phase 9) | Replacement effect on all components — 1 ATOM |
| 729.3e | TESTABLE (DEFERRED Phase 9) | Card replacement includes token components — 1 ATOM |
| 730.1 | TESTABLE | Day/night designation tracking — 2 ATOMs |
| 730.1a | TESTABLE | Day/night transition triggers — 2 ATOMs |
| 730.2 | TESTABLE | Untap step day/night check — 1 ATOM |
| 730.2a | TESTABLE | Day → night transition — 2 ATOMs |
| 730.2b | TESTABLE | Night → day transition — 2 ATOMs |
| 730.2c | TESTABLE | Neither day nor night check skip — 1 ATOM |
| 731.1 | PURE-DEF | Shortcut informal rules |
| 731.1a | PURE-DEF | Shortcut informality |
| 731.1b | TESTABLE | Loop detection — 1 ATOM |
| 731.1c | OUT-OF-SCOPE | Tournament rules override |
| 731.2 | PURE-DEF | Shortcut proposal procedure |
| 731.2a | TESTABLE | Shortcut proposal validation — 1 ATOM |
| 731.2b | TESTABLE | Shortcut shortening — 1 ATOM |
| 731.2c | TESTABLE | Shortcut execution — 1 ATOM |
| 731.3 | TESTABLE | Fragmented loop detection — 1 ATOM |
| 731.4 | TESTABLE | Mandatory loop draw — 1 ATOM |
| 731.5 | TESTABLE | Loop-break action constraint — 1 ATOM |
| 731.6 | TESTABLE | Unless-clause loop — 1 ATOM |
| 732.1 | ALREADY-HANDLED-BY-DESIGN | Engine prevents illegal actions; T18 covers casting rollback |
| 732.2 | ALREADY-HANDLED-BY-DESIGN | Priority retention natural after failed validation |

**Totals:**
- **TESTABLE:** 96 ATOM tests across 79 TESTABLE sub-rules (net: +3 new ATOMs from 730.1a/727.1a/716.2c, -4 removed from 732.1/732.2 = 95 ATOMs across 78 TESTABLE sub-rules)
- **BOUNDARY-DEF:** 2 sub-rules (716.2c, 727.1a)
- **PURE-DEF:** 28 sub-rules
- **OUT-OF-SCOPE:** 19 sub-rules (713.x, 717.x, 726.7, 731.1c)
- **DEFERRED (one-liner):** 24 sub-rules (726.x except 726.7, 728.x)
- **DEFERRED (with full ATOM spec):** 16 sub-rules (729.x)
- **ALREADY-HANDLED-BY-DESIGN:** 2 sub-rules (732.1, 732.2)

---

### Composition Tests

**COMP-SAGA-LIFECYCLE-001**
- **Rule:** 714.3a + 714.2b + 714.4 — Full Saga lifecycle: ETB with lore counter → chapter trigger → lore counter each turn → sacrifice SBA
- **Composes:** ATOM-714.3a-001, ATOM-714.2b-001, ATOM-714.3b-001, ATOM-714.4-001
- **Mechanism:** Complete Saga progression from cast to sacrifice
- **Minimal Board:** Player A casts a Saga with chapters I, II, III. It resolves.
- **Action:** Play through three turns (ETB + 2 precombat main phases).
- **Expected Result:** Turn 1: ETB with 1 lore counter, chapter I triggers. Turn 2: precombat main adds counter (→2), chapter II triggers. Turn 3: precombat main adds counter (→3), chapter III triggers. After chapter III resolves and leaves the stack, SBA sacrifices the Saga.
- **Phase:** Phase 7 + Phase 8
- **Ticket:** NEW — Saga full lifecycle composition test

**COMP-ADVENTURE-LIFECYCLE-001**
- **Rule:** 715.3 + 715.3a + 715.3b + 715.3d + 715.4 — Full Adventure lifecycle: cast as Adventure → alternative characteristics on stack → resolve to exile → re-cast as creature from exile
- **Composes:** ATOM-715.3-001, ATOM-715.3a-001, ATOM-715.3b-001, ATOM-715.3d-001, ATOM-715.4-001
- **Mechanism:** Complete Adventure card lifecycle
- **Minimal Board:** Player A has an adventurer card in hand.
- **Action:** Cast as Adventure → resolves → exiled → later cast as creature from exile.
- **Expected Result:** On stack: alternative characteristics. Resolves: goes to exile (not graveyard). From exile: can be played as creature (normal characteristics), NOT as Adventure again (via this permission).
- **Phase:** Phase 9 (D4)
- **Ticket:** D4

**COMP-CLASS-LEVELUP-001**
- **Rule:** 716.2 + 716.2a + 716.2d + 716.3 — Class level progression: level 1 (base abilities) → level 2 (new abilities) → level 3 (more abilities)
- **Composes:** ATOM-716.2-001, ATOM-716.2a-001, ATOM-716.2d-001, ATOM-716.3-001
- **Mechanism:** Complete Class progression
- **Minimal Board:** Player A controls a Class at level 1 with base, level 2, and level 3 abilities.
- **Action:** Activate level 2, then later activate level 3.
- **Expected Result:** At level 1: only base abilities. After level 2 activation: base + level 2 abilities. After level 3 activation: base + level 2 + level 3 abilities. Level 2 can only be activated from level 1, level 3 only from level 2.
- **Phase:** Phase 8
- **Ticket:** NEW — Class full level progression composition test

**COMP-PROTOTYPE-LIFECYCLE-001**
- **Rule:** 718.3 + 718.3a + 718.3b + 718.4 + 718.5 — Prototype lifecycle: cast as prototyped → alternative P/T/cost/color on stack and battlefield → types/abilities unchanged
- **Composes:** ATOM-718.3-001, ATOM-718.3a-001, ATOM-718.3b-001, ATOM-718.4-001, ATOM-718.5-001
- **Mechanism:** Complete prototype card lifecycle
- **Minimal Board:** Player A has a prototype card in hand.
- **Action:** Cast as prototyped spell → resolves → battlefield permanent.
- **Expected Result:** On stack: prototype P/T, mana cost, color. On battlefield: same. Types, name, abilities unchanged. If it later goes to graveyard: normal characteristics resume.
- **Phase:** Phase 9
- **Ticket:** NEW — Prototype full lifecycle composition test

**COMP-END-TURN-FULL-001**
- **Rule:** 723.1a + 723.1b + 723.1c + 723.1d + 723.1e + 723.1f — Full end-the-turn procedure
- **Composes:** ATOM-723.1a-001, ATOM-723.1b-001, ATOM-723.1c-001, ATOM-723.1d-001, ATOM-723.1e-001, ATOM-723.1f-002
- **Mechanism:** Complete end-the-turn sequence in one test
- **Minimal Board:** It is the main phase. The stack has an end-the-turn spell, another spell, and a pending trigger. A creature has lethal damage. Player A has an "at the beginning of your end step" ability.
- **Action:** End-the-turn spell resolves.
- **Expected Result:** (a) Pending trigger purged. (b) All stack objects exiled. (c) SBA destroys creature, no "dies" trigger stacked. (d) Game skips to cleanup. (e) End step trigger does NOT fire. (f) No priority during cleanup (assuming no triggers during the process).
- **Phase:** Phase 8
- **Ticket:** NEW — End-the-turn full procedure composition test

**COMP-MONARCH-COMBAT-001**
- **Rule:** 724.2 + 724.3 — Monarch combat transfer: creature deals combat damage to monarch → controller becomes monarch → old monarch loses designation
- **Composes:** ATOM-724.2-002, ATOM-724.3-001
- **Mechanism:** Combat-driven monarch transfer
- **Minimal Board:** Player A is the monarch. Player B attacks with a creature.
- **Action:** Combat damage is dealt to Player A.
- **Expected Result:** The combat damage trigger fires. Player B becomes monarch. Player A ceases to be monarch. At Player B's next end step, they draw a card (monarch end-step trigger).
- **Phase:** Phase 7 (D15)
- **Ticket:** D15

**COMP-MERGE-LEAVE-001**
- **Rule:** 729.2 + 729.3 + 729.3c — Merge then leave: merge two creatures → destroy → both components go to graveyard → effects that track "it" find both
- **Composes:** ATOM-729.2-001, ATOM-729.3-001, ATOM-729.3c-001
- **Mechanism:** Merged permanent lifecycle through destruction
- **Minimal Board:** Two creature cards merged. An effect exiles the merged permanent with "return it to the battlefield."
- **Action:** The merged permanent is exiled.
- **Expected Result:** Both components are exiled. Both are returned to the battlefield as separate permanents.
- **Phase:** Phase 9
- **Ticket:** NEW — Merged permanent exile-and-return composition test

---

### Gap Report

| Gap | Description | Expected Coverage |
|-----|-------------|-------------------|
| **Saga + Read Ahead + Continuous Effects** | A Saga with read ahead that is also a creature (714.1a) — interaction between chapter abilities, lore counters, and creature combat. No explicit CR sub-rule tests this intersection. | Phase 8 — needs a COMP test combining 714.1a + 714.3a + creature combat |
| **Adventure exile play permission tracking** | 715.3d grants play permission from exile. The engine needs a mechanism to track this permission (per-player, per-card). No ticket explicitly covers the permission tracking data structure — D4 covers CardLayout but not exile-play-permission. | Phase 9 (D4) — may need NEW ticket for exile permission tracking |
| **Omen shuffle-into-library vs replacement effects** | 720.3d says the Omen is shuffled into the library. If a replacement effect says "if this would go to the library, exile it instead," the interaction is untested. | Phase 9 (D4) + Phase 6 — needs a COMP test |
| **Station + Summoning Sickness interaction** | 721.2b makes a Station become a creature. If it gained creature type mid-turn (via counter addition), does it have summoning sickness? 302.6 says yes unless it had haste or was controlled since start of turn. No explicit test for this Station-specific interaction. | Phase 8 — likely covered by existing summoning sickness rules but needs explicit Station test |
| **Player control + triggered abilities** | 722.5 says the controller makes all decisions. What happens if a triggered ability controlled by the controlled player asks for a decision during the controlled turn? The engine needs to route that to the controlling player's DecisionProvider. Not explicitly covered. | Phase 8 — DecisionProvider routing for triggered ability decisions during player control |
| **Day/Night + daybound/nightbound transforms** | 730.1 defines day/night designations but the actual daybound/nightbound keyword (702.145) triggers DFC transformations when the designation changes. This session covers the designation tracking; the DFC transform trigger is in Session 8 (702.145). Cross-session dependency. | Phase 9 (D14) — Session 8 covers 702.145; Session 9B covers 730.x |
| **Loop detection + delta log integration** | 731.1b–731.6 define loop rules. The engine's loop detection (D26) will need to integrate with the delta log to detect repeated game states. The delta log is designed for trigger scanning (Phase 7), not loop detection. D26 roadmap mentions "GameNumber stub" but doesn't detail how delta log serves loop detection. | Phase 7 (D26) / Phase 9 — may need architecture note for delta log + loop detection integration |
| ~~Illegal action rollback~~ | ~~Removed: 732.1/732.2 reclassified to ALREADY-HANDLED-BY-DESIGN. Engine prevents illegal actions via validation; T18 covers casting rollback for partial failures.~~ | ~~Covered by T18~~ |

--- End of Chunk 4 ---

