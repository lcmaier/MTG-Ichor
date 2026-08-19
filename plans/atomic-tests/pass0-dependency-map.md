# Pass 0 — Dependency Map & Merge Analysis

> Generated from `merge-input-compressed.md`, `roadmap.md`, and `implementation-plan-final.md`.
> Purpose: Pre-merge analysis for atomic test consolidation.

---

## 1. META → Concrete Mapping Table

Cross-cutting principles from all sessions, mapped to the specific ATOM/COMP tests and tickets that make them concrete.

> **Classification note:** Session summaries tagged many items as "META" that are actually SHARED-BEHAVIOR cluster notes (implementation sharing between keywords), single-keyword implementation notes, or architecture concerns. This table includes ONLY true cross-cutting META rules — principles that spawn concrete tests in multiple independent systems. Cluster notes are in Section 3. Single-keyword notes are in the compressed input's Section 4 and don't need separate tracking here.

| META ID | Principle | Concrete Tests | Tickets / Phases |
|---------|-----------|----------------|------------------|
| **META-101.1** (Card Text Overrides Rules) | Card text takes precedence over rules | Deferred to per-card/keyword sessions (702.x, 604, 609, 613, 614) | Per-card implementation |
| **META-101.2** ("Can't" Overrides "Can") | "Can't" wins over "can" | ATOM-614.17-001, ATOM-614.17a-001, ATOM-614.17b-001, ATOM-614.17c-001, ATOM-614.17d-001, ATOM-615.12-001, ATOM-615.12a-001 | Phase 6 (NEW-614.17, NEW-615.12) |
| **META-101.3** (Impossible Instructions Ignored) | Impossible instructions are no-ops | ATOM-609.3-001, ATOM-609.3-002 | Phase 8 (NEW-609.3) |
| **META-107.2** (Mana Symbol Ordering) | Canonical ordering — pure definitional | None (naming convention only) | N/A |
| **META-113.6b** (Zone-Activated Ability Pattern) | Pattern descriptor, not testable itself | Per-keyword: Cycling (702.29a), Unearth (702.84), Scavenge (702.97), Channel, etc. | Phase 8 (per keyword) |
| **META-300.2** (Multi-Type Objects) | Multi-type objects combine aspects of all types | Implicit in `is_creature`, `is_artifact`, `has_card_type` across targeting, SBAs, L4, combat, zone guards | Cross-cutting — verified by type predicate tests |
| **META-EPOCH-STAMP** (400.7) | ObjectId persists across zones; epoch-stamp for "new object" semantics | ATOM-400.7-001 (session 4, ticket 4), COMP-ZONE-TRANSITION-001 | Phase 5 (L18 LKI), zone infrastructure |
| **META-TRIGGER-TIMING** (508.2a) | Attack/block triggers snapshot at declaration time | ATOM-508.1-series (already impl), future trigger tests in Phase 7 | Phase 7 |
| **META-COMBAT-REQUIREMENTS-SOLVER** (508.1d, 509.1c) | Constraint satisfaction for attack/block requirements | T21d (combat requirements solver), ATOM-508.1d/509.1c series | Phase 5-Pre (T21d) |
| **META-MANA-ABILITY-WINDOWS** (508.1i) | Every "pay costs" step must open mana ability window | ATOM-605.3a-001/002/003 (impl), ATOM-508.1i (combat costs) | Cross-cutting: T18 (casting), T21b (combat) |
| **META-DP-ORDERING-CONSOLIDATION** (401.4, 404.3) | Single `choose_ordering` DP method for all simultaneous ordering | ATOM-401.4 (session 4, ticket 11), ATOM-404.3 (session 4, ticket 16) | Architecture decision — implement with Phase 7 trigger ordering |
| **META-GAMESTATE-SNAPSHOT** (session 5) | Casting rollback requires GameState snapshot before 601.2a | ATOM-601.2-series, also loop detection (731) | T18 (casting pipeline) |
| **META-CAST-PERMISSION-LAYERS** (601.3) | Single `can_begin_casting()` checking timing, prohibitions, flash, zone permissions | ATOM-601.3a–f (D17, Phase 8) | D17, L15 (CantCastSpells) |
| **META-MULTI-CONDITION-TRIGGERS** (session 5) | Per-turn event tracking for multi-condition triggers | `TurnEventLog` architecture needed before Phase 7 | Phase 7 architecture |
| **META-HIDDEN-ZONE-TRIGGER-COMPLEXITY** (session 5) | Trigger checking must happen AFTER all replacements + zone changes finalize | ATOM-603.6-series (Phase 7) | Phase 7 |
| **META-TWO-TIER-TRIGGER-STACKING** (session 5) | Classify triggers-on-trigger vs triggers-on-event; two-pass stacking | ATOM-603.3b-001/002 | Phase 7 |
| **META-LINKED-ABILITY-STORAGE** (session 5) | `linked_data: HashMap<AbilityId, LinkedAbilityData>` per permanent. Also covers Session 8's LINKED-ABILITY-PATTERN (Devour/Exploit/Tribute/Evolve). | ATOM-607.1-001, ATOM-607.2a-001/002, ATOM-607.2d-001/002 | T20 (linked abilities) |
| **META-7B-01** (Unified Evasion Framework) | `EvasionRestriction` + `BlockerFilter` enum | NEW-3 (Phase 8), affects Flying, Shadow, Fear, Intimidate, Horsemanship, Landwalk, Skulk, Daunt, Menace | Phase 8 |
| **META-7B-02** (ProtectionQuality Enum) | Centralized `matches_quality()` for hexproof-from + protection. Includes Protection-from-Everything as `ProtectionQuality::Everything` variant. | ATOM-702.16a–f series, ATOM-702.16j-001, ATOM-702.11c/d/e series | T22 (Phase 5-Pre), Phase 8 expansions |
| **META-7B-03** (Copy-Spell vs Copy-Card) | Pattern A (Storm), B (exile copy), C (Fork) — `copy_spell_on_stack()` vs `create_card_copy()` | ATOM-702.40a-001 (Storm), ATOM-700.2g-001 (spell copy modes) | Phase 7 (D19), Phase 8 |
| **META-7B-04** (Unified Trample DP) | `TrampleContext` struct for all trample variants (normal, over-PW, over-battle) | NEW-4, ATOM-702.19c/e/f | Phase 8 |
| **META-DECLARED-VS-ENTERS-ATTACKING** (session 8) | Engine must track how a creature became attacking (declared vs entered) | Affects Exalted, Melee, "attacks alone" triggers, Myriad tokens | Phase 7/8 |
| **META-MID-RESOLUTION-STATIC-CHECKS** (session 8) | Ascend/city's blessing checks between sequential effects within resolving spell | Architecture concern — not same as SBAs | Phase 8 |
| **META-PRINTED-VS-GRANTED** (session 8) | Backup grants only printed abilities; copy copies only copiable values. Read `CardData` for printed, not `compute_characteristics()`. | Affects Backup, Copy (707.2), any "grants abilities" mechanic | Phase 8 |

