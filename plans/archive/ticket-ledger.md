# Ticket Ledger

Single-page status of Part 1 tickets. Grounded in `implementation-plan-final.md` + `git log`. Use this as the source of truth for "what's shipped, what's pending, what's next."

Last updated: 2026-04-17.

---

## Part 1 — Shipped (confirmed by git log)

| Ticket | Scope (from plan) | Commit | Primary CR rules touched |
|---|---|---|---|
| **T01** | Counters on `BattlefieldEntity` + expand `CounterType` enum (evergreen keyword counters) | `a6b22c9 Closes T1` | 122.1, 122.1b |
| **T02** | Player counters: `poison_counters`, `commander_damage_taken` on `PlayerState` | `5189e39 Closes T02` | 122.1f, 704.5c (data) |
| **T03** | `is_token`, `is_copy` flags on `GameObject` | `bc4ada2 T03, T05, T06 — Tier 1 data model additions` | 108.2b, 111.7, 704.5d (data only) |
| **T04** | Attachment tracking (`attached_to`, `attached_by`) + `cleanup_zone_state` detach hook | `3d8dce7 T04: Add attachment tracking` | 301.5, 301.5b (data) |
| **T05** | `color_indicator: Option<Vec<Color>>` on `CardData` | `bc4ada2 T03, T05, T06` (same commit) | 202.2a (data only) |
| **T06** | `x_value: Option<u32>` on `BattlefieldEntity` + carry from `StackEntry` | `bc4ada2 T03, T05, T06` (same commit) | 107.3g, 107.3j (data) |
| **T07** | Resolved in-place (E8 signedness verified, E10 battle type fix, E11 enum completeness verified). No ticket/commit. | — | N/A |
| **T09** | Replace `summoning_sick: bool` with turn-based tracking (`entered_battlefield_turn`, `controller_since_turn`) + `has_summoning_sickness` query | `4b31843 T09: Replacing summoning_sick bool` | 302.1 (summoning sickness) |
| **T10** | Summoning-sickness guard on `Cost::Untap` ({Q}) | `67e863d T10: Add summoning sickness check to Cost::Untap` | 107.6 |
| **T11** | `source: Option<ObjectId>` on `GameEvent::LifeChanged` | `a11cbef T11: Add source field to LifeChanged event` | 119 (life changes attribution) |
| **T12** | Mana spending restrictions — **design spike only** (document: `plans/mana-restrictions-design.md`) | `a37980d T12 Design Spike done` | 106.4 (design, no code) |
| **T12b** | `ManaPool` sidecar types + methods for restricted/granted mana | `2d96f79 mana: add ManaPool sidecar (T12b)` | 106.4, 106.6 |
| **T13** | Counter annihilation SBA + token cease-to-exist SBA | `cacdb0f T13: Counter annihilation + token cease-to-exist SBAs` | 704.5d, 704.5q (counter), 111.8 |
| **T14** | Legend rule SBA + planeswalker 0-loyalty SBA | `72c8f98 T14: add legend rule and planeswalker 0-loyalty SBAs` | 704.5i, 704.5j, 205.4d, 306.5 |
| **T15** | Aura/Equipment legality SBAs (704.5m/n/p) | `e0e8a33 T15 — Aura/Equipment legality SBAs` | 704.5m, 704.5n, 704.5p, 303.4c |
| **T15b** | Aura attachment logic: `attach_aura_on_etb`, `enchant_filter` on `CardData`, unified `SelectionFilter` validation, deletion of `EnchantRestriction` | `0eee19f engine: complete T15b Aura attachment` | 303.4, 303.4a, 303.4c, 303.4d, 303.4e, 303.4f, 702.5a, 702.5d |
| **T16** | SBAs — poison loss, commander damage loss, indestructible-aware lethal damage, cleanup re-loop | `e15a4e8 T16: Remaining SBAs — poison, commander, indestructible, cleanup re-loop` | 704.5a, 704.5b, 704.5c, 704.5g (indestructible-aware), 514.3a |
| **T17** | Alternative/additional cost framework — type definitions (`AltCost`, `AddCost` structs) | `dfe0e57 T17 — Alternative and additional cost framework` | 117.9 (types only; wiring in T18a) |
| **T18a** | Casting pipeline restructure — X cost extraction, alt/additional cost assembly, rollback infra | `6ae2d1e T18a: Casting pipeline restructure` | 601.2a, 601.2b (X, alt costs), 601.2f, 601.2h |
| **SPECIAL-1a/b/c** | `DecisionProvider` refactor: generic trait + typed `ask_*` functions, 4-primitive DP, legacy trait removed | `ffc2e6f` / `0497678` / `eeb3108` | architecture |
| **SPECIAL-2** | Priority retry on failed cast + `activate_ability` rollback (combined with 601.2g implementation) | `a30ba97 SPECIAL-2 + 601.2g: mana-ability window, priority retry` | 601.2g, 117.3d |
| **SPECIAL-8** | Blocker legality pre-filter + CR 509.1c retry loop | `5570df4 SPECIAL8: engine: blocker legality pre-filter` | 509.1c |

**Total Part 1 shipped:** 19 T-tickets (T01–T18a excluding T07 and T08-none) + 5 SPECIAL tickets.

---

## Part 1 — Not yet shipped

