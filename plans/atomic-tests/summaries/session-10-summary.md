# Session 10 Summary: Chapters 8 & 9 — Multiplayer Rules & Casual Variants

> **CR Sections:** 800–811 (Chapter 8), 900–905 (Chapter 9)
> **Generated:** 2026-04-09
> **Audited:** 2026-04-09

---

## ATOM Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-800.4a-001 | 800.4a | Player leaves: owned objects leave, control effects end | Phase 9 | D24 | multiplayer, player-leaves |
| ATOM-800.4a-002 | 800.4a | Player leaves: controlled-not-owned objects exiled | Phase 9 | D24 | multiplayer, player-leaves |
| ATOM-800.4b-001 | 800.4b | Can't create tokens/objects under left-game player's control | Phase 9 | D24 | multiplayer, player-leaves |
| ATOM-800.4c-001 | 800.4c | Orphaned object exiled when control effect ends + default controller left | Phase 9 | D24 | multiplayer, player-leaves |
| ATOM-800.4d-001 | 800.4d | Triggered ability of left-game player not put on stack | Phase 9 | D24 | multiplayer, player-leaves |
| ATOM-800.4e-001 | 800.4e | Combat damage to left-game player not assigned | Phase 9 | D24 | multiplayer, player-leaves |
| ATOM-800.4j-001 | 800.4j | Active player leaves mid-turn: turn continues, priority passes | Phase 9 | D24 | multiplayer, player-leaves |
| ATOM-800.6-001 | 800.6 | Multiplayer free mulligan (first doesn't count) | Phase 9 | D12 | multiplayer, mulligan |
| ATOM-800.7-001 | 800.7 | Starting player draws in multiplayer (not 2HG) | Phase 9 | NEW-S10-01 | multiplayer, draw |
| ATOM-802.2-001 | 802.2 | All opponents are defending players in combat | Phase 9 | D7 | multiplayer, combat |
| ATOM-802.2a-001 | 802.2a | "Defending player" on attacking creature = player it attacks | Phase 9 | D7 | multiplayer, combat |
| ATOM-802.2a-002 | 802.2a | General "defending player" = any player being attacked | Phase 9 | D7 | multiplayer, combat |
| ATOM-802.3-001 | 802.3 | Each attacker individually chooses attack target | Phase 9 | D7 | multiplayer, combat |
| ATOM-802.4-001 | 802.4 | Blockers declared in APNAP order | Phase 9 | D7 | multiplayer, combat, APNAP |
| ATOM-802.5-001 | 802.5 | Combat damage assigned in APNAP order | Phase 9 | D7 | multiplayer, combat, APNAP |
| ATOM-903.2-001 | 903.2 | Commander default = FFA + attack multiple + no range | Phase 9 | D7 | commander, setup |
| ATOM-903.3-001 | 903.3 | Commander designation: valid types, persists across zones | Phase 9 | NEW-S10-02 | commander, designation |
| ATOM-903.3-002 | 903.3 | Copy of commander is NOT a commander | Phase 9 | NEW-S10-02 | commander, designation |
| ATOM-903.3-003 | 903.3 | Face-down commander retains commander designation | Phase 9 | NEW-S10-02 | commander, designation |
| ATOM-903.3d-001 | 903.3d | "Control your commander" true when commander on battlefield | Phase 9 | NEW-S10-02 | commander, reference |
| ATOM-903.3d-002 | 903.3d | "Control your commander" false when commander in command zone | Phase 9 | NEW-S10-02 | commander, reference |
| ATOM-903.3d-003 | 903.3d | "Target commander" targets permanent that is a commander | Phase 9 | NEW-S10-02 | commander, reference |
| ATOM-903.4-001 | 903.4 | Color identity: mana symbols in rules text contribute | Phase 9 | NEW-S10-03 | commander, color-identity |
| ATOM-903.4-002 | 903.4 | Color identity: color indicator contributes | Phase 9 | NEW-S10-03 | commander, color-identity |
| ATOM-903.4c-001 | 903.4c | Color identity: reminder text ignored | Phase 9 | NEW-S10-03 | commander, color-identity |
| ATOM-903.4d-001 | 903.4d | Color identity: DFC back face included | Phase 9 | NEW-S10-03 | commander, color-identity |
| ATOM-903.5a-001 | 903.5a | Deck must be exactly 100 cards | Phase 9 | NEW-S10-04 | commander, deck-validation |
| ATOM-903.5b-001 | 903.5b | Singleton rule (except basic lands + "any number" cards) | Phase 9 | NEW-S10-04 | commander, deck-validation |
| ATOM-903.5c-001 | 903.5c | Card color identity ⊆ commander color identity | Phase 9 | NEW-S10-04 | commander, deck-validation |
| ATOM-903.5d-001 | 903.5d | Basic land type restricted by commander color identity | Phase 9 | NEW-S10-04 | commander, deck-validation |
| ATOM-903.6-001 | 903.6 | Commander starts in command zone, library = 99 | Phase 9 | NEW-S10-05 | commander, setup |
| ATOM-903.7-001 | 903.7 | Starting life = 40, hand = 7 | Phase 9 | NEW-S10-05 | commander, setup |
| ATOM-903.8-001 | 903.8 | Cast commander from command zone, first cast no tax | Phase 9 | NEW-S10-06 | commander, casting |
| ATOM-903.8-002 | 903.8 | Commander tax: second cast = +{2} | Phase 9 | NEW-S10-07 | commander, tax |
| ATOM-903.8-003 | 903.8 | Commander tax: third cast = +{4} | Phase 9 | NEW-S10-07 | commander, tax |
| ATOM-903.8-004 | 903.8 | Commander tax: only from command zone, not graveyard | Phase 9 | NEW-S10-07 | commander, tax |
| ATOM-903.9a-001 | 903.9a | Commander in graveyard → may SBA to command zone | Phase 9 | NEW-S10-08 | commander, zone-return |
| ATOM-903.9a-002 | 903.9a | Commander in exile → may SBA to command zone | Phase 9 | NEW-S10-08 | commander, zone-return |
| ATOM-903.9b-001 | 903.9b | Commander would go to hand → replacement to command zone | Phase 9 | NEW-S10-09 | commander, zone-return |
| ATOM-903.9b-002 | 903.9b | Commander would go to library → replacement to command zone | Phase 9 | NEW-S10-09 | commander, zone-return |
| ATOM-903.10a-001 | 903.10a | 21+ combat damage from one commander → lose (SBA) | Phase 9 | T16 | commander, damage |
| ATOM-903.10a-002 | 903.10a | Commander damage tracked per-commander, not total | Phase 9 | T16 | commander, damage |
| ATOM-903.10a-003 | 903.10a | Only combat damage counts, not non-combat | Phase 9 | T16 | commander, damage |
| ATOM-903.10a-004 | 903.10a | Commander damage persists across zone changes | Phase 9 | T16 | commander, damage |
| ATOM-903.10a-005 | 903.10a | Partner commanders: damage tracked independently per commander | Phase 9 | T16+NEW-S10-10 | commander, damage, partner |

---

## COMP Index

| ID | Rules | Summary | Composes | Phase | Ticket |
|----|-------|---------|----------|-------|--------|
| COMP-903-FULL-GAME-001 | 903.6+903.7+903.8+903.10a | Full Commander lifecycle: setup → cast → damage → win | ATOM-903.6-001, 903.7-001, 903.8-001, 903.10a-001 | Phase 9 | D7+T16 |
| COMP-903-TAX-AND-RETURN-001 | 903.8+903.9a | Commander dies → SBA return → recast with tax | ATOM-903.8-002, 903.9a-001 | Phase 9 | NEW-S10-11 |
| COMP-903-BOUNCE-REPLACEMENT-001 | 903.9b+903.8 | Bounce → replacement to CZ → recast with tax | ATOM-903.9b-001, 903.8-002 | Phase 9 | NEW-S10-11 |
| COMP-800-PLAYER-LEAVES-COMMANDER-001 | 800.4a+903.10a | Player leaves: cleanup + commander damage persistence | ATOM-800.4a-001, 903.10a-001 | Phase 9 | D24 |

---

## Classification Summary Table

| Rule | Classification | Phase/Notes |
|------|---------------|-------------|
| 800.1 | PURE-DEF | Multiplayer definition |
| 800.2 | PURE-DEF | Options/variants framing |
| 800.3 | PURE-DEF | Tournament rules reference |
| 800.4 | PURE-DEF | Framing: games continue after player leaves |
| 800.4a | TESTABLE | Phase 9 — D24. ATOM-800.4a-001, -002 |
| 800.4b | TESTABLE | Phase 9 — D24. ATOM-800.4b-001 |
| 800.4c | TESTABLE | Phase 9 — D24. ATOM-800.4c-001 |
| 800.4d | TESTABLE | Phase 9 — D24. ATOM-800.4d-001 |
| 800.4e | TESTABLE | Phase 9 — D24. ATOM-800.4e-001 |
| 800.4f | DEFERRED | Phase 9 — D24. Needs choice delegation infra |
| 800.4g | DEFERRED | Phase 9 — D24. Needs choice delegation infra |
| 800.4h | DEFERRED | Phase 9 — D24. Needs choice delegation infra |
| 800.4i | DEFERRED | Phase 9 — D24. Needs LKI (Phase 5) |
| 800.4j | TESTABLE | Phase 9 — D24. ATOM-800.4j-001 |
| 800.4k | DEFERRED | Phase 9 — D24. Tested implicitly by 800.4j |
| 800.4m | DEFERRED | Phase 9 — D24. Needs continuous effects (Phase 5) |
| 800.4n | OUT-OF-SCOPE | Ante zone |
| 800.4p | OUT-OF-SCOPE | Planechase |
| 800.5 | PURE-DEF | Seating order |
| 800.6 | TESTABLE | Phase 9 — D12. ATOM-800.6-001 |
| 800.7 | TESTABLE | Phase 9 — NEW-S10-01. ATOM-800.7-001 |
| 801.1–801.18 | DEFERRED | Phase 9 (stretch) — Range of influence |
| 802.1 | PURE-DEF | Attack multiple players framing |
| 802.2 | TESTABLE | Phase 9 — D7. ATOM-802.2-001 |
| 802.2a | TESTABLE | Phase 9 — D7. ATOM-802.2a-001, -002 |
| 802.3 | TESTABLE | Phase 9 — D7. ATOM-802.3-001 |
| 802.3a | DEFERRED | Phase 9 — T21d |
| 802.3b | OUT-OF-SCOPE | Banding |
| 802.4 | TESTABLE | Phase 9 — D7. ATOM-802.4-001 |
| 802.4a | DEFERRED | Phase 9 — D7 |
| 802.4b | DEFERRED | Phase 9 — D7 |
| 802.5 | TESTABLE | Phase 9 — D7. ATOM-802.5-001 |
| 803.1, 803.1a, 803.1b | OUT-OF-SCOPE | Attack left/right options |
| 804.1, 804.2 | OUT-OF-SCOPE | Deploy creatures option |
| 805.1–805.10f | OUT-OF-SCOPE | Shared team turns |
| 806.1 | PURE-DEF | Free-for-All definition |
| 806.2, 806.2a–c | PURE-DEF | FFA defaults; tested via ATOM-903.2-001, no FFA wrapper for v1 |
| 806.3 | PURE-DEF | Random seating |
| 807.1–807.5b | OUT-OF-SCOPE | Grand Melee |
| 808.1–808.5 | OUT-OF-SCOPE | Team vs. Team |
| 809.1–809.7 | OUT-OF-SCOPE | Emperor |
| 810.1–810.11 | OUT-OF-SCOPE | Two-Headed Giant — team format, shared payments too complex |
| 811.1–811.5 | OUT-OF-SCOPE | Alternating Teams |
| 900.1 | PURE-DEF | Casual variants framing |
| 900.2 | PURE-DEF | Supplemental zones framing |
| 901.1–901.15c | OUT-OF-SCOPE | Planechase |
| 902.1–902.7 | DEFERRED | Future (stretch) — Vanguard |
| 903.1 | PURE-DEF | Commander variant framing |
| 903.2 | TESTABLE | Phase 9 — D7. ATOM-903.2-001 |
| 903.3 | TESTABLE | Phase 9 — NEW-S10-02. ATOM-903.3-001, -002, -003 |
| 903.3a | DEFERRED | Phase 9 — "Can be your commander" ability |
| 903.3b | DEFERRED | Phase 9 (stretch) — Meld + commander |
| 903.3c | DEFERRED | Phase 9 (stretch) — Merged permanent + commander |
| 903.3d | TESTABLE | Phase 9 — NEW-S10-02. ATOM-903.3d-001, -002, -003 |
| 903.3e | DEFERRED | Phase 9 — Cross-zone "your commander" reference |
| 903.4 | TESTABLE | Phase 9 — NEW-S10-03. ATOM-903.4-001, -002 |
| 903.4a | PURE-DEF | Color identity timing |
| 903.4b | DEFERRED | Phase 9 — Pre-game color choice |
| 903.4c | TESTABLE | Phase 9 — NEW-S10-03. ATOM-903.4c-001 |
| 903.4d | TESTABLE | Phase 9 — NEW-S10-03. ATOM-903.4d-001 |
| 903.4e | DEFERRED | Phase 9 — Adventure color identity |
| 903.4f | DEFERRED | Phase 9 — Undefined color identity |
| 903.5 | PURE-DEF | Deck construction framing |
| 903.5a | TESTABLE | Phase 9 — NEW-S10-04. ATOM-903.5a-001 |
| 903.5b | TESTABLE | Phase 9 — NEW-S10-04. ATOM-903.5b-001 |
| 903.5c | TESTABLE | Phase 9 — NEW-S10-04. ATOM-903.5c-001 |
| 903.5d | TESTABLE | Phase 9 — NEW-S10-04. ATOM-903.5d-001 |
| 903.5e | PURE-DEF | No sideboards (note: Companion exception, see 702.139) |
| 903.6 | TESTABLE | Phase 9 — NEW-S10-05. ATOM-903.6-001 |
| 903.7 | TESTABLE | Phase 9 — NEW-S10-05. ATOM-903.7-001 |
| 903.8 | TESTABLE | Phase 9 — NEW-S10-06/07. ATOM-903.8-001 through -004 |
| 903.9 | PURE-DEF | Commander return framing |
| 903.9a | TESTABLE | Phase 9 — NEW-S10-08. ATOM-903.9a-001, -002 |
| 903.9b | TESTABLE | Phase 9 — NEW-S10-09. ATOM-903.9b-001, -002 |
| 903.9c | DEFERRED | Phase 9 (stretch) — Meld/merge zone-return |
| 903.10 | PURE-DEF | Win/loss framing |
| 903.10a | TESTABLE | Phase 9 — T16. ATOM-903.10a-001 through -005 |
| 903.11 | DEFERRED | Phase 9 — Outside-game cards |
| 903.11a | DEFERRED | Phase 9 — Wish restrictions (note: Companion cross-ref) |
| 903.12a–903.12h | DEFERRED | Phase 9 (stretch) — Brawl option |
| 903.13a–903.13g | DEFERRED | Future — Commander Draft, requires engine first |
| 904.1–904.13d | OUT-OF-SCOPE | Archenemy |
| 905.1–905.6 | OUT-OF-SCOPE | Conspiracy Draft |

---

## NEW Tickets

| Ticket ID | Description | Phase |
|-----------|-------------|-------|
| NEW-S10-01 | Commander GameConfig: first_player_draws = true for multiplayer | Phase 9 |
| NEW-S10-02 | Commander designation validation (valid types, persistence, copy ≠ commander, face-down, reference resolution) | Phase 9 |
| NEW-S10-03 | Color identity computation (mana cost, rules text, color indicator, DFC back face, reminder text exclusion) | Phase 9 |
| NEW-S10-04 | Commander deck validation (100-card, singleton + "any number" override, color identity restriction, basic land type restriction) | Phase 9 |
| NEW-S10-05 | Commander game setup (command zone placement, 40 life, 7 cards) | Phase 9 |
| NEW-S10-06 | Commander casting from command zone | Phase 9 |
| NEW-S10-07 | Commander tax implementation (additional {2} per previous command zone cast, zone specificity) | Phase 9 |
| NEW-S10-08 | Commander zone-return SBA (graveyard/exile → command zone choice) | Phase 9 |
| NEW-S10-09 | Commander zone-return replacement effect (hand/library → command zone choice) | Phase 9 |
| NEW-S10-10 | Partner commander support (two commanders, union color identity, per-commander tax, per-commander damage tracking) | Phase 9 |
| NEW-S10-11 | Commander tax + zone-return integration test | Phase 9 |

---

## Gap Report

| # | Gap | Description | Recommendation |
|---|-----|-------------|----------------|
| G1 | Partner commanders | 702.124 allows two commanders. Affects tax, color identity, deck size, command zone. | NEW-S10-10 |
| G2 | Commander as DFC | Casting DFC commander from command zone — which face? | Test when DFC casting implemented |
| G3 | Commander damage + prevention | Prevented damage never dealt → doesn't count | NEW: ATOM-903.10a-006 |
| G4 | Commander damage + redirection | Redirected damage same source → counts for new recipient | NEW: ATOM-903.10a-007 |
| G5 | APNAP for N players | Core APNAP for >2 players is Phase 9 prerequisite | Ensure D7 covers |
| G6 | Commander + continuous effects | Modified P/T affects commander damage tracking | Cross-ref Phase 5 layers |

---

## OUT-OF-SCOPE List

| Rules | Reason |
|-------|--------|
| 800.4n | Ante zone |
| 800.4p | Planechase |
| 802.3b | Banding |
| 803.1, 803.1a, 803.1b | Attack left/right options |
| 804.1, 804.2 | Deploy creatures (Emperor) |
| 805.1–805.10f | Shared team turns (2HG/Archenemy) |
| 807.1–807.5b | Grand Melee |
| 808.1–808.5 | Team vs. Team |
| 809.1–809.7 | Emperor |
| 810.1–810.11 | Two-Headed Giant — team format, shared payments too complex |
| 811.1–811.5 | Alternating Teams |
| 901.1–901.15c | Planechase |
| 904.1–904.13d | Archenemy |
| 905.1–905.6 | Conspiracy Draft |

---

## DEFERRED List

| Rules | Phase | Reason |
|-------|-------|--------|
| 800.4f–800.4i | Phase 9 (D24) | Player-leaves-game: costs, choices, LKI — needs choice delegation + LKI infra |
| 800.4k, 800.4m | Phase 9 (D24) | Player-leaves-game: turn skip, continuous effect duration |
| 801.1–801.18 | Phase 9 (stretch) | Range of influence — Commander default OFF |
| 802.3a | Phase 9 (T21d) | Attack restriction/requirement evaluation for multiplayer |
| 802.4a, 802.4b | Phase 9 (D7) | Block legality details in multiplayer |
| 902.1–902.7 | Future (stretch) | Vanguard |
| 903.3a | Phase 9 | "Can be your commander" ability |
| 903.3b, 903.3c | Phase 9 (stretch) | Meld/merge + commander |
| 903.3e | Phase 9 | Cross-zone "your commander" reference |
| 903.4b | Phase 9 | Pre-game color choice |
| 903.4e | Phase 9 | Adventure color identity |
| 903.4f | Phase 9 | Undefined color identity |
| 903.9c | Phase 9 (stretch) | Meld/merge zone-return |
| 903.11, 903.11a | Phase 9 | Outside-game card restrictions (note: Companion cross-ref) |
| 903.12a–903.12h | Phase 9 (stretch) | Brawl option |
| 903.13a–903.13g | Future | Commander Draft — requires engine first |

---

## Counts

| Category | Count |
|----------|-------|
| ATOM tests | 51 |
| COMP tests | 4 |
| DEFERRED rules | 65 |
| OUT-OF-SCOPE rules | 180 |
| PURE-DEF rules | 21 |
| NEW tickets | 11 |
| Gaps identified | 6 |
