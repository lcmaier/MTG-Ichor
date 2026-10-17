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

### Review round 4 (2026-09-27)

The owner's code review of #193, after the three design rounds. It changed
names and shapes, and no behavior:
- **One evaluator.** `ConditionYou` was `Option<PlayerId>` under another
  name, and it carried a second wrapper, `settled_holds_for`. `settled_holds`
  takes `locked_you: Option<PlayerId>`. `None` reads CR 109.5's "you" live, as
  a static ability does, and the three askers that have fixed it pass `Some`.
- **`players`, not `whose`,** on `Condition::Player` and TR-2a's
  `HistoryCount`. The field is a `PlayerSet`, and `whose` read as a question
  at the call site.
- **`BoundReads` is a bit set**, built like `Channels`, and each walker is a
  pure function that returns the facts it reads.
- **`EventKind` counts itself.** `counted_enum!` reads `COUNT` off the variant
  list, where the last variant's index plus one was a second edit a new kind
  could forget. The mask derives `Default`.
- **`ApplyOptionalEffect`**, beside `ApplyOptionalReplacement`.
- **Names and comments:** `hand_over_departed_frame`, `is_permanent`,
  `ResolutionWalk::start()`, and four comments that said less than they meant.
- **Tests:** Innocent Blood's order at four seats, where the caster chooses
  third; the countered spell and the discarded card as two tests; the stolen
  source's rules cited. Twenty-six tests.
- **Docs:** the glossary defines dispatch without leaning on "matcher", and
  defines the matcher. `engineering-practices.md` gains §2c, one collection of
  records rather than two kept in step, and §4's docs figure is written net of
  generated files.
- **Filed, not built:** `StackEntry.effect` is an owned `Effect` that a fork
  deep-copies, item 180's shape, bounded by the stack (`codebase-state.md`
  item 183). It is its own small PR after this one.

The round's head played the same games as the landing head: every counter
`IDENTICAL` on both pools at two seats and four, and the audit agreed on the
same dispatches (`fuzz-record.md`, TR-2b).

#### TR-3a — the delayed-trigger registry, with Final Fortune and Blessed Wine — ✅ landed 2026-10-07

*Evicted 2026-10-07 from `plans/triggers-architecture.md` §12, where the heading and a stub remain. TR-3 was sized whole; the plan below is that sizing as written, and TR-3b's half of it stays live in §12.*

### The plan as sized (TR-3, as §12 carried it on 2026-10-07)

| Piece | ~additions |
|---|---|
| the registry, `DelayedTrigger`, `DelayedSource`, `DelayedDuration`, `ObjectRef`, `Instant`, `ExtraTurnId` on `turn_queue` and `current_turn_origin`, `Primitive::CreateDelayedTrigger`, provenance from `ResolutionContext` and from a rider, 107.3n's X, `ChooseDelayedTriggerEvent`, cleanup expiry of `ThisTurn`; `Effect::Reflexive` and the immediate check; `UntilEvent` resolved at dispatch (610.3) | ~520 |
| `Primitive::ReturnToBattlefield` and `ReturnToHand` made real over `change_zone` / `EnterBattlefield` (the stub arm at `resolve.rs:1417`), with 610.3c's owner's control; a source-relative "another" for a sacrifice chooser | ~120 |
| cards: **Final Fortune** (603.7d, a named extra turn; its ruling — a skipped extra turn loses nothing — is the `ExtraTurnId` test), **Flickerwisp** (603.7e from a triggered ability, 603.7c through exile, CR 400.7; its second ruling is 513.2's sibling), **Cornered Crook** (603.12: `Optional` then reflexive, any target — Heart-Piercer Manticore prints the same shape with an LKI power read and cannot register whole, since embalm is `backlog.md` §2.3's and CV's), **Banishing Light** (610.3's until-return, no stack; its ruling that an Aura or Equipment on the exiled permanent falls off is CR 400.7's, and "leaves before the trigger resolves, nothing is exiled" is 610.3a); Flickerwisp and Banishing Light pooled (the registry, the until path) | ~320 |
| tests, 30: §13's 20 TR-3 atoms (513.2 both ways, 603.7f through a rider fixture and 603.7g's fixture among them); Heart-Piercer Manticore's four trigger rulings as fixtures (the LKI power read); Tatsumasa's simultaneous choice as a fixture; Sneak Attack's ruling as a fixture board (the card's indefinite haste is expressible since CV-1b); the three card rulings above — Final Fortune's, Flickerwisp's second, Banishing Light's Aura; `refs` under Parallel Lives, each token made exiled (§3.9's amendment) | ~1,110 |
| docs, ledger, record | ~250 |

**Split 2026-09-24**, by the re-count below. **TR-3a** is the delayed
registry, with Final Fortune. **TR-3b** is reflexive triggers, the returns
and "until" (610.3), with Flickerwisp, Cornered Crook and Banishing Light.

### As landed (2026-10-07)

Six commits: item 222, the extra turn's id, the registry, its atoms' tests,
"the next turn's" binding, and the two cards.

**Item 222, first, as its own arm.** A copy or grant row naming its objects
(`SourceOnly`, `Fixed`, `Host`) files the record kinds its triggered
abilities read in `RegistryScopeSummary::named_unattributed_trigger_kinds`,
and the dispatcher reads the named objects off the rows only for a window
carrying one of them: the replacement gather's named leg with the printed
leg's mask. A `Filter` row keeps opening its zones. The unused
`puts_a_triggered_ability` became `triggered_ability_kinds`, which both read.

**The extra turn's id** (§3.9's 2026-09-24 amendment). `turn_queue` holds
`ExtraTurn { id, player }`, minted by `schedule_extra_turn`;
`next_turn_taker` hands the id back; `GameAction::BeginTurn` carries it as
`extra`, and its performer records `GameState::extra_turn`. The turn's
`EventPattern` field waits for the first of the four cards that replace "a
player would begin an extra turn".