| Ticket | Scope | Blocker / Status |
|---|---|---|
| **T18b** | Mode choice (`ChoiceKind::ChooseModes`, `ask_choose_modes`) + conditional targets + target uniqueness rules (115.3, 115.3/4) | Ready — `SPECIAL-1c` unblocked it. Branch `T18b-mode-choice-and-target-rules` exists. |
| **T18c** | `Cost::Sacrifice(filter, count)` implementation, payment ordering, distribution (damage/counters), partial resolution | Parallel with T18b/d. No blocker. |
| **T18d** | Casting restrictions (can't-cast static abilities), no-mana-cost guard, legendary sorcery rule | Parallel with T18b/c. Depends on T18a (landed). |
| **T19** | Activation restrictions + zone-activated abilities (graveyard-only abilities, sorcery-speed loyalty, etc.) | No blocker. Parallel with all of T18. |
| **T20** | Linked abilities + mana ability debug assertion | No blocker. |
| **T20b** | LKI system | **DEFERRED TO PART 2** — requires `EffectiveCharacteristics` from layer work. |
| **T12c** | Mana restrictions engine integration (spend & restriction checks) | Blocked on T17 (landed) + T12b (landed). Ready. |
| **T12d** | First restricted-mana cards + persistence | Depends on T12c + L04 (persistence). |
| **T21a** | Zone guards (instants/sorceries can't ETB) + CastInfo carried to permanent | Depends on T17 (landed). Ready. |
| **T21b** | Combat removal + evasion framework + trample co-assigned damage | No blocker. |
| **T21c** | Infect/Wither + planeswalker damage routing + Toxic | Depends on T01, T02 (both landed). Ready. |
| **T21d** | Combat requirements solver (attack/block maximization) | Depends on T21b. |
| **T22** | Duration enum + turn structure + targeting fixes (hexproof, shroud, protection, EOT expiry) | No blocker. |
| **SPECIAL-3/4/5/6/9** | Various DP cleanup / test-harness / property-test QoL tickets | Non-blocking QoL. Pick up anytime. |

**Total Part 1 pending:** 13 T-tickets + ~5 SPECIAL. T20b deferred to Part 2.

---

## Key findings from the ledger

1. **The phase-index Ticket column is not a reliable mapping to implementation-plan-final tickets.** Example: `ATOM-103.5-001` (London mulligan) is tagged `T05` in the phase-index, but T05 is `color_indicator`. The session authors wrote ticket tags without cross-referencing the plan. **The Ticket column should not be trusted for planning; use the CR rule number and consult this ledger or implementation-plan-final directly.**

2. **`implementation-plan-final.md` is the authoritative scope document.** The phase-index and session summaries are the authoritative *rule catalog*. The two drift — there is no automated link between them.

3. **The `NEW-*` bloc (~45 ATOMs) is the real spec-vs-plan drift.** These are ATOMs the atomic-test pass identified that `implementation-plan-final.md` doesn't track as tickets. Most are Phase 8 keyword implementations (Bestow, Overload, Awaken, Emerge, Improvise, Spectacle, Escape, Foretell, Cleave, Impending, Harmonize, Warp, Mayhem, Web-slinging, Sneak, Manifest, Exert, Cloak) plus ~10 Ch.1 primitives (`NEW-CH1-013/014/016/017/018` = hybrid/Phyrexian/generic-reduction mana mechanics). **Decision needed:** fold into existing tickets, create new tickets, or defer.

4. **T07 and T08 don't exist as commits.** T07 was resolved in-place with no separate ticket. T08 is not listed in `implementation-plan-final.md` — the numbering skipped from T06 to T09.

5. **SPECIAL tickets accreted after the original plan.** SPECIAL-1 (a/b/c), SPECIAL-2, SPECIAL-5, SPECIAL-6, SPECIAL-8, SPECIAL-9 were added during execution (DP refactor, fuzz-gate blockers). They are tracked in `implementation-plan-final.md` §15b/c/d but aren't in the original Part 1 numbering.

---

## What I don't have confidence in yet

- **Rule → ticket mapping in the "CR rules touched" column above** is derived from the plan's `Source` (E-item) + `Steps` fields, not from reading the commit's diff. A proper cross-check would grep the commit's diff for rule number mentions and compare. Worth doing if you want a trustworthy mapping, but skippable for planning purposes.
- **Completeness of "Shipped" row claims vs. their `Tests:` list in the plan.** I did not verify that every listed test in each ticket's plan entry was actually written. That's a separate, smaller audit.

---

## Recommended next actions (pick ONE; ordered by tractability)

1. **T18d (Small, no blockers, highest ROI).** Casting restrictions + no-mana-cost guard + legendary sorcery. Small scope, finishes a concrete rule family (118.6, 202.1b, 205.4e). Good "get-back-on-the-horse" ticket after the T18 explosion.
2. **T19 (Medium, no blockers).** Activation restrictions + zone-activated abilities. Self-contained, no dependencies on other pending tickets.
3. **T21a (Small, T17 landed).** Zone guards + CastInfo carry. Small scope, concrete.
4. **Fold the `NEW-*` bloc into the plan.** Open a short session to decide: which NEW-* become new Part 1 tickets, which get deferred to Phase 8, which get folded into existing pending tickets (e.g., `NEW-CH1-016` hybrid payment probably belongs in T12c or a T12-family ticket).

What I'd **not** recommend as the immediate next action: T18b, T18c, or T22. T18b/c are still large and within the area that exploded. T22 covers 7 E-items and 5+ rule families — high surface area.

---

## How to use this document going forward

- **When scoping a new ticket:** reference this ledger + `implementation-plan-final.md` §Part 1 for the ticket's detailed Steps. Do not consult the phase-index Ticket column for scope.
- **When looking for rule coverage:** consult the phase-index + session summaries for ATOMs, then cross-reference to this ledger to see if an implementing ticket exists.
- **When a new rule-level concern arises during implementation:** decide — is it in scope for the active ticket, or does it become a new bookkeeping entry here (as a pending ticket or a `NEW-*` consideration)?
- **Update cadence:** update this file after each ticket lands, before starting the next. Keep it scannable (≤ 2 pages).
