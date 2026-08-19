# Session 10: Chapters 8 & 9 — Multiplayer Rules & Casual Variants

> **CR Sections:** 800–811 (Chapter 8), 900–905 (Chapter 9)
> **Generated:** 2026-04-09
> **Informed by:** design_doc.md, roadmap.md, implementation-plan-final.md, CR text files

---

## Scope & Classification Guidance

- **OUT-OF-SCOPE (permanently excluded):** Planechase (901), Archenemy (904), Conspiracy Draft (905), Grand Melee (807), Emperor (809), Alternating Teams (811), Shared Team Turns option (805), Deploy Creatures option (804), Attack Left/Right options (803), Team vs. Team (808), Two-Headed Giant (810)
- **DEFERRED — Phase 9 (Commander):** Commander rules (903.x), general multiplayer foundations needed for Commander (800.x, 802.x)
- **DEFERRED — Future:** Vanguard (902.x) — stretch goal; Commander Draft (903.13) — requires engine first
- **DEFERRED — Phase 9 (stretch):** Brawl (903.12) — alt Commander option

Most of Chapter 8 is DEFERRED or OUT-OF-SCOPE. Chapter 9's Commander section (903.x) is the primary source of TESTABLE rules for Phase 9.

---

## Chunk Plan

| Chunk | Rule Range | Expected Output |
|-------|-----------|-----------------|
| **0** | (this header) | Session header, chunk plan |
| **1** | 800.x – 811.x (Chapter 8) | Mostly DEFERRED Phase 9 one-liners; a few ATOM tests for 800.4x, 802.x |
| **2** | 900.x – 905.x (Chapter 9) | Commander (903.x) full ATOM tests; others OOS/DEFERRED |
| **3** | Summary table, COMP tests, Gap Report | Final classification + cross-references |

---

## Chunk 1: Chapter 8 — Multiplayer Rules (800–811)

### 800. General

**800.1** — PURE-DEF. Defines "multiplayer game" as >2 players. Prerequisite for all 800.x rules.

**800.2** — PURE-DEF. Options + variants framing.

**800.3** — PURE-DEF. Tournament rules reference. No mechanical consequence.

**800.4** — PURE-DEF. Framing statement: multiplayer games continue after players leave.

> **Audit note — DEFERRED vs TESTABLE rationale for 800.4x:**
> The TESTABLE sub-rules (800.4a–e, 800.4j) describe **observable state transformations** with clear before/after assertions — objects leaving the game, objects being exiled, damage not being assigned, priority passing. These can be tested with minimal board states.
> The DEFERRED sub-rules (800.4f–i, 800.4k, 800.4m) describe **edge-case resolution policies** — what happens when a left-game player would need to pay costs, make choices, or be referenced by LKI. These require deeper infrastructure (choice redirection, LKI cross-zone queries) that depends on systems not yet built. They are testable in principle but deferred because they can't be meaningfully tested until their prerequisite systems exist (LKI from Phase 5, choice delegation from Phase 7). They remain under D24 and will become TESTABLE when their dependencies land.

**ATOM-800.4a-001**
- **Rule:** 800.4a — When a player leaves the game, all objects owned by that player leave the game, control-changing effects end, unrepresented stack objects cease to exist, remaining controlled objects are exiled.
- **Mechanism:** Player-leaves-game cleanup — owned objects removal
- **Minimal Board:** 4-player Commander game. Player B owns a creature on the battlefield and a spell on the stack. Player A controls an Aura (Mind Control) attached to Player C's creature.
- **Action:** Player A leaves the game.
- **Expected Result:** Mind Control (owned by A) leaves the game. Player C's creature reverts to C's control. A's objects in all zones leave the game. Priority passes to next player in turn order.
- **Phase:** Phase 9
- **Ticket:** D24 (Player-leaves-game cleanup)

**ATOM-800.4a-002**
- **Rule:** 800.4a — When a player leaves and controls objects they don't own (no control-changing effect to end), those objects are exiled.
- **Mechanism:** Player-leaves-game cleanup — exile controlled-but-not-owned objects
- **Minimal Board:** 4-player game. Player A used Bribery to put Player B's Serra Angel onto the battlefield under A's control (no ongoing control effect — A is the default controller).
- **Action:** Player A leaves the game.
- **Expected Result:** Serra Angel is exiled (A controlled it, B owns it, no control effect to end, so the "objects still controlled by that player" clause exiles it).
- **Phase:** Phase 9
- **Ticket:** D24

**ATOM-800.4b-001**
- **Rule:** 800.4b — Object can't change control to a player who has left the game; token can't be created under their control; object can't be put onto battlefield/stack under their control.
- **Mechanism:** Player-leaves-game — prevention of control/creation
- **Minimal Board:** 4-player game. Player A has left the game. An effect on the stack would create a token under A's control.
- **Action:** Token creation resolves.
- **Expected Result:** No token is created.
- **Phase:** Phase 9
- **Ticket:** D24

**ATOM-800.4c-001**
- **Rule:** 800.4c — If a control-changing effect ends, no other effect gives control to another in-game player, and the default controller has left the game, the object is exiled immediately.
- **Mechanism:** Player-leaves-game — orphaned object exile
- **Minimal Board:** 4-player game. Player B owns a creature. Player A cast Act of Treason targeting it (temporary control). Player B then leaves the game. At end of turn, Act of Treason's effect ends.
- **Action:** Act of Treason's "until end of turn" control effect ends.
- **Expected Result:** The creature is exiled (default controller B has left the game, no other control effect exists).
- **Phase:** Phase 9
- **Ticket:** D24

**ATOM-800.4d-001**
- **Rule:** 800.4d — Object that would be owned by a left-game player isn't created. Triggered ability controlled by left-game player isn't put on stack.
- **Mechanism:** Player-leaves-game — trigger suppression
- **Minimal Board:** 4-player game. Player B controls Astral Slide and exiles Player A's creature with a delayed trigger to return it. Player B then leaves the game.
- **Action:** Delayed trigger fires at end step.
- **Expected Result:** The triggered ability is not put on the stack. The creature remains exiled.
- **Phase:** Phase 9
- **Ticket:** D24

