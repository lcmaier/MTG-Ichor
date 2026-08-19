# Session 4 Summary — Zones (Chapter 4) & Turn Structure (Chapter 5)

> **CR Sections:** 400–408, 500–514
> **Generated:** 2026-04-03
> **Source:** `session-4.md`

---

## ATOM Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-400.2-001 | 400.2 | Zone visibility classification (public vs hidden) | Phase 1 | NEW — zone visibility query | |
| ATOM-400.3-001 | 400.3 | Owner routing for library/graveyard/hand | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-400.4a-001 | 400.4a | Instant/sorcery can't enter battlefield | Phase 5-Pre | T21a | |
| ATOM-400.5-001 | 400.5 | Zone ordering preserved (Vec) | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-400.6-001 | 400.6 | Replacement effects applied before zone change | Phase 6 | NEW — replacement effects on zone transitions | |
| ATOM-400.6-002 | 400.6 | Contradictory replacement effects — controller chooses | Phase 6 | NEW — replacement effect conflict resolution | |
| ATOM-400.7-001 | 400.7 | New object identity on zone change (epoch-stamp) | Phase 5-Pre + Phase 7 | NEW — zone_change_epoch infrastructure | |
| ATOM-400.7-002 | 400.7 | Blink creates new object with no memory | Phase 8 (Phase 5-Pre infra) | NEW — blink identity reset | |
| ATOM-400.7-003 | 400.7 | Multiple simultaneous trackers see stale epoch | Phase 5-Pre | Same as 400.7-001 | |
| ATOM-400.7a-001 | 400.7a | Spell-to-permanent color-change effect continuity | Phase 5 | NEW — stack-to-permanent effect continuity | |
| ATOM-400.7a-002 | 400.7a | Text-changing effect on spell persists to permanent | Phase 5 | NEW — text-changing effect persistence | |
| ATOM-400.7b-001 | 400.7b | Static ability grants continue to permanent | Phase 5 | L06 | |
| ATOM-400.7c-001 | 400.7c | Prevention effect on spell continues to permanent | Phase 6 | NEW — prevention effect continuity | |
| ATOM-400.7c-002 | 400.7c | Prevention on source ≠ prevention on tokens from source | Phase 6 | NEW — prevention effect source identity | |
| ATOM-400.7d-001 | 400.7d | CastInfo carried to permanent (kicker check) | Phase 5-Pre | T21a | |
| ATOM-400.7e-001 | 400.7e | LTB trigger finds new object in public zone (Rancor) | Phase 7 | NEW — zone-change trigger object tracking | |
| ATOM-400.7e-002 | 400.7e | "From anywhere" triggers don't need 400.7e (Vigor) | Phase 7 | NEW — "from anywhere" trigger distinction | |
| ATOM-400.7f-001 | 400.7f | Aura LTB trigger finds aura in graveyard (simultaneous) | Phase 7 + Phase 5-Pre | NEW — Aura LTB simultaneous graveyard | |
| ATOM-400.7j-001 | 400.7j | Effect moves object to public zone, finds it | Phase 8 | NEW — effect self-referencing after zone change | |
| ATOM-400.8-001 | 400.8 | Re-exile creates new object identity (epoch bump) | Phase 8 | NEW — re-exile epoch bump | |
| ATOM-400.12-001 | 400.12 | Batch zone-to-zone move ("shuffle GY into library") | Phase 8 | NEW — batch zone move | |
| ATOM-401.2-001 | 401.2 | Library face-down; no public visibility | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-401.3-001 | 401.3 | Any player can count any library | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-401.4-001 | 401.4 | Multi-card library placement ordering via DP | Phase 8 | NEW — multi-card library ordering | |
| ATOM-401.5-001 | 401.5 | "Play with top card revealed" continuous effect | Phase 8 | NEW — top-card-revealed effect | |
| ATOM-401.5-002 | 401.5 | Top-of-library reveal freeze during casting | Phase 8 | NEW — reveal freeze during casting | |
| ATOM-401.5-003 | 401.5 | Reveal freeze during ability activation | Phase 8 | Same as 401.5-002 | |
| ATOM-401.7-001 | 401.7 | "Nth from top" fallback to bottom | Phase 8 | NEW — library positional insertion | |
| ATOM-402.2-001 | 402.2 | Max hand size cleanup discard | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-402.2-002 | 402.2 | No hand-size enforcement outside cleanup | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-402.3-001 | 402.3 | Own hand visible, opponent hand hidden | Phase 1 | NEW — hand visibility enforcement | |
| ATOM-402.3-002 | 402.3 | "Reveal" = all players see (Thoughtseize) | Phase 8 | NEW — reveal keyword implementation | |
| ATOM-402.3-003 | 402.3 | "Look at" = controller only (Gitaxian Probe) | Phase 8 | NEW — look-at keyword implementation | |
| ATOM-402.3-004 | 402.3 | Persistent "play with hand revealed" (Telepathy) | Phase 8 | NEW — persistent reveal effect | |
| ATOM-403.2-001 | 403.2 | Default scope is battlefield (destroy target creature) | Phase 2 | ALREADY-IMPLEMENTED | |
| ATOM-403.2-002 | 403.2 | Player-targeting can't target permanent | Phase 2 | ALREADY-IMPLEMENTED | |
| ATOM-403.2-003 | 403.2 | Graveyard-targeting can't target battlefield | Phase 2 | ALREADY-IMPLEMENTED | |
| ATOM-403.4-001 | 403.4 | ETB permanent is new object (no memory) | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-404.1-001 | 404.1 | Resolved spell goes on top of graveyard | Phase 2 | ALREADY-IMPLEMENTED | |
| ATOM-404.1-002 | 404.1 | Graveyards start empty | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-404.2-001 | 404.2 | Graveyard face-up, publicly visible | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-404.3-001 | 404.3 | Simultaneous graveyard ordering (auto-order) | Phase 8 | NEW — simultaneous graveyard ordering | |
| ATOM-404.3-002 | 404.3 | Simultaneous graveyard ordering (manual via DP) | Phase 8 | Same as 404.3-001 | |
| ATOM-405.2-001 | 405.2 | Stack LIFO ordering | Phase 2 | ALREADY-IMPLEMENTED | |
| ATOM-405.3-001 | 405.3 | APNAP simultaneous stack placement | Phase 7 | NEW — APNAP stack placement | |
| ATOM-405.3-002 | 405.3 | Player chooses order of own simultaneous triggers | Phase 7 | NEW — player trigger ordering via DP | |
| ATOM-405.4-001 | 405.4 | Spell retains card characteristics on stack | Phase 2 | ALREADY-IMPLEMENTED | |
| ATOM-405.4-002 | 405.4 | Ability on stack has no card characteristics | Phase 2 | ALREADY-IMPLEMENTED | |
| ATOM-405.4-003 | 405.4 | Spell controller = caster | Phase 2 | ALREADY-IMPLEMENTED | |
| ATOM-405.4-004 | 405.4 | Ability controller = activator (not owner) | Phase 2 + Phase 5 | ALREADY-IMPLEMENTED | |
| ATOM-405.4-005 | 405.4 | "Any player may activate" controller = activator | Phase 8 | NEW — any-player-activate controller | |
| ATOM-405.5-001 | 405.5 | All pass → top resolves | Phase 2 | ALREADY-IMPLEMENTED | |
| ATOM-405.5-002 | 405.5 | Empty stack + all pass → phase/step ends | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-405.6c-001 | 405.6c | Mana abilities resolve immediately | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-405.6c-002 | 405.6c | Mana abilities don't use stack (negative test) | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-405.6e-001 | 405.6e | TBAs don't use stack (draw step) | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-405.6f-001 | 405.6f | SBAs don't use stack; happen before priority | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-406.3-001 | 406.3 | Exiled cards face-up by default | Phase 8 | NEW — exile zone visibility | |
| ATOM-406.3-002 | 406.3 | Face-down exile hidden from all | Phase 8 | NEW — face-down exile tracking | |
| ATOM-408.2-001 | 408.2 | Emblems created in command zone | Phase 8 | NEW — emblem creation | |
| ATOM-500.1-001 | 500.1 | Five phases in fixed order every turn | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-500.2-001 | 500.2 | Phase/step ends only on empty stack + all pass | Phase 2 | ALREADY-IMPLEMENTED | |
| ATOM-500.3-001 | 500.3 | No-priority steps end after TBAs complete | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-500.4-001 | 500.4 | "Until" duration expires as step/phase begins | Phase 5-Pre | T22 | |
| ATOM-500.5-001 | 500.5 | End-of-step effect expiry + mana pool emptying | Phase 1 + Phase 5-Pre | ALREADY-IMPL (mana) + T22 | |
| ATOM-500.5a-001 | 500.5a | "Until end of combat" expires at end of combat phase | Phase 5-Pre | T22 | |
| ATOM-500.6-001 | 500.6 | "At the beginning of" triggers fire at step/phase start | Phase 7 | NEW — step/phase beginning trigger checking | |
| ATOM-502.3-001 | 502.3 | Untap all permanents simultaneously (TBA) | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-502.3-002 | 502.3 | "Doesn't untap" continuous effect prevents untap | Phase 5 | NEW — doesn't-untap effect filtering | |
| ATOM-502.3-003 | 502.3 | Constrained untap choice (Winter Orb pattern) | Phase 5 + Phase 8 | NEW — constrained untap via DP | |
| ATOM-502.4-001 | 502.4 | No priority during untap; triggers held until upkeep | Phase 7 | NEW — trigger hold-and-release | |
| ATOM-503.1-001 | 503.1 | Upkeep: no TBA, active player gets priority | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-503.1a-001 | 503.1a | Untap-step + upkeep triggers on stack before priority | Phase 7 | NEW — trigger batch at upkeep | |
| ATOM-504.1-001 | 504.1 | Draw step TBA (active player draws) | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-504.1-002 | 504.1 | Starting player skips first draw step | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-505.1-001 | 505.1 | Two main phases separated by combat | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-505.1a-001 | 505.1a | Only first main phase is precombat | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-505.1a-002 | 505.1a | Multiple postcombat main phases each trigger effects | Phase 9 + Phase 7 | NEW — multiple postcombat trigger firing | |
| ATOM-505.6a-001 | 505.6a | Sorcery-speed spells only during main phase | Phase 2 | ALREADY-IMPLEMENTED | |
| ATOM-505.6b-001 | 505.6b | Land play timing (main phase, stack empty, has priority) | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-505.6b-002 | 505.6b | Land play doesn't use stack | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-505.6b-003 | 505.6b | Already-played-land-this-turn rejection | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-505.6b-004 | 505.6b | Additional land plays (Exploration) | Phase 5-Pre | T22 | |
| ATOM-506.1-001 | 506.1 | Combat phase five steps + skip rule | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-506.1-002 | 506.1 | Two combat damage steps with first/double strike | Phase 4 | ALREADY-IMPLEMENTED | |
| ATOM-506.2-001 | 506.2 | Active player = attacker, nonactive = defender | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-506.3-001 | 506.3 | Only creatures can attack/block | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-506.3a-001 | 506.3a | Noncreature ETB-attacking enters but not attacking | Phase 8 | NEW — noncreature enters-attacking guard | |
| ATOM-506.3b-001 | 506.3b | Non-attacking-player ETB-attacking enters but not attacking | Phase 8 | NEW — ETB-attacking controller validation | |
| ATOM-506.3c-001 | 506.3c | Invalid target ETB-attacking enters but not attacking | Phase 8 | NEW — ETB-attacking target validation | |
| ATOM-506.3d-001 | 506.3d | Late ETB-attacking during declare blockers — unblocked | Phase 8 | NEW — late ETB-attacking unblocked | |
| ATOM-506.3d-002 | 506.3d | Late ETB-attacking during combat damage — unblocked | Phase 8 | Same as 506.3d-001 | |
| ATOM-506.3d-003 | 506.3d | Late ETB-attacking during end of combat — unblocked | Phase 8 | Same as 506.3d-001 | |
| ATOM-506.3e-001 | 506.3e | ETB-blocking controller mismatch — enters but not blocking | Phase 9 | NEW — ETB-blocking controller validation | |
| ATOM-506.4-001 | 506.4 | Removed from combat on leaving battlefield | Phase 5-Pre | T21b | |
| ATOM-506.4-002 | 506.4 | Removed from combat on controller change | Phase 5 | T21b | |
| ATOM-506.4-003 | 506.4 | Removed from combat on stops-being-creature | Phase 5 | T21b | |
| ATOM-506.4-004 | 506.4 | Removed from combat on phasing out | Phase 9 | NEW — phasing removes from combat | |
| ATOM-506.4-005 | 506.4 | Explicit remove-from-combat effect (Maze of Ith) | Phase 8 | T21b | |
| ATOM-506.4a-001 | 506.4a | Post-declaration restriction (Defender) doesn't remove | Phase 4 + Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-506.4b-001 | 506.4b | Tapping attacker doesn't remove from combat | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-506.4c-001 | 506.4c | Orphaned attacker (PW removed) — no damage if unblocked | Phase 8 | NEW — orphaned attacker no damage | |
| ATOM-506.6-001 | 506.6 | "Had to attack" check (goad) | Phase 5-Pre + Phase 8 | T21d | |
| ATOM-508.1-001 | 508.1 | Declare attackers TBA (atomic validation) | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-508.1a-001 | 508.1a | Attackers must be untapped | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-508.1a-002 | 508.1a | Summoning sickness / haste check | Phase 3 + Phase 4 | ALREADY-IMPLEMENTED | |
| ATOM-508.1b-001 | 508.1b | PW/battle attack target selection | Phase 8 | NEW — per-attacker target selection | |
| ATOM-508.1c-001 | 508.1c | Defender restriction prevents attacking | Phase 4 | ALREADY-IMPLEMENTED | |
| ATOM-508.1c-002 | 508.1c | "Can't attack alone" aggregate constraint | Phase 5-Pre | T21d | |
| ATOM-508.1d-001 | 508.1d | Requirement maximization (attacks-if-able) | Phase 5-Pre | T21d | |
| ATOM-508.1d-002 | 508.1d | Cost-gated requirement is optional | Phase 5-Pre | T21d | |
| ATOM-508.1f-001 | 508.1f | Attackers tapped on declaration (not a cost) | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-508.1g-001 | 508.1g | Optional attack cost framework | Phase 8 | NEW — optional attack costs | |
| ATOM-508.1h-001 | 508.1h | Attack cost lock-in mechanism | Phase 8 | NEW — attack cost lock-in | |
| ATOM-508.1k-001 | 508.1k | Creatures become attacking (mid-declaration control change) | Phase 3 + Phase 5 | ALREADY-IMPL (basic) + NEW | |
| ATOM-508.1m-001 | 508.1m | Declare-attacker triggers fire | Phase 7 | NEW — declare-attacker triggers | |
| ATOM-508.2a-001 | 508.2a | Attack trigger characteristic snapshot at declaration time | Phase 7 | NEW — trigger characteristic snapshot | META-trigger-timing |
| ATOM-508.4-001 | 508.4 | ETB-attacking never "attacked" for triggers | Phase 8 | NEW — ETB-attacking trigger suppression | |
| ATOM-508.8-001 | 508.8 | Skip blockers/damage if no attackers | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-509.1-001 | 509.1 | Declare blockers TBA (atomic validation) | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-509.1a-001 | 509.1a | Blockers must be untapped | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-509.1a-002 | 509.1a | Each blocker blocks exactly one attacker | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-509.1a-003 | 509.1a | Multiplayer: blocker only blocks attacker targeting its controller | Phase 9 | NEW — multiplayer blocker validation | |
| ATOM-509.1b-001 | 509.1b | Blocking restrictions (flying evasion) | Phase 4 | ALREADY-IMPLEMENTED | |
| ATOM-509.1b-002 | 509.1b | Cumulative evasion (flying + shadow) | Phase 5-Pre | T21b | |
| ATOM-509.1b-003 | 509.1b | Post-declaration evasion change doesn't invalidate block | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-509.1c-001 | 509.1c | Blocking requirement maximization | Phase 5-Pre | T21d | |
| ATOM-509.1c-002 | 509.1c | Menace + "must block" aggregate interaction | Phase 5-Pre | T21d + T21b | |
| ATOM-509.1d-001 | 509.1d | Blocking cost determination and lock-in | Phase 8 | NEW — blocking cost lock-in | |
| ATOM-509.1g-001 | 509.1g | Creatures become blocking (BlockingInfo set) | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-509.1h-001 | 509.1h | Blocked status persists after blocker removed | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-509.4-001 | 509.4 | ETB-blocking never "blocked" for triggers | Phase 8 | NEW — ETB-blocking trigger suppression | |
| ATOM-509.4a-001 | 509.4a | ETB-blocking invalid target — enters but not blocking | Phase 8 | NEW — ETB-blocking target validation | |
| ATOM-509.4b-001 | 509.4b | ETB-blocking ignores evasion/restrictions | Phase 8 | NEW — ETB-blocking evasion bypass | |
| ATOM-510.1-001 | 510.1 | Damage assignment TBA (attacker then blocker) | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-510.1a-001 | 510.1a | Power = damage; 0 power = no damage | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-510.1b-001 | 510.1b | Unblocked creature assigns to attack target | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-510.1b-002 | 510.1b | Orphaned attacker assigns no damage | Phase 8 | NEW — orphaned attacker no damage | |
| ATOM-510.1c-001 | 510.1c | Blocked, zero remaining blockers = no damage | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-510.1c-002 | 510.1c | Single blocker gets all damage | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-510.1c-003 | 510.1c | Multiple blockers: free damage division (2025 rules) | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-510.1d-001 | 510.1d | Blocker blocking multiple attackers: free damage division | Phase 8 | NEW — multi-block damage division | |
| ATOM-510.1e-001 | 510.1e | Total damage assignment legality check | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-510.2-001 | 510.2 | All combat damage dealt simultaneously | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-510.3-001 | 510.3 | Priority after damage (SBA check first) | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-510.4-001 | 510.4 | First strike two-step combat damage | Phase 4 | ALREADY-IMPLEMENTED | |
| ATOM-510.4-002 | 510.4 | No first/double strike = single damage step | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-510.4-003 | 510.4 | Double strike deals damage in both steps | Phase 4 | ALREADY-IMPLEMENTED | |
| ATOM-511.2-001 | 511.2 | "At end of combat" triggers fire | Phase 7 | NEW — end-of-combat triggers | |
| ATOM-511.2-002 | 511.2 | "Until end of combat" expires at phase end | Phase 5-Pre | T22 | |
| ATOM-511.3-001 | 511.3 | All creatures removed from combat at step end | Phase 3 | ALREADY-IMPLEMENTED | |
| ATOM-513.2-001 | 513.2 | End-step trigger "no back up" rule | Phase 7 | NEW — end-step no-back-up | |
| ATOM-513.2-002 | 513.2 | Delayed trigger "next end step" waits if created during end step | Phase 7 | NEW — delayed trigger timing | |
| ATOM-513.2-003 | 513.2 | "Until end of turn" still expires this turn (not carried over) | Phase 5-Pre | T22 | |
| ATOM-514.1-001 | 514.1 | Cleanup discard to max hand size (TBA) | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-514.2-001 | 514.2 | Damage removed + "until end of turn" effects end simultaneously | Phase 1 + Phase 5-Pre | ALREADY-IMPL (damage) + T22 | |
| ATOM-514.3-001 | 514.3 | No priority during cleanup (default) | Phase 1 | ALREADY-IMPLEMENTED | |
| ATOM-514.3a-001 | 514.3a | Cleanup re-loop: SBAs/triggers → priority → new cleanup | Phase 5-Pre | T16 | |
| ATOM-514.3a-002 | 514.3a | Re-looped cleanup runs full TBAs again | Phase 5-Pre | T16 | |

