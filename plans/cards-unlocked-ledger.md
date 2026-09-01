<!-- Ticket vocabulary: `L##` / `T##` / `SPECIAL-#` ids are defined in
     `plans/archive/implementation-plan-final.md`. That file is historical and must not
     be acted on — it is cited here only as the dictionary for these ids. -->

# Cards & Integration Tests Unlocked Ledger

> Tracks which real Magic cards become implementable after each ticket, and which integration test checkpoints should verify end-to-end behavior.
>
> **Purpose:** Catch cross-ticket integration bugs early via rolling card implementation + integration tests, rather than deferring all integration testing to L20/L21.
>
> **Status key:** ✅ = ticket done, 🃏 = card implemented & registered, 🧪 = integration test written, 📋 = designed in a `plans/` doc, nothing built

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
| Sol Ring | {1} Artifact — `{T}: Add {C}{C}` | {1} | ProduceMana; **the pool's first artifact**, which is what gives March of the Machines (Layer 7b) something to animate in fuzz decks | 🃏 2026-08-24 (`cards/artifacts.rs`) |
| Merfolk Thaumaturgist | {2}{U} Creature 1/2 — `{T}: Switch target creature's power and toughness until end of turn` | {2}{U} | SwitchPowerToughness (Layer 7d) and **the registry's first activated ability** — stack path, target selection, tap cost under CR 302.6 | 🃏 2026-08-24 (`cards/utility_creatures.rs`) |

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
| L11 (+ T09) | Control-change effects | Act of Treason ✅ (2026-08-23); Control Magic / Mind Control still need an "enchanted permanent" `AffectedSet` | "Mind Snare" was an invented name — see ATOM-613.1b-001's corpus correction |
| L12 | Text-changing effects | Mind Bend, Sleight of Mind (deferred — infrastructure ready) | |
| L14 | Dependency detection for all layer interactions | Blood Moon + Urborg interaction (L17/L20) | |
| L15 | Player action restrictions, cost modification scaffolding, `lands_per_turn` | Exploration (+1 land/turn); Thalia (cost increase) — scaffolding only | |
| L16 | All-zone static abilities | Dryad Arbor-style cards (forward-compatible) | |

---

### Sub-Plan 5C: Cards + Testing (L17–L21)

| Ticket | Cards | Status |
|--------|-------|--------|
| L17 | Honor of the Pure, Tarmogoyf, Urborg, Blood Moon (~~Mind Snare~~ — invented name, corrected in the L11 row above) | 🃏 (in plan) |
| L18 | LKI system (no cards — infrastructure) | |
| L19 | Humility, Opalescence | 🃏 (in plan) |
| L20 | 27+ integration tests | 🧪 (in plan) |
| L21 | Fuzz regression with all Phase 5 cards | 🧪 (in plan) |

---

---

## Part 3: Replacement Effects (Phases RA–RE)

Ticket ids here are the RA/RB/... sub-phases of `plans/replacement-architecture.md`
§9, not the `L##`/`T##` vocabulary above.

### Phase RA — the event spine — ✅ 2026-08-25

**Unlocks no cards by itself**, and that is the point: RA made every observable
mutation a proposal without changing what any of them do.

### Phase RB — the CR 616.1 pipeline — ✅ 2026-08-26

