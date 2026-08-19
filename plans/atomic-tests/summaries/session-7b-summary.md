# Session 7B Summary: Keyword Abilities (702.1–702.80)

> **Source:** `session-7b.md` (audited via `session-7b-audit-response.md`)
> **Scope:** Rules 702.1 through 702.80 (Deathtouch through Wither)
> **Date:** 2026-04-08

---

## 1. ATOM Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-702.2d-001 | 702.2d | Deathtouch from non-battlefield zone | Phase 6 | DEFERRED | deathtouch, any-zone |
| ATOM-702.2e-001 | 702.2e | Deathtouch LKI after zone change | Phase 5 | T20b | deathtouch, LKI |
| ATOM-702.4c-001 | 702.4c | Remove double strike mid-combat stops 2nd step | Phase 5 | DEFERRED | double-strike, mid-combat, continuous-effects |
| ATOM-702.4d-001 | 702.4d | Grant double strike to first-striker after 1st step | Phase 5 | DEFERRED | double-strike, first-strike, mid-combat, continuous-effects |
| ATOM-702.5a-001 | 702.5a | Aura targeting restricted by enchant ability | Phase 5-Pre | T15b | enchant, aura, targeting |
| ATOM-702.5c-001 | 702.5c | Multiple enchant instances compose via AND | Phase 8 | DEFERRED | enchant, aura, multiple-instances |
| ATOM-702.5d-001 | 702.5d | Enchant player Aura can't target permanents | Phase 5-Pre | T15b | enchant, aura, enchant-player |
| ATOM-702.6a-001 | 702.6a | Equip activation, attachment, sorcery-speed | Phase 5-Pre | T15b | equip, attachment, sorcery-speed |
| ATOM-702.6a-002 | 702.6a | Equip targets only own creatures | Phase 5-Pre | T15b | equip, targeting, controller |
| ATOM-702.6a-003 | 702.6a | Equip sorcery-speed enforcement (negative) | Phase 5-Pre | T15b | equip, sorcery-speed, negative-case |
| ATOM-702.6c-001 | 702.6c | Equip with quality restriction | Phase 8 | DEFERRED | equip, quality-restriction |
| ATOM-702.6c-002 | 702.6c | Non-equip attachment bypasses quality restriction | Phase 8 | DEFERRED | equip, quality-restriction, non-equip-attachment |
| ATOM-702.6d-001 | 702.6d | Multiple equip abilities coexist independently | Phase 8 | DEFERRED | equip, multiple-abilities, quality-restriction |
| ATOM-702.6e-001 | 702.6e | Equip planeswalker variant | Phase 8 | DEFERRED | equip, planeswalker |
| ATOM-702.7c-001 | 702.7c | Gain first strike after 1st step doesn't block 2nd | Phase 5 | DEFERRED | first-strike, mid-combat, continuous-effects |
| ATOM-702.7c-002 | 702.7c | Remove first strike after 1st step doesn't grant 2nd | Phase 5 | DEFERRED | first-strike, mid-combat, continuous-effects |
| ATOM-702.8a-001 | 702.8a | Flash bypasses sorcery-speed timing | Phase 5-Pre | T18 | flash, timing, instant-speed |
| ATOM-702.8a-002 | 702.8a | Flash from non-hand zone | Phase 8 | DEFERRED | flash, any-zone |
| ATOM-702.11b-001 | 702.11b | Hexproof blocks opponent targeting | Phase 5-Pre | T22 | hexproof, targeting |
| ATOM-702.11b-002 | 702.11b | Hexproof allows self-targeting | Phase 5-Pre | T22 | hexproof, self-targeting |
| ATOM-702.11c-001 | 702.11c | Hexproof on player | Phase 8 | DEFERRED | hexproof, player |
| ATOM-702.11d-001 | 702.11d | Hexproof from [quality] variant | Phase 8 | DEFERRED | hexproof, hexproof-from, quality |
| ATOM-702.11e-001 | 702.11e | Losing hexproof strips all hexproof-from | Phase 8 | DEFERRED | hexproof, hexproof-from, ability-removal |
| ATOM-702.12b-001 | 702.12b | Indestructible prevents lethal damage destruction | Phase 5-Pre | T09 | indestructible, lethal-damage, SBA |
| ATOM-702.12b-002 | 702.12b | Indestructible prevents destroy effects | Phase 5-Pre | T09 | indestructible, destroy-effect |
| ATOM-702.13b-001 | 702.13b | Intimidate evasion: artifact or shared color | Phase 8 | DEFERRED | intimidate, evasion, blocking |
| ATOM-702.14c-001 | 702.14c | Landwalk evasion: unblockable if defender has land | Phase 8 | DEFERRED | landwalk, evasion, blocking |
| ATOM-702.14c-002 | 702.14c | Landwalk inapplicable without matching land | Phase 8 | DEFERRED | landwalk, evasion, blocking |
| ATOM-702.14d-001 | 702.14d | Landwalk abilities don't cancel each other | Phase 8 | DEFERRED | landwalk, evasion, no-cancel |
| ATOM-702.15c-001 | 702.15c | Lifelink LKI after zone change | Phase 5 | T20b | lifelink, LKI |
| ATOM-702.15d-001 | 702.15d | Lifelink from non-battlefield zone | Phase 6 | DEFERRED | lifelink, any-zone |
| ATOM-702.16a-001 | 702.16a | Protection ability exists and is queryable | Phase 5-Pre | T22 | protection, ability-query |
| ATOM-702.16a-002 | 702.16a | Protection from [cardname] quality matching | Phase 8 | DEFERRED | protection, protection-from-cardname |
| ATOM-702.16a-003 | 702.16a | Protection from snow (all source types) | Phase 8 | DEFERRED | protection, protection-from-snow, supertype, snow-spell |
| ATOM-702.16b-001 | 702.16b | Protection targeting restriction (matching) | Phase 5-Pre | T22 | protection, targeting |
| ATOM-702.16b-002 | 702.16b | Protection doesn't block non-matching quality | Phase 5-Pre | T22 | protection, targeting, non-matching |
| ATOM-702.16c-001 | 702.16c | Protection causes illegal Auras to fall off (SBA) | Phase 5-Pre | T22 | protection, aura, SBA |
| ATOM-702.16d-001 | 702.16d | Protection causes illegal Equipment to detach (SBA) | Phase 5-Pre | T22 | protection, equipment, SBA |
| ATOM-702.16e-001 | 702.16e | Protection prevents all damage from matching source | Phase 5-Pre | T22 | protection, damage-prevention, combat |
| ATOM-702.16f-001 | 702.16f | Protection evasion: can't be blocked by matching | Phase 5-Pre | T22 | protection, blocking, evasion |
| ATOM-702.16j-001 | 702.16j | Protection from everything | Phase 8 | DEFERRED | protection, protection-from-everything |
| ATOM-702.16k-001 | 702.16k | Protection from [player] | Phase 9 | DEFERRED | protection, protection-from-player, multiplayer |
| ATOM-702.16k-002 | 702.16k | Protection from player checks controller not owner | Phase 9 | DEFERRED | protection, protection-from-player, multiplayer, control-vs-ownership |
| ATOM-702.16n-001 | 702.16n | Self-exception Aura on protection | Phase 8 | DEFERRED | protection, aura, self-exception |
| ATOM-702.18a-001 | 702.18a | Shroud blocks all targeting including controller | Phase 5-Pre | T22 | shroud, targeting |
| ATOM-702.18a-002 | 702.18a | Shroud on player | Phase 8 | DEFERRED | shroud, player |
| ATOM-702.18a-003 | 702.18a | Shroud blocks opponent targeting | Phase 5-Pre | T22 | shroud, targeting, opponent |
| ATOM-702.19c-001 | 702.19c | Trample over planeswalkers | Phase 8 | DEFERRED | trample, trample-over-planeswalkers |
| ATOM-702.19d-001 | 702.19d | Trample all blockers removed → all to defender | Phase 4 | NEW-2 | trample, all-blockers-removed, regression |
| ATOM-702.19e-001 | 702.19e | Trample-over-PW with PW removed | Phase 8 | DEFERRED | trample, trample-over-planeswalkers, PW-removed |
| ATOM-702.19f-001 | 702.19f | Normal trample can't overflow to player when attacking PW | Phase 8 | DEFERRED | trample, planeswalker, no-overflow-to-player |
| ATOM-702.21a-001 | 702.21a | Ward trigger: counter unless pay | Phase 7 | NEW-1 | ward, triggered-ability, counter-unless-pay |
| ATOM-702.21a-002 | 702.21a | Ward doesn't trigger for controller's own targeting | Phase 7 | NEW-1 | ward, triggered-ability, self-targeting |
| ATOM-702.21b-001 | 702.21b | Ward X evaluated at resolution | Phase 8 | DEFERRED | ward, variable-cost, resolution-time |
| ATOM-702.23a-001 | 702.23a | Rampage triggered P/T bonus | Phase 8 | DEFERRED | rampage, triggered-ability |
| ATOM-702.24a-001 | 702.24a | Cumulative upkeep escalating cost + sacrifice | Phase 8 | DEFERRED | cumulative-upkeep, triggered-ability, age-counters |
| ATOM-702.25a-001 | 702.25a | Flanking debuffs non-flanking blocker | Phase 8 | DEFERRED | flanking, triggered-ability |
| ATOM-702.26a-001 | 702.26a | Phasing event during untap step | Phase 8 | DEFERRED | phasing, untap-step, phase-out |
| ATOM-702.26b-001 | 702.26b | Phased-out permanents are invisible | Phase 8 | DEFERRED | phasing, phased-out, invisible |
| ATOM-702.26d-001 | 702.26d | Phasing is NOT a zone change | Phase 8 | DEFERRED | phasing, no-zone-change, tokens, counters |
| ATOM-702.26g-001 | 702.26g | Indirect phasing of attached permanents | Phase 8 | DEFERRED | phasing, indirect-phasing, attachment |
| ATOM-702.27a-001 | 702.27a | Buyback additional cost + hand return | Phase 8 | DEFERRED | buyback, additional-cost, hand-return, T17 |
| ATOM-702.27a-002 | 702.27a | Buyback not paid → normal graveyard | Phase 8 | DEFERRED | buyback, no-buyback |
| ATOM-702.28b-001 | 702.28b | Shadow bidirectional evasion | Phase 8 | DEFERRED | shadow, evasion, blocking |
| ATOM-702.29a-001 | 702.29a | Cycling discard-from-hand to draw | Phase 8 | DEFERRED | cycling, activated-ability, discard-draw |
| ATOM-702.29e-001 | 702.29e | Typecycling searches for specific type | Phase 8 | DEFERRED | cycling, typecycling, library-search |
| ATOM-702.30a-001 | 702.30a | Echo trigger: sacrifice or pay on first upkeep | Phase 8 | DEFERRED | echo, triggered-ability, sacrifice |
| ATOM-702.30a-002 | 702.30a | Echo re-triggers after control change | Phase 8 | DEFERRED | echo, control-change, triggered-ability |
| ATOM-702.31b-001 | 702.31b | Horsemanship evasion | Phase 8 | DEFERRED | horsemanship, evasion, blocking |
| ATOM-702.32a-001 | 702.32a | Fading ETB counters + upkeep removal + sacrifice | Phase 8 | DEFERRED | fading, counters, sacrifice |
| ATOM-702.33a-001 | 702.33a | Kicker additional cost + kicked status | Phase 8 | DEFERRED | kicker, additional-cost, T17 |
| ATOM-702.33c-001 | 702.33c | Multikicker repeated payments | Phase 8 | DEFERRED | kicker, multikicker |
| ATOM-702.33g-001 | 702.33g | Conditional targets based on kicked status | Phase 8 | DEFERRED | kicker, conditional-targets |
| ATOM-702.34a-001 | 702.34a | Flashback from graveyard + exile on leave-stack | Phase 8 | DEFERRED | flashback, alternative-cost, graveyard-cast, exile, T17 |
| ATOM-702.35a-001 | 702.35a | Madness replacement on discard + cast from exile | Phase 8 | DEFERRED | madness, replacement-effect, discard, exile-cast |
| ATOM-702.36b-001 | 702.36b | Fear evasion: artifact or black blocker | Phase 8 | DEFERRED | fear, evasion, blocking |
| ATOM-702.37a-001 | 702.37a | Morph casting: face-down 2/2 for {3} | Phase 8 | DEFERRED | morph, face-down, casting |
| ATOM-702.37e-001 | 702.37e | Morph face-up special action | Phase 8 | DEFERRED | morph, face-up, special-action |
| ATOM-702.38a-001 | 702.38a | Amplify ETB counters from revealed hand cards | Phase 8 | DEFERRED | amplify, counters, ETB |
| ATOM-702.39a-001 | 702.39a | Provoke forces block + untaps target | Phase 8 | DEFERRED | provoke, triggered-ability, forced-block |
| ATOM-702.40a-001 | 702.40a | Storm copies based on spells-cast-this-turn | Phase 8 | DEFERRED | storm, triggered-ability, copy, spell-count |
| ATOM-702.41a-001 | 702.41a | Affinity cost reduction by permanent count | Phase 8 | DEFERRED | affinity, cost-reduction, T17 |
| ATOM-702.42a-001 | 702.42a | Entwine additional cost enables all modes | Phase 8 | DEFERRED | entwine, modal, additional-cost |
| ATOM-702.43a-001 | 702.43a | Modular ETB counters + death trigger transfer | Phase 8 | DEFERRED | modular, counters, death-trigger |
| ATOM-702.43a-002 | 702.43a | Modular ETB counter placement | Phase 8 | DEFERRED | modular, counters, ETB |
| ATOM-702.44a-001 | 702.44a | Sunburst ETB counters by mana colors spent | Phase 8 | DEFERRED | sunburst, counters, mana-colors |
| ATOM-702.45a-001 | 702.45a | Bushido trigger on blocking/being blocked | Phase 8 | DEFERRED | bushido, triggered-ability |
| ATOM-702.46a-001 | 702.46a | Soulshift death trigger recovers Spirit | Phase 8 | DEFERRED | soulshift, death-trigger, spirit |
| ATOM-702.46a-002 | 702.46a | Soulshift MV cap (negative case) | Phase 8 | DEFERRED | soulshift, MV-cap, negative-case |
| ATOM-702.47a-001 | 702.47a | Splice adds rules text from hand to spell | Phase 8 | DEFERRED | splice, text-changing, additional-cost |
| ATOM-702.48a-001 | 702.48a | Offering sacrifice for cost reduction + instant speed | Phase 8 | DEFERRED | offering, sacrifice, cost-reduction |
| ATOM-702.49a-001 | 702.49a | Ninjutsu swap unblocked attacker for hand creature | Phase 8 | DEFERRED | ninjutsu, activated-ability, swap, ETB-attacking |
| ATOM-702.49a-002 | 702.49a | Ninjutsu after first-strike damage | Phase 8 | DEFERRED | ninjutsu, first-strike, timing |
| ATOM-702.49a-003 | 702.49a | Ninjutsu in end-of-combat (no damage) | Phase 8 | DEFERRED | ninjutsu, end-of-combat, timing |
| ATOM-702.50a-001 | 702.50a | Epic locks out casting + recurring copies | Phase 8 | DEFERRED | epic, casting-restriction, delayed-trigger |
| ATOM-702.51a-001 | 702.51a | Convoke tap creatures as mana payment | Phase 8 | DEFERRED | convoke, cost-modification, tap-creatures, T17 |
| ATOM-702.52a-001 | 702.52a | Dredge replace draw with mill + graveyard return | Phase 8 | DEFERRED | dredge, replacement-effect, mill, graveyard |
| ATOM-702.52b-001 | 702.52b | Dredge unavailable if library < N | Phase 8 | DEFERRED | dredge, library-size, boundary, negative-case |
| ATOM-702.53a-001 | 702.53a | Transmute hand-based tutor by MV | Phase 8 | DEFERRED | transmute, activated-ability, tutor |
| ATOM-702.54a-001 | 702.54a | Bloodthirst conditional ETB counters | Phase 8 | DEFERRED | bloodthirst, counters, ETB |
| ATOM-702.55a-001 | 702.55a | Haunt trigger exiles card haunting a creature | Phase 8 | DEFERRED | haunt, exile, haunting |
| ATOM-702.56a-001 | 702.56a | Replicate additional cost + triggered copies | Phase 8 | DEFERRED | replicate, additional-cost, copy |
| ATOM-702.57a-001 | 702.57a | Forecast hand-based upkeep-only once-per-turn | Phase 8 | DEFERRED | forecast, activated-ability, upkeep |
| ATOM-702.58a-001 | 702.58a | Graft ETB counters + trigger to share | Phase 8 | DEFERRED | graft, counters, triggered-ability |
| ATOM-702.59a-001 | 702.59a | Recover trigger: pay to recover or exile | Phase 8 | DEFERRED | recover, graveyard-trigger |
| ATOM-702.60a-001 | 702.60a | Ripple reveal + free-cast same-name | Phase 8 | DEFERRED | ripple, triggered-ability, free-cast |
| ATOM-702.61a-001 | 702.61a | Split second restricts casting/activation | Phase 8 | DEFERRED | split-second, stack, casting-restriction |
| ATOM-702.61b-001 | 702.61b | Split second exceptions: mana abilities, triggers | Phase 8 | DEFERRED | split-second, mana-abilities, triggers |
| ATOM-702.62a-001 | 702.62a | Suspend initial exile from hand (special action) | Phase 8 | DEFERRED | suspend, exile, special-action |
| ATOM-702.62a-002 | 702.62a | Suspend upkeep counter removal | Phase 8 | DEFERRED | suspend, upkeep-trigger, time-counters |
| ATOM-702.62a-003 | 702.62a | Suspend free cast on last counter removal + haste | Phase 8 | DEFERRED | suspend, free-cast, haste, last-counter |
| ATOM-702.63a-001 | 702.63a | Vanishing ETB counters + upkeep removal + sacrifice | Phase 8 | DEFERRED | vanishing, time-counters, sacrifice |
| ATOM-702.63b-001 | 702.63b | Vanishing with 0 counters (Solemnity): no sacrifice | Phase 8 | DEFERRED | vanishing, solemnity, zero-counters |
| ATOM-702.63b-002 | 702.63b | External counter restarts vanishing countdown | Phase 8 | DEFERRED | vanishing, external-counter, sacrifice |
| ATOM-702.64a-001 | 702.64a | Absorb prevents N damage per source | Phase 8 | DEFERRED | absorb, damage-prevention |
| ATOM-702.64a-002 | 702.64a | Absorb per-source with multiple sources | Phase 8 | DEFERRED | absorb, per-source, multiple-sources |
| ATOM-702.65a-001 | 702.65a | Aura swap exchange Aura with hand | Phase 8 | DEFERRED | aura-swap, exchange |
| ATOM-702.66a-001 | 702.66a | Delve exile graveyard cards for generic cost | Phase 8 | DEFERRED | delve, cost-modification, graveyard-exile, T17 |
| ATOM-702.67a-001 | 702.67a | Fortify attaches Fortification to land | Phase 8 | DEFERRED | fortify, fortification, attachment |
| ATOM-702.68a-001 | 702.68a | Frenzy trigger on unblocked attack | Phase 8 | DEFERRED | frenzy, triggered-ability |
| ATOM-702.69a-001 | 702.69a | Gravestorm copies based on permanents-died count | Phase 8 | DEFERRED | gravestorm, triggered-ability, copy |
| ATOM-702.70a-001 | 702.70a | Poisonous gives poison counters on combat damage | Phase 8 | DEFERRED | poisonous, poison-counters, triggered-ability |
| ATOM-702.71a-001 | 702.71a | Transfigure sacrifice-tutor for same MV creature | Phase 8 | DEFERRED | transfigure, sacrifice, tutor |
| ATOM-702.72a-001 | 702.72a | Champion ETB exile + LTB return | Phase 8 | DEFERRED | champion, ETB, LTB, exile, linked-abilities |
| ATOM-702.72a-002 | 702.72a | Champion no valid target → sacrifice | Phase 8 | DEFERRED | champion, no-valid-target, sacrifice |
| ATOM-702.72a-003 | 702.72a | Champion declines exile → sacrifice | Phase 8 | DEFERRED | champion, decline-exile, sacrifice |
| ATOM-702.73a-001 | 702.73a | Changeling grants all creature types (CDA) | Phase 8 | DEFERRED | changeling, CDA, creature-types |
| ATOM-702.73a-002 | 702.73a | Changeling works in all zones | Phase 8 | DEFERRED | changeling, CDA, all-zones |
| ATOM-702.74a-001 | 702.74a | Evoke alternative cost + ETB sacrifice | Phase 8 | DEFERRED | evoke, alternative-cost, ETB, sacrifice, T17 |
| ATOM-702.75a-001 | 702.75a | Hideaway ETB: peek + face-down exile | Phase 8 | DEFERRED | hideaway, ETB, face-down-exile |
| ATOM-702.76a-001 | 702.76a | Prowl conditional alternative cost | Phase 8 | DEFERRED | prowl, alternative-cost, creature-type |
| ATOM-702.77a-001 | 702.77a | Reinforce hand-based +1/+1 counters | Phase 8 | DEFERRED | reinforce, activated-ability, counters |
| ATOM-702.78a-001 | 702.78a | Conspire tap creatures + triggered copy | Phase 8 | DEFERRED | conspire, additional-cost, copy |
| ATOM-702.79a-001 | 702.79a | Persist death trigger: return with -1/-1 (LKI check) | Phase 8 | DEFERRED | persist, death-trigger, minus-counters, LKI |
| ATOM-702.79a-002 | 702.79a | Persist no trigger if had -1/-1 counters (LKI) | Phase 8 | DEFERRED | persist, no-trigger, minus-counters, LKI |
| ATOM-702.80a-001 | 702.80a | Wither: damage → -1/-1 counters on creatures | Phase 8 | DEFERRED | wither, minus-counters, damage-replacement |
| ATOM-702.80a-002 | 702.80a | Wither doesn't affect player damage | Phase 8 | DEFERRED | wither, player-damage |
| ATOM-702.80b-001 | 702.80b | Wither LKI after zone change | Phase 8 | DEFERRED | wither, LKI |