---

## BOUNDARY-DEF Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-400.2-001 | 400.2 | Zone visibility classification (public vs hidden) | Phase 1 | NEW — zone visibility query | |
| 506.3 | 506.3 | Only creatures attack/block; only players/PW/battles attacked | Phase 3 | ALREADY-IMPLEMENTED | |

---

## COMP Index

| ID | Rule | Summary | Phase | Ticket | Composes |
|----|------|---------|-------|--------|----------|
| COMP-ZONE-TRANSITION-001 | 400.3 + 400.7 | Owner routing + new object identity on destroy | Phase 1 + Phase 5 | NEW | ATOM-400.3-001, ATOM-400.7-001 |
| COMP-COMBAT-FULL-001 | 508.1 + 509.1 + 510.1 + 510.2 + 511.3 | Full combat sequence: declare → block → damage → cleanup | Phase 3 | ALREADY-IMPLEMENTED | ATOM-508.1-001, ATOM-509.1-001, ATOM-510.1-001, ATOM-510.2-001, ATOM-511.3-001 |
| COMP-FIRST-STRIKE-KILL-001 | 510.4 + 510.1a + 510.1c | First-strike kills blocker before normal damage | Phase 4 | ALREADY-IMPLEMENTED | ATOM-510.4-001, ATOM-510.1a-001, ATOM-510.1c-001 |
| COMP-CLEANUP-RELOOP-001 | 514.1 + 514.2 + 514.3a | Cleanup step triggers re-loop | Phase 5-Pre + Phase 7 | T16 | ATOM-514.1-001, ATOM-514.2-001, ATOM-514.3a-001, ATOM-514.3a-002 |
| COMP-UNTAP-TRIGGER-UPKEEP-001 | 502.3 + 502.4 + 503.1a | Untap trigger held until upkeep | Phase 7 | NEW | ATOM-502.3-001, ATOM-502.4-001, ATOM-503.1a-001 |
| COMP-508.8-SKIP-001 | 508.8 + 506.1 | No attackers → skip blockers/damage → end of combat | Phase 3 | ALREADY-IMPLEMENTED | ATOM-508.8-001, ATOM-506.1-001, ATOM-511.3-001 |

