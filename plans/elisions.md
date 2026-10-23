# What the engine does not ask, and what it does not compute

**The inventory `codebase-state.md` item 185 asked for.** The engine sometimes
answers a question for a player, or skips a piece of work, because it has
decided the result cannot change the game. When that decision is right it saves
a prompt or a search. When it is wrong, nothing fails: the engine asks one
question fewer or computes one thing fewer, and the game is wrong only on a
board nobody foresaw. This page lists every such place, what each one claims,
the condition the claim depends on (its **premise**), and what checks that
condition.

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
triggered ability to put on the stack, one scried card to order. And an
instruction with nothing to choose from (CR 101.3): no creature able to
attack, a card with no alternative cost, a scry of an empty library.

## 2. Forced prompts

**No CR rule says a forced choice is skipped.** Comments across the tree cite
CR 102.2 for it, but 102.2 is "In a two-player game, a player's opponent is
the other player"; A4k's census caught this (`backlog.md` §2.22, rule 1). The
claim is the engine's own: a question with one legal answer gets that answer
from any player or bot, so not asking changes nothing.

**The premise every row shares**: the candidates and the count are every
legal answer, exactly the list the prompt would have shown. So a forced
prompt assumes nothing the prompt itself does not: if the list were wrong,
the prompt would have been wrong too. What a row adds is the test that decides
"only one answer", and whether that test is the function the decision counter
uses (`only_pick` `ui/ask.rs:190`, `only_number` `ui/ask.rs:254`,
`only_order` `ui/ask.rs:425`, `forced_allocation` `ui/ask.rs:72`) or a hand
copy of it.

| Question | Where it is skipped | One answer when | The test |
|---|---|---|---|
| a target or a "choose", CR 601.2c, 608.2d | `ask_select_recipients` `ui/ask.rs:686` | the count is fixed and takes none of the legal candidates, or all of them | a copy of `only_pick`'s |
| which cards to discard, CR 701.9b | `ask_discard` `ui/ask.rs:946` | the count takes the whole hand | a copy of `only_pick`'s |
| the value of X, CR 601.2b, 107.3a | `ask_choose_x_value` `ui/ask.rs:603` | one value is legal (`legal_x_values`) | `has_one_value`, which `only_number` restates |
| a split: combat damage among blockers (CR 510.1c), trample (702.19b), generic mana (601.2h), "the next N damage" (615.7) | `forced_allocation` `ui/ask.rs:72`, called by the four asks that split | nothing is left once each part has its minimum; or nothing is spare once each has its maximum; or only one part can take the rest, and it can hold it | the counter's own function; its doc argues the three shapes, and `ui/ask.rs`'s unit tests pin them. A split it answers never reaches `validate_allocation` `ui/ask.rs:260`, so it answers `None` to anything malformed |
| which permanents to sacrifice for a cost, CR 601.2h, 701.21a | `plan_payment` `engine/costs.rs:305` | there are exactly as many candidates as the cost takes | the caller's; `ask_choose_sacrifice_for_cost` asserts more |
| what a copy effect copies, CR 608.2d, 614.12a | `apply_copy` `engine/resolve.rs:1900`, `entry_copy_donor` `engine/replacement/pipeline.rs:3017` | one candidate, and the copy is not a "may" | the caller's; `ask_choose_copy_source` asserts two or more |
| a source of damage, CR 609.7a | `fill_chosen_damage_source` `engine/resolve.rs:2133` | one candidate | the caller's; `ask_choose_damage_source` asserts two or more |
| "an opponent of your choice" as a permanent enters, CR 614.12a | `entering_controller` `engine/replacement/pipeline.rs:2966` | one opponent | the caller's; `ask_choose_entering_controller` asserts two or more |
| what a creature put onto the battlefield attacking attacks, CR 508.4 | `decide_entry_attacks` `engine/actions.rs:2269` | one candidate | the caller's; `ask_choose_entering_defender` asserts two or more |
| whether to pass priority, CR 117.3d | `run_priority_round` `engine/priority.rs:38` | passing is the only legal action, and the seat does not stop at every priority point | the caller's; the decision log writes the pass as a forced line |

An ask's assertion catches a caller that would prompt with one candidate. The
premise itself rests on the list of legal candidates, which the prompt would
have used anyway.

## 3. Elisions

The risky kind: there really are several answers, and the engine claims they
all leave the same game. Each claim rests on an argument about what the
effects involved can read and write.