---

## 2. BOUNDARY-DEF Index

No BOUNDARY-DEF tests in this session.

---

## 3. COMP Index

| ID | Rule Pair | Summary | Composes | Phase | Ticket | Tags |
|----|-----------|---------|----------|-------|--------|------|
| COMP-702-001 | 702.2b + 702.7b | Deathtouch + first strike kills before blocker hits back | ATOM-702.2b (IMPL) + ATOM-702.7b (IMPL) | IMPL | — | deathtouch, first-strike |
| COMP-702-002 | 702.2b + 702.19b | Deathtouch + trample: 1 per blocker = lethal, rest overflows | ATOM-702.2b (IMPL) + ATOM-702.19b (IMPL) | IMPL | — | deathtouch, trample |
| COMP-702-003 | 702.4b + 702.15b | Double strike + lifelink: life gained in both steps | ATOM-702.4b (IMPL) + ATOM-702.15b (IMPL) | IMPL | — | double-strike, lifelink |
| COMP-702-004 | 702.9b + 702.17b | Flying + reach: reach can block flying | ATOM-702.9b (IMPL) + ATOM-702.17b (IMPL) | IMPL | — | flying, reach |
| COMP-702-005 | 702.16e + 702.19b | Protection + trample: damage prevented but trample still overflows | ATOM-702.16e-001 + ATOM-702.19b (IMPL) | Phase 5-Pre | T22 | protection, trample, damage-prevention |
| COMP-702-006 | 702.80a + 702.79a | Wither + persist: -1/-1 counters prevent persist retrigger | ATOM-702.80a-001 + ATOM-702.79a-001 | Phase 8 | DEFERRED | wither, persist |
| COMP-702-007 | 702.11b + 608.2b | Hexproof granted mid-stack → fizzle | ATOM-702.11b-001 + resolution rules | Phase 5-Pre | T22 | hexproof, targeting, fizzle |
| COMP-702-008 | 702.24a + Solemnity | CU + Solemnity: 0 age counters → free upkeep | ATOM-702.24a-001 + counter prevention | Phase 8 | DEFERRED | cumulative-upkeep, solemnity, counters |

