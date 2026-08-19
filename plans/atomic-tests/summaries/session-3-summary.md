# Session 3 Summary: Chapter 3 — Card Types (Rules 300–315)

> Generated: 2026-04-04
> Source: `plans/atomic-tests/session-3.md` (post-audit)
> CR Source: Chapter 3 — Card Types

---

## ATOM Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-300.1-001 | 300.1 | CardType enum has exactly 15 types | Phase 5-Pre | T07 | boundary, enum |
| ATOM-300.2a-001 | 300.2a | Land+other type: cast rejected, play-as-land only | Phase 8 | D25 | land, casting |
| ATOM-300.2a-002 | 300.2a | Land+other type CAN be played as land | Phase 8 | D25 | land, positive |
| ATOM-301.5-001 | 301.5 | Equipment attaches to creature (legal) | Phase 5-Pre | T04, T15 | equipment, attachment |
| ATOM-301.5-002 | 301.5 | Equipment can't attach to non-creature | Phase 5-Pre | T15 | equipment, attachment |
| ATOM-301.5b-001 | 301.5b | Equipment ETB unattached | Phase 5-Pre | T04 | equipment, etb |
| ATOM-301.5b-002 | 301.5b | Equip ability attaches Equipment to creature | Phase 8 | NEW | equipment, equip |
| ATOM-301.5b-003 | 301.5b | Effect attaching Equipment to illegal target: no move | Phase 8 | NEW | equipment, guard |
| ATOM-301.5c-001 | 301.5c | Creature-Equipment can't equip (no reconfigure) | Phase 8 | NEW | equipment, restriction |
| ATOM-301.5c-002 | 301.5c | Equipment loses subtype → SBA unattaches | Phase 5-Pre | T15 | equipment, sba |
| ATOM-301.5c-003 | 301.5c | Equipment can't equip itself | Phase 8 | NEW | equipment, restriction |
| ATOM-301.5c-004 | 301.5c | Equipment on destroyed creature → unattached, stays on BF (SBA) | Phase 5-Pre | T15, T04 | equipment, sba |
| ATOM-301.5c-005 | 301.5c | Equipment can't equip >1 creature; equip moves it | Phase 8 | NEW | equipment, move |
| ATOM-301.5d-001 | 301.5d | Control change of creature doesn't change Equipment controller | Phase 5 Layers | L11 | equipment, control |
| ATOM-301.5d-002 | 301.5d | Only Equipment controller activates its abilities | Phase 8 | NEW | equipment, activation |
| ATOM-301.5d-003 | 301.5d | Equipment-granted ability activated by creature's controller | Phase 8 | NEW | equipment, granted-ability |
| ATOM-301.5e-001 | 301.5e | Equipment ETB via effect attached to illegal target → unattached | Phase 8 | NEW | equipment, etb, guard |
| ATOM-301.6-001 | 301.6 | Fortification attaches to land, not non-land | Phase 8–9 | NEW | fortification, attachment |
| ATOM-301.7a-001 | 301.7a | Vehicle (non-creature) has no P/T characteristics | Phase 8 | NEW | vehicle, characteristic |
| ATOM-301.7a-002 | 301.7a | Vehicle (creature) has printed P/T | Phase 8 | NEW | vehicle, characteristic |
| ATOM-301.7b-001 | 301.7b | Vehicle becoming creature gets P/T + anthem applies | Phase 8 | NEW | vehicle, layers |
| ATOM-302.4-001 | 302.4 | Non-creature has no P/T | Phase 5 Layers | L04 | creature, characteristic |
| ATOM-302.4-002 | 302.4 | Creature has P/T (positive test) | Phase 5 Layers | L04 | creature, characteristic, positive |
| ATOM-302.4c-001 | 302.4c | P/T = printed + continuous effects (Giant Growth) | Phase 5 Layers | L07, L08 | creature, pt, layers |
| ATOM-303.4-001 | 303.4 | Aura ETB attached to target creature | Phase 5-Pre | T15b | aura, etb |
| ATOM-303.4-002 | 303.4 | Aura with "Enchant player" attaches to player | Phase 8 | NEW | aura, player-attach |
| ATOM-303.4a-001 | 303.4a | Aura spell requires target; no legal target → can't cast | Phase 5-Pre | T15b | aura, targeting |
| ATOM-303.4c-001 | 303.4c | Aura on illegal object (type removed) → graveyard SBA | Phase 5-Pre | T15 | aura, sba |
| ATOM-303.4c-002 | 303.4c | Aura host destroyed → graveyard SBA | Phase 5-Pre | T15, T04 | aura, sba |
| ATOM-303.4c-003 | 303.4c | Aura on player who leaves game → graveyard SBA | Phase 9 | NEW | aura, multiplayer, sba |
| ATOM-303.4d-001 | 303.4d | Self-enchanting Aura → graveyard SBA | Phase 5-Pre | T15 | aura, sba |
| ATOM-303.4d-002 | 303.4d | Aura+Creature → unattach then graveyard SBA | Phase 8 | T15 | aura, sba, type-change |
| ATOM-303.4e-001 | 303.4e | Control change of enchanted object doesn't change Aura controller | Phase 5 Layers | L11 | aura, control |
| ATOM-303.4e-002 | 303.4e | Aura-granted ability activated by enchanted object's controller | Phase 8 | NEW | aura, granted-ability |
| ATOM-303.4e-003 | 303.4e | Pacifism cast on opponent's creature: caster = Aura controller | Phase 5-Pre | T15b | aura, control, positive |
| ATOM-303.4f-001 | 303.4f | Non-stack Aura ETB: controller chooses (hexproof OK) | Phase 5-Pre | T15b | aura, etb, hexproof |
| ATOM-303.4g-001 | 303.4g | Aura from non-stack, no legal target → stays in zone | Phase 5-Pre / Phase 8 | T15b | aura, guard |
| ATOM-303.4g-002 | 303.4g | Aura from stack, target dies → fizzle to graveyard | ALREADY-IMPL | ALREADY-IMPL | aura, fizzle, regression |
| ATOM-303.4g-003 | 303.4g | Aura token, no legal target → not created | Phase 8 | NEW | aura, token, guard |
| ATOM-303.4h-001 | 303.4h | Non-attachment permanent entering "attached" → unattached | Phase 8 | NEW | attachment, guard |
| ATOM-303.4i-001 | 303.4i | Aura from non-stack attached to illegal host → stays in zone | Phase 8 | NEW | aura, guard |
| ATOM-303.4i-002 | 303.4i | Aura from stack attached to illegal host → graveyard | Phase 8 | NEW | aura, guard |
| ATOM-303.4i-003 | 303.4i | Aura token attached to illegal host → not created | Phase 8 | NEW | aura, token, guard |
| ATOM-303.4j-001 | 303.4j | Move Aura to illegal enchant target → doesn't move | Phase 8 | NEW | aura, re-attach, guard |
| ATOM-303.7a-001 | 303.7a | Multiple Roles from same controller: keep newest (SBA) | Phase 8 | NEW | role, sba |
| ATOM-303.7a-002 | 303.7a | Roles from different controllers coexist | Phase 8 | NEW | role, sba, positive |
| ATOM-304.4-001 | 304.4 | Instant can't enter battlefield → stays in previous zone | Phase 5-Pre | T21a | instant, zone-guard |
| ATOM-304.5-001 | 304.5 | "As an instant" = priority only; no instant card needed | Phase 5-Pre | T19 | timing, instant-speed |
| ATOM-304.5-002 | 304.5 | "Can't cast instants" doesn't block "as an instant" abilities | Phase 5 Layers / Phase 8 | NEW | timing, restriction |
| ATOM-305.2-001 | 305.2 | Default 1 land/turn; second rejected | Phase 5-Pre | T22 | land, limit |
| ATOM-305.2-002 | 305.2 | Continuous effect increases land limit; second land OK | Phase 5 Layers | L15 | land, limit, continuous |
| ATOM-305.2a-001 | 305.2a | "Put" land doesn't count; player can still play | Phase 5-Pre / Phase 5 Layers | T22 | land, counter |
| ATOM-305.2b-001 | 305.2b | At limit, effect instruction to play land is ignored | Phase 5-Pre / Phase 8 | T22, NEW | land, hard-cap |
| ATOM-305.3-001 | 305.3 | Can't play land on opponent's turn | ALREADY-IMPL | ALREADY-IMPL | land, timing |
| ATOM-305.4-001 | 305.4 | "Put land onto BF" ≠ "play"; counter not incremented | Phase 8 | NEW | land, put-vs-play |
| ATOM-305.6-001 | 305.6 | Basic land type → intrinsic mana ability | ALREADY-IMPL | ALREADY-IMPL | land, mana |
| ATOM-305.6-002 | 305.6 | Non-basic gains basic land type (Urborg) → gains mana ability | Phase 5 Layers | L17 | land, urborg, mana |
| ATOM-305.6-003 | 305.6 | Non-basic land types (Desert, Gate) don't grant mana ability | Phase 5 Layers | L17 | land, boundary |
| ATOM-305.7-001 | 305.7 | Set land subtype to basic → removes old land types | Phase 5 Layers | L17 | land, blood-moon |
| ATOM-305.7-002 | 305.7 | Set to basic type → loses rules-text abilities, gains intrinsic mana | Phase 5 Layers | L17 | land, blood-moon |
| ATOM-305.7-003 | 305.7 | Set to basic type → keeps abilities granted by other effects | Phase 5 Layers | L17 | land, blood-moon, granted |
| ATOM-305.7-004 | 305.7 | Set subtypes doesn't change card types or supertypes | Phase 5 Layers | L17 | land, blood-moon, supertype |
| ATOM-305.7-005 | 305.7 | "In addition to" keeps old types + adds new types/mana | Phase 5 Layers | L17, NEW | land, additive |
| ATOM-305.8-001 | 305.8 | Land with basic type but no "Basic" supertype = nonbasic | Phase 5 Layers | L17 | land, supertype, boundary |
| ATOM-305.8-002 | 305.8 | Land with "Basic" supertype IS basic (positive) | ALREADY-IMPL | ALREADY-IMPL | land, supertype, positive |
| ATOM-306.5-001 | 306.5 | Loyalty is characteristic only planeswalkers have | Phase 5-Pre | T14 | planeswalker, boundary |
| ATOM-306.5a-001 | 306.5a | Non-BF PW loyalty = printed number | Phase 5-Pre / Phase 5 Layers | T14, L04 | planeswalker, loyalty |
| ATOM-306.5b-001 | 306.5b | PW ETB with loyalty counters = printed loyalty | Phase 5-Pre | T14 | planeswalker, etb, counters |
| ATOM-306.5c-001 | 306.5c | BF PW loyalty = loyalty counter count | Phase 5-Pre | T14 | planeswalker, loyalty, counters |
| ATOM-306.5d-001 | 306.5d | Loyalty ability activated at sorcery speed | Phase 5-Pre | T19 | planeswalker, activation, timing |
| ATOM-306.5d-002 | 306.5d | Only one loyalty ability per PW per turn | Phase 5-Pre | T19 | planeswalker, activation, once |
| ATOM-306.5d-003 | 306.5d | Loyalty ability rejected on opponent's turn / non-empty stack | Phase 5-Pre | T19 | planeswalker, activation, timing |
| ATOM-306.6-001 | 306.6 | Planeswalker can be attacked | Phase 5-Pre / Phase 8 | NEW | planeswalker, combat |
| ATOM-306.8-001 | 306.8 | Damage to PW removes loyalty counters | Phase 5-Pre | T21c | planeswalker, damage |
| ATOM-306.8-002 | 306.8 | Excess damage to PW absorbed (no overflow) | Phase 5-Pre | T21c | planeswalker, damage, boundary |
| ATOM-306.9-001 | 306.9 | PW with 0 loyalty → graveyard SBA | Phase 5-Pre | T14 | planeswalker, sba |
| ATOM-306.9-002 | 306.9 | PW with >0 loyalty stays (positive) | Phase 5-Pre | T14 | planeswalker, sba, positive |
| ATOM-307.4-001 | 307.4 | Sorcery can't enter battlefield → stays in previous zone | Phase 5-Pre | T21a | sorcery, zone-guard |
| ATOM-307.5-001 | 307.5 | "As a sorcery" = priority + main + stack empty; no sorcery needed | Phase 5-Pre | T19 | timing, sorcery-speed |
| ATOM-307.5-002 | 307.5 | "As a sorcery" rejected on opponent's turn | Phase 5-Pre | T19 | timing, sorcery-speed |
| ATOM-307.5-003 | 307.5 | "Can't cast sorceries" doesn't block "as a sorcery" abilities | Phase 5 Layers / Phase 8 | NEW | timing, restriction |
| ATOM-307.5-004 | 307.5 | Teferi-style: opponent's instant cast legal at sorcery speed | Phase 5 Layers / Phase 8 | NEW | timing, teferi, positive |
| ATOM-307.5-005 | 307.5 | Teferi-style: opponent can't cast at instant speed | Phase 5 Layers / Phase 8 | NEW | timing, teferi, restriction |
| ATOM-307.5a-001 | 307.5a | "Couldn't cast as sorcery" = retroactive timing check | Phase 8 | NEW | timing, necromancy |
| ATOM-308.1-001 | 308.1 | Kindred card follows other type's casting rules | Phase 8 | NEW | kindred, casting |

