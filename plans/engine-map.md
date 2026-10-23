# The engine map

**Where to start reading the engine.** It follows one event from the proposal
to the triggers it causes, names the subsystem that owns each step and every
seam where one subsystem reads another's output, then lays out the structure
those steps run on. Depth lives in the architecture docs and the trace pages
each row points to; the map only says where to go.

**Pinned to `22113bbb`** (2026-10-22, the item 6 audit's re-pin from
`75b1860`) and the frozen CR `tmnt`. Every `file:line` below was read at that
commit; the tree, not an architecture doc,
was the reference. **Refreshed, not frozen**: `plans/check_engine_map.py`
fails when a name here stops existing or the chokepoint's arms stop matching
`GameAction`, and `--drift` lists the lines that moved since the pin (§3).

**Why a markdown page and not a trace page.** A trace page is pinned and never
maintained (`engineering-practices.md` §7); this page is refreshed, so it sits
with the docs that are maintained. Markdown is also what the check can read
line by line, what a review diffs line by line, and what `grep` finds. It
costs the mermaid diagrams: §1.1 is a text sketch instead.

---

## 1. One event, proposal to triggers

### 1.1 The path in one sketch

```
PROPOSE   resolve_top_of_stack → propose_entry → execute_actions                  [chokepoint]
DECIDE    execute_batch_inner, phase 1 → apply_replacements (CR 616.1)            [replacement]
            is_prohibited first ─── a "can't" wins                                [restriction]
            gather ─── reads effective ability lists, the entry's frame           [layers]
            apply_rewrite → enter_as_copy ─── copiable_values                     [copy]
            ↺ re-gather against the rewritten entry (loop 1)
PERFORM   phase 2 → perform_action → perform_zone_change, perform_entry           [chokepoint]
            register_static_effects ─── files the gates' printed legs             [layers, triggers]
            emit_event ─── records in the batch's window                          [events]
SETTLE    the outermost close → settle_layer_consequences: control, then combat  [triggers, combat]
            membership (CR 506.4) ─── its records join the window, before DETECT    [events]
DETECT    the outermost close → dispatch_batch → detect → find_matches            [triggers]
            ─── reads effective ability lists (a copy's at layer 1)               [layers]
QUEUE     queue_matches → pending_triggers, AbilityTriggered (tier 2)             [triggers]
STATE     each dispatch's end → check_state_triggers (CR 603.8)               [triggers]
            ─── the board, not a record; also before PLACE, and at cleanup     [layers]
RIDERS    after the dispatch → execute_riders (CR 615.5): a batch of their own,  [replacement]
            dispatched at its close; a batch that joined an enclosing window
            runs its riders inside it (resolve_riders)
PLACE     perform_sba_and_triggers: SBAs ↺ (loop 3), then place_pending_triggers  [priority, triggers]
RESOLVE   resolve_top_of_stack → resolve_effect → execute_action … ↺ (loop 4)     [stack, resolve]
```

Every layer walk on the way is CR 613.8's order (loop 2), inside
`compute_characteristics` `engine/layers/compute.rs:103`.

### 1.2 The walk: Spark Double copies Soul Warden beside Master Biomancer