---

## 4. META Entries

**META-7B-01: Unified Evasion Framework (NEW-3)**
Flying, Shadow, Fear, Intimidate, Horsemanship, and Landwalk all share the pattern "can't be blocked except by [filter]." Current engine hardcodes flying check in `validate_blockers`. Proposed: `EvasionRestriction` struct with `BlockerFilter` enum (`HasAnyKeyword`, `Bidirectional`, `ArtifactOrSharesColor`, `HasKeyword`, `ConditionalUnblockable`). Validation loop becomes `for evasion in get_evasion_restrictions(game, attacker_id)`. Implement when Phase 8 lands the first non-Flying evasion keyword.

**META-7B-02: ProtectionQuality Enum**
The CR uses "quality" informally for hexproof-from and protection-from. Proposed enum: `Color(Color)`, `CardType(CardType)`, `Subtype(SubtypeId)`, `CardName(String)`, `Everything`, `Player(PlayerId)`, `ManaValueAtMost(u32)`. A centralized `matches_quality()` function serves both hexproof-from and protection.

**META-7B-03: Copy-Spell vs Copy-Card**
Pattern A (Storm/Replicate/Conspire): copy spell directly on stack, not "cast," doesn't trigger cast triggers. Pattern B (rare exile effects): copy a card in a zone, grant cast permission, IS casting. Pattern C (Fork/Reverberate): copy target existing spell on stack. Engine needs `copy_spell_on_stack()` (Pattern A/C) and `create_card_copy()` (Pattern B).