**Total ATOM tests: 87**

---

## BOUNDARY-DEF Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-300.1-001 | 300.1 | CardType enum completeness (15 types) | Phase 5-Pre | T07 | boundary, enum |
| ATOM-301.5-001/002 | 301.5 | Equipment: legal (creature) and illegal (non-creature) attachment | Phase 5-Pre | T04, T15 | equipment, boundary |
| ATOM-302.4-001/002 | 302.4 | P/T: only creatures have it | Phase 5 Layers | L04 | creature, boundary |
| ATOM-305.8-001/002 | 305.8 | Basic = supertype, not subtype | Phase 5 Layers / ALREADY-IMPL | L17 | land, boundary |
| ATOM-306.5-001 | 306.5 | Loyalty only for planeswalkers | Phase 5-Pre | T14 | planeswalker, boundary |
| ATOM-310.4 | 310.4 | Defense only for battles | DEFERRED | — | battle, boundary |

---

## COMP Index

| ID | Composes | Summary | Phase | Ticket |
|----|----------|---------|-------|--------|
| COMP-301.5c+303.4c-001 | ATOM-301.5c-004, ATOM-303.4c-002 | Creature destroyed: Equipment stays on BF unattached, Aura to GY | Phase 5-Pre | T04, T15 |
| COMP-305.7+305.6-001 | ATOM-305.7-001, ATOM-305.7-002, ATOM-305.6-002 | Blood Moon sets land to Mountain; gains {R}, loses old abilities | Phase 5 Layers | L17 |
| COMP-306.5b+306.8+306.9-001 | ATOM-306.5b-001, ATOM-306.8-001, ATOM-306.9-001 | PW enters with loyalty, takes lethal damage, SBA kills it | Phase 5-Pre | T14, T21c |
| COMP-303.4f+303.4c-001 | ATOM-303.4f-001, ATOM-303.4c-001 | Aura from GY attaches to creature, type removed → SBA | Phase 5 Layers + Phase 5-Pre | T15, T15b, L09 |
| COMP-301.5d+303.4e-001 | ATOM-301.5d-001, ATOM-303.4e-001 | Control change: creature changes, Equipment+Aura stay with original | Phase 5 Layers | L11 |
| COMP-305.7+305.8-001 | ATOM-305.7-004, ATOM-305.8-001 | Blood Moon: nonbasic stays nonbasic, keeps Legendary | Phase 5 Layers | L17 |
| COMP-303.4c+303.4e+L11-001 | ATOM-303.4c-001, ATOM-303.4e-001 | "Enchant creature you control" Aura falls off on control change | Phase 5 Layers + Phase 5-Pre | L11, T15, T15b |

