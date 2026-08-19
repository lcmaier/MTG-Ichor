# Phase 5-Pre — Test Index

> 280 entries extracted from session summaries

---

## Phase 5-Pre

**280 entries**

| ID | Rule | Summary | Ticket | Session | Tags |
|----|------|---------|--------|---------|------|
| ATOM-100.2a-001 | 100.2a | Constructed deck min 60 cards | GameConfig | S1 |  |
| ATOM-100.2a-002 | 100.2a | No more than 4 copies non-basic | GameConfig | S1 |  |
| ATOM-100.2a-003 | 100.2a | Basic lands exempt from copy limit | GameConfig | S1 |  |
| ATOM-100.2b-001 | 100.2b | Limited deck min 40 cards | GameConfig | S1 |  |
| ATOM-100.2b-002 | 100.2b | Limited no copy limit | GameConfig | S1 |  |
| ATOM-100.4a-001 | 100.4a | Constructed sideboard max 15 | GameConfig | S1 |  |
| ATOM-100.4a-002 | 100.4a | 4-copy limit spans main+sideboard | NEW | S1 |  |
| ATOM-103.5-001 | 103.5 | London mulligan draw 7 then bottom N | T05 | S1 |  |
| ATOM-103.6-001 | 103.6 | Starting hand size | T06 | S1 |  |
| ATOM-104.3d-001 | 104.3d | 10+ poison counters → lose (SBA) | T16 | S1 |  |
| ATOM-105.2-001 | 105.2 | Colorless is not a color | T12 | S1 |  |
| ATOM-105.2a-002 | 105.2a | Hybrid card is both colors | T12 | S1 |  |
| ATOM-106.3-001 | 106.3 | Mana pool stores colored + colorless separately | Architecture | S1 | hybrid-mana |
| ATOM-107.1b-001 | 107.1b | Negative value clamped to zero for effect | NEW-CH1-013 | S1 |  |
| ATOM-107.3a-001 | 107.3a | X defined by caster during casting | T18 | S1 |  |
| ATOM-107.3b-001 | 107.3b | Cast free with undefined X → X=0 | DEFERRED | S1 |  |
| ATOM-107.3g-001 | 107.3g | Mana value with X=0 off-stack | NEW-CH1-014 | S1 |  |
| ATOM-107.3j-001 | 107.3j | X in mana value on stack uses chosen value | NEW-CH1-014 | S1 |  |
| ATOM-107.3m-001 | 107.3m | {X}{X} costs twice the chosen X | T18 | S1 |  |
| ATOM-107.4e-001 | 107.4e | Hybrid {W/U} paid with {W} | NEW-CH1-016 | S1 |  |
| ATOM-107.4e-002 | 107.4e | Monocolored hybrid {2/B} paid with generic | NEW-CH1-016 | S1 |  |
| ATOM-107.4e-003 | 107.4e | Hybrid {W/U} paid with second color {U} | NEW-CH1-016 | S1 |  |
| ATOM-107.4f-001 | 107.4f | Phyrexian {R/P} paid with 2 life | NEW-CH1-017 | S1 |  |
| ATOM-107.4f-002 | 107.4f | Hybrid Phyrexian {W/U/P} paid with {U} | NEW-CH1-017 | S1 |  |
| ATOM-107.4f-003 | 107.4f | Phyrexian {R/P} paid with mana | NEW-CH1-017 | S1 |  |
| ATOM-107.4h-002 | 107.4h | Generic cost reduction doesn't reduce {S} | NEW-CH1-018 | S1 |  |
| ATOM-107.6-002 | 107.6 | Summoning-sick creature can't activate {Q} | T10 | S1 |  |
| ATOM-108.2b-001 | 108.2b | Tokens aren't cards | T03 | S1 | boundary |
| ATOM-110.4-001 | 110.4 | Instant/sorcery can't enter battlefield | T21a | S1 | boundary |
| ATOM-111.7-001 | 111.7 | Token in non-battlefield zone ceases to exist (SBA) | T13 | S1 |  |
| ATOM-111.8-002 | 111.8 | Bounced token ceases to exist (SBA) | T13 | S1 |  |
| ATOM-113.6b-001 | 113.6b | Zone-activated ability (graveyard only) | T19 | S1 |  |
| ATOM-113.6j-001 | 113.6j | Ability with graveyard-only cost activates from graveyard | T19 | S1 |  |
| ATOM-115.3-001 | 115.3 | Different "target" instances can share same target | T18 | S1 |  |
| ATOM-115.3-002 | 115.3 | Same "target" instance can't choose same object twice | T18 | S1 |  |
| ATOM-115.3/4-001 | 115.3/4 | Two distinct target slots filled by distinct objects | T18 | S1 |  |
| ATOM-115.3/4-002 | 115.3/4 | Single object can't fill two target slots (sad path) | T18 | S1 |  |
| ATOM-115.6-001 | 115.6 | "Up to" allows 0 targets; spell resolves | T18 | S1 |  |
| ATOM-117.5-001 | 117.5 | SBA cascade: token lethal → graveyard → cease-to-exist | T13 | S1 |  |
| ATOM-118.6-001 | 118.6 | Unpayable mana cost → cast attempt fails at payment | T18 | S1 |  |
| ATOM-118.9a-001 | 118.9a | Only one alt cost per spell | T18 | S1 |  |
| ATOM-122.1-001 | 122.1 | Counters are markers, not objects | T14 | S1 | boundary |
| ATOM-122.1e-001 | 122.1e | PW with 0 loyalty → SBA graveyard | T14 + T16 | S1 |  |
| ATOM-122.1f-001 | 122.1f | 10+ poison → SBA loss | T16 | S1 |  |
| ATOM-122.2-001 | 122.2 | Counters removed on zone change | T14 | S1 |  |
| ATOM-122.3-001 | 122.3 | +1/+1 and -1/-1 annihilate (SBA) | T16 | S1 |  |
| ATOM-201.2a-001 | 201.2a | Same-name comparison (Bile Blight pattern) | T14, NEW-S2-01 | S2 |  |
| ATOM-202.1b-001 | 202.1b | No mana cost = unpayable, cast rejected | T18 | S2 |  |
| ATOM-205.4d-001 | 205.4d | Legendary supertype → legend rule SBA | T14 | S2 |  |
| ATOM-205.4e-001 | 205.4e | Legendary instant/sorcery can't cast w/o legendary creature/PW | T18 | S2 |  |
| ATOM-205.4e-002 | 205.4e | Legendary sorcery CAN cast with legendary creature | T18 | S2 |  |
| ATOM-209.1-001 | 209.1 | PW enters with loyalty counters = printed loyalty | T14 | S2 |  |
| ATOM-209.2-002 | 209.2 | Loyalty ability once-per-turn restriction | T19 | S2 |  |
| ATOM-300.1-001 | 300.1 | CardType enum has exactly 15 types | T07 | S3 | boundary, enum |
| ATOM-301.5-001 | 301.5 | Equipment attaches to creature (legal) | T04, T15 | S3 | equipment, attachment |
| ATOM-301.5-002 | 301.5 | Equipment can't attach to non-creature | T15 | S3 | equipment, attachment |
| ATOM-301.5-001/002 | 301.5 | Equipment: legal (creature) and illegal (non-creature) attachment | T04, T15 | S3 | equipment, boundary |
| ATOM-301.5b-001 | 301.5b | Equipment ETB unattached | T04 | S3 | equipment, etb |
| ATOM-301.5c-002 | 301.5c | Equipment loses subtype → SBA unattaches | T15 | S3 | equipment, sba |
| ATOM-301.5c-004 | 301.5c | Equipment on destroyed creature → unattached, stays on BF (SBA) | T15, T04 | S3 | equipment, sba |
| COMP-301.5c+303.4c-001 | ATOM-301.5c-004, ATOM-303.4c-002 | Creature destroyed: Equipment stays on BF unattached, Aura to GY | T04, T15 | S3 |  |
| ATOM-303.4-001 | 303.4 | Aura ETB attached to target creature | T15b | S3 | aura, etb |
| ATOM-303.4a-001 | 303.4a | Aura spell requires target; no legal target → can't cast | T15b | S3 | aura, targeting |
| ATOM-303.4c-001 | 303.4c | Aura on illegal object (type removed) → graveyard SBA | T15 | S3 | aura, sba |
| ATOM-303.4c-002 | 303.4c | Aura host destroyed → graveyard SBA | T15, T04 | S3 | aura, sba |
| ATOM-303.4d-001 | 303.4d | Self-enchanting Aura → graveyard SBA | T15 | S3 | aura, sba |
| ATOM-303.4e-003 | 303.4e | Pacifism cast on opponent's creature: caster = Aura controller | T15b | S3 | aura, control, positive |
| ATOM-303.4f-001 | 303.4f | Non-stack Aura ETB: controller chooses (hexproof OK) | T15b | S3 | aura, etb, hexproof |
| ATOM-304.4-001 | 304.4 | Instant can't enter battlefield → stays in previous zone | T21a | S3 | instant, zone-guard |
| ATOM-304.5-001 | 304.5 | "As an instant" = priority only; no instant card needed | T19 | S3 | timing, instant-speed |
| ATOM-305.2-001 | 305.2 | Default 1 land/turn; second rejected | T22 | S3 | land, limit |
| ATOM-306.5-001 | 306.5 | Loyalty is characteristic only planeswalkers have | T14 | S3 | planeswalker, boundary |
| ATOM-306.5b-001 | 306.5b | PW ETB with loyalty counters = printed loyalty | T14 | S3 | planeswalker, etb, counters |
| ATOM-306.5c-001 | 306.5c | BF PW loyalty = loyalty counter count | T14 | S3 | planeswalker, loyalty, counters |
| ATOM-306.5d-001 | 306.5d | Loyalty ability activated at sorcery speed | T19 | S3 | planeswalker, activation, timing |
| ATOM-306.5d-002 | 306.5d | Only one loyalty ability per PW per turn | T19 | S3 | planeswalker, activation, once |
| ATOM-306.5d-003 | 306.5d | Loyalty ability rejected on opponent's turn / non-empty stack | T19 | S3 | planeswalker, activation, timing |
| COMP-306.5b+306.8+306.9-001 | ATOM-306.5b-001, ATOM-306.8-001, ATOM-306.9-001 | PW enters with loyalty, takes lethal damage, SBA kills it | T14, T21c | S3 |  |
| ATOM-306.8-001 | 306.8 | Damage to PW removes loyalty counters | T21c | S3 | planeswalker, damage |
| ATOM-306.8-002 | 306.8 | Excess damage to PW absorbed (no overflow) | T21c | S3 | planeswalker, damage, boundary |
| ATOM-306.9-001 | 306.9 | PW with 0 loyalty → graveyard SBA | T14 | S3 | planeswalker, sba |
| ATOM-306.9-002 | 306.9 | PW with >0 loyalty stays (positive) | T14 | S3 | planeswalker, sba, positive |
| ATOM-307.4-001 | 307.4 | Sorcery can't enter battlefield → stays in previous zone | T21a | S3 | sorcery, zone-guard |
| ATOM-307.5-001 | 307.5 | "As a sorcery" = priority + main + stack empty; no sorcery needed | T19 | S3 | timing, sorcery-speed |
| ATOM-307.5-002 | 307.5 | "As a sorcery" rejected on opponent's turn | T19 | S3 | timing, sorcery-speed |
| COMP-ZONE-TRANSITION-001 | 400.3 + 400.7 | Owner routing + new object identity on destroy | NEW | S4 | ATOM-400.3-001, ATOM-400.7-001 |
| ATOM-400.4a-001 | 400.4a | Instant/sorcery can't enter battlefield | T21a | S4 |  |
| ATOM-400.7-003 | 400.7 | Multiple simultaneous trackers see stale epoch | Same as 400.7-001 | S4 |  |
| ATOM-400.7a-001 | 400.7a | Spell-to-permanent color-change effect continuity | NEW — stack-to-permanent effect continuity | S4 |  |
| ATOM-400.7a-002 | 400.7a | Text-changing effect on spell persists to permanent | NEW — text-changing effect persistence | S4 |  |
| ATOM-400.7b-001 | 400.7b | Static ability grants continue to permanent | L06 | S4 |  |
| ATOM-400.7d-001 | 400.7d | CastInfo carried to permanent (kicker check) | T21a | S4 |  |
| ATOM-405.4-004 | 405.4 | Ability controller = activator (not owner) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-500.4-001 | 500.4 | "Until" duration expires as step/phase begins | T22 | S4 |  |
| ATOM-500.5-001 | 500.5 | End-of-step effect expiry + mana pool emptying | ALREADY-IMPL (mana) + T22 | S4 |  |
| ATOM-500.5a-001 | 500.5a | "Until end of combat" expires at end of combat phase | T22 | S4 |  |
| ATOM-502.3-002 | 502.3 | "Doesn't untap" continuous effect prevents untap | NEW — doesn't-untap effect filtering | S4 |  |
| ATOM-505.6b-004 | 505.6b | Additional land plays (Exploration) | T22 | S4 |  |
| ATOM-506.4-001 | 506.4 | Removed from combat on leaving battlefield | T21b | S4 |  |
| ATOM-506.4-002 | 506.4 | Removed from combat on controller change | T21b | S4 |  |
| ATOM-506.4-003 | 506.4 | Removed from combat on stops-being-creature | T21b | S4 |  |
| ATOM-508.1c-002 | 508.1c | "Can't attack alone" aggregate constraint | T21d | S4 |  |
| ATOM-508.1d-001 | 508.1d | Requirement maximization (attacks-if-able) | T21d | S4 |  |
| ATOM-508.1d-002 | 508.1d | Cost-gated requirement is optional | T21d | S4 |  |
| ATOM-508.1k-001 | 508.1k | Creatures become attacking (mid-declaration control change) | ALREADY-IMPL (basic) + NEW | S4 |  |
| ATOM-509.1b-002 | 509.1b | Cumulative evasion (flying + shadow) | T21b | S4 |  |
| ATOM-509.1c-001 | 509.1c | Blocking requirement maximization | T21d | S4 |  |
| ATOM-509.1c-002 | 509.1c | Menace + "must block" aggregate interaction | T21d + T21b | S4 |  |
| ATOM-511.2-002 | 511.2 | "Until end of combat" expires at phase end | T22 | S4 |  |
| ATOM-513.2-003 | 513.2 | "Until end of turn" still expires this turn (not carried over) | T22 | S4 |  |
| ATOM-514.2-001 | 514.2 | Damage removed + "until end of turn" effects end simultaneously | ALREADY-IMPL (damage) + T22 | S4 |  |
| ATOM-514.3a-001 | 514.3a | Cleanup re-loop: SBAs/triggers → priority → new cleanup | T16 | S4 |  |
| ATOM-514.3a-002 | 514.3a | Re-looped cleanup runs full TBAs again | T16 | S4 |  |
| ATOM-601.2-001 | 601.2 | Rollback on failed casting step | T18 | S5 |  |
| ATOM-601.2b-001 | 601.2b | Mode choice stored on StackEntry | T18 | S5 |  |
| ATOM-601.2b-002 | 601.2b | Alt/add cost choice stored on StackEntry | T17, T18 | S5 |  |
| ATOM-601.2b-003 | 601.2b | Only one alternative cost allowed per cast | T18 | S5 |  |
| ATOM-601.2b-004 | 601.2b | X value announced and stored on StackEntry | T18 | S5 |  |
| ATOM-601.2b-007 | 601.2b | X choice for additional-cost-only X | T18 | S5 |  |
| ATOM-601.2c-002 | 601.2c | Conditional targets based on kicker/mode | T18 | S5 |  |
| ATOM-601.2c-003 | 601.2c | Per-instance target uniqueness | T18 | S5 |  |
| ATOM-601.2c-004 | 601.2c | Different target instances can share a target | T18 | S5 |  |
| ATOM-601.2c-006 | 601.2c | Conditional targets absent when add cost not paid | T18 | S5 |  |
| ATOM-601.2c-007 | 601.2c | Kicker changes target legality criteria | T18 | S5 |  |
| ATOM-601.2d-001 | 601.2d | Damage/counter distribution at cast time | T18 | S5 |  |
| ATOM-601.2d-002 | 601.2d | Zero-allocation rejected | T18 | S5 |  |
| ATOM-601.2e-001 | 601.2e | Post-proposal legality check with rollback | T18 | S5 |  |
| ATOM-601.2f-003 | 601.2f | Cost lock-in prevents later modifications | T18 | S5 |  |
| ATOM-601.2g-001 | 601.2g | Mana ability activation window during casting | T18 | S5 |  |
| ATOM-601.2h-001 | 601.2h | Cost payment ordering | T18 | S5 |  |
| ATOM-601.2h-003 | 601.2h | Player chooses order of cost components via DP | T18 | S5 |  |
| ATOM-601.4-001 | 601.4 | Intra-step look-ahead for mode/cost choices | T18 | S5 |  |
| ATOM-601.5-001 | 601.5 | Post-proposal re-check | T18 | S5 |  |
| ATOM-601.5-002 | 601.5 | Cost-phase illegality does NOT cause rewind | T18 | S5 |  |
| ATOM-602.2-002 | 602.2 | Activation rollback on failure | T18 | S5 |  |
| ATOM-602.2a-003 | 602.2a | Ability object characteristic profile distinct from spell | T19 | S5 |  |
| ATOM-602.2b-001 | 602.2b | Activation follows casting pipeline steps | T18, T19 | S5 |  |
| ATOM-602.5b-001 | 602.5b | Once-per-turn restriction persists across controller change | T19 | S5 |  |
| ATOM-602.5d-001 | 602.5d | Sorcery-speed activation restriction | T19 | S5 |  |
| ATOM-602.5d-002 | 602.5d | Sorcery-speed requires empty stack | T19 | S5 |  |
| ATOM-602.5e-001 | 602.5e | Instant-speed activation restriction | T19 | S5 |  |
| COMP-602+605-001 | 602.5a, 605.1a | Summoning sick: tap blocked, sacrifice allowed | IMPL | S5 |  |
| ATOM-604.5-001 | 604.5 | Stack-zone static abilities | T17, T18 | S5 |  |
| ATOM-604.5-002 | 604.5 | Stack-zone static for alternative cost | T17, T18 | S5 |  |
| ATOM-604.6-001 | 604.6 | Hand-zone static abilities for cast permissions | T18 | S5 |  |
| ATOM-604.6-002 | 604.6 | Hand-zone static restricts casting timing | T18 | S5 |  |
| ATOM-605.1a-002 | 605.1a | Target disqualifies mana ability classification | NEW-2 | S5 |  |
| ATOM-605.1a-004 | 605.1a | Target disqualifies even with mana production | NEW-2 | S5 |  |
| ATOM-607.1-001 | 607.1 | Linked ability scoping — reads only first ability's data | T20 | S5 |  |
| ATOM-607.1c-001 | 607.1c | Self-linked ability (Tyrant's Choice) | T20 | S5 |  |
| ATOM-607.2a-001 | 607.2a | Exile-reference linking (O-Ring pattern) | T20 | S5 |  |
| ATOM-607.2a-002 | 607.2a | Per-ability exile tracking (two independent linked pairs) | T20 | S5 |  |
| ATOM-607.2c-001 | 607.2c | ETB-creation linking | T20 | S5 |  |
| ATOM-607.2d-001 | 607.2d | Choice-value linking | T20 | S5 |  |
| ATOM-607.2d-002 | 607.2d | Choice-value persistence through zone change (Cavern) | T20 | S5 |  |
| ATOM-607.2h-001 | 607.2h | Same-paragraph static+triggered linking | T20 | S5 |  |
| ATOM-607.2i-001 | 607.2i | Kicker-style additional cost linking | T17, T20 | S5 |  |
| ATOM-607.2i-002 | 607.2i | Per-kicker-cost linking (Stormscape Battlemage) | T17, T20 | S5 |  |
| ATOM-607.2j-001 | 607.2j | Variable cost value linking | T17, T18, T20 | S5 |  |
| ATOM-607.2q-001 | 607.2q | Cast-cost-exile linking | T17, T20 | S5 |  |
| ATOM-608.2b-002 | 608.2b | Partial-target resolution (Plague Spores) | T18 | S5 |  |
| ATOM-608.2b-005 | 608.2b | Partial-target resolution (Jagged Lightning) | T18 | S5 |  |
| ATOM-608.2d-002 | 608.2d | Resolution-time untargeted distribution | T18 | S5 |  |
| ATOM-608.2d-003 | 608.2d | Flexible distribution with minimum-per-object | T18 | S5 |  |
| ATOM-608.2i-001 | 608.2i | Historical look-back exception to 608.2h | T18 | S5 |  |
| ATOM-608.3b-001 | 608.3b | Targeted permanent fizzle or bestow fallback | T15b | S5 |  |
| ATOM-608.3c-001 | 608.3c | Aura ETB attachment | T15b | S5 |  |
| ATOM-611.2c-003 | 611.2c | Mixed effect: char-mod locks in, rule-mod dynamic | NEW-611.2c-mix | S6 |  |
| COMP-613-LAYERS-FULL-STACK-001 | 613.1a–g | Full 7-layer stack on single permanent | L04–L12 | S6 | ATOM-613.1a-001 through ATOM-613.1g-001 |
| ATOM-613.7a-001 | 613.7a | Static ability uses later of object vs granting effect timestamp | NEW-613.7a | S6 |  |
| ATOM-613.7c-001 | 613.7c | Counter timestamp updates when new counter of same kind added | NEW-613.7c | S6 |  |
| ATOM-613.7e-001 | 613.7e | Equipment re-timestamp on attach | NEW-613.7e | S6 |  |
| ATOM-700.2a-001 | 700.2a | Modal mode choice at cast — illegal modes excluded | T18 | S7a |  |
| ATOM-700.2c-001 | 700.2c | Mode-conditional targeting — unchosen modes need no targets | T18 | S7a |  |
| ATOM-700.2d-001 | 700.2d | Mode uniqueness enforcement — can't choose same mode twice | T18 | S7a |  |
| ATOM-700.2e-001 | 700.2e | Opponent chooses mode when specified | T18 | S7a |  |
| ATOM-700.2h-001 | 700.2h | Per-mode additional costs aggregated in casting | T18 | S7a |  |
| ATOM-701.3a-001 | 701.3a | Attach Equipment to creature — basic | T15 | S7a |  |
| ATOM-701.3a-002 | 701.3a | Attach to invalid target — rejected | T15 | S7a |  |
| ATOM-701.3b-001 | 701.3b | Failed attach — no movement | T15 | S7a |  |
| ATOM-701.3b-002 | 701.3b | Reattach to same target — no-op | T15 | S7a |  |
| ATOM-701.3b-003 | 701.3b | Non-Aura/Equipment attach — does nothing | T15 | S7a |  |
| ATOM-701.3c-001 | 701.3c | Reattach to different target — new timestamp | T15 | S7a | dependency, layers |
| ATOM-701.3d-001 | 701.3d | Unattach Equipment — stays on battlefield | T15b | S7a |  |
| ATOM-701.3d-002 | 701.3d | Creature leaves → Equipment becomes unattached | T15b | S7a |  |
| COMP-7A-005 | 701.3d + 701.21a | Equipment unattach on creature sacrifice |  | S7a |  |
| ATOM-701.21a-001 | 701.21a | Sacrifice — bypasses destroy/indestructible | T15 | S7a |  |
| ATOM-701.21a-002 | 701.21a | Can't sacrifice what you don't control | T15 | S7a |  |
| ATOM-701.21a-003 | 701.21a | Can't sacrifice non-permanents | T15 | S7a |  |
| COMP-7A-001 | 701.21a + 701.8b | Sacrifice indestructible bypasses destroy replacement |  | S7a |  |
| ATOM-701.40a-001 | 701.40a | Manifest — face-down 2/2 creature | NEW | S7a |  |
| ATOM-701.40b-001 | 701.40b | Turn manifested creature face up — pay mana cost | NEW | S7a |  |
| ATOM-701.40b-002 | 701.40b | Non-creature can't turn face up via manifest | NEW | S7a |  |
| ATOM-701.40g-001 | 701.40g | Instant/sorcery can't turn face up — stays face-down | NEW | S7a |  |
| ATOM-701.43a-001 | 701.43a | Exert — skip next untap step | NEW | S7a |  |
| ATOM-701.43b-001 | 701.43b | Exert stacking — both expire same untap step | NEW | S7a |  |
| ATOM-701.58a-001 | 701.58a | Cloak — face-down 2/2 with ward {2} | NEW | S7a |  |
| ATOM-701.62a-001 | 701.62a | Manifest dread — look at 2, manifest 1, GY other | NEW | S7a |  |
| ATOM-702.2e-001 | 702.2e | Deathtouch LKI after zone change | T20b | S7b | deathtouch, LKI |
| ATOM-702.4c-001 | 702.4c | Remove double strike mid-combat stops 2nd step | DEFERRED | S7b | double-strike, mid-combat, continuous-effects |
| ATOM-702.4d-001 | 702.4d | Grant double strike to first-striker after 1st step | DEFERRED | S7b | double-strike, first-strike, mid-combat, continuous-effects |
| ATOM-702.5a-001 | 702.5a | Aura targeting restricted by enchant ability | T15b | S7b | enchant, aura, targeting |
| ATOM-702.5d-001 | 702.5d | Enchant player Aura can't target permanents | T15b | S7b | enchant, aura, enchant-player |
| ATOM-702.6a-001 | 702.6a | Equip activation, attachment, sorcery-speed | T15b | S7b | equip, attachment, sorcery-speed |
| ATOM-702.6a-002 | 702.6a | Equip targets only own creatures | T15b | S7b | equip, targeting, controller |
| ATOM-702.6a-003 | 702.6a | Equip sorcery-speed enforcement (negative) | T15b | S7b | equip, sorcery-speed, negative-case |
| ATOM-702.7c-001 | 702.7c | Gain first strike after 1st step doesn't block 2nd | DEFERRED | S7b | first-strike, mid-combat, continuous-effects |
| ATOM-702.7c-002 | 702.7c | Remove first strike after 1st step doesn't grant 2nd | DEFERRED | S7b | first-strike, mid-combat, continuous-effects |
| ATOM-702.8a-001 | 702.8a | Flash bypasses sorcery-speed timing | T18 | S7b | flash, timing, instant-speed |
| ATOM-702.11b-001 | 702.11b | Hexproof blocks opponent targeting | T22 | S7b | hexproof, targeting |
| ATOM-702.11b-002 | 702.11b | Hexproof allows self-targeting | T22 | S7b | hexproof, self-targeting |
| COMP-702-007 | 702.11b + 608.2b | Hexproof granted mid-stack → fizzle | Phase 5-Pre | S7b | T22 |
| ATOM-702.12b-001 | 702.12b | Indestructible prevents lethal damage destruction | T09 | S7b | indestructible, lethal-damage, SBA |
| ATOM-702.12b-002 | 702.12b | Indestructible prevents destroy effects | T09 | S7b | indestructible, destroy-effect |
| ATOM-702.15c-001 | 702.15c | Lifelink LKI after zone change | T20b | S7b | lifelink, LKI |
| ATOM-702.16a-001 | 702.16a | Protection ability exists and is queryable | T22 | S7b | protection, ability-query |
| ATOM-702.16b-001 | 702.16b | Protection targeting restriction (matching) | T22 | S7b | protection, targeting |
| ATOM-702.16b-002 | 702.16b | Protection doesn't block non-matching quality | T22 | S7b | protection, targeting, non-matching |
| ATOM-702.16c-001 | 702.16c | Protection causes illegal Auras to fall off (SBA) | T22 | S7b | protection, aura, SBA |
| ATOM-702.16d-001 | 702.16d | Protection causes illegal Equipment to detach (SBA) | T22 | S7b | protection, equipment, SBA |
| ATOM-702.16e-001 | 702.16e | Protection prevents all damage from matching source | T22 | S7b | protection, damage-prevention, combat |
| ATOM-702.16f-001 | 702.16f | Protection evasion: can't be blocked by matching | T22 | S7b | protection, blocking, evasion |
| COMP-702-005 | 702.16e + 702.19b | Protection + trample: damage prevented but trample still overflows | Phase 5-Pre | S7b | T22 |
| ATOM-702.18a-001 | 702.18a | Shroud blocks all targeting including controller | T22 | S7b | shroud, targeting |
| ATOM-702.18a-003 | 702.18a | Shroud blocks opponent targeting | T22 | S7b | shroud, targeting, opponent |
| ATOM-702.90d-001 | 702.90d | Infect: LKI determines infect on zone-changed source | L18 | S8 |  |
| ATOM-702.95e-002 | 702.95e | Soulbond: type loss → unpaired | (same) | S8 |  |
| ATOM-702.95e-003 | 702.95e | Soulbond: controller change → unpaired | (same) | S8 |  |
| ATOM-702.96a-001 | 702.96a | Overload: alt cost, "target" → "each" text change | NEW — Overload | S8 | TEXT-CHANGING-EFFECT |
| ATOM-702.96b-001 | 702.96b | Overloaded spell has no targets, hits untargetable | (same) | S8 |  |
| ATOM-702.98a-001 | 702.98a | Unleash: optional ETB counter + can't block with counter | NEW — Unleash | S8 |  |
| ATOM-702.98a-002 | 702.98a | Unleash without counter CAN block | (same) | S8 |  |
| ATOM-702.98a-003 | 702.98a | Unleash: can't block with +1/+1 from any source | (same) | S8 |  |
| ATOM-702.103a-001 | 702.103a | Bestow: cast creature as Aura for alt cost | NEW — Bestow | S8 |  |
| ATOM-702.103d-001 | 702.103d | Bestow: castability uses Aura characteristics | (same) | S8 |  |
| ATOM-702.113a-001 | 702.113a | Awaken: alt cost, land → 0/0 Elemental + N counters | NEW — Awaken | S8 |  |
| ATOM-702.114a-001 | 702.114a | Devoid: CDA makes object colorless everywhere | L05 | S8 |  |
| ATOM-702.114a-002 | 702.114a | Devoid: colorless despite colored mana symbols | (same) | S8 |  |
| BOUNDARY-702.114a-001 | 702.114a | Devoid removes color only, not mana cost | L05 | S8 |  |
| ATOM-702.119a-001 | 702.119a | Emerge: alt cost + sacrifice creature + MV reduction | T17 + NEW — Emerge | S8 |  |
| COMP-CREW-TYPE-205.1b-001 | ATOM-702.122-003 + Rule 205.1b | Crew on non-artifact → additive type change (enchantment artifact creature) | (same as 702.122a) | S8 |  |
| ATOM-702.125a-001 | 702.125a | Undaunted: cost reduced by {1} per opponent | T17 + NEW — Undaunted | S8 |  |
| ATOM-702.125c-001 | 702.125c | Multiple undaunted instances each reduce separately | (same) | S8 |  |
| ATOM-702.126a-001 | 702.126a | Improvise: tap artifacts for generic mana | T17 + NEW — Improvise | S8 | SHARED-BEHAVIOR convoke-improvise |
| ATOM-702.126a-002 | 702.126a | Improvise: only pays generic, not colored | (same) | S8 |  |
| ATOM-702.137a-001 | 702.137a | Spectacle: alt cost if opponent lost life | T17 + NEW — Spectacle | S8 |  |
| ATOM-702.138a-001 | 702.138a | Escape: cast from GY, exile cards as cost | T17 + NEW — Escape | S8 |  |
| ATOM-702.143a-001 | 702.143a | Foretell: pay {2} exile face-down, cast later for alt cost | T17 + NEW — Foretell | S8 |  |
| ATOM-702.147a-001 | 702.147a | Decayed: can't block + attack → sacrifice at EOC | NEW — Decayed | S8 |  |
| ATOM-702.148a-001 | 702.148a | Cleave: alt cost removes bracketed text | NEW — Cleave | S8 | TEXT-CHANGING-EFFECT |
| ATOM-702.151b-001 | 702.151b | Reconfigure: not a creature while attached | (same) | S8 |  |
| ATOM-702.161a-001 | 702.161a | Living metal: creature during your turn only | NEW — Living Metal | S8 |  |
| ATOM-702.176a-001 | 702.176a | Impending: alt cost → enters with N time counters | NEW — Impending | S8 |  |
| ATOM-702.176a-002 | 702.176a | Impending: not a creature while has time counters | (same) | S8 |  |
| ATOM-702.180a-001 | 702.180a | Harmonize: GY cast, tap creature for cost reduction | NEW — Harmonize | S8 |  |
| ATOM-702.185a-001 | 702.185a | Warp: alt cost from hand | NEW — Warp | S8 |  |
| ATOM-702.187b-001 | 702.187b | Mayhem: cast from GY if discarded this turn | NEW — Mayhem | S8 |  |
| ATOM-702.188a-001 | 702.188a | Web-slinging: alt cost + bounce own tapped creature | NEW — Web-slinging | S8 |  |
| ATOM-702.190a-001 | 702.190a | Sneak: declare-blockers alt cost, bounce unblocked creature | NEW — Sneak | S8 | SHARED-BEHAVIOR ninjutsu-sneak |
| ATOM-703.4c-002 | 703.4c | Winter Orb selective untap restriction | NEW — Untap restriction effects | S9a | continuous-effects |
| ATOM-703.4p-001 | 703.4p | Cleanup damage removal + EOT effects end simultaneously | ALREADY-IMPLEMENTED; T22 | S9a |  |
| ATOM-704.3-002 | 704.3 | Cleanup step SBA shortcut — no priority if no SBAs | T16 | S9a |  |
| ATOM-704.3-003 | 704.3 | Cleanup SBA re-loop with priority | T16 | S9a |  |
| ATOM-704.4-001 | 704.4 | SBAs not checked mid-resolution | L04 | S9a | layers |
| ATOM-704.5c-001 | 704.5c | 10+ poison counters → player loses | T16 | S9a |  |
| ATOM-704.5c-002 | 704.5c | 9 poison counters → no loss (negative case) | T16 | S9a |  |
| ATOM-704.5d-001 | 704.5d | Token in non-battlefield zone ceases to exist | T13 | S9a |  |
| ATOM-704.5i-001 | 704.5i | Planeswalker 0 loyalty → graveyard | T14 | S9a |  |
| ATOM-704.5i-002 | 704.5i | Planeswalker loyalty > 0 stays (negative case) | T14 | S9a |  |
| ATOM-704.5j-001 | 704.5j | Legend rule — same name, same controller → choose one | T14 | S9a |  |
| ATOM-704.5j-002 | 704.5j | Legend rule — different names, no SBA (negative) | T14 | S9a |  |
| ATOM-704.5j-003 | 704.5j | Legend rule — same name, different controllers, no SBA (negative) | T14 | S9a |  |
| ATOM-704.5m-001 | 704.5m | Unattached Aura → graveyard | T15 | S9a |  |
| ATOM-704.5m-002 | 704.5m | Aura host left → graveyard | T15 | S9a |  |
| ATOM-704.5n-001 | 704.5n | Equipment on non-creature → unattach, stays on battlefield | T15 | S9a |  |
| ATOM-704.5p-001 | 704.5p | Creature illegally attached → unattach | T15 | S9a |  |
| ATOM-704.5q-001 | 704.5q | +1/+1 and -1/-1 counter annihilation (unequal) | T13 | S9a |  |
| ATOM-704.5q-002 | 704.5q | Counter annihilation (equal counts) | T13 | S9a |  |
| COMP-9A-001 | 704.5q + 704.5f + 704.8 | SBA cascade: counter annihilation + lethal damage + LKI |  | S9a |  |
| ATOM-704.8-001 | 704.8 | Pre-SBA LKI snapshot for undying eligibility | L18 | S9a | lki |
