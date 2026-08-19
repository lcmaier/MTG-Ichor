# Global Atomic Test Index

> Extracted from 12 session summaries
> Total entries: 1780 (after dedup)

---

## Phase Counts

| Phase | ATOMs | BOUNDARYs | COMPs | Total |
|-------|-------|-----------|-------|-------|
| ALREADY-IMPL | 195 | 0 | 7 | 202 |
| Phase 5-Pre | 268 | 1 | 11 | 280 |
| Phase 5-Layers | 165 | 0 | 13 | 178 |
| Phase 6 | 123 | 7 | 8 | 138 |
| Phase 7 | 194 | 2 | 6 | 202 |
| Phase 8 | 531 | 3 | 13 | 547 |
| Phase 9 | 210 | 5 | 11 | 226 |
| Post-v1 | 3 | 0 | 0 | 3 |
| Cross-cutting | 1 | 0 | 0 | 1 |
| DEFERRED | 3 | 0 | 0 | 3 |

---

## ALREADY-IMPL

**202 entries**

| ID | Rule | Summary | Ticket | Session | Tags |
|----|------|---------|--------|---------|------|
| ATOM-103.3-001 | 103.3 | Deck shuffle → libraries at game start | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-103.4-001 | 103.4 | Starting life = 20 | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-103.8a-001 | 103.8a | Starting player skips first draw (2p) | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-104.2a-001 | 104.2a | Last player standing wins | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-104.3b-001 | 104.3b | Life ≤ 0 → lose (SBA) | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-104.3c-001 | 104.3c | Draw from empty library → lose (SBA) | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-104.3c-002 | 104.3c | Multiple draws from empty → single loss | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-104.4a-001 | 104.4a | All lose simultaneously → draw | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-105.1-001 | 105.1 | Five colors exist | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-106.1a-001 | 106.1a | Five mana colors mapped correctly | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-106.1b-001 | 106.1b | Six mana types (5 color + colorless) | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-106.4-001 | 106.4 | Mana pool empties at step/phase end | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-106.6-001 | 106.6 | Mana ability produces mana (Forest → {G}) | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-107.1a-001 | 107.1a | Integer-only game values | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-107.1b-003 | 107.1b | Creature with 0 or less power deals no combat damage | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-107.4-001 | 107.4 | All mana symbol types representable | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-107.4c-001 | 107.4c | {C} payable only with colorless mana | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-107.5-001 | 107.5 | Already tapped can't pay {T} | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-107.5-002 | 107.5 | Summoning-sick creature can't activate {T} | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-107.6-001 | 107.6 | Already untapped can't pay {Q} | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-108.3-001 | 108.3 | Owner = started in deck | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-108.4-001 | 108.4 | Non-stack/battlefield card has no controller | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-108.4-002 | 108.4 | Exiled card has no controller | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-109.2-001 | 109.2 | "Creature" means battlefield permanent | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-110.2-001 | 110.2 | Permanent controller = caster | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-110.4a-001 | 110.4a | is_permanent() predicate for 6 types | ALREADY-IMPLEMENTED | S1 | boundary |
| ATOM-110.4b-001 | 110.4b | Permanent spell excludes land | ALREADY-IMPLEMENTED | S1 | boundary |
| ATOM-110.5-001 | 110.5 | Four permanent status categories | ALREADY-IMPLEMENTED + D1/D2 | S1 | boundary |
| ATOM-110.5b-001 | 110.5b | Permanents enter untapped by default | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-110.5d-001 | 110.5d | Non-battlefield cards have no status | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-112.2-001 | 112.2 | Spell controller = caster | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-113.3-001 | 113.3 | Four ability categories in type system | ALREADY-IMPLEMENTED | S1 | boundary |
| ATOM-113.4-001 | 113.4 | Mana abilities don't use stack | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-113.7a-001 | 113.7a | Ability on stack independent of source | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-113.9-001 | 113.9 | Abilities can't be countered by "counter target spell" | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-113.9-002 | 113.9 | CounterActivatedAbility primitive works | ALREADY-IMPL + Phase 7 | S1 |  |
| ATOM-115.2-001 | 115.2 | Default target zone = battlefield | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-115.4-001 | 115.4 | "Any target" = creature/player/PW/battle | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-115.5-001 | 115.5 | Spell can't target itself | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-116.2a-001 | 116.2a | Play land = special action, once/turn | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-116.2a-002 | 116.2a | Can't play second land without effect | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-116.3-001 | 116.3 | Priority returned after special action | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.1a-001 | 117.1a | Sorcery castable in main phase, empty stack | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.1a-002 | 117.1a | Sorcery can't be cast with non-empty stack | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.1a-003 | 117.1a | Instant castable any time with priority | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.1b-001 | 117.1b | Activated ability any time with priority | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.1d-001 | 117.1d | Mana ability during spell casting | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.2c-001 | 117.2c | TBAs happen before priority at step start | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.3a-001 | 117.3a | No priority during untap step | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.3b-001 | 117.3b | Active player gets priority after spell resolves | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.3c-001 | 117.3c | Caster gets priority after casting | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.3d-001 | 117.3d | Pass priority → next player in turn order | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.4-001 | 117.4 | All pass + stack non-empty → resolve top | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.4-002 | 117.4 | All pass + empty stack → phase/step ends | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-117.7-001 | 117.7 | LIFO stack (response resolves first) | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-118.3-001 | 118.3 | Can't pay cost without resources (life) | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-118.3-002 | 118.3 | Already tapped can't pay {T} | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-118.3a-001 | 118.3a | Paying mana removes from pool | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-118.3b-001 | 118.3b | Paying life subtracts from total | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-118.5-001 | 118.5 | {0} cost not auto-paid | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-118.5a-001 | 118.5a | {0} spell must be cast normally | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-118.10-001 | 118.10 | One payment per cost (can't sac same creature twice) | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-119.1-001 | 119.1 | Starting life from GameConfig | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-119.2-001 | 119.2 | Damage → life loss | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-119.3-001 | 119.3 | Gain life adjusts total upward | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-119.4-001 | 119.4 | Can't pay life > current total | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-119.6-001 | 119.6 | 0 life → SBA loss | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-120.1a-001 | 120.1a | Damage can't target noncreature/non-PW/non-battle | ALREADY-IMPLEMENTED | S1 | boundary |
| ATOM-120.2a-001 | 120.2a | Combat damage = creature's power | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-120.3a-001 | 120.3a | Non-infect damage → life loss | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-120.3e-001 | 120.3e | Normal damage to creature → marked damage | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-120.3f-001 | 120.3f | Lifelink → controller gains life | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-120.4a-001 | 120.4a | Trample excess damage to player | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-120.4a-002 | 120.4a | Deathtouch makes 1 damage lethal for trample | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-120.5-001 | 120.5 | Damage doesn't destroy; SBA does | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-120.6-001 | 120.6 | Damage removed at cleanup | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-120.8-001 | 120.8 | 0 damage = no-op | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-121.1-001 | 121.1 | Draw = top of library → hand | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-121.2-001 | 121.2 | N draws = N individual draws | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-121.4-001 | 121.4 | Empty library draw → SBA loss | ALREADY-IMPLEMENTED | S1 |  |
| ATOM-202.2a-001 | 202.2a | Five colors enum completeness | ALREADY-IMPLEMENTED | S2 | boundary |
| ATOM-205.2a-001 | 205.2a | CardType enum completeness | ALREADY-IMPLEMENTED | S2 | boundary |
| ATOM-205.3g-001 | 205.3g | Artifact types set membership | ALREADY-IMPLEMENTED | S2 | boundary |
| ATOM-205.3h-001 | 205.3h | Enchantment types set membership | ALREADY-IMPLEMENTED | S2 | boundary |
| ATOM-205.3i-001 | 205.3i | Basic land types set membership | ALREADY-IMPLEMENTED | S2 | boundary |
| ATOM-205.3m-001 | 205.3m | Creature types set membership | ALREADY-IMPLEMENTED | S2 | boundary |
| ATOM-205.4a-001 | 205.4a | Supertype enum completeness | ALREADY-IMPLEMENTED | S2 | boundary |
| ATOM-303.4g-002 | 303.4g | Aura from stack, target dies → fizzle to graveyard | ALREADY-IMPL | S3 | aura, fizzle, regression |
| ATOM-305.3-001 | 305.3 | Can't play land on opponent's turn | ALREADY-IMPL | S3 | land, timing |
| ATOM-305.6-001 | 305.6 | Basic land type → intrinsic mana ability | ALREADY-IMPL | S3 | land, mana |
| ATOM-305.8-002 | 305.8 | Land with "Basic" supertype IS basic (positive) | ALREADY-IMPL | S3 | land, supertype, positive |
| ATOM-305.8-001/002 | 305.8 | Basic = supertype, not subtype | L17 | S3 | land, boundary |
| ATOM-400.2-001 | 400.2 | Zone visibility classification (public vs hidden) | NEW — zone visibility query | S4 |  |
| ATOM-400.3-001 | 400.3 | Owner routing for library/graveyard/hand | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-400.5-001 | 400.5 | Zone ordering preserved (Vec) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-401.2-001 | 401.2 | Library face-down; no public visibility | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-401.3-001 | 401.3 | Any player can count any library | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-402.2-001 | 402.2 | Max hand size cleanup discard | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-402.2-002 | 402.2 | No hand-size enforcement outside cleanup | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-402.3-001 | 402.3 | Own hand visible, opponent hand hidden | NEW — hand visibility enforcement | S4 |  |
| ATOM-403.2-001 | 403.2 | Default scope is battlefield (destroy target creature) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-403.2-002 | 403.2 | Player-targeting can't target permanent | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-403.2-003 | 403.2 | Graveyard-targeting can't target battlefield | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-403.4-001 | 403.4 | ETB permanent is new object (no memory) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-404.1-001 | 404.1 | Resolved spell goes on top of graveyard | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-404.1-002 | 404.1 | Graveyards start empty | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-404.2-001 | 404.2 | Graveyard face-up, publicly visible | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-405.2-001 | 405.2 | Stack LIFO ordering | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-405.4-001 | 405.4 | Spell retains card characteristics on stack | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-405.4-002 | 405.4 | Ability on stack has no card characteristics | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-405.4-003 | 405.4 | Spell controller = caster | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-405.5-001 | 405.5 | All pass → top resolves | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-405.5-002 | 405.5 | Empty stack + all pass → phase/step ends | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-405.6c-001 | 405.6c | Mana abilities resolve immediately | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-405.6c-002 | 405.6c | Mana abilities don't use stack (negative test) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-405.6e-001 | 405.6e | TBAs don't use stack (draw step) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-405.6f-001 | 405.6f | SBAs don't use stack; happen before priority | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-500.1-001 | 500.1 | Five phases in fixed order every turn | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-500.2-001 | 500.2 | Phase/step ends only on empty stack + all pass | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-500.3-001 | 500.3 | No-priority steps end after TBAs complete | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-502.3-001 | 502.3 | Untap all permanents simultaneously (TBA) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-503.1-001 | 503.1 | Upkeep: no TBA, active player gets priority | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-504.1-001 | 504.1 | Draw step TBA (active player draws) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-504.1-002 | 504.1 | Starting player skips first draw step | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-505.1-001 | 505.1 | Two main phases separated by combat | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-505.1a-001 | 505.1a | Only first main phase is precombat | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-505.6a-001 | 505.6a | Sorcery-speed spells only during main phase | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-505.6b-001 | 505.6b | Land play timing (main phase, stack empty, has priority) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-505.6b-002 | 505.6b | Land play doesn't use stack | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-505.6b-003 | 505.6b | Already-played-land-this-turn rejection | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-506.1-001 | 506.1 | Combat phase five steps + skip rule | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-506.1-002 | 506.1 | Two combat damage steps with first/double strike | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-506.2-001 | 506.2 | Active player = attacker, nonactive = defender | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-506.3-001 | 506.3 | Only creatures can attack/block | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-506.4a-001 | 506.4a | Post-declaration restriction (Defender) doesn't remove | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-506.4b-001 | 506.4b | Tapping attacker doesn't remove from combat | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-508.1-001 | 508.1 | Declare attackers TBA (atomic validation) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-508.1a-001 | 508.1a | Attackers must be untapped | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-508.1a-002 | 508.1a | Summoning sickness / haste check | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-508.1c-001 | 508.1c | Defender restriction prevents attacking | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-508.1f-001 | 508.1f | Attackers tapped on declaration (not a cost) | ALREADY-IMPLEMENTED | S4 |  |
| COMP-COMBAT-FULL-001 | 508.1 + 509.1 + 510.1 + 510.2 + 511.3 | Full combat sequence: declare → block → damage → cleanup | ALREADY-IMPLEMENTED | S4 | ATOM-508.1-001, ATOM-509.1-001, ATOM-510.1-001, ATOM-510.2-001, ATOM-511.3-001 |
| ATOM-508.8-001 | 508.8 | Skip blockers/damage if no attackers | ALREADY-IMPLEMENTED | S4 |  |
| COMP-508.8-SKIP-001 | 508.8 + 506.1 | No attackers → skip blockers/damage → end of combat | ALREADY-IMPLEMENTED | S4 | ATOM-508.8-001, ATOM-506.1-001, ATOM-511.3-001 |
| ATOM-509.1-001 | 509.1 | Declare blockers TBA (atomic validation) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-509.1a-001 | 509.1a | Blockers must be untapped | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-509.1a-002 | 509.1a | Each blocker blocks exactly one attacker | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-509.1b-001 | 509.1b | Blocking restrictions (flying evasion) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-509.1b-003 | 509.1b | Post-declaration evasion change doesn't invalidate block | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-509.1g-001 | 509.1g | Creatures become blocking (BlockingInfo set) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-509.1h-001 | 509.1h | Blocked status persists after blocker removed | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-510.1-001 | 510.1 | Damage assignment TBA (attacker then blocker) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-510.1a-001 | 510.1a | Power = damage; 0 power = no damage | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-510.1b-001 | 510.1b | Unblocked creature assigns to attack target | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-510.1c-001 | 510.1c | Blocked, zero remaining blockers = no damage | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-510.1c-002 | 510.1c | Single blocker gets all damage | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-510.1c-003 | 510.1c | Multiple blockers: free damage division (2025 rules) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-510.1e-001 | 510.1e | Total damage assignment legality check | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-510.2-001 | 510.2 | All combat damage dealt simultaneously | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-510.3-001 | 510.3 | Priority after damage (SBA check first) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-510.4-001 | 510.4 | First strike two-step combat damage | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-510.4-002 | 510.4 | No first/double strike = single damage step | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-510.4-003 | 510.4 | Double strike deals damage in both steps | ALREADY-IMPLEMENTED | S4 |  |
| COMP-FIRST-STRIKE-KILL-001 | 510.4 + 510.1a + 510.1c | First-strike kills blocker before normal damage | ALREADY-IMPLEMENTED | S4 | ATOM-510.4-001, ATOM-510.1a-001, ATOM-510.1c-001 |
| ATOM-511.3-001 | 511.3 | All creatures removed from combat at step end | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-514.1-001 | 514.1 | Cleanup discard to max hand size (TBA) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-514.3-001 | 514.3 | No priority during cleanup (default) | ALREADY-IMPLEMENTED | S4 |  |
| ATOM-601.2a-001 | 601.2a | Card moves from hand to stack; controller set | N/A | S5 |  |
| ATOM-601.2c-001 | 601.2c | Target selection during casting | N/A | S5 |  |
| ATOM-601.2h-002 | 601.2h | Atomic cost payment — no partial | N/A | S5 |  |
| ATOM-601.2i-002 | 601.2i | Priority returns to caster after casting | N/A | S5 |  |
| COMP-601+608-002 | 601.2c, 608.2b | Fizzle on target removal | IMPL | S5 |  |
| COMP-601+608-001 | 601.2a–i, 608.2c, 608.2n | Full cast pipeline → resolution → GY | IMPL | S5 |  |
| ATOM-602.1a-001 | 602.1a | Activation cost charged to activating player | N/A | S5 |  |
| ATOM-602.2-001 | 602.2 | Activation restricted to controller | N/A | S5 |  |
| ATOM-602.2a-001 | 602.2a | Ability object created on stack | N/A | S5 |  |
| ATOM-602.5a-001 | 602.5a | Summoning sickness blocks {T}/{Q} activation | N/A | S5 |  |
| ATOM-602.5a-002 | 602.5a | Haste bypasses summoning sickness for tap/untap costs | N/A | S5 |  |
| COMP-601+604+605-001 | 604.2, 601.2f, 605.3a | Static cost reduction + mana + cast | Phase 5-Layers | S5 |  |
| ATOM-605.1a-001 | 605.1a | Mana ability classification for activated abilities | N/A | S5 |  |
| ATOM-605.2-001 | 605.2 | Mana ability classification is static not state-dependent | N/A | S5 |  |
| ATOM-605.3a-001 | 605.3a | Mana ability activation window during casting | N/A | S5 |  |
| ATOM-605.3a-003 | 605.3a | Mana ability window during ability activation cost | N/A | S5 |  |
| ATOM-605.3b-001 | 605.3b | Mana ability immediate resolution | N/A | S5 |  |
| ATOM-605.3c-001 | 605.3c | Mana ability re-activation lock (non-tap cost) | N/A | S5 | structural-guard |
| COMP-601+602+605-001 | 605.3a, 601.2g, 605.3b | Mana ability during casting mana window | IMPL | S5 |  |
| ATOM-605.5b-001 | 605.5b | Spells always use the stack regardless of mana output | N/A | S5 |  |
| ATOM-608.1-001 | 608.1 | Stack resolution trigger on all-pass | N/A | S5 |  |
| ATOM-608.2b-001 | 608.2b | All-targets-illegal fizzle | N/A | S5 |  |
| ATOM-608.2b-003 | 608.2b | Zone-change makes target illegal | N/A | S5 |  |
| ATOM-608.2c-001 | 608.2c | Sequential instruction execution | N/A | S5 |  |
| ATOM-608.2n-001 | 608.2n | Post-resolution zone transition (spell → GY) | N/A | S5 |  |
| ATOM-608.2n-002 | 608.2n | Ability removal from stack post-resolution | N/A | S5 |  |
| ATOM-608.3a-001 | 608.3a | Untargeted permanent spell → ETB | N/A | S5 |  |
| ATOM-702.19d-001 | 702.19d | Trample all blockers removed → all to defender | NEW-2 | S7b | trample, all-blockers-removed, regression |
| ATOM-702.111b-001 | 702.111b | Menace: requires 2+ blockers | T21 | S8 |  |
| ATOM-702.111b-002 | 702.111b | Menace: legal block with 2+ creatures | T21 | S8 |  |
| ATOM-703.3-001 | 703.3 | TBAs execute before SBAs and priority | ALREADY-IMPLEMENTED | S9a |  |
| ATOM-703.4c-001 | 703.4c | Untap TBA — all permanents untap simultaneously | ALREADY-IMPLEMENTED | S9a |  |
| ATOM-703.4d-001 | 703.4d | Draw step TBA — draw one card | ALREADY-IMPLEMENTED | S9a |  |
| ATOM-703.4q-001 | 703.4q | Mana pool empties at end of step/phase | ALREADY-IMPLEMENTED | S9a |  |
| ATOM-704.3-001 | 704.3 | SBA check-repeat loop — simultaneous destruction | ALREADY-IMPLEMENTED | S9a |  |

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

---

## Phase 5-Layers

**178 entries**

