# The engine map

**Where to start reading the engine.** It follows one event from the proposal
to the triggers it causes, names the subsystem that owns each step and every
seam where one subsystem reads another's output, then lays out the structure
those steps run on. Depth lives in the architecture docs and the trace pages
each row points to; the map only says where to go.

**Pinned to `75b1860`** (main, 2026-09-29) and the frozen CR `tmnt`. Every
`file:line` below was read at that commit; the tree, not an architecture doc,
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
PROPOSE   resolve_top_of_stack → change_zone → propose_entry → execute_actions   [chokepoint]
DECIDE    execute_batch_inner, phase 1 → apply_replacements (CR 616.1)            [replacement]
            is_prohibited first ─── a "can't" wins                                [restriction]
            gather ─── reads effective ability lists, the entry's frame           [layers]
            apply_rewrite → enter_as_copy ─── copiable_values                     [copy]
            ↺ re-gather against the rewritten entry (loop 1)
PERFORM   phase 2 → perform_action → perform_zone_change, place_on_battlefield    [chokepoint]
            register_static_effects ─── files the gates' printed legs             [layers, triggers]
            emit_event ─── records in the batch's window                          [events]
RIDERS    phase 3 → resolve_rider (CR 615.5)                                      [replacement]
DETECT    the outermost close → dispatch_batch → detect → find_matches            [triggers]
            ─── reads effective ability lists (a copy's at layer 1)               [layers]
QUEUE     queue_matches → pending_triggers, AbilityTriggered (tier 2)             [triggers]
PLACE     perform_sba_and_triggers: SBAs ↺ (loop 3), then place_pending_triggers  [priority, triggers]
RESOLVE   resolve_top_of_stack → resolve_effect → execute_action … ↺ (loop 4)     [stack, resolve]
```

Every layer walk on the way is CR 613.8's order (loop 2), inside
`compute_characteristics` `engine/layers/compute.rs:94`.

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
sink installed (`test_support::install_trace`), every record read in order. The
record each step produced is in brackets; `plans/trace_spine.py` renders the
same records for any board.

| # | Step | Owner | Where | Reads, and the seam |
|---|---|---|---|---|
| 1 | Spark Double resolves. CR 110.2b's default controller rides on `GameState::resolving` while the `StackEntry` is gone; a permanent spell then enters (CR 608.3a) | stack | `resolve_top_of_stack` `engine/stack.rs:28`; the entry at `engine/stack.rs:205` | — |
| 2 | Entering is one event. `change_zone` routes a battlefield destination to an entry proposal, never a `ZoneChange` onto the battlefield; `entry_proposal` seeds `EnterMods::NONE` and refuses CR 800.4b's departed controller | chokepoint | `change_zone` `engine/actions.rs:1123`, `propose_entry` `engine/actions.rs:1840`, `entry_proposal` `engine/actions.rs:1816` | S1 · [rc-4b](traces/rc-4b-entering-is-one-event.html) |
| 3 | The batch opens: the outermost call mints a `BatchId`, a nested one joins it [`batch`] | chokepoint, events | `execute_actions` `engine/actions.rs:647`, `open_batch` `events/event.rs:602`, `execute_batch_inner` `engine/actions.rs:814` | `replacement-architecture.md` §4.2 |
| 4 | Phase 1 groups members by subject, APNAP by chooser; an entry is its own subject | replacement | `subject_of` `engine/replacement/gather.rs:88`, `apnap_batch_order` `engine/actions.rs:1049`, phase 1 at `engine/actions.rs:857` | [rd-2](traces/rd-2-a-decision-is-per-subject.html) |
| 5 | CR 616.1, iteration 1. The frame is built lazily; a "can't" is asked first; then the gather: source 1a reads Spark Double's own `SourceOnly` copy effect off the CR 614.12 frame [`layer_walk` entering, 0/0], and the battlefield sweep reads Biomancer's effective list [`layer_walk` member] | replacement | `apply_replacements` `engine/replacement/pipeline.rs:318`; `EntryFrame::new` `engine/replacement/pipeline.rs:385`; `is_prohibited` `engine/replacement/pipeline.rs:400`; `gather` `engine/replacement/pipeline.rs:412` | S2, S3, S4 |
| 6 | Two candidates, one step: CR 616.1c's copy step outranks Biomancer's 616.1e, so `must_choose_among` leaves one and nobody is asked which [`pipeline` iteration 1, `single`] | replacement | `must_choose_among` `engine/replacement/gather.rs:1082`; `Decided::Single` `engine/replacement/pipeline.rs:463` | `replacement-architecture.md` §4.1 |
| 7 | The copy. The donor prompt is the event's only one, and the "you may" is its empty pick [`decision` `ChooseCopySource`, bounds (0, 1)]; the donor's copiable values are captured once; the three exceptions are made, each CR 707.9f condition checked on a frame of the copy without itself; CR 614.17d is asked of the added counter | replacement → copy | `Rewrite::EnterAsCopy` `engine/replacement/pipeline.rs:1987`, `entry_copy_donor` `engine/replacement/pipeline.rs:2878`, `ask_may_choose_copy_source` `ui/ask.rs:1282`, `copiable_values` `engine/layers/copy.rs:279`, `enter_as_copy` `engine/replacement/entry_copy.rs:29`, `strip_prohibited_counters` `engine/replacement/pipeline.rs:2929` | S5 · [cv-1](traces/cv-1-a-copy-is-a-snapshot.html), [cv-2b](traces/cv-2b-an-exception-is-checked-without-itself.html) |
| 8 | Iteration 2 re-gathers against the rewritten entry (CR 616.1f; **loop 1**): the frame is now a 2/2 Soul Warden, Spark Double's own effect is in CR 614.5's applied set, and Biomancer's `EnterWith` is the one candidate. Its amount reads Biomancer off the real board (2), and the Mutant is an edit on the entry [`pipeline` iteration 2] | replacement | `Rewrite::EnterWith` `engine/replacement/pipeline.rs:1951`, `evaluate_enter_template` `engine/replacement/pipeline.rs:2980` | S6, S7 · [rg](traces/rg-an-entry-write-changes-what-applies-next.html), [rc-5](traces/rc-5-applying-an-entry-can-move-the-board.html) |
| 9 | Iteration 3 gathers nothing new [`pipeline` iteration 3, frame 4/4], and the loop returns the decided entry: the copy, three counters, the Mutant | replacement | the empty gather's return, `engine/replacement/pipeline.rs:436` | — |
| 10 | Before performing, CR 603.10's frames are taken while the board is untouched; nothing departs here | chokepoint → triggers | `departs_an_ability_list_source` `engine/triggers/dispatch.rs:549`, `capture_departure_frames` `engine/triggers/dispatch.rs:572`, called at `engine/actions.rs:946` | S8 |
| 11 | Phase 2. The move, then its announcement, then the entity. The move first ends every reference to the object as it was (CR 400.7: rows that named it pruned, save a permanent spell's under 400.7a and 400.7c); then `PermanentState::entering`, the constructor the frame used; registration files the copy's abilities, Soul Warden's trigger among them; the entry is announced [`event` ZoneChange, `event` ETB, `batch_end`] | chokepoint | `perform_action` `engine/actions.rs:1153`, `perform_zone_change` `engine/actions.rs:1735`, `move_object` `engine/zones.rs:47`, `break_references_to` `engine/zones.rs:456`, `announce_zone_change` `engine/actions.rs:1776`, `place_on_battlefield` `state/game_state.rs:1298`, `PermanentState::entering` `state/battlefield.rs:239`, `register_static_effects` `state/game_state.rs:1583`, `emit_event` `state/trace.rs:485` | S9, S10, S11 |
| 12 | Phase 3: no rider was queued | replacement | `resolve_rider` `engine/actions.rs:1082` | `replacement-architecture.md` §4.1a |
| 13 | The outermost close dispatches the window: histories first, then the gate reads `trigger_sources` and finds both Soul Wardens | triggers | `dispatch_after_batch` `engine/actions.rs:682`, `dispatch_batch` `engine/triggers/dispatch.rs:297`, `advance_history` `engine/triggers/history.rs:64`, `detect` `engine/triggers/dispatch.rs:430`, `battlefield_readers` `engine/triggers/dispatch.rs:531` | S12 · [tr-1](traces/tr-1-a-trigger-is-matched-at-the-close.html) |
| 14 | The match reads each candidate's effective list, Spark Double's being the copy's since layer 1 applies its `entered_as`. The original matches the ETB record; the copy is refused by "another" [`trigger` ×4] | triggers → layers | `find_matches` `engine/triggers/dispatch.rs:774`, `match_def` `engine/triggers/dispatch.rs:1007` | S13 |
| 15 | Queue, not resolve: a `PendingTrigger` on `GameState`, and an unstamped `AbilityTriggered` dispatched as it is emitted (CR 603.3b's second tier) [`event` AbilityTriggered] | triggers | `queue_matches` `engine/triggers/dispatch.rs:469`, `emit_event_unstamped` `state/trace.rs:492` | `triggers-architecture.md` §4.8 |
| 16 | The next CR 117.5 moment: SBAs until none (**loop 3**), then placement in APNAP order with the tiers, a stack object with `is_spell` false and its targets chosen per instance [`pending`] | priority, triggers | `perform_sba_and_triggers` `engine/priority.rs:288`, `check_state_based_actions_loop` `engine/sba.rs:539`, `place_pending_triggers` `engine/triggers/placement.rs:32`, `place_one` `engine/triggers/placement.rs:155` | `triggers-architecture.md` §5 · [a4i](traces/a4i-a-target-belongs-to-an-instance.html) |
| 17 | It resolves and proposes `GainLife` through the same chokepoint: a new batch, an empty gather, `LifeChanged`, another dispatch that matches nothing (**loop 4**) | stack, resolve | `resolve_effect_with_announced_targets` `engine/resolve.rs:151`, called at `engine/stack.rs:167` | S14 |

### 1.3 The seams

Each is one subsystem reading another's output, in path order.

| | Seam | Where | Doc | Trace |
|---|---|---|---|---|
| S1 | The chokepoint routes entering to one proposal, so every reader of an entry sees one event | `change_zone` `engine/actions.rs:1123` | `replacement-architecture.md` §9 (RC-4b) | [rc-4b](traces/rc-4b-entering-is-one-event.html) |
| S2 | **Replacement reads layers**: the entering object's CR 614.12 frame. `EntryFrame` decides when; `compute_as_entering` computes it through the walk's accessor pair (§2.4, §2.5) | `frame_of` `engine/replacement/lookahead.rs:93`, `compute_as_entering` `engine/layers/lookahead.rs:156` | `replacement-architecture.md` §5, §5d | [rc-4b](traces/rc-4b-entering-is-one-event.html), [li-1](traces/li-1-one-pass-per-board.html) |
| S3 | **Replacement reads layers and registration**: the gather reads effective ability lists, behind a gate whose legs registration and the registry summary keep (§2.3) | `gather` `engine/replacement/gather.rs:180` | `replacement-architecture.md` §3.3 | [rf](traces/rf-a-source-off-the-battlefield.html) |
| S4 | **Replacement reads restriction**: a "can't" is asked ahead of the gather and wins (CR 101.2, 614.17) | `is_prohibited` `engine/restriction/predicate.rs:60` | `cant-effects-architecture.md` §4.1, §5.3 | — |
| S5 | **Replacement runs copy, copy reads layers**: the rewrite captures the donor's copiable values at the end of layer 1 and carries them on the proposal | `enter_as_copy` `engine/replacement/entry_copy.rs:29`, `copiable_values` `engine/layers/copy.rs:279` | `copy-effects-architecture.md` §4.1, §4.1a | [cv-1](traces/cv-1-a-copy-is-a-snapshot.html), [cv-2b](traces/cv-2b-an-exception-is-checked-without-itself.html) |
| S6 | **Replacement reads its own output**: an entry's writes decide what the next iteration gathers, and the feeds table is what may skip the prompt | `EntryWrites` `engine/replacement/pipeline.rs:1414`, `ordering_cannot_change_outcome` `engine/replacement/pipeline.rs:956` | `replacement-architecture.md` §3.5 | [rg](traces/rg-an-entry-write-changes-what-applies-next.html) |
| S7 | **An amount reads the real board for an existing source and the frame for the entering one** | `evaluate_enter_template` `engine/replacement/pipeline.rs:2980` | `replacement-architecture.md` §5b | [rc-5](traces/rc-5-applying-an-entry-can-move-the-board.html) |
| S8 | **Triggers read the batch's decisions**: CR 603.10's look-back frames and snapshots are taken between deciding and performing | `capture_departure_frames` `engine/triggers/dispatch.rs:572` | `triggers-architecture.md` §4.3, §6.1 | — |
| S9 | **Layers read the performer's entity**: the copy and the edits an entry fixed apply from `PermanentState::entered_as`, the copy at layer 1 | `applications_in_layer` `engine/layers/board.rs:923`, the notes at `engine/layers/board.rs:963` | `replacement-architecture.md` §3.5; `copy-effects-architecture.md` §4.1 | [rg](traces/rg-an-entry-write-changes-what-applies-next.html) |
| S10 | **Every gate reads registration**: the arrived-with abilities, an entry copy's included, go into the printed legs, `trigger_sources` among them | `register_static_effects` `state/game_state.rs:1583`, the trigger leg at `state/game_state.rs:1624` | `copy-effects-architecture.md` §4.7; `triggers-architecture.md` §4.2 | — |
| S11 | **Triggers read events**: `emit_event` is the one door, and every record carries its batch | `emit_event` `state/trace.rs:485` | `triggers-architecture.md` §4.1 | [tr-1](traces/tr-1-a-trigger-is-matched-at-the-close.html) |
| S12 | **Triggers read the chokepoint's close**: the window is dispatched when the outermost batch returns, after its riders | `dispatch_after_batch` `engine/actions.rs:682` | `triggers-architecture.md` §4.1 | [tr-1](traces/tr-1-a-trigger-is-matched-at-the-close.html) |
| S13 | **Triggers read layers**: the matcher asks each candidate's effective ability list | `find_matches` `engine/triggers/dispatch.rs:774` | `triggers-architecture.md` §4.2 | [tr-1](traces/tr-1-a-trigger-is-matched-at-the-close.html) |
| S14 | **The chokepoint reads triggers' output**: a resolution proposes through `execute_action` like any other | `resolve_effect` `engine/resolve.rs:129` | `triggers-architecture.md` §6 | — |

### 1.4 The loops the path passes

Named only; `plans/references/feedback-loops.md` counts the cards that close
each, and reads these seams for a fifth loop, finding none.

1. **An entry's writes and CR 616.1's re-gather** — step 8, S6.
2. **A layer effect and CR 613.8's dependency** — inside every walk: `depends_on` `engine/layers/board.rs:1433`, `next_ready` `engine/layers/board.rs:1483`; [item-7](traces/item-7-an-effect-waits-for-what-it-reads.html).
3. **A state-based action's result and the next check (CR 704.3)** — step 16.
4. **An event a triggered ability produces and what else triggers (CR 603)** — steps 15 and 17.

---

## 2. The structure

### 2.1 The modules and what each owns

143 files under `mtgsim/src`; `mod.rs` files declare and re-export only
(`engineering-practices.md` §6).

| Module | Owns |
|---|---|
| `engine/actions.rs` | the chokepoint: `GameAction`, `ActionContext`, `execute_actions`' three phases, `perform_action`'s arms, `change_zone`, entry and token proposals |
| `engine/replacement/` | CR 614–616: `pipeline.rs` the CR 616.1 loop, `apply_rewrite` and the riders; `gather.rs` the five sources and their gate; `lookahead.rs` `EntryFrame`; `entry_copy.rs` CR 707.9's exceptions on an entry copy; `instance.rs` CR 614.5's identity |
| `engine/restriction/` | CR 101.2 and 614.17: `predicate.rs` `is_prohibited`, the one reader of every "can't" |
| `engine/cost_determination/` | CR 601.2f: `gather.rs` which cost effects apply; `total.rs` the arithmetic and its order |
| `engine/layers/` | CR 613: `compute.rs` `compute_characteristics` and the memo's door; `board.rs` the board-wide pass, the accessor pair and CR 613.8's order; `lookahead.rs` the CR 614.12 overlay; `copy.rs` copiable values; `cda.rs` CDAs at layers 4, 5 and 7a; `condition.rs` "as long as"; `intrinsic.rs` CR 306.5b's loyalty ability; `land_types.rs` CR 305.6–305.7; `types.rs` the vocabulary |
| `engine/triggers/` | CR 603: `dispatch.rs` detection at a window's close and the CR 603.10 frames; `delayed.rs` CR 603.7's registry, a leg of every dispatch; `placement.rs` CR 603.3; `binding.rs` the bound facts at resolution; `bound_reads.rs` what a def reads, for the ordering elision; `history.rs` the histories' one writer; `audit.rs` the dispatch audit |
| `engine/stack.rs`, `engine/resolve.rs` | CR 608: resolution and fizzle; `ResolutionContext` and each `Primitive` turned into proposals and registry rows |
| `engine/put_on_stack.rs` | CR 601.2 and 602.2: `cast_spell`, `activate_ability`, the announcement, the mana window |
| `engine/priority.rs`, `engine/sba.rs`, `engine/turns.rs` | CR 117 priority rounds and CR 117.5's `perform_sba_and_triggers`; CR 704; CR 500's turn, phase and step proposals and the turn-based actions |
| `engine/zones.rs`, `engine/leaving.rs` | `move_object`, the one mover, and zone-state cleanup; CR 800.4, a player leaving |
| `engine/combat/` | CR 506–511: `steps.rs` the combat steps; `validation.rs` attack and block legality; `resolution.rs` damage assignment; `keywords.rs` first strike, double strike, trample |
| `engine/targeting.rs` | CR 115 and 601.2c: target instances, validation, the selection-side `object_matches_filter` |
| `engine/costs.rs`, `engine/mana.rs` | CR 601.2h/602.2b payment; CR 605 mana abilities |
| `engine/zone_function.rs`, `engine/keywords.rs`, `engine/trace_records.rs` | CR 113.6's `functions_in`; lifelink and deathtouch on damage; every trace record's shape |
| `state/game_state.rs` | `GameState` and `StackEntry`; timestamps and `battlefield_ids_ordered`; `place_on_battlefield`, `register_static_effects` and the printed-source sets |
| `state/battlefield.rs`, `state/player.rs` | `PermanentState` and its one constructor for an entry; `PlayerState` |
| `state/continuous_effects.rs`, `state/replacement_effects.rs`, `state/restrictions.rs`, `state/duration_registry.rs` | the three registries of resolution-created rows, their shared CR 514.2 expiry, and `RegistryScopeSummary`, the gates' granted and copied legs |
| `state/layer_memo.rs`, `state/history.rs`, `state/diagnostics.rs`, `state/trace.rs` | the layer memo; each player's bounded history; seed-deterministic work counters; the trace sink and `emit_event` |
| `state/game.rs`, `state/game_config.rs` | `Game`: setup and the turn loop a harness drives; format configuration |
| `events/event.rs`, `events/recorder.rs` | `GameEvent`, the batch window and `BatchId`; the whole stream for readers outside the game |
| `objects/card_data.rs`, `objects/object.rs` | `CardData`, `AbilityDef` and the builder; `GameObject` |
| `oracle/characteristics.rs` | the layer-routed accessors every reader of a characteristic uses |
| `oracle/legality.rs`, `oracle/mana_helpers.rs`, `oracle/board.rs` | candidate actions and legal selections; castability and `activatable_abilities`; board queries |
| `types/` | the type surface: `effects.rs`, `replacement.rs`, `restriction.rs`, `triggers.rs`, `cost_modification.rs`, `costs.rs`, `zones.rs`, `ids.rs`, `card_types.rs`, `colors.rs`, `mana.rs`, `keywords.rs`, `keyword_actions.rs`, `history.rs`; `mod.rs` holds `counted_enum!` |
| `ui/decision.rs`, `ui/ask.rs`, `ui/choice_types.rs` | the four-method `DecisionProvider`; the typed `ask_*` bridge, where every prompt is validated and traced; `ChoiceKind` |
| `ui/random.rs`, `ui/cli.rs`, `ui/mana_window_stop.rs`, `ui/auto_payer.rs`, `ui/auto_yield.rs`, `ui/full_control.rs`, `ui/display.rs` | two providers, three decorators, the full-control switch above a person's stack, the text formatters |
| `cards/registry.rs`, `cards/random_deck.rs`, `cards/authoring/` | the registry and the two pools; the fuzz harness's decks; the card-authoring words, `triggers.rs` today |
| `cards/*.rs` | card definitions, filed by the phase that first needed them |
| `bin/fuzz_games.rs`, `bin/cli_play.rs`, `test_support.rs`, `lib.rs`, `main.rs` | the random-versus-random harness; terminal play; the `test-support` feature's helpers; the crate root; a stub |

### 2.2 The chokepoint's arms

`perform_action` `engine/actions.rs:1153` has one arm per `GameAction` variant
(`engine/actions.rs:101`) and no wildcard. A nested proposal is **contained**
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
| `Attach` | `GameState::attach`, nothing if already there; `Attached` | — |
| `Tap` | on the transition only; `Tapped` | — |
| `AddCounters` | counters on a permanent or a player; `CountersChanged` | — |
| `RemoveCounters` | as many as there are (CR 701.2); announced only if any | — |
| `Destroy` | the outer event only | `ZoneChange` to the graveyard |
| `EnterBattlefield` | the move (or a token's creation), then `place_on_battlefield` | — |
| `CreateTokens` | the objects, then every entry as one batch | `EnterBattlefield` per token |
| `CreateTokenIn` | `put_token_into`; `TokenCreated` | — |
| `BeginTurn` | `begin_turn`; `TurnBegin` | — |
| `BeginPhase` | the phase; `PhaseBegin` | — |
| `BeginStep` | the step; `StepBegin` | — |
| `Scry` | CR 701.22 whole in the arm, the prompt included (`ask_scry`); `Scried` | — |
| `ShuffleLibrary` | `shuffle_library`, drawing on `GameState::rng`; `LibraryShuffled` | — |
| `PlayerLoses` | the loss and `PlayerLost`, then CR 800.4a | `player_left_the_game`'s moves, joining the batch |
| `PlayerWins` | the result; `PlayerWon` | — |
| `ProduceMana` | the pool, its one writer; `ManaAdded` | — |
<!-- arms:end -->

Three doors sit beside `execute_action` `engine/actions.rs:598`:
`execute_actions` for a simultaneous event, `execute_actions_decomposing`
`engine/actions.rs:723` for CR 121.2's draws, and `execute_actions_new_batch`
`engine/actions.rs:765` for `// AUXILIARY-MOVE:` (CR 614.13). The
`// CAST-ROLLBACK:` exemption is `rollback_cast_to_hand`
`engine/put_on_stack.rs:629`, and `announce_zone_change` has one caller outside
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
| replacement | `gather` `engine/replacement/gather.rs:180`; the gate at `engine/replacement/gather.rs:262` | `replacement_ability_sources`, `zone_replacement_ability_sources` | one pair for both since RF: `unattributed_replacement_zones`, `any_named_unattributed_replacement` |
| restriction | `is_prohibited` `engine/restriction/predicate.rs:60`; the gate at `engine/restriction/predicate.rs:82` | `restriction_ability_sources` | `any_granted_restriction`, `any_copied_restriction` |
| cost | `cost_modifications_for` `engine/cost_determination/gather.rs:54`; the gate at `engine/cost_determination/gather.rs:63` | `cost_modification_ability_sources` | `any_granted_cost_modification`, `any_copied_cost_modification` |
| trigger | `detect` `engine/triggers/dispatch.rs:430`; the gate at `engine/triggers/dispatch.rs:446` | `trigger_sources`, `zone_trigger_sources` | one pair for both: `unattributed_trigger_zones` for a `Filter` row's zones, `named_unattributed_trigger_kinds` for a named row's objects; a departure's CR 603.10a frame and a look-back snapshot are two more probes |

The printed legs are written by `register_static_effects`
`state/game_state.rs:1583` as an object arrives; the other two are
`RegistryScopeSummary` `state/continuous_effects.rs:51`, recomputed from the
registry on every change. → `copy-effects-architecture.md` §4.7,
`triggers-architecture.md` §4.2.

### 2.4 The two `object_matches_filter`s, and the accessor pair

- **Inside the walk**: `object_matches_filter` `engine/layers/compute.rs:671`
  asks whether a continuous effect applies to a member, off a partly applied
  frame, with "you" resolved through the effect's source (`FilterPlayers`
  `engine/layers/compute.rs:456`).
- **Outside it**: `GameState::object_matches_filter` `engine/targeting.rs:633`
  asks whether an object is a legal selection, or inside a replacement's or a
  restriction's `ObjectSet`, with "you" passed in. Its variants
  `object_matches_filter_for_instance` `engine/targeting.rs:661` (CR 601.2c's
  "another target"), `object_matches_filter_of_source`
  `engine/targeting.rs:694` (`NotSource`; `set_affects`' door) and
  `object_matches_filter_in_frame` `engine/targeting.rs:718` (a CR 614.12
  frame) share one leaf table, `object_matches_filter_with`
  `engine/targeting.rs:740`. The two must agree: "enchant creature you control"
  means one thing to CR 704.5n and to a static ability.
- **The accessor pair**: every concrete-state read the walk makes goes through
  `Board::entity` `engine/layers/board.rs:342` (the entity: controller, CR
  302.6's clock, counters) or `rows_in_layer` `engine/layers/board.rs:904`
  (the registry's slice). That is where the look-ahead answers for one object
  without a `GameState` clone; `Board::timestamp_of`
  `engine/layers/board.rs:362` is the third read it answers.

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
| the priority loop | `run_priority_round` `engine/priority.rs:38`; locals at `engine/priority.rs:84` | the last rejection, which the re-ask names and nothing filters by since item 193, and `rejections` | not outcome-bearing; a resume mid-round is `codebase-state.md` item 140 |
| the mana window | `run_mana_ability_window` `engine/put_on_stack.rs:540`; the set at `engine/put_on_stack.rs:566` | `failed` | not outcome-bearing |
| CR 601.2b–d's announcement | `cast_spell` `engine/put_on_stack.rs:52`, rewound by `rollback_cast_to_hand` `engine/put_on_stack.rs:629` | the unpushed `StackEntry` | violator 2 |
| the CR 616.1 loop | `apply_replacements` `engine/replacement/pipeline.rs:318`; the sets at `engine/replacement/pipeline.rs:339` | `applied`, `declined`, `exempt_applied`, the rewritten members | violator 1 |
| the fix's shape | `entry_selection` `state/game_state.rs:532`, `prevention_allocations` `state/game_state.rs:542`, `pending_triggers` `state/game_state.rs:584` | — | on `GameState`; placement drains the queue, so a clone resumes |

### 2.7 Every `CLAUDE.md` invariant, where it lives

| Invariant | Lives at | Doc |
|---|---|---|
| Characteristics of a battlefield or stack object come through the layers | `oracle/characteristics.rs` → `compute_characteristics` `engine/layers/compute.rs:94`; exemptions tagged `// PRE-LAYER ZONE:` and `// AS PRINTED:` (`printed_faces` `ui/display.rs:160`), and `register_static_effects` `state/game_state.rs:1583` | `layers-architecture.md` §11.1 |
| Ability indices are into the effective list | `activatable_abilities` `oracle/mana_helpers.rs:347`, the re-derivation by id at `engine/priority.rs:166`, `activate_ability` `engine/put_on_stack.rs:361` | `cant-effects-architecture.md` §4.4 |
| Registry membership is not effect existence | `static_ability_still_exists` `engine/layers/board.rs:1099`, asked per layer; the descending ceiling is `Board::frame_of` `engine/layers/board.rs:411` | `layers-architecture.md` §5.2 |
| CDAs are never registry effects | `cda_modifications` `engine/layers/cda.rs:71`; skipped at registration, `state/game_state.rs:1692`; a Layer 6 grant clears the flag at `engine/layers/compute.rs:1119`, in `apply_resolved` | `layers-architecture.md` §6 |
| No observable mutation outside `perform_action`'s arms | `perform_action` `engine/actions.rs:1153` | `replacement-architecture.md` §2 |
| A `ZoneChange` names its cause, with no catchall | `ZoneChangeCause` `types/zones.rs:189` | `replacement-architecture.md` §3.1 |
| Performers are loud; callers check legality | the `Untap` arm `engine/actions.rs:1371`; `Primitive::Untap` `engine/resolve.rs:799` | `replacement-architecture.md` §9 (RA-2) |
| "Becomes" events fire on the transition only | the `Tap` arm `engine/actions.rs:1408` | CR 603.2e |
| An observable order goes through the timestamp order | `battlefield_ordered` `state/game_state.rs:1121`, `battlefield_ids_ordered` `state/game_state.rs:1130`, keyed on `PermanentState::timestamp` `state/battlefield.rs:62` | `plans/id-hasher.md` §6 |
| One performer, one emitter | `move_object` `engine/zones.rs:47`, `announce_zone_change` `engine/actions.rs:1776` | `replacement-architecture.md` §11, item 20 |
| A simultaneous rule needs a batch | `execute_actions` `engine/actions.rs:647`; `execute_actions_new_batch` `engine/actions.rs:765` | `replacement-architecture.md` §4.2 |
| No outcome-bearing decision state off `GameState` | §2.6 | `codebase-state.md` item 40 |
| Never prompt with fewer than two candidates | `Decided::Single` `engine/replacement/pipeline.rs:463` | `replacement-architecture.md` §4.1 |
| A "can't" is not a replacement effect | `is_prohibited` `engine/replacement/pipeline.rs:400`, ahead of the gather | `cant-effects-architecture.md` §4.1 |
| A static replacement ability is found off the effective list | `gather` `engine/replacement/gather.rs:180`; the legs, §2.3 | `replacement-architecture.md` §3.3 |
| Declining is tracked apart from CR 614.5's set | `declined` `engine/replacement/pipeline.rs:343` | `replacement-architecture.md` §4.1 |
| Deciding is separated from performing | phase 1 `engine/actions.rs:857`, phase 2 `engine/actions.rs:959` | `replacement-architecture.md` §4.2 |
| Riders run after the performed event, carrying its applied set | phase 3 `engine/actions.rs:1032`; `Rider::lineage` `engine/replacement/pipeline.rs:76` | `replacement-architecture.md` §4.1a |
| `EventPattern` grows one arm per variant; `Rewrite` is closed; variety goes in `then` | `EventPattern` `types/replacement.rs:203`, `Rewrite` `types/replacement.rs:843`, `ReplacementDef::then` `types/replacement.rs:136` | `replacement-architecture.md` §3.2a, §3.2b |
| Randomness is owned | `GameState::rng` `state/game_state.rs:660`; the provider's own `rng` `ui/random.rs:163`; `IdHash` `types/ids.rs:268`; `tests/determinism_test.rs` | `plans/id-hasher.md` §6 |

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