**The board.** Player 0 controls Master Biomancer ("each other creature you
control enters with +1/+1 counters equal to this creature's power and as a
Mutant") and Soul Warden ("whenever another creature enters, you gain 1
life"), and casts Spark Double from exactly `{3}{U}`. As it resolves, they
choose Soul Warden to copy. **The outcome**: Spark Double enters as a 4/4 Soul
Warden with three +1/+1 counters (one from its own CR 707.9e exception, two
from Biomancer) and the Mutant type; the original Soul Warden triggers once and
player 0 goes from 20 to 21 life. The copy has Soul Warden's trigger too, and
it is asked and refused: "another" excludes itself.

**How this was verified**: a throwaway probe on the pinned tree with the trace
sink installed (`test_support::install_trace`), every record read in order,
and run again at each re-pin. The
record each step produced is in brackets; `plans/trace_spine.py` renders the
same records for any board.

| # | Step | Owner | Where | Reads, and the seam |
|---|---|---|---|---|
| 1 | Spark Double resolves. CR 110.2b's default controller rides on `GameState::resolving` while the `StackEntry` is gone; a permanent spell then enters (CR 608.3a) | stack | `resolve_top_of_stack` `engine/stack.rs:31`; the entry at `engine/stack.rs:218` | — |
| 2 | Entering is one event. The resolution proposes the entry itself, joining what the effect says about how the permanent enters (an Aura's host, CR 303.4f) to the proposal ahead of any replacement; `change_zone` routes any other battlefield destination to the same proposal, never a `ZoneChange` onto the battlefield; `entry_proposal` seeds `EnterMods::NONE` and refuses CR 800.4b's departed controller | chokepoint | `propose_entry` `engine/actions.rs:2245`, `entry_proposal` `engine/actions.rs:2199`, `change_zone` `engine/actions.rs:1431` | S1 · [rc-4b](traces/rc-4b-entering-is-one-event.html) |
| 3 | The batch opens: the outermost call mints a `BatchId`, a nested one joins it [`batch`] | chokepoint, events | `execute_actions` `engine/actions.rs:821`, `open_batch` `events/event.rs:1009`, `execute_batch_inner` `engine/actions.rs:1052` | `replacement-architecture.md` §4.2 |
| 4 | Phase 1 groups members by subject, APNAP by chooser; an entry is its own subject | replacement | `subject_of` `engine/replacement/gather.rs:114`, `subject_groups` `engine/actions.rs:1292`, phase 1 at `engine/actions.rs:1105` | [rd-2](traces/rd-2-a-decision-is-per-subject.html) |
| 5 | CR 616.1, iteration 1. The frame is built lazily; a "can't" is asked first; then the gather: source 1a reads Spark Double's own `SourceOnly` copy effect off the CR 614.12 frame [`layer_walk` entering, 0/0], and the battlefield sweep reads Biomancer's effective list [`layer_walk` member] | replacement | `apply_replacements` `engine/replacement/pipeline.rs:380`; `EntryFrame::new` `engine/replacement/pipeline.rs:441`; `prohibition` `engine/replacement/pipeline.rs:456`; `gather` `engine/replacement/pipeline.rs:471` | S2, S3, S4 |
| 6 | Two candidates, one step: CR 616.1c's copy step outranks Biomancer's 616.1e, so `must_choose_among` leaves one and nobody is asked which [`pipeline` iteration 1, `single`] | replacement | `must_choose_among` `engine/replacement/gather.rs:1114`; `Decided::Single` `engine/replacement/pipeline.rs:523` | `replacement-architecture.md` §4.1 |
| 7 | The copy. The donor prompt is the event's only one, and the "you may" is its empty pick [`decision` `ChooseCopySource`, bounds (0, 1)]; the donor's copiable values are captured once; the three exceptions are made, each CR 707.9f condition checked on a frame of the copy without itself; CR 614.17d is asked of the added counter | replacement → copy | `Rewrite::EnterAsCopy` `engine/replacement/pipeline.rs:2136`, `entry_copy_donor` `engine/replacement/pipeline.rs:3017`, `ask_may_choose_copy_source` `ui/ask.rs:1247`, `copiable_values` `engine/layers/copy.rs:286`, `enter_as_copy` `engine/replacement/entry_copy.rs:30`, `strip_prohibited_counters` `engine/replacement/pipeline.rs:3068` | S5 · [cv-1](traces/cv-1-a-copy-is-a-snapshot.html), [cv-2b](traces/cv-2b-an-exception-is-checked-without-itself.html) |
| 8 | Iteration 2 re-gathers against the rewritten entry (CR 616.1f; **loop 1**): the frame is now a 2/2 Soul Warden, Spark Double's own effect is in CR 614.5's applied set, and Biomancer's `EnterWith` is the one candidate. Its amount reads Biomancer off the real board (2), and the Mutant is an edit on the entry [`pipeline` iteration 2] | replacement | `Rewrite::EnterWith` `engine/replacement/pipeline.rs:2100`, `evaluate_enter_template` `engine/replacement/pipeline.rs:3121` | S6, S7 · [rg](traces/rg-an-entry-write-changes-what-applies-next.html), [rc-5](traces/rc-5-applying-an-entry-can-move-the-board.html) |
| 9 | Iteration 3 gathers nothing new [`pipeline` iteration 3, frame 4/4], and the loop returns the decided entry: the copy, three counters, the Mutant | replacement | the empty gather's return, `engine/replacement/pipeline.rs:495` | — |
| 10 | Before performing, CR 603.10's frames are taken while the board is untouched; nothing departs here | chokepoint → triggers | `departs_an_ability_list_source` `engine/triggers/dispatch.rs:998`, `capture_lki` `engine/triggers/dispatch.rs:1026`, called at `engine/actions.rs:1167` | S8 |
| 11 | Phase 2. The move, then its announcement, then the entity. The move first ends every reference to the object as it was (CR 400.7: rows that named it pruned, save a permanent spell's continuous rows under 400.7a; a prevention shield watching the spell is re-pointed at the permanent under 400.7c); then `PermanentState::entering`, the constructor the frame used; registration files the copy's abilities, Soul Warden's trigger among them; the entry is announced, then the counters it entered with (CR 122.6) [`event` ZoneChange, `event` ETB, `event` CountersPutOn, `batch_end`] | chokepoint | `perform_action` `engine/actions.rs:1461`, `perform_zone_change` `engine/actions.rs:2028`, `move_object` `engine/zones.rs:50`, `break_references_to` `engine/zones.rs:499`, `announce_zone_change` `engine/actions.rs:2133`, `perform_entry` `state/game_state.rs:1563`, `PermanentState::entering` `state/battlefield.rs:265`, `register_static_effects` `state/game_state.rs:1984`, `emit_event` `state/trace.rs:846` | S9, S10, S11 |
| 12 | The outermost close dispatches the window: histories first, then the gate reads `trigger_sources` and finds both Soul Wardens | triggers | `execute_batch`'s close `engine/actions.rs:898`, `dispatch_batch` `engine/triggers/dispatch.rs:574`, `advance_history` `engine/triggers/history.rs:75`, `detect` `engine/triggers/dispatch.rs:735`, `battlefield_readers` `engine/triggers/dispatch.rs:923` | S12 · [tr-1](traces/tr-1-a-trigger-is-matched-at-the-close.html) |
| 13 | The match reads each candidate's effective list, Spark Double's being the copy's since layer 1 applies its `entered_as`. The original matches the ETB record; the copy is refused by "another"; each is asked of all three records [`trigger` ×6] | triggers → layers | `find_matches` `engine/triggers/dispatch.rs:1341`, `match_def` `engine/triggers/dispatch.rs:1763` | S13 |
| 14 | Queue, not resolve: a `PendingTrigger` on `GameState`, and an unbatched `AbilityTriggered`, caused by CR 603.2's rule, dispatched as it is emitted (CR 603.3b's second tier) [`event` AbilityTriggered] | triggers | `queue_matches` `engine/triggers/dispatch.rs:773`, `emit_event_unbatched` `state/trace.rs:865` | `triggers-architecture.md` §4.8 |
| 15 | No rider was queued, so once the window is dispatched (steps 12–14) `execute_riders` returns at once and opens no batch of its own | replacement | `execute_riders` `engine/actions.rs:1272`, called after `dispatch_batch` at `engine/actions.rs:899` | `replacement-architecture.md` §4.1a |
| 16 | The next CR 117.5 moment: SBAs until none (**loop 3**), then placement in APNAP order with the tiers, a stack object with `is_spell` false and its targets chosen per instance [`pending`] | priority, triggers | `perform_sba_and_triggers` `engine/priority.rs:266`, `check_state_based_actions_loop` `engine/sba.rs:554`, `place_pending_triggers` `engine/triggers/placement.rs:35`, `place_one` `engine/triggers/placement.rs:185` | `triggers-architecture.md` §5 · [a4i](traces/a4i-a-target-belongs-to-an-instance.html) |
| 17 | It resolves and proposes `GainLife` through the same chokepoint: a new batch, an empty gather, `LifeChanged`, another dispatch that matches nothing (**loop 4**) | stack, resolve | `resolve_effect_with_announced_targets` `engine/resolve.rs:221`, called at `engine/stack.rs:167` | S14 |

### 1.3 The seams

Each is one subsystem reading another's output, in path order.

| | Seam | Where | Doc | Trace |
|---|---|---|---|---|
| S1 | The chokepoint routes entering to one proposal, so every reader of an entry sees one event | `propose_entry` `engine/actions.rs:2245`, which `change_zone` `engine/actions.rs:1431` routes any battlefield destination to | `replacement-architecture.md` §9 (RC-4b) | [rc-4b](traces/rc-4b-entering-is-one-event.html) |
| S2 | **Replacement reads layers**: the entering object's CR 614.12 frame. `EntryFrame` decides when; `compute_as_entering` computes it through the walk's accessor pair (§2.4, §2.5) | `frame_of` `engine/replacement/lookahead.rs:93`, `compute_as_entering` `engine/layers/lookahead.rs:156` | `replacement-architecture.md` §5, §5d | [rc-4b](traces/rc-4b-entering-is-one-event.html), [li-1](traces/li-1-one-pass-per-board.html) |
| S3 | **Replacement reads layers and registration**: the gather reads effective ability lists, behind a gate whose legs registration and the registry summary keep (§2.3) | `gather` `engine/replacement/gather.rs:202` | `replacement-architecture.md` §3.3 | [rf](traces/rf-a-source-off-the-battlefield.html) |
| S4 | **Replacement reads restriction**: a "can't" is asked ahead of the gather and wins (CR 101.2, 614.17) | `prohibition` `engine/restriction/predicate.rs:96` | `cant-effects-architecture.md` §4.1, §5.3 | — |
| S5 | **Replacement runs copy, copy reads layers**: the rewrite captures the donor's copiable values at the end of layer 1 and carries them on the proposal | `enter_as_copy` `engine/replacement/entry_copy.rs:30`, `copiable_values` `engine/layers/copy.rs:286` | `copy-effects-architecture.md` §4.1, §4.1a | [cv-1](traces/cv-1-a-copy-is-a-snapshot.html), [cv-2b](traces/cv-2b-an-exception-is-checked-without-itself.html) |
| S6 | **Replacement reads its own output**: an entry's writes decide what the next iteration gathers, and the feeds table is what may skip the prompt | `EntryWrites` `engine/replacement/pipeline.rs:1538`, `ordering_cannot_change_outcome` `engine/replacement/pipeline.rs:1083` | `replacement-architecture.md` §3.5 | [rg](traces/rg-an-entry-write-changes-what-applies-next.html) |
| S7 | **An amount reads the real board for an existing source and the frame for the entering one** | `evaluate_enter_template` `engine/replacement/pipeline.rs:3121` | `replacement-architecture.md` §5b | [rc-5](traces/rc-5-applying-an-entry-can-move-the-board.html) |
| S8 | **Triggers read the batch's decisions**: CR 603.10's look-back frames and snapshots are taken between deciding and performing | `capture_lki` `engine/triggers/dispatch.rs:1026` | `triggers-architecture.md` §4.3, §6.1 | — |
| S9 | **Layers read the performer's entity**: the copy and the edits an entry fixed apply from `PermanentState::entered_as`, the copy at layer 1 | `applications_in_layer` `engine/layers/board.rs:1018`, the notes at `engine/layers/board.rs:1055` | `replacement-architecture.md` §3.5; `copy-effects-architecture.md` §4.1 | [rg](traces/rg-an-entry-write-changes-what-applies-next.html) |
| S10 | **Every gate reads registration**: the arrived-with abilities, an entry copy's included, go into the printed legs, `trigger_sources` among them | `register_static_effects` `state/game_state.rs:1984`, the trigger leg at `state/game_state.rs:2028` | `copy-effects-architecture.md` §4.7; `triggers-architecture.md` §4.2 | — |
| S11 | **Triggers read events**: `emit_event` is the one door, and every record carries its batch | `emit_event` `state/trace.rs:846` | `triggers-architecture.md` §4.1 | [tr-1](traces/tr-1-a-trigger-is-matched-at-the-close.html) |
| S12 | **Triggers read the chokepoint's close**: the window is dispatched when the outermost batch returns, and a batch that queued riders dispatches what it performed before them, the riders then being a batch of their own (`execute_riders`) | `execute_batch` `engine/actions.rs:843` (the window dispatched at `engine/actions.rs:898`), then `execute_riders` `engine/actions.rs:1272` | `triggers-architecture.md` §4.1 | [tr-1](traces/tr-1-a-trigger-is-matched-at-the-close.html) |
| S13 | **Triggers read layers**: the matcher asks each candidate's effective ability list | `find_matches` `engine/triggers/dispatch.rs:1341` | `triggers-architecture.md` §4.2 | [tr-1](traces/tr-1-a-trigger-is-matched-at-the-close.html) |
| S14 | **The chokepoint reads triggers' output**: a resolution proposes through `execute_action` like any other | `resolve_effect` `engine/resolve.rs:199` | `triggers-architecture.md` §6 | — |

### 1.4 The loops the path passes

Named only; `plans/references/feedback-loops.md` counts the cards that close
each, and reads these seams for a fifth loop, finding none.

1. **An entry's writes and CR 616.1's re-gather** — step 8, S6.
2. **A layer effect and CR 613.8's dependency** — inside every walk: `depends_on` `engine/layers/board.rs:1583`, `next_ready` `engine/layers/board.rs:1636`; [item-7](traces/item-7-an-effect-waits-for-what-it-reads.html).
3. **A state-based action's result and the next check (CR 704.3)** — step 16.
4. **An event a triggered ability produces and what else triggers (CR 603)** — steps 14 and 17.

---

## 2. The structure

### 2.1 The modules and what each owns

Every file under `mtgsim/src` is named here, and `--drift` lists one that is
not (§3); `mod.rs` files declare and re-export only
(`engineering-practices.md` §6).

| Module | Owns |
|---|---|
| `engine/actions.rs` | the chokepoint: `GameAction`, `ActionContext`, a batch's two phases (decide, perform) and its riders after them (`execute_riders`, `resolve_riders`), `perform_action`'s arms, `change_zone`, entry and token proposals |
| `engine/layer_settlement.rs` | what the layers' output changed, settled at an outermost batch's head and at its close before its window is dispatched, at each state-based check and in cleanup: CR 603.10d's change of control, then CR 506.4's combat membership |
| `engine/replacement/` | CR 614–616: `pipeline.rs` the CR 616.1 loop, `apply_rewrite` and the riders; `gather.rs` the five sources and their gate; `lookahead.rs` `EntryFrame`; `entry_copy.rs` CR 707.9's exceptions on an entry copy; `instance.rs` CR 614.5's identity |
| `engine/restriction/` | CR 101.2 and 614.17: `predicate.rs` `is_prohibited`, the one reader of every "can't" |
| `engine/cost_determination/` | CR 601.2f: `gather.rs` which cost effects apply; `total.rs` the arithmetic and its order |
| `engine/layers/` | CR 613: `compute.rs` `compute_characteristics` and the memo's door; `board.rs` the board-wide pass, the accessor pair and CR 613.8's order; `lookahead.rs` the CR 614.12 overlay; `copy.rs` copiable values; `cda.rs` CDAs at layers 4, 5 and 7a; `condition.rs` "as long as"; `intrinsic.rs` CR 306.5b's loyalty ability; `land_types.rs` CR 305.6–305.7; `explain.rs` the pass run again with a recorder, for a client's "how did it get this"; `types.rs` the vocabulary |
| `engine/triggers/` | CR 603: `dispatch.rs` detection at a window's close and the CR 603.10 frames; `delayed.rs` CR 603.7's registry, a leg of every dispatch, beside CR 610.3's "until" returns, which a dispatch performs as one batch (`engine/returns.rs`); `state.rs` CR 603.8's state check; `control.rs` CR 603.10d's change of control, announced by a sweep; `placement.rs` CR 603.3; `binding.rs` the bound facts at resolution; `binding_reads.rs` what a def reads of its binding, for the ordering elision; `history.rs` the histories' one writer; `audit.rs` the dispatch audit |
| `engine/stack.rs`, `engine/resolve.rs` | CR 608: resolution and fizzle; `ResolutionContext` and each `Primitive` turned into proposals and registry rows |
| `engine/put_on_stack.rs` | CR 601.2 and 602.2: `cast_spell`, `activate_ability`, the announcement, the mana window |
| `engine/priority.rs`, `engine/sba.rs`, `engine/turns.rs` | CR 117 priority rounds and CR 117.5's `perform_sba_and_triggers`; CR 704; CR 500's turn, phase and step proposals and the turn-based actions |
| `engine/zones.rs`, `engine/leaving.rs` | `move_object`, the one mover, and zone-state cleanup; CR 800.4, a player leaving |
| `engine/combat/` | CR 506–511: `steps.rs` the combat steps; `validation.rs` attack and block legality; `resolution.rs` damage assignment; `keywords.rs` first strike, double strike, trample |
| `engine/targeting.rs` | CR 115 and 601.2c: target instances, validation, the selection-side `object_matches_filter` |
| `engine/costs.rs`, `engine/mana.rs` | CR 601.2h/602.2b payment; CR 605 mana abilities |
| `engine/zone_function.rs`, `engine/keywords.rs`, `engine/trace_records.rs` | CR 113.6's `functions_in`; lifelink and deathtouch on damage; every trace record's shape |
| `state/game_state.rs` | `GameState` and `StackEntry`; timestamps and `battlefield_ids_ordered`; `perform_entry`, `register_static_effects` and the printed-source sets |
| `state/battlefield.rs`, `state/player.rs` | `PermanentState` and its one constructor for an entry; `PlayerState` |
| `state/continuous_effects.rs`, `state/replacement_effects.rs`, `state/restrictions.rs`, `state/duration_registry.rs` | the three registries of resolution-created rows, their shared CR 514.2 expiry, and `RegistryScopeSummary`, the gates' granted and copied legs |
| `state/layer_memo.rs`, `state/history.rs`, `state/history_log.rs`, `state/diagnostics.rs`, `state/trace.rs` | the layer memo; `GameHistory`, what the game remembers: the history log (one row per event a card can ask about, kept for the whole game in `Arc`-chained chunks), each player's `PlayerHistory` beside it, and each ability's "this turn" facts; seed-deterministic work counters; the trace sink and `emit_event` |
| `state/shared_stores.rs` | `ObjectTable` and `SharedList`, the chunked stores a fork shares: objects, the battlefield, the stack and its entries, waiting triggers, the memo's frames |
| `state/decision_log/` | the decision log, every answer a game's choices get, written by the engine: `hook.rs` the client's writer, `start.rs` where a game began, `text.rs` the format |
| `scenario/` | a board at rest and its text (`setup-architecture.md`): `text.rs` the grammar both ways, `board.rs` the parsed value, `build.rs` the loader through the construction doors, `setup.rs` the `then:` lines, `write.rs` a game written back, `error.rs` the refusals |
| `state/game.rs`, `state/game_config.rs` | `Game`: setup and the turn loop a harness drives; format configuration |
| `events/event.rs`, `events/recorder.rs` | `GameEvent`, the batch window and `BatchId`; the whole stream for readers outside the game |
| `objects/card_data.rs`, `objects/object.rs` | `CardData`, `AbilityDef` and the builder; `GameObject` |
| `oracle/characteristics.rs` | the layer-routed accessors every reader of a characteristic uses |
| `oracle/legality.rs`, `oracle/mana_helpers.rs`, `oracle/board.rs` | candidate actions and legal selections; castability and `activatable_abilities`; board queries |
| `oracle/mana_supply.rs`, `oracle/x_values.rs` | the mana check: a player's `ManaSupply` and whether it pays a cost (CR 601.2g–h's existence half); the values of X a caster may announce (CR 107.3a, 601.2b) |
| `types/` | the type surface: `effects.rs`, `replacement.rs`, `restriction.rs`, `triggers.rs`, `cost_modification.rs`, `costs.rs`, `zones.rs`, `ids.rs`, `card_types.rs`, `colors.rs`, `mana.rs`, `keywords.rs`, `keyword_actions.rs`, `history.rs`; `mod.rs` holds `counted_enum!` |
| `ui/decision.rs`, `ui/ask.rs`, `ui/choice_types.rs` | the five-method `DecisionProvider` (`pick_n`, `pick_number`, `allocate`, `choose_ordering`, and the defaulted `seat_mode`); the typed `ask_*` bridge, where every prompt is validated and traced; `ChoiceKind` |
| `ui/random.rs`, `ui/cli.rs`, `ui/replay.rs`, `ui/mana_window_stop.rs`, `ui/auto_payer.rs`, `ui/auto_yield.rs`, `ui/auto_order_triggers.rs`, `ui/full_control.rs`, `ui/display.rs` | three providers (random, the terminal, a decision log's replay), four decorators, the full-control switch above a person's stack, the text formatters |
| `ui/why.rs`, `ui/what_happened.rs`, `ui/waiting.rs` | the engine's answers a client draws: why an object is the way it is or an option is not offered; an event's why, read from a replay's trace; what the game holds for a later event or turn |
| `cards/registry.rs`, `cards/random_deck.rs`, `cards/authoring/` | the registry and the two pools; the fuzz harness's decks; the card-authoring words, `triggers.rs` today |
| `cards/*.rs` | card definitions, filed by the phase that first needed them |
| `bin/fuzz_games.rs`, `bin/cli_play.rs`, `test_support.rs`, `lib.rs`, `main.rs` | the random-versus-random harness; terminal play; the `test-support` feature's helpers; the crate root; a stub |

### 2.2 The chokepoint's arms

`perform_action` `engine/actions.rs:1461` has one arm per `GameAction` variant
(`engine/actions.rs:138`) and no wildcard. A nested proposal is **contained**
(its own CR 614.5 set) unless marked decomposed. The check fails when this
table, the enum and the match disagree.

<!-- arms:begin -->
| Arm | Performs | Proposes inside it |
|---|---|---|
| `DealDamage` | CR 120.3's results off the target's effective types: marked damage, deathtouch, commander damage; `DamageDealt` | `LoseLife` (120.3a), `RemoveCounters` of loyalty (120.3c) |
| `DrawCards` | CR 121.2: `n` individual draws, each its own batch | `DrawCard`, decomposed (`execute_actions_decomposing`; [re-2](traces/re-2-a-draw-carries-its-lineage.html)) |
| `DrawCard` | `draw_card`, with CR 121.6a's empty-library flag | the library-to-hand `change_zone` |
| `GainLife` | the life total; `LifeChanged` | — |
| `LoseLife` | the life total, 0 a local no-op; `LifeChanged` | — |
| `ZoneChange` | `perform_zone_change`; loud on a battlefield destination | — |
| `Untap` | on the transition only (CR 603.2e); `Untapped` | — |
| `Attach` | `GameState::attach`, nothing if already there; an `Unattached` from the old host, then `Attached` | — |
| `Unattach` | `GameState::detach`, nothing if attached to nothing (CR 704.5n, 704.5p); `Unattached` | — |
| `Tap` | on the transition only; `Tapped` | — |
| `AddCounters` | counters on a permanent or a player; `CountersPutOn`, with its putter and the count left | — |
| `RemoveCounters` | as many as there are (CR 609.3); `CountersRemovedFrom` only if any, with its remover where a player is one | — |
| `Destroy` | the outer event only; its `ZoneChange` to the graveyard, decided with it in phase 1 (`decide_destruction_moves`, item 248), is performed right after it | — |
| `EnterBattlefield` | the move (or a token's creation), then `perform_entry`, which announces a `CountersPutOn` for each counter row (CR 122.6); an Aura entering attached is then attached (CR 303.4f), with `Attached` | — |
| `CreateTokens` | the objects, then every entry as one batch | `EnterBattlefield` per token |
| `CreateTokenIn` | `put_token_into`; `TokenCreated` | — |
| `TokenCeasesToExist` | CR 704.5d: out of its zone and the store; `TokenCeasedToExist` | — |
| `BeginTurn` | `begin_turn`; `TurnBegin` | — |
| `BeginPhase` | the phase; `PhaseBegin` | — |
| `BeginStep` | the step; `StepBegin` | — |
| `Scry` | CR 701.22 whole in the arm, the prompt included (`ask_scry`); `Scried` | — |
| `ShuffleLibrary` | `shuffle_library`, drawing on `GameState::rng`; `LibraryShuffled` | — |
| `PlayerLoses` | the loss and `PlayerLost`, then CR 800.4a | `player_left_the_game`'s moves, joining the batch |
| `PlayerWins` | the result; `PlayerWon` | — |
| `GameEndsInADraw` | the result, a draw (CR 104.4); `GameEndedInADraw` | — |
| `ProduceMana` | the pool, its one writer; `ManaAdded` | — |
<!-- arms:end -->

Four doors sit beside `execute_action` `engine/actions.rs:772`:
`execute_actions` for a simultaneous event, `execute_caused_actions`
`engine/actions.rs:834` for one whose members carry causes of their own (CR
704.3's state-based actions, CR 610.3d's returns), `execute_actions_decomposing`
`engine/actions.rs:947` for CR 121.2's draws, and `execute_actions_new_batch`
`engine/actions.rs:999` for `// AUXILIARY-MOVE:` (CR 614.13). The
`// CAST-ROLLBACK:` exemption is `rollback_cast_to_hand`
`engine/put_on_stack.rs:732`, and `announce_zone_change` has one caller outside
the arms: `cast_spell`, at CR 601.2i.

### 2.3 The gates, and the three legs a new source extends

A static ability is discovered off the **effective** ability list, and each
reader skips its sweep behind a gate. Each gate has three legs, one per route
onto that list: **printed** (a set registration fills, an entry copy's list
included), **granted** (a Layer 6 row) and **copied** (a layer 1 row). A new
route onto the list needs a leg on every gate, or the ability is dead on every
board the gate skips. Layer 3, text change, is the route with no leg yet.

| Gate | Where | Printed | Granted, copied |
|---|---|---|---|
| replacement | `gather` `engine/replacement/gather.rs:202`; the gate at `engine/replacement/gather.rs:272` | `replacement_ability_sources`, `zone_replacement_ability_sources` | one pair for both since RF: `unattributed_replacement_zones`, `any_named_unattributed_replacement` |
| restriction | `prohibition` `engine/restriction/predicate.rs:96`; the gate at `engine/restriction/predicate.rs:114` | `restriction_ability_sources` | `any_granted_restriction`, `any_copied_restriction` |
| cost | `cost_modifications_for` `engine/cost_determination/gather.rs:56`; the gate at `engine/cost_determination/gather.rs:62` | `cost_modification_ability_sources` | `any_granted_cost_modification`, `any_copied_cost_modification` |
| trigger | `detect` `engine/triggers/dispatch.rs:735`; the gate at `engine/triggers/dispatch.rs:741` | `trigger_sources`, `zone_trigger_sources`, and `newcomer_trigger_sources`: off the battlefield, a printed trigger whose every event arm is the object itself moving (`TriggerDef::triggers_only_on_this_object_moving`), asked only of the window that brought the object there (`newcomer_trigger_carriers`) | one pair for both: `unattributed_trigger_zones` for a `Filter` row's zones, `named_unattributed_trigger_kinds` for a named row's objects; a departure's CR 603.10a frame and a look-back snapshot are two more probes |
| state trigger | `check_state_triggers` `engine/triggers/state.rs`, the trigger gate's legs under `EventKind::BoardState` | the same two, by kind | the same, with `unattributed_trigger_kinds` saying whether a `Filter` row's zones hold one |
| trigger multiplier (CR 603.2d; `glossary.md`'s **multiplier**) | `count_additional_triggers` `engine/triggers/dispatch.rs:1530`, over `trigger_multiplier_candidates` `engine/triggers/dispatch.rs:1481`; the gate at `trigger_multiplier_sources_now` `engine/triggers/dispatch.rs:1510` | `trigger_multiplier_sources` | `any_granted_trigger_multiplier`, `any_copied_trigger_multiplier`; a departure's CR 603.10a frame and a look-back snapshot are two more probes |

The printed legs are written by `register_static_effects`
`state/game_state.rs:1984` as an object arrives; the other two are
`RegistryScopeSummary` `state/continuous_effects.rs:58`, recomputed from the
registry on every change. → `copy-effects-architecture.md` §4.7,
`triggers-architecture.md` §4.2.

### 2.4 The one `ObjectFilter` table, and the accessor pair

- **One table**: `evaluate` `engine/object_filter.rs:215` answers each leaf for
  one object, over a frame its asker supplies and the asker's
  `FilterReferents` `engine/object_filter.rs:34`: what "you", "this", "its
  owner" and the targets announced so far mean where the filter is asked.
  Whether a `PlayerRef` names a player is `player_ref_names`
  `engine/object_filter.rs:361`, for the table's leaves, a trigger's "whose"
  and a `SourceFilter`. → `layers-architecture.md` §3.4.
- **Inside the walk**: `object_matches_filter` `engine/layers/compute.rs:834`
  asks it of a member, off a partly applied frame, with "you" resolved on
  first use through the effect's source (`FilterPlayers`
  `engine/layers/compute.rs:573`).
- **Outside it**: `GameState::object_matches_filter`
  `engine/targeting.rs:669` asks it for a legal selection, a replacement's or a
  restriction's `ObjectSet` and pattern, a trigger's subject or a cost, with
  "you" passed in. Its variants `object_matches_filter_for_instance` (CR
  601.2c's "another target") and `object_matches_filter_of_source` (an
  effect's `NotSource` and "its owner"; `set_affects`' door) supply the words.
- **The accessor pair**: every concrete-state read the walk makes goes through
  `Board::entity` `engine/layers/board.rs:346` (the entity: controller, CR
  302.6's clock, counters) or `rows_in_layer` `engine/layers/board.rs:999`
  (the registry's slice). That is where the look-ahead answers for one object
  without a `GameState` clone; `Board::timestamp_of`
  `engine/layers/board.rs:363` is the third read it answers.

### 2.5 The look-ahead frame: two files, one frame

- `engine/layers/lookahead.rs` is **how**: `Lookahead`
  `engine/layers/lookahead.rs:32` holds the entity `PermanentState::entering`
  would build, the rows `register_static_effects` would write and their
  summary; `compute_as_entering` `engine/layers/lookahead.rs:156` runs one pass
  with it threaded through `Board`, so the accessor pair answers for the
  entering object and the real board answers for everything else.
- `engine/replacement/lookahead.rs` is **when, and for whom**: `EntryFrame`
  `engine/replacement/lookahead.rs:40` is the pipeline's frame of the
  proposal's subject, computed at most once per CR 616.1 iteration and only if
  something asks (`frame_of` `engine/replacement/lookahead.rs:93`). The gather
  and `is_prohibited` share it; `EntryFrame::for_entering`
  `engine/replacement/lookahead.rs:64` builds one for a synthetic question, the
  counters an exception or a template would add.
- **Why two**: the overlay has to live beside the accessors it answers
  through, which are private to the walk (`pub(super)` and a private function),
  and the lifetime has to live with the loop whose iterations rewrite the
  proposal it is built from (CR 614.12's first clause). Either merge exports
  the other module's internals. → `replacement-architecture.md` §5, §5d.

### 2.6 The decision sites `codebase-state.md` main item 40 tracks

The invariant: no decision site holds outcome-bearing state off `GameState`.
Every prompt goes through `ui/ask.rs`, which validates the answer and traces a
`decision` record.

| Site | Where | Held off `GameState` | Status |
|---|---|---|---|
| the priority loop | `run_priority_round` `engine/priority.rs:38`; locals at `engine/priority.rs:74` | the last rejection, which the re-ask names and nothing filters by since item 193, and `rejections` | not outcome-bearing; a resume mid-round is `codebase-state.md` item 140 |
| the mana window | `run_mana_ability_window` `engine/put_on_stack.rs:647`; the set at `engine/put_on_stack.rs:668` | `failed` | not outcome-bearing |
| CR 601.2b–d's announcement | `cast_spell` `engine/put_on_stack.rs:93`, rewound by `rollback_cast_to_hand` `engine/put_on_stack.rs:732` | the unpushed `StackEntry` | violator 2 |
| the CR 616.1 loop | `apply_replacements` `engine/replacement/pipeline.rs:380`; the sets at `engine/replacement/pipeline.rs:401` | `applied`, `declined`, `exempt_applied`, the rewritten members | violator 1 |
| the fix's shape | `entry_selection` `state/game_state.rs:624`, `prevention_allocations` `state/game_state.rs:634`, `pending_triggers` `state/game_state.rs:705` | — | on `GameState`; placement drains the queue, so a clone resumes |

### 2.7 Every `CLAUDE.md` invariant, where it lives

| Invariant | Lives at | Doc |
|---|---|---|
| Characteristics of an object in any zone come through the layers | `oracle/characteristics.rs` → `compute_characteristics` `engine/layers/compute.rs:103`; exemptions tagged `// PRE-LAYER ZONE:`, `// AS PRINTED:` (`printed_faces` `ui/display.rs:172`) and `// NO ROW REACHES:` (`prints_cost_ability` `engine/cost_determination/gather.rs:208`, `objects_that_may_trigger` `engine/triggers/audit.rs:154`), and `register_static_effects` `state/game_state.rs:1984` | `layers-architecture.md` §11.1; the tags, `engineering-practices.md` §11 |
| Ability indices are into the effective list | `activatable_abilities` `oracle/mana_helpers.rs:360`, the re-derivation by id at `engine/priority.rs:143`, `activate_ability` `engine/put_on_stack.rs:484` | `cant-effects-architecture.md` §4.4 |
| Registry membership is not effect existence | `static_ability_still_exists` `engine/layers/board.rs:1192`, asked per layer; nothing inside the pass calls the top-level entry, and only a non-member's walk recurses, at a strictly lower ceiling: `Board::frame_of` `engine/layers/board.rs:412` | `layers-architecture.md` §5.2 |
| CDAs are never registry effects | `cda_modifications` `engine/layers/cda.rs:71`; skipped at registration, `state/game_state.rs:2090`; a Layer 6 grant clears the flag at `engine/layers/compute.rs:1211`, in `apply_resolved` | `layers-architecture.md` §6 |
| No observable mutation outside `perform_action`'s arms | `perform_action` `engine/actions.rs:1461` | `replacement-architecture.md` §2; the gate and its two tags, `engineering-practices.md` §11 |
| A `ZoneChange` names its act and a proposal its cause, with no catchall | `ZoneChangeAct` `types/zones.rs:226`, `EventCause` `events/cause.rs:21` | `replacement-architecture.md` §2b.2, §2b.3 |
| Performers are loud; callers check legality | the `Untap` arm `engine/actions.rs:1645`; `Primitive::Untap` `engine/resolve.rs:1030` | `replacement-architecture.md` §9 (RA-2) |
| "Becomes" events fire on the transition only | the `Tap` arm `engine/actions.rs:1706` | CR 603.2e |
| An observable order goes through the timestamp order | `battlefield_ordered` `state/game_state.rs:1375`, `battlefield_ids_ordered` `state/game_state.rs:1385`, keyed on `PermanentState::timestamp` `state/battlefield.rs:63` | `plans/id-hasher.md` §6; the gate, `engineering-practices.md` §11 |
| One performer, one emitter | `move_object` `engine/zones.rs:50`, `announce_zone_change` `engine/actions.rs:2133` | `replacement-architecture.md` §11, item 20 |
| A simultaneous rule needs a batch | `execute_actions` `engine/actions.rs:821`; `execute_actions_new_batch` `engine/actions.rs:999` | `replacement-architecture.md` §4.2 |
| No outcome-bearing decision state off `GameState` | §2.6 | `codebase-state.md` item 40 |
| Never prompt with fewer than two candidates | `Decided::Single` `engine/replacement/pipeline.rs:523` | `replacement-architecture.md` §4.1 |
| A "can't" is not a replacement effect | `prohibition` `engine/replacement/pipeline.rs:456`, ahead of the gather | `cant-effects-architecture.md` §4.1 |
| A static replacement ability is found off the effective list | `gather` `engine/replacement/gather.rs:202`; the legs, §2.3 | `replacement-architecture.md` §3.3 |
| Declining is tracked apart from CR 614.5's set | `declined` `engine/replacement/pipeline.rs:405` | `replacement-architecture.md` §4.1 |
| Deciding is separated from performing | phase 1 `engine/actions.rs:1105`, phase 2 `engine/actions.rs:1180` | `replacement-architecture.md` §4.2 |
| Riders run after the performed event, carrying its applied set | `execute_riders` `engine/actions.rs:1272` (a batch that dispatches its own window: its riders as a batch of their own, after that dispatch) and `resolve_riders` `engine/actions.rs:1379` (a nested or decomposed batch's, inside the window it joined); `Rider::lineage` `engine/replacement/pipeline.rs:81` | `replacement-architecture.md` §4.1a |
| `EventPattern` grows one arm per variant; `Rewrite` is closed; variety goes in `then` | `EventPattern` `types/replacement.rs:204`, `Rewrite` `types/replacement.rs:855`, `ReplacementDef::then` `types/replacement.rs:137` | `replacement-architecture.md` §3.2a, §3.2b |
| Randomness is owned | `GameState::rng` `state/game_state.rs:812`; the provider's own `rng` `ui/random.rs:207`; `IdHash` `types/ids.rs:310`; `tests/determinism_test.rs`; refused elsewhere by `clippy.toml` | `plans/id-hasher.md` §6 |

---

## 3. Keeping it true

`python plans/check_engine_map.py` is in `CLAUDE.md`'s check line and CI. It
fails when:

- the arm table in §2.2, `GameAction`'s variants and `perform_action`'s arms are
  not the same set, or the match grows a wildcard;
- a file, directory or trace page named here does not exist;
- a name cited with a location does not occur in that file's code (comments
  stripped), or the line is past the file's end;
- any other code name in backticks occurs nowhere in the crate's code;
- a section number after a doc's name is not one of that doc's headings.

It reports, and does not fail on, lines that drifted since the pin and files
this page does not name: `--drift` lists both. **A refresh** is: run
`--drift`, move each listed line, re-read any step whose function changed,
re-run the walk's board if the path changed, and re-pin the header.