---

## META Entries

**META-EPOCH-STAMP** (Rule 400.7)
- Engine persists `ObjectId` across zone changes; uses epoch-stamp for "new object" semantics.
- `GameState.zone_change_epoch: u64` — global monotonic counter incremented on every `move_object`.
- `GameObject.last_zone_change_epoch: u64` — per-object timestamp.
- `ObjectRef { id: ObjectId, zone_epoch: u64 }` — stale-reference checking by effects/targeting.
- u64 overflow is impossible under normal play or loop shortcutting (see session-4.md for full analysis).

**META-TRIGGER-TIMING** (Rule 508.2a)
- Attack/block triggers snapshot characteristics at the moment of declaration. Later characteristic changes don't retroactively fire triggers.

**META-COMBAT-REQUIREMENTS-SOLVER** (Rules 508.1d, 509.1c)
- Attack and block requirements are a constraint satisfaction problem (maximum cardinality subset of requirements satisfiable without violating any restriction). Potentially NP-hard in general, but tractable for realistic game states (<20 creatures). Recommended: backtracking with pruning or ILP over boolean variables. Attack and blocking solvers should share infrastructure.

**META-MANA-ABILITY-WINDOWS** (Rule 508.1i)
- Wherever the engine has a "pay costs" step, it must open a mana ability window first. Cross-cutting concern across casting (601.2g), attack costs (508.1i), block costs (509.1e), and mana ability resolution (605.3a).

