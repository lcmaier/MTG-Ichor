# Handoff — `batches-durations-and-control`, the design note

**Opened 2026-10-25**, A6e's cleanup PR 5 (`item-6-audit.md`'s cleanup plan,
row 5, and its entry under "What each PR takes"). This file is the design
note, reviewed before the build. **The PR is two since #293's review**
(decision 1): `durations-and-control` builds first on this branch, and
`batch-legality-and-entry-choices` follows on its own; the second deletes
this file as it lands. Every size below was read against the tree at
`5b578606` (#292's merge).

## Where the work sits

Most of the PR is in the batch: `execute_batch_inner`'s two phases, deciding
every member against one board and then performing them (CR 704.3). Item 26
guards phase 2. Items 61, 62 and 134 are entry replacements whose choices are
made in phase 1 and whose moves or payments happen around it. Item 17 ends a
resolution's "for as long as" rows as their object leaves or changes hands,
which reads the control sweep (`engine/triggers/control.rs`, CR 603.10d);
items 300, 324 and 325 tidy that sweep and what it remembers, and "Before
Layers" 13 gives `Primitive::GainControl` a recipient. Item 20 is the cleanup
step's damage wipe (CR 514.2), and item 148 a primitive that moves a card into
its owner's graveyard.

**Words this note uses.**

- A **batch** is one event made of several things that happen at once: the
  creatures one state-based check destroys (CR 704.3), or the two tokens one
  effect creates. Cards drawn one at a time are a batch each (CR 121.2).
- **Phase 1 (deciding)** works out, for each thing in the batch and against
  the board as it was before any of it, which replacement effects apply and
  in what order (CR 616.1). An "as this enters" choice is made here (CR
  614.12a): which creatures devour sacrifices, whether a shockland's
  controller pays 2 life. Nothing has happened yet.
- **Phase 2 (performing)** does everything phase 1 decided, in the batch's
  order. Then the batch's triggers are checked, against the whole event.
- **The narrow form** (decision 7) is item 300's own term: announce a waiting
  change of control only before the next control-changing instruction. The
  **wide form** announces before every continuous effect a resolution
  writes, of any layer.

## The re-count

Each item's **Sized:** line, read against the tree. Code and tests are
counted apart because the plan carries code at 1.5–2×.

| Item | What it builds | Code | Tests | Against its Sized line |
|---|---|---:|---:|---|
| 17 | The replacement and restriction registries swept as their row's object leaves; a stale object registers nothing (CR 611.2b); the control leg; the check that no such row outlives its object | 110–150 | 180–240 | Larger: the line predates TR-3b, which did the continuous registry, and its three fixtures plus one row in each other registry are most of it |
| 20 | CR 514.2's wipe asks a restriction before it removes damage | 50–70 | 60–90 | Larger: no `Restriction` arm can say it (decision 6) |
| 26 | Phase 2 skips a member that's no longer legal (CR 608.2b), with a debug assertion | 100–130 | 80–120 | Larger: `GameAction` has 27 variants, not 10 |
| 61 | One entry event's auxiliary moves as one batch, counted by what was performed | 150–220 | 120–160 | Larger, for decision 2 |
| 62 | A granted devour chosen by the entering creature's controller | 0–20 | 50–70 | As sized: the gather gives a granted ability's instance the entering controller; the fixture turns into a real Layer 6 grant |
| 134 | An entry choice's cost, and CR 614.12b's check over a batch | 110–150 | 120–160 | Larger: the may-pay prompt and the deferred payment (decision 3) |
| "Before Layers" 13 | `GainControl` names its new controller; "an opponent" chosen at resolution | 60–90 | 70–100 | As sized |
| 300 | Two control changes from one resolution announced apart | 15–25 | 45–60 | As sized |
| 324 | One record of what the last control sweep announced | 30–50 | 10–20 | As sized |
| 325 | `controller_since_turn` goes; the frame seeds from the entry turn | 20–40 | 20–30 | As sized |
| **Total** | | **645–945** | **755–1,050** | **1,400–2,000** sized |

Carried at the plan's 1.5–2× on code, that's **1,720–2,940**, against the plan's
1,750–2,100. The bottom of the range is the plan's figure; the top passes
2,500. Two items aren't in the total:

- **Item 148 is re-slotted** (decision 4). "Put target face-up exiled card
  into its owner's graveyard" needs a target in exile, and `SelectionFilter`
  can't select a card outside the battlefield and the stack. That surface is
  about 260–400 lines more, in 16 files.
- **Item 330** (how a trigger's filter reads an object its own event moved)
  joined after #290 and waits on two judges' questions. If they answer before
  the build, it costs 10–20 lines and 60–100 of fixtures (its **Sized:**
  line). If they don't, it stays filed.

## Decisions

1. **The size.** **Settled: two PRs, split by concept** (the owner, at
   #293's review, 2026-10-25), one past the cleanup's planned four.
   `durations-and-control` takes items 17, 20, 300, 324, 325 and "Before
   Layers" 13 with decisions 5, 6 and 7: ~670–965 sized, ~810–1,390 carried.
   `batch-legality-and-entry-choices` takes items 26, 61, 62, 134 and 330
   with decisions 2 and 3: ~730–1,030 sized, ~910–1,550 carried. Neither
   needs the other. Both sit at or under the band's floor; the owner chose
   a shorter review over the floor. The options as they were put:
   Carried, the re-count's top passes 2,500, so by the brief I
   stop and ask before building. Sized, it's in the band.
   - (a) Build all ten items as one PR, as planned, and flag a crossing at the
     commit that makes it.
   - (b) Move item 134 to `layers-combat-and-counters` (PR 7, carried
     1,650–2,000): that leaves 1,170–1,690 sized and 1,435–2,480 carried here.
     Item 134 has no registered customer and reuses decision 2's mechanism,
     so it can follow it.

   **Recommended: (a).** The top of the carried range takes every item at its
   high end, and decision 2's option D is most of the risk. If the build
   crosses 2,500 at item 61, item 134 moves then, as (b).

2. **Item 61: when devour's sacrifices are performed.** **Settled: D** (the
   owner, at #293's review, 2026-10-25). Today each
   application performs its moves through its own batch at once, so two
   devour creatures entering together sacrifice at two moments, and "whenever
   one or more creatures die" triggers twice (the Thunder-Thrash Elder
   ruling: "All creatures devoured this way are sacrificed at the same
   time"). The difficulty the plan didn't price: within one entry's CR 616.1
   loop a later replacement can read devour's counters (Corpsejack Menace
   doubles them, Hardened Scales applies only if there are some), so the
   count has to be known inside the loop, while the moves can't perform until
   every entry has chosen.
   - (A) Perform every move when phase 1 ends, as one batch, and merge each
     entry's counters then. Later replacements in the loop see no counters,
     so devour with Corpsejack enters undoubled. Least code (~100), wrong
     with any counter replacement.
   - (B) Pause a subject group's loop after an auxiliary move applies, run
     the event's moves as one batch when every group has paused or finished,
     then resume the paused loops. CR 614.5's applied set is the loop's state,
     so resuming is calling the loop again. Exact for one devour per entry.
     One entry with two (CR 614.13b's own example, Thunder-Thrash Elder under
     Jund) pauses twice and sacrifices at two moments.
   - (D) **Decide each move as its replacement applies, and perform them all
     when phase 1 ends.** When devour applies, its moves go through phase 1
     at once, which tells the count exactly: a move a replacement drops is
     known then (CR 614.6), so the count is what will be performed. Every
     decided move is then performed as one batch, with a new batch id
     dispatched at its close as today, ahead of the entries. Nothing performs
     in between, so every decision is made against the event's one board.
     It needs `execute_batch_inner` split so a decided batch can be held and
     performed later (~60–100 lines of the 150–220). The item 61 check, that
     one entry event's moves write one batch, holds by construction, and
     `test_a_prevented_sacrifice_is_not_counted` keeps its answer.

   **Recommended: D.** It's exact for every board above, and the cost is one
   split of the batch body, not a second loop.

3. **Item 134: when an entry choice's cost is paid.** **Settled: (b)** (the
   owner, at #293's review, 2026-10-25). "As this land enters,
   you may pay 2 life. If you don't, it enters tapped." CR 614.12b ("may not
   make choices ... that would cause the combined costs of those effects to
   not be payable") only means something if the costs are paid after the
   choices. CR 614.13c's Ashiok example has a replacement apply to the
   payment as the choice is made.
   - (a) Pay at application, in the loop. Paying in sequence makes 614.12b
     hold by itself for life, and pays mid-decision, before the other entries
     have chosen.
   - (b) **Choose in the loop, deciding the payment there, and pay with
     decision 2's decided moves when phase 1 ends.** At each choice the
     player may pay only if the event's costs chosen so far plus this one are
     payable (CR 119.4 for life). The payment is a decided `LoseLife` caused
     by the cost, so Ashiok's replacement applies as it's chosen.

   **Recommended: (b),** with D in decision 2: one mechanism for both of CR
   614.12's side effects. With A or B there, (a).

4. **Item 148: the target in exile.** **Settled: (a)**, re-slotted to
   `ironworks-loop` (the owner, at #293's review, 2026-10-25), and its
   **Slotted:** line says so. "Gaps a phase finds" files a gap outside
   the PR's subsystem with no registered customer, and targeting is neither
   this PR's subsystem nor reached by any registered card: no card targets a
   card in a graveyard or exile. `ironworks-loop` already builds graveyard
   targets (Scrap Trawler, Myr Retriever), and exile is the same surface, a
   card in a public zone.
   - (a) **Re-slot item 148 to `ironworks-loop`**, beside the graveyard
     targets it builds, with the Colossus test (a Colossus in exile is
     shuffled into its owner's library instead) as that row's fixture.
   - (b) Build the target surface here: a `SelectionFilter` arm for a card in
     a public zone, its enumeration, CR 608.2b's zone re-check, and
     castability's legal-target check (+260–400 lines).

   **Recommended: (a).**

5. **"Before Layers" 13: where "an opponent" is chosen.** **Settled: (a)**
   (the owner, at #293's review, 2026-10-25).
   `resolve_player_ref` names the one opponent there is and returns an `Err`
   above two players. CR 608.2d makes a choice the instruction offers a
   choice at resolution, and the Akroan Horse ruling says so for this one.
   - (a) **Every instruction asks through one function.** The three callers
     that apply an instruction (`GainControl`'s new recipient,
     `GetCounters`' `by`, `Effect::Optional`'s chooser) go through one
     function that asks the `DecisionProvider` among the opponents still in
     the game, only when there are two or more. The two callers that only
     read (an amount's "an opponent's hand", the enters-attacking filter's
     player test) are `&self` and can't ask, so they keep the `Err` above two
     players: the choice would have to be made earlier in their instruction,
     which is their first card's work. It adds a `ChoiceKind` arm, which
     brings in the dev GUI gate.
   - (b) Only `GainControl`'s arm asks, and the other two instruction sites
     keep their `Err`.

   **Recommended: (a).** CR 608.2d covers any instruction, and asking per
   site would be one choice written three times. The row stores
   `SetController(PlayerRef::Player(chosen))`, settled as the effect begins,
   as the item's own table says every non-`You` identity is.

6. **Item 20: the restriction CR 514.2's wipe asks.** **Settled:
   `Restriction::RuleDoesNotApply { rule, affected }`** (the owner, at #293's
   review, 2026-10-25), an axis for every rule whose own action a card can
   switch off. `rule` is `StateBased(StateBasedAction)`, reusing the
   existing one-variant-per-CR-704.5-subrule enum, or `CleanupDamageRemoval`
   (CR 514.2); `ManaEmptying` (CR 500.4) joins with its first card. Printed
   families: 7 cards keep damage through cleanup, 31 keep mana between steps,
   11 switch off the legend rule (CR 704.5j) and 8 say "you don't lose the
   game for having 0 or less life" (704.5a; Scryfall, 2026-10-25). The wipe
   asks it per permanent, and the state-based check asks it once per subrule
   candidate, gated on a registry flag so a board with no such effect pays
   nothing: every 704.5 subrule is switchable at once, printed or custom,
   with fixtures for 704.5a and 704.5j beside the wipe's (+40–60 lines and
   their fixtures). 704.5a needs this axis and not the event axis's "can't
   lose": the check's one `PlayerLoses` names its first reason, so vetoing
   it would also save a player with ten poison counters. Not a CR 609.4 "as
   though" effect, which feeds a rule a pretended fact
   (`cant-effects-architecture.md#37-the-mirror-this-document-must-not-block--cr-6094s-as-though`,
   `as-though-effects`); this removes the rule's action. The options as they
   were put: "Damage isn't removed
   from creatures you control during cleanup steps" forbids part of a
   turn-based action, which proposes no `GameAction`. `Restriction`'s axes
   are an event, a choice and applying a replacement, and none of them is
   asked by a rule's own action.
   - (a) **A fourth axis, one arm**: `Restriction::RemoveDamage { affected }`,
     asked per permanent by the wipe. That's one arm per axis with a consumer,
     as the type's growth contract says, and the 248 "doesn't untap during"
     cards are an `Event` on the untap step's `Untap`, so this axis has no
     second customer to wait for.
   - (b) Propose the wipe as a `GameAction`, so the `Event` axis watches it.
     That makes it replaceable, which CR 614 doesn't (the wipe is tagged
     `// NOT REPLACEABLE:`), and it adds an `EventPattern` arm.

   **Recommended: (a).**

7. **Item 300: where the narrow announcement goes.** **Settled: the wide
   form** (the owner, at #293's review, 2026-10-25): every continuous effect
   a resolution writes settles the layers' consequences first, so each
   change of control, and each combatant a type change removes (CR 506.4),
   is an event at the instant it happens (CR 603.2). No printed card was
   found that needs it over the narrow form; a custom card can, and the
   engine keeps the general form. Its cost: the resolver's row writers take
   an `ActionContext`, and both pools move where Ray of Command resolves,
   its steal announced before its delayed trigger is created, attributed
   game by game at the close-out. The options as they were put: Two `GainControl`
   instructions in one resolution with no batch between them are announced
   as one change today, both looking back to the board before the first.
   `GainControl`'s arm is the resolver's one Layer 2 writer.
   - (a) **`GainControl`'s arm settles the layers' consequences before it
     writes its row** (`settle_layer_consequences`, the batch head's own
     call). It's gated on the layer epoch, so with nothing since the last
     batch it costs one comparison. Ray of Command's untap batch settles at
     its close, so its steal finds nothing waiting and its order is kept.
   - (b) `add_resolution_row` announces before every row: this covers a Layer
     6 row that ends a steal too, but it needs a provider on every row writer
     and moves Ray of Command's announcement ahead of its delayed trigger, a
     re-record of a pooled stream.

   **Recommended: (a),** the narrow form the brief names. The Layer 6 case
   stays item 300's remainder, filed.

## What the build checks

Each item's own check, from its entry: item 17's "no row whose duration ends
with its source names an object that has left the battlefield" after each
performed batch; item 26's "a member that passed performs without `Err`";
item 61's "one entry event's auxiliary moves write one batch". All are in
debug builds. Each PR's close-out runs the A/B on both pools, the floors
and `specdb owed --phase "Phase 7"`.

- `durations-and-control`: both pools predicted to differ only where Ray of
  Command resolves (decision 7's wide form), attributed game by game; the
  CPU budget on item 17's sweep (per departure) and the settlement per row
  a resolution writes. The dev GUI gate runs: decision 5 adds a
  `ChoiceKind` arm.
- `batch-legality-and-entry-choices`: both pools predicted identical, since
  no registered card reaches a new path; the CPU budget on item 26's check
  (per batch member).