**Total COMP tests: 7**

---

## META Entries

### 300.2 — META

Rule text: "Some objects have more than one card type (for example, an artifact creature). Such objects combine the aspects of each of those card types, and are subject to spells and abilities that affect either or all of those card types."

Cross-cutting rule affecting: targeting (type filters), SBAs (creature/artifact/enchantment/planeswalker-specific checks), continuous effects (L4 type changes on multi-type objects), combat (creature-only attack/block), zone guards (land+other casting restriction). Verified implicitly by type-checking predicates (`is_creature`, `is_artifact`, `has_card_type`) across all systems.

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
| 309.1–309.7 | DEFERRED | Dungeons (Phase 8–9) |
| 310.1–310.11b | DEFERRED | Battles (Phase 8–9) |
| 310.3 | PURE-DEF | Battle subtype format |
| 311.1–311.7 | OUT-OF-SCOPE | Planes (Planechase) |
| 312.1–312.7 | OUT-OF-SCOPE | Phenomena (Planechase) |
| 313.1–313.7 | DEFERRED | Vanguards (stretch goal) |
| 314.1–314.7 | OUT-OF-SCOPE | Schemes (Archenemy) |
| 315.1–315.7 | OUT-OF-SCOPE | Conspiracies |

---

## NEW Tickets

