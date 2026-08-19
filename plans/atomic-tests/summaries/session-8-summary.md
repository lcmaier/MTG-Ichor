# Session 8 — Condensed Summary (702.81–702.190)

**Source:** `plans/atomic-tests/session-8.md`
**Scope:** 110 keywords (Retrace through Sneak)

---

## 1. ATOM Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-702.81a-001 | 702.81a | Retrace: cast from GY by discarding land | Phase 8 | NEW — Retrace | |
| ATOM-702.82a-001 | 702.82a | Devour N: sacrifice creatures on ETB, N×devour counters | Phase 6 + 8 | NEW — Devour | |
| ATOM-702.82b-001 | 702.82b | "It devoured" linked ability references sacrifice count | Phase 7 + 8 | (same) | LINKED-ABILITY-PATTERN |
| ATOM-702.83a-001 | 702.83a | Exalted: +1/+1 on solo attack trigger | Phase 7 + 8 | NEW — Exalted | |
| ATOM-702.83a-002 | 702.83a + 506.5 | Exalted triggers even when tokens enter attacking | Phase 7 + 8 | (same) | DECLARED-VS-ENTERS-ATTACKING |
| ATOM-702.84a-001 | 702.84a | Unearth: return from GY with haste, delayed exile | Phase 8 | NEW — Unearth | |
| ATOM-702.84a-002 | 702.84a | Unearth: "leave BF → exile instead" replacement | Phase 6 + 8 | (same) | |
| ATOM-702.85a-001 | 702.85a | Cascade: exile loop, find cheaper nonland, may cast free | Phase 7 + 8 | NEW — Cascade | |
| ATOM-702.85a-002 | 702.85a | Cascade checks resulting spell's MV (split cards) | Phase 9 | NEW — Cascade + split MV | |
| ATOM-702.85a-003 | 702.85a | Cascade: player declines to cast found spell | Phase 7 + 8 | (same) | |
| ATOM-702.85c-001 | 702.85c | Multiple cascade instances trigger separately | Phase 7 + 8 | NEW — Cascade multi | |
| ATOM-702.86a-001 | 702.86a | Annihilator N: attack trigger, sacrifice N permanents | Phase 7 + 8 | NEW — Annihilator | |
| ATOM-702.87a-001 | 702.87a | Level up: sorcery-speed activated, level counter | Phase 8 | NEW — Level Up | CROSS-REF rule 711 |
| ATOM-702.88a-001 | 702.88a | Rebound: exile instead of GY, free cast next upkeep | Phase 6 + 7 + 8 | NEW — Rebound | |
| ATOM-702.88a-002 | 702.88a | Rebound only works if cast from hand | Phase 8 | (same) | |
| ATOM-702.89a-001 | 702.89a | Umbra armor: replace destruction of enchanted permanent | Phase 6 + 8 | NEW — Umbra Armor | |
| ATOM-702.89a-002 | 702.89a | Umbra armor prevents destruction from lethal damage | Phase 6 + 8 | (same) | |
| ATOM-702.90d-001 | 702.90d | Infect: LKI determines infect on zone-changed source | Phase 5 (LKI) | L18 | |
| ATOM-702.90e-001 | 702.90e | Infect functions regardless of zone | Phase 8 | T21c | SHARED-BEHAVIOR wither |
| ATOM-702.91a-001 | 702.91a | Battle cry: attack trigger, +1/+0 to other attackers | Phase 7 + 8 | NEW — Battle Cry | |
| ATOM-702.92a-001 | 702.92a | Living weapon: ETB creates 0/0 Germ, auto-attach | Phase 7 + 8 | NEW — Living Weapon | |
| ATOM-702.93a-001 | 702.93a | Undying: dies trigger, return with +1/+1 if had none | Phase 7 + 8 | NEW — Undying | SHARED-BEHAVIOR persist |
| ATOM-702.94a-001 | 702.94a | Miracle: first-draw reveal, triggered alt-cost cast | Phase 7 + 8 | NEW — Miracle | |
| ATOM-702.95a-001 | 702.95a | Soulbond: pair with unpaired creature on ETB | Phase 7 + 8 | NEW — Soulbond | |
| ATOM-702.95c-001 | 702.95c | Soulbond pairing fails if conditions unmet at resolution | Phase 7 + 8 | (same) | |
| ATOM-702.95e-001 | 702.95e | Paired creature unpairs if partner leaves BF | Phase 7 + 8 | (same) | |
| ATOM-702.95a-002 | 702.95a | Soulbond second trigger: another creature enters | Phase 7 + 8 | (same) | |
| ATOM-702.95e-002 | 702.95e | Soulbond: type loss → unpaired | Phase 5 + 7 + 8 | (same) | |
| ATOM-702.95e-003 | 702.95e | Soulbond: controller change → unpaired | Phase 5 + 7 + 8 | (same) | |
| ATOM-702.96a-001 | 702.96a | Overload: alt cost, "target" → "each" text change | Phase 5 + 8 | NEW — Overload | TEXT-CHANGING-EFFECT |
| ATOM-702.96b-001 | 702.96b | Overloaded spell has no targets, hits untargetable | Phase 5 + 8 | (same) | |
| ATOM-702.97a-001 | 702.97a | Scavenge: GY activated, exile self, counters on target | Phase 8 | NEW — Scavenge | |
| ATOM-702.98a-001 | 702.98a | Unleash: optional ETB counter + can't block with counter | Phase 5 + 6 + 8 | NEW — Unleash | |
| ATOM-702.98a-002 | 702.98a | Unleash without counter CAN block | Phase 5 + 8 | (same) | |
| ATOM-702.98a-003 | 702.98a | Unleash: can't block with +1/+1 from any source | Phase 5 + 8 | (same) | |
| ATOM-702.99a-001 | 702.99a | Cipher: encode on creature, combat damage → copy + free cast | Phase 8 | NEW — Cipher | |
| ATOM-702.99a-003 | 702.99a | Cipher: copy of cipher spell not encoded | Phase 8 | (same) | |
| ATOM-702.99c-001 | 702.99c | Cipher encoding persists through controller change | Phase 8 | (same) | |
| ATOM-702.100a-001 | 702.100a | Evolve: ETB trigger, P or T greater → +1/+1 counter | Phase 7 + 8 | NEW — Evolve | |
| ATOM-702.100a-002 | 702.100a | Evolve triggers on toughness-only greater | Phase 7 + 8 | (same) | |
| ATOM-702.100b-001 | 702.100b | "Creature evolves" = counter placed by evolve | Phase 7 + 8 | (same) | LINKED-ABILITY-PATTERN |
| ATOM-702.100b-002 | 702.100b | Evolve: no counter placed → doesn't "evolve" | Phase 7 + 8 | (same) | |
| ATOM-702.101a-001 | 702.101a | Extort: cast trigger, optional {W/B}, drain opponents | Phase 7 + 8 | NEW — Extort | |
| ATOM-702.102a-001 | 702.102a | Fuse: cast both halves of split card from hand | Phase 9 | NEW — Fuse | |
| ATOM-702.102c-001 | 702.102c | Fused spell's total cost = both halves combined | Phase 9 | (same) | |
| ATOM-702.102d-001 | 702.102d | Fused spell resolves left half first, then right | Phase 9 | (same) | |
| ATOM-702.103a-001 | 702.103a | Bestow: cast creature as Aura for alt cost | Phase 5 + 8 | NEW — Bestow | |
| ATOM-702.103d-001 | 702.103d | Bestow: castability uses Aura characteristics | Phase 5 + 8 | (same) | |
| ATOM-702.103e-001 | 702.103e | Bestow: illegal target → enters as creature | Phase 8 | (same) | |
| ATOM-702.103f-001 | 702.103f | Bestow: unattach → ceases bestow, becomes creature | Phase 8 | (same) | |
| ATOM-702.104a-001 | 702.104a | Tribute N: opponent chooses counters on ETB | Phase 6 + 7 + 8 | NEW — Tribute | LINKED-ABILITY-PATTERN |
| ATOM-702.104b-001 | 702.104b | Tribute not paid → conditional trigger fires | Phase 7 + 8 | (same) | |
| ATOM-702.104b-002 | 702.104b | Tribute paid → conditional trigger does NOT fire | Phase 7 + 8 | (same) | |
| ATOM-702.105a-001 | 702.105a | Dethrone: attack trigger, +1/+1 vs highest life | Phase 7 + 8 | NEW — Dethrone | |
| ATOM-702.107a-001 | 702.107a | Outlast: sorcery-speed tap activated, +1/+1 counter | Phase 8 | NEW — Outlast | |
| ATOM-702.108a-001 | 702.108a | Prowess: noncreature cast trigger, +1/+1 until EOT | Phase 7 + 8 | NEW — Prowess | |
| ATOM-702.108a-002 | 702.108a | Prowess does NOT trigger on creature spell | Phase 7 + 8 | (same) | |
| ATOM-702.109a-001 | 702.109a | Dash: alt cost, haste, return to hand at end step | Phase 8 | NEW — Dash | |
| ATOM-702.109a-002 | 702.109a | Dash: normal cast → no haste/return | Phase 8 | (same) | |
| ATOM-702.110a-001 | 702.110a | Exploit: ETB trigger, may sacrifice a creature | Phase 7 + 8 | NEW — Exploit | |
| ATOM-702.110b-001 | 702.110b | Exploit: "creature it exploited" linked ability | Phase 7 + 8 | (same) | LINKED-ABILITY-PATTERN |
| ATOM-702.111b-001 | 702.111b | Menace: requires 2+ blockers | Phase 4 | T21 | |
| ATOM-702.111b-002 | 702.111b | Menace: legal block with 2+ creatures | Phase 4 | T21 | |
| ATOM-702.112a-001 | 702.112a | Renown N: combat damage trigger, counters + designation | Phase 7 + 8 | NEW — Renown | |
| ATOM-702.112c-001 | 702.112c | Multiple renown: first sets renowned, rest no-op | Phase 7 + 8 | (same) | |
| ATOM-702.113a-001 | 702.113a | Awaken: alt cost, land → 0/0 Elemental + N counters | Phase 5 + 8 | NEW — Awaken | |
| ATOM-702.113a-002 | 702.113a | Awaken: original spell effects still apply | Phase 8 | (same) | |
| ATOM-702.113b-001 | 702.113b | Awaken target only chosen if awaken cost paid | Phase 8 | (same) | |
| ATOM-702.114a-001 | 702.114a | Devoid: CDA makes object colorless everywhere | Phase 5 | L05 | |
| ATOM-702.114a-002 | 702.114a | Devoid: colorless despite colored mana symbols | Phase 5 | (same) | |
| ATOM-702.115a-001 | 702.115a | Ingest: combat damage → exile top of opponent's library | Phase 7 + 8 | NEW — Ingest | |
| ATOM-702.116a-001 | 702.116a | Myriad: attack trigger, token copies per opponent | Phase 7 + 9 | NEW — Myriad | |
| ATOM-702.117a-001 | 702.117a | Surge: alt cost if spell cast this turn | Phase 8 | NEW — Surge | |
| ATOM-702.118b-001 | 702.118b | Skulk: can't be blocked by greater power | Phase 8 | NEW — Skulk | |
| ATOM-702.118b-002 | 702.118b | Skulk: CAN be blocked by ≤ power | Phase 8 | (same) | |
| ATOM-702.119a-001 | 702.119a | Emerge: alt cost + sacrifice creature + MV reduction | Phase 5-Pre T17 + 8 | T17 + NEW — Emerge | |
| ATOM-702.120a-001 | 702.120a | Escalate: additional cost per extra mode | Phase 8 | NEW — Escalate | |
| ATOM-702.121a-001 | 702.121a | Melee: +1/+1 per opponent attacked this combat | Phase 7 + 9 | NEW — Melee | |
| ATOM-702.122a-001 | 702.122a | Crew N: tap creatures ≥ N power → animate Vehicle | Phase 8 | NEW — Crew | |
| ATOM-702.122a-002 | 702.122a | Crew fails if total power < N | Phase 8 | (same) | |
| ATOM-702.122-003 | 702.122a | Crew on non-Vehicle (via ability copying) | Phase 8 | (same) | |
| ATOM-702.122b-001 | 702.122b | "Crews a Vehicle" trigger event | Phase 7 + 8 | (same) | CREW-EVENT |
| ATOM-702.122c-001 | 702.122c | "Can't crew Vehicles" restriction | Phase 8 | (same) | |
| ATOM-702.123a-001 | 702.123a | Fabricate N: choose counters or Servo tokens on ETB | Phase 7 + 8 | NEW — Fabricate | |
| ATOM-702.123a-002 | 702.123a | Fabricate: choosing counters path | Phase 7 + 8 | (same) | |
| ATOM-702.125a-001 | 702.125a | Undaunted: cost reduced by {1} per opponent | Phase 5-Pre T17 + 9 | T17 + NEW — Undaunted | |
| ATOM-702.125c-001 | 702.125c | Multiple undaunted instances each reduce separately | Phase 5-Pre T17 + 8 | (same) | |
| ATOM-702.126a-001 | 702.126a | Improvise: tap artifacts for generic mana | Phase 5-Pre T17 | T17 + NEW — Improvise | SHARED-BEHAVIOR convoke-improvise |
| ATOM-702.126a-002 | 702.126a | Improvise: only pays generic, not colored | Phase 5-Pre T17 + 8 | (same) | |
| ATOM-702.127a-001 | 702.127a | Aftermath: cast from GY, exile on leave stack | Phase 9 | NEW — Aftermath | |
| ATOM-702.127a-002 | 702.127a | Aftermath half cannot be cast from hand | Phase 9 | (same) | |
| ATOM-702.128a-001 | 702.128a | Embalm: GY activated, exile self, modified token copy | Phase 8 | NEW — Embalm | |
| ATOM-702.129a-001 | 702.129a | Eternalize: GY activated, 4/4 black Zombie token copy | Phase 8 | NEW — Eternalize | |
| ATOM-702.130a-001 | 702.130a | Afflict N: becomes-blocked trigger, defender loses N life | Phase 7 + 8 | NEW — Afflict | |
| ATOM-702.131a-001 | 702.131a | Ascend on spell: city's blessing if 10+ permanents | Phase 8 | NEW — Ascend | |
| ATOM-702.131b-001 | 702.131b | Ascend on permanent: immediate designation at 10+ | Phase 8 | NEW — Ascend (static) | ARCHITECTURE-CONCERN mid-resolution-static-checks |
| ATOM-702.131b-002 | 702.131b | Ascend: mid-resolution acquisition | Phase 8 | (same) | |
| ATOM-702.131c-001 | 702.131c | City's blessing persists below 10 permanents | Phase 8 | (same) | |
| ATOM-702.132a-001 | 702.132a | Assist: another player pays generic mana | Phase 8 + 9 | NEW — Assist | |
| ATOM-702.132a-002 | 702.132a | Assist: chosen player declines → caster pays full | Phase 8 + 9 | (same) | |
| ATOM-702.133a-001 | 702.133a | Jump-start: cast from GY + discard + exile on leave | Phase 8 | NEW — Jump-Start | |
| ATOM-702.133a-002 | 702.133a | Jump-start from hand → normal GY (no exile) | Phase 8 | (same) | |
| ATOM-702.134a-001 | 702.134a | Mentor: attack trigger, +1/+1 on weaker attacker | Phase 7 + 8 | NEW — Mentor | |
| ATOM-702.135a-001 | 702.135a | Afterlife N: dies trigger, N Spirit tokens | Phase 7 + 8 | NEW — Afterlife | |
| ATOM-702.136a-001 | 702.136a | Riot: choose +1/+1 counter or haste on ETB | Phase 6 + 8 | NEW — Riot | |
| ATOM-702.136a-002 | 702.136a | Riot: choosing counter path | Phase 6 + 8 | (same) | |
| ATOM-702.136b-001 | 702.136b | Multiple riot: each gives separate choice | Phase 6 + 8 | (same) | |
| ATOM-702.137a-001 | 702.137a | Spectacle: alt cost if opponent lost life | Phase 5-Pre T17 + 8 | T17 + NEW — Spectacle | |
| ATOM-702.137a-002 | 702.137a | Spectacle: can't use if no life lost | Phase 8 | (same) | |
| ATOM-702.138a-001 | 702.138a | Escape: cast from GY, exile cards as cost | Phase 5-Pre T17 + 8 | T17 + NEW — Escape | |
| ATOM-702.138c-001 | 702.138c | Escape: enters with additional counters if escaped | Phase 6 + 8 | (same) | |
| ATOM-702.138d-001 | 702.138d | Escape: gains ability if escaped | Phase 8 | (same) | |
| ATOM-702.139a-001 | 702.139a | Companion: reveal from outside, pay {3} to hand | Phase 9 | NEW — Companion | |
| ATOM-702.140a-001 | 702.140a | Mutate: alt cost targeting non-Human creature | Phase 8 | NEW — Mutate | |
| ATOM-702.140b-001 | 702.140b | Mutate: illegal target → enters as creature | Phase 8 | (same) | |
| ATOM-702.140c-001 | 702.140c | Mutate: merge with target, choose top/bottom | Phase 8 | (same) | |
| ATOM-702.140e-001 | 702.140e | Mutated permanent: all abilities from all, other chars from top | Phase 8 | (same) | |
| ATOM-702.141a-001 | 702.141a | Encore: GY activated, per-opponent tokens, forced attack | Phase 7 + 8 + 9 | NEW — Encore | |
| ATOM-702.142a-001 | 702.142a | Boast: activated only if attacked this turn, once/turn | Phase 8 | NEW — Boast | |
| ATOM-702.142a-002 | 702.142a | Boast: can't activate if didn't attack | Phase 8 | (same) | |
| ATOM-702.143a-001 | 702.143a | Foretell: pay {2} exile face-down, cast later for alt cost | Phase 5-Pre T17 + 8 | T17 + NEW — Foretell | |
| ATOM-702.143a-002 | 702.143a | Foretold card can't be cast same turn | Phase 8 | (same) | |
| ATOM-702.144a-001 | 702.144a | Demonstrate: copy spell, opponent also copies | Phase 7 + 8 | NEW — Demonstrate | |
| ATOM-702.147a-001 | 702.147a | Decayed: can't block + attack → sacrifice at EOC | Phase 5 + 7 + 8 | NEW — Decayed | |
| ATOM-702.148a-001 | 702.148a | Cleave: alt cost removes bracketed text | Phase 5 + 8 | NEW — Cleave | TEXT-CHANGING-EFFECT |
| ATOM-702.149a-001 | 702.149a | Training: co-attacker power trigger, +1/+1 counter | Phase 7 + 8 | NEW — Training | |
| ATOM-702.150a-001 | 702.150a | Compleated: reduces starting loyalty per Phyrexian mana | Phase 8 | NEW — Compleated | |
| ATOM-702.151a-001 | 702.151a | Reconfigure: attach/unattach activated, sorcery speed | Phase 8 | NEW — Reconfigure | |
| ATOM-702.151b-001 | 702.151b | Reconfigure: not a creature while attached | Phase 5 + 8 | (same) | |
| ATOM-702.152a-001 | 702.152a | Blitz: alt cost, haste, dies-draw, delayed sacrifice | Phase 8 | NEW — Blitz | |
| ATOM-702.152a-002 | 702.152a | Blitz: normal cast → no blitz effects | Phase 8 | (same) | |
| ATOM-702.153a-001 | 702.153a | Casualty N: sacrifice ≥ N power → copy spell | Phase 7 + 8 | NEW — Casualty | |
| ATOM-702.153a-002 | 702.153a | Casualty: not paying → no copy | Phase 8 | (same) | |
| ATOM-702.154a-001 | 702.154a | Enlist: tap creature on attack for +X/+0 | Phase 7 + 8 | NEW — Enlist | |
| ATOM-702.155b-001 | 702.155b | Read ahead: choose starting lore counter count | Phase 8 | NEW — Read Ahead | |
| ATOM-702.155-002 | 702.155 | Read ahead: counter doubler modifies actual entry | Phase 6 + 8 | (same) | |
| ATOM-702.156a-001 | 702.156a | Ravenous: enters with X counters, draw if X ≥ 5 | Phase 7 + 8 | NEW — Ravenous | |
| ATOM-702.156a-002 | 702.156a | Ravenous: no draw if X < 5 | Phase 7 + 8 | (same) | |
| ATOM-702.157a-001 | 702.157a | Squad: pay additional cost N times, N token copies | Phase 7 + 8 | NEW — Squad | |
| ATOM-702.160a-001 | 702.160a | Prototype: cast with alternate characteristics | Phase 9 | NEW — Prototype | |
| ATOM-702.161a-001 | 702.161a | Living metal: creature during your turn only | Phase 5 + 8 | NEW — Living Metal | |
| ATOM-702.163a-001 | 702.163a | For Mirrodin!: ETB creates 2/2 Rebel, auto-attach | Phase 7 + 8 | NEW — For Mirrodin! | SHARED-BEHAVIOR for-mirrodin-job-select |
| ATOM-702.164a-001 | 702.164a | Toxic N: combat damage → N poison (additive) | Phase 7 + 8 | NEW — Toxic | |
| ATOM-702.164b-001 | 702.164b | Multiple toxic instances trigger separately | Phase 7 + 8 | (same) | |
| ATOM-702.165a-001 | 702.165a | Backup N: ETB counters + conditional ability grant | Phase 7 + 8 | NEW — Backup | TRIGGERED-ABILITY-INFRA 603.3c |
| ATOM-702.165a-002 | 702.165a | Backup targeting self: counters but no ability grant | Phase 7 + 8 | (same) | |
| ATOM-702.165c-001 | 702.165c | Backup: only printed abilities granted, not gained | Phase 7 + 8 | (same) | ARCHITECTURE-NOTE printed-vs-granted |
| ATOM-702.165d-001 | 702.165d | Backup: abilities locked in at trigger stack time | Phase 7 + 8 | (same) | |
| ATOM-702.166a-001 | 702.166a | Bargain: optional sacrifice additional cost | Phase 8 | NEW — Bargain | |
| ATOM-702.167a-001 | 702.167a | Craft: exile self + materials, return transformed | Phase 8 + 9 | NEW — Craft | |
| ATOM-702.168a-001 | 702.168a | Disguise: cast face down 2/2 with ward {2} for {3} | Phase 8 | NEW — Disguise | SHARED-BEHAVIOR morph-disguise |
| ATOM-702.168b-001 | 702.168b | Disguise: turn face up by paying disguise cost | Phase 8 | (same) | |
| ATOM-702.169a-001 | 702.169a | Solved: Case becomes solved when condition met | Phase 8 | NEW — Solved/Case | |
| ATOM-702.170a-001 | 702.170a | Plot: pay plot cost, exile face up, cast free later | Phase 8 | NEW — Plot | ARCHITECTURE-CONCERN special-action-mod |
| ATOM-702.170a-002 | 702.170a | Plotted card can't be cast same turn | Phase 8 | (same) | |
| ATOM-702.170f-001 | 702.170f | Plotted card leaves exile → new object, not plotted | Phase 8 | (same) | |
| ATOM-702.171a-001 | 702.171a | Saddle N: tap creatures ≥ N power, saddled designation | Phase 8 | NEW — Saddle | SHARED-BEHAVIOR crew-saddle |
| ATOM-702.172a-001 | 702.172a | Spree: choose additional costs, each adds a mode | Phase 8 | NEW — Spree | |
| ATOM-702.172a-002 | 702.172 | Spree: must choose at least one mode | Phase 8 | (same) | |
| ATOM-702.173a-001 | 702.173a | Freerunning: alt cost if Assassin/new creature dealt damage | Phase 8 | NEW — Freerunning | COMMANDER-INTERACTION |
| ATOM-702.174a-001 | 702.174a | Gift on instant/sorcery: opponent choice + benefit first | Phase 8 | NEW — Gift | |
| ATOM-702.174a-002 | 702.174a | Gift on permanent: opponent choice + ETB trigger | Phase 7 + 8 | (same) | |
| ATOM-702.174a-003 | 702.174a | Gift: declining (no opponent chosen) | Phase 8 | (same) | |
| ATOM-702.175a-001 | 702.175a | Offspring: pay additional cost, ETB 1/1 token copy | Phase 7 + 8 | NEW — Offspring | |
| ATOM-702.176a-001 | 702.176a | Impending: alt cost → enters with N time counters | Phase 5-Pre T17 + 6 | NEW — Impending | |
| ATOM-702.176a-002 | 702.176a | Impending: not a creature while has time counters | Phase 5 | (same) | |
| ATOM-702.176a-003 | 702.176a | Impending: end step remove time counter → becomes creature | Phase 7 | (same) | |
| ATOM-702.176a-004 | 702.176a | Impending: normal cast → creature immediately | Phase 8 | (same) | |
| ATOM-702.177a-001 | 702.177a | Exhaust: activate-only-once (ever) | Phase 8 | NEW — Exhaust | |
| ATOM-702.177a-002 | 702.177a | Exhaust: zone change resets restriction | Phase 8 | (same) | |
| ATOM-702.178a-001 | 702.178a | Max speed: ability conditional on speed = 4 | Phase 8 | NEW — Max Speed | |
| ATOM-702.179a-001 | 702.179a | Start your engines!: SBA sets speed to 1 | Phase 8 | NEW — Start Your Engines! + Speed | |
| ATOM-702.179d-001 | 702.179d | Speed inherent trigger: opp life loss → speed +1 | Phase 7 + 8 | (same) | |
| ATOM-702.180a-001 | 702.180a | Harmonize: GY cast, tap creature for cost reduction | Phase 5-Pre T17 + 8 | NEW — Harmonize | |
| ATOM-702.180a-002 | 702.180a | Harmonize: exile instead of going anywhere on leave stack | Phase 6 | (same) | |
| ATOM-702.181a-001 | 702.181a | Mobilize N: attack trigger, N Warrior tokens tapped+attacking | Phase 7 + 8 | NEW — Mobilize | |
| ATOM-702.182a-001 | 702.182a | Job select: ETB creates 1/1 Hero + auto-attach | Phase 7 + 8 | NEW — Job Select | SHARED-BEHAVIOR for-mirrodin-job-select |
| ATOM-702.183a-001 | 702.183a | Tiered: choose one mode, pay that mode's additional cost | Phase 8 | NEW — Tiered | |
| ATOM-702.184a-001 | 702.184a | Station: tap creature → charge counters = power | Phase 8 | NEW — Station | |
| ATOM-702.185a-001 | 702.185a | Warp: alt cost from hand | Phase 5-Pre T17 | NEW — Warp | |
| ATOM-702.185a-002 | 702.185a | Warp: permanent exiled at end step, castable from exile | Phase 7 + 8 | (same) | |
| ATOM-702.186b-001 | 702.186b | ∞: ability conditional on harnessed designation | Phase 8 | NEW — ∞ | |
| ATOM-702.186b-002 | 702.186b | ∞: harness acquisition activates static | Phase 8 | (same) | |
| ATOM-702.187b-001 | 702.187b | Mayhem: cast from GY if discarded this turn | Phase 5-Pre T17 + 8 | NEW — Mayhem | |
| ATOM-702.187b-002 | 702.187b | Mayhem: can't cast if not discarded this turn | Phase 8 | (same) | |
| ATOM-702.187b-003 | 702.187b | Mayhem: GY leave+return → new object, not discarded | Phase 8 | (same) | |
| ATOM-702.187c-001 | 702.187c | Mayhem (no cost): play from GY (includes lands) | Phase 8 | (same) | |
| ATOM-702.188a-001 | 702.188a | Web-slinging: alt cost + bounce own tapped creature | Phase 5-Pre T17 + 8 | NEW — Web-slinging | |
| ATOM-702.189a-001 | 702.189a | Firebending N: attack trigger, add N {R}, persists through combat | Phase 7 + 8 | NEW — Firebending | |
| ATOM-702.190a-001 | 702.190a | Sneak: declare-blockers alt cost, bounce unblocked creature | Phase 5-Pre T17 + 8 | NEW — Sneak | SHARED-BEHAVIOR ninjutsu-sneak |
| ATOM-702.190b-001 | 702.190b | Sneak: enters tapped+attacking same target | Phase 8 | (same) | |

