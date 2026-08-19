# Session 7A — Condensed Summary (Rules 700.x + 701.x)

> **Source:** `plans/atomic-tests/session-7a.md`
> **CR Source:** `MTG-Rules/LLM-Chapter-Splits/ch7-pt-1.txt` (rules 700.1–701.68)
> **Scope:** General additional rules (700.x) + keyword actions (701.x)

---

## ATOM Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-700.2a-001 | 700.2a | Modal mode choice at cast — illegal modes excluded | Phase 5-Pre | T18 | |
| ATOM-700.2b-001 | 700.2b | Modal triggered ability — mode choice + removal if no legal mode | Phase 7 | NEW | |
| ATOM-700.2c-001 | 700.2c | Mode-conditional targeting — unchosen modes need no targets | Phase 5-Pre | T18 | |
| ATOM-700.2d-001 | 700.2d | Mode uniqueness enforcement — can't choose same mode twice | Phase 5-Pre | T18 | |
| ATOM-700.2d-002 | 700.2d | Repeated mode when allowed — effects applied N times in order | Phase 8 | NEW | |
| ATOM-700.2e-001 | 700.2e | Opponent chooses mode when specified | Phase 5-Pre | T18 | |
| ATOM-700.2g-001 | 700.2g | Spell copy preserves modes | Phase 7 | NEW (D19) | |
| ATOM-700.2h-001 | 700.2h | Per-mode additional costs aggregated in casting | Phase 5-Pre | T18 | |
| ATOM-700.2i-001 | 700.2i | Pawprint budget mode selection | Phase 8 | NEW | |
| ATOM-700.3a-001 | 700.3a | Pile formation — each object in exactly one pile | Phase 8 | NEW | |
| ATOM-700.3c-001 | 700.3c | Pile formation — no zone-change events during forming | Phase 8 | NEW | |
| ATOM-700.5-001 | 700.5 | Devotion to single color — mana symbol count | Phase 8 | NEW | |
| ATOM-700.5-002 | 700.5 | Devotion to two colors — hybrid counts once | Phase 8 | NEW | |
| ATOM-700.5a-001 | 700.5a | Devotion partial-layer (after L1–L3, before L4–L7) | Phase 5-Layers | NEW (6b-D14) | dependency, layers |
| ATOM-700.5a-002 | 700.5a | Devotion modifier effect (Altar of the Pantheon) | Phase 5-Layers | NEW | |
| ATOM-701.3a-001 | 701.3a | Attach Equipment to creature — basic | Phase 5-Pre | T15 | |
| ATOM-701.3a-002 | 701.3a | Attach to invalid target — rejected | Phase 5-Pre | T15 | |
| ATOM-701.3b-001 | 701.3b | Failed attach — no movement | Phase 5-Pre | T15 | |
| ATOM-701.3b-002 | 701.3b | Reattach to same target — no-op | Phase 5-Pre | T15 | |
| ATOM-701.3b-003 | 701.3b | Non-Aura/Equipment attach — does nothing | Phase 5-Pre | T15 | |
| ATOM-701.3c-001 | 701.3c | Reattach to different target — new timestamp | Phase 5-Pre | T15 | dependency, layers |
| ATOM-701.3d-001 | 701.3d | Unattach Equipment — stays on battlefield | Phase 5-Pre | T15b | |
| ATOM-701.3d-002 | 701.3d | Creature leaves → Equipment becomes unattached | Phase 5-Pre | T15b | |
| ATOM-701.7a-001 | 701.7a | Create tokens — specified characteristics on battlefield | Phase 8 | NEW | |
| ATOM-701.7b-001 | 701.7b | Token creation replacement ordering (creation → continuous → ETB) | Phase 8 | NEW | dependency, replacement-effects, layers |
| ATOM-701.8b-001 | 701.8b | Destroy delta tagging — destroy vs sacrifice distinction | Phase 6 | NEW (D20) | dependency, replacement-effects |
| ATOM-701.8c-001 | 701.8c | Regeneration replaces destruction | Phase 6 | NEW | dependency, replacement-effects |
| ATOM-701.9b-001 | 701.9b | Random discard — engine picks at random | Phase 8 | NEW | |
| ATOM-701.9b-002 | 701.9b | Opponent-chosen discard | Phase 8 | NEW | |
| ATOM-701.9c-001 | 701.9c | Discard to hidden zone — characteristic undefined | Phase 8 | NEW | dependency, replacement-effects |
| ATOM-701.10a-001 | 701.10a | Double P/T — L7c continuous effect (+X/+Y) | Phase 5-Layers | NEW | |
| ATOM-701.10b-001 | 701.10b | Double power only — snapshot at resolution | Phase 5-Layers | NEW | |
| ATOM-701.10c-001 | 701.10c | Double negative power — becomes more negative | Phase 5-Layers | NEW | |
| ATOM-701.10d-001 | 701.10d | Double life total — via gain/loss event | Phase 8 | NEW | |
| ATOM-701.10d-002 | 701.10d | Double negative life total — via loss event | Phase 8 | NEW | |
| ATOM-701.10e-001 | 701.10e | Double counters — add same count | Phase 8 | NEW | |
| ATOM-701.10f-001 | 701.10f | Double mana — new mana is unrestricted | Phase 8 | NEW | |
| ATOM-701.10g-001 | 701.10g | Damage doubling replacement effect | Phase 6 | NEW | dependency, replacement-effects |
| ATOM-701.11a-001 | 701.11a | Triple P/T — L7c continuous effect (+2X/+2Y) | Phase 5-Layers | NEW | |
| ATOM-701.11b-001 | 701.11b | Triple power only | Phase 5-Layers | NEW | |
| ATOM-701.11c-001 | 701.11c | Triple negative power | Phase 5-Layers | NEW | |
| ATOM-701.12a-001 | 701.12a | Exchange all-or-nothing — partial failure cancels all | Phase 8 | NEW | |
| ATOM-701.12b-001 | 701.12b | Control exchange — simultaneous swap | Phase 8 | NEW | |
| ATOM-701.12b-002 | 701.12b | Same-controller exchange — no-op | Phase 8 | NEW | |
| ATOM-701.12c-001 | 701.12c | Life total exchange — via gain/loss events | Phase 8 | NEW | |
| ATOM-701.12c-002 | 701.12c | Life exchange blocked by can't-gain-life (119.7) | Phase 8 | NEW | dependency, replacement-effects |
| ATOM-701.12d-001 | 701.12d | Mass zone exchange (hand ↔ graveyard) | Phase 8 | NEW | |
| ATOM-701.12g-001 | 701.12g | Numerical value exchange (life ↔ power) | Phase 8 | NEW | |
| ATOM-701.14a-001 | 701.14a | Fight — mutual simultaneous damage | Phase 8 | NEW | |
| ATOM-701.14b-001 | 701.14b | Fight — creature left battlefield, neither fights | Phase 8 | NEW | |
| ATOM-701.14b-002 | 701.14b | Fight — illegal target, neither fights | Phase 8 | NEW | |
| ATOM-701.14c-001 | 701.14c | Self-fight — double power self-damage | Phase 8 | NEW | |
| ATOM-701.14d-001 | 701.14d | Fight damage is NOT combat damage | Phase 8 | NEW | |
| ATOM-701.16a-001 | 701.16a | Investigate — create Clue token | Phase 8 | NEW | |
| ATOM-701.17a-001 | 701.17a | Mill N — top N to graveyard | Phase 8 | NEW | |
| ATOM-701.17b-001 | 701.17b | Mill capped to library size | Phase 8 | NEW | |
| ATOM-701.17b-002 | 701.17b | Mill as cost — unpayable if library too small | Phase 8 | NEW | |
| ATOM-701.19a-001 | 701.19a | Regeneration shield — one-shot replacement on destroy | Phase 6 | NEW | replacement-effects |
| ATOM-701.19a-002 | 701.19a | Regen shield single-use — second destroy succeeds | Phase 6 | NEW | |
| ATOM-701.19b-001 | 701.19b | Static regeneration — replaces every destruction | Phase 6 | NEW | |
| ATOM-701.19c-001 | 701.19c | Can't-be-regenerated blocks shield application | Phase 6 | NEW | |
| ATOM-701.20a-001 | 701.20a | Reveal — mark cards visible to all players | Phase 8 | NEW | |
| ATOM-701.21a-001 | 701.21a | Sacrifice — bypasses destroy/indestructible | Phase 5-Pre | T15 | |
| ATOM-701.21a-002 | 701.21a | Can't sacrifice what you don't control | Phase 5-Pre | T15 | |
| ATOM-701.21a-003 | 701.21a | Can't sacrifice non-permanents | Phase 5-Pre | T15 | |
| ATOM-701.22a-001 | 701.22a | Scry N — top/bottom reorder | Phase 8 | NEW | |
| ATOM-701.22b-001 | 701.22b | Scry 0 — no event, no trigger | Phase 8 | NEW | |
| ATOM-701.23a-001 | 701.23a | Search zone — find matching card | Phase 8 | NEW | |
| ATOM-701.23b-001 | 701.23b | Fail-to-find in hidden zone — optional | Phase 8 | NEW | |
| ATOM-701.23d-001 | 701.23d | Mandatory quantity search — must find N | Phase 8 | NEW | |
| ATOM-701.24b-001 | 701.24b | Search-then-shuffle — found card excluded | Phase 8 | NEW | |
| ATOM-701.25a-001 | 701.25a | Surveil N — top cards to GY or top | Phase 8 | NEW | |
| ATOM-701.25c-001 | 701.25c | Surveil 0 — no event, no trigger | Phase 8 | NEW | |
| ATOM-701.29a-001 | 701.29a | Fateseal — scry on opponent's library | Phase 8 | NEW | |
| ATOM-701.34a-001 | 701.34a | Proliferate — +1 of each counter kind | Phase 8 | NEW | |
| ATOM-701.35a-001 | 701.35a | Detain — can't attack/block/activate until next turn | Phase 8 | NEW | |
| ATOM-701.36a-001 | 701.36a | Populate — copy chosen creature token | Phase 8 | NEW | |
| ATOM-701.36b-001 | 701.36b | Populate with no creature tokens — no-op | Phase 8 | NEW | |
| ATOM-701.37a-001 | 701.37a | Monstrosity N — counters + designation | Phase 8 | NEW | |
| ATOM-701.37a-002 | 701.37a | Monstrosity guard — already monstrous = no-op | Phase 8 | NEW | |
| ATOM-701.39a-001 | 701.39a | Bolster N — counters on lowest-toughness creature | Phase 8 | NEW | |
| ATOM-701.39a-002 | 701.39a | Bolster tie-break — controller chooses | Phase 8 | NEW | |
| ATOM-701.40a-001 | 701.40a | Manifest — face-down 2/2 creature | Phase 5-Pre | NEW | |
| ATOM-701.40b-001 | 701.40b | Turn manifested creature face up — pay mana cost | Phase 5-Pre | NEW | |
| ATOM-701.40b-002 | 701.40b | Non-creature can't turn face up via manifest | Phase 5-Pre | NEW | |
| ATOM-701.40f-001 | 701.40f | ETB prohibition prevents manifest | Phase 6 | NEW | |
| ATOM-701.40g-001 | 701.40g | Instant/sorcery can't turn face up — stays face-down | Phase 5-Pre | NEW | |
| ATOM-701.41a-001 | 701.41a | Support N on permanent — +1/+1 on up to N OTHER creatures | Phase 8 | NEW | |
| ATOM-701.41a-002 | 701.41a | Support N on instant/sorcery — no "other" restriction | Phase 8 | NEW | |
| ATOM-701.43a-001 | 701.43a | Exert — skip next untap step | Phase 5-Pre | NEW | |
| ATOM-701.43b-001 | 701.43b | Exert stacking — both expire same untap step | Phase 5-Pre | NEW | |
| ATOM-701.43d-001 | 701.43d | Exert as optional attack cost + linked trigger | Phase 7 | NEW | |
| ATOM-701.44a-001 | 701.44a | Explore — land path (to hand) | Phase 8 | NEW | |
| ATOM-701.44a-002 | 701.44a | Explore — nonland path (counter + GY) | Phase 8 | NEW | |
| ATOM-701.44a-003 | 701.44a | Explore — nonland, keep on top | Phase 8 | NEW | |
| ATOM-701.44c-001 | 701.44c | Explore LKI — permanent left battlefield | Phase 8 | NEW | |
| ATOM-701.46a-001 | 701.46a | Adapt N — counters if none present | Phase 8 | NEW | |
| ATOM-701.46a-002 | 701.46a | Adapt guard — has counters = no-op | Phase 8 | NEW | |
| ATOM-701.47a-001 | 701.47a | Amass — no Army → create token + counters + subtype | Phase 8 | NEW | |
| ATOM-701.47a-002 | 701.47a | Amass — existing Army gets counters | Phase 8 | NEW | |
| ATOM-701.47a-003 | 701.47a | Amass — grants new subtype to existing Army | Phase 8 | NEW | |
| ATOM-701.50a-001 | 701.50a | Connive — draw/discard, nonland → +1/+1 counter | Phase 8 | NEW | |
| ATOM-701.50a-002 | 701.50a | Connive — discard land → no counter | Phase 8 | NEW | |
| ATOM-701.50e-001 | 701.50e | Connive N — draw N/discard N, counters = nonland count | Phase 8 | NEW | |
| ATOM-701.53a-001 | 701.53a | Incubate N — DFC Incubator token + counters | Phase 9 (D3) | D3 | |
| ATOM-701.53b-001 | 701.53b | Incubator transforms to 0/0 Phyrexian creature | Phase 9 (D3) | D3 | |
| ATOM-701.54a-001 | 701.54a | Ring tempts — emblem + ring-bearer designation | Phase 8 | NEW | |
| ATOM-701.54a-002 | 701.54a | Ring tempts again — redesignate + progressive unlock | Phase 8 | NEW | |
| ATOM-701.54c-001 | 701.54c | Ring — all 4 progressive abilities at temptation 4 | Phase 8 | NEW | |
| ATOM-701.55a-001 | 701.55a | Villainous choice — pick A or B | Phase 8 | NEW | |
| ATOM-701.55b-001 | 701.55b | Villainous choice — impossible option, perform as much as possible | Phase 8 | NEW | |
| ATOM-701.57a-001 | 701.57a | Discover N — exile loop + free cast | Phase 8 | NEW | |
| ATOM-701.57a-002 | 701.57a | Discover — choose hand instead of cast | Phase 8 | NEW | |
| ATOM-701.57a-003 | 701.57a | Discover — whiff (all lands) | Phase 8 | NEW | |
| ATOM-701.58a-001 | 701.58a | Cloak — face-down 2/2 with ward {2} | Phase 5-Pre | NEW | |
| ATOM-701.59a-001 | 701.59a | Collect evidence N — exile GY cards with MV ≥ N | Phase 8 | NEW | |
| ATOM-701.59b-001 | 701.59b | Collect evidence — can't choose if GY MV insufficient | Phase 8 | NEW | |
| ATOM-701.60a-001 | 701.60a | Suspect — gains suspected designation | Phase 8 | NEW | |
| ATOM-701.60c-001 | 701.60c | Suspected grants menace + can't block | Phase 8 | NEW | |
| ATOM-701.62a-001 | 701.62a | Manifest dread — look at 2, manifest 1, GY other | Phase 5-Pre | NEW | |
| ATOM-701.63a-001 | 701.63a | Endure N — counters on self path | Phase 8 | NEW | |
| ATOM-701.63a-002 | 701.63a | Endure N — Spirit token path | Phase 8 | NEW | |
| ATOM-701.63b-001 | 701.63b | Endure 0 — no-op | Phase 8 | NEW | |
| ATOM-701.64a-001 | 701.64a | Harness — gains harnessed designation | Phase 8 | NEW | |
| ATOM-701.64a-002 | 701.64a | Harness guard — already harnessed = no-op | Phase 8 | NEW | |
| ATOM-701.65a-001 | 701.65a | Airbend — exile + persistent cast-from-exile for {2} | Phase 8 | NEW | |
| ATOM-701.65a-002 | 701.65a | Airbend — owner casts exiled card for {2} | Phase 8 | NEW | |
| ATOM-701.65a-003 | 701.65a | Airbend — batch exile of multiple objects | Phase 8 | NEW | |
| ATOM-701.66a-001 | 701.66a | Earthbend N — land animation + counters + delayed trigger | Phase 5-Layers/7 | NEW | |
| ATOM-701.66a-002 | 701.66a | Earthbend — animated land dies, delayed trigger returns it | Phase 7 | NEW | |
| ATOM-701.67a-001 | 701.67a | Waterbend — tap artifacts/creatures for generic portion | Phase 8 | NEW | |
| ATOM-701.67b-001 | 701.67b | Waterbend — cost isolation (only waterbend's generic) | Phase 8 | NEW | |
| ATOM-701.68a-001 | 701.68a | Blight N — put N -1/-1 counters on own creature | Phase 8 | NEW | |
| ATOM-701.68b-001 | 701.68b | Blight legality — must control a creature | Phase 8 | NEW | |

## BOUNDARY-DEF Index

| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| BOUNDARY-DEF-700.6-001 | 700.6 | Historic = legendary OR artifact OR Saga | Phase 8 | NEW | |
| BOUNDARY-DEF-700.9-001 | 700.9 | Modified = counters OR equipped OR friendly Aura | Phase 8 | NEW | |
| BOUNDARY-DEF-700.12-001 | 700.12 | Outlaw = Assassin/Mercenary/Pirate/Rogue/Warlock | Phase 8 | NEW | |

## COMP Index

| ID | Rules Composed | Summary | Phase |
|----|---------------|---------|-------|
| COMP-7A-001 | 701.21a + 701.8b | Sacrifice indestructible bypasses destroy replacement | Phase 5-Pre/6 |
| COMP-7A-002 | 701.14a + 701.14d + 702.15 | Fight + lifelink + non-combat damage flag | Phase 8 |
| COMP-7A-003 | 701.34a + 701.46a | Proliferate + adapt guard interaction | Phase 8 |
| COMP-7A-004 | 701.19a + 701.19c + 701.8a | Regen shield vs can't-be-regenerated + destroy | Phase 6 |
| COMP-7A-005 | 701.3d + 701.21a | Equipment unattach on creature sacrifice | Phase 5-Pre |
| COMP-7A-006 | 701.10a + 613.4b/c | Double P/T + layer ordering with set-P/T effect | Phase 5-Layers |

## META Entries

No META entries in this session.

## Classification Summary Table

| Rule | Classification | Phase | Ticket | Notes |
|------|---------------|-------|--------|-------|
| 700.1 | PURE-DEF | — | — | Event definition |
| 700.2 | PURE-DEF | — | — | Modal definition (header) |
| 700.2a | TESTABLE | Phase 5-Pre | T18 | Modal mode choice at cast time |
| 700.2b | TESTABLE | Phase 7 | NEW | Modal triggered ability mode choice + removal |
| 700.2c | TESTABLE | Phase 5-Pre | T18 | Mode-conditional targeting |
| 700.2d | TESTABLE | Phase 5-Pre / Phase 8 | T18 / NEW | Mode uniqueness + repeated modes |
| 700.2e | TESTABLE | Phase 5-Pre | T18 | Opponent mode choice |
| 700.2f | PURE-DEF | — | — | Target change can't change mode |
| 700.2g | TESTABLE | Phase 7 | NEW (D19) | Copy preserves modes |
| 700.2h | TESTABLE | Phase 5-Pre | T18 | Per-mode additional costs |
| 700.2i | TESTABLE | Phase 8 | NEW | Pawprint budget |
| 700.3 | PURE-DEF | — | — | Piles header |
| 700.3a | TESTABLE | Phase 8 | NEW | One pile per object |
| 700.3b | PURE-DEF | — | — | Pile not an object |
| 700.3c | TESTABLE | Phase 8 | NEW | Pile zone invariant |
| 700.3d | PURE-DEF | — | — | Pile can be empty |
| 700.4 | ALREADY-IMPL | — | — | "Dies" definition |
| 700.5 | TESTABLE | Phase 8 | NEW | Devotion single color |
| 700.5 (2-color) | TESTABLE | Phase 8 | NEW | Devotion two colors |
| 700.5a | TESTABLE | Phase 5-Layers | NEW (6b-D14) | Devotion partial-layer (2 tests) |
| 700.6 | BOUNDARY-DEF | Phase 8 | NEW | Historic predicate |
| 700.7 | DEFERRED | Phase 7 | — | "This [something]" identity |
| 700.8–700.8d | DEFERRED | Phase 8 | — | Party mechanic |
| 700.9 | BOUNDARY-DEF | Phase 8 | NEW | Modified predicate |
| 700.10 | DEFERRED | Phase 8 | — | "Activated this turn" |
| 700.11 | DEFERRED | Phase 8 | — | Descended |
| 700.12 | BOUNDARY-DEF | Phase 8 | NEW | Outlaw predicate |
| 700.12a | PURE-DEF | — | — | Outlaw permanents clarification |
| 700.13 | DEFERRED | Phase 8 | — | Committing a crime |
| 700.14 | DEFERRED | Phase 8 | — | Expend |
| 700.15 | PURE-DEF | — | — | "Enters" shorthand |
| 701.1 | PURE-DEF | — | — | Keyword actions intro |
| 701.2a | ALREADY-IMPL | — | — | Activate |
| 701.3a | TESTABLE | Phase 5-Pre | T15 | Attach basic |
| 701.3a (illegal) | TESTABLE | Phase 5-Pre | T15 | Attach to invalid target |
| 701.3b (can't attach) | TESTABLE | Phase 5-Pre | T15 | Failed attach no movement |
| 701.3b (same target) | TESTABLE | Phase 5-Pre | T15 | Same-target no-op |
| 701.3b (non-attachment) | TESTABLE | Phase 5-Pre | T15 | Non-Aura/Equip does nothing |
| 701.3c | TESTABLE | Phase 5-Pre | T15 | Reattach new timestamp |
| 701.3d (unattach) | TESTABLE | Phase 5-Pre | T15b | Unattach Equipment |
| 701.3d (creature leaves) | TESTABLE | Phase 5-Pre | T15b | Creature leaves → unattach |
| 701.4–701.4b | DEFERRED | Phase 8 | — | Behold |
| 701.5a | ALREADY-IMPL | — | — | Cast |
| 701.5b | PURE-DEF | — | — | "Cast a card" |
| 701.6a | ALREADY-IMPL | — | — | Counter |
| 701.6b | ALREADY-IMPL | — | — | No cost refund |
| 701.7a | TESTABLE | Phase 8 | NEW | Create tokens |
| 701.7b | TESTABLE | Phase 8 | NEW | Token creation replacement ordering |
| 701.7c | PURE-DEF | — | — | Errata |
| 701.8a | ALREADY-IMPL | — | — | Destroy |
| 701.8b | TESTABLE | Phase 6 | NEW (D20) | Destroy delta tagging |
| 701.8c | TESTABLE | Phase 6 | NEW | Regeneration replaces destroy |
| 701.9a | ALREADY-IMPL | — | — | Discard |
| 701.9b | TESTABLE | Phase 8 | NEW | Random / opponent discard |
| 701.9c | TESTABLE | Phase 8 | NEW | Discard to hidden zone |
| 701.10a | TESTABLE | Phase 5-Layers | NEW | Double P/T as L7c |
| 701.10b | TESTABLE | Phase 5-Layers | NEW | Double snapshot |
| 701.10c | TESTABLE | Phase 5-Layers | NEW | Negative power doubling |
| 701.10d | TESTABLE | Phase 8 | NEW | Double life total |
| 701.10e | TESTABLE | Phase 8 | NEW | Double counters |
| 701.10f | TESTABLE | Phase 8 | NEW | Double mana |
| 701.10g | TESTABLE | Phase 6 | NEW | Damage doubling replacement |
| 701.11a | TESTABLE | Phase 5-Layers | NEW | Triple P/T as L7c |
| 701.11b | TESTABLE | Phase 5-Layers | NEW | Triple power only |
| 701.11c | TESTABLE | Phase 5-Layers | NEW | Negative power tripling |
| 701.12a | TESTABLE | Phase 8 | NEW | Exchange all-or-nothing |
| 701.12b | TESTABLE | Phase 8 | NEW | Control exchange |
| 701.12b (same ctrl) | TESTABLE | Phase 8 | NEW | Same-controller no-op |
| 701.12c | TESTABLE | Phase 8 | NEW | Life total exchange |
| 701.12c (can't gain) | TESTABLE | Phase 8 | NEW | Life exchange + can't gain |
| 701.12d | TESTABLE | Phase 8 | NEW | Zone card exchange |
| 701.12e | DEFERRED | Phase 8 | — | Attachment transfer |
| 701.12f | DEFERRED | Phase 8 | — | Empty zone exchange |
| 701.12g | TESTABLE | Phase 8 | NEW | Numerical value exchange |
| 701.12h | DEFERRED | Phase 8 | — | Text-box exchange |
| 701.13a | ALREADY-IMPL | — | — | Exile |
| 701.14a | TESTABLE | Phase 8 | NEW | Fight mutual damage |
| 701.14b (no longer on BF) | TESTABLE | Phase 8 | NEW | Fight validity |
| 701.14b (illegal target) | TESTABLE | Phase 8 | NEW | Fight illegal target |
| 701.14c | TESTABLE | Phase 8 | NEW | Self-fight |
| 701.14d | TESTABLE | Phase 8 | NEW | Fight non-combat damage |
| 701.15a–d | DEFERRED | Phase 9 | — | Goad |
| 701.16a | TESTABLE | Phase 8 | NEW | Investigate / Clue token |
| 701.17a | TESTABLE | Phase 8 | NEW | Mill basic |
| 701.17b | TESTABLE | Phase 8 | NEW | Mill cap + cost legality |
| 701.17c | DEFERRED | Phase 8 | — | Milled card tracking |
| 701.17d | DEFERRED | Phase 8 | — | Multi-mill references |
| 701.18a | ALREADY-IMPL | — | — | Play a land |
| 701.18b | DEFERRED | Phase 8 | — | Play permission via continuous effects (no new enum) |
| 701.18c–e | PURE-DEF | — | — | Play terminology |
| 701.19a | TESTABLE | Phase 6 | NEW | Regen shield (one-shot) |
| 701.19a (single-use) | TESTABLE | Phase 6 | NEW | Regen single-use |
| 701.19b | TESTABLE | Phase 6 | NEW | Static regeneration |
| 701.19c | TESTABLE | Phase 6 | NEW | Can't-be-regenerated |
| 701.20a | TESTABLE | Phase 8 | NEW | Reveal |
| 701.20b–c | PURE-DEF | — | — | Reveal zone/re-reveal |
| 701.20d | DEFERRED | Phase 8 | — | Shuffle stops reveal |
| 701.20e | PURE-DEF | — | — | "Look at" |
| 701.21a | TESTABLE | Phase 5-Pre | T15 | Sacrifice (3 tests) |
| 701.22a | TESTABLE | Phase 8 | NEW | Scry |
| 701.22b | TESTABLE | Phase 8 | NEW | Scry 0 no-op |
| 701.22c | DEFERRED | Phase 8 | — | Simultaneous scry (APNAP, valid in 2-player) |
| 701.22d | PURE-DEF | — | — | Scry trigger timing |
| 701.23a | TESTABLE | Phase 8 | NEW | Search |
| 701.23b | TESTABLE | Phase 8 | NEW | Fail-to-find |
| 701.23c | PURE-DEF | — | — | Undefined quality |
| 701.23d | TESTABLE | Phase 8 | NEW | Mandatory quantity search |
| 701.23e | PURE-DEF | — | — | Found cards not revealed |
| 701.23f–h | DEFERRED | Phase 8 | — | Search variants |
| 701.23i | DEFERRED | Phase 8 | — | Simultaneous search (APNAP, valid in 2-player) |
| 701.23j | DEFERRED | Phase 8 | — | Outside-game search |
| 701.24a | ALREADY-IMPL | — | — | Shuffle |
| 701.24b | TESTABLE | Phase 8 | NEW | Search-shuffle exclusion |
| 701.24c–e | PURE-DEF | — | — | Shuffle edge cases |
| 701.24f | DEFERRED | Phase 7 | — | Simultaneous shuffle triggers |
| 701.24g | DEFERRED | Phase 8 | — | Shuffle + position |
| 701.25a | TESTABLE | Phase 8 | NEW | Surveil |
| 701.25b | DEFERRED | Phase 8 | — | Additional-card surveil |
| 701.25c | TESTABLE | Phase 8 | NEW | Surveil 0 no-op |
| 701.25d | PURE-DEF | — | — | Surveil trigger timing |
| 701.26a | ALREADY-IMPL | — | — | Tap |
| 701.26b | ALREADY-IMPL | — | — | Untap |
| 701.27a–g | DEFERRED | Phase 9 | — | Transform |
| 701.28a–f | DEFERRED | Phase 9 | — | Convert |
| 701.29a | TESTABLE | Phase 8 | NEW | Fateseal |
| 701.30a–d | DEFERRED | Phase 8 | — | Clash |
| 701.31a–d | OUT-OF-SCOPE | — | — | Planeswalk (Planechase) |
| 701.32a–c | OUT-OF-SCOPE | — | — | Set in Motion (Archenemy) |
| 701.33a–b | OUT-OF-SCOPE | — | — | Abandon (Archenemy) |
| 701.34a | TESTABLE | Phase 8 | NEW | Proliferate |
| 701.34b | OUT-OF-SCOPE | — | — | THG poison (team format) |
| 701.35a | TESTABLE | Phase 8 | NEW | Detain |
| 701.36a | TESTABLE | Phase 8 | NEW | Populate |
| 701.36b | TESTABLE | Phase 8 | NEW | Populate empty no-op |
| 701.37a | TESTABLE | Phase 8 | NEW | Monstrosity + guard |
| 701.37b | PURE-DEF | — | — | Monstrous designation |
| 701.37c | DEFERRED | Phase 8 | — | Monstrosity X variable |
| 701.38a–d | DEFERRED | Phase 9 | — | Vote |
| 701.39a | TESTABLE | Phase 8 | NEW | Bolster + tie-break |
| 701.40a–h | **ATOM** | Phase 5-Pre | NEW | Manifest — face-down infrastructure |
| 701.41a | TESTABLE | Phase 8 | NEW | Support (2 tests) |
| 701.42a–c | DEFERRED | Phase 9 | — | Meld |
| 701.43a–d | **ATOM** | Phase 5-Pre | NEW | Exert — skip-untap tracking |
| 701.44a–d | **ATOM** | Phase 8 | NEW | Explore — multi-step + LKI |
| 701.45a | OUT-OF-SCOPE | — | — | Assemble (Unstable) |
| 701.46a | TESTABLE | Phase 8 | NEW | Adapt + guard |
| 701.47a–d | **ATOM** | Phase 8 | NEW | Amass — conditional token + subtype |
| 701.48a | DEFERRED | Phase 8 | — | Learn |
| 701.49a–d | DEFERRED | Phase 9 | — | Venture — unique infra |
| 701.50a–e | **ATOM** | Phase 8 | NEW | Connive — draw-discard-counter |
| 701.51a–c | OUT-OF-SCOPE | — | — | Open Attraction (Unfinity) |
| 701.52a | OUT-OF-SCOPE | — | — | Roll to Visit (Unfinity) |
| 701.53a–b | **ATOM** | Phase 9 (D3) | D3 | Incubate — simplest DFC |
| 701.54a–e | **ATOM** | Phase 8 | NEW | Ring Tempts You |
| 701.55a–d | **ATOM** | Phase 8 | NEW | Villainous Choice |
| 701.56a | DEFERRED | Phase 8 | — | Time Travel |
| 701.57a–c | **ATOM** | Phase 8 | NEW | Discover — free-cast pipeline |
| 701.58a–h | **ATOM** | Phase 5-Pre | NEW | Cloak — manifest + ward {2} |
| 701.59a–c | **ATOM** | Phase 8 | NEW | Collect Evidence |
| 701.60a | TESTABLE | Phase 8 | NEW | Suspect designation |
| 701.60b | PURE-DEF | — | — | Suspected designation rules |
| 701.60c | TESTABLE | Phase 8 | NEW | Suspected grants menace + can't block |
| 701.60d | PURE-DEF | — | — | Re-suspect no-op |
| 701.61a | DEFERRED | Phase 8 | — | Forage |
| 701.62a–b | **ATOM** | Phase 5-Pre | NEW | Manifest Dread |
| 701.63a | TESTABLE | Phase 8 | NEW | Endure (counters + token) |
| 701.63b | TESTABLE | Phase 8 | NEW | Endure 0 no-op |
| 701.64a | TESTABLE | Phase 8 | NEW | Harness + guard |
| 701.64b | PURE-DEF | — | — | Harnessed designation |
| 701.65a–b | **ATOM** | Phase 8 | NEW | Airbend — exile + cast for {2} |
| 701.66a–b | **ATOM** | Phase 5-Layers/7 | NEW | Earthbend — land animation + delayed trigger |
| 701.67a–c | **ATOM** | Phase 8 | NEW | Waterbend — tap-for-generic |
| 701.68a | TESTABLE | Phase 8 | NEW | Blight |
| 701.68b | TESTABLE | Phase 8 | NEW | Blight legality |
| 701.68c | PURE-DEF | — | — | "Blighted creature" term |
| 701.68d | PURE-DEF | — | — | Blight trigger timing |

## Classification Totals

| Category | Count |
|----------|-------|
| ATOM / TESTABLE tests | ~123 individual test specs |
| BOUNDARY-DEF | 3 (Historic, Modified, Outlaw) |
| PURE-DEF | ~34 sub-rules |
| ALREADY-IMPLEMENTED | 13 sub-rules |
| DEFERRED | ~50 sub-rules |
| OUT-OF-SCOPE | ~15 sub-rules |

## New Tickets

| Gap | Rule(s) | Phase | Priority |
|-----|---------|-------|----------|
| Token creation primitive | 701.7a | Phase 8 | High |
| Fight keyword action | 701.14a–d | Phase 8 | Medium |
| Scry primitive | 701.22a | Phase 8 | High |
| Search primitive | 701.23a–d | Phase 8 | High |
| Surveil primitive | 701.25a | Phase 8 | Medium |
| Mill primitive | 701.17a–b | Phase 8 | Medium |
| Reveal primitive | 701.20a | Phase 8 | Medium |
| Proliferate | 701.34a | Phase 8 | Medium |
| Destroy delta tagging (destroy vs sacrifice) | 701.8b | Phase 6 | High |
| Regeneration shield system | 701.19a–c, 701.8c | Phase 6 | Medium |
| Double/Triple P/T as L7c effect | 701.10a–c, 701.11a–c | Phase 5-Layers | Medium |
| Damage doubling replacement | 701.10g | Phase 6 | Medium |
| Exchange effects | 701.12a–c,g | Phase 8 | Low |
| Random/opponent discard variants | 701.9b | Phase 8 | Low |
| Attach/Unattach primitives | 701.3a–d | Phase 5-Pre | High |

## Gap Report

### Dependency Chain

```
Phase 5-Pre:  Attach/Unattach (T15), Sacrifice expansion (T15), Modal spells (T18),
              Face-down infrastructure (Manifest/Cloak/Manifest Dread),
              Skip-untap tracking (Exert), Special action framework (TurnFaceUp)
     ↓
Phase 5-Layers:  Double/Triple P/T (L7c), Devotion partial-layer,
                 Earthbend land animation (L4 type + L7b P/T)
     ↓
Phase 6:  Destroy delta tagging (D20), Regeneration shields, Damage doubling replacement,
          Manifest ETB prohibition (701.40f)
     ↓
Phase 7:  Modal triggered abilities (700.2b), Shuffle triggers (701.24f),
          Earthbend delayed triggers (701.66), Exert linked triggers (701.43d)
     ↓
Phase 8:  Token creation, Fight, Scry, Search, Surveil, Mill, Reveal, Proliferate,
          Investigate, Bolster, Monstrosity, Adapt, Detain, Populate, Support,
          Suspect, Endure, Harness, Blight, Exchange, Random discard, Devotion,
          Historic/Modified/Outlaw predicates,
          Explore, Amass, Connive, Ring Tempts You, Villainous Choice,
          Discover (+ CastPermission::FreeFromExile),
          Airbend (+ CastPermission::AlternateCostFromExile), Collect Evidence,
          Waterbend (+ tap-for-generic cost pipeline),
          Emblem system, Designation registry
     ↓
Phase 9:  Goad, Vote, Transform/Convert, Meld, Incubate (DFC anchor, D3),
          Venture (arch notes recorded), multiplayer APNAP variants
```

## ALREADY-IMPLEMENTED List

700.4, 701.2a, 701.5a, 701.6a, 701.6b, 701.8a, 701.9a, 701.13a, 701.18a, 701.24a, 701.26a, 701.26b

## OUT-OF-SCOPE List

| Rule | Reason |
|------|--------|
| 701.31a–d | Planeswalk — Planechase format |
| 701.32a–c | Set in Motion — Archenemy (team format) |
| 701.33a–b | Abandon — Archenemy (team format) |
| 701.34b | THG poison — Two-Headed Giant (team format) |
| 701.45a | Assemble — Unstable / silver-bordered |
| 701.51a–c | Open Attraction — Unfinity |
| 701.52a | Roll to Visit — Unfinity |

## DEFERRED List

| Rule | Phase | Reason |
|------|-------|--------|
| 700.7 | Phase 7 | "This [something]" object identity — epoch tracking |
| 700.8–700.8d | Phase 8 | Party mechanic — optimal assignment across 4 types |
| 700.10 | Phase 8 | "Activated this turn" — delta log per-turn tracking |
| 700.11 | Phase 8 | Descended — per-turn permanent card GY tracking |
| 700.13 | Phase 8 | Committing a crime — per-event target tracking |
| 700.14 | Phase 8 | Expend — per-turn mana-spent accumulator |
| 701.4–701.4b | Phase 8 | Behold — niche Aetherdrift mechanic |
| 701.12e | Phase 8 | Attachment transfer during zone exchange |
| 701.12f | Phase 8 | Empty-zone exchange |
| 701.12h | Phase 8 | Text-box exchange (Exchange of Words) |
| 701.15a–d | Phase 9 | Goad — multiplayer attack requirements |
| 701.17c | Phase 8 | Milled card tracking in public zones |
| 701.17d | Phase 8 | Multi-milled-card references |
| 701.18b | Phase 8 | Play permission — continuous effect model |
| 701.20d | Phase 8 | Shuffle stops reveal |
| 701.22c | Phase 8 | Simultaneous scry (APNAP, valid in 2-player) |
| 701.23f–h | Phase 8 | Search variants (portion replacement, etc.) |
| 701.23i | Phase 8 | Simultaneous search (APNAP, valid in 2-player) |
| 701.23j | Phase 8 | Outside-game search (wish effects) |
| 701.24f | Phase 7 | Simultaneous shuffle triggers |
| 701.24g | Phase 8 | Shuffle + positional placement |
| 701.25b | Phase 8 | Additional-card surveil variant |
| 701.27a–g | Phase 9 | Transform — DFC infrastructure |
| 701.28a–f | Phase 9 | Convert — follows transform rules |
| 701.30a–d | Phase 8 | Clash — niche Lorwyn mechanic |
| 701.37c | Phase 8 | Monstrosity X variable |
| 701.38a–d | Phase 9 | Vote — multiplayer mechanic |
| 701.40c–d | Phase 8 | Manifest + morph/disguise turn-face-up |
| 701.42a–c | Phase 9 | Meld — DFC infrastructure |
| 701.44d | Phase 8 | Simultaneous explore (APNAP, valid in 2-player) |
| 701.48a | Phase 8 | Learn — requires outside-game card pool |
| 701.49a–d | Phase 9 | Venture — unique dungeon infrastructure |
| 701.50d | Phase 8 | Simultaneous connive (APNAP, valid in 2-player) |
| 701.55c | Phase 6 | Villainous choice replacement effect multiplying |
| 701.55d | Phase 9 | Villainous choice APNAP for multiplayer |
| 701.56a | Phase 8 | Time Travel — requires suspend infrastructure |
| 701.61a | Phase 8 | Forage — requires Food token infrastructure |

--- End of Session 7A Summary ---