| Ticket | Rule(s) | Summary |
|--------|---------|---------|
| D25 | 300.2a | Land+other-type casting restriction |
| NEW — Equip keyword ability | 301.5b | Equip ability implementation |
| NEW — Equipment attachment legality guard in effect resolution | 301.5b | Effect-based attach rejected for illegal target |
| NEW — Equipment-creature equip restriction | 301.5c | Creature-Equipment can't equip (no reconfigure) |
| NEW — Equipment self-equip prevention | 301.5c | Equipment can't target itself with equip |
| NEW — Equip moves Equipment | 301.5c | Equip resolution detaches from old creature |
| NEW — Equipment ability activation controller check | 301.5d | Only Equipment's controller activates its abilities |
| NEW — Equipment-granted ability controller routing | 301.5d | Granted ability activated by creature's controller |
| NEW — Equipment ETB illegal-attachment guard | 301.5e | Effect putting Equipment attached to illegal target → unattached |
| NEW — Fortification attachment rules | 301.6 | Fortification attaches to lands |
| NEW — Vehicle P/T only when creature | 301.7a | Vehicle P/T gated on creature type |
| NEW — Vehicle P/T when crewed | 301.7a | Vehicle P/T active after crew |
| NEW — Vehicle P/T with continuous effects | 301.7b | Crew → P/T → anthem interaction |
| NEW — AttachTarget Player variant | 303.4 | Enchant player Auras (Curse cycle) |
| NEW — Aura falls off when enchanted player leaves | 303.4c | Multiplayer player-leaves SBA |
| NEW — Aura-granted ability controller routing | 303.4e | Granted ability activated by enchanted object's controller |
| NEW — Aura token creation guard | 303.4g | No legal target → token not created |
| NEW — Non-attachment permanent ETB unattached guard | 303.4h | Non-Aura/Equip/Fort entering "attached" → unattached |
| NEW — Aura ETB illegal-attachment guard (non-stack) | 303.4i | Aura from non-stack zone attached to illegal host stays in zone |
| NEW — Aura ETB illegal-attachment guard (stack) | 303.4i | Aura from stack attached to illegal host → graveyard |
| NEW — Aura token illegal-host guard | 303.4i | Aura token with illegal host not created |
| NEW — Aura re-attachment legality guard | 303.4j | Move Aura to illegal target → doesn't move |
| NEW — SBA for Role uniqueness | 303.7a | Multiple Roles from same controller: keep newest |
| NEW — "As an instant" timing independence | 304.5 | Casting prohibition doesn't block instant-speed abilities |
| NEW — "Put land" vs "play land" counter distinction | 305.4 | "Put" doesn't increment land play count |
| NEW — "In addition to" land type grant | 305.7 | Additive land type setting (vs replacing) |
| NEW — Planeswalker as attack target | 306.6 | validate_attackers accepts PW targets |
| NEW — "As a sorcery" timing independence | 307.5 | Casting prohibition doesn't block sorcery-speed abilities |
| NEW — Opponent sorcery-speed restriction (Teferi-style) | 307.5 | Teferi: opponents cast only at sorcery speed |
| NEW — Opponent sorcery-speed enforcement | 307.5 | Reject opponent cast at non-sorcery-speed window |
| NEW — Retroactive sorcery-timing check | 307.5a | `was_cast_at_non_sorcery_speed` flag (Necromancy) |
| NEW — Kindred card type casting dispatch | 308.1 | Kindred follows other type's casting rules |
| NEW — Effect land-play instruction ignored at limit | 305.2b | Hard cap overrides effect instruction |

