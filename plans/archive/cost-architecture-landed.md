# cost-architecture.md — landed phases, evicted

**A record of finished work, not a plan.** Every section here sat under a ✅
heading in `plans/cost-architecture.md` and was moved out on 2026-09-11 once the phase had shipped,
leaving the heading, a stub and a pointer in the live doc. Nothing here is
owed and nothing here should be acted on; what each section is *for* is the
reasoning — the design as sized, what the building changed, the measurement.
`check_state_of_play.py` reads the ✅ headings that stay, and fails when a
landed section keeps more than 40 lines in the live doc
(`engineering-practices.md` §4). Later phases are appended by the PR that
lands them.

#### CM-4 — the mana window and the payer — ✅ 2026-09-08

*Evicted 2026-09-11 from `plans/cost-architecture.md`, where the heading and a stub remain.*


Built as §3.4 and §6 say, with two corrections the building forced.

**One payer became two decorators.** §3.4 named a single `ui::AutoPayer<D>`
answering four prompts, and §6 sized it with a flag. Both are wrong for the
same reason: the four prompts do not belong to one client. `ManaWindowStop`
and `AutoPayer` toggle independently — a human turning off auto-pay wants the
window to keep offering, an agent without a stop has only
`WINDOW_ACTIVATION_CAP` — and a scope enum inside one payer would have been a
closed enumeration of the subsets of something that already composes. Clients
compose a stack; the invariant is one decorator per `ChoiceKind`.

**And the sacrifice prompt left the payer**, on a criterion rather than a
judgement call: a payer answers a prompt when every legal answer leaves the
same game state except for mana (§3.4). That criterion also placed §8 item 4 —
CR 732.1's reversal offer goes with critical-path item 6, not with the payer.

The residual 601.2g reading is closed: a component reduced to nothing opens no
window, because the lock-in discards the history that would distinguish it from
a printed `{0}`. Both directions are tested and both tests fail with the gate
removed.

**No new card.** §6's claim that §3.11's step 3 builds out of CM-3's registered
five held — Ironworks, Mind Stone and two spare artifacts — so
`PERFORMANCE_POOL` stays at 73 and `engineering-practices.md` §3's table is not
re-recorded. Its `Memo hits` row is the one number the tree now disagrees with
(+0.8% on both pools, from the extra enumeration the window costs); recorded in
`codebase-state.md`'s CM-4 block rather than re-recorded here, since the pool
did not move.

**§6's own prediction was wrong, in the opposite half from CM-3's.** CM-3
predicted no new pooled path and was wrong about the pool; CM-4 predicted the
same and is wrong about the engine. The counters moved — 1 game in 200 on
`performance`, 8 in 200 on `stress` — and a fourth binary attributed all of it
to the decorator rather than to either engine change: with the stack dropped,
the 601.2g gate and the removed early return together leave `stress`
byte-identical and `performance` different by two memo hits and no game-state
counter. The board is `codebase-state.md` item 83, a source tapping itself for
mana inside its own window; the payer stops asking a question no mana ability
could answer, which `main` did not. Traced to Chainbreaker once and Mind Stone
eleven times, by name, in a debug build.

**No trace page** (`engineering-practices.md` §7): no read is answered
differently, only asked by a different party. And no seam for recording why the
payer chose what it chose — its answers are a pure function of
`(ChoiceKind, remaining_cost)`, already in the prompt stream, and a decorator
buffering anything to explain itself would be a decision site holding
outcome-bearing state off `GameState`. A4c's four emit points stay four.
