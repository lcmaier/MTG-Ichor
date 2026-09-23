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
