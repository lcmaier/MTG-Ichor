# Cards & Integration Tests Unlocked Ledger

> Tracks which real Magic cards become implementable after each ticket, and which integration test checkpoints should verify end-to-end behavior.
>
> **Purpose:** Catch cross-ticket integration bugs early via rolling card implementation + integration tests, rather than deferring all integration testing to L20/L21.
>
> **Status key:** ✅ = ticket done, 🃏 = card implemented & registered, 🧪 = integration test written

---

## How to Use This File

1. After completing a ticket (or cluster of tickets), check which cards are newly unlocked below.
2. Pick 1–3 representative cards from the "Example Cards" column and implement them in `cards/`.
3. Register them in `cards/registry.rs`.
4. Write integration tests in the appropriate `tests/` file exercising the end-to-end game loop.
5. Mark the card 🃏 and test 🧪 in this ledger.

---

## Already Buildable (Phases 1–4.5 + T01–T14)

These cards use **only** already-implemented primitives (DealDamage, DrawCards, GainLife, LoseLife, ProduceMana, CounterSpell, CounterAbility, Destroy, Untap) and already-completed infrastructure (10 keywords, counters, legend rule, PW loyalty, token/copy flags, attachment tracking, summoning sickness rework).

| Card | Type | Mana | What It Exercises | Implemented? |
|------|------|------|-------------------|-------------|
| Shock | {1}{R} Instant — deal 2 damage to any target | {1}{R} | DealDamage (simpler Bolt variant) | |
| Divination | {2}{U} Sorcery — draw 2 cards | {2}{U} | DrawCards, sorcery timing | |
| Dark Ritual | {B} Instant — add {B}{B}{B} | {B} | ProduceMana (mana-producing spell, not ability) | |
| Sign in Blood | {B}{B} Sorcery — target player draws 2, loses 2 life | {B}{B} | DrawCards + LoseLife, player targeting | |
| Naturalize | {1}{G} Instant — destroy target artifact or enchantment | {1}{G} | Destroy with non-creature targeting | |

**Immediate wins (no new primitives needed):**

| Card | Type | What It Exercises | Implemented? |
|------|------|-------------------|-------------|
| Night's Whisper | {1}{B} Sorcery — draw 2, lose 2 life | Sequence(DrawCards, LoseLife) | |
| Heroes' Reunion | {G}{W} Instant — gain 7 life | GainLife, multicolor cost | |
| Negate | {1}{U} Instant — counter target noncreature spell | CounterSpell + noncreature restriction (if targeting supports it, else use Cancel) | |

**Legendary creatures (exercises T14 legend rule):**

| Card | Type | What It Exercises | Implemented? |
|------|------|-------------------|-------------|
| Isamaru, Hound of Konda | {W} 2/2 Legendary Creature — Dog | Legend rule SBA | |

---

## Part 1: Pre-Phase 5 Engine Fixes

### Tier 1: Data Model (T01–T06) — all ✅

These tickets add fields/flags but don't create new observable game behavior on their own. No standalone integration tests needed — they're exercised by downstream tickets.

| Ticket | What It Enables | Downstream User |
|--------|----------------|-----------------|
| T01 ✅ | Counters on permanents | T13 (annihilation SBA), T14 (PW loyalty), L06 (layer 7c), T21c (infect -1/-1) |
| T02 ✅ | Poison counters, commander damage | T16 (poison SBA), T21c (infect to player) |
| T03 ✅ | `is_token` / `is_copy` flags | T13 (token cease-to-exist SBA) |
| T04 ✅ | Attachment tracking | T15/T15b (Aura/Equipment SBAs) |
| T05 ✅ | `color_indicator` on CardData | L10 (Layer 5 color) |
| T06 ✅ | `x_value` carried to permanent | T18 (X-cost spells) |

---

### Tier 2: State Tracking (T09–T12b) — all ✅

| Ticket | What It Enables | Downstream User |
|--------|----------------|-----------------|
| T09 ✅ | `controller_since_turn` summoning sickness | L11 (Layer 2 control change re-sickness) |
| T10 ✅ | `{Q}` cost summoning sickness | Cards with untap-symbol costs (Knacksaw Clique, Pili-Pala) |
| T11 ✅ | `LifeChanged` event source field | Phase 6 triggers ("whenever you gain life") |
| T12 ✅ | Mana restrictions design spike | T12b/T12c/T12d |
| T12b ✅ | ManaPool sidecar types | T12c (engine integration) |

