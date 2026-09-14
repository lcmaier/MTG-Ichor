# RE-5 review — captured 2026-09-14, from the owner's read of PR #134

Seven findings, captured before any is fixed (`engineering-practices.md` §4).
Triage: **fix** / **doc** / **defer** / **design**. Close one theme per
session, cold from this file; delete the file in the PR that lands the last
theme.

## Theme A — the putter is an authored fact, and a rule is not closed by an empty Scryfall query (R2, R4, R5) — ✅ closed 2026-09-14

*Built as triaged, one commit: `EntryCounters` / `EntryCountersTemplate` with
`by`, the `(kind, putter)` merge, `putter_of` and `resolve_putter`, `by` on
the two primitives; `AddCounters` / `RemoveCounters` in place of
`CounterChange { adding }`; item 43 reopened and closed as built; §11 item 83
rewritten; the §4 rule and the Deferred Migrations head sentence; four tests
in, one out. Re-measured: see the archive's "Reviewed" paragraph.*

**R2 — item 43 was closed on the wrong premise.** "No printed effect names
the putter" was the finding, and the owner's rule is that it does not close
anything: a card can be printed next set, and custom card creation is a
post-v1 goal, so any author can write the rule's first sentence. **Bold
Plagiarist** is the card that shows the shape today — "Whenever an opponent
puts one or more counters on a creature they control, *they* put the same
number and kind of counters on this creature", ruling: "Your opponent places
the counters on both permanents" — a trigger (item 6's) whose *effect* names
a putter who is not the effect's controller and not the object's controller.
So the putter has to be authorable on a proposal, not only at an entry, and
`Primitive::AddCounters` writing `by: ctx.controller` is the same
closed-on-absence shortcut one level down. **Fix (design first):**
`Primitive::AddCounters` and `GetCounters` gain `by: Option<PlayerRef>`
(`None` = the effect's controller, CR 122.6a's shape on a proposal);
`EnterModsTemplate.counters` entries gain `Option<PlayerRef>`,
`EnterMods.counters` entries `Option<PlayerId>`, `merge` keyed on `(kind,
player)`, the door reading `by.unwrap_or(controller)`; a fixture card
exercising each (an "as this enters, target opponent puts two -1/-1
counters on it" fixture; Bold Plagiarist's effect as a resolution fixture
under Vorinclex). Reopen item 43 as owed-and-built rather than closed-on-
absence; §11 item 83 rewritten. ~100 lines with tests. **Rule finding for
`engineering-practices.md`:** the CR is the customer; a printed card is the
*test*. A rule-stated facility with no printed card gets a fixture test, not
a closed item — the reachability line says "no printed producer", never
"nothing owed".

**R4 — `CounterChange.adding: bool` is the one place the growth contract is
broken, and the rewrite cannot infer it.** The pattern describes the event
watched, the rewrite what to do, and `Prevent`, `Instead(RemoveCounters
FromAffected)` and every `Restriction::Event` carry no `Plus` to read the
direction from. The honest fix is the contract's own: two arms,
`AddCounters { counter, by }` and `RemoveCounters { counter }`, restoring
one arm per `GameAction` variant and deleting the "`by` must be `None` when
`adding` is false" caveat outright. **Fix**, ~40 lines: three
`pattern_watches` arms, `reads_the_amount`, six cards, two RC-4 fixtures,
the RE-5 tests, the type's doc and §3.2a's "sixteen arms" paragraph.

**R5 — "nothing prints a remover" could become false.** Scryfall
2026-09-14: no printed "would remove" replacement and no "if an opponent
would remove" (the one "counters can't be removed" is Fear of Sleep
Paralysis, RS's). With R4's split the trap goes: `RemoveCounters` has no
`by`, so the day a card prints a remover it is a field added with that
card and a compile error at every reader until it is answered, not a
pattern that silently matches nothing. `RemoveCounters` on the action
would gain `by: Option<PlayerId>` the same day — damage removing loyalty
has no remover, a cost's payer is one. **Answered by R4**; the test
`a_removal_pattern_naming_a_putter_matches_nothing` is deleted with it.

## Theme B — the needless prompts (R1) — ✅ closed 2026-09-14

*Built in this PR after all: `ordering_cannot_change_outcome` is the
commutation table with the kind axis, `classify` exhaustive over the rewrite
arms, `commutes` the table, `check_order_invariance` dispatching on the
chosen member's cell. Two Scales, Scales beside Biomancer on a present kind,
disjoint-kind arithmetic and Visitation beside Parallel Lives ask nothing;
a plus beside an `EnterWith` writing a new kind stays real. §2.29 graduated.
Re-measured: see the archive's "Reviewed" paragraph.*

