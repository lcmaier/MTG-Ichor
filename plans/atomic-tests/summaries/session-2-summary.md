# Session 2 Summary: Chapter 2 — Parts of a Card (Rules 200–213)

> Generated: 2026-04-02 | Post-audit condensed summary (pass 1 + pass 2)
> 71 ATOM tests | 3 COMP tests | 74 total | 19 new tickets | 0 META rules

---

## ATOM Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-201.2a-001 | 201.2a | Same-name comparison (Bile Blight pattern) | Phase 5-Pre | T14, NEW-S2-01 | |
| ATOM-201.2a-002 | 201.2a | Nameless objects never share a name | Phase 9 | NEW-S2-02 | |
| ATOM-201.2b-001 | 201.2b | "Different names" group predicate — nameless blocks | Phase 9 | NEW-S2-03 | |
| ATOM-201.2b-002 | 201.2b | "Different names" rejects duplicates in group | Phase 8 | NEW-S2-03 | |
| ATOM-201.2c-001 | 201.2c | Singular "different name" — nameless returns false | Phase 9 | NEW-S2-04 | |
| ATOM-201.2c-002 | 201.2c | Singular "different name" group — shared name fails | Phase 8 | NEW-S2-04 | |
| ATOM-201.2c-003 | 201.2c | Singular "different name" — nameless vs group w/ nameless | Phase 9 | NEW-S2-04 | |
| ATOM-202.1b-001 | 202.1b | No mana cost = unpayable, cast rejected | Phase 5-Pre | T18 | |
| ATOM-202.1b-002 | 202.1b | Token default mana cost = None | Phase 8 | NEW-S2-05 | |
| ATOM-202.2-001 | 202.2 | Color derived from mana cost ({2}{W} → white) | Phase 5 Layers | L10 | |
| ATOM-202.2-002 | 202.2 | Multicolor from mana cost ({2}{W}{B} → W+B) | Phase 5 Layers | L10 | |
| ATOM-202.2a-001 | 202.2a | Five colors enum completeness | ALREADY-IMPL | ALREADY-IMPLEMENTED | boundary |
| ATOM-202.2b-001 | 202.2b | Pure-generic cost → colorless | Phase 5 Layers | L10 | |
| ATOM-202.2b-002 | 202.2b | No mana cost + no color indicator → colorless | Phase 5 Layers | L10 | |
| ATOM-202.2c-001 | 202.2c | Two+ colored symbols → multicolored | Phase 5 Layers | L10 | |
| ATOM-202.2d-001 | 202.2d | Hybrid mana contributes all colors | Phase 5 Layers | L10 | |
| ATOM-202.2d-002 | 202.2d | Phyrexian mana contributes its color | Phase 5 Layers | L10 | |
| ATOM-202.2e-001 | 202.2e | Color indicator overrides absent mana cost | Phase 5-Pre + Layers | T05, L10 | |
| ATOM-202.3-001 | 202.3 | Mana value = total mana in cost | Phase 5 Layers | L10 | |
| ATOM-202.3a-001 | 202.3a | No mana cost → MV 0 | Phase 5 Layers | L10 | |
| ATOM-202.3a-002 | 202.3a | DFC back face MV uses front face cost | Phase 9 | NEW-S2-06 | |
| ATOM-202.3b-001 | 202.3b | Copy of DFC back face MV = 0 | Phase 9 + Phase 6 | NEW-S2-07 | |
| ATOM-202.3e-001 | 202.3e | X = 0 off-stack for MV | Phase 5 Layers | L10 | |
| ATOM-202.3e-002 | 202.3e | X = chosen value on stack for MV | Phase 5-Pre + Layers | T06, L10 | |
| ATOM-202.3f-001 | 202.3f | Hybrid MV uses largest component | Phase 5 Layers | L10 | |
| ATOM-202.3f-002 | 202.3f | MonoHybrid {2/C} contributes 2 to MV | Phase 5 Layers | L10 | |
| ATOM-202.3g-001 | 202.3g | Phyrexian mana contributes 1 each to MV | Phase 5 Layers | L10 | |
| ATOM-205.1a-001 | 205.1a | Type replacement — new type replaces existing | Phase 5 Layers | L10 | |
| ATOM-205.1a-002 | 205.1a | Instant/sorcery type retention during type set | Phase 5 Layers | L10 | |
| ATOM-205.1a-003 | 205.1a | Subtype set replacement scoped by card type | Phase 5 Layers | L10 | |
| ATOM-205.1a-004 | 205.1a | Subtype removal on type loss (correlated subtypes) | Phase 5 Layers | L10 | |
| ATOM-205.1a-005 | 205.1a | Counters/effects/damage persist across type change | Phase 5 Layers | L10 | |
| ATOM-205.1b-001 | 205.1b | "Artifact creature" retains all prior types | Phase 5 Layers | L10 | |
| ATOM-205.1b-002 | 205.1b | "Becomes a [type] artifact creature" replaces creature subtypes only | Phase 5 Layers | L10 | |
| ATOM-205.1b-003 | 205.1b | "Still a [type]" retention language | Phase 5 Layers | L10 | |
| ATOM-205.1b-004 | 205.1b | "In addition to its other types" retention | Phase 5 Layers | L10 | |
| ATOM-205.2a-001 | 205.2a | CardType enum completeness | ALREADY-IMPL | ALREADY-IMPLEMENTED | boundary |
| ATOM-205.2b-001 | 205.2b | Multi-type object satisfies criteria for any type | Phase 5 Layers | L13 | |
| ATOM-205.3c-001 | 205.3c | Subtype-to-card-type correlation query | Phase 5 Layers | L10 | |
| ATOM-205.3d-001 | 205.3d | Can't gain non-corresponding subtype | Phase 5 Layers | L10 | |
| ATOM-205.3e-001 | 205.3e | Choose subtype validation by card type | Phase 8 | NEW-S2-08 | |
| ATOM-205.3g-001 | 205.3g | Artifact types set membership | ALREADY-IMPL | ALREADY-IMPLEMENTED | boundary |
| ATOM-205.3h-001 | 205.3h | Enchantment types set membership | ALREADY-IMPL | ALREADY-IMPLEMENTED | boundary |
| ATOM-205.3i-001 | 205.3i | Basic land types set membership | ALREADY-IMPL | ALREADY-IMPLEMENTED | boundary |
| ATOM-205.3j-001 | 205.3j | Planeswalker types set membership | Phase 8 | NEW-S2-09 | boundary |
| ATOM-205.3k-001 | 205.3k | Spell types set membership | Phase 8–9 | NEW-S2-10 | boundary |
| ATOM-205.3m-001 | 205.3m | Creature types set membership | ALREADY-IMPL | ALREADY-IMPLEMENTED | boundary |
| ATOM-205.3p-001 | 205.3p | Dungeon types set membership | Phase 8–9 | NEW-S2-11 | boundary |
| ATOM-205.3q-001 | 205.3q | Battle types (Siege) set membership | Phase 8–9 | NEW-S2-12 | boundary |
| ATOM-205.4a-001 | 205.4a | Supertype enum completeness | ALREADY-IMPL | ALREADY-IMPLEMENTED | boundary |
| ATOM-205.4b-001 | 205.4b | Type-supertype independence (type change preserves supertypes) | Phase 5 Layers | L10 | |
| ATOM-205.4c-001 | 205.4c | Basic supertype → is basic land (positive) | Phase 5 Layers | L10 | |
| ATOM-205.4c-002 | 205.4c | No Basic supertype → nonbasic (even with basic land types) | Phase 5 Layers | L10 | |
| ATOM-205.4d-001 | 205.4d | Legendary supertype → legend rule SBA | Phase 5-Pre | T14 | |
| ATOM-205.4e-001 | 205.4e | Legendary instant/sorcery can't cast w/o legendary creature/PW | Phase 5-Pre | T18 | |
| ATOM-205.4e-002 | 205.4e | Legendary sorcery CAN cast with legendary creature | Phase 5-Pre | T18 | |
| ATOM-205.4f-001 | 205.4f | World supertype → world rule SBA | Phase 5-Pre or Phase 8 | NEW-S2-13 | |
| ATOM-205.4g-001 | 205.4g | Snow supertype = snow permanent classification | Phase 8 | NEW-S2-14 | boundary |
| ATOM-208.2a-001 | 208.2a | CDA P/T in all zones (Tarmogoyf) | Phase 5 Layers | L04, L17 | |
| ATOM-208.2a-002 | 208.2a | CDA undetermined value → 0 | Phase 5 Layers | L04 | |
| ATOM-208.2b-001 | 208.2b | Replacement-effect P/T = 0/0 off-battlefield | Phase 5 Layers + Phase 7 | L04, NEW-S2-15 | |
| ATOM-208.2b-002 | 208.2b | Replacement-effect P/T choice becomes copiable | Phase 6 + Phase 7 | NEW-S2-16 | |
| ATOM-208.3-001 | 208.3 | Noncreature permanent has no P/T (Vehicle not crewed) | Phase 5 Layers | L04 | |
| ATOM-208.3-002 | 208.3 | Noncreature off-battlefield has printed P/T (Vehicle in GY) | Phase 5 Layers | L04 | |
| ATOM-208.3a-001 | 208.3a | Dormant P/T effect activates when noncreature becomes creature | Phase 5 Layers | L04, L07 | |
| ATOM-208.4b-001 | 208.4b | "Base P/T" query ignores L7c/L7d | Phase 5 Layers | NEW-S2-17 | |
| ATOM-208.5-001 | 208.5 | No P/T value → fallback to 0 | Phase 5 Layers | L04 | |
| ATOM-209.1-001 | 209.1 | PW enters with loyalty counters = printed loyalty | Phase 5-Pre | T14 | |
| ATOM-209.2-001 | 209.2 | Loyalty ability sorcery-speed activation | Phase 5-Pre + Phase 8 | T19, NEW-S2-18 | |
| ATOM-209.2-002 | 209.2 | Loyalty ability once-per-turn restriction | Phase 5-Pre | T19 | |
| ATOM-210.1-001 | 210.1 | Battle enters with defense counters = printed defense | Phase 8–9 | NEW-S2-19 | |

