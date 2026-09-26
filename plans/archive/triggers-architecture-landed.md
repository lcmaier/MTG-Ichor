# triggers-architecture.md — landed phases, evicted

**A record of finished work, not a plan.** Every section here sat under a ✅
heading in `plans/triggers-architecture.md` and was moved out when its phase
shipped, leaving the heading, a stub and a pointer in the live doc. Nothing
here is owed and nothing here should be acted on; what each section is *for*
is the reasoning — the design as sized, what the building changed, the
measurement. `check_state_of_play.py` reads the ✅ headings that stay, and
fails when a landed section keeps more than 40 lines in the live doc
(`engineering-practices.md` §4). Later phases are appended by the PR that
lands them.

#### TR-1 — the spine: dispatch, the queue, placement, the stack object (~2,300) — ✅ landed 2026-09-19

*Evicted 2026-09-19 from `plans/triggers-architecture.md` §12, where the heading and a stub remain.*

### The plan as sized (2026-09-18)

| Piece | Measured | ~additions |
|---|---|---|
| `TriggerDef`, `TriggerCondition`, `TriggerEvent` with the arms its cards need (`ZoneChange` for the battlefield classes, `EntersBattlefield`, `StepBegins`/`PhaseBegins`/`TurnBegins`, `ManaAdded`, `DamageDealt`, `BecomesTapped`/`Untapped`, `AbilityTriggers`), `Occurrence`, `TriggerBinding` with `EventSeq` on `EventLog` and the four arm projections, `PendingTrigger`, `TriggerOrigin::Object`, `Tier`, the three bound-fact leaves, `ObjectFilter::NotSource`, `Effect::Triggered`, `AbilityIdentity`'s two fields | 13 `(source, ability)` sites; `Effect::instances` +1 arm; 5 `EffectRecipient` exhaustive matches, 2 `AmountExpr` | ~420 |
| `AbilityTriggered`; item 10's three fields; item 18's deletion; `PermanentState.cast` (who, from where) | 34 emit sites read, 3 performers; literal sites `StepBegin {` 5/12, `DamageDealt {` 10/7, `LifeChanged {` 10/9; `format_event` −3 +1 | ~120 |
| the dispatcher: the window at both doors, the candidate set (legs 1, 2, 4), the gate (`trigger_sources`, the summary's two zone fields, `puts_a_triggered_ability`), the visibility predicate, look-back off `from`/`to`, `OncePerEvent`, the entry join, 605.1b's immediate resolution, the two trace records | `gather.rs`' shape, 1,222 lines, as the template; `register_static_effects` +2 doors | ~520 |
| placement: the state-check stub (a no-op until TR-6), 800.4d, tiers, APNAP, `OrderTriggers` and its elision, `announce_targets`, the stack object, `StackEntry.trigger` | 13 + 5 `StackEntry` literals; `put_on_stack.rs:437`'s twin | ~260 |
| resolution: 608.2a's check, `Effect::Conditional`, the binding on `ResolutionContext`, `LastKnownInformation`'s readers | `resolve.rs:220`; `ResolutionContext`'s 48-site `Option` pair (item 137) untouched | ~180 |
| cards: **Soul Warden** (603.6a's "another", the batch), **Blood Artist** (dies incl. itself — 603.10a look-back; a target chosen at placement), **Verdant Force** (each upkeep, a token), **Wild Growth** (605.1b, an Aura's `Host`), **Felidar Sovereign** (603.4 at both instants, `WinGame`); Soul Warden, Blood Artist and Wild Growth into `PERFORMANCE_POOL` (91 → 94: the matcher, the placement prompt, the stackless path) | rulings read: Soul Warden (1), Blood Artist (1), Verdant Force (2), Wild Growth (1), Felidar (via 603.4's atoms) | ~300 |
| tests: §13's TR-1 atoms, Eon Hub's two (item 121), the 800.4d four-player fixture, the Humility-beside-a-creature entry, Guile's two boards with Yixlid Jailer, the elision's expiry conditions, a fixture "whenever damage is dealt to you" that closes four partials | `phase_tr1_integration_test.rs` | ~700 |
| docs: this section's stub, `codebase-state.md` items 1, 3, 7, 9, 10, 18 closed, the ledger, `fuzz-record.md` | | ~250 |

**Trace page at close** (`engineering-practices.md` §7 names item 6): the
dispatch changes *how* a read is answered — `tr-1-a-trigger-is-matched-at-
the-close.html`, walking Soul Warden beside Humility entering together,
Blood Artist in a wipe, and Wild Growth inside the mana window.

### As landed (2026-09-19)

**What shipped, where.** `types/triggers.rs` is the type surface — `TriggerDef`,
`TriggerCondition`, `TriggerEvent` with fourteen arms and the four projections
as exhaustive matches, `Occurrence`, `Subject`, `DamageRecipient`,
`TriggerBinding`, `PendingTrigger`, `TriggerOrigin::Object`, `Tier`,
`TriggerLimit` (declared, TR-2's), `ObjectRef`, `TriggerSeq`, `ArmIndex`.
`engine/triggers/` is the two instants: `dispatch.rs` (the window at both
doors, the gate, the candidate legs, the matcher, CR 605.4a's immediate
resolution, the `trigger` trace record, `AbilityTriggered`), `placement.rs`
(CR 800.4d's refusal, the tiers, APNAP over `apnap_index`, the ordering
prompt and item 163's elision, CR 603.3d through `announce_targets`, the
stack object, the `pending` record) and `binding.rs` (the bound facts read
back through the arm's projections). `EventSeq` and `EventLog::record` are
the binding's handle; `emit_event_unstamped` is the second door beside the
one. `zone_function::functioning_zones` derives CR 113.6k from the
condition; `register_static_effects` and `cleanup_zone_state` keep
`trigger_sources` and `zone_trigger_sources`; `RegistryScopeSummary` gained
`granted_trigger_zones` and `copied_trigger_zones`. The five cards are
`cards/phase_tr1_cards.rs`; the tests `tests/phase_tr1_integration_test.rs`,
fifty of them.

**What the sizing did not predict, and the doc's own words that moved.**

1. **`ObjectFilter::NotSource` was not written.** `ObjectFilter::EachOther`
   already states the predicate — "other than the filter's source", answered
   from the object id — and a second leaf would have given one quality two
   spellings, the rule `ObjectFilter::Token`'s doc already applies. "Another"
   is `And(filter, EachOther)`; `phase_tr1_cards::another` spells it once.
   *Repointed: theme A renamed the leaf `NotSource`; theme B moved the
   helper to `cards::authoring::another` and deleted a second copy that
   had been sitting unread in `dispatch.rs`.*
2. **Two arms shipped early and narrow, because §13's TR-1 row owed their
   atoms.** `Attacks { attacker, occurrence }` for ATOM-508.1m-001 (TR-5
   widens it to the five shapes with item 11's defender) and `GainsLife {
   player, occurrence }` for ATOM-119.9-001/-002 (TR-2's `LosesLife` is the
   other half of the split). §3.3's table placed both later; §13's row placed
   their atoms here, and the row won because an atom needs a test.
3. **Leg 3 is swept, not only registered.** §12 listed legs 1, 2 and 4 with
   `zone_trigger_sources` maintained; legs 3 and 4 turned out to be one map
   — a card put into a graveyard arrives through `arrive_in_zone`, which
   registers an ability that functions there — so the dispatcher sweeps the
   map and Guile's two boards are the test. Nothing is registered that
   nothing reads.
4. **The gate has a sixth probe.** A departed permanent is out of
   `trigger_sources` by the time its window closes (`cleanup_zone_state`
   removed it), so leg 2 cannot be gated by a set: the dispatcher scans each
   departure record's frame for a triggered def. A `Vec` of a few defs per
   battlefield departure, on top of the five probes; the A/B is the reading.
5. **`StackEntry.trigger` holds the binding, and the binding holds the
   def.** §3.13 named `Option<TriggerBinding>`; the resolution's 608.2a check
   needs the def's clause, and §6.3's "the binding is indices and one `Arc`"
   is that `Arc`. `ResolutionContext.trigger` is the same one field, with the
   51 literal sites patched by script.
6. **A look-back arm on a *surviving* permanent reads its post-event list.**
   §4.2's leg 2 looks back for the departed; a Blood Artist that survives a
   wipe that took Humility reads the list it has after the wipe, where
   CR 603.10 wants the one before. Filed as `codebase-state.md` main item
   167; the frames a record carries were never going to answer it.
   *Closed 2026-09-22 by the review's theme E: §4.3's look-back snapshot.*
7. **The dispatch is where `test_support::put_on_battlefield`'s direct
   writes became visible.** The helper wrote `entered_battlefield_turn` and
   `controller_since_turn` after `place_on_battlefield` had walked nothing;
   the entry's dispatch now walks the board, and the memo audit caught the
   write at once. The helper bumps the epoch, as every writer of a walk
   input must.
8. **The `(ObjectId, AbilityId)` pair sites stayed a pair.** The brief read
   §3.6 as "both take the two new fields"; the fourteen tuple sites are the
   mana window's keys and `enumerate_activatable_mana_abilities`' dedupe, and
   §3.6 says why a mana ability's two instances need no telling apart. Giving
   them the instance would have re-offered the second Citanul Hierophants
   the window used to list and moved the random agent's stream, which §11's
   `IDENTICAL` prediction for the engine arm forbids. *Amended 2026-09-22
   (theme E):* the ids in those pairs now carry a granted instance's grant,
   and the dedupe keys on `AbilityId::definition()` — the same answer.
9. **Line counts.** `types/triggers.rs` 446, `engine/triggers/` 1,015
   (`dispatch.rs` 754 of them), the cards 278, the tests 1,807 — the sizing's
   700 was well under half: fifty tests, each a board — and the docs above
   the 250 sized. The PR lands above `engineering-practices.md` §4's band,
   and the peel-off point the doc named (605.1b with Wild Growth) is about
   200 of those lines, so it was not taken; the reviewer decides.

**Asked at the review and kept** (the TR-1 review, 2026-09-20 to 09-22 —
theme A's three and theme D's four), so they are not asked again:

- **#19, `retain` for a stack removal:** it keeps the stack's order where
  `swap_remove` would move the last object into the hole, and it matches the
  three sibling removals; at 608.2a the object is the top by construction, so
  a `pop` with a debug assertion would also be exact, and is not worth the
  change.
- **#20, the three `EventLog` wrappers:** `records` is private, so `record` is
  the newtype's only reader (14 callers); `next_seq` (one caller) keeps the
  length-to-sequence conversion inside the log; `emit_unstamped` is the
  second door, named so the exemption is greppable.
- **#32, `TriggerSeq`:** it pairs with `EventSeq`, the other monotone
  counter; the `*Id` family is hashed ids, so `TriggerId` would file it in
  the wrong family by name.

- **#9, `could_add_mana` on `Modal`:** a modal trigger with one mana mode is a
  mana ability by CR 605.1b's "could add mana" — 605.2 keeps the class when
  the state cannot produce it, and the class must be knowable before modes
  are chosen because a mana ability never reaches the stack. Modal triggers
  are `backlog.md` §2.7's, and the mana resolver refuses a `Modal` root with
  an `Err`, so nothing misresolves quietly.
- **#15, the timestamp sort:** no determinism problem — every object in the
  store carries a unique timestamp from one monotonic counter, so the keys
  never tie.
- **#18, CR 603.3d's equivalence:** it holds; F2's rewrite of `place_one`'s
  comment says why.
- **#25, the two source sets:** they mirror the replacement pair for its
  stated reason — leg 1 probes a membership (a kind mask per source since
  theme C) while walking the ordered battlefield, leg 3 iterates a map whose
  value is the functioning-ability list. That value was the one F1 found the
  dispatcher never read; theme C asks CR 113.6 of each def and each
  condition instead.

**Measured.** `plans/fuzz-record.md`, the TR-1 block: the probe found no
dispatch passing the gate on the old pools at two seats or four; the engine
arm read `IDENTICAL` on every counter on both pools at both seat counts; the
shipped arm's `Triggers placed` row and the reachability counts are there.

#### TR-1b — the dispatch audit (~550) — ✅ landed 2026-09-22

*Evicted 2026-09-22 from `plans/triggers-architecture.md` §12, where the heading and a stub remain.*

### The plan as sized (2026-09-22)

The TR-1 review's closing question was how anyone knows detection is
right, and the answer was that nothing checked it (§4.10). Scheduled by
the owner on 2026-09-22 as the next piece of work, before TR-2, so that
TR-2's new trigger conditions arrive under the check.

| Piece | ~additions |
|---|---|
| Item 174: capture each decided departure's frame at the batch's seam, where §4.3's snapshot is taken, and have the move read it; a fixture in both batch orders | ~100 |
| The audit's capture at every seam, the reference matcher, the comparison and its report, the runtime switch, `fuzz_games --audit` and `fuzz_ab.py`'s counter runs | ~250 |
| The dispatcher's counts as diagnostics rows (§4.10 decision 4) | ~30 |
| Tests: the audit on over each candidate set's board (a printed source, a zone-map card, a granted trigger, a departed frame, a survivor's snapshot), and a comparison fed two different answers | ~170 |

**Gate:** the usual, plus an audited sitting on both pools at two seats and
four, and on the forced boards (Humility with Blood Artist at four copies;
Soul Warden, Blood Artist and Wild Growth at eight), reading zero
disagreements. **The audit shown to bite**, recorded rather than committed:
with item 167's snapshot reverted, and separately item 174's fix, an
audited sitting reports the disagreement; F1, which no pooled card
reaches, is shown on Dread's fixture with the audit on. A/B: audited and
unaudited counters `IDENTICAL`, and the unaudited arm against `main` moving
only by the new rows.

### As landed (2026-09-22)

| Piece | sized | landed |
|---|---|---|
| Item 174's seam capture and its two fixtures | ~100 | 195 (95 code, 100 tests) |
| The audit: capture, reference, comparison, switch, `fuzz_games --audit`, `fuzz_ab.py` | ~250 | 555 |
| The dispatcher's rows | ~30 | 67, and a 28-line test |
| Tests of the audit | ~170 | 285 (237 integration, 48 unit) |
| `plans/profile/` (not sized) | — | 76 |

**Where the audit grew.** The sizing counted a reference that reused the
dispatcher's candidate sets with the lists swapped in. The design says the
reference is independent of those sets, so it builds its own: every object
in every zone at every seam and at every dispatch, paired across the two by
`ObjectRef` so an object that changed zones is two existences (CR 400.7),
and each ability asked once across its two lists. The comparison's report
renders the window and both sides by name, and the whole-game invisibility
test plays a game twice. Commit 2 changed the dispatcher's shape only as
far as the audit needed: the gate and the match became `detect`, the queue
`queue_matches`, and the internals the reference shares became `pub(super)`.

**Two findings.** One ability could trigger twice on one record across a
survivor's two lists (CR 603.2c allows once per event): filed as item 175
and then fixed in the review, in the matching loop the dispatcher and the
audit share. A window's departure frames and newcomers answer records from
before and after their own existence: `codebase-state.md` main item 175
(numbered 176 until the first was removed), unreachable on the pools and the
audit's to report first.

### Review round 1 (2026-09-23)

The owner's review found the audit overbuilt: a second matcher
(`reference_ask`, `pair`, its own CR 113.6 filter and fold), 555 lines against
~250, and 11–19× the CPU per game from rebuilding a frame for every object in
every zone twice a batch and discarding it. Rebuilt as the dispatcher with its
shortcuts off, in four commits:

| Commit | What |
|---|---|
| The loop takes its candidates | `find_matches` split into choosing the candidates and `match_candidates`; no behavior change |
| CR 603.2c across two lists | one match per ability identity per record, the first arm; item 175 as filed deleted and the next renumbered; the fixture red before it (4 triggers, 2 right) |
| The shortcuts-off audit | `audit.rs` rebuilt on `match_candidates` over every object that could carry a triggered ability and a `LookBackSnapshot` of them at every batch; `ObjectSnapshot`; the audit's snapshots reach the dispatch as one `Option` |
| Tidy | item 174's capture in one pass; the three rows one `TriggerDispatchWork`; "seam" out of the code |

The audit is 284 lines where it was 468 (28 and 48 of them unit tests), and costs
2.2×, 2.3× and 2.7× the CPU per game at two seats, four and Commander scale
where it cost 11×, 14× and 19×. What it gave up is checking the shared loop's
rules on its own, so F1's demonstration no longer reproduces.

#### TR-2a — the histories, the gates, and each player — ✅ landed 2026-09-24

*Evicted 2026-09-24 from `plans/triggers-architecture.md` §12, where the heading and a stub remain. TR-2 was sized whole; the plan below is that sizing as written, and TR-2b's half of it stays live in §12.*

### The plan as sized (TR-2, as §12 carried it on 2026-09-23)

| Piece | ~additions |
|---|---|
| `TurnSummary`, `PlayerHistory`, `own_turns`, the record-by-record advance, the four `Condition` leaves (three edits each), `FirstTimeEachTurn`, the two gate sets and their two writers, `TriggerLimit` on the def, the 603.7h count off `AbilityResolved` and its condition, the arms `DrawsCard`, `GainsLife`, `LosesLife`, `CastsSpell`, `AbilityResolves`, `ShufflesLibrary`; `Condition::ResolvedThisTurn(n)` (§6.5); the `departed` frames and their reader (§6.1's amendment) | ~550 |
| `Effect::Optional` with `OptionalEffect` and its chooser, `last_cost_answer` for "if you do / don't / can't" (§6.2's amendment; main item 24), 118.12's cost-object check | ~160 |
| `EffectRecipient::EachPlayer(PlayerSet)` in APNAP order (S2, item 122); Alms Collector's rider re-encoded | ~80 |
| cards: **Paladin of Atonement** (last turn, whoever's; `AmountExpr::TriggeringToughness` off the frame), **Vengeful Warchief** ("for the first time each turn"), **Elvish Warmaster** ("one or more", "triggers only once each turn"), **Nykthos Paragon** (603.2h, "may", "that many" on each creature), **Psychosis Crawler** (draws, each opponent, a CDA), **Temple Bell** (each player draws), **Cosi's Trickster** ("whenever an opponent shuffles", "may"; its three rulings); Warchief, Warmaster, Crawler and Trickster pooled (the histories, the gate, `EachPlayer`, the shuffle arm) | ~400 |
| tests, 24: §13's 9 TR-2 atoms (603.1b's fixture in Avatar Aang's shape is one; 118.12-002 partial, on Wicked Guardian's prevented damage); Nykthos Paragon's six rulings as six tests; Cosi's Trickster's three; Elvish Warmaster's once each turn; Ashling the Pilgrim's count as a fixture (the card needs two amount leaves and waits); 121.2c against Alms Collector; the elision's binding-read board (below); an "if you can't" fixture; an enters trigger's "if" and power read after its source is sacrificed in response (§6.1). The pregame-sweep question is measured too, a probe recorded and not a test | ~890 |
| docs, ledger, record | ~260 |

**Carried in from the TR-1 review** (2026-09-22):

- **Before it:** the review's theme E — provenance ids (§3.6's amendment),
  which TR-2's gates key on, and item 167's snapshot (§4.3). Both landed
  2026-09-22. **And item 30's capture (the rulings pass, 2026-09-23), its
  own PR:** mana spent recorded at payment, with the additional and
  alternative costs `StackEntry` already holds, carried to `CastFacts`. A
  fact, so recorded on sight (`engineering-practices.md` §5), and TR-2 would
  pass the band carrying it; its readers come with their cards. **Landed
  2026-09-23**, the mana by type; its source is `roadmap-v2.md` §3a B9.
- **Its first commit:** `StackWatcher` moves to `test_support` — five uses in
  `phase_tr1_integration_test.rs` today, and every trigger phase asks
  "before priority".
- **Beside its binding reader:** item 163's elision compares only the bound
  facts the effect *reads* (walk it for the three `Triggering*` leaves), so
  two landfall triggers from two lands stop prompting when the effect
  ignores the land. Tireless Provisioner is correct today: its
  Food-or-Treasure choice is CR 608.2d's, made as the effect applies (no
  bulleted modes, CR 700.2), so two identical stack objects give the same
  game in either order.

### The split (2026-09-24)

Re-counted before any code, by §12's method: 37 lines a test, and ×1.9 on
the largest code row for the top end.

| | code | top end | tests | code + tests |
|---|---|---|---|---|
| TR-2, 2026-09-23 | 1,190 | 1,685 | ~890 (24) | 2,080–2,575 |
| TR-2, re-counted | 1,635 | 2,050 | ~1,480 (40) | 3,115–3,530 |
| TR-2a, as split | 925 | 1,360 | ~700 (19) | 1,630–2,065 |
| TR-2b, as split | 710 | 910 | ~780 (21) | 1,490–1,690 |

The growth had three sources:
- **The amendments.** The gate keyed by controller, frames from every zone,
  and the frame's cost decisions came to about 125 lines.
- **Gaps the card row hid,** about 230 lines:
  - no recipient for "this creature";
  - one-shots over "each creature you control" doing nothing;
  - Psychosis Crawler's CDA reading hand size.
- **Rulings the row had not counted,** nine tests.

The owner took the split with "each player" moved into TR-2a. That put item
122, which was reachable and wrong, in the first PR. The owner's other
decisions at the sizing:
- **Names.** `EffectRecipient::ThisObject`, not `This`, and a rule for names
  read at the call site (`engineering-practices.md` §2b).
- **`last_cost_answer`** lives in the resolver's walk (§6.2).
- **§3.10's fields** are built now, each named for the side it counts.
- **The lifelink fix** is folded in.
- **Alms Collector's toughness** is fixed as the last card commit, after the
  engine arm.

### As landed (2026-09-24)

It landed at +2,875 additions in code and tests, about 270 of them
one-for-one renames and a moved test helper. That is over the band. "Each player", about
245 of those lines, stays in: the owner's condition was that the coding was
done, and moving it back would not have brought TR-2a under 2,500.

What the sizing did not foresee:
- **Each player admits `DealDamage`.** The 608.2p fixture's magecraft deals
  damage to each opponent.
- **Fifteen `Implicit` sites became `ThisObject`.** `Implicit` had been
  spelling "this object" in all of them.
- **The Layer 4 route.** The dispatch audit panicked at four seats on
  `stress` once the new cards changed the decks: seed 777, game 889, Blood
  Moon beside Ashaya, Soul of the Wild (§4.10).

**Trace page: no**, decided at its close. §7's test is a change to *how* a
read is answered:
- TR-2a's new reads are new fields, read where the dispatcher and the
  resolver already read.
- `ThisObject` is a new recipient, and the flicker test states it
  completely.
- The one changed path is a Layer 4 row counting as an ability-list source.
  That is a set membership: the snapshot it now takes is TR-1's review's,
  read the same way.

### Review round 1 (2026-09-24)

The owner's review of #186, before merge:
- **The history's shape.** A row is `[u64; TurnFact::COUNT]`, keyed by the
  fact; it had been a struct that repeated `TurnFact` field for field, with a
  sorted `Vec` per card type. The history moved from a seat-indexed `Vec` on
  `GameState` onto `PlayerState`.
- **The pregame.** The opening hands are drawn in turn 0, which has no row,
  rather than recorded and then cleared.
- **Names.** `place_in_turn`, `HistoryUpdate`, `met_by`, `you_for`,
  `Cost::TapSelf`, `Cost::UntapSelf`, and the test file's helpers.
- **One each-player recipient.** `EachOf(PlayerGroup)` replaced
  `EachPlayer` and `YouAndThatPlayer`, so a new phrase is a context arm
  rather than a recipient.
- **A new test.** A spell's filtered one-shot reads "you" as the spell
  resolves.

What stayed, and why:
- **Lifelink's accumulator stays on `GameState`.** Item 40 forbids
  outcome-bearing state off it while a nested batch can prompt mid-perform,
  which is also why `prevention_allocations` is there.
- **Durations stay per atom.** Each continuous effect has its own (CR
  611.2a), and a card-authoring helper for "X and Y until end of turn" is
  offered, not built.
- **Conditions naming "your" stay separate variants.** Folding them into one
  variant with a `PlayerSet`, as `HistoryCount` does, is offered.

The performance question from the first sitting was re-taken at seven
rounds: −0.2% CPU per decision at four seats and −2.4% at two, against
+2.9% and +3.8% at three rounds.

### Review round 2 (2026-09-24)

The owner asked why CR 702.15e needed a lifelink mechanism at all. It
doesn't. The rule only fixes the count: one gain event per source per
simultaneous damage event. So the gain is now the batch's own result. As
each member performs, `execute_batch_inner` adds that member's damage to a
per-source sum, then proposes one `GainLife` per source before the batch
closes.

What went:
- the `lifelink_gains` field on `GameState`;
- `note_lifelink`'s write into it;
- the save/restore around every batch.

Round 1's argument, that item 40 forces the sum onto `GameState`, misread
the item. A prompt inside a batch is not a fork point (§15 item 13), and the
batch's own `performed` list already lived on the stack. A nested batch or a
rider performs its own members, so it still gains separately.

The round's head played the same games as round 1's head: counters
`IDENTICAL` on both pools at two seats and four.

#### TR-2b — "may", CR 118.12's answer, and the `departed` frames (2,165–2,355) — ✅ landed 2026-09-26

*Evicted 2026-09-26 from `plans/triggers-architecture.md` §12, where the heading and a stub remain.*

### The design as reviewed (2026-09-26)

Split from TR-2 on 2026-09-24; it builds on TR-2a's gates and histories.
**Designed 2026-09-26**, against the tree after the bounded-state PR, #190 and
LL, and reviewed by the owner before code.

**The pieces, re-counted against the tree** by §12's method: 37 lines a test,
and the largest code row at ×1.9 for the top end.

| Piece | code | tests |
|---|---|---|
| The fold: six `Condition` leaves become one (decision 1) | ~130 | the leaves' unit tests, rewritten |
| `EventKindMask` at a width that follows `EventKind` (decision 5); the arms `DrawsCard` and `ShufflesLibrary`, their projections, matching and authoring words | ~90 | 1: ATOM-121.5-001 made full — a move to the hand without "draw" fires no draw trigger |
| `Effect::Optional` with its chooser, the walk's answer, `Condition::CostAnswer`, `OptionalEffect`, and CR 603.2h's writer reading the answer (decision 2) | ~210 | 3: ATOM-603.5-001; ATOM-118.12-002's partial, a "may" whose damage is prevented still answering `Does`; "if you do" and "if you don't" reading one answer |
| A player's choice at resolution, `EffectRecipient::ChosenBy` with a slot for each shape the census found, and `Sacrifice` on it (decision 4) | ~150 | 5: ATOM-118.12-001 on Standstill's board, as a fixture; the "if you can't" fixture; a stolen source answering `Cant`; "sacrifice that creature"; each opponent choosing in turn and sacrificing at once, at four seats |
| `Primitive::event_for`, and the six object verbs moved onto it (decision 4) | ~50 | 1: an exile edict through the same choice, which shows the choice belongs to every verb and not to sacrifice |
| `AddCounters` over a filter; `CountOf(CardsInHand)` in the layer walk; `EachOf` over `LoseLife`, which Crawler's "each opponent loses 1 life" needs and the row did not list | ~40 | through the cards' tests |
| The `departed` frames, one capture and one writer, and CR 109.5's "you" at both instants and in a resolving "if" (decision 3) | ~140 | 5: an enters trigger's "if" and power read after its source is sacrificed in response; the recheck of a stolen source; a frame from the stack, from a hand, and from an effect that moved its own source |
| Item 163's predicate over the facts a def reads (decision 6) | ~120 | 3: the binding-read board, the source row, equal and unequal amounts; the three migrated tests are edits |
| **Nykthos Paragon**, **Psychosis Crawler**, **Cosi's Trickster**; Crawler and Trickster pooled (175 → 178 registered, 96 → 98 pooled) | ~105 | 11: Paragon's six rulings (the fourth is ATOM-603.2h-002, made full), Trickster's three, Crawler's one, and Crawler cast from hand with exact mana under `ManaWindowStop` |
| **Total** | ~1,035, top end ~1,225 | 29, ~1,130 with the edits |

**2,165–2,355 in code and tests**, inside §4's band, against the
2026-09-24 count's 1,720–2,180. Docs add ~500 more: this section,
archived at landing, with the stub, the record and the items. What the count
moved:
- `EachOf` refuses `LoseLife` by name today, and Crawler needs it.
- The fold is six leaves, not five: `CardInYourGraveyard` says "your" too.
- The predicate compares the source when the def reads it (decision 6).
- The owner's review (2026-09-26) moved the choice out of `Sacrifice` into
  a recipient every verb can take, gave it a slot for each shape a census
  of the pool found, and put `event_for` under the object verbs. It also
  made the mask's width follow `EventKind` (decisions 4 and 5).
  - Together these add about 185 lines. Soul Shatter was offered at 100
    more and not taken (the owner, 2026-09-26).
- Standstill and Wicked Guardian stay fixtures. Standstill's "each of that
  player's opponents" is a group relative to the bound player, which
  `PlayerGroup` cannot say. Wicked Guardian's "another creature you control"
  is chosen at resolution (CR 608.2d), where the engine announces a `Choose`
  at placement, and its "another" is TR-3b's.

**Decision 1 — the fold: one variant, `whose` and a fact.** Six leaves each
read one fact about one player and carry the player in their name:

```rust
Condition::Player { whose: PlayerSet, fact: PlayerFact }
pub enum PlayerFact {
    ControlsPermanent(ObjectFilter), // "you control a Forest"
    LifeAtLeast(AmountExpr),         // "you have 40 or more life"
    LifeAtMost(AmountExpr),          // "you have 5 or less life"
    LibraryEmpty,                    // "your library has no cards in it"
    CardInGraveyard(ObjectFilter),   // "a red card in your graveyard"
}
// Kird Ape:   Condition::Player { whose: PlayerSet::You, fact: PlayerFact::ControlsPermanent(forest) }
// Bloodghast: Condition::Player { whose: PlayerSet::Opponents, fact: PlayerFact::LifeAtMost(AmountExpr::Fixed(10)) }
```

- **It holds when any player `whose` names meets the fact**, over the
  players still in the game, as `EachOf` and `resolve_player_ref` already read a
  set. "An opponent controls a creature" asks whether one opponent does, and
  `You` is one player.
- **It quantifies and does not sum.** `HistoryCount` sums its rows, and has
  to: "a creature died this turn" is every row. A sum would read "an opponent
  controls three artifacts" across two opponents, and a life total is not a
  count. The same sum misreads "an opponent lost 3 life this turn" at four
  seats. No registered card asks it, and when one does it takes the same
  quantifier.
- **Maintainability.** One arm in `holds`, `condition_reads` and
  `zone_function`'s match in place of six. A new player fact is one
  `PlayerFact` arm, and "each opponent" is a quantifier added with its card.
- **The first commit changes no behavior**, since each old leaf is exactly
  one pair. The scans are the same scans. §2b's rule 8 cites three of the
  old names as its examples and changes with them.

**Decision 2 — "may": its chooser, and CR 118.12's answer.**

```rust
Effect::Optional { chooser: PlayerRef, effect: Box<Effect> }
pub enum CostAnswer { Does, Doesnt, Cant }
Condition::CostAnswer(CostAnswer)   // "if you do", "if you don't", "if you can't"
struct ResolutionWalk { instance_cursor: usize, last_cost_answer: Option<CostAnswer> }
```

- **The walk.** `resolve_effect_at`'s `cursor: &mut usize` becomes
  `walk: &mut ResolutionWalk`, and the count moves to the walk's
  `instance_cursor`. It counts the instances of "target" declared so far,
  the glossary's *cursor* (2); sense (1) is the turn plan's. Each resolution
  makes a fresh walk, so a rider never reads its parent's answer (§6.2's
  amendment).
- **The chooser** is a `PlayerRef` (§6.2), resolved as `AddCounters`' `by`
  is: `You` is the controller, `Opponent` a player target or the only
  opponent. "That player may" (53 cards) is the bound player, which
  `PlayerRef` cannot name; it gets an arm with its first card.
- **The prompt** is `OptionalEffect { source }` (§9): yes or no, asked
  whenever the optional is reached.
- **The answer's writers.**
  - An atom writes `Does` as it performs: CR 118.12's "started to pay".
  - An atom that cannot start writes `Cant`. In TR-2b that is `Sacrifice`
    alone (decision 4). Another primitive gets its "can't" with its first "if
    you can't" card.
  - `Optional` writes `Doesnt` when declined. When accepted it keeps the
    action's answer, except that `Cant` becomes `Doesnt`: CR 118.3 lets no
    player pay a cost they can't, so "you may sacrifice a creature; if you
    don't, …" with no creature is "you don't".
  - The yes-or-no is asked even when the action cannot start. Sparing it is a
    per-primitive pre-check, `backlog.md` §2.22's rule 1, and no TR-2b card
    needs one: Paragon's and Trickster's counters always start.
  - A clause's own atoms don't answer for the action before it. The walk
    restores the answer after a `Conditional`, so "if you do … if you don't …"
    reads one answer.
- **The reader is a leaf, not a second combinator.** CR 118.12 calls the
  clause an "if", and `Conditional` is the tree's "if". A leaf also composes
  under `All`.
  - The walk's `Conditional` arm answers `CostAnswer` itself, inside `All`
    too, and hands every other leaf to the evaluator.
  - `holds` treats `CostAnswer` as it treats `ModeChosen`: a static context
    has no answer.
  - Item 24's sized `IfYouDo { did, didnt }` is the combinator not built.
- **CR 603.2h's writer reads the resolution's last answer** and records the
  action only on `Does` (§6.4). So Paragon's declined "may" leaves the gate
  open, its first and third rulings. A prevented action still answers `Does`,
  Wicked Guardian's third ruling, because the answer is the choice and never
  the stream.

**Decision 3 — the `departed` frame, after the bounded-state PR.** A binding
copies its records at dispatch, and the window is flushed after it. So a
departure after the dispatch is in no record the entry holds. Its frame has to
reach the entry as the object leaves.

- **The frame and where it lives.**
  `DepartedFrame { object: ObjectRef, frame: Arc<EffectiveCharacteristics> }`
  sits in a `departed` list on `PendingTrigger`, on `StackEntry` and on
  `ResolvingObject`. `StackEntry` means every entry: CR 113.7a names
  activated abilities too (§15 item 15).
  - Placement moves the list onto the stack entry.
  - Resolution moves it onto `resolving`. Resolution takes the entry off the
    stack, and an effect can move its own source mid-resolution (CR 608.2h's
    "the effect has moved it").
  - TR-4a swaps the frame for `LastKnownInformation`. Its `cost_choices` and a
    stack object's `entry` are item 169's other two amendments (§3.11).
- **One capture, one writer.** Item 174's pass already frames every permanent
  a batch's decided members take off the battlefield, before any of them
  performs. It now also frames any other mover that a pending, stacked or
  resolving entry names (its source, or `binding.subject`), from any zone.
  - The performer that takes a mover's frame hands it to each entry naming
    the mover, so for a battlefield departure the entry holds the record's
    own `Arc`.
  - The gate is a scan of those entries, which on the common board are empty
    or short.
  - Rejected: a `GameState` map keyed by `ObjectRef`. Every exit an entry has
    from the stack would have to prune it, and a missed prune is the slow leak
    the bounded-state PR removed.
  - Rejected: writing at the window's close. Only a battlefield departure's
    record carries a frame.
- **The readers.**
  - `bound_characteristics` reads the record's frame when the event was the
    departure, the live object while its epoch holds, and the entry's
    `departed` frame after that. TR-2a's refusal then marks a missing
    capture.
  - The intervening "if", at both instants, and a resolving effect's own "if"
    read CR 109.5's "you" as the ability's. `settled_holds` gains a form that
    takes the player, and `you_for` and `FilterPlayers::for_source` answer it
    before the source's frame. The player is the candidate's controller at
    dispatch and the entry's locked controller at the recheck (CR 603.3a). In
    `Effect::Conditional` it is the resolution's controller. A static ability
    keeps its source's current controller.
- **When the source is gone.** `SourceUntapped`, `SpellWasKicked` and
  `HostMatches` still read the live object, and answer false once it has
  left. Their last known answer is a status or a cost decision. Those are
  §3.11's fields, and TR-4a points the leaves at the frame when it builds the
  type.
- **Item 169 closes here.** Its base, the widening and "from every zone" land
  in TR-2b. The frame's cost decisions and a stack object's entry are §3.11's
  fields, which TR-4a's row builds.

**Decision 4 — a player's choice at resolution is a recipient, and
`Sacrifice` takes `Destroy`'s grammar.** An edict's "a creature" is one of
many objects a player chooses as the effect applies (CR 608.2d), from a set
defined relative to that player. The shape is not sacrifice's: 209 cards have
a player sacrifice, 81 a player exile, and 332 print "… of their choice"
across the verbs.

**The census, taken at the owner's review (2026-09-26)**, because the first
draft missed Soul Shatter's rank. Nine Scryfall queries for the family
returned 2,291 cards, and 783 of them carry a clause where a verb acts on
objects a player picks. Every printed shape found:

| Shape | Cards | Examples | Its slot in the type | Built |
|---|---:|---|---|---|
| a filter and a number, over the chooser's permanents | ~420 | Diabolic Edict, Abyssal Gorestalker, Azorius Chancery | `Pick { filter, count }`, the scope's first arm | TR-2b |
| a rank: "with the greatest … among" | 22 | Soul Shatter, Crackling Doom, Blot Out, Bounce Chamber | a constructor on `Pick` | with its first card |
| "up to N" | 47 | Covetous Elegy, Archfiend of Depravity | a count arm | with its first card |
| a fraction, rounded | 10 | Pox, Curse of the Cabal, Rakdos the Defiler | a count arm | with its first card |
| keep the chosen, and act on the rest | 21, and 2 where you choose for each player | Balance, Cataclysm, Breakthrough, Tragic Arrogance | a side arm | with its first card |
| one pick per card type | 6 | Cataclysm, Catch // Release, Mythos of Snapdax | several `Pick`s; one permanent may answer two (Cataclysm's rulings) | with its first card |
| another player's objects: an opponent chooses yours; you choose from their hand | 6; 88 | Wormfang Crab, Forgotten Lore; Duress, Coercion | a scope arm; a hand's waits for the reveal (`backlog.md` §2.9) | with its first card |
| a graveyard | 13 | Augusta, Order Returned; Curse of Oblivion | a scope arm | with its first card |
| a status in the filter | 20 | Celestial Flare's "attacking or blocking creature" | `ObjectFilter` leaves, which every filter shares | with its first card |
| piles | 18 | Make an Example | CR 700.3's piles, not this recipient | — |

Queries, each plus `game:paper -is:funny`:
- the three counts above: `o:/(each|target) (player|opponent) sacrifices/`;
  `o:/(each|target) (player|opponent) exiles/`;
  `o:/(sacrifices?|exiles?|returns?|destroys?|taps?|untaps?|discards?) [^.]* of (their|his or her|your) choice/`.
- the census's nine, together 2,291 cards:
  `o:/(each|target|that|defending|chosen|the chosen) (player|opponent)s? sacrifices?/`;
  `o:/(each|target|that|defending) (player|opponent)s? exiles?/`;
  `o:/(each|target|that) (player|opponent)s? returns? /`;
  `o:/return (a|an|two|three) [a-z ]*you control to (its|their) owner.s hand/`;
  `o:/(sacrifices?|destroy|exiles?|returns?|discards?) (the rest|all (other|others|the rest))/`;
  `o:/chooses? (a|an|one|two|three|x|up to|any number of|from among) /`;
  `o:/(greatest|highest|least|lowest) (mana value|power|toughness)/`;
  the "of their choice" query above; and `o:bolster`.
  - The 783 are the cards with a clause where one of the verbs meets a
    number, a choice or "the rest".
  - The ~420 is what no shape tagged: plain edicts, the bounce lands and
    bolster, with some noise ("a source of your choice", choosing a
    counter).
- a rank: `o:/(sacrifices?|exiles?|returns?) (a|an|one) [^.]*(greatest|highest|least|lowest) (mana value|power|toughness)/`
- up to: `o:/(chooses?|sacrifices?|exiles?|returns?) up to (one|two|three|x) [^.]*(they|you) control/`
- a fraction: `o:/(sacrifices?|discards?|exiles?) (half|a third|one third) /`
- the rest: `o:/chooses? [^.]*,? then (sacrifices?|discards?|exiles?|returns?|destroys?|puts?) (the rest|all other)/`,
  and `o:/you choose [^.]*(that player|each player) controls/ o:/(sacrifices?|destroy|exile) all other/`
- per type: `o:/(chooses?|sacrifices?) (from among [^.]* )?an artifact, a creature, an enchantment/`
- another player's: `o:/(an opponent|target opponent|each opponent) chooses [^.]*(you control|your graveyard|your hand|you own)/`;
  `o:/you choose [^.]*(card|nonland card|creature card|land card) from (it|their hand|that player.s hand)/`
- a graveyard: `o:/(each|target) (player|opponent)s? exiles? [^.]*from (their|his or her) graveyard/`
- a status: `o:/(sacrifices?|exiles?|returns?) (an?|one|two) (attacking|blocking|tapped|untapped)/`
- piles: `o:/(separates?|piles?)/ o:/(sacrifices?|chooses?)/`

**So every axis has a slot now, and each later row is one more arm.** A row
adds an arm on an enum the type already has, or a constructor on `Pick`. No
row restructures the type. `Pick`s are built through constructors, so a field
added later changes no card's literal:

```rust
EffectRecipient::ChosenBy(Box<Choice>)
pub struct Choice {
    pub chooser: EffectRecipient,  // who picks: Controller, a target player, EachOf(..)
    pub among: ChoiceScope,        // whose objects, and where; TR-2b: the chooser's permanents
    pub picks: Vec<Pick>,          // an edict's one; Cataclysm's four
    pub acts_on: ChoiceSide,       // TR-2b: the chosen; later, the rest
}
pub struct Pick { pub filter: ObjectFilter, pub count: PickCount }   // TR-2b: PickCount::Exactly(AmountExpr)
// Diabolic Edict: Atom(Sacrifice, ChosenBy(Choice { chooser: Target(Player, Exactly(1)), among: ChoosersPermanents, picks: [Pick::exactly(1, Creature)], acts_on: Chosen }))
// Standstill:     Atom(Sacrifice, ThisObject)
```

- **Who chooses, and when.** Each player `chooser` names chooses in APNAP
  order (CR 101.4). The verb then acts on everything chosen in one batch,
  which is CR 101.4's "then the actions happen simultaneously" and Soul
  Shatter's ruling.
  - **A later chooser knows the earlier choices only where they are public**
    (101.4b). A choice in a hidden zone stays face down (101.4a). Balance's
    first ruling makes its lands and creatures known as they are chosen, and
    reveals its discards only once every player has chosen.
  - Which earlier choices a prompt may show is the information model's
    (`backlog.md` §2.9). TR-2b's choices are all on the battlefield.
  - Today's edict performs one batch per player, so a four-seat "each
    opponent sacrifices" would split. That is unreachable, since the one
    registered edict targets one player.
- **The candidates** are the scope's objects that match the pick's filter.
  - The filter reads "you" as the effect's controller, as every filter does.
    Wormfang Crab is why: "an opponent chooses a permanent you control".
  - Removed: any the verb's own event would be prohibited on (CR 101.2,
    `cant-effects-architecture.md` §4.9). Under Sigarda an opponent's edict
    finds no candidate at all, and an exile edict is untouched.
  - Asked with two or more, and forced when there are only as many as the
    count.
- **`Sacrifice` takes no payload.** Its recipient is the object, as
  `Destroy`'s is: `ThisObject`, `TriggeringObject` or `ChosenBy`.
  - The player who sacrifices (CR 701.21a) is the chooser, or the
    resolution's controller for a named object.
  - A permanent that player doesn't control, or that a "can't" protects, is
    not sacrificed, and the atom answers `Cant` (decision 2). Those cases are
    Standstill exiled before its trigger resolves (ATOM-118.12-001), a stolen
    source, and an empty choice.
  - "Its controller sacrifices it", a named object and another player, is a
    field with its first card.
- **One abstraction under the object verbs, and the choice is its first
  reader.**
  - A verb applied to objects has three parts. The recipient names the
    objects (targets, "this", "that", a filter, a choice). The verb names one
    object's event. One batch performs them all (CR 608.2f).
  - `Destroy`, `Exile`, `Tap`, `Untap`, `AddCounters` and `RemoveCounters`
    each write the middle part inline today.
  - The choice needs that part before anything is chosen, for the "can't"
    check. So it becomes one method,
    `Primitive::event_for(object) -> Option<GameAction>`, which the performer
    and the check both read. One table, so the event Sigarda is asked about
    is the event performed.
  - **All six move onto the method here** (the owner, 2026-09-26): about 50
    lines, with no behavior change, since the engine arm stays `IDENTICAL`.
    The choice then works for every verb that has an `event_for` arm, and
    `ReturnToHand` gets its arm with TR-3.
  - Not taken: the method for `Sacrifice` alone, with each other verb moved
    by its first chosen card. It was 50 lines fewer, and it left two
    mappings per verb that must agree until then.
- **The rank lands with its first card, which is not TR-2b's.** Soul
  Shatter was offered as that card here, at about 100 lines, and the owner
  left it for later (2026-09-26). It needs no retrofit: it is one
  constructor on `Pick` and one narrowing step.
- **Rejected: the first draft's `Sacrifice(Sacrificed)` payload.** It made
  the choice sacrifice's own, and every verb in the census would have grown a
  copy.

**Decision 5 — the mask: a width that follows `EventKind`.** The fourteen
kinds with `CardDrawn` and `LibraryShuffled` fill `u16`. The question is how
far the kinds grow.
- **Cards don't add kinds.** A card that needs something no arm expresses
  adds a field to a record (§3.3's contract). A new kind is a new
  `GameEvent`: an action the engine performs that some printed trigger
  reads.
- **Custom cards are the same case.** A new kind needs a new action, which
  is engine work, and the kind comes with it.
- **The ceiling is the CR's actions.** §3.3's table has thirty kinds with an
  arm by TR-5. CR 701 lists 67 keyword actions in tmnt (701.2–701.68), and a
  set can add one.
  - The survey's Phase 8 row plans one record per watched keyword action:
    explore, cycle, crew, connive and the rest.
  - Then come dice (CR 706), coins (705) and designations (724, 725, 730).
  - So a fixed `u64` can be outgrown within Phase 8.
- **The shape.** `EventKindMask([u64; EventKind::WORDS])`, with
  `WORDS = EventKind::COUNT.div_ceil(64)` and `COUNT` taken off the last
  variant under a compile-time assertion.
  - While the kinds fit in 64 it is one word, and the same instructions as a
    `u64`. A 65th kind grows the array with no edit to the mask.
  - It costs about 10 lines more than a fixed width.
- **Rejected: `u128`.** It is one more fixed width to outgrow.
- **Not precluded.** One record kind for the whole keyword-action family,
  with the action as a field, is a schema choice for Phase 8.

**Decision 6 — the elision's predicate: equal on every fact the def reads.**
Four of §5.2's conditions stay: equal defs, no instance of "target", no mode,
tier 1. "Identical bindings" becomes this: the entries agree on every fact the
def reads.

`TriggerDef::bound_reads()` walks the effect and the intervening "if". It
matches exhaustively over `Effect`, `EffectRecipient`, `AmountExpr` and
`Condition`, and over `Primitive` for its amounts, so a new leaf cannot
compile until it says what it reads.

| The def reads | Compared across the entries |
|---|---|
| "that object", "that spell", "that ability": the event's subject (`TriggeringObject`) | `binding.subject` |
| "that player" (`TriggeringPlayer`) | `bound_player` |
| "that many" (`TriggeringAmount`) | `bound_amount` |
| "its power", "its toughness" | the subject, and the record its frame comes from |
| "this object" (`ThisObject`, `SourceInZone`, `SourceUntapped`, `HostMatches`, `SpellWasKicked`, `Attach`) | `origin` |
| "this ability", the resolving trigger's own (the CR 603.2h gate, `ResolvedThisTurn`) | each identity's gate and count |

A fact the def does not read may differ. Two landfall triggers whose effect
ignores the land go on the stack unasked, and so do Soul Warden's two triggers
for Raise the Alarm's two Soldiers. Two Paragons with both gates open
(ruling 2) still go unasked. Their states are equal, so either order is the
same game.

**"That ability" is an event's subject, not "this ability".**
- **Battlemage's Bracers**: "whenever an ability of equipped creature is
  activated, … copy that ability". It binds the activated ability as its
  subject once TR-5's `ActivatesAbility` arm puts the stack object on
  `AbilityActivated`, which carries only the ability's durable identity
  today. The predicate then compares it as it compares any subject.
- **A copy of an ability that has left the stack** reads its `departed`
  frame from the stack (decision 3), whose `entry` is §3.11's (TR-4a). The
  copy itself is CV-4's. "You may pay {1}" is CP-1's payment, which
  answers `Does` like any action.
- **A tier-2 trigger's "that ability"** is `triggered_by`, and tier 2 is
  never elided.

**The source row corrects TR-1's predicate.** §5.2 compares bindings because
"that creature" is otherwise a different object. By the same argument, "this
creature" is a different object when the sources differ, and TR-1 compares
no source. Two Vengeful Warchiefs triggering on one life loss each put a
counter on themselves. Which one has its counter is on the board in the
window between the two resolutions, and a response can use it, so the two
orders are two games. `main` elides that choice, and the new predicate asks
it. So the random agent's stream moves in both directions.

**The migration, sized by running it.** Both predicates were patched in as a
throwaway and the whole suite run. Under each, 1,760 tests passed and the
same three failed, all in `phase_tr1_integration_test.rs`. **Three of the six
scripted `OrderTriggers` expectations move:**
- `soul_warden_entering_beside_two_creatures_triggers_for_each_of_them` and
  `an_artifact_dying_in_the_wipe_still_sees_the_creatures_die` each gain 1
  life and read nothing bound. They are now placed unasked.
- `identical_triggers_with_different_bindings_are_asked_their_order` has the
  prompt as its instrument, so it becomes decision 6's binding-read board.
- Blood Artist's two tests, which have targets, and the two different defs
  are asked as before.

**The A/B, predicted before any arm runs.** `fuzz_ab.py` on both pools at two
seats and four, 200 games at seed 12345. The counter runs are audited, and
timing is 3 × 200 on `performance`. There are four arms:
- **`main`**: #191's merge, `5fdf342`.
- **engine**: TR-2b with TR-1's predicate, and the three cards unregistered.
- **elision**: the engine arm plus the new predicate, with the cards still
  unregistered.
- **shipped**: the cards registered and two of them pooled.

The elision arm is the brief's third arm. It needs its own because pooling
Crawler and Trickster changes every deck, and §3 never reads a number across a
pool change. The shipped arm is that pool change, recorded and not budgeted,
as TR-2a's was.

| | predicted |
|---|---|
| engine vs `main` | Every gameplay row `IDENTICAL` on both pools at both seat counts, and the dumps identical game by game. `Windows past gate` and `Candidate visits` identical, or up where a zone-map or granted trigger exists: a draw's or a shuffle's window now has a kind, as a cast's did in TR-2a (+0.6 visits at four seats). Every other row identical, since only a mover outside the battlefield that an entry names costs a new frame, and no pooled card makes one. CPU per decision inside the 2.5-point budget |
| elision vs engine | `differ`, and **each diverging game's first difference is an `OrderTriggers` prompt that one arm asks and the other does not**. Only the engine arm asks Soul Warden's triggers for two or more creatures entering in one batch (Raise the Alarm, doubled by Parallel Lives). Only the elision arm asks two Vengeful Warchiefs of one player on one first life loss, and on `stress` two Paladins of Atonement at an upkeep. Every other game is identical |
| shipped | A pool change. `Triggers placed` rises with Crawler's draws, one trigger per draw per Crawler. Trickster's trigger is rare: on `performance` its only shuffle is an opponent's Darksteel Colossus shuffled back in, which `--require` reads |
| audit | agrees on every arm |

**The commits.** Each commit carries its own tests and re-measures the band.
1. The fold, A6b's first commit.
2. The mask and the two arms.
3. "May" and CR 118.12's answer.
4. The choice at resolution, `event_for` under the object verbs, and
   `Sacrifice` on them.
5. The three small facilities.
6. The `departed` frames and "you". This head is the engine arm.
7. The predicate and the migration. This head is the elision arm.
8. The cards, registered and pooled. This head is the shipped arm.
9. The record.

§5.2, §6.1, §6.2, §7 and §11 are rewritten by the commits that build their
pieces.

### As landed (2026-09-26)

It landed at +2,430 additions in code and tests, against the count's
2,165–2,355, and inside §4's band. Twenty-four tests, where the table planned
29: boards that share a setup share a test.
- **The `departed` frames' five boards are four tests.** The stack and the
  hand share one.
- **The cards' eleven are seven.** Rulings that share a board share a test.
  Paragon's fourth ruling is tested on ATOM-603.2h-002's own board, two
  lifelink creatures dealing combat damage at once.
- **Cosi's Trickster's second ruling, that cascade is not a shuffle, takes a
  `no-registered-card` disposition.** Nothing registered cascades, and no
  primitive puts cards on the bottom in a random order.

What the close-out found:
- **A redundant CR 101.2 query.** The first sitting read `Restriction
  queries` 2,216 → 2,217 at four seats on `performance`, where the prediction
  said no such row would move. `Sacrifice` re-ran `admits` on a permanent its
  choice's candidate filter had just admitted, with nothing moved in between.
  The re-check was folded out of the choice's commit, and the re-run reads the
  row identical to `main`.
- **`Memo hits` fell by 1–9**, which the prediction also missed. A probe
  rebuilt the engine arm with the old reads restored, and every reading came
  back within 1 of `main`'s. The edict's choice reads its filter only for the
  chooser's own permanents, where `main`'s enumeration read every creature's.
  The named "you" skips the source's frame, 2 of four-seat `stress`'s 8.

**Trace page: no**, decided at its close. §7's test is a change to *how* a
read is answered:
- The `departed` frame is a new place `bound_characteristics` reads, after the
  two it had. Four tests walk each route: a permanent, the stack and a hand,
  and an effect that moves its own source.
- CR 109.5's "you" is a named player where a derived one was. Two tests, the
  owner's graveyard and a steal, state it whole.
- The elision's change is which columns are compared, and `bound_reads.rs`'s
  exhaustive match is that table.

