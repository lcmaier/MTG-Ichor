# Session 1: Chapter 1 — Game Concepts (Rules 100–123)

> Generated: 2026-04-02
> CR Source: Chapter 1 — Game Concepts
> Scope: 24 top-level rule sections (100–123), ~350 sub-rules
> Session covers: foundational game concepts, mana, colors, objects, permanents, tokens, spells, abilities, targets, priority, costs, life, damage, drawing, counters, stickers

---

## Rules 100–101: General + Golden Rules

### 100. General

**100.1** — PURE-DEF. Defines scope of rules (two or more players). No independent mechanical consequence.

**100.1a** — PURE-DEF. Defines "two-player game." Naming only.

**100.1b** — PURE-DEF. Defines "multiplayer game." Naming only.

**100.2** — PURE-DEF. Physical requirements for play. Not engine-relevant.

**100.2a** — TESTABLE. Constructed deck minimum 60 cards, max 4 copies of non-basic-land cards.

**ATOM-100.2a-001**
- **Rule:** 100.2a — Constructed deck minimum size is 60 cards
- **Mechanism:** Deck validation in `GameConfig` / deck construction
- **Minimal Board:** A decklist with 59 cards
- **Action:** Validate the decklist against `GameConfig::standard()`
- **Expected Result:** Validation fails with "deck too small" error
- **Phase:** Phase 5-Pre (already in `GameConfig` via `DeckLimits`)
- **Ticket:** Already covered by `GameConfig::standard()` preset — verify `DeckLimits.min_deck_size == 60`

**ATOM-100.2a-002**
- **Rule:** 100.2a — No more than 4 copies of a non-basic-land card in constructed
- **Mechanism:** Deck validation copy-count check
- **Minimal Board:** A decklist with 5 copies of Lightning Bolt
- **Action:** Validate the decklist against `GameConfig::standard()`
- **Expected Result:** Validation fails with "too many copies" error
- **Phase:** Phase 5-Pre (already in `GameConfig` via `DeckLimits.max_copies`)
- **Ticket:** Already covered by `GameConfig::standard()` — verify `max_copies == Some(4)`

**ATOM-100.2a-003**
- **Rule:** 100.2a — Any number of basic land cards allowed in constructed
- **Mechanism:** Deck validation basic-land exemption
- **Minimal Board:** A decklist with 40 Mountains (a basic land)
- **Action:** Validate the decklist against `GameConfig::standard()`
- **Expected Result:** Validation passes (basic lands exempt from copy limit)
- **Phase:** Phase 5-Pre
- **Ticket:** Already covered by `GameConfig` — verify basic land exemption in deck validation

**100.2b** — TESTABLE. Limited deck minimum 40 cards, unlimited duplicates.

**ATOM-100.2b-001**
- **Rule:** 100.2b — Limited deck minimum size is 40 cards
- **Mechanism:** Deck validation in `GameConfig::limited()`
- **Minimal Board:** A decklist with 39 cards
- **Action:** Validate against `GameConfig::limited()`
- **Expected Result:** Validation fails
- **Phase:** Phase 5-Pre
- **Ticket:** Already covered by `GameConfig::limited()` — verify `min_deck_size == 40` and `max_copies == None`

**ATOM-100.2b-002**
- **Rule:** 100.2b — Limited format does NOT enforce the 4-copy limit from constructed
- **Mechanism:** Deck validation copy-count check with `GameConfig::limited()`
- **Minimal Board:** A limited decklist with 5 copies of the same non-basic-land card
- **Action:** Validate against `GameConfig::limited()`
- **Expected Result:** Validation passes (limited has no copy limit)
- **Phase:** Phase 5-Pre
- **Ticket:** Already covered by `GameConfig::limited()` — verify `max_copies == None`

**100.2c** — DEFERRED. Commander deckbuilding. Deferred to Phase 9.

**100.2d** — OUT-OF-SCOPE. Supplementary decks (Attractions, Planechase, Archenemy).

**100.3** — DEFERRED. Coins/dice mechanics. Rules 705 (flipping coins) and 706 (rolling dice) are engine-relevant for cards like Mana Crypt, Krark's Thumb, etc. Tests deferred to Chapter 7 session covering 705/706.

**100.4** — PURE-DEF. Sideboard concept definition.

**100.4a** — TESTABLE. Constructed sideboard max 15 cards, 4-card limit applies to combined deck + sideboard.

**ATOM-100.4a-001**
- **Rule:** 100.4a — Constructed sideboard max 15 cards
- **Mechanism:** Deck validation sideboard size check
- **Minimal Board:** A decklist with 16 sideboard cards
- **Action:** Validate against `GameConfig::standard()`
- **Expected Result:** Validation fails
- **Phase:** Phase 5-Pre
- **Ticket:** Already covered by `GameConfig` — verify `sideboard_size == Some(15)`

**ATOM-100.4a-002**
- **Rule:** 100.4a — 4-copy limit applies across main deck AND sideboard combined
- **Mechanism:** Deck validation counts copies across main + sideboard
- **Minimal Board:** A decklist with 3 copies of Lightning Bolt in main and 2 in sideboard (5 total)
- **Action:** Validate against `GameConfig::standard()`
- **Expected Result:** Validation fails — 5 total copies exceeds 4-copy limit
- **Phase:** Phase 5-Pre
- **Ticket:** NEW — Combined main+sideboard copy-count validation

**100.4b** — DEFERRED/BOUNDARY-DEF. Limited play is in scope (sealed/draft). Deck validation for limited (40-card minimum, no copy limit) is testable. Sideboard-swap workflow ("exchange cards between main deck and sideboard between games") is a match-level concern deferred to match management implementation.

**100.4c–d** — OUT-OF-SCOPE. Team variant sideboard rules.

**100.5** — PURE-DEF. Defines "minimum deck size" term. No max for non-Commander.

**100.6–100.7** — OUT-OF-SCOPE. Tournament rules, casual/Un-set rules.

---

### 101. The Magic Golden Rules

**101.1** — META. Card text overrides rules.

**META-101.1:** Card text directly contradicts rules → card takes precedence.
- **Expected systems:** Spell resolution, continuous effects, static abilities, keyword abilities, replacement effects, triggered abilities
- **Concrete tests deferred to:** Sessions covering rules 604 (static abilities), 609 (effects), 613 (continuous effects), 614 (replacement effects), specific keyword rules (702.x)
- **Note:** Card-specific rule-override tests belong with the card/keyword in question, not in a "full game rules" suite. The real risk is cards that require architectural changes to support — these are unpredictable and often arise from WotC design mistakes. The engine's mechanism for card text overriding rules is the `Effect` resolution system taking priority over hardcoded behavior.

**101.2** — META. "Can't" overrides "can."

**META-101.2:** When one effect allows something and another says it can't happen, the "can't" effect wins.
- **Expected systems:** Combat (blocking restrictions vs. requirements), targeting (hexproof/shroud vs. "target any"), mana (spending restrictions), life gain/loss ("can't gain life"), damage prevention, land plays ("can't play lands"), casting restrictions, ability activation
- **Concrete tests deferred to:** Sessions covering 508/509 (combat), 115 (targeting), 119 (life), 120 (damage), 305 (lands), 601 (casting), 602 (activation)
- **Note:** The design doc notes D9 says this is "already embedded as design pattern." Each system must still have at least one concrete test.

**101.2a** — TESTABLE. Adding/removing abilities is NOT a "can't overrides can" situation.

**ATOM-101.2a-001**
- **Rule:** 101.2a — Adding abilities and removing abilities don't fall under the "can't overrides can" rule
- **Mechanism:** Layer 6 (ability add/remove) interaction — most recent effect prevails per rule 613, not "can't wins"
- **Minimal Board:** Creature with flying. Effect A removes flying. Effect B (later timestamp) adds flying.
- **Action:** Compute effective abilities
- **Expected Result:** Creature has flying (later timestamp prevails, NOT "remove wins because it's a can't")
- **Phase:** Phase 5 Layers (L06 — Layer 6 abilities)
- **Ticket:** L06

**101.3** — META. Impossible instructions are ignored.

**META-101.3:** Any part of an instruction that's impossible to perform is simply ignored.
- **Expected systems:** Effect resolution (draw from empty library as part of a multi-draw, sacrifice when you control no creatures, etc.), cost payment, targeting
- **Concrete tests deferred to:** Sessions covering 608 (resolving), 701 (keyword actions)

**101.4** — TESTABLE. APNAP order for simultaneous choices.

**ATOM-101.4-001**
- **Rule:** 101.4 — Active player makes choices first, then each nonactive player in turn order, then actions happen simultaneously
- **Mechanism:** APNAP ordering in engine choice resolution
- **Minimal Board:** 2-player game. An effect says "each player sacrifices a creature." P0 (active) controls Creature A. P1 controls Creature B.
- **Action:** Resolve the effect
- **Expected Result:** P0 chooses first, then P1 chooses. Both sacrifices happen simultaneously.
- **Phase:** Phase 7 (triggered abilities with simultaneous resolution) / Phase 8 (mass effect primitives)
- **Ticket:** NEW — APNAP ordering for simultaneous player choices

**101.4a–b** — PURE-DEF. Elaborations on APNAP (hidden zone face-down, knowledge of previous choices).

**101.4c** — TESTABLE. Sequential instruction ordering within a single effect's resolution — distinct from APNAP. Tests that the effect resolver serves choices to DecisionProviders in the order written on the card, applies APNAP within each step, and doesn't batch or reorder steps.

**ATOM-101.4c-001**
- **Rule:** 101.4c — Simultaneous choices are made in the order written, with APNAP within each step
- **Mechanism:** Sequential resolution ordering in `resolve_effect()` for multi-step instructions
- **Minimal Board:** 2-player game. Smallpox resolves: "Each player loses 1 life, discards a card, sacrifices a creature, then sacrifices a land." P0 (active) controls a creature and a land. P1 controls a creature and a land.
- **Action:** Resolve Smallpox
- **Expected Result:** Steps execute in written order: (1) both lose 1 life, (2) P0 chooses discard first (APNAP), then P1, (3) P0 chooses sacrifice creature first, then P1, (4) P0 chooses sacrifice land first, then P1. Steps are NOT batched or reordered.
- **Phase:** Phase 8 (mass effect primitives with sequential resolution)
- **Ticket:** NEW — Sequential instruction ordering for multi-step effects

**101.4d** — PURE-DEF. Historical catchall from the damage-redirection-to-planeswalkers era. "If there's no obvious ordering, the controller of the spell/ability on the stack chooses." No known modern application. Flag for re-examination if a card triggers this.

**101.4e** — PURE-DEF. Pregame APNAP ordering. Naming.

---

## Rules 102–104: Players, Starting the Game, Ending the Game

### 102. Players

**102.1** — PURE-DEF. Defines "player," "active player," "nonactive player."

**102.2** — PURE-DEF. In a two-player game, the other player is the opponent.

**102.3–102.4** — OUT-OF-SCOPE. Multiplayer teams. Two-Headed Giant out of scope.

---

### 103. Starting the Game

**103.1** — PURE-DEF. Starting player determination. Engine uses `GameConfig` to set starting player; the method of choosing is outside the engine.

**103.1a–c** — OUT-OF-SCOPE. Shared team turns, Archenemy, Power Play card.

**103.2** — PURE-DEF. Lists additional pregame steps (applied in order). Framework only.

**103.2a** — PURE-DEF. Sideboard/substitute cards set aside. Naming.

**103.2b** — DEFERRED. Companion reveal. Deferred (D20).

**103.2c** — DEFERRED. Commander setup. Phase 9.

**103.2d** — OUT-OF-SCOPE. Sticker sheets. Un-set mechanic.

**103.2e** — OUT-OF-SCOPE. Conspiracy Draft.

**103.3** — TESTABLE. Each player shuffles their deck; decks become libraries.

**ATOM-103.3-001**
- **Rule:** 103.3 — Players shuffle their decks to random order; decks become libraries
- **Mechanism:** `Game::setup()` shuffles decklists into libraries
- **Minimal Board:** A game with two players, each with a 60-card deck
- **Action:** Call `Game::setup()`
- **Expected Result:** Each player's library has exactly the same cards as their decklist (in some order). Library order differs from original order (probabilistic — verify with a large enough deck that shuffle is not identity).
- **Phase:** Already implemented in `Game::setup()`
- **Ticket:** ALREADY-IMPLEMENTED

**103.3a** — OUT-OF-SCOPE. Supplementary deck shuffle.

**103.4** — TESTABLE. Starting life total is 20.

**ATOM-103.4-001**
- **Rule:** 103.4 — Each player begins the game with a starting life total of 20
- **Mechanism:** `GameConfig.starting_life` initialization
- **Minimal Board:** A new game with `GameConfig::standard()`
- **Action:** Check player life totals after `Game::setup()`
- **Expected Result:** Both players have life total == 20
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**103.4a–e** — DEFERRED. Variant game life totals (Two-Headed Giant, Vanguard, Commander, Brawl, Archenemy). Commander/Brawl life (103.4c) is Phase 9.

**103.5** — TESTABLE. London mulligan procedure.

**ATOM-103.5-001**
- **Rule:** 103.5 — Each player draws starting hand size (7) cards, may mulligan (London mulligan)
- **Mechanism:** `Game::setup()` mulligan handling
- **Minimal Board:** A new 2-player game
- **Action:** Player takes one mulligan. They draw 7, put 1 on bottom of library.
- **Expected Result:** After one mulligan, player has 6 cards in hand, 1 card on bottom of library. Library size is deck_size - 7.
- **Phase:** Phase 9 (D12 — mulligan implementation currently stubbed)
- **Ticket:** D12 — Mulligan implementation

**103.5a–d** — DEFERRED. Vanguard hand modifier, "any time could mulligan" actions, multiplayer first-mulligan-free rule, shared team turns mulligan. Deferred.

**103.5c** — DEFERRED. Multiplayer first mulligan free.

**103.6** — TESTABLE. Pregame actions from opening hand.

**ATOM-103.6-001**
- **Rule:** 103.6 — After mulligans, starting player may take opening-hand actions (e.g., Leylines), then each other player in turn order
- **Mechanism:** Pregame action hook in `Game::setup()`
- **Minimal Board:** Player has a Leyline card in opening hand
- **Action:** Run pregame action phase
- **Expected Result:** Leyline enters the battlefield with `controller_since_turn = 0` (pregame sentinel). The permanent is NOT summoning sick on turn 1 (per T09 convention: `0 >= 1` is false).
- **Phase:** Phase 8 (D27)
- **Ticket:** D27 — Pregame actions (103.6)

**103.6a–c** — Elaborations on 103.6. Covered by ATOM-103.6-001.

**103.7** — OUT-OF-SCOPE. Planechase starting plane.

**103.8** — PURE-DEF. Starting player takes first turn.

**103.8a** — TESTABLE. In a two-player game, the starting player skips their first draw step.

**ATOM-103.8a-001**
- **Rule:** 103.8a — Starting player skips draw step of first turn in two-player game
- **Mechanism:** `skip_first_draw` flag on `GameState`
- **Minimal Board:** A new 2-player game with `GameConfig` where `first_player_draws == false`
- **Action:** Run first turn. Check if draw step was skipped.
- **Expected Result:** Starting player did not draw a card on turn 1. They draw normally on turn 2.
- **Phase:** Already implemented (Phase 1 / Pre-Phase 3)
- **Ticket:** ALREADY-IMPLEMENTED

**103.8b** — OUT-OF-SCOPE. Two-Headed Giant first draw skip.

**103.8c** — TESTABLE. In multiplayer (non-2HG), no player skips first draw.

**ATOM-103.8c-001**
- **Rule:** 103.8c — In multiplayer games (not 2HG), no player skips the draw step of their first turn
- **Mechanism:** `GameConfig.first_player_draws` setting for multiplayer
- **Minimal Board:** A 3+ player game
- **Action:** Run first turn of starting player
- **Expected Result:** Starting player draws a card on their first draw step
- **Phase:** Phase 9 (multiplayer support)
- **Ticket:** NEW — Multiplayer first-draw-not-skipped config

---

### 104. Ending the Game

**104.1** — PURE-DEF. Game ends on win, draw, or restart. Framework statement.

**104.2** — PURE-DEF. Introduces ways to win.

**104.2a** — TESTABLE. Last player standing wins.

**ATOM-104.2a-001**
- **Rule:** 104.2a — A player wins if all opponents have left the game
- **Mechanism:** `Game::check_game_over()` checks `player_lost` flags
- **Minimal Board:** 2-player game. P1 has `player_lost == true`.
- **Action:** Call `check_game_over()`
- **Expected Result:** Returns `GameResult::Winner(P0)`
- **Phase:** Already implemented (Pre-Phase 3)
- **Ticket:** ALREADY-IMPLEMENTED

**104.2b** — TESTABLE. An effect may state a player wins.

**ATOM-104.2b-001**
- **Rule:** 104.2b — An effect may state that a player wins the game
- **Mechanism:** `Primitive::WinTheGame` or equivalent in effect resolution
- **Minimal Board:** A spell on the stack with effect "You win the game"
- **Action:** Resolve the spell
- **Expected Result:** That player wins immediately. `Game.result == GameResult::Winner(controller)`
- **Phase:** Phase 8 (remaining primitives)
- **Ticket:** NEW — WinTheGame / LoseTheGame primitives

**104.2c–d** — OUT-OF-SCOPE. Multiplayer team wins, Emperor variant.

**104.3** — PURE-DEF. Introduces ways to lose.

**104.3a** — TESTABLE. A player can concede at any time.

**ATOM-104.3a-001**
- **Rule:** 104.3a — A player can concede at any time; they immediately leave and lose
- **Mechanism:** Deferred concession model — `pending_concession` flag set immediately, processed at next safe point (before priority is given, similar to SBA timing). In 2-player, game ends immediately. In multiplayer, conceding player's permanents/effects cleaned up per 800.4.
- **Minimal Board:** A game in progress
- **Action:** Player concedes during opponent's turn
- **Expected Result:** That player's `player_lost` is set to true at next safe point. Game ends with opponent winning.
- **Phase:** Phase 8 or Phase 9
- **Ticket:** NEW — Concession action (deferred concession model)
- **Note:** Slightly imprecise vs. rules (rules say "immediately") but architecturally necessary — can't safely mutate game state mid-resolution. The window between `pending_concession` and processing is at most one resolution/SBA cycle.
- **Architectural note (concession model):** Even if a player has fully disconnected (extreme case in networked play), we inject the `PlayerLost` event into the SBA queue and treat that player's objects as active until processed. The cleanup path (800.4a in multiplayer: owned objects leave, control effects end, controlled-but-not-owned objects exiled) runs as part of normal SBA processing, not as a special teardown. XMage uses a similar pattern: `PlayerImpl.concede()` sets a `hasLeft` flag, and `GameImpl.leave()` runs synchronously during the next SBA check cycle — the network layer handles disconnect separately from the game rules layer. Our `pending_concession` → SBA processing design mirrors this separation.

**104.3b** — TESTABLE. Life ≤ 0 → lose as SBA.

**ATOM-104.3b-001**
- **Rule:** 104.3b — Player with 0 or less life loses the game (SBA)
- **Mechanism:** SBA check in `sba.rs` — 704.5a
- **Minimal Board:** Player at 0 life
- **Action:** Check SBAs
- **Expected Result:** Player's `player_lost` flag is set. `LossReason::LifeReachedZero`.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**104.3c** — TESTABLE. Drawing from empty library → lose as SBA.

**ATOM-104.3c-001**
- **Rule:** 104.3c — Player required to draw from empty library → lose as SBA
- **Mechanism:** `draw_card` returns `Ok(None)` on empty library, SBA flags loss
- **Minimal Board:** Player with 0 cards in library
- **Action:** Attempt to draw a card. Check SBAs.
- **Expected Result:** `player_lost` flag set. `LossReason::DrawnFromEmptyLibrary`.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-104.3c-002**
- **Rule:** 104.3c — Player required to draw more cards than in library draws remaining, then loses
- **Mechanism:** `draw_cards(n)` calls `draw_card()` N times; partial success then failure
- **Minimal Board:** Player with 2 cards in library. Effect says "draw 3 cards."
- **Action:** Resolve the effect. Check SBAs.
- **Expected Result:** Player draws 2 cards successfully. Third draw fails (empty library). SBA flags loss. Player has 2 new cards in hand.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**104.3d** — TESTABLE. 10+ poison counters → lose as SBA.

**ATOM-104.3d-001**
- **Rule:** 104.3d — Player with 10+ poison counters loses (SBA)
- **Mechanism:** SBA check for poison counters (704.5c)
- **Minimal Board:** Player with 10 poison counters
- **Action:** Check SBAs
- **Expected Result:** `player_lost` flag set. `LossReason::PoisonCounters`.
- **Phase:** Phase 5-Pre (T16)
- **Ticket:** T16

**104.3e** — TESTABLE. An effect may state a player loses.

**ATOM-104.3e-001**
- **Rule:** 104.3e — An effect may state that a player loses the game
- **Mechanism:** `Primitive::LoseTheGame` or equivalent
- **Minimal Board:** A spell on the stack with effect "Target player loses the game"
- **Action:** Resolve the spell
- **Expected Result:** That player's `player_lost` flag is set immediately.
- **Phase:** Phase 8 (remaining primitives)
- **Ticket:** NEW — WinTheGame / LoseTheGame primitives (same ticket as 104.2b)

**104.3f** — PURE-DEF. Simultaneous win + lose → player loses. Theoretical catchall — no known cards produce this situation. The rule exists for completeness in the CR.



**104.3g–k** — OUT-OF-SCOPE. Multiplayer team losses, limited range of influence, Emperor, Commander combat damage (104.3j is Phase 9 via T16/T02), tournament penalties.

**104.3j** — TESTABLE but Phase 9. Commander damage 21+ from same commander.

**ATOM-104.3j-001**
- **Rule:** 104.3j — Player dealt 21+ combat damage by same commander loses (SBA)
- **Mechanism:** SBA check for commander damage (704.5 commander)
- **Minimal Board:** Player with `commander_damage_taken[cmd_id] == 21`
- **Action:** Check SBAs
- **Expected Result:** `player_lost` flag set. `LossReason::CommanderDamage`.
- **Phase:** Phase 5-Pre (T16) data model, Phase 9 (Commander format)
- **Ticket:** T16 (SBA) + Phase 9 format

**104.4** — PURE-DEF. Introduces draw conditions.

**104.4a** — TESTABLE. All remaining players lose simultaneously → draw.

**ATOM-104.4a-001**
- **Rule:** 104.4a — If all remaining players lose simultaneously, the game is a draw
- **Mechanism:** `check_game_over()` when all `player_lost` flags are true
- **Minimal Board:** 2-player game. Both players at 0 life (e.g., from an effect dealing lethal to both).
- **Action:** SBAs set both `player_lost` flags. `check_game_over()` runs.
- **Expected Result:** `GameResult::Draw`
- **Phase:** Already implemented (Pre-Phase 3)
- **Ticket:** ALREADY-IMPLEMENTED

**104.4b** — TESTABLE. Mandatory loop with no way to stop → draw. (Deferred: D11)

**ATOM-104.4b-001**
- **Rule:** 104.4b — Game enters mandatory loop with no way to stop → draw
- **Mechanism:** Loop detection in game loop
- **Minimal Board:** Two mandatory triggered abilities that trigger each other infinitely
- **Action:** Run game loop
- **Expected Result:** Game detects the mandatory loop and declares a draw
- **Phase:** Post-v1 (D11 — mandatory loop detection)
- **Ticket:** D11

**104.4c** — TESTABLE. An effect may state the game is a draw.

**ATOM-104.4c-001**
- **Rule:** 104.4c — An effect may state the game is a draw
- **Mechanism:** `Primitive::DrawTheGame` or equivalent
- **Minimal Board:** Spell resolving with "the game is a draw"
- **Action:** Resolve spell
- **Expected Result:** `GameResult::Draw`
- **Phase:** Phase 8
- **Ticket:** NEW — DrawTheGame primitive

**104.4d–i** — OUT-OF-SCOPE. Multiplayer team draws, limited range of influence, Emperor, intentional draws.

**104.5** — PURE-DEF. Losing/drawing player leaves the game. Multiplayer rules handle details.

**104.6** — DEFERRED. Karn Liberated restarts the game. Saw extensive competitive play and the mechanic could appear on future cards. Retrofit cost is moderate — `Game::new()` already encapsulates setup, so a "restart" could create a fresh `GameState` with modified starting conditions (exiled cards become starting hands). Deferred to Phase 9+.

---

## Rules 105–107: Colors, Mana, Numbers and Symbols

### 105. Colors

**105.1** — BOUNDARY-DEF. Five colors: white, blue, black, red, green.

**ATOM-105.1-001**
- **Rule:** 105.1 — There are exactly five colors in Magic
- **Mechanism:** `Color` enum completeness
- **Minimal Board:** N/A (type system check)
- **Action:** Verify `Color` enum has exactly 5 variants: White, Blue, Black, Red, Green
- **Expected Result:** Enum has exactly these 5 variants. No "Colorless" variant in Color (colorless is NOT a color).
- **Phase:** Already implemented in `types/`
- **Ticket:** ALREADY-IMPLEMENTED

**105.2** — TESTABLE. An object's color is defined by mana symbols in its mana cost, color indicator, or CDA.

**ATOM-105.2-001**
- **Rule:** 105.2 — An object is the color(s) of the mana symbols in its mana cost
- **Mechanism:** Color derivation from `ManaCost`
- **Minimal Board:** A card with mana cost {1}{R}{G}
- **Action:** Query the card's colors
- **Expected Result:** Card is Red and Green (multicolored)
- **Phase:** Phase 5 Layers (L10 — Layer 5 color)
- **Ticket:** L10

**ATOM-105.2-002**
- **Rule:** 105.2 — A card with no colored mana symbols and no color indicator is colorless
- **Mechanism:** Color derivation from `ManaCost`
- **Minimal Board:** A card with mana cost {4} and no color indicator
- **Action:** Query the card's colors
- **Expected Result:** Card is colorless (empty color set)
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

**ATOM-105.2-003**
- **Rule:** 105.2 — A card's color may be defined by a color indicator
- **Mechanism:** `color_indicator` field on `CardData` overrides mana-cost-derived color
- **Minimal Board:** A card with no mana cost but `color_indicator: Some(vec![Green])`
- **Action:** Query the card's colors
- **Expected Result:** Card is Green
- **Phase:** Phase 5-Pre (T05 adds field) + Phase 5 Layers (L10 reads it)
- **Ticket:** T05 + L10

**105.2** — **Note:** The Devoid mechanic is a CDA that sets color to colorless regardless of mana cost. Tests for Devoid deferred to 604 (CDAs) session.

**105.2a** — BOUNDARY-DEF. Defines "monocolored." Predicate `is_monocolored()` derivable from color set.

**ATOM-105.2a-002**
- **Rule:** 105.2a — An object with exactly one color is monocolored
- **Mechanism:** `is_monocolored()` predicate on object characteristics
- **Minimal Board:** A card with mana cost {2}{R} (one color)
- **Action:** Query `is_monocolored()`
- **Expected Result:** True. A card with {R}{G} returns false (multicolored). A colorless card returns false.
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

