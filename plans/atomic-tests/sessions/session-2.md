# Session 2: Chapter 2 — Parts of a Card (Rules 200–213)

> Generated: 2026-04-02
> CR Source: Chapter 2 — Parts of a Card
> Scope: Rules 200.1–213.1g (~95 sub-rules)
> Simulator context: Post-Phase 4.5, pre-Phase 5

---

## Rules 200–201: General + Name

### 200.1 — Parts of a card enumeration

**Classification: PURE-DEF.** Lists the physical parts of a card (name, mana cost, illustration, etc.). No independent mechanical consequence — each part is defined by its own subsequent rule.

### 200.2 — Parts that are characteristics

**Classification: PURE-DEF.** Cross-reference to rule 109.3 (characteristics definition). Names a relationship.

### 200.3 — Non-card objects have characteristic parts only

**Classification: PURE-DEF.** Tokens and copies have characteristic parts but not non-characteristic parts. Cross-references 111 and 707.

---

### 201.1 — Name printed location

**Classification: PURE-DEF.** Physical layout information — no game-mechanical consequence.

### 201.2 — Name is always the English version

**Classification: PURE-DEF.** Oracle reference convention. Engine stores names as strings; language is not a concern.

### 201.2a — "Same name" definition (at least one name in common; no-name objects never share a name)

This defines how name comparison works for **all** name-checking effects: legend rule (704.5j), targeting effects like Bile Blight ("target creature and all other creatures with the same name"), graveyard-recursive effects like Bloodbond March ("returns all cards with the same name"), and "choose a card name" effects. The engine's `has_same_name(a, b) -> bool` utility must be flexible enough to accommodate all of these.

**ATOM-201.2a-001**
- **Rule:** 201.2a — Two objects have the same name if they share at least one name in common.
- **Mechanism:** General same-name comparison for effects (Bile Blight pattern: "all creatures with the same name")
- **Minimal Board:** Three creatures on the battlefield: two named "Rat Token" and one named "Grizzly Bears". An effect says "target creature and all other creatures with the same name get -3/-3 until end of turn" targeting one of the Rat Tokens.
- **Action:** Resolve the effect
- **Expected Result:** Both Rat Tokens get -3/-3; Grizzly Bears is unaffected. The same-name check matches both Rat Tokens.
- **Phase:** Phase 5 Pre-Work (T14 uses same-name for legend rule; general same-name utility is broader)
- **Ticket:** T14, NEW — general `has_same_name` utility function

**ATOM-201.2a-002**
- **Rule:** 201.2a — An object with no name doesn't have the same name as any other object, including another object with no name.
- **Mechanism:** Name comparison for nameless objects (face-down creatures)
- **Minimal Board:** Two face-down creatures (no name) on the battlefield. An effect says "target creature and all other creatures with the same name get -3/-3" targeting one face-down creature.
- **Action:** Resolve the effect
- **Expected Result:** Only the targeted face-down creature gets -3/-3. The other face-down creature is NOT affected — nameless objects never share a name, even with other nameless objects.
- **Phase:** Phase 9 (face-down permanents)
- **Ticket:** NEW — nameless-object name comparison guard

### 201.2b — "Different names" definition (each has at least one name, no names in common)