---

## 2. Cross-Session Duplicates Table

ATOMs or rules that appear in multiple sessions with overlapping scope. These need dedup during merge.

| Rule / ATOM | Session(s) | Overlap Description | Resolution |
|-------------|------------|---------------------|------------|
| **105.2** (Color from mana cost) | S1 (ATOM-105.2-002/003), S2 (ATOM-202.2-001/002) | S1 covers color derivation generally; S2 covers it via rule 202.2 with more specificity | Keep S2 ATOMs (more specific), mark S1 as superseded |
| **105.2b** (Colorless from generic cost) | S1 (ATOM-105.2b-001/002), S2 (ATOM-202.2b-001/002) | Same concept — colorless from no colored symbols | Keep S2 ATOMs, mark S1 as superseded |
| **202.2e + 105.2** (Color indicator) | S1 (ATOM-105.2-003), S2 (ATOM-202.2e-001) | Color indicator overrides | S2 is more precise; S1 is general statement |
| **107.3g/107.3j** (X in mana value) | S1 (ATOM-107.3g-001, ATOM-107.3j-001), S2 (ATOM-202.3e-001/002) | X=0 off stack, X=chosen on stack | S1 and S2 test same concept from different rule refs; keep both but cross-reference |
| **205.4a** (Supertype enum) | S2 (ATOM-205.4a-001), S3 (implicit in 300.1-001) | Enum completeness | S2 is canonical; S3 is a subset |
| **301.5** (Equipment attachment) | S1 (implicit), S3 (ATOM-301.5-001/002), S7a (ATOM-701.3a-001/002) | Legal/illegal attachment | S3 covers Equipment-specific; S7a covers Attach keyword action. Different rules, complementary |
| **305.6/305.7** (Land types + intrinsic mana) | S1 (ATOM-305.6-001), S3 (ATOM-305.6-002, ATOM-305.7-series) | Intrinsic mana from land types | S3 is more detailed; S1 is already-implemented |
| **306.5** (Loyalty only for PWs) | S2 (ATOM-209.1-001), S3 (ATOM-306.5-001) | Both cover PW loyalty | S3 is boundary-def; S2 is ETB behavior. Complementary |
| **602.5a** (Summoning sickness for activated abilities) | S1 (ATOM-107.5-002, ATOM-107.6-002), S5 (ATOM-602.5a-001/002) | Tap/untap cost sickness | S5 is canonical rule ref; S1 covers specific symbol cases |
| **608.2b** (Fizzle) | S1 (implicit), S5 (ATOM-608.2b-001 through 005) | Target illegality | S5 is comprehensive |
| **611.2a** (Continuous effect duration) | S6 (ATOM-611.2a-001/002), also covered by L02/L07 in implementation plan | Duration expiry | S6 ATOMs are the tests; L02/L07 are the implementation tickets |
| **613.x** (Layer system) | S6 (comprehensive ATOM-613.x series), also S2 (ATOM-208.x for P/T layers) | Layer ordering and P/T | S6 is the canonical layer test suite; S2 P/T ATOMs are pre-layer behavioral specs |
| **702.15** (Lifelink) | S1 (ATOM-120.3f-001), S7b (ATOM-702.15c-001, ATOM-702.15d-001) | Lifelink basics vs LKI/zone variants | S1 is already-implemented basic; S7b extends to LKI and non-battlefield |
| **702.16** (Protection) | S1 (implicit in T22), S7b (ATOM-702.16a through 702.16p) | Protection targeting, SBA, evasion, damage | S7b is comprehensive; S1 just references T22 |
| **T14** (Legend rule + PW loyalty) | S1 (ATOM-122.1e-001), S2 (ATOM-209.1-001, ATOM-205.4d-001), S3 (ATOM-306.5-001) | PW loyalty SBA, legend rule | All feed into T14; no actual test duplication |
| **T15/T15b** (Aura/Equip SBAs) | S3 (ATOM-301.5-series, ATOM-303.4-series), S5 (ATOM-608.3b/c), S7b (ATOM-702.5a/6a) | Attachment legality from different angles | S3 covers type-level rules; S5 covers resolution; S7b covers keyword abilities. Complementary |
| **T18** (Casting pipeline) | S1 (ATOM-107.3a, ATOM-118.x), S5 (ATOM-601.2-series, ATOM-608.2-series) | Casting costs, pipeline steps | S5 is the canonical 601.2 pipeline; S1 covers cost mechanics. Complementary |

