# codebase-state.md — closed Deferred Migrations items

**A record of finished work, not a plan.** Every entry here was a live
Deferred Migrations item in `plans/codebase-state.md` and was evicted once
its dated `**Reachability:**` verdict said `closed`. Nothing here is owed,
and nothing here should be acted on — that is what the rest of
`plans/archive/` means too. What each entry is *for* is the reasoning:
why the debt existed, what it would have cost, and what closed it.

**Ids are section-scoped, exactly as they are in the live file** (see its
Deferred Migrations header). An entry is cited as "'Before Layers' item 8",
never bare. The live file keeps a three-line stub at every number here — the
heading, the verdict's closing sentence, and a pointer back — so every existing
"item N" citation still resolves at its own number and nothing was renumbered.

Written by the archive split of 2026-09-09 (`codebase-state.md`, the
2026-09-03 triage's step 3). Later closures are appended by the PR that
closes them.

## Before Replacement effects (CR 614–616)

1. **Zone-change migration — ✅ done (2026-04-18).** `move_object` is now `pub(crate)` with documentation directing external callers to `change_zone` / `execute_action(GameAction::ZoneChange)`. All 12 previously-direct callers (5 SBA sites in `engine/sba.rs`, `Cost::SacrificeSelf` in `engine/costs.rs`, push-to-stack + 4 rollbacks in `engine/cast.rs`, cleanup discard in `state/game.rs`) now route through the chokepoint. `engine/actions.rs::change_zone(id, to)` is the new convenience wrapper. Internal helpers (`draw_card`, `play_land`, and the `GameAction::ZoneChange` arm itself) continue to call `move_object` directly from inside `engine/zones.rs`.

   **Reachability (2026-09-03):** closed — struck 2026-04-18; predates the PR
   record.

2. **Open-coded zone bookkeeping — ✅ CLOSED 2026-08-25 (RA-3 ticket 7).**
   - `engine/resolve.rs` `Primitive::CounterSpell` calls `change_zone(id, Graveyard)`, which tears down the `StackEntry` via `remove_from_zone_collection(Stack)`, then emits `SpellCountered` (2026-04-18).
   - The three `engine/stack.rs` sites (permanent-spell ETB, instant/sorcery → graveyard, `handle_fizzle`) are closed. They bypassed because the stack-pop-first pattern removes the object from the stack `Vec` before resolution begins, so `move_object` would have double-removed. **The fix names the in-between state rather than routing around it:** `GameState::resolving` records the popped object and the CR 110.2b controller the destroyed `StackEntry` was carrying, and two readers consult it — `remove_from_zone_collection(Stack)` (a missing entry is expected for exactly that object, and a bug for anything else) and `init_zone_state` (an entering permanent takes the resolving spell's controller). Cleared on every path out of `resolve_top_of_stack`, including the error ones, which is what `resolve_popped` exists to make single-sited.
   - All three now carry a cause, and `Resolved` vs `Fizzled` finally separates CR 608.2n/608.3 from CR 608.2b's counter-by-game-rules — previously indistinguishable Stack→Graveyard moves. **`// REPLACEMENT-BYPASS:` no longer names anything;** `move_object`'s doc comment is down to one exception, `// CAST-ROLLBACK:`, which is permanent.
   - Note for later: `resolving` also makes the pop-first pattern replaceable. CR 608.2 keeps a resolving spell *on* the stack; the engine pops it early so in-flight effects cannot see it. Turning that into a mark-resolving flag is now a change to `CounterSpell` and targeting alone, not to the zone code.

   **Reachability (2026-09-03):** closed — RA-3, PR #60 (825c602).

3. **Event-stream refit — ✅ CLOSED 2026-08-25 (Phase RA, three PRs).** Specified 2026-08-24 as the CR 614 phase's opening ticket block, from a census of all 42 production `events.emit` sites plus the known bypasses. Every item shipped:

   - **Draw-step draw through the chokepoint**, with `CardDrawn` — CR 121.5 makes a library→hand move without the word "draw" a different, trigger-visible fact (RA-2). Opening-hand draws stay direct: pregame, nothing can observe them.
   - **Tap/untap through the chokepoint**, with transition-only `Tapped`/`Untapped` per CR 603.2e (RA-2). The untap sweep is ordered, and since RA-3 it is one batch — CR 502.1 untaps simultaneously.
   - **Life mutations inside the chokepoint** — lifelink and `Cost::PayLife` propose `GainLife`/`LoseLife` instead of writing `life_total` (RA-2). Both *emitted* `LifeChanged` while writing by hand, which is exactly how the 42-site census missed them: the pipeline reads the proposal, not the event. One member of the class remains undecomposed and is *inside* the chokepoint — `perform_action(DealDamage)` performs a player's life loss inline, so a `LoseLife` watcher would miss combat damage. That is CR 120.3's results-of-damage decomposition, scheduled with Phase RD (which also owes CR 120.3c — see the CR 1 map's 120 row).
   - **`AbilityActivated` plus an identity-bearing `AbilityResolved`** (RA-2), for CR 603.7h counting.
   - **Payload upgrades** (RA-3): the CR 603.10a LKI frame on battlefield-leaving zone changes, the `cause`, a `BatchId`, and the resolution that proposed the event. The last two ride an `EventRecord` envelope rather than per-variant fields — they are the same two facts for every kind of event, and a variant that forgets to carry them fails silently.
   - **The three `// REPLACEMENT-BYPASS:` sites closed** (RA-3; item 2 above).
   - **Type-specific death events demoted** to display sugar (RA-3), with `PermanentLeftBattlefield` deleted and `SpellResolved` renamed `StackObjectResolved`.

   **Exit criterion, met:** every state mutation observable by CR 614 or CR 603 is emitted from exactly one place, and an event-log replay can distinguish drawn from tutored, destroyed from sacrificed, and countered from resolved.

   **Reachability (2026-09-03):** closed — Phase RA, PRs #58, #59, #60.

4. **~~Unresolved architecture fork~~ — ✅ RESOLVED 2026-08-24 (owner decision): trigger detection is the performed-action event stream; the delta log is rejected.** No `engine/delta_log.rs` will exist. The shape: `GameAction` (proposed) → CR 614 replacement pipeline → perform → `GameEvent` (performed record carrying LKI frame, cause, batch id, resolution context) → **synchronous dispatch at the mutation instant** (turn trackers update; event triggers match against *effective* ability lists across all zones; state triggers and designations evaluate against live state) → pending-trigger queue → APNAP placement at the CR 603.3 moment (the `engine/priority.rs:240` stub). Detection happens per atomic event; only *placement* defers — CR 702.131d, the city's-blessing-before-SBA ruling, and CR 603.8's momentary-condition example all require exactly that split.

   `state-tracking-architecture.md` remains the statement of the four problems; its **Resolution postscript** records how each is answered and why the delta representation fails them (semantic identity is not recoverable from `(old,new)` state pairs — CR 121.5, 603.10e; evaluating past instants requires replaying the layer engine — CR 603.10, 603.6b, 702.131d). What the delta doc got right is adopted: central detection with no per-site knowledge of conditions, single-funnel emission discipline (item 3 above is that ticket list), LKI-over-type-specific-events, and resolution-context stamping. Loop detection Tiers 1–3 and D26 survive, with D26 transcripts re-based on performed-action sequences. Struck in writing: `roadmap.md` delta block + 2026-04-06 blockquote, `design_doc.md` §8 subsections + decision rows 2026-04-01 / 2026-04-11×2, and the CLAUDE.md authority-table carve-out.

   **N-player from day one:** trackers are per-`PlayerId` vectors plus global; the pending queue takes the player set and orders per CR 603.3b APNAP; CR 616.1 ordering hangs off the affected object's controller / affected player, with APNAP among simultaneous choosers.

   **Reachability (2026-09-03):** closed — owner decision 2026-08-24, PR #54
   (ccd6ac1).

5. **CR 601.2a announces a move that CR 601.2 may un-happen (recorded 2026-08-25, RA-3).** `cast_spell` proposes the hand→stack move at CR 601.2a — correctly, since the object really is on the stack while costs are paid — but the `ZoneChange` is emitted *then*, before it is knowable whether the cast rewinds. A rewind leaves the forward event in the log, so a replay sees a move CR 601.2 says never happened. RA-3 fixed the other half (the rollback itself is silent, which is what `// CAST-ROLLBACK:` had always claimed and never delivered); this half needs the announcement deferred to CR 601.2i without deferring the move, which is a two-phase cast rather than a payload change. **Sized:** one function, `cast_spell`, plus wherever the deferred event is flushed. `tests/phase_ra_integration_test.rs::test_a_failed_cast_announces_nothing` documents the gap where it lives. Not blocking RB — no replacement effect applies to a rewind — but it is a wrong entry in a log the trigger matcher will read, so it should land before Phase 6.

   **Reachability (2026-09-03):** closed — by RC-4b (PR #87, 6541d0b), and
   nobody struck it: `cast.rs` moves the card with the silent `CAST-ROLLBACK`
   mover in both directions and `announce_zone_change` records the 601.2a move
   at 601.2i beside `SpellCast` (`cast.rs:269`), so a rewound cast leaves no
   forward event. Item 51 recorded the closure under its own number; this is the
   same fix.

7. **The early stack pop — ✅ DELETED 2026-09-01 (phase RC-1).** `resolve_top_of_stack` used to remove the object from the `stack` `Vec` before resolving, documented as keeping an in-flight Counterspell from seeing the resolving object. Nothing could see it: CR 608.2g forbids casting a spell or activating an ability during a resolution, so no effect can *acquire* it as a target mid-resolution, and a spell cannot choose itself at CR 601.2c because `enumerate_legal_selections` (`oracle/legality.rs`) and `has_any_legal_choice` (`engine/targeting.rs`) already exclude it by `exclude_id`. The CR meanwhile keeps a resolving spell **on** the stack (CR 608.2; 608.2n/608.3a move it at the end), so the pop was an engine artifact the rules do not have.

    **What shipped.** The pop is gone; the `StackEntry` is still taken (the body owns it). `move_object`'s `remove_from_zone_collection(Stack)` now does the removal it was always asked to do and finds the object where CR 608.2 says it is, which deleted the leniency branch that had licensed a stack removal finding nothing — a branch that could mask a genuinely missing stack object. The two ability paths (`resolve_taken`'s completed arm and `handle_fizzle`'s) remove from `stack` explicitly, because an ability ceasing to exist is not a zone change. `resolve_popped` was renamed `resolve_taken`: there is no pop for it to be named after. **`GameState::resolving` now has exactly one reader** — `init_zone_state` at `engine/zones.rs`, CR 110.2b's default controller — and it is a rules question rather than an engine artifact, so the field's doc no longer describes a window.

    **The audit, re-run 2026-09-01 and corrected.** `replacement-architecture.md` §9 claimed five `stack.is_empty()` readers; there are **six**. `zones.rs:169` had drifted to `:177`, and the unlisted one is **`ui/display.rs:287`, `format_stack`** — which has **no production caller at all** (it is `pub`, and every use is a test in its own file), so the deletion changed no rendering. The CR is on the deletion's side even where it does render: CR 608.2 puts the resolving object on the stack, so showing it is right, and the CLI would only ever reach it through a mid-resolution `DecisionProvider` prompt. `engine/stack.rs:27`'s guard is a seventh occurrence and correctly excluded from both counts — it runs before the resolution, not during. The `GameState::resolving` count of six was exact. **None of the six is reachable during a resolution today**, which is what made the deletion safe; CR 608.2g's "unless an effect instructs" case makes `engine/cast.rs`'s reachable once RC-era cards arrive, and that site now carries a comment saying so and naming the choice it will have to make. Not fixed here.

    **Exit met, mechanically.** Whole suite green, zero warnings, and `fuzz_games --games 200 --seed 12345` byte-identical to a same-day `main` binary on **both** pools outside the `=== Timing ===` block; three runs each, identical. A `--dump-events` diff at 40 games was added on top of the summary and is identical too, after canonicalizing the per-process v4 `ObjectId`s — 18,097 event lines on `performance`, 18,738 on `stress`, same kinds and same counts. No new `GameEvent`, no new behavior.

    **Reachability (2026-09-03):** closed — RC-1, PR #80 (093e12a).


## Found by CV-1's reachability mode (2026-09-02)

16c. ~~**Spells resolve without being paid for, and `cast_spell` is one missing
    `rollback_cast_to_hand` away from correct**~~ ✅ **closed 2026-09-02,
    `fix/cast-rollback-on-payment-failure`** (found 2026-09-02 by CV-1's
    `--require`; diagnosed the same session; fixed the next).

    **The defect, as diagnosed.** `cast.rs` had five fallible steps between CR
    601.2a's silent move to the stack and CR 601.2i's announcement. Four called
    `rollback_cast_to_hand` before returning `Err`; the fifth — `pay_costs`,
    the last statement before 601.2i — was a bare `?`. When it failed the card
    stayed on the stack, was never announced as cast, and resolved on the next
    pass with the mana still in the pool. `activate_ability` had handled the
    identical failure correctly the whole time, and `priority.rs`'s "leave game
    state clean" contract named the one that did not.

    **The mechanism, confirmed by the failing test rather than inferred.**
    `can_pay_costs` passes; then `ask_choose_generic_mana_allocation` lets the
    player split the generic part across every type in the pool, capped only by
    each type's amount, so a split that spends a color a pip still needs
    passes the prompt's own validation and `ManaPool::pay` refuses it. Grizzly
    Bears `{1}{G}` against `{R}{G}` with the generic put on Green is the whole
    reproducer, and on the pre-fix tree it fails at "back in hand: left
    `Stack`". The prompt only runs when `generic_count() > 0`, which is why
    every ghost had a generic component and no zero-generic card ever was one.
    Census of the **199** in 40 `performance` games on `main` at 650a263:
    Merfolk Thaumaturgist 17, Blood Moon 15, Volcanic Upheaval 13, Serra Angel
    10, March of the Machines 10, … and never Lightning Bolt, Counterspell,
    Dark Ritual or Giant Growth. (The 206 the first draft cited was at 103acf1,
    before CV-1 grew the pool.)

    **What landed.** (1) The rollback, in the shape of its four siblings.
    (2) `test_a_cast_whose_payment_fails_rewinds_and_keeps_the_mana`
    (`phase_ra_integration_test.rs`), shown failing with `mtgsim/src` stashed:
    back in hand, both mana still in the pool, no `SpellCast`, stack and
    `stack_entries` empty, event log unchanged. (3) **The durable half:
    `fuzz_games` checks, in every game and every mode, that every object
    leaving the stack with `ZoneChangeCause::Resolved` has a prior `SpellCast`
    for that object** — counted per object, so a recast owes a second one; an
    ability ceases to exist and emits no zone change, so it never trips it. A
    violation prints beside errors and panics with the card and the event
    index, sums as `Uncast resolved:` in the results block, and fails the run.
    By the same rule `main`'s dumps read **199 / 188** (`performance` /
    `stress`, 40 games) and the fixed tree reads **0** in every run — three
    rounds of 200 games per pool, plus the 40- and 50-game runs. It would have
    caught this on the first fuzz run after the cast pipeline landed.

    **What the free spells were worth — 200 games / seed 12345 / `--threads
    1`, medians of three interleaved rounds, both binaries in one sitting.**
    `main` resolved **5.5** spells per `performance` game and **5.0** per
    `stress` game that it never announced — 1,104 and 991 in 200 games — on
    top of the 21.7 / 20.4 it did. The fixed tree announces **27.9 / 24.8**, so
    *about the same number of spells reach the battlefield*; they are now paid
    for, and the mana they cost is not spent on something else. That is what
    moved everything else: turns 30.8 → 33.3 and 30.0 → 30.9, and the cost
    rows with them — walks 99,952 → 116,233 and 95,855 → 100,619, of which
    walks *per turn* are +7.5% / +2%. Per-walk time is flat-to-down, 1.125 →
    1.081 and 1.056 → 0.988 ms per 1,000 walks (−4% / −6%, interleaved), and
    `Frames/walk` fell on both pools (1.37 → 1.33, 1.33 → 1.29): the ghosts
    were disproportionately three- and four-mana statics — Humility, Blood
    Moon, March of the Machines and Glorious Anthem are 37 of the 199 — which
    are exactly the permanents that put a sub-frame under every walk. Creatures
    died is flat (7.9 → 7.8, 4.9 → 4.6). `engineering-practices.md` §3 carries
    the 50-game table, re-recorded: the first re-record where the pool did not
    move, so every row in it is the engine's.

    **Attributed, not assumed:** 40 `--dump-events` games per pool, ids
    masked, first divergence per game. `performance`: all 40 diverge, and
    every first divergence lies inside its game's first ghost's window — after
    the card was drawn, at or before the event where `main` resolved it unpaid
    — none outside, none in a game without a ghost. `stress`: 39 of 40 the
    same way, and the one game with no ghost on `main` is byte-identical after
    masking. In 30 of the 40 `performance` games the logs are identical *up to*
    the unpaid resolution itself: the failed cast is silent on both trees, the
    random agent's next decisions do not depend on where the card went, and the
    trees part at the pass-pass that resolves a spell on one and nothing on the
    other.

    **Should an illegal split be refused at the prompt instead? Yes; it is
    small; and it is its own PR, deliberately.** Every other `ask_*` offers a
    `DecisionProvider` only legal choices — targets are enumerated legal,
    priority actions are, CR 616.1 asks only among applicable effects — and a
    DP should not need payment law to answer "which mana". The clamp is exact
    and about ten lines: each bucket's max becomes `available − pips of that
    type`, and whenever `can_pay` passed the clamped maxima still sum to at
    least the generic count, so a feasible answer always exists. Two reasons it
    is not here, both consequences of the fix landing first: it moves game
    content a second time — every one of those ~5 rewinds per game becomes a
    cast — and the measurement above is readable only with one change per
    binary (RC-4's A/B/C table is the shape for the follow-up); and this PR's
    regression test drives the rollback *through* the bad split, which the
    clamp turns into a `validate_allocation` panic before payment. The
    follow-up replaces that test with a prompt-shape one and leaves the
    rollback as the CR 601.2 backstop the fuzz guard watches. `backlog.md`
    §2.18 owns it beside the CR 732.1 reversal; note that after this PR the
    random agent's rewind rate is §2.18's ~7.5 per game *plus* these ~5.

    **The clamp landed 2026-09-03 (`mana/generic-split-clamp`), and it moved no
    game content at all — which is the finding.** `ask_choose_generic_mana_allocation`
    now caps each bucket at `available − pips of that type`, with a `debug_assert`
    that the clamped maxima still reach the generic count (they do whenever
    `can_pay` passed, which is the same inequality). What the paragraph above
    did not know is that **16d had already fixed this for the one DP that
    plays**: `RandomDecisionProvider` clamped for itself, as policy, when it
    learned to tap for the pip it owes. So the ~5 rewinds per game were gone
    before this PR started, and the honest expectation was not "every rewind
    becomes a cast" but "nothing changes and the DP stops needing payment law".
    Leaving the DP's copy in place would have *created* a delta — it subtracts
    the pips a second time from maxima that already exclude them, narrowing the
    split to the surplus beyond twice the pips — so it came out with the clamp,
    and the random agent now takes the prompt's maxima at face value. Measured
    both ways: `fuzz_ab.py` reads **IDENTICAL** on every counter, both pools,
    200 games, and the 40-game `--dump-events` streams are byte-identical after
    masking ids. Land taps per spell cast is unchanged at 3.11 (40-game
    `performance` dumps, `Tapped:` lines naming a land over `SpellCast:` lines);
    16d's 3.18 is the same quantity counted a slightly different way and is not
    a movement. `engineering-practices.md` §3's fixture rows reproduce to the
    digit, so the table is not re-recorded — it is confirmed.

    **What the payment-failure arm is now.** Nothing reachable through
    `cast_spell` can fail `pay_costs` after `can_pay_costs` passed: the prompt
    cannot offer an unpayable split, and a DP that ignores the maxima trips
    `validate_allocation` first. The rollback stays as the backstop — item 9's
    kicker double-count is the other route to it and is not reachable yet — and
    `fuzz_games`' "no spell resolves without a `SpellCast`" guard is what
    watches it. The regression test that drove the rollback *through* the bad
    split is replaced by a pair on the same board — Grizzly Bears `{1}{G}`
    against `{R}{G}`, 16c's own reproducer: the legal split pays and the cast
    completes, and the split that used to cost the cast is a `should_panic`
    naming the clamped number (`DP allocated 1 to bucket 1 but maximum is 0`).
    **`validate_allocation` printing the max is what makes the maxima testable
    without instrumentation** — the first draft of this PR carried two ~50-line
    recording `DecisionProvider`s to read the bounds, and the panic message says
    the same thing in one line. Its `ATOM-601.2-001` partial claim moved to `phase_rc4b`'s
    `test_a_rewound_cast_keeps_its_mana_abilities_and_leaves_no_zone_change`,
    which proves the same half (card in hand, stack unchanged, mana still in
    the pool); `ATOM-601.2h-002` was already claimed there and in
    `test_a_failed_cast_announces_nothing`. `specdb owed` is unchanged at 9.

    **A second route to the same hole, unreachable today:** "Before card
    breadth" item 9 — `can_pay_costs` checks each `Cost::Mana` entry against
    the whole pool, so a kicker's mana is double-counted and `pay_costs` fails
    with the base already spent. The rollback returns the card; it does not
    return the mana.

    **Reachability (2026-09-03):** closed — PR #90 (6dedaf8), and the clamp in
    PR #94 (0ed6836).


## Found by the Everywhere pool change (2026-09-03)

16e. **96% of layer walks repeat an object nothing has touched, so item 7's
    memoization half is split out as 7a and moved ahead of triggers
    (2026-09-03, owner's call, on the Everywhere PR).** A temporary probe keyed
    every walk on `(ObjectId, execute_actions batch)`: 108,626 walks and 4,030
    distinct keys per `performance` game, 111,418 and 4,183 on `stress`, at
    313 batches a game. The 2026-08-23 reasons for deferring the memo ("Before
    Layers", the Layer 2 phase's "cross-call memoization deliberately did NOT
    land here") are all about a key that enumerates its inputs; an epoch key
    has none, and it is what the 613.8 board-wide pass will store into. Design,
    the bump-site census, the bypasses and the acceptance test are in
    `layers-architecture.md` §12 "7a"; `CLAUDE.md`'s critical path carries the
    order. This PR sharpened the motive as well as measuring it: Everywhere
    made each land walk heavier (+36% per walk, `engineering-practices.md`
    §3), and a memo pays that once per epoch rather than once per query.

    **Closed 2026-09-03 — 7a ✅ (`layers/epoch-memo`).** Walks per game
    108,626 → 2,663 on `performance` and 111,418 → 2,719 on `stress` at 50
    games, with every other §3 row reproduced to the digit and 40-game
    event streams byte-identical to `main`'s; CPU per game 146.8 → 13.8 ms on `performance` (200 games, medians of
    three interleaved rounds, −90.6%) and 1.47 → 0.139 ms per 1,000
    questions asked. The epoch is
    bumped at the write by one funnel per input (three inputs got a funnel
    for it: `remove_object`, `remove_counters`, `set_stack_entry` /
    `take_stack_entry`), the registry's half of it moves only when a row
    arrives or leaves, and every hit is audited against a fresh walk in
    debug — which caught three tests writing a walk input directly and
    nothing in the engine. `Layer walks` is the miss count now and `Memo
    hits` sits beside it. The as-built details, the numbers and the
    residual are in `layers-architecture.md` §12 "7a"; item 7 decides a
    finer key against that residual.

    **Reachability (2026-09-03):** closed — critical-path item 7a, PR #92
    (cbd7e59).


## Found by the RC-4 review's nesting audit (2026-09-02)

51. **~~A rewound cast leaves its CR 601.2a move in the log.~~ — ✅ CLOSED 2026-09-02 (RC-4b).** `cast_spell`
    moves the card to the stack through `change_zone` with cause `Cast`, which
    emits, and the four rewind sites — 601.2b's cost choices, 601.2c's
    targets, 601.2h's can-pay check — move it back with the silent
    `CAST-ROLLBACK` mover. So a cast that fails leaves
    `ZoneChange { Hand → Stack, Cast }` in the log with no counterpart.
    `SpellCast` is emitted only at 601.2i, so a cast trigger keyed on it is
    safe; a matcher on the zone change is not. Same class as the entry hop
    (item 4 under "Before Triggered abilities"), different fix: nothing in the
    CR replaces a card being put onto the stack, so the 601.2a move is not a
    replaceable event and should use the silent mover in both directions, with
    the `ZoneChange` recorded at 601.2i beside `SpellCast` — the moment CR
    601.2i says the spell becomes cast. Mana abilities activated in 601.2g stay
    performed and stay in the log, which is CR 732.1. No rewind site exists
    after cost payment begins, so nothing paid is ever un-paid. **Bundled
    into RC-4b** (`replacement-architecture.md` §9, design item 7): ~30
    lines in `cast.rs`, and RS-2's Tier 1b/1e exits are more rewind sites,
    so it gets more reachable with time, not less. The rule both fixes
    follow is `replacement-architecture.md` §11 item 20. **Closed as
    designed:** the 601.2a move is `move_object` in both directions, tagged
    `CAST-ROLLBACK`, and `announce_zone_change` records it at 601.2i beside
    `SpellCast`; a rewound cast leaves no `ZoneChange` and keeps its 601.2g
    taps (`phase_rc4b_integration_test`).

    **Reachability (2026-09-03):** closed — RC-4b, PR #87 (6541d0b).


## Found by RC-5 — applying an entry can move the board (2026-09-03)

57. **`AmountExpr::SourcePower` has exactly one evaluator, and the other two
    refuse it.** `replacement::evaluate_enter_template` reads
    `EntryFrame::frame_of(source)` — `Some` only when the source *is* the
    entering object — and falls back to the real board, which is the whole of
    §5b's asymmetry in one line: an entering permanent's own "with a counter for
    each …" reads its hypothetical self, and Master Biomancer is read off the
    board. `compute::evaluate_amount` and `resolve::evaluate_amount` grow an arm
    that errors rather than guessing at a source they were not given. **A third
    evaluator is the thing to be careful about**: the answer depends on which
    board the caller is entitled to, and only the entry path knows.

    **Reachability (2026-09-07):** closed — CM-2, and the answer is that
    there is no third evaluator. `engine::layers::compute::settled_amount` is
    a third *caller* of the one leaf table: `evaluate_amount` over
    `Board::settled()` at the full ceiling, the same line
    `condition::settled_holds` already was. The item's warning was that the
    answer depends on which board the caller is entitled to; the sharper form
    is that it depends on which **object** the caller means by "source". The
    cost pipeline always hands the evaluator the ability's *own* source — the
    permanent for a `Spells` subject, the spell for `Itself` — so
    `object == source` and `SourcePower` would read `chars.power` with no
    cross-object read at all. The walk cannot say that: there `object_id` is
    the affected object and `origin.source` is elsewhere on the board, which
    is the CR 613.8 dependency it refuses on purpose. So `SourcePower` is
    **answered in the reader and still refused by the walk**, with a test each
    way and `phase_cm_cards::power_reducer` — a fixture, since the objection to
    Golden-Tail Trainer was about the card's name and never about the arm — as
    its consumer. Its board is the entitlement claim made observable: an anthem
    on the source moves the amount. → `cost-architecture.md` §3.7.

    **Sized:** none here.

64. **~~`PermanentFilter` filters objects in zones where nothing is a
    permanent, and the name now lies.~~ ✅ Renamed to `ObjectFilter`
    2026-09-07 (CM-0, `cost-architecture.md` §3.2).** CR 110.1 makes a
    permanent a card *on the battlefield*; RC-5's `AuxiliaryMove.filter`
    matched creature **cards in a graveyard**, `EventPattern::ZoneChange`'s
    `object` filter had matched cards in graveyards and libraries since RB
    (Grafdigger's Cage, Rest in Peace), and CM-1 applies it to spells on the
    stack. The "~120 call sites" this item guessed were 275 in `src/` and 119
    in `tests/` when counted; the sweep also renamed `permanent_matches_filter`
    and its `_in_frame`/`_with` forms to `object_matches_filter*`, and left
    `EffectRecipient::FilteredPermanents` and `SelectionFilter::Permanent`
    alone, since both still name permanents. Zero behaviour: the suite and a
    same-seed `fuzz_games` diff are the check.

    **Reachability (2026-09-07):** closed — renamed.


## Found by CM-1 — cost modification (2026-09-07)

70. **`run_mana_ability_window` closes the moment the pool covers the cost,
    and CR 605.3a has no such clause.** A player may activate mana abilities
    "whenever they are casting a spell or activating an ability that requires
    a mana payment" — with no "until it is paid" — and the Ironworks loop's
    step 3 is exactly the play the early return forbids (`cost-architecture.md`
    §3.11, confirmed by a judge's walkthrough). The stop is a *payer's*
    policy sitting in the engine's loop.

    It was reachable and wrong on any board with a second mana ability worth
    activating after the cost was covered, and invisible to the fuzz harness
    because its provider never wanted to.

    **Reachability (2026-09-08):** closed — CM-4.

    **✅ CLOSED 2026-09-08 (CM-4).** The early return is gone; the loop ends on
    a decline or an empty enumeration and on nothing else. The stop is
    `ui::ManaWindowStop`, a decorator every shipped client stacks — its own
    type rather than part of the payer, because the two toggle independently
    (a human turning off auto-pay wants the window to keep offering; an agent
    without a stop has only `WINDOW_ACTIVATION_CAP`). §3.11's step 3 is now a
    test that fails against the pre-fix tree.

    **What it cost, measured and not predicted:** the stop is narrower than the
    engine's was. The engine returned when `can_pay_costs` succeeded over the
    *whole* cost list; the decorator declines when the *mana component* is
    covered, which is the only thing a mana ability can fix. They differ on one
    board and it is reachable — see item 83.

71. **The window's opening condition is right by accident.** CR 601.2g opens
    it only "if the total cost includes a mana payment" — casting Mox Opal
    offers none — and `run_mana_ability_window` is called unconditionally,
    returning at once because a zero cost is already payable. Item 70's fix
    removes that return, so it must add the 601.2g test: open iff the locked
    mana component is non-empty (a component reduced to nothing, "considered
    to be {0}", read the same way — the one residual question §3.11 leaves
    for a judge).

It was a record for item 70's fix — the answer was right, re-derived
    after CM-2 — and what moved was the *board*: affinity is the first printed
    mechanic that reduces a mana component to nothing, so "a component reduced
    to nothing, considered to be {0}" became reachable from a measured game (a
    pooled Myr Enforcer behind seven artifacts) rather than only from a
    fixture. Sized at ~5 lines with item 70, and that is what it took.

    **Reachability (2026-09-08):** closed — CM-4.

    **✅ CLOSED 2026-09-08 (CM-4).** The gate is explicit and the residual
    reading is decided: a component reduced to nothing opens no window, the
    same as a printed `{0}`. By 601.2g the total is locked — a `Vec<Cost>` with
    no record of how it got there — so distinguishing the two would mean
    carrying a history CR 601.2f exists to discard. Tested both ways (seven
    artifacts make Myr Enforcer free and no window opens; six leave `{1}` and
    one does), and both tests fail with the gate removed. The gate reads the
    component's *symbols*: `determine_total_cost` always emits a `Cost::Mana`,
    empty when the total is `{0}`.

74. **The generic split and the mana window read only the first
    `Cost::Mana` — closed by CM-1's merge.** A kicked spell's kicker mana
    was a second `Cost::Mana` in the assembled list, so its generic was
    allocated against the base cost's split and the window sized itself
    against the base cost alone. CR 601.2f's "the mana component of the total
    cost" is singular; `determine_total_cost` merges every `Cost::Mana` into
    one before the arithmetic. No registered card kicks, so no fuzz game had
    reached it.

    **Reachability (2026-09-07):** closed — CM-1.


## Found by CM-2 — the spell's own cost abilities (2026-09-07)

76. **`Effect::as_…` says what an ability *is*, never where it applies from,
    and all three cost gates are about where.** Two gates got this wrong on
    the same day, from opposite directions. `register_static_effects` recorded
    any static whose body is a cost modification, so an affinity permanent
    became a battlefield "source" that widened CR 601.2f's sweep on every cast
    for a match no permanent can satisfy (`CostSubject::Itself` is an identity
    test). Source 2's gate asked whether the card prints a cost ability, which
    is true of a Thalia *in hand*, so her frame was computed at every
    castability preview and then refused. Both now ask `CostSubject` a
    question — `applies_from_battlefield` and `applies_to_its_own_object`,
    matched exhaustively and **not** each other's negation, since CR 602.2b's
    activated abilities will answer `true` and `false` respectively.

    **Reachability (2026-09-07):** closed — CM-2. Neither was ever a wrong
    *answer*; both were wasted work, and the second was found only by the A/B
    arm that must reproduce `main` (+5 layer walks per 200 games, every
    gameplay counter identical). That is the arm's whole justification: a
    five-walk regression is not worth finding by argument.


## Found by CM-3 — lock-in's payment side (2026-09-07)

82. **CR 704.5p's first sentence is not implemented: an Equipment that
    becomes a creature stays attached.** "If a battle or creature is attached
    to an object or player, it becomes unattached and remains on the
    battlefield. **Similarly**, if any nonbattle, noncreature permanent that's
    neither an Aura, an Equipment, nor a Fortification is attached …" —
    `engine::sba` implements the second sentence only, filtering out Auras,
    Equipment and Fortifications, so a permanent that qualifies under the
    *first* sentence is skipped by the very predicate meant to spare it. The
    block is also commented "704.5q", which is the +1/+1 / −1/−1 counter rule;
    the other attachment comments in that function want the same audit.

    **Reachability (2026-09-08):** closed — fixed the same day. `engine::sba`
    now implements 704.5p as one pass over the attachments: a permanent that
    *is* a creature is unattached whatever its subtypes say, and the second
    sentence's catch-all follows. `ATOM-704.5p-001` — uncovered since Phase
    5-Pre — is covered, and the standing Aura TODO closed with it rather than
    beside it: an Aura that becomes a creature is unattached by the same
    predicate, and 704.5m puts it into its owner's graveyard on the loop's next
    pass, which is the CR's own composition and is now a test.

    **Two labels were wrong and are fixed with it.** The Equipment-with-an-
    illegal-host block was commented 704.5p (it is 704.5n), the catch-all was
    commented 704.5q (it is 704.5p's second sentence; 704.5q is counter
    annihilation), and the Aura block's inner comments said 704.5n for cases
    that are both 704.5m. A rule number in a comment is the only index this
    file has into the CR, so a wrong one is worse than none.

    **An 11% speed-up came with the bug, and it was not the bug's doing.**
    Both attachment sweeps asked their subtype questions *before* reading
    `attached_to` — three `has_subtype` calls for every permanent on the
    battlefield, every check, to answer a question about the handful that were
    attached. Hoisting the field read in front of the frame computations takes
    memo hits **99,530 → 62,215 per `performance` game (−37.5%)** and
    97,783 → 61,424 on `stress`, and CPU/game **16.09 → 14.29 ms (−11.2%)**,
    with layer walks (378 → 379) and frames (4,504 → 4,510) unchanged — so it
    is fewer questions asked, not cheaper answers. It more than repays CM-3's
    pool addition. `engineering-practices.md` §3 has the re-recorded table.

    **How it was found, which is the point:** the rulings pass
    (`engineering-practices.md` §3.4), on its first day, reading a ruling on a
    card registered months earlier — "If an Equipment becomes a creature, it
    can no longer equip a creature. If it's currently attached to a creature,
    it becomes unattached." Nothing in the corpus, the test suite or the fuzz
    harness had said so.


## Found by RD-2 — CR 615.7 prevention shields, and the loop's unit (2026-09-09)

91. ~~**`AmountRewrite::PreventUpTo` has a performer and no printed
    producer.**~~ **Closed by RD-3 (2026-09-09): both printed producers are
    registered.** Guardian Seraph writes it against a player with a source-side
    `ByController(Opponent)`; Daunting Defender writes it against a filtered
    object set with no source constraint at all, which is CR 615.10's own
    example and is why the two are not one path twice. Guardian Seraph is in
    `PERFORMANCE_POOL`, so the arm is now reachable from a measured game.

    The original entry, for the record: RD-2 shipped the arm because a CR 615.7
    count is cut down to it at application (`AmountRewrite::capped`) and because
    the group form's member-uniform path needed a static partial prevention to
    prove "rewrites per member" against two simultaneous sources — done through
    a fixture row whose test name said whose card it was waiting for. That
    fixture is now the printed board.

    **Reachability (2026-09-09):** closed — both printed producers are
    registered and Guardian Seraph is pooled, so the arm is reachable from a
    measured game (145 cast / 145 resolved in 102 of 200 forced `stress`
    games).

    **Sized:** none.


## Found by the RD-2 review (2026-09-09)

97. **A stale claim written the same day it was found, and only a reader
    caught it.** RD-2's docs commit closed item 65 with "`forced_bucket` keeps
    its name — 'bucket' is CR 616.1a–e's own word there". **It is not.**
    CR 616.1a–e is a ladder of steps, each reading "if any … one of them must
    be chosen. If not, proceed to [the next]"; the word "bucket" appears
    nowhere in the rule, and it entered this codebase as an implementation
    word. The reviewer did not check the rule — they asked what a bucket
    *was*, which is the same instrument pointed at the same defect.

    So the rename went through: `forced_bucket` → `must_choose_among`
    (616.1a's own sentence), the local `bucket` → `choosable`, and the ~25
    doc uses in the 616.1 sense → "step". The word survives in
    `DecisionProvider::allocate`, where a bucket is a thing you allocate a
    total across and is nobody's confusion.

    **This is item 89's shape for the third time** — a comment stating a
    checkable fact, false when written, invisible to every test. Item 89 said
    the fix is "a re-read with an instrument". The instrument that worked here
    was a human asking what a word meant, which does not scale and does not
    run in CI; "Before card breadth" item 11's glossary check is the one that
    would have.

    **Reachability (2026-09-09):** closed — the claim is corrected in place
    and item 65 records what it got wrong.

    **Sized:** none beyond item 11.

98. **`next_damage_shares` re-checked its own guard, and the second check read
    as a mystery.** The chooser agreement test was `if chooser.is_none() ||
    any(differs) { return Err }` followed by `chooser.expect("checked")` — and
    the `expect` was read on review as *comparing a string*, which is a fair
    reading of a line whose only visible argument is a string. It is one
    `match` now, with no unreachable arm to explain. Recorded because the
    lesson is not about `expect`: **a guard whose failure path returns and
    whose success path re-derives the same fact wants to be one expression**,
    and the tell is that the second step needs a comment.

    **Reachability (2026-09-09):** closed.

    **Sized:** none.


## Found by RD-3 — sources (2026-09-09)

101. **`EffectRecipient::FilteredPermanents` now has two readers with the same
     semantics, and its doc said it had none.** The variant's comment read "Not
     used at cast/resolution time — only read by the ETB hook to register
     continuous effects"; RD-2's `Primitive::CreateReplacement` already read it
     at resolution (CR 615.11's one row per permanent) and RD-3's
     `Primitive::DealDamage` is the second. Both resolve it the same way —
     `battlefield_ids_ordered` filtered by `object_matches_filter` against the
     resolution's controller, **now**, not captured — which is what makes it
     the right vehicle for "each creature" and for CR 615.11 alike.

     The filter is resolved inside each primitive rather than filled into
     `ctx.targets`, and that is deliberate: writing it into the targets would
     make "each creature" a *targeting* fact, which CR 115.1's "targets are
     announced as the spell is cast" says it is not, and would change what the
     recipient means to the static-ability path that shares it.

     **Reachability (2026-09-09):** closed — two readers, both tested.

     **Sized:** none. The doc line is corrected in place.


## Before Layers (CR 613) — now DURING Layers

1. **Pre-layer P/T shim — ✅ done.** `PermanentState.power_modifier` / `toughness_modifier` no longer exist anywhere in `src/`. Layer 7c output replaced them.

   **Reachability (2026-09-03):** closed.

2. **Direct `CardData` reads — ✅ done (2026-08-19).** 21 battlefield/stack call sites now route through `oracle/characteristics.rs`. New predicate helpers `has_type`, `has_subtype`, `has_supertype`, `has_permanent_type` join the existing `is_creature` / `get_effective_*` wrappers.
   - Migrated: `engine/sba.rs` (8 — planeswalker loyalty, legend rule, Aura/Equipment/Fortification attachment SBAs), `engine/targeting.rs` (7 — creature target, creature-or-planeswalker target, the whole `ObjectFilter` match), `engine/resolve.rs` (Aura ETB), `engine/stack.rs` (2 — permanent-spell routing, Aura spell), `state/game_state.rs` (ETB loyalty counters), `ui/display.rs`, `ui/random.rs`.
   - **Deliberately NOT migrated (6 sites):** `engine/zones.rs:144` (play a land from hand), `oracle/legality.rs:59` (playable lands in hand), `oracle/mana_helpers.rs` (×4 — castable spells in hand, instant/flash timing). These are cast-zone / play-from-hand legality, evaluated before the object is a permanent, so the layer system has nothing to contribute. Same exemption as `engine/cast.rs`. Each is tagged `// PRE-LAYER ZONE:` in source so a future grep audit doesn't re-flag it.
   - Regression coverage: `mtgsim/tests/layer_aware_queries_test.rs`, 5 tests. Verified to fail against the pre-fix tree and pass after.

   **Reachability (2026-09-03):** closed — 2026-08-19.

3. **~~Cost modification pipeline stub — ❌ still a passthrough.~~ ✅ CM-1 (2026-09-07, `plans/cost-architecture.md`).** `engine/cost_determination/` is CR 601.2f's order — merge, gather, increases, reductions in the caster's order under CR 118.7a–d, Trinisphere, the lock — and `assemble_total_cost` calls it. **Not** "wired to the continuous-effects registry", as this item said on 2026-08-24: a cost effect has no layer and applies to no object, so it is discovered off its source's *effective* ability list at 601.2f, the way a replacement effect or a "can't" is (§3.1 of the doc has the reasoning; the sentence here was written before RB built that pattern). Thalia, Guardian of Thraben (pooled), Goblin Electromancer and Trinisphere are the consumers. **Commander tax is still a cost modification** (CR 903.8) and still has no payer: it ships with `GameConfig::commander()` as one arm in `total.rs` step 1 (§3.8).

   **Vocabulary gap this also owns.** Golden-Tail Trainer — "Aura and Equipment spells you cast cost {X} less to cast, where X is this creature's power" — is a static ability whose amount is read live. `AmountExpr` cannot say "this creature's power": `TargetPower` means the target of a resolving spell, and `Variable` is CR 107.3's X, chosen as a spell is cast. A `SourcePower`-style variant is needed, and the card is blocked on this item too, since cost modification is CR 613.11 / 601.2f rather than a characteristic change.

   **Reachability (2026-09-07):** closed — CM-1. What is left is sized in
   `cost-architecture.md` §6: CM-2 (the spell's own cost abilities and the
   dynamic-amount evaluator, which is item 57's third and has its board
   argument in §3.7), CM-3 (sacrifice as a cost, the CR 601.2h example),
   CM-4 (the mana window and the payer), CP-1 (payment); commander tax with
   `GameConfig::commander()`.

   **Sized:** in the doc, per phase.

5. **Timestamps — ✅ live.** `PermanentState.timestamp` is now read by the layer system for 613.7 ordering (4 read sites). The CR 613.8 *dependency* algorithm is still unimplemented; ordering is timestamp-only.

   **Reachability (2026-09-03):** closed — timestamps are live; the 613.8 half
   is item 8 below.

6. **Direct `card_data.abilities` reads — ✅ done (2026-08-20, Phase LD Part B).** 7 battlefield sites route through the new `oracle::characteristics::get_effective_abilities`, because CR 305.7 makes printed abilities wrong for a Blood-Mooned land.
   - Migrated: `oracle/mana_helpers.rs` (×2 — `available_mana_sources`, `activatable_abilities`), `engine/mana.rs` (`activate_mana_ability`), `engine/priority.rs` (×2 — mana dispatch, id→index), `engine/cast.rs` (`activate_ability`), `ui/display.rs`.
   - **Index coupling:** `activatable_abilities` produces an ability index, `priority.rs` re-derives it by id, `cast.rs::activate_ability` consumes it. All three index the *effective* list. Migrating one alone silently activates the wrong ability — they move together or not at all.
   - **Deliberately NOT migrated:** `state/game_state.rs::register_static_effects` — see item 7. Plus `engine/cast.rs:56`, `oracle/mana_helpers.rs:174`, `engine/stack.rs:259` (spell abilities read pre-battlefield, same class as the `// PRE-LAYER ZONE:` sites).
   - Cost: `fuzz_games` went 25.9 → 29.2 ms/game (+12%, ±3% run-to-run) at 200 games / seed 12345, because mana-source enumeration is now a `compute_characteristics` walk per permanent instead of a field read. Accepted for now — `layers-architecture.md` §15.2 item 1 defers caching until after profiling, and this is the profiling. Revisit if it compounds when Layer 6 lands.

   **Reachability (2026-09-03):** closed — 2026-08-20.

7a. **Frame cache is live.** `layers-architecture.md` §5.2's per-call `(ObjectId, layer_ceiling)` memo now exists, because the existence check needs another object's characteristics mid-walk. The strictly-descending ceiling **is** the termination argument, and it is load-bearing: `test_self_stripping_land_terminates_and_is_stable` overflows the stack if the check asks at the full ceiling instead of `layer_index`. Only sub-computations are memoized; the top-level frame is requested once per call, so caching it would be a pure clone.

    **Reachability (2026-09-03):** closed — and the cross-call half is
    critical-path item 7a, PR #92.

7b. **CR 613.7a clause 2 — ✅ implemented (2026-08-23).** "…or the timestamp of the effect that created the ability, whichever is later." It is the `max()` in `GameState::static_effect_timestamp`, which now takes `granted_at: Option<Timestamp>`; `None` means printed, and `register_static_effects` — running at ETB off printed text — is the caller that always passes it. `resolve::register_granted_static_effects` is the clause-2 caller.

    Tested so the two clauses disagree: grantee entered long ago, a Layer 7b `SetPowerToughness(9,9)` sits at timestamp 50, and the granting spell resolves after 60. Clause 1 alone gives 9/9; clause 2 gives 3/3. Deleting the `max()` fails that test and only that test.

    **One shape does not work, and it asserts rather than failing silently.** A granted static ability whose own effect lands in **layers 1–6** cannot apply: the grant applies *at* layer 6, so at any layer ≤ 6 the frame the CR 604.2 existence check reads is the pre-grant frame, and the derived effect finds no ability to justify itself. This is not a corner — it is CR 613.7a's own worked example, Rune of Flight granting "Equipped creature has flying", which is a layer 6 effect. That card needs Equip as well, so it is out of reach twice over. `register_granted_static_effects` carries a `debug_assert!` at the registration site so a card author is stopped instead of shipping a card that quietly does nothing.

    **The two halves of that limitation are not one problem, and only one is waiting on anything.**

    - **Layer 6 exactly** — Rune of Flight, "As long as enchanted permanent is an Equipment, it has 'Equipped creature has flying.'" The CR resolves this purely by timestamp within layer 6, and our *ordering* is already correct for it: clause 2 makes the derived timestamp `max(grantee, grant)`, which is ≥ the grant's, and on a tie the grant row is registered first so it wins the `EffectId` tiebreak in `effects_in_layer`. The grant therefore always sorts at-or-before its own derived effect. The single missing piece is that `static_ability_still_exists` asks `compute_to_ceiling(source, layer_index)` — the frame as of the *end of the previous layer* — and so cannot see a partially-applied current layer. Item 8 step 4's board-wide sequential pass is exactly that frame: apply a layer over its ordered applications, mutating a per-object map, and the check at position *k* sees everything applied earlier in the same layer. That is the same change 613.8b's loop rule needs to become exact, which is why they belong in one phase. Nothing about the Layer 6 work needs redoing — only the frame the check reads.
      (Rune of Flight additionally needs Equip and item 7f's conditional statics, so it is three things away, not one. **Two of the three landed** — Equip with LH-2, the condition with LI-3 — and the third turned out to be item 7g, not this: the Equipment clause grants a *static ability* from a static ability, and only a resolution derives those rows. LI-3's `phase_li_cards::flight_clause` is the line above it, "as long as enchanted permanent is a creature, it has flying", which needs none of that.)

    - **Layers 1–5** — a layer 6 grant whose ability generates a layer 4 or 5 effect. This is *not* scheduled, and not because it is hard. CR 613.8a(a) confines dependency to a single layer, so the CR itself supplies no mechanism for a later layer to reach back into an earlier one; any ordering we picked would be invented rather than implemented. Searched Scryfall for granted statics that define a type, color or subtype — every hit is a false positive (quoted text inside a granted *activated* ability, plus Animate Dead's enchant clause). Real grants are of triggered abilities, activated abilities, keywords, or layer 7 statics. Revisit if a card ever appears; there is nothing to build against today.

    **Reachability (2026-09-06):** closed — LI-1, for the layer 6 case. The
    board-wide pass (`engine/layers/board.rs`) applies a layer over its
    ordered applications with one live frame per member, so the existence
    check at a derived row's turn sees the grant applied earlier in layer 6;
    `register_granted_static_effects`' assert admits layer 6 and refuses
    layers 1–5 only. Humility + Citanul Hierophants gives the CR's answer
    in both orders (`test_humility_before_hierophants_retires_the_grant`,
    the flipped pin), and `tests/phase_li_integration_test.rs` builds the
    Rune-of-Flight shape by resolution. The layers 1–5 half stays as
    recorded above: no CR mechanism and no card.

    **Sized:** done for layer 6 (LI-1); layers 1–5 have nothing to build.

7c. **CR 613.6 "existence persists once started" — implemented, untested.** The `started` set in `apply_effects` keys on `EffectGroup`, so an effect that has begun applying keeps applying even if a later layer removes its ability. No test: every construction available today puts the strip in the *same* layer as the effect's first part, so the correct answer depends on 613.8 dependency ordering, and a test now would pin the timestamp-only answer that 613.8 must change. See item 8.

    **Reachability (2026-09-06):** closed — LI-2, tested. Humility +
    Opalescence in both orders and the two-Opalescence board
    (`tests/phase_li2_integration_test.rs`, the 2009-10-01 and 2006-02-01
    rulings quoted beside the assertions): an Opalescence animated by
    another loses its ability in layer 6 and its 7b part still applies to
    the set it locked in layer 4; Humility animated by Opalescence loses its
    own and its 7b part still applies. `// COVERS: ATOM-613.6-003`. (History:
    LI-1's pass built the locked set — `Board::started`, per `EffectGroup` —
    and the boards that separate the two answers are same-layer
    strip-plus-effect constructions whose order is CR 613.8's, which is why
    the test waited for LI-2.)

    **Sized:** done.

7e. **Derivation silently drops non-`Fixed` amounts — ✅ done (2026-08-22).** `EffectModification::{SetPowerToughness, ModifyPowerToughness}` now carry a `PtValue`: `Fixed(i32)` for a signed literal, `Dynamic(AmountExpr)` for an expression re-evaluated at every layer by `compute::evaluate_pt_value`. Two variants rather than one because `AmountExpr::Fixed` is `u64` and `ModifyPowerToughness { power: -1 }` needs a sign. Resolution-time effects stay `Fixed` (CR 608.2h locks a resolving spell's value in); it is static abilities that must stay live (CR 604.7). March of the Machines is back to its printed "equal to its mana value" via `AmountExpr::AffectedManaValue`.

    Residue, now loud instead of silent: the evaluator returns `Option<i32>` and `debug_assert!`s on an amount with no static-context meaning. `Variable` genuinely cannot appear on a static ability (CR 107.3's X is chosen as a spell is cast), but the `Target*` family points at a real vocabulary gap — see item 3.

    **Reachability (2026-09-03):** closed — 2026-08-22.

7h. **Two epoch bumps the 7a memo is owed (recorded 2026-09-03).** The memo's
    key is one epoch, and a walk input written without a bump is a stale
    answer that only the debug audit can see, and only when a hit is served.
    (1) **LH** makes `attached_to` / `attached_by` and the CR 613.7 timestamp
    walk inputs; every writer of them — `attach_to` / `detach` and the direct
    field writes in `resolve.rs`, `sba.rs`, `stack.rs`, `zones.rs`, and
    `test_support::attach` — calls `GameState::bump_layer_epoch` in the same
    PR that makes the walk read them, and not before (`layers-architecture.md`
    §13a). (2) **Item 7's board-wide pass**, if it stores an order into
    registry rows in place: `ContinuousEffectRegistry::mutating` tells a
    write from a no-op by `len`, which is exact because no `DurationRegistry`
    mutator edits a row in place today; an in-place mutator bumps `mutations`
    itself or the memo serves the pre-pass order.

    **Reachability (2026-09-05):** closed — LH-2. `attach` reassigns the
    CR 613.7 timestamp (613.7e) under the bump it already had for
    `attached_to`, then re-stamps the source's static rows in place
    (`retime_static_rows`, 613.7a's third sentence) — part (2)'s mutator,
    built rather than avoided (review, 2026-09-06). The `len` heuristic it
    would have fooled is gone: `DurationRegistry` keeps a generation counter
    that every add, remove and in-place edit bumps, and
    `ContinuousEffectRegistry::mutations` *is* that counter, so a closure
    that added and removed in one call, or edited a row, is seen. Item 7's
    in-place order, if it stores one, goes through `update_rows`. (History,
    2026-09-04: (1)'s `attached_to` half landed in LH-1 — the walk reads it
    through `AffectedSet::Host`, and every writer goes through
    `GameState::attach` / `detach`, which bump.)

    **Sized:** the timestamp bump, one line inside LH-2's attach site — the
    placement this item first gave *all* the bumps was stale, since it is the
    PR that makes the walk read a field that owes the bump, and for
    `attached_to` that was LH-1; item 7's in-place mutator bumps itself, in
    item 7's PR.

8. **CR 613.8 dependency — two known-wrong cases, both Blood Moon — ✅ done (2026-09-06, LI-1 + LI-2).** Under timestamp-only ordering the engine gets both of these wrong. They are the concrete motivating cases for the 613.8 phase, and together they show why 305.7 is applied per-effect: dependency detection needs effect identity to hang a relation on.

   - **Rootpath Purifier** ("Lands you control and land cards in your library are basic") changes the set of permanents Blood Moon affects, so Blood Moon *depends* on it and applies second regardless of timestamp — Blood Moon never touches that player's lands. Its ruling (Scryfall, 2022-10-14) says exactly this: "if an opponent controls Blood Moon … and you play Rootpath Purifier, Blood Moon can no longer apply to the lands you control because they are all basic." We get this wrong whenever Blood Moon has the earlier timestamp.

   - **Intra-layer re-evaluation is part of 613.8, and the written design omits it.** `layers-architecture.md` §5 orders each layer once via `resolve_order_within_layer`, then applies in that order. The CR re-evaluates dependencies **after each effect is applied** (613.8c) — apply one independent effect, recompute every remaining pair, repeat — which is how "Urborg no longer has an effect so we're done in layer 4" falls out, and Urborg's own ruling (Scryfall, 2021-03-19) states the result: an effect "such as that of Magus of the Moon" that sets it to a basic land type not in addition to its others means "it won't turn lands into Swamps, no matter in what order those effects started to apply". §9's hybrid algorithm needs to run inside that loop, not once per layer. The walkthrough this bullet cited from memory — a judge's answer on a Reddit thread, Blood Moon + Urborg + Ashaya + Opalescence, four effects in layer 4 — was recovered on 2026-09-06 and is quoted in full in `plans/references/blood-moon-urborg-ashaya-opalescence-judge-answer.md`: Opalescence, then Ashaya, then Blood Moon, and Urborg never applies. It is a worked procedure rather than a ruling, and it is LI-2's CR 613.8c test board.

   - **CDAs are not in the DAG. Here is what to do if that ever costs us.** `engine/layers/cda.rs` applies characteristic-defining abilities intrinsically, so no CDA is ever a registry row. 613.8a(c)'s *first* clause — "neither effect is from a characteristic-defining ability" — therefore holds structurally, for free. Its *second* clause, both-CDA dependency, is currently unreachable, and the reason is worth stating precisely rather than filed as "can't do".

     **What would trigger it.** A CDA that reads a characteristic another CDA sets **in the same layer**: the hypothetical "this creature's power is equal to the greatest power among other creatures on the battlefield". 604.3a(3) bars a CDA from *affecting* another object but not from *reading* one, and 613.8a(a) requires the same layer, so this is the only shape that qualifies. Every printed CDA reads non-layer information (graveyards, hands, life) or strictly lower-layer information — Nightmare and Master of Etherium read Layer 4 type counts at Layer 7a — and those are independent under 613.8a(a). Searched Scryfall for both same-layer shapes: zero cards.

     **What we would get wrong.** Two copies of that card depend on each other, which is a *loop*, and 613.8b resolves loops by ignoring dependency and applying in timestamp order — so the symmetric case needs nothing. The asymmetric case is the live one: one power-reader plus any other Layer 7a CDA (a Tarmogoyf). The power-reader depends on Tarmogoyf and Tarmogoyf does not depend back, so 613.8b makes Tarmogoyf apply first and the reader must see its **post-7a** power. `evaluate_amount` resolves other objects at `compute_to_ceiling(other, layer_index)` — the frame as of the *end of the previous layer* — so it would read Tarmogoyf's pre-7a value.

     **The fix, and why it is not extra work.** The root cause is not that CDAs are intrinsic; it is that `compute_characteristics` walks **one object at a time** while CR 613 describes a board-wide pass per layer. The per-object walk with a descending ceiling (`layers-architecture.md` §5.2) is an optimization that is exact exactly while no two objects have a same-layer dependency — which is also the condition 613.8 exists to handle for registry effects. So:

     1. Make the unit of ordering in a layer an *application* rather than a registry row: either a `ContinuousEffect` row or one object's intrinsic CDA application. The intrinsic pass already produces `EffectModification`s, so this is a wrapper type, not a redesign — it is the whole of what CDAs need to join the DAG.
     2. `resolve_order_within_layer` then sees both kinds. 613.8a(c) prunes CDA↔non-CDA pairs; 613.3 keeps CDAs ahead of the independents.
     3. Termination stops being "the ceiling descends" and becomes "the dependency graph is acyclic", with 613.8b's loop rule supplying the acyclicity: a loop means no edges, so members apply in timestamp order.
     4. Mechanically that wants a per-layer memo keyed `(ObjectId, layer_index)` holding the **post-layer** value, filled in dependency order. Note the honest limit of the cheap version: marking a key in-progress and falling back to its pre-layer value on re-entry *approximates* 613.8b rather than implementing it — 613.8b has loop members apply in timestamp order relative to each other, so the second one does see the first one's result. Getting that exact means applying the layer board-wide in one sequential pass, i.e. a `compute_all(game)` with today's single-object entry point as a projection of it.

     Step 4 is the expensive one and it interacts with §12's cross-call memoization, which is already scheduled between Layer 2 and 613.8 — they should land together. Steps 1–3 are cheap and are the ones the 613.8 phase would do anyway.

   - **A test is waiting on this.** CR 613.6's "an effect that started applying keeps applying even if its ability is removed" is implemented (item 7c) but untested, because every construction available today puts the strip in the same layer as the effect's first part. Once 613.8 lands, that test can assert a stable answer.

   - **Urborg, Tomb of Yawgmoth.** Urborg is itself a Legendary — therefore nonbasic — Land, so Blood Moon turns Urborg into a Mountain and CR 305.7 strips the ability generating Urborg's effect. Applying Blood Moon changes the *existence* of Urborg's effect (613.8a(b)), so Urborg is dependent and applied last, by which point it does nothing. **Blood Moon wins in both orders.** There is no reverse dependency: Urborg grants the Swamp subtype, never the `Basic` supertype, and CR 305.8 makes a land nonbasic on the supertype alone. We currently produce order-dependent results here. Fixing it needs 613.8 *and* item 7 (a stripped static ability must retire the effect it registered at ETB) — 613.8 alone is not sufficient. `phase_ld_cards::urborg_effect()` is deliberately an Enchantment so the 305.6 tests don't depend on any of this.

   **Reachability as recorded 2026-09-03, superseded below:** reachable — wrong today, on a third board the
   pool can build; the two above cannot be (Rootpath Purifier is not in the
   tree; `urborg_effect` is an unregistered Enchantment fixture). The third is
   **Humility + Citanul Hierophants**, both in `PERFORMANCE_POOL`, probed
   2026-09-03: Humility on the battlefield first, then Citanul Hierophants and a
   Grizzly Bears under the other player — the engine gives the Bears `{T}: Add
   {G}`; the CR gives it nothing. CR 613.8a(b) makes the Hierophants' grant
   depend on Humility (applying Humility removes the ability that generates it),
   so Humility applies first whatever the timestamps say, and when the grant's
   turn comes its ability no longer exists — the same answer the
   Humility-plus-lord rulings give. The engine orders by timestamp, which here
   happens to agree, and then applies the grant anyway, because
   `static_ability_still_exists` reads the Hierophants' frame as of the end of
   Layer 5 (item 7b's limitation). With the Hierophants first the answer is
   right. Observable in a game: a creature under Humility taps for mana.

   **Reachability (2026-09-06):** closed — LI-2. Steps 1–4 are built:
   `engine/layers/board.rs`'s `resolve_order_within_layer` decides
   dependencies against the live board (`depends_on`: a static channel check,
   then a hypothetical applied under a journal and taken back), applies the
   earliest application that waits on nothing pending — or, in a loop, the
   earliest of the loop (613.8b) — and re-decides after every application
   (613.8c). The three boards answer as the rulings do, in both timestamp
   orders (`tests/phase_li2_integration_test.rs`): Urborg, Tomb of Yawgmoth
   is registered and pooled (its 2021-03-19 ruling); the Rootpath Purifier
   ruling's board is `phase_li_cards::purifier_clause`; Ashaya, Soul of the
   Wild is registered with a CR-derived answer; and the judge answer's
   four-card board applies Opalescence, Ashaya, Blood Moon and never Urborg,
   asserted step by step through the pass's trace hook. One departure from
   the steps below: the unit of ordering is an *effect's rows in the layer*,
   not a row — Ashaya's two layer-4 rows must not be split by Blood Moon —
   and item 16 records what that keying leaves. History, kept for the
   reasoning — the older paragraph read: reachable — wrong today on the two Blood
   Moon boards; the third is fixed. Step 4 is built — LI-1,
   `engine/layers/board.rs`, traced call by call in
   `plans/traces/li-1-one-pass-per-board.html`: Humility + Citanul Hierophants answers as the
   CR does in both orders, and the pool's "engine, pool unchanged" A/B arm
   differs from `main` in one game in forty per pool for exactly that reason
   (`engineering-practices.md` §3). Step 1 is built with it (an
   `Application` per row, CDA or counter, carrying `is_cda`); steps 2 and 3
   — the dependency graph, 613.8b's loop rule, 613.8c's re-evaluation — are
   LI-2 — walked call by call in
   `plans/traces/item-7-an-effect-waits-for-what-it-reads.html`, with LI-3's
   conditional-existence board beside it — worked from the rulings: Urborg, Tomb of Yawgmoth registered, the
   Rootpath Purifier ruling's board as a named fixture (the Purifier itself
   waits on item 9), Opalescence registered for the Humility rulings, and
   Ashaya, Soul of the Wild as the printed card of the applies-to shape with
   a CR-derived answer no ruling covers (`layers-architecture.md` §13b).

   **Sized:** done — LI-2 (`layers-architecture.md` §13b, as-built); step 4
   shipped in LI-1 at +1,013 / −775 in `src`.

12. **The card → registry lowering is loud — ✅ done (2026-08-23).** `register_static_effects` had five arms that declined to lower something and `continue`d, registering nothing and saying nothing. Every one now `debug_assert!`s first.

    **Why this class is worth its own item.** A dropped atom produces a card that is *inert* — it panics nothing, computes nothing wrong, and stays perfectly deterministic. `fuzz_games` structurally cannot see it: it catches crashes and non-determinism, and a card that does nothing exhibits neither. This codebase has already paid for the pattern twice (item 7e was a `continue` on a non-`Fixed` amount that "silently dropped the whole atom and failed no test"; item 7f was the same shape until LI-3 closed it, and item 7g is a third variant — loud at the door and silent afterwards, which is why it took a board to find). Refusing to be quiet at the door is the only check that catches it.

    - **The two lowering steps are now shared,** as `GameState::static_ability_atoms` and `GameState::static_affected_set`, and `resolve::register_granted_static_effects` routes through both. They had been two hand-copied matches, which is the drift `static_primitive_rows`' doc comment already warns about: identical card text must behave identically whether printed or granted.
    - **`debug_assert!` rather than a hard error,** matching the existing asserts in `register_granted_static_effects` and `compute::evaluate_pt_value`. A card author running the suite is stopped; release keeps the old skip-and-carry-on rather than panicking mid-game.
    - **Nine unit tests in `state/game_state.rs`** (`mod static_lowering`) — three positive controls plus one `#[should_panic]` per declining arm. An assertion nothing exercises is indistinguishable from one that never fires.
    - **`tests/card_pool_lowering_test.rs` puts every registered card onto the battlefield** under live assertions, for both controllers. Neither existing check reaches this: `fuzz_games` is a release build so `debug_assert!` is compiled out, and the per-phase suites each place only their own handful of cards. Verified non-vacuous by temporarily registering a conditional static — it fails by card name with an actionable message.

    **Findings from the audit, none of them bugs, all of them scope.** The existing 63-card pool lowers completely clean. But three `static_primitive_rows` arms had *no card reaching them at all*, and the reasons differ:

    - **`GrantAbility` — now covered.** `cards/phase_lf_cards::citanul_hierophants` ("Creatures you control have '{T}: Add {G}'", real card, verbatim). Every other `GrantAbility` in the pool arrives through a *resolution*, which builds `AffectedSet::Fixed` against the spell's targets — a different path from a live `Filter`. It works because the granted body is a *mana* ability and so generates no continuous effect of its own; swap it for a static body and it lands in item 7's open filter half immediately. It also crosses into mana enumeration, which is the `activatable_abilities` / `priority` / `cast::activate_ability` index coupling CLAUDE.md warns about, and which had no card able to test it end to end.
    - **`RemoveKeywordFlag` — still uncovered, and the reason is a modeling gap, not laziness.** Searched Scryfall for static keyword removal (`oracle:/creatures your opponents control lose/`, excluding instants and sorceries): 9 cards, and 8 of them also say "**can't have or gain** [keyword]". That prohibition is a CR 613.1f continuous effect the engine cannot express at all, and without it the card is not merely incomplete but *wrong* — a later grant would put the keyword back. The one clean exception is Melira, Sylvok Outcast ("Creatures your opponents control lose infect"), and `infect` is not in `KeywordFlag`'s 16 variants, while Melira's other two clauses (poison counters, `-1/-1` counter prohibition) are also unmodeled. **So: no honest card exists for this arm yet, and prohibition effects are what gate the whole class.** Worth its own item when someone reaches for an Archetype.
    - **`LoseAbility(AbilityId)` — no natural card shape.** Real cards say "loses all abilities" (Humility, covered) rather than naming one. The arm exists for effects that already hold an id. Not a gap; recorded so the next audit does not re-flag it.

    **Reachability (2026-09-03):** closed — 2026-08-23. Its residues are
    records: `RemoveKeywordFlag` stays uncovered because its honest cards need a
    "can't have or gain" restriction (RS-era), and `LoseAbility(AbilityId)` has
    no natural card.

11. **Filter `PlayerRef` resolution — ✅ done (2026-08-23), ahead of Layer 2.** `AffectedSet::Filter` carried a `controller: Option<PlayerId>` that `register_static_effects` resolved from `ObjectFilter::ByController(PlayerRef::You)` at ETB. That is a snapshot of who controlled the source when it entered, and CR 109.5 says the opposite — "for a static ability, [you] is the *current* controller of the object it's on". Glorious Anthem kept buffing the team of whoever controlled it at ETB.

    Demonstrable before Layer 2 exists, which is why it shipped as a bugfix rather than as scaffolding: CR 110.2 makes `PermanentState.controller` the default controller, `compute_to_ceiling` seeds `chars.controller` from it, and writing that field is the pre-Layer-2 half of gaining control. `tests/filter_controller_test.rs` was shown failing against the pre-fix tree.

    - **The field is gone; `object_matches_filter` owns the whole question.** `ByController` used to return `true` unconditionally and defer to the `AffectedSet` field, so one question lived in two functions and only one half was re-asked during the walk. That split is what let the snapshot hide, and it had a second victim: `extract_controller_from_filter` walked only `And` nodes, so `Not(ByController(You))` silently dropped its constraint and matched nothing.
    - **"You" is origin-dependent, and both arms are CR text.** `EffectOrigin::StaticAbility` → the source's *effective* controller, via `compute_to_ceiling(effect.source, layer_index)` (CR 109.5). `EffectOrigin::Resolution` → `effect.controller`, fixed when the effect began (CR 611.2c). Same `layer_index` ceiling `static_ability_still_exists` uses — never the full ceiling, per `layers-architecture.md` §5.2.
    - **All four `PlayerRef` variants resolve; none asserts.** `Opponent` is matched as a *predicate* (`controller != you`) rather than resolved to an id, because CR 102.2 makes it one player in a two-player game but CR 102.3 makes "your opponents" a set in multiplayer — the predicate is the same answer in both, and an `Option<PlayerId>` would have been the wrong shape for half the CR. `Owner` resolves to the source object's owner (CR 108.3 / 110.2); no card says it, but it is exactly determined, so asserting would be inventing a restriction.
    - **What Layer 2 hit — ✅ confirmed 2026-08-23, nothing needed redoing.** A Layer 2 effect whose own filter says "you control" asks at `layer_index == 1`, i.e. the frame *before* Layer 2 applied — `PermanentState.controller`. That is exact whenever the source is not itself under a control-changing effect, and it is the CR's own fallback when it is: two same-layer effects where applying one changes what the other applies to are dependent under CR 613.8a, a mutual pair is a dependency *loop*, and 613.8b resolves loops in timestamp order. The exact version needs the frame the check reads to be a partially-applied layer — item 8 step 4's board-wide sequential pass, the same missing piece a granted Layer 6 static ability already waits on. Nothing here needs redoing when it arrives; only the ceiling it asks at.
    - **Two fixes for the cost, both exact, and the attribution matters for Layer 2.** `effect_applies_to` runs *before* the CR 604.2 existence check, so it fires for objects the filter goes on to reject — unlike the existence check, which only fires for matches. Resolving "you" unconditionally therefore added a source-frame walk per non-matching permanent per layer. The fixes: resolve **lazily**, so a filter with no `ByController` node and an `And` that short-circuits on type both cost what they cost before; and **gate** on `RegistryScopeSummary::any_control_changing`, reading `PermanentState.controller` directly while no `SetController` row is registered, because Layer 2 is the only channel that writes `chars.controller` (no CDA lives there, no counter touches it) so the walk's seed *is* its answer.

      All four combinations, interleaved, 200 games / seed 12345, against a 73.0 ms/game pre-refactor baseline:

      | | gate on | gate off |
      |---|---|---|
      | **lazy** (shipped) | 74.8 | 78.1 |
      | **eager** | 82.2 | **749** |

      The 749 belongs to *eager and ungated together*. An earlier revision of this item pinned it on the gate, which would have told the Layer 2 phase to expect a 10x when the flag starts coming on. **It should expect the 78.1 cell — +7% over baseline, and an upper bound at that,** since the measurement forces the walk on every board while the real flag is only true while a `SetController` row exists. Laziness is the half that has to survive future refactoring here; the gate is a further ~4%.

      **How the prediction held, measured 2026-08-23 with Act of Treason in the card pool** — see item 13 for the full numbers. The **+7% ceiling was right and generous: the phase cost +4%**. The **~4% attributed to the gate was wrong by an order of magnitude in the other direction: it is now +28%,** because the phase put 20 more call sites behind it, several inside per-permanent sweeps. And the **sharper gate was built, measured and discarded** — it is not faster, because `ObjectId` is a v4 UUID and the set probe costs a SipHash at every migrated call site on every board to save on the rare one. The bigger lever for this whole class remains `layers-architecture.md` §12's cross-call memoization, already scheduled between Layer 2 and 613.8: the frame cache is discarded per top-level call today, so a board-wide sweep recomputes each source's frame once per object it looks at.
    - **One thing this did NOT fix, recorded as a note rather than an item.** CR 611.2c also says a resolution effect's affected *set* locks in when it begins; an `EffectOrigin::Resolution` row over an `AffectedSet::Filter` still re-filters on every walk, so ATOM-611.2c-001 stays uncovered. No card produces that combination — all three production `Filter` construction sites are static abilities, where re-filtering is what CR 611.3a wants — so there are zero call sites to migrate. It becomes real the first time a resolving spell wants a filter instead of a target list.
    - **Cost: none measurable.** Pre- and post-fix binaries built side by side and run **interleaved**, 200 games / seed 12345: pre 82.0 / 81.0 / 82.0 / 80.6, post 83.7 / 82.5 / 83.0 / 76.5 ms/game — same mean, and the spread inside each set is wider than the gap between them. Measuring the two in separate batches first showed a phantom +7%; interleave, or the drift the perf protocol already warns about invents a regression. Game outcomes are byte-identical to the pre-fix tree at seed 12345, and three runs at one seed still agree byte for byte.

    **Reachability (2026-09-03):** closed — 2026-08-23. The CR 611.2c note is
    unreachable: no `Resolution` row carries a `Filter`; all three `Filter`
    construction sites are static abilities.

14. **The targeting-side `ObjectFilter` could not resolve a `PlayerRef` — ✅ done (2026-08-23).** There are two functions called `object_matches_filter`, and item 11 rewrote only one. `compute::object_matches_filter` asks whether a continuous effect applies to a permanent mid-layer-walk and reads an `EffectiveCharacteristics` frame; `targeting::object_matches_filter` asks whether a permanent is a legal *selection* and reads the finished board. The second still had `_ => Err("PlayerRef {:?} not supported")` for every variant but `Player(_)`, untouched since Phase LD.

    The consequence was silent and total: SBA 704.5n calls `validate_selection` on an Aura's host, got `Err`, and treated the Aura as validly attached. **Every "Enchant creature you control" Aura had an unenforceable restriction** — Ethereal Armor, Gryff's Boon, Angelic Destiny, the whole cycle.

    Fixed by threading `you: PlayerId` through `validate_targets`, `validate_selection`, `validate_permanent_target`, `object_matches_filter`, `has_any_legal_choice`, `is_single_target_legal`, `any_targets_still_legal` and `enumerate_legal_selections`. Every caller had the value already: the caster in `cast.rs`, the spell's controller in `stack.rs`, the Aura's controller in `resolve::try_attach_aura_on_etb` and SBA 704.5n, the enumerating player in `mana_helpers`.

    Two tests fail if either half is reverted — restoring the `Err` arm, or reading `PermanentState.controller` instead of the effective one.

    **Reachability (2026-09-03):** closed — 2026-08-23.

15. **The corpus named a card that does not exist.** ATOM-613.1b-001's board said "Mind Snare". Scryfall 404s on both `cards/named?fuzzy=` and an exact-name search. The name was invented in `plans/archive/implementation-plan-final.md` §L17 ("{3}{U}{U} Instant, GainControl with WhileTargetOnBattlefield" — a re-costed Control Magic) and propagated into the corpus and into `roadmap.md` from there. Substituted Act of Treason, verbatim; the atom's claim is unchanged because its untap and haste clauses are inert for a P/T query. `plans/archive/*` is historical per CLAUDE.md and was left alone; `cards-unlocked-ledger.md` was corrected, and the two live `roadmap.md` sites (the Tier 1 card list and Milestone 4's criterion) followed on 2026-08-24 — they sat inside the slice the staleness banner tells readers to trust, which is the one place a fake card can still mislead.

    **The general lesson is worth more than the fix.** The corpus is authored from a close read of the CR, but its *boards* were written against a plan document rather than against Scryfall, so a card name in an atom is not evidence the card exists. Verify before building to one.

    **Reachability (2026-09-03):** closed — a record; the corpus was corrected.


## Before Triggered abilities (CR 603)

4. **~~The entry hop: Containment Priest's substitute leaves a permanent's worth of zone changes in the log for a card the CR says never entered~~ — ✅ CLOSED 2026-09-02 (RC-4b).** Entering is one proposal: `GameAction::EnterBattlefield` carries `from`, `change_zone` routes a battlefield destination to it, and its performer moves the card, announces the zone change, builds the entity and announces the entry. The Priest's substitute is one `ZoneChange { Graveyard → Exile }` with no LKI and one epoch, a dropped entry leaves the card where it was, and a "can't enter" may watch the entry (`phase_rc4b_integration_test`; `replacement-architecture.md` §9, RC-4b). The token residual is item 52. The record as found (2026-09-02, RC-4; sharpened in review): The `ZoneChange` performer moves the card into the battlefield zone and *then* proposes the `EnterBattlefield` (RC-2's one-`emit`-wide window), so "exile it instead" is performed as a `ZoneChange { from: Battlefield, to: Exile }`. Three things observe that: (a) the log holds a `ZoneChange` *into* the battlefield, so an ETB matcher on the zone change would fire — it must key on `PermanentEnteredBattlefield`, the performer's event, which a permanent that never entered does not have; (b) the log holds a `ZoneChange` *out of* it, `from: Battlefield` with a CR 603.10a LKI frame, so a leaves-the-battlefield or "exiled from the battlefield" matcher would fire, and a "leaves your graveyard" matcher would not, because the recorded `from` is wrong; (c) `zone_change_epoch` advances twice, so CR 400.7 sees two new objects. None is reachable today — no trigger matcher exists — but (b) and (c) have no keying rule that fixes them, so this is a bug-in-waiting for item 6, not a convention. **The fix is to reverse the nesting**, and it is the same restructuring as Deferred Migrations item 46: `GameAction::EnterBattlefield` carries `from`, its performer does the move, the placement and both emissions, and the `ZoneChange { to: Battlefield }` arm forwards to it *before* moving anything. Then the Priest's substitute is one `ZoneChange { from: <source zone>, to: Exile }`, the window is gone, `propose_entry`'s "replaced away" error is gone (a dropped entry leaves the card where it was, which is CR 614.6), a CR 614.17d "can't enter" may watch the entry, and a multi-entry batch is decided in phase 1 like any other proposal. The Priest stays in Root Maze's CR 616.1 bucket, which is what §11 item 19 needs reachable — moving the Priest to the zone change instead would split that bucket and force Priest-first. **Sized:** ~300–500 additions in `actions.rs` (two arms, `propose_entry`), `pipeline.rs` (the `Instead` arm), the token path in `resolve.rs` (`from: None`), and the RC-4 Priest tests' log assertions; CLAUDE.md's "one emitter" line is restated to name the entry performer. **Planned as RC-4b** — `replacement-architecture.md` §9 has the design, the token and CR 608.3e decisions, and the sizing — as its own PR ahead of RC-5, which needs entries to be batch members anyway.

   **Reachability (2026-09-03):** closed — RC-4b, PR #87 (6541d0b).


## Before Commander (CR 903)

1. **Commander damage increment — ✅ done (2026-04-18).** `GameObject.is_commander: bool` added; `execute_action(DealDamage)` accumulates `commander_damage_taken[source]` when `is_combat && target == Player && source.is_commander`. 5 unit tests. The loss SBA (`engine/sba.rs:73`) now has a live writer.

   **Reachability (2026-09-03):** closed — 2026-04-18.

