# Session 9A — Condensed Summary (Rules 703.x–712.x)

> **Source:** `plans/atomic-tests/session-9a.md`
> **CR Source:** `MTG-Rules/LLM-Chapter-Splits/ch7-pt-4.txt` (rules 703–712)
> **Scope:** Turn-based actions, state-based actions, coin flipping, die rolling, copying objects, face-down spells/permanents, split cards, flip cards, leveler cards, double-faced cards
> **Audited:** 2026-04-09

---

## ATOM Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-703.3-001 | 703.3 | TBAs execute before SBAs and priority | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-703.4c-001 | 703.4c | Untap TBA — all permanents untap simultaneously | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-703.4c-002 | 703.4c | Winter Orb selective untap restriction | Phase 5 | NEW — Untap restriction effects | continuous-effects |
| ATOM-703.4d-001 | 703.4d | Draw step TBA — draw one card | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-703.4n-001 | 703.4n | Cleanup discard TBA — discard to max hand size | Pre-Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-703.4p-001 | 703.4p | Cleanup damage removal + EOT effects end simultaneously | Phase 1 / Phase 5 | ALREADY-IMPLEMENTED; T22 | |
| ATOM-703.4q-001 | 703.4q | Mana pool empties at end of step/phase | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-704.3-001 | 704.3 | SBA check-repeat loop — simultaneous destruction | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-704.3-002 | 704.3 | Cleanup step SBA shortcut — no priority if no SBAs | Phase 5-Pre | T16 | |
| ATOM-704.3-003 | 704.3 | Cleanup SBA re-loop with priority | Phase 5-Pre | T16 | |
| ATOM-704.4-001 | 704.4 | SBAs not checked mid-resolution | Phase 5 | L04 | layers |
| ATOM-704.5c-001 | 704.5c | 10+ poison counters → player loses | Phase 5-Pre | T16 | |
| ATOM-704.5c-002 | 704.5c | 9 poison counters → no loss (negative case) | Phase 5-Pre | T16 | |
| ATOM-704.5d-001 | 704.5d | Token in non-battlefield zone ceases to exist | Phase 5-Pre | T13 | |
| ATOM-704.5e-001 | 704.5e | Spell copy not on stack ceases to exist | Phase 6 | NEW — SBA spell/card copy | dependency |
| ATOM-704.5e-002 | 704.5e | Card copy in graveyard (cast-a-copy declined) ceases to exist | Phase 6 | NEW — SBA card copy | dependency |
| ATOM-704.5i-001 | 704.5i | Planeswalker 0 loyalty → graveyard | Phase 5-Pre | T14 | |
| ATOM-704.5i-002 | 704.5i | Planeswalker loyalty > 0 stays (negative case) | Phase 5-Pre | T14 | |
| ATOM-704.5j-001 | 704.5j | Legend rule — same name, same controller → choose one | Phase 5-Pre | T14 | |
| ATOM-704.5j-002 | 704.5j | Legend rule — different names, no SBA (negative) | Phase 5-Pre | T14 | |
| ATOM-704.5j-003 | 704.5j | Legend rule — same name, different controllers, no SBA (negative) | Phase 5-Pre | T14 | |
| ATOM-704.5k-001 | 704.5k | World rule — older world permanent destroyed | Phase 8 | NEW — World rule SBA | |
| ATOM-704.5k-002 | 704.5k | World rule — tied timestamps → both destroyed | Phase 8 | NEW — World rule SBA | |
| ATOM-704.5m-001 | 704.5m | Unattached Aura → graveyard | Phase 5-Pre | T15 | |
| ATOM-704.5m-002 | 704.5m | Aura host left → graveyard | Phase 5-Pre | T15 | |
| ATOM-704.5n-001 | 704.5n | Equipment on non-creature → unattach, stays on battlefield | Phase 5-Pre | T15 | |
| ATOM-704.5p-001 | 704.5p | Creature illegally attached → unattach | Phase 5-Pre | T15 | |
| ATOM-704.5q-001 | 704.5q | +1/+1 and -1/-1 counter annihilation (unequal) | Phase 5-Pre | T13 | |
| ATOM-704.5q-002 | 704.5q | Counter annihilation (equal counts) | Phase 5-Pre | T13 | |
| ATOM-704.5r-001 | 704.5r | Counter cap SBA — remove excess | Phase 8 | NEW — Counter cap SBA | |
| ATOM-704.6c-001 | 704.6c | Commander damage 21+ → player loses | Phase 9 | T16 | commander |
| ATOM-704.7-001 | 704.7 | SBA coalescing — single replacement for multiple same-result SBAs | Phase 6 | NEW — SBA coalescing | replacement-effects |
| ATOM-704.8-001 | 704.8 | Pre-SBA LKI snapshot for undying eligibility | Phase 5 | L18 | lki |
| ATOM-705.1-001 | 705.1 | Coin flip RNG produces heads or tails | Phase 8 | NEW — Coin flip system | |
| ATOM-705.2-001 | 705.2 | Call-based coin flip — correct call wins | Phase 8 | NEW — Coin flip system | |
| ATOM-705.2-002 | 705.2 | Call-based coin flip — wrong call loses | Phase 8 | NEW — Coin flip system | |
| ATOM-705.3-001 | 705.3 | Coin flip result override by effect | Phase 8 | NEW — Coin flip system | |
| ATOM-706.2-001 | 706.2 | Die roll natural vs. final result with modifiers | Phase 8 | NEW — Die roll system | |
| ATOM-706.2b-001 | 706.2b | Die roll modifier ordering (reroll first, then arithmetic) | Phase 8 | NEW — Die roll system | |
| ATOM-706.3a-001 | 706.3a | Results table lookup | Phase 8 | NEW — Die roll system | |
| ATOM-706.3c-001 | 706.3c | "Roll again" recursive die roll | Phase 8 | NEW — Die roll system | |
| ATOM-706.6-001 | 706.6 | Ignored roll — never happened, no triggers | Phase 8 | NEW — Die roll system | |
| ATOM-707.2-001 | 707.2 | Copy acquires copiable values only (not animation effects) | Phase 6 | D5 | copy |
| ATOM-707.2-002 | 707.2 | Copy of face-down creature gets face-down characteristics | Phase 9 | D5 + Phase 9 | copy, face-down |
| ATOM-707.2-003 | 707.2 | Counters on original not copied | Phase 6 | D5 | copy |
| ATOM-707.2b-001 | 707.2b | Changing original after copy doesn't update copy | Phase 6 | D5 | copy |
| ATOM-707.3-001 | 707.3 | Layered copy copiable value propagation | Phase 6 | D5 | copy |
| ATOM-707.4-001 | 707.4 | Re-copy on battlefield — no ETB/LTB, noncopy effects remain | Phase 6 | D5 | copy |
| ATOM-707.5-001 | 707.5 | "Enters as a copy" — copied replacement effects apply | Phase 6 | D5 | copy |
| ATOM-707.5-002 | 707.5 | "Enters as a copy" — copied ETB triggers fire | Phase 6 + Phase 7 | D5 + Phase 7 | copy, triggers |
| ATOM-707.6-001 | 707.6 | Copy resets "as enters" choices | Phase 6 | D5 | copy |
| ATOM-707.9a-001 | 707.9a | Copy-with-added-ability becomes copiable | Phase 6 | D5 | copy |
| ATOM-707.9b-001 | 707.9b | Copy-with-modified-characteristic becomes copiable | Phase 6 | D5 | copy |
| ATOM-707.9d-001 | 707.9d | P/T override strips CDA (Quicksilver Gargantuan) | Phase 6 | D5 | copy, cda |
| ATOM-707.9d-002 | 707.9d | "In addition to types" preserves type CDA | Phase 6 | D5 | copy, cda |
| ATOM-707.9e-001 | 707.9e | Additional-effect exception lost on re-copy | Phase 6 | D5 | copy |
| ATOM-707.9f-001 | 707.9f | Conditional exception negative — land not creature | Phase 6 | D5 | copy |
| ATOM-707.9f-002 | 707.9f | Conditional exception positive — creature gets counters+changeling | Phase 6 | D5 | copy |
| ATOM-707.10-001 | 707.10 | Spell copy inherits modes/targets/X, not "cast" | Phase 6 | D5 | copy |
| ATOM-707.10-002 | 707.10 | Spell copy references sacrificed object (Fling) | Phase 6 | D5 | copy |
| ATOM-707.10-003 | 707.10 | Spell copy can't reference mana spent (Dawnglow Infusion) | Phase 6 | D5 | copy |
| ATOM-707.10b-001 | 707.10b | Ability copy has same source — counter goes on source | Phase 7 | D5 + Phase 7 | copy, triggers |
| ATOM-707.10c-001 | 707.10c | Retarget copy — illegal targets may remain | Phase 6 | D5 | copy |
| ATOM-707.10d-001 | 707.10d | Copy for each legal target — one per target | Phase 6 | D5 | copy |
| ATOM-707.10e-001 | 707.10e | Copy with specified single target | Phase 6 | D5 | copy |
| ATOM-707.10e-002 | 707.10e | Replacement causes multiple targets — controller picks one | Phase 6 | D5 | copy |
| ATOM-707.12-001 | 707.12 | "Cast a copy" follows 601.2a–h, triggers fire | Phase 6 | D5 | copy |
| ATOM-708.2-001 | 708.2 | Face-down characteristic suppression (morph) | Phase 9 | NEW — Face-down system | face-down |
| ATOM-708.2a-001 | 708.2a | Default face-down characteristics (Ixidron) | Phase 9 | NEW — Face-down system | face-down |
| ATOM-708.2b-001 | 708.2b | Face-down can't be turned face down again | Phase 9 | NEW — Face-down system | face-down |
| ATOM-708.3-001 | 708.3 | Face-down enter — ETB suppressed | Phase 9 | NEW — Face-down system | face-down |
| ATOM-708.4-001 | 708.4 | Face-down casting — CMC 0 on stack | Phase 9 | NEW — Face-down system | face-down |
| ATOM-708.8-001 | 708.8 | Face-up revert — effects persist, no ETB | Phase 9 | NEW — Face-down system | face-down |
| ATOM-708.10-001 | 708.10 | Face-down copy — shows copied chars on face-up | Phase 9 + Phase 6 | D5 + Phase 9 | face-down, copy |
| ATOM-708.11-001 | 708.11 | "As this is turned face up" applied during transition | Phase 9 | NEW — Face-down system | face-down |
| ATOM-709.3-001 | 709.3 | Split card half selection on cast | Phase 9 | NEW — Split card system | split |
| ATOM-709.3a-001 | 709.3a | Only chosen half evaluated for castability | Phase 9 | NEW — Split card system | split |
| ATOM-709.3b-001 | 709.3b | On stack only cast half's characteristics exist | Phase 9 | NEW — Split card system | split |
| ATOM-709.4-001 | 709.4 | Combined characteristics in non-stack zones | Phase 9 | NEW — Split card system | split |
| ATOM-709.4a-001 | 709.4a | Split card name matching (Pithing Needle) | Phase 9 | NEW — Multi-name support | split |
| ATOM-709.4b-001 | 709.4b | Combined mana cost, colors, MV | Phase 9 | NEW — Split card system | split |
| ATOM-710.2-001 | 710.2 | Flip card characteristic switching on flip | Phase 9 | NEW — Flip card system | flip |
| ATOM-710.2-002 | 710.2 | Flip card — only normal chars in non-battlefield zones | Phase 9 | NEW — Flip card system | flip |
| ATOM-710.4-001 | 710.4 | Flip is one-way; zone change resets status | Phase 9 | NEW — Flip card system | flip |
| ATOM-711.2-001 | 711.2 | Level counter conditional P/T + abilities (mid-range) | Phase 9 | NEW — Leveler system | leveler |
| ATOM-711.2-002 | 711.2 | Level counter max range — all abilities active | Phase 9 | NEW — Leveler system | leveler |
| ATOM-711.4-001 | 711.4 | Level up activation always available | Phase 9 | NEW — Leveler system | leveler |
| ATOM-711.5-001 | 711.5 | Below first level range → base P/T | Phase 9 | NEW — Leveler system | leveler |
| ATOM-711.6-001 | 711.6 | Leveler in non-battlefield zone uses base P/T | Phase 9 | NEW — Leveler system | leveler |
| ATOM-712.4a-001 | 712.4a | Meld — exile both, enter as single permanent | Phase 9 | NEW — Meld system | dfc, meld |
| ATOM-712.4c-001 | 712.4c | Meld cards can't transform/convert | Phase 9 | NEW — Meld system | dfc, meld |
| ATOM-712.8-001 | 712.8 | DFC per-face characteristic isolation | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.8a-001 | 712.8a | DFC in non-battlefield zone → front face only | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.8c-001 | 712.8c | Nonmodal DFC cast transformed — MV from front face | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.8e-001 | 712.8e | Back-face-up DFC MV from front face (survives destruction) | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.8e-002 | 712.8e | Copy of back face has MV = 0 | Phase 9 | NEW — DFC system | dfc, copy |
| ATOM-712.8g-001 | 712.8g | Melded permanent MV = sum of front faces | Phase 9 | NEW — Meld system | dfc, meld |
| ATOM-712.8g-002 | 712.8g | Copy of melded permanent has MV = 0 | Phase 9 | NEW — Meld system | dfc, meld, copy |
| ATOM-712.9-001 | 712.9 | Non-DFC can't transform (Clone negative case) | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.9-002 | 712.9 | DFC card that is a copy can still transform | Phase 9 | NEW — DFC system | dfc, copy |
| ATOM-712.10-001 | 712.10 | Transform into instant/sorcery face → nothing | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.11-001 | 712.11 | DFC spell default front face up on stack | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.11a-001 | 712.11a | Cast "transformed" → back face up on stack | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.11b-001 | 712.11b | Modal DFC face selection on cast | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.12-001 | 712.12 | Modal DFC land play — choose land face, special action | Phase 9 | NEW — Modal DFC land play | dfc |
| ATOM-712.12-002 | 712.12 | Modal DFC land play doesn't use stack | Phase 9 | NEW — Modal DFC land play | dfc |
| ATOM-712.13-001 | 712.13 | DFC spell resolves with same face as on stack | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.13a-001 | 712.13a | DFC enter transformed — sorcery back face → graveyard | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.13a-002 | 712.13a | Stress test: Clone + Mystic Reflection + Siege sorcery back | Phase 9 | NEW — DFC system | dfc, copy, stress |
| ATOM-712.14-001 | 712.14 | DFC from non-stack zone → front face up default | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.14a-001 | 712.14a | DFC "transformed" entry; non-DFC stays in zone | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.14b-001 | 712.14b | Modal DFC non-permanent front face → stays in zone | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.16-001 | 712.16 | DFC can't be turned face down | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.18-001 | 712.18 | Transform doesn't create new object — effects persist | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.20-001 | 712.20 | "As this transforms" applied during transform | Phase 9 | NEW — DFC system | dfc |
| ATOM-712.21-001 | 712.21 | Meld death — 1 permanent leaves, 2 cards to zone | Phase 9 | NEW — Meld system | dfc, meld |
| ATOM-712.21a-001 | 712.21a | Meld to graveyard/library — owner orders 2 cards | Phase 9 | NEW — Meld system | dfc, meld |
| ATOM-712.21c-001 | 712.21c | Meld tracking — effect finds both cards | Phase 9 | NEW — Meld system | dfc, meld |
| ATOM-712.21d-001 | 712.21d | Meld replacement non-splitting — one effect for both cards | Phase 9 | NEW — Meld system | dfc, meld, replacement-effects |
| ATOM-712.21e-001 | 712.21e | Meld = 1 object moved, 2 cards moved | Phase 9 | NEW — Meld system | dfc, meld |