**Classification: TESTABLE.** The CR includes an Example (Liliana's Contract), confirming non-obvious behavior. However, the "different names" check is used by triggered abilities and specific card effects, not core engine rules.

**ATOM-201.2b-001**
- **Rule:** 201.2b — Objects have different names only if each has at least one name and no names in common. A nameless object prevents a group from all having "different names."
- **Mechanism:** "Different names" predicate for card effects
- **Minimal Board:** Three named Demons + one face-down creature (no name) that is a Demon. An effect checks "four or more Demons with different names."
- **Action:** The check is evaluated
- **Expected Result:** Returns false — the nameless creature prevents the group from qualifying as "four Demons with different names"
- **Phase:** Phase 9 (face-down permanents needed for nameless objects; triggered ability check needs Phase 7)
- **Ticket:** NEW — "different names" group predicate utility function

**ATOM-201.2b-002**
- **Rule:** 201.2b — Two objects in the group sharing a name disqualifies the group from "all different names."
- **Mechanism:** "Different names" predicate rejects duplicates within the group
- **Minimal Board:** Four Demons: "Griselbrand," "Razaketh," "Griselbrand" (duplicate), and "Belzenlok." An effect checks "four or more Demons with different names."
- **Action:** The check is evaluated
- **Expected Result:** Returns false — two Demons share the name "Griselbrand," so the group does not have four Demons with different names
- **Phase:** Phase 8 (card effect checks)
- **Ticket:** NEW — "different names" group predicate utility function

### 201.2c — "Different name" (singular comparison) — first object must have a name and no names in common with others

**Classification: TESTABLE.** Subtle asymmetry: a nameless first object does NOT have a "different name" from named objects.

**ATOM-201.2c-001**
- **Rule:** 201.2c — An object with no name does not have a "different name" than any other object.
- **Mechanism:** Singular "different name" comparison predicate
- **Minimal Board:** One face-down creature (no name), one named creature
- **Action:** An effect checks if the face-down creature has a "different name" than the named creature
- **Expected Result:** Returns false — nameless object never has a "different name"
- **Phase:** Phase 9 (face-down permanents)
- **Ticket:** NEW — singular "different name" predicate

**ATOM-201.2c-002**
- **Rule:** 201.2c — The group check variant: the first object has a "different name" than each object in a group if it has at least one name and shares no names with any of them.
- **Mechanism:** Singular "different name" against a group
- **Minimal Board:** Five creatures: "Alpha" (the first object), then a group of "Beta," "Gamma," "Alpha," "Delta." An effect checks if the first creature "has a different name than each other creature you control."
- **Action:** The check is evaluated
- **Expected Result:** Returns false — the first object ("Alpha") shares a name with another object in the group ("Alpha"). For the check to pass, the first object would need no names in common with any member of the group.
- **Phase:** Phase 8 (card effect checks)
- **Ticket:** NEW — singular "different name" predicate (group variant)

**ATOM-201.2c-003**
- **Rule:** 201.2c — A nameless first object compared against a group that includes another nameless object still returns false.
- **Mechanism:** Singular "different name" — nameless vs group containing nameless
- **Minimal Board:** Three creatures: one face-down (no name, the first object), one named "Beta," one face-down (no name). An effect checks if the first creature "has a different name than each other creature you control."
- **Action:** The check is evaluated
- **Expected Result:** Returns false — a nameless object never has a "different name" than any other object (per 201.2c), regardless of whether the group contains other nameless objects. The predicate requires the first object to have at least one name.
- **Phase:** Phase 9 (face-down permanents)
- **Ticket:** NEW — singular "different name" predicate (nameless edge case)

### 201.3 — Interchangeable names

**Classification: OUT-OF-SCOPE.** Interchangeable names are a recent rules addition for specific promotional/reprint cards. No current or planned cards use this mechanic.

### 201.3a — Interchangeable names treated as same name for rules

**Classification: OUT-OF-SCOPE.** Depends on 201.3.

### 201.3b — Interchangeable names for deck construction

**Classification: OUT-OF-SCOPE.** Deck construction detail.

### 201.3c — Interchangeable names indicator

**Classification: PURE-DEF.** Physical card indicator.

### 201.4 — "Choose a card name" must be Oracle-legal name

**Classification: DEFERRED.** Requires Oracle card reference database integration. Relevant when Pithing Needle / Meddling Mage are implemented (D18 in roadmap, Phase 8).

### 201.4a — Choose a name with characteristics uses Oracle text

**Classification: DEFERRED.** Same as 201.4.

### 201.4b — Split card name choice

**Classification: DEFERRED.** Split cards are Phase 9 (D4).

### 201.4c — Flip card alternative name

**Classification: DEFERRED.** Flip cards — Vintage/Legacy legal. Phase 9.

### 201.4d — Back face of DFC name choice

**Classification: DEFERRED.** DFCs are Phase 9 (D3).

### 201.4e — Meld pair combined back face name

**Classification: DEFERRED.** Meld — Pioneer/Modern legal. Phase 9.

### 201.4f — Adventurer card alternative name

**Classification: DEFERRED.** Adventure is Phase 9 (D4).

### 201.4g — Interchangeable names and "choose a name"

**Classification: OUT-OF-SCOPE.** Depends on 201.3.

### 201.5 — Self-referential name means "this object"

**Classification: PURE-DEF.** The engine architecture gives us this for free. When cards are written to the registry, they use "this permanent/spell/creature" language referencing the `ObjectId`, not a text string. Abilities refer to game objects, not names. No independent test needed — correctness is structural.

### 201.5a — Granted ability referring to source by name

**Classification: PURE-DEF.** Same as 201.5 — abilities refer to game objects via `ObjectId`, not text strings. When an ability grants another ability, the source binding is structural (the granted ability stores a reference to the source object's ID). No independent test needed.

### 201.5b — Gained ability name substitution

**Classification: PURE-DEF.** Same as 201.5/201.5a — the engine's ability system binds to `ObjectId`, not name strings. When an object gains an ability, the ability's "self" reference automatically points to the gaining object because the ability is instantiated on that object. The CR's examples (Quicksilver Elemental, Glacial Ray) describe text-level name substitution, but the engine never operates on name strings for self-reference. No independent test needed.

### 201.5c — Shortened name references

**Classification: PURE-DEF.** Oracle errata concern, not engine behavior.

### 201.6 — Secondary title bar / alternate name

**Classification: OUT-OF-SCOPE.** Promotional/alternate-art card layout concern. The engine uses Oracle names exclusively. No planned cards use secondary title bars. Consistent with session-1 classification of non-mechanical card features.

---

## Rules 202: Mana Cost and Color

### 202.1 — Mana cost indicated by mana symbols

**Classification: PURE-DEF.** Physical card layout description. The engine already stores mana cost as `Vec<ManaSymbol>`.

### 202.1a — Paying mana cost requires matching colored/colorless symbols + generic

**Classification: PURE-DEF.** Prerequisite understanding for the mana payment system. The payment logic is already implemented in `types/mana.rs` (`can_pay`/`pay`). This rule describes the semantics of mana payment — tested implicitly by every spell-casting test.

### 202.1b — Objects with no mana cost (lands, tokens, etc.) — unpayable cost

**Classification: TESTABLE.** An object with no mana cost has an unpayable cost per rule 118.6. The engine must reject attempts to cast such objects without an alternative cost.

**ATOM-202.1b-001**
- **Rule:** 202.1b — Having no mana cost represents an unpayable cost.
- **Mechanism:** Cast rejection for no-mana-cost cards
- **Minimal Board:** A card in hand with `mana_cost: None` and no alternative costs
- **Action:** Attempt to cast the card
- **Expected Result:** Cast is rejected — cannot pay an unpayable cost
- **Phase:** Phase 5 Pre-Work (T18 — no-mana-cost guard, E27)
- **Ticket:** T18 (step 2)

**ATOM-202.1b-002**
- **Rule:** 202.1b — Tokens have no mana cost unless the creating effect specifies otherwise.
- **Mechanism:** Token mana cost default
- **Minimal Board:** A token created by an effect that does not specify a mana cost
- **Action:** Query the token's mana cost
- **Expected Result:** Token has no mana cost (mana_cost = None)
- **Phase:** Phase 8 (token creation pipeline)
- **Ticket:** NEW — token default mana cost = None
- **Note:** Named tokens that copy a card (e.g., tokens created by effects that specify a mana cost) should be tested in the "named tokens" section. Cross-reference when that section is written.

### 202.2 — Color determined from mana symbols in mana cost

**Classification: TESTABLE.** This is a computed property — the engine must derive color from mana cost. The CR includes an Example confirming non-obvious behavior.

**ATOM-202.2-001**
- **Rule:** 202.2 — An object is the color(s) of the mana symbols in its mana cost.
- **Mechanism:** Color derivation from mana cost
- **Minimal Board:** A card with mana cost {2}{W} (one white symbol + generic)
- **Action:** Query the card's colors
- **Expected Result:** The card is white (and only white)
- **Phase:** Phase 5 Layers (L10 — `get_effective_colors` oracle function)
- **Ticket:** L10

**ATOM-202.2-002**
- **Rule:** 202.2 — Color is determined from mana symbols, not frame color.
- **Mechanism:** Color derivation ignores frame
- **Minimal Board:** A card with mana cost {2}{W}{B}
- **Action:** Query the card's colors
- **Expected Result:** The card is both white AND black (multicolored)
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

### 202.2a — The five colors

**Classification: BOUNDARY-DEF.** Defines the set of exactly five colors.

**ATOM-202.2a-001**
- **Rule:** 202.2a — The five colors are white, blue, black, red, and green.
- **Mechanism:** Color enum completeness
- **Minimal Board:** N/A (type system test)
- **Action:** Enumerate the Color enum
- **Expected Result:** Exactly five variants: White, Blue, Black, Red, Green. No additional colors exist. Colorless is NOT a color.
- **Phase:** Phase 1 (already implemented in `types/colors.rs`)
- **Ticket:** ALREADY-IMPLEMENTED — Color enum exists with 5 variants

### 202.2b — No colored mana symbols → colorless

**Classification: TESTABLE.** An object with only generic mana symbols is colorless. The engine must compute this correctly.

**ATOM-202.2b-001**
- **Rule:** 202.2b — Objects with no colored mana symbols in their mana costs are colorless.
- **Mechanism:** Colorless derivation from pure-generic cost
- **Minimal Board:** An artifact with mana cost {2} (only generic)
- **Action:** Query its colors
- **Expected Result:** The object is colorless (empty color set)
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

**ATOM-202.2b-002**
- **Rule:** 202.2b — An object with no mana cost is colorless (unless it has a color indicator).
- **Mechanism:** Colorless derivation from absent mana cost
- **Minimal Board:** A land card (no mana cost, no color indicator)
- **Action:** Query its colors
- **Expected Result:** The object is colorless
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

### 202.2c — Multiple colored symbols → multicolored

**Classification: TESTABLE.** Object with two+ different colored mana symbols is each of those colors.

**ATOM-202.2c-001**
- **Rule:** 202.2c — An object with two or more different colored mana symbols is each of those colors.
- **Mechanism:** Multicolor derivation
- **Minimal Board:** A card with mana cost {1}{W}{U}{B}
- **Action:** Query its colors
- **Expected Result:** The object is white, blue, AND black (three colors)
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

### 202.2d — Hybrid and Phyrexian mana symbols contribute ALL their colors

**Classification: TESTABLE.** A card with {W/U} in its cost is both white and blue, even though only one color is paid.

**ATOM-202.2d-001**
- **Rule:** 202.2d — An object with hybrid mana symbols is all colors of those symbols.
- **Mechanism:** Hybrid mana color contribution
- **Minimal Board:** A card with mana cost {W/U}{W/U}
- **Action:** Query its colors
- **Expected Result:** The card is both white AND blue
- **Phase:** Phase 5 Layers (L10) — hybrid mana symbols already exist in `ManaSymbol::Hybrid`
- **Ticket:** L10

**ATOM-202.2d-002**
- **Rule:** 202.2d — An object with Phyrexian mana symbols is the color of those symbols.
- **Mechanism:** Phyrexian mana color contribution
- **Minimal Board:** A card with mana cost {1}{W/P}{W/P}
- **Action:** Query its colors
- **Expected Result:** The card is white (Phyrexian white mana symbol contributes white)
- **Phase:** Phase 5 Layers (L10) — Phyrexian mana symbols exist in `ManaSymbol::Phyrexian`
- **Ticket:** L10

### 202.2e — Color indicator

**Classification: TESTABLE.** An object with a color indicator is each color denoted by that indicator. Cross-references rule 204.

**ATOM-202.2e-001**
- **Rule:** 202.2e — An object with a color indicator is each color denoted by that indicator.
- **Mechanism:** Color indicator overrides absent mana cost
- **Minimal Board:** A card with no mana cost but `color_indicator: Some(vec![Color::Red, Color::Green])`
- **Action:** Query its colors
- **Expected Result:** The card is red and green
- **Phase:** Phase 5 Pre-Work (T05 — color_indicator field) + Phase 5 Layers (L10 — color computation)
- **Ticket:** T05, L10

### 202.2f — Effects may change color

**Classification: PURE-DEF.** Cross-reference to rule 105.3. The mechanism for changing color is Layer 5, which is covered by L10. No independent test needed here — the layer system tests cover this.

### 202.3 — Mana value = total mana in cost regardless of color

**Classification: TESTABLE.** Computed value. The CR includes an Example. Engine must implement `get_mana_value()`.

**ATOM-202.3-001**
- **Rule:** 202.3 — Mana value is the total amount of mana in the mana cost.
- **Mechanism:** Mana value computation
- **Minimal Board:** A card with mana cost {3}{U}{U}
- **Action:** Query its mana value
- **Expected Result:** Mana value = 5
- **Phase:** Backlog — cost pipeline (was Phase 5 Layers)
- **Ticket:** L10

### 202.3a — No mana cost → mana value 0 (with exceptions for nonmodal DFC backs and melds)

**Classification: TESTABLE.** Objects with no mana cost have MV 0, except DFC backs and melds (which use front face cost).

**ATOM-202.3a-001**
- **Rule:** 202.3a — The mana value of an object with no mana cost is 0.
- **Mechanism:** Mana value for costless objects
- **Minimal Board:** A land card (no mana cost)
- **Action:** Query its mana value
- **Expected Result:** Mana value = 0
- **Phase:** Backlog — cost pipeline (was Phase 5 Layers)
- **Ticket:** L10

**ATOM-202.3a-002**
- **Rule:** 202.3a — Exception: back face of a nonmodal DFC has mana value calculated from front face.
- **Mechanism:** DFC back face mana value
- **Minimal Board:** A nonmodal DFC with front face mana cost {2}{R}{G}, transformed to back face
- **Action:** Query the back face's mana value
- **Expected Result:** Mana value = 4 (uses front face cost)
- **Phase:** Phase 9 (DFCs — D3)
- **Ticket:** NEW — DFC back face mana value uses front face cost

### 202.3b — Nonmodal DFC back face MV calculation (with copy exception)

**Classification: TESTABLE.** CR includes three Examples — clearly non-obvious. Copy of a DFC back face has MV 0.

**ATOM-202.3b-001**
- **Rule:** 202.3b — A copy of the back face of a nonmodal DFC has mana value 0.
- **Mechanism:** Copy of DFC back face mana value
- **Minimal Board:** A Clone that enters as a copy of a transformed DFC back face
- **Action:** Query the Clone's mana value
- **Expected Result:** Mana value = 0 (it's a copy of a back face, not the back face itself)
- **Phase:** Phase 9 (DFCs) + Phase 6 (copy effects)
- **Ticket:** NEW — copy of DFC back face MV = 0

### 202.3c — Melded permanent MV = combined front faces

**Classification: DEFERRED (stretch goal).** Meld cards are legal in common constructed formats (Pioneer, Modern). However, meld is a complex mechanic that requires significant infrastructure (tracking paired cards, combined permanents). Deferred until meld infrastructure exists.

> **Audit note:** Reclassified from OUT-OF-SCOPE. Cards like Urza, Lord Protector / Mishra, Claimed by Gix see competitive play.

### 202.3d — Split card MV (combined off-stack, chosen half on stack)

**Classification: DEFERRED.** Split cards are Phase 9 (D4).

### 202.3e — X in mana cost: X=0 off stack, X=chosen value on stack

**Classification: TESTABLE.** Computed value that changes based on zone. Engine must handle X differently by zone.

**ATOM-202.3e-001**
- **Rule:** 202.3e — X is treated as 0 while the object is not on the stack.
- **Mechanism:** Mana value computation with X in non-stack zones
- **Minimal Board:** A card with mana cost {X}{R} in a player's hand
- **Action:** Query its mana value
- **Expected Result:** Mana value = 1 (X=0, plus {R}=1)
- **Phase:** Backlog — cost pipeline (was Phase 5 Layers)
- **Ticket:** L10

**ATOM-202.3e-002**
- **Rule:** 202.3e — X is treated as the chosen value while on the stack.
- **Mechanism:** Mana value computation with X on stack
- **Minimal Board:** A spell with mana cost {X}{R} on the stack with X=5
- **Action:** Query its mana value
- **Expected Result:** Mana value = 6 (X=5, plus {R}=1)
- **Phase:** Backlog — cost pipeline (was Phase 5 Pre-Work)
- **Ticket:** T06, L10

### 202.3f — Hybrid mana MV uses largest component

**Classification: TESTABLE.** CR includes two Examples. {W/U} contributes 1, {2/B} contributes 2.

**ATOM-202.3f-001**
- **Rule:** 202.3f — Hybrid mana uses largest component for MV calculation.
- **Mechanism:** Mana value computation with hybrid symbols
- **Minimal Board:** A card with mana cost {1}{W/U}{W/U}
- **Action:** Query its mana value
- **Expected Result:** Mana value = 3 (1 generic + 1 for each hybrid, largest component of {W/U} is 1)
- **Phase:** Backlog — cost pipeline (was Phase 5 Layers)
- **Ticket:** L10

**ATOM-202.3f-002**
- **Rule:** 202.3f — MonoHybrid {2/C} contributes 2 to MV.
- **Mechanism:** Mana value computation with mono-hybrid symbols
- **Minimal Board:** A card with mana cost {2/B}{2/B}{2/B}
- **Action:** Query its mana value
- **Expected Result:** Mana value = 6 (each {2/B} contributes 2, the larger component)
- **Phase:** Backlog — cost pipeline (was Phase 5 Layers)
- **Ticket:** L10

### 202.3g — Phyrexian mana contributes 1 each to MV

**Classification: TESTABLE.** CR includes an Example.

**ATOM-202.3g-001**
- **Rule:** 202.3g — Each Phyrexian mana symbol contributes 1 to mana value.
- **Mechanism:** Mana value computation with Phyrexian symbols
- **Minimal Board:** A card with mana cost {1}{W/P}{W/P}
- **Action:** Query its mana value
- **Expected Result:** Mana value = 3 (1 generic + 1 per Phyrexian symbol)
- **Phase:** Backlog — cost pipeline (was Phase 5 Layers)
- **Ticket:** L10

### 202.4 — Additional costs are not part of mana cost

**Classification: PURE-DEF.** Defines a boundary between mana cost and additional costs. The engine already separates these (mana cost on `CardData`, additional costs via `AdditionalCost` enum from T17). No independent test — correctness is verified by casting pipeline tests in T18.

---

## Rules 203–204: Illustration + Color Indicator

### 203.1 — Illustration has no effect on game play

**Classification: PURE-DEF.** Explicitly states no game-mechanical consequence. The engine does not model illustrations.

### 204.1 — Color indicator printed location and description

**Classification: PURE-DEF.** Physical layout description. The engine stores `color_indicator: Option<Vec<Color>>` on `CardData` (T05).

### 204.2 — Object with color indicator is each color denoted by it

**Classification: TESTABLE.** Duplicate of 202.2e — same mechanical behavior. Test already generated as ATOM-202.2e-001. No additional test needed; cross-reference only.

---

## Rules 205: Type Line

### 205.1 — Type line contains card type(s), subtype(s), supertype(s)

**Classification: PURE-DEF.** Structural description. The engine stores types, subtypes, and supertypes on `CardData`.

### 205.1a — Effects that set card types: new type replaces existing (except instant/sorcery retained), subtypes correlation, removal cascading

**Classification: TESTABLE.** This is a complex rule governing Layer 4 type-changing behavior. Multiple testable behaviors:

**ATOM-205.1a-001**
- **Rule:** 205.1a — When an effect sets an object's card type, the new type(s) replace existing types.
- **Mechanism:** Layer 4 type replacement
- **Minimal Board:** An artifact on the battlefield. An effect sets its type to "creature."
- **Action:** Apply the type-setting effect; query the object's types
- **Expected Result:** The object is a creature only (artifact type removed)
- **Phase:** Phase 5 Layers (L10 — Layer 4 type operations)
- **Ticket:** L10

**ATOM-205.1a-002**
- **Rule:** 205.1a — An object with instant or sorcery card type retains that type when other types are set.
- **Mechanism:** Instant/sorcery type retention in L4
- **Minimal Board:** An instant spell on the stack. An effect attempts to set its type to "creature."
- **Action:** Apply the type-setting effect; query the object's types
- **Expected Result:** The object is both an instant and a creature (instant type retained)
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

> **Audit note:** This may be a degenerate/catchall case. No known effects set a nonpermanent spell's type to a permanent type (type changes can only apply on the stack for instants/sorceries). Retained for completeness — the engine's Layer 4 logic should still handle this correctly even if no current card triggers it.

**ATOM-205.1a-003**
- **Rule:** 205.1a — When an effect sets subtypes, new subtypes replace existing subtypes of the appropriate set only.
- **Mechanism:** Subtype set replacement scoped by card type
- **Minimal Board:** A "Land Creature — Forest Dryad" permanent. An effect sets its creature type to "Goblin."
- **Action:** Apply the subtype-setting effect; query subtypes
- **Expected Result:** Subtypes are Forest (land type, unaffected) and Goblin (replaced Dryad in creature type set)
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

**ATOM-205.1a-004**
- **Rule:** 205.1a — If an object's card type is removed, correlated subtypes are removed unless they correlate to a remaining card type.
- **Mechanism:** Subtype removal on type loss
- **Minimal Board:** A "Land Creature — Forest Dryad" permanent. An effect removes the creature type.
- **Action:** Apply the type-removal effect; query subtypes
- **Expected Result:** Dryad subtype is removed (creature subtype with no creature type). Forest subtype is retained (land subtype, land type still present).
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

> **Audit note:** No subtype overlap exists between type categories in the CR (creature types, land types, artifact types, enchantment types, planeswalker types, and spell types are all disjoint sets). A subtype can only correlate to one card type at a time, so the "unless they correlate to a remaining card type" clause is relevant only for multi-type objects where a subtype's category matches a remaining type (e.g., Forest stays because the land type is still present).

**ATOM-205.1a-005**
- **Rule:** 205.1a — Counters, effects, and damage remain on an object when its type changes, even if meaningless to the new type.
- **Mechanism:** State persistence across type change
- **Minimal Board:** A creature with 2 damage marked and a +1/+1 counter. An effect removes its creature type (making it a non-creature).
- **Action:** Apply the type change; inspect damage and counters
- **Expected Result:** Damage marking and +1/+1 counter remain on the permanent (even though they are meaningless on a non-creature)
- **Phase:** Phase 5 Layers (L10) + Phase 5 Pre-Work (T01 counters)
- **Ticket:** L10

### 205.1b — Effects that retain prior types ("in addition to," "still a," "artifact creature")

**Classification: TESTABLE.** CR includes two Examples. Governs how type-adding effects interact with existing types.

**ATOM-205.1b-001**
- **Rule:** 205.1b — An effect that makes something an "artifact creature" retains all prior card types and subtypes.
- **Mechanism:** Type-adding with "artifact creature" retention rule
- **Minimal Board:** An enchantment permanent is on the battlefield. An effect makes it an "artifact creature."
- **Action:** Apply the effect; query types and subtypes
- **Expected Result:** The permanent is an artifact, enchantment, AND creature. The "artifact creature" phrasing implicitly retains all prior types and subtypes per 205.1b.
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

> **Audit note:** Changed from plain enchantment to artifact enchantment to properly test the "retains all prior card types" behavior — a plain enchantment becoming "artifact enchantment creature" doesn't exercise the implicit retention of the enchantment type (it could just be re-adding it).

**ATOM-205.1b-002**
- **Rule:** 205.1b — An effect that states "becomes a [creature type] artifact creature" retains prior card types and non-creature subtypes, but replaces creature types.
- **Mechanism:** Selective creature type replacement with other type retention
- **Minimal Board:** A "Land Creature — Forest Dryad" permanent. An effect makes it a "Goblin artifact creature."
- **Action:** Apply the effect; query types and subtypes
- **Expected Result:** Types include land, artifact, creature. Subtypes include Forest (land type retained), Goblin (replaced Dryad). Dryad is gone.
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

**ATOM-205.1b-003**
- **Rule:** 205.1b — "still a [type]" language retains the specified type alongside new types.
- **Mechanism:** "Still a" type retention in Layer 4
- **Minimal Board:** A land permanent on the battlefield (e.g., a Forest). An effect says "All lands are 1/1 creatures that are still lands."
- **Action:** Apply the effect; query the land's types
- **Expected Result:** The permanent is both a land AND a creature. It retains the Forest land subtype. The "still a land" language ensures the land type is not replaced.
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

**ATOM-205.1b-004**
- **Rule:** 205.1b — "in addition to its other types" language retains all prior types.
- **Mechanism:** "In addition to" type retention in Layer 4
- **Minimal Board:** An artifact permanent on the battlefield (e.g., an Equipment). An effect says "target artifact becomes an enchantment in addition to its other types."
- **Action:** Apply the effect; query the artifact's types
- **Expected Result:** The permanent is both an artifact AND an enchantment. The Equipment subtype is retained. "In addition to its other types" preserves all existing types/subtypes.
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

### 205.2 — Card Types

**Classification: PURE-DEF.** Header for card type rules.

### 205.2a — The card types are artifact, battle, conspiracy, creature, dungeon, enchantment, instant, kindred, land, phenomenon, plane, planeswalker, scheme, sorcery, and vanguard

**Classification: BOUNDARY-DEF.** Defines the complete set of card types.

**ATOM-205.2a-001**
- **Rule:** 205.2a — The card types enumeration.
- **Mechanism:** CardType enum completeness
- **Minimal Board:** N/A (type system test)
- **Action:** Verify `CardType` enum contains all game-relevant types
- **Expected Result:** At minimum: Artifact, Creature, Enchantment, Instant, Land, Planeswalker, Sorcery, Battle, Kindred. (Conspiracy, Dungeon, Phenomenon, Plane, Scheme, Vanguard are supplemental product types — may be deferred.) Each type is distinct.
- **Phase:** Phase 1 (already implemented in `types/card_types.rs`)
- **Ticket:** ALREADY-IMPLEMENTED — CardType enum exists. Boundary test verifies no spurious types are included.

### 205.2b — Objects with multiple card types satisfy criteria for any of their types

**Classification: TESTABLE.** An artifact creature counts as both an artifact and a creature for effects that check type.

**ATOM-205.2b-001**
- **Rule:** 205.2b — An object with multiple card types satisfies criteria for any of its types.
- **Mechanism:** Multi-type matching for effects
- **Minimal Board:** An artifact creature (e.g., Solemn Simulacrum, 2/2) on the battlefield. Two effects on the stack: (1) "target creature gets +2/+2 until end of turn" (top of stack, resolves first) and (2) "destroy target artifact" (bottom of stack, resolves second).
- **Action:** Resolve both effects in stack order: +2/+2 resolves first (making it 4/4), then destroy resolves.
- **Expected Result:** Both effects are legal and apply — the artifact creature satisfies "creature" for (1) and "artifact" for (2). After +2/+2 resolves, it is a 4/4 artifact creature. When destroy resolves, it goes to the graveyard as a 4/4.
- **Phase:** Phase 5 Layers (L13 — oracle routing for type queries)
- **Ticket:** L13

> **Audit note:** +2/+2 must resolve first (top of stack) so the creature is still on the battlefield when destroy resolves. If destroy resolved first, the creature would already be gone.

### 205.2c — Tokens have card types even though they aren't cards

**Classification: PURE-DEF.** Prerequisite understanding. Engine already gives tokens card types via `CardData`. No independent test.

### 205.3 — Subtypes

**Classification: PURE-DEF.** Header.

### 205.3a — A card can have one or more subtypes

**Classification: PURE-DEF.** Structural.

### 205.3b — Subtypes listed after long dash; creature subtypes can be two words

**Classification: PURE-DEF.** Formatting/parsing concern, not engine behavior.

### 205.3c — Multi-type cards: each subtype correlated to appropriate card type

**Classification: TESTABLE.** The CR includes an Example (Dryad Arbor: Forest is land type, Dryad is creature type). The rule states that each subtype is correlated to its appropriate card type — the engine must maintain this mapping for subtype operations (removal, replacement) to be scoped correctly.

**ATOM-205.3c-001**
- **Rule:** 205.3c — Each subtype on a multi-type card is correlated to its appropriate card type.
- **Mechanism:** Subtype-to-card-type correlation query
- **Minimal Board:** A "Land Creature — Forest Dryad" permanent (Dryad Arbor).
- **Action:** Query which card type each subtype correlates to
- **Expected Result:** Forest correlates to the land type; Dryad correlates to the creature type. The engine's subtype correlation map correctly associates each subtype with the right card type category.
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

> **Audit note:** Fixed from a subtype-removal test (which duplicates 205.1a-004) to a direct correlation query test. The rule's core assertion is that "each subtype is correlated to its appropriate card type" — the test should verify the correlation itself, not the downstream effect of removal.

### 205.3d — Object can't gain a subtype that doesn't correspond to one of its types

**Classification: TESTABLE.** A non-creature can't gain a creature subtype; a non-land can't gain a land subtype.

**ATOM-205.3d-001**
- **Rule:** 205.3d — An object can't gain a subtype that doesn't correspond to one of its types.
- **Mechanism:** Subtype gain validation
- **Minimal Board:** An enchantment (not a creature) on the battlefield. An effect attempts to give it the creature subtype "Goblin."
- **Action:** Apply the subtype-granting effect
- **Expected Result:** The enchantment does NOT gain the Goblin subtype (it's not a creature). The effect is silently ignored for that aspect.
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

### 205.3e — "Choose a subtype" must be one existing subtype of appropriate card type

**Classification: TESTABLE.** When a player chooses a subtype, it must be from the right set.

**ATOM-205.3e-001**
- **Rule:** 205.3e — A player instructed to choose a creature type cannot choose a land type.
- **Mechanism:** Subtype choice validation
- **Minimal Board:** An effect instructs a player to choose a creature type
- **Action:** Player attempts to choose "Forest" (a land type)
- **Expected Result:** Choice is rejected — Forest is a land type, not a creature type
- **Phase:** Phase 8 (when "choose a type" cards are implemented)
- **Ticket:** NEW — subtype choice validation by card type category

### 205.3f — Obsolete subtypes / Oracle errata

**Classification: PURE-DEF.** Oracle reference concern, not engine behavior.

### 205.3g — Artifact types enumeration

**Classification: BOUNDARY-DEF.** Defines the complete set of artifact subtypes.

**ATOM-205.3g-001**
- **Rule:** 205.3g — The artifact types are Equipment, Vehicle, Food, Treasure, Clue, Blood, etc.
- **Mechanism:** Artifact subtype set membership
- **Minimal Board:** N/A (type system test)
- **Action:** Verify that Equipment IS an artifact type (in-set) and that "Goblin" is NOT an artifact type (out-of-set)
- **Expected Result:** Equipment accepted as valid artifact subtype; Goblin rejected
- **Phase:** Phase 1 (types already defined in `types/card_types.rs`)
- **Ticket:** ALREADY-IMPLEMENTED — ArtifactType enum. Boundary: verify Equipment ∈ ArtifactType, Goblin ∉ ArtifactType.

### 205.3h — Enchantment types enumeration

**Classification: BOUNDARY-DEF.** Defines the complete set of enchantment subtypes.

**ATOM-205.3h-001**
- **Rule:** 205.3h — The enchantment types include Aura, Saga, Class, Case, etc.
- **Mechanism:** Enchantment subtype set membership
- **Minimal Board:** N/A (type system test)
- **Action:** Verify Aura IS an enchantment type; Equipment is NOT
- **Expected Result:** Aura accepted; Equipment rejected
- **Phase:** Phase 1 (types already defined)
- **Ticket:** ALREADY-IMPLEMENTED — EnchantmentType enum exists or will be checked.

### 205.3i — Land types enumeration (including basic land types)

**Classification: BOUNDARY-DEF.** Defines land subtypes. Critically identifies the 5 basic land types: Forest, Island, Mountain, Plains, Swamp.

**ATOM-205.3i-001**
- **Rule:** 205.3i — The basic land types are Forest, Island, Mountain, Plains, and Swamp.
- **Mechanism:** Basic land type set membership
- **Minimal Board:** N/A (type system test)
- **Action:** Verify Forest IS a basic land type (in-set); Cave is a land type but NOT a basic land type (out-of-set for basic)
- **Expected Result:** Forest ∈ basic land types; Cave ∈ land types but ∉ basic land types
- **Phase:** Phase 1 (already implemented in `types/card_types.rs`)
- **Ticket:** ALREADY-IMPLEMENTED — LandType enum with basic land type distinction

### 205.3j — Planeswalker types enumeration

**Classification: BOUNDARY-DEF.** Defines PW subtypes. Engine must have these for legend rule (PW uniqueness rule was removed in 2023, but PW types still exist for cards that reference them).

**ATOM-205.3j-001**
- **Rule:** 205.3j — Planeswalker types (Jace, Chandra, etc.) are a distinct subtype set.
- **Mechanism:** Planeswalker subtype set
- **Minimal Board:** N/A (type system test)
- **Action:** Verify Jace IS a planeswalker type; Goblin is NOT
- **Expected Result:** Jace accepted; Goblin rejected
- **Phase:** Phase 8 (when planeswalker cards are implemented)
- **Ticket:** NEW — PlaneswalkerType enum or equivalent

### 205.3k — Spell types (shared by instants and sorceries)

**Classification: BOUNDARY-DEF.** Adventure, Arcane, Lesson, Omen, Trap.

**ATOM-205.3k-001**
- **Rule:** 205.3k — Spell types are Adventure, Arcane, Lesson, Omen, Trap.
- **Mechanism:** Spell subtype set membership
- **Minimal Board:** N/A (type system test)
- **Action:** Verify Arcane IS a spell type; Equipment is NOT
- **Expected Result:** Arcane accepted; Equipment rejected
- **Phase:** Phase 8–9 (when spell-type cards arrive)
- **Ticket:** NEW — SpellType enum

### 205.3m — Creature types enumeration (shared with kindred)

**Classification: BOUNDARY-DEF.** The full creature type list. Engine needs this for subtype validation.

**ATOM-205.3m-001**
- **Rule:** 205.3m — Creature types include Human, Goblin, Elf, etc. One two-word type: "Time Lord."
- **Mechanism:** Creature subtype set membership
- **Minimal Board:** N/A (type system test)
- **Action:** Verify Human IS a creature type; Forest is NOT a creature type
- **Expected Result:** Human accepted; Forest rejected
- **Phase:** Phase 1 (creature types referenced by existing cards)
- **Ticket:** ALREADY-IMPLEMENTED — CreatureType enum exists (partial coverage; will expand as cards are added)

### 205.3n — Planar types enumeration

**Classification: OUT-OF-SCOPE.** Plane cards are supplemental products, not in any planned phase.

### 205.3p — Dungeon types

**Classification: BOUNDARY-DEF.** Dungeons appear in standard-legal sets (AFR). The single dungeon type is Undercity.

**ATOM-205.3p-001**
- **Rule:** 205.3p — The dungeon type is Undercity.
- **Mechanism:** Dungeon subtype set membership
- **Minimal Board:** N/A (type system test)
- **Action:** Verify Undercity IS a dungeon type; Siege is NOT a dungeon type
- **Expected Result:** Undercity accepted; Siege rejected
- **Phase:** Phase 8–9 (when dungeon cards are implemented)
- **Ticket:** NEW — DungeonType enum or equivalent

> **Audit note:** Reclassified from OUT-OF-SCOPE. Dungeons are legal in common formats.

### 205.3q — Battle types (Siege)

**Classification: BOUNDARY-DEF.** Battles are legal in Standard and other common formats (March of the Machine).

**ATOM-205.3q-001**
- **Rule:** 205.3q — The battle type is Siege.
- **Mechanism:** Battle subtype set membership
- **Minimal Board:** N/A (type system test)
- **Action:** Verify Siege IS a battle type; Aura is NOT a battle type
- **Expected Result:** Siege accepted; Aura rejected
- **Phase:** Phase 8–9 (when battle cards are implemented)
- **Ticket:** NEW — BattleType enum or equivalent

> **Audit note:** Reclassified from OUT-OF-SCOPE. Battle cards see competitive play in Standard/Pioneer.

### 205.3r — Phenomenon, scheme, vanguard, conspiracy cards have no subtypes

**Classification: OUT-OF-SCOPE.** Supplemental product types.

### 205.4 — Supertypes

**Classification: PURE-DEF.** Header.

### 205.4a — Supertypes are basic, legendary, ongoing, snow, world

**Classification: BOUNDARY-DEF.** Defines the complete set of supertypes.

**ATOM-205.4a-001**
- **Rule:** 205.4a — The supertypes are basic, legendary, ongoing, snow, and world.
- **Mechanism:** Supertype enum completeness
- **Minimal Board:** N/A (type system test)
- **Action:** Verify Legendary IS a supertype; Artifact is NOT a supertype (it's a card type)
- **Expected Result:** Legendary ∈ supertypes; Artifact ∉ supertypes
- **Phase:** Phase 1 (already implemented)
- **Ticket:** ALREADY-IMPLEMENTED — Supertype enum exists

### 205.4b — Supertype is independent of card type and subtype; changes don't cascade

**Classification: TESTABLE.** CR includes an Example. Changing types doesn't change supertypes and vice versa.

**ATOM-205.4b-001**
- **Rule:** 205.4b — Changing an object's card types doesn't change its supertypes.
- **Mechanism:** Type-supertype independence
- **Minimal Board:** A legendary land on the battlefield. An effect changes it to a creature.
- **Action:** Apply the type change; query supertypes
- **Expected Result:** The permanent is still legendary (supertype unchanged by type change)
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

> **Audit note (pass 2):** ATOM-205.4b-002 (promoted from COMP-205.1a+205.4b-001) was dropped. It tested type *replacement* (via 205.1a) preserving supertypes, but from the supertype's perspective the rule under test is identical to 001 — supertypes don't change regardless of whether types are added or replaced. The 205.1a replacement behavior is already covered by ATOM-205.1a-001. Keeping only 001 avoids a near-duplicate.

### 205.4c — Any land with supertype "basic" is a basic land; lands without it are nonbasic (even with basic land types)

**Classification: TESTABLE.** A Shock Land (e.g., Steam Vents — Land: Island Mountain) has basic land types but is NOT a basic land because it lacks the Basic supertype.

**ATOM-205.4c-001**
- **Rule:** 205.4c — A land with the Basic supertype is a basic land.
- **Mechanism:** Basic land classification via supertype (positive case)
- **Minimal Board:** A "Basic Land — Forest" permanent (has both the Basic supertype and the Forest land type)
- **Action:** Query whether it is a basic land
- **Expected Result:** is_basic_land = true — the Basic supertype makes it a basic land
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

**ATOM-205.4c-002**
- **Rule:** 205.4c — A land without the Basic supertype is nonbasic, even if it has a basic land type.
- **Mechanism:** Basic land classification via supertype (negative case)
- **Minimal Board:** A dual land with subtypes Island and Mountain but NO Basic supertype (e.g., Steam Vents)
- **Action:** Query whether it is a basic land; query whether it is a nonbasic land
- **Expected Result:** is_basic_land = false; is_nonbasic_land = true — having basic land types is not sufficient; the Basic supertype is required
- **Phase:** Phase 5 Layers (L10 — needed for Blood Moon's "nonbasic lands" filter)
- **Ticket:** L10 (Blood Moon filter: PC12 confirms no Basic supertype added)

### 205.4d — Legendary supertype → legend rule SBA (704.5j)

**Classification: TESTABLE.** Cross-reference to the SBA. The link between the supertype and the SBA must be correct.

**ATOM-205.4d-001**
- **Rule:** 205.4d — A permanent with the legendary supertype is subject to the legend rule.
- **Mechanism:** Legend rule SBA triggered by supertype
- **Minimal Board:** Two permanents with the legendary supertype and the same name, controlled by the same player
- **Action:** State-based actions are checked
- **Expected Result:** Legend rule fires — one must be chosen to keep
- **Phase:** Phase 5 Pre-Work (T14)
- **Ticket:** T14

### 205.4e — Legendary instant/sorcery casting restriction

**Classification: TESTABLE.** A player can't cast a legendary instant or sorcery unless they control a legendary creature or planeswalker.

**ATOM-205.4e-001**
- **Rule:** 205.4e — A legendary instant/sorcery can't be cast unless the caster controls a legendary creature or planeswalker.
- **Mechanism:** Casting restriction for legendary spells
- **Minimal Board:** A legendary sorcery in hand. Player controls no legendary creatures or planeswalkers.
- **Action:** Attempt to cast the legendary sorcery
- **Expected Result:** Cast is rejected — no legendary creature/planeswalker controlled
- **Phase:** Phase 5 Pre-Work (T18 — step 3, E28)
- **Ticket:** T18

**ATOM-205.4e-002**
- **Rule:** 205.4e — A legendary sorcery CAN be cast if the caster controls a legendary creature.
- **Mechanism:** Casting restriction satisfied
- **Minimal Board:** A legendary sorcery in hand. Player controls a legendary creature.
- **Action:** Attempt to cast the legendary sorcery
- **Expected Result:** Cast is allowed
- **Phase:** Phase 5 Pre-Work (T18)
- **Ticket:** T18

### 205.4f — World supertype → world rule SBA (704.5k)

**Classification: TESTABLE.** World permanents have their own SBA (the "world rule").

**ATOM-205.4f-001**
- **Rule:** 205.4f — A permanent with the world supertype is subject to the world rule (704.5k).
- **Mechanism:** World rule SBA
- **Minimal Board:** Two permanents with the World supertype on the battlefield (different controllers or same)
- **Action:** State-based actions are checked
- **Expected Result:** The one with the earlier timestamp is put into its owner's graveyard; the newer one survives. If timestamps are equal, they are all put into their owners' graveyards.
- **Phase:** Phase 5 Pre-Work (SBA expansion) or Phase 8
- **Ticket:** NEW — world rule SBA (704.5k). Not currently in T13–T16.

### 205.4g — Snow supertype: any permanent with it is a snow permanent

**Classification: BOUNDARY-DEF.** A permanent with the Snow supertype is a "snow permanent"; without it, it's a "nonsnow permanent."

**ATOM-205.4g-001**
- **Rule:** 205.4g — A permanent with the snow supertype is a snow permanent; without it, nonsnow.
- **Mechanism:** Snow classification via supertype
- **Minimal Board:** A Snow-Covered Mountain (has Snow supertype) and a regular Mountain (no Snow supertype)
- **Action:** Query is_snow for each
- **Expected Result:** Snow-Covered Mountain → is_snow = true; Mountain → is_snow = false
- **Phase:** Phase 8 (when snow cards arrive; snow mana already in `ManaSymbol::Snow`)
- **Ticket:** NEW — snow permanent classification oracle function

### 205.4h — Ongoing supertype on scheme cards

**Classification: OUT-OF-SCOPE.** Scheme cards are supplemental products.

---

## Rules 206: Expansion Symbol

### 206.1 — Expansion symbol indicates set, no effect on game play

**Classification: PURE-DEF.** Explicitly no game-mechanical consequence. Engine does not model expansion symbols.

### 206.2 — Symbol color indicates rarity

**Classification: PURE-DEF.** No game-mechanical consequence.

### 206.3 — Old cards checking expansion symbol → errata to "name originally printed in"

**Classification: PURE-DEF.** Oracle errata concern.

### 206.3a — City in a Bottle (Arabian Nights names)

**Classification: DEFERRED (stretch goal).** Technically legal in Vintage/Legacy. Card-specific hardcoded name list. Deferred until these niche cards are prioritized.

> **Audit note:** Reclassified from OUT-OF-SCOPE. These cards have unique rulings and are legal in eternal formats, but as cards with hardcoded name lists they are super-deferred.

### 206.3b — Golgothian Sylex (Antiquities names)

**Classification: DEFERRED (stretch goal).** Same as 206.3a.

### 206.3c — Apocalypse Chime (Homelands names)

**Classification: DEFERRED (stretch goal).** Same as 206.3a.

### 206.4 — Deck construction: cards from any printing allowed in format

**Classification: PURE-DEF.** Tournament rule, not engine behavior.

### 206.5 — Full list of expansions on website

**Classification: PURE-DEF.** External reference.

---

## Rules 207: Text Box

### 207.1 — Text box contains rules text defining abilities

**Classification: PURE-DEF.** Structural description. The engine stores abilities on `CardData` via `AbilityDef`.

### 207.2 — Italicized text has no game function

**Classification: PURE-DEF.** Explicitly no game function.

### 207.2a — Reminder text (in parentheses) summarizes rules, no game function

**Classification: PURE-DEF.** No game function.

### 207.2b — Flavor text has no game function

**Classification: PURE-DEF.** No game function.

### 207.2c — Ability words (italicized, no rules meaning)

**Classification: PURE-DEF.** Explicitly no special rules meaning. The engine does not need to track ability words.

### 207.2d — Flavor words (italicized, no rules meaning)

**Classification: PURE-DEF.** Same as 207.2c.

### 207.3 — Decorative icons have no effect on game play

**Classification: PURE-DEF.** Explicitly no effect.

### 207.4 — Chaos symbol on plane cards

**Classification: OUT-OF-SCOPE.** Planechase supplemental format.

### 207.5 — Cryptic Spires circled colors

**Classification: OUT-OF-SCOPE.** Single-card-specific rule for a draft-matters card. Not in any planned phase.

---

## Rules 208: Power/Toughness

### 208.1 — Creature card has power/toughness; can be modified by effects

**Classification: PURE-DEF.** Structural description. Engine already stores P/T on `CardData` and computes effective P/T through the layer system.

### 208.2 — Star (*) in power and/or toughness

**Classification: PURE-DEF.** Header for CDA and replacement-effect P/T rules. The star represents a variable value.

### 208.2a — Characteristic-defining ability (CDA) sets P/T; functions everywhere including outside the game; uses 0 for undetermined numbers

**Classification: TESTABLE.** CR includes an Example (Lost Order of Jarkeld). CDAs define P/T in all zones, and the engine must implement this correctly. Tarmogoyf is the canonical example.

**ATOM-208.2a-001**
- **Rule:** 208.2a — A CDA that sets P/T functions everywhere, even outside the game.
- **Mechanism:** CDA P/T computation in all zones
- **Minimal Board:** A Tarmogoyf-like creature card in a player's graveyard (not on battlefield). Graveyards contain 3 distinct card types.
- **Action:** Query the creature's P/T while in the graveyard
- **Expected Result:** Power = 3, Toughness = 4 (CDA applies in all zones, counts graveyard card types)
- **Phase:** Phase 5 Layers (L04 — `compute_characteristics` CDA handling, L17 — Tarmogoyf card)
- **Ticket:** L04, L17

**ATOM-208.2a-002**
- **Rule:** 208.2a — If a CDA's computation needs an undetermined number, use 0.
- **Mechanism:** CDA fallback to 0 for undetermined values
- **Minimal Board:** A creature with CDA "power and toughness are each equal to 1 plus the number of creatures the chosen player controls" — but no player has been chosen (creature is not on the battlefield).
- **Action:** Query P/T while not on battlefield
- **Expected Result:** Power = 1, Toughness = 1 (undetermined "chosen player" → 0 creatures → 1+0=1)
- **Phase:** Phase 5 Layers (L04)
- **Ticket:** L04

### 208.2b — Replacement effect sets P/T as creature enters or is turned face up; P/T is 0/0 while not on battlefield

**Classification: TESTABLE.** Cards like Corrupted Shapeshifter that choose P/T from a list as they enter. While not on the battlefield, their P/T is 0/0. The "turned face up" clause appears to be futureproofing — no known cards use it currently.

**ATOM-208.2b-001**
- **Rule:** 208.2b — A creature with a replacement-effect-based P/T (choosing from specific values as it enters) has 0/0 while not on the battlefield.
- **Mechanism:** P/T for replacement-based creatures off-battlefield
- **Minimal Board:** A Corrupted Shapeshifter-style card in hand ("As this creature enters, it becomes your choice of a 3/3 creature with flying, a 2/5 creature with vigilance, or a 0/12 creature with defender")
- **Action:** Query its P/T while in hand
- **Expected Result:** Power = 0, Toughness = 0
- **Phase:** Phase 5 Layers (L04) + Phase 7 (replacement effects for ETB choices)
- **Ticket:** NEW — replacement-effect P/T cards show 0/0 off-battlefield

**ATOM-208.2b-002**
- **Rule:** 208.2b — The chosen P/T from a replacement effect affects the creature's copiable values.
- **Mechanism:** Replacement-effect P/T choice becomes copiable
- **Minimal Board:** A Corrupted Shapeshifter enters the battlefield; the player chooses 3/3 with flying.
- **Action:** The creature enters; query its P/T and abilities. Then a Clone copies it.
- **Expected Result:** Corrupted Shapeshifter is a 3/3 with flying. The Clone is also a 3/3 with flying (the chosen values are copiable per rule 707.2).
- **Phase:** Phase 6 (copy effects) + Phase 7 (replacement effects)
- **Ticket:** NEW — copiable values from ETB replacement effect choices

> **Audit note:** Changed from Clone to Corrupted Shapeshifter as the primary example. Clones don't fall under 208.2b — they copy another creature's values rather than choosing from a list of specific P/T values. The "turned face up" variant appears to be futureproofing with no current card examples.

### 208.3 — Noncreature permanent has no P/T; noncreature object not on battlefield has P/T only if printed

**Classification: TESTABLE.** A Vehicle on the battlefield (not crewed) has no P/T. A creature card in hand has printed P/T.

**ATOM-208.3-001**
- **Rule:** 208.3 — A noncreature permanent has no power or toughness.
- **Mechanism:** P/T gated behind creature type
- **Minimal Board:** A Vehicle (artifact, not creature) on the battlefield with printed P/T 4/4
- **Action:** Query its effective P/T
- **Expected Result:** P/T is None/undefined — the Vehicle has no power or toughness because it's not a creature
- **Phase:** Phase 5 Layers (L04 — `EffectiveCharacteristics` gates P/T behind creature type)
- **Ticket:** L04

**ATOM-208.3-002**
- **Rule:** 208.3 — A noncreature object not on the battlefield has P/T only if printed on it.
- **Mechanism:** Off-battlefield P/T for noncreature cards with printed P/T
- **Minimal Board:** A Vehicle card (e.g., Cargo Ship, printed P/T 2/3, artifact — not a creature) in the graveyard. An effect says "return target card with power 2 or less from your graveyard to the battlefield" (no creature restriction).
- **Action:** Check if Cargo Ship is a legal target for the effect
- **Expected Result:** Legal target — Cargo Ship has printed P/T 2/3, so per 208.3 it has power 2 while in the graveyard, satisfying "power 2 or less." The effect doesn't require "creature card," so the Vehicle qualifies.
- **Phase:** Phase 5 Layers (L04)
- **Ticket:** L04

> **Audit note (pass 1):** Changed example from a creature card to a Vehicle card to properly exercise the rule.
>
> **Audit note (pass 2 — judge forum confirmation):** Confirmed via judge ruling: the only reason Alesha, Who Smiles at Death can't return a Vehicle (e.g., Cargo Ship, 2/3) from the graveyard is because Alesha specifies "creature card." If the ability said "card with power 2 or less" without a creature restriction, the Vehicle would be a legal target. Per 208.3, noncreature cards off-battlefield have P/T if printed — stats do not equal creature. Creatures always have stats, but not the reverse. This is also why Gods (e.g., Theros gods) work differently: they lose creature status on the battlefield conditionally, but in other zones they ARE creature cards and have P/T normally. Vehicles/Stations gain creature status on the battlefield but are noncreature cards elsewhere — yet they still have printed P/T available for off-battlefield queries.

### 208.3a — Effects that would set/modify P/T of a noncreature permanent are created but dormant until it becomes a creature

**Classification: TESTABLE.** CR includes an Example (Veteran Motorist + Vehicle). A +1/+1 effect on a Vehicle before it's crewed creates the effect, but it doesn't do anything until the Vehicle becomes a creature.

**ATOM-208.3a-001**
- **Rule:** 208.3a — A P/T-modifying effect on a noncreature permanent is created and applies when it becomes a creature.
- **Mechanism:** Dormant P/T effect activation on type change
- **Minimal Board:** A Vehicle (not a creature) on the battlefield. An effect gives it +1/+1 until end of turn.
- **Action:** The Vehicle is crewed (becomes a creature). Query its P/T.
- **Expected Result:** The +1/+1 effect now applies — P/T includes the +1/+1 bonus on top of the Vehicle's base P/T
- **Phase:** Phase 5 Layers (L04 P/T gating + L07/L08 effect registration)
- **Ticket:** L04, L07
- **Dependency:** Requires Vehicle/crew mechanic (Phase 8, D13). Test is conceptually atomic but needs crew infrastructure.

### 208.4 — "Base power"/"base toughness" terminology

**Classification: PURE-DEF.** Header for base P/T rules.

### 208.4a — Effects that set P/T to specific values may refer to "base" P/T; other effects may further modify

**Classification: PURE-DEF.** Defines terminology relationship to layer 7 sublayers (7b set, 7c modify). Implicitly tested by layer system.

### 208.4b — Effects that check "base" P/T see CDA + set effects, ignoring modify/counter effects

**Classification: TESTABLE.** "Base power" = after L7a (CDA) + L7b (set), ignoring L7c (modify) and L7d (switch).

**ATOM-208.4b-001**
- **Rule:** 208.4b — Checking "base power" ignores effects and counters that modify P/T without setting them.
- **Mechanism:** Base P/T query reads through L7a+L7b only
- **Minimal Board:** A 2/2 creature with a +1/+1 counter and a Giant Growth (+3/+3 until EOT) active
- **Action:** An effect checks the creature's "base power"
- **Expected Result:** Base power = 2 (printed value; ignoring the +1/+1 counter from L7c and Giant Growth from L7c)
- **Phase:** Phase 5 Layers (L04/L06 — need a `get_base_power` oracle function that stops at L7b)
- **Ticket:** NEW — `get_base_power`/`get_base_toughness` oracle functions that compute through L7a+L7b only

### 208.5 — Creature with no value for P/T → use 0

**Classification: TESTABLE.** Safety rule for edge cases.

**ATOM-208.5-001**
- **Rule:** 208.5 — If a creature has no value for its power, its power is 0. Same for toughness.
- **Mechanism:** P/T fallback to 0
- **Minimal Board:** A creature token created without specifying P/T (hypothetical edge case)
- **Action:** Query its P/T
- **Expected Result:** Power = 0, Toughness = 0
- **Phase:** Phase 5 Layers (L04)
- **Ticket:** L04

---

## Rules 209: Loyalty

### 209.1 — Planeswalker card has loyalty number; enters with that many loyalty counters

**Classification: TESTABLE.** The printed loyalty number determines initial loyalty counters on ETB.

**ATOM-209.1-001**
- **Rule:** 209.1 — A planeswalker enters the battlefield with loyalty counters equal to its printed loyalty.
- **Mechanism:** Planeswalker ETB loyalty counter initialization
- **Minimal Board:** A planeswalker card with printed loyalty 4 resolving from the stack
- **Action:** The planeswalker enters the battlefield
- **Expected Result:** The planeswalker permanent has 4 loyalty counters
- **Phase:** Phase 5 Pre-Work (T14 — step 4, planeswalker ETB sets loyalty counters)
- **Ticket:** T14

### 209.2 — Loyalty abilities: activated with loyalty symbol, sorcery-speed, once per turn per permanent

**Classification: TESTABLE.** Loyalty abilities have special activation timing restrictions.

**ATOM-209.2-001**
- **Rule:** 209.2 — A loyalty ability can only be activated at sorcery speed (priority, stack empty, main phase, own turn).
- **Mechanism:** Loyalty ability activation timing
- **Minimal Board:** A planeswalker on the battlefield controlled by the active player, during their main phase, stack empty
- **Action:** Attempt to activate a loyalty ability
- **Expected Result:** Activation succeeds
- **Phase:** Phase 5 Pre-Work (T19 — activation restrictions, SorcerySpeed) + Phase 8 (PW cards)
- **Ticket:** T19, NEW — loyalty ability activation wiring

**ATOM-209.2-002**
- **Rule:** 209.2 — Only one loyalty ability per permanent per turn.
- **Mechanism:** Loyalty ability once-per-turn restriction
- **Minimal Board:** A planeswalker that has already activated a loyalty ability this turn
- **Action:** Attempt to activate another loyalty ability on the same planeswalker
- **Expected Result:** Activation is rejected — a loyalty ability of this permanent was already activated this turn
- **Phase:** Phase 5 Pre-Work (T19 — OncePerTurn restriction)
- **Ticket:** T19

> **Audit note:** Effects exist that bypass this restriction via different mechanisms:
> - **The Chain Veil:** "you may activate one of [planeswalker’s] loyalty abilities this turn as though none of its loyalty abilities have been activated" — this uses a "as though" override that resets the activation check.
> - **Oath of Teferi:** "You may activate the loyalty abilities of planeswalkers you control twice each turn rather than only once." — this changes the limit from 1 to 2.
> These represent different backend implementations: Chain Veil bypasses the restriction check entirely for one activation, while Oath of Teferi modifies the restriction threshold. Both are Phase 8+ concerns but the engine’s once-per-turn system should be designed to accommodate both patterns ("as though" override vs. threshold modification).

---

## Rules 210: Defense

### 210.1 — Battle card has defense number; enters with that many defense counters

**Classification: TESTABLE.** Battles are legal in Standard and other common formats (March of the Machine). Similar to planeswalker loyalty ETB (209.1).

**ATOM-210.1-001**
- **Rule:** 210.1 — A battle enters the battlefield with defense counters equal to its printed defense.
- **Mechanism:** Battle ETB defense counter initialization
- **Minimal Board:** A battle card with printed defense 5 resolving from the stack
- **Action:** The battle enters the battlefield
- **Expected Result:** The battle permanent has 5 defense counters
- **Phase:** Phase 8–9 (when battle cards are implemented)
- **Ticket:** NEW — battle ETB defense counter initialization (analogous to PW loyalty in T14)

> **Audit note:** Reclassified from OUT-OF-SCOPE. Battle cards see competitive play in Standard/Pioneer. The ETB counter pattern is identical to planeswalker loyalty (209.1).

---

## Rules 211–213: Hand Modifier, Life Modifier, Information Below Text Box

### 211.1 — Vanguard hand modifier

**Classification: DEFERRED (stretch goal).** Vanguard hand modifier. Deferred to stretch goal phase.

### 212.1 — Vanguard life modifier

**Classification: DEFERRED (stretch goal).** Vanguard life modifier. Deferred to stretch goal phase.

### 213.1 — Information below text box has no effect on game play

**Classification: PURE-DEF.** Explicitly no effect on game play.

### 213.1a — Collector numbers

**Classification: PURE-DEF.** No game-mechanical consequence.

### 213.1b — Rarity indicator

**Classification: PURE-DEF.** No game-mechanical consequence.

### 213.1c — Promotional information

**Classification: PURE-DEF.** No game-mechanical consequence.

### 213.1d — Interchangeable name version info

**Classification: PURE-DEF.** No game-mechanical consequence.

### 213.1e — Set code and language code

**Classification: PURE-DEF.** No game-mechanical consequence.

### 213.1f — Illustration credit

**Classification: PURE-DEF.** No game-mechanical consequence.

### 213.1g — Legal text (trademark/copyright)

**Classification: PURE-DEF.** No game-mechanical consequence.

---

## Composition Tests

Tests requiring 2+ atomic mechanisms working together. Tests reclassified as effectively atomic during audit are noted.

> **Audit note (pass 1):** Re-evaluated all COMP tests for atomicity. Two were reclassified:
> - **COMP-205.1a+205.4b-001** → initially promoted to ATOM-205.4b-002, then dropped in pass 2 as functionally identical to ATOM-205.4b-001 (supertype independence doesn't care whether types are added or replaced).
> - **COMP-202.3e+209.2-001** → removed. This just observes ATOM-202.3e-002's MV-on-stack value via a counterspell targeting check — the counterspell is the observation method, not a second atomic mechanism.

**COMP-202.2e+202.2b-001**
- **Rule:** 202.2e + 202.2b — Color indicator provides color to an otherwise colorless card (no mana cost).
- **Composes:** ATOM-202.2e-001, ATOM-202.2b-002
- **Mechanism:** Color derivation from color indicator on a card with no mana cost
- **Minimal Board:** A card with no mana cost (would be colorless per 202.2b) and `color_indicator: Some(vec![Color::Green])` (provides color per 202.2e)
- **Action:** Query the card's colors
- **Expected Result:** The card is green (color indicator overrides the colorless default from having no mana cost)
- **Phase:** Phase 5 Pre-Work (T05) + Phase 5 Layers (L10)
- **Ticket:** T05, L10

**COMP-205.4c+L10-001**
- **Rule:** 205.4c + Blood Moon interaction — Blood Moon makes nonbasic lands into Mountains but does NOT add the Basic supertype.
- **Composes:** ATOM-205.4c-002, L10 SetLandType
- **Mechanism:** SetLandType (Blood Moon) + Basic supertype classification
- **Minimal Board:** A nonbasic land (no Basic supertype) on the battlefield. Blood Moon is in play (SetLandType → Mountain on nonbasic lands).
- **Action:** Query whether the affected land is a basic land
- **Expected Result:** is_basic_land = false — Blood Moon changes the land type to Mountain but does NOT add the Basic supertype (per PC12)
- **Phase:** Phase 5 Layers (L10, L17)
- **Ticket:** L10, L17

**COMP-208.3+205.1b-001**
- **Rule:** 208.3 + 205.1b — A noncreature permanent animated into a creature gains P/T.
- **Composes:** ATOM-208.3-001, ATOM-205.1b-001
- **Mechanism:** P/T gating + type addition (Opalescence-style animation)
- **Minimal Board:** An enchantment (noncreature, MV 3) on the battlefield. An Opalescence-style effect makes it an "enchantment creature" with P/T equal to its MV.
- **Action:** Query the enchantment's P/T
- **Expected Result:** P/T = 3/3 (now a creature, P/T applies)
- **Phase:** Phase 5 Layers (L04 P/T gating, L19 Opalescence)
- **Ticket:** L04, L19

---

## Classification Summary

| Rule | Classification | Notes |
|------|---------------|-------|
| 200.1 | PURE-DEF | Card parts enumeration |
| 200.2 | PURE-DEF | Parts that are characteristics — cross-ref 109.3 |
| 200.3 | PURE-DEF | Non-card objects have characteristic parts only |
| 201.1 | PURE-DEF | Name printed location |
| 201.2 | PURE-DEF | English version convention |
| 201.2a | TESTABLE | "Same name" definition — ATOM-201.2a-001, -002 |
| 201.2b | TESTABLE | "Different names" group predicate — ATOM-201.2b-001, -002 |
| 201.2c | TESTABLE | Singular "different name" — ATOM-201.2c-001, -002, -003 |
| 201.3 | OUT-OF-SCOPE | Interchangeable names |
| 201.3a | OUT-OF-SCOPE | Interchangeable names rules effect |
| 201.3b | OUT-OF-SCOPE | Interchangeable names deck construction |
| 201.3c | PURE-DEF | Interchangeable names indicator |
| 201.4 | DEFERRED | "Choose a card name" — needs Oracle DB (Phase 8, D18) |
| 201.4a | DEFERRED | Choose name with characteristics |
| 201.4b | DEFERRED | Split card name choice — Phase 9 (D4) |
| 201.4c | DEFERRED | Flip card name — Phase 9 |
| 201.4d | DEFERRED | DFC back face name — Phase 9 (D3) |
| 201.4e | DEFERRED | Meld pair name — Phase 9 |
| 201.4f | DEFERRED | Adventurer name — Phase 9 (D4) |
| 201.4g | OUT-OF-SCOPE | Interchangeable names + choose |
| 201.5 | PURE-DEF | Self-referential name = this object (engine uses ObjectId, structural) |
| 201.5a | PURE-DEF | Granted ability source binding (structural via ObjectId) |
| 201.5b | PURE-DEF | Gained ability name substitution (structural via ObjectId) |
| 201.5c | PURE-DEF | Shortened name references |
| 201.6 | OUT-OF-SCOPE | Secondary title bar — promotional card layout |
| 202.1 | PURE-DEF | Mana cost location |
| 202.1a | PURE-DEF | Mana cost payment semantics |
| 202.1b | TESTABLE | No mana cost = unpayable — ATOM-202.1b-001, -002 |
| 202.2 | TESTABLE | Color from mana cost — ATOM-202.2-001, -002 |
| 202.2a | BOUNDARY-DEF | Five colors — ATOM-202.2a-001 |
| 202.2b | TESTABLE | No colored symbols → colorless — ATOM-202.2b-001, -002 |
| 202.2c | TESTABLE | Multiple colors → multicolored — ATOM-202.2c-001 |
| 202.2d | TESTABLE | Hybrid/Phyrexian contribute all colors — ATOM-202.2d-001, -002 |
| 202.2e | TESTABLE | Color indicator — ATOM-202.2e-001 |
| 202.2f | PURE-DEF | Effects change color — cross-ref 105.3 |
| 202.3 | TESTABLE | Mana value computation — ATOM-202.3-001 |
| 202.3a | TESTABLE | No mana cost → MV 0 — ATOM-202.3a-001, -002 |
| 202.3b | TESTABLE | DFC back face copy MV = 0 — ATOM-202.3b-001 |
| 202.3c | DEFERRED | Meld MV — deferred (Pioneer/Modern legal) |
| 202.3d | DEFERRED | Split card MV — Phase 9 (D4) |
| 202.3e | TESTABLE | X in MV by zone — ATOM-202.3e-001, -002 |
| 202.3f | TESTABLE | Hybrid MV uses largest — ATOM-202.3f-001, -002 |
| 202.3g | TESTABLE | Phyrexian MV = 1 each — ATOM-202.3g-001 |
| 202.4 | PURE-DEF | Additional costs ≠ mana cost |
| 203.1 | PURE-DEF | Illustration no effect |
| 204.1 | PURE-DEF | Color indicator layout |
| 204.2 | TESTABLE | Color indicator = color (duplicate of 202.2e, cross-ref only) |
| 205.1 | PURE-DEF | Type line structure |
| 205.1a | TESTABLE | Type replacement, instant/sorcery retention, subtype correlation — ATOM-205.1a-001 through -005 |
| 205.1b | TESTABLE | Type retention ("in addition to", "still a", "artifact creature") — ATOM-205.1b-001 through -004 |
| 205.2 | PURE-DEF | Card Types header |
| 205.2a | BOUNDARY-DEF | Card types enumeration — ATOM-205.2a-001 |
| 205.2b | TESTABLE | Multi-type satisfaction — ATOM-205.2b-001 |
| 205.2c | PURE-DEF | Tokens have card types |
| 205.3 | PURE-DEF | Subtypes header |
| 205.3a | PURE-DEF | One or more subtypes |
| 205.3b | PURE-DEF | Subtype formatting |
| 205.3c | TESTABLE | Subtype-card type correlation — ATOM-205.3c-001 |
| 205.3d | TESTABLE | Can't gain non-corresponding subtype — ATOM-205.3d-001 |
| 205.3e | TESTABLE | Choose subtype validation — ATOM-205.3e-001 |
| 205.3f | PURE-DEF | Obsolete subtypes |
| 205.3g | BOUNDARY-DEF | Artifact types — ATOM-205.3g-001 |
| 205.3h | BOUNDARY-DEF | Enchantment types — ATOM-205.3h-001 |
| 205.3i | BOUNDARY-DEF | Land types (basic land types) — ATOM-205.3i-001 |
| 205.3j | BOUNDARY-DEF | Planeswalker types — ATOM-205.3j-001 |
| 205.3k | BOUNDARY-DEF | Spell types — ATOM-205.3k-001 |
| 205.3m | BOUNDARY-DEF | Creature types — ATOM-205.3m-001 |
| 205.3n | OUT-OF-SCOPE | Planar types |
| 205.3p | BOUNDARY-DEF | Dungeon types — ATOM-205.3p-001 |
| 205.3q | BOUNDARY-DEF | Battle types (Siege) — ATOM-205.3q-001 |
| 205.3r | OUT-OF-SCOPE | No-subtype supplemental types |
| 205.4 | PURE-DEF | Supertypes header |
| 205.4a | BOUNDARY-DEF | Supertypes enumeration — ATOM-205.4a-001 |
| 205.4b | TESTABLE | Type-supertype independence — ATOM-205.4b-001 |
| 205.4c | TESTABLE | Basic supertype = basic land — ATOM-205.4c-001, -002 |
| 205.4d | TESTABLE | Legendary → legend rule — ATOM-205.4d-001 |
| 205.4e | TESTABLE | Legendary instant/sorcery casting restriction — ATOM-205.4e-001, -002 |
| 205.4f | TESTABLE | World supertype → world rule — ATOM-205.4f-001 |
| 205.4g | BOUNDARY-DEF | Snow supertype = snow permanent — ATOM-205.4g-001 |
| 205.4h | OUT-OF-SCOPE | Ongoing supertype (scheme cards) |
| 206.1 | PURE-DEF | Expansion symbol no effect |
| 206.2 | PURE-DEF | Rarity indicator |
| 206.3 | PURE-DEF | Old expansion symbol checks errata |
| 206.3a | DEFERRED | City in a Bottle — stretch goal |
| 206.3b | DEFERRED | Golgothian Sylex — stretch goal |
| 206.3c | DEFERRED | Apocalypse Chime — stretch goal |
| 206.4 | PURE-DEF | Deck construction printing rule |
| 206.5 | PURE-DEF | Expansion list reference |
| 207.1 | PURE-DEF | Text box structure |
| 207.2 | PURE-DEF | Italicized text no function |
| 207.2a | PURE-DEF | Reminder text |
| 207.2b | PURE-DEF | Flavor text |
| 207.2c | PURE-DEF | Ability words no meaning |
| 207.2d | PURE-DEF | Flavor words no meaning |
| 207.3 | PURE-DEF | Decorative icons no effect |
| 207.4 | OUT-OF-SCOPE | Chaos symbol (Planechase) |
| 207.5 | OUT-OF-SCOPE | Cryptic Spires |
| 208.1 | PURE-DEF | P/T structure |
| 208.2 | PURE-DEF | Star (*) P/T header |
| 208.2a | TESTABLE | CDA P/T in all zones — ATOM-208.2a-001, -002 |
| 208.2b | TESTABLE | Replacement-effect P/T (Corrupted Shapeshifter-style) — ATOM-208.2b-001, -002 |
| 208.3 | TESTABLE | Noncreature P/T rules — ATOM-208.3-001, -002 |
| 208.3a | TESTABLE | Dormant P/T effects on noncreatures — ATOM-208.3a-001 |
| 208.4 | PURE-DEF | "Base P/T" header |
| 208.4a | PURE-DEF | Set-P/T = base; other effects modify further |
| 208.4b | TESTABLE | "Base P/T" ignores L7c/L7d — ATOM-208.4b-001 |
| 208.5 | TESTABLE | No P/T value → 0 — ATOM-208.5-001 |
| 209.1 | TESTABLE | PW enters with loyalty counters — ATOM-209.1-001 |
| 209.2 | TESTABLE | Loyalty ability restrictions — ATOM-209.2-001, -002 |
| 210.1 | TESTABLE | Battle enters with defense counters — ATOM-210.1-001 |
| 211.1 | DEFERRED | Vanguard hand modifier — stretch goal |
| 212.1 | DEFERRED | Vanguard life modifier — stretch goal |
| 213.1 | PURE-DEF | Info below text box no effect |
| 213.1a | PURE-DEF | Collector numbers |
| 213.1b | PURE-DEF | Rarity indicator |
| 213.1c | PURE-DEF | Promotional info |
| 213.1d | PURE-DEF | Interchangeable name version info |
| 213.1e | PURE-DEF | Set/language codes |
| 213.1f | PURE-DEF | Illustration credit |
| 213.1g | PURE-DEF | Legal text |

---

## Gap Report

### Mechanisms in roadmap/implementation plan that SHOULD have Chapter 2 tests but don't map to a single CR sub-rule:

1. **Color identity (Commander — rule 903.4).** Color identity is computed from mana cost + color indicator + mana symbols in rules text. This is a Chapter 9 concept (Commander format) that uses Chapter 2 characteristics. No Chapter 2 rule defines color identity — it's defined in 903.4. Tests belong in the Commander/Phase 9 session.

2. **`get_mana_value()` for permanents on the battlefield.** Rule 202.3 defines MV for cards, but a permanent's MV on the battlefield (after potential copy effects changing its mana cost) needs to route through `compute_characteristics`. The L10 ticket covers this, but no single CR rule in Chapter 2 says "a permanent's MV on the battlefield uses its current (post-layer) characteristics." This is implicit in the layer system.

3. **Mana value of copies.** Rule 202.3b covers DFC back face copies (MV=0), but the general rule that a copy's MV uses the copiable mana cost (not the original card's) is rule 707.2, not Chapter 2. Tests for copy MV belong in the Phase 6 (copy effects) session.

4. **World rule SBA (704.5k).** Referenced by 205.4f but not implemented in any current ticket (T13–T16). Need: **NEW ticket or addition to T16** for world rule SBA. Low priority — very few World permanents exist in competitive Magic.

5. **`get_base_power`/`get_base_toughness` oracle functions.** Rule 208.4b requires querying P/T through L7a+L7b only (ignoring L7c modify and L7d switch). No current ticket implements this partial-layer query. Need: **NEW ticket** in Phase 5 Layers or Phase 8 when cards that check "base power" arrive (e.g., Skulk from T21b already uses power comparison, though it compares effective power, not base power).

6. **Subtype-to-card-type correlation tracking.** Rules 205.1a, 205.3c, 205.3d all require the engine to know which subtypes correlate to which card types. The L10 `SetLandType` mechanism handles land subtypes, but a general-purpose subtype correlation system (for "remove all creature subtypes" or "set creature type to X") needs to be explicit in the Layer 4 implementation. L10 partially covers this but the general case should be verified.

### NEW tickets identified in this session:

| ID | Description | Target Phase |
|----|-------------|-------------|
| NEW-S2-01 | General `has_same_name` utility function (broader than legend rule) | Phase 5 Pre-Work |
| NEW-S2-02 | Nameless object name comparison guard | Phase 9 |
| NEW-S2-03 | "Different names" group predicate utility function | Phase 8–9 |
| NEW-S2-04 | Singular "different name" predicate (incl. group variant) | Phase 8–9 |
| NEW-S2-05 | Token default mana cost = None | Phase 8 |
| NEW-S2-06 | DFC back face mana value uses front face cost | Phase 9 |
| NEW-S2-07 | Copy of DFC back face MV = 0 | Phase 9 |
| NEW-S2-08 | Subtype choice validation by card type category | Phase 8 |
| NEW-S2-09 | PlaneswalkerType enum | Phase 8 |
| NEW-S2-10 | SpellType enum | Phase 8–9 |
| NEW-S2-11 | DungeonType enum | Phase 8–9 |
| NEW-S2-12 | BattleType enum | Phase 8–9 |
| NEW-S2-13 | World rule SBA (704.5k) | Phase 5 Pre-Work or Phase 8 |
| NEW-S2-14 | Snow permanent classification oracle function | Phase 8 |
| NEW-S2-15 | Replacement-effect P/T cards show 0/0 off-battlefield | Phase 7 |
| NEW-S2-16 | Copiable values from ETB replacement effect choices | Phase 6–7 |
| NEW-S2-17 | `get_base_power`/`get_base_toughness` oracle (L7a+L7b only) | Phase 5 Layers or Phase 8 |
| NEW-S2-18 | Loyalty ability activation wiring (connects T19 to PW cards) | Phase 8 |
| NEW-S2-19 | Battle ETB defense counter initialization | Phase 8–9 |

> **Audit note:** Removed NEW-S2-04/05 (granted ability source binding, ability name substitution) — reclassified as PURE-DEF since the engine's ObjectId system handles these structurally. Added NEW-S2-01 (general `has_same_name`), NEW-S2-11 (DungeonType), NEW-S2-12 (BattleType), NEW-S2-16 (copiable values from ETB replacement), NEW-S2-19 (battle ETB defense counters).

### Test counts:

- **ATOM tests generated:** 71
- **COMP tests generated:** 3
- **Total tests:** 74

> **Audit delta:** +10 new ATOM tests (201.2b-002, 201.2c-002, 205.1b-003, 205.1b-004, 205.3p-001, 205.3q-001, 205.4b-002, 205.4c-001, 208.2b-002, 210.1-001). −3 ATOM tests removed (201.5-001, 201.5a-001, 201.5b-001 reclassified as PURE-DEF). Net ATOM change: +7. −2 COMP tests (COMP-205.1a+205.4b-001 promoted to ATOM-205.4b-002; COMP-202.3e+209.2-001 removed as redundant with ATOM-202.3e-002). Net COMP change: −2.
>
> **Previous count was 48 ATOM + 5 COMP = 53.** The pre-audit count was actually 64 ATOM + 5 COMP = 69 (the original "48" was an undercount). After audit: 71 ATOM + 3 COMP = 74.