**META-7B-04: Unified Trample DP (NEW-4)**
Proposed `TrampleContext` struct with `blockers`, `intermediates` (planeswalker loyalty / battle defense thresholds), and `final_target` (player). Normal trample: empty intermediates. Trample-over-PW: one intermediate with loyalty threshold. One DP method, one validation path.

**META-7B-05: Protection-from-Everything Is an Enum Variant, Not a Base Case**
`ProtectionQuality::Everything` where `matches_quality()` returns true unconditionally. Additive matching ("protection FROM [quality]") is the correct mental model, not subtractive filtering ("block everything except").

**META-7B-06: Haunting Is a Designation**
"Haunting" / "is haunted by" is a game-state relationship (like "paired" or "renowned"), not a characteristic (rule 109.3). Track as an association between exiled card and haunted permanent.

**META-7B-07: Splice Implementation Sketch**
Splice modeled as a temporary continuous effect adding rules text during casting (step 601.2b). Engine creates `SpliceEffect { source_card_id: ObjectId, rules_text: Vec<Effect> }` attached to the spell on the stack. Resolution iterates `[spell_effects] ++ [splice_effects]`. Splice card stays in hand. Effect stripped when spell leaves stack. Avoids true text-changing engine — just extends the resolution step list.

---

## 5. Classification Summary Table

