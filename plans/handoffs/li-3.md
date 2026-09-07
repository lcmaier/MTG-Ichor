# Phase LI — where to resume after LI-2 (2026-09-06)

LI-1 (the board-wide sequential pass, #103) and LI-2 (CR 613.8a/b/c,
`layers/li-2-dependency`) are in, or on a PR against `main`. The plan for
the whole phase is `layers-architecture.md` §13b, carrying both as-built
records; this file is the shorter "what to pick up next" and is deleted
when LI-3 lands.

## Resume with LI-3 — conditional statics

Branch from `main` once the LI-2 PR is merged (or stack on
`layers/li-2-dependency` if it is not). §13b's LI-3 section is the plan; the
pieces are numbered there. What LI-2 left as the seams LI-3 touches, in
`engine/layers/board.rs` and `state/game_state.rs`:

- `static_ability_still_exists(game, board, effect, layer_index)` finds the
  generating ability on the source's live frame. LI-3's clause goes there: if
  the ability's body is `Effect::Conditional(cond, inner)`, evaluate `cond`
  against the live board then and there, and the effect exists iff it holds.
  `row_affected` is the one caller; `Affected::Gone` is what "the condition
  fails" becomes, and nothing else in the loop needs to know.
- The dependency check already treats existence as a read of the source's
  frame (`Reads::source`, `Channels::ABILITIES`). A condition reads more —
  `ControlPermanent(filter)` reads every member's frame through the filter's
  leaves, with the source's controller as "you" — so `effect_channels`
  gains: for a conditional ability, `filter_reads(cond's filter, ..)` into
  `reads.members` (and `reads.source |= CONTROLLER` for a static). Without
  that a condition that flips when another effect applies would be settled
  "independent" by the static check and never reach the hypothetical. Kird
  Ape under Blood Moon does *not* need it (7c reads layer 4's output, two
  layers) — the Rune-of-Flight fixture, a layer-6 condition on the host
  beside a layer-6 grant, is where it bites.
- `GameState::static_ability_atoms` asserts on `Effect::Conditional`; the
  lowering arm lowers `inner`'s atoms exactly as today (piece 1). Rows carry
  no condition — it lives on the ability, which the existence check already
  fetches (decision 5). `lookahead::would_be_rows` goes through the same
  function, so the CR 614.12 look-ahead needs nothing.
- `engine/layers/condition.rs` is new: `holds(cond, game, board, source,
  layer_index) -> bool` for the eight leaves, plus the one host predicate the
  Rune shape needs. `Board::frame_of`, `Board::battlefield_ids` and
  `FilterPlayers::for_row`-style resolution are the reads it has; nothing
  it needs is private to `board.rs` today except `frames`, which
  `frame_of` covers.

Tests LI-3 owes (§13b lists them; `specdb.py show` each atom first): Kird
Ape with and without a Forest, and under Blood Moon beside a Breeding-Pool
shape (a nonbasic Forest stops being a Forest at layer 4, the Ape loses its
bonus at 7c — no dependency involved); the Rune-of-Flight fixture in both
orders against Humility; a condition whose flip is itself a dependency (so
the `effect_channels` clause above is what a test pins). Kird Ape's text,
verified 2026-09-06: {R}, 1/1, "This creature gets +1/+2 as long as you
control a Forest." Kird Ape goes in `PERFORMANCE_POOL`: the first row whose
existence is a condition, a new path in the check.

## The A/B protocol, as run this session

- Sync and rebuild `main`'s binary before the sitting: `git -C
  ../mtgsim_v2_main pull --ff-only && (cd ../mtgsim_v2_main/mtgsim && cargo
  build --release --bin fuzz_games)`. The worktree was four commits behind
  `origin/main` this session; a worktree synced to a commit is not a binary
  built at it.
- Three arms, not two: `main`, the new engine with the registry and pools
  unchanged (patch `registry.rs`, build, copy the exe aside, restore), and
  the shipped tree. The middle arm is the one that proves the engine change
  alone: for LI-2 it reproduced `main` byte for byte outside `=== Timing ===`
  on both pools once the new `Dependency checks` line was ignored —
  `fuzz_ab.py` reports "differ" for a new counter row, so diff the raw
  outputs under `--out` minus that line.
- `python plans/fuzz_ab.py --arm main=<exe> --arm engine=<exe> --arm
  new=<exe> --out <dir>` alone on the CPU (~35 s for three arms). The
  50-game §3 rows come out of the same run.
- Attribution: 40-game `--dump-events` per pool per binary at `--threads 1`,
  then a per-game masked diff (full UUIDs, 8-hex id prefixes, the `[was ..]`
  LKI annotation). A pool change diverges every game from its first draw —
  the registry's name list changes `random_deck`'s stream — so the middle
  arm is the only one whose diff attributes anything.
- Three serial runs per pool outside `=== Timing ===` for determinism
  (`fuzz_ab.py` covers `performance`; run `stress` by hand).
- `specdb owed` — the default scope, diffed against `main`'s output, which
  is the only way it says anything for a Phase 5-Layers PR.

## Phase exit (what is left after LI-2)

Done: item 8 closed in Deferred Migrations (LI-2); 7c's test written (LI-2);
§5.2 rewritten (LI-1); `resolve_order_within_layer` real (LI-2); §9 updated;
§15.2 item 3 closed. Left for LI-3: item 7f closed; RS-3b unblocked in
`cant-effects-architecture.md` §7.1; item 7 ✅ in `CLAUDE.md` within its 200
lines (it is at 200 today — a line has to be traded); `roadmap-v2.md` A3 ✅;
`check_state_of_play.py --write`; the trace page for the phase
(`engineering-practices.md` §7 lists item 7 — the four-card board through
`board::next_ready` is its natural trace, with the LI-1 page's
Humility + Hierophants walk beside it). Delete this file in the PR that
lands LI-3.