## BOUNDARY-DEF Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| BOUNDARY-707.2c-001 | 707.2c | Static ability copy locks copiable values at application time | Phase 6 | D5 | copy |
| BOUNDARY-707.7-001 | 707.7 | Copied linked abilities remain linked, can't cross-link | Phase 7 | T07-linked | copy, linked-abilities |
| BOUNDARY-707.9g-001 | 707.9g | Copy-effect linked trigger invalidated by subsequent copy | Phase 7 | D5 + Phase 7 | copy, triggers |
| BOUNDARY-708.12-001 | 708.12 | Face-down reveal bypasses layer system | Phase 9 | NEW — Face-down system | face-down, layers |
| BOUNDARY-709.3c-001 | 709.3c | Copy of split card vs copy of split spell on stack | Phase 9 | NEW — Split card copy zone-awareness | split, copy |
| BOUNDARY-712.4b-001 | 712.4b | Meld back face only determines chars when melded on battlefield | Phase 9 | NEW — Meld system | dfc, meld |
| BOUNDARY-712.11d-001 | 712.11d | Front-face ability enabling transformed cast evaluated for castability | Phase 9 | NEW — DFC system | dfc |
| BOUNDARY-712.21b-001 | 712.21b | Meld exile — player determines relative timestamps of 2 cards | Phase 9 | NEW — Meld system | dfc, meld |