**105.2b** — BOUNDARY-DEF. Defines "multicolored." Predicate `is_multicolored()` derivable from color set.

**ATOM-105.2b-002**
- **Rule:** 105.2b — An object with two or more colors is multicolored
- **Mechanism:** `is_multicolored()` predicate
- **Minimal Board:** A card with mana cost {R}{G}
- **Action:** Query `is_multicolored()`
- **Expected Result:** True. A monocolored card returns false.
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

**105.2c** — BOUNDARY-DEF. Defines "colorless." Predicate `is_colorless()` derivable from color set.

**ATOM-105.2c-002**
- **Rule:** 105.2c — An object with no colors is colorless
- **Mechanism:** `is_colorless()` predicate
- **Minimal Board:** A card with mana cost {4} and no color indicator
- **Action:** Query `is_colorless()`
- **Expected Result:** True. A card with any colored mana symbol returns false.
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

**105.3** — TESTABLE. Effects may change an object's color. New color replaces all previous colors (unless "in addition").

**ATOM-105.3-001**
- **Rule:** 105.3 — An effect that gives an object a new color replaces all previous colors
- **Mechanism:** Layer 5 color-setting continuous effect
- **Minimal Board:** A Red creature. A continuous effect says "Target creature becomes blue."
- **Action:** Apply Layer 5. Query color.
- **Expected Result:** Creature is Blue only (not Red and Blue)
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

**ATOM-105.3-002**
- **Rule:** 105.3 — An effect that gives a color "in addition" adds to existing colors
- **Mechanism:** Layer 5 color-adding continuous effect
- **Minimal Board:** A Red creature. A continuous effect says "Target creature becomes blue in addition to its other colors."
- **Action:** Apply Layer 5. Query color.
- **Expected Result:** Creature is Red and Blue
- **Phase:** Phase 5 Layers (L10)
- **Ticket:** L10

**105.4** — BOUNDARY-DEF. When asked to choose a color, must choose one of the five. "Multicolored" and "colorless" are not colors.

**ATOM-105.4-001**
- **Rule:** 105.4 — A player must choose one of the five colors when asked to choose a color
- **Mechanism:** Color choice validation in `DecisionProvider`
- **Minimal Board:** An effect asks a player to choose a color
- **Action:** Player attempts to choose "colorless"
- **Expected Result:** Choice is rejected. Only White/Blue/Black/Red/Green are valid.
- **Phase:** Phase 8 (when color-choice effects are implemented)
- **Ticket:** NEW — Color choice validation

**105.5** — PURE-DEF. Defines "color pair" — exactly two of the five colors. Enumeration of the 10 pairs. Naming.

---

### 106. Mana

**106.1** — PURE-DEF. Mana is the primary resource. Players spend it to pay costs.

**106.1a** — BOUNDARY-DEF. Five colors of mana: white, blue, black, red, green.

**ATOM-106.1a-001**
- **Rule:** 106.1a — There are five colors of mana
- **Mechanism:** `ManaSymbol::Colored(Color)` covers exactly 5 colors
- **Minimal Board:** N/A (type check)
- **Action:** Verify colored mana production and payment for each of W, U, B, R, G
- **Expected Result:** Each color of mana can be produced and used to pay costs of that color
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**106.1b** — BOUNDARY-DEF. Six types of mana: white, blue, black, red, green, and colorless.

**ATOM-106.1b-001**
- **Rule:** 106.1b — There are six types of mana (5 colors + colorless)
- **Mechanism:** Mana pool tracks all 6 types. Colorless is distinct from generic.
- **Minimal Board:** A permanent that produces {C} (colorless mana)
- **Action:** Tap for colorless mana. Attempt to pay a cost requiring {C}.
- **Expected Result:** Colorless mana pays {C} costs. Colored mana does NOT pay {C} costs.
- **Phase:** Already implemented in `types/mana.rs`
- **Ticket:** ALREADY-IMPLEMENTED (basic {C} payment works)

**106.2** — PURE-DEF. Mana symbols represent mana and mana costs. Reference to 107.4.

**106.3** — TESTABLE. Mana produced by mana abilities (rule 605). Source tracking — the engine must track which permanent produced which mana for snow mana, color-restricted mana (Cavern of Souls, Gwenna), and creature-only mana.

**ATOM-106.3-001**
- **Rule:** 106.3 — Mana retains its source identity in the pool
- **Mechanism:** Hybrid mana architecture: unrestricted mana uses the fast-path `HashMap<ManaColor, u32>` pool (no per-atom tracking needed). Restricted/tagged mana (snow-sourced, creature-only, persistent, etc.) is stored in a separate `Vec<ManaAtom>` with source metadata. The test verifies that snow-sourced mana is routed to the `ManaAtom` Vec, not the fast-path.
- **Minimal Board:** Snow-Covered Mountain tapped for {R}.
- **Action:** Query the mana in pool for its source
- **Expected Result:** The {R} is in the `ManaAtom` Vec (not the fast-path HashMap) because it carries a snow tag. It can pay {S} costs. Unrestricted {R} from a regular Mountain would be in the fast-path HashMap.
- **Phase:** Phase 8 (when snow/restricted mana sources exist)
- **Ticket:** NEW — Mana source tracking in pool (hybrid architecture)
- **CROSS-CUT:** Mana Abilities — full classification tests deferred to 605 session

**106.4** — TESTABLE. Mana goes into mana pool. Can be spent immediately or remain as unspent mana. Pool empties at end of each step and phase.

**ATOM-106.4-001**
- **Rule:** 106.4 — Mana pool empties at the end of each step and phase
- **Mechanism:** Mana pool clearing in `engine/turns.rs` at phase/step transitions
- **Minimal Board:** Player with {R}{R} in mana pool. Phase transition occurs.
- **Action:** Advance to next phase/step
- **Expected Result:** Player's mana pool is empty after the transition
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**106.4a** — PURE-DEF. Player announces remaining mana after spending. UI concern.

**106.4b** — PURE-DEF. Player announces mana pool on passing priority. UI concern.

**106.5** — TESTABLE. Mana of undefined type → produces no mana.

**ATOM-106.5-001**
- **Rule:** 106.5 — If an ability would produce mana of an undefined type, it produces no mana instead
- **Mechanism:** Mana production validation in `engine/mana.rs`
- **Minimal Board:** Meteor Crater ("Choose a color of a permanent you control. Add one mana of that color.") with no colored permanents.
- **Action:** Activate Meteor Crater's ability
- **Expected Result:** No mana is added to the pool (ability resolves but produces nothing)
- **Phase:** Phase 8 (when parameterized mana abilities exist)
- **Ticket:** NEW — Undefined mana type produces nothing
- **DP Note:** Engine should pre-compute valid choices. If set is empty, short-circuit to "produce nothing" without calling DecisionProvider. If exactly one choice, auto-select. Only call DP when there's a genuine decision.

**ATOM-106.5-002**
- **Rule:** 106.5 — Mana ability with no valid choices still activates but produces nothing
- **Mechanism:** Mana ability activation allowed even when output is empty
- **Minimal Board:** Meteor Crater with no colored permanents you control
- **Action:** Activate the ability
- **Expected Result:** Ability activates (mana abilities can always be activated). No mana produced. No DP call made (zero valid choices → short-circuit).
- **Phase:** Phase 8
- **Ticket:** NEW — Empty-choice mana ability short-circuit

**106.6** — TESTABLE. Mana with spending restrictions doesn't change the mana's type.

**ATOM-106.6-001**
- **Rule:** 106.6 — Mana spending restrictions don't affect the mana's type
- **Mechanism:** Restricted mana in mana pool retains its color/type
- **Minimal Board:** Player has {R} restricted to creature spells only
- **Action:** Query mana pool type
- **Expected Result:** The mana is still Red mana. It can pay {R} costs of creature spells.
- **Phase:** Phase 5-Pre (T12 design spike, T12b implementation)
- **Ticket:** T12

**Note:** Doubling Cube ("double each type of mana in your pool") is a clean integration test for 106.6 + mana doubling interaction. Not atomic — deferred to integration test suite.

**106.6a** — TESTABLE. Replacement effects that increase mana production apply restrictions/additional effects to all mana produced.

**ATOM-106.6a-001**
- **Rule:** 106.6a — If a replacement effect increases mana produced, restrictions apply to all mana
- **Mechanism:** Mana replacement effect propagation
- **Minimal Board:** A land that produces {G} restricted to creature spells. Mana Reflection doubles mana produced.
- **Action:** Tap the land. Replacement doubles: produces {G}{G}.
- **Expected Result:** Both {G} are restricted to creature spells
- **Phase:** Phase 6 (replacement effects)
- **Ticket:** NEW — Mana replacement restriction propagation

**106.7** — TESTABLE. "Could produce" mana determination.

