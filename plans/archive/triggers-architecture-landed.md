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

#### TR-5f — creatures put onto the battlefield attacking, and removal from combat — ✅ landed 2026-10-17

*Evicted 2026-10-17 from `plans/triggers-architecture.md` §12, where the heading and a stub remain.*

### The plan as designed (TR-5f, as §12 carried it on 2026-10-17)
*Split from TR-5e at its review (2026-10-16, decision 9). Designed
2026-10-16 against the tree at `1e20410`, and reviewed four times on
2026-10-17 (the owner, on the design PR): decisions 11–16 decided, at the
end of this section. Nothing built.*

**Where it sits.** TR-5e's arm reads only the declaration (CR 508.1). Two
ways a creature's part in combat changes write nothing. A creature put onto
the battlefield attacking (CR 508.4, item 128) cannot be made at all. A
creature removed from combat (CR 506.4, 511.3) writes no record, so
"defending player" for a creature that left combat but not the battlefield
reads nothing (CR 508.5's second sentence, item 270), and nothing answers
"creatures that attacked this turn". TR-5c's block triggers read what this
phase keeps.

**What the re-read found**
- **CR 511.3 is a removal from combat too.** CR 508.5's "before it was
  removed from combat" makes no exception for it, and decision 9 counted
  only CR 506.4's causes. A reader after combat exists in the rules: a
  delayed trigger that an attack trigger creates has the attacker as its
  source (CR 603.7e), so it is "an ability of an attacking creature" when it
  resolves in the end step. No printed card reads one. Of the 8 cards that
  say "defending player" beside "end step" or "next turn" (`o:"defending
  player" o:"end step"`, 6; `o:"defending player" o:"next turn"`, 2;
  Scryfall, 2026-10-16), none reads it once combat is over. Decision 12.
- **CR 508.8 has a second leg the engine does not read.** The steps are
  skipped only if no creature is "declared as attackers or put onto the
  battlefield attacking". `attacks_declared` is the declaration's flag, so a
  token put onto the battlefield attacking in the beginning of combat step,
  with nothing declared after it, would have no blockers step and deal no
  damage.
- **Removing several creatures at once must read the board once.**
  `remove_from_combat` edits the other creatures' lists: removing an
  attacker takes it out of its blockers' `blocking`. Removed one at a time,
  a blocker removed after its attacker would report blocking nothing. CR
  511.3's removal is of every creature at once, and an effect can remove
  several (`Primitive::RemoveFromCombat` over a filter). So each member's
  part is read before any member is removed, as a batch decides each
  member against one board before it performs (CR 704.3).
- **Divine Visitation keeps the attack.** Its first ruling: "anything else
  specified in the effect creating the token (such as tapped, attacking …)
  still applies". The pipeline's `Replace` arm carries `enters_tapped` across
  by hand (`pipeline.rs`, the `TokenSubstitution::Replace` arm), so the new
  field joins it there. Parallel Lives' extra tokens repeat the definition,
  so they carry it with no edit (its second ruling).
- **The choice is not a `SelectRecipients`.** That prompt carries an
  `EffectRecipient`, which item 141's rule keeps out of a payload, and this
  choice has none. It is `ChooseEnteringController`'s shape: an entry's
  choice, one object, players for options, asked only with two or more.
- **What a printed card specifies is one of three things** (the 13 of
  `o:"tapped and attacking that"`, read 2026-10-17). A player the
  instruction names: the player a "for each opponent" names (Adeline,
  Stampede Surfer, four more), or the player its trigger attacked (Soaring
  Lightbringer, Kaalia, five more). That player or a planeswalker they
  control, the controller choosing (Adeline, Zara). Or ninjutsu's "the same
  player, planeswalker, or battle as the creature that was returned" (CR
  702.49c). Each is read as the effect resolves (CR 508.4a).
- **Every way onto the battlefield goes through one entry.** Of the 118
  cards that say "tapped and attacking" (`o:"tapped and attacking"`), 76
  create tokens and 42 put a card onto the battlefield; 16 of those return
  one (`o:"return" o:"to the battlefield tapped and attacking"`). A token's
  creation and `Primitive::ReturnToBattlefield` are two instructions, and
  both propose the same `GameAction::EnterBattlefield`, which the batch
  decides and `perform_entry` performs. So the attack is a word each
  instruction writes into that entry's `EnterMods`, decided and performed
  once, in the chokepoint, for every instruction: the two built here, and
  ninjutsu's 37 (`keyword:ninjutsu`) and Yuriko's commander ninjutsu (CR
  702.49d, which also functions in the command zone) when
  `permission-architecture.md` builds abilities that work in a hand or the
  command zone. Phoenix Chick, Warcry Phoenix and Persistent Marshstalker
  return themselves with `ThisObject`; the rest of the 42 choose a card in a
  hand or a graveyard, which no `SelectionFilter` reaches, and the attack is
  not what they wait for.
- **The type-change cause is reached today, and wrong** (a probe,
  2026-10-17: 200 games a table, seed 12345). A combatant that is no longer
  a creature appeared in 1 of 200 four-seat `performance` games and 9 of 200
  four-seat `stress` games, and in `stress` at two seats. Three causes, all
  registered cards, and in each the creature stayed "attacking":
  - A player who lost in the combat damage step left the game (CR 800.4a)
    with their March of the Machines, and the attacking player's animated
    Bonesplitter was an Equipment again (`performance` seed 12525, turn 50).
  - Two Opalescences that animated each other died blocking each other, and
    the third they had animated was an enchantment again (`stress` seed
    12439, turn 44).
  - Mirrorweave made every other creature a copy of an animated land, and
    five attackers were lands (`stress` seed 12497, turn 73).

  Decision 13.
- **CR 508.4b, "an effect states that a creature is attacking"**, has no
  printed card by two searches (`o:"becomes attacking"`, 0; `o:"is
  attacking" (o:"target creature" or o:"that creature")`, 13, each a
  condition). The entry's checks are one function, which such an effect's
  performer calls when one is written.

**The pieces**
1. **Item 128, with CR 508.4: the attack is the entry's.** What the
   instruction says, on `TokenDef.enters_attacking` (23 literals) and
   `Primitive::ReturnToBattlefield`'s new `attacking` (12 literals):
   ```rust
   pub enum EntersAttacking {
       /// CR 508.4: its controller chooses among everything that can be
       /// attacked.
       Chosen,
       /// "Attacking that player": a player the instruction names, in the
       /// vocabulary its other player instructions use, and `among`, what
       /// of theirs it may attack, which the controller chooses between.
       Player { player: EffectRecipient, among: DefenderFilter },
       /// CR 702.49c: what another creature is attacking, or was as it left.
       SameAs(Antecedent),
   }
   ```
   `among` is TR-5e's `DefenderFilter`, read over the three kinds CR 508.4
   lists for one player: the player, a planeswalker they control, a battle
   they protect. "That player" is `Player(None)`; Adeline's and Zara's "that
   player or a planeswalker they control" is `Or` of it and a planeswalker
   filter; "that player, a planeswalker they control, or a battle they
   protect" is the `Or` of all three, and a filter can narrow a kind further
   (review, 2026-10-17: no kind is keyed to the printed cards). The
   resolution reads the word, since only the resolution holds the binding
   that says who "that player" is, and writes what the creature may attack
   into the entry, `EnterMods.attacking: Option<EntryAttack>`, with
   `EntryAttack { candidates: Vec<AttackTarget> }`. `Chosen` gives every
   candidate, `Player` the named player's that `among` matches, and `SameAs`
   one. With none, because the player has left or is no defending player,
   the word is dropped and the card or token still enters (CR 506.3c,
   508.4a), and the performer checks it again as it enters. Then the one
   door does the rest, for every instruction alike:
   - **The batch decides it.** After CR 616.1 settles each entry in the
     batch's decide phase, so its controller is known, the batch asks that
     controller `ChoiceKind::ChooseEnteringDefender { object }` for each
     entry with two or more candidates, in the batch's APNAP order (CR
     101.4), and takes the one when there is one. It asks only when the
     creature could attack: during combat (CR 506.2), for an attacking
     player (506.3b). Everything that can be attacked is the declaration's
     own list (`process_declare_attackers`'s, today the opponents, CR
     802.2), so `backlog.md` §2.23's planeswalkers and battles widen both at
     once. Asked per entry, so each of Parallel
     Lives' copies is asked (Hanweir Garrison's first ruling). At two seats
     nothing is asked, so no fixture moves.
   - **`perform_entry` performs it.** It writes `AttackingInfo` after the
     permanent exists, when CR 506.2, 506.3a (a creature, by its frame),
     506.3b (its effective controller is an attacking player), 506.3c and
     506.3f (no battle) allow, and it is the only writer. Nothing consults
     the declaration's requirements or restrictions (CR 508.4c). One
     arriving after blockers are declared is unblocked (506.3d, 508.4d),
     which `is_blocked: false` with no declaration to come already says.
   There is no declaration record, so no attack trigger, no `Attacked` row,
   and no part in "attacks alone" (CR 508.3a, 508.3b, 508.4, 506.5).
2. **CR 508.8's second leg.** `perform_entry`, as it writes `AttackingInfo`,
   sets the flag the steps read, which is renamed for what it now means:
   `attacks_declared` → `combat_had_attackers` (17 sites).
3. **Removal from combat is a record, one for each removal** (decision 9,
   with decision 12): `GameEvent::RemovedFromCombat { removed:
   Vec<CombatExit> }`, with `CombatExit { object: ObjectRef, attacking:
   Option<AttackTarget>, blocking: Vec<ObjectId> }`, what each was attacking
   or blocking as it was removed. One removal, one record, as one
   declaration is one `AttackersDeclared`: the end of combat removing six
   creatures is one record, and no log line (decision 12). `remove_from_combat` performs
   and announces nothing, as now; each caller reads every member's part
   first, removes them all, then announces.

   | Cause | Caller | Cause on the record | Window |
   |---|---|---|---|
   | An effect, regeneration's rider among them (CR 506.4, 701.19a) | `Primitive::RemoveFromCombat` | the resolution's | the resolution's |
   | A change of control | piece 5's settlement, as now | `GameRule::ControlChanges` | the change's, after its `ControlChanged` |
   | Stops being a creature | piece 5's settlement | a new `GameRule` | its own |
   | The end of combat (CR 511.3; decision 12) | `turns.rs`'s phase end | a new `GameRule::CombatEnds` | its own |
   | Leaving the battlefield | — | — | its zone change, whose frame keeps the attack (TR-5e) |

   A departure writes no second record: its zone change says it, and the
   departed existence's "defending player" is read off that frame.
   `EventKind::from_record` answers none (`o:"removed from combat"
   o:"whenever"` is 0), and `for_each_object_named` names each object.
4. **The combat's history** (decision 11): what each existence did in this
   round's combats, for its readers below, advanced by `advance_history`
   from three records.
   ```rust
   pub struct CombatHistory {
       /// The combat phases begun this turn (CR 500.8 adds them), counted
       /// off `PhaseBegin`.
       combats_this_turn: u32,
       /// Shared, so a clone copies a pointer (`Arc::make_mut` on write).
       rows: Arc<Vec<CombatRow>>,
   }
   pub struct CombatRow {
       object: ObjectRef,
       turn: u32,
       combat: u32,
       fact: CombatFact,
   }
   pub enum CombatFact {
       /// CR 508.1: declared an attacker, attacking this.
       Attacked(AttackTarget),
       /// CR 506.4, 511.3: removed from combat, attacking this as it was.
       Removed { attacking: Option<AttackTarget> },
   }
   ```
   Each field is a reader's:

   | Field | What reads it | Printed readers (Scryfall, 2026-10-17) |
   |---|---|---|
   | `object`, an existence (CR 400.7) | every reader: a creature that left and came back never attacked, and a creature stolen and then killed is read through its frame's existence | — |
   | `turn` | "attacked this turn"; "attacked during your last turn" | `o:"attacked this turn"` 83, of which 53 are "you attacked this turn", the player's count `TurnFact::AttackersDeclared` already keeps; `o:"creature(s) that attacked this turn"` 11 (Relentless Assault); `o:"attacked during"` 4 (Giant Turtle, Goblin Rock Sled, Halls of Mist, Tangle Kelp) |
   | `combat` | "attacked this combat"; CR 508.5 over the latest combat | `o:"attacked this combat"` 4 (Custodi Soulcaller, Kytheon) |
   | `Attacked`'s target | "attacked you this turn" | `o:"attacked you"` 7 (Jabari's Influence) |
   | `Removed`'s target | "defending player" once its creature has left combat (item 270) | — |

   **What it does not hold yet, and who adds it.** TR-5c adds a block row
   from `BlockersDeclared`: "attacked or blocked this turn" (4, Lurker),
   "attacked or blocked this combat" (`o:"blocked this combat"` 14, the
   Clockwork creatures), "blocked or was blocked this turn" (Heat Stroke,
   Fyndhorn Druid). A creature put onto the battlefield attacking has no row
   of its own: CR 508.4 makes it never "attacked", and its removal row
   answers CR 508.5. **The rows outlive the turn by one round**, kept from
   the earliest turn any player's "your last turn" still names, so the four
   "attacked during your last turn" cards can be written on it; no reader of
   that span is built here. Only the declaration writes an `Attacked` row.
5. **One settlement for what the layers' output changes** (decision 13).
   Something performed starts or ends a continuous effect, and what that
   does to another permanent's controller or types is the layer walk's
   answer afterwards, not an event of its own (CR 613.1b, 613.1d). The CR
   makes two consequences of that immediate today: the change of control is
   an event (CR 603.10d, TR-4d) that removes the permanent from combat, and
   a combatant that stops being a creature, or becomes a battle, leaves
   combat (CR 506.4). `announce_control_changes` becomes one function that
   settles both, in that order, at TR-4d's three points (the head of an
   outermost batch, each pass of the state-based check, the cleanup step)
   and at the close of each outermost batch, before its window is
   dispatched (decision 13). Each part has its own gate: control as now;
   combat membership only while `combat_had_attackers` and only when the
   layer epoch has moved since its last pass, since a frame cannot change
   otherwise. `backlog.md` §2.23's checks, an attacked planeswalker or
   battle that stops being one (CR 506.4, 506.4c–e), join the combat part.
6. **"Defending player" once its creature has left combat** (item 270).
   `CurrentOrLastKnown::defending_player` reads the creature's attack, now or
   in its frame, as today; with none it reads the history's last `Removed`
   row for the existence in the latest combat. The existence is the frame's
   for a creature that left after it was removed, so a creature stolen and
   then killed in response still names its former defender. One read site,
   which all three of TR-5e's player vocabularies go through.
7. **The save reports the history** (decision 14): "what creatures did in
   combat this turn (CR 508.4, 508.5)", whenever it holds a row. The board
   is played through its turn instead.
8. **Hanweir Garrison**, registered and pooled: {2}{R} 2/3 Human Soldier,
   "Whenever this creature attacks, create two 1/1 red Human creature tokens
   that are tapped and attacking." Its meld half prints no meld ability
   (Hanweir Battlements does), so nothing on it is dead. Its rulings
   (Scryfall, 2026-10-16) are tests: #1, each token's defender chosen as it
   is created, at four seats; #2, never declared, so Hellrider does not
   trigger for them; #3, ordered before or after another attack trigger, and
   a targeted one cannot target the tokens, since they do not exist when its
   target is chosen (CR 603.3d). #4 points at an article.