| Rule Range | Keyword | Classification | Phase | Ticket | Notes |
|---|---|---|---|---|---|
| 702.1 | (General) | PURE-DEF | — | — | Defines keyword ability concept |
| 702.2 | Deathtouch | ALREADY-IMPL + DEFERRED(c,d,e) | P4 done / P5-P6 | T20b | Core done; LKI + any-zone deferred |
| 702.3 | Defender | ALREADY-IMPL | P4 done | — | Fully implemented |
| 702.4 | Double Strike | ALREADY-IMPL + DEFERRED(c,d) | P4 done / P5 | — | Core done; mid-combat grant/remove deferred |
| 702.5 | Enchant | TESTABLE | P5-Pre | T15b | Aura targeting restrictions |
| 702.6 | Equip | TESTABLE + DEFERRED(c,d,e) | P5-Pre / P8 | T15b | Core equip testable; quality/PW/multi-equip deferred. 702.6d reclassified PURE-DEF→TESTABLE. |
| 702.7 | First Strike | ALREADY-IMPL + DEFERRED(c) | P4 done / P5 | — | Core done; mid-combat grant/remove deferred |
| 702.8 | Flash | TESTABLE + DEFERRED(a-zone) | P5-Pre / P8 | T18 | Basic flash testable; any-zone deferred |
| 702.9 | Flying | ALREADY-IMPL | P4 done | — | Fully implemented |
| 702.10 | Haste | ALREADY-IMPL | P4 done | — | Fully implemented |
| 702.11 | Hexproof | TESTABLE + DEFERRED(c,d,e) | P5-Pre / P8 | T22 | Permanent hexproof testable; player/from-quality deferred |
| 702.12 | Indestructible | TESTABLE | P5-Pre | T09 | Both lethal-damage and destroy-effect tests |
| 702.13 | Intimidate | DEFERRED | P8 | — | Deprecated evasion keyword |
| 702.14 | Landwalk | DEFERRED | P8 | — | Niche evasion |
| 702.15 | Lifelink | ALREADY-IMPL + DEFERRED(c,d) | P4 done / P5-P6 | T20b | Core done; LKI + any-zone deferred |
| 702.16 | Protection | TESTABLE + DEFERRED(j,k,n,p) | P5-Pre / P8-P9 | T22 | Core DEBT tests; everything/player/self-exception deferred |
| 702.17 | Reach | ALREADY-IMPL | P4 done | — | Fully implemented |
| 702.18 | Shroud | TESTABLE + DEFERRED(player) | P5-Pre / P8 | T22 | Permanent shroud testable; player shroud deferred |
| 702.19 | Trample | ALREADY-IMPL + TESTABLE(d) + DEFERRED(c,e,f) | P4 done / P8 | — | Core done; all-blockers-removed regression NEW; PW variants deferred |
| 702.20 | Vigilance | ALREADY-IMPL | P4 done | — | Fully implemented |
| 702.21 | Ward | TESTABLE + DEFERRED(b) | P7 | NEW | Triggered ability; ward-X deferred |
| 702.22 | Banding | OUT-OF-SCOPE (all) | — | — | Disproportionate cost (~15 cards), invasive combat changes |
| 702.23 | Rampage | DEFERRED | P8 | — | Niche triggered ability |
| 702.24 | Cumulative Upkeep | DEFERRED | P8 | — | Niche triggered ability |
| 702.25 | Flanking | DEFERRED | P8 | — | Niche triggered ability |
| 702.26 | Phasing | DEFERRED | P8 | — | Complex status change mechanic |
| 702.27 | Buyback | DEFERRED | P8 | T17 | Additional cost + hand return |
| 702.28 | Shadow | DEFERRED | P8 | — | Niche bidirectional evasion |
| 702.29 | Cycling | DEFERRED | P8 | — | Hand-based activated ability |
| 702.30 | Echo | DEFERRED | P8 | — | Niche upkeep trigger |
| 702.31 | Horsemanship | DEFERRED | P8 | — | Niche evasion (flying clone) |
| 702.32 | Fading | DEFERRED | P8 | — | Niche counter mechanic |
| 702.33 | Kicker | DEFERRED | P8 | T17 | Additional cost + kicked status |
| 702.34 | Flashback | DEFERRED | P8 | T17 | Alternative cost from graveyard |
| 702.35 | Madness | DEFERRED | P8 | — | Replacement effect on discard |
| 702.36 | Fear | DEFERRED | P8 | — | Deprecated evasion |
| 702.37 | Morph | DEFERRED | P8 | — | Face-down infrastructure (rule 708) |
| 702.38 | Amplify | DEFERRED | P8 | — | Niche ETB counters |
| 702.39 | Provoke | DEFERRED | P8 | — | Niche forced-block |
| 702.40 | Storm | DEFERRED | P8 | — | Spell copying |
| 702.41 | Affinity | DEFERRED | P8 | T17 | Cost reduction |
| 702.42 | Entwine | DEFERRED | P8 | — | Modal + additional cost |
| 702.43 | Modular | DEFERRED | P8 | — | ETB counters + death trigger |
| 702.44 | Sunburst | DEFERRED | P8 | — | Mana-color tracking |
| 702.45 | Bushido | DEFERRED | P8 | — | Niche triggered ability |
| 702.46 | Soulshift | DEFERRED | P8 | — | Niche death trigger |
| 702.47 | Splice | DEFERRED | P8 | — | Text-changing during cast |
| 702.48 | Offering | DEFERRED | P8 | — | Sacrifice + cost reduction |
| 702.49 | Ninjutsu | DEFERRED | P8 | — | Hand-based swap mechanic |
| 702.50 | Epic | DEFERRED | P8 | — | Casting restriction + delayed trigger |
| 702.51 | Convoke | DEFERRED | P8 | T17 | Creature-tap-as-mana |
| 702.52 | Dredge | DEFERRED + TESTABLE(b) | P8 | — | Draw replacement. 702.52b reclassified PURE-DEF→TESTABLE. |
| 702.53 | Transmute | DEFERRED | P8 | — | Hand-based tutor |
| 702.54 | Bloodthirst | DEFERRED | P8 | — | Conditional ETB counters |
| 702.55 | Haunt | DEFERRED | P8 | — | Exile association mechanic |
| 702.56 | Replicate | DEFERRED | P8 | — | Additional cost + spell copying |
| 702.57 | Forecast | DEFERRED | P8 | — | Hand-based upkeep ability |
| 702.58 | Graft | DEFERRED | P8 | — | ETB counters + triggered counter sharing |
| 702.59 | Recover | DEFERRED | P8 | — | Graveyard trigger |
| 702.60 | Ripple | DEFERRED | P8 | — | Library reveal + free cast |
| 702.61 | Split Second | DEFERRED | P8 | — | Stack restriction |
| 702.62 | Suspend | DEFERRED | P8 | — | Exile + time counters + free cast |
| 702.63 | Vanishing | DEFERRED + TESTABLE(b) | P8 | — | Time counters + sacrifice. 702.63b expanded to 2 ATOMs. |
| 702.64 | Absorb | DEFERRED | P8 | — | Damage prevention per source |
| 702.65 | Aura Swap | DEFERRED | P8 | — | Exchange mechanic |
| 702.66 | Delve | DEFERRED | P8 | T17 | Graveyard exile as payment |
| 702.67 | Fortify | DEFERRED | P8 | — | Land attachment |
| 702.68 | Frenzy | DEFERRED | P8 | — | Niche triggered ability |
| 702.69 | Gravestorm | DEFERRED | P8 | — | Died-this-turn counting + copies |
| 702.70 | Poisonous | DEFERRED | P8 | — | Poison counters |
| 702.71 | Transfigure | DEFERRED | P8 | — | Sacrifice-tutor |
| 702.72 | Champion | DEFERRED | P8 | — | ETB exile + LTB return |
| 702.73 | Changeling | DEFERRED | P8 | — | CDA for creature types |
| 702.74 | Evoke | DEFERRED | P8 | T17 | Alternative cost + ETB sacrifice |
| 702.75 | Hideaway | DEFERRED | P8 | — | Face-down exile (hidden info only; NOT rule 708) |
| 702.76 | Prowl | DEFERRED | P8 | — | Conditional alternative cost |
| 702.77 | Reinforce | DEFERRED | P8 | — | Hand-based counter ability |
| 702.78 | Conspire | DEFERRED | P8 | — | Tap-as-cost + spell copying |
| 702.79 | Persist | DEFERRED | P8 | T20b | Death trigger + conditional return. LKI dependency. |
| 702.80 | Wither | DEFERRED | P8 | — | Damage → -1/-1 counters |

