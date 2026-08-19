# Phase 5-Pre Audit

Reconciliation of shipped tickets (T01–T17 window) against the atomic-tests spec in `phase-index-phase-5-pre.md`.

**Scope:** rows in the Phase 5-Pre index tagged with shipped tickets (plus co-ticketed rows where at least one ticket is shipped). Pending tickets (T18, T19, T20, T21a–d, T22) are out of scope — those get audited when their tickets are re-scoped. Also out of scope: ~45 `NEW-*` rows that represent plan drift (separate reconciliation).

**Verification protocol:**

- For clean-A / clean-C rows: index summary + code grep is enough.
- For ambiguous rows: consult `sessions/session-N.md` for full enriched spec AND `MTG-Rules/Chapter *.txt` for CR wording.
- Code citations are `file:line-range` from the `mtgsim/src/` tree.

**Status legend:**

- **A** — Plausibly aligned. Code covers the ATOM; cite provided.
- **B-critical** — Drift that actively breaks something. Needs a single-ATOM fix ticket.
- **B-cosmetic** — Technically wrong but no behavioral impact (stale label, wrong rule number in comment). Logged; fix when area is touched.
- **C** — Not shipped; covered by a pending ticket, explicitly deferred, or requires infrastructure not yet in place (e.g. L4 type-changing for some scenarios).
- **D** — Unclear from triage; needs deeper code dive.

---

## T15 — Aura/Equipment SBAs (17 rows)

| ATOM ID | Rule | Summary | Status | Evidence / Note |
|---|---|---|---|---|
| ATOM-301.5-002 | 301.5 | Equipment can't attach to non-creature (negative) | **C** | Depends on Equip activation (702.6a). Equip ability not shipped — `grep` finds no 702.6a handler in engine/. Session marks this Phase 5-Pre but the code path that would reject the attachment doesn't exist yet. |
| ATOM-301.5c-002 | 301.5c | Equipment loses subtype → SBA unattaches | **C** | **Index misclassifies as 5-Pre/T15.** `sessions/session-3.md:207-210` says **Phase 8, NEW ticket** — requires L4 type-changing effects to remove Equipment subtype. |
| ATOM-303.4c-001 | 303.4c | Aura on illegal object (type removed) → graveyard SBA | **A** | `engine/sba.rs:220-225` — `enchant_filter` check in the 704.5m block (CR cross-ref: 303.4c is the rule, 704.5n the SBA). Test: `test_enchant_filter_creature_only` at `sba.rs:1136+`. |
| ATOM-303.4d-001 | 303.4d | Self-enchanting Aura → graveyard SBA | **C** | **Index misclassifies.** `sessions/session-3.md:637-640` says **Phase 8, NEW ticket** — only constructible via type/text manipulation. |
| ATOM-701.3a-001 | 701.3a | Attach Equipment to creature — basic | **C** | No general `attach()` primitive exists. `resolve.rs:325` only has `attach_aura_on_etb`. Equip activation (the Equipment path) not shipped. |
| ATOM-701.3a-002 | 701.3a | Attach to invalid target — rejected | **C** | Same as above — path doesn't exist to exercise. |
| ATOM-701.3b-001 | 701.3b | Failed attach — no movement | **C** | Same. |
| ATOM-701.3b-002 | 701.3b | Reattach to same target — no-op | **C** | Same. |
| ATOM-701.3b-003 | 701.3b | Non-Aura/Equipment attach — does nothing | **C** | Same. |
| ATOM-701.3c-001 | 701.3c | Reattach to different target — new timestamp | **C** | Same + requires timestamp refresh logic (tagged `dependency, layers`). |
| ATOM-701.21a-001 | 701.21a | Sacrifice bypasses destroy/indestructible | **A** | `Cost::SacrificeSelf` at `engine/costs.rs:91-96` (legality) + `186-188` (execution moves directly to graveyard, not through Destroy). Architecturally bypasses indestructible. |
| ATOM-701.21a-002 | 701.21a | Can't sacrifice what you don't control | **C** | `Cost::Sacrifice(_, _)` at `engine/costs.rs:97` returns NotImplemented. Deferred to T18c per memory. |
| ATOM-701.21a-003 | 701.21a | Can't sacrifice non-permanents | **C** | Same as above — primitive not implemented. |
| ATOM-704.5m-001 | 704.5m | Unattached Aura → graveyard | **A** | `engine/sba.rs:201-214`. Test `test_sba_unattached_aura_dies` at `sba.rs:804-830`. |
| ATOM-704.5m-002 | 704.5m | Aura host left → graveyard | **A** | `engine/sba.rs:215-219` (`!battlefield.contains_key(&host_id)` branch). `cleanup_zone_state` at `engine/zones.rs:311-314` also clears `attached_to` when host leaves, belt-and-suspenders. Test `test_sba_aura_host_left_battlefield`. |
| ATOM-704.5n-001 | 704.5n | Equipment on non-creature → unattach, stays on BF | **B-cosmetic** | Behavior covered at `engine/sba.rs:241-268`. Test `test_sba_equipment_on_noncreature_unattaches`. **However**, code comment labels block `704.5p` and the catch-all block `704.5q`. Per CR (`MTG-Rules/Chapter 7.txt:2285-2289`), current numbering is: 704.5n = Equipment/Fortification, 704.5p = attachment catch-all, 704.5q = counter annihilation. Code reflects an older CR. See **Labeling discrepancies** section. |
| ATOM-704.5p-001 | 704.5p | Creature illegally attached → unattach | **C** | Scenario (creature with `attached_to` set) can only arise via L4 type-changing. `engine/sba.rs:300-305` explicitly TODOs this case for when L4 lands. Test at `sba.rs:960+` constructs the scenario artificially to exercise the catch-all code path; real-game trigger is Phase 5 Layers-gated. |