**Total ATOM tests: ~155**

---

## 2. BOUNDARY-DEF Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| BOUNDARY-702.114a-001 | 702.114a | Devoid removes color only, not mana cost | Phase 5 | L05 | |

---

## 3. COMP Index

| ID | Composes | Summary | Phase | Ticket |
|----|----------|---------|-------|--------|
| COMP-INFECT-TRAMPLE-001 | ATOM-702.90b + Trample 702.19b | Infect + trample: -1/-1 to blocker, poison overflow to player | Phase 8 | T21c + trample |
| COMP-CASCADE-STORM-001 | ATOM-702.85a + Storm 702.40 | Storm copies don't trigger cascade (copies aren't cast) | Phase 7 + 8 | NEW — Cascade + Storm |
| COMP-BESTOW-UNDYING-001 | ATOM-702.103f + ATOM-702.93a | Bestowed Aura unattaches when undying creature dies+returns | Phase 7 + 8 | — |
| COMP-TOXIC-INFECT-001 | ATOM-702.164a + ATOM-702.90b/c | Infect replaces damage + toxic triggers additionally | Phase 8 | — |
| COMP-DASH-BLITZ-001 | ATOM-702.109a + ATOM-702.152a | Only one alt cost (dash/blitz) can be paid | Phase 8 | T17 |
| COMP-CREW-TYPE-205.1b-001 | ATOM-702.122-003 + Rule 205.1b | Crew on non-artifact → additive type change (enchantment artifact creature) | Phase 5 + 8 | (same as 702.122a) |