---

## 6. NEW Tickets List

| ID | Keyword | Description | Phase |
|---|---|---|---|
| NEW-1 | Ward | Ward triggered ability: on-target trigger + counter-unless-pay | Phase 7 |
| NEW-2 | Trample | Regression test: `assign_trample_damage` with empty blocker list (all blockers removed before damage) | Phase 4 (immediate) |
| NEW-3 | Evasion | Unified Evasion Framework: `EvasionRestriction` + `BlockerFilter` enum in `validate_blockers` | Phase 8 |
| NEW-4 | Trample | Unified Trample DP: `TrampleContext` struct for trample / trample-over-PW / trample-over-battle | Phase 8 |

---

## 7. Gap Report

### Infrastructure Dependencies (blocking multiple keywords)

1. **Cost Modification Framework (T17)** — Blocks: Kicker, Buyback, Flashback, Affinity, Convoke, Delve, Evoke, Offering, Prowl. The `601.2e` cost pipeline (base → increases → reductions → Trinisphere floor) is documented in `cast.rs` but not implemented.

2. **LKI System (T20b)** — Blocks: Deathtouch-LKI (702.2e), Lifelink-LKI (702.15c), Wither-LKI (702.80b). Required for any "zone change before pending damage" scenario.

3. **Continuous Effects & Layers (Phase 5)** — Blocks: Mid-combat keyword grant/removal tests (702.4c/d, 702.7c). Required for dynamic keyword changes.