## COMP Index

| ID | Rules Composed | Summary | ATOMs Required |
|----|---------------|---------|----------------|
| COMP-9A-001 | 704.5q + 704.5f + 704.8 | SBA cascade: counter annihilation + lethal damage + LKI | ATOM-704.5q-001, ATOM-704.8-001 |
| COMP-9A-002 | 707.5 + 707.2 + 707.6 | Copy entering with ETB replacement + choice reset | ATOM-707.5-001, ATOM-707.2-003, ATOM-707.6-001 |
| COMP-9A-003 | 708.10 + 708.8 + 707.3 | Face-down copy → face-up shows copied chars | ATOM-708.10-001, ATOM-708.8-001 |
| COMP-9A-004 | 712.9 + 707.4 + 712.18 | DFC transform through copy effect | ATOM-712.9-002, ATOM-707.4-001, ATOM-712.18-001 |
| COMP-9A-005 | 712.21 + 712.21d + 712.21e | Meld death + replacement non-splitting + card counting | ATOM-712.21-001, ATOM-712.21d-001, ATOM-712.21e-001 |
| COMP-9A-006 | 707.9d + 707.2 | CDA stripping vs "in addition to types" preservation | ATOM-707.9d-001, ATOM-707.9d-002 |
| COMP-9A-007 | 709.3b + 709.4 + 709.4b | Split card characteristics shift between zones and stack | ATOM-709.3b-001, ATOM-709.4-001, ATOM-709.4b-001 |