---

### Tier 3: SBAs (T13–T16, T15b)

| Ticket(s) | Cards Unlocked | Example Cards | Status |
|-----------|---------------|---------------|--------|
| T13 ✅ (+ T03) | Token-creating spells (token cease-to-exist SBA works) | Raise the Alarm ({1}{W}, create two 1/1 Soldier tokens) — **needs `Primitive::CreateToken` (stubbed)** | |
| T14 ✅ (+ T01) | Legendary creatures, planeswalkers | Isamaru, Hound of Konda ({W} 2/2 Legendary); test planeswalker (4 loyalty) | |
| T15 (+ T04) | Auras (unattached → GY SBA); Equipment (unattach from non-creature SBA) | — (needs T15b for full attachment on cast) | |
| T15b (+ T04) | **Full Aura lifecycle:** cast → attach → SBA if host dies | Holy Strength ({W} Aura, enchanted creature gets +1/+2); Pacifism ({1}{W} Aura, enchanted creature can't attack or block) | |
| T16 (+ T01, T02) | Indestructible creatures; poison win condition | Darksteel Colossus (11/11 indestructible); test-only "Poison Fang" creature | |

**Checkpoint A: After T13 + T14** (already done)

Integration tests buildable *right now*:
| # | Test | Cards Needed | Status |
|---|------|-------------|--------|
| 1 | Two legendary creatures with same name → legend rule, one kept | 2× Isamaru | |
| 2 | Planeswalker ETB sets loyalty, Bolt removes 3 loyalty, second Bolt → 0 loyalty → GY | Test planeswalker + Lightning Bolt | |
| 3 | Counter annihilation: +1/+1 and -1/-1 counters on same creature cancel out | Test creature + manual counter placement | |

**Checkpoint B: After T15b + T16** (not yet done)

Recommended integration tests (`tests/phase5_pre_integration_test.rs`):
| # | Test | Cards Needed | Status |
|---|------|-------------|--------|
| 4 | Cast Aura on creature, verify attachment, destroy creature, Aura goes to GY | Holy Strength + Grizzly Bears + Lightning Bolt | |
| 5 | Aura with no legal target on ETB → goes to GY immediately | Holy Strength (no creatures on battlefield) | |
| 6 | Indestructible creature survives lethal damage and Destroy effects | Darksteel Myr (0/1 indestructible) + Lightning Bolt | |
| 7 | Token created, bounced to hand, ceases to exist via SBA | Raise the Alarm + Unsummon — **needs CreateToken + ReturnToHand primitives** | |

---

### Tier 4: Casting & Activation (T17–T20, T12c)

| Ticket(s) | Cards Unlocked | Example Cards | Status |
|-----------|---------------|---------------|--------|
| T17 + T18 | Kicker spells, modal spells, X-cost spells, legendary sorceries | Vines of Vastwood ({G}, kicker {G}); Blaze ({X}{R} deal X damage); Urza's Ruinous Blast (legendary sorcery) | |
| T19 | Once-per-turn abilities, sorcery-speed abilities, graveyard-activated abilities | Deathrite Shaman (GY-activated); any creature with cycling | |
| T20 | Linked abilities (e.g., imprint + use) | Chrome Mox (imprint + produce mana of imprinted color) | |
| T12c | Restricted mana in casting pipeline | — (needs T12d for actual cards) | |

**Checkpoint C: After T17 + T18**

Recommended integration tests:
| # | Test | Cards Needed | Status |
|---|------|-------------|--------|
| 8 | Cast X-cost spell, X=3, deal 3 damage to player | Blaze + Mountains | |
| 9 | Cast X-cost draw spell, X=2, draw 2 cards | Mind Spring + Islands | |
| 10 | Cast kicker spell, verify additional cost paid and effect enhanced | Vines of Vastwood (kicked = +4/+4 + hexproof) | |
| 11 | Cast modal spell, choose mode, verify correct mode resolves | Charm variant (e.g., test-only Izzet Charm) | |
| 12 | Cast legendary sorcery rejected without legendary permanent | Urza's Ruinous Blast with no legendary creature | |
| 13 | Sorcery-speed ability can't activate on opponent's turn | Test creature with sorcery-speed activated ability | |

---

### Tier 5: Zone, Combat, Damage, Targeting (T21a–T22)

| Ticket(s) | Cards Unlocked | Example Cards | Status |
|-----------|---------------|---------------|--------|
| T21a (+ T17) | CastInfo on permanents; instant/sorcery can't enter battlefield | Any permanent spell (CastInfo auto-populated) | |
| T21b | Menace, Shadow, Fear, Intimidate, Skulk, Horsemanship, Landwalk creatures | Goblin War Drums creature (menace); Soltari Trooper (shadow); Jhessian Infiltrator (unblockable ~ skulk); Bog Wraith (swampwalk) | |
| T21c (+ T01, T02) | **Infect creatures**, wither creatures, toxic creatures, planeswalker combat | Glistener Elf ({G} 1/1 infect); Blighted Agent ({1}{U} 1/1 infect unblockable); Plague Stinger ({1}{B} 1/1 flying infect) | |
| T21d (+ T21b) | Goad, Lure effects, "must block" / "must attack" enforcement | — (population hooks only, needs Phase 6 for actual triggers) | |
| T22 | Hexproof creatures, shroud creatures, protection creatures; new durations | Slippery Bogle ({G/U} 1/1 hexproof); Invisible Stalker ({1}{U} 1/1 hexproof unblockable); Troll Ascetic ({1}{G}{G} 3/2 hexproof); Progenitus (protection from everything) | |

**Checkpoint D: After T21a–T22**

Recommended integration tests:
| # | Test | Cards Needed | Status |
|---|------|-------------|--------|
| 14 | Menace creature can't be blocked by one creature | Menace creature + 1 blocker | |
| 15 | Infect creature deals combat damage → poison counters, no life loss | Glistener Elf attacks, connects | |
| 16 | Infect creature deals damage to creature → -1/-1 counters, no damage marked | Glistener Elf blocked by Bears, Bears get -1/-1 counters | |
| 17 | 10 poison counters → player loses (full game loop) | Glistener Elf + pump spell (Giant Growth from L07) | |
| 18 | Hexproof creature can't be targeted by opponent's spell | Slippery Bogle + opponent's Lightning Bolt → targeting fails | |
| 19 | Hexproof creature CAN be targeted by controller's spell | Slippery Bogle + controller's Giant Growth → succeeds | |
| 20 | Protection from red blocks red spell targeting | Pro-red creature + Lightning Bolt → targeting fails | |
| 21 | Swampwalk creature unblockable when defender controls Swamp | Bog Wraith + defender has Swamp | |
| 22 | Planeswalker takes combat damage → loyalty removed → 0 loyalty → GY | Test PW attacked by creature | |

---

### Cross-Cutting: Mana Restrictions (T12d)

| Ticket | Cards Unlocked | Example Cards | Status |
|--------|---------------|---------------|--------|
| T12d | Restricted-mana lands, mana grants | Cavern of Souls (creature-only mana + uncounterable grant); Boseiju, Who Shelters All (instant/sorcery + uncounterable) | |

---

## Part 2: Phase 5 Continuous Effects & Layer System

### Sub-Plan 5A: Foundation + P/T (L01–L08)

| Ticket(s) | Cards Unlocked | Example Cards | Status |
|-----------|---------------|---------------|--------|
| L07 | P/T buff/debuff spells | Giant Growth ({G} +3/+3 UntilEOT) | 🃏 (in plan) |
| L08 | Static P/T anthem enchantments | Glorious Anthem ({1}{W}{W} creatures you control +1/+1) | 🃏 (in plan) |

**Gate 3 tests are defined in L07/L08 tickets** — Giant Growth on Bears = 5/5, Anthem on Bears = 3/3, etc.

---

### Sub-Plan 5B: Remaining Layers + Dependency (L09–L16)

| Ticket(s) | Cards Unlocked | Example Cards | Status |
|-----------|---------------|---------------|--------|
| L09 | Ability-granting/removing effects | Any "gains flying" spell; Humility (L19) | |
| L10 (+ T05) | Type-changing, color-changing, SetLandType effects | Blood Moon, Urborg (L17) | |
| L11 (+ T09) | Control-change effects | Mind Snare / Control Magic (L17) | |
| L12 | Text-changing effects | Mind Bend, Sleight of Mind (deferred — infrastructure ready) | |
| L14 | Dependency detection for all layer interactions | Blood Moon + Urborg interaction (L17/L20) | |
| L15 | Player action restrictions, cost modification scaffolding, `lands_per_turn` | Exploration (+1 land/turn); Thalia (cost increase) — scaffolding only | |
| L16 | All-zone static abilities | Dryad Arbor-style cards (forward-compatible) | |

---

### Sub-Plan 5C: Cards + Testing (L17–L21)

| Ticket | Cards | Status |
|--------|-------|--------|
| L17 | Honor of the Pure, Tarmogoyf, Urborg, Blood Moon, Mind Snare | 🃏 (in plan) |
| L18 | LKI system (no cards — infrastructure) | |
| L19 | Humility, Opalescence | 🃏 (in plan) |
| L20 | 27+ integration tests | 🧪 (in plan) |
| L21 | Fuzz regression with all Phase 5 cards | 🧪 (in plan) |

---

## Card Registry Expansion Tracker

Cards currently in registry (24): 5 basic lands, 5 spells (alpha.rs), 4 vanilla creatures, 11 keyword creatures.

### Cards to Add — Priority Order

Priority is based on: (1) exercises the most tickets, (2) catches the most integration bugs, (3) stays in the registry permanently for fuzz/regression.

#### Immediately Buildable (T01–T14 done, existing primitives)

| Card | Type | Key Tickets Exercised | Implemented? |
|------|------|-----------------------|-------------|
| Isamaru, Hound of Konda | {W} 2/2 Legendary Creature — Dog | T14 (legend rule) | |
| Night's Whisper | {1}{B} Sorcery — draw 2, lose 2 life | Sequence(DrawCards, LoseLife) | |
| Lava Spike | {R} Sorcery — deal 3 damage to target player | DealDamage, player-only target | |
| Divination | {2}{U} Sorcery — draw 2 cards | DrawCards, sorcery timing | |
| Heroes' Reunion | {G}{W} Instant — gain 7 life | GainLife, multicolor | |
| Negate | {1}{U} Instant — counter target noncreature spell | CounterSpell + type restriction | |

#### After T15b (Aura lifecycle)

| Card | Type | Key Tickets Exercised | Implemented? |
|------|------|-----------------------|-------------|
| Holy Strength | {W} Enchantment — Aura, +1/+2 | T04, T15, T15b | |

#### After T16 (indestructible, poison)

| Card | Type | Key Tickets Exercised | Implemented? |
|------|------|-----------------------|-------------|
| Darksteel Myr | {3} 0/1 Artifact Creature, Indestructible | T16 (indestructible SBA guard) | |

#### After T17 + T18 (casting pipeline)

| Card | Type | Key Tickets Exercised | Implemented? |
|------|------|-----------------------|-------------|
| Blaze | {X}{R} Sorcery, deal X damage | T06, T17, T18 (X-cost pipeline) | |
| Mind Spring | {X}{U}{U} Sorcery, draw X cards | T06, T17, T18 (X-cost pipeline) | |

#### After T21c (infect/toxic) + T22 (hexproof/protection)

| Card | Type | Key Tickets Exercised | Implemented? |
|------|------|-----------------------|-------------|
| Glistener Elf | {G} 1/1 Creature — Elf Warrior, Infect | T21c, T01, T02, T16 (poison SBA) | |
| Slippery Bogle | {G/U} 1/1 Creature, Hexproof | T22 (hexproof targeting) | |

#### After CreateToken primitive is implemented

| Card | Type | Key Tickets Exercised | Implemented? |
|------|------|-----------------------|-------------|
| Raise the Alarm | {1}{W} Instant, create two 1/1 tokens | T03, T13 (token cease-to-exist) | |

#### Medium Priority (exercises specific subsystems)

| Card | Type | Key Tickets Exercised | Implemented? |
|------|------|-----------------------|-------------|
| Vines of Vastwood | {G} Instant, kicker {G} (+4/+4 & hexproof if kicked) | T17, T18, T22 | |
| Goblin War Drums | — or test menace creature | T21b (menace evasion) | |
| Bog Wraith | {3}{B} 3/3 Creature, Swampwalk | T21b (landwalk evasion) | |
| Soltari Trooper | {1}{W} 2/1 Creature, Shadow | T21b (shadow evasion) | |
| Plague Stinger | {1}{B} 1/1 Creature, Flying Infect | T21c + T21b (infect + evasion combo) | |
| Test Planeswalker | Custom 3-loyalty PW with +1/-3 abilities | T14 (loyalty SBA), T21c (PW damage) | |

#### Lower Priority (forward-compatible, nice to have)

| Card | Type | Key Tickets Exercised | Implemented? |
|------|------|-----------------------|-------------|
| Pro-Red Knight | {W}{W} 2/2, Protection from Red | T22 (protection targeting) | |
| Invisible Stalker | {1}{U} 1/1 Hexproof, can't be blocked | T22 + T21b | |
| Troll Ascetic | {1}{G}{G} 3/2 Hexproof, Regenerate (regen deferred) | T22 | |

---

## Integration Test Checkpoints

### Checkpoint A: Now (T13 + T14 done)

**Prerequisites:** T01–T06, T09–T12b, T13, T14 all done.
**Cards to add:** Isamaru, Night's Whisper, Lava Spike, Divination, test planeswalker.
**Tests:** #1–#3 from Tier 3 table above.
**Goal:** Legend rule end-to-end, PW loyalty lifecycle, counter annihilation. Also backfill simple spell cards (draw, burn, life gain) to grow the fuzz pool.

### Checkpoint B: After T15b + T16

**Prerequisites:** T15, T15b, T16 done.
**Cards to add:** Holy Strength, Darksteel Myr.
**Tests:** #4–#7 from Tier 3 table above.
**Goal:** Aura attachment lifecycle, indestructible guard, token SBA (if CreateToken is available).

### Checkpoint C: After T17 + T18

**Prerequisites:** T17, T18 done.
**Cards to add:** Blaze, Mind Spring, Vines of Vastwood, test modal spell.
**Tests:** #8–#13 from Tier 4 table above.
**Goal:** Verify the 601.2 casting pipeline handles X-cost, kicker, modes, and restrictions end-to-end.

### Checkpoint D: After T21a–T22

**Prerequisites:** T21a–T22 done.
**Cards to add:** Glistener Elf, Slippery Bogle, menace creature, Bog Wraith, test PW.
**Tests:** #14–#22 from Tier 5 table above.
**Goal:** Verify infect→poison→SBA chain, hexproof/protection targeting, evasion, PW damage routing.

### Checkpoint E: Gate 2 Verification — full Part 1 regression

**Prerequisites:** All Part 1 tickets done + checkpoints A–D passing.
**Actions:** Re-run full `cargo test` + 500 fuzz games.
**Goal:** Green light for Part 2.

### Checkpoint F: L20/L21 — Phase 5 integration (already in plan)

**Prerequisites:** L17–L19 done.
**Tests:** 27+ integration tests defined in L20 + 500 fuzz games in L21.
**Goal:** Full layer system + dependency detection + complex card interactions verified.

---

## Running Totals

| Metric | Baseline (post-T14) | After CP-A | After CP-B | After CP-C | After CP-D | After Gate 2 | After Gate 5 |
|--------|---------------------|------------|------------|------------|------------|--------------|-------------|
| Cards in registry | 24 | ~30 | ~33 | ~37 | ~43 | ~43 | ~51+ |
| Unit tests | 296 | ~296 | | | | ~420 est. | ~504+ est. |
| Integration tests | 48 | ~51 | ~55 | ~61 | ~70 | ~70 | ~97+ |
| Fuzz games | 500 | — | — | — | — | 500 | 500+ |

---

## Notes

- **Test planeswalker:** No real PW is simple enough for early testing. Create a test-only `TestPlaneswalker` card: {2}{U}{U}, 4 loyalty, +1: draw a card, -3: counter target spell. Register as "Test Planeswalker" in the registry. This avoids needing the full PW activation pipeline (which requires Phase 6 for loyalty ability activation as a special action) — for now, just test ETB loyalty counters + combat damage → loyalty removal → 0 loyalty SBA.
- **Raise the Alarm requires token creation primitive.** If `Primitive::CreateToken` isn't implemented yet, defer this card until it is. The token cease-to-exist SBA (T13) can be unit-tested without a real token-creating card.
- **Aura casting requires T15b.** Holy Strength needs the full Aura attachment-on-resolve path from T15b, not just the SBA from T15.
- **Cards stay in the registry permanently.** Once added, they become part of the fuzz pool and regression suite. Choose cards that are simple enough to not break the random player but complex enough to exercise the new systems.
