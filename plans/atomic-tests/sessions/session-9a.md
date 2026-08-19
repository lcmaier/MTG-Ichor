# Session 9A — Atomic Test Specifications

**CR Sections:** 703 (Turn-Based Actions), 704 (State-Based Actions), 705 (Coin Flipping), 706 (Die Rolling), 707 (Copying Objects), 708 (Face-Down Spells/Permanents), 709 (Split Cards), 710 (Flip Cards), 711 (Leveler Cards), 712 (Double-Faced Cards)

**Source file:** `MTG-Rules/LLM-Chapter-Splits/ch7-pt-4.txt`

**Cross-references:** T13–T16 (SBAs), Phase 6 (copy/Layer 1), Phase 7 (triggers), Phase 9 (DFCs, split, face-down)

---

## Chunk Plan

| Chunk | Rule Range | Focus |
|-------|-----------|-------|
| 1 | 703.x + 704.x | Turn-based actions + State-based actions |
| 2 | 705.x + 706.x + 707.x | Coin flipping, die rolling, copying objects |
| 3 | 708.x + 709.x + 710.x + 711.x + 712.x | Face-down, split, flip, leveler, DFCs |
| 4 | — | Classification summary, composition tests, gap report |

---

## Chunk 1 — 703.x Turn-Based Actions + 704.x State-Based Actions

### 703. Turn-Based Actions

**703.1** — PURE-DEF. Defines what TBAs are. No independent mechanical consequence.

**703.1a** — PURE-DEF. Distinguishes TBAs from triggered abilities. Prerequisite for understanding triggers (Phase 7).

**703.2** — PURE-DEF. TBAs are not controlled by any player.

**703.3** — TESTABLE. TBAs happen before SBAs, triggers, and priority.

**ATOM-703.3-001**
- **Rule:** 703.3 — TBAs are dealt with before SBAs, triggers, and priority.
- **Mechanism:** TBA ordering relative to SBA and priority
- **Minimal Board:** Active player at 0 life, entering draw step with 1 card in library.
- **Action:** Draw step begins.
- **Expected Result:** The draw TBA executes first (player draws a card). Only THEN are SBAs checked. The player loses to 704.5a (0 life), not 704.5b — the draw succeeded because the TBA happened before the SBA check.
- **Phase:** Phase 1 (turn structure)
- **Ticket:** ALREADY-IMPLEMENTED (turns.rs processes TBAs before priority/SBA loop)

**703.4** — PURE-DEF. Introduces the TBA list. No independent consequence.

**703.4a** — DEFERRED — Phase 9: Phasing. Phasing TBA during untap step.

**703.4b** — DEFERRED — Phase 9: Day/Night designation check during untap step.

**703.4c** — TESTABLE. Untap TBA: active player untaps their permanents simultaneously.

**ATOM-703.4c-001**
- **Rule:** 703.4c — Active player determines which permanents to untap, then untaps them all simultaneously.
- **Mechanism:** Untap step TBA
- **Minimal Board:** Active player controls two tapped creatures.
- **Action:** Untap step begins.
- **Expected Result:** Both creatures are untapped simultaneously. (No selective untap without effects — all permanents untap by default.)
- **Phase:** Phase 1
- **Ticket:** ALREADY-IMPLEMENTED (turns.rs untap step)

**ATOM-703.4c-002**
- **Rule:** 703.4c — "Determines which permanents they control will untap" — effects may restrict untapping.
- **Mechanism:** Selective untap with restriction effect
- **Minimal Board:** Active player controls Winter Orb ("Players can't untap more than one land during their untap steps") and two tapped lands + one tapped creature.
- **Action:** Untap step begins.
- **Expected Result:** Player untaps at most one land (DecisionProvider chooses which). The creature untaps normally (not restricted by Winter Orb). The other land remains tapped.
- **Phase:** Phase 5 (continuous effects restricting untap)
- **Ticket:** NEW — Untap restriction effects (703.4c + continuous effects)

**703.4d** — TESTABLE. Draw step TBA: active player draws a card.

**ATOM-703.4d-001**
- **Rule:** 703.4d — Immediately after draw step begins, active player draws a card.
- **Mechanism:** Draw step TBA
- **Minimal Board:** Active player with 3 cards in library, 0 in hand.
- **Action:** Draw step begins.
- **Expected Result:** Player draws 1 card. Hand size = 1, library size = 2.
- **Phase:** Phase 1
- **Ticket:** ALREADY-IMPLEMENTED (turns.rs draw step)

**703.4e** — OUT-OF-SCOPE. Archenemy scheme action.

**703.4f** — DEFERRED — Phase 8–9: Saga lore counter TBA during precombat main phase.

**703.4g** — OUT-OF-SCOPE. Attractions (Un-set mechanic).

**703.4h** — OUT-OF-SCOPE. Multiplayer defending player choice during beginning of combat (Two-Headed Giant / multiplayer variant — not supporting 2HG or multiplayer attack routing).

**703.4i** — ALREADY-IMPLEMENTED. Declare attackers TBA (combat/steps.rs).

**703.4j** — ALREADY-IMPLEMENTED. Declare blockers TBA (combat/steps.rs).

**703.4k** — ALREADY-IMPLEMENTED. Combat damage assignment TBA (combat/resolution.rs).

**703.4m** — ALREADY-IMPLEMENTED. Combat damage dealing TBA (combat/resolution.rs).

**703.4n** — TESTABLE. Cleanup discard TBA.

**ATOM-703.4n-001**
- **Rule:** 703.4n — If active player's hand exceeds max hand size, discard to max hand size.
- **Mechanism:** Cleanup step discard TBA
- **Minimal Board:** Active player has 9 cards in hand, max hand size = 7.
- **Action:** Cleanup step begins.
- **Expected Result:** Player is asked to discard 2 cards via DecisionProvider. Hand size becomes 7.
- **Phase:** Pre-Phase 3
- **Ticket:** ALREADY-IMPLEMENTED (game.rs cleanup handling)

**703.4p** — TESTABLE. Cleanup damage removal + "until end of turn"/"this turn" effects end simultaneously.

**ATOM-703.4p-001**
- **Rule:** 703.4p — After discard, all damage removed from permanents and all "until end of turn"/"this turn" effects end simultaneously.
- **Mechanism:** Cleanup step damage/effect removal TBA
- **Minimal Board:** A 3/3 creature with 2 damage marked. An active "until end of turn" +3/+3 effect on another creature.
- **Action:** Cleanup step, after discard.
- **Expected Result:** Damage on first creature removed (damage_taken = 0). The +3/+3 effect expires. Both happen simultaneously.
- **Phase:** Phase 1 (damage removal), Phase 5 (duration expiry)
- **Ticket:** ALREADY-IMPLEMENTED (damage removal in turns.rs); T22 (duration expiry hooks)

**703.4q** — TESTABLE. Mana pool empties at end of each step/phase.

**ATOM-703.4q-001**
- **Rule:** 703.4q — Unspent mana empties from mana pool as each step/phase ends.
- **Mechanism:** Mana pool emptying TBA
- **Minimal Board:** Player has {R}{R}{G} in mana pool at end of main phase.
- **Action:** Main phase ends.
- **Expected Result:** Mana pool is empty (all mana drained).
- **Phase:** Phase 1
- **Ticket:** ALREADY-IMPLEMENTED (mana pool clearing in turns.rs)

### 704. State-Based Actions

**704.1** — PURE-DEF. Defines what SBAs are. Prerequisite for all 704.5x rules.

**704.1a** — PURE-DEF. Distinguishes SBAs from triggered abilities. Prerequisite for Phase 7.

**704.2** — PURE-DEF. SBAs are checked throughout the game and not controlled by any player.

**704.3** — TESTABLE. SBA check-repeat loop: check SBAs → if any performed, repeat; else place waiting triggers, repeat; if none of either, grant priority. Cleanup variant: if no SBAs and no triggers on first check, no priority granted and step ends.

**ATOM-704.3-001**
- **Rule:** 704.3 — SBA check repeats until no more SBAs are performed and no triggers are waiting.
- **Mechanism:** SBA repeat loop
- **Minimal Board:** A 2/2 creature with 2 damage marked, and a 1/1 creature with 1 damage marked. Both have lethal damage.
- **Action:** Priority would be granted (SBA check runs).
- **Expected Result:** Both creatures are destroyed simultaneously in a single SBA check. The check repeats — no more SBAs found, so priority is granted.
- **Phase:** Phase 1
- **Ticket:** ALREADY-IMPLEMENTED (priority.rs SBA loop)

**ATOM-704.3-002**
- **Rule:** 704.3 — Cleanup step: if no SBAs performed and no triggers waiting on first check, no player gets priority and step ends.
- **Mechanism:** Cleanup step SBA shortcut
- **Minimal Board:** Active player has 5 cards in hand (under max), no damage on permanents, no triggered abilities.
- **Action:** Cleanup step begins.
- **Expected Result:** No discard needed. Damage removal is a no-op. SBA check finds nothing. No triggers waiting. Step ends without any player getting priority.
- **Phase:** Phase 5-Pre
- **Ticket:** T16 (cleanup SBA re-loop)

**ATOM-704.3-003**
- **Rule:** 704.3 — Cleanup step: if SBAs ARE performed during cleanup, players get priority (re-loop).
- **Mechanism:** Cleanup SBA re-loop with priority
- **Minimal Board:** Active player has a creature with an "at end of turn, sacrifice this" delayed trigger that resolved during the end step, putting it in the graveyard. A token was also created "until end of turn" and the effect ending causes it to be sacrificed. SBA check during cleanup finds the token in a non-battlefield zone.
- **Action:** Cleanup step begins; damage removed and "until end of turn" effects end; SBA check finds token ceased to exist.
- **Expected Result:** SBAs performed → cleanup re-loops (damage removal + discard again) and players receive priority.
- **Phase:** Phase 5-Pre
- **Ticket:** T16 (cleanup SBA re-loop)

**704.4** — TESTABLE. SBAs don't apply during resolution — only checked when a player would get priority.

**ATOM-704.4-001**
- **Rule:** 704.4 — SBAs pay no attention to what happens during resolution of a spell or ability.
- **Mechanism:** Mid-resolution SBA immunity
- **Minimal Board:** Player controls a creature whose P/T is defined by hand size (CDA, e.g., `*/*` = cards in hand). Player has 3 cards in hand. Spell on stack: "Discard your hand, then draw 3 cards."
- **Action:** Spell resolves: hand goes to 0 (creature is temporarily 0/0), then draws 3 (creature is 3/3 again).
- **Expected Result:** Creature survives. SBAs are not checked mid-resolution. When SBAs are finally checked, creature has toughness 3.
- **Phase:** Phase 5 (CDAs in layer 7a)
- **Ticket:** L04 (Layer 7a CDA)

**704.5a** — ALREADY-IMPLEMENTED. Player at 0 or less life loses.

**704.5b** — ALREADY-IMPLEMENTED. Player who attempted to draw from empty library loses.

**704.5c** — TESTABLE. 10+ poison counters → player loses.

**ATOM-704.5c-001**
- **Rule:** 704.5c — Player with 10 or more poison counters loses the game.
- **Mechanism:** Poison counter SBA
- **Minimal Board:** Player A has 10 poison counters.
- **Action:** SBA check.
- **Expected Result:** Player A loses the game. LossReason::PoisonCounters.
- **Phase:** Phase 5-Pre
- **Ticket:** T16

**ATOM-704.5c-002**
- **Rule:** 704.5c — Player with 9 poison counters does NOT lose.
- **Mechanism:** Poison counter SBA threshold
- **Minimal Board:** Player A has 9 poison counters.
- **Action:** SBA check.
- **Expected Result:** No SBA performed. Player A does not lose.
- **Phase:** Phase 5-Pre
- **Ticket:** T16

**704.5d** — TESTABLE. Token in zone other than battlefield ceases to exist.

**ATOM-704.5d-001**
- **Rule:** 704.5d — Token in a zone other than the battlefield ceases to exist.
- **Mechanism:** Token cease-to-exist SBA
- **Minimal Board:** A token creature in the graveyard (e.g., it died from combat damage).
- **Action:** SBA check.
- **Expected Result:** Token is removed from the game entirely. It does not trigger "dies" again — it ceases to exist. GameEvent::TokenCeasedToExist emitted.
- **Phase:** Phase 5-Pre
- **Ticket:** T13

**704.5e** — TESTABLE. Two clauses: (1) copy of spell not on stack ceases to exist; (2) copy of card not on stack or battlefield ceases to exist.

**ATOM-704.5e-001**
- **Rule:** 704.5e — A copy of a spell in a zone other than the stack ceases to exist.
- **Mechanism:** Spell copy cease-to-exist SBA
- **Minimal Board:** A copy of an instant spell has somehow moved to the graveyard (e.g., countered copy moved to GY instead of ceasing — hypothetical implementation error).
- **Action:** SBA check.
- **Expected Result:** The copy ceases to exist (removed from the game).
- **Phase:** Phase 6 (copy effects)
- **Ticket:** NEW — SBA for spell copies ceasing to exist (D5 copy system)