### T15 co-tickets (T04, T15) and (T15, T04)

| ATOM ID | Rule | Summary | Status | Evidence / Note |
|---|---|---|---|---|
| ATOM-301.5-001 | 301.5 | Equipment attaches to creature (legal) | **C** | Positive case of 301.5-002. Same blocker — no Equip activation path. |
| ATOM-301.5-001/002 | 301.5 | Composite legal+illegal | **C** | Same. |
| ATOM-301.5c-004 | 301.5c | Equipment on destroyed creature → unattached, stays on BF | **A** | `engine/zones.rs:cleanup_zone_state` clears `attached_to` on zone exit (lines 311-314). Equipment stays on BF with `attached_to=None` after host dies. Test at `engine/zones.rs:490-494` asserts exactly this. |
| COMP-301.5c+303.4c-001 | 301.5c + 303.4c | Creature destroyed: Equipment stays BF unattached, Aura to GY | **A** | Composite of ATOM-301.5c-004 (A) and ATOM-303.4c-002 (A below). Both sides covered. |

---

## T15b — Aura ETB + Enchant filter (13 rows)

| ATOM ID | Rule | Summary | Status | Evidence / Note |
|---|---|---|---|---|
| ATOM-303.4-001 | 303.4 | Aura ETB attached to target creature | **A** | `engine/resolve.rs:325 attach_aura_on_etb`. Spell-cast path attaches the Aura when resolving on the stack. |
| ATOM-303.4a-001 | 303.4a | Aura spell requires target; no legal target → can't cast | **A** | `engine/targeting.rs validate_selection` + `has_any_legal_choice` (memory `144463b3`). Cast legality pre-check rejects casts with no legal host. |
| ATOM-303.4c-002 | 303.4c | Aura host destroyed → graveyard SBA | **A** | Same mechanism as ATOM-704.5m-002. `cleanup_zone_state` clears attached_to; 704.5m SBA catches unattached Aura. |
| ATOM-303.4e-003 | 303.4e | Pacifism cast on opponent's creature: caster = Aura controller | **A** | Aura controller = caster per standard spell-resolution (rule 110.2). Covered by `engine/stack.rs` using `entry.controller` for BattlefieldEntity when permanent ETBs. |
| ATOM-303.4f-001 | 303.4f | Non-stack Aura ETB: controller chooses (hexproof OK) | **A** | `attach_aura_on_etb` takes a controller parameter; `has_any_legal_choice` used to pre-check. Non-stack ETB bypasses targeting so hexproof does not restrict. |
| ATOM-608.3b-001 | 608.3b | Targeted permanent fizzle or bestow fallback | **D** | Plain-fizzle half is covered by `any_targets_still_legal` in `engine/stack.rs`. Bestow fallback requires Bestow (rule 702.103), which is a Phase 8 `NEW — Bestow` ticket. Needs a closer look to confirm the fizzle path correctly short-circuits without bestow. |
| ATOM-608.3c-001 | 608.3c | Aura ETB attachment | **A** | Duplicate of ATOM-303.4-001 coverage. |
| ATOM-701.3d-001 | 701.3d | Unattach Equipment — stays on battlefield | **A** | Covered by SBA at `engine/sba.rs:241-268` + `cleanup_zone_state` clearing attached_to. |
| ATOM-701.3d-002 | 701.3d | Creature leaves → Equipment becomes unattached | **A** | Same mechanism; test at `engine/zones.rs:490`. |
| ATOM-702.5a-001 | 702.5a | Aura targeting restricted by enchant ability | **A** | `CardData.enchant_filter` + `validate_selection`. Memory `144463b3`. |
| ATOM-702.5d-001 | 702.5d | Enchant player Aura can't target permanents | **A** | `SelectionFilter::Player` via `enchant_filter`. |
| ATOM-702.6a-001 | 702.6a | Equip activation, attachment, sorcery-speed | **C** | **Ticket marked done but atomic-test claim not satisfied.** Equip activated ability is not implemented — grep for `702.6a`, `equip_activation`, or Equip keyword handler in `engine/` returns nothing. T15b memory confirms scope was "enchant_filter / Aura ETB attach" only; Equip was outside scope despite the index tagging these 702.6a rows as T15b. |
| ATOM-702.6a-002 | 702.6a | Equip targets only own creatures | **C** | Same blocker. |
| ATOM-702.6a-003 | 702.6a | Equip sorcery-speed enforcement (negative) | **C** | Same blocker. |