**ATOM-106.7-001**
- **Rule:** 106.7 — Determining what type of mana a permanent "could produce" considers abilities as if they resolved now, ignoring costs
- **Mechanism:** Hypothetical mana production query
- **Minimal Board:** Exotic Orchard + opponent controls a Forest
- **Action:** Query what Exotic Orchard could produce
- **Expected Result:** It could produce {G} (because opponent's Forest could produce {G})
- **Phase:** Phase 8 (when "could produce" cards are implemented)
- **Ticket:** NEW — "Could produce" mana query

**106.8** — TESTABLE. Hybrid mana symbol added to pool → player chooses one half.

**ATOM-106.8-001**
- **Rule:** 106.8 — Adding mana represented by a hybrid symbol requires choosing one half
- **Mechanism:** Hybrid mana production resolution
- **Minimal Board:** An ability that would add {W/U} to a player's mana pool
- **Action:** Player chooses the white half
- **Expected Result:** One {W} is added to pool (not {W/U})
- **Phase:** Phase 8 (when hybrid mana production exists)
- **Ticket:** NEW — Hybrid mana production

**106.9** — TESTABLE. Phyrexian mana symbol added to pool → one mana of that color.

**ATOM-106.9-001**
- **Rule:** 106.9 — Adding mana represented by a Phyrexian symbol produces one mana of that color
- **Mechanism:** Phyrexian mana production
- **Minimal Board:** An ability that would add {R/P} to a player's mana pool
- **Action:** Resolve the ability
- **Expected Result:** One {R} is added to pool
- **Phase:** Phase 8
- **Ticket:** NEW — Phyrexian mana production

**106.10** — TESTABLE. Generic mana symbol added to pool → that much colorless mana.

**ATOM-106.10-001**
- **Rule:** 106.10 — Adding mana represented by a generic symbol produces that much colorless mana
- **Mechanism:** Generic-symbol mana production
- **Minimal Board:** An ability that would add {2} to a player's mana pool
- **Action:** Resolve the ability
- **Expected Result:** Two colorless mana are added to pool
- **Phase:** Phase 8 (currently mana production uses specific color outputs)
- **Ticket:** NEW — Generic symbol mana production

**106.11** — TESTABLE. Snow mana symbol added to pool → that much colorless mana.

**ATOM-106.11-001**
- **Rule:** 106.11 — Adding mana represented by snow mana symbols produces that much colorless mana
- **Mechanism:** Snow mana production
- **Minimal Board:** An ability that would add {S} to a player's mana pool
- **Action:** Resolve the ability
- **Expected Result:** One colorless mana is added to pool
- **Phase:** Phase 8
- **Ticket:** NEW — Snow mana production

**106.12** — PURE-DEF. "Tap for mana" = activate a mana ability with {T} in cost. Naming.

**106.12a** — TESTABLE (DEFERRED). Trigger definition for "tapped for mana." Engine needs a way to track "this permanent was tapped to produce mana this turn" for cards like Mana Web. Deferred to Phase 7/8.

**ATOM-106.12a-001**
- **Rule:** 106.12a — Engine tracks which permanents were tapped for mana
- **Mechanism:** `activate_mana_ability()` emits a `PermanentTappedForMana { permanent_id, mana_produced, controller }` event to the trigger queue. A flat boolean is insufficient — cards like Mana Web need to know *which* permanent was tapped and *what types* of mana it could produce. The oracle module's `available_mana_sources` already tracks per-permanent mana production capability (the query side); this adds the event emission side.
- **Minimal Board:** A Forest is tapped for {G}
- **Action:** Query whether the Forest was "tapped for mana" this turn
- **Expected Result:** True. An untapped Forest returns false. A tapped-by-effect (not for mana) Forest returns false.
- **Phase:** Phase 7/8 (when "tapped for mana" triggers/effects exist)
- **Ticket:** NEW — "Tapped for mana" event emission
- **Architectural note (Mana Web complexity):** The most complex "tapped for mana" card is Mana Web: "Whenever a land an opponent controls is tapped for mana, tap all lands that player controls that could produce any type of mana that land could produce." This requires: (1) the `PermanentTappedForMana` event to fire as a trigger, (2) the trigger handler to query `oracle::mana_helpers` for what types the tapped land *could* produce, (3) then query all other lands the controller controls for whether they *could produce any of those types*, (4) then tap all matches. Steps 2-3 are already partially supported by `available_mana_sources`. The primary use case for "tapped for mana" detection is land auras (e.g., Wild Growth, Fertile Ground) that detect when the enchanted permanent is tapped for mana and add additional mana — these are simpler (single-permanent trigger, fixed bonus) but use the same event infrastructure.

**106.12b** — TESTABLE (DEFERRED). Replacement effect definition for "tapped for mana." Phase 6 infrastructure. Same tracking requirement as 106.12a.

**106.13** — DEFERRED. Drain Power specific card rule. Niche.

---

### 107. Numbers and Symbols

**107.1** — PURE-DEF. Magic uses only integers.

**107.1a** — TESTABLE. Can't choose fractional numbers, deal fractional damage, etc.

**ATOM-107.1a-001**
- **Rule:** 107.1a — No fractional numbers. Damage, life, etc. are always integers.
- **Mechanism:** All game values use integer types (`i32`, `u32`, `i64`, `u64`)
- **Minimal Board:** N/A (type system guarantee)
- **Action:** Verify that damage, life, P/T are integer types
- **Expected Result:** All relevant types are integer. No floating-point game values.
- **Phase:** Already implemented (by type system)
- **Ticket:** ALREADY-IMPLEMENTED

**107.1b** — TESTABLE. Negative values are possible for game values (e.g., creature power) but effects that yield negative results use 0 instead (with exceptions for doubling/setting life/P/T).

**ATOM-107.1b-001**
- **Rule:** 107.1b — A creature's power can be negative (e.g., 3/4 creature gets -5/-0 → -2/4)
- **Mechanism:** P/T computation allows negative values
- **Minimal Board:** A 3/4 creature. A continuous effect gives it -5/-0.
- **Action:** Compute effective P/T
- **Expected Result:** Power is -2, toughness is 4. The creature does not assign combat damage (negative power → 0 damage in combat).
- **Phase:** Phase 5 Layers (L04/L08 — P/T sublayers)
- **Ticket:** L04 / L08

**ATOM-107.1b-002**
- **Rule:** 107.1b — An effect that would produce a negative amount uses 0 instead (e.g., mana production from negative power)
- **Mechanism:** Effect result clamping
- **Minimal Board:** Viridian Joiner (1/2, "{T}: Add {G} equal to this creature's power"). Effect gives it -2/-0.
- **Action:** Activate Viridian Joiner's ability
- **Expected Result:** No mana is added (power is -1, clamped to 0 for mana production)
- **Phase:** Phase 8 (when power-referencing mana abilities exist)
- **Ticket:** NEW — Negative-to-zero clamping for effect results

**ATOM-107.1b-003**
- **Rule:** 107.1b — Negative calculation result clamped to 0 for modification effects (not set/double)
- **Mechanism:** Effect result clamping for +X/+X where X is a calculated negative value
- **Minimal Board:** Chameleon Colossus (4/4, "{2}{G}{G}: gets +X/+X where X is its power"). Effect gives it -6/-0 (now -2/4). Activate ability.
- **Action:** Resolve the ability. X = power = -2.
- **Expected Result:** Creature remains -2/4. X is clamped to 0 because +X/+X is a modification (not a set/double/triple). Per 107.1b: "zero is used instead, unless that effect doubles, triples, or sets to a specific value a player's life total or the power and/or toughness." The +X/+X ability is neither doubling nor setting; it's modifying. So X = max(0, -2) = 0.
- **Phase:** Phase 5 Layers + Phase 8
- **Ticket:** L08 + NEW — Negative-to-zero clamping for modification effects

**ATOM-107.1b-004**
- **Rule:** 107.1b — Exception: doubling P/T does NOT clamp negative values to 0
- **Mechanism:** P/T doubling bypasses negative-to-zero clamping
- **Minimal Board:** A creature with -2 power (from other effects, e.g., a 2/4 creature with a -4/-0 modifier). An ability on the stack resolves with effect "double target creature's power and toughness until end of turn."
- **Action:** Apply the doubling effect to the creature with -2 effective power
- **Expected Result:** Power becomes -4 (not 0). Per 107.1b: the "unless" clause exempts effects that "double... the power and/or toughness of a creature." Doubling -2 yields -4, and the negative-to-zero clamp does NOT apply.
- **Phase:** Phase 5 Layers (L08)
- **Ticket:** L08

**107.1c** — PURE-DEF. "Choose any number" means any positive number or zero.

**107.2** — META/PURE-DEF. Meta-mathematical constraint: "you can't define numbers via non-standard notation to get around integer limits." No card or game state can trigger this. CR lawyer-proofing rule. No atom needed.

**107.3** — PURE-DEF. X is a placeholder for a number. Framework.

**107.3a** — TESTABLE. X in mana/activation cost is chosen and announced during casting/activation.

**ATOM-107.3a-001**
- **Rule:** 107.3a — Controller chooses X when casting a spell with {X} in its mana cost
- **Mechanism:** `choose_x_value` DP method, stored in `StackEntry.x_value`
- **Minimal Board:** A spell with mana cost {X}{R}. Player has 5 mana available.
- **Action:** Cast the spell. DP chooses X=3.
- **Expected Result:** `StackEntry.x_value == Some(3)`. Total mana cost is {3}{R} = 4 mana.
- **Phase:** Phase 5-Pre (T18 — 601.2 casting pipeline)
- **Ticket:** T18

**107.3b** — TESTABLE (DEFERRED). Casting for free with no defined X → X must be 0. Requires alternative costs (T18/Phase 5-Pre) before it can be tested.

**ATOM-107.3b-001**
- **Rule:** 107.3b — If casting without paying mana cost and X isn't defined, X = 0
- **Mechanism:** X value forced to 0 when alt cost bypasses mana cost
- **Minimal Board:** A spell with {X}{U}{U} cast via "without paying its mana cost"
- **Action:** Cast the spell via the free-cast effect
- **Expected Result:** X is forced to 0. The spell is cast for {0}{U}{U} → just free.
- **Phase:** Phase 5-Pre (T18) + Phase 8 (free-cast effects)
- **Ticket:** T18
- **Dependency:** Requires T18 (alternative costs) to be implemented first

**107.3c** — PURE-DEF. X defined by text of the spell/ability. No player choice.

**107.3d** — DEFERRED. X in special action costs (suspend, morph). Phase 9.

**107.3e** — TESTABLE (DEFERRED to Phase 7). X referencing the spell's X value in a triggered ability retains its value.

**ATOM-107.3e-001**
- **Rule:** 107.3e — X in a triggered ability retains the value from the triggering spell's cost
- **Mechanism:** Triggered ability's X reads from the triggering spell's `StackEntry.x_value`
- **Minimal Board:** Zaxara, the Exemplary on battlefield ("Whenever you cast a spell with {X} in its mana cost, create a 0/0 green Hydra creature token, then put X +1/+1 counters on it.") Cast a spell with X=4.
- **Action:** Zaxara's trigger resolves
- **Expected Result:** A 0/0 Hydra token is created with 4 +1/+1 counters (making it 4/4). X retains its value from the cast spell.
- **Phase:** Phase 7 (triggered abilities) + Phase 8 (token creation)
- **Ticket:** Phase 7 + Phase 8
- **Note:** Arguably an integration test (casting + trigger + token + counters), but the rule citation is atomic. 

**107.3f** — PURE-DEF. X in text but not in a cost — chosen at appropriate time.

**107.3g** — TESTABLE. X in mana cost of a card not on the stack → treated as 0.

**ATOM-107.3g-001**
- **Rule:** 107.3g — {X} in mana cost of a card not on the stack is treated as 0
- **Mechanism:** CMC/mana value calculation for cards in zones other than stack
- **Minimal Board:** A card with mana cost {X}{R}{R} in a player's graveyard
- **Action:** Query the card's mana value (converted mana cost)
- **Expected Result:** Mana value is 2 (X=0, so {0}{R}{R} = 2)
- **Phase:** Phase 5 Layers (mana value calculation)
- **Ticket:** NEW — Mana value calculation with X=0 in non-stack zones

**107.3h** — TESTABLE. Paying an object's mana cost that includes {X}: X=0 unless it's a spell on the stack.

**ATOM-107.3h-001**
- **Rule:** 107.3h — If an effect instructs paying a cost with {X}, X=0 unless the object is a spell on the stack
- **Mechanism:** X value resolution when paying object's mana cost from a non-stack zone
- **Minimal Board:** Back from the Brink (enchantment: "Exile a creature card from your graveyard and pay its mana cost: Create a token that's a copy of that card."). Exile a creature with mana cost {X}{G}{G} from graveyard.
- **Action:** Activate Back from the Brink. Determine cost to pay for the exiled creature.
- **Expected Result:** Cost is {G}{G} (X=0 because the card is not on the stack). The token copy has mana value reflecting X=0.
- **Phase:** Phase 8
- **Ticket:** NEW — X=0 for non-stack mana cost payments

**107.3i** — PURE-DEF. All instances of X on an object have the same value.

**107.3j** — TESTABLE. X in a gained ability uses the gaining ability's definition, or 0 if undefined.

**ATOM-107.3j-001**
- **Rule:** 107.3j — X in a gained ability uses the gaining ability's definition, or 0 if undefined
- **Mechanism:** X value scoping for granted abilities
- **Minimal Board:** Jodah, the Unifier ("Creatures you control get +X/+X, where X is the number of legendary creatures you control.") controls 3 legendary creatures including himself. A vanilla 1/1 token is on the battlefield.
- **Action:** Compute the token's effective P/T
- **Expected Result:** Token is 4/4 (1/1 + 3/3 from Jodah's granted ability, X=3 legendary creatures)
- **Phase:** Phase 5 Layers (L06 ability granting + L08 P/T)
- **Ticket:** L06 + L08
- **Note:** Integration test case: a creature with its own X ability (e.g., from its mana cost) AND a Jodah-granted X ability should have two independent X values.

**107.3k** — PURE-DEF. X in an activated ability's activation cost is independent of other X values on the object.

**107.3m** — TESTABLE. ETB triggered ability/replacement effect references X from the spell's cost.

**ATOM-107.3m-001**
- **Rule:** 107.3m — An ETB trigger/replacement referring to X uses the value of X from the spell that became that permanent
- **Mechanism:** `CastInfo.x_value` on `BattlefieldEntity` read by ETB effects
- **Minimal Board:** A creature with "enters with X +1/+1 counters" cast with X=3
- **Action:** Creature resolves and enters the battlefield
- **Expected Result:** Creature enters with 3 +1/+1 counters (X from CastInfo). The permanent's own X (in mana cost) is 0 per 107.3g, but the ETB effect uses the spell's X.
- **Phase:** Phase 5-Pre (T06 carries x_value, T21a carries CastInfo) + Phase 6 (ETB replacement)
- **Ticket:** T06 + T21a + Phase 6

**107.3n** — TESTABLE (DEFERRED to Phase 7). Delayed trigger's X uses the creating spell/ability's X. Tests X-value persistence across delayed triggers.

**ATOM-107.3n-001**
- **Rule:** 107.3n — Delayed trigger uses X from the spell that created it
- **Mechanism:** Delayed trigger stores X value from creating spell
- **Minimal Board:** Disorder in the Court resolves with X=3 (instant {X}{W}{U}: "Exile X target creatures, then investigate X times. Return the exiled cards to the battlefield tapped under their owners' control at the beginning of the next end step."). 3 creatures exiled.
- **Action:** At beginning of next end step, delayed trigger fires
- **Expected Result:** All 3 exiled creatures return to the battlefield tapped. The delayed trigger remembers X=3 from the original spell.
- **Phase:** Phase 7 (delayed triggers)
- **Ticket:** Phase 7 — Delayed trigger X-value persistence

**107.3p** — PURE-DEF. Y follows same rules as X. Naming.

**107.4** — BOUNDARY-DEF. Comprehensive list of mana symbols.

**ATOM-107.4-001**
- **Rule:** 107.4 — All mana symbol types exist in the type system
- **Mechanism:** `ManaSymbol` enum completeness
- **Minimal Board:** N/A (type check)
- **Action:** Verify `ManaSymbol` enum covers: Colored(5), Generic(N), Colorless, X, Hybrid(10), MonoHybrid(10), Phyrexian(5), HybridPhyrexian(10), Snow
- **Expected Result:** All symbol types are representable
- **Phase:** Already implemented in `types/mana.rs`
- **Ticket:** ALREADY-IMPLEMENTED

**107.4a** — PURE-DEF. Colored mana symbols. Naming + payment rule (colored mana pays colored costs).

**107.4b** — PURE-DEF. Numerical/variable symbols represent generic mana. Generic mana payable with any type.

**107.4c** — TESTABLE. {C} represents colorless mana and a cost payable only with colorless mana.

**ATOM-107.4c-001**
- **Rule:** 107.4c — {C} cost can only be paid with colorless mana
- **Mechanism:** `ManaPool::pay` rejects colored mana for {C} costs
- **Minimal Board:** Player has {R}{R} in pool. Cost is {C}.
- **Action:** Attempt to pay {C} with red mana
- **Expected Result:** Payment fails. Red mana cannot pay {C}.
- **Phase:** Already implemented in `types/mana.rs`
- **Ticket:** ALREADY-IMPLEMENTED

**107.4d** — PURE-DEF. {0} represents zero mana. Placeholder for no-cost actions.

**107.4e** — TESTABLE. Hybrid mana symbols can be paid in one of two ways. Hybrid symbols are all of their component colors.

**ATOM-107.4e-001**
- **Rule:** 107.4e — {W/U} can be paid with either {W} or {U}
- **Mechanism:** Hybrid mana payment in `ManaPool::pay`
- **Minimal Board:** Player has {W} in pool. Cost includes {W/U}.
- **Action:** Pay {W/U} with {W}
- **Expected Result:** Payment succeeds
- **Phase:** Phase 5-Pre (known gap — hybrid payment not yet implemented)
- **Ticket:** NEW — Hybrid mana payment implementation

**ATOM-107.4e-002**
- **Rule:** 107.4e — {2/B} can be paid with {B} or two mana of any type
- **Mechanism:** Monocolored hybrid mana payment
- **Minimal Board:** Player has {R}{R} in pool. Cost includes {2/B}.
- **Action:** Pay {2/B} with {R}{R} (generic half)
- **Expected Result:** Payment succeeds (two generic mana pays the generic half)
- **Phase:** Phase 5-Pre
- **Ticket:** NEW — Monocolored hybrid mana payment

**ATOM-107.4e-003**
- **Rule:** 107.4e — {W/U} paid with the other half (choosing {U})
- **Mechanism:** Hybrid mana payment choosing the second color
- **Minimal Board:** Player has {U} in pool. Cost includes {W/U}.
- **Action:** Pay {W/U} with {U}
- **Expected Result:** Payment succeeds
- **Phase:** Phase 5-Pre
- **Ticket:** NEW — Hybrid mana payment implementation

**107.4f** — TESTABLE. Phyrexian mana can be paid with color or 2 life. Hybrid Phyrexian can be paid with either component color or 2 life.

**ATOM-107.4f-001**
- **Rule:** 107.4f — {R/P} can be paid with {R} or by paying 2 life
- **Mechanism:** Phyrexian mana payment in `ManaPool::pay` + life payment
- **Minimal Board:** Player at 20 life with empty mana pool. Cost is {R/P}.
- **Action:** Player chooses to pay 2 life
- **Expected Result:** Player's life total becomes 18. Cost is paid.
- **Phase:** Phase 5-Pre (known gap — Phyrexian payment not implemented)
- **Ticket:** NEW — Phyrexian mana payment implementation

**ATOM-107.4f-002**
- **Rule:** 107.4f — {W/U/P} can be paid with {W}, {U}, or 2 life
- **Mechanism:** Hybrid Phyrexian mana payment
- **Minimal Board:** Player has {U} in pool. Cost is {W/U/P}.
- **Action:** Pay with {U}
- **Expected Result:** Payment succeeds
- **Phase:** Phase 5-Pre
- **Ticket:** NEW — Hybrid Phyrexian mana payment

**ATOM-107.4f-003**
- **Rule:** 107.4f — {R/P} paid with mana instead of life
- **Mechanism:** Phyrexian mana payment choosing the mana option
- **Minimal Board:** Player has {R} in pool. Cost is {R/P}.
- **Action:** Player chooses to pay with {R}
- **Expected Result:** Payment succeeds. No life lost.
- **Phase:** Phase 5-Pre
- **Ticket:** NEW — Phyrexian mana payment implementation

**107.4g** — PURE-DEF. {H} in rules text means any Phyrexian symbol. Naming.

**107.4h** — TESTABLE. {S} cost payable with any mana from a snow source. Generic cost reductions don't affect {S}.

**ATOM-107.4h-001**
- **Rule:** 107.4h — {S} can be paid with one mana of any type produced by a snow source
- **Mechanism:** Snow mana payment tracking (mana must be tagged as snow-sourced)
- **Minimal Board:** Player taps Snow-Covered Mountain for {R} (snow source). Cost is {S}.
- **Action:** Pay {S} with the snow-sourced {R}
- **Expected Result:** Payment succeeds
- **Phase:** Phase 8 (when snow sources exist)
- **Ticket:** NEW — Snow mana source tracking + payment

**ATOM-107.4h-002**
- **Rule:** 107.4h — Generic cost reductions don't reduce {S} costs
- **Mechanism:** Cost reduction exemption for snow mana
- **Minimal Board:** A spell costs {1}{S}. An effect reduces generic costs by {1}.
- **Action:** Calculate total cost
- **Expected Result:** Cost is {0}{S} (the {1} generic was reduced, but {S} is not affected by generic reductions)
- **Phase:** Phase 5 Layers (cost modification pipeline) + Phase 8
- **Ticket:** NEW — Snow mana cost reduction exemption

**ATOM-107.4h-003**
- **Rule:** 107.4h — Generic cost reduction does NOT reduce a pure {S} cost
- **Mechanism:** Snow mana cost is not generic; generic reductions don't apply
- **Minimal Board:** A spell costs just {S}. An effect reduces generic costs by {1}.
- **Action:** Calculate total cost
- **Expected Result:** Cost is still {S} (snow mana is not generic; the reduction has nothing to reduce)
- **Phase:** Phase 8
- **Ticket:** NEW — Snow mana cost reduction exemption

**107.5** — TESTABLE. {T} in activation cost means "Tap this permanent." Already tapped can't pay. Summoning-sick creatures can't activate {T} abilities.

**ATOM-107.5-001**
- **Rule:** 107.5 — Already tapped permanent can't be tapped to pay {T} cost
- **Mechanism:** `check_cost_resource(Cost::Tap)` checks tapped state
- **Minimal Board:** A tapped permanent with a {T} activated ability
- **Action:** Attempt to activate the ability
- **Expected Result:** Activation fails — permanent is already tapped
- **Phase:** Already implemented in `engine/costs.rs`
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-107.5-002**
- **Rule:** 107.5 — Summoning-sick creature can't activate {T} ability
- **Mechanism:** Summoning sickness check in `check_cost_resource(Cost::Tap)`
- **Minimal Board:** A creature that entered the battlefield this turn (no haste)
- **Action:** Attempt to activate its {T} mana ability
- **Expected Result:** Activation fails — summoning sickness
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**107.6** — TESTABLE. {Q} in activation cost means "Untap this permanent." Already untapped can't pay. Same summoning sickness rule as {T}.

**ATOM-107.6-001**
- **Rule:** 107.6 — Already untapped permanent can't be untapped to pay {Q} cost
- **Mechanism:** `check_cost_resource(Cost::Untap)` checks untapped state
- **Minimal Board:** An untapped permanent with a {Q} activated ability
- **Action:** Attempt to activate the ability
- **Expected Result:** Activation fails — permanent is already untapped
- **Phase:** Already implemented in `engine/costs.rs`
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-107.6-002**
- **Rule:** 107.6 — Summoning-sick creature can't activate {Q} ability
- **Mechanism:** Summoning sickness check for {Q} cost
- **Minimal Board:** A creature that entered this turn with a {Q} activated ability
- **Action:** Attempt to activate
- **Expected Result:** Activation fails — summoning sickness
- **Phase:** Phase 5-Pre (T10)
- **Ticket:** T10

**107.7** — TESTABLE. Planeswalker loyalty symbols: [+N] puts N loyalty counters, [-N] removes N.

**ATOM-107.7-001**
- **Rule:** 107.7 — [+N] adds N loyalty counters; [-N] removes N loyalty counters
- **Mechanism:** Loyalty ability cost payment
- **Minimal Board:** A planeswalker with 4 loyalty counters and abilities [+1] and [-3]
- **Action:** Activate the [+1] ability
- **Expected Result:** Planeswalker now has 5 loyalty counters
- **Phase:** Phase 5-Pre (T14 sets up loyalty counters) + Phase 8 (loyalty abilities)
- **Ticket:** T14 + Phase 8

**ATOM-107.7-002**
- **Rule:** 107.7 — [-N] removes N loyalty counters
- **Mechanism:** Loyalty ability cost payment
- **Minimal Board:** Planeswalker with 4 loyalty. Activate [-3].
- **Action:** Pay the loyalty cost
- **Expected Result:** Planeswalker now has 1 loyalty counter
- **Phase:** Phase 5-Pre (T14) + Phase 8
- **Ticket:** T14 + Phase 8

**ATOM-107.7-003**
- **Rule:** 107.7 — [0] loyalty ability neither adds nor removes loyalty counters
- **Mechanism:** Loyalty ability cost payment with zero cost
- **Minimal Board:** Planeswalker with 4 loyalty counters and a [0] ability
- **Action:** Activate the [0] ability
- **Expected Result:** Planeswalker still has 4 loyalty counters. The ability goes on the stack normally.
- **Phase:** Phase 5-Pre (T14) + Phase 8
- **Ticket:** T14 + Phase 8

**107.8–107.8b** — DEFERRED. Level Up cards. Modern-legal. Rules 107.8 describe the visual template; mechanical behavior lives with keyword 702.87 (Level Up). Implementation deferred to 702.87 session.

**107.9** — PURE-DEF. Tombstone icon. No game effect.

**107.10** — PURE-DEF. Type icon on Future Sight cards. No game effect.

**107.11–107.12** — OUT-OF-SCOPE. Planechase symbols.

**107.13** — PURE-DEF. Color indicator description. Covered by 105.2 tests.

**107.14** — TESTABLE. Energy symbol {E} represents one energy counter. Paying {E} removes one energy counter.

**ATOM-107.14-001**
- **Rule:** 107.14 — {E} represents one energy counter. Paying {E} removes one from the player.
- **Mechanism:** Energy counter tracking on player + cost payment
- **Minimal Board:** Player with 3 energy counters. Ability costs {E}{E}.
- **Action:** Pay the {E}{E} cost
- **Expected Result:** Player now has 1 energy counter
- **Phase:** Phase 8 (energy counter system)
- **Ticket:** NEW — Energy counter system

**107.15–107.15b** — PURE-DEF. Saga chapter symbol location on physical cards.

**107.15a-b** — DEFERRED. Saga rules, Phase 7 (triggered abilities) + Phase 8.

**107.16–107.16a** — DEFERRED. Class cards. Phase 8+.

**107.17–107.17a** — OUT-OF-SCOPE. Ticket counters. Sticker/Un-set mechanic.

**107.18** — DEFERRED. Pawprint symbol. Modal indicator only.

---

## Rules 108–110: Cards, Objects, Permanents

### 108. Cards

**108.1** — PURE-DEF. Use Oracle text for card wording. Engine uses `CardData` from `CardRegistry`.

**108.2** — PURE-DEF. "Card" means only a Magic card or object represented by one.

**108.2a** — PURE-DEF. Traditional vs. nontraditional cards. Physical description.

**108.2b** — BOUNDARY-DEF. Tokens aren't considered cards.

**ATOM-108.2b-001**
- **Rule:** 108.2b — Tokens aren't cards, even if represented by a physical card
- **Mechanism:** `is_token` flag on `GameObject` distinguishes tokens from cards
- **Minimal Board:** A token on the battlefield
- **Action:** Query whether the token is a "card"
- **Expected Result:** `is_token == true`. Any effect that says "target card" or "card in graveyard" does not match tokens. Effects that reference "permanent" do match tokens on the battlefield.
- **Phase:** Phase 5-Pre (T03 adds `is_token`)
- **Ticket:** T03

**108.3** — TESTABLE. Owner of a card = player who started the game with it in their deck.

**ATOM-108.3-001**
- **Rule:** 108.3 — A card's owner is the player who started the game with it in their deck
- **Mechanism:** `GameObject.owner` field set during game setup
- **Minimal Board:** A card owned by P0 that has been moved to P1's control (e.g., via control change)
- **Action:** Query `obj.owner`
- **Expected Result:** Owner is still P0, regardless of current controller
- **Phase:** Already implemented (`GameObject.owner` set at creation)
- **Ticket:** ALREADY-IMPLEMENTED

**108.3a** — OUT-OF-SCOPE. Planechase planar deck owner.

**108.3b** — DEFERRED. Cards from outside the game. Sideboard interaction.

**Integration test note:** Wish effects (Burning Wish, etc.) pulling from sideboard need correct owner tracking, sideboard-to-game zone transfer, and return-to-sideboard on game end/player loss. Deferred integration test for Phase 8.

**108.4** — TESTABLE. Only objects on the stack or battlefield have a controller.

**ATOM-108.4-001**
- **Rule:** 108.4 — A card in a zone other than stack/battlefield has no controller
- **Mechanism:** Controller query returns `None` for cards in hand/library/graveyard/exile
- **Minimal Board:** A card in a player's graveyard
- **Action:** Query the card's controller
- **Expected Result:** No controller (use owner instead per 108.4a)
- **Phase:** Already implemented (controller only tracked in `BattlefieldEntity` and `StackEntry`)
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-108.4-002**
- **Rule:** 108.4 — A card in exile has no controller
- **Mechanism:** Controller query returns `None` for exiled cards
- **Minimal Board:** A card in exile (e.g., exiled by Path to Exile)
- **Action:** Query the card's controller
- **Expected Result:** No controller. Owner is still the original owner.
- **Phase:** Already implemented (exile zone has no controller tracking)
- **Ticket:** ALREADY-IMPLEMENTED

**108.4a** — TESTABLE. If anything asks for controller of a card without one, use owner.

**ATOM-108.4a-001**
- **Rule:** 108.4a — Use owner when controller is needed for a non-permanent/non-spell
- **Mechanism:** Controller fallback to owner
- **Minimal Board:** A card in graveyard owned by P0. An effect references "this card's controller."
- **Action:** Resolve the reference
- **Expected Result:** Returns P0 (the owner)
- **Phase:** Phase 8 (when effects reference controller of non-stack/non-battlefield objects)
- **Ticket:** NEW — Controller-to-owner fallback for non-controlled objects
- **Architectural note:** While specific card examples invoking this fallback are rare, the mechanism is architecturally important — any `get_controller()` helper must implement this fallback to avoid panics when resolving effects that reference controller of graveyard/exile/hand cards.
- **Judge forum findings (multiplayer controller chain, 2026-04-02):** In multiplayer, when a player leaves the game, 800.4a applies in strict order: (1) all objects *owned* by that player leave the game — regardless of zone or controller, (2) all control-change effects giving that player control of objects end, (3) objects on the stack not represented by cards and still controlled by that player cease to exist, (4) remaining objects still controlled by that player are exiled. **Step 1 is absolute and fires first.** If the *owner* leaves, their card leaves the game — it doesn't matter who controls it or what control-change effects exist.
  - **Gonti scenario:** Player A exiles Player B's Grizzly Bears face-down with Gonti, then casts the Bears (A is default controller per 110.2). Player C steals the Bears with Act of Treason (duration-based control-change). If Player A (caster/default controller) leaves: A's control-change effects end (none here — A had default control, not a control-change effect), and the Bears are still owned by B and controlled by C via Act of Treason. The Bears persist. When Act of Treason ends in cleanup, control returns to the default controller — but A is gone, so the Bears have no valid default controller and are exiled per 800.4c.
  - **If Player B (owner) leaves:** 800.4a step 1 — the Bears leave the game immediately, regardless of who controls them. Control-change effects are irrelevant.
  - **Aethersnatch variant:** If Player C uses Aethersnatch on the Bears spell before it resolves, C gains control via a duration-less control-change effect. C is now controller; A is still default controller (caster). If A leaves: A's control-change effects end (none), Bears persist under C. If *C* leaves: the Aethersnatch effect ends (step 2), control returns to default controller A. Key insight: Aethersnatch's control-change has no duration, so it doesn't end in cleanup — it only ends when the player who gained control leaves (step 2 of 800.4a).
  - **General rule:** Owner leaving → card leaves (always, step 1). Controller leaving → control-change effects end, control falls back to default controller (step 2). Default controller leaving → object eventually exiled if no valid controller remains (step 4).
  - The full cleanup chain is complex and belongs with Chapter 8 (rules 800.x). Confidently deferred.

**108.5** — PARTIALLY DEFERRED. Mentions dungeon cards (which are in scope), rest out of scope.

**108.6** — PURE-DEF. Reference to section 2.

---

### 109. Objects

**109.1** — BOUNDARY-DEF. An object is: ability on stack, card, copy of card, token, spell, permanent, or emblem.

**ATOM-109.1-001**
- **Rule:** 109.1 — Exhaustive list of object types
- **Mechanism:** Engine's object model covers all object types
- **Minimal Board:** N/A (architecture check)
- **Action:** Verify that `GameObject` can represent cards, tokens, copies. `StackEntry` represents spells/abilities on stack. `BattlefieldEntity` represents permanents. Emblem type exists or is planned.
- **Expected Result:** All 7 categories are representable in the data model
- **Phase:** Mostly implemented. Emblems are Phase 8+.
- **Ticket:** NEW — Emblem object type (Phase 8)

**109.2** — TESTABLE. A description with card type/subtype but no zone/card/spell/source word means a permanent on the battlefield.

**ATOM-109.2-001**
- **Rule:** 109.2 — "Creature" with no qualifier means creature permanent on battlefield
- **Mechanism:** Target/effect resolution defaults to battlefield permanents
- **Minimal Board:** Creature on battlefield + creature card in graveyard. Effect says "destroy target creature."
- **Action:** Choose targets
- **Expected Result:** Only the battlefield creature is a legal target. The graveyard creature card is not.
- **Phase:** Already implemented (targeting defaults to battlefield)
- **Ticket:** ALREADY-IMPLEMENTED
- **Cross-ref:** The "can only target battlefield creatures" aspect is also covered by 115.x (targeting rules). This atom tests the 109.2 semantic constraint; 115.x tests the targeting validation mechanism.

**109.2a** — PURE-DEF. "Card" + zone name = card in that zone.

**109.2b** — PURE-DEF. "Spell" = spell on the stack.

**109.2c** — PURE-DEF. "Source" = source in any zone.

**Note (109.2 vs. 109.2a-c classification):** 109.2 is TESTABLE because it defines a *behavioral constraint*: the word "creature" on a card can ONLY refer to a creature permanent on the battlefield. The engine must enforce this (targeting validation, effect resolution). Rules 109.2a-c are PURE-DEF because they define vocabulary ("creature card," "creature spell," etc.) that the engine already handles via its type system — no new enforcement obligation.

**109.2d** — OUT-OF-SCOPE. Scheme cards.

**109.3** — BOUNDARY-DEF. Object characteristics: name, mana cost, color, color indicator, card type, subtype, supertype, rules text, abilities, power, toughness, loyalty, defense, hand modifier, life modifier.

**ATOM-109.3-001**
- **Rule:** 109.3 — Object characteristics are a defined set; other info (tapped, target, owner, controller) is NOT a characteristic
- **Mechanism:** `EffectiveCharacteristics` struct includes exactly the listed fields
- **Minimal Board:** A permanent on the battlefield
- **Action:** Query characteristics vs. non-characteristics
- **Expected Result:** `EffectiveCharacteristics` contains name, mana_cost, colors, color_indicator, types, subtypes, supertypes, abilities, power, toughness, loyalty, defense. It does NOT contain tapped state, controller, owner (those are separate fields on `BattlefieldEntity`/`GameObject`).
- **Phase:** Phase 5 Layers (L01 defines `EffectiveCharacteristics`)
- **Ticket:** L01

**109.4** — TESTABLE. Only objects on stack or battlefield have a controller. (Same as 108.4 — cross-reference.)

**109.4a** — PURE-DEF. Mana ability controller. Rule 605 reference.

**109.4b** — PURE-DEF. Triggered ability controller before stack placement. Phase 7.

**109.4c** — DEFERRED (Phase 8). Emblems are created by planeswalkers and some other cards (The Ring Tempts You, etc.). Many Standard-legal cards create emblems. Controller tracking for emblems is in-scope.

**ATOM-109.4c-001**
- **Rule:** 109.4c — An emblem's controller is the player who was told to get it
- **Mechanism:** Emblem controller tracking in command zone
- **Minimal Board:** P0 activates a planeswalker ultimate that says "You get an emblem with [ability]"
- **Action:** Create the emblem. Query its controller.
- **Expected Result:** Emblem's controller is P0. The ability on the emblem applies to P0's objects.
- **Phase:** Phase 8 (emblem creation)
- **Ticket:** Phase 8 — Emblem controller tracking

**109.4d–g** — OUT-OF-SCOPE. Planechase, Vanguard, Archenemy, Conspiracy controllers.
NOTE: 109.4e is IN stretch scope, as Vanguard is a stretch goal to support.

**109.5** — PURE-DEF. "You"/"your" refers to controller/would-be controller/owner. Interpretation rule for card text.

---

### 110. Permanents

**110.1** — PURE-DEF. A permanent is a card or token on the battlefield. Becomes permanent on enter, stops on leave. Naming.

**110.2** — TESTABLE. Permanent's owner = card's owner. Controller = player under whose control it entered the battlefield.

**ATOM-110.2-001**
- **Rule:** 110.2 — Permanent's controller is by default the player under whose control it entered
- **Mechanism:** `BattlefieldEntity.controller` set by `init_zone_state_with_controller`
- **Minimal Board:** P0 casts a creature spell. It resolves.
- **Action:** Check the permanent's controller
- **Expected Result:** Controller is P0 (the player who cast it)
- **Phase:** Already implemented (Phase 3 fix — `stack.rs` uses `entry.controller`)
- **Ticket:** ALREADY-IMPLEMENTED

**110.2a** — TESTABLE. If an effect puts an object onto the battlefield, it enters under that player's control (unless effect says otherwise).

**ATOM-110.2a-001**
- **Rule:** 110.2a — Effect putting object onto battlefield → enters under that player's control
- **Mechanism:** Controller assignment in `init_zone_state_with_controller`
- **Minimal Board:** An effect says "put target creature card from your graveyard onto the battlefield under your control." P0 owns the card, P1 controls the effect.
- **Action:** Resolve the effect. The card enters under P1's control (the effect's controller).
- **Expected Result:** Permanent's controller is P1. Owner remains P0.
- **Phase:** Phase 8 (when return-to-battlefield effects exist)
- **Ticket:** NEW — ETB controller from effect controller

**ATOM-110.2a-002**
- **Rule:** 110.2a — A permanent can enter the battlefield under an opponent's control
- **Mechanism:** ETB controller assignment uses the effect's specified controller, not the spell's caster
- **Minimal Board:** Xantcha, Sleeper Agent ("enters the battlefield under the control of an opponent of your choice"). P0 casts Xantcha, choosing P1.
- **Action:** Xantcha resolves and enters the battlefield
- **Expected Result:** Xantcha's controller is P1 (the chosen opponent). Owner is P0 (the caster). P1 controls the permanent despite P0 casting it.
- **Phase:** Phase 8 (ETB controller override effects)
- **Ticket:** NEW — ETB opponent-control override

**110.2b** — TESTABLE. Gaining control of a permanent spell → controller of the permanent is the new controller, but "default" controller is original caster.

**ATOM-110.2b-001**
- **Rule:** 110.2b — Gaining control of a permanent spell → the gaining player controls the resulting permanent
- **Mechanism:** `StackEntry.controller` modification + permanent resolution
- **Minimal Board:** P0 casts a creature. P1 gains control of the spell on the stack.
- **Action:** The spell resolves.
- **Expected Result:** The permanent enters under P1's control. (Multiplayer note: P0 is the "default" controller for 800.4c purposes.)
- **Phase:** Phase 5 Layers (L11 — Layer 2 control) + stack controller change
- **Ticket:** L11

**Note (110.2b multiplayer):** If the player who "gained control" of another player's permanent leaves the game, rule 800.4a governs what happens. Flagged for multiplayer investigation — no changes now.

**110.3** — PURE-DEF. Nontoken permanent's characteristics = printed + continuous effects. Reference to 613.

**110.4** — BOUNDARY-DEF. Six permanent types: artifact, battle, creature, enchantment, land, planeswalker. Instant/sorcery can't enter battlefield.

**ATOM-110.4-001**
- **Rule:** 110.4 — Instant and sorcery cards can't enter the battlefield
- **Mechanism:** Battlefield entry guard in `engine/zones.rs`
- **Minimal Board:** An instant card in graveyard. An effect tries to put it onto the battlefield.
- **Action:** Attempt to move the instant to the battlefield
- **Expected Result:** The move is prevented. The card stays in its current zone.
- **Phase:** Phase 5-Pre (T21a — instant/sorcery battlefield guard)
- **Ticket:** T21a

**110.4a** — BOUNDARY-DEF. "Permanent card" = artifact, battle, creature, enchantment, land, or planeswalker card.

**ATOM-110.4a-001**
- **Rule:** 110.4a — "Permanent card" filter includes exactly the 6 permanent types
- **Mechanism:** `CardType::is_permanent()` predicate
- **Minimal Board:** Cards of each type
- **Action:** Check `is_permanent()` for Artifact, Battle, Creature, Enchantment, Land, Planeswalker (should be true) and Instant, Sorcery (should be false)
- **Expected Result:** 6 types return true, 2 return false
- **Phase:** Already implemented (E10 resolved — `stack.rs` uses `is_permanent()`)
- **Ticket:** ALREADY-IMPLEMENTED

**110.4b** — BOUNDARY-DEF. "Permanent spell" = artifact, battle, creature, enchantment, or planeswalker spell (NOT land — lands are never cast as spells).

**ATOM-110.4b-001**
- **Rule:** 110.4b — "Permanent spell" excludes land spells (lands aren't cast)
- **Mechanism:** Permanent spell check in stack resolution
- **Minimal Board:** A creature spell on the stack
- **Action:** Check if it's a "permanent spell"
- **Expected Result:** True. An instant spell would return false. A land is not a spell at all.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**110.4c** — TESTABLE. A permanent that loses all permanent types stays on the battlefield.

**ATOM-110.4c-001**
- **Rule:** 110.4c — If a permanent loses all its permanent types, it remains on the battlefield
- **Mechanism:** No SBA or engine rule removes a typeless permanent
- **Minimal Board:** A creature on the battlefield. A continuous effect removes all its types.
- **Action:** Apply the effect. Check SBAs.
- **Expected Result:** The permanent remains on the battlefield. It's still a permanent (just typeless).
- **Phase:** Phase 5 Layers (L09 — Layer 4 type changes)
- **Ticket:** L09

**110.5** — BOUNDARY-DEF. Permanent status categories: tapped/untapped, flipped/unflipped, face up/face down, phased in/phased out.

**ATOM-110.5-001**
- **Rule:** 110.5 — Each permanent has exactly one value for each of 4 status categories
- **Mechanism:** `BattlefieldEntity` tracks tapped (bool), flipped (bool/future), face_down (bool/future), phased_out (bool/future)
- **Minimal Board:** A permanent on the battlefield
- **Action:** Check all status values
- **Expected Result:** Permanent has `tapped: false` (untapped), and future fields for flipped/face-down/phased-out
- **Phase:** Partially implemented (tapped exists). Flipped/face-down/phased-out are Phase 9 (D1, D2).
- **Ticket:** ALREADY-IMPLEMENTED (tapped) + D1/D2 (others)

**110.5a** — PURE-DEF. Status is not a characteristic. Confirmed by 109.3.

**110.5b** — TESTABLE. Permanents enter the battlefield untapped, unflipped, face up, phased in (unless spell/ability says otherwise).

**ATOM-110.5b-001**
- **Rule:** 110.5b — Permanents enter the battlefield untapped by default
- **Mechanism:** `BattlefieldEntity::new()` sets `tapped: false`
- **Minimal Board:** A creature spell resolves
- **Action:** Check the permanent's tapped status
- **Expected Result:** `tapped == false`
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-110.5b-002**
- **Rule:** 110.5b — "Unless a spell or ability says otherwise" — e.g., "enters the battlefield tapped"
- **Mechanism:** ETB replacement effect modifies default status
- **Minimal Board:** A tapland with "enters the battlefield tapped"
- **Action:** Play the tapland
- **Expected Result:** The land enters tapped
- **Phase:** Phase 6 (replacement effects — ETB tapped)
- **Ticket:** Phase 6 — ETB replacement effects

**110.5c** — TESTABLE. A permanent retains its status even if irrelevant (e.g., flipped creature becomes copy of non-flip creature — stays flipped).

**ATOM-110.5c-001**
- **Rule:** 110.5c — A permanent retains its status even when irrelevant
- **Mechanism:** Status fields persist through copy effects
- **Minimal Board:** A flipped creature. It becomes a copy of a non-flip creature.
- **Action:** Check flipped status
- **Expected Result:** Still flipped (status unchanged by copy)
- **Phase:** Phase 6 (copy effects) + Phase 9 (flip cards)
- **Ticket:** Phase 6 + Phase 9

**110.5d** — TESTABLE. Only permanents have status. Cards not on battlefield are neither tapped nor untapped.

**ATOM-110.5d-001**
- **Rule:** 110.5d — Cards not on the battlefield have no tapped/untapped status
- **Mechanism:** No `tapped` field on objects outside battlefield
- **Minimal Board:** A card in a player's graveyard
- **Action:** Query whether the card is "tapped" or "untapped"
- **Expected Result:** Neither — the card has no status. The query is inapplicable.
- **Phase:** Already implemented (tapped only exists on `BattlefieldEntity`)
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-110.5d-002**
- **Rule:** 110.5d — Face-down exiled card does NOT have the "face down" battlefield status
- **Mechanism:** Face-down in exile is hidden information, not the battlefield "face down" status
- **Minimal Board:** A card exiled face-down (e.g., by Ashiok, Dream Render)
- **Action:** Query whether the card has the "face down" status
- **Expected Result:** The card does NOT have the battlefield face-down status. "Face down" as a status only exists on battlefield permanents. In exile, face-down is a visibility/information restriction, not a permanent status.
- **Phase:** Phase 5-Pre (data model) + Phase 9 (face-down gameplay)
- **Ticket:** Phase 5-Pre — Face-down data model, Phase 9 — Face-down gameplay
- **Architectural note (Phase 5-Pre concern):** The `is_face_down` field on `BattlefieldEntity` is already the right place for battlefield face-down status. For exile, face-down is a property of the exile zone entry — we likely need a `face_down: bool` on whatever struct represents an exile zone entry (currently just zone membership). This is core data model work that should be done in Phase 5-Pre alongside other zone-state fields, not deferred to Phase 9, to avoid retrofitting. The *gameplay* of face-down cards (morph, manifest, disguise) is Phase 9, but the data model slot should exist earlier.

---

## Rules 111–113: Tokens, Spells, Abilities

### 111. Tokens

**111.1** — PURE-DEF. Tokens are markers representing permanents not represented by cards.

**111.2** — TESTABLE. The player who creates a token is its owner. Token enters under that player's control.

**ATOM-111.2-001**
- **Rule:** 111.2 — Token creator is its owner; token enters under creator's control
- **Mechanism:** Token creation sets `owner` and `controller` to the creating player
- **Minimal Board:** P0 resolves an effect that creates a 1/1 token
- **Action:** Create the token
- **Expected Result:** Token's `owner == P0` and `BattlefieldEntity.controller == P0`
- **Phase:** Phase 8 (CreateToken primitive)
- **Ticket:** Phase 8 — Token creation pipeline

**111.3** — TESTABLE. Token characteristics are defined by the creating spell/ability. These are its copiable values. A token has no characteristics not defined by its creator.

**ATOM-111.3-001**
- **Rule:** 111.3 — A token's characteristics are exactly those defined by the creating effect; nothing more
- **Mechanism:** Token `CardData` only has fields explicitly set by the creating effect
- **Minimal Board:** Effect creates a "1/1 green Saproling creature token"
- **Action:** Query the token's characteristics
- **Expected Result:** Token has: types=[Creature], subtypes=[Saproling], colors=[Green], P/T=1/1. It has NO mana cost, NO supertypes, NO rules text, NO abilities. (Name defaults to "Saproling Token" per 111.4 — tested separately.)
- **Phase:** Phase 8 (CreateToken primitive)
- **Ticket:** Phase 8
- **Note:** This atom focuses purely on "characteristics = only what the effect specifies." Default value behavior (name defaulting, mana cost being absent) is tested in 111.4 atoms.

**111.4** — TESTABLE. Token name defaults to subtype(s) + "Token" if not specified by the creating effect.

**ATOM-111.4-001**
- **Rule:** 111.4 — Unspecified token name = subtype(s) + "Token"
- **Mechanism:** Token naming logic in token factory
- **Minimal Board:** Effect creates "two 2/1 red Dwarf Berserker creature tokens"
- **Action:** Create the tokens
- **Expected Result:** Each token's name is "Dwarf Berserker Token"
- **Phase:** Phase 8
- **Ticket:** Phase 8

**ATOM-111.4-002**
- **Rule:** 111.4 — If creating effect specifies a name, use that name instead
- **Mechanism:** Named token creation
- **Minimal Board:** Effect creates "Boo, a legendary 1/1 red Hamster creature token with trample and haste"
- **Action:** Create the token
- **Expected Result:** Token's name is "Boo" (not "Hamster Token")
- **Phase:** Phase 8
- **Ticket:** Phase 8

**111.5** — TESTABLE. If a permanent with the token's characteristics can't enter the battlefield, the token is not created. Also, no token for copy of instant/sorcery.

**ATOM-111.5-001**
- **Rule:** 111.5 — Token that is a copy of an instant or sorcery card is not created
- **Mechanism:** Token creation guard for non-permanent card types
- **Minimal Board:** An effect says "create a token that's a copy of target card" targeting an instant card
- **Action:** Attempt to create the token
- **Expected Result:** No token is created
- **Phase:** Phase 8 (token creation + copy)
- **Ticket:** Phase 8

**ATOM-111.5-002**
- **Rule:** 111.5 — Token not created if an effect prevents permanents with its characteristics from entering
- **Mechanism:** Token creation guard checks ETB prevention effects (cross-cut with 101.2 "can't beats can")
- **Minimal Board:** An effect says "creatures can't enter the battlefield" is active. Another effect says "create a 1/1 creature token."
- **Action:** Attempt to create the token
- **Expected Result:** No token is created (the ETB prevention prevents the token from ever existing)
- **Phase:** Phase 8 (token creation + ETB prevention)
- **Ticket:** Phase 8 — Token creation ETB prevention guard

**111.6** — PURE-DEF. Tokens are subject to all permanent/type rules. Tokens aren't cards.

**111.7** — TESTABLE. A token in a zone other than the battlefield ceases to exist (SBA).

**ATOM-111.7-001**
- **Rule:** 111.7 — Token in a non-battlefield zone ceases to exist as an SBA
- **Mechanism:** SBA check for tokens in non-battlefield zones (704.5d)
- **Minimal Board:** A token creature that was destroyed and is now in the graveyard
- **Action:** Check SBAs
- **Expected Result:** Token is removed from the graveyard (ceases to exist). Note: triggered abilities that trigger on the zone change fire BEFORE the token ceases to exist.
- **Phase:** Phase 5-Pre (T13 — token cease-to-exist SBA)
- **Ticket:** T13

**111.8** — TESTABLE. A token that has left the battlefield can't move to another zone or return. It ceases to exist at next SBA check.

**ATOM-111.8-001**
- **Rule:** 111.8 — A token that has left the battlefield can't change zones again
- **Mechanism:** Zone change prevention for off-battlefield tokens
- **Minimal Board:** A token in the graveyard. An effect tries to return it to the battlefield.
- **Action:** Attempt to move the token from graveyard to battlefield
- **Expected Result:** The token remains in the graveyard. It ceases to exist at the next SBA check.
- **Phase:** Phase 8 (token zone-change prevention)
- **Ticket:** NEW — Token zone-change lock after leaving battlefield

**ATOM-111.8-002**
- **Rule:** 111.8 — A token bounced to hand ceases to exist as SBA
- **Mechanism:** Token in hand zone triggers SBA cease-to-exist
- **Minimal Board:** A token creature on the battlefield. An effect says "return target creature to its owner's hand."
- **Action:** Bounce the token. Check SBAs.
- **Expected Result:** Token momentarily enters hand zone. At next SBA check, it ceases to exist (removed from game entirely). It is NOT in hand afterward.
- **Phase:** Phase 5-Pre (T13 — token cease-to-exist SBA)
- **Ticket:** T13

**111.9** — PURE-DEF. Legendary token naming format. Syntactic sugar.

**111.10–111.10v** — BOUNDARY-DEF. Predefined token types (Treasure, Food, Gold, Clue, Blood, etc.).

**ATOM-111.10-001**
- **Rule:** 111.10a–v — Predefined tokens have specific characteristics
- **Mechanism:** Token factory / predefined token definitions
- **Minimal Board:** Effect says "create a Treasure token"
- **Action:** Create the token
- **Expected Result:** Token is a colorless Treasure artifact with "{T}, Sacrifice this token: Add one mana of any color."
- **Phase:** Phase 8 (token creation)
- **Ticket:** Phase 8 — Predefined token definitions

**ATOM-111.10-002**
- **Rule:** 111.10 — Food token has specific predefined characteristics
- **Mechanism:** Token factory / predefined token definitions
- **Minimal Board:** Effect says "create a Food token"
- **Action:** Create the token
- **Expected Result:** Token is a colorless Food artifact with "{2}, {T}, Sacrifice this token: You gain 3 life."
- **Phase:** Phase 8
- **Ticket:** Phase 8

**111.11** — TESTABLE. Token created by name (not predefined) uses Oracle card reference for characteristics.

**ATOM-111.11-001**
- **Rule:** 111.11 — Token created by non-predefined name uses the named card's Oracle characteristics
- **Mechanism:** Token creation looks up `CardRegistry` for the named card
- **Minimal Board:** Effect says "create a Tarmogoyf token" (Tarmogoyf is not a predefined token type)
- **Action:** Create the token
- **Expected Result:** Token has the same characteristics as the card named Tarmogoyf from the Oracle reference
- **Phase:** Phase 8
- **Ticket:** Phase 8

**111.12** — TESTABLE. Creating a token that's a copy of a nonexistent object → no token created.

**ATOM-111.12-001**
- **Rule:** 111.12 — Copy of nonexistent object → no token created
- **Mechanism:** Copy-token creation guard
- **Minimal Board:** Mimic Vat with no exiled card. Activate "Create a token that's a copy of a card exiled with this."
- **Action:** Attempt to create the token
- **Expected Result:** No token is created (source object doesn't exist)
- **Phase:** Phase 8
- **Ticket:** Phase 8

**111.13** — TESTABLE. A copy of a permanent spell becomes a token (not "created" — no ETB creation triggers).

**ATOM-111.13-001**
- **Rule:** 111.13 — Copy of permanent spell becomes a token on resolution, but is not "created"
- **Mechanism:** Spell copy resolution path
- **Minimal Board:** A copy of a creature spell is on the stack
- **Action:** The copy resolves
- **Expected Result:** A token enters the battlefield with the spell's characteristics. No "create a token" triggers fire (it wasn't "created").
- **Phase:** Phase 7 (spell copying via D19)
- **Ticket:** D19 — Spell copying

---

### 112. Spells

**112.1** — PURE-DEF. A spell is a card on the stack. Becomes a spell when cast (moved to stack). Remains until it resolves, is countered, or leaves.

**112.1a** — PURE-DEF. A copy of a spell is also a spell.

**112.1b** — PURE-DEF. Casting a copy of a card creates a spell.

**112.2** — TESTABLE. Spell's owner = card's owner (or controller if it's a copy with no card). Spell's controller = player who put it on the stack.

**ATOM-112.2-001**
- **Rule:** 112.2 — Spell's controller is the player who put it on the stack
- **Mechanism:** `StackEntry.controller` set during `cast_spell`
- **Minimal Board:** P0 casts Lightning Bolt
- **Action:** Check the spell's controller on the stack
- **Expected Result:** `StackEntry.controller == P0`
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-112.2-002**
- **Rule:** 112.2 — A copy of a spell on the stack has the copier as controller
- **Mechanism:** Spell copy `StackEntry.controller` set to the player who created the copy
- **Minimal Board:** P0 casts Lightning Bolt. P1 casts Fork ("Copy target instant or sorcery spell") targeting it.
- **Action:** Fork resolves, creating a copy of Lightning Bolt on the stack
- **Expected Result:** Original Lightning Bolt has `controller == P0`. The copy has `controller == P1`.
- **Phase:** Phase 7 (spell copying via D19)
- **Ticket:** D19 — Spell copying

**112.2a** — PURE-DEF. Copy created by an effect — owner is the player instructed to create/cast it.

**112.3** — PURE-DEF. Noncopy spell's characteristics = printed + continuous effects. Reference to 613.

**112.4** — TESTABLE. If a resolving spell/ability changes characteristics of a permanent spell, the change persists on the permanent.

**ATOM-112.4-001**
- **Rule:** 112.4 — Characteristic changes to a permanent spell persist on the resulting permanent
- **Mechanism:** Continuous effects on spells carry through resolution (rule 400.7)
- **Minimal Board:** A black creature spell on the stack. An effect changes it to white.
- **Action:** The spell resolves, becoming a permanent.
- **Expected Result:** The permanent enters the battlefield as white (the color change persists for the effect's duration)
- **Phase:** Phase 5 Layers (continuous effects on stack — deferred per D16 in roadmap)
- **Ticket:** Post-v1 (D16 from session 6b — continuous effects on stack)

---

### 113. Abilities

**113.1** — PURE-DEF. Three kinds of abilities: characteristic on object, player ability, activated/triggered on stack. Framework.

**113.1a** — PURE-DEF. Abilities are defined by rules text or creating effects. Generate effects.

**113.1b** — PURE-DEF. Player abilities. Granted by effects.

**113.1c** — PURE-DEF. Activated/triggered abilities on the stack are objects.

**113.2** — PURE-DEF. Abilities can affect their own object or other objects/players.

**113.2a** — PURE-DEF. Abilities can be beneficial or detrimental.

**113.2b** — PURE-DEF. Additional/alternative costs are abilities of the card.

**113.2c** — PURE-DEF. Multiple abilities per object. Each instance functions independently.

**113.2d** — PURE-DEF. Abilities generate one-shot or continuous effects. Some are replacement/prevention.

**113.3** — BOUNDARY-DEF. Four categories of abilities: spell, activated, triggered, static.

**ATOM-113.3-001**
- **Rule:** 113.3 — Four ability categories exist in the type system
- **Mechanism:** `AbilityType` or equivalent enum
- **Minimal Board:** N/A (type system check)
- **Action:** Verify ability type enum has: Mana (subset of activated), Activated, Triggered, Static. (Spell abilities are implicit in instant/sorcery resolution.)
- **Expected Result:** All relevant types are representable
- **Phase:** Already implemented in `AbilityDef.ability_type`
- **Ticket:** ALREADY-IMPLEMENTED

**113.3a** — PURE-DEF. Spell abilities: instructions during instant/sorcery resolution.

**113.3b** — PURE-DEF. Activated abilities: "[Cost]: [Effect]." Player activates with priority. Goes on stack.

**113.3c** — PURE-DEF. Triggered abilities: "when/whenever/at" condition. Goes on stack when triggered.

**113.3d** — PURE-DEF. Static abilities: always-true statements creating continuous effects.

**113.4** — TESTABLE. Mana abilities don't use the stack and can be activated without priority (under certain circumstances).

**ATOM-113.4-001**
- **Rule:** 113.4 — Mana abilities don't use the stack; they resolve immediately
- **Mechanism:** `activate_mana_ability()` resolves immediately without going on stack
- **Minimal Board:** A Forest on the battlefield
- **Action:** Activate the Forest's mana ability
- **Expected Result:** {G} is added to the mana pool immediately. No stack entry is created.
- **Phase:** Already implemented (Phase 4.5f fix — mana abilities route to `activate_mana_ability()`)
- **Ticket:** ALREADY-IMPLEMENTED

**113.5** — TESTABLE. Loyalty abilities: can activate only at sorcery speed, once per turn per permanent.

**ATOM-113.5-001**
- **Rule:** 113.5 — Loyalty ability can only be activated when player has priority, stack empty, main phase, and no loyalty ability of that permanent was activated this turn
- **Mechanism:** Loyalty ability activation restrictions
- **Minimal Board:** A planeswalker with a [+1] ability. It's P0's main phase, stack is empty.
- **Action:** Activate the [+1] ability. Then attempt to activate another loyalty ability of the same planeswalker.
- **Expected Result:** First activation succeeds. Second activation fails (once per turn per permanent).
- **Phase:** Phase 8 (loyalty abilities)
- **Ticket:** Phase 8 — Loyalty ability restrictions

**113.6** — TESTABLE. Abilities of instant/sorcery function only on stack. Others function only on battlefield. Exceptions listed below.

**ATOM-113.6-001**
- **Rule:** 113.6 — Ability of a permanent functions only on the battlefield (default)
- **Mechanism:** Static/triggered abilities only active while source is on battlefield
- **Minimal Board:** A creature with "Creatures you control get +1/+1" in the graveyard (not on battlefield)
- **Action:** Check if the ability applies
- **Expected Result:** The ability does NOT apply (not on battlefield)
- **Phase:** Phase 5 Layers (static ability registration on ETB, removal on zone exit)
- **Ticket:** L03 (static ability registration)

**113.6a** — TESTABLE. CDAs function everywhere, even outside the game.

**ATOM-113.6a-001**
- **Rule:** 113.6a — Characteristic-defining abilities function in all zones
- **Mechanism:** CDA evaluation in `compute_characteristics` regardless of zone
- **Minimal Board:** Tarmogoyf card in a player's graveyard
- **Action:** Query Tarmogoyf's P/T (its CDA defines its power and toughness)
- **Expected Result:** P/T is computed based on card types in graveyards, even though Tarmogoyf is not on the battlefield
- **Phase:** Phase 5 Layers (L18 — CDA handling in all zones)
- **Ticket:** L18

**113.6b** — META. Describes a *pattern* (some keyword abilities specify activation from specific zones) rather than a specific testable mechanic. The actual tests belong with each keyword: Cycling from hand, Unearth from graveyard, Scavenge from graveyard, Channel from hand, etc.

**ATOM-113.6b-001** (retained as a pattern test)
- **Rule:** 113.6b — "Activate this ability only from your graveyard" functions only in the graveyard
- **Mechanism:** `activation_zone` on `AbilityDef`
- **Minimal Board:** Reassembling Skeleton in graveyard with "{1}{B}: Return this card from your graveyard to the battlefield tapped"
- **Action:** Check if the ability can be activated from graveyard (yes) vs. from battlefield (no)
- **Expected Result:** Activatable from graveyard only
- **Phase:** Phase 5-Pre (T19 — zone-activated abilities)
- **Ticket:** T19
- **Note:** Keywords that use this pattern: Cycling (hand), Unearth (graveyard), Scavenge (graveyard), Channel (hand), Transmute (hand), Escape (graveyard).

**113.6c** — PURE-DEF. Ability states zones where it doesn't function → works everywhere else.

**113.6d** — PURE-DEF. Alternative cost abilities function on the stack. Framework.

**113.6e** — PURE-DEF. Casting restriction abilities function in playable zones + stack.

**113.6f** — PURE-DEF. Zone-restriction abilities function everywhere.

**113.6g** — PURE-DEF. "Can't be countered"/"can't be copied" functions on stack.

**113.6h** — PURE-DEF. ETB-modifying abilities function as the object enters. Reference to 614.12.

**113.6i** — PURE-DEF. "Counters can't be put on" functions on entry + battlefield.

**113.6j** — TESTABLE. Activated ability whose cost can't be paid on battlefield functions from any zone where cost is payable.

**ATOM-113.6j-001**
- **Rule:** 113.6j — An activated ability with a cost unpayable on battlefield activates from any zone where it's payable
- **Mechanism:** Zone-agnostic activation for abilities with zone-locked costs
- **Minimal Board:** A card in the graveyard with "{2}, Exile this card from your graveyard: [effect]"
- **Action:** Activate the ability from the graveyard
- **Expected Result:** Activation succeeds (cost requires exile from graveyard, which is payable from graveyard)
- **Phase:** Phase 5-Pre (T19)
- **Ticket:** T19

**113.6k** — PURE-DEF. Trigger conditions that can't trigger from battlefield function in other zones.

**113.6m** — PURE-DEF. Ability that moves its object from a specific zone functions only in that zone (with exceptions).

**113.6n** — DEFERRED. Deck construction abilities (e.g. "This card can be your commander." when the card is not a legendary creature). Pre-game, not engine.

**113.6p** — DEFERRED. Emblem/plane/vanguard/scheme/conspiracy abilities in command zone.

**113.7** — PURE-DEF. Source of ability = the object that generated it. Naming.

**113.7a** — TESTABLE. Once on stack, ability exists independently of source. Destroying source doesn't affect the ability.

**ATOM-113.7a-001**
- **Rule:** 113.7a — Ability on stack exists independently of its source; destroying source doesn't affect it
- **Mechanism:** Ability on stack is a separate object, not linked to source's existence
- **Minimal Board:** Creature with activated ability "{T}: Deal 1 damage to any target." Ability is on the stack. Creature is destroyed.
- **Action:** The ability resolves.
- **Expected Result:** The ability still resolves and deals 1 damage, even though its source is gone. (Uses LKI for source characteristics if needed.)
- **Phase:** Already implemented (abilities are separate `StackEntry` objects)
- **Ticket:** ALREADY-IMPLEMENTED

**113.8** — PURE-DEF. Controller of activated ability = activator. Controller of triggered ability = controller of source when triggered.

**113.9** — TESTABLE. Activated/triggered abilities on the stack are NOT spells. Can't be countered by "counter target spell."

**ATOM-113.9-001**
- **Rule:** 113.9 — Activated/triggered abilities can't be countered by effects that counter only spells
- **Mechanism:** `CounterSpell` primitive checks target is a spell, not an ability
- **Minimal Board:** An activated ability on the stack. A Counterspell targeting it.
- **Action:** Attempt to target the ability with Counterspell
- **Expected Result:** Targeting fails — Counterspell says "Counter target spell" and the ability is not a spell
- **Phase:** Already implemented (targeting validates spell vs. ability)
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-113.9-002**
- **Rule:** 113.9 — Effects that specifically counter activated abilities CAN counter them
- **Mechanism:** `CounterActivatedAbility` primitive targeting an activated ability on the stack
- **Minimal Board:** An activated ability on the stack. A spell resolving with effect `CounterActivatedAbility` targeting it.
- **Action:** Resolve the counter effect
- **Expected Result:** The activated ability is countered and removed from the stack.
- **Phase:** Already implemented (`CounterAbility` primitive) + Phase 7 (activated-only targeting restriction)
- **Ticket:** ALREADY-IMPLEMENTED (base) + Phase 7 (activated-only targeting)
- **Note:** Real cards using this primitive (Bind, Interdict, etc.) compose it with other effects (draw a card, etc.). If the primitive works in isolation, it works composed.

**ATOM-113.9-003**
- **Rule:** 113.9 — Effects that specifically counter triggered abilities CAN counter them
- **Mechanism:** `CounterTriggeredAbility` primitive targeting a triggered ability on the stack
- **Minimal Board:** A triggered ability on the stack (e.g., ETB trigger). A spell resolving with effect `CounterTriggeredAbility` targeting it.
- **Action:** Resolve the counter effect
- **Expected Result:** The triggered ability is countered and removed from the stack.
- **Phase:** Phase 7 (triggered abilities on stack + triggered-only targeting)
- **Ticket:** Phase 7
- **Note:** Real cards using this primitive (Consign to Memory, Voidslime's triggered mode, etc.) compose it with other effects. If the primitive works in isolation, it works composed.

**113.10** — PURE-DEF. Effects can add/remove abilities. "Gains"/"has" adds; "loses" removes. Framework for Layer 6.

**113.10a** — PURE-DEF. Activation instructions added with the ability.

**113.10b** — TESTABLE. Removing an ability removes ALL instances of it.

**ATOM-113.10b-001**
- **Rule:** 113.10b — Effects that remove an ability remove all instances of it
- **Mechanism:** Layer 6 ability removal affects all instances
- **Minimal Board:** A creature with two instances of flying (one printed, one granted). An effect says "loses flying."
- **Action:** Apply the effect
- **Expected Result:** Creature has zero instances of flying (both removed)
- **Phase:** Phase 5 Layers (L06)
- **Ticket:** L06

**113.10c** — PURE-DEF. Most recent add/remove prevails. Reference to 613.

**113.11** — TESTABLE. "Can't have" an ability = loses it if it has it, and can't gain it.

**ATOM-113.11-001**
- **Rule:** 113.11 — An effect that says an object "can't have" an ability prevents gaining and removes existing instances
- **Mechanism:** "Can't have" prohibition in Layer 6
- **Minimal Board:** A creature with flying. An effect says "creatures can't have flying."
- **Action:** Apply the effect. Then a second effect tries to give the creature flying.
- **Expected Result:** Creature has no flying (lost it). The second effect's flying grant does not apply. Other parts of the second effect still apply.
- **Phase:** Phase 8 (D10 — "can't have" ability prohibition)
- **Ticket:** D10
- **Cross-ref:** The "other parts of the second effect still apply" case (partial resolution due to "can't have") belongs with 608 (resolving spells and abilities) in the Chapter 6 session, not here.

**113.12** — BOUNDARY-DEF. Distinction between "granting an ability" vs. "defining a characteristic" vs. "stating a quality." The engine needs to know which abilities are CDAs so it can apply them in the correct layer (Layer 7a for P/T CDAs, Layer 5 for color CDAs like Devoid). More than pure naming — this is an engine-relevant classification.

**ATOM-113.12-001**
- **Rule:** 113.12 — A P/T CDA is applied in Layer 7a, not as a granted ability
- **Mechanism:** CDA classification in ability processing routes to correct layer
- **Minimal Board:** Tarmogoyf (CDA: power/toughness based on card types in graveyards) on the battlefield
- **Action:** Verify that Tarmogoyf's P/T is computed in Layer 7a (CDA sublayer), not Layer 7d (counters) or Layer 7c (modifications)
- **Expected Result:** P/T computed in Layer 7a. A "remove all abilities" effect does NOT remove the CDA-defined P/T (CDAs define characteristics, not grant abilities in the Layer 6 sense).
- **Phase:** Phase 5 Layers (L18 — CDA handling)
- **Ticket:** L18

**ATOM-113.12-002**
- **Rule:** 113.12 — A color CDA is applied in Layer 5, not Layer 6
- **Mechanism:** CDA classification routes color-defining CDAs to Layer 5 (color), not Layer 6 (abilities)
- **Minimal Board:** A card with Devoid (CDA: "this card is colorless") on the battlefield. The card's mana cost contains colored symbols.
- **Action:** Verify the card's color is computed in Layer 5, and that "remove all abilities" does not restore the card's color
- **Expected Result:** Card is colorless (Layer 5 CDA overrides mana-cost-derived color). A "remove all abilities" effect (Layer 6) does NOT re-add the color, because Devoid is a CDA applied in Layer 5, which is evaluated before Layer 6. The characteristic is set, not granted as an ability.
- **Phase:** Phase 5 Layers (L18 — CDA handling)
- **Ticket:** L18
- **Note:** Specific CDA tests deferred to 604 session. This atom tests the *classification* mechanism only.

---

## Rules 114–116: Emblems, Targets, Special Actions

### 114. Emblems

**114.1** — PURE-DEF. Emblems are markers in the command zone with one or more abilities. Naming.

**114.2** — TESTABLE. "[Player] gets an emblem with [ability]" → player puts emblem into command zone, owned and controlled by that player.

**ATOM-114.2-001**
- **Rule:** 114.2 — An emblem is owned and controlled by the player who received it, placed in the command zone
- **Mechanism:** Emblem creation in `CreateEmblem` effect
- **Minimal Board:** A planeswalker ultimate resolves: "You get an emblem with 'Creatures you control get +1/+1'"
- **Action:** Resolve the ability
- **Expected Result:** P0 has an emblem in the command zone. Emblem `owner == P0`, `controller == P0`. The emblem's ability applies to P0's creatures.
- **Phase:** Phase 8 (emblem creation)
- **Ticket:** NEW — Emblem object type + creation

**114.3** — BOUNDARY-DEF. Emblem has no characteristics other than abilities. No types, no mana cost, no color, usually no name.

**ATOM-114.3-001**
- **Rule:** 114.3 — Emblem has no types, mana cost, or color
- **Mechanism:** Emblem `CardData` / characteristics are minimal
- **Minimal Board:** An emblem in the command zone
- **Action:** Query the emblem's characteristics
- **Expected Result:** No types, no mana cost, no color, no P/T, no loyalty. Only abilities defined by the creating effect.
- **Phase:** Phase 8
- **Ticket:** Phase 8 — Emblem characteristics

**114.4** — PURE-DEF. Emblem abilities function in the command zone. Cross-ref 113.6p.

**114.5** — BOUNDARY-DEF. An emblem is neither a card nor a permanent. "Emblem" is not a card type.

**ATOM-114.5-001**
- **Rule:** 114.5 — Emblem is not a card, not a permanent, not a card type
- **Mechanism:** Emblem type checks
- **Minimal Board:** An emblem in the command zone
- **Action:** Check `is_card()`, `is_permanent()`, `card_types`
- **Expected Result:** All return false/empty. "Destroy target permanent" cannot target an emblem.
- **Phase:** Phase 8
- **Ticket:** Phase 8

---

### 115. Targets

**115.1** — PURE-DEF. Some spells/abilities require choosing targets. Targets declared on stack. Can't be changed except by specific effects.

**115.1a** — PURE-DEF. Instant/sorcery is targeted if it uses "target [something]." Targets chosen on cast.

**115.1b** — TESTABLE. Aura spells are always targeted (enchant keyword specifies target). Aura permanent does not target.

**ATOM-115.1b-001**
- **Rule:** 115.1b — Aura spell targets a legal object per its enchant keyword; Aura permanent does not target
- **Mechanism:** Aura spell targeting during cast + Aura permanent attachment without targeting
- **Minimal Board:** An Aura with "Enchant creature" cast targeting a creature with hexproof
- **Action:** Attempt to cast the Aura targeting the hexproof creature
- **Expected Result:** Casting fails (illegal target — hexproof). But if the Aura is already on the battlefield attached to a creature (e.g., put there by an effect), the hexproof creature can be the enchanted creature (Aura permanents don't target).
- **Phase:** Phase 8 (Aura implementation)
- **Ticket:** Phase 8 — Aura spell targeting vs. attachment

**115.1c** — PURE-DEF. Activated ability targets chosen on activation.

**115.1d** — PURE-DEF. Triggered ability targets chosen when put on stack.

**115.1e** — PURE-DEF. Some keyword abilities represent targeted abilities.

**115.2** — TESTABLE. Only permanents are legal targets by default (unless the spell/ability specifies another zone or targets a spell/ability).

**ATOM-115.2-001**
- **Rule:** 115.2 — Default target zone is the battlefield (permanents)
- **Mechanism:** Target legality check defaults to battlefield zone
- **Minimal Board:** Creature on battlefield + creature card in graveyard. Spell says "target creature."
- **Action:** Choose legal targets
- **Expected Result:** Only the battlefield creature is legal. The graveyard card is not.
- **Phase:** Already implemented (targeting defaults to battlefield)
- **Ticket:** ALREADY-IMPLEMENTED

**115.3** — TESTABLE. Same target can't be chosen multiple times for one instance of "target." Different instances of "target" can share the same target.

**ATOM-115.3-001**
- **Rule:** 115.3 — Same target can't be chosen twice for one instance of "target"
- **Mechanism:** Target deduplication per "target" instance in target selection
- **Minimal Board:** A spell with "target creature gets +1/+1 and target creature gets -1/-1" (two separate instances of "target")
- **Action:** Choose the same creature for both targets
- **Expected Result:** Legal — different instances of "target" can share the same target
- **Phase:** Phase 5-Pre (T18 — 601.2c targeting)
- **Ticket:** T18

**ATOM-115.3-002**
- **Rule:** 115.3 — Same target can't be chosen twice for one "target" instance
- **Mechanism:** Target deduplication
- **Minimal Board:** A spell with "choose two target creatures" (one instance of "target" requiring two choices)
- **Action:** Attempt to choose the same creature twice
- **Expected Result:** Illegal — same object can't be chosen twice for the same "target" instance
- **Phase:** Phase 5-Pre (T18)
- **Ticket:** T18

**115.4** — TESTABLE. "Any target" / "another target" / "two targets" can be creatures, players, planeswalkers, or battles.

**ATOM-115.4-001**
- **Rule:** 115.4 — "Any target" can be a creature, player, planeswalker, or battle
- **Mechanism:** `TargetSpec::AnyTarget` accepts all four categories
- **Minimal Board:** Creature, planeswalker, and player available. Spell says "deal 3 damage to any target."
- **Action:** Each of creature, planeswalker, and player should be legal targets
- **Expected Result:** All three are legal targets. A noncreature artifact is NOT a legal target.
- **Phase:** Already implemented (`TargetSpec::AnyTarget`)
- **Ticket:** ALREADY-IMPLEMENTED

**115.5** — TESTABLE. A spell or ability on the stack is an illegal target for itself.

**ATOM-115.5-001**
- **Rule:** 115.5 — A spell/ability can't target itself
- **Mechanism:** Self-targeting prevention in target legality check
- **Minimal Board:** A spell on the stack that says "counter target spell"
- **Action:** Attempt to target itself
- **Expected Result:** Illegal target — can't target itself
- **Phase:** Already implemented (target validation excludes self)
- **Ticket:** ALREADY-IMPLEMENTED

**115.6** — TESTABLE. "Up to N targets" means 0 is valid, and the spell/ability still resolves (with no targets). Engine must support zero-target selection.

**ATOM-115.6-001**
- **Rule:** 115.6 — "Up to" allows choosing zero targets; spell still resolves
- **Mechanism:** Target selection allows count of 0 for "up to" targeting
- **Minimal Board:** A spell with "up to two target creatures get +1/+1." No creatures on battlefield.
- **Action:** Cast the spell choosing 0 targets
- **Expected Result:** Spell is legal to cast with 0 targets. It resolves (doing nothing). Per 115.6: the spell "is still said to require targets, but that spell or ability is targeted only if one or more targets have been chosen for it." So with 0 targets chosen, the spell is *not* targeted — it won't be affected by "counter target spell that targets" or similar.
- **Phase:** Phase 5-Pre (T18 — targeting)
- **Ticket:** T18

**ATOM-115.3/4-001**
- **Rule:** 115.3/115.4 — Spell with two separate target requirements must choose distinct legal targets for each
- **Mechanism:** Target selection validates that each target slot is filled with a distinct legal object
- **Minimal Board:** A spell with "Destroy target creature and target enchantment." One creature and one enchantment on the battlefield (distinct objects).
- **Action:** Cast the spell choosing the creature for slot 1 and the enchantment for slot 2
- **Expected Result:** Legal cast — two distinct legal objects, each satisfying its respective target requirement.
- **Phase:** Phase 5-Pre (T18 — targeting)
- **Ticket:** T18

**ATOM-115.3/4-002** (sad path)
- **Rule:** 115.3/115.4 — A single object can't fill two "target" slots even if it satisfies both criteria
- **Mechanism:** Target deduplication across separate "target" words
- **Minimal Board:** A spell with "Destroy target creature and target enchantment." Only one object on the battlefield: an enchantment creature (e.g., Courser of Kruphix).
- **Action:** Attempt to cast the spell, choosing the enchantment creature for both slots
- **Expected Result:** Illegal — the spell has two separate instances of the word "target", so it needs two distinct objects. A single enchantment creature can't be chosen for both. The spell is uncastable with only one legal object on the battlefield.
- **Phase:** Phase 5-Pre (T18 — targeting)
- **Ticket:** T18
- **Cross-ref:** Detailed targeting choice mechanics (including "another target" deduplication) belong with 601.2c in the Chapter 6 session.

**115.7** — PURE-DEF. Some effects change targets or choose new targets. Framework.

**115.7a–f** — PURE-DEF / DEFERRED. Details on changing/choosing new targets, division/distribution. Phase 8+.

**115.8** — TESTABLE (DEFERRED to Phase 8). Target-changing effects (Deflection, Redirect, Spellskite). Testable but requires target-change infrastructure.

**ATOM-115.8-001**
- **Rule:** 115.8 — An effect can change the target of a spell/ability
- **Mechanism:** Target replacement on stack entries
- **Minimal Board:** P0 casts Lightning Bolt targeting P1's creature. P1 activates Spellskite ("Change a target of target spell or ability to Spellskite").
- **Action:** Resolve Spellskite's ability
- **Expected Result:** Lightning Bolt's target is now Spellskite instead of the original creature. Lightning Bolt resolves dealing 3 damage to Spellskite.
- **Phase:** Phase 8 (target-changing effects)
- **Ticket:** NEW — Target-changing effects

**115.9a** — TESTABLE (DEFERRED to Phase 7). Triggered abilities that count targets on spells (e.g., Voracious Bibliophile). Requires the engine to count targets on stack entries.

**ATOM-115.9a-001**
- **Rule:** 115.9a — Engine can count the number of targets a spell has
- **Mechanism:** `StackEntry.targets.len()` or equivalent query
- **Minimal Board:** A spell with 2 targets on the stack. Voracious Bibliophile ("whenever you cast a spell with one or more targets, draw that many cards") triggers.
- **Action:** Trigger resolves
- **Expected Result:** Player draws 2 cards (spell had 2 targets)
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7

**115.9b–c** — TESTABLE (DEFERRED). "Targets only [something]" vs. "targets [something]" nuances. Phase 7/8.

**115.10** — PURE-DEF. Spells/abilities can affect non-targets. Non-targets aren't chosen until resolution.

**115.10a** — PURE-DEF. "You" is not a target.

**115.10b** — PURE-DEF. Restates 115.10a more specifically.

---

### 116. Special Actions

**116.1** — PURE-DEF. Special actions don't use the stack. Distinct from TBAs and SBAs. Framework.

**116.2** — BOUNDARY-DEF. Twelve special actions enumerated.

**116.2a** — TESTABLE. Playing a land is a special action. Once per turn, during main phase with empty stack and priority.

**ATOM-116.2a-001**
- **Rule:** 116.2a — Playing a land is a special action, once per turn, main phase, stack empty
- **Mechanism:** `play_land()` checks `lands_played_this_turn < max_land_plays`, phase is main, stack is empty
- **Minimal Board:** P0's main phase, stack empty, P0 has a land in hand, hasn't played a land this turn
- **Action:** Play the land
- **Expected Result:** Land enters the battlefield. `lands_played_this_turn` increments to 1.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-116.2a-002**
- **Rule:** 116.2a — Can't play a second land unless an effect allows it
- **Mechanism:** `play_land()` rejects when `lands_played_this_turn >= max_land_plays`
- **Minimal Board:** P0 has already played a land this turn. Attempts to play a second.
- **Action:** Attempt `play_land()`
- **Expected Result:** Rejected — already played max lands this turn
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**116.2b** — DEFERRED. Turning face-down creature face up (morph). Phase 9.

**116.2c** — DEFERRED. Ending continuous effects as a special action (e.g., Dominating Licid turning into an Aura). Licid mechanic is niche but tests the "special action" framework. Deferred to Phase 8+.

**ATOM-116.2c-001**
- **Rule:** 116.2c — A player can end a continuous effect as a special action
- **Mechanism:** Special action framework for ending effects
- **Minimal Board:** Dominating Licid is attached to a creature as an Aura. Its controller wants to end the effect (turning it back into a creature).
- **Action:** Take the special action to end the continuous effect
- **Expected Result:** Licid detaches and becomes a creature again. This doesn't use the stack.
- **Phase:** Phase 8+ (Licid implementation)
- **Ticket:** NEW — Special action: end continuous effect

**116.2d** — DEFERRED. Ignoring a static ability's effect for a duration as a special action (e.g., Leonin Arbiter: "pay {2} to search"). Tests "take a special action to ignore a restriction."

**ATOM-116.2d-001**
- **Rule:** 116.2d — A player can take a special action to ignore a restriction
- **Mechanism:** Special action payment to override a static restriction
- **Minimal Board:** Leonin Arbiter on battlefield ("Players can't search libraries. Any player may pay {2} for that player to ignore this effect for the rest of the turn."). P0 wants to search.
- **Action:** P0 pays {2} as a special action
- **Expected Result:** P0 can search libraries for the rest of the turn. This doesn't use the stack.
- **Phase:** Phase 8 (restriction-override special actions)
- **Ticket:** NEW — Special action: pay to ignore restriction

**116.2e** — DEFERRED. Circling Vultures specific card rule.

**116.2f** — DEFERRED. Suspend. Phase 9.

**116.2g** — TESTABLE. Companion: pay {3} to put companion from outside game into hand. Once per game, sorcery speed.

**ATOM-116.2g-001**
- **Rule:** 116.2g — Companion can be put into hand for {3}, sorcery speed, once per game
- **Mechanism:** Companion special action
- **Minimal Board:** P0 has a chosen companion outside the game. Main phase, stack empty.
- **Action:** Pay {3}, put companion into hand
- **Expected Result:** Companion moves to P0's hand. This action can't be repeated this game.
- **Phase:** Phase 9 (companion implementation — D20)
- **Ticket:** D20

**116.2h** — DEFERRED. Foretell. Phase 9.

**116.2i** — OUT-OF-SCOPE. Planechase planar die roll.

**116.2j** — OUT-OF-SCOPE. Conspiracy Draft face-up flip.

**116.2k** — DEFERRED. Plot. Phase 9.

**116.2m** — DEFERRED. Room enchantments (unlock second room). Standard-legal, common mechanic in recent sets.

**ATOM-116.2m-001**
- **Rule:** 116.2m — Unlocking the second room of a Room enchantment is a special action
- **Mechanism:** Room unlock special action with cost payment
- **Minimal Board:** A Room enchantment with its first room active. P0 controls it and has mana to pay the unlock cost.
- **Action:** Pay the unlock cost as a special action
- **Expected Result:** Second room becomes active. This doesn't use the stack.
- **Phase:** Phase 8+ (Room enchantment implementation)
- **Ticket:** NEW — Room enchantment unlock special action

**116.3** — TESTABLE. Player who takes a special action receives priority afterward.

**ATOM-116.3-001**
- **Rule:** 116.3 — After taking a special action, that player receives priority
- **Mechanism:** Priority return after `play_land()` or other special action
- **Minimal Board:** P0's main phase. P0 plays a land.
- **Action:** After the land play, check who has priority
- **Expected Result:** P0 still has priority (received it back after the special action)
- **Phase:** Already implemented (land play returns priority to active player)
- **Ticket:** ALREADY-IMPLEMENTED

---

## Rules 117–118: Timing and Priority, Costs

### 117. Timing and Priority

**117.1** — PURE-DEF. Priority system determines who can act. Player with priority may cast spells, activate abilities, take special actions.

**117.1a** — TESTABLE. Instants can be cast any time the player has priority. Non-instants only during main phase with empty stack.

**ATOM-117.1a-001**
- **Rule:** 117.1a — Noninstant spell can only be cast during controller's main phase with empty stack
- **Mechanism:** `check_cast_legality()` timing check
- **Minimal Board:** P0's main phase, stack empty. P0 has a sorcery in hand.
- **Action:** Cast the sorcery
- **Expected Result:** Casting succeeds (main phase, stack empty)
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-117.1a-002**
- **Rule:** 117.1a — Noninstant spell can't be cast when stack is non-empty
- **Mechanism:** `check_cast_legality()` timing check
- **Minimal Board:** P0's main phase. A spell is on the stack. P0 has a creature in hand.
- **Action:** Attempt to cast the creature
- **Expected Result:** Casting fails — stack is not empty
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-117.1a-003**
- **Rule:** 117.1a — Instant can be cast any time the player has priority
- **Mechanism:** `check_cast_legality()` allows instants regardless of phase/stack state
- **Minimal Board:** Opponent's combat phase. Stack has a spell on it. P0 has Lightning Bolt in hand.
- **Action:** Cast Lightning Bolt
- **Expected Result:** Casting succeeds (instant can be cast any time with priority)
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**117.1b** — TESTABLE. Activated abilities can be activated any time the player has priority.

**ATOM-117.1b-001**
- **Rule:** 117.1b — A player may activate an activated ability any time they have priority
- **Mechanism:** Activated ability activation check — no timing restriction beyond having priority
- **Minimal Board:** P0's opponent's turn. P0 has priority. P0 controls a creature with an activated ability.
- **Action:** Activate the ability
- **Expected Result:** Activation succeeds (no sorcery-speed restriction unless the ability specifies one)
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**117.1c** — PURE-DEF. Special actions timing. Cross-ref 116.

**117.1d** — TESTABLE. Mana abilities can be activated whenever the player has priority, or during casting/activation that requires a mana payment, or when a rule/effect asks for mana.

**ATOM-117.1d-001**
- **Rule:** 117.1d — Mana abilities can be activated during spell casting (mana payment step)
- **Mechanism:** Mana ability activation allowed during `pay_costs` step of casting
- **Minimal Board:** P0 is casting a spell that costs {R}. P0 has a Mountain untapped.
- **Action:** During mana payment, activate Mountain's mana ability
- **Expected Result:** {R} is added to pool and immediately spent to pay the cost
- **Phase:** Already implemented (tap-and-cast in `queue_tap_and_cast`)
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-117.1d-002**
- **Rule:** 117.1d — Mana abilities can be activated during resolution when an effect asks for a mana payment
- **Mechanism:** Mana ability activation window during effect resolution
- **Minimal Board:** P0 has a creature spell on the stack. P1 casts Mana Leak ("Counter target spell unless its controller pays {3}"). P0 has 3 untapped Islands but {0} in pool.
- **Action:** Mana Leak resolves. P0 is asked to pay {3}. P0 activates 3 Island mana abilities during the resolution payment window.
- **Expected Result:** P0 pays {3}. The creature spell is NOT countered. Mana abilities were legally activated during another spell's resolution.
- **Phase:** Phase 8 (resolution-time mana payment windows)
- **Ticket:** NEW — Mana ability activation during resolution payment

**117.2** — PURE-DEF. Some abilities/actions are automatic (triggers, static effects, TBAs, SBAs). No priority needed.

**117.2a** — TESTABLE. Triggered abilities can trigger at any time but are only placed on the stack when a player would receive priority.

**ATOM-117.2a-001**
- **Rule:** 117.2a — Triggered abilities are placed on the stack when a player would receive priority, not when they trigger
- **Mechanism:** `pending_triggers` queue drained into stack before priority is given
- **Minimal Board:** During resolution of a spell, a triggered ability triggers
- **Action:** The trigger is queued. After resolution, before priority, it is placed on the stack.
- **Expected Result:** Trigger is on the stack after the spell resolves, before any player gets priority
- **Phase:** Phase 7 (triggered abilities — pending_triggers queue)
- **Ticket:** Phase 7 — Trigger queue draining

**117.2b** — PURE-DEF. Static abilities continuously affect the game. Priority doesn't apply. Cross-ref 604, 611.

**117.2c** — TESTABLE. Turn-based actions happen automatically at start of steps/phases, before priority.

**ATOM-117.2c-001**
- **Rule:** 117.2c — Turn-based actions happen before a player receives priority at start of a step/phase
- **Mechanism:** TBA execution in `run_step()` before calling `pass_priority()`
- **Minimal Board:** Beginning of draw step
- **Action:** Step begins
- **Expected Result:** Draw step TBA (draw a card) happens first. Then triggers are placed. Then active player gets priority.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED
- **Cross-ref:** The *specific* TBAs (what happens in each step) are defined by rule 703 and will be tested in the Chapter 7 session. This atom tests only the *timing*: TBAs happen before priority.

**117.2d** — PURE-DEF. SBAs happen automatically when conditions are met. Cross-ref 704.

**117.2e** — PURE-DEF. During resolution, no player has priority (but it is possible mana abilities may be activated).

**117.3** — PURE-DEF. Rules for determining who has priority. Framework.

**117.3a** — TESTABLE. Active player receives priority at beginning of most steps/phases (after TBAs and triggers). No priority during untap step.

**ATOM-117.3a-001**
- **Rule:** 117.3a — No player receives priority during the untap step
- **Mechanism:** Untap step skips priority entirely
- **Minimal Board:** Beginning of untap step
- **Action:** Run the untap step
- **Expected Result:** Permanents untap (TBA). No priority is given. Step ends immediately.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**117.3b** — TESTABLE. Active player receives priority after a spell or ability (other than mana ability) resolves.

**ATOM-117.3b-001**
- **Rule:** 117.3b — Active player gets priority after a spell resolves
- **Mechanism:** Priority reset to active player after `resolve_top_of_stack()`
- **Minimal Board:** P0 (active) cast a spell. P1 passed. Spell resolves.
- **Action:** Check who has priority after resolution
- **Expected Result:** P0 (active player) has priority
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**117.3c** — TESTABLE. After casting a spell, activating an ability, or taking a special action, that player receives priority.

**ATOM-117.3c-001**
- **Rule:** 117.3c — After casting a spell, the caster receives priority
- **Mechanism:** Priority returned to acting player after cast/activate
- **Minimal Board:** P0 casts Lightning Bolt
- **Action:** Check who has priority after the cast
- **Expected Result:** P0 has priority (can respond to their own spell)
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**117.3d** — TESTABLE. Passing priority: player passes, next player in turn order gets priority. Mana pool announced on pass.

**ATOM-117.3d-001**
- **Rule:** 117.3d — When a player passes, the next player in turn order receives priority
- **Mechanism:** `pass_priority()` advances to next player
- **Minimal Board:** P0 has priority and passes
- **Action:** P0 passes
- **Expected Result:** P1 receives priority
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**117.4** — TESTABLE. If all players pass in succession: top of stack resolves (or phase/step ends if stack is empty).

**ATOM-117.4-001**
- **Rule:** 117.4 — All players pass in succession with stack non-empty → top of stack resolves
- **Mechanism:** `pass_priority()` detects all-pass → calls `resolve_top_of_stack()`
- **Minimal Board:** Lightning Bolt on stack. P0 passes. P1 passes.
- **Action:** Both players pass
- **Expected Result:** Lightning Bolt resolves
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-117.4-002**
- **Rule:** 117.4 — All players pass with empty stack → phase/step ends
- **Mechanism:** `pass_priority()` detects all-pass + empty stack → step advances
- **Minimal Board:** P0's main phase. Stack is empty. P0 passes. P1 passes.
- **Action:** Both players pass
- **Expected Result:** The current phase/step ends. Game advances to next phase/step.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**117.5** — TESTABLE. Before a player gets priority: perform all SBAs (repeat until none), then put triggered abilities on stack (repeat SBA+trigger cycle until stable).

**ATOM-117.5-001**
- **Rule:** 117.5 — Multiple SBAs are processed in a single SBA cycle before priority is given
- **Mechanism:** `perform_sbas()` loop repeats until no SBAs are performed in a pass
- **Minimal Board:** A token creature that has taken lethal damage (e.g., a 1/1 token dealt 2 damage).
- **Action:** Run the SBA cycle before granting priority
- **Expected Result:** **SBA round 1:** Token has lethal damage → destroyed, moved to graveyard. **SBA round 2:** Token is in graveyard (not battlefield) → ceases to exist (704.5d). Both SBAs happen in the same SBA cycle. After the cycle, the token is NOT in the graveyard (it ceased to exist entirely). No priority was granted between the two SBA rounds.
- **Phase:** Phase 5-Pre (T13 — token cease-to-exist SBA). SBA loop structure already exists.
- **Ticket:** T13
- **Note:** This test deliberately avoids triggers to isolate pure SBA cascading. The original version used a die trigger, which would put an ability on the stack and grant priority, breaking the "no priority between SBAs" invariant. Token lethal → graveyard → cease-to-exist is a clean 2-SBA cascade with zero triggers.

**117.6** — OUT-OF-SCOPE. Shared team turns priority. Phase 9.

**117.7** — TESTABLE. Casting/activating in response to something already on the stack → the new item resolves first.

**ATOM-117.7-001**
- **Rule:** 117.7 — A spell cast in response to another resolves first (LIFO stack)
- **Mechanism:** Stack is LIFO — last added resolves first
- **Minimal Board:** P0 casts Grizzly Bears. P1 responds with Counterspell targeting Grizzly Bears. Both pass.
- **Action:** Counterspell resolves first
- **Expected Result:** Counterspell resolves (countering Grizzly Bears). Grizzly Bears is countered and goes to graveyard. It never resolves.
- **Phase:** Already implemented (stack is a Vec, top resolves first)
- **Ticket:** ALREADY-IMPLEMENTED

---

### 118. Costs

**118.1** — PURE-DEF. A cost is an action/payment necessary for another action. Framework.

**118.2** — PURE-DEF. Mana costs give a chance to activate mana abilities. Cross-ref 601.2f–h.

**118.3** — TESTABLE. A player can't pay a cost without the necessary resources.

**ATOM-118.3-001**
- **Rule:** 118.3 — Can't pay a cost without sufficient resources (e.g., can't pay 2 life at 1 life)
- **Mechanism:** Cost validation before payment
- **Minimal Board:** Player at 1 life. An ability costs 2 life.
- **Action:** Attempt to activate the ability
- **Expected Result:** Activation fails — insufficient life to pay cost
- **Phase:** Already implemented (Cost::PayLife check in `engine/costs.rs`)
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-118.3-002**
- **Rule:** 118.3 — Already tapped permanent can't be tapped to pay a cost
- **Mechanism:** Cost::Tap validation
- **Minimal Board:** A tapped creature. An ability costs {T}.
- **Action:** Attempt to activate
- **Expected Result:** Activation fails — already tapped
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED (cross-ref 107.5)

**118.3a** — TESTABLE. Paying mana = removing from mana pool. Players can always pay 0 mana.

**ATOM-118.3a-001**
- **Rule:** 118.3a — Paying mana removes it from the mana pool
- **Mechanism:** `ManaPool::spend()` removes mana
- **Minimal Board:** Player has {R}{R}{G} in pool. Pays {R}{G}.
- **Action:** Pay the cost
- **Expected Result:** Pool now contains only {R}
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**118.3b** — TESTABLE. Paying life = subtracting from life total. Players can always pay 0 life.

**ATOM-118.3b-001**
- **Rule:** 118.3b — Paying life subtracts from life total
- **Mechanism:** `pay_life()` reduces life total
- **Minimal Board:** Player at 20 life. Pays 3 life.
- **Action:** Pay the cost
- **Expected Result:** Player at 17 life
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**118.3c** — PURE-DEF. Activating mana abilities is not mandatory even when paying a cost is.

**118.4** — PURE-DEF. Costs with {X}. Cross-ref 107.3.

**118.5** — TESTABLE. {0} cost still requires player acknowledgment; it's not automatically paid.

**ATOM-118.5-001**
- **Rule:** 118.5 — A cost of {0} is not automatically paid; player must choose to pay it
- **Mechanism:** Cost payment pipeline treats {0} as a real cost requiring player action
- **Minimal Board:** An artifact with mana cost {0} in hand
- **Action:** Player must still go through the casting process (not auto-cast)
- **Expected Result:** The spell is placed on the stack through the normal casting pipeline. It is NOT automatically cast.
- **Phase:** Already implemented (casting pipeline treats {0} same as any cost)
- **Ticket:** ALREADY-IMPLEMENTED

**118.5a** — TESTABLE. {0} mana cost spell must be cast normally; it won't cast itself.

**ATOM-118.5a-001**
- **Rule:** 118.5a — A spell with mana cost {0} must still be cast through normal casting steps
- **Mechanism:** Same as 118.5
- **Minimal Board:** Ornithopter ({0} creature) in hand
- **Action:** Player casts it — goes through 601.2 steps
- **Expected Result:** Ornithopter goes on stack, resolves normally. It doesn't bypass the stack.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**118.6** — TESTABLE. Objects with no mana cost have unpayable mana cost. Can attempt to cast but can't pay.

**ATOM-118.6-001**
- **Rule:** 118.6 — Attempting to cast a spell with unpayable cost is legal, but paying is illegal
- **Mechanism:** No-mana-cost objects have `ManaCost::None` which is unpayable
- **Minimal Board:** A card with no mana cost (e.g., a land-type card that somehow needs to be cast, or a suspend-only card)
- **Action:** Attempt to cast
- **Expected Result:** The cast attempt itself is legal (goes through the motions). But when cost payment is reached, it fails (unpayable) and the cast is reversed — unless an alternative cost is applied.
- **Phase:** Phase 5-Pre (T18 — casting pipeline handles unpayable costs)
- **Ticket:** T18

**118.6a** — TESTABLE. Unpayable cost + increase/additional cost = still unpayable. But alternative cost can replace it.

**ATOM-118.6a-001**
- **Rule:** 118.6a — Alternative cost can replace an unpayable cost
- **Mechanism:** Alternative cost override in casting pipeline
- **Minimal Board:** A card with no mana cost. An effect says "you may cast this without paying its mana cost."
- **Action:** Cast using the alternative cost
- **Expected Result:** Cast succeeds — the unpayable mana cost is replaced by the alternative cost ({0})
- **Phase:** Phase 8 (free-cast effects)
- **Ticket:** Phase 8

**118.7** — TESTABLE. Cost reduction effects can reduce costs. If reduced to nothing, it's {0}. Paying the reduced cost counts as paying the original.

**ATOM-118.7-001**
- **Rule:** 118.7 — Cost reduced to nothing is considered {0}
- **Mechanism:** Cost reduction pipeline in casting
- **Minimal Board:** A spell with mana cost {1}{R}. An effect reduces costs by {2}.
- **Action:** Calculate the final cost
- **Expected Result:** Cost is {R} (generic reduced from {1} to {0}, colored component untouched per 118.7a)
- **Phase:** Phase 5 Layers (cost modification pipeline — T18 601.2e)
- **Ticket:** T18

**ATOM-118.7-002**
- **Rule:** 118.7 — Reducing cost to {0} allows free casting
- **Mechanism:** Cost reduction pipeline reduces total cost to zero
- **Minimal Board:** A spell with mana cost {2}. An effect reduces costs by {3}.
- **Action:** Calculate the final cost and cast
- **Expected Result:** Cost is {0} (reduced from {2}; excess {1} is lost). Player can cast for free. This is distinct from 118.5 (objects that *naturally* cost {0}) — this tests that the reduction pipeline correctly produces a zero cost.
- **Phase:** Phase 5 Layers (cost modification pipeline)
- **Ticket:** T18

**118.7a** — TESTABLE. Generic cost reductions only affect generic mana component.

**ATOM-118.7a-001**
- **Rule:** 118.7a — Effects reducing by generic mana only affect the generic component
- **Mechanism:** Cost reduction targets generic component only
- **Minimal Board:** Spell costs {2}{R}{R}. Effect reduces by {3}.
- **Action:** Calculate final cost
- **Expected Result:** Cost is {R}{R} (generic {2} reduced to {0}; the extra {1} reduction doesn't affect colored)
- **Phase:** Phase 5 Layers (cost modification)
- **Ticket:** T18

**118.7b** — TESTABLE. If a cost is reduced by colored/colorless mana the cost doesn't require, reduce generic instead.

**ATOM-118.7b-001**
- **Rule:** 118.7b — Colored reduction on a cost without that color reduces generic instead
- **Mechanism:** Cost reduction overflow from colored to generic
- **Minimal Board:** Spell costs {3}{R}. Effect reduces by {U}.
- **Action:** Calculate final cost
- **Expected Result:** Cost is {2}{R} (no {U} component, so {U} reduction → reduce generic by 1)
- **Phase:** Phase 5 Layers (cost modification)
- **Ticket:** T18

**118.7c** — TESTABLE. Colored reduction exceeding colored component overflows to generic.

**ATOM-118.7c-001**
- **Rule:** 118.7c — Excess colored reduction overflows to generic
- **Mechanism:** Cost reduction overflow
- **Minimal Board:** Spell costs {2}{R}. Effect reduces by {R}{R}.
- **Action:** Calculate final cost
- **Expected Result:** Cost is {1} (first {R} reduces the {R} component; second {R} overflows to reduce generic by 1)
- **Phase:** Phase 5 Layers (cost modification)
- **Ticket:** T18

**118.7d** — TESTABLE. Colorless reduction exceeding colorless component overflows to generic.

**ATOM-118.7d-001**
- **Rule:** 118.7d — Excess colorless reduction overflows to generic
- **Mechanism:** Cost reduction overflow for colorless
- **Minimal Board:** Spell costs {2}{C}. Effect reduces by {C}{C}.
- **Action:** Calculate final cost
- **Expected Result:** Cost is {1} (first {C} removes {C}; second overflows to reduce generic by 1)
- **Phase:** Phase 5 Layers (cost modification)
- **Ticket:** T18

**118.7e** — TESTABLE. Hybrid mana reduction: when a cost reduction is itself a hybrid mana symbol (e.g., an effect says "Spells cost {2/U} less to cast"), the player paying the cost chooses which half of the *reduction* to apply. This is analogous to how paying a hybrid cost works — the reduction is a hybrid symbol and you choose which half to reduce by.

**ATOM-118.7e-001**
- **Rule:** 118.7e — Hybrid mana reduction symbol: player chooses which half of the reduction to apply
- **Mechanism:** The reduction itself is a hybrid symbol (e.g., {R/G}). Player chooses to reduce by {R} or by {G}.
- **Minimal Board:** Spell costs {2}{R}{G}. An effect says "Spells cost {R/G} less to cast."
- **Action:** Player chooses the {R} half of the reduction
- **Expected Result:** Cost is {2}{G} (the {R} component was reduced). If player had chosen the {G} half, cost would be {2}{R}.
- **Phase:** Phase 8 (hybrid cost reduction)
- **Ticket:** NEW — Hybrid mana reduction symbol

**ATOM-118.7e-002**
- **Rule:** 118.7e — Two-brid (e.g., {2/W}) as a reduction symbol: player chooses to reduce by {2} or by {W}
- **Mechanism:** Two-brid reduction symbol choice. Analogous to 118.7f (Phyrexian reduction) — the reduction is a special mana symbol and you pick which half.
- **Minimal Board:** Spell costs {3}{W}. An effect says "Spells cost {2/W} less to cast."
- **Action:** Player chooses the {W} half of the reduction
- **Expected Result:** Cost is {3} (the {W} was reduced). If player had chosen the {2} half, cost would be {1}{W}.
- **Phase:** Phase 8 (two-brid cost reduction)
- **Ticket:** NEW — Two-brid mana reduction symbol

**118.7f** — TESTABLE. Phyrexian mana reduction → reduces by one of that color.

**ATOM-118.7f-001**
- **Rule:** 118.7f — Phyrexian reduction reduces by one mana of that symbol's color
- **Mechanism:** Phyrexian cost reduction
- **Minimal Board:** Spell costs {1}{R}{R}. Effect reduces by {R/P}.
- **Action:** Calculate final cost
- **Expected Result:** Cost is {1}{R} (one {R} removed)
- **Phase:** Phase 8
- **Ticket:** NEW — Phyrexian cost reduction

**118.7g** — TESTABLE. Snow mana reduction → reduces generic mana.

**ATOM-118.7g-001**
- **Rule:** 118.7g — Snow mana reduction reduces generic mana
- **Mechanism:** Snow cost reduction mapped to generic
- **Minimal Board:** Spell costs {3}{R}. Effect reduces by {S}.
- **Action:** Calculate final cost
- **Expected Result:** Cost is {2}{R} (generic reduced by 1)
- **Phase:** Phase 8
- **Ticket:** NEW — Snow cost reduction

**118.8** — PURE-DEF. Additional costs framework. Applied during casting/activation.

**118.8a** — PURE-DEF. Multiple additional costs can apply. Announced per 601.2b.

**118.8b** — PURE-DEF. Some additional costs are optional.

**118.8c** — PURE-DEF. "Cast if able" with mandatory additional cost in hidden zone — not required.

**Note (118.8c engine approach):** When processing "cast if able" effects, the engine must check if all mandatory additional costs *can* be paid before forcing the cast. The simplest approach: before forcing a cast, run the full `can_cast()` check (which includes cost payability). If `can_cast()` returns false, don't force the cast. This avoids special-casing hidden zones — the existing affordability check handles it.

**118.8d** — TESTABLE. Additional costs don't change the spell's mana cost.

**ATOM-118.8d-001**
- **Rule:** 118.8d — Additional costs don't change a spell's mana cost (mana value stays the same)
- **Mechanism:** Mana value calculation ignores additional costs
- **Minimal Board:** A spell with mana cost {2}{R} and additional cost "sacrifice a creature." Player pays {2}{R} + sacrifices.
- **Action:** Query the spell's mana value on the stack
- **Expected Result:** Mana value is 3 (only the mana cost matters, not the additional cost)
- **Phase:** Phase 5 Layers (mana value calculation)
- **Ticket:** L01

**118.9** — PURE-DEF. Alternative costs framework. "You may [action] rather than pay [mana cost]." Only one alternative cost per spell.

**118.9a** — TESTABLE. Only one alternative cost can be applied per spell.

**ATOM-118.9a-001**
- **Rule:** 118.9a — Only one alternative cost can be applied to a spell
- **Mechanism:** Alternative cost selection is singular in casting pipeline
- **Minimal Board:** A spell with two available alternative costs
- **Action:** Attempt to apply both
- **Expected Result:** Only one is applied. Player must choose one.
- **Phase:** Phase 5-Pre (T18 — 601.2b cost selection)
- **Ticket:** T18

**118.9b** — PURE-DEF. Alternative costs are generally optional.

**118.9c** — TESTABLE. Alternative cost doesn't change mana cost (same as additional cost rule).

**ATOM-118.9c-001**
- **Rule:** 118.9c — Alternative cost doesn't change the spell's mana cost for mana value purposes
- **Mechanism:** Same as 118.8d — mana value unaffected by alt cost
- **Minimal Board:** A spell with mana cost {4}{U}{U} cast via "without paying its mana cost"
- **Action:** Query the spell's mana value on the stack
- **Expected Result:** Mana value is 6 (original mana cost, not the alternative {0})
- **Phase:** Phase 5 Layers
- **Ticket:** L01

**118.9d** — TESTABLE. Additional costs, cost increases, and cost reductions apply to alternative costs too.

**ATOM-118.9d-001**
- **Rule:** 118.9d — Cost modifications apply to alternative costs
- **Mechanism:** Cost modification pipeline applies to whatever cost is being paid
- **Minimal Board:** A spell with mana cost {4}{R}{R} has alternative cost {R}{R}. An effect increases costs by {1}.
- **Action:** Calculate final cost using alternative cost
- **Expected Result:** Final cost is {1}{R}{R} (alternative cost {R}{R} + increase {1})
- **Phase:** Phase 5 Layers (cost modification pipeline)
- **Ticket:** T18

**118.10** — TESTABLE. Each cost payment applies to only one spell/ability/effect.

**ATOM-118.10-001**
- **Rule:** 118.10 — Can't sacrifice one creature to pay two different costs
- **Mechanism:** Cost payment is one-to-one; paid resources are consumed
- **Minimal Board:** Two permanents with "Sacrifice a creature: [effect]." Player controls only one other creature.
- **Action:** Attempt to sacrifice the same creature for both
- **Expected Result:** Only one ability can be paid. The creature is gone after the first sacrifice.
- **Phase:** Already implemented (sacrifice removes the permanent)
- **Ticket:** ALREADY-IMPLEMENTED

**118.11** — PURE-DEF. Actions performed when paying a cost may be modified by effects; cost is still considered paid.

**118.12** — TESTABLE. "[Do something]. If [player] [does/doesn't], [effect]." The [do something] is a cost paid on resolution. The engine must implement this as a two-part resolution: attempt the action, then check whether it succeeded/was chosen.

**Atomicity note:** The tests below are each testing a distinct failure/success mode of the "if you do" mechanism. ATOM-118.12-001 tests the "cost object no longer exists" failure path. ATOM-118.12-002 tests the "action was taken but outcome was altered by replacement effect" success path. These are independent code paths in the resolution pipeline.

**ATOM-118.12-001**
- **Rule:** 118.12 — "If you do" fails when the cost object no longer exists
- **Mechanism:** Resolution-time cost payment failure handling
- **Minimal Board:** Standstill (enchantment) on the battlefield. A player casts a spell, triggering Standstill's "When a player casts a spell, sacrifice Standstill. If you do, each of that player's opponents draws three cards." Before the trigger resolves, another effect exiles Standstill.
- **Action:** The triggered ability resolves. Attempt to sacrifice Standstill.
- **Expected Result:** Standstill is no longer on the battlefield (exiled). Sacrifice fails. "If you do" clause fails. No cards are drawn.
- **Phase:** Phase 7 (triggered abilities + resolution-time cost failure)
- **Ticket:** Phase 7 — "If you do" resolution-time cost failure

**ATOM-118.12-002**
- **Rule:** 118.12 — "If you do" succeeds when the action was chosen/attempted, even if the outcome was altered
- **Mechanism:** Resolution-time "if you do" checks whether the *action was taken*, not the *exact outcome*
- **Minimal Board:** Dermoplasm (face-down) turns face up via morph trigger: "you may put a creature card with morph from your hand onto the battlefield face up. If you do, return Dermoplasm to its owner's hand." Opponent controls Gather Specimens ("If a creature would enter the battlefield under an opponent's control, it enters under your control instead"). P0 chooses to put a creature from hand.
- **Action:** The creature enters the battlefield, but under the opponent's control (replacement effect from Gather Specimens). Then check "if you do."
- **Expected Result:** "If you do" succeeds — P0 *chose* to put the creature onto the battlefield (the action was taken). Dermoplasm returns to P0's hand. The fact that the creature ended up under the opponent's control doesn't negate the cost payment.
- **Phase:** Phase 8 (replacement effects + "if you do" interaction)
- **Ticket:** Phase 8 — "If you do" with replacement-altered outcomes
- **Cross-ref:** 608 (resolving spells and abilities) for full resolution pipeline.

**118.12a** — PURE-DEF. "Unless" phrasing. Equivalent to optional cost.

**118.12b** — PURE-DEF (subsumed). Search + "If [player] does" checks whether player *chose to search*, not whether they *found* anything. Searching and failing to find (legal in hidden zones) still counts as "doing it." This is a specific application of the general "if you do" mechanism tested in ATOM-118.12-001 and -002 above — the hidden-zone search nuance is covered by the principle that "if you do" checks the *action*, not the *outcome*. Full search mechanics deferred to 701 (keyword actions) session.

**Cross-ref:** The search-specific "fail to find" rule belongs with 701.19 (search) in the keyword actions session. The "if you do" mechanism is already tested here.

**118.13** — PURE-DEF. Costs with hybrid/Phyrexian symbols have multiple payment options. Framework.

**118.13a** — PURE-DEF. Choice of how to pay hybrid/Phyrexian made during 601.2b.

**118.13b** — PURE-DEF. Cost during resolution with multi-pay symbol — choice made immediately before payment.

**118.13c** — PURE-DEF. Special action cost with multi-pay symbol — choice made immediately before payment.

**118.14** — TESTABLE. "Mana of any type can be spent" = mana treated as colorless or any color for that cost.

**ATOM-118.14-001**
- **Rule:** 118.14 — "Mana of any type can be spent" allows any mana to pay any colored/colorless component
- **Mechanism:** Cost payment override that treats all mana as wild
- **Minimal Board:** Player has {G}{G}{G}. Spell costs {W}{U}{B}. An effect says "mana of any type can be spent to cast this spell."
- **Action:** Pay {G}{G}{G} for {W}{U}{B}
- **Expected Result:** Payment succeeds — each {G} is treated as {W}, {U}, and {B} respectively
- **Phase:** Phase 8 (when "any type" mana effects exist)
- **Ticket:** NEW — "Mana of any type" cost override

**ATOM-118.14-002**
- **Rule:** 118.14 — "Mana of any type" does NOT override spending restrictions
- **Mechanism:** Mana restriction tracking is separate from mana type
- **Minimal Board:** Gwenna, Eyes of Gaea produces {G}{G} that can only be spent on creature spells. An effect says "mana of any type can be spent to cast this spell" on a noncreature spell.
- **Action:** Attempt to use Gwenna's restricted {G}{G} to pay for the noncreature spell
- **Expected Result:** Payment fails. "Mana of any type" changes what *color* the mana is treated as, not the *restriction* on what it can be spent on. Gwenna's mana is still creature-only.
- **Phase:** Phase 8 (mana restrictions + "any type" interaction)
- **Ticket:** Phase 8 — Mana restriction persistence with "any type"

---

## Rules 119–121: Life, Damage, Drawing a Card

### 119. Life

**119.1** — TESTABLE. Starting life total is 20 (duplicates 103.4).

**ATOM-119.1-001**
- **Rule:** 119.1 — Each player begins with the starting life total defined by the game format
- **Mechanism:** `GameConfig.starting_life` → `PlayerState.life`
- **Minimal Board:** New game with `GameConfig { starting_life: 20, .. }`
- **Action:** Check life totals after setup
- **Expected Result:** Both players at `game_config.starting_life` (20 for Standard). The test should verify against `GameConfig`, not a hardcoded 20, to support Commander (40) and other formats.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED (cross-ref ATOM-103.4-001)

**119.1a–e** — DEFERRED. Variant starting life totals (2HG, Vanguard, Commander, Brawl, Archenemy). Commander (119.1c) deferred to Phase 9.

**119.2** — TESTABLE. Damage dealt to a player causes life loss. Cross-ref 120.3.

**ATOM-119.2-001**
- **Rule:** 119.2 — Damage dealt to a player causes that player to lose that much life
- **Mechanism:** `perform_action(DealDamage)` → `lose_life()`
- **Minimal Board:** Player at 20 life. Lightning Bolt resolves dealing 3 damage.
- **Action:** Resolve the damage
- **Expected Result:** Player at 17 life
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**119.3** — TESTABLE. Gain life / lose life adjusts life total accordingly.

**ATOM-119.3-001**
- **Rule:** 119.3 — Effect causing life gain adjusts life total upward
- **Mechanism:** `gain_life()` adds to `life`
- **Minimal Board:** Player at 15 life. Effect causes them to gain 5 life.
- **Action:** Resolve
- **Expected Result:** Player at 20 life
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**119.4** — TESTABLE. Paying life > 0 requires life total ≥ payment amount.

**ATOM-119.4-001**
- **Rule:** 119.4 — Can't pay life if life total < payment amount
- **Mechanism:** Life payment validation in cost check
- **Minimal Board:** Player at 2 life. Cost requires paying 3 life.
- **Action:** Attempt to pay
- **Expected Result:** Payment rejected — insufficient life
- **Phase:** Already implemented (Cost::PayLife check)
- **Ticket:** ALREADY-IMPLEMENTED

**119.4a** — OUT-OF-SCOPE. Two-Headed Giant team life payment.

**119.4b** — TESTABLE. Players can always pay 0 life, even if an effect says "can't pay life."

**ATOM-119.4b-001**
- **Rule:** 119.4b — Paying 0 life is always legal
- **Mechanism:** Zero-life payment bypass
- **Minimal Board:** Player with "can't pay life" restriction. A cost requires paying 0 life.
- **Action:** Pay 0 life
- **Expected Result:** Payment succeeds
- **Phase:** Phase 8 (when "can't pay life" effects exist)
- **Ticket:** NEW — Zero-life payment always legal

**119.5** — TESTABLE. Effect setting life to a specific number → player gains or loses the difference.

**ATOM-119.5-001**
- **Rule:** 119.5 — Setting life total to N causes gain/loss of the difference
- **Mechanism:** `set_life()` computes delta, calls `gain_life()` or `lose_life()`
- **Minimal Board:** Player at 15 life. Effect sets life to 10.
- **Action:** Resolve the effect
- **Expected Result:** Player loses 5 life (not: "life becomes 10" directly). Triggers that care about life loss fire.
- **Phase:** Phase 8 (SetLife primitive)
- **Ticket:** NEW — SetLife primitive

**ATOM-119.5-002**
- **Rule:** 119.5 — Setting life higher causes life gain
- **Mechanism:** Same as above
- **Minimal Board:** Player at 10 life. Effect sets life to 20.
- **Action:** Resolve
- **Expected Result:** Player gains 10 life. Life gain triggers fire.
- **Phase:** Phase 8
- **Ticket:** NEW — SetLife primitive (same)

**119.6** — TESTABLE. Player at 0 or less life loses (SBA). Cross-ref 104.3b.

**ATOM-119.6-001**
- **Rule:** 119.6 — Player at 0 or less life loses as SBA
- **Mechanism:** SBA check in `sba.rs`
- **Minimal Board:** Player at 0 life
- **Action:** Check SBAs
- **Expected Result:** Player loses the game
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED (cross-ref ATOM-104.3b-001)

**119.7** — TESTABLE. "Can't gain life" prevents life-total-increasing exchanges, redistributions, and replacement effects that would produce life gain.

**ATOM-119.7-001**
- **Rule:** 119.7 — "Can't gain life" prevents exchanges that would increase life total
- **Mechanism:** Life gain prevention check in exchange resolution
- **Minimal Board:** Player A at 5 life with "can't gain life." Player B at 20. An effect exchanges their life totals.
- **Action:** Attempt the exchange
- **Expected Result:** Exchange doesn't happen (A would go from 5 to 20, which is gaining life, which is prevented)
- **Phase:** Phase 8 (life exchange + "can't gain life" restriction)
- **Ticket:** NEW — "Can't gain life" prevention

**ATOM-119.7-002**
- **Rule:** 119.7 — "Can't gain life" blocks redistribute-life effects
- **Mechanism:** Redistribute life checks gain/loss for each player
- **Minimal Board:** Players A (5 life, "can't gain life"), B (15 life), C (10 life). Effect redistributes total life (30) among all players.
- **Action:** Attempt to redistribute giving A 15, B 10, C 5
- **Expected Result:** A's redistribution is blocked (would gain life). The redistribution fails entirely (per 119.7, the restriction prevents the life-total-increasing portion, which invalidates the whole redistribution).
- **Phase:** Phase 8 (life redistribution + "can't gain life")
- **Ticket:** Phase 8

**ATOM-119.7-003**
- **Rule:** 119.7 — "Can't gain life" blocks an opponent's life gain when it's used as an alternate cost
- **Mechanism:** Alternate cost requiring an opponent to gain life is unpayable if that opponent has "can't gain life"
- **Minimal Board:** Player B controls a Forest. Player A has "can't gain life" restriction (e.g., from Erebos, God of the Dead). Player B wants to cast Invigorate ("If you control a Forest, rather than pay this spell's mana cost, you may have an opponent gain 3 life. Target creature gets +4/+4 until end of turn.") choosing Player A as the opponent for the alternate cost.
- **Action:** Attempt to cast Invigorate using the alternate cost targeting Player A for the life gain
- **Expected Result:** The alternate cost is unpayable — Player A can't gain life, so "have an opponent gain 3 life" targeting Player A fails. Player B must either pay the mana cost normally or choose a different opponent (if one exists without the restriction).
- **Phase:** Phase 8
- **Ticket:** Phase 8

**ATOM-119.7-004**
- **Rule:** 119.7 — Replacement effects that would produce life gain don't apply under "can't gain life"
- **Mechanism:** Replacement effect application blocked when result would be prevented
- **Minimal Board:** Player with "can't gain life" and "If you would gain life, instead draw that many cards." An effect would cause 3 life gain.
- **Action:** Resolve
- **Expected Result:** The life gain is prevented by "can't gain life." The replacement effect doesn't apply (there's no event to replace). No cards drawn.
- **Phase:** Phase 6 (replacement effects) + Phase 8 ("can't gain life")
- **Ticket:** Phase 6 + Phase 8

**119.8** — TESTABLE. "Can't lose life" prevents exchanges that would decrease life total.

**ATOM-119.8-001**
- **Rule:** 119.8 — "Can't lose life" prevents exchanges that would decrease life total
- **Mechanism:** Life loss prevention check in exchange resolution
- **Minimal Board:** Player A at 20 life with "can't lose life." Player B at 5. An effect exchanges their life totals.
- **Action:** Attempt the exchange
- **Expected Result:** Exchange doesn't happen (A would go from 20 to 5, which is losing life, which is prevented)
- **Phase:** Phase 8
- **Ticket:** NEW — "Can't lose life" prevention

**119.9** — TESTABLE. "Whenever [player] gains life" triggers treat each source of life gain as a separate event. 0 life gain doesn't trigger.

**ATOM-119.9-001**
- **Rule:** 119.9 — Each source of simultaneous life gain is a separate event, triggering separately
- **Mechanism:** Multiple life gain events emitted for simultaneous sources
- **Minimal Board:** Player controls two creatures with lifelink (2/2 and 3/3). Both deal combat damage simultaneously. Player has "Whenever you gain life, put a +1/+1 counter on target creature" (e.g., Ajani's Pridemate-like trigger).
- **Action:** Both creatures deal combat damage in the same combat damage step
- **Expected Result:** Two separate life gain events fire (one for 2, one for 3). The "whenever you gain life" trigger fires TWICE (once per source), not once for the total (5). Player gains 2 life and 3 life separately.
- **Phase:** Phase 7 (triggered abilities — life gain triggers)
- **Ticket:** Phase 7

**ATOM-119.9-002**
- **Rule:** 119.9 — Gaining 0 life does not trigger "whenever you gain life" abilities
- **Mechanism:** Life gain event emission gated on amount > 0
- **Minimal Board:** Player with "Whenever you gain life, draw a card." An effect causes 0 life gain.
- **Action:** Resolve the effect
- **Expected Result:** No trigger fires. No card drawn.
- **Phase:** Phase 7 (triggered abilities — life gain triggers)
- **Ticket:** Phase 7

**119.10** — TESTABLE. "If [player] would gain life" replacement effects don't apply to 0-life-gain events.

**ATOM-119.10-001**
- **Rule:** 119.10 — Replacement effects for life gain don't apply when gaining 0 life
- **Mechanism:** Replacement effect application gated on amount > 0
- **Minimal Board:** Player with "If you would gain life, gain that much life plus 1 instead." An effect causes 0 life gain.
- **Action:** Resolve
- **Expected Result:** No replacement applies. Player gains 0 life (the replacement effect does NOT fire, so "plus 1" is NOT applied). If the rule were violated and the replacement applied, player would gain 1 life.
- **Phase:** Phase 6 (replacement effects)
- **Ticket:** Phase 6
- **Note:** Using "plus 1" instead of "double" because doubling 0 still yields 0, which doesn't demonstrate the distinction. "Plus 1" makes the test meaningful: violation would be detectable (1 vs. 0).

---

### 120. Damage

**120.1** — PURE-DEF. Objects can deal damage to battles, creatures, planeswalkers, and players. Source of damage = the dealing object.

**120.1a** — BOUNDARY-DEF. Damage can't be dealt to non-battle, non-creature, non-planeswalker objects.

**ATOM-120.1a-001**
- **Rule:** 120.1a — Damage can't be dealt to a noncreature, nonplaneswalker, nonbattle permanent
- **Mechanism:** Damage target validation
- **Minimal Board:** A noncreature artifact on the battlefield. An effect tries to deal 3 damage to it.
- **Action:** Attempt to deal damage
- **Expected Result:** Damage is not dealt (illegal damage recipient)
- **Phase:** Already implemented (DealDamage validates target type)
- **Ticket:** ALREADY-IMPLEMENTED

**120.2** — PURE-DEF. Any object can deal damage. Framework.

**120.2a** — TESTABLE. Combat damage = attacking/blocking creatures deal damage equal to their power.

**ATOM-120.2a-001**
- **Rule:** 120.2a — Combat damage equals creature's power
- **Mechanism:** `assign_combat_damage()` uses effective power
- **Minimal Board:** 3/3 creature attacks unblocked
- **Action:** Combat damage step
- **Expected Result:** 3 damage dealt to defending player
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**120.2b** — PURE-DEF. Noncombat damage dealt by spells/abilities. The spell/ability specifies the source.

**120.3** — PURE-DEF. Damage results depend on recipient type and source characteristics. Framework for sub-rules.

**120.3a** — TESTABLE. Damage from source without infect → player loses that much life.

**ATOM-120.3a-001**
- **Rule:** 120.3a — Damage to player from non-infect source → life loss
- **Mechanism:** `perform_action(DealDamage)` → `lose_life(amount)`
- **Minimal Board:** Player at 20 life. 3 damage dealt by a source without infect.
- **Action:** Deal the damage
- **Expected Result:** Player at 17 life
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**120.3b** — TESTABLE. Damage from source with infect → poison counters instead of life loss.

**ATOM-120.3b-001**
- **Rule:** 120.3b — Damage to player from infect source → poison counters
- **Mechanism:** Infect check in `perform_action(DealDamage)` → `give_poison_counters()`
- **Minimal Board:** Player at 20 life. 3 damage dealt by a source with infect.
- **Action:** Deal the damage
- **Expected Result:** Player stays at 20 life. Player receives 3 poison counters.
- **Phase:** Phase 8 (infect keyword)
- **Ticket:** Phase 8 — Infect

**120.3c** — TESTABLE. Damage to a planeswalker removes that many loyalty counters.

**ATOM-120.3c-001**
- **Rule:** 120.3c — Damage to planeswalker removes loyalty counters
- **Mechanism:** `perform_action(DealDamage)` for planeswalker targets → remove loyalty counters
- **Minimal Board:** Planeswalker with 5 loyalty. 3 damage dealt to it.
- **Action:** Deal the damage
- **Expected Result:** Planeswalker has 2 loyalty counters
- **Phase:** Phase 5-Pre (T14 — loyalty counters) + Phase 8 (PW damage)
- **Ticket:** T14 + Phase 8

**120.3d** — TESTABLE. Damage from source with wither/infect to creature → -1/-1 counters.

**ATOM-120.3d-001**
- **Rule:** 120.3d — Damage from wither/infect source to creature → -1/-1 counters instead of damage marked
- **Mechanism:** Wither/infect check in damage processing → put -1/-1 counters
- **Minimal Board:** 3/3 creature. Source with wither deals 2 damage.
- **Action:** Deal the damage
- **Expected Result:** Creature gets 2 -1/-1 counters (now 1/1). No damage is marked.
- **Phase:** Phase 8 (wither keyword)
- **Ticket:** Phase 8 — Wither

**120.3e** — TESTABLE. Damage from source without wither/infect to creature → marked damage.

**ATOM-120.3e-001**
- **Rule:** 120.3e — Damage to creature from normal source is marked on the creature
- **Mechanism:** `BattlefieldEntity.damage_marked += amount`
- **Minimal Board:** 3/3 creature. 2 damage dealt by a normal source.
- **Action:** Deal the damage
- **Expected Result:** Creature has 2 damage marked. It's not destroyed yet (toughness 3 > damage 2).
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**120.3f** — TESTABLE. Lifelink: damage dealt by source with lifelink causes controller to gain life.

**ATOM-120.3f-001**
- **Rule:** 120.3f — Lifelink causes controller to gain life equal to damage dealt
- **Mechanism:** `apply_lifelink()` in `engine/keywords.rs`
- **Minimal Board:** 3/3 creature with lifelink attacks unblocked. Controller at 20 life.
- **Action:** Combat damage
- **Expected Result:** 3 damage to defending player. Attacking player gains 3 life (now 23).
- **Phase:** Already implemented (Phase 4)
- **Ticket:** ALREADY-IMPLEMENTED

**120.3g** — TESTABLE. Toxic: combat damage to player → additional poison counters equal to toxic value.

**ATOM-120.3g-001**
- **Rule:** 120.3g — Toxic creature dealing combat damage to player gives poison counters equal to toxic value (in addition to damage)
- **Mechanism:** Toxic check in combat damage processing
- **Minimal Board:** 2/2 creature with toxic 1 attacks unblocked.
- **Action:** Combat damage
- **Expected Result:** 2 damage dealt to player (life loss). Player also receives 1 poison counter.
- **Phase:** Phase 8 (toxic keyword)
- **Ticket:** Phase 8 — Toxic

**120.3h** — TESTABLE. Damage to a battle removes defense counters.

**ATOM-120.3h-001**
- **Rule:** 120.3h — Damage to battle removes that many defense counters
- **Mechanism:** `perform_action(DealDamage)` for battle targets → remove defense counters
- **Minimal Board:** Battle with 5 defense counters. 3 damage dealt.
- **Action:** Deal the damage
- **Expected Result:** Battle has 2 defense counters
- **Phase:** Phase 9 (battles)
- **Ticket:** Phase 9

**120.4** — PURE-DEF. Damage is processed in a 4-part sequence. Framework.

**120.4a** — TESTABLE. Excess damage redirection (e.g., trample-like effects).

**ATOM-120.4a-001**
- **Rule:** 120.4a — Excess damage to a creature is damage beyond lethal; deathtouch makes any amount > 1 excess
- **Mechanism:** Excess damage calculation in combat/damage assignment
- **Minimal Board:** 5/5 trampler blocked by 2/2. Source has no deathtouch.
- **Action:** Assign damage
- **Expected Result:** 2 damage to blocker (lethal), 3 excess assigned to defending player
- **Phase:** Already implemented (trample in `assign_trample_damage`)
- **Ticket:** ALREADY-IMPLEMENTED

**ATOM-120.4a-002**
- **Rule:** 120.4a — Deathtouch makes any damage > 1 excess
- **Mechanism:** `lethal_damage_for()` returns 1 when source has deathtouch
- **Minimal Board:** 5/5 trampler with deathtouch blocked by 4/4.
- **Action:** Assign damage
- **Expected Result:** 1 damage to blocker (lethal with deathtouch), 4 excess to defending player
- **Phase:** Already implemented (Phase 4 — deathtouch + trample interaction)
- **Ticket:** ALREADY-IMPLEMENTED

**120.4b** — PURE-DEF. Damage is dealt as modified by replacement/prevention effects. Triggers fire.

**120.4c** — PURE-DEF. Damage results processed as modified by replacement effects (e.g., life loss replacements).

**120.4d** — PURE-DEF. Damage event occurs. Framework.

**120.5** — TESTABLE. Damage doesn't directly destroy creatures. SBAs do that.

**ATOM-120.5-001**
- **Rule:** 120.5 — Damage doesn't destroy; SBAs destroy creatures with lethal damage
- **Mechanism:** Damage is marked → SBA checks `damage_marked >= toughness` → destroys
- **Minimal Board:** 2/2 creature. 3 damage dealt.
- **Action:** Deal damage. Then check SBAs.
- **Expected Result:** After damage: creature has 3 damage marked. After SBA: creature is destroyed.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**120.6** — TESTABLE. Damage marked on creature remains until cleanup step. Lethal damage = total damage ≥ toughness → SBA destroys.

**ATOM-120.6-001**
- **Rule:** 120.6 — Damage is removed during cleanup step
- **Mechanism:** `cleanup_step()` resets `damage_marked` to 0
- **Minimal Board:** 4/4 creature with 2 damage marked. Cleanup step occurs.
- **Action:** Run cleanup step
- **Expected Result:** `damage_marked` is reset to 0
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**120.7** — PURE-DEF. Source of damage. Definition of legal "source" choices. Framework.

**120.8** — TESTABLE. Source dealing 0 damage does not deal damage at all. Triggers don't fire. Replacement effects don't apply.

**ATOM-120.8-001**
- **Rule:** 120.8 — 0 damage = no damage dealt, no triggers, no replacements
- **Mechanism:** `perform_action(DealDamage)` with amount 0 is a no-op
- **Minimal Board:** Effect tries to deal 0 damage to a player.
- **Action:** Execute the damage action
- **Expected Result:** No damage event. No "whenever damage is dealt" triggers. No life change.
- **Phase:** Already implemented (`test_execute_zero_damage_is_noop`)
- **Ticket:** ALREADY-IMPLEMENTED

**120.9** — PURE-DEF. "Damage dealt" by a specific source refers only to that source's damage.

**120.10** — TESTABLE. Excess damage check for triggered abilities.

**ATOM-120.10-001**
- **Rule:** 120.10 — Excess damage to a creature = amount beyond lethal
- **Mechanism:** Excess damage calculation for triggers
- **Minimal Board:** 2/2 creature dealt 5 damage.
- **Action:** Check excess
- **Expected Result:** 3 excess damage (5 - 2 lethal = 3)
- **Phase:** Phase 7 (excess damage triggers)
- **Ticket:** Phase 7

---

### 121. Drawing a Card

**121.1** — TESTABLE. Drawing = putting top card of library into hand. Turn-based action during draw step. Also via costs/effects.

**ATOM-121.1-001**
- **Rule:** 121.1 — Drawing moves top card of library to hand
- **Mechanism:** `draw_card()` pops from library, pushes to hand
- **Minimal Board:** Player with 5 cards in library
- **Action:** Draw a card
- **Expected Result:** Library has 4 cards. Hand gains the card that was on top of library.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**121.2** — TESTABLE. Multiple cards drawn = individual draws one at a time.

**ATOM-121.2-001**
- **Rule:** 121.2 — Drawing N cards = N individual draws
- **Mechanism:** `draw_cards(n)` calls `draw_card()` N times
- **Minimal Board:** Player with 5 cards in library. Effect says "draw 3 cards."
- **Action:** Resolve
- **Expected Result:** 3 separate draw events. Library has 2 cards. Hand has 3 new cards.
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED

**121.2a** — TESTABLE. Replacement effects on "draw N cards" modify the count before individual draws.

**ATOM-121.2a-001**
- **Rule:** 121.2a — Replacement effect modifying number of draws is applied before individual draws
- **Mechanism:** Draw-count replacement in `draw_cards` before the loop
- **Minimal Board:** Player with "If you would draw cards, draw twice that many instead." Effect says "draw 3."
- **Action:** Resolve
- **Expected Result:** Player draws 6 cards (replacement doubles the count before individual draws begin)
- **Phase:** Phase 6 (replacement effects — draw replacement)
- **Ticket:** Phase 6

**121.2b** — TESTABLE. "Can't draw more than one card each turn" limits individual draws but instructions to draw multiple may be partially carried out.

**ATOM-121.2b-001**
- **Rule:** 121.2b — "Can't draw more than one card each turn" partially carries out multi-draw instructions
- **Mechanism:** Draw restriction check per individual draw
- **Minimal Board:** Player with "can't draw more than one card each turn" (e.g., Narset, Parter of Veils). Player has drawn 0 cards this turn. Effect says "draw 3 cards."
- **Action:** Attempt to draw 3
- **Expected Result:** Player draws exactly 1 card (the first draw succeeds, subsequent draws blocked by restriction). Partial execution is natural because "draw N" is N individual `draw_card()` calls, each of which checks the restriction.
- **Phase:** Phase 8 (draw restriction effects)
- **Ticket:** NEW — Draw restriction enforcement

**121.2c** — TESTABLE. Multiple players drawing → active player draws all first, then next player in turn order.

**ATOM-121.2c-001**
- **Rule:** 121.2c — Active player performs all draws first, then next player in APNAP order
- **Mechanism:** APNAP ordering in multi-player draw effects
- **Minimal Board:** Effect says "each player draws 3 cards." P0 is active.
- **Action:** Resolve
- **Expected Result:** P0 draws 3 first, then P1 draws 3
- **Phase:** Phase 8 (mass effect primitives with APNAP)
- **Ticket:** NEW — APNAP draw ordering

**121.2d** — OUT-OF-SCOPE. Shared team turns draw ordering. Phase 9.

**121.3** — TESTABLE. If library is empty and an effect offers a choice to draw, player can choose to do so (and will eventually lose to SBA). But if "can't draw" is active, the choice can't be taken.

**ATOM-121.3-001**
- **Rule:** 121.3 — Player can choose to draw from empty library (they'll just lose via SBA later)
- **Mechanism:** Optional draw from empty library is permitted
- **Minimal Board:** Player with 0 cards in library. Effect says "you may draw a card."
- **Action:** Player chooses to draw
- **Expected Result:** Draw attempt proceeds. `attempted_draw_from_empty` flag set. Player loses at next SBA check.
- **Phase:** Already partially implemented (draw_card returns error on empty library)
- **Ticket:** Phase 8 — Optional draw from empty library flag

**ATOM-121.3-002**
- **Rule:** 121.3 — "Can't draw cards" prevents the optional draw choice entirely
- **Mechanism:** DecisionProvider not offered the draw choice when "can't draw" is active
- **Minimal Board:** Player with "can't draw cards" effect active. Effect says "you may draw a card."
- **Action:** Check if the player can make the choice
- **Expected Result:** Player cannot choose to draw. The option is not presented. This is distinct from 121.2b (draw restriction per turn) — "can't draw" is absolute.
- **Phase:** Phase 8 (draw prevention effects)
- **Ticket:** Phase 8 — "Can't draw" absolute prevention

**121.3a** — PURE-DEF. Same principle applies if a different player would draw for another player.

**121.4** — TESTABLE. Drawing from empty library → lose as SBA. Cross-ref 104.3c.

**ATOM-121.4-001**
- **Rule:** 121.4 — Attempting to draw from empty library → lose at next SBA check
- **Mechanism:** SBA flag set on failed draw
- **Minimal Board:** Player with 0 cards in library
- **Action:** Attempt to draw. Check SBAs.
- **Expected Result:** Player loses the game
- **Phase:** Already implemented
- **Ticket:** ALREADY-IMPLEMENTED (cross-ref ATOM-104.3c-001)

**121.5** — BOUNDARY-DEF. Moving cards from library to hand without "draw" is not drawing. Important for draw triggers/replacements and empty-library loss.

**ATOM-121.5-001**
- **Rule:** 121.5 — Moving cards from library to hand without using "draw" is not a draw
- **Mechanism:** `move_object()` from library to hand does NOT set draw-related flags/events
- **Minimal Board:** Effect says "put the top card of your library into your hand" (no "draw" word)
- **Action:** Resolve the effect
- **Expected Result:** Card moves to hand. No "draw" event emitted. No draw triggers fire. If library was empty, this does NOT cause the player to lose.
- **Phase:** Phase 7 (trigger discrimination — draw vs. non-draw library-to-hand)
- **Ticket:** Phase 7

**121.6** — PURE-DEF. Some effects replace card draws. Framework.

**Note (121.6 Phase 6 re-examination):** When the replacement effect system is built (Phase 6), revisit this rule with specific test scenarios: (1) Draw replaced by "reveal top card, put into hand if creature, otherwise put back" (Courser of Kruphix-style). (2) Chain of draw replacements: "if you would draw, instead X" where X also involves drawing. (3) Interaction between draw replacement and "can't draw." These scenarios test the replacement effect *framework*, not 121.6 specifically.

**121.6a** — TESTABLE. Draw replacement applies even if library is empty.

**ATOM-121.6a-001**
- **Rule:** 121.6a — Draw replacement effect applies even with empty library
- **Mechanism:** Replacement effect intercepts draw event before checking library
- **Minimal Board:** Player with 0 cards in library and "If you would draw a card, instead [do something else]"
- **Action:** Attempt to draw
- **Expected Result:** The replacement effect fires. The actual draw is replaced. Player does NOT lose (draw was replaced, not attempted).
- **Phase:** Phase 6 (replacement effects — draw replacement)
- **Ticket:** Phase 6

**121.6b** — PURE-DEF. Draw replacement within a sequence completes before resuming.

**121.6c** — PURE-DEF. Additional actions on drawn card don't apply if the draw was replaced.

**121.7** — PURE-DEF. Replacement/prevention effects that result in card draws: non-replaced parts happen first, then draws one at a time.

**Note (121.7 Phase 6 re-examination):** The chain of "if you would draw, instead X" → "but X also involves drawing" → "apply next replacement" is complex and better tested when the replacement effect framework exists. Flag for Phase 6 test suite with specific scenarios involving nested draw replacements.

**121.8** — PURE-DEF. Card drawn while another spell is being cast is kept face down until the spell is fully cast.

**Note (121.8 future test):** Canonical example: Chromatic Sphere ("{1}, {T}, Sacrifice this artifact: Add one mana of any color. Draw a card.") is a *mana* ability (because it produces mana), so it resolves immediately without using the stack. If activated during the process of casting a spell (e.g., to pay a colored cost), the draw happens mid-cast. Per 121.8, the drawn card is kept face-down until the spell is fully cast — the player cannot look at it to inform targeting/modal choices already being made. This is the cleanest test case because Chromatic Sphere's draw is part of a mana ability (not a triggered ability), so it happens synchronously during cost payment. Defer to Phase 8+ when mana abilities with non-mana effects are implemented.

**121.9** — PURE-DEF. Effect allowing reveal of drawn card: player may look at it before deciding.

---

## Rules 122–123: Counters, Stickers

### 122. Counters

**122.1** — BOUNDARY-DEF. A counter is a marker on an object or player. Counters are not objects. Not tokens. Same-named counters are interchangeable.

**ATOM-122.1-001**
- **Rule:** 122.1 — Counters are markers, not objects; they have no characteristics
- **Mechanism:** Counter data stored as `HashMap<CounterType, u32>` on objects/players — no `GameObject` for counters
- **Minimal Board:** A creature with a +1/+1 counter
- **Action:** Verify the counter is a numeric entry, not a game object
- **Expected Result:** Counter exists as a count in the map. It has no zone, no controller, no characteristics.
- **Phase:** Phase 5-Pre (T14 — counter data model)
- **Ticket:** T14

**122.1a** — TESTABLE. +X/+Y counters add X to power and Y to toughness. -X/-Y counters subtract.

**ATOM-122.1a-001**
- **Rule:** 122.1a — +1/+1 counter adds 1 to power and 1 to toughness
- **Mechanism:** Counter contribution in P/T computation (Layer 7d)
- **Minimal Board:** A 2/2 creature with two +1/+1 counters
- **Action:** Compute effective P/T
- **Expected Result:** 4/4
- **Phase:** Phase 5 Layers (L04/L08 — P/T sublayer 7d: counters)
- **Ticket:** L04 / L08

**ATOM-122.1a-002**
- **Rule:** 122.1a — -1/-1 counter subtracts 1 from power and 1 from toughness
- **Mechanism:** Same as above
- **Minimal Board:** A 3/3 creature with one -1/-1 counter
- **Action:** Compute effective P/T
- **Expected Result:** 2/2
- **Phase:** Phase 5 Layers (L04/L08)
- **Ticket:** L04 / L08

**ATOM-122.1a-003**
- **Rule:** 122.1a — Non-standard P/T counters (e.g., +2/+0) do NOT annihilate with +1/+1 or -1/-1 counters
- **Mechanism:** Counter annihilation only applies to +1/+1 vs. -1/-1 specifically (per 122.3)
- **Minimal Board:** A creature with 1 +2/+0 counter and 2 -1/-1 counters
- **Action:** Check SBAs
- **Expected Result:** No annihilation occurs. The +2/+0 counter and -1/-1 counters coexist. Net P/T effect: +2/+0 from the +2/+0 counter, -2/-2 from the two -1/-1 counters = net +0/-2.
- **Phase:** Phase 5 Layers (L04/L08 — counter P/T computation)
- **Ticket:** L04 / L08
- **Note:** +2/+0 counters exist on cards like Frankenstein's Monster. Only +1/+1 and -1/-1 specifically annihilate (rule 122.3).

**122.1b** — TESTABLE. Keyword counters cause the object to gain that keyword.

**ATOM-122.1b-001**
- **Rule:** 122.1b — A keyword counter (e.g., flying counter) gives the permanent that keyword
- **Mechanism:** Keyword counter contribution in Layer 6 (abilities)
- **Minimal Board:** A 2/2 creature with a flying counter
- **Action:** Query abilities
- **Expected Result:** Creature has flying
- **Phase:** Phase 5 Layers (L06 — keyword counters)
- **Ticket:** L06

**122.1c** — TESTABLE. Shield counters create replacement + prevention effects protecting the permanent.

**ATOM-122.1c-001**
- **Rule:** 122.1c — Shield counter prevents destruction and removes a counter
- **Mechanism:** Shield counter replacement/prevention effects
- **Minimal Board:** A creature with 1 shield counter. An effect would destroy it.
- **Action:** Apply the destruction effect
- **Expected Result:** Creature is NOT destroyed. Shield counter is removed instead.
- **Phase:** Phase 6 (replacement/prevention effects)
- **Ticket:** Phase 6 — Shield counters

**ATOM-122.1c-002**
- **Rule:** 122.1c — Shield counter prevents damage and removes a counter
- **Mechanism:** Shield counter prevention effect
- **Minimal Board:** A creature with 1 shield counter. 3 damage would be dealt.
- **Action:** Deal the damage
- **Expected Result:** Damage is prevented. Shield counter is removed. Creature has 0 damage marked.
- **Phase:** Phase 6
- **Ticket:** Phase 6

**122.1d** — TESTABLE. Stun counters: "If a permanent with a stun counter would become untapped, instead remove a stun counter."

**ATOM-122.1d-001**
- **Rule:** 122.1d — Stun counter prevents untapping; removes counter instead
- **Mechanism:** Stun counter replacement effect on untap
- **Minimal Board:** A tapped creature with 1 stun counter. Untap step occurs.
- **Action:** Attempt to untap
- **Expected Result:** Creature stays tapped. Stun counter is removed. Next untap step, creature will untap normally.
- **Phase:** Phase 6 (replacement effects)
- **Ticket:** Phase 6 — Stun counters

**122.1e** — TESTABLE. Planeswalker's loyalty counters = its loyalty. 0 loyalty → SBA puts it in graveyard.

**ATOM-122.1e-001**
- **Rule:** 122.1e — Planeswalker with 0 loyalty is put into graveyard as SBA
- **Mechanism:** SBA check for loyalty == 0 on planeswalkers (704.5i)
- **Minimal Board:** Planeswalker with 0 loyalty counters on battlefield
- **Action:** Check SBAs
- **Expected Result:** Planeswalker is put into its owner's graveyard
- **Phase:** Phase 5-Pre (T14 — loyalty counters + T16 — SBA for 0 loyalty)
- **Ticket:** T14 + T16

**122.1f** — TESTABLE. 10+ poison counters → player loses as SBA. "Poisoned" = 1+ poison counter.

**ATOM-122.1f-001**
- **Rule:** 122.1f — Player with 10+ poison counters loses as SBA
- **Mechanism:** SBA check for poison counters (704.5c)
- **Minimal Board:** Player with 10 poison counters
- **Action:** Check SBAs
- **Expected Result:** Player loses the game
- **Phase:** Phase 5-Pre (T16)
- **Ticket:** T16 (cross-ref ATOM-104.3d-001)

**122.1g** — TESTABLE. Battle with 0 defense counters → SBA puts it in graveyard (unless it's the source of a triggered ability that hasn't left the stack).

**ATOM-122.1g-001**
- **Rule:** 122.1g — Battle with 0 defense is put into graveyard as SBA
- **Mechanism:** SBA check for defense == 0 on battles
- **Minimal Board:** Battle with 0 defense counters
- **Action:** Check SBAs
- **Expected Result:** Battle is put into its owner's graveyard
- **Phase:** Phase 9 (battles)
- **Ticket:** Phase 9

**122.1h** — TESTABLE. Finality counter: "If this permanent would be put into a graveyard from the battlefield, exile it instead."

**ATOM-122.1h-001**
- **Rule:** 122.1h — Finality counter causes exile instead of going to graveyard from battlefield
- **Mechanism:** Finality counter replacement effect on zone change
- **Minimal Board:** A creature with a finality counter. An effect destroys it.
- **Action:** Creature is destroyed
- **Expected Result:** Creature goes to exile instead of graveyard
- **Phase:** Phase 6 (replacement effects)
- **Ticket:** Phase 6 — Finality counters

**122.1i** — DEFERRED (Phase 8). Rad counters. Legal in Modern, Legacy, and Vintage (Fallout Commander cards). In-scope for completeness.

**ATOM-122.1i-001**
- **Rule:** 122.1i — Rad counters cause mill + life loss at beginning of precombat main phase
- **Mechanism:** Rad counter triggered ability: "At the beginning of your precombat main phase, for each rad counter you have, mill a card. For each nonland card milled this way, you lose 1 life and a rad counter."
- **Minimal Board:** Player with 3 rad counters. Top 3 cards of library: creature, land, instant.
- **Action:** Precombat main phase begins
- **Expected Result:** Mill 3 cards. 2 nonland cards milled → lose 2 life, remove 2 rad counters. Player now has 1 rad counter.
- **Phase:** Phase 8 (rad counters + triggered ability)
- **Ticket:** Phase 8 — Rad counter implementation

**122.2** — TESTABLE. Counters on an object are NOT retained when it changes zones. They cease to exist.

**ATOM-122.2-001**
- **Rule:** 122.2 — Counters are removed when an object changes zones
- **Mechanism:** `cleanup_zone_state()` or zone change clears counter map
- **Minimal Board:** A creature with 3 +1/+1 counters. It dies (moves to graveyard).
- **Action:** Move to graveyard
- **Expected Result:** The creature card in the graveyard has 0 counters
- **Phase:** Phase 5-Pre (T14 — counter data model + zone change cleanup)
- **Ticket:** T14

**122.3** — TESTABLE. +1/+1 and -1/-1 counters on the same permanent annihilate as SBA.

**ATOM-122.3-001**
- **Rule:** 122.3 — If a permanent has both +1/+1 and -1/-1 counters, remove N of each where N = min(plus, minus)
- **Mechanism:** SBA check for counter annihilation (704.5q)
- **Minimal Board:** A creature with 3 +1/+1 counters and 2 -1/-1 counters
- **Action:** Check SBAs
- **Expected Result:** 2 of each are removed. Creature now has 1 +1/+1 counter and 0 -1/-1 counters.
- **Phase:** Phase 5-Pre (T16 — SBA for counter annihilation)
- **Ticket:** T16

**122.4** — TESTABLE. "Can't have more than N counters of a kind" → SBA removes excess.

**ATOM-122.4-001**
- **Rule:** 122.4 — If a permanent has more than N counters of a restricted kind, SBA removes excess
- **Mechanism:** SBA check for counter cap
- **Minimal Board:** A permanent with "can't have more than 3 charge counters" and 5 charge counters
- **Action:** Check SBAs
- **Expected Result:** 2 charge counters removed. Permanent now has 3.
- **Phase:** Phase 8 (counter cap effects)
- **Ticket:** Phase 8

**122.5** — TESTABLE. "Move" a counter = remove from one object, put on another. If either action is impossible, nothing happens.

**ATOM-122.5-001**
- **Rule:** 122.5 — Moving a counter removes from source and puts on destination; fails if either action is impossible
- **Mechanism:** Counter move logic
- **Minimal Board:** Object A has 1 +1/+1 counter. Effect says "move a +1/+1 counter from A to B."
- **Action:** Move the counter
- **Expected Result:** A has 0 +1/+1 counters. B has 1 +1/+1 counter.
- **Phase:** Phase 8 (counter manipulation primitives)
- **Ticket:** Phase 8

**ATOM-122.5-002**
- **Rule:** 122.5 — Can't move a counter if first and second objects are the same
- **Mechanism:** Counter move validation: same-object check
- **Minimal Board:** Object A has 2 +1/+1 counters. Effect says "move a +1/+1 counter from A to A."
- **Action:** Attempt the move
- **Expected Result:** Nothing happens (source and destination are the same object)
- **Phase:** Phase 8
- **Ticket:** Phase 8

**ATOM-122.5-003**
- **Rule:** 122.5 — Can't move a counter the source doesn't have
- **Mechanism:** Counter move validation: source has counter check
- **Minimal Board:** Object A has 0 +1/+1 counters. Effect says "move a +1/+1 counter from A to B."
- **Action:** Attempt the move
- **Expected Result:** Nothing happens (A doesn't have the counter to remove)
- **Phase:** Phase 8
- **Ticket:** Phase 8

**ATOM-122.5-004**
- **Rule:** 122.5 — Can't move a counter if destination can't receive counters
- **Mechanism:** Counter move validation: destination can-receive check
- **Minimal Board:** Object A has 1 +1/+1 counter. Object B has "counters can't be put on this permanent." Effect says "move a +1/+1 counter from A to B."
- **Action:** Attempt the move
- **Expected Result:** Nothing happens (B can't receive counters). A retains its counter.
- **Phase:** Phase 8
- **Ticket:** Phase 8

**ATOM-122.5-005**
- **Rule:** 122.5 — Can't move a counter if either object left the correct zone
- **Mechanism:** Counter move validation: zone check at resolution time
- **Minimal Board:** Object A has 1 +1/+1 counter on battlefield. Effect to move counter is on the stack. Before resolution, A leaves the battlefield (e.g., bounced to hand).
- **Action:** Resolve the move effect
- **Expected Result:** Nothing happens (A is no longer on the battlefield). The counter move fails.
- **Phase:** Phase 8
- **Ticket:** Phase 8

**122.6** — PURE-DEF. "Put counters on" includes entering with counters. Framework.

**122.6a** — TESTABLE. If entering with counters and no player specified to put them on, the permanent's controller puts them on.

**ATOM-122.6a-001**
- **Rule:** 122.6a — Controller places ETB counters when no player is specified
- **Mechanism:** Counter placement routing to controller in ETB processing
- **Minimal Board:** A creature with "enters the battlefield with two +1/+1 counters" (no player specified). P0 controls it.
- **Action:** Creature enters the battlefield
- **Expected Result:** P0 (the controller) is the player who puts the counters on. This matters for effects that trigger on "a player putting counters on a permanent" or effects that modify counter placement (e.g., Doubling Season only applies to counters placed by you).
- **Phase:** Phase 6 (ETB counter placement + replacement effects)
- **Ticket:** Phase 6 — ETB counter placement routing

**122.7** — TESTABLE. "When the Nth [kind] counter is put on" triggers when crossing the N threshold.

**ATOM-122.7-001**
- **Rule:** 122.7 — Trigger fires when counter count crosses from below N to N or above
- **Mechanism:** Counter threshold trigger in `put_counters()`
- **Minimal Board:** Permanent with 2 charge counters. "When the third charge counter is put on this, draw a card." Put 1 charge counter.
- **Action:** Put 1 counter (now 3 total)
- **Expected Result:** Trigger fires (crossed from 2 to 3, which meets "third counter" threshold)
- **Phase:** Phase 7 (triggered abilities — counter triggers)
- **Ticket:** Phase 7
- **Note:** Sagas are the primary use case for this rule (lore counter threshold triggers chapter abilities). The mechanism is general but Sagas will be the most common cards exercising it.

**122.8** — TESTABLE (DEFERRED to Phase 7). Triggered ability putting one object's counters on another: puts same number/kind, not "move." This is distinct from 122.5 (move) — it creates NEW counters on the destination equal to what the source had.

**ATOM-122.8-001**
- **Rule:** 122.8 — Triggered ability referencing counters on a sacrificed permanent creates new counters, doesn't move
- **Mechanism:** Counter creation (not move) in triggered ability resolution
- **Minimal Board:** Creature A with 3 +1/+1 counters. Triggered ability: "When A dies, put a number of +1/+1 counters equal to the number A had on target creature." A dies.
- **Action:** Trigger resolves targeting creature B
- **Expected Result:** B gets 3 NEW +1/+1 counters. This is "put" not "move" — replacement effects that apply to "putting counters" (e.g., Doubling Season) DO apply. A's counters ceased to exist when A left the battlefield (122.2).
- **Phase:** Phase 7 (triggered abilities)
- **Ticket:** Phase 7

**122.9** — TESTABLE (DEFERRED to Phase 7). Activated ability with sacrifice cost putting counters: same as 122.8 — creates new counters equal to what the sacrificed permanent had.

**ATOM-122.9-001**
- **Rule:** 122.9 — Activated ability referencing counters on a sacrificed permanent creates new counters
- **Mechanism:** Same as 122.8 but for activated abilities with sacrifice costs
- **Minimal Board:** Creature A with 2 +1/+1 counters. Activated ability: "Sacrifice A: Put a number of +1/+1 counters equal to the number A had on target creature."
- **Action:** Activate the ability targeting creature B. Resolve.
- **Expected Result:** B gets 2 NEW +1/+1 counters. Same reasoning as 122.8.
- **Phase:** Phase 7 (activated abilities with sacrifice costs)
- **Ticket:** Phase 7

---

### 123. Stickers

**123.1–123.9** — OUT-OF-SCOPE. Stickers are an Unfinity mechanic. All sub-rules (123.1 through 123.9 and their sub-rules) are out of scope for this simulator.

---

# Classification Summary

## Atomic Test Count

| ID Range | TESTABLE | BOUNDARY-DEF | PURE-DEF | META | DEFERRED | OUT-OF-SCOPE | ALREADY-IMPL | Total Tests |
|----------|----------|--------------|----------|------|----------|--------------|--------------|-------------|
| 100.x    | 4 rules  | 0            | 6        | 0    | 3        | 3            | 0            | 7           |
| 101.x    | 2 rules  | 0            | 4        | 3    | 0        | 0            | 0            | 3           |
| 102.x    | 0        | 0            | 2        | 0    | 0        | 1            | 0            | 0           |
| 103.x    | 5 rules  | 0            | 3        | 0    | 5        | 6            | 2            | 6           |
| 104.x    | 9 rules  | 0            | 4        | 0    | 1        | 3            | 4            | 12          |
| 105.x    | 5 rules  | 2            | 1        | 0    | 0        | 0            | 1            | 10          |
| 106.x    | 10 rules | 2            | 5        | 0    | 3        | 0            | 2            | 14          |
| 107.x    | 14 rules | 1            | 8        | 1    | 8        | 2            | 4            | 32          |
| 108.x    | 3 rules  | 1            | 3        | 0    | 2        | 1            | 3            | 5           |
| 109.x    | 3 rules  | 2            | 5        | 0    | 1        | 2            | 1            | 4           |
| 110.x    | 7 rules  | 2            | 3        | 0    | 0        | 0            | 4            | 14          |
| 111.x    | 8 rules  | 1            | 3        | 0    | 0        | 0            | 0            | 14          |
| 112.x    | 2 rules  | 0            | 4        | 0    | 0        | 0            | 1            | 3           |
| 113.x    | 8 rules  | 2            | 18       | 1    | 2        | 0            | 3            | 15          |
| 114.x    | 1 rule   | 2            | 1        | 0    | 0        | 0            | 0            | 3           |
| 115.x    | 8 rules  | 0            | 7        | 0    | 4        | 0            | 3            | 11          |
| 116.x    | 3 rules  | 1            | 0        | 0    | 8        | 2            | 2            | 7           |
| 117.x    | 10 rules | 0            | 6        | 0    | 0        | 1            | 8            | 16          |
| 118.x    | 16 rules | 0            | 11       | 0    | 0        | 0            | 5            | 27          |
| 119.x    | 8 rules  | 0            | 0        | 0    | 1        | 1            | 4            | 16          |
| 120.x    | 10 rules | 1            | 5        | 0    | 0        | 0            | 7            | 16          |
| 121.x    | 6 rules  | 1            | 5        | 0    | 0        | 1            | 3            | 10          |
| 122.x    | 14 rules | 1            | 1        | 0    | 3        | 0            | 0            | 25          |
| 123.x    | 0        | 0            | 0        | 0    | 0        | 1 (all)      | 0            | 0           |
| **TOTAL**| **~156** | **~19**      | **~105** | **5**| **41**   | **24**       | **~57**      | **270**     |

**Total atomic test specs generated: 270** (up from 174 pre-audit)

**Audit-driven additions: ~54 new atoms** across rules 100-122. Key additions:
- Reclassifications: 100.3→DEFERRED, 106.3→TESTABLE, 106.12a/b→DEFERRED, 107.2→META, 113.6b→META, 113.12→BOUNDARY-DEF, 115.6/115.8/115.9a→TESTABLE, 116.2c/d/m→DEFERRED, 118.12/118.12b→TESTABLE, 122.1i→DEFERRED, 122.6a→TESTABLE, 122.8/122.9→TESTABLE DEFERRED
- New atoms for: Devoid CDA, Smallpox ordering, Standstill "if you do", Dermoplasm + Gather Specimens, counter movement failure modes (4), token bounce/cease-to-exist, two-brid cost reduction, mana restriction persistence, simultaneous lifelink triggers, non-standard P/T counters, rad counters, ETB counter placement routing, and more

---

## ALREADY-IMPLEMENTED Rules (cross-referenced to existing code)

| Rule | Description | Implementation Location |
|------|-------------|------------------------|
| 103.3 | Deck shuffle → libraries | `Game::setup()` |
| 103.4 | Starting life = 20 | `GameConfig.starting_life` |
| 103.8a | Starting player skips first draw (2p) | `Game::run_turn()` flag |
| 104.2a | Last player standing wins | `check_game_over()` |
| 104.3b | Life ≤ 0 → lose (SBA) | `sba.rs` |
| 104.3c | Draw from empty lib → lose (SBA) | `sba.rs` + `draw_card()` |
| 104.4a | All lose simultaneously → draw | `check_game_over()` |
| 105.1 | Five colors | `Color` enum |
| 106.1a | Five mana colors | `ManaSymbol::Colored` |
| 106.1b | Six mana types (5 color + colorless) | `types/mana.rs` |
| 106.4 | Mana pool empties at step/phase end | `engine/turns.rs` |
| 107.1a | Integer-only game values | Type system |
| 107.4 | All mana symbol types | `ManaSymbol` enum |
| 107.4c | {C} payable only with colorless | `ManaPool::pay` |
| 107.5 | Tap cost / summoning sickness | `engine/costs.rs` |
| 107.6 | Untap cost validation | `engine/costs.rs` |
| 108.3 | Owner = started in deck | `GameObject.owner` |
| 108.4 | Controller only on stack/battlefield | Architecture |
| 109.2 | "Creature" = battlefield permanent | Targeting |
| 110.2 | Permanent controller from ETB | `stack.rs` / `init_zone_state_with_controller` |
| 110.4a | `is_permanent()` predicate | `CardType` |
| 110.4b | Permanent spell check | Stack resolution |
| 110.5b | ETB untapped default | `BattlefieldEntity::new()` |
| 110.5d | No status for non-battlefield cards | Architecture |
| 112.2 | Spell controller = caster | `StackEntry.controller` |
| 113.3 | Four ability categories | `AbilityDef.ability_type` |
| 113.4 | Mana abilities don't use stack | `activate_mana_ability()` |
| 113.7a | Ability independent of source | `StackEntry` is separate |
| 113.9 | Abilities ≠ spells, can't be "counter spell'd" | Targeting validation |
| 115.2 | Default targets = battlefield | Targeting |
| 115.4 | "Any target" categories | `TargetSpec::AnyTarget` |
| 115.5 | Can't target self | Target validation |
| 116.2a | Land play = special action | `play_land()` |
| 116.3 | Priority after special action | Priority system |
| 117.1a | Sorcery-speed / instant-speed casting | `check_cast_legality()` |
| 117.1b | Activated ability timing | Activation check |
| 117.1d | Mana ability during casting | `queue_tap_and_cast()` |
| 117.2c | TBAs before priority | `run_step()` |
| 117.3a | No priority during untap | Untap step |
| 117.3b | Priority after resolution → active player | `resolve_top_of_stack()` |
| 117.3c | Priority after cast → caster | Priority system |
| 117.3d | Pass priority → next player | `pass_priority()` |
| 117.4 | All pass → resolve/advance | `pass_priority()` |
| 117.7 | LIFO stack | Stack Vec |
| 118.3 | Can't pay without resources | `engine/costs.rs` |
| 118.3a | Paying mana removes from pool | `ManaPool::spend()` |
| 118.3b | Paying life subtracts | `pay_life()` |
| 118.5 | {0} not auto-paid | Casting pipeline |
| 118.10 | One payment per cost | Sacrifice removes permanent |
| 119.1 | Starting life 20 | `GameConfig` |
| 119.2 | Damage → life loss | `perform_action(DealDamage)` |
| 119.3 | Gain/lose life | `gain_life()` / `lose_life()` |
| 119.4 | Life payment validation | `Cost::PayLife` |
| 119.6 | 0 life → SBA loss | `sba.rs` |
| 120.1a | Damage only to creatures/PWs/battles/players | `DealDamage` validation |
| 120.2a | Combat damage = power | `assign_combat_damage()` |
| 120.3a | Normal damage → life loss | `perform_action` |
| 120.3e | Normal damage → marked | `damage_marked` |
| 120.3f | Lifelink | `apply_lifelink()` |
| 120.4a | Trample excess / deathtouch excess | `assign_trample_damage` / `lethal_damage_for` |
| 120.5 | Damage ≠ destroy; SBA does | `sba.rs` |
| 120.6 | Damage cleared at cleanup | `cleanup_step()` |
| 120.8 | 0 damage = no-op | `test_execute_zero_damage_is_noop` |
| 121.1 | Draw = top of library → hand | `draw_card()` |
| 121.2 | N draws = N individual | `draw_cards(n)` |
| 121.4 | Empty library draw → SBA loss | `sba.rs` |

---

## OUT-OF-SCOPE Summary (Permanently Excluded)

> **Verified by `extract_classifications.py` — 24 entries, 0 overlap with DEFERRED.**

| Rule(s) | Topic |
|---------|-------|
| 100.2d | Supplementary decks (Attractions, Planechase, Archenemy) |
| 100.4c–d | Team variant sideboard rules |
| 100.6–100.7 | Tournament / casual Un-set rules |
| 102.3–102.4 | Multiplayer teams |
| 103.1a–c | Shared team turns / Archenemy / Power Play |
| 103.2d | Sticker sheets (Un-set) |
| 103.2e | Conspiracy reveal |
| 103.3a | Supplementary deck shuffle |
| 103.7 | Planechase starting plane |
| 103.8b | Two-Headed Giant first draw |
| 104.2c–d | Team wins / Emperor |
| 104.3g–k | Team losses / limited range / tournament |
| 104.4d–i | Team draws / intentional draws |
| 107.11–107.12 | Planechase symbols |
| 107.17–107.17a | Ticket counters (Sticker/Un-set) |
| 108.3a | Planechase planar deck owner |
| 109.2d | Scheme cards (Archenemy) |
| 109.4d–g | Variant controllers (Planechase / Archenemy / Conspiracy) |
| 116.2i | Planechase planar die roll |
| 116.2j | Conspiracy Draft face-up flip |
| 117.6 | Shared team turns priority |
| 119.4a | 2HG life payment |
| 121.2d | Shared team turns draw order |
| 123.1–123.9 | Stickers (Un-set) |

## DEFERRED Summary (Planned for Later Phases)

> **Verified by `extract_classifications.py` — 41 entries, 0 overlap with OUT-OF-SCOPE.**

| Rule(s) | Topic | Target Phase | Type |
|---------|-------|-------------|------|
| 100.2c | Commander deckbuilding | Phase 9 | DEFERRED |
| 100.3 | Coins/dice mechanics (705/706) | Phase 8 | DEFERRED |
| 100.4b | Limited deck validation / sideboard swap | Match mgmt | DEFERRED |
| 103.2b | Companion reveal | Phase 9 (D20) | DEFERRED |
| 103.2c | Commander setup | Phase 9 | DEFERRED |
| 103.4a–e | Variant life totals (2HG, Vanguard, Commander, Brawl, Archenemy) | Phase 9 | DEFERRED |
| 103.5a–d | Mulligan variants | Phase 9 | DEFERRED |
| 103.5c | Multiplayer first mulligan free | Phase 9 | DEFERRED |
| 104.6 | Karn Liberated restart | Phase 9+ | DEFERRED |
| 106.12a | "Tapped for mana" event tracking | Phase 7/8 | TESTABLE+DEF |
| 106.12b | "Tapped for mana" replacement effects | Phase 6 | TESTABLE+DEF |
| 106.13 | Drain Power | Phase 8 | DEFERRED |
| 107.3b | Cast free with undefined X → X=0 | Phase 5-Pre | TESTABLE+DEF |
| 107.3d | X in special action costs (suspend, morph) | Phase 9 | DEFERRED |
| 107.3e | X in triggered ability retains value | Phase 7 | TESTABLE+DEF |
| 107.3n | Delayed trigger X persistence | Phase 7 | TESTABLE+DEF |
| 107.8–107.8b | Level Up cards | Phase 8 | DEFERRED |
| 107.15a-b | Saga rules | Phase 7/8 | DEFERRED |
| 107.16–107.16a | Class cards | Phase 8+ | DEFERRED |
| 107.18 | Pawprint symbol | Phase 8 | DEFERRED |
| 108.3b | Cards from outside the game (Wish) | Phase 8 | DEFERRED |
| 108.5 | Dungeon cards (partially) | Phase 8 | DEFERRED |
| 109.4c | Emblem controller | Phase 8 | DEFERRED |
| 113.6n | Deck construction abilities | Pre-game / Phase 9 | DEFERRED |
| 113.6p | Command zone abilities (emblem/plane/vanguard/scheme) | Phase 9 | DEFERRED |
| 115.7a–f | Target changing/choosing details | Phase 8+ | DEFERRED |
| 115.8 | Target-changing effects (Deflection, Spellskite) | Phase 8 | TESTABLE+DEF |
| 115.9a | Count targets on spells | Phase 7 | TESTABLE+DEF |
| 115.9b–c | "Targets only" nuances | Phase 7/8 | TESTABLE+DEF |
| 116.2b | Morph face-up | Phase 9 | DEFERRED |
| 116.2c | End continuous effect special action (Licid) | Phase 8+ | DEFERRED |
| 116.2d | Pay to ignore restriction (Leonin Arbiter) | Phase 8 | DEFERRED |
| 116.2e | Circling Vultures | Phase 8 | DEFERRED |
| 116.2f | Suspend | Phase 9 | DEFERRED |
| 116.2h | Foretell | Phase 9 | DEFERRED |
| 116.2k | Plot | Phase 9 | DEFERRED |
| 116.2m | Room enchantments (unlock second room) | Phase 8+ | DEFERRED |
| 119.1a–e | Variant starting life totals | Phase 9 | DEFERRED |
| 122.1i | Rad counters | Phase 8 | DEFERRED |
| 122.8 | Triggered ability counter transfer | Phase 7 | TESTABLE+DEF |
| 122.9 | Activated ability counter transfer | Phase 7 | TESTABLE+DEF |

---

## NEW Tickets Identified

| Ticket | Rule(s) | Description | Phase |
|--------|---------|-------------|-------|
| NEW-CH1-001 | 101.4 | APNAP ordering for simultaneous player choices | Phase 7/8 |
| NEW-CH1-002 | 104.2b, 104.3e | WinTheGame / LoseTheGame primitives | Phase 8 |
| NEW-CH1-003 | 104.3a | Concession action | Phase 8/9 |
| NEW-CH1-004 | 104.3f | Simultaneous win+lose → player loses | Phase 8 |
| NEW-CH1-005 | 104.4b | Mandatory loop detection → draw (D11) | Post-v1 |
| NEW-CH1-006 | 104.4c | DrawTheGame primitive | Phase 8 |
| NEW-CH1-007 | 105.4 | Color choice validation (reject colorless) | Phase 8 |
| NEW-CH1-008 | 106.5 | Undefined mana type produces nothing | Phase 8 |
| NEW-CH1-009 | 106.6a | Mana replacement restriction propagation | Phase 6 |
| NEW-CH1-010 | 106.7 | "Could produce" mana query | Phase 8 |
| NEW-CH1-011 | 106.8 | Hybrid mana production (choose half) | Phase 8 |
| NEW-CH1-012 | 106.9-11 | Phyrexian / generic / snow mana production | Phase 8 |
| NEW-CH1-013 | 107.1b | Negative-to-zero clamping for effect results | Phase 5/8 |
| NEW-CH1-014 | 107.3g | Mana value calculation with X=0 off-stack | Phase 5 |
| NEW-CH1-015 | 107.3h | X=0 for non-stack mana cost payments | Phase 8 |
| NEW-CH1-016 | 107.4e | Hybrid mana payment | Phase 5-Pre |
| NEW-CH1-017 | 107.4f | Phyrexian mana payment (life option) | Phase 5-Pre |
| NEW-CH1-018 | 107.4h | Snow mana source tracking + payment | Phase 8 |
| NEW-CH1-019 | 107.14 | Energy counter system | Phase 8 |
| NEW-CH1-020 | 108.4a | Controller-to-owner fallback | Phase 8 |
| NEW-CH1-021 | 109.1 | Emblem object type | Phase 8 |
| NEW-CH1-022 | 110.2a | ETB controller from effect controller | Phase 8 |
| NEW-CH1-023 | 111.8 | Token zone-change lock after leaving battlefield | Phase 8 |
| NEW-CH1-024 | 114.2 | Emblem creation + command zone | Phase 8 |
| NEW-CH1-025 | 118.7e | Hybrid cost reduction | Phase 8 |
| NEW-CH1-026 | 118.7f | Phyrexian cost reduction | Phase 8 |
| NEW-CH1-027 | 118.7g | Snow cost reduction → generic | Phase 8 |
| NEW-CH1-028 | 118.14 | "Mana of any type" cost override | Phase 8 |
| NEW-CH1-029 | 119.4b | Zero-life payment always legal | Phase 8 |
| NEW-CH1-030 | 119.5 | SetLife primitive (gain/lose difference) | Phase 8 |
| NEW-CH1-031 | 119.7 | "Can't gain life" prevention | Phase 8 |
| NEW-CH1-032 | 119.8 | "Can't lose life" prevention | Phase 8 |
| NEW-CH1-033 | 121.2b | Draw restriction enforcement | Phase 8 |
| NEW-CH1-034 | 121.2c | APNAP draw ordering | Phase 8 |

---

## META Rules (Deferred Concrete Tests)

| Rule | Principle | Concrete Tests Deferred To |
|------|-----------|---------------------------|
| 101.1 | Card text overrides rules | Per-card/per-keyword sessions (702.x, 604, 609, 613, 614) |
| 101.2 | "Can't" overrides "can" | Per-system sessions (combat 508/509, targeting 115, life 119, damage 120, lands 305, casting 601, activation 602) |
| 101.3 | Impossible instructions ignored | Per-system sessions (608 resolution, 701 keyword actions) |

---

## Phase Dependency Heatmap

| Phase | Test Specs Waiting |
|-------|--------------------|
| **Already Implemented** | ~57 (verification only) |
| **Phase 5-Pre** | ~24 (T03, T05, T06, T10, T12, T13, T14, T16, T18, T19, T21a) |
| **Phase 5 Layers** | ~20 (L01, L03, L04, L06, L08, L09, L10, L11, L18) |
| **Phase 6 (Replacement)** | ~14 (replacement effects, prevention, ETB tapped, draw replacement, shield/stun/finality counters, ETB counter routing, life gain replacement) |
| **Phase 7 (Triggers)** | ~16 (trigger queue, SBA+trigger loop, life gain triggers, counter triggers, excess damage, Stifle, "if you do" failure, counter-on-sacrifice triggers, spell copy controller) |
| **Phase 8 (Effects/Cards)** | ~85 (tokens, emblems, loyalty, infect, wither, toxic, mana production variants, cost reduction variants, life exchange, energy, Aura, rad counters, draw restrictions, "can't gain/lose life", mana restrictions, target-changing, room enchantments, etc.) |
| **Phase 9 (Formats)** | ~7 (commander damage, multiplayer, companion, battles, face-down) |
| **Post-v1** | ~2 (mandatory loop detection, continuous effects on stack) |

---

*End of Session 1 — Chapter 1: Game Concepts (Rules 100–123)*
*270 atomic test specs generated across 24 top-level rule sections (post-audit).*
*~40 new tickets identified. 5 META rules. ~57 rules verified as already implemented.*
*Audit applied: ~96 new atoms added, ~15 reclassifications, cross-references and architectural notes throughout.*