| Ticket | Cards Unlocked | Example Cards | Status |
|---|---|---|---|
| RB item 5 | **Every card that puts a shield, stun or finality counter on a permanent — **164** printed cards, verified on Scryfall 2026-08-26 (92 + 31 + 41). No card text at all: CR 122.1c/d/h state the effects and `engine::replacement::gather` synthesizes them from the counter | Stun (92): Unstoppable Slasher, Mjölnir, Storm Hammer. Shield (31): Titan of Industry, Elspeth Resplendent. Finality (41): Meathook Massacre II, Scavenger's Talent | 🧪 2026-08-26 — the *mechanic* is covered end to end; **no card registered**, because each of these needs a primitive that puts the counter on |
| RB item 6 | **Regeneration** — CR 701.19a shields from a resolving ability, CR 701.19b static regeneration, CR 701.19c "can't be regenerated" | Drudge Skeletons, Mossbridge Troll, Wall of Bone; the "it can't be regenerated" clause on hundreds of removal spells | 🧪 2026-08-26 — `Primitive::Regenerate` and `Primitive::CantBeRegenerated` exist; no card registered |
| RB item 7 | **Kalitas, Traitor of Ghet** and the two-sided-filter shape it stands for | Kalitas | 🃏 2026-08-26 (`cards/phase_rb_cards.rs`); registered 2026-08-29 into the stress pool — see the note below |
| RB items 8–9 | **Commander zone redirection**, both halves: CR 704.6d (graveyard/exile, a state-based action) and CR 903.9b (hand/library, a replacement) | Every commander in every Commander deck | 🧪 2026-08-26 — reachable only once something sets `GameObject.is_commander`, which is CR 903.7's setup hook and still missing |
| RB — vocabulary | Four stubbed primitives implemented as a side effect: `Tap`, `AddCounters`, `RemoveCounters`, `CreateToken` | **Raise the Alarm is now buildable** — the Notes below flagged it as blocked on `Primitive::CreateToken` | |

**Kalitas is registered, and the pool it moves is not the one that matters.**
It was held out of the registry at first on the grounds that registering a card
moves the `fuzz_games` baseline. That was an argument for splitting the pool,
not for keeping the card out: an unregistered card is invisible to `fuzz_games`,
to `card_pool_lowering_test` and to `cli_play` at once. Since 2026-08-29 there
are two pools — `PERFORMANCE_POOL`, the representative board an engine change is
A/B'd against, and the stress pool that `--pool stress` plays. Kalitas is in the
second only, and was in neither's way: 50 stress games at seed 12345 produce 202
Zombie tokens and no panics. (`PERFORMANCE_POOL` was *frozen* until 2026-09-01
and now grows one card per new engine path — `engineering-practices.md` §3.)
**Register every card you write** (`plans/engineering-practices.md` §3);
`PERFORMANCE_POOL` is what protects the baseline.

**What RB did not unlock, despite appearances.** CR 615's prevention machinery
is Phase RD, not this one. RB has `Rewrite::Prevent` and CR 122.1c's prevention
half; it has no damage shields, no prevention amounts, and no redirection — so
Fog, Healing Salve, Circle of Protection and the whole "prevent the next N
damage" family stay blocked.

### Phase RC — ETB replacements

**The largest single entry this ledger will take (~1,350 cards).** RC ships as
four PRs; this table grows one row each.