**ATOM-800.4e-001**
- **Rule:** 800.4e — Combat damage to a player who has left the game isn't assigned.
- **Mechanism:** Player-leaves-game — combat damage suppression
- **Minimal Board:** 4-player game. Player A attacks Player B. Before damage step, Player B leaves the game.
- **Action:** Combat damage step.
- **Expected Result:** No combat damage is assigned to B. The attacking creature deals no damage.
- **Phase:** Phase 9
- **Ticket:** D24

**800.4f** — DEFERRED — Phase 9. Cost payment by left-game player. Classified with D24.

**800.4g** — DEFERRED — Phase 9. Choice redirection when player has left. Classified with D24.

**800.4h** — DEFERRED — Phase 9. Rule-required choice by left-game player → next in turn order. Classified with D24.

**800.4i** — DEFERRED — Phase 9. LKI for departed player information. Classified with D24. Cross-ref: LKI system (Phase 5 L18).

**ATOM-800.4j-001**
- **Rule:** 800.4j — If active player leaves during their turn, the turn continues without an active player. Priority passes to next player in turn order.
- **Mechanism:** Player-leaves-game — active player departure mid-turn
- **Minimal Board:** 4-player game (A, B, C, D). It is Player A's turn, main phase 1. A has priority.
- **Action:** Player A leaves the game (e.g., concedes).
- **Expected Result:** The turn continues. Priority passes to Player B (next in turn order). The phase/step structure completes normally. A's next turn is skipped (800.4k).
- **Phase:** Phase 9
- **Ticket:** D24

**800.4k** — DEFERRED — Phase 9. Left-game player's turn doesn't begin. Classified with D24. Tested implicitly by 800.4j-001.

**800.4m** — DEFERRED — Phase 9. Continuous effect duration handling when player leaves. Classified with D24.

**800.4n** — OUT-OF-SCOPE. Ante zone. Ante is permanently excluded.

**800.4p** — OUT-OF-SCOPE. Planechase-specific rule.

**800.5** — PURE-DEF. Seating order determination. No mechanical consequence for engine.

**ATOM-800.6-001**
- **Rule:** 800.6 — In a multiplayer game, the first mulligan a player takes doesn't count toward the number of cards that player will put on the bottom of their library or the number of mulligans that player may take.
- **Mechanism:** Multiplayer free mulligan
- **Minimal Board:** 4-player Commander game setup. Player A takes a mulligan.
- **Action:** Player A takes their first mulligan.
- **Expected Result:** Player A draws 7 cards (not 6). The mulligan count for bottom-of-library purposes is 0. On a second mulligan, they draw 7 and put 1 on the bottom.
- **Phase:** Phase 9
- **Ticket:** D12 (Mulligan implementation)

**ATOM-800.7-001**
- **Rule:** 800.7 — In a multiplayer game other than Two-Headed Giant, the starting player doesn't skip their first draw step.
- **Mechanism:** Multiplayer first-player draw rule
- **Minimal Board:** 4-player Commander game. Player A goes first.
- **Action:** Player A's first turn, draw step.
- **Expected Result:** Player A draws a card (unlike 2-player where starting player skips draw). GameConfig for Commander should set `first_player_draws: true`.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander GameConfig: first_player_draws = true for multiplayer

### 801. Limited Range of Influence Option

**801.1–801.18** (all sub-rules) — DEFERRED — Phase 9 (stretch). The default Commander setup (903.2) does NOT use limited range of influence. All 801.x rules are relevant only if the option is enabled. Classify entire section as DEFERRED. If Commander ever enables range of influence as an option, these rules would need implementation.

Sub-rules: 801.1, 801.2, 801.2a, 801.2b, 801.2c, 801.2d, 801.3, 801.4, 801.5, 801.5a, 801.5b, 801.5c, 801.6, 801.7, 801.7a, 801.8, 801.9, 801.10, 801.11, 801.12, 801.13, 801.13a, 801.13b, 801.14, 801.15, 801.16, 801.17, 801.18.

### 802. Attack Multiple Players Option

Commander default uses this option (903.2 → 806.2 → attack multiple players). These rules ARE Phase 9 scope.

**802.1** — PURE-DEF. Framing: active player may attack multiple opponents.

**ATOM-802.2-001**
- **Rule:** 802.2 — All opponents are defending players during combat (no single defending player chosen at combat start).
- **Mechanism:** Multiplayer combat — all-opponents-defend
- **Minimal Board:** 4-player Commander game (A, B, C, D). A's combat phase. A controls two creatures.
- **Action:** Combat phase begins.
- **Expected Result:** B, C, and D are all defending players. A does not choose a single defending player. Each attacker will individually choose its target (802.3).
- **Phase:** Phase 9
- **Ticket:** D7 (Multiplayer systems)

**ATOM-802.2a-001**
- **Rule:** 802.2a — "Defending player" as used on an attacking creature means the player that creature is attacking, or the controller/protector of the planeswalker/battle it's attacking. For any other reference, "defending player" means any player who is being attacked.
- **Mechanism:** "Defending player" reference resolution — per-creature vs general
- **Minimal Board:** 4-player Commander game (A, B, C, D). A attacks B with Creature X (has "Whenever this creature attacks, defending player discards a card") and attacks C with Creature Y.
- **Action:** Creature X's triggered ability resolves.
- **Expected Result:** "Defending player" on Creature X refers to Player B (the player X is attacking), not C or D. B discards a card.
- **Phase:** Phase 9
- **Ticket:** D7 (Multiplayer systems)

**ATOM-802.2a-002**
- **Rule:** 802.2a — General "defending player" reference (not on a specific attacking creature) means any player being attacked.
- **Mechanism:** "Defending player" reference resolution — general context
- **Minimal Board:** 4-player Commander game (A, B, C, D). A attacks B and C. A controls an enchantment that says "Whenever you attack, defending players can't gain life this turn."
- **Action:** The enchantment's ability triggers on attack declaration.
- **Expected Result:** Both B and C (all defending players) can't gain life this turn. D (not being attacked) is unaffected.
- **Phase:** Phase 9
- **Ticket:** D7

**ATOM-802.3-001**
- **Rule:** 802.3 — Each attacking creature individually chooses a defending player (or PW/battle) to attack.
- **Mechanism:** Multiplayer combat — per-creature attack target
- **Minimal Board:** 4-player Commander game (A, B, C, D). A controls Creature X and Creature Y.
- **Action:** A declares attackers: Creature X attacks B, Creature Y attacks C.
- **Expected Result:** Both declarations are legal. X is attacking B, Y is attacking C. B and C are each defending players for their respective attackers.
- **Phase:** Phase 9
- **Ticket:** D7

