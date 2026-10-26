# What the engine does not ask, and what it does not compute

**The inventory `codebase-state.md` item 185 asked for.** The engine sometimes
answers a question for a player, or skips a piece of work, because it has
decided the result cannot change the game. When that decision is right it saves
a prompt or a search. When it is wrong, nothing fails: the engine asks one
question fewer or computes one thing fewer, and the game is wrong only on a
board nobody foresaw. This page lists every such place, what each one claims,
the condition the claim depends on (its **premise**), what checks that
condition, and what the game's log records when it is taken (§6).

**Maintained, and checked.** `plans/check_engine_map.py` reads this page the
way it reads the engine map: a function named beside a `file:line` must exist
in that file, so a renamed site fails the check instead of leaving a stale row,
and `--drift` lists the lines that moved. `engine-map.md` §2.3, §2.6 point
here.

## 1. Three kinds

| Kind | What the engine does | Example |
|---|---|---|
| **forced prompt** | the question has one legal answer, and the engine gives it | Lightning Bolt with one legal target |
| **elision** | the question has two or more legal answers that all leave the same game, and the engine picks one | two Soul Warden triggers from two creatures entering at once |
| **skipped work** | a cheap check says an expensive search would find nothing, and the search is skipped | no permanent can have a replacement ability, so the pipeline does not look at each one |

The first two words are `glossary.md`'s. Item 185 called all three
"elisions"; this page keeps the glossary's narrower sense.

**Not on this page**: places where the rules give no choice at all, so there
is nothing to skip. A rule that chooses only among two or more gives none
below two: CR 616.1 with one replacement effect (`Decided::Single`), one
triggered ability to put on the stack, one scried card to order. An
instruction with nothing to choose from (CR 101.3): no creature able to
attack, a card with no alternative cost, a scry of an empty library, a count
of zero, a division of nothing. And a cheaper route that reads everything its
answer depends on: `next_ready` `engine/layers/board.rs:1636`'s key-first
application reads the head's dependencies, `compute_non_member_recorded`
`engine/layers/compute.rs:369` exits only when no CDA and no noted row reaches
the object, and `any_replacement_counter` `engine/replacement/gather.rs:1018`
is computed, not cached.

## 2. Forced prompts

**No CR rule says a forced choice is skipped.** The tree once cited CR 102.2
for it, but 102.2 is "In a two-player game, a player's opponent is the other
player"; A4k's census caught the slip (`backlog.md` §2.22, rule 1), and #278
corrected every citation. The claim is the engine's own: a question with one
legal answer gets that answer from any player or bot, so not asking changes
nothing.

**The premise every row shares**: the candidates and the count are every
legal answer, exactly the list the prompt would have shown. So a forced
prompt assumes nothing the prompt itself does not: if the list were wrong,
the prompt would have been wrong too. **Every one is decided in `ui/ask.rs`**,
by the function the decision counter uses: a pick through `pick_unless_forced`
`ui/ask.rs:118`, which asks `only_pick` `ui/ask.rs:242`; a split through
`allocate_unless_forced` `ui/ask.rs:141`, which asks `forced_allocation`
`ui/ask.rs:73`; X through `has_one_value`, which `only_number`
`ui/ask.rs:306` restates. The guard and the count cannot drift apart, and a
new ask that calls the helper inherits both, and the record.