---

## BOUNDARY-DEF Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-202.2a-001 | 202.2a | Five colors enum completeness | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-205.2a-001 | 205.2a | CardType enum completeness | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-205.3g-001 | 205.3g | Artifact types set membership | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-205.3h-001 | 205.3h | Enchantment types set membership | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-205.3i-001 | 205.3i | Basic land types set membership | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-205.3j-001 | 205.3j | Planeswalker types set membership | Phase 8 | NEW-S2-09 | |
| ATOM-205.3k-001 | 205.3k | Spell types set membership | Phase 8–9 | NEW-S2-10 | |
| ATOM-205.3m-001 | 205.3m | Creature types set membership | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-205.3p-001 | 205.3p | Dungeon types set membership | Phase 8–9 | NEW-S2-11 | |
| ATOM-205.3q-001 | 205.3q | Battle types (Siege) set membership | Phase 8–9 | NEW-S2-12 | |
| ATOM-205.4a-001 | 205.4a | Supertype enum completeness | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-205.4g-001 | 205.4g | Snow supertype = snow permanent | Phase 8 | NEW-S2-14 | |

---

## COMP Index

| ID | Rules | Summary | Composes | Phase | Ticket |
|----|-------|---------|----------|-------|--------|
| COMP-202.2e+202.2b-001 | 202.2e + 202.2b | Color indicator on costless card → green | ATOM-202.2e-001, ATOM-202.2b-002 | Phase 5-Pre + Layers | T05, L10 |
| COMP-205.4c+L10-001 | 205.4c + Blood Moon | Blood Moon doesn't add Basic supertype | ATOM-205.4c-002, L10 SetLandType | Phase 5 Layers | L10, L17 |
| COMP-208.3+205.1b-001 | 208.3 + 205.1b | Animated enchantment gains P/T (Opalescence) | ATOM-208.3-001, ATOM-205.1b-001 | Phase 5 Layers | L04, L19 |

