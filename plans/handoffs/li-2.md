# Phase LI — where to resume after LI-1 (2026-09-06)

LI-1, the board-wide sequential pass, is on `layers/li-1-board-pass` as a
PR against `main`. The plan for the whole phase is `layers-architecture.md`
§13b, written before LI-1 and carrying LI-1's as-built record; this file is
the shorter "what to pick up next" and is deleted when LI-3 lands.

## Resume with LI-2 — CR 613.8a/b/c

Branch `layers/li-2-<name>` from `main` once #LI-1 is merged (or stacked on
`layers/li-1-board-pass` if it is not). §13b's LI-2 section is the plan;
the pieces are numbered there. What LI-1 left as hooks, in `engine/layers/board.rs`:

- `apply_layer(game, board, layer_index, apps)` applies in key order. LI-2
  replaces its body with the loop and the function becomes
  `resolve_order_within_layer` — it applies as it orders (CR 613.8c), so
  §9's reserved `Vec<EffectId>` return is superseded; §9 says so.
- `Application { kind, timestamp, tiebreak }` with `Application::is_cda()`
  for 613.8a(c), and `Kind::Row { effect, would_be } | Kind::Own { object,
  cda, modification }`. Every kind reads and writes through the same two
  functions: `affected_members` (what it applies to) and
  `static_ability_still_exists` (existence, rows only; a CDA's existence
  check is inline in `apply_one`).
- The hypothetical check (§13b decision 3) clones `board.frames[m]` for
  each member `m` that B affects, applies B to the clone through
  `compute::resolve_modification` + `apply_resolved`, and re-evaluates A's
  read. The channel sets (what a modification writes, what a read reads)
  are the static check that prunes almost every pair; `SetSubtypes` on a
  land also writes abilities (CR 305.7, `land_types::apply_set_subtypes`).
- `FilterPlayers::for_row(effect, game, board, layer_index)` is how a row's
  "you" is resolved; `Board::frame_of` is the one read of another object.

Tests LI-2 owes (§13b lists them; `specdb.py show` each atom first):
Urborg + Blood Moon both orders; Ashaya + Blood Moon both orders; the
613.8b loop fixture on creature types (`SetSubtypes` both ways, so the two
orders differ); the 613.8c chain; ATOM-613.8a-003; **7c's CR 613.6 test**
(`codebase-state.md` "Before Layers" 7c); and the **row-older-than-counter
order** of `test_a_counter_older_than_a_power_reading_row_applies_first`
in `tests/phase_li_integration_test.rs`, which LI-1 deliberately left
unpinned because 613.8 changes it (the row depends on the counter).
ATOM-613.8-001's "all activated abilities of other creatures" is not
buildable — claim partial or nothing.

Cards: **Urborg, Tomb of Yawgmoth** (Legendary Land; registered, and in
`PERFORMANCE_POOL` beside Blood Moon) and **Ashaya, Soul of the Wild**
(registered, `stress` only). Both Scryfall-verified 2026-09-06 — the texts
are quoted in §13b. `phase_ld_cards::urborg_effect` stays the Enchantment
fixture the CR 305.6 tests rest on. Rootpath Purifier is *not* LI-2's card:
its library clause needs layers item 9.

## Then LI-3 — conditional statics

§13b's LI-3 section: the `Effect::Conditional` lowering arm, an evaluator
in `engine/layers/condition.rs` over the live board, the clause in the
existence check, Kird Ape ({R}, "This creature gets +1/+2 as long as you
control a Forest" — verified 2026-09-06) in `PERFORMANCE_POOL`, and the
Rune-of-Flight shape as a named fixture. Rune of Flight itself waits on
item 6 (its draw is a trigger).

## The A/B protocol, as run this session

- Rebuild `main`'s binary in the worktree before the sitting:
  `cd ../mtgsim_v2_main/mtgsim && cargo build --release --bin fuzz_games`.
  The binary there was a day stale and every game differed from its first
  draw; §3 now says so.
- `python plans/fuzz_ab.py --arm main=<exe> --arm new=<exe>` alone on the
  CPU (~20 s for two arms). The 50-game §3 rows come out of the same run.
- Attribution: 40-game `--dump-events` per pool per binary at `--threads
  1`, then a per-game masked diff (the three masks in the memory note:
  8-hex id prefix, full UUIDs, the LKI annotation on zone changes). LI-2's
  "engine, pool unchanged" arm will differ from `main` on any game where
  Blood Moon met Urborg in the pool; count those games and say why.
- Three serial runs per pool outside `=== Timing ===` for determinism.
- `specdb owed` — the default scope. `owed --phase LH` in LH-2's record
  matched no atom and was vacuous; do not repeat the claim.

## Phase exit (unchanged from the prompt)

Item 8 and 7f closed in Deferred Migrations; 7c's test written (LI-2);
§5.2 rewritten (done in LI-1); `resolve_order_within_layer` real (LI-2);
RS-3b unblocked in `cant-effects-architecture.md` §7.1; item 7 ✅ in
`CLAUDE.md` within its 200 lines (it is at 200 today — a line has to be
traded); `roadmap-v2.md` A3 ✅; `check_state_of_play.py --write`; a trace
page for the phase (`engineering-practices.md` §7 lists item 7). Delete
this file in the PR that lands LI-3.
