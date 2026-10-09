# Handoff — the triggers midpoint audit (A6k)

**Opened 2026-10-08**, after TR-3b (PR #234) merged and before TR-3c, under
`engineering-practices.md` §9's midpoint rule. This file is the plan and the
contract: the board lists it under half-finished work until the last PR
lands, each PR updates §4's status, and **the PR that lands the last one
deletes it** and writes `codebase-state.md`'s `### … — audited` heading.

Docs and small fixes only. A finding is fixed in one of the PRs below or
becomes a numbered item with reachability, a size and a slot. Existing
numbered items are not fixed here; where the read sharpened one, it carries
a dated paragraph.

## 0. Where things stand

- Branch `audit/triggers-midpoint`, off `main` at `0ac5c98` (#234's merge).
  The TR-3b dev GUI PR, which the brief put first, is not open yet
  (`tr-3b-devgui-notes.md` is still here); it touches `devgui/` only, and
  item 234 below gives it one engine half.
- Every count below was read from the tree on 2026-10-08. A later PR
  re-reads what it touches.

## 1. Pass 1 — hygiene (§9's pass 3)

### 1.1 The comment census, reproduced first

`engineering-practices.md` §2.1's third record carries the readings beside
2026-09-15's. What the reproduction found:

- **The wide tier reproduces exactly**: §2.1's grep over comment lines in
  `.rs` files and every line of live `plans/` (archive and corpus out) reads
  456 in `mtgsim/src` (133 in `cards/`), 140 in `tests/` and 1,049 in
  `plans/` at `949a519`, the first record's tree.
- **The second record's per-file count reproduces exactly**: lines starting
  `//`, test modules included, give its PRs' 3,235 → 2,798, 10,164 → 9,722
  and 521 → 492, and #147's −564.
- **Its tree total does not.** "15,619 comment lines" in non-card, non-test
  `src/` matches no definition tried (16,415 with test modules, 15,245
  without, 14,889 cut at the first `#[cfg(test)]`, all at `1e80147`), and
  the 57 touched files alone summed 17,278. So 15,806, the touched files
  after the sweep, is not a tree total: the brief's comparison of 20,105
  with it compared two instruments. The tree-wide row is 16,415 → 15,034 →
  21,295.
- **The tight tier's regex was never written down.** This run's reads 63
  (34 undated) at `949a519` against the record's 58 (39), and 19 undated at
  `1e80147`, the record's after-reading.

Today: wide 431 / 149 / 217 / 1,864 (non-card `src` with `bin` / cards /
tests / live plans); tight 82, 53 undated; 21,295 comment lines in 71,176
non-card, non-binary lines (29.9%, from 32.5% after the sweep); 107 blocks
over four lines narrate history (1,814 lines), against 80 (1,311) after the
sweep and 251 (4,380) before it; inline blocks over four lines 1,918, against
1,582 and 2,854.

**Sized: one PR.** At the second sweep's yield (16,415 → 15,034, 1,381 lines
out of the tree), today's candidates are about half its size: ~600–900 lines
out, a ~1,200–2,000-line diff, inside §4's band. Largest first:
`replacement/pipeline.rs` (321 history lines), `types/replacement.rs` (180),
`state/game_state.rs`, `engine/targeting.rs`, `types/effects.rs`,
`engine/actions.rs`, `replacement/gather.rs`. The undated tight-tier list is
the floor again ("counted", "measured", "about one" describing the code),
but for `layers/types.rs`'s Layer 3 census and its undated clone measurement.

### 1.2 `TODO`s

One in `mtgsim/src` (`layers/board.rs`, the other P/T counters), owned by
`codebase-state.md` "Before card breadth" item 3, which is live. None in
`devgui/src`. 2026-09-15 re-owned twelve.

### 1.3 Clippy, each allow re-counted with the allow removed

A scratch copy of the crate, the five `= "allow"` lines dropped, `cargo
clippy --all-targets` with a target directory of its own. Nothing else
warns.

| lint | `Cargo.toml` says | 2026-10-08 | what moved |
|---|---:|---:|---|
| `too_many_arguments` | 11 | 11 | the count held and the set did not: one of today's is the trigger code's `match_def` (TR-1, 2026-09-19) |
| `large_enum_variant` | 4 | 6 | beside the four it names, `explain::StepResult` and `CopyException` |
| `new_without_default` | 3 | 4 | beside the three it names, `RecorderHandle::new` |
| `type_complexity` | 3 | 7 | today's: the `board.rs` and `pipeline.rs` tuples, `GameEvent::objects_named_as_announced`'s, the scenario writer's, the registry's `IN_DEVELOPMENT`, two in tests |
| `needless_range_loop` | 3 | 4 | beside the three it names, `mana_supply.rs`'s loop over the six mana types in `less` |

Each new site is the shape its lint's reason already allows, so the five
stay allowed. Four of the five counts went stale with nothing to say so,
§2.1's failure exactly: the sweep PR dates them.

### 1.4 Helpers built more than once

`plans/similar_functions.py` reads 1,146 functions, 26 near-copy pairs and
7 same-shape pairs, the brief's numbers. The five engine pairs:

| pair | disposition |
|---|---|
| `object_matches_filter` / `_for_instance` | the first is the second with no identity; item 229 (TR-3c) changes the signature, and makes it one call (its dated line) |
| `capture_named_frame` / `capture_departure_frame` | one function behind one guard: item 236 |
| `ManaPool::can_pay` / `can_pay_with_context` | item 215's (A6e) |
| `playable_lands` / `castable_spells_with` | kept: each is one line over its own check, and the shared half is a walk over a hand |
| `TriggerEvent::subject_of` / `player_of` | kept: one shape by design, a projection each over the same pairs; item 231 changes both |

The board-query greps (item 215's dated line has the sites): the permanents
a player controls are still built inline at five sites beside
`oracle::board::permanents_controlled_by`, one more than item 215 counted;
the opponents at two, one of them a private helper; creatures only inside
the blockers' two copies (item 188's `can_block` candidate); untapped lands
nowhere, the `!entry.tapped` reads being tap-cost checks.

## 2. Pass 2 — the trigger code's shapes

The surface the brief named, 5,294 lines in 11 files, read whole.

### 2.1 Positions

| list | its entries carry | a position across a call or a mutation |
|---|---|---|
| `players` | the index is the `PlayerId`; a player who leaves stays | the index is the identity |
| `stack` | `ObjectId`s | none in the engine: `last()`, and removal by id. The scenario loader's `picked_at` holds a depth across the cast it scripts, which CR 601.2 keeps still |
| `exile`, `command` | `ObjectId`s | none: `position` then `remove` in one statement |
| `last_turn_began`, `player_lost` | indexed by `PlayerId` | the index is the identity |
| `turn_queue` | `ExtraTurnId` | none: `pop` (CR 500.7), found by id |
| `pending_triggers` | `TriggerSeq` | none: placement asks by `TriggerSeq`; the ordering answer indexes the prompt's own list |
| `delayed_triggers` | `DelayedTriggerId` | none: a match carries the id, found at use |
| `until_returns` | **nothing** | none: `take_returns_due` partitions; `ui::waiting` lists it by position, for display — item 234 |
| `look_back_snapshots` | the `BatchId` taken for | inside one dispatch only, into the window's own partition |
| `departure_frames` | the `ObjectRef` framed | `execute_batch_inner`'s `frames_from` mark, held across the batch's performance: a stack, safe while nested batches truncate to their own later marks and nothing else removes |

Not one of the twelve: `EventWindow` finds a record by `EventSeq`, the
position derived at the lookup from the first held record's number.

### 2.2 Facts the rules fix at a moment, recomputed later

- The window's close is after its riders, and the matcher reads the board
  then: item 175's two readers, and three more (its dated paragraph).
- A look-back trigger's intervening "if" reads the live source: five leaves,
  where `triggers-architecture.md` §6.1 said three, and a delayed trigger's
  reads its source by id across a CR 400.7 move. §6.1 rewritten in place;
  TR-4a's.
- "Its owner" of a delayed trigger and an until return: item 235.
- Items 168 and 177 (an identity's epoch at dispatch) are known.
- Kept: `history_update` reads a cast spell's types and the attacking player
  as an unbatched record is emitted, which is the event's own moment.

### 2.3 Matching outside `occurrences_matching_arm` and the bound reads

- `match_delayed` repeats `match_def`'s arm loop, limit and "if" over the
  shared matcher. Kept: the steps it skips are the rules' (a delayed trigger
  is no object's ability, so CR 603.2f's visibility and CR 113.6 do not
  apply), and the one shared step that differs is §6.1's line above.
- `until_has_happened` matches a leaving by hand: item 230.
- `history_update` matches "dies", a gain, a loss, a cast and a draw to
  count them. Kept: a count, not a trigger, each one line, and "dies" reads
  the record's frame as the arm does.
- `binding::departure_frame` and `dispatch::frame_of` both read a record's
  CR 603.10a frame, the first checking the subject. Kept.
- The projections' and the matcher's wildcards: item 231.

### 2.4 Per-event cost

callgrind under WSL, `prof_arm.sh` on `close_out.py`'s board (`--games 20
--seed 12345 --pool performance --players 4 --deck-size 100 --life 40`,
`MTGSIM_HASH_SEED=1`, one thread). The branch is `main`'s tree, so the one
profile is `main`'s; the code-fix PR reads its own arms.

- **7,333,785,806 instructions**, native and valgrind counters identical,
  565 decisions a game: **0.6490 M per decision**, the first reading on the
  106-card pool (TR-3b's engine arm read 0.6434 M on 104).
- **The trigger surface:** `dispatch_batch` 4.48% inclusive and the
  unbatched dispatches 0.12%; its own instructions about 0.3%. 3.39% is
  `compute_characteristics` called from `find_matches` (3,649 calls): the
  first layer read after a batch. A throwaway probe recorded each board walk
  the dispatcher started and whether the next read outside it came at the
  same layer epoch: of 1,101, 1,062 did and 39 did not (3.5%, about 0.1% of
  the instructions). The dispatcher pays a walk the next reader would pay.
  Its scans that grow with the board cost nothing that shows:
  `battlefield_readers` 0.06% over 44,317 calls, `named_trigger_carriers`
  0.01%, `candidates_now` 0.01%.
- **The one per-event scan that grows with the board is outside it:** item
  138's lever 6, the timestamp sorts, 1.45 G inclusive (19.8%), the
  state-based check's share 10.5% (its dated line).

## 3. Findings

`codebase-state.md`, "Found by the triggers midpoint audit (2026-10-08)":

| item | what | reachable | slot |
|---|---|---|---|
| 231 | the projections and the matcher end in a wildcard; `occurrences_of` has never had a caller | no | the code-fix PR, before TR-4's arms |
| 232 | `bound_reads` files `ExileUntil` as reading nothing | no | the code-fix PR |
| 233 | three prompts in the trigger code hold what they decide off `GameState` | no | the owner's call (§5) |
| 234 | CR 610.3's returns have no id and no record | not wrong | the TR-3b dev GUI PR's engine half |
| 235 | a delayed trigger's "its owner" falls back to its controller | no | the code-fix PR |
| 236 | the frame capture written twice, twice | not wrong | the code-fix PR, ahead of TR-4a |

Sharpened in place, each with a dated paragraph: items 40 (pointer to 233),
138 (lever 6's reading), 175 (three more readers), 215 (the board-query
sites), 229 (the filter twin). Rewritten in place:
`triggers-architecture.md` §6.1's three leaves.

## 4. The PRs

| PR | what | status |
|---|---|---|
| 1 | this plan, the findings, §2.1's third record, the A6k row, §9's midpoint rule; docs only, a draft | open |
| 2 | the comment sweep (§1.1), with `Cargo.toml`'s five counts dated, and the stale comments pass 2 met: `GameEvent`'s "will be handled by a replacement effect registry" and its pointer to module docs that do not exist, `AbilityResolved`'s link to a `StackObjectResolved` that does not exist and its "an activated ability", `dispatch.rs`'s "before this phase" | — |
| 3 | items 231, 232, 235 and 236, each shown failing on the pre-fix tree first (231 by a scratch arm that compiles there); `fuzz_ab.py` against `main` on both pools at two seats and four, `IDENTICAL` predicted | — |
| last | deletes this file, writes the audited heading | — |

Anything that changes how games play gets its own PR and a
`fuzz-record.md` block; nothing found so far does.

## 5. Open for the owner

- **Item 233: when.** The three prompts are unreachable until something
  forks inside a round, and item 40's two violators wait for the first
  fork-based harness. What is cheap now is the rule for the prompts TR-4–TR-7
  add (CR 603.3c's modes, CR 603.5's "may"): a prompt in detection,
  placement or a return keeps what it has decided on `GameState`, the
  drain's shape. Recommended: the rule into `triggers-architecture.md` §5
  with PR 3, the three fixes at item 40's slot.
- **The midpoint rule.** Written into §9 as the brief stated it; strike it
  there if not.