| Ticket | Cards Unlocked | Example Cards | Status |
|---|---|---|---|
| RC-1 | **No cards.** A pure deletion — the early stack pop — with no new behaviour | — | ✅ shipped 2026-09-01 |
| RC-2 | **"Enters tapped" (CR 110.5b, 773 printed cards) and "enters with counters" (CR 122.6a, 580)**, for the `AffectedSet::SourceOnly` case — the permanent's own ability, about itself, which CR 614.12's first sentence names explicitly. That is the overwhelming majority of both populations | 🃏 **Idyllic Beachfront** and **Chainbreaker** (both in `PERFORMANCE_POOL`), plus 🃏 **Adaptive Shimmerer** in the stress pool — a 0/0, which is the only board state that dies if CR 122.6a's counters arrive after the entry is observable. Also newly correct without card work: every planeswalker (CR 306.5b's loyalty is an entry replacement now) and every token (CR 111.1 makes a token's entry a proposal like any other) | ✅ shipped 2026-09-01 |
| RC-3 | **The *other* population — an entry-modifying effect that is not on the entering permanent.** Root Maze, Kismet, Loxodon Gatekeeper, Frozen Aether, and every Clone against Dress Down. Blocked today by one gate in `compute.rs`, not by missing vocabulary | Root Maze, Kismet, Dress Down | 📋 designed |
| RC-4 | CR 614.12's look-ahead frame, 614.12a's choice-before-entry, and the CR 616.1b/c buckets | Grist the Hunger Tide, Master Biomancer, the Theros gods | 📋 designed; unblocked by RS-1 |

**What RC-2 did not unlock, and it is worth naming.** CR 616.1's
multi-candidate branch is still unreachable in a game — measured 2026-09-01, the
same way RB's Kalitas gap was. Two applicable effects on one entry needs either
two entry-modifying abilities on one card (the printed population is five cards,
each needing {X}, a condition or a trigger) or an `AffectedSet::Filter` effect,
which cannot match an object that is not on the battlefield yet. **That is
RC-3's row above**, and it is the strongest argument for doing RC-3 next.

---

## Part 4: "Can't" Effects (Phases RS-1–RS-4)

Ticket ids are the `RS` sub-phases of `plans/cant-effects-architecture.md` §7.
Counts are clause counts from `plans/references/cant-census.py` (2026-08-27,
`o:"can't" -is:funny`: 1,857 cards / 2,034 clauses) plus the keyword
populations §2.2 measures separately — **the keyword numbers are the larger
half and the census cannot see them**, because those cards never print the word.

| Ticket | Cards Unlocked | Example Cards | Status |
|---|---|---|---|
| RS-0 | **No cards.** Lifts the ten methods `ContinuousEffectRegistry` and `ReplacementEffectRegistry` already duplicate into one shared duration registry, so RS-1 uses it rather than inventing a third | — | ✅ shipped 2026-08-31 (§9 finding 7). `ReplacementEffectRegistry` came out a **type alias**, not the wrapper the finding sketched |
| RS-1 | **Tier 2 — event-time "can't" (CR 614.17).** 236 clauses / 220 cards printed, plus **indestructible** (524 cards with or granting it) moved off its hardcoded arm onto the model. Tier 3's regeneration half (155 clauses) came with it | ✅ **Sigarda, Host of Herons** and **Diabolic Edict** registered — Tajuru Preserver prints Sigarda's sentence verbatim and is a rename away. Still waiting on other phases: Solemnity and Melira (RC — CR 113.6i makes them fire *as the object enters*); Platinum Emperion and Teferi's Protection (RE — `EventPattern` has no player-scoped arms); Abyssal Persecutor and Platinum Angel (RE's `PlayerLoses`); Skullcrack (its life-gain half is RE, its prevention half RD) | ✅ shipped 2026-08-31; **RC-4 unblocked** |
| RS-2 | **Tiers 1b/1c/1d — casting, activating, targeting.** 206 clauses / ~200 cards, plus **hexproof (336), shroud (35), protection (197)** — the whole `T22` ticket | Slippery Bogle, Troll Ascetic, Progenitus; Grafdigger's Cage's cast half, Drannith Magistrate, Rule of Law, Aggressive Mining, Conduit of Worlds, Rakdos Lord of Riots; Pithing Needle, Cursed Totem, Stony Silence | 📋 designed 2026-08-27 |
| RS-3a | **Tier 1a — combat, the predicate half** (`T21b`). **1,267 of 1,277** Tier-1a clauses: 1,219 per-creature restrictions plus 48 per-attacker blocker counts — plus **menace (405), landwalk (122), the fear/intimidate/shadow/skulk/horsemanship family (147)**, protection's blocking half, and Defender re-expressed as data | Goblin War Drums, Bog Wraith, Soltari Trooper; the 457 turn-scoped "target creature can't block this turn" effects; Alpha Authority | 📋 designed 2026-08-27; does **not** need CR 613.8 |
| RS-3b | **Tier 1a — the CR 508.1d/509.1c solver** (`T21d`). The remaining **13** cross-creature clauses, **2** global-cap cards, and the ~**150** *requirement* cards ("attacks each combat if able", goad) that CR 508.1d makes inseparable from them | Silent Arbiter, Dueling Grounds, Bonded Construct, Orcish Conscripts; goad (83 cards) and every Commander deck that runs it | 📋 designed 2026-08-27; **wants CR 613.8 first** (evasion is cumulative) |
| RS-4 | **Tier 1e — cost payment (CR 614.17b).** 16 printed clauses; the rest is *derived* from RS-1 through a 10-arm projection over the closed `Cost` enum | Yasharn Implacable Earth, Angel of Jubilation, Karn's Sylex — and Platinum Emperion's cost half, which the card never states | 📋 designed 2026-08-27 |

