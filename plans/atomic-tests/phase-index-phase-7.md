# Phase 7 — Test Index

> 202 entries extracted from session summaries

---

## Phase 7

**202 entries**

| ID | Rule | Summary | Ticket | Session | Tags |
|----|------|---------|--------|---------|------|
| ATOM-101.4-001 | 101.4 | Smallpox APNAP simultaneous choices | NEW-CH1-001 | S1 |  |
| ATOM-101.4c-001 | 101.4c | Simultaneous choices are independent | NEW-CH1-001 | S1 |  |
| ATOM-106.12a-001 | 106.12a | "Tapped for mana" event tracking | DEFERRED | S1 |  |
| ATOM-107.3e-001 | 107.3e | X in triggered ability retains value from source | DEFERRED | S1 |  |
| ATOM-107.3n-001 | 107.3n | Delayed trigger X persistence | DEFERRED | S1 |  |
| ATOM-111.13-001 | 111.13 | Copy of permanent spell becomes token (not "created") | D19 | S1 |  |
| ATOM-112.2-002 | 112.2 | Spell copy controller = copier | D19 | S1 |  |
| ATOM-113.9-003 | 113.9 | CounterTriggeredAbility primitive works | Phase 7 | S1 |  |
| ATOM-115.9a-001 | 115.9a | Count targets on a spell | Phase 7 | S1 |  |
| ATOM-117.2a-001 | 117.2a | Triggers placed on stack before priority | Phase 7 | S1 |  |
| ATOM-118.12-001 | 118.12 | "If you do" fails when cost object gone | Phase 7 | S1 |  |
| ATOM-119.9-001 | 119.9 | Each lifelink source triggers separately | Phase 7 | S1 |  |
| ATOM-119.9-002 | 119.9 | Gaining 0 life doesn't trigger | Phase 7 | S1 |  |
| ATOM-120.10-001 | 120.10 | Excess damage = amount beyond lethal | Phase 7 | S1 |  |
| ATOM-121.5-001 | 121.5 | Library-to-hand without "draw" is not a draw | Phase 7 | S1 |  |
| ATOM-122.7-001 | 122.7 | "Nth counter" trigger fires on threshold | Phase 7 | S1 |  |
| ATOM-122.8-001 | 122.8 | Triggered ability creates new counters (not move) | Phase 7 | S1 |  |
| ATOM-122.9-001 | 122.9 | Activated ability creates new counters (not move) | Phase 7 | S1 |  |
| ATOM-208.2b-001 | 208.2b | Replacement-effect P/T = 0/0 off-battlefield | L04, NEW-S2-15 | S2 |  |
| ATOM-208.2b-002 | 208.2b | Replacement-effect P/T choice becomes copiable | NEW-S2-16 | S2 |  |
| ATOM-400.7-001 | 400.7 | New object identity on zone change (epoch-stamp) | NEW — zone_change_epoch infrastructure | S4 |  |
| ATOM-400.7e-001 | 400.7e | LTB trigger finds new object in public zone (Rancor) | NEW — zone-change trigger object tracking | S4 |  |
| ATOM-400.7e-002 | 400.7e | "From anywhere" triggers don't need 400.7e (Vigor) | NEW — "from anywhere" trigger distinction | S4 |  |
| ATOM-400.7f-001 | 400.7f | Aura LTB trigger finds aura in graveyard (simultaneous) | NEW — Aura LTB simultaneous graveyard | S4 |  |
| ATOM-405.3-001 | 405.3 | APNAP simultaneous stack placement | NEW — APNAP stack placement | S4 |  |
| ATOM-405.3-002 | 405.3 | Player chooses order of own simultaneous triggers | NEW — player trigger ordering via DP | S4 |  |
| ATOM-500.6-001 | 500.6 | "At the beginning of" triggers fire at step/phase start | NEW — step/phase beginning trigger checking | S4 |  |
| COMP-UNTAP-TRIGGER-UPKEEP-001 | 502.3 + 502.4 + 503.1a | Untap trigger held until upkeep | NEW | S4 | ATOM-502.3-001, ATOM-502.4-001, ATOM-503.1a-001 |
| ATOM-502.4-001 | 502.4 | No priority during untap; triggers held until upkeep | NEW — trigger hold-and-release | S4 |  |
| ATOM-503.1a-001 | 503.1a | Untap-step + upkeep triggers on stack before priority | NEW — trigger batch at upkeep | S4 |  |
| ATOM-508.1m-001 | 508.1m | Declare-attacker triggers fire | NEW — declare-attacker triggers | S4 |  |
| ATOM-508.2a-001 | 508.2a | Attack trigger characteristic snapshot at declaration time | NEW — trigger characteristic snapshot | S4 | META-trigger-timing |
| ATOM-511.2-001 | 511.2 | "At end of combat" triggers fire | NEW — end-of-combat triggers | S4 |  |
| ATOM-513.2-001 | 513.2 | End-step trigger "no back up" rule | NEW — end-step no-back-up | S4 |  |
| ATOM-513.2-002 | 513.2 | Delayed trigger "next end step" waits if created during end step | NEW — delayed trigger timing | S4 |  |
| COMP-CLEANUP-RELOOP-001 | 514.1 + 514.2 + 514.3a | Cleanup step triggers re-loop | T16 | S4 | ATOM-514.1-001, ATOM-514.2-001, ATOM-514.3a-001, ATOM-514.3a-002 |
| ATOM-601.2i-001 | 601.2i | SpellCast event emitted after all steps complete | Phase 7 | S5 |  |
| ATOM-603.1b-001 | 603.1b | Multi-condition trigger tracking | Phase 7 | S5 |  |
| ATOM-603.2-001 | 603.2 | Trigger detection (no immediate effect) | Phase 7 | S5 |  |
| ATOM-603.2a-001 | 603.2a | Triggers bypass activation prohibitions | Phase 7 | S5 |  |
| ATOM-603.2b-001 | 603.2b | Phase/step-begin trigger | Phase 7 | S5 |  |
| ATOM-603.2c-001 | 603.2c | Per-occurrence triggering | Phase 7 | S5 |  |
| ATOM-603.2d-001 | 603.2d | Trigger multiplier | Phase 7 | S5 |  |
| ATOM-603.2e-001 | 603.2e | "Becomes tapped" doesn't trigger on ETB tapped | Phase 7 | S5 |  |
| ATOM-603.2e-002 | 603.2e | "Becomes" requires state change — no re-trigger on same state | Phase 7 | S5 |  |
| ATOM-603.2f-001 | 603.2f | Hidden-zone trigger suppression | Phase 7 | S5 |  |
| ATOM-603.2g-001 | 603.2g | Prevented events don't cause triggers | Phase 7 | S5 |  |
| ATOM-603.2h-001 | 603.2h | Once-per-turn trigger restriction | Phase 7 | S5 |  |
| ATOM-603.2h-002 | 603.2h | Once-per-turn checked at resolution | Phase 7 | S5 |  |
| ATOM-603.3-001 | 603.3 | Trigger → stack placement timing | Phase 7 | S5 |  |
| ATOM-603.3a-001 | 603.3a | Trigger controller = source controller at trigger time | Phase 7 | S5 |  |
| ATOM-603.3b-001 | 603.3b | APNAP trigger stacking order | Phase 7 | S5 |  |
| ATOM-603.3b-002 | 603.3b | Two-tier trigger stacking order | Phase 7 | S5 |  |
| ATOM-603.3c-001 | 603.3c | Modal trigger mode selection | Phase 7 | S5 |  |
| ATOM-603.3c-002 | 603.3c | Modal trigger fizzle when no mode is legal | Phase 7 | S5 |  |
| ATOM-603.3d-001 | 603.3d | Trigger target selection follows casting rules | Phase 7 | S5 |  |
| ATOM-603.4-001 | 603.4 | Intervening-if double check (true at both times) | Phase 7 | S5 |  |
| ATOM-603.4-002 | 603.4 | Intervening-if blocks trigger | Phase 7 | S5 |  |
| ATOM-603.4-003 | 603.4 | Intervening-if fails at resolution | Phase 7 | S5 |  |
| ATOM-603.5-001 | 603.5 | Optional trigger still stacks | Phase 7 | S5 |  |
| ATOM-603.6-001 | 603.6 | Zone-change trigger resolution finds object in new zone | Phase 7 | S5 |  |
| ATOM-603.6a-001 | 603.6a | ETB trigger detection including newcomer self-trigger | Phase 7 | S5 |  |
| ATOM-603.6c-001 | 603.6c | LTB trigger zone tracking | Phase 7 | S5 |  |
| ATOM-603.6c-002 | 603.6c | First-zone-only tracking on LTB trigger | Phase 7 | S5 |  |
| ATOM-603.6e-001 | 603.6e | Aura LTB trigger cross-zone resolution | Phase 7, T15 | S5 |  |
| ATOM-603.6e-002 | 603.6e | Aura trigger tracks enchanted creature across zones | Phase 7, T15 | S5 |  |
| ATOM-603.7-001 | 603.7 | Delayed trigger creation and storage | Phase 7 | S5 |  |
| ATOM-603.7a-001 | 603.7a | No retroactive delayed triggers | Phase 7 | S5 |  |
| ATOM-603.7b-001 | 603.7b | One-shot delayed trigger | Phase 7 | S5 |  |
| ATOM-603.7b-002 | 603.7b | Simultaneous-event delayed trigger choice | Phase 7 | S5 |  |
| ATOM-603.7c-001 | 603.7c | Delayed trigger tracks object identity not characteristics | Phase 7 | S5 |  |
| ATOM-603.7d-001 | 603.7d | Delayed trigger source/controller from spell | Phase 7 | S5 |  |
| ATOM-603.7e-001 | 603.7e | Delayed trigger source inheritance from ability | Phase 7 | S5 |  |
| ATOM-603.7f-001 | 603.7f | Delayed trigger from replacement inherits static source | Phase 7 | S5 |  |
| ATOM-603.7g-001 | 603.7g | Static-action delayed trigger source/controller | Phase 7 | S5 |  |
| ATOM-603.7h-001 | 603.7h | Resolution-count delayed trigger | Phase 7 | S5 |  |
| ATOM-603.8-001 | 603.8 | State-trigger one-shot until resolution | Phase 7 | S5 |  |
| ATOM-603.8-002 | 603.8 | State trigger fires during spell resolution | Phase 7 | S5 |  |
| ATOM-603.9-001 | 603.9 | Player-loss trigger | Phase 7 | S5 |  |
| ATOM-603.10a-001 | 603.10a | LTB trigger look-back | Phase 7 | S5 |  |
| ATOM-603.10a-002 | 603.10a | Death trigger on dying creature | Phase 7 | S5 |  |
| ATOM-603.10c-001 | 603.10c | Unattach trigger look-back | Phase 7, T04 | S5 |  |
| ATOM-603.10c-002 | 603.10c | Re-attach triggers unattach from original | Phase 7, T04 | S5 |  |
| ATOM-603.10c-003 | 603.10c | Re-attach to different creature triggers "becomes attached" | Phase 7, T04 | S5 |  |
| ATOM-603.10d-001 | 603.10d | Control-change trigger look-back | Phase 7, L11 | S5 |  |
| ATOM-603.10e-001 | 603.10e | Counter-trigger look-back | Phase 7 | S5 |  |
| ATOM-603.12-001 | 603.12 | Reflexive trigger created during resolution | Phase 7 | S5 |  |
| ATOM-603.12a-001 | 603.12a | Reflexive trigger coalesces multiple payments | Phase 7 | S5 |  |
| ATOM-605.1b-001 | 605.1b | Triggered mana ability classification | Phase 7 | S5 |  |
| ATOM-605.3a-002 | 605.3a | Mana ability window during rule-required payment | Phase 7 | S5 |  |
| ATOM-605.4a-001 | 605.4a | Triggered mana ability immediate resolution | Phase 7 | S5 |  |
| ATOM-605.5a-001 | 605.5a | Target/wrong-trigger-source disqualifies mana ability | Phase 7 | S5 |  |
| ATOM-608.2-001 | 608.2 | Full resolution procedure order verification | T18 | S5 |  |
| ATOM-608.2a-001 | 608.2a | Intervening-if resolution check | Phase 7 | S5 |  |
| ATOM-608.2d-001 | 608.2d | Resolution-time choice announcement | Phase 7, T18 | S5 |  |
| ATOM-608.2k-001 | 608.2k | Untargeted-reference persistence | Phase 7 | S5 |  |
| ATOM-608.2p-001 | 608.2p | Resolution-count tracking (Ashling Flame Dancer) | Phase 7 | S5 |  |
| ATOM-610.3-001 | 610.3 | "Until leaves" zone-change return effect (O-Ring) | NEW-610.3 | S6 |  |
| ATOM-610.3a-001 | 610.3a | "Until" event already occurred before spell resolves | NEW-610.3 | S6 |  |
| ATOM-610.3b-001 | 610.3b | "Until" event occurred before triggered ability resolves | NEW-610.3 | S6 |  |
| ATOM-610.3c-001 | 610.3c | "Until" return goes to owner's control | NEW-610.3 | S6 |  |
| ATOM-610.3c-002 | 610.3c | "Until" return under specific controller override | NEW-610.3 | S6 |  |
| ATOM-610.3d-001 | 610.3d | Simultaneous "until" returns are simultaneous | NEW-610.3 | S6 |  |
| ATOM-610.5-001 | 610.5 | Static ability grants Convoke at cast time | NEW-610.5 | S6 |  |
| ATOM-610.5-002 | 610.5 | Grant persists after granting source destroyed | NEW-610.5 | S6 |  |
| ATOM-611.2e-001 | 611.2e | "Is [characteristic]" applies simultaneously with entering | NEW-611.2e | S6 |  |
| ATOM-611.3d-001 | 611.3d | Static grant persists after source leaves | NEW-611.3d | S6 |  |
| ATOM-611.3d-002 | 611.3d | Dream Devourer foretell grant persists | NEW-611.3d | S6 |  |
| ATOM-700.2b-001 | 700.2b | Modal triggered ability — mode choice + removal if no legal mode | NEW | S7a |  |
| ATOM-700.2g-001 | 700.2g | Spell copy preserves modes | NEW (D19) | S7a |  |
| ATOM-701.43d-001 | 701.43d | Exert as optional attack cost + linked trigger | NEW | S7a |  |
| ATOM-701.66a-002 | 701.66a | Earthbend — animated land dies, delayed trigger returns it | NEW | S7a |  |
| ATOM-702.21a-001 | 702.21a | Ward trigger: counter unless pay | NEW-1 | S7b | ward, triggered-ability, counter-unless-pay |
| ATOM-702.21a-002 | 702.21a | Ward doesn't trigger for controller's own targeting | NEW-1 | S7b | ward, triggered-ability, self-targeting |
| ATOM-702.82b-001 | 702.82b | "It devoured" linked ability references sacrifice count | (same) | S8 | LINKED-ABILITY-PATTERN |
| ATOM-702.83a-001 | 702.83a | Exalted: +1/+1 on solo attack trigger | NEW — Exalted | S8 |  |
| ATOM-702.83a-002 | 702.83a + 506.5 | Exalted triggers even when tokens enter attacking | (same) | S8 | DECLARED-VS-ENTERS-ATTACKING |
| ATOM-702.85a-001 | 702.85a | Cascade: exile loop, find cheaper nonland, may cast free | NEW — Cascade | S8 |  |
| ATOM-702.85a-003 | 702.85a | Cascade: player declines to cast found spell | (same) | S8 |  |
| ATOM-702.85c-001 | 702.85c | Multiple cascade instances trigger separately | NEW — Cascade multi | S8 |  |
| COMP-CASCADE-STORM-001 | ATOM-702.85a + Storm 702.40 | Storm copies don't trigger cascade (copies aren't cast) | NEW — Cascade + Storm | S8 |  |
| ATOM-702.86a-001 | 702.86a | Annihilator N: attack trigger, sacrifice N permanents | NEW — Annihilator | S8 |  |
| ATOM-702.91a-001 | 702.91a | Battle cry: attack trigger, +1/+0 to other attackers | NEW — Battle Cry | S8 |  |
| ATOM-702.92a-001 | 702.92a | Living weapon: ETB creates 0/0 Germ, auto-attach | NEW — Living Weapon | S8 |  |
| ATOM-702.93a-001 | 702.93a | Undying: dies trigger, return with +1/+1 if had none | NEW — Undying | S8 | SHARED-BEHAVIOR persist |
| ATOM-702.94a-001 | 702.94a | Miracle: first-draw reveal, triggered alt-cost cast | NEW — Miracle | S8 |  |
| ATOM-702.95a-001 | 702.95a | Soulbond: pair with unpaired creature on ETB | NEW — Soulbond | S8 |  |
| ATOM-702.95a-002 | 702.95a | Soulbond second trigger: another creature enters | (same) | S8 |  |
| ATOM-702.95c-001 | 702.95c | Soulbond pairing fails if conditions unmet at resolution | (same) | S8 |  |
| ATOM-702.95e-001 | 702.95e | Paired creature unpairs if partner leaves BF | (same) | S8 |  |
| ATOM-702.100a-001 | 702.100a | Evolve: ETB trigger, P or T greater → +1/+1 counter | NEW — Evolve | S8 |  |
| ATOM-702.100a-002 | 702.100a | Evolve triggers on toughness-only greater | (same) | S8 |  |
| ATOM-702.100b-001 | 702.100b | "Creature evolves" = counter placed by evolve | (same) | S8 | LINKED-ABILITY-PATTERN |
| ATOM-702.100b-002 | 702.100b | Evolve: no counter placed → doesn't "evolve" | (same) | S8 |  |
| ATOM-702.101a-001 | 702.101a | Extort: cast trigger, optional {W/B}, drain opponents | NEW — Extort | S8 |  |
| COMP-BESTOW-UNDYING-001 | ATOM-702.103f + ATOM-702.93a | Bestowed Aura unattaches when undying creature dies+returns | — | S8 |  |
| ATOM-702.104b-001 | 702.104b | Tribute not paid → conditional trigger fires | (same) | S8 |  |
| ATOM-702.104b-002 | 702.104b | Tribute paid → conditional trigger does NOT fire | (same) | S8 |  |
| ATOM-702.105a-001 | 702.105a | Dethrone: attack trigger, +1/+1 vs highest life | NEW — Dethrone | S8 |  |
| ATOM-702.108a-001 | 702.108a | Prowess: noncreature cast trigger, +1/+1 until EOT | NEW — Prowess | S8 |  |
| ATOM-702.108a-002 | 702.108a | Prowess does NOT trigger on creature spell | (same) | S8 |  |
| ATOM-702.110a-001 | 702.110a | Exploit: ETB trigger, may sacrifice a creature | NEW — Exploit | S8 |  |
| ATOM-702.110b-001 | 702.110b | Exploit: "creature it exploited" linked ability | (same) | S8 | LINKED-ABILITY-PATTERN |
| ATOM-702.112a-001 | 702.112a | Renown N: combat damage trigger, counters + designation | NEW — Renown | S8 |  |
| ATOM-702.112c-001 | 702.112c | Multiple renown: first sets renowned, rest no-op | (same) | S8 |  |
| ATOM-702.115a-001 | 702.115a | Ingest: combat damage → exile top of opponent's library | NEW — Ingest | S8 |  |
| ATOM-702.116a-001 | 702.116a | Myriad: attack trigger, token copies per opponent | NEW — Myriad | S8 |  |
| ATOM-702.121a-001 | 702.121a | Melee: +1/+1 per opponent attacked this combat | NEW — Melee | S8 |  |
| ATOM-702.122b-001 | 702.122b | "Crews a Vehicle" trigger event | (same) | S8 | CREW-EVENT |
| ATOM-702.123a-001 | 702.123a | Fabricate N: choose counters or Servo tokens on ETB | NEW — Fabricate | S8 |  |
| ATOM-702.123a-002 | 702.123a | Fabricate: choosing counters path | (same) | S8 |  |
| ATOM-702.130a-001 | 702.130a | Afflict N: becomes-blocked trigger, defender loses N life | NEW — Afflict | S8 |  |
| ATOM-702.134a-001 | 702.134a | Mentor: attack trigger, +1/+1 on weaker attacker | NEW — Mentor | S8 |  |
| ATOM-702.135a-001 | 702.135a | Afterlife N: dies trigger, N Spirit tokens | NEW — Afterlife | S8 |  |
| ATOM-702.141a-001 | 702.141a | Encore: GY activated, per-opponent tokens, forced attack | NEW — Encore | S8 |  |
| ATOM-702.144a-001 | 702.144a | Demonstrate: copy spell, opponent also copies | NEW — Demonstrate | S8 |  |
| ATOM-702.149a-001 | 702.149a | Training: co-attacker power trigger, +1/+1 counter | NEW — Training | S8 |  |
| ATOM-702.153a-001 | 702.153a | Casualty N: sacrifice ≥ N power → copy spell | NEW — Casualty | S8 |  |
| ATOM-702.154a-001 | 702.154a | Enlist: tap creature on attack for +X/+0 | NEW — Enlist | S8 |  |
| ATOM-702.156a-001 | 702.156a | Ravenous: enters with X counters, draw if X ≥ 5 | NEW — Ravenous | S8 |  |
| ATOM-702.156a-002 | 702.156a | Ravenous: no draw if X < 5 | (same) | S8 |  |
| ATOM-702.157a-001 | 702.157a | Squad: pay additional cost N times, N token copies | NEW — Squad | S8 |  |
| ATOM-702.163a-001 | 702.163a | For Mirrodin!: ETB creates 2/2 Rebel, auto-attach | NEW — For Mirrodin! | S8 | SHARED-BEHAVIOR for-mirrodin-job-select |
| ATOM-702.164a-001 | 702.164a | Toxic N: combat damage → N poison (additive) | NEW — Toxic | S8 |  |
| ATOM-702.164b-001 | 702.164b | Multiple toxic instances trigger separately | (same) | S8 |  |
| ATOM-702.165a-001 | 702.165a | Backup N: ETB counters + conditional ability grant | NEW — Backup | S8 | TRIGGERED-ABILITY-INFRA 603.3c |
| ATOM-702.165a-002 | 702.165a | Backup targeting self: counters but no ability grant | (same) | S8 |  |
| ATOM-702.165c-001 | 702.165c | Backup: only printed abilities granted, not gained | (same) | S8 | ARCHITECTURE-NOTE printed-vs-granted |
| ATOM-702.165d-001 | 702.165d | Backup: abilities locked in at trigger stack time | (same) | S8 |  |
| ATOM-702.174a-002 | 702.174a | Gift on permanent: opponent choice + ETB trigger | (same) | S8 |  |
| ATOM-702.175a-001 | 702.175a | Offspring: pay additional cost, ETB 1/1 token copy | NEW — Offspring | S8 |  |
| ATOM-702.176a-003 | 702.176a | Impending: end step remove time counter → becomes creature | (same) | S8 |  |
| ATOM-702.179d-001 | 702.179d | Speed inherent trigger: opp life loss → speed +1 | (same) | S8 |  |
| ATOM-702.181a-001 | 702.181a | Mobilize N: attack trigger, N Warrior tokens tapped+attacking | NEW — Mobilize | S8 |  |
| ATOM-702.182a-001 | 702.182a | Job select: ETB creates 1/1 Hero + auto-attach | NEW — Job Select | S8 | SHARED-BEHAVIOR for-mirrodin-job-select |
| ATOM-702.185a-002 | 702.185a | Warp: permanent exiled at end step, castable from exile | (same) | S8 |  |
| ATOM-702.189a-001 | 702.189a | Firebending N: attack trigger, add N {R}, persists through combat | NEW — Firebending | S8 |  |
| ATOM-707.5-002 | 707.5 | "Enters as a copy" — copied ETB triggers fire | D5 + Phase 7 | S9a | copy, triggers |
| BOUNDARY-707.7-001 | 707.7 | Copied linked abilities remain linked, can't cross-link | T07-linked | S9a | copy, linked-abilities |
| BOUNDARY-707.9g-001 | 707.9g | Copy-effect linked trigger invalidated by subsequent copy | D5 + Phase 7 | S9a | copy, triggers |
| ATOM-707.10b-001 | 707.10b | Ability copy has same source — counter goes on source | D5 + Phase 7 | S9a | copy, triggers |
| ATOM-714.2b-001 | 714.2b | Chapter trigger on lore counter threshold crossing | NEW — Saga chapter trigger | S9b |  |
| ATOM-714.2b-002 | 714.2b | Multi-chapter trigger on bulk counter add | (same) | S9b |  |
| ATOM-714.2b-003 | 714.2b | Chapter trigger suppression when threshold already met | (same) | S9b |  |
| ATOM-714.2c-001 | 714.2c | Multi-numeral chapter symbol fires at each number | NEW — Saga multi-numeral | S9b |  |
| ATOM-714.2d-001 | 714.2d | Final chapter number = greatest chapter numeral | NEW — Saga final chapter | S9b |  |
| ATOM-714.3a-001 | 714.3a | Saga ETB with 1 lore counter (no read ahead) | NEW — Saga ETB counter | S9b |  |
| ATOM-714.3b-001 | 714.3b | Precombat main phase lore counter TBA | NEW — Saga TBA | S9b |  |
| COMP-SAGA-LIFECYCLE-001 | 714.3a + 714.2b + 714.4 | Full Saga lifecycle: ETB → chapters → sacrifice | Phase 7 + 8 | S9b | NEW |
| ATOM-714.4-001 | 714.4 | Saga sacrifice SBA when counters ≥ final chapter | NEW — Saga sacrifice SBA | S9b |  |
| ATOM-714.4-002 | 714.4 | Saga sacrifice deferred while chapter on stack | (same) | S9b |  |
| ATOM-719.3a-001 | 719.3a | Case "To solve" end-step triggered ability | NEW — Case solve trigger | S9b |  |
| ATOM-719.3a-002 | 719.3a | Case solve trigger suppressed when already solved | (same) | S9b |  |
| ATOM-724.1-001 | 724.1 | Monarch designation initialization | D15 | S9b |  |
| ATOM-724.2-001 | 724.2 | Monarch end-step card draw trigger | D15 | S9b |  |
| ATOM-724.2-002 | 724.2 | Monarch combat damage transfer trigger | D15 | S9b |  |
| COMP-MONARCH-COMBAT-001 | 724.2 + 724.3 | Monarch combat transfer | Phase 7 (D15) | S9b | D15 |
| ATOM-724.3-001 | 724.3 | Monarch exclusivity (one at a time) | D15 | S9b |  |
| ATOM-724.5-001 | 724.5 | Monarch-dependent effect null-check (no monarch) | D15 | S9b |  |
| ATOM-725.1-001 | 725.1 | Initiative designation initialization | D15 | S9b |  |
| ATOM-725.2-002 | 725.2 | Initiative combat damage transfer trigger | D15 | S9b |  |
| ATOM-725.3-001 | 725.3 | Initiative exclusivity (one at a time) | D15 | S9b |  |
