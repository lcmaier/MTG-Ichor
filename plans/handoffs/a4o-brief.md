# A4o brief — "counter target spell" may not name an activated ability

The prompt for the next leg, written 2026-09-17 at the close of the A4i audit
(PR #162) so a cold session on another machine can start from it. Rides in
PR #162 rather than a PR of its own (owner's rule, 2026-09-15: a brief is never
a PR by itself). **Delete this file in the PR that lands A4o.** A4p's brief is
the short section at the end; do it second, on its own branch.

Every fact below was verified against `d0102bd` plus PR #162's docs on
2026-09-17. Re-grep before trusting a line number.

---

## Read first, in order

1. `plans/codebase-state.md` item 159 — the defect, the fixture board, the size.
2. `plans/roadmap-v2.md` §3a row A4o — why it is first in the slot and why before A6.
3. `plans/handoffs/a4i-review.md` theme I.1 — how it was found and reproduced.
4. `mtgsim/src/engine/targeting.rs` — `validate_spell_target` (line 448's arm),
   `has_legal_choices`' `Spell` arm (line 981), and the `DamageSource` arm two
   below it, which already asks `is_spell` (line 1000) and is the pattern.
5. `mtgsim/src/oracle/legality.rs` — `enumerate_legal_selections_upto`'s
   `Spell` arm (line 259) and the `DamageSource` arm (line 274) beside it.
6. `plans/engineering-practices.md` §3 (the two pools, what an A/B compares)
   and §3.1 (the budget); `plans/fuzz-record.md`'s A4i block for the current
   baseline shape.
7. `CLAUDE.md` "Conventions": a bugfix is shown to fail against the pre-fix
   tree first.

## Verified against the tree (2026-09-17)

- **Three sites, one pattern.** The `Spell` filter is answered at three places
  and none asks `is_spell`: the validator (`targeting.rs:448` →
  `validate_spell_target`, which checks `stack.contains` only), the count arm
  (`targeting.rs:981`, `stack.iter().filter(!= exclude).count() >= n`) and the
  enumeration arm (`legality.rs:259`, `stack().map(RT::Object)`). The
  `DamageSource` arm at each of the three (`targeting.rs:473`, `:1000`;
  `legality.rs:274`) already filters on `stack_entries[id].is_spell`. Copy the
  predicate, not the arm.
- **An activated ability on the stack is a `GameObject` with its source's
  `CardData` cloned in** (`put_on_stack.rs::activate_ability`), which is why
  `Primitive::CounterSpell`'s `change_zone` to the graveyard produces a second
  copy of the card rather than an error.
- **Both halves are pooled.** `PERFORMANCE_POOL` is 91 (`registry.rs:53`);
  Counterspell (`registry.rs:61`) and Merfolk Thaumaturgist are in it. So the
  defect is in every measured game that lines them up, and the fix moves the
  random agent's stream.
- **No registered card uses `Primitive::CounterAbility`** (0 files), so the
  ability side has no consumer and no filter of its own; see the open decision.
- **Counterspell tests exist in three files** — `phase2_integration_test.rs`
  (8 refs), `pre_phase3_integration_test.rs` (8), `phase_lg_integration_test.rs`
  (1) — and **none activates an ability** (0 `activate_ability` refs in all
  three), so no existing test depends on the wrong behavior.
- **The fixture board that reproduces it**, through the public API only:
  `setup_two_player_game()`; `put_on_battlefield(Thaumaturgist, 1)` with
  `controller_since_turn = 0`; `put_in_hand(counterspell(), 0)`; two blue in
  player 0's pool; `activate_ability(1, thaum, idx, &dp)` (target forced);
  `castable_spells(&game, 0)` **offers Counterspell**; `cast_spell(0, cs, &dp)`
  (target forced — the ability is the only other stack object);
  `resolve_top_of_stack`. Ends with the ability object in `Zone::Graveyard`
  and player 1's graveyard naming "Merfolk Thaumaturgist" while the real one
  is still on the battlefield.
- **Atoms.** `ATOM-115.5-001` exists (`session-1.md:1982`) and is the
  self-target rule, not this. CR 701.6a is marked ALREADY-IMPLEMENTED in
  `session-7a.md:352` with no atom of its own, and CR 115.1's "a spell" is the
  rule actually broken. Run `python plans/specdb.py orphaned` before claiming
  anything; expect this to be a `COVERS-PARTIAL` at best, or no atom — say
  which in the PR.
- **The `main` worktree at `../mtgsim_v2_main` is at `aafb79a`, two merges
  stale.** Move it to `main`'s tip (post-#162) before building the baseline
  arm, or the A/B compares against a tree without A4i.

## Decisions the row leaves open — name them in the PR, do not decide silently

1. **Where `is_spell` lives.** Three copies of one predicate at three arms is
   the shape that cost item 8 a redesign. A `GameState::is_spell_on_stack(id)`
   helper that the `Spell` *and* `DamageSource` arms both call is the smaller
   surface; the alternative is three inline copies matching the existing
   `DamageSource` ones. Either is fine; say which and why.
2. **The ability side.** "Counter target activated or triggered ability" (Stifle's
   class) needs a `SelectionFilter` that is the complement of this one, and
   `Primitive::CounterAbility` already exists with no filter to reach it.
   **Not this PR** — no card — but the fix should not make it harder: the
   predicate above is the one that filter negates.
3. **Whether a triggered ability is caught for free.** A6's triggers will be
   the same ephemeral stack objects with `is_spell: false`, so the fix covers
   them if A6 keeps that shape. Write one sentence in the A6 doc's inbox
   (`roadmap-v2.md` A6 row, or a line on item 159 when struck) so the doc
   inherits the constraint.

## Measurement

- **Four arms, both pools, both seat counts.** `main` (post-#162 tip) and
  `fix`, each at `--players 2` and `--players 4`; `plans/fuzz_ab.py` with
  `--rounds 3 --games 200`, `performance` and `stress`.
- **Prediction: `differ` on both pools at both seat counts.** The fix removes
  Counterspell from `castable_spells` whenever only abilities sit on the stack,
  which changes the candidate action list and so the stream. That is the
  honest reading, not a failure. Confirm the cause the way
  `ab-differ` is confirmed: diff the `--dump-events` logs game by game, and the
  first divergence in every diverging game should be a priority pass where an
  ability was on the stack (memory: mask the three things before comparing).
- **Cost.** `µs/decision` inside §3.1's 2.5-point budget; nothing about the
  fix is hot, so the prediction is inside noise.
- **Re-record** both tables in `fuzz-record.md` as a new block at the top,
  since the pool is unchanged but the stream moved (§3's rule).

## Binding rules

- Show the regression failing against the pre-fix tree first. In a worktree,
  set the fix aside as a WIP commit rather than a bare `git stash`.
- The regression **casts from hand** (`cast_spell`), the way item 152's did;
  a staged `ResolutionContext` cannot check the question. Exact mana, and wrap
  the provider in `ManaWindowStop` if the board has mana abilities.
- Register no card; `PERFORMANCE_POOL` stays 91.
- Strike item 159 in `codebase-state.md` with the closing date and the PR;
  regenerate `state-of-play.md` with `--write`; delete this file.
- `cargo build --all-targets` zero warnings; `cargo clippy --all-targets -- -D warnings`;
  all six `check_*.py`; `fuzz_games` deterministic across three
  `MTGSIM_HASH_SEED`s.
- Branch off `main` after #162 merges: `targeting/a4o-spell-is-a-spell` or
  similar. PR body in the A4i shape: the finding, the A/B table, the gates.

## Exit criteria

- The fixture above passes with the ability object still on the stack after
  Counterspell resolves (it was never a legal target, so `castable_spells`
  does not offer Counterspell and the test asserts that first).
- Counterspell's existing 17 test references still pass unchanged.
- A/B run at four arms, `differ` explained game-by-game, tables re-recorded.
- Item 159 struck; row A4o marked ✅ with the date and PR; this file deleted.

---

## Then A4p — departed seats (second leg, own branch)

`codebase-state.md` item 160; row A4p. Three edits, verified 2026-09-17:

- `legality.rs:231` — `let players = || (0..game.num_players()).map(RT::Player);`
  gains `.filter(|p| game.in_game(*p))`. That closure feeds both the `Player`
  arm (`:251`) and the `Any` arm (`:254`).
- `targeting.rs:956` (`Player` arm) and `:961` (`Any` arm, which seeds `found`
  with `players.len()`) count in-game seats instead of the vector's length.
- `targeting.rs:530` `validate_any_target` gains the `in_game` check that
  `validate_player_target` (`:518`) already has, with its CR 800.4a comment.
  After the fix, that validator's comment ("not offered at CR 601.2c") is
  true; leave it.

Fixture at four seats (`setup_game(4)`, `player_lost[3] = true` is `pub`):
both filters must stop offering `Player(3)` and `validate_targets` must refuse
it for `Any`. Prediction: `IDENTICAL` on both pools at two seats (a two-player
departure ends the game, CR 104.2a); `differ` on the four-seat `stress` arm,
which is the arm that proves it. Item 160 struck, row A4p ✅, its own
`fuzz-record.md` block.