**802.3a** — DEFERRED — Phase 9. Restriction/requirement evaluation for multi-player attacks. Classified with combat requirements solver (T21d).

**802.3b** — OUT-OF-SCOPE. Banding restriction for multi-player attacks. Banding is not planned.

**ATOM-802.4-001**
- **Rule:** 802.4 — Multiple defending players declare blockers in APNAP order.
- **Mechanism:** Multiplayer combat — APNAP blocker declaration
- **Minimal Board:** 4-player Commander game (A, B, C, D). A attacks B and C with separate creatures. B and C each control creatures.
- **Action:** Declare blockers step begins.
- **Expected Result:** B (next in APNAP after A) declares all their blocks first, then C declares all their blocks. Each player only blocks creatures attacking them (802.4a).
- **Phase:** Phase 9
- **Ticket:** D7

**802.4a** — DEFERRED — Phase 9. Defending player blocks only creatures attacking them. Tested implicitly by 802.4-001.

**802.4b** — DEFERRED — Phase 9. Block legality ignores other players' attackers/blockers. Tested implicitly by 802.4-001.

**ATOM-802.5-001**
- **Rule:** 802.5 — Combat damage is assigned in APNAP order in multiplayer.
- **Mechanism:** Multiplayer combat — APNAP damage assignment
- **Minimal Board:** 4-player Commander game (A, B, C, D). A attacks B with Creature X (unblocked) and C with Creature Y (unblocked).
- **Action:** Combat damage step.
- **Expected Result:** Damage from creatures attacking B is assigned first (B is first defending player in APNAP after A), then damage from creatures attacking C. Both assignments happen, then all damage is dealt simultaneously.
- **Phase:** Phase 9
- **Ticket:** D7

### 803. Attack Left and Attack Right Options

**803.1, 803.1a, 803.1b** — OUT-OF-SCOPE. Commander default uses "attack multiple players" (802), not attack left/right. These options are for specific variants (Grand Melee, etc.) that are out of scope.

### 804. Deploy Creatures Option

**804.1, 804.2** — OUT-OF-SCOPE. Emperor-only option. Team formats excluded.

### 805. Shared Team Turns Option

**805.1–805.10f** (all sub-rules) — OUT-OF-SCOPE. Used only in Two-Headed Giant and Archenemy, both excluded team formats. Sub-rules: 805.1, 805.2, 805.3, 805.3a, 805.3b, 805.4, 805.4a, 805.4b, 805.4c, 805.4d, 805.5, 805.5a, 805.5b, 805.6, 805.6a, 805.7, 805.8, 805.9, 805.10, 805.10a, 805.10b, 805.10c, 805.10d, 805.10e, 805.10f.

### 806. Free-for-All Variant

**806.1** — PURE-DEF. FFA = individuals competing.

**806.2, 806.2a, 806.2b, 806.2c** — PURE-DEF. Free-for-All default options (no limited range, attack multiple players, no deploy creatures). For v1, Commander is implemented directly without an intermediate FFA abstraction layer. The FFA defaults are tested via ATOM-903.2-001 which verifies the Commander GameConfig matches these options. No separate FFA test needed.

**806.3** — PURE-DEF. Random seating. No engine consequence.

### 807. Grand Melee Variant

**807.1–807.5b** (all sub-rules) — OUT-OF-SCOPE. Grand Melee is a specialized 10+ player variant not in project scope. Sub-rules: 807.1, 807.2, 807.2a, 807.2b, 807.2c, 807.3, 807.4, 807.4a, 807.4b, 807.4c, 807.4d, 807.4e, 807.4f, 807.4g, 807.4h, 807.4i, 807.4j, 807.5, 807.5a, 807.5b.

### 808. Team vs. Team Variant

**808.1–808.5** (all sub-rules) — OUT-OF-SCOPE. Team formats excluded. Sub-rules: 808.1, 808.2, 808.3, 808.3a, 808.3b, 808.4, 808.5.

### 809. Emperor Variant

**809.1–809.7** (all sub-rules) — OUT-OF-SCOPE. Emperor is a team format, excluded. Sub-rules: 809.1, 809.2, 809.3, 809.3a, 809.3b, 809.3c, 809.4, 809.5, 809.5a, 809.5b, 809.5c, 809.6, 809.6a, 809.7.

### 810. Two-Headed Giant Variant

**810.1–810.11** (all sub-rules) — OUT-OF-SCOPE. Two-Headed Giant is a team format with shared life totals, shared poison counters, and complex shared-payment rules that are prohibitively difficult to simulate correctly. Team formats excluded. Sub-rules: 810.1, 810.2, 810.3, 810.4, 810.5, 810.6, 810.7, 810.8, 810.8a, 810.8b, 810.8c, 810.8d, 810.9, 810.9a, 810.9b, 810.9c, 810.9d, 810.9e, 810.9f, 810.9g, 810.9h, 810.10, 810.10a, 810.10b, 810.10c, 810.10d, 810.11.

### 811. Alternating Teams Variant

**811.1–811.5** (all sub-rules) — OUT-OF-SCOPE. Team format, excluded. Sub-rules: 811.1, 811.2, 811.2a, 811.2b, 811.2c, 811.3, 811.4, 811.5.

--- End of Chunk 1 ---

## Chunk 2: Chapter 9 — Casual Variants (900–905)

### 900. General

**900.1** — PURE-DEF. Framing for casual variants section.

**900.2** — PURE-DEF. Supplemental zones/rules/cards framing.

### 901. Planechase

**901.1–901.15c** (all sub-rules) — OUT-OF-SCOPE. Planechase is permanently excluded. Sub-rules: 901.1, 901.2, 901.3, 901.3a, 901.4, 901.5, 901.6, 901.7, 901.7a, 901.8, 901.9, 901.9a, 901.9b, 901.9c, 901.9d, 901.10, 901.10a, 901.10b, 901.11, 901.11a, 901.11b, 901.11c, 901.12, 901.12a, 901.12b, 901.12c, 901.12d, 901.13, 901.14, 901.14a, 901.14b, 901.15, 901.15a, 901.15b, 901.15c.

### 902. Vanguard