4. **Triggered Abilities (Phase 7)** — Blocks: Ward (702.21), plus all Phase 8 triggered-ability keywords (Rampage, Flanking, Bushido, Storm, etc.).

5. **Spell Copying** — Blocks: Storm (702.40), Replicate (702.56), Conspire (702.78), Gravestorm (702.69).

6. **Face-Down Infrastructure (rule 708)** — Blocks: Morph (702.37). Hideaway (702.75) does NOT require 708.

7. **Aura/Equipment Attachment Model (T15b)** — Blocks: Enchant (702.5), Equip (702.6).

---

## 8. ALREADY-IMPLEMENTED List

702.2 (Deathtouch), 702.3 (Defender), 702.4 (Double Strike), 702.7 (First Strike), 702.9 (Flying), 702.10 (Haste), 702.15 (Lifelink), 702.17 (Reach), 702.19 (Trample), 702.20 (Vigilance)

*Note: 702.2, 702.4, 702.7, 702.15, 702.19 have deferred sub-rules (LKI, any-zone, mid-combat changes).*

---

## 9. OUT-OF-SCOPE List

| Rule | Reason |
|------|--------|
| 702.22 (Banding, all sub-rules 702.22b–k) | Disproportionate implementation cost (~15 cards), invasive combat subsystem changes, combinatorial interaction surface with trample/deathtouch/wither. Stretch goal only. |
| 702.33h (Sticker Kicker) | Un-set mechanics / stickers permanently out of scope. |

---

## 10. DEFERRED List