---

## 3. Shared Mechanism Clusters

Groups of keywords/rules that share implementation infrastructure. These should be implemented together or in sequence to avoid redundant work.

### Cluster A: Alternative Cost Framework
**Shared infra:** T17 alt/add cost types → T18 601.2 pipeline
| Keyword | Rule | Alt/Add | Phase | Notes |
|---------|------|---------|-------|-------|
| Flashback | 702.34 | Alternative | 8 | GY cast + exile |
| Escape | 702.138 | Alternative | 8 | GY cast + exile cards |
| Dash | 702.109 | Alternative | 8 | Haste + delayed return |
| Blitz | 702.152 | Alternative | 8 | Haste + dies-draw |
| Spectacle | 702.137 | Alternative (conditional) | 8 | Damage condition |
| Surge | 702.117 | Alternative (conditional) | 8 | Player-cast condition |
| Prowl | 702.76 | Alternative (conditional) | 8 | Damage-by-type condition |
| Emerge | 702.119 | Alternative + sacrifice | 8 | MV reduction |
| Overload | 702.96 | Alternative + text-change | 8 | L12 text walker |
| Evoke | 702.74 | Alternative + ETB sac | 8 | T17 |
| Bestow | 702.103 | Alternative + type change | 9 | Complex |
| Freerunning | 702.173 | Alternative (conditional) | 8 | Commander cross-ref |
| Sneak | 702.190 | Alternative + bounce | 8 | Combat timing |
| Harmonize | 702.180 | Alternative + tap reduction | 8 | GY cast |
| Warp | 702.185 | Alternative + delayed exile | 8 | Future free cast |
| Mayhem | 702.187 | Alternative (conditional) | 8 | Discard condition GY cast |

