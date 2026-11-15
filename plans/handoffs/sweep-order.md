# Handoff — what order a battlefield sweep walks in

**Opened 2026-11-14**, from #326's review: the owner asked why the battlefield's
ids need ordering at all ("every time I see it it feels like a mistake"), and
turned down the answer that a `DecisionProvider` picks by index. This note is
read-only research: no engine change rides with it. It is deleted when the
work it decides lands.

**Revised 2026-11-14 at the owner's first read.** The first draft recommended
keeping the sort because a saved scenario reloads and sweeps alike only under
timestamp order. The owner: that is "the same 'tail wagging the dog' mistake
that got us here in the first place — bend the auxiliary system to fit the
engine, not the other way around", to be a standing rule (queued in
`process-pr.md`). With the scenario's reason struck, nothing in the engine
needs the sort, and the recommendation is C.

**The question.** A sweep of the battlefield whose order no rule names (the
state-based checks, the legal actions, the trigger sources, a "destroy all
creatures") walks in CR 613.7 timestamp order today, through
`GameState::battlefield_ordered`, `battlefield_ordered_where`,
`battlefield_ids_ordered` and `in_battlefield_order`. Since CL-1 the table
itself walks in id order, the same in every process and in every fork. Would
that order do?

## The options, retrofit cost first

| | **A. Keep the sort** (today) | **B. Drop it where nothing reads the order** | **C. Id order everywhere** |
|---|---|---|---|
| Retrofit | none | two sweeps, the only ones whose result is a count or an `any`: `a_combatant_strikes_first` in `engine/combat/steps.rs` and `AmountExpr::CountOf` in `engine/resolve.rs` walk the table under an `ORDER-FREE:` tag, each sorting first when the trace is on, as `battlefield_ordered_where` does. ~15 lines | below |
| Saves | nothing | two sorts, too small to read | the sort, which #325 left under half a point of instructions on the Commander board (`fuzz-record.md`, the ordered-sweeps block), and one rule fewer to hold: a battlefield walk is the table's walk. Not the `Vec` each sweep collects: item 77 found the allocation, not the sort, is the cost, and a sweep that performs actions as it goes snapshots the board either way |
| Moves | nothing | no gameplay row; the `any` stops after a different number of layer walks, so the diagnostics rows move | every option list the battlefield feeds: the legal actions, attackers and blockers, mana sources, the `OrderTriggers` and CR 616.1 candidates, each simultaneous batch's member order and so its log lines. The A/B moves on every row the bots touch; that alone is no design cost |

**Recommended: C.** No rule and no engine reader needs the sort (the next two
sections); what needed it was the scenario's round trip, and the scenario
adapts.

**C's retrofit, engine first:**

- **The sweeps.** `battlefield_ordered`, `battlefield_ordered_where` and
  `battlefield_ids_ordered` become the table's walk, and their audit and trace
  branch go with the sort. Whether the three keep their names or callers walk
  `battlefield` directly is the build's naming call.
- **The three hash-keyed source sets still need an order.**
  `trigger_sources` (an `IdMap`), `trigger_multiplier_sources` and
  `cost_modification_ability_sources` (`IdSet`s) iterate in hash order, which a
  rebuild can change (CL-1's rule 3). `in_battlefield_order` and the cost
  gather sort them by timestamp today; under C they sort by id. So `ObjectId`
  gains `Ord`, reversing `clone-architecture.md`'s "no `Ord` on purpose", or
  those three sort on `raw()`. `Ord` is the honest one: id order becomes an
  order the engine names.
- **The gate and its words.** `check_invariants.py`'s unordered rule keeps
  the hash-keyed fields and drops `battlefield`; its message and the 35
  `ORDER-FREE:` tags reworded where they cite timestamp order. CLAUDE.md's
  determinism section, `clone-architecture.md`'s order rule, and the eight
  architecture docs plus `engine-map`, `codebase-state`, `glossary`,
  `backlog`, `elisions` and `id-hasher` where they name the sweeps.
- **Tests (the probe below): 8 in debug.** The four `ordered_sweeps` unit
  tests go with the sort. The two CR 613.7e reattachment tests assert where
  the permanent sorts in a sweep; they assert its timestamp instead, which is
  the rule's content. The board-scale token test reads a list in sweep order
  and is re-pinned or reads by identity.

**The scenario adapts (the auxiliary side):**

- **The writer** (`scenario/write.rs`) sorts the battlefield by timestamp
  itself, as it already sorts the commanders, since the loader stamps in line
  order and CR 613.7's comparisons must read the same after a load.
- **The round trip** (`scenario_round_trip_test.rs`) compares the
  battlefield by timestamp order in its own `places`, not by the engine's
  sweep. What it checks is that the board came back, and a loaded board's ids
  are renumbered by design (`setup-architecture.md#42-names-not-ids`).
- **A reloaded board presents its lists in another order** than the board it
  was saved from, because its ids are minted in line order. Nothing in the
  engine reads that; a replay finds answers by identity. Making a reload
  present alike would mean the file carrying creation order apart from line
  order, a format change bought for presentation, so this note does not
  propose it.

B folds into C (the sort it drops is gone everywhere). If the owner keeps A
instead, `a_combatant_strikes_first`'s comment, which justifies its sort by a
`HashMap` order CL-1 removed, still wants fixing.

## Who sees the order, and does a rule fix it

**No rule fixes the order of a simultaneous event's members.** Every place the
order of permanents could change an outcome, the CR hands it to a player:
CR 603.3b (each player orders the triggers they control), CR 616.1 (the
affected object's controller or the affected player chooses the replacement),
CR 704.5j (the legend rule's keeper), CR 101.4c (a player's own simultaneous
choices). Everything else is simultaneous by rule: CR 704.3's single event,
CR 608.2e's step processed at once, CR 510.1c's damage divided as the
controller chooses. CR 613.7 orders *effects* within a layer, never the objects
a sweep visits. So the sweep order is the engine's choice of presentation; the
rules ask only that the game not depend on it.

**Who reads it:**

- the order of an option list a player answers: the legal actions, the
  attackers and blockers (`oracle/legality.rs`), the mana sources, the
  `OrderTriggers` candidates (`engine/triggers/dispatch.rs`'s
  `battlefield_readers`), the CR 616.1 candidates (`engine/replacement/gather.rs`),
  an Aura's new host (`engine/returns.rs`);
- the order of a batch's members (`engine/sba.rs`'s seven sweeps, a
  `FilteredPermanents` effect in `engine/resolve.rs`, combat's removal from
  combat in `engine/turns.rs`), and with it the order of the batch's log lines
  and of the triggers it queues, which is the order the `OrderTriggers` prompt
  and the window's auto-orderer start from;
- the trace and the "what happened" view, which read the log;
- the `scenario` writer (`scenario/write.rs`), which writes the battlefield in
  this order because the loader stamps in line order;
- a decision log, **not**: a replay finds each answer by what was chosen, not
  where it sat (`setup-architecture.md`, the decision log's decision D), so a
  reordered list replays the same game.

## Is id order the same on every route a game takes

| Route | Id order | Timestamp order |
|---|---|---|
| a fork | the parent's: a clone copies the table and the id counter (CL-1's rule 3) | the parent's |
| two processes, the same seed | the same | the same |
| saved through `scenario` and loaded | **renumbered**: the loader mints ids in the file's line order, and the writer writes lines in timestamp order, so the loaded game's id order is roughly the saved game's *timestamp* order (`setup-architecture.md#43-the-writer`, `#6-determinism`) | **the same**, by design: the writer exists to keep "a timestamp comparison a rule makes" |
| a board file vs a dealt game reaching the same board | different: a dealt game mints every card's id in `Game::new`, seat by seat in decklist order, before the shuffle | the board's own, which the file states |
| a permanent that left and came back | **unchanged**: the id survives a zone change and the epoch bumps (`engine/zones.rs`) | new, as CR 400.7 and 613.7d make it |

So id order is the order the cards were *constructed* in: seat 0's whole deck
before seat 1's, each in decklist order, then tokens and copies as made. It is
deterministic, the same in a fork and in every process, and that is all the
engine asks of an order no rule names. The save does not keep it, and under C
that is the scenario's to absorb (above), not the engine's.

**What C shows, and who it shows it to.** A board saved from the dev GUI and
loaded lists its legal actions, mana sources and attackers in another order,
and a state-based batch on it logs its deaths in another order. In a four-seat
game every list starts with seat 0's permanents, and a reanimated creature
keeps its old place. All of it is presentation to a client, which can sort
what it is handed as it likes (the window by timestamp, a seat by its own
permanents first).

## What would change the answer

- **A rule found that orders a sweep's members.** The rules pass above found
  none; one would be a timestamp sort at that sweep, named for its rule, not a
  sort on every sweep.
- **An engine reader, not a client, that needs timestamp order from a sweep.**
  The census found none: each reader feeds a player's choice, a simultaneous
  batch or a client, and CR 613.7's own comparisons read the timestamps
  themselves. The layer walk's member order (`engine/layers/board.rs`) was
  not traced by hand; the probe's passing tests include the layer suite and
  its memo audit.

## The probe: C's test cost, measured

A throwaway edit, reverted before this note was committed: `sort_by_timestamp`
a no-op and `in_timestamp_order` sorting on the id, so every sweep walks in id
order; then `cargo test --no-fail-fast` in debug (2026-11-14). **3,087 passed,
8 failed:**

- the four `ordered_sweeps` unit tests in `state/game_state.rs`, which pin the
  sweep's own order, and its audit's;
- `test_a_reattachment_moves_only_the_attachment_to_the_end`
  (`determinism_test.rs`) and
  `test_a_reattached_equipment_gets_a_timestamp_later_than_humility`
  (`phase_lh_integration_test.rs`): CR 613.7e's new timestamp moves the
  reattached permanent to the end of a sweep, and an id never moves;
- `a_token_entering_under_another_player_is_read_under_that_player`
  (`phase_board_scale_integration_test.rs`), a per-permanent list read in
  sweep order;
- `two_written_boards_load_as_played_in_debug` (`scenario_round_trip_test.rs`):
  **42 of 52 boards came back different.** The writer writes the battlefield in
  the sweep's order and the test compares the battlefield in the sweep's order
  before and after, so under id order a loaded board no longer sweeps as the
  board it was saved from. Whether a timestamp sort in the writer alone brings
  all 42 back was not tried.

Not read: the release-only tests (the four-seat round trip among them) and the
fuzz tables, where C would move every gameplay row through the bots' picks.