| Question | Where it is answered | One answer when |
|---|---|---|
| a target or a "choose", CR 601.2c, 608.2d | `ask_select_recipients` `ui/ask.rs:736` | the count is fixed and takes every legal candidate |
| which cards to discard, CR 701.9b | `ask_discard` `ui/ask.rs:970` | the count takes the whole hand |
| the value of X, CR 601.2b, 107.3a | `ask_choose_x_value` `ui/ask.rs:649` | one value is legal (`legal_x_values`) |
| a split: trample (CR 702.19b), generic mana (601.2h), "the next N damage" (615.7) | `allocate_unless_forced` `ui/ask.rs:141`, called by the four asks that split | nothing is left once each part has its minimum; or nothing is spare once each has its maximum; or only one part can take the rest, and it can hold it. `forced_allocation`'s doc argues the three shapes and `ui/ask.rs`'s unit tests pin them. A split it answers never reaches `validate_allocation` `ui/ask.rs:312`, so it answers `None` to anything malformed. Combat damage among blockers (CR 510.1c) has no per-blocker maximum, so it is forced only at power 0, which divides nothing |
| which permanents to sacrifice for a cost, CR 601.2h, 701.21a | `ask_choose_sacrifice_for_cost` `ui/ask.rs:1332` | there are exactly as many candidates as the cost takes |
| what a copy effect copies, CR 608.2d, 614.12a | `ask_choose_copy_source` `ui/ask.rs:1228` | one candidate, and the copy is not a "may" |
| a source of damage, CR 609.7a | `ask_choose_damage_source` `ui/ask.rs:1278` | one candidate |
| "an opponent of your choice" as a permanent enters, CR 614.12a: to control it (CR 616.1b), or to put its counters (CR 702.104a) | `ask_choose_player_as_it_enters` `ui/ask.rs:1140` | one opponent |
| what a creature put onto the battlefield attacking attacks, CR 508.4 | `ask_choose_entering_defender` `ui/ask.rs:1160` | one candidate |
| whether to pass priority, CR 117.3d | `run_priority_round` `engine/priority.rs:38` | passing is the only legal action, and the seat does not stop at every priority point |

The priority pass is the one row decided outside `ui/ask.rs`, and the one not
recorded in the game's log: the decision log writes it as a forced line, and
in the game's log it would add half the lines at two seats and four-fifths at
four (§6).

## 3. Elisions

The risky kind: there really are several answers, and the engine claims they
all leave the same game. Each claim rests on an argument about what the
effects involved can read and write.