---

## T14 — Legendary, Planeswalker loyalty, counters (14 rows)

| ATOM ID | Rule | Summary | Status | Evidence / Note |
|---|---|---|---|---|
| ATOM-104.3d-001 | 104.3d | 10+ poison counters → lose (SBA) | **A** | T14+T16 co-coverage: `engine/sba.rs:56-82` (704.5c). Tests `test_sba_poison_10_loses` / `test_sba_poison_9_survives` at `sba.rs:1062+`. |
| ATOM-122.1-001 | 122.1 | Counters are markers, not objects | **A** | `counters: HashMap<CounterType, u32>` on `BattlefieldEntity` (`state/battlefield.rs:97`). Not ObjectIds. Boundary/type test. |
| ATOM-122.1e-001 | 122.1e | PW with 0 loyalty → SBA graveyard | **A** | Co-ticket T14+T16. `engine/sba.rs:135-155`. |
| ATOM-122.1f-001 | 122.1f | 10+ poison → SBA loss | **A** | Duplicate of 104.3d-001 coverage. |
| ATOM-122.2-001 | 122.2 | Counters removed on zone change | **A** | `counters` HashMap is on `BattlefieldEntity`; leaving battlefield drops the entity, counters gone by construction. |
| ATOM-122.3-001 | 122.3 | +1/+1 and -1/-1 annihilate (SBA) | **A** | `engine/sba.rs:307-329` (labeled 704.5q in code, which matches current CR). Tests in sba.rs. |
| ATOM-201.2a-001 | 201.2a | Same-name comparison (Bile Blight pattern) | **A** | Covered by name-based comparison used in legend rule and similar. Co-ticket `T14, NEW-S2-01`. |
| ATOM-205.4d-001 | 205.4d | Legendary supertype → legend rule SBA | **A** | `engine/sba.rs:157+` (704.5j). Tests `test_sba_legend_rule_*`. |
| ATOM-209.1-001 | 209.1 | PW enters with loyalty counters = printed loyalty | **A** | `init_etb_counters` at `state/game_state.rs` per memory `6f7bd2e9`. |
| ATOM-306.5-001 | 306.5 | Loyalty is PW-only characteristic | **A** | Loyalty is driven by counters on BattlefieldEntity; only PW cards use them. Boundary test. |
| ATOM-306.5b-001 | 306.5b | PW ETB with loyalty counters = printed loyalty | **A** | Same as 209.1-001. |
| ATOM-306.5c-001 | 306.5c | BF PW loyalty = counter count | **A** | `get_effective_loyalty` reads counters map. |
| ATOM-306.9-001 | 306.9 | PW with 0 loyalty → graveyard SBA | **A** | Duplicate of 122.1e-001. |
| ATOM-306.9-002 | 306.9 | PW with >0 loyalty stays (negative) | **A** | Same SBA, negative case. |