---

## 4. META Entries

**META: IMPLEMENTATION NOTE (Menace 702.111)** — Menace is in `KeywordAbility` enum but NOT enforced in `validate_blockers()`. Needs post-assignment check: for each attacker with menace, if blocked, must be blocked by ≥ 2. Phase 4 fix ticket.

**META: Tag LINKED-ABILITY-PATTERN** — Devour count, Exploit target, Tribute paid/not-paid, Evolve all share a pattern: ETB/replacement stores a value, linked triggered ability references it. Needs generic `LinkedAbilityData` mechanism.

**META: Tag TEXT-CHANGING-EFFECT** — Overload (702.96), Splice (702.47), Cleave (702.148) all modify spell text. Shared `TextModification` layer needed. Cross-ref rule 612.

**META: Tag DECLARED-VS-ENTERS-ATTACKING** — Affects Exalted, Melee, "attacks alone" triggers. Engine must track how a creature became attacking (declared vs entered).

**META: Tag SHARED-BEHAVIOR convoke-improvise** — Convoke (702.50) and Improvise (702.126) share "tap permanents to reduce cost." Convoke: GenericOrColor; Improvise: GenericOnly. Share `tap_to_reduce_cost(filter, reduction_type)`.

**META: Tag SHARED-BEHAVIOR persist** — Undying (702.93) and Persist (702.79) are structural duals. Share `dies_return_with_counter(counter_type, absence_check)`.