| Question | Where it is skipped | The premise | What checks it |
|---|---|---|---|
| CR 616.1: which of several replacement effects applies first, among those the player must choose from at that step of 616.1a–e | `ordering_cannot_change_outcome` `engine/replacement/pipeline.rs:1083` | applying one never stops another from applying, never changes what another writes, is never overwritten by another, and never makes an effect apply that would multiply another's write | `replacement-architecture.md` §3.5's feeds table: its inputs are matched exhaustively (`EntryWrites` `engine/replacement/pipeline.rs:1538`), so a new kind of write does not compile until it is classified. `check_order_invariance` `engine/replacement/pipeline.rs:1748` re-checks the first property after every skipped prompt, in debug builds only, so a release game pays nothing. Two tests reach every order for the fourth property |
| CR 603.3b: the order of one player's triggered abilities that triggered together | `trigger_order_cannot_change_outcome` `engine/triggers/placement.rs:110` | they are the same ability, in the first of 603.3b's two tiers, with no target and no mode, and they agree on every fact the ability reads off its trigger: which object, which player, how much, its power, the source, X, the ability's own count (`entries_agree_on` `engine/triggers/placement.rs:122`) | `trigger_binding_reads` `engine/triggers/binding_reads.rs:108` lists what an ability reads, and it is an exhaustive match: a new kind of effect does not compile until it says what it reads. TR-2b's and TR-5a's tests pin both answers. Nothing checks, during a game, that a resolving ability reads only what the list says |
| CR 603.7b: which of several events that happened at once triggers a delayed triggered ability | `choose_delayed_cause` `engine/triggers/delayed.rs:526` | the events agree on every fact the ability reads, as in the row above | the row above's function and list; TR-3b's two tokens dying at once |
| CR 601.2f: the order of two or more cost reductions. **Owed, not taken** | asked in `modify` `engine/cost_determination/total.rs:95` | every order gives the same total (`cost-architecture.md` §3.4's theorem), until a reduction can be a hybrid symbol or have a floor | item 165, slotted at CP-1; `AutoPayer` answers it until then |

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
(`engine-map.md` §2.3). The premise is that an object can come to have such an ability only by
one of those three routes. A leg that says "yes" too often only costs a wasted
search. A missing leg is the danger: if the engine gains a fourth route, and
text-changing effects (layer 3) are the known one, the gate answers "nothing
here" on a board where something is, and the card silently does nothing. The
restriction, cost and trigger readers have gates of the same shape.

| What is skipped | Where | The claim, and what it depends on | What checks it |
|---|---|---|---|
| looking at every permanent for a replacement effect | `gather` `engine/replacement/gather.rs:202`; the gate at `engine/replacement/gather.rs:272` | the three legs; counters that make replacement effects (shield, stun, finality) are found directly (`any_replacement_counter`), and the commander rule (CR 903.9b) and the entering object are asked before the gate | the argument beside it and each leg's tests. Nothing compares the gate's answer with the full search |
| looking at every permanent for a "can't" | `prohibition` `engine/restriction/predicate.rs:96`; the gate at `engine/restriction/predicate.rs:114` | the three legs; keywords that forbid things are asked before the gate | the same: nothing compares |
| looking at every permanent for a cost increase or reduction | `cost_modifications_for` `engine/cost_determination/gather.rs:56` | the three legs, and the spell's own printed card (`prints_cost_ability`) | the same: nothing compares |
| looking at every object for a triggered ability an event triggers | `detect` `engine/triggers/dispatch.rs:735`; `check_state_triggers` `engine/triggers/state.rs:53`; `count_additional_triggers` `engine/triggers/dispatch.rs:1530` | the three legs, the lists of objects outside the battlefield with triggered abilities, the objects that just arrived, and a snapshot of every ability list taken only when an event moves an object that had one (`LookBackSnapshot`) | TR-1b's audit, `audit_dispatch` `engine/triggers/audit.rs:118`: under `fuzz_games --audit`, and in every A/B's counter runs, each dispatch is answered again with every shortcut off and the two compared (`triggers-architecture.md` §4.10) |
| computing a permanent's controller through the layers, and the sweep that announces control changes (TR-4d) | `get_effective_controller` `oracle/characteristics.rs:167`; `settle_announced_controllers` `engine/triggers/control.rs:118`; `keep_frames_before_control_change` `engine/triggers/control.rs:57` | with no continuous effect changing control, every permanent is controlled by whoever it entered under; and if there was none at the last sweep either, nothing has changed hands. Both rest on layer 2 being the only thing that sets a controller (CR 613.1b) | the type: a copy effect's values (`CopiableValues`) have no controller field, so a copy cannot set one; TR-4d's tests. Nothing compares the sweep with a full walk |
| recomputing an object's characteristics | `compute_characteristics` `engine/layers/compute.rs:103`; `LayerMemo` `state/layer_memo.rs:45` | a result stored at the current epoch is still right: every write to anything the layers read moves the epoch, and a few inputs (`GatedWalkInput` `state/layer_memo.rs:89`) move it only when a stored result read them | `audit_memo_hit` `engine/layers/compute.rs:210` recomputes every stored result it serves, in debug builds |
| checking each "target" clause when deciding whether a spell can be cast (A4q) | `every_instance_has_a_choice` `oracle/mana_helpers.rs:415` | two identical clauses that do not depend on earlier targets have the same answer | A4q's measurement, every gameplay counter unchanged. Nothing checks it |
| walking every object for commanders that changed zones (CR 704.6d) | `arrivals_since_last_sba_check`, kept as objects arrive | every arrival the state-based check needs is noted | `moved_since` `engine/sba.rs:142` walks every object in debug builds and compares |
| the four feedback loops' skipped work | `references/feedback-loops.md` §2, §3, §4, §5 | loop 1's is the CR 616.1 row above; loop 2's pre-check covers every filter, condition and amount; loop 3 skips nothing; loop 4's is the dispatcher row | there |

## 5. What checks what

- **The compiler**: the feeds table's inputs; `trigger_binding_reads`.
- **A check at runtime**: the memo's audit and `check_order_invariance` in
  debug builds; TR-1b's audit under `--audit`; the commanders' walk in debug
  builds.
- **Only an argument and tests**: the replacement, restriction and cost
  gates; the controller shortcut and the control sweep; A4q's fold; and, for
  the two trigger elisions, the premise that a resolving ability reads only
  what `trigger_binding_reads` says it does.