### T14 co-tickets

| ATOM ID | Rule | Summary | Status | Evidence / Note |
|---|---|---|---|---|
| COMP-306.5b+306.8+306.9-001 | 306.5b + 306.8 + 306.9 | PW enters, takes lethal damage, SBA kills | **D** | Co-ticket `T14, T21c`. T14 half (loyalty ETB + zero-loyalty SBA) covered. T21c half (damage-to-PW removes loyalty) is pending — T21c ticket not shipped. Partial. |
| ATOM-704.5j-001 / 002 / 003 | 704.5j | Legend rule variants | **A** | All three covered by `engine/sba.rs:157+` and corresponding tests `test_sba_legend_rule_*`. |
| ATOM-704.5i-001 / 002 | 704.5i | PW 0 loyalty → GY / PW >0 loyalty stays | **A** | Same block as 306.9. |

---

## T16 — Loss SBAs + cleanup re-loop (9 rows)

| ATOM ID | Rule | Summary | Status | Evidence / Note |
|---|---|---|---|---|
| ATOM-117.5-001 | 117.5 | SBA cascade: token lethal → GY → cease-to-exist | **A** | Cascade handled across sba.rs blocks (704.5g lethal → 704.5d token cleanup at `sba.rs:332+`). |
| ATOM-514.3a-001 | 514.3a | Cleanup re-loop: SBAs/triggers → priority → new cleanup | **A** | Cleanup re-loop in `engine/turns.rs` per memory `b80a3a1d`. |
| ATOM-514.3a-002 | 514.3a | Re-looped cleanup runs full TBAs again | **A** | Same. |
| ATOM-704.3-002 | 704.3 | Cleanup SBA shortcut — no priority if no SBAs | **A** | Cleanup step logic. |
| ATOM-704.3-003 | 704.3 | Cleanup SBA re-loop with priority | **A** | Same. |
| ATOM-704.5c-001 | 704.5c | 10+ poison → lose | **A** | Duplicate of 104.3d-001. |
| ATOM-704.5c-002 | 704.5c | 9 poison → no loss (negative) | **A** | `test_sba_poison_9_survives`. |

---

## T13 — Tokens + counter annihilation (6 rows)

| ATOM ID | Rule | Summary | Status | Evidence / Note |
|---|---|---|---|---|
| ATOM-111.7-001 | 111.7 | Token in non-battlefield zone ceases to exist (SBA) | **A** | `engine/sba.rs:332+` (704.5d). |
| ATOM-111.8-002 | 111.8 | Bounced token ceases to exist (SBA) | **A** | Same block. |
| ATOM-704.5d-001 | 704.5d | Token in non-BF zone ceases to exist | **A** | Same block. |
| ATOM-704.5q-001 | 704.5q | +1/+1 and -1/-1 annihilation (unequal) | **A** | `engine/sba.rs:307-329`. |
| ATOM-704.5q-002 | 704.5q | Counter annihilation (equal counts) | **A** | Same. |
| COMP-9A-001 | 704.5q + 704.5f + 704.8 | SBA cascade: counter annihilation + lethal + LKI | **D** | LKI half (704.8) is T20b territory (deathtouch/lifelink LKI) — not yet shipped. Counter annihilation + lethal covered. |

---

## GameConfig (6 rows)

| ATOM ID | Rule | Summary | Status | Evidence / Note |
|---|---|---|---|---|
| ATOM-100.2a-001 | 100.2a | Constructed deck min 60 cards | **A** | `state/game_config.rs:55 min_deck_size: 60`. |
| ATOM-100.2a-002 | 100.2a | No more than 4 copies non-basic | **A** | `max_copies: Some(4)` at `game_config.rs:57`. |
| ATOM-100.2a-003 | 100.2a | Basic lands exempt from copy limit | **D** | `max_copies` exists but enforcement + basic-land exemption not verified. Need to grep for where `max_copies` is consumed during deck validation. |
| ATOM-100.2b-001 | 100.2b | Limited deck min 40 | **A** | `game_config.rs:72 min_deck_size: 40`. |
| ATOM-100.2b-002 | 100.2b | Limited no copy limit | **A** | `game_config.rs:74 max_copies: None`. |
| ATOM-100.4a-001 | 100.4a | Constructed sideboard max 15 | **A** | `sideboard_size: Some(15)` at `game_config.rs:58`. |