| ID | Rule | Summary | Ticket | Session | Tags |
|----|------|---------|--------|---------|------|
| ATOM-105.2-002 | 105.2 | Card color derived from mana cost colors | L01 | S1 |  |
| ATOM-105.2-003 | 105.2 | Color indicator overrides mana cost | L01 | S1 |  |
| ATOM-105.2a-001 | 105.2a | Multicolor = 2+ colors | L01 | S1 |  |
| ATOM-105.2b-001 | 105.2b | Colorless mana cost → colorless card | L01 | S1 |  |
| ATOM-105.2b-002 | 105.2b | No mana cost + no color indicator → colorless | L01 | S1 |  |
| ATOM-105.2c-001 | 105.2c | Devoid CDA makes card colorless | L18 | S1 |  |
| ATOM-105.2c-002 | 105.2c | Devoid card retains colored mana cost | L18 | S1 |  |
| ATOM-107.1b-002 | 107.1b | Negative power allowed on creatures | L04 | S1 |  |
| ATOM-109.3-001 | 109.3 | Characteristics vs non-characteristics | L01 | S1 | boundary |
| ATOM-110.2b-001 | 110.2b | Gain control of spell → control permanent | L11 | S1 |  |
| ATOM-110.4c-001 | 110.4c | Typeless permanent stays on battlefield | L09 | S1 |  |
| ATOM-113.6-001 | 113.6 | Permanent ability functions only on battlefield | L03 | S1 |  |
| ATOM-113.6a-001 | 113.6a | CDA functions in all zones | L18 | S1 |  |
| ATOM-113.10b-001 | 113.10b | "Loses [ability]" removes all instances | L06 | S1 |  |
| ATOM-113.12-001 | 113.12 | P/T CDA applied in Layer 7a | L18 | S1 | boundary |
| ATOM-113.12-002 | 113.12 | Color CDA (Devoid) applied in Layer 5 | L18 | S1 | boundary |
| ATOM-118.7-001 | 118.7 | Cost reduced to partial → remaining cost | T18 | S1 |  |
| ATOM-118.7-002 | 118.7 | Cost reduced to {0} → free cast | T18 | S1 |  |
| ATOM-118.7a-001 | 118.7a | Generic reduction only affects generic component | T18 | S1 |  |
| ATOM-118.7b-001 | 118.7b | Colored reduction on missing color → reduce generic | T18 | S1 |  |
| ATOM-118.7c-001 | 118.7c | Excess colored reduction overflows to generic | T18 | S1 |  |
| ATOM-118.7d-001 | 118.7d | Excess colorless reduction overflows to generic | T18 | S1 |  |
| ATOM-118.8d-001 | 118.8d | Additional costs don't change mana value | L01 | S1 |  |
| ATOM-118.9c-001 | 118.9c | Alt cost doesn't change mana value | L01 | S1 |  |
| ATOM-118.9d-001 | 118.9d | Cost modifications apply to alt costs | T18 | S1 |  |
| ATOM-122.1a-001 | 122.1a | +1/+1 counter adds to P/T | L04/L08 | S1 |  |
| ATOM-122.1a-002 | 122.1a | -1/-1 counter subtracts from P/T | L04/L08 | S1 |  |
| ATOM-122.1a-003 | 122.1a | Non-standard P/T counters don't annihilate | L04/L08 | S1 |  |
| ATOM-122.1b-001 | 122.1b | Keyword counter grants keyword | L06 | S1 |  |
| ATOM-202.2-001 | 202.2 | Color derived from mana cost ({2}{W} → white) | L10 | S2 |  |
| ATOM-202.2-002 | 202.2 | Multicolor from mana cost ({2}{W}{B} → W+B) | L10 | S2 |  |
| ATOM-202.2b-001 | 202.2b | Pure-generic cost → colorless | L10 | S2 |  |
| ATOM-202.2b-002 | 202.2b | No mana cost + no color indicator → colorless | L10 | S2 |  |
| ATOM-202.2c-001 | 202.2c | Two+ colored symbols → multicolored | L10 | S2 |  |
| ATOM-202.2d-001 | 202.2d | Hybrid mana contributes all colors | L10 | S2 |  |
| ATOM-202.2d-002 | 202.2d | Phyrexian mana contributes its color | L10 | S2 |  |
| ATOM-202.2e-001 | 202.2e | Color indicator overrides absent mana cost | T05, L10 | S2 |  |
| COMP-202.2e+202.2b-001 | 202.2e + 202.2b | Color indicator on costless card → green | Phase 5-Pre + Layers | S2 | T05, L10 |
| ATOM-202.3-001 | 202.3 | Mana value = total mana in cost | L10 | S2 |  |
| ATOM-202.3a-001 | 202.3a | No mana cost → MV 0 | L10 | S2 |  |
| ATOM-202.3e-001 | 202.3e | X = 0 off-stack for MV | L10 | S2 |  |
| ATOM-202.3e-002 | 202.3e | X = chosen value on stack for MV | T06, L10 | S2 |  |
| ATOM-202.3f-001 | 202.3f | Hybrid MV uses largest component | L10 | S2 |  |
| ATOM-202.3f-002 | 202.3f | MonoHybrid {2/C} contributes 2 to MV | L10 | S2 |  |
| ATOM-202.3g-001 | 202.3g | Phyrexian mana contributes 1 each to MV | L10 | S2 |  |
| ATOM-205.1a-001 | 205.1a | Type replacement — new type replaces existing | L10 | S2 |  |
| ATOM-205.1a-002 | 205.1a | Instant/sorcery type retention during type set | L10 | S2 |  |
| ATOM-205.1a-003 | 205.1a | Subtype set replacement scoped by card type | L10 | S2 |  |
| ATOM-205.1a-004 | 205.1a | Subtype removal on type loss (correlated subtypes) | L10 | S2 |  |
| ATOM-205.1a-005 | 205.1a | Counters/effects/damage persist across type change | L10 | S2 |  |
| ATOM-205.1b-001 | 205.1b | "Artifact creature" retains all prior types | L10 | S2 |  |
| ATOM-205.1b-002 | 205.1b | "Becomes a [type] artifact creature" replaces creature subtypes only | L10 | S2 |  |
| ATOM-205.1b-003 | 205.1b | "Still a [type]" retention language | L10 | S2 |  |
| ATOM-205.1b-004 | 205.1b | "In addition to its other types" retention | L10 | S2 |  |
| ATOM-205.2b-001 | 205.2b | Multi-type object satisfies criteria for any type | L13 | S2 |  |
| ATOM-205.3c-001 | 205.3c | Subtype-to-card-type correlation query | L10 | S2 |  |
| ATOM-205.3d-001 | 205.3d | Can't gain non-corresponding subtype | L10 | S2 |  |
| ATOM-205.4b-001 | 205.4b | Type-supertype independence (type change preserves supertypes) | L10 | S2 |  |
| ATOM-205.4c-001 | 205.4c | Basic supertype → is basic land (positive) | L10 | S2 |  |
| ATOM-205.4c-002 | 205.4c | No Basic supertype → nonbasic (even with basic land types) | L10 | S2 |  |
| COMP-205.4c+L10-001 | 205.4c + Blood Moon | Blood Moon doesn't add Basic supertype | Phase 5 Layers | S2 | L10, L17 |
| ATOM-208.2a-001 | 208.2a | CDA P/T in all zones (Tarmogoyf) | L04, L17 | S2 |  |
| ATOM-208.2a-002 | 208.2a | CDA undetermined value → 0 | L04 | S2 |  |
| ATOM-208.3-001 | 208.3 | Noncreature permanent has no P/T (Vehicle not crewed) | L04 | S2 |  |
| ATOM-208.3-002 | 208.3 | Noncreature off-battlefield has printed P/T (Vehicle in GY) | L04 | S2 |  |
| ATOM-208.3a-001 | 208.3a | Dormant P/T effect activates when noncreature becomes creature | L04, L07 | S2 |  |
| COMP-208.3+205.1b-001 | 208.3 + 205.1b | Animated enchantment gains P/T (Opalescence) | Phase 5 Layers | S2 | L04, L19 |
| ATOM-208.4b-001 | 208.4b | "Base P/T" query ignores L7c/L7d | NEW-S2-17 | S2 |  |
| ATOM-208.5-001 | 208.5 | No P/T value → fallback to 0 | L04 | S2 |  |
| ATOM-301.5d-001 | 301.5d | Control change of creature doesn't change Equipment controller | L11 | S3 | equipment, control |
| COMP-301.5d+303.4e-001 | ATOM-301.5d-001, ATOM-303.4e-001 | Control change: creature changes, Equipment+Aura stay with original | L11 | S3 |  |
| ATOM-302.4-001 | 302.4 | Non-creature has no P/T | L04 | S3 | creature, characteristic |
| ATOM-302.4-002 | 302.4 | Creature has P/T (positive test) | L04 | S3 | creature, characteristic, positive |
| ATOM-302.4-001/002 | 302.4 | P/T: only creatures have it | L04 | S3 | creature, boundary |
| ATOM-302.4c-001 | 302.4c | P/T = printed + continuous effects (Giant Growth) | L07, L08 | S3 | creature, pt, layers |
| ATOM-303.4e-001 | 303.4e | Control change of enchanted object doesn't change Aura controller | L11 | S3 | aura, control |
| COMP-303.4f+303.4c-001 | ATOM-303.4f-001, ATOM-303.4c-001 | Aura from GY attaches to creature, type removed → SBA | T15, T15b, L09 | S3 |  |
| COMP-303.4c+303.4e+L11-001 | ATOM-303.4c-001, ATOM-303.4e-001 | "Enchant creature you control" Aura falls off on control change | L11, T15, T15b | S3 |  |
| ATOM-305.2-002 | 305.2 | Continuous effect increases land limit; second land OK | L15 | S3 | land, limit, continuous |
| ATOM-305.2a-001 | 305.2a | "Put" land doesn't count; player can still play | T22 | S3 | land, counter |
| ATOM-305.6-002 | 305.6 | Non-basic gains basic land type (Urborg) → gains mana ability | L17 | S3 | land, urborg, mana |
| ATOM-305.6-003 | 305.6 | Non-basic land types (Desert, Gate) don't grant mana ability | L17 | S3 | land, boundary |
| ATOM-305.7-001 | 305.7 | Set land subtype to basic → removes old land types | L17 | S3 | land, blood-moon |
| ATOM-305.7-002 | 305.7 | Set to basic type → loses rules-text abilities, gains intrinsic mana | L17 | S3 | land, blood-moon |
| ATOM-305.7-003 | 305.7 | Set to basic type → keeps abilities granted by other effects | L17 | S3 | land, blood-moon, granted |
| ATOM-305.7-004 | 305.7 | Set subtypes doesn't change card types or supertypes | L17 | S3 | land, blood-moon, supertype |
| ATOM-305.7-005 | 305.7 | "In addition to" keeps old types + adds new types/mana | L17, NEW | S3 | land, additive |
| COMP-305.7+305.6-001 | ATOM-305.7-001, ATOM-305.7-002, ATOM-305.6-002 | Blood Moon sets land to Mountain; gains {R}, loses old abilities | L17 | S3 |  |
| COMP-305.7+305.8-001 | ATOM-305.7-004, ATOM-305.8-001 | Blood Moon: nonbasic stays nonbasic, keeps Legendary | L17 | S3 |  |
| ATOM-305.8-001 | 305.8 | Land with basic type but no "Basic" supertype = nonbasic | L17 | S3 | land, supertype, boundary |
| ATOM-306.5a-001 | 306.5a | Non-BF PW loyalty = printed number | T14, L04 | S3 | planeswalker, loyalty |
| ATOM-601.2a-002 | 601.2a | Spell-modifying continuous effects apply on stack entry | L15, T18 | S5 |  |
| ATOM-601.2f-001 | 601.2f | Cost assembly pipeline | T18, L15 | S5 |  |
| ATOM-601.2f-002 | 601.2f | Cost floor at {0} | L15 | S5 |  |
| ATOM-601.2f-004 | 601.2f | Cost reduction ordering choice via DP | L15 | S5 |  |
| ATOM-601.2i-003 | 601.2i | Spell characteristics modified by continuous effects on completion | L10 | S5 |  |
| ATOM-601.3-001 | 601.3 | Cast permission check | L15 | S5 |  |
| ATOM-601.7-001 | 601.7 | Cost alteration doesn't retroactively modify stack | T18 | S5 |  |
| ATOM-602.1e-001 | 602.1e | Activation cost modifications apply to total | L15 | S5 |  |
| ATOM-602.1e-002 | 602.1e | Single cost increase on activation cost | L15 | S5 |  |
| ATOM-602.4-001 | 602.4 | Cost alteration from abilities doesn't retroactively affect stack | L15 | S5 |  |
| ATOM-602.5-001 | 602.5 | Activation prohibition check | L15, T19 | S5 |  |
| ATOM-603.6b-001 | 603.6b | Continuous effects apply at moment of ETB | Phase 7, L13 | S5 |  |
| ATOM-603.6b-002 | 603.6b | Ability removal prevents ETB trigger | L09, Phase 7 | S5 |  |
| ATOM-604.2-001 | 604.2 | Static ability lifetime = source lifetime on BF | L08 | S5 |  |
| ATOM-604.3-001 | 604.3 | CDA applies in all zones | L04 | S5 |  |
| ATOM-604.3a-001 | 604.3a | CDA classification validation | L01 | S5 |  |
| ATOM-604.4-001 | 604.4 | Attachment-based static effect re-targeting | T04, L08 | S5 |  |
| ATOM-604.7-001 | 604.7 | Static ability CDA fails when source gone (no LKI) | L18 | S5 |  |
| ATOM-608.2b-004 | 608.2b | LKI for source during target recheck | L18 | S5 |  |
| ATOM-608.2h-001 | 608.2h | Single-determination + LKI fallback | L18 | S5 |  |
| ATOM-608.2h-002 | 608.2h | LKI for departed source | L18 | S5 |  |
| ATOM-608.2j-001 | 608.2j | Strict characteristic matching | L10 | S5 |  |
| ATOM-609.2-001 | 609.2 | Effects default to permanents only | L10 | S6 |  |
| ATOM-609.2-002 | 609.2 | Effects applying to non-permanent zones (Thalia cost increase) | L15 | S6 |  |
| ATOM-611.2a-001 | 611.2a | Spell continuous effect with stated duration expires | L07 | S6 |  |
| ATOM-611.2a-002 | 611.2a | No stated duration → lasts indefinitely | L02 | S6 |  |
| ATOM-611.2c-001 | 611.2c | Characteristic-modifying effect locks in affected set | L07 | S6 |  |
| ATOM-611.2d-001 | 611.2d | Variable X locked at resolution | L07 | S6 |  |
| ATOM-611.3a-001 | 611.3a | Static ability effect applies dynamically | L08 | S6 |  |
| ATOM-611.3b-001 | 611.3b | Static effect ceases when source leaves battlefield | L08 | S6 |  |
| ATOM-611.3c-001 | 611.3c | Static effect applies as permanent enters, not after | L08 | S6 |  |
| ATOM-612.2-001 | 612.2 | Text change respects word context (color word) | L12 | S6 |  |
| ATOM-612.2a-001 | 612.2a | Text change affects creature-type-derived token names | L12 | S6 |  |
| ATOM-612.3-001 | 612.3 | Granted abilities immune to text change on object | L12 | S6 |  |
| ATOM-612.8-001 | 612.8 | Effect that sets name replaces all existing names | L12 | S6 |  |
| ATOM-613.1-001 | 613.1 | Layer system applies effects in order 1–7 | L04, L10 | S6 |  |
| ATOM-613.1b-001 | 613.1b | Layer 2 control change before type/color/ability/PT | L11 | S6 |  |
| ATOM-613.1c-001 | 613.1c | Layer 3 text change before type/color/ability/PT | L12 | S6 |  |
| ATOM-613.1d-001 | 613.1d | Layer 4 type change before color/ability/PT (Opalescence) | L10, L19 | S6 |  |
| ATOM-613.1e-001 | 613.1e | Layer 5 color change before ability/PT | L10 | S6 |  |
| ATOM-613.1f-001 | 613.1f | Layer 6 ability changes before P/T (Humility + Tarmogoyf) | L09, L19 | S6 |  |
| ATOM-613.1f-002 | 613.1f | Keyword counters grant abilities in L6 | NEW-613.1f-kw | S6 |  |
| ATOM-613.1g-001 | 613.1g | Layer 7 P/T is final (counter + Growth additive) | L06, L07 | S6 |  |
| COMP-613-HUMILITY-OPALESCENCE-001 | 613.1f+1g+4b+7 | Humility + Opalescence timestamp | L19, L20 | S6 | ATOM-613.1f-001, ATOM-613.4b-001, ATOM-613.7-001 (was -002, subsumed) |
| COMP-613-TARMOGOYF-HUMILITY-001 | 613.1f+4a+4b | Tarmogoyf under Humility = 1/1 | L19, L20 | S6 | ATOM-613.1f-001, ATOM-613.4a-001, ATOM-613.4b-001 |
| ATOM-613.3-001 | 613.3 | CDAs apply before non-CDAs in L2–6 (L5 color test) | L04 | S6 |  |
| ATOM-613.4a-001 | 613.4a | L7a CDAs set P/T before other effects (Tarmogoyf) | L04, L17 | S6 |  |
| ATOM-613.4b-001 | 613.4b | L7b P/T setting overrides L7a CDA | L04, L07 | S6 |  |
| ATOM-613.4b-002 | 613.4b | L7b with subsequent L7c modifiers | L04, L06, L07 | S6 |  |
| ATOM-613.4c-001 | 613.4c | L7c counters and effects stack additively | L06, L07, L08 | S6 |  |
| ATOM-613.4d-001 | 613.4d | Basic P/T switch | L04 | S6 |  |
| ATOM-613.4d-002 | 613.4d | Switch with subsequent modifier removal | L04 | S6 |  |
| ATOM-613.4d-003 | 613.4d | Double switch cancels out | L04 | S6 |  |
| ATOM-613.4d-004 | 613.4d | Modifier after switch applies unswitched then re-switches | L04 | S6 |  |
| ATOM-613.5-001 | 613.5 | Color change in L5 immediately triggers L7 re-evaluation | L08, L10 | S6 |  |
| ATOM-613.5-002 | 613.5 | Complex multi-layer interaction (Gray Ogre CR example) | L04, L06, L07, L08 | S6 |  |
| ATOM-613.6-001 | 613.6 | Multi-layer effect applies to same locked set | L05 | S6 |  |
| ATOM-613.6-002 | 613.6 | Act of Treason: control in L2, haste in L6 | L09, L11 | S6 |  |
| ATOM-613.6-003 | 613.6 | Multi-layer effect persists after generating ability removed | L05 | S6 |  |
| COMP-613-SVOGTHOS-001 | 613.6+613.4a | Svogthos multi-layer activation (L4+L5+L7a) | L04, L05, L10 | S6 | ATOM-613.6-001, ATOM-613.4a-001, ATOM-613.3-001 |
| ATOM-613.7-001 | 613.7 | Timestamp ordering via conflicting L7b set-PT effects | L03 | S6 |  |
| ATOM-613.7b-001 | 613.7b | Spell effect timestamp set at creation | L07 | S6 |  |
| ATOM-613.7d-001 | 613.7d | Object timestamp set on zone entry | L03 | S6 |  |
| ATOM-613.7m-001 | 613.7m | APNAP ordering for simultaneous timestamps | L03 | S6 |  |
| ATOM-613.7n-001 | 613.7n | Static ability timestamp < resolving effect when simultaneous | L08 | S6 |  |
| ATOM-613.8-001 | 613.8 | Dependency overrides timestamp order | L14 | S6 |  |
| ATOM-613.8a-001 | 613.8a | Blood Moon + Urborg dependency analysis | L14, L17 | S6 |  |
| ATOM-613.8a-002 | 613.8a | Dependency condition (b): existence change | L14 | S6 |  |
| ATOM-613.8a-003 | 613.8a | CDA guard: CDA + non-CDA always independent | L14 | S6 |  |
| ATOM-613.8b-001 | 613.8b | Circular dependency falls back to timestamp | L14 | S6 |  |
| ATOM-613.8c-001 | 613.8c | Dependency re-evaluation after each application | L14 | S6 |  |
| COMP-613-BLOOD-MOON-URBORG-001 | 613.8a+8 | Blood Moon + Urborg dependency | L14, L17, L20 | S6 | ATOM-613.8a-001, ATOM-613.8-001 |
| ATOM-613.9-001 | 613.9 | Conflicting ability effects: later timestamp wins | L09 | S6 |  |
| ATOM-613.9-002 | 613.9 | Color change causes downstream effect to apply | L08, L10 | S6 |  |
| ATOM-613.10-001 | 613.10 | Player-affecting continuous effects applied after object chars | L15 | S6 |  |
| ATOM-613.11-001 | 613.11 | Game-rule-modifying effects apply after L1–L7 | L15 | S6 |  |
| ATOM-613.11-002 | 613.11 | Cost modification: increases before reductions before Trinisphere | L15 | S6 |  |
| ATOM-700.5a-001 | 700.5a | Devotion partial-layer (after L1–L3, before L4–L7) | NEW (6b-D14) | S7a | dependency, layers |
| ATOM-700.5a-002 | 700.5a | Devotion modifier effect (Altar of the Pantheon) | NEW | S7a |  |
| ATOM-701.10a-001 | 701.10a | Double P/T — L7c continuous effect (+X/+Y) | NEW | S7a |  |
| ATOM-701.10b-001 | 701.10b | Double power only — snapshot at resolution | NEW | S7a |  |
| ATOM-701.10c-001 | 701.10c | Double negative power — becomes more negative | NEW | S7a |  |
| COMP-7A-006 | 701.10a + 613.4b/c | Double P/T + layer ordering with set-P/T effect |  | S7a |  |
| ATOM-701.11a-001 | 701.11a | Triple P/T — L7c continuous effect (+2X/+2Y) | NEW | S7a |  |
| ATOM-701.11b-001 | 701.11b | Triple power only | NEW | S7a |  |
| ATOM-701.11c-001 | 701.11c | Triple negative power | NEW | S7a |  |
| ATOM-701.66a-001 | 701.66a | Earthbend N — land animation + counters + delayed trigger | NEW | S7a |  |