---

## Gap Report

1. **Equip (702.6)** — Tested via 301.5b result but activation/resolution pipeline is Ch7 scope.
2. **Enchant (702.5)** — EnchantRestriction targeting model (T15b) tested via 303.4a/303.4f but enchant ability itself is Ch7 scope.
3. **Crew (702.122)** — Vehicle P/T tests (301.7a/b) depend on crew; crew itself is Ch7 scope.
4. **Fortify (702.67)** — 301.6 depends on fortify keyword; Ch7 scope.
5. **Loyalty abilities (606)** — 306.5d covers timing; full loyalty cost payment pipeline is Ch6 scope.
6. **PW as attack target** — 306.6 tests legality; attack-target-choice pipeline in Ch5 (508). T21c step 2.
7. **"Put land onto BF" primitive** — 305.4 distinguishes put/play but no Primitive variant exists. Phase 8 gap.
8. **Blood Moon + Urborg dependency (613.8)** — 305.7 ATOMs test isolation; dependency is Ch6 session (Gate 4).
9. **Role SBA (303.7a)** — Not in current SBA list (T13–T16). Needs new ticket at Phase 8.

---

## ALREADY-IMPLEMENTED List

301.1, 301.2, 302.1, 302.2, 302.5, 302.6, 302.7, 303.1, 303.2, 304.1, 304.2, 305.1, 305.3, 306.1, 306.2, 307.1, 307.2