| Question | Where it is skipped | The premise | What checks it |
|---|---|---|---|
| CR 616.1: which of several replacement effects applies first, among those the player must choose from at that step of 616.1a–e | `ordering_cannot_change_outcome` `engine/replacement/pipeline.rs:1081` | applying one never stops another from applying, never changes what another writes, is never overwritten by another, and never makes an effect apply that would multiply another's write | `replacement-architecture.md` §3.5's feeds table: its inputs are matched exhaustively (`EntryWrites` `engine/replacement/pipeline.rs:1536`), so a new kind of write does not compile until it is classified. `check_order_invariance` `engine/replacement/pipeline.rs:1746` re-checks the first property after every skipped prompt, in debug builds only, so a release game pays nothing. Two tests reach every order for the fourth property |
| CR 603.3b: the order of one player's triggered abilities that triggered together | `trigger_order_cannot_change_outcome` `engine/triggers/placement.rs:119` | they are the same ability, in the first of 603.3b's two tiers, and they agree on every fact the ability reads off its trigger: which object, which player, how much, its power, the source, X, the ability's own count (`entries_agree_on` `engine/triggers/placement.rs:128`) | `trigger_binding_reads` `engine/triggers/binding_reads.rs:108` lists what an ability reads, and it is an exhaustive match: a new kind of effect does not compile until it says what it reads. `binding_reads_are_named` `engine/stack.rs:326` checks the list in every debug test and game: each fact a resolving triggered ability reads off its entry is noted where it is read (`note_binding_read` `engine/resolve.rs:2948`), and must be one its def names. TR-2b's and TR-5a's tests pin both answers. A target or a mode is no condition: each is chosen as its entry goes on the stack (CR 603.3c, 603.3d), and agreeing entries are interchangeable, so any order with any sequence of choices builds the stack the first order builds with the same sequence (item 317; TR-1's `one_abilitys_agreeing_triggers_are_not_ordered_and_each_is_still_targeted`) |
| CR 603.7b: which of several events that happened at once triggers a delayed triggered ability | `choose_delayed_cause` `engine/triggers/delayed.rs:374` | the events agree on every fact the ability reads, as in the row above | the row above's list and its check; TR-3b's two tokens dying at once |
| CR 601.2f: the order of two or more cost reductions. **Owed, not taken** | asked in `modify` `engine/cost_determination/total.rs:95` | every order gives the same total (`cost-architecture.md` §3.4's theorem), until a reduction can be a hybrid symbol or have a floor | item 165, slotted at `hybrid-and-phyrexian-payment`; `AutoPayer` answers it until then |

**What the trigger check sees.** It notes the binding's facts where the
resolution reads them: the triggering objects by event role, the triggering
player, the amount, the triggering object's power or toughness, the referred
objects, X, and the source through `this_object` and its counters. A read of
the source by another route, such as a damage source or a mana ability's
producer, is not noted, so a leaf that reads the source that way and does not
name it is caught only by review and the tests.

## 4. Skipped work

**What a gate is.** Before an event happens, the replacement pipeline could
look at every permanent's abilities, computed through the layers, for a
replacement effect that applies: one layer walk per permanent, for every
event. The gate first asks three cheap questions:
- does any object's **printed** card have a replacement ability? The engine
  keeps a set of these, filled as objects arrive;
- does any continuous effect **grant** one (layer 6)?
- does any copy effect **copy** one onto something (layer 1)?

If all three say no, nothing on the battlefield can have one, and the search
is skipped. The three questions are the gate's **legs**
(`engine-map.md` §2.3). The premise is that an object can come to have such an
ability only by one of those three routes. A leg that says "yes" too often only
costs a wasted search. A missing leg is the danger: if the engine gains a
fourth route, and text-changing effects (layer 3) are the known one, the gate
answers "nothing here" on a board where something is, and the card silently
does nothing. The restriction, cost and trigger readers have gates of the same
shape.

**What an audit is.** Under `fuzz_games --audit`, which every A/B's counter
runs use, a skipped search is done anyway and the two answers compared: the
same sweep with its gate open (`Sweep::Everything`), over every object that
could carry an ability of its kind (`objects_that_may_carry`), and a panic
naming the gate and the seed when they differ (`assert_shortcut_agrees`
`engine/audit.rs:84`). Its reads leave no trace (`unobserved`
`engine/audit.rs:49`), so an audited game's counters and trace are an
unaudited one's. The switch is `enable_audit` `engine/audit.rs:34`. A test
takes each gate's printed leg away and sees the panic
(`tests/phase_a6h_integration_test.rs`).

| What is skipped | Where | The claim, and what it depends on | What checks it |
|---|---|---|---|
| looking at every permanent for a replacement effect | `gather` `engine/replacement/gather.rs:203`; the gate at `engine/replacement/gather.rs:292` | the three legs; counters that make replacement effects (shield, stun, finality) are found directly (`any_replacement_counter`), and the commander rule (CR 903.9b) and the entering object are asked before the gate | its audit: `gather_sweeping` `engine/replacement/gather.rs:221` with the gate open, every off-battlefield object asked with no printed precheck |
| looking at every permanent for a "can't" | `prohibition` `engine/restriction/predicate.rs:97`; the gate at `engine/restriction/predicate.rs:133` | the three legs; keywords that forbid things are asked before the gate | its audit: `prohibition_sweeping` `engine/restriction/predicate.rs:115` with the gate open |
| looking at every permanent for a cost increase or reduction | `cost_modifications_for` `engine/cost_determination/gather.rs:57` | the three legs, and the spell's own printed card (`prints_cost_ability`) | its audit: `cost_modifications_sweeping` `engine/cost_determination/gather.rs:69` over every permanent |
| looking at every object for a triggered ability an event triggers | `detect` `engine/triggers/dispatch.rs:735`; `check_state_triggers` `engine/triggers/state.rs:53`; `count_additional_triggers` `engine/triggers/dispatch.rs:1530` | the three legs, the lists of objects outside the battlefield with triggered abilities, the objects that just arrived, and a snapshot of every ability list taken only when an event moves an object that had one (`LookBackSnapshot`) | TR-1b's audit, `audit_dispatch` `engine/triggers/audit.rs:92`: each dispatch answered again with every shortcut off (`triggers-architecture.md` §4.10). It shares the matching loop, so it checks the gate and the candidates and not the loop |
| computing a permanent's controller through the layers, and the sweep that announces control changes (TR-4d) | `get_effective_controller` `oracle/characteristics.rs:167`; `settle_announced_controllers` `engine/triggers/control.rs:122`; `keep_frames_before_control_change` `engine/triggers/control.rs:58` | with no continuous effect changing control, every permanent is controlled by whoever it entered under; and if there was none at the last sweep either, nothing has changed hands. Both rest on layer 2 being the only thing that sets a controller (CR 613.1b) | the type: a copy effect's values (`CopiableValues`) have no controller field, so a copy cannot set one. Its audit, `audit_announced_controllers` `engine/triggers/control.rs:159`: after each sweep, every permanent's controller walked against the one announced. Between sweeps `get_effective_controller` is held by the same premise and checked only at the next sweep |
| settling control and combat after the layers' output changed | `settle_layer_consequences` `engine/layer_settlement.rs:36` | with the layer epoch where the last settlement left it, every frame is the one that settlement read, so there is nothing to find: the memo's premise, the row below | the memo's audit |
| recomputing an object's characteristics | `compute_characteristics` `engine/layers/compute.rs:103`; `LayerMemo` `state/layer_memo.rs:45` | a result stored at the current epoch is still right: every write to anything the layers read moves the epoch, and a few inputs (`GatedWalkInput` `state/layer_memo.rs:89`) move it only when a stored result read them | `audit_memo_hit` `engine/layers/compute.rs:210` recomputes every stored result it serves, in debug builds |
| checking each "target" clause when deciding whether a spell can be cast (A4q) | `every_instance_has_a_choice` `oracle/mana_helpers.rs:415` | two identical clauses that do not depend on earlier targets have the same answer | its audit answers the folded clause anyway. No test takes the premise away, since no board makes two equal clauses past the feed answer differently; Seeds of Strength, pooled, reaches it in audited games |
| walking every object for commanders that changed zones (CR 704.6d) | `arrivals_since_last_sba_check`, kept as objects arrive | every arrival the state-based check needs is noted | `moved_since` `engine/sba.rs:142` walks every object in debug builds and compares |
| the four feedback loops' skipped work | `references/feedback-loops.md` §2, §3, §4, §5 | loop 1's is the CR 616.1 row above; loop 2's pre-check covers every filter, condition and amount; loop 3 skips nothing; loop 4's is the dispatcher row | there |

## 5. What checks what

- **The compiler**: the feeds table's inputs; `trigger_binding_reads`.
- **Every debug test and game**: the memo's audit, `check_order_invariance`,
  the trigger check, the commanders' walk.
- **`--audit`**: the dispatcher's, the three gates', the control sweep's and
  the castability fold's audits.
- **Only an argument and tests**: the source read by a route the trigger check
  does not note (§3); `get_effective_controller` between two sweeps; and CR
  616.1's second to fourth properties outside the two tests that reach every
  order, which the feeds table's exhaustive matches hold at compile time.

## 6. The record

**Each forced prompt and elision the engine takes is recorded in the game's
log**: who would have been asked, the question, the answer and why
(`events::unasked`, `UnaskedChoice` `events/unasked.rs:19`). It is written
through `note_unasked` `events/recorder.rs:236` beside the event recorder's
records, placed before the record it preceded, and drawn by `format_event_log`
(`format_unasked` `ui/display.rs:1215`), so `--dump-events`, `cli_play`'s log
and a test's log carry it. Three lines from `fuzz_games --games 10 --seed
185 --dump-events` at #278:

```
NotAsked: P1 SelectRecipients for Act of Treason (#71): Wall of Stone (#7), its one legal answer
NotAsked: P0 GenericManaAllocation for Wall of Stone (#7): 1 to mana {W}, 0 to mana {R}, its one legal answer
NotAsked: P0 ChooseReplacementEffect: Mana Reflection (#7), every order leaves one game (CR 616.1)
```

The dev GUI's log does not draw it (`log_lines` `ui/display.rs:1164` reads
records only), as it does not draw a mana ability's activation. An action
reversed under CR 732.1 takes back the questions it answered unasked, since the
log shows nothing of it (`take_back_unasked` `events/recorder.rs:228`); the
decision log keeps its answers, which a replay asks again. It is an
observer: it takes no `EventId`, enters no window, matches no trigger, moves
no counter and draws no randomness, so a game performs the same events
whether anything records. Skipped work is not recorded each time, since it
happens per dispatch or per walk; its audits hold it instead.

**Adding a site.** A new forced prompt is decided in `ui/ask.rs` through
`pick_unless_forced` or `allocate_unless_forced`, and inherits its record. A
new elision writes `note_unasked` with an `Elision` and gets a row in §3. A
new gate or sweep gets an audit through `engine::audit` and a row in §4.
