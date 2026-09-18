# A4p brief — the `Player` and `Any` arms offer seats that have left the game

Written 2026-09-17 at the close of the A4i audit as the second half of
`a4o-brief.md`; **split into its own file 2026-09-18 when A4o landed and that
brief was deleted**, since a brief consumed by the leg after next cannot ride
in the file the leg before it deletes. Line numbers re-derived against
`2613f77` (A4o's fix moved both files). **Delete this file in the PR that
lands A4p.**

`codebase-state.md` item 160 is the defect and the sizing; `roadmap-v2.md` row
A4p is why it is scheduled here; `plans/handoffs/a4i-review.md` theme I.1 is
how it was found and reproduced. Read those three first.

## Three edits, verified against the tree (2026-09-18)

- `legality.rs:231` — `let players = || (0..game.num_players()).map(RT::Player);`
  gains `.filter(|p| game.in_game(*p))`. That closure feeds both the `Player`
  arm (`:254`) and the `Any` arm (`:257`).
- `targeting.rs:970` (`has_legal_choices`' `Player` arm) and `:974` (its `Any`
  arm, which seeds `found` with `players.len()`) count in-game seats instead of
  the vector's length.
- `targeting.rs:530` `validate_any_target` gains the `in_game` check that
  `validate_player_target` (`:506`, the check at `:518`) already has, with its
  CR 800.4a comment. After the fix, that validator's comment ("not offered at
  CR 601.2c") is true; leave it.

## The fixture, and the measurement

Four seats (`setup_game(4)`; `player_lost[3] = true` is `pub`): both filters
must stop offering `Player(3)`, and `validate_targets` must refuse it for
`Any` — today it accepts, so "any target" damage resolves against a player who
is not in the game.

Prediction: `IDENTICAL` on both pools at **two** seats (a two-player departure
ends the game, CR 104.2a); `differ` on the four-seat `stress` arm, which is the
arm that proves it. Four arms, `plans/fuzz_ab.py --rounds 3 --games 200` at
`--players 2` and `--players 4`, both pools; `main`'s worktree at
`../mtgsim_v2_main` fast-forwarded first. A4o's block in `fuzz-record.md` is
the shape to follow, including the game-by-game attribution of any `differ`.

Item 160 struck to a stub and its body appended to
`plans/archive/codebase-state-closed.md`; row A4p ✅ with the date and PR; its
own `fuzz-record.md` block; this file deleted.