## Classification Summary

| Classification | Count | Rules |
|---------------|-------|-------|
| **TESTABLE** | ~78 | 703.3, 703.4c, 703.4d, 703.4n, 703.4p, 703.4q, 704.3, 704.4, 704.5c, 704.5d, 704.5e, 704.5i, 704.5j, 704.5k, 704.5m, 704.5n, 704.5p, 704.5q, 704.5r, 704.7, 704.8, 705.1, 705.2, 705.3, 706.2, 706.2b, 706.3a, 706.3c, 706.6, 707.2, 707.2b, 707.3, 707.4, 707.5, 707.6, 707.9a, 707.9b, 707.9d, 707.9e, 707.9f, 707.10, 707.10b, 707.10c, 707.10d, 707.10e, 707.12, 708.2, 708.2a, 708.2b, 708.3, 708.4, 708.8, 708.10, 708.11, 709.3, 709.3a, 709.3b, 709.4, 709.4a, 709.4b, 710.2, 710.4, 711.2, 711.4, 711.5, 711.6, 712.4a, 712.4c, 712.8, 712.8a, 712.8c, 712.8e, 712.8g, 712.9, 712.10, 712.11, 712.11a, 712.11b, 712.12, 712.13, 712.13a, 712.14, 712.14a, 712.14b, 712.16, 712.18, 712.20, 712.21, 712.21a, 712.21c, 712.21d, 712.21e |
| **BOUNDARY-DEF** | 9 | 707.2c, 707.7, 707.9g, 708.12, 709.3c, 712.4b, 712.11d, 712.21b |
| **PURE-DEF** | ~62 | 703.1, 703.1a, 703.2, 703.4, 705 (physical coin), 706.1, 706.1a, 706.1b, 706.2a, 706.3, 706.3b, 706.4, 707.1, 707.2a, 707.9, 707.9c, 707.10a, 707.11, 708.1, 708.5, 708.6, 708.7, 708.9, 709.1, 709.2, 709.4c, 710.1–710.1c, 710.3, 710.5, 711.1, 711.3, 711.7, 712.1, 712.2–712.2c, 712.3–712.3c, 712.4, 712.5–712.5g, 712.6, 712.7, 712.8b, 712.8d, 712.8f, 712.11c, 712.14c, 712.17, 712.19 |
| **OUT-OF-SCOPE** | 13 | (see below) |
| **DEFERRED** | 21 | (see below) |
| **ALREADY-IMPLEMENTED** | 10 | (see below) |