### Cluster B: Additional Cost Framework
| Keyword | Rule | Type | Phase |
|---------|------|------|-------|
| Kicker/Multikicker | 702.33 | Additional (optional, repeatable) | 8 |
| Buyback | 702.27 | Additional + hand return | 8 |
| Escalate | 702.120 | Per-mode additional | 8 |
| Casualty | 702.153 | Sacrifice + copy | 8 |
| Bargain | 702.166 | Optional sacrifice | 8 |
| Spree | 702.172 | Cost-selects-mode | 8 |
| Tiered | 702.183 | Modal + per-mode cost | 8 |
| Entwine | 702.42 | Additional to enable all modes | 8 |
| Replicate | 702.56 | Additional + copies | 8 |
| Conspire | 702.78 | Tap-as-cost + copy | 8 |
| Offspring | 702.175 | Additional + 1/1 token copy | 8 |
| Strive | 702.107 | Per-extra-target additional | 8 |
| Crew | 702.122 | Tap creatures (power ≥ N) | 8 |
| Saddle | 702.171 | Tap creatures (power ≥ N) | 8 |

> **Cluster note (crew-saddle):** Crew and Saddle share `tap_creatures_for_power(n, filter)` infrastructure. Different post-activation effects but identical cost mechanism.

### Cluster C: Cost Reduction by Resource
Shared pattern: reduce generic (or colored) cost by consuming a resource during casting. Sub-grouped by resource type since implementation differs.

**C1: Tap-to-reduce** — Tap permanents to reduce cost. Share `tap_to_reduce_cost(filter, reduction_type)`.
| Keyword | Rule | What Taps | Reduction | Phase |
|---------|------|-----------|-----------|-------|
| Convoke | 702.51 | Creatures | Generic or colored | 8 |
| Improvise | 702.126 | Artifacts | Generic only | 8 |
| Waterbend | 701.67 | Artifacts/creatures | Generic portion | 8 |

**C2: Count-to-reduce** — Passive reduction based on board state count. No activation cost.
| Keyword | Rule | Counted | Reduction | Phase |
|---------|------|---------|-----------|-------|
| Affinity | 702.41 | Permanents with property | Generic, 1-per-permanent | 8 |

**C3: Exile-to-reduce** — Zone-change payment (GY → exile) to reduce cost.
| Keyword | Rule | Resource | Reduction | Phase |
|---------|------|----------|-----------|-------|
| Delve | 702.66 | GY cards exiled | Generic, 1-per-card | 8 |

### Cluster D: ETB Trigger + Store + Read (Linked Abilities)
| Keyword | Rule | What's Stored | Phase |
|---------|------|---------------|-------|
| Devour | 702.82 | Sacrifice count | 8 |
| Exploit | 702.110 | Sacrificed creature | 8 |
| Tribute | 702.104 | Counters placed or not | 8 |
| Evolve | 702.100 | P/T comparison result | 8 |
| Fabricate | 702.123 | Counter-or-token choice | 8 |
| Ravenous | 702.156 | X-based counter count | 8 |
| Modular | 702.43 | ETB counter count + death trigger transfer | 8 |