**META-DP-ORDERING-CONSOLIDATION** (Rules 401.4, 404.3)
- Multiple rules require "choose ordering" for simultaneous events (library placement, graveyard ordering, trigger ordering). Should consolidate into a single `choose_ordering(&[ObjectId], context: OrderingContext) -> Vec<ObjectId>` method on DecisionProvider rather than separate methods per context.

---

## Classification Summary Table

### Chapter 4: Zones (400–408)

| Rule | Classification | Notes |
|------|---------------|-------|
| 400.1 | PURE-DEF | Names the seven zones |
| 400.2 | BOUNDARY-DEF | Public vs hidden zones |
| 400.3 | TESTABLE | Owner routing for library/graveyard/hand |
| 400.4 | PURE-DEF | Parent rule |
| 400.4a | TESTABLE | Instant/sorcery can't enter battlefield |
| 400.4b | OUT-OF-SCOPE | Conspiracy/phenomenon/plane/scheme/vanguard |
| 400.5 | TESTABLE | Zone ordering preserved |
| 400.6 | TESTABLE | Zone-change replacement effects |
| 400.7 | TESTABLE | New object identity on zone change |
| 400.7a | TESTABLE | Spell-to-permanent effect continuity |
| 400.7b | TESTABLE | Static ability grants continue |
| 400.7c | TESTABLE | Prevention effects continue |
| 400.7d | TESTABLE | CastInfo on permanent |
| 400.7e | TESTABLE | Zone-change triggers find new object |
| 400.7f | TESTABLE | Aura LTB trigger finds aura in graveyard |
| 400.7g | DEFERRED | Cast-permission continuity (Phase 8+) |
| 400.7h | DEFERRED | Effect-grants-cast finding new object (Phase 8+) |
| 400.7i | DEFERRED | Effect-grants-land-play finding new object (Phase 8+) |
| 400.7j | TESTABLE | Effect moves object to public zone, finds it |
| 400.7k | DEFERRED | Madness (Phase 8+) |
| 400.7m | OUT-OF-SCOPE | Stickers (not in scope) |
| 400.8 | TESTABLE | Re-exile creates new object identity |
| 400.9 | DEFERRED | Face-down command zone (Phase 9) |
| 400.10 | DEFERRED | Command zone re-entry (Phase 9) |
| 400.11 | PURE-DEF | "Outside the game" not a zone |
| 400.11a | PURE-DEF | Sideboard cards |
| 400.11b | DEFERRED | Bring from outside (Phase 8+ Wish) |
| 400.11c | DEFERRED | Affecting outside cards (Phase 8+) |
| 400.12 | TESTABLE | Batch zone-to-zone move |
| 401.1 | PURE-DEF | Library at game start |
| 401.2 | TESTABLE | Library face-down |
| 401.3 | TESTABLE | Count any library |
| 401.4 | TESTABLE | Multi-card library placement ordering |
| 401.5 | TESTABLE | Top-of-library reveal during casting |
| 401.6 | DEFERRED | Revealed top card identity (Phase 8+) |
| 401.7 | TESTABLE | "Nth from top" fallback |
| 402.1 | PURE-DEF | Hand definition |
| 402.2 | TESTABLE | Max hand size + cleanup discard |
| 402.3 | TESTABLE | Hand visibility (reveal vs look-at) |
| 403.1 | PURE-DEF | Battlefield starts empty |
| 403.2 | TESTABLE | Default battlefield scope |
| 403.3 | PURE-DEF | Permanents on battlefield |
| 403.4 | TESTABLE | New object on ETB |
| 403.5 | PURE-DEF | Historical note |
| 404.1 | TESTABLE | Graveyard placement |
| 404.2 | TESTABLE | Graveyard face-up, public |
| 404.3 | TESTABLE | Simultaneous graveyard ordering |
| 405.1 | PURE-DEF | Stack intro |
| 405.2 | TESTABLE | Stack LIFO ordering |
| 405.3 | TESTABLE | APNAP simultaneous stack placement |
| 405.4 | TESTABLE | Spell/ability characteristics on stack |
| 405.5 | TESTABLE | Resolution rule (all pass → resolve) |
| 405.6 | PURE-DEF | Parent: things that don't use stack |
| 405.6a | PURE-DEF | Effects don't go on stack |
| 405.6b | PURE-DEF | Static abilities don't use stack |
| 405.6c | TESTABLE | Mana abilities resolve immediately |
| 405.6d | PURE-DEF | Special actions don't use stack |
| 405.6e | TESTABLE | TBAs don't use stack |
| 405.6f | TESTABLE | SBAs don't use stack |
| 405.6g | PURE-DEF | Concession is immediate |
| 405.6h | DEFERRED | Multiplayer leaves-game (Phase 9) |
| 406.1 | PURE-DEF | Exile definition |
| 406.2 | PURE-DEF | "To exile" definition |
| 406.3 | TESTABLE | Exile face-up default + face-down |
| 406.3a | DEFERRED | Face-down exile characteristics (Phase 8+) |
| 406.3b | DEFERRED | Casting from face-down exile (Phase 8+) |
| 406.4 | DEFERRED | Face-down exile piles (Phase 8+) |
| 406.5 | PURE-DEF | Exile pile organization |
| 406.6 | PURE-DEF | Linked abilities reference |
| 406.7 | TESTABLE | Re-exile identity (duplicate of 400.8) |
| 406.8 | PURE-DEF | Historical note |
| 407.1–407.4 | OUT-OF-SCOPE | Ante |
| 408.1 | PURE-DEF | Command zone definition |
| 408.2 | TESTABLE | Emblems in command zone |
| 408.3 | DEFERRED | Format-specific command zone (Phase 9) |