(Also: ATOM-305.6-001 intrinsic mana, ATOM-305.8-002 basic supertype, ATOM-305.3-001 land timing, ATOM-303.4g-002 fizzle logic)

---

## OUT-OF-SCOPE List

| Rule(s) | Reason |
|---------|--------|
| 311.1–311.7 | Planes — Planechase supplemental format |
| 312.1–312.7 | Phenomena — Planechase supplemental format |
| 314.1–314.7 | Schemes — Archenemy supplemental format |
| 315.1–315.7 | Conspiracies — Conspiracy Draft format |

---

## DEFERRED List

| Rule(s) | Reason |
|---------|--------|
| 303.4k | Face-down Aura turning face up — Phase 9 (morph/manifest/disguise) |
| 309.1–309.7 | Dungeons — Phase 8–9 |
| 310.1–310.11b | Battles — Phase 8–9 |
| 313.1–313.7 | Vanguards — stretch goal |

---

## Test Count Summary (post-audit)

- **ATOM tests:** 87
- **COMP tests:** 7
- **Total tests:** 94
- **TESTABLE:** 36
- **BOUNDARY-DEF:** 6
- **PURE-DEF:** 28
- **META:** 1
- **ALREADY-IMPLEMENTED:** 18
- **DEFERRED:** 40
- **OUT-OF-SCOPE:** 27
- **DUPLICATE:** 1
- **Sub-rules accounted:** ~157