### Cluster E: Dies/LTB Trigger + Conditional Return
| Keyword | Rule | Condition | Counter | Phase |
|---------|------|-----------|---------|-------|
| Undying | 702.93 | No +1/+1 counters | +1/+1 | 8 |
| Persist | 702.79 | No -1/-1 counters | -1/-1 | 8 |
| Encore | 702.141 | GY activated | Per-opponent tokens | 8+9 |

### Cluster F: Face-Down Casting
| Keyword | Rule | Cost | Extra | Phase |
|---------|------|------|-------|-------|
| Morph | 702.37 | {3} face-down | Face-up for morph cost | 8 |
| Megamorph | 702.37 | {3} face-down | +1/+1 counter on face-up | 8 |
| Disguise | 702.168 | {3} face-down | Ward {2} | 8 |
| Manifest | 701.40 | N/A (from library) | Face-up for mana cost | 8 |
| Cloak | 701.58 | N/A (from library) | Ward {2} on face-down | 8 |

### Cluster G: Combat Attack Triggers
| Keyword | Rule | Trigger | Effect | Phase |
|---------|------|---------|--------|-------|
| Exalted | 702.83 | Solo attack | +1/+1 | 8 |
| Battle Cry | 702.91 | Attack | +1/+0 to other attackers | 8 |
| Melee | 702.121 | Attack | Per-opponent bonus | 9 |
| Dethrone | 702.105 | Attack highest-life player | +1/+1 counter | 8 |
| Myriad | 702.116 | Attack | Per-opponent token copies | 9 |
| Annihilator | 702.86 | Attack | Forced sacrifice | 8 |
| Mobilize | 702.181 | Attack | Warrior tokens | 8 |
| Firebending | 702.189 | Attack | Mana + combat persistence | 8 |
| Enlist | 702.154 | Attack declaration | Tap non-attacker for power bonus | 8 |
| Training | 702.149 | Attack with higher-power co-attacker | +1/+1 counter | 8 |

### Cluster H: Evasion Keywords (META-7B-01 Unified Framework)
| Keyword | Rule | Filter | Phase |
|---------|------|--------|-------|
| Flying | 702.9 | HasKeyword(Flying) or HasKeyword(Reach) | Already impl |
| Shadow | 702.28 | Bidirectional(Shadow) | 8 |
| Fear | 702.36 | ArtifactOrColor(Black) | 8 |
| Intimidate | 702.13 | ArtifactOrSharesColor | 8 |
| Horsemanship | 702.31 | HasKeyword(Horsemanship) | 8 |
| Landwalk | 702.14 | DefenderControlsLand(type) | 8 |
| Skulk | 702.118 | PowerGreaterThan(attacker.power) | 8 |
| Menace | 702.111 | MinBlockers(2) | 8 (enforcement needed) |

### Cluster I: GY-Activated Abilities
| Keyword | Rule | Cost | Effect | Phase |
|---------|------|------|--------|-------|
| Unearth | 702.84 | Mana | Return + haste + delayed exile | 8 |
| Embalm | 702.128 | Mana + exile | Modified token copy | 8 |
| Eternalize | 702.129 | Mana + exile | 4/4 token copy | 8 |
| Scavenge | 702.97 | Mana + exile | Counters on target | 8 |
| Encore | 702.141 | Mana + exile | Per-opponent tokens | 8+9 |
| Jump-Start | 702.133 | Mana + discard | GY cast + exile | 8 |
| Retrace | 702.81 | Mana + discard land | GY cast | 8 |
| Flashback | 702.34 | Alt cost | GY cast + exile | 8 |
| Escape | 702.138 | Alt cost + exile cards | GY cast | 8 |
| Aftermath | 702.127 | Normal cost | GY-only second half | 8 |

> **Cluster note:** Flashback and Escape also appear in Cluster A (Alternative Cost Framework). They live in both clusters — GY-activated for the zone permission, alt-cost for the cost pipeline.