9. **Relentless Assault** (decision 15): `ObjectFilter::AttackedThisTurn`
   over the `Attacked` rows, with `Primitive::Untap` over
   `FilteredPermanents` (Aggravated Assault's) and `Primitive::ExtraPhases`
   (RE-10). Registered; Hanweir Garrison is the pooled card.
10. **Atoms.** ATOM-506.4-003 covered, and ATOM-506.4-002 annotated on
    `a_stolen_attacker_is_removed_from_combat`. Also within reach:
    ATOM-506.3a-001, 506.3b-001, 506.3d-001/002/003, 506.4-005, 508.4-001 and
    511.3-001; ATOM-506.3c-001 partly, its planeswalker form waiting for
    §2.23.

**Planned shapes corrected here, with no decision**
- Decision 9's "each caller of `remove_from_combat` announces it" loses the
  departure (its zone change announces it) and gains the end of combat
  (decision 12), and a record is one removal rather than one creature.
- The choice is a new `ChooseEnteringDefender`, not `SelectRecipients`, and
  the batch asks it, not the instruction (above).
- `EntersAttacking` names what printed cards specify, `Player` and `SameAs`,
  in place of the design's `Specified(EffectRecipient)`, which let any
  recipient through and read ninjutsu's "the same as" through `AttackedBy`
  (review, 2026-10-17).
- `Primitive::ReturnToBattlefield` takes the word too: 12 literals and one
  fixture, about 70 lines in all, since the entry it proposes is the one the
  word is decided in.

**Sized.** Code at 1.5–2× plus tests, re-counted 2026-10-17 after review.

| Piece | Code | Tests |
|---|---|---|
| Item 128: `EntersAttacking` on two instructions (35 literals), Divine Visitation's carry, `EntryAttack` at resolution, the batch's decision and `ChooseEnteringDefender` at its five matches, the performer's checks and write; CR 508.8's rename | ~175 | ~380: never "attacked", by 508.3a (Hellrider), 508.3b and 508.3d; "attacks alone" beside one, both ways; the four-seat choice; no prompt at two seats; 506.3a; 506.3b; 506.3c and 508.4a, a player gone; 506.3d's three steps; "that player"; Parallel Lives; Divine Visitation; 508.8's second leg; the return door |
| The record at four causes, decided on one board, and its readers | ~60 | ~120: each cause; two blockers and an attacker removed at once; 511.3 |
| The combat's history, its round's span, and item 270's read | ~75 | ~180: two combats in one turn (RE-10); a row kept into the next turn and pruned after the round; the former defender after a steal, an effect and a regeneration at four seats; stolen then killed; a creature put onto the battlefield attacking writes no `Attacked` row |
| The game-wide history struct (decision 11): the three loose "this turn" fields moved in | ~45 at 33 sites, mechanical | — |
| The settlement and the type-change cause | ~55 | ~140: ATOM-506.4-003 for an attacker and a blocker, which stays blocking nothing while its attacker stays blocked (CR 509.1h); the two boards the probe found, a player leaving with March of the Machines at four seats and Mirrorweave's copy of an animated land |
| The save's report | ~5 | ~10 |
| Which records are a log line: one predicate beside `format_record`, read by the dev GUI's log and `cli_play`'s, hiding a rule's sweep unless a trigger reads it (decisions 12 and 16) | ~20 | ~30: the end of combat and the untap step hidden; an untap a trigger reads shown, with the rest of its event |
| The settlement at each outermost batch's close (decision 13) | ~10 | ~40: a trigger in the causing batch's window reads the board without the removed combatant; a fixture's "as long as it's attacking" ends with the removal; a control change a batch causes is announced in that batch's window |
| Hanweir Garrison | ~20 | in item 128's row, and ruling #3, ~40 |
| Relentless Assault | ~35 | ~100: untaps what attacked and not a token put onto the battlefield attacking; Hellrider triggers in the second combat (its fourth ruling) |
| **Whole: 1,770–1,995** | ~500 (730–955) | ~1,040 |

Against TR-5e's 945–1,065: the type-change cause and its settlement,
Hanweir Garrison, Relentless Assault and the return door are new, CR 511.3's
record, CR 508.8's leg and the round's span were not counted, and the
history struct gathers three fields.

**Measured at design** (the probe, 2026-10-17). The settlement's combat part
reads each combatant's frame. Over the four tables above, the engine's
layer walks were identical with the reads and without them, so every read
was a memo hit: the frame was already computed, or would have been by the
next priority's legal actions. Memo hits per game rose by +36 at two seats
and +115 at four in `performance` with the epoch gate (+0.07% of 48,569 and
159,199), and by +189 and +524 without it (+0.4% and +0.3%). The epoch gate
cut the sweeps that read anything from 19,620 to 4,122 over 200 games at
two seats.

**Predictions for the close-out** (to be re-stated before `close_out.py`).
Engine arm against `main`: gameplay rows identical at both seat counts
except the games where a combatant stops being a creature, which the probe
counts (1 of 200 at four seats in `performance`). Every combat's fuzz event
dump, which prints every record, gains one `RemovedFromCombat` line at its
end, and the dispatch counters move by one window a combat, and by a
control change's window where a batch causes one. Shipped arm: every dealt
game moves once Hanweir Garrison is pooled, and its `stress` distortion is
reported.

**Decided at review (2026-10-17)**

- **11. The combat history lives in a game-wide history struct**, with the
    three loose "this turn" fields moved into it. There was none:
    `state::history`'s `PlayerHistory` is per player and holds only counts,
    a slot per `TurnFact` (item 179), and what is game-wide sat in
    `action_taken_this_turn`, `triggered_this_turn` and
    `resolutions_this_turn` on `GameState`, each cleared by `begin_turn`.
    `state::history::GameHistory`, beside `PlayerHistory`, holds the three
    sets and `CombatHistory`, with one `begin_turn` that clears the sets and
    prunes the rows to the round. 33 sites move, mechanically, and the
    save's exhaustive destructure names one field instead of three.

    **The rows are shared, and why that is not a trick on the floor.** Floor
    2 prices one clone because an AI search clones the board at every
    checkpoint (item 143) and throws most of the forks away. With the rows
    behind an `Arc`, a clone copies a pointer: no allocation and 8 bytes,
    against floor 2's 39 of 64 and floor 3's 100 KB of 128. The rows are
    copied only by a fork that writes one: `Arc::make_mut` copies while
    another holder shares them, so a fork's first combat record copies the
    rows once, about 1.5 KB at most, and its later writes are free. The cost
    moves from every clone to the forks that play through a declaration or a
    removal, once each, and floor 1 (decisions per core-second) still
    measures it. The one writer is the history's arm in `advance_history`,
    so it is one site, with nothing else to do. Item 180 fixed the
    registry's rows the same way: 68 allocations a clone with every fork
    copying their payloads, 34 with them shared.
- **12. CR 511.3 writes one `RemovedFromCombat`, which the logs show only
  when a trigger reads it.** Every removal is a record, so the history
  needs two kinds of row and the reader one rule: the last `Removed` row in
  the latest combat. The end of combat is a rule removing everything with
  no choice in it, so its record is implicit in the dev GUI's log and
  `cli_play`'s, through one predicate beside `format_record`, unless a
  triggered ability reads it. Every `AbilityTriggered` carries `caused_by`,
  the record it matched, so the predicate shows the hidden records of the
  event a trigger read: for a "one or more" trigger, every record of that
  event (CR 603.2c). The fuzz event dump keeps every record. The why panel
  does not give this for free: it opens from a record's own line and lists
  the triggers asked about it, so a hidden line has nothing to open. A
  removal by an effect, a steal or a type change is always a line. Nothing
  triggers on a removal today (`EventKind::from_record` answers none), so
  the caveat matters first for decision 16.
- **13. One settlement, at TR-4d's three points and at each outermost
  batch's close** (decided at review, 2026-10-17, after the check below).
  The owner: a phase moving a later A/B is no cost; only performance and
  the floors are.

  **Checked** (2026-10-17): the rules each part reads, the printed cards
  that read combat membership or end creature-ness, and every point where
  a layer input is written, against every reader that could run before the
  next settlement.
  - *The order holds.* Control first, then combat membership. A removal is
    idempotent, so a permanent stolen and made a noncreature at once is
    removed once, by its control change. CR 506.4d and 506.4e make the
    combat part read a permanent's roles and types together (a blocking
    creature that is also an attacked planeswalker leaves combat only when
    it is neither; an attacked planeswalker-battle by its protector), so
    §2.23's checks join this phase's part rather than make a third. The
    controller they read is derived, so it is current whichever part runs
    first.
  - *The epoch gate holds.* A frame changes only when a layer input is
    written, and every such write bumps the epoch (`bump_layer_epoch`'s
    list), so a pass with the epoch unmoved finds nothing.
  - *Simultaneity holds.* One settlement's removals are one event and one
    record: CR 506.4 removes a permanent as its cause happens, and
    everything one cause ends, it ends at once.
  - *TR-4d's points alone left a gap, which TR-4d has too, and the batch's
    close closes it.* The layers always answer a permanent's current
    controller, so TR-4d could announce a change at the next point and
    every reader in between still saw the right controller. Combat
    membership is stored: `AttackingInfo` stays until something writes it.
    A batch that ends March of the Machines dispatches its own window before
    the next point, and CR 603.10 checks triggers against "objects that
    exist immediately after an event", by which time CR 506.4 has removed
    the artifact. So that window's triggers, and static abilities that read
    the attack continuously, would read a combatant that is not one: "as
    long as ~ is attacking" is 7 printed cards (`o:"as long as ~ is
    attacking"`; Wardscale Dragon's "defending player can't cast spells",
    Gaea's Liege's power and toughness). TR-4d's own removal (item 267) had
    the same gap: a creature a batch returns to its owner stayed in combat
    through that batch's window. Nothing registered read either: Hellrider
    reads the declaration's record, and the 5 cards that say "if ~ is
    attacking" are activation restrictions, read at priority.

  **What it changes.** The whole settlement also runs at each outermost
  batch's close, before its window is dispatched, so its records join the
  window of the event that caused them, and that window's triggers read the
  board without the removed permanents, as `control.rs` already removes a
  stolen permanent "as part of the change, so the change's triggers read
  the board without it". A control change a batch causes is announced in
  that batch's window, one event with its cause, rather than in the next.
  Its look-back frames are the ones the batch's head kept, as now. The
  triggers either window queues are placed together at the next priority,
  so no gameplay row should move. With the epoch gate, the head after a
  close finds nothing new, so the reads move earlier rather than adding up.
  About 10 lines.
- **14. The save reports the history.** The owner: a board that tests a
    multi-combat interaction is played through its turn, not saved with
    each object's part in it, and the words would cost about 40 lines for a
    feature no one uses. The report is about 5.
- **15. Relentless Assault comes in.** It is the printed reader of the
    `Attacked` rows, and it tests them.
- **16. The untap step's untaps leave the log too**, by the same predicate,
  with `TurnBasedAction::Untap` its second hidden cause, and show when a
  trigger reads one. They print today, one line for each permanent the
  untap step untaps (a permanent already untapped announces nothing, CR
  603.2e): 86 of the 995 lines of one seeded game (`performance` seed
  12345, two seats), and the last two lines of the owner's example at
  review.

### As landed (2026-10-17)

**Built as designed**, with decisions 11–16 as the four review rounds
decided them. Seven commits: the game-wide history with the combat's rows
and CR 508.8's rename; removal from combat as a record, with item 270's
read; the settlement; item 128 through the one entry; the log rule; the two
cards; and the entry's attack boxed, after the close-out's first reading
found `GameAction` grown by 24 bytes.

**What the build corrected.**
- **The settlement's gate reads `layer_epoch()`**, the field plus the
  continuous-effect registry's own mutations. The first draft read the
  field alone and missed a cleanup step's expiry, which two of TR-4d's
  steal tests caught. The design's probe read the field alone too, so its
  gated counts undercount; its ungated +0.3% in memo hits bounds the reads,
  and the close-out's callgrind reading is the price.
- **The control sweep joins an enclosing window.** At an outermost batch's
  close it emits into the open window and leaves the dispatch to the close:
  a dispatch there flushes the buffer the close still reads.
- **The combat's history takes the turn from the game.** A new game is on
  turn 1 before any `begin_turn`, so a counter of its own started at 0.
- **`ExtraPhases` names its anchor.** Relentless Assault's first ruling
  found RE-10's splice ignoring "after this main phase" (7 cards, Aggravated
  Assault among them), which adds nothing outside a main phase; "after this
  phase" (26 cards) splices after any. Nothing registered reached the
  difference: Aggravated Assault is sorcery speed.