**META: Tag SHARED-BEHAVIOR crew-saddle** — Crew (702.122) and Saddle (702.171) share "tap creatures with total power ≥ N." Different effects. Share `tap_creatures_for_power(n, filter)`.

**META: Tag SHARED-BEHAVIOR morph-disguise** — Disguise (702.168) and Morph (702.36) share face-down casting. Disguise adds ward {2}.

**META: Tag SHARED-BEHAVIOR ninjutsu-sneak** — Sneak (702.190) and Ninjutsu (702.49) share "swap creature during combat" pattern.

**META: Tag SHARED-BEHAVIOR for-mirrodin-job-select** — For Mirrodin! (702.163) and Job Select (702.182) share "ETB creates token + auto-attach Equipment." Share `create_token_and_attach(token_def, equipment_id)`.

**META: Tag ARCHITECTURE-CONCERN mid-resolution-static-checks** — Ascend requires engine to check static abilities between sequential effects within a resolving spell. Not the same as SBAs.

**META: Tag ARCHITECTURE-CONCERN special-action-mod** — Plot and Foretell use special actions with cost payment + exile. T17 cost modification pipeline needs to extend to special actions.

**META: Tag TRIGGERED-ABILITY-INFRA 603.3c** — "No legal targets → trigger doesn't go on stack" is a general rule. Applies to Mentor, Backup, Fabricate, etc.

**META: Tag ARCHITECTURE-NOTE printed-vs-granted-abilities** — Backup (702.165c) grants only printed abilities. Copy effects (707.2) copy only copiable values (printed). Granted abilities (Layer 6) are not copiable. Read `CardData` for printed, not `compute_characteristics()`.

**META: Tag CROSS-REF rule 711** — Level Up (702.87) uses leveler layout. Session 9B should cover rule 711 level-symbol-to-ability mapping.

