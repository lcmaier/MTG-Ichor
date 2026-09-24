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


## Found by the look-ahead frame (2026-09-02, RC-4)

46. **The frame is per entry, not per batch, and §5b says it should be per
    batch.** Two Master Biomancers entering as one event should give each other
    nothing — every member's look-ahead reads the pre-batch board. Today an
    entry is proposed *inside* its zone change's performer (RC-2's nested
    `propose_entry`), so the second member of a `[ZoneChange, ZoneChange]`
    batch is decided after the first was performed and sees it on the
    battlefield. **Unreachable rather than wrong**: no caller produces a
    multi-entry batch (`Primitive::ReturnToBattlefield` is a stub;
    `CreateToken` loops `propose_entry`). The fix is structural — decide the
    entry in phase 1 beside its zone change, perform it in phase 2 — and it is
    the same restructuring CR 613.7m needs ("Before card breadth" item 4), so
    the two are scheduled together as RC-5 part 2 (`replacement-architecture.md`
    §9). **Sized:** ~400 additions in `execute_batch_inner` and
    `perform_action`'s `ZoneChange` arm. The entry hop ("Before Triggered
    abilities" item 4) was the same restructuring seen from the log's side,
    and **RC-4b closed it (2026-09-02)**: an entry is a phase-1 proposal that
    carries `from`, so a multi-entry batch would now decide every member
    against the pre-batch board. What is left here is producing one —
    `CreateToken` still loops `propose_entry` one token per batch — and
    CR 613.7m's APNAP timestamps.

    **Re-read against the tree 2026-09-03 (RC-5's re-size), and the two halves
    split.** The frame half is *done*, not merely designed: phase 1 decides
    every member before phase 2 performs any, `EntryFrame::new` is built from
    the proposal inside phase 1, and a `ZoneChange { to: Battlefield }`
    proposal is a debug assertion — so §5b's two Master Biomancers already give
    each other nothing. RC-5 proves it at the `execute_actions` boundary
    (`test_two_biomancers_entering_together_give_each_other_nothing`) and does
    not pretend that proves the pool. **The producer is the whole of what is
    left, and it is bigger than it looks**: `Primitive::ReturnToBattlefield` is
    the natural one, and it needs a graveyard leaf on `SelectionFilter` (which
    enumerates only battlefield, stack and players) plus item 48's `controller`
    field, because a mass return is exactly item 48's wrong fourth road.
    **Sized:** ~350 and a card. It is what makes CR 614.13a's second clause —
    "nor any other object entering the battlefield at the same time" — and the
    batch-scoped frame reachable from a game rather than from a test. CR 613.7m
    is *not* part of this any more; see item 4 under "Before card breadth".

    **`Primitive::CreateToken` is the cheaper producer and it should stop
    looping** (asked on review 2026-09-03). "Create three 1/1 Soldiers" is one
    event by CR 111's own shape, and the loop makes it three — three batches,
    three CR 616.1 passes, three chances for an entry replacement to see a token
    the others just made. Phase RE's `GameAction::CreateTokens` is where that
    stops, and it arrives there for CR 614.16's doublers anyway, so the fix is
    free at the point of use rather than a job of its own. **It is also the
    cheaper route to a multi-entry batch than a mass return**: no graveyard leaf,
    no item 48 controller field, and the pool already makes tokens (Kalitas's
    rider). Whoever builds RE should expect it to close this item and 613.7m
    together. **On storage:** a token is a full `GameObject` today, and a wide
    board of them is the shape simulators historically bog down on. Nothing has
    measured it here — `fuzz_games` makes few tokens — so it is not a claim, but
    a batched creation is the prerequisite for ever storing them any other way,
    because a per-token loop hard-codes one object per token at the *proposal*.

    **Reachability (2026-09-03):** unreachable — still no producer of a
    multi-entry batch: `ReturnToBattlefield` is the stub arm (`resolve.rs:868`),
    `CreateToken` loops `propose_entry` (`resolve.rs:630`), and Kalitas makes
    one token per death.

    **Owner (2026-09-11):** RE-4 — `replacement-architecture.md` §9, RE
    decision 3; Raise the Alarm and Hordeling Outburst are the first plural
    creations, and CR 613.7m's prompt is *not* asked for a homogeneous batch.

47. **`pipeline::ordering_cannot_change_outcome` is a semantics-assuming
    shortcut, and these are its expiry conditions.** (Named
    `order_invariant_entry_bucket` until RD-2; item 65.) It skips CR 616.1's prompt when every
    member of the bucket is an `EnterWith` whose applicability no `EnterMods`
    field can move, which is true today because the only characteristic the
    mods feed is power (through `+1/+1` and `-1/-1` counters, CR 122.1a) and
    the only leaf that reads power is `ObjectFilter::PowerLE`, which the
    predicate excludes. It goes false, silently, the day any of these lands:
    (a) `EnterMods` gains a field that feeds a characteristic — **face-down**,
    which is Layer 1 and changes everything (Phase CV); (b) `ObjectFilter`
    gains a leaf that reads power, toughness, keywords or counters
    (`ToughnessLE`, `HasKeyword`, `HasCounter` — RS-2/RS-3 candidates); (c)
    `EventPattern::EnterBattlefield` gains a field that reads `mods`.
    `filter_is_mods_invariant` is matched exhaustively, so (b) is a compile
    error rather than a silent default; (a) and (c) are not, and
    `check_order_invariance` is the debug-build check that computes the theorem
    the other way — re-gather after the suppressed choice and assert the rest
    still apply — which catches either on any board a test or a debug fuzz run
    reaches. **The rule for whoever adds (a) or (c): revisit the predicate in
    the same commit.** **A fourth condition arrived with RC-5 and fired
    immediately** — an `EnterModsTemplate` amount that reads the CR 614.12 frame
    does not commute, so the predicate now asks for `Fixed` or a source that is
    not the entering object. The rule was followed: see item 58.

    **A second bucket shape arrived with RD-2 (2026-09-09), by decision rather
    than by accident** (`replacement-architecture.md` §11 item 29). The
    predicate — renamed `ordering_cannot_change_outcome`, item 65 — now also
    admits a bucket that is entirely `Amount(Multiplier(n ≥ 1))` on
    `EventPattern::DealDamage`, under the same shared clauses. It goes false
    the day (d) an `EventPattern::DealDamage` field reads the *amount* — or
    (e) a `Multiplier(0)` is printed, which the `n ≥ 1` clause refuses rather
    than defaults on. The debug re-gather checks per group member since RD-2's
    group form, so (d) is caught on any board a debug run reaches.

    **(d) re-derived at RD-3 (2026-09-09), which added the arm's first two
    fields, and the suppression stands.** `source` is CR 609.7's predicate over
    the object *dealing* the damage and `combat` is CR 510.2's flag on the
    proposal; neither reads the amount, so no member of a multiplier bucket can
    fall out of applicability as another changes the number. Item 102 is the
    entry; the rule this item states for whoever adds such a field — revisit
    the predicate in the same commit — was followed.

    **Reachability (2026-09-03):** nothing owed — expiry conditions for a
    predicate; the rule is "revisit in the same commit".

    **Sized:** none.

48. **`default_enter_controller` has three roads and answers a fourth wrongly
    — and RC-4 stopped standing on it.** The three that exist are exact: a
    resolving permanent spell (`GameState::resolving`, CR 110.2b's default),
    a land drop (owner), a token (owner, CR 111.2). The fourth is an effect
    putting a card onto the battlefield *under a player's control* who is not
    its owner — Reanimate's "put target creature card from a graveyard onto
    the battlefield under your control" — where owner is wrong and nothing in
    the proposal says otherwise. No registered effect takes that road
    (`Primitive::ReturnToBattlefield` is a stub). **Sized:** the mover has to
    say under whose control — a `controller: Option<PlayerId>` on the
    `ZoneChange` proposal, or a `propose_entry` argument the `Returned` arm
    threads — one field, read in one place. The `base_controller` `resolving`
    leg RC-3 added was re-checked for this phase as the brief asked: it is
    consulted for an entering object only by the finished-board
    `object_matches_filter`, which RC-4 no longer uses for an entry (the
    frame seeds its controller from the proposal), so the frame does not stand
    on it and it stays as RC-3 left it — right for the three roads, inert in a
    game.

    **Reachability (2026-09-03):** unreachable — `default_enter_controller`
    (`game_state.rs:738`) still answers resolving-or-owner, and no registered
    effect puts a card onto the battlefield under a non-owner's control.

49. **`is_prohibited` has no source-1a leg.** `gather` asks the entering
    permanent itself for its `SourceOnly` replacement abilities ahead of the
    battlefield sweep, because `replacement_ability_sources` is written by the
    performer; the restriction sweep has no twin, so an entering Tatterkite's
    "this creature can't have counters put on it" (Melira's Keepers has the same
    sentence) is invisible to an entry that would give it counters. CR 614.17d's
    parenthesis licenses the leg. No registered effect gives an entering
    permanent counters *from outside* — Master Biomancer is RC-5's — so there is
    no board to fail on yet; add the leg with the first such card, mirroring
    `gather`'s `SelfScope::EnteringSelf`, ~20 lines. **Master Biomancer landed
    2026-09-03 and the leg is still owed — the board now exists.** An entering
    Tatterkite under a Biomancer should get no counters and would get two:
    `strip_prohibited_counters` asks `is_prohibited`, which still has no
    source-1a sweep. Neither Tatterkite nor Melira's Keepers is registered, so
    nothing fails; this is the first entry on this list whose *reproducer* is
    now one card away rather than two.

    **Reachability (2026-09-03):** unreachable, one card away — Master Biomancer
    is registered; Tatterkite and Melira's Keepers are not, and no other
    registered permanent forbids counters on itself.

    **Sized:** ~20 lines mirroring `gather`'s
    `SelfScope::EnteringSelf` in `restriction/predicate.rs`, with the first such
    card.

50. **A count enumerates the battlefield twice per CDA.** `SetPowerToughness`
    evaluates its two amounts separately, so Keldon Warlord's `CountOf` sorts
    `battlefield_ids_ordered` and matches every permanent twice per layer-7a
    application — the second pass hits the frame cache for every frame but
    repeats the sort and the filter walk. Not measured to matter (see the RC-4
    block above); recorded so that whoever sees `Frames/walk` climb on a
    CDA-heavy board knows the factor of two is here and not in the cache.

    **Reachability (2026-09-03):** reachable — not wrong; perf only (Keldon
    Warlord is in `PERFORMANCE_POOL`; the factor of two sits inside
    `Frames/walk`, not the cache).

    **Sized:** evaluate a CDA's two amounts in one pass, or memoize
    `CountOf` per filter within a walk, ~30 lines in `cda.rs`/`compute.rs`; only
    when a CDA-heavy board measures it.

    **Closed 2026-09-13 (RE-4).** `GameAction::CreateTokens`' performer
    proposes every token's entry as one `execute_actions` — the producer this
    item was waiting for — and Raise the Alarm in `PERFORMANCE_POOL` makes it
    a batch a measured game builds. Two tests reach the frame half from a
    printed card: `two_soldiers_under_master_biomancer_each_get_its_counters`
    and `two_biomancer_tokens_entering_together_give_each_other_nothing`, the
    RC-5 board through a token def that carries Biomancer's ability. The mass
    return (`Primitive::ReturnToBattlefield`) is still a stub and is no longer
    what this item is about; CR 613.7m is "Before card breadth" item 4's, and
    stays not asked (`replacement-architecture.md` §9, RE decision 3).

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


## Found by RC-4b — entering is one event (2026-09-02)

52. **A token whose entry is exiled instead records `from: Battlefield`.** A
    token is created in `Zone::Battlefield` with no entity and in no
    collection until its entry is decided (`Primitive::CreateToken`), and its
    `EnterBattlefield` carries `from: None`. `Instead(ZoneChangeTo)` on it is
    performed as `ZoneChange { from: Battlefield, to }` with no LKI, so the
    log says the token left the battlefield where CR 111 says it was created
    in exile (Hallowed Moonlight's ruling); CR 704.5d then removes it. A
    *dropped* token entry un-creates the object (CR 111.5). **Unreachable
    today**: Containment Priest excludes tokens and no registered card is
    Hallowed Moonlight. The honest fix is Phase RE's `CreateTokens` proposal
    (CR 614.16's doublers need it anyway), where the creation is the event and
    the entry's decision sets its destination —
    `replacement-architecture.md` §9, RC-4b's token decision. **Sized:** the
    `CreateTokens` arm of `GameAction` and `EventPattern`, ~150, inside RE.
    **Not optional before Phase 8 (owner, RC-4b review):** Dour Port-Mage
    ("Whenever one or more other creatures you control leave the battlefield
    without dying, draw a card.") and Aang, Airbending Master ("... you get an
    experience counter.") are the matcher that reads this line — a
    leaves-the-battlefield trigger keyed on `ZoneChange { from: Battlefield }`
    — and both would fire for a token that was created in exile and never left
    anything. Cross-listed as "Before card breadth" item 8.

    **Reachability (2026-09-03):** unreachable — re-checked: Containment Priest
    excludes tokens, no other registered replacement acts on an *entering* token
    (Rest in Peace, Leyline and Kalitas act on graveyard-bound moves), and
    Hallowed Moonlight is not registered.

    **Owner (2026-09-11):** RE-4 — `replacement-architecture.md` §9, RE
    decision 3: a `from`-less `CreateTokenIn` variant rather than an `Option`
    on `ZoneChange.from`, with Hallowed Moonlight registered as the consumer.

    **Closed 2026-09-13 (RE-4).** `pipeline::substitute` returns
    `GameAction::CreateTokenIn { object, zone }` for an `Instead(ZoneChangeTo)`
    on an entry with `from: None`; its performer, `GameState::put_token_into`,
    adds the token to the zone's collection and stamps the epoch CR 704.5d's
    sweep orders by, and the arm announces `GameEvent::TokenCreated { Exile }`.
    No `ZoneChange` is emitted for the token at all. Hallowed Moonlight is
    registered and is the consumer; the probe that showed the pre-fix log
    saying `from: Battlefield` is the test's own assertion
    (`hallowed_moonlight_creates_the_token_in_exile_and_it_ceases_to_exist`),
    and RC-4b's test that asserted the cheap answer by name now asserts the
    honest one.

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


## Found by the RD-1 review (2026-09-08)

*Item 88 evicted 2026-09-15 by the post-RE audit's close-out. It had been
fixed the day it was found; its verdict never said "closed", so the board
counted it as an unchecked claim for a week.*

88. **A mill of N was N batches, and it should have been one (fixed in the
    same review).** `Primitive::Mill` looped `change_zone`, so each card's move
    opened its own batch. CR 701.17a says "that player puts **that many cards**
    from the top of their library into their graveyard" — one simultaneous
    move — and the CR has no analogue here to CR 121.2's "cards may only be
    drawn one at a time", which is the rule that makes *drawing* the exception.
    The consequence is CR 603.2c's: "whenever one or more cards are put into
    your graveyard" would have fired once per card. Now one `execute_actions`
    batch of N `ZoneChange` members, which keeps each card its own event for
    CR 614.5 (Leyline of the Void applies to every card, not the first) while
    giving the whole mill one `BatchId`.

    **Reachability (2026-09-08):** it was unreachable as a *wrong answer* —
    no trigger exists — and reachable as a wrong *shape*, which is why it was
    fixed rather than deferred: item 6 would have inherited it silently.
    Pinned by `a_mill_is_one_batch_of_many_moves`.

    **Reachability (2026-09-15):** closed — the verdict above, re-worded so
    the board reads it.

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


## Found by RD-4 — redirection and unpreventable damage (2026-09-09)

108. **A player who has left the game keeps their permanents, and CR 800.4a
     says they should not.** `GameState::player_lost[p]` is set by the SBAs and
     read by `Game::check_game_over`, by `entering_controller`'s opponent list
     and now by CR 614.9's re-check. Nothing removes that player's objects from
     the battlefield, their cards from their zones, or their spells from the
     stack, which CR 800.4a requires.

     **Reachability (2026-09-09):** unreachable in a two-player game, where the
     loss ends the game in the same SBA sweep. Reachable the moment a game has
     three or more players — which is v1's target, not a corner
     (`v1-is-commander-and-parallel-ai`). `fuzz_games` plays two.

     **Sized:** CR 800.4a is a list of six clauses over five zones plus the
     stack plus control-change effects, each proposing through `change_zone`;
     it is the multiplayer phase's, not a patch. Phase 9 (`roadmap-v2.md`,
     formats and multiplayer) owns it. RD-4's own test builds the state by
     hand and says so.

     **Owner (2026-09-11, re-cut on review): RE-7** — `replacement-architecture.md`
     §9, RE decision 5 — immediately after RE-6, which builds the `PlayerLoses`
     performer this hangs from, the rotation half (800.4j/k, item 113) and the
     `--players 4` fuzz mode. The first cut left this with B3; the review's
     objection stands: the day RE-6 lands this is *reachable and wrong* in
     the four-player run, and the ledger's rule is that a reachable wrong answer
     is fixed first. CR 802's defending player stays "Before Commander" item 4's.

     **Reachability (2026-09-12, RE-6 landed): reachable, wrong today, and
     measured.** `fuzz_games --players 4` plays it in every game that has a
     departure before the end, and the harness prints the wrong answer as a
     row: **"Departed-owned permanents"** is the count of battlefield
     permanents a player who has left still owns when the game ends, and
     **"Turns after a departure"** is how long they stayed. Both are in
     `engineering-practices.md` §3's four-player table, recorded as RE-7's
     starting point; RE-7 zeroes the first. The two-player verdict above
     stands unchanged: a loss there ends the game in the same sweep. **Owner
     unchanged: RE-7**, and the closer is the one named there — CR 800.4a
     inside the `PlayerLoses` performer.

     **Reachability (2026-09-13): closed — RE-7.** CR 800.4a's four clauses run
     inside the `PlayerLoses` performer and are gated on CR 800.1's seat count,
     so a two-player game is untouched to the byte and a game that began with
     three or more takes the rule. The four-player `performance` run's
     "Departed-owned permanents" row went 32.7 → 0.0 and `stress`'s 32.8 → 0.0
     (`engineering-practices.md` §3). The closer is the one the entry named.


## Found by RE's sizing (2026-09-11)

*Item 118 evicted 2026-09-15 by the post-RE audit's close-out, which fixed
it: one proposal inside the 514.3a loop, shown to fail first.*

118. **CR 514.3a's repeated cleanup step announces nothing.** RE-1 made a
     step's beginning an event, and `Game::run_turn`'s 514.3a loop — "if
     state-based actions are performed during the cleanup step, ... another
     cleanup step begins" — re-runs `perform_cleanup_actions` and a priority
     round without proposing a second `GameAction::BeginStep { Cleanup }`. So
     the log shows one cleanup step where the rules had two, and a skip that
     should meet the second occurrence meets nothing. Pre-existing in shape —
     the loop has always re-run without a transition — and newly *visible*,
     which is why it is recorded now rather than earlier.

     **Reachability (2026-09-11):** reachable but not wrong today — nothing
     triggers at cleanup (item 6's), and no printed card skips a cleanup step,
     so the only reader of the missing event is the event log itself. It
     becomes wrong the day either lands.

     **Sized:** one `begin_step` call inside the 514.3a loop, ~10 lines, plus
     the test that the log holds two `StepBegin { Cleanup }` when SBAs fire
     during the first. The care is that CR 614.10's skips are per *occurrence*,
     so the second cleanup step is genuinely skippable and must be proposed
     rather than assumed.

     **Reachability (2026-09-15):** closed — the fix above, in the close-out
     PR of `plans/handoffs/post-re-audit.md`; no pooled game reaches a
     repeated cleanup step, so no table moved.

## Found by the fork-and-search question (2026-09-01)

43. **CR 122.6a names a player and `EnterMods` does not carry one (recorded
    2026-09-01, RC-2) — ✅ CLOSED 2026-09-13 (RE-5).** "If an object enters the battlefield with counters on
    it, the effect causing the object to be given counters **may specify which
    player puts those counters on it**. If the effect doesn't specify a player,
    the object's controller puts them on." `EnterMods.counters` is
    `Vec<(CounterType, u32)>`, so only the default half exists.

    **Nothing in reach needs the named half** — no registered card specifies a
    player, and ATOM-122.6a-001 is covered by the default. What needs it is
    Phase **RE**: the atom's own expected result says so, because Doubling
    Season doubles counters *you* put on, so a doubler has to know who put them
    on before it can decide whether it applies. The field is one `Option<PlayerId>`
    per entry and the merge already coalesces by kind, which is the thing that
    would have to change — two effects giving counters of the same kind on
    behalf of different players cannot share a row. **Size it before RE writes
    its first doubler, not after.**

    **Reachability (2026-09-03):** unreachable — no registered effect names the
    player who puts the counters; `EnterMods.counters` is still
    `Vec<(CounterType, u32)>`.

    **Sized:** an `Option<PlayerId>` per entry on both `EnterMods`
    and `EnterModsTemplate`, with `merge` keyed on `(kind, player)` instead of
    `kind`, ~60–80 lines plus tests; the first commit of RE's doubler, before
    the doubler is written.

    **Owner (2026-09-11):** RE-5 — `replacement-architecture.md` §9, RE
    decision 4; Vorinclex, Monstrous Raider is the field's first reader.

    **Closed (2026-09-13, RE-5):** the rules pass ran before the field was
    written. CR 122.6a's first sentence — an effect that "may specify which
    player puts those counters on it" — has no printed customer: three
    Scryfall queries return nothing, and the seven printed "would put one or
    more counters" watchers all read the default. The premise that Doubling
    Season "doubles counters *you* put on" was wrong about the card, whose
    counter half reads "a permanent you control"; Vorinclex, Monstrous Raider
    is the reader, and what it reads at an entry is CR 122.6a's default off
    the entry's `controller`, which CR 616.1b settles ahead of anything that
    asks. The `Option<PlayerId>` per counters entry and the `(kind, player)`
    merge key are recorded on `EnterMods::counters` for the card that prints
    one. `replacement-architecture.md` §11 item 83.

    **Reopened and built (2026-09-14, RE-5's review, theme A).** The owner
    rejected the close: a rule the CR states is owed whether or not a card
    prints it, since a card can be printed next set and custom card creation
    is a post-v1 goal — the CR is the customer, a printed card is the test
    (`engineering-practices.md` §4). Bold Plagiarist shows the shape on a
    proposal ("*they* put the same number and kind of counters on this
    creature": the opponent puts counters on a creature they do not
    control), which RE-5 had not looked for. Built as sized: `EntryCounters
    { counter, n, by: Option<PlayerId> }` and `EntryCountersTemplate { ..,
    by: Option<PlayerRef> }`, `merge` keyed on `(kind, putter)`, the door and
    the CR 101.2 check reading each row's putter ahead of the entry's
    controller, `pipeline::putter_of` resolving a template's `PlayerRef`, and
    `by: Option<PlayerRef>` on `Primitive::AddCounters` and `GetCounters`
    through `resolve_putter`. Three fixture tests; no printed producer.

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


### Item 9 — closed 2026-09-14 by LJ

**What closed it.** `ObjectSet::Filter` carries a `ZoneSet`, `Board::seed`
admits the objects a zone-reaching row names, and `RegistryScopeSummary::
reachable_zones` keeps that free on every board that plays no such card.
Yixlid Jailer is the first consumer and is in `PERFORMANCE_POOL`;
ATOM-614.12-001 is covered. The design, the five decisions and the
source/affected split are `layers-architecture.md` §13c.

**Three things this item said that the tree did not**, all corrected in the
closing PR rather than left: `effect_applies_to` had not existed since RC-3
(the gate was `Board::in_battlefield_zone_or_entering`, and is now
`in_zones_or_entering`); "one loop over a zone list" understated it, because
the filter's candidates come from `Board::members` and a graveyard card was
not one, making this a working-set change rather than a predicate change; and
`RegistryScopeSummary` had eight fields, not the three §5.1 still described.

**What was owed and went elsewhere.** CR 613.7d's object timestamps — this
item's second listed piece — are not needed by the filter half at all, because
CR 613.7 orders *effects* and a row's timestamp is read off its **source**.
Yixlid Jailer's source is on the battlefield. The timestamp is the
*source-side* facility, which is CR 113.6 / A5's, and Wonder needs both
together.

The original entry, as it stood on 2026-09-04:

9. **Abilities granted to cards outside the battlefield — ❌ inexpressible.** The layer system can only apply filter-based effects to objects in the battlefield *zone*: `effect_applies_to`'s gate is `in_battlefield_zone_or_entering` (`engine/layers/compute.rs`, since RC-3), and the filter type is `ObjectFilter`. So a whole class of real cards has no representation — Yawgmoth's Will and Underworld Breach (flashback on graveyard cards), Aminatou, Veil Piercer ("Each enchantment card in your hand has miracle"), Future Sight and Bolas's Citadel (playing off the library), foretell-style grants on face-down exile.

   Two pieces are needed, in this order:
   - **A card filter and a zone-aware `ObjectSet`,** so the effect can say which zone it reaches. This is the actual blocker; it is a type change, not a tuning problem.
   - **Timestamps must move off `PermanentState` and onto the object.** CR 613.7d gives an object a timestamp when it enters *any* zone; we store one only on `PermanentState`. Wonder ("as long as this card is in your graveyard and you control an Island, creatures you control have flying" — a static ability functioning from the graveyard, CR 113.6b) has nowhere to read one from, so `GameState::static_effect_timestamp` has no answer for it. Its `None` arm is unreachable today only because `register_static_effects` is called from `place_on_battlefield`.

   - **A `reachable_zones` bitmask on `ContinuousEffectRegistry`,** maintained on add/remove. **This now has a home:** `RegistryScopeSummary` exists (`state/continuous_effects.rs`), recomputed on every `add`/`remove`, carrying the one field the CR 604.2 existence check needed. `layers-architecture.md` §5.1 already specifies `touches_hidden_zones` / `touches_stack` / `has_active_cdas` on that same struct — extend it rather than adding a parallel counter. `compute_characteristics` checks the object's zone against it and returns base characteristics on a miss. This keeps the cost at zero until someone actually plays a zone-reaching card, and even then confines it to the one zone that card reaches — queried on demand at castability-check time, never as an eager sweep over every card in the game.

   **Narrowed by the CDA phase (2026-08-22).** This item once carried CR 604.3's "CDAs function in all zones" as well. It doesn't: a CDA has no filter (CR 604.3a(3)), so it never needed a zone-aware `ObjectSet`, and it now works in every zone via the intrinsic pass. What remains here is the original thing — *filter-based* effects reaching other zones. Note for whoever builds the `reachable_zones` fast path: it must not early-out an object that has a CDA of its own, which is why `apply_effects`' existing fast path already has a third term.

   The mask also generalizes the existing fast path, which today early-outs only when the registry is *entirely* empty: with it, a card in hand early-outs even with many battlefield effects registered. Worth building **with** the first zone-reaching card, not before — there is nothing to test against otherwise. Note that Aminatou additionally needs item 3 (the cost-modification pipeline) for "its miracle cost is equal to its mana cost reduced by {4}".

   **Reachability (2026-09-04):** unreachable — no registered card grants an
   ability to a card outside the battlefield; Leyline of the Void's opening-hand
   clause is left off the registered card (`phase_rb_cards.rs`), which is the
   narrower-card precedent, not a wrong answer. Re-derived with the breadth,
   since "niche" was the risk: Scryfall gives 24 graveyard-side "has/have"
   statics, 103 static cast-from-graveyard permissions, 25 "spells you cast
   have" grants reaching the stack and 3 "cards in your hand have" — with the
   replacement doc's ~390 sources outside the battlefield, a few hundred
   cards.

   **Sized:** a zone-aware `ObjectSet` with a card filter, CR
   613.7d timestamps on `GameObject`, and `reachable_zones` on
   `RegistryScopeSummary`: ~400–600 lines; with the first zone-reaching card,
   and `replacement-architecture.md` §3.3 source 2 (Leyline's clause) rides the
   same change.

   **Three corrections (2026-09-04, after LH-1's review).**
   1) *One filter type, not two.* "A card filter" above assumed a second type
   beside the characteristic filter (then `PermanentFilter`). CR 108.4a — a
   card that has no controller uses its owner wherever a controller is asked
   for — means every existing leaf, `ByController` included, reads correctly
   off a card in hand, so the shape is one `ObjectFilter` with a zone leaf and
   `CardFilter` (three variants, five uses, `Condition::CardInGraveyard`) folded
   in. **The rename landed 2026-09-07 as CM-0** (`cost-architecture.md` §3.2,
   the first consumer that applies the filter to a spell): 275 sites in 25
   `src/` files, 119 in 18 test files, the live plan docs, and the three
   `permanent_matches_filter*` matchers became `object_matches_filter*`. The
   zone leaf and the `CardFilter` fold are still this item's. ATOM-614.12-001 (Yixlid Jailer,
   "Cards in graveyards lose all abilities") is the atom the zone leaf unblocks,
   and that card is also the right **first consumer**: its source is on the
   battlefield, so it needs nothing from CR 113.6 (A5) — only the zone-scoped
   filter and a `LoseAllAbilities` row reaching a graveyard. Wonder needs A5 and
   the `Condition` AST (7f) as well; it is the second card, not the first.
   2) *Aminatou is four systems deep, and this item is only the first.*
   Verified text (Scryfall, 2026-09-04): "Each enchantment card in your hand
   has miracle. Its miracle cost is equal to its mana cost reduced by {4}." It
   needs this item (a Layer 6 grant reaching hand, filtered by card type), A5's
   CR 113.6 so miracle functions from hand, miracle itself (CR 702.94, a static
   linked to a draw trigger — item 6, and RE's post-replacement draw stream),
   and cost modification for the reduction (item 3, `backlog.md` §2.1). A
   Phase 8 card, not this item's consumer.
   3) The sentence at the top of this item was stale since RC-3 and is
   corrected above: the gate is the battlefield *zone*, not `game.battlefield`
   membership.

## Before card breadth (Phase 8) — added by the RD-2 review (2026-09-09)

8. **A token created in exile instead logs `from: Battlefield` — RC-4b's cheap token answer, item 52 (recorded 2026-09-02).** Dour Port-Mage and Aang, Airbending Master — "leave the battlefield without dying" — read exactly that line and would draw a card or grant an experience counter for a token Hallowed Moonlight created in exile. The fix is Phase RE's `CreateTokens` proposal, whose destination the entry's decision sets, and it lands before any pool pairs a token-exiling replacement with a leaves-without-dying trigger. A hard back-stop, not an RE nicety.

   **Reachability (2026-09-03):** unreachable — as main item 52, re-checked
   there.

   **Sized:** with item 52, ~150 lines inside RE — **RE-4** as of 2026-09-11
   (`replacement-architecture.md` §9, RE decision 3), with Hallowed Moonlight
   registered there as the card that reaches it.

   **Closed 2026-09-13 (RE-4)** — with main item 52; the line no longer
   exists to be read.

11. **The codebase has enough invented vocabulary to need a glossary, and
    nothing defines the words in one place.** Reported on the RD-2 review, on
    "subject group" — a term RD-2 introduced, defined in a doc comment on
    `Member` in `pipeline.rs`, used in three plans and in two commit messages.
    It is not alone: *frame*, *rider*, *instance* vs *member* vs *candidate*,
    *applied set*, *bucket* (which RD-2 deleted for exactly this reason), the
    three senses of *shield*, *pool* (card pool) vs *pool* (mana pool),
    *chokepoint*, *gate leg*, *arm*. A reader meets each of them in whichever
    file happens to introduce it, and the definition is wherever the phase
    that coined it put it.

    **Reachability (2026-09-09):** reachable and costing time now — the review
    that produced this item asked what two of these words meant, and both were
    defined only in a doc comment inside the module that uses them.

    **Sized:** ~150–200 lines, one PR of its own. Two constraints from the
    scars this file already records: (a) it goes in `README.md`, not
    `CLAUDE.md` — the budget there is 8 lines and a glossary is not an
    invariant; and (b) **it needs an anti-rot check or it is item 89 waiting
    to happen** (a comment stating a fact, going stale, with nothing re-reading
    it). The check is the cheap kind trace tier 3 already wants: a script that
    asserts every glossary term still appears in `mtgsim/src`, and that every
    word in a short watch-list (the ones above) appears in the glossary — so a
    rename breaks CI in the same commit, which is how `must_choose_among`
    would have been caught. `check_claude_md.py` and `check_module_layout.py`
    are the template.

    **Closed 2026-09-11 by the glossary pass** (`plans/glossary.md`,
    `plans/check_glossary.py`, PR #123). 31 terms, seven of them carrying more
    than one meaning. Three things the item did not predict:

    - **The check wants three assertions, not two.** The RD-4 review supplied
      the third: *a word with more than one meaning carries all its senses,
      numbered*. One sense per word is what let `Rewrite::Retarget`'s arms reach
      the build as `ToSource` and `ToSourceController` — two different sources,
      adjacent in one enum — and both had to be renamed mid-PR. `POLYSEMOUS` in
      the script is the list, and a new collision is a line there and a numbered
      sense in the doc, together.
    - **The seed watch-list was wrong in both directions.** *gate leg* is not a
      phrase anybody writes — *gate* and *leg* are separate words and both are
      used constantly — so it was split. *census* turned out polysemous and was
      not on the list at all (a Scryfall card census, and a call-site census).
      *bucket*, which this item recorded as deleted, is alive: it is a recipient
      slot in an `allocate` decision, and only its choice-ladder sense went
      away. The list is the check's input, so a word on it that nobody uses is a
      false alarm forever; it was pruned and extended against the tree.
    - **Writing the entries found five stale pointers to one name, and fixed
      them here.** `engine::replacement::is_blocked` was real: RB's CR 614.17
      check, born 42e0516 and deleted by RS-1 (68bfdad) when the "can't" spine
      became `engine::restriction::is_prohibited`. Six references outlived it.
      Four were live pointers and now name `is_prohibited` — `CLAUDE.md`,
      `engine/actions.rs`, `engine/resolve.rs` and `pipeline.rs`'s bare use.
      Two are dated history and were dated rather than rewritten:
      `cant-effects-architecture.md` §1, which argues about what RB built, and
      the live file's "Status 2026-08-26: Phase RB ✅" block, where the name was
      correct when written. It was carried as a deferred item for about an hour
      before the owner called it correctly: a six-line doc correction inside a
      PR whose subject is *what the words mean* is not a rename, and filing it
      as its own PR is how the backlog bloats.

      Two lessons, and the second is sharper. The first draft of this record
      said the name had "never existed" — nobody had run `git log -S`, and the
      pointer being *wrong* is a different claim from the pointer being *stale*.
      The second: `check_glossary.py` cannot catch this class. `is_blocked`
      still resolves as a word, because combat has an `AttackingInfo` field by
      that name, so assertion 1 passes on a pointer aimed at the wrong
      subsystem. The docstring says so rather than papering over it.

    **Two decisions the PR owed, both recorded here.**

    1. **Where it lives: `plans/glossary.md`, not `README.md` — this item's
       answer overruled.** README is 299 lines and is the project's front door;
       a 200-line glossary makes it 40% glossary and buries Getting started
       under vocabulary. The audience is also wrong for it: the glossary defines
       `EntryFrame`, `must_choose_among` and `gather`'s legs, which is a reader
       already inside the code, while README's reader is deciding whether to go
       in. Every other authority in this project lives under `plans/` and is
       reached from the Documentation map, and that indirection already exists
       for exactly this. README gains the map row, a note that the file is
       checked, and a Contributing line — 7 lines, not 200.
    2. **A word the pass found genuinely ambiguous gets defined, not renamed —
       and a stale pointer is not a rename.** No word came up needing a rename;
       `is_blocked` is the case that tested the line, and the owner drew it in
       the right place. Renaming a live symbol carries a diff a reviewer cannot
       check without re-deriving the definition, so it stays out. Correcting a
       pointer to a symbol deleted five commits ago carries no such diff — it is
       six lines, it belongs to whoever noticed, and deferring it to "its own
       PR" is how a backlog grows entries nobody will ever pick up. The rule is
       about the *reviewability* of the change, not about which file it touches.

    **What the glossary deliberately does not do.** It does not restate. Each
    entry says what the word means and names the file or doc section that owns
    the reasoning, the same budget `CLAUDE.md` keeps for invariants — an entry
    that re-argues §4.1 is a second copy, and the second copy is the one that
    goes stale. The one block that *moved* rather than being summarized is
    `replacement-architecture.md` §9's "Three things the CR calls a shield",
    which was vocabulary sitting inside a design section and findable only by
    someone reading RD; §9 keeps the `Uses::NextDamage` naming argument, which
    is design, and points at the glossary for the disambiguation.

    **Reachability (2026-09-11):** closed — the glossary pass, PR #123.


## Before Triggered abilities (CR 603)

4. **~~The entry hop: Containment Priest's substitute leaves a permanent's worth of zone changes in the log for a card the CR says never entered~~ — ✅ CLOSED 2026-09-02 (RC-4b).** Entering is one proposal: `GameAction::EnterBattlefield` carries `from`, `change_zone` routes a battlefield destination to it, and its performer moves the card, announces the zone change, builds the entity and announces the entry. The Priest's substitute is one `ZoneChange { Graveyard → Exile }` with no LKI and one epoch, a dropped entry leaves the card where it was, and a "can't enter" may watch the entry (`phase_rc4b_integration_test`; `replacement-architecture.md` §9, RC-4b). The token residual is item 52. The record as found (2026-09-02, RC-4; sharpened in review): The `ZoneChange` performer moves the card into the battlefield zone and *then* proposes the `EnterBattlefield` (RC-2's one-`emit`-wide window), so "exile it instead" is performed as a `ZoneChange { from: Battlefield, to: Exile }`. Three things observe that: (a) the log holds a `ZoneChange` *into* the battlefield, so an ETB matcher on the zone change would fire — it must key on `PermanentEnteredBattlefield`, the performer's event, which a permanent that never entered does not have; (b) the log holds a `ZoneChange` *out of* it, `from: Battlefield` with a CR 603.10a LKI frame, so a leaves-the-battlefield or "exiled from the battlefield" matcher would fire, and a "leaves your graveyard" matcher would not, because the recorded `from` is wrong; (c) `zone_change_epoch` advances twice, so CR 400.7 sees two new objects. None is reachable today — no trigger matcher exists — but (b) and (c) have no keying rule that fixes them, so this is a bug-in-waiting for item 6, not a convention. **The fix is to reverse the nesting**, and it is the same restructuring as Deferred Migrations item 46: `GameAction::EnterBattlefield` carries `from`, its performer does the move, the placement and both emissions, and the `ZoneChange { to: Battlefield }` arm forwards to it *before* moving anything. Then the Priest's substitute is one `ZoneChange { from: <source zone>, to: Exile }`, the window is gone, `propose_entry`'s "replaced away" error is gone (a dropped entry leaves the card where it was, which is CR 614.6), a CR 614.17d "can't enter" may watch the entry, and a multi-entry batch is decided in phase 1 like any other proposal. The Priest stays in Root Maze's CR 616.1 bucket, which is what §11 item 19 needs reachable — moving the Priest to the zone change instead would split that bucket and force Priest-first. **Sized:** ~300–500 additions in `actions.rs` (two arms, `propose_entry`), `pipeline.rs` (the `Instead` arm), the token path in `resolve.rs` (`from: None`), and the RC-4 Priest tests' log assertions; CLAUDE.md's "one emitter" line is restated to name the entry performer. **Planned as RC-4b** — `replacement-architecture.md` §9 has the design, the token and CR 608.3e decisions, and the sizing — as its own PR ahead of RC-5, which needs entries to be batch members anyway.

   **Reachability (2026-09-03):** closed — RC-4b, PR #87 (6541d0b).

5. **~~Tier 2 of the trace plan — a `TraceSink` on `GameState`, owed before the
   dispatcher~~ — ✅ CLOSED 2026-09-18 (A4c, PR #170).** What shipped: `state::trace`,
   with `TraceSink` (a `Mutex`-guarded writer behind an `Arc`, so `GameState`
   stays `Send`), `TraceHandle` (the pointer plus a branch number that rides
   the state; its hand-written `Clone` writes a `fork` record) and `Record`
   (the JSON builder). The five emit points: `execute_batch_inner` writes a
   `batch` and a `batch_end`, `apply_replacements` a `pipeline` per CR 616.1
   iteration, `compute_characteristics`, `compute_as_entering` and the LKI
   walk a `layer_walk`, the four `validate_*` helpers a `decision`, and
   `emit_event` — the one door every emitter now uses — an `event` whose text
   is `format_event`'s; the priority loop adds `priority_rejected`. Reachable
   the three ways the item asked: `fuzz_games --trace DIR [--trace-game N]`,
   `cli_play --trace PATH`, `test_support::install_trace` and
   `install_trace_file`. `plans/trace_spine.py` renders one game's lines as a
   page's spine through `plans/traces/viewer.html`, or as `--dump-events`'
   text. The record as scheduled follows. Trace pages are hand-authored today (`engineering-practices.md`
   §7): two to three hours per phase, which is the right cost at a phase's close
   and the wrong cost for a question asked mid-debugging. A sink recording what
   those pages record by hand — each proposal entering a batch, each
   `apply_replacements` iteration (candidates and their verdicts, whether the
   frame was computed, the bucket, the chooser, the choice or the suppression,
   the rewrite), each top-level layer walk with its frame count, and the
   performed events the log already holds — makes the page generated rather
   than written, and makes the same question answerable at a breakpoint. JSON
   lines, off by default, gated the way `Diagnostics` is. Emit points:
   `execute_batch_inner`, `apply_replacements`, `compute_characteristics`,
   `compute_as_entering`. Reachable three ways: `cli_play --trace`,
   `fuzz_games --trace-game N`, and a `test_support` helper so any `// COVERS:`
   test can write its own trace, which is how a page is regenerated after a
   refactor. **Scheduled 2026-09-08: its own PR, after CM-4 and before item 6**
   (`roadmap-v2.md` row A4c). It had been cargo on A6's first PR, which is the
   PR least able to carry it — the trigger phase is 4–6 PRs of new subsystem,
   and "why did this fire, or not" is a question you want answerable *before*
   starting it. Ordering-free against CM-4, which goes first because it is the
   next phase on the spine.

   **Two corrections to this item, from reading the tier-1 pages against it
   (2026-09-08, before any code):** "makes the page generated rather than
   written" cannot hold — a page's step rows are half mechanical (call site,
   `file:line`, which frame was consulted, the candidates and verdicts, the
   choice) and half authored counterfactual, and its summary tables
   ("Where the reads differ", the two-commit before/after) are entirely
   authored. The sink generates a **spine**; §7.1 needs a sentence saying tier 1
   is not subsumed. And "gated the way `Diagnostics` is" describes no
   mechanism: those are thirteen always-on `Cell<u64>`s — seven when this was
   written — free because incrementing is free, while `compute_characteristics`
   — one of the four emit points — runs ~62,000 times per measured game and
   `GameState` derives `Clone`, which the diagnostics ride deliberately and a
   growing buffer must not. What "off" costs is
   this phase's first decision, and the check is that a sink-compiled-in-but-off
   arm is `IDENTICAL` to `main` on both pools.

   **A fifth emit point, named 2026-09-16 (A4h): the decision boundary.**
   The four above are proposals, pipeline iterations and layer walks, and
   none of them is the prompt — so a sink built to this spec would not have
   found what A4h found, which was a prompt whose *option list* was wrong.
   No event log can show that one: a cast the enumeration offered and CR
   601.2g could not pay performs nothing and emits nothing, so
   `--dump-events` is blind to the re-ask by construction. What answers "why
   was I offered this" is the candidate enumeration and the list handed to
   `ask_choose_priority_action` — the result, the blacklist, the retry index.
   The instrument that did find it is `tests/priority_fork_test.rs`, which
   compares offered lists across a fork; that is an assertion, not a
   facility. The row already expects the dispatcher to add a point of its
   own, so this is a sixth rather than a re-plan.

   **The higher-value artifact item 5 does not name:** a two-version trace diff
   — one board through two engine builds, compared — which is what a human
   cannot do by hand and what `fuzz_ab.py` already does for counters. **Sized:** ~300–400 lines Rust, ~300 viewer, ~100
   script; one small phase. The seam it rides is `execute_batch_inner` and the
   entry performer, which RC-4b gave the shape they will keep.

   **Reachability (2026-09-03):** nothing owed to correctness — tooling, sized
   in the entry.

   **Reachability (2026-09-18):** closed — A4c, PR #170. The two corrections
   held: a sink generates a spine, and "off" is one branch with the payload
   behind it, `IDENTICAL` to `main` compiled in and off, and on.

2. **~~Event shape audit.~~ — ✅ CLOSED 2026-09-18 (A6 step 1, the trigger
   survey).** What closed it: `plans/references/trigger-survey.md`, with
   `plans/references/trigger-survey.py` regenerating every count — each event
   CR 603.1b–603.12a names against the corpus and the printed population
   (14,149 paper cards carry a trigger), and the printed distribution against
   the performed event that would carry it. Each of the three bullets below
   got its answer: granularity is per permanent and per occurrence, with the
   batch as the one-or-more boundary; timing is post-action, and CR 603.2g's
   prevented events never reach the stream; context is where the gaps were —
   nine, filed as "Before Triggered abilities" items 10–18. The item as
   written follows.

   Every `events.emit(...)` call site is a potential trigger source. Before wiring triggers, audit that:

   **Known missing already (found 2026-08-24, registering the first activated ability):** `GameEvent` has no variant for an activated ability being put on the stack or resolving. `put_on_stack.rs::activate_ability` pushes the ability object onto the stack without `move_object`, so not even a `ZoneChange` is emitted, and the resolution emits nothing either — an activation is completely invisible in the event log. `AbilityCountered` exists, which is the whole of the vocabulary. Triggers that watch activations ("Whenever a player activates an ability…") have nothing to watch, and the event log cannot be used to audit activation behavior at all — measuring how often Merfolk Thaumaturgist's ability resolved needed a temporary probe in `resolve.rs`. Fix as part of the event-stream refit (Replacement item 3): the fork was resolved 2026-08-24 — `AbilityActivated` plus an identity-bearing `AbilityResolved` (source + ability, for CR 603.7h counting), emitted from the chokepoint.

   - Events are emitted at the correct granularity (e.g., `PermanentEnteredBattlefield` fires per-permanent, not per-batch).
   - Event timing is post-action, not pre-action, so triggers observe the completed state change.
   - Events carry enough context for trigger predicates (controller, source, type filters).

   **Reachability (2026-09-03):** nothing owed to correctness — a checklist for
   critical-path item 6; the "known missing" half closed with RA-2
   (`AbilityActivated` and the identity-bearing `AbilityResolved`, PR #59).

   **Sized:** a half-day read of the emit sites (42 at RA's census)
   against the three bullets, inside item 6's first PR.

1. **~~Trigger dispatcher stub.~~ — ✅ CLOSED 2026-09-19 (TR-1).** The stub is `place_pending_triggers`, drained inside `perform_sba_and_triggers` in APNAP order over the seat list with CR 603.3b's two tiers; detection is `engine::triggers::dispatch`, at the close of the outermost batch and at an unbatched emission.
   The record as found: `let triggers_placed = false; // Phase 7 stub` at `engine/priority.rs:235`. This is the single-point insertion — **for placement only** (2026-08-24): detection runs synchronously at event dispatch per the resolved Replacement item 4; this stub is where the pending queue drains onto the stack in APNAP order (CR 603.3b, over the full player set).

   **Reachability (2026-09-03):** unreachable — no registered card has a
   triggered ability (`AbilityType::Triggered` appears in no card file), and the
   stub at `priority.rs:247` places nothing.

   **Sized:** unknown until the triggers architecture doc exists.
   `roadmap-v2.md` §8 gives critical-path item 6 4–6 PRs and calls it "unsized —
   size first", the dominant route risk; the layers, replacement, "can't" and
   copy tracks each got a doc before a line, and this one has none yet. Write
   the doc first.

   **Phase (2026-09-18):** TR-1 — the stub becomes `place_pending_triggers`, drained inside `perform_sba_and_triggers` in APNAP order over the seat list with CR 603.3b's two tiers; `triggers-architecture.md` §5, §12.

3. **~~LKI formalization.~~ — ✅ CLOSED 2026-09-19 (TR-1).** The reader is `engine::triggers::binding` — `bound_object`, `bound_player` and `bound_amount` over the record through the matched arm's projections, CR 603.6's "unable to be found" and CR 400.7 as one epoch comparison. The frame's *status* half is TR-4's `LastKnownInformation`; the two `sba.rs` probes were existence checks, not frames, and stay.
   The record as found: Several dies-handling sites already read `self.objects.get(&id)` *before* `move_object` to capture pre-move state (see `engine/sba.rs` dies handlers). This is ad-hoc LKI. Triggered abilities that reference "the creature that died" need a formalized `LastKnownInformation` snapshot mechanism, especially after layers land (LKI needs *post-layer* characteristics at moment-of-death, per rule 603.10 / 608.2h).

   **Reachability (2026-09-03):** unreachable — no trigger reads LKI yet. RA-3
   already captures the CR 603.10a frame on every battlefield-leaving
   `ZoneChange`, so what remains ad hoc is the three `self.objects.get(&id)`
   reads in `sba.rs` (`:356`, `:423`, `:454`).

   **Sized:** a `LastKnownInformation` reader over the event's
   `lki` frame, retiring the three ad-hoc reads, ~100–150 lines, inside
   critical-path item 6.

   **Phase (2026-09-18):** TR-1 — the reader is `engine::lki::LastKnown` over the record's frame; the two probes left in `sba.rs` (`:327`, `:425`) are existence checks ahead of a subtype read, not frames, and the item closes on the reader; `triggers-architecture.md` §3.11.

7. **~~CR 800.4d's second sentence has no site until the dispatcher exists.~~ — ✅ CLOSED 2026-09-19 (TR-1).** One `in_game` read at the head of `place_pending_triggers`, with a `pending` trace record per refusal. The four-player fixture is a Blood Artist whose controller loses in the state-based check that kills another creature — its frame sees the death, the trigger queues under the departed player, placement refuses it (`a_trigger_a_departed_player_would_control_is_not_put_on_the_stack`); ATOM-800.4d-001 is `COVERS` there. Astral Slide's delayed shape is TR-3's.
   The record as found: "If
   a triggered ability that would be controlled by a player who has left the
   game would be put onto the stack, it isn't put on the stack" — a refusal at
   the moment CR 603.3 puts an ability on the stack, which is the one moment
   this engine does not have. Its first sentence (an object owned by a departed
   player is not created) landed with RE-7 at `Primitive::CreateToken`;
   `ATOM-800.4d-001` is `COVERS-PARTIAL` on that test and names this half as
   the reason. The rule's own example is Astral Slide's delayed trigger, which
   is also the shape that will reach it first: a *delayed* trigger outlives the
   departure that its source did not.

   **Reachability (2026-09-13):** unreachable — no ability is put onto the
   stack by a trigger, so there is nothing to refuse.

   **Sized:** one `in_game` read at the dispatcher's put-on-stack site,
   ~5 lines and a four-player fixture, inside critical-path item 6.

   **Phase (2026-09-18):** TR-1 — one `in_game` read at the head of `place_pending_triggers`, Astral Slide's shape as the four-player fixture; `triggers-architecture.md` §5.3.

9. **~~The dispatcher is the trace sink's sixth emit point, and it does not
   exist yet (A4c, 2026-09-18).~~ — ✅ CLOSED 2026-09-19 (TR-1).** `trace_records::trigger` per matcher decision — the record, the identity with its instance, the zone, matched or the predicate that refused it, and whether it resolved as a mana ability — and `trace_records::pending` per placement or refusal, with the targets; `plans/traces/viewer.html` renders both kinds.
   The record as found: The five item 5 named are built — the batch,
   the CR 616.1 iteration, the layer walk, the decision boundary, the
   performed event — and "why did this fire, or not" is answered by none of
   them: whether a trigger matched an event is a read the matcher makes, and
   a sink can only record what it is handed. Owed with the dispatcher: a
   `trigger` record per matcher decision (the event, the ability's identity,
   its source, matched or not and which predicate said so) and a `pending`
   record when the queue drains onto the stack in APNAP order (CR 603.3b),
   each written behind `game.trace` the way the five are, so a game nobody
   traces pays one branch at each. `plans/traces/viewer.html` gets a summary
   arm for both kinds in the same PR.

   **Reachability (2026-09-18):** unreachable — no dispatcher, so no record it
   could write.

   **Sized:** ~40 lines at the two sites the dispatcher adds, inside
   critical-path item 6's first PR; the viewer's two arms ~20.

   **Phase (2026-09-18):** TR-1 — the `trigger` and `pending` records, and the viewer's two arms; `triggers-architecture.md` §4.8.

10. **~~Three performers drop a proposal field the record needs (the trigger
    survey, 2026-09-18).~~ — ✅ CLOSED 2026-09-19 (TR-1).** `StepBegin.player`, `PhaseBegin.player`, `DamageDealt.is_combat` and `LifeChanged.cause: Option<LifeLossCause>` (`None` for a gain) off the three performers, every literal site patched, and the engine arm read `IDENTICAL` on every counter as the item predicted (`fuzz-record.md`, TR-1).
   The record as found: `GameAction::BeginStep` and `BeginPhase` carry
    `player`; `GameEvent::StepBegin` and `PhaseBegin` do not, and "at the
    beginning of your upkeep" (1,167 cards), "your end step" (976), "combat"
    (313) and every delayed "at the beginning of the next" (391) read whose
    turn it is. `DealDamage` carries `is_combat`; `DamageDealt` does not, and
    804 cards say "combat damage". `LoseLife` carries a `LifeLossCause`;
    `LifeChanged` does not, and CR 727.1a's "from radiation" reads it (one
    card). The first is live-derivable at dispatch — the active player owns
    every step — and the second is not: a triggered ability resolving during
    the combat damage step deals noncombat damage in that step, so the step
    cannot say. `plans/references/trigger-survey.md` §5.

    **Reachability (2026-09-18):** unreachable — no dispatcher reads any
    event; a record that cannot say is wrong only once something reads it,
    and the trace's `event` line is the only reader it has today.

    **Sized:** three fields, three performers and `format_event`'s three
    arms, plus the literal sites that name the shapes (`StepBegin {` at 5 in
    `src` and 12 in `tests`, `DamageDealt {` 10 and 7, `LifeChanged {` 10 and
    9), ~40 lines; each field's shape is the triggers doc's. It changes no
    decision and does change the dump's text, so it rides a stream-neutral PR
    whose A/B prediction is `IDENTICAL`.

    **Phase (2026-09-18):** TR-1 — all three fields, a stream-neutral change whose A/B prediction is `IDENTICAL`; `triggers-architecture.md` §3.12.

18. **~~Three `GameEvent` variants are never emitted (the trigger survey,
    2026-09-18).~~ — ✅ CLOSED 2026-09-19 (TR-1).** `PhaseEnd`, `StepEnd` and `TurnEnd` deleted with their `format_event` arms; `CountersAnnihilated` is TR-5's, once CR 704.5q proposes.
   The record as found: `PhaseEnd`, `StepEnd` and `TurnEnd` are declared and
    written by no performer; no trigger reads an end — "at end of combat" is
    the end-of-combat step beginning (CR 511.2) and "at end of turn" was
    errata'd to "at the beginning of the end step" (CR 513.1a). Emit or
    delete; delete, since an arm the stream cannot carry misleads the reader
    the way `replacement-architecture.md` §3.2a's unapplied pattern arm does.

    **Reachability (2026-09-18):** nothing owed to correctness — dead
    vocabulary, and the survey's stream table counts it.

    **Sized:** three variants and their `format_event` arms, ~15 lines, any
    time.

    **Phase (2026-09-18):** TR-1 — deleted; `triggers-architecture.md` §3.12.

## Before Commander (CR 903)

1. **Commander damage increment — ✅ done (2026-04-18).** `GameObject.is_commander: bool` added; `execute_action(DealDamage)` accumulates `commander_damage_taken[source]` when `is_combat && target == Player && source.is_commander`. 5 unit tests. The loss SBA (`engine/sba.rs:73`) now has a live writer.

   **Reachability (2026-09-03):** closed — 2026-04-18.


## Cross-cutting — keep this section honest

124. **Three types carry an object set called `affected`, and two of them now
     have a player sibling — so the bare name is wrong in two places and will be
     wrong in a third.** RD-4's review already made this call for
     `Restriction::ApplyReplacement`, which is `to_objects` / `to_players`: *"a
     bare `to` beside a `to_players` reads as the whole set with a modifier hung
     off it, and it is not — the two are unioned and neither is primary."*
     RE-3 renamed `Restriction::Event.affected` to `affected_objects` on that
     argument (19 sites). Two are left:

     - **`ReplacementDef.affected`** beside `affected_players`, which is the
       original instance of the asymmetry and the largest: ~135 field uses
       across the card files.
     - **`ContinuousEffect.affected`**, which has no player sibling *yet*. It is
       the one that will need it: CR 611.1's continuous effects are not all
       about objects — "you have no maximum hand size", "players can't untap
       more than one permanent" — and the day one of those is written as a
       layer row rather than a restriction, this field grows the same pair.

     **Reachability (2026-09-12):** reachable, **not wrong** — a name, not an
     answer. Every reader of both fields already asks for the object half
     explicitly.

     **Sized:** a mechanical rename, ~135 + ~24 sites, and it is
     `AbilityDef`'s named-constructors shape (item 120): **its own PR**, so a
     sweep does not ride inside a rules change. `AffectedSet` → `ObjectSet`
     (302 mentions) is the same PR's second half if it is taken — the type is
     already object-only, and the name says "affected" where the field name
     now says it twice.

    **Reachability (2026-09-14):** closed — `refactor/object-set-rename`, PR #136.

    **What the sweep found that this entry had wrong: there are five fields
    named `affected`, not three.** `board.rs` keeps two more — `TraceStep` and
    `Observation`, both module-private, both a resolved `Vec<ObjectId>` rather
    than a selector for one. For those the bare noun is already the right
    name: they hold the objects one application *reached*, and there is no
    player half to be half of. This entry counted the `AffectedSet` carriers
    and generalized from them, which is why the rename went through the
    compiler rather than a token sweep — `affected` would have taken
    those two and nine locals with it.

    Both halves landed in one PR. The field rename alone leaves
    `affected_objects: AffectedSet`, which is the one spelling that reads
    worse than either end, so the split this entry offered ("the same PR's
    second half if it is taken") was taken. Sizes as built: 55 lines for the
    fields, 372 for the type, 125 across the live plan docs.

    `plans/archive/` keeps the old spelling, deliberately. It is a record of
    what shipped, and CM-0 set the precedent seven days earlier —
    `PermanentFilter` still stands in three archive files with the live docs
    fully swept.

### Item 111 — closed 2026-09-15 by RE-9

**What closed it.** `GameAction::ProduceMana` is the mana production event (CR
106.6a, 106.12b), proposed by `resolve_mana_effect` and `Primitive::ProduceMana`
and performed by one `perform_action` arm that writes the pool and emits
`GameEvent::ManaAdded` — emitted for the first time since the log was written,
its `HashMap` a `Vec` in proposal order. Mana Reflection and Nyxbloom Ancient
are the two printed replacements the entry named, Deep Water the third card,
and CR 605.1b's eight triggered mana abilities read `tapped_for_mana` off the
event when critical-path item 6 builds them. The A/B on the hottest path, which
the sizing called the risk, read +1.2% CPU/game at two seats against a
2.5-point gate; `replacement-architecture.md` §9's RE-9 stub and the archive
carry the reading. **The item's one wrong sentence:** it said `resolve.rs:337`,
which RE-5's and RE-10's arms had pushed to 530 by the time it was read.

*Original entry:*

111. **Mana production is a direct write with no event.** `mana.rs:91`
     (`resolve_mana_effect`) and `resolve.rs:337` (`Primitive::ProduceMana`)
     both write `mana_pool` below the chokepoint, and `GameEvent::ManaAdded`
     is emitted at zero sites — which is why `--dump-events` has no mana lines
     and the fuzz A/B recipe counts `Tapped:` land lines instead. RA's census
     walked emissions and so could not see a mutation that emitted nothing.
     CR 106.6a's two printed replacements and CR 605.1b's eight triggered mana
     abilities read this event.

     **Reachability (2026-09-11):** reachable — every land tap in every game;
     wrong in the *log*, not on the board, since nothing watches mana yet.

     **Sized:** RE-9, `replacement-architecture.md` §9 — one
     `GameAction::ProduceMana`, one performer replacing two writers, ~300
     engine lines; the A/B on the hottest path is the risk, not the diff.

## Found by the CV-1 review (2026-09-02; absorbed 2026-09-03)

### Item 69 — closed 2026-09-15 by the post-RE audit's pass 3

**What closed it.** Two halves, and both landed without this item being
touched. `fuzz_games --players N` shipped with RE-7 on 2026-09-13 —
`fuzz-record.md`'s blocks carry four-seat columns from RE-7 on — which was
the whole of the "Sized" line, and RE-7's CR 800.4a work answered the
"Blocked on CR 800" line the same day. The Commander-scale board was
measured on 2026-09-15 by a throwaway probe: four 100-card decks at 40 life
on both pools, 100 games each at seed 12345 — 83 turns a game against 61–62
at 60 cards, 29.6–31.7 permanents on the battlefield at a priority prompt
(max 78–81) against 25.0–26.6 (max 69–73), 406 / 889 objects at most
(`performance` / `stress`) against 246 / 253, and 87 / 137 ms of CPU a game
against 51 / 76 ms. So the board is about 1.2× the size and the game about
1.7–1.8× the cost, and the cost is mostly length. The numbers live in
`codebase-state.md` items 138 and 143, which re-rank the levers this item
said to re-measure first. **What the item had wrong:** its opening sentence
was stale from RE-7 on.

*Original entry:*

69. **Every performance number this project owns is two-player, and v1's
    profile is four (recorded 2026-09-07).** `fuzz_games` builds
    `Game::new(config, vec![deck1, deck2])` — a literal pair, no `--players`
    flag — so `fuzz-record.md`'s tables, `layers-architecture.md` §12's
    measurements and §13b's scaling table all describe a board the target use
    case does not build. `Game::new` already takes a `Vec` of decks and
    `GameState.players` is a `Vec`, so the harness is the only thing that is
    two-player here.

    **Why it matters more than a percentage.** Since 7a the cost model is
    roughly *board walks × cost of one pass*, and a pass is linear in members
    and in the applications each layer holds. Both scale with player count: a
    four-player Commander board carries several times the permanents of a
    pooled two-player game, and several times the static abilities. The LI-1
    scaling table is the one to read, and it spans two orders of magnitude
    across the range a real board covers — 6.4 µs at 10 permanents with one
    row, 1,195 µs at 80 with 80. Which end v1 sits at is not known, and no
    amount of two-player fuzzing will say.

    **This is the measurement to bump, not the optimizations.** The two
    remaining answer-preserving levers — the `Arc<Vec<AbilityDef>>` elision
    (item 67) and interning `EffectGroup` (§12) — were both measured *before*
    7a cut walks per game by ~40×, and item 67 already says re-measure before
    paying. Optimising a two-player 70-card profile for a four-player
    Commander target is optimising the wrong board; the fix is to be able to
    see the right one.

    **What triggers do and do not change.** Item 6 adds *queries* — a
    characteristics read per trigger condition and per intervening "if" — and
    since 7a most of those are memo hits against an unchanged board. What it
    adds to the cost model is *writes*, which bump the epoch and force a fresh
    pass. So triggers move the `Board walks` term, which the harness already
    prints, rather than introducing a term nobody has measured. That is why
    the board-size question can be asked now and the query-volume question
    cannot: the first is a property of the game state, the second of a system
    that does not exist.

    **Reachability (2026-09-07):** reachable — not a wrong answer, a blind
    spot. Nothing is mis-computed; the profile is simply invisible.

    **Sized:** `--players N` on `fuzz_games` plus `random_deck` per player and
    the two-player assumptions in the harness's own summary rows, ~80–120
    lines. Independently owed by `CLAUDE.md`'s "write new systems N-player-shaped
    from the start" — the harness is a system and it is not. **Blocked on CR
    800** for a game that *runs* correctly past two players (priority passes
    loop player0 → player1, line 189 above), so the honest order is: CR 800,
    then this, then re-measure, then choose a lever.

### Item 67 — closed 2026-09-16 by A4f (PR #157)

**What closed it.** The shape the item named, built as named:
`CardData.abilities`, `EffectiveCharacteristics.abilities` and
`CopiableValues.abilities` are `Arc<Vec<AbilityDef>>`,
`get_effective_abilities` returns the memoized frame's own `Arc`, and the
seven writers copy on write — six through `Arc::make_mut`, the two clears by
replacing the `Arc`, since `make_mut` on a list still shared with the card
would deep-clone it only to empty it. A frame no layer writes shares the
card's allocation (`Arc::ptr_eq`, a unit test) and a Layer 6 grant copies
before it writes (the other). `CopiableValues` stays a value under the
sharing: every writer copies before it writes while the capture holds the
list, so CV-1's snapshot is kept by copy-on-write rather than by a deep
copy. "Re-measure before paying" was paid twice: the 2026-09-15 callgrind
read put the clone at 22.5% of a four-seat `stress` game, and the
2026-09-16 re-read — both arms rebuilt on the same 161-card pool, since PR
#156 had moved it — put `main` at 906.8 M instructions a game and this at
622.4 M, −31.4%, with the `to_vec` row gone and `hash_one` unchanged to the
instruction (`layers-architecture.md` §12). Native: −30.6% CPU a game at
four seats, −33.6% at two, every counter `IDENTICAL`. **What the item had
right:** the fix it named on 2026-09-02 was the fix; the one addition, the
wrapper returning the `Arc`, the 2026-09-15 re-measure had already made.

*Original entry:*

67. **`CopiableValues::apply_to` deep-clones a `Vec<AbilityDef>` into every
    frame of every copied object (C5).** The `String`, five `HashSet`s and the
    ability tree are cloned twice per walk — once seeding the frame from
    `CardData`, once overwriting it from the capture. Not a new lever:
    `layers-architecture.md` §12 measured eliding the per-frame
    `Vec<AbilityDef>` clone at ~30% before 7a and names the fix —
    `Arc<Vec<AbilityDef>>` on `CardData`, `Arc::make_mut` in the Layer 4 and 6
    arms — and `CopiableValues.abilities` takes the identical treatment.

    **Reachability (2026-09-03):** reachable — not wrong; perf only, on every
    walk of a copied permanent. Note that 7a landed since (walks per game
    108,626 → 2,663), so the per-frame cost is paid forty times less often than
    when §12 measured it; re-measure before paying.

    **Sized:** ~100–150 lines across `CardData`, `CopiableValues`,
    `compute.rs` and the two mutating arms; answer-preserving; lands when a wide
    copied board (Mirrorform onto twenty permanents) measures it, or with §12's
    next perf item.

    **Re-measured 2026-09-15 (callgrind, the post-RE audit's pass 3; item
    138's lever 1).** Cloning `Vec<AbilityDef>` is 22.5% of all instructions
    in a four-seat `stress` game: 6.6% is this item's per-frame clone from
    `CardData` on a walk, and ~15% is `get_effective_abilities` cloning the
    list out of a memo hit for the enumeration wrappers and the two sweeps —
    so the `Arc` this item names is the largest single lever the engine has,
    the wrapper returns the `Arc` with it, and "re-measure before paying" is
    paid.

### Item 144 — closed 2026-09-16 by A4g (PR #158)

**What closed it.** The shape the item decided, built as decided, in the two
arms its "Sized" line asked for. `ObjectId` and `AbilityId` are newtypes over
a `u64` (the second open question, answered: the thirteen pair sites can no
longer swap their halves silently). An `ObjectId` is stamped in
`GameState::add_object` from `next_object_id`, beside CR 613.7d's timestamp —
the one door into the store — and `GameObject::new` hands out
`ObjectId::UNASSIGNED` until then, so every caller reads the id off the
return value; a fork clones the counter. A printed `AbilityId` is
`AbilityId::printed(card_name, ordinal)`, derived in `CardDataBuilder::build`
over every def reachable from the card — the printed list, then each def's
nested defs (a granted ability, a token's abilities) through
`Effect::for_each_ability_def_mut` — and a def that already carries an id
keeps it, which is what the CR 113.10b test needs. That settles the first
open question the other way from the intrinsic site: a granted ability keeps
the id its def carries rather than deriving one from the granting object,
because "printed + granted the same def" must read as two instances of one
ability. The intrinsic site is `AbilityId::derived_on(object, land_type)` over
the integer, same inputs, same stability, and the top two bits of an
`AbilityId` say which derivation made it so the three ranges cannot meet.
`new_object_id` and `new_ability_id` are test-only process counters. The
`uuid` crate left with its whole tail — thirty crates out of the lockfile.

The hasher, `types::ids::IdHash`, sits on all 27 id-keyed declarations
(the item counted 25): one multiply and a fold per word, seeded once per
process from `MTGSIM_HASH_SEED`; CI's determinism step runs its three runs
under three seeds and `fuzz_ab.py` gives each timing round its own, which
is the re-arm the item asked for. `tests/determinism_test.rs` compares its
two logs verbatim, `strip()` and the "share one `RandomState`" sentence gone;
the three display prefixes are gone with the UUID they sliced.

**Measured.** Callgrind at four seats on `stress`, 200 games, both arms on
A4f's 161-card pool: 622.4 M instructions a game → 568.8 M for the swap alone
(−8.6%: SipHash over eight bytes instead of sixteen) → 380.7 M with the hasher
(**−38.8%**). The `hash_one` row (32.6%) has no successor; `sip.rs`' self cost
fell from 14.1% to 5.0%, and what is left of it is the frame's
`HashSet<CardType>` and `HashSet<Subtype>`; `LayerMemo::get` fell 72%. Native,
`µs / decision`: 76.9 → 68.9 → 48.2 at four seats (−37.3%), 43.9 → 41.1 →
29.0 at two (−34.0%). The hasher arm reads identical to the swap arm on
every counter at both seat counts on both pools under a different seed per
round — no sweep leaks iteration order. **What the item did not foresee:** the
swap is not `IDENTICAL` to `main` on every board. Six games in 800 diverge,
every one through two Citanul Hierophants under one controller: their two
grants of `{T}: Add {G}` now share an id, the mana window's pair-keyed dedupe
offers one candidate where `main` offered two, and the random agent's pick
moves. Attributed by a probe build that restored per-copy uniqueness and read
`main` to the digit. That is the per-definition id the item chose ("the 122
random bits buy nothing a per-card index would not") meeting CR 113.10b's
"instances", and it is item 149 now, with what it leaves for the triggers doc.
**What the item had right:** the census, the door, the derivation and the
re-arm were the design; the two counts it guessed at (25 declarations, six
prefix sites) were 27 and three.

*Original entry:*

144. **`ObjectId` and `AbilityId` are v4 UUIDs, and the decision (the owner,
     2026-09-16) is to replace both with process-stable ids.** The census:

     - **Two ids are `Uuid`; every other id is a counter.** `types/ids.rs`
       aliases `ObjectId` and `AbilityId` to `Uuid`; `EffectId`, `RowId`,
       `ReplacementEffectId`, `RestrictionId`, `BatchId`, the timestamps and
       the epochs are `u64`s from counters, and `PlayerId` is a `usize`.
     - **v4 is minted at one production site for objects** (`GameObject::new`)
       and once per `AbilityDef` at card construction (`CardDataBuilder` and
       the card files), from the operating system's randomness rather than
       `GameState.rng` — ambient, and tolerated only because an id never
       orders anything (`CLAUDE.md`, "never `ObjectId`";
       `battlefield_ordered`'s own note says sorting by id is no fix because
       the key is itself random).
     - **v5 exists for one function.** `land_types.rs::intrinsic_ability_id`
       (2026-08-20) derives the CR 305.6 intrinsic mana ability's id from the
       object id and the land type, because the ability is synthesized inside
       the layer walk on every read, has nowhere to store a minted id, and is
       handed out as an activation handle that a recomputed list must match.
       It is the one place the engine wanted a deterministic id, and it is the
       shape the rest should have.
     - **`AbilityId` is per `CardData`, not per object.** A plural token
       creation shares one `Arc<CardData>` across equal defs
       (`create_tokens`), and a copy keeps its source's `AbilityDef` ids
       (`engine/layers/copy.rs`), so the engine already keys ability identity
       as the pair `(ObjectId, AbilityId)` (13 sites) and finds an ability by
       `a.id == ability_id` within one object's effective list (13 sites). The
       122 random bits buy nothing a per-card index would not.
     - **The cost is frequency times the default hasher, not the comparison.**
       25 `HashMap` and `HashSet` declarations are keyed by an id or the pair —
       `objects`, `battlefield`, `stack_entries`, `LayerMemo`, `Board`'s frames
       and sub-cache, the three ability-source sets, the combat maps, the
       random provider's mana maps — and every lookup runs SipHash-1-3 over 16
       bytes, 32 for a pair: 13.5 M `is_creature`, 10.5 M
       `object_matches_filter` and 7.0 M `has_type` lookups in 200 four-seat
       `stress` games (`layers-architecture.md` §12), 22.1% of instructions.
       SipHash defends against hostile keys, and no untrusted key ever reaches
       these maps.

     **The decision, and what it buys beyond the hash.** `ObjectId` becomes a
     `u64` stamped in `GameState::add_object`, exactly where the CR 613.7d
     timestamp is already stamped ("the only door into the store");
     `AbilityId` becomes a `u64` too, a printed ability's derived from the card
     name and the ability's index the way the v5 site already derives its own,
     so it is stable across processes and threads with no counter. Then: every
     id halves, across 88 `Vec<ObjectId>` sites, every `GameEvent` and every
     map key, and the wire ids item 141 wants; the fuzz-dump masks and item
     41's fork-test id mask go, because ids agree across runs and binaries;
     the `uuid` dependency and the engine's one ambient-randomness call go
     with it; and the hasher becomes a one-line multiplicative mix, since
     sequential keys under an identity hash share hashbrown's 7-bit tag. Item
     138's lever 2 is this item now, and the hasher rides inside it rather
     than beside it.

     **The one cost, and the fix that rides with it.** With process-stable ids
     and a fixed hasher, `HashMap` iteration order becomes process-stable, so
     the three-run determinism check stops catching an unordered sweep that
     leaks order. Re-arm it in the same PR: the CI determinism step runs its
     three runs with a hasher seed that differs per run (an environment
     variable the `BuildHasher` reads, ~10 lines), which restores the property
     the check had under `RandomState`. Two notes beside it: a fork shares the
     counter, so two diverging branches mint the same next id for different
     objects — harmless unless branches are merged, which nothing does — and
     `tests/determinism_test.rs`'s doc comment, "two runs inside the same
     process share one `RandomState`", is imprecise (each map draws its own
     keys) and is rewritten when the test gains its fork rows.

     **Open for the PR to decide, not decided here:** how a *granted* or
     *synthesized* ability's id is minted — from the state's counter at the
     grant, or derived from the granting object and a tag the way the
     intrinsic site does — and whether `ObjectId` and `AbilityId` stay two
     aliases of one integer type or become two newtypes, which is what would
     let the compiler catch a swapped argument in the 13 pair sites.

     **Reachability (2026-09-16):** reachable — not wrong; a cost (22.1% of
     instructions), a mask at every log comparison, and one ambient-randomness
     call per object.

     **Sized:** the swap sites are few because both ids are type aliases — 1
     production minting site for objects and about 30 test sites calling
     `new_object_id()` (a test-local counter), 39 direct `Uuid::` uses outside
     `ids.rs`, nearly all in test modules (`new_v4` in registry and store
     tests, `nil()` twice, `from_u128` once, the v5 site), 6 id-formatting
     sites (the eight-character prefix in `ui/display.rs`), the 25
     declarations behind a type alias, and the CI seed; about 200–300 lines.
     One PR in the band (`roadmap-v2.md` A4g), A/B'd as two arms the way the
     Everywhere PR was: the type swap alone, expected `IDENTICAL` on every
     counter at two and four seats on both pools, then the hasher, read as a
     CPU delta and a callgrind re-read against §12's reading.

## Found by the post-RE audit, pass 3 — parallel-play readiness (2026-09-15)

*Closed by A4h, 2026-09-16. The live file keeps the stub at item 139; this is
the entry as it stood when the work was sized.*

139. **A retry re-prompt offers a list computed before the rejected action
     changed the board — the one thing the fork test found on the stack.**
     `run_priority_round` computes `all_candidates` once per round and the
     retry loop re-asks with that list minus a `blacklist`; when the rejected
     action was a cast whose mana abilities stay activated (CR 732.1's "may
     not reverse" branch, item 72), the board at the re-prompt has fewer
     untapped sources and more mana floating than the list was computed
     from, so the re-prompt offers casts a fresh enumeration would not — and
     a fork at that prompt, resuming with a fresh round, offers a different
     list. Measured 2026-09-15 by running item 41's test as a throwaway
     probe — record every provider answer, clone `GameState` at the first
     prompt of a priority round, replay the recorded answers from that
     prompt on, compare the rendered logs with ids masked: **779 forks over
     50 games, 741 replayed the original game event for event, and every one
     of the other 38 was this mechanism** — a prompt mismatch at the fork
     itself, and none diverged without one. Twenty-one of the 38 rejoined
     the original game anyway (the random agent's next pick converged);
     seventeen played a different game. The three boards: `performance` at
     two seats, 353 forks, 336 identical; `stress` at four, 314, 296;
     Commander scale, 112, 109.

     **Why it is not a rules bug.** The candidate list is an overapproximation
     by contract (`plans/atomic-tests/supplemental-docs/dp-middleware-and-candidate-enumeration.md`
     §2) and the engine rejects what it cannot pay; the game played is
     legal. What is wrong is that item 40's table called `all_candidates`
     and `blacklist` harmless — "drop them and the fork re-offers a cast that
     fails again, slower, same game" — and it is not the same game: the
     re-offer is of a *different* list. So the prompt is not a function of
     the state, which is the property the fork model needs and the one item
     40's invariant was written to protect.

     **Reachability (2026-09-15):** reachable — not wrong today; a legal
     game, and a prompt a fork cannot rebuild. Every game reaches it: 54–85
     same-player re-asks a game at four seats.

     **Sized:** recompute the candidates after a rejected action — move the
     enumeration inside the retry loop, minus the blacklist — ~5 lines in
     `engine/priority.rs`; it moves the random agent's stream, so its own PR
     with the A/B and a `fuzz-record.md` block, and item 41's test rides in
     it (the test cannot be green without it). The `blacklist` stays
     stack-resident until item 140.

     **Closed 2026-09-16 (A4h).** The enumeration moved inside the retry
     loop; the blacklist stayed on the stack, and the fork test that rode in
     with the fix says it is still outcome-bearing there — item 140 owns
     that half. The measurement the fix is read against is
     `tests/priority_fork_test.rs`' 192-game sweep and the A4h block of
     `fuzz-record.md`.

### Item 159 — closed 2026-09-18 by A4o (PR #163)

**What closed it.** The fix the item sized, at the three sites it named, with
the predicate in one place rather than three: `GameState::is_spell_on_stack`,
which the `Spell` arms now ask and the CR 609.7a `DamageSource` arms ask
instead of their three inline copies. That is the item's open decision
answered the smaller way — six copies of one question would have been the
shape that cost main item 8 a redesign — and it leaves the complement
("counter target activated or triggered ability", `Primitive::CounterAbility`
with no filter to reach it) as a negation of one function.

**The predicate is the entry, not the stack.** An ability on the stack is a
`GameObject` like any other (CR 113.7a) and nothing about the object says
which it is; `StackEntry::is_spell` does, and it is the field the resolution
and the fizzle already branch on. The one object the new predicate declines is
the spell *currently resolving*, whose entry `resolve_top_of_stack` has
already taken — the same object the `DamageSource` arm had always declined,
and unreachable either way, since CR 601.2c and CR 603.3d both choose before a
resolution starts.

**What the item did not predict: how loud it was.** The item said "in every
`performance` and `stress` game that lines the two cards up" and sized the
A/B's prediction as `differ`. Both held, and the dumps put numbers on them:
11 of 200 games diverge at two seats on `performance`, 10 on `stress`, 14 and
21 at four seats, and in `main` those runs manufactured 26 phantom cards over
25 games from nine different sources — not only the Merfolk Thaumaturgist the
fixture used, but Chainbreaker, Bonesplitter, Samite Healer, Mind Stone, Words
of Worship, Deep Water, Circle of Protection: Red and Aggravated Assault. Every
diverging game is a game where this filter's answer changed, in all four arms,
which is the containment the A/B exists to show; the converse does not hold,
because a shorter option list the agent would have passed on anyway leaves the
stream where it was. `fuzz-record.md`, the A4o block.

**And a second changed answer the sizing had folded into the first.** The item
counted three sites and they are three, but they do not all change the same
thing: the count arm withdraws the *offer* (no legal target, so CR 601.2c
forbids the cast), while the enumeration arm only shortens the *candidate
list* — which is the whole difference whenever a real spell sits on the stack
beside the ability, where the count is ≥ 1 either way. A probe of the count
arm alone left one diverging game unexplained; both together leave none.

*Original entry:*

159. **`SelectionFilter::Spell` accepts an activated ability on the stack, and
     `Primitive::CounterSpell` then puts the ephemeral ability object into a
     graveyard as a card (found by A4i's audit, 2026-09-17).**
     `validate_spell_target` checks `stack.contains` and nothing else, the
     enumeration's `stack()` closure yields every stack id, and A4i's
     `has_legal_choices` `Spell` arm counts every stack id; only the sibling
     `DamageSource` arm filters on `is_spell`. An activated ability on the
     stack is a `GameObject` carrying a clone of its source's `CardData`
     (`put_on_stack.rs::activate_ability`), so "counter target spell" can name
     it, and the counter primitive's `change_zone` to the graveyard lands that
     clone in the owner's graveyard as a second copy of the card.

     **Reproduced with a fixture (2026-09-17), and it is in the measured
     games.** Counterspell and Merfolk Thaumaturgist's activated ability are
     both in `PERFORMANCE_POOL`. Thaumaturgist on the battlefield under player
     1, Counterspell in player 0's hand with {U}{U}; player 1 activates, player
     0 casts — `castable_spells` offers it, and the target is forced with no
     prompt since the ability is the only other stack object — and the stack
     resolves to **two Merfolk Thaumaturgist objects, one on the battlefield
     and one in the graveyard**. Nothing panics, which is why no fuzz run
     noticed.

     **Reachability (2026-09-17):** reachable — wrong today, in every
     `performance` and `stress` game that lines the two cards up. Row A4o.

     **Sized:** ~20 lines and a regression that casts from hand. The `is_spell`
     filter the `DamageSource` arm already has, in three places — the
     validator, the enumeration arm, the count arm. It changes what
     `castable_spells` offers whenever an ability is on the stack, so it moves
     the random agent's stream and owes its own A/B and a `fuzz-record.md`
     block, with `differ` the honest prediction on both pools.

### Item 160 — closed 2026-09-18 by A4p (PR #164)

**What closed it.** The three edits the item sized, at the three sites it
named. One of them does more than its line suggests: the enumeration's
`players()` closure is shared by the `Player` arm and the `Any` arm, so
filtering it once withdraws the offer from both, and the `Any` arm needed no
edit of its own. The two count arms share a closure for the same reason,
written as a closure rather than a `let` so the battlefield filters do not pay
for a seat scan on `mana_helpers`' hot path.

**The two halves failed differently, and the item was right that they did.**
"Target player" was offered and then refused — `validate_player_target` has had
the CR 800.4a check since it was written, so the oracle promised a cast the
engine rewound, which costs a priority action and lands no effect. "Any target"
was caught nowhere, and that is the half that cost something: three damage,
resolving against a player the game no longer had.

**What the item did not predict, three things.** The A/B prediction named the
four-seat `stress` arm as the one that would differ; four-seat `performance`
differs too, and by nearly as much — 65 of 200 games against `stress`' 68, with
Lightning Bolt pooled on both. **Only the enumeration ever changed an answer in the
400 measured games:** every registered card with a `Player` or `Any`
instance declares `TargetCount::Exactly(1)`, so both count arms are asked
`n = 1` and say `true` while any seat remains — the count half is correct and
unreachable at once, and what would reach it is a card with two instances of
"target player" and two seats left. And **the damage did not come through
CR 608.2b**, which is where the item's fixture put it: `validate_any_target`
never refused a target on the fixed arm at all, because the enumeration had
stopped offering the seat first. The eleven Lightning Bolts `main` resolved
against a departed seat were aimed at one *at announcement*, by an enumeration
that still listed it. The validator is the back-stop for the narrower window
the fixture builds — a seat that leaves between announcement and resolution —
and that window did not open once in 400 games, which is exactly why it needed
a fixture rather than a fuzz run.

*Original entry:*

160. **The `Player` and `Any` selection arms count and offer seats that have
     left the game (found by A4i's audit, 2026-09-17; pre-existing, inherited
     by A4i's count logic).** `GameState::num_players()` is the player
     vector's length, which CR 800.4a never shrinks; `enumerate_legal_selections_upto`'s
     `players()` closure yields `0..num_players()` for `Player` and `Any`, and
     `has_legal_choices` counts `players.len()` for `Player` and seeds `Any`'s
     count with it. `validate_player_target` refuses a departed seat under
     CR 800.4a and its comment says such a seat is "not offered at CR 601.2c",
     which the enumeration contradicts; `validate_any_target` never asks
     `in_game` at all.

     **Reproduced with a fixture (2026-09-17)** at four seats with seat 3
     departed: both filters offer `Player(3)`; `validate_targets` refuses it
     for `Player` — a cast the oracle offered and the engine rewinds, item
     139's class — and **accepts it for `Any`**, so "any target" damage
     resolves against a player who is not in the game.

     **Reachability (2026-09-17):** reachable — wrong today at four seats, in
     `fuzz_games --players 4` from the first elimination on, and v1 is four
     seats. Unreachable at two, where a departure ends the game (CR 104.2a), so
     every recorded two-seat table is untouched. Row A4p.

     **Sized:** three edits, ~15 lines, and two fixtures at four seats. The
     player iterator filters on `in_game`; the two count arms count in-game
     seats; `validate_any_target` gains the check the `Player` validator has.
     A/B prediction: `IDENTICAL` on both pools at two seats; the four-seat
     `stress` arm differs, and that is the finding the arm exists to show.

### Item 132 — closed 2026-09-18 by A4m (PR #165)

**What closed it.** `EngineCounters` is `Diagnostics`, `GameState.counters` is
`GameState.diagnostics`, and the 27 recorders and accessors keep their names. The
owner took `Diagnostics` over this entry's proposed `EngineMeters`: the module is
already `state::diagnostics`, so the type stops fighting its path, and the one
place the stutter would show — the fully-qualified field declaration — takes a
`use` instead.

**The sizing was low — 81 against 120 — and both misses are the same kind.** This
entry said six `EngineCounters` sites and ~75 `.counters.` calls; `roadmap-v2.md`
row A4m later said 72 sites across 12 files, lower still. The tree at 573ca8b had
**120 lines across 21 files**. `ui/ask.rs` alone is 48 of them, because the diagnostics reach
its four `validate_*` helpers as an *argument* — 25 call sites pass
`&game.counters`, and its own unit tests build 11 `EngineCounters::default()`s —
and seven integration test files read the accessors for another 34. Neither count
included an argument site or a test file, which is what a count taken by grepping
for the type and the method calls will always miss.

**What made the sweep safe was not the count.** Every substitution was anchored
on one of the 27 recorders and accessors, never on `.counters.` alone, which
matches two `HashMap`s one struct over. That the anchor cannot hit CR 122's
counters is checkable rather than hopeful: a `HashMap` has no `record_layer_walk`.

**Nothing moved, as a rename should not.** `plans/fuzz_ab.py --rounds 0` against a
`main` arm built at 573ca8b reads `IDENTICAL` on both pools at two seats and at
four, and raw `fuzz_games` output between the arms is byte-identical apart from
the four timing lines. The printed row labels and `fuzz_ab.py`'s `ROWS` table are
untouched on purpose — they are what every table in `fuzz-record.md` is keyed on —
so this PR has no `fuzz-record.md` block.

*Original entry:*

132. **`GameState.counters` is the engine's diagnostics, and the two fields
     one struct over with the same name are CR 122's counters.**
     `PermanentState.counters` and `PlayerState.counters` hold +1/+1, loyalty,
     poison and energy; `GameState.counters` holds `EngineCounters` — the
     layer walks, gathers and productions `fuzz_games` prints. Three fields,
     one spelling, two meanings; RE-9's design check wrote "a permanent
     counter" about a diagnostic row and the review asked which
     (`replacement-architecture.md` §11 item 97).

     **Reachability (2026-09-15):** reachable, and not wrong — a name. Every
     reader compiles and every number is right; what is wrong is what a
     reader assumes before the type tells them.

     **Sized:** a mechanical sweep, six `EngineCounters` sites and ~75
     `.counters.` calls and accessors, **its own PR** on main item 124's
     precedent (`refactor/object-set-rename`): a rename does not ride inside
     a rules change. Proposed name `EngineMeters` / `game.meters`, a word the
     CR never uses and one that reads as measurement at every call site.

## Found by RE's sizing (2026-09-11)

### Item 121 — closed 2026-09-19 by TR-1

Closed by the two tests the item asked for, in `tests/phase_tr1_integration_test.rs`: a skipped upkeep emits no `StepBegin` so Verdant Force never triggers, and a fixture's untap trigger is held through the skipped step and goes on the stack at the draw step's first grant. Both carry `// RULING: Eon Hub #2` and `#3`.

*Original entry:*

121. **Eon Hub's two trigger-shaped rulings have no test and cannot have one
     until item 6.** *"Upkeep-triggered abilities don't trigger"* and *"any
     triggered abilities that triggered during the untap step will go onto the
     stack at the start of the draw step"* are the two halves of what a skipped
     step does to CR 603, and RE-1 landed the events they read
     (`GameEvent::StepBegin`) without anything to read them. The first falls
     out — a step that does not begin emits nothing — and the second does not:
     it says the *next* step that begins is where the waiting triggers go, and
     nothing in RE-1 could assert that.

     **Reachability (2026-09-15):** nothing owed — a record for item 6.
     Eon Hub is in `PERFORMANCE_POOL`, so the board is in front of every
     measured game already; what is missing is a trigger to watch. (The
     2026-09-11 verdict said "nothing to build"; re-worded at the post-RE
     audit so the board reads it.)

     **Sized:** two integration tests in item 6's file, ~60 lines, on a board
     `phase_re_cards::eon_hub` plus one upkeep trigger and one untap-step
     trigger. **Item 6's own doc should list them** — the card file's module
     doc records both rulings as "item 6's" and this is the line that says
     where they land.

## Found by A4g — process-stable ids (PR #158, 2026-09-16)

### Item 149 — closed 2026-09-19 by TR-1

Closed by `AbilityIdentity { source, zone_change_epoch, ability, instance }` (`triggers-architecture.md` §3.6): the instance is an ordinal among same-id defs so a later grant does not renumber an earlier one, and CR 603.7h's count (TR-2) ignores it. The window's pair sites were left as pairs on purpose — see the TR-1 archive's note 8.

*Amended 2026-09-22 (the TR-1 review, theme E).* The ordinal renumbered when an earlier grant ended, so provenance ids replaced it: `AbilityId` became `{ definition, grant }`, a granted instance's grant the granting row's `EffectId`, and the identity `{ source, ability }`. The mana window dedupes on the definition and lists what it listed; `LoseAbility` and CR 603.7h's count key on the definition too.

*Original entry:*

149. **Two instances of one ability on one object are indistinguishable by
     id, and the mana window lists them once.** Found by A4g's A/B. A printed
     `AbilityId` is derived from the card name and the def's ordinal (item
     144's decision), so two copies of Citanul Hierophants under one
     controller grant every creature `{T}: Add {G}` under *one* id, where two
     runs of the factory used to mint two. CR 113.10b calls those two
     instances of one ability, and `LoseAbility` already removes both (phase
     LF's test pins it); `enumerate_activatable_mana_abilities` dedupes by
     `(ObjectId, AbilityId)` and offers one candidate where `main` offered
     two, so the random agent's uniform pick lands on a different option in
     the games where the board occurs — 1 of 200 two-seat `stress` games, 5
     of 200 four-seat `performance`, 2 of 200 four-seat `stress`, none on
     two-seat `performance`. Every one of the six shows two Hierophants under
     the acting player at the divergence, and a probe that restored per-copy
     uniqueness read `main` to the digit (`fuzz-record.md`, the A4g block).
     Not an order leak: the hasher arm is identical to the swap arm on every
     counter, under a different hasher seed per round.

     **What it leaves open, for the triggers doc.** A rule that counts *per
     instance* — CR 603.7h's "this ability has resolved for the third time",
     which `AbilityIdentity { source, ability }` exists to carry — cannot
     tell two instances apart by the pair. The index into the effective list
     can, and whether CR 603.7h wants the instance or the ability is the
     triggers doc's question — seam **S3** in `roadmap-v2.md` §3b and A6,
     where the doc's prep will find it — not this item's.

     **Reachability (2026-09-16):** reachable — a policy-visible difference
     (which of two identical candidates the random agent is offered), never a
     wrong answer: activating either instance taps the same creature for the
     same mana, and a "loses" effect removes both, as the rule says.

     **Sized:** nothing owed unless the triggers doc wants per-instance
     identity; then `AbilityIdentity` gains the index (~10 lines) and the 13
     pair sites are read once more.

## Found by TR-1 — the trigger spine (2026-09-19)

### Item 167 — closed 2026-09-22 by the TR-1 review, theme E

Closed by `engine::triggers::LookBackSnapshot` (`triggers-architecture.md` §4.3): a batch whose decided members depart the source of a row that writes ability lists keeps each look-back reader's frame from before it performs, and the close asks look-back arms of that frame and every other arm of the live list. Both signs are fixtures in `phase_tr1_integration_test.rs` — Humility beside a surviving Blood Artist, and a grant whose source dies in the wipe — each red on the pre-fix tree. The snapshot reads the zone map too, so Bridge from Below's graveyard half is covered the day item 173 files Bridge there; its fixture lands with 173. `fuzz-record.md`'s theme E block has the cost and the reachability.

*Original entry:*

167. **A look-back arm on a *surviving* permanent reads its post-event
     ability list.** CR 603.10 looks back "using the existence of those
     abilities ... immediately prior to the event", and the dispatcher does so
     off the CR 603.10a frame for a permanent that *left* (§4.2 leg 2). A
     permanent that stays reads the list it has after the window closed: a
     Blood Artist surviving the wipe that took Humility triggers on the
     deaths, where before the event it had no abilities and should not. The
     frames a record carries cannot answer this — nothing about a survivor is
     recorded — and the memo's stale entry is a cache, not a record
     (`triggers-architecture.md` §15 item 4).

     **Reachability (2026-09-19):** reachable — wrong today: Humility and Blood
     Artist are both pooled, and one state-based check that kills Humility
     and a creature while Blood Artist lives is the board. Rare, and the
     answer is one extra trigger.

     **Sized:** decided 2026-09-22 by the owner (the TR-1 review, theme D): a
     snapshot at the batch's open, built in the review's theme E. The fix as
     first filed was too narrow on two axes.

     - **Sources.** A surviving source can be off the battlefield. Bridge
       from Below in a graveyard under Yixlid Jailer, and one wipe takes
       Jailer and an opponent's creature: neither of Bridge's abilities
       triggers, because CR 603.10 looks back to "the existence of those
       abilities ... immediately prior to the event", and Jailer's effect
       applied then. A read after the wipe finds them restored. So the
       snapshot covers `zone_trigger_sources` as well as the battlefield.
       That half has no board today: Bridge never triggers from a graveyard
       at all (item 173), so its fixture lands with item 173. The sign runs
       both ways — a look-back ability *granted* by a row whose source leaves
       in the same batch existed before the event and not after, and a live
       read misses it.
     - **Events.** CR 603.10a has three classes, and TR-4 adds cards leaving
       a graveyard and visible cards going to hand or library, so "a
       battlefield departure" is today's one class, not the rule.

     **The trigger** is not `trigger_sources` being non-empty — that is an
     `IdMap<ObjectId, EventKindMask>` now, and whether a source's mask meets
     the window is the close's question. It is **the batch departs a
     permanent that carries a registry row** (Layer 1, 3 or 6), decided at the
     batch's open against the registry: the lists before and after the event
     differ only when such a row arrived or left in the same batch, so a
     board without one pays a probe and nothing else.

     **The design:** for every source in both maps, an `Arc` clone of its
     look-back defs when the registry holds a row that can reach a departing
     permanent, matched at the close in place of the live list. ~80 lines;
     the fixture is Humility beside Blood Artist in one wipe; an A/B, because
     it adds work at the open on boards with rows and Humility is pooled.
     Not chosen: TR-4's `LastKnownInformation` growing a per-window frame
     for every source the batch touched, which makes a frame carry a list for
     an object that did not move.

### Item 174 — closed 2026-09-22 by TR-1b

Closed by `GameState::capture_departure_frames` (`triggers-architecture.md` §4.3): between deciding and performing, a batch frames every permanent its decided members take off the battlefield — a zone change from it, a destruction, and every permanent when a player leaves, since CR 800.4a's fourth clause decides what it exiles only after its first two — and the move reads that frame instead of walking the board it finds. A permanent an enclosing batch already framed keeps that frame, so a destruction's move, a nested batch, reads the frame of the event it belongs to. The frames are memo reads, as the look-back snapshot's are. Both halves are fixtures in `phase_tr1b_integration_test.rs`, each in both batch orders and each red on the pre-fix tree in the order the entry named: Humility and Blood Artist (1 trigger with Humility first, 0 is right), and March of the Machines with an artifact it animates (0 with March first, 1 is right). `fuzz-record.md`'s TR-1b block has the A/B.

*Original entry:*

174. **A departing permanent's CR 603.10a frame is captured when that
     member moves, so it depends on batch order.** `perform_zone_change`
     walks the leaving permanent just before its own move (`actions.rs`,
     the `lki` capture), after the batch's earlier members have already
     moved. If an earlier member was the source of an effect on the later
     one, the frame shows the later one after that source left. Destroy
     Humility and Blood Artist as one event: with Humility first, Blood
     Artist's frame has its ability and it triggers on its own death, where
     CR 603.10 reads the list before the wipe and nothing triggers; with
     Blood Artist first the frame is right. The frame's appearance has the
     same fault: an artifact creature under March of the Machines, moved
     after March, is framed as a noncreature artifact, and "whenever a
     creature dies" misses it. Found 2026-09-22 answering the owner's
     question on #178; the survivors' half was item 167.

     **Reachability (2026-09-22):** reachable — wrong today: Humility and
     Blood Artist are both pooled, and a wipe built in
     `battlefield_ids_ordered` order performs Humility first whenever it
     entered first. Proved with a throwaway fixture (deleted): Humility
     first, 1 trigger; Blood Artist first, 0.

     **Sized:** capture each decided departure's frame at the batch's seam,
     the moment §4.3's snapshot is taken, and have the move read the
     captured frame; ~100 lines with a fixture in both orders. TR-1b's
     first commit (`triggers-architecture.md` §4.10), since the dispatch
     audit would report it on the pools.

### Item 122 — closed 2026-09-24 by TR-2a

Closed by `EffectRecipient::EachPlayer(PlayerSet)` and `EffectRecipient::YouAndThatPlayer` (`triggers-architecture.md` §6.6). Each resolves to the seats still in the game, in APNAP order (CR 101.4), and `DrawCards` performs one instruction per player in that order, so CR 121.2c's "the active player performs all of their draws first" is the loop. Alms Collector is `Rewrite::Prevent` plus one rider, "you and that player each draw a card", whose draws carry the replaced event's applied set (CR 614.5). The seed-12345 A/B moved one four-seat `stress` game in 800, by the rider's order and nothing else.

*Original entry:*

122. **CR 121.2c's two-player draw order is unexpressible, and RE-2 shipped its
     first customer.** *"If more than one player is instructed to draw cards,
     the active player performs all of their draws first, then each other
     player in turn order does the same."* Alms Collector's rider — "instead
     **you and that player** each draw a card" — is the first effect in the
     crate that instructs two players to draw, and it is an `Effect::Sequence`,
     which resolves in the order the card's text was written. When the affected
     opponent is the active player the two draws come out backwards.

     **Reachability (2026-09-11):** reachable, wrong today, and only in the
     event log. Alms Collector is registered and not pooled, so no fuzz game
     reaches it; a fixture does, and the order is asserted nowhere because
     asserting it would freeze the wrong answer. It becomes gameplay-visible
     the day item 6 lands "whenever you draw a card", where two players'
     triggers would go on the stack in the wrong order.

     **Sized: not one line.** The facility is APNAP ordering over *an effect's
     recipients*, and `Effect` has no arm that says "these atoms are one
     instruction to several players" — a `Sequence` is CR 608.2c's instruction
     sequencing, which is deliberately *not* reordered. The two candidate
     shapes are a recipient-plural draw primitive
     (`Primitive::DrawCards` with an `EffectRecipient::Filter`-style player set,
     ordered by `apnap_index` at resolution, ~40 lines and one new recipient
     reading) or a `Effect::Simultaneous` arm that sorts its atoms by chooser
     the way `apnap_batch_order` already sorts a batch (~60 lines, and a second
     ordering rule beside the batch's). CR 121.2d's shared-team-turns variant
     is a third leg on whichever lands. **One customer today**, which is why
     neither is built: §8c's "two customers before a leaf", applied to an
     ordering rule rather than a filter.

     **Scheduled (2026-09-15, post-RE audit):** critical-path item 6's
     architecture doc must carry CR 121.2c's recipient ordering —
     `roadmap-v2.md` A6's row says so now — because "whenever you draw a
     card" is the rule's first gameplay reader, and the choice between the
     two shapes below is that doc's to make with its trigger ordering.

     **Narrowed 2026-09-11, at RE-2's close.** Alms Collector's rider turned out
     to be one draw and not two — CR 614.5 forced the affected player's half
     into the rewrite (item 53 there) — so the order is no longer the card's
     text order but a structural one: the replaced event is performed, then the
     rider (§4.1a). That is still not CR 121.2c's, and it is now wrong in a
     narrower and more predictable way: the affected player always draws first,
     where the rule says the active player does. The facility is unchanged and
     so is the sizing.

     → `replacement-architecture.md` §11 item 52. ~~**Owner: RE-6**, which is
     where turn order stops being `(0..n)` because a lost player has left it.~~
     **Re-owned 2026-09-12, at RE-6's close.** RE-6 did make the rotation
     read `player_lost` (`GameState::next_player_in_game`), and that is not
     this item: the facility here is APNAP ordering over *an effect's
     recipients*, which §9's "Out of RE" declines on the same one-customer
     argument as before — Laboratory Maniac's second ruling is the second
     customer, and it is unexpressible for the same reason. **Owner: the
     first each-player draw producer**, wherever Phase 8 lands it; the
     rotation it will sort by exists now.

### Item 172 — closed 2026-09-24 by TR-2a

Closed by `AmountExpr::TriggeringPower` and `AmountExpr::TriggeringToughness` over `TriggerBinding::bound_characteristics` (`triggers-architecture.md` §3.11): the record's CR 603.10a frame when the matched event was the bound object's departure, the live object while it is still where the event left it (CR 608.2h's "determined only once, when the effect is applied"), and nothing otherwise. The leaf refuses by name there, and TR-2b's `departed` frames answer it. Paladin of Atonement gains life equal to its toughness as it last existed on the battlefield, and a value below 0 is no amount.

*Original entry:*

172. **The three bound-fact leaves are `TriggeringObject`, `TriggeringPlayer`
     and `TriggeringAmount`; `TriggeringPower` waits.** §3.4 named four; the
     fourth reads the live object or the frame's power (CR 608.2h), and the
     frame that carries a status is TR-4's. Nothing prints it before Paladin
     of Atonement's toughness read (TR-2) and Heart-Piercer Manticore's power
     (TR-3).

     **Reachability (2026-09-19):** nothing owed — a record for TR-2, whose
     `TriggeringToughness` is the same leaf with the other box.