### Chapter 5: Turn Structure (500–514)

| Rule | Classification | Notes |
|------|---------------|-------|
| 500.1 | TESTABLE | Five phases in order |
| 500.2 | TESTABLE | Phase/step end condition |
| 500.3 | TESTABLE | No-priority steps end after TBAs |
| 500.4 | TESTABLE | "Until" duration expiry at step/phase begin |
| 500.5 | TESTABLE | End-of-step effect expiry + mana emptying |
| 500.5a | TESTABLE | "Until end of combat" timing |
| 500.5b | PURE-DEF | "Until end of turn" reference |
| 500.6 | TESTABLE | "At the beginning of" triggers |
| 500.7 | DEFERRED | Extra turns (Phase 9) |
| 500.8 | DEFERRED | Extra phases (Phase 9) |
| 500.9 | DEFERRED | Extra steps (Phase 9) |
| 500.10 | DEFERRED | Adding step after phase (Phase 9) |
| 500.10a | DEFERRED | "You get" step (Phase 9) |
| 500.11 | DEFERRED | Skip step/phase/turn (Phase 9) |
| 500.12 | PURE-DEF | No events between steps |
| 501.1 | PURE-DEF | Beginning phase structure |
| 502.1 | DEFERRED | Phasing (Phase 9) |
| 502.2 | DEFERRED | Day/Night (Phase 9) |
| 502.2a | DEFERRED | Multiplayer day/night (Phase 9) |
| 502.3 | TESTABLE | Untap all permanents |
| 502.4 | TESTABLE | No priority during untap |
| 503.1 | TESTABLE | Upkeep: no TBA, priority |
| 503.1a | TESTABLE | Untap + upkeep triggers on stack |
| 503.2 | DEFERRED | Multiple upkeep steps (Phase 9) |
| 504.1 | TESTABLE | Draw step TBA |
| 504.2 | PURE-DEF | Priority after draw TBA |
| 505.1 | TESTABLE | Two main phases |
| 505.1a | TESTABLE | Precombat vs postcombat identity |
| 505.1b | PURE-DEF | "First/second main phase" text |
| 505.2 | PURE-DEF | Main phase end condition |
| 505.3 | OUT-OF-SCOPE | Archenemy scheme (not in scope) |
| 505.4 | DEFERRED | Saga lore counter (Phase 8) |
| 505.5 | OUT-OF-SCOPE | Attractions (not in scope) |
| 505.6 | PURE-DEF | Priority during main phase |
| 505.6a | TESTABLE | Sorcery-speed timing |
| 505.6b | TESTABLE | Land play timing |
| 506.1 | TESTABLE | Combat phase five steps + skip rule |
| 506.2 | TESTABLE | Attacking/defending player roles |
| 506.2a | DEFERRED | Multiplayer defending (Phase 9) |
| 506.2b | OUT-OF-SCOPE | Shared team turns (not in scope) |
| 506.3 | BOUNDARY-DEF | Only creatures attack/block |
| 506.3a | TESTABLE | Noncreature ETB-attacking |
| 506.3b | TESTABLE | Non-attacking-player ETB-attacking |
| 506.3c | TESTABLE | Invalid target ETB-attacking |
| 506.3d | TESTABLE | Late ETB-attacking enters unblocked |
| 506.3e | TESTABLE | ETB-blocking controller mismatch |
| 506.3f | DEFERRED | Battle + creature (Phase 8+) |
| 506.3g | DEFERRED | Battle becomes creature (Phase 8+) |
| 506.4 | TESTABLE | Removal from combat conditions |
| 506.4a | TESTABLE | Post-declaration restrictions don't remove |
| 506.4b | TESTABLE | Tap/untap doesn't remove from combat |
| 506.4c | TESTABLE | Orphaned attacker (PW removed) |
| 506.4d | DEFERRED | Blocking creature + attacked PW (Phase 8+) |
| 506.4e | DEFERRED | Attacked PW + battle (Phase 8+) |
| 506.5 | PURE-DEF | "Attacks alone" / "blocking alone" |
| 506.6 | TESTABLE | "Had to attack" check |
| 506.7 | PURE-DEF | Combat timing restrictions parent |
| 506.7a–g | TESTABLE | Deferred to Session 5 (casting pipeline) |
| 507.1 | DEFERRED | Multiplayer defending player TBA (Phase 9) |
| 507.2 | PURE-DEF | Priority after beginning of combat |
| 508.1 | TESTABLE | Declare attackers TBA |
| 508.1a | TESTABLE | Untapped + summoning sickness checks |
| 508.1b | TESTABLE | PW/battle attack target selection |
| 508.1c | TESTABLE | Attack restrictions (Defender, etc.) |
| 508.1d | TESTABLE | Requirement maximization |
| 508.1e | OUT-OF-SCOPE | Banding (not in scope) |
| 508.1f | TESTABLE | Attackers tapped (not a cost) |
| 508.1g | TESTABLE | Optional attack costs |
| 508.1h | TESTABLE | Attack cost lock-in |
| 508.1i | TESTABLE | Mana ability window (cross-cutting) |
| 508.1j | PURE-DEF | Pay all costs |
| 508.1k | TESTABLE | Creatures become attacking |
| 508.1m | TESTABLE | Declare-attacker triggers |
| 508.2 | PURE-DEF | Priority after declaration |
| 508.2a | TESTABLE | Attack trigger characteristic snapshot |
| 508.2b | PURE-DEF | Trigger APNAP placement |
| 508.3 | PURE-DEF | Trigger conditions parent |
| 508.3a–f | TESTABLE | Deferred to Phase 7 (trigger conditions) |
| 508.4 | TESTABLE | ETB-attacking never "attacked" |
| 508.4a–d | TESTABLE | Covered by 506.3a–d tests |
| 508.5 | PURE-DEF | "Defending player" definition |
| 508.5a | DEFERRED | Multiplayer disambiguation (Phase 9) |
| 508.6 | PURE-DEF | "[Player] is attacking [player]" |
| 508.7 | DEFERRED | Attack target reselection (Phase 8+) |
| 508.7a–e | DEFERRED | Reselection details (Phase 8+) |
| 508.8 | TESTABLE | Skip blockers/damage if no attackers |
| 509.1 | TESTABLE | Declare blockers TBA |
| 509.1a | TESTABLE | Blocker untapped + assignment |
| 509.1b | TESTABLE | Blocking restrictions (evasion) |
| 509.1c | TESTABLE | Blocking requirement maximization |
| 509.1d | TESTABLE | Blocking cost lock-in |
| 509.1e | PURE-DEF | Mana ability window |
| 509.1f | PURE-DEF | Pay all costs |
| 509.1g | TESTABLE | Creatures become blocking |
| 509.1h | TESTABLE | Blocked/unblocked status |
| 509.1i | PURE-DEF | Block triggers fire point |
| 509.2 | PURE-DEF | Priority after block declaration |
| 509.2a | PURE-DEF | Block triggers APNAP |
| 509.3 | PURE-DEF | Block trigger conditions parent |
| 509.3a–g | TESTABLE | Deferred to Phase 7 (trigger conditions) |
| 509.4 | TESTABLE | ETB-blocking never "blocked" |
| 509.4a | TESTABLE | ETB-blocking invalid target |
| 509.4b | TESTABLE | ETB-blocking ignores evasion/restrictions |
| 510.1 | TESTABLE | Damage assignment TBA |
| 510.1a | TESTABLE | Power = damage; 0 power = no damage |
| 510.1b | TESTABLE | Unblocked → target; orphaned → nothing |
| 510.1c | TESTABLE | Blocked creature damage division (2025) |
| 510.1d | TESTABLE | Blocking creature damage division |
| 510.1e | TESTABLE | Total assignment legality check |
| 510.2 | TESTABLE | Simultaneous damage dealing |
| 510.3 | TESTABLE | Priority after damage (SBA check first) |
| 510.3a | TESTABLE | Damage triggers (Phase 7) |
| 510.4 | TESTABLE | First strike / double strike two steps |
| 511.1 | PURE-DEF | End of combat: no TBA, priority |
| 511.2 | TESTABLE | End-of-combat triggers + duration expiry |
| 511.3 | TESTABLE | Remove all from combat |
| 512.1 | PURE-DEF | Ending phase structure |
| 513.1 | PURE-DEF | End step: no TBA, priority |
| 513.1a | PURE-DEF | Historical errata note |
| 513.2 | TESTABLE | End-step trigger "no back up" |
| 514.1 | TESTABLE | Cleanup discard to hand size |
| 514.2 | TESTABLE | Damage removal + effect expiry |
| 514.3 | TESTABLE | No priority during cleanup (default) |
| 514.3a | TESTABLE | Cleanup re-loop |