**META: Tag COMMANDER-INTERACTION freerunning** — Freerunning condition includes commander combat damage. Relevant Phase 9.

**META: Tag CREW-EVENT** — Engine needs "creature crewed a Vehicle" event for 702.122b triggers.

---

## 5. Classification Summary Table

| Rule | Keyword | Classification | Phase | Notes |
|------|---------|---------------|-------|-------|
| 702.81 | Retrace | DEFERRED | Phase 8 | Zone-cast from GY + discard-land additional cost |
| 702.82 | Devour | DEFERRED | Phase 6 + 8 | ETB replacement, sacrifice creatures, counters |
| 702.83 | Exalted | DEFERRED | Phase 7 + 8 | Triggered on solo attack |
| 702.84 | Unearth | DEFERRED | Phase 8 | GY activated, haste, delayed exile |
| 702.85 | Cascade | DEFERRED | Phase 7 + 8 | **PRIORITY.** Triggered on cast, exile loop, free cast |
| 702.86 | Annihilator | DEFERRED | Phase 7 + 8 | Attack trigger, forced sacrifice |
| 702.87 | Level Up | DEFERRED | Phase 8 | Sorcery-speed activated, level counters |
| 702.88 | Rebound | DEFERRED | Phase 6 + 7 + 8 | Replacement exile + delayed free cast |
| 702.89 | Umbra Armor | DEFERRED | Phase 6 + 8 | Replacement: prevent destroy, destroy Aura |
| 702.90 | Infect | ALREADY-IMPL (partial) | T21c / Phase 5 | 702.90b/c implemented; 702.90d/e deferred |
| 702.91 | Battle Cry | DEFERRED | Phase 7 + 8 | Attack trigger, +1/+0 to others |
| 702.92 | Living Weapon | DEFERRED | Phase 7 + 8 | ETB trigger, Germ token, auto-attach |
| 702.93 | Undying | DEFERRED | Phase 7 + 8 | Dies trigger with counter check |
| 702.94 | Miracle | DEFERRED | Phase 7 + 8 | First-draw reveal + triggered alt-cost |
| 702.95 | Soulbond | DEFERRED | Phase 7 + 8 | Dual ETB triggers, pairing designation |
| 702.96 | Overload | DEFERRED | Phase 5 + 8 | **PRIORITY.** Alt cost + text-changing effect |
| 702.97 | Scavenge | DEFERRED | Phase 8 | GY activated, exile self, counters on target |
| 702.98 | Unleash | DEFERRED | Phase 5 + 6 + 8 | Optional ETB counter + blocking restriction |
| 702.99 | Cipher | DEFERRED | Phase 8 | Exile encoding, combat damage copy |
| 702.100 | Evolve | DEFERRED | Phase 7 + 8 | ETB trigger, P/T comparison, counter |
| 702.101 | Extort | DEFERRED | Phase 7 + 8 | Cast trigger, optional payment, drain |
| 702.102 | Fuse | DEFERRED | Phase 9 | Split card dual-cast |
| 702.103 | Bestow | DEFERRED | Phase 5 + 8 | **PRIORITY.** Alt cost, type change, Aura mode |
| 702.104 | Tribute | DEFERRED | Phase 6 + 7 + 8 | ETB opponent choice + conditional trigger |
| 702.105 | Dethrone | DEFERRED | Phase 7 + 8 | Attack trigger, life comparison |
| 702.106 | Hidden Agenda | OUT-OF-SCOPE | — | Conspiracy format |
| 702.107 | Outlast | DEFERRED | Phase 8 | Sorcery-speed tap activated, counter |
| 702.108 | Prowess | DEFERRED | Phase 7 + 8 | **PRIORITY.** Noncreature cast trigger |
| 702.109 | Dash | DEFERRED | Phase 8 | **PRIORITY.** Alt cost, haste, delayed return |
| 702.110 | Exploit | DEFERRED | Phase 7 + 8 | ETB trigger, optional sacrifice |
| 702.111 | Menace | ALREADY-IMPL | Phase 4 | **PRIORITY.** Blocking restriction (2+ blockers) |
| 702.112 | Renown | DEFERRED | Phase 7 + 8 | Combat damage trigger, renowned designation |
| 702.113 | Awaken | DEFERRED | Phase 5 + 8 | Alt cost, land animation, counters |
| 702.114 | Devoid | DEFERRED | Phase 5 | CDA, colorless override |
| 702.115 | Ingest | DEFERRED | Phase 7 + 8 | Combat damage trigger, exile top card |
| 702.116 | Myriad | DEFERRED | Phase 7 + 9 | Attack trigger, per-opponent token copies |
| 702.117 | Surge | DEFERRED | Phase 8 | Conditional alt cost |
| 702.118 | Skulk | DEFERRED | Phase 8 | Power-based blocking restriction |
| 702.119 | Emerge | DEFERRED | Phase 5-Pre T17 + 8 | Alt cost + sacrifice + MV reduction |
| 702.120 | Escalate | DEFERRED | Phase 8 | Per-mode additional cost |
| 702.121 | Melee | DEFERRED | Phase 7 + 9 | Attack trigger, per-opponent bonus |
| 702.122 | Crew | DEFERRED | Phase 8 | **PRIORITY.** Tap-creatures cost, Vehicle animation |
| 702.123 | Fabricate | DEFERRED | Phase 7 + 8 | ETB trigger, counters-or-tokens |
| 702.124 | Partner | DEFERRED | Phase 9 | Commander deck construction |
| 702.125 | Undaunted | DEFERRED | Phase 5-Pre T17 + 9 | Cost reduction per opponent |
| 702.126 | Improvise | DEFERRED | Phase 5-Pre T17 | Tap artifacts for generic mana |
| 702.127 | Aftermath | DEFERRED | Phase 9 | Split card, GY-only cast, exile on leave |
| 702.128 | Embalm | DEFERRED | Phase 8 | GY activated, modified token copy |
| 702.129 | Eternalize | DEFERRED | Phase 8 | GY activated, 4/4 modified token |
| 702.130 | Afflict | DEFERRED | Phase 7 + 8 | Becomes-blocked trigger, life loss |
| 702.131 | Ascend | DEFERRED | Phase 8 | City's blessing designation |
| 702.132 | Assist | DEFERRED | Phase 8 + 9 | Multiplayer cost sharing |
| 702.133 | Jump-Start | DEFERRED | Phase 8 | GY cast, discard additional cost, exile |
| 702.134 | Mentor | DEFERRED | Phase 7 + 8 | Attack trigger, power-comparison counters |
| 702.135 | Afterlife | DEFERRED | Phase 7 + 8 | Dies trigger, Spirit tokens |
| 702.136 | Riot | DEFERRED | Phase 6 + 8 | ETB choice: counter or haste |
| 702.137 | Spectacle | DEFERRED | Phase 5-Pre T17 + 8 | Conditional alt cost (opponent life loss) |
| 702.138 | Escape | DEFERRED | Phase 5-Pre T17 + 8 | **PRIORITY.** GY cast, exile cards as cost |
| 702.139 | Companion | DEFERRED | Phase 9 | Outside-game, special action |
| 702.140 | Mutate | DEFERRED | Phase 8 | **PRIORITY.** Alt cost, merge, rule 729 |
| 702.141 | Encore | DEFERRED | Phase 7 + 8 + 9 | GY activated, per-opponent tokens |
| 702.142 | Boast | DEFERRED | Phase 8 | Attack-conditioned, once-per-turn |
| 702.143 | Foretell | DEFERRED | Phase 5-Pre T17 + 8 | Special action exile, future alt cost |
| 702.144 | Demonstrate | DEFERRED | Phase 7 + 8 | Cast trigger, mutual copy |
| 702.145 | Daybound/Nightbound | DEFERRED | Phase 9 | DFC + day/night system |
| 702.146 | Disturb | DEFERRED | Phase 9 | DFC cast from GY transformed |
| 702.147 | Decayed | DEFERRED | Phase 5 + 7 + 8 | Can't block + attack → sacrifice |
| 702.148 | Cleave | DEFERRED | Phase 5 + 8 | Alt cost + bracket text removal |
| 702.149 | Training | DEFERRED | Phase 7 + 8 | Co-attacker power trigger, counter |
| 702.150 | Compleated | DEFERRED | Phase 8 | Phyrexian mana loyalty reduction |
| 702.151 | Reconfigure | DEFERRED | Phase 8 | **PRIORITY.** Dual activate, Equipment creature |
| 702.152 | Blitz | DEFERRED | Phase 8 | **PRIORITY.** Alt cost, haste, dies-draw |
| 702.153 | Casualty | DEFERRED | Phase 7 + 8 | **PRIORITY.** Sacrifice additional cost, copy |
| 702.154 | Enlist | DEFERRED | Phase 7 + 8 | Tap-on-attack, power bonus |
| 702.155 | Read Ahead | DEFERRED | Phase 8 | Saga starting chapter choice |
| 702.156 | Ravenous | DEFERRED | Phase 7 + 8 | X-based counters + conditional draw |
| 702.157 | Squad | DEFERRED | Phase 7 + 8 | Repeatable additional cost, token copies |
| 702.158 | Space Sculptor | OUT-OF-SCOPE | — | Un-sets / Unfinity |
| 702.159 | Visit | OUT-OF-SCOPE | — | Un-sets / Unfinity |
| 702.160 | Prototype | DEFERRED | Phase 9 | Alternate characteristics on stack/BF |
| 702.161 | Living Metal | DEFERRED | Phase 5 + 8 | Turn-conditional creature type |
| 702.162 | More Than Meets the Eye | DEFERRED | Phase 9 | DFC casting transformed |
| 702.163 | For Mirrodin! | DEFERRED | Phase 7 + 8 | ETB trigger, Rebel token, auto-attach |
| 702.164 | Toxic | DEFERRED | Phase 7 + 8 | Combat damage trigger, additive poison |
| 702.165 | Backup | DEFERRED | Phase 7 + 8 | **PRIORITY.** ETB counters + ability grant |
| 702.166 | Bargain | DEFERRED | Phase 8 | **PRIORITY.** Optional sacrifice additional cost |
| 702.167 | Craft | DEFERRED | Phase 8 + 9 | Exile self + materials, return transformed |
| 702.168 | Disguise | DEFERRED | Phase 8 | Face-down cast, ward {2} |
| 702.169 | Solved | DEFERRED | Phase 8 | Case condition-based designation |
| 702.170 | Plot | DEFERRED | Phase 8 | **PRIORITY.** Special action exile, future free cast |
| 702.171 | Saddle | DEFERRED | Phase 8 | Tap-creatures cost, saddled designation |
| 702.172 | Spree | DEFERRED | Phase 8 | Cost-selects-mode additional costs |
| 702.173 | Freerunning | DEFERRED | Phase 8 | Conditional alt cost |
| 702.174 | Gift | DEFERRED | Phase 7 + 8 | Two-ability: opponent choice + benefit delivery |
| 702.175 | Offspring | DEFERRED | Phase 7 + 8 | Additional cost, conditional 1/1 token copy |
| 702.176 | Impending | DEFERRED | Phase 5-Pre T17 + 5 + 6 + 7 | 4 abilities: alt cost, counters, not-creature, counter removal |
| 702.177 | Exhaust | DEFERRED | Phase 8 | Activate-only-once restriction |
| 702.178 | Max Speed | DEFERRED | Phase 8 | Speed-conditional static ability |
| 702.179 | Start Your Engines! | DEFERRED | Phase 7 + 8 | SBA speed init + inherent trigger |
| 702.180 | Harmonize | DEFERRED | Phase 5-Pre T17 + 6 + 8 | GY cast, creature-tap cost reduction, exile on leave |
| 702.181 | Mobilize | DEFERRED | Phase 7 + 8 | Attack trigger, Warrior tokens tapped+attacking |
| 702.182 | Job Select | DEFERRED | Phase 7 + 8 | ETB trigger, Hero token, auto-attach |
| 702.183 | Tiered | DEFERRED | Phase 8 | Modal + per-mode additional cost |
| 702.184 | Station | DEFERRED | Phase 8 | Tap creature → charge counters |
| 702.185 | Warp | DEFERRED | Phase 5-Pre T17 + 7 + 8 | Alt cost, delayed exile, future free cast |
| 702.186 | ∞ | DEFERRED | Phase 8 | Harness-conditional static ability |
| 702.187 | Mayhem | DEFERRED | Phase 5-Pre T17 + 8 | GY cast if discarded this turn |
| 702.188 | Web-slinging | DEFERRED | Phase 5-Pre T17 + 8 | Alt cost + bounce tapped creature |
| 702.189 | Firebending | DEFERRED | Phase 7 + 8 | Attack trigger, mana add, combat persistence |
| 702.190 | Sneak | DEFERRED | Phase 5-Pre T17 + 8 | Declare-blockers alt cost, bounce unblocked |