**What this does not unlock.** Tier 3's other half — "damage can't be
prevented" (32 clauses: Skullcrack, Banefire, Stomp) — needs Phase RD's
prevention machinery to have something to withhold. CR 113.11's "can't have or
gain [ability]" (the Archetype cycle, 6 cards) is the **layer system's** by
CR 101.2a and is not in any RS phase. And the 149 "can't … unless" clauses
— 138 of them combat — need `Effect::Conditional`, which is Phase 6.

**Read the keyword column carefully.** RS-2 and RS-3a unlock more cards than
their clause counts suggest and RS-1 unlocks fewer: indestructible is one
keyword and 524 cards, while Tier 2's 236 printed clauses are spread across a
dozen mechanics that mostly need other phases anyway (`PlayerLoses` is RE's,
`EnterBattlefield` is RC's).

---

## Part 5: Copy Effects (Phases CV-1–CV-7)

Ticket ids are the `CV` sub-phases of `plans/copy-effects-architecture.md` §7.
Counts are from `plans/references/copy-census.py` (2026-08-29). **Two different
units, never summed:** the copy *clauses* (1,093 over 752 cards printing
"copy"/"copies"/"copied") and the face/state *card* populations, which never
print the word and which the clause census structurally cannot see.

| Ticket | Cards Unlocked | Example Cards | Status |
|---|---|---|---|
| CV-1 | **Tier C — "becomes a copy" (CR 707.4), turn-bounded.** **56 of 81 clauses**. Also the spine every later CV phase carries: `CopiableValues`, `EffectModification::CopyFrom`, `Primitive::Copy`, and the two ETB-scan legs (`codebase-state.md` item 16) | **Cytoshape** (instant, `AffectedSet::Fixed` by CR 611.2c, until end of turn); Mirrorweave and Nanogene Conversion, which are also `Fixed` once resolved; a Clone-of-an-Anthem probe, the only way to see the `register_static_effects` hole | 📋 designed 2026-08-29, boundary corrected 2026-08-30 |
| CV-1b | **The indefinite-duration copies.** **25 of 81** Tier C clauses, printed by name by `copy-census.py --scope`. A row that never expires is reachable by neither expiry nor `remove_by_source`, so it outlives its subject without bound | **Dimir Doppelganger**, Lazav the Multifarious, Likeness Looter (activated, self-scoped, capture from a graveyard *card*); True Polymorph, Metamorphic Alteration | 📋 designed 2026-08-29; **blocked on `codebase-state.md` item 10 (CR 400.7)** — and unlike a pump spell's, this exposure is unbounded |
| CV-2 | **Tier B — "enters as a copy" (CR 707.5/616.1c).** 69 clauses / 69 cards, and it gives `ReplacementClass::CopyOnEnter` the producer RB shipped the bucket for. Carries CR 707.9a–d's exceptions: **139 of the 331** printed "as a copy" cards say "except" | Clone, Phantasmal Image, Sakashima of a Thousand Faces, Spark Double, Evil Twin; every Clone-shaped legend a Commander deck runs | 📋 designed 2026-08-29; **needs RC-2**, and takes CR 616.1c off RC-4 |
| CV-3 | **Tier A — token copies (CR 707.1/111.1).** 318 clauses / **306 cards**, the largest layer-touching bucket and the cheapest mechanism in the document — no row, no layer, no duration | Kiki-Jiki Mirror Breaker, Helm of the Host, Splinter Twin, Esika's Chariot; the 266 cards printing "token that's a copy", 256 of them Commander-legal | 📋 designed 2026-08-29 |
| CV-4 | **Tiers D+E — spell and cast copies (CR 707.10/707.12).** 542 + 73 clauses over **306 + 47 cards**, and **it touches no layer, no registry and no replacement**. 186 of those clauses are CR 707.10c's retarget prompt alone — one shared prompt, not 186 implementations | Fork, Twincast, Reverberate, Dualcaster Mage; Zada Hedron Grinder and Ink-Treader Nephilim (707.10d); Isochron Scepter and Panoptic Mirror (707.12) | 📋 designed 2026-08-29; **free to land at any point** |
| CV-5 | **CR 712 faces — transform + modal DFC.** **496 cards** (396 nonmodal + 100 modal), 486 Commander-legal, and **120 of them can *be* a commander**. Gives `ReplacementClass::BackFaceUp` its 616.1d producer | Delver of Secrets, Huntmaster of the Fells, Withengar Unbound; the modal DFC lands (Agadeem's Awakening, Turntimber Symbiosis); Brutal Cathar as a DFC commander | 📋 designed 2026-08-29; **highest risk — a second card model** |
| CV-6 | **Face-down (CR 708).** **304 producers** — Layer 1b plus the CR 708.4 cast-face-down path and the turn-face-up special action. Coupled to CV-1 only through the capture ceiling (CR 707.2 makes copiable values depend on face-down status) | Willbender, Brine Elemental, Den Protector; the manifest and disguise/cloak families | 📋 scheduled 2026-08-30, **undesigned** — no type surface yet |
| CV-7 | **Merging (CR 729) + meld (CR 712.4).** 34 + 21 cards, and the multi-component `BattlefieldEntity` every other phase writes code against the absence of | Brisela Voice of Nightmares (Bruna + Gisela); Illuna Apex of Wishes and Nethroi Apex of Death (mutate); Urza Lord Protector, which melds | 📋 scheduled 2026-08-30, **undesigned**; **back-stop: before Phase 8 card breadth** |

**Nothing in this cluster is out of v1** (§6, revised 2026-08-30 — an earlier
draft scoped meld, mutate, face-down and flip cards out on population and
effort, and that is withdrawn). Two reasons, and the first is the project's own
doctrine: "is this permanent one object or several?" is a **fact**, not a
feature, so deferring CV-7 does not defer its cost — every phase in between
writes more code against the single-component assumption. And 21 meld cards is
not 21 cards of demand when one of them is Brisela. **Flip cards** (CR 710, 25
cards) ride along in CV-5 once faces exist.

**Cards is the sizing unit; the clause counts are provenance.** A copy card
carries one mechanism plus rider sentences about it — unlike a "can't" card,
which can carry several independent restrictions — so clauses count sentences,
not work. CV-4 is the extreme: 542 clauses over 306 cards, because 186 of those
are one shared CR 707.10c prompt. And in the other direction, CV-5's 496 cards
print no copy clause at all, which is why §2.2's population table exists.

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
- ~~**Raise the Alarm requires token creation primitive.**~~ — discharged 2026-08-26. `Primitive::CreateToken` landed with Phase RB item 7 (Kalitas's rider needed it). Note what it is *not*: token creation is not yet a `GameAction`, so CR 614.16's doublers have nothing to replace — that is Phase RE. A token's *entering* is not a proposal either; that is Phase RC.
- **Aura casting requires T15b.** Holy Strength needs the full Aura attachment-on-resolve path from T15b, not just the SBA from T15.
- **Cards stay in the registry permanently.** Once added, they become part of the fuzz pool and regression suite. Choose cards that are simple enough to not break the random player but complex enough to exercise the new systems.