---

## META Entries

No META rules were identified in Chapter 2. All rules are definitional, testable, or scoped out.

---

## Classification Summary Table

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
| 201.5 | PURE-DEF | Self-referential name = this object (engine uses ObjectId) |
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
| 202.3c | DEFERRED | Meld MV — stretch goal (Pioneer/Modern legal) |
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

## NEW Tickets List

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

---

## Gap Report

### Mechanisms needing tests but not mapping to a single CR sub-rule:

1. **Color identity (Commander — rule 903.4).** Uses Chapter 2 characteristics but defined in 903.4. Tests belong in Commander/Phase 9 session.
2. **`get_mana_value()` for permanents on battlefield.** Post-layer MV computation is implicit in the layer system; no single Chapter 2 rule.
3. **Mana value of copies.** General copy MV is rule 707.2 (Phase 6 session), not Chapter 2.
4. **World rule SBA (704.5k).** Referenced by 205.4f, not in any current ticket (T13–T16). Need NEW ticket or T16 addition.
5. **`get_base_power`/`get_base_toughness` oracle.** Rule 208.4b requires partial-layer query (L7a+L7b only). No current ticket.
6. **Subtype-to-card-type correlation tracking.** Rules 205.1a, 205.3c, 205.3d all require it. L10 `SetLandType` partially covers land subtypes; general case needs verification.