**R1 — Scales beside Scales, and Scales beside Biomancer on one kind, commute
and are asked.** Yes: both have one outcome, and §11 item 19's rule is that
a choice with one outcome is not put to a player — so a needless prompt is
a violation of a rule this project keeps, not a nicety. RE-5 recorded
rather than built, on §2.29's "wants a session of its own"; the owner's
read is that the session is now. **Design:** the shape is not "all `Plus`":
a plus beside an `EnterWith` commutes only while every kind the plus
matches is already in the mods before either applies (a kind the `EnterWith`
adds afterwards is not raised, CR 614.5). So the premise is per `(rewrite
class, kind)`, which is §2.29's table with the kind axis it asked for:
factor the common clauses, classify each member as multiplicative /
additive / mods-adding / absorbing exit / idempotent substitute with the
kind set it touches, and a bucket is one-outcome iff every pair commutes on
every kind both touch. ~150 lines, one debug check per class, and
`Replacement prompts` on the pooled `performance` arm should fall back
toward 0.74 from 1.14. Decide in this review whether it lands here or as
its own PR; the recommendation is its own PR off this branch, since it is
the predicate's fourth correction and §4.1 says to ask the standing question
of every clause.

## Theme C — the store, and costs (R3, R6) — ✅ closed 2026-09-14

*R3 is `backlog.md` §2.6's new bullet; R6 is a sentence on the performer arm, the player store and the test.*

**R3 — is `CounterSubject` extensible to Skullbriar and suspend?** The
*subject* is: `Object(ObjectId)` names any object, and CR 122.1a/b already
speak of "a creature card in a zone other than the battlefield". What is
not is the *store*: counters live on `PermanentState`, the performer
refuses an object off the battlefield, and `move_object` drops the entity,
which is CR 122.2 by construction. Suspend (time counters on an exiled
card), Skullbriar ("counters remain … as it moves"), Darigaaz's egg
counters and CR 122.1a's off-battlefield +1/+1 all want counters on the
`GameObject`, with `PermanentState` keeping only CR 613.7c's timestamps —
and Skullbriar's rulings say the retained counters are not "placed", so no
`AddCounters` is proposed for them and Doubling Season does not see them.
**Doc/defer:** a `backlog.md` §2.6 line (suspend is CR 702.62) naming the
store move, ~60 lines when the first exiled-with-counters card lands; the
subject enum needs nothing.

**R6 — as-much-as-possible vs "can't pay an impossible cost".** Two rules,
two sites. CR 701.2 is about an *effect's* instruction — "remove three
counters" from a permanent with one removes one — and that is the
performer's clamp. CR 118.3 is about a *cost*, and a cost is validated
before it is paid (`Cost::Sacrifice` counts candidates; `Cost::
RemoveCounters` is the `Err("not yet implemented")` arm in `costs.rs` that
will count the same way, and paying {E} — "you can't pay more energy
counters than you have" — is that arm's player twin). A validated cost
never reaches the clamp. **Doc:** say so on the performer arm and on the
two tests' docs, and name the validation arm as the cost's site.

## Theme D — the test that is a card and not a rule (R7) — ✅ closed 2026-09-14

*Deleted; Live Fast's doc points at the engine test.*

**R7 — the Live Fast card test.** The engine test earlier in the file
covers `GetCounters`, the map and the log line; the card's other two
instructions are RE-2's and RA's; and card-as-printed tests are card
breadth's, when fixtures move to set folders. Vorinclex's player-half test
still uses Live Fast as its producer, which is the card doing a job.
**Fix:** delete `live_fast_gives_its_caster_two_energy`; the card's doc
points at the engine test instead.

## Second pass (2026-09-14), on theme A — both closed the same day

**R8 — `CountersPut` or `CountersAdded`?** Neither: every other arm bears its
`GameAction`'s name (`DealDamage`, `CreateTokens`, `PlayerLoses`), so the
pair is `EventPattern::AddCounters` and `EventPattern::RemoveCounters`, the
same words as the action and the primitive, and the collision is the
convention rather than a headache. **Fixed.**

**R9 — `None | Some(You)` in `resolve_putter`.** Two names for one player; a
smell. The primitives' `by` is a plain `PlayerRef` and every printed one-shot
writes `You`. The template's `Option` stays, since there `None` is a
different player — the entering object's controller, CR 122.6a's default,
which no `PlayerRef` can name. **Fixed.**

## Order

A first (it changes the type R4 and R5 read and the door R1's table reads),
then D and C in one sitting, then B as its own decision. Each theme
re-measures only if it touches the engine: A and B do; C and D do not.