---

## T12 — Color identity (2 rows)

| ATOM ID | Rule | Summary | Status | Evidence / Note |
|---|---|---|---|---|
| ATOM-105.2-001 | 105.2 | Colorless is not a color | **A** | `ManaType::Colorless` is a distinct variant, not listed among WUBRG colors. `types/mana.rs`. |
| ATOM-105.2a-002 | 105.2a | Hybrid card is both colors | **D** | Hybrid card color-identity derivation not directly greppable. Need to check `CardData.colors` population from ManaCost. |

---

## T09 — Indestructible (2 rows)

| ATOM ID | Rule | Summary | Status | Evidence / Note |
|---|---|---|---|---|
| ATOM-702.12b-001 | 702.12b | Indestructible prevents lethal damage destruction | **A** | 704.5g check in `sba.rs:105-132` skips indestructible (per T09 ticket). |
| ATOM-702.12b-002 | 702.12b | Indestructible prevents destroy effects | **A** | Destroy action respects indestructible. |

---

## Single-row shipped tickets

| ATOM ID | Rule | Summary | Ticket | Status | Evidence / Note |
|---|---|---|---|---|---|
| ATOM-103.5-001 | 103.5 | London mulligan draw 7 then bottom N | T05 | **B-critical** | Mulligan is stubbed. `state/game.rs:88-90`: "Mulligan handling is stubbed — players always keep their first hand." T05 marked done but the actual mulligan mechanic isn't wired. |
| ATOM-103.6-001 | 103.6 | Starting hand size | T06 | **A** | `starting_hand_size: 7` in GameConfig; `game.rs:98-104` draws that many. |
| ATOM-300.1-001 | 300.1 | CardType enum has exactly 15 types | T07 | **D** | Need to count variants in `types/card_types.rs`. Boundary/enum test. |
| ATOM-301.5b-001 | 301.5b | Equipment ETB unattached | T04 | **A** | `state/battlefield.rs:99 attached_to: None` default. |
| ATOM-107.6-002 | 107.6 | Summoning-sick creature can't activate {Q} | T10 | **D** | Need to verify {Q} (untap symbol) handling — grep shows it exists in Cost but activation-time check not traced. |
| ATOM-108.2b-001 | 108.2b | Tokens aren't cards | T03 | **A** | `GameObject.is_token: bool` distinguishes at type level. Tagged `boundary`. |

---

## ALREADY-IMPL / Architecture / Misc

| ATOM ID | Rule | Summary | Status | Evidence / Note |
|---|---|---|---|---|
| ATOM-106.3-001 | 106.3 | Mana pool stores colored + colorless separately | **A** | Covered by `types/mana.rs ManaPool`. Tagged `Architecture` ticket. |
| ATOM-405.4-004 | 405.4 | Ability controller = activator (not owner) | **A** | Marked ALREADY-IMPLEMENTED in index. Confirmed: ability activation records activator as controller. |
| ATOM-500.5-001 | 500.5 | End-of-step effect expiry + mana pool emptying | **D** | Co-tag `ALREADY-IMPL (mana) + T22`. Mana-pool half is A; T22 half (EOT effect expiry) is pending. |
| ATOM-514.2-001 | 514.2 | Damage removed + "until end of turn" end simultaneously | **D** | Same pattern — damage removal is A, EOT effects pending. |
| ATOM-508.1k-001 | 508.1k | Creatures become attacking (mid-declaration control change) | **D** | Basic case ALREADY-IMPL; mid-declaration control change requires continuous effects. |
| ATOM-703.4p-001 | 703.4p | Cleanup damage removal + EOT effects | **D** | Same split as 514.2-001. |
| COMP-602+605-001 | 602.5a + 605.1a | Summoning sick: tap blocked, sacrifice allowed | **A** | Marked IMPL in index. Confirmed by Cost::Tap summoning-sickness check and Cost::SacrificeSelf not requiring tap. |