- **Reminder text is not rules text** (CR 207.2, the registry's own test):
  Hanweir Garrison's "(Melds with Hanweir Battlements.)" stays in its doc
  comment.

**At the build's review (2026-10-17).** The return door had no "tapped"
word, so Phoenix Chick's "return this card … tapped and attacking" entered
untapped; `ReturnToBattlefield.tapped` carries it, as `TokenDef.enters_tapped`
does a token's (156 cards return something to the battlefield tapped). Two
more copies merged: `DefenderFilter::admits` is the trigger matcher's and the
entry's one reading of an attack target, and `players_named_by` the
resolution's one reading of the players a recipient names. The history's
writer writes and returns nothing; a record's place is read after it, off
the four kinds that have one. Renamed for their call sites: `EntryAttack` to
`EntryAttackCandidates`, `engine/settlement.rs` to
`engine/layer_settlement.rs`. The CR-general tests of CR 506.5 and 508.8 run
over both doors, a token and a card from a hand. The review's question on "as
long as" found the memo serving stale frames for four inputs no write moved
the layer epoch for: life totals and the tapped status (items 199 and 187),
combat and the histories. At the owner's word they are fixed here, as
`GatedWalkInput`s: the walk notes the read, and the write moves the epoch only
when a frame valid now read it (`layers-architecture.md` §12, 7a). The
settlement, gated on the same epoch, sees an "as long as" flip with them.

**What it leaves.** Item 272, "for each opponent, … attacking that player"
(6 cards), needs a players `Selector`. CR 506.4c–e, an attacked
planeswalker or battle that stops being one, join the settlement's combat
part with `backlog.md` §2.23. Item 11's blockers half is TR-5c's.

**Size.** +2,547/−286 in code and tests (code +1,374/−247, tests
+1,170/−35, the dev GUI +3/−4), against 1,770–1,995, past the band by
the review's fixes. Thirty-six integration
tests in
`phase_tr5f_integration_test` and three unit tests in `state::history`,
each fix shown to fail with its piece taken out.

#### TR-5b — counters, and one history struct — ✅ landed 2026-10-17

*Evicted 2026-10-17 from `plans/triggers-architecture.md` §12, where the heading and a stub remain.*

### The plan as designed (TR-5b, as §12 carried it on 2026-10-17)

*Designed 2026-10-17 against the tree at `9f265c3` (`main` at `65d33bc`,
and #257's review follow-ups on this branch). Reviewed once the same day
(the owner, on the design PR) in four rounds: decisions 2, 3 and 4 in the
first, the removal's remover in the second, and decision 1 in the fourth,
once the third had shared its snapshot. All four are decided, at the end of
this section. Nothing built.*

**Where it sits.** Counters reach the chokepoint since RB:
`GameAction::AddCounters` and `RemoveCounters` each announce
`CountersChanged`, and `EventKind::from_record` files that record as one no
arm reads. Three changes to counters are not announced the way a trigger
needs. Counters a permanent enters with announce nothing (CR 122.6, §3.12's
item 17). CR 704.5q's annihilation writes both kinds directly, after the
state-based batch, and announces `CountersAnnihilated` (`codebase-state.md`
item 6's last half; §3.12's item 18). And an entry template cannot read X
(CR 107.3m). This phase builds the two arms over the records, the count CR
122.7 compares, and §4.4's per-subject reading. It starts with the history
move the owner asked for at #257's review: the per-player rows join the
game's.

**What the re-read found**
- **Three printed readings of how many triggers one event makes**, all on
  these arms, each with a ruling (Scryfall, 2026-10-17):
  - *A counter.* "Whenever a +1/+1 counter is put on this creature" triggers
    once per counter, entry counters included (Fathom Mage #8 and #9,
    Bloodcrazed Hoplite #2 and #3). A removal is the same (Protean Hydra #4).
  - *One or more on an object.* Once per object: Enduring Scalelord #2,
    Hapatra #2, Generous Patron #1. Simic Ascendancy reads this way.
  - *One or more on one or more objects.* Once per event, since one event
    is one occurrence (CR 603.2c): Invisible Woman, Sue Storm's "on one or
    more other Heroes you control", and Cloaked Cadet's "on one or more
    Humans". The Woman has no ruling, and the Cadet's #1 is also what its
    once-each-turn limit gives, so the owner confirmed the reading at
    review. Under Basri's Solidarity ("put a +1/+1 counter on each creature
    you control"), the Woman as printed triggers once, and worded as
    Enduring Scalelord is, once for each other Hero.

  So the counter arms take three multiplicities, and the second is not
  `OncePerEvent` behind a filter.
- **Counters a permanent enters with are "put" on it** (CR 122.6): Simic
  Ascendancy #1, Enduring Scalelord #1, Fathom Mage #8. The putter is the
  entering permanent's controller unless the effect names one (CR 122.6a;
  Generous Patron #6).
- **A player is a subject too.** `o:"whenever you get one or more"` is 4,
  Territorial Gorger among them, and its ruling #5 says effects that interact
  with counters a player gets interact with energy.
- **Cards read who put the counters on, and who removed them.** 22 cards
  say "whenever you put one or more" counters on something (`o:"you put one
  or more"`; Generous Patron, Hapatra). One says "whenever you remove"
  (`o:"whenever you remove"`): Watcher of Hours, about its own time counters
  being removed while it is suspended in exile, which waits for counters on
  a card off the battlefield (`backlog.md` §2.6). A trigger on the removal
  itself needs no remover and is in this phase: "whenever one or more
  counters are removed from this creature" is Chandra, Fire Artisan's and
  Regenerations Restored's.
- **Every put has a player, and so does a removal, unless a rule removes
  the counters.** The controller of an instruction does what it says (CR
  608.2c), so Vampire Hexmage's "remove all counters from target permanent"
  is its controller's removal, as a put is. Damage that puts counters names
  its putter, the source's controller (CR 120.3d; Nest of Scarabs #2), but
  damage that removes loyalty names no one (CR 120.3c), and neither does CR
  704.5q. So both records name a player: a put's always, a removal's when
  there is one.
- **CR 122.7's threshold** is printed on 10 cards, by a regex over "the
  second" to "the hundredth ... counter is put" (Claim the Kingdom's "fourth
  plan counter" among them). Every Saga's chapter reads the same comparison
  (CR 714.2b), which `backlog.md` §2.44 builds on this arm.
- **"The last" is read on a removal, on the battlefield too.** Orcish Mine's
  "when the last ore counter is removed from this Aura", and vanishing's
  trigger (CR 702.63a), beside suspend's in exile (CR 702.62a).
- **CR 704.5q's board where the creature also dies is ATOM-704.8-001.**
  Strangleroot Geist, registered, has undying and a +1/+1 counter and takes
  three -1/-1 counters. One check removes a pair and puts it into the
  graveyard (CR 704.5f), and CR 704.8 frames it before either, with its
  +1/+1 counter, so undying does not return it. Persist is the mirror: Kitchen
  Finks's and Wingrattle Scarecrow's ruling #7, TR-7's board.
- **`CountersAnnihilated` is read by the log and one test helper**, at 6
  sites.
- **X is on `GameState::resolving` already**: the spell resolving into the
  entering object carries its X (`ResolvingObject::x_value`), and
  `PermanentState::entering` copies it. `evaluate_enter_template` refuses
  every amount but `Fixed` and `SourcePower`.

**The pieces**
1. **One history struct** (decision 1), its own first commit, with no
   behavior change. `PlayerState.history` becomes `GameHistory.players:
   Vec<PlayerHistory>`, indexed by `PlayerId` as `GameState.players` is, and
   built for the seats by `GameHistory::new(seats)`. Each player's snapshot
   for "since your last turn" is shared between clones behind an `Arc`,
   since one writer replaces it once a turn (decision 1). `begin_turn_history`
   reads every player's row at once, so it becomes a `GameHistory` method
   beside `turn_began`. `since_your_last_turn` takes the two players. 36
   reads and writes move in 11 files, plus the field in `player.rs`:
   `condition.rs`, `zones.rs`, the history writer, `game_state.rs`,
   `scenario/build.rs`, `scenario/write.rs` (the save's destructure of
   `PlayerState` loses the field), `state/history.rs`'s own tests, and four
   test files. CR 800.4i's departed player keeps its row, since the seat list
   never shrinks.
2. **Two records for two directions** (decision 2):
   ```rust
   /// CR 122.1, 122.6: counters were put on an object or given to a player.
   CountersPutOn {
       subject: CounterSubject,
       counter_type: CounterType,
       n: u32,
       by: PlayerId,
       now: u32,
   },
   /// CR 122.1, 701.2: counters were removed, as many as there were.
   CountersRemovedFrom {
       subject: CounterSubject,
       counter_type: CounterType,
       n: u32,
       by: Option<PlayerId>,
       now: u32,
   },
   ```
   - `by` is the putter (item 17). For an instruction it is
     `AddCounters::by`. For an entry it is the row's named player, or else
     the entering permanent's controller as it enters, which CR 616.1b has
     settled before the performer runs. A `PlayerId`, since every put has
     one.
   - A removal's `by` is the remover, which `GameAction::RemoveCounters`
     gains at its five producers:

     | Producer | The remover |
     |---|---|
     | An instruction, `Primitive::RemoveCounters`, a rider's included | its controller (CR 608.2c) |
     | Damage to a planeswalker (CR 120.3c) | none: the rule names no player |
     | A replacement's "instead remove" (shield and stun counters, CR 122.1c–d) | the replaced event's: its instruction's controller, none for a rule's or a turn-based action's |
     | CR 704.5q | none |
     | A cost, `Cost::RemoveCounters`, once its payment is built | the player paying it |
   - `now` is the subject's count of that kind after the record, taken by
     the performer. CR 122.7's condition reads it (piece 4). So would "when
     the last time counter is removed" (CR 702.62a, 702.63a), once counters
     off the battlefield let suspend land.
   - `CountersChanged` goes, at 9 lines in `src` and 5 in the tests.
     `CountersPutOn` is about 40 bytes, inside the 72 that
     `the_event_keeps_its_size` pins.
3. **An entry announces its counters** (item 17). `perform_entry` announces
   one `CountersPutOn` per `EnterMods.counters` row, after
   `PermanentEnteredBattlefield` and in the entry's batch. Announced, never
   proposed: CR 614.16's doublers applied to the entry already (RC-5), so
   none sees the counters twice. Protean Hydra's X counters and Master
   Biomancer's are two rows of one kind, so two records of one event.
4. **The two arms**, §3.3's row over the split records:
   ```rust
   CountersPutOn {
       subject: CountersOn,
       counter_type: Option<CounterType>,
       by: Option<PlayerRef>,
       multiplicity: Multiplicity,
   },
   CountersRemovedFrom {
       subject: CountersOn,
       counter_type: Option<CounterType>,
       by: Option<PlayerRef>,
       multiplicity: Multiplicity,
   },

   /// What the counters are on (CR 122.1): an object, or a player.
   pub enum CountersOn {
       Object(TriggerSubject),
       Player(Option<PlayerRef>),
   }
   ```
   - **Occurrences.** `PerOccurrence` is one per counter:
     `triggering_events_in` returns `n` triggering events for one record, as
     `Attacks` returns one per attacker. "That many" of one is 1.
   - **`Multiplicity::OncePerSubject`, §4.4's third reading.** One trigger
     for each object or player the window's matching records are about.
     `TriggeringAmount` sums that subject's records, and "that creature"
     (`EventRole::Object`) is the subject, as for one occurrence.
     `match_candidates` folds it beside `OncePerEvent`, keyed on the
     subject, and `delayed.rs`'s fold does the same. A variant of the shared
     enum rather than a counter field, because the fold is the dispatcher's.
     The registry refuses it on an arm that names no subject, which is every
     other arm today.
   - **CR 122.7's threshold is the ability's own condition** (the owner, at
     review), not a field on the arm. "When the fourth plan counter is put
     on this" is the put arm on `ThisObject`, read `OncePerSubject`, with an
     intervening "if": `Condition::CounterCountReached { counter_type, n }`,
     the count was less than N and became at least N. That is CR 714.2b's
     own wording for a Saga's chapter, and CR 122.7 says the same without
     the "if". After is the count the trigger's last record of that kind
     left (`now`), and before is after less the counters of that kind its
     records put. It reads the records, so it answers the same at the
     trigger and at resolution (CR 603.4), and two records on one creature
     in one event, as two wither sources make, are one comparison. The
     registry refuses it on a trigger that is not a put read once per
     subject. ATOM-122.7-001 is its fixture.
   - **`by` asks the record's player.** "Whenever you remove" is `by:
     Some(You)` on the removal arm, and a removal no player made matches no
     `by`.
   - **The subject is read after the event** (CR 603.10). No counter arm
     looks back. A newcomer answers them (`answered_arms`), so a creature's
     own entry counters trigger it (Fathom Mage #8). A permanent the same
     event moved is not asked from the zone it went to.
   - The arms join about 15 exhaustive matches: `reads`, `amount_of`,
     `multiplicity`, `looks_back`, `EventKind`, the bound reads, the
     authoring words and the audit. `EventKind` gains two kinds and loses
     none.
5. **CR 704.5q in the state-based batch** (decision 3). Each permanent with
   both kinds gets two `RemoveCounters` members, gathered against the same
   board as every other member. The sweep after the batch goes, and so does
   `CountersAnnihilated`: item 6 closes, and the variant is item 18's fourth
   deletion. The batch's CR 704.7 collapse keys on the subject and the
   result. A result is a departure or a loss, an unattach, or one kind's
   removal, so the two removals collapse neither with each other nor with a
   death. The removals go at the batch's head, ahead of the moves, so each
   is performed while its permanent is still there. CR 704.3 makes them one
   event with the moves, and a moved permanent's frame is taken before the
   batch performs, as now (CR 704.8).
6. **X in an entry template** (CR 107.3m). `evaluate_enter_template`'s
   `AmountExpr::X` arm reads `GameState::resolving` when the resolving spell
   is the object entering. Every other entry reads 0, as the permanent's own
   X is (107.3m's last clause, 107.3g). ATOM-107.3m-001.
7. **`CounterType::Growth`**, named "growth".
8. **Simic Ascendancy**, registered. {G}{U} enchantment:
   - "{1}{G}{U}: Put a +1/+1 counter on target creature you control."
   - `CountersPutOn { subject: Object(Filter(a creature you control)),
     counter_type: Some(PlusOnePlusOne), multiplicity: OncePerSubject, .. }`,
     putting `TriggeringAmount` growth counters on this enchantment.
   - "At the beginning of your upkeep, if this enchantment has twenty or more
     growth counters on it, you win the game": `Condition::CounterCount` and
     `Primitive::WinGame`.

   Its four rulings are tests. #1, entry counters. #2, a board that reaches
   twenty only after the upkeep began. #3, counters removed in response.
   #4, the count it had as it left, which `CounterCount` reads off the `Lki`
   (TR-5a).
9. **Protean Hydra**, registered and pooled. {X}{G} 0/0 Hydra:
   - It enters with X +1/+1 counters (piece 6).
   - "If damage would be dealt to this creature, prevent that damage and
     remove that many +1/+1 counters from it" is Angel of Suffering's shape:
     `Prevent`, with a rider reading `ReplacedAmount` (CR 615.5, 615.12).
   - "Whenever a +1/+1 counter is removed from this creature, put two +1/+1
     counters on it at the beginning of the next end step" is
     `CountersRemovedFrom { subject: Object(ThisObject), counter_type:
     Some(PlusOnePlusOne), multiplicity: PerOccurrence }`. It creates a
     delayed trigger on `StepBegins { step: End }`, Flickerwisp's shape,
     whose "it" is `ThisObject`, found by identity (CR 400.7).

   Its six rulings are tests. #1, all the damage prevented and the counters
   removed as far as they go. #2, unpreventable damage still removes them
   (CR 615.12). #3, any removal. #4, that many triggers. #5, CR 704.5q's
   removal. #6, removed during an end step, so the next end step is next
   turn's.
10. **Atoms.** ATOM-122.7-001, the brief's. Within reach and planned:
    ATOM-107.3m-001 (piece 6), ATOM-704.5q-001 and ATOM-704.5q-002 (piece
    5), and ATOM-704.8-001 (piece 5, with Strangleroot Geist).

**Planned shapes corrected here, with no decision**
- **CR 122.7's threshold reads the records' counts, not the board**
  (§3.12a's correction, corrected again), and it is the ability's condition
  rather than §3.3's `nth` on the arm (piece 4). "The count now minus the
  window's `added`" reads the
  board at the dispatch, which is right only while the window moved nothing
  away: a creature annihilated and destroyed in one check reads 0. Each
  record keeps the count after it, so the event's before and after come
  from its first and last record, with no board read.
- **`by` is a `PlayerId`**, where item 17 had an `Option<PlayerId>`. The
  entry's default putter is known when the performer announces, so the
  record never has to say "the default".
- **The removal names its remover**, an `Option<PlayerId>` on the record
  and on `GameAction::RemoveCounters`, which had none (piece 2).
  `EventPattern::RemoveCounters` stays without one until a replacement or a
  restriction reads it.
- **§3.3's counter row and its closing paragraph**: the counters become two
  records with an arm each, so only life keeps two arms over one record, and
  the performed kinds after TR-5 are thirty-five, not thirty-four.

**Not in this phase**
- §3.10's `counters_put` summary: no registered card reads a count of
  counters put this turn.
- "The last" on `CountersRemovedFrom`, read off `now`: about 10 lines and a
  fixture, with the first registered card that reads it (vanishing, Orcish
  Mine).

**Sized.** Code at 1.5–2× plus tests, counted against the tree on
2026-10-17.

| Piece | Code | Tests |
|---|---|---|
| 1. The history move: 36 sites, `GameHistory::new`, the turn snapshot moved, the snapshot behind an `Arc` (decision 1's (a″)) | ~60 | ~30, the sites in four test files and the struct's own tests, and a clone sharing the snapshot |
| 2–3. The two records, `by` and `now`, `CountersChanged` at 14 lines; the entry's announcement; the remover on `RemoveCounters` at five producers | ~75 | ~150: each putter (an instruction, an entry's default and its named player, CR 122.6a); each remover (an instruction, a replacement's, none for damage and CR 704.5q); `now` after two rows of one kind |
| 4. The two arms, three multiplicities, CR 122.7's condition and its read of the trigger's records, the player leg, the registry's two refusals | ~155 | ~280: per counter (Fathom Mage's #7–#9 on a fixture), per object (Enduring Scalelord #2 on a fixture), per event (Cloaked Cadet #1 on a fixture), `by` (Hapatra #2 on a fixture), the player leg (Territorial Gorger on a fixture), ATOM-122.7-001, two records on one creature in one event, a newcomer's own entry counters, both refusals |
| 5. CR 704.5q in the batch | ~35 | ~140: ATOM-704.5q-001 and -002; ATOM-704.8-001 with Strangleroot Geist; persist's #7; the collapse keeps both removals beside a death; the state-based test helper moved to the removals |
| 6. X in an entry template | ~15 | ~50: ATOM-107.3m-001; 0 when not cast, so a Hydra put onto the battlefield dies; X beside Master Biomancer's counters |
| 7–8. The growth counter, Simic Ascendancy | ~45 | ~140, its four rulings |
| 9. Protean Hydra | ~50 | ~190, its six rulings |
| **Whole: 1,630–1,850** | ~435 (650–870) | ~980 |

Against the 2026-10-12 row's 1,510–1,715: the history move, the player leg,
the remover and `now` are new. Docs at the close-out, about 350 more: the stub
and the archive, `fuzz-record.md`'s block, §3.3, §3.10, §4.4, the glossary's
multiplicity entry and `engine-map.md`'s history row.

**Predictions for the close-out** (to be re-stated before `close_out.py`)
- *The history commit.* An A/B against `main` reads IDENTICAL on every row,
  both pools, two seats and four. Clone floor 2 reads three allocations
  fewer at four seats (39 to 36 of 64), and floor 3 about 3 KB less.
- *The engine arm against `main`*, with no new card registered. Gameplay
  rows identical: CR 704.5q decides against the board it read before, and
  no registered card reads a counter record. The event dumps move:
  `CountersChanged` renamed, each `CountersAnnihilated` line two
  `CountersRemovedFrom` lines (item 6 counted 10 in 200 `stress` games on
  2026-09-01), and a `CountersPutOn` line for each counter row an entry
  carries. Those records now reach the gate, so the dispatch counters move.
  Instructions per decision +0.1% to +0.3%.
- *The shipped arm.* Every dealt game moves once Protean Hydra is pooled,
  and Simic Ascendancy's share of `stress` games won is reported (decision
  4).

**Decisions** (reviewed 2026-10-17, all four decided)

- **1. The history's shape — (a″), the rows in `GameHistory` and each
  snapshot shared** (the owner, at review's fourth round, after it was put
  again twice). The owner asked
  first whether one history struct costs the clone floor, and would keep
  the rows on `PlayerState` if it does (option (c)). Then, whether "since
  your last turn"'s snapshot, seats × seats summaries, grows past the floor
  as `TurnSummary` gains facts. As stored today it would. The fix is the
  same whichever struct holds the rows: share the snapshot between clones.

  **What "since your last turn" keeps.** It asks what a player did between
  the end of your last turn and now (CR 730's day and night, Arboria). The
  history answers it by subtraction. As your turn ends, it writes down every
  player's totals for the game so far: a `TurnSummary` each, 192 bytes, with
  cards drawn, life lost and the rest. Later, "since your last turn" is
  their totals now less what was written down. So each player keeps one
  written-down summary per player, its snapshot: at four seats, 4 × 4 = 16
  summaries. Today each player keeps its snapshot in its own `Vec`
  (`PlayerHistory::at_your_last_turn`), four heap allocations a clone. The
  rest of a player's history is its rows, three summaries (this turn, last
  turn, this game) inside `PlayerState`, in `GameState.players`' one
  allocation.

  **It grows two ways, and today a clone copies all of it.** Each new
  `TurnFact` is 8 bytes in every summary, and the snapshots are seats²
  summaries: 64, or 12 KB, at eight seats. With the rows, each new fact adds
  224 bytes to every four-seat clone today, and floor 3 has 28 KB of room.

  **Shared, a snapshot costs a clone nothing.** One writer replaces a
  player's snapshot, once a turn, as the next turn begins
  (`last_turn_ended`), and nothing edits one in place. So it can sit behind
  an `Arc<[TurnSummary]>`, as TR-5f's combat rows sit behind theirs. A
  clone copies the pointer, with no allocation and no bytes counted
  (`clone_bound_test` counts what a clone allocates). A new turn builds the
  next snapshot, one allocation, as today's `Vec` does. No fork ever copies
  one, since a write replaces it rather than editing it. An empty snapshot
  reads as zeros, as today's empty `Vec` does, so the commit's A/B still
  reads IDENTICAL. What a clone still copies is the rows, three summaries a
  player: linear in the seats, and 96 bytes a new fact at four seats.

  Counted from the code for `clone_bound_test`'s board, four seats, where
  floor 2 reads 39 of 64:

  | Option | The rows | The snapshots | Allocations a clone, four seats (two) | The history's bytes in a four-seat clone |
  |---|---|---|---|---|
  | Today | `PlayerState` | a `Vec` per player | 4 (2) | 5.4 KB |
  | (a′), round one | `GameHistory.players` | one `Vec` on `GameHistory` | 2 (2) | 5.3 KB |
  | **(a″), decided** | `GameHistory.players` | an `Arc` per player | 1 (1) | 2.4 KB |
  | (c″) | `PlayerState` | an `Arc` per player | 0 (0) | 2.4 KB |

  - *(a″)* is the one struct #257's review asked for, at one allocation,
    three fewer than today: floor 2 reads 36, and floor 3 about 3 KB less.
    The one allocation is the rows' `Vec`, the same whatever the seats or
    the facts.
  - *(c″)* is the floor's minimum, no allocation at all, and keeps the two
    structs: every read of "since your last turn" still reaches into both.
  - Round one's (a′), the 16 summaries in one `Vec`, is dropped: sharing
    beats it on both counts. So is (b), the rows flattened into a list per
    view.

  If the rows ever weigh on floor 3, the next lever is `SpellsCastOfType`:
  one slot per card type, 15 of a summary's 24.

  **Asked at review: what "this game" asks of the history** (Approach of
  the Second Sun, Echogoyfs). 37 cards say "this game" (`o:"this game"`,
  2026-10-17), five of them with no paper printing. Each kind of count has
  a home, and none is closed off by this shape:

  | What it counts | Cards | Where it lives |
  |---|---|---|
  | A player's count of one fact over the game | Flourishing Forces (life gained); First Contact and Once Upon a Time (the first spell this game); the two Frodos (the Ring tempting you); Captain Vargus Wrath and sixteen more (casting a commander from the command zone) | a `TurnFact` slot, read off the rows' `this_game` view, which is a running total rather than a window. A slot is 8 bytes in each of a player's three summaries, 96 bytes a four-seat clone, and is authored with its card (§3.10's rule) |
  | The same, about where the spell's card came from | Echogoyfs and Etali, Primal Garden: spells cast this game "that weren't from your starting deck" | a slot as above, beside one fact each card keeps: the starting deck it began the game in. CR 103.2a defines a starting deck, so this is not digital-only in the rules: a card brought in from outside the game, or cast from an opponent's library, was not from yours. Echogoyfs's power is then a CDA over the history, which the layer memo's `Histories` input already gates. Its double team is an Arena mechanic the CR does not define, which is the post-v1 custom-card question, not this one |
  | A count by card name | Approach of the Second Sun (spells cast with its name), Yidaro, Wandering Monster (cards cycled with its name), Gollum, Obsessed Stalker (opponents a creature with its name dealt combat damage) | not a slot, since names are unbounded: a per-player map from a name some registered card watches to its count, kept for the game. Bounded by the names watched, not the turns, and a store of its own, built with its first card |
  | A count for one object | Rampaging Ursaguana (the times it attacked); The Fallen (whom it has damaged); CR 903.8's commander tax, and the four cards that read one commander's casts (Liesa, Myth Unbound, Opal Palace, Study Hall) | on the object: a permanent's own count, which CR 400.7 starts over for a new object, or for a commander B2's designation, which keeps it across zones (CR 903.8 counts "each previous time the player casting it has cast it") |
  | Un-cards | Bureaucracy, Goblin Cruciverbalist, Pietra, Crafter of Clowns, Time Machine | out of scope |

  One cost to watch is the first kind's: most of its facts are read only
  over the game, so their this-turn and last-turn slots are spent for
  nothing. If those facts pile up, a game-only row beside the summaries
  takes each at 8 bytes a player. And §3.10's sketch of the commander tax
  as one slot on the summary is a player's total, which those seventeen
  cards read; the tax itself is per commander.
- **2. The counter records — split by direction** (the owner, at review).
  `CountersPutOn` and `CountersRemovedFrom` (piece 2) mirror their
  proposals, `AddCounters` and `RemoveCounters`, as their `EventPattern`
  arms and the trigger arms do. A put's `by` is a `PlayerId`, since every
  put has a putter, and a removal's an `Option`, since a rule's removal has
  none (second round, after Vampire Hexmage). The counts are unsigned. The
  alternative kept the signed `CountersChanged` with two arms over it, as
  `LifeChanged` has, where one `by` would have to be an `Option` for both
  directions.
- **3. CR 704.5q's two removals — every applicable action is performed**
  (the owner, at review). The collapse keys on the subject and the result,
  and the removals go at the batch's head, ahead of the moves (piece 5). CR
  704.8 holds as it does today, since the frame is taken before the batch
  performs, so ATOM-704.8-001's Strangleroot Geist and persist's #7 do not
  return. A Protean Hydra that survives the check triggers once for each
  +1/+1 counter removed (its #5). One the check kills triggers nothing: its
  ability does not function in the graveyard (CR 113.6), and a counter
  trigger does not look back (CR 603.10). The alternative skipped the
  annihilation on a permanent the same check moves, since its counters cease
  to exist with it (CR 122.2). No printed trigger tells the two apart, but
  CR 704.3 makes no such exception.
- **4. Card stress distortion — both registered** (the owner, at review:
  "it's a nontrivial win condition to reach, so enough game will have almost
  certainly occurred to not impact the diagnostics too much"). Protean
  Hydra is pooled. Simic Ascendancy's share of the `stress` games it wins is
  still measured and reported at the close-out, natural and with
  `--require`, at two seats and four, and whether it joins
  `PERFORMANCE_POOL` is read off that number.

### As landed (2026-10-17)

**Built as designed**, with decisions 1–4 as the four review rounds decided
them. Ten code commits: the history move; the two records; an entry's
counters; the arms, the fold and the condition; CR 704.5q in the batch; X
in an entry template; the two cards; their registration; the dev GUI's two
pictures; the pool.

**What the build corrected.**
- **The permanent's controller removes its shield and stun counters.** A
  shield counter's effects (CR 122.1c) and a stun counter's (CR 122.1d)
  each say "instead remove a [shield/stun] counter from it", and the design
  had those removals name no player. But a shield counter has two effects,
  and the damage one already removed its counter through a CR 615.5 rider,
  which names the controller of the effect: the permanent's controller. So
  a destroyed permanent's shield removal said "no one" and a damaged one's
  said "its controller". Both name the controller now, and so does a stun
  counter's. The removals that name no player are the ones a rule makes:
  CR 704.5q's, and the loyalty damage takes off a planeswalker (CR 120.3c).
- **An entry's rows merge by kind and putter** (`EnterMods`'s
  `same_kind_and_putter`), so "Protean Hydra's X counters and Master
  Biomancer's are two records" was wrong: they are one row and one record.
  Two of one kind are two records only under two putters.
- **CR 122.7's condition is answered per record.** One event's puts on one
  object chain their counts, so some record crossing N is the event
  crossing it, at the trigger, where the condition sees the record being
  matched, and at resolution, where it sees the binding's.
- **Two real prompts the tests answer**: Protean Hydra beside Master
  Biomancer is CR 616.1e's choice between two entry effects, and three
  delayed triggers made by three resolutions ask their order (CR 603.3b),
  since each is its own.

**At the close-out.** The reachability pass cast Protean Hydra 0 times in
400 games: MA-1's gate refuses an `{X}` symbol, which `mana-architecture.md`
§3.7 gives to MA-2. The owner moved MA-2 up to the next PR and kept the
Hydra registered and pooled for the one PR between. Simic Ascendancy is
pooled too, for the variety of ways a game ends.

**What it leaves.** "The last" on the removal arm (vanishing, Orcish Mine),
with the first registered card that reads it. A remover on
`EventPattern::RemoveCounters`, with the first replacement or restriction
that reads one. Counts over the whole game, `backlog.md` §2.47.

**Size.** +2,103 in code and tests (code +946, tests +1,157), against
1,630–1,850, over by tests. Thirty-seven integration tests in
`phase_tr5b_integration_test`.

#### TR-5c — blocks, targeting and activation — ✅ landed 2026-10-18

*Evicted 2026-10-18 from `plans/triggers-architecture.md` §12, where the heading and a stub remain.*

### The plan as designed (TR-5c, as §12 carried it on 2026-10-18)

*Designed 2026-10-18 against the tree at `2e9ac04` (`main`), and reviewed
twice the same day (the owner, on the design PR). Its one decision is
decided: three of round one's four refused families are built here, and
excess damage and item 275 moved to TR-5d, which the phase's name follows.
Pieces 11 and 12 stay below as written, for TR-5d's design to start from.*

**Where it sits.** A trigger reads the performed stream. A record is
announced, `EventKind::from_record` files it, the dispatch matches each
source's `TriggerEvent` arms, the binding keeps each object by the event role
it played (TR-5a), and placement puts the trigger on the stack. Four records
reach that path with no arm to read them, or short of what an arm needs:
- `BlockersDeclared` is written once per defending player who blocks, and not
  at all when nobody blocks (`engine/combat/steps.rs:172`): item 11's
  blockers half. `from_record` files it as unread.
- `Targeted`, built in #231 for the log (item 12), is filed as unread.
- `AbilityActivated` is filed as unread, and only an ability that uses the
  stack writes it (`engine/put_on_stack.rs:512`). A mana ability's activation
  writes nothing (`engine/mana.rs:27`).
- `DamageDealt` says how much damage was dealt, and not how much of it was
  excess (CR 120.10).

This phase completes the blockers record and builds an arm over each of the
four. Beside them it builds the "this turn" object filter, a recipient that
joins recipients, and item 275's fix to the ordering elision. Item 275 is
slotted here because this phase moves the random agent's stream anyway. The
decision's second round moved the excess reading and item 275 to TR-5d, which
reads the damage batch's other record and moves the stream too.

**What the re-read found** (CR 509.3a–g, 115, 602.2, 603.2 and 120.10, the
cards' rulings, and Scryfall's counts on 2026-10-18)

*The "this turn" filter*
- **The count at #257 included an evasion effect.** `o:"blocked this turn"`
  is 234, but 205 of them are `o:"can't be blocked this turn"`, which RS-3a
  owns. 11 cards read what a creature did (`o:"blocked this turn" -o:"be
  blocked this turn"`): Heat Stroke's "each creature that blocked or was
  blocked this turn", Fyndhorn Druid's "if it was blocked this turn", Defiant
  Vanguard's "all creatures it blocked this turn", and the four that say
  "attacked or blocked this turn" (Gideon's Triumph, Inferno Hellion, Lurker,
  Vizier of Deferment).
- **The other three counts split the same way.** Of `o:"attacked this
  turn"`'s 83, 53 are "you attacked this turn" (raid), the player's count
  `TurnFact::AttackersDeclared` already keeps, and 30 read an object. Of
  `o:"dealt damage this turn"`'s 50, 35 are "was dealt damage this turn". Of
  `o:"entered this turn"`'s 48, 27 are "that entered this turn".
- **A combat is shorter than a turn.** `o:"blocked this combat"` is 14, the
  Clockwork creatures among them (Clockwork Beast: "if this creature attacked
  or blocked this combat"). RE-10 builds additional combats, so "this combat"
  and "this turn" already differ in the engine, and TR-5f's combat rows keep
  which combat each row is from.
- **"That entered this turn" needs no history.** A permanent keeps the turn it
  entered (`PermanentState::entered_battlefield_turn`), and a new object is a
  new permanent (CR 400.7).
- **"This turn" is two families** (asked at review, with Betor, Ancestor's
  Voice). A *player's* count over the turn is TR-2a's turn summary
  (`TurnFact`, §3.10), which `Condition::ThisTurn` already reads. What it
  lacks is an amount: Betor's "equal to the amount of life you gained this
  turn" is one of 31 (`o:"life you gained this turn"`), and no `AmountExpr`
  reads a summary. An *object's* deed is this phase's filter, and it is wider
  than the four the row named. Each deed needs a writer, and the writers come
  in four kinds:

  | Deed (Scryfall, 2026-10-18) | Cards | Its writer |
  |---|---|---|
  | attacked, this turn or this combat | 30, 4 | the combat rows (TR-5f) |
  | blocked, was blocked, this turn or this combat | 11, 14 | the combat rows, from the declaration (piece 2) |
  | entered | 29 | the permanent itself |
  | was dealt damage; dealt damage to a player or creature | 35; 10 | rows off `DamageDealt` |
  | died, or was put into a graveyard from the battlefield; discarded; milled; put into a graveyard | 36; 10; 3; 3 | rows off the zone change, keyed by the object the move made (CR 400.7, RH-3) |
  | was cast | 11 | the permanent's `cast` facts, which CR 400.7d keeps |
  | crewed or saddled it | 13 | crew and saddle, neither built |
  | came under your control; turned face up | 1; 1 | the control sweep; morph (CV-6) |

  The queries are `o:"attacked this turn" -o:"you attacked this turn"`,
  `o:"attacked this combat"`, the two above for blocks, `o:"was dealt damage
  this turn"`, `(o:"that dealt damage to" or o:"that dealt combat damage to")
  o:"this turn"`, `(o:"that entered this turn" or o:"entered the battlefield
  this turn")`, `(o:"put into your graveyard from the battlefield this turn"
  or o:"put into a graveyard from the battlefield this turn" or o:"that died
  this turn")`, `o:"discarded this turn"`, `o:"milled this turn"`,
  `(o:"put into a graveyard this turn" or o:"put into your graveyard this
  turn")`, `(o:"was cast this turn" or o:"cast it this turn")`,
  `(o:"crewed it this turn" or o:"saddled it this turn" or o:"saddled this
  turn")`, `(o:"came under your control this turn" or o:"gained control of
  this turn")` and `o:"turned face up this turn"`.

*Blocks*
- **CR 509.3a–d differ in two things, as 508.3a–e did** (TR-5e's decision 5):
  what one triggering event is, and which side's filter is asked. 509.3a is
  once per blocking creature. 509.3b and 509.3d are once per blocking creature
  and creature it blocks. 509.3c is once per blocked creature. "This creature
  blocks a creature" and "a creature becomes blocked by this creature" name the
  same events. 509.3e counts the other side: Lairwatch Giant's "blocks two or
  more creatures", Amphibious Kavu's "becomes blocked by one or more blue
  and/or black creatures".
- **509.3g reads every attacking creature, not every declared attacker.** "It
  will trigger even if the creature was never declared as an attacker (for
  example, if it entered the battlefield attacking)", so the record names each
  attacking creature with no blocker declared, not only the pairs. 39 cards
  (`o:"attacks and isn't blocked"`). And "it won't trigger if the attacking
  creature is blocked and then all its blockers are removed from combat": the
  declaration decides it (CR 509.1h), which a record keeps and the board after
  does not.
- **One record per declaration step.** In multiplayer each defending player
  declares in APNAP order (CR 802.4), and CR 509.2a puts the triggers on the
  stack once, before the active player gets priority. §3.12's item 11 row
  decided one record per step for 509.3g's sake, and the re-read agrees.
- **32 cards name "that creature" across two arms.** "Whenever this creature
  blocks or becomes blocked by a non-Wall creature, destroy that creature at
  end of combat" (Cockatrice, the Basilisks; `o:"blocks or becomes blocked by"
  o:"that creature"`). "That creature" is the attacker in one arm and the
  blocker in the other. Both arms have both event roles, so a card that names
  `Attacker` passes the registry's event-role test and, on the second arm,
  destroys itself.
- **Effects that make blocks are 4 cards, and none is in the engine.** "Target
  unblocked attacking creature becomes blocked" (Curtain of Light, Dazzling
  Beauty, Trap Runner; CR 509.3c's "blocked by an effect") and Aetherplasm's
  creature put onto the battlefield blocking (CR 509.4, ATOM-509.4-001, a
  Phase 8 atom). Only the declaration writes a block (`steps.rs:152–166`),
  beside the scenario loader and the test helpers.
- **CR 506.5's "blocks alone" is a field** (the owner, at review's second
  round, since the rule defines it). It means "the only creature declared as
  a blocker during the declare blockers step": the only blocker at all,
  across every defending player, since all of them declare in that step (CR
  802.4). No printed trigger reads it (`o:"blocks alone"` 0), where "attacks
  alone" does. Its sibling "is blocking alone", blocking while no other
  creature is, is printed once, as a condition on a creature that died:
  Thijarian Witness's "if it was attacking or blocking alone", off its frame
  and not on this arm.
- **Loyal Sentry's one ruling.** The Sentry and the creature it blocks are
  destroyed before combat damage, and the blocked creature is destroyed even
  if the Sentry has left the battlefield before its trigger resolves. So the
  two objects are found one by one, and destroyed in one event (CR 608.2f). 2
  cards say "that creature and this creature" (`(o:"that creature and this
  creature" or o:"this creature and that creature")`); Dream Fighter is the
  other.

*Targeting*
- **122 cards say "becomes the target of"** (`o:"becomes the target of"`;
  ward's 210 are apart, `kw:ward`). "An opponent controls" is 31, "you
  control" 14, neither 42. "A spell" without "or ability" is 22, "an ability"
  3, "an Aura spell" 2. A player becomes the target on 2: "whenever you become
  the target of a spell or ability an opponent controls" (Amulet of
  Safekeeping, Dormant Gomazoa). 12 name "that spell or ability" (Frost Titan,
  Kira, the Glasskites).
- **A spell's targets are chosen in one announcement** (CR 601.2c), but
  `Targeted` is one record per (spell or ability, target), and each unbatched
  record is its own window (`state/trace.rs:886`). A "one or more ... become
  the target" trigger would then trigger once per target. No printed card
  says it (`o:"becomes the target of" o:"one or more"` finds Jace, Cunning
  Castaway and Mila, Crafty Companion, neither on this arm). TR-4d's control
  sweep already announces several records as one window
  (`engine/triggers/control.rs:156`), and the targeting records can too.
- **"For the first time each turn" is 17 cards** (valiant, Kira, the
  Glasskites): once per creature per turn. `TriggerLimit::FirstTimeEachTurn`
  reads a record's place in its turn (`engine/triggers/dispatch.rs:1533`),
  which only a cast, a draw, a life gain and a life loss have, each among its
  player's (`engine/triggers/history.rs:45`). So a valiant card written today
  never triggers, and nothing refuses it.
- **Frost Titan's first ruling**: its first ability "affects each spell
  (including Aura spells), activated ability, and triggered ability that's
  controlled by an opponent and that Frost Titan becomes a target of".
  `Targeted` is announced at all three of CR 601.2c's sites already: the cast,
  the activation, and CR 603.3d's placement. "Unless its controller pays {2}"
  is a payment inside a resolution, CP-1's (§6.2), and rulings #2–4 are about
  its second ability. "Counter that spell or ability" has no instruction yet:
  `CounterSpell` and `CounterAbility` each take one kind, and `CounterSpell`
  handed an ability would move it to a graveyard.
- **ATOM-702.21a-002** ("ward only triggers for opponent's spells/abilities")
  is Frost Titan's "an opponent controls". TR-5c covers its trigger half,
  partially, and the atom stays CP-1's with ward.

*Activation*
- **A mana ability's activation is an activation** (CR 602.2, 605.3a). Harsh
  Mentor's "if it isn't a mana ability" is an intervening "if", which says
  nothing unless a mana ability's activation triggers it. Ceaseless Searblades'
  ruling has it trigger whenever "you activate an activated ability of an
  Elemental permanent", which a mana ability is (CR 605.1a). Of the 39
  activation triggers (`(o:"whenever you activate" or
  o:"whenever a player activates" or o:"whenever an opponent activates" or
  o:"activates an ability")`), 14 mention a mana ability. 8 read a loyalty
  ability, which nothing has until CP-2, 8 "copy that ability" (CV-4, item
  256), and 5 an exhaust ability, a keyword the engine lacks.
- **The source is asked as it was on the battlefield.** Harsh Mentor #2 and
  Crackdown Construct #3: an ability of a card in a hand or a graveyard
  triggers neither. Every activation is from the battlefield today
  (`put_on_stack.rs:414`) until PM's. A cost can sacrifice the source before
  the record is written (CR 602.2b runs 601.2h before 601.2i), so the record
  keeps the source's frame, as `DamageDealt` keeps a gone source's.

*Excess damage*
- **The readings** (CR 120.10, and Aegar, the Freezing Flame's rulings).
  Damage already marked counts (#2). Any damage from a deathtouch source is
  lethal, so the rest of it is excess (#4; CR 702.2c, 120.4a). A creature that
  is also a planeswalker takes the smaller lethal amount (#6; CR 120.10's "the
  greatest of the calculated amounts"). And excess is a fact of the event,
  "those sources together": two sources into one creature make one excess.
- **Who reads it** (`o:"excess damage" (o:when or o:whenever)`, 12 cards):
  - the arm, "is dealt excess [noncombat] damage", once per permanent per
    event (Aegar, Fall of Cair Andros);
  - a reflexive trigger on its own damage (Rhino's Rampage, Unleash the
    Inferno);
  - "if excess damage was dealt this way", the instruction's own records
    (Bolg of the North, Cramped Vents, Mephit's Enthusiasm, Molten Impact);
  - "was dealt excess damage this turn", an object's history (Maarika,
    Brutal Gladiator; Overclocked Electromancer).

  Megatron, Tyrant's "if excess damage would be dealt to that creature this
  way, instead that damage is dealt to that creature's controller" is CR
  120.4a's modification of the event: a self-replacement effect, the
  resolving ability replacing part of its own effect (CR 614.15), applied
  before any other replacement (CR 616.1a). It is not 120.10.
- **Lethal damage has one reading in the tree already.** Trample's
  `lethal_damage_owed` (`engine/combat/keywords.rs:63`) reads toughness less
  damage marked, less what other sources assign in the same step, and 1 for a
  deathtouch source (CR 702.19b). CR 120.10's lethal damage is the same
  quantity.

*Item 275*
- **What the elision is.** When one player has several triggers waiting, CR
  603.3b lets them choose the order. The engine does not ask when the order
  cannot matter: every entry has one def, and the entries agree on each fact
  the def reads (`trigger_order_cannot_change_outcome`, item 163). "This
  object" is one of those facts (`BoundReads::SOURCE`), and today it compares
  the entries' origins. A delayed trigger's origin carries its own registry
  number, so no two delayed triggers ever agree, and Protean Hydra hit for
  three asks the order of three identical triggers. Item 275 compares the
  source instead of the origin.
- **What that uncovers: X.** Once the source is compared, two delayed
  triggers of one template can agree for the first time, so the elision has
  to compare every fact in which two of them can differ. One is X: a delayed
  trigger has its creator's X (CR 107.3n, `PendingTrigger::x_value`), and the
  elision reads `AmountExpr::X` as reading nothing
  (`engine/triggers/bound_reads.rs:210`), since no two entries that could
  differ in X ever reached the comparison before. An ability that resolves
  twice, with X = 2 and then X = 5, each time creating "at the beginning of
  the next end step, draw X cards", would put the two on the stack in either
  order without asking, though the player may want the 5 to resolve first.
- **How it is dealt with.** The fix adds X to the facts the elision
  compares: `AmountExpr::X` reads a new `BoundReads::X`, and
  `entries_agree_on` compares the entries' `x_value` when the def reads it
  (piece 12). No registered delayed trigger reads X, so nothing changes in
  today's games, and a fixture proves the guard.

**The pieces**
1. **The declaration's record** (item 11's blockers half). One record per
   declare-blockers step, written after the last defending player declares
   (CR 802.4), and written when nobody blocks:
   ```rust
   /// CR 509.1: every defending player's blocks (CR 802.4), each as (blocker,
   /// attacker), and each attacking creature no creature was declared
   /// blocking, which became an unblocked creature (CR 509.1h).
   BlockersDeclared { blocks: Vec<(ObjectId, ObjectId)>, unblocked: Vec<ObjectId> },
   ```
   - **The order carries no rules meaning.** The blocks are declared together,
     and CR 509.2a says the order in which the triggers triggered doesn't
     matter: their controller orders them (CR 603.3b). It is fixed only so a
     game replays: the dispatch walks the record in order, so the record's
     order becomes the order of the triggering events and of a prompt's
     options, which a `DecisionProvider` picks by index. Nothing is sorted.
     The pairs keep the order the defending players answered in, which CR
     802.4 already puts in APNAP order, and `unblocked` is
     `battlefield_ordered`'s, by timestamp (CLAUDE.md's determinism rule).
   - Announced with the step's provider, as `AttackersDeclared` is: one record
     can hold two triggering events of one arm, which CR 603.7b's "the next
     time" asks between.
   - 48 bytes, inside the 72 that `the_event_keeps_its_size` pins. It has 5
     sites, none in a test: the emitter, the variant, the log line
     (`ui/display.rs:1091`), `for_each_object_named` and `from_record`. The
     history writer becomes its first reader.
   - *Set aside:* keeping a record per defending player and adding one when
     nobody blocks, two shapes for one step that 509.3g would have to join.
2. **Block rows on the combat history.** `CombatFact::Blocked(ObjectRef)`: a
   blocker's row names the attacker it was declared blocking, so "all creatures
   it blocked this turn" (Defiant Vanguard) has its answer, and "was blocked"
   reads the rows that name the creature. The rows share one `Arc<Vec>`, so a
   clone allocates nothing more, and the save writes the fact as a word, as it
   writes an attack.
3. **The "this turn" filter** (the decision's part (a)). `AttackedThisTurn`
   folds into it at 7 sites. **Its names, decided at review's second round**:
   the first draft's `Did { deed: ObjectDeed, during: DeedSpan }` would have
   confused at its call sites, and of the three shapes below the owner took
   the window as the variant, as the player's history already names it
   (`Condition::ThisTurn`, `LastTurn`, `SinceYourLastTurn`, `ThisGame`), with
   the fact in the card's own past tense:
   ```rust
   /// "Creatures that attacked this turn" (Relentless Assault): what the
   /// object did, read off the history, or off the permanent for its entry.
   /// A new object did nothing (CR 400.7).
   ThisTurn(ObjectFact),
   /// "If this creature attacked or blocked this combat" (Clockwork Beast).
   ThisCombat(ObjectFact),

   pub enum ObjectFact { Attacked, Blocked, WasBlocked, Entered }
   ```
   The three shapes, as two cards would write them:

   | Shape | Relentless Assault, "creatures that attacked this turn" | Clockwork Beast, "attacked or blocked this combat" | Its cost |
   |---|---|---|---|
   | **The window as the variant (decided)** | `And(Creature, ThisTurn(ObjectFact::Attacked))` | `Or(ThisCombat(ObjectFact::Attacked), ThisCombat(ObjectFact::Blocked))` | a variant per window: two now, and "attacked during your last turn" (Giant Turtle, 4 cards) a third with its card. Reads as the player's conditions do |
   | One leaf, fact and window as fields | `And(Creature, History { fact: ObjectFact::Attacked, during: Window::ThisTurn })` | `Or(History { fact: ObjectFact::Attacked, during: Window::ThisCombat }, History { .. })` | one variant for every window, and the longest call site |
   | A variant per fact | `And(Creature, Attacked(Window::ThisTurn))` | `Or(Attacked(Window::ThisCombat), Blocked(Window::ThisCombat))` | reads most like the card, but each of the census's dozen deeds is a new `ObjectFilter` variant and an arm in every exhaustive match over the filter (four today) |

   - `Attacked` reads only the declaration's rows (CR 508.4). `Entered` reads
     `entered_battlefield_turn`, or the `Lki` for a frame, which gains the
     turn. It has no combat window: the registry refuses
     `ThisCombat(ObjectFact::Entered)`.
   - Each deed the census lists beyond these four is an `ObjectFact` variant
     with its writer, added with its first card (the decision's part (a)).
     The combat rows become the object's rows when the first deed that is
     not combat's lands, and are renamed then.
   - The layer memo already gates the leaf on `Histories`
     (`engine/layers/board.rs:751`).
4. **The block arm**, as fields:
   ```rust
   /// CR 509.3a–g, read off the declaration (CR 509.1) at the dispatch, which
   /// is CR 509.3f's moment.
   Blocks {
       blocker: TriggerSubject,
       attacker: TriggerSubject,
       occurrence: BlockOccurrence,
       /// CR 509.3e: how many of the other side's creatures match.
       count: Option<CountIs>,
       /// CR 506.5: the only creature declared as a blocker.
       alone: bool,
       multiplicity: Multiplicity,
   },

   /// What one occurrence of a block trigger is (CR 603.2c).
   pub enum BlockOccurrence {
       /// CR 509.3a: each blocking creature, however many it blocks.
       Blocker,
       /// CR 509.3b, 509.3d: each blocking creature and each creature it blocks.
       Block,
       /// CR 509.3c: each blocked creature, however many block it.
       Blocked,
       /// CR 509.3g: each attacking creature no creature was declared blocking.
       Unblocked,
   }
   ```
   - **`occurrence` and `multiplicity` answer two questions** (asked at review,
     of the first draft's `per`). CR 603.2c: "an ability triggers only once
     each time its trigger event occurs. However, it can trigger repeatedly if
     one event contains multiple occurrences." `occurrence` is what one
     occurrence is for this arm, which CR 509.3a–d each state as "triggers
     once for each": a blocking creature, a block, a blocked creature, an
     unblocked one. `multiplicity` is whether each occurrence triggers
     (`PerOccurrence`) or one trigger takes all of an event's occurrences
     (`OncePerEvent`, "one or more"). So "whenever a creature you control
     becomes blocked" is `occurrence: Blocked` read `PerOccurrence`, and
     "whenever one or more creatures you control become blocked" the same
     occurrence read `OncePerEvent`. One declaration with three blocked
     creatures you control triggers the first three times and the second
     once. The word collides with nothing: the glossary's **occurrence** is
     already "what the arm says one of its events is", and
     `Multiplicity::PerOccurrence` counts them. Decided at review's second
     round. `Attacks` has the same field under TR-5e's name, `per:
     AttackEventPer`; renamed to `occurrence: AttackOccurrence` here, at 15
     sites, the two arms say it the same way (the decision's part (e)).
   - **Per blocking creature or per blocked creature, the other side's filter
     picks which of its creatures count**, and the event needs at least one,
     or `count`. "Becomes blocked by one or more blue and/or black creatures"
     is `occurrence: Blocked` with the blocker's filter. `CountIs` gains `Exactly`,
     CR 509.3e's "a particular number".
   - **The registry refuses** a blocker filter or a `count` with `Unblocked`,
     and a `count` with `Block`.
   - **Event roles.** `Blocker` is new. `Attacker` widens from a declared
     attacker to an attacking creature: a creature put onto the battlefield
     attacking can be blocked, and is unblocked for 509.3g. The triggering
     events are `Block { blocker, attacker }`, `Blocking { blocker, attackers
     }` (per blocking creature), `BlockedBy { attacker, blockers }` (per
     blocked creature) and `Unblocked { attacker }`. Only the two
     per-creature shapes hold a `Vec`, as TR-5e's `Defense` does, and
     `may_have_several` refuses "its power" of their several.
   - **"That player"** is none of the record's: "defending player" reads the
     attacker (`PlayerRef::DefendingPlayer`), as Hellrider's does.
   - **"That creature" across two arms** (the decision's part (d)). A new
     event role, `OtherCombatant` until the build names it, is the creature
     in the block that is not the ability's source: the attacker on an arm
     whose blocker is `ThisObject`, the blocker on one whose attacker is. The
     binding resolves it per arm, through the arm that matched. The registry
     refuses it on an arm with no `ThisObject` side, and refuses a def that
     names `Blocker` or `Attacker` where some arm of its condition fixes that
     side to the ability's own source: the self-destruction above.
   - **Read after the event, never looking back** (CR 509.3f). About 15
     exhaustive matches take the arm.
5. **A recipient that joins recipients.**
   ```rust
   /// "Destroy that creature and this creature": each part found as it is on
   /// its own, and every object they find acted on in one instruction (CR
   /// 608.2f).
   Together(Vec<EffectRecipient>),
   ```
   - **A list of recipients, not two antecedents** (asked at review: "would
     taking a vec be better?"). No printed join found names three objects,
     but the joins do name more than "that" and "this". Hunting Kavu exiles
     "this creature and target creature without flying that's attacking you",
     a target. Defiant Vanguard destroys "it and all creatures it blocked this
     turn", a set. Loyal Sentry and Dream Fighter join two antecedents. 10
     cards say "and this creature" or "and this permanent" beside a verb
     (`(o:"and this creature" or o:"and this permanent") (o:destroy or
     o:exile or o:sacrifice or o:return)`), and 15 "[verb] this creature and"
     (`(o:"destroy this creature and" or o:"exile this creature and" or
     o:"sacrifice this creature and")`). The list costs no clone: a card's
     effect sits behind its `Arc`, which a game's clone shares.
   - **Each part is found on its own**, as it would be alone: `ThisObject` by
     identity, `Triggering` off the binding, a filter swept now. One that
     finds nothing adds nothing, which is Loyal Sentry's ruling.
   - **One instruction.** The parts' objects reach the primitive as one target
     slice, so `Destroy` makes one batch: one event, and one "whenever one or
     more creatures die".
   - **Built narrow.** A part that is an instance of "target" or "choose" is
     refused by the registry until its first card (Hunting Kavu): its instance
     has to be declared in the walk's order and fizzle on its own (CR 608.2b),
     which no part needs here. A list of fewer than two parts is refused too.
     The bound reads are the parts'.
6. **Loyal Sentry**, registered and pooled. {W} 1/1 Human Soldier: `Blocks {
   blocker: ThisObject, attacker: Any, occurrence: Block, .. }`, then `Destroy`
   on `Together(vec![Triggering(Attacker), ThisObject])`.
7. **The targeting arm** (item 12):
   ```rust
   /// "Whenever [this creature/you] become[s] the target of a [spell/ability/
   /// spell or ability] [an opponent controls]": once per spell or ability,
   /// however many of its instances chose it (§3.12a, question 11).
   BecomesTarget { target: TargetSubject, by: TargetedBy, controller: Option<PlayerRef>, multiplicity: Multiplicity },

   pub enum TargetSubject { Object(TriggerSubject), Player(Option<PlayerRef>) }
   pub enum TargetedBy { SpellOrAbility, Spell(Option<ObjectFilter>), Ability }
   ```
   - **Event roles** `Target` and `SpellOrAbility`; the triggering event is
     `Target { target: ObjectOrPlayer, by: ObjectRef }`. The stack object is
     an `ObjectRef` like any other, so "that spell or ability" is found while it
     is on the stack, and is nothing once it has resolved, been countered or
     left (CR 400.7).
   - **"That player"** is the controller of the spell or ability: ward's "that
     player", Frost Titan's "its controller".
   - **One window per spell or ability.** `announce_targeted_records` opens one
     for its records, as the control sweep does.
   - **Valiant's "for the first time each turn"** (the decision's part (b)).
     Each `Targeted` record writes a row on the history: the object targeted,
     the turn, the targeting controller, and whether a spell or an ability
     targeted it. `FirstTimeEachTurn` on this arm asks whether an earlier row
     this turn would match the arm, so Heartfire Hero's "a spell or ability
     you control" ignores an opponent's targeting and Kira's ignores no one's.
     A new object starts with no rows (CR 400.7). `FirstTimeEachTurn` stays
     refused on every other arm whose record has no place in its turn.
   - `TargetSubject` has `CountersOn`'s shape under a name for its call site.
     The two stay apart, since each reads well only at its own arm.
8. **"Counter that spell or ability."** `Primitive::Counter` counters what it
   is handed, by what the stack object is: a spell into its owner's graveyard
   (CR 701.6a), an ability off the stack. Its two bodies are `CounterSpell`'s
   and `CounterAbility`'s, which keep their targets' kinds, and `CounterSpell`
   refuses an ability rather than moving it. Frost Titan's, Kira's and ward's
   instruction.
9. **Cephalid Aristocrat**, registered and pooled: {4}{U} 3/3, `BecomesTarget
   { target: Object(ThisObject), by: SpellOrAbility, controller: None, .. }`,
   then mill two. **Frost Titan, a fixture**: its first ability without "unless
   its controller pays {2}", `controller: Some(Opponent)`, then `Counter` on
   `Triggering(SpellOrAbility)`. Its first ruling is three tests (a spell, an
   Aura spell; an activated ability; a triggered ability), and its own
   controller's spell does not trigger it (ATOM-702.21a-002, partial).
10. **The activation arm**, with a mana ability's activation (the
    decision's part (c)):
    ```rust
    /// "Whenever [a player] activates an ability of [a permanent] [that isn't
    /// a mana ability]" (CR 602.2), off the record its costs announce.
    ActivatesAbility { player: Option<PlayerRef>, source: TriggerSubject, mana_ability: Option<bool> },
    ```
    - **Both producers write the record**: `activate_ability` as today, and
      `activate_mana_ability` after its costs and before it resolves
      (`engine/mana.rs:58`; CR 605.3b). Harsh Mentor's "if it isn't a mana
      ability" is `mana_ability: Some(false)`, and Ceaseless Searblades' arm
      leaves it `None`.
    - **CR 605.1b's leg.** A triggered ability that triggers on a mana
      ability's activation and could add mana is itself a mana ability:
      `is_mana_ability` accepts this arm with `mana_ability: Some(true)`
      beside `ManaAdded`.
    - The record gains where its source was, and the source's frame if a cost
      moved it: `AbilityActivated { identity, controller, from: Zone,
      source_lki: Option<Lki> }`, at 6 sites. Every producer writes
      `Battlefield` today. The arm reads only a source activated on the
      battlefield (Harsh Mentor #2), as it was then.
    - Event role `Object`, the source. "That player" is the activator (Harsh
      Mentor's "that player").
    - No `loyalty` field until CP-2's loyalty abilities.
    - Its fixture is Harsh Mentor's shape: an opponent's activation triggers
      it, its controller's does not, a source sacrificed as a cost still does,
      and the trigger resolves first (Harsh Mentor #3).
11. **Excess damage** (CR 120.10). *Moved to TR-5d at review's second round,
    as written, with the damage facts of part (a).*
    - **Decided with the batch.** After a batch decides and before it
      performs (`engine/actions.rs:1082`), each permanent the decided damage is
      dealt to gets the damage still lethal to it. For a creature that is its
      toughness less its damage marked, `lethal_damage_owed`'s reading, shared
      rather than copied, or 1 if a deathtouch source deals it any (CR
      702.2c). For a planeswalker it is its loyalty, and for a creature
      planeswalker the smaller of the two. Battles join with `backlog.md`
      §2.23, which owns their defense counters (CR 120.3h).
    - **A Gideon needs nothing of its own** (asked at review). The Gideons
      that become creatures prevent the damage dealt to them: Gideon Jura's
      and Gideon, Ally of Zendikar's "prevent all damage that would be dealt
      to him this turn", Gideon Blackblade's "during your turn". Prevented
      damage is never dealt (CR 615.6), and the lethal amounts are taken after
      the batch has decided its preventions, so there is no excess. Damage
      that can't be prevented (CR 615.12) is dealt, and the excess is CR
      120.10's greatest, from the smaller lethal amount. Indestructible
      changes neither amount: it stops a destruction (CR 702.12b), not the
      damage being lethal, and the loyalty still falls (CR 120.3c), which CR
      704.5i reads. Aegar's #6 is the test: a 5/5 creature planeswalker with
      indestructible and three loyalty counters, dealt 4 that can't be
      prevented, is dealt 1 excess.
    - **Each record takes its share.** The batch keeps the amounts on
      `GameState` while it performs, saved and restored for a nested batch as
      `prevention_allocations` is. Each `DealDamage` performer takes what of
      its amount is past what is still lethal, after the batch's earlier
      members to the same permanent. So a permanent's excess in one event is
      the sum of its records' `excess`, which is CR 120.10's "those sources
      together". `DamageDealt` gains `excess: u64` (18 literal sites), and
      `GameAction::DealDamage`'s 81 do not change.
    - **The arm.** `TriggerEvent::DamageDealt` gains `excess: bool`, "is dealt
      excess damage", whose "that much" is the excess. It is read once per
      permanent: the damage arm joins `supports_once_per_subject`, keyed on
      the recipient, which "one or more creatures deal combat damage to a
      player" will read too, and the registry refuses `excess` read per
      record.
    - ATOM-120.10-001 on a fixture ("whenever a creature is dealt excess
      damage, you gain that much life"), with damage already marked, a
      deathtouch source, two sources in one combat, a planeswalker, a creature
      planeswalker, and a prevention.
    - *Set aside:* the permanent's total on each of its records, which every
      reader would have to read once rather than sum; and a record of its own
      per permanent, which is no performed event (§3.3's contract).
12. **Item 275.** *Moved to TR-5d at review's second round, as written,
    with its three-arm sitting.* `SOURCE` compares the source (`origin.source_ref()`) where
    it compared the origin, and the source's last known information each
    entry carries, so a delayed trigger made after its source left is not
    elided with one made before. `AmountExpr::X` reads a new `BoundReads::X`,
    compared on `x_value`. Protean Hydra hit for three asks nothing at the end
    step.

**Planned shapes corrected here, with no decision**
- **§3.3's `Blocks { shape: BlockShape }`**, an enum of five readings,
  becomes fields, as `Attacks` did at TR-5e's decision 5. The enum had two
  spellings of one trigger ("this creature blocks a creature" and "a creature
  becomes blocked by this creature") and no place for 509.3e's count. What
  one occurrence is takes CR 603.2c's word, `occurrence`.
- **§3.3's `BecomesTarget { .., first_time_each_turn }`** drops the field:
  "for the first time each turn" is `TriggerDef::limit`'s, as on every other
  arm, and reads the targeting rows (piece 7).
- **§3.3's `ActivatesAbility { controller, source, loyalty }`** drops
  `loyalty` until CP-2's loyalty abilities, and gains `mana_ability`.
- **Item 275's fix** carries X.
- **`EventRole::Attacker`** is an attacking creature, not only a declared one.

**Not in this phase**
- Effects that make blocks (CR 509.3a's "as the result of an effect", 509.3c's
  "blocked by an effect", 509.4's creature put onto the battlefield blocking):
  4 cards, each writing its own record when it is built. An item, slotted with
  the first registered card.
- "If excess damage was dealt this way" (4 cards) and "was dealt excess damage
  this turn" (2): an amount and a history fact over `excess`, with the first
  registered card of each.
- The object deeds past the four, each an `ObjectFact` and its writer (the
  census table above; the decision's part (a)). One item lists them by
  writer, each slotted with its first registered card.
- An amount over a player's turn summary: Betor's "the amount of life you
  gained this turn" (31 cards) and "life you lost this turn" (6). A
  `HistoryCount`'s fact and players, read as a number rather than compared,
  about 15 lines of code and 30 of tests. Not this filter's, and not a record
  this phase reads, so an item of its own, slotted with the first registered
  card.
- CR 506.5's "blocks alone" (above).
- Ward (210) and every "counter unless": CP-1.
- "Copy that ability": CV-4 (item 256).
- Prevention's arm, Selfless Squire and the multiplier: TR-5d.

**Sized.** Code at 1.5–2× plus tests, counted against the tree on
2026-10-18, for the path decided at review's second round: (b), (c), (d),
"blocks alone" and `Attacks`' rename built here, and excess damage and item
275 moved to TR-5d.

| Piece | Code | Tests |
|---|---|---|
| 1–2. The record, one per step with the unblocked; its 5 sites; the block rows and their save word | ~60 | ~100: two defending players in one record; nobody blocks; a creature that entered attacking is unblocked (509.3g); blocked, then its blocker removed (509.1h); the save's round trip |
| 3. The "this turn" leaf, four facts and two windows; `AttackedThisTurn` folded; `Lki`'s entry turn | ~45 | ~90: each fact; a second combat (RE-10's) against the first; a new object; a frame's entry turn; Relentless Assault unmoved |
| 4. The block arm: four occurrences, the count, `alone`, `Blocker`, the other creature in a block (d), four triggering events, ~15 matches, the registry's refusals | ~185 | ~280: CR 509.3a–e and g on fixtures; 509.3f's characteristics at the declaration; "blocks alone" across two defending players; Cockatrice's two arms on a fixture; each refusal |
| 5–6. `Together` and its refusals; Loyal Sentry | ~60 | ~120: its ruling; one event for both deaths; an indestructible attacker; a target part refused |
| 7–9. The targeting arm, its player leg and its window, `Counter`, valiant's row (b), Cephalid Aristocrat, Frost Titan's fixture | ~200 | ~340: Frost Titan #1 three ways, and its controller's own spell (ATOM-702.21a-002); one spell choosing one creature twice; a player target; a triggered ability's target (CR 603.3d); valiant's first time by its controller, after an opponent's targeting (Heartfire Hero #1–#2 on a fixture); Kira's first time for any controller |
| 10. The activation arm and a mana ability's activation (c): the record at both producers, its zone and frame, CR 605.1b's leg, the fixture | ~85 | ~140: Harsh Mentor's shape; a mana ability's activation triggers "whenever you activate an ability" and fails Harsh Mentor's "if"; a source sacrificed as a cost |
| (e) `Attacks`' `per` renamed `occurrence` | ~15 | — |
| ~~11. Excess damage~~ and ~~12. Item 275~~ | to TR-5d | |
| **Whole: 2,045–2,370** | ~650 (975–1,300) | ~1,070 |

Against the row's 1,920–2,250: the window per spell, `Counter`, the
record's zone and frame, `Together`'s list, `Entered`, valiant, the mana
leg, the other creature in a block, `alone`, the rename and the refusals
are new, and excess damage and item 275 have gone. Built whole here, with
excess damage and item 275 kept, it would be 2,450–2,830, past the band.
Docs at the close-out, about 350 more: the stub and the archive,
`fuzz-record.md`'s block, §3.3's four rows, §3.12's item 11 row, and the
glossary's new event roles. TR-5f and TR-5b each landed past their sizing's
top, by 28% and 14%, so the build tracks the band at each commit and stops
at 2,500 to say so. If it gets there, the seam is activation (piece 10)
against blocks and targeting.

**Predictions for the close-out** (to be re-stated before `close_out.py`;
the usual sitting, since item 275's three arms go with it to TR-5d)
- *The engine arm, against `main`.* Every gameplay row identical, both
  pools, two seats and four: no registered card reads a new arm, and
  Relentless Assault's leaf answers as before. The event dumps move: one
  `BlockersDeclared` line per declare-blockers step where `main` writes one
  per defending player who blocked, so a new line in every combat with no
  block; an `AbilityActivated` line for every mana ability activated; and the
  targeting records share a window, so their batch numbers move. The new
  kinds reach the gate, so the dispatch counters move. Instructions per
  decision +0.1% to +0.5%, most of it the mana abilities' records.
- *The shipped arm.* Every dealt game moves once Loyal Sentry and Cephalid
  Aristocrat are pooled. Their stress distortion is measured and reported at
  two seats and four, as TR-5b's decision 4 did for Simic Ascendancy: Loyal
  Sentry trades with whatever it blocks, and Cephalid Aristocrat mills its
  controller each time it is targeted, which could deck a long game.

**Decision**

1. **Scope, and where each family goes — decided at review's second round
   (the owner, 2026-10-18): the path as proposed.** Asked again at review's second
   round (the owner: why push valiant, is there a more natural slot; isn't
   the mana work Wild Growth's; what is (d)'s "that creature"). The first
   round sorted the four families by the band. Asked instead which phase
   each belongs to, (b), (c) and (d) belong here, and of (a) only the damage
   facts have a phase, TR-5d:

   | Part | What it needs | Its natural slot | Decided |
   |---|---|---|---|
   | (a) The damage facts: "was dealt damage this turn", "dealt damage to", 45 cards | rows written off `DamageDealt` | TR-5d, which reads the damage batch's other record (`DamagePrevented`, CR 615.13) | TR-5d, with excess damage |
   | (a) The zone-change facts: "died", "discarded", "milled", "put into a graveyard" this turn, 52 cards | rows written off the zone change, keyed by the object the move made (CR 400.7) | none: no phase on the route reads them | its first registered card, one item |
   | (a) The rest: "was cast", "crewed" or "saddled", "came under your control", "turned face up", 26 cards | a writer each | crew and saddle with their keywords; face up with CV-6 | its first registered card, the same item |
   | (b) Valiant and Kira's "for the first time each turn", 17 cards | a row per targeting record: the object targeted, the turn, and the targeting controller, which the limit reads | **this phase**: it is the targeting arm's own limit, over the record this phase reads, and the row is a fact of this phase's history | now |
   | (c) A mana ability's activation | `AbilityActivated` written for a mana ability too | **this phase**: it is the record this phase reads, left incomplete | now |
   | (d) "That creature" across two block arms, 32 cards | an event role for the other creature in a block | **this phase**: it is the block arm's | now |
   | (e) `Attacks`' `per` | a rename to `occurrence: AttackOccurrence`, 15 sites, no behavior | this phase, which names the same field on the block arm | now |

   - **Valiant was pushed only for the band.** Its rulings make it a fact of
     the targeting record: Heartfire Hero's "will trigger if that creature
     hasn't yet been the target of a spell or ability you control this turn"
     (#3, #4). So each targeting record writes a row, the limit asks whether
     an earlier row this turn matches the arm, and Kira, which names no
     controller, asks about every row. Changed targets (CR 115.7) and copies
     (CV-4) write rows when they are built, as they announce `Targeted`.
   - **(c) is not Wild Growth's work.** Wild Growth's "whenever enchanted land
     is tapped for mana" reads the mana the ability *produces* (`ManaAdded`,
     CR 106.12a, "whenever such a mana ability resolves and produces mana"),
     which TR-1 built with CR 605.1b's triggered mana ability. Activating the
     ability is a separate, earlier event (CR 605.3b: activate, then it
     resolves at once), and nothing announces it. `ManaAdded` cannot stand in:
     Dark Ritual, a spell, writes one, and so does a triggered mana ability,
     and neither is an activation. So "whenever you activate an ability of an
     Elemental" (Ceaseless Searblades) misses an Elemental's mana ability, and
     Harsh Mentor's "if it isn't a mana ability" never has one to exclude.
     The fix is one record per mana ability activated, after its costs and
     before it resolves (`engine/mana.rs:58`), and the cost every game pays is
     that record: about as many as `ManaAdded`'s, +0.1% to +0.3% instructions
     per decision predicted.
   - **What (d) is.** Cockatrice: "Whenever this creature blocks or becomes
     blocked by a non-Wall creature, destroy that creature at end of combat."
     When Cockatrice blocks a Grizzly Bears, "that creature" is the Bears, the
     attacker. When Cockatrice attacks and the Bears blocks it, "that
     creature" is the Bears again, now the blocker. The ability is two block
     arms, one for each half of "blocks or becomes blocked by", and its one
     instruction names one event role. `Attacker` is right for the first half
     and is Cockatrice itself in the second, so the card destroys itself.
     The fix is an event role the binding resolves per arm: the creature in
     the block that is not the ability's source.
   - **The path, to keep both phases in the band.** With (b)–(e) and "blocks
     alone" here, TR-5c built whole would be 2,450–2,830. Two of its pieces
     sit more naturally in TR-5d. Excess damage (piece 11) is decided in the
     damage batch's decide step, beside CR 615.7's prevention allocations,
     and TR-5d builds prevention's arm over the same step's
     `DamagePrevented`; the damage facts of (a) join it there. Item 275
     (piece 12) needs a phase that moves the random agent's stream, and TR-5d
     does (Selfless Squire, Panharmonicon), so its three-arm sitting serves.
     **TR-5c then sizes at 2,045–2,370, and TR-5d at 1,995–2,300** (its
     1,470–1,700, with excess damage ~295–340, the damage facts ~120–140 and
     item 275 ~110–120). Item 275's slot and the brief's "items 11, 12 and
     275 closed" move with it.

   The other way, round one's: keep excess damage and item 275 here and
   refuse (b), (c) and (d) until their first cards, at 2,085–2,400 with
   "blocks alone" kept. Each refusal files an item with a slot no later
   phase owns.

### As landed (2026-10-18)

**Built as designed**, with the decision as review's second round took it.
Five code commits: the record and the "this turn" leaf; the block arm, the
other creature and `Together` (`Group` since review); the targeting arm,
valiant and `Counter`;
the activation arm and a mana ability's record; the log. Each card was
registered in the commit after the engine played it. Two more came from
the close-out's model pass: two fixes, then eight copies merged.

**What the build corrected.**
- **The record says whether its ability is a mana ability.** The design gave
  `AbilityActivated` its zone and its frame, but the arm's `mana_ability`
  reads which kind of ability was activated, and one source can have both.
  Each producer writes `mana_ability`.
- **The log leaves a mana ability's activation implicit.** Its tap and its
  mana already say it, and every payment would add a line; the full event
  log keeps it. The dev GUI's why panel counts one more batch per payment,
  so two of its pictures were redrawn (`why_event`, `why_prevented`).
- **The save reports a block row and a targeting row, and writes neither.**
  The design had the save write a block as a word, "as it writes an
  attack", but the save writes no combat row: it reports any row of the
  turn in progress (`scenario/write.rs`), which covers a block with no
  change. A targeting row is reported the same way.
- **One fixture's deal moved.** `prompt_subject_test`'s fourth game lost
  its blockers once Loyal Sentry's registration moved the registry's deal,
  and plays a new seed.

**At the close-out.** Every prediction held: the engine arm read IDENTICAL
on every table, at +0.15% instructions per decision. The 50-game two-seat
stress table read average turns 26.8 → 23.8 against `main`; 400 games at
two seeds read 24.7 → 24.5 and 26.3 → 26.6, the deal's noise. The audited
smoke's one game at the turn limit is seed 810's Words of Worship standoff,
recorded twice before.

**The model pass** (`plans/references/duplicate-inventory.md` §3.7) found
two readings this phase wrote in one place and missed in another, each
fixed and shown failing first: a source sacrificed for its own cost was
matched off its record's frame but read without it at the trigger and at
resolution (T17), and a block arm's `subject` named its blocker even where
the text names the attacker (T18). Eight copies merged with no game moved,
50 audited games on both pools at two seats and four reading identical
against the shipped arm; the rest are kept with a reason or are item 247's.

**Review, round three (2026-10-18).** Re-reading CR 605.1b for the owner's
question on Crumbling Vestige found the leg built against the arm: "it
triggers from the activation or resolution of an activated mana ability"
is the triggering event's to say, so "whenever you activate an ability,
add {C}" went on the stack off a land's mana ability. `is_mana_ability`
takes the record, and `could_be_a_mana_ability` is the def's half for the
mana supply; the test failed first. The activation's record names its
source as `source_before_paying` reads it, its zone included, which
withdrew item 282, and which costs can move a source is
`Cost::can_move_its_source`, an exhaustive match. Seven names moved for the
call site (`lki_now`, `Group` and `resolve_group_member`,
`GameEvent::source_lki`, `rows_during`, `role_as_recorded`,
`TriggerBindingReads`, `can_tell_a_first_time_each_turn`), and two tests
joined: an attacker that entered attacking can be blocked (CR 509.3g's
converse), and an ability that adds mana with a target is no mana ability
(CR 605.1a, Deathrite Shaman's shape).

**Review, round four (2026-10-18).** The owner's question on `source_lki`'s
two records found a third: a mana source its own cost moved (Crystal Vein,
a Treasure) has gone when its mana is made, and the production read it in
its owner's graveyard, so a stolen land's mana missed "a land you control"
in a trigger and in Deep Water's replacement. The production carries the
activation's LKI (`ManaProducer`, `GameAction::ProduceMana.source_lki`,
`GameEvent::ManaAdded.source_lki`); both tests failed first, and no game
moved. Item 279 took the per-turn history's shape for TR-5d.

**What it leaves.** Items 278–281 and 283 (`codebase-state.md`, "Found by
TR-5c"). Excess damage, the "this turn" damage facts and item 275 are
TR-5d's, with pieces 11 and 12 above as written. CR 509.3a against 509.3b
on one creature blocking two is item 6's, with the first card that lets a
creature block an additional one.

**Size.** +2,630/−354 in code and tests (code +1,384/−345, tests
+1,246/−9), against 2,045–2,370, past the band by the review's rounds:
round three's rename commit is +188/−170, lines renamed in place, and round
four's mana fix +140/−48. Forty-five integration tests in
`phase_tr5c_integration_test` and one unit test in `state::history`.

#### TR-5h — one history of what happened — ✅ landed 2026-10-19

*Evicted 2026-10-19 from `plans/triggers-architecture.md` §12, where the heading and a stub remain.*

### The plan as designed (TR-5h, as §12 carried it on 2026-10-19)

*Raised at TR-5d's design review (the owner, 2026-10-19): "We've done a few
abstractions already with respect to game history, are we sure this is the
end of that? I'd rather write it once and get it right than continually
rewrite it." Designed the same day against the same tree, for review with
TR-5d and TR-5g. It lands first, and TR-5d's damage facts are its first new
rows. Nothing built.*

**Why the history keeps being rewritten.** Five phases have reshaped it,
each for a reader its store could not hold:
- TR-2a counted each player's facts in three summaries (this turn, last turn,
  this game), and item 179 bounded them.
- A6r gave a draw its place in its turn.
- TR-5b joined the players' rows into one struct and shared the "since your
  last turn" snapshot.
- TR-5f and TR-5c added rows for what creatures did in combat, and rows for
  targeting.
- TR-5d's first draft generalized the combat rows for damage.

Each fixed the reader in front of it. This section asks every printed reader
at once.

**The census** (Scryfall, 2026-10-19; `plans/references/history-census.py`
re-runs the second row's count):

| What a reader asks | Examples | Today's store | Holds it? |
|---|---|---|---|
| A count of a player's events in a span | "for each spell you've cast this turn" (Aetherflux Reservoir); raid; "the life that player lost this turn"; "if no spells were cast last turn" (the werewolves) | `TurnSummary`'s slots | yes |
| A count, total or aggregate over what the things were at the time | "for each Zubera that died this turn" (Ashen-Skin Zubera); "if you sacrificed three or more Clues this turn" (Briarbridge Patrol); "a player who was dealt combat damage by three or more Pirates this turn" (Admiral Beckett Brass); "the number of Treasure tokens you created this turn" (Vazi, Keen Negotiator); "the total mana value of other spells you've cast this turn" (Call Forth the Tempest); "each card type among spells you've cast this turn" (April O'Neil, Hacktivist). **124 cards** over eleven phrasings, a lower bound | none. A count has no characteristics. `SpellsCastOfType`'s fifteen slots are card types alone, and six of them are types no spell has. A subtype, a color, a mana value or a name is no slot at all | **no** |
| An object's own past | TR-5c's census, about 210 cards; "the first time +1/+1 counters have been put on that permanent this turn" (Botanical Brawler); "the first time that creature has become tapped this turn" (Captain America, Living Legend); "target planeswalker that was activated this turn" (Cut Short) | the combat rows: attacked, blocked, removed | three facts of it |
| The order of events | "the first spell you cast each turn"; "your second card each turn"; valiant; "the last red instant or sorcery spell that dealt damage to you this turn" (Suffocation) | a record's place in its turn, for casts, draws and life; the targeting rows' order | for those kinds |
| Last turn, your last turn, since your last turn | the werewolves; Giant Turtle; "a nonblack creature that entered since your last turn ended" (Premature Burial); "if you haven't been dealt combat damage since your last turn" (Invasion of Fiora) | the players' last turn and the seats × seats snapshot; the combat rows, kept a round | players' counts, and objects' combat facts |
| This game | 17 count casts of a commander from the command zone; Approach of the Second Sun, Yidaro and Gollum count by name; Flourishing Forces' life gained; First Contact's first spell; Rampaging Ursaguana's "attacked this game"; The Fallen's "dealt damage to this game"; Echogoyfs' "not from your starting deck" | `this_game`'s slots | counts only |
| The abilities' own facts | "the second time this ability has resolved this turn" (28 cards); "only once each turn" | three per-turn sets | yes |

The second row is what the next rewrite would be for. No number of slots
answers it, and every slot costs every clone.

**The shape, written once: the game's log** (decided at review, the owner,
2026-10-19). One store answers every span a card can name, and the
abilities' gates stay as they are.
1. **The game's log.** It records events, never the board: a row is one
   thing that happened, and its frame is the one object's characteristics
   then, behind a pointer. Every event a reader of the past can ask about is
   one row, kept for the whole game. A row is the event as it happened: its
   turn and combat, whose event it was, the object it was about as the
   existence it was (CR 400.7), that object's frame then, and what happened.
   ```rust
   /// One event a reader of the past can ask about, as it was then.
   pub struct HistoryRow {
       /// The record it was written from, which orders the log: "the first",
       /// "the last", "for the first time each turn".
       pub seq: EventSeq,
       pub turn: u32,
       /// Which of the turn's combats, from 1; 0 outside combat.
       pub combat: u32,
       /// Whose event: the caster, the drawer, the player whose life moved or
       /// who was dealt damage, who controlled the creature as it died.
       pub player: Option<PlayerId>,
       /// The object it was about, as the existence it was.
       pub object: Option<ObjectRef>,
       /// That object as it was then (CR 608.2h): a record's frame where it
       /// carries one, else the object's frame as the event left it.
       pub frame: Option<Arc<EffectiveCharacteristics>>,
       pub fact: RowFact,
   }

   /// What happened. A fact a later card asks is a variant and an arm in the
   /// writer, never a new store.
   pub enum RowFact {
       Cast { from: Zone },
       Drew,
       GainedLife(u64),
       LostLife(u64),
       Entered { from: Option<Zone> },
       /// Died, discarded, milled, sacrificed, exiled: the move and its act.
       LeftZone { from: Zone, to: Option<Zone>, act: ZoneChangeAct },
       DealtDamage { to: ObjectOrPlayer, amount: u64, combat: bool },
       Attacked(AttackTarget),
       Blocked(ObjectRef),
       RemovedFromCombat { attacking: Option<AttackTarget> },
       BecameTarget { by_spell: bool, controller: PlayerId },
   }
   ```
   - **Why the whole game, not a round.** A round is the longest span a
     printed card names short of "this game": the 11 that say "your last
     turn", "their last turn" or "since your last turn" (Giant Turtle, Halls
     of Mist, Premature Burial) reach back up to a full rotation. "This game"
     has no shorter store that is always right. Counts per fact miss a
     filter ("the first creature spell you've cast this game"), and counts
     compiled at setup from the cards in the game miss a chosen creature
     type, a text change (CR 612) and a card that joins from outside the
     decks, silently. Only the whole log is right for every card, printed or
     custom.
   - **The rows today's readers need**, and the damage rows TR-5d's facts
     read: casts, draws, life, entries, zone changes, damage, combat and
     targeting. Counters, taps, activations, token creations, cycling and the
     rest each join with their first card, as a variant.
   - **Stored as a chain of fixed chunks, each behind an `Arc`**, appended
     within the newest. A clone copies one pointer, the chain's head, however
     long the game. The first write after a fork copies only the open chunk,
     at most sixteen rows, and every full chunk stays shared by every clone in
     a search. A read walks back from the newest chunk and stops where its
     span starts. "This game" walks the whole log, about 600 rows at our fuzz
     games' average, and only a "this game" reader does.
   - **The frame is one the engine already has.** A departure's record
     carries it, and an object still where the event left it has the layer
     memo's (an `Arc` clone, with no walk while it is current).
   - **Never truncated.** If what a game holds ever needs to shrink, two
     storage changes keep every answer and touch no reader: rows older than a
     round swap their frame for a compact copy of what filters read (332
     bytes to about 64), and identical frames are shared (a thousand
     identical Treasure tokens, one frame). Each waits for a measurement that
     asks for it.
2. **The abilities' gates**, unchanged: what has triggered, been done once,
   or resolved this turn, by ability. They are rule gates keyed by an
   ability, and no reader filters them.

`TurnSummary` and its three views, and the seats × seats snapshot, go:
this turn, last turn, your last turn, since your last turn and this game are
all spans of the log. `PlayerHistory` keeps the turn each player last began
and the draws in a draw step in progress (CR 504). `GameHistory.combat` and
`.targetings` fold into the log.

**Item 179's bound changes** (the owner, 2026-10-19). It made the history
"the table's size, never the turn count's". That still holds for what a clone
copies, which is one pointer. It no longer holds for what a game holds: the
log grows with the game's events, by about 3.5 KB a turn at four seats (the
cost table below), held once and shared by every clone. Each close-out
reads it, on the longest fuzz game and on TR-7's persist loop.

**One way to read it.** Every reader asks the log the same question, with
the span it names:
```rust
/// "[Players'] [fact] [of what] [span]": the rows that answer, read as a
/// count, a total of their amounts, or "any".
pub struct HistoryCount { pub players: PlayerSet, pub fact: TurnFact, pub of: Option<ObjectFilter>, pub is: CountIs }
```
- **`TurnFact` stays the vocabulary of what is counted**, and gains `of`, a
  filter over each row's frame: "noncreature spells you've cast this turn" is
  `SpellsCast` of `Not(ByType(Creature))`. `SpellsCastOfType(t)` becomes
  `SpellsCast` of `ByType(t)`, so its fifteen slots go.
- **`Condition::ThisTurn`, `LastTurn`, `SinceYourLastTurn` and `ThisGame`
  keep their names and their cards**, and all four read the log.
- **`ObjectFilter::ThisTurn` and `ThisCombat`** read the rows that name the
  object, as TR-5c built them, with TR-5d's damage facts.
- **An `AmountExpr` reads a `HistoryCount` as a number**: "the amount of
  life you gained this turn" (Betor, Ancestor's Voice; item 280). The
  aggregates ("the greatest mana value among", "card types among") are
  further readings of the same rows, each with its first card.
- **A record's place in its turn** is counted off the log as the row is
  written, so "the first spell you cast each turn" reads what it reads today.
- **`backlog.md` §2.47's "this game" counts are readings too**, each with its
  first card: a commander's casts from the command zone (B2's designation),
  a count by name (Approach of the Second Sun, Yidaro, Gollum), an object's
  lifetime facts (Rampaging Ursaguana, The Fallen). None needs a store.

**The layer memo's gate**, keyed by the rows a read touches (item 279): a key
for each kind of row, the combat count and the turn number, in place of
today's one `GatedWalkInput::Histories` (`state/layer_memo.rs:75`), with
`GatedInputs` widened from a `u8` to a `u16`.
- **Noted inside the log's lookups.** Each lookup takes the walk's note,
  notes its own kind of row, and notes the combat count when it asks for the
  latest combat. So a filter over two facts notes both, and no caller can
  leave one out. `Condition::DefendingPlayer` notes the removal rows through
  `attacking_when_removed`, never by name.
- **Bumped by kind.** The writer bumps the keys of the rows it wrote, and a
  turn's start bumps the turn number. The memo audit, which re-walks fresh
  against each hit in a debug build, catches a key a lookup misses.

**The scenario** (`setup-architecture.md` §5.1). Today it writes each
player's three summaries as count words (`scenario/text.rs`). A count word
for a fact no filter narrows, such as life gained, writes rows with no
object. A fact a filter can narrow names its objects ("cast this turn" and
the cards), and reads them back with their printed frames. What it cannot
say, it reports, as it reports a combat row today (`scenario/write.rs:194`).

**What it costs, against today** (a size probe, a frame measured over 9,291
samples, and 60 four-seat games in both pools, 2026-10-19):

| | Today | The log |
|---|---|---|
| Bytes a four-seat clone copies for the history | 2,464: four players × 616, each with three 192-byte summaries of 24 counts | about 100: four players' turn and draw-step counts; the log is one pointer |
| Allocations a clone makes for it | 1, the players' `Vec` | 1 |
| A fork's first write | the summaries in place; a whole round of combat or targeting rows (48 bytes each) once one is written | the open chunk: at most 16 rows of about 80 bytes, 1 allocation |
| A new fact | 96 bytes on every four-seat clone (a slot in three summaries, four players). A fact by characteristic needs a slot per value, or cannot have one | a variant: nothing on the clone, and a row only when it happens |
| Held once per game, shared by every clone of it | the combat and targeting rows, about 1–2 KB | about 8–10 rows a turn of these kinds (the games wrote 42.7 records a turn, p90 65), each about 80 bytes, and the frame each keeps alive, 332 bytes on average where nothing else shares it: at most about 3.5 KB a turn. Over a whole game, 220–250 KB on average and about 400 KB at most (the longest game 118 turns, the busiest turn 116 rows). The clone floors do not count it, since no clone copies it |

A read of "this turn" scans its turn's rows of one kind, about ten, where a
count read one slot. The readers are a static's condition, which the layer
memo holds, and a trigger's "if", so floor 1 is not expected to move. Taking
a live object's frame for its row is the new per-record work.

**The first commit is a measurement gate.** It builds the log's storage and
its writer for today's kinds, and moves no reader. Its sitting reads:
- bytes and allocations per clone on `clone_bound_test`'s board;
- instructions per decision on the fuzz tables, against `main`;
- what a game holds at 74, 118 and 200 turns.

The build stops and reports if a clone costs more than it does today, if
instructions per decision rise past +1.5%, or if a game holds more than
about 5 KB a turn. Only then do the readers move.

**What it closes.** Item 279's damage half, since TR-5d's facts are rows
here, and its shape for every other fact: each later one is a variant and a
writer arm. Item 280. And `backlog.md` §2.47, whose counts become readings of
the log with their first cards.

**Planned shapes corrected here, with no decision**
- **§3.10's `TurnSummary` and its three views** go; every span is the log's.
- **TR-5c's "the combat rows become the object's rows when the first deed
  that is not combat's lands"** happens here, for every reader at once.

**Not in this phase**, each with its first card: the aggregates ("greatest",
"total", "among"); counters, taps, activations, token creations and cycling
as rows; the "this game" readings above. "Dealt damage to it" and a source as
it was then (Brine Hag, Giant Albatross, Aegar, the Freezing Flame) are a
filter over a damage row's frame and its recipient, which the shape holds;
their leaf comes with the first card. The two storage changes wait for a
measurement that asks for them.

**Sized.** Code at 1.5–2× plus tests, counted against the tree on
2026-10-19. The lines it deletes (`TurnSummary`, the snapshot, the type
slots, `CombatHistory`, `TargetingHistory`) are not counted against it.

| Piece | Code | Tests |
|---|---|---|
| The row, its facts, the chunk chain and its spans | ~140 | ~780 across the phase: today's history tests moved onto the log (~150 lines changed); a filtered count read off a frame after the object changed; since your last turn at four seats; last turn; this game; a clone shares the log and a fork's write copies one chunk; the gate, twice; the scenario's round trip by card name, twice; a place in a turn; Betor's amount; the held-memory reading at the gate; the damage facts (~150, from TR-5d) |
| The writer: today's facts as rows, frames, places | ~100 | |
| The readers: `HistoryCount`'s `of`, the span conditions, the filter's facts, the targeting limit, the defending player, the amount | ~110 | |
| The gate by kind of row | ~55 | |
| The scenario's words | ~100 | |
| TR-5d's damage rows and its two facts, `WasDealtDamage` and `DealtDamageTo { player, combat }` | ~40 | |
| The held-memory reading, beside `clone_bound_test` | ~25 | |
| **Whole: 1,630–1,920** | ~570 (855–1,140) | ~780 |

Docs at the close-out, about 400: the stub and the archive, `fuzz-record.md`'s
block, §3.10 rewritten to the log, item 179's bound restated,
`setup-architecture.md` §5.1's words, and the glossary's history entry.

**Predictions for the close-out** (re-stated before `close_out.py`; it moves
what a clone holds, so its sitting carries a cost arm)
- *The engine arm, against `main`.* Every gameplay row identical, both pools,
  two seats and four: every reader answers as before, and a record's place
  in its turn is the same number. Instructions per decision +0.3% to +1%,
  from a row and a frame per logged record. Floor 3 about 2.3 KB lower at
  four seats, and floor 2 unchanged. A game holds about 3.5 KB a turn.
- *No shipped arm*: no card is registered.

### As landed (2026-10-19)

**Built as designed, its gate in two steps.** Ten code commits on the design
PR, #262: the log written beside the old stores, and its measurement gate;
the draw rows' frames, dropped and then restored; the combat's and the
targeting's readers, with the gate by kind of row; the players' counts; the
scenario through the log, and the summaries deleted; the window's own rows in
its frames; `of`, the amount and the damage facts, with the scenario's named
casts; the model pass's round.

**What the build corrected.**
- **The gate's draw-row frames.** The design kept every row's frame. To win
  back instructions the build dropped the draw rows' (the owner's option C,
  conditional on what it would catch), predicted at about a point. It
  measured 0.07, and elimination arms put 0.93 points on the entry rows'
  frames instead. The owner approved exact frames on the condition that the
  log not bring back the memory he had worried over: held stayed at 4.6 KB a
  turn at most, the clone unchanged.
- **A row's frame sees its own window's rows.** Commit 3 took the frames
  after the window's counts, so a static reading this turn's history saw the
  record that changed it (CR 603.10). Once the counts moved onto the log, the
  rows were written after their frames and that was lost: a lifelinker's
  damage row read the creature without the "+1/+0 as long as you've gained
  life this turn" its own gain turned on. The gate already says when it
  matters, since a bump of a kind the window wrote moves the epoch only if a
  frame read it: then the window's frames are taken again and written onto
  its newest rows. A first version collected the window's seqs into a `Vec`
  and read +0.31 points with no second pass run; reading them in place costs
  nothing.
- **"Defending player" in a layer row's filter noted nothing.** It read the
  combat without a gate note; `defending_player` takes the walk's note.
- **The comparison left the count.** The design wrote
  `HistoryCount { players, fact, of, is }`; an amount has no `is`, so
  `Condition::ThisTurn(count, is)` compares and `AmountExpr::History(span,
  count)` reads, one question for both.
- **The scenario names casts, and only casts.** The design's sentence,
  counts for what no filter narrows and the cards by name for the rest, is
  built where the typed `<type> spells cast` words stood: `cast <card>` is a
  row with the printed card's frame. Draws, deaths, attacks and damage stay
  counts (item 287). A stated count is a row of the kind it stands in for,
  `RowFact::Stated`, and a card that narrows one is loud in a debug build.
- **The gate's mask is a `u32`**, five keys and eleven kinds of row, where
  the design said a `u16`.
- **The returns name their parts** (`engineering-practices.md` §2b, rule 10):
  `Targeting` and `RowCount` for the log's lookups, `NamedCast` for the
  loader's.

**At the close-out.** Every gameplay prediction held: the engine arm read
IDENTICAL on both pools at two seats and four, the audit agreeing. Instructions
per decision read +1.37% against `main`, past the predicted +0.3% to +1%; the
entry rows' frames, which the cost table named as the new per-record work,
are most of it. Floor 3 read 2.2 KB lower per clone on average over the clone
test's 190 readings (predicted about 2.3), its worst 93.5 → 91.2 KB;
allocations unchanged, 39 at worst. A game holds 2.7–4.6 KB a turn, 378 KB in
the largest of `history_held_test`'s 23 (predicted about 3.5 KB a turn). The
audited smoke's one game at the turn limit is seed 810's Words of Worship
standoff, recorded before.

**The model pass** (`plans/references/duplicate-inventory.md` §3.8) found
three faults, each fixed and shown failing first: a static's history amount
read "you" off its registry row, so a stolen source counted its old
controller's (CR 109.5; the hand-size amount shared it); a history count's
filter dropped a gone source's last known owner and could panic on "its
owner"; and "your last turn" was written after `begin_turn` rather than by
it. The two history totals merged into `GameHistory::total`. The defending
player of an attacked planeswalker or battle went to `backlog.md` §2.23.

**What it leaves.** Items 286 and 287 (`codebase-state.md`, "Found by
TR-5h"); item 279's zone-change facts and the rest, each with its first
card. TR-5d builds on the damage row next.

**Size.** +2,058/−870 in code and tests (code +1,463/−808 with the dev GUI's
two arms, tests +595/−62), against 1,630–1,920 with the deletions uncounted:
code past its 855–1,140 by the scenario's named casts and the reframe, tests
under their ~780. Twelve integration tests in `phase_tr5h_integration_test`,
the release-only `history_held_test`, and unit tests in `state::history_log`,
`state::history` and `state::layer_memo`.

#### TR-5d — prevention and excess damage — ✅ landed 2026-10-19

*Evicted 2026-10-19 from `plans/triggers-architecture.md` §12, where the heading and a stub remain.*

### The plan as designed (TR-5d, as §12 carried it on 2026-10-19)


*Designed 2026-10-18 against the tree at `336a0f7` (`main`, #261 merged).
Re-counted first: the row's 1,995–2,300 counted neither item 279's history
nor what the card hunt found, and the whole re-counts at 2,660–3,040, so it
split in two at the brief's seam, with TR-5g below. At review (2026-10-19)
the history became its own phase, TR-5h above, and this phase keeps the
damage batch's two records and item 275. Nothing built.*

**Where it sits.** A trigger reads the performed stream. A record is
announced, `EventKind::from_record` files it, the dispatch matches each
source's arms, the binding keeps each object by its event role, and
placement puts the trigger on the stack. The damage batch writes two
records, and neither reaches an arm whole:
- `DamagePrevented` has been announced while the batch decides since #260
  (`engine/replacement/pipeline.rs:673`, item 16), and is filed as unread
  (`types/triggers.rs:571`). Selfless Squire's second ability reads it.
- `DamageDealt` says how much damage was dealt, and not how much of it was
  excess (CR 120.10).

This phase builds the prevention arm and CR 120.10's excess, in the batch,
on the damage arm and on TR-5h's damage row. Item 275 rides with it, since
registering Selfless Squire moves the random agent's stream.

**What the re-read found** (CR 615.13, 120.10, 120.4a, 702.2c and 603.10,
the cards' rulings, and Scryfall's counts on 2026-10-18)

*Prevention*
- **One printed card triggers on CR 615.13's event.** Selfless Squire:
  "Whenever damage that would be dealt to you is prevented, put that many
  +1/+1 counters on this creature." Its first ability's prevention lasts
  after the Squire leaves (#1), and any prevention effect triggers the
  second, not only its own (#2). The other five with "is prevented" in a
  trigger say "prevented this way" (Phyrexian Vindicator, New Way Forward,
  Outfitted Jouster, Judgment of Alexander, Samite Ministration; `(o:"is
  prevented" or o:"are prevented") (o:whenever or o:when)`). They ask which
  effect prevented the damage, which is the record's `by`.
- **One record is one application.** CR 615.13 triggers "each time a
  prevention effect is applied to one or more simultaneous damage events".
  The pipeline applies an effect once per subject group, CR 616.1's affected
  object or player (`replacement-architecture.md` §9's RD decision 3), and
  #260 writes one record each time. So two attackers into a Fog at you are
  one record and one trigger, whose "that many" is their total. A shield and
  a Fog that each prevent part of one damage are two records. A Fog that
  prevents damage to you and to your creature is applied twice, once to
  each, and only the first is "dealt to you".

*Excess damage* (TR-5c's piece 11, kept in the archive as written, re-checked)
- **The readings** (CR 120.10, and Aegar, the Freezing Flame's rulings).
  Damage already marked counts (#2). Any damage from a deathtouch source to
  a creature is lethal, so the rest of it is excess (#4; CR 702.2c, 120.4a),
  and deathtouch changes nothing for a planeswalker (#4). A creature
  planeswalker takes the smaller lethal amount (#6). And excess is a fact of
  the event, "those sources together": two sources into one creature make
  one excess.
- **Taken before the batch performs.** A planeswalker's loyalty falls in a
  removal nested after its damage (`engine/actions.rs:1466`), and damage is
  marked member by member. So the lethal amounts are taken once the batch has
  decided its preventions and before it performs (`engine/actions.rs:1082`).
  Prevented damage is never dealt (CR 615.6), so a Gideon that prevents the
  damage dealt to it is dealt no excess. Damage that can't be prevented is
  dealt: Aegar #6's 5/5 creature planeswalker with three loyalty counters,
  dealt 4, is dealt 1 excess, indestructible or not.
- **Who reads it** (12 cards, `o:"excess damage" (o:when or o:whenever)`):
  the arm, "is dealt excess [noncombat] damage" (Aegar, Fall of Cair Andros);
  "was dealt excess damage this turn" (Maarika, Brutal Gladiator; Overclocked
  Electromancer), TR-5h's damage row with its excess; and "if excess damage
  was dealt this way", a resolution's own records (4). Megatron, Tyrant's "if
  excess damage would be dealt" is CR 120.4a's modification of the event,
  not 120.10's.

*Item 275* (TR-5c's piece 12, kept in the archive as written). The elision
compares the origin under `SOURCE` (`engine/triggers/placement.rs:129`), and a
delayed trigger's origin is its own registry number, so Protean Hydra hit for
three asks the order of three identical triggers. Comparing the source
uncovers X (CR 107.3n): two delayed triggers of one template can differ in
their creator's X, which the elision reads as reading nothing
(`engine/triggers/binding_reads.rs:265`).

**The pieces**
1. **The prevention arm** (item 16), §3.3's row with the damage arm's name
   for its field:
   ```rust
   /// "Whenever damage that would be dealt to you is prevented" (CR 615.13):
   /// once each time a prevention effect is applied to one recipient's
   /// damage in a batch, however many sources it came from.
   DamageIsPrevented { recipient: DamageRecipient, multiplicity: Multiplicity },
   ```
   - `EventKind::DamagePrevented` leaves `from_record`'s unread list. "That
     many" is the record's `amount`, and an object recipient's event role is
     `DamageRecipient`, as on the damage arm. About 16 exhaustive matches
     take the arm.
   - **Matched at the batch's close**, though announced while it decides.
     The dispatch reads the window, so the trigger sees the board after the
     damage, as CR 603.10 says. The record names no source, so the arm has
     no `source` field.
2. **Selfless Squire**, registered and pooled. {3}{W} 1/1 Human Soldier with
   flash. "When this creature enters, prevent all damage that would be dealt
   to you this turn" is RD-2's shield with one subject (the shape at
   `cards/phase_rd_cards.rs:608`). Its second ability is `DamageIsPrevented {
   recipient: Player(Some(You)), .. }`, putting `TriggeringAmount` +1/+1
   counters on itself.
3. **Excess damage** (CR 120.10):
   - **Decided with the batch.** Between deciding and performing, each
     permanent the decided damage is dealt to gets the damage still lethal
     to it. For a creature that is its toughness less its damage marked, or 1
     if a deathtouch source deals it any; that is `lethal_damage_owed`'s
     reading (`engine/combat/keywords.rs:63`), shared rather than copied. For
     a planeswalker it is its loyalty, and for a creature planeswalker the
     smaller of the two. Battles join with `backlog.md` §2.23, which owns
     their defense counters (CR 120.3h).
   - **Each record takes its share.** The batch keeps the amounts on
     `GameState` while it performs, saved and restored for a nested batch as
     `prevention_allocations` is (`engine/actions.rs:1053`). Each `DealDamage`
     performer takes what of its amount is past what is still lethal after
     the batch's earlier members to the same permanent. So a permanent's
     excess in one event is the sum of its records', which is CR 120.10's
     "those sources together". `DamageDealt` gains `excess: u64` (18 literal
     sites), and `GameAction::DealDamage` does not change. A player's record
     says 0. The log line names the excess where there is some.
   - **The arm.** `TriggerEvent::DamageDealt` gains `excess: bool`, "is
     dealt excess damage", whose "that much" is the excess. It is read once
     per permanent: the damage arm joins `supports_once_per_subject`, keyed
     on the recipient, which "one or more creatures you control deal combat
     damage to a player" will read too. `occurrence_subject` then names an
     object or a player for both families rather than a `CounterSubject`. The
     registry refuses `excess` read per record, or with a player recipient.
   - **The row.** TR-5h's `DealtDamage` row gains the excess, and
     `ObjectFact::WasDealtExcessDamage` reads it: "was dealt excess damage
     this turn", two printed cards, under the size at which an arm ships
     with its type (`engineering-practices.md` §4).
   - ATOM-120.10-001 on a fixture ("whenever a creature is dealt excess
     damage, you gain that much life").
   - *Set aside* (at TR-5c): the permanent's total on each of its records,
     which every reader would have to read once rather than sum; and a
     record of its own per permanent, which is no performed event (§3.12a's
     contract).
4. **Item 275**, as piece 12 wrote it. `SOURCE` compares
   `origin.source_ref()` and the source's last known information each entry
   carries, and `AmountExpr::X` reads a new `TriggerBindingReads::X`, compared
   on `x_value`. Protean Hydra hit for three asks nothing at the end step,
   and a fixture's two delayed draws of X = 2 and X = 5 still ask.

**Planned shapes corrected here, with no decision**
- **§3.3's `DamageIsPrevented { target, multiplicity }`** names its field
  `recipient`, the damage arm's word for the same `DamageRecipient`.
- **§3.3's `DamageDealt` row** gains `excess`, and the damage arm reads
  `OncePerSubject` by its recipient.

**Not in this phase** (each an item at the close-out, slotted with its first
registered card)
- "Prevented this way" (5 cards): the arm's `by`.
- "If excess damage was dealt this way" (4 cards): an amount over a
  resolution's own records.

**Sized.** Code at 1.5–2× plus tests, counted against the tree on
2026-10-18. Tests are counted as tests, at TR-5c's 27 lines each:

| Piece | Code | Tests |
|---|---|---|
| 1–2. The prevention arm, ~16 matches, its event role and amount; Selfless Squire | ~95 | ~160: Squire #1 and #2; two effects on one damage, two triggers; one effect over two attackers, one; a creature's prevention, not "to you"; an object arm's event role |
| 3. Excess: the lethal amounts, shared with trample's; the shares, saved for a nested batch; the record's field at 18 sites; the arm's `excess`, per recipient, and its refusals; the row's excess and its fact; the log | ~130 | ~270: ATOM-120.10-001 with damage marked, deathtouch, two sources, a planeswalker, a creature planeswalker (Aegar #6), a prevention; once per permanent; noncombat only; "was dealt excess damage this turn"; the refusals |
| 4. Item 275 with X | ~15 | ~90: Protean Hydra hit for three; X = 2 and X = 5; a source that left between |
| **Whole: 880–1,000** | ~240 (360–480) | ~520 |

Docs at the close-out, about 300 more: the stub and the archive,
`fuzz-record.md`'s block, §3.3's two rows, and the two items above.

**Predictions for the close-out** (re-stated before `close_out.py`; the
three-arm sitting item 275 carries)
- *The engine arm, against `main`.* Every gameplay row identical, both pools,
  two seats and four, but for the games where Protean Hydra's delayed
  triggers would have asked their order: item 275 drops that prompt, the
  random agent draws less, and those games diverge from there. The three-arm
  sitting isolates it: `main`, the tree without item 275's commit, and the
  tree. No registered card reads a new arm or fact. The event dumps move: a
  `DamageDealt` line names its excess. Instructions per decision within
  +0.2%, from the lethal amounts per damage batch.
- *The shipped arm.* Every dealt game moves once Selfless Squire is pooled,
  and its stress distortion is reported at two seats and four. It is a 1/1
  for four that grows only when damage to its controller is prevented, so a
  small one is expected.

### As landed (2026-10-20)

**Built as designed, then widened at review.** Seven code commits on #263:
the prevention arm with the Squire unregistered, its registration, excess
damage, item 275, the close-out's merges; then at review the names, item
289 with Goblin Negotiation and its registration, and item 290. A rider
renamed `TriggerBindingReads::NOTHING` and its helpers first, and two dev GUI
pictures were redrawn.

**What the build corrected.**
- **`occurrence_subject` names an `EventSubject`**, the replacement
  pipeline's "what an event is about", so the counters' and the damage's
  once-per-subject grouping key on one type.
- **A history row grew from 104 to 112 bytes**: `DealtDamage` is the widest
  `RowFact`, and `excess` adds 8 bytes to every row. A chunk of sixteen is
  1,792 bytes, under 2 KB.
- **The "dealt this way" family is 24 cards**, not 4: `o:"excess damage"
  o:"this way"` finds 25, less Megatron, Tyrant's CR 120.4a replacement.
- **`event_log_test` pinned the prevention record as unread**; the arm's
  commit carries the change.
- **Two dev GUI pictures moved**: Bolt into a 0/1 logs "for 3, 2 excess",
  and item 275 removed the end-step prompt `why_prevented` stopped at.

**What the review changed** (the owner, 2026-10-20).
- **The held-memory gate asserts every game together.** Its decks are built
  from the registry by a seeded picker, so pooling the Squire changed every
  seed's decks, and a 42-turn game holding an ordinary 329 KB read 8,018 B a
  turn alone. The bound was TR-5h's stop-and-report line, not a v1
  requirement.
- **Names:** `take_excess_damage` became `calculate_excess_damage`, which
  only reads, beside `reduce_lethal_damage_left`; `deals_damage_with` became
  `deals_damage_with_keyword`. `TriggerBindingReads`, 30 of 32 bits used,
  asserts its width at compile time.
- **The phase's loose ends, built rather than filed.** Item 289:
  `AmountExpr::DamageDealtThisWay` and `ExcessDamageDealtThisWay` sum the
  damage records the resolution performed, which the event window holds
  until it ends, picked out by the resolution's stamp from
  `ResolvingObject::began_at`, so a bite's damage counts and a rider's does
  not; Goblin Negotiation is the card. Item 290, half of what the model pass
  said: resolution reads an object still in its zone as it is (CR 608.2h),
  so the gaps were a control change's object as the ability triggers (CR
  603.10d) and an unattach's departed end, whose frame the record now
  carries. Item 288 is a design rather than a fix, CR 607.2h's linked
  abilities and the prevented damage's sources, and went to A6s.

**At the close-out.** Two sittings, the second after the review's code.
Every prediction held. pre275 read IDENTICAL on both pools at two seats and
four in both, at +0.13% instructions per decision against `main`; the
engine arm's +0.50% is over games item 275 moved. Five of 200 `performance`
games diverge between the two arms, each after Protean Hydra's delayed
triggers fired together. The audited smoke ran clean; its two turn-limit
games are Words of Worship standoffs, and one game ran 14.8 s on a correct,
exponential Pariah and Protean Hydra loop. The clone floors read 41
allocations and 91.3 KB at worst; the logs held 3,923 B a turn together.

**The model pass** (`plans/references/duplicate-inventory.md` §3.9) named
five copies the branch made, each merged with no game moved (T25–T29), and
T30, fixed at review as item 290.

**What it leaves.** Item 288, at A6s. TR-5g next.

**Size.** +1,543/−193 in code and tests (code +691/−169, of it the two
cards and their registrations +168; tests +852/−24), against 880–1,000 for
the phase as designed: the review's items 289 and 290 and Goblin
Negotiation took it into the 1,500–2,500 band. Twenty-six integration tests
in `phase_tr5d_integration_test`, and a unit test in
`engine::triggers::binding`.

#### TR-5g — the multiplier, tokens and scry — ✅ landed 2026-10-20

*Evicted 2026-10-20 from `plans/triggers-architecture.md` §12, where the heading and a stub remain.*

### The plan as designed (TR-5g, as §12 carried it on 2026-10-20)

*Split from TR-5d at its design (2026-10-18), now in the archive's
"TR-5d", and designed with it for one review. It reads nothing TR-5h or
TR-5d builds, and is built after them. Nothing built.*

**Where it sits.** Three readers, of two records and one static ability.
`TokenCreated` has no arm, and does not say which definition made its token
(§3.12a's `TokenCreated` row). `Scried` has no arm. And CR 603.2d's
"triggers an additional time" changes how many triggers one match makes,
after the dispatch has matched and before it queues
(`engine/triggers/dispatch.rs:703`). §4.4 holds the multiplier's design, and
this re-reads it against the rulings and the tree.

**What the re-read found**

*The multiplier* (CR 603.2d and 603.10; the rulings of Panharmonicon,
Gandalf the White, Teysa Karlov, Yarok, the Desecrated and Elesh Norn,
Mother of Machines; 38 cards, `o:"triggers an additional time"`)
- **What one says.** An event that causes the trigger (Panharmonicon's "an
  artifact or creature entering", Teysa's "a creature dying", Gandalf's "a
  legendary permanent or an artifact entering or leaving the battlefield"),
  whose triggered ability it doubles ("of a permanent you control", Roaming
  Throne's "of another creature you control of the chosen type"), and how
  many more times ("an additional time"). Some name no event: Roaming Throne,
  Twinflame Travelers, Delney.
- **The trigger event, not the arm's words.** "The trigger event doesn't
  have to specifically refer to legendary permanents or to artifacts"
  (Gandalf #9; Teysa #3): "whenever a creature dies" triggers twice when an
  artifact creature dies. And the object is read as it was on the battlefield
  (Gandalf #10, Teysa #6). So the multiplier asks its own event of each
  triggering record, with the frames its own arm reads. A leaving is read off
  the departure's frame, even where the doubled arm, "from anywhere", reads
  the object after the move (item 251; §3.12a's note on Compost).
- **An act is a different event.** "An ability that triggers 'whenever you
  sacrifice a creature' triggers only once" (Teysa #5). The engine writes a
  sacrifice and the death it causes as one `ZoneChange` record, whose act is
  `Sacrifice`, so a record that matches the multiplier's event is not
  enough: the doubled arm must not name an act the multiplier's arm does not.
- **When the multiplier must exist.** A permanent entering with Panharmonicon,
  Panharmonicon included, is doubled (#7; Yarok #3, Elesh Norn #7). A creature
  dying with Teysa, Teysa included, is doubled (Teysa #8), and so is the
  trigger of a permanent that leaves with it (#7). CR 603.10 says which board
  is read: the continuous effects after the event for an ordinary trigger,
  and the abilities as they existed immediately before for one that looks
  back. So the multiplier is read off the board the doubled arm reads.
- **The rest of §4.4 holds.** Only an object's own triggered abilities, never
  a delayed or reflexive one (CR 603.2d's last sentence), a replacement
  (#2) or an "as enters" (#3). Additive (#6). Each instance is its own
  trigger, with its own choices, targets and "may" (#5). "This ability
  triggers only once each turn" is read before the multiplier, and "do this
  only once each turn" after. Linked abilities (#8, #9) are `backlog.md`
  §2.2's.
- **A triggered mana ability is multiplied too.** CR 605.1b makes a trigger
  on a mana ability's activation or resolution that could add mana a mana
  ability, which resolves at once rather than queuing (§4.7). Vorinclex,
  Voice of Hunger's "whenever you tap a land for mana, add one mana of any
  type that land produced" is one, and so is Kinnan, Bonder Prodigy's for a
  nonland permanent. A multiplier that names no event reaches one: Roaming
  Throne naming Praetor beside Vorinclex makes a Forest tap for three, and
  Delney, Streetwise Lookout doubles Kinnan, a 2/2. So the count reaches
  that path as well as the queue. Birgi, God of Storytelling's "whenever you
  cast a spell, add {R}" triggers on a cast, not a mana ability, and "uses
  the stack" (its first ruling): Roaming Throne naming God doubles it
  through the queue.

*Tokens* (CR 111.2, 111.13, 701.7 and 603.12; Ajani, Nacatl Avenger's rulings)
- **Ajani's "when you do" asks for the definition.** Under Doubling Season the
  reflexive trigger triggers once for each Cat Warrior; under Divine
  Visitation, which creates Angels instead, not at all (#8). Both creations
  announce `TokenCreated` records that look alike, so the record says which
  definition made the token (§3.12a's `TokenCreated` row). The 47 "when you
  do" creations (`o:"when you do" o:create o:token`) read the same way.
- **Few triggers read creation itself**: "whenever you create one or more
  tokens" (2 cards) and "whenever you create a [kind] token" (3). "Whenever one
  or more tokens you control enter" (9) is the entry arm over `Token`, never
  this (item 8). A copy of a permanent spell is no creation (CR 111.13) and
  writes no record already.
- **The definition does not reach the emitter.** `announce_token_created`
  (`engine/actions.rs:2074`) has two callers, the entry's token arm and
  `CreateTokenIn`, and neither has it: `create_tokens` lowered it to the
  object's printed `CardData` and proposed the entries
  (`engine/actions.rs:2174`).

*Scry* (CR 701.22; Elrond, Master of Healing's rulings)
- **X is what was looked at.** A scry 3 with two cards in the library makes
  X 2 (#1), which `Scried.looked_at` holds. A scry 0 is no event (CR 701.22b)
  and writes no record already. 21 cards say "whenever you scry"; Celeborn
  the Wise, Elminster, Elrond and Elvish Mariner read the cards looked at.
- **Elrond's X is a count of targets**: "each of up to X target creatures".
  CR 601.2c lets a spell, and through CR 603.3d a triggered ability, have a
  variable number of targets, "defined by the spell's text", and
  `TargetCount` holds only fixed ones (`Exactly(u32)`, `UpTo(u32)`). 26 cards
  say "up to X target ... where X" (`o:"up to X target" o:"where X"`), Elvish
  Mariner among the scry four. A fixture that put X counters on one target
  would test the arm and not the card.
- Elrond #2 is about its second ability, TR-5c's targeting arm, and is
  tested on the same fixture.

**The pieces**
1. **`Effect::TriggerMultiplier`**, the fourth static shape with no layer
   rows, beside `Replacement`, `Restriction` and `CostModification`:
   ```rust
   /// CR 603.2d — "triggers an additional time". Read off the source's
   /// effective ability list as the dispatch queues what a window triggered.
   TriggerMultiplier(Box<TriggerMultiplierDef>),

   pub struct TriggerMultiplierDef {
       /// "If an artifact or creature entering causes": the event a
       /// triggering event must be, or any of several ("entering or
       /// leaving"). `None` for one that names no event (Roaming Throne).
       pub caused_by: Option<TriggerCondition>,
       /// "A triggered ability of a permanent you control": the permanent
       /// whose ability it is, whose "you" is this ability's controller.
       pub abilities_of: ObjectFilter,
       /// "An additional time" is 1.
       pub additional: u32,
   }
   ```
   - **Three legs**, `cost_modifications_for`'s
     (`engine/cost_determination/gather.rs:56`). Printed is a source set that
     `register_static_effects` files on the battlefield and a permanent
     leaving empties. Granted and copied are two summary flags
     (`state/continuous_effects.rs:311`). About ten exhaustive matches over
     `Effect` take the variant.
   - The registry refuses a `State` condition and an `additional` of 0.
2. **The count at the queue.** `queue_matches` asks, of each object's match,
   how many more times it triggers, and queues one `PendingTrigger` more for
   each, with its own `TriggerSeq` and `AbilityTriggered` record. A
   multiplier adds its `additional` when all three hold:
   - **It is on the board the matched arm reads**: the battlefield now; or,
     for an arm that looks back, the battlefield as the window's frames had
     it before the event, through the frames the dispatch reads a look-back
     candidate through (each permanent that left in the window, off its
     departure's frame, and none that entered in it).
   - **The trigger's source is a permanent `abilities_of` admits**, read as
     the candidate frame the dispatch read it through: on the battlefield, or
     off its frame from the battlefield (Teysa #1 and #7). A card's ability
     in another zone is no permanent's.
   - **`caused_by` is `None`, or one of the match's triggering records
     matches it**, through `triggering_events_in` with the multiplier's own
     referents, and the matched arm names no act its arm does not (Teysa #5).
   - **The gate.** A window with no match asks nothing, and a match asks only
     while the source set is non-empty, a flag is set, or a departure frame
     in the window carries the ability. So a board with none pays one set
     check per window that triggered something.
   - **The mana path.** A match that `is_mana_ability` makes a mana ability
     resolves at once, as many times as it triggered (§4.7).
   - **Delayed and reflexive triggers are queued by `queue_delayed`**, which
     the count never reaches (CR 603.2d).
   - **The audit leg** (§4.10). The audit re-derives each match's count with
     every object that could carry the ability swept and no set or flag
     consulted, and the two counts must agree.
3. **Panharmonicon**, registered and pooled. {4} artifact: `caused_by:
   Some(Event(EntersBattlefield { subject: Filter(Or(ByType(Artifact),
   ByType(Creature))), .. }))`, `abilities_of: ByController(You)`,
   `additional: 1`. **Gandalf the White, a fixture**: its third ability, its
   `caused_by` the entering and the leaving (`ZoneChange { from: Battlefield,
   to: None }`) of `Or(BySupertype(Legendary), ByType(Artifact))`. Its flash
   and its permission to cast as though with flash are PM's.
4. **The token arm**, §3.3's row:
   ```rust
   /// "Whenever you create one or more tokens" (CR 111.2, 701.7a). A
   /// reflexive "when you do" after "create a [token]" names its definition,
   /// so a creation a replacement changed to another is not "that action"
   /// (Ajani, Nacatl Avenger's eighth ruling).
   CreatesToken { owner: Option<PlayerRef>, definition: Option<Box<TokenDef>>, multiplicity: Multiplicity },
   ```
   - **The record carries its definition**: `TokenCreated { object_id, owner,
     zone, definition: Arc<TokenDef> }`, 8 bytes inside the 72
     `the_event_keeps_its_size` pins. `create_tokens` keeps each object's
     definition on `GameState` while its entries perform, saved and restored
     for a nested creation as `prevention_allocations` is, and
     `announce_token_created` takes it from there. 17 sites, most of them a
     match with `..`.
   - *Set aside:* the token object keeping its definition, which every clone
     would carry for every token, for a fact read once, at the creation; and
     the definition on the entry proposal, which would grow `GameAction`.
   - `zone` leaves §3.3's row, since no printed trigger asks where a token
     was created. The event role is `Object`, the token, and "that many" is
     one a record.
   - **Ajani, Nacatl Avenger, a fixture**: its 0 ability as an activated
     ability, since loyalty abilities are CP-2's (item 283). Create a 2/1
     white Cat Warrior; then a reflexive `CreatesToken { owner: Some(You),
     definition: Some(cat_warrior), .. }`, with the intervening "if you
     control a red permanent other than" this, deals damage equal to the
     number of creatures you control to any target.
5. **The scry arm**:
   ```rust
   /// "Whenever you scry" (CR 701.22): one record per scry, and none for a
   /// scry 0 (CR 701.22b). "The number of cards looked at while scrying this
   /// way" is the record's `looked_at` (Elrond, Master of Healing's first
   /// ruling).
   Scries { player: Option<PlayerRef> },
   ```
   "That many" is `looked_at`. A scry names no object, so it has no event
   role.
6. **A count of targets the text defines** (CR 601.2c, 603.3d):
   ```rust
   /// "Up to X target creatures, where X is ...": read as the targets are
   /// announced, off the trigger's binding or the spell's announced X.
   UpToAmount(AmountExpr),
   ```
   `TargetCount` stops being `Copy`. Its two consumers
   (`engine/put_on_stack.rs:368`, `engine/targeting.rs:465`) read the amount
   as the announcement starts, and the validation reads the count announced.
   **Elrond, Master of Healing, a fixture**: both abilities, each with its
   ruling.
   - *Set aside:* Elrond's fixture with X as a count of counters on one
     target, about 100 lines less, which tests the arm and leaves the count
     to its first registered card.

**Planned shapes corrected here, with no decision**
- **§4.4's `TriggerMultiplierDef { caused_by: TriggerEvent, source:
  ObjectFilter, additional }`.** `caused_by` takes a `TriggerCondition`, for
  Gandalf's two events, and is optional. `source` is named `abilities_of`,
  since it filters the doubled ability's permanent and not the multiplier's.
- **§3.3's `CreatesToken { owner, zone, kind: Option<TokenKind>,
  multiplicity }`** drops `zone`, and names `kind` `definition`, which the G2
  amendment says it is.

**Not in this phase**
- A token arm over the token's characteristics ("whenever you create an
  artifact token", 3 cards), with its first card.
- Strict Proctor's `AbilityTriggers.caused_by` asks only the record's kind
  (`engine/triggers/dispatch.rs:1846`). The multiplier's reading would serve
  it, and CP-1 builds the card.
- Elesh Norn, Mother of Machines' "permanents entering don't cause abilities
  of permanents your opponents control to trigger" is a "can't" on
  triggering (CR 101.2), not a multiplier, with its first card.

**Sized**

| Piece | Code | Tests |
|---|---|---|
| 1. The variant, its def, ~10 matches, the three legs | ~55 | — |
| 2. The count: the board each arm reads, the three tests, the sum, the instances, the mana path, the audit leg | ~110 | — |
| 3. Panharmonicon; Gandalf's fixture; 1–3 tested | ~25 | ~460: ATOM-603.2d-001; Panharmonicon #1–#7; Gandalf #9 against a "from anywhere" arm, and #8 dying beside an artifact; a sacrifice (Teysa #5's shape); a capped trigger; "do this only once" doubled; a delayed trigger not; a triggered mana ability; the granted and copied legs; an opponent's Panharmonicon |
| 4. The token arm, the record's definition, the creation's table; Ajani's fixture | ~75 | ~160: Ajani #7 and #8 (Doubling Season, Divine Visitation); the "if" failing; "one or more tokens" once |
| 5. The scry arm; Elrond's fixture | ~40 | ~130: Elrond #1 and #2; a scry 0; an opponent's scry |
| 6. The count of targets | ~50 | ~50: fewer than X chosen; an X of 0 |
| **Whole: 1,330–1,510** | ~355 (530–710) | ~800 |

Docs at the close-out, about 350: the stub and the archive,
`fuzz-record.md`'s block, §3.3's two rows, and §4.4's paragraph.

**Predictions for the close-out** (re-stated before `close_out.py`)
- *The engine arm, against `main`.* Every gameplay row identical, both pools,
  two seats and four: no registered card names a multiplier, a token's
  definition or a scry until the registrations. Instructions per decision
  within ±0.3%, from one set check per window that triggered.
- *The shipped arm.* Every dealt game moves once Panharmonicon is pooled, and
  its stress distortion is reported. It doubles every entering trigger its
  controller's permanents have, so more triggers per game are expected.

**The three together.** TR-5h, TR-5d, then TR-5g, at 1,630–1,920, 880–1,000
and 1,330–1,510. TR-5d and TR-5g built as one would be 2,210–2,510, at the
band's edge, and TR-5c landed 11% past its sizing's top, TR-5b 14% and TR-5f
28%. So they stay apart, and each tracks the band at each commit and stops
at 2,500 to say so. Item 275 goes with TR-5d, the first to move the random
agent's stream, since TR-5h registers nothing. The brief's cards split with
them: Selfless Squire and ATOM-120.10-001 are TR-5d's, and Panharmonicon
and ATOM-603.2d-001 TR-5g's.

### As landed (2026-10-20)

**Built as designed.** Five engine commits on #264, then Panharmonicon's
registration and its pooling as two, so its stress distortion was read
registered and unpooled first: the variant and its three legs, the count at
the queue, Panharmonicon unregistered with the count's tests, the token and
scry arms with the record's definition, and the count of targets with
Elrond's fixture.

**What the build corrected.**
- **The count reads the dispatch's own candidates.** `match_candidates` now
  says which candidate each match is of, and the count reads the trigger's
  source through it: on the battlefield, or off its frame from it (Teysa
  Karlov's first and seventh rulings). `find_matches`' candidate building is
  split into `lists_before`, `push_live_candidates` and
  `departure_candidate`, which the multiplier candidates share, and
  `AnsweredArms::admits` is the one test the match and the board read. A
  look-back snapshot holds the multipliers too, so a survivor's is read as
  it was before the batch.
- **A capped ability queues once, whatever the count**, and "do this only
  once" doubled acts once and asks nothing the second time.
- **The resolution's `CountOf` was never built.** "The number of creatures
  you control" errored at resolution; the layer pass alone counted it. Both
  now read `Selector::permanent_filter`.
- **A token's entry has one maker.** Two fixtures (`phase_rc4`'s Containment
  Priest, `phase_rc`'s token entry) proposed a token's entry by hand; each
  now creates it, with the claim it made.
- **A reflexive trigger needs a resolution**, so Ajani's fixture is an
  activated ability resolved off the stack, its loyalty CP-2's.
- **The ask enforces the announced bound**: a provider that returns more
  targets than X breaks its contract, so "more than X" is a `should_panic`
  test on the bound rather than a refusal.
- **`check_rulings.py` matched a card only by its full name**, so no
  double-faced card could be cited by a face's; it now maps each face's name
  to its card.

**At the close-out.** Two sittings, the second after the review's code.
The engine arm read IDENTICAL on both pools at two seats and four in both,
at +0.21% and then +0.18% instructions per decision; the shipped arm before
and after the review played the same games, cost rows included. The audited
smoke ran clean. `history_held_test` read 3,877 B a turn. The release clone
test failed on shipped's decks: `performance` 12353 at four seats ends with
31 Sphere Grid triggers waiting, one allocation each, 82 in all against 64,
and the same game clones at 68 and 77 at two trigger-ordering prompts
mid-turn. A pending trigger's binding holds its events in a `Vec`; no
multiplier is involved, and `roadmap-v2.md` B10 names the class.

**The model pass** (`plans/references/duplicate-inventory.md` §3.10) named
12 facts and a mirror. Four already disagreed, all in the branch's new code,
and were fixed (T31-T34), with the mirror; four were merged (T35-T38); the
rest are kept there with their reasons.

**What the review changed** (the owner, 2026-10-20). The clone reading is
B10's, a batch of triggers mid-turn, and storm's copies are the same shape,
so keeping one triggering event inline would have fixed one board and not
the class. A finished game drops its waiting triggers instead (CR 104.1:
nobody receives priority to put them on the stack), through
`GameState::end_game`, which became the result's one writer; the test's end
checkpoint then reads the board, 50 allocations for 12353. The readings and
the per-object costs went to `codebase-state.md` item 183, and B10's design
moved up to right after TR-5g.

**What it leaves.** B10's design, next; no new item, and the intake list was
empty.

**Size.** +1,781/−251 in code and tests (code +905/−201, of it
Panharmonicon and its registration +140; tests +876/−50), against
1,330–1,510 as designed: the review's four fixes and the resolution's
`CountOf` took it past the design and inside the 1,500–2,500 band.
Thirty-two integration tests in `phase_tr5g_integration_test`, four unit
tests, and two fixtures moved onto the creation.