---

## NEW Tickets List

| # | Ticket | Mechanism | Source Rules |
|---|--------|-----------|-------------|
| 1 | Zone visibility classification query | Public/hidden zone predicate | 400.2 |
| 2 | Replacement effects on zone transitions | Zone-change replacement effect framework | 400.6 |
| 3 | Replacement effect conflict resolution | Controller chooses among contradictory replacement effects | 400.6 (rule 616) |
| 4 | zone_change_epoch infrastructure | ObjectRef stale-reference checking via epoch stamps | 400.7 |
| 5 | Blink identity reset | Exile-and-return creates new object with no memory | 400.7 |
| 6 | Stack-to-permanent effect continuity | Color/text-changing effects persist across stack→battlefield | 400.7a |
| 7 | Prevention effect continuity | Prevention on spell persists to permanent | 400.7c |
| 8 | Prevention effect source identity | Tokens ≠ source permanent for prevention | 400.7c |
| 9 | Zone-change trigger object tracking | LTB triggers find new object in public zone | 400.7e, 400.7f |
| 10 | Batch zone-to-zone move | "Shuffle graveyard into library" pattern | 400.12 |
| 11 | Multi-card library placement ordering | Scry/put-on-top via DecisionProvider | 401.4 |
| 12 | "Play with top card revealed" effects | Continuous + casting-freeze variants | 401.5 |
| 13 | Library positional insertion with fallback | "Nth from top" with fewer than N cards | 401.7 |
| 14 | Hand visibility enforcement | Per-player visibility filter in oracle layer | 402.3 |
| 15 | Reveal/look-at keyword implementations | "Reveal" = all players; "look at" = controller only | 402.3 |
| 16 | Simultaneous graveyard ordering | Batch zone-move with auto-order fallback | 404.3 |
| 17 | APNAP trigger ordering | Simultaneous triggered abilities stack placement | 405.3 |
| 18 | "Any player may activate" controller | Non-controller activation legality | 405.4 |
| 19 | Exile zone visibility | Face-up default + face-down tracking | 406.3 |
| 20 | Emblem creation in command zone | Planeswalker ultimate emblems | 408.2 |