**The registry** (§3.9 and §4.6's as-built notes have the decisions):
`engine/triggers/delayed.rs` and the dispatcher's second leg; the arm
matcher reading `TriggerReferents`; `TriggerOrigin::Delayed(AbilityIdentity)` on
`AbilityId::delayed`; a source's departure record, noted by the three
movers whose record follows the move (`perform_zone_change`, a player leaving, CR 601.2i's cast);
`created_at` and `created_in`; `ChooseDelayedTriggerEvent` through every
list of prompt kinds; `TriggerEvent::AbilityResolves`, with `EventKind`'s
seventeenth kind; `AmountExpr::X` reading the resolving object's X; the
cleanup and turn-begin expiries; the scenario writer reporting a delayed
trigger and an extra turn in progress as unwritten.

**Two findings while building.**
- *CR 603.7a needed code.* "The window is the future" holds for a
  resolution, which runs outside any batch, and fails for a rider: it runs
  inside the batch it rides on, and Destroy's move closes its own nested
  batch with its riders, before the batch's next member is destroyed. A
  rider's "the next time a creature dies" saw the other creature of the same
  wipe die. An entry now skips the batch it was created in, and the test
  failed before that.
- *"The next turn's upkeep" is not "the next upkeep".* A second upkeep this
  turn (CR 500.10) is not the next turn's. Nothing makes one yet, so the
  binding is written for the CR's case: `DelayedTurn::NextTurn` resolves to
  `TriggerTurn::LaterThan` the creating turn, and Blessed Wine's test proposes
  a second upkeep by hand.

**The cards, and the owner's call.** The brief named Final Fortune,
registered and not pooled. Measured before landing, at seed 12345: it ended
29 of 200 natural two-seat stress games at its extra turn, was cast 3 times
in 200 four-seat games, and with a copy forced into every deck the random
agent cast it in 65% of two-seat games. Every printed "that turn" card makes
its caster lose (Final Fortune, Last Chance, Warrior's Oath, Alchemist's
Gambit), so no other card spares stress that. The owner kept Final Fortune,
for the shapes stress hunts (its caster leaving mid-turn at four seats, CR
800.4), and added Blessed Wine, an Ice Age cantrip whose delayed draw ends no
game. Neither is pooled: §12 pools the registry with TR-3b's two cards.

**What TR-3a left for TR-3b.** `refs`: a trigger naming an object other than
its source. The atom 603.7b-002's own board is Tatsumasa's "that token"
under a doubler, and 603.7e-001's card is Flickerwisp; both atoms are covered
on the rule's own boards (two deaths at once under a filter; an activated
ability and a token's enters trigger), and the boards wait for `refs`.

**Size.** +2,278 in code and tests: code +1,205, tests +1,073, against the
re-count's 1,600–1,980. The tests ran at their usual multiple, and the
code's overage is the `TriggerReferents` refactor (`ArmReader` as it was
first written) and the two findings above. The review round (2026-10-08)
added code +120 and tests +91: item 224, `source_frame`, the
`delayed_occurrences` split and the explainers, for +2,489 in all.

#### TR-3b — the returns and "until", with Flickerwisp and Banishing Light — ✅ landed 2026-10-08

*Evicted 2026-10-08 from `plans/triggers-architecture.md` §12, where the heading and a stub remain, with TR-3c's section after it.*

### The plan as sized (TR-3b, as §12 carried it on 2026-10-08)

TR-3's other half, by the 2026-09-24 re-count (the last subsection); TR-3a's
stub says what landed of the plan.

| Piece |
|---|
| `Effect::Reflexive` and the immediate check over the resolution's own records (§4.6); `refs`, the objects a creating instruction names, filled from those records (Flickerwisp's "that card", the Tatsumasa board's "that token"), each with a departure record as the source has; `UntilEvent` resolved at dispatch (610.3) |
| `Primitive::ReturnToBattlefield` and `ReturnToHand` made real over `change_zone` / `EnterBattlefield` (the stub arm in `resolve.rs`), with 610.3c's owner's control; a source-relative "another" for a sacrifice chooser |
| **`codebase-state.md` item 223's row half**, carried by `plans/handoffs/item-223-rows.md` (the owner's split at #233's design review, 2026-10-08), ~170 lines: the ten sites that write a resolution's registry row read `ResolutionContext::effect_source`, and `cleanup_zone_state`'s sweep ends only the rows that end with their source, a static ability's (CR 611.3b) and a "for as long as" (611.2b), beside 610.3's "until"; closes items 223 and 95. **Item 225**, ~70 lines: a chosen-source shield kept past its source's departure, since Cornered Crook killed in response deals its damage as it last existed. **Item 226**, one token: Cobbled Wings' printed {2}, riding this phase's `stress` re-record |
| cards: **Flickerwisp** (603.7e from a triggered ability, 603.7c through exile, CR 400.7; its second ruling is 513.2's sibling), **Cornered Crook** (603.12: `Optional` then reflexive, any target — Heart-Piercer Manticore prints the same shape with an LKI power read and cannot register whole, since embalm is `backlog.md` §2.3's and CV's), **Banishing Light** (610.3's until-return, no stack; its ruling that an Aura or Equipment on the exiled permanent falls off is CR 400.7's, and "leaves before the trigger resolves, nothing is exiled" is 610.3a); Flickerwisp and Banishing Light pooled (the registry, the until path) |
| tests: §13's 7 TR-3b atoms; Heart-Piercer Manticore's four trigger rulings as fixtures (the LKI power read); the Tatsumasa board, under a doubler; Sneak Attack's ruling as a fixture board; Flickerwisp's second ruling and Banishing Light's Aura; `refs` under Parallel Lives, each token made exiled (§3.9's amendment) |

**Re-counted 2026-10-08: the card-by-card gap hunt** (`engineering-practices.md`
§4), at TR-3b's sizing. Every clause of the three cards and every test board
was read against the tree. The rows stand, with three corrections: Banishing
Light's "leaves before the trigger resolves" is CR 610.3b, since its ability is
triggered; `ReturnToHand` moves to TR-4 with Rancor, because ATOM-610.3b-001's
bounce is the test's own proposal; and §3.9's "610.3d's simultaneity falling out
of the window" does not hold (item 3). Ten facilities the rows did not list:

1. **The window does not hold a resolution's records.**
   `flush_window_unless_nested` (`engine/triggers/dispatch.rs`) empties it at
   the close of every outermost batch, and each instruction of a resolution is
   one. By the time "when you do" or "that card" is reached, the records it
   reads are gone. §4.6's reader, `resolution_records(stamp)`, needs the
   resolution counted as a reader: the window holds until it ends.
2. **"Another" at resolution.** `object_matches_filter` carries no source, so
   `ObjectFilter::NotSource` refuses there, and `choose_as_it_applies` reads
   the refusal as no candidate. Heart-Piercer Manticore's "sacrifice another
   creature" would sacrifice nothing. The five resolution-time
   `FilteredPermanents` sweeps in `resolve.rs` have the same gap, and no
   registered card reaches one. One helper that passes the effect's source
   answers all six.
3. **CR 610.3d is one batch.** A return proposed per entry is a batch per
   entry, so the returns one event causes are proposed together.
4. **The until-event's object can be a target**: Calix, Destiny's Hand's "until
   target enchantment you control leaves the battlefield", which is
   ATOM-610.3a-001's board. Of 135 printed "until … leaves the battlefield"
   cards (`o:/until .* leaves the battlefield/`), the other 134 watch the
   source.
5. **A return names its controller.** CR 610.3c's "unless otherwise specified"
   and Flickerwisp's "under its owner's control" need it. ATOM-610.3c-002 is a
   fixture, since no printed until-exile says "under your control".
   *(Sharpened at the review: that is the until shape only. `ReturnUnder::You`
   itself has printed customers in plenty, Cloudshift's and Restoration
   Angel's flickers and Animate Dead's reanimation among the 60 cards that
   print "return … to the battlefield under your control".)*
6. **CR 303.4f/g's host choice.** Who chooses: the player under whose control
   the Aura enters. From what: what CR 704.5m's check accepts, among
   permanents already on the battlefield (Calix's last ruling). Which prompt.
   And 303.4g's "remains in its current zone".
7. **An Aura that enchants a player has no host to enter attached to**, since
   `PermanentState::attached_to` names objects. There are 47 printed, 46 of
   them legal in Commander (`o:"enchant player" t:aura`), all Curses, and none
   registered. CR 303.4f refuses one loudly; a new `codebase-state.md` item is
   to be slotted at the review.
8. **A replaced move refers to nothing.** A card Rest in Peace exiles instead
   of a graveyard is not "that card" in the graveyard, so a reference is
   taken only where its record put the object. *(Corrected at the build: the
   example first given, a commander Flickerwisp exiles, is CR 903.9a's
   state-based action, not a replacement; it ends as a new object all the
   same.)*
9. **Item 225's identity.** The shield's chosen source and the damage's dealer
   are both an `ObjectId`, and an id survives a move (only the epoch changes).
   Dropping the prune alone would let the shield see the new object's damage.
10. **Item 99's other legs.** "An object referred to by an object on the
    stack" is what lets Circle of Protection: Red choose Cornered Crook after
    it died in response. These are the delayed leg's candidates read off two
    more collections, so they are one enumeration.

**Size**, in code and tests. The returns half: about 780 in code and 960 in
tests. The reflexive half: about 320 and 480. Whole, that is about 2,550, and
about 2,900 with code at its usual 1.3× of the count, against the band's
2,500. The proposed split follows the seam the brief named:
- **TR-3b**, the returns, about 1,750–2,000: the held window, `refs`,
  `ReturnToBattlefield`, CR 303.4f/g, CR 610.3 with its six atoms, item 223's
  rows, item 226, and Flickerwisp and Banishing Light pooled.
- **TR-3c**, the reflexive trigger, about 800–950: CR 603.12, the
  source-relative filter, item 225, item 99's three legs, and Cornered Crook,
  with Heart-Piercer Manticore's rulings as fixtures.

Whole or split is decided at the design review.

### As landed (2026-10-08)

**The design review** (#234) took four decisions:
- item 225's damage names its dealer's existence through the departed frame
  it already carries;
- CR 303.4f's host is asked through the existing non-targeting
  `SelectRecipients`, and Auras that enchant a player go to Phase 8
  (`codebase-state.md` item 227);
- `refs` is `Effect::Remember`, marking the instruction rather than naming a
  zone, after the owner asked why the affected objects' ids would not do. The
  ids are what is stored, and the question is which ones: a zone rule misses
  a target that never moves (Slave of Bolas) and every player;
- CR 610.3's return is a list of its own.

The owner then split the phase: TR-3c is the reflexive half.

Seven commits:
1. Item 223's rows, with the handoff's three tests failing on the pre-fix
   tree, and the pump test failing with the sites moved and the sweep not.
2. `Effect::Remember`, with the held window; four of its tests fail without
   the hold.
3. `ReturnToBattlefield`, CR 303.4f/g and Flickerwisp.
4. CR 610.3 and Banishing Light, whose 610.3d test fails when each return is
   its own batch.
5. Item 226.
6. The registration.
7. The close-out.

**Where the band went.** Code +999 against a count of about 780, which ran at
the usual multiple: exhaustive matches answering three new variants, a
`moves_into` table over every primitive, and the doc comments. Tests +1,017
for thirty-one tests and their fixtures.

**prompt_subject_test** walks decks cut from the sorted registry, and two new
names shift every window after them. At forty turns it raised 367 prompts
and no blockers; at fifty it raises 548 with every kind it guards. Blockers
are at one in every variant tried, so the guard is still near its edge.

**The review** (#234, 2026-10-08) took three changes after the close-out.
- **The until-event is any `TriggerEvent`.** It was `UntilLeaves`, an object
  whose leaving ended the exile. The owner asked why a leaving was built into
  the type when a return can wait on something that is no object's, and
  Palace Jailer's "until an opponent becomes the monarch" is such a card. The
  event is now read by the trigger matcher (`take_returns_due`), and
  `refers_to` carries Calix's target as `TriggerSubject::Referred`. CR
  610.3a/b's "already occurred" needs the event's history since the trigger,
  which is kept only for a leaving, so any other event's resolution is
  refused loudly (`codebase-state.md` item 230, Phase 9 with the monarch).
- **CR 111.8.** Cloudshift's ruling, that an exiled token "won't return to
  the battlefield", is the rule, and the engine let a token come back.
  `execute_actions` drops a departed token's move before anything is
  proposed, since the rule is no event and no replacement sees it. The test
  fails without the drop. Covers ATOM-111.8-001.
- **The 611.2b half of `ends_with_its_source` is `Duration`'s.** A duration
  that ends with its source says so wherever it is registered, so item 17's
  sweep over the replacement and restriction registries reads the same
  predicate as the continuous one.

The review added 345 lines of code and tests and removed 127, most of them
this phase's own, so the phase closes at +2,235 against `main`. Against shipped
`a8fc1da`, every row outside timing played identically on both pools at two
seats and four (`fuzz-record.md`, TR-3b).

#### TR-3c — the reflexive trigger, item 225 and item 99 — ✅ landed 2026-10-09

*Evicted 2026-10-09 from `plans/triggers-architecture.md` §12, where the heading and a stub remain, with TR-4's section after it.*

### The plan as sized (TR-3c, as §12 carried it on 2026-10-09)

TR-3b's other half, split at its design review (#234, 2026-10-08) along the
seam its brief named. It reads the window TR-3b holds. Sized 800–950 in code
and tests by the 2026-09-24 re-count; item 103 joined after it (#238's
triage).

| Piece |
|---|
| CR 603.12: `DelayedDuration::Reflexive` on `Primitive::CreateDelayedTrigger` rather than §4.6's `Effect::Reflexive`, since 603.12 has reflexive triggers follow the delayed rules (one producer, one door). Checked as it is created against `EventWindow::resolution_records`, queued once per matching record (603.12a), never waiting. "When you do" is `ZoneChange { cause: Sacrificed, to: any }` |
| `codebase-state.md` item 229: a source-relative filter at resolution, so "sacrifice another creature" has candidates |
| **Item 225**, as #234's review decided it: the damage's `source_frame` a `DepartedFrame`, `SourcePattern.object` an `ObjectRef`, the prune sparing a chosen source that moved, and a spell becoming a permanent re-pointing it (CR 400.7c). **Item 99**'s (a): an object referred to by a stack object, a waiting replacement or prevention effect, or a delayed trigger, even once it has left its zone (CR 609.7a): one enumeration, its options told apart where two existences share an id |
| cards: **Cornered Crook** (603.12: `Optional`, then reflexive, any target), registered, not pooled; Heart-Piercer Manticore's shape as a fixture (embalm keeps the card out) |
| tests: ATOM-603.12-001; Manticore's four trigger rulings (the LKI power read, "another"); Cornered Crook's ruling, and the Crook killed in response dealing its damage as it last existed (lifelink, deathtouch); item 225 on a Perilous-Myr-shaped fixture under Circle of Protection: Red; item 99's legs; the Crook cast from hand under `ManaWindowStop` |

**Re-counted 2026-10-09: the card-by-card gap hunt** (`engineering-practices.md`
§4), at the build's start. Every clause of Cornered Crook, the Manticore
fixture and the Perilous-Myr fixture, and every test board, was read against
the tree. The rows stand. Twelve facilities they did not name:

1. **The resolution's start, as a mark.** `resolution_records` reads from a
   mark, and nothing marked where a resolution began: `ResolvingObject::began_at`.
2. **The queue's tail as one function**, so a reflexive trigger is queued and
   its `AbilityTriggered` announced as a dispatch does it (`queue_pending`).
3. **`GameEvent::DamageDealt`'s frame a `DepartedFrame` too**, since the event
   carries the proposal's.
4. **The departure record's frame a `DepartedFrame`.** A dies trigger names its
   source by the card it became (CR 400.7's exception for finding the new
   object), so only the record's frame knows which creature died, and so
   which one deals "it deals 2 damage". The row named the damage's frame and
   not this one; a shield that chose the creature would not have matched.
5. **Redirection's "the source's controller"** reads the new frame type.
6. **One damage-source enumeration of existences**, replacing the three copies
   (enumerate, count, validate) that spelled the categories each.
7. **`ChoiceOption::Departed`**, so two existences of one id are told apart:
   the decision log's words and the dev GUI's option view each gain an arm.
8. **The filter table reads its object only where no frame answers**, so a
   token the store has lost is still red as it last existed: item 103's
   last-known-information half, which no frame-holding path could reach while
   the object was read first.
9. **Two test fixtures**: a deathtouch grant and a sacrifice outlet.
10. **The dev GUI's `view_model.rs`**, whose `DelayedDuration` match is
    exhaustive.
11. **An authoring word for "is sacrificed"** (`sacrificed`).
12. **`DelayedProvenance` carries the creating resolution**, and CR 603.7a's
    "existed before" filter is skipped for the check.

With item 103 and the "doesn't" form the hunt put the phase at 1,250–1,500,
inside the band and over the row's 800–950.

**The design questions** (the owner, at the build's start). CR 603.12's
"doesn't" form: built now, with a fixture. The brief had said three printed
cards use it; none does as a reflexive trigger (Olivia, Crimson Bride's is a
state trigger, and Heart of Bogardan's and Thought Lash's are their own
triggered abilities), and two phrasings of the search found no other. The
reflexive entry's log line: announced always, as "reflexive", since a dropped
entry otherwise leaves no trace.

### As landed (2026-10-09)

PR #239, eight commits ahead of the docs.

- **Item 229 and item 103's assert.** `matches_for_effect` asks a resolution's
  filters with `ResolutionContext::effect_source`, at the six sites;
  `object_matches_filter` is a call to `object_matches_filter_for_instance`.
  `targeting::matched` reads a filter's `Err` as no, loud in a debug build, at
  the six sites that swallowed one.
- **Item 225, item 103's frame half, and item 99's first half.** A departure
  record's frame and the frame damage carries are a `DepartedFrame`.
  `SourcePattern.object` is an `ObjectRef`, matched against `gather::dealer`,
  the existence that deals the damage; no move ends it, and `move_object`
  re-points it at CR 400.7c. `oracle::legality::damage_sources` is CR 609.7a's
  one enumeration, offering an existence that has left as
  `ChoiceOption::Departed`, a trigger on its subject's departure naming the
  object that left.
- **The reflexive trigger**, through `register_delayed_trigger` (§3.9, §4.6),
  `Does` and `Doesnt`, with `sacrificed`.
- **Cornered Crook**, registered, not pooled (187 → 188), its one ruling
  linked.
- **The resolving object** (found during the build). CR 608.2 keeps a resolving
  spell on the stack, and its taken entry had kept it, and a resolving
  ability's referents, out of the choice: item 99's last paragraph.
  `ResolvingObject::subject_left_as` carries the departed existence over.

**Tests.** Twenty-one in `phase_tr3c_integration_test`: ATOM-603.12-001 and the
Manticore fixture's four trigger rulings; item 229's "another"; Cornered Crook
cast from hand from exactly {4}{R}, its ruling, killed in response with the
lifelink and deathtouch it last had, and under Circle of Protection: Red; the
"doesn't" form and CR 603.12a; item 225 and its other half on the
Perilous-Myr fixture; item 99's three legs, the told-apart case and the
resolving spell; item 103's token, and the creature whose owner lost as it
died, found at the sitting. Each new behavior's test was shown to
fail without it: item 225's two on the pre-fix tree, the token's on the eager
object read, ten reflexive tests with the check disabled, the Manticore's
five with "another" given no source, and the resolving spell's without its
leg. BOUNDARY-DEF-609.7a-001 is covered whole; ATOM-609.7a-001 stays partial
on the command zone (B2).

**Found.** `codebase-state.md` item 241: the reflexive check reads the arm's
event over the whole resolution, which is "that action" while the resolution
performs it once.

**Size.** +1,626 in code and tests: code +663 (the dev GUI +5), tests +963,
against the gap hunt's 1,250–1,500 and the row's 800–950. The code ran at
the hunt's count; the tests ran over it, at about twice their row, as the
track's re-count said they do.

**Measured** (`fuzz-record.md`, the TR-3c block). The engine arm played every
`performance` game at two seats as `main` did. At four seats two of 200 were
apart, where `IDENTICAL` was predicted. In each, a creature died in the
state-based check that made its owner lose, and Blood Artist's "another
creature dies" filter, asking the store for the departed object, read its
absence as no on `main`. Item 103's frame half answers from the CR 603.10a
frame, and a test pins it. The `stress` games apart, one at two seats and
three at four, are each at Circle of Protection: Red's choice, offered a
source items 99 and 225 now reach. Every dispatch agreed; instructions per
decision +0.17%.

### Review round (2026-10-10)

Eight comments on #239, in three commits ahead of the docs.

- **Comment 7, "how do we know only card-authoring errors reach
  `matched`?"** Nothing said so, and one refusal that is no authoring error
  could reach it after item 103's frame half (a token or owner leaf asked of
  a source the store has lost). The refusals are a type, `FilterRefusal`, and
  every site that read one as no goes through `matched`, the seven outside
  item 103's six among them. Routing them found `And` evaluating its right arm
  after its left said no, and a cost's filter asked with no source:
  "Sacrifice another creature:", on 162 printed cards, offered nothing.
  `matches_for_text_of` asks a filter for the text of an object, for a cost
  and for a resolution's effect alike.
- **Comments 2, 4 and 5**, on the reflexive check: the door is
  `create_delayed_trigger`, CR 603.7a's word, since a function named for
  registering read the past; cards write `when_you_do` and `when_you_dont`,
  and `is_sacrificed`. The owner ruled out "no printed card does it" as a
  reason, which this PR had used twice: the check now reads "that action"
  (item 241, closed) and a record's place in its turn,
  `EventRecord::place_in_turn`, which replaced `TurnOrdinals`.
- **Comments 3, 6 and 8**, on `dealer`, LKI vocabulary and
  `objects_referred_to`: the rule "a trigger on its subject's departure
  names the existence that left" was spelled three times in this PR, which
  `similar_functions.py` could not see (each under its 60-token floor). It is
  `carried_lki`, asked through `StackEntry::lki_of` and
  `ResolvingObject::lki_of`. `damage_source_existence` sits beside
  `damage_source_characteristics`. This PR's names say LKI, and the
  enumeration's leg is `referred_damage_sources`. The older "departed frame"
  names (TR-2b) are a sweep of their own, offered to the owner.
- **Comment 1:** `sacrificed` became `is_sacrificed`, beside `dies` and
  `enters`; the suggested `sacrifice_self` would say the subject is the
  trigger's own source, which it is not.

Three fixtures and tests (Twofold Tithe, Firstlight Lantern, Marrowgnaw Ghoul),
each shown to fail without its change. +575 and −288 in code and tests over
the PR as
it stood. Predicted and read `IDENTICAL` against the pre-review binary on
both pools at two seats and four.

#### TR-4a — the frame's facts and their readers — ✅ landed 2026-10-14

*Evicted 2026-10-14 from `plans/triggers-architecture.md` §12, where the heading and a stub remain, with TR-4's section after it.*

### The plan as sized (TR-4a, as §12 carried it on 2026-10-14)

Split from the 2026-09-24 TR-4a by A6p's re-count (2026-10-12), with TR-4c
its other half. Code at 1.5–2× its sizing plus tests.

| Pieces | Cards, fixtures and atoms | Code (×1.5–2) | Tests | Total |
|---|---|---|---|---|
| `Lki` widened behind its `Arc`, so `GameEvent` keeps its 72 bytes: `Status` with what was attached to the object, the cost decisions, `cast`, the owner (§3.11); `DelayedTrigger.source_frame` an `Lki`; the reader methods; the five live-source leaves and a delayed trigger's "if" read off the frame (§6.1); `TriggeringObjectHadCounters` with its polarity; item 168, with the kept LKI seeded from the frame; items 251 and 252; `ReturnToBattlefield` with entry counters; an amount for "the counters it had" (CR 122.8, 122.9); the dispatch audit's snapshot keeping the host and the status | Strangleroot Geist, Demonic Vigor, Compost (item 251's boards, with a Clone and with Darkest Hour); Kitchen Finks' seven rulings as undying's; ATOM-603.6e-002, ATOM-122.8-001 and ATOM-122.9-001 | ~635 (950–1,270) | ~1,170 | 2,120–2,440 |

### As landed (2026-10-14)

PR #251, two code commits ahead of the docs.

- **The type.** `Lki(Arc<LkiBody>)`: the existence, the frame (the memo's
  `Arc`, shared), the owner, the cost decisions and `Option<PermanentFacts>`
  with the tapped status, counters by kind and attachments both ways. The
  body is private; `object()`, `chars()`, `frame()`, `owner()`,
  `cost_choices()`, `was_tapped()`, `counters(kind)`, `host()` and
  `had_attached(id)` are the surface. `last_known_information` builds it
  before the batch performs; the hand-over clones the pointer. The audit's
  `ObjectSnapshot` holds one, and `DelayedTrigger.source_lki` replaced
  `source_frame`, seeded at creation when the source has already gone.
- **The readers.** `ConditionReferents { source_lki, triggering_object }`
  through `settled_holds`, built by the dispatch
  (`triggers_whose_intervening_if_holds`), the recheck and an effect's own
  "if" (`condition_objects`) and the delayed registry. `Condition::CounterCount
  { object, counter, is }`, which took `SourceHasCounters`' place;
  `AmountExpr::SourceCounters`; `Primitive::ReturnToBattlefield { under,
  counters }`. `owner_of_source` replaced `owner_now_or`, refusing a source
  with no owner anywhere. `HeldFrame::LastKnown` hands the filter table an
  owner with the frame.
- **The items.** 251: `SubjectRead::of_departure` gives an arm the frame
  only when it looks back, and `subject_lki` likewise. 252: a look-back
  `Host` asks `had_attached`. 168: `TriggerCandidate.object` is the
  existence whose list it is, and `MatchedTrigger.source_lki` seeds the
  entry. Each fix was reverted in turn against the TR-4a tests, and each
  failed its fixtures (2, 3 and 1; the five leaves 12; the two owner reads
  1 each).
- **The cards.** Strangleroot Geist pooled; Demonic Vigor and Compost
  registered. `persist()` and `undying()` are authoring words over one
  shape. Kitchen Finks waits for CP-1's hybrid payment.

The sitting is `fuzz-record.md`'s TR-4a block.

### The model pass round (2026-10-14)

§9's model pass read the 22 engine files the branch changed, its report
unseen, and found seventeen facts written twice. Three readings this branch
wrote or exposed already disagreed, and each is fixed with a fixture shown
failing on `5699bc9`:
- **"It" at the trigger was per record, and at resolution per occurrence**,
  so two attackers declared together read no "it" at all.
  `triggers_whose_intervening_if_holds` asks each one, for both matchers.
- **A resolving spell's cost decisions were missed by the capture**, so a
  kicked spell's delayed trigger read it unkicked. `GameState::cost_choices_of`
  reads a permanent, a spell on the stack or the resolving spell.
- **"Its owner" in a gone source's condition panicked**: the `holds` guard
  now lets a held `Lki` through, and `FilterPlayers::owner` expected a row.
  `for_source` takes the owner off the `Lki`.

Two merges in its own code rode with them: `CurrentOrLastKnown` is the one
type for "it", and `lki_kept_by_resolving` the one lookup. The rest are the
duplicate inventory's §3.6, beside A6e (item 247). Against `shipped`, every
row outside timing played identically on both pools at two seats and four.

### Review round (2026-10-14)

The owner's comments were about names, and about two shapes:
- **One counter leaf.** `SourceHasCounters { counter, at_least }` and the
  new `TriggeringObjectHadCounters { counter, had }` became
  `CounterCount { object: ThisObject | TriggeringObject, counter, is:
  CountIs }`: which object and how many, with persist's "had no" as
  `AtMost(0)` and no polarity flag.
- **One condition entry.** `settled_holds_with` wrapped `settled_holds`,
  which wraps `holds`; `settled_holds` takes the referents, and every static
  asker passes `ConditionReferents::NONE` by name.
- **Names.** `ConditionObjects` became `ConditionReferents`, with an
  explicit `NONE` in place of `Default`; `HeldFrame::Chars`,
  `Characteristics`; `Return`, `ReturningObject`; `bound_as_it_was`,
  `triggering_object`; `kept_lki_of`, `lki_kept_by_resolving`;
  `SubjectRead`'s `LookingBack` and `LastKnown`, `LeftInThisEvent` and
  `GoneBeforeThisEvent`: both read last known information, and what sets
  them apart is whether the subject was a permanent at the moment the
  trigger asks about; `IfAsker`, `TriggerSource`;
  `occurrences_whose_if_holds`, `triggers_whose_intervening_if_holds`; and
  `last_known_with` folded into `last_known_information`, which takes the
  frame its caller may read.
- **The tests.** The intervening-if tests are named for their board and
  their answer, and the five leaves' helper says what it builds and checks.
- **A bug the rename exposed.** A damage source that had already left the
  battlefield was matched to an Aura's "enchanted creature" by id, so after
  a flicker the Aura on the new object saw damage the old one dealt (CR
  400.7, 608.2h). An object gone before the event has nothing attached to
  it, and `Host` now answers no for it; the Spirit Link fixture failed
  first, at 23 life against 21.

#### TR-4b — attachment's records — ✅ landed 2026-10-14

*Evicted 2026-10-14 from `plans/triggers-architecture.md` §12, where the heading and a stub remain, with TR-4's section after it.*

### The plan as sized (TR-4b, as §12 carried it on 2026-10-14)

Split by A6p's re-count (2026-10-12). Code at 1.5–2× its sizing plus tests.

| Pieces | Cards, fixtures and atoms | Code (×1.5–2) | Tests | Total |
|---|---|---|---|---|
| `Unattached` with its route (four sites: an unattach, a re-attach, the attachment leaving, the host leaving), `former_host` an `ObjectRef`; the arms `BecomesAttached` and `BecomesUnattached`; item 228; item 178; main item 6's detaches and token sweep as batch members (CR 701.3d, 603.10c, 603.2e; CR 704.5n, 704.5p and 704.5d in CR 704.3's batch) | Grafted Wargear; a fixture shaped like Siona's ruling; ATOM-603.10c-001, -002 and -003, and ATOM-603.2e-002 | 365–505 (550–1,010) | 600–840 | 1,150–1,850 |

TR-4b was under the band's floor at its low end; the seam is CR 701.3's, as
RH-2's was its batch's.

### As landed (2026-10-14)

PR #252, eight code commits ahead of the docs.

- **The record.** `Unattached { attachment: ObjectRef, former_host:
  ObjectRef, route: UnattachRoute }`, the route `AttachedToNothing`,
  `AttachedElsewhere`, `AttachmentLeft` or `HostLeft`, and
  `announce_unattached` its one emitter. A re-attach announces the unattach
  ahead of the `Attached` the arm already emitted, in one batch, which
  ATOM-603.10c-003 needs. The leave performers read the links before
  `move_object` drops them and announce one record per link after their own,
  under the move's cause: the shape `EnterBattlefield` uses for entry
  counters, since the leaving is real without the unattach. When both ends
  leave in one event, the first to move announces it.
- **The state-based members.** `GameAction::Unattach` (loud only for an
  object that does not exist; an attachment the batch moved was unattached
  by leaving, CR 701.3d) and `GameAction::TokenCeasesToExist`, decided
  against the board before the check, so the census's timing case is a test
  and a token that dies in a check ceases to exist at the next. Neither has
  an `EventPattern` arm. CR 704.7's collapse key gained whether the member
  unattaches: on the subject alone, an indestructible animated Equipment with
  lethal damage would keep its destruction, see it refused, and never be
  unattached while the damage stays.
- **Item 228, the host on the entry.** `EnterMods.attached_to`, set by the
  effect putting the Aura there (`propose_entry`'s new `instructed` argument, the returns'
  merge) and attached by the entry's performer after the arrival. CR 303.4h:
  no attach for a permanent that entered as no Aura, Equipment or
  Fortification, or whose object has gone. The CR 614.12 frame does not
  predict it: no replacement reads what an entering Aura enchants.
  `GameAction` grew 112 → 128 bytes with it.
- **Item 178.** `Primitive::Attach` finds its Equipment through
  `this_object`.
- **The arms.** `BecomesAttached { attachment, host }` reads after the event
  (Siona's fourth ruling); `BecomesUnattached { attachment, former_host }`
  reads each end as it was, on the board while it is still that existence
  there and else off its departure's frame in the window
  (`GameState::subject_read_looking_back`). The binding's "that permanent" is the record's
  existence (`subject_after`). An `Attach` or `Unattach` of an attached
  ability-list source takes the look-back snapshot.
- **The cards.** Grafted Wargear (each ruling a test, the first once per
  route) and Bramble Elemental, both registered; Bramble Elemental pooled at
  the review, as §3's one card for the phase's new path.

**Three probes the tests did not pass at first.** A Grizzly Bears card in a
graveyard is still a creature card, so ATOM-603.10c-001's first board passed
with the look-back broken; it now puts the Equipment on an animated
artifact. The Wargear's flicker test passed with `subject_after` broken,
because the dispatch runs at each batch's close and the creature was still
in exile; the reading that matters is a "return that permanent" after the
host died, whose fixture fails with it. And a granted unattach trigger failed
first, at 20 life against 25, until the snapshot reached an unattach.

#### TR-4c — the other two look-back classes and the zone statement — ✅ landed 2026-10-15

*Evicted 2026-10-15 from `plans/triggers-architecture.md` §12, where the heading and a stub remain, with TR-4's section after it.*

### The plan as sized (TR-4c, as §12 carried it on 2026-10-15)

Split by A6p's re-count (2026-10-12). Code at 1.5–2× its sizing plus tests.

| Pieces | Cards, fixtures and atoms | Code (×1.5–2) | Tests | Total |
|---|---|---|---|---|
| the capture for a graveyard departure and for a visible card put into a hand or library, with `looks_back` for both and the hand-over for a non-battlefield mover; item 173; item 249; S1's exemption (§4.2); items 259 and 260 | Guerrilla Tactics, Bridge from Below, Angelic Destiny, Rancor; Guile's two boards under Yixlid Jailer; Golgari Brownscale's fixture, since dredge does not fit `Rewrite` (§15 item 5); ATOM-603.6e-001 and ATOM-400.7f-001 | ~350 (525–700) | ~1,040 | 1,560–1,750 |

### As landed (2026-10-15)

PR #253, six code commits ahead of the docs.

- **The zone statement** (item 173). `trigger_zones` reads the intervening
  "if"'s stated zones (`trigger_stated_zones`), which replace CR 113.6k's
  derivation, and `condition_functions_in` takes the def so each condition
  answers the same (CR 113.6b's "only").
- **The newcomer** (item 259). `match_candidates` finds the objects the window
  made (`object_made_by`: a zone change's new object, a created token), and a
  live candidate among them answers no look-back arm. The brief proposed
  comparing each record with the candidate's arrival; a window is one batch
  and so one event, so the rule takes the whole window. The audit, sharing
  `match_candidates`, already refused the board off its snapshot.
- **The two classes** (item 15). `zone_change_looks_back(from, to)`;
  `capture_lki` frames a graveyard departure, a reanimation's entry, a public
  move into a hand or library and a departing player's graveyard;
  `hand_over_before_leaving` takes the destination; the arm's `looks_back`
  reads the classes as it can match them. Priced first on 200 four-seat
  Commander games a pool: 72 and 97 graveyard departures, 0 and 70 public
  moves into a hand or library, against about 5,000 battlefield departures
  already framed, and 6,545 and 5,403 graveyard cards leaving the game with
  their owner, so the capture is unconditional. A card that reaches a
  graveyard and leaves it in one batch (a creature dying in the check its
  owner loses in) is framed as it leaves; the debug assertion, which caught
  it, now covers battlefield departures only.
- **S1** (`visible_as_read`). CR 603.2f at the instant a list is read: a
  frame's before the event, where it was; any other after it. The brief's
  "where it is now" refuses a creature returned to a hidden hand.
- **A move outside the classes** (item 249). `condition_zones` asks a
  look-back move of the card itself in the zone left and any other where the move can leave
  the object, less the hidden zones. Not the item's look-back from the hand:
  CR 702.29c and Yixlid Jailer's second ruling ask such a trigger where the
  card went, and the Jailer stops Guerrilla Tactics there.
- **CR 400.7f** (item 260). `TriggerBinding.source_found_in_graveyard`, set at
  the dispatch for the same event (`aura_put_into_graveyard_beside_its_host`)
  and by `perform_zone_change` at CR 704.5m's move
  (`aura_put_into_graveyard_by_the_check`), read by `this_object`;
  `TriggerEvent::watches_its_host_leave` is CR 603.6e's arm. The same-event
  half did not already hold.
- **The cards.** Bridge from Below (pooled), Guerrilla Tactics, Angelic
  Destiny and Rancor, each ruling a test. Forced into every `stress` deck at
  two seats, Bridge read 27.1 turns against 26.7 (seed 777) and triggered 24
  times from a graveyard; Angelic Destiny 25.3, Rancor 26.0 and Guerrilla
  Tactics 26.8 against 27.6 (seed 12345). At seed 12345 Bridge's decks reach
  a board no game finishes (item 264), so its reading moved to seed 777.

**Four tests moved with the frame**: RE-7's departing player (the graveyard
card carries one), RC-4b's Containment Priest (its Graveyard → Exile move is
framed), the event log (a card leaving a graveyard prints its type there),
and TR-3c's choose-a-source board, which offers the Myr's graveyard
existence too, as CR 609.7a allows.

**Probes the tests did not pass at first.** The Yixlid Jailer boards passed
before their fixes because nothing triggered at all. Each was shown to bite
on the fixed tree: Bridge's with the look-back snapshot switched off, and
the leaves-your-graveyard card's with its frame read where the card went and
the gate held open, since the gate itself reads the frame and returns first.

### Review round (2026-10-15)

- **Erebos's Titan's third ruling.** "In a multiplayer game, the last
  ability won't trigger when a player with a creature card in their
  graveyard leaves the game." The PR had framed a departing player's
  graveyard for that card, which came from an oracle-text search whose
  rulings were never read. Leaving the game now counts only from the
  battlefield (CR 603.6c), in `zone_change_looks_back` and in the matcher's
  `LeftTheGame` arm, and the graveyard capture is gone. With it gone, no card
  reaches a graveyard and leaves it inside one batch, so `take_captured_lki`'s
  assertion is strict again. The fixture checks the ruling, and failed
  first.
- **Rest in Peace and CR 400.7f.** The CR 704.5m hook checked the cause and
  not the destination, so an Aura exiled instead was found and returned from
  exile. It asks for the graveyard now. Leyline of the Void's board failed
  first; under Rest in Peace the creature itself is exiled and nothing
  triggers.
- **Library of Leng.** The derivation left the hidden zones out of where a
  discard trigger works, which folded CR 603.2f into CR 113.6k;
  `603-2f-complexity.md` had settled visibility as the gate. The hidden zones
  are in, and a Leng-shaped board holds CR 603.2f to the answer.
- **Names.** `AnsweredArms` and `answered_arms`, `newcomer_from`,
  `zones_its_text_states` and `intervening_if_zones` (CR 113.6b as one arm of
  `functioning_zones`), and `when_this_card_moves`; "own move" is gone from
  the docs.
- **Item 264** points at `GameNumber` (`backlog.md` §2.28), the owner's call:
  a symbolic count, since an effect touches all of the tokens, none, or a
  finite few.
- **Measured again** (`fuzz-record.md`, the TR-4c block's review round): the
  engine arm plays every gameplay row as `main` does, the audit agrees, and
  instructions per decision read +0.28%. Shipped's `stress` gate grew with
  the library among Guerrilla Tactics' zones (item 266).
- **Item 266 built**, at the owner's call: an ability whose every condition
  is the object itself moving waits off the battlefield on
  `newcomer_trigger_sources`, asked only as one of the window's newcomers.
  Shipped's `stress` windows past the gate fell from 301.6 to 62.6 a game at
  two seats and from 815.8 to 170.7 at four; instructions per decision
  +0.06% in the third sitting.
- **CR 400.7f's found card stays on `TriggerBinding`.** It is the binding's
  one field written after the ability triggered, so it was moved beside
  `kept_lki` on the pending, stacked and resolving entries, and moved back
  the same day at the owner's second look. The move touched 21 struct
  literals and left three hand-offs that a future path could forget, a copy
  of the ability among them (CR 707.10). On the binding it travels by
  itself, and its doc comment says why it lives there.

#### TR-4d — control, countering, loss, and the records' missing facts — ✅ landed 2026-10-15

*Evicted 2026-10-15 from `plans/triggers-architecture.md` §12, where the heading and a stub remain, with TR-4's section after it.*

### The plan as designed (TR-4d, as §12 carried it on 2026-10-15)

*Re-read against the tree at `7705986` with four rulings the 2026-10-12 sizing
did not have, and sized below. Reviewed before any code; the questions the
owner decided are the last subsection.*

**Where it sits.** TR-4a to TR-4c finished CR 603.10a's zone-change
look-backs: a departure's record carries the frame from before it. The other
three look-backs have nothing like it. A control change has no performer,
since control is the layer walk's output (§3.12 item 13), so nothing proposes
it and nothing records it. A countered spell's record carries no frame (item
253). A loss is read off its player's departures. Items 131 and 258 are the
two facts the zone-change records still drop: a mill that a replacement keeps
in its library leaves no record, and a find does not ask whether a zone is
public.

**Four rulings** (Scryfall, 2026-10-15):
1. **Losing control includes leaving the battlefield.** Khârn the Betrayer's
   first ruling, and Gustha's Scepter's. So "you lose control of" also reads a
   battlefield departure. "An opponent gains control" does not.
2. **An "until end of turn" steal ends in the cleanup step, and its trigger
   lands there.** It goes on the stack, players get priority, and another
   cleanup step follows (Ogre Geargrabber #4, Ray of Command #3, CR 514.3a).
   Six of the nine "lose control of" cards are delayed triggers: Krovikan
   Vampire, Magus of the Unseen, Ogre Geargrabber, Ray of Command, Seraph and
   Stolen Uniform. The other three are Duplicity, Gustha's Scepter and Khârn.
3. **Four seats.** A player who leaves ends every effect that gave them
   control (CR 800.4a), and control going back is a change the sweep
   announces. Their own trigger on it is then kept off the stack (CR 800.4d).
4. **A trigger reads the record; an effect finds the card.** Zellix, Sanity
   Flayer still makes a Horror under Wheel of Sun and Moon (item 131), so the
   new record carries the card as it was milled. CR 701.17c finds a milled
   card only in a public zone (item 258).

And one the re-read found. Zidane, Tantalus Thief's first ruling: a stolen
Zidane triggers its own "whenever an opponent gains control of a permanent
from you", and one taken beside others triggers once for each. So the changed
object is asked off its own frame from before, and each record is one
occurrence.

**What the re-read corrected.** These are defects in the plan, fixed below,
not decisions:
- **The gate misses the change back.** `RegistryScopeSummary` is rebuilt as
  rows come and go. When Act of Treason's row ends at cleanup,
  `any_control_changing` is false, and a sweep gated on it alone never
  announces the return. The gate is `any_control_changing ||
  control_announced_away`. The second is a flag the sweep writes: true while
  some permanent's announced controller is not the one it entered under.
- **The frame before is missing for the first change.** A frame kept only
  while a control row exists holds nothing when the first row is written,
  which is Act of Treason on a board with no steal in progress: the common
  case. With the gate off, the sweep walks nothing, so there is nothing to
  keep. Decision 1 is the fix.
- **A candidate's list from before keeps today's controller.**
  `find_matches` gives a `TriggerCandidateFrame::Before` candidate the live
  controller. The snapshots so far came from batches that move no controller.
  A control change does, and CR 603.10 reads the object "immediately prior to
  the event", so that candidate's "you" (CR 603.3a) is its frame's controller.
- **"That many" is no zone change's.** `TriggerEvent::amount_of` answers
  `None` for a zone-change arm, so item 131's "gain life equal to the number
  milled" refuses today; it does not read 2. A zone-change arm's amount
  becomes one per record: the objects moved, counted.
- **`PlayerLoses` already names two things.** `GameAction::PlayerLoses` and
  `EventPattern::PlayerLoses` (33 hits in `src`). The trigger arm takes the
  same word on a third enum, as `EventPattern` mirrors `GameAction`.
- **ATOM-603.9-001 needs three seats.** At two the loss ends the game (CR
  104.2a), and a game that has ended places nothing.

**The pieces**

1. **The sweep (item 13, CR 603.10d).** `announce_control_changes` compares
   each permanent's effective controller with
   `PermanentState.announced_controller`, in `battlefield_ids_ordered` order,
   behind the gate above.
   - **The record.** Each difference is one `GameEvent::ControlChanged {
     object_id, from, to, lki }`, caused by `GameRule::ControlChanges`, since
     the sweep cannot name the effect (§3.12a). The changes one sweep finds
     are one window, so one event (§4.1), which is Zidane's "for each of those
     permanents".
   - **Its points.** The head of an outermost `execute_actions` outside a
     dispatch, so Act of Treason's change is announced before its "untap that
     creature" (CR 603.4's instant). Each `perform_sba_and_triggers` pass,
     after the state-based actions and before placement. And the cleanup
     step, after CR 514.2's expiry and before CR 514.3a's check, so the change
     back makes the step grant priority. CR 800.4a's ending is announced at
     the pass after the check the player lost in.
   - **Where a permanent starts.** Announced under the controller it entered
     under, read through the layers while a control row exists. So a
     permanent that enters under an effect has made no change.
   - **The frame before.** `PermanentState` keeps the frame each sweep read,
     while a control row exists. It is the walk the comparison already does,
     so keeping it costs an `Arc` clone a permanent. The record's `lki` is
     built from it. The window carries a `LookBackSnapshot` of the changed
     permanents' frames, and of each survivor whose ability list the change
     altered, which the existing `Before` candidate reads. The dispatch
     audit's snapshot is the same frames.
   - **Exact, not "back to the last sweep point".** Control moves only when
     the board does: in a batch, in a row a resolution writes, in a static's
     registration, or in an expiry. A batch's head is a sweep point. The
     other three keep the frames just before they write, while the gate is
     on: a pump and then a steal in one resolution are read with the pump.
     Before the first control row, decision 1.
   - **The scenario save** writes neither field. A saved board is at a
     priority point, where every change has been announced, so the loader
     derives both.
2. **The arm `ControlChanges { subject, from: Option<PlayerRef>, to:
   Option<PlayerRef> }`.** "You lose control of" is `from: Some(You)`; "an
   opponent gains control of a permanent from you" adds `to: Some(Opponent)`,
   read against the same "you". ATOM-603.10d-001's "a permanent you own" is
   the subject's filter. It reads `ControlChanged`, and for an arm with `to:
   None` a battlefield departure whose frame's controller is `from`: a
   `ZoneChange` from the battlefield, or a `LeftTheGame` (ruling 1). It looks
   back. A delayed trigger's "the creature" is `TriggerSubject::Referred`.
3. **`IsCountered { spell: Option<ObjectFilter>, caster: Option<PlayerRef> }`
   (item 253, CR 603.10e).** Reads `SpellCountered`, which gains `lki: Lki`:
   the stack object framed before `change_zone` (`resolve.rs:882`), with
   `Lki.entry`, a shared copy of its `StackEntry` (§3.11), which CV-4's
   `copy_of` reads too.
   - **"A spell you've cast"** reads `StackEntry::caster()`, the controller
     the entry was made with, for a spell that was cast. Nothing writes
     `controller` after the cast: `put_on_stack.rs:205` makes the entry, and
     `:282` writes only `mana_spent`. A spell an effect steals (CR 108.4,
     `Primitive::GainControl` on a spell) changes its effective controller,
     not the field.
   - **No arm on `AbilityCountered`.** No printed trigger reads an ability
     being countered: `(o:"when " or o:"whenever ") o:"ability" o:"is
     countered"` is 2, Green Slime and Tishana's Tidebinder, both "countered
     this way" effects. `SpellFizzled` stays arm-less (Multani's Presence's
     ruling).
4. **`PlayerLoses { player: Option<PlayerRef> }` (CR 603.9, 603.10f).** Reads
   `PlayerLost`, and looks back. The loser's permanents leave in the same
   window, since `player_left_the_game` joins it, so each is asked off its
   departure's frame. One the loser controlled is kept off the stack (CR
   800.4d, §5.3). One the loser owned and another player controlled triggers
   for that player. Nothing leaves the game without losing (CR 104.3a's
   concession is a loss), so the record is the only one to read.
5. **Item 131's record.** `GameEvent::SameZoneMove { object_id, owner, zone,
   act, applied, lki }`, emitted where `perform_zone_change` now returns
   silently for `from == to` (`actions.rs:1919`), when a replacement sent the
   move there. Only an arm that names an act reads it (§3.12a decision 3),
   since a move within a zone is no departure and makes no new object (CR
   400.7). Its `lki` is the card as it was milled, which Zellix's "creature
   cards" reads.
6. **Item 258.** The binding's subject, a remembered object and
   `put_into_since` refuse a hidden zone (CR 400.7e, 400.7j). In the other
   direction, `put_into_since` finds a moved object in whatever public zone
   the move put it in, a replacement's destination included, for every verb
   (CR 400.7j; CR 701.17c says the same of a mill). So a delayed trigger's
   "that card" finds the card Leyline of the Void exiled, and the creature
   Rest in Peace exiled instead of letting it die (decision 2). CR 610.3's
   "until" return reads the same finds; no printed replacement redirects an
   exile (`(o:"would be exiled" or o:"would be put into exile")
   o:instead` is 0), so no game changes.
7. **Cards.** Multani's Presence, whose one ruling is the `SpellFizzled` test.
   Ray of Command, which TR-3a's registry can express: `Effect::Remember`
   around the steal, then `CreateDelayedTrigger` of `ControlChanges {
   subject: Referred, from: Some(You), to: None }` that taps it. So fuzz games
   and `--audit` reach the sweep and the cleanup path, not only a fixture. Its
   stress distortion is measured before it registers.

**Fixtures, each shown failing first**
- A Khârn-shaped "when you lose control of this, draw two cards":
  - stolen by Act of Treason, its owner draws, since the frame before names
    them;
  - when the steal ends at cleanup, the thief draws, in a repeated cleanup
    step that granted priority;
  - when it dies, its controller draws (ruling 1);
  - when the thief loses at four seats, control goes back and the thief's
    trigger is dropped.
- The first change on a board with no control row, with the trigger granted
  by a "creatures you control have" ability of the owner's: it triggers. This
  is decision 1's test.
- Act of Treason's change is announced before its untap.
- ATOM-603.10d-001: an opponent gains control of a permanent you own. A
  Zidane-shaped watcher stolen with another permanent triggers twice.
- Ray of Command: the creature taps in the cleanup step, and taps when a third
  player steals it before then.
- ATOM-603.10e-001 and Multani's Presence: a countered spell triggers; a spell
  that failed on illegal targets does not; an opponent's countered spell does
  not; a spell of yours another player stole, then countered, draws you the
  card and not them.
- ATOM-603.9-001 at three seats. At four, a copy the loser owned and another
  player controlled triggers for that player, and the loser's own is kept off
  the stack.
- Item 131: mill three with a Darksteel Colossus among them, under "gain life
  equal to the number milled", gains 3 (it refuses today). A "mills one or
  more creature cards" trigger with the Colossus as the only creature
  triggers. An arm with no act ignores the record.
- Item 258: a delayed trigger's "that card" finds the milled card Leyline of
  the Void exiled, does not find the Colossus shuffled back, and finds nothing
  in a hand. TR-3b's `an_object_a_replacement_sent_elsewhere_is_not_referred_to`
  flips, and is rewritten as "return that card to the battlefield" so the
  answer shows on the board: under Rest in Peace, the creature comes back from
  exile (Rescue from the Underworld's fourth ruling).
- `the_event_keeps_its_size` stays at 72 bytes.

**Sized**

| Piece | Code (×1.5–2) | Tests |
|---|---|---|
| the sweep, its points, the frames it keeps and the writes that keep them (decision 1), the `Before` controller, the audit's snapshot | ~250 (375–500) | ~330 |
| `ControlChanged`, `ControlChanges` with the departure reading | ~70 (105–140) | (above) |
| `IsCountered`, `Lki.entry`, `StackEntry::caster` | ~90 (135–180) | ~150 |
| `PlayerLoses` | ~25 (40–50) | ~100 |
| item 131's record and the zone-change amount | ~65 (100–130) | ~100 |
| item 258 | ~40 (60–80) | ~90 |
| Multani's Presence, Ray of Command | ~70 (105–140) | (above) |
| **Total** | **~610 (920–1,220)** | **~770** |

1,690–1,990 in code and tests, inside the band. The rulings and Ray of
Command are counted. Projected docs at close-out: about 400. The sweep is
priced on 200 four-seat stress games before its first commit: how many
batches run with a control row, and what each sweep costs under the gate. If
the build crosses 2,500, it splits on the rule seam into the mill records
(items 131 and 258) and the three look-back arms.

**Decisions, taken 2026-10-15**

1. **The frame before the first change: (a), the owner's choice.** Every
   603.10d reader needs the board from just before the change, and the sweep
   sees a change only after
   it. While a control row exists, the sweep walks every permanent anyway, so
   keeping each frame is free and exact. The open case is the first row: with
   none before it, nothing walked, and nothing was kept.
   - **(a), recommended: keep the frames just before the first control row
     is written, at the sites that already keep them while the gate is on.**
     Two places can write the first control row: a resolution's Layer 2 row
     (`add_resolution_effect`, which Act of Treason's `Primitive::GainControl`
     calls) and a printed or copied static's registration
     (`game_state.rs:1900`, `:1978`). A granted static cannot, since layers
     1–5 refuse a grant (`resolve.rs:1771`). So at those two sites the
     condition is "the gate is on, or this row sets a controller". In debug
     builds, the sweep fails on a change it has no frame for, which names a
     missed site. Cost: a walk of each permanent per steal. Maintenance: a new
     site that writes a row needs the call, and the assertion catches one
     that doesn't. Not CLAUDE.md's "reconcile at a chokepoint": that rule is
     about deciding whether an effect exists outside the walk. This decides
     nothing; it keeps last known information before a write, as
     `capture_lki` does before a batch performs.
   - **(b): read the first change off the board now, with "you" taken from
     the record's `from`.** No new site. It is exact for every printed reader:
     the three object readers print their trigger, and the six delayed ones
     have a fixed "you" (CR 603.7d). It is wrong where the change itself moved
     a "you control" grant onto or off the reader, or a "you control" type
     change onto a subject. That is a known wrong answer on a fixture board.
   - **(c): refresh the frames at every sweep point on every board.** Exact,
     with no new site, but it costs a walk of every permanent at every batch
     head, in every game. Not priced; the gate exists because forcing it off
     once cost 28% a game (`RegistryScopeSummary::any_control_changing`).
2. **Item 258's reach: every verb, settled by a ruling.** The question was
   whether a delayed trigger may find an object a replacement sent to
   another public zone, or only a milled one. CR 701.17c and CR 400.7j give
   the same answer (the owner's reading), and Rescue from the Underworld's
   fourth ruling applies it to a delayed trigger: "if the sacrificed
   creature is put into another public zone instead of the graveyard,
   perhaps because it's your commander or because of another replacement
   effect, it will return to the battlefield from the zone it went to". Its
   card was moved by a cost, CR 400.7j's second sentence; an effect's move is
   the first sentence, read the same way. Its commander clause predates CR
   903.9a's state-based move. So TR-3b's
   `an_object_a_replacement_sent_elsewhere_is_not_referred_to` pinned a wrong
   answer, on a fixture whose "exile it" changed nothing visible, and it
   flips. What a card's words ask for still decides: Come Back Wrong's "if a
   creature card is put into a graveyard this way" names the zone, so under
   Rest in Peace it returns nothing.

### As landed (2026-10-15)

PR #254: the design commit, then six code commits.

- **Item 253.** `Primitive::CounterSpell` frames the spell before
  `change_zone`; `SpellCountered.lki` is not optional. `Lki.entry` is kept by
  `last_known_information` for any object framed on the stack, so CV-4's
  copy finds it too. `GameEvent::lki` returns the countered spell's frame, so
  the spell is also a frame candidate, in the stack, as a departure is.
- **The loss.** `PlayerLoses` as designed. A loser's watcher another player
  controlled triggers for that player; the loser's own is kept off the stack
  (CR 800.4d), each a test at four seats.
- **Item 131.** `capture_lki` frames a move a replacement keeps in its zone
  as it frames a departure, and `perform_zone_change` announces it through
  `announce_same_zone_move`. A zone-change arm's `amount_of` is one per
  record.
- **Item 258.** Narrower in one place than designed: the binding's refusal
  applies to a zone-change arm only, CR 400.7e's scope, so a draw trigger's
  card in hand keeps its reading; and `triggering_object`'s characteristics
  read is unchanged. `exiled_since` reads the same finds; no printed
  replacement redirects an exile.
- **Item 13.** As designed, plus four things the build met. A static's
  registration keeps the frames only for the first control row, since inside
  a batch the head kept them. `perform_entry` announces a permanent under the
  controller it arrives with, so a spell stolen on the stack enters with no
  change. The scenario loader settles the announced controllers
  (`settle_announced_controllers`), and four older tests that wrote a steal
  straight into the registry call it, since the sweep otherwise announced a
  change their story never had and found no frame for it. The audit's own
  candidate from a snapshot takes the frame's controller, as the
  dispatcher's now does.
- **The cards.** Multani's Presence and Ray of Command, each ruling a test.
  Forced into every `stress` deck at two seats (200 games, seed 12345), Ray
  read 25.2 turns and Multani's Presence 25.3, against 24.9. Ray of Command
  made 45 control changes and triggered 22 times over 40 such games; Multani's
  Presence saw ten counters in 200 games, none of a spell its controller cast.
- **Priced before the sweep's commit**: 200 four-seat `stress` games read
  every gameplay counter as `main` does; with eight Act of Treasons a deck,
  +2 layer walks a game and +0.9% CPU, and the audit agreed on every dispatch.

**Probes the tests did not pass at first.** Each piece was broken once and
its test failed: the `Before` controller, the gate's second half, the frames
before the first control row, the sweep at a batch's head, the sweep after
the cleanup expiry, and the departure reading. Item 131's and item 258's
tests failed against the tree before them, as did TR-3b's flipped test.

### Review round (2026-10-16)

- **The snapshot holds every permanent's kept frame.** The first build kept
  a frame for each permanent whose controller changed, and for any other
  whose ability list differed across the sweep, which took a second struct
  pairing two frames (`Unchanged`) and a list comparison. The owner found the
  struct strange. Every permanent's kept frame now goes in, which is the same
  answer by construction: a list the change left alone answers the same
  before and after. `a_grant_the_steal_moves_still_triggers_the_creatures_left_behind`
  is the board that needs the survivors, a stolen "creatures you control
  have" source, and failed with only the changed permanents' frames.
- **One function, one name.** `settle_announced_controllers` was a one-line
  wrapper over `sweep_controllers`. It is the sweep now, returning a
  `ControlSweep`, and `announce_control_changes` announces what it returns.
- **The gate's flag** was `control_announced_away`, true while a permanent
  was announced under another controller than it entered under. It is
  `control_effects_at_last_sweep`, whether a control-changing effect existed
  at the last sweep: the same runs of the sweep, said as what it asks.
- **One-sided Humility, both ways** (the owner's request). The thief's makes
  the owner's creature trigger-less before the steal, and it does not
  trigger though it has the trigger after; the owner's takes the trigger
  away after the steal, and it triggers off its list from before. A "lose
  control" trigger refuses the first board on its "you" too, so
  `a_trigger_on_any_change_the_creature_lacked_before_does_not_trigger`
  names no player and fails with the snapshot off.


#### TR-5a — the binding's roles — ✅ landed 2026-10-16

*Evicted 2026-10-16 from `plans/triggers-architecture.md` §12, where the heading and a stub remain, with TR-5e's design and TR-5's section after it.*

### The plan as designed (TR-5a, as §12 carried it on 2026-10-16)

*Designed 2026-10-16 against the tree at `a3577d4`, clause by clause through
Hellrider and Sphere Grid, with their rulings and those of four cards that
share their templates. The count crossed the band, so the phase splits on its
rule seam: TR-5a, the binding's roles, then TR-5e, the attack declaration,
below. Reviewed before any code; decisions 1, 2, 3 and 4 are TR-5a's, at the
end of this section, and 5 to 7 are TR-5e's.*

**Where it sits.** A record reaches the dispatch. The dispatch matches each
occurrence the record holds (CR 603.2c), and the binding keeps what matched
for the resolution, whose "that creature", "it" and "them" read it back
(§3.4, §6.3). Today the binding keeps the records and one `subject`: an
`ObjectRef` for a `PerOccurrence` trigger, `None` for a "one or more" one. Two
of its readers get the wrong object, or none:
- **Damage names two objects, and the binding keeps the wrong one** (item
  255). The arm filters the source, and `subject_of` projects the recipient.
  So Sphere Grid's "put a +1/+1 counter on that creature", after "a creature
  you control deals combat damage to a player", binds nothing.
- **"Them" reads only zone changes** (item 250). `triggering_objects` takes
  each record's `new_object`, which only a zone change has. So "them" over
  damage, an attack or an entry is refused.

**What the re-read found.** Each is a defect in the row as sized, fixed below:
- **A source that left before dealing its damage is not the object its id
  names now.** The record carries `source_lki` when the source dealt damage
  after leaving (CR 608.2h). An `ObjectRef` read off the store at the dispatch
  would take the card in the graveyard, a new object (CR 400.7), for "that
  creature". The occurrence keeps the existence that dealt the damage, which
  is the frame's object.
- **Item 267 has nothing to propose.** Removal from combat is `NOT AN EVENT:`
  (`engineering-practices.md` §11.1, CR 506.4). Regeneration's rider reaches
  `remove_from_combat` through `Primitive::RemoveFromCombat` under that tag, so
  the sweep calls the same function under the same tag.
- **"Party" and "role" are the CR's words** (CR 700.8's party, and the Role
  tokens), so the types below say *participants*.

**The pieces**

1. **The binding keeps occurrences** (items 250 and 255; decision 1):
   ```rust
   pub struct TriggerBinding {
       pub def: Arc<TriggerDef>,
       pub event: EventIndex,
       /// What triggered it (CR 603.2c): one occurrence for a `PerOccurrence`
       /// trigger, every one of its window's for a `OncePerEvent` trigger.
       pub occurrences: Vec<TriggerOccurrence>,
       pub triggered_by: Option<TriggerSeq>,
       pub referred: Referred,
       pub source_found_in_graveyard: Option<ObjectRef>,
   }

   /// One occurrence of the matched event, with the record that holds it,
   /// copied whole (§3.4).
   pub struct TriggerOccurrence {
       pub record: EventRecord,
       pub participants: TriggerParticipants,
   }

   /// The objects one occurrence is about, each by the part it played, as the
   /// existence the dispatch found (CR 400.7): the one fact a record, which
   /// names its objects by id, does not carry.
   pub enum TriggerParticipants {
       /// A step beginning, a life change, a player losing.
       NoObject,
       /// The card a zone change moved, as the object the move made; the
       /// permanent that became tapped; the spell cast; an attach's host.
       Object(ObjectRef),
       /// CR 120.1: the damage's source, as the existence that dealt it, and
       /// what it was dealt to.
       Damage { source: ObjectRef, recipient: ObjectOrPlayer },
       /// CR 508.1: one creature declared as an attacker. TR-5e adds what
       /// it was declared attacking.
       Attack { attacker: ObjectRef },
   }
   ```
   `TriggerEvent` is already the arm enum, CR 603.2's "trigger event" that an
   ability watches for, so one match of it is a `TriggerOccurrence`, CR
   603.2c's word (the owner, at review). The records and their objects are one
   collection, not two kept in step (`engineering-practices.md` §2c).
   `subject` goes. The binding answers "that
   creature" as `triggering_object()`, the one occurrence's, and `None` for a
   "one or more" binding, as today. It answers "them" as
   `triggering_objects()`, each occurrence's. Which participant an
   instruction acts on is the card definition's, never the engine's (decision
   2). Every reader of `subject` or `records` moves to those two or to
   `occurrences`:
   - `bound_object`, `triggering_object`'s CR 608.2h read and `subject_lki`;
   - `ResolvingObject.subject` and the kept LKI's naming;
   - the elision, the delayed registry's CR 603.7b choice and its fold;
   - the audit and the trace.

   `subject_of` becomes the matcher's projection to `TriggerParticipants`,
   exhaustive as it is now, each arm saying which objects its record names.
   Each object is taken where `subject_after` takes it today. A zone change's
   new object and an unattach's ends come off the record, a departed source
   off its frame, and anything else as the dispatch finds it, right after the
   event.
2. **Each instruction names the participant it reads** (item 255; decision
   2). `Participant` names the parts the records have, in the CR's roles:
   `Object`, the one object of a record about one object; `Source` and
   `Recipient` (CR 120.1); `Attacker` (CR 508.1); `Attachment` and `Host` (CR
   701.3). TR-5c adds the blocks' and the targeting's, and TR-5e the
   defender. Every reader of a triggering object takes one:
   `EffectRecipient::TriggeringObject(Participant)` and
   `TriggeringObjects(Participant)`, `AmountExpr::TriggeringPower(..)` and
   `TriggeringToughness(..)`, and `ConditionObject::TriggeringObject(..)`. So
   Sphere Grid's "that creature" is `TriggeringObject(Source)`, and "whenever
   this creature deals damage to a creature, destroy that creature" is
   `TriggeringObject(Recipient)`. Each arm says which participants its
   occurrences have (`TriggerEvent::participants`), and a registry test asks
   each registered card's references against every arm of its condition. A
   participant the record does not have fails that test, not a game. The
   existing readers migrate mechanically, 30 sites in `src` and 19 in
   `tests`. "That player" keeps its one projection per arm in TR-5a, since no
   TR-5a card reads a player with two candidates (item 268).
3. **"Them" over every record** (item 250) is each occurrence's "it", found
   while it is still that object. 23 of item 250's 28 printed cards are
   damage, attack and entry triggers. The damage and entry readers work after
   TR-5a, and the attack readers after TR-5e.
4. **`ObjectFilter::WithCounters { kind: Option<CounterType>, count: CountIs
   }`** (decision 3), on `Condition::CounterCount`'s two axes. `None` is any
   kind: 45 cards say "with a counter on" (`(o:"with a counter on" or o:"with
   one or more counters on" or o:"with counters on")`), beside 227 that name
   +1/+1 (`(o:"with a +1/+1 counter on" or o:"with one or more +1/+1 counters
   on" or o:"with +1/+1 counters on")`). It reads the permanent, or its `Lki`
   for a frame. No layer writes a counter, so it is never a CR 613.8
   dependency.
5. **Item 267: a change of control removes the permanent from combat** (CR
   506.4). The sweep's `announce` calls `remove_from_combat` for each changed
   permanent that is attacking or blocking, before it emits the event, so the
   event's triggers see the board CR 506.4 leaves. A blocked attacker whose
   blocker is stolen stays blocked (CR 509.1h), as `remove_from_combat`
   already keeps it. The scenario loader's settle is left alone, since a
   saved board states its own combat. Shown failing first on item 267's
   board.
6. **Sphere Grid**, registered and pooled. "Unlock Ability" is a flavor word
   (CR 207.2d). Its keywords are a static grant to each creature you control
   matching `WithCounters { kind: Some(PlusOnePlusOne), count: AtLeast(1) }`,
   the shape Wonder's grant takes.

**Its ruling** (Scryfall, 2026-10-16). Sphere Grid #1: a creature that
blocked with the Grid's reach keeps blocking once its counters go or the Grid
leaves (CR 506.4a). Both routes are tests.

**Sized.** Code at 1.5–2× plus tests:

| Piece | Code | Tests |
|---|---|---|
| The occurrences, the projection and the readers moved (pieces 1 and 3) | ~265 | ~210: "them" over damage, an attack and an entry; an object moved again is not found; the migrated reads |
| The participants the instructions name, and the registry test (piece 2) | ~80 | ~200: Sphere Grid's source, the recipient fixture, a departed source finding nothing, "its power" off the source, a participant a record lacks refused |
| `WithCounters` | ~40 | ~60 |
| Item 267 | ~15 | ~110: a stolen attacker, failing first; a stolen blocker, whose attacker stays blocked |
| Sphere Grid | ~40 | ~140: the grant, losing it, and ruling #1 by both routes |
| **Total: 1,380–1,600** | ~440 (660–880) | ~720 |

ATOM-508.2a-001 moves to TR-5e with the attack trigger it tests, so TR-5a
owes no atom (§13). The engine arm should play every gameplay row as `main`
does, except the games where Ray of Command resolves on a creature in combat,
which item 267's fix moves. No registered card reads a damage or attack
binding.

**Decisions, for review**

1. **The binding keeps a list of occurrences, each with its record and its
   participants**, as `TriggerParticipants` above. The alternative, the
   records plus a list of objects beside them, is two collections kept in step
   (`engineering-practices.md` §2c). **Agreed at review (2026-10-16)**, with
   the two types named for triggers.
2. **Each instruction names the participant it reads** (decided at review,
   2026-10-16, reversing a first ruling for the condition). A damage event
   involves a source and a recipient, and "that creature" is the source on
   Sphere Grid and the recipient on "whenever this creature deals damage to a
   creature, destroy that creature". The first ruling put the choice on the
   condition, once per arm. The owner then asked whether the list of such
   choices was complete. Counted from the code rather than from printed
   phrases, it was not:
   - **The 38 record kinds are closed, but the candidates are many.** Every
     object has an owner and a controller, and every record has a cause.
     So "that player" has two or more candidates on almost every record
     about an object, and the printed cards use both. For the dies record
     alone, 126 cards say "its owner" (`(o:"when" or o:"whenever") o:"dies"
     o:"its owner"`) and 25 say "its controller" (`(o:"when" or
     o:"whenever") o:"dies" (o:"its controller" or o:"that creature's
     controller")`). On the condition, that is a field on most arms.
   - **Cards name two participants in one effect.** Frost Titan's kind names
     "that spell" beside its target, and a block trigger names "it" beside
     "the creature it blocks". A custom card can combine any of them, and
     custom cards are a post-v1 goal (the owner, 2026-10-16). The condition's
     one choice would need a second mechanism for the other participant. The
     instruction is one mechanism.
   - **No list of choices then has to be complete.** What must be complete
     is each record's participants, which §3.12a's first question, "which
     object", already asks of every kind.

   What it costs: the four readers each take a participant, the existing
   ones migrate (30 sites in `src`, 19 in `tests`), and a registry test
   checks every named participant against every arm of its condition. There
   is no run-time cost.

   **The engine still reads no English.** Which existence each participant
   is, and whether it can still be found, is rules (CR 400.7, 603.6, 608.2h),
   so the engine keeps every participant. Which one "that creature" names is
   not: CR 608.2c leaves it to "the rules of English". So the card
   definition states it, through the authoring words for each template ("[a
   creature] deals damage" against "[a creature] is dealt damage"). The
   count of "target" is the contrast, since CR 601.2c counts its instances,
   which makes that one the engine's.
3. **`WithCounters` takes an optional kind.** "Any counter" is its own printed
   family, and the filter shares `CountIs` with `Condition::CounterCount`.
   **Agreed at review (2026-10-16).**
4. **The split, and where item 267 goes.** TR-5a plus TR-5e count
   3,070–3,510, against 2,040–2,390 for the row. The row missed the attack
   readings' defender occurrence, `Lki`'s attack, item 128's CR 506.3 checks
   and specified defender, item 267, and "defending player". The brief's
   seam was the binding's roles, then the attack declaration with items 11,
   128 and 267. This design keeps that seam but moves **item 267 into
   TR-5a**: it is reachable and wrong today, and it needs nothing from either
   half. **Item 11 stays in TR-5e**, since every reader of the defender is
   there, and a PR in a split carries a consumer of what it builds
   (`engineering-practices.md` §4). TR-5a is then near the band's floor.
   The alternative is the brief's split as written, which moves about 135
   lines of item 267 into TR-5e. **Agreed at review (2026-10-16).**

#### TR-5e — the attack declaration — ✅ landed 2026-10-16

*Evicted 2026-10-16 from `plans/triggers-architecture.md` §12, where the heading and a stub remain.*

### The plan as designed (TR-5e, as §12 carried it on 2026-10-16)

*Designed with TR-5a on 2026-10-16, built after it. Re-counted against the
tree TR-5a left on 2026-10-16, below, which corrected three planned shapes and
left decisions 8–10. Decisions 8 and 10 were decided at review; decision 9,
rewritten there, proposes splitting TR-5f off.*

**Where it sits.** The declaration (CR 508.1) makes the one record that attack
triggers read. It names the attackers but not what each attacks (item 11).
The arm reads only CR 508.3a's first sentence, and the readers of "the player
it's attacking" and "defending player" have nothing to read. A creature put
onto the battlefield attacking (item 128) cannot be made at all.

**What the re-read found**
- **"The player it's attacking" is read off the attacker at resolution, not
  off the record.** Fathom Fleet Swordjack's ruling: if the attacker leaves
  the battlefield before the trigger resolves, "the player or planeswalker it
  was attacking will be dealt damage". If it "remains on the battlefield but
  leaves combat", nobody is. That is CR 608.2h: the attacker as it is now
  while it is where it was expected, and its last known information once it
  has left. So Hellrider's recipient reads the attacker's own attack, and the
  `Lki` keeps it. The row's "a recipient for the counterpart" is not the
  record's defender.
- **"Defending player" is the defender's printed majority reader** (CR 508.5,
  802.2a; decision 6). 152 cards: `o:"defending player" (o:"whenever"
  o:"attacks")`, against 30 for "it's attacking". Once its creature has left
  combat, it means the player that creature was attacking before.
- **CR 508.3e triggers once for each player attacked.** That is in the rulings
  of Echoing Assault, Horizon Explorer, Long-Range Sensor and Soaring
  Lightbringer. "Attacks [a player] with one or more creatures" triggers once
  for each attacking player (Curse of Chaos, Curse of Inertia).
- **"Attacks alone" is CR 506.5, not 508.3**, and §3.3's row calls seven names
  five shapes. Decision 5 replaces the enum.
- **Extra tokens made "tapped and attacking" are attacking too.** That is
  Parallel Lives' second ruling, and Parallel Lives is registered: the token
  definition a replacement repeats carries the attack. So the defender is
  chosen as each token's entry is proposed, inside the creation and not
  before it. A prompt there is not a fork point (§15 item 13), as a CR 616.1
  prompt is not.
- **Four things put onto the battlefield attacking never attack.** A creature
  under any player but the attacking one (CR 506.3b). A noncreature (506.3a).
  One whose specified defender has gone (508.4a). One outside the combat
  phase, where no player is defending (CR 506.2). One arriving after blockers
  are declared is unblocked (508.4d), which `is_blocked: false` already says.
- **The record names no attacking player.** Each attacker's controller is one,
  and with shared team turns a team attacks (CR 506.2b).

**Re-counted against the tree (2026-10-16)**
- **The record has six readers, not three.** `AttackersDeclared { attackers:
  Vec<ObjectId> }` has one emit site (`combat/steps.rs`). Its readers are the
  display line, the fuzz dump, the turn history (which only counts),
  `objects_named_as_announced` (which hands the attackers out as an
  `&[ObjectId]` slice), a test table in `types/triggers.rs`, and the
  dispatch's `Attacks` arm.
- **The arm is built in five places, not eight**: one in `src` and four in
  tests. No registered card has an attack trigger, so Hellrider is the first.
- **A defender's triggering event fits none of `TriggeringEventObjects`'
  three assumptions.** The type is `Copy`, `objects()` assumes no event names more
  than two objects, and `triggering_objects` reads one object per event
  through `get`. That is 47 references in four files (decision 8).
- **`remove_from_combat` sets `attacking = None`**, which loses the defender
  CR 508.5 reads once the creature has left combat (decision 9).
- **`TokenDef.enters_tapped` has 23 struct literals**: 6 in `src`, 17 in
  tests and none in the dev GUI. Each takes item 128's field.
- **"Defending player" reaches nine `PlayerRef` matches** and all three
  implementations of `FilterReferents`. 48 cards say "target creature
  defending player controls" (`o:"target creature defending player
  controls"`), most of them "whenever this creature attacks, tap target
  creature …". So the word is read by a target's filter at placement (CR
  603.3d), as well as at resolution.

**Planned shapes corrected here, with no decision**
- **"Defending player" names its creature:
  `PlayerRef::DefendingPlayer(ConditionObject)`.** Decision 6 read "its
  source or a trigger's attacker". Those are CR 508.5's two clauses, and at
  four seats they can disagree. Take "whenever a creature you control
  attacks, defending player …" on a source that attacks another player
  itself: the text means the trigger's attacker. So the text states its
  creature, as TR-5a's participants do (CR 608.2c). `ThisObject` is "an
  ability of an attacking creature", such as Robber of the Rich's "if
  defending player has more cards in hand than you". `Triggering(Participant::Attacker)`
  is the trigger's attacker, such as Heart-Piercer Bow's "whenever equipped
  creature attacks, … target creature defending player controls".
  `AttackedBy` takes the same word (item 269).
- **`Participant::Attacked`, the permanent attacked.** It is the "it" of CR
  508.3b's "whenever a planeswalker you control is attacked", and the "that
  planeswalker" after "attacks a planeswalker you control". A player who is
  attacked is no participant.
- **"That player" on the attack arm is item 268's.** `player_of` reads a
  record, and one declaration holds a triggering event for each attacker or
  each defender, each with its own player: the attacker's controller, the player
  attacked, or the controller of the planeswalker attacked. Item 268 makes
  "that player" name its participant, which is what an attack needs. Its slot
  widens to the first registered attack trigger that reads "that player".
  Nothing registered reads `TriggeringPlayer` today, Hellrider included. The
  readings table's last column below is item 268's to build.

**The pieces**
1. **Item 11:** `AttackersDeclared { attackers: Vec<(ObjectId, AttackTarget)>
   }`, at its one emit site and its six readers. `objects_named_as_announced`
   hands out the pairs, as it does `BlockersDeclared`'s.
   `TriggeringEventObjects::Attack` gains `defender: ObjectOrPlayer`, and a
   defender's triggering event is `Defense { defender, attackers }`
   (decision 8).
2. **The arm** (decision 5):
   ```rust
   Attacks {
       attacker: TriggerSubject,
       defender: Option<DefenderFilter>,
       /// CR 603.2c: one triggering event for each attacker, or for each
       /// player, planeswalker or battle attacked.
       per: AttackEventPer,
       /// CR 506.5: the only creature declared as an attacker.
       alone: bool,
       multiplicity: Multiplicity,
   }
   ```
   `DefenderFilter` is `Player(PlayerRef)`, `Permanent(ObjectFilter)` or
   `PlayerOrTheirPlaneswalker(PlayerRef)`. The last is "attacks you or a
   planeswalker you control", 17 cards. Nothing attacks a planeswalker or a
   battle until `backlog.md` §2.23's attack path lands, but the filter reads
   `AttackTarget` whole now. Both kinds of triggering event have the
   participants `Attacker` and `Attacked`: one of each for an attacker's,
   and every attacker of one defender for a defender's.

   **Only a declared attacker counts.** Every reading is matched against
   the declaration's record, which only CR 508.1's declaration writes. A
   creature put onto the battlefield attacking (piece 5) is in no record. So
   it makes no attacker's triggering event, it is none of a defender's
   attackers, and it makes no defender's triggering event of its own (CR
   508.3a, 508.3b, 508.3e, 508.4: such creatures "never 'attacked'"). The
   defender is whatever the record names, a player, a planeswalker or a
   battle alike, and `DefenderFilter` chooses among them.

   The readings, whose "that player" is item 268's (above):

   | Rule | Text | `attacker` | `defender` | `per` | `multiplicity` | "It" · "them" · "that player" |
   |---|---|---|---|---|---|---|
   | 508.3a | "whenever [a creature] attacks" | its filter | — | `Attacker` | `PerOccurrence` | the attacker · — · its controller |
   | 508.3a | "… attacks [a player, planeswalker or battle]" | its filter | `Some` | `Attacker` | `PerOccurrence` | the same |
   | 508.3b | "whenever [a player, planeswalker or battle] is attacked" | `Any` | `Some` | `Defender` | `PerOccurrence` | the attacked permanent · its attackers · the attacked player, or the permanent's controller or protector |
   | 508.3c | "whenever [a player] attacks with [a creature]" | its filter, with the controller | — | `Attacker` | `PerOccurrence` | as 508.3a |
   | 508.3d | "whenever [a player] attacks" | a creature that player controls | — | `Attacker` | `OncePerEvent` | — · the attackers · their controller |
   | 508.3e | "whenever [a player] attacks [another player]" | as 508.3d | `Some(Player(..))` | `Defender` | `PerOccurrence` | — · its attackers · the player attacked |
   | 506.5 | "whenever [a creature] attacks alone" | its filter | — | `Attacker`, `alone` | `PerOccurrence` | as 508.3a |
3. **"The player or planeswalker it's attacking"**:
   `EffectRecipient::AttackedBy(ConditionObject)`. While the attacker is on
   the battlefield it is the attacker's own `AttackingInfo.target`, and
   nothing once it has left combat. Once the attacker has left the
   battlefield it is the attack its `Lki` kept, a new `PermanentFacts.attacking`.
4. **"Defending player"** (decision 6, with its creature named above):
   `PlayerRef::DefendingPlayer(ConditionObject)`. It is the player that
   creature is attacking, or the controller or protector of what it attacks.
   Once the creature has left combat, it is the one it was attacking before
   (CR 508.5). For a creature that left the battlefield while attacking,
   its `Lki` keeps that attack. For one that left combat and stayed, it is
   TR-5f's combat history (decision 9), and nothing until TR-5f lands. Each
   `FilterReferents` finds the creature: a target's filter at placement
   reads the trigger's attacker off the binding. With no creature named, it
   is decision 10's predicate.
5. **Hellrider**, registered and pooled, with its stress distortion
   measured and reported (decision 7).
6. **ATOM-508.2a-001.** A creature turned green after its declaration
   triggers nothing, since the dispatch at the declaration is CR 508.2a's
   instant. It is a test, not code.

**Hellrider's ruling** (Scryfall, 2026-10-16). Hellrider #1: creatures that
attack several players or planeswalkers each deal their 1 damage to their
own. The test uses four seats.

**Sized.** Code at 1.5–2× plus tests, re-counted 2026-10-16 and again at
review. As designed it was 1,690–1,910: code ~445 (670–890) and tests
~1,020. The last column is the split decision 9 made.

| Piece | Code | Tests | Split |
|---|---|---|---|
| Item 11, the record, at its six readers | ~35 | ~20 | TR-5e |
| The arm, `DefenderFilter`, the projections, the authoring words and its five literals | ~145 | ~420: the seven readings, with four seats for 508.3b and 508.3e; ATOM-508.2a-001 | TR-5e |
| A defender's triggering event (decision 8): `Copy` dropped at its by-value sites, `objects()` over the attackers, `triggering_objects` through it, `Participant::Attacked`, and the registry's refusal | ~50 | ~40 | TR-5e |
| "The player it's attacking" and `Lki`'s attack | ~60 | ~170: Hellrider's ruling at four seats, both halves of Swordjack's, and a steal in response | TR-5e |
| Hellrider | ~30 | in the row above | TR-5e |
| "Defending player" while its creature attacks (decision 6), naming its creature, at nine `PlayerRef` matches and three `FilterReferents` | ~85 | ~110: a source and a trigger's attacker that disagree at four seats; a target's filter at placement | TR-5e |
| Decision 10's predicate | ~25 | ~60 | TR-5e |
| Item 128, with CR 508.4, and `TokenDef`'s 23 literals | ~115 | ~315: never "attacked", by 508.3a, 508.3b and 508.3d; the four-seat choice; 508.4d; 506.3a and 506.3b; 508.4a; a specified defender; Parallel Lives' extra tokens | TR-5f |
| Decision 9: the record, the combat's history and its save, and "defending player" once its creature has left combat | ~125 | ~270 | TR-5f |
| **Whole: 2,410–2,745** | ~670 (1,005–1,340) | ~1,405 | |
| **TR-5e: 1,465–1,680** | ~430 (645–860) | ~820 | |
| **TR-5f: 945–1,065** | ~240 (360–480) | ~585 | |

Decisions 8 and 9 are new work. "Defending player" reaches more matches
than the design counted, and the token literals and the record's readers
are more than it counted. Only the arm's literals came in fewer.

**Decisions, for review**

5. **The attack readings are fields, not an enum of seven.** §3.3 planned one
   enum: `Creature`, `CreatureAgainst`, `PlayerIsAttacked`,
   `PlayerAttacksWith`, `PlayerAttacks`, `PlayerAttacksPlayer` and `Alone`.
   But CR 508.3a–e differ in two things only: what one triggering event is
   (an attacker or something attacked), and which of three filters they ask. 508.3d's
   "one or more" is `OncePerEvent`, the axis every other arm already has.
   - **Fields (recommended).** Every combination means something, and there
     is one way to write each trigger.
   - **The seven-variant enum.** It has two spellings of one trigger:
     `Creature` over "a creature you control" against `PlayerAttacksWith(You)`.
     It also allows combinations no rule names, such as `Creature` with a
     defender.

   The table above keeps the seven readings as the tests, each named by its
   rule. **Fields, decided at review (2026-10-16).**
6. **"Defending player" goes in TR-5e** (decided at review, 2026-10-16),
   rather than waiting for its first registered card. In TR-5e it costs about 215 lines and the phase stays
   inside the band. It is the attack declaration's main reader.
   `cr-coverage-audit.md` §3 had planned it on the record. A field on the
   permanent also answers an attacking creature's static or activated
   ability, which CR 508.5 names too and which holds no record.
7. **Hellrider is pooled, and its stress distortion is measured and
   reported** (decided at review, 2026-10-16). The design had proposed a
   benign attack trigger beside it if it distorted `stress`, as Bramble
   Elemental was beside Grafted Wargear in TR-4b. The owner's reasons against:
   Hellrider asks the random agents no choice to get stuck on, and the pool
   has to take on complex effects at some point.
8. **A defender's triggering event holds its attackers** (decided at
   review, 2026-10-16: on the triggering event, as recommended). CR
   508.3b and 508.3e trigger once for each player, planeswalker or battle
   attacked, and "them" is the creatures declared attacking it. Today one
   triggering event holds at most two objects, by value, and each is fixed
   as the existence the dispatch found (TR-5a). Only declared attackers are
   among them, as the pieces say: a creature put onto the battlefield
   attacking is in no record.
   - **Hold them on the triggering event (recommended).**
     `Defense { defender: ObjectOrPlayer, attackers: Vec<ObjectRef> }`. The
     type stops being `Copy`. `objects()` chains the attackers after its
     fixed pair, so it still allocates nothing. `triggering_objects` reads
     `objects()` rather than `get`, and `get` answers only a participant
     that is one object. So the registry test refuses a reader of one object
     ("its power", an intervening "if"'s "it") of `Attacker` on an arm whose
     events are per defender. Of the 47 references, most build a variant and
     do not move. The ones that do are the by-value matches, and the
     trigger-time view, which holds the event's objects by value and would
     hold them by reference. The `Vec` allocates only for a defender's
     triggering event, and no registered card has one: Hellrider's are per
     attacker. So the clone counters stay where they are, and the type's
     size is the one thing to read at close-out.
   - **Read the attackers off the record at resolution.** The type stays
     `Copy` and nothing above moves. But the record names its attackers by
     `ObjectId`, so a creature that left and came back (CR 400.7) would be
     found as one of "them". That gives up TR-5a's rule that every object is
     fixed when the trigger matches, and adds a second reader for "them".
9. **Removal from combat is a record, and the combat's history keeps what
   each creature did** (CR 506.4, 508.5; rewritten twice at review, then
   decided with the split, 2026-10-16; built in TR-5f).

   **Why anything is kept.** At two seats nothing is: during combat the
   defending player is the nonactive player (CR 506.2), whatever the
   creature does. With more seats every opponent of the attacking player is
   a defending player (CR 802.2), and "defending player" means the one the
   creature is attacking (CR 508.5, 802.2a), which `AttackingInfo.target`
   says. CR 508.5's second sentence covers a reader that outlives the
   attack: "If that creature is no longer attacking, the defending player
   it's referring to is the player that creature was attacking before it
   was removed from combat". The reader is nearly always a trigger on the
   stack, such as "whenever this creature attacks, defending player loses 2
   life", resolving after its creature stopped attacking. The creature may
   have left the battlefield, killed in response. Or it may have stayed and
   left combat: stolen in response (Ray of Command and Act of Treason are
   registered, and since item 267 a change of control removes the creature
   from combat), regenerated, or removed by an effect (25 cards,
   `o:"remove" o:"from combat"`). Then `attacking` is `None`, and at four
   seats "defending player" has three candidates and nothing to choose by.

   **What this is.** The fixes proposed first each patched CR 508.5's one
   sentence with a store of its own. But what the sentence reads is a fact
   about a creature's part in this combat that outlives its attack, and the
   engine already keeps such facts one way. The stream's records say what
   happened, and the histories (§3.10) materialize what a card later asks
   about the past, advanced record by record by one writer
   (`advance_history`). Removal from combat is the one change to an attack
   that writes no record. RB ruled CR 506.4's consequence "NOT AN EVENT"
   (`replacement-architecture.md` §8a), which is about a replaceable
   proposal: nothing replaces it or forbids it. That ruling does not stop
   it being announced, as `AttackersDeclared` is announced with no
   `GameAction` behind it.
   - **The record:** `GameEvent::RemovedFromCombat { object: ObjectRef,
     attacking: Option<AttackTarget>, blocking: Vec<ObjectId> }`. Each
     caller of `remove_from_combat` announces it after performing the
     removal (one performer, one emitter): a departure, a change of control
     (item 267), and an effect, regeneration's rider among them. It names
     what the creature was attacking or blocking at that moment. That is the
     fact CR 508.5 reads, whatever the declaration said or a reselection
     (Portal Mage, `o:"reselect"`) changed since. No trigger reads it
     (`o:"removed from combat" o:"whenever"` is 0), so
     `EventKind::from_record` answers none. The engine performs no other
     cause of CR 506.4 today. Phasing (CR 702.26) is not built, and a type
     change that ends a creature's creature-ness removes nothing yet. Each
     announces as it is built.
   - **The combat's history**: a list on `state::history` beside the turn's
     counts, advanced by the same writer from two records. Each declared
     attacker comes with its target (`AttackersDeclared`, item 11), and each
     removal with what it was attacking (`RemovedFromCombat`). Each row
     names its existence (CR 400.7) and which combat of the turn it was in,
     and the list is cleared as the next turn begins, with the counts.
     "Defending player" for a creature no longer attacking reads its last
     removal row in this combat. That is one reader for every way a creature
     leaves combat, and for any kind of trigger: TR-5c's "whenever this
     creature becomes blocked, defending player discards a card" (15 cards)
     reads the same rows, so its block event needs no defender of its own.
     The rows also answer, with no store of their own, what nothing answers
     today: "creatures that attacked this turn" and "attacked this combat"
     (22 cards, Relentless Assault's "untap all creatures that attacked this
     turn" among them), and CR 508.4's "never attacked", since only a
     declaration writes an attack row. TR-5e builds none of those readers.
   - **Its cost.** The list allocates once in a turn with an attack, so a
     clone taken during that turn makes one more allocation, against floor
     2's 39 of 64. The save writes each row as a word, as it writes the
     turn's counts. `attacking: None` keeps its one meaning for its 23
     readers. About 125 lines of code (190–250) and 270 of tests: the record
     at each of its three causes, the rows across two combats in one turn
     (RE-10's additional combat), the former defender after a steal, an
     effect and a death at four seats, a creature put onto the battlefield
     attacking writing no attack row, and the save's round trip.
   - The `Lki` still keeps the attack, for "the player it's attacking" (CR
     608.2h, Swordjack's ruling). That reader asks about the creature as it
     last existed, and this one about the combat.

   **Set aside:** the narrower stores proposed first. A combat-scoped map
   written by `remove_from_combat`, or a field on each permanent, would
   answer CR 508.5 alone for about 10 lines and leave removal from combat
   unannounced. Keeping nothing and reading the trigger's own event fails
   after a reselection and on a block trigger, which are the two gaps the
   record and the history close. A flag inside `AttackingInfo` must be
   checked by each of the 23 readers.

   **It split TR-5e** (decided at review, 2026-10-16). Built whole, the
   phase was 2,410–2,745, past the band. The seam is CR 508.4: an attack that was never declared,
   and the end of one. **TR-5e, the attack declaration** (CR 508.1–508.3,
   506.5, 508.5's first sentence): item 11, the arm and decision 8, "the
   player it's attacking" with `Lki`'s attack, "defending player" while its
   creature attacks (decisions 6 and 10), Hellrider and ATOM-508.2a-001, at
   1,465–1,680. **TR-5f, creatures put onto the battlefield attacking, and
   removal from combat** (CR 506.3, 506.4, 508.4, 508.5's second sentence):
   item 128, this decision's record and history, and "defending player"
   once its creature has left combat, at 945–1,065. TR-5f is below the band
   and comes next, before TR-5b, since TR-5c's block triggers read its rows.
   Until it lands, "defending player" for a creature no longer attacking
   reads nothing. No registered card reads "defending player", Hellrider
   included.
10. **CR 802.2a's last sentence, the "defending player" no creature settles**
    (revised at review, then decided, 2026-10-16: the predicate in TR-5e,
    as recommended). "If there are multiple
    defending players that could be chosen, the controller of the spell or
    ability chooses one." Decision 6 covers the readings tied to a creature,
    and the corrected shape above names that creature. What is left is text
    that names none. Five cards among `o:"defending player" (t:instant or
    t:sorcery)`'s 7 and `o:"defending player" o:"activate only"`'s 6 do so:
    Yare, Blaze of Glory and False Orders ("target creature defending player
    controls" on a spell), and Arcum's Sleigh and Kjeldoran Guard ("activate
    only during combat and only if defending player controls …"). The others
    tie it to a creature or say "each defending player".

    **For all five, what the controller picks settles the choice** (the
    owner, at review). CR 802.2 makes every opponent of the attacking player
    a defending player for the whole combat phase, from its start. Arcum's
    Sleigh can be activated in any combat step, the beginning of combat
    included, before anything attacks. Its controller picks a defending
    player who controls a snow land, so it can be activated when any of them
    does. On another player's turn its controller may be a defending player
    too. Yare's target picks its player. So for these five, "defending
    player" is a predicate, "is a defending player", as `PlayerRef::Opponent`
    already is in a filter (CR 102.3). That needs no prompt, no answer kept,
    and nothing in the cast. The choice is real only where nothing the
    controller picks settles it: the same player read twice (two targets, or
    a target and then "that player loses 2 life"), every creature that player
    controls, a condition in an effect, or the player acted on. No printed
    card that names no creature does any of these.
    - **Build the predicate in TR-5e (recommended).**
      `PlayerRef::AnyDefendingPlayer`: in one target's filter or an
      activation's restriction, any of the attacking player's opponents
      during combat (CR 506.2, 802.2), and nobody outside combat. It has an
      arm at each of the nine matches. As the player an action acts on, with
      two or more candidates, it is an authoring error and loud, as
      `Opponent` is in `resolve_player_ref`. The kept choice for the shapes
      it does not settle is a debt item slotted to RS-2, whose cast,
      activation and target checks it would be asked in. About 25 lines of
      code (40–50) and 60 of tests: Arcum's Sleigh's restriction at four
      seats in the beginning of combat step, and a Yare-shaped target.
    - **Build the kept choice too.** It is asked as the stack object is made
      and kept on its entry, at 19 struct literals. That is about 90 lines of
      code and 150 of tests on top, which takes TR-5e past the band.
    - **All of it to RS-2**, as first recommended, and nothing in TR-5e.

### As landed (2026-10-16)

**Built as designed**, with decisions 5–10 as review decided them and TR-5f
split off. Five commits: item 11's record, the arm with decision 8, "the
player or planeswalker it's attacking" with `Lki`'s attack and Hellrider's
card, "defending player", and Hellrider's registration, then a clippy fix.

**Three corrections the build found.**
- **"Defending player" is read through three player vocabularies, not one.**
  Decision 6 planned `PlayerRef::DefendingPlayer`. Of the 155 printed attack
  triggers that say it (`o:"defending player" o:"whenever" o:"attacks"`), 70
  read it in a filter ("creature defending player controls"), which is
  `PlayerRef`; 63 act on the player ("defending player loses 2 life"), which
  no `PlayerRef` reaches; and 12 ask a condition ("if defending player"),
  whose `PlayerSet` answers without a board by design. So it is also
  `EffectRecipient::DefendingPlayer` and `Condition::DefendingPlayer`, each
  naming its creature. A target's filter reads the trigger's attacker off the
  binding, at placement and at CR 608.2b's re-check, through the text's named
  objects that `FilterIdentity` now carries. `player_ref_names` and
  `SourceFilter::matches` take the game, which the predicate and the
  creature's attack need.
- **A player reader that finds nobody skips its instruction.**
  `resolve_player_for_self` defaults to the controller, so the first draft of
  the test made player 0 lose the life. `AttackedBy` and `DefendingPlayer`
  return before the primitive when they find nothing. "That player" has the
  same fallback, and item 268 records it.
- **`ActivationRestriction` has no condition form**, so "activate only if
  defending player controls …" cannot be written at all. Decision 10's
  predicate is read in a target's filter, and item 271 carries the
  restriction half to RS-2.

**What it leaves.** Item 270: "defending player" for a creature that left
combat and stayed names nobody until TR-5f. Item 271: CR 802.2a's choice
where nothing the controller picks settles it, and the activation's
condition, both RS-2's. Item 11 stays open for its blockers half, TR-5c's.
Item 128 is TR-5f's.

**Size.** +1,286 in code and tests (code +727, tests +559), against
1,465–1,680. Seventeen tests in `phase_tr5e_integration_test`, each new
reading shown to bite by breaking the engine and restoring it.

### Review round (2026-10-16)

The owner's code review of #256, twelve comments.
- **Names.** The check that refuses "its power" over a defender's
  attackers reads in its own words now: `BoundReads::needs_one` (the
  instruction needs exactly one object in that part),
  `participants_needing_one`, and `TriggerEvent::may_have_several`.
  `ConditionReferents::object` is `object_for`, with a doc that says what
  comes back. Two older names were renamed at the owner's choice:
  `Participant` is `EventRole`, and `ConditionObject` is `Antecedent`
  (`TextObject` was offered and declined as no clearer). Its `Triggering`
  variant keeps its name: renamed `InEvent` for a commit, it dropped the word
  `EffectRecipient::Triggering` and `TriggeringEvent` use for the same event. Prose says "event role", never a bare "role", which the Role
  tokens own (CR 111.10); the glossary's entries follow.
- **`GameEvent::for_each_object_named`** replaces
  `objects_named_as_announced`. That function packed each variant into three
  slots and chained them, and TR-5e's attackers had been bolted on as a
  fourth with a second dispatch on the variant. Each arm now calls `each` on
  its own objects, with no allocation on an event's emission.
- **`DefenderFilter::Or`** replaces `PlayerOrTheirPlaneswalker`. One
  combinator answers every "X or Y" defender phrase, and the authoring word
  `player_or_their_planeswalker` keeps the 17-card phrase readable. The
  "attacks you" test runs both spellings.
- **The "defending player" test** is two tests, one for each of CR 508.5's
  clauses, over a shared board.
- **Answered without a change:** `alone` is the card's question and is
  answered off the declaration's record, since CR 506.5 counts the creatures
  declared; "one or more" attack triggers are `multiplicity`, independent of
  `per`; `BoundReads` is a `u32` because the needs-one bits took it past 16.
  The "attacks alone" tests beside a creature put onto the battlefield
  attacking need item 128, so they are TR-5f's pieces.