**Totals:** 87 ATOM tests, 9 BOUNDARY tests, 7 COMP tests, ~193 classified sub-rules

## NEW Tickets

| Ticket | Description | Phase | Rules |
|--------|-------------|-------|-------|
| NEW — Untap restriction effects | Winter Orb-style selective untap via continuous effects | Phase 5 | 703.4c |
| NEW — Coin flip system | RNG, win/lose, result override | Phase 8 | 705.1–705.3 |
| NEW — Die roll system | dN, modifiers, results table, roll again, ignore | Phase 8 | 706.2–706.6 |
| NEW — SBA spell/card copy cease-to-exist | 704.5e implementation | Phase 6 | 704.5e |
| NEW — Planeswalker 0-loyalty SBA | Extend SBA loop | Phase 5-Pre | 704.5i |
| NEW — Legend rule SBA | DecisionProvider for legend choice | Phase 5-Pre | 704.5j |
| NEW — World rule SBA | Timestamp-based world rule | Phase 8 | 704.5k |
| NEW — Counter cap SBA | Remove excess counters | Phase 8 | 704.5r |
| NEW — SBA coalescing for replacements | 704.7 single replacement for multiple same-result SBAs | Phase 6 | 704.7 |
| NEW — Face-down system | Characteristic overlay, morph, face-up revert, reveal | Phase 9 | 708.x |
| NEW — Split card system | Two halves, combined characteristics, half-cast | Phase 9 | 709.1–709.4b |
| NEW — Split card copy zone-awareness | Copy of split card vs copy of split spell on stack | Phase 9 | 709.3c + 709.3b |
| NEW — Rooms system | Shared type line, lock/unlock | Phase 9+ | 709.5–709.5j |
| NEW — Flip card system | One-way flip, characteristic switching | Phase 9 | 710.x |
| NEW — Leveler card system | Level counters, conditional P/T | Phase 9 | 711.x |
| NEW — DFC system | Transform, convert, modal, meld | Phase 9 | 712.x |

