# Session 5 Summary: Rules 600–608 — Spells, Abilities, and Effects

> **Source:** `session-5.md` (audited via `session-5-audit-response.md`)
> **Scope:** Casting (601), Activation (602), Triggered abilities (603), Static abilities (604), Mana abilities (605), Loyalty abilities (606), Linked abilities (607), Resolution (608)

---

## 1. ATOM Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-601.1a-001 | 601.1a | "Play a card" routes to land play for lands | Phase 8 | NEW-1 | |
| ATOM-601.1a-002 | 601.1a | "Play a card" routes to cast for non-lands | Phase 8 | NEW-1 | |
| ATOM-601.2-001 | 601.2 | Rollback on failed casting step | Phase 5-Pre | T18 | |
| ATOM-601.2a-001 | 601.2a | Card moves from hand to stack; controller set | IMPL | N/A | |
| ATOM-601.2a-002 | 601.2a | Spell-modifying continuous effects apply on stack entry | Phase 5-Layers | L15, T18 | |
| ATOM-601.2a-003 | 601.2a | move_to_stack works from zones other than Hand | Phase 8 | T18 | |
| ATOM-601.2b-001 | 601.2b | Mode choice stored on StackEntry | Phase 5-Pre | T18 | |
| ATOM-601.2b-002 | 601.2b | Alt/add cost choice stored on StackEntry | Phase 5-Pre | T17, T18 | |
| ATOM-601.2b-003 | 601.2b | Only one alternative cost allowed per cast | Phase 5-Pre | T18 | |
| ATOM-601.2b-004 | 601.2b | X value announced and stored on StackEntry | Phase 5-Pre | T18 | |
| ATOM-601.2b-005 | 601.2b | Hybrid mana choice recorded | DEFERRED | Hybrid | |
| ATOM-601.2b-006 | 601.2b | Phyrexian mana choice recorded | DEFERRED | Phyrexian | |
| ATOM-601.2b-007 | 601.2b | X choice for additional-cost-only X | Phase 5-Pre | T18 | |
| ATOM-601.2c-001 | 601.2c | Target selection during casting | IMPL | N/A | |
| ATOM-601.2c-002 | 601.2c | Conditional targets based on kicker/mode | Phase 5-Pre | T18 | |
| ATOM-601.2c-003 | 601.2c | Per-instance target uniqueness | Phase 5-Pre | T18 | |
| ATOM-601.2c-004 | 601.2c | Different target instances can share a target | Phase 5-Pre | T18 | |
| ATOM-601.2c-005 | 601.2c | Target-forcing effects maximized | Phase 8 | NEW-3 | |
| ATOM-601.2c-006 | 601.2c | Conditional targets absent when add cost not paid | Phase 5-Pre | T18 | |
| ATOM-601.2c-007 | 601.2c | Kicker changes target legality criteria | Phase 5-Pre | T18 | |
| ATOM-601.2d-001 | 601.2d | Damage/counter distribution at cast time | Phase 5-Pre | T18 | |
| ATOM-601.2d-002 | 601.2d | Zero-allocation rejected | Phase 5-Pre | T18 | |
| ATOM-601.2e-001 | 601.2e | Post-proposal legality check with rollback | Phase 5-Pre | T18 | |
| ATOM-601.2f-001 | 601.2f | Cost assembly pipeline | Phase 5-Pre/Layers | T18, L15 | |
| ATOM-601.2f-002 | 601.2f | Cost floor at {0} | Phase 5-Layers | L15 | |
| ATOM-601.2f-003 | 601.2f | Cost lock-in prevents later modifications | Phase 5-Pre | T18 | |
| ATOM-601.2f-004 | 601.2f | Cost reduction ordering choice via DP | Phase 5-Layers | L15 | |
| ATOM-601.2g-001 | 601.2g | Mana ability activation window during casting | Phase 5-Pre | T18 | |
| ATOM-601.2h-001 | 601.2h | Cost payment ordering | Phase 5-Pre | T18 | |
| ATOM-601.2h-002 | 601.2h | Atomic cost payment — no partial | IMPL | N/A | |
| ATOM-601.2h-003 | 601.2h | Player chooses order of cost components via DP | Phase 5-Pre | T18 | |
| ATOM-601.2i-001 | 601.2i | SpellCast event emitted after all steps complete | Phase 7 | Phase 7 | |
| ATOM-601.2i-002 | 601.2i | Priority returns to caster after casting | IMPL | N/A | |
| ATOM-601.2i-003 | 601.2i | Spell characteristics modified by continuous effects on completion | Phase 5-Layers | L10 | |
| ATOM-601.3-001 | 601.3 | Cast permission check | Phase 5-Layers | L15 | |
| ATOM-601.3a-001 | 601.3a | Cast-time look-ahead past prohibition | Phase 8 | D17 | |
| ATOM-601.3b-001 | 601.3b | Flash grant look-ahead | Phase 8 | D17 | |
| ATOM-601.3c-001 | 601.3c | Conditional flash grant from alt/add cost | Phase 8 | D17 | |
| ATOM-601.3c-002 | 601.3c | Alternative cost flash grant | Phase 8 | D17 | |
| ATOM-601.3d-001 | 601.3d | Conditional flash | Phase 8 | D17 | |
| ATOM-601.3e-001 | 601.3e | Alternative characteristic check for cast permissions | Phase 9 | Phase 9 | morph |
| ATOM-601.3f-001 | 601.3f | Face-down exile cast permission requires visibility | Phase 8 | D21 | |
| ATOM-601.3f-002 | 601.3f | Face-down exile cast denied + info leak prevention | Phase 8 | D21 | |
| ATOM-601.4-001 | 601.4 | Intra-step look-ahead for mode/cost choices | Phase 5-Pre | T18 | |
| ATOM-601.5-001 | 601.5 | Post-proposal re-check | Phase 5-Pre | T18 | |
| ATOM-601.5-002 | 601.5 | Cost-phase illegality does NOT cause rewind | Phase 5-Pre | T18 | |
| ATOM-601.5a-001 | 601.5a | Flash condition persistence after casting begins | Phase 8 | D17 | |
| ATOM-601.6-001 | 601.6 | Opponent-directed choice during casting | Phase 8 | NEW-4 | |
| ATOM-601.6a-001 | 601.6a | Controller selects which opponent decides | Phase 9 | Phase 9 | multiplayer |
| ATOM-601.6b-001 | 601.6b | Ordering exception for simultaneous cast-time actions | Phase 8 | NEW-5 | |
| ATOM-601.7-001 | 601.7 | Cost alteration doesn't retroactively modify stack | Phase 5-Layers | T18 | |
| ATOM-602.1a-001 | 602.1a | Activation cost charged to activating player | IMPL | N/A | |
| ATOM-602.1e-001 | 602.1e | Activation cost modifications apply to total | Phase 5-Layers | L15 | |
| ATOM-602.1e-002 | 602.1e | Single cost increase on activation cost | Phase 5-Layers | L15 | |
| ATOM-602.2-001 | 602.2 | Activation restricted to controller | IMPL | N/A | |
| ATOM-602.2-002 | 602.2 | Activation rollback on failure | Phase 5-Pre | T18 | |
| ATOM-602.2a-001 | 602.2a | Ability object created on stack | IMPL | N/A | |
| ATOM-602.2a-002 | 602.2a | Reveal on activation from hidden zone | Phase 8 | T19 | |
| ATOM-602.2a-003 | 602.2a | Ability object characteristic profile distinct from spell | Phase 5-Pre | T19 | |
| ATOM-602.2b-001 | 602.2b | Activation follows casting pipeline steps | Phase 5-Pre | T18, T19 | |
| ATOM-602.4-001 | 602.4 | Cost alteration from abilities doesn't retroactively affect stack | Phase 5-Layers | L15 | |
| ATOM-602.5-001 | 602.5 | Activation prohibition check | Phase 5-Layers | L15, T19 | |
| ATOM-602.5a-001 | 602.5a | Summoning sickness blocks {T}/{Q} activation | IMPL | N/A | |
| ATOM-602.5a-002 | 602.5a | Haste bypasses summoning sickness for tap/untap costs | IMPL | N/A | |
| ATOM-602.5b-001 | 602.5b | Once-per-turn restriction persists across controller change | Phase 5-Pre | T19 | |
| ATOM-602.5c-001 | 602.5c | Per-acquisition restriction scoping | Phase 8 | Phase 8 | |
| ATOM-602.5d-001 | 602.5d | Sorcery-speed activation restriction | Phase 5-Pre | T19 | |
| ATOM-602.5d-002 | 602.5d | Sorcery-speed requires empty stack | Phase 5-Pre | T19 | |
| ATOM-602.5e-001 | 602.5e | Instant-speed activation restriction | Phase 5-Pre | T19 | |
| ATOM-603.1b-001 | 603.1b | Multi-condition trigger tracking | Phase 7 | Phase 7 | |
| ATOM-603.2-001 | 603.2 | Trigger detection (no immediate effect) | Phase 7 | Phase 7 | |
| ATOM-603.2a-001 | 603.2a | Triggers bypass activation prohibitions | Phase 7 | Phase 7 | |
| ATOM-603.2b-001 | 603.2b | Phase/step-begin trigger | Phase 7 | Phase 7 | |
| ATOM-603.2c-001 | 603.2c | Per-occurrence triggering | Phase 7 | Phase 7 | |
| ATOM-603.2d-001 | 603.2d | Trigger multiplier | Phase 7 | Phase 7 | |
| ATOM-603.2e-001 | 603.2e | "Becomes tapped" doesn't trigger on ETB tapped | Phase 7 | Phase 7 | |
| ATOM-603.2e-002 | 603.2e | "Becomes" requires state change — no re-trigger on same state | Phase 7 | Phase 7 | |
| ATOM-603.2f-001 | 603.2f | Hidden-zone trigger suppression | Phase 7 | Phase 7 | |
| ATOM-603.2g-001 | 603.2g | Prevented events don't cause triggers | Phase 7 | Phase 7 | |
| ATOM-603.2h-001 | 603.2h | Once-per-turn trigger restriction | Phase 7 | Phase 7 | |
| ATOM-603.2h-002 | 603.2h | Once-per-turn checked at resolution | Phase 7 | Phase 7 | |
| ATOM-603.3-001 | 603.3 | Trigger → stack placement timing | Phase 7 | Phase 7 | |
| ATOM-603.3a-001 | 603.3a | Trigger controller = source controller at trigger time | Phase 7 | Phase 7 | |
| ATOM-603.3b-001 | 603.3b | APNAP trigger stacking order | Phase 7 | Phase 7 | |
| ATOM-603.3b-002 | 603.3b | Two-tier trigger stacking order | Phase 7 | Phase 7 | |
| ATOM-603.3c-001 | 603.3c | Modal trigger mode selection | Phase 7 | Phase 7 | |
| ATOM-603.3c-002 | 603.3c | Modal trigger fizzle when no mode is legal | Phase 7 | Phase 7 | |
| ATOM-603.3d-001 | 603.3d | Trigger target selection follows casting rules | Phase 7 | Phase 7 | |
| ATOM-603.4-001 | 603.4 | Intervening-if double check (true at both times) | Phase 7 | Phase 7 | |
| ATOM-603.4-002 | 603.4 | Intervening-if blocks trigger | Phase 7 | Phase 7 | |
| ATOM-603.4-003 | 603.4 | Intervening-if fails at resolution | Phase 7 | Phase 7 | |
| ATOM-603.5-001 | 603.5 | Optional trigger still stacks | Phase 7 | Phase 7 | |
| ATOM-603.6-001 | 603.6 | Zone-change trigger resolution finds object in new zone | Phase 7 | Phase 7 | |
| ATOM-603.6a-001 | 603.6a | ETB trigger detection including newcomer self-trigger | Phase 7 | Phase 7 | |
| ATOM-603.6b-001 | 603.6b | Continuous effects apply at moment of ETB | Phase 5-Layers/7 | Phase 7, L13 | |
| ATOM-603.6b-002 | 603.6b | Ability removal prevents ETB trigger | Phase 5-Layers/7 | L09, Phase 7 | |
| ATOM-603.6c-001 | 603.6c | LTB trigger zone tracking | Phase 7 | Phase 7 | |
| ATOM-603.6c-002 | 603.6c | First-zone-only tracking on LTB trigger | Phase 7 | Phase 7 | |
| ATOM-603.6d-001 | 603.6d | "Enters tapped" is NOT a triggered ability | Phase 6 | Phase 6 | |
| ATOM-603.6e-001 | 603.6e | Aura LTB trigger cross-zone resolution | Phase 7 | Phase 7, T15 | |
| ATOM-603.6e-002 | 603.6e | Aura trigger tracks enchanted creature across zones | Phase 7 | Phase 7, T15 | |
| ATOM-603.7-001 | 603.7 | Delayed trigger creation and storage | Phase 7 | Phase 7 | |
| ATOM-603.7a-001 | 603.7a | No retroactive delayed triggers | Phase 7 | Phase 7 | |
| ATOM-603.7b-001 | 603.7b | One-shot delayed trigger | Phase 7 | Phase 7 | |
| ATOM-603.7b-002 | 603.7b | Simultaneous-event delayed trigger choice | Phase 7 | Phase 7 | |
| ATOM-603.7c-001 | 603.7c | Delayed trigger tracks object identity not characteristics | Phase 7 | Phase 7 | |
| ATOM-603.7d-001 | 603.7d | Delayed trigger source/controller from spell | Phase 7 | Phase 7 | |
| ATOM-603.7e-001 | 603.7e | Delayed trigger source inheritance from ability | Phase 7 | Phase 7 | |
| ATOM-603.7f-001 | 603.7f | Delayed trigger from replacement inherits static source | Phase 7/6 | Phase 7 | |
| ATOM-603.7g-001 | 603.7g | Static-action delayed trigger source/controller | Phase 7 | Phase 7 | |
| ATOM-603.7h-001 | 603.7h | Resolution-count delayed trigger | Phase 7 | Phase 7 | |
| ATOM-603.8-001 | 603.8 | State-trigger one-shot until resolution | Phase 7 | Phase 7 | |
| ATOM-603.8-002 | 603.8 | State trigger fires during spell resolution | Phase 7 | Phase 7 | |
| ATOM-603.9-001 | 603.9 | Player-loss trigger | Phase 7 | Phase 7 | |
| ATOM-603.10a-001 | 603.10a | LTB trigger look-back | Phase 7 | Phase 7 | |
| ATOM-603.10a-002 | 603.10a | Death trigger on dying creature | Phase 7 | Phase 7 | |
| ATOM-603.10c-001 | 603.10c | Unattach trigger look-back | Phase 7 | Phase 7, T04 | |
| ATOM-603.10c-002 | 603.10c | Re-attach triggers unattach from original | Phase 7 | Phase 7, T04 | |
| ATOM-603.10c-003 | 603.10c | Re-attach to different creature triggers "becomes attached" | Phase 7 | Phase 7, T04 | |
| ATOM-603.10d-001 | 603.10d | Control-change trigger look-back | Phase 7 | Phase 7, L11 | |
| ATOM-603.10e-001 | 603.10e | Counter-trigger look-back | Phase 7 | Phase 7 | |
| ATOM-603.12-001 | 603.12 | Reflexive trigger created during resolution | Phase 7 | Phase 7 | |
| ATOM-603.12a-001 | 603.12a | Reflexive trigger coalesces multiple payments | Phase 7 | Phase 7 | |
| ATOM-604.2-001 | 604.2 | Static ability lifetime = source lifetime on BF | Phase 5-Layers | L08 | |
| ATOM-604.3-001 | 604.3 | CDA applies in all zones | Phase 5-Layers | L04 | |
| ATOM-604.3a-001 | 604.3a | CDA classification validation | Phase 5-Layers | L01 | |
| ATOM-604.4-001 | 604.4 | Attachment-based static effect re-targeting | Phase 5-Pre/Layers | T04, L08 | |
| ATOM-604.5-001 | 604.5 | Stack-zone static abilities | Phase 5-Pre | T17, T18 | |
| ATOM-604.5-002 | 604.5 | Stack-zone static for alternative cost | Phase 5-Pre | T17, T18 | |
| ATOM-604.6-001 | 604.6 | Hand-zone static abilities for cast permissions | Phase 5-Pre | T18 | |
| ATOM-604.6-002 | 604.6 | Hand-zone static restricts casting timing | Phase 5-Pre | T18 | |
| ATOM-604.7-001 | 604.7 | Static ability CDA fails when source gone (no LKI) | Phase 5-Layers | L18 | |
| ATOM-605.1a-001 | 605.1a | Mana ability classification for activated abilities | IMPL | N/A | |
| ATOM-605.1a-002 | 605.1a | Target disqualifies mana ability classification | Phase 5-Pre | NEW-2 | |
| ATOM-605.1a-003 | 605.1a | Loyalty ability disqualifies mana ability | Phase 8 | Phase 8 | |
| ATOM-605.1a-004 | 605.1a | Target disqualifies even with mana production | Phase 5-Pre | NEW-2 | |
| ATOM-605.1b-001 | 605.1b | Triggered mana ability classification | Phase 7 | Phase 7 | |
| ATOM-605.2-001 | 605.2 | Mana ability classification is static not state-dependent | IMPL | N/A | |
| ATOM-605.3a-001 | 605.3a | Mana ability activation window during casting | IMPL | N/A | |
| ATOM-605.3a-002 | 605.3a | Mana ability window during rule-required payment | Phase 7 | Phase 7 | |
| ATOM-605.3a-003 | 605.3a | Mana ability window during ability activation cost | IMPL | N/A | |
| ATOM-605.3b-001 | 605.3b | Mana ability immediate resolution | IMPL | N/A | |
| ATOM-605.3c-001 | 605.3c | Mana ability re-activation lock (non-tap cost) | IMPL | N/A | structural-guard |
| ATOM-605.4a-001 | 605.4a | Triggered mana ability immediate resolution | Phase 7 | Phase 7 | |
| ATOM-605.5a-001 | 605.5a | Target/wrong-trigger-source disqualifies mana ability | Phase 7 | Phase 7 | |
| ATOM-605.5b-001 | 605.5b | Spells always use the stack regardless of mana output | IMPL | N/A | |
| ATOM-606.2-001 | 606.2 | Loyalty ability classification | DEFERRED | Phase 8 | planeswalker |
| ATOM-606.3-001 | 606.3 | Loyalty ability timing + once-per-turn | DEFERRED | Phase 8 | planeswalker |
| ATOM-606.3-002 | 606.3 | Loyalty ability sorcery-speed enforcement | DEFERRED | Phase 8 | planeswalker |
| ATOM-606.4-001 | 606.4 | Loyalty counter cost payment | DEFERRED | Phase 8 | planeswalker |
| ATOM-606.5-001 | 606.5 | Loyalty cost combination | DEFERRED | Phase 8 | planeswalker |
| ATOM-606.6-001 | 606.6 | Loyalty counter floor check | DEFERRED | Phase 8 | planeswalker |
| ATOM-607.1-001 | 607.1 | Linked ability scoping — reads only first ability's data | Phase 5-Pre | T20 | |
| ATOM-607.1c-001 | 607.1c | Self-linked ability (Tyrant's Choice) | Phase 5-Pre | T20 | |
| ATOM-607.1d-001 | 607.1d | Cross-object linked abilities | DEFERRED | Phase 8 | |
| ATOM-607.2a-001 | 607.2a | Exile-reference linking (O-Ring pattern) | Phase 5-Pre | T20 | |
| ATOM-607.2a-002 | 607.2a | Per-ability exile tracking (two independent linked pairs) | Phase 5-Pre | T20 | |
| ATOM-607.2b-001 | 607.2b | Replacement-exile linking | Phase 6 | T20, Phase 6 | |
| ATOM-607.2c-001 | 607.2c | ETB-creation linking | Phase 5-Pre/7 | T20 | |
| ATOM-607.2d-001 | 607.2d | Choice-value linking | Phase 5-Pre | T20 | |
| ATOM-607.2d-002 | 607.2d | Choice-value persistence through zone change (Cavern) | Phase 5-Pre | T20 | |
| ATOM-607.2e-001 | 607.2e | Noted-information linking | Phase 8 | Phase 8 | |
| ATOM-607.2f-001 | 607.2f | Word-choice linking | Phase 8 | Phase 8 | |
| ATOM-607.2g-001 | 607.2g | ETB-cost linking | Phase 6 | Phase 6 | |
| ATOM-607.2h-001 | 607.2h | Same-paragraph static+triggered linking | Phase 5-Pre/7 | T20 | |
| ATOM-607.2i-001 | 607.2i | Kicker-style additional cost linking | Phase 5-Pre | T17, T20 | |
| ATOM-607.2i-002 | 607.2i | Per-kicker-cost linking (Stormscape Battlemage) | Phase 5-Pre | T17, T20 | |
| ATOM-607.2j-001 | 607.2j | Variable cost value linking | Phase 5-Pre | T17, T18, T20 | |
| ATOM-607.2q-001 | 607.2q | Cast-cost-exile linking | Phase 5-Pre | T17, T20 | |
| ATOM-607.3-001 | 607.3 | Multi-exile linked ability resolution | Phase 8 | Phase 8 | |
| ATOM-607.5-001 | 607.5 | Acquired-pair isolation | Phase 8 | Phase 8 | |
| ATOM-607.5a-001 | 607.5a | Undefined choice from broken link | Phase 8 | Phase 8 | |
| ATOM-608.1-001 | 608.1 | Stack resolution trigger on all-pass | IMPL | N/A | |
| ATOM-608.2-001 | 608.2 | Full resolution procedure order verification | Phase 7/5-Pre | T18 | |
| ATOM-608.2a-001 | 608.2a | Intervening-if resolution check | Phase 7 | Phase 7 | |
| ATOM-608.2b-001 | 608.2b | All-targets-illegal fizzle | IMPL | N/A | |
| ATOM-608.2b-002 | 608.2b | Partial-target resolution (Plague Spores) | Phase 5-Pre | T18 | |
| ATOM-608.2b-003 | 608.2b | Zone-change makes target illegal | IMPL | N/A | |
| ATOM-608.2b-004 | 608.2b | LKI for source during target recheck | Phase 5-Layers | L18 | |
| ATOM-608.2b-005 | 608.2b | Partial-target resolution (Jagged Lightning) | Phase 5-Pre | T18 | |
| ATOM-608.2c-001 | 608.2c | Sequential instruction execution | IMPL | N/A | |
| ATOM-608.2d-001 | 608.2d | Resolution-time choice announcement | Phase 7 | Phase 7, T18 | |
| ATOM-608.2d-002 | 608.2d | Resolution-time untargeted distribution | Phase 5-Pre | T18 | |
| ATOM-608.2d-003 | 608.2d | Flexible distribution with minimum-per-object | Phase 5-Pre | T18 | |
| ATOM-608.2e-001 | 608.2e | APNAP-ordered resolution steps | Phase 9 | T18, Phase 9 | multiplayer |
| ATOM-608.2f-001 | 608.2f | Simultaneous action processing | Phase 9 | Phase 9 | multiplayer |
| ATOM-608.2f-002 | 608.2f | APNAP-ordered simultaneous sacrifice | Phase 9 | T18, Phase 9 | multiplayer |
| ATOM-608.2g-001 | 608.2g | Cast-during-resolution (cascade-like) | Phase 8 | Phase 8 | |
| ATOM-608.2h-001 | 608.2h | Single-determination + LKI fallback | Phase 5-Layers/7 | L18 | |
| ATOM-608.2h-002 | 608.2h | LKI for departed source | Phase 5-Layers | L18 | |
| ATOM-608.2i-001 | 608.2i | Historical look-back exception to 608.2h | Phase 5-Pre | T18 | |
| ATOM-608.2j-001 | 608.2j | Strict characteristic matching | Phase 5-Layers/7 | L10 | |
| ATOM-608.2k-001 | 608.2k | Untargeted-reference persistence | Phase 7 | Phase 7 | |
| ATOM-608.2m-001 | 608.2m | Resolution continuation after stack departure | Phase 8 | Phase 8 | low-priority |
| ATOM-608.2n-001 | 608.2n | Post-resolution zone transition (spell → GY) | IMPL | N/A | |
| ATOM-608.2n-002 | 608.2n | Ability removal from stack post-resolution | IMPL | N/A | |
| ATOM-608.2p-001 | 608.2p | Resolution-count tracking (Ashling Flame Dancer) | Phase 7 | Phase 7 | |
| ATOM-608.3a-001 | 608.3a | Untargeted permanent spell → ETB | IMPL | N/A | |
| ATOM-608.3b-001 | 608.3b | Targeted permanent fizzle or bestow fallback | Phase 5-Pre | T15b | |
| ATOM-608.3b-002 | 608.3b | Bestow fallback | Phase 9 | Phase 9 | |
| ATOM-608.3c-001 | 608.3c | Aura ETB attachment | Phase 5-Pre | T15b | |
| ATOM-608.3e-001 | 608.3e | ETB prohibition fallback | Phase 6 | Phase 6 | |
| ATOM-608.3f-001 | 608.3f | Spell-copy → token (not "created") | Phase 8 | Phase 8 | |
| ATOM-608.3g-001 | 608.3g | Stack static → delayed trigger on ETB (Dash/Blitz) | Phase 8 | Phase 8 | |

---

## 2. BOUNDARY-DEF Index

| ID | Rule | Summary | Phase | Ticket |
|----|------|---------|-------|--------|
| ATOM-601.1a-001/002 | 601.1a | "Play a card" routes to land play or cast | Phase 8 | NEW-1 |
| ATOM-607.1-001 | 607.1 | Linked ability scoping definition | Phase 5-Pre | T20 |
| ATOM-604.7-001 | 604.7 | Static abilities can't use LKI | Phase 5-Layers | L18 |
| 601.2 header | 601.2 | Casting is sequential 601.2a–i with rollback | Phase 5-Pre | T18 |
| 608.2 header | 608.2 | Resolution follows 608.2a–b then 608.2c–m then 608.2n/p | Phase 7/5-Pre | T18 |
| 608.3 header | 608.3 | Permanent spell resolution follows 608.3a–e | Phase 5-Pre | — |

---

## 3. COMP Index

| ID | Rules | Summary | Composes | Phase |
|----|-------|---------|----------|-------|
| COMP-601+608-001 | 601.2a–i, 608.2c, 608.2n | Full cast pipeline → resolution → GY | All 601.2 ATOMs + 608.2c/n | IMPL |
| COMP-601+608-002 | 601.2c, 608.2b | Fizzle on target removal | 601.2c-001, 608.2b-001 | IMPL |
| COMP-601+602+605-001 | 605.3a, 601.2g, 605.3b | Mana ability during casting mana window | 605.3a-001, 601.2g-001, 605.3b-001 | IMPL |
| COMP-602+605-001 | 602.5a, 605.1a | Summoning sick: tap blocked, sacrifice allowed | 602.5a-001, 605.1a-001 | IMPL |
| COMP-603+608-001 | 603.2, 603.3, 603.6a, 608.2c | ETB trigger → stack → resolution | 603.2-001, 603.3-001, 603.6a-001, 608.2c-001 | Phase 7 |
| COMP-601+604+605-001 | 604.2, 601.2f, 605.3a | Static cost reduction + mana + cast | 604.2-001, 601.2f-001, 605.3a-001 | Phase 5-Layers |
| COMP-607+608-001 | 607.1, 607.2a, 608.3a, 603.6c | O-Ring exile + return linked pattern | 607.1-001, 607.2a-001, 603.6c-001 | Phase 5-Pre/7 |

---

## 4. META Entries

> **META-GAMESTATE-SNAPSHOT (deferred — no architectural risk):** Casting rollback requires GameState snapshot before 601.2a. On failure at any step, restore snapshot. Overlap with loop detection (rule 731) — both need state snapshot/comparison infrastructure. Potential implementation: clone mutable portions of GameState before 601.2a; restore on failure. Simple and correct. **Deferred:** No architectural decisions needed now. The `GameState` struct is already a complete, concise representation of game state, which is a sufficient starting point for both rollback and future loop detection. Implementation can happen when casting pipeline (T18) is built. See also `state-tracking-architecture.md`.

> **META-CAST-PERMISSION-LAYERS:** 601.3 is a meta-rule. Multiple rule subsystems feed into cast permission: timing (505.6a/b), prohibitions (L15 CantCastSpells), flash grants (601.3b–d), zone permissions (601.3f). Implementation should have a single `can_begin_casting()` function that queries all subsystems.

> **META-MULTI-CONDITION-TRIGGERS:** Multi-condition triggers ("whenever you cast a creature AND an artifact in the same turn") require per-turn event tracking. Architecture: `TurnEventLog` on GameState tracking event categories per player per turn. Trigger matcher checks log for all conditions being met. This is an architectural decision needed before Phase 7 implementation.

> **META-HIDDEN-ZONE-TRIGGER-COMPLEXITY:** Reference `plans/atomic-tests/603-2f-complexity.md` for the Library of Leng + Guerrilla Tactics + Future Sight scenario. Architectural takeaway: trigger checking must happen AFTER all replacement effects resolve and zone changes finalize, using the final public/hidden zone status of each object.

> **META-TWO-TIER-TRIGGER-STACKING:** Engine needs a way to classify triggers as "triggers-on-trigger" vs "triggers-on-event." During `stack_pending_triggers()`, process in two passes.

> **META-LINKED-ABILITY-STORAGE:** Engine needs per-permanent storage for linked ability data: `linked_data: HashMap<AbilityId, LinkedAbilityData>` on BattlefieldEntity or GameObject. LinkedAbilityData stores exiled ObjectIds, chosen values, noted information, paid costs, etc. Each ability that writes data tags it with its AbilityId; the reading ability looks up only its linked AbilityId's data.

---

## 5. Classification Summary Table

### PURE-DEF Rules (no test needed)

| Rule | Description |
|------|-------------|
| 600.1 | Chapter header |
| 601.1 | Errata: "playing" → "casting" |
| 602.1 | Activated abilities format definition |
| 602.1b | Activation instructions after effect |
| 602.1c | Only activated abilities can be "activated" |
| 602.1d | Errata: "playing" → "activating" |
| 602.3 | Opponent choice (parallel to 601.6) |
| 602.3a | Controller picks opponent (parallel to 601.6a) |
| 602.3b | Simultaneous actions (parallel to 601.6b) |
| 603.1 | Triggered ability format ("When/Whenever/At") |
| 603.1a | Post-effect instructions |
| 603.10 | "Look back in time" trigger concept definition |
| 603.11 | Static+triggered link (reference to 607) |
| 604.1 | Static abilities "simply true" |
| 605.1 | Mana ability criteria header |
| 605.3 | Activated mana ability header |
| 605.4 | Triggered mana ability header |
| 606.1 | Loyalty abilities naming |
| 607.1a | Granted ability is "printed on" |
| 607.1b | DFC abilities are "printed on" |
| 607.2 | Kinds of linked abilities header |
| 607.4 | Ability in multiple linked pairs |

### ALREADY-IMPLEMENTED Rules

601.2a (partial), 601.2h (partial), 601.2i (partial), 602.1a, 602.2 (partial), 602.2a (partial), 602.5a, 605.1a (partial), 605.2, 605.3a, 605.3b, 605.3c, 605.5b, 608.1, 608.2b (partial), 608.2c, 608.2n, 608.3a

---

## 6. NEW Tickets

| Ticket | Description | Rules |
|--------|-------------|-------|
| NEW-1 | "Play a card" permission routing (land play vs. cast) | 601.1a |
| NEW-2 | Mana ability classification: target check | 605.1a |
| NEW-3 | Target-forcing effect maximization during 601.2c | 601.2c |
| NEW-4 | Opponent-directed choices during casting (601.6) | 601.6 |
| NEW-5 | Controller-first simultaneous actions during casting (601.6b) | 601.6b |

---

## 7. Gap Report

### Missing Engine Capabilities

1. **Casting pipeline completeness (T18):**
   - Mode choice storage on StackEntry (601.2b)
   - Additional/alternative cost declaration (601.2b, T17)
   - X value storage on StackEntry (601.2b, T06)
   - Conditional targets based on kicker/mode (601.2c)
   - Per-instance target uniqueness enforcement (601.2c)
   - Distribution storage on StackEntry (601.2d)
   - Post-proposal legality recheck with rollback (601.2e)
   - Full cost assembly pipeline: base + increases − reductions + lock-in (601.2f, L15)
   - Explicit mana ability window (601.2g — currently implicit)
   - Partial-target resolution (608.2b — currently all-or-nothing)

2. **Activation restrictions (T19):**
   - Once-per-turn tracking per ability (602.5b)
   - Sorcery-speed activation restriction (602.5d)
   - `CantActivateAbilities` enforcement (602.5, L15)
   - Reveal-on-activation from hidden zone (602.2a)

3. **Linked abilities (T20):**
   - Per-ability exile tracking ("exiled with [this]")
   - Choice storage on permanents (607.2d)
   - Kicker-paid flags on StackEntry/permanent (607.2i)
   - Self-linked ability detection (607.1c)

4. **Triggered ability infrastructure (Phase 7):**
   - pending_triggers queue (architectural decision documented)
   - APNAP ordering for trigger stacking (603.3b)
   - Intervening-if double check (603.4)
   - State triggers (603.8)
   - Delayed triggered abilities (603.7)
   - Reflexive triggered abilities (603.12)
   - Look-back-in-time for LTB triggers (603.10a)
   - "Becomes" event filtering (603.2e)
   - Trigger doubling (603.2d)
   - Once-per-turn trigger restriction (603.2h)

5. **Multi-condition trigger tracking (603.1b):**
   - Structured multi-condition trigger representation
   - Condition evaluation pipeline
   - META-MULTI-CONDITION-TRIGGERS architecture note documents the approach

6. **Two-tier trigger stacking (603.3b):**
   - APNAP ordering for trigger placement on stack
   - Within-player ordering via DecisionProvider choice
   - Two-tier system: inter-player (APNAP, mandatory) then intra-player (choice)

7. **GameState snapshot infrastructure (601.2e):**
   - Full GameState snapshot before casting proposal begins
   - Rollback capability if proposal becomes illegal at 601.2e recheck
   - See state-tracking-architecture.md for approach comparison

8. **LKI system (L18):**
   - LKISnapshot wrapping EffectiveCharacteristics
   - Snapshot on zone change
   - Source LKI for target recheck (608.2b)
   - Current-info vs. LKI determination (608.2h)

9. **Cost modification (L15):**
   - CostModification enum (IncreaseCost, ReduceCost, SetMinimumCost)
   - PlayerActionRestriction for CantCastSpells, CantActivateAbilities
   - Post-layer pass computation

---

## 8. ALREADY-IMPLEMENTED List

601.2a, 601.2c, 601.2h, 601.2i, 602.1a, 602.2, 602.2a, 602.5a, 605.1a, 605.2, 605.3a, 605.3b, 605.3c, 605.5b, 608.1, 608.2b, 608.2c, 608.2n, 608.3a

---

## 9. OUT-OF-SCOPE List

| Rule | Reason |
|------|--------|
| 603.10g | Planechase excluded |
| 607.2n | Conspiracy cards only |
| 607.2p | Conspiracy cards only |

---

## 10. DEFERRED List

| Rule | Target Phase | Reason |
|------|-------------|--------|
| 601.2b (hybrid) | Future | Hybrid mana payment |
| 601.2b (Phyrexian) | Future | Phyrexian mana payment |
| 601.3a | Phase 8 (D17) | Cast legality look-ahead |
| 601.3b | Phase 8 (D17) | Flash look-ahead for bestow/morph |
| 601.3c | Phase 8 (D17) | Conditional flash from alt/add cost |
| 601.3d | Phase 8 (D17) | Conditional flash |
| 601.3e | Phase 9 | Alternative characteristics for cast legality (morph) |
| 601.3f | Phase 8 (D21) | Face-down exile cast permission |
| 601.5a | Phase 8 (D17) | Flash condition persistence |
| 601.6a | Phase 9 | Controller picks opponent (multiplayer) |
| 602.5c | Phase 8 | Per-acquisition restriction scoping (ability copying) |
| 603.10b | Phase 9 | Phase-out triggers look back (Phasing) |
| 606.2–606.6 | Phase 8 | All loyalty ability rules (Planeswalkers) |
| 607.1d | Phase 8 | Cross-object linked abilities |
| 607.2e | Phase 8 | Noted-information linking |
| 607.2f | Phase 8 | Word-choice linking |
| 607.2g | Phase 6 | ETB-cost linking |
| 607.2k | Phase 9 | Champion linked abilities |
| 607.2m | Phase 9 | Anchor word linked abilities |
| 607.3 | Phase 8 | Multi-exile from copied ability |
| 607.5 | Phase 8 | Acquired linked ability isolation |
| 607.5a | Phase 8 | Undefined choice from broken link |
| 608.2g | Phase 8 | Cast-during-resolution |
| 608.2m | Phase 8 | Resolution continuation after stack departure |
| 608.3b (bestow) | Phase 9 | Bestow fallback |
| 608.3d | Phase 9 | Mutating creature spell → merge |
| 608.3e | Phase 6 | ETB prohibition fallback |
| 608.3f | Phase 8 | Spell-copy → token |
| 608.3g | Phase 8 | Stack static → delayed trigger on ETB |

---

## Session Statistics

- **Total sub-rules processed:** ~156
- **TESTABLE atoms generated:** ~113
- **BOUNDARY-DEF atoms generated:** 14
- **PURE-DEF rules (no test):** 22
- **ALREADY-IMPLEMENTED rules:** 19
- **DEFERRED rules:** 31
- **OUT-OF-SCOPE rules:** 3
- **Composition tests:** 7
- **New tickets identified:** 5
- **META entries:** 6