---

## Gap Report

### Mechanisms in roadmap/implementation-plan with tests in this session

| Ticket | Mechanism | Session 4 Tests |
|--------|-----------|-----------------|
| T16 | Cleanup re-loop (514.3a) | ATOM-514.3a-001, ATOM-514.3a-002 |
| T21a | Zone guards + CastInfo | ATOM-400.4a-001, ATOM-400.7d-001 |
| T21b | Combat removal on control/type change | ATOM-506.4-001/002/003, ATOM-509.1b-002 |
| T21d | Combat requirements solver | ATOM-508.1c-002, ATOM-508.1d-001/002, ATOM-509.1c-001/002, ATOM-506.6-001 |
| T22 | Duration + turn structure fixes | ATOM-500.4-001, ATOM-500.5-001, ATOM-500.5a-001, ATOM-505.6b-004, ATOM-511.2-002, ATOM-513.2-003, ATOM-514.2-001 |

### CR rules in Chapters 4–5 with no matching implementation ticket

1. **Zone visibility query (400.2)** — No ticket. Needs a public/hidden zone predicate.
2. **Replacement effect on zone changes (400.6)** — Phase 6 scope, no specific ticket yet.
3. **Object identity break verification (400.7)** — Implicit in `move_object` but no explicit test that effect references break.
4. **Stack-to-permanent effect continuity (400.7a)** — No ticket. Phase 5 scope.
5. **Prevention effect continuity (400.7c)** — No ticket. Phase 6 scope.
6. **Zone-change trigger object tracking (400.7e, 400.7f)** — No ticket. Phase 7 scope.
7. **Simultaneous graveyard ordering (404.3)** — No ticket. Needs batch zone-move.
8. **APNAP trigger ordering (405.3)** — No ticket. Phase 7 scope.
9. **"Doesn't untap" continuous effect (502.3)** — No ticket. Phase 5 scope.
10. **Trigger hold-and-release from no-priority steps (502.4)** — No ticket. Phase 7 scope.
11. **Step/phase beginning trigger checking (500.6)** — No ticket. Phase 7 infrastructure.
12. **End-step "no back up" rule (513.2)** — No ticket. Phase 7 scope.
13. **ETB-attacking/blocking effects (506.3a–d, 508.4, 509.4)** — No tickets. Phase 8 scope.
14. **Attack/blocking cost framework (508.1g–h, 509.1d)** — No tickets. Phase 8 scope.
15. **Multi-block damage division (510.1d)** — No ticket. Phase 8 scope.
16. **Orphaned attacker (506.4c, 510.1b)** — No ticket. Phase 8 (PW combat).
17. **Constrained untap choice (502.3)** — No ticket. Winter Orb / Stasis pattern. Phase 5 + Phase 8.
18. **Multiplayer blocker controller validation (509.1a)** — No ticket. Phase 9 (Commander combat).
19. **Phasing removes from combat (506.4)** — No ticket. Phase 9 (phasing, D1).
20. **ETB-blocking evasion bypass (509.4b)** — No ticket. Phase 8. Low priority (no current card).