---

## Phase 6

**138 entries**

| ID | Rule | Summary | Ticket | Session | Tags |
|----|------|---------|--------|---------|------|
| ATOM-106.6a-001 | 106.6a | Mana replacement restriction propagation | NEW-CH1-009 | S1 |  |
| ATOM-110.5b-002 | 110.5b | "Enters tapped" ETB replacement | Phase 6 | S1 |  |
| ATOM-119.10-001 | 119.10 | Life gain replacement doesn't apply to 0 gain | Phase 6 | S1 |  |
| ATOM-121.2a-001 | 121.2a | Draw replacement modifies count before draws | Phase 6 | S1 |  |
| ATOM-121.6a-001 | 121.6a | Draw replacement applies even with empty library | Phase 6 | S1 |  |
| ATOM-122.1c-001 | 122.1c | Shield counter prevents destruction | Phase 6 | S1 |  |
| ATOM-122.1c-002 | 122.1c | Shield counter prevents damage | Phase 6 | S1 |  |
| ATOM-122.1d-001 | 122.1d | Stun counter prevents untap | Phase 6 | S1 |  |
| ATOM-122.1h-001 | 122.1h | Finality counter → exile instead of graveyard | Phase 6 | S1 |  |
| ATOM-122.6a-001 | 122.6a | Controller places ETB counters | Phase 6 | S1 |  |
| ATOM-400.6-001 | 400.6 | Replacement effects applied before zone change | NEW — replacement effects on zone transitions | S4 |  |
| ATOM-400.6-002 | 400.6 | Contradictory replacement effects — controller chooses | NEW — replacement effect conflict resolution | S4 |  |
| ATOM-400.7c-001 | 400.7c | Prevention effect on spell continues to permanent | NEW — prevention effect continuity | S4 |  |
| ATOM-400.7c-002 | 400.7c | Prevention on source ≠ prevention on tokens from source | NEW — prevention effect source identity | S4 |  |
| COMP-603+608-001 | 603.2, 603.3, 603.6a, 608.2c | ETB trigger → stack → resolution | Phase 7 | S5 |  |
| ATOM-603.6d-001 | 603.6d | "Enters tapped" is NOT a triggered ability | Phase 6 | S5 |  |
| COMP-607+608-001 | 607.1, 607.2a, 608.3a, 603.6c | O-Ring exile + return linked pattern | Phase 5-Pre/7 | S5 |  |
| ATOM-607.2b-001 | 607.2b | Replacement-exile linking | T20, Phase 6 | S5 |  |
| ATOM-607.2g-001 | 607.2g | ETB-cost linking | Phase 6 | S5 |  |
| ATOM-608.3e-001 | 608.3e | ETB prohibition fallback | Phase 6 | S5 |  |
| ATOM-609.7a-001 | 609.7a | All valid damage source categories selectable | NEW-609.7a | S6 |  |
| BOUNDARY-DEF-609.7a-001 | 609.7a | Valid damage source choices (permanent vs hand card) | NEW-609.7a | S6 |  |
| ATOM-609.7b-001 | 609.7b | Prevention shield rechecks source properties | NEW-609.7b | S6 |  |
| ATOM-609.7c-001 | 609.7c | Static prevention applies to non-battlefield sources | NEW-609.7c | S6 |  |
| ATOM-611.2c-002 | 611.2c | Game-rule-modifying effect does NOT lock in | NEW-611.2c-mix | S6 |  |
| ATOM-613.1a-001 | 613.1a | Layer 1 copy before P/T modification | NEW-613.1a | S6 |  |
| ATOM-613.2a-001 | 613.2a | Layer 1a copiable effects establish base chars | NEW-613.2a | S6 |  |
| ATOM-613.2c-001 | 613.2c | Post-Layer 1 chars are copiable values (clone of clone) | NEW-613.2c | S6 |  |
| BOUNDARY-DEF-614.1a-001 | 614.1a | "Instead" identifies replacement effect | NEW-614.1a-d | S6 |  |
| BOUNDARY-DEF-614.1b-001 | 614.1b | "Skip" identifies replacement effect | NEW-614.1a-d | S6 |  |
| BOUNDARY-DEF-614.1c-001 | 614.1c | ETB modification patterns are replacement effects | NEW-614.1a-d | S6 |  |
| BOUNDARY-DEF-614.1d-001 | 614.1d | Continuous ETB modification is replacement effect | NEW-614.1a-d | S6 |  |
| ATOM-614.4-001 | 614.4 | Replacement must exist before the event | NEW-614.4 | S6 |  |
| ATOM-614.5-001 | 614.5 | Replacement doesn't self-repeat (two doublers = 4x) | NEW-614.5 | S6 |  |
| COMP-614-616-DOUBLE-REPLACEMENT-001 | 614.5+616.1 | Two doublers with player choice = 8x | NEW | S6 | ATOM-614.5-001, ATOM-616.1-001 |
| COMP-614-DAMAGE-ORDERING-001 | 614.5+616.1 | Non-commutative damage mods: +1 then ×2 vs ×2 then +1 | NEW | S6 | ATOM-614.5-001, ATOM-616.1-001 |
| ATOM-614.6-001 | 614.6 | Replaced event never happens; modified event triggers | NEW-614.6 | S6 |  |
| ATOM-614.7-001 | 614.7 | Replacement of non-event is a no-op | NEW-614.7 | S6 |  |
| ATOM-614.7a-001 | 614.7a | Zero damage is non-event; "+1" doesn't upgrade 0 | NEW-614.7 | S6 | ALREADY-IMPLEMENTED (partial) |
| ATOM-614.8-001 | 614.8 | Regeneration replaces destruction | NEW-614.8 | S6 |  |
| ATOM-614.8-002 | 614.8 | Regeneration: damage triggers still fire | NEW-614.8 | S6 |  |
| ATOM-614.9-001 | 614.9 | Damage redirection: destination gone → no-op | NEW-614.9 | S6 |  |
| ATOM-614.10-001 | 614.10 | Skip replaces step/phase with nothing | NEW-614.10 | S6 |  |
| ATOM-614.10-002 | 614.10 | Skip mid-step doesn't end current step | NEW-614.10 | S6 |  |
| ATOM-614.10a-001 | 614.10a | Two skip effects consume two occurrences | NEW-614.10 | S6 |  |
| ATOM-614.10b-001 | 614.10b | Skip with follow-up defers to next real occurrence | NEW-614.10 | S6 |  |
| ATOM-614.11-001 | 614.11 | Draw replacement applies even with empty library | NEW-614.11 | S6 |  |
| ATOM-614.11-002 | 614.11 | Lab Maniac: draw replacement win condition | NEW-614.11 | S6 |  |
| ATOM-614.11a-001 | 614.11a | Draw replacement completes before sequence resumes | NEW-614.11 | S6 |  |
| ATOM-614.11b-001 | 614.11b | Additional action on drawn card lost if draw replaced | NEW-614.11 | S6 |  |
| ATOM-614.12-001 | 614.12 | ETB replacement uses look-ahead chars (Scarwood + Jailer) | NEW-614.12 | S6 |  |
| ATOM-614.12-002 | 614.12 | ETB look-ahead: own static abilities apply (Voice of All) | NEW-614.12 | S6 |  |
| ATOM-614.12-003 | 614.12 | ETB look-ahead: self doesn't affect self (Orb of Dreams) | NEW-614.12 | S6 |  |
| ATOM-614.12a-001 | 614.12a | ETB replacement choices made before entering | NEW-614.12 | S6 |  |
| ATOM-614.13-001 | 614.13 | ETB replacement causes other zone changes (Devour) | NEW-614.13 | S6 |  |
| ATOM-614.13a-001 | 614.13a | Can't choose entering/simultaneous objects (Sutured Ghoul) | NEW-614.13 | S6 |  |
| ATOM-614.13b-001 | 614.13b | Object can't be chosen for multiple ETB replacements | NEW-614.13 | S6 |  |
| ATOM-614.15-001 | 614.15 | Self-replacement applies before other replacements | NEW-614.15 | S6 |  |
| ATOM-614.15-002 | 614.15 | Self-replacement real card: Aang's Journey kicked search | NEW-614.15 | S6 |  |
| ATOM-614.16-001 | 614.16 | Token replacement applies to tokens from other replacements | NEW-614.16 | S6 |  |
| ATOM-614.17-001 | 614.17 | "Can't" overrides prevention | NEW-614.17 | S6 | META-101.2 |
| ATOM-614.17a-001 | 614.17a | "Can't" must pre-exist the event | NEW-614.17 | S6 |  |
| ATOM-614.17b-001 | 614.17b | Can't pay costs involving impossible events | NEW-614.17 | S6 | META-101.2 |
| ATOM-614.17c-001 | 614.17c | "Can't" event only replaceable by self-replacement changing type | NEW-614.17 | S6 | META-101.2 |
| ATOM-614.17d-001 | 614.17d | ETB "can't" uses look-ahead (enters tapped) | NEW-614.17 | S6 | META-101.2, META-614.17d |
| BOUNDARY-DEF-615.1a-001 | 615.1a | "Prevent" identifies prevention effect | NEW-615.4 | S6 |  |
| ATOM-615.4-001 | 615.4 | Prevention must exist before damage event | NEW-615.4 | S6 |  |
| ATOM-615.5-001 | 615.5 | Prevention with additional effect fires after prevention | NEW-615.5 | S6 |  |
| ATOM-615.6-001 | 615.6 | Prevented damage never happens; suppresses triggers | NEW-615.6 | S6 |  |
| ATOM-615.7-001 | 615.7 | Prevention shield depletes per damage point | NEW-615.7 | S6 |  |
| ATOM-615.7-002 | 615.7 | Multiple simultaneous sources: controller chooses allocation | NEW-615.7 | S6 |  |
| ATOM-615.8-001 | 615.8 | Prevent next instance from source: amount-independent | NEW-615.8 | S6 |  |
| ATOM-615.9-001 | 615.9 | Prevention shield rechecks source properties | NEW-615.9 | S6 |  |
| ATOM-615.10-001 | 615.10 | Static prevention applies per-event independently (Pyroclasm) | NEW-615.10 | S6 |  |
| ATOM-615.11-001 | 615.11 | Per-creature prevention shield assigned at resolution | NEW-615.11 | S6 |  |
| ATOM-615.12-001 | 615.12 | Unpreventable damage: shields preserved | NEW-615.12 | S6 | META-101.2 |
| ATOM-615.12-002 | 615.12 | Unpreventable damage: additional effect conditional on 0 | NEW-615.12 | S6 |  |
| ATOM-615.12a-001 | 615.12a | Prevention on unpreventable: single application, no loop | NEW-615.12 | S6 |  |
| COMP-615-UNPREVENTABLE-SHIELD-001 | 615.12+615.7 | Unpreventable damage preserves shield | NEW | S6 | ATOM-615.12-001, ATOM-615.7-001 |
| ATOM-616.1-001 | 616.1 | Player chooses which replacement/prevention to apply | NEW-616.1 | S6 |  |
| ATOM-616.1a-001 | 616.1a | Self-replacement must be chosen first | NEW-616.1 | S6 |  |
| ATOM-616.1b-001 | 616.1b | Control-changing replacement second priority | NEW-616.1 | S6 |  |
| ATOM-616.1c-001 | 616.1c | Copy replacement third priority (Essence of the Wild) | NEW-616.1 | S6 |  |
| ATOM-616.1f-001 | 616.1f | Iterative application until no more apply | NEW-616.1 | S6 |  |
| ATOM-616.1g-001 | 616.1g | Outer event replacement before inner event replacement | NEW-616.1 | S6 |  |
| ATOM-616.2-001 | 616.2 | Replacement chains: new replacement applies to modified event | NEW-616.2 | S6 |  |
| ATOM-701.8b-001 | 701.8b | Destroy delta tagging — destroy vs sacrifice distinction | NEW (D20) | S7a | dependency, replacement-effects |
| ATOM-701.8c-001 | 701.8c | Regeneration replaces destruction | NEW | S7a | dependency, replacement-effects |
| ATOM-701.10g-001 | 701.10g | Damage doubling replacement effect | NEW | S7a | dependency, replacement-effects |
| ATOM-701.19a-001 | 701.19a | Regeneration shield — one-shot replacement on destroy | NEW | S7a | replacement-effects |
| ATOM-701.19a-002 | 701.19a | Regen shield single-use — second destroy succeeds | NEW | S7a |  |
| ATOM-701.19b-001 | 701.19b | Static regeneration — replaces every destruction | NEW | S7a |  |
| ATOM-701.19c-001 | 701.19c | Can't-be-regenerated blocks shield application | NEW | S7a |  |
| COMP-7A-004 | 701.19a + 701.19c + 701.8a | Regen shield vs can't-be-regenerated + destroy |  | S7a |  |
| ATOM-701.40f-001 | 701.40f | ETB prohibition prevents manifest | NEW | S7a |  |
| ATOM-702.2d-001 | 702.2d | Deathtouch from non-battlefield zone | DEFERRED | S7b | deathtouch, any-zone |
| ATOM-702.15d-001 | 702.15d | Lifelink from non-battlefield zone | DEFERRED | S7b | lifelink, any-zone |
| ATOM-702.82a-001 | 702.82a | Devour N: sacrifice creatures on ETB, N×devour counters | NEW — Devour | S8 |  |
| ATOM-702.84a-002 | 702.84a | Unearth: "leave BF → exile instead" replacement | (same) | S8 |  |
| ATOM-702.88a-001 | 702.88a | Rebound: exile instead of GY, free cast next upkeep | NEW — Rebound | S8 |  |
| ATOM-702.89a-001 | 702.89a | Umbra armor: replace destruction of enchanted permanent | NEW — Umbra Armor | S8 |  |
| ATOM-702.89a-002 | 702.89a | Umbra armor prevents destruction from lethal damage | (same) | S8 |  |
| ATOM-702.104a-001 | 702.104a | Tribute N: opponent chooses counters on ETB | NEW — Tribute | S8 | LINKED-ABILITY-PATTERN |
| ATOM-702.136a-001 | 702.136a | Riot: choose +1/+1 counter or haste on ETB | NEW — Riot | S8 |  |
| ATOM-702.136a-002 | 702.136a | Riot: choosing counter path | (same) | S8 |  |
| ATOM-702.136b-001 | 702.136b | Multiple riot: each gives separate choice | (same) | S8 |  |
| ATOM-702.138c-001 | 702.138c | Escape: enters with additional counters if escaped | (same) | S8 |  |
| ATOM-702.155-002 | 702.155 | Read ahead: counter doubler modifies actual entry | (same) | S8 |  |
| ATOM-702.180a-002 | 702.180a | Harmonize: exile instead of going anywhere on leave stack | (same) | S8 |  |
| ATOM-704.5e-001 | 704.5e | Spell copy not on stack ceases to exist | NEW — SBA spell/card copy | S9a | dependency |
| ATOM-704.5e-002 | 704.5e | Card copy in graveyard (cast-a-copy declined) ceases to exist | NEW — SBA card copy | S9a | dependency |
| ATOM-704.7-001 | 704.7 | SBA coalescing — single replacement for multiple same-result SBAs | NEW — SBA coalescing | S9a | replacement-effects |
| ATOM-707.2-001 | 707.2 | Copy acquires copiable values only (not animation effects) | D5 | S9a | copy |
| ATOM-707.2-003 | 707.2 | Counters on original not copied | D5 | S9a | copy |
| ATOM-707.2b-001 | 707.2b | Changing original after copy doesn't update copy | D5 | S9a | copy |
| BOUNDARY-707.2c-001 | 707.2c | Static ability copy locks copiable values at application time | D5 | S9a | copy |
| ATOM-707.3-001 | 707.3 | Layered copy copiable value propagation | D5 | S9a | copy |
| ATOM-707.4-001 | 707.4 | Re-copy on battlefield — no ETB/LTB, noncopy effects remain | D5 | S9a | copy |
| ATOM-707.5-001 | 707.5 | "Enters as a copy" — copied replacement effects apply | D5 | S9a | copy |
| COMP-9A-002 | 707.5 + 707.2 + 707.6 | Copy entering with ETB replacement + choice reset |  | S9a |  |
| ATOM-707.6-001 | 707.6 | Copy resets "as enters" choices | D5 | S9a | copy |
| ATOM-707.9a-001 | 707.9a | Copy-with-added-ability becomes copiable | D5 | S9a | copy |
| ATOM-707.9b-001 | 707.9b | Copy-with-modified-characteristic becomes copiable | D5 | S9a | copy |
| ATOM-707.9d-001 | 707.9d | P/T override strips CDA (Quicksilver Gargantuan) | D5 | S9a | copy, cda |
| ATOM-707.9d-002 | 707.9d | "In addition to types" preserves type CDA | D5 | S9a | copy, cda |
| ATOM-707.9e-001 | 707.9e | Additional-effect exception lost on re-copy | D5 | S9a | copy |
| ATOM-707.9f-001 | 707.9f | Conditional exception negative — land not creature | D5 | S9a | copy |
| ATOM-707.9f-002 | 707.9f | Conditional exception positive — creature gets counters+changeling | D5 | S9a | copy |
| COMP-9A-006 | 707.9d + 707.2 | CDA stripping vs "in addition to types" preservation |  | S9a |  |
| ATOM-707.10-001 | 707.10 | Spell copy inherits modes/targets/X, not "cast" | D5 | S9a | copy |
| ATOM-707.10-002 | 707.10 | Spell copy references sacrificed object (Fling) | D5 | S9a | copy |
| ATOM-707.10-003 | 707.10 | Spell copy can't reference mana spent (Dawnglow Infusion) | D5 | S9a | copy |
| ATOM-707.10c-001 | 707.10c | Retarget copy — illegal targets may remain | D5 | S9a | copy |
| ATOM-707.10d-001 | 707.10d | Copy for each legal target — one per target | D5 | S9a | copy |
| ATOM-707.10e-001 | 707.10e | Copy with specified single target | D5 | S9a | copy |
| ATOM-707.10e-002 | 707.10e | Replacement causes multiple targets — controller picks one | D5 | S9a | copy |
| ATOM-707.12-001 | 707.12 | "Cast a copy" follows 601.2a–h, triggers fire | D5 | S9a | copy |
| ATOM-727.1a-001 | 727.1a | Radiation life loss tagging for replacement effects | NEW — Radiation tagging | S9b |  |

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

---

## Phase 8

**547 entries**