**902.1–902.7** (all sub-rules) — DEFERRED — Future (stretch goal). Vanguard is a stretch goal per project scope. Sub-rules: 902.1, 902.2, 902.3, 902.4, 902.5, 902.5a, 902.5b, 902.6, 902.7.

### 903. Commander

This is the primary in-scope section for Phase 9. Full ATOM tests generated below.

**903.1** — PURE-DEF. Commander variant framing. Prerequisite for all 903.x rules.

> **Design note:** Architect the commander validation system with two tiers: (1) **generic** — any permanent spell can be a commander (no restriction, or just "must be a permanent card"), and (2) **standard** — must be a legendary creature, Vehicle, or Spacecraft with P/T boxes (903.3). This allows alt formats like "nonlegendary commander" to be trivially supported via a GameConfig flag without engine changes. The restriction tier is a property of the format, not hardcoded into the commander system.

**ATOM-903.2-001**
- **Rule:** 903.2 — Commander default multiplayer setup is Free-for-All with attack multiple players option and without limited range of influence.
- **Mechanism:** Commander format configuration
- **Minimal Board:** Commander game being initialized with 4 players.
- **Action:** Game setup.
- **Expected Result:** Game variant = Free-for-All (806). Attack multiple players = ON (802). Limited range of influence = OFF. Verified via GameConfig/Format trait.
- **Phase:** Phase 9
- **Ticket:** D7 (Multiplayer systems)

**ATOM-903.3-001**
- **Rule:** 903.3 — Each deck has a legendary card designated as its commander. Must be (a) creature card, (b) Vehicle card, or (c) Spacecraft card with P/T boxes. Designation is an attribute of the card, not a characteristic. Retained across zone changes.
- **Mechanism:** Commander designation — valid commander types
- **Minimal Board:** A deck with a legendary creature card designated as commander.
- **Action:** Validate deck construction.
- **Expected Result:** Legendary creature card is a valid commander. Non-legendary creature is rejected. Non-creature, non-Vehicle legendary card (e.g., legendary enchantment) is rejected. Commander designation persists when card moves zones (e.g., battlefield → graveyard → command zone).
- **Phase:** Phase 9
- **Ticket:** NEW — Commander designation validation

**ATOM-903.3-002**
- **Rule:** 903.3 — Commander designation is an attribute of the card, not a characteristic. A copy of a commander is NOT a commander.
- **Mechanism:** Commander designation — copy is not commander
- **Minimal Board:** Player A has a commander (legendary creature) on the battlefield. Player B controls a Clone that copies A's commander.
- **Action:** Check if Clone is a commander.
- **Expected Result:** Clone is NOT a commander. It has the same characteristics but not the commander designation. Commander damage dealt by Clone does not count as commander damage.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander designation validation

**ATOM-903.3-003**
- **Rule:** 903.3 — A commander that's been turned face down (e.g., due to Ixidron's effect) is still a commander.
- **Mechanism:** Commander designation — persistence when face down
- **Minimal Board:** Player A's commander is on the battlefield face up. An effect turns it face down (e.g., Ixidron).
- **Action:** Check if the face-down permanent is still a commander.
- **Expected Result:** The face-down permanent IS still a commander. Commander designation is an attribute of the card, not a characteristic, so it is not lost when face-down status removes characteristics. Combat damage dealt by this face-down creature still counts as commander damage.
- **Phase:** Phase 9
- **Ticket:** NEW-S10-02 (Commander designation validation)

**903.3a** — DEFERRED — Phase 9. "Can be your commander" ability (e.g., planeswalker commanders). Rules-modifying ability for deck construction. Implement when such cards are added.

**903.3b** — DEFERRED — Phase 9 (stretch). Meld + commander. Very niche interaction.

**903.3c** — DEFERRED — Phase 9 (stretch). Merged permanent + commander. Mutate interaction.