| Rule | Keyword | Phase | Reason |
|------|---------|-------|--------|
| 702.2d | Deathtouch | Phase 6 | Any-zone damage requires zone-activated abilities |
| 702.2e | Deathtouch | Phase 5 | LKI system (T20b) |
| 702.4c/d | Double Strike | Phase 5 | Mid-combat keyword changes require continuous effects |
| 702.5c | Enchant | Phase 8 | Multiple enchant instances (rare, needs ability-granting) |
| 702.6c/d/e | Equip | Phase 8 | Quality restrictions, multi-equip, PW equip |
| 702.7c | First Strike | Phase 5 | Mid-combat keyword changes require continuous effects |
| 702.8a (zone) | Flash | Phase 8 | Cast-from-graveyard permission infrastructure |
| 702.11c/d/e | Hexproof | Phase 8 | Player hexproof, hexproof-from, ability removal |
| 702.13 | Intimidate | Phase 8 | Deprecated evasion keyword |
| 702.14 | Landwalk | Phase 8 | Niche evasion, land-type checking in blocker validation |
| 702.15c | Lifelink | Phase 5 | LKI system (T20b) |
| 702.15d | Lifelink | Phase 6 | Any-zone damage |
| 702.16j/k/n/p | Protection | Phase 8–9 | Everything, player, self-exception variants |
| 702.18a (player) | Shroud | Phase 8 | Player-as-target infrastructure |
| 702.19c/e/f | Trample | Phase 8 | Planeswalker combat variants |
| 702.21b | Ward | Phase 8 | Variable-cost ward-X |
| 702.23 | Rampage | Phase 8 | Niche triggered ability |
| 702.24 | Cumulative Upkeep | Phase 8 | Niche triggered ability with counter management |
| 702.25 | Flanking | Phase 8 | Niche triggered ability |
| 702.26 | Phasing | Phase 8 | Complex status change mechanic |
| 702.27 | Buyback | Phase 8 | Additional cost + hand return (T17) |
| 702.28 | Shadow | Phase 8 | Niche bidirectional evasion |
| 702.29 | Cycling | Phase 8 | Hand-based activated ability |
| 702.30 | Echo | Phase 8 | Niche upkeep trigger |
| 702.31 | Horsemanship | Phase 8 | Niche evasion (flying clone) |
| 702.32 | Fading | Phase 8 | Niche counter mechanic |
| 702.33 | Kicker | Phase 8 | Additional cost + kicked status (T17) |
| 702.34 | Flashback | Phase 8 | Alternative cost from graveyard (T17) |
| 702.35 | Madness | Phase 8 | Replacement effect on discard + exile cast |
| 702.36 | Fear | Phase 8 | Deprecated evasion |
| 702.37 | Morph | Phase 8 | Face-down infrastructure (rule 708) |
| 702.38 | Amplify | Phase 8 | Niche ETB counters |
| 702.39 | Provoke | Phase 8 | Niche forced-block trigger |
| 702.40 | Storm | Phase 8 | Spell-cast counting + spell copying |
| 702.41 | Affinity | Phase 8 | Cost reduction (T17) |
| 702.42 | Entwine | Phase 8 | Modal + additional cost |
| 702.43 | Modular | Phase 8 | ETB counters + death trigger |
| 702.44 | Sunburst | Phase 8 | Mana-color-spent tracking |
| 702.45 | Bushido | Phase 8 | Niche triggered ability |
| 702.46 | Soulshift | Phase 8 | Niche death trigger |
| 702.47 | Splice | Phase 8 | Text-changing during cast (T17 + rule 612) |
| 702.48 | Offering | Phase 8 | Sacrifice + cost reduction |
| 702.49 | Ninjutsu | Phase 8 | Hand-based swap mechanic |
| 702.50 | Epic | Phase 8 | Casting restriction + delayed trigger |
| 702.51 | Convoke | Phase 8 | Creature-tap-as-mana (T17) |
| 702.52 | Dredge | Phase 8 | Draw replacement + self-mill |
| 702.53 | Transmute | Phase 8 | Hand-based tutor by MV |
| 702.54 | Bloodthirst | Phase 8 | Conditional ETB counters |
| 702.55 | Haunt | Phase 8 | Exile association mechanic |
| 702.56 | Replicate | Phase 8 | Additional cost + spell copying |
| 702.57 | Forecast | Phase 8 | Hand-based upkeep ability |
| 702.58 | Graft | Phase 8 | ETB counters + triggered counter sharing |
| 702.59 | Recover | Phase 8 | Graveyard trigger |
| 702.60 | Ripple | Phase 8 | Library reveal + free cast |
| 702.61 | Split Second | Phase 8 | Stack restriction on casting/activation |
| 702.62 | Suspend | Phase 8 | Exile + time counters + free cast |
| 702.63 | Vanishing | Phase 8 | Time counters + sacrifice |
| 702.64 | Absorb | Phase 8 | Damage prevention per source |
| 702.65 | Aura Swap | Phase 8 | Exchange mechanic |
| 702.66 | Delve | Phase 8 | Graveyard exile as payment (T17) |
| 702.67 | Fortify | Phase 8 | Land attachment |
| 702.68 | Frenzy | Phase 8 | Niche triggered ability |
| 702.69 | Gravestorm | Phase 8 | Died-this-turn counting + spell copying |
| 702.70 | Poisonous | Phase 8 | Poison counter tracking |
| 702.71 | Transfigure | Phase 8 | Sacrifice-tutor |
| 702.72 | Champion | Phase 8 | ETB exile + LTB return (linked abilities) |
| 702.73 | Changeling | Phase 8 | CDA for creature types |
| 702.74 | Evoke | Phase 8 | Alternative cost + ETB sacrifice (T17) |
| 702.75 | Hideaway | Phase 8 | Face-down exile (hidden info, NOT rule 708) |
| 702.76 | Prowl | Phase 8 | Conditional alternative cost |
| 702.77 | Reinforce | Phase 8 | Hand-based counter ability |
| 702.78 | Conspire | Phase 8 | Tap-as-cost + spell copying |
| 702.79 | Persist | Phase 8 | Death trigger + conditional return (LKI T20b) |
| 702.80 | Wither | Phase 8 | Damage → -1/-1 counters |

---

### Statistics

- **Total top-level rules audited:** 80 (702.1–702.80)
- **Total sub-rules classified:** ~260
- **ATOM tests:** 106
- **COMP tests:** 8
- **META notes:** 7
- **ALREADY-IMPLEMENTED:** 10 keywords
- **TESTABLE:** 7 keywords
- **DEFERRED:** 57 keywords
- **OUT-OF-SCOPE:** 702.22 (banding) + 702.33h (sticker kicker)
- **Reclassifications:** 5
- **NEW tickets:** 4

--- End of Session 7B Summary ---