| ID | Rule | Summary | Ticket | Session | Tags |
|----|------|---------|--------|---------|------|
| ATOM-104.2b-001 | 104.2b | "You win the game" effect | NEW-CH1-002 | S1 |  |
| ATOM-104.3a-001 | 104.3a | Player concedes → leaves game | NEW-CH1-003 | S1 |  |
| ATOM-104.3e-001 | 104.3e | "You lose the game" effect | NEW-CH1-002 | S1 |  |
| ATOM-104.3f-001 | 104.3f | Simultaneous win+lose → player loses | NEW-CH1-004 | S1 |  |
| ATOM-104.4c-001 | 104.4c | "The game is a draw" effect | NEW-CH1-006 | S1 |  |
| ATOM-105.4-001 | 105.4 | "Choose a color" validates 5 colors only | NEW-CH1-007 | S1 |  |
| ATOM-106.5-001 | 106.5 | Producing mana of undefined type produces nothing | NEW-CH1-008 | S1 |  |
| ATOM-106.5-002 | 106.5 | Producing "one mana of any color" → player chooses | NEW-CH1-008 | S1 |  |
| ATOM-106.7-001 | 106.7 | "Could produce" mana query | NEW-CH1-010 | S1 |  |
| ATOM-106.8-001 | 106.8 | Hybrid mana production (choose half) | NEW-CH1-011 | S1 |  |
| ATOM-106.9-001 | 106.9 | Phyrexian mana production produces colored | NEW-CH1-012 | S1 |  |
| ATOM-106.10-001 | 106.10 | Generic mana production (player chooses type) | NEW-CH1-012 | S1 |  |
| ATOM-106.11-001 | 106.11 | Snow mana production tags source as snow | NEW-CH1-012 | S1 |  |
| ATOM-107.1b-004 | 107.1b | Doubling effect on negative value → double the negative | Phase 8 | S1 |  |
| ATOM-107.3h-001 | 107.3h | X=0 for non-stack mana cost payments | NEW-CH1-015 | S1 |  |
| ATOM-107.4h-001 | 107.4h | Snow {S} paid with snow-sourced mana | NEW-CH1-018 | S1 |  |
| ATOM-107.4h-003 | 107.4h | Generic reduction doesn't reduce pure {S} cost | NEW-CH1-018 | S1 |  |
| ATOM-107.7-001 | 107.7 | [+N] adds N loyalty counters | T14 + Phase 8 | S1 |  |
| ATOM-107.7-002 | 107.7 | [-N] removes N loyalty counters | T14 + Phase 8 | S1 |  |
| ATOM-107.7-003 | 107.7 | [0] loyalty ability costs 0 loyalty | T14 + Phase 8 | S1 |  |
| ATOM-107.14-001 | 107.14 | {E} energy counter payment | NEW-CH1-019 | S1 |  |
| ATOM-108.4a-001 | 108.4a | Controller fallback to owner | NEW-CH1-020 | S1 |  |
| ATOM-109.1-001 | 109.1 | All 7 object types representable | NEW-CH1-021 | S1 | boundary |
| ATOM-109.4c-001 | 109.4c | Emblem controller tracking | Phase 8 | S1 |  |
| ATOM-110.2a-001 | 110.2a | ETB controller from effect | NEW-CH1-022 | S1 |  |
| ATOM-110.2a-002 | 110.2a | ETB under opponent's control | NEW-CH1-022 | S1 |  |
| ATOM-111.2-001 | 111.2 | Token creator is owner+controller | Phase 8 | S1 |  |
| ATOM-111.3-001 | 111.3 | Token characteristics = only what effect specifies | Phase 8 | S1 |  |
| ATOM-111.4-001 | 111.4 | Token name defaults to subtype(s) + "Token" | Phase 8 | S1 |  |
| ATOM-111.4-002 | 111.4 | Named token uses specified name | Phase 8 | S1 |  |
| ATOM-111.5-001 | 111.5 | Token copy of instant/sorcery not created | Phase 8 | S1 |  |
| ATOM-111.5-002 | 111.5 | Token not created if ETB prevention active | Phase 8 | S1 |  |
| ATOM-111.8-001 | 111.8 | Token that left battlefield can't change zones | NEW-CH1-023 | S1 |  |
| ATOM-111.10-001 | 111.10 | Predefined Treasure token characteristics | Phase 8 | S1 | boundary |
| ATOM-111.10-002 | 111.10 | Predefined Food token characteristics | Phase 8 | S1 | boundary |
| ATOM-111.11-001 | 111.11 | Non-predefined named token uses Oracle card | Phase 8 | S1 |  |
| ATOM-111.12-001 | 111.12 | Copy of nonexistent object → no token | Phase 8 | S1 |  |
| ATOM-113.5-001 | 113.5 | Loyalty ability: sorcery speed, once/turn/permanent | Phase 8 | S1 |  |
| ATOM-113.11-001 | 113.11 | "Can't have" ability prevents gaining + removes | D10 | S1 |  |
| ATOM-114.2-001 | 114.2 | Emblem owned+controlled by receiver, in command zone | NEW-CH1-024 | S1 |  |
| ATOM-114.3-001 | 114.3 | Emblem has no types/mana cost/color | Phase 8 | S1 | boundary |
| ATOM-114.5-001 | 114.5 | Emblem is not card/permanent/card type | Phase 8 | S1 | boundary |
| ATOM-115.1b-001 | 115.1b | Aura spell targets; Aura permanent does not | Phase 8 | S1 |  |
| ATOM-115.8-001 | 115.8 | Target-changing effect (Spellskite) | NEW — Target-changing | S1 |  |
| ATOM-116.2c-001 | 116.2c | End continuous effect as special action | NEW — End effect SA | S1 |  |
| ATOM-116.2d-001 | 116.2d | Pay to ignore restriction (Leonin Arbiter) | NEW — Ignore restriction SA | S1 |  |
| ATOM-116.2m-001 | 116.2m | Room enchantment unlock special action | NEW — Room unlock SA | S1 |  |
| ATOM-117.1d-002 | 117.1d | Mana ability during resolution payment (Mana Leak) | NEW — Resolution mana window | S1 |  |
| ATOM-118.6a-001 | 118.6a | Alt cost replaces unpayable cost | Phase 8 | S1 |  |
| ATOM-118.7e-001 | 118.7e | Hybrid reduction symbol: player chooses half | NEW-CH1-025 | S1 |  |
| ATOM-118.7e-002 | 118.7e | Two-brid {2/W} reduction: choose {2} or {W} | NEW-CH1-025 | S1 |  |
| ATOM-118.7f-001 | 118.7f | Phyrexian reduction → one colored mana | NEW-CH1-026 | S1 |  |
| ATOM-118.7g-001 | 118.7g | Snow reduction → generic | NEW-CH1-027 | S1 |  |
| ATOM-118.12-002 | 118.12 | "If you do" succeeds if action taken despite altered outcome | Phase 8 | S1 |  |
| ATOM-118.14-001 | 118.14 | "Mana of any type" allows any mana for any color | NEW-CH1-028 | S1 |  |
| ATOM-118.14-002 | 118.14 | "Mana of any type" doesn't override spending restrictions | Phase 8 | S1 |  |
| ATOM-119.4b-001 | 119.4b | Paying 0 life always legal | NEW-CH1-029 | S1 |  |
| ATOM-119.5-001 | 119.5 | SetLife → gain/lose difference | NEW-CH1-030 | S1 |  |
| ATOM-119.5-002 | 119.5 | SetLife higher → life gain | NEW-CH1-030 | S1 |  |
| ATOM-119.7-001 | 119.7 | "Can't gain life" blocks exchange | NEW-CH1-031 | S1 |  |
| ATOM-119.7-002 | 119.7 | "Can't gain life" blocks redistribution | Phase 8 | S1 |  |
| ATOM-119.7-003 | 119.7 | "Can't gain life" blocks alt cost requiring opponent life gain | Phase 8 | S1 |  |
| ATOM-119.7-004 | 119.7 | "Can't gain life" prevents replacement that would produce gain | Phase 6 + Phase 8 | S1 |  |
| ATOM-119.8-001 | 119.8 | "Can't lose life" blocks exchange | NEW-CH1-032 | S1 |  |
| ATOM-120.3b-001 | 120.3b | Infect damage → poison counters | Phase 8 | S1 |  |
| ATOM-120.3c-001 | 120.3c | Damage to PW removes loyalty | T14 + Phase 8 | S1 |  |
| ATOM-120.3d-001 | 120.3d | Wither/infect damage to creature → -1/-1 counters | Phase 8 | S1 |  |
| ATOM-120.3g-001 | 120.3g | Toxic combat damage → additional poison | Phase 8 | S1 |  |
| ATOM-121.2b-001 | 121.2b | "Can't draw more than one/turn" partially carries out | NEW-CH1-033 | S1 |  |
| ATOM-121.2c-001 | 121.2c | APNAP draw ordering (active first) | NEW-CH1-034 | S1 |  |
| ATOM-121.3-001 | 121.3 | Optional draw from empty library permitted | Phase 8 | S1 |  |
| ATOM-121.3-002 | 121.3 | "Can't draw" prevents optional draw choice | Phase 8 | S1 |  |
| ATOM-122.1i-001 | 122.1i | Rad counter mill + life loss trigger | Phase 8 | S1 |  |
| ATOM-122.4-001 | 122.4 | Counter cap SBA removes excess | Phase 8 | S1 |  |
| ATOM-122.5-001 | 122.5 | Move counter: remove from A, put on B | Phase 8 | S1 |  |
| ATOM-122.5-002 | 122.5 | Can't move counter to same object | Phase 8 | S1 |  |
| ATOM-122.5-003 | 122.5 | Can't move counter source doesn't have | Phase 8 | S1 |  |
| ATOM-122.5-004 | 122.5 | Can't move counter if dest can't receive | Phase 8 | S1 |  |
| ATOM-122.5-005 | 122.5 | Can't move counter if object left zone | Phase 8 | S1 |  |
| ATOM-201.2b-002 | 201.2b | "Different names" rejects duplicates in group | NEW-S2-03 | S2 |  |
| ATOM-201.2c-002 | 201.2c | Singular "different name" group — shared name fails | NEW-S2-04 | S2 |  |
| ATOM-202.1b-002 | 202.1b | Token default mana cost = None | NEW-S2-05 | S2 |  |
| ATOM-205.3e-001 | 205.3e | Choose subtype validation by card type | NEW-S2-08 | S2 |  |
| ATOM-205.3j-001 | 205.3j | Planeswalker types set membership | NEW-S2-09 | S2 | boundary |
| ATOM-205.3k-001 | 205.3k | Spell types set membership | NEW-S2-10 | S2 | boundary |
| ATOM-205.3p-001 | 205.3p | Dungeon types set membership | NEW-S2-11 | S2 | boundary |
| ATOM-205.3q-001 | 205.3q | Battle types (Siege) set membership | NEW-S2-12 | S2 | boundary |
| ATOM-205.4f-001 | 205.4f | World supertype → world rule SBA | NEW-S2-13 | S2 |  |
| ATOM-205.4g-001 | 205.4g | Snow supertype = snow permanent classification | NEW-S2-14 | S2 | boundary |
| ATOM-209.2-001 | 209.2 | Loyalty ability sorcery-speed activation | T19, NEW-S2-18 | S2 |  |
| ATOM-210.1-001 | 210.1 | Battle enters with defense counters = printed defense | NEW-S2-19 | S2 |  |
| ATOM-300.2a-001 | 300.2a | Land+other type: cast rejected, play-as-land only | D25 | S3 | land, casting |
| ATOM-300.2a-002 | 300.2a | Land+other type CAN be played as land | D25 | S3 | land, positive |
| ATOM-301.5b-002 | 301.5b | Equip ability attaches Equipment to creature | NEW | S3 | equipment, equip |
| ATOM-301.5b-003 | 301.5b | Effect attaching Equipment to illegal target: no move | NEW | S3 | equipment, guard |
| ATOM-301.5c-001 | 301.5c | Creature-Equipment can't equip (no reconfigure) | NEW | S3 | equipment, restriction |
| ATOM-301.5c-003 | 301.5c | Equipment can't equip itself | NEW | S3 | equipment, restriction |
| ATOM-301.5c-005 | 301.5c | Equipment can't equip >1 creature; equip moves it | NEW | S3 | equipment, move |
| ATOM-301.5d-002 | 301.5d | Only Equipment controller activates its abilities | NEW | S3 | equipment, activation |
| ATOM-301.5d-003 | 301.5d | Equipment-granted ability activated by creature's controller | NEW | S3 | equipment, granted-ability |
| ATOM-301.5e-001 | 301.5e | Equipment ETB via effect attached to illegal target → unattached | NEW | S3 | equipment, etb, guard |
| ATOM-301.6-001 | 301.6 | Fortification attaches to land, not non-land | NEW | S3 | fortification, attachment |
| ATOM-301.7a-001 | 301.7a | Vehicle (non-creature) has no P/T characteristics | NEW | S3 | vehicle, characteristic |
| ATOM-301.7a-002 | 301.7a | Vehicle (creature) has printed P/T | NEW | S3 | vehicle, characteristic |
| ATOM-301.7b-001 | 301.7b | Vehicle becoming creature gets P/T + anthem applies | NEW | S3 | vehicle, layers |
| ATOM-303.4-002 | 303.4 | Aura with "Enchant player" attaches to player | NEW | S3 | aura, player-attach |
| ATOM-303.4d-002 | 303.4d | Aura+Creature → unattach then graveyard SBA | T15 | S3 | aura, sba, type-change |
| ATOM-303.4e-002 | 303.4e | Aura-granted ability activated by enchanted object's controller | NEW | S3 | aura, granted-ability |
| ATOM-303.4g-001 | 303.4g | Aura from non-stack, no legal target → stays in zone | T15b | S3 | aura, guard |
| ATOM-303.4g-003 | 303.4g | Aura token, no legal target → not created | NEW | S3 | aura, token, guard |
| ATOM-303.4h-001 | 303.4h | Non-attachment permanent entering "attached" → unattached | NEW | S3 | attachment, guard |
| ATOM-303.4i-001 | 303.4i | Aura from non-stack attached to illegal host → stays in zone | NEW | S3 | aura, guard |
| ATOM-303.4i-002 | 303.4i | Aura from stack attached to illegal host → graveyard | NEW | S3 | aura, guard |
| ATOM-303.4i-003 | 303.4i | Aura token attached to illegal host → not created | NEW | S3 | aura, token, guard |
| ATOM-303.4j-001 | 303.4j | Move Aura to illegal enchant target → doesn't move | NEW | S3 | aura, re-attach, guard |
| ATOM-303.7a-001 | 303.7a | Multiple Roles from same controller: keep newest (SBA) | NEW | S3 | role, sba |
| ATOM-303.7a-002 | 303.7a | Roles from different controllers coexist | NEW | S3 | role, sba, positive |
| ATOM-304.5-002 | 304.5 | "Can't cast instants" doesn't block "as an instant" abilities | NEW | S3 | timing, restriction |
| ATOM-305.2b-001 | 305.2b | At limit, effect instruction to play land is ignored | T22, NEW | S3 | land, hard-cap |
| ATOM-305.4-001 | 305.4 | "Put land onto BF" ≠ "play"; counter not incremented | NEW | S3 | land, put-vs-play |
| ATOM-306.6-001 | 306.6 | Planeswalker can be attacked | NEW | S3 | planeswalker, combat |
| ATOM-307.5-003 | 307.5 | "Can't cast sorceries" doesn't block "as a sorcery" abilities | NEW | S3 | timing, restriction |
| ATOM-307.5-004 | 307.5 | Teferi-style: opponent's instant cast legal at sorcery speed | NEW | S3 | timing, teferi, positive |
| ATOM-307.5-005 | 307.5 | Teferi-style: opponent can't cast at instant speed | NEW | S3 | timing, teferi, restriction |
| ATOM-307.5a-001 | 307.5a | "Couldn't cast as sorcery" = retroactive timing check | NEW | S3 | timing, necromancy |
| ATOM-308.1-001 | 308.1 | Kindred card follows other type's casting rules | NEW | S3 | kindred, casting |
| ATOM-400.7-002 | 400.7 | Blink creates new object with no memory | NEW — blink identity reset | S4 |  |
| ATOM-400.7j-001 | 400.7j | Effect moves object to public zone, finds it | NEW — effect self-referencing after zone change | S4 |  |
| ATOM-400.8-001 | 400.8 | Re-exile creates new object identity (epoch bump) | NEW — re-exile epoch bump | S4 |  |
| ATOM-400.12-001 | 400.12 | Batch zone-to-zone move ("shuffle GY into library") | NEW — batch zone move | S4 |  |
| ATOM-401.4-001 | 401.4 | Multi-card library placement ordering via DP | NEW — multi-card library ordering | S4 |  |
| ATOM-401.5-001 | 401.5 | "Play with top card revealed" continuous effect | NEW — top-card-revealed effect | S4 |  |
| ATOM-401.5-002 | 401.5 | Top-of-library reveal freeze during casting | NEW — reveal freeze during casting | S4 |  |
| ATOM-401.5-003 | 401.5 | Reveal freeze during ability activation | Same as 401.5-002 | S4 |  |
| ATOM-401.7-001 | 401.7 | "Nth from top" fallback to bottom | NEW — library positional insertion | S4 |  |
| ATOM-402.3-002 | 402.3 | "Reveal" = all players see (Thoughtseize) | NEW — reveal keyword implementation | S4 |  |
| ATOM-402.3-003 | 402.3 | "Look at" = controller only (Gitaxian Probe) | NEW — look-at keyword implementation | S4 |  |
| ATOM-402.3-004 | 402.3 | Persistent "play with hand revealed" (Telepathy) | NEW — persistent reveal effect | S4 |  |
| ATOM-404.3-001 | 404.3 | Simultaneous graveyard ordering (auto-order) | NEW — simultaneous graveyard ordering | S4 |  |
| ATOM-404.3-002 | 404.3 | Simultaneous graveyard ordering (manual via DP) | Same as 404.3-001 | S4 |  |
| ATOM-405.4-005 | 405.4 | "Any player may activate" controller = activator | NEW — any-player-activate controller | S4 |  |
| ATOM-406.3-001 | 406.3 | Exiled cards face-up by default | NEW — exile zone visibility | S4 |  |
| ATOM-406.3-002 | 406.3 | Face-down exile hidden from all | NEW — face-down exile tracking | S4 |  |
| ATOM-408.2-001 | 408.2 | Emblems created in command zone | NEW — emblem creation | S4 |  |
| ATOM-502.3-003 | 502.3 | Constrained untap choice (Winter Orb pattern) | NEW — constrained untap via DP | S4 |  |
| ATOM-506.3a-001 | 506.3a | Noncreature ETB-attacking enters but not attacking | NEW — noncreature enters-attacking guard | S4 |  |
| ATOM-506.3b-001 | 506.3b | Non-attacking-player ETB-attacking enters but not attacking | NEW — ETB-attacking controller validation | S4 |  |
| ATOM-506.3c-001 | 506.3c | Invalid target ETB-attacking enters but not attacking | NEW — ETB-attacking target validation | S4 |  |
| ATOM-506.3d-001 | 506.3d | Late ETB-attacking during declare blockers — unblocked | NEW — late ETB-attacking unblocked | S4 |  |
| ATOM-506.3d-002 | 506.3d | Late ETB-attacking during combat damage — unblocked | Same as 506.3d-001 | S4 |  |
| ATOM-506.3d-003 | 506.3d | Late ETB-attacking during end of combat — unblocked | Same as 506.3d-001 | S4 |  |
| ATOM-506.4-005 | 506.4 | Explicit remove-from-combat effect (Maze of Ith) | T21b | S4 |  |
| ATOM-506.4c-001 | 506.4c | Orphaned attacker (PW removed) — no damage if unblocked | NEW — orphaned attacker no damage | S4 |  |
| ATOM-506.6-001 | 506.6 | "Had to attack" check (goad) | T21d | S4 |  |
| ATOM-508.1b-001 | 508.1b | PW/battle attack target selection | NEW — per-attacker target selection | S4 |  |
| ATOM-508.1g-001 | 508.1g | Optional attack cost framework | NEW — optional attack costs | S4 |  |
| ATOM-508.1h-001 | 508.1h | Attack cost lock-in mechanism | NEW — attack cost lock-in | S4 |  |
| ATOM-508.4-001 | 508.4 | ETB-attacking never "attacked" for triggers | NEW — ETB-attacking trigger suppression | S4 |  |
| ATOM-509.1d-001 | 509.1d | Blocking cost determination and lock-in | NEW — blocking cost lock-in | S4 |  |
| ATOM-509.4-001 | 509.4 | ETB-blocking never "blocked" for triggers | NEW — ETB-blocking trigger suppression | S4 |  |
| ATOM-509.4a-001 | 509.4a | ETB-blocking invalid target — enters but not blocking | NEW — ETB-blocking target validation | S4 |  |
| ATOM-509.4b-001 | 509.4b | ETB-blocking ignores evasion/restrictions | NEW — ETB-blocking evasion bypass | S4 |  |
| ATOM-510.1b-002 | 510.1b | Orphaned attacker assigns no damage | NEW — orphaned attacker no damage | S4 |  |
| ATOM-510.1d-001 | 510.1d | Blocker blocking multiple attackers: free damage division | NEW — multi-block damage division | S4 |  |
| ATOM-601.1a-001 | 601.1a | "Play a card" routes to land play for lands | NEW-1 | S5 |  |
| ATOM-601.1a-002 | 601.1a | "Play a card" routes to cast for non-lands | NEW-1 | S5 |  |
| ATOM-601.1a-001/002 | 601.1a | "Play a card" routes to land play or cast | NEW-1 | S5 |  |
| ATOM-601.2a-003 | 601.2a | move_to_stack works from zones other than Hand | T18 | S5 |  |
| ATOM-601.2c-005 | 601.2c | Target-forcing effects maximized | NEW-3 | S5 |  |
| ATOM-601.3a-001 | 601.3a | Cast-time look-ahead past prohibition | D17 | S5 |  |
| ATOM-601.3b-001 | 601.3b | Flash grant look-ahead | D17 | S5 |  |
| ATOM-601.3c-001 | 601.3c | Conditional flash grant from alt/add cost | D17 | S5 |  |
| ATOM-601.3c-002 | 601.3c | Alternative cost flash grant | D17 | S5 |  |
| ATOM-601.3d-001 | 601.3d | Conditional flash | D17 | S5 |  |
| ATOM-601.3f-001 | 601.3f | Face-down exile cast permission requires visibility | D21 | S5 |  |
| ATOM-601.3f-002 | 601.3f | Face-down exile cast denied + info leak prevention | D21 | S5 |  |
| ATOM-601.5a-001 | 601.5a | Flash condition persistence after casting begins | D17 | S5 |  |
| ATOM-601.6-001 | 601.6 | Opponent-directed choice during casting | NEW-4 | S5 |  |
| ATOM-601.6b-001 | 601.6b | Ordering exception for simultaneous cast-time actions | NEW-5 | S5 |  |
| ATOM-602.2a-002 | 602.2a | Reveal on activation from hidden zone | T19 | S5 |  |
| ATOM-602.5c-001 | 602.5c | Per-acquisition restriction scoping | Phase 8 | S5 |  |
| ATOM-605.1a-003 | 605.1a | Loyalty ability disqualifies mana ability | Phase 8 | S5 |  |
| ATOM-606.2-001 | 606.2 | Loyalty ability classification | Phase 8 | S5 | planeswalker |
| ATOM-606.3-001 | 606.3 | Loyalty ability timing + once-per-turn | Phase 8 | S5 | planeswalker |
| ATOM-606.3-002 | 606.3 | Loyalty ability sorcery-speed enforcement | Phase 8 | S5 | planeswalker |
| ATOM-606.4-001 | 606.4 | Loyalty counter cost payment | Phase 8 | S5 | planeswalker |
| ATOM-606.5-001 | 606.5 | Loyalty cost combination | Phase 8 | S5 | planeswalker |
| ATOM-606.6-001 | 606.6 | Loyalty counter floor check | Phase 8 | S5 | planeswalker |
| ATOM-607.1d-001 | 607.1d | Cross-object linked abilities | Phase 8 | S5 |  |
| ATOM-607.2e-001 | 607.2e | Noted-information linking | Phase 8 | S5 |  |
| ATOM-607.2f-001 | 607.2f | Word-choice linking | Phase 8 | S5 |  |
| ATOM-607.3-001 | 607.3 | Multi-exile linked ability resolution | Phase 8 | S5 |  |
| ATOM-607.5-001 | 607.5 | Acquired-pair isolation | Phase 8 | S5 |  |
| ATOM-607.5a-001 | 607.5a | Undefined choice from broken link | Phase 8 | S5 |  |
| ATOM-608.2g-001 | 608.2g | Cast-during-resolution (cascade-like) | Phase 8 | S5 |  |
| ATOM-608.2m-001 | 608.2m | Resolution continuation after stack departure | Phase 8 | S5 | low-priority |
| ATOM-608.3f-001 | 608.3f | Spell-copy → token (not "created") | Phase 8 | S5 |  |
| ATOM-608.3g-001 | 608.3g | Stack static → delayed trigger on ETB (Dash/Blitz) | Phase 8 | S5 |  |
| ATOM-609.3-001 | 609.3 | Impossible partial effect: discard more than hand size | NEW-609.3 | S6 |  |
| ATOM-609.3-002 | 609.3 | Impossible partial effect: mill with insufficient library | NEW-609.3 | S6 |  |
| ATOM-609.4-001 | 609.4 | "As though flash" scoped to stated effect only | NEW-609.4 | S6 |  |
| ATOM-609.4a-001 | 609.4a | Two "as though" effects combine | NEW-609.4 | S6 |  |
| ATOM-609.4b-001 | 609.4b | "Mana of any type" doesn't change actual mana identity | NEW-609.4 | S6 |  |
| ATOM-609.4b-002 | 609.4b | "Any color" vs "any type" distinction ({C} can't pay colored) | NEW-609.4 | S6 |  |
| ATOM-609.4b-003 | 609.4b | Mana restrictions persist through "as though" spending | NEW-609.4 | S6 |  |
| ATOM-611.2b-001 | 611.2b | "For as long as" duration never starts → no effect | NEW-611.2b | S6 |  |
| ATOM-613.2-001 | 613.2 | Layer 1 sublayer ordering: 1a before 1b | NEW-613.2 | S6 |  |
| ATOM-700.2d-002 | 700.2d | Repeated mode when allowed — effects applied N times in order | NEW | S7a |  |
| ATOM-700.2i-001 | 700.2i | Pawprint budget mode selection | NEW | S7a |  |
| ATOM-700.3a-001 | 700.3a | Pile formation — each object in exactly one pile | NEW | S7a |  |
| ATOM-700.3c-001 | 700.3c | Pile formation — no zone-change events during forming | NEW | S7a |  |
| ATOM-700.5-001 | 700.5 | Devotion to single color — mana symbol count | NEW | S7a |  |
| ATOM-700.5-002 | 700.5 | Devotion to two colors — hybrid counts once | NEW | S7a |  |
| BOUNDARY-DEF-700.6-001 | 700.6 | Historic = legendary OR artifact OR Saga | NEW | S7a |  |
| BOUNDARY-DEF-700.9-001 | 700.9 | Modified = counters OR equipped OR friendly Aura | NEW | S7a |  |
| BOUNDARY-DEF-700.12-001 | 700.12 | Outlaw = Assassin/Mercenary/Pirate/Rogue/Warlock | NEW | S7a |  |
| ATOM-701.7a-001 | 701.7a | Create tokens — specified characteristics on battlefield | NEW | S7a |  |
| ATOM-701.7b-001 | 701.7b | Token creation replacement ordering (creation → continuous → ETB) | NEW | S7a | dependency, replacement-effects, layers |
| ATOM-701.9b-001 | 701.9b | Random discard — engine picks at random | NEW | S7a |  |
| ATOM-701.9b-002 | 701.9b | Opponent-chosen discard | NEW | S7a |  |
| ATOM-701.9c-001 | 701.9c | Discard to hidden zone — characteristic undefined | NEW | S7a | dependency, replacement-effects |
| ATOM-701.10d-001 | 701.10d | Double life total — via gain/loss event | NEW | S7a |  |
| ATOM-701.10d-002 | 701.10d | Double negative life total — via loss event | NEW | S7a |  |
| ATOM-701.10e-001 | 701.10e | Double counters — add same count | NEW | S7a |  |
| ATOM-701.10f-001 | 701.10f | Double mana — new mana is unrestricted | NEW | S7a |  |
| ATOM-701.12a-001 | 701.12a | Exchange all-or-nothing — partial failure cancels all | NEW | S7a |  |
| ATOM-701.12b-001 | 701.12b | Control exchange — simultaneous swap | NEW | S7a |  |
| ATOM-701.12b-002 | 701.12b | Same-controller exchange — no-op | NEW | S7a |  |
| ATOM-701.12c-001 | 701.12c | Life total exchange — via gain/loss events | NEW | S7a |  |
| ATOM-701.12c-002 | 701.12c | Life exchange blocked by can't-gain-life (119.7) | NEW | S7a | dependency, replacement-effects |
| ATOM-701.12d-001 | 701.12d | Mass zone exchange (hand ↔ graveyard) | NEW | S7a |  |
| ATOM-701.12g-001 | 701.12g | Numerical value exchange (life ↔ power) | NEW | S7a |  |
| ATOM-701.14a-001 | 701.14a | Fight — mutual simultaneous damage | NEW | S7a |  |
| ATOM-701.14b-001 | 701.14b | Fight — creature left battlefield, neither fights | NEW | S7a |  |
| ATOM-701.14b-002 | 701.14b | Fight — illegal target, neither fights | NEW | S7a |  |
| ATOM-701.14c-001 | 701.14c | Self-fight — double power self-damage | NEW | S7a |  |
| ATOM-701.14d-001 | 701.14d | Fight damage is NOT combat damage | NEW | S7a |  |
| COMP-7A-002 | 701.14a + 701.14d + 702.15 | Fight + lifelink + non-combat damage flag |  | S7a |  |
| ATOM-701.16a-001 | 701.16a | Investigate — create Clue token | NEW | S7a |  |
| ATOM-701.17a-001 | 701.17a | Mill N — top N to graveyard | NEW | S7a |  |
| ATOM-701.17b-001 | 701.17b | Mill capped to library size | NEW | S7a |  |
| ATOM-701.17b-002 | 701.17b | Mill as cost — unpayable if library too small | NEW | S7a |  |
| ATOM-701.20a-001 | 701.20a | Reveal — mark cards visible to all players | NEW | S7a |  |
| ATOM-701.22a-001 | 701.22a | Scry N — top/bottom reorder | NEW | S7a |  |
| ATOM-701.22b-001 | 701.22b | Scry 0 — no event, no trigger | NEW | S7a |  |
| ATOM-701.23a-001 | 701.23a | Search zone — find matching card | NEW | S7a |  |
| ATOM-701.23b-001 | 701.23b | Fail-to-find in hidden zone — optional | NEW | S7a |  |
| ATOM-701.23d-001 | 701.23d | Mandatory quantity search — must find N | NEW | S7a |  |
| ATOM-701.24b-001 | 701.24b | Search-then-shuffle — found card excluded | NEW | S7a |  |
| ATOM-701.25a-001 | 701.25a | Surveil N — top cards to GY or top | NEW | S7a |  |
| ATOM-701.25c-001 | 701.25c | Surveil 0 — no event, no trigger | NEW | S7a |  |
| ATOM-701.29a-001 | 701.29a | Fateseal — scry on opponent's library | NEW | S7a |  |
| ATOM-701.34a-001 | 701.34a | Proliferate — +1 of each counter kind | NEW | S7a |  |
| COMP-7A-003 | 701.34a + 701.46a | Proliferate + adapt guard interaction |  | S7a |  |
| ATOM-701.35a-001 | 701.35a | Detain — can't attack/block/activate until next turn | NEW | S7a |  |
| ATOM-701.36a-001 | 701.36a | Populate — copy chosen creature token | NEW | S7a |  |
| ATOM-701.36b-001 | 701.36b | Populate with no creature tokens — no-op | NEW | S7a |  |
| ATOM-701.37a-001 | 701.37a | Monstrosity N — counters + designation | NEW | S7a |  |
| ATOM-701.37a-002 | 701.37a | Monstrosity guard — already monstrous = no-op | NEW | S7a |  |
| ATOM-701.39a-001 | 701.39a | Bolster N — counters on lowest-toughness creature | NEW | S7a |  |
| ATOM-701.39a-002 | 701.39a | Bolster tie-break — controller chooses | NEW | S7a |  |
| ATOM-701.41a-001 | 701.41a | Support N on permanent — +1/+1 on up to N OTHER creatures | NEW | S7a |  |
| ATOM-701.41a-002 | 701.41a | Support N on instant/sorcery — no "other" restriction | NEW | S7a |  |
| ATOM-701.44a-001 | 701.44a | Explore — land path (to hand) | NEW | S7a |  |
| ATOM-701.44a-002 | 701.44a | Explore — nonland path (counter + GY) | NEW | S7a |  |
| ATOM-701.44a-003 | 701.44a | Explore — nonland, keep on top | NEW | S7a |  |
| ATOM-701.44c-001 | 701.44c | Explore LKI — permanent left battlefield | NEW | S7a |  |
| ATOM-701.46a-001 | 701.46a | Adapt N — counters if none present | NEW | S7a |  |
| ATOM-701.46a-002 | 701.46a | Adapt guard — has counters = no-op | NEW | S7a |  |
| ATOM-701.47a-001 | 701.47a | Amass — no Army → create token + counters + subtype | NEW | S7a |  |
| ATOM-701.47a-002 | 701.47a | Amass — existing Army gets counters | NEW | S7a |  |
| ATOM-701.47a-003 | 701.47a | Amass — grants new subtype to existing Army | NEW | S7a |  |
| ATOM-701.50a-001 | 701.50a | Connive — draw/discard, nonland → +1/+1 counter | NEW | S7a |  |
| ATOM-701.50a-002 | 701.50a | Connive — discard land → no counter | NEW | S7a |  |
| ATOM-701.50e-001 | 701.50e | Connive N — draw N/discard N, counters = nonland count | NEW | S7a |  |
| ATOM-701.54a-001 | 701.54a | Ring tempts — emblem + ring-bearer designation | NEW | S7a |  |
| ATOM-701.54a-002 | 701.54a | Ring tempts again — redesignate + progressive unlock | NEW | S7a |  |
| ATOM-701.54c-001 | 701.54c | Ring — all 4 progressive abilities at temptation 4 | NEW | S7a |  |
| ATOM-701.55a-001 | 701.55a | Villainous choice — pick A or B | NEW | S7a |  |
| ATOM-701.55b-001 | 701.55b | Villainous choice — impossible option, perform as much as possible | NEW | S7a |  |
| ATOM-701.57a-001 | 701.57a | Discover N — exile loop + free cast | NEW | S7a |  |
| ATOM-701.57a-002 | 701.57a | Discover — choose hand instead of cast | NEW | S7a |  |
| ATOM-701.57a-003 | 701.57a | Discover — whiff (all lands) | NEW | S7a |  |
| ATOM-701.59a-001 | 701.59a | Collect evidence N — exile GY cards with MV ≥ N | NEW | S7a |  |
| ATOM-701.59b-001 | 701.59b | Collect evidence — can't choose if GY MV insufficient | NEW | S7a |  |
| ATOM-701.60a-001 | 701.60a | Suspect — gains suspected designation | NEW | S7a |  |
| ATOM-701.60c-001 | 701.60c | Suspected grants menace + can't block | NEW | S7a |  |
| ATOM-701.63a-001 | 701.63a | Endure N — counters on self path | NEW | S7a |  |
| ATOM-701.63a-002 | 701.63a | Endure N — Spirit token path | NEW | S7a |  |
| ATOM-701.63b-001 | 701.63b | Endure 0 — no-op | NEW | S7a |  |
| ATOM-701.64a-001 | 701.64a | Harness — gains harnessed designation | NEW | S7a |  |
| ATOM-701.64a-002 | 701.64a | Harness guard — already harnessed = no-op | NEW | S7a |  |
| ATOM-701.65a-001 | 701.65a | Airbend — exile + persistent cast-from-exile for {2} | NEW | S7a |  |
| ATOM-701.65a-002 | 701.65a | Airbend — owner casts exiled card for {2} | NEW | S7a |  |
| ATOM-701.65a-003 | 701.65a | Airbend — batch exile of multiple objects | NEW | S7a |  |
| ATOM-701.67a-001 | 701.67a | Waterbend — tap artifacts/creatures for generic portion | NEW | S7a |  |
| ATOM-701.67b-001 | 701.67b | Waterbend — cost isolation (only waterbend's generic) | NEW | S7a |  |
| ATOM-701.68a-001 | 701.68a | Blight N — put N -1/-1 counters on own creature | NEW | S7a |  |
| ATOM-701.68b-001 | 701.68b | Blight legality — must control a creature | NEW | S7a |  |
| COMP-702-001 | 702.2b + 702.7b | Deathtouch + first strike kills before blocker hits back | IMPL | S7b | — |
| COMP-702-002 | 702.2b + 702.19b | Deathtouch + trample: 1 per blocker = lethal, rest overflows | IMPL | S7b | — |
| COMP-702-003 | 702.4b + 702.15b | Double strike + lifelink: life gained in both steps | IMPL | S7b | — |
| ATOM-702.5c-001 | 702.5c | Multiple enchant instances compose via AND | DEFERRED | S7b | enchant, aura, multiple-instances |
| ATOM-702.6c-001 | 702.6c | Equip with quality restriction | DEFERRED | S7b | equip, quality-restriction |
| ATOM-702.6c-002 | 702.6c | Non-equip attachment bypasses quality restriction | DEFERRED | S7b | equip, quality-restriction, non-equip-attachment |
| ATOM-702.6d-001 | 702.6d | Multiple equip abilities coexist independently | DEFERRED | S7b | equip, multiple-abilities, quality-restriction |
| ATOM-702.6e-001 | 702.6e | Equip planeswalker variant | DEFERRED | S7b | equip, planeswalker |
| ATOM-702.8a-002 | 702.8a | Flash from non-hand zone | DEFERRED | S7b | flash, any-zone |
| COMP-702-004 | 702.9b + 702.17b | Flying + reach: reach can block flying | IMPL | S7b | — |
| ATOM-702.11c-001 | 702.11c | Hexproof on player | DEFERRED | S7b | hexproof, player |
| ATOM-702.11d-001 | 702.11d | Hexproof from [quality] variant | DEFERRED | S7b | hexproof, hexproof-from, quality |
| ATOM-702.11e-001 | 702.11e | Losing hexproof strips all hexproof-from | DEFERRED | S7b | hexproof, hexproof-from, ability-removal |
| ATOM-702.13b-001 | 702.13b | Intimidate evasion: artifact or shared color | DEFERRED | S7b | intimidate, evasion, blocking |
| ATOM-702.14c-001 | 702.14c | Landwalk evasion: unblockable if defender has land | DEFERRED | S7b | landwalk, evasion, blocking |
| ATOM-702.14c-002 | 702.14c | Landwalk inapplicable without matching land | DEFERRED | S7b | landwalk, evasion, blocking |
| ATOM-702.14d-001 | 702.14d | Landwalk abilities don't cancel each other | DEFERRED | S7b | landwalk, evasion, no-cancel |
| ATOM-702.16a-002 | 702.16a | Protection from [cardname] quality matching | DEFERRED | S7b | protection, protection-from-cardname |
| ATOM-702.16a-003 | 702.16a | Protection from snow (all source types) | DEFERRED | S7b | protection, protection-from-snow, supertype, snow-spell |
| ATOM-702.16j-001 | 702.16j | Protection from everything | DEFERRED | S7b | protection, protection-from-everything |
| ATOM-702.16n-001 | 702.16n | Self-exception Aura on protection | DEFERRED | S7b | protection, aura, self-exception |
| ATOM-702.18a-002 | 702.18a | Shroud on player | DEFERRED | S7b | shroud, player |
| ATOM-702.19c-001 | 702.19c | Trample over planeswalkers | DEFERRED | S7b | trample, trample-over-planeswalkers |
| ATOM-702.19e-001 | 702.19e | Trample-over-PW with PW removed | DEFERRED | S7b | trample, trample-over-planeswalkers, PW-removed |
| ATOM-702.19f-001 | 702.19f | Normal trample can't overflow to player when attacking PW | DEFERRED | S7b | trample, planeswalker, no-overflow-to-player |
| ATOM-702.21b-001 | 702.21b | Ward X evaluated at resolution | DEFERRED | S7b | ward, variable-cost, resolution-time |
| ATOM-702.23a-001 | 702.23a | Rampage triggered P/T bonus | DEFERRED | S7b | rampage, triggered-ability |
| ATOM-702.24a-001 | 702.24a | Cumulative upkeep escalating cost + sacrifice | DEFERRED | S7b | cumulative-upkeep, triggered-ability, age-counters |
| COMP-702-008 | 702.24a + Solemnity | CU + Solemnity: 0 age counters → free upkeep | Phase 8 | S7b | DEFERRED |
| ATOM-702.25a-001 | 702.25a | Flanking debuffs non-flanking blocker | DEFERRED | S7b | flanking, triggered-ability |
| ATOM-702.26a-001 | 702.26a | Phasing event during untap step | DEFERRED | S7b | phasing, untap-step, phase-out |
| ATOM-702.26b-001 | 702.26b | Phased-out permanents are invisible | DEFERRED | S7b | phasing, phased-out, invisible |
| ATOM-702.26d-001 | 702.26d | Phasing is NOT a zone change | DEFERRED | S7b | phasing, no-zone-change, tokens, counters |
| ATOM-702.26g-001 | 702.26g | Indirect phasing of attached permanents | DEFERRED | S7b | phasing, indirect-phasing, attachment |
| ATOM-702.27a-001 | 702.27a | Buyback additional cost + hand return | DEFERRED | S7b | buyback, additional-cost, hand-return, T17 |
| ATOM-702.27a-002 | 702.27a | Buyback not paid → normal graveyard | DEFERRED | S7b | buyback, no-buyback |
| ATOM-702.28b-001 | 702.28b | Shadow bidirectional evasion | DEFERRED | S7b | shadow, evasion, blocking |
| ATOM-702.29a-001 | 702.29a | Cycling discard-from-hand to draw | DEFERRED | S7b | cycling, activated-ability, discard-draw |
| ATOM-702.29e-001 | 702.29e | Typecycling searches for specific type | DEFERRED | S7b | cycling, typecycling, library-search |
| ATOM-702.30a-001 | 702.30a | Echo trigger: sacrifice or pay on first upkeep | DEFERRED | S7b | echo, triggered-ability, sacrifice |
| ATOM-702.30a-002 | 702.30a | Echo re-triggers after control change | DEFERRED | S7b | echo, control-change, triggered-ability |
| ATOM-702.31b-001 | 702.31b | Horsemanship evasion | DEFERRED | S7b | horsemanship, evasion, blocking |
| ATOM-702.32a-001 | 702.32a | Fading ETB counters + upkeep removal + sacrifice | DEFERRED | S7b | fading, counters, sacrifice |
| ATOM-702.33a-001 | 702.33a | Kicker additional cost + kicked status | DEFERRED | S7b | kicker, additional-cost, T17 |
| ATOM-702.33c-001 | 702.33c | Multikicker repeated payments | DEFERRED | S7b | kicker, multikicker |
| ATOM-702.33g-001 | 702.33g | Conditional targets based on kicked status | DEFERRED | S7b | kicker, conditional-targets |
| ATOM-702.34a-001 | 702.34a | Flashback from graveyard + exile on leave-stack | DEFERRED | S7b | flashback, alternative-cost, graveyard-cast, exile, T17 |
| ATOM-702.35a-001 | 702.35a | Madness replacement on discard + cast from exile | DEFERRED | S7b | madness, replacement-effect, discard, exile-cast |
| ATOM-702.36b-001 | 702.36b | Fear evasion: artifact or black blocker | DEFERRED | S7b | fear, evasion, blocking |
| ATOM-702.37a-001 | 702.37a | Morph casting: face-down 2/2 for {3} | DEFERRED | S7b | morph, face-down, casting |
| ATOM-702.37e-001 | 702.37e | Morph face-up special action | DEFERRED | S7b | morph, face-up, special-action |
| ATOM-702.38a-001 | 702.38a | Amplify ETB counters from revealed hand cards | DEFERRED | S7b | amplify, counters, ETB |
| ATOM-702.39a-001 | 702.39a | Provoke forces block + untaps target | DEFERRED | S7b | provoke, triggered-ability, forced-block |
| ATOM-702.40a-001 | 702.40a | Storm copies based on spells-cast-this-turn | DEFERRED | S7b | storm, triggered-ability, copy, spell-count |
| ATOM-702.41a-001 | 702.41a | Affinity cost reduction by permanent count | DEFERRED | S7b | affinity, cost-reduction, T17 |
| ATOM-702.42a-001 | 702.42a | Entwine additional cost enables all modes | DEFERRED | S7b | entwine, modal, additional-cost |
| ATOM-702.43a-001 | 702.43a | Modular ETB counters + death trigger transfer | DEFERRED | S7b | modular, counters, death-trigger |
| ATOM-702.43a-002 | 702.43a | Modular ETB counter placement | DEFERRED | S7b | modular, counters, ETB |
| ATOM-702.44a-001 | 702.44a | Sunburst ETB counters by mana colors spent | DEFERRED | S7b | sunburst, counters, mana-colors |
| ATOM-702.45a-001 | 702.45a | Bushido trigger on blocking/being blocked | DEFERRED | S7b | bushido, triggered-ability |
| ATOM-702.46a-001 | 702.46a | Soulshift death trigger recovers Spirit | DEFERRED | S7b | soulshift, death-trigger, spirit |
| ATOM-702.46a-002 | 702.46a | Soulshift MV cap (negative case) | DEFERRED | S7b | soulshift, MV-cap, negative-case |
| ATOM-702.47a-001 | 702.47a | Splice adds rules text from hand to spell | DEFERRED | S7b | splice, text-changing, additional-cost |
| ATOM-702.48a-001 | 702.48a | Offering sacrifice for cost reduction + instant speed | DEFERRED | S7b | offering, sacrifice, cost-reduction |
| ATOM-702.49a-001 | 702.49a | Ninjutsu swap unblocked attacker for hand creature | DEFERRED | S7b | ninjutsu, activated-ability, swap, ETB-attacking |
| ATOM-702.49a-002 | 702.49a | Ninjutsu after first-strike damage | DEFERRED | S7b | ninjutsu, first-strike, timing |
| ATOM-702.49a-003 | 702.49a | Ninjutsu in end-of-combat (no damage) | DEFERRED | S7b | ninjutsu, end-of-combat, timing |
| ATOM-702.50a-001 | 702.50a | Epic locks out casting + recurring copies | DEFERRED | S7b | epic, casting-restriction, delayed-trigger |
| ATOM-702.51a-001 | 702.51a | Convoke tap creatures as mana payment | DEFERRED | S7b | convoke, cost-modification, tap-creatures, T17 |
| ATOM-702.52a-001 | 702.52a | Dredge replace draw with mill + graveyard return | DEFERRED | S7b | dredge, replacement-effect, mill, graveyard |
| ATOM-702.52b-001 | 702.52b | Dredge unavailable if library < N | DEFERRED | S7b | dredge, library-size, boundary, negative-case |
| ATOM-702.53a-001 | 702.53a | Transmute hand-based tutor by MV | DEFERRED | S7b | transmute, activated-ability, tutor |
| ATOM-702.54a-001 | 702.54a | Bloodthirst conditional ETB counters | DEFERRED | S7b | bloodthirst, counters, ETB |
| ATOM-702.55a-001 | 702.55a | Haunt trigger exiles card haunting a creature | DEFERRED | S7b | haunt, exile, haunting |
| ATOM-702.56a-001 | 702.56a | Replicate additional cost + triggered copies | DEFERRED | S7b | replicate, additional-cost, copy |
| ATOM-702.57a-001 | 702.57a | Forecast hand-based upkeep-only once-per-turn | DEFERRED | S7b | forecast, activated-ability, upkeep |
| ATOM-702.58a-001 | 702.58a | Graft ETB counters + trigger to share | DEFERRED | S7b | graft, counters, triggered-ability |
| ATOM-702.59a-001 | 702.59a | Recover trigger: pay to recover or exile | DEFERRED | S7b | recover, graveyard-trigger |
| ATOM-702.60a-001 | 702.60a | Ripple reveal + free-cast same-name | DEFERRED | S7b | ripple, triggered-ability, free-cast |
| ATOM-702.61a-001 | 702.61a | Split second restricts casting/activation | DEFERRED | S7b | split-second, stack, casting-restriction |
| ATOM-702.61b-001 | 702.61b | Split second exceptions: mana abilities, triggers | DEFERRED | S7b | split-second, mana-abilities, triggers |
| ATOM-702.62a-001 | 702.62a | Suspend initial exile from hand (special action) | DEFERRED | S7b | suspend, exile, special-action |
| ATOM-702.62a-002 | 702.62a | Suspend upkeep counter removal | DEFERRED | S7b | suspend, upkeep-trigger, time-counters |
| ATOM-702.62a-003 | 702.62a | Suspend free cast on last counter removal + haste | DEFERRED | S7b | suspend, free-cast, haste, last-counter |
| ATOM-702.63a-001 | 702.63a | Vanishing ETB counters + upkeep removal + sacrifice | DEFERRED | S7b | vanishing, time-counters, sacrifice |
| ATOM-702.63b-001 | 702.63b | Vanishing with 0 counters (Solemnity): no sacrifice | DEFERRED | S7b | vanishing, solemnity, zero-counters |
| ATOM-702.63b-002 | 702.63b | External counter restarts vanishing countdown | DEFERRED | S7b | vanishing, external-counter, sacrifice |
| ATOM-702.64a-001 | 702.64a | Absorb prevents N damage per source | DEFERRED | S7b | absorb, damage-prevention |
| ATOM-702.64a-002 | 702.64a | Absorb per-source with multiple sources | DEFERRED | S7b | absorb, per-source, multiple-sources |
| ATOM-702.65a-001 | 702.65a | Aura swap exchange Aura with hand | DEFERRED | S7b | aura-swap, exchange |
| ATOM-702.66a-001 | 702.66a | Delve exile graveyard cards for generic cost | DEFERRED | S7b | delve, cost-modification, graveyard-exile, T17 |
| ATOM-702.67a-001 | 702.67a | Fortify attaches Fortification to land | DEFERRED | S7b | fortify, fortification, attachment |
| ATOM-702.68a-001 | 702.68a | Frenzy trigger on unblocked attack | DEFERRED | S7b | frenzy, triggered-ability |
| ATOM-702.69a-001 | 702.69a | Gravestorm copies based on permanents-died count | DEFERRED | S7b | gravestorm, triggered-ability, copy |
| ATOM-702.70a-001 | 702.70a | Poisonous gives poison counters on combat damage | DEFERRED | S7b | poisonous, poison-counters, triggered-ability |
| ATOM-702.71a-001 | 702.71a | Transfigure sacrifice-tutor for same MV creature | DEFERRED | S7b | transfigure, sacrifice, tutor |
| ATOM-702.72a-001 | 702.72a | Champion ETB exile + LTB return | DEFERRED | S7b | champion, ETB, LTB, exile, linked-abilities |
| ATOM-702.72a-002 | 702.72a | Champion no valid target → sacrifice | DEFERRED | S7b | champion, no-valid-target, sacrifice |
| ATOM-702.72a-003 | 702.72a | Champion declines exile → sacrifice | DEFERRED | S7b | champion, decline-exile, sacrifice |
| ATOM-702.73a-001 | 702.73a | Changeling grants all creature types (CDA) | DEFERRED | S7b | changeling, CDA, creature-types |
| ATOM-702.73a-002 | 702.73a | Changeling works in all zones | DEFERRED | S7b | changeling, CDA, all-zones |
| ATOM-702.74a-001 | 702.74a | Evoke alternative cost + ETB sacrifice | DEFERRED | S7b | evoke, alternative-cost, ETB, sacrifice, T17 |
| ATOM-702.75a-001 | 702.75a | Hideaway ETB: peek + face-down exile | DEFERRED | S7b | hideaway, ETB, face-down-exile |
| ATOM-702.76a-001 | 702.76a | Prowl conditional alternative cost | DEFERRED | S7b | prowl, alternative-cost, creature-type |
| ATOM-702.77a-001 | 702.77a | Reinforce hand-based +1/+1 counters | DEFERRED | S7b | reinforce, activated-ability, counters |
| ATOM-702.78a-001 | 702.78a | Conspire tap creatures + triggered copy | DEFERRED | S7b | conspire, additional-cost, copy |
| ATOM-702.79a-001 | 702.79a | Persist death trigger: return with -1/-1 (LKI check) | DEFERRED | S7b | persist, death-trigger, minus-counters, LKI |
| ATOM-702.79a-002 | 702.79a | Persist no trigger if had -1/-1 counters (LKI) | DEFERRED | S7b | persist, no-trigger, minus-counters, LKI |
| ATOM-702.80a-001 | 702.80a | Wither: damage → -1/-1 counters on creatures | DEFERRED | S7b | wither, minus-counters, damage-replacement |
| ATOM-702.80a-002 | 702.80a | Wither doesn't affect player damage | DEFERRED | S7b | wither, player-damage |
| ATOM-702.80b-001 | 702.80b | Wither LKI after zone change | DEFERRED | S7b | wither, LKI |
| COMP-702-006 | 702.80a + 702.79a | Wither + persist: -1/-1 counters prevent persist retrigger | Phase 8 | S7b | DEFERRED |
| ATOM-702.81a-001 | 702.81a | Retrace: cast from GY by discarding land | NEW — Retrace | S8 |  |
| ATOM-702.84a-001 | 702.84a | Unearth: return from GY with haste, delayed exile | NEW — Unearth | S8 |  |
| ATOM-702.87a-001 | 702.87a | Level up: sorcery-speed activated, level counter | NEW — Level Up | S8 | CROSS-REF rule 711 |
| ATOM-702.88a-002 | 702.88a | Rebound only works if cast from hand | (same) | S8 |  |
| ATOM-702.90e-001 | 702.90e | Infect functions regardless of zone | T21c | S8 | SHARED-BEHAVIOR wither |
| COMP-INFECT-TRAMPLE-001 | ATOM-702.90b + Trample 702.19b | Infect + trample: -1/-1 to blocker, poison overflow to player | T21c + trample | S8 |  |
| ATOM-702.97a-001 | 702.97a | Scavenge: GY activated, exile self, counters on target | NEW — Scavenge | S8 |  |
| ATOM-702.99a-001 | 702.99a | Cipher: encode on creature, combat damage → copy + free cast | NEW — Cipher | S8 |  |
| ATOM-702.99a-003 | 702.99a | Cipher: copy of cipher spell not encoded | (same) | S8 |  |
| ATOM-702.99c-001 | 702.99c | Cipher encoding persists through controller change | (same) | S8 |  |
| ATOM-702.103e-001 | 702.103e | Bestow: illegal target → enters as creature | (same) | S8 |  |
| ATOM-702.103f-001 | 702.103f | Bestow: unattach → ceases bestow, becomes creature | (same) | S8 |  |
| ATOM-702.107a-001 | 702.107a | Outlast: sorcery-speed tap activated, +1/+1 counter | NEW — Outlast | S8 |  |
| ATOM-702.109a-001 | 702.109a | Dash: alt cost, haste, return to hand at end step | NEW — Dash | S8 |  |
| ATOM-702.109a-002 | 702.109a | Dash: normal cast → no haste/return | (same) | S8 |  |
| COMP-DASH-BLITZ-001 | ATOM-702.109a + ATOM-702.152a | Only one alt cost (dash/blitz) can be paid | T17 | S8 |  |
| ATOM-702.113a-002 | 702.113a | Awaken: original spell effects still apply | (same) | S8 |  |
| ATOM-702.113b-001 | 702.113b | Awaken target only chosen if awaken cost paid | (same) | S8 |  |
| ATOM-702.117a-001 | 702.117a | Surge: alt cost if spell cast this turn | NEW — Surge | S8 |  |
| ATOM-702.118b-001 | 702.118b | Skulk: can't be blocked by greater power | NEW — Skulk | S8 |  |
| ATOM-702.118b-002 | 702.118b | Skulk: CAN be blocked by ≤ power | (same) | S8 |  |
| ATOM-702.120a-001 | 702.120a | Escalate: additional cost per extra mode | NEW — Escalate | S8 |  |
| ATOM-702.122a-001 | 702.122a | Crew N: tap creatures ≥ N power → animate Vehicle | NEW — Crew | S8 |  |
| ATOM-702.122a-002 | 702.122a | Crew fails if total power < N | (same) | S8 |  |
| ATOM-702.122-003 | 702.122a | Crew on non-Vehicle (via ability copying) | (same) | S8 |  |
| ATOM-702.122c-001 | 702.122c | "Can't crew Vehicles" restriction | (same) | S8 |  |
| ATOM-702.128a-001 | 702.128a | Embalm: GY activated, exile self, modified token copy | NEW — Embalm | S8 |  |
| ATOM-702.129a-001 | 702.129a | Eternalize: GY activated, 4/4 black Zombie token copy | NEW — Eternalize | S8 |  |
| ATOM-702.131a-001 | 702.131a | Ascend on spell: city's blessing if 10+ permanents | NEW — Ascend | S8 |  |
| ATOM-702.131b-001 | 702.131b | Ascend on permanent: immediate designation at 10+ | NEW — Ascend (static) | S8 | ARCHITECTURE-CONCERN mid-resolution-static-checks |
| ATOM-702.131b-002 | 702.131b | Ascend: mid-resolution acquisition | (same) | S8 |  |
| ATOM-702.131c-001 | 702.131c | City's blessing persists below 10 permanents | (same) | S8 |  |
| ATOM-702.132a-001 | 702.132a | Assist: another player pays generic mana | NEW — Assist | S8 |  |
| ATOM-702.132a-002 | 702.132a | Assist: chosen player declines → caster pays full | (same) | S8 |  |
| ATOM-702.133a-001 | 702.133a | Jump-start: cast from GY + discard + exile on leave | NEW — Jump-Start | S8 |  |
| ATOM-702.133a-002 | 702.133a | Jump-start from hand → normal GY (no exile) | (same) | S8 |  |
| ATOM-702.137a-002 | 702.137a | Spectacle: can't use if no life lost | (same) | S8 |  |
| ATOM-702.138d-001 | 702.138d | Escape: gains ability if escaped | (same) | S8 |  |
| ATOM-702.140a-001 | 702.140a | Mutate: alt cost targeting non-Human creature | NEW — Mutate | S8 |  |
| ATOM-702.140b-001 | 702.140b | Mutate: illegal target → enters as creature | (same) | S8 |  |
| ATOM-702.140c-001 | 702.140c | Mutate: merge with target, choose top/bottom | (same) | S8 |  |
| ATOM-702.140e-001 | 702.140e | Mutated permanent: all abilities from all, other chars from top | (same) | S8 |  |
| ATOM-702.142a-001 | 702.142a | Boast: activated only if attacked this turn, once/turn | NEW — Boast | S8 |  |
| ATOM-702.142a-002 | 702.142a | Boast: can't activate if didn't attack | (same) | S8 |  |
| ATOM-702.143a-002 | 702.143a | Foretold card can't be cast same turn | (same) | S8 |  |
| ATOM-702.150a-001 | 702.150a | Compleated: reduces starting loyalty per Phyrexian mana | NEW — Compleated | S8 |  |
| ATOM-702.151a-001 | 702.151a | Reconfigure: attach/unattach activated, sorcery speed | NEW — Reconfigure | S8 |  |
| ATOM-702.152a-001 | 702.152a | Blitz: alt cost, haste, dies-draw, delayed sacrifice | NEW — Blitz | S8 |  |
| ATOM-702.152a-002 | 702.152a | Blitz: normal cast → no blitz effects | (same) | S8 |  |
| ATOM-702.153a-002 | 702.153a | Casualty: not paying → no copy | (same) | S8 |  |
| ATOM-702.155b-001 | 702.155b | Read ahead: choose starting lore counter count | NEW — Read Ahead | S8 |  |
| COMP-TOXIC-INFECT-001 | ATOM-702.164a + ATOM-702.90b/c | Infect replaces damage + toxic triggers additionally | — | S8 |  |
| ATOM-702.166a-001 | 702.166a | Bargain: optional sacrifice additional cost | NEW — Bargain | S8 |  |
| ATOM-702.167a-001 | 702.167a | Craft: exile self + materials, return transformed | NEW — Craft | S8 |  |
| ATOM-702.168a-001 | 702.168a | Disguise: cast face down 2/2 with ward {2} for {3} | NEW — Disguise | S8 | SHARED-BEHAVIOR morph-disguise |
| ATOM-702.168b-001 | 702.168b | Disguise: turn face up by paying disguise cost | (same) | S8 |  |
| ATOM-702.169a-001 | 702.169a | Solved: Case becomes solved when condition met | NEW — Solved/Case | S8 |  |
| ATOM-702.170a-001 | 702.170a | Plot: pay plot cost, exile face up, cast free later | NEW — Plot | S8 | ARCHITECTURE-CONCERN special-action-mod |
| ATOM-702.170a-002 | 702.170a | Plotted card can't be cast same turn | (same) | S8 |  |
| ATOM-702.170f-001 | 702.170f | Plotted card leaves exile → new object, not plotted | (same) | S8 |  |
| ATOM-702.171a-001 | 702.171a | Saddle N: tap creatures ≥ N power, saddled designation | NEW — Saddle | S8 | SHARED-BEHAVIOR crew-saddle |
| ATOM-702.172a-002 | 702.172 | Spree: must choose at least one mode | (same) | S8 |  |
| ATOM-702.172a-001 | 702.172a | Spree: choose additional costs, each adds a mode | NEW — Spree | S8 |  |
| ATOM-702.173a-001 | 702.173a | Freerunning: alt cost if Assassin/new creature dealt damage | NEW — Freerunning | S8 | COMMANDER-INTERACTION |
| ATOM-702.174a-001 | 702.174a | Gift on instant/sorcery: opponent choice + benefit first | NEW — Gift | S8 |  |
| ATOM-702.174a-003 | 702.174a | Gift: declining (no opponent chosen) | (same) | S8 |  |
| ATOM-702.176a-004 | 702.176a | Impending: normal cast → creature immediately | (same) | S8 |  |
| ATOM-702.177a-001 | 702.177a | Exhaust: activate-only-once (ever) | NEW — Exhaust | S8 |  |
| ATOM-702.177a-002 | 702.177a | Exhaust: zone change resets restriction | (same) | S8 |  |
| ATOM-702.178a-001 | 702.178a | Max speed: ability conditional on speed = 4 | NEW — Max Speed | S8 |  |
| ATOM-702.179a-001 | 702.179a | Start your engines!: SBA sets speed to 1 | NEW — Start Your Engines! + Speed | S8 |  |
| ATOM-702.183a-001 | 702.183a | Tiered: choose one mode, pay that mode's additional cost | NEW — Tiered | S8 |  |
| ATOM-702.184a-001 | 702.184a | Station: tap creature → charge counters = power | NEW — Station | S8 |  |
| ATOM-702.186b-001 | 702.186b | ∞: ability conditional on harnessed designation | NEW — ∞ | S8 |  |
| ATOM-702.186b-002 | 702.186b | ∞: harness acquisition activates static | (same) | S8 |  |
| ATOM-702.187b-002 | 702.187b | Mayhem: can't cast if not discarded this turn | (same) | S8 |  |
| ATOM-702.187b-003 | 702.187b | Mayhem: GY leave+return → new object, not discarded | (same) | S8 |  |
| ATOM-702.187c-001 | 702.187c | Mayhem (no cost): play from GY (includes lands) | (same) | S8 |  |
| ATOM-702.190b-001 | 702.190b | Sneak: enters tapped+attacking same target | (same) | S8 |  |
| ATOM-704.5k-001 | 704.5k | World rule — older world permanent destroyed | NEW — World rule SBA | S9a |  |
| ATOM-704.5k-002 | 704.5k | World rule — tied timestamps → both destroyed | NEW — World rule SBA | S9a |  |
| ATOM-704.5r-001 | 704.5r | Counter cap SBA — remove excess | NEW — Counter cap SBA | S9a |  |
| ATOM-705.1-001 | 705.1 | Coin flip RNG produces heads or tails | NEW — Coin flip system | S9a |  |
| ATOM-705.2-001 | 705.2 | Call-based coin flip — correct call wins | NEW — Coin flip system | S9a |  |
| ATOM-705.2-002 | 705.2 | Call-based coin flip — wrong call loses | NEW — Coin flip system | S9a |  |
| ATOM-705.3-001 | 705.3 | Coin flip result override by effect | NEW — Coin flip system | S9a |  |
| ATOM-706.2-001 | 706.2 | Die roll natural vs. final result with modifiers | NEW — Die roll system | S9a |  |
| ATOM-706.2b-001 | 706.2b | Die roll modifier ordering (reroll first, then arithmetic) | NEW — Die roll system | S9a |  |
| ATOM-706.3a-001 | 706.3a | Results table lookup | NEW — Die roll system | S9a |  |
| ATOM-706.3c-001 | 706.3c | "Roll again" recursive die roll | NEW — Die roll system | S9a |  |
| ATOM-706.6-001 | 706.6 | Ignored roll — never happened, no triggers | NEW — Die roll system | S9a |  |
| ATOM-714.3a-002 | 714.3a | Saga with read ahead ETB counter choice | (same) | S9b |  |
| ATOM-716.2-001 | 716.2 | Class level-up activation + static ability | NEW — Class level-up | S9b |  |
| ATOM-716.2a-001 | 716.2a | Class level activation restriction (level N-1, sorcery) | NEW — Class restriction | S9b |  |
| ATOM-716.2b-001 | 716.2b | Class level designation persistence + non-copiability | NEW — Class designation | S9b |  |
| ATOM-716.2c-001 | 716.2c | Mana spending restriction: "gain a Class level" | T17 + NEW | S9b |  |
| ATOM-716.2d-001 | 716.2d | Default level = 1 for permanents without level | NEW — Default level | S9b |  |
| COMP-CLASS-LEVELUP-001 | 716.2 + 716.2a + 716.2d + 716.3 | Class level progression: 1 → 2 → 3 | Phase 8 | S9b | NEW |
| ATOM-716.3-001 | 716.3 | Class top-section abilities always active | NEW — Class top abilities | S9b |  |
| ATOM-719.3b-001 | 719.3b | Solved designation persistence + non-copiability | NEW — Solved designation | S9b |  |
| ATOM-719.3c-001 | 719.3c | Solved ability active only when solved | NEW — Solved ability | S9b |  |
| ATOM-721.2a-001 | 721.2a | Station counter-threshold ability grant | NEW — Station threshold | S9b |  |
| ATOM-721.2b-001 | 721.2b | Station counter-threshold creature transformation | NEW — Station creature | S9b |  |
| ATOM-721.2c-001 | 721.2c | Station no P/T in non-battlefield zones | NEW — Station zone P/T | S9b |  |
| ATOM-721.4-001 | 721.4 | Station base abilities always active | NEW — Station base abilities | S9b |  |
| ATOM-722.1-001 | 722.1 | Player control effect duration (next actual turn) | NEW — Player control | S9b |  |
| ATOM-722.1a-001 | 722.1a | Player control overwrite (last wins) | (same) | S9b |  |
| ATOM-722.1b-001 | 722.1b | Player control persists through skipped turns | (same) | S9b |  |
| ATOM-722.3-001 | 722.3 | Player control vs object control distinction | (same) | S9b |  |
| ATOM-722.4-001 | 722.4 | Player control information visibility | (same) | S9b |  |
| ATOM-722.5-001 | 722.5 | Player control decision routing to DP | (same) | S9b |  |
| ATOM-722.5a-001 | 722.5a | Player control resource isolation | (same) | S9b |  |
| ATOM-722.6-001 | 722.6 | Player control cannot force concession | (same) | S9b |  |
| ATOM-722.7-001 | 722.7 | Player control effect action restrictions | (same) | S9b |  |
| ATOM-722.9-001 | 722.9 | Player self-control no-op | (same) | S9b |  |
| ATOM-723.1-001 | 723.1 | End-the-turn procedure entry point | NEW — End-the-turn | S9b |  |
| ATOM-723.1a-001 | 723.1a | End-the-turn pending trigger purge | (same) | S9b |  |
| ATOM-723.1b-001 | 723.1b | End-the-turn stack exile | (same) | S9b |  |
| ATOM-723.1c-001 | 723.1c | End-the-turn SBA check with suppression | (same) | S9b |  |
| ATOM-723.1d-001 | 723.1d | End-the-turn phase skip to cleanup | (same) | S9b |  |
| ATOM-723.1d-002 | 723.1d | End-the-turn during cleanup → new cleanup | (same) | S9b |  |
| ATOM-723.1e-001 | 723.1e | End-the-turn skips end-step triggers | (same) | S9b |  |
| ATOM-723.1f-001 | 723.1f | End-the-turn cleanup-deferred triggers + priority | (same) | S9b |  |
| ATOM-723.1f-002 | 723.1f | End-the-turn cleanup no priority when no triggers | (same) | S9b |  |
| COMP-END-TURN-FULL-001 | 723.1a–f | Full end-the-turn procedure | Phase 8 | S9b | NEW |
| ATOM-723.2-001 | 723.2 | End-combat-phase procedure entry point | NEW — End-combat-phase | S9b |  |
| ATOM-723.2a-001 | 723.2a | End-combat-phase pending trigger purge | (same) | S9b |  |
| ATOM-723.2b-001 | 723.2b | End-combat-phase stack exile | (same) | S9b |  |
| ATOM-723.2c-001 | 723.2c | End-combat-phase SBA with suppression | (same) | S9b |  |
| ATOM-723.2d-001 | 723.2d | End-combat-phase termination + duration expiry | (same) | S9b |  |
| ATOM-723.2e-001 | 723.2e | End-combat-phase skips end-of-combat triggers | (same) | S9b |  |
| ATOM-723.2f-001 | 723.2f | End-combat-phase deferred triggers in following phase | (same) | S9b |  |
| ATOM-723.2g-001 | 723.2g | End-combat-phase no-op outside combat | (same) | S9b |  |
| ATOM-727.1-001 | 727.1 | Rad counter triggered ability (mill + life loss) | NEW — Rad counter | S9b |  |
| ATOM-727.1-002 | 727.1 | Rad counter no-trigger at 0 counters | (same) | S9b |  |
| ATOM-727.1-003 | 727.1 | Rad counter full nonland mill scenario | (same) | S9b |  |

---

## Phase 9

**226 entries**

| ID | Rule | Summary | Ticket | Session | Tags |
|----|------|---------|--------|---------|------|
| ATOM-103.8c-001 | 103.8c | Starting player skips first draw (Commander) | Phase 9 | S1 |  |
| ATOM-104.3j-001 | 104.3j | Commander 21+ combat damage → lose | Phase 9 | S1 |  |
| ATOM-105.3-001 | 105.3 | Color identity (all mana symbols + color indicator) | Phase 9 | S1 |  |
| ATOM-105.3-002 | 105.3 | Hybrid mana in cost contributes both colors to identity | Phase 9 | S1 |  |
| ATOM-110.5c-001 | 110.5c | Status retained even when irrelevant | Phase 6 + Phase 9 | S1 |  |
| ATOM-110.5d-002 | 110.5d | Face-down in exile ≠ battlefield face-down status | Phase 5-Pre + Phase 9 | S1 |  |
| ATOM-116.2g-001 | 116.2g | Companion: pay {3} to put in hand, once/game | D20 | S1 |  |
| ATOM-120.3h-001 | 120.3h | Damage to battle removes defense counters | Phase 9 | S1 |  |
| ATOM-122.1g-001 | 122.1g | Battle with 0 defense → SBA graveyard | Phase 9 | S1 |  |
| ATOM-201.2a-002 | 201.2a | Nameless objects never share a name | NEW-S2-02 | S2 |  |
| ATOM-201.2b-001 | 201.2b | "Different names" group predicate — nameless blocks | NEW-S2-03 | S2 |  |
| ATOM-201.2c-001 | 201.2c | Singular "different name" — nameless returns false | NEW-S2-04 | S2 |  |
| ATOM-201.2c-003 | 201.2c | Singular "different name" — nameless vs group w/ nameless | NEW-S2-04 | S2 |  |
| ATOM-202.3a-002 | 202.3a | DFC back face MV uses front face cost | NEW-S2-06 | S2 |  |
| ATOM-202.3b-001 | 202.3b | Copy of DFC back face MV = 0 | NEW-S2-07 | S2 |  |
| ATOM-303.4c-003 | 303.4c | Aura on player who leaves game → graveyard SBA | NEW | S3 | aura, multiplayer, sba |
| ATOM-505.1a-002 | 505.1a | Multiple postcombat main phases each trigger effects | NEW — multiple postcombat trigger firing | S4 |  |
| ATOM-506.3e-001 | 506.3e | ETB-blocking controller mismatch — enters but not blocking | NEW — ETB-blocking controller validation | S4 |  |
| ATOM-506.4-004 | 506.4 | Removed from combat on phasing out | NEW — phasing removes from combat | S4 |  |
| ATOM-509.1a-003 | 509.1a | Multiplayer: blocker only blocks attacker targeting its controller | NEW — multiplayer blocker validation | S4 |  |
| ATOM-601.3e-001 | 601.3e | Alternative characteristic check for cast permissions | Phase 9 | S5 | morph |
| ATOM-601.6a-001 | 601.6a | Controller selects which opponent decides | Phase 9 | S5 | multiplayer |
| ATOM-608.2e-001 | 608.2e | APNAP-ordered resolution steps | T18, Phase 9 | S5 | multiplayer |
| ATOM-608.2f-001 | 608.2f | Simultaneous action processing | Phase 9 | S5 | multiplayer |
| ATOM-608.2f-002 | 608.2f | APNAP-ordered simultaneous sacrifice | T18, Phase 9 | S5 | multiplayer |
| ATOM-608.3b-002 | 608.3b | Bestow fallback | Phase 9 | S5 |  |
| ATOM-701.53a-001 | 701.53a | Incubate N — DFC Incubator token + counters | D3 | S7a |  |
| ATOM-701.53b-001 | 701.53b | Incubator transforms to 0/0 Phyrexian creature | D3 | S7a |  |
| ATOM-702.16k-001 | 702.16k | Protection from [player] | DEFERRED | S7b | protection, protection-from-player, multiplayer |
| ATOM-702.16k-002 | 702.16k | Protection from player checks controller not owner | DEFERRED | S7b | protection, protection-from-player, multiplayer, control-vs-ownership |
| ATOM-702.85a-002 | 702.85a | Cascade checks resulting spell's MV (split cards) | NEW — Cascade + split MV | S8 |  |
| ATOM-702.102a-001 | 702.102a | Fuse: cast both halves of split card from hand | NEW — Fuse | S8 |  |
| ATOM-702.102c-001 | 702.102c | Fused spell's total cost = both halves combined | (same) | S8 |  |
| ATOM-702.102d-001 | 702.102d | Fused spell resolves left half first, then right | (same) | S8 |  |
| ATOM-702.127a-001 | 702.127a | Aftermath: cast from GY, exile on leave stack | NEW — Aftermath | S8 |  |
| ATOM-702.127a-002 | 702.127a | Aftermath half cannot be cast from hand | (same) | S8 |  |
| ATOM-702.139a-001 | 702.139a | Companion: reveal from outside, pay {3} to hand | NEW — Companion | S8 |  |
| ATOM-702.160a-001 | 702.160a | Prototype: cast with alternate characteristics | NEW — Prototype | S8 |  |
| ATOM-704.6c-001 | 704.6c | Commander damage 21+ → player loses | T16 | S9a | commander |
| ATOM-707.2-002 | 707.2 | Copy of face-down creature gets face-down characteristics | D5 + Phase 9 | S9a | copy, face-down |
| ATOM-708.2-001 | 708.2 | Face-down characteristic suppression (morph) | NEW — Face-down system | S9a | face-down |
| ATOM-708.2a-001 | 708.2a | Default face-down characteristics (Ixidron) | NEW — Face-down system | S9a | face-down |
| ATOM-708.2b-001 | 708.2b | Face-down can't be turned face down again | NEW — Face-down system | S9a | face-down |
| ATOM-708.3-001 | 708.3 | Face-down enter — ETB suppressed | NEW — Face-down system | S9a | face-down |
| ATOM-708.4-001 | 708.4 | Face-down casting — CMC 0 on stack | NEW — Face-down system | S9a | face-down |
| ATOM-708.8-001 | 708.8 | Face-up revert — effects persist, no ETB | NEW — Face-down system | S9a | face-down |
| ATOM-708.10-001 | 708.10 | Face-down copy — shows copied chars on face-up | D5 + Phase 9 | S9a | face-down, copy |
| COMP-9A-003 | 708.10 + 708.8 + 707.3 | Face-down copy → face-up shows copied chars |  | S9a |  |
| ATOM-708.11-001 | 708.11 | "As this is turned face up" applied during transition | NEW — Face-down system | S9a | face-down |
| BOUNDARY-708.12-001 | 708.12 | Face-down reveal bypasses layer system | NEW — Face-down system | S9a | face-down, layers |
| ATOM-709.3-001 | 709.3 | Split card half selection on cast | NEW — Split card system | S9a | split |
| ATOM-709.3a-001 | 709.3a | Only chosen half evaluated for castability | NEW — Split card system | S9a | split |
| ATOM-709.3b-001 | 709.3b | On stack only cast half's characteristics exist | NEW — Split card system | S9a | split |
| BOUNDARY-709.3c-001 | 709.3c | Copy of split card vs copy of split spell on stack | NEW — Split card copy zone-awareness | S9a | split, copy |
| COMP-9A-007 | 709.3b + 709.4 + 709.4b | Split card characteristics shift between zones and stack |  | S9a |  |
| ATOM-709.4-001 | 709.4 | Combined characteristics in non-stack zones | NEW — Split card system | S9a | split |
| ATOM-709.4a-001 | 709.4a | Split card name matching (Pithing Needle) | NEW — Multi-name support | S9a | split |
| ATOM-709.4b-001 | 709.4b | Combined mana cost, colors, MV | NEW — Split card system | S9a | split |
| ATOM-710.2-001 | 710.2 | Flip card characteristic switching on flip | NEW — Flip card system | S9a | flip |
| ATOM-710.2-002 | 710.2 | Flip card — only normal chars in non-battlefield zones | NEW — Flip card system | S9a | flip |
| ATOM-710.4-001 | 710.4 | Flip is one-way; zone change resets status | NEW — Flip card system | S9a | flip |
| ATOM-711.2-001 | 711.2 | Level counter conditional P/T + abilities (mid-range) | NEW — Leveler system | S9a | leveler |
| ATOM-711.2-002 | 711.2 | Level counter max range — all abilities active | NEW — Leveler system | S9a | leveler |
| ATOM-711.4-001 | 711.4 | Level up activation always available | NEW — Leveler system | S9a | leveler |
| ATOM-711.5-001 | 711.5 | Below first level range → base P/T | NEW — Leveler system | S9a | leveler |
| ATOM-711.6-001 | 711.6 | Leveler in non-battlefield zone uses base P/T | NEW — Leveler system | S9a | leveler |
| ATOM-712.4a-001 | 712.4a | Meld — exile both, enter as single permanent | NEW — Meld system | S9a | dfc, meld |
| BOUNDARY-712.4b-001 | 712.4b | Meld back face only determines chars when melded on battlefield | NEW — Meld system | S9a | dfc, meld |
| ATOM-712.4c-001 | 712.4c | Meld cards can't transform/convert | NEW — Meld system | S9a | dfc, meld |
| ATOM-712.8-001 | 712.8 | DFC per-face characteristic isolation | NEW — DFC system | S9a | dfc |
| ATOM-712.8a-001 | 712.8a | DFC in non-battlefield zone → front face only | NEW — DFC system | S9a | dfc |
| ATOM-712.8c-001 | 712.8c | Nonmodal DFC cast transformed — MV from front face | NEW — DFC system | S9a | dfc |
| ATOM-712.8e-001 | 712.8e | Back-face-up DFC MV from front face (survives destruction) | NEW — DFC system | S9a | dfc |
| ATOM-712.8e-002 | 712.8e | Copy of back face has MV = 0 | NEW — DFC system | S9a | dfc, copy |
| ATOM-712.8g-001 | 712.8g | Melded permanent MV = sum of front faces | NEW — Meld system | S9a | dfc, meld |
| ATOM-712.8g-002 | 712.8g | Copy of melded permanent has MV = 0 | NEW — Meld system | S9a | dfc, meld, copy |
| ATOM-712.9-001 | 712.9 | Non-DFC can't transform (Clone negative case) | NEW — DFC system | S9a | dfc |
| ATOM-712.9-002 | 712.9 | DFC card that is a copy can still transform | NEW — DFC system | S9a | dfc, copy |
| COMP-9A-004 | 712.9 + 707.4 + 712.18 | DFC transform through copy effect |  | S9a |  |
| ATOM-712.10-001 | 712.10 | Transform into instant/sorcery face → nothing | NEW — DFC system | S9a | dfc |
| ATOM-712.11-001 | 712.11 | DFC spell default front face up on stack | NEW — DFC system | S9a | dfc |
| ATOM-712.11a-001 | 712.11a | Cast "transformed" → back face up on stack | NEW — DFC system | S9a | dfc |
| ATOM-712.11b-001 | 712.11b | Modal DFC face selection on cast | NEW — DFC system | S9a | dfc |
| BOUNDARY-712.11d-001 | 712.11d | Front-face ability enabling transformed cast evaluated for castability | NEW — DFC system | S9a | dfc |
| ATOM-712.12-001 | 712.12 | Modal DFC land play — choose land face, special action | NEW — Modal DFC land play | S9a | dfc |
| ATOM-712.12-002 | 712.12 | Modal DFC land play doesn't use stack | NEW — Modal DFC land play | S9a | dfc |
| ATOM-712.13-001 | 712.13 | DFC spell resolves with same face as on stack | NEW — DFC system | S9a | dfc |
| ATOM-712.13a-001 | 712.13a | DFC enter transformed — sorcery back face → graveyard | NEW — DFC system | S9a | dfc |
| ATOM-712.13a-002 | 712.13a | Stress test: Clone + Mystic Reflection + Siege sorcery back | NEW — DFC system | S9a | dfc, copy, stress |
| ATOM-712.14-001 | 712.14 | DFC from non-stack zone → front face up default | NEW — DFC system | S9a | dfc |
| ATOM-712.14a-001 | 712.14a | DFC "transformed" entry; non-DFC stays in zone | NEW — DFC system | S9a | dfc |
| ATOM-712.14b-001 | 712.14b | Modal DFC non-permanent front face → stays in zone | NEW — DFC system | S9a | dfc |
| ATOM-712.16-001 | 712.16 | DFC can't be turned face down | NEW — DFC system | S9a | dfc |
| ATOM-712.18-001 | 712.18 | Transform doesn't create new object — effects persist | NEW — DFC system | S9a | dfc |
| ATOM-712.20-001 | 712.20 | "As this transforms" applied during transform | NEW — DFC system | S9a | dfc |
| ATOM-712.21-001 | 712.21 | Meld death — 1 permanent leaves, 2 cards to zone | NEW — Meld system | S9a | dfc, meld |
| ATOM-712.21a-001 | 712.21a | Meld to graveyard/library — owner orders 2 cards | NEW — Meld system | S9a | dfc, meld |
| BOUNDARY-712.21b-001 | 712.21b | Meld exile — player determines relative timestamps of 2 cards | NEW — Meld system | S9a | dfc, meld |
| ATOM-712.21c-001 | 712.21c | Meld tracking — effect finds both cards | NEW — Meld system | S9a | dfc, meld |
| ATOM-712.21d-001 | 712.21d | Meld replacement non-splitting — one effect for both cards | NEW — Meld system | S9a | dfc, meld, replacement-effects |
| ATOM-712.21e-001 | 712.21e | Meld = 1 object moved, 2 cards moved | NEW — Meld system | S9a | dfc, meld |
| COMP-9A-005 | 712.21 + 712.21d + 712.21e | Meld death + replacement non-splitting + card counting |  | S9a |  |
| ATOM-715.2-001 | 715.2 | Adventure zone-dependent characteristics on stack | D4 | S9b |  |
| ATOM-715.2a-001 | 715.2a | "Has an Adventure" flag query in non-stack zone | D4 | S9b |  |
| ATOM-715.2b-001 | 715.2b | Adventure alternative characteristics are copiable values | D4 | S9b |  |
| ATOM-715.3-001 | 715.3 | Adventure cast mode choice via DecisionProvider | D4 | S9b |  |
| ATOM-715.3a-001 | 715.3a | Adventure cast legality uses alternative characteristics | D4 | S9b |  |
| ATOM-715.3b-001 | 715.3b | Adventure spell has only alternative characteristics on stack | D4 | S9b |  |
| ATOM-715.3c-001 | 715.3c | Copy of Adventure spell is also an Adventure | D4 + D19 | S9b |  |
| ATOM-715.3d-001 | 715.3d | Adventure resolves to exile with re-cast permission | D4 | S9b |  |
| COMP-ADVENTURE-LIFECYCLE-001 | 715.3 + 715.3a + 715.3b + 715.3d + 715.4 | Full Adventure lifecycle: cast → stack → exile → re-cast | Phase 9 (D4) | S9b | D4 |
| ATOM-715.4-001 | 715.4 | Adventure normal characteristics in non-stack zones | D4 | S9b |  |
| ATOM-715.5-001 | 715.5 | Adventure alternative name choice | D4 + D18 | S9b |  |
| ATOM-718.2-001 | 718.2 | Prototype alternative characteristics on stack/battlefield | NEW — Prototype chars | S9b |  |
| ATOM-718.2a-001 | 718.2a | Prototype copiable values | NEW — Prototype copy | S9b |  |
| ATOM-718.3-001 | 718.3 | Prototype cast mode choice | NEW — Prototype cast | S9b |  |
| ATOM-718.3a-001 | 718.3a | Prototype cast legality uses alternative cost | (same) | S9b |  |
| ATOM-718.3b-001 | 718.3b | Prototype color derived from alternative mana cost | NEW — Prototype color | S9b |  |
| ATOM-718.3c-001 | 718.3c | Copy of prototyped spell preserves prototype | NEW + D19 | S9b |  |
| ATOM-718.3d-001 | 718.3d | Copy of prototyped permanent has alt characteristics | NEW — Prototype perm copy | S9b |  |
| COMP-PROTOTYPE-LIFECYCLE-001 | 718.3 + 718.3a + 718.3b + 718.4 + 718.5 | Prototype lifecycle: cast → stack/battlefield → graveyard | Phase 9 | S9b | NEW |
| ATOM-718.4-001 | 718.4 | Prototype normal characteristics in non-stack/battlefield | NEW — Prototype zone chars | S9b |  |
| ATOM-718.4-002 | 718.4 | Non-prototyped permanent uses normal characteristics | (same) | S9b |  |
| ATOM-718.5-001 | 718.5 | Prototype non-overridden characteristics unchanged | NEW — Prototype types/abilities | S9b |  |
| ATOM-720.2-001 | 720.2 | Omen zone-dependent characteristics on stack | D4 | S9b |  |
| ATOM-720.2a-001 | 720.2a | "Has an Omen" flag query in non-stack zone | D4 | S9b |  |
| ATOM-720.2b-001 | 720.2b | Omen alternative characteristics are copiable values | D4 | S9b |  |
| ATOM-720.3-001 | 720.3 | Omen cast mode choice via DecisionProvider | D4 | S9b |  |
| ATOM-720.3a-001 | 720.3a | Omen cast legality uses alternative characteristics | D4 | S9b |  |
| ATOM-720.3b-001 | 720.3b | Omen spell has only alternative characteristics on stack | D4 | S9b |  |
| ATOM-720.3c-001 | 720.3c | Copy of Omen spell is also an Omen | D4 + D19 | S9b |  |
| ATOM-720.3d-001 | 720.3d | Omen resolves by shuffling into library | D4 | S9b |  |
| ATOM-720.4-001 | 720.4 | Omen normal characteristics in non-stack zones | D4 | S9b |  |
| ATOM-720.5-001 | 720.5 | Omen alternative name choice | D4 + D18 | S9b |  |
| ATOM-724.4-001 | 724.4 | Monarch transfer on player exit (active player gets it) | D15 | S9b |  |
| ATOM-724.4-002 | 724.4 | Monarch fallback to next player if active player leaves | D15 | S9b |  |
| ATOM-725.2-001 | 725.2 | Initiative upkeep venture trigger | D15 | S9b |  |
| ATOM-725.2-003 | 725.2 | Initiative take → venture trigger | D15 | S9b |  |
| ATOM-725.4-001 | 725.4 | Initiative transfer on player exit | D15 | S9b |  |
| ATOM-725.5-001 | 725.5 | Initiative re-take still triggers venture | D15 | S9b |  |
| ATOM-729.2-001 | 729.2 | Merge action: component stacking | NEW — Merge action | S9b |  |
| ATOM-729.2a-001 | 729.2a | Topmost component characteristics | NEW — Merge top chars | S9b |  |
| ATOM-729.2a-002 | 729.2a | Ability aggregation from all components | (same) | S9b |  |
| ATOM-729.2b-001 | 729.2b | Merge does not trigger ETB | NEW — Merge no ETB | S9b |  |
| ATOM-729.2c-001 | 729.2c | Merged permanent identity continuity | NEW — Merge identity | S9b |  |
| ATOM-729.2d-001 | 729.2d | Merged permanent token status from top | NEW — Merge token | S9b |  |
| ATOM-729.2e-001 | 729.2e | Merged permanent face status from top | NEW — Merge face | S9b |  |
| ATOM-729.2f-001 | 729.2f | Bulk face-down/up for all components | NEW — Merge bulk face | S9b |  |
| ATOM-729.2g-001 | 729.2g | Instant/sorcery component blocks face-up | NEW — Merge face lock | S9b |  |
| ATOM-729.2h-001 | 729.2h | Flip card in merged permanent | NEW — Merge flip | S9b |  |
| ATOM-729.2i-001 | 729.2i | DFC transform scoping in merged permanent | D14 | S9b |  |
| ATOM-729.2j-001 | 729.2j | DFC component prevents face-down | D14 | S9b |  |
| COMP-MERGE-LEAVE-001 | 729.2 + 729.3 + 729.3c | Merge → destroy → components separate → track all | Phase 9 | S9b | NEW |
| ATOM-729.3-001 | 729.3 | Component separation on leave battlefield | NEW — Merge separation | S9b |  |
| ATOM-729.3a-001 | 729.3a | Component ordering on graveyard/library | (same) | S9b |  |
| ATOM-729.3b-001 | 729.3b | Exile timestamp ordering for components | (same) | S9b |  |
| ATOM-729.3c-001 | 729.3c | Multi-object tracking for "return it" effects | (same) | S9b |  |
| ATOM-729.3d-001 | 729.3d | Replacement effect applies to all components | (same) | S9b |  |
| ATOM-729.3e-001 | 729.3e | "Card" replacement includes token components | (same) | S9b |  |
| ATOM-730.1-001 | 730.1 | Game starts with neither day nor night | D14 | S9b |  |
| ATOM-730.1-002 | 730.1 | Day/night permanence after initialization | D14 | S9b |  |
| ATOM-730.1a-001 | 730.1a | Day→night transition trigger fires | D14 + NEW | S9b |  |
| ATOM-730.1a-002 | 730.1a | Night→day transition trigger fires | D14 | S9b |  |
| ATOM-730.2-001 | 730.2 | Untap step day/night transition check | D14 | S9b |  |
| ATOM-730.2a-001 | 730.2a | Day→night: previous player cast 0 spells | D14 | S9b |  |
| ATOM-730.2a-002 | 730.2a | Day stays day: previous player cast ≥1 spell | D14 | S9b |  |
| ATOM-730.2b-001 | 730.2b | Night→day: previous player cast ≥2 spells | D14 | S9b |  |
| ATOM-730.2b-002 | 730.2b | Night stays night: previous player cast 0-1 spells | D14 | S9b |  |
| ATOM-730.2c-001 | 730.2c | Neither day nor night → check skipped | D14 | S9b |  |
| ATOM-731.1b-001 | 731.1b | Loop detection: engine identifies repeated game state | D26 | S9b |  |
| ATOM-731.2a-001 | 731.2a | Shortcut proposal validation | D26 | S9b |  |
| ATOM-731.2b-001 | 731.2b | Shortcut shortening (fewer iterations) | D26 | S9b |  |
| ATOM-731.2c-001 | 731.2c | Shortcut execution (bulk-apply net delta) | D26 | S9b |  |
| ATOM-731.3-001 | 731.3 | Fragmented loop: active player must choose differently | D26 | S9b |  |
| ATOM-731.4-001 | 731.4 | Mandatory-only loop = game is a draw | D26 | S9b | META-104.4b, META-104.4f |
| ATOM-731.5-001 | 731.5 | Loop-break actions limited to involved objects | D26 | S9b |  |
| ATOM-731.6-001 | 731.6 | Unless-clause loop: can't force [B] | D26 | S9b |  |
| ATOM-800.4a-001 | 800.4a | Player leaves: owned objects leave, control effects end | D24 | S10 | multiplayer, player-leaves |
| ATOM-800.4a-002 | 800.4a | Player leaves: controlled-not-owned objects exiled | D24 | S10 | multiplayer, player-leaves |
| ATOM-800.4b-001 | 800.4b | Can't create tokens/objects under left-game player's control | D24 | S10 | multiplayer, player-leaves |
| ATOM-800.4c-001 | 800.4c | Orphaned object exiled when control effect ends + default controller left | D24 | S10 | multiplayer, player-leaves |
| ATOM-800.4d-001 | 800.4d | Triggered ability of left-game player not put on stack | D24 | S10 | multiplayer, player-leaves |
| ATOM-800.4e-001 | 800.4e | Combat damage to left-game player not assigned | D24 | S10 | multiplayer, player-leaves |
| ATOM-800.4j-001 | 800.4j | Active player leaves mid-turn: turn continues, priority passes | D24 | S10 | multiplayer, player-leaves |
| COMP-800-PLAYER-LEAVES-COMMANDER-001 | 800.4a+903.10a | Player leaves: cleanup + commander damage persistence | Phase 9 | S10 | D24 |
| ATOM-800.6-001 | 800.6 | Multiplayer free mulligan (first doesn't count) | D12 | S10 | multiplayer, mulligan |
| ATOM-800.7-001 | 800.7 | Starting player draws in multiplayer (not 2HG) | NEW-S10-01 | S10 | multiplayer, draw |
| ATOM-802.2-001 | 802.2 | All opponents are defending players in combat | D7 | S10 | multiplayer, combat |
| ATOM-802.2a-001 | 802.2a | "Defending player" on attacking creature = player it attacks | D7 | S10 | multiplayer, combat |
| ATOM-802.2a-002 | 802.2a | General "defending player" = any player being attacked | D7 | S10 | multiplayer, combat |
| ATOM-802.3-001 | 802.3 | Each attacker individually chooses attack target | D7 | S10 | multiplayer, combat |
| ATOM-802.4-001 | 802.4 | Blockers declared in APNAP order | D7 | S10 | multiplayer, combat, APNAP |
| ATOM-802.5-001 | 802.5 | Combat damage assigned in APNAP order | D7 | S10 | multiplayer, combat, APNAP |
| ATOM-903.2-001 | 903.2 | Commander default = FFA + attack multiple + no range | D7 | S10 | commander, setup |
| ATOM-903.3-001 | 903.3 | Commander designation: valid types, persists across zones | NEW-S10-02 | S10 | commander, designation |
| ATOM-903.3-002 | 903.3 | Copy of commander is NOT a commander | NEW-S10-02 | S10 | commander, designation |
| ATOM-903.3-003 | 903.3 | Face-down commander retains commander designation | NEW-S10-02 | S10 | commander, designation |
| ATOM-903.3d-001 | 903.3d | "Control your commander" true when commander on battlefield | NEW-S10-02 | S10 | commander, reference |
| ATOM-903.3d-002 | 903.3d | "Control your commander" false when commander in command zone | NEW-S10-02 | S10 | commander, reference |
| ATOM-903.3d-003 | 903.3d | "Target commander" targets permanent that is a commander | NEW-S10-02 | S10 | commander, reference |
| ATOM-903.4-001 | 903.4 | Color identity: mana symbols in rules text contribute | NEW-S10-03 | S10 | commander, color-identity |
| ATOM-903.4-002 | 903.4 | Color identity: color indicator contributes | NEW-S10-03 | S10 | commander, color-identity |
| ATOM-903.4c-001 | 903.4c | Color identity: reminder text ignored | NEW-S10-03 | S10 | commander, color-identity |
| ATOM-903.4d-001 | 903.4d | Color identity: DFC back face included | NEW-S10-03 | S10 | commander, color-identity |
| ATOM-903.5a-001 | 903.5a | Deck must be exactly 100 cards | NEW-S10-04 | S10 | commander, deck-validation |
| ATOM-903.5b-001 | 903.5b | Singleton rule (except basic lands + "any number" cards) | NEW-S10-04 | S10 | commander, deck-validation |
| ATOM-903.5c-001 | 903.5c | Card color identity ⊆ commander color identity | NEW-S10-04 | S10 | commander, deck-validation |
| ATOM-903.5d-001 | 903.5d | Basic land type restricted by commander color identity | NEW-S10-04 | S10 | commander, deck-validation |
| ATOM-903.6-001 | 903.6 | Commander starts in command zone, library = 99 | NEW-S10-05 | S10 | commander, setup |
| COMP-903-FULL-GAME-001 | 903.6+903.7+903.8+903.10a | Full Commander lifecycle: setup → cast → damage → win | Phase 9 | S10 | D7+T16 |
| ATOM-903.7-001 | 903.7 | Starting life = 40, hand = 7 | NEW-S10-05 | S10 | commander, setup |
| ATOM-903.8-001 | 903.8 | Cast commander from command zone, first cast no tax | NEW-S10-06 | S10 | commander, casting |
| ATOM-903.8-002 | 903.8 | Commander tax: second cast = +{2} | NEW-S10-07 | S10 | commander, tax |
| ATOM-903.8-003 | 903.8 | Commander tax: third cast = +{4} | NEW-S10-07 | S10 | commander, tax |
| ATOM-903.8-004 | 903.8 | Commander tax: only from command zone, not graveyard | NEW-S10-07 | S10 | commander, tax |
| COMP-903-TAX-AND-RETURN-001 | 903.8+903.9a | Commander dies → SBA return → recast with tax | Phase 9 | S10 | NEW-S10-11 |
| ATOM-903.9a-001 | 903.9a | Commander in graveyard → may SBA to command zone | NEW-S10-08 | S10 | commander, zone-return |
| ATOM-903.9a-002 | 903.9a | Commander in exile → may SBA to command zone | NEW-S10-08 | S10 | commander, zone-return |
| ATOM-903.9b-001 | 903.9b | Commander would go to hand → replacement to command zone | NEW-S10-09 | S10 | commander, zone-return |
| ATOM-903.9b-002 | 903.9b | Commander would go to library → replacement to command zone | NEW-S10-09 | S10 | commander, zone-return |
| COMP-903-BOUNCE-REPLACEMENT-001 | 903.9b+903.8 | Bounce → replacement to CZ → recast with tax | Phase 9 | S10 | NEW-S10-11 |
| ATOM-903.10a-001 | 903.10a | 21+ combat damage from one commander → lose (SBA) | T16 | S10 | commander, damage |
| ATOM-903.10a-002 | 903.10a | Commander damage tracked per-commander, not total | T16 | S10 | commander, damage |
| ATOM-903.10a-003 | 903.10a | Only combat damage counts, not non-combat | T16 | S10 | commander, damage |
| ATOM-903.10a-004 | 903.10a | Commander damage persists across zone changes | T16 | S10 | commander, damage |
| ATOM-903.10a-005 | 903.10a | Partner commanders: damage tracked independently per commander | T16+NEW-S10-10 | S10 | commander, damage, partner |

---

## Post-v1

**3 entries**

| ID | Rule | Summary | Ticket | Session | Tags |
|----|------|---------|--------|---------|------|
| ATOM-104.4b-001 | 104.4b | Mandatory loop → draw | NEW-CH1-005 | S1 |  |
| ATOM-112.4-001 | 112.4 | Characteristic changes on spell persist on permanent | Post-v1 (D16) | S1 |  |
| ATOM-703.4n-001 | 703.4n | Cleanup discard TBA — discard to max hand size | ALREADY-IMPLEMENTED | S9a |  |

---

## Cross-cutting

**1 entries**

| ID | Rule | Summary | Ticket | Session | Tags |
|----|------|---------|--------|---------|------|
| ATOM-101.2a-001 | 101.2a | "Can't" overrides "can" | META deferred | S1 |  |

---

## DEFERRED

**3 entries**

| ID | Rule | Summary | Ticket | Session | Tags |
|----|------|---------|--------|---------|------|
| ATOM-310.4 | 310.4 | Defense only for battles | — | S3 | battle, boundary |
| ATOM-601.2b-005 | 601.2b | Hybrid mana choice recorded | Hybrid | S5 |  |
| ATOM-601.2b-006 | 601.2b | Phyrexian mana choice recorded | Phyrexian | S5 |  |

---