### Phase dependency heatmap:

| Phase | Test Specs Waiting |
|-------|--------------------|
| **Already Implemented** | ~7 (boundary enum checks: 202.2a, 205.2a, 205.3g, 205.3h, 205.3i, 205.3m, 205.4a) |
| **Phase 5-Pre** | ~9 (T05, T06, T14, T18, T19) |
| **Phase 5 Layers** | ~38 (L04, L07, L10, L13, L17, NEW-S2-17) |
| **Phase 6 (Copy)** | ~2 (NEW-S2-07, NEW-S2-16) |
| **Phase 7 (Replacement/Triggers)** | ~2 (NEW-S2-15, NEW-S2-16) |
| **Phase 8 (Effects/Cards)** | ~9 (NEW-S2-05, -08, -09, -10, -13, -14, -18 + 205.3e, 205.3j) |
| **Phase 8–9** | ~5 (NEW-S2-03, -04, -11, -12, -19) |
| **Phase 9** | ~4 (NEW-S2-02, -06, -07 + 201.2a-002) |

### Test counts:

- **ATOM tests:** 71
- **COMP tests:** 3
- **Total tests:** 74

---

## ALREADY-IMPLEMENTED List

202.2a, 205.2a, 205.3g, 205.3h, 205.3i, 205.3m, 205.4a

---

## OUT-OF-SCOPE List

| Rule(s) | Reason |
|---------|--------|
| 201.3, 201.3a, 201.3b | Interchangeable names (promotional/reprint cards) |
| 201.4g | Interchangeable names + choose a name |
| 201.6 | Secondary title bar (promotional card layout) |
| 205.3n | Planar types (Planechase supplemental) |
| 205.3r | No-subtype supplemental card types |
| 205.4h | Ongoing supertype (scheme cards) |
| 207.4 | Chaos symbol (Planechase) |
| 207.5 | Cryptic Spires (draft-matters single card) |

---

## DEFERRED List

| Rule(s) | Target Phase | Reason |
|---------|-------------|--------|
| 201.4 | Phase 8 (D18) | "Choose a card name" — needs Oracle DB |
| 201.4a | Phase 8 (D18) | Choose name with characteristics |
| 201.4b | Phase 9 (D4) | Split card name choice |
| 201.4c | Phase 9 | Flip card name |
| 201.4d | Phase 9 (D3) | DFC back face name |
| 201.4e | Phase 9 | Meld pair name |
| 201.4f | Phase 9 (D4) | Adventurer name |
| 202.3c | Phase 9 | Meld MV — stretch goal (Pioneer/Modern legal) |
| 202.3d | Phase 9 (D4) | Split card MV |
| 206.3a | Stretch goal | City in a Bottle (Vintage/Legacy) |
| 206.3b | Stretch goal | Golgothian Sylex (Vintage/Legacy) |
| 206.3c | Stretch goal | Apocalypse Chime (Vintage/Legacy) |
| 211.1 | Stretch goal | Vanguard hand modifier |
| 212.1 | Stretch goal | Vanguard life modifier |

---

*End of Session 2 Summary — Chapter 2: Parts of a Card (Rules 200–213)*