---

## ALREADY-IMPLEMENTED List

400.3, 400.5, 401.2, 401.3, 402.2, 403.2, 403.4, 404.1, 404.2, 405.2, 405.4, 405.5, 405.6c, 405.6e, 405.6f, 500.1, 500.2, 500.3, 502.3, 503.1, 504.1, 505.1, 505.1a, 505.6a, 505.6b, 506.1, 506.2, 506.3, 506.4a, 506.4b, 508.1, 508.1a, 508.1c, 508.1f, 508.1k, 508.8, 509.1, 509.1a, 509.1b, 509.1g, 509.1h, 510.1, 510.1a, 510.1b, 510.1c, 510.1e, 510.2, 510.3, 510.4, 511.3, 514.1, 514.3

---

## OUT-OF-SCOPE List

| Rule(s) | Reason |
|---------|--------|
| 400.4b | Conspiracy/phenomenon/plane/scheme/vanguard supplemental types |
| 400.7m | Stickers (Un-set, permanently out of scope) |
| 407.1–407.4 | Ante (permanently out of scope) |
| 505.3 | Archenemy scheme TBA (team format, out of scope) |
| 505.5 | Attractions roll TBA (Un-set mechanic, out of scope) |
| 506.2b | Shared team turns (team format, out of scope) |
| 508.1e | Banding (permanently out of scope) |

---

## DEFERRED List

| Rule(s) | Target Phase | Reason |
|---------|-------------|--------|
| 400.7g | Phase 8+ | Cast-permission continuity |
| 400.7h | Phase 8+ | Effect-grants-cast finding new object |
| 400.7i | Phase 8+ | Effect-grants-land-play finding new object |
| 400.7k | Phase 8+ | Madness |
| 400.9 | Phase 9 | Face-down command zone |
| 400.10 | Phase 9 | Command zone re-entry identity break |
| 400.11b | Phase 8+ | Wish effects (bring from outside the game) |
| 400.11c | Phase 8+ | Cards outside the game can't be affected |
| 401.6 | Phase 8+ | Revealed top card identity tracking |
| 405.6h | Phase 9 | Multiplayer player-leaves-game cleanup |
| 406.3a | Phase 8+ | Face-down exile characteristics |
| 406.3b | Phase 8+ | Casting from face-down exile |
| 406.4 | Phase 8+ | Face-down exile piles management |
| 408.3 | Phase 9 | Format-specific command zone (Commander) |
| 500.7 | Phase 9 | Extra turns (mutable TurnPlan) |
| 500.8 | Phase 9 | Extra phases |
| 500.9 | Phase 9 | Extra steps |
| 500.10 | Phase 9 | Adding step after phase |
| 500.10a | Phase 9 | "You get" additional step |
| 500.11 | Phase 9 + Phase 6 | Skip step/phase/turn |
| 502.1 | Phase 9 | Phasing during untap step |
| 502.2 | Phase 9 | Day/Night transition during untap |
| 502.2a | Phase 9 | Multiplayer shared team turns day/night |
| 503.2 | Phase 9 | Multiple upkeep steps |
| 505.4 | Phase 8 | Saga lore counter TBA |
| 506.2a | Phase 9 | Multiplayer defending player choice |
| 506.3f | Phase 8+ | Battle + creature entering attacking/blocking |
| 506.3g | Phase 8+ | Battle becomes attacking/blocking creature |
| 506.4d | Phase 8+ | Blocking creature + attacked planeswalker |
| 506.4e | Phase 8+ | Attacked PW + battle |
| 507.1 | Phase 9 | Multiplayer defending player TBA |
| 508.5a | Phase 9 | Multiplayer "defending player" disambiguation |
| 508.7 | Phase 8+ | Attack target reselection |
| 508.7a–e | Phase 8+/Phase 9 | Reselection details |

---

## Session 4 Statistics

- **Total sub-rules analyzed:** ~190
- **ATOM tests generated:** 159
- **COMP tests generated:** 6
- **Total tests:** 165
- **Classifications:**
  - TESTABLE: 100
  - BOUNDARY-DEF: 2
  - PURE-DEF: 46
  - DEFERRED: 37 (Phase 8+ and Phase 9 mechanics)
  - OUT-OF-SCOPE: 7 (400.4b supplemental types, 400.7m stickers, 407.x ante, 505.3 Archenemy, 505.5 Attractions, 506.2b shared team turns, 508.1e banding)
  - Deferred to other sessions: ~20 (506.7a–g, 508.3a–f, 509.3a–g)
- **ALREADY-IMPLEMENTED:** 42 rules (covering ~48 ATOM tests)
- **NEW tickets identified:** 20

---

*End of Session 4 Summary — Chapters 4 & 5: Zones (400–408) & Turn Structure (500–514)*