**ATOM-903.3d-001**
- **Rule:** 903.3d — "Controlling a commander" refers to a permanent on the battlefield that is a commander. "Casting a commander" refers to a spell that is a commander. "Commander in a specific zone" refers to a card in that zone that is a commander.
- **Mechanism:** Commander reference resolution — "control your commander" condition
- **Minimal Board:** Player A's commander is on the battlefield. A controls a card with "You may cast this spell without paying its mana cost if you control your commander." A's commander is a permanent on the battlefield.
- **Action:** Player A attempts to cast the free-cost spell.
- **Expected Result:** The condition "you control your commander" is true (A's commander is a permanent on the battlefield). The spell may be cast without paying its mana cost.
- **Phase:** Phase 9
- **Ticket:** NEW-S10-02

**ATOM-903.3d-002**
- **Rule:** 903.3d — "Controlling a commander" is false when commander is in command zone (not battlefield).
- **Mechanism:** Commander reference resolution — "control your commander" fails in command zone
- **Minimal Board:** Player A's commander is in the command zone (not yet cast). A controls the same free-cost card as above.
- **Action:** Player A attempts to use the free-cast condition.
- **Expected Result:** The condition "you control your commander" is FALSE. The commander is in the command zone, not the battlefield, so it is not a permanent A controls. The spell must be cast normally with mana payment.
- **Phase:** Phase 9
- **Ticket:** NEW-S10-02

**ATOM-903.3d-003**
- **Rule:** 903.3d — "Target commander" refers to a permanent on the battlefield that is a commander.
- **Mechanism:** Commander reference resolution — targeting a commander
- **Minimal Board:** Player A's commander is on the battlefield. Player B controls a spell "Target commander gains lifelink until end of turn."
- **Action:** B casts the spell targeting A's commander.
- **Expected Result:** A's commander is a legal target (it is a permanent on the battlefield that is a commander). It gains lifelink until end of turn. A Clone copying A's commander is NOT a legal target (not a commander per 903.3).
- **Phase:** Phase 9
- **Ticket:** NEW-S10-02

**903.3e** — DEFERRED — Phase 9. "Your commander" characteristic reference across all zones. Requires cross-zone visibility infrastructure.

**ATOM-903.4-001**
- **Rule:** 903.4 — Color identity of a card = colors of mana symbols in mana cost + rules text + colors from CDA + color indicator.
- **Mechanism:** Color identity computation — mana cost symbols
- **Minimal Board:** Card: Bosh, Iron Golem — mana cost {8}, ability "{3}{R}, Sacrifice an artifact: ...".
- **Action:** Compute color identity.
- **Expected Result:** Color identity = {Red}. The {R} in the ability's cost contributes to color identity even though the mana cost has no colored symbols.
- **Phase:** Phase 9
- **Ticket:** NEW — Color identity computation

**ATOM-903.4-002**
- **Rule:** 903.4 — Color identity includes colors from color indicator.
- **Mechanism:** Color identity computation — color indicator
- **Minimal Board:** Card with no mana cost but a red color indicator (e.g., back face of a DFC).
- **Action:** Compute color identity.
- **Expected Result:** Color identity includes Red (from color indicator).
- **Phase:** Phase 9
- **Ticket:** NEW — Color identity computation

**903.4a** — PURE-DEF. Color identity is established before game begins. No independent test — prerequisite for 903.5c.

**903.4b** — DEFERRED — Phase 9. Pre-game color choice for commanders (e.g., "choose a color"). Niche.

**ATOM-903.4c-001**
- **Rule:** 903.4c — Reminder text is ignored when determining color identity.
- **Mechanism:** Color identity computation — reminder text exclusion
- **Minimal Board:** Card with reminder text containing a mana symbol (e.g., extort reminder text mentions {W/B}).
- **Action:** Compute color identity.
- **Expected Result:** The mana symbols in reminder text do NOT contribute to color identity. Only mana symbols in rules text (non-reminder) and mana cost count.
- **Phase:** Phase 9
- **Ticket:** NEW — Color identity computation

**ATOM-903.4d-001**
- **Rule:** 903.4d — Back face of a DFC is included in color identity determination. Exception to 712.8a.
- **Mechanism:** Color identity computation — DFC back face
- **Minimal Board:** DFC: front face has mana cost {2}{U}, back face has red color indicator.
- **Action:** Compute color identity.
- **Expected Result:** Color identity = {Blue, Red}. Both faces contribute.
- **Phase:** Phase 9
- **Ticket:** NEW — Color identity computation

**903.4e** — DEFERRED — Phase 9. Adventure card alternative characteristics in color identity. Implement with Adventure cards.

**903.4f** — DEFERRED — Phase 9. Undefined color identity when no commander. Edge case for effects referencing commander color identity.

**ATOM-903.5a-001**
- **Rule:** 903.5a — Commander deck must contain exactly 100 cards including its commander.
- **Mechanism:** Commander deck validation — size
- **Minimal Board:** Deck with 100 cards (including commander). Another deck with 99 cards. Another with 101 cards.
- **Action:** Validate each deck.
- **Expected Result:** 100-card deck is valid. 99-card and 101-card decks are rejected.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander deck validation

**ATOM-903.5b-001**
- **Rule:** 903.5b — Other than basic lands, each card must have a different English name (singleton rule).
- **Mechanism:** Commander deck validation — singleton
- **Minimal Board:** Deck with two copies of a non-basic-land card. Another deck with 4 copies of a basic land.
- **Action:** Validate each deck.
- **Expected Result:** Deck with duplicate non-basic is rejected. Deck with multiple basic lands is valid.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander deck validation

> **Note:** Some cards have the ability "A deck can have any number of cards named [cardname]" (e.g., Relentless Rats, Shadowborn Apostle, Persistent Petitioners, Dragon's Approach). These override the singleton rule. The deck validator must check for this ability before rejecting duplicates. This also applies to 60-card constructed formats (rule 100.2a). Cross-ref: deck validation must query card abilities for this override.

**ATOM-903.5c-001**
- **Rule:** 903.5c — Every color in a card's color identity must be in the commander's color identity.
- **Mechanism:** Commander deck validation — color identity restriction
- **Minimal Board:** Commander has color identity {R, G}. Deck contains a card with color identity {R} (valid), a card with {U} (invalid), and a colorless card (valid).
- **Action:** Validate deck.
- **Expected Result:** The {U} card is rejected. The {R} and colorless cards are accepted.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander deck validation

**ATOM-903.5d-001**
- **Rule:** 903.5d — A card with a basic land type may be included only if each color of mana it could produce is in the commander's color identity.
- **Mechanism:** Commander deck validation — basic land type restriction
- **Minimal Board:** Commander has color identity {R, G}. Deck includes a Mountain (valid), a Forest (valid), a Plains (invalid — produces {W}).
- **Action:** Validate deck.
- **Expected Result:** Mountain and Forest accepted. Plains rejected.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander deck validation

**903.5e** — PURE-DEF. No sideboards in Commander. Affects GameConfig — `sideboard_size: None`.

> **Note:** Companions (keyword ability 702.139) are a likely exception to "no sideboards." Per 702.139a, a companion is revealed from outside the game before the game begins and can later be moved to hand by paying {3}. In Commander, 903.11a restricts cards brought from outside the game, but the Companion mechanic is specifically designed to work within Commander (the companion must still obey color identity). Cross-ref: 702.139 (Companion keyword), 903.11a (outside-game restrictions in Commander). The deck validator should allow exactly one companion declared from outside the deck, subject to its companion condition and color identity.

**ATOM-903.6-001**
- **Rule:** 903.6 — At game start, each player puts their commander from their deck face up into the command zone, then shuffles remaining 99 cards as their library.
- **Mechanism:** Commander game setup — command zone placement
- **Minimal Board:** 4-player Commander game. Each player has a 100-card deck with a designated commander.
- **Action:** Game setup.
- **Expected Result:** Each player's commander is in the command zone (face up). Each player's library contains 99 cards (shuffled). Commanders are not in the library.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander game setup

**ATOM-903.7-001**
- **Rule:** 903.7 — Each player sets life total to 40 and draws 7 cards.
- **Mechanism:** Commander starting life total
- **Minimal Board:** 4-player Commander game setup.
- **Action:** Game initialization completes.
- **Expected Result:** Each player's life total = 40. Each player's hand = 7 cards. GameConfig for Commander has `starting_life: 40`.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander GameConfig: starting_life = 40

**ATOM-903.8-001**
- **Rule:** 903.8 — A player may cast a commander they own from the command zone. The first cast has no additional tax.
- **Mechanism:** Commander casting from command zone — first cast, no tax
- **Minimal Board:** 4-player Commander game. Player A's commander has mana cost {3}{G}{G}. It is in the command zone. This is A's first time casting it from the command zone.
- **Action:** Player A casts their commander from the command zone.
- **Expected Result:** The commander moves from the command zone to the stack as a spell. Total cost = {3}{G}{G} (base cost, no tax). It resolves and enters the battlefield.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander casting from command zone

**ATOM-903.8-002**
- **Rule:** 903.8 — Commander tax: second cast from command zone costs an additional {2}.
- **Mechanism:** Commander tax — second cast incremental cost
- **Minimal Board:** Player A's commander has mana cost {3}{G}{G}. A has previously cast it from the command zone once this game. Commander is back in the command zone.
- **Action:** Player A casts their commander from the command zone.
- **Expected Result:** Total cost = {5}{G}{G} ({3}{G}{G} base + {2} tax). The tax is an additional cost, not a modification to mana cost.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander tax implementation

**ATOM-903.8-003**
- **Rule:** 903.8 — Commander tax accumulates: third cast costs additional {4}.
- **Mechanism:** Commander tax — cumulative stacking
- **Minimal Board:** Player A's commander has mana cost {1}{W}. A has cast it from the command zone twice before. Commander is in the command zone.
- **Action:** Player A casts their commander.
- **Expected Result:** Total cost = {5}{W} ({1}{W} base + {4} tax for two previous casts).
- **Phase:** Phase 9
- **Ticket:** NEW — Commander tax implementation

**ATOM-903.8-004**
- **Rule:** 903.8 — Commander tax only applies to casts FROM THE COMMAND ZONE, not from other zones.
- **Mechanism:** Commander tax — zone specificity
- **Minimal Board:** Player A's commander has been cast from the command zone once. It dies and goes to the graveyard. A chooses NOT to return it to the command zone (903.9a). A has an effect that lets them cast creatures from the graveyard.
- **Action:** Player A casts their commander from the graveyard.
- **Expected Result:** No commander tax is applied. The tax only counts "cast from the command zone." The cast-from-command-zone counter does not increment.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander tax implementation

**903.9** — PURE-DEF. Framing: commander may return to command zone.

**ATOM-903.9a-001**
- **Rule:** 903.9a — If a commander is in a graveyard or exile and was put there since the last SBA check, its owner may put it into the command zone. This is a state-based action.
- **Mechanism:** Commander return to command zone — SBA from graveyard
- **Minimal Board:** Player A's commander is on the battlefield.
- **Action:** Commander is destroyed (goes to graveyard). SBAs are checked.
- **Expected Result:** Player A is given the choice to put the commander into the command zone. If they choose yes, the commander moves from graveyard to command zone. If no, it stays in the graveyard.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander zone-return SBA. Cross-ref: T16 (SBA framework)

**ATOM-903.9a-002**
- **Rule:** 903.9a — Commander return to command zone from exile via SBA.
- **Mechanism:** Commander return to command zone — SBA from exile
- **Minimal Board:** Player A's commander is on the battlefield. Player B casts an exile effect (e.g., Swords to Plowshares) targeting it.
- **Action:** Commander is exiled. SBAs are checked.
- **Expected Result:** Player A may put their commander into the command zone from exile. This is a state-based action, not a replacement effect.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander zone-return SBA

**ATOM-903.9b-001**
- **Rule:** 903.9b — If a commander would be put into its owner's hand or library from anywhere, its owner may put it into the command zone instead (replacement effect).
- **Mechanism:** Commander return to command zone — replacement effect for hand/library
- **Minimal Board:** Player A's commander is on the battlefield. An effect would return it to A's hand (e.g., Unsummon).
- **Action:** Unsummon resolves, would put commander into A's hand.
- **Expected Result:** Player A may choose to put the commander into the command zone instead of their hand. This is a replacement effect, not an SBA. If A declines, the commander goes to hand normally.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander zone-return replacement effect

**ATOM-903.9b-002**
- **Rule:** 903.9b — Commander replacement for library tuck.
- **Mechanism:** Commander return to command zone — replacement effect for library
- **Minimal Board:** Player A's commander is on the battlefield. An effect would shuffle it into A's library (e.g., Condemn / Terminus).
- **Action:** Effect resolves, would put commander into library.
- **Expected Result:** Player A may choose to put the commander into the command zone instead of the library.
- **Phase:** Phase 9
- **Ticket:** NEW — Commander zone-return replacement effect

**903.9c** — DEFERRED — Phase 9 (stretch). Melded/merged commander zone-return. Niche interaction with meld/mutate.

**903.10** — PURE-DEF. Framing: Commander win/loss specifications.

**ATOM-903.10a-001**
- **Rule:** 903.10a — A player who's been dealt 21 or more combat damage by the same commander over the course of the game loses the game (SBA).
- **Mechanism:** Commander damage tracking — lethal threshold
- **Minimal Board:** 4-player Commander game. Player A's commander (7/7) is on the battlefield. Player B has been dealt 14 combat damage by A's commander previously.
- **Action:** A's commander deals 7 more combat damage to B.
- **Expected Result:** B has now been dealt 21 combat damage by A's commander. SBA check: B loses the game.
- **Phase:** Phase 9
- **Ticket:** T16 (Commander damage SBA). Cross-ref: T02 (player counters for commander damage tracking)

**ATOM-903.10a-002**
- **Rule:** 903.10a — Commander damage is tracked PER commander, not total across all commanders.
- **Mechanism:** Commander damage tracking — per-commander isolation
- **Minimal Board:** 4-player game. Player A's commander deals 11 combat damage to Player D. Player B's commander deals 11 combat damage to Player D.
- **Action:** SBAs are checked.
- **Expected Result:** Player D does NOT lose the game. They have 11 damage from A's commander and 11 from B's commander (22 total), but neither exceeds 21 individually.
- **Phase:** Phase 9
- **Ticket:** T16

**ATOM-903.10a-003**
- **Rule:** 903.10a — Only COMBAT damage counts for commander damage, not non-combat damage.
- **Mechanism:** Commander damage tracking — combat-only restriction
- **Minimal Board:** Player A's commander has an activated ability that deals 5 damage to target player. A's commander has dealt 18 combat damage to B previously.
- **Action:** A activates the ability targeting B, dealing 5 non-combat damage.
- **Expected Result:** B's commander damage counter from A's commander remains at 18, not 23. The 5 non-combat damage does NOT count toward the 21 threshold. B does not lose from commander damage SBA.
- **Phase:** Phase 9
- **Ticket:** T16

**ATOM-903.10a-004**
- **Rule:** 903.10a — Commander damage tracks the card, not the object. If a commander dies and is recast, damage from the new object still accumulates.
- **Mechanism:** Commander damage tracking — persistence across zone changes
- **Minimal Board:** Player A's commander (5/5) deals 10 combat damage to B. Commander dies, returns to command zone, is recast.
- **Action:** A's commander (recast, same card) deals 11 more combat damage to B.
- **Expected Result:** B has been dealt 21 total combat damage by A's commander (the card). SBA: B loses the game. The zone change did not reset the counter.
- **Phase:** Phase 9
- **Ticket:** T16

**ATOM-903.10a-005**
- **Rule:** 903.10a + 702.124 (Partner) — Commander damage is tracked per commander card. With Partner, a player has two commanders, and damage from each is tracked separately.
- **Mechanism:** Commander damage tracking — Partner commanders tracked independently
- **Minimal Board:** 4-player Commander game. Player A has two Partner commanders: Commander X (4/4) and Commander Y (3/3). Both are on the battlefield.
- **Action:** Commander X deals 11 combat damage to Player B. Commander Y deals 11 combat damage to Player B. SBAs are checked.
- **Expected Result:** Player B does NOT lose the game. B has 11 damage from Commander X and 11 from Commander Y. Neither individual commander has dealt 21. B would only lose if one of them individually reaches 21.
- **Phase:** Phase 9
- **Ticket:** T16 + NEW-S10-10

**903.11** — DEFERRED — Phase 9. Restrictions on bringing cards from outside the game. Low priority.

**903.11a** — DEFERRED — Phase 9. Additional restrictions on wish effects in Commander. Low priority.

### 903.12. Brawl Option

**903.12a–903.12h** (all sub-rules) — DEFERRED — Phase 9 (stretch). Brawl is an alternative Commander option with different deck size (60), life totals (25/30), and no commander damage. Could be implemented as a GameConfig variant after base Commander. Sub-rules: 903.12a, 903.12b, 903.12c, 903.12d, 903.12e, 903.12f, 903.12g, 903.12h.

### 903.13. Commander Draft

**903.13a–903.13g** (all sub-rules) — DEFERRED — Future. Draft infrastructure requires a working engine first; not permanently excluded, but no timeline. Sub-rules: 903.13a, 903.13b, 903.13c, 903.13d, 903.13e, 903.13f, 903.13g.

### 904. Archenemy

**904.1–904.13d** (all sub-rules) — OUT-OF-SCOPE. Archenemy is permanently excluded. Sub-rules: 904.1, 904.2, 904.2a, 904.2b, 904.3, 904.4, 904.5, 904.6, 904.7, 904.8, 904.9, 904.10, 904.11, 904.12, 904.12a, 904.12b, 904.12c, 904.13, 904.13a, 904.13b, 904.13c, 904.13d.

### 905. Conspiracy Draft

**905.1–905.6** (all sub-rules) — OUT-OF-SCOPE. Conspiracy Draft is permanently excluded. Sub-rules: 905.1, 905.1a, 905.1b, 905.1c, 905.1d, 905.2, 905.2a, 905.2b, 905.2c, 905.3, 905.4, 905.4a, 905.5, 905.6.

--- End of Chunk 2 ---

## Chunk 3: Classification Summary, Composition Tests, Gap Report

### Composition Tests

**COMP-903-FULL-GAME-001**
- **Rule:** 903.6 + 903.7 + 903.8 + 903.10a — Full Commander game lifecycle: setup → cast commander → deal commander damage → win condition
- **Mechanism:** End-to-end Commander game flow
- **Minimal Board:** 4-player Commander game. Player A's commander is a 7/7 legendary creature with mana cost {5}{G}{G}.
- **Action:** Game starts (903.6: commander to command zone, 903.7: life = 40). A casts commander (903.8: from command zone). Over multiple combats, A's commander deals 21+ combat damage to B.
- **Expected Result:** B loses to commander damage SBA (903.10a). A's commander was cast with no tax the first time (903.8). Game setup correctly placed commander in command zone and set life to 40.
- **Composes:** ATOM-903.6-001, ATOM-903.7-001, ATOM-903.8-001, ATOM-903.10a-001
- **Phase:** Phase 9
- **Ticket:** D7 + T16

**COMP-903-TAX-AND-RETURN-001**
- **Rule:** 903.8 + 903.9a + 903.9b — Commander dies, returns to command zone, recast with tax
- **Mechanism:** Commander death → SBA return → recast with tax accumulation
- **Minimal Board:** Player A's commander (mana cost {2}{R}) is on the battlefield. It has been cast from command zone once before.
- **Action:** Commander is destroyed → goes to graveyard → SBA: A chooses to put it into command zone (903.9a) → A recasts it from command zone (903.8).
- **Expected Result:** Recast cost = {4}{R} ({2}{R} base + {2} tax for one previous command zone cast). Commander tax counter increments to 2.
- **Composes:** ATOM-903.8-002, ATOM-903.9a-001
- **Phase:** Phase 9
- **Ticket:** NEW — Commander tax + zone-return integration

**COMP-903-BOUNCE-REPLACEMENT-001**
- **Rule:** 903.9b + 903.8 — Commander bounced to hand, replacement effect redirects to command zone, recast with tax
- **Mechanism:** Commander replacement effect for hand → command zone, then recast
- **Minimal Board:** Player A's commander (mana cost {1}{U}) is on the battlefield. Cast from command zone once before.
- **Action:** Opponent casts Unsummon targeting A's commander. A chooses to put it into command zone instead of hand (903.9b). A recasts from command zone.
- **Expected Result:** Commander goes to command zone (not hand). Recast cost = {3}{U} ({1}{U} + {2} tax). Tax counter increments.
- **Composes:** ATOM-903.9b-001, ATOM-903.8-002
- **Phase:** Phase 9
- **Ticket:** NEW — Commander replacement + tax integration

**COMP-800-PLAYER-LEAVES-COMMANDER-001**
- **Rule:** 800.4a + 903.10a — Player leaves Commander game, cleanup interacts with commander damage tracking
- **Mechanism:** Player-leaves-game cleanup in Commander context
- **Minimal Board:** 4-player Commander game (A, B, C, D). A has dealt 15 commander damage to B. A has dealt 10 commander damage to C.
- **Action:** Player A leaves the game.
- **Expected Result:** A's objects leave the game (800.4a). Commander damage counters from A's commander persist on B and C (they were already dealt). However, since A's commander no longer exists, no further commander damage can accumulate from it. B (at 15) and C (at 10) do not lose from commander damage.
- **Composes:** ATOM-800.4a-001, ATOM-903.10a-001
- **Phase:** Phase 9
- **Ticket:** D24

### Gap Report

| # | Gap | Description | Recommendation |
|---|-----|-------------|----------------|
| G1 | **Partner commanders** | Rule 702.124 (Partner) allows two commanders. Not covered in 903.x directly but referenced in 903.13f. Affects commander tax (tracked per commander), color identity (union of both), deck size (still 100), command zone (both start there). | NEW ticket for Phase 9: Partner commander support |
| G2 | **Commander as DFC** | 903.4d covers color identity for DFC commanders. But casting a DFC commander from command zone and which face it enters as needs testing. | Add ATOM test when DFC casting is implemented (Phase 8/9) |
| G3 | **Commander damage + damage prevention** | If combat damage from a commander is prevented, does it count toward the 21 threshold? Per CR, prevented damage is never dealt, so no. Needs explicit test. | NEW: ATOM-903.10a-006 — Prevented commander combat damage does not count |
| G4 | **Commander damage + damage redirection** | If a commander's combat damage is redirected to a different player (e.g., via Deflecting Palm), does the redirected damage count as commander damage to the new recipient? Per CR, redirected damage is from the same source, so yes. | NEW: ATOM-903.10a-007 — Redirected commander combat damage counts |
| G5 | **Multiplayer priority/turn order** | APNAP for 4 players (101.4 from Session 1) is used extensively in 802.x but the core APNAP implementation for >2 players is a Phase 9 prerequisite not explicitly ticketed. | Ensure D7 (Multiplayer systems) covers APNAP for N players |
| G6 | **Commander + continuous effects interaction** | If a commander's P/T is modified by a continuous effect (e.g., +3/+3 from an Aura), the modified power determines combat damage dealt, which affects commander damage tracking. This is a cross-phase dependency (Phase 5 layers + Phase 9 commander damage). | Cross-ref: Phase 5 continuous effects must feed into Phase 9 commander damage calculation |

### Classification Summary Table

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

### NEW Tickets

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

### ATOM Test Count Summary

| Section | ATOM Tests | COMP Tests | DEFERRED | OUT-OF-SCOPE | PURE-DEF |
|---------|-----------|------------|----------|-------------|----------|
| 800.x | 9 | 1 | 7 | 2 | 5 |
| 801.x | 0 | 0 | 28 | 0 | 0 |
| 802.x | 7 | 0 | 3 | 1 | 1 |
| 803.x | 0 | 0 | 0 | 3 | 0 |
| 804.x | 0 | 0 | 0 | 2 | 0 |
| 805.x | 0 | 0 | 0 | 25 | 0 |
| 806.x | 0 | 0 | 0 | 0 | 6 |
| 807.x | 0 | 0 | 0 | 20 | 0 |
| 808.x | 0 | 0 | 0 | 7 | 0 |
| 809.x | 0 | 0 | 0 | 14 | 0 |
| 810.x | 0 | 0 | 0 | 27 | 0 |
| 811.x | 0 | 0 | 0 | 8 | 0 |
| 900.x | 0 | 0 | 0 | 0 | 2 |
| 901.x | 0 | 0 | 0 | 35 | 0 |
| 902.x | 0 | 0 | 9 | 0 | 0 |
| 903.x | 35 | 3 | 18 | 0 | 7 |
| 904.x | 0 | 0 | 0 | 22 | 0 |
| 905.x | 0 | 0 | 0 | 14 | 0 |
| **Total** | **51** | **4** | **65** | **180** | **21** |

--- End of Chunk 3 (Final) ---

## Audit Response Log

**Audit date:** 2026-04-09

| # | Feedback | Action Taken |
|---|----------|-------------|
| A1 | 800.4: Explain DEFERRED vs TESTABLE rationale | Added audit note block after 800.4 explaining observable-state-transform (TESTABLE) vs edge-case-resolution-policy (DEFERRED with prerequisite dependencies) |
| A2 | 802.2a: Make standalone tests, rule is verbose not complex | Reclassified TESTABLE. Added ATOM-802.2a-001 (per-creature "defending player") and ATOM-802.2a-002 (general "defending player" = any attacked player) |
| A3 | 806: Skip FFA wrapper, implement Commander directly | Removed ATOM-806.2-001. Reclassified 806.2/2a/2b/2c as PURE-DEF with note that FFA defaults are tested via ATOM-903.2-001 |
| A4 | 810: OUT-OF-SCOPE, team shared payments too difficult | Changed from DEFERRED to OUT-OF-SCOPE with rationale |
| A5 | 903.1: Design note — generic vs standard commander tiers | Added design note about two-tier commander validation (generic for alt formats, standard for 903.3) |
| A6 | 903.3: Test face-down commander example | Added ATOM-903.3-003 (face-down permanent retains commander designation) |
| A7 | 903.3d: Has mechanical consequences, not PURE-DEF | Reclassified TESTABLE. Added ATOM-903.3d-001 ("control your commander" true on battlefield), -002 (false in command zone), -003 ("target commander" targeting) |
| A8 | 903.5b: Note "any number of copies" cards | Added note about Relentless Rats, Shadowborn Apostle, etc. overriding singleton rule. Also applies to 60-card formats |
| A9 | 903.5e: Companion exception to no sideboards | Added note cross-referencing 702.139 (Companion) and 903.11a. Deck validator should allow one companion outside deck |
| A10 | 903.8: Tests 001/002 identical, remove one | Merged 001+002 into single test (cast from CZ, first cast = no tax). Renumbered remaining: old 003→002, 004→003, 005→004 |
| A11 | 903.10a: Test Partner interaction | Added ATOM-903.10a-005 (Partner commanders tracked independently for commander damage) |
| A12 | 903.13: Not permanently OOS, just needs engine first | Changed from OUT-OF-SCOPE to DEFERRED — Future |