**Totals:** ALREADY-IMPLEMENTED: 2 | DEFERRED: 105 | OUT-OF-SCOPE: 3

---

## 6. NEW Tickets List

| Ticket | Keywords/Rules |
|--------|---------------|
| NEW — Retrace | 702.81 (zone-cast GY + discard-land additional cost) |
| NEW — Devour | 702.82 (ETB replacement, sacrifice, counters, linked ability) |
| NEW — Exalted | 702.83 (triggered on solo attack) |
| NEW — Unearth | 702.84 (GY activated, haste, delayed exile, replacement exile) |
| NEW — Cascade | 702.85 (triggered on cast, exile loop, free cast) |
| NEW — Cascade + split MV | 702.85a (split card MV check) |
| NEW — Cascade multi | 702.85c (multiple instances) |
| NEW — Annihilator | 702.86 (attack trigger, forced sacrifice) |
| NEW — Level Up | 702.87 (sorcery-speed activated, level counters) |
| NEW — Rebound | 702.88 (replacement exile + delayed free cast) |
| NEW — Umbra Armor | 702.89 (replacement: prevent destroy, destroy Aura) |
| NEW — Battle Cry | 702.91 (attack trigger, +1/+0) |
| NEW — Living Weapon | 702.92 (ETB trigger, Germ token, auto-attach) |
| NEW — Undying | 702.93 (dies trigger, counter check) |
| NEW — Miracle | 702.94 (first-draw reveal + triggered alt-cost) |
| NEW — Soulbond | 702.95 (dual ETB triggers, pairing) |
| NEW — Overload | 702.96 (alt cost + text-changing effect) |
| NEW — Scavenge | 702.97 (GY activated, counters on target) |
| NEW — Unleash | 702.98 (optional ETB counter + blocking restriction) |
| NEW — Cipher | 702.99 (exile encoding, combat damage copy) |
| NEW — Evolve | 702.100 (ETB trigger, P/T comparison, counter) |
| NEW — Extort | 702.101 (cast trigger, optional payment, drain) |
| NEW — Fuse | 702.102 (split card dual-cast) |
| NEW — Bestow | 702.103 (alt cost, type change, Aura mode) |
| NEW — Tribute | 702.104 (ETB opponent choice + conditional trigger) |
| NEW — Dethrone | 702.105 (attack trigger, life comparison) |
| NEW — Outlast | 702.107 (sorcery-speed tap activated, counter) |
| NEW — Prowess | 702.108 (noncreature cast trigger) |
| NEW — Dash | 702.109 (alt cost, haste, delayed return) |
| NEW — Exploit | 702.110 (ETB trigger, optional sacrifice) |
| NEW — Renown | 702.112 (combat damage trigger, renowned designation) |
| NEW — Awaken | 702.113 (alt cost, land animation, counters) |
| NEW — Ingest | 702.115 (combat damage trigger, exile top card) |
| NEW — Myriad | 702.116 (attack trigger, per-opponent token copies) |
| NEW — Surge | 702.117 (conditional alt cost) |
| NEW — Skulk | 702.118 (power-based blocking restriction) |
| NEW — Emerge | 702.119 (alt cost + sacrifice + MV reduction) |
| NEW — Escalate | 702.120 (per-mode additional cost) |
| NEW — Melee | 702.121 (attack trigger, per-opponent bonus) |
| NEW — Crew | 702.122 (tap-creatures cost, Vehicle animation) |
| NEW — Fabricate | 702.123 (ETB trigger, counters-or-tokens) |
| NEW — Undaunted | 702.125 (cost reduction per opponent) |
| NEW — Improvise | 702.126 (tap artifacts for generic mana) |
| NEW — Aftermath | 702.127 (GY-only cast, exile on leave) |
| NEW — Embalm | 702.128 (GY activated, modified token copy) |
| NEW — Eternalize | 702.129 (GY activated, 4/4 modified token) |
| NEW — Afflict | 702.130 (becomes-blocked trigger, life loss) |
| NEW — Ascend | 702.131 (city's blessing designation) |
| NEW — Assist | 702.132 (multiplayer cost sharing) |
| NEW — Jump-Start | 702.133 (GY cast, discard additional cost, exile) |
| NEW — Mentor | 702.134 (attack trigger, power-comparison counters) |
| NEW — Afterlife | 702.135 (dies trigger, Spirit tokens) |
| NEW — Riot | 702.136 (ETB choice: counter or haste) |
| NEW — Spectacle | 702.137 (conditional alt cost) |
| NEW — Escape | 702.138 (GY cast, exile cards as cost) |
| NEW — Companion | 702.139 (outside-game, special action) |
| NEW — Mutate | 702.140 (alt cost, merge, rule 729) |
| NEW — Encore | 702.141 (GY activated, per-opponent tokens) |
| NEW — Boast | 702.142 (attack-conditioned, once-per-turn) |
| NEW — Foretell | 702.143 (special action exile, future alt cost) |
| NEW — Demonstrate | 702.144 (cast trigger, mutual copy) |
| NEW — Decayed | 702.147 (can't block + attack sacrifice) |
| NEW — Cleave | 702.148 (alt cost + bracket text removal) |
| NEW — Training | 702.149 (co-attacker power trigger, counter) |
| NEW — Compleated | 702.150 (Phyrexian mana loyalty reduction) |
| NEW — Reconfigure | 702.151 (dual activate, Equipment creature) |
| NEW — Blitz | 702.152 (alt cost, haste, dies-draw, delayed sacrifice) |
| NEW — Casualty | 702.153 (sacrifice additional cost, conditional copy) |
| NEW — Enlist | 702.154 (tap-on-attack, power bonus) |
| NEW — Read Ahead | 702.155 (Saga starting chapter choice) |
| NEW — Ravenous | 702.156 (X-based counters + conditional draw) |
| NEW — Squad | 702.157 (repeatable additional cost, token copies) |
| NEW — Prototype | 702.160 (alternate characteristics on stack/BF) |
| NEW — Living Metal | 702.161 (turn-conditional creature type) |
| NEW — For Mirrodin! | 702.163 (ETB trigger, Rebel token, auto-attach) |
| NEW — Toxic | 702.164 (combat damage trigger, additive poison) |
| NEW — Backup | 702.165 (ETB counters + conditional ability grant) |
| NEW — Bargain | 702.166 (optional sacrifice additional cost) |
| NEW — Craft | 702.167 (exile self + materials, return transformed) |
| NEW — Disguise | 702.168 (face-down cast, ward {2}) |
| NEW — Solved/Case | 702.169 (condition-based designation) |
| NEW — Plot | 702.170 (special action exile, future free cast) |
| NEW — Saddle | 702.171 (tap-creatures cost, saddled designation) |
| NEW — Spree | 702.172 (cost-selects-mode additional costs) |
| NEW — Freerunning | 702.173 (conditional alt cost, Assassin/new-creature) |
| NEW — Gift | 702.174 (opponent choice + benefit delivery) |
| NEW — Offspring | 702.175 (additional cost, 1/1 token copy) |
| NEW — Impending | 702.176 (alt cost, time counters, not-creature, removal) |
| NEW — Exhaust | 702.177 (activate-only-once) |
| NEW — Max Speed | 702.178 (speed-conditional static) |
| NEW — Start Your Engines! + Speed | 702.179 (SBA + inherent trigger) |
| NEW — Harmonize | 702.180 (GY cast, creature-tap reduction, exile) |
| NEW — Mobilize | 702.181 (attack trigger, Warrior tokens) |
| NEW — Job Select | 702.182 (ETB trigger, Hero token, auto-attach) |
| NEW — Tiered | 702.183 (modal + per-mode additional cost) |
| NEW — Station | 702.184 (tap creature → charge counters) |
| NEW — Warp | 702.185 (alt cost, delayed exile, future free cast) |
| NEW — ∞ | 702.186 (harness-conditional static) |
| NEW — Mayhem | 702.187 (discard-conditional GY cast) |
| NEW — Web-slinging | 702.188 (alt cost + bounce tapped creature) |
| NEW — Firebending | 702.189 (attack trigger, mana add, combat persistence) |
| NEW — Sneak | 702.190 (declare-blockers alt cost, bounce unblocked) |
| NEW — Cascade + Storm | COMP (storm copies don't trigger cascade) |

---

## 7. Gap Report

1. **Cost Modification Pipeline (T17) — HIGH.** ~24 keywords depend on T17. Single most critical Phase 8 dependency.
2. **Triggered Ability Infrastructure (Phase 7) — HIGH.** ~40 keywords depend on Phase 7 trigger system.
3. **Zone-Casting Framework (T19) — MEDIUM.** ~11 keywords need casting from non-hand zones.
4. **Face-Down Infrastructure (Rule 708) — MEDIUM.** Disguise, Foretell depend on face-down casting/exile.
5. **Vehicle/Mount Ecosystem — LOW.** Crew, Saddle, Station, Living Metal, Max Speed, Start Your Engines!, Mobilize, Job Select share patterns.
6. **Merging Permanents (Rule 729) — LOW.** Mutate only.
7. **Per-Permanent Designations (D17) — MEDIUM.** Renowned, saddled, exhausted, city's blessing, paired, solved.
8. **DFC System (Phase 9) — Expected.** Daybound/Nightbound, Disturb, More Than Meets the Eye, Prototype, Craft.
9. **Speed System (702.178 + 702.179) — NEW subsystem.** Player-level numeric attribute (0–4), new SBA, inherent trigger.
10. **Harness Infrastructure (Rule 701.64) — NEW subsystem.** ∞ keyword depends on harness action.
11. **Linked Ability Pattern — MEDIUM.** Devour, Tribute, Exploit, Evolve need generic `LinkedAbilityData`.
12. **Text-Changing Effects (Rule 612) — MEDIUM.** Overload, Splice, Cleave need shared `TextModification` layer.
13. **Evasion Framework — Phase 4 fix.** Menace NOT enforced in `validate_blockers()`. Skulk adds power-comparison restriction.
14. **Crew/Saddle Shared Infrastructure — LOW.** Share `tap_creatures_for_power(n, filter)`.

---

## 8. ALREADY-IMPLEMENTED List

- 702.90 Infect (partial — 702.90b/c via T21c; 702.90d/e deferred)
- 702.111 Menace (Phase 4 — needs enforcement fix in `validate_blockers()`)

---

## 9. OUT-OF-SCOPE List

| Rule | Keyword | Reason |
|------|---------|--------|
| 702.106 | Hidden Agenda | Conspiracy format — permanently out of scope |
| 702.158 | Space Sculptor | Un-sets / Unfinity attraction mechanic |
| 702.159 | Visit | Un-sets / Unfinity attraction mechanic |

---

## 10. DEFERRED List

| Rule | Keyword | Target Phase | Reason |
|------|---------|-------------|--------|
| 702.81 | Retrace | Phase 8 | Zone-cast from GY + discard-land additional cost |
| 702.82 | Devour | Phase 6 + 8 | ETB replacement + linked ability |
| 702.83 | Exalted | Phase 7 + 8 | Triggered on solo attack |
| 702.84 | Unearth | Phase 8 | GY activated + delayed triggers + replacement |
| 702.85 | Cascade | Phase 7 + 8 | Triggered on cast, exile loop, free cast |
| 702.86 | Annihilator | Phase 7 + 8 | Attack trigger, forced sacrifice |
| 702.87 | Level Up | Phase 8 | Sorcery-speed activated, level counters + rule 711 |
| 702.88 | Rebound | Phase 6 + 7 + 8 | Replacement exile + delayed free cast |
| 702.89 | Umbra Armor | Phase 6 + 8 | Replacement: prevent destroy, destroy Aura |
| 702.91 | Battle Cry | Phase 7 + 8 | Attack trigger, +1/+0 to other attackers |
| 702.92 | Living Weapon | Phase 7 + 8 | ETB trigger, Germ token, auto-attach |
| 702.93 | Undying | Phase 7 + 8 | Dies trigger with counter check |
| 702.94 | Miracle | Phase 7 + 8 | First-draw reveal + triggered alt-cost |
| 702.95 | Soulbond | Phase 7 + 8 | Dual ETB triggers, pairing |
| 702.96 | Overload | Phase 5 + 8 | Alt cost + text-changing effect |
| 702.97 | Scavenge | Phase 8 | GY activated, exile self, counters |
| 702.98 | Unleash | Phase 5 + 6 + 8 | ETB counter + blocking restriction |
| 702.99 | Cipher | Phase 8 | Exile encoding, combat damage copy |
| 702.100 | Evolve | Phase 7 + 8 | ETB trigger, P/T comparison |
| 702.101 | Extort | Phase 7 + 8 | Cast trigger, optional payment, drain |
| 702.102 | Fuse | Phase 9 | Split card dual-cast |
| 702.103 | Bestow | Phase 5 + 8 | Alt cost, type change, Aura mode |
| 702.104 | Tribute | Phase 6 + 7 + 8 | ETB opponent choice + conditional trigger |
| 702.105 | Dethrone | Phase 7 + 8 | Attack trigger, life comparison |
| 702.107 | Outlast | Phase 8 | Sorcery-speed tap activated, counter |
| 702.108 | Prowess | Phase 7 + 8 | Noncreature cast trigger |
| 702.109 | Dash | Phase 8 | Alt cost, haste, delayed return |
| 702.110 | Exploit | Phase 7 + 8 | ETB trigger, optional sacrifice |
| 702.112 | Renown | Phase 7 + 8 | Combat damage trigger, designation |
| 702.113 | Awaken | Phase 5 + 8 | Alt cost, land animation, counters |
| 702.114 | Devoid | Phase 5 | CDA, colorless override |
| 702.115 | Ingest | Phase 7 + 8 | Combat damage trigger, exile top card |
| 702.116 | Myriad | Phase 7 + 9 | Per-opponent token copies |
| 702.117 | Surge | Phase 8 | Conditional alt cost |
| 702.118 | Skulk | Phase 8 | Power-based blocking restriction |
| 702.119 | Emerge | Phase 5-Pre T17 + 8 | Alt cost + sacrifice + MV reduction |
| 702.120 | Escalate | Phase 8 | Per-mode additional cost |
| 702.121 | Melee | Phase 7 + 9 | Per-opponent-attacked bonus |
| 702.122 | Crew | Phase 8 | Tap-creatures cost, Vehicle animation |
| 702.123 | Fabricate | Phase 7 + 8 | ETB trigger, counters-or-tokens |
| 702.124 | Partner | Phase 9 | Commander deck construction |
| 702.125 | Undaunted | Phase 5-Pre T17 + 9 | Cost reduction per opponent |
| 702.126 | Improvise | Phase 5-Pre T17 | Tap artifacts for generic mana |
| 702.127 | Aftermath | Phase 9 | Split card, GY-only cast, exile |
| 702.128 | Embalm | Phase 8 | GY activated, modified token copy |
| 702.129 | Eternalize | Phase 8 | GY activated, 4/4 token |
| 702.130 | Afflict | Phase 7 + 8 | Becomes-blocked trigger, life loss |
| 702.131 | Ascend | Phase 8 | City's blessing designation |
| 702.132 | Assist | Phase 8 + 9 | Multiplayer cost sharing |
| 702.133 | Jump-Start | Phase 8 | GY cast, discard, exile |
| 702.134 | Mentor | Phase 7 + 8 | Attack trigger, power-comparison |
| 702.135 | Afterlife | Phase 7 + 8 | Dies trigger, Spirit tokens |
| 702.136 | Riot | Phase 6 + 8 | ETB choice: counter or haste |
| 702.137 | Spectacle | Phase 5-Pre T17 + 8 | Conditional alt cost |
| 702.138 | Escape | Phase 5-Pre T17 + 8 | GY cast, exile cards as cost |
| 702.139 | Companion | Phase 9 | Outside-game, special action |
| 702.140 | Mutate | Phase 8 | Alt cost, merge, rule 729 |
| 702.141 | Encore | Phase 7 + 8 + 9 | GY activated, per-opponent tokens |
| 702.142 | Boast | Phase 8 | Attack-conditioned, once-per-turn |
| 702.143 | Foretell | Phase 5-Pre T17 + 8 | Special action exile, future alt cost |
| 702.144 | Demonstrate | Phase 7 + 8 | Cast trigger, mutual copy |
| 702.145 | Daybound/Nightbound | Phase 9 | DFC + day/night system |
| 702.146 | Disturb | Phase 9 | DFC cast from GY |
| 702.147 | Decayed | Phase 5 + 7 + 8 | Can't block + attack sacrifice |
| 702.148 | Cleave | Phase 5 + 8 | Alt cost + bracket text removal |
| 702.149 | Training | Phase 7 + 8 | Co-attacker power trigger |
| 702.150 | Compleated | Phase 8 | Phyrexian mana loyalty reduction |
| 702.151 | Reconfigure | Phase 8 | Dual activate, Equipment creature |
| 702.152 | Blitz | Phase 8 | Alt cost, haste, dies-draw |
| 702.153 | Casualty | Phase 7 + 8 | Sacrifice additional cost, copy |
| 702.154 | Enlist | Phase 7 + 8 | Tap-on-attack, power bonus |
| 702.155 | Read Ahead | Phase 8 | Saga starting chapter choice |
| 702.156 | Ravenous | Phase 7 + 8 | X-based counters + conditional draw |
| 702.157 | Squad | Phase 7 + 8 | Repeatable additional cost, tokens |
| 702.160 | Prototype | Phase 9 | Alternate characteristics |
| 702.161 | Living Metal | Phase 5 + 8 | Turn-conditional creature type |
| 702.162 | More Than Meets the Eye | Phase 9 | DFC casting transformed |
| 702.163 | For Mirrodin! | Phase 7 + 8 | ETB trigger, Rebel token, auto-attach |
| 702.164 | Toxic | Phase 7 + 8 | Combat damage trigger, additive poison |
| 702.165 | Backup | Phase 7 + 8 | ETB counters + conditional ability grant |
| 702.166 | Bargain | Phase 8 | Optional sacrifice additional cost |
| 702.167 | Craft | Phase 8 + 9 | Exile self + materials, return transformed |
| 702.168 | Disguise | Phase 8 | Face-down cast, ward {2} |
| 702.169 | Solved | Phase 8 | Case condition-based designation |
| 702.170 | Plot | Phase 8 | Special action exile, future free cast |
| 702.171 | Saddle | Phase 8 | Tap-creatures cost, saddled designation |
| 702.172 | Spree | Phase 8 | Cost-selects-mode additional costs |
| 702.173 | Freerunning | Phase 8 | Conditional alt cost |
| 702.174 | Gift | Phase 7 + 8 | Opponent choice + benefit delivery |
| 702.175 | Offspring | Phase 7 + 8 | Additional cost, 1/1 token copy |
| 702.176 | Impending | Phase 5-Pre T17 + 5 + 6 + 7 | 4 abilities: alt cost, counters, type, trigger |
| 702.177 | Exhaust | Phase 8 | Activate-only-once |
| 702.178 | Max Speed | Phase 8 | Speed-conditional static |
| 702.179 | Start Your Engines! | Phase 7 + 8 | SBA speed init + inherent trigger |
| 702.180 | Harmonize | Phase 5-Pre T17 + 6 + 8 | GY cast, creature-tap reduction, exile |
| 702.181 | Mobilize | Phase 7 + 8 | Attack trigger, Warrior tokens |
| 702.182 | Job Select | Phase 7 + 8 | ETB trigger, Hero token, auto-attach |
| 702.183 | Tiered | Phase 8 | Modal + per-mode additional cost |
| 702.184 | Station | Phase 8 | Tap creature → charge counters |
| 702.185 | Warp | Phase 5-Pre T17 + 7 + 8 | Alt cost, delayed exile, future free cast |
| 702.186 | ∞ | Phase 8 | Harness-conditional static |
| 702.187 | Mayhem | Phase 5-Pre T17 + 8 | Discard-conditional GY cast |
| 702.188 | Web-slinging | Phase 5-Pre T17 + 8 | Alt cost + bounce tapped creature |
| 702.189 | Firebending | Phase 7 + 8 | Attack trigger, mana add, combat persistence |
| 702.190 | Sneak | Phase 5-Pre T17 + 8 | Declare-blockers alt cost, bounce unblocked |

--- End of Session 8 Summary ---