## Gap Report

| Gap | Description | Suggested Ticket |
|-----|-------------|-----------------|
| GAP-1 | Coin flip infrastructure — RNG, win/lose, result override | NEW — Coin flip system (705.x) |
| GAP-2 | Die roll infrastructure — dN, modifiers, results table, roll again, ignore | NEW — Die roll system (706.x) |
| GAP-3 | Copy system (D5) — copiable values, CDA stripping, copy-with-exception, spell copy | D5 (existing, substantial) |
| GAP-4 | Face-down system — overlay, morph cast, face-up revert, ETB suppression, copy interaction | NEW — Face-down system (708.x) |
| GAP-5 | Split card data model — two-name, combined cost, half-cast, stack suppression | NEW — Split card system (709.x) |
| GAP-6 | Flip card system — one-way flip, characteristic switching, zone-based selection | NEW — Flip card system (710.x) |
| GAP-7 | Leveler card system — level counters, conditional P/T/abilities, level up | NEW — Leveler system (711.x) |
| GAP-8 | DFC system (largest gap) — nonmodal, modal, meld, zone-change splitting | NEW — DFC system (712.x) |
| GAP-9 | Room mechanics (Duskmourn) — shared type line, lock/unlock, designations | NEW — Rooms system (709.5.x) |
| GAP-10 | SBA for copies ceasing to exist requires D5 | D5 prerequisite |
| GAP-11 | Planeswalker 0-loyalty + legend rule SBAs need DecisionProvider hooks | T14 |
| GAP-12 | Counter cap + world rule SBAs — straightforward additions | NEW tickets |
| GAP-13 | SBA coalescing requires replacement effect system | Phase 6 prerequisite |
| GAP-14 | Pre-SBA LKI snapshot requires LKI system | L18 prerequisite |