---

## Labeling discrepancies

Code uses an older CR numbering for rules 704.5n/p/q. The current CR (2024) renumbers:

- **704.5n** (CR) = Equipment/Fortification on illegal permanent/player → unattach. Code labels this block `704.5p` at `engine/sba.rs:241`.
- **704.5p** (CR) = Battle/creature/other attached to object → unattach (catch-all). Code labels this `704.5q` at `engine/sba.rs:270`.
- **704.5q** (CR) = +1/+1 / -1/-1 annihilation. Code labels this `704.5q` correctly at `engine/sba.rs:307` (only because the label happens to match after the shift).
- `engine/sba.rs:300` TODO is labeled `704.5p` which matches current CR (Aura-that-is-also-creature falls under the 704.5p catch-all).

Behavior is correct; only comments reflect old numbering. Fix-when-touched. No runtime impact.

---

## Summary

| Status | Count | Notes |
|---|---|---|
| **A** | ~45 | Shipped and aligned. |
| **B-critical** | 1 | ATOM-103.5-001 (mulligan stubbed, T05 marker misleading). |
| **B-cosmetic** | 1 | ATOM-704.5n-001 (CR-number labeling drift in sba.rs comments). |
| **C** | ~25 | Not shipped — primarily Equip activation (702.6a) and sacrifice-another primitive (Cost::Sacrifice). Also includes L4-gated ATOMs misclassified as 5-Pre. |
| **D** | ~10 | Need a closer code dive before resolving. Mostly boundary-count ATOMs, EOT-effect interactions, hybrid color identity. |

**Key findings:**

1. **T15b's index scope overstates what shipped.** The three ATOM-702.6a rows (Equip activation) are tagged T15b but not implemented. T15b shipped Aura ETB + enchant filter only. This is a drift the index's Ticket column doesn't reflect.
2. **T05 (mulligan) is a shipped marker over a stub.** `game.rs:88` literally comments "Mulligan handling is stubbed."
3. **CR numbering drift in sba.rs comments** (704.5n/p mislabeled as 704.5p/q). Cosmetic, but affects rule-number grep fidelity.
4. **~5 index rows classify as Phase 5-Pre but their session entries say Phase 8** (ATOM-301.5c-002, 303.4d-001, 704.5p-001, others). The phase-index extractor's classification is noisier than expected.
5. **Sacrifice primitive is narrow.** Only `Cost::SacrificeSelf` ships; `Cost::Sacrifice(filter, count)` is `NotImplemented`. Several 701.21a ATOMs wait on T18c.

**Recommended actions (not part of this audit):**

- 1 B-critical → becomes a small fix ticket: wire London mulligan into `game.rs` setup using the DecisionProvider.
- 1 B-cosmetic → fix-when-touched; note added here.
- ~5 misclassified rows → reclassification note for the session summaries (or a one-time script to adjust); not urgent.
- T15b scope discrepancy → either retroactively ship the 702.6a Equip work as T15c, or move those ATOMs to a new ticket explicitly.

---

## Out of scope (enumerated for tracking, not audited)

**Pending Part-1 tickets** (will be audited when re-scoped):

- T17 (alt costs / additional costs infrastructure): several co-ticket rows
- T18 (casting pipeline 601.2): 46 rows
- T19 (activation, loyalty abilities, timing): 14 rows
- T20 (linked abilities): 8 rows
- T21a–d (zone guards, combat removal, PW damage, requirements): 15 rows combined
- T22 (targeting protection, EOT effects, hexproof, shroud, protection): 17 rows

**Drift (`NEW-*`):** ~45 rows representing work the atomic-tests identified that `implementation-plan-final.md` doesn't track. Needs a separate reconciliation pass. Most are keyword implementations (Bestow, Overload, Unleash, Awaken, Emerge, Undaunted, Improvise, Spectacle, Escape, Foretell, Decayed, Cleave, Living Metal, Impending, Harmonize, Warp, Mayhem, Web-slinging, Sneak, Manifest, Exert, Cloak, Manifest dread) plus several `NEW-CH1-*` cost/X/hybrid primitives.

**Phase 5-Layers bleed-in:** 7 rows misclassified (L04, L05, L06, L04–L12, L18). Belong to Phase 5-Layers index.