### Cluster J: DFC / Transform Infrastructure (Phase 9)
| System | Rules | Dependencies |
|--------|-------|-------------|
| DFC core | 712.1–712.21 | Zone-dependent characteristics, copiable values |
| Transform | 701.27 | DFC core |
| Convert | 701.28 | DFC core (follows transform rules) |
| Meld | 701.42, 712.4 | DFC core + multi-card permanent |
| Daybound/Nightbound | 702.145, 730.x | DFC core + day/night system |
| Disturb | 702.146 | DFC core + GY cast |
| More Than Meets the Eye | 702.162 | DFC core + transformed cast |
| Craft | 702.167 | DFC core + exile materials |
| Modal DFC | 712.11b, 712.12 | DFC core + face selection |

### Cluster K: Replacement Effect Framework (Phase 6)
| Category | Rules | Test Count |
|----------|-------|------------|
| Core replacement | 614.1a–d, 614.4–614.7 | ~8 ATOMs |
| Regeneration | 614.8, 701.19a–c | ~5 ATOMs |
| Damage redirection | 614.9 | 1 ATOM |
| Skip effects | 614.10–614.10b | ~4 ATOMs |
| Draw replacement | 614.11–614.11b | ~4 ATOMs |
| ETB look-ahead | 614.12–614.13b | ~6 ATOMs |
| Self-replacement priority | 614.15–614.16 | ~3 ATOMs |
| "Can't" effects | 614.17–614.17d | ~5 ATOMs |
| Prevention core | 615.4–615.12a | ~12 ATOMs |
| Multiple replacement ordering | 616.1–616.2 | ~7 ATOMs |
| **Total Phase 6 framework** | | **~55 ATOMs** |

### Cluster L: Text-Changing Effects
Shared pattern: modify spell text during casting/resolution. Cross-ref rule 612. Need shared `TextModification` infrastructure.
| Keyword | Rule | Modification | Phase |
|---------|------|-------------|-------|
| Overload | 702.96 | "target" → "each" (alt cost) | 8 |
| Splice | 702.47 | Add rules text from hand card during casting | 8 |
| Cleave | 702.148 | Remove bracketed text (alt cost) | 8 |

### Cluster M: Combat Swap (Ninjutsu Pattern)
Shared pattern: swap a creature during combat.
| Keyword | Rule | Timing | Phase |
|---------|------|--------|-------|
| Ninjutsu | 702.49 | Unblocked attacker → return, put from hand | 8 |
| Sneak | 702.190 | Declare-blockers alt cost, bounce unblocked | 8 |

### Cluster N: ETB Token + Auto-Attach
Shared pattern: ETB creates a token and automatically attaches an Equipment.
| Keyword | Rule | Token | Phase |
|---------|------|-------|-------|
| For Mirrodin! | 702.163 | Equipment becomes creature + 1/1 Rebel | 8 |
| Job Select | 702.182 | 1/1 Hero token + auto-attach | 8 |

---

## 4. Recommended Merge-Half Split

The merge-input contains ~1700 ATOMs, ~62 BOUNDARY-DEFs, ~69 COMPs, and ~25 true META entries across 12 sessions. The recommended split for two merge passes:

### Merge-Half A: Foundation + Layers + Core Mechanics (Sessions 1–6)
**Scope:** Rules 100–122, 200–210, 300–310, 400–406, 500–514, 601–616
**ATOMs:** ~1050 (sessions 1–6)
**Focus:**
- Game structure, costs, mana, life, damage, draw, counters
- Types, subtypes, supertypes, characteristics
- Card types (artifact, creature, enchantment, instant, land, planeswalker, sorcery)
- Zones, zone transitions, epoch-stamp
- Turn structure, combat, priority
- Casting pipeline (601.2), resolution (608), ability classification (602–607)
- Layer system (611–613) — all 7 layers + dependency + timestamp
- Replacement effects framework (614–616)
- Prevention effects (615)
- Already-implemented baseline validation

**Key deliverables after Merge-Half A:**
- Complete test index for Phases 5-Pre through 6
- All BOUNDARY-DEFs for core rules
- All META entries for architecture decisions
- Layer system test matrix (613.x)
- Replacement/prevention test matrix (614–616)

