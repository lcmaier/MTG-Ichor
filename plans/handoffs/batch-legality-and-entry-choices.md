# Handoff — `batch-legality-and-entry-choices`, the design note

**Opened 2026-10-25**, A6e's cleanup PR 5b (`item-6-audit.md`'s cleanup
plan, row 5b, and its entry under "What each PR takes"). This file is the
design note, reviewed before the build. It holds only what #293 left open:
the decisions #293 settled for this PR are in
`plans/handoffs/batches-durations-and-control.md` (its decisions 2 and 3,
and item 330's answer), and the landing deletes both files. Every size below
was read against `384f2243` (#293's merge).

## Where the work sits

All of it is one batch's path: a proposal, the CR 616.1 loop that decides
every member against one board (phase 1), the performance (phase 2), then the
triggers. The words "batch", "phase 1" and "phase 2" are the #293 note's.

- **Phase 1**: the entry replacements' choices. Devour (items 61, 62, 63), a
  shockland's 2 life (item 134), tribute's opponent and its "may" (item 332).
- **Between the phases**: what phase 1 decided to move or pay, performed as
  one batch of its own (#293's decisions 2 and 3).
- **Phase 2**: item 26 skips a member an earlier member already moved.
- **Triggers**: item 330's filter reads the object as its event found it.

No card is registered. Each item's card is a fixture in its test file
(Thromok, Dragon Broodmother's token, Corpsejack Menace, a shockland, a
tribute creature, Cloudsculpt Armorer), which is how both pools stay
identical.

## The re-count

Code and tests apart, since the plan carries code at 1.5–2×.

| Item | What it builds | Code | Tests |
|---|---|---:|---:|
| 26 | A per-variant fact on `GameAction` (27 arms, no wildcard), the check before each perform, and its debug assertion | 90–120 | 70–100 |
| 61 | `execute_batch_inner` split into deciding and performing; the held decided batch; devour decides and counts its moves; the entering batch performs them before its own phase 2; the one-batch check | 150–210 | 120–160 |
| 62 | `apply_auxiliary_move`'s unused entering-controller parameter goes; `grants_devour` becomes a real Layer 6 grant, its chooser asserted | 0–15 | 50–70 |
| 63 | `PerChosen::Fixed(n)` and `PerChosen::Count`, at four literals and two readers | 20–35 | 30–50 |
| 134 | An entry template's cost, CR 614.12b's running total, the held payment, and its prompt | 110–160 | 120–160 |
| 330 | `named_subject_matches` reads the record's existence as its event found it | 10–20 | 60–90 |
| 332 | Tribute's opponent asked as the creature enters, the counters' "may", and its prompt | 50–80 | 60–90 |
| **Total** | | **430–640** | **510–720** |

That's **940–1,360 sized** and **1,155–2,000 carried**, against the brief's
900–1,270 and 1,100–1,900. The top is higher by the two prompts decision 5
adds (about 40 lines each). The bottom is under the band's floor, which the
owner accepted for the split.

## Decisions

1. **Item 26: what phase 2 compares, and where the per-variant fact
   lives.** Phase 2 has to skip a member whose object an earlier member of
   the same batch already moved (CR 608.2b, 609.3), and still treat every
   other performer error as a bug. Both options put the fact on
   `GameAction`, beside the enum, as the practices page asks of a property
   whose arms grow: `GameAction::objects_acted_on`, one arm per variant,
   naming the objects the event acts on and not the ones it only reads (a
   damage source, a life gain's source). `Attach` names two; the player-only
   variants name none, since a player who is gone is item 292's (PR 6).
   - (a) **The existence each member was decided against.** When phase 1
     ends, before anything performs, each member's objects are recorded by
     existence (CR 400.7). Phase 2 skips a member any of whose objects is no
     longer that existence. An existence changes with every zone change, so
     this asks "still there, and still the same object". It costs up to two
     `ObjectRef`s per member, in one `Vec` per batch beside the decided list.
   - (b) **The zone, re-read live.** The fact also names the zone each object
     is expected in (`ZoneChange`'s `from`, the battlefield for the rest), and
     phase 2 reads the object's zone now. Nothing is recorded, but an object
     that left and came back in the same event passes, though CR 400.7 makes
     it a new object. That can happen now that devour's moves perform between
     the phases.

   **Recommended: (a).** Either costs a lookup or two per member against the
   performer's own work; the budget reads it. A skipped member is left out of
   what the batch reports as performed, so a caller that asks whether its
   proposal happened (an entry, a token's entries, a turn's beginning) reads
   it as not having happened. `Unattach` and
   `BecomeBlocked` already shrug off an object that's gone (CR 701.3d,
   603.2e); under (a) they're skipped before that, with the same answer.

2. **Item 26's fixture: the spell its Slotted line names can't be written
   yet.** "Exile target creature and target creature" as one instruction needs
   a recipient spanning two instances of "target". `EffectRecipient::Group`
   takes only fixed objects and sweeps today, and its doc says a target member
   waits for its first card. That's item 281 ("`Group` takes a target, and each
   member fizzles alone"), slotted to PR 6, `resolution-recipients-and-amounts`.
   Written as two instructions (`Effect::Sequence`), the spell is two events
   and already exiles once, since the second finds no creature on the
   battlefield. Every other batch the engine makes names each object once:
   the state-based check's collapse, combat, the untap step, and devour's CR
   614.13b exclusion.
   - (a) **A batch fixture here, the spell in PR 6.** The test proposes one
     batch of two exiles of one creature through `execute_actions`, shown
     failing today on the second move's `Err`. Item 281's Slotted line gains
     the spell as its fixture. The brief's first bullet ("a spell whose
     instruction names one creature twice exiles it once") lands with PR 6.
   - (b) **Pull item 281 forward**: a target member of `Group`, its place in
     the announcement, and its own CR 608.2b fizzle. Not sized here. It's
     targeting, not this PR's batch.

   **Recommended: (a).**

3. **Item 61: the held decided batch's shape.** #293 settled the timing:
   devour's moves are decided as each one applies, and all of them are
   performed together when phase 1 ends. This is the shape, for review. No
   other shape changes a game, so it isn't put as options.
   - `execute_batch_inner` splits in two. `decide_batch` runs from today's head
     through the sort into batch order and returns a `DecidedBatch`.
     `perform_decided` runs from the lethal-damage read to the end and returns
     what it does today. Its three callers call the two in turn and behave as
     before.
   - `DecidedBatch` holds the members in batch order, each with its own cause
     and applied set, plus the riders their replacements queued.
   - It's held on `GameState.entry_selection`, the per-batch CR 614.13 state
     the entering batch already saves and restores, as one `DecidedBatch` that
     each application extends. It has to be on `GameState`: a second entry's
     devour prompt comes while the moves are held, and a fork at that prompt
     needs them (`codebase-state.md` item 40). Outside an entry it's empty and
     costs a clone nothing.
   - The entering batch takes it when its phase 1 ends and performs it under a
     new batch id. That batch's window is dispatched at its close and its
     riders run after, before the entering batch's phase 2, so the devoured
     creatures die before the entry, as today.
   - Devour counts the held members that are still moves (CR 614.6 drops the
     others). The check, in debug builds: the held batch performed exactly the
     members counted, and wrote one batch id.

4. **Item 134: the cost's type.** #293 settled the timing: the cost is chosen
   in the loop under CR 614.12b and paid with the held moves. A shockland's
   text is "As this land enters, you may pay 2 life. If you don't, it enters
   tapped". That's an entry template whose mods apply unless its controller
   pays: `EnterModsTemplate.unless_paid`, so the shockland is
   `{ status: Tapped, unless_paid: Some(..) }`. CR 614.12b is checked as
   each payment is chosen, against what the event has chosen so far: life at
   least the sum (CR 119.4), and nothing that says the player can't lose life
   (CR 119.8). Choosing one at a time this way reaches every set of payments
   the player could afford together, since any part of an affordable set is
   affordable. The payment is a held `LoseLife` caused by
   `EventCause::Cost { player, paying_for }`, which is what CR 119.4 says
   paying life is.
   - (a) **`Cost`, the cost subsystem's type**, with only `PayLife` built here.
     Any other arm is an `Err` as it applies, the convention `Cost` already
     keeps for its unbuilt arms (`can_pay_costs`).
   - (b) **A narrow `EntryCost` with one arm, `Life(u64)`.** Nothing can be
     written that the pipeline can't apply, which is the growth contract. But
     it's a second type for one fact, paying life, the shape
     `one-fact-one-type` (#291) merged out of the tree.

   **Recommended: (a).**

5. **The prompts items 134 and 332 ask.** Three questions come up as a
   permanent enters. A shockland's controller is asked whether to pay. A
   tribute creature's controller is asked which opponent (CR 614.12a: before
   it enters). That opponent is asked whether to put the counters (Fanatic of
   Xenagos: "an opponent of your choice may put a +1/+1 counter on it").
   Tribute is written as a counters row with `by: Some(PlayerRef::Opponent)`
   and a new `optional` flag, which asks `by` rather than CR 616.1's chooser.
   An unpayable payment isn't offered (CR 118.3), and the mods apply.
   - (a) **Two new arms and a reuse.** `ChoiceKind::PayAsItEnters { object,
     source }` for the payment and `ChoiceKind::PutCountersAsItEnters {
     object, source }` for the counters, each a yes or no. Tribute's opponent
     reuses `ChoiceKind::ChooseOpponent { source }`, its doc widened from a
     resolving instruction's "an opponent" (CR 608.2d) to an entry effect's
     (CR 702.104a). Choosing an opponent is the same question with the same
     options, and `source` tells a client why it's being asked. An AI harness
     can tell paying a cost from giving an opponent a choice by the arm alone.
     The dev GUI gate runs.
   - (b) **One new arm for both yes-or-no questions**, an entry's "may",
     with the opponent reused as in (a). One arm fewer, but a client can tell
     a payment from a gift only by reading the card's text.
   - (c) **A new `ChooseCounterPutter { object }`** for the opponent, beside
     `ChooseEnteringController`, keyed by the entering object like the other
     entry choices. It names the role, at one more arm for the same list of
     players.

   `ApplyOptionalReplacement` can't serve either yes or no: its "no" means
   the replacement didn't apply (CR 614.5's opportunity taken), while a
   shockland's "no" applies it, and tribute applies once an opponent is
   chosen whatever that opponent answers.

   **Recommended: (a).**

### #293's two open questions, riding if you answer

6. **Does "with the first registered card" stay a legal Slotted line?** At
   #293's review you ruled that a filed gap with printed customers takes a
   route row, and commit `7c1aaafd` moved nine items. No open item is homed
   on the slot alone now (the board counts 0); two still name it beside a
   phase (an Aura that gains control, and Dimir Doppelganger's "this ability"
   copy). A gap with no printed customer is owed anyway, since the CR is the
   customer. **Recommended: retire it.** The two lines drop the clause, the
   practices page's "Gaps a phase finds" loses it,
   `check_state_of_play.py` stops accepting it, and item 284 (the check that
   flags such an item when its card is registered) closes as moot. That's
   docs and one script, about 30 lines, a commit on this branch.
7. **Should `Query::Event` take a `Try` prefix like `TryApplyReplacement`
   and `TryApplyRule`?** Those two ask whether *applying* something is
   forbidden. `Event` asks whether the event may happen at all, the "can't"
   that CR 614.17 names by its event ("can't be destroyed").
   `is_prohibited(game, &Query::Event { .. })` already reads as "is this event
   prohibited", and `TryEvent` reads as nothing. **Recommended: leave it.**

## What the build checks

- Item 26: a member that passed the check and performs with an `Err` fails a
  debug assertion, so a performer's error stays a bug.
- Item 61: one entry event's held moves write one batch, and it performs
  exactly what devour counted (debug builds).
- `cargo build --all-targets` with zero warnings, `cargo test`,
  `cargo clippy --all-targets -- -D warnings`, and the `check_*.py` family.
- The close-out's A/B on both pools, predicted identical: no registered card
  reaches a new path, and the split of `execute_batch_inner` lands alone,
  before anything uses it.
- The CPU budget on item 26's check (per batch member), the floors, and
  `specdb owed --phase "Phase 7"` still clean.
- The audited smoke, since phase 2 changes.
- The dev GUI gate, for decision 5's new arms.

Each fix is shown failing first against the tree without it.