## ALREADY-IMPLEMENTED

703.4i, 703.4j, 703.4k, 703.4m, 704.5a, 704.5b, 704.5f, 704.5g, 704.5h

## OUT-OF-SCOPE

| Rule | Reason |
|------|--------|
| 703.4e | Archenemy scheme action |
| 703.4g | Attractions (Un-set mechanic) |
| 703.4h | 2HG / multiplayer attack routing — not supporting team formats |
| 704.5t | Dungeon venture marker (D&D crossover) |
| 704.5u | Space sculptor sector (Unfinity) |
| 704.6a | Two-Headed Giant team life SBA |
| 704.6b | Two-Headed Giant team poison SBA |
| 704.6e | Archenemy scheme SBA |
| 704.6f | Planechase phenomenon SBA |
| 706.5 | Celebr-8000 (single-card rule) |
| 706.7 | Planechase planar die |
| 706.8–706.8c | Centaur of Attention stored results (single-card rule) |

## DEFERRED

| Rule | Phase | Reason |
|------|-------|--------|
| 703.4a | Phase 9 | Phasing TBA during untap |
| 703.4b | Phase 9 | Day/night designation check |
| 703.4f | Phase 8–9 | Saga lore counter TBA |
| 704.5s | Phase 8–9 | Saga sacrifice SBA |
| 704.5v–704.5x | Phase 8–9 | Battle SBAs (defense 0, no protector, siege) |
| 704.5y | Phase 8–9 | Roles — keep most recent |
| 704.5z | Phase 8–9 | Speed designation SBA (Standard-legal but niche) |
| 704.6c | Phase 9 | Commander damage 21+ SBA |
| 704.6d | Phase 9 | Commander zone return option |
| 707.8 | Phase 9 | DFC copy uses face-up copiable values |
| 707.8a | Phase 9 | Token copy of DFC → double-faced token |
| 707.10f | Phase 6+ | Copy of permanent spell → token permanent |
| 707.10g | Phase 9 | Copy of DFC spell → double-faced token |
| 707.13 | Phase 9 | Garth One-Eye — copy from Oracle by name |
| 707.14 | Phase 9 | Magar — copy with noted name from graveyard LKI |
| 709.4d | Phase 9 | Fuse — combined halves on stack |
| 709.5–709.5j | Phase 9+ | Rooms mechanic (Duskmourn) — all sub-rules |
| 712.15–712.15a | Phase 9 | DFC face-down interaction |

---

*Generated from audited session-9a.md on 2026-04-09.*