### Merge-Half B: Keywords + Card Types + Formats (Sessions 7–10)
**Scope:** Rules 700–732, 800–903
**ATOMs:** ~650 (sessions 7a, 7b, 8, 9a, 9b, 10)
**Focus:**
- Keyword actions (700–701): modal, devotion, fight, scry, search, surveil, mill, etc.
- Evergreen keywords (702.x): all 190+ keyword abilities
- SBA extensions (703–704): saga, battle, role, speed, commander
- Copy effects (707), face-down (708), split cards (709), DFC (712)
- Sagas (714), Adventures (715), Class (716), Prototype (718), Cases (719)
- Omens (720), Stations (721), Player control (722), End-the-turn (723)
- Monarch/Initiative (724–725), Rad counters (727), Merge/Mutate (729)
- Day/Night (730), Loops (731)
- Multiplayer (800–802), Commander (903)
- DEFERRED master list items

**Key deliverables after Merge-Half B:**
- Complete keyword ability test matrix
- Shared-mechanism cluster cross-references
- DFC/Transform/Meld infrastructure test suite
- Commander format test suite
- Phase 8/9 implementation ordering

### Split Rationale
- **Half A** covers the *engine foundation* — rules that every card and every mechanic depends on. These must be solid before building keywords.
- **Half B** covers *breadth* — individual mechanics that are mostly independent of each other but depend on Half A's infrastructure.
- The split aligns with the implementation plan's Part 1 (data model + SBAs + casting) → Part 2 (layers) → Phase 6 (replacement) → Phase 7 (triggers) → Phase 8 (keywords) progression.
- Sessions 1–6 are heavily cross-referenced and interdependent (layers reference types, types reference characteristics, characteristics reference costs). Sessions 7–10 are more independent per-keyword.

---

## 5. Classification Totals (from merge-input Section 7)

| Category | Count |
|----------|-------|
| **ATOM tests** | ~1688 |
| **BOUNDARY-DEF entries** | ~62 |
| **COMP tests** | ~69 |
| **META entries** | ~25 (true cross-cutting; ~15 additional reclassified as cluster/impl notes) |
| **NEW tickets** | ~345 |
| **ALREADY-IMPLEMENTED** | ~198 rules |
| **DEFERRED** | ~330+ rules (Section 8) |
| **OUT-OF-SCOPE** | ~88 rule ranges (Section 9) |

---

## 6. Phase → Test Volume Estimates

| Phase | ATOM Count (approx) | Key Systems |
|-------|---------------------|-------------|
| ALREADY-IMPL | ~150 | Core game loop, priority, combat basics, mana, life, damage, draw |
| Phase 5-Pre | ~180 | T01–T22: data model, SBAs, casting pipeline, targeting, combat evasion |
| Phase 5 Layers | ~120 | L01–L21: continuous effects, 7 layers, dependency, oracle routing |
| Phase 6 | ~100 | Replacement effects (614), prevention (615), ordering (616), ETB mods |
| Phase 7 | ~150 | Triggered abilities (603), delayed triggers, state triggers, trigger stacking |
| Phase 8 | ~600+ | Keywords (702.x), keyword actions (701.x), tokens, emblems, advanced rules |
| Phase 9 | ~250+ | DFC, split cards, face-down, sagas, adventures, prototype, commander, multiplayer |
| Post-v1 | ~30 | Restart (726), subgames (728), niche |

---

## 7. Critical Cross-References

