# Session 1 Summary: Chapter 1 — Game Concepts (Rules 100–123)

> Generated: 2026-04-02 | Post-audit condensed summary
> 270 atomic test specs | ~57 already implemented | ~40 new tickets | 5 META rules

---

## ATOM Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-100.2a-001 | 100.2a | Constructed deck min 60 cards | Phase 5-Pre | GameConfig | |
| ATOM-100.2a-002 | 100.2a | No more than 4 copies non-basic | Phase 5-Pre | GameConfig | |
| ATOM-100.2a-003 | 100.2a | Basic lands exempt from copy limit | Phase 5-Pre | GameConfig | |
| ATOM-100.2b-001 | 100.2b | Limited deck min 40 cards | Phase 5-Pre | GameConfig | |
| ATOM-100.2b-002 | 100.2b | Limited no copy limit | Phase 5-Pre | GameConfig | |
| ATOM-100.4a-001 | 100.4a | Constructed sideboard max 15 | Phase 5-Pre | GameConfig | |
| ATOM-100.4a-002 | 100.4a | 4-copy limit spans main+sideboard | Phase 5-Pre | NEW | |
| ATOM-101.2a-001 | 101.2a | "Can't" overrides "can" | Per-system | META deferred | |
| ATOM-101.4-001 | 101.4 | Smallpox APNAP simultaneous choices | Phase 7/8 | NEW-CH1-001 | |
| ATOM-101.4c-001 | 101.4c | Simultaneous choices are independent | Phase 7/8 | NEW-CH1-001 | |
| ATOM-103.3-001 | 103.3 | Deck shuffle → libraries at game start | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-103.4-001 | 103.4 | Starting life = 20 | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-103.5-001 | 103.5 | London mulligan draw 7 then bottom N | Phase 5-Pre | T05 | |
| ATOM-103.6-001 | 103.6 | Starting hand size | Phase 5-Pre | T06 | |
| ATOM-103.8a-001 | 103.8a | Starting player skips first draw (2p) | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-103.8c-001 | 103.8c | Starting player skips first draw (Commander) | Phase 9 | Phase 9 | |
| ATOM-104.2a-001 | 104.2a | Last player standing wins | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-104.2b-001 | 104.2b | "You win the game" effect | Phase 8 | NEW-CH1-002 | |
| ATOM-104.3a-001 | 104.3a | Player concedes → leaves game | Phase 8/9 | NEW-CH1-003 | |
| ATOM-104.3b-001 | 104.3b | Life ≤ 0 → lose (SBA) | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-104.3c-001 | 104.3c | Draw from empty library → lose (SBA) | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-104.3c-002 | 104.3c | Multiple draws from empty → single loss | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-104.3d-001 | 104.3d | 10+ poison counters → lose (SBA) | Phase 5-Pre | T16 | |
| ATOM-104.3e-001 | 104.3e | "You lose the game" effect | Phase 8 | NEW-CH1-002 | |
| ATOM-104.3f-001 | 104.3f | Simultaneous win+lose → player loses | Phase 8 | NEW-CH1-004 | |
| ATOM-104.3j-001 | 104.3j | Commander 21+ combat damage → lose | Phase 9 | Phase 9 | |
| ATOM-104.4a-001 | 104.4a | All lose simultaneously → draw | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-104.4b-001 | 104.4b | Mandatory loop → draw | Post-v1 | NEW-CH1-005 | |
| ATOM-104.4c-001 | 104.4c | "The game is a draw" effect | Phase 8 | NEW-CH1-006 | |
| ATOM-105.1-001 | 105.1 | Five colors exist | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-105.2-001 | 105.2 | Colorless is not a color | Phase 5-Pre | T12 | |
| ATOM-105.2-002 | 105.2 | Card color derived from mana cost colors | Phase 5 Layers | L01 | |
| ATOM-105.2-003 | 105.2 | Color indicator overrides mana cost | Phase 5 Layers | L01 | |
| ATOM-105.2a-001 | 105.2a | Multicolor = 2+ colors | Phase 5 Layers | L01 | |
| ATOM-105.2a-002 | 105.2a | Hybrid card is both colors | Phase 5-Pre | T12 | |
| ATOM-105.2b-001 | 105.2b | Colorless mana cost → colorless card | Phase 5 Layers | L01 | |
| ATOM-105.2b-002 | 105.2b | No mana cost + no color indicator → colorless | Phase 5 Layers | L01 | |
| ATOM-105.2c-001 | 105.2c | Devoid CDA makes card colorless | Phase 5 Layers | L18 | |
| ATOM-105.2c-002 | 105.2c | Devoid card retains colored mana cost | Phase 5 Layers | L18 | |
| ATOM-105.3-001 | 105.3 | Color identity (all mana symbols + color indicator) | Phase 9 | Phase 9 | |
| ATOM-105.3-002 | 105.3 | Hybrid mana in cost contributes both colors to identity | Phase 9 | Phase 9 | |
| ATOM-105.4-001 | 105.4 | "Choose a color" validates 5 colors only | Phase 8 | NEW-CH1-007 | |
| ATOM-106.1a-001 | 106.1a | Five mana colors mapped correctly | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-106.1b-001 | 106.1b | Six mana types (5 color + colorless) | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-106.3-001 | 106.3 | Mana pool stores colored + colorless separately | Phase 5-Pre | Architecture | hybrid-mana |
| ATOM-106.4-001 | 106.4 | Mana pool empties at step/phase end | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-106.5-001 | 106.5 | Producing mana of undefined type produces nothing | Phase 8 | NEW-CH1-008 | |
| ATOM-106.5-002 | 106.5 | Producing "one mana of any color" → player chooses | Phase 8 | NEW-CH1-008 | |
| ATOM-106.6-001 | 106.6 | Mana ability produces mana (Forest → {G}) | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-106.6a-001 | 106.6a | Mana replacement restriction propagation | Phase 6 | NEW-CH1-009 | |
| ATOM-106.7-001 | 106.7 | "Could produce" mana query | Phase 8 | NEW-CH1-010 | |
| ATOM-106.8-001 | 106.8 | Hybrid mana production (choose half) | Phase 8 | NEW-CH1-011 | |
| ATOM-106.9-001 | 106.9 | Phyrexian mana production produces colored | Phase 8 | NEW-CH1-012 | |
| ATOM-106.10-001 | 106.10 | Generic mana production (player chooses type) | Phase 8 | NEW-CH1-012 | |
| ATOM-106.11-001 | 106.11 | Snow mana production tags source as snow | Phase 8 | NEW-CH1-012 | |
| ATOM-106.12a-001 | 106.12a | "Tapped for mana" event tracking | Phase 7/8 | DEFERRED | |
| ATOM-107.1a-001 | 107.1a | Integer-only game values | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-107.1b-001 | 107.1b | Negative value clamped to zero for effect | Phase 5/8 | NEW-CH1-013 | |
| ATOM-107.1b-002 | 107.1b | Negative power allowed on creatures | Phase 5 Layers | L04 | |
| ATOM-107.1b-003 | 107.1b | Creature with 0 or less power deals no combat damage | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-107.1b-004 | 107.1b | Doubling effect on negative value → double the negative | Phase 8 | Phase 8 | |
| ATOM-107.3a-001 | 107.3a | X defined by caster during casting | Phase 5-Pre | T18 | |
| ATOM-107.3b-001 | 107.3b | Cast free with undefined X → X=0 | Phase 5-Pre | DEFERRED | |
| ATOM-107.3e-001 | 107.3e | X in triggered ability retains value from source | Phase 7 | DEFERRED | |
| ATOM-107.3g-001 | 107.3g | Mana value with X=0 off-stack | Phase 5 | NEW-CH1-014 | |
| ATOM-107.3h-001 | 107.3h | X=0 for non-stack mana cost payments | Phase 8 | NEW-CH1-015 | |
| ATOM-107.3j-001 | 107.3j | X in mana value on stack uses chosen value | Phase 5 | NEW-CH1-014 | |
| ATOM-107.3m-001 | 107.3m | {X}{X} costs twice the chosen X | Phase 5-Pre | T18 | |
| ATOM-107.3n-001 | 107.3n | Delayed trigger X persistence | Phase 7 | DEFERRED | |
| ATOM-107.4-001 | 107.4 | All mana symbol types representable | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-107.4c-001 | 107.4c | {C} payable only with colorless mana | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-107.4e-001 | 107.4e | Hybrid {W/U} paid with {W} | Phase 5-Pre | NEW-CH1-016 | |
| ATOM-107.4e-002 | 107.4e | Monocolored hybrid {2/B} paid with generic | Phase 5-Pre | NEW-CH1-016 | |
| ATOM-107.4e-003 | 107.4e | Hybrid {W/U} paid with second color {U} | Phase 5-Pre | NEW-CH1-016 | |
| ATOM-107.4f-001 | 107.4f | Phyrexian {R/P} paid with 2 life | Phase 5-Pre | NEW-CH1-017 | |
| ATOM-107.4f-002 | 107.4f | Hybrid Phyrexian {W/U/P} paid with {U} | Phase 5-Pre | NEW-CH1-017 | |
| ATOM-107.4f-003 | 107.4f | Phyrexian {R/P} paid with mana | Phase 5-Pre | NEW-CH1-017 | |
| ATOM-107.4h-001 | 107.4h | Snow {S} paid with snow-sourced mana | Phase 8 | NEW-CH1-018 | |
| ATOM-107.4h-002 | 107.4h | Generic cost reduction doesn't reduce {S} | Phase 5/8 | NEW-CH1-018 | |
| ATOM-107.4h-003 | 107.4h | Generic reduction doesn't reduce pure {S} cost | Phase 8 | NEW-CH1-018 | |
| ATOM-107.5-001 | 107.5 | Already tapped can't pay {T} | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-107.5-002 | 107.5 | Summoning-sick creature can't activate {T} | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-107.6-001 | 107.6 | Already untapped can't pay {Q} | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-107.6-002 | 107.6 | Summoning-sick creature can't activate {Q} | Phase 5-Pre | T10 | |
| ATOM-107.7-001 | 107.7 | [+N] adds N loyalty counters | Phase 5-Pre + Phase 8 | T14 + Phase 8 | |
| ATOM-107.7-002 | 107.7 | [-N] removes N loyalty counters | Phase 5-Pre + Phase 8 | T14 + Phase 8 | |
| ATOM-107.7-003 | 107.7 | [0] loyalty ability costs 0 loyalty | Phase 5-Pre + Phase 8 | T14 + Phase 8 | |
| ATOM-107.14-001 | 107.14 | {E} energy counter payment | Phase 8 | NEW-CH1-019 | |
| ATOM-108.2b-001 | 108.2b | Tokens aren't cards | Phase 5-Pre | T03 | boundary |
| ATOM-108.3-001 | 108.3 | Owner = started in deck | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-108.4-001 | 108.4 | Non-stack/battlefield card has no controller | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-108.4-002 | 108.4 | Exiled card has no controller | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-108.4a-001 | 108.4a | Controller fallback to owner | Phase 8 | NEW-CH1-020 | |
| ATOM-109.1-001 | 109.1 | All 7 object types representable | Phase 8 | NEW-CH1-021 | boundary |
| ATOM-109.2-001 | 109.2 | "Creature" means battlefield permanent | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-109.3-001 | 109.3 | Characteristics vs non-characteristics | Phase 5 Layers | L01 | boundary |
| ATOM-109.4c-001 | 109.4c | Emblem controller tracking | Phase 8 | Phase 8 | |
| ATOM-110.2-001 | 110.2 | Permanent controller = caster | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-110.2a-001 | 110.2a | ETB controller from effect | Phase 8 | NEW-CH1-022 | |
| ATOM-110.2a-002 | 110.2a | ETB under opponent's control | Phase 8 | NEW-CH1-022 | |
| ATOM-110.2b-001 | 110.2b | Gain control of spell → control permanent | Phase 5 Layers | L11 | |
| ATOM-110.4-001 | 110.4 | Instant/sorcery can't enter battlefield | Phase 5-Pre | T21a | boundary |
| ATOM-110.4a-001 | 110.4a | is_permanent() predicate for 6 types | ALREADY-IMPL | ALREADY-IMPLEMENTED | boundary |
| ATOM-110.4b-001 | 110.4b | Permanent spell excludes land | ALREADY-IMPL | ALREADY-IMPLEMENTED | boundary |
| ATOM-110.4c-001 | 110.4c | Typeless permanent stays on battlefield | Phase 5 Layers | L09 | |
| ATOM-110.5-001 | 110.5 | Four permanent status categories | Partial IMPL + D1/D2 | ALREADY-IMPLEMENTED + D1/D2 | boundary |
| ATOM-110.5b-001 | 110.5b | Permanents enter untapped by default | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-110.5b-002 | 110.5b | "Enters tapped" ETB replacement | Phase 6 | Phase 6 | |
| ATOM-110.5c-001 | 110.5c | Status retained even when irrelevant | Phase 6 + Phase 9 | Phase 6 + Phase 9 | |
| ATOM-110.5d-001 | 110.5d | Non-battlefield cards have no status | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-110.5d-002 | 110.5d | Face-down in exile ≠ battlefield face-down status | Phase 5-Pre + Phase 9 | Phase 5-Pre + Phase 9 | |
| ATOM-111.2-001 | 111.2 | Token creator is owner+controller | Phase 8 | Phase 8 | |
| ATOM-111.3-001 | 111.3 | Token characteristics = only what effect specifies | Phase 8 | Phase 8 | |
| ATOM-111.4-001 | 111.4 | Token name defaults to subtype(s) + "Token" | Phase 8 | Phase 8 | |
| ATOM-111.4-002 | 111.4 | Named token uses specified name | Phase 8 | Phase 8 | |
| ATOM-111.5-001 | 111.5 | Token copy of instant/sorcery not created | Phase 8 | Phase 8 | |
| ATOM-111.5-002 | 111.5 | Token not created if ETB prevention active | Phase 8 | Phase 8 | |
| ATOM-111.7-001 | 111.7 | Token in non-battlefield zone ceases to exist (SBA) | Phase 5-Pre | T13 | |
| ATOM-111.8-001 | 111.8 | Token that left battlefield can't change zones | Phase 8 | NEW-CH1-023 | |
| ATOM-111.8-002 | 111.8 | Bounced token ceases to exist (SBA) | Phase 5-Pre | T13 | |
| ATOM-111.10-001 | 111.10 | Predefined Treasure token characteristics | Phase 8 | Phase 8 | boundary |
| ATOM-111.10-002 | 111.10 | Predefined Food token characteristics | Phase 8 | Phase 8 | boundary |
| ATOM-111.11-001 | 111.11 | Non-predefined named token uses Oracle card | Phase 8 | Phase 8 | |
| ATOM-111.12-001 | 111.12 | Copy of nonexistent object → no token | Phase 8 | Phase 8 | |
| ATOM-111.13-001 | 111.13 | Copy of permanent spell becomes token (not "created") | Phase 7 | D19 | |
| ATOM-112.2-001 | 112.2 | Spell controller = caster | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-112.2-002 | 112.2 | Spell copy controller = copier | Phase 7 | D19 | |
| ATOM-112.4-001 | 112.4 | Characteristic changes on spell persist on permanent | Post-v1 | Post-v1 (D16) | |
| ATOM-113.3-001 | 113.3 | Four ability categories in type system | ALREADY-IMPL | ALREADY-IMPLEMENTED | boundary |
| ATOM-113.4-001 | 113.4 | Mana abilities don't use stack | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-113.5-001 | 113.5 | Loyalty ability: sorcery speed, once/turn/permanent | Phase 8 | Phase 8 | |
| ATOM-113.6-001 | 113.6 | Permanent ability functions only on battlefield | Phase 5 Layers | L03 | |
| ATOM-113.6a-001 | 113.6a | CDA functions in all zones | Phase 5 Layers | L18 | |
| ATOM-113.6b-001 | 113.6b | Zone-activated ability (graveyard only) | Phase 5-Pre | T19 | |
| ATOM-113.6j-001 | 113.6j | Ability with graveyard-only cost activates from graveyard | Phase 5-Pre | T19 | |
| ATOM-113.7a-001 | 113.7a | Ability on stack independent of source | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-113.9-001 | 113.9 | Abilities can't be countered by "counter target spell" | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-113.9-002 | 113.9 | CounterActivatedAbility primitive works | ALREADY-IMPL + Phase 7 | ALREADY-IMPL + Phase 7 | |
| ATOM-113.9-003 | 113.9 | CounterTriggeredAbility primitive works | Phase 7 | Phase 7 | |
| ATOM-113.10b-001 | 113.10b | "Loses [ability]" removes all instances | Phase 5 Layers | L06 | |
| ATOM-113.11-001 | 113.11 | "Can't have" ability prevents gaining + removes | Phase 8 | D10 | |
| ATOM-113.12-001 | 113.12 | P/T CDA applied in Layer 7a | Phase 5 Layers | L18 | boundary |
| ATOM-113.12-002 | 113.12 | Color CDA (Devoid) applied in Layer 5 | Phase 5 Layers | L18 | boundary |
| ATOM-114.2-001 | 114.2 | Emblem owned+controlled by receiver, in command zone | Phase 8 | NEW-CH1-024 | |
| ATOM-114.3-001 | 114.3 | Emblem has no types/mana cost/color | Phase 8 | Phase 8 | boundary |
| ATOM-114.5-001 | 114.5 | Emblem is not card/permanent/card type | Phase 8 | Phase 8 | boundary |
| ATOM-115.1b-001 | 115.1b | Aura spell targets; Aura permanent does not | Phase 8 | Phase 8 | |
| ATOM-115.2-001 | 115.2 | Default target zone = battlefield | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-115.3-001 | 115.3 | Different "target" instances can share same target | Phase 5-Pre | T18 | |
| ATOM-115.3-002 | 115.3 | Same "target" instance can't choose same object twice | Phase 5-Pre | T18 | |
| ATOM-115.3/4-001 | 115.3/4 | Two distinct target slots filled by distinct objects | Phase 5-Pre | T18 | |
| ATOM-115.3/4-002 | 115.3/4 | Single object can't fill two target slots (sad path) | Phase 5-Pre | T18 | |
| ATOM-115.4-001 | 115.4 | "Any target" = creature/player/PW/battle | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-115.5-001 | 115.5 | Spell can't target itself | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-115.6-001 | 115.6 | "Up to" allows 0 targets; spell resolves | Phase 5-Pre | T18 | |
| ATOM-115.8-001 | 115.8 | Target-changing effect (Spellskite) | Phase 8 | NEW — Target-changing | |
| ATOM-115.9a-001 | 115.9a | Count targets on a spell | Phase 7 | Phase 7 | |
| ATOM-116.2a-001 | 116.2a | Play land = special action, once/turn | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-116.2a-002 | 116.2a | Can't play second land without effect | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-116.2c-001 | 116.2c | End continuous effect as special action | Phase 8+ | NEW — End effect SA | |
| ATOM-116.2d-001 | 116.2d | Pay to ignore restriction (Leonin Arbiter) | Phase 8 | NEW — Ignore restriction SA | |
| ATOM-116.2g-001 | 116.2g | Companion: pay {3} to put in hand, once/game | Phase 9 | D20 | |
| ATOM-116.2m-001 | 116.2m | Room enchantment unlock special action | Phase 8+ | NEW — Room unlock SA | |
| ATOM-116.3-001 | 116.3 | Priority returned after special action | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.1a-001 | 117.1a | Sorcery castable in main phase, empty stack | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.1a-002 | 117.1a | Sorcery can't be cast with non-empty stack | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.1a-003 | 117.1a | Instant castable any time with priority | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.1b-001 | 117.1b | Activated ability any time with priority | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.1d-001 | 117.1d | Mana ability during spell casting | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.1d-002 | 117.1d | Mana ability during resolution payment (Mana Leak) | Phase 8 | NEW — Resolution mana window | |
| ATOM-117.2a-001 | 117.2a | Triggers placed on stack before priority | Phase 7 | Phase 7 | |
| ATOM-117.2c-001 | 117.2c | TBAs happen before priority at step start | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.3a-001 | 117.3a | No priority during untap step | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.3b-001 | 117.3b | Active player gets priority after spell resolves | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.3c-001 | 117.3c | Caster gets priority after casting | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.3d-001 | 117.3d | Pass priority → next player in turn order | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.4-001 | 117.4 | All pass + stack non-empty → resolve top | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.4-002 | 117.4 | All pass + empty stack → phase/step ends | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-117.5-001 | 117.5 | SBA cascade: token lethal → graveyard → cease-to-exist | Phase 5-Pre | T13 | |
| ATOM-117.7-001 | 117.7 | LIFO stack (response resolves first) | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-118.3-001 | 118.3 | Can't pay cost without resources (life) | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-118.3-002 | 118.3 | Already tapped can't pay {T} | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-118.3a-001 | 118.3a | Paying mana removes from pool | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-118.3b-001 | 118.3b | Paying life subtracts from total | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-118.5-001 | 118.5 | {0} cost not auto-paid | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-118.5a-001 | 118.5a | {0} spell must be cast normally | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-118.6-001 | 118.6 | Unpayable mana cost → cast attempt fails at payment | Phase 5-Pre | T18 | |
| ATOM-118.6a-001 | 118.6a | Alt cost replaces unpayable cost | Phase 8 | Phase 8 | |
| ATOM-118.7-001 | 118.7 | Cost reduced to partial → remaining cost | Phase 5 Layers | T18 | |
| ATOM-118.7-002 | 118.7 | Cost reduced to {0} → free cast | Phase 5 Layers | T18 | |
| ATOM-118.7a-001 | 118.7a | Generic reduction only affects generic component | Phase 5 Layers | T18 | |
| ATOM-118.7b-001 | 118.7b | Colored reduction on missing color → reduce generic | Phase 5 Layers | T18 | |
| ATOM-118.7c-001 | 118.7c | Excess colored reduction overflows to generic | Phase 5 Layers | T18 | |
| ATOM-118.7d-001 | 118.7d | Excess colorless reduction overflows to generic | Phase 5 Layers | T18 | |
| ATOM-118.7e-001 | 118.7e | Hybrid reduction symbol: player chooses half | Phase 8 | NEW-CH1-025 | |
| ATOM-118.7e-002 | 118.7e | Two-brid {2/W} reduction: choose {2} or {W} | Phase 8 | NEW-CH1-025 | |
| ATOM-118.7f-001 | 118.7f | Phyrexian reduction → one colored mana | Phase 8 | NEW-CH1-026 | |
| ATOM-118.7g-001 | 118.7g | Snow reduction → generic | Phase 8 | NEW-CH1-027 | |
| ATOM-118.8d-001 | 118.8d | Additional costs don't change mana value | Phase 5 Layers | L01 | |
| ATOM-118.9a-001 | 118.9a | Only one alt cost per spell | Phase 5-Pre | T18 | |
| ATOM-118.9c-001 | 118.9c | Alt cost doesn't change mana value | Phase 5 Layers | L01 | |
| ATOM-118.9d-001 | 118.9d | Cost modifications apply to alt costs | Phase 5 Layers | T18 | |
| ATOM-118.10-001 | 118.10 | One payment per cost (can't sac same creature twice) | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-118.12-001 | 118.12 | "If you do" fails when cost object gone | Phase 7 | Phase 7 | |
| ATOM-118.12-002 | 118.12 | "If you do" succeeds if action taken despite altered outcome | Phase 8 | Phase 8 | |
| ATOM-118.14-001 | 118.14 | "Mana of any type" allows any mana for any color | Phase 8 | NEW-CH1-028 | |
| ATOM-118.14-002 | 118.14 | "Mana of any type" doesn't override spending restrictions | Phase 8 | Phase 8 | |
| ATOM-119.1-001 | 119.1 | Starting life from GameConfig | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-119.2-001 | 119.2 | Damage → life loss | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-119.3-001 | 119.3 | Gain life adjusts total upward | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-119.4-001 | 119.4 | Can't pay life > current total | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-119.4b-001 | 119.4b | Paying 0 life always legal | Phase 8 | NEW-CH1-029 | |
| ATOM-119.5-001 | 119.5 | SetLife → gain/lose difference | Phase 8 | NEW-CH1-030 | |
| ATOM-119.5-002 | 119.5 | SetLife higher → life gain | Phase 8 | NEW-CH1-030 | |
| ATOM-119.6-001 | 119.6 | 0 life → SBA loss | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-119.7-001 | 119.7 | "Can't gain life" blocks exchange | Phase 8 | NEW-CH1-031 | |
| ATOM-119.7-002 | 119.7 | "Can't gain life" blocks redistribution | Phase 8 | Phase 8 | |
| ATOM-119.7-003 | 119.7 | "Can't gain life" blocks alt cost requiring opponent life gain | Phase 8 | Phase 8 | |
| ATOM-119.7-004 | 119.7 | "Can't gain life" prevents replacement that would produce gain | Phase 6 + Phase 8 | Phase 6 + Phase 8 | |
| ATOM-119.8-001 | 119.8 | "Can't lose life" blocks exchange | Phase 8 | NEW-CH1-032 | |
| ATOM-119.9-001 | 119.9 | Each lifelink source triggers separately | Phase 7 | Phase 7 | |
| ATOM-119.9-002 | 119.9 | Gaining 0 life doesn't trigger | Phase 7 | Phase 7 | |
| ATOM-119.10-001 | 119.10 | Life gain replacement doesn't apply to 0 gain | Phase 6 | Phase 6 | |
| ATOM-120.1a-001 | 120.1a | Damage can't target noncreature/non-PW/non-battle | ALREADY-IMPL | ALREADY-IMPLEMENTED | boundary |
| ATOM-120.2a-001 | 120.2a | Combat damage = creature's power | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-120.3a-001 | 120.3a | Non-infect damage → life loss | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-120.3b-001 | 120.3b | Infect damage → poison counters | Phase 8 | Phase 8 | |
| ATOM-120.3c-001 | 120.3c | Damage to PW removes loyalty | Phase 5-Pre + Phase 8 | T14 + Phase 8 | |
| ATOM-120.3d-001 | 120.3d | Wither/infect damage to creature → -1/-1 counters | Phase 8 | Phase 8 | |
| ATOM-120.3e-001 | 120.3e | Normal damage to creature → marked damage | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-120.3f-001 | 120.3f | Lifelink → controller gains life | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-120.3g-001 | 120.3g | Toxic combat damage → additional poison | Phase 8 | Phase 8 | |
| ATOM-120.3h-001 | 120.3h | Damage to battle removes defense counters | Phase 9 | Phase 9 | |
| ATOM-120.4a-001 | 120.4a | Trample excess damage to player | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-120.4a-002 | 120.4a | Deathtouch makes 1 damage lethal for trample | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-120.5-001 | 120.5 | Damage doesn't destroy; SBA does | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-120.6-001 | 120.6 | Damage removed at cleanup | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-120.8-001 | 120.8 | 0 damage = no-op | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-120.10-001 | 120.10 | Excess damage = amount beyond lethal | Phase 7 | Phase 7 | |
| ATOM-121.1-001 | 121.1 | Draw = top of library → hand | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-121.2-001 | 121.2 | N draws = N individual draws | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-121.2a-001 | 121.2a | Draw replacement modifies count before draws | Phase 6 | Phase 6 | |
| ATOM-121.2b-001 | 121.2b | "Can't draw more than one/turn" partially carries out | Phase 8 | NEW-CH1-033 | |
| ATOM-121.2c-001 | 121.2c | APNAP draw ordering (active first) | Phase 8 | NEW-CH1-034 | |
| ATOM-121.3-001 | 121.3 | Optional draw from empty library permitted | Phase 8 | Phase 8 | |
| ATOM-121.3-002 | 121.3 | "Can't draw" prevents optional draw choice | Phase 8 | Phase 8 | |
| ATOM-121.4-001 | 121.4 | Empty library draw → SBA loss | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-121.5-001 | 121.5 | Library-to-hand without "draw" is not a draw | Phase 7 | Phase 7 | |
| ATOM-121.6a-001 | 121.6a | Draw replacement applies even with empty library | Phase 6 | Phase 6 | |
| ATOM-122.1-001 | 122.1 | Counters are markers, not objects | Phase 5-Pre | T14 | boundary |
| ATOM-122.1a-001 | 122.1a | +1/+1 counter adds to P/T | Phase 5 Layers | L04/L08 | |
| ATOM-122.1a-002 | 122.1a | -1/-1 counter subtracts from P/T | Phase 5 Layers | L04/L08 | |
| ATOM-122.1a-003 | 122.1a | Non-standard P/T counters don't annihilate | Phase 5 Layers | L04/L08 | |
| ATOM-122.1b-001 | 122.1b | Keyword counter grants keyword | Phase 5 Layers | L06 | |
| ATOM-122.1c-001 | 122.1c | Shield counter prevents destruction | Phase 6 | Phase 6 | |
| ATOM-122.1c-002 | 122.1c | Shield counter prevents damage | Phase 6 | Phase 6 | |
| ATOM-122.1d-001 | 122.1d | Stun counter prevents untap | Phase 6 | Phase 6 | |
| ATOM-122.1e-001 | 122.1e | PW with 0 loyalty → SBA graveyard | Phase 5-Pre | T14 + T16 | |
| ATOM-122.1f-001 | 122.1f | 10+ poison → SBA loss | Phase 5-Pre | T16 | |
| ATOM-122.1g-001 | 122.1g | Battle with 0 defense → SBA graveyard | Phase 9 | Phase 9 | |
| ATOM-122.1h-001 | 122.1h | Finality counter → exile instead of graveyard | Phase 6 | Phase 6 | |
| ATOM-122.1i-001 | 122.1i | Rad counter mill + life loss trigger | Phase 8 | Phase 8 | |
| ATOM-122.2-001 | 122.2 | Counters removed on zone change | Phase 5-Pre | T14 | |
| ATOM-122.3-001 | 122.3 | +1/+1 and -1/-1 annihilate (SBA) | Phase 5-Pre | T16 | |
| ATOM-122.4-001 | 122.4 | Counter cap SBA removes excess | Phase 8 | Phase 8 | |
| ATOM-122.5-001 | 122.5 | Move counter: remove from A, put on B | Phase 8 | Phase 8 | |
| ATOM-122.5-002 | 122.5 | Can't move counter to same object | Phase 8 | Phase 8 | |
| ATOM-122.5-003 | 122.5 | Can't move counter source doesn't have | Phase 8 | Phase 8 | |
| ATOM-122.5-004 | 122.5 | Can't move counter if dest can't receive | Phase 8 | Phase 8 | |
| ATOM-122.5-005 | 122.5 | Can't move counter if object left zone | Phase 8 | Phase 8 | |
| ATOM-122.6a-001 | 122.6a | Controller places ETB counters | Phase 6 | Phase 6 | |
| ATOM-122.7-001 | 122.7 | "Nth counter" trigger fires on threshold | Phase 7 | Phase 7 | |
| ATOM-122.8-001 | 122.8 | Triggered ability creates new counters (not move) | Phase 7 | Phase 7 | |
| ATOM-122.9-001 | 122.9 | Activated ability creates new counters (not move) | Phase 7 | Phase 7 | |

---

## BOUNDARY-DEF Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-108.2b-001 | 108.2b | Tokens aren't cards | Phase 5-Pre | T03 | |
| ATOM-109.1-001 | 109.1 | All 7 object types representable | Phase 8 | NEW-CH1-021 | |
| ATOM-109.3-001 | 109.3 | Characteristics vs non-characteristics | Phase 5 Layers | L01 | |
| ATOM-110.4-001 | 110.4 | Instant/sorcery can't enter battlefield | Phase 5-Pre | T21a | |
| ATOM-110.4a-001 | 110.4a | is_permanent() predicate for 6 types | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-110.4b-001 | 110.4b | Permanent spell excludes land | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-110.5-001 | 110.5 | Four permanent status categories | Partial IMPL + D1/D2 | ALREADY-IMPLEMENTED + D1/D2 | |
| ATOM-111.10-001 | 111.10 | Predefined Treasure token | Phase 8 | Phase 8 | |
| ATOM-111.10-002 | 111.10 | Predefined Food token | Phase 8 | Phase 8 | |
| ATOM-113.3-001 | 113.3 | Four ability categories | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-113.12-001 | 113.12 | P/T CDA applied in Layer 7a | Phase 5 Layers | L18 | |
| ATOM-113.12-002 | 113.12 | Color CDA (Devoid) in Layer 5 | Phase 5 Layers | L18 | |
| ATOM-114.3-001 | 114.3 | Emblem has no types/mana cost/color | Phase 8 | Phase 8 | |
| ATOM-114.5-001 | 114.5 | Emblem not card/permanent/card type | Phase 8 | Phase 8 | |
| ATOM-120.1a-001 | 120.1a | Damage can't target noncreature/non-PW/non-battle | ALREADY-IMPL | ALREADY-IMPLEMENTED | |
| ATOM-122.1-001 | 122.1 | Counters are markers, not objects | Phase 5-Pre | T14 | |

---

## COMP Index

No COMP (composition) tests were generated in Session 1. All tests are atomic or boundary-definition.

---

## META Entries

### META-101.1: Card Text Overrides Rules
- **Principle:** If a card's text contradicts the rules, the card text takes precedence.
- **Concrete tests deferred to:** Per-card/per-keyword sessions (702.x, 604, 609, 613, 614)

### META-101.2: "Can't" Overrides "Can"
- **Principle:** If one effect says something can happen and another says it can't, the "can't" effect wins.
- **Concrete tests deferred to:** Per-system sessions (combat 508/509, targeting 115, life 119, damage 120, lands 305, casting 601, activation 602)

### META-101.3: Impossible Instructions Ignored
- **Principle:** If an instruction is impossible to perform, it is simply ignored.
- **Concrete tests deferred to:** Per-system sessions (608 resolution, 701 keyword actions)

### META-107.2: Mana Symbol Ordering
- **Principle:** 107.2 defines the canonical ordering of mana symbols. PURE-DEF naming rule; no behavioral consequence.

### META-113.6b: Zone-Activated Ability Pattern
- **Principle:** 113.6b describes a pattern (some keyword abilities specify activation from specific zones) rather than a specific testable mechanic. Actual tests belong with each keyword: Cycling from hand, Unearth from graveyard, Scavenge from graveyard, Channel from hand, etc.

---

## Classification Summary Table

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

---

## NEW Tickets List

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

## Gap Report / Phase Dependency Heatmap

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

## ALREADY-IMPLEMENTED List

103.3, 103.4, 103.8a, 104.2a, 104.3b, 104.3c, 104.4a, 105.1, 106.1a, 106.1b, 106.4, 107.1a, 107.4, 107.4c, 107.5, 107.6, 108.3, 108.4, 109.2, 110.2, 110.4a, 110.4b, 110.5b, 110.5d, 112.2, 113.3, 113.4, 113.7a, 113.9, 115.2, 115.4, 115.5, 116.2a, 116.3, 117.1a, 117.1b, 117.1d, 117.2c, 117.3a, 117.3b, 117.3c, 117.3d, 117.4, 117.7, 118.3, 118.3a, 118.3b, 118.5, 118.5a, 118.10, 119.1, 119.2, 119.3, 119.4, 119.6, 120.1a, 120.2a, 120.3a, 120.3e, 120.3f, 120.4a, 120.5, 120.6, 120.8, 121.1, 121.2, 121.4

---

## OUT-OF-SCOPE List

| Rule(s) | Reason |
|---------|--------|
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
| 109.4d–g | Variant controllers (Planechase/Archenemy/Conspiracy) |
| 116.2i | Planechase planar die roll |
| 116.2j | Conspiracy Draft face-up flip |
| 117.6 | Shared team turns priority |
| 119.4a | 2HG life payment |
| 121.2d | Shared team turns draw order |
| 123.1–123.9 | Stickers (Un-set) |

---

## DEFERRED List

| Rule(s) | Target Phase | Reason |
|---------|-------------|--------|
| 100.2c | Phase 9 | Commander deckbuilding |
| 100.3 | Phase 8 | Coins/dice mechanics (705/706) |
| 100.4b | Match mgmt | Limited deck validation / sideboard swap |
| 103.2b | Phase 9 (D20) | Companion reveal |
| 103.2c | Phase 9 | Commander setup |
| 103.4a–e | Phase 9 | Variant life totals (2HG, Vanguard, Commander, Brawl, Archenemy) |
| 103.5a–d | Phase 9 | Mulligan variants |
| 103.5c | Phase 9 | Multiplayer first mulligan free |
| 104.6 | Phase 9+ | Karn Liberated restart |
| 106.12a | Phase 7/8 | "Tapped for mana" event tracking |
| 106.12b | Phase 6 | "Tapped for mana" replacement effects |
| 106.13 | Phase 8 | Drain Power |
| 107.3b | Phase 5-Pre | Cast free with undefined X → X=0 |
| 107.3d | Phase 9 | X in special action costs (suspend, morph) |
| 107.3e | Phase 7 | X in triggered ability retains value |
| 107.3n | Phase 7 | Delayed trigger X persistence |
| 107.8–107.8b | Phase 8 | Level Up cards |
| 107.15a-b | Phase 7/8 | Saga rules |
| 107.16–107.16a | Phase 8+ | Class cards |
| 107.18 | Phase 8 | Pawprint symbol |
| 108.3b | Phase 8 | Cards from outside the game (Wish) |
| 108.5 | Phase 8 | Dungeon cards (partially) |
| 109.4c | Phase 8 | Emblem controller |
| 113.6n | Phase 9 | Deck construction abilities |
| 113.6p | Phase 9 | Command zone abilities (emblem/plane/vanguard/scheme) |
| 115.7a–f | Phase 8+ | Target changing/choosing details |
| 115.8 | Phase 8 | Target-changing effects (Deflection, Spellskite) |
| 115.9a | Phase 7 | Count targets on spells |
| 115.9b–c | Phase 7/8 | "Targets only" nuances |
| 116.2b | Phase 9 | Morph face-up |
| 116.2c | Phase 8+ | End continuous effect special action (Licid) |
| 116.2d | Phase 8 | Pay to ignore restriction (Leonin Arbiter) |
| 116.2e | Phase 8 | Circling Vultures |
| 116.2f | Phase 9 | Suspend |
| 116.2h | Phase 9 | Foretell |
| 116.2k | Phase 9 | Plot |
| 116.2m | Phase 8+ | Room enchantments (unlock second room) |
| 119.1a–e | Phase 9 | Variant starting life totals |
| 122.1i | Phase 8 | Rad counters |
| 122.8 | Phase 7 | Triggered ability counter transfer |
| 122.9 | Phase 7 | Activated ability counter transfer |

---

*End of Session 1 Summary — Chapter 1: Game Concepts (Rules 100–123)*