**ATOM-704.5e-002**
- **Rule:** 704.5e — A copy of a card in a zone other than stack or battlefield ceases to exist.
- **Mechanism:** Card copy cease-to-exist SBA
- **Minimal Board:** An effect says "Choose a creature card in your graveyard. Copy it. You may cast the copy." Player copies a creature card but does not cast the copy. The copy is created in the graveyard zone.
- **Action:** SBA check.
- **Expected Result:** The copy ceases to exist (it's a copy of a card in a zone other than the stack or battlefield). This covers non-token copies created by cast-a-copy effects where the player declines to cast.
- **Phase:** Phase 6
- **Ticket:** NEW — SBA for card copies ceasing to exist

**704.5f** — ALREADY-IMPLEMENTED. Creature with toughness ≤ 0 → owner's graveyard. Regeneration can't replace.

**704.5g** — ALREADY-IMPLEMENTED. Creature with lethal damage (damage ≥ toughness, toughness > 0) → destroyed. Regeneration can replace.

**704.5h** — ALREADY-IMPLEMENTED. Creature dealt deathtouch damage → destroyed. Regeneration can replace.

**704.5i** — TESTABLE. Planeswalker with loyalty 0 → owner's graveyard.

**ATOM-704.5i-001**
- **Rule:** 704.5i — Planeswalker with loyalty 0 is put into its owner's graveyard.
- **Mechanism:** Planeswalker 0-loyalty SBA
- **Minimal Board:** A planeswalker permanent with 0 loyalty counters on the battlefield.
- **Action:** SBA check.
- **Expected Result:** Planeswalker is moved to owner's graveyard.
- **Phase:** Phase 5-Pre
- **Ticket:** T14

**ATOM-704.5i-002**
- **Rule:** 704.5i — Planeswalker with loyalty > 0 is NOT removed.
- **Mechanism:** Planeswalker loyalty SBA (negative case)
- **Minimal Board:** A planeswalker with 3 loyalty counters.
- **Action:** SBA check.
- **Expected Result:** Planeswalker remains on the battlefield.
- **Phase:** Phase 5-Pre
- **Ticket:** T14

**704.5j** — TESTABLE. Legend rule: two+ legendary permanents with same name controlled by same player → controller chooses one, rest go to owners' graveyards.

**ATOM-704.5j-001**
- **Rule:** 704.5j — Two legendary permanents with the same name controlled by the same player: controller keeps one, other goes to graveyard.
- **Mechanism:** Legend rule SBA
- **Minimal Board:** Player A controls two legendary creatures both named "Thalia, Guardian of Thraben."
- **Action:** SBA check.
- **Expected Result:** DecisionProvider::choose_legend_to_keep is called. One is kept, the other is put into its owner's graveyard.
- **Phase:** Phase 5-Pre
- **Ticket:** T14

**ATOM-704.5j-002**
- **Rule:** 704.5j — Two legendary permanents with different names do NOT trigger legend rule.
- **Mechanism:** Legend rule SBA (different names, negative case)
- **Minimal Board:** Player A controls legendary creature "Thalia" and legendary creature "Avacyn."
- **Action:** SBA check.
- **Expected Result:** Both remain on the battlefield. No legend rule SBA fires.
- **Phase:** Phase 5-Pre
- **Ticket:** T14

**ATOM-704.5j-003**
- **Rule:** 704.5j — Same-name legendaries controlled by different players do NOT trigger legend rule.
- **Mechanism:** Legend rule SBA (different controllers, negative case)
- **Minimal Board:** Player A controls "Thalia." Player B controls "Thalia."
- **Action:** SBA check.
- **Expected Result:** Both remain. Legend rule is per-controller.
- **Phase:** Phase 5-Pre
- **Ticket:** T14

**704.5k** — TESTABLE. World rule: two+ world-supertype permanents → keep only the one with shortest world-supertype duration; ties → all destroyed.

**ATOM-704.5k-001**
- **Rule:** 704.5k — Two world permanents: keep the one with the shortest time having world supertype; other goes to graveyard.
- **Mechanism:** World rule SBA
- **Minimal Board:** World enchantment A (entered turn 3) and world enchantment B (entered turn 5) on battlefield.
- **Action:** SBA check.
- **Expected Result:** Enchantment A (older) is put into owner's graveyard. Enchantment B (newer/shorter time as world) survives.
- **Phase:** Phase 8
- **Ticket:** NEW — World rule SBA

**ATOM-704.5k-002**
- **Rule:** 704.5k — Tied world permanents: all go to graveyard.
- **Mechanism:** World rule SBA (tie case)
- **Minimal Board:** Two world enchantments that entered the battlefield simultaneously (same timestamp).
- **Action:** SBA check.
- **Expected Result:** Both are put into their owners' graveyards.
- **Phase:** Phase 8
- **Ticket:** NEW — World rule SBA (tie case)

**704.5m** — TESTABLE. Aura attached to illegal object/player, or not attached → owner's graveyard.

**ATOM-704.5m-001**
- **Rule:** 704.5m — Unattached Aura is put into its owner's graveyard.
- **Mechanism:** Aura legality SBA
- **Minimal Board:** An Aura permanent on the battlefield with attached_to = None.
- **Action:** SBA check.
- **Expected Result:** Aura is moved to owner's graveyard.
- **Phase:** Phase 5-Pre
- **Ticket:** T15

**ATOM-704.5m-002**
- **Rule:** 704.5m — Aura attached to an object that left the battlefield is put into graveyard.
- **Mechanism:** Aura host-left SBA
- **Minimal Board:** Aura was attached to creature X. Creature X is destroyed (leaves battlefield). Aura's attached_to points to a non-existent battlefield permanent.
- **Action:** SBA check.
- **Expected Result:** Aura goes to owner's graveyard.
- **Phase:** Phase 5-Pre
- **Ticket:** T15

**704.5n** — TESTABLE. Equipment or Fortification attached to illegal permanent or player → unattach, stays on battlefield.

**ATOM-704.5n-001**
- **Rule:** 704.5n — Equipment attached to a non-creature (illegal) becomes unattached but remains on the battlefield.
- **Mechanism:** Equipment legality SBA
- **Minimal Board:** Equipment attached to a permanent that is no longer a creature (e.g., type-changing effect ended).
- **Action:** SBA check.
- **Expected Result:** Equipment's attached_to becomes None. Host's attached_by no longer includes Equipment. Equipment stays on battlefield.
- **Phase:** Phase 5-Pre
- **Ticket:** T15

**704.5p** — TESTABLE. Battle/creature attached to something → unattach. Non-Aura, non-Equipment, non-Fortification permanent attached → unattach.

**ATOM-704.5p-001**
- **Rule:** 704.5p — A creature that is attached to an object becomes unattached and remains on the battlefield.
- **Mechanism:** Illegal attachment catch-all SBA
- **Minimal Board:** A creature permanent has attached_to pointing to another permanent (e.g., a type-changing effect turned an Aura into a creature while it was attached).
- **Action:** SBA check.
- **Expected Result:** Creature becomes unattached (attached_to = None). Remains on battlefield.
- **Phase:** Phase 5-Pre
- **Ticket:** T15

**704.5q** — TESTABLE. +1/+1 and -1/-1 counter annihilation.

**ATOM-704.5q-001**
- **Rule:** 704.5q — If a permanent has both +1/+1 and -1/-1 counters, remove N of each where N = min of both counts.
- **Mechanism:** Counter annihilation SBA
- **Minimal Board:** Creature with 3 +1/+1 counters and 2 -1/-1 counters.
- **Action:** SBA check.
- **Expected Result:** 2 of each removed. Creature ends with 1 +1/+1 counter and 0 -1/-1 counters.
- **Phase:** Phase 5-Pre
- **Ticket:** T13

**ATOM-704.5q-002**
- **Rule:** 704.5q — Equal counts of +1/+1 and -1/-1 → both zeroed.
- **Mechanism:** Counter annihilation SBA (equal case)
- **Minimal Board:** Creature with 3 +1/+1 counters and 3 -1/-1 counters.
- **Action:** SBA check.
- **Expected Result:** All counters removed. 0 of each type remains.
- **Phase:** Phase 5-Pre
- **Ticket:** T13

**704.5r** — TESTABLE. Permanent with "can't have more than N counters of a kind" has more → remove excess.

**ATOM-704.5r-001**
- **Rule:** 704.5r — Permanent with ability capping counter count at N has more than N → remove excess.
- **Mechanism:** Counter cap SBA
- **Minimal Board:** A permanent with an ability "can't have more than 3 charge counters on it" and it currently has 5 charge counters.
- **Action:** SBA check.
- **Expected Result:** 2 charge counters removed, leaving exactly 3.
- **Phase:** Phase 8
- **Ticket:** NEW — Counter cap SBA (704.5r)

**704.5s** — DEFERRED — Phase 8–9: Saga sacrifice SBA (lore counters ≥ final chapter number).

**704.5t** — OUT-OF-SCOPE. Dungeon venture marker SBA (Dungeons & Dragons crossover mechanic — not a standard MTG mechanic in the simulator's scope).

**704.5u** — OUT-OF-SCOPE. Space sculptor sector designation SBA (Unfinity mechanic).

**704.5v** — DEFERRED — Phase 8–9: Battle with defense 0 → graveyard SBA.

**704.5w** — DEFERRED — Phase 8–9: Battle with no protector and no attackers → choose protector or graveyard.

**704.5x** — DEFERRED — Phase 8–9: Siege controller = protector → choose new protector or graveyard.

**704.5y** — DEFERRED — Phase 8–9: Multiple Roles from same player → keep most recent, rest to graveyard.

**704.5z** — DEFERRED — Phase 8–9: Speed designation SBA (Accelerate/Start Your Engines — from a Standard-legal set, but niche mechanic deferred until card-type support exists).

**704.6a** — OUT-OF-SCOPE. Two-Headed Giant team life SBA.

**704.6b** — OUT-OF-SCOPE. Two-Headed Giant team poison SBA.

**704.6c** — DEFERRED — Phase 9: Commander. Commander damage 21+ → player loses.

**ATOM-704.6c-001**
- **Rule:** 704.6c — Player dealt 21+ combat damage by the same commander loses.
- **Mechanism:** Commander damage SBA
- **Minimal Board:** Player A has taken 21 combat damage from Player B's commander (tracked in commander_damage_taken).
- **Action:** SBA check.
- **Expected Result:** Player A loses the game. LossReason::CommanderDamage.
- **Phase:** Phase 9 (Commander) / Phase 5-Pre data model (T16)
- **Ticket:** T16 (data model + SBA stub); Phase 9 full implementation

**704.6d** — DEFERRED — Phase 9: Commander. Commander in graveyard/exile → owner may move to command zone.

**704.6e** — OUT-OF-SCOPE. Archenemy scheme SBA.

**704.6f** — OUT-OF-SCOPE. Planechase phenomenon SBA.

**704.7** — TESTABLE. Multiple SBAs with same result at same time → single replacement effect replaces all.

**ATOM-704.7-001**
- **Rule:** 704.7 — If multiple SBAs would have the same result simultaneously, a single replacement effect replaces all of them.
- **Mechanism:** SBA result coalescing for replacement effects
- **Minimal Board:** Player at 0 life AND has drawn from empty library. Player controls Lich's Mirror ("If you would lose the game, instead [shuffle + draw + set life to 20]").
- **Action:** SBA check.
- **Expected Result:** Both 704.5a and 704.5b would cause the player to lose. A single application of Lich's Mirror's replacement effect replaces both losses. The player doesn't lose twice or get two replacement applications.
- **Phase:** Phase 6 (replacement effects)
- **Ticket:** NEW — SBA coalescing for replacement effects (704.7)

**704.8** — TESTABLE. LKI for permanents leaving battlefield via SBA is derived from game state BEFORE any SBAs were performed.

**ATOM-704.8-001**
- **Rule:** 704.8 — LKI of a permanent leaving via SBA is from the game state before any SBAs in that batch.
- **Mechanism:** Pre-SBA LKI snapshot
- **Minimal Board:** A 1/1 creature with undying (returns if it had no +1/+1 counters) and one +1/+1 counter on it. A spell places three -1/-1 counters on it. Before SBAs: creature has 1 +1/+1 and 3 -1/-1 counters. SBAs will: (a) annihilate 1 pair of +1/+1 and -1/-1 (704.5q), and (b) put the 0-toughness creature into graveyard (704.5f).
- **Action:** SBA check performs all simultaneously.
- **Expected Result:** LKI of the creature as it last existed on the battlefield shows it WITH a +1/+1 counter (pre-SBA state). Undying does NOT trigger because LKI shows a +1/+1 counter was present.
- **Phase:** Phase 5 Layers (L18 LKI system)
- **Ticket:** L18 (LKI)

--- End of Chunk 1 ---

## Chunk 2 — 705.x Coin Flipping + 706.x Die Rolling + 707.x Copying Objects

### 705. Flipping a Coin

**705.1** — TESTABLE. Coin flip produces one of two equally likely outcomes (heads/tails).

**ATOM-705.1-001**
- **Rule:** 705.1 — A coin flip must produce one of two equally likely outcomes: heads or tails.
- **Mechanism:** Coin flip RNG
- **Minimal Board:** A spell instructs Player A to flip a coin.
- **Action:** Coin flip is performed.
- **Expected Result:** Engine's RNG returns one of exactly two outcomes. The result is stored and communicated to effects that care about the result.
- **Phase:** Phase 8
- **Ticket:** NEW — Coin flip infrastructure

**705.2** — TESTABLE. Two clauses: (1) heads-or-tails-only effects — no player wins/loses; (2) call-based flips — caller wins if call matches, loses otherwise. Only the flipping player wins or loses.

**ATOM-705.2-001**
- **Rule:** 705.2 — Call-based coin flip: player calls heads or tails; if call matches, player wins the flip.
- **Mechanism:** Coin flip win/lose determination
- **Minimal Board:** Player A is instructed to flip a coin and call it. Player A calls heads.
- **Action:** Coin flip. Result is heads.
- **Expected Result:** Player A wins the flip. No other player wins or loses the flip.
- **Phase:** Phase 8
- **Ticket:** NEW — Coin flip win/lose tracking

**ATOM-705.2-002**
- **Rule:** 705.2 — Call-based coin flip: player calls wrong → player loses the flip.
- **Mechanism:** Coin flip lose path
- **Minimal Board:** Player A calls tails. Coin lands heads.
- **Action:** Coin flip resolves.
- **Expected Result:** Player A loses the flip.
- **Phase:** Phase 8
- **Ticket:** NEW — Coin flip win/lose tracking

**705.3** — TESTABLE. Effect states a coin flip has a certain result → override actual result.

**ATOM-705.3-001**
- **Rule:** 705.3 — An effect may override the actual coin flip result with a predetermined outcome.
- **Mechanism:** Coin flip result override
- **Minimal Board:** Player controls Krark's Thumb ("If you would flip a coin, instead flip two coins and ignore one"). Actually test: an effect states "you win the flip."
- **Action:** Coin flip is performed.
- **Expected Result:** Regardless of actual RNG result, the indicated result is used. Player wins the flip if the effect says so.
- **Phase:** Phase 8
- **Ticket:** NEW — Coin flip result override (705.3)

### 706. Rolling a Die

**706.1** — PURE-DEF. An effect specifies what kind of die and how many.

**706.1a** — PURE-DEF. Defines N-sided die / dN notation.

**706.1b** — PURE-DEF. Alternate methods allowed with same number of equally likely outcomes.

**706.2** — TESTABLE. Natural result (before modifiers) vs. final result (after modifiers).

**ATOM-706.2-001**
- **Rule:** 706.2 — Natural result is the face value; final result includes modifiers.
- **Mechanism:** Die roll modifier system
- **Minimal Board:** Player rolls a d20 (natural result = 14). A modifier effect says "add 2 to the result."
- **Action:** Die roll with modifier.
- **Expected Result:** Natural result = 14. Final result = 16. Effects use the final result.
- **Phase:** Phase 8
- **Ticket:** NEW — Die roll infrastructure with modifier support

**706.2a** — PURE-DEF. Modifiers may be optional / have costs. Mana abilities can be activated before applying.

**706.2b** — TESTABLE. Multiple modifier effects → player chooses order: rerolls first, then +/- modifiers.

**ATOM-706.2b-001**
- **Rule:** 706.2b — Two+ modifier effects: player chooses. Reroll modifiers considered first, then arithmetic modifiers.
- **Mechanism:** Die roll modifier ordering
- **Minimal Board:** Player rolls d20. Two effects: one allows reroll, one adds +2. Natural result = 3.
- **Action:** Player chooses to apply reroll first (gets 15), then +2 modifier.
- **Expected Result:** Final result = 17. Reroll was applied in the first step, arithmetic in the second.
- **Phase:** Phase 8
- **Ticket:** NEW — Die roll modifier ordering (706.2b)

**706.3** — PURE-DEF. Results tables exist on some die-rolling abilities.

**706.3a** — TESTABLE. Results table lookup: single number, range "N1–N2", or "N+" determines which effect happens.

**ATOM-706.3a-001**
- **Rule:** 706.3a — After a die roll, the final result is matched against the results table to determine the effect.
- **Mechanism:** Results table lookup
- **Minimal Board:** Player activates ability with a d20 results table: "1–9: nothing; 10–19: draw a card; 20: draw two cards." Player rolls 15.
- **Action:** Die roll resolves, result matched to table.
- **Expected Result:** Result 15 falls in range 10–19 → "draw a card" effect executes.
- **Phase:** Phase 8
- **Ticket:** NEW — Results table resolution (706.3a)

**706.3b** — PURE-DEF. Roll instruction + modifiers + results table are all one ability.

**706.3c** — TESTABLE. "Roll again" in results table uses same die kind/count and modifiers.

**ATOM-706.3c-001**
- **Rule:** 706.3c — "Roll again" uses the same kind and number of dice with applicable modifiers.
- **Mechanism:** Recursive die roll
- **Minimal Board:** d20 results table has "20: [effect], then roll again." Player rolls 20, then re-rolls.
- **Action:** First roll = 20 → effect applies, then a second roll of a d20 is made.
- **Expected Result:** Second roll uses same d20 with same modifiers. Its result is also checked against the table.
- **Phase:** Phase 8
- **Ticket:** NEW — "Roll again" die mechanic (706.3c)

**706.4** — PURE-DEF. Some die-roll abilities have no results table; text describes how to use the result.

**706.5** — OUT-OF-SCOPE. Single-card rule (Celebr-8000 doubles). Card-specific.

**706.6** — TESTABLE. Ignored roll: considered never happened; no triggers, no effects.

**ATOM-706.6-001**
- **Rule:** 706.6 — An ignored roll is considered to have never happened.
- **Mechanism:** Die roll ignore
- **Minimal Board:** Player rolls two d20s. An effect says "ignore the lowest roll." Results: 7 and 15.
- **Action:** Lowest roll (7) is ignored.
- **Expected Result:** Only the 15 is used. No abilities trigger from the 7. If tied for lowest, player chooses which to ignore.
- **Phase:** Phase 8
- **Ticket:** NEW — Die roll ignore mechanic (706.6)

**706.7** — OUT-OF-SCOPE. Planechase planar die interaction.

**706.8** — OUT-OF-SCOPE. Single-card rule (Centaur of Attention stored results). Card-specific.

**706.8a** — OUT-OF-SCOPE. Stored result definition — card-specific.

**706.8b** — OUT-OF-SCOPE. Reroll stored results — card-specific.

**706.8c** — OUT-OF-SCOPE. Linked abilities for stored results — card-specific.

### 707. Copying Objects

**707.1** — PURE-DEF. Introduces copy effects. No independent mechanical consequence.

**707.2** — TESTABLE. Core copy rule: copy acquires copiable values (printed text + other copy effects + face-down status + "as enters" P/T setters). Does NOT copy other effects, status, counters, or stickers.

**ATOM-707.2-001**
- **Rule:** 707.2 — Copy acquires copiable values only. Type-changing effects, counters, and status are not copied.
- **Mechanism:** Copiable values determination
- **Minimal Board:** Chimeric Staff (artifact) has been animated to a 5/5 creature via its own ability. Clone enters as a copy of Staff.
- **Action:** Clone enters the battlefield.
- **Expected Result:** Clone is an artifact (not a creature), matching Staff's printed characteristics. The animation effect is not copiable. Clone has Staff's activated ability.
- **Phase:** Phase 6 (Layer 1 copy effects)
- **Ticket:** D5 (copy system)

**ATOM-707.2-002**
- **Rule:** 707.2 — Copy of a face-down creature gets the face-down characteristics (2/2, no name, no types, no abilities, no mana cost).
- **Mechanism:** Face-down copiable values
- **Minimal Board:** Clone enters as a copy of a face-down morph creature.
- **Action:** Clone enters the battlefield.
- **Expected Result:** Clone is a 2/2 colorless creature with no name, no types, no abilities, no mana cost. Clone is face UP (it copies the face-down characteristics but isn't itself face down).
- **Phase:** Phase 9 (face-down + copy interaction)
- **Ticket:** D5 + Phase 9

**ATOM-707.2-003**
- **Rule:** 707.2 — Counters on the original are NOT copied.
- **Mechanism:** Counter non-copying
- **Minimal Board:** Original creature has 3 +1/+1 counters. Clone enters as a copy.
- **Action:** Clone enters.
- **Expected Result:** Clone has 0 +1/+1 counters. Its P/T matches the printed P/T of the original, not the boosted values.
- **Phase:** Phase 6
- **Ticket:** D5

**707.2a** — PURE-DEF. Clarifies color comes from mana cost/color indicator; abilities come from rules text. No double-counting.

**707.2b** — TESTABLE. Changing copiable values of original after copy is made does not update the copy.

**ATOM-707.2b-001**
- **Rule:** 707.2b — Changing original's copiable values after copy won't change the copy.
- **Mechanism:** Copy independence from original
- **Minimal Board:** Clone copied Grizzly Bears (2/2). Later, Grizzly Bears becomes a copy of something else.
- **Action:** Original changes copiable values.
- **Expected Result:** Clone remains a 2/2 Grizzly Bears. No retroactive update.
- **Phase:** Phase 6
- **Ticket:** D5

**707.2c** — BOUNDARY-DEF. Static ability copy effect: copiable values locked at time effect first applies.

**BOUNDARY-707.2c-001**
- **Rule:** 707.2c — A continuous copy effect from a static ability locks copiable values at the time the effect first applies.
- **Boundary:** Defines when copiable values are "frozen" for static-ability-driven copy effects.
- **Engine Constraint:** The copy system must snapshot copiable values at effect application time, not continuously re-derive them.
- **Phase:** Phase 6
- **Ticket:** D5 (copy system timestamp)

**707.3** — TESTABLE. Copy's copiable values become the "new" copiable values, modified by copy's own status. Further copies use these new values.

**ATOM-707.3-001**
- **Rule:** 707.3 — A copy's copiable values include the copied info as modified by the copy's status (e.g., face-down, flipped). Further copies use these new copiable values.
- **Mechanism:** Layered copy copiable value propagation
- **Minimal Board:** Vesuvan Doppelganger entered as a copy of Runeclaw Bear. Clone enters as a copy of the Doppelganger.
- **Action:** Clone enters.
- **Expected Result:** Clone has Runeclaw Bear's characteristics PLUS the Doppelganger's retained upkeep ability. Clone is blue (Doppelganger's color exception). The copiable values chain correctly.
- **Phase:** Phase 6
- **Ticket:** D5

**707.4** — TESTABLE. Re-copy while on battlefield: no ETB/LTB triggers; noncopy effects remain.

**ATOM-707.4-001**
- **Rule:** 707.4 — A permanent changing what it copies doesn't trigger ETB/LTB. Noncopy effects still apply.
- **Mechanism:** Mid-battlefield re-copy
- **Minimal Board:** Unstable Shapeshifter on battlefield, affected by Giant Growth (+3/+3 until EOT). A new creature enters.
- **Action:** Shapeshifter becomes a copy of the new creature.
- **Expected Result:** Shapeshifter has the new creature's copiable values. Still gets +3/+3 from Giant Growth. No ETB or LTB triggers fire for the Shapeshifter.
- **Phase:** Phase 6
- **Ticket:** D5

**707.5** — TESTABLE. "Enters as a copy" — becomes a copy AS it enters. Copied ETB replacement effects apply. Copied ETB triggers fire.

**ATOM-707.5-001**
- **Rule:** 707.5 — Object entering "as a copy" becomes a copy during the enter event. Copied "enters with" abilities apply.
- **Mechanism:** Copy-on-enter with replacement effects
- **Minimal Board:** Clone enters as a copy of Skyshroud Behemoth (enters tapped, fading 2 = enters with 2 fade counters).
- **Action:** Clone enters.
- **Expected Result:** Clone enters tapped with 2 fade counters. It has the Behemoth's characteristics.
- **Phase:** Phase 6
- **Ticket:** D5

**ATOM-707.5-002**
- **Rule:** 707.5 — Copied ETB triggered abilities fire for the copy.
- **Mechanism:** Copy-on-enter with ETB triggers
- **Minimal Board:** Clone enters as a copy of Wall of Omens ("When this enters, draw a card").
- **Action:** Clone enters.
- **Expected Result:** Clone's controller draws a card (the copied ETB trigger fires).
- **Phase:** Phase 6 + Phase 7 (triggers)
- **Ticket:** D5 + Phase 7

**707.6** — TESTABLE. Choices made for the original permanent are not copied. Copy's controller makes new "as enters" choices.

**ATOM-707.6-001**
- **Rule:** 707.6 — Choices made for original aren't copied. Copy's controller makes new choices.
- **Mechanism:** Copy "as enters" choice reset
- **Minimal Board:** Adaptive Automaton (chose "Elf" as its creature type) on battlefield. Clone enters as a copy.
- **Action:** Clone enters.
- **Expected Result:** Clone's controller is prompted to choose a creature type. The original's choice of "Elf" is NOT inherited.
- **Phase:** Phase 6
- **Ticket:** D5

**707.7** — BOUNDARY-DEF. Copied linked abilities remain linked on the copy. Cannot link to other abilities.

**BOUNDARY-707.7-001**
- **Rule:** 707.7 — Linked abilities on the original remain linked on the copy. They can't link to unrelated abilities.
- **Boundary:** Copy system must preserve ability linkage metadata.
- **Engine Constraint:** When copying linked abilities, the copy's linked pair references each other, not the original's abilities and not any other abilities the copy may have.
- **Phase:** Phase 7 (linked abilities)
- **Ticket:** T07-linked (linked abilities system)

**707.8** — DEFERRED — Phase 9: Copying DFC/melded permanent uses face-up copiable values.

**707.8a** — DEFERRED — Phase 9: Token copy of DFC creates double-faced token with both faces.

**707.9** — PURE-DEF. Copy effects may include modifications/exceptions.

**707.9a** — TESTABLE. Copy effect grants additional ability → becomes part of copiable values.

**ATOM-707.9a-001**
- **Rule:** 707.9a — Copy effect that grants an ability makes that ability part of the copy's copiable values.
- **Mechanism:** Copy-with-added-ability copiable value propagation
- **Minimal Board:** Unstable Shapeshifter ("except it has this ability") copies Quirion Elves. Then Clone copies the Shapeshifter.
- **Action:** Clone enters.
- **Expected Result:** Clone has Quirion Elves' characteristics PLUS the Shapeshifter's self-copy ability. The added ability is part of the copiable values.
- **Phase:** Phase 6
- **Ticket:** D5

**707.9b** — TESTABLE. Copy effect modifies a characteristic → modified value becomes copiable.

**ATOM-707.9b-001**
- **Rule:** 707.9b — A copy effect that modifies a characteristic makes the modified value part of copiable values.
- **Mechanism:** Copy-with-modified-characteristic
- **Minimal Board:** Copy Artifact enters as a copy of Juggernaut, "except it's also an enchantment."
- **Action:** Copy Artifact enters.
- **Expected Result:** Copy Artifact's copiable types are artifact, creature, AND enchantment. Further copies of this object would also be all three types.
- **Phase:** Phase 6
- **Ticket:** D5

**707.9c** — PURE-DEF. Some copy effects don't copy certain characteristics; objects retain originals.

**707.9d** — TESTABLE. When a copy effect overrides a characteristic, CDAs defining that characteristic are not copied. Exception: "in addition to its other types" preserves type CDAs.

**ATOM-707.9d-001**
- **Rule:** 707.9d — Copy effect that overrides P/T strips the CDA that defines P/T on the original.
- **Mechanism:** CDA stripping on copy with P/T override
- **Minimal Board:** Quicksilver Gargantuan ("enters as a copy, except it's 7/7") copies Tarmogoyf (CDA: `*/*` based on card types in graveyards).
- **Action:** Gargantuan enters.
- **Expected Result:** Gargantuan is 7/7. It does NOT have Tarmogoyf's CDA. Its P/T is fixed at 7/7.
- **Phase:** Phase 6
- **Ticket:** D5

**ATOM-707.9d-002**
- **Rule:** 707.9d — "In addition to its other types" exception: CDAs for types ARE copied.
- **Mechanism:** CDA preservation for "in addition to types" copy
- **Minimal Board:** Glasspool Mimic ("copy, except it's a Shapeshifter Rogue in addition to its other types") copies a creature with changeling (CDA for all creature types).
- **Action:** Mimic enters.
- **Expected Result:** Mimic has changeling and all creature types. The CDA for subtype is preserved because the exception uses "in addition to."
- **Phase:** Phase 6
- **Ticket:** D5

**707.9e** — TESTABLE. Copy effect with exception that's an additional effect (not characteristic modification): if another copy effect applies after, the exception doesn't happen.

**ATOM-707.9e-001**
- **Rule:** 707.9e — Exception that is an additional effect (e.g., "enters with X counters") is lost if a subsequent copy effect applies.
- **Mechanism:** Additional-effect exception invalidation on re-copy
- **Minimal Board:** Altered Ego ("copy, except enters with X +1/+1 counters") copies Clone (which itself copies nothing). Altered Ego then applies Clone's replacement to copy a real creature.
- **Action:** Altered Ego enters.
- **Expected Result:** Altered Ego becomes a copy of the chosen creature but does NOT enter with +1/+1 counters. The exception was from the first copy effect and is invalidated by the second.
- **Phase:** Phase 6
- **Ticket:** D5

**707.9f** — TESTABLE. Conditional exceptions ("if it's a creature"): evaluate as if the copy effect applied without that exception.

**ATOM-707.9f-001**
- **Rule:** 707.9f — Conditional copy exception: evaluate the would-be result without that exception to determine if it applies.
- **Mechanism:** Conditional exception evaluation (negative case)
- **Minimal Board:** Moritte of the Frost ("copy, except legendary+snow; if creature, +2 counters and changeling") copies a land that's temporarily a creature.
- **Action:** Moritte enters.
- **Expected Result:** Moritte enters as a noncreature land (temporary creature effect is not copiable). The "if it's a creature" conditional does NOT apply. No +1/+1 counters, no changeling. It IS legendary and snow.
- **Phase:** Phase 6
- **Ticket:** D5

**ATOM-707.9f-002**
- **Rule:** 707.9f — Conditional copy exception: positive case — copy IS a creature, so conditional exception applies.
- **Mechanism:** Conditional exception evaluation (positive case)
- **Minimal Board:** Moritte of the Frost copies a creature (e.g., Grizzly Bears 2/2).
- **Action:** Moritte enters.
- **Expected Result:** Moritte enters as a copy of Grizzly Bears. It IS a creature, so the conditional applies: enters with two additional +1/+1 counters (making it 4/4), has changeling (all creature types), and is legendary and snow.
- **Phase:** Phase 6
- **Ticket:** D5

**707.9g** — BOUNDARY-DEF. Copy effect with linked triggered ability in same paragraph: if another copy effect overrides, the linked trigger doesn't fire.

**BOUNDARY-707.9g-001**
- **Rule:** 707.9g — Linked triggered ability in same paragraph as copy effect is invalidated if another copy effect applies afterward.
- **Boundary:** Copy system must track whether a copy effect's linked trigger is still valid after subsequent copy applications.
- **Engine Constraint:** Flag on copy-effect metadata indicating whether its linked trigger is still active.
- **Phase:** Phase 7 (triggers + copy interaction)
- **Ticket:** D5 + Phase 7

**707.10** — TESTABLE. Copying a spell/ability: put copy on stack. Copy inherits characteristics + all decisions (modes, targets, X, costs). Resolution choices NOT copied. Copy of spell is a spell; copy of ability is an ability. Not "cast" or "activated."

**ATOM-707.10-001**
- **Rule:** 707.10 — Copy of a spell on the stack inherits modes, targets, and X value. It is itself a spell but was not "cast."
- **Mechanism:** Spell copy on stack
- **Minimal Board:** Player casts Fork targeting Lightning Bolt (targeting opponent). Fork resolves.
- **Action:** Copy of Lightning Bolt is put on the stack.
- **Expected Result:** Copy has the same target (opponent) and same mode. Copy is a spell on the stack. "Whenever you cast" triggers do NOT fire (copy is not cast). Copy can be countered.
- **Phase:** Phase 6
- **Ticket:** D5 (spell copy system)

**ATOM-707.10-002**
- **Rule:** 707.10 — Copy of a spell references original's cost-payment objects (sacrifice is an object).
- **Mechanism:** Cost-object reference on spell copy
- **Minimal Board:** Fling ("As an additional cost to cast this spell, sacrifice a creature" + "Fling deals damage equal to the sacrificed creature's power to any target") was cast sacrificing a 4/4. Copy of Fling is created.
- **Action:** Copy resolves.
- **Expected Result:** Copy deals 4 damage. It references the creature sacrificed to pay for the original Fling, because the sacrificed creature is an object.
- **Phase:** Phase 6
- **Ticket:** D5

**ATOM-707.10-003**
- **Rule:** 707.10 — Mana is not an object, so a copy of a spell that references mana spent to cast it gets nothing.
- **Mechanism:** Mana-is-not-an-object on spell copy
- **Minimal Board:** Dawnglow Infusion ("You gain X life if {G} was spent to cast this spell and X life if {W} was spent to cast it") was cast with X=5, spending {G} and {W}. A copy of Dawnglow Infusion is created.
- **Action:** Copy resolves.
- **Expected Result:** Copy causes the player to gain 0 life. Mana is not an object, so the copy can't reference what mana was spent to cast the original.
- **Phase:** Phase 6
- **Ticket:** D5

**707.10a** — PURE-DEF. Restates 704.5e (copy of spell not on stack ceases to exist). Already covered under SBAs.

**707.10b** — TESTABLE. Copy of an ability has same source as original. Self-name references point to same object.

**ATOM-707.10b-001**
- **Rule:** 707.10b — A copy of an ability has the same source as the original. Name references in the copy refer to the same object.
- **Mechanism:** Ability copy source tracking
- **Minimal Board:** Creature with ability "Whenever ~ deals damage, put a +1/+1 counter on ~." Ability is copied.
- **Action:** Copy of the ability resolves.
- **Expected Result:** The counter is placed on the original source creature (same object), not on some other object with the same name.
- **Phase:** Phase 7 (triggers)
- **Ticket:** D5 + Phase 7

**707.10c** — TESTABLE. Copy spell/ability with "choose new targets": may leave illegal targets unchanged; new targets must be legal.

**ATOM-707.10c-001**
- **Rule:** 707.10c — When choosing new targets for a copy, unchanged targets may remain illegal. Changed targets must be legal.
- **Mechanism:** Retargeting copied spell
- **Minimal Board:** Fork copies a spell with two targets: one legal, one now illegal. Controller chooses new targets.
- **Action:** Controller leaves illegal target unchanged, changes the legal one to a new legal target.
- **Expected Result:** Allowed. The copy is placed on the stack. The illegal target remains (it will cause that part to fail on resolution if still illegal).
- **Phase:** Phase 6
- **Ticket:** D5

**707.10d** — TESTABLE. Copy "for each target it could target" — one copy per legal target; multi-target spells require same player/object for all.

**ATOM-707.10d-001**
- **Rule:** 707.10d — Copy for each possible target: one copy per legal target, each targeting that player/object.
- **Mechanism:** For-each-target copy multiplication
- **Minimal Board:** Spell with one target. Effect: "copy for each creature it could target." 3 legal creature targets on battlefield.
- **Action:** Effect resolves.
- **Expected Result:** 3 copies created, each targeting a different creature. Placed on stack in controller's chosen order.
- **Phase:** Phase 6
- **Ticket:** D5

**707.10e** — TESTABLE. Copy specifying a new target: if the spell has more than one target, each must be that object. If illegal for any, no copy. Replacement effects causing multiple targets → controller picks one.

**ATOM-707.10e-001**
- **Rule:** 707.10e — Copy with a specified new single target: the copy targets that object.
- **Mechanism:** Forced single-target retarget on copy
- **Minimal Board:** Moment of Triumph ("Target creature gets +2/+2 and you gain 2 life") targeting a creature. Frontline Heroism triggers: "create a 1/1 Soldier token, then copy that spell. The copy targets that token."
- **Action:** Copy created targeting the new Soldier token.
- **Expected Result:** Copy of Moment of Triumph targets the Soldier token. The Soldier gets +2/+2 and controller gains 2 life.
- **Phase:** Phase 6
- **Ticket:** D5

**ATOM-707.10e-002**
- **Rule:** 707.10e — If a replacement effect causes the copy to target more than one object, the controller chooses one.
- **Mechanism:** Replacement-caused multi-target resolution
- **Minimal Board:** Same as 707.10e-001 but Anointed Procession ("create twice that many tokens") is also in play. Two Soldier tokens are created. Copy effect says "the copy targets that token."
- **Action:** Copy created.
- **Expected Result:** Controller chooses ONE of the two tokens as the copy's target. The copy does not target both.
- **Phase:** Phase 6
- **Ticket:** D5

**707.10f** — DEFERRED — Phase 6+: Copy of permanent spell → resolves as token permanent.

**707.10g** — DEFERRED — Phase 9: Copy of DFC permanent spell → double-faced token.

**707.11** — PURE-DEF. Effect tracking a permanent by name continues tracking even after name change or becoming a copy. In the engine, effects always reference ObjectId rather than card names in rules text, so this is inherently satisfied by the architecture. No separate test needed.

**707.12** — TESTABLE. "Cast a copy" follows casting rules (601.2a–h). Copy is created in the same zone, then cast. Once cast, it's a spell that can resolve or be countered.

**ATOM-707.12-001**
- **Rule:** 707.12 — Casting a copy follows normal casting steps. The copy IS cast (triggers "whenever you cast" abilities).
- **Mechanism:** Cast-a-copy pipeline
- **Minimal Board:** Effect says "you may cast a copy of [card in exile]." Player chooses to cast it.
- **Action:** Copy is created in exile, then cast following 601.2a–h.
- **Expected Result:** Copy goes on stack as a spell. "Whenever you cast a spell" triggers fire. It can be countered or resolve normally.
- **Phase:** Phase 6
- **Ticket:** D5

**707.13** — DEFERRED — Phase 9: Single-card rule (Garth One-Eye). Creates a copy of a card defined by name using Oracle reference. Requires copy-from-Oracle infrastructure.

**707.14** — DEFERRED — Phase 9: Single-card rule (Magar of the Magic Strings). Creates a copy of a card with a noted name using LKI from graveyard. Requires LKI + copy interaction.

--- End of Chunk 2 ---

## Chunk 3 — 708.x Face-Down + 709.x Split + 710.x Flip + 711.x Leveler + 712.x DFCs

### 708. Face-Down Spells and Permanents

**708.1** — PURE-DEF. Some cards allow face-down spells/permanents.

**708.2** — TESTABLE. Face-down objects have no characteristics other than those listed by the enabling ability/rule. Listed characteristics are copiable values.

**ATOM-708.2-001**
- **Rule:** 708.2 — Face-down objects have only the characteristics listed by the rule/ability that allowed them to be face down.
- **Mechanism:** Face-down characteristic suppression
- **Minimal Board:** A creature with morph is cast face down.
- **Action:** Face-down creature is on the battlefield.
- **Expected Result:** Object's characteristics are only those specified by the morph rule (2/2, no name, no types beyond creature, no abilities, no mana cost). Original printed characteristics are hidden.
- **Phase:** Phase 9 (morph/face-down)
- **Ticket:** NEW — Face-down permanent system

**708.2a** — TESTABLE. Face-up permanent turned face down by a spell/ability with no listed characteristics → 2/2 creature, no text/name/subtypes/mana cost.

**ATOM-708.2a-001**
- **Rule:** 708.2a — A face-up permanent turned face down with no listed characteristics becomes a 2/2 creature with no text, name, subtypes, or mana cost.
- **Mechanism:** Default face-down characteristics
- **Minimal Board:** Ixidron turns all other creatures face down (no listed characteristics).
- **Action:** A 5/5 Dragon with flying is turned face down.
- **Expected Result:** It becomes a 2/2 creature with no name, no subtypes, no abilities, no mana cost. These are its copiable values.
- **Phase:** Phase 9
- **Ticket:** NEW — Face-down default characteristics

**708.2b** — TESTABLE. Face-down permanent can't be turned face down again. Attempt does nothing.

**ATOM-708.2b-001**
- **Rule:** 708.2b — A face-down permanent can't be turned face down. Attempting to do so is a no-op.
- **Mechanism:** Double face-down prevention
- **Minimal Board:** A face-down morph creature on the battlefield.
- **Action:** Effect tries to turn it face down.
- **Expected Result:** Nothing happens. Characteristics and copiable values unchanged.
- **Phase:** Phase 9
- **Ticket:** NEW — Face-down idempotency guard

**708.3** — TESTABLE. Objects put onto battlefield face down are turned face down BEFORE entering → ETB abilities don't trigger.

**ATOM-708.3-001**
- **Rule:** 708.3 — Face-down enter: object is face down before it enters, so ETB triggers of its printed face don't fire.
- **Mechanism:** Pre-enter face-down suppression
- **Minimal Board:** A creature with "When this enters, draw a card" is put onto the battlefield face down.
- **Action:** Creature enters face down.
- **Expected Result:** No "draw a card" trigger fires. The permanent is a 2/2 with no abilities.
- **Phase:** Phase 9
- **Ticket:** NEW — Face-down ETB suppression

**708.4** — TESTABLE. Objects cast face down are turned face down BEFORE put on stack → casting restrictions/effects see only face-down characteristics.

**ATOM-708.4-001**
- **Rule:** 708.4 — Casting face down: the object is face down before going on the stack. Effects see a 2/2 with no characteristics.
- **Mechanism:** Face-down casting characteristics
- **Minimal Board:** Player casts a creature with morph face down. An effect says "whenever you cast a creature spell with CMC 3+, draw a card."
- **Action:** Morph creature is cast face down for {3}.
- **Expected Result:** The spell on the stack has no mana cost (CMC = 0). The "CMC 3+" trigger does NOT fire. The permanent it becomes will be face down.
- **Phase:** Phase 9
- **Ticket:** NEW — Face-down casting pipeline

**708.5** — PURE-DEF. Controller may look at their own face-down spells on the stack or face-down permanents they control at any time. Can't look at opponent's. **Note (110.5d interaction):** This applies specifically to face-down cards being cast face-down on the stack and face-down permanents on the battlefield. Cards exiled face-down by other abilities (e.g., Gonti, Lord of Luxury) are NOT covered by this rule — those are governed by the specific ability that exiled them. The "face down" nomenclature in 708.5 is specifically about morph/manifest/disguise-style face-down casting, not general face-down exile.

**708.6** — PURE-DEF. Must differentiate face-down objects from each other. Implementation concern (UI/tracking), not a game-mechanics test.

**708.7** — PURE-DEF. Enabling ability/rule may allow controller to turn face-down permanent face up. Spells normally can't.

**708.8** — TESTABLE. Turning face up: copiable values revert to normal. Existing effects still apply. No ETB triggers (already entered).

**ATOM-708.8-001**
- **Rule:** 708.8 — Turning face up reverts copiable values. Existing effects still apply. No ETB triggers.
- **Mechanism:** Face-up revert
- **Minimal Board:** A face-down morph creature (2/2) with Giant Growth (+3/+3) on it. Controller pays morph cost.
- **Action:** Creature is turned face up (e.g., a 4/4 Angel with flying).
- **Expected Result:** Now a 4/4 Angel with flying + still gets +3/+3 = 7/7. ETB abilities of the Angel do NOT trigger. The permanent's characteristics are those of its printed face.
- **Phase:** Phase 9
- **Ticket:** NEW — Face-up morph revert

**708.9** — PURE-DEF. Face-down permanent leaving battlefield → owner must reveal it to all players. **Audit note:** This rule exists primarily as an anti-cheating measure (proving you legally had the face-down card). There are no current cards that mechanically care about a face-down permanent being *revealed* on zone change (as opposed to being *turned face up* on the battlefield, which is a different event). A GameEvent::FaceDownRevealed primitive is likely unnecessary and could muddle the data model. The engine can simply make the card's identity public knowledge when it moves zones — this happens naturally since the card is face-up in the graveyard/hand/exile. No separate event primitive needed unless a future card mechanically triggers on "whenever a face-down permanent is revealed."

**708.10** — TESTABLE. Face-down permanent copies another → copiable values are the copy's, modified by face-down status. Characteristics stay face-down. Turned face up → shows copied characteristics.

**ATOM-708.10-001**
- **Rule:** 708.10 — Face-down permanent becomes a copy: still has face-down characteristics on battlefield. If turned face up, shows the copied permanent's characteristics.
- **Mechanism:** Face-down copy interaction
- **Minimal Board:** Face-down morph creature becomes a copy of Branchsnap Lorian (4/1 with trample and morph {G}).
- **Action:** Copy effect applies, then creature is turned face up.
- **Expected Result:** While face down: still 2/2, no abilities. When turned face up (paying {G}): becomes Branchsnap Lorian (4/1, trample, morph {G}).
- **Phase:** Phase 9 + Phase 6 (copy)
- **Ticket:** D5 + Phase 9

**708.11** — TESTABLE. "As this is turned face up" ability applied DURING the face-up process, not afterward.

**ATOM-708.11-001**
- **Rule:** 708.11 — "As [this] is turned face up" ability is applied during the face-up transition.
- **Mechanism:** Face-up replacement timing
- **Minimal Board:** Face-down permanent with "As this is turned face up, choose a color."
- **Action:** Permanent is turned face up.
- **Expected Result:** Choice is made during the face-up process. By the time the permanent is fully face up, the choice has already been made.
- **Phase:** Phase 9
- **Ticket:** NEW — Face-up "as" replacement timing

**708.12** — BOUNDARY-DEF. Revealing a face-down permanent for information uses characteristics ignoring continuous effects.

**BOUNDARY-708.12-001**
- **Rule:** 708.12 — When revealing face-down permanent for information, use characteristics ignoring continuous effects.
- **Boundary:** The reveal inspection must bypass the layer system and use base characteristics.
- **Engine Constraint:** A "reveal for info" function must read from the card's base data, not from the computed continuous-effect-applied characteristics.
- **Phase:** Phase 9
- **Ticket:** NEW — Face-down reveal bypasses layers

### 709. Split Cards

**709.1** — PURE-DEF. Split cards have two card faces on one card. Normal Magic back.

**709.2** — PURE-DEF. Split card is one card, not two.

**709.3** — TESTABLE. Player chooses which half to cast before putting on stack.

**ATOM-709.3-001**
- **Rule:** 709.3 — Player chooses which half of a split card to cast before putting it onto the stack.
- **Mechanism:** Split card half selection
- **Minimal Board:** Player has Fire//Ice in hand during main phase.
- **Action:** Player casts the spell, choosing "Fire."
- **Expected Result:** DecisionProvider is called to choose which half. "Fire" half is placed on the stack. The spell on the stack has only Fire's characteristics.
- **Phase:** Phase 9
- **Ticket:** NEW — Split card casting pipeline

**709.3a** — TESTABLE. Only chosen half evaluated for castability; only that half on stack.

**ATOM-709.3a-001**
- **Rule:** 709.3a — Only the chosen half is checked for legality and put onto the stack.
- **Mechanism:** Half-only evaluation
- **Minimal Board:** Player wants to cast "Fire" half. An effect says "you can't cast blue spells." Ice is blue but Fire is red.
- **Action:** Player attempts to cast Fire.
- **Expected Result:** Legal. Fire is red, not blue. The prohibition on blue spells doesn't prevent casting Fire even though the other half (Ice) is blue.
- **Phase:** Phase 9
- **Ticket:** NEW — Split card half-only evaluation

**709.3b** — TESTABLE. On stack, only the cast half's characteristics exist.

**ATOM-709.3b-001**
- **Rule:** 709.3b — While on the stack, only the characteristics of the cast half exist. The other half is treated as nonexistent.
- **Mechanism:** Stack characteristic suppression for non-cast half
- **Minimal Board:** Fire//Ice on stack, "Fire" half cast. An effect asks "is this spell blue?"
- **Action:** Check characteristics of spell on stack.
- **Expected Result:** Spell is red (Fire), not blue. Ice's characteristics don't exist while on the stack.
- **Phase:** Phase 9
- **Ticket:** NEW — Split card stack characteristics

**709.3c** — BOUNDARY-DEF. Copy of split card retains both halves' characteristics. **Notable exception:** If the split card is already on the stack (one half being cast), a copy effect copies only the half that is being cast (per 709.3b — only the cast half's characteristics exist on the stack). This rule applies when copying the *card* (e.g., from another zone), not when copying the *spell* on the stack.

**BOUNDARY-709.3c-001**
- **Rule:** 709.3c — Copy of a split card (from a non-stack zone) retains both halves. But copy of a split *spell* on the stack copies only the cast half.
- **Boundary:** Copy system must distinguish between copying a split card (both halves) vs. copying a split spell on the stack (one half only, per 709.3b).
- **Engine Constraint:** When determining copiable values of a split card, check whether the source is a spell on the stack (single half) or a card in another zone (both halves).
- **Phase:** Phase 9
- **Ticket:** NEW — Split card copy zone-awareness (709.3c + 709.3b)

**709.4** — TESTABLE. In every zone except stack, characteristics are both halves combined.

**ATOM-709.4-001**
- **Rule:** 709.4 — In hand/graveyard/library, split card has combined characteristics of both halves.
- **Mechanism:** Combined characteristic zones
- **Minimal Board:** Fire//Ice in graveyard. An effect asks "is there a red card in your graveyard?" and "is there a blue card in your graveyard?"
- **Action:** Check characteristics.
- **Expected Result:** Both are true. Fire//Ice is both red and blue in the graveyard.
- **Phase:** Phase 9
- **Ticket:** NEW — Split card combined characteristics

**709.4a** — TESTABLE. Split card has two names; name-choosing must pick one, not both.

**ATOM-709.4a-001**
- **Rule:** 709.4a — Choosing a split card's name: must choose one of the two names, not both. An object "has the chosen name" if one of its names matches.
- **Mechanism:** Split card name matching
- **Minimal Board:** Player names "Fire" with Pithing Needle. Fire//Ice is in hand.
- **Action:** Check if Fire//Ice is named "Fire."
- **Expected Result:** Yes — it has the name "Fire" (one of its two names matches).
- **Phase:** Phase 9
- **Ticket:** NEW — Multi-name object support

**709.4b** — TESTABLE. Combined mana cost; combined colors and mana value. Symbol references see separate symbols.

**ATOM-709.4b-001**
- **Rule:** 709.4b — Split card's mana cost is combined. Colors and MV derived from combined cost.
- **Mechanism:** Combined mana cost calculation
- **Minimal Board:** Assault//Battery in hand. Assault = {R}, Battery = {3}{G}.
- **Action:** Check mana cost, colors, mana value.
- **Expected Result:** Combined mana cost = {3}{R}{G}. Colors = red and green. Mana value = 5.
- **Phase:** Phase 9
- **Ticket:** NEW — Split card mana cost combination

**709.4c** — PURE-DEF. Split card has each type and ability from both halves. Mechanical consequence is subsumed by 709.4.

**709.4d** — DEFERRED — Phase 9: Fused split spell characteristics (both halves combined on stack). Requires fuse keyword.

**709.5** — DEFERRED — Phase 9+: Room permanent cards with shared type line, locked/unlocked designations. This is the Rooms mechanic (Duskmourn).

**709.5a–709.5j** — DEFERRED — Phase 9+: All Room sub-rules (shared type line, unlock costs, lock/unlock triggers, "door" terminology). These are all part of the Rooms mechanic and deferred together.

> **Architecture accommodation note (audit):** Rooms introduce several concepts that may need early accommodation:
> - **Designations** (709.5c): "left half unlocked" / "right half unlocked" are a new kind of permanent status. The engine's permanent status model may need a `designations: HashSet<Designation>` field or similar. This is the same pattern as monstrous/renowned/suspected — if we already plan a generic designation system, Rooms slot in naturally.
> - **Unlock cost as special action** (709.5e): This is a new special action type (like morph face-up). The special action framework should be designed to be extensible.
> - **Shared type line** (709.5a): Two halves sharing types is unusual but doesn't break the characteristic system — it's similar to how split cards combine characteristics in non-stack zones.
> - **Verdict:** No immediate architecture changes required if we design designations and special actions generically. Flag for review when implementing Phase 9 special actions.

### 710. Flip Cards

**710.1** — PURE-DEF. Flip cards have a two-part frame. Normal characteristics right-side up, alternative upside down.

**710.1a** — PURE-DEF. Top half = normal characteristics. Usually contains flip trigger.

**710.1b** — PURE-DEF. Bottom half = alternative characteristics. Used only if permanent is flipped and on battlefield.

**710.1c** — PURE-DEF. Color and mana cost don't change on flip. External effects still apply.

**710.2** — TESTABLE. In all zones except battlefield (and on battlefield before flipping), only normal characteristics. After flip, alternative characteristics replace name, text box, type line, P/T.

**ATOM-710.2-001**
- **Rule:** 710.2 — Unflipped: normal characteristics. Flipped: alternative name, text box, type line, P/T apply; color and mana cost unchanged.
- **Mechanism:** Flip card characteristic switching
- **Minimal Board:** Akki Lavarunner (2/1 nonlegendary creature) on battlefield, unflipped. Flip condition met.
- **Action:** Permanent flips to Tok-Tok, Volcano Born (legendary).
- **Expected Result:** Name = "Tok-Tok, Volcano Born." Type line includes legendary. P/T = Tok-Tok's values. Color and mana cost = same as Akki Lavarunner. "Legendary creatures get +2/+2" now applies.
- **Phase:** Phase 9
- **Ticket:** NEW — Flip card system

**ATOM-710.2-002**
- **Rule:** 710.2 — In library/hand/graveyard, flip card has only normal characteristics.
- **Mechanism:** Flip card zone-based characteristics
- **Minimal Board:** Akki Lavarunner in library. Effect: "Search your library for a legendary card."
- **Action:** Search.
- **Expected Result:** Akki Lavarunner is NOT found (it's nonlegendary in the library). Only normal characteristics are visible in non-battlefield zones.
- **Phase:** Phase 9
- **Ticket:** NEW — Flip card zone characteristics

**710.3** — PURE-DEF. Must visually track flipped/unflipped status. Implementation concern.

**710.4** — TESTABLE. Flipping is one-way. Can't unflip. Leaves battlefield → loses flipped status.

**ATOM-710.4-001**
- **Rule:** 710.4 — Flipping is permanent and one-way. If it leaves the battlefield, it loses flipped status.
- **Mechanism:** One-way flip + zone-change reset
- **Minimal Board:** Tok-Tok (flipped Akki Lavarunner) is bounced to hand.
- **Action:** Creature returns to hand.
- **Expected Result:** In hand, it's Akki Lavarunner (normal characteristics). If re-cast and enters battlefield, it enters unflipped.
- **Phase:** Phase 9
- **Ticket:** NEW — Flip card one-way + status reset

**710.5** — PURE-DEF. Player may choose a flip card's alternative name when choosing a card name.

### 711. Leveler Cards

**711.1** — PURE-DEF. Leveler cards have striated text box, three P/T boxes, two level symbols.

**711.2** — TESTABLE. Level symbol = keyword ability representing a static ability. Defines a P/T and abilities conditional on level counter count.

**ATOM-711.2-001**
- **Rule:** 711.2 + 711.2a + 711.2b — Level symbols define conditional P/T and abilities based on level counter count.
- **Mechanism:** Level counter conditional characteristics
- **Minimal Board:** Leveler creature with: base 0/1; LEVEL 2-4: 1/2 + flying; LEVEL 5+: 3/3 + flying + first strike. Currently has 3 level counters.
- **Action:** Check characteristics.
- **Expected Result:** P/T = 1/2. Has flying. Does NOT have first strike (needs 5+ counters).
- **Phase:** Phase 9
- **Ticket:** NEW — Leveler card static ability system

**ATOM-711.2-002**
- **Rule:** 711.2a + 711.2b — Level counters at 5+: top-level abilities apply.
- **Mechanism:** Level counter maximum range
- **Minimal Board:** Same leveler creature with 7 level counters.
- **Action:** Check characteristics.
- **Expected Result:** P/T = 3/3. Has flying and first strike.
- **Phase:** Phase 9
- **Ticket:** NEW — Leveler card level 5+ range

**711.3** — PURE-DEF. Text box striations have no game significance beyond grouping.

**711.4** — TESTABLE. Non-level-symbol abilities (including level up activation) always available, regardless of counter count.

**ATOM-711.4-001**
- **Rule:** 711.4 — Level up ability is always available, even with 0 level counters.
- **Mechanism:** Level up always-active
- **Minimal Board:** Leveler creature with 0 level counters.
- **Action:** Player activates level up.
- **Expected Result:** Level counter is added. The level up ability was available despite having 0 counters.
- **Phase:** Phase 9
- **Ticket:** NEW — Level up activation

**711.5** — TESTABLE. Level counters < N1 → uses uppermost (base) P/T box.

**ATOM-711.5-001**
- **Rule:** 711.5 — Fewer level counters than the first level symbol range → base P/T.
- **Mechanism:** Base P/T fallback
- **Minimal Board:** Leveler creature with LEVEL 2-4 as first symbol. Currently has 1 level counter.
- **Action:** Check P/T.
- **Expected Result:** Uses the uppermost P/T box (base P/T), not the LEVEL 2-4 values.
- **Phase:** Phase 9
- **Ticket:** NEW — Leveler base P/T

**711.6** — TESTABLE. In non-battlefield zones, leveler card uses uppermost P/T box.

**ATOM-711.6-001**
- **Rule:** 711.6 — In hand/graveyard/library, leveler card has base (uppermost) P/T.
- **Mechanism:** Leveler non-battlefield P/T
- **Minimal Board:** Leveler card in graveyard. Effect checks "is this card's power 3 or greater?"
- **Action:** Check.
- **Expected Result:** Uses the base P/T (uppermost box), not any level-conditional P/T.
- **Phase:** Phase 9
- **Ticket:** NEW — Leveler zone-based P/T

**711.7** — PURE-DEF. Class enchantments are not level up; class levels don't interact with level counters. Cross-reference to rule 716.

### 712. Double-Faced Cards

**712.1** — PURE-DEF. DFC has a Magic face on each side (or half oversized for meld). Three kinds: nonmodal DFC, modal DFC, meld cards.

**712.2** — PURE-DEF. Nonmodal DFCs can transform/convert. Introduces the concept.

**712.2a** — PURE-DEF. Front-face symbol description by set. No game-mechanics consequence.

**712.2b** — PURE-DEF. Back-face symbol description by set. No game-mechanics consequence.

**712.2c** — PURE-DEF. Gray P/T reminder text on front face. No gameplay effect.

**712.3** — PURE-DEF. Modal DFCs have independent faces; may have transform/convert.

**712.3a** — PURE-DEF. Modal DFC front-face symbol.

**712.3b** — PURE-DEF. Modal DFC back-face symbol.

**712.3c** — PURE-DEF. Hint bar is reminder text, no gameplay effect.

**712.4** — PURE-DEF. Meld cards have a Magic face on one side and half oversized face on other.

**712.4a** — TESTABLE. Meld: exile both cards in a pair and put them onto the battlefield combined with back faces up as a single permanent.

**ATOM-712.4a-001**
- **Rule:** 712.4a — Melding two cards exiles both and puts them onto the battlefield as a single permanent with back faces up.
- **Mechanism:** Meld zone transition + combined permanent
- **Minimal Board:** Player controls both halves of a meld pair (e.g., Midnight Scavengers + Graf Rats). Meld trigger condition is met.
- **Action:** Meld ability resolves.
- **Expected Result:** Both cards are exiled, then enter the battlefield as a single permanent (Chittering Host) with back faces up. One object, two cards. Characteristics are those of the combined back face.
- **Phase:** Phase 9
- **Ticket:** NEW — Meld system

**712.4b** — BOUNDARY-DEF. Back faces of meld pair used only for characteristics of the melded permanent on the battlefield.

**BOUNDARY-712.4b-001**
- **Rule:** 712.4b — Back face of a meld card can only determine characteristics when it's part of a melded permanent on the battlefield. Otherwise, fails to determine characteristics.
- **Boundary:** Engine must not allow meld card back-face characteristics to be queried unless the card is part of an active melded permanent.
- **Engine Constraint:** characteristic_of_back_face() returns None/error for a lone meld card.
- **Phase:** Phase 9
- **Ticket:** NEW — Meld back-face restriction

**712.4c** — TESTABLE. Meld cards cannot transform or convert. Instructions to do so are ignored.

**ATOM-712.4c-001**
- **Rule:** 712.4c — Meld cards cannot transform or convert. Any instruction to do so is ignored.
- **Mechanism:** Transform/convert rejection for meld
- **Minimal Board:** Melded permanent (Chittering Host) on battlefield. An effect says "transform target permanent."
- **Action:** Effect targets Chittering Host.
- **Expected Result:** Nothing happens. Meld cards are explicitly excluded from transforming.
- **Phase:** Phase 9
- **Ticket:** NEW — Meld transform rejection

**712.5** — PURE-DEF. Enumerates the seven meld pairs. Data, not mechanics.

**712.5a–712.5g** — PURE-DEF. Specific meld pair listings.

**712.6** — PURE-DEF. Players may look at both sides of a DFC they're allowed to see.

**712.7** — PURE-DEF. Opaque sleeves / substitute cards for hidden zones. Tournament rule.

**712.8** — TESTABLE. Each face of a non-meld DFC has its own characteristics. Meld front face and combined back face each have their own characteristics.

**ATOM-712.8-001**
- **Rule:** 712.8 — Each face of a DFC has independent characteristics. Only the face that's "up" provides characteristics in relevant contexts.
- **Mechanism:** DFC per-face characteristic isolation
- **Minimal Board:** A nonmodal DFC permanent with front face up on the battlefield.
- **Action:** Query characteristics.
- **Expected Result:** Only front-face characteristics are returned. Back-face characteristics are not accessible.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC face-based characteristic system

**712.8a** — TESTABLE. DFC outside the game or in non-battlefield/non-stack zone → front face characteristics only.

**ATOM-712.8a-001**
- **Rule:** 712.8a — In hand/library/graveyard/exile, DFC has only front-face characteristics.
- **Mechanism:** DFC zone-based face selection
- **Minimal Board:** A nonmodal DFC (front = creature, back = creature) in graveyard.
- **Action:** Effect checks "is there a card with flying in your graveyard?" (flying is on back face only).
- **Expected Result:** No. Only front-face characteristics visible in graveyard.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC zone characteristics

**712.8b** — PURE-DEF. Meld card on stack has only front-face characteristics. Subsumed by 712.8a logic.

**712.8c** — TESTABLE. Nonmodal DFC spell: front face up by default. If cast "transformed"/"converted" → back face up on stack. Mana value always uses front-face mana cost.

**ATOM-712.8c-001**
- **Rule:** 712.8c — Nonmodal DFC cast "transformed" has back face up on stack, but mana value calculated from front face.
- **Mechanism:** Transformed casting MV calculation
- **Minimal Board:** Nonmodal DFC (front = 3 CMC creature, back = 0 CMC creature). Cast "transformed."
- **Action:** Spell is on the stack with back face up.
- **Expected Result:** Characteristics are those of the back face. Mana value = 3 (from front face's mana cost, not back face).
- **Phase:** Phase 9
- **Ticket:** NEW — DFC transformed casting + MV

**712.8d** — PURE-DEF. Front-face-up DFC permanent has only front-face characteristics. Subsumed by 712.8.

**712.8e** — TESTABLE. Nonmodal DFC back-face-up permanent: back-face characteristics but mana value from front face. Copy of back face → MV = 0.

**ATOM-712.8e-001**
- **Rule:** 712.8e — Back-face-up nonmodal DFC: mana value derived from front face, not back face.
- **Mechanism:** Back-face MV derivation (destruction check)
- **Minimal Board:** Nonmodal DFC with front face CMC = 4. Back face is up on the battlefield (it's a creature). Effect says "Destroy all creatures with mana value 3 or less."
- **Action:** Effect resolves.
- **Expected Result:** DFC survives. Its mana value is 4 (from front face), not 0 (back faces of nonmodal DFCs don't have mana costs). 4 > 3.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC back-face MV rules

**ATOM-712.8e-002**
- **Rule:** 712.8e — A copy of the back face of a nonmodal DFC has MV = 0.
- **Mechanism:** Copy-of-back-face MV
- **Minimal Board:** Nonmodal DFC with front CMC = 4, back face up. Clone copies the back face.
- **Action:** Check MV of the Clone.
- **Expected Result:** Clone's MV = 0. It copied the back face, which has no independent mana cost. The front-face MV derivation only applies to the actual DFC card, not copies of its back face.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC back-face MV rules

**712.8f** — PURE-DEF. Modal DFC spell/permanent: characteristics of the face that's up. Subsumed by 712.8.

**712.8g** — TESTABLE. Melded permanent: combined back face characteristics. MV = sum of front faces' MVs. Copy of melded permanent → MV = 0.

**ATOM-712.8g-001**
- **Rule:** 712.8g — Melded permanent's MV = sum of both front faces' mana values.
- **Mechanism:** Melded permanent MV calculation
- **Minimal Board:** Chittering Host (melded from Midnight Scavengers MV=5 + Graf Rats MV=2) on battlefield. Effect says "Destroy all creatures with mana value 6 or less."
- **Action:** Effect resolves.
- **Expected Result:** Chittering Host survives. Its MV = 5 + 2 = 7. 7 > 6.
- **Phase:** Phase 9
- **Ticket:** NEW — Meld MV calculation

**ATOM-712.8g-002**
- **Rule:** 712.8g — A copy of a melded permanent has MV = 0.
- **Mechanism:** Copy of melded permanent MV
- **Minimal Board:** Chittering Host (MV=7) on battlefield. Clone copies it. Effect says "Destroy all creatures with mana value 0."
- **Action:** Effect resolves.
- **Expected Result:** Clone is destroyed (MV = 0). Chittering Host survives (MV = 7).
- **Phase:** Phase 9
- **Ticket:** NEW — Meld MV calculation

**712.9** — TESTABLE. Only DFC tokens and non-meld DFC cards can transform/convert. Non-DFC permanent instructed to transform → nothing happens.

**ATOM-712.9-001**
- **Rule:** 712.9 — Non-DFC permanent can't transform. Instruction to do so → nothing.
- **Mechanism:** Transform eligibility check
- **Minimal Board:** Clone (not a DFC) entered as a copy of a DFC's back face. Effect says "transform target creature."
- **Action:** Transform targets the Clone.
- **Expected Result:** Nothing happens. Clone is not a DFC card.
- **Phase:** Phase 9
- **Ticket:** NEW — Transform eligibility guard

**ATOM-712.9-002**
- **Rule:** 712.9 — A DFC card that has become a copy of a non-DFC creature can still transform (it's still a DFC card).
- **Mechanism:** DFC identity persists through copy
- **Minimal Board:** DFC creature (front face) becomes a copy of Elite Vanguard via Cytoshape. Effect says "Transform all Humans."
- **Action:** Transform instruction.
- **Expected Result:** The DFC card transforms (it's still physically a DFC). Back face is now up, but it's still a copy of Elite Vanguard this turn.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC transform through copy

**712.10** — TESTABLE. Transform/convert into instant or sorcery face → nothing happens.

**ATOM-712.10-001**
- **Rule:** 712.10 — If transform/convert would result in an instant or sorcery face, nothing happens.
- **Mechanism:** Transform-to-instant/sorcery rejection
- **Minimal Board:** DFC permanent whose back face is a sorcery. Effect says "transform this permanent."
- **Action:** Transform attempted.
- **Expected Result:** Nothing happens. Can't transform into an instant or sorcery face.
- **Phase:** Phase 9
- **Ticket:** NEW — Transform face-type validation

**712.11** — TESTABLE. DFC spell cast front face up by default.

**ATOM-712.11-001**
- **Rule:** 712.11 — A DFC spell is put on the stack with its front face up by default.
- **Mechanism:** Default DFC casting orientation
- **Minimal Board:** Player casts a nonmodal DFC creature spell normally.
- **Action:** Spell goes on stack.
- **Expected Result:** Front face is up. Spell has front-face characteristics.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC casting default face

**712.11a** — TESTABLE. Cast "transformed"/"converted" → back face up on stack.

**ATOM-712.11a-001**
- **Rule:** 712.11a — Casting a DFC "transformed" puts it on the stack with back face up.
- **Mechanism:** Transformed casting orientation
- **Minimal Board:** An ability allows a DFC to be cast "transformed." Player casts it.
- **Action:** Spell placed on stack.
- **Expected Result:** Back face is up on the stack. Characteristics are those of the back face.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC transformed casting

**712.11b** — TESTABLE. Modal DFC: player chooses which face to cast before putting on stack.

**ATOM-712.11b-001**
- **Rule:** 712.11b — When casting a modal DFC, player chooses which face before putting it on the stack.
- **Mechanism:** Modal DFC face selection on cast
- **Minimal Board:** Player has a modal DFC (front = instant, back = creature) in hand. Stack is empty, main phase.
- **Action:** Player chooses to cast the creature (back face).
- **Expected Result:** Creature face is placed on the stack. Spell has creature characteristics.
- **Phase:** Phase 9
- **Ticket:** NEW — Modal DFC casting

**712.11c** — PURE-DEF. Only the face that will be up on the stack is evaluated for castability. Subsumed by 712.11b.

**712.11d** — BOUNDARY-DEF. Front face ability that allows cast "transformed" is also considered when evaluating castability (exception to 712.11c).

**BOUNDARY-712.11d-001**
- **Rule:** 712.11d — Front-face ability enabling cast "transformed" is considered even though back face would be up.
- **Boundary:** Castability checker must also evaluate front-face abilities that enable transformed casting.
- **Engine Constraint:** When checking if a DFC can be cast transformed, both the back-face characteristics and the front-face enabling ability must be considered.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC transformed-cast evaluation exception

**712.12** — TESTABLE. Playing a modal DFC as a land: choose a land face before placing on battlefield.

**ATOM-712.12-001**
- **Rule:** 712.12 — Playing a modal DFC as a land: player chooses which face (must be a land face) before it enters.
- **Mechanism:** Modal DFC land-play face selection
- **Minimal Board:** Player has a modal DFC (front = creature, back = land). It's their main phase, no land played yet.
- **Action:** Player plays it as a land, choosing the back face.
- **Expected Result:** Card enters the battlefield with its back (land) face up. This is a special action (playing a land) — it does NOT use the stack. No player receives priority between the land play and it being on the battlefield.
- **Phase:** Phase 9
- **Ticket:** NEW — Modal DFC land play

**ATOM-712.12-002**
- **Rule:** 712.12 — Playing a modal DFC as a land is a special action, not a spell cast.
- **Mechanism:** Modal DFC land play does not use stack
- **Minimal Board:** Player has a modal DFC (front = instant, back = land). Opponent controls a "Whenever a player casts a spell" trigger.
- **Action:** Player plays the DFC as a land (back face).
- **Expected Result:** Land enters the battlefield. The "whenever a player casts a spell" trigger does NOT fire. The stack was not used. No priority passes occurred.
- **Phase:** Phase 9
- **Ticket:** NEW — Modal DFC land play

**712.13** — TESTABLE. Resolving DFC spell → enters battlefield with same face that was up on stack.

**ATOM-712.13-001**
- **Rule:** 712.13 — Resolving DFC spell: permanent enters with same face up as on stack.
- **Mechanism:** DFC resolution face preservation
- **Minimal Board:** DFC creature spell on stack with front face up. It resolves.
- **Action:** Spell resolves.
- **Expected Result:** Permanent enters the battlefield with front face up.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC resolution face carry-through

**712.13a** — TESTABLE. Ability causes DFC spell (front face up on stack) to enter transformed → if back face is instant/sorcery, goes to graveyard instead.

**ATOM-712.13a-001**
- **Rule:** 712.13a — DFC entering "transformed" but back face is instant/sorcery → goes to graveyard instead of entering.
- **Mechanism:** DFC enter-transformed rejection
- **Minimal Board:** DFC with sorcery back face on stack (front face up). Daybound forces it to enter back-face-up because it's night.
- **Action:** Spell resolves.
- **Expected Result:** Can't enter with sorcery face up. Put into owner's graveyard instead.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC sorcery-face enter rejection

**ATOM-712.13a-002**
- **Rule:** 712.13a — Stress test: Clone copying a daybound DFC creature + Mystic Reflection + Siege battle with sorcery back face.
- **Mechanism:** Multi-rule DFC enter-transformed rejection via clone + replacement
- **Minimal Board:** Player controls Mycosynth Lattice + March of the Machines (all permanents are artifact creatures). Player controls a Clone that is a copy of Bird Admirer (a creature with daybound). It is currently night, but the Clone can't transform (not a DFC card). Player casts Mystic Reflection targeting the Clone, then casts Invasion of Kylem (a Siege battle whose back face is a sorcery).
- **Action:** Invasion of Kylem resolves. Because of March of the Machines it would enter as a creature. Mystic Reflection's replacement effect tries to make it enter as a copy of the Clone (which is a copy of Bird Admirer). Since it is night, the daybound ability would normally force it to enter with its back face up. But its back face is a sorcery.
- **Expected Result:** Invasion of Kylem can't enter with its sorcery back face up. It is put into its owner's graveyard instead of entering the battlefield.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC sorcery-face enter rejection (complex)

**712.14** — TESTABLE. DFC put onto battlefield from non-stack zone → front face up by default.

**ATOM-712.14-001**
- **Rule:** 712.14 — DFC entering from non-stack zone defaults to front face up.
- **Mechanism:** DFC default battlefield orientation (non-stack)
- **Minimal Board:** Effect returns a DFC creature card from graveyard to battlefield.
- **Action:** Card enters battlefield.
- **Expected Result:** Enters with front face up.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC non-stack entry default

**712.14a** — TESTABLE. Put onto battlefield "transformed"/"converted" → back face up. Non-DFC instructed to enter transformed → stays in current zone.

**ATOM-712.14a-001**
- **Rule:** 712.14a — DFC put onto battlefield "transformed" enters with back face up. Non-DFC card stays in current zone.
- **Mechanism:** Transformed entry + non-DFC rejection
- **Minimal Board:** Effect puts a DFC "transformed" onto the battlefield. Also tries to put a normal card "transformed."
- **Action:** Both cards are affected.
- **Expected Result:** DFC enters with back face up. Normal card stays in its current zone (does not enter).
- **Phase:** Phase 9
- **Ticket:** NEW — DFC transformed entry

**712.14b** — TESTABLE. Modal DFC put onto battlefield and front face isn't a permanent → stays in current zone.

**ATOM-712.14b-001**
- **Rule:** 712.14b — Modal DFC whose front face is an instant/sorcery can't enter the battlefield normally → stays in current zone.
- **Mechanism:** Modal DFC non-permanent front face rejection
- **Minimal Board:** Modal DFC (front = instant, back = land) in hand. Effect says "put this card onto the battlefield."
- **Action:** Attempt to put onto battlefield.
- **Expected Result:** Front face is instant (not a permanent). Card stays in hand.
- **Phase:** Phase 9
- **Ticket:** NEW — Modal DFC front-face permanent check

**712.14c** — PURE-DEF. Melded permanent entering melded → back faces up combined. Subsumed by 712.4a.

**712.15** — DEFERRED — Phase 9: DFC cast face-down / enters face-down. Face-down mechanics + DFC interaction.

**712.15a** — DEFERRED — Phase 9: Face-down DFC can't transform/convert. Turned face up → front face up.

**712.16** — TESTABLE. DFC permanents (including melded) can't be turned face down. Attempt → nothing.

**ATOM-712.16-001**
- **Rule:** 712.16 — DFC permanents can't be turned face down. Attempting to do so is a no-op.
- **Mechanism:** DFC face-down rejection
- **Minimal Board:** A DFC permanent on the battlefield (front face up). Effect says "turn target permanent face down."
- **Action:** Effect targets the DFC.
- **Expected Result:** Nothing happens. DFCs can't be turned face down.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC face-down rejection

**712.17** — PURE-DEF. DFC exiled face down remains hidden. Substitute card / sleeves.

**712.18** — TESTABLE. Transforming/converting doesn't make a new object. Existing effects persist.

**ATOM-712.18-001**
- **Rule:** 712.18 — Transform/convert doesn't create a new object. Effects that applied before still apply.
- **Mechanism:** Transform object continuity
- **Minimal Board:** Village Ironsmith (front face, DFC) has +2/+2 until end of turn from Giant Growth. It transforms into Ironfang.
- **Action:** Transform occurs.
- **Expected Result:** Ironfang still has +2/+2 until end of turn. Same object, effects persist.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC transform continuity

**712.19** — PURE-DEF. Name-choosing: may choose either face's name of a DFC but not both. May choose combined meld back face name.

**712.20** — TESTABLE. "As this transforms" ability applied during transform, not after.

**ATOM-712.20-001**
- **Rule:** 712.20 — "As [this] transforms" ability is applied during the transform process.
- **Mechanism:** Transform-time replacement
- **Minimal Board:** DFC with "As this transforms, choose a color" on its back face. It transforms.
- **Action:** Transform occurs.
- **Expected Result:** Color choice is made during the transform event, before the permanent is fully on its new face.
- **Phase:** Phase 9
- **Ticket:** NEW — DFC transform-time "as" ability

**712.21** — TESTABLE. Melded permanent leaves → one permanent leaves, two cards go to appropriate zone.

**ATOM-712.21-001**
- **Rule:** 712.21 — Melded permanent leaving battlefield: one permanent leaves, two cards put into the zone.
- **Mechanism:** Meld zone-change card split
- **Minimal Board:** Chittering Host (melded) dies.
- **Action:** Creature dies.
- **Expected Result:** "Whenever a creature dies" triggers once. "Whenever a card is put into a graveyard" triggers twice (two cards). Both Midnight Scavengers and Graf Rats are in the graveyard.
- **Phase:** Phase 9
- **Ticket:** NEW — Meld death/zone-change handling

**712.21a** — TESTABLE. Melded permanent to graveyard/library → owner arranges two cards in any order.

**ATOM-712.21a-001**
- **Rule:** 712.21a — Melded permanent put into graveyard or library: owner may arrange the two cards in any order.
- **Mechanism:** Meld card ordering on zone entry
- **Minimal Board:** Chittering Host is put into owner's library (e.g., tucked).
- **Action:** Two cards go to library.
- **Expected Result:** Owner chooses the order of the two cards. Order is not revealed if going to library.
- **Phase:** Phase 9
- **Ticket:** NEW — Meld library ordering

**712.21b** — BOUNDARY-DEF. Exiling a melded permanent: player determines relative timestamp of two cards.

**BOUNDARY-712.21b-001**
- **Rule:** 712.21b — When exiling a melded permanent, the exiling player determines the relative timestamp order of the two exiled cards.
- **Boundary:** Engine must support per-card timestamp assignment on meld exile.
- **Engine Constraint:** When a melded permanent is exiled, the controller picks which card has the more recent timestamp (relevant for "last card exiled" effects).
- **Phase:** Phase 9
- **Ticket:** NEW — Meld exile timestamp ordering

**712.21c** — TESTABLE. Effect that finds the new object from a melded permanent leaving → finds both cards. Actions taken on those cards apply to each.

**ATOM-712.21c-001**
- **Rule:** 712.21c — Effect tracking a melded permanent that left the battlefield finds both cards. Actions apply to each.
- **Mechanism:** Meld double-card tracking
- **Minimal Board:** Otherworldly Journey exiles Chittering Host. At next end step, "return that card with a +1/+1 counter."
- **Action:** End step trigger resolves.
- **Expected Result:** Both Midnight Scavengers and Graf Rats return to the battlefield, each with a +1/+1 counter.
- **Phase:** Phase 9
- **Ticket:** NEW — Meld card tracking on zone change

**712.21d** — TESTABLE. If multiple replacement effects could apply to a melded permanent leaving the battlefield, the controller chooses ONE replacement effect and it applies to BOTH cards. The controller cannot split different replacement effects among the two cards.

**ATOM-712.21d-001**
- **Rule:** 712.21d — Multiple replacement effects on melded permanent leaving: controller picks one, it applies to both cards. Cannot split.
- **Mechanism:** Meld replacement effect non-splitting
- **Minimal Board:** Chittering Host (melded) dying. Two replacement effects apply: Leyline of the Void ("exile instead of graveyard") and Wheel of Sun and Moon ("put on bottom of library instead"). Both could apply.
- **Action:** Controller chooses Leyline of the Void.
- **Expected Result:** Both Midnight Scavengers and Graf Rats are exiled. The controller cannot choose Leyline for one card and Wheel for the other — both cards must be affected by the same chosen replacement effect.
- **Phase:** Phase 9
- **Ticket:** NEW — Meld replacement non-splitting

**712.21e** — TESTABLE. Melded permanent counts as one object that moved but two cards that moved.

**ATOM-712.21e-001**
- **Rule:** 712.21e — Melded permanent = 1 object moved, 2 cards moved.
- **Mechanism:** Meld object vs. card count
- **Minimal Board:** Chittering Host dies alongside another creature. Effect: "for each creature that died this turn, draw a card."
- **Action:** Count creatures that died.
- **Expected Result:** 2 creatures died (Chittering Host = 1 object + the other creature = 1). But "for each card put into a graveyard from the battlefield," Chittering Host counts as 2 cards.
- **Phase:** Phase 9
- **Ticket:** NEW — Meld object/card counting

--- End of Chunk 3 ---

## Chunk 4 — Classification Summary, Composition Tests, Gap Report

### Classification Summary

| Classification | Count | Rules |
|---------------|-------|-------|
| **TESTABLE** | ~78 | 703.3, 703.4c, 703.4d, 703.4n, 703.4p, 703.4q, 704.3, 704.4, 704.5c, 704.5d, 704.5e, 704.5i, 704.5j, 704.5k, 704.5m, 704.5n, 704.5p, 704.5q, 704.5r, 704.7, 704.8, 705.1, 705.2, 705.3, 706.2, 706.2b, 706.3a, 706.3c, 706.6, 707.2, 707.2b, 707.3, 707.4, 707.5, 707.6, 707.9a, 707.9b, 707.9d, 707.9e, 707.9f, 707.10, 707.10b, 707.10c, 707.10d, 707.10e, 707.12, 708.2, 708.2a, 708.2b, 708.3, 708.4, 708.8, 708.10, 708.11, 709.3, 709.3a, 709.3b, 709.4, 709.4a, 709.4b, 710.2, 710.4, 711.2, 711.4, 711.5, 711.6, 712.4a, 712.4c, 712.8, 712.8a, 712.8c, 712.8e, 712.8g, 712.9, 712.10, 712.11, 712.11a, 712.11b, 712.12, 712.13, 712.13a, 712.14, 712.14a, 712.14b, 712.16, 712.18, 712.20, 712.21, 712.21a, 712.21c, 712.21d, 712.21e |
| **BOUNDARY-DEF** | 9 | 707.2c, 707.7, 707.9g, 708.12, 709.3c, 712.4b, 712.11d, 712.21b |
| **PURE-DEF** | ~62 | 703.1, 703.1a, 703.2, 703.4, 705 (physical coin), 706.1, 706.1a, 706.1b, 706.2a, 706.3, 706.3b, 706.4, 707.1, 707.2a, 707.9, 707.9c, 707.10a, 707.11, 708.1, 708.5, 708.6, 708.7, 708.9, 709.1, 709.2, 709.4c, 710.1–710.1c, 710.3, 710.5, 711.1, 711.3, 711.7, 712.1, 712.2–712.2c, 712.3–712.3c, 712.4, 712.5–712.5g, 712.6, 712.7, 712.8b, 712.8d, 712.8f, 712.11c, 712.14c, 712.17, 712.19 |
| **OUT-OF-SCOPE** | 13 | 703.4e (Archenemy), 703.4g (Attractions), 703.4h (2HG/multiplayer attack routing), 704.5t (Dungeons), 704.5u (Space Sculptor), 704.6a (2HG life), 704.6b (2HG poison), 704.6e (Archenemy scheme), 704.6f (Planechase), 706.5 (Celebr-8000), 706.7 (Planechase die), 706.8–706.8c (Centaur of Attention) |
| **DEFERRED** | 21 | 703.4a (phasing, Phase 9), 703.4b (day/night, Phase 9), 703.4f (Saga lore, Phase 8–9), 704.5s (Saga SBA, Phase 8–9), 704.5v–704.5x (Battle SBAs, Phase 8–9), 704.5y (Roles, Phase 8–9), 704.5z (Speed SBA, Phase 8–9), 704.6c (Commander damage, Phase 9), 704.6d (Commander zone, Phase 9), 707.8 (DFC copy, Phase 9), 707.8a (DFC token copy, Phase 9), 707.10f (permanent spell copy, Phase 6+), 707.10g (DFC spell copy, Phase 9), 707.13 (Garth One-Eye, Phase 9), 707.14 (Magar, Phase 9), 709.4d (Fuse, Phase 9), 709.5–709.5j (Rooms, Phase 9+), 712.15–712.15a (DFC face-down, Phase 9) |
| **ALREADY-IMPLEMENTED** | 10 | 703.4i (declare attackers), 703.4j (declare blockers), 703.4k (combat damage assign), 703.4m (combat damage deal), 704.5a (0 life), 704.5b (empty library draw), 704.5f (toughness ≤ 0), 704.5g (lethal damage), 704.5h (deathtouch damage) |

**Total ATOM tests generated:** 87 (post-audit: +9 new tests from audit fixes)
**Total BOUNDARY tests generated:** 9 (post-audit: +1 BOUNDARY-709.3c-001)
**Total classified sub-rules:** ~193

### Composition Tests

**COMP-9A-001 — SBA cascade with counter annihilation and lethal damage**
- **Rules composed:** 704.5q + 704.5f + 704.8
- **Scenario:** Creature with undying has 1 +1/+1 counter. Spell puts 3 -1/-1 counters on it. SBA check simultaneously annihilates counters (704.5q) and checks toughness. LKI uses pre-SBA state (704.8) to determine undying eligibility.
- **Why composition:** Three SBA rules interact in a single check batch with LKI dependency.
- **ATOMs required:** ATOM-704.5q-001, ATOM-704.8-001

**COMP-9A-002 — Copy entering as a copy with ETB replacement + trigger**
- **Rules composed:** 707.5 + 707.2 + 707.6
- **Scenario:** Clone enters as a copy of a creature with "enters with 2 fade counters" and "as this enters, choose a color." Clone should get the counters (707.5), have the copied characteristics without the original's counters (707.2), and make a new "as enters" choice (707.6).
- **Why composition:** Three copy rules interact during a single ETB event.
- **ATOMs required:** ATOM-707.5-001, ATOM-707.2-003, ATOM-707.6-001

**COMP-9A-003 — Face-down permanent becomes a copy, then turns face up**
- **Rules composed:** 708.10 + 708.8 + 707.3
- **Scenario:** Face-down morph creature becomes a copy of another creature (708.10). While face down, characteristics unchanged. Turned face up → shows copied creature's characteristics (708.10 + 707.3). Existing effects still apply (708.8).
- **Why composition:** Copy + face-down + face-up revert interact.
- **ATOMs required:** ATOM-708.10-001, ATOM-708.8-001

**COMP-9A-004 — DFC transform through copy effect (712.9 + 707.4)**
- **Rules composed:** 712.9 + 707.4 + 712.18
- **Scenario:** DFC creature becomes a copy of a non-DFC creature (707.4). "Transform all Humans" is cast. The DFC can still transform (712.9 — it's physically a DFC). After transforming, it still has the copy's characteristics this turn but its back face is now up (712.18 — effects persist).
- **Why composition:** Transform eligibility, copy effect persistence, and object continuity all interact.
- **ATOMs required:** ATOM-712.9-002, ATOM-707.4-001, ATOM-712.18-001

**COMP-9A-005 — Melded permanent dies with replacement effects (712.21 + 712.21d + 704.5d)**
- **Rules composed:** 712.21 + 712.21d + 712.21e
- **Scenario:** Chittering Host (melded) dies. A replacement effect (Leyline of the Void) exiles it instead. One creature died (712.21e), two cards changed zones (712.21e), and the replacement effect applies to both cards from a single choice (712.21d).
- **Why composition:** Meld zone-change splitting, replacement broadcasting, and object/card counting all in one event.
- **ATOMs required:** ATOM-712.21-001, ATOM-712.21d-001, ATOM-712.21e-001

**COMP-9A-006 — Copy effect with CDA stripping + "in addition to types" exception (707.9d)**
- **Rules composed:** 707.9d + 707.2
- **Scenario:** Quicksilver Gargantuan (7/7 override) copies Tarmogoyf (P/T CDA) → CDA stripped. Then separately, Glasspool Mimic ("Shapeshifter Rogue in addition to") copies a changeling creature → type CDA preserved. Tests both branches of 707.9d.
- **Why composition:** Two variants of the same rule tested in contrast.
- **ATOMs required:** ATOM-707.9d-001, ATOM-707.9d-002

**COMP-9A-007 — Split card characteristics shift between zones and stack (709.3b + 709.4 + 709.4b)**
- **Rules composed:** 709.3b + 709.4 + 709.4b
- **Scenario:** Fire//Ice in hand (combined characteristics: MV=4, red+blue). Player casts Fire (709.3). On stack: red spell, MV=2, only Fire's characteristics (709.3b). If countered, returns to graveyard with combined characteristics again (709.4).
- **Why composition:** Zone transitions change which characteristics are visible.
- **ATOMs required:** ATOM-709.3b-001, ATOM-709.4-001, ATOM-709.4b-001

### Gap Report

| Gap | Description | Suggested Ticket |
|-----|-------------|-----------------|
| **GAP-1** | Coin flip infrastructure not yet designed. Need RNG abstraction, win/lose tracking, result override hooks. | NEW — Coin flip system (705.x) |
| **GAP-2** | Die roll infrastructure not yet designed. Need dN abstraction, modifier pipeline (reroll first, arithmetic second), results table lookup, "roll again" recursion, ignore semantics. | NEW — Die roll system (706.x) |
| **GAP-3** | Copy system (D5) is a known Phase 6 deliverable but copiable-value determination, CDA stripping logic, copy-with-exception handling, and spell-copy pipeline are all unimplemented. 707.x generates the most ATOM tests in this session. | D5 (existing ticket, substantial scope) |
| **GAP-4** | Face-down permanent system not yet designed. Need face-down characteristic overlay, morph casting pipeline, face-up revert, ETB suppression, reveal-on-leave, face-down copy interaction. | NEW — Face-down system (708.x, Phase 9) |
| **GAP-5** | Split card data model (two-name, combined mana cost, half-selection on cast, stack-characteristic suppression) not yet designed. | NEW — Split card system (709.x, Phase 9) |
| **GAP-6** | Flip card system (one-way flip, characteristic switching, zone-based characteristic selection) not yet designed. | NEW — Flip card system (710.x, Phase 9) |
| **GAP-7** | Leveler card system (level counters, conditional P/T/abilities via static ability, level up activation) not yet designed. | NEW — Leveler card system (711.x, Phase 9) |
| **GAP-8** | DFC system is the largest gap: nonmodal DFC (transform/convert), modal DFC (face selection), meld (combined permanent, zone-change card splitting, replacement broadcasting). Covers 712.x entirely. | NEW — DFC system (712.x, Phase 9) |
| **GAP-9** | Room permanent mechanics (709.5–709.5j) are new as of Duskmourn. Shared type line, lock/unlock designations, unlock cost special action. Entirely undesigned. | NEW — Rooms system (709.5.x, Phase 9+) |
| **GAP-10** | SBA for spell/card copies ceasing to exist (704.5e) requires copy system from D5 first. | D5 prerequisite |
| **GAP-11** | Planeswalker 0-loyalty SBA (704.5i) and legend rule SBA (704.5j) need DecisionProvider hooks for legend-rule choice. Both are Phase 5-Pre tickets. | T14 |
| **GAP-12** | Counter cap SBA (704.5r) and world rule SBA (704.5k) are niche but straightforward additions to the SBA loop. | NEW — Counter cap SBA, NEW — World rule SBA |
| **GAP-13** | SBA coalescing for replacement effects (704.7) requires replacement effect system from Phase 6. | Phase 6 prerequisite |
| **GAP-14** | Pre-SBA LKI snapshot (704.8) requires LKI system from L18. | L18 prerequisite |

### NEW Tickets Summary

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

--- End of Session 9A ---