### Implementation Plan Tickets → Merge-Input Sections
| impl-plan Ticket | merge-input ATOMs (sample) | Section |
|-----------------|---------------------------|---------|
| T01 (counters) | ATOM-122.1a-001/002/003, ATOM-122.2-001, ATOM-122.3-001 | S1 |
| T14 (legend rule) | ATOM-205.4d-001, ATOM-209.1-001, ATOM-122.1e-001 | S1, S2 |
| T15/T15b (Aura/Equip) | ATOM-301.5-series, ATOM-303.4-series, ATOM-702.5a/6a | S3, S7b |
| T18 (casting) | ATOM-601.2-series, ATOM-608.2-series, ATOM-700.2-series | S5, S7a |
| T09 (summoning sickness) | ATOM-702.10b/c (Haste) | S7b |
| T20 (linked abilities) | ATOM-607.1-001, ATOM-607.2a-001/002, ATOM-607.2d-001/002 | S5 |
| T22 (duration/targeting) | ATOM-702.11b, ATOM-702.16b, ATOM-702.18a | S7b |
| L04 (EffectiveChars) | ATOM-208.2a/3/5, ATOM-302.4, ATOM-613.3/4 | S2, S3, S6 |
| L10 (Layers 4-5) | ATOM-202.2/3, ATOM-205.1a/b, ATOM-305.7 | S2, S3 |
| L14 (Dependency) | ATOM-613.8/8a/8b/8c | S6 |
| L17 (Tier 1 cards) | ATOM-613.8a-001 (Blood Moon+Urborg), COMP-305.7 | S3, S6 |

### Deferred Items (D-numbers) → Merge-Input Locations
| D# | Summary | merge-input Location |
|----|---------|---------------------|
| D1 | Layer 1 Copy Effects | S6: ATOM-613.1a-001, ATOM-613.2a-001 |
| D2 | Layer 3 Text-Changing | S6: ATOM-612.2-001, L12 |
| D4 | Aura/Equipment re-timestamp | S6: ATOM-613.7e-001 |
| D5 | Static ability timestamp | S6: ATOM-613.7a-001 |
| D7 | "For as long as" duration | S6: ATOM-611.2b-001 |
| D9 | "Until" zone-change effects | S6: ATOM-610.3-001 |
| D10 | Keyword counters in L6 | S6: ATOM-613.1f-002 |
| D14 | DFC infrastructure | S9a: ATOM-712.x series, S9b: ATOM-730.x |
| D15 | Monarch/Initiative | S9b: ATOM-724.x, ATOM-725.x |
| D17 | Cast legality look-ahead | S8 DEFERRED: ATOM-601.3a–f |
| D19 | Spell copying | S7a: ATOM-700.2g-001, S9a: copy rules |
| D24 | Player-leaves-game | S10: ATOM-800.4-series |
| D26 | Loop detection | S9b: ATOM-731.x series |

---

## 8. Architecture Decisions Required Before Implementation

| Decision | Needed By | Source |
|----------|-----------|--------|
| `TurnEventLog` structure for multi-condition triggers | Phase 7 | META-MULTI-CONDITION-TRIGGERS |
| `ObjectRef` with epoch-stamp for stale references | Phase 5-Pre (T22) | META-EPOCH-STAMP |
| `LinkedAbilityData` storage per permanent | Phase 5-Pre (T20) | META-LINKED-ABILITY-STORAGE |
| `EvasionRestriction` + `BlockerFilter` enum | Phase 8 | META-7B-01 |
| `ProtectionQuality` enum + `matches_quality()` | Phase 5-Pre (T22) | META-7B-02 |
| `TrampleContext` for unified trample DP | Phase 8 | META-7B-04 |
| Casting rollback via GameState snapshot | Phase 5-Pre (T18) | META-GAMESTATE-SNAPSHOT |
| `can_begin_casting()` unified permission check | Phase 8 (D17) | META-CAST-PERMISSION-LAYERS |
| Two-pass trigger stacking | Phase 7 | META-TWO-TIER-TRIGGER-STACKING |
| Trigger checking after all replacements finalize | Phase 7 | META-HIDDEN-ZONE-TRIGGER-COMPLEXITY |
| `choose_ordering` DP consolidation | Phase 7 | META-DP-ORDERING-CONSOLIDATION |
| Copy-spell vs copy-card distinction | Phase 7 (D19) | META-7B-03 |
| Zone-agnostic trigger scanner (scan ALL zones, not just battlefield) | Phase 7 | Alchemy audit Q7 + paper MTG (Bloodghast, Narcomoeba, emblems, suspend) |
| Splice as temporary resolution extension | Phase 8 | META-7B-07 |
| Menace enforcement in `validate_blockers` | Immediate (bug fix, not architectural) | Existing code — add min-blockers check |

---
